# Transport Primitives as DSL Patterns

**Status**: Proposed
**Author**: Claude
**Date**: 2026-03-01

## Problem Statement

The current transport middleware (rate limiting, retry, credential management) is implemented as hand-written Rust runtime code in `lib/transport/`. This creates several problems:

1. **Language coupling** — Emitting to Go/C/MIPS would require reimplementing all middleware in each target language
2. **Domain data in code** — Rate limits (e.g., GitHub 5000/hour) are hardcoded rather than modeled as service metadata
3. **Inconsistent syntax** — Transport concerns use Rust patterns while everything else uses DSL patterns

## Design Principle

Transport behaviors should be **fundamental DSL patterns** — syntactically and semantically consistent with other control-flow primitives like `loop`, `branch`, and `upsert`.

```
// Current: Rust runtime code interprets config
RateLimitMiddleware::new(RateLimitConfig { ... })

// Target: DSL pattern that emits to any backend
rate_limit github:core { budget: 5000 per hour }
```

## Proposed DSL Patterns

### 1. `rate_limit` Pattern

Rate limiting as a scoped block that the compiler understands.

```
// Service-level declaration (domain data)
service github.Gist {
    config {
        rate_limit core {
            budget: 5000 per hour
            burst: 100
            honor_retry_after: true
        }

        rate_limit search {
            budget: 30 per minute
        }
    }
}

// Operation-level scope binding
operation List {
    uses rate_limit: core  // binds to service's "core" rate limit
    transport rest { method: GET, path: "/gists" }
}
```

**Semantics:**
- `budget: N per {hour|minute|second}` — sustained rate
- `burst: N` — immediate burst capacity before throttling
- `honor_retry_after: bool` — respect server's Retry-After header
- Compiler emits target-language code for token bucket / sliding window

### 2. `retry` Pattern

Retry as a composable block with backoff semantics.

```
// Service-level default
service github.Gist {
    config {
        retry {
            max_attempts: 3
            backoff: exponential { base: 100ms, max: 2s, jitter: true }
            on: [429, 500, 502, 503, 504]
            on_network_error: true
        }
    }
}

// Operation-level override
operation Create {
    retry { max_attempts: 1 }  // no retry for non-idempotent
    transport rest { method: POST, path: "/gists" }
}

// Inline retry expression (like loop/branch)
result = retry { max: 3, backoff: fixed(100ms) } {
    risky_operation(args)
}
```

**Semantics:**
- `max_attempts: N` — total attempts including first
- `backoff: fixed(D) | linear(D) | exponential { base, max, jitter }`
- `on: [status_codes]` — HTTP statuses that trigger retry
- `on_network_error: bool` — retry transport-level failures
- `when: predicate` — custom retry condition

### 3. `circuit_breaker` Pattern

Circuit breaker as a named, shared resource.

```
service github.Gist {
    config {
        circuit_breaker api {
            failure_threshold: 5
            reset_timeout: 30s
            half_open_max: 1
        }
    }
}

operation Get {
    uses circuit_breaker: api
    transport rest { method: GET, path: "/gists/{id}" }
}
```

**Semantics:**
- State machine: Closed → Open → Half-Open → Closed
- Shared across operations bound to same circuit
- Compiler emits thread-safe state management

### 4. `credential` Pattern

Credential acquisition and caching as a typed resource.

```
service github.Gist {
    config {
        credential token {
            provider: oauth_bearer
            inject: header("Authorization", "Bearer {}")
            cache_key: "github-token"
            refresh_at: 80%  // proactive refresh at 80% TTL
        }
    }
}

operation Create {
    uses credential: token
    transport rest { method: POST, path: "/gists" }
}
```

**Semantics:**
- `provider` — credential source (oauth, api_key, gcp_wif)
- `inject` — how credential material enters the request
- `cache_key` — shared cache across operations
- `refresh_at` — proactive refresh threshold

## Relationship to Existing Primitives

| Pattern | Similar to | Key difference |
|---------|-----------|----------------|
| `rate_limit` | `uses` resource | Scoped budget, time-based |
| `retry` | `loop` | Conditional repetition with backoff |
| `circuit_breaker` | `uses` resource | Shared failure state |
| `credential` | `uses` resource | TTL-aware caching |

## Compilation Strategy

### Phase 1: Parse & Typecheck
- New AST nodes for transport patterns
- Type rules for budget expressions (`5000 per hour`)
- Scope binding validation

### Phase 2: Lower to IR
- Transport patterns become IR nodes
- Rate limit budgets become `TransportMiddlewareConfig`
- Retry policies become `RetryConfig`

### Phase 3: Emit
Each target backend emits appropriate code:

**Rust emit:**
```rust
// Generated from: rate_limit core { budget: 5000 per hour }
let rate_limiter = TokenBucket::new(
    tokens_per_second: 5000.0 / 3600.0,
    burst: 100,
);
```

**Go emit:**
```go
// Generated from: rate_limit core { budget: 5000 per hour }
rateLimiter := ratelimit.NewTokenBucket(
    rate.Limit(5000.0 / 3600.0),
    100, // burst
)
```

## Migration Path

### Phase 1: DSL Syntax (no emit changes)
1. Add `rate_limit`, `retry`, `circuit_breaker`, `credential` to grammar
2. Parse and typecheck transport blocks
3. Lower to existing `TransportMiddlewareConfig` IR
4. Rust runtime continues to interpret

### Phase 2: Domain Data Migration
1. Move rate limit budgets from Rust to `dsl/services/*.dag`
2. Service definitions become source of truth
3. Rust middleware reads from compiled IR

### Phase 3: Emit Migration
1. Emit transport logic per target language
2. Rust middleware becomes optional/fallback
3. Generated code is self-contained

### Phase 4: Runtime Elimination
1. Remove `lib/transport/` middleware
2. All transport logic is generated
3. Only thin executor layer remains

## Files Affected

**Add:**
- `core/daglang/daglang-syntax/src/ast/transport.rs` — AST nodes
- `core/daglang/daglang-typecheck/src/transport.rs` — type rules
- `core/daglang/daglang-lower/src/transport.rs` — lowering
- `core/daglang/daglang-emit/src/transport_*.rs` — per-backend emit

**Modify:**
- `dsl/services/*.dag` — add transport blocks
- `core/ir/src/transport/middleware.rs` — IR types (already exists)

**Eventually Remove:**
- `lib/transport/src/rate_limit.rs`
- `lib/transport/src/retry.rs`
- `lib/transport/src/credential.rs`
- `lib/transport/src/pipeline.rs`

## Open Questions

1. **Shared state across processes?** Rate limiters and circuit breakers need shared state. In-process is easy; distributed requires external store.

2. **Observability hooks?** Metrics middleware needs emit hooks. Should the DSL have `on_request`, `on_response`, `on_error` blocks?

3. **Testing primitives?** How do generated rate limiters interact with MockSpec and test fixtures?

## References

- `docs/handbook.md` § Compositional Modeling
- `docs/design/modeling/annotation-to-dag-modeling.md` — annotation migration
- `SPEC.md` § Transport Model
