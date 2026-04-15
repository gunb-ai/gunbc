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

## §2. Baseline: `free-cod-972` current state (M1(2.7))

**Substrate shape** (after M1(2.7) structural fix PR):

- `TypeConnective`: 6 variants (Atom, Conj, Disj, Arrow, Cardinality, Instantiation)
- `AtomPayload`: `UnresolvedIdentifier(String) | ResolvedIdentifier(DeclarationId) | TypeParam(String) | Literal(LiteralBits)` — phase coproduct dissolved in M1(2.6)
- `ArrowBody`: `UserDefined(NodeId) | ExternalRealization(DeclarationId) | Pending | Unparsed(SourceSpan)` — **4 variants**. `Pending` is the bootstrap-realization scaffold (dissolves by M3 via §8.11 ratchet). `Unparsed(SourceSpan)` is the block-body surface-grammar scaffold added in M1(2.7) for `fn f(x) -> T { body }` forms the grammar can't yet parse (dissolves when M2 grammar adopts match/pipe/lambda).
- `TransformTarget`: **2 variants** — `Callable(DeclarationId) | Operator(crate::operators::OperatorKind)`. Added in M1(2.7). **No `Unresolved` variant** — unresolved names live in the `Declaration` connective as `UnresolvedIdentifier` and are authoritatively rewritten by the resolve sweep. The plan's earlier draft proposed a third `Unresolved` variant; the implementer's cleaner factoring dodges it.
- `OperatorKind`: `Arithmetic(ArithmeticOp) | Comparison(ComparisonOp)` — 10 total variants (4+6). Lives in `operators.rs`, which is a 120-line surface-syntax-to-typed-kind table with `from_symbol` + `symbol` + dissolution-receipt header. No semantic functions (no `signature`, no `primitive_cost`, no `descent_shrink`) — those belong to `std/algebra.dag` and `dsl/extdeps/languages/*.dag`.
- `CardinalityBound`: `Exact(u32) | AtMostOne | Unbounded`
- `Declaration`: `meta_tag` + `inhabits` split (value-construction vs algebra-inhabitance)
- `DeclarationId` + `NodeId` distinct newtypes; cached `int_shape()` / `bool_shape()` / `string_shape()` / `realization_meta_id()` getters on `Dag` (added M1(2.7), replaces name-keyed primitive lookups)
- Five L1 behaviors unchanged since M0 (Value, Transform, Branch, Loop, Bind)

**Test status:** **60 green** (1 lib realization smoke + 41 m0_acceptance
+ 11 m1_substrate + 7 real_stdlib_parse_smoke), clippy clean, all real
`dsl/std/*.dag` bootstrap files parsing. Grew from 51 → 60 in M1(2.7)
via 8 new `m17_*` tests covering each resolved structural gap.

**Structural concerns from rounds 6 and 7: all closed in M1(2.7).**
The six items the earlier draft of this plan enumerated as open
(operator dispatch via `UnresolvedIdentifier`, string-match descent,
scattered operator helpers, untyped `TransformNode.target`,
`lower_type_for_port` primitive whitelist, `resolve_pending_identifiers`
block-body `Fn` silent-skip) are all dissolved. Verification receipts
inline in the code at:
- `operators.rs` header — dissolution trigger for the whole enum
- `lower.rs:1079-1094` — `QW5 SINGLE AUTHORITY` receipt (port-type whitelist gone)
- `lower.rs:1266` — structural descent check receipt (no `target == "-"`)
- `infer.rs:456` — `OPERATOR_FIELD_MAP is gone` receipt

This document now functions as **design oracle + retrospective for
PR-A (done) + forward plan for PR-B (not started)**. §6.1 is a
retrospective; §6.2 remains forward-looking, revised for the
reader-only lens invariant (§12.6).

---

## §3. Design Oracle

This section answers four substrate-design questions that govern how
§4 classifies consumer rows. The postmortem on `free-cod-972` concluded
that reviewer rounds kept discovering new bridges because each round
was an implementation sketch exerting gravity on design, rather than
an oracle committed to in advance. The cost of pinning answers here
is low; the cost of unpinning them later is the pattern that produced
this document.

The Design Oracle is load-bearing for §4's S/R/N classifications —
without §3, the same row could be "S" under one oracle and "R" under
another. As of M1(2.7), most of §3's answers have been validated in
practice: some of the implementer's cleaner factoring choices
(dropping `TransformTarget::Unresolved`, dropping `DescentEvidence`)
actually updated the oracle rather than the other way around. That
loop is the process working as intended.

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
| Operator **surface syntax** (`+`, `<`, ...) | `operators.rs` from PR-A onwards. The `operators` module is the ONLY place that maps symbol strings to `OperatorKind`. It is a surface-syntax-to-typed-kind table, NOT a semantic authority. | Parser calls `operators::from_symbol` at parse time; downstream only sees `TransformTarget::Operator(OperatorKind)`. Grep-enforced in §8. |
| Operator **semantics** (signature, primitive cost, descent shrink) | At M1(2.7): signature is encoded structurally in the `OperatorKind` variant (`Arithmetic(_) → operand type`, `Comparison(_) → Bool`) via `infer::resolve_operator_arrow`. Primitive cost lives in `dsl/extdeps/languages/<target>.dag` realization declarations, read by `lens_cost` in PR-B. Descent shrink is hard-coded to `ArithmeticOp::Sub` at M1 scope; M2+ extensions move to algebra-declaration annotations. | `operators.rs` does NOT manufacture these facts. Dual authority with `std/algebra.dag` or `rust.dag` is forbidden — if you're tempted to put a `signature(kind)` function in `operators.rs`, that's the round-7 reviewer flag telling you a second semantic universe is being born. |
| Primitive operator realizations (`Int.add → i64::wrapping_add`) | `dsl/extdeps/languages/<target>.dag`. No Rust file manufactures realizations. | `emit` and `lens_cost` both read the same language-spec declaration. PR-B closes this. |
| **Lens outputs** | **There are no writer lenses.** Every lens is a pure reader over substrate + language spec. If a lens "needs" to write a fact, the fact is actually a substrate field that was missing upstream — add it to the substrate, do not put it in lens-side storage. Caching the result of a pure-reader lens is a separate optimization that can be added later based on profiling; it is not part of the lens contract. | Per §12.6. The M0 reader lenses (`lens_provenance`, `lens_depth`) already follow this pattern; PR-B's `lens_cost` extends it. No side tables, no `Dag::lens_results` map, no per-Port annotations for lens output. |
| Descent evidence (is this call strictly smaller?) | `lower::is_strictly_smaller` reads the raw `SurfaceExpr::Operator` structurally (`matches!(op, ArithmeticOp::Sub)` + positive-literal check). The plan proposed a typed `DescentEvidence` struct on `TransformNode`; the implementer dropped it and the direct SurfaceExpr walk is leaner. | `lower::descent_provable` and future bounded-recursion lens. At M1 scope, the check runs pre-lowering on `SurfaceExpr`; M2+ extensions (lexicographic, structural) become algebra-declaration annotations read post-lowering. |
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
| 1 | `TemplateArgument { parameter: p, value: p }` (self-reference stub) | **No ✅** | **M1(2.7) Class 3** — `build_template_arguments` returns `Vec::new()` on stub templates and on arity mismatch. The self-reference branch is deleted. |
| 2 | Post-sweep `AtomPayload::UnresolvedIdentifier(name)` reachable from a well-formed program | **No ✅** | **M1(2.7) Class 2/3** — operators never become `UnresolvedIdentifier` (parser emits `SurfaceExpr::Operator` directly); block-body `Fn` is a distinct `SurfaceItem::FnExternalBody` that lowers to `ArrowBody::Unparsed`. The resolve sweep's two skip branches are gone. |
| 3 | `ArrowBody::Pending` reachable at emission time | Yes (at M1(2.7)); no (post-PR-B) | PR-B: §8.11 ratchet hits zero in the same commit that adds the dissolution mechanism. At M1(2.7) `Pending` is structurally valid at inference time (signature type-checks via inhabitance, body-walking skipped); its emission-time invalidity is enforced by future emitter code that fails on the variant. |
| 3a | `TransformTarget` error state reached without fail-closing | **Not representable ✅** | **M1(2.7) Class 2** — `TransformTarget` has only 2 variants (`Callable`, `Operator`). There is no error-state variant; unresolved names live in the referenced `Declaration`'s connective and downstream readers follow resolved-identifier chains. The plan's earlier draft proposed `Unresolved` as a third variant; the implementer's cleaner factoring dodges it. Compile-time match exhaustiveness confirms the two arms are handled at every reader (infer.rs:232). |
| 3b | `ArrowBody` error state reached at inference without explicit variant handling | **Not representable ✅** | **M1(2.7) Class 3** — all 4 variants (`UserDefined`, `ExternalRealization`, `Pending`, `Unparsed`) matched exhaustively at infer.rs:297. `Pending` and `Unparsed` are documented scaffolded-state allowances at inference time; their emission-time fail-closed enforcement lands in PR-B. |
| 4 | Type parameter appearing in `dag.declarations()` top-level iteration | Verify | Pre-existing in free-cod-972; not touched by M1(2.7). Catalogue row stays open for future verification. |
| 5 | Sum variant resolved via `declaration_by_name` | Verify | Same — not touched by M1(2.7). Catalogue row stays open. |
| 6 | `TransformTarget` distinguishing callable vs operator via `UnresolvedIdentifier(name)` | **No ✅** | **M1(2.7) Class 2** — `TransformTarget` is a structural coproduct; operators never allocate a stub declaration. |
| 7 | `DescentEvidence` re-derivable by string-matching `target == "-"` | **No ✅** | **M1(2.7) Class 2** — `is_strictly_smaller` checks `matches!(op, OperatorKind::Arithmetic(ArithmeticOp::Sub))` at lower.rs:1309. Note: the plan proposed a `DescentEvidence` enum; the implementer dropped it and the result is leaner. |
| 8 | ~~`lower_type_for_port` accepting only `"Int"\|"Bool"\|"String"` via whitelist~~ | **STALE — never landed** | This row was added based on a misread of round-7 review feedback. The whitelist was already dissolved before the plan draft (QW5 SINGLE AUTHORITY receipt at lower.rs:1079-1094). Row retained as history, not a live invariant. |
| 9 | ~~Block-body `Fn` item silently skipped by `resolve_pending_identifiers`~~ | **STALE — never landed** | Same — this row was added based on a misread. Block-body `Fn` was already handled via `ArrowBody::Unparsed` + `lower_fn_item_unparsed` before the plan draft. Row retained as history. |
| 10 | `Declaration.inhabits: Some(id)` where `id` doesn't exist or isn't an algebra | Yes — no well-formedness check | Deferred to M1(3)+; captured here for tracking |
| 11 | `Declaration.meta_tag: Some(id)` where `id` is not itself a meta-type | Yes — no structural marker for "this IS a meta-type" | Deferred to M1(5)+; captured here for tracking |
| 12 | Operator dispatch before parse completes | **Not representable ✅** | **M1(2.7) Class 2** — parser calls `OperatorKind::from_symbol` at parse time; post-parse every operator is a typed variant. |
| 13 | Primitive identity resolved by name lookup (e.g. `declaration_by_name("Int")` at dispatch time) | **No ✅** | **M1(2.7) Class 1** — `Dag::int_shape() / bool_shape() / string_shape() / realization_meta_id()` cached at bootstrap; dispatch-time lookups compare `DeclarationId`, not `String`. (New row: the plan didn't enumerate this invariant; the implementer added the fix anyway.) |
| 14 | `is_realization_shape` comparing realization meta-type by name | **No ✅** | **M1(2.7) Class 4** — compares cached `DeclarationId` via `dag.realization_meta_id()`. (New row, same reason as 13.) |
| 15 | Growing the set of operators requires a Rust-side compiler edit (`OperatorKind` enum variant) | **Yes** — open | **Deferred, tracked.** As long as `OperatorKind` exists as a compiler-defined coproduct, adding a new operator (e.g., `%`, `&&`, unary `-`) requires a Rust enum variant. This is a dual-authority pattern: operator identity lives in both `operators.rs` and `dsl/std/algebra.dag`. The full dissolution is "operators become regular `Callable`s on algebra-field declarations" — see operators.rs:45-48 dissolution trigger. Until that lands in M2+, the row stays open as acknowledged debt. §9 non-goals intentionally excludes all operator-set growth from M1 to prevent this row from ratcheting in the wrong direction. |
| 16 | `ArrowBody::Unparsed(SourceSpan)` is a scaffold state on a core substrate boundary with no sanctioned exception in `INVARIANTS.md` | **Yes** — open, blocked on INVARIANTS.md decision | **Deferred, tracked.** `INVARIANTS.md` §"No short-term solutions" sanctions exactly two exceptions: emission via language spec and `ArrowBody::Pending` for primitive realization lag under a numeric ratchet. `ArrowBody::Unparsed` was added in M1(2.7) for block-body `fn` declarations whose bodies use grammar not yet parseable (e.g., `match` in `classical_and`). It is structurally parallel to `Pending` — "the compiler knows a fact but the structure can't fully express it yet" — but is not currently sanctioned. Resolution requires either (a) extending `INVARIANTS.md` with a parallel exception + numeric ratchet, (b) extending the grammar so match/pipe/lambda parse, or (c) rewriting std/ functions to avoid the unparseable forms. **This row blocks substrate-level cleanup until the choice is made.** |

**Status summary:** rows 1, 2, 3a, 3b, 6, 7, 12, 13, 14 are closed in
M1(2.7). Row 3 (`Pending` at emission) is closed at inference time
and becomes impossible-by-type when PR-B deletes the variant. Rows 4, 5
are inherited from the earlier plan and need verification in free-cod-972
(the plan never audited them against the actual code). Rows 8, 9 were
stale when added and retained only as history. Rows 10, 11 are deferred
to M1(3)+ and M1(5)+.

**How the M1(2.7) implementation actually enforces lifecycle
boundaries.** The plan's earlier draft described two enforcement
shapes — structural (split the type) vs exhaustive-match + diagnostic
(keep the type, fail-closed arms). The implementer did something even
cleaner for `TransformTarget`: dropped the error variant entirely, so
the type has only valid-state variants. For `ArrowBody`, the four
variants are matched exhaustively in infer.rs with scaffolded-state
arms (`Pending`, `Unparsed`) that allow signature type-checking and
explicitly defer body-walking with a documented dissolution trigger.
Compile-time match exhaustiveness is the structural enforcement;
documented dissolution triggers are the audit trail. No grep gate
needed.

This is a stronger invariant than the plan originally committed to,
and it changes the scoring on the "make illegal states unrepresentable"
invariant from "partially satisfied by exhaustive-match discipline"
to "structurally satisfied — the illegal variant doesn't exist."

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

**Fact-drop discipline (reconciled with §12.6 minimality).** The
earlier draft of this section said "if an upstream stage has a fact
in hand and no downstream consumer has committed to reading it YET,
the fact goes on the substrate ANYWAY." **That language is rescinded**
— it contradicts §12.6's minimality invariant and `INVARIANTS.md`
§"No short-term solutions," both of which forbid speculative substrate
growth. The correct discipline is:

1. **Enumerate future consumer questions in §4 as N rows** — this is
   the design-oracle inventory, not substrate population. N rows are
   questions the plan tracks; they do not imply substrate fields have
   been added. This is a pure documentation activity.
2. **Add substrate fields only when a consumer lands in the same PR
   that adds the field** — per §3.1 authority model ("no new fact
   layer without a consumer"), §12.6 rule 1(b), and
   `INVARIANTS.md:1329-1348`. Substrate additions are consumer-driven.
3. **Dropping a known fact and re-deriving it later is still a bridge
   — but the fix is not to pre-populate the substrate.** The fix is
   to add the consumer in the same PR that adds the fact. If the
   consumer isn't ready, the fact isn't ready either; it stays as an
   N row in §4 until the consumer is committed to.
4. **The Producer column in §4 forces the question at consumer-
   writing time**, not at substrate-design time. When you go to write
   a new consumer, §4 tells you which stage should authoritatively
   produce its facts. If the answer is "a stage that doesn't yet
   produce that fact," the PR that introduces the consumer also
   introduces the producer. Neither lands without the other.

The earlier wording blurred enumeration (valuable — a list of
questions) with substrate population (forbidden without a consumer).
Enumeration is the design oracle; substrate edits are the
implementation. Keep them separate.

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

### §4.3 `infer.rs` (653 lines, all R-rows closed in M1(2.7))

| # | Question | Producer | Status | Notes |
|---|---|---|---|---|
| 1 | What's a Port's state (Uninferred/Resolved/Unresolved)? | `lower(bodies)` + `infer` | S | `Port::state()` |
| 2 | What's a Node's Behavior variant? | `lower(bodies)` | S | `dag.node()` pattern match |
| 3 | What's a Declaration's TypeConnective? | `lower(collect)` + `lower(bodies)` | S | `decl.connective` pattern match |
| 4 | For Arrow: inputs/output? | `lower(bodies)` | S | `TypeConnective::Arrow { .. }` |
| 5 | For Arrow: body kind? | `lower(bodies)` / `bootstrap` | S | `ArrowBody` 4 variants (`UserDefined`, `ExternalRealization`, `Pending`, `Unparsed`) — see §2 baseline |
| 6 | Is a Bind's value Port resolved? | `infer` | S | Port state check |
| 7 | Is a Transform target callable or operator? | `parse` | **S ✅** | Closed in M1(2.7). `SurfaceExpr::Operator` is a first-class parser variant; lower builds `TransformTarget::Callable \| Operator` directly. |
| 8 | What kind of operator is this (arithmetic vs comparison)? | `parse` (via `OperatorKind::from_symbol`) | **S ✅** | Closed in M1(2.7). `OperatorKind::Arithmetic(_) \| Comparison(_)` is the structural split. |
| 9 | What's the operator's return type semantics (returns T or Bool)? | `infer::resolve_operator_arrow` reading `OperatorKind` variant | **S ✅** | Closed in M1(2.7). infer.rs:473 encodes "Arithmetic → operand type, Comparison → `bool_shape()`" as a structural match, not a sibling string match. |
| 10 | What DeclarationId does an Identifier resolve to? | `resolve_sweep` | S | `ResolvedIdentifier(DeclarationId)` carries it |
| 11 | What does a DeclarationId map to at port-level? | `infer::walk_to_type_shape` over `decl.connective` | S | Returns `TypeShape::new(current)` for named decls |
| 12 | For Instantiation: template + arguments? | `lower(bodies)` | S | `Instantiation { template, arguments }` |
| 13 | Walk substitution context? | `infer` | S | `SubstStack` |

**All three closed in M1(2.7).** Operator identity flows forward from
parse through lower to infer without any name-matching reconstruction.
Verification: §8 gates grep for the deleted helpers return zero matches
in production code.

### §4.4 Descent evidence (`lower::is_strictly_smaller`)

| # | Question | Producer | Status | Notes |
|---|---|---|---|---|
| 1 | Is this call operating on a structurally smaller sub-value? | `lower(bodies)` | **S ✅** | Closed in M1(2.7). `is_strictly_smaller` structurally matches `OperatorKind::Arithmetic(ArithmeticOp::Sub)` on the SurfaceExpr::Operator, checks operand identity and positive-literal shrink. No string matching. The plan earlier proposed a `DescentEvidence` struct on `TransformNode`; the implementer dropped it and the SurfaceExpr walk is leaner. |
| 2 | What's the "smaller-by-1" pattern for each operator? | Hard-coded to `ArithmeticOp::Sub` at M1(2.7); extends via `dsl/std/algebra.dag` descent annotations when M2+ needs lexicographic/structural descent | **S ✅** | Closed at M1(2.7) scope. Future extensions (lex/structural descent) become substrate fields on algebra declarations; the current hard-coding is honest about M1 scope rather than pretending the general case is supported. |

**Both closed in M1(2.7).** No `DescentEvidence` struct was introduced —
the direct SurfaceExpr walk is lean enough. Termination checking
extensions move into substrate (algebra-declaration fields) rather than
inventing a parallel descent vocabulary.

**Note on the plan's mid-draft proposal.** The earlier plan introduced
a `DescentEvidence { StrictSubValue | PreservedValue | Unrelated }` enum
and a `ShrinkFactor { Constant(u32) | Structural }` enum as §6.1
substrate additions. Neither shipped. The review feedback that pushed
back on "unclassified coproducts" was correct — both enums were
premature formalization of a concern that didn't yet need its own
vocabulary. The implementer's decision to keep `is_strictly_smaller` as
a direct SurfaceExpr reader at M1 scope, with algebra-declaration
annotations as the M2+ extension path, is the right shape.

### §4.5 `lens_cost.rs` (future, M1(3)) — reader-only

| # | Question | Producer | Status | Notes |
|---|---|---|---|---|
| 1 | What's a Node's per-op cost? | `lang_spec` (PR-B) | N | Cost-per-primitive is a field on the primitive's realization declaration in `rust.dag`. The lens reads on demand; no per-node storage. |
| 2 | What's a Transform's target cost? | `OperatorKind` (substrate) + `lang_spec` declaration | N | Pure function of `TransformTarget`: `Callable(id)` walks through the Arrow body; `Operator(kind)` looks up the algebra declaration's realization. No intermediate storage. |
| 3 | How do costs compose across Behavior kinds? | `lens_cost` composition kernel | N | Sequence / Loop / Branch composition is a pure function of the Node's Behavior variant + input Port costs. Each query walks the Dag fresh. |
| 4 | ~~Where does the lens STORE its results?~~ | **Moot — minimal system has no storage to pick** | **N/A** | Per §12.6, the minimality invariant dissolves the storage question: if cost is derivable from substrate + `rust.dag`, there is nothing to store. Memoization, if profiling ever demands it, is a transparent local cache — never a substrate-level decision and never shared across lenses. |
| 5 | For ExternalRealization: target-world cost? | `lang_spec` (PR-B) | N | Blocked on `rust.dag` declaring realization costs. Same pure-read pattern as row 1. |

**Four live N-rows, one moot.** All blocked on PR-B's `rust.dag`
realization-schema decision, not on a lens-storage mechanism. The
storage question from earlier drafts is closed by the minimality
invariant, not postponed.

### §4.6 Rust emitter skeleton (future, M1(4))

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

### §4.7 Interpreter (future, later)

| # | Question | Producer | Status | Notes |
|---|---|---|---|---|
| 1 | For each Node: evaluate its value given input Port bindings | `interp` consumes substrate | N | Tree walker over L1 behaviors |
| 2 | For Transform over primitive: call host primitive | `lang_spec` (PR-B) | N | Reads ExternalRealization for runtime binding |
| 3 | For Transform over user function: recursively evaluate sub-DAG | `interp` consumes substrate | N | Same substrate as emission |
| 4 | Termination: is the current walk bounded? | `lower::is_strictly_smaller` at M1 scope; algebra-declaration descent annotations at M2+ | N | Reads the existing descent check; no new vocabulary required. |

**Four N-rows.** All depend on §4.3 + §4.4 R-rows being closed.

---

## §5. Gap analysis

**Open R-rows (reconstructive facts currently in the codebase): 0**

All five genuine R-rows from the plan's original enumeration are
closed in M1(2.7):

1. ✅ Transform target kind (callable/operator) — §4.3 via `TransformTarget` coproduct
2. ✅ Operator kind (arithmetic/comparison) — §4.3 via `OperatorKind` coproduct
3. ✅ Operator return type semantics — §4.3 via `resolve_operator_arrow` structural dispatch
4. ✅ Descent evidence — §4.4 via `is_strictly_smaller` structural match on `OperatorKind::Arithmetic(ArithmeticOp::Sub)`
5. ✅ Descent shrink factor — §4.4 hard-coded at M1 scope; M2+ extends via algebra-declaration annotations

Two R-rows (ports 6 and 7, the port-type whitelist and block-body `Fn`
silent-skip) were added to the plan based on a misread of round-7
review feedback — both had already been dissolved before the plan
draft was written. Those are documented as history in §3.3 rows 8–9,
not as closed work.

**Beyond-plan fixes the M1(2.7) implementation added that were not in
§4:** Class 1 primitive identity caching (`Dag::int_shape()` etc.) and
Class 4 realization shape comparison by `DeclarationId` instead of
name. Both address invariants the plan did not enumerate and landed in
§3.3 as new catalogue rows 13 and 14. Their absence from the original
§4 enumeration is honest evidence that the plan's consumer walk was
not exhaustive; the implementer caught what the planner missed. §6.1
retrospective treats this as a process data point, not a failure.

**Open N-rows (future consumer questions): 15** (5 cost, 6 emit, 4 interp)

- 5 cost lens questions now block only on PR-B's `rust.dag` schema
  decision; the storage-mechanism question from earlier drafts is
  moot per §12.6 minimality invariant
- 6 emit questions block on parser support for `realization { ... }`
  literals + `rust.dag` bootstrap; row 6 (ownership) stays orthogonal
- 4 interpreter questions block on M1(3)+ interpreter skeleton

**Conclusion:** PR-A is done. PR-B is unblocked except for its own
design-doc work on `rust.dag` realization schema. Every R-row that
ever existed for PR-A's scope is closed in M1(2.7); every fact PR-B
consumes is either already in substrate or will become a substrate
field before any lens reads it.

---

## §6. Atomic work units

Two large PRs. Each updates substrate + every affected consumer
simultaneously. No intermediate states, no bridges.

### §6.1 PR-A: Structural operator handling + round-7 cleanup (est. 10–14 hours)

**Status:** ✅ **Complete.** M1(2.7) structural fix PR shipped
2026-04-14 — 60/60 tests green, clippy clean, 10 files
(+1276/−493 LOC), 14 enumeration gaps closed across 4 classes. This
section is now a retrospective receipt rather than a forward plan.

**What the implementer shipped (4 classes, 14 gaps):**

- **Class 1 — primitive identity (4 gaps).** `Dag::int_shape() /
  bool_shape() / string_shape() / realization_meta_id()` cached at
  bootstrap; dispatch-time lookups compare `DeclarationId`, not
  `String`. `lower_type_for_port` dissolved into `type_to_declaration_id`
  with a fresh-stub guard (QW5 SINGLE AUTHORITY receipt at
  lower.rs:1079).
- **Class 2 — operator dispatch (2 gaps).**
  `TransformTarget = Callable(DeclarationId) | Operator(OperatorKind)`.
  `SurfaceExpr::Operator` is a first-class parser variant; operators
  never allocate stub declarations. `OperatorKind =
  Arithmetic(ArithmeticOp) | Comparison(ComparisonOp)` encodes the
  output-type rule in the variant. `is_strictly_smaller` structurally
  matches `ArithmeticOp::Sub`. Deleted: `OPERATOR_FIELD_MAP`,
  `is_operator_name`, `is_comparison_operator`, `unresolved_operator_name`.
- **Class 3 — scaffold honesty (3 gaps + module/import).**
  `ArrowBody::Unparsed(SourceSpan)` fourth variant with a distinct
  dissolution trigger. `SurfaceItem::Fn` requires a `SurfaceExpr` body;
  `FnExternalBody` is a sibling variant lowering to `Unparsed`.
  `SurfaceItem::Data / Module / Import` emit real facts at the parser
  boundary. `TemplateArgument` stub self-reference branch deleted —
  stub templates yield `Vec::new()` arguments.
- **Class 4 — parallel authorities (2 gaps).** `is_realization_shape`
  compares cached `DeclarationId` instead of name. Port-type authority
  dissolved via Class 1.

**Where the implementer's factoring diverged from the plan (cleaner):**

1. **No `TransformTarget::Unresolved` variant.** The plan's earlier
   draft proposed three variants including
   `Unresolved { name, span, diagnostic }`. The implementer dropped
   it — unresolved names live in the `Declaration` connective as
   `UnresolvedIdentifier` and the resolve sweep is authoritative.
   Only two valid-state variants remain. Compile-time match
   exhaustiveness is trivially satisfied at every reader (infer.rs:232).

2. **No `DescentEvidence` struct and no `ShrinkFactor` enum.** The
   plan proposed a typed descent-proof vocabulary on `TransformNode`.
   The implementer kept `is_strictly_smaller` as a direct SurfaceExpr
   reader (structurally matching `ArithmeticOp::Sub` + positive
   literal at lower.rs:1309). No second vocabulary, no invariant
   drift. The earlier reviewer pushback on "unclassified coproducts"
   was correct — both types were premature formalization of a
   concern that didn't need its own vocabulary.

3. **`operators.rs` smaller than planned.** The plan proposed
   `from_symbol` + `kind_to_algebra_ref`. The implementer shipped
   `from_symbol` + `symbol` (a diagnostic-display helper). Algebra-ref
   linking is deferred to M2+ when the surface grammar exposes
   algebra-field access directly; at that point `SurfaceExpr::Operator`,
   `TransformTarget::Operator`, and `OperatorKind` ALL dissolve into
   regular algebra-field calls. The dissolution trigger is documented
   inline at operators.rs:45-48.

**Where the plan missed what the implementer caught:**

1. **Class 1 primitive identity caching** was not in the §4
   enumeration. The plan's §3.1 named primitives as "resolved once
   at bootstrap" but did not audit the cached-getter API or flag
   dispatch-time name lookups as an R-row. Row 13 in §3.3 now tracks
   it.
2. **Class 4 `is_realization_shape`** was also not in §4. Row 14 in
   §3.3 now tracks it.
3. **`SurfaceItem::Module / Import` first-class representation.** The
   plan did not enumerate these as facts that should flow forward
   through the parser boundary. The implementer made them first-class
   `SurfaceItem` variants even though lowering is a no-op at M1(2.7)
   (consumed later in M2+ module scoping).

These catches are honest evidence that the plan's consumer walk was
not exhaustive. §12's design-doc-first process does NOT guarantee
exhaustive enumeration — it guarantees that an incomplete enumeration
is visible up front and correctable in review. M1(2.7) exercised that
correction loop in practice, and the correction loop worked.

**Deletions in M1(2.7) (verified by grep — zero live references):**

- `infer::unresolved_operator_name`
- `infer::is_comparison_operator`
- `operators::is_operator_name`
- `operators::OPERATOR_FIELD_MAP`
- `lower::is_strictly_smaller` string-match against `"-"` (rewritten structural)
- `lower::lower_type_for_port` primitive whitelist (already gone — QW5)
- `resolve_pending_identifiers` operator-skip branch
- `resolve_pending_identifiers` block-body `Fn` silent-skip branch

**Verification receipts (inline in free-cod-972):**

- `operators.rs:3-48` — whole-module dissolution receipt + 4-pattern audit + dissolution trigger
- `lower.rs:1079-1094` — `QW5 SINGLE AUTHORITY` receipt for `lower_type_for_port`
- `lower.rs:1262-1267` — structural descent check receipt
- `infer.rs:447-472` — `resolve_operator_arrow` + `OPERATOR_FIELD_MAP is gone` receipt
- `infer.rs:489-494` — `is_realization_shape` DeclarationId-compare receipt
- 8 new `m17_*` tests in `tests/m1_substrate_test.rs`, one per class gap

**Closes:** all 5 genuine R-rows from the plan's original §4
enumeration (R-rows 6, 7 were stale when added and retained only as
history in §3.3 rows 8-9), plus the substrate-level invariants from
§3.3 rows 1, 2, 3a, 3b, 6, 7, 12, 13, 14.

**Verdict on the plan's contribution:** The Design Oracle discipline
(§3 + §12) made PR-A's correction loop fast — reviewer findings
mapped to specific oracle rows rather than requiring full re-review.
Specific type proposals (`TransformTarget::Unresolved`, `DescentEvidence`,
`kind_to_algebra_ref`) were over-formalized and the implementer's
simpler factoring won on every axis. The plan's load-bearing
contributions were the **authority model** (§3.1), the
**invalid-state catalogue** (§3.3), the **boundary-flow discipline**
(§3.4), and the **process commitment** (§12). The specific substrate
shapes belonged in the implementation, not the plan.

### §6.2 PR-B: M1(3) + M1(4) — cost lens + Rust emitter (reader-only, est. 10–14 hours)

**Status:** Not started. **Design-closed.** Realization schema is
pinned in §11 question 7 as ordinary `Conj` declarations with typed
fields (`for: DeclarationId`, `target: DeclarationId`, `body: String`,
`cost: Int`) and `meta_tag` edge to the `Realization` meta-type. No
compiler-native struct in `dag.rs`.

**Purpose:** first cost analysis, first real target-language emission,
`rust.dag` as the first real language spec, `ArrowBody::Pending`
dissolution at emission time — all as one atomic unit under the
**minimality invariant (§12.6)**: no writer lenses, no side tables,
no storage-mechanism decision.

**What PR-B is NOT doing:**

- Not building a writer-lens storage mechanism. No per-lens side
  tables, no `Dag::lens_results` map, no per-Port annotations for
  lens output. Per §12.6.
- Not caching cost-per-node — cost is a pure function of
  `(substrate, rust.dag)`, recomputed on demand at every query. If
  profiling ever demands caching, it's a transparent local cache
  added later, not a substrate-level or shared-lens decision.
- Not moving operator cost / signature / descent into `operators.rs`
  — those stay in `std/algebra.dag` + `rust.dag` per §3.1 authority
  model. `operators.rs` remains surface-syntax-only.
- Not inventing a god table upstream to make lenses cheap to write.
  If a cost-composition kernel is shared across lenses, it lives as
  a pure-function module (`cost_composition.rs`) — not as a field on
  `Dag` or `Node`. Per §12.6 minimality.

**Substrate additions:**

- **Realization schema pinned (see §11 question 7).** A realization
  item lowers to an ordinary `Conj` declaration with typed fields:
  `for: DeclarationId`, `target: DeclarationId`, `body: String`,
  `cost: Int`. The declaration's `meta_tag` points at the
  `Realization` meta-type declaration to distinguish realization
  declarations from ordinary records. **No compiler-native
  `Realization` struct in `dag.rs`.** Realizations participate in
  the normal declaration table, declaration_by_name, resolve sweep,
  and walks — just like any other `Conj`.
- Parser support in `parse.rs` for `realization { for: X.add; target:
  rust_target; body: "i64::wrapping_add"; cost: 1 }` item syntax —
  the one remaining parser gap from M1_FOLLOWUPS.md. The `target`
  field references a target-language Declaration (e.g., `rust_target`
  as a module-level marker in `rust.dag`), not a loose `"rust"` string.
- `dsl/extdeps/languages/rust.dag` parsed as an 8th bootstrap file.
  Each entry is a `realization` declaration with the typed fields
  above. **`bootstrap::inject_realization_stub` is already deleted
  in M1(2.7); the comment reference in M1_DESIGN.md §8.6 is stale
  and gets cleaned up in this PR.**
- `ArrowBody::Pending` ratchet hits zero once `rust.dag` populates
  primitive Arrows directly with `ExternalRealization(decl_id)`.
  PR-B ends with `Pending` ready to delete as a variant; the variant
  deletion itself lands as a follow-up once any non-primitive uses
  are audited.

**New files:**

- `src/v3/compiler/src/lens_cost.rs` — pure reader. Given a `NodeId`
  and `SubstStack`, returns a `CostSummary` by walking the Dag.
  No persistent storage. Kernel is a straight match on `Behavior`
  variant with recursive composition via input Ports. Cost-per-primitive
  read from the referenced realization declaration's `cost` field.
- `src/v3/compiler/src/emit.rs` — pure reader. Given a `Dag` and a
  **target language `DeclarationId`** (the `rust` target-marker
  declaration inside `rust.dag`, not a loose `"rust"` string), walks
  the Dag and produces Rust source via the language spec. Fails
  closed on `ArrowBody::Pending` and `ArrowBody::Unparsed` (both
  invalid at emission). No hardcoded `"i64::"` / `quote!` strings
  — every target-side fact is read from `rust.dag`. Target identity
  is a typed edge per §11 question 7.
- `dsl/extdeps/languages/rust.dag` — first real language spec.
  `realization` declarations for each primitive operator and
  primitive type.

**Files touched:**

- `parse.rs` — adds `realization` item parsing + record-literal body
- `bootstrap.rs` — deletes `inject_realization_stub`; parses
  `dsl/extdeps/languages/rust.dag` as the 8th bootstrap file
- `dag.rs` — **no new struct.** Realization declarations reuse the
  existing `TypeConnective::Conj` shape with field accessors reading
  `for`, `target`, `body`, `cost` off the children list. Per §11
  question 7 pinning and §3.1 language-spec authority row. Any PR
  that proposes adding a `Realization` field-schema struct to
  `dag.rs` is reopening a pinned decision and is out of scope.
- Tests: new `smoke_compile_and_run` takes `let x: Int = 1 + 2`
  through parse → lower → infer → lens_cost → emit → `cargo check`
  → run returning `3`. Test asserts `lens_cost` is a pure function:
  two back-to-back queries with no intervening mutation return
  identical results structurally.

**Horizontal collapse opportunities (5):**

1. **Cost lens and emitter share a `StructuralVisitor` trait.** Both
   walk `TypeConnective` variants, dispatch on `ArrowBody`, consume
   `OperatorKind`. Writing them together reveals the shared walker.
   Extracting it after the fact would be a separate refactor; doing
   it together avoids the bridge.
2. **`rust.dag` IS the unified cost + realization + body source.**
   One declaration per primitive, typed fields for everything needed
   at emission and cost-query time. Cost lens and emitter read the
   SAME declaration. No temporary cost table that the emitter later
   rereads.
3. **`ArrowBody::Pending` dissolution lands in the same PR as the
   mechanism to populate `ExternalRealization`.** Primitive Arrows
   are constructed directly with `ExternalRealization(decl_id)` from
   `rust.dag` parse. Post-bootstrap, zero `Pending` in production
   declarations in the same commit as the dissolution mechanism
   arrives.
4. **`inject_realization_stub` deletion is the natural falling-out.**
   PR-B adds parser support AND deletes the stub. Zero Rust-manufactured
   declarations in production bootstrap.
5. **Reader-only lens discipline generalizes across lenses.**
   `lens_cost` is the first M1(3) lens; ownership, effects, purity,
   space-bounds will follow the same pattern. Because the storage
   question is moot (§12.6), every future lens is a same-shape pure
   reader. Substrate edits happen only when a NEW fact needs to exist
   as an authoritative field, which is a substrate decision, not a
   lens decision. The v3 thesis's "new lens = new file, zero
   substrate edits" success bar is provable by construction.

**Acceptance gates (in addition to §8 universal gates):**

- End-to-end: `compile("let x: Int = 1 + 2").emit_rust()` produces Rust
  source that compiles under `cargo check` and runs returning `3`.
- **Purity gate:** `lens_cost::query(&dag, node_id)` is a pure
  function. Two back-to-back invocations with no intervening mutation
  return identical results. Test asserts this structurally and
  structurally rules out a hidden side table via `#[deny(mutable_state)]`
  or equivalent module-level discipline.
- **Minimality gate:** `lens_cost.rs` and `emit.rs` MUST NOT introduce
  any `Dag::*_results`, any `HashMap<NodeId, *>` side table, or any
  `Port` / `Node` annotation field for their own output. Enforcement:
  any new field on `Dag`, `Port`, or `Node` added by PR-B must be an
  inherently-substrate fact (an input authored by the user or computed
  at lower time), not a derived fact computed by a lens.
- `grep "inject_realization_stub\|Pending" src/v3/compiler/src/` →
  only `#[cfg(test)]` and dissolution-receipt comments; zero
  production code paths.
- `lens_cost.rs` is < 250 lines (success-bar budget).
- `grep "\"i64::\\|quote!\\|\"Rust " src/v3/compiler/src/emit.rs` → zero
  matches (the emitter has no hardcoded target knowledge).
- Cost lens + emit both go through a shared walker pattern (proven by
  refactoring, not mandated in advance).

**Closes:** 4 live cost N-rows (all except the moot storage question)
+ 5 emit N-rows (all except ownership, which is M1(5)+).

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
9. **Lifecycle-boundary exhaustive-match gate** — every downstream
   reader of `TransformTarget` and `ArrowBody` must match every variant
   with no wildcard arm. `TransformTarget` has only valid-state variants
   (`Callable`, `Operator`) post-M1(2.7), so exhaustive match is
   trivially satisfied. `ArrowBody` has 4 variants at M1(2.7) —
   `UserDefined`, `ExternalRealization`, `Pending`, `Unparsed` — all
   must be matched explicitly. `Pending` and `Unparsed` are
   scaffolded-state allowances at inference time (signature type-checks,
   body-walking deferred) but MUST fail-closed at emission time
   (PR-B). Enforcement:
   - Rust's compile-time `match` exhaustiveness (no wildcard `_ => ...`
     arms allowed in `infer.rs`, `lower.rs`, `lens_*.rs`, or
     `emit.rs` when scrutinizing these types; grep pattern:
     `grep -E 'match .*\.target.*\{[^}]*_ =>|match .*\.body.*\{[^}]*_ =>'`
     returns zero).
   - Per-stage test that feeds a deliberately-scaffolded program
     through the stage and asserts the stage behaves correctly:
     inference allows scaffolded variants with documented dissolution
     triggers; emission fails closed with a diagnostic.
   This is the invariant-grade enforcement for §3.3 rows 2, 3, 3a, 3b.

These gates enforce invariants structurally rather than by convention.
Any PR failing one is not mergeable.

---

## §9. Non-goals (explicit scope exclusions)

Out of scope for M1(2.6) → M1(4):

- **Operator extensions beyond M1(2.7)'s shipped set.** `OperatorKind`
  is exactly `Arithmetic(ArithmeticOp) | Comparison(ComparisonOp)` — 10
  variants total (4 arithmetic: Add, Sub, Mul, Div; 6 comparison:
  Eq, Ne, Lt, Le, Gt, Ge). Explicit non-goals:
  - `%` (modulo / remainder) — post-M1, adds `ArithmeticOp::Mod` or dissolves directly into algebra-field call
  - Boolean operators (`&&`, `||`, `!`) — post-M1, may require a new
    `OperatorKind::Bool(BoolOp)` family since return-type invariants
    differ from `Comparison`
  - Unary operators (unary `-`, `!x`) — post-M1, `TransformNode.inputs`
    shape differs (one input, not two)
  - Null-coalescing, ternary, bitwise ops, shifts — post-M1, deferred
    until a user-code driver forces the question
  Every post-M1 operator addition goes through the §12 design-doc
  process OR dissolves `OperatorKind` entirely if the M2 grammar
  exposes algebra-field access (see operators.rs:45-48 dissolution
  trigger). Pre-baking extension seams is itself a bridge pattern.
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

## §11. Open questions

**Closed by M1(2.7) implementation** (all 5 earlier questions):

1. ~~**Should `TransformTarget::Unresolved` carry the failed name and
   span?**~~ **Moot — the variant doesn't exist.** Implementer dropped
   it; unresolved names live in the `Declaration` connective and the
   resolve sweep is authoritative. Cleaner than any of the
   plan-proposed shapes.

2. ~~**Is `OperatorKind` the right split (Arith vs Cmp), or flatter?**~~
   **SHIPPED:** hierarchical, exactly `Arithmetic(ArithmeticOp) |
   Comparison(ComparisonOp)` with 10 total variants. The output-type
   rule is encoded in the variant: `resolve_operator_arrow` at
   infer.rs:473 matches on `Arithmetic(_) → operand type` vs
   `Comparison(_) → bool_shape()`.

3. ~~**Where does `DescentEvidence` live?**~~ **Moot — the struct
   doesn't exist.** Implementer kept `is_strictly_smaller` as a direct
   SurfaceExpr reader (structurally matching `ArithmeticOp::Sub` +
   positive literal). No second vocabulary. The reviewer pushback on
   "unclassified coproducts" was correct.

4. ~~**`CostUnit` for primitive cost in `operators.rs`?**~~ **Moot —
   `operators.rs` has no cost function.** Cost lives on `rust.dag`
   realization declarations, read by `lens_cost` in PR-B as a pure
   function.

5. ~~**Does `resolve_type_expr` exist as a unified entry point?**~~
   **YES — named `type_to_declaration_id`** (lower.rs:507). The
   unified entry point predates the plan draft. `lower_type_for_port`
   already delegates to it; the plan's proposed introduction step
   was unnecessary. Reviewer's hard-blocker concern on this question
   is satisfied.

**Closed by §12.6 minimality invariant:**

6. ~~**Exact shape of the lens storage mechanism for PR-B.**~~ **Moot
   — no writer lenses.** Per §12.6, the minimality invariant dissolves
   the storage question entirely. Cost is a pure function of
   `(substrate, rust.dag)`. Caching, if ever needed, is a transparent
   local optimization — not a substrate decision, not shared across
   lenses. The earlier reviewer who preferred per-lens side tables was
   answering a question that turns out not to need an answer: if the
   fact is pure, it has no storage. The refinement over either
   "reader-only" or "per-lens side tables" is the user's own:
   minimality is the tiebreaker; side-table duplication across lenses
   is an invariant violation, and god tables upstream are also not
   the answer.

**Closed by reviewer push in this cleanup:**

7. ~~**`rust.dag` realization schema.**~~ **PINNED: realizations are ordinary
   `Conj` declarations.** A `realization` item in `rust.dag` lowers to a
   regular `Declaration` whose connective is a `Conj` with named fields:
   - `for: DeclarationId` — the primitive being realized (typed edge,
     not a name string)
   - `target: DeclarationId` — the target language declaration itself
     (e.g., the `Rust` target declaration in a future `targets.dag` or
     the `rust.dag` module-level marker). NOT a loose `"rust"` string.
   - `body: String` — the target-language body text (the one part that
     is inherently string-valued because it's emission-side source code)
   - `cost: Int` — an Int-typed field at M1 scope. The shape of cost
     (single cycle count vs. `{cycles, allocations, io}` record) is
     deferred to when a second cost-sensitive consumer forces the
     question; at M1 one `Int` field suffices.

   **No compiler-native `Realization` schema in `dag.rs`.** A realization
   is a Declaration like any other; it participates in `declaration_by_name`,
   `resolve_sweep`, `TypeConnective::Conj`, and the normal walk. The
   `meta_tag` field already points at the `Realization` meta-type
   Declaration to distinguish realization declarations from ordinary
   records — the existing substrate carries this discipline.

   **Why this is the thesis-consistent choice.** Per §3.1 language-spec
   authority row and THESIS.md's "one spec file edit per new target"
   claim, introducing a new target language must be a `dsl/extdeps/
   languages/<target>.dag` edit, not a `dag.rs` edit. A compiler-native
   `Realization { target, body, cost }` struct would fork the shape at
   every new target (Go, Python, Swift, each needs a new compiler edit).
   Ordinary Conj declarations keep new targets in spec-file-edit
   territory.

8. ~~**Is `Realization` its own substrate shape?**~~ **Answered by #7
   above.** No. Ordinary `Conj` declarations with the `meta_tag` edge
   to a `Realization` meta-type. The compiler gets no new struct.

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

### §12.6 Minimality invariant — lenses are pure readers; no duplication

**The winning design is the cheapest to implement and maintain.**
Minimality is the tiebreaker between competing substrate shapes. This
section pins three related rules that together rule out the most
expensive failure modes: writer lenses with persistent storage,
parallel side tables across lenses, and god tables upstream.

**Rule 1 — Lenses are pure readers over substrate.** Every lens is a
function `(&Dag, &LangSpec, Query) -> Result`. No persistent storage,
no internal cache, no mutation. If you find yourself wanting a lens
to "write a fact," one of two things is true:

- (a) The fact is **derivable** from `(substrate, lang_spec)`. Then
  recomputing on each query is free relative to the substrate walk,
  and the lens stays pure. Caching is a transparent optimization
  that can be added later based on profiling; it is NOT part of the
  lens contract, and it MUST NOT leak into substrate.
- (b) The fact is **authoritative** — it is a piece of truth about
  the program that cannot be computed from what's already in
  substrate. Then it is a substrate fact, and it belongs on the
  `Dag` / `Declaration` / `Node` / `Port` / language-spec
  declaration at the grain that fits. The substrate field is added
  BEFORE the lens that consumes it, in the same PR or an earlier
  one. "Lens writes a fact" is the wrong factoring of (b) — the
  fact is upstream, not lateral.

Writer lenses as a category do not exist in this codebase. M0's
reader lenses (`lens_provenance`, `lens_depth`) already embody the
pattern. PR-B's `lens_cost` extends it.

**Rule 2 — No duplication of lens-local state across lenses.** If
lens A and lens B both want to cache the same composition kernel
(e.g., "walk a Dag, compute something per-Node"), that kernel lives
as a shared pure-function module (`src/v3/compiler/src/visitor.rs`
or similar). NOT as a field on `Dag`, NOT as a per-Port annotation,
NOT as a hashmap in some analysis struct reached via trait. If lens A
ships a side table and lens B reinvents a near-identical side table,
the second side table is a red flag — it's signaling that either (i)
the fact belongs in substrate per Rule 1(b), or (ii) the two lenses
share a computation and the shared computation belongs in a single
pure-function module. Either remediation is cheaper than two side
tables.

**Rule 3 — No god tables upstream.** Rule 1(b) is not a license to
shove every derived fact into a mega-field on `Dag`. Each upstream
fact must earn its place: it must be an authority that multiple
downstream readers need, it must have a single natural grain
(per-Declaration, per-Node, per-Port, etc.), and it must be
independently justified. "I might need this later" is not
justification. "Lens A and lens B both need this" is not
justification if the shared need can be served by a shared
pure-function kernel. Upstream state is always cheaper to add than
to remove, so adding it is the exception, not the default.

**What these rules rule out (concrete failure modes):**

- `Dag::lens_results: HashMap<LensId, Box<dyn LensResult>>` — a
  type-erased global scratch space for lens outputs. Violates Rule 1
  and recreates the variant-count problem at a different layer.
- `Port::cost: Option<u64>` — a per-Port annotation field that
  `lens_cost` writes and `emit` reads. Violates Rule 3 by bloating
  the substrate `Port` with a derived fact.
- `lens_cost.rs` side table + `lens_ownership.rs` side table where
  both internally cache "walked-arrow result for callable X" —
  violates Rule 2. Fix: extract the shared kernel.
- `Dag::inherit_cached_facts(&mut self)` — a method that walks the
  Dag and populates per-node analysis fields. Violates all three
  rules at once.

**What these rules DO allow:**

- Adding a new field to a `Declaration` or `Node` when the field
  represents an authoritative fact with a single natural grain and
  multiple consumers (per Rule 1(b)). Example: `Node::span: SourceSpan`
  (already substrate — authoritative, multi-consumer).
- Transparent memoization inside a single lens module when profiling
  shows a walk is hot, provided the memoization does not leak out of
  the lens and provided the lens remains a pure function of its
  inputs from the caller's perspective. This is a last-resort
  optimization and must be justified in the PR description.
- Shared pure-function utility modules (`visitor.rs`, `walk.rs`,
  `cost_composition.rs`) that multiple lenses call. These are not
  substrate and not lens-local state — they are reusable algorithms.

**How this interacts with §3.1.** The authority model names `Dag`,
`Declaration`, language-spec declarations, etc. as producers of
specific facts. §12.6 adds a negative constraint: lens output is NOT
a producer. If a lens's output needs to be consumed by another
component as authority, that means the lens should not have existed
as a lens — its "output" is actually a substrate fact at the wrong
layer. Remediation: identify the grain, move to substrate, delete the
lens.

**M0 reader lens compliance.** `lens_provenance` and `lens_depth`
already satisfy the invariant by construction — both are pure
functions over substrate with no internal state. They can be rewritten
as free functions (`fn provenance_for(dag: &Dag, port: PortId) -> ...`)
without loss. The "lens" terminology is a convenience for organizing
reader logic, not a commitment to a mutable analysis pass.

**PR-B compliance.** `lens_cost.rs` and `emit.rs` ship as pure
readers. Cost-per-primitive lives in `rust.dag` as a realization
field. Cost composition lives as a pure function (on-node or in a
`cost_composition.rs` helper, TBD in PR-B). Zero side tables, zero
annotation fields on `Port` or `Node`, zero storage decisions. If
PR-B cannot ship under these constraints, the design is wrong — back
up and audit what fact is missing from substrate, per Rule 1(b).

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
