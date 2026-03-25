use crate::types::Config;

#[derive(Debug)]
pub struct AppError {
    pub config: Config,
    pub message: String,
}

impl AppError {
    pub fn new(config: Config, message: &str) -> Self {
        Self { config, message: message.to_string() }
    }
}
