pub mod agent;
pub mod devices;
pub mod health;
pub mod realtime;
pub mod telemetry;
pub mod tutor;

use axum::{Router, routing::post};

use crate::state::AppState;

pub fn v1() -> Router<AppState> {
    Router::new()
        .route("/devices/register", post(devices::register))
        .route("/devices/refresh", post(devices::refresh))
        .route("/realtime/client-secret", post(realtime::client_secret))
        .route("/tutor/turns", post(tutor::turn))
        .route("/agent/runs", post(agent::create_run))
        .route("/agent/runs/{id}/turns", post(agent::next_turn))
        .route("/agent/runs/{id}/stop", post(agent::stop_run))
        .route("/telemetry/batch", post(telemetry::batch))
}
