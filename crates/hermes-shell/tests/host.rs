//! Task 5 plugin-host contract tests — written before the implementation.
//!
//! These are gateway-free and display-free: a mock host uses interior
//! mutability to record manifests, and no GatewayClient, socket or display is
//! ever constructed. RED until `PluginHost` and `PluginManifest` exist.

use std::cell::RefCell;

use hermes_core::PluginManifest;
use hermes_shell::PluginHost;

/// In-memory host implementing `PluginHost` with a `RefCell` so `register`
/// can take `&self`, exactly as task 8's registry will.
#[derive(Default)]
struct MockHost {
    registered: RefCell<Vec<PluginManifest>>,
}

impl PluginHost for MockHost {
    fn register(&self, manifest: PluginManifest) {
        self.registered.borrow_mut().push(manifest);
    }

    fn enabled(&self) -> Vec<PluginManifest> {
        self.registered.borrow().clone()
    }
}

fn manifest(id: &str, name: &str) -> PluginManifest {
    PluginManifest {
        id: id.into(),
        name: name.into(),
        version: "1.2.3".into(),
        entrypoint: "main.wasm".into(),
        description: "contract test plugin".into(),
    }
}

#[test]
fn mock_host_registers_one_manifest_and_reports_enabled() {
    let host = MockHost::default();
    assert!(host.enabled().is_empty(), "a fresh host enables nothing");

    host.register(manifest("hermes.test.alpha", "Alpha"));

    let enabled = host.enabled();
    assert_eq!(
        enabled.len(),
        1,
        "register must record exactly one manifest"
    );
    assert_eq!(enabled[0].id, "hermes.test.alpha");
    assert_eq!(enabled[0].name, "Alpha");
    assert_eq!(enabled[0].version, "1.2.3");
    assert_eq!(enabled[0].entrypoint, "main.wasm");
}

#[test]
fn plugin_manifest_round_trips_through_json() {
    let m = manifest("hermes.test.beta", "Beta");
    let json = serde_json::to_string(&m).expect("serialize manifest");
    let back: PluginManifest = serde_json::from_str(&json).expect("deserialize manifest");
    assert_eq!(back, m);
}
