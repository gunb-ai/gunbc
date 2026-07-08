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

## 10. Dissolution triggers

- **Hash-consing (§3/§5):** content-addressed construction makes pure forks unwritable; the pure arm dissolves into construction. The effect/placement arms persist.
- **Spine `Share` / Half A:** when materialization describes all computation and `materialize` dedups by construction, this qualifier *is* `materialize`'s key — no separate lens remains (the #6372 `WallAfterGrounding → RealizationDispatch` dissolution, completed).
- **`IdentityUnknown` drain:** each typed-cause bottom dissolves when its named concept is grounded or enforcement added — the lens's own shrinking backlog. Never a permanent "undecidable" (§4).
