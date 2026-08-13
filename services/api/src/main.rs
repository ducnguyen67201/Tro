use std::sync::Arc;

use api::{
    AppConfig, AppState, CloudProvider, OpenRouterTutorProvider, Repository, build_router,
    repositories::{MemoryRepository, PgRepository},
    services::device_tokens,
};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(AppConfig::from_env()?);
    init_tracing(&config.log_format);

    let repository: Arc<dyn Repository> = if config.database_url == "memory://" {
        let repository = Arc::new(MemoryRepository::default());
        if let Some(token) = &config.development_device_token {
            let digest =
                device_tokens::token_digest(config.device_token_hmac_key.expose(), token.expose())
                    .map_err(|_| std::io::Error::other("invalid development device token"))?;
            repository.seed_device_token_digest(digest);
        }
        repository
    } else {
        let pool = PgPoolOptions::new()
            .max_connections(8)
            .connect(&config.database_url)
            .await?;
        let migration_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
        sqlx::migrate::Migrator::new(migration_path)
            .await?
            .run(&pool)
            .await?;
        Arc::new(PgRepository::new(pool))
    };
    let provider = Arc::new(CloudProvider::new(&config));
    let tutor_provider = Arc::new(OpenRouterTutorProvider::new(&config));
    let state = AppState::new(config.clone(), repository, provider, tutor_provider);
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    tracing::info!(component = "api", operation = "listen", address = %config.bind_addr);
    axum::serve(listener, build_router(state)).await?;
    Ok(())
}

fn init_tracing(format: &str) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    if format == "json" {
        builder.json().init();
    } else {
        builder.init();
    }
}
