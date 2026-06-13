# Affected-set CI gating recommendation — ci_floor (Wave-1 §11.7 kill-criterion aggregate)

Work item: `adhoc-0fa0067b-131` (sunny-ant-116) · CI-investigation tree (swift-stag-552) ·
operator un-parked `#g-fusion` 2026-06-12.

This is the **aggregate + written recommendation** the kill-criterion instrumentation was built to
feed (see `affected-set-ci-kill-criterion-instrumentation-2026-06-02.md`). It is a
**recommendation, not a gating flip**. Per the CI-investigation manager brief and the dep-graph
coordinator (snappy-crab-849) hard constraint: **no CI gating change lands without coordinating
through the dep-graph coordinator**, and any selection layer must *narrow* the corpus-gate semantic
witness set being enrolled into `v4_lens_ci` (#4785, in design-sign), **not race it**.

## TL;DR

The data **supports moving toward affected-set gating of the `ci_floor` bootstrap path, but does
not support a blanket flip today.** Two preconditions must close first:

1. **Coverage gap (safety):** the affected-set `affects_v4` allowlist is *narrower* than the
   floor's actual compile closure (`v2-compiler compile --source-root src/v4`). Three real
   on-main `src/v4/**/*.dag` files in the floor's closure are classified skip-all. This is the only
   structural false-negative class in the corpus and must be closed (made fail-closed) before
   gating. It is a one-predicate semantic correction, specified below.
2. **Savings unmeasured (value):** `saved_minutes` is structurally `0` in the v1 receipt
   (`actual_run_minutes` is never populated). The wall-clock payoff is *modeled* (≈ a few minutes
   per skip-eligible PR) but **not yet measured**. Wire the job-timings aggregator before claiming
   savings as a gating justification.

Empirically the *prediction* is already safe: **0 diff-attributable floor false-negatives** and
**0 fail-closed rows** across 1988 receipts. The risk is structural (the allowlist gap), not
observed run history.

## Corpus

| | value |
|---|---|
| Receipt window | 2026-06-02 02:01Z → 2026-06-12 23:43Z |
| Total receipts (`affected-set-ci-receipt` artifacts) | 1988 |
| Schema version | all `1` |
| Distinct workflow runs | 1986 |
| Distinct branches | 321 |
| `fail_closed` rows (git-diff read failed) | **0** |

All 1988 artifacts downloaded and parsed; none expired, none unreadable. Method and reproduction in
the appendix.

## Skip rate — how often would gating let us skip the floor?

`bootstrap_required = false` ⇒ the affected-set would skip the `ci_floor` v2→v4 bootstrap path
(`ci_floor` stage0 freshness, `ci_floor_parity` DAG parity + census, `ci_floor_emit` M1 emit probe).

| view | would-skip / total | rate |
|---|---|---|
| per-run (every push) | 274 / 1988 | **13.8%** |
| per-branch (latest run per branch ≈ per-PR) | 71 / 321 | **22.1%** |

Within the would-skip rows: **219 runs** select *nothing* (docs-only / fully-empty selection),
**55 runs** are v3-only. These two classes are the unambiguously-safe slice — neither touches any
v2/v4 bootstrap input — and they are where the first gating step should live.

Selected-component frequency (per-run, n=1988): `testclaim_corpus` 67.1%, `v4` 62.8%, `v2` 30.5%,
`v3` 24.2%, `workflow_policy` 12.6%, `release_distribution` 0.9%. The floor is dominated by the
v4/testclaim closure — consistent with most real PRs touching `src/v4`.

## False-negative analysis — would gating have hidden a real failure?

For the 274 would-skip runs we pulled the actual GHA job conclusions for the three floor jobs.

| floor-job conclusion across would-skip runs | count |
|---|---|
| success | 472 |
| cancelled (concurrency supersede / known lens 35m-wall, now resolved) | 103 |
| **failure** | **1** |

The single failure (run `27102544145`, branch `docs/consolidation-authority-dag`) is a
**markdown/docs-only diff** (`DIRECTION-CHECKLIST.md`, `ROADMAP.md`, `THESIS.md`,
`docs/thesis/doc-authority.md`, `scripts/check_doc_refs.py`). The failing floor job is `ci_floor`
(v2 **stage0 freshness**) — a property that **cannot** be broken by a markdown edit. It is
base-inherited staleness or infra flake, **not diff-attributable**. So:

- **Diff-attributable floor false-negatives: 0 / 274.**
- `cancelled` rows are not failures (superseded pushes + the v4_lens 35m-wall that #4719 resolved
  2026-06-12); they are excluded from the false-negative count, not counted as passes.

Two `v4_lens_gate` failures also occurred on would-skip runs (`valiant-crane-469`,
`loyal-moth-527`). Lens is **outside** the `bootstrap_required` model (the kill-criterion models the
ci_floor bootstrap path only), and both align with the now-resolved lens infra wall, not a
selection defect. Flagged because **a future selection layer that also gates `v4_lens_ci` must
treat lens coverage as its own question** — this receipt does not certify lens skip-safety.

## Coverage gap — the one real structural false-negative class

The M1 emit probe (`.github/ci-floor/v4-rust-full-tree-emit-probe.sh`) compiles the **whole tree**:

```
v2-compiler compile --source-root src/v4 --target rust
```

So *every* `src/v4/**/*.dag` is in the floor's compile closure. But the detector's
`ci_changed_path_affects_v4` (`tools/ci_affected_components/src/lib.rs`) is an **allowlist** of
specific prefixes (`compiler/`, `std/`, `extdeps/`, `lens/`, `bin/main.dag`,
`workflow/bootstrap.dag`, specific test paths). Real `src/v4` `.dag` files fall *outside* it and are
therefore classified skip-all:

| unclassified `src/v4/*.dag` on main, in the floor closure | seen in would-skip runs |
|---|---|
| `src/v4/program.dag` | yes (3) |
| `src/v4/workflow/runtime_run.dag` | yes (4) |
| `src/v4/workflow/lens_ci_gate.dag` | yes (1) |

**8 would-skip runs** actually touched one of these three on-main paths. (A fourth path,
`src/v4/program/program.dag`, also appeared in the receipts but only on the unmerged
`royal-ferret-510-s4-runnable-io` branch — it is *not* in main's `--source-root src/v4` closure, so
it is excluded here; every run that touched it also touched `workflow/runtime_run.dag`, so the count
of 8 is unchanged.) None happened to break the full-tree emit, so it never bit — but **gating the
floor today would risk skipping a v4-emit regression on exactly these paths.** This is the gap that
must close before any flip.

**Fix (specified, fail-closed, turnkey — but deliberately NOT applied in this PR; see scope note):**
make the v4 predicate default-*include* any `src/v4` `.dag` so the detector closure ⊇ the
emit-probe closure:

```rust
// ci_changed_path_affects_v4: add a catch-all so the detector's src/v4 closure is a superset
// of `--source-root src/v4` (the emit probe's actual closure). Over-selection is safe; the
// allowlist's under-selection is the only structural false-negative.
|| (path.starts_with("src/v4/") && path.ends_with(".dag"))
```

This converts the only structural false-negative class into safe over-selection. It touches a
mirror-parity-checked detector, so it is a **separate, coordinated change** (it also widens what the
shadow receipt predicts), not folded into this recommendation doc.

## Savings — not yet measurable from receipts

`saved_minutes` is `0.0` in 1987 / 1988 rows; `actual_run_minutes` is `0` in **all** rows (the
`affected` job runs before job timings exist — a known v1 limitation). The lone `saved_minutes =
15.0` row is the receipt's own introducing branch (`snappy-tern-441`, 2026-06-02 02:05Z) running an
**early pre-guard binary** (it reports savings with `bootstrap_required=true` and
`actual_run_minutes=0`, both of which the shipped `saved_minutes` guards now forbid). It is a single
bootstrap-day artifact and is excluded.

Modeled (not measured) ceiling: 22.1% per-PR skip × the provisional ~13–15m floor p50 ≈ **~3 min
average wall-clock per PR**, concentrated on docs-only / v3-only PRs. **This must be confirmed by
populating `actual_run_minutes`** via the follow-up aggregator job (the bin already exposes
`--job-timings` / `--actual-run-minutes`) before savings is used to justify a flip.

## Recommendation

1. **Do not flip `ci_floor` gating now.** Hard constraint: coordinate through the dep-graph
   coordinator (snappy-crab-849); `#4785` corpus gate is in design-sign and any selection layer
   must narrow its witness set, not race it.
2. **Close the `affects_v4` coverage gap first** (the predicate above) as a separate coordinated
   change, with the existing ci.dag mirror-parity test updated. This is the gating blocker.
3. **Wire the job-timings aggregator** to populate `actual_run_minutes` so `saved_minutes` becomes
   real. Re-aggregate once a 20+ measured-timing sample exists.
4. **First gating step is the safe slice, not the whole 22%:** skip the floor only on
   empty-selection (docs-only) and v3-only runs — the 219 + 55 rows where the bootstrap path is
   unambiguously irrelevant. Never skip on any `src/v4` path until (2) lands. Roll out as
   shadow → canary (compare predicted-skip vs actual floor outcome on a small cohort) → enforce.
5. **Selection narrows, never expands, the corpus-gate set.** Sequence behind #4785; the value of
   selection is shrinking the per-entry closure re-resolve cost in `v4_lens_ci`
   (DRAM-bandwidth-bound — see memory `ci-lens-timeout-dram-bandwidth`), which is worth more than
   the floor-skip minutes alone.

## Appendix — method / reproduction

- All `affected-set-ci-receipt` artifacts enumerated via
  `gh api repos/gunb-ai/gunbc/actions/artifacts` (paged), one JSON receipt per workflow run.
- False-negative correlation: for each `bootstrap_required=false` run, the floor-job conclusions
  (`ci_floor`, `ci_floor_parity`, `ci_floor_emit`) were read from
  `gh api repos/.../actions/runs/<id>/jobs`.
- Floor compile closure confirmed against `.github/ci-floor/v4-rust-full-tree-emit-probe.sh`
  (`--source-root src/v4`); detector allowlist against
  `tools/ci_affected_components/src/lib.rs::ci_changed_path_affects_v4`.
- `cancelled` floor jobs are excluded from both the pass and the false-negative tallies (they are
  superseded/aborted, not signal).
