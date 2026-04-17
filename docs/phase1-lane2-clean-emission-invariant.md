> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) > [single-emitter-design.md](./single-emitter-design.md)

# Lane 1 Stage 1c — Clean-emission invariant

**Lane:** 1 (Emission unification)
**Stage:** 1c (after substrate keyed-lookup; before consolidation build plan)
**Size:** M (split across PRs — see §2 Scope split)
**Status:** In progress. PR 1 (Rust pilot + contract type system) in flight.

> Role in the plan: establishes E-5 invariant (warnings-by-construction)
> as the contract that Stage 1e's consolidation dispatches on. The
> pilot implementation (unused pattern bindings) proves the contract
> shape before mass application.

---

## Motivation

**The compiler should never generate code that triggers warnings.**

Today, emitted code carries:
- Unconditional parenthesis wrapping (`((*(p0)) + (*(p0)))` → `unused_parens`)
- Pattern bindings emitted even when the body doesn't reference them
  (`Behavior::Value(v) => ...` where `v` is unused → `unused_variables`)
- Eager `use v3_compiler::diagnostics::*` imports regardless of need
  (→ `unused_imports`)
- Struct fields constructed but never read in specific test contexts
  (→ `dead_code`)

Each is a real bug: the emitter produces code that a target-native
verifier flags. The current workaround is `#[allow(warnings)]` on
emitted modules — a band-aid that lies about the code being correct.

**The structural fix:** every target declares its clean-code contract
as a substrate fact. Emission respects the contract by construction.
No post-emission cleanup. No suppression.

This lane designs the contract, codifies it as an invariant, and lands
the pilot implementation that proves the shape.

---

## Scope

Three deliverables.

### 1. Design doc: the invariant

Output: this file plus an `INVARIANTS.md` E-5 entry. Concrete contract
shape is pinned in [`design-clean-emission-contract.md`](./design-clean-emission-contract.md)
(DB-4) and is load-bearing for this stage — the 8-rule contract shape
below is authoritative; any drift between this sketch and DB-4 gets
reconciled against DB-4.

**The invariant (E-5):**
> For every target T and program P, the emission pipeline produces
> source code that passes T's declared clean-code verifier without
> modification. Violations are emission bugs, not formatting
> differences.
>
> **What "no escape hatches" means:** the contract GROWS by adding
> typed rules to `CleanEmissionContract` when new warning categories
> are encountered. It does NOT grow by adding `#[allow(...)]` /
> `# noqa` / pragma suppression. E-5's commitment is to
> constructive emission, not "we've already covered every
> category" — new targets will surface new categories, and each
> lands as a structural rule, not a silenced warning.

Per-target spec extension (DB-4 carries the full 8-rule field set;
this sketch names the fields, not their rule-variant dispatch):

```
// in spec/rust.dag (analogous for go.dag, python.dag, etc.)
data rust_clean_emission: CleanEmissionContract = {
  expression_wrapping: WrapOnlyInOperandPosition
  pattern_bindings: EmitUnderscoreWhenUnused
  imports: IncludeOnlyReferenced
  block_return: NoWrappingOnTerminalExpression
  variable_bindings: EmitUnderscoreWhenUnused
  match_arm_body: NoWrappingOnNonComplexBody
  correction_style: rust_correction_style
  post_emit_verifier: {
    command: "rustc"
    args: ["--edition=2021", "-D", "warnings"]
    syntax_only: false
    expected_exit_code: 0
    output_policy: IgnoreVerifierOutput
  }
}
```

`CleanEmissionContract` itself (8 fields) and the per-rule coproducts
(closed enums with `NotApplicable` variants where relevant) live in
`std/clean_emission.dag`. See DB-4 for the complete type table.

**Explicitly out of the E-5 surface:** emitted-struct dead-code.
Removed from DB-4 per codex feedback on PR #491 — that's a
publicity/visibility concern, not a clean-emission one. E-5 covers
constructive rendering rules only.

### 2. Pilot implementation: unused pattern bindings (Rust first)

Pick **one** clean-code rule and solve it structurally end-to-end.
Choice: `pattern_bindings: EmitUnderscoreWhenUnused`.

**Why this one:**
- Smallest localized change (`render_branch_pattern` + helpers)
- Doesn't require template restructuring (parens do)
- No test wrapper changes (imports do)
- Clean test: pattern match arms with unused bindings must emit `_`

**Scope split across PRs.** The stage originally sketched "all three
current targets (Rust/Go/Python) in one sweep." Practical scope split:

- **PR 1 (this PR):** Rust pilot — `std/clean_emission.dag` type
  system; `rust_clean_emission` data item in `spec/rust.dag`;
  `render_branch_pattern` dispatch; targeted test under
  `#[deny(unused_variables)]` proving the rule fires; E-5 entry in
  `INVARIANTS.md`.
- **PR 2 (follow-up):** Go + Python pilots — `go_clean_emission` /
  `python_clean_emission` data items and per-emitter dispatch.
  Deferred because Go's binding surface is `{x} := {expr};` (blank
  identifier path) and Python's pattern-bind doesn't emit the binding
  as part of the pattern at all — two distinct structural proofs,
  separate from Rust's `render_branch_pattern` path. E-6 holds in PR 1
  because each target's spec item lands only when its emitter
  consumer lands.
- **PR 3 (follow-up):** Wire `post_emit_verifier` into the test
  harness (rustc `-D warnings` hard gate, test-wrapper
  `#[allow(unused_variables)]` strike). Deferred because the test
  harness change is a distinct chunk that doesn't gate correctness
  of the rule dispatch.

**Implementation sketch (PR 1):**
1. `std/clean_emission.dag` — new file; declares
   `CleanEmissionContract` (8 fields per DB-4) and the coproduct
   rule types.
2. `spec/rust.dag` — adds `rust_clean_emission` data item with
   concrete rule variants.
3. `dag.rs` — caches `rust_clean_emission` in `TargetSyntaxCache`
   (name-lookup at bootstrap end, same pattern as `rust_rendering`).
4. `emit_rust.rs` — parses `CleanEmissionContract` into a typed
   binding; `render_branch_pattern` dispatches on
   `contract.pattern_bindings`. Port-liveness walk answers "is the
   binding's payload_port consumed by anything reachable from
   `path.output`?" — if not, emit `_`; else emit the binding name.
5. Targeted test — `#[deny(unused_variables)]` on a single fixture
   containing a match arm whose payload binding is unused; asserts
   compilation succeeds (i.e. emission produced `_`).

### 3. CI gate: post_emit_verifier as a hard check (follow-up PR)

Each emitter test suite adds a step: after emission, invoke the target's
declared `post_emit_verifier` and apply its `output_policy`. If it
reports any diagnostic or output shape the contract marks as failure,
the test fails.

**Today's rustc roundtrip tests do this partially** — they invoke rustc
but without `-D warnings`. After Stage 1c's follow-up PR, `-D warnings`
is declared IN THE SPEC and the test harness reads it from there.

**Transition plan:**
- Keep `#[allow(warnings)]` attributes in emitted modules during PR 1
  (pilot proves contract dispatch via a narrow `#[deny(unused_variables)]`
  test, not by stripping the umbrella allow).
- PR 3 strikes `unused_variables` from the test-wrapper allow list
  (once Rust + Go + Python all dispatch the rule).
- Lane 1d/1e consolidates the remaining categories (parens, imports,
  etc.) and strikes those as they land.

**Do NOT attempt to fix all four warning categories in this lane.** The
pilot validates the contract shape. The rest are structural work that
folds into Lane 1d/1e's emitter consolidation.

---

## Out-of-scope

- Parens cleanup (`unused_parens`) — fold into P2's template redesign
- Conditional imports (`unused_imports`) — fold into P2's generic module header
- Dead-code attributes on generated structs (`dead_code`) — test-context
  concern, separate lane
- Any Shape B emission concerns (SPICE / Verilog / English) — **not compiler targets** per THESIS.md §"Two shapes"; Shape B artifacts are produced by `.dag` PROGRAMS, not the compiler
- `gofmt`/`black`/`pylint` wrapper work beyond declaring the
  verifier commands — actual tool integration is P2

---

## Direction

**Design before implementation.** Finalize `CleanEmissionContract`
shape in the design doc before touching code. The contract type is
what P2's entire consolidation will dispatch on; getting it right once
is cheaper than patching it three times.

**Pilot is a proof, not a sweep.** One warning category, across all
three current targets (Rust/Go/Python), structurally fixed. If the
pilot requires awkward contortions to implement the rule cleanly,
that's a signal the contract shape is wrong — refactor the contract,
not the implementation.

**Invariant in INVARIANTS.md is load-bearing.** Once E-5 is declared,
every future emitter change must justify compliance. This is the
mechanism that prevents regression.

---

## Escalation criteria

Stop work and surface if:

1. **Port liveness analysis reveals scope complications** — e.g.,
   captures into lambdas, Bind parameters referenced indirectly through
   Branch paths. If the analysis can't be done cleanly in
   `render_branch_pattern`, the primitive may need to be a
   substrate-level lens (port liveness lens). Surface — this would be
   a legitimate new lens, not scope creep.

2. **CleanEmissionContract shape doesn't compose** — e.g., two rules
   interact (pattern binding rule depends on which imports are
   present). If composition matters, the contract needs a
   precedence/ordering semantics. Surface; don't invent it locally.

3. **Target spec bloat** — if each target's `clean_emission` field
   becomes large (>20 subfields), the contract is probably too
   low-level. Aggregate into higher-level named policies (e.g.,
   `"rust_default"`, `"go_strict"`). Surface.

4. **Pilot implementation requires >200 lines in emit_rust.rs** —
   beyond that, the pattern-binding fix is absorbing unrelated
   concerns. Surface and slice tighter.

---

## Acceptance gates

Stage 1c is done when the following hold across its PRs.

**PR 1 (Rust pilot + contract type system) — gates:**

- DB-4 (`docs/design-clean-emission-contract.md`) exists with the
  E-5 invariant framing, the 8-rule `CleanEmissionContract` type
  table, per-target rule choices, and the "reject post-emission
  cleanup" rationale. (Already landed; referenced here.)
- `INVARIANTS.md` has an E-5 entry referencing DB-4 and this stage
  doc.
- `std/clean_emission.dag` declares the 8-rule contract type plus
  each rule's coproduct (with `NotApplicable` variants where
  relevant).
- `spec/rust.dag` declares `rust_clean_emission: CleanEmissionContract`
  populated with concrete rule variants per DB-4.
- `dag.rs` caches `rust_clean_emission` in `TargetSyntaxCache` with
  an accessor on `Dag`.
- `emit_rust.rs` parses the contract into a typed binding and
  dispatches `PatternBindingRule::EmitUnderscoreWhenUnused` in
  `render_branch_pattern`. Port-liveness walk determines binding use.
- A targeted test under `#[deny(unused_variables)]` compiles a
  match-with-unused-payload and asserts rustc accepts the emitted
  code (i.e. the pilot emits `_` and no unused-variable warning
  fires).
- Zero new `#[allow(...)]` attributes added in this PR.

**PR 2 (Go + Python pilots) — gates:**

- `spec/go.dag` / `spec/python.dag` each declare their
  `*_clean_emission` data item.
- `emit_go.rs` / `emit_python.rs` each dispatch their
  `pattern_bindings` rule. Per-target surface: Go emits `_` for the
  blank-identifier path; Python dispatches
  `EmitPrefixedUnderscoreWhenUnused` (`_v` convention).
- Targeted tests per target mirror the Rust pilot.

**PR 3 (post_emit_verifier CI gate) — gates:**

- Test harness reads `post_emit_verifier` from each target's
  contract and invokes it with its verdict policy (Rust: rustc
  with `-D warnings` + `IgnoreVerifierOutput`; Go: `gofmt -l` +
  `RequireEmptyStdout`; Python: py_compile +
  `IgnoreVerifierOutput`).
- `unused_variables` removed from each test-wrapper
  `#[allow(warnings, clippy::all)]` umbrella (replaced with an
  explicit residual list naming only the not-yet-structurally-fixed
  categories).
- CI fails on any warning diagnostic the verifier emits.

**Cross-stage gate:** zero new `#[allow(...)]` attributes added
anywhere during Stage 1c. Existing umbrella allow lists narrow
monotonically as each rule lands.

---

## Dependencies

- **Requires:** P1-L1 complete (uses the post-Consumed renderer as
  the starting point; confusing to layer clean-emission on top of a
  renderer that's mid-change).
- **Blocks:** P2-L1 (consolidation uses the `CleanEmissionContract`
  as its invariant framework). Without this lane, consolidation has
  no clean-code contract to dispatch on.
- **Does not block:** P1-L3 (design-only; can proceed in parallel).

---

## Estimate

- Design doc + `INVARIANTS.md` entry: 2 days
- `CleanEmissionContract` type + spec population: 1 day
- Pilot implementation (pattern bindings, 3 targets): 2 days
- CI wiring + test cleanup: 1 day

Total: ~6 implementer-days.

---

## Success signal

Imagine a contributor adds a Shape A target like Swift or Kotlin. If the clean-emission
contract is right, they should be able to write:

```
data swift_clean_emission: CleanEmissionContract = {
  pattern_bindings: ...   // whatever Swift's convention is
  ...
  post_emit_verifier: { command: "swiftc", args: ["-parse", "-warnings-as-errors"] }
}
```

…and the existing generic emitter (from P2) produces warning-clean
Swift without any new Rust code. That's the shape this lane is
designing toward.

---

## Companion invariants (historical — already landed)

Half B proposed three emission-discipline invariants that were
originally listed here as "candidates to evaluate alongside E-5."
All three landed in `INVARIANTS.md` ahead of this stage:

- **E-6** — No target-spec field lands without a same-PR consumer.
  (PR #493, 2026-04-16.)
- **E-7** — No target-private realization schema lands without a
  dissolution ratchet. (PR #493, 2026-04-16.)
- **E-8** — Unsupported core behaviors fail closed; never collapse
  semantically. (PR #493, 2026-04-16.)

**E-9** — External realization lives on `Arrow.body` — landed in
PR #497, 2026-04-17.

This section is retained as a pointer so readers of the original
stage doc know these invariants are load-bearing and already
enforced; Stage 1c does not re-evaluate them.
