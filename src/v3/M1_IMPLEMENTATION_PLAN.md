# M1(2.6+) Implementation Plan — Upfront Enumeration

**Baseline:** `free-cod-972` as of round-6 + dissolution work (current HEAD
at time of writing).
**Status:** plan-of-record for M1(2.6) tail + M1(3) + M1(4).
**Enforced by:** `INVARIANTS.md` §"No short-term solutions," §"No bridges,"
§"No deprecations," and the enumeration discipline codified in
`feedback_enumerate_before_substrate.md`.

---

## §1. Why this document exists

Every reviewer round on `free-cod-972` has found one overloaded field where
a downstream stage reconstructs a fact the upstream stage did not represent
structurally. The pattern is consistent:

1. Upstream writes a field `X`
2. Downstream needs to distinguish two cases from `X`
3. Downstream reconstructs via name inspection / string match / lifecycle-
   state inference
4. Reviewer flags it as a bridge
5. Round fixes one field
6. Next round finds the next overloaded field
7. Go to step 1

This document breaks the loop. Its purpose is to **enumerate every
structural distinction downstream consumers need before writing any more
downstream code**, AND to pin the authority / scope / invalid-state /
boundary-flow model first so downstream stages share a single oracle.
Gaps become pre-identified substrate work, not post-implementation
bridges. No consumer writes code until the substrate answers its
questions structurally.

**Per `INVARIANTS.md` §"No short-term solutions":** PRs under this plan
are deliberately larger than industry default. See §7 below for rationale.
The short version: bridges appear at layer boundaries, and a PR that can
only see one layer at a time will invent a provisional shape that the
next layer has to reconstruct. The antidote is atomic PRs that touch
substrate + every affected consumer + their shared reading patterns in
one push.

---

## §2. Baseline: `free-cod-972` current state

**Substrate shape** (after round-6 + latest dissolutions):

- `TypeConnective`: 6 variants (Atom, Conj, Disj, Arrow, Cardinality, Instantiation)
- `AtomPayload`: `UnresolvedIdentifier(String) | ResolvedIdentifier(DeclarationId) | TypeParam(String) | Literal(LiteralBits)` — phase coproduct dissolved in the last round
- `ArrowBody`: `UserDefined(NodeId) | ExternalRealization(DeclarationId) | Pending` — 3 variants, Pending restricted (per thesis §Q7) to primitive realization lag
- `CardinalityBound`: `Exact(u32) | AtMostOne | Unbounded`
- `Declaration`: `meta_tag` + `inhabits` split (value-construction vs algebra-inhabitance)
- `DeclarationId` + `NodeId` distinct newtypes
- Five L1 behaviors unchanged since M0 (Value, Transform, Branch, Loop, Bind)

**Test status:** 51 green (40 M0 acceptance + 4 M1 substrate + 7 real_stdlib
parse smoke), clippy clean, all real `dsl/std/*.dag` bootstrap files
parsing.

**Known open structural concerns** from the latest review rounds (6 and 7):

1. Operator dispatch threads through `UnresolvedIdentifier("+")` — reuses
   the unresolved-identifier shape as an operator-token sentinel
2. `lower.rs` descent check still does `target == "-"` string match
3. Operator knowledge scattered across `operators.rs`, `infer::is_comparison_operator`,
   `lower::is_strictly_smaller`
4. `TransformNode.target: DeclarationId` has no type-level distinction
   between callable Arrow, operator token, and placeholder
5. **`lower::lower_type_for_port` string-matches `"Int"|"Bool"|"String"`** —
   same name-dispatch bridge as #1, relocated to a different function
6. **`resolve_pending_identifiers` silently skips block-body `Fn` items** —
   fail-closed discipline violation AND catalogue violation (post-sweep
   `UnresolvedIdentifier` should be unreachable)

Rounds 1–5 resolved earlier instances of the same root cause. Rounds 6 and
7 are open. The §3 Design Oracle below pins the answers that will close
them structurally rather than by another round of patching.

---

## §3. Design Oracle — pin these before §4 starts

This section answers four substrate-design questions before any consumer
row in §4 can be closed. The postmortem on `free-cod-972` concluded that
reviewer rounds kept discovering new bridges because each round was an
implementation sketch exerting gravity on design, rather than an oracle
committed to in advance. The cost of pinning answers here is low; the
cost of unpinning them later is the pattern that produced this document.

A Design Oracle section is load-bearing BEFORE the consumer-enumeration
§4 because §4's S/R/N classifications depend on §3's answers. Without
§3, the same row could be "S" under one oracle and "R" under another,
and reviewers cannot tell which oracle the plan commits to.

### §3.1 Authority model

For every structural fact the compiler reasons about, exactly one place
is the authoritative producer. Downstream stages read from that
authority — they do NOT re-derive from another fact.

| Fact | Authority | Downstream must read via |
|---|---|---|
| Primitive types (`Int`, `Bool`, `Char`, `Byte`, ...) | `dsl/std/*.dag` bootstrap; `declaration_by_name` resolves the canonical name ONCE at bootstrap time. | `ResolvedIdentifier(DeclarationId)` — never the name. |
| What counts as a top-level declaration | `dag.declarations()`. A top-level declaration has no parent. Type parameters and sum variants are NOT top-level and do not appear here. | `dag.declarations()` iteration or `declaration_by_name` — both filtered to top-level by construction, not by caller discipline. |
| Template parameters | The owning `Declaration`'s `type_params: Vec<TypeParamId>`. `TypeParamId` is a distinct newtype. | `SubstStack` for substitution; the owning declaration's `type_params` slot for enumeration. Never via `declaration_by_name`. |
| Sum variants | The parent Disj `Declaration`'s `variants: Vec<VariantId>`. Variants are NOT top-level. | Pattern-match code reads `decl.variants` on the match subject's type. Never resolved globally. |
| Operator dispatch (`+`, `<`, ...) | `operators.rs` from PR-A onwards. The `operators` module is the ONLY place that maps symbols to `OperatorKind`. | Parser calls `operators::from_symbol` at parse time; downstream only sees `TransformTarget::Operator(OperatorKind)`. Grep-enforced in §8. |
| Primitive operator realizations (`Int.add → i64::wrapping_add`) | `dsl/extdeps/languages/<target>.dag`. No Rust file manufactures realizations. | `emit` and `lens_cost` both read the same language-spec declaration. PR-B closes this. |
| Descent evidence (is this call strictly smaller?) | Set at lowering time on `TransformNode.descent: DescentEvidence`. Lowering is the only stage with operator kind + literal + argument shapes simultaneously in scope. | `lens_termination` and future bounded-recursion lens. Never re-walked from SurfaceExpr. |
| Diagnostics (what went wrong and where) | `dag.diagnostics()`. Every error path attaches there. | CLI, tests, any stage that needs to fail-closed. Produced at point of discovery, not inferred from downstream state. |
| Sub-value / descent structure | Parser produces the field-binding list; lower records it on the parent `Declaration`. | Descent termination lens, `infer::classify_let_value`. Never inferred from names matching "child" patterns. |
| Identifier resolution phase | `resolve_pending_identifiers` is the SINGLE writer of the `UnresolvedIdentifier → ResolvedIdentifier` transition. | Every read after the sweep sees `ResolvedIdentifier` or a diagnostic. Post-sweep `UnresolvedIdentifier` IS a diagnostic-bearing error state, not a valid live-path shape. |

**Negative authorities (do NOT read from these):**

- `decl.name` is NOT the authority on anything semantic. Names are for
  diagnostics and authored-source traceability. Any code that branches
  on `decl.name == "Int"` or `decl.name.starts_with("...")` or similar
  is a bridge and must land in the §3.3 catalogue as an R row.
- `connective` variant alone is NOT the authority on behavior family.
  Arrow-with-Pending-body is not the same thing as Arrow-with-UserDefined-
  body; consumers must read `ArrowBody`, not just "is this an Arrow."
- Post-sweep presence of `UnresolvedIdentifier` is NOT a valid production
  read — it is an error state that should be unreachable by construction.
  The test suite must prove it is unreachable.

### §3.2 Scope and visibility

Top-level declarations, type parameters, sum variants, and sub-values
live in different scopes. The substrate must make it impossible to
confuse them. **Commitment: typed ID newtypes — no raw `usize` indices,
no `Vec<Declaration>` that mixes top-level with children.**

- `DeclarationId`, `NodeId`, `PortId`, `TypeParamId`, `VariantId`,
  `DiagnosticId` — six distinct newtypes, each allocated from a distinct
  counter. `DeclarationId::from(usize)` is private to `dag.rs`.
- `dag.declarations(): &[Declaration]` returns only top-level.
- Type parameters live in `decl.type_params: Vec<TypeParamId>` pointing
  at a separate `type_params` table. Variants live in `decl.variants:
  Vec<VariantId>` pointing at a separate `variants` table.
- `declaration_by_name(name)` first-match searches `declarations()` —
  impossible to return a type param or variant by construction.
- `dag.type_param(TypeParamId)`, `dag.variant(VariantId)` have distinct
  getters. Cross-table lookup is impossible by type.
- Any "find X by string" must explicitly pick the table. String →
  `TypeParamId` must go through the owning declaration's `type_params`
  slot, not a global lookup.

**Alternative considered and rejected: parent edge + filtered global
lookup.** Keeping everything in one `Vec<Declaration>` with
`parent: Option<Id>` and filtering on read is what led to the round-5
bug where a global name lookup could return a type parameter from an
unrelated declaration. Type-level separation removes the foot-gun
entirely; the filter-on-read approach requires every caller to remember
the filter.

**Commitment to scope discipline in PR-A:**

- `dag.rs` introduces any newtypes not already present (`TypeParamId`,
  `VariantId`) and separate tables.
- `declarations()` returns top-level only. Internal storage can stay
  unified if needed, but the public iterator is top-level only.
- Code review gate: any new call to `declaration_by_name` must be
  justified inline (one-line comment naming why global-lookup
  semantics are correct), because first-match is load-bearing and
  easy to misuse.

### §3.3 Invalid-state catalogue

The substrate must make the following states unrepresentable by type.
Where a state is currently representable, the plan names the PR that
eliminates it.

| # | Invalid state | Representable today? | Eliminated by |
|---|---|---|---|
| 1 | `TemplateArgument { parameter: p, value: p }` (self-reference stub) | Yes — `lower::build_template_arguments` has the stub path | PR-A: parser/lower emit resolved arguments from the start; stub path deleted |
| 2 | Post-sweep `AtomPayload::UnresolvedIdentifier(name)` reachable from a well-formed program | Yes — `resolve_pending_identifiers` skips operator names AND block-body `Fn` items | PR-A: operators never become `UnresolvedIdentifier`; block-body `Fn` silent-skip removed |
| 3 | `ArrowBody::Pending` reachable at emission time | Yes — bootstrap writes `Pending` and relies on post-pass to patch | PR-B: §8.11 ratchet hits zero in the same commit that adds the dissolution mechanism |
| 4 | Type parameter appearing in `dag.declarations()` top-level iteration | Yes — round-5 bug was an instance | PR-A: §3.2 typed-ID table separation |
| 5 | Sum variant resolved via `declaration_by_name` | Yes — variants share the top-level name space with declarations | PR-A: separate `variants` table per §3.2 |
| 6 | `TransformTarget` distinguishing callable vs operator via `UnresolvedIdentifier(name)` | Yes — current `free-cod-972` state | PR-A: `TransformTarget` enum |
| 7 | `DescentEvidence` re-derivable by string-matching `target == "-"` | Yes — current `is_strictly_smaller` | PR-A: descent classified at lowering time, stored on `TransformNode` |
| 8 | `lower_type_for_port` accepting only `"Int"\|"Bool"\|"String"` via whitelist | Yes — round-7 finding | PR-A: delete the whitelist; route through `resolve_type_expr` like every other type reference |
| 9 | Block-body `Fn` item silently skipped by `resolve_pending_identifiers` with no diagnostic | Yes — round-7 finding | PR-A: remove silent skip; emit diagnostic if not yet supported, or fully lower it |
| 10 | `Declaration.inhabits: Some(id)` where `id` doesn't exist or isn't an algebra | Yes — no well-formedness check | Deferred to M1(3)+; captured here for tracking |
| 11 | `Declaration.meta_tag: Some(id)` where `id` is not itself a meta-type | Yes — no structural marker for "this IS a meta-type" | Deferred to M1(5)+; captured here for tracking |
| 12 | Operator dispatch before parse completes | Yes by construction today (name dispatch is stage-agnostic) | PR-A: operators are classified at parse time; post-parse everything is typed |

Rows 1–9 are actionable in this plan and block PR-A merge. Rows 10–12
are tracked as future substrate work; the catalogue exists so that
when a downstream consumer needs them, the need does not arrive as a
surprise.

### §3.4 Boundary fact flow

The §4 tables below include a **Producer** column naming the stage that
authoritatively writes each fact. For S rows, the producer is the
canonical path. For R rows, the producer is where the fact CAN be
written per §3.1's authority model — the current code either drops it
or reconstructs it downstream. For N rows, the producer is the stage
that WILL write it when the consumer lands.

Boundary-flow vocabulary used in the Producer column:

- `parse` — produces raw surface forms + span metadata
- `lower(collect)` — `collect_symbols_phase` allocates top-level decls + type params
- `lower(bodies)` — `lower_bodies_phase` fills bodies, resolves in-file references
- `resolve_sweep` — `resolve_pending_identifiers` does cross-file resolution
- `bootstrap` — `Dag::new` via `dsl/std/*.dag`
- `lang_spec` — `dsl/extdeps/languages/*.dag` (future, PR-B)

**Fact-drop discipline:** if an upstream stage has a fact in hand and
no downstream consumer has committed to reading it YET, the fact goes
on the substrate ANYWAY — with the consumer row pre-enumerated as N.
Dropping a known fact and re-deriving it later is a bridge; this
enumeration catches it before it happens. The Producer column is the
forcing function: if a row has a Producer but no consumer reads it
downstream, that is a fact on its way to becoming dead weight; if a row
has a consumer but no Producer, that is the R-row pattern.

---

## §4. Downstream consumer enumeration

For each consumer that reads the substrate, list every structural question
it asks. Per §3.4, each row names the Producer and its status:

- **S** = answered structurally by current substrate (Producer writes it; consumer reads it)
- **R** = currently reconstructed (bridge risk; Producer has the fact or can produce it, consumer reconstructs instead)
- **N** = not yet asked (future work; Producer is pre-committed even if no consumer exists yet)

### §4.1 `lens_provenance.rs` (76 lines, working)

| # | Question | Producer | Status |
|---|---|---|---|
| 1 | What is a Port's `produced_by` NodeId? | `lower(bodies)` | S |
| 2 | What Behavior variant is the producer? | `lower(bodies)` | S |
| 3 | Is there a producer at all (leaf port)? | `lower(bodies)` | S |

All S. Zero reconstruction. v3 success-bar proof point #1.

### §4.2 `lens_depth.rs` (74 lines, working)

| # | Question | Producer | Status |
|---|---|---|---|
| 1 | What is a Port's `produced_by` NodeId? | `lower(bodies)` | S |
| 2 | What Behavior variant is a Node? | `lower(bodies)` | S |
| 3 | For Transform: input Ports? | `lower(bodies)` | S |
| 4 | For Branch: condition Port + path output Ports? | `lower(bodies)` | S |
| 5 | For Loop: source + init Ports? | `lower(bodies)` | S |
| 6 | For Bind: value Port? | `lower(bodies)` | S |

All S. Zero reconstruction. v3 success-bar proof point #2.

### §4.3 `infer.rs` (653 lines, mostly working — 3 open R-rows)

| # | Question | Producer | Status | Notes |
|---|---|---|---|---|
| 1 | What's a Port's state (Uninferred/Resolved/Unresolved)? | `lower(bodies)` + `infer` | S | `Port::state()` |
| 2 | What's a Node's Behavior variant? | `lower(bodies)` | S | `dag.node()` pattern match |
| 3 | What's a Declaration's TypeConnective? | `lower(collect)` + `lower(bodies)` | S | `decl.connective` pattern match |
| 4 | For Arrow: inputs/output? | `lower(bodies)` | S | `TypeConnective::Arrow { .. }` |
| 5 | For Arrow: body kind? | `lower(bodies)` / `bootstrap` | S | `ArrowBody` 3 variants |
| 6 | Is a Bind's value Port resolved? | `infer` | S | Port state check |
| 7 | **Is a Transform target callable, operator, or unresolved?** | SHOULD be `parse` (PR-A) | **R** | Currently reconstructed via `unresolved_operator_name(decl)` — string match on `UnresolvedIdentifier.name` against `OPERATOR_FIELD_MAP`. Producer authority moves to `parse` in PR-A. |
| 8 | **What kind of operator is this (arithmetic vs comparison)?** | SHOULD be `parse` (PR-A) | **R** | Reconstructed via `is_comparison_operator(name)` — separate string match. PR-A makes this a field on `OperatorKind`. |
| 9 | **What's the operator's return type semantics (returns T or Bool)?** | SHOULD be `operators.rs` (PR-A) | **R** | Hard-coded dispatch: comparison → Bool, else → T from inhabitance walk. PR-A derives from `OperatorKind::signature`. |
| 10 | What DeclarationId does an Identifier resolve to? | `resolve_sweep` | S | `ResolvedIdentifier(DeclarationId)` carries it |
| 11 | What does a DeclarationId map to at port-level? | `infer::walk_to_type_shape` over `decl.connective` | S | Returns `TypeShape::new(current)` for named decls |
| 12 | For Instantiation: template + arguments? | `lower(bodies)` | S | `Instantiation { template, arguments }` |
| 13 | Walk substitution context? | `infer` | S | `SubstStack` |

**Three R-rows** — all related to operator dispatch. Producer for each
moves from "post-parse reconstruction" to "parse-time classification via
`operators.rs`" in PR-A.

### §4.4 Descent evidence (currently in `lower.rs`, `is_strictly_smaller`)

| # | Question | Producer | Status | Notes |
|---|---|---|---|---|
| 1 | **Is this call operating on a structurally smaller sub-value?** | SHOULD be `lower(bodies)` | **R** | Walks SurfaceExpr, matches `target == "-"` + constant pattern. Producer authority is `lower(bodies)` per §3.1 — lowering is the only stage with operator kind + constant literal + argument shapes simultaneously in scope. |
| 2 | **What's the "smaller-by-1" pattern for each operator?** | SHOULD be `operators.rs` (PR-A) | **R** | Hard-coded for subtraction only; lexicographic / structural descent not supported. PR-A moves to `operators::descent_shrink(kind)`. |

**Two R-rows** — descent evidence reconstruction. Facts known at parse/lower
time (the typed operator kind + the constant literal), dropped at lowering,
reconstructed by re-reading SurfaceExpr.

### §4.5 `lower::lower_type_for_port` (round-7 finding)

| # | Question | Producer | Status | Notes |
|---|---|---|---|---|
| 1 | **What declaration does a port's type name reference?** | SHOULD be `lower(bodies)` via `resolve_type_expr` | **R** | Current code string-matches `"Int"\|"Bool"\|"String"` and falls back to a placeholder for any other name. This is the same name-dispatch bridge as §4.3's rows 7–9, relocated to a different function. Round-5 "eliminate all name bridges" was incomplete because it only touched type-shape collapse; `lower_type_for_port` escaped the sweep. |

**One R-row.** PR-A deletes the whitelist and routes every port type
reference through `resolve_type_expr` — the same path every other type
reference in the compiler uses. Grep gate in §8 prevents reintroduction.

### §4.6 `resolve_pending_identifiers` Fn silent-skip (round-7 finding)

| # | Question | Producer | Status | Notes |
|---|---|---|---|---|
| 1 | **What happens when an unresolved identifier names a `Fn` item appearing in a block body?** | SHOULD be `resolve_sweep` | **R** | Currently silently skipped — the sweep emits no diagnostic, the identifier persists as `UnresolvedIdentifier` past the sweep, violating §3.3 row 2 (post-sweep `UnresolvedIdentifier` should be unreachable). Fail-closed invariant violation per `feedback_fail_closed_discipline.md` — every detectable problem must produce a Diagnostic. |

**One R-row.** PR-A removes the silent skip. Two valid fix shapes:

1. **If block-body `Fn` is not yet supported:** emit a diagnostic
   ("block-body function declarations are not yet implemented"),
   attach to the sweep stage, fail-closed at the lowering boundary.
2. **If block-body `Fn` is supposed to resolve:** implement the
   lowering path (lift the Fn to a top-level declaration with a
   synthetic scope marker, or add a parent-edge + scoped lookup
   table consistent with §3.2).

PR-A picks (1) unless the implementer verifies (2) is tractable within
PR-A's scope. Either way, the silent-skip path is deleted in PR-A.

### §4.7 `lens_cost.rs` (future, M1(3))

| # | Question | Producer | Status | Notes |
|---|---|---|---|---|
| 1 | What's a Node's per-op cost? | SHOULD be `lang_spec` (PR-B) | N | Needs cost-per-primitive declared in `rust.dag` |
| 2 | What's a Transform's target cost? | SHOULD be `operators.rs` (PR-A) + `lang_spec` (PR-B) | N | Depends on typed operator kind (§4.3 rows 7–9) |
| 3 | How do costs compose across Behavior kinds? | `lens_cost` itself | N | Sequence / Loop / Branch composition rules |
| 4 | **Where does the lens STORE its results?** | `lens_cost` substrate decision | N | Deferred from M0 — first writer lens forces the decision |
| 5 | For ExternalRealization: target-world cost? | `lang_spec` (PR-B) | N | Blocked on language spec declaring realization costs per primitive |

**Five N-rows.** Rows 1–3 are blocked on §4.3 R-rows being closed.
Row 4 is a substrate decision that applies to every future writer lens.

### §4.8 Rust emitter skeleton (future, M1(4))

| # | Question | Producer | Status | Notes |
|---|---|---|---|---|
| 1 | How does each TypeConnective project to Rust syntax? | `lang_spec` (PR-B) — `dsl/extdeps/languages/rust.dag` | N | Reads language spec declaratively |
| 2 | For Arrow UserDefined: emit sub-DAG as Rust fn body | `emit` consumes substrate | N | Walks computation substrate |
| 3 | For Arrow ExternalRealization: emit target-language binding | `lang_spec` (PR-B) | N | Reads realization declaration |
| 4 | For Arrow Pending: fail-closed | `emit` (invariant) | N | Enforced by §8.11 ratchet; must not reach emission |
| 5 | For Instantiation: substitute template args, emit specialization | `emit` + `SubstStack` | N | Lazy substitution via SubstStack |
| 6 | Ownership: which fields get `Rc`, which get moves? | Ownership lens (M1(5)+) | N | Orthogonal — different lens, different milestone |

**Six N-rows.** Rows 1, 3 depend on §4.3 R-rows and on parser support for
`realization { ... }` record literals. Row 6 is orthogonal (different
lens, different milestone).

### §4.9 Interpreter (future, later)

| # | Question | Producer | Status | Notes |
|---|---|---|---|---|
| 1 | For each Node: evaluate its value given input Port bindings | `interp` consumes substrate | N | Tree walker over L1 behaviors |
| 2 | For Transform over primitive: call host primitive | `lang_spec` (PR-B) | N | Reads ExternalRealization for runtime binding |
| 3 | For Transform over user function: recursively evaluate sub-DAG | `interp` consumes substrate | N | Same substrate as emission |
| 4 | Termination: is the current walk bounded? | `lower(bodies)` via `DescentEvidence` | N | Reads descent evidence (blocked on §4.4) |

**Four N-rows.** All depend on §4.3 + §4.4 R-rows being closed.

---

## §5. Gap analysis

**Open R-rows (reconstructive facts currently in the codebase):** 7

1. Transform target kind (callable/operator/unresolved) — §4.3
2. Operator kind (arithmetic/comparison) — §4.3
3. Operator return type semantics — §4.3
4. "Is this a smaller sub-value?" (descent evidence) — §4.4
5. "What's the smaller-by-1 pattern?" (descent shrink factor) — §4.4
6. **Port-type name whitelist (`Int|Bool|String`)** — §4.5
7. **Block-body `Fn` silent-skip** — §4.6

R-rows 6 and 7 were discovered in round 7 after the first draft of this
plan was written; they are named explicitly because their discovery
during planning is live proof that the original enumeration was
incomplete — see `feedback_enumerate_before_substrate.md`. The plan
is stronger for catching them in design rather than in review.

**Open N-rows (future consumer questions):** 15 (5 cost, 6 emit, 4 interp)

**Dependency analysis of N-rows on R-rows:**

- All 5 cost lens questions depend on R-rows 1–3 (cost lookup needs typed
  operator kinds; target cost needs typed call kind)
- Emit rows 1, 3 depend on R-rows 1–3 (typed dispatch at emission)
- All 4 interpreter questions depend on R-rows 1–5 (interpreting operators
  + descent)
- R-rows 6 and 7 are orthogonal to the N-rows but load-bearing for the
  "post-sweep `UnresolvedIdentifier` is unreachable" invariant (§3.3
  row 2), which every downstream consumer assumes

**Conclusion:** closing the 7 R-rows in one PR (PR-A below) unblocks 11 of
15 future consumer questions AND eliminates the catalogue violations
that would otherwise force every downstream stage to carry defensive
"did the sweep actually resolve this?" logic. The remaining 4 N-rows
(cost lens storage mechanism, emission ownership, emission language
spec reading, interpreter) are independent substrate additions addressed
in PR-B.

---

## §6. Atomic work units

Two large PRs. Each updates substrate + every affected consumer
simultaneously. No intermediate states, no bridges.

### §6.1 PR-A: Structural operator handling + round-7 cleanup (est. 10–14 hours)

**Purpose:** close all 7 R-rows by lifting operator knowledge out of
string-matching and descent-evidence reconstruction into structural types.
Consolidate operator knowledge into a single authority. Eliminate the
two round-7 name-dispatch / silent-skip bridges in the same push.

**Substrate changes (`dag.rs`):**

```rust
pub struct TransformNode {
    pub id: NodeId,
    pub target: TransformTarget,           // was DeclarationId
    pub descent: DescentEvidence,          // new structural field
    pub inputs: Vec<PortId>,
    pub output: PortId,
    pub span: SourceSpan,
}

pub enum TransformTarget {
    Callable(DeclarationId),
    Operator(OperatorKind),
    Unresolved(String, SourceSpan),        // must have diagnostic attached
}

pub enum OperatorKind {
    Arith(ArithOp),
    Cmp(CmpOp),
}
pub enum ArithOp { Add, Sub, Mul, Div }
pub enum CmpOp { Eq, Ne, Lt, Le, Gt, Ge }

pub enum DescentEvidence {
    StrictSubValue { parameter: String, shrink: ShrinkFactor },
    PreservedValue { parameter: String },
    Unrelated,
}
pub enum ShrinkFactor { Constant(u32), Structural }
```

**Scope-and-visibility scaffolding (§3.2):**

If `TypeParamId` / `VariantId` are not already distinct newtypes in
`dag.rs`, PR-A introduces them as part of the atomic push. The goal is
that `dag.declarations()` return top-level only and that
`declaration_by_name` cannot possibly return a type param or variant
by construction.

**`operators.rs` becomes the single authority:**

```rust
pub fn from_symbol(s: &str) -> Option<OperatorKind> { ... }
pub fn signature(kind: OperatorKind) -> OperatorSignature { ... }
pub fn descent_shrink(kind: OperatorKind) -> Option<ShrinkFactor> { ... }
pub fn primitive_cost(kind: OperatorKind) -> CostUnit { ... }
```

(If implementation reveals all four functions are field lookups on a
single `OperatorSpec` record, collapse to `pub const OPERATOR_SPECS:
&[OperatorSpec]`. See collapse opportunity #3 below.)

**Round-7 cleanup:**

- `lower::lower_type_for_port` whitelist deleted. Port type references
  route through the same `resolve_type_expr` path the rest of the
  compiler uses. If `resolve_type_expr` does not yet exist as the
  unified entry point, PR-A introduces it (this is the same change as
  "every type reference has one authoritative resolver").
- `resolve_pending_identifiers` block-body `Fn` silent-skip deleted.
  Emits a diagnostic if block-body `Fn` is not yet supported, OR fully
  lowers it. See §4.6 for the two valid fix shapes.

**Files touched:**

- `parse.rs` — parser emits typed operator kinds at parse time via
  `operators::from_symbol`
- `lower.rs` — `TransformTarget` dispatch; descent evidence classification
  at lowering time; deletes `is_strictly_smaller`; deletes
  `lower_type_for_port` whitelist; routes port types through
  `resolve_type_expr`
- `infer.rs` — dispatches on `TransformTarget` variants structurally;
  deletes `unresolved_operator_name`, `is_comparison_operator`
- `operators.rs` — becomes single authority for symbol/kind/signature/
  cost/descent
- `dag.rs` — adds `TransformTarget`, `OperatorKind`, `DescentEvidence`;
  refactors `TransformNode`; adds `TypeParamId`/`VariantId` if missing
- `bootstrap.rs` — removes any remaining name-dispatch that reads
  `decl.name` for semantic purposes (should be zero post-PR-A)
- `m0_acceptance.rs` — updates test helpers; recursive descent tests read
  `DescentEvidence` structurally
- `m1_substrate_test.rs` — new tests:
  - new operator can be added by editing ONLY `operators.rs`
  - port-type resolution accepts any user-defined type, not just primitives
  - block-body `Fn` either lowers correctly OR emits a diagnostic (no silent skip)

**Deletions:**

- `infer::unresolved_operator_name`
- `infer::is_comparison_operator`
- `operators::is_operator_name` (operators are structural post-parse, not
  name-matched)
- `lower::is_strictly_smaller`
- `lower::lower_type_for_port` whitelist branch (replaced with single-path resolution)
- `resolve_pending_identifiers` operator-skip branch
- `resolve_pending_identifiers` block-body `Fn` silent-skip branch

**Horizontal collapse opportunities (5):**

1. **Three dispatch paths become one.** Currently callable goes through
   `decide_transform` + Arrow walk, operator goes through
   `unresolved_operator_name` + `OPERATOR_FIELD_MAP` + `is_comparison_operator`,
   unresolved goes through sweep-error. After: one match on `TransformTarget`
   in `decide_transform`. Every consumer gets the same structural shape.

2. **Descent evidence is no longer operator-special.** `DescentEvidence`
   is a general structural fact on every Transform. Adding lexicographic
   descent or structural descent over children lists means adding a variant
   — no new string matches, no new reconstruction.

3. **`operators.rs` may collapse to declarative data.** If `signature`,
   `descent_shrink`, and `primitive_cost` are all field lookups on a
   single `OperatorSpec` record, the module becomes
   `pub const OPERATOR_SPECS: &[OperatorSpec]` — pure data, not scattered
   functions. This collapse is not visible in the enumeration but emerges
   when PR-A writes the three functions side by side.

4. **Every type reference goes through `resolve_type_expr`.**
   `lower_type_for_port` whitelist deletion isn't a local fix — it's an
   instance of the broader "every type reference uses the same resolver"
   pattern. The same path handles Arrow input/output, Bind value types,
   Port types, and Instantiation arguments. Before PR-A: four call sites,
   one with a three-name whitelist. After PR-A: one entry point, no
   whitelist, consistent error reporting for every type-reference context.

5. **`resolve_pending_identifiers` simplifies twice over.** Currently two
   skip branches (operator-name and block-body `Fn`). After PR-A: both
   deletions land in the same commit, and the sweep body shrinks to
   "resolve every pending identifier, fail-closed on non-matches." This
   is the single largest structural simplification in the file and
   removes the two catalogue violations that made post-sweep
   `UnresolvedIdentifier` representable.

**Acceptance gates (in addition to §8 universal gates):**

- `grep -E "target == \"|name == \"|is_operator_name|unresolved_operator_name|is_comparison_operator|is_strictly_smaller" src/v3/compiler/src/` → zero matches
- `grep -E '"Int" =>|"Bool" =>|"String" =>' src/v3/compiler/src/lower.rs` → zero matches (whitelist deletion gate)
- `grep -E 'Fn.*skip|skip.*Fn' src/v3/compiler/src/lower.rs` → zero matches (silent-skip gate)
- `grep 'OPERATOR_FIELD_MAP' src/` → matches only inside `operators.rs`
- `TransformTarget` has exactly 3 variants (compile-time assertion)
- `OperatorKind` has exactly 2 variants; `ArithOp` has 4; `CmpOp` has 6
  (compile-time assertions)
- New test: adding a new operator symbol to the enum updates ONE file
  (`operators.rs`); parser/lowerer/infer/descent all read the new variant
  structurally without edits
- New test: port type references accept any user-defined type (at least
  one test case uses a std-library type that is NOT Int/Bool/String)
- New test: block-body `Fn` produces either a correctly-lowered Declaration
  OR a diagnostic (no silent-skip path remains)

**Closes:** R-rows 1, 2, 3, 4, 5, 6, 7 (all 7 open reconstructive facts).

### §6.2 PR-B: M1(3) + M1(4) — cost lens + Rust emitter (est. 12–18 hours)

**Purpose:** first writer lens, first real target-language emission,
parser support for `realization` literals, `ArrowBody::Pending` dissolution
— all as one atomic unit because they share reading patterns and
interact.

**Substrate additions:**

- Lens storage mechanism (deferred from M0 retrospective — first writer
  lens forces the decision). Options: per-lens side tables vs `Dag::lens_results:
  HashMap<LensId, Box<dyn LensResult>>` vs annotations map on Port. PR-B
  picks one and all subsequent writer lenses follow.
- Parser support in `parse.rs` for `realization { for: X.add; target:
  rust; body: "i64::wrapping_add" }` item syntax (the one remaining
  parser gap identified in M1_FOLLOWUPS.md)

**New files:**

- `src/v3/compiler/src/lens_cost.rs` — first writer lens; reads substrate
  + OperatorKind + ArrowBody; produces per-Node cost
- `src/v3/compiler/src/emit.rs` — walks Dag via language spec; produces
  Rust source
- `dsl/extdeps/languages/rust.dag` — first real language spec; declares
  Rust primitive realizations; replaces `inject_realization_stub`

**Files touched:**

- `parse.rs` — adds `realization` item parsing + record-literal body
- `bootstrap.rs` — deletes `inject_realization_stub`; parses
  `dsl/extdeps/languages/rust.dag` as the 8th bootstrap file
- Tests: new `smoke_compile_and_run` takes a user program through
  parse → lower → infer → lens_cost → emit → `cargo check` → run

**Horizontal collapse opportunities (5):**

1. **Cost lens and emitter share a `StructuralVisitor` trait.** Both
   walk `TypeConnective` variants, both dispatch on `ArrowBody`, both
   consume `OperatorKind` from PR-A. Writing them together reveals the
   shared "walk a Dag, produce a per-node output" pattern. Writing them
   separately would produce two walkers and notice the duplication
   later.

2. **The language spec IS the unified cost + realization source.** The
   `rust.dag` declaration for `Int.add` contains both "realizes as
   `i64::wrapping_add`" (emission) and "costs 1 machine instruction"
   (cost analysis). Both consumers read the SAME declaration. Building
   cost lens first with a temporary cost table and then realizing the
   emitter should read the same table is exactly the bridge pattern
   the invariants forbid.

3. **`ArrowBody::Pending` eliminates in the same PR that introduces the
   mechanism to populate `ExternalRealization`.** Before: bootstrap
   initializes with Pending, fixed up later. After: bootstrap parses
   `rust.dag`, primitive Arrows land with `ExternalRealization(decl_id)`
   from the start. Pending is never written to production declarations.
   §8.11 ratchet reaches zero in the same commit that introduces the
   dissolution mechanism. This is the cleanest possible scaffolded-state
   dissolution.

4. **`inject_realization_stub` deletion is the natural falling-out.** The
   stub exists because parse couldn't handle `realization` literal syntax.
   PR-B adds the parser support AND deletes the stub. Before → after:
   compiler bootstrap has zero Rust-manufactured declarations in
   production paths.

5. **Lens storage decision applies to every future writer lens.** Cost
   is the first; ownership, effects, purity, space-bounds will all
   follow the same pattern. Getting storage right in PR-B means PR-C
   through PR-N have zero substrate impact — the v3 thesis's "new lens
   = new file, zero substrate edits" success bar becomes provable by
   construction.

**Acceptance gates (in addition to §8 universal gates):**

- End-to-end: `compile("let x: Int = 1 + 2").emit_rust()` produces Rust
  source that compiles under `cargo check` and runs returning `3`
- `grep "inject_realization_stub\|Pending" src/v3/compiler/src/` → only
  historical / doc references, zero active code paths
- `lens_cost.rs` is < 250 lines (success-bar budget)
- `grep "\"i64::\\|quote!\\|\"Rust " src/v3/compiler/src/emit.rs` → zero
  matches (the emitter has no hardcoded target knowledge; it reads
  language spec declarations)
- Cost lens + emit both go through a shared `StructuralVisitor`-like
  pattern (proven by refactoring, not mandated in advance — the PR may
  discover the right abstraction shape)

**Closes:** 11 of 15 N-rows (all 5 cost; 5 of 6 emit; last emit row is
ownership, deferred).

---

## §7. Why larger PRs here — rationale

This document defaults to two large PRs rather than the ~6 smaller ones a
conventional code review workflow would prefer. Three reasons specific to
gunbc:

1. **Bridges appear at layer boundaries, not inside single files.** Every
   reviewer round on `free-cod-972` found a bridge where one piece of
   code assumed an upstream stage represented a fact structurally while
   the upstream stage represented it by name/lifecycle. A smaller PR
   that lands ONE layer at a time ensures the author can't see the next
   layer's needs, so they invent a provisional shape and the next layer
   has to reconstruct. A larger PR that touches substrate + parser +
   lowerer + inference + consumers at once forces reconciling every
   layer's needs against a single design — exactly what prevents
   reconstructive patterns.

2. **Horizontal collapses require seeing related code simultaneously.**
   The `StructuralVisitor` trait collapse in PR-B, the `OperatorSpec`
   record collapse in PR-A, the `resolve_pending_identifiers`
   operator-skip removal, and the `TransformTarget::Unresolved`/
   `alloc_identifier_stub` simplification are all things a smaller PR
   would miss because the related code would be in separate branches
   or already merged. Each PR in this plan explicitly lists 5 collapse
   opportunities it enables. Missing horizontal collapses is worse
   than PR-review cost.

3. **Non-production repo tolerates the reviewability cost.** gunbc has
   no external users blocked by the review window, no CI/CD pressure
   to merge fast, no team coordination overhead that scales with PR
   count. The typical "small PR for reviewability" argument assumes
   review is the bottleneck; here structural debt propagation is the
   bottleneck, and larger atomic PRs are the response. Review cost
   scales linearly with change size; debt cost scales quadratically
   with time-to-remediation. Paying the linear cost is obviously
   correct.

**The rule for atomicity:** for a PR to be atomic under this plan,
every consumer that reads or writes the changed substrate must be
updated in the same PR. If that scope exceeds ~20 hours of work, the
representation change is the wrong size — split the representation
change, not the consumer updates.

---

## §8. Universal acceptance gates (every PR)

1. **All existing tests stay green** — 51 minimum, growing as PRs add
   their own tests.
2. **Clippy clean** — `cargo clippy -p v3-compiler --all-targets -- -D warnings`
3. **No-bridges audit** — grep for adapter-function name patterns:
   `grep -E "fn .*_to_.*|fn convert_.*|fn adapt_.*|fn bridge_.*" src/v3/compiler/src/`
   returns zero new matches (INVARIANTS.md §"No bridges")
4. **No-name-dispatch audit** — `grep -E 'target ==|name ==|\.name\(\) ==|"Int" =>|"Bool" =>|"String" =>' src/v3/compiler/src/infer.rs src/v3/compiler/src/lower.rs`
   returns zero matches (parser is exempt — raw input). Enforces that
   downstream consumers read structural facts, not names.
5. **No-deprecation audit** — `grep -E "TODO.*M[0-9]|scope-bound|dissolves in|_legacy|_v1|_v2" src/v3/compiler/src/`
   returns zero new matches (INVARIANTS.md §"No deprecations")
6. **Variant-count closure** — compile-time `const _ASSERT_*` match-
   exhaustiveness checks ensure no new enum variants were added
   silently. Any new variant requires explicit sign-off against the
   C1-class stop signal (INVARIANTS.md §"No short-term solutions"
   and THESIS.md §"The substrate").
7. **No-silent-skip audit** — `grep -E '// skip|continue.*unresolved|continue.*Fn' src/v3/compiler/src/lower.rs`
   returns zero new matches. Per `feedback_fail_closed_discipline.md`,
   every detectable problem must produce a diagnostic; silent skipping
   is an invariant violation.
8. **Authority-model audit** — any new code path that reads `decl.name`
   for semantic purposes (branching on name content) must be flagged
   in the PR description with explicit justification. Grep pattern:
   `grep -E 'decl\.name|\.name ==|\.name\.starts_with|\.name\.ends_with' src/v3/compiler/src/`
   — only diagnostic/display code is allowed.

These gates enforce invariants structurally rather than by convention.
Any PR failing one is not mergeable.

---

## §9. Non-goals (explicit scope exclusions)

Out of scope for M1(2.6) → M1(4):

- Ownership analysis / lattice-on-bindings (M1(5)+)
- Interpreter (deferred; revisit when omni-emission starts)
- Omni-emission Shape B user-program artifacts (YAML, Terraform,
  SPICE, docs) — M2+
- Unification (five behaviors as patterns over Node) — recorded as
  future candidate in THESIS.md, not committed
- Three-primitive reduction — recorded as future candidate, not
  committed
- `dsl/std/meta.dag` as a first-class mechanism — defer until second
  consumer appears
- Law verification (associativity, commutativity, etc.) — M1(5)+
  algebraic simplification lens
- Substrate-extension full CI audit — simple grep-based version lands
  with PR-A; full mechanism deferred to after PR-B
- v3-compiler in the CI workflow — tracked separately; lands as a small
  PR ASAP, not folded into this plan

---

## §10. How to use this document

**When starting a new M1 iteration:**

1. Read §2 to confirm the baseline is still accurate. If `free-cod-972`
   has moved beyond what §2 describes, update §2 first.
2. Read §3 (Design Oracle) and confirm every sub-oracle (authority,
   scope, invalid-state, boundary-flow) still holds. If any is
   contradicted by your upcoming change, §3 must be updated BEFORE
   implementation — per §12 process commitment.
3. Read §4 and identify which consumer you are about to touch. Every
   question that consumer asks must already be in §4 with status
   S or R. If you discover a new question not in §4, STOP — add the
   row (with status R if the code currently reconstructs it, or N
   if you're pre-enumerating for future work) and resolve it
   structurally before writing the consumer code.
4. Map your work to one of the PRs in §6. If your work does not fit
   an existing PR, either it belongs in a future M1(5+) that this
   document does not cover, or the document needs a new §6 entry
   before implementation begins.
5. Before merge, run every §8 universal gate plus the PR-specific
   gates listed in §6.

**When you find a reconstructed fact in the codebase:**

Per `INVARIANTS.md` §"No short-term solutions" escalation procedure:
stop, back up, assess the damage, raise it as alarming. **Do not**
silently work around it in your own code. Add it as an R-row in §4,
check it against §3.3 (invalid-state catalogue), and figure out whether
it needs to land in the current in-flight PR or a new one.

**When enumerating a new downstream consumer:**

1. Read its expected code path mentally or sketch it.
2. List every question it asks about Nodes, Declarations, Ports, or
   Behaviors.
3. For each question, name the authoritative Producer per §3.1.
4. Mark each S/R/N.
5. For each R: that's a bridge to eliminate now.
6. For each N: confirm the substrate will answer structurally when
   the consumer is written, or flag the gap as future substrate work.

The enumeration is the design phase. Writing code is the
implementation phase. Implementation must not create new R-rows; any
R-row discovered mid-implementation means the design phase missed
something and the enumeration needs an update before proceeding.

---

## §11. Open questions (to resolve before starting PR-A)

Small design decisions that would benefit from being pinned before
implementation begins:

1. **Should `TransformTarget::Unresolved` carry the failed name and span,
   or just a generic "unresolved" marker with the name living in the
   attached Diagnostic?** Trade-off: carrying the name makes
   diagnostics self-contained but couples `TransformTarget` to
   diagnostic text; not carrying it requires consumers to cross-
   reference the diagnostic table. Likely answer: carry it — the
   name is a structural fact, not just diagnostic metadata.

2. **Is `OperatorKind` the right split (Arith vs Cmp), or should it be
   flatter (10 variants directly)?** Trade-off: the hierarchical form
   lets some dispatch code match just on the category (`if matches!(k,
   OperatorKind::Cmp(_))`) without enumerating operators; the flat form
   is simpler but loses that. Likely answer: hierarchical — the Cmp
   family shares a return type `Bool`, the Arith family shares `T →
   T`, and that distinction is structurally real.

3. **Where does `DescentEvidence` live — on `TransformNode` directly, or
   on a separate analysis table?** On-node is more honest (facts flow
   forward on the node); side table is more flexible for late analysis.
   Likely answer: on-node, per §"facts flow forward" invariant. Cost
   is a small struct per Transform; storage overhead is negligible.

4. **What's the `CostUnit` type for primitive cost in PR-A's operator
   table?** A single `u64` cycle-count? A `{cycles, allocations,
   io}` record? Defer: `u64` is enough for PR-A's purposes, the real
   cost-algebra decision happens in PR-B's `lens_cost.rs`. PR-A just
   reserves a field.

5. **Does `resolve_type_expr` already exist as a unified entry point,
   or does PR-A need to introduce it?** If it exists, `lower_type_for_port`
   simply delegates to it. If not, PR-A's scope grows to include
   unifying every current type-reference path. Must be verified before
   PR-A starts.

These are intentionally left open — they get pinned in the PR itself,
not in this plan.

---

## §12. Process commitment — design oracle first

Every previous reviewer round on `free-cod-972` found bridges that the
implementer did not see coming. The pattern cause is not implementer
error — it is process: the implementer was operating from an
implementation sketch (partial type enum + partial consumer set) rather
than a design oracle (full authority model + full consumer set +
invalid-state catalogue + boundary-flow table). Sketches exert gravity
on implementation; oracles define implementation.

**This document is itself the design oracle for M1(2.6)+. §12 codifies
the process so the next iteration does not repeat the same pattern.**

### §12.1 Compiler changes require a design doc

Any PR that touches `src/v3/compiler/src/` and adds/modifies substrate,
adds/modifies a lens, or adds/modifies a parser/lowerer path requires
a design doc that meets the following criteria. The design doc can
be a new markdown file under `src/v3/`, a new subsection of this
document, or a PR description that covers every point below — but
the content must exist before implementation starts.

**The five design questions (all must be answered before implementation):**

1. **Authority:** for each fact you're introducing or touching, which
   stage is the authoritative producer? Who reads from it? This answer
   must be consistent with §3.1 or must explicitly update §3.1.

2. **Scope:** what's the scope of the fact (top-level, per-declaration,
   per-node, per-port)? Which ID newtype does it live under? This
   answer must be consistent with §3.2 or must explicitly update §3.2.

3. **Invalid states:** list every combination of fields that would be
   semantically invalid. For each, is it representable by type? If yes,
   why, and what's the catalogue row that will eliminate it? This
   answer must be consistent with §3.3 or must explicitly update §3.3.

4. **Boundary flow:** for each fact, which stage produces it, and which
   stages consume it? Are there any drops (fact known at stage A, not
   carried to stage B, reconstructed at stage C)? This answer must
   be consistent with §3.4 and must either have zero drops or justify
   each drop with a tracking row.

5. **Forcing-function test:** what's the minimal test that proves the
   substrate change works end-to-end? This test must be written BEFORE
   the substrate change lands, must fail beforehand, and must pass
   afterward with no additional scaffolding.

### §12.2 Invariant audit

Before implementation begins, the design doc must explicitly audit
against every invariant in `INVARIANTS.md`. For each invariant, state
one of:

- "This PR does not touch [invariant]." (trivially compatible)
- "This PR is compatible with [invariant] because [justification]." (requires reasoning)
- "This PR requires [invariant] to be updated because [reason]." (requires sign-off)

The audit lives in the design doc, not the PR description. It is a
design artifact, not a merge artifact.

### §12.3 Downstream consumer naming

Every design doc must explicitly name every downstream consumer of
the new substrate shape, organized by S/R/N per §4. A design doc that
lists fewer than 3 consumers should be regarded with suspicion — almost
every substrate change affects `infer`, at least one lens, and usually
a test helper; if the doc lists fewer than 3, the author probably
missed one.

### §12.4 Reviewer validation

The design doc must be reviewed BEFORE implementation by someone other
than the implementer. The reviewer's job is not to approve or block —
it is to answer four questions:

1. Is the authority model internally consistent?
2. Are any downstream consumers missing from the enumeration?
3. Are there invalid states in the catalogue that need different
   type shapes to become unrepresentable?
4. Does any fact flow have a drop that isn't justified?

If any answer is "no" or "unclear," the design doc gets another
iteration before any code is written. Per §7 rationale, the cost of
a design-doc round-trip is linear; the cost of a PR round-trip with
bridge discovery is quadratic. Design-doc iteration is always
cheaper.

### §12.5 Elevation path

If §12 proves useful beyond M1(2.6), these process commitments should
graduate to a standalone `PROCESS.md` at the repo root, or to a new
subsection of `INVARIANTS.md`. This document intentionally keeps them
scoped to M1(2.6)+ until the discipline has been exercised at least
twice end-to-end and shown to prevent round-trips. Early elevation is
premature abstraction; late elevation is acceptable.

---

## §13. References

- `THESIS.md` §"The substrate: two coordinated shapes"
- `THESIS.md` §"Two groundings: static validation vs efficient realization"
- `THESIS.md` §"Epistemic stacking: every concept grounds in primitives"
- `INVARIANTS.md` §"No short-term solutions (this is not a production codebase)"
- `INVARIANTS.md` §"No bridges"
- `INVARIANTS.md` §"No deprecations"
- `src/v3/M1_DESIGN.md` — the substrate shape spec that M1(2.5) executed against
- `src/v3/M0_RETROSPECTIVE.md` — M0 closing notes; history
- `feedback_enumerate_before_substrate.md` — the memory that this document
  operationalizes
- `feedback_fail_closed_discipline.md` — basis for §3.3 row 2 and §8 gate 7
