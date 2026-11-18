//! Shared application state.

use hermes_core::Config;
use hermes_gateway::GatewayClient;

/// State shared by every shell window.
///
/// The gateway client is optional and is only ever attached after discovery
/// succeeds. Constructing `GlobalState` therefore never spawns a process,
/// opens a socket or touches the network — headless unit tests need no
/// gateway at all.
#[derive(Debug, Clone)]
pub struct GlobalState {
    pub config: Config,
    pub gateway: Option<GatewayClient>,
}

impl GlobalState {
    /// Build state from `config` with no gateway connection.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            gateway: None,
        }
    }

    /// Whether a gateway client has been attached yet.
    pub fn gateway_connected(&self) -> bool {
        self.gateway.is_some()
    }

    /// Attach (or detach) a previously constructed gateway client.
    ///
    /// Construction of the client remains the caller's responsibility, which
    /// keeps discovery/connection failures out of `GlobalState::new`.
    pub fn set_gateway(&mut self, gateway: Option<GatewayClient>) {
        self.gateway = gateway;
    }
}

impl Default for GlobalState {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
