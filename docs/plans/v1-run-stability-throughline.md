# v1 run-stability throughline — the memory axis (corpus-OOM lane)

> **Status:** executing, 2026-07-13, session sleek-deer-172 ("corpus OOM"). Same-day arc: proposed → amended per the routed review (sharp-bee-290: success metric corrected from floor peak — a governor-pinned design constant — to completion-within-step-budget at width > 1; M0 widened to test the retention→parallelism mechanism; M2 trigger reworded; eval-memo scope boundary added) → operator sign-off ("start executing on this now") → **M0 DONE (receipt §1) and M1b LANDED (receipt §1)**, both in this PR; M1a next; M1's floor-dial (throughput) receipt outstanding and declared so. Operator-requested alignment doc: the decision tree + milestone list for making whole-corpus floor runs complete reliably inside the 16 GiB slot envelope, for the months the self-host waves need. **This doc is a bridge plan, not a destination:** its terminal dissolution trigger is the witness corpus running as emitted native artifacts (ROADMAP §④ / Wave 3+), at which point the interpreted cost class this doc stabilizes stops being the live path.
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

- Residual inductive mass (95M) = genuinely-differing-list concats — multiplicities already baked into different parents' lists before the fix; full set-semantics dedupe is a **separate decision** (changes list multiplicities consumers could observe; staged as an M1b follow-on question, not smuggled into this cut).
- **Outstanding half of the M1 done-bar (declared, not asserted): the floor dial receipt** — one whole-corpus floor run completing within step budget at governor width > 1. Lands from the first whole-corpus run on this tree (local attempt or the post-merge CI run); until then M1 is *retention-receipted, throughput-pending*.
- Pre-fix equivalence baseline for the M1a oracle: `corpus_fingerprint=d34a91a4d8fe8845` at the M1b-landed state.

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
- **M1 — construction fixes for §2's three sites (short-term; the "step 1" previously proposed; sign-off basis: §6 bare-minimum-cost — a proven cost-shape defect is always fixed, so M1 is justified regardless of the floor outcome).** (a) accumulate the union from the largest-surface parent (or true k-way spine-sharing); (b) give `merge_inductive_fields` the same skip-if-equal + base-sharing treatment `str_bindings` got in #6487 cut 2, and de-duplicate the collision concat; (c) same for the cycle/recursive sets. Correctness oracles already in tree: every-order equivalence suite, byte-identical assembled view, `regen --verify`, once-per-node receipt (`union_resolve_typechecks_each_node_once`). Done-bar, two independent axes: **retention** — isolated probe peak materially down (per-module constant flat across closure scales — the superlinear term gone; probe peak ≤ ~2–3 GB); **throughput** — one whole-corpus floor run completing **within the step budget at governor width > 1** (`forced_serial=0`, `hard_backoffs` ≈ 0, wall materially down). `floor_peak_post` is explicitly *not* a done-bar: the governor re-packs freed headroom with more shards, so the cgroup peak stays at the line on success (§0). Blast radius: `src/v1/04_infer.dag` + `04_env.dag` (+ regenerated seed — the `.rs` files are generated; the authority edit is the `.dag`). These are load-bearing files — gated on operator sign-off of this doc. **Status: sign-off given 2026-07-13 ("start executing on this now"; routed review concurs on the §6 basis). M0's receipt re-ordered the cuts by mass: M1b LANDED (receipt §1, retention axis green, probe 5.27→2.05 GB); M1a next; M1c demoted (cycle sets 24k entries, negligible mass). Throughput half of the done-bar outstanding — declared in the M1b receipt, lands with the first whole-corpus floor run.**
- **M2 — retention strip at the cache grain (medium-term; the honest "eviction"), decision-gated on M1's receipts (trigger: M1's width/wall-clock gain is insufficient to complete the whole-corpus run within the step budget — i.e., per-shard retention still forces too few shards; NOT "floor peak still at 15 GiB", which the governor pins by design).** The #5886 projection applied per-row: once every importer of a module in the *roster's remaining demand* is typechecked (roster-aware predicate from `module_graph_facts` + worker queue; fail-closed: uncertain ⇒ retain), strip `type_env`/`func_env` to the shared empty singletons, keeping `module`/`items`/`item_registry`. Claim-path-scoped (the emit path genuinely reads envs — one shape, N retention policies; the parameter is retention, never meaning). Receipt = the #5886 teeth method rerun: full floor + emit-host smokes GREEN with envs stripped, items-strip control RED. Explicitly **not** whole-row eviction (Rc-pinned payloads make that free almost nothing) and **not** an LRU (recompute reds the once-per-node contract). Note the A3 shelving verdict's own flip-trigger (~7.8 GiB co-resident) is tripped ~2× over — this milestone is the trigger's pre-authorized action, corrected to the grain that actually frees bytes.
- **M3 — durable capture (existing lane, referenced not forked).** Interface-summary retention (drop typed bodies, keep the exported surface) is resolver-graph-major **S2a Increment B's named memory prize** (#6487 landed its structural prerequisite). If M1+M2 leave the floor pinned, the decision escalates to the operator: pull Inc-B forward vs. envelope change. This doc does not schedule another lane's work.

**Cross-cutting rows (owned elsewhere, tracked here for the stability picture):**
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
