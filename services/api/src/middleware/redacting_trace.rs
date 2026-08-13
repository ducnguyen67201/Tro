use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tracing::Level;

pub fn layer()
-> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>>
{
    TraceLayer::new_for_http().on_response(DefaultOnResponse::new().level(Level::INFO))
}
