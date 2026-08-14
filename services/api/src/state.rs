use std::sync::Arc;

use crate::{
    config::AppConfig,
    repositories::Repository,
    services::{
        computer_provider::ComputerProvider,
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
    pub computer_provider: Arc<dyn ComputerProvider>,
    pub tutor_provider: Arc<dyn TutorProvider>,
    pub identity_provider: Arc<dyn IdentityProvider>,
}

impl AppState {
    pub fn new<P>(
        config: Arc<AppConfig>,
        repository: Arc<dyn Repository>,
        provider: Arc<P>,
        tutor_provider: Arc<dyn TutorProvider>,
    ) -> Self
    where
        P: Provider + ComputerProvider + 'static,
    {
        Self {
            config,
            repository,
            provider: provider.clone(),
            computer_provider: provider,
            tutor_provider,
            identity_provider: Arc::new(GoogleIdentityProvider),
        }
    }

    pub fn new_with_providers(
        config: Arc<AppConfig>,
        repository: Arc<dyn Repository>,
        provider: Arc<dyn Provider>,
        computer_provider: Arc<dyn ComputerProvider>,
        tutor_provider: Arc<dyn TutorProvider>,
    ) -> Self {
        Self {
            config,
            repository,
            provider,
            computer_provider,
            tutor_provider,
            identity_provider: Arc::new(GoogleIdentityProvider),
        }
    }

    pub fn with_identity_provider(mut self, identity_provider: Arc<dyn IdentityProvider>) -> Self {
        self.identity_provider = identity_provider;
        self
    }
}
