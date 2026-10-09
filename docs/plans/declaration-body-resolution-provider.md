# Declaration-body types: one resolution per ingest, produced on demand

Work item adhoc-603dcdcb-f4b (royal-stag-371). Plan only; no code lands under this document until
quiet-gull-780 and neat-boar-16 clear it. Rulings it builds to: quiet-gull-780 (demand-driven, one
resolution, not eager) and neat-boar-16 (conditions a, b and c, stated in §5).

## 1. The defect, re-derived

The brief assumed field and payload type positions reach resolve as unlowered shells. On main that
is no longer true. `v2.compiler.body_lowering_fold` already lowers record fields
(`body_lower_field_decl_edge_optional`), positional payloads (`body_lower_positional_payload`) and
alias right-hand sides (`body_lower_alias_rhs_optional`) through `body_lower_type_expr_lowered_optional`,
and #12629 adds the where-head. So `gunbc.recurring_failure_mode`
`declaration_body_type_shell_preserved_unresolved` overstates its frontier.

The earliest unjustified boundary is one link later. `v2.compiler.infer` `infer_projection_receiver`
(on the field-projection lane, #12506) reads the receiver's declaration from
`ResolvedTree.symbol_index`. That index holds declarations AS AUTHORED: it is built by
`v2.compiler.name_resolve` `resolution_context` from normalized roots, before any resolve. So
`artifact.tree` is typed as the bare atom `ParseTree`, while a parameter annotated `ParseTree`
carries the resolved reference. The match-arm mismatch that calm-pike-507 measured on #12641
follows directly.

#12629's `ResolvedTree.resolved_declarations` cannot repair it. `resolved_declarations_of(root)`
folds the SUBJECT module's declarations only, and the real seven reads `v2.compiler.parse`
`ParseArtifact.tree` from another module. On the native lane that declaring module often has no
resolve result at all. The seed main (`v1.compiler.emit_rust`, the native driver loop) resolves,
infers and evaluates one universe member at a time, in universe order rather than import order, and
the universe is test-bearing modules only.

Why a type position needs its DECLARING module: the reference resolves in that module's namespace
(its import bindings and origins). `v2.compiler.name_resolve` `namespace_for_subject_root` builds
that namespace per subject, lazily, every time the subject is resolved. It is carried nowhere.

## 2. The construction

Two produced values, both scoped to ONE INGEST and keyed by declaration identity, both on the
existing per-ingest carrier `v2.compiler.name_resolve` `ResolutionContext` (#11401 R1: one context
per native ingest, built in `v2.compiler.compile`).

**P1. The module namespace provider.**
- Identity: the module's `QualifiedName`, the key `ValidatedModuleRoots.by_name` already uses.
- Value: the admitted `Namespace` that `namespace_for_subject_root` computes today.
- Scope and retention: one ingest, meaning every module the native ingest read into the closure
  (not only universe members). Released with the context.
- Production: on first demand, by either consumer (the module's own resolve, or a declaration-body
  resolution in P2). Both read the same stored value.
- Refusal: a module outside the ingested closure refuses at the provider with a typed, located
  diagnostic. There is no per-subject rebuild and no fallback.
- `namespace_for_subject_root` stops being a builder that callers invoke and becomes the provider's
  producer. It has exactly one call site, the provider's miss arm.

**P2. The resolved declaration-body provider.**
- Identity: `std.decl_ref` `DeclarationRef`, the identity `v2.compiler.resolve`
  `resolved_reference_node` already writes into resolved trees.
- Subjects: every declaration whose TYPE POSITIONS a later stage reads -- type declarations
  (record fields, variant payloads, alias right-hand sides, where-heads) AND function signatures
  (parameter and return types), because infer reads a callee's Arrow across modules too.
- Value: the declaration's body with every type position resolved in its declaring module's P1
  namespace, by the same `resolve_node_walk` (no second resolver).
- Scope and retention: one ingest, beside P1.
- Production: on first demand. A module's own resolve DEMANDS its own type declarations and grafts
  the provided bodies instead of walking them, so no type position is ever resolved twice. After
  the walk, the module's resolve demands the transitive closure of declarations its resolved tree
  references in type positions, following resolved bodies (`ParseArtifact` then `ParseTree`, and so
  on).
- Refusal: a reference whose declaring module is outside the ingest refuses at P1, and so does a
  declaration P1's namespace does not hold.

**P2 mechanism: how "one resolution" and "the module grafts" are realized.**
- Subject grain: one top-level declaration, either a type declaration or a function SIGNATURE (the
  Arrow's parameter and return types; never a function body, which is not a declaration type).
  Identity: `DeclarationRef` (declaring module, declaration name).
- Production: ONE WALK OF THE DECLARING MODULE IN A DECLARATION-TYPES-ONLY SELECTION. The walker is
  the existing `resolve_node_walk` over the module's validated normalized root and its P1 namespace,
  and `ResolveContext` gains a selection: `WholeModule` or `DeclarationTypesOnly`. Under the second,
  value bodies are carried unwalked: an Arrow's named body edges and a `data` declaration's value.
  Every type position is walked exactly as the module's own walk would walk it, so each declaration
  gets the scope, position and `under_module_root` the full walk would give it BY CONSTRUCTION. The
  spine descent is not replicated by hand. Production grain is the module, because one selective walk
  covers all its declarations. Storage and snapshot grain is the declaration: entries are cut from
  that walk's resolved root, keyed by `DeclarationRef`. A refusal in a declaration type is stored as
  that declaration's outcome, and the grafting module surfaces it, located.
- Graft: before walking, a module's own resolve demands every type declaration and signature it
  declares from P2. The walk then carries a READ-ONLY map from occurrence id to provided node in
  `ResolveContext`, and at such a node it returns the stored node instead of descending. The walk
  itself stays pure; only the pre-walk demand threads the context.
- Snapshot: after the walk, the module's resolve demands the transitive closure of declarations its
  resolved tree references in type positions, and `ResolvedTree` carries that set as a projection of
  the provider's values (C10).

**Every loop over modules threads the context in PR2.** In PR1 a module's namespace is demanded only
by its own resolve, so two loops drop the returned context at no cost: `v2.compiler.compile`
`native_demand_execute`, the demand-engine drain that replaced the seed main's lane loop in #12401,
and `gunbc.namespace_xl2_rehearsal_census` `xl2_observe_module`. Once PR2 adds cross-module
declaration demands, a dropped context re-produces what another module already produced. So PR2
carries the context on `NativeDemandRun` and through that census fold.

**Sequencing (quiet-gull-780 decision A, 2026-09-30).** #12629, #12407, #12506 and smart-newt-725's PR
land first. PR2 then cuts over on main in one commit: every reader switch plus the deletion. P2 is
built meanwhile on a branch off PR1, not opened for merge, and never merges without its production
reader switch in the same change. Before cutting, the reader census is re-taken on main at identity
grain, because each of the four adds readers of per-module `resolved_declarations`. If one stalls
more than a day, quiet-gull-780 decides whether its reader is pulled into PR2.

**Purity: how "at most once" is realized in a pure substrate.** A provider cannot mutate. The
context is threaded as a value: every resolve returns `(ResolvedTree, ResolutionContext)`, and the
driver loop carries the returned context into the next module. The native driver loop is the seed
main rendered by `v1.compiler.emit_rust`; that template changes so the module loop binds the
context it gets back. That is a v1 change admitted under `gunbc.v1_maintenance_standing` purpose
admission, because it serves the v2 native route. Inserting under a key that is already present
refuses (`resolve_provider_entry_built_twice`), so a double build is a typed error, never a silent
overwrite.

**Infer reads a snapshot, not the provider.** `ResolvedTree` carries the P2 bodies demanded for this
module (the closure computed above). Infer stays pure and single-argument. A lookup outside that
snapshot refuses as `infer_declaration_outside_resolved_closure`; it never widens to `symbol_index`.

## 3. The PRs (stacked)

**PR1: namespace provider (P1), replacing the lazy per-subject build.** CLEARED (quiet-gull-780, 2026-09-30) to land alone as one motion: the provider replaces the lazy build, the lazy build is deleted in the same PR, and no reader can reach both. Condition (a) then governs PR2. The v1 `emit_rust` main-loop change is admitted under v1 purpose admission because it serves the v2 self-host. A root cut in one motion: after
it, "what is module X's namespace" has one producer and every caller reads it. Context threading
lands here, through `native_lane_module_resolution`, `native_test_resolve_module` and its
`_walk`, the seed main template and the stage0 mirror, `v2.compiler.compile`'s single-subject
route, and the test harnesses. Condition (a): whether PR1 may land alone, as a self-contained
replacement with no two structures, or must fold into PR2, is neat-boar-16's call. Folding it in is
mechanical.

**PR2: declaration-body provider (P2) and the reader cut, together.** P2 lands; resolve grafts P2
bodies; four infer readers move to the snapshot in the same commit: `infer_projection_receiver`
(field projection), `refinement_declaration` (#12407, per wise-bat-862), and
`infer_declaration_reference_facts`'s callee-Arrow read (#12506, found by calm-pike-507: it reads
`symbol_index_lookup` and gets unresolved atoms for the seven's cross-module
`parse_module_prepared(...)` call; its Arrow feeds `infer_application_callee_arrow_with_facts` and
`infer_arrow_declared_return_type`), and `infer_match_coproduct_of_type` with
`infer_binding_type_in_scope` (#12641, calm-pike-507: variant payload and field types for match
binders; type parameters are still substituted from use-site arguments, over the resolved body).
A fifth consumer moves in the same cut (quiet-gull-780, 2026-09-30): smart-newt-725's
record-construct field typing. It reads #12629's per-module `ResolvedTree.resolved_declarations` for
same-module records, so it must move in the commit that deletes that field; its cross-module
population is what P2 adds. smart-newt-725 names this PR as its trigger. A sixth consumer (quiet-gull-780, 2026-09-30): quick-hawk-799's gunbc#12809, a v2 `String`
literal lowered through the unfold. Its judge sees `declared = Ref(q.String)`, an imported
declaration, so it needs `q.String`'s resolved body, which #12629's root-only index lacks. It
stays draft until this PR lands. What dissolves in this
commit is #12629's `ResolvedTree.resolved_declarations` with `resolved_declarations_of`; no reader
is left on it.
All of these took declaration types off an authored or subject-local node; after PR2 none of them can
reach it. Branch: `session/royal-stag-371-pr2-declaration-bodies`. calm-pike-507 stacks the return-type derivation on this branch. `ResolvedTree.resolved_declarations` and `resolved_declarations_of` (#12629) are deleted
in the same commit. The RFM row is retired, and its red stays enrolled.

**PR3 (stacked on PR2): call-return derivation** (scope added by quiet-gull-780 when calm-pike-507's
node closed with #12641 merged into #12506's branch). `infer_arrow_declared_return_type` derives a
declared corpus return type beyond kernel and `Bool`, from the resolved callee Arrow PR2 hands
`infer_declaration_reference_facts`. As a result the seven's `parse_module_prepared(...)` is typed as
`Outcome<ParseArtifact>`. Proof: calm-pike-507's seven's-frontier claim in
`v2.test.claim.match_binder.match_binder_typing`
(`infer_match_scrutinee_type_underived` on that call shape) flips to a typed binder (C12). Concretely,
`mbt_the_sevens_call_scrutinee_is_a_counted_frontier_holds` goes red and is replaced by a positive
claim that the binder `artifact` in `mbt_seven_src` is `ParseArtifact` (`mbp_fact_of` with
`mbp_find_atom(^artifact)`, the shape of `mbt_binder_is_typed_parse_artifact_from_accepted_value_holds`).
The coproduct readers (`infer_match_coproduct_of_type`, `infer_binding_type_in_scope`,
`infer_match_field_type_instantiated`) currently leave non-parameter field types that are scoped to
their declaration underived on purpose. In PR2 they read the resolved types from the provider
instead.

## 4. Controls

| # | Control | Kind |
|---|---|---|
| C1 | calm-pike-507's one-module projection-vs-param arm (`v2.test.claim.match_binder.match_binder_typing` `mbt_match_is_typed_parse_tree_holds`) | red before PR2, green after |
| C2 | cross-module `ParseArtifact.tree`, where the declaring module is not a universe member | red before PR2, green after |
| C3 | a field whose type names an undeclared type refuses at resolve, located | red |
| C4 | a declaration demanded from a module outside the ingest refuses at P1, typed | red |
| C5 | AT MOST ONCE: two subjects that both demand module X's namespace build it once. The double-insert refusal goes red if it is built twice. | red if violated |
| C6 | IDENTITY: the `DeclarationRef` for a type in the P2 body equals the one the module's own resolved root holds at the same use | positive |
| C7 | wise-bat-862's `body_cast_node` rows 15, 15b, 15c, 15c2 and 15d stay green, and 15d, the miss, still refuses | regression |
| C8 | #12629's where-head controls (`declaration_graft_where_alias_*`), re-pointed at the P2 read | regression |
| C10 | SNAPSHOT IDENTITY: an entry in infer's per-module snapshot and the P2 provider entry it projects are the identical `DeclarationRef` and body; the snapshot is a derived projection, never re-resolved | positive |
| C11 | a cross-module callee's Arrow read by `infer_declaration_reference_facts` carries resolved parameter and return types (calm-pike-507's measurement on the stacked branch) | red before PR2, green after |
| C12 | the seven's `parse_module_prepared(...)` scrutinee types as `Outcome<ParseArtifact>`, so calm-pike-507's `infer_match_scrutinee_type_underived` frontier claim flips to a typed binder | red before PR3, green after |
| C9 | the retired RFM row's discriminating red stays enrolled as a regression control (condition c) | regression |

## 5. Conditions this plan builds to (neat-boar-16, via quiet-gull-780)

- (a) There is no moment where two structures answer the same question. The reader switch, the
  deletion of per-module `resolved_declarations`, and the deletion of the lazy per-subject namespace
  build all land in the transition that makes them redundant.
- (b) The provider has no fallback. Out-of-closure refuses at the provider. "In the ingest" means
  every module the native ingest read into the closure. The provider's identity and per-ingest scope
  are named the way #11401's `ResolutionContext` is. At-most-once is C5.
- (c) The RFM row retires in PR2, and its red stays enrolled (C9).

## 6. Cost

Baseline, taken before PR1 on `origin/main` 077af249458: `gunbc test //v2/test/claim/body_lowering/...`
on the native route, reading the per-module `[native-prepare-split]` `resolve_nanos` lines and the
`[native-cost-partition]` rows. The same pattern is re-run on each PR head. Cite the run by tag; do not
transcribe its numbers. Expected shape, not a claim: P1 moves namespace work from once per resolve to
once per module per ingest. For a declaring module that is not a universe member it is new work,
bounded by demand. Demand-scoping follows bold-bat-516's cost lane (no eager whole-closure
admission). The baseline measures the change; it does not define it.
