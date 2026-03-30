# gunbc Testing Strategy

This document describes the testing and verification strategy. See
[ROADMAP.md](../ROADMAP.md) M3 for current status and work items.

## Guarantee receipt

The compiler emits a JSON receipt on every run — the single authority
for what the compilation proved, tested, and left uncertain. Markdown
dashboards are generated FROM the receipt; they are never the source
of truth. If a guarantee is not in the receipt, it does not exist.

```json
{
  "source_digest": "...",
  "compiler_digest": "...",
  "target": "rust",
  "discovered": {
    "dag_files": 87, "services": 42,
    "workflows": 9, "pure_functions": 149
  },
  "structural": {
    "decidability": "proven",
    "name_opacity": "ratcheting:70",
    "parse_item_keyword_arms": 0
  },
  "gated": {
    "all_dsl_files_parse": "pass",
    "full_dsl_compiles": "fail:1",
    "generated_rust_tests": "pass",
    "edge_contract_coverage": { "covered": 812, "uncovered": 4 }
  },
  "report_only": {
    "ownership_coverage": "61/149",
    "emitted_rust_errors": 880
  }
}
```

## Compositional mock generation

Mock data is compositional, like everything else. Sample values live
ON the type definition and propagate through composition. The compiler
generates realistic mocks by composing child samples. Hand-written
`mock_response` blocks are only needed for cross-field scenarios.

### Level 1 — Structural witnesses (compiler-generated, automatic)

Every type gets a canonical witness from its structure. No hand-written
data. Tests plumbing: serialization, field access, pattern matching.

| Type pattern | Canonical witness |
|---|---|
| Product (all edges) | All fields present with child witnesses |
| Coproduct (one edge) | First variant + each variant separately |
| Optional | BOTH: present(witness) AND absent |
| Refinement (`where`) | Value satisfying the constraint |
| Collection | Empty + one-element with child witness |
| Leaf (Int, String, Bool) | Zero / empty / false |

### Cardinality coverage falls out of structure

Cardinality = edge exists or doesn't. This is the substrate binary.
The compiler enumerates all cardinality combinations for any type
structurally — no hand-writing.

**At the type level:** A type with N optional fields has 2^N
cardinality combinations. The compiler generates witnesses for all
of them. For `AccessToken { token: Secret, scheme: AuthScheme,
expires_at: Timestamp? }` — that's 2 witnesses (expires_at present,
expires_at absent). For a type with 3 optional fields → 8 witnesses.

**At the function level:** A function taking `T?` is tested with
BOTH `Some(witness(T))` AND `None`. A function returning `T?` has
its output checked: did it produce `Some` when expected? `None`
when expected? Both paths covered.

**At the service level:** An optional response field means the mock
contract must cover both present and absent. The compiler checks:
does the `mock_response` include a case where this field is null/
absent? If not → `under_specified` → compile error.

**Cross-field cardinality:** If field A being present implies field B
is absent (mutual exclusion), that's a structural constraint. The
witness generator enumerates only VALID combinations — not all 2^N,
just the ones that satisfy the declared constraints.

This means cardinality testing is never a separate concern. It
falls out of the structural witness generation at every level:
types, functions, services, and cross-field constraints. The
compiler sees the edges and enumerates the possibilities.

### Level 2 — Compositional samples (type-authored, domain-specific)

Types carry sample values as part of their definition. These propagate
upward — any type using `Url` automatically gets realistic URLs.

```dag
// Sample data lives on the type, not in a separate mock file
type Url = String where pattern("https?://.*") {
  samples: [
    "https://example.com",
    "https://api.github.com/repos/owner/repo"
  ]
}

type GitHubLogin = String where non_empty {
  samples: ["octocat", "defunkt"]
}

// PullRequest mock is COMPOSED from field type samples:
//   number <- Int.samples -> [0, 1, -1]
//   title <- String.samples -> ["", "hello"]
//   html_url <- Url.samples -> ["https://example.com"]
//   user.login <- GitHubLogin.samples -> ["octocat"]
// No hand-writing { number: 42, title: "Add widget support", ... }
type PullRequest {
  number: Int
  title: String
  html_url: Url
  user: GitHubUser
}
```

### Level 3 — Scenario mocks (hand-authored, cross-field)

Only needed when fields have cross-cutting constraints the type
structure alone can't express: "when state is 'open', merged_at
should be null." These are the existing `mock_response` blocks —
they supplement compositional mocks, not replace them.

**Why this avoids duplicate representation:** sample data lives on
the type definition (one place), not copied into mock_response blocks
(many places). Adding a sample to `Url` improves mocks everywhere
`Url` is used. The mock_response blocks in existing .dag files can
shrink to only the cross-field scenarios.

## Output verification: the type IS the oracle

Generating inputs is solved (compositional mocks from type samples).
The question is: how do we know what output to expect? The answer:
**the output type's constraints are the oracle.** Five levels of
verification, all but the last derive from structure:

| Level | What it checks | Oracle | Hand-written? |
|---|---|---|---|
| **Type correctness** | Output has the right shape | Compiler (static) | No |
| **Constraint satisfaction** | Output satisfies `where` predicates | Run output through type's predicates | No |
| **Algebraic laws** | Operations satisfy declared laws | Laws on algebra types | No |
| **Cross-target agreement** | Same input → same output across targets | Differential comparison | No |
| **Known values** | Specific input → specific output | Isomorphism derivation or hand-authored | Sometimes |

**The algebra declares the laws. The compiler checks them.**

```dag
type FreeMonoid<T> {
  // Laws become generated property tests:
  law associative: concat(concat(a, b), c) == concat(a, concat(b, c))
  law identity: concat(a, empty) == a
  law filter_preserves: filter(xs, p) |> all(p) == true
  law map_length: map(xs, f) |> count == xs |> count
}
```

The compiler reads these laws and generates property tests. Input:
compositional mock. Expected output: satisfies the laws. No
hand-written expected values needed.

**Worked example: `filter(xs, x > 3)` with input `[1, 2, 3, 4, 5]`:**

- Type correctness: output is `List<Int>` (static)
- Constraint: every element satisfies `x > 3` (run the predicate
  on each output element — the predicate IS the oracle)
- Algebraic: `output.count <= input.count` (filter never grows)
- Cross-target: Rust, Python, Go produce `[4, 5]` (differential)
- Known value: `[4, 5]` — but levels 1-3 already prove correctness
  without knowing this specific value

**Worked example: `to_fahrenheit(Celsius(0.0))`:**

- Type correctness: output is `Fahrenheit` (static)
- Constraint: output satisfies `Float where range(...)` (predicate)
- Algebraic: `to_celsius(to_fahrenheit(0.0)) == 0.0` (round-trip
  law — both are isomorphisms through Kelvin)
- Cross-target: all targets produce `32.0` (differential)
- Known value: `32.0` — derivable from the isomorphism chain:
  `0 + 273.15 = 273.15K`, `273.15 * 9/5 - 459.67 = 32.0`

**When do you need hand-written expected values?**

Only when the function has behavior that isn't captured by type
constraints or algebraic laws. For most well-modeled functions, the
type structure provides enough oracle information that specific
expected values are redundant — the laws prove correctness
generically. Hand-written golden values are useful as documentation
and regression anchors, but they're not the primary correctness
mechanism.

## Testing is compilation

Structural and algebraic tests run DURING compilation. If they fail,
the code doesn't emit. This is the same philosophy as decidability:
if the compiler can't prove termination, it doesn't compile. If the
compiler can't verify correctness, it doesn't compile.

| What runs at compile time | What happens on failure |
|---|---|
| Type correctness (static) | Compile error (already the case) |
| Structural witnesses (construct/roundtrip) | Compile error |
| Constraint satisfaction (output through predicates) | Compile error |
| Algebraic law checks (with samples) | Compile error |
| Sample coverage (enough samples declared?) | Compile error if below threshold |

The compiler runs all unit/property tests before emitting output.
The developer gets immediate feedback: "your type has no samples"
or "your function violates its algebra's round-trip law" — as compile
errors, not as a test report they might not read.

### Construct statuses

The compiler assigns a status to every construct:

| Status | Meaning | Blocks compile? |
|---|---|---|
| `proven` | Compiler proved structurally (type, decidability, ownership) | No — nothing to test |
| `tested` | Hermetic test ran and passed | No |
| `under_specified` | Missing samples, mocks, witnesses, or behavioral oracle | **Yes** |
| `invalid` | Hermetic test failed or law violated | **Yes** |
| `integration_pending` | Integration contract exists, not yet verified live | No |
| `integration_failed` | Live integration test failed (CI lane only) | No — does not block ordinary compile |

`under_specified` and `invalid` block compilation. The compiler
cannot form a trustworthy test surface for these constructs.
`integration_pending` does NOT block compilation — a network outage
should not prevent `gunbc compile`.

### Pipeline order

```
1. parse / resolve / infer
2. collect proof + test obligations
3. fail immediately on invalid / under_specified
4. run hermetic checks (DryRun + Selective Real)
5. emit target artifacts to temp location
6. run generated target tests (cargo check, syntax, etc.)
7. publish final artifacts only if all hermetic verification passes
```

Artifacts are emitted LAST, not first. A construct that can't prove
its correctness never reaches the output directory.

### Phased compile-error policy

Not all test tracks are implemented. The compile-error policy
phases in as each track lands:

| Phase | What becomes a compile error | When |
|---|---|---|
| **Now** | Missing `mock_response` for service claiming hermetic verification | M2 |
| **Phase 1** | Failed DryRun / service mock tests | M3 (track implemented) |
| **Phase 2** | Missing samples for types requiring behavioral testing | M3 (samples track) |
| **Phase 3** | Failed type roundtrip tests | M3 (roundtrip track) |
| **Phase 4** | Failed algebraic law checks, missing oracle | M3 (law track) |
| **Phase 5** | Failed workflow dry-run coverage | M3 (workflow track) |

Each track promotes from "not checked" to "compile error" as it
becomes implemented. The receipt records which tracks are active.

### What blocks compile vs what doesn't

```
BLOCKS COMPILE (hermetic):
  Missing mock for a declared hermetic boundary
  Missing samples/witness for a construct requiring tests
  Missing behavioral oracle for non-structurally-provable construct
  Failed DryRun / Selective Real test
  Stale generated artifacts (freshness violation)

DOES NOT BLOCK COMPILE (integration):
  GitHub API is down
  Cloud service returns 500
  Network timeout
  Live integration test failure
  (These fail in the CI integration lane, not in gunbc compile)

BLOCKS COMPILE (integration contract):
  Missing integration profile for an external boundary
  Missing DryRun mock for a transport
  Missing response type contract for a service
  (The CONTRACT must exist even if the live test hasn't run)
```

## Integration testing: generated, not blocking

Integration tests (real HTTP, real databases, real file I/O) CANNOT
block compilation — they need real credentials, external services,
and network access. But the compiler can still own them:

```
                    compile time                    post-compile
                    ------------                    ------------
Tier 1 (DryRun)    runs during compilation          --
                    All transports mocked.
                    Proves: wiring, types, coercion,
                    structural properties.
                    Failure = compile error.

Tier 2 (Selective) runs during compilation          --
                    Hermetic effects only (temp
                    dirs, env vars, timestamps).
                    Failure = compile error.

Tier 3 (Full Real) NOT run during compilation       generated as output
                    Requires real credentials,       artifact. Runs in CI,
                    live services, network.           staging, or manually.
                    Compiler GENERATES the test.      Receipt marks: "generated
                    Compiler VERIFIES the mock         -not-run"
                    contract matches the service
                    type signature.
```

**What the compiler proves about integration at compile time:**
- The DryRun mock matches the service's type signature (structural)
- The mock_response values satisfy the response type constraints
- The wiring between service operations is type-correct
- The workflow completes structurally with mocked transports

**What the compiler generates for post-compile verification:**
- Integration test artifacts per service operation
- Each test: create real client, call real endpoint, assert response
  satisfies the same type constraints the DryRun checked structurally
- The test is a `.dag`-generated artifact, not hand-written

**The guarantee receipt records both categories:**

```json
{
  "compile_time_proven": {
    "weather.convert.to_fahrenheit": "all levels pass",
    "github.pulls.List": "DryRun pass, types match, mock contract valid"
  },
  "generated_not_run": {
    "github.pulls.List.integration": "test artifact at tests/integration/github_pulls.rs",
    "github.pulls.Create.integration": "test artifact at tests/integration/github_pulls.rs"
  }
}
```

The developer sees: structural correctness is proven (compile error
if broken). Integration correctness has a generated test (run it in
CI with real credentials). The receipt tells you exactly what's
proven vs what still needs external verification.

**For transports specifically:**
- The transport TYPE is modeled in `.dag` (endpoint, auth, rate
  limits, retry policy, response types)
- The `mock_response` block provides DryRun behavior
- At compile time: DryRun proves the wiring (right types flow through
  right operations with right auth)
- The compiler generates the real integration test as an output
  artifact — same assertions, real HTTP instead of mock
- CI runs the generated integration test with scoped credentials

## No tautological tests

Every node in the graph carries a **proof status**: what's already
been proven about it by the compiler or by a higher-level test.
The test generator only produces tests for undischarged obligations —
never for things already proven.

```
Node: to_fahrenheit(c: Celsius) -> Fahrenheit

Already proven (skip these):
  Type correctness      -- compiler proved statically
  Decidability (O(1))   -- complexity analyzer proved
  Ownership (SoleOwner) -- ownership analyzer proved
  Return type shape     -- inference proved

Undischarged (generate tests for these):
  ? Constraint satisfaction -- need to run output through Fahrenheit predicates
  ? Round-trip law          -- need to check to_celsius(to_fahrenheit(x)) == x
  ? Cross-target agreement  -- need to run on multiple targets

NOT generated (tautological):
  "to_fahrenheit returns Fahrenheit" -- already proven by type checker
  "to_fahrenheit terminates"         -- already proven by decidability
  "c is consumed once"               -- already proven by ownership
```

The rule: **a test is justified only at a boundary the compiler
doesn't control.** Inside the compiler's proof envelope, tests are
tautological. At integration boundaries (real HTTP, real filesystem,
real external service), tests are essential because the compiler
proved the mock contract but not the real service's behavior.

This means:
- Structural properties -> never tested (compiler proves them)
- Algebraic laws -> tested only with sample VALUES (the law itself
  is structural; the test checks that the implementation satisfies
  it with concrete inputs)
- Integration boundaries -> always tested (compiler can't prove
  external behavior)
- The receipt marks each obligation: `proven` (no test needed),
  `tested` (test ran and passed), `generated` (test exists, not yet
  run)

## Test generation tracks

**Track 1 — Discovery gates:**
- `all_dsl_files_parse`, `full_dsl_compiles`
- Emitted Rust/Go/Python syntax or compile checks
- Test freshness: regenerate -> diff -> empty

**Track 2 — Behavioral tests by construct:**

| .dag construct | Generated test | How it works | Status |
|---|---|---|---|
| **Type** | Roundtrip (structural witness) | Construct from composed samples -> serialize -> deserialize -> assert equal | Not yet |
| **Type** | Sample coverage | Each sample value roundtrips correctly | Not yet |
| **Service + `mock_response`** | Scenario invocation | DryRunMode, call with scenario mock, assert ok | Working (6 syntax tests) |
| **Service (no mock_response)** | Structural invocation | DryRunMode with compositional witness, assert shape | Not yet |
| **Pure function (`fn`)** | Property test | Composed sample inputs, assert no panics, assert return type | Not yet |
| **Workflow (`func`)** | Dry-run test | All services use compositional mocks, assert completes | Partial |

**Track 3 — Edge-contract coverage:**
For every edge in every compiled DAG, generate a producer->consumer
harness that executes with synthesized witness values and asserts port
cardinality, coercion, shape compatibility, and error behavior. For
joins/splits/guards, generate cross-products across adjacent ports and
branch outcomes. This is a natural extension of DryRun — wiring,
cardinality, coercion, guards, branching, topological ordering.

**Track 4 — Execution tiers:**
- Tier 1 DryRun: graph wiring, cardinality, coercion, ordering
- Tier 2 Selective Real: hermetic value correctness
- Tier 3 Full Real: controlled integration only

**Track 5 — Differential/parity:**
Same `.dag`, same mocks, same assertion shape across Rust/Go/Python.

## Obligation-driven test selection

The generator collects proof obligations per construct, discharges
obligations the compiler already proves structurally (type
compatibility, cardinality, acyclicity), and generates tests only for
undischarged obligations. This avoids tautological testing — don't
re-test what the compiler already guarantees by construction.

## Ratchet direction

Ratchets are checkpoints on the path to structural guarantees. Each
ratchet should trend toward its target value and eventually become
either a Tier 1 guarantee (structurally unrepresentable) or a Tier 2
guarantee (tested and gated in CI). A ratchet that stops moving is a
design signal — it means the current approach can't reach the target
and the machinery needs to change.
