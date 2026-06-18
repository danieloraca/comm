use std::{
    env,
    io::{self, Write},
    process::Command,
};

use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::{
    auth::AppState,
    clock::activity_timestamp,
    link_preview::{self, LinkPreview},
    store::{ActivityLog, ReadReceipt, StoredAttachment, StoredMessage},
};

const MAX_MESSAGE_LEN: usize = 2_000;

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type")]
pub enum ServerEvent {
    #[serde(rename = "message")]
    Message(ChatMessage),
    #[serde(rename = "delete_for_me")]
    DeleteForMe {
        id: i64,
        #[serde(skip)]
        username: String,
    },
    #[serde(rename = "delete_for_everyone")]
    DeleteForEveryone { id: i64 },
    #[serde(rename = "typing")]
    Typing { from: String, is_typing: bool },
    #[serde(rename = "presence")]
    Presence { username: String, online: bool },
    #[serde(rename = "read")]
    Read {
        id: i64,
        by: String,
        read_at: String,
    },
    #[serde(rename = "activity_logs")]
    ActivityLogs {
        #[serde(skip)]
        username: String,
        logs: Vec<ActivityLogEntry>,
    },
    #[serde(rename = "link_preview")]
    LinkPreview { id: i64, preview: ChatLinkPreview },
}

#[derive(Clone, Debug, Serialize)]
pub struct ChatMessage {
    id: i64,
    from: String,
    body: String,
    attachment: Option<ChatAttachment>,
    link_preview: Option<ChatLinkPreview>,
    created_at: String,
    read_at: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ChatAttachment {
    id: i64,
    mime_type: String,
    original_name: Option<String>,
    size_bytes: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct ChatLinkPreview {
    url: String,
    title: Option<String>,
    description: Option<String>,
    site_name: Option<String>,
    image_url: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct ActivityLogEntry {
    occurred_at: String,
    username: String,
    action: String,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientEvent {
    #[serde(rename = "message")]
    Message {
        body: String,
        attachment_id: Option<i64>,
    },
    #[serde(rename = "delete_for_me")]
    DeleteForMe { id: i64 },
    #[serde(rename = "delete_for_everyone")]
    DeleteForEveryone { id: i64 },
    #[serde(rename = "typing")]
    Typing { is_typing: bool },
    #[serde(rename = "read")]
    Read { id: i64 },
    #[serde(rename = "activity_logs")]
    ActivityLogs,
}

pub async fn websocket(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    let Some(username) = state.authenticated_user(&headers) else {
        return axum::http::StatusCode::UNAUTHORIZED.into_response();
    };

    ws.on_upgrade(move |socket| handle_socket(state, username, socket))
}

async fn handle_socket(state: AppState, username: String, socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();
    let chat_tx = state.chat_sender();
    let mut chat_rx = chat_tx.subscribe();
    let store = state.message_store();
    let send_username = username.clone();

    match store.recent_messages(&username).await {
        Ok(history) => {
            for message in history
                .into_iter()
                .map(ChatMessage::from)
                .map(ServerEvent::Message)
            {
                let Ok(payload) = serde_json::to_string(&message) else {
                    continue;
                };

                if sender.send(Message::Text(payload.into())).await.is_err() {
                    return;
                }
            }
        }
        Err(error) => eprintln!("failed to load message history for {username}: {error}"),
    }

    let became_online = state.connect_user(&username);
    for online_username in state.online_users() {
        let event = ServerEvent::Presence {
            username: online_username,
            online: true,
        };
        let Ok(payload) = serde_json::to_string(&event) else {
            continue;
        };

        if sender.send(Message::Text(payload.into())).await.is_err() {
            state.disconnect_user(&username);
            return;
        }
    }

    if became_online {
        log_presence(&store, &username, "online").await;
        let _ = chat_tx.send(ServerEvent::Presence {
            username: username.clone(),
            online: true,
        });
    }

    let send_task = tokio::spawn(async move {
        loop {
            match chat_rx.recv().await {
                Ok(event) => {
                    if matches!(
                        &event,
                        ServerEvent::DeleteForMe {
                            username: target_username,
                            ..
                        } if target_username != &send_username
                    ) {
                        continue;
                    }

                    if matches!(
                        &event,
                        ServerEvent::ActivityLogs {
                            username: target_username,
                            ..
                        } if target_username != &send_username
                    ) {
                        continue;
                    }

                    if matches!(&event, ServerEvent::Typing { from, .. } if from == &send_username)
                    {
                        continue;
                    }

                    let Ok(payload) = serde_json::to_string(&event) else {
                        continue;
                    };

                    if sender.send(Message::Text(payload.into())).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    while let Some(Ok(message)) = receiver.next().await {
        match message {
            Message::Text(body) => {
                let Ok(event) = serde_json::from_str::<ClientEvent>(&body) else {
                    continue;
                };

                match event {
                    ClientEvent::Message {
                        body,
                        attachment_id,
                    } => {
                        let body = body.trim();

                        if (body.is_empty() && attachment_id.is_none())
                            || body.len() > MAX_MESSAGE_LEN
                        {
                            continue;
                        }

                        let Ok(stored) = store.save_message(&username, body, attachment_id).await
                        else {
                            continue;
                        };
                        let message_id = stored.id;
                        let body = body.to_owned();

                        let _ = chat_tx.send(ServerEvent::Message(ChatMessage::from(stored)));

                        if let Some(url) = link_preview::first_url(&body) {
                            let store = store.clone();
                            let chat_tx = chat_tx.clone();
                            tokio::spawn(async move {
                                let Some(preview) = link_preview::fetch(&url).await else {
                                    return;
                                };

                                let Ok(true) = store
                                    .update_message_link_preview(message_id, &preview)
                                    .await
                                else {
                                    return;
                                };

                                let _ = chat_tx.send(ServerEvent::LinkPreview {
                                    id: message_id,
                                    preview: ChatLinkPreview::from(preview),
                                });
                            });
                        }
                    }
                    ClientEvent::DeleteForMe { id } => {
                        let Ok(true) = store.hide_message_for_user(&username, id).await else {
                            continue;
                        };

                        let _ = chat_tx.send(ServerEvent::DeleteForMe {
                            id,
                            username: username.clone(),
                        });
                    }
                    ClientEvent::DeleteForEveryone { id } => {
                        let Ok(true) = store.delete_message_for_everyone(&username, id).await
                        else {
                            continue;
                        };

                        let _ = chat_tx.send(ServerEvent::DeleteForEveryone { id });
                    }
                    ClientEvent::Typing { is_typing } => {
                        let _ = chat_tx.send(ServerEvent::Typing {
                            from: username.clone(),
                            is_typing,
                        });
                    }
                    ClientEvent::Read { id } => {
                        let Ok(Some(receipt)) = store.mark_message_read(&username, id).await else {
                            continue;
                        };

                        let _ = chat_tx.send(ServerEvent::from(receipt));
                    }
                    ClientEvent::ActivityLogs => {
                        let Ok(logs) = store.recent_activity_logs().await else {
                            continue;
                        };

                        let _ = chat_tx.send(ServerEvent::ActivityLogs {
                            username: username.clone(),
                            logs: logs.into_iter().map(ActivityLogEntry::from).collect(),
                        });
                    }
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    send_task.abort();

    if state.disconnect_user(&username) {
        log_presence(&store, &username, "offline").await;
        let _ = chat_tx.send(ServerEvent::Presence {
            username,
            online: false,
        });
    }
}

async fn log_presence(store: &crate::store::MessageStore, username: &str, status: &str) {
    let _ = store.record_activity_log(username, status).await;
    play_presence_sound();
    println!("{} user {status}: {username}", activity_timestamp());
}

fn play_presence_sound() {
    if let Ok(sound_file) = env::var("COMM_PRESENCE_SOUND")
        && !sound_file.trim().is_empty()
    {
        let _ = Command::new("afplay").arg(sound_file).spawn();
        return;
    }

    print!("\x07");
    let _ = io::stdout().flush();
}

impl From<ActivityLog> for ActivityLogEntry {
    fn from(log: ActivityLog) -> Self {
        Self {
            occurred_at: log.occurred_at,
            username: log.username,
            action: log.action,
        }
    }
}

impl From<StoredMessage> for ChatMessage {
    fn from(message: StoredMessage) -> Self {
        Self {
            id: message.id,
            from: message.sender,
            body: message.body,
            attachment: message.attachment.map(ChatAttachment::from),
            link_preview: message.link_preview.map(ChatLinkPreview::from),
            created_at: message.created_at,
            read_at: message.read_at,
        }
    }
}

impl From<StoredAttachment> for ChatAttachment {
    fn from(attachment: StoredAttachment) -> Self {
        Self {
            id: attachment.id,
            mime_type: attachment.mime_type,
            original_name: attachment.original_name,
            size_bytes: attachment.size_bytes,
        }
    }
}

impl From<LinkPreview> for ChatLinkPreview {
    fn from(preview: LinkPreview) -> Self {
        Self {
            url: preview.url,
            title: preview.title,
            description: preview.description,
            site_name: preview.site_name,
            image_url: preview.image_url,
        }
    }
}

impl From<ReadReceipt> for ServerEvent {
    fn from(receipt: ReadReceipt) -> Self {
        Self::Read {
            id: receipt.message_id,
            by: receipt.username,
            read_at: receipt.read_at,
        }
    }
}
