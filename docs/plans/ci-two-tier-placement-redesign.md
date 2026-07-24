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
  (All from `ci-floor-child-spawn-attribution.md` + its TSVs, 2026-07-23. **State has since
  moved:** #7122 + #7128 pooled these — 6 children on current main, verified §10 row 1; the
  floor dropped 48.4m → 38m. The diagnosis stands as the history that priced the fix.)

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
   2026-07-11 ruling (`gunbc.commit_workflow.commit_gate_rust_suite_removed_disposition` —
   the exact decl name; DESIGN.md and `ci_spec.dag` carry a miscite whose fix is DEFERRED to
   ride PR-1: both carrying authorities are hub files, and a token-only PR pays a full-corpus
   floor for them — priced live by run 30063529282, see §9.8. Its 2026-07-15 amendment
   re-enrolled the fmt half: fmt rides CI, only the test suite is local-only); in CI the seed is
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
D0 #7129 — MERGED (bd5afd6bc3), NOT closed: whole-schedule arming hoist
   + the three post-merge P1s (2026-07-24 review, below)
D1 child-spawn dissolution — LANDED on main (#7122 one-child-per-call,
   #7128 cheap_gate_pool union): 26 cold children → 6 pooled children,
   verified by execution on run 30052571652 (floor 37m58s). Residue:
   6 pool builds/run = 1 cheap-gate union + 4 ingest overlay + 1
   reads-real-bytes. The 4 ingest children are BY CONSTRUCTION
   (ingest_pool_separation_note: root-set-keyed MultiEntryIndex over
   per-run mktemp overlays, 414s on run 30009199696; dissolves on W3,
   "never on merging the overlay roots" — D1b cannot touch them). The
   →1 endgame is therefore owned in two parts: 2 children via D1b
   (#7129 worker), 4 via W3 (store-econ lane).
  └─► D2 measurement pass — warm per-gate cost table (TSV receipt);
      also attributes discovery's 643s resolve fresh and the ~2.5m
      store-teardown cost (the unledgered rows)
        └─► D3 CiSpec placement axis (PrTier | Gauntlet) + gauntlet
            workflow + revert-on-red policy on main
D1b (deferred — §9.6) in-process claims: run_claims_in_process landed
    with #7129 (cli_run.rs:9580), INACTIVE; activation gated on D0
    close-out + its three fixes. NOT in PR-1.
D4 selection-control step deletion — PRECONDITION: the falsifier cadence
    must be green first (today it is red 5/5 by crawl-timeout, §10 row 2)
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

**D1b activation preconditions** (same review; status verified by direct read of
`origin/main:cli_run.rs:9580` on 2026-07-24): `run_claims_in_process` — the in-executor
terminal, landed inactive with #7129 as its M2 Deliverable 2 — is not activation-ready:
it **returns `true` for an empty claim list where `claim_batch` refuses** (CONFIRMED by read —
no empty arm; a §5 fail-open, hard-reject class), never arms/completes retention (CONFIRMED —
no arming call in the body), and reuses stale source snapshots across calls (plausible,
unverified). Its own docstring states the safety condition: folding claims into the long-lived
executor is safe ONLY because schedule-derived eviction bounds retention — which is exactly
what D0's P1s show is not yet true. So D1b = D0 closes → the three fixes land → activate,
with empty-list refusal behaviorally identical to `claim_batch`'s. Until then the pooled
child (which dies and frees) stays the vehicle — per `run_gunbc_claims_pooled_note`'s
recorded ruling. **Scope refinement (implementer read, 2026-07-24):** the body calls the
plain `resolve_entry_graph`, not the `_shared`/`_with_index` variants beside it — so
activation as written removes the process spawns but NOT the per-entry index rebuild; the
warm-index share ("resolve once, share by reference") is W3-class work either way. D1b is
real surgery, not a switch-flip, and its displaced cost is the smaller share of the residue.

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
| batch 1 cheap gates (12 claims) | 1 pooled union child (was 479.6s across 12 cold spawns) | PR tier (warm eval 19.2s total; per-claim ≪5s) |
| batch 3 discovery | ~13m, mostly resolve | PR tier (it IS the affected set); resolve cost owned by D2 attribution + store-econ |
| batches 2/4/5 rows | measured in D2 | to be rostered from `gunbc_ci_floor_gates` with receipts — not assigned by guess here |
| batch 6 source_root_ingest | 4 pooled overlay children, 414s (run 30009199696; the 12.15m figure was pre-pooling) | children stay 4 by construction (`ingest_pool_separation_note`); tier from D2 warm measurement |
| batch 7 reads_real_bytes | 1 pooled child (was 2 cold) | D1b-absorbable; tier from D2 |
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
  only on the 125 GB build box is not evidence for the capped runners. (The fleet IS
  reachable: branch CI runs execute on the capped srv slots; a `workflow_dispatch` harness on
  the probe branch is the instrument — no special access.) Output: (a) the placement roster,
  (b) the filled expectation sheet below. This is also where "well within 5s" gets its
  empirical check (operator: "we do some testing ourselves to see what is acceptable").
  **Added 2026-07-24 (lens reintroduction rides PR-1):** the probe also runs the v2-door lens
  audit over the corpus — route a non-blocking `validate_then_compile` pass and inventory
  every violation the seed door has been silently stamping past (`empty_complexity_report`) —
  so PR-1 flips the door knowing its red set in advance instead of discovering it on landing
  day.
- **PR-1 — the single ATOMIC redesign PR (operator ruling 2026-07-24: "CI entirely reworked
  atomically — specifically so we don't have to deal with migrations").** Contents: `CiSpec`
  placement axis with the measured roster (D3) · gauntlet split (D3b) · `DiffBaseline` (D5) ·
  the **derived clamp model** replacing the hand-set budget rows (§9.8: per-unit hard max +
  expected-average aggregate) · **the lens-door reintroduction (`ts-lens-door`)** — the
  floor's three seed-path compile sites route through the v2 door so
  `always_required_root_lenses` execute on CI, `empty_complexity_report` stamping deletes,
  and `lens_contract_complexity` flips AuditOnly → Blocking with a planted-quadratic RED;
  violations the probe's lens audit surfaced are fixed in-PR or landed as a counted typed
  quarantine roster (each row reason + dissolve-on — the §7 frontier shape, not a silent
  skip). **D4 rides PR-1 iff a green falsifier cadence run exists at landing time** (D0 is
  merging ahead, so the cadence has runway to green); otherwise it stays the fast-follow
  micro-PR citing the first green run. D1 needs nothing here (landed); D1b stays excluded
  (§9.6). Each piece carries its own witness battery so review is per-piece, but the landing
  is atomic — and so is the revert: one `git revert` restores today's process wholesale. The
  concentration is deliberate and operator-chosen.

### The before/after expectation sheet (X filled by the probe; every row falsification-bounded)

| Metric | Before (receipts) | After (expected) | Reworked if |
|---|---|---|---|
| trivial/docs-diff ci-job wall | re-anchor at probe time (attribution-era ~40m predates #7122/#7128) | ≤ X (single-digit-minutes target) | > 2X |
| leaf `.dag`-diff floor step | 37m58s (run 30052571652) | ≤ X | > 2X |
| `.rs`-diff floor step | ~38m (widened baseline) | ≤ before − audit − roster moves | any regression |
| pooled claim_batch children / run | 6 (verified, run 30052571652; was 26 pre-#7122) | ≤ 6 after PR-1; →1 endgame owned: 2 via D1b (#7129 worker), 4 via W3 (store-econ) — never by merging overlay roots | growth without a cap edit |
| selection-control step | 4–5m every run | **absent from ci.yml** | present |
| per-gate warm cost | unmeasured | every `PrTier` row ≤5s, receipt attached | any row over |
| peak floor RSS / cgroup | 9.2 GB / 10.7 GB | ≤ before + small margin (in-executor pooling is ~0 marginal) | clamp regime entered |
| gauntlet safety | n/a | planted violation caught ≤ 1 cadence window (RED demonstrated in PR) | missed |
| falsifier divergence classes | baseline | no new class | new class appears |

A missed After bound means the placement model missed a cost class: **rework the roster — never
widen a budget to absorb it** (§5; the absorbing-fallback rule applied to our own plan).

## 9. Pre-PR decisions (sign-off checklist for plan reviewers)

1. **"Well within 5s" made crisp** — the carrier ALREADY EXISTS: the witness discipline's
   "operator 5 s fast-lane law" (`run_claim_measured` / `budget_completion_outcome`,
   `cli_run.rs`, landed with #7129), measured in **thread-CPU time** with over-budget
   converting a silent Pass to a typed refusal. Recommendation: the placement threshold
   REUSES this authority (§3 — same constant, same refusal discipline), never a second
   5s definition; the roster records the measured value, not a pass/fail bit.
   **Caveat (implementer, 2026-07-24):** the carrier's quantity is per-WITNESS thread-CPU;
   placement needs per-GATE warm cost (closure resolve + startup amortization + Σ witness
   eval). Reuse the threshold and the typed-refusal shape — not the mechanism blindly: the
   probe records BOTH wall and thread-CPU per gate on fleet-class slots, and the roster's
   basis column names which quantity each row was placed on.
2. **Gauntlet home** — recommendation: extend `falsifier.yml` (the cadence already exists) plus
   a post-merge main-push job for the wet set; alternative is cadence-only (cheaper; detection
   latency up to 4h).
3. **Red-on-main mechanics** — recommendation: auto-file a loud issue + operator-click revert;
   no auto-revert in this iteration.
4. **`DiffBaseline` (D5) placement** — recommendation: rides PR-1 with its own witnesses;
   acceptable alternative: a tiny separate PR landed before PR-1.
5. **Non-selectable diffs** — confirmed unchanged in PR-1 (loud widening stays; deferral is a
   later, separate decision).
6. **D1b vehicle (in-process claims vs pooled children)** — recommendation: keep the pooled
   child (green today, dies-and-frees) and route `run_claims_in_process` activation to the
   #7129 worker after D0 closes; PR-1 takes no dependency on it. Rejecting this means PR-1
   inherits cross-owner `cli_run.rs` surgery — the drag the single-PR directive exists to
   avoid. (Weight shifted 2026-07-24 by the implementer's read: D1b absorbs only 2 of the 6
   residual children and does not remove their per-entry resolves — §5.)
7. **D4 delivery shape** — resolved per review finding 5: fast-follow micro-PR after the
   first green falsifier cadence post-PR-0; PR-1 does not wait on it (§8).
8. **Batch-3 budget × hub-file selection — PR-1 will hit this wall; decide the raise now.**
   Live receipt (run 30063529282, 2026-07-24, srv4-05): a one-token edit to `ci_spec.dag` +
   `design_document.dag` selected the full corpus — 2,315 witnesses, **all PASS**, RSS 8.9 GB
   healthy, no thrash — and batch 3 refused at wall 1,498,748 ms vs the 1,320,000 ms budget
   (`gunbc_ci_floor_batch_wall_budget_seconds[2]`). Attribution is clean: the same branch
   without the hub files ran green (0d54cdc340) hours earlier. So the honest full-corpus
   discovery wall on a capped slot is ~25 min today, above the 22-min budget — and PR-1
   necessarily touches `CiSpec` (the placement axis lives there). **Escalated fleet-wide
   2026-07-24 (operator: "most CI runs are running into budget issues"):** seven observed
   full-corpus batch-3 walls across five branches — 1344 · 1381 · 1499 · 1543 · 1564 · 1582 ·
   1629 s — every one over the 1320s budget, every sampled failure with ALL witnesses passing
   and memory healthy; meanwhile main is 12/12 green through and after #7129's merge (small
   selected diffs). Verdict: **no regression — the budget is mis-denominated.** A scalar
   wall-time budget conflates three axes: workload size (selection is diff-proportional BY
   DESIGN, ~5× swing), host speed (±20% fleet envelope), and the quantity actually worth
   bounding (per-entry cost creep — ~1.57–2.0 s/entry stable across all seven points). No
   fixed value is right: tight enough to catch regressions on median runs = guaranteed red on
   every legitimate plumbing PR; loose enough for full-corpus-on-slow-host = catches nothing.
   **Durable fix (rides PR-1's CiSpec work): re-denominate** —
   `budget = fixed_overhead + selected_units × per_unit_rate[host_class]`, with the per-unit
   rate the operator-signed constant (it is the regression dial the humans already compute by
   hand in every one of these triages) and the 55-min step timeout staying the absolute
   backstop. **Operator ruling (2026-07-24): no interim raise — the clamp model lands with
   PR-1 atomically.** The signed shape is TWO constants per unit class, with different jobs:
   a **hard max per unit** (witness: 5s — the existing fast-lane authority, unchanged, typed
   refusal per witness) and an **expected average** as the aggregate coefficient (witness:
   1s). Batch clamp = overhead + units × average (full corpus ≈ 300s + 2,316 × 1s ≈ 44 min,
   under the 55-min cap, ~1.6× the honest healthy wall — catches runaway quickly, never
   refuses legitimate work); the per-unit max protects the tail the average can't see.
   Changing either constant requires an appended operator-signed line (the existing
   `budget_note` discipline re-pointed at these constants; the unit count needs no signature —
   the schedule computes it). The hand-set `gunbc_ci_floor_batch_wall_budget_seconds` rows
   delete. These clamps are declared interim mechanics: the structural wall is the complexity
   lens (§8 — every clamp is `cost ≤ a + b·n`, the linearity assertion the lens makes
   structurally at compile time), and the clamp demotes to host-pathology backstop when the
   lens goes Blocking.

Sign-offs recorded here with name + date once reviewed.

## 10. Issue-closure checklist (operator-requested 2026-07-24; worked through by execution before review)

Every issue from the 2026-07-23/24 CI discussions, keyed to its closing mechanism and the
verification actually performed. Statuses are honest: LANDED (verified), PLANNED (in PR-0/PR-1),
BLOCKED (named precondition), OUT-OF-SCOPE (named owner elsewhere — listed so nothing silently
drops).

| # | Issue | Mechanism | Status · verified how (2026-07-24) |
|---|---|---|---|
| 1 | 26 cold child processes (~23 min, 47.5% of floor) | #7122 one-child-per-call + #7128 `cheap_gate_pool` union (K-chunked, cap 16/child) | **LANDED** — 6 pooled children counted in run 30052571652's ci log (readiness-probe quadruples at 6 distinct timestamps); floor 48.4m → 37m58s. Residue 6 = 1 union + 4 ingest-overlay (by construction, W3-only) + 1 reads-class (D1b-absorbable; D1b removes spawns, not per-entry resolves) |
| 2 | Selection-control audit (4m08s every run) | D4 deletion; falsifier cadence = surviving control | **BLOCKED — do not delete yet**: falsifier is red 5/5 scheduled runs (latest 30044928186: crawl regime, cgroup pinned 16.1G, swap 32G saturated, one module typecheck 3,171s, killed at the 170m cap). Today the per-PR audit is the only WORKING selection control; deleting it now would be the §5 fail-open. Expected unblock: #7129 eviction + PR-0's P1 fixes let the cold walk finish; D4 ships as a fast-follow micro-PR citing the first green cadence run (§9.7) |
| 3 | Retention truth (compile-clean graph pinned · all-hit keys unregistered · arming≠loader closure) | PR-0, #7129 worker | **PLANNED/ROUTED** — empty-list fail-open + missing arming CONFIRMED by direct read of `run_claims_in_process` (cli_run.rs:9580); blocker 1 is also the falsifier-crawl mechanism (row 2) and part of the 9.2GB floor |
| 4 | `run_claims_in_process` fail-open (empty→true) | D1b activation fixes, #7129 worker | **PLANNED/ROUTED** — must NOT activate in PR-1 (§9.6) |
| 5 | Discovery ~643s resolve (eval 18.4s) | D2 fresh attribution → store-econ class | **OPEN, probe-owned** — stale "fixpoint" lever retired; no fix dispatched before attribution |
| 6 | Store teardown ~2.5m (paid-twice Drop) | D2 ledger row; store-econ / ROADMAP ① | **OUT-OF-SCOPE for PR-1**, named owner |
| 7 | Whole-tree startup index every process (#6848 census heads) | census re-grounding on SymbolIndex (namespace lane) | **OUT-OF-SCOPE**, mitigated: 27→7 payments/run via pooling |
| 8 | DiffBaseline `origin/main` hardcode (stacked-PR mis-selection) | D5 in PR-1 | **PLANNED** — brief already delivered |
| 9 | Trivial-diff floor tax | rows 1+2 combined | **PARTIALLY LANDED** — children pooled; audit minutes pend row 2; sheet re-anchors Before at probe time |
| 10 | 9.2GB resident floor | PR-0 blocker-1 (compile-clean unpin) + census (row 7) + walk_memo (named follow-on) | **SPLIT** — largest slice PLANNED (PR-0); residuals named, owned elsewhere |
| 11 | 2,652 `UnlistedImportUse` fork (CLI vs floor compile-clean policy) | needs an owner — namespace-lane promotion staging vs fail-open fork, undetermined | **UNOWNED — flagged to operator** (not this plan's scope; recorded so it cannot drop) |
| 12 | `.rs`-diff whole-tree widening | deliberate policy (§3.3, §9.5) | **UNCHANGED BY DESIGN** |
| 13 | Serial chain ~10m (build 2.4 + regen 5.3 + deploy 2.2) | regen scoping exists; further work unpriced | **OUT-OF-SCOPE**, named residual |
| 14 | 5s rule needs a crisp definition | already modeled: the fast-lane law (thread-CPU, typed over-budget refusal) | **EXISTS** — §9.1 reuses the threshold + refusal shape; the per-GATE quantity is the probe's to define (§9.1 caveat) |
| 15 | Batch-3 budget under the honest full-corpus wall — fleet-wide, most PR runs red | §9.8 — interim: one signed raise to ~1680s on main; durable: re-denominated budget (overhead + units × rate[host]) in PR-1 | **OPEN, escalated** — 7 walls (1344–1629s) on 5 branches, all witnesses passing, main 12/12 green: no regression, mis-denominated budget |

## 11. D5 — the DiffBaseline brief (in-tree copy)

The brief was delivered in-chat 2026-07-24 with no in-tree artifact; the implementer could not
find it. This section is the authoritative copy — self-contained so PR-1 needs no side-channel.

**Defect.** `dag/gunbc/ci_diff_defaults.dag` holds `ci_merge_base_ref: String = "origin/main"` —
a bare string literal standing where a typed ref belongs; DESIGN §3(c)'s own tell (an argv/row
carrying a literal it should receive as a parameter) live in our own CI modeling. Four sites
conflate three facts (which ref · which remote · which policy):

1. `gunbc.ci_spec` `diff_policy` (base: `ci_merge_base_ref`, head `"HEAD"`, `DiffMergeBase`) —
   the selection authority every affected-set decision flows through.
2. `merge_admission_produce` — consumes the same literal.
3. `dispatch_git_remote_ref` (`roadmap_dispatch_actuator.dag`) — a **forked second carrier** of
   the same fact (§3 nickname).
4. `Push { branches: ["main"] }` literals in `ci_workflow.dag` — the workflow-trigger copy.

**Live consequence.** `GITHUB_BASE_REF` is consumed nowhere, so a **stacked PR** (base = another
PR's branch) diffs against `origin/main` and mis-selects its affected set — selection
correctness, not cosmetics.

**Model.** One authority:
`DiffBaseline = MergeTarget | PushParent | OperatorOverride { ref: GitRef }` —
pull_request events resolve `MergeTarget` from the event's base ref; push events resolve
`PushParent`; the override is a declared row, never an env toggle (§5 no escape hatches).
Grounded on the existing extdeps git atoms (`GitRef`, `git_remote_ref_parts`,
`git_diff_range_argv`) — no new string vocabulary. All four sites re-ground on it; the
dispatch fork dissolves.

**Witnesses.** RED: a synthetic stacked PR (base = a branch, not main) must select against
that branch — today it selects against origin/main, so the RED is discriminating on arrival.
Controls: push-event resolves PushParent; override row round-trips; `ci.yml` drift gate green
after regen.
