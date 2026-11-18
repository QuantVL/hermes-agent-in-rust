//! Contract tests for `hermes-gateway`.
//!
//! Every mock is in-process (a Unix control socket, a raw TCP HTTP server and a
//! tokio-tungstenite WS server), so CI needs no Python gateway and no `hermes`
//! binary on PATH. The control-socket tests are `#[cfg(unix)]`: CI is ubuntu and
//! `control.rs` exposes a cfg-gated `unsupported` stub on Windows.

#[cfg(unix)]
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use hermes_gateway::{
    join_api_origin, parse_ensure_response, validate_ticket_token, ErrorCode, GatewayError,
};
#[cfg(unix)]
use hermes_gateway::{GatewayClient, GatewayEndpoint, GatewayEvent, TicketPurpose};
use serde_json::{json, Value};

#[cfg(unix)]
use std::collections::VecDeque;
#[cfg(unix)]
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::net::TcpListener;
#[cfg(unix)]
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::sync::atomic::{AtomicUsize, Ordering};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

#[cfg(unix)]
fn ready_endpoint(api_origin: &str, control_home: &Path) -> GatewayEndpoint {
    GatewayEndpoint {
        profile_id: "profile-a".into(),
        instance_id: "instance-a".into(),
        authority_epoch: 1,
        runtime_protocol: 1,
        api_origin: api_origin.into(),
        supervisor: Value::Null,
        capabilities: vec!["session-authority-v1".into()],
        control_home: Some(control_home.to_path_buf()),
        code_sha: "sha-test".into(),
    }
}

fn base_endpoint_value() -> Value {
    json!({
        "profile_id": "profile-a",
        "instance_id": "instance-a",
        "authority_epoch": 1,
        "runtime_protocol": 1,
        "api_origin": "http://127.0.0.1:1234",
        "supervisor": { "kind": "test" },
        "capabilities": ["session-authority-v1"],
        "control_home": "/tmp/hermes-gw-test",
        "code_sha": "sha-test"
    })
}

fn envelope(endpoint: Value, state: &str) -> String {
    json!({
        "state": state,
        "endpoint": endpoint,
        "reason_code": Value::Null,
        "detail": Value::Null,
        "client_code_sha": "client-sha"
    })
    .to_string()
}

/// Spawn a mock control socket that replies to `session-ticket` frames.
/// Returns the temp dir (socket lives at `<dir>/control.sock`) and the list of
/// decoded request frames it received.
#[cfg(unix)]
fn spawn_mock_control(
    tickets: &[&str],
    mismatch_profile: bool,
) -> (tempfile::TempDir, Arc<Mutex<Vec<Value>>>) {
    use std::os::unix::net::UnixListener;

    let dir = tempfile::tempdir().expect("tempdir");
    let socket_path: PathBuf = dir.path().join("control.sock");
    let listener = UnixListener::bind(&socket_path).expect("bind control socket");
    let requests = Arc::new(Mutex::new(Vec::new()));
    let requests_c = requests.clone();
    let mut queue: VecDeque<String> = tickets.iter().map(|t| (*t).to_string()).collect();

    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            let mut reader = match stream.try_clone() {
                Ok(s) => BufReader::new(s),
                Err(_) => continue,
            };
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                continue;
            }
            let request: Value = match serde_json::from_str(line.trim()) {
                Ok(v) => v,
                Err(_) => continue,
            };
            requests_c.lock().unwrap().push(request);

            let ticket = queue
                .pop_front()
                .unwrap_or_else(|| "fallback-ticket".into());
            let response = json!({
                "ticket": ticket,
                "profile_id": if mismatch_profile { "wrong-profile" } else { "profile-a" },
                "instance_id": "instance-a",
                "runtime_protocol": 1
            });
            let _ = writeln!(stream, "{response}");
            let _ = stream.flush();
        }
    });

    (dir, requests)
}

/// Minimal raw HTTP/1.1 server; records lower-cased header pairs per request.
#[cfg(unix)]
fn spawn_mock_http(requests: usize) -> (String, Arc<Mutex<Vec<Vec<(String, String)>>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind http");
    let port = listener.local_addr().unwrap().port();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let seen_c = seen.clone();

    std::thread::spawn(move || {
        for _ in 0..requests {
            let Ok((mut stream, _)) = listener.accept() else {
                break;
            };
            let mut headers = Vec::new();
            let mut reader = match stream.try_clone() {
                Ok(s) => BufReader::new(s),
                Err(_) => continue,
            };
            let mut line = String::new();
            loop {
                line.clear();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
                let l = line.trim_end().to_string();
                if l.is_empty() {
                    break;
                }
                if let Some((k, v)) = l.split_once(':') {
                    headers.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
                }
            }
            seen_c.lock().unwrap().push(headers);
            let body = "{}";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });

    (format!("http://127.0.0.1:{port}"), seen)
}

#[cfg(unix)]
fn header(headers: &[(String, String)], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.clone())
}

// ---------------------------------------------------------------------------
// 1. Origin lock
// ---------------------------------------------------------------------------

#[test]
fn origin_rejects_protocol_relative_double_slash_host() {
    let err = join_api_origin("http://127.0.0.1:1234", "//evil").unwrap_err();
    assert!(matches!(err, GatewayError::InvalidOrigin(_)), "{err:?}");
}

#[test]
fn origin_rejects_userinfo() {
    let err = join_api_origin("http://127.0.0.1:1234", "/@evil").unwrap_err();
    assert!(matches!(err, GatewayError::InvalidOrigin(_)), "{err:?}");
}

#[test]
fn origin_rejects_absolute_foreign_url() {
    let err = join_api_origin("http://127.0.0.1:1234", "http://other/").unwrap_err();
    assert!(matches!(err, GatewayError::InvalidOrigin(_)), "{err:?}");
}

#[test]
fn origin_rejects_path_without_leading_slash() {
    let err = join_api_origin("http://127.0.0.1:1234", "api/status").unwrap_err();
    assert!(matches!(err, GatewayError::InvalidOrigin(_)), "{err:?}");
}

#[test]
fn origin_accepts_single_slash_path() {
    let url = join_api_origin("http://127.0.0.1:1234", "/api/status").unwrap();
    assert_eq!(url, "http://127.0.0.1:1234/api/status");
}

// ---------------------------------------------------------------------------
// 2. Discovery validation (pure validator used by `discover`)
// ---------------------------------------------------------------------------

#[test]
fn discovery_accepts_valid_ready_payload() {
    let line = envelope(base_endpoint_value(), "ready");
    let endpoint = parse_ensure_response(&line).unwrap();
    assert_eq!(endpoint.profile_id, "profile-a");
    assert_eq!(endpoint.instance_id, "instance-a");
    assert_eq!(endpoint.authority_epoch, 1);
    assert_eq!(endpoint.runtime_protocol, 1);
    assert_eq!(endpoint.api_origin, "http://127.0.0.1:1234");
    assert_eq!(endpoint.capabilities, vec!["session-authority-v1"]);
    assert_eq!(endpoint.code_sha, "sha-test");
    assert!(endpoint.control_home.is_some());
}

#[test]
fn discovery_rejects_runtime_protocol_2() {
    let mut endpoint = base_endpoint_value();
    endpoint["runtime_protocol"] = json!(2);
    let err = parse_ensure_response(&envelope(endpoint, "ready")).unwrap_err();
    assert!(matches!(err, GatewayError::Incompatible(_)), "{err:?}");
}

#[test]
fn discovery_rejects_missing_session_authority_capability() {
    let mut endpoint = base_endpoint_value();
    endpoint["capabilities"] = json!(["some-other-cap"]);
    let err = parse_ensure_response(&envelope(endpoint, "ready")).unwrap_err();
    assert!(matches!(err, GatewayError::InvalidState(_)), "{err:?}");
}

#[test]
fn discovery_rejects_api_origin_with_path() {
    let mut endpoint = base_endpoint_value();
    endpoint["api_origin"] = json!("http://127.0.0.1:1234/extra");
    let err = parse_ensure_response(&envelope(endpoint, "ready")).unwrap_err();
    assert!(matches!(err, GatewayError::InvalidOrigin(_)), "{err:?}");
}

#[test]
fn discovery_rejects_zero_authority_epoch() {
    let mut endpoint = base_endpoint_value();
    endpoint["authority_epoch"] = json!(0);
    let err = parse_ensure_response(&envelope(endpoint, "ready")).unwrap_err();
    assert!(matches!(err, GatewayError::InvalidState(_)), "{err:?}");
}

#[test]
fn discovery_maps_non_ready_state_to_error_code() {
    let err = parse_ensure_response(&envelope(base_endpoint_value(), "incompatible")).unwrap_err();
    assert!(
        matches!(
            err,
            GatewayError::ExitCode {
                error_code: Some(ErrorCode::Incompatible),
                ..
            }
        ),
        "{err:?}"
    );
}

#[test]
fn error_code_maps_all_ensure_exit_codes() {
    assert_eq!(ErrorCode::from_exit_code(2), Some(ErrorCode::Invalid));
    assert_eq!(ErrorCode::from_exit_code(3), Some(ErrorCode::Incompatible));
    assert_eq!(ErrorCode::from_exit_code(4), Some(ErrorCode::Authorization));
    assert_eq!(ErrorCode::from_exit_code(5), Some(ErrorCode::Starting));
    assert_eq!(ErrorCode::from_exit_code(6), Some(ErrorCode::Draining));
    assert_eq!(ErrorCode::from_exit_code(7), Some(ErrorCode::Inaccessible));
    assert_eq!(ErrorCode::from_exit_code(0), None);
    assert_eq!(ErrorCode::from_exit_code(42), None);
    assert_eq!(ErrorCode::Invalid.as_i32(), 2);
}

// ---------------------------------------------------------------------------
// 3. Native control-socket tickets
// ---------------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn ticket_mint_sends_native_http_purpose_on_the_wire() {
    let (dir, requests) = spawn_mock_control(&["ticket-1"], false);
    let broker = hermes_gateway::TicketBroker::new(dir.path(), "profile-a", "instance-a");
    let ticket = broker.mint(TicketPurpose::NativeHttp).unwrap();
    assert_eq!(ticket.as_str(), "ticket-1");

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(
        requests[0],
        json!({
            "protocol": 1,
            "id": 1,
            "verb": "session-ticket",
            "params": {
                "profile_id": "profile-a",
                "instance_id": "instance-a",
                "purpose": "native-http"
            }
        })
    );
}

#[cfg(unix)]
#[test]
fn ticket_mint_uses_all_four_wire_purposes() {
    let (dir, requests) = spawn_mock_control(&["t1", "t2", "t3", "t4"], false);
    let broker = hermes_gateway::TicketBroker::new(dir.path(), "profile-a", "instance-a");
    for purpose in [
        TicketPurpose::Interactive,
        TicketPurpose::NativeHttp,
        TicketPurpose::NativeProfileCreate,
        TicketPurpose::NativeProfileRegistry,
    ] {
        broker.mint(purpose).unwrap();
    }
    let requests = requests.lock().unwrap();
    let purposes: Vec<&str> = requests
        .iter()
        .map(|r| r["params"]["purpose"].as_str().unwrap())
        .collect();
    assert_eq!(
        purposes,
        vec![
            "interactive",
            "native-http",
            "native-profile-create",
            "native-profile-registry"
        ]
    );
}

#[cfg(unix)]
#[test]
fn two_consecutive_mints_return_distinct_tickets() {
    let (dir, _requests) = spawn_mock_control(&["ticket-a", "ticket-b"], false);
    let broker = hermes_gateway::TicketBroker::new(dir.path(), "profile-a", "instance-a");
    let first = broker.mint(TicketPurpose::NativeHttp).unwrap();
    let second = broker.mint(TicketPurpose::NativeHttp).unwrap();
    assert_ne!(
        first.as_str(),
        second.as_str(),
        "tickets must never be cached"
    );
    assert_eq!(first.as_str(), "ticket-a");
    assert_eq!(second.as_str(), "ticket-b");
}

#[cfg(unix)]
#[test]
fn ticket_mint_rejects_mismatched_profile_id() {
    let (dir, _requests) = spawn_mock_control(&["ticket-1"], true);
    let broker = hermes_gateway::TicketBroker::new(dir.path(), "profile-a", "instance-a");
    let err = broker.mint(TicketPurpose::NativeHttp).unwrap_err();
    assert!(matches!(err, GatewayError::InvalidState(_)), "{err:?}");
}

#[test]
fn ticket_token_validation_accepts_rfc6455_tokens_only() {
    assert!(validate_ticket_token("wsticket-1").is_ok());
    assert!(validate_ticket_token("abc_DEF.123~x").is_ok());
    for bad in ["abc==", "a/b", "a b", "a\tb", "", "a=b", "a?b"] {
        assert!(
            matches!(
                validate_ticket_token(bad),
                Err(GatewayError::InvalidTicket(_))
            ),
            "{bad:?} should be rejected"
        );
    }
}

// ---------------------------------------------------------------------------
// 4. REST: fresh ticket header, never an Origin header
// ---------------------------------------------------------------------------

#[cfg(unix)]
#[test]
fn rest_get_sends_fresh_ticket_and_no_origin_header() {
    let (base, seen) = spawn_mock_http(2);
    let (dir, _requests) = spawn_mock_control(&["rest-ticket-1", "rest-ticket-2"], false);
    let endpoint = ready_endpoint(&base, dir.path());
    let client = GatewayClient::new(&endpoint).unwrap();

    assert_eq!(client.get("/api/status").unwrap(), json!({}));
    assert_eq!(client.get("/api/status").unwrap(), json!({}));

    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 2, "two GETs must reach the server");
    let first_ticket = header(&seen[0], "x-hermes-gateway-ticket").expect("first ticket header");
    let second_ticket = header(&seen[1], "x-hermes-gateway-ticket").expect("second ticket header");
    assert_eq!(first_ticket, "rest-ticket-1");
    assert_eq!(second_ticket, "rest-ticket-2");
    assert_ne!(
        first_ticket, second_ticket,
        "ticket must be fresh per request"
    );
    assert!(header(&seen[0], "origin").is_none(), "never send Origin");
    assert!(header(&seen[1], "origin").is_none(), "never send Origin");
}

// ---------------------------------------------------------------------------
// 5. WS: ticket-as-subprotocol, typed events, invalid charset fails pre-dial
// ---------------------------------------------------------------------------

#[cfg(unix)]
#[tokio::test]
async fn ws_connect_uses_both_subprotocols_and_parses_message_delta() {
    use futures::{SinkExt, StreamExt};
    use tokio::net::TcpListener as TokioTcpListener;
    use tokio_tungstenite::tungstenite::handshake::server::{ErrorResponse, Request, Response};
    use tokio_tungstenite::tungstenite::Message;

    let listener = TokioTcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let captured: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let captured_c = captured.clone();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let callback = move |req: &Request, resp: Response| -> Result<Response, ErrorResponse> {
            *captured_c.lock().unwrap() = req
                .headers()
                .get("sec-websocket-protocol")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);
            Ok(resp)
        };
        let mut ws = tokio_tungstenite::accept_hdr_async(stream, callback)
            .await
            .expect("ws handshake");
        ws.send(Message::Text(
            r#"{"event":"message.delta","delta":"hello"}"#.into(),
        ))
        .await
        .unwrap();
        tokio::time::sleep(Duration::from_millis(500)).await;
        let _ = ws.close(None).await;
    });

    let (dir, _requests) = spawn_mock_control(&["wsticket-1"], false);
    let base = format!("http://127.0.0.1:{port}");
    let endpoint = ready_endpoint(&base, dir.path());
    let client = GatewayClient::new(&endpoint).unwrap();
    let mut stream = client.ws_connect().unwrap();

    let event = tokio::time::timeout(Duration::from_secs(5), stream.next())
        .await
        .expect("timed out waiting for WS event")
        .expect("stream ended");
    assert_eq!(event, GatewayEvent::MessageDelta("hello".into()));

    // Give the server callback time to run before reading what it captured.
    tokio::time::sleep(Duration::from_millis(100)).await;
    let protocols = captured.lock().unwrap().clone().unwrap_or_default();
    assert!(
        protocols.contains("hermes-gateway-v1"),
        "protocols = {protocols:?}"
    );
    assert!(
        protocols.contains("hermes-gateway-ticket.wsticket-1"),
        "protocols = {protocols:?}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn ws_connect_rejects_invalid_ticket_charset_before_dialing() {
    use tokio::net::TcpListener as TokioTcpListener;

    let listener = TokioTcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let accepts = Arc::new(AtomicUsize::new(0));
    let accepts_c = accepts.clone();
    tokio::spawn(async move {
        loop {
            if listener.accept().await.is_ok() {
                accepts_c.fetch_add(1, Ordering::SeqCst);
            }
        }
    });

    let (dir, _requests) = spawn_mock_control(&["bad==ticket"], false);
    let base = format!("http://127.0.0.1:{port}");
    let endpoint = ready_endpoint(&base, dir.path());
    let client = GatewayClient::new(&endpoint).unwrap();

    let err = client.ws_connect().unwrap_err();
    assert!(matches!(err, GatewayError::InvalidTicket(_)), "{err:?}");
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(
        accepts.load(Ordering::SeqCst),
        0,
        "must not dial on invalid ticket"
    );
}

// Keep imports used on every platform even when the unix-only tests are gated.
#[test]
fn fixtures_are_deterministic() {
    let a = base_endpoint_value();
    let b = base_endpoint_value();
    assert_eq!(a, b);
    let _ = (UNIX_EPOCH, SystemTime::now(), Duration::from_secs(1));
}
