> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) > [single-emitter-design.md](./single-emitter-design.md)

# Lane 1 Stage 1c — Clean-emission invariant

**Lane:** 1 (Emission unification)
**Stage:** 1c (after substrate keyed-lookup; before consolidation build plan)
**Time budget:** ~1 week
**Status:** Plan. No code changes yet.

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

Three deliverables:

### 1. Design doc: the invariant

Output: this file's successor becomes authoritative (`docs/clean-emission-invariant.md`),
plus an `INVARIANTS.md` entry.

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

Per-target spec extension:

```
// in spec/rust.dag (analogous for go.dag, python.dag, etc.)
data rust_clean_emission: CleanEmissionContract = {
  expression_wrapping: WrapOnlyInOperandPosition
  pattern_bindings: EmitUnderscoreWhenUnused
  imports: IncludeOnlyReferenced
  block_return: NoRedundantWrapping
  post_emit_verifier: {
    command: "rustc"
    args: ["--edition=2021", "-D", "warnings", "..."]
  }
}

type CleanEmissionContract {
  expression_wrapping: ExpressionWrappingRule
  pattern_bindings: PatternBindingRule
  imports: ImportRule
  block_return: BlockReturnRule
  post_emit_verifier: PostEmitVerifier
}

type ExpressionWrappingRule
  = WrapEverything           // current behavior — rejected
  | WrapOnlyInOperandPosition // correct
```

### 2. Pilot implementation: unused pattern bindings

Pick **one** clean-code rule and solve it structurally end-to-end.
Choice: `pattern_bindings: EmitUnderscoreWhenUnused`.

**Why this one:**
- Smallest localized change (`render_branch_pattern` + helpers)
- Doesn't require template restructuring (parens do)
- No test wrapper changes (imports do)
- Clean test: pattern match arms with unused bindings must emit `_`

**Implementation sketch:**
1. Add `CleanEmissionContract` type in `src/v3/spec/clean_emission.dag` (new file)
2. Add `pattern_bindings` field to each target's spec (`rust.dag`, `go.dag`, `python.dag`)
3. In `emit_rust.rs` `render_branch_pattern`:
   - Walk the arm body to collect referenced PortIds
   - For each payload binding in the pattern: emit `_` if its port is
     not in the referenced set, else emit the binding name
4. In `emit_go.rs` / `emit_python.rs`: analogous using Go's `_` and
   Python's `_var_name` conventions per each target's rule
5. Add a `render_locals_analysis` helper (target-agnostic) that
   computes the referenced-ports set for a body — this is the
   "port liveness" primitive P2 will also consume

### 3. CI gate: post_emit_verifier as a hard check

Each emitter test suite adds a step: after emission, invoke the target's
declared `post_emit_verifier`. If it reports any diagnostic (warning or
error), the test fails.

**Today's rustc roundtrip tests do this partially** — they invoke rustc
but without `-D warnings`. After L2, `-D warnings` is declared IN THE
SPEC and the test harness reads it from there.

**Transition plan:**
- Keep `#[allow(warnings)]` attributes in emitted modules during this
  lane's pilot implementation
- As each warning category is structurally fixed, remove the
  corresponding lint from the allow list
- When the pilot (unused pattern bindings) ships: remove
  `unused_variables` from the allow list
- P2 consolidates the remaining categories (parens, imports, etc.)

**Do NOT attempt to fix all four warning categories in this lane.** The
pilot validates the contract shape. The rest are structural work that
folds into P2's emitter consolidation.

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

Lane is done when all four hold:

- `docs/clean-emission-invariant.md` exists with the E-5 invariant,
  `CleanEmissionContract` type definition, per-target rule table, and
  the "reject post-emission cleanup" rationale.
- `INVARIANTS.md` has an E-5 entry referencing the design doc.
- `spec/clean_emission.dag` (or equivalent) declares the contract
  type; each of `spec/rust.dag`, `spec/go.dag`, `spec/python.dag` has
  a populated `clean_emission` field.
- The unused-pattern-binding rule fires end-to-end: a test program
  with `match x { Value(v) => 0, ... }` produces Rust with
  `Behavior::Value(_) => 0` (not `Value(v)`), and `unused_variables`
  is removed from the test-wrapper allow list for that specific case.
- Zero new `#[allow(...)]` attributes added anywhere in the codebase
  during this lane.

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
data verilog_clean_emission: CleanEmissionContract = {
  pattern_bindings: ...   // whatever Swift's convention is
  ...
  post_emit_verifier: { command: "verilator", args: ["--lint-only"] }
}
```

…and the existing generic emitter (from P2) produces warning-clean
Swift without any new Rust code. That's the shape this lane is
designing toward.
