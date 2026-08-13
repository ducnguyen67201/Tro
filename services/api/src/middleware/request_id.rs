use axum::{
    body::Body,
    http::{Request, header::HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

pub async fn attach(mut request: Request<Body>, next: Next) -> Response {
    let request_id = format!("req_{}", Uuid::new_v4().simple());
    request.extensions_mut().insert(request_id.clone());
    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert("x-request-id", value);
    }
    response
}
