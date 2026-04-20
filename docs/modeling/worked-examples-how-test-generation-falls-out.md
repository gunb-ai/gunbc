### Worked examples: how test generation falls out

In circuit design (PSPICE), the schematic IS the test specification.
You don't write tests separately — the testbench is derived from the
circuit model. Synthesis produces the circuit. Testbenching stress-tests
it against the spec. The designer writes one artifact; two things
fall out.

The same principle applies here. A .dag file contains:

- Type constraints (what values are valid)
- Response mappings (what outcomes are possible)
- Error codes (what can go wrong)
- Rate limits (how fast you can go)
- Retry policies (how to recover)
- Mock fixtures (example valid responses)
- Dependency structure (what depends on what)

Every one of these is a testable proposition. The test generator
reads the model and derives: what inputs to try, what outputs to
expect, what error paths to exercise, what boundary conditions to
probe. No separate test specification needed.

#### Type constraints generate test values

From `dsl/std/types.dag`:

```dag
type CommitSha    = String where pattern("^[a-f0-9]{40}$")
type RetryCount   = Int where range(min: 1, max: 5)
type HttpStatus   = Int where range(min: 100, max: 599)
type PositiveInt  = Int where range(min: 1)
type NonEmptyStr  = String where non_empty
```

Each refinement predicate generates test values at the boundaries:

```
CommitSha:
  valid:    "a" * 40, "f" * 40, "0123456789abcdef" * 2 + "01234567"
  invalid:  "" (too short), "g" * 40 (wrong chars), "a" * 39 (too short),
            "a" * 41 (too long), "A" * 40 (uppercase)

RetryCount:
  valid:    1 (min), 3 (mid), 5 (max)
  invalid:  0 (below min), 6 (above max), -1 (negative)

HttpStatus:
  valid:    100 (min), 200, 301, 404, 500, 599 (max)
  invalid:  99 (below), 600 (above), 0, -1

NonEmptyStr:
  valid:    "a", "hello", " " (whitespace counts)
  invalid:  "" (empty)
```

The generation rule for each predicate type:

| Predicate | Valid values | Invalid values |
|---|---|---|
| `range(min, max)` | min, max, midpoint | min-1, max+1, 0, negative |
| `pattern(regex)` | strings matching regex | empty, wrong chars, wrong length |
| `non_empty` | "a", "test" | "" |
| `brand(name)` | valid base type values | values of different brand |

This is mechanical. The predicate IS the test spec.

#### Product types generate field-combination tests

From `dsl/std/types.dag`:

```dag
type AccessToken {
  token: Secret
  scheme: AuthScheme
  expires_at: Timestamp?
}

type AuthScheme = Bearer | Header { name: String } | Basic { username: String }
```

A product (AND) requires all fields. Test generation:

```
AccessToken tests:
  // All fields valid (happy path)
  { token: "sk-test", scheme: Bearer, expires_at: "2025-01-01T00:00:00" }

  // Each optional field absent
  { token: "sk-test", scheme: Bearer, expires_at: none }

  // Each sum-type field with each variant
  { token: "sk-test", scheme: Bearer, expires_at: ... }
  { token: "sk-test", scheme: Header { name: "X-Api-Key" }, expires_at: ... }
  { token: "sk-test", scheme: Basic { username: "admin" }, expires_at: ... }

  // Each required field invalid (expect rejection)
  { token: "", scheme: Bearer, ... }  // empty Secret
```

The rule: for a product of N fields, generate one test with all
fields valid, one test per optional field absent, one test per
sum-type variant, one test per field with an invalid value. This is
AND-decomposition — testing each conjunct independently.

For a coproduct (OR), generate one test per variant:

```
AuthScheme tests:
  Bearer                         // variant 1
  Header { name: "X-Api-Key" }  // variant 2, with valid field
  Header { name: "" }           // variant 2, with invalid field
  Basic { username: "admin" }   // variant 3, with valid field
```

This is OR-decomposition — testing each disjunct. The coproduct
tells you exactly how many cases to cover.

#### Operation contracts generate integration tests

From `dsl/extdeps/git.dag`:

```dag
operation CurrentBranch {
  input {}
  output { branch: String }
  readonly
  transport shell { argv: ["git", "rev-parse", "--abbrev-ref", "HEAD"] }
  exit {
    0 => Unit
    128 => String "Not a git repository"
  }
}
```

The contract defines all possible outcomes. Each is a test case:

```
Test 1 — happy path (exit 0):
  precondition: inside a git repository
  execute: git rev-parse --abbrev-ref HEAD
  assert: exit code = 0
  assert: output.branch is a non-empty string
  assert: output.branch matches known branch format

Test 2 — error path (exit 128):
  precondition: NOT inside a git repository (e.g., /tmp)
  execute: git rev-parse --abbrev-ref HEAD
  assert: exit code = 128
  assert: error message contains "Not a git repository"
```

One test per exit code. The exit mapping IS the test matrix.

For the readonly modifier:

```
Test 3 — readonly assertion:
  precondition: known repository state (files, branches)
  execute: CurrentBranch
  assert: repository state is unchanged after execution
  (no files modified, no refs changed, no staging area changes)
```

The modifier IS the assertion. `readonly` means "the world state
before and after must be identical." That's a testable proposition.

#### Response mappings generate API test matrices

From `dsl/extdeps/github/gists.dag`:

```dag
operation Create {
  input {
    description: String
    content: String
    public: Bool = false
    auth_token: Secret
  }
  output { id: GistId, html_url: Url }
  transport rest { method: POST, path: "/gists", body: { ... } }
  response {
    201 => Gist
    401 => GitHubErrorShape
    403 => GitHubErrorShape
  }
  mock_response {
    status: 201
    body: { id: "abc123", html_url: "https://gist.github.com/abc123" }
  }
}
```

Three response codes → three test tiers:

```
Tier 1 — Mock tests (no network, from mock_response):
  input: { description: "test", content: "hello", auth_token: "token" }
  mock: { status: 201, body: { id: "abc123", html_url: "..." } }
  assert: result matches Gist type
  assert: result.id matches GistId constraints
  assert: result.html_url matches Url constraints

Tier 2 — Contract tests (per response code):
  Test 2a (201 Created):
    input: valid description, content, auth_token
    assert: response body deserializes to Gist
    assert: Gist.id is GistId (non-empty, matches pattern)
    assert: Gist.html_url is Url (matches URL pattern)

  Test 2b (401 Unauthorized):
    input: valid description, content, INVALID auth_token
    assert: response body deserializes to GitHubErrorShape
    assert: GitHubErrorShape.message is non-empty

  Test 2c (403 Forbidden):
    input: valid everything but insufficient scopes
    assert: response body deserializes to GitHubErrorShape

Tier 3 — Real execution (with actual GitHub API):
    input: valid description, content, REAL auth_token
    assert: gist created, response matches contract
    cleanup: delete the created gist
```

The response mapping IS the test matrix. Each status code is a test
case. The mock_response IS the fixture. The output type constraints
ARE the assertions.

#### Rate limits and retry generate stress tests

From `dsl/extdeps/github/gists.dag`:

```dag
config {
  rate_limit: { requests: 5000, per: hour, scope: core }
  retry: { max_attempts: 3, backoff: exponential, retry_on: [429, 500, 502, 503, 504] }
}
```

Rate limit test:

```
Test — rate limit boundary:
  execute: N requests in rapid succession (where N approaches limit)
  assert: first N requests succeed (status != 429)
  assert: request N+1 returns 429 (Too Many Requests)
  assert: retry-after header is present
  assert: system respects retry-after before continuing

  // The rate_limit spec tells us exactly:
  //   - how many requests to try (5000)
  //   - the time window (1 hour)
  //   - what scope is limited (core)
```

Retry test:

```
Test — retry on each retryable status:
  for each status in [429, 500, 502, 503, 504]:
    simulate: server returns {status} on first call
    assert: client retries (up to 3 attempts)
    assert: backoff is exponential (delay doubles)
    simulate: server returns 201 on retry
    assert: final result is success

Test — retry exhaustion:
  simulate: server returns 500 on all 3 attempts
  assert: after 3 attempts, error is propagated (not silently swallowed)
  assert: error contains enough context to diagnose

Test — non-retryable status:
  for each status NOT in [429, 500, 502, 503, 504]:
    simulate: server returns {status}
    assert: NO retry (immediate failure)
    // e.g., 401 should fail immediately, not retry 3 times
```

The retry spec IS the test spec. `retry_on: [429, 500, 502, 503, 504]`
tells you exactly which codes to test retry behavior for AND which
codes to test immediate-failure behavior for (everything NOT in
the list).

#### Workflow structure generates integration tests

From `dsl/tools/build.dag`:

```dag
func build_all() -> { overall_success: Bool, summary: Summary, report: String } {
  build  = cargo.Build.Build()
  test   = cargo.Build.Test()    [after build, when build.success]
  clippy = cargo.Build.Clippy()  [after build, when build.success]
  ...
  return { overall_success: build.success && test.success && clippy.success, ... }
}
```

The dependency graph IS the test structure:

```
Test — all succeed:
  mock: build succeeds, test succeeds, clippy succeeds
  assert: overall_success = true
  assert: summary.passed = 3, summary.failed = 0

Test — build fails (guards block downstream):
  mock: build fails
  assert: test is NOT executed (guarded by build.success)
  assert: clippy is NOT executed (guarded by build.success)
  assert: overall_success = false
  assert: summary.failed >= 1

Test — build succeeds, test fails:
  mock: build succeeds, test fails, clippy succeeds
  assert: test executed (guard satisfied)
  assert: clippy executed (independent of test)
  assert: overall_success = false (because test.success = false)
  assert: summary.passed = 2, summary.failed = 1

Test — build succeeds, clippy fails:
  mock: build succeeds, test succeeds, clippy fails
  assert: overall_success = false
  assert: summary.passed = 2, summary.failed = 1
```

The `[after X, when Y]` annotations tell you:
- What to mock (the dependency)
- What guard to test both ways (when=true, when=false)
- What to assert about downstream effects (skipped vs executed)

For N steps with M guards, the test matrix is derived mechanically.

#### Composition: testing a service tests its types

The test tiers compose the same way the model composes:

```
Level 1 — Type tests (innermost):
  Generate valid/invalid values for each refined type.
  GistId satisfies its pattern? CommitSha is 40 hex chars?
  Boundary values for RetryCount, HttpStatus, etc.

Level 2 — Operation tests (contract level):
  For each operation: one test per response code.
  Inputs generated from Level 1 (valid GistId, valid CommitSha).
  Outputs validated against Level 1 (response body matches type).

Level 3 — Service tests (integration level):
  Mock the transport layer, replay mock_response fixtures.
  Test rate limit boundaries.
  Test retry behavior for each retryable status code.

Level 4 — Workflow tests (end-to-end):
  Mock each service call at Level 3.
  Test dependency graph: each guard true/false.
  Test parallel independence: order shouldn't matter.
  Test error propagation: failure at step N → correct skip at step N+1.

Level 5 — Real execution (system test):
  Replace mocks with real I/O.
  Verify the actual system behaves as the contract claims.
  Cleanup after mutating operations (delete created gists, etc.).
```

Each level tests the propositions at that layer. Level 1 tests
type predicates (AND constraints). Level 2 tests operation
contracts (IMPLIES + OR of outcomes). Level 3 tests service
properties (rate limits, retry). Level 4 tests workflow logic
(guarded implication chains). Level 5 validates grounding (do the
axioms actually hold in the real world?).

This is the PSPICE analogy:
- Level 1 = component characterization (does this resistor match spec?)
- Level 2 = subcircuit verification (does this amplifier stage work?)
- Level 3 = block-level simulation (does the power supply regulate?)
- Level 4 = system simulation (does the full board work?)
- Level 5 = hardware test (does the physical board match simulation?)

#### What the test generator needs from the model

No additional test specification is needed. The model already contains:

| Model element | What it generates | Test tier |
|---|---|---|
| `where pattern(...)` | valid + invalid strings | Type |
| `where range(min, max)` | boundary values: min, max, min-1, max+1 | Type |
| `where non_empty` | valid: "a", invalid: "" | Type |
| `where brand(...)` | nominal distinctness checks | Type |
| Product `{ fields }` | one test per field valid/invalid | Type |
| Coproduct `A \| B \| C` | one test per variant | Type |
| Optional `T?` | test with Some, test with None | Type |
| `response { 201 => T, 401 => E }` | one test per status code | Operation |
| `exit { 0 => T, 128 => E }` | one test per exit code | Operation |
| `mock_response { ... }` | fixture for mock-level test | Operation |
| `readonly` / `idempotent` | state-preservation assertion | Operation |
| `rate_limit: { N, per: T }` | N+1 request boundary test | Service |
| `retry: { on: [...] }` | retry/no-retry per status code | Service |
| `[after X, when P]` | guard true + guard false paths | Workflow |
| `[after X]` (no when) | ordering dependency | Workflow |

The test generator walks the DAG model, reads these elements, and
emits test code at each tier. The propositions in the model ARE the
assertions in the tests. Writing a .dag file simultaneously writes
its test specification.

