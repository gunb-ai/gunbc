//! Rate limit middleware.
//!
//! Provides rate limiting using token bucket or sliding window algorithms.
//! Rate limits are tracked per scope (e.g., "github:core", "github:search")
//! to respect provider-specific quotas.
//!
//! # Configuration
//!
//! Rate limiting is configured via `RateLimitConfig` from the IR:
//!
//! ```ignore
//! RateLimitConfig {
//!     scope_key: "github:core",
//!     algorithm: RateLimitAlgorithm::TokenBucket,
//!     max_burst: 20,
//!     requests: 5000,
//!     window_seconds: 3600,  // per hour
//!     honor_retry_after: true,
//! }
//! ```

use crate::classify::{classify_for_middleware, ClassifiedErrorKind};
use crate::middleware::{
    MiddlewareContext, MiddlewareOutcome, PostProcessOutcome, TransportMiddleware,
};
use gunbc_ir::transport::{
    RateLimitAlgorithm, RateLimitConfig, TransportRequest, TransportResponse,
};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Token bucket rate limiter state.
#[derive(Debug)]
struct TokenBucket {
    /// Current number of available tokens.
    tokens: f64,
    /// Last time tokens were refilled.
    last_refill: Instant,
    /// Maximum tokens (burst capacity).
    max_tokens: f64,
    /// Tokens added per second.
    refill_rate: f64,
    /// Explicit pause until a specific time (for Retry-After handling).
    /// If set and in the future, all requests are blocked.
    pause_until: Option<Instant>,
}

impl TokenBucket {
    fn new(max_burst: u32, requests: u32, window_seconds: u32) -> Self {
        // Convert raw (requests, window) to tokens/second without integer truncation.
        let safe_rate = if requests == 0 || window_seconds == 0 {
            1.0 / 3600.0 // 1 per hour as minimum
        } else {
            requests as f64 / window_seconds as f64
        };

        // max_burst of 0 means no requests allowed - use at least 1
        let safe_max = if max_burst == 0 {
            1.0
        } else {
            max_burst as f64
        };

        Self {
            tokens: safe_max,
            last_refill: Instant::now(),
            max_tokens: safe_max,
            refill_rate: safe_rate,
            pause_until: None,
        }
    }

    /// Try to acquire a token. Returns wait time if rate limited.
    fn try_acquire(&mut self) -> Result<(), Duration> {
        let now = Instant::now();

        // Check explicit pause first (from Retry-After)
        if let Some(until) = self.pause_until {
            if now < until {
                return Err(until.duration_since(now));
            }
            // Pause expired, clear it
            self.pause_until = None;
        }

        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            Ok(())
        } else {
            // Calculate time to wait for next token
            let tokens_needed = 1.0 - self.tokens;
            let wait_secs = tokens_needed / self.refill_rate;
            Err(Duration::from_secs_f64(wait_secs))
        }
    }

    /// Refill tokens based on elapsed time.
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        let new_tokens = elapsed.as_secs_f64() * self.refill_rate;
        self.tokens = (self.tokens + new_tokens).min(self.max_tokens);
        self.last_refill = now;
    }

    /// Current headroom as fraction (0.0 = exhausted, 1.0 = full).
    fn headroom(&self) -> f64 {
        // If paused, headroom is 0
        if let Some(until) = self.pause_until {
            if Instant::now() < until {
                return 0.0;
            }
        }
        self.tokens / self.max_tokens
    }

    /// Force a wait until a specific time (for Retry-After handling).
    fn force_wait_until(&mut self, until: Instant) {
        let now = Instant::now();
        if until > now {
            // Set explicit pause - this is the only safe way to enforce wait
            self.pause_until = Some(until);
            // Also drain tokens to prevent any requests after pause expires
            self.tokens = 0.0;
        }
    }
}

/// Sliding window rate limiter state.
#[derive(Debug)]
struct SlidingWindow {
    /// Request timestamps within the current window.
    requests: Vec<Instant>,
    /// Window duration.
    window: Duration,
    /// Maximum requests per window.
    max_requests: u32,
    /// Explicit pause until a specific time (for Retry-After handling).
    pause_until: Option<Instant>,
}

impl SlidingWindow {
    fn new(requests: u32, window_seconds: u32) -> Self {
        let window_secs = if window_seconds == 0 {
            60
        } else {
            window_seconds
        };
        Self {
            requests: Vec::new(),
            window: Duration::from_secs(window_secs as u64),
            max_requests: requests,
            pause_until: None,
        }
    }

    /// Try to record a request. Returns wait time if rate limited.
    fn try_acquire(&mut self) -> Result<(), Duration> {
        let now = Instant::now();

        // Check explicit pause first (from Retry-After)
        if let Some(until) = self.pause_until {
            if now < until {
                return Err(until.duration_since(now));
            }
            // Pause expired, clear it
            self.pause_until = None;
        }

        self.prune(now);

        if (self.requests.len() as u32) < self.max_requests {
            self.requests.push(now);
            Ok(())
        } else {
            // Calculate time until oldest request falls out of window
            if let Some(oldest) = self.requests.first() {
                // Use checked_duration_since to avoid panics on future timestamps
                if let Some(oldest_age) = now.checked_duration_since(*oldest) {
                    if oldest_age < self.window {
                        let wait = self.window - oldest_age;
                        return Err(wait);
                    }
                } else {
                    // oldest is in the future (shouldn't happen, but be safe)
                    return Err(Duration::from_secs(1));
                }
            }
            // Should not happen if prune worked correctly
            Err(Duration::from_secs(1))
        }
    }

    /// Remove requests older than the window.
    fn prune(&mut self, now: Instant) {
        // Use saturating_sub to avoid underflow when now < window
        let cutoff = now.checked_sub(self.window).unwrap_or(now);
        self.requests.retain(|t| *t > cutoff);
    }

    /// Current headroom as fraction.
    fn headroom(&self) -> f64 {
        // If paused, headroom is 0
        if let Some(until) = self.pause_until {
            if Instant::now() < until {
                return 0.0;
            }
        }
        let used = self.requests.len() as f64;
        let max = self.max_requests as f64;
        (max - used) / max
    }

    /// Force a wait until a specific time (for Retry-After handling).
    fn force_wait_until(&mut self, until: Instant) {
        let now = Instant::now();
        if until > now {
            // Use explicit pause instead of manipulating timestamps
            // This avoids underflow panics when retry-after > window
            self.pause_until = Some(until);
            // Clear requests to be safe
            self.requests.clear();
        }
    }
}

/// Rate limiter that supports both algorithms.
#[derive(Debug)]
enum RateLimiter {
    TokenBucket(TokenBucket),
    SlidingWindow(SlidingWindow),
}

impl RateLimiter {
    fn new(config: &RateLimitConfig) -> Self {
        match config.algorithm {
            RateLimitAlgorithm::TokenBucket => RateLimiter::TokenBucket(TokenBucket::new(
                config.max_burst,
                config.requests,
                config.window_seconds,
            )),
            RateLimitAlgorithm::SlidingWindow => RateLimiter::SlidingWindow(SlidingWindow::new(
                config.requests,
                config.window_seconds,
            )),
        }
    }

    fn try_acquire(&mut self) -> Result<(), Duration> {
        match self {
            RateLimiter::TokenBucket(tb) => tb.try_acquire(),
            RateLimiter::SlidingWindow(sw) => sw.try_acquire(),
        }
    }

    fn headroom(&self) -> f64 {
        match self {
            RateLimiter::TokenBucket(tb) => tb.headroom(),
            RateLimiter::SlidingWindow(sw) => sw.headroom(),
        }
    }

    fn force_wait_until(&mut self, until: Instant) {
        match self {
            RateLimiter::TokenBucket(tb) => tb.force_wait_until(until),
            RateLimiter::SlidingWindow(sw) => sw.force_wait_until(until),
        }
    }
}

/// Shared rate limit state across requests.
#[derive(Debug, Default)]
pub struct RateLimitState {
    /// Rate limiters keyed by scope.
    limiters: Mutex<HashMap<String, RateLimiter>>,
}

impl RateLimitState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create a rate limiter for a scope.
    fn get_or_create(&self, scope: &str, config: &RateLimitConfig) -> Result<(), Duration> {
        let mut limiters = self.limiters.lock().unwrap();
        let limiter = limiters
            .entry(scope.to_string())
            .or_insert_with(|| RateLimiter::new(config));
        limiter.try_acquire()
    }

    /// Get headroom for a scope.
    fn headroom(&self, scope: &str) -> Option<f64> {
        let limiters = self.limiters.lock().unwrap();
        limiters.get(scope).map(|l| l.headroom())
    }

    /// Apply Retry-After from a 429 response.
    fn apply_retry_after(&self, scope: &str, retry_after_ms: u64) {
        let mut limiters = self.limiters.lock().unwrap();
        if let Some(limiter) = limiters.get_mut(scope) {
            let until = Instant::now() + Duration::from_millis(retry_after_ms);
            limiter.force_wait_until(until);
        }
    }
}

/// Rate limit middleware.
pub struct RateLimitMiddleware {
    config: RateLimitConfig,
    state: RateLimitState,
}

impl RateLimitMiddleware {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            state: RateLimitState::new(),
        }
    }

    /// Create with shared state (for use in pipeline).
    pub fn with_shared_state(config: RateLimitConfig, state: RateLimitState) -> Self {
        Self { config, state }
    }
}

impl TransportMiddleware for RateLimitMiddleware {
    fn pre_request(
        &self,
        request: TransportRequest,
        _ctx: &mut MiddlewareContext,
    ) -> MiddlewareOutcome {
        // Maximum retry attempts for rate limit acquisition.
        // Prevents infinite loops while handling thundering herd scenarios where
        // multiple threads wake up simultaneously after sleeping.
        const MAX_ACQUIRE_ATTEMPTS: u32 = 10;
        let mut attempts = 0;

        loop {
            attempts += 1;

            match self
                .state
                .get_or_create(&self.config.scope_key, &self.config)
            {
                Ok(()) => {
                    // Request allowed, record headroom for metrics
                    if let Some(headroom) = self.state.headroom(&self.config.scope_key) {
                        // Metrics sink would be called here via ctx.shared_state
                        let _ = headroom;
                    }
                    return MiddlewareOutcome::Continue(request);
                }
                Err(wait_duration) => {
                    if attempts >= MAX_ACQUIRE_ATTEMPTS {
                        // Too many failed attempts - abort rather than loop forever
                        return MiddlewareOutcome::Abort(gunbc_exec::ExecError::new(format!(
                            "Rate limit exhausted for scope '{}' after {} attempts",
                            self.config.scope_key, attempts
                        )));
                    }

                    // Rate limited - wait synchronously then retry
                    // In a real async implementation, this would be async sleep
                    std::thread::sleep(wait_duration);
                    // Loop continues to retry acquisition
                }
            }
        }
    }

    fn post_response(
        &self,
        _request: &TransportRequest,
        response: TransportResponse,
        ctx: &mut MiddlewareContext,
    ) -> PostProcessOutcome {
        // Check for 429 and apply Retry-After
        if self.config.honor_retry_after {
            if let Some(classified) =
                classify_for_middleware(&response, ctx.config.response_classification.as_ref())
            {
                if classified.kind == ClassifiedErrorKind::RateLimit {
                    if let Some(retry_after_ms) = classified.retry_after_ms {
                        self.state
                            .apply_retry_after(&self.config.scope_key, retry_after_ms);
                    }
                }
            }
        }

        PostProcessOutcome::Complete(response)
    }

    fn name(&self) -> &'static str {
        "rate_limit"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::SharedMiddlewareState;
    use gunbc_ir::transport::{LocalRequest, TransportMiddlewareConfig};
    use std::sync::Arc;

    fn test_config(max_burst: u32, per_minute: u32) -> RateLimitConfig {
        RateLimitConfig {
            scope_key: "test".to_string(),
            algorithm: RateLimitAlgorithm::TokenBucket,
            max_burst,
            requests: per_minute,
            window_seconds: 60,
            honor_retry_after: true,
        }
    }

    #[test]
    fn token_bucket_allows_burst() {
        let mut bucket = TokenBucket::new(5, 60, 60); // 60 req/min

        // Should allow 5 requests immediately (burst)
        for _ in 0..5 {
            assert!(bucket.try_acquire().is_ok());
        }

        // 6th request should be rate limited
        assert!(bucket.try_acquire().is_err());
    }

    #[test]
    fn token_bucket_refills_over_time() {
        let mut bucket = TokenBucket::new(2, 120, 60); // 2 tokens/sec (120/min)

        // Use all tokens
        assert!(bucket.try_acquire().is_ok());
        assert!(bucket.try_acquire().is_ok());
        assert!(bucket.try_acquire().is_err());

        // Wait for refill
        std::thread::sleep(Duration::from_millis(600));

        // Should have ~1 token now
        assert!(bucket.try_acquire().is_ok());
    }

    #[test]
    fn sliding_window_limits_requests() {
        let mut window = SlidingWindow::new(3, 60); // 3 per minute

        assert!(window.try_acquire().is_ok());
        assert!(window.try_acquire().is_ok());
        assert!(window.try_acquire().is_ok());
        assert!(window.try_acquire().is_err());
    }

    #[test]
    fn rate_limit_state_tracks_multiple_scopes() {
        let state = RateLimitState::new();

        let config1 = RateLimitConfig {
            scope_key: "scope1".to_string(),
            algorithm: RateLimitAlgorithm::TokenBucket,
            max_burst: 2,
            requests: 60,
            window_seconds: 60,
            honor_retry_after: false,
        };

        let config2 = RateLimitConfig {
            scope_key: "scope2".to_string(),
            algorithm: RateLimitAlgorithm::TokenBucket,
            max_burst: 3,
            requests: 60,
            window_seconds: 60,
            honor_retry_after: false,
        };

        // Each scope has its own bucket
        assert!(state.get_or_create("scope1", &config1).is_ok());
        assert!(state.get_or_create("scope1", &config1).is_ok());
        assert!(state.get_or_create("scope1", &config1).is_err()); // scope1 exhausted

        // scope2 still has capacity
        assert!(state.get_or_create("scope2", &config2).is_ok());
        assert!(state.get_or_create("scope2", &config2).is_ok());
        assert!(state.get_or_create("scope2", &config2).is_ok());
        assert!(state.get_or_create("scope2", &config2).is_err()); // scope2 exhausted
    }

    #[test]
    fn middleware_allows_request_within_limit() {
        let config = test_config(10, 600);
        let mw = RateLimitMiddleware::new(config);

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
    fn headroom_decreases_with_requests() {
        let state = RateLimitState::new();
        let config = RateLimitConfig {
            scope_key: "test".to_string(),
            algorithm: RateLimitAlgorithm::TokenBucket,
            max_burst: 4,
            requests: 60,
            window_seconds: 60,
            honor_retry_after: false,
        };

        assert!(state.get_or_create("test", &config).is_ok());
        let h1 = state.headroom("test").unwrap();

        assert!(state.get_or_create("test", &config).is_ok());
        let h2 = state.headroom("test").unwrap();

        assert!(h2 < h1, "headroom should decrease: {} < {}", h2, h1);
    }

    #[test]
    fn apply_retry_after_blocks_requests() {
        let state = RateLimitState::new();
        let config = RateLimitConfig {
            scope_key: "test".to_string(),
            algorithm: RateLimitAlgorithm::TokenBucket,
            max_burst: 10,
            requests: 60,
            window_seconds: 60,
            honor_retry_after: true,
        };

        // Prime the limiter
        assert!(state.get_or_create("test", &config).is_ok());

        // Apply a retry-after
        state.apply_retry_after("test", 100); // 100ms

        // Should be rate limited now
        let result = state.get_or_create("test", &config);
        assert!(result.is_err());
    }
}
