# Codebase-wide duplicate-work lens — design

**Status:** DESIGN (pre-implementation). Grounds entirely on authorities that exist today; **no S2 / execution-spine dependency** (§7). The discriminant must be proven by the witness (§8) before any corpus fold — if the discriminant is wrong the lens is noise, and that is the whole risk.

**Origin:** the operator's original framing — "detect a redundant dependency and force a rewire; kill compile → … → recompile" — generalized from the point-instance `v2.lens.duplicate_computation` (shell/argv transports, #6372) to the whole codebase, then sharpened through five refinements (recorded in §2–§5). This doc is the converged model; the existing shell lens becomes one instance of it (§6).

**Principle roots:** §2 (minimize redundancy — one computation, one producer), §3 (single authority — a forked realization is a §3 violation at the realization layer; and **an undecidable identity often *is* a §3 fork**, §5), §4 (closed + bounded ⇒ decidable-with-constraints — the "richer source always exists"), §5 (correctness by construction: Delete > Share > Cache; fail-closed bottom, no absorbing fallback), §6 (priced in displaced cost — the moat is the located root cause, not the symptom), §7 (the same lattice the complexity/termination lenses already use).

---

## 1. The law — duplication is materialization, not fork

The first design drove at *structural forks* (the same work written twice as two nodes). Testing that against real pain (§3) broke it: the flagship case — CI building the release binary three times across independent jobs — is **not** a fork. It is **one** modeled computation materialized N times with no sharing. The correct, unifying law is `materialize` itself:

> **Duplication = N materializations of one computation-identity with no Share edge between them.**

Two independent pieces:

- **computation-identity** — "is this the same work?" Answered by `content_hash` (`src/v2/std/node.dag:1449`), which is **occurrence-independent** (`canonicalize_node_for_content_hash` stamps `SyntheticOccurrence`, `:594`) — so the same operation on content-identical inputs hashes equal at any position. This is "same expected inputs and outputs at an intent level," made a decidable structural fact. Identity is a *lattice*, not a boolean (§4).
- **multiplicity without sharing** — the same identity is materialized more than once and no Share edge connects the materializations. The **source** of the multiplicity (§5) is the axis that picks the fix.

The substrate does **not** hash-cons (`node_with_occurrence_id`, `:115`, mints a distinct `occurrence_id` per node), so forks genuinely exist — but a fork is only the *degenerate* multiplicity source, not the general case.

## 2. Computation-identity is a bounded lattice (§4), not a boolean

"Same intent-level inputs/outputs" has graded strength. In a **Turing-complete** language the strongest form (semantic equivalence of structurally-different programs) is Rice-undecidable and permanent. But `.dag` is **closed, bounded, and total** (§4: forward execution, finite measures, `Int` grounded on machine widths `Int8..UInt128`, no non-termination). Over a bounded domain, extensional equivalence is **decidable by enumeration** — it terminates because everything terminates. So semantic equivalence here is not Rice-impossible; it is decidable *once the bound is supplied*, at a cost equal to the domain size (§6-priced). The lattice, mirroring `DescentEvidence` (`dag/std/termination.dag`) and `UnknownComplexity { diagnostic }` (`src/v2/lens/complexity.dag`):

```
type ComputationIdentity
  = StructurallyIdentical                       -- content_hash equal          (cheap wall, top)
  | NormalizedIdentical    { normalizer: ... }  -- equal under a declared normalizer (cheap wall)
  | ExtensionallyIdentical { bound: ... }       -- equal on the modeled bounded domain (§6-priced wall)
  | IdentityUnknown        { cause: IdentityUnknownCause }  -- fail-closed BOTTOM
```

The top three are **walls** (decidable, sound; the second relative to a declared normalizer, the third relative to a modeled bound). `IdentityUnknown` is the fail-closed bottom — but see §4: it is never a permanent "undecidable."

## 3. Testing the law against real pain — the corrected coverage table

Five real, painful cases from the live tree / receipts, each run through the law:

| # | Case | identity | multiplicity source | verdict |
|---|---|---|---|---|
| 1 | **Lever-A double compile** — two `gunbc compile` in one script (#6361) | content-hash eq | **fork** (2 nodes, 1 hash) | wall **now** — caught by #6372 |
| 2 | **CI release build ×3** — `ci_release_build_script()` run by `ci_job`(:381) / `ci_regen_job`(:414) / `rust_tests` warm(:255), all `needs:[]`, no artifact edge | content-hash eq (same fn) | **placement** (1 node, 3 runners, no Share edge) | wall **after modeling** placement — **the flagship** |
| 3 | **resolve ×4 cross-batch** (M1) — one `resolve_entry_graph`, 4 batch call-sites | content-hash eq | **runtime-call** (1 node, N demands) | wall **now** (static demand count); shipped fix was a *cache* (M1) — dispreferred |
| 4 | **merge_envs O(M²)** — parent surface recomputed per fold iteration (#6360) | content-hash eq per-iter | **runtime-loop** (loop-invariant) | wall **after modeling** free-variable scope |
| 5 | **intern_table per-module** O(corpus²) (#5867) — each module builds its own table | wholes **differ**; shared *sub*-tree eq | **wrong-scope** | 5a shared-subtree: wall **now** (content_hash over subtrees). 5b genuinely-different-but-equivalent: wall **after supplying bound** |

The first design caught only case 1 — the rarest class. The law + multiplicity axis covers all five.

## 4. Undecidability = missing concept or missing enforcement (the reframe that empties the bottom)

`IdentityUnknown` is **not** "the universe forbids knowing." In a closed, bounded, grounded substrate, an undecidable identity is a **symptom of anemic modeling** (§2/§4): we are missing the *concept* that unifies two things, or the *enforcement* that canonicalizes them. §4 states it — "in a closed system a heuristic is never necessary; the richer source always exists or can be written" — so reaching for "undecidable" *is* reaching for a heuristic where a richer source was available but unmodeled.

The sharpest instance: **the undecidability often *is* the §3 fork.** Two computations are "hard to prove equal" precisely because they are two structures for one concept. Ground them on a single authority and structural identity becomes trivial. (Receipt: the old "vendor" fork — `CpuVendor` closed enum vs `GpuFacts.vendor` stringly — made "same computation?" hard; grounding on `Vendor<Domain>` made identity trivially structural. The undecidability *was* the fork.)

So the bottom carries a **typed cause** — conflating the causes would be the state-space-conflation failure mode (§5):

```
type IdentityUnknownCause
  = MissingConcept     { candidate_authority: ... }  -- two structures, one concept: ground them (§3)
  | MissingEnforcement { mechanism: ... }            -- canonical construction / normalizer absent
  | BoundUnmodeled     { domain: ... }               -- extensional check available once bound declared
```

Every entry in §3's "after modeling" / "after bound" column resolves to one of these:

| symptom | cause | missing |
|---|---|---|
| `sort(sort x)` vs `sort x` | idempotence law undeclared | `MissingConcept` (algebraic law) |
| build ×3 runners redundant? | placement + share-edge unmodeled | `MissingConcept` (`PlacementDependsOn` exists, unused) |
| will this loop recompute? | free-variable scope uncomputed | `MissingEnforcement` (the analysis) |
| pure fork writable at all? | construction not canonical | `MissingEnforcement` (hash-consing) |
| two different algos, same fn | input domain bound undeclared | `BoundUnmodeled` |

**Consequence for the lens's product.** Its deliverable is not "here are dups." It is **"here is redundant work, and here is the specific concept or enforcement whose absence lets it exist."** That is the moat framing (§6): the lens locates the *root* (a missing authority), not the symptom. It closes the loop to §2/§3 — a duplicate is a failed decomposition, and the lens names which one. This makes the lens, at its limit, the **leaf-side decomposition-debt detector** parked as an open thread ("can a lens mechanically diagnose the leaf-side of decomposition?"): `IdentityUnknown { cause: MissingConcept }` *is* that finger.

The bottom is therefore a **located, typed modeling backlog item**, never a graveyard. The undecidable *categorization* is itself decidable (which cause applies is a decidable classification) — exactly as `UnknownComplexity { diagnostic }` categorizes decidably what it cannot compute.

## 5. The multiplicity-source axis — same law, one axis picks the fix

Identity (§2) says "same work." The **source** of the multiplicity says how to fix it. This is the axis the first design collapsed by assuming every source was "fork":

| Source | Detect via | Fix | Static now? |
|---|---|---|---|
| **fork** | dup content_hash, distinct `occurrence_id` | delete → rewire consumers to survivor | ✅ (case 1) |
| **placement** | one identity, N consumers across a placement boundary, no Share edge | add artifact-Share edge (`needs: build` + upload/download) | ✅ after modeling placement (case 2) |
| **runtime-call** | one node, N `DataDependsOn` demands | memoize / hoist producer | ✅ static count (case 3) |
| **runtime-loop** | loop-invariant subtree in a fold (free-var ∌ loop var) | hoist out of loop | ✅ after free-var analysis (case 4) |
| **wrong-scope** | shared content-identical *subtree* recomputed at wrong grain | re-scope the intent to one authority | ✅ subtree-hash (case 5a) |

Reuse (one node, N in-edges) is the goal state, never flagged — it is what every fix *produces*.

### Safe-to-collapse and worth-collapsing (the two gates that survive from the first design)

- **Safe to collapse (purity = license, idempotency = the effect-case proof).** Purity is not what makes work duplicated (identity settled that); it is what makes the fix *provably safe*. Pure node → delete-and-rewire is behavior-preserving by construction. Effectful node → safe only if the effect is idempotent/collapsible, a proof `dag/std/effects.dag` already discharges: `is_idempotent_effect` (`:31`), pairwise `create_double_init_collapsible` (`:73`). Two content-identical `AppendEffect`s (non-idempotent) genuinely happen twice — *not* redundant. **Extension owed:** a content-hash group is N-ary but `create_double_init_collapsible` is pairwise, so the lens needs `effect_group_collapsible(shapes)` = a fold over the group reusing the pairwise authority (not a new authority). This is the one genuinely-new piece.
- **Worth collapsing (cost floor, §6).** Every `[]`, `Int(0)`, bare atom is a content-twin; flagging them is the purity-trap flood. Only a group whose shared computation clears a cost floor (`src/v2/lens/cost.dag`) is a wall. The threshold selects *what to report*, never *whether to fail open* — a reporting scope, not an absorbing fallback (§5).

## 6. Fix hierarchy and the relationship to #6372

**Delete > Share > Cache** (the operator's ordering, = §2/§5):

1. **Cache / Memoize — dispreferred.** Keep the duplicates, memoize. The confession that the redundancy was *not* removed (§2's "eleven hand-rolled `HashMap` caches"). Sharp corollary: **every cache in the tree is a map of known redundancy** — `pure_call_memo`, `ParseTable`, `resolved_graph_cache`, sccache, and M1's `walk_memo` (case 3's shipped fix) each exist because someone knew work would be redone. The caches are this lens's own candidate-site list.
2. **Share / rewire — preferred.** One node / one artifact, N consumers. The actionable output.
3. **Construction / hash-cons — endgame (§5 strongest).** Content-addressed construction returns the existing node; a pure fork becomes **unwritable** (`MissingEnforcement` supplied). The lens sizes this endgame.

**#6372 becomes an instance.** `v2.lens.duplicate_computation` already models this shape at argv grain: `ComputationDemandFact { computation_key, guard, replication }`, grouped by key, refused when N unguarded demands share a key, with a declared-`ReplicatedOracle { runs: N }` escape for intentional duplication (the emit-determinism x2 oracle, refused if observed ≠ declared). Its `computation_key` is the argv projection of `content_hash`; its guard is the guarded-fallback-never-duplicates rule; its replication escape is the declared-exception the general lens needs verbatim. So the general lens **subsumes** it — the CI-job lens is not a separate point-thing, it is this lens at placement grain.

## 7. Why no S2 dependency

Three threads were conflated; only the third needed the spine:

| Thread | Needs spine? | Home |
|---|---|---|
| detect/enforce duplicate **dependencies** (the original ask) | **No** — static structural fact | this lens |
| **measure** re-evaluation (recompute-trace) | No — a diagnostic | landed #6372 |
| make **eval-grain** duplication cheap (runtime Share/memoize) | **Yes** | execution-spine `materialize`/Share |

The safe-to-collapse split *is* this boundary: the **effect / placement** grain is a static wall (decidable now — sparse effect/placement nodes, one-pass content-hash grouping, no whole-corpus N² eval walk). The **pure-value runtime** grain (the ~135k eval-duplicates the recompute-trace found) is the spine's `Share`, correctly deferred — but the pure *node* fork is still statically detectable and rewireable, which is what the operator wants. **One law, several grains; the placement/effect grains ship now, the eval grain waits for the spine.**

## 8. De-risking — the witness (build FIRST), placement-grain first

Before any corpus fold, a witness proves the discriminant. **Placement-grain is the first witness** (the flagship, and the grain where the missing concept is nameable and the fix concrete):

- **RED — placement duplication.** The three-job CI model: one `ci_release_build_script` identity materialized at 3 `needs:[]` jobs with no artifact Share edge → **exactly one** flagged materialization-without-share, naming the rewire (one build job → artifact → `needs: build` + download). Red control: add the Share edge (model `needs: build` + upload/download) → violation disappears.
- **ADMISSIBLE — declared per-placement.** A computation legitimately required per-placement (e.g. a per-runner health check) carries a declared escape (the `ReplicatedOracle` analogue) → not flagged; observed ≠ declared → flagged.

Then the identity-lattice cases (fork/effect/cost), carried from the first design:

- **GREEN — reuse.** One node, two in-edges → must not flag (red control: flag reuse ⇒ multiplicity-source axis is broken).
- **RED — pure fork.** Two distinct-occurrence content-identical pure nodes above the cost floor → one violation (red control: perturb one subtree so hashes differ → violation disappears, proving content-identity not position).
- **EFFECT-GATED.** Two `AppendEffect` (non-idempotent) → not flagged; two `CreateIfAbsent`-equal-key → flagged (proves D3 reads `effects.dag`).
- **COST-FLOOR.** Two `Int(0)` below floor → not flagged; drop floor to 0 → flagged (proves the floor suppresses the flood).
- **BOTTOM.** A structurally-distinct pair with no declared normalizer/bound → classified `IdentityUnknown { cause }` with the *typed cause*, reported never-merged (red control: a silent merge here is the absorbing-fallback §5 forbids).

## 9. Sequence and scope

- **In scope now:** the `materialize`-law lens over the corpus — identity lattice (structural + normalized), multiplicity-source axis (fork + placement + runtime-call + subtree), effect-idempotency + cost gates, declared-exception escape, and the typed `IdentityUnknown` bottom. Placement-grain witness first.
- **Deferred to the spine:** pure-value *runtime* Share (eval-grain memoization). Static detection/rewire of pure node forks stays in scope.
- **Order:** (a) placement-grain witness on the CI build ×3 case; (b) the identity-lattice + effect/cost witnesses; (c) `effect_group_collapsible` N-ary fold over the pairwise `effects.dag` authority; (d) corpus fold; (e) fold #6372 in as the argv-grain instance.

## 10. Dissolution triggers

- **Hash-consing (§3/§5):** content-addressed construction (`MissingEnforcement` supplied) makes pure forks unwritable; the pure arm dissolves into construction. The effect/placement arms persist (not value-shared by hash-consing).
- **Spine `Share`:** when demands are execution-spine `DependencyView` nodes and `materialize` dedups by construction, the eval-grain residue retires to the undeclared-demand residue — the same dissolution #6372 declares (`WallAfterGrounding → RealizationDispatch`).
- **`IdentityUnknown` drain:** each typed-cause bottom dissolves when its named concept is grounded or enforcement added — the lens's own backlog, shrinking as the model matures. It never becomes a permanent "undecidable" (§4).
