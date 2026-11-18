//! Discovery via `hermes --profile P gateway ensure --json`.
//!
//! stdout carries exactly one JSON line. The wire fields below are binding;
//! `runtime_protocol` must be 1 and `session-authority-v1` must be present,
//! otherwise discovery fails loudly instead of guessing a protocol.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::origin::validate_bare_origin;
use crate::{error::ErrorCode, GatewayError, Result};

/// Required capability advertised by a usable gateway endpoint.
pub const REQUIRED_CAPABILITY: &str = "session-authority-v1";
/// Only this runtime protocol version is understood.
pub const SUPPORTED_RUNTIME_PROTOCOL: u32 = 1;

/// The `endpoint` object from an `ensure --json` success line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GatewayEndpoint {
    pub profile_id: String,
    pub instance_id: String,
    pub authority_epoch: u64,
    pub runtime_protocol: u32,
    /// Bare `http(s)` loopback origin with an explicit port; no path/query/userinfo.
    pub api_origin: String,
    #[serde(default)]
    pub supervisor: Value,
    pub capabilities: Vec<String>,
    /// Nullable; when present it is the directory containing `control.sock`.
    #[serde(default)]
    pub control_home: Option<PathBuf>,
    pub code_sha: String,
}

#[derive(Debug, Deserialize)]
struct EnsureEnvelope {
    state: String,
    endpoint: Option<GatewayEndpoint>,
    #[serde(default)]
    reason_code: Value,
    #[serde(default)]
    detail: Value,
    #[serde(default)]
    client_code_sha: Value,
}

/// Parse and validate one `gateway ensure --json` line.
///
/// This is the pure validator used by [`discover`]; exposing it lets contract
/// tests cover every rejection class without a Python gateway on PATH.
pub fn parse_ensure_response(line: &str) -> Result<GatewayEndpoint> {
    let envelope: EnsureEnvelope = serde_json::from_str(line).map_err(|e| {
        GatewayError::Discovery(format!("could not parse `gateway ensure --json` line: {e}"))
    })?;
    // Parsed for wire fidelity; reserved for future client-sha pinning.
    let _ = &envelope.client_code_sha;

    if envelope.state != "ready" {
        let error_code = ErrorCode::from_state(&envelope.state);
        let detail = if envelope.detail.is_null() {
            envelope.reason_code.to_string()
        } else {
            envelope.detail.to_string()
        };
        return Err(GatewayError::ExitCode {
            code: error_code.map(ErrorCode::as_i32).unwrap_or(-1),
            error_code,
            message: format!("gateway state={}: {detail}", envelope.state),
        });
    }

    let endpoint = envelope.endpoint.ok_or_else(|| {
        GatewayError::InvalidState("state=ready but no endpoint object was returned".into())
    })?;

    if endpoint.runtime_protocol != SUPPORTED_RUNTIME_PROTOCOL {
        return Err(GatewayError::Incompatible(format!(
            "runtime_protocol={} but this client requires {SUPPORTED_RUNTIME_PROTOCOL}",
            endpoint.runtime_protocol
        )));
    }
    if endpoint.authority_epoch == 0 {
        return Err(GatewayError::InvalidState(
            "authority_epoch must be greater than zero".into(),
        ));
    }
    if !endpoint
        .capabilities
        .iter()
        .any(|capability| capability == REQUIRED_CAPABILITY)
    {
        return Err(GatewayError::InvalidState(format!(
            "capabilities missing {REQUIRED_CAPABILITY:?}: {:?}",
            endpoint.capabilities
        )));
    }
    validate_bare_origin(&endpoint.api_origin)?;

    Ok(endpoint)
}

/// Spawn `hermes --profile <profile> gateway ensure --json` and return the
/// validated endpoint.
///
/// If the `hermes` CLI is not installed or not on `PATH`, this returns a clear
/// [`GatewayError::Discovery`] error; no endpoint is guessed.
pub fn discover(profile: &str) -> Result<GatewayEndpoint> {
    let output = std::process::Command::new("hermes")
        .args(["--profile", profile, "gateway", "ensure", "--json"])
        .output()
        .map_err(|e| {
            GatewayError::Discovery(format!(
                "failed to spawn `hermes` CLI (install it and put it on PATH): {e}"
            ))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if let Some(line) = stdout.lines().find(|line| !line.trim().is_empty()) {
        let endpoint = parse_ensure_response(line.trim())?;
        if !output.status.success() {
            return Err(GatewayError::ExitCode {
                code: output.status.code().unwrap_or(-1),
                error_code: output.status.code().and_then(ErrorCode::from_exit_code),
                message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }
        return Ok(endpoint);
    }

    if output.status.success() {
        return Err(GatewayError::Discovery(
            "`gateway ensure --json` produced no JSON line on stdout".into(),
        ));
    }

    let code = output.status.code().unwrap_or(-1);
    Err(GatewayError::ExitCode {
        code,
        error_code: ErrorCode::from_exit_code(code),
        message: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
}
