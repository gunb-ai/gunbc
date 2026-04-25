# Testing — gunbc

**Philosophy.** Google C++-style: hermetic, behavior-driven, unit-first.
Each test names an **interface** and a **behavior** that interface
promises. Heavy integration tests are an exception, not the rule.
Dependency injection + minimal-constructed inputs are the default;
full-pipeline `compile_to_dag(...)` is reserved for tests whose
subject genuinely is the pipeline.

## Adoption

This document describes the live discipline for **new code and
refactors**. The existing test suite does not satisfy every
prescription — known divergences (large integration-test share,
imperative substrate walks, exhaustive testgen validation) are
documented in the *Migration audit* section below, with refactor
priorities tracked against ROADMAP.

**Enforcement stance:** reviewers should flag a NEW PR that
violates these guidelines as a `KEEP_ITERATING` signal. An
EXISTING file that violates them is documented debt, not a
present failure — it migrates on touch or via a dedicated paydown
lane. This keeps the doc honest against the live-state invariant
without forcing a blocking refactor backlog.

## Five principles

### 1. Hermetic

Each test declares its own inputs and assertions. No cross-test
mutable state. No fixture that "sets things up" across many
tests — if two tests need the same input, they each declare it.
The runtime (e.g. `cached_compile_to_dag`, `.dag`-native test
runner under DB-15 R2) amortizes identical inputs behind the
scenes; the **logical** hermeticity stays at the test level.

### 2. Behavior-driven, not implementation-driven

A test names the **interface** and the **behavior**:

- Interface: `cost_of(dag, port) -> CostLookup`
- Behavior: *"returns `FoundCost(n)` where n grows linearly with
  input list size"*

Not: *"the resulting HashMap has 3 keys in alphabetical order."*
Implementation details of how a cost is computed are not the
contract; the contract is what the caller can rely on. Tests
that break on internal refactors pin the wrong surface.

### 3. Cost of change

A good test survives any refactor that preserves semantics. If
`Dag`'s internal storage changes from `Vec<Declaration>` to
`BTreeMap<DeclarationId, Declaration>`, tests that asked
"is there a declaration named X?" still pass. Tests that asked
"is index 7 of the declarations vec equal to X?" break — and
shouldn't have existed. Pin behavior, not layout.

### 4. One claim per test

Each `#[test]` (or `data foo: TestClaim`) makes **one**
structural claim. Multi-claim tests bundle unrelated behaviors
and obscure which contract broke when CI goes red. The test
name should read like a sentence — subject, verb, object, under
what condition:

```rust
#[test]
fn cost_of_empty_fold_returns_constant_zero() { ... }

#[test]
fn cost_of_linear_fold_scales_with_source_port_size() { ... }
```

Not:

```rust
#[test]
fn cost_lens_works() {  // what behavior?
    // 20 lines, 6 assertions, 3 unrelated claims
}
```

### 5. Mocks over compile

The default for unit tests is to **construct a minimal `Dag`**
with the structural shape under test, not to compile a source
string. Compiling `"let x = 1 + 2"` and then asserting
"there's a Transform node for `+`" exercises the entire
pipeline (parse + lower + infer) as a side effect of testing
one structural invariant. If that test fails, the root cause
could be anywhere in ~30k lines of compiler code.

Instead, construct the exact shape under test and run the lens
against it.

**Availability today — honest read of the current tree.** The
public surface of `Dag` is narrow: `Dag::new()`,
`declaration_by_name`, typed handle accessors, primitive shape
lookups. Crate-private there's one builder helper
(`Dag::alloc_port`); a full minimal-construction API
(`push_value`, `push_transform`, `push_bind`, etc.) **does
not yet exist**. This means the guidance above is today more
aspiration than available workflow.

**Tracked follow-up:** land a public (or at minimum
`pub(crate)`-with-test-harness-access) builder surface on
`Dag` covering the behaviors lenses analyze. The absence of
that surface is the single biggest blocker to the
`~75% unit` target ratio.

**Practical guidance for now:**

- For crate-internal unit tests (`#[cfg(test)] mod tests`
  inside `src/v3/compiler/src/`): `alloc_port` and the other
  `pub(crate)` APIs are in scope. Write tests that need only
  those.
- For integration tests in `tests/`: `compile_to_dag(small_fixture)`
  is the practical entry point today. Keep the fixture
  minimal (single `let`, single `fn`) and assert on the lens
  output, not pipeline intermediates.
- When you find yourself wanting a `push_*` helper that
  doesn't exist, proposing it is the right move — see the
  follow-up above.

Full-pipeline `compile_to_dag` is the **intended** entry point
for:
- **Integration tests** — deliberate end-to-end coverage of the
  pipeline itself
- **Thesis tests** — claims about user-facing behavior, where
  the source text is part of the interface
- **Boundary tests** — target-language roundtrips (rustc, go, python)

**Scope clarifier:** the "mocks over compile" anti-pattern applies
to lens / accessor / single-pass tests where the subject under
test is narrower than the whole pipeline. For the three
categories above, `compile_to_dag` **is** the correct entry point
— the pipeline is the unit. Don't force minimal-`Dag` construction
where the test legitimately targets end-to-end behavior.

## Test layers (target ratios)

| Layer | Share of tests | What belongs here | Typical time per test |
|---|---|---|---|
| **Unit** | ~75% | lenses, single-pass behaviors, accessors, typed handles, substrate walks against hand-built `Dag` | <5ms |
| **Integration** | ~15% | multi-stage pipeline behaviors, fixed-point convergence, cross-stage invariants | <100ms |
| **Boundary** | ~10% | rustc/go/python roundtrips, emitted-module behavior | <2s |
| Thesis | across the above | claims in the thesis vocabulary (cost bounds, parallelism, structural invariants). Write as whichever layer is natural — usually unit or integration | — |

**Red-flag ratios** — if your suite is >30% integration or any
test takes >2s locally (cold bootstrap aside), the suite is
drifting into heavy-integration territory. See
`feedback_test_timeout_2s`.

## Mocks and dependency injection

Prefer passing `&Dag` / `&dyn Lens` / etc. over globals. Prefer
constructing the minimal Dag shape over compiling source.
Prefer typed-carrier assertions over name-keyed lookups (a test
that does `dag.declaration_by_name("Foo")` is pinning the name,
not the structure — use typed `DeclarationId` handles once
resolved).

**Constructing a minimal Dag — eventual shape.** Builder
helpers on `Dag` that produce specific shapes (value literals,
transforms, binds, port allocations) are the intended mocking
surface. Today only `alloc_port` exists as a `pub(crate)`
helper; the rest are the tracked follow-up named earlier in
this document. When proposing a new helper, match the natural
per-variant granularity of `Behavior` / `TypeConnective`.

**When compile IS the unit** — parser / lowering / inference
tests legitimately test the compile pipeline. Those tests use
`compile_to_dag(source, file)` because the **pipeline itself**
is the interface they're testing. That's fine. Most lens and
accessor tests are not in this category.

## `.dag`-native testing (DB-15 R2 trajectory)

The long-term shape: tests are **declarations** in `.dag`, not
Rust imperative harnesses. Each test is a
`data test_foo: TestClaim = { name, source, predicate }`
declaration. The test runner reads the declaration graph,
amortizes identical `source` compiles via the dependency walk
(not via `OnceLock`-cached Rust state), and evaluates each
predicate structurally.

At that point the hermetic principle still holds — each
`TestClaim` declaration is its own unit, the runtime just
shares the compile work when sources match.

> **🔄 RETRACTED 2026-04-25 (cascade promotion of `docs/design-pure-bootstrap-zero.md`).** The previous "two residual categories" framing (compiler-internal unit tests + external-toolchain boundary tests stay Rust-authored permanently) is **retracted under the 0-floor target**. Both categories dissolve:
>
> - **Rust-only-helper unit tests vanish naturally** — under 0-floor (per `docs/design-pure-bootstrap-zero.md` LIVE), there are no Rust-only helpers in v3's source tree; everything generates from `.dag`. Unit tests for non-existent helpers don't need to exist.
> - **External-toolchain boundary tests migrate to `ExecuteCommand`-based `.dag` `TestClaim` declarations** as the cascade-named successor pattern. **Capability state (2026-04-25, PB-Runtime `ExecuteCommand` extension landed):** the schema is still `src/v3/std/verification.dag:115-119` (PR #678). The Rust `TestRunner` and the M1.5 testgen harness now share one `std::process` path for arbitrary `command` + `args` + `expect_exit_code` — `evaluate_execute_command_exit_code` and `parse_execute_command_fields` in `src/v3/compiler/src/test_runner.rs`; exit mismatches, spawn errors, wall-clock **timeout**, and **policy** fails (e.g. common shells with `-c` and a **bare** `&` that may be shell background, after a small elision of `&&` and common `n>&m` / `&>`-style token spellings) are distinguishable `ClaimResult::Fail` messages. **Linux** (when the host allows `unshare(1)`) also wraps the logical command in `unshare(1) -c -f -p` (user + PID namespace) so exiting the namespace **init** can tear down a contained process **subtree**; on `unshare(1)` `EPERM` (e.g. some container defaults), the runner **falls back** to the same direct `Child`+wall+null stdio path as non-Linux, without that isolation. **Non-Linux** uses that direct path. The child’s stdin/stdout/stderr are **not** captured (null device) so exit-code–only claims cannot OOM the runner on huge output. A fixed wall bound (`EXECUTE_COMMAND_WALL_TIMEOUT` in the same file) fail-closes unbounded hang; on Unix, timeout also signals the child’s process group. The M1.5 allowlist + fail-closed panic for disallowed `ExecuteCommand` shapes are **retired** (see `src/v3/compiler/tests/integration/m1_5_testgen_test.rs` — the harness is bool-only; triage strings are `TestRunner`’s). A landed T-PB-B-1 boundary migration example is `src/v3/compiler/tests/dag/t_pb_b_1_execute_command_boundary.dag` (`t_pb_b_1_dag_runner_test`), structurally paralleling the class-5 *spawn + exit* receipt from e.g. `m1_4_emit_python_test::python_stdout` without requiring CPython in every CI run. Residual **bulk** migration of older Rust `Command::new("rustc" | "python3" | …)` boundary tests to `.dag` remains its own work item on ROADMAP. Full process-tree and sandbox **policy** (beyond `&` in shell `-c` and pgrp+timeout) is a separate, Director-level track.
>
> **0-residual is the target.** Until v3's source tree reaches 0 hand-authored files, write tests in whichever shape (Rust integration test OR `.dag` `TestClaim` with `ExecuteCommand`) is cleanest. With the full `ExecuteCommand` runner available, new boundary intent should default to the `.dag` `TestClaim` form when exit-code is enough; residual Rust tests migrate that way. See `docs/design-pure-bootstrap-zero.md` §"PB-Runtime" for the broader cascade scope.

Until DB-15 R2's runtime lands, write Rust integration tests
that match the guidelines above so the eventual port is a
rewrite of shape, not of intent.

~~**Post-R2 shape:** once DB-15 R2 ships, most of this document collapses into "see `dsl/std/verification.dag` for the test surface." The Rust-side residual is: compiler-internal unit tests (`#[cfg(test)] mod tests`) inside `src/v3/compiler/src/`, and boundary tests that invoke external toolchains. Everything else ports to `.dag`.~~

**0-floor shape (post cascade promotion 2026-04-25):** once DB-15 R2 ships AND the Zero-Floor program reaches 0 hand-authored files (per `docs/design-pure-bootstrap-zero.md` LIVE), this document collapses into "see `dsl/std/verification.dag` for the test surface." (The `ExecuteCommand` runner extension is already in place — arbitrary `command` + `args` with exit-code capture; see capability-state note above.) There is **no Rust-side residual**: compiler-internal unit tests don't exist (no Rust-only helpers); external-toolchain tests use `ExecuteCommand`-based `.dag` `TestClaim` declarations. Everything is `.dag`.

## Anti-patterns

### Don't compile a full source to test a single lens

```rust
// ❌ anti-pattern
#[test]
fn cost_lens_handles_fold() {
    let dag = compile_to_dag(
        "let total: Int = fold(singleton(1), 0, |acc, x| acc + x)",
        "fold.v3"
    ).expect("compiles");
    let total = find_bind_by_name(&dag, "total");
    assert_eq!(cost_of(&dag, &total.value), CostLookup::FoundCost(3));
}
```

What's wrong: the test exercises parse, lowering, inference,
and cost — a failure anywhere reports "the cost lens broke."
And the `"let total: Int = fold(...)"` fixture is now a
coupled input for both the parser and the cost lens.

```rust
// ✅ unit: construct the shape the cost lens analyzes
#[test]
fn cost_of_fold_over_linear_source_returns_linear() {
    let mut dag = Dag::new();
    let source = dag.alloc_port_with_linear_shape(/* size_var */);
    let fold = dag.push_fold_transform(source, /* init, body */);
    assert!(matches!(
        cost_of(&dag, &fold.output()),
        CostLookup::FoundCost(SymbolicCost::LinearCost(_)),
    ));
}
```

### Don't assert on implementation details

```rust
// ❌ pinning layout
assert_eq!(dag.declarations().len(), 247);
assert_eq!(dag.declarations()[3].name, Some("OrderedRing".into()));

// ❌ pinning error message text
assert!(diagnostic.detail().contains("missing field"));

// ❌ pinning HashMap iteration order
let keys: Vec<_> = report.unused.keys().collect();
assert_eq!(keys, vec!["foo", "bar"]);
```

```rust
// ✅ pinning structure
assert!(dag.declaration_by_name("OrderedRing").is_some());
assert!(matches!(diagnostic, Diagnostic::FieldNotFound { .. }));
assert!(report.unused.contains_key("foo"));
assert!(report.unused.contains_key("bar"));
```

### Don't use cross-test shared state

Shared setup via `OnceLock` or `lazy_static` is a runtime
amortization (see `cached_compile_to_dag`) — it speeds up
compiles that happen to repeat across tests, but each test's
**logical** inputs stay per-test. A test that reads from a
shared `Arc<Mutex<Report>>` populated by earlier tests is not
hermetic and will fail under `--test-threads=1` or random
ordering.

### Don't write tests that span multiple behaviors

If a test needs three distinct assertions to "tell the whole
story," split into three tests. Each failure then points at
exactly one broken contract.

### Don't test private state through public surfaces

Tests should exercise public interfaces. If you need to inspect
a private field, the interface is under-specified — either
expose the fact structurally (a new typed accessor, a lens
output field) or accept that the private state isn't part of
the contract and shouldn't be tested.

## Naming convention

`<subject>_<verb>_<object>_<condition>`:

- `cost_of_empty_fold_returns_constant_zero`
- `field_access_on_non_conj_type_is_rejected_with_diagnostic`
- `branch_arm_of_returns_none_for_non_bool_port`
- `workflow_effect_linear_chain_composes_idempotent_when_all_ops_idempotent`

Tests named `test_X`, `foo_works`, `basic_thing`, or
`regression_N` are under-specified — the reader can't tell what
the test proves without reading the body. The name IS the
contract claim.

## Migration audit (current state)

The v3 integration suite has ~357 tests across 29 files. Most
predate these guidelines. Rough current breakdown:

| Category | Files | Tests | Status |
|---|---|---|---|
| Thesis / behavior | 2 | 28 | well-shaped; port to `.dag` under DB-15 R2 |
| Substrate walks | 4 | 100 | imperative Rust assertions of structural shapes; candidates for lens-based consolidation |
| Lens outputs | 6 | 60 | natural fit for `.dag`-native declarations |
| Class-5 boundary | 6 | 19 | keep in Rust until runtime is `.dag`-native |
| Target emission | 4 | 54 | keep; boundary tests |
| Milestone / feature | 4 | 80 | mixed; `m0_acceptance` likely subsumed by later milestones |
| Testgen / meta | 1 | 3 | over-expensive; reshape or `#[ignore]`-by-default |
| Smoke / low-value | 4 | 32 | consolidate or delete (`real_stdlib_parse_smoke` is weakest) |

Refactor targets, in order of ROI:
1. **Audit `m0_acceptance.rs`** — 41 tests from M0 skeleton; cross-check coverage against later milestones and delete overlaps.
2. **Collapse `m1_substrate_test.rs`** — 91 imperative substrate-walk assertions into ~20 lens-based structural claims.
3. **Reshape `m1_5_testgen_test.rs`** — spot-check instead of exhaustive compile-every-claim.
4. **Port thesis + lens tests to `.dag`** once DB-15 R2 runtime lands.

See `docs/design-test-infra.md` and ROADMAP §Lane 2 Stage 2c for
the DB-15 trajectory. This document is the near-term discipline
for Rust-side tests while that runtime matures.

## Cementing tests (Band C — lens subsumption)

**Where this applies.** Whenever a brief, roadmap line, or review
claims that v3 **subsumes** or is **behaviorally equivalent** to a
v2 analysis for the same program text, that claim is a Band-C
statement about the *function* of the lens, not about whether the
`.dag` file is structurally terminal. The authoritative inventory
is `docs/v3-lens-capability-register.md` (behavioral axis + v2
counterpart column). Any `LensRegistryEntry` in
`src/v3/compiler/regen.dag` whose row moves to
`BEHAVIORALLY COMPLETE` while still naming a real v2 counterpart
(not `None (v3-native)` / not `N/A`) inherits the same obligation.

**What “cementing” means.** A cementing test is a **behavioral**
regression that would fail if v3 silently changed the analysis
while prose still claimed parity. Concretely:

- **v2 counterpart exists:** run the same minimal fixture through
  both implementations (v2 oracle + v3 lens output) and assert
  semantic equality on the published carrier shape — or assert a
  documented, reviewed projection when the types differ but the
  claim is about a specific homomorphism.
- **No v2 counterpart (`N/A` / v3-native) but the register marks
  `BEHAVIORALLY COMPLETE`:** pin the lens’s published behavioral
  contract on **minimal constructed `Dag` shapes** (or a single
  tiny `compile_to_dag` fixture when the contract genuinely spans
  the pipeline). This is still a cementing test: it cements the
  `COMPLETE` row against accidental semantic drift.

**Where tests live.** Add modules under
`src/v3/compiler/tests/integration/cementing/` and register each
file from `src/v3/compiler/tests/integration.rs` via `#[path =
"integration/cementing/<file>.rs"]`. Prefer one focused module per
lens (or per v2 parity claim), not a kitchen-sink file.

**Dispatch (same PR as the claim).** Editing
`src/v3/compiler/regen.dag` or promoting a register row to
`COMPLETE` with v2 parity is not complete until the matching
cementing module exists and `tests/integration.rs` wires it. Extend
`CEMENTING_MODULES_FOR_V2_COMPLETE_CLAIMS` in
`tests/integration/cementing/cementing_lens_registry_dispatch_test.rs`
when the register row is upgraded; the test
`cementing_escalation_slice_matches_capability_register` derives the
required registry `name` keys from `docs/v3-lens-capability-register.md`
plus `regen.dag` and asserts the slice matches **exactly** (the slice
is a projection, not a parallel inventory). The test
`cementing_test_modules_exist_for_escalated_v2_complete_registry_claims`
then requires each listed stem to resolve to an on-disk `.rs` file under
`cementing/` **and** to appear in `tests/integration.rs` behind an
exact `#[path = "integration/cementing/<stem>.rs"]` attribute (so an
orphan file cannot satisfy the ratchet).

**Anti-scope.** This section is not a mandate to prove full lens
equivalence for every `.dag` file, and it does not require new
substrate accessors or emitters. It only governs **explicit**
subsumption or `COMPLETE`-with-counterpart claims.

## Related

- `INVARIANTS.md` — project-level invariants; some have direct
  test implications (fail-closed, illegal-states-unrepresentable)
- `MODELING.md` — what to model in `std/`; tests should exercise
  the modeled concepts, not rebuild them
- `docs/design-test-infra.md` (DB-15) — the `.dag`-native test
  infrastructure this document is the stopgap for
- `docs/v3-lens-capability-register.md` — Band-C behavioral axis;
  cross-check before dispatching “v3 replaces v2 X” briefs
