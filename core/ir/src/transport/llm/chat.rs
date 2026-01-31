//! Chat completion types for LLM providers.
//!
//! These types provide a unified interface for chat-based LLM APIs
//! (OpenAI Chat Completions, OpenAI Responses, Anthropic Messages).
//!
//! Provider-specific serialization is handled by conversion functions
//! in the provider modules (openai.rs, anthropic.rs, openai_responses.rs).
//!
//! # Content Model
//!
//! Messages support both simple text and structured content blocks:
//!
//! ```ignore
//! // Simple text (most common)
//! ChatMessage::user("Hello!")
//!
//! // Structured blocks with cache hints
//! ChatMessage::user_blocks(vec![
//!     ContentBlock::text("Long system context...").with_cache(CacheControl::ephemeral()),
//!     ContentBlock::text("Actual question"),
//! ])
//! ```
//!
//! Provider modules translate cache hints to provider-specific formats:
//! - **Anthropic**: `cache_control: {"type": "ephemeral"}` on content blocks
//! - **OpenAI**: Automatic prefix caching (no API changes; hints ignored)
//!
//! # Thinking / Reasoning
//!
//! Extended thinking is configured per-request via `ThinkingConfig`:
//!
//! ```ignore
//! let req = ChatRequest::new("claude-sonnet-4-5", messages)
//!     .thinking(ThinkingConfig::anthropic(10000));
//!
//! let req = ChatRequest::new("o3", messages)
//!     .thinking(ThinkingConfig::openai(ReasoningEffort::High));
//! ```
//!
//! # API References
//!
//! - Anthropic Messages: <https://docs.anthropic.com/en/api/messages>
//! - Anthropic Prompt Caching: <https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching>
//! - Anthropic Extended Thinking: <https://docs.anthropic.com/en/docs/build-with-claude/extended-thinking>
//! - OpenAI Chat Completions: <https://platform.openai.com/docs/api-reference/chat>
//! - OpenAI Responses: <https://platform.openai.com/docs/api-reference/responses>
//! - OpenAI Prompt Caching: <https://platform.openai.com/docs/guides/prompt-caching>
//! - OpenAI Reasoning: <https://platform.openai.com/docs/guides/reasoning>

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Content blocks and cache control
// ---------------------------------------------------------------------------

/// Cache hint for a content block.
///
/// Providers translate this to their native format:
/// - **Anthropic**: `cache_control: {"type": "ephemeral"}` on the content block.
///   Caches everything from the start of the request up to this breakpoint.
///   5-minute TTL by default; min 1024 tokens (Sonnet/Opus 4), 2048 (Haiku 3).
///   Cache reads cost 10% of base input price; writes cost 125%.
///   Ref: <https://docs.anthropic.com/en/docs/build-with-claude/prompt-caching>
///
/// - **OpenAI**: Automatic prefix caching for identical prefixes >= 1024 tokens.
///   No API changes needed; place static content first for best cache hit rate.
///   Cached tokens cost 50% of input price. 5-10min TTL (up to 1hr off-peak).
///   Ref: <https://platform.openai.com/docs/guides/prompt-caching>
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheControl {
    /// The caching strategy.
    pub cache_type: CacheType,
}

impl CacheControl {
    /// Mark content as cacheable with the default ephemeral (5-minute) TTL.
    pub fn ephemeral() -> Self {
        Self {
            cache_type: CacheType::Ephemeral,
        }
    }
}

/// Cache strategy type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheType {
    /// Cache with a short TTL (5 minutes on Anthropic; automatic on OpenAI).
    Ephemeral,
}

/// A structured content block within a message.
///
/// Most messages use simple text, but content blocks enable cache hints
/// and multi-part content (e.g., a cached system prompt followed by
/// variable instructions).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContentBlock {
    /// Plain text content with an optional cache hint.
    Text {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    /// Thinking block from an extended-thinking response (Anthropic).
    /// Must be preserved verbatim when passing back in multi-turn tool-use.
    Thinking {
        thinking: String,
        signature: String,
    },
    /// Redacted thinking block (Anthropic safety filter).
    /// Must be preserved verbatim for reasoning continuity.
    RedactedThinking {
        data: String,
    },
}

impl ContentBlock {
    /// Create a plain text block.
    pub fn text(text: impl Into<String>) -> Self {
        ContentBlock::Text {
            text: text.into(),
            cache_control: None,
        }
    }

    /// Attach a cache hint to this block.
    pub fn with_cache(self, cc: CacheControl) -> Self {
        match self {
            ContentBlock::Text { text, .. } => ContentBlock::Text {
                text,
                cache_control: Some(cc),
            },
            other => other, // cache hints only apply to text blocks
        }
    }

    /// Get the text content, if this is a text block.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ContentBlock::Text { text, .. } => Some(text),
            _ => None,
        }
    }
}

/// Message content — either simple text or structured content blocks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content (the common case).
    Text(String),
    /// Structured content blocks with optional cache hints.
    Blocks(Vec<ContentBlock>),
}

impl MessageContent {
    /// Get the text content. For blocks, concatenates all text blocks.
    pub fn text(&self) -> String {
        match self {
            MessageContent::Text(s) => s.clone(),
            MessageContent::Blocks(blocks) => blocks
                .iter()
                .filter_map(|b| b.as_text())
                .collect::<Vec<_>>()
                .join(""),
        }
    }

    /// Whether this is simple text content.
    pub fn is_text(&self) -> bool {
        matches!(self, MessageContent::Text(_))
    }

    /// Whether this uses structured content blocks.
    pub fn is_blocks(&self) -> bool {
        matches!(self, MessageContent::Blocks(_))
    }

    /// Get the content blocks, if structured.
    pub fn blocks(&self) -> Option<&[ContentBlock]> {
        match self {
            MessageContent::Blocks(blocks) => Some(blocks),
            _ => None,
        }
    }
}

impl From<String> for MessageContent {
    fn from(s: String) -> Self {
        MessageContent::Text(s)
    }
}

impl From<&str> for MessageContent {
    fn from(s: &str) -> Self {
        MessageContent::Text(s.to_string())
    }
}

// ---------------------------------------------------------------------------
// Thinking / Reasoning configuration
// ---------------------------------------------------------------------------

/// Extended thinking / reasoning configuration.
///
/// Each provider implements thinking differently:
///
/// - **Anthropic** (`thinking` request param): Enables step-by-step reasoning
///   with a token budget. Returns `thinking` content blocks in the response.
///   Budget must be < `max_tokens`. Incompatible with `temperature` / `top_k`.
///   Supported on: Claude Sonnet 3.7+, Haiku 4.5, Opus 4+.
///   Ref: <https://docs.anthropic.com/en/docs/build-with-claude/extended-thinking>
///
/// - **OpenAI** (`reasoning` request param): Controls reasoning depth via an
///   effort level. Reasoning tokens are hidden; only summaries are visible.
///   Uses `max_completion_tokens` (Chat Completions) or `max_output_tokens`
///   (Responses API) to cap total output. Supported on: o1, o3, o4-mini.
///   Ref: <https://platform.openai.com/docs/guides/reasoning>
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ThinkingConfig {
    /// Anthropic extended thinking.
    Anthropic {
        /// Maximum tokens for internal reasoning. Must be < max_tokens.
        budget_tokens: u64,
    },
    /// OpenAI reasoning.
    OpenAI {
        /// How much effort to spend on reasoning (low/medium/high).
        effort: ReasoningEffort,
        /// Whether to include a reasoning summary in the response.
        #[serde(skip_serializing_if = "Option::is_none")]
        summary: Option<ReasoningSummary>,
    },
}

impl ThinkingConfig {
    /// Anthropic extended thinking with the given token budget.
    pub fn anthropic(budget_tokens: u64) -> Self {
        ThinkingConfig::Anthropic { budget_tokens }
    }

    /// OpenAI reasoning with the given effort level.
    pub fn openai(effort: ReasoningEffort) -> Self {
        ThinkingConfig::OpenAI {
            effort,
            summary: None,
        }
    }

    /// OpenAI reasoning with effort and summary.
    pub fn openai_with_summary(effort: ReasoningEffort, summary: ReasoningSummary) -> Self {
        ThinkingConfig::OpenAI {
            effort,
            summary: Some(summary),
        }
    }
}

/// Reasoning effort level for OpenAI models (o1, o3, o4-mini).
///
/// Controls the trade-off between reasoning depth and speed/cost.
/// Default is `Medium`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}

impl ReasoningEffort {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReasoningEffort::Low => "low",
            ReasoningEffort::Medium => "medium",
            ReasoningEffort::High => "high",
        }
    }
}

/// Reasoning summary verbosity for OpenAI models.
///
/// Controls how much of the model's reasoning is returned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningSummary {
    /// Brief summary of the reasoning process.
    Concise,
    /// More complete summary of the reasoning process.
    Detailed,
}

impl ReasoningSummary {
    pub fn as_str(&self) -> &'static str {
        match self {
            ReasoningSummary::Concise => "concise",
            ReasoningSummary::Detailed => "detailed",
        }
    }
}

// ---------------------------------------------------------------------------
// Roles and messages
// ---------------------------------------------------------------------------

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
///
/// Content can be simple text or structured content blocks with cache hints:
///
/// ```ignore
/// ChatMessage::user("Simple text")
/// ChatMessage::user_blocks(vec![
///     ContentBlock::text("Cached context").with_cache(CacheControl::ephemeral()),
///     ContentBlock::text("Variable question"),
/// ])
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The role of the message author.
    pub role: Role,
    /// The content of the message.
    pub content: MessageContent,
}

impl ChatMessage {
    /// Create a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: MessageContent::Text(content.into()),
        }
    }

    /// Create a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: MessageContent::Text(content.into()),
        }
    }

    /// Create an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: MessageContent::Text(content.into()),
        }
    }

    /// Create a system message with structured content blocks.
    pub fn system_blocks(blocks: Vec<ContentBlock>) -> Self {
        Self {
            role: Role::System,
            content: MessageContent::Blocks(blocks),
        }
    }

    /// Create a user message with structured content blocks.
    pub fn user_blocks(blocks: Vec<ContentBlock>) -> Self {
        Self {
            role: Role::User,
            content: MessageContent::Blocks(blocks),
        }
    }

    /// Create an assistant message with structured content blocks.
    /// Used to pass back thinking + tool_use blocks in multi-turn conversations.
    pub fn assistant_blocks(blocks: Vec<ContentBlock>) -> Self {
        Self {
            role: Role::Assistant,
            content: MessageContent::Blocks(blocks),
        }
    }

    /// Get the text content of this message.
    pub fn text(&self) -> String {
        self.content.text()
    }
}

// ---------------------------------------------------------------------------
// Request
// ---------------------------------------------------------------------------

/// Parameters for a chat completion request.
///
/// Works with all supported APIs:
/// - OpenAI Chat Completions (`/v1/chat/completions`)
/// - OpenAI Responses (`/v1/responses`)
/// - Anthropic Messages (`/v1/messages`)
///
/// Provider modules translate to the appropriate wire format.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatRequest {
    /// Model identifier (e.g., "gpt-4o", "o3", "claude-sonnet-4-5").
    pub model: String,
    /// Conversation messages.
    pub messages: Vec<ChatMessage>,
    /// Sampling temperature (0.0 - 2.0). Lower = more deterministic.
    /// Incompatible with Anthropic extended thinking.
    pub temperature: Option<f64>,
    /// Maximum tokens to generate (includes thinking budget for Anthropic).
    pub max_tokens: Option<u64>,
    /// Stop sequences.
    #[serde(default)]
    pub stop: Vec<String>,
    /// Extended thinking / reasoning configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
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
            thinking: None,
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

    /// Enable extended thinking / reasoning.
    pub fn thinking(mut self, config: ThinkingConfig) -> Self {
        self.thinking = Some(config);
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

// ---------------------------------------------------------------------------
// Response
// ---------------------------------------------------------------------------

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

/// A block of content in a response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ResponseBlock {
    /// Generated text.
    Text { text: String },
    /// Thinking / reasoning output (Anthropic extended thinking, or OpenAI
    /// reasoning summary).
    Thinking {
        thinking: String,
        /// Encrypted signature for Anthropic (must be preserved for multi-turn).
        /// `None` for OpenAI reasoning summaries.
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
    /// Redacted thinking (Anthropic safety filter). Must be preserved verbatim.
    RedactedThinking { data: String },
}

/// Token usage statistics.
///
/// Extends basic input/output counts with provider-specific cache and
/// reasoning token tracking.
///
/// # Anthropic Cache Tokens
///
/// When prompt caching is active, Anthropic reports:
/// - `cache_creation_input_tokens`: tokens written to cache (billed at 125%)
/// - `cache_read_input_tokens`: tokens read from cache (billed at 10%)
///
/// These are separate from `input_tokens` (non-cached input).
///
/// # OpenAI Cache and Reasoning Tokens
///
/// OpenAI reports `cached_tokens` inside `prompt_tokens_details` (50% discount)
/// and `reasoning_tokens` inside `completion_tokens_details`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Tokens in the prompt/input.
    pub input_tokens: u64,
    /// Tokens in the completion/output.
    pub output_tokens: u64,
    /// Anthropic: tokens written to the prompt cache on this request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
    /// Anthropic: tokens read from the prompt cache on this request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
    /// OpenAI: tokens served from the automatic prompt cache.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u64>,
    /// Tokens used for reasoning/thinking (both providers).
    /// Anthropic: thinking output tokens. OpenAI: reasoning_tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u64>,
}

impl Usage {
    /// Total tokens used (input + output).
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
    /// Token usage statistics (includes cache and reasoning token details).
    pub usage: Usage,
    /// Thinking / reasoning text, if extended thinking was enabled.
    /// For Anthropic: the summarized (or full, for 3.7) thinking output.
    /// For OpenAI: the reasoning summary (if requested).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    /// Full structured response blocks (text, thinking, redacted_thinking).
    /// Empty if the response only contained plain text.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_blocks: Vec<ResponseBlock>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_message_constructors() {
        let sys = ChatMessage::system("You are helpful.");
        assert_eq!(sys.role, Role::System);
        assert_eq!(sys.text(), "You are helpful.");

        let user = ChatMessage::user("Hello!");
        assert_eq!(user.role, Role::User);

        let asst = ChatMessage::assistant("Hi there!");
        assert_eq!(asst.role, Role::Assistant);
    }

    #[test]
    fn test_chat_message_blocks() {
        let msg = ChatMessage::system_blocks(vec![
            ContentBlock::text("Cached system prompt").with_cache(CacheControl::ephemeral()),
            ContentBlock::text("Variable instructions"),
        ]);

        assert_eq!(msg.role, Role::System);
        assert!(msg.content.is_blocks());
        assert_eq!(msg.text(), "Cached system promptVariable instructions");

        let blocks = msg.content.blocks().unwrap();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(&blocks[0], ContentBlock::Text { cache_control: Some(_), .. }));
        assert!(matches!(&blocks[1], ContentBlock::Text { cache_control: None, .. }));
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
        assert!(req.thinking.is_none());
    }

    #[test]
    fn test_chat_request_thinking() {
        let req = ChatRequest::new(
            "claude-sonnet-4-5",
            vec![ChatMessage::user("Think about this.")],
        )
        .max_tokens(16000)
        .thinking(ThinkingConfig::anthropic(10000));

        assert!(matches!(
            req.thinking,
            Some(ThinkingConfig::Anthropic { budget_tokens: 10000 })
        ));

        let req = ChatRequest::new("o3", vec![ChatMessage::user("Reason about this.")])
            .thinking(ThinkingConfig::openai_with_summary(
                ReasoningEffort::High,
                ReasoningSummary::Concise,
            ));

        assert!(matches!(
            req.thinking,
            Some(ThinkingConfig::OpenAI {
                effort: ReasoningEffort::High,
                summary: Some(ReasoningSummary::Concise),
            })
        ));
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
            ..Default::default()
        };
        assert_eq!(usage.total(), 30);
    }

    #[test]
    fn test_usage_cache_fields() {
        let usage = Usage {
            input_tokens: 17,
            output_tokens: 700,
            cache_creation_input_tokens: Some(1370),
            cache_read_input_tokens: Some(0),
            ..Default::default()
        };
        assert_eq!(usage.cache_creation_input_tokens, Some(1370));
    }

    #[test]
    fn test_content_block_text() {
        let block = ContentBlock::text("Hello");
        assert_eq!(block.as_text(), Some("Hello"));

        let cached = ContentBlock::text("System").with_cache(CacheControl::ephemeral());
        assert!(matches!(cached, ContentBlock::Text { cache_control: Some(_), .. }));
    }

    #[test]
    fn test_message_content_from_str() {
        let mc: MessageContent = "hello".into();
        assert!(mc.is_text());
        assert_eq!(mc.text(), "hello");
    }
}
