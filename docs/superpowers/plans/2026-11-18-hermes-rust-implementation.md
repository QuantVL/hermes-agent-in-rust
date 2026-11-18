# Hermes Agent in Rust Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite the Hermes Agent desktop shell in Rust as a GitHub-native workspace of small crates, each landing as an independently CI-green commit, ending with a binary that connects to the real Python gateway.

**Architecture:** A Cargo workspace. `hermes-core` holds protocol-agnostic config/JSON-RPC envelopes/errors. `hermes-gateway` owns all Python-gateway interaction: discovery via subprocess, ticket minting via a native private control socket, REST over `api_origin`, and JSON-RPC streaming over `/api/ws`. GPUI 0.2.2 is the UI framework, proven by a headless vertical-slice spike before any UI crate. Tokio runs on a dedicated thread bridged to GPUI via channel + `cx.notify()`.

**Tech Stack:** Rust stable (2021/2024 editions), serde/serde_yaml/thiserror/anyhow/uuid, reqwest (rustls), tokio (minimal features), tokio-tungstenite (rustls), gpui `=0.2.2` from crates.io.

**Spec:** `docs/superpowers/specs/2026-11-18-hermes-rust-design.md`

## Global Constraints

- CI gate on `ubuntu-latest`: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check --workspace --all-targets`, `cargo test --workspace` — ALL green before the next commit.
- **No git dependencies in any `Cargo.toml`** — crates.io only. `gpui = "=0.2.2"` pinned exactly.
- One `#[from]` per source error type (E0119). Derive `Default` on enums via `#[default]` (clippy).
- Target `0.2.2` GPUI API only — docs.rs examples from `main` do not compile.
- Ticket minting: native control socket, `session-ticket` verb, one newline-terminated JSON frame, single-use tickets (~30s), central refresh layer. Never cache tickets in callers.
- REST: fresh `x-hermes-gateway-ticket` header per request, **no `Origin` header**.
- WS: ticket as subprotocol `hermes-gateway-ticket.<ticket>` + `hermes-gateway-v1`; validate ticket is a valid RFC 6455 token (reject `=`, `/`, whitespace).
- Origin lock: reject API paths not starting with a single `/`, `//host`, userinfo, or cross-origin.
- One crate per commit. No local build required; GitHub Actions is the judge.

## Review Focus

- **Ticket expiry/concurrency:** two concurrent REST calls must not reuse one ticket — contract test with a mock control socket asserts a fresh mint per request, serialized.
- **Malicious API path (`//evil`, `@host`):** must be rejected before any network I/O — unit test in `hermes-gateway`.
- **Gateway protocol drift (`runtime_protocol != 1`, missing `session-authority-v1`):** must fail discovery loudly — discovery unit tests.
- **WS subprotocol invalid ticket charset:** must fail at dial, not hang — dial unit test.
- **CI cannot reach a real gateway:** mock control socket + mock HTTP/WS server must prove the protocol without Python — contract tests in step 2.

---

### Task 1: GPUI vertical-slice spike

**Files:**
- Modify: `Cargo.toml` (workspace deps: `gpui = "=0.2.2"`, `tokio` minimal, `reqwest` rustls)
- Create: `crates/hermes-gpui/Cargo.toml`, `crates/hermes-gpui/src/lib.rs`, `crates/hermes-gpui/tests/spike.rs`

**Interfaces:**
- Produces: proof that `gpui 0.2.2` supports window + text render + action + tokio bridge on `ubuntu-latest`.

- [ ] **Step 1: Add pinned gpui, tokio, reqwest to workspace deps**

In root `Cargo.toml` `[workspace.dependencies]` add exactly:
`gpui = "=0.2.2"`, `tokio = { version = "1", features = ["rt-multi-thread","net","macros","time","sync"] }`, `reqwest = { version = "0.12", default-features = false, features = ["json","rustls-tls"] }`, `tokio-tungstenite = { version = "0.24", features = ["rustls-tls-webpki-roots"] }`.

- [ ] **Step 2: Write the spike test**

`crates/hermes-gpui/tests/spike.rs`: with `gpui::test::TestAppContext`, construct a window, render a text node `"spike"`, dispatch one action, and spawn a tokio task that completes a `reqwest` GET to a local mock server. Assert the rendered text and that the task result arrives via a `tokio::sync::mpsc` channel and is received inside a GPUI `cx.notify()` callback.

- [ ] **Step 3: Run the local pre-flight gate**

Run: `cd <scratch>/hermes-rust-build && cargo fmt --all && cargo test -p hermes-gpui --test spike 2>&1 | tail -30`
Expected: PASS. If `gpui 0.2.2` lacks any required API, STOP — pivot to egui and restack tasks 3–7 (per spec D2).

- [ ] **Step 4: Commit and push, verify CI green**

`git add -A && git commit -m "spike: prove gpui 0.2.2 window/text/action + tokio bridge" && git push`
Then `gh run list --repo QuantVL/hermes-agent-in-rust --limit 1` — wait for `conclusion=success`. If FAIL, do not proceed to Task 2.

---

### Task 2: hermes-gateway — discovery, tickets, REST, WS, contract tests

**Files:**
- Create: `crates/hermes-gateway/Cargo.toml`
- Create: `crates/hermes-gateway/src/lib.rs`, `discovery.rs`, `ticket.rs`, `control.rs`, `rest.rs`, `stream.rs`, `origin.rs`, `error.rs`
- Create: `crates/hermes-gateway/tests/contract.rs`

**Interfaces:**
- Consumes: `hermes_core::{Config, HermesError, Result}` (task 0).
- Produces:
  - `discover(profile: &str) -> Result<GatewayEndpoint>`
  - `GatewayEndpoint { profile_id, instance_id, authority_epoch, runtime_protocol, api_origin, supervisor, capabilities, control_home, code_sha }`
  - `TicketBroker { fn mint(&self, purpose: TicketPurpose) -> Result<Ticket> }` where `TicketPurpose = Interactive | NativeHttp | NativeProfileCreate | NativeProfileRegistry`, `Ticket(String)` with `as_str()`.
  - `GatewayClient { fn get(&self, path: &str) -> Result<serde_json::Value>, fn ws_connect(&self) -> Result<WsStream> }`
  - `join_api_origin(base: &str, path: &str) -> Result<String>`
  - `WsStream` implementing `Stream<Item = GatewayEvent>` where `GatewayEvent` is an enum over the D1 streaming vocabulary.
  - `ErrorCode` enum for ensure exit codes 2/3/4/5/6/7 with documented client behavior.

- [ ] **Step 1: Write failing tests for origin lock and discovery validation**

`tests/contract.rs`: `join_api_origin` rejects `"//evil"`, `"@evil"`, `"http://other/"`, `"/good"` with base `"http://127.0.0.1:1234"` returns full URL. `discover` rejects `runtime_protocol=2`, missing `session-authority-v1`, `api_origin` with a path.

- [ ] **Step 2: Run tests, verify they fail**

Run: `cargo test -p hermes-gateway --test contract 2>&1 | tail -20`
Expected: FAIL — functions/structs not defined.

- [ ] **Step 3: Implement `origin.rs` and `discovery.rs`**

`join_api_origin`: parse base with `reqwest::Url`, enforce single leading `/`, reject `//`, userinfo, and `joined.origin() != base.origin()`. Return string. `discover`: spawn `hermes --profile P gateway ensure --json`, parse one JSON line with serde, validate `runtime_protocol==1`, capability, `api_origin` has no path/query/userinfo, map exit codes.

- [ ] **Step 4: Write failing mock-socket ticket test**

`tests/contract.rs`: spawn a Unix-domain mock control socket in a temp dir that accepts `session-ticket` frames and returns `{ticket, profile_id, instance_id, runtime_protocol:1}`. Assert `TicketBroker::mint(NativeHttp)` sends purpose `"native-http"` in the JSON frame; assert two consecutive mints return two different tickets; assert a response with mismatched `profile_id` is rejected. (Windows path via `query_runtime_control` is cfg-gated out of CI; document it.)

- [ ] **Step 5: Run tests, verify failure**

Run: `cargo test -p hermes-gateway --test contract 2>&1 | tail -20`
Expected: FAIL — `TicketBroker` not defined.

- [ ] **Step 6: Implement `control.rs` and `ticket.rs`**

`control.rs`: `query(home: &Path, request: &str) -> Result<String>` connects to `<home>/control.sock` (Unix: `std::os::unix::net::UnixStream`; Windows: document `query_runtime_control` cfg). Enforce same-user peer, one-line request/response, 5s timeout. `ticket.rs`: `mint(purpose)` builds `{"protocol":1,"id":1,"verb":"session-ticket","params":{profile_id,instance_id,purpose}}`, validates response identity, returns `Ticket`. No caching; caller calls `mint` per request.

- [ ] **Step 7: Write failing REST + WS contract tests**

`tests/contract.rs`: mock HTTP server asserts (a) `GET /api/status` carries `x-hermes-gateway-ticket` header, (b) NO `Origin` header, (c) ticket changes between two calls. Mock WS server asserts subprotocols `hermes-gateway-v1` and `hermes-gateway-ticket.<ticket>`, and that a `message.delta` event parses into `GatewayEvent::MessageDelta`. Assert invalid ticket charset (`"abc=="`) fails before dial.

- [ ] **Step 8: Run tests, verify failure**

Run: `cargo test -p hermes-gateway --test contract 2>&1 | tail -20`
Expected: FAIL — `GatewayClient` not defined.

- [ ] **Step 9: Implement `rest.rs`, `stream.rs`, `error.rs`, `lib.rs`**

`rest.rs`: `get(path)` → `mint(NativeHttp)` → `reqwest` GET with ticket header, no Origin. `stream.rs`: `ws_connect()` → `mint(Interactive)` → validate ticket token → `tokio_tungstenite::connect_async` with subprotocols → `Stream` mapping frames to `GatewayEvent` enum (`MessageStart/Delta/Complete`, `ReasoningDelta`, `ThinkingDelta`, `ToolStart/Progress/Complete`, `Other`). `error.rs`: `GatewayError` with one `#[from]` per source; `ErrorCode` enum for exit codes.

- [ ] **Step 10: Run full local gate, commit, push, verify CI**

Run: `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace 2>&1 | tail -20`
Expected: all PASS. Then `git add -A && git commit -m "feat: hermes-gateway — discovery, native tickets, REST, WS" && git push`; wait for CI success before Task 3.

---

### Task 3: hermes-gpui — theme, windows, app shell skeleton

**Files:**
- Create: `crates/hermes-gpui/src/theme.rs`, `window.rs`, `state.rs`, `app.rs`
- Test: `crates/hermes-gpui/tests/theme.rs`

**Interfaces:**
- Consumes: `hermes_core::Config` (task 0), GPUI 0.2.2 API proven in Task 1.
- Produces: `Theme { colors: ColorTokens, fonts: FontTokens }`, `enum WindowType { Main, Hud, PetOverlay, QuickEntry, WakeIndicator }` with `as_str()`, `GlobalState`, and `run(profile: Option<String>, window: WindowType)`.

- [ ] **Step 1: Write failing theme/window tests**

`tests/theme.rs`: `Theme::default().colors.bg` is non-transparent; `WindowType::Main.as_str() == "main"` and each variant maps to the expected string.

- [ ] **Step 2: Run, verify failure**

`cargo test -p hermes-gpui --test theme 2>&1 | tail -15` → FAIL.

- [ ] **Step 3: Implement theme/window/state/app**

`theme.rs`: `ColorTokens`, `FontTokens`, `Theme::default()`. `window.rs`: `WindowType` with `#[derive(Debug,Clone,Copy,PartialEq)]` and `as_str()`. `state.rs`: `GlobalState` holding `Config` + optional `GatewayClient`. `app.rs`: `run()` initializes GPUI app, opens the requested `WindowType`, uses the Task 1 tokio bridge.

- [ ] **Step 4: Run full gate, commit, push, verify CI**

Same gate as Task 2 Step 10. Commit: `"feat: hermes-gpui — theme, window types, app shell skeleton"`.

---

### Task 4: hermes-chat — message model, composer, streaming render

**Files:**
- Create: `crates/hermes-chat/Cargo.toml` and `src/{message,composer,session_list,streaming,lib}.rs`
- Test: `crates/hermes-chat/tests/render.rs`

**Interfaces:**
- Consumes: `hermes_core::{Request, Notification}`, `hermes_gateway::{GatewayClient, GatewayEvent, WsStream}` (task 2), GPUI from task 3.
- Produces: `Message { role, content, ts }`, `Composer` with `submit() -> Result<Request>`, `SessionList` from `GatewayClient::get("/api/sessions")`, `fn render_stream(&mut self, ev: GatewayEvent)`.

- [ ] **Step 1: Write failing composer + stream mapping tests**

`tests/render.rs`: `Composer::submit("hello")` produces `Request` with method `"prompt.submit"` and params containing the text. `render_stream(GatewayEvent::MessageDelta("hi"))` appends `"hi"` to the current message content.

- [ ] **Step 2: Run, verify failure** → FAIL.

- [ ] **Step 3: Implement message/composer/session_list/streaming**

Map all `GatewayEvent` variants; no generic `token`/`delta`. Composer emits `prompt.submit` per D1.

- [ ] **Step 4: Run full gate, commit, push, verify CI**

Commit: `"feat: hermes-chat — message model, composer, streaming"`.

---

### Task 5: hermes-shell — sidebar, titlebar, statusbar, settings, palette, plugin host API

**Files:**
- Create: `crates/hermes-shell/Cargo.toml` and `src/{app_shell,sidebar,titlebar,statusbar,workspace,command_palette,settings,plugin_host,lib}.rs`
- Test: `crates/hermes-shell/tests/host.rs`

**Interfaces:**
- Consumes: `hermes_chat::{Composer, SessionList}`, `hermes_gateway::GatewayClient`, GPUI.
- Produces: `AppShell`, `PluginHost` trait with `fn register(&self, manifest: hermes_core::PluginManifest)` — **defined here** so task 8 plugins has a stable integration point; `Settings` with 7 tabs.

- [ ] **Step 1: Write failing plugin-host test**

`tests/host.rs`: a mock `PluginHost` accepts a `PluginManifest` and records it; assert registered count == 1.

- [ ] **Step 2: Run, verify failure** → FAIL.

- [ ] **Step 3: Implement app_shell + plugin_host + settings**

`plugin_host.rs`: `trait PluginHost { fn register(&self, m: PluginManifest); fn enabled(&self) -> Vec<PluginManifest>; }`. Note: `PluginManifest` must be added to `hermes-core` in this task (it is protocol-agnostic metadata). Shell composes sidebar/titlebar/statusbar/workspace/palette/settings.

- [ ] **Step 4: Run full gate, commit, push, verify CI**

Commit: `"feat: hermes-shell — shell windows, settings, plugin host API"`.

---

### Tasks 6–10 (condensed; same gate/commit pattern applies)

**Task 6 — hermes-hud** (deps: chat, gpui, **gateway**): floating HUD + session switcher reading `GatewayClient`; test asserts session switch emits `session.activate`. Commit: `"feat: hermes-hud — floating HUD and session switcher"`.

**Task 7 — hermes-overlay** (deps: chat, gpui, **gateway**): pet, quick entry (calls `prompt.submit` via chat), wake indicator; test asserts quick-entry produces a `Request`. Commit: `"feat: hermes-overlay — pet, quick entry, wake indicator"`.

**Task 8 — hermes-plugins** (deps: core, **shell**): manifest/context/registry/loader implementing task 5's `PluginHost`; test asserts a manifest loads and registers. Commit: `"feat: hermes-plugins — manifest, context, registry, loader"`.

**Task 9 — hermes-update** (deps: core, **gateway**): pinned manifest check + shell-based install; no config migration (gateway owns config). Test asserts a manifest version comparison and a rollback command string. Commit: `"feat: hermes-update — manifest check and install"`.

**Task 10 — binary** (deps: all): `hermes-rust` CLI with `--window`, `--profile`, `--config`, `--theme`, `--debug` (clap derive). Test asserts arg parsing. Commit: `"feat: binary — hermes-rust CLI wiring"`. After CI green, perform the **manual** acceptance: run `hermes-rust --window=main` against a live Python gateway and verify a streamed reply renders (CI cannot do this — gateway absent on ubuntu).

---

## Final Verification

- [ ] All commits 1–10 have `conclusion=success` in `gh run list --repo QuantVL/hermes-agent-in-rust`.
- [ ] `grep -rn 'git =' --include=Cargo.toml .` returns nothing (no git deps).
- [ ] Manual step-10 acceptance performed and result recorded in a kanban comment.
