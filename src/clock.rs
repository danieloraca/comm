use std::{env, sync::OnceLock};

use chrono::Utc;
use chrono_tz::Tz;

const DEFAULT_TIMEZONE: Tz = chrono_tz::Europe::London;
const DEFAULT_TIMEZONE_NAME: &str = "Europe/London";

static APP_TIMEZONE: OnceLock<Tz> = OnceLock::new();

pub fn configured_timezone() -> Tz {
    *APP_TIMEZONE.get_or_init(|| match env::var("COMM_TIMEZONE") {
        Ok(name) if !name.trim().is_empty() => name.trim().parse().unwrap_or_else(|error| {
            eprintln!(
                "invalid COMM_TIMEZONE `{}`: {error}; falling back to {DEFAULT_TIMEZONE_NAME}",
                name.trim()
            );
            DEFAULT_TIMEZONE
        }),
        _ => DEFAULT_TIMEZONE,
    })
}

pub fn activity_timestamp() -> String {
    Utc::now()
        .with_timezone(&configured_timezone())
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}
