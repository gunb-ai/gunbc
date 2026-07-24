# CI two-tier placement redesign — the 5-second rule

**Status:** DESIGN, operator decisions recorded 2026-07-24. Authority stays with the carriers
(`gunbc.ci_spec`, `gunbc.ci_workflow`, DESIGN.md Building & checks); this doc records the
accepted placement law, the three operator decisions, and the staged dependency order. It
dissolves when D3 lands the placement axis on `CiSpec` rows — the spec becomes the authority.

## 1. Diagnosis (by receipt, not impression)

The affected-set **selection layer landed and works** (468 of 2,206 witness rows ran on the
measured run; compile-clean scopes to shard-entry closures on `.dag`-only diffs; regen skips on
provable disjointness). What never landed is **cost proportionality**: the process model still
pays whole-tree-shaped prices around a small selected workload.

- Pooled and warm, batch-1's 12 wet claims cost **25.8s resolve + 19.2s eval** (~1.6s/claim).
  Spawned cold, the same work costs **479.6s**.
- Discovery's 643s per-run resolve contains **18.4s of eval**.
- 26 cold child processes across batches 1/6/7 cost **~23.0 min ≈ 47.5% of the floor step**,
  identical on trivial diffs — a diff-independent tax.
  (All from `ci-floor-child-spawn-attribution.md` + its TSVs, 2026-07-23.)

The ~40 minutes is not compilation or testing — it is repeated re-derivation of the same module
index and closures. The redesign is therefore a **re-partition of placement**, not a rebuild of
the machinery.

## 2. The placement law (operator, 2026-07-24)

> A check rides the PR path **iff** its measured warm cost is well within **5 seconds** (the
> 5-second rule) **or** its cost is proportional to the diff. Everything else is a gauntlet row.

- **Wetness is not the axis.** "Wet" was a cheap heuristic for "slow"; the criterion is measured
  cost. A wet gate that finishes well within 5s warm rides the PR path — provided it is
  classified hermetic/ephemeral (the live-read classification lens is the existing carrier;
  the effect-grants lane is the eventual wall).
- **Fail-closed admission:** a row rides the PR tier only with (a) a measured warm-cost receipt
  and (b) a hermetic/ephemeral classification. Unmeasured or unclassified → gauntlet. No row is
  placed by taste.
- Placement is a **data axis on `CiSpec` rows** (`PrTier | Gauntlet`), never per-site prose.

## 3. Operator decisions recorded

1. **Wet-if-fast.** See §2. The acceptable threshold is to be confirmed by our own testing
   (D2's measurement pass), with 5s as the declared bar.
2. **The per-PR selection-control audit step is DELETED, not re-placed** (operator: "I have no
   idea why that audit is there — just delete it", 2026-07-24). Carrier:
   `ci_selection_control_step()` in `dag/gunbc/ci_workflow.dag` (plus its
   `gunbc_ci_selection_control_step_timeout_minutes` row), removed from the ci job's step list
   and `ci.yml` regenerated — never a hand-edit of the yml (drift gate). **§5 justification —
   this is a duplicated control, not a lost one:** the scheduled falsifier
   (`.github/workflows/falsifier.yml`, cron `0 0,4,8,12,16,20 * * *`) builds and runs
   `floor_skip_discovery_witness` cold every 4 hours with predictions recorded, and the floor's
   inline selection refusals (a provenance gap refuses, never widens) stay. A wrong skip still
   surfaces as a counted divergence within one cadence window. Deleting a control is fail-open
   only when it is the *last* control; here the surviving control is named, scheduled, and
   counted. Cost displaced: ~4–5 min on every PR and main run.
3. **Rust seed PR signal stays execution-shaped.** The cargo suite is local-only per the
   2026-07-11 ruling (`gunbc.commit_workflow.rust_tests_removed_disposition`); in CI the seed is
   tested by execution — release build + fmt gate + regen fixed-point + the selected floor
   running the binaries over the witness corpus (DESIGN §7: "its tests are data").
   Optional follow-on, decided by D2's measurements and not now: re-enroll a fast subset of the
   cargo suite under the same 5s rule.

## 4. The two tiers

**PR tier — proportional to the diff, one warm process:**
diff vs merge target (D5's `DiffBaseline`, not the `origin/main` literal) → touched modules
(path⇄module binding) → reverse import closure → compile exactly that closure → run the
witnesses bound to those modules, dry-run/mocked, in the same process → receipt + merge
admission. Plus the ≤5s gate rows. Non-selectable diffs (`.rs`, manifests, workflow files,
departed paths, selection refusals) keep today's loud widening; whether they may instead defer
to the gauntlet is a separate later decision, not assumed.

**Gauntlet — main pushes + the 4h falsifier cadence:**
whole-tree cold compile, full witness corpus with predictions compared, wet gates over 5s,
regen cold control, divergence counting. A red is loud and blocks/reverts on main.

## 5. Dependency graph (exact)

(Logical dependencies only — **delivery is compressed to one redesign PR**, §8; D0 is routed
to the #7129 worker as its own close-out PR and sequenced first.)

```
D0 #7129 footprint bound — merged, NOT closed: whole-schedule arming hoist
   + the three post-merge P1s (2026-07-24 review, below)
  └─► D1 child-spawn dissolution — gate claims run on the executor's warm
      process_shared_index ("resolve once, share by reference")          −13–17 min
        └─► D2 measurement pass — warm per-gate cost table (TSV receipt);
            also attributes discovery's 643s resolve fresh and the ~2.5m
            store-teardown cost (the unledgered rows)
              └─► D3 CiSpec placement axis (PrTier | Gauntlet) + gauntlet
                  workflow + revert-on-red policy on main
D4 selection-control step deletion — independent; can land any time
D5 DiffBaseline fix — independent; unblocks stacked-PR selection
```

**D0's post-merge P1s (2026-07-24 adversarial review of merged #7129)** — all three are
retention-truth defects, and they gate D1 because pooling multiplies whatever the retention
model gets wrong:

1. **Compile-clean's aggregate graph memo pins the whole tree.** Compile-clean warms the shared
   index before discovery and memoizes its aggregate resolve graph; discovery only removes
   per-entry subjects, so the whole-tree compile graph's `TypedModule`s retain forever. Fix:
   remove the aggregate memo subject after emission. (This is also the mechanical explanation
   for a large slice of the measured 9.2GB resident floor.)
2. **Prewarmed cache hits never register eviction keys.** The all-hit probe returns without
   `index_record_schedule_module`; only the slow path records — and compile-clean deliberately
   makes all-hit the common case. Completion then reports evictions while removing nothing.
   Fix: record keys immediately after every confirmed hit.
3. **Arming closure ≠ loader closure.** Arming uses `selection_adjacency`; the loader
   additionally scans qualified references per source, and `record_module` checks only global
   membership — so a module omitted for one entry but loaded for another is neither counted
   `RetentionUnknown` nor re-evicted. Fix: arm from the loader's exact closure authority, or
   track unknowns per entry/module. **This is the #6985 Class-B root re-surfacing** (import-edge
   adjacency standing in for the real reference closure) — its third appearance; the fix should
   name the one closure authority, not add a third adjacency.

**D1 activation preconditions** (same review): `run_claims_in_process` — the pooling terminal —
exists but is not activation-ready: it never arms/completes retention, **returns `true` for an
empty claim list where `claim_batch` refuses** (a §5 fail-open, hard-reject class), and reuses
stale source snapshots across calls. D1 = close these, then activate; the empty-list refusal
must be behaviorally identical to `claim_batch`'s.

Ordering is load-bearing twice: measuring before D1 measures startup, not gates; pooling before
#7129 hits the memory wall on capped runners (the Pi bench: pooled 72.9m ≈ spawn-sum on a
memory-starved host — the 5.1× is a warm-cache win that vanishes at the wall).

Orthogonal, compounding later: `ts-store-econ` (resolve → durable-store hits), the
census/SymbolIndex re-grounding (removes the whole-tree startup index every process pays), and
the 2a native flip (deletes the interpreted-eval class).

## 6. Current-step roster with dispositions

| Step / class | Today | Disposition |
|---|---|---|
| build (release bins) | 2.4m warm | PR tier (fixed prerequisite) |
| fmt gate (`.rs` diffs) | seconds | PR tier |
| regen fixed-point | 5.3m, skip-scoped | keep skip on PRs; cold control on main (gauntlet) |
| batch 1 compile-clean | closure-scoped for `.dag`-only; whole-tree on widen | PR tier (diff-proportional by construction) |
| batch 1 cheap gates (12 claims) | 479.6s cold-spawned | PR tier **after D1** (warm eval 19.2s total; per-claim ≪5s) |
| batch 3 discovery | ~13m, mostly resolve | PR tier (it IS the affected set); resolve cost owned by D2 attribution + store-econ |
| batches 2/4/5 rows | measured in D2 | to be rostered from `gunbc_ci_floor_gates` with receipts — not assigned by guess here |
| batch 6 source_root_ingest (3 claims) | 12.15m ≈ all child spawns | measure warm in D2; likely PR tier post-D1, else gauntlet |
| batch 7 reads_real_bytes (2 claims) | 3.3m = 2 cold children | same as batch 6 |
| resolve/materialization receipt gates | ~0s | PR tier |
| **selection-control audit** | **4–5m every run** | **DELETE (D4)** — falsifier cadence is the surviving control |
| merge-admission gate | 18s | PR tier |
| deploy | main-only | unchanged |

## 7. Acceptance and REDs

- **D0 controls (from the review's own miss):** the E2E retention control must **arm after
  prewarming** — the merged test arms before, which is exactly the order-blindness that let the
  all-hit no-registration defect through green. And the eviction-disabled "retain-all" control
  must actually retain all (today it still unconditionally evicts resolved graphs — not a valid
  pre-M2 baseline).
- **D1**: pooled gates green by execution; the child-spawn counterfactual TSV is the ready-made
  before/after (its doc bind dissolves on this landing). REDs: an empty claim list must refuse
  (parity with `claim_batch`); a stale-snapshot control (source edited between two pooled calls
  must be observed by the second).
- **D2**: a probe TSV with a warm-cost row for **every** gate; a gate without a row defaults to
  gauntlet (fail-closed). Includes the fresh discovery-resolve attribution and teardown row.
- **D3**: placement axis on `CiSpec`; witness reds when a `PrTier` row lacks a measured receipt
  or hermetic classification; leaf-PR wall recorded as an actual, not a promise (target:
  single-digit minutes).
- **D4**: step removed from the workflow authority, `ci.yml` regenerated and drift-gated;
  falsifier cadence verified live in the same PR (its most recent scheduled run linked in the
  PR body). RED: the falsifier's divergence counting must be demonstrably intact — the deletion
  PR cites a cadence run where predictions were recorded and compared.
- **Non-goals:** no witness-semantics changes; no corpus edits; receipt file formats unchanged;
  not gated on the effect-grants wall; gauntlet-deferral of non-selectable diffs deliberately
  NOT decided here.

## 8. Delivery: one redesign PR (operator directive 2026-07-24)

No staged drag-out: **measurements and decisions happen up front; the workflow changes land as
one PR.** Structure:

- **PR-0 — D0 close-out (routed to the #7129 worker, sequenced first).** The three P1 retention
  fixes + the whole-schedule arming hoist. Sequenced first for two reasons: retention receipts
  must be *true* before the redesign PR cites them, and both PRs edit the same `cli_run.rs`
  retention machinery — parallel landings would conflict textually and semantically.
- **Pre-PR probe (nothing lands).** The D2 measurement pass runs on a branch: a pooled
  warm-cost row for **every** gate (batches 2/4/5 rostered from `gunbc_ci_floor_gates`, not
  guessed), measured on **fleet-class hardware (srv)** — the Pi lesson: a warm win measured
  only on the 125 GB build box is not evidence for the capped runners. Output: (a) the
  placement roster, (b) the filled expectation sheet below. This is also where "well within
  5s" gets its empirical check (operator: "we do some testing ourselves to see what is
  acceptable").
- **PR-1 — the single redesign PR.** `run_claims_in_process` activation fixes + in-executor
  pooling (D1) · `CiSpec` placement axis with the measured roster (D3) · gauntlet split ·
  audit-step deletion (D4) · `DiffBaseline` (D5) · the probe TSV as the receipt basis. Each
  piece carries its own witness battery so review is per-piece, but the landing is atomic —
  and so is the revert: one `git revert` restores today's process wholesale. The concentration
  is deliberate.

### The before/after expectation sheet (X filled by the probe; every row falsification-bounded)

| Metric | Before (receipts) | After (expected) | Reworked if |
|---|---|---|---|
| trivial/docs-diff ci-job wall | ~40m (23m cold children + 4–5m audit + floor) | ≤ X (single-digit-minutes target) | > 2X |
| leaf `.dag`-diff floor step | ~38m | ≤ X | > 2X |
| `.rs`-diff floor step | ~38m (widened baseline) | ≤ before − children − audit | any regression |
| cold child processes / run | 26 | **0** | any spawn observed |
| selection-control step | 4–5m every run | **absent from ci.yml** | present |
| per-gate warm cost | unmeasured | every `PrTier` row ≤5s, receipt attached | any row over |
| peak floor RSS / cgroup | 9.2 GB / 10.7 GB | ≤ before + small margin (in-executor pooling is ~0 marginal) | clamp regime entered |
| gauntlet safety | n/a | planted violation caught ≤ 1 cadence window (RED demonstrated in PR) | missed |
| falsifier divergence classes | baseline | no new class | new class appears |

A missed After bound means the placement model missed a cost class: **rework the roster — never
widen a budget to absorb it** (§5; the absorbing-fallback rule applied to our own plan).

## 9. Pre-PR decisions (sign-off checklist for plan reviewers)

1. **"Well within 5s" made crisp** — recommendation: warm p95 ≤ 5s per gate row, measured on
   srv-class; the roster records the measured value, not a pass/fail bit.
2. **Gauntlet home** — recommendation: extend `falsifier.yml` (the cadence already exists) plus
   a post-merge main-push job for the wet set; alternative is cadence-only (cheaper; detection
   latency up to 4h).
3. **Red-on-main mechanics** — recommendation: auto-file a loud issue + operator-click revert;
   no auto-revert in this iteration.
4. **`DiffBaseline` (D5) placement** — recommendation: rides PR-1 with its own witnesses;
   acceptable alternative: a tiny separate PR landed before PR-1.
5. **Non-selectable diffs** — confirmed unchanged in PR-1 (loud widening stays; deferral is a
   later, separate decision).

Sign-offs recorded here with name + date once reviewed.
