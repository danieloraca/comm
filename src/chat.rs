use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::sync::broadcast;

use crate::{auth::AppState, store::StoredMessage};

const MAX_MESSAGE_LEN: usize = 2_000;

#[derive(Clone, Debug, Serialize)]
pub struct ChatMessage {
    id: i64,
    from: String,
    body: String,
    created_at: String,
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

    if let Ok(history) = store.recent_messages().await {
        for message in history.into_iter().map(ChatMessage::from) {
            let Ok(payload) = serde_json::to_string(&message) else {
                continue;
            };

            if sender.send(Message::Text(payload.into())).await.is_err() {
                return;
            }
        }
    }

    let send_task = tokio::spawn(async move {
        loop {
            match chat_rx.recv().await {
                Ok(message) => {
                    let Ok(payload) = serde_json::to_string(&message) else {
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
                let body = body.trim();

                if body.is_empty() || body.len() > MAX_MESSAGE_LEN {
                    continue;
                }

                let Ok(stored) = store.save_message(&username, body).await else {
                    continue;
                };

                let _ = chat_tx.send(ChatMessage::from(stored));
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    send_task.abort();
}

impl From<StoredMessage> for ChatMessage {
    fn from(message: StoredMessage) -> Self {
        Self {
            id: message.id,
            from: message.sender,
            body: message.body,
            created_at: message.created_at,
        }
    }
}
