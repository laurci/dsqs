use anyhow::{bail, Result};

use crate::queue::QueueBehavior;

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

pub struct Config {
    pub server: ServerConfig,
    pub queue_behavior: QueueBehavior,
}

impl Config {
    pub fn load_from_env() -> Result<Self> {
        let host = std::env::var("HOST").unwrap_or("0.0.0.0".to_owned());
        let port = std::env::var("PORT").unwrap_or("6841".to_owned()).parse()?;

        let queue_behavior = std::env::var("QUEUE_BEHAVIOR").unwrap_or("queue".to_owned());
        let queue_max_size: Option<usize> = if let Ok(max_size) = std::env::var("QUEUE_MAX_SIZE") {
            Some(max_size.parse()?)
        } else {
            None
        };

        let queue_behavior = match queue_behavior.as_str() {
            "queue" => QueueBehavior::QueueMessages {
                max_size: queue_max_size,
            },
            "drop" => QueueBehavior::DropMessages,
            _ => bail!(
                "Invalid queue behavior '{}'. Valid options are 'queue' (default) or 'drop'.",
                queue_behavior
            ),
        };

        Ok(Config {
            server: ServerConfig { host, port },
            queue_behavior,
        })
    }
}
