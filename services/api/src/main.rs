use std::sync::Arc;

use api::{AppConfig, AppState, OpenAiProvider, build_router, repositories::PgRepository};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Arc::new(AppConfig::from_env()?);
    init_tracing(&config.log_format);

    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&config.database_url)
        .await?;
    let migration_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    sqlx::migrate::Migrator::new(migration_path)
        .await?
        .run(&pool)
        .await?;

    let repository = Arc::new(PgRepository::new(pool));
    let provider = Arc::new(OpenAiProvider::new(
        config.openai_api_key.expose().to_owned(),
        config.openai_realtime_model.clone(),
        config.openai_computer_model.clone(),
        config.openai_realtime_voice.clone(),
    ));
    let state = AppState::new(config.clone(), repository, provider);
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
