# Namespace-only resolution — the graph is the sole naming authority

**Goal:** one naming authority with no silent rebinding. **Method:** build one containment tree, resolve uniquely on the ancestor chain, project downward and derive dependencies from resolved references. **Validation:** exact target identity, typed unresolved/ambiguous controls, shared runtime dispatch and removal of import machinery under the current namespace-cut stage contract.

Operator rulings below supersede earlier proposals explicitly. §13 replaces nearest-wins and expected-type pool filtering; its later no-ambient-prelude ruling bounds current native scope. Current cut sequencing comes from `gunbc.namespace_cut.namespace_cut_landing_order.current_landing_order` and `namespace_cut_stage.stage_prerequisites`, not this document’s historical per-subtree schedule. [Namespace-cut execution](namespace-cut-replacement-plan.md) carries the cut list. [Pinned history](https://github.com/gunb-ai/gunbc/blob/06d3c1c3149a7420c4a548b9aba9b15c721c635d/docs/plans/namespace-resolution-design.md) retains the dated raw censuses and superseded derivations.

## 1. The two governing rules

Operator, 2026-07-06: minimal representation, with no dual representation at any layer; no ambiguity at any layer. This supersedes the import-name-universe half of `resolver-graph-major-design` §1c while retaining binding-edge constructor ownership and forbidding expected-type owner-picking.

## 2. What the rules force (the derivation)

References already express usage, so authored import lists and visibility maps duplicate it. Derive dependency edges from exact targets. Reachability follows containment; redundant qualification is a lint, never a reason to guess. More than one eligible binding produces a typed, located `Ambiguous(candidates)`; no eligible binding produces `Unresolved`.

## 3. The model — ONE structure: the containment tree (operator, 2026-07-06)

Module/namespace nesting, fields, variants and local binders inhabit one tree. Position gives qualified identity (`T.a` differs from `T.a.a`); `.` descends one child for field, module and variant projection alike. This strict sub-value edge also supplies `std.induction.SubValueRelation` descent evidence. The frontend materializes nesting without resolution policy; the backend walks upward for the first segment, then downward for remaining segments. §13’s **unique-on-chain**, not this section’s original nearest-wins, is the lookup rule.

Sibling containers may be named from their parent chain; their members require projection. Source-root mapping remains to be verified, but namespace position is never derived from a file path (§10). The former ambient-root-kernel reading is superseded by §13’s no-prelude ruling.

## 4. The uniqueness test — DECIDED: (Y), position-information (operator, 2026-07-06)

Historical (Y) filtered whole-pool homonyms by expected type; **re-adjudicated 2026-07-22**: deleting the pool tier deletes that filter. Core `resolve(name, position)` has no expected-type input. The surviving convenience is pre-resolve sugar only: a bare member plus a **syntactically local written container annotation** (annotated scrutinee, let, field or return type) becomes `T.member`. Inferred/flow-typed containers must qualify or alias; otherwise an edit to another declaration could silently retarget the reference.

## 5. The collision census (empirical grounding, 2026-07-06)

Historical declaration counts did not establish current reachability. Distinguish semantic forks from genuine homonyms, shared enum arms, file-local helpers and seed mirrors. The precise census corrected its own false positives: `ImportTrigger` belongs to a whole-file v1 mirror and retires with that seed; the generic effect-grounding and budget-admission `Reconciliation` types are different concepts. Do not consolidate either merely for matching names. Earlier claims that expected type or nearest-wins eliminate qualification are superseded by §13.

### 5.1 Precise census — declared `module` paths (2026-07-06)

Use declared namespace paths and declaration kind, not directory proxies. Seed duplication is not a standalone rename/consolidation mandate; it dissolves with the seed. The dated census forced no consolidate-now forks after semantic inspection; consult the pinned receipt for its population, not as live coverage.

### 5.2 Tier-2 homonym triage receipt (2026-07-16, quiet-carp-507)

`live_tree_disposition` and `extdeps_external_authority_anchor` are per-entry/per-module construction facts, not shared concepts duplicated by their fixed leaf names: `ConstructionProtocolNoAction`. `emit` is a genuine homonym; the census found no cross-subtree bare uses. `global_bare_corpus_census_test.rs:tier2_namespace_homonym_invariants` checks both ambiguity (UNIQUE proceeds, AMBIGUOUS reds, ABSENT refuses) and reachability (no ambiguous protocol-name refs or cross-subtree bare `emit`), never declaration counts alone.

## 6. Resolution mechanics carried from §1c (unchanged, re-homed)

Constructor ownership follows the binding’s parent edge, never a scan. No expected-type picker/filter survives core resolve. Pattern-via-scrutinee convenience is the syntactically local annotation sugar in §4. Resolve all collisions through the same typed outcomes.

## 7. What dissolves (all Rule-1 dual-representations)

Delete import blocks/grammar, `source_visible_names`, visibility side maps, `masked`, `resolve_node_bounded_masked_boundary`, `UnlistedImportUse`, advisory collection, family-closure and `get_exported_names` re-export of `specific_names`. Alias is an ordinary path binding, never an import visibility gate. While import-scoped policy exists, its gate concerns qualified imported identities, not bare spelling alone.

Delete `parameterized_use_site_prefers_parameterized_decl` with the pivot: it is a second precedence path, not a generic-parameter exception to preserve. The terminal wall is uniqueness **on the reference’s chain**, not whether a globally unique name was listed. The import mechanism and its reconciliation machinery retire together.

## 7.5 Substrate — this rides on the `SymbolIndex` (do NOT fork the index)

`v2.std.symbol_index.SymbolIndex` is the one containment-position→node authority. Fill once; policy gates lookup, never narrows fill into a second index. Historical #6809 delivered the shared index and discharged the scaling gate. Its then-current import-DAG fill and `global_bare` map are migration mechanisms, not terminal semantics: §13 deletes global lookup after its offline oracle use. Resolve, runtime dispatch, content identity and descent consume the same tree.

## 8. Migration — per-subtree, behind a swappable policy

The original policy-gated subtree schedule is historical; current typed stage/order authorities govern execution. Preserve these obligations regardless of schedule:

- Resolution and runtime dispatch change together. No flip before its exact-target qualification/alias mechanism exists; no bare binding depends on incidental shared-pool membership.
- `cross_representation_equality_test:RECEIPTS_SOURCE` discriminates kernel `Nat` from `v2.std.nat.Nat`: the intended coproduct yields the recorded Bool results; same-depth competing `Nat` refuses. The old `Err(NoSuchVariable{Zero})` is not success. This is a historical witness shape, not permission to restore nearest-wins.
- Import-from-definer migration preserves exact defining targets, including re-exports (`EmitResult`, `parse_with_table`, `default_artifact_plan`, `Rust`); own-bindings plus memoized re-export-chain preservation preceded this migration.
- Delete import grammar only at its admitted final stage; it must actually refuse parsing, not remain inert alias syntax. Derive dependency/build/fill edges from references.

PR-5b’s interim `extend_with_bare_reference_closure` used own-tree/pool census lookup, excluded local binders/named-argument keys and test declarations, and refused missing provenance. It was not the unbuilt git-show replay scaffold. #7061 proved its apparent coverage depended on unrelated imports: further stripping was blocked pending closure-independent binding or a coverage construction proof, and waves had to close over imports-from unless defining-module migration landed first.

The former `LayeringImportsGate` direction rule is retired. Surviving source-layout/containment duties read `v2.std.layer_import_scan.layer_import_facts_live`, producing `v2.std.layer.LayerImportFact`: import syntax plus strict reference edges for import-less files, with syntax winning duplicate pairs until grammar deletion removes that arm. Preserve `realization_vocabulary_containment` and `meta_exec_confinement`, acyclicity and extdeps citation duties; do not revive folder-prefix direction as law. The host bridge’s dissolution remains `migrate_when_namespace_only_resolution_lands`.

## 9. Open / to-verify

Confirm the source-root→namespace-root map and remove ad hoc `v2.` special cases through that authority. Namespace position remains declared. An added eligible homonym may loudly break a reference; it may never silently retarget it. The old global/sibling-subtree ambiguity examples must be read under §13’s chain rule.

## 10. Namespace scope is syntactic — the header declares a position AND an extent (operator, 2026-07-19; routed from sharp-bee-290, addendum on #6848)

`module a.b.c` is placement sugar for grafting the declaration forest under `a { b { c { … } } }`. Extent is containment, not end-of-file. Header sugar and explicit braces compose exactly once: header supplies the common prefix, braces nest below it. Two fragments claiming one position refuse. Never infer position from path or keep metadata-edge and containment representations both authoritative.

Stages: header→graft without surface churn; then keyword-free explicit containers via grammar/fold rows; then header dissolution with import deletion and delta-first storage projections. `ModuleStorageBinding` owns storage separately, supporting many-to-many fragments. Step 1 follows #6848; step 2 waits for its dissolution receipts.

#6968 delivered graft/normalize/`validate_module_roots` ingress and partial C5 cleanup, with zero-metadata shape, marker-strip and normalize witnesses. It did **not** satisfy the complete step-1 bar. Remaining obligations from that receipt:

1. Collapse `qualified_name_from_module_node_graft_aware` into ordinary nesting-position `qualified_name_from_module_node`; break its std/extdeps cycle by lifting spine helpers or a parse-only ingress boundary; delete wrapper and `namespace_graft_pre_graft_module_qn`. This does not depend on strip/pool coverage.
2. C6 seed regeneration after wrapper deletion; C7 wet `source_root_ingest_gate_passes` plus real-ingest red controls.
3. Remove metadata-edge/`under_module_root` preserve chains (`dag_resolve_preserve_module_metadata_subtree`, `namespace_graft_parse_projection_edge`) when body lowering consumes grafted trees.
4. Replace `symbol_index_fill_module_root`’s `module_qn` prefix seeding with `symbol_index_fill_containment_node` as sole authority; retire `symbol_index_containment_disposition` scaffold.
5. Review 41316: reject non-grafted export admission and replace `_node_projection` suffix heuristics with declared grammar rows.

The historical post-C5 excludes in `docs/probes/census_extra_excludes.txt` and `still-hawk-row-coordination.txt` were ephemeral CLI inputs, never `ci_layer_roots`; the oracle was derived-tip equality with the recovered population. #6979’s chain binding coordinated after C5. These are receipts, not fresh execution instructions.

## 11. Terminology: "module" → "namespace" (operator-ruled, 2026-07-19)

Containment is the relation; namespace is a pure container; module names the dissolving namespace/storage-file fusion. There is no export/privacy boundary; dependency, compilation and storage have separate authorities. Use namespace in new design/carriers, reserve module for storage binding. No standalone corpus keyword rename or alias period; any sugar-keyword rename rides the terminal import wave.

## 12. Runtime dispatch is another consumer of the containment tree — the fork is three-way, not two (zesty-deer-446, 2026-07-19)

Historical direct `twin_sig` probes found three conflicting rules: authored import selection, `lookup_resolved_sig` first-hit `func_env.parents`, and interpreter last-insert bare-name `fn_nodes`. Neither implementation honored the selective name list. Absence of diagnostics proved nothing: label and declared/actual return-type checks did not discriminate those shapes. #6896 fixed the application-site mismatch class; equal-signature wrong targets remain a naming problem.

Delete `fn_nodes`; dispatch by the resolver’s exact `QualifiedName` into shared `SymbolIndex.entries`, one map hit after resolution, not an ancestor walk per call. Do not independently teach runtime imports or stamp targets from a wrong resolver. A temporary holding measure may expose actual dispatch in the existing typed binding-fork ledger, never widen or introduce another binding rule.

### 12.4 The divergence census (operator-directed 2026-07-19; delivered, and no longer a prerequisite under the 2026-10-30 process ruling)

The read-only census compares actual resolver return identities, not diagnostic silence. Every whole-corpus/both-root call site records callee, caller, legacy target and containment target. Bucket into Agree, Diverge (list both targets), ContainmentAmbiguous (qualification owed), ContainmentUnresolved (fill gaps reported separately); never skip. Record lookup cost to falsify the substitution claim without changing policy, resolver or dispatch.

Delivered by `resolution_divergence_census` (#6936/#6967), consumed by step 1 on 2026-07-22; raw receipt `docs/probes/resolution_divergence_census_2026-07-22.tsv`. It separated import-unresolved, neither-bound/builtin/local/genuinely-unbound and owner-mismatch cases, and measured chain cost. The live silent-pick specimen was `gunbc.falsifier_workflow`’s `ci_repo_root_shell` choosing `ci_spec` over `merge_admission_produce`; consolidate or qualify. `std.interface_summary` owns interface-grain declared/use arity, not another name census.

## 13. Resolution is unique-on-chain, not nearest (operator ruling, ratified 2026-07-21)

Resolve the first segment to the **unique binder on the ancestor chain**: zero→located `Unresolved`; multiple→located typed `Ambiguous` with every candidate and qualify/alias/rename fixes; one→project remaining segments downward. Bare names use the same rule. Shadowing refuses **at the reference**, not declaration. No nearest-wins, global-unique fallback, first-hit or expected-type picker: edits elsewhere may break a reference loudly, never silently change its meaning.

Aliases are ordinary binding nodes through the same rule. `alias A = m.path.A` encodes the target once; unused aliases are lintable. Import→alias migration follows the walk’s exact target with the old import only a cross-check. `symbol_index_global_unique_lookup`/`GlobalBareLookup` survive only as offline migration oracles, then delete. The [operational companion](namespace-unique-on-chain-operational-plan.md) owns witness/production-consumer/census detail.

Historical step-1 receipt (2026-07-22): host-set thread-local `NameResolutionPolicy` defaulted OFF/ImportScoped; strict type/value and function paths were available without a subtree flip. `FuncSigLookup` gives Resolved/Unresolved/Ambiguous; ambiguity never falls through to census fallback. Semantic bind sites match the whole outcome; analysis-only `func_sig_if_resolved` conservatively declines enrichment. `namespace_unique_on_chain_policy_test.rs` compares policy modes, full candidate refusals, unique-chain success and unbound controls. The then-current zero-chain/pool-homonym ambiguity behavior is historical, not the terminal zero→Unresolved rule. Strict flips require aliases with or before them; current typed cut stages supersede the old scheduling prose.

**No ambient prelude** (operator 2026-10-04/13; kernel ruling B, 2026-10-14): a module sees its own declarations and imports. Native free roster calls bind only imported `std.algebra.collection_*_shape` identities (or within std.algebra); there is no free `fn concat`, and `kernel_mint_declaration_rows` is Optional mint only. `resolve_atom_unscoped` must refuse invisible roster heads rather than fall through to primitive authority. Seed `infer_builtin_call_type`/registry acceptance is a frozen migration window bounded by generated import sweep #14017, not native success. Restoration: native emit of `v2.cli.compile_cli` over dag+src/v2 produces no concat/count missing-import unbound fatal.

Receiver-inhabitant methods remain independent of imports: removing free-call fallback must preserve carrier templates for map_get/count/map/any. `collection_map_get_shape` free identity is deleted; `map_lookup` survives. Inline map_get method and map_lookup remain one operation represented twice until v1 method migration/dedicated rename; map_insert/empty_map/map_lookup await #13872’s real functions. Bare-member sugar remains restricted to syntactically local annotations (§4).

Historical flip-red residue: homonym/local consolidation did not depend on Root-A; two-std Set/Map/Byte/Char forks did, with only Char using its emit seam. Re-measure current residue through the owning instruments rather than treating those dated totals as standing.
