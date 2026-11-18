//! Task 3 contract tests for the `hermes-gpui` app skeleton.
//!
//! These are written before the implementation exists: they pin the exact
//! interfaces later UI tasks consume (theme tokens, window-name strings, and
//! a `GlobalState` that never dials a gateway on construction).

use hermes_core::Config;
use hermes_gpui::{GlobalState, Theme, WindowType};

#[test]
fn theme_default_background_is_opaque() {
    let theme = Theme::default();
    assert!(
        theme.colors.bg.is_opaque(),
        "the default theme background must not be transparent"
    );
    assert_eq!(
        theme.colors.bg.a,
        u8::MAX,
        "the default theme background alpha must be fully opaque"
    );
}

#[test]
fn window_type_as_str_maps_all_five_variants() {
    assert_eq!(WindowType::Main.as_str(), "main");
    assert_eq!(WindowType::Hud.as_str(), "hud");
    assert_eq!(WindowType::PetOverlay.as_str(), "pet-overlay");
    assert_eq!(WindowType::QuickEntry.as_str(), "quick-entry");
    assert_eq!(WindowType::WakeIndicator.as_str(), "wake-indicator");
}

#[test]
fn global_state_constructs_without_a_gateway_connection() {
    let state = GlobalState::new(Config::default());
    assert!(
        state.gateway.is_none(),
        "a freshly constructed GlobalState must not hold a live gateway client"
    );
    assert!(!state.gateway_connected());
    assert_eq!(state.config.profile, "default");
}
