use std::{env, process::Stdio};

use tokio::{process::Command, time};

const DEFAULT_NTFY_SERVER: &str = "https://ntfy.sh";
const NTFY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

pub fn notify_presence(username: &str, status: &str) {
    let Some(config) = NtfyConfig::from_env() else {
        return;
    };

    let body = format!(
        "{} {}",
        notification_username(username),
        notification_status(status)
    );
    tokio::spawn(async move {
        if let Err(error) = publish_ntfy(&config.url, &body).await {
            eprintln!("failed to send ntfy presence notification: {error}");
        }
    });
}

struct NtfyConfig {
    url: String,
}

impl NtfyConfig {
    fn from_env() -> Option<Self> {
        let topic = env::var("COMM_NTFY_TOPIC").ok()?;
        let server =
            env::var("COMM_NTFY_SERVER").unwrap_or_else(|_| DEFAULT_NTFY_SERVER.to_owned());

        Self::from_values(&server, &topic)
    }

    fn from_values(server: &str, topic: &str) -> Option<Self> {
        let topic = topic.trim().trim_matches('/');
        let server = server.trim().trim_end_matches('/');

        if topic.is_empty() || topic.contains("://") {
            return None;
        }
        if server.is_empty() {
            return None;
        }

        Some(Self {
            url: format!("{server}/{topic}"),
        })
    }
}

fn notification_username(username: &str) -> String {
    username
        .strip_prefix('u')
        .filter(|suffix| {
            !suffix.is_empty() && suffix.chars().all(|character| character.is_ascii_digit())
        })
        .map(|suffix| format!("m{suffix}"))
        .unwrap_or_else(|| username.to_owned())
}

fn notification_status(status: &str) -> &str {
    match status {
        "online" | "in" => "up",
        "offline" | "out" => "down",
        other => other,
    }
}

async fn publish_ntfy(url: &str, body: &str) -> Result<(), String> {
    let status = time::timeout(
        NTFY_TIMEOUT,
        Command::new("curl")
            .arg("--silent")
            .arg("--show-error")
            .arg("--fail")
            .arg("--max-time")
            .arg("5")
            .arg("--data-binary")
            .arg(body)
            .arg("-H")
            .arg("Tags: bust_in_silhouette")
            .arg(url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status(),
    )
    .await
    .map_err(|_| "request timed out".to_owned())?
    .map_err(|error| error.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("curl exited with status {status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::{NtfyConfig, notification_status, notification_username};

    #[test]
    fn ntfy_config_uses_default_server() {
        let config = NtfyConfig::from_values("https://ntfy.sh", "comm-secret-topic").unwrap();

        assert_eq!(config.url, "https://ntfy.sh/comm-secret-topic");
    }

    #[test]
    fn ntfy_config_trims_server_and_topic() {
        let config =
            NtfyConfig::from_values("https://ntfy.example.test/", "/comm-secret-topic/").unwrap();

        assert_eq!(config.url, "https://ntfy.example.test/comm-secret-topic");
    }

    #[test]
    fn ntfy_config_rejects_empty_topic() {
        assert!(NtfyConfig::from_values("https://ntfy.sh", " ").is_none());
    }

    #[test]
    fn ntfy_config_rejects_topic_that_looks_like_url() {
        assert!(NtfyConfig::from_values("https://ntfy.sh", "https://example.test/topic").is_none());
    }

    #[test]
    fn notification_username_maps_numbered_users() {
        assert_eq!(notification_username("u1"), "m1");
        assert_eq!(notification_username("u2"), "m2");
    }

    #[test]
    fn notification_username_keeps_other_usernames() {
        assert_eq!(notification_username("alice"), "alice");
        assert_eq!(notification_username("u"), "u");
    }

    #[test]
    fn notification_status_maps_in_out_to_up_down() {
        assert_eq!(notification_status("online"), "up");
        assert_eq!(notification_status("offline"), "down");
        assert_eq!(notification_status("in"), "up");
        assert_eq!(notification_status("out"), "down");
    }
}
