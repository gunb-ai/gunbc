# Computation-identity de-fork — census, unification, repairs

**Status:** executed census (3 parallel sweeps over `dag/**`, `src/v1/stage0/**`, `src/v2/**` + the two carrier design docs, 2026-07-22) · staging carrier per the doc-reachability wall (bind row in `dag/gunbc/plans/dag_v2_defork_audit.dag`) · **dissolution:** each repair below dissolves into a mark on its carrier as it lands; the census as a whole dissolves when `ComputationIdentity` is a live in-tree type and the repair table is empty — at that point the carriers tell the whole story.

**Operator directive (2026-07-21):** "we need a consistent terminology for what 'identical computation' means … not solving it will lead to more and more forking — we need a de-forking task to find the common patterns among these, unify them, and then move on." This doc is that task's first executable artifact. It gates **W3** (the cross-run typed-module store — its key must not be identity-fork N+1) and **P5** (demands derived — the memo obligation reads identity).

Companion authorities (NOT duplicated here): [duplicate-work-graph-lens-design.md](duplicate-work-graph-lens-design.md) (the `ComputationIdentity` lattice + 10a/10b instance census), [module-identity-storage-binding-design.md](module-identity-storage-binding-design.md) (the path⇄module binding thread), [cross-entry-typed-module-memo-sketch.md](cross-entry-typed-module-memo-sketch.md) (the typed-module key), `dag_v2_defork_audit` (the dag↔v2 std-library de-fork — a *different* fork family, same lane).

---

## 1. The finding in one paragraph

The **hash base is not forked**: every content-hash producer in the tree bottoms out in one primitive pair (`std.content_hash` → `atom_identity_hash`/`hash_combine` → `v1_rt.rs` fnv1a64), with exactly one hand re-inline (R3). The forks live one level up, in three places: **(a) input-set composition** — three layouts of the same "build-inputs" key (`ArtifactKey`, `emit_on_demand_key`, `typed_module_key`); **(b) parallel non-hash comparison predicates** answering the same question — `std.realization.reconcile`'s effect-shape `key_source_eq` vs `v2.std.materialize`'s content hash (both emit `Materialization`), and the five path-string ledgers all using a path as module identity; **(c) the missing unifier** — four module notes defer to a `ComputationIdentity` type that exists only as design-doc prose; there is no `type ComputationIdentity` anywhere in the substrate.

## 2. The one law (the unification)

One sentence, applied at every grain and frame (DESIGN §2 horizontal):

> **The identity of a computation is the content hash of (process ⊕ declared inputs), where "declared inputs" is complete for the key's frame** — every input that can vary within the frame's lifetime is a key term (the frame-elision law), and a missing term is a typed refusal, never a partial key.

- **Evidence type:** the `ComputationIdentity` lattice (`StructurallyIdentical | NormalizedIdentical{normalizer} | ExtensionallyIdentical{bound} | IdentityUnknown{cause}`, `IdentityUnknown` a typed located debt, never permanent) — from the duplicate-work design §4, to be landed in-tree (R1).
- **Coarse keys are folds of fine keys:** `closure_digest` = fold of per-file hashes; `module_key` = source ⊕ import interfaces; `typed_module_key` = module_key ⊕ compiler identity; `ArtifactKey` = closure ⊕ emitter ⊕ target ⊕ toolchain. Same fold shape at every grain — this is one concept, not four.
- **The anti-axis is real and must be typed, not fused (anti-over-merge):** `membership_reconcile` correctly keys members by **stable identity, not content** (content drift = Modified, never Remove+Add). "Content-is-identity" (recompute-sharing) and "identity-survives-content-change" (membership/teardown) are two verbs on one axis — model the axis (`IdentityBasis = ByContent | ByStableKey` or equivalent), don't collapse the two. Likewise the affected set stays a forward-only peer (not a materialization), and `SourceRef` (storage) stays distinct from module identity (both rulings already in the companion docs).

## 3. Census — the mechanisms (condensed; file:line receipts verified 2026-07-22)

**Tier 0 — the non-forked base.** `dag/std/content_hash.dag:5-15` (`content_hash_atom/combine/tagged`) → primitives (`dag/std/primitives.dag:489,497`) → `v1_rt.rs:700-777` (fnv1a64; `bytes_identity_hash`, `atom_identity_hash`, `hash_combine`). v2 twin `src/v2/std/node.dag:1514` `content_hash` (occurrence-stripped canonicalization `:659`) — correctly derived, occurrence identity (`occurrence_id`) deliberately excluded.

**Tier 1 — build-inputs keys (one concept, three layouts).**
- `ArtifactKey`/`artifact_key_hash` — `dag/std/artifact_store.dag:10-31`: {closure_digest, emitter_identity, target_language, toolchain}.
- `emit_on_demand_key` — `dag/std/emit_on_demand.dag:33-45`: same set with toolchain *derived* from `ArtifactGrain`; delegates to `artifact_key_hash` (constructor, borderline fork).
- `typed_module_key` — `dag/std/interface_summary.dag:99-114`: source_hash ⊕ direct-import interface hashes ⊕ compiler_identity; v2 producer `src/v2/lens/interface_summary.dag:135`; Rust realization `cli_run.rs:5211` `typed_module_content_key` (**sound**: refuses on any missing term; compiler identity = `transform_content_digest`, shared with the resolved-graph cache).
- Rust `subject_digest` — `resolved_graph_cache.rs:147-231`: closure content ⊕ `transform_content_digest` (compiler binary bytes). **Sound**; the reference key layout.

**Tier 2 — the Materialization fork (the primary §3 fork, already named by the duplicate-work design).** `dag/std/effects.dag:27-115` `create_double_init_collapsible`/`key_source_eq` (pairwise effect-shape equality) vs `src/v2/std/materialize.dag:39-57` (DAG-wide content hash) — both produce `Share|Recompute`. `materialize`'s own note declares itself the generalization; the effect-key path is the residue to dissolve (Half B of the duplicate-work lane).

**Tier 3 — path-string-as-module-identity (one fact, five ledgers; the module-identity design already rules the fix).** `CompilerModuleFrontierRow.module_path` (`src/v2/compiler/self_host/frontier.dag:91`), `FrontierQualifiedModuleBinding` hand rows (`frontier.dag:52,112` — self-described "THE SECOND AUTHORITY", parity-gated), host `build_module_path_index` (`cli_run.rs:963-1010`, third re-derivation), `CommitWitnessClaim.entry` enrollment keys (`dag/gunbc/commit_workflow.dag:35`, `src/v2/workflow/witness_admission.dag:37`), affected-set path intersection (`src/v2/lens/module_graph.dag:296-417`, host twin `cli_run.rs:3470-3536` — both **refuse on provenance gaps**, sound). Fix = the derived path⇄module binding on `source_authority` (companion doc §2/§3); `typed_module_key` is the content-keyed identity these should reuse at the typed-module grain.

**Tier 4 — realized memo/cache frames (Rust; all sound unless flagged).** Cross-run: resolved-graph disk cache (`resolved_graph_cache.rs`, fail-closed reads, LRU cap) and the native artifact cache (`v1_interpreter.rs:7168-7466` — **R2 flag below**). Per-process: `pool_qualified_fill`/pool census singletons (unkeyed whole-tree fill — correct by construction), `resolved_graph_memo`. Per-thread: `process_shared_index` keyed on **canonicalized** roots (`cli_run.rs:4737`) vs `PROCESS_RESOLVE_STORE` keyed on **raw** roots (`:4767`) — **R4 flag**. Per-batch: claim_executor `walk_memo` keyed `(entry, ExecutionMode)` (`claim_executor.rs:1539`; mode correctly in-key). Per-ctx: `EvalCallMemo` (structural re-verify + effect-free gate + Weak-liveness — sound), `PureCallMemo` (rc-pointer keys held live by keepalive), `ParseTableMemo`, `CanonKey` (`v1_interpreter.rs:242-347`, collision-safe: falls to structural eq). Per-diff: regen input closure (`cli_run.rs:2179-2258`, path-membership identity, fail-closed skip logic).

**Cache-catalog classification vocabularies (policy layer, partially unified already):** `CacheKeying{ContentKeyed|ExistenceKeyed}` (`dag/std/materialization_ladder.dag:53`, the #6352 existence≠identity wall) ← projected from `CatalogKeyInputs{ContentAddressed|NativeInternal|HandAuthored}` (`dag/extdeps/cache/types.dag:58-103`); the `HandAuthored` github-actions row remains the standing identity-vs-existence hazard, already counted there.

## 4. Repairs, ranked (each names its carrier and lands independently)

| # | Repair | Class | Carrier / lane |
|---|--------|-------|----------------|
| R1 | **Land `type ComputationIdentity`** (the 4-arm lattice + `IdentityUnknownCause`) in `dag/std` beside `realization`; first consumer = `v2.std.materialize` (already implements the `StructurallyIdentical` corner); the four citing notes repoint from prose to the type. | missing authority | duplicate-work lane, Half B |
| R2 | **Native-cache key omits toolchain identity in a durable frame.** `emit_on_demand_key`'s toolchain term is the nominal `toolchain_for_grain` string, not actual rustc/compiler identity; with `GUNBC_NATIVE_CACHE_ROOT` durable across runner toolchain upgrades, a byte-identical closure warm-skips a rebuild the new toolchain should perform (`.native_ready`, `v1_interpreter.rs:7368`). Fix at the key (fold real toolchain identity — the `rustup` pin or `rustc -V` digest — into the key or the root path), not a validation check. Until fixed, the falsifier cold control is the honest backstop, and the flag upgrades the declared "toolchain pin" follow-up from nicety to soundness repair. | frame elision (soundness) | witness-realization P6 belt; #6990 follow-up |
| R3 | **fnv1a64 re-inlined by hand** at `cli_run.rs:16057` (`source_root_ingest_content_hash_fnv1a64` — second copy of the constants) → route through `v1_rt::bytes_identity_hash`. | hash-authority fork | seed cleanup, mechanical |
| R4 | **Roots-key canonicalization fork**: `PROCESS_RESOLVE_STORE` keys raw `source_roots.join`, `process_shared_index` keys canonicalized roots — path-spelling variants duplicate resolves between two adjacent caches. Key both on the canonical form. | duplicate-work | seed cleanup, mechanical |
| R5 | **The memo that isn't**: `dag_node_surface_fingerprint_memo` (`v1_compiler_dag_collect_support.rs:226`) passes through; `dag_collect_fp_memo_reset` is a no-op. Either memoize or rename — a `_memo` name asserting a frame that is elided to nothing is a small honesty defect. | inert scaffold | seed cleanup, mechanical |
| R6 | **Unify the Tier-1 layouts**: one key composition (`closure/source digest ⊕ upstream-interface digests ⊕ transform identity ⊕ target/toolchain`) with the three named keys as declared projections of it, so W3's store key and the artifact keys are provably the same concept at different grains. | input-set fork | interface_summary / artifact_store carriers; prerequisite for W3 |
| R7 | **Dissolve the Tier-2 effect-key residue** onto `materialize`'s content-hash qualification (the duplicate-work design's existing Half B scope). | Materialization fork | duplicate-work lane |
| R8 | **Type the identity axis** (`ByContent` vs `ByStableKey`) so `membership_reconcile`'s deliberate identity≠content stance and the content-identity family stop being an implicit conceptual fork. | missing axis | membership-reconcile spine carrier |

Group-B path-ledger collapse is **not** an R-row here: it is the module-identity design's existing Phase plan (its parity witness is the interim honesty mechanism); this census only confirms its census and adds no scope.

## 5. What this unblocks

- **W3 (cross-run typed-module store):** `typed_module_content_key` is *already sound* (all terms present, refuse-on-missing). W3 can proceed on it directly once R6 declares it a projection of the unified composition — the store is engineering, not identity design.
- **P5 (demands derived):** the memo obligation ("≥2 across isolation ⇒ obligation at the LCA") reads `ComputationIdentity` (R1) + the frame vocabulary already landed in `materialization_ladder`. No new identity concept needed.
- **The whack-a-mole ending:** the resolve caches' recurring breakage is partly key-fork churn (R4 is a live instance); one law + typed refusals on missing terms is the structural end of "fixed resolve 10 times."
