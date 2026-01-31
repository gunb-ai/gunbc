//! Content-addressable response caching for LLM providers.
//!
//! Caches LLM responses keyed by a hash of (provider, model, messages, params).
//! Each provider manages its own cache namespace so that OpenAI and Anthropic
//! caches don't collide.
//!
//! # Design
//!
//! The cache sits above the transport layer — it intercepts at the LlmOps level,
//! not the REST level. This means:
//!
//! - Cache keys are based on semantic content (messages, model), not HTTP details
//! - Provider-specific formatting differences don't affect cache hits
//! - The same logical request to different providers gets different cache entries
//!
//! # Cache Key
//!
//! The cache key is a deterministic hash of:
//! - Provider ID
//! - Model name
//! - All messages (role + content)
//! - Temperature (if set)
//! - Max tokens (if set)
//!
//! Stop sequences and other parameters are excluded from the key since they
//! typically don't change the fundamental response character.

use gunbc_ir::transport::llm::{ChatRequest, ChatResponse};
use std::collections::HashMap;

/// Cache key for an LLM request.
///
/// Deterministic string built from the semantically significant parts
/// of a chat completion request.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey(String);

impl CacheKey {
    /// Build a cache key from a provider ID and chat request.
    pub fn from_request(provider_id: &str, request: &ChatRequest) -> Self {
        let mut parts = Vec::new();

        parts.push(format!("provider:{}", provider_id));
        parts.push(format!("model:{}", request.model));

        for msg in &request.messages {
            parts.push(format!("msg:{}:{}", msg.role, msg.text()));
        }

        if let Some(t) = request.temperature {
            // Normalize to 2 decimal places to avoid float comparison issues
            parts.push(format!("temp:{:.2}", t));
        }

        if let Some(n) = request.max_tokens {
            parts.push(format!("max_tokens:{}", n));
        }

        // Simple deterministic hash: join all parts with null separator
        // then compute a portable hash. We use a simple string key for
        // now (debuggable, serializable) rather than a cryptographic hash.
        CacheKey(parts.join("\0"))
    }

    /// Get the raw key string (for debugging/logging).
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Cached LLM response entry.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// The cached response.
    pub response: ChatResponse,
    /// Number of times this entry has been hit.
    pub hit_count: u64,
}

/// Per-provider LLM response cache.
///
/// Each provider has its own cache instance, allowing independent
/// eviction policies and size limits.
#[derive(Debug)]
pub struct LlmCache {
    /// Provider ID this cache belongs to.
    provider_id: String,
    /// Cached entries keyed by request hash.
    entries: HashMap<CacheKey, CacheEntry>,
    /// Maximum number of entries (0 = unlimited).
    max_entries: usize,
}

impl LlmCache {
    /// Create a new cache for a provider.
    pub fn new(provider_id: impl Into<String>) -> Self {
        Self {
            provider_id: provider_id.into(),
            entries: HashMap::new(),
            max_entries: 1000,
        }
    }

    /// Set the maximum number of cache entries.
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Look up a cached response.
    pub fn get(&mut self, request: &ChatRequest) -> Option<&ChatResponse> {
        let key = CacheKey::from_request(&self.provider_id, request);
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.hit_count += 1;
            Some(&entry.response)
        } else {
            None
        }
    }

    /// Store a response in the cache.
    pub fn put(&mut self, request: &ChatRequest, response: ChatResponse) {
        let key = CacheKey::from_request(&self.provider_id, request);

        // Simple eviction: if at capacity, remove the least-hit entry
        if self.max_entries > 0 && self.entries.len() >= self.max_entries {
            self.evict_one();
        }

        self.entries.insert(
            key,
            CacheEntry {
                response,
                hit_count: 0,
            },
        );
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all cached entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Provider ID.
    pub fn provider_id(&self) -> &str {
        &self.provider_id
    }

    /// Evict the least-hit entry.
    fn evict_one(&mut self) {
        if let Some(key) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.hit_count)
            .map(|(k, _)| k.clone())
        {
            self.entries.remove(&key);
        }
    }
}

/// Registry of per-provider caches.
///
/// Manages one `LlmCache` per provider, lazily created on first access.
#[derive(Debug, Default)]
pub struct CacheRegistry {
    caches: HashMap<String, LlmCache>,
    /// Default max entries for new caches.
    default_max_entries: usize,
}

impl CacheRegistry {
    /// Create a new cache registry.
    pub fn new() -> Self {
        Self {
            caches: HashMap::new(),
            default_max_entries: 1000,
        }
    }

    /// Set default max entries for new caches.
    pub fn with_default_max(mut self, max: usize) -> Self {
        self.default_max_entries = max;
        self
    }

    /// Get or create the cache for a provider.
    pub fn cache_for(&mut self, provider_id: &str) -> &mut LlmCache {
        let max = self.default_max_entries;
        self.caches
            .entry(provider_id.to_string())
            .or_insert_with(|| LlmCache::new(provider_id).with_max_entries(max))
    }

    /// Look up a cached response for a provider.
    pub fn get(
        &mut self,
        provider_id: &str,
        request: &ChatRequest,
    ) -> Option<&ChatResponse> {
        // We need a two-step lookup to satisfy the borrow checker
        if !self.caches.contains_key(provider_id) {
            return None;
        }
        self.caches.get_mut(provider_id).and_then(|c| c.get(request))
    }

    /// Store a response in the provider's cache.
    pub fn put(
        &mut self,
        provider_id: &str,
        request: &ChatRequest,
        response: ChatResponse,
    ) {
        self.cache_for(provider_id).put(request, response);
    }

    /// Clear all caches.
    pub fn clear_all(&mut self) {
        self.caches.clear();
    }

    /// Clear a specific provider's cache.
    pub fn clear_provider(&mut self, provider_id: &str) {
        if let Some(cache) = self.caches.get_mut(provider_id) {
            cache.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_ir::transport::llm::{ChatMessage, FinishReason, Usage};

    fn sample_request() -> ChatRequest {
        ChatRequest::new(
            "gpt-4o",
            vec![
                ChatMessage::system("Be helpful."),
                ChatMessage::user("Hello!"),
            ],
        )
        .temperature(0.7)
    }

    fn sample_response() -> ChatResponse {
        ChatResponse {
            content: "Hi there!".to_string(),
            model: "gpt-4o".to_string(),
            finish_reason: FinishReason::Stop,
            usage: Usage {
                input_tokens: 10,
                output_tokens: 5,
                ..Default::default()
            },
            thinking: None,
            content_blocks: Vec::new(),
        }
    }

    #[test]
    fn test_cache_key_deterministic() {
        let req = sample_request();
        let k1 = CacheKey::from_request("openai", &req);
        let k2 = CacheKey::from_request("openai", &req);
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_cache_key_provider_differs() {
        let req = sample_request();
        let k1 = CacheKey::from_request("openai", &req);
        let k2 = CacheKey::from_request("anthropic", &req);
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_cache_key_message_differs() {
        let req1 = sample_request();
        let req2 = ChatRequest::new(
            "gpt-4o",
            vec![ChatMessage::user("Different message")],
        )
        .temperature(0.7);

        let k1 = CacheKey::from_request("openai", &req1);
        let k2 = CacheKey::from_request("openai", &req2);
        assert_ne!(k1, k2);
    }

    #[test]
    fn test_cache_put_and_get() {
        let mut cache = LlmCache::new("openai");
        let req = sample_request();
        let resp = sample_response();

        cache.put(&req, resp.clone());
        let cached = cache.get(&req).unwrap();
        assert_eq!(cached.content, "Hi there!");
    }

    #[test]
    fn test_cache_miss() {
        let mut cache = LlmCache::new("openai");
        let req = sample_request();
        assert!(cache.get(&req).is_none());
    }

    #[test]
    fn test_cache_hit_count() {
        let mut cache = LlmCache::new("openai");
        let req = sample_request();
        cache.put(&req, sample_response());

        cache.get(&req);
        cache.get(&req);
        cache.get(&req);

        let key = CacheKey::from_request("openai", &req);
        assert_eq!(cache.entries[&key].hit_count, 3);
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = LlmCache::new("openai").with_max_entries(2);

        let req1 = ChatRequest::new("m1", vec![ChatMessage::user("a")]);
        let req2 = ChatRequest::new("m2", vec![ChatMessage::user("b")]);
        let req3 = ChatRequest::new("m3", vec![ChatMessage::user("c")]);

        cache.put(&req1, sample_response());
        cache.put(&req2, sample_response());

        // Hit req2 so it has higher hit_count
        cache.get(&req2);

        // This should evict req1 (least hit)
        cache.put(&req3, sample_response());

        assert_eq!(cache.len(), 2);
        assert!(cache.get(&req1).is_none());
        assert!(cache.get(&req2).is_some());
    }

    #[test]
    fn test_registry_per_provider() {
        let mut registry = CacheRegistry::new();

        let req = sample_request();
        registry.put("openai", &req, sample_response());

        assert!(registry.get("openai", &req).is_some());
        assert!(registry.get("anthropic", &req).is_none());
    }

    #[test]
    fn test_registry_clear_provider() {
        let mut registry = CacheRegistry::new();

        let req = sample_request();
        registry.put("openai", &req, sample_response());
        registry.put("anthropic", &req, sample_response());

        registry.clear_provider("openai");

        assert!(registry.get("openai", &req).is_none());
        assert!(registry.get("anthropic", &req).is_some());
    }
}
