pub mod agent;
pub mod auth;
pub mod devices;
pub mod health;
pub mod realtime;
pub mod telemetry;
pub mod tutor;

use axum::{Router, routing::post};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::limit::RequestBodyLimitLayer;

use crate::{middleware::body_limit::JSON_BODY_LIMIT_BYTES, state::AppState};

pub fn v1() -> Router<AppState> {
    let auth = Router::new()
        .route("/auth/google/start", post(auth::google_start))
        .route("/auth/google/complete", post(auth::google_complete))
        .layer(ConcurrencyLimitLayer::new(16))
        .layer(RequestBodyLimitLayer::new(JSON_BODY_LIMIT_BYTES));
    Router::new()
        .merge(auth)
        .route("/devices/register", post(devices::register))
        .route("/devices/refresh", post(devices::refresh))
        .route("/realtime/client-secret", post(realtime::client_secret))
        .route("/tutor/turns", post(tutor::turn))
        .route("/agent/runs", post(agent::create_run))
        .route("/agent/runs/{id}/turns", post(agent::next_turn))
        .route("/agent/runs/{id}/stop", post(agent::stop_run))
        .route("/telemetry/batch", post(telemetry::batch))
}
