# Transport Primitives as DSL Patterns

**Status**: Proposed
**Lane**: 7 (Transport Domain Modeling)
**Author**: Claude
**Date**: 2026-03-01

## Executive Summary

Transport behaviors (rate limiting, retry, credentials, error parsing) are currently split between:
- **Domain data in Rust** (anti-pattern) — e.g., GitHub's 5000/hour limit hardcoded
- **OS mechanisms in Rust** (correct) — e.g., `TokenBucket`, mutexes, sockets

This design moves **domain data** into `.dag` while keeping **OS mechanisms** in per-language Target SDKs.

---

## The Core Distinction: "What" vs "How"

### Domain Policy (What) → belongs in `.dag`

Questions like:
- "What are GitHub's rate limits?" (5000/hour core, 30/minute search)
- "What status codes trigger retry?" (429, 500, 502, 503, 504)
- "What shape does GCP return errors in?" (`{ "error": { "code": N, "message": "..." } }`)
- "How do we authenticate to this provider?" (Bearer token in `Authorization` header)

These are **external facts about services** — they belong in `.dag` files alongside service definitions.

### OS Mechanisms (How) → belongs in Target SDK

Questions like:
- "How do I pause a thread for 100ms?" (`std::thread::sleep`)
- "How do I atomically decrement a counter?" (`AtomicU64`)
- "How do I share state across threads?" (`Arc<Mutex<T>>`)
- "How do I open a TCP socket?" (`TcpStream::connect`)

These are **operating system primitives** — they belong in a Target SDK per language.

### Why This Matters

If we tried to express Token Bucket math in `.dag`:
```
// DON'T DO THIS — turns .dag into a systems language
fn acquire_token() {
    let now = Instant::now()                    // Clock access
    let elapsed = now - self.last_refill        // Duration math
    let tokens = elapsed.as_secs_f64() * rate   // Float arithmetic
    self.available.fetch_add(tokens, Ordering::Release)  // Atomic CAS
    ...
}
```

We'd have to add mutexes, clocks, atomics, and duration math to `.dag`. That accidentally turns it into C++, defeating its purpose as a high-level orchestration language.

---

## The Architecture: Protobuf/gRPC Pattern

This follows the same pattern as Protocol Buffers + gRPC:

```
.proto (schema)         →  Generated code    →  Language runtime
────────────────            ───────────────      ────────────────
message User { ... }       user.pb.go           grpc-go library
service UserService { }    user_grpc.pb.go      (handles sockets, HTTP/2)
```

For gunbc:

```
.dag (domain policy)    →  Generated config  →  Target SDK
────────────────────        ────────────────     ──────────────
rate_limit core {          RateLimitMiddleware  gunbc-rust-transport
  budget: 5000 per hour     ::new(5000, 3600)   (TokenBucket, Mutex)
}
```

**The compiler generates configuration code** that links to a Target SDK.
It does NOT generate the Token Bucket algorithm line-by-line.

If we want Python tomorrow, we:
1. Write `gunbc-python-transport` once (using `asyncio`)
2. Compiler emits: `RateLimitMiddleware(budget=5000, window_secs=3600)`
3. The Python SDK handles all the async/await mechanics

---

## What Moves to `.dag`

### 1. Rate Limit Budgets

**Current (anti-pattern):** Hardcoded in Rust
```rust
// lib/transport/src/rate_limit.rs
const GITHUB_CORE_LIMIT: u64 = 5000;
const GITHUB_SEARCH_LIMIT: u64 = 30;
```

**Target:** Declared in service definition
```
service github.Gist {
    config {
        rate_limit core { budget: 5000 per hour, burst: 100 }
        rate_limit search { budget: 30 per minute }
    }
}
```

### 2. Retry Policies

**Current:** Hardcoded retry logic
```rust
const RETRYABLE_STATUSES: &[u16] = &[429, 500, 502, 503, 504];
```

**Target:** Declared per-service or per-operation
```
service github.Gist {
    config {
        retry {
            max_attempts: 3
            backoff: exponential { base: 100ms, max: 2s, jitter: true }
            on: [429, 500, 502, 503, 504]
        }
    }
}

operation Create {
    retry { max_attempts: 1 }  // override: no retry for non-idempotent
}
```

### 3. Provider Error Shapes

**Current (major anti-pattern):** Rust code knows GitHub's error format
```rust
// lib/transport/src/classify.rs
if host.contains("github.com") {
    // Parse { "message": ..., "documentation_url": ... }
}
```

**Target:** Error shape in service definition
```
service github.Gist {
    config {
        error_shape {
            message: ".message"
            code: ".status"
            docs: ".documentation_url"
        }
    }
}
```

### 4. Credential Injection

**Current:** Auth scheme hardcoded per provider
```rust
fn inject_auth(provider: &str, token: &str) -> Header {
    match provider {
        "github" => ("Authorization", format!("Bearer {}", token)),
        "gcp" => ("Authorization", format!("Bearer {}", token)),
        ...
    }
}
```

**Target:** Credential block in service config
```
service github.Gist {
    config {
        credential token {
            provider: BearerToken
            inject: header("Authorization", "Bearer {token}")
        }
    }
}
```

---

## What Stays in Target SDK

The following remain in `lib/transport/` (Rust) or equivalent per-language SDK:

| Component | Why it stays |
|-----------|--------------|
| `TokenBucket` struct | Atomic counters, clock access, thread sleep |
| `RetryExecutor` | Loop mechanics, backoff calculation, jitter RNG |
| `CircuitBreaker` | State machine, mutex-protected transitions |
| `CredentialCache` | TTL tracking, thread-safe refresh |
| `HttpClient` | TCP sockets, TLS, HTTP/2 framing |

The Target SDK is **domain-agnostic**. It doesn't know "GitHub" or "GCP" — it only knows:
- "I was configured with budget=5000, window=3600"
- "I should retry on status codes [429, 500, 502, 503, 504]"
- "I should inject this header with this format"

---

## DSL Syntax

### `rate_limit` Block

```
rate_limit <name> {
    budget: <N> per <hour|minute|second>
    burst: <N>?                           // default: 10% of budget
    honor_retry_after: <bool>?            // default: true
}
```

### `retry` Block

```
retry {
    max_attempts: <N>
    backoff: fixed(<duration>)
           | linear { base: <duration>, step: <duration> }
           | exponential { base: <duration>, max: <duration>, jitter: <bool> }
    on: [<status_codes>]
    on_network_error: <bool>?             // default: true
}
```

### `error_shape` Block

```
error_shape {
    message: <json_path>
    code: <json_path>?
    details: <json_path>?
}
```

### `credential` Block

```
credential <name> {
    provider: BearerToken | ApiKey | GcpWif | AwsSigV4
    inject: header(<name>, <format>)
          | query(<name>)
    cache_key: <string>?
    refresh_at: <percentage>?             // default: 80%
}
```

---

## Compilation Flow

### Phase 1: Parse & Typecheck

```
rate_limit core { budget: 5000 per hour }
     ↓ parse
RateLimitDecl { name: "core", budget: 5000, window: Hour, ... }
     ↓ typecheck
✓ budget > 0, window is valid unit, name unique within service
```

### Phase 2: Lower to IR

```
RateLimitDecl
     ↓ lower
TransportMiddlewareConfig::RateLimit {
    id: "github.Gist.core",
    tokens_per_second: 5000.0 / 3600.0,
    burst: 100,
}
```

### Phase 3: Emit (per target)

**Rust emit:**
```rust
// Generated: links to gunbc-rust-transport SDK
let rate_limiter = RateLimitMiddleware::new(RateLimitConfig {
    id: "github.Gist.core",
    tokens_per_second: 1.389,
    burst: 100,
});
```

**Go emit (future):**
```go
// Generated: links to gunbc-go-transport SDK
rateLimiter := transport.NewRateLimiter(transport.RateLimitConfig{
    ID: "github.Gist.core",
    TokensPerSecond: 1.389,
    Burst: 100,
})
```

---

## Migration Path

### Phase 1: DSL Syntax + IR (no runtime changes)

1. Add grammar rules for `rate_limit`, `retry`, `error_shape`, `credential`
2. Parse into AST nodes
3. Typecheck scopes and references
4. Lower to existing `TransportMiddlewareConfig` IR
5. **Rust runtime continues to interpret** — no behavioral changes

**Deliverable:** `.dag` files can express transport policy; compiler validates and lowers.

### Phase 2: Domain Data Migration

1. Move hardcoded values from Rust to `.dag` service definitions
2. Delete `GITHUB_CORE_LIMIT` constants
3. Delete provider-specific `if host.contains("github.com")` branches
4. Target SDK reads all config from compiled IR

**Deliverable:** Rust transport code is provider-agnostic.

### Phase 3: Multi-Target Emit

1. Emit transport configuration per target language
2. Define `gunbc-go-transport` SDK interface
3. Compiler emits Go code that configures the Go SDK

**Deliverable:** Same `.dag` file produces working Rust and Go code.

### Phase 4: Substrate Cleanup

1. `lib/transport/` becomes a pure Target SDK (no domain knowledge)
2. Delete `classify.rs` provider-specific branches
3. All domain facts live in `.dag`

**Deliverable:** Complete separation of concerns.

---

## Design Decisions (Pre-Resolved)

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Rate limit state scope | Per-process | Distributed state requires external store; defer to later |
| Retry jitter | Full jitter (0 to backoff) | AWS recommendation; prevents thundering herd |
| Circuit breaker sharing | Per-service, not per-operation | Most APIs share rate limits across endpoints |
| Credential refresh | Proactive at 80% TTL | Avoids request-time refresh latency |
| Error shape extraction | JSON path syntax | Familiar, handles nested structures |

## Open Questions (To Resolve Before Implementation)

1. **Shared state across replicas?** For distributed deployments, rate limiters need coordination. Options: Redis, distributed token bucket, or "best effort" per-replica. *Decision needed before Phase 3.*

2. **Observability hooks?** Should `.dag` have `on_request`, `on_response`, `on_retry` blocks for metrics/logging? Or is this Target SDK concern? *Decision needed before Phase 2.*

3. **Test fixtures?** How do `rate_limit` and `retry` interact with `mock` blocks? Should tests auto-skip rate limiting? *Decision needed before Phase 1.*

---

## References

- `docs/handbook.md` § Compositional Modeling Philosophy
- `docs/design/modeling/annotation-to-dag-modeling.md` — annotation migration tracking
- `SPEC.md` § Transport Model — IR representation
- External: [AWS Exponential Backoff](https://docs.aws.amazon.com/general/latest/gr/api-retries.html)
