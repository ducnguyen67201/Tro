use std::sync::Arc;

use crate::{config::AppConfig, repositories::Repository, services::openai::Provider};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub repository: Arc<dyn Repository>,
    pub provider: Arc<dyn Provider>,
}

impl AppState {
    pub fn new(
        config: Arc<AppConfig>,
        repository: Arc<dyn Repository>,
        provider: Arc<dyn Provider>,
    ) -> Self {
        Self {
            config,
            repository,
            provider,
        }
    }
}
