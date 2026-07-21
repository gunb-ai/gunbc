# v1 run-stability throughline — the memory axis (corpus-OOM lane)

> **Status:** executing, 2026-07-13, session sleek-deer-172 ("corpus OOM"). Same-day arc: proposed → amended per the routed review (sharp-bee-290: success metric corrected from floor peak — a governor-pinned design constant — to completion-within-step-budget at width > 1; M0 widened to test the retention→parallelism mechanism; M2 trigger reworded; eval-memo scope boundary added) → operator sign-off ("start executing on this now") → **M0 DONE, M1b + M1b-2 LANDED, M1 dial receipt CAPTURED (all receipts §1)** in #6528 (merged); **M1a LANDED (receipt §1)** in the follow-on PR — M1 is complete at the local envelope, M1c demoted (negligible mass), CI-slot confirmation declared-outstanding. The dial run also restored the falsifier cold sweep itself, which immediately caught three latent corpus reds (two pre-existing, fixed in #6530; one this PR's own orphan doc, fixed here) — the masking the lane exists to end, receipted in §1. Operator-requested alignment doc: the decision tree + milestone list for making whole-corpus floor runs complete reliably inside the 16 GiB slot envelope, for the months the self-host waves need. **This doc is a bridge plan, not a destination:** its terminal dissolution trigger is the witness corpus running as emitted native artifacts (ROADMAP §④ / Wave 3+), at which point the interpreted cost class this doc stabilizes stops being the live path.
>
> DESIGN refs: §1 (time is the value; safety = wrong answers cost later at interest), §2 (minimize redundancy — the located defect is duplicated ancestry materialization), §5 (fail-closed — no widening arms; the cap raises this doc forbids are the receipted widening arm), §6 (denominate in displaced cost; every scaffold lands with a dissolution trigger), §7 (the seed shrinks toward zero — nothing here invests in v1 beyond stabilizing the bridge).
>
> Single-authority note (§3): this doc **references** the lanes that already own adjacent work — resolver-graph-major (S2a Increment B), space-lens-minimal-project (unsigned), bounded-input-cost-envelope-scheduling (P2), cost-risk-benefit-floor-model (Stages 3–5) — it does not fork them. Where a milestone below is another lane's named deliverable, this doc's row points there.

---

## 0. The displaced cost (why this lane, in one line)

Whole-corpus floor runs currently pin at the 15 GiB `memory.high` reclaim clamp and die by step-cap at swap speed (runs 29183446733: 270 min; 29197126623: 60 min; four floor_peak_post reads pinned within 0.65 MiB of exactly 16,106,127,360 B, 2026-07-11 — `dag/gunbc/ci_floor_measurement.dag`). Every interpreter-touching PR, every main push with a wide diff, and every falsifier cold run pays this. The affected-set path is healthy (run 29220300831: 114/1,759 affected, 2.12 GiB peak, ci job 11m40s) — **the unstable regime is exactly the whole-corpus-affected run**.

**The failure axis, stated precisely (review correction, 2026-07-13):** the adaptive governor *packs width to the `memory.high` line by construction* (`ci_floor_measurement.dag`: "The run-time memory governor packs to this line by construction: it reads the tightest memory.high as its budget and backs off on the creep signals") — so the floor's cgroup peak is a **design constant** whenever queued work remains, not a defect signal. The defect is that per-worker retention is so large the governor is forced to `forced_serial=1` (one worker ≈ the whole budget, 474 ceiling back-offs), the corpus runs serially at swap speed, and the run **dies by step-cap — a time death caused by a memory shape**. Retention reduction therefore buys *width and wall-clock*, not a lower floor peak; the success metric throughout this doc is **completion within the step budget at width > 1**, with the **isolated probe peak** (single-process `measure_whole_tree_resolve`) as the retention-reduction measure. `floor_peak_post` staying pinned at ~15 GiB after a successful cut is expected, not failure.

## 1. Baseline, measured (2026-07-13, this tree)

| Fact | Value | Source |
|---|---|---|
| Corpus on disk (dag + src/v2) | **11.1 MB**, 1,999 files (~5.6 KB avg) | `find … -name '*.dag' \| wc -c` this tree |
| One strict whole-tree resolve (1,102 modules, 897 excluded) | **5.28 GB peak RSS** | `measure_whole_tree_resolve`, this tree, 2026-07-13 |
| — of which parse (all modules) | 684 MiB (linear, ~62× source) | same run, `[gantt] frontend.done` |
| — of which typecheck | **+4.35 GiB (87% of peak)** | same run, `[gantt] reconcile.done` |
| Floor whole-corpus peak | ≥15 GiB **censored** (memory.high clamp); 24 GiB cap-KILL @1,393 modules uncensored | `ci_floor_measurement.dag:26`; run 29123623007 |
| Per-module retention, same-grain pair | 2.2 MiB @228-module closure → ~11.5 MiB @~1,329 | #6487 cut-1 receipt (503→508 MiB); 15 GiB/1,329 |
| Growth law | per-module ∝ closure size ⇒ **superlinear total** (n^~1.4 between receipt points) | pairing above |
| Envs' share of module representation | **93.2%** (`full=5,723,421,964 B, retained=391,710,819 B`, 462 modules) | #5886 receipt (built, closed unmerged) |
| Interpreter reads envs at eval? | **No** — 1,109-witness floor GREEN with envs evicted; RED with items evicted (discriminating control) | #5892 squash (1f59c8be23) |
| Governor on whole-corpus runs | `forced_serial=1, hard_backoffs=474, budget_exceeded=1` | run 29215148169 receipt |

**Verdict the numbers force:** parsing is fat-but-linear; the instability is the typecheck-env web — per-module *materialized ancestry* whose size grows with the closure, retained for the life of the process in `typed_module_cache`. Not a leak; a cost-shape defect (§6 bare-minimum-cost class: "a proven quadratic fold is always fixed").

### M0 receipt (2026-07-13, `--ancestry-report`, 1,101 modules / 898 excluded, peak RSS 5,269,540,864 B)

The instrument landed and the numbers **re-order M1: the dominant mass is gun (b), not gun (a).**

| Field (all modules) | Retained entries | Spine dup_factor |
|---|---:|---:|
| `te.inductive_fields` **list mass** | **409,240,584** | 1.00 |
| all ten map fields combined | 1,068,601 | 1.00 |
| `tec.str_bindings` / `tec.deps_map` | 304,577 each | 1.00 |
| `te.ancestry_str_bindings` | 300,782 | 1.00 |

- The inductive-field **list mass is ~400× all map entries combined**: 409M `Rc<InductiveField>` slots ≈ **3.3 GB of Vec slots alone** — essentially the whole +4.35 GiB typecheck bloat. Worst modules: `tools.floor_effect_gate_witness` **22,610,058** entries, `tools.ci_gates` 20,135,582 (≈17k duplicates per type at ~1,338 types) — the `merge_inductive_fields` concat-on-collision compounding **multiplicatively along the import DAG** (each module re-concats its parents' already-duplicated lists).
- `dup_factor=1.00` on every map field at spine grain = zero cross-module sharing anywhere (every module a fresh spine), as diagnosed — but the map *entry* totals (~1M) are not where the bytes are.
- Dial-side pairing (from the governor receipts already recorded above): at this retention, `forced_serial=1`, `hard_backoffs=474`, wall 270 min → step-cap death.
- Consequence: **M1b (inductive-fields dedupe + base-sharing) is the first cut**; M1a (union base-choice) second; M1c (cycle sets: 23,945 entries — negligible) demoted to opportunistic.

### M1b receipt (2026-07-13, landed in this PR — sign-off provenance: operator "start executing on this now" 2026-07-13 + routed review "sign off M1 as a §6 bare-minimum-cost fix" with the amended done-bar)

The cut: `merge_inductive_fields` skip-if-equal + insert-not-concat on absent keys (`src/v1/04_env.dag`), parents fold init from the first parent's map instead of `empty_map()` (`src/v1/04_infer.dag`, both `build_type_env` paths), seed regenerated. Receipts, all by execution on this tree:

| Receipt | Pre-M1b | Post-M1b |
|---|---|---|
| Whole-tree strict resolve peak RSS (1,101 modules) | 5,274,587,136 B (5.27 GB) | **2,045,423,616 B (2.05 GB, −61%)** |
| `te.inductive_fields` list mass | 409,240,584 | **95,050,553 (−77%)** |
| Worst module (`floor_effect_gate_witness`) | 22,610,058 | 4,091,670 |
| `corpus_fingerprint` (controlled pair, same tree + binary shape) | `d34a91a4d8fe8845` | **`d34a91a4d8fe8845` — byte-identical** |
| `emit_graph_fingerprint` | `cfdc338c2795a035` | `cfdc338c2795a035` — identical |
| v1-compiler lib battery (incl. module-grain + import-closure equivalence modules) | — | **212 passed / 0 failed** |
| `claim_executor` memo purity oracles | — | 3/3 |
| `regen_stage0 --verify` | — | `regen_divergence_count=0`, matches fresh self-compile |

(The earlier `82e9c68b96617067` baseline was a different binary/tree state — 1,102 vs 1,101 modules — and is superseded by the controlled pair above.)

- Residual inductive mass (95M) = genuinely-differing-list concats — multiplicities already baked into different parents' lists before the fix; full set-semantics dedupe is a **separate decision** (changes list multiplicities consumers could observe; staged as an M1b follow-on question, not smuggled into this cut). → **Taken as M1b-2, receipt below.**

### M1b-2 receipt (2026-07-13, set-semantics merge — filtered append, identity = variant|field|shape|element)

Consumer analysis first: `inductive_fields_for` readers use filter/first per (variant, field) — first-match, multiplicity-insensitive; `has_inductive_field` is any-match. Dedupe keeps the first occurrence of each fact, so every consumer sees the same winner. Then by execution:

| Receipt | Pre-M1b | M1b | **M1b-2** |
|---|---:|---:|---:|
| `te.inductive_fields` list mass | 409,240,584 | 95,050,553 | **82,037 (−99.98%)** |
| Worst module list mass | 22,610,058 | 4,091,670 | **334** |
| Whole-tree resolve peak RSS | 5.27 GB | 2.05 GB | **1.48 GB (−72% total)** |
| `corpus_fingerprint` | `d34a91a4d8fe8845` | identical | **identical** |
| `emit_graph_fingerprint` | `cfdc338c2795a035` | identical | **identical** |

Every list is duplicate-free by induction under the new merge (locals are duplicate-free; merges never append a present fact), so the mass cannot re-accumulate — the multiplicative class is **unwritable**, not just reduced (§5 construction, not validation).
- Pre-fix equivalence baseline for the M1a oracle: `corpus_fingerprint=d34a91a4d8fe8845` at the M1b-landed state.

### M1 dial receipt (2026-07-13, whole-corpus falsifier cold run — `gunbc_falsifier_batches`, local)

Run shape: predict-only cold sweep, **all 1,757 discovery witnesses across 537 entries, 0 skipped**, M1b-2 seed binaries, this tree + the #6530 red-fixes. Honesty note: local cgroup budget is **33.6 GB vs CI's 16 GB slot**, so the width the governor packs is not CI's width — but the observed peak fits even CI's envelope with headroom, and the dial variables the done-bar names are envelope-independent signs of health.

| Dial | Pre-M1b (CI whole-corpus runs, §1 baseline) | **M1b-2 (this run)** |
|---|---:|---:|
| outcome | step-cap TIME death at swap speed (270 m / 60 m runs; nightly killed at 35.8 GB censored) | **EXIT=0 green, ~8 m wall** (resolve 406 s + eval 36 s) |
| governor width | `forced_serial=1` (one worker ≈ whole budget) | **max_width_reached=9, forced_serial=0** |
| hard_backoffs / budget_exceeded | 474 / 1 | **0 / 0** |
| peak | ≥15 GiB censored (packed to the clamp) | **8.83 GB VmHWM; governor peak_current 11.35 GB** — inside even a 16 GB slot |

**M1 done-bar: met at the local envelope** — completion within step budget at width > 1, zero governor distress. The CI-slot confirmation (16 GB `memory.high`, where the governor will pack to a narrower width by construction) lands with the first post-merge whole-corpus CI run; declared, not asserted.

**What restoring the cold sweep immediately paid for (the lane's premise, receipted):** the falsifier had been memory-dead since before 2026-07-10, and in that shadow three corpus reds accumulated undetected — (1) #6520's fold-API dissolution left 3 manual body-lowering claim modules unresolvable (typed refusal at frontier population; falsifier-blocking); (2) #6506 added `git.Core.Toplevel` to the git mock corpus without extending the totality consumer (witness red); (3) this PR's own throughline doc was a doc-graph orphan (2 witness reds) until the DESIGN.md open-threads link in this commit. (1)+(2) proven pre-existing on pure main by execution with the merge-base binary and fixed in **#6530**; (3) caught pre-merge by the very cold run M1 revived. CI's affected-set selection ran none of these consumers for the diffs that broke them — missing-selection-edge follow-ups flagged on #6530.

### M1a receipt (2026-07-13, union base-choice — sized dispatch, landed in this PR)

Mechanism (authority: `src/v1/04_env.dag`, note row `union_base_choice_note`; realization regenerated into `v1_compiler_infer_env.rs`): each of the four guarded unions (`str_bindings`, `deps_map`, `variant_locals`, `bool_set`) is now a **sized dispatcher** — `count` on both maps is O(1) (`im_rc` len) — that walks the **smaller** map into the **larger** as the shared spine, with an orientation-mirrored `*_into_overlay` body whose winner/conflict semantics are proven identical to the original `*_into_acc` direction (same-authority keeps state, differing keeps base + emits the conflict row with `existing_site`/`incoming_site` in their semantic roles, absent inserts). This removes the "first parent is a small leaf ⇒ path-copy the whole closure surface" term of defect §2.1 **by construction** — no threshold, no heuristic; the fold init in `union_parent_type_env_caches` (`04_infer.dag`) is untouched because the per-step dispatch already handles a small first parent. Bundled from the #6528 review commitment: `append_inductive_field_absent` (note row `put_inductive_field_guard_note`) makes the M1b-2 set-semantics base case constructed-not-observed on both `put_inductive_field` and `put_inductive_field_cross`.

Controlled pair (isolated whole-corpus resolve probe, same tree, same EX list, only the binary differs):

| | PRE (main binary) | **POST (M1a binary)** |
|---|---:|---:|
| probe peak (VmHWM) | 1,481,617,408 B | **1,401,417,728 B (−5.4%)** |
| `corpus_fingerprint` | d34a91a4d8fe8845 | **d34a91a4d8fe8845 (byte-identical)** |
| `emit_graph_fingerprint` | cfdc338c2795a035 | **cfdc338c2795a035 (byte-identical)** |
| modules resolved / excluded | 1,101 / 903 | 1,101 / 903 |

Note the probe's per-module **entry counts are invariant by construction** — the probe counts semantic entries, and M1a changes only which spine is shared (bytes), so the fingerprint identity is the equivalence oracle and VmHWM the retention oracle. The modest −5.4% is consistent with M0's mass ranking (inductive fields were the dominant term, already taken by M1b/M1b-2; the union base was second). Battery green: `cargo test --release -p v1-compiler --lib` **214 passed / 0 failed**, `regen_stage0 --verify` **regen_divergence_count=0**, memo oracles 3/3.

Probe honesty note: the M0 receipt's 22-subpath scaffold EX list was never baked into an authority — `ci_layer_roots.dag`'s `whole_tree_strict_resolve_exclusion_substrings` carries only 4 of them — so this pair re-derived the list by execution (2 iterations to converge; saved with the probe scripts). **RESOLVED (operator ruling 2026-07-13, "just make it single authority"):** the 22 subpaths are baked into `whole_tree_strict_resolve_exclusion_substrings` with a scaffold-class note row (`whole_tree_strict_resolve_scaffold_exclusion_note`); the probe now runs with zero `--exclude-subpath` args. Equivalence proven by execution with a genuine control (post-merge main tree): pre-bake authority + 22 args vs baked authority + no args → both **1,102 resolved / 904 excluded, `corpus_fingerprint=ef92c94a5f0bf16d`, `emit_graph_fingerprint=8ae181241349ddca`** — byte-identical. Sweep follow-on (2026-07-13): the 24 files behind the baked rows were audited file-by-file with execution receipts; 7 genuinely dead scaffolds deleted with their rows (EX list 27→20), and the note's cause corrected — the remaining rows' strict-resolve reds are universe-relative artifacts of the list's own blanket `/test/` rows (import targets exist in-tree), not tree rot. Post-sweep receipts: probe 1,105/903 green at 1.40 GB; whole-corpus falsifier 1,780 witnesses PASS at width 9, `forced_serial=0`.

### Track A denomination receipt (2026-07-13, resolve-split instrument #6535 — whole-corpus falsifier, EXIT=0)

The per-entry resolve cost was one lump (`resolve_nanos`); #6535 attributes it through a worker-thread-local slot (exact at any governor width, unlike the last-writer-wins `phase_profile`). Whole-corpus run, 1,776 witnesses, width 9, zero governor distress, resolve lump 371.1s, **residue 0.4s (99.9% attributed)**:

| Stage | Time | Share | Routing |
|---|---:|---:|---|
| `typecheck_compute` (genuine cold computes) | 194.7s | 52% | cross-worker share prize — **design:** [cross-worker-typecheck-share-design](cross-worker-typecheck-share-design.md) (S2a increment C). Inc B landed (#6543); post-merge resolve-split 2026-07-14: typecheck 201.6s — **neutral, prize still OPEN** (Inc B is the denomination rung; increment C owns the displaced cost) |
| `reconcile_assembly` (residue rerun per entry even at 100% cache hits) | 107.7s | 29% | resolver-graph-major lane |
| `normalize` (pure per-module diags) | 28.5s | 8% | this lane — within-worker memo (next cut) |
| `ownership` (per-typed-graph proofs) | 24.3s | 7% | this lane — within-worker memo (next cut) |
| `resolve` (`resolve_modules`) | 4.7s | 1.3% | none — the pre-receipt candidate cut, disproven |
| `parse` / `load` / `parent_envs` / other | ~11s | ~3% | none |

Lesson the receipt bought: the intuitive cut (memoize `resolve_module_imports`) was worth 1.3% — the instrument re-priced the plan before any code was written on the wrong target. The two dominant terms route to their owning lanes with hard numbers; this lane's remaining cheap win is normalize+ownership (~53s).

## 2. The defect, located (verified against the live tree, 2026-07-13)

Cleared: `func_env` is a proper scope-chain post-#5893 (local sigs, shared parent chain, O(n+edges)) — not the problem.

1. **Union base-choice** — `union_parent_type_env_caches` (`v1_compiler_infer.rs:11882–11935`) Rc-shares the accumulator spine with the **first** import's cache only; `guarded_union_str_bindings` (`v1_compiler_infer_env.rs:207–275`) then freshly path-copies every key of parents 2..k's full flattened closure surfaces. First imports are typically small leaves; modules average ~6 imports ⇒ nearly every module materializes ≈ its whole closure's name surface, fresh, across four maps. #6487 cut 2 (skip-if-equal) prevents re-inserting *equal* keys; it does not fix the base.
2. **`merge_inductive_fields` folds from empty** (`v1_compiler_infer.rs:12742–12751` → `infer_env.rs:593–616`) — re-inserts the first parent's entire closure-merged map into a fresh HAMT per module (zero spine sharing; the #6487 treatment was never applied here), and concat-on-collision duplicates field lists once per import path (diamond-multiplicative).
3. **Fresh per-module cycle/recursive sets** over closure-wide names (`v1_compiler_infer.rs:12708–12736`) — same shape, smaller mass.

Secondary (linear, cheap, optional): `NewlineIndex.char_codes` = 8 B per source character (~110 MB corpus-wide); per-span cloned file-path `String`s.

## 3. Decision tree / milestones

```
M0 denominate (≤1 day) ──► M1 construction fixes (short-term; "step 1")
                                  │ probe + floor (width, wall, outcome) receipts
                                  ▼
        whole-corpus floor completes within step budget at width > 1?
                        │ yes                    │ no
                        ▼                        ▼
                DONE (bridge holds;      M2 retention strip at cache grain
                watch ratchet, §5)       (#5886 projection; medium-term)
                                                  │ receipts
                                                  ▼
                             still serial / still step-capped? ► escalate:
                                     M3 pull-forward (S2a Inc-B interface
                                     summary) vs. envelope decision
```

- **M0 — re-denominate the prize (immediate, measurement only).** Two halves, so the plan *tests* its own verdict rather than confirming it: **(a) retention side** — re-run `measure_whole_tree_resolve` on today's corpus with a Σ per-module |ancestry-map| entry-count report, retained-vs-distinct-spine (the definitive quadratic-witness the deleted `cache_walk` never measured — it counted payload-Rc sharing, which is healthy; the byte carrier is the map spines); **(b) dial side** — pair the retention level with the floor's observed `(target_width, hard_backoffs, forced_serial, wall, outcome)` from the governor receipts (already emitted per run), so "retention throttles parallelism" is the *measured* mechanism M1 is priced against, not an assumption. Done-bar: both halves on the carrier, before any cut. *The M0 instrument is measurement-only (flag-gated `--ancestry-report`; default probe flow unchanged) — the behavioral cuts are M1's, gated and receipted separately.* **Status: DONE (receipt in §1).**
- **M1 — construction fixes for §2's three sites (short-term; the "step 1" previously proposed; sign-off basis: §6 bare-minimum-cost — a proven cost-shape defect is always fixed, so M1 is justified regardless of the floor outcome).** (a) accumulate the union from the largest-surface parent (or true k-way spine-sharing); (b) give `merge_inductive_fields` the same skip-if-equal + base-sharing treatment `str_bindings` got in #6487 cut 2, and de-duplicate the collision concat; (c) same for the cycle/recursive sets. Correctness oracles already in tree: every-order equivalence suite, byte-identical assembled view, `regen --verify`, once-per-node receipt (`union_resolve_typechecks_each_node_once`). Done-bar, two independent axes: **retention** — isolated probe peak materially down (per-module constant flat across closure scales — the superlinear term gone; probe peak ≤ ~2–3 GB); **throughput** — one whole-corpus floor run completing **within the step budget at governor width > 1** (`forced_serial=0`, `hard_backoffs` ≈ 0, wall materially down). `floor_peak_post` is explicitly *not* a done-bar: the governor re-packs freed headroom with more shards, so the cgroup peak stays at the line on success (§0). Blast radius: `src/v1/04_infer.dag` + `04_env.dag` (+ regenerated seed — the `.rs` files are generated; the authority edit is the `.dag`). These are load-bearing files — gated on operator sign-off of this doc. **Status: sign-off given 2026-07-13 ("start executing on this now"; routed review concurs on the §6 basis). M0's receipt re-ordered the cuts by mass: M1b LANDED (receipt §1, retention axis green, probe 5.27→2.05 GB); M1a LANDED (receipt §1, sized union dispatch, probe 1.48→1.40 GB, fingerprints byte-identical); M1c demoted (cycle sets 24k entries, negligible mass). Throughput half of the done-bar outstanding — declared in the M1b receipt, lands with the first whole-corpus floor run.**
- **M2 — retention strip at the cache grain (medium-term; the honest "eviction"), decision-gated on M1's receipts (trigger: M1's width/wall-clock gain is insufficient to complete the whole-corpus run within the step budget — i.e., per-shard retention still forces too few shards; NOT "floor peak still at 15 GiB", which the governor pins by design).** The #5886 projection applied per-row: once every importer of a module in the *roster's remaining demand* is typechecked (roster-aware predicate from `module_graph_facts` + worker queue; fail-closed: uncertain ⇒ retain), strip `type_env`/`func_env` to the shared empty singletons, keeping `module`/`items`/`item_registry`. Claim-path-scoped (the emit path genuinely reads envs — one shape, N retention policies; the parameter is retention, never meaning). Receipt = the #5886 teeth method rerun: full floor + emit-host smokes GREEN with envs stripped, items-strip control RED. Explicitly **not** whole-row eviction (Rc-pinned payloads make that free almost nothing) and **not** an LRU (recompute reds the once-per-node contract). Note the A3 shelving verdict's own flip-trigger (~7.8 GiB co-resident) is tripped ~2× over — this milestone is the trigger's pre-authorized action, corrected to the grain that actually frees bytes.
- **M3 — durable capture (existing lane, referenced not forked).** Interface-summary retention (drop typed bodies, keep the exported surface) is resolver-graph-major **S2a Increment B's named memory prize** (#6487 landed its structural prerequisite). If M1+M2 leave the floor pinned, the decision escalates to the operator: pull Inc-B forward vs. envelope change. This doc does not schedule another lane's work.
- **A (cross-entry typed-module memo) — the content-keyed, host-budget-bounded instance of this term-3 axis.** [cross-entry-typed-module-memo-sketch](cross-entry-typed-module-memo-sketch.md) is the operator-signed design sketch (sleek-ram-450, 2026-07-15) that reads M2/M3 as one law at two grains: env-strip (M2) is the zero-recompute inner ring, `SpacePacked` whole-entry eviction is the host-budget outer ring, and — per the store-path read — the retained payload must be the **interface summary (M3/Inc-B shape)** for eviction to free the Node-dominated mass. It couples A to Inc-B; the coupling decision is on the operator's desk (sketch §6).

**Cross-cutting rows (owned elsewhere, tracked here for the stability picture):**
- *Whole-tree compile-clean time (2026-07-21):* execution-proven pin that compile-clean itself is **~3 min / ~6 GiB** on fleet slots (fits 16 GiB, swap=0); the 60+ min operator pain is **same-process batch-2 accumulation** pinning `memory.high` at t≈9 min. The **+3.1 GiB compile-clean reconcile step** (1.35 → 4.2 GiB) is **#6848 whole-pool `pool_parse`**, not residual M1 env duplication; #6956 heads-only left footprint flat. Thrash mechanism + bisect receipts: [compile-clean whole-tree time diagnosis](compile-clean-whole-tree-time-diagnosis.md).
- *Compute axis / CI wall:* the compute-side complement of this memory lane — measured kernel decomposition (`infer_expr` 81.5% of the fold, resolve 12.5%), cross-box environment **parity** (srv1/srv2/srv3/container within ~3%; the environmental theories are refuted in its §3.4 claims register), the one-day 346s→2,413s tree-growth finding, and the W2–W4 endgame (kernel root fix → cross-run store → sharding) — lives in [compile-wall-endgame](compile-wall-endgame.md) (clever-seal-476, 2026-07-17).
- *Falsifier fit:* the nightly cold run (~14.9 GiB envelope) doesn't fit the 16 GiB slot — M1/M2 are its fix too; until then it stays the declared two-sided deficit (`falsifier_workflow.dag:25`).
- *Governor:* stays as the reactive safety net; demotes when graph-derived `CostAccount.space` lands (its own dissolve-on). The **space-lens fold** (space-lens-minimal-project.md, awaiting sign-off) is the model-level answer to "know the cost before running" — separate decision, unblocked by nothing in this lane; M0's calibration data feeds it.
- *Time axis:* the 5 s fast-lane budget + affected-set selection already guard the eval-time side; no work here.
- *Eval-memo (#6441/#6469) — deliberately out of scope, and why:* `eval_call_memo_frame_exit` drains the memo per witness in the floor loop (`claim_batch.rs:458/638`, `claim_executor.rs:623/691`), so it does **not** accumulate co-resident across the whole-corpus floor; its OOM class is the single giant witness (s1_closure), owned by the long/ lane. This doc and that lane must not both claim to have "fixed the corpus OOM" — the corpus-resident axis is this doc's; the single-witness axis is theirs.

## 4. Non-goals / guardrails (§5)

- **No envelope raises.** Raising `memory.high`/`MemoryMax` is the receipted widening arm ("buys its last GiB at swap speed", `ci_budget_tree.dag:133`). The budget stays; the footprint moves.
- **No absorbing fallbacks:** M2's predicate refuses (retain + typed counted diagnostic) when uncertain; a post-strip demand is a counted refusal, never a silent recompute.
- **No v1 investment beyond the bridge:** no bytecode VM, no JIT, no new interpreted-workload features (ROADMAP §④ review-reject stands). Every milestone carries the same terminal dissolution trigger: the Wave 3+ native-artifact cutover.
- **No re-litigating the doomed-seed ruling:** M1 edits are the same class as landed cuts (#6155, #6487); M2 is the tripped trigger's own pre-authorized action; M3 belongs to its existing lane.

## 5. Receipts to land with each milestone

| Milestone | Discriminating receipt |
|---|---|
| M0 | (a) probe line: modules, peak RSS, Σ ancestry-map entries retained-vs-distinct; (b) paired floor dial receipt: `(target_width, hard_backoffs, forced_serial, wall, outcome)` — both on the measurement carrier |
| M1 | retention: probe before/after, per-module constant flat across 228-vs-full closure scales; throughput: one whole-corpus floor run completing within step budget at width > 1 (`forced_serial=0`); every-order + assembled-view oracles green; `regen --verify` green. (`floor_peak_post` expected to stay ≈ the line — not a metric) |
| M2 | teeth pair (envs-strip GREEN / items-strip RED); whole-corpus completion within step budget at width ≥ the M1 receipt's width; counted post-strip-demand refusals = 0 on a green run |
| exit | two consecutive whole-corpus main runs + one falsifier cold run completing within their step budgets at width > 1, zero step-cap kills, zero swap-speed tails (sub-MiB `memory.high` overshoot signature absent) |

## 6. Open decisions for the operator

1. **Sign-off on this throughline** (gates M1 — load-bearing `infer` files; basis = §6 bare-minimum-cost, so M1 stands on the located defect regardless of the floor outcome).
2. **M2 go/no-go criterion:** proposed = "M1's width/wall-clock gain is insufficient to complete the whole-corpus run within the step budget" (per-shard retention still forces too few shards); alternative = schedule M2 regardless (defense in depth). (The earlier "floor still censored at 15 GiB" wording was wrong — the governor pins the peak at the line whether or not M1 succeeds.)
3. **Space-lens fold sign-off** (separate lane; unblocks the governor's dissolution; M0 feeds its calibration).
