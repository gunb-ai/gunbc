# Duplicate work = the content-hash qualification of `Materialization` — design

**Status:** DESIGN (pre-implementation). No S2-self-emit dependency; couples to the **materialization model** (`std.realization` + the execution spine's `materialize`), a std concept that is partially built (§8). The discriminant is proven green-by-execution at placement grain (§9); the corpus form waits on the coupling in §8.

**Origin & correction.** The operator's original ask — "detect a redundant dependency and force a rewire; kill compile → … → recompile" — was first drafted as a standalone "duplicate-computation lens" over `content_hash`. The operator then caught a §3 fork: "materialization" already exists as a first-class concept, and a second one is a nickname. This doc is the corrected model — **duplicate-work is not a new concept; it is the content-hash qualification step *inside* the existing `materialize`, i.e. the generalization of `std.realization`'s `reconcile`.**

**Principle roots:** §2 (minimize redundancy — one computation, one producer/materialization), §3 (single authority — reuse the real `Materialization`, don't nickname it; and an undecidable identity often *is* a §3 fork, §6), §4 (closed + bounded ⇒ decidable-with-constraints), §5 (Delete > Share > Cache; fail-closed bottom, no absorbing fallback), §6 (priced in displaced cost; the moat is the located root cause), §7 (reuse the `DescentEvidence`/`UnknownComplexity` lattice pattern).

---

## 1. The concept already exists — and already does half of this

`dag/std/realization.dag`:
```
type Materialization = Recompute | Memoize | Share            -- :105
type RealizedStep<S> { shape, placement, materialization: Materialization, cost }   -- :107
fn reconcile<S>(steps) = fold: if has_collapsible_peer(acc, s) then mark_collapsible_share else keep  -- :134
```
`reconcile` **is** duplicate-detection → dedup: it finds a collapsible peer and flips its `materialization` to `Share`. The execution spine seals it: `run ≜ realize ∘ materialize ∘ dependency_view` (execution-spine-design.md §2), where `materialize` *is* the `Recompute | Memoize | Share` decision, and FLAG D keeps `Materialization` as a reading of the DependencyView.

So the "duplicate-work lens" is the qualification step *inside* `materialize`, and `reconcile` is its current prototype. The prior draft's "materialize law" and the `ComputationDemandFact` carried from #6372 were both nicknames for pieces of this (§3 fork — corrected here).

## 2. Why it is only half-baked — the two axes `reconcile` is narrow on

`reconcile` today is narrow on **both** axes, and each gap is one half of the work — the operator's "two half-baked concepts that should land together":

- **Its key is `create_double_init_collapsible`** — effect-shape, pairwise, create-if-absent only. It cannot see two *pure* twins or two content-identical compiles. → **Half B — generalize the key to the `ComputationIdentity` lattice (§4).** This is the qualifier `reconcile` keys on, not a lens beside materialization.
- **It ranges only over `RealizedStep`s** (realized effect steps). Most computation never becomes a `RealizedStep`, so it has no materialization record to hash. → **Half A — materialization must describe the result of *all* computation** (the operator's "it doesn't yet today").

**Assembly-line framing (the operator's).** Materialization = the part each station produces (today only some stations record one). Duplicate-detection = hash/qualify every part, collapse identical ones to `Share` (= generalized `reconcile`). QC over a line where most stations emit no part is half-baked — so the halves land together (§8).

## 3. The law, restated on the real concept

> **Duplication = N materializations of one computation-identity with no `Share` between them.**

- **computation-identity** — `content_hash` (`src/v2/std/node.dag:1449`), **occurrence-independent** (`canonicalize_node_for_content_hash` stamps `SyntheticOccurrence`, `:594`): the same operation on content-identical inputs hashes equal at any position. Identity is a *lattice* (§4).
- **N materializations, no Share** — the multiplicity's **source** (§5) picks the fix. The substrate does not hash-cons (`node_with_occurrence_id`, `:115`, mints distinct `occurrence_id`), so a *fork* is only the degenerate source, not the general case.

### The state × decision table — the final verdict logic (operator-signed 2026-07-09; implemented `dag/std/materialization_ladder.dag`, 14/14 witnesses green)

One axis decides everything: **when the redundancy is knowable, and whether what was knowable was prepared for.** Redundancy itself is never the error — *knowable-but-unprepared* is. Scopes nest (eval → plan → process → emitted shell → CI run — a "run" at any layer is a frame one scope up, all emitted from one substrate), and **plurality of demand per identity** is the primitive throughout:

| state — plurality × frame | decision | if unprepared |
|---|---|---|
| 0 × pure computation | `DeadComputation` — authored waste (the degenerate rule-1) | the error is the verdict |
| 0 × effect | `AcceptedEffectIsTheUse` — valid; the effect *is* the use | — |
| 1, no declaring frame | `AcceptedSingleRecompute` — a cache is dead weight | — |
| ≥2, LCA = shared-state frame (rewireable) | `AuthoredDuplication` — ERROR; fix = rewire/Share, never a cache | the error is the verdict |
| ≥2, LCA = isolation boundary | **memo obligation at the LCA** — must be `Discharged` by a covering, content-keyed provider with declared eviction | `RefusedNoProvider` / `RefusedScopeTooNarrow` / `RefusedExistenceKeyed` |
| 1 under a **declared-emergent** frame (`ReplayedFrame attempts>1`, `UnboundedSiblingsFrame`) | obligates **up front** — checkpointing and cross-run caches are *derived*, prepare-before-demand | `RefusedNoProvider` at the declaring frame |
| unexpected-emergent (measurement-only, e.g. recompute-trace) | typed **acceptance + finding** — converts to a declared row for the next run | not an error the first time |

Nature gates ride on top: `FreshEffect` never memoizes (duplicated measurement is intentional → `ExemptFreshEffect`); a redundant `WorldRead` without a declared staleness envelope is `RefusedUnmodeledWorldRead` — **a TTL is always one of two confessions: an unmodeled dependency (fix: into the key) or a tolerated staleness (fix: declare the envelope)**; below-cost-floor duplication is `AcceptedBelowCostFloor` (roster-visible; deleting the roster row flips red — witnessed). Rule-3 eviction is by construction: `CacheProvider` cannot be written without an `EvictionPolicy` (`ScopeExit` scoped — eviction = frame exit, derived; `SpacePacked` persistent — dropping pure facts only ever costs recompute). Rule-4 keying wall: `ExistenceKeyed` providers are refused even when covering — existence ≠ identity (receipt: build-if-absent stale binary, #6352).

**§3 convergence — `v1.compiler.ownership` is the eval-frame instance** (found on operator recall, consolidated 2026-07-09): `binding_fan_out` = demand plurality with `Threaded` (iteration-carry) excluded — exactly the reduce-spine exclusion; `SoleOwner` = single-demand move; fan_out>1 in a function body (a shared-state frame) = the emitter's Share/clone decision; `SharedError{consumer_count, sites}` = plurality refused in an affine-uniqueness context (a context-requirement refinement the std ladder does not yet carry — the residue keeping the v1 instance load-bearing); the v3-era zero arm (`fan_out==0 ∧ pure → dead code; ==0 ∧ effect → valid`) is `DeadComputation`/`AcceptedEffectIsTheUse`, back-ported into the ladder. Dissolution: emit-stage migration onto the v2/std spine grounds ownership on the ladder; the v1 instance retires with the seed (convergence note on the ladder module, seed untouched to avoid regen churn).

## 4. Computation-identity is a bounded lattice (§4), not a boolean

In a Turing-complete language the strongest form (semantic equivalence of structurally-different programs) is Rice-undecidable and permanent. `.dag` is **closed, bounded, total** (forward execution, finite measures, `Int` on machine widths, no non-termination), so extensional equivalence over a bounded domain is **decidable by enumeration**. Semantic equivalence here is not Rice-impossible — it is decidable once the bound is supplied, §6-priced. Mirrors `DescentEvidence` (`dag/std/termination.dag`) and `UnknownComplexity { diagnostic }` (`src/v2/lens/complexity.dag`):

```
type ComputationIdentity
  = StructurallyIdentical                       -- content_hash equal          (cheap wall, top)
  | NormalizedIdentical    { normalizer }       -- equal under a declared normalizer (cheap wall)
  | ExtensionallyIdentical { bound }            -- equal on the modeled bounded domain (§6-priced wall)
  | IdentityUnknown        { cause }            -- fail-closed BOTTOM (§6)
```
This lattice is the generalized *key* `has_collapsible_peer` computes (§2 Half B).

## 5. The multiplicity-source axis — one law, one axis picks the fix

Identity says "same work." The **source** of the multiplicity says how to fix it (the prior draft's error was assuming every source was "fork"):

| Source | Detect via | Fix | Static now? |
|---|---|---|---|
| **fork** | dup content_hash, distinct `occurrence_id` | delete → rewire to survivor | ✅ |
| **placement** | one identity, N consumers across a placement boundary, no Share edge | add artifact-Share edge (`needs:` + up/download) | ✅ after modeling placement — **the CI-build flagship (§9)** |
| **runtime-call** | one node, N `DataDependsOn` demands | memoize / hoist producer | ✅ static count |
| **runtime-loop** | loop-invariant subtree in a fold | hoist out of loop | ✅ after free-var analysis |
| **wrong-scope** | shared content-identical *subtree* recomputed at wrong grain | re-scope intent to one authority | ✅ subtree-hash |

Reuse (one materialization, N consumers) is the goal state, never flagged — it is what every fix produces (= `materialization: Share`).

**Grain coverage — one identity, N detection surfaces (operator direction, 2026-07-16).** The concept is *not* shell-specific: "same inputs + deterministic process → same content-identity → the second is duplicate work" is the whole law, and the surface it's read on is a *realization axis*, not the concept. Three surfaces, one `ComputationIdentity`:

| grain | surface read | detector today | reaches the double-compile? |
|---|---|---|---|
| **within-script** | argv / `ShellWord` tokens | `v2.lens.duplicate_computation` (dissolves into the general form, §7) | no — not a shell command |
| **within-graph** | `content_hash` over Node subtrees | `v2.std.materialize` (analysis-only MVP, structural identity) | no — not a compiled-graph subtree |
| **within-run execution-frame** | plurality of demand *inside one process/seed run* | **none yet — the uncovered grain** | this is where it lives |

The **~275s double-resolve is the exemplar of the uncovered grain**: batch-1 (discovery) resolves the corpus, then batch-2 (execution) resolves it again — two `compile_to_resolved` calls inside one `claim_executor` **v1-seed** process. Neither the argv lens (no second command) nor the graph detector (Rust seed, not a Node subtree) can see it. Per the ladder it is unambiguously classed: one process = **shared-state frame ⇒ AuthoredDuplication ⇒ REWIRE, never a cache** (fix the seed to resolve once and share) — distinct from the *cross-run* repeat (isolation boundary ⇒ the content-keyed store obligation). Same `ComputationIdentity`, remedy chosen by frame. Direction: the general detector must reach the execution-frame grain (the demand-plurality read the ladder already specifies for `eval → plan → process` scopes), at which point the argv lens dissolves into it (§7) and the within-graph MVP extends down into it — one detector, the surfaces its realizations.

**Safe-to-collapse** (survives from the draft): purity is the *license* to rewire (pure → behavior-preserving by construction); effects are safe only if idempotent/collapsible — `dag/std/effects.dag` `is_idempotent_effect` (`:31`), pairwise `create_double_init_collapsible` (`:73`). Owed extension: `effect_group_collapsible(shapes)` — an N-ary fold over the pairwise authority (this generalizes `mark_collapsible_share` from create-if-absent to any collapsible peer). **Worth-collapsing:** a cost floor (`src/v2/lens/cost.dag`) — reports *what*, never *whether to fail open* (not an absorbing fallback, §5).

## 6. Undecidability = missing concept or missing enforcement (empties the bottom)

`IdentityUnknown` is **not** "unknowable." In a closed/bounded/grounded substrate it is a symptom of anemic modeling (§2/§4): a missing *concept* that unifies two things, or a missing *enforcement* that canonicalizes them (§4: "in a closed system a heuristic is never necessary — the richer source always exists"). Sharpest instance: **the undecidability often *is* the §3 fork** — two computations are "hard to prove equal" because they are two structures for one concept; ground them and identity becomes trivially structural (receipt: the old `CpuVendor` vs `GpuFacts.vendor` fork, dissolved by `Vendor<Domain>`). So the bottom carries a typed cause (conflating them = the state-space-conflation failure mode, §5):

```
type IdentityUnknownCause
  = MissingConcept     { candidate_authority }   -- two structures, one concept: ground them (§3)
  | MissingEnforcement { mechanism }             -- canonical construction / normalizer absent (e.g. hash-consing)
  | BoundUnmodeled     { domain }                -- extensional check available once bound declared
```
Every "after modeling" entry in §5 resolves to one of these. **The lens's product is therefore not "here are dups" but "here is redundant work, and here is the concept/enforcement whose absence lets it exist"** — the moat framing (§6): it locates the root (a missing authority), making it, at its limit, the leaf-side decomposition-debt detector parked as an open thread. The bottom is a *located, typed modeling backlog item*, never a graveyard; the *categorization* is decidable even when the answer isn't (as `UnknownComplexity { diagnostic }`).

## 7. Fix hierarchy — Delete > Share > Cache

1. **Cache / Memoize — dispreferred.** Keeps the duplicates. The confession that redundancy was not removed (§2). Corollary: **every cache in the tree is a map of known redundancy** (`pure_call_memo`, `ParseTable`, `resolved_graph_cache`, sccache, floor M1 `walk_memo`) — this lens's candidate-site list.
2. **Share / rewire — preferred.** One materialization, N consumers (`materialization: Share`). The actionable output.
3. **Construction / hash-cons — endgame (§5 strongest).** Content-addressed construction returns the existing node; a pure fork becomes unwritable (`MissingEnforcement` supplied). The lens sizes this endgame.

**#6372 becomes an instance.** `v2.lens.duplicate_computation` already models this at argv grain (`ComputationDemandFact` grouped by key, refused when N unguarded demands share a key, with a declared-`ReplicatedOracle { runs: N }` escape). Its `computation_key` is the argv projection of `content_hash`; its escape is the declared-exception the general form needs verbatim. The general form **subsumes** it — the CI-job lens is not a separate point-thing, it is this at placement grain (§9).

## 8. Scope boundary and sequence — the two halves, and who owns which

This lane does **not** fork the spine's materialization work. The boundary:

- **Half B — THIS lane.** Generalize `reconcile`/`has_collapsible_peer` from the effect-pairwise key to the `ComputationIdentity` lattice (§4); the `effect_group_collapsible` N-ary fold (§5); the declared-exception escape; the typed `IdentityUnknown` bottom (§6); the placement-grain witness (§9). Advances statically over today's materializations (RealizedStep effects + the #6372 argv projection) — a real improvement, but only a **partial view** until Half A.
- **Half A — the execution-spine lane (`materialize`/FLAG D).** Extend `Materialization` to describe the result of *all* computation, not just `RealizedStep` effects. This is the spine's authority; this lane consumes it, does not duplicate it.
- **Land together.** A generalized qualifier (B) over a partial materialization (A) is half-baked by the operator's assembly-line test — so completeness is declared only when B keys over a materialization model that covers all computation. Until then B ships as an honest partial with its coverage boundary stated (no absorbing "covers everything" claim, §5).

**Sequence:** (a) placement-grain witness on the CI build ×3 (§9 — **done, green**); (b) `effect_group_collapsible` N-ary fold over `effects.dag`; (c) generalize `has_collapsible_peer`'s key to `ComputationIdentity` (coordinated with the spine so it reads the real materialization record, not a nickname); (d) fold #6372 in as the argv-grain instance; (e) corpus form once Half A lands.

## 9. De-risking — the placement-grain witness (built, green)

`src/v2/test/claim/duplicate_computation/placement_grain_witness_test.dag` — proven green-by-execution (local `claim_batch`, interpreted), reusing #6372's live lens (no new production code), proving the flagship is caught by the *same* logic once each independently-building job is projected as one demand:

- **RED — placement duplication.** `ci_release_build_script` materialized at 3 `needs:[]` jobs (`ci_job`/`ci_regen_job`/`rust_tests` warm; `ci_workflow.dag:255/381/414`), no artifact Share edge → **exactly one** violation.
- **GREEN — the rewire.** One producer + an artifact-consuming (guarded) job → clean. A consumer emits no build demand — the fix the lens forces.
- **ADMISSIBLE — declared per-placement.** Genuinely per-runner work with a declared `ReplicatedOracle` (observed == declared) → not flagged.
- **RED control on the escape.** 3 observed > 2 declared → violation (the escape is a declaration, not a blanket silence).

Remaining witnesses (carried from the identity lattice): pure fork (perturb-subtree red control), effect-gated (`Append` vs `CreateIfAbsent`), cost-floor, and the typed `IdentityUnknown` bottom classification.

## 10a. Consolidation plan — realization ⊕ materialization ⊕ ownership are one law (operator, 2026-07-09)

**The unification.** Three concepts overlap because they are one law read at three places: **Materialization** (`Recompute | Memoize | Share`, authority `dag/std/realization.dag`) is the *verdict vocabulary*; **ownership** (`v1.compiler.ownership`) is the *verdict computation at the eval frame* (plurality via `binding_fan_out`); **realization** is the *provider selection* that discharges the verdict at each frame's carrier. Nothing merges structurally yet — the consolidation is (a) one verdict vocabulary everywhere, (b) every hand-rolled instance named as a provider/instance row with a dissolution trigger, (c) new instances unwritable outside the qualifier.

**Share's realization spectrum — layer-aware providers.** `Share` is one verdict; its *handler* is picked by frame-carrier capability × readonly-ness. The cheapest cache is a **reference**: if the demanders live in the same program, the carrier supports reference passing, and the value is readonly (nobody's reads are disturbed), the provider is *point-at-it* — no memory destroyed, no copy built. Demotion down the spectrum must be **priced, never silent** (#6249's silent clone-fallback O(n²) is the receipt for what silent demotion costs):

| frame carrier | Share handler | requirement gate |
|---|---|---|
| same process, ref-capable | reference (`&T` / `Rc` / arena index) | **readonly** + same carrier (the D3 analog at value grain) |
| same process, value-semantic structures | structural sharing (im-rc HAMT, #6369) | persistent carrier |
| process lifetime | content-keyed in-memory memo | scope-derived lifetime (evict = frame exit) |
| cross process, one run | artifact hand-off | content-keyed |
| cross run / fleet | CAS tier (sccache) | content-keyed + space budget (sound-to-drop) |

**The census — every hand-rolled instance found, its cell, its action** (the tracker for this consolidation; hunt 2026-07-09):

| instance | frame | today | ladder cell | action → trigger |
|---|---|---|---|---|
| `v1.compiler.ownership` (`src/v1/ownership.dag`) | eval | fan-out move/share/affine-error | Share-at-eval verdict | ✅ consolidated (convergence note; zero arms back-ported); retires with seed |
| interpreter `Rc` + arena; im-rc HAMT (#6369) | eval/value | reference & structural-share handlers | Share realizations | keep; name as provider rows (reference tier) |
| emitter clone-fallback (#6249, fixed instance) | eval | silent demotion to copy | priced demotion | wall: demotion requires a priced reason (C4) |
| `ParseTable` (v2 parse; DESIGN §2 debt) | process | hand-rolled memo | Memoize@process | ground on Realization carrier (C5) |
| `cached_stage` (`src/v2/std/staging.dag`) | process | hand-rolled stage memo | Memoize@process | fold into qualifier; fold-ergonomics #1 names it (C5) |
| M1 within-walk resolve memo (claim_batch "resolved once") | plan | correct Share@plan | Discharged | declare provider row (C2) |
| `PROCESS_RESOLVE_STORE` (`cli_run.rs`) | **process (wrong scope)** | memo, ⊤-lifetime → 9 GB | scope = demand span | **W4/C3: entry-scope + evict-at-exit; receipt = 9 GB → ~2 GB** |
| intern / `resolved_graph_cache` (one-shared #5867) | process | Share of canonical values | Discharged | declare provider row (C2) |
| assumed-green node-frontier skip (re-verify) | plan | cone-keyed verdict memo | Memoize, content-keyed done right | declare provider row (C2) |
| affected-set witness skip (#6061) | plan | verdict memo on affected closure | **peer, not instance** | stays a peer (forward-only; consumes same identity facts) |
| sccache | fleet/CAS | content-keyed compile cache | Discharged | provider row + honesty receipt (exit-0-no-binary class) (C2) |
| build-if-absent (#6352, removed) | CI | existence-keyed | `RefusedExistenceKeyed` | ✅ landed as the keying wall |
| Cargo cache action (`ci_cache_key`) | CI | hand-rolled key string | Memoize@CI | provider row, SpacePacked (C2/F) |
| resolve-cache default-on (#5789 reverted) | cross-run | persistent memo, IO 11× | rule-4 *cannot meet requirements* | typed requirement row; revisit under provider reqs (C2) |
| recompute-trace (unbundled extension) | eval, dynamic | unexpected-emergent detector | state-4 finding source | own follow-up PR; findings convert to declared rows |

**What stays distinct** (anti-over-merge): `Independence`/`Placement` remain sibling *readings* of the DependencyView (FLAG D); the affected set is a forward-only **peer**, not a materialization instance; mutable keyed state is a database, out of scope by construction.

**Sequence:** **C1** (this PR) ladder + zero arms + convergence + docs + live CI gate. **C2** provider rows for the correct live instances, each with a warm==cold receipt. **C3** = W4, the resolve-store scope fix (first *derived* eviction; the 9 GB receipt). **C4** emitter reference-tier demotion pricing (ownership officially = eval-frame provider selection; seed-touching, time with emit-stage work). **C5** ParseTable/`cached_stage` grounding on the Realization carrier (the standing DESIGN §2 debt, now with a home).

## 10b. Migration census (exhaustive sweep, 2026-07-09) + forward wiring — nothing gets reverse-wired again

**Scope ruling (operator, 2026-07-09):** extdeps is fine for *ground truths and interfaces* — that is its job — but the consolidation target is **materialization/realization in std, asap**: the displaced cost is every future re-invented memo, and it stops being paid the day the fleet sees materialization actually used (a working example beats any lens at preventing re-invention). And **v1-specific (compiler-internal) mechanisms are out of migration scope** — the seed retires with self-host, so census groups A–D below are *inventory only* (each a declared retire-with-seed row, never a migration lane); the W4 9 GB resolve-store pain, if it still bites before the seed retires, is an ops fix, not a modeling target. What remains live: **E** — the .dag substrate's own carriers (v2 `ParseTable`, `cached_stage`), promoted to the front as the demonstration; and **F** — the CI/shell providers. C3/C4 as modeling lanes are dropped by this ruling (C4's value-tier *vocabulary* stays — it is std and already landed; only the seed-emitter refactor drops).

**The §3 discovery that reorders the sequence.** The tree already carries a second cache-description vocabulary: `dag/extdeps/cache/cache.dag` holds a 9-row `cache_catalog` of `CacheInterfaceCatalogFacts` (sccache, actions-cache, BuildBuddy CAS, cargo target dir, rustup store, resolved-graph cache, parse-table memo, parse/typed-module caches) with `CacheInterfaceId` identities in `std.cache_identity`, cited key-derivation (`CatalogKeyInputs = ContentAddressed | NativeInternal{cited_inputs}`), placement and IO projections — plus `extdeps/realization/{resolved_graph,parse_table_memo,reconcile_in_process,hermetic_fixture}.dag` modeling the seed's carriers, and a warm==cold purity oracle (`extdeps/realization/cache_purity.dag`, realized at `resolved_graph_cache.rs` `audit_warm_equals_cold`). The ladder's `CacheProvider` row hand-writes tier/keying/eviction facts beside that catalog — a parallel representation the moment both describe sccache. The §3 split is the extdeps shape/transport/policy rule applied to caches: the **catalog** owns what a cache system *is* (cited upstream facts: keying, eviction, placement); the **provider row** owns the *role* it plays at one obligation (scope, coverage, discharge). Neither may restate the other's half.

**C0 — ground provider rows on the catalog (first migration step, before more C2 rows).** `CacheProvider.id: String = "sccache"` is a stringly nickname for `sccache_local_id: CacheInterfaceId`; the tier facts (`CasTier{ContentKeyed}`, `SpacePacked`) duplicate `extdeps/cache/sccache.dag`. The fix respects the layer arrow (std ← extdeps: the ladder cannot import the catalog): a projection **beside the catalog** — `provider_from_catalog(row: CacheInterfaceCatalogFacts, scope, coverage) -> CacheProvider` — derives tier axes from cited facts (ContentAddressed → `KeyAddressed{ContentKeyed}`; placement → the `Placement` axis; invalidation trigger → `EvictionPolicy`), exactly the `cited_operating_system_surface` dispatch pattern (dispatch lives in extdeps, std stays projection-only). `ci_sccache_provider` then becomes `provider_from_catalog(sccache_local, scope: root, coverage: [release build])` and its hand-typed tier facts are deleted. Receipt: the ladder witnesses unchanged; `ci_materialization.dag` loses its duplicated facts; a catalog edit (e.g. sccache eviction policy) flows into the ladder with zero provider-row edits.

**The exhaustive census (mechanism grain; sweep receipt 2026-07-09).** 52 mechanisms in six groups; each row's migration is (i) provider/catalog description, (ii) live gate binding its frame demands, (iii) keep-now-governed or dissolve. Counts: **A** process/thread-lifetime stores 8 (two families: a 9-site `OnceLock` derived-fact family and the **311-site generated `static CACHED` nullary-memo family** the emitter stamps on every compiled constant); **B** within-walk/scoped memos 13; **C** interning/dedup 5; **D** reference-sharing (Rc env chain, im-rc HAMT, COW helpers, `take_owned`) 8; **E** .dag-layer carriers 12 (incl. the catalog umbrella itself and v2 `ParseTable`); **F** CI/shell-layer 6. Group dispositions:

| group | ladder reading | migration action |
|---|---|---|
| A: process stores (`PROCESS_RESOLVE_STORE`, `MODULE_GRAPH_FACTS_CACHE`, OnceLock family, 311-site `static CACHED` family, on-disk resolved-graph cache) | MemoTier@process; the on-disk cache is already CAS/ContentKeyed + SizeBounded and *modeled* — C0 wiring only | `PROCESS_RESOLVE_STORE` = the W4/C3 wrong-scope receipt (name/path-keyed, ⊤-lifetime, 9 GB); nullary families are sound (key = the compiled-in tree, evict = process exit) and become governed **by construction** when the emitter stamps them from provider rows — one roster row for the whole 311-site family, since one emitter owns it |
| B: scoped memos (`parse_cache`, `typed_module_cache`, `TypeEnvCache`/ancestry, cross-batch `walk_memo`, `pure_call_memo`, `parse_table_memo`, interpreter node-ptr micro-memos) | MemoTier@walk, `ScopeExit` derived | `parse_cache` (file-path key) and `typed_module_cache` (module-name key) are **already flagged impurity witnesses** (`reconcile_in_process.dag` Phase 2c) → content-key or declare the staleness envelope; `pure_call_memo`/`parse_table_memo` are content-keyed done right → provider rows + warm==cold receipts |
| C: interning (`InternTable`, `SymbolInterner`, fnv1a64 primitives) | ReferenceTier, ContentKeyed, grow-only | scope-exit rows (die with index/ctx); fnv1a64 is the *keying primitive*, not a store — its §3 dual-surface thread (`std.content_hash`) already tracked in DESIGN |
| D: reference shares (Rc `Env` parent chain, im-rc HAMT, `rc_*` COW helpers, `take_owned`, ctx shared corpora) | ReferenceTier realizations; COW = the priced Reference→Copy demotion edge | the value tier (`value_materialization`) now *describes* these exactly; C4 = emit stage selects them through it, so demotion is priced never silent (#6249) |
| E: .dag carriers (v2 `ParseTable`, `cached_stage`, extdeps/realization facts rows, `RecordedFixtureStore` + hermetic-fixture model) | Memoize@process / ArtifactTier@disk | `ParseTable`/`cached_stage` ground on the Realization carrier (C5); the facts rows are the catalog side of C0 — wire, don't re-model; hermetic fixtures join `cache_catalog` (sweep found them un-enrolled) |
| F: CI/shell (sccache ✅ F-commit, actions-cache, cargo target dir, rustup store, BuildBuddy CAS, build-if-absent †) | CasTier/ArtifactTier@fleet | remaining rows = C0 projections + warm==cold receipts (C2); † build-if-absent = `RefusedExistenceKeyed` wall, landed — its .dag prose rows stay as the cited anti-pattern receipt (#6352) |

**Forward wiring — three walls so a cache can only be *born* as a provider row** (the "apply generically, never reverse-wire" answer):

1. **Demands derive from the DependencyView, not declarations.** When spine `materialize` is the single execution door (Half A), every computation that runs *is* a `FrameDemand` by construction — enrollment cannot be forgotten because it is not a step. Until Half A lands, the CI gate's pattern (extract demands from the live authority the artifact is emitted from, never a fixture copy) is the interim discipline.
2. **The hand-cache shape wall.** A `.entry().or_insert_with(pure_f)` / new `OnceLock` in the seed, or a fresh Map-with-lookup-insert carrier in .dag, is the *shape* of an undeclared provider. A roster-bound lens (same containment-guard pattern as the realization-vocab guard) reds any instance outside a declared provider realization — the 311 emitted sites are one roster row (one emitter), hand-rolled strays are each their own violation. New undeclared caches become unwritable.
3. **Provider rows are the only authoring surface, and they must be live.** An inert provider row (no demand it covers — the inert-lens backstop pattern) reds; a store-tier discharge without a warm==cold receipt reds (reuse `cache_purity.dag` / `audit_warm_equals_cold`, don't re-mint the oracle). A provider that exists only to be pointed at is coverage by illusion.

**Sequence (re-scoped per the ruling):** **C0** ✅ landed — `extdeps/cache/materialization.dag` `provider_from_catalog` derives provider rows from cited catalog rows (mechanism→class eviction mapping: InProcess ⇒ ScopeExit; Ttl/Lru/SizeBounded ⇒ SpacePacked; Never/Manual outside a process ⇒ `ProjectionRefused`, counted by the enrolled `ci_catalog_projection_has_no_refusals` witness; keying refuse-leaning: HandAuthored or prefix-fallback ⇒ ExistenceKeyed until a key-completeness receipt upgrades it). The CI sccache row is now derived, hand-typed tier facts deleted, 6/6 witnesses green. Next: **ParseTable/`cached_stage` grounding** (was C5, now first — the .dag-substrate demonstration; prerequisite is inhabiting the v2 Realization carrier, the standing DESIGN §6 debt) → **C2** remaining CI/shell provider rows as C0 projections + warm==cold receipts (actions-cache will project ExistenceKeyed until its key-completeness receipt lands — correct, that is the staleness class). The forward walls land with the row that makes each true (wall 2 with C2's roster; wall 3 with the first C2 receipt; wall 1 rides Half A). The two `EvictionPolicy` vocabularies (cited mechanism in `std.cache_interface` vs required class in the ladder) are distinct concepts bridged only by the projection — the mapping's single home.

## 10. Dissolution triggers

- **Hash-consing (§3/§5):** content-addressed construction makes pure forks unwritable; the pure arm dissolves into construction. The effect/placement arms persist.
- **Spine `Share` / Half A:** when materialization describes all computation and `materialize` dedups by construction, this qualifier *is* `materialize`'s key — no separate lens remains (the #6372 `WallAfterGrounding → RealizationDispatch` dissolution, completed).
- **`IdentityUnknown` drain:** each typed-cause bottom dissolves when its named concept is grounded or enforcement added — the lens's own shrinking backlog. Never a permanent "undecidable" (§4).
