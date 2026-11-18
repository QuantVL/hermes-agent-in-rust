//! Fold typed gateway events into the chat transcript.
//!
//! Every [`GatewayEvent`] variant is mapped explicitly. In particular there
//! is no generic `token`/`delta` event: visible reply text arrives only via
//! `MessageDelta`, exactly as the gateway's real vocabulary defines it.

use hermes_gateway::GatewayEvent;

use crate::Message;

/// Accumulates one streamed assistant reply plus the completed transcript.
#[derive(Debug, Clone, Default)]
pub struct StreamRenderer {
    /// Message currently being streamed into.
    pub current: Message,
    /// Completed messages, oldest first.
    pub messages: Vec<Message>,
    /// Reasoning text (kept separate from the visible answer).
    pub reasoning: String,
    /// Thinking text (kept separate from the visible answer).
    pub thinking: String,
    /// Partial voice transcript.
    pub voice_partial: String,
    /// Tool and subagent-tool events observed during the stream.
    pub tool_events: Vec<GatewayEvent>,
    /// Unrecognised events retained for diagnostics.
    pub other_events: Vec<GatewayEvent>,
    /// True while a `message.start`/`message.complete` pair is open.
    pub streaming: bool,
}

impl StreamRenderer {
    pub fn new() -> Self {
        Self {
            current: Message::assistant(""),
            ..Self::default()
        }
    }

    /// Fold one gateway event into the renderer state.
    pub fn render_stream(&mut self, ev: GatewayEvent) {
        match &ev {
            GatewayEvent::Connected => {}
            GatewayEvent::Disconnected(_) => self.streaming = false,
            GatewayEvent::MessageStart { .. } => {
                self.finish_current();
                self.current = Message::assistant("");
                self.streaming = true;
            }
            GatewayEvent::MessageDelta(_) => {
                fold_into_message(&mut self.current, &ev);
            }
            GatewayEvent::MessageComplete { .. } => {
                self.finish_current();
                self.streaming = false;
            }
            GatewayEvent::ReasoningDelta(delta) => self.reasoning.push_str(delta),
            GatewayEvent::ThinkingDelta(delta) => self.thinking.push_str(delta),
            GatewayEvent::VoicePartial(delta) => self.voice_partial.push_str(delta),
            GatewayEvent::ToolStart { .. }
            | GatewayEvent::ToolProgress { .. }
            | GatewayEvent::ToolComplete { .. }
            | GatewayEvent::ToolGenerating { .. }
            | GatewayEvent::ToolOutputRisk { .. }
            | GatewayEvent::SubagentTool(_) => self.tool_events.push(ev.clone()),
            GatewayEvent::Other(_) => self.other_events.push(ev.clone()),
        }
    }

    fn finish_current(&mut self) {
        if !self.current.content.is_empty() {
            self.messages.push(std::mem::take(&mut self.current));
        }
    }
}

/// Pure single-event fold into one message.
///
/// Returns `true` only when `ev` carried visible message text. All other
/// variants leave the message untouched.
pub fn fold_into_message(message: &mut Message, ev: &GatewayEvent) -> bool {
    match ev {
        GatewayEvent::MessageDelta(delta) => {
            message.append(delta);
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_message_delta_mutates_a_message() {
        let mut message = Message::assistant("");
        assert!(!fold_into_message(
            &mut message,
            &GatewayEvent::ReasoningDelta("r".into())
        ));
        assert!(fold_into_message(
            &mut message,
            &GatewayEvent::MessageDelta("a".into())
        ));
        assert_eq!(message.content, "a");
    }
}
