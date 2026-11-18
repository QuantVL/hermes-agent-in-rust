//! Origin lock — reproduce the Python `joinBackendApiUrl` safety rules.
//!
//! A path is only accepted when it is a single absolute path segment sequence
//! (one leading `/`) that cannot change the origin of the loopback base.

use crate::{GatewayError, Result};

/// Validate that `base` is a bare `http(s)` origin with an explicit port and no
/// path, query, fragment or userinfo.
pub fn validate_bare_origin(base: &str) -> Result<()> {
    let url = reqwest::Url::parse(base)
        .map_err(|e| GatewayError::InvalidOrigin(format!("invalid base origin {base:?}: {e}")))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_none()
    {
        return Err(GatewayError::InvalidOrigin(format!(
            "base must be a bare http(s) origin with an explicit port and no path/query/userinfo: {base:?}"
        )));
    }

    let host = url
        .host_str()
        .ok_or_else(|| GatewayError::InvalidOrigin(format!("base origin has no host: {base:?}")))?;
    if !is_loopback_host(host) {
        return Err(GatewayError::InvalidOrigin(format!(
            "api_origin host must be loopback (127.0.0.0/8, ::1 or localhost), got {host:?}"
        )));
    }

    Ok(())
}

/// Loopback hosts a gateway may advertise: `127.0.0.0/8`, `::1` and
/// `localhost` (case-insensitive). All other hosts are rejected.
fn is_loopback_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }
    // `Url::host_str()` brackets IPv6 literals, e.g. "[::1]".
    let address = host.trim_start_matches('[').trim_end_matches(']');
    match address.parse::<std::net::IpAddr>() {
        Ok(std::net::IpAddr::V4(ip)) => ip.octets()[0] == 127,
        Ok(std::net::IpAddr::V6(ip)) => ip == std::net::Ipv6Addr::LOCALHOST,
        Err(_) => false,
    }
}

/// Join `path` onto a bare loopback `base`, rejecting all four cross-origin
/// join classes: no leading single `/`, `//host`, userinfo `@`, and absolute
/// foreign URLs. Returns the full URL on success.
pub fn join_api_origin(base: &str, path: &str) -> Result<String> {
    validate_bare_origin(base)?;

    if !path.starts_with('/') {
        return Err(GatewayError::InvalidOrigin(format!(
            "path must start with a single '/': {path:?}"
        )));
    }
    if path.starts_with("//") {
        return Err(GatewayError::InvalidOrigin(format!(
            "protocol-relative '//host' paths are forbidden: {path:?}"
        )));
    }
    if path.contains('@') {
        return Err(GatewayError::InvalidOrigin(format!(
            "userinfo '@' is forbidden in a gateway path: {path:?}"
        )));
    }

    let base_url = reqwest::Url::parse(base)
        .map_err(|e| GatewayError::InvalidOrigin(format!("invalid base origin {base:?}: {e}")))?;
    let joined = base_url.join(path).map_err(|e| {
        GatewayError::InvalidOrigin(format!("cannot join {path:?} onto {base:?}: {e}"))
    })?;

    if joined.origin() != base_url.origin()
        || !joined.username().is_empty()
        || joined.password().is_some()
    {
        return Err(GatewayError::InvalidOrigin(format!(
            "joined URL {joined} does not stay on origin {base:?}"
        )));
    }

    Ok(joined.to_string())
}
