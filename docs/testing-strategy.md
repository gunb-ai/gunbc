# gunbc Testing Strategy

This document describes the testing and verification strategy — what the
compiler proves, what it tests, and what it generates for external
verification. See [ROADMAP.md](../ROADMAP.md) M3 for current status.

## Core thesis: testing is compilation

Tests run DURING compilation. Failure is a compile error, not a report
the developer might not read. This is the same philosophy as
decidability: if the compiler can't prove termination, it doesn't
compile. If the compiler can't verify correctness, it doesn't compile.

The compiler assigns a status to every construct:

| Status | Meaning | Blocks compile? |
|---|---|---|
| `proven` | Compiler proved structurally (type, decidability, ownership) | No — nothing to test |
| `tested` | Hermetic test ran and passed | No |
| `under_specified` | Missing samples, mocks, witnesses, or behavioral oracle | **Yes** |
| `invalid` | Hermetic test failed or law violated | **Yes** |
| `integration_pending` | Integration contract exists, not yet verified live | No |
| `integration_failed` | Live integration test failed (CI lane only) | No |

## No tautological tests

Every node carries a **proof status**: what's already been proven by
the compiler or by a higher-level test. The test generator only
produces tests for undischarged obligations — never for things already
proven.

```
Node: to_fahrenheit(c: Celsius) -> Fahrenheit

Already proven (skip these):
  Type correctness      -- compiler proved statically
  Decidability (O(1))   -- complexity analyzer proved
  Ownership (SoleOwner) -- ownership analyzer proved
  Return type shape     -- inference proved

Undischarged (generate tests for these):
  ? Constraint satisfaction -- run output through Fahrenheit predicates
  ? Round-trip law          -- check to_celsius(to_fahrenheit(x)) == x
  ? Cross-target agreement  -- run on multiple targets

NOT generated (tautological):
  "to_fahrenheit returns Fahrenheit" -- already proven by type checker
  "to_fahrenheit terminates"         -- already proven by decidability
```

**A test is justified only at a boundary the compiler doesn't control.**

---

## Testing levels

Seven levels, ordered by what they prove. Each level subsumes the one
below — if Level 4 passes, Levels 0-3 are implied.

### Level 0: Data declaration correctness

**What:** Structural tests auto-generated from data declarations.
Every data declaration carries a testable claim about the system's behavior.

**How:** The compiler iterates over data tables (TypeCheckpoint,
InhabitantDecl, AlgebraFieldTemplate, etc.) and generates assertions
that the lookup functions return the declared values. Adding a new
data entry auto-generates a new test.

**Current state:** Implemented for coercion data. `extract_coercion_tests()`
in `coercion.dag` reads `target_checkpoints()` / `target_inhabitants()` and
produces `CoercionTestEntry` values. `compiler_tests_rust.dag` renders these
to Rust test functions. 24 checkpoint assertions + container + copy + template
tests, all derived from `dsl/extdeps/languages/*/types.dag`.

**Concept types:** `std/verification.dag` (AssertKind, TestClaim, TestCase),
`CoercionAssertion` / `CoercionTestEntry` in `coercion.dag`.

**Next candidates:**
- Algebra method semantics (AlgebraMethodSemantics → complexity analyzer tests)
- Keyword/reserved word tables (languages.dag → tokenizer tests)
- Runtime function registry → runtime shim coverage tests
- Container-to-algebra mapping → type resolution tests

### Level 1: Function correctness

**What:** Each compiler function produces correct output for known inputs.
Unit and integration tests that call pipeline functions directly.

**Current state:** 358 enabled tests in `v2-compiler-tests` crate.
Pipeline tests (226), source audits (37), parse tests (43), inference
semantics (34), diagnostics (11), bootstrap (7). Tests call stage0
functions directly — no subprocess overhead.

**Infrastructure:** `helpers.rs` provides `compile_dag()`,
`compile_dag_target()`, `compile_multi()`, `diagnostic_messages()`,
`assert_no_diagnostics()`, `find_file()`. FF-9 import resolution
via `build_module_index()` + `resolve_imports_transitively()`.

**Gap:** These tests are hand-written Rust. Many test the same patterns
that Level 0 could auto-generate. As structural test generation matures,
hand-written tests should shrink to only the cases that can't be derived
from data.

### Level 2: Bootstrap correctness

**What:** The compiler can process its own source. Self-hosting proof.

**Current state:** 7 bootstrap tests (mostly `#[ignore]` — expensive).
- `strict_compile_diagnostic_count`: 0 diagnostics (ratchet)
- `bootstrap_stage0_to_stage1`: emitted Rust cargo-checks (0 errors)
- `bootstrap_fixed_point`: stage1 == stage2 (idempotence)
- `performance_ratchet`: ~4.8s compile time (30s budget)
- Stage0 freshness gate: `regenerate → diff → empty`

**Infrastructure:** `regenerate-stage0.sh`, `check-stage0-freshness.sh`.
All stage0 files fully generated (0 hand-maintained). Bootstrap D complete.

**Self-test:** The compiler's own test suite (`compiler_tests.rs`) is
generated by the emitter. When the compiler self-compiles, it produces
its own tests — including Level 0 structural tests from its own data
declarations. The compiler tests itself using the same framework it
provides to user code.

### Level 3: Syntax validity

**What:** Emitted code is syntactically valid in the target language.

**Current state:**
- Rust: `cargo check` on stage1 (0 errors)
- Python: `python3 -c "import ast; ast.parse(...)"` on emitted files
- Go: structural checks (package, import, func Test, testing.T)

**What this proves:** The emitter produces parseable target-language code.
What it doesn't prove: the code does the right thing when run.

### Level 4: Semantic correctness

**What:** Emitted code runs and produces correct results.

**Five verification oracles, all but the last derive from structure:**

| Oracle | What it checks | Hand-written? |
|---|---|---|
| Type correctness | Output has the right shape | No (compiler proves statically) |
| Constraint satisfaction | Output satisfies `where` predicates | No (run output through type's predicates) |
| Algebraic laws | Operations satisfy declared laws | No (laws on algebra types) |
| Cross-target agreement | Same input → same output across targets | No (differential comparison) |
| Known values | Specific input → specific output | Sometimes |

**Current state:** Almost nothing. Only bootstrap fixed-point indirectly
executes emitted code. No tests run emitted Rust/Python/Go with inputs
and check outputs.

**What's needed:**
- Structural witnesses as test inputs (compositional mock generation)
- Constraint predicates as test oracles
- Algebraic law declarations as property tests
- Cross-target differential testing

### Level 5: Cross-language equivalence

**What:** Same `.dag` source → equivalent behavior in Rust, Python, Go.

**Current state:** Nothing. Each target is tested in isolation. No
differential tests.

**How:** Compile the same `.dag` to all targets. Run each with the same
inputs. Assert outputs match (or are equivalent modulo representation).
The test oracle is structural: if both targets inhabit the same algebra,
their behavior under that algebra's laws must agree.

### Level 6: Exhaustive form coverage

**What:** Every structural form the language can express compiles correctly.

**How:** The .dag input language is decidable and Node-bounded, so the
space of structural forms reaching the emitter is finite. The emission
algebra is `(NodeKind × TypeForm × Cardinality)` — enumerable from the
.dag type definitions. A test generator synthesizes one minimal `.dag`
program per element and compiles it to every target.

**Current state:** Nothing. `full_dsl_compiles` tests that existing .dag
files compile, but doesn't systematically cover the form space.

**What this proves:** Emission is total. No `compile_error!` or
`TODO` paths reachable by valid .dag input. New forms added to the
language automatically produce new test cases.

### Level 7: Performance contracts

**What:** Compile time, memory, operation counts stay within bounds.

**Current state:** Minimal.
- `performance_ratchet`: 30s budget, ~4.8s actual
- Individual tests >2s are suspect (project convention)
- Per-module reconcile profiling (RSS checkpoints, ignored)

**What's needed:** Operation-count contracts (node visits, inference passes,
emit calls). More deterministic than wall-clock, catches algorithmic
regressions independent of machine load.

---

## Compositional mock generation

Mock data is compositional. Sample values live ON the type definition
and propagate through composition. The compiler generates realistic
mocks by composing child samples.

### Structural witnesses (compiler-generated, automatic)

Every type gets a canonical witness from its structure:

| Type pattern | Canonical witness |
|---|---|
| Product (all edges) | All fields present with child witnesses |
| Coproduct (one edge) | First variant + each variant separately |
| Optional | BOTH: present(witness) AND absent |
| Refinement (`where`) | Value satisfying the constraint |
| Collection | Empty + one-element with child witness |
| Leaf (Int, String, Bool) | Zero / empty / false |

**Cardinality coverage falls out of structure.** A type with N optional
fields has 2^N cardinality combinations. The compiler generates witnesses
for all valid combinations (respecting cross-field constraints).

### Compositional samples (type-authored, domain-specific)

Types carry sample values as part of their definition:

```dag
type Url = String where pattern("https?://.*") {
  samples: ["https://example.com", "https://api.github.com/repos/owner/repo"]
}

// PullRequest mock is COMPOSED from field type samples:
//   number <- Int.samples -> [0, 1, -1]
//   title <- String.samples -> ["", "hello"]
//   html_url <- Url.samples -> ["https://example.com"]
type PullRequest { number: Int, title: String, html_url: Url, user: GitHubUser }
```

### Scenario mocks (hand-authored, cross-field)

Only needed when fields have cross-cutting constraints the type structure
alone can't express: "when state is 'open', merged_at should be null."
These supplement compositional mocks, not replace them.

---

## Test generation: what the compiler produces

The compiler generates test artifacts as part of emission. Tests are
output files alongside application code.

### Currently implemented

| Generator | Input | Output | Level |
|---|---|---|---|
| `extract_coercion_tests()` | TypeCheckpoint, InhabitantDecl data | Coercion assertion test functions | L0 |
| `extract_test_projections()` | Service operations with `mock_*` fields | Per-service test functions (DryRun) | L4 (partial) |
| `emit_compiler_tests_module()` | Compiler pipeline (self-compile) | compiler_tests.rs with bootstrap + structural tests | L0-L2 |
| Stage0 freshness gate | .dag source ↔ committed stage0 | Pass/fail (CI blocking) | L2 |
| Per-module test files | TestProjection per module | Rust/Python/Go test files | L4 (partial) |

### Not yet implemented

| Generator | Input | Output | Level |
|---|---|---|---|
| Structural witness generator | Type definitions | Canonical witness values per type | L4 |
| Algebraic law tester | `law` declarations on algebra types | Property tests from laws | L4 |
| Constraint oracle | `where` predicates on types | Output validation tests | L4 |
| Cross-target differential | Same .dag + multiple targets | Equivalence assertions | L5 |
| Form space enumerator | .dag type definitions (emission algebra) | One program per structural form | L6 |
| Operation-count contracts | Pipeline stage boundaries | Deterministic perf assertions | L7 |

---

## Execution tiers

```
Tier 1 (DryRun)       runs during compilation
                      All transports mocked.
                      Proves: wiring, types, coercion, structural.
                      Failure = compile error.

Tier 2 (Selective)    runs during compilation
                      Hermetic effects only (temp dirs, env vars).
                      Failure = compile error.

Tier 3 (Full Real)    NOT run during compilation
                      Requires real credentials, live services.
                      Compiler GENERATES the test as output artifact.
                      Receipt marks: "generated-not-run"
```

**Classification:** `std/fidelity.dag` defines `TestClass = Unit | Hermetic
| Integration` and `TransportClass`. `classify_transports()` derives the
tier from the transitive transport closure — max transport dominates.

---

## Guarantee receipt

The compiler emits a JSON receipt — the single authority for what the
compilation proved, tested, and left uncertain. If a guarantee is not
in the receipt, it does not exist.

```json
{
  "source_digest": "...",
  "compiler_digest": "...",
  "target": "rust",
  "structural": {
    "decidability": "proven",
    "type_correctness": "proven",
    "bootstrap_freshness": "pass"
  },
  "tested": {
    "coercion_data_integrity": "24 checkpoint + 15 container assertions",
    "service_dry_run": "pass (6 operations)",
    "self_compile_diagnostics": "0"
  },
  "generated_not_run": {
    "integration_tests": "tests/integration/*.rs"
  },
  "not_yet_covered": {
    "semantic_correctness": "no execution tests",
    "cross_target_equivalence": "not implemented",
    "exhaustive_form_coverage": "not implemented"
  }
}
```

**Status:** Not yet implemented as a concrete artifact. The concept is
sound; implementation depends on M3 (test generation and guarantee
receipt).

---

## Concept vocabulary

Test concepts live in `dsl/std/` (grounded, target-agnostic) and the
compiler pipeline (domain-specific realization).

### std/ layer

| File | Types | Purpose |
|---|---|---|
| `std/verification.dag` | AssertKind, TestClaim, TestCase | What a test IS — a named conjunction of verifiable claims |
| `std/fidelity.dag` | TestClass, TransportClass, DerivedClassification | Test cost classification — Unit vs Hermetic vs Integration |
| `std/behavioral.dag` | OperationBehavior, FailureMode, EdgeCase | Behavioral contracts for external systems |

### Compiler pipeline layer

| File | Types | Purpose |
|---|---|---|
| `05_emit.dag` | TestProjection, TestConventions | Service operation test extraction + per-target naming |
| `coercion.dag` | CoercionAssertion, CoercionTestEntry | Structural tests from coercion data declarations |
| `compiler_tests_rust.dag` | (rendering functions) | Rust-specific test string rendering |
| `languages.dag` | TestConventions, TestNameStyle | Per-language test file/function naming |

### Per-backend rendering

Each backend translates target-agnostic test data into its test framework:

| Backend | Test syntax | Assert syntax | File pattern |
|---|---|---|---|
| Rust | `#[test] fn name()` / `#[tokio::test] async fn` | `assert_eq!()`, `assert!()` | `tests/test_module.rs` |
| Python | `def test_name() -> None:` | `assert expr` | `tests/test_module.py` |
| Go | `func TestName(t *testing.T)` | `if got != want { t.Fatalf(...) }` | `module_test.go` |

---

## The compiler tests itself

The compiler is a .dag program. Its own testing follows the same
principles it provides to user code:

1. **Level 0** — Coercion data declarations generate coercion tests.
   AlgebraMethodSemantics data could generate complexity classifier tests.
   Runtime function registry could generate runtime coverage tests.

2. **Level 1** — 358 hand-written tests in v2-compiler-tests. These
   should shrink as Level 0 auto-generation covers more ground.

3. **Level 2** — Bootstrap tests verify self-hosting: the compiler can
   parse, compile, and emit its own source. Fixed-point convergence
   proves idempotence.

4. **Level 3** — Stage0→stage1 cargo check proves emitted Rust is valid.
   Python/Go syntax validation proves multi-target correctness.

When the compiler self-compiles, the emitter produces `compiler_tests.rs`
— which includes Level 0 structural tests generated from its own data
declarations. The compiler is both the test generator and the test subject.

---

## Phased implementation

Each level becomes a compile-error gate as it's implemented:

| Phase | What becomes a compile error | Status |
|---|---|---|
| **Done** | Missing service mock for declared hermetic boundary | Working |
| **Done** | Structural test failures from data declarations | Working (coercion) |
| **Next** | Type roundtrip failures (structural witnesses) | Not yet |
| **Next** | Failed algebraic law checks with samples | Not yet |
| **Future** | Cross-target disagreement | Not yet |
| **Future** | Exhaustive form coverage gaps | Not yet |

The receipt records which levels are active. Each level promotes from
"not checked" to "compile error" as it becomes implemented.

---

## Ratchet direction

Ratchets are checkpoints on the path to structural guarantees. Each
ratchet should trend toward its target and eventually become either a
structural guarantee (unrepresentable by construction) or a tested
gate (verified and gated in CI).

| Ratchet | Current | Target | Dissolves into |
|---|---|---|---|
| Self-compile diagnostics | 314 | 0 | L2 bootstrap gate |
| L1 type knowledge | 37 | 0 | L6 structural identity |
| Complexity violations | 164 | 0 | L0 from algebra data |
| Emitted Rust errors | 0 | 0 | L3 syntax validity (done) |

A ratchet that stops moving is a design signal — the current approach
can't reach the target and the machinery needs to change.
