//! Application shell: GPUI startup, window creation and the tokio bridge.

use gpui::AppContext;
use hermes_core::Config;

use crate::state::GlobalState;
use crate::window::WindowType;

/// Root view for every window the shell opens.
///
/// Later UI tasks replace this placeholder body with theme-aware widgets;
/// keeping one minimal view now makes `run()` a real GPUI window path rather
/// than a stub.
struct ShellView {
    state: GlobalState,
    window: WindowType,
    /// Keeps the tokio bridge sender alive for as long as the window exists.
    _bridge: tokio::sync::mpsc::UnboundedSender<()>,
}

impl gpui::Render for ShellView {
    fn render(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        use gpui::ParentElement as _;
        let label: gpui::SharedString = format!(
            "hermes · {} · {}",
            self.window.as_str(),
            self.state.config.profile
        )
        .into();
        gpui::div().child(label)
    }
}

/// Start a dedicated OS thread owning a multi-thread tokio runtime.
///
/// This is the tokio-on-a-thread half of the bridge Task 1's spike proved:
/// GPUI keeps its single-threaded run loop, async work runs on this runtime,
/// and results cross back over `tokio::sync::mpsc`. Later tasks enqueue work
/// through the returned sender; until then the receiver simply parks.
fn spawn_tokio_bridge() -> tokio::sync::mpsc::UnboundedSender<()> {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
    std::thread::Builder::new()
        .name("hermes-tokio-bridge".to_string())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("build tokio runtime for hermes-gpui");
            runtime.block_on(async move {
                loop {
                    if rx.recv().await.is_none() {
                        break;
                    }
                }
            });
        })
        .expect("spawn tokio bridge thread");
    tx
}

/// Initialize the GPUI application and open `window`.
///
/// `profile`, when given, overrides the profile recorded in the loaded
/// config. This function needs a display and is intentionally excluded from
/// unit tests; tests exercise `Theme`, `WindowType` and `GlobalState` directly.
pub fn run(profile: Option<String>, window: WindowType) {
    let mut config = Config::load().unwrap_or_else(|_| Config::default());
    if let Some(profile) = profile {
        config.profile = profile;
    }
    let state = GlobalState::new(config);
    let bridge = spawn_tokio_bridge();

    gpui::Application::new().run(move |cx: &mut gpui::App| {
        let opened = cx.open_window(gpui::WindowOptions::default(), move |_window, cx| {
            cx.new(|_cx| ShellView {
                state,
                window,
                _bridge: bridge,
            })
        });
        if opened.is_err() {
            panic!("failed to open {} window", window.as_str());
        }
    });
}
