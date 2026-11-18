//! `hermes-chat` — message model, composer and streaming render.
//!
//! All types are plain Rust values so composer, session parsing and stream
//! folding can be tested headlessly. Chat submission is WebSocket-only via
//! the `prompt.submit` JSON-RPC method; there is no `/api/chat` REST route.

pub mod composer;
pub mod message;
pub mod session_list;
pub mod streaming;

pub use composer::{Composer, PROMPT_SUBMIT_METHOD};
pub use message::{Message, Role};
pub use session_list::{parse_sessions, SessionInfo, SessionList, SESSIONS_PATH};
pub use streaming::{fold_into_message, StreamRenderer};
