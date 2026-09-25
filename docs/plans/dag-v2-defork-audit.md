# Plan — De-fork `dag/` ↔ `src/v2/` (collapse the duplicated standard library)

**Status:** carrier-grounded audit + sequencing · **DESIGN.md + the carriers remain the authority** — this doc is an audit/tracker, not a fact ledger (DESIGN §6 "no parallel-ledger docs"). Each collapse dissolves into a mark on its carrier (a deleted file, a re-pointed import) when it lands. A task's real state is its branch/PR, not this file. Linked from `ROADMAP.md` §5 *Self-host v2 → delete `src/v1`* (the de-fork sub-lane).

**Re-verified against the live tree on 2026-06-22 by execution** (decl-name set `comm` per concept + shared-type-body diff). This pass **corrected a load-bearing project assumption**: the dag↔v2 std fork is *not* "mostly temporary v2 mirrors of dag." It is **2 true mirrors + 2 pure name-collisions + 7 divergent groundings** — the same concepts grounded on *different axes/realizations*. The hard part of the de-fork is therefore a grounding-unification **design** (operator-owned, downstream of the numeric tower #5428 and the model↔realization grounding), not a mechanical sweep. The prior version of this doc ("11 mechanical PRs / 5 collapses / `Classical`↔`Bool` nickname") was wrong by execution and is superseded below.

**Re-confirmed by execution 2026-06-23 (post-#5640), for the
[#2 resolver-wall](resolver-type-name-collision-wall.md) (PR #5652) Q2 ruling — see §2A.** Net distribution now: **2 true mirrors** (`reducible` DONE #5507, `measure` in-flight
#5509) **+ 6 divergent groundings** (`algebra`/`logic`/`nat`/`integer`/`float`/`effects`) **+ 2
v1-artifact name-collisions** (`node`, `coercion`) **+ 1 RESOLVED** (`verification`, #5640 — it was a
name-collision resolved by rename-apart, *not* a grounding-merge, so it leaves the cluster). **Executed
verdict: cant-unify-yet (needs a milestone later than the grounding de-fork) == {node, coercion}
exactly — no third stuck mirror — so the signed `flag-ANY + {node,coercion}` wall policy HOLDS.**

---

## 2A. 2026-06-23 executed Q2 confirmation (post-#5640) — supersedes the `verification` census row

Read-only, by execution. Structural extraction of every top-level `type` decl in `dag/std/<b>.dag`
vs `src/v2/std/<b>.dag`; shared **unqualified type names** compared with the #2 guard's own
`structural_inequality` predicate (normalized body equality) — this run is also the §5 proof that the
predicate classifies correctly. Reachability = BFS of all **351** `_test.dag` floor entries' whole-module
import closures testing whether `std.<b>` and `v2.std.<b>` co-occur in one closure (= the guard fires).

| basename | shared type-names | structural verdict | floor co-occurrence | category | unify status |
| --- | --- | --- | --- | --- | --- |
| **algebra** | 16 (13 differ; 3 byte-identical `Lattice`/`Magma`/`Ordering`) | same concept, divergent **encoding** (dag flat records vs v2 compositional/coproduct) | **LIVE — 75 entries** | grounding (Root-A: v2 coproduct authority RULED 2026-06-22) | cant-unify-yet, **has path** (de-fork-resolved, not exempted) |
| **nat** | 1 (`Nat`: `= CommutativeSemiring<Magnitude>` vs `= Zero \| Succ`) | same concept, divergent **model** (semiring alias vs Peano coproduct — representational, not nominal) | **LIVE — 138 floor closures** (946 scanned; was 4 @ 351) — **risk measure, not a task count** (see §2A note) | grounding (#5428 numeric tower; census nat-grounding-unification-design.md) | **DESIGN READY** · IMPLEMENTATION LARGE/ATOMIC · SEQUENCED WITH integer+float repoint (not last) |
| **effects** | 3 (`EffectShape`/`KeySource`/`CreateCause`; `EffectShape` body re-modeled on a different axis) | divergent **axis** (operation-kind vs idempotency-class) | latent (0) | grounding | cant-unify-yet, has path (axis-authority decision) |
| **float** | 3 (2 differ; 1 byte-identical `Float = Float64`) | same concept, divergent encoding | latent (0) | grounding (#5428) | cant-unify-yet, has path |
| **integer** | 14 (11 differ only on `MachineWidth<8>` vs `<Word8>`; 3 byte-identical) | same concept, near-identical (token diff) | latent (0) | grounding (#5428) | cant-unify-yet, has path (mechanical once the tower lands) |
| **verification** | **0** (was 1: `TestClaim`) | **RESOLVED #5640** (dag record renamed `TestClaim → AssertionClaim`) | none now | **RESOLVED name-collision** (not a grounding) | **DONE** |
| **logic** | **0** (dag `Classical` vs v2 `Bool`) | divergent concept, **no name overlap** | none (0 shared) | grounding-or-rename (undecided) | **no type-name collision → guard never fires** |
| **coercion** | **0** (dag cast vocab vs v2 `coercion_fold`) | divergent concept, **no name overlap** | none (0 shared) | v1-artifact name-collision | dissolve-on v1-delete; **0 type-name collisions → exemption vacuous for the guard** |
| **node** | **0** (dag v1-only `compiler_inductive_fields` vs v2 Node substrate) | divergent concept, **no name overlap** | none (0 shared) | v1-artifact name-collision | dissolve-on v1-delete; **0 type-name collisions → exemption vacuous for the guard** |

**Co-occurrence closure counts (algebra 75, nat 138):** each integer counts floor `*_test.dag` **import closures** where both `std.<b>` and `v2.std.<b>` are reachable — a **risk and closure-population measure** for the LIVE fail-open pair, **not** a migration task count or work estimate. New floor entries and tests can grow these numbers legitimately; **do not** ratchet on the integer (gate the unsafe facts instead — nat census §7.6 in [nat-grounding-unification-design.md](nat-grounding-unification-design.md)). Until the fork dissolves, mixed closures silently shadow record-with-record (benign today for nat; coproduct-variant-drop for algebra).

**Confirms parent's derived distribution; cant-unify-yet == {node, coercion} exactly; flag-ANY +
{node,coercion} HOLDS.** Two scope refinements (strengthen, do not refute the ruling):

1. **The {node,coercion} exemption is VACUOUS for the type-name guard.** node/coercion (and logic,
and now verification) share **zero** type names — only the *basename*. The guard keys on a shared
unqualified **type name** within one closure, so it never fires on them; their de-fork is a
*module-basename* rename (Route-C / v1-delete), a different surface. The guard needs no
{node,coercion} roster entry to land green — keeping it would be a dead (non-firing) exemption,
itself a §5 smell. Recommend marking the roster entry explicitly "Route-C basename, non-firing"
or dropping it from the guard model.
2. **The guard's real lands-green gate is the grounding de-fork of the shared-type-name basenames**,
fronted by the LIVE pair `{algebra (75 floor entries), nat (138)}` — they silently fail-open today
(benign only because they shadow record-with-record, not the coproduct-variant-drop that broke
`verification` under A1), and a flag-ANY wall reds them on landing. `{effects, float, integer}`
are latent (no co-occurrence today) but re-arm risks (exactly how verification went latent→LIVE
under A1). So the wall lands-already-green only after `{algebra, nat}` (then the latent three)
de-fork — that sequence, not the {node,coercion} exemption, gates activation.

---

## 2D. 2026-09-24 — the shared type-name census, classified by what the body's names RESOLVE to (the one list)

**This is the single census of type names declared in both `dag/std` and `src/v2/std`; it supersedes the per-basename `shared type-names` column of §2A and no second list exists.** Population: every top-level `type` declaration name under each tree, joined by name — 32 names at this writing. Each side is classified by resolving every capitalized name in its body to the declaration it binds (its own module's `type`/`data`/`fn`, else the module whose `import` list carries it) and comparing the resolved referents, NOT the body text: `Float = Float64` is byte-identical on both sides and binds two different `Float64`s, and `IntPlatform` differs by one space while every referent differs. Text comparison is what misclassified six pairs before. A **meaning fork** is one name for materially different concepts (DESIGN §3); it is resolved by renaming or deleting one side at its declaration, delete-first with no alias, and the new name is taken from the cited framework rather than coined.

| name(s) | dag/std side (resolved) | src/v2/std side (resolved) | class | disposition |
| --- | --- | --- | --- | --- |
| `ProjectionKind` | `std.planar_projection` — drawing views (`PlanFromAbove`, `ElevationFromSouth`, …) | `v2.std.projection` — compiler lens kinds; module imported by nobody | MEANING FORK | **RESOLVED (this change):** v2 module deleted — it had zero importers, so deletion, not rename, is the repair |
| `AlgebraCarrier` | `std.algebra` — record: `name`, `AlgebraProfile`, `List<CarrierSpelling>` | `v2.std.rust_leaf_model_claim` — one-arm enum `AlgebraCarrierOrderedRing`; its only datum was imported and never read | MEANING FORK | **RESOLVED (this change):** v2 type + datum deleted (dead); `std.algebra` `AlgebraProfile` `OrderedRingProfile` already names that structure |
| `ApproximateField` | `std.approximate_field` — wrapper: `Field<F>` + rounding/precision/special values | `v2.std.algebra` — ops record (`add`/`zero`/`mul`/`compare`); zero consumers | MEANING FORK | **RESOLVED (this change):** v2 record deleted (dead; ops come from inhabitance, DESIGN §4) |
| `EffectRequest` | `std.authorization_profile` — authorization request: subject / effect / target / context / evidence (XACML Request shape) | `v2.std.runtime` — a performed effect operation with its arguments and continuation frame | MEANING FORK | **RESOLVED (this change):** v2 renamed `OperationCall` (+ `OperationCallKind`), Plotkin–Pretnar *Handling Algebraic Effects* (LMCS 2013) `op(v; y.c)` = *operation call* |
| `Float32` / `Float64` | `std.float` — aliases `= Real32` / `= Real64` (algebraic reals at a width) | `v2.std.float` — records over `FloatBody` (sign / biased exponent / trailing significand) | MEANING FORK | **RESOLVED (this change):** v2 renamed `Binary32` / `Binary64`, IEEE 754-2019 §3.6 interchange-format names; v2 `Float = Binary64` |
| `RankingDimension` | `std.termination` — kinds of well-founded measure (`TreeSize`, `ListLength`, …) | `v2.std.cardinality` — record `measured: Symbol`, one entry of a lexicographic tuple | MEANING FORK | **RESOLVED (this change):** v2 renamed `RankingComponent`, Ben-Amram & Genaim *Ranking Functions for Linear-Constraint Loops* (JACM 2014): a lexicographic ranking function's *components* |
| `NonNegativeInt` / `PositiveInt` | `std.integer` — `= Nat` / `= Nat where gt_zero` (refinement of `std.nat` `Nat`) | `v2.std.refinement` — record `refined: Refined<Int>` (smart-constructor over `v2.std.integer` `Int`), consumed by the formatter extdeps | MEANING FORK | OPEN — next PR of this wave (v2 side has ~40 consumer sites across `v2.extdeps.formatters.*`) |
| `Float` | `= Float64` → `std.float` `Float64` | `= Binary64` → `v2.std.float` `Binary64` (was `= Float64`, byte-identical text, different referent) | resolution-divergent (text was identical) | OUT OF SCOPE — seed kernel name, awaiting operator ruling |
| `String` | `= FreeMonoid<Char>` → `std.algebra` `FreeMonoid`, `std.types` `Char` | same referents | one concept, two declarations (duplicate) | OUT OF SCOPE — seed kernel name, awaiting operator ruling |
| `List` | `= FreeMonoid<element>` → `std.algebra` `FreeMonoid` | `= std.algebra.FreeMonoid<T>` → same | one concept, two declarations (duplicate) | OUT OF SCOPE — seed kernel name, awaiting operator ruling |
| `Bool` | `std.types` `= True \| False` | `v2.std.logic` `= True \| False` (arms are two more same-name declarations) | one concept, two declarations (duplicate) | OUT OF SCOPE — seed kernel name, awaiting operator ruling |
| `Nat` | `= CommutativeSemiring<Magnitude>` | `= Zero \| Succ` (Peano coproduct) | same concept, divergent model | NEEDS A DECISION RECORD (integer family root) |
| `Int`, `UInt` | → `std.nat` `Nat` (via `std.algebra` `GroupCompletion`) | → `v2.std.nat` `Nat` | same concept, divergent referent | NEEDS A DECISION RECORD (integer family) |
| `Int8`…`Int128`, `UInt8`…`UInt128` | → `std.machine_constraints` `Compose` / `MachineWidth<N>` + `std.integer` `Int`/`UInt` | → `v2.std.integer` `Compose`, `v2.std.machine` `MachineWidth<WordN>` + `v2.std.integer` `Int`/`UInt` | same concept, every referent divergent | NEEDS A DECISION RECORD (integer family) |
| `IntPlatform`, `UIntPlatform` | → `std.machine_constraints` `Compose`/`MachineWidth`/`PointerWidth` | → `v2.std.integer` `Compose`, `v2.std.machine` `MachineWidth`/`PointerWidth` | text near-identical, every referent divergent | NEEDS A DECISION RECORD (integer family) |
| `Compose`, `MachineWidth` | `std.machine_constraints` — phantom composition / width over `WidthResolution` | `v2.std.integer` / `v2.std.machine` — opaque, unconstrained parameter | same concept, divergent constraint | NEEDS A DECISION RECORD (integer family) |
| `TerminationProof` | `std.termination` — `dimensions: List<RankingDimension>` | `v2.std.cardinality` — `non_increasing: List<RankingComponent>`, `strict: RankingComponent` | same question, divergent model | NEEDS A DECISION RECORD |
| `Unit` | `std.types` — empty record | `v2.std.cardinality` — empty record | one concept, two declarations | NEEDS A DECISION RECORD |

---

## 0. Thesis — the only duplication is v2's bootstrap copies of dag

`dag/` is the single authority (the standard library + the grounded `extdeps/` domain models + the CI spec). `src/v2/` is the **compiler**, and a compiler needs a standard library to run. During bootstrap it made copies of pieces of `dag/std` inside `src/v2/std`. Those copies are the fork surface.

"De-fork" is: **delete v2's duplicate copies and point v2 at the dag authority**, until no concept has two homes — and no genuinely historical fork survives (a name shared by two copies, or two names for one concept). Folder/module naming must reflect one authority, not the fork's history.

**The correction (2026-06-22):** only a *minority* of the overlapping basenames are clean copies. Most are the **model↔realization fork** (DESIGN open thread) surfacing across `std`: the *same concept* modeled on a different axis in each tree (e.g. `EffectShape` by operation-kind in dag vs by idempotency-class in v2), or grounded into a realization in one tree and left thin in the other (the numeric tower). Those cannot be "delete + repoint" — repointing breaks consumers at missing-symbol level, and the *shared* type's body itself disagrees. They need a single-authority **design** first.

---

## 1. Cross-tree import — ACTIVATED (the former blocker is dissolved)

The machinery to import `dag/` from `src/v2/` is **wired and on**, proven by execution:

- Grounded `source_root` tagging landed (#5473 `source_root: SourceRootRef` on `DagSourceReadWitness`; #5486 grounded cross-tree admission — the QualifiedName-prefix fallback is **deleted**, and `tree_fundamentality_order(V2Tree, DagTree) = MoreFundamental` is derived, not guessed).
- `src/v2/compiler/03_name_resolve.dag` no longer carries `FundamentalityUnknown`; `admit_import_entry` calls the grounded `cross_tree_edge_decision`.
- **Activation arbiter witness** (#5506, `src/v2/test/claim/cross_tree_real_ingest_activation_test.dag`) feeds a host-tagged `SourceRootIngest` through the *real* grounding machinery (`program_assembly_fold_ingest` builds the QN→`SourceRootRef` index; `source_root_set_from_ingest` builds the active-root set) and runs the real per-edge decision: v2→dag ⟹ `EdgeCrossAdmitted`, tags-reversed dag→v2 ⟹ `EdgeCrossDenied`. So cross-tree import is **live on main, carrier consumed, no operator flip pending**.
- The source-root admission abs-vs-rel host bug (#5473's `source_root_ref_token_for_path`) that blocked the rust gate is **fixed** (#5504): file paths and `--source-root` values are grounded through `repo_relative_dag_path` before matching, so admission is invocation-independent.

Remaining concrete blocker, scoped to the first real cross-tree **data** import only (NOT the std collapses — std collapses resolve cross-tree clean, no `std.*` collision, no `Option`/`Optional`): `src/v2/std/probe_selector.dag:52` — v2 cannot import `dag/gunbc/product/compute_fabric` (`Option<T>` vs `Optional<T>`; `std.*` namespace collision under dual source-root). **RESOLVED (#5904):** `compute_fabric` is now the 130-line thin connector (imports only `std.types` + `std.measure`); the namespace collision is gone. Repro test `dag/test/claim/probe_selector_compute_fabric_import_repro_test.dag` deleted (blocker dissolved).

---

## 2. Fork census, re-verdicted by execution (the real shape)

Three tiers. **Mirror** = v2's used symbols ⊆ dag → mechanical delete + repoint. **Not-a-fork** = shared symbol count is *zero*, the basename is the only collision → rename to disambiguate — **but where the dag-side file is consumed by the `src/v1` seed (import + emitted Rust + a guard test), the rename cascades into the seed and is held** (see the v1-coupled subsection). **Grounding cluster** = the same concept grounded on a different axis/realization in each tree (shared symbols exist, often with a *diverging shared-type body*) → single-authority **design**, operator-owned, downstream of #5428 + the model↔realization grounding. *Not* a repoint and *not* a clean additive merge.

### Mechanical lane (still-deer authority, bright-stag review) — finishable independently

| concept | shared | each side | verdict / state |
| --- | --- | --- | --- |
| **reducible** | 8 (full set) | v2 header self-declares "ported from dag" | **Mirror — DONE** (#5507) |
| **measure** | dag 40-decl authority ⊇ all 10 importer symbols (only 1 importer, `timeseries_signal`, across all trees) | v2 = 3-decl declared mirror | **Mirror — in flight** (#5509, CI-green, ready) |
| **probe_selector** (step-4) | — | `Option<T>` vs `Optional<T>` + `std.*` collision on the `compute_fabric` data import | **DONE** (#5904). Thin connector replaces `compute_fabric`; `std.*` collision gone; repro test deleted. |

### Name-collision renames — V1-SEED-COUPLED (held for ruling)

`coercion` and `node` are genuine not-a-forks (shared symbol count = 0; the dag and v2 files denote different concepts), **but they are not clean-lane renames**: the dag-side file in each is consumed by the `src/v1` bootstrap seed, so renaming its module cascades into the seed. *Caught by execution — importer greps that omitted `src/v1` initially mis-scoped both as low/zero-importer.* **Ruled DEFER** (bright-stag, 2026-06-22): both renames touch `04_infer` (a DESIGN-named load-bearing inference stage), the emitted Rust seed, and a guard test — high cost + load-bearing **now**, for a pure naming-collision fix whose collision is largely a v1-existence artifact. The cost drops toward zero once v1's consumers are gone (DESIGN §7 / ROADMAP §5 shrink-to-zero), and **the deferred work shrinks rather than just moving** (see each row). Dissolution trigger: *when v1's consumers of `dag/std/{node,coercion}` are removed (v1 shrink).*

| concept | shared | each side | v1 coupling | verdict / state |
| --- | --- | --- | --- | --- |
| **coercion** | **0** | dag = cast / type-representation vocab (`TypeCheckpoint`, `CastRule`, `CastSyntax`, `CallableRepr`, `dag_cast_rules`); v2 = coercion-as-homomorphism (`coercion_fold` via `find_witness`, `CoercionWitness/Result/Quality`) | dag side imported by `src/v1/coercion.dag` + `src/v1/04_infer.dag`, emitted to `src/v1/stage0/src/std_coercion.rs` (+ 4 `dag/extdeps/languages/*/types.dag` that survive v1) | **DEFERRED.** The 4 extdeps importers survive v1, so `dag/std/coercion.dag` persists — on v1-delete the rename **shrinks to a v1-free scope** (4 extdeps + v2, no seed / no `04_infer` / no guard test), a clean disambiguation. End-state (DESIGN §4): the *fold* **is** coercion → keep `coercion` for the v2 side, rename the dag cast vocab to `cast_rules`. (#5510 attempted it but broke v1 — missed the `src/v1` consumers — and over-scoped by renaming the v2 side; **closed**, not merged.) |
| **node** | **0** | dag = `compiler_inductive_fields`/`compiler_recursive_types`/`is_compiler_recursive_type` — the complexity analyzer's inductive-structure *authority* (instance of `std/induction.dag`); v2 = 126-decl, 506-importer real Node substrate | dag side imported by `src/v1/04_infer.dag`, emitted to `src/v1/stage0/src/std_node.rs`, guarded by `src/v1/tests/src/source_audit.rs`; `compiler_inductive_fields` is consumed **only by v1, never v2** | **DEFERRED — self-dissolves.** Since `compiler_inductive_fields` is v1-only, on v1-delete `dag/std/node.dag` goes **dead → delete it, collision gone, no rename needed at all.** It is a live v1-only authority today (not a dead imposter); the collision with the v2 substrate is a v1-existence artifact. |

### 2B. 2026-09-22 executed correction to the `coercion` row — shared-symbol-count 0 was the wrong discriminator

**This audit classified `coercion` as a genuine not-a-fork on a shared *symbol* count of 0. Measured by execution, that count answers the wrong question.** The two modules share no symbol and denote no different concept: they answer ONE question — may this value cross from one model to another — and they answer it with **two admission authorities keyed on different identity relations**. `std.coercion` `dag_can_cast` folds over `dag_cast_rules` + `grounded_primitive_coproduct_identities`, keyed on type-name **STRINGS**, and it EXECUTES (`v1.compiler.infer` consults it at argument-versus-formal admission, at literal admission, and in the type-relation fold). `v2.std.coercion` `coercion_fold` asks `v2.std.find_witness` `find_witness` for a structure-preserving witness, keyed on node **IDENTITY**, and has **no established executing consumer**. A shared-symbol count cannot see this, because the two sides cannot share a symbol: they cannot share a KEY.

**The recognition rule, available before any execution: a fixture that wants to ask both authorities about one subject must author the subject TWICE.** No conversion between a type-name spelling and a node identity exists anywhere in the corpus, so `v2.test.claim.coercion.two_algebra_fork` carries `source_type`/`target_type` (strings) beside `source_name`/`target_name` (symbols) on every row. That duplication is not fixture awkwardness — it **is** the measurement. This is a keys/hashing conformance subject (DESIGN §3b names structural node identity as a home there), not a coercion subject, and the finding generalizes: the class is filed as `gunbc.recurring_failure_mode` `two_admission_authorities_keyed_on_different_relations`, with an independent same-day specimen in a different surface (one resource under `Network` and `std.resources.Network`, a diagnostic field carrying the caller's import spelling while the wall compared resolved declaration identity).

**EXECUTED RESULT over seven supplied source/target pairs — the admission sets differ in BOTH directions, so neither side is merely the stricter one.** `coercion_fold`'s only realized preservation rule is `preservation_rule_exact_structural_equality_zip_fold`, whose predicate holds exactly when the candidate is structurally equal to the source, so the homomorphism side cannot SPELL a non-identity crossing at all. **Admitted name-only:** `Int→Float`, `Bool→Int`, `Int→Nat`. **Admitted homomorphism-only:** `String→String` — an IDENTITY crossing, which `dag_cast_rules` carries no row for. That last is the sharpest single fact, because an identity coercion is the one case a reader assumes is free. Both sides matched their OWN independently specified contract on all seven pairs (the declared rosters for one, the structural-preservation predicate for the other), so this is a fork between two correct-by-their-own-lights authorities and not a defect in either.

**NOT REPAIRED, by ruling (snappy-deer-443, 2026-09-22).** The repair is a replacement migration (DESIGN §3, replacement migrations cut over at the root) — one key wins, the other algebra is cut at its root — with its own blast radius, and it is a separate lane that should start from these executed values rather than from the derivation that found them. The DEFERRED verdict in the table above is therefore **superseded as to its reasoning, not its timing**: the deferral was priced as a module-BASENAME rename held for v1-shrink, and the actual subject is an admission-key consolidation whose cost is not a rename at all.

**Instrument:** `gunbc test`-reachable claims in `v2.test.claim.coercion.two_algebra_fork` — `name_keyed_coercion_matches_the_declared_cast_rosters_holds`, `homomorphism_coercion_matches_the_structural_preservation_contract_holds`, `homomorphism_coercion_admits_only_identity_quality_holds`. Each pins ONE algebra to its OWN independently specified contract, so a move in either side's admission set — consolidation included — reds the claim that owns it and the author must restate the row. **There is deliberately no fourth claim comparing the two algebras to each other** (review 70065): with both sides pinned to independent expectations, such a comparison is ENTAILED and its RED is not independently authorable, which DESIGN §4b rules out before the check is written. The agreement relation is DERIVED from the two expectation columns a row carries side by side. Two mutation arms establish discrimination rather than change detection: adding a `Bool→Float` row to `dag_cast_rules` reds ONLY the roster claim, and weakening the structural-preservation predicate reds ONLY the two homomorphism claims.

---

### 2C. 2026-09-22 — the bindings selection fold is a NEGATIVE, and the shape of the negative matters

`std.occurrence_binding` `occurrence_binding_from_candidates` (a FoldZero/FoldOne/FoldMany accumulator) and `v2.std.symbol_index` `symbol_index_lexical_selection` (`match length(candidates)`) are two implementations of one none/one/many trichotomy. Measured over five supplied populations — empty, singleton, two distinct, **duplicate**, three — **they agree on all five, and each independently matches the trichotomy as the modeled contract states it**, so this is not merely "the two match".

**It is NOT agreement by construction, and the distinction is load-bearing.** Agreement-by-construction would mean both routes read one reader, making disagreement unwritable. Here nothing makes them agree: **both contracts are SILENT on duplicates and both realizations inherit that silence.** Neither declares a deduplication step, so a population naming one candidate twice is a two-candidate population in both — by omission, not by rule. The first side to declare a dedup rule silently forks the other. In inventory terms this is **no-evidence with a named absence**, not AgreesByConstruction. **What would close it:** a declared statement, on whichever carrier owns the trichotomy, of whether the rule ranges over list MULTIPLICITY or over the cardinality of DISTINCT candidates. The duplicate row stays enrolled although it is green on both sides — it is the probe that reds when that silence is broken on one side only.

**SCOPE, stated because it is the whole point.** The subject is the selection fold on a SUPPLIED population. Their agreement on a PRODUCTION SITE is not measured and is not measurable today: no v2 resolution site is answered by std's fold, and `std.occurrence_binding`'s own staged-adoption note states that `LexicalLookup` is not an interim consumer because it carries no exact reference-occurrence `Node` or occurrence containment identity. The candidate PRODUCERS (a supplied list versus the containment chain walk) are legitimately layered and are not compared. `symbol_index_lexical_selection` was split out of `symbol_index_lexical_lookup` — same composition, no behaviour change — so the boundary can be asked with its inputs supplied (DESIGN §3, the witness standing rule); the split's pairing obligation is discharged by `the_real_chain_walk_emits_the_population_the_fixtures_supply_holds`, which runs the real `symbol_index_lexical_collect` walk over a real `SymbolIndex` and feeds THAT population into the same selection fold, so deleting the walk cannot leave the suite green. Mutation arms, each redding ONLY the claim that owns the mutated side: collapsing std's fold so it never leaves `FoldOne` reds std's claim alone; making v2's selection answer `Ambiguous` on the empty population reds v2's claim alone. As in §2B there is no third claim comparing the two folds — it would be entailed by these two, both being pinned to the same expected class.

### Grounding cluster (operator authority) — 7 concepts, a unification *design*

Each row is **operator decision-input**: what is shared (and whether the shared body itself diverges), what each side uniquely carries, and the specific grounding entanglement that makes it a design, not a repoint.

| concept | shared | shared body diverges? | dag-only / v2-only | grounding entanglement |
| --- | --- | --- | --- | --- |
| **algebra** | 16 abstract structures (`Magma`…`Field`, `FreeMonoid`, `Ordering`) | no (the structures match) | dag-only 19 = template/codegen machinery **+ `GroupCompletion`/`FieldOfFractions`**; v2-only 74 = **41 `_node`/`_type_node` substrate projections** + the list/fold framework (213 importers) | numeric tower (`GroupCompletion` = `Int = GroupCompletion<Nat>`) **+** model↔realization substrate reflection. Each side owns a different grounding. |
| **logic** | **0** | — (no overlap) | dag-only = `Classical`/`classical_and/or/not`; v2-only = `Bool` + boolean-algebra instance + 6 node-projections + `BoolEncodingFact`/`BoolWidthFact`/`BoolPrimitiveFacts` | Bool grounded into **bit-width** (model↔realization). Operator picks: genuinely different concepts (→ rename) **or** one boolean modeled twice (→ unify on v2's grounded). |
| **nat** | `Nat` (the name) | yes (thin alias vs coproduct) | dag = 4-decl `Nat = CommutativeSemiring<Magnitude>`; v2 = 12-decl coproduct `Zero/Succ` + `nat_cata`/`nat_add`/`nat_mul`/`is_zero`/`nat_lte`/`nat_gte`/`NatAlgebraLawObligation` | **#5428** grounded numeric tower (`Zero → Int(0)`, `Succ → Int(k+1)`). Census complete (2026-08-01): **DESIGN READY** — [nat-grounding-unification-design.md](nat-grounding-unification-design.md); prerequisites discharged (#6341 merged; coproduct keystone green). **IMPLEMENTATION LARGE/ATOMIC**, sequenced with integer+float repoint (not last). |
| **integer** | 14 = the **whole `Int`/`UInt` width tower** (`Int`, `Int8…128`, `UInt`, `UInt8…128`, `IntPlatform`, `UIntPlatform`) | no | v2-only +72 = the arithmetic ops grounded **on** the tower | `Int = GroupCompletion<Nat>` numeric tower. |
| **float** | 3 (`Float`, `Float32`, `Float64`) | no | v2-only +18 = algebraic-vs-bit-level ops | the algebraic-vs-bit-level layer of the numeric tower + bit-width. |
| **effects** | 6 core (`EffectShape`, `KeySource`, `CreateCause`, +3 fns) | **YES — the shared `EffectShape` body is re-modeled on a different axis** | dag = **operation axis** (`ReadEffect`/`UpsertEffect`/`DeleteEffect`/`CreateEffect`/`AppendEffect`) + derivation (`derive_effect_shape`/`OperationEffect`/`compose_effects`); v2 = **idempotency-class axis** (`IsIdempotent(IdempotentShape)`/`IsBreaking(BreakingShape)`) + idempotency machinery + node-projections | DESIGN §4 "idempotency dissolved from an `idempotent:Bool` flag into the `EffectShape` variant" — v2 is that grounding. Unification question: *which axis is the single authority, or are they orthogonal dimensions of one `EffectShape`?* |
| **verification** | 1 = `TestClaim` (**one concept modeled twice**) | **YES** | dag = `TestClaim { kind, label }` simple proposition (+ `TestCase`/`TestNodeRef`/`std.measure Nanosecond`/cost-dimension); v2 = closed assertion coproduct `CompilesClaim`/`DiagnosticClaim`/`EqualsClaim`/`StructuralEqualsClaim`/`RoundTripClaim` (each carrying `anchor`/`Node`/`classification`) | v2's coproduct is entangled with **#5428 + the `Value::Null` straddle fencing + the testgen-oracle** (`GeneratedCrossRepresentationEquality`). Merge + grounding-adjacent. |

---

## 3. Sequencing

**Mechanical lane — proceeds now (still-deer authority + bright-stag review, each PR atomic):**

1. **Mirrors:** `reducible` (DONE #5507), `measure` (#5509). Delete the v2 copy, repoint imports to `dag/std/*`; decrement the `fact_cardinality.dag` cross-tree baseline by *exactly* the deleted symbol count; stay green by execution (the existing witnesses are the oracle).
2. **Step-4 data import:** `probe_selector` `Option`/`Optional` + `std.*` collision, so `dag/gunbc/product/compute_fabric` imports cleanly into v2. **DONE (#5904).**

**Deferred to v1-shrink (the not-a-fork renames):** `coercion`, `node`. v1-seed-coupled; ruled DEFER with the dissolution trigger *"when v1's consumers of `dag/std/{node,coercion}` are removed"* — at which point `node` self-dissolves (delete the dead file) and `coercion` shrinks to a v1-free disambiguation. Not dispatched until then.

**Grounding cluster — held for the operator** (a single-authority unification *design*, downstream of the numeric tower #5428 and the model↔realization grounding): `algebra`, `logic`, `nat`, `integer`, `float`, `effects`, `verification`. These are **not** dispatched as mechanical PRs. The grounded dag authority for each has to be *designed* before any fan-out can repoint to it (the same shape as the project's "substrate migration precedes the ratchet, is not a path to it"). **`nat` census complete (2026-08-01):** per-concept design in [nat-grounding-unification-design.md](nat-grounding-unification-design.md) — **DESIGN READY**; prerequisites discharged (`algebra` `FreeMonoid` shadow #6341 merged; generic-alias coproduct keystone green). **IMPLEMENTATION LARGE/ATOMIC**, **SEQUENCED WITH integer+float repoint** (not last). Operational gate before the wave: bounded P3b PR2 overflow on `std.integer` lands first (#7511 merged; both lanes touch the numeric authority — one writer at a time). **Atomic wave file ownership (one push):** `dag/std/nat.dag` ← coproduct + Peano ops; delete dag semiring alias; `v2.std.nat` ← thin reimport + node-bound law roster only; `integer`/`float` repoint `GroupCompletion<Nat>` to `std.nat.Nat`; all importer repoints in the same push (§5 auto-committer hazard).

**Hazard (atomicity):** the dashboard auto-committer can snapshot a multi-file rename/collapse mid-edit (a symbol deleted in file A while file B still calls it) → an internally-inconsistent intermediate commit caught by a *frozen* merge-sha CI run → phantom "not found in scope" red. `gh run rerun` cannot fix it (it re-merges the broken head); only a fresh consistent push. Stage every file of a rename/collapse together.

---

## 3b. OPERATOR RULING — grounding-cluster anchor (2026-06-22) — the brief

The operator ruled the FreeMonoid/algebra single authority that category (b) was parked on. **Both confirmed:**

1. **Structural authority = the coproduct** `type FreeMonoid<T> = Empty | Cons { head, tail }` (the `src/v2/std/algebra.dag` form). The `dag/std/algebra.dag` record-of-methods form is **derived from inhabitance** (DESIGN §4 — "ops from inhabitance, no per-type ops"), **not** a second definition. The record surface is a projection over the coproduct, not an authority.
2. **Grounded-realization wins.** Generalize the **#5428 `RustCorpusRepr` seam** (`HostNative` → `Nat/Int`=`i64`, `List`=`Vec`; `FaithfulFreeMonoid` → coproduct) from List/Nat to **all FreeMonoid carriers**, so `String = FreeMonoid<Char>`, `List<T> = FreeMonoid<T>`, `QualifiedName = FreeMonoid<Symbol>` are **aliases** grounding to native `Vec` in the seed and to the faithful coproduct in pure-v2. **SUPERSEDED PREMISE (eager-deer-389):** `RustCorpusRepr` is DELETED -- there is no `HostNative`/`FaithfulFreeMonoid` seam left to generalize, and `rust_corpus_repr` no longer exists. Realization is keyed on each type's declaring module. This item is quarry per DESIGN section 3 (existing designs are re-read as quarry, never authority over the replacement): its OBSERVATION about FreeMonoid carriers may still hold, but its proposed MECHANISM is gone and cannot be built.

### Two roots, two lanes (do not collide)

- **Root A — grounding (jolly-cat owns, build-now).** The `src/v1/05_emit_rust.dag` emit-seam (`rust_seed_host_container_base`/`rust_named_type_base`/`rust_corpus_repr`) generalized to String/FreeMonoid → host `Vec`. **Emit-layer only — does NOT change `std/algebra.dag` definitions**, so it runs concurrently with Root B without conflict. Kills ~48% of the 16,071 self-host cargo errors (String+List). The carrier-independent tail (import-dedup E0252, closure-Debug) rides here too. **SUPERSEDED PREMISE (eager-deer-389):** `RustCorpusRepr` is DELETED -- there is no `HostNative`/`FaithfulFreeMonoid` seam left to generalize, and `rust_corpus_repr` no longer exists. Realization is keyed on each type's declaring module. This item is quarry per DESIGN section 3 (existing designs are re-read as quarry, never authority over the replacement): its OBSERVATION about FreeMonoid carriers may still hold, but its proposed MECHANISM is gone and cannot be built.
- **Root B — generic-inference keystone + de-fork (THIS lane).** Phased, in order:

1. **Keystone (partially landed via #5552):** simple generic-alias instantiation resolves — `type X = FreeMonoid<Symbol>` defines and construct/match/use-site works (proven by `generic_alias_coproduct_instantiation_test.dag`); `qualified_name.dag` is fully dissolved (incl. the v1 seed host-builtins made generic — `free_monoid_symbol_*`). **Still open:** raw variant-matching a value bound from a recursive `Cons.tail` field still fails *"variant not found in type FreeMonoid"* (`04_infer` fixpoint gap) — consumers avoid it via the `list_head`/algebra idiom; `catalog.dag` `fold_list`/`ParseTable` still await it. This residue is load-bearing resolver/infer work (`03_resolve`/`04_infer`) — model-before-implement, escalate before touching under any pre-dating brief.
2. **Definition unification:** make the coproduct the single authority in `std/algebra.dag`, delete the dag record-of-methods *definition* (re-express its surface as inhabitance), introduce the aliases. Only after the keystone lands (so the aliases compile). jolly-cat's Root A emit-seam should already be in by here — coordinate the `algebra.dag` touch.
3. **Repoints** (mechanical once 1–2 land): `nat`/`integer`/`float` (numeric tower `Int = GroupCompletion<Nat>`), `logic`, `effects` (operation-axis ⊕ idempotency-axis grounding), `verification` (entangled with #5428 + `Value::Null` — may stay last). Per the #5511 per-concept census above.
4. **Dissolve the 🟡 markers:** `qualified_name.dag` — **DONE** (`QnEmpty/QnCons` → `Empty/Cons`, `qualified_name_eq/for_all/singleton` → FreeMonoid ops + structural `==`, v1 seed bridge made generic); `catalog.dag` `fold_list` still awaits the recursive-tail resolver residue. The self_gen8 3 ignores fold into the def-unification PR — their premise flips from "stays_unemitted" to "expects grounded `List<T>=Vec<T>` emission" (`node://adhoc-9d2bb9c3-e7b`).

### Fences (DESIGN §3/§6, and the #5516 respawn-mandate lesson)

- **v1-coupled `coercion`/`node` renames stay DEFERRED to v1-delete** — category (c). Out of scope for this lane regardless of title.
- **The emit-seam (Root A) is jolly-cat's** — this lane does not touch `05_emit_rust.dag`.
- **Stage every file of a rename/collapse together** (the auto-committer atomicity hazard, §3 above).

### Execution update (#6341, 2026-07-07) — Root B narrowed to shadow-removal; three walls re-map the follow-ons

`#6341` landed the **core of Root B**: the `FreeMonoid`/`Empty`/`Cons` variant-drop shadow is removed — the coproduct now lives once in `dag/std/algebra.dag:111`, `v2.std.algebra` imports it, 158 importers repointed. But implementing the *fuller* Root B (move the ops; unify the definition; promote QualifiedName) surfaced **three walls by-execution** that re-scope the remaining lanes. Full trace + census: **[algebra-grounding-unification-design.md §9](algebra-grounding-unification-design.md)**. Summary for the roadmap:

- **A — reserved-name → builtin dissolution.** `dag/std/algebra.dag`'s templates *declare* the builtin collection-method names (`fold`/`map`/`filter`/`length`/`any`/`contains`/`skip`/…); a `std.*` free fn matching one breaks the whole method table (bare `fold(...)` fails corpus-wide). So the 8 ops `filter`/`length`/`any`/`contains`/`skip`/`is_empty` (+ `non_empty`/`bag_eq`) **cannot** move to `std.*` — they are free-fn duplicates of dag builtins and must *dissolve into* `.filter()`/`.length()` at v2 call sites (a lane, not a rename — a `list_filter` nickname is a §3 violation).
- **B — Root A is UNOWNED and confirmed load-bearing for the ops.** `src/v1/05_emit_rust.dag` hardcodes `crate::v2_std_algebra::{freemonoid_empty,fold_list,list_snoc_item}` for the `String=FreeMonoid<Char>` host bridge — so those 3 ops can't move until Root A relocates them. **jolly-cat (the assigned Root-A owner above) is no longer in the tree** (operator, 2026-07-06); the `String`/`List` alias grounding (§3b.2 above) is blocked until Root A is re-dispatched.
- **C — QualifiedName is `node`-gated, not algebra-gated.** `src/v2/std/qualified_name.dag` imports `v2.std.node`/`v2.std.diagnostic`/`v2.std.compilers.lexing`, all v2-only (`dag→v2` denied). FreeMonoid single-authority was **necessary but not sufficient**; promoting QualifiedName to `std.qualified_name` (unparking `type-env-single-authority-design.md` §5.1) needs the deferred **`node` defork** too (category (c), v1-delete).

Definition-unification (§3b.2: delete the dag record-of-methods *definition*) and the structure-record grounding (12 of 14 records diverge) remain deferred; the `flag-ANY` co-occurrence wall for the full `algebra` pair greens only after those. `#6341` removed the *dangerous* variant-drop shadow — the record shadow left is the benign record-with-record kind.

## Dissolution trigger (DESIGN §6)

Delete this doc when the fork census reaches zero — when no `std` basename denotes two concepts and every concept has a single authority. The census now has **three categories**, dissolving on different triggers: **(a) True mirrors** (`reducible`, `measure`) — mechanical collapse; dissolve into their carriers (absent files, re-pointed imports) as each PR lands. Plus `probe_selector` (the step-4 data-import unblock), independent. **(b) Grounding divergences** (`algebra`, `logic`, `nat`, `integer`, `float`, `effects`, `verification`) — downstream of the operator's grounding-unification design; tracked here as decision-input until that design lands and the repoints become mechanical. **(c) v1-artifact collisions** (`node`, `coercion`) — deferred to v1-shrink; trigger "when v1's consumers of `dag/std/{node,coercion}` are removed" (`node` self-dissolves, `coercion` shrinks to a v1-free rename). At the point all three are resolved the carriers tell the whole story and this audit is redundant.
