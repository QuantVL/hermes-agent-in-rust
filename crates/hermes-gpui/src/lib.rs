//! GPUI 0.2.2 app skeleton (theme, window kinds, state, shell) plus the
//! original vertical-slice spike proof types from Task 1.
//!
//! `tests/spike.rs` still drives `SpikeView` through a window, a text render,
//! an action and a tokio bridge. `tests/theme.rs` pins the app-skeleton
//! interfaces consumed by later UI tasks.

pub mod app;
pub mod state;
pub mod theme;
pub mod window;

pub use app::run;
pub use state::GlobalState;
pub use theme::{ColorTokens, FontTokens, Rgba, Theme};
pub use window::WindowType;

/// Root view for the spike window.
///
/// [`gpui::Render`] emits a single text node containing [`Self::text`],
/// exercising GPUI 0.2.2's element/text pipeline. The other fields are
/// written by the action handler and by the tokio network bridge before
/// `cx.notify()` is called.
pub struct SpikeView {
    /// Text rendered as the view's single child node.
    pub text: gpui::SharedString,
    /// Payload delivered from the tokio `reqwest` task.
    pub received: Option<String>,
    /// Number of dispatched actions handled so far.
    pub action_count: usize,
}

impl SpikeView {
    /// Build a view that renders `text` as its single text node.
    pub fn new(text: impl Into<gpui::SharedString>) -> Self {
        Self {
            text: text.into(),
            received: None,
            action_count: 0,
        }
    }
}

impl gpui::Render for SpikeView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        use gpui::ParentElement as _;
        gpui::div().child(self.text.clone())
    }
}

// Compile-time proof that the whole pinned crates.io stack type-checks
// alongside GPUI 0.2.2. `tests/spike.rs` supplies the runtime proof.

/// Pinned `reqwest` client type.
pub type SpikeHttpClient = reqwest::Client;
/// Pinned tokio runtime type.
pub type SpikeRuntime = tokio::runtime::Runtime;
/// Pinned `tokio-tungstenite` stream over the pinned tokio TCP transport.
pub type SpikeWebSocket = tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>;
/// Pinned mio driver type (transitive compatibility pin).
pub type SpikeIoDriver = mio::Poll;
/// Pinned libc integer type (transitive compatibility pin).
pub type SpikeErrno = libc::c_int;
