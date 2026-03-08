//! Credential middleware with TTL-aware caching and proactive refresh.
//!
//! Provides credential management for the transport pipeline:
//! - Caches credentials by key to avoid repeated acquisition
//! - Tracks TTL and proactively refreshes at configurable threshold (default 80%)
//! - Thread-safe credential store for concurrent access
//!
//! # Configuration
//!
//! ```ignore
//! CredentialConfig {
//!     provider: CredentialProvider::OAuthBearer,
//!     injection: CredentialInjection::AuthorizationBearer,
//!     cache_key: Some("github-token".to_string()),
//!     cache_ttl_ms: None,  // Use credential's natural TTL
//!     refresh_threshold_pct: 80,  // Proactive refresh at 80% of TTL
//! }
//! ```

use crate::middleware::{MiddlewareContext, MiddlewareOutcome, TransportMiddleware};
use gunbc_exec::ExecError;
use gunbc_ir::transport::{CredentialConfig, TransportRequest};
use gunbc_ir::{AuthScheme, Credential, Secret};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};

/// Cached credential entry with timing metadata.
#[derive(Debug)]
struct CachedCredential {
    credential: Credential,
    fetched_at: Instant,
    /// Absolute expiry time (from credential's TTL or config override).
    expires_at: Option<SystemTime>,
    /// Total TTL duration computed at fetch time.
    /// Used for proactive refresh threshold calculation.
    total_ttl: Option<Duration>,
}

impl CachedCredential {
    fn new(credential: Credential, ttl_override: Option<u64>) -> Self {
        let fetched_at = Instant::now();
        let now = SystemTime::now();

        let (expires_at, total_ttl) = if let Some(ttl_ms) = ttl_override {
            let ttl = Duration::from_millis(ttl_ms);
            (Some(now + ttl), Some(ttl))
        } else if let Some(expiry) = credential.secret().expires_at() {
            // Compute TTL from expiry - now
            let ttl = expiry.duration_since(now).ok();
            (Some(expiry), ttl)
        } else {
            (None, None)
        };

        Self {
            credential,
            fetched_at,
            expires_at,
            total_ttl,
        }
    }

    /// Whether this credential is still valid (not expired).
    fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(expiry) => SystemTime::now() < expiry,
            None => true,
        }
    }

    /// Whether this credential should be proactively refreshed.
    fn should_refresh(&self, threshold_pct: u8) -> bool {
        let Some(total_ttl) = self.total_ttl else {
            return false; // No expiry = never refresh
        };

        // threshold_duration is when we should start refreshing
        // e.g., 80% threshold on 1 hour TTL = start refreshing after 48 minutes
        let threshold_duration = total_ttl * threshold_pct as u32 / 100;
        let elapsed = self.fetched_at.elapsed();

        elapsed > threshold_duration
    }
}

/// Thread-safe credential cache.
#[derive(Debug, Default)]
pub struct CredentialCache {
    entries: Mutex<HashMap<String, CachedCredential>>,
}

impl CredentialCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a cached credential if valid.
    pub fn get(&self, key: &str) -> Option<Credential> {
        let entries = self.entries.lock().unwrap();
        entries
            .get(key)
            .filter(|c| c.is_valid())
            .map(|c| c.credential.clone())
    }

    /// Check if credential should be refreshed.
    pub fn should_refresh(&self, key: &str, threshold_pct: u8) -> bool {
        let entries = self.entries.lock().unwrap();
        entries
            .get(key)
            .map(|c| c.should_refresh(threshold_pct))
            .unwrap_or(true) // Not cached = needs refresh
    }

    /// Store a credential with optional TTL override.
    pub fn put(&self, key: String, credential: Credential, ttl_override: Option<u64>) {
        let mut entries = self.entries.lock().unwrap();
        entries.insert(key, CachedCredential::new(credential, ttl_override));
    }

    /// Remove a cached credential.
    pub fn remove(&self, key: &str) {
        let mut entries = self.entries.lock().unwrap();
        entries.remove(key);
    }

    /// Clear all cached credentials.
    pub fn clear(&self) {
        let mut entries = self.entries.lock().unwrap();
        entries.clear();
    }

    /// Number of cached entries.
    pub fn len(&self) -> usize {
        self.entries.lock().unwrap().len()
    }

    /// Whether cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Credential provider function type.
///
/// Takes a config and returns a credential (or error).
pub type CredentialProviderFn =
    Box<dyn Fn(&CredentialConfig) -> Result<Credential, ExecError> + Send + Sync>;

/// Credential middleware.
pub struct CredentialMiddleware {
    config: CredentialConfig,
    cache: CredentialCache,
    provider: Option<CredentialProviderFn>,
}

impl CredentialMiddleware {
    /// Create middleware with configuration.
    ///
    /// Without a provider function, the middleware will only work with
    /// credentials supplied externally (via context or request auth field).
    pub fn new(config: CredentialConfig) -> Self {
        Self {
            config,
            cache: CredentialCache::new(),
            provider: None,
        }
    }

    /// Create middleware with a credential provider function.
    pub fn with_provider(config: CredentialConfig, provider: CredentialProviderFn) -> Self {
        Self {
            config,
            cache: CredentialCache::new(),
            provider: Some(provider),
        }
    }

    /// Get or acquire a credential.
    fn get_credential(&self) -> Result<Credential, ExecError> {
        let cache_key =
            self.config.cache_key.clone().unwrap_or_else(|| {
                format!("{:?}:{:?}", self.config.provider, self.config.injection)
            });

        // Check cache
        let cached = self.cache.get(&cache_key);
        let needs_refresh = self
            .cache
            .should_refresh(&cache_key, self.config.refresh_threshold_pct);

        // If we have a valid cached credential and don't need refresh, use it
        if let Some(ref cred) = cached {
            if !needs_refresh {
                return Ok(cred.clone());
            }
        }

        // Try to acquire new credential (either proactive refresh or initial fetch)
        if let Some(provider) = &self.provider {
            match provider(&self.config) {
                Ok(credential) => {
                    self.cache
                        .put(cache_key, credential.clone(), self.config.cache_ttl_ms);
                    return Ok(credential);
                }
                Err(e) => {
                    // If refresh failed but we have a valid cached credential, use it
                    if let Some(cred) = cached {
                        return Ok(cred);
                    }
                    // No cached credential and refresh failed - propagate error
                    return Err(e);
                }
            }
        }

        // No provider - can only use cached
        if let Some(cred) = cached {
            Ok(cred)
        } else {
            Err(ExecError::new(
                "No credential provider configured and no cached credential available",
            ))
        }
    }
}

impl TransportMiddleware for CredentialMiddleware {
    fn pre_request(
        &self,
        mut request: TransportRequest,
        _ctx: &mut MiddlewareContext,
    ) -> MiddlewareOutcome {
        // Only apply to REST requests
        if let TransportRequest::Rest(ref mut rest) = request {
            // Check if request already has auth
            if rest.auth.is_some() {
                // Auth already set, apply it
                if let Some(cred) = rest.auth.take() {
                    cred.apply(rest);
                }
                return MiddlewareOutcome::Continue(request);
            }

            // Check if request requires auth
            if !rest.requires_auth {
                return MiddlewareOutcome::Continue(request);
            }

            // Get credential and apply
            match self.get_credential() {
                Ok(cred) => {
                    cred.apply(rest);
                    MiddlewareOutcome::Continue(request)
                }
                Err(e) => MiddlewareOutcome::Abort(e),
            }
        } else {
            // Non-REST requests pass through
            MiddlewareOutcome::Continue(request)
        }
    }

    fn name(&self) -> &'static str {
        "credential"
    }
}

/// Create a static credential provider for testing.
pub fn static_credential_provider(token: &str) -> CredentialProviderFn {
    let token = token.to_string();
    Box::new(move |_config| {
        Ok(Credential::new(
            Secret::static_value(token.clone()),
            AuthScheme::Bearer,
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::SharedMiddlewareState;
    use gunbc_ir::transport::{
        CredentialInjection, CredentialProvider, LocalRequest, RestRequest,
        TransportMiddlewareConfig,
    };
    use std::sync::Arc;

    fn test_config() -> CredentialConfig {
        CredentialConfig {
            provider: CredentialProvider::OAuthBearer,
            injection: CredentialInjection::AuthorizationBearer,
            cache_key: Some("test".to_string()),
            cache_ttl_ms: None,
            refresh_threshold_pct: 80,
        }
    }

    #[test]
    fn cache_stores_and_retrieves() {
        let cache = CredentialCache::new();
        let cred = Credential::new(Secret::static_value("token123"), AuthScheme::Bearer);

        cache.put("key1".to_string(), cred.clone(), None);

        let retrieved = cache.get("key1");
        assert!(retrieved.is_some());
    }

    #[test]
    fn cache_returns_none_for_missing_key() {
        let cache = CredentialCache::new();
        assert!(cache.get("nonexistent").is_none());
    }

    #[test]
    fn cache_removes_entry() {
        let cache = CredentialCache::new();
        let cred = Credential::new(Secret::static_value("token"), AuthScheme::Bearer);

        cache.put("key".to_string(), cred, None);
        assert!(!cache.is_empty());

        cache.remove("key");
        assert!(cache.is_empty());
    }

    #[test]
    fn middleware_applies_credential_to_rest_request() {
        let config = test_config();
        let mw =
            CredentialMiddleware::with_provider(config, static_credential_provider("test-token"));

        let mw_config = Arc::new(TransportMiddlewareConfig::default());
        let shared = Arc::new(SharedMiddlewareState::new());
        let mut ctx = MiddlewareContext::new("test.op", false, true, mw_config, shared);

        let mut request = RestRequest::get("https://api.example.com");
        request.requires_auth = true;
        let request = TransportRequest::Rest(request);

        let outcome = mw.pre_request(request, &mut ctx);

        match outcome {
            MiddlewareOutcome::Continue(TransportRequest::Rest(r)) => {
                assert!(r.headers.contains_key("Authorization"));
                assert!(r.headers["Authorization"].starts_with("Bearer "));
            }
            _ => panic!("Expected Continue with Rest request"),
        }
    }

    #[test]
    fn middleware_skips_non_rest_requests() {
        let config = test_config();
        let mw = CredentialMiddleware::new(config);

        let mw_config = Arc::new(TransportMiddlewareConfig::default());
        let shared = Arc::new(SharedMiddlewareState::new());
        let mut ctx = MiddlewareContext::new("test.op", false, true, mw_config, shared);

        let request = TransportRequest::Local(LocalRequest {
            inputs: serde_json::json!({}),
        });

        let outcome = mw.pre_request(request, &mut ctx);
        assert!(matches!(outcome, MiddlewareOutcome::Continue(_)));
    }

    #[test]
    fn middleware_skips_requests_not_requiring_auth() {
        let config = test_config();
        let mw = CredentialMiddleware::new(config);

        let mw_config = Arc::new(TransportMiddlewareConfig::default());
        let shared = Arc::new(SharedMiddlewareState::new());
        let mut ctx = MiddlewareContext::new("test.op", false, true, mw_config, shared);

        let mut request = RestRequest::get("https://api.example.com");
        request.requires_auth = false;
        let request = TransportRequest::Rest(request);

        let outcome = mw.pre_request(request, &mut ctx);

        match outcome {
            MiddlewareOutcome::Continue(TransportRequest::Rest(r)) => {
                assert!(!r.headers.contains_key("Authorization"));
            }
            _ => panic!("Expected Continue with Rest request"),
        }
    }

    #[test]
    fn middleware_caches_credentials() {
        let config = test_config();
        let call_count = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let provider: CredentialProviderFn = Box::new(move |_| {
            call_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(Credential::new(
                Secret::static_value("token"),
                AuthScheme::Bearer,
            ))
        });

        let mw = CredentialMiddleware::with_provider(config, provider);

        // First call should invoke provider
        let _ = mw.get_credential();
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);

        // Second call should use cache
        let _ = mw.get_credential();
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[test]
    fn middleware_fails_without_provider_and_cache() {
        let config = test_config();
        let mw = CredentialMiddleware::new(config);

        let result = mw.get_credential();
        assert!(result.is_err());
    }
}
