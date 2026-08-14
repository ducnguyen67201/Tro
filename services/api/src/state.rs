use std::sync::Arc;

use crate::{
    config::AppConfig,
    repositories::Repository,
    services::{
        google_auth::{GoogleIdentityProvider, IdentityProvider},
        openai::Provider,
        tutor::TutorProvider,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub repository: Arc<dyn Repository>,
    pub provider: Arc<dyn Provider>,
    pub tutor_provider: Arc<dyn TutorProvider>,
    pub identity_provider: Arc<dyn IdentityProvider>,
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
            identity_provider: Arc::new(GoogleIdentityProvider),
        }
    }

    pub fn with_identity_provider(mut self, identity_provider: Arc<dyn IdentityProvider>) -> Self {
        self.identity_provider = identity_provider;
        self
    }
}
