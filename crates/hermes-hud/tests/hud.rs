//! Task 6 `hermes-hud` contract tests — written before the implementation.
//!
//! These are gateway-free and display-free: no `GatewayClient`, socket, GPUI
//! window or event loop is ever constructed. Every assertion runs against
//! plain Rust state. RED until `HudWindow`, `HudComposer` and
//! `SessionSwitcher` exist.

use hermes_chat::{Composer, SessionInfo};
use hermes_core::Config;
use hermes_gpui::WindowType;
use hermes_hud::{HudComposer, HudWindow, SessionSwitcher, SESSION_ACTIVATE_METHOD};
use serde_json::json;

fn session(id: &str) -> SessionInfo {
    SessionInfo {
        id: id.into(),
        title: Some("Example chat".into()),
        updated_at: None,
    }
}

// (a) selecting a session produces a session.activate Request carrying the id.
#[test]
fn activation_request_uses_the_session_activate_method() {
    assert_eq!(SESSION_ACTIVATE_METHOD, "session.activate");

    let req = SessionSwitcher::activation_request(&session("sess-42"));
    assert_eq!(req.method, "session.activate");
    assert_eq!(req.jsonrpc, "2.0");
}

#[test]
fn activation_request_carries_the_selected_session_id() {
    let req = SessionSwitcher::activation_request(&session("sess-42"));
    let wire = serde_json::to_value(&req).expect("serialize activation request");
    assert_eq!(
        wire["params"],
        json!({ "session_id": "sess-42" }),
        "session.activate params must carry the selected session id"
    );
}

#[test]
fn selecting_a_session_records_it_and_builds_the_request() {
    let mut switcher = SessionSwitcher::new();
    assert!(
        switcher.selected.is_none(),
        "switcher starts with no selection"
    );

    let req = switcher.select(session("sess-7"));
    assert_eq!(req.method, "session.activate");
    assert_eq!(switcher.selected_id(), Some("sess-7"));
}

// (b) HUD state starts with no active session and can set one via a pure helper.
#[test]
fn hud_starts_with_no_active_session() {
    let hud = HudWindow::new(Config::default());
    assert!(hud.active_session.is_none());
    assert!(!hud.has_active_session());
    assert_eq!(hud.window, WindowType::Hud);
}

#[test]
fn hud_active_session_can_be_set_with_a_pure_helper() {
    let base = HudWindow::new(Config::default());
    let with_session = base.clone().with_active_session(session("sess-99"));

    assert!(
        base.active_session.is_none(),
        "the pure helper must not mutate its receiver"
    );
    assert_eq!(
        with_session
            .active_session
            .as_ref()
            .expect("active session is set")
            .id,
        "sess-99"
    );
    assert!(with_session.has_active_session());
}

// (c) HUD composer submit delegates to Composer::submit producing prompt.submit.
#[test]
fn hud_composer_submit_delegates_to_composer() {
    let hud = HudComposer::new();
    let via_hud = hud.submit("hello hud").expect("hud submit");
    let via_composer = Composer::new()
        .submit("hello hud")
        .expect("composer submit");

    assert_eq!(via_hud.method, "prompt.submit");
    assert_eq!(via_hud.method, via_composer.method);
    assert_eq!(via_hud.params, via_composer.params);
}
