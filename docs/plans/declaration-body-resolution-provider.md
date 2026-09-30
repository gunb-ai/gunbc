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
bodies; three infer readers move to the snapshot in the same commit: `infer_projection_receiver`
(field projection), `refinement_declaration` (#12407, per wise-bat-862), and
`infer_declaration_reference_facts`'s callee-Arrow read (#12506, found by calm-pike-507: it reads
`symbol_index_lookup` and gets unresolved atoms for the seven's cross-module
`parse_module_prepared(...)` call). calm-pike-507 stacks the return-type derivation on this branch. `ResolvedTree.resolved_declarations` and `resolved_declarations_of` (#12629) are deleted
in the same commit. The RFM row is retired, and its red stays enrolled.

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
