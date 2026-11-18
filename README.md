# Hermes Agent in Rust

A clean-room Rust rewrite of the Hermes Agent desktop application.

## Architecture

This workspace is being built incrementally. Each crate is added only after
the previous one passes CI (`cargo fmt`, `cargo clippy -D warnings`,
`cargo check`, `cargo test`) on GitHub Actions.

Current crates:

| Crate | Status | Purpose |
|-------|--------|---------|
| `hermes-core` | in progress | Config, JSON-RPC protocol envelopes, error types |

## Development

```bash
cargo check --workspace --all-targets
cargo test --workspace
```

CI runs on every push and pull request to `main`.
