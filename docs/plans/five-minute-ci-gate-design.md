# Five-minute CI gate — program scoping

Scoping only. No implementation lands from this note alone. Registers the
operator-signed product boundary and six sub-lanes in `gunbc.roadmap_authority`
so dispatch, witnesses, and receipts share one program identity.

**Product boundary (verbatim):** an ordinary source edit reaches a required CI
verdict in five minutes without recomputing any semantic fact whose inputs did
not change.

**This is a program, not a PR.** Each sub-lane is independently dispatchable;
the parent carries the end-to-end contract only. Receipts stay on the sub-lane
that owns them — the parent carries no stored rung or wall-clock field.

**Interpreter endgame (linked, not duplicated):** moving `cli_run`/interpreter
helpers to `.dag` improves authority but does **not** make CI fast while the
`.dag` is still interpreted. The win arrives when `.dag` is the source
authority for a native, reusable realization — *authored in `.dag` → emitted
once to native → content-addressed → reused → thin native CLI* — not
*large hand-written Rust executor → large interpreted `.dag` executor*.
`cli_run`'s end-state is a thin host adapter (parse invocation, open
materialization provider, select generated plan/executor, perform host effects,
render typed outcome). See [witness realization plan](witness-realization-plan.md)
and `dag/gunbc/plans/cli_run_hollowing_plan.dag`.

---

## Sub-lanes (six)

| id | role |
|---|---|
| `warm-merge-admission` | Stamp and admit merges from warm receipts — fold resolve/materialization receipt gates into merge admission so the PR path does not pay cold recomputation for facts already fixed for the run. **In flight:** #7522. |
| `native-selected-witness-bundle` | Run only the witnesses the affected set selects, natively on the pooled floor — selection shrinks work; native execution avoids per-row process tax. Builds on #6879 partition kernel + #7029 shared runtime. |
| `module-grain-materialization` | Module-grain semantic memo — editing one module recomputes only that module, semantically affected dependents, and the affected shard. **First rung:** exact-tree cross-process cache (#7534, opt-in) — not the final consumer and does not close dashboard startup (serve still builds the corpus index before the probe). |
| `pre-index-materialization-lookup` | Warm manifest lookup before corpus-scale index construction — the serve/dashboard fix where index build currently precedes the cache probe. |
| `phased-single-process-ci` | Regen, floor, and admission share one initialized substrate with phase boundaries and separate verdicts — regen still gates floor; **not** flattening into ordinary batches. |
| `per-entry-assembly-decomposition` | Remaining compiler term after the shared-typecheck hypothesis closed — measurement first, owned by the slice-2 lane (#7533). |

---

## Dispositions beside the program (not sub-lanes)

| subject | disposition |
|---|---|
| `entry-graph-union-construction` | **CLOSED** — slice 2 (#7533, candidate head): three disjoint N≤50 windows + reorder control, `decision_ratio = 0` repeated typecheck misses, order-stable. No union construction; redirect to `per-entry-assembly-decomposition`. Slice 1 banked as #7483. |
| retention (M2) | **PARKED** unless a real M2-aware floor shows pressure. Dispatch J receipt (#7581): 842-entry whole-corpus batch 1, peak **6.27 GiB**, `schedule_evictions=2094`, `retention_unknown=0`, PASS at width 1. |
| exact-tree materialization (#7534) | First rung inside `module-grain-materialization`; opt-in; explicitly does not close dashboard startup. |

---

## Product budgets (targets, not achievements)

| envelope | target |
|---|---|
| normal affected PR, warm | p50 ≤ **5 min** |
| broad/cold PR | p95 ≤ **10–15 min** |
| falsifier / cold corpus cadence | ≤ **20–30 min** initially |
| witness bodies (once native artifacts exist) | seconds, not minutes |

**Honest current expectation** after the in-flight board: substantial PR ~30–35 min;
broad/cold 35–45 min. Nothing currently in review provides the remaining 5–6×; it
comes from native realization + module-grain materialization.

---

## Verified receipts (execution-backed in tree)

Only figures verified in committed receipts or PR bodies are cited here.

| receipt | source | what stands |
|---|---|---|
| slice 1 partition | #7483 · [entry-graph-union slice 1](entry-graph-union-slice1-measurement.md) | exclusive partition; `load` eliminated as union target |
| slice 2 zero-repeat | #7533 PR body (open) | `decision_ratio = 0` across three disjoint 50-entry windows |
| M2 retention width-1 | #7581 PR body | peak 6.27 GiB, `schedule_evictions=2094`, 842 entries PASS |
| child-spawn tax | [ci-floor-child-spawn-attribution](ci-floor-child-spawn-attribution.md) | single cold child 49.8s (CI) vs 47.2s pooled; 389s Pi spawned (8.2×) |
| compile clean 3-root | [ci_floor_pi_srv_stretch TSV](../probes/ci_floor_pi_srv_stretch_2026-07-23.tsv) | 3m29s / 5.9GB standalone CLI compile |

**Not verified in tree (omitted from authority):** #6663 158.2s→61.6s, #7029
3m29s→2m19s, #7522 26m05s floor + 197.13s baseline — flag for operator receipt
before transcribing.

---

## Related work outside this program

- **Placement chain** — host/job placement; separate lane owner `ci-placement`.
- **Frontend escape scan** — tokenizer cost shape; landed Lane A from
  `inner-cost-lanes-scoping.md`.
- **Compile-wall / shard cold pass** — may compress the envelope once
  materialization and selection are live; tracked in `compile-wall-endgame.md`.

---

## Dissolution trigger

Delete this note when every sub-lane row is accepted or honestly retired and the
program parent `five-minute-ci-gate` is accepted on fleet receipts showing a
representative leaf `.dag` PR at ≤5 minutes wall with byte-identical verdicts vs a
cold recompute control — or when the operator recuts the program.

DESIGN refs: §1 (time is the value), §2 (one materialization kernel), §5 (refuse,
never widen), §6 (displaced cost priced in receipts).
