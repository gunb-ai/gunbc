//! Chat completion types for LLM providers.
//!
//! These types provide a unified interface for chat-based LLM APIs.
//! Provider-specific serialization is handled by conversion functions
//! in the provider modules (openai.rs, anthropic.rs).

use serde::{Deserialize, Serialize};

/// Role in a chat conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System instructions that guide the model's behavior.
    System,
    /// User input.
    User,
    /// Assistant (model) response.
    Assistant,
}

impl Role {
    /// Parse a role from a string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "system" => Some(Role::System),
            "user" => Some(Role::User),
            "assistant" => Some(Role::Assistant),
            _ => None,
        }
    }

    /// Get the role as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
        }
    }
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A single message in a chat conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The role of the message author.
    pub role: Role,
    /// The content of the message.
    pub content: String,
}

impl ChatMessage {
    /// Create a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
        }
    }

    /// Create a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
        }
    }

    /// Create an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
        }
    }
}

/// Parameters for a chat completion request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatRequest {
    /// Model identifier (e.g., "gpt-4o", "claude-sonnet-4-20250514").
    pub model: String,
    /// Conversation messages.
    pub messages: Vec<ChatMessage>,
    /// Sampling temperature (0.0 - 2.0). Lower = more deterministic.
    pub temperature: Option<f64>,
    /// Maximum tokens to generate.
    pub max_tokens: Option<u64>,
    /// Stop sequences.
    #[serde(default)]
    pub stop: Vec<String>,
}

impl ChatRequest {
    /// Create a new chat request with the given model and messages.
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            max_tokens: None,
            stop: Vec::new(),
        }
    }

    /// Set the sampling temperature.
    pub fn temperature(mut self, t: f64) -> Self {
        self.temperature = Some(t);
        self
    }

    /// Set the maximum tokens to generate.
    pub fn max_tokens(mut self, n: u64) -> Self {
        self.max_tokens = Some(n);
        self
    }

    /// Add a stop sequence.
    pub fn stop(mut self, s: impl Into<String>) -> Self {
        self.stop.push(s.into());
        self
    }

    /// Get system messages from the conversation.
    pub fn system_messages(&self) -> Vec<&ChatMessage> {
        self.messages
            .iter()
            .filter(|m| m.role == Role::System)
            .collect()
    }

    /// Get non-system messages from the conversation.
    pub fn non_system_messages(&self) -> Vec<&ChatMessage> {
        self.messages
            .iter()
            .filter(|m| m.role != Role::System)
            .collect()
    }
}

/// Reason the model stopped generating.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FinishReason {
    /// Model reached a natural stopping point.
    Stop,
    /// Hit the max_tokens limit.
    Length,
    /// Content was filtered.
    ContentFilter,
    /// Unknown or provider-specific reason.
    Other(String),
}

impl FinishReason {
    /// Parse an OpenAI-style finish reason.
    pub fn from_openai(s: &str) -> Self {
        match s {
            "stop" => FinishReason::Stop,
            "length" => FinishReason::Length,
            "content_filter" => FinishReason::ContentFilter,
            other => FinishReason::Other(other.to_string()),
        }
    }

    /// Parse an Anthropic-style stop reason.
    pub fn from_anthropic(s: &str) -> Self {
        match s {
            "end_turn" => FinishReason::Stop,
            "max_tokens" => FinishReason::Length,
            other => FinishReason::Other(other.to_string()),
        }
    }
}

/// Token usage statistics.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Tokens in the prompt/input.
    pub input_tokens: u64,
    /// Tokens in the completion/output.
    pub output_tokens: u64,
}

impl Usage {
    /// Total tokens used.
    pub fn total(&self) -> u64 {
        self.input_tokens + self.output_tokens
    }
}

/// Parsed response from a chat completion API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatResponse {
    /// The generated text content.
    pub content: String,
    /// Model that generated the response.
    pub model: String,
    /// Why the model stopped generating.
    pub finish_reason: FinishReason,
    /// Token usage statistics.
    pub usage: Usage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_constructors() {
        let sys = ChatMessage::system("You are helpful.");
        assert_eq!(sys.role, Role::System);
        assert_eq!(sys.content, "You are helpful.");

        let user = ChatMessage::user("Hello!");
        assert_eq!(user.role, Role::User);

        let asst = ChatMessage::assistant("Hi there!");
        assert_eq!(asst.role, Role::Assistant);
    }

    #[test]
    fn test_chat_request_builder() {
        let req = ChatRequest::new(
            "gpt-4o",
            vec![
                ChatMessage::system("Be concise."),
                ChatMessage::user("What is 2+2?"),
            ],
        )
        .temperature(0.7)
        .max_tokens(100)
        .stop("\n");

        assert_eq!(req.model, "gpt-4o");
        assert_eq!(req.messages.len(), 2);
        assert_eq!(req.temperature, Some(0.7));
        assert_eq!(req.max_tokens, Some(100));
        assert_eq!(req.stop, vec!["\n"]);
    }

    #[test]
    fn test_system_message_extraction() {
        let req = ChatRequest::new(
            "test",
            vec![
                ChatMessage::system("System prompt"),
                ChatMessage::user("Hello"),
                ChatMessage::assistant("Hi"),
            ],
        );

        assert_eq!(req.system_messages().len(), 1);
        assert_eq!(req.non_system_messages().len(), 2);
    }

    #[test]
    fn test_role_parse() {
        assert_eq!(Role::parse("system"), Some(Role::System));
        assert_eq!(Role::parse("USER"), Some(Role::User));
        assert_eq!(Role::parse("Assistant"), Some(Role::Assistant));
        assert_eq!(Role::parse("invalid"), None);
    }

    #[test]
    fn test_finish_reason_parsing() {
        assert_eq!(FinishReason::from_openai("stop"), FinishReason::Stop);
        assert_eq!(FinishReason::from_openai("length"), FinishReason::Length);
        assert_eq!(FinishReason::from_anthropic("end_turn"), FinishReason::Stop);
        assert_eq!(
            FinishReason::from_anthropic("max_tokens"),
            FinishReason::Length
        );
    }

    #[test]
    fn test_usage_total() {
        let usage = Usage {
            input_tokens: 10,
            output_tokens: 20,
        };
        assert_eq!(usage.total(), 30);
    }
}
