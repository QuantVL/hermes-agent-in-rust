//! Task 7 `hermes-overlay` contract tests — written before the implementation.
//!
//! These are gateway-free and display-free: no `GatewayClient`, socket, GPUI
//! window or event loop is ever constructed. Every assertion runs against
//! plain Rust state. RED until `PetOverlay`, `QuickEntry` and `WakeIndicator`
//! exist.

use hermes_chat::Composer;
use hermes_gpui::WindowType;
use hermes_overlay::{
    toggle_state, PetOverlay, QuickEntry, WakeIndicator, WakeState, PET_FRAME_COUNT,
    PROMPT_SUBMIT_METHOD,
};
use serde_json::json;

// (a) quick-entry submit produces a prompt.submit Request carrying the text.
#[test]
fn quick_entry_submit_produces_prompt_submit_request() {
    assert_eq!(PROMPT_SUBMIT_METHOD, "prompt.submit");
    let entry = QuickEntry::new();
    let req = entry.submit("hello overlay").expect("quick entry submit");
    assert_eq!(req.method, "prompt.submit");
    assert_eq!(req.jsonrpc, "2.0");
}

#[test]
fn quick_entry_request_carries_the_text() {
    let req = QuickEntry::new().submit("deploy now").expect("submit");
    let wire = serde_json::to_value(&req).expect("serialize request");
    assert_eq!(wire["params"], json!({ "text": "deploy now" }));
}

#[test]
fn quick_entry_delegates_to_composer() {
    let via_entry = QuickEntry::new().submit("same text").expect("entry submit");
    let via_composer = Composer::new()
        .submit("same text")
        .expect("composer submit");
    assert_eq!(via_entry.method, via_composer.method);
    assert_eq!(via_entry.params, via_composer.params);
}

#[test]
fn quick_entry_submit_draft_clears_the_draft() {
    let mut entry = QuickEntry::new();
    entry.draft = "one more".into();
    let req = entry.submit_draft().expect("submit draft");
    assert_eq!(req.method, "prompt.submit");
    assert!(entry.draft.is_empty());
}

// (b) wake indicator toggles awake/asleep via a pure helper.
#[test]
fn wake_toggle_helper_flips_both_states() {
    assert_eq!(toggle_state(WakeState::Asleep), WakeState::Awake);
    assert_eq!(toggle_state(WakeState::Awake), WakeState::Asleep);
}

#[test]
fn wake_indicator_toggling_is_pure_and_mutating() {
    let asleep = WakeIndicator::new();
    assert!(asleep.is_asleep());
    assert!(!asleep.is_awake());

    let awake = asleep.toggled();
    assert!(awake.is_awake());
    assert!(asleep.is_asleep(), "toggled() must not mutate its receiver");

    let mut indicator = WakeIndicator::new();
    indicator.toggle();
    assert!(indicator.is_awake());
    indicator.toggle();
    assert!(indicator.is_asleep());
}

#[test]
fn wake_indicator_uses_the_wake_window_kind() {
    let indicator = WakeIndicator::new();
    assert_eq!(indicator.window, WindowType::WakeIndicator);
}

// (c) pet overlay advances its frame/step state deterministically.
#[test]
fn pet_overlay_starts_at_frame_zero() {
    let pet = PetOverlay::new();
    assert_eq!(pet.frame, 0);
    assert_eq!(pet.step, 0);
    assert_eq!(pet.window, WindowType::PetOverlay);
    assert_eq!(pet.frame_count(), PET_FRAME_COUNT);
}

#[test]
fn pet_overlay_advances_frames_and_wraps_deterministically() {
    let mut pet = PetOverlay::new();
    pet.advance();
    assert_eq!((pet.frame, pet.step), (1, 1));
    for _ in 1..PET_FRAME_COUNT {
        pet.advance();
    }
    assert_eq!(pet.frame, 0, "frame wraps after PET_FRAME_COUNT advances");
    assert_eq!(pet.step, PET_FRAME_COUNT);
}

#[test]
fn pet_overlay_next_state_is_a_pure_helper() {
    assert_eq!(PetOverlay::next_state(0, 0), (1, 1));
    assert_eq!(PetOverlay::next_state(PET_FRAME_COUNT - 1, 9), (0, 10));

    let pet = PetOverlay::new();
    let next = pet.next();
    assert_eq!((next.frame, next.step), (1, 1));
    assert_eq!((pet.frame, pet.step), (0, 0), "next() must not mutate");
}
