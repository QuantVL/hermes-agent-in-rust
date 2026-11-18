# Hermes Agent in Rust — Design

**Date:** 2026-11-18
**Status:** Approved for planning (builder review addressed)
**Repo:** https://github.com/QuantVL/hermes-agent-in-rust

## Intent

A clean-room Rust rewrite of the Hermes Agent desktop application, built from
scratch in GitHub with CI as the only judge of correctness. The Electron shell
is the reference for behavior; the Python gateway is the backend of record.

Why: the previous local attempt failed because it wrote ~50 files against
unverified APIs (git dependencies that 404, an invented chat protocol,
duplicate `From` impls, Windows toolchain traps) and only tried to compile at
the end. This design makes that failure mode structurally impossible.

## Goals

- Feature parity with the Electron desktop shell at the UI level (main chat
  window, HUD, overlays, plugin system, updater).
- Every commit independently verifiable by GitHub Actions.
- Talk to the **real** Python gateway over its actual interface — no invented
  protocol.
- Native look and feel (GPUI), matching Approach A.

## Non-Goals

- Rewriting the Python gateway itself. It stays the backend of record.
- Bug-for-bug fidelity to Electron rendering quirks.
- Building locally. Source of truth and compilation both live in GitHub.

## Hard Constraints (non-negotiable)

1. **CI gate per commit.** Every commit passes `cargo fmt --check`,
   `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo check --workspace --all-targets`, and `cargo test --workspace` on
   `ubuntu-latest` before the next commit is written.
2. **crates.io only.** No git dependencies. Ever. (Auth/404 failures.)
3. **Compile early.** Any new API surface is proven by a minimal spike commit
   before substantial code is written against it.
4. **One `#[from]` per source error type.** (E0119.)
5. **Derive, don't hand-roll.** `Default` on enums via `#[default]`.
6. Linux CI runner avoids the Windows `dlltool.exe` / linker trap entirely.

## Decisions Already Made

### D1 — Gateway interface (recon-verified)

Recon of the actual source settled this. The Rust client must:

1. **Discover** via `hermes gateway ensure --json` subprocess. stdout is one
   JSON line. Top-level keys: `state`, `endpoint` (object|null), `reason_code`,
   `detail`, `client_code_sha`.
   `GatewayEndpoint` fields (`hermes_cli/gateway_runtime.py:23-39`):
   `profile_id`, `instance_id`, `authority_epoch` (>0), `runtime_protocol`
   (must == 1), `api_origin` (bare loopback http(s) origin with explicit port,
   no path/query/userinfo), `supervisor`, `capabilities` (must contain
   `session-authority-v1`), `control_home`, `code_sha`.
   Exit codes: ready=0, invalid=2, incompatible=3, authorization=4,
   starting/deadline=5, draining=6, inaccessible/conflict=7.
   Stale-owner detection compares `endpoint.code_sha` vs `client_code_sha`.

2. **Tickets via NATIVE control socket (decided, overrides the earlier
   subprocess choice).** `hermes gateway attach` was rejected because it mints
   only `purpose="interactive"` (`gateway_runtime_attach.py:92`,
   `gateway_client.py:284`) — insufficient for REST. Rust implements the
   `session-ticket` verb directly over the private control socket:
   - Socket path: `<home>/control.sock` inside
     `hermes-gw-<_home_hash(home)>` (`gateway_runtime_discovery.py:163-167`);
     on Windows use `query_runtime_control` (`runtime_bootstrap_windows.py`).
   - Peer auth: same OS user only (the socket authenticates the peer).
   - Request: one newline-terminated JSON line,
     `{"protocol":1,"id":1,"verb":"session-ticket","params":{"profile_id":..,
     "instance_id":..,"purpose":..}}` (`gateway_client.py:293-297`).
   - Response: one JSON line with `ticket`, validated against
     `profile_id`/`instance_id`/`runtime_protocol==1` (`gateway_client.py:305-310`).
   - Purposes: `interactive` (WS), `native-http` (REST), `native-profile-create`,
     `native-profile-registry`.
   - Tickets are **single-use** and expire ~30s after minting; the client holds
     ONE refresh layer that serializes request→mint→send (no caller caches).
   - Discovery supplies `control_home` for multiplexer-served profiles.

3. **REST** over `api_origin`. All ~90 endpoints carry a **fresh, one-use**
   ticket in header `x-hermes-gateway-ticket`, with **no `Origin` header**
   (any Origin, even empty, disqualifies the request). Representative endpoints:
   - `GET  /api/status` (liveness/readiness)
   - `GET  /api/sessions`, `/api/sessions/{id}/messages`, `PATCH/DELETE /api/sessions/{id}`
   - `GET/PUT /api/config`, `/api/config/defaults`, `/api/config/schema`
   - `GET  /api/profiles`, `POST /api/profiles`, `GET/PUT /api/profiles/{name}/soul`
   - `GET  /api/model/info`, `POST /api/model/set`, `GET /api/model/options`
   - `GET/POST/PUT/DELETE /api/cron/jobs` (+ pause/resume/trigger/cancel)
   - `GET/POST /api/plugins`, `POST /api/ops/backup|doctor|security-audit`
   - `POST /api/audio/transcribe|speak`, `GET /api/audio/elevenlabs/voices`
   - `GET  /api/logs`, `GET/PUT /api/env`, `GET /api/memory`
   There is **no `/api/chat` REST endpoint** — chat input is WS only.

4. **Chat / streaming** over WS at `api_origin` with `http->ws` + `/api/ws`
   (`gateway_client.py:39-45`). Auth: ticket stripped from URL and presented as
   subprotocol `hermes-gateway-ticket.<ticket>` alongside `hermes-gateway-v1`.
   Core RPC methods (`apps/desktop/src/api/client.ts:46-75`,
   `canonical-protocol.ts:318-325,424-441`): `prompt.submit`,
   `session.create/resume/activate/rewind/branch/mutate`,
   `session.interrupt/redirect/steer`, `session.events.since`,
   `approval.respond`, `clarify.respond`, `config.set`, `gateway.ping`.
   Streaming events (`gateway-contract.generated.ts:7467-7547`):
   `message.start/delta/complete`, `reasoning.delta`, `thinking.delta`,
   `voice.partial`, `tool.start/progress/complete/generating/output_risk`,
   `subagent.tool`, plus client-local events (`approval.request`,
   `gateway.reconnecting`, `gateway.stderr`, …). No generic `token`/`delta`
   events. Ticket charset must be a valid RFC 6455 subprotocol token — validate
   at dial time; reject/fail loudly if it contains `=`, `/`, or whitespace.

5. **Origin locking.** Reproduce `joinBackendApiUrl`
   (`apps/desktop/electron/backend-api-url.ts:11-50`) in Rust: reject paths
   that do not start with a single `/`, `//host`, userinfo (`@host`), or any
   path that changes the URL origin.

### D2 — UI framework: GPUI 0.2.2 with a vertical-slice spike

**Verified on crates.io (2026-11-18):** `gpui` latest is `0.2.2`, published
2025-10-22 by Zed maintainer mikayla-maki, 63,048 Rust LOC, edition 2024,
`repository = zed-industries/zed`. It is the genuine Zed framework, not a stub.
It is, however, stale relative to Zed `main`: upstream issue #55271 confirms
current GPUI cannot be published (`gpui_macros`/`gpui_util` are `publish=false`),
so docs.rs examples from `main` will not compile against `0.2.2`. All UI code
targets the `0.2.2` API specifically (docs.rs/crate/gpui/0.2.2).

The spike (step 2) is therefore a **headless vertical slice**, not just a
window type: window + text render + one action/event + a tokio task completing
a network call, proving the async bridge. If it fails, pivot cost is a full UI
restack to egui (NOT "one commit"). If it passes, build on it.

### D3 — Async architecture (decided)

Networking (`reqwest`, `tokio-tungstenite`) requires tokio; GPUI has its own
executor. Decision: run a tokio runtime on a dedicated background thread;
bridge results into GPUI via a channel plus `cx.notify()` on the GPUI side.
The spike must prove this bridge before UI crates are written.

### D4 — Repository layout: workspace of small crates

Current: `hermes-core` (done, protocol-agnostic — generic JSON-RPC envelopes,
config, errors; reconciled against D1, no rework needed). Planned order, each
its own CI-gated commit:

| Step | Crate | Depends on | Scope |
|------|-------|-----------|-------|
| 0 | `hermes-core` | — | DONE: config, JSON-RPC envelopes, errors |
| 1 | GPUI spike | core | headless vertical slice (compile proof, no product code) |
| 2 | `hermes-gateway` | core | discovery, **native control-socket tickets**, REST client, WS stream, origin lock, mock-server contract tests |
| 3 | `hermes-gpui` | core | Theme tokens, window types, app shell skeleton |
| 4 | `hermes-chat` | core, gpui, gateway | Message model, composer, streaming event render |
| 5 | `hermes-shell` | chat, gpui, gateway | Sidebar, titlebar, statusbar, settings, command palette, plugin host API |
| 6 | `hermes-hud` | chat, gpui, **gateway** | Floating HUD + session switcher |
| 7 | `hermes-overlay` | chat, gpui, **gateway** | Pet, quick entry (prompt submit), wake indicator |
| 8 | `hermes-plugins` | core, **shell** | Manifest, context, registry, loader (host API defined in step 5) |
| 9 | `hermes-update` | core, **gateway** | Update manifest/check/install (no Electron config migration — config is owned by gateway via `/api/config`) |
| 10 | Binary | all | `hermes-rust` CLI wiring (`--window`, `--profile`, etc.) |

## Builder Questions (resolved)

- **Q1 — WS client crate:** `tokio-tungstenite` with rustls, no native TLS.
- **Q2 — Async runtime:** `tokio` minimal features (`rt-multi-thread`, `net`,
  `macros`, `time`, `sync`); HTTP via `reqwest` with `rustls-tls`, no default
  features. One runtime, one TLS stack.
- **Q3 — GPUI spike:** `gpui = "=0.2.2"` pinned exactly; vertical slice as above.
- **Q4 — Update strategy:** pinned manifest check + shell-based install; no
  self-update binary in v1, no config migration (gateway owns config).

## Risks

| Risk | Mitigation |
|------|-----------|
| GPUI 0.2.2 API stale vs main | Target 0.2.2 docs specifically; vertical-slice spike proves needed APIs exist |
| Python gateway interface drifts | Pin `runtime_protocol==1`, `code_sha` check, fail loudly |
| Tickets single-use, 30s expiry | Central refresh/serialization layer in `hermes-gateway`; never cache in callers |
| CI cannot run the Python gateway | Mock control socket + mock HTTP/WS server contract tests derived from D1 |
| Native control-socket protocol port drift | Contract tests pin exact JSON frames; fail loudly on identity mismatch |
| Scope creep across 11 commits | One crate per commit, no exceptions |

## Acceptance Criteria

- Steps 1–10 land as separate commits (step 0 is pre-landing), each with a
  green CI run.
- At step 10, `hermes-rust --window=main` connects to a running Python gateway
  (discovery + native ticket + WS `prompt.submit`) and renders a streamed reply.
- No git dependencies in any `Cargo.toml`.
- `cargo clippy -D warnings` green across the whole workspace at every step.
- Origin-lock, no-Origin-header, and ticket-id-validation rules unit-tested.
- Step 10 end-to-end is verified **manually** against a live Python gateway;
  CI proves compile + unit + mock-contract correctness only (gateway absent on
  ubuntu-latest).
