use axum::{Router, middleware, routing::get};
use tower_http::limit::RequestBodyLimitLayer;

use crate::{
    middleware::{body_limit::MEDIA_BODY_LIMIT_BYTES, redacting_trace, request_id},
    routes,
    state::AppState,
};

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(routes::health::healthz))
        .nest("/v1", routes::v1())
        .layer(RequestBodyLimitLayer::new(MEDIA_BODY_LIMIT_BYTES))
        .layer(redacting_trace::layer())
        .layer(middleware::from_fn(request_id::attach))
        .with_state(state)
}
