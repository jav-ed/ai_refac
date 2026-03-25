use crate::types::Config;
use crate::models::user::UserModel;

pub struct UserService {
    config: Config,
}

impl UserService {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn create_model(&self) -> UserModel {
        UserModel::new(self.config.clone())
    }
}
