//! Contract tests for `hermes-chat`, written before the implementation.
//!
//! Every test is gateway-free and display-free: the composer and session
//! parser are pure, and stream folding consumes typed `GatewayEvent` values
//! directly. No socket, no GPUI window and no Python gateway is needed.

use hermes_chat::{parse_sessions, Composer, Message, Role, StreamRenderer};
use hermes_gateway::GatewayEvent;
use serde_json::json;

#[test]
fn composer_submit_uses_prompt_submit_method_and_carries_text() {
    let req = Composer::new()
        .submit("hello")
        .expect("submit must build a Request");

    assert_eq!(
        req.method, "prompt.submit",
        "D1 chat is WS prompt.submit only"
    );
    assert_eq!(req.jsonrpc, "2.0");

    let params = req.params.expect("prompt.submit must carry params");
    assert_eq!(params, json!({ "text": "hello" }));
    assert!(params.to_string().contains("hello"));
}

#[test]
fn message_delta_appends_to_current_message() {
    let mut renderer = StreamRenderer::new();

    renderer.render_stream(GatewayEvent::MessageDelta("hi".into()));
    assert_eq!(renderer.current.content, "hi");
    assert_eq!(renderer.current.role, Role::Assistant);

    renderer.render_stream(GatewayEvent::MessageDelta(" there".into()));
    assert_eq!(renderer.current.content, "hi there");
}

#[test]
fn parse_sessions_reads_sample_array() {
    let sessions = parse_sessions(json!([
        { "id": "s1", "title": "First" },
        { "id": "s2", "title": "Second" }
    ]));

    assert_eq!(sessions.len(), 2);
    assert_eq!(sessions[0].id, "s1");
    assert_eq!(sessions[0].title.as_deref(), Some("First"));
    assert_eq!(sessions[1].id, "s2");
}

#[test]
fn message_append_streams_text() {
    let mut message = Message::assistant("");
    message.append("a");
    message.append("b");
    assert_eq!(message.content, "ab");
}

#[test]
fn stream_lifecycle_starts_and_completes() {
    let mut renderer = StreamRenderer::new();
    renderer.render_stream(GatewayEvent::MessageStart { id: None });
    renderer.render_stream(GatewayEvent::MessageDelta("hello".into()));
    renderer.render_stream(GatewayEvent::MessageComplete { id: None });

    assert!(renderer.messages.iter().any(|m| m.content == "hello"));
    assert!(!renderer.streaming);
    assert!(renderer.current.content.is_empty());
}

#[test]
fn reasoning_thinking_voice_are_buffered_separately() {
    let mut renderer = StreamRenderer::new();
    renderer.render_stream(GatewayEvent::ReasoningDelta("why".into()));
    renderer.render_stream(GatewayEvent::ThinkingDelta("think".into()));
    renderer.render_stream(GatewayEvent::VoicePartial("hello".into()));

    assert_eq!(renderer.reasoning, "why");
    assert_eq!(renderer.thinking, "think");
    assert_eq!(renderer.voice_partial, "hello");
    assert!(
        renderer.current.content.is_empty(),
        "non-content deltas must not leak into the visible message"
    );
}

#[test]
fn parse_sessions_accepts_envelope_and_ignores_malformed_entries() {
    let sessions = parse_sessions(json!({
        "sessions": [
            { "id": 42, "name": "Numeric id" },
            { "title": "missing id" },
            "not an object"
        ]
    }));

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].id, "42");
    assert_eq!(sessions[0].title.as_deref(), Some("Numeric id"));
}

#[test]
fn tool_events_are_recorded_without_touching_content() {
    let mut renderer = StreamRenderer::new();
    renderer.render_stream(GatewayEvent::ToolStart {
        name: "search".into(),
        id: None,
    });
    renderer.render_stream(GatewayEvent::ToolComplete {
        name: "search".into(),
        id: None,
    });
    renderer.render_stream(GatewayEvent::Other(json!({ "event": "unknown" })));

    assert_eq!(renderer.tool_events.len(), 2);
    assert_eq!(renderer.other_events.len(), 1);
    assert!(renderer.current.content.is_empty());
}

#[test]
fn role_defaults_to_assistant() {
    assert_eq!(Role::default(), Role::Assistant);
    let message = Message::default();
    assert_eq!(message.role, Role::Assistant);
    assert!(message.content.is_empty());
    assert_eq!(message.ts, 0);
}
