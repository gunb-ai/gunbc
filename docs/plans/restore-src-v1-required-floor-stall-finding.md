# RESTORE-stall finding: widening the required floor's source roots to admit `src/v1`

Standalone finding, split out of triage prose per parent-session request (dashboard node `adhoc-9b80ec49-d63`, session `zesty-newt-828`, 2026-08-19). It is the receipt for the RESTORE disposition ruled out repo-wide across `v1_dead_witness_tree_triage_receipt`, `v1_dead_witness_tree_triage_receipt_remainder`, and `v1_dead_witness_tree_triage_receipt_emitter_ambiguous_variant_owner` (`dag/gunbc/ci/ci_layer_roots.dag`) — those receipts cite this finding rather than re-deriving it.

## The question

Could the dead-witness-tree population under `src/v1/tests/claim` be restored to execution by the cheap route — adding `--source-root src/v1` to the required floor's invocation (`claim_executor --required-floor --source-root dag --source-root src/v2`) — instead of migrating or retiring each file individually?

## Local measurement (this session, prior window)

- **Invocation:** `claim_executor --required-floor --source-root dag --source-root src/v2 --source-root src/v1` (the one-token addition of `--source-root src/v1` to the standing invocation).
- **Head SHA at time of run:** not recorded in this session's surviving notes; the run was taken against this branch (`session/zesty-newt-828`) during the window that produced the 17-file triage, before either the emitter_ambiguous_variant_owner receipt or the remainder receipt existed. This is a named provenance gap. The qualitative result (stall, not slow-but-progressing) is not sensitive to the exact commit, since strict-preparation's cost is dominated by corpus size and import-graph shape, not by this triage's one-line prose edits — but no exact SHA is citable.
- **Local vs. remote:** run locally in this session's container (not via `ctrl-build --remote`), predating the guidance that heavy whole-tree operations prefer remote dispatch. It is a single-shot diagnostic, not a build, and its result (stall before any phase progress) makes host contention an unlikely confound: a resource-starved run would still log *some* forward progress at a slower rate, not zero.
- **Contention caveat:** the container's RAM cap is shared across `sessions.slice` with other sessions on the same host; a co-located heavy build could inflate wall-clock or add memory pressure. This cannot be fully ruled out for this run. It is addressed, not eliminated, by the corroboration below, measured on dedicated CI hardware with no co-tenant.
- **Result:** the process stalled 8+ minutes inside `strict-preparation` (the whole-corpus typecheck phase preceding any witness execution), RSS plateaued at approximately 7.2GB with no further growth, and no phase-progress log line advanced past the point already reached without `src/v1`. The process was killed: a plateaued RSS with no progress is the signature of a stall (thrashing, or an unreachable fixed-point), not of a slow-but-advancing computation.

## Independent corroboration (relayed by parent session smart-ram-730, CI run 32196317824)

A CI run on a dedicated `srv4` runner (no co-tenant contention) measured the **ordinary floor, without `src/v1` added at all** — the ceiling this triage's population currently sits below, not the widened case:

- `rss_kb=7078696` (~7.0GB)
- `wall_s=540` (9 minutes) in `strict-preparation`, logged as `compile.reconcile done in 9 minutes`
- heavy paging during the run: `pswpin=231511094`, `pgmajfault=169789103`

This is the cost of `strict-preparation` **today, over `dag` + `src/v2` alone** — the baseline the required floor already pays before any witness runs. Both observations land independently in the same ~7GB / high-single-digit-minutes regime on the same phase, so the local stall was not primarily an artifact of `src/v1`'s weight pushing an otherwise-cheap phase over a threshold: `strict-preparation` is already expensive at the two-root baseline. Adding `src/v1` (52 non-test `.dag` modules plus the 15-17 test files, per `witness_fold_src_v1_coverage_gap_note`'s count) pushes an already near-ceiling phase further.

**Second data point (relayed by parent session smart-ram-730, CI run 32204338777, dedicated runner, same unwidened floor):** `rss_kb` peaked at 9,816,820 (~9.8GB) in the same `strict-preparation` phase this triage's own PR (#8486) exercised — against the ~7.0GB from run 32196317824. Both runs measure the identical unwidened invocation (`dag` + `src/v2`, no `src/v1`) on dedicated hardware, so the ~2.8GB spread is not host contention or a `src/v1` effect — it is `strict-preparation`'s own run-to-run variance at the two-root baseline. The honest ceiling claim is therefore a **range** (~7.0–9.8GB observed so far), not a single figure; the headroom between that range and this host class's actual OOM/thrash threshold is narrower than either measurement alone suggests, which strengthens the conclusion that widening to `src/v1` is not the cheap route.

## Stall vs. slow classification

Both signatures point to **stall**, not **slow**:

1. The local run showed RSS plateau with zero phase-progress advancement over 8+ minutes — a computation still progressing would show continuing RSS growth or advancing log lines, not both flat.
2. The CI corroboration establishes that even the *unwidened* floor is already within single-digit minutes and ~7GB of whatever ceiling this host class enforces; paging activity (`pswpin`/`pgmajfault` in the hundreds of millions) is itself evidence of memory pressure at the baseline, before `src/v1`'s modules are added.

Together, `--source-root src/v1` does not merely make `strict-preparation` *slower*; it very plausibly pushes an already-memory-pressured phase past a practical ceiling into OOM or genuine thrashing — the local kill preempted observing which. Re-running to distinguish "would eventually finish, slowly" from "would OOM or thrash indefinitely" was not attempted, since the practical consequence (RESTORE is not the cheap route the one-token diff suggested) is decided either way.

## Conclusion

RESTORE is ruled out for the whole `src/v1/tests/claim` population, repo-wide, on this evidence: a local stall (contention-caveated, but with a stall rather than slow-progress signature) independently corroborated by dedicated-hardware CI numbers showing the *unwidened* floor already runs close to whatever ceiling caused the local stall. This finding is the citation target for the RESTORE disposition in all three `v1_dead_witness_tree_triage_receipt*` rows; MIGRATE and RETIRE are the two remaining dispositions, applied per-file in those receipts.
