use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub update_interval: Duration,
    pub default_save_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            update_interval: Duration::from_secs(1),
            default_save_path: String::from("stats"),
        }
    }
}

impl Config {
    pub fn new(update_interval_secs: u64) -> Self {
        Self {
            update_interval: Duration::from_secs(update_interval_secs),
            ..Default::default()
        }
    }
}
