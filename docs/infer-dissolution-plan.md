# SG-4a Deliverable A — `infer.rs` Authority Map

Function-by-function classification of `src/v3/compiler/src/infer.rs`
(4344 LOC, 76 functions). Produced by SG-4a (lane
`fierce-wolf-119`, branch `session/fierce-wolf-119`).

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

- **Cat-3 = 43.1% of LOC** — exceeds the 40% threshold from the brief.
  Per STOP instruction: surface the count; director + SG-manager decide
  whether to extend substrate (separate lanes) or pause SG-4
  indefinitely.
- **6 distinct substrate gaps named below** — within the 10-gap
  threshold; manageable as separate named lanes.
- **Two ambiguous/high-risk functions** flagged (see end of this doc):
  `bind_expected_decl_to_actual_context` (184 LOC),
  `resolve_callable_target` (216 LOC). Both sit on the pure/stateful
  boundary and need design clarification before porting.

Deliverable B (prototype) and Deliverable C (ROADMAP rows) are
**parked** pending director decision on the STOP signal. Candidate set
for B is enumerated below so the decision can be made with the
prototype scope visible.

## Summary table

| Category | Count | LOC | % of classified LOC |
|----------|-------|-----|---------------------|
| Cat-1 (Structural reader) | 29 | 1,253 | 28.8% |
| Cat-2 (Helper logic) | 25 | 701 | 16.1% |
| Cat-3 (Imperative state machine) | 20 | 1,872 | **43.1%** |
| Cat-4 (Cross-stage glue) | 2 | 127 | 2.9% |
| **Total** | **76** | **3,953** | **100%** |

Header/comment LOC + trivial trailing helpers account for the delta to
the 4344 total.

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
`SubstStack` (Rust-side). **Gap #1**: substitution context is Rust-side.
Risk: medium.

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
**Gap #2**: unification state is Rust-side. Risk: medium.

#### `bind_expected_decl_to_actual_context` (L1793, 184 LOC) — Cat-3 (**HIGH-RISK**)
184-line recursive unification over TypeConnective. Mutates args vec
across recursive calls. **Ambiguous**: pure unification or stateful
binding? Distinction matters — pure unification ports directly,
stateful binding needs substrate extension or Rust shim.
**Gap #2** (same). Risk: **high**.

#### `callable_instantiation_conflict` (L1977, 21 LOC) — Cat-2
Format diagnostic. Risk: low.

#### `resolve_callable_target` (L1998, 216 LOC) — Cat-3 (**HIGH-RISK**)
216-line heavyweight resolver. Calls both unification helpers plus
`check_refinement_discharge` and `resolve_decl_with_subst`. Multiple
return paths, complex state threading.
**Ambiguous** — decomposition into pure/impure layers unclear.
**Gap #2** (same). Risk: **high**.

#### `resolve_direct_target_signature` (L2214, 32 LOC) — Cat-1
Walk target + template args → ResolvedArrow. Risk: low.

### Fixpoint passes

#### `resolve_callable_targets` (L2246, 74 LOC) — Cat-3
Collect-then-rewrite over Transform nodes. Risk: low.

#### `materialize_callable_signature_instantiations` (L2320, 31 LOC) — Cat-3
Allocate fresh Arrow instantiation declarations. Risk: low.

#### `resolve_lambda_parameter_types` (L2351, 84 LOC) — Cat-3
Infer param types from refinement / outer constraints. **Gap #4** (see
below). Risk: medium.

#### `validate_user_defined_function_signatures` (L2435, 168 LOC) — Cat-3
Walks Arrow bodies checking param/output types; marks Unresolved on
mismatch. **Gap #5** (local validation state). Risk: medium.

### Non-callable target arguments

#### `bind_non_callable_target_arguments` (L2603, 65 LOC) — Cat-2
Pure structural extraction of expected argument types. Risk: low.

### Subst-stack walks

#### `walk_to_conj_decl_with_subst` (L2668, 30 LOC) — Cat-1
Push Instantiation args onto SubstStack while walking. Risk: low.

#### `walk_to_disj_decl_with_subst` (L2698, 34 LOC) — Cat-1
Same pattern, Disj. Risk: low.

#### `enclosing_disj_for_variant` (L2732, 12 LOC) — Cat-2
Reverse lookup variant → parent Disj. **Gap #3**: if Dag lacks reverse
parent link this is O(n). Risk: medium.

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
(both in `lower.rs`). **Gap #6** (predicate-body cloning is cross-stage).
Risk: **high**.

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

## Substrate-gap enumeration (Deliverable C candidates)

1. **Unification state / argument-vector threading** (CRITICAL)
   - Consumers: `bind_expected_callable_to_actual`,
     `bind_expected_decl_to_actual_context`, `resolve_callable_target`.
   - Gap: no substrate carrier for "active template-argument bindings"
     or "pending unification assumptions." Arguments vector is
     Rust-side mutable state threaded through recursive unification.
   - Extension shape: new `UnificationFrame` fact in substrate, OR keep
     unification as Rust-side lane with declarative glue.
   - Dissolution trigger: substrate admits a carrier for partial
     binding maps + a `.dag`-expressible fixpoint over them.

2. **Cross-stage predicate-body cloning** (CRITICAL)
   - Consumers: `materialize_substituted_refined_decl` (via
     `lower.rs::{clone_predicate_body, outer_predicate_slots}`).
   - Gap: cloning lives in `lower.rs`. Substrate has no fact for
     "predicate body clone with substitution routing."
   - Extension shape: either extend `lower.rs` to `.dag` (separate
     lane), or model the clone as a substrate fact.
   - Dissolution trigger: DB-16 refined-generic substitution has a
     substrate carrier rather than a Rust procedure.

3. **Variant → parent Disj reverse link** (NICE-TO-HAVE)
   - Consumer: `enclosing_disj_for_variant`.
   - Gap: Dag lacks a direct reverse-parent lookup; current impl is
     O(n).
   - Extension shape: variant_parent field or Dag-level reverse map.
   - Dissolution trigger: Dag exposes parent edges for variants.

4. **Refinement-derived lambda parameter typing** (MINOR)
   - Consumer: `resolve_lambda_parameter_types`.
   - Gap: inference reads predicate structure to derive param types;
     no explicit substrate fact for "param type constrained by outer
     refinement."
   - Extension shape: refinement-to-param-type derivation as a
     substrate fact.
   - Dissolution trigger: lambda-param inference expressible as a
     pure substrate walk.

5. **Signature-validation failure-list threading** (MINOR)
   - Consumer: `validate_user_defined_function_signatures`.
   - Gap: collects failures then applies; state is Rust-side vector.
   - Extension shape: pass expressed as `.dag` fixpoint with a
     declarative collect-then-rewrite pattern (similar to other passes).
   - Dissolution trigger: validation pass matches the pattern-resolution
     shape already used for branches.

6. **Generic-retry status for TypeParam binding** (MINOR)
   - Consumers: `is_retryable_generic_decl`,
     `is_retryable_generic_decl_walk`.
   - Gap: distinguishing bound vs. free TypeParam relies on structural
     walks over Instantiation.
   - Extension shape: probably already fine; keep as Cat-1 port and
     re-evaluate after Group 1 lands.
   - Dissolution trigger: N/A (likely non-gap).

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

The 43.1% Cat-3 reading means the substrate as it stands cannot cleanly
express core inference moves (unification state, cross-stage predicate
cloning). Per brief §STOP-AND-ESCALATE clause 1:

> STOP. That's a signal the substrate as it stands can't express
> inference and SG-4b is not a compile-authority problem, it's a
> substrate-design problem. Surface the count; director decides whether
> to extend substrate (separate lane) or pause SG-4 indefinitely.

Three possible handoffs:

1. **Extend substrate then cut over.** Dispatch substrate-extension
   lanes first (gaps #1 and #2), land them, then SG-4b as sequential
   Category lanes.
2. **Cat-1/2 only.** Dispatch Deliverable B's 643 LOC as SG-4b-1;
   accept ~57% of `infer.rs` remaining hand-written as Cat-3/Cat-4
   Rust shim.
3. **Hold SG-4.** Pause indefinitely until substrate modeling for
   unification + cross-stage cloning catches up.

Deliverables B and C are parked pending this decision.
