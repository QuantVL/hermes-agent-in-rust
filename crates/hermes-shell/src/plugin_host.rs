//! Host-side API for plugin registration.
//!
//! Defined here so `hermes-plugins` has a stable integration point: that
//! crate depends on `hermes-core` for [`PluginManifest`] and on this crate for
//! the trait, then implements it with a real registry.

use hermes_core::PluginManifest;

/// Contract for a host that registers and enumerates plugins.
///
/// `register` takes `&self` so a host can share itself across shell
/// components using interior mutability (e.g. a registry behind a mutex),
/// exactly as the in-memory mock in `tests/host.rs` does with a `RefCell`.
pub trait PluginHost {
    /// Register a plugin manifest with the host.
    fn register(&self, manifest: PluginManifest);

    /// Return every currently enabled plugin manifest.
    fn enabled(&self) -> Vec<PluginManifest>;
}
