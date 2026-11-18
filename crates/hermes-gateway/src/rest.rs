//! REST client: one fresh single-use ticket per request, never an `Origin` header.

use serde_json::Value;

use crate::discovery::GatewayEndpoint;
use crate::join_api_origin;
use crate::ticket::{TicketBroker, TicketPurpose};
use crate::{GatewayError, Result};

/// A gateway client bound to one discovered endpoint.
#[derive(Debug, Clone)]
pub struct GatewayClient {
    pub(crate) api_origin: String,
    pub(crate) broker: TicketBroker,
}

impl GatewayClient {
    /// Build a client from a validated discovery endpoint.
    pub fn new(endpoint: &GatewayEndpoint) -> Result<Self> {
        let control_home = endpoint.control_home.clone().ok_or_else(|| {
            GatewayError::InvalidState("gateway endpoint has no control_home for tickets".into())
        })?;
        Ok(Self::with_broker(
            endpoint.api_origin.clone(),
            TicketBroker::new(control_home, &endpoint.profile_id, &endpoint.instance_id),
        ))
    }

    /// Build a client from an explicit origin and ticket broker (used in tests).
    pub fn with_broker(api_origin: impl Into<String>, broker: TicketBroker) -> Self {
        Self {
            api_origin: api_origin.into(),
            broker,
        }
    }

    /// `GET` a gateway API path. A brand-new `native-http` ticket is minted for
    /// every call and sent in `x-hermes-gateway-ticket`. No `Origin` header is
    /// ever added.
    ///
    /// The async request runs on a short-lived tokio worker thread so this
    /// synchronous API is safe to call from both sync and async contexts.
    pub fn get(&self, path: &str) -> Result<Value> {
        let url = join_api_origin(&self.api_origin, path)?;
        let ticket = self.broker.mint(TicketPurpose::NativeHttp)?;

        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let _ = std::thread::spawn(move || {
            let outcome = (|| {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| GatewayError::Discovery(format!("tokio runtime: {e}")))?;
                runtime.block_on(async {
                    let client = reqwest::Client::new();
                    let response = client
                        .get(&url)
                        .header("x-hermes-gateway-ticket", ticket.as_str())
                        .send()
                        .await?;
                    response.json::<Value>().await.map_err(GatewayError::from)
                })
            })();
            let _ = tx.send(outcome);
        });

        rx.recv().map_err(|_| {
            GatewayError::Discovery("HTTP worker thread stopped unexpectedly".into())
        })?
    }
}
