# Five-minute CI gate — program scoping

Scoping only. No implementation lands from this note alone. Registers the
operator-signed product boundary, six sub-lanes, and the **dependency sequence**
in `gunbc.roadmap_authority` so dispatch, witnesses, and receipts share one
program identity.

**Product boundary (re-anchored by operator ruling 2026-08-02):** every required
operation performs the bare minimum computation needed to serve it — an ordinary
source edit's verdict recomputes no semantic fact whose inputs did not change,
and its cost is the touched closure's delta, not the corpus. **Five minutes is
not the contract** — the operator's words: the earlier sentence presenting it as
the contract was a mistake; five minutes is the distress ceiling ("holy crap
this is hard / not working well"), retained only as the Stage C checkpoint
bound. Reaching 5:00 does not complete the program while redundant computation
remains on a required path; the bar is scoped by the realization/materialization
goals, and wall-clock falls out of them.

**This is a program, not a PR.** Each sub-lane is independently dispatchable;
the parent carries the end-to-end contract and the ordered sequence below.
Receipts stay on the sub-lane that owns them — the parent carries no stored
rung or wall-clock field.

**Native witness execution — settled at small scale (#7599 MERGED; #7671 enrollment MERGED).**
The program no longer debates whether moving semantic execution out of the
interpreter works at small scale. **Banked:** `native_at_small_scale_transition_receipt`.
**Open (operational):** fleet native rate on ordinary floors (closeout receipt:
known fallback class; recoverable logs omit `native_count`). Do not credit Stage B
wall from enrollment alone. Next mechanism work is shared entry-view construction
(operator 4-PR sequence), not another broad interpreted-path optimization.

**Interpreter endgame (linked, not duplicated):** moving `cli_run`/interpreter
helpers to `.dag` improves authority but does **not** make CI fast while the
`.dag` is still interpreted. The win arrives when `.dag` is the source
authority for a native, reusable realization. See [witness realization plan](witness-realization-plan.md)
and `dag/gunbc/plans/cli_run_hollowing_plan.dag`.

---

## Operator sequence (dependency graph)

Registered as roadmap edges where noted; order is binding for dispatch.

| step | work | disposition |
|---|---|---|
| 1 | Finish **#7522** (`warm-merge-admission`) with real net wall receipt and stage profile | **MERGED** — `admission_settle=CaptureMeasuredNetOpen` (`gunbc.ci_cost_arc_closeout_receipt`) |
| 2 | Merge **#7599** once controls and reviews clear | **MERGED** — typed receipt `native_at_small_scale_transition_receipt` |
| 3 | **Immediately** cut over a bounded production witness population to the native execution kind | **MERGED enrollment (#7671)** — `native_fleet_settle=EnrollmentLandedFleetRateOpen`. Not a completed native wall-clock speedup |
| 4 | Continue **#7534** as shared materialization substrate — no default CI savings credited until ordinary invocations consume it | **MERGED opt-in** — `warm_hit_settle=PartiallyDeliveredSkipProofOpen` |
| 5 | Union program **closed** per #7533; assembly observation remains a compiler candidate only | **CLOSED** / `compiler_next_measured_target=PerEntryAssembly` (not end-to-end dominant) |
| 6 | This program (`five-minute-ci-gate`) makes the bare-minimum-computation contract and this graph explicit; five minutes stays the distress checkpoint (Stage C bound), never the completion criterion | this registration · `gunbc.ci_cost_arc_closeout_receipt` |

**Terminal sequence (operator 2026-08-04):** #7760 banked the typed affected-CI audit and exposed `unattributed_ci_wall`. **PR2 denominator reconciliation is now measured on that same exact subject:** `ci_job_wall_population_receipts` partitions 100% of the 2,509,000 ms CI-job wall into adjacent, non-overlapping elapsed populations; `unattributed_ci_wall=0`. The original capture/resolve/eval harvest remains typed as the initial partial observation rather than being silently rewritten. **PR3** attacks the largest reconciled preparation term only after exclusive attribution identifies the duplicated construction. **PR4** streams that term or commits to module-grain persistent materialization. Entry-view / width-2 / broad native / retention / union remain frozen by their independent negative receipts, not by an unanswered denominator.

**Materialization hierarchy** (step 4 feeds step 3, not a parallel fork):

module parse / type / interface materializations → resolved entry materializations
→ selected native witness bundle.

Roadmap edges encoding sequence: `native-selected-witness-bundle` →
`warm-merge-admission`; `pre-index-materialization-lookup` and
`per-entry-assembly-decomposition` → `module-grain-materialization` (existing).

---

## Sub-lanes (six)

| id | role |
|---|---|
| `warm-merge-admission` | Stamp and admit merges from warm receipts — fold resolve/materialization receipt gates into merge admission. **#7522 MERGED**; net stamp+gate join still OPEN (receipt). |
| `native-selected-witness-bundle` | **#7599 MERGED** (substrate). **#7671 MERGED** (bounded production enrollment). Fleet native rate OPEN — do not credit Stage B wall from enrollment alone. |
| `module-grain-materialization` | Shared materialization substrate (#7534 MERGED, opt-in). Hierarchy above; no default CI savings until ordinary invocations consume hits. Warm-hit skip proof OPEN after #7728. |
| `pre-index-materialization-lookup` | Warm manifest lookup before corpus-scale index construction — serve/dashboard path. |
| `phased-single-process-ci` | Regen, floor, and admission share one initialized substrate with separate verdict stamps. |
| `per-entry-assembly-decomposition` | Narrow measured lane after #7533 closed union — assembly/reconcile attribution only. |

---

## Dispositions beside the program (not sub-lanes)

| subject | disposition |
|---|---|
| `entry-graph-union-construction` | **CLOSED** — #7533: `decision_ratio = 0` repeated typecheck misses. Redirect to `per-entry-assembly-decomposition`. Slice 1 banked as #7483. |
| retention / floor prep-tax (D1) | **P1 REJECT banked (#7725)** — schedule-retention eviction is not the ~2s/entry tax; redirect = assembly / materialization reuse. M2 width-1 memory thesis still stands (#7581). Width-2 / broad native HOLD until shared preparation. |
| exact-tree materialization (#7534) | Substrate under `module-grain-materialization`; opt-in; no default CI activation credited; `warm_hit_settle=PartiallyDeliveredSkipProofOpen`. |
| end-to-end residual | **`end_to_end_residual=ReconciledCiJobWall`** — exact affected subject partitioned into 15 adjacent population intervals; `unattributed_ci_wall=0`. |
| compiler next measured target | **`compiler_next_measured_target=PerEntryAssembly`** — next measured compiler candidate inside `discovery_resolve_wall`; not proven to dominate end-to-end wall. |

---

## Staged expectations (not flat wall-clock claims)

**Stage A — after current safety/cost board lands:** modest direct reduction only
(principally net admission saving from #7522); floor witness bodies still
interpreted.

**Stage B — after first production native-bundle cutover:** first plausible
order-of-magnitude witness-execution reduction (gain depends on native-emittable
share of the selected set + bundle construction cost).

**Stage C — five-minute affected-PR gate defensible when ALL hold:**

- most selected witnesses execute natively;
- native artifacts hit across ordinary invocations;
- unchanged modules do not reparse or re-typecheck;
- cache lookup happens before corpus-scale index construction;
- regen / floor / admission share one initialized substrate where safe;
- interpreted fallback is counted and steadily approaches zero.

Product budgets (Stage C targets): normal affected PR warm p50 ≤ **5 min**;
broad/cold p95 ≤ **10–15 min**; falsifier cadence ≤ **20–30 min** initially.

---

## Verified receipts (execution-backed)

Only figures verified by execution in this session or committed on main are cited
at full altitude. Claimed-not-verified figures are typed separately.

| receipt | source | what stands |
|---|---|---|
| slice 1 partition | #7483 · [entry-graph-union slice 1](entry-graph-union-slice1-measurement.md) | exclusive partition; `load` eliminated as union target |
| slice 2 zero-repeat | #7533 MERGED | `decision_ratio = 0` across three disjoint 50-entry windows |
| M2 retention width-1 | #7581 PR body | peak 6.27 GiB, `schedule_evictions=2094`, 842 entries PASS |
| P1 retention-vs-drain | #7725 MERGED · [p1 receipt](p1-retention-vs-drain-cohort-receipt.md) | **REJECT** — eviction ≠ tax |
| CI-cost arc closeout harvest | run [30863019228](https://github.com/gunb-ai/gunbc/actions/runs/30863019228) · `gunbc.ci_cost_arc_closeout_receipt` | HandHarvestedDatedAudit; pipeline_sum 3256000 ms; ci_job 2509000 ms; initial nested capture/resolve/eval observation 606958 ms; elapsed population reconciliation **2509000 ms classified / 0 ms unattributed**; selection 297/916; peaks typed as ByteSize |
| native bundle witness suite | **executed** at #7599 head `2f9e780e6` · `native_selected_witness_bundle_test.dag` | **25/25 PASS** (`claim_batch --wet`, srv remote ctrl-build, 2026-08-01); cold/warm wet witnesses log `compile_skipped=false` then `compile_skipped=true` |

**Open with typed trigger (not claimed-not-verified):**

| claim | status | trigger |
|---|---|---|
| #7522 net admission vs 197.13s | `admission_settle=CaptureMeasuredNetOpen` | PR2 join on-success stage-1/2 `wall_ms` from uploaded floor-attempt receipts |
| #7671 fleet native_count/fallback_count | `native_fleet_settle=EnrollmentLandedFleetRateOpen` | PR2 upload `target/native-selected-witness-transition-receipt.tsv` every enrolled floor |
| #7534 warm-hit skips semantic recompute | `warm_hit_settle=PartiallyDeliveredSkipProofOpen` | PR2 land `cross_process_hit_skips_semantic_recompute` against TYPECHECK_COMPUTE_COUNT |
| CI-job unattributed remainder | **CLOSED:** `unattributed_ci_wall=0` on subject A | `ci_job_wall_population_receipts` assigns 100% of `ci_job_wall` to non-overlapping elapsed populations; dropping the final cleanup population makes the control red |
| native bundle timing ratios | claimed-not-verified (#7599 body) | committed receipt artifact or fleet rerun |

**Not verified in tree (omitted from authority):** #6663 158.2s→61.6s, #7029
3m29s→2m19s, #7522 probable ~1–1.5 min net admission — retired; use closeout OPEN triggers instead.

---

## Related work outside this program

- **Floor prep-tax program** — [floor-prep-tax-program.md](floor-prep-tax-program.md):
  P1 REJECT banked (#7725); redirect to assembly / shared entry-view. Does **not**
  reopen retention-vs-drain.
- **Placement / build-flake lane** — srv1 cache miss + placement crawl; **do not
  raise step cap first** (cap lives in `gunbc.ci_spec`; measure after Arm A).
- **Placement chain** — host/job placement; owner `ci-placement`.
- **Frontend escape scan** — tokenizer cost; Lane A from `inner-cost-lanes-scoping.md`.
- **Compile-wall / shard cold pass** — `compile-wall-endgame.md`.

---

## Dissolution trigger

Delete this note when every sub-lane row is accepted or honestly retired and the
program parent `five-minute-ci-gate` is accepted on fleet receipts showing Stage C
on a representative leaf `.dag` PR — or when the operator recuts the program.

DESIGN refs: §1 (time is the value), §2 (one materialization kernel), §5 (refuse,
never widen), §6 (displaced cost priced in receipts).
