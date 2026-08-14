use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use api::{AppConfig, ComputerProvider, ScaleCuaComputerProvider};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Redirect,
    routing::post,
};
use contracts::{
    CaptureScope, ComputerAction, ImageMime, ObservationBinding, PlannerStatus,
    UiObservationMetadata,
};
use serde_json::{Value, json};
use tokio::sync::mpsc;

use api::services::computer_provider::ComputerProviderRequest;

#[derive(Clone)]
struct CaptureState {
    sender: mpsc::UnboundedSender<(HeaderMap, Value)>,
    response: Value,
}

async fn capture_request(
    State(state): State<CaptureState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> (StatusCode, Json<Value>) {
    state
        .sender
        .send((headers, body))
        .expect("test receiver remains open");
    (StatusCode::OK, Json(state.response))
}

fn observation() -> UiObservationMetadata {
    UiObservationMetadata {
        binding: ObservationBinding {
            observation_id: "obs-42".to_owned(),
            app_id: "fixture-browser".to_owned(),
            window_generation: 1,
            layout_generation: 1,
        },
        capture_scope: CaptureScope::ExactWindow,
        elements: Vec::new(),
        truncated: false,
    }
}

fn tool_response(arguments: Value) -> Value {
    json!({
        "choices": [{
            "message": {
                "tool_calls": [{
                    "id": "call-1",
                    "type": "function",
                    "function": {
                        "name": "computer",
                        "arguments": arguments.to_string()
                    }
                }]
            }
        }]
    })
}

#[tokio::test]
async fn sends_native_schema_auth_and_normalizes_coordinates_without_leaking_continuation() {
    let (sender, mut receiver) = mpsc::unbounded_channel();
    let state = CaptureState {
        sender,
        response: tool_response(json!({
            "action": "left_click",
            "coordinate": [250, 750],
            "duration": null,
            "scroll_amount": null,
            "scroll_direction": null,
            "start_coordinate": null,
            "text": null
        })),
    };
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("local listener");
    let address = listener.local_addr().expect("listener address");
    let server = tokio::spawn(
        axum::serve(
            listener,
            Router::new()
                .route("/v1/chat/completions", post(capture_request))
                .with_state(state),
        )
        .into_future(),
    );

    let mut config = AppConfig::test();
    config.scale_cua_base_url = format!("http://{address}/v1");
    let provider = ScaleCuaComputerProvider::new(&config).expect("provider builds");
    let observation = observation();
    let screenshot = b"private Hoa Tui screenshot bytes";
    let result = provider
        .turn(ComputerProviderRequest {
            goal: "Message Hoa Tui",
            turn_number: 0,
            observation: &observation,
            available_apps: &[],
            receipts: &[],
            screenshot,
            screenshot_mime: ImageMime::Png,
            continuation: None,
        })
        .await
        .expect("valid provider response");

    let (headers, body) = receiver.recv().await.expect("captured request");
    assert_eq!(
        headers
            .get("authorization")
            .and_then(|value| value.to_str().ok()),
        Some("Bearer test-scale-cua-key")
    );
    assert_eq!(
        body.pointer("/tools/0/function/name"),
        Some(&json!("computer"))
    );
    assert_eq!(
        body.pointer("/tool_choice/function/name"),
        Some(&json!("computer"))
    );
    assert_eq!(body.get("parallel_tool_calls"), Some(&json!(false)));
    assert!(
        body.pointer("/messages/1/content/1/image_url/url")
            .and_then(Value::as_str)
            .is_some_and(|url| url.starts_with("data:image/png;base64,"))
    );
    let PlannerStatus::Actions { actions } = result.status else {
        panic!("expected one action");
    };
    let ComputerAction::Click { point, .. } = actions[0].action else {
        panic!("expected click");
    };
    assert_eq!((point.x, point.y), (0.25, 0.75));
    assert_eq!(actions[0].observation_id, "obs-42");
    assert_eq!(result.provider_kind, "scale_cua");
    assert_eq!(result.model, "scalecua");
    assert!(!result.continuation.contains("Hoa Tui"));
    assert!(!result.continuation.contains("data:image"));
    assert!(!result.continuation.contains("250"));
    server.abort();
}

#[derive(Clone)]
struct RedirectState {
    followed: Arc<AtomicUsize>,
}

async fn redirect() -> Redirect {
    Redirect::temporary("/redirected")
}

async fn redirected(State(state): State<RedirectState>) -> StatusCode {
    state.followed.fetch_add(1, Ordering::AcqRel);
    StatusCode::OK
}

#[tokio::test]
async fn refuses_redirects_instead_of_resubmitting_screen_content() {
    let followed = Arc::new(AtomicUsize::new(0));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("local listener");
    let address = listener.local_addr().expect("listener address");
    let server = tokio::spawn(
        axum::serve(
            listener,
            Router::new()
                .route("/v1/chat/completions", post(redirect))
                .route("/redirected", post(redirected))
                .with_state(RedirectState {
                    followed: followed.clone(),
                }),
        )
        .into_future(),
    );
    let mut config = AppConfig::test();
    config.scale_cua_base_url = format!("http://{address}/v1");
    let provider = ScaleCuaComputerProvider::new(&config).expect("provider builds");
    let observation = observation();
    let result = provider
        .turn(ComputerProviderRequest {
            goal: "Open fixture",
            turn_number: 0,
            observation: &observation,
            available_apps: &[],
            receipts: &[],
            screenshot: b"private screenshot",
            screenshot_mime: ImageMime::Jpeg,
            continuation: None,
        })
        .await;
    let error = match result {
        Ok(_) => panic!("redirect must be a provider failure"),
        Err(error) => error,
    };
    assert_eq!(error.app.code.as_str(), "provider_unavailable");
    assert_eq!(followed.load(Ordering::Acquire), 0);
    server.abort();
}
