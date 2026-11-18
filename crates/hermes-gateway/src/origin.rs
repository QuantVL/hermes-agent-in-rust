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
    Ok(())
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
