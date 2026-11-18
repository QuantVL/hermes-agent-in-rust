//! Protocol-agnostic plugin metadata.
//!
//! A [`PluginManifest`] is declarative metadata read from a plugin's manifest
//! file. It deliberately carries no gateway, host or shell dependencies:
//! `hermes-plugins` depends on `hermes-core` alone for this type and
//! implements the `PluginHost` trait (defined in `hermes-shell`) later.

use serde::{Deserialize, Serialize};

/// Declarative metadata describing one installable plugin.
///
/// This is a pure data type: no network calls, host hooking or display state
/// live here. The host decides what registering a manifest means.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PluginManifest {
    /// Stable, unique identifier (e.g. `hermes.example.plugin`).
    pub id: String,
    /// Human-readable display name shown in the shell.
    pub name: String,
    /// Version string. Kept as text — version comparison is a host policy,
    /// not a concern of this protocol-agnostic metadata type.
    pub version: String,
    /// Entry point (module path or script) the loader uses to start the plugin.
    pub entrypoint: String,
    /// Short description shown in the shell's Plugins settings tab.
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_round_trips_through_json() {
        let manifest = PluginManifest {
            id: "hermes.test".into(),
            name: "Test".into(),
            version: "1.0.0".into(),
            entrypoint: "main.wasm".into(),
            description: "a test plugin".into(),
        };
        let json = serde_json::to_string(&manifest).expect("serialize manifest");
        assert!(json.contains("hermes.test"));

        let parsed: PluginManifest = serde_json::from_str(&json).expect("deserialize manifest");
        assert_eq!(parsed, manifest);
    }

    #[test]
    fn missing_fields_deserialize_to_defaults() {
        let parsed: PluginManifest = serde_json::from_str(r#"{"id":"only-id"}"#)
            .expect("partial manifest must still deserialize");
        assert_eq!(parsed.id, "only-id");
        assert_eq!(parsed.name, PluginManifest::default().name);
        assert_eq!(parsed.version, PluginManifest::default().version);
        assert_eq!(parsed.entrypoint, PluginManifest::default().entrypoint);
        assert_eq!(parsed.description, PluginManifest::default().description);
    }
}
