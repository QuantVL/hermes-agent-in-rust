//! WebSocket JSON stream: ticket-as-subprotocol auth and typed gateway events.

use std::pin::Pin;
use std::task::{Context, Poll};

use futures::{Stream, StreamExt};
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

use crate::origin::validate_bare_origin;
use crate::ticket::{validate_ticket_token, TicketPurpose};
use crate::{GatewayClient, GatewayError, Result};

/// Typed events from the gateway's real streaming vocabulary.
///
/// `Other` is the escape hatch for unknown events; there is deliberately no
/// generic `Token`/`Delta` variant — those event names do not exist.
#[derive(Debug, Clone, PartialEq)]
pub enum GatewayEvent {
    /// Client-local: the transport connected (not a server frame).
    Connected,
    /// Client-local: the transport ended; `Some(reason)` on error/close.
    Disconnected(Option<String>),
    MessageStart {
        id: Option<String>,
    },
    MessageDelta(String),
    MessageComplete {
        id: Option<String>,
    },
    ReasoningDelta(String),
    ThinkingDelta(String),
    VoicePartial(String),
    ToolStart {
        name: String,
        id: Option<String>,
    },
    ToolProgress {
        name: String,
        id: Option<String>,
        message: Option<String>,
    },
    ToolComplete {
        name: String,
        id: Option<String>,
    },
    ToolGenerating {
        name: Option<String>,
    },
    ToolOutputRisk {
        name: Option<String>,
        level: Option<String>,
    },
    SubagentTool(Value),
    Other(Value),
}

impl GatewayEvent {
    /// Parse a single text frame.
    pub fn from_wire_text(text: &str) -> Self {
        match serde_json::from_str::<Value>(text) {
            Ok(value) => Self::from_value(value),
            Err(_) => Self::Other(Value::String(text.to_string())),
        }
    }

    /// Map an event JSON object onto the typed vocabulary.
    pub fn from_value(value: Value) -> Self {
        let name = value
            .get("event")
            .or_else(|| value.get("type"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let data = value.get("data").or_else(|| value.get("params")).cloned();
        let data = data.as_ref();

        let id = extract(&value, data, &["id", "message_id", "tool_call_id"]);
        let tool_name = || extract(&value, data, &["name", "tool"]).unwrap_or_default();

        match name.as_str() {
            "message.start" => Self::MessageStart { id },
            "message.delta" => Self::MessageDelta(delta(&value, data)),
            "message.complete" => Self::MessageComplete { id },
            "reasoning.delta" => Self::ReasoningDelta(delta(&value, data)),
            "thinking.delta" => Self::ThinkingDelta(delta(&value, data)),
            "voice.partial" => Self::VoicePartial(delta(&value, data)),
            "tool.start" => Self::ToolStart {
                name: tool_name(),
                id,
            },
            "tool.progress" => Self::ToolProgress {
                name: tool_name(),
                id,
                message: extract(&value, data, &["message", "progress"]),
            },
            "tool.complete" => Self::ToolComplete {
                name: tool_name(),
                id,
            },
            "tool.generating" => Self::ToolGenerating {
                name: extract(&value, data, &["name", "tool"]),
            },
            "tool.output_risk" => Self::ToolOutputRisk {
                name: extract(&value, data, &["name", "tool"]),
                level: extract(&value, data, &["level", "risk"]),
            },
            "subagent.tool" => Self::SubagentTool(value),
            "client.connected" => Self::Connected,
            "client.disconnected" => {
                Self::Disconnected(extract(&value, data, &["reason", "message"]))
            }
            _ => Self::Other(value),
        }
    }
}

fn delta(value: &Value, data: Option<&Value>) -> String {
    extract(value, data, &["delta", "text", "content"]).unwrap_or_default()
}

fn extract(value: &Value, data: Option<&Value>, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(text) = value.get(key).and_then(Value::as_str) {
            return Some(text.to_string());
        }
        if let Some(text) = data.and_then(|data| data.get(key)).and_then(Value::as_str) {
            return Some(text.to_string());
        }
    }
    None
}

/// A stream of typed [`GatewayEvent`]s from a connected gateway WS.
#[derive(Debug)]
pub struct WsStream {
    rx: tokio::sync::mpsc::Receiver<GatewayEvent>,
}

impl Stream for WsStream {
    type Item = GatewayEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().rx.poll_recv(cx)
    }
}

impl GatewayClient {
    /// Dial `api_origin` with `http`→`ws` and `/api/ws`.
    ///
    /// A fresh `interactive` ticket is minted and validated as an RFC 6455
    /// token before any socket is opened. It is sent as the subprotocol
    /// `hermes-gateway-ticket.<ticket>` alongside `hermes-gateway-v1`, never as
    /// a query parameter.
    pub fn ws_connect(&self) -> Result<WsStream> {
        let ticket = self.broker.mint(TicketPurpose::Interactive)?;
        validate_ticket_token(ticket.as_str())?;
        validate_bare_origin(&self.api_origin)?;

        let ws_origin = self
            .api_origin
            .replacen("https://", "wss://", 1)
            .replacen("http://", "ws://", 1);
        let ws_url = format!("{}/api/ws", ws_origin.trim_end_matches('/'));
        let request = tokio_tungstenite::tungstenite::http::Request::builder()
            .uri(ws_url)
            .header(
                "Sec-WebSocket-Protocol",
                format!(
                    "hermes-gateway-v1, hermes-gateway-ticket.{}",
                    ticket.as_str()
                ),
            )
            .body(())
            .map_err(|e| GatewayError::InvalidState(format!("invalid WS request: {e}")))?;

        let (tx, rx) = tokio::sync::mpsc::channel::<GatewayEvent>(64);
        let _ = std::thread::spawn(move || {
            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(e) => {
                    tracing::warn!("cannot build WS runtime: {e}");
                    return;
                }
            };
            runtime.block_on(async move {
                match tokio_tungstenite::connect_async(request).await {
                    Ok((mut ws, _)) => {
                        while let Some(frame) = ws.next().await {
                            match frame {
                                Ok(Message::Text(text)) => {
                                    if tx.send(GatewayEvent::from_wire_text(&text)).await.is_err() {
                                        break;
                                    }
                                }
                                Ok(Message::Close(_)) => {
                                    let _ = tx.send(GatewayEvent::Disconnected(None)).await;
                                    break;
                                }
                                Ok(_) => {}
                                Err(e) => {
                                    tracing::warn!("WS frame error: {e}");
                                    let _ = tx
                                        .send(GatewayEvent::Disconnected(Some(e.to_string())))
                                        .await;
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("WS connect failed: {e}");
                        let _ = tx
                            .send(GatewayEvent::Disconnected(Some(e.to_string())))
                            .await;
                    }
                }
            });
        });

        Ok(WsStream { rx })
    }
}
