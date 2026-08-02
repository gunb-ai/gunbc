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

**Native witness execution — settled at small scale (#7599, open PR).** The
program no longer debates whether moving semantic execution out of the
interpreter works. **In-flight executing slice** (#7599, head
`2f9e780e6`): modeled `SelectedWitnessPlan`, content-addressed bundle identity,
one-process direct calls, cold/warm receipts, interpreter equivalence per
member, swapped-body divergence control, typed refusals (stale compiler identity,
missing native realization, missing red evidence, invalid process counts, fallback
use). Dispositive at small scale per branch witness receipts; **banking trigger =
#7599 merge to main**. **Open question (operational):** how rapidly can the
production selected set migrate onto that native execution kind while keeping
equivalence, fallback, artifact identity, and memory fail-closed? Next major CI
slice after #7599 merges: **bounded production witness cutover** to the native
execution kind (dispatched to the #7599 owner) — not another broad interpreted-path
optimization.

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
| 1 | Finish **#7522** (`warm-merge-admission`) with real net wall receipt and stage profile | **NEXT** — probable net recovery ~1–1.5 min (not ~3 min); final-head run decides |
| 2 | Merge **#7599** once controls and reviews clear | **in-flight** — banking trigger = merge |
| 3 | **Immediately** cut over a bounded production witness population to the native execution kind | **NEXT** — #7599 owner; first major CI slice after merge |
| 4 | Continue **#7534** as shared materialization substrate — no default CI savings credited until ordinary invocations consume it | **NEXT** under `module-grain-materialization` |
| 5 | Union program **closed** per #7533; remaining assembly observation → `per-entry-assembly-decomposition` (narrow measured lane) | **CLOSED** / **NEXT** |
| 6 | This program (`five-minute-ci-gate`) makes the five-minute objective and this graph explicit | this registration |

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
| `warm-merge-admission` | Stamp and admit merges from warm receipts — fold resolve/materialization receipt gates into merge admission. **NEXT:** #7522 in flight. |
| `native-selected-witness-bundle` | **in-flight:** #7599 executing slice (plan, bundle identity, native one-process calls, interpreter oracle; banking trigger = merge). **NEXT:** production cutover of bounded selected-witness population onto native execution kind. |
| `module-grain-materialization` | Shared materialization substrate (#7534 continues). Hierarchy above; no default CI savings until ordinary invocations consume hits. **First rung:** exact-tree (#7534, opt-in). |
| `pre-index-materialization-lookup` | Warm manifest lookup before corpus-scale index construction — serve/dashboard path. |
| `phased-single-process-ci` | Regen, floor, and admission share one initialized substrate with separate verdict stamps. |
| `per-entry-assembly-decomposition` | Narrow measured lane after #7533 closed union — assembly/reconcile attribution only. |

---

## Dispositions beside the program (not sub-lanes)

| subject | disposition |
|---|---|
| `entry-graph-union-construction` | **CLOSED** — #7533: `decision_ratio = 0` repeated typecheck misses. Redirect to `per-entry-assembly-decomposition`. Slice 1 banked as #7483. |
| retention (M2) | **PARKED** unless M2-aware floor shows pressure (#7581: 842 entries, peak 6.27 GiB, PASS width 1). |
| exact-tree materialization (#7534) | Substrate under `module-grain-materialization`; opt-in; no default CI activation credited. |

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
| slice 2 zero-repeat | #7533 PR body (open) | `decision_ratio = 0` across three disjoint 50-entry windows |
| M2 retention width-1 | #7581 PR body | peak 6.27 GiB, `schedule_evictions=2094`, 842 entries PASS |
| child-spawn tax | [ci-floor-child-spawn-attribution](ci-floor-child-spawn-attribution.md) | single cold child 49.8s (CI) vs 47.2s pooled; 389s Pi spawned (8.2×) |
| compile clean 3-root | [ci_floor_pi_srv_stretch TSV](../probes/ci_floor_pi_srv_stretch_2026-07-23.tsv) | 3m29s / 5.9GB standalone CLI compile |
| native bundle witness suite | **executed** at #7599 head `2f9e780e6` · `native_selected_witness_bundle_test.dag` | **25/25 PASS** (`claim_batch --wet`, srv remote ctrl-build, 2026-08-01); cold/warm wet witnesses log `compile_skipped=false` then `compile_skipped=true` |

**Claimed-not-verified (provenance typed; not at executed-receipt altitude):**

| claim | provenance | verification trigger |
|---|---|---|
| native bundle timing (3-fn slice) | reported in #7599 PR body at `2f9e780e6` (`CTRL_BUILD_MODE=local` author run) | merge + committed receipt artifact or fleet rerun; `claim_batch` does not emit native/interpreted wall breakdown |
| ~3 ms native direct / ~20 ms interpreted / ~457 ms warm `cargo run` | same | same |
| cold+warm host witness ~993 ms; swapped-body red ~977 ms | same | same |

**Not verified in tree (omitted from authority):** #6663 158.2s→61.6s, #7029
3m29s→2m19s, #7522 26m05s floor + 197.13s baseline, #7522 probable ~1–1.5 min
net admission — operator expectation only until measured.

---

## Related work outside this program

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
