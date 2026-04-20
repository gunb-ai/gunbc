# SG-4a Deliverable A — `infer.rs` Authority Map

Function-by-function classification of `src/v3/compiler/src/infer.rs`
(4344 LOC, 76 functions). Produced by SG-4a (lane
`fierce-wolf-119`, branch `session/fierce-wolf-119`).

**Snapshot.** Line numbers below are pinned to `infer.rs` at commit
`05616d166` (the commit that introduced this document). Any edit to
`infer.rs` will drift them; re-anchor before relying on them.

Categories (per `docs/briefs/sg-4a-infer-foundation.md`):

- **Cat-1 Structural reader** — pure function over typed substrate
  facts → typed output. SG-4b candidate for direct `.dag` port.
- **Cat-2 Helper logic** — list manipulation, scope threading,
  substitution mechanics. SG-4b candidate for helper-level generation
  matching `infer_helpers_generated.rs`.
- **Cat-3 Imperative state machine** — mutates `Dag`, threads
  diagnostics through local state. SG-4c territory; may need substrate
  extension.
- **Cat-4 Cross-stage glue** — calls lower, emits diagnostics,
  populates `emit_anchors`. Residual Rust shim likely.

## STOP-and-escalate summary

**Revised reading (post-review).** The initial draft triggered the
brief's §STOP clause 1 ("Cat-3 > 40% → substrate design problem") on
a 47.4% Cat-3 share. Both briansrls and codex flagged that reading as
miscategorized: the state the Cat-3 functions thread (argument vectors
in unification, predicate-body clone machinery) is **local per-walk
state**, not Dag facts and not cross-pass consumption. Under the layer
model, local traversal state is implementation work inside the future
`infer.dag` / `regen_infer` scope, not a substrate-expressivity gap.

Revised reading:

- **Cat-3 = 47.4% of classified LOC** (re-baselined — see denominator
  fix below). This is *volume* of Dag-mutating work, not substrate
  breakage. The 40% threshold in the brief was about substrate
  expressivity; since the gaps below are now re-classified as
  implementation, the threshold is not tripped in its original sense.
  SG-4b dispatch is a regen-scope question, not a substrate-extension
  question.
- **Remaining genuinely substrate-shaped concerns: 0–1** (see revised
  §gap enumeration). Most "gaps" in the initial draft were local
  implementation that regen machinery handles.
- **Two ambiguous/high-risk functions** flagged (see end of this doc):
  `bind_expected_decl_to_actual_context` (184 LOC),
  `resolve_callable_target` (216 LOC). Pure/stateful boundary
  clarification is still the real blocker for Cat-3 porting — but
  again, this is regen-design work, not substrate.

Deliverable B (prototype) and Deliverable C (ROADMAP rows) remain
**parked** — but the parking rationale has narrowed: the decision is
now "what's the SG-4b regen shape?" rather than "do we need to extend
substrate?"

## Summary table

| Category | Count | LOC | % of classified LOC |
|----------|-------|-----|---------------------|
| Cat-1 (Structural reader) | 29 | 1,253 | 31.7% |
| Cat-2 (Helper logic) | 25 | 701 | 17.7% |
| Cat-3 (Imperative state machine) | 20 | 1,872 | **47.4%** |
| Cat-4 (Cross-stage glue) | 2 | 127 | 3.2% |
| **Total** | **76** | **3,953** | **100%** |

Denominator is classified LOC (3,953); header/comment LOC + trivial
trailing helpers account for the delta to the 4,344 total. Initial
draft divided by 4,344 while labeling the column "classified LOC" —
corrected here per codex-review observation.

## Function classifications

### Entry / fixpoint

#### `infer` (L41, 129 LOC) — Cat-3
- `&mut Dag → ()`; fixpoint driver calling `decide` per node plus the
  seven resolution passes.
- No substrate gap — Dag mutation is native.
- Risk: low.

### Pattern / payload resolution

#### `walk_to_disj_decl` (L170, 23 LOC) — Cat-1
Pure structural walk Instantiation/ResolvedIdentifier → Disj. Risk: low.

#### `existing_optional_match_disj_decl` (L193, 7 LOC) — Cat-2
Reads `optional_match_disj` memo. Risk: low.

#### `walk_to_optional_cardinality_decl` (L200, 17 LOC) — Cat-1
Parallel to `walk_to_disj_decl`, targets Cardinality(AtMostOne). Risk: low.

#### `ensure_optional_match_disj` (L217, 94 LOC) — Cat-3
Allocates 3 synthetic declarations (Some-payload, None-payload, Disj)
and memoizes via `set_optional_match_disj`. Straightforward declaration
construction; no substrate gap. Risk: low.

#### `resolve_branch_patterns` (L311, 217 LOC) — Cat-3
Two-phase: collect path rewrites matching BranchPattern names against
Disj variants; apply rewrites + coverage checks. Pure Dag-side
rewriting. Risk: low.

#### `resolve_branch_payload_bindings` (L528, 160 LOC) — Cat-3
Parallel to pattern resolution for payload-binding types. Threads
`SubstStack` (Rust-side). Implementation note: local walk state; not
a substrate gap (see triage §). Risk: medium.

#### `payload_binding_span` (L688, 13 LOC) — Cat-2
Span-extraction utility. Risk: low.

### Decision dispatcher

#### `decide` (L701, 150 LOC) — Cat-3
Dispatches on Value/Transform/Branch/Bind/Loop. Returns
`Decision::{Set,Fail,Retry}`. Risk: low.

#### `decide_transform` (L851, 237 LOC) — Cat-3
Dispatches on TransformTarget (FieldProject/Operator/Callable); threads
diagnostics on mismatch. No substrate gap. Risk: low.

### Refinement discharge

#### `check_refinement_discharge` (L1088, 49 LOC) — Cat-2
Calls `predicate_discharges`; returns diagnostic on failure. Risk: low.

#### `predicate_discharges` (L1137, 48 LOC) — Cat-1
Structural discharge: identity OR walk-equal OR conjunct subset. Risk: low.

#### `body_discharges` (L1185, 30 LOC) — Cat-1
Flatten-and-subset over conjunct leaves. Risk: low.

#### `collect_conjunct_leaves` (L1215, 21 LOC) — Cat-2
Recursive unfold of Transform(Logical(And)). Risk: low.

#### `predicate_info` (L1236, 15 LOC) — Cat-2
Extract (param, body) ports from predicate Arrow. Risk: low.

#### `refinement_ports_equal` (L1251, 35 LOC) — Cat-1
Walk-equality with param-pair substitution. Risk: low.

#### `refinement_targets_equal` (L1286, 117 LOC) — Cat-1
Structural equality dispatch on TransformTarget. Risk: low.

### Template argument plumbing

#### `declaration_is_callable` (L1403, 22 LOC) — Cat-1
Walk Instantiation/Identifier → Arrow or Atom(TypeParam). Risk: low.

#### `is_retryable_generic_decl` (L1425, 4 LOC) — Cat-2
Depth-0 wrapper. Risk: low.

#### `is_retryable_generic_decl_walk` (L1429, 28 LOC) — Cat-1
Unbound-TypeParam/Instantiation walk. Risk: low.

#### `callable_template_arguments` (L1457, 13 LOC) — Cat-2
Extract (template id, args) from Instantiation. Risk: low.

#### `template_argument_value` (L1470, 10 LOC) — Cat-2
Lookup argument by parameter id. Risk: low.

#### `resolve_template_argument_value` (L1480, 17 LOC) — Cat-2
Recursive lookup with cycle detection. Risk: low.

#### `retained_template_arguments_for_target` (L1497, 40 LOC) — Cat-2
Filter args to those referenced by target Arrow. Risk: low.

#### `template_arguments_match` (L1537, 8 LOC) — Cat-2
Equal-length pairwise match. Risk: low.

#### `push_template_argument_binding` (L1545, 18 LOC) — Cat-2
Add-or-update binding; fails on conflict. Risk: low.

#### `resolve_arrow_decl_walk` (L1563, 40 LOC) — Cat-1
Walk → Arrow, pushing Instantiation args onto SubstStack. Risk: low.

#### `literal_decl_id` (L1603, 9 LOC) — Cat-2
Literal → primitive declaration id. Risk: low.

#### `port_type_context` (L1612, 71 LOC) — Cat-1
Build PortTypeContext (declaration + SubstStack) for a resolved port.
Risk: low.

#### `resolve_binding_decl` (L1683, 24 LOC) — Cat-1
Walk a declaration through substitution. Risk: low.

#### `callable_signature_context` (L1707, 46 LOC) — Cat-1
Extract signature context from callable declaration. Risk: low.

### Unification / callable binding (danger zone)

#### `bind_expected_callable_to_actual` (L1753, 40 LOC) — Cat-3
Unifies expected callable against actual declaration. Mutates args vec.
Implementation note: per-call unification state; not substrate. Risk: medium.

#### `bind_expected_decl_to_actual_context` (L1793, 184 LOC) — Cat-3 (**HIGH-RISK**)
184-line recursive unification over TypeConnective. Mutates args vec
across recursive calls. **Ambiguous**: pure unification or stateful
binding? Distinction matters for regen codegen shape, not substrate.
Risk: **high**.

#### `callable_instantiation_conflict` (L1977, 21 LOC) — Cat-2
Format diagnostic. Risk: low.

#### `resolve_callable_target` (L1998, 216 LOC) — Cat-3 (**HIGH-RISK**)
216-line heavyweight resolver. Calls both unification helpers plus
`check_refinement_discharge` and `resolve_decl_with_subst`. Multiple
return paths, complex state threading.
**Ambiguous** — decomposition into pure/impure layers unclear; this is
a regen-design clarification, not a substrate gap. Risk: **high**.

#### `resolve_direct_target_signature` (L2214, 32 LOC) — Cat-1
Walk target + template args → ResolvedArrow. Risk: low.

### Fixpoint passes

#### `resolve_callable_targets` (L2246, 74 LOC) — Cat-3
Collect-then-rewrite over Transform nodes. Risk: low.

#### `materialize_callable_signature_instantiations` (L2320, 31 LOC) — Cat-3
Allocate fresh Arrow instantiation declarations. Risk: low.

#### `resolve_lambda_parameter_types` (L2351, 84 LOC) — Cat-3
Infer param types from refinement / outer constraints. Implementation
of a rule over Dag facts; not a substrate gap. Risk: medium.

#### `validate_user_defined_function_signatures` (L2435, 168 LOC) — Cat-3
Walks Arrow bodies checking param/output types; marks Unresolved on
mismatch. Local validation vector; implementation concern. Risk: medium.

### Non-callable target arguments

#### `bind_non_callable_target_arguments` (L2603, 65 LOC) — Cat-2
Pure structural extraction of expected argument types. Risk: low.

### Subst-stack walks

#### `walk_to_conj_decl_with_subst` (L2668, 30 LOC) — Cat-1
Push Instantiation args onto SubstStack while walking. Risk: low.

#### `walk_to_disj_decl_with_subst` (L2698, 34 LOC) — Cat-1
Same pattern, Disj. Risk: low.

#### `enclosing_disj_for_variant` (L2732, 12 LOC) — Cat-2
Reverse lookup variant → parent Disj. **Only remaining candidate
substrate concern** — Dag lacks reverse parent link; current impl is
O(n). Low priority; doesn't block SG-4b. Risk: medium.

### Payload materialization

#### `resolve_payload_binding_type` (L2744, 48 LOC) — Cat-2
Resolve variant payload type; handles record specialization. Risk: low.

#### `materialize_specialized_payload_record` (L2792, 36 LOC) — Cat-3
Fresh Conj specializing a template variant. Risk: low.

### Concretization

#### `concretize_decl_with_subst` (L2828, 102 LOC, **pub(crate)**) — Cat-3
Walk + materialize Instantiation/Cardinality declarations.
Deduplicates via `find_equivalent_anonymous_*` helpers. Risk: low.

#### `refinement_base_requires_substitution` (L2930, 15 LOC) — Cat-1
Gate check. Risk: low.

#### `refinement_base_walk` (L2945, 41 LOC) — Cat-1
Unbound-TypeParam/Instantiation walk. Risk: low.

#### `materialize_substituted_refined_decl` (L2986, 161 LOC) — Cat-3 / Cat-4 border
DB-16 closure. Calls `outer_predicate_slots`, `clone_predicate_body`
(both in `lower.rs`). Clone helpers are pure structural Dag-reads;
co-location in `lower.rs` is organizational, not a true cross-stage
boundary. Port as shared `.dag` utilities. Risk: **high** (volume).

#### `find_equivalent_substituted_refined_decl` (L3147, 68 LOC) — Cat-1
Deduplication scan. Risk: low.

#### `predicate_bodies_equal_under_subst` (L3215, 63 LOC) — Cat-1
Walk-equality under differing SubstStacks. Risk: low.

#### `transform_targets_equal_under_subst` (L3278, 50 LOC) — Cat-1
Dispatch on TransformTarget with per-context substitution. Risk: low.

#### `callable_decls_equal_under_subst` (L3328, 65 LOC) — Cat-1
Walk-equal declarations under substitution. Risk: low.

#### `normalized_instantiation_args` (L3393, 22 LOC) — Cat-2
Resolve args through substitution chains. Risk: low.

#### `find_equivalent_anonymous_conj` (L3415, 17 LOC) — Cat-1
Table scan. Risk: low.

#### `find_equivalent_anonymous_instantiation` (L3432, 26 LOC) — Cat-1
Table scan. Risk: low.

#### `find_equivalent_anonymous_cardinality` (L3458, 29 LOC) — Cat-1
Table scan. Risk: low.

### Field projection

#### `resolve_field_project` (L3487, 94 LOC) — Cat-2
Walk input → Conj; find field; resolve output type. Risk: low.

#### `decide_field_project` (L3581, 13 LOC) — Cat-3
Thin dispatcher. Risk: low.

#### `resolve_field_project_targets` (L3594, 84 LOC) — Cat-3
Fixpoint rewrite pass. Risk: low.

### Operator arrow

#### `resolve_operator_arrow` (L3678, 87 LOC) — Cat-1
Algebra Conj lookup OR primitive mono fallback. Risk: low.

#### `strip_refinement_to_base` (L3765, 28 LOC) — Cat-1
Walk Atom(ResolvedIdentifier) skipping refinement. Risk: low.

#### `read_algebra_field` (L3793, 53 LOC) — Cat-1
Extract algebra operator field with receiver substitution. Risk: low.

#### `substitute_receiver` (L3846, 54 LOC) — Cat-2
Substitute receiver into formal parameter. Risk: low.

#### `is_realization_shape` (L3900, 14 LOC) — Cat-1
Realization meta_tag check. Risk: low.

### Final walks / equivalence

#### `resolve_arrow_walk` (L3914, 69 LOC) — Cat-1
Walk to Arrow with substitution; filter callable parameters. Risk: low.

#### `walk_to_type_shape` (L3983, 51 LOC) — Cat-1
Declaration → TypeShape under substitution. Risk: low.

#### `signature_type_shape` (L4034, 68 LOC) — Cat-1
Signature component → TypeShape. Risk: low.

#### `resolve_decl_with_subst` (L4102, 53 LOC) — Cat-1
Walk declaration through substitution. Risk: low.

#### `find_equivalent_decl_instantiation` (L4155, 23 LOC) — Cat-1
Table scan. Risk: low.

#### `find_equivalent_decl_cardinality` (L4178, 20 LOC) — Cat-1
Table scan. Risk: low.

### Utility

#### `target_display_name` (L4198, 18 LOC) — Cat-2
Risk: low.

#### `transform_target_display_name` (L4216, 8 LOC) — Cat-2
Risk: low.

#### `node_span_for_port` (L4224, 6 LOC) — Cat-2
Risk: low.

#### `synthetic_span` (L4230, 4 LOC) — Cat-2
Risk: low.

#### `type_shapes_equivalent` (L4234, 7 LOC) — Cat-1
Risk: low.

#### `declaration_shapes_equivalent` (L4241, ~100 LOC) — Cat-1
Final structural-equivalence helper. Risk: low.

## Candidate set for Deliverable B prototype

Three orthogonal groups of Cat-1/Cat-2 functions. Total **643 LOC ≈
14.8% of `infer.rs`**, within the 10–20% target.

**Group 1 — Pure walkers (264 LOC).** `walk_to_disj_decl` (23),
`walk_to_optional_cardinality_decl` (17), `walk_to_conj_decl_with_subst`
(30), `walk_to_disj_decl_with_subst` (34), `resolve_arrow_decl_walk`
(40), `walk_to_type_shape` (51), `resolve_arrow_walk` (69).

**Group 2 — Template argument helpers (128 LOC).**
`callable_template_arguments` (13), `template_argument_value` (10),
`resolve_template_argument_value` (17),
`retained_template_arguments_for_target` (40),
`template_arguments_match` (8), `push_template_argument_binding` (18),
`normalized_instantiation_args` (22).

**Group 3 — Refinement/discharge (251 LOC).** `predicate_discharges`
(48), `body_discharges` (30), `collect_conjunct_leaves` (21),
`refinement_ports_equal` (35), `refinement_targets_equal` (117).

Each group can be ported independently and tested against the
existing `infer_helpers_generated.rs` pattern (single-module .dag →
`emit_rust_module` → committed `*_generated.rs` read from `infer.rs`).

## Triage — substrate gaps vs. implementation work

Post-review, applying the layer-model filter: **a concern is
substrate-level only if the fact it represents lives on the Dag or is
consumed across a pass boundary.** Per-walk local state threaded
through recursive Rust calls is implementation work inside future
`infer.dag` (handled by regen machinery), not substrate work.

### Re-classified as implementation (not substrate gaps)

- **Unification argument-vector threading.** Consumers:
  `bind_expected_callable_to_actual`,
  `bind_expected_decl_to_actual_context`, `resolve_callable_target`.
  The `Vec<TemplateArgument>` is per-call scoped; it never becomes a
  Dag fact and no downstream pass consumes it. Port into `infer.dag`
  as locally-scoped collection-accumulator patterns when regen
  machinery supports them. **Not a substrate extension.**

- **Predicate-body cloning helpers.** Consumer:
  `materialize_substituted_refined_decl` via
  `lower.rs::{clone_predicate_body, outer_predicate_slots}`. The
  "cross-stage" framing in the initial draft was wrong: these helpers
  are pure structural Dag-reads that happen to live in `lower.rs` as a
  code-organization choice. The fact produced (a substituted refined
  declaration) already lives on the Dag via
  `materialize_substituted_refined_decl`'s push. Port the clone
  helpers as shared `.dag` utilities (mirroring the
  `lenses/infer_helpers.dag` pattern) when porting the caller.
  **Not a substrate extension.**

- **Refinement-derived lambda parameter typing**
  (`resolve_lambda_parameter_types`). Reads predicate structure and
  writes port types. All inputs and outputs are Dag facts; the
  derivation rule is implementation. **Not a substrate extension.**

- **Signature-validation failure-list threading**
  (`validate_user_defined_function_signatures`). Local failure vector
  is a regen-codegen concern. Pattern already used for branch pattern
  resolution — same collect-then-rewrite shape. **Not a substrate
  extension.**

### Remaining candidate substrate concern (1, low-priority)

- **Variant → parent Disj reverse link.** Consumer:
  `enclosing_disj_for_variant`. Dag currently lacks a direct
  reverse-parent lookup; scan is O(n). This *is* a Dag-structure
  question: does the substrate carry parent edges for variants?
  - Extension shape: `variant_parent: DeclarationId` field on variant
    declarations, or Dag-level reverse map.
  - Dissolution trigger: Dag exposes parent edges for variants.
  - **Priority: low.** Doesn't block SG-4b; O(n) is acceptable at
    current Dag sizes. Noted for future optimization.

### Net result

Initial draft claimed 2 critical substrate blockers + 3 minor. After
re-grounding: **0 blockers, 1 nice-to-have.** SG-4b dispatch is
**not** gated on substrate extension. The real gating question is the
regen-codegen shape for the high-Cat-3 volume (see handoff section).

## High-risk function detail

### `bind_expected_decl_to_actual_context` (L1793, 184 LOC) — HIGH

184-line recursive unification over TypeConnective (TypeParam, Atom,
Instantiation, Conj, Disj, Cardinality). Mutates `args` vec across
recursive calls. State threading through multiple unification goals
makes active binding assumptions hard to track at each branch. The
critical question: is this pure unification (Cat-1 candidate) or
stateful binding (Cat-3 requiring substrate extension)?

### `resolve_callable_target` (L1998, 216 LOC) — HIGH

216-line heavyweight resolver. Calls both
`bind_expected_callable_to_actual` (callable args) and
`bind_expected_decl_to_actual_context` (non-callable args). Also calls
`check_refinement_discharge` and `resolve_decl_with_subst`. Multiple
return paths, complex unification threading across two binding
helpers. Decomposition into pure/impure layers is the blocker for
Cat-3b dispatch.

## Handoff — decision required

Revised post-review: Cat-3 at 47.4% of classified LOC is *volume*, not
substrate breakage. Substrate expressivity is not the gating question.
The gating question is the regen-codegen shape for Cat-3: how does
`regen_infer` handle local mutable-vector threading, recursive
unification, and Dag-mutation fixpoint passes?

Three possible handoffs:

1. **Dispatch SG-4b with regen_infer covering Cat-3 as implementation.**
   Full cutover; `regen_infer` machinery grows to handle Cat-3 patterns
   (locally-scoped accumulators, shared `.dag` helpers cloned from
   `lower.rs`, fixpoint passes). No substrate lane needed. Scope is
   XXL (~3,950 LOC port) and requires regen-machinery design work.
2. **Cat-1/2 only as SG-4b-1.** Dispatch Deliverable B's 643 LOC; treat
   Cat-3 as residual Rust shim indefinitely. Accepts ~53% of `infer.rs`
   staying hand-written. Smallest viable win; preserves optionality.
3. **Hold SG-4 pending regen-machinery design.** Pause until a written
   `regen_infer` spec covers the Cat-3 patterns; then re-dispatch as
   option 1.

Deliverable B (prototype) and Deliverable C (ROADMAP rows) remain
parked. Note that C's scope collapses under the revised triage: the
only carry-forward substrate candidate is variant→parent reverse
lookup, low-priority.

## Revision trail

- `05616d166` — initial draft (6 substrate gaps, STOP tripped on 43.1%
  Cat-3).
- `3a39eee30` — snapshot note + dropped self-described non-gap.
- **current** — substrate-vs-implementation triage applied per review;
  gaps re-classified as implementation work inside regen scope;
  percentages corrected (denominator was total file LOC, not
  classified LOC); STOP framing revised; handoff options rewritten
  around regen shape rather than substrate extension.
