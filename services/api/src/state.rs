use std::sync::Arc;

use crate::{
    config::AppConfig,
    repositories::Repository,
    services::{openai::Provider, tutor::TutorProvider},
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub repository: Arc<dyn Repository>,
    pub provider: Arc<dyn Provider>,
    pub tutor_provider: Arc<dyn TutorProvider>,
}

impl AppState {
    pub fn new(
        config: Arc<AppConfig>,
        repository: Arc<dyn Repository>,
        provider: Arc<dyn Provider>,
        tutor_provider: Arc<dyn TutorProvider>,
    ) -> Self {
        Self {
            config,
            repository,
            provider,
            tutor_provider,
        }
    }
}
