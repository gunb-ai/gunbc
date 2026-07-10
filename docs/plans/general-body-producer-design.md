# General body producer — the forward reading of the grammar rows

**Status:** design-first (model-before-implement). This document is the Stage-0 deliverable for the
GENERAL BODY PRODUCER milestone — the successor keystone to
[body-lowering](body-lowering-design.md), whose Stages 1–3 (the `DescentEvidence` consolidation,
node-aware Loop multiplicity, and the `04_infer` Loop arm) landed in PR #6373 together with the
first un-forked slice (`src/v2/compiler/fold_lowering.dag`, fold→`Loop` desugaring). No load-bearing
pipeline/substrate edits land in this design PR; each stage below is a separate, separately-signed
implementation PR.

Reasoned serially, per DESIGN.md's preamble: §1 fixes the gap from receipts; each later section is a
consequence, not a restatement.

---

## 1. The gap (from receipts, not assertion)

Real ingested function bodies never become substrate behaviors. The census ingest
(tokenize → `parse_module` → `normalize`) carries every body only as a **surface production spine**
— nested production Conj wrappers stamped `^grammar_production_identity_node_projection` /
`^grammar_production_captured_node_projection` (`02_parse.dag:599-619`) — and nothing downstream
turns that spine into the closed vocabulary (`Behavior = Value | Transform | Branch | Loop | Bind |
Match`, `node.dag:26-32`). Receipts:

1. **The MVP producer is structurally fixture-bound.** `03_body_producer.dag` emits the right
   canonical shapes (Arrow + `^arrow_body_edge`, Transform/Branch/Match/Bind/Loop — the seam eval
   executes), but its *reader* half accepts only the bespoke flat-token production family
   (`dag_grammar_production_fn_add/fn_pick/…`, `dag.dag:3664-3739`): `sequence_flatten_atoms`
   (`03_body_producer.dag:276-312`) rejects any production Conj wrapper (typed refusal
   `^body_producer_reason_resolved_shape`), captured slots are read by **literal flat indexes**
   (params at 5/9, operator at 15, …), every param type is hardwired `Int`, and each producer knows
   its whole body statically — no recursion into subexpressions. A real `fn_decl` from the census
   grammar (`dag_production_fn_decl`, `dag.dag:2901-2905`) cannot enter it at all.

2. **Param scoping is a scaffold that exists only because the Arrow doesn't.** Resolve pushes a
   `ScopeFrame` at fn_decl production wrappers via `scope_with_fn_decl_params`
   (`03_resolve.dag:525-535`) reading `dag_fn_decl_param_binding_atoms` — an interim reader whose
   declared dissolution trigger (`dag.dag:5612`) is precisely "body-lowering lands fn_decl → Arrow
   in normalize," making `add_arrow_domain_named_params` the single param-scoping authority.

3. **Every fact-consumer downstream is reduced to surface reads or refusals.** The
   accumulator-copy complexity lens walks the production spine because that is all there is; under
   its prove-safe-or-refuse polarity (operator ruling 2026-07-09) each missing structural fact is a
   named `Unclassifiable` refusal — `^fold_accumulator_unread`, `^copied_port_name_may_alias`,
   `^call_head_unreadable`, `^copied_port_computed_argument`, … Each refusal cause is exactly one
   missing *edge* (a binder edge, a definition edge, a resolved callee reference). Likewise the cost
   lens prices every call at `unit_cost` (`cost.dag:178-184`) because no call site carries its
   callee — a loop over `list_append` prices **linear** today even when it is quadratic.

4. **The forward direction of the grammar is modeled and unconsumed.** DESIGN.md §4 fixes "one
   grammar, read in both directions." The backward direction is fully realized: `serialize_target`
   selects `GrammarRelationRow` rows by exactly-one backward match
   (`grammar_relation_row_reverse_parse_selection`, `grammar.dag:2228-2276` — `None`/`One`/`Many`,
   ambiguity a typed refusal) and recurses token-by-token. The forward direction half-exists:
   `GrammarInterpretationDirection` with a forward selection predicate is declared
   (`grammar.dag:1669-1684`) with **zero consumers**, and 02_parse already stamps every node with
   the production identity a forward row-lookup would key on. The rows are even derivable from
   productions (`derive_grammar_relation_row`, `grammar.dag:855-860`). The missing piece is one
   fold.

**Net:** the substrate's body vocabulary, the eval protocol that executes it, the typed loop
inference that consumes it (Stages 1–3), and the row machinery that should produce it all exist;
what does not exist is the *forward reader* connecting real ingested bodies to them. Everything
between ingest and inference is stuck in the open surface spine, where mechanical unification and
decomposition (§4's decidability) cannot operate.

## 2. Negative authority — the reverted fork (what this design must not be)

PR #6373's own history contains the failed attempt, and it is the sharpest constraint. The reverted
~10k-line "cost-shape lowering" was a second, hand-rolled body producer: **one if-arm per
production** (bespoke dispatch, not row-selected), operating **pre-resolve** (its dataflow facts
were about unresolved lexemes), producing a **third body shape** that matched neither the inferred
tree nor its own test subjects (parallel-representation debt with no consumer parity), and it
**caught zero real quadratics** — no displaced cost (§6's purity trap). What survived the revert is
exactly the non-fork residue: the `04_infer` Loop arm, measure-derived `loop_multiplicity`, and
`fold_lowering.dag` as the first row-shaped slice.

Standing consequences: never a second rule table; never per-production if-arm dispatch; never a
producer that runs before names are resolved; never a body shape that is not the one eval executes.

## 3. Thesis — one fold, row-selected, over the resolved tree

The general body producer is the **forward reading of the same rows the emitter reads backward**: a
`fold_node` over the production-stamped tree where, at each Conj carrying
`^grammar_production_identity_node_projection`, the row selected by emitted identity dictates (a)
which core `NodeKind` to build and (b) how captured/bound slots map onto core edges. Structure:

- **Selection** mirrors the backward discipline exactly: exactly-one row per production identity;
  no-row and many-rows are typed, located refusals. This *consumes* the already-modeled
  `ObligationForwardDeterminism` / `ObligationSlotBijection` carriers (`grammar.dag:1673-1677`)
  instead of re-minting direction vocabulary.
- **New coverage = rows** (or productions they derive from), never fold edits — the same "N rows,
  not N×M adapters" shape as emit. The fold itself is a pure reader; language-specific rows live in
  `extdeps/languages/dag/`, core shapes in `std/`.
- **Semantic desugars stay keyed on resolved callee identity** (`fold_lowering.dag:35`):
  fold-family calls, and later recursion/`For`/`While`, all route to the same `Loop` reader —
  Stages 1–3 then type them all at once.
- **Precedence/pass-through spines dissolve in the fold** (expr/binary/unary/postfix/primary/
  block_expr wrappers produce no core node); module metadata wrappers (module_header, import_decl,
  qualified_name) are **preserved untouched** — `qualified_name_from_module_node`
  (`program_assembly.dag:91`), resolve's metadata-preserve predicate (`03_resolve.dag:483-505`),
  and 06_translate's import reads (`06_translate.dag:3725-3759`) depend on them.

The surface-form inventory maps onto the closed vocabulary as follows (the row table this design
commits to):

| Surface production | Core target |
|---|---|
| fn_decl, fn_literal | `TypeNode{Arrow}` + domain Conj of `Named{binding}` edges + `^arrow_body_edge` |
| let_expr | `ComputationNode{Bind}` (3 positional: binder, value, body) |
| if_expr | `ComputationNode{Branch}` |
| match_expr | `ComputationNode{Match}` |
| call / operator exprs | `ComputationNode{Transform}` (resolved callee reference + args); operators ground via `CanonicalOperation` |
| fold-family call | `ComputationNode{Loop}` (semantic desugar — already landed as `fold_lowering.dag`) |
| ident reference | `Atom` (binding-resolved occurrence) |
| literals | `Atom` (grounded value) |
| record construct | `TypeNode{Conj}` with named field edges |
| type_decl family | already routed by core connective (`emit_semantic_decl.dag:83-136`) |

**Honest residue** — four forms with no sound core target today; each is a **declared refusal row
with reason and dissolution trigger** (the §7 typed-frontier discipline), never a silent
pass-through or a conflated encoding:

1. **Statement sequences with discarded/effectful non-final statements.** There is no `Seq`
   behavior, and encoding "sequence" as Bind-with-ignored-binder is state-space conflation.
   Dissolves into the statement-chain grounding thread (#5587 lane), not here.
2. **Postfix field access (`.`).** The namespace-only resolution design (operator-signed
   2026-07-06) owns `.`-as-projection; the producer must not mint an independent projection
   carrier. Refuses until that lane lands its projection edge.
3. **Pattern-payload binders in match arms.** `match_arm_pattern`/`match_arm_body` edge names are
   declared (`node.dag:91-92`) but unrealized; constructor patterns with field bindings introduce
   binders the Match shape cannot yet carry. Named follow-on row.
4. **match_arm_stmt_body parse hook** (`parse_engine_hooks`, `02_parse.dag:3-7`) — a parse-side
   wart the fold consumes but must not replicate.

## 4. The output contract is fixed by eval (not negotiable per-stage)

The producer emits exactly the shapes `05_eval` executes; these are receipts, not choices:

- **Arrow:** `TypeNode{Arrow}`, domain = `children[0]` (raw index — the body edge must be appended
  after), exactly one `Named{^arrow_body_edge}` child (`find_arrow_body_child`,
  `node_query.dag:299-324`, Ambiguous/missing both fail-closed).
- **Binding:** domain param edges are `Named{binding}`; eval binds args positionally over Named
  edges only (`eval_bind_arrow_params`, `05_eval.dag:1228-1262`) under
  `EnvironmentBindingKey{ declaration: symbol_atom_node(binding) }`, and body param references
  resolve by **structural node equality** against that key (`05_eval.dag:1996-2001`) with silent
  fall-through to `allocate_literal` on miss. Consequence: the lowering **must emit
  occurrence-normalized synthetic param-ref atoms** (bare Atom, identity = binding symbol, empty
  children, `SyntheticOccurrence`) — a param ref carrying real source occurrence provenance would
  silently change semantics, a §5 fail-open. (FLAG D below names the alternative of re-grounding
  the binding key on identity; until that is ruled, the lowering conforms to the protocol as-is.)
- **Facts precondition:** every lowered node must flow through `04_infer` —
  `inferred_facts_for_eval` (`05_eval.dag:975-1001`) fail-closes on any facts-lookup miss. The
  producer cannot bypass inference.
- **Loop:** body + `Named{^loop_bound_edge}` measure registered in the descent registry, exactly
  the `fold_lowering.dag:88-96` seam — so Stage-2/3 termination and cost consumers read one edge.
- **Bounded fold:** the producer carries a subtree-count measure like the emitter's serialize
  budget (`06_translate.dag:215-225`); exhaustion is a typed diagnostic, never an unbounded walk.

## 5. Placement — in normalize, before resolve, one rule table

Placement is forced, not preferred: resolve's param-scope scaffold dissolves only if the Arrow
exists **pre-resolve** (§1 receipt 2), and normalize already owns the production-identity-keyed
rewrite precedent (`normalize_production_coproduct_fold`, `03_normalize.dag:77-112`, which lowers
`^dag_surface_type_alias_rhs` through `sugar_fold_coproduct_pipe_chain`). The census lenses already
treat "normalize terminal" as their read point, so lowering in normalize is what changes what they
see — that is the dissolution, not a side effect.

One consolidation is owed on entry: normalize today has **two** rewrite hooks — the
production-identity-keyed fold (hand-inlined, one identity) and the atom-keyed `SugarRule` table
(`sugar.dag:44-73`). Adding fn_decl as a second hand-inlined identity would widen a fork. The
design generalizes the `SugarRule` key to a coproduct (surface-atom identity | production emitted
identity) so both hooks become rows of **one table** (FLAG E), with the existing
type_alias_rhs arm migrated onto it as the proof.

## 6. Staged plan (each stage = one signed PR; strictly ordered)

The stages are denominated in **displaced cost**: each is priced by the refusal buckets it
dissolves in the accumulator-copy lens (per-cause dependency analysis, 2026-07-09) and the
scaffolds it deletes. Acceptance per stage: the target refusal-cause count goes to **zero on the
discriminating corpus while the RED controls still fire** (a genuine copy still alarms; a genuinely
unresolvable callee still refuses).

- **Stage 0 — this design PR (non-load-bearing).** This document + the DESIGN.md open-thread
  update registering the milestone. No behavior change.

- **Stage A — within-body lowering.** fn_decl → Arrow and fn_literal → Arrow (domain from real
  param lists, not hardwired `Int`/arity-2), let_expr → Bind with definition edges, calls → Transform
  with resolved heads, and the Loop **accumulator edge** (FLAG F): the seam Loop grows a third edge
  binding the step-fn's carrier binder, so the carrier read becomes a named-edge lookup.
  *Dissolves:* the resolve scope scaffold (`scope_with_fn_decl_params` + readers, per their declared
  trigger), `^fold_accumulator_unread`, `^copied_port_name_may_alias` (def-edge closure decides
  aliasing — including `let grown = acc` — and kills the shadowing false-alarm residue), and the
  misindex half of `^copied_port_argument_missing`.
  *Discipline:* the Loop edge addition must update `loop_behavior_edges_conform`,
  `loop_edge_contributes_to_iteration_fold` (`node.dag:374-387`, consumed by `cost.dag:381`), and
  canonical hashing **in one motion** — a partial update forks the well-formedness authority.
  *Landed (PR #6443, 2026-07-10) — the within-body FACT layer:* FLAG E one-table consolidation
  (`SugarKey` = surface-atom | production identity; type_alias_rhs migrated as the proof); FLAG F
  3-edge seam Loop (exactly-one-legal-name `^loop_carrier_edge`, at-most-one, excluded from the
  iteration fold in the same motion; both extra-named-edge RED controls still fire) with the seam
  widened to the whole fold family and the lens's carrier read repointed to
  `loop_carrier_binder_target` (the surface fn_literal navigation deleted — one authority);
  the let binder is lexeme-stamped via `dag_grammar_binding_name_terminal` (the Bind prerequisite);
  and the lens's definition closure decides aliasing (`let grown = acc` is now a PROVEN suspect,
  transitive; pure-literal RHS decides fresh/clean; an unbound RHS records no binding and the
  may_alias refusal stands — RED control `let_unknown_alias_still_refuses_may_alias`).
  `^fold_accumulator_unread` narrows to the named-step residue (`f: step` — a callee-resolution
  fact, Stage B). *Remaining in Stage A:* the fn_decl/fn_literal → Arrow producer rows and the
  scaffold dissolution — **gated on the namespace lane's containment `SymbolIndex`** (decl names
  bind via module-root edges / the namespace authority, not decl subtrees; a near-zero-coverage
  slice would land a dual authority without dissolving anything), staged behind the typed
  wrapper-retained frontier (lowered | wrapper-retained{cause}, counted, corpus stays green).

- **Stage B — cross-decl resolution.** Callee signatures resolve named args to declaration
  positions; qualified heads resolve through the containment tree (rides the namespace-only lane's
  `SymbolIndex`, never a parallel name index).
  *Dissolves:* `^call_head_unreadable` (refined, for higher-order heads, to a more precise
  head-resolved-to-parameter cause — a fact-gap refinement, never a widen) and the named-arg half
  of `^copied_port_argument_missing`.

- **Stage C — per-op derivation.** Run the cost lens over each callee's own lowered body and
  derive, per parameter port, whether output size is linear in that port — materialized as
  `CopiedPortFact` rows (reuse the carrier, `algebra.dag:707-710`; §2's net-concepts test). The
  hand registry becomes a citation cache; fold-family membership (`fold_family_head`, owned by
  `v2.compiler.fold_lowering` since PR #6443 — one authority for seam and lens) is likewise derived.
  *Dissolves:* `^combiner_unregistered_carrier_reaches` and the calls-inside-argument half of
  `^copied_port_computed_argument`; the closed-world refusal is **retained** for callees whose
  bodies cannot be resolved or priced.
  *Prerequisite defect, named:* the cost fold prices Arrow signature/type children into decl cost
  (no `^arrow_body_edge` filter; `node.dag:381-385` passes all non-Loop edges) — per-op rows are
  unsound until the fold prices only the body edge. Lands first inside Stage C.

- **Stage D — MVP producer subsumption.** The general fold reproduces the ~28 `produce_*` claim
  consumers' behavior **by execution** (behavioral equivalence + the existing swapped-operand RED
  controls — never byte/shape-matching the fixtures' accidents), then the bespoke flat-token
  grammar family, StampBinding tables (`dag.dag:4010-4104`), and fixture pass-throughs dissolve.

Stages A→B→C are ordered by fact dependency (each consumes the previous stage's edges); Stage D can
interleave after A. Load-bearing files throughout (`03_normalize.dag`, `03_resolve.dag`,
`04_infer.dag`, `node.dag`): higher bar, execution-proven, escalate on anything beyond the declared
scope.

## 7. Cost-model follow-on carriers (named here, built later)

The quadratic *conclusion* — replacing the lens's hand-stamped `poly_two_cost` with a derived
product — needs four carriers that are downstream of Stages A–C and are registered here so they are
rows on the frontier, not surprises:

1. **Measure-as-size-of-operand:** `LoopBound.measure` is an opaque Node; the lowering must link it
   to the folded operand (a def-edge), with a named projection separating evidence-symbol from
   size-of-operand-ref so the descent meet stays fail-closed.
2. **SizeVariable identity:** cost variables unify by def-edge identity, so nested folds over the
   same list price n·m vs n² honestly (today's degree-2 worst case is sound but conflated).
3. **Per-decl CostSignature substitution at call sites:** callee cost over param-size variables,
   substituted with argument sizes — this is what makes a loop over `list_append` price quadratic.
4. **Accumulator size-recurrence:** "size(carrier) at iteration i is Θ(i)" — genuinely new; reads
   the Stage-A accumulator edge + body-result carry, priced fail-closed (`UnknownCost` on any
   unresolvable recurrence, never a fabricated degree).

## 8. Fail-closed guarantees (§5)

- An unlowerable body form is a **typed, located, countable refusal row** (the
  `^fold_lowering_shape_invalid` pattern) — never a silent spine pass-through, never an absorbing
  widen. The residue table in §3 is the complete declared frontier at Stage-0.
- Row selection is exactly-one; None and Many are distinct typed refusals (the backward
  discipline's mirror).
- The lens polarity is preserved end-to-end: a refusal cause may only dissolve by a **fact that
  decides it** — never by defaulting clean. Every zeroed bucket ships with a RED control that
  still fires.
- No escape hatches: there is no mode in which the producer "proceeds as if" a refusal had not
  fired; diagnostic replay reads state only.

## 9. Flags — SIGNED (operator, 2026-07-10)

- **FLAG D — binding-key protocol. SIGNED: conform now.** Eval resolves param refs by structural
  node equality against `symbol_atom_node(binding)`; the lowering conforms by emitting
  occurrence-normalized synthetic param refs. The identity re-grounding of `EnvironmentBindingKey`
  (which also deletes the silent `allocate_literal` fall-through, a §5 fail-open the current
  protocol carries) is **required cleanup before this feature closes** — operator condition on the
  sign, carried in the dissolution trigger below, not an optional follow-on.
- **FLAG E — one rule table. SIGNED: one table.** Generalize the `SugarRule` key to (surface-atom
  | production identity) and migrate the existing type_alias_rhs hook onto it as the proof; no
  sibling hand-inlined arm.
- **FLAG F — Loop accumulator edge. SIGNED: 3-edge shape.** The seam Loop grows body + measure +
  carrier binder; `LoopBoundEdges` conformance, the iteration-fold edge predicate, and canonical
  hashing update in one motion. Termination and cost consumers unaffected (they read the measure
  edge, unchanged).

## Dissolution trigger (DESIGN §6)

This document dissolves when BOTH hold: (1) Stage D lands — the general producer subsumes
`03_body_producer.dag`'s fixture family and the lens's within-body/cross-decl refusal buckets are
zeroed with live RED controls; and (2) the FLAG D binding-key re-grounding lands —
`EnvironmentBindingKey` keyed on binding identity and the silent `allocate_literal` fall-through
deleted (the feature does not close over the conform-time protocol; operator condition,
2026-07-10). At that point the surviving content (the §7 cost carriers) migrates to its own
follow-on design and this file is deleted.
