# v4 predicate dependency graph — from 0/6 to 6/6 PROVEN

PM-authored forward-projection. Maps the labour-graph from current state (0/6 v4-done predicates PROVEN per `docs/planning/v4-done-predicate-burn-down-2026-05-30.md`) to all six PROVEN.

**State-of-truth deferral**: this doc forward-projects labour and dependencies. It does NOT re-author predicate state. The authoritative predicate-state source is merry-badger-222's `docs/planning/v4-done-predicate-burn-down-2026-05-30.md`; the authoritative MW-D8 source is sharp-otter-407's `docs/planning/v4-mw-d8-wave1-exit-ledger-2026-05-30.md`.

**Scope**: not a schedule, not a timeline. Each work-unit is named by the receipt it produces, the gating dependencies, and the responsible manager lane.

## §1. The six predicates (TASKS.md:806–817, verbatim)

Per PR #3938 §8 D4: v4-done = ALL six PROVEN collectively. Any single predicate at non-PROVEN blocks v4-done close. **No relaxation by manager pass.**

Authoritative-source citations verbatim from TASKS.md + burn-down blocking-receipt:

| # | Predicate (TASKS.md verbatim) | Anchor | Blocking receipt per burn-down |
|---|---|---|---|
| P1 | Every other scheduled task in this plan complete (whole plan minus T-15) | `src/v4/TASKS.md:806-812` | Meta-gate; each scheduled task PROVEN, named-blocked, or out-of-scope per its own gate |
| P2 | v4 compiles `src/v4/compiler/*.dag` end-to-end | `src/v4/TASKS.md:813` | **Compiler-of-record pipeline** active + **resolve-posture bridge** (`.github/workflows/ci.yml:293-300`) deleted (per burn-down P2 row: "Resolve-posture bridge still live; v4 compiler-of-record not proven") |
| P3 | v4 emits Rust source that compiles to a binary | `src/v4/TASKS.md:814` | **emit → binary** PROVEN: v4 emits Rust source, that Rust compiles to a working binary (per burn-down P3 row: "~2978 E0423 class — dominant Pareto; landing by Jun 1 is possible but emit→binary PROVEN is not") |
| P4 | Binary on `src/v4/compiler/*.dag` produces bit-identical output | `src/v4/TASKS.md:815` | **Self-host fixed-point**: binary regenerates bit-identical output across two consecutive runs on compiler source (per burn-down P4: "T-15 runner scaffold; B1 pins open; W2.5 ladder fixtures support path only; bit-identical fixpt is Wave 3 (W3.5)") |
| P5 | TestClaim suite passes | `src/v4/TASKS.md:816` | **Modeled T-22 eval + structural-bridge deletion**: per burn-down P5 row, "Jun 1 anti-shelfware asks structural-bridge deletion schedule, not full T-38 GREEN. `scripts/v4-testclaim-corpus-gate.sh` likely still live Jun 1." |
| P6 | Hand-authored Rust is not the editable authority — proven by REPRODUCTION (A3) | `src/v4/TASKS.md:817` | **Reproduction**: rebuild-from-(.dag + frozen-pinned seed)-only reproduces the pinned hash; the seed's own hash matches its pin. Gated on P4 + P3. |

Note on P3: predicate is specifically about Rust-source-to-binary (TASKS.md:814 verbatim). Python + Go emit (per SUPPORTED.md + flavor (iv)) are **alpha/WIP outside the v4-done predicate bar**; they don't block P3 PROVEN.

## §2. Dependency graph

Per-predicate dependency-list (replaces prior ASCII graph; cursor RC#2 noted the prior graph encoded the pre-fix multi-target P3 framing that §3 now contradicts):

```
v4-done = 6/6 PROVEN
  requires: P1 ∧ P2 ∧ P3 ∧ P4 ∧ P5 ∧ P6 all PROVEN

P1 (every-other-task, TASKS.md:806-812)
  ← P1-A every-other-task roster authoring (Close/Receipt)
  ← P1-B per-task close adjudication (Close/Receipt)
  ← P1-C close-remaining-open-tasks (distributed manager lanes)
  ← (W3.1 / T-24 generated ci.yml emit may home here per operator decision D2,
     unless self-host fixed-point harness explicitly consumes generated CI)

P2 (compiler-of-record self-bootstrap, TASKS.md:813)
  ← P2-A compiler-of-record pipeline active end-to-end on src/v4/compiler/*.dag
       (NO resolve-posture bridge fallback; scoped to compiler subdir, NOT full corpus)
  ← P2-B resolve-posture bridge (.github/workflows/ci.yml:293-300) deleted
  ← P2-C P2 PROVEN ledger entry

P3 (v4 emits Rust source that compiles to a binary, TASKS.md:814)
  ← P3-A SG-1 TargetAtomRealization lands (#3956 in flight)
  ← P3-B fresh M1 rustc probe + tail-classification by missing modeled fact
       (operator rule: "fresh measurement after systemic fix")
  ← P3-C SG-6 BoundedLattice (W2.2 follow-on)
  ← P3-D per-class blocker closure (DFS worksheet + TR carrier cycles)
       (operator rule: each fresh measurement reclassifies the residuals)
  ← P3-E emit-to-binary executes (v4 Rust emission → cargo builds working binary)
  ← P3-F P3 PROVEN ledger entry

  Substrate-already-on-main feeding P3 readiness:
    SG-5 ✓ + SG-2 ✓ + Upsert<T> ✓ + CiUpsertStep ✓ + SG-7 dissolved ✓

P4 (bit-identical self-output, TASKS.md:815)
  ← P4-A Wave 2 closure (SG-1 + W2.3 worker landings)
  ← P4-B compiler.dag bootstraps from .dag source
       (gated: P2 PROVEN + P3 PROVEN — actual predicate bars, NOT proxies)
  ← P4-C fixed-point harness (two consecutive runs bit-identical)
  ← P4-D P4 PROVEN ledger entry

P5 (TestClaim suite passes, TASKS.md:816) — SPLIT per operator decision D4:

  P5-minimum-viable receipt (first observable):
    ← P5-A rung4 transport ships (#4047 smart-stag-885 in flight)
         ← #4046 runtime-value rosters MERGED ✓
    ← P5-B nat_semiring complete claim roster zero-Deferred (one fixture)

  P5-PROVEN (suite-wide):
    ← P5-C fixture roster widens (branch_dispatch, loop_linear_bound, more)
    ← P5-D F4 algebra-laws preservation post-emit
         (ONLY if part of authoritative TestClaim suite roster
          per operator decision D3; else this is rung-6 not P5)
    ← P5-E structural-bridge (scripts/v4-testclaim-corpus-gate.sh) deleted
         — T-22 eval becomes sole authority
    ← P5-F P5 PROVEN ledger entry

P6 (hand-Rust not editable authority, TASKS.md:817)
  ← P6-A reproduction-from-.dag (gated: P4 + P3 PROVEN)
  ← P6-B hand-Rust edit detection lens
  ← P6-C P6 PROVEN ledger entry

ALPHA-TARGET WORK (NOT in any v4-done predicate dependency chain;
parallel to predicates, tracked separately per SUPPORTED.md + flavor iv):
  #4040 weather → Python emit fix (smart-stag sub-worker, draft)
  #4041 weather → Go emit fix (smart-stag sub-worker, draft)
  weather demo verification (snappy-bee subtree)
```


## §3. Per-predicate work-unit lists

Each work-unit is named by the receipt-flip it produces. Manager lanes per PR #3938 §11.1.

### §3.1. P1 — every-other-task

**Status**: YELLOW (substrate widening across multiple lanes). Hardest predicate to measure because "every other task" depends on TASKS.md authoritative scope.

**Work units required**:
1. **P1-A — define "every-other-task" scope**: PM + Close/Receipt audit TASKS.md for the non-six-predicate task list. Output: explicit list of "other tasks" that count for P1 closure.
   - Lane: Close/Receipt (sharp-otter-407) — adjudication
   - Gating: none; can dispatch now
   - Receipt: `docs/planning/v4-p1-other-task-roster-YYYY-MM-DD.md`
2. **P1-B — per-other-task close adjudication**: for each task in P1-A roster, determine PROVEN / named-blocked / out-of-scope.
   - Lane: Close/Receipt
   - Gating: P1-A
   - Receipt: per-task status entries appended to roster doc
3. **P1-C — close remaining open other-tasks**: per blocker, dispatch worker(s) in the right manager lane.
   - Lane: distributed across manager lanes
   - Gating: P1-B per-task status
   - Receipt: PR landings closing each open other-task

### §3.2. P2 — v4 compiles `src/v4/compiler/*.dag` end-to-end (TASKS.md:813)

**Status**: YELLOW. Per burn-down P2 row: "Resolve-posture bridge still live; v4 compiler-of-record not proven." Blocking receipts: **compiler-of-record pipeline active** + **resolve-posture bridge (`.github/workflows/ci.yml:293-300`) deletion**.

**Bar is NOT "zero rustc errors on full-tree v4 corpus"** — that's a measurement proxy. The predicate is: v4 compiles its own `src/v4/compiler/*.dag` end-to-end via the modeled pipeline (not via the resolve-posture bridge fallback).

**Work units required**:
1. **P2-A — Compiler-of-record pipeline active end-to-end**: the v4 compiler pipeline (parse / resolve / infer / emit) processes `src/v4/compiler/*.dag` without falling back to the resolve-posture bridge.
   - Lane: Compiler Spine (smart-stag-871) + Modeling DFS (substrate support)
   - Gating: substrate landings that the compiler depends on (subset of SG fixes; specifically what the compiler subdirectory needs, not the full corpus)
   - Receipt: compiler pipeline traversal log showing no resolve-posture-bridge fallback
2. **P2-B — Resolve-posture bridge deleted from CI**: `.github/workflows/ci.yml:293-300` (the bridge that lets v4 bootstrap fall back to v2-compiled output when v4 compile fails) removed. Per INVARIANTS A3/P5 (NOT predicate P5): the bridge is a stand-in honesty signal, not a real compiler.
   - Lane: Compiler Spine (smart-stag-871) + Close/Receipt (sharp-otter-407 — adjudication)
   - Gating: P2-A (compiler-of-record working without the bridge)
   - Receipt: CI workflow with bridge code removed + green compiler-of-record run on main
3. **P2-C — P2 PROVEN ledger entry**: closure adjudication per burn-down/ledger framework.
   - Lane: Close/Receipt
   - Gating: P2-A + P2-B
   - Receipt: P2 PROVEN

**Subset Pareto closers** (advance P2 indirectly by closing errors that affect compiler-of-record bootstrap; NOT the predicate bar itself):
- SG-1 #3956 closes ~2978 E0423 (Symbol-as-callable). Subset relevant if compiler depends on Symbol surface.
- SG-5 #3957 ✓ MERGED — closes Set/collection-realization subset.
- SG-6 follow-on (gated on SG-1).

Per burn-down P2 row + #4014 evidence: "advances ci.dag authority, not corpus compile-of-record." Wave 1 SG-7 closure was substrate, not P2-direct.

### §3.3. P3 — v4 emits Rust source that compiles to a binary (TASKS.md:814)

**Status**: YELLOW. Per burn-down P3 row: "Wave 2 primary: #3964 SG-1 re-dispatch + held #3956. ~2978 E0423 class — dominant Pareto; landing by Jun 1 is possible but **emit→binary PROVEN** is not."

**Bar is specifically Rust → binary.** Per TASKS.md:814 verbatim. Python + Go emit (per SUPPORTED.md + flavor (iv)) are alpha targets, NOT part of P3 PROVEN.

**Work units required**:
1. **P3-A — SG-1 TargetAtomRealization lands**: closes ~2978 E0423 — the dominant Pareto class blocking Rust emit-to-binary.
   - Lane: Target Realization (keen-heron-687 / zesty-carp-242)
   - Gating: cursor RC clearance on #3956
   - Receipt: #3956 merged
2. **P3-B — fresh M1 probe + tail reclassification**: re-run full-tree v4 → cargo check; reclassify residuals by missing modeled fact.
   - Lane: Close/Receipt + Modeling DFS
   - Gating: P3-A
   - Receipt: updated rustc error catalog + per-class DFS worksheet routing
3. **P3-C — SG-6 BoundedLattice realization**: per merge-wave §5 W2.2.
   - Lane: Target Realization
   - Gating: P3-A
   - Receipt: SG-6 merged
4. **P3-D — each remaining error class for Rust binary closes**: per P3-B reclassification, each class blocking Rust-source-to-binary gets DFS worksheet + TR carrier + per-class PR cycle.
   - Lane: Modeling DFS + Target Realization (per-class)
   - Gating: P3-B per-class routing
   - Receipt: per-class PR cycles
5. **P3-E — emit-to-binary executes**: v4 emits Rust source → cargo builds it to an actual binary.
   - Lane: Compiler Spine + Self-host/Release
   - Gating: P3-D for all blocking classes
   - Receipt: working binary produced from v4 Rust emit
6. **P3-F — P3 PROVEN ledger entry**.
   - Lane: Close/Receipt
   - Gating: P3-E
   - Receipt: P3 PROVEN

**Alpha-target work (NOT part of P3 PROVEN, but happening in parallel)**:
- **#4040 Python emit fix** (smart-stag sub-worker): weather → Python passes `py_compile`. Tracked for SUPPORTED.md upgrade, not P3.
- **#4041 Go emit fix** (smart-stag sub-worker): weather → Go passes `go build`. Same.
- **Weather demo verification** (snappy-bee-513 subtree): Rust path verified end-to-end. Demo evidence, not P3.

### §3.4. P4 — bit-identical-self-output (T-15)

**Status**: YELLOW. Heaviest predicate; requires full v4 compile path stable.

**Work units required**:
1. **P4-A — Wave 2 closure**: SG-1 + W2.3 worker + W2.4 + W2.6c all land.
   - Multiple lanes
   - Receipt: Wave 2 ledger entry: CLOSED
2. **P4-B — compiler.dag bootstraps from .dag source**: gunbc binary compiles itself starting from `src/v4/compiler/`.
   - Lane: Compiler Spine + Self-host/Release
   - Gating: **P2 PROVEN (compiler-of-record per TASKS.md:813) + P3 PROVEN (Rust → binary per TASKS.md:814)** — actual predicate bars, NOT proxies (corrected per cursor RC#2 + operator decision D1)
   - Receipt: bootstrap-from-source receipt

[Note: W3.1 / T-24 generated ci.yml emit was previously listed here as P4-B; per operator decision D2 (00:30Z) it homes under P1 / T-24 release-authority lane unless the self-host fixed-point harness explicitly consumes generated CI. Removed from P4 dependency chain.]
3. **P4-C — fixed-point harness: two consecutive bootstrap runs produce bit-identical output**.
   - Lane: Self-host/Release (merry-badger-222 successor)
   - Gating: P4-B
   - Receipt: `docs/audit/v4-self-host-fixpoint-receipt-YYYY-MM-DD.md`
4. **P4-D — P4 PROVEN ledger entry**.
   - Lane: Close/Receipt
   - Gating: P4-C
   - Receipt: P4 PROVEN

### §3.5. P5 — TestClaim-suite-passes

**Status**: YELLOW. Per quick-tern F5 ratified: zero-Deferred verdicts on ≥1 fixture complete roster is minimum-viable.

**Work units required**:
1. **P5-A — rung4 transport ships (#4047 in flight)**: smart-stag-885 worker on `session/smart-stag-885`. `run_emit_host_rust` transport for one fixture end-to-end.
   - Lane: Runtime/TestClaim (quick-tern-735 / smart-stag-885)
   - Gating: cursor/openai-pro review on draft #4047 + duplicate-roster cleanup against landed #4046
   - Receipt: #4047 merged + nat_semiring rung4 cell flips SKIP → PASS|FAIL
2. **P5-B — nat_semiring complete claim roster zero-Deferred**: all 6 EqualsClaim + 1 DiagnosticClaim rows execute end-to-end with Verdict that is not `Deferred`.
   - Lane: Runtime/TestClaim
   - Gating: P5-A
   - Receipt: VerdictTally entry: nat_semiring 7/7 non-Deferred
3. **P5-C — fixture roster widens** (branch_dispatch, loop_linear_bound, then more): each fixture's complete roster reaches zero-Deferred.
   - Lane: Runtime/TestClaim
   - Gating: P5-B (template) + per-fixture transport availability
   - Receipt: per-fixture VerdictTally entries
4. **P5-D — F4 algebra-laws preservation post-emit (rung 6)**: per Wave F F4 ratified — nat_semiring additive-Monoid laws (assoc + identities) on Rust then Python (mul / annihilator tranche-2).
   - Lane: Modeling DFS + Runtime/TestClaim
   - Gating: P5-A (transport) + T-38 rung-4 host runner
   - Receipt: per-law preservation receipt
5. **P5-E — Structural-bridge deletion**: per burn-down P5 row, the **anti-shelfware closure** asks `scripts/v4-testclaim-corpus-gate.sh` (the structural-bridge) be **deleted** so T-22 eval is the sole authority. Not a partial-subset close.
   - Lane: Compiler Spine + Close/Receipt
   - Gating: P5-A + P5-B + P5-C + P5-D
   - Receipt: structural-bridge script removed from CI + T-22 eval is the only suite-pass authority
6. **P5-F — P5 PROVEN ledger entry**: TestClaim suite passes per TASKS.md:816 with T-22 eval as sole authority (no structural-bridge fallback).
   - Lane: Close/Receipt
   - Gating: P5-E
   - Receipt: P5 PROVEN

### §3.6. P6 — hand-Rust-not-editable-authority-proven-by-reproduction

**Status**: GRAY. Gated on P4 + P3 per burn-down + operator framework.

**Work units required**:
1. **P6-A — reproduction-from-.dag verified**: regenerate the entire compiler binary from `src/v4/compiler/*.dag` source without hand-Rust edits.
   - Lane: Self-host/Release
   - Gating: P4 PROVEN + P3 PROVEN
   - Receipt: clean-reproduction receipt
2. **P6-B — hand-Rust edit detection**: lens / discipline that fails-closed if hand-Rust authority is introduced (no hand-Rust = compiler regenerable from .dag only).
   - Lane: Modeling DFS + Self-host/Release
   - Gating: P6-A
   - Receipt: lens activation + failing-test for hand-Rust regression
3. **P6-C — P6 PROVEN ledger entry**.
   - Lane: Close/Receipt
   - Gating: P6-A + P6-B
   - Receipt: P6 PROVEN

## §4. Critical paths (parallel lanes, NOT one serial chain)

Per operator critique 00:30Z: P2, P3, and P5 are parallel where they can be. Only P4/P6 have hard gates. Revised per-lane critical paths:

```
P1 path (independent):
  P1-A roster authoring
  → P1-B per-task close adjudication
  → P1-C close-remaining-open-tasks
  → P1 PROVEN

P2 path (compiler-of-record, NOT corpus-wide):
  compiler-of-record pipeline runs on src/v4/compiler/*.dag
  → no resolve-posture bridge fallback
  → resolve-posture bridge deleted from CI
  → P2 PROVEN

P3 path (Rust → binary):
  SG-1 lands (#3956 in flight)
  → fresh M1 probe (operator rule: fresh measurement after systemic fix)
  → residual Rust emit blockers classified by Modeling DFS
  → SG-6 + remaining TR fixes (per-class cycles)
  → v4 emits Rust source
  → cargo builds emitted Rust to binary
  → P3 PROVEN

P5 path (suite passes):
  run_emit_host_rust transport ships (#4047 in flight)
  → first fixture rung4 cell SKIP → PASS|FAIL (P5-minimum-viable)
  → fixture roster widens (zero-Deferred per fixture)
  → structural-bridge deleted (T-22 sole authority)
  → P5 PROVEN (suite-wide)

P4 path (hard-gated by P2 + P3):
  P2 PROVEN ∧ P3 PROVEN
  → self-host bootstrap from .dag source
  → two-run bit-identical fixed point
  → P4 PROVEN

P6 path (hard-gated by P3 + P4):
  P3 PROVEN ∧ P4 PROVEN
  → reproduction from (.dag + frozen-pinned seed)
  → hand-Rust edit detection lens active
  → P6 PROVEN

v4-done:
  P1 ∧ P2 ∧ P3 ∧ P4 ∧ P5 ∧ P6 all PROVEN
```

**Critical path length notes**:
- P3-D (residual class closure) is the elastic stage — count depends on tail-classification after SG-1 + fresh M1 probe. Honest unknown.
- P5 path is the most decoupled — can reach P5-minimum-viable independent of P2/P3 progress.
- P4 + P6 are hard-gated; no shortcut.
- W3.1 / T-24 generated ci.yml is NOT a P4 prerequisite per operator decision D2 (homed under P1 / release-authority unless self-host harness requires it).

## §5. Parallelizable lanes (operator-revised)

All of these run in parallel; none gate each other (except where noted):

- **P1-A roster authoring**: dispatch NOW; no upstream dependency.
- **P2-A compiler-of-record probe**: dispatch NOW on src/v4/compiler/*.dag scope.
- **P3 SG-1 + M1 probe + tail** (P3-A through P3-E): single-thread critical path within P3 lane (sequential), but parallel to P2 + P5.
- **P5 rung4 transport + fixture widening** (P5-A through P5-D): parallel to P2 + P3 entirely.
- **W2.3 Buckets B/C/D**: parallel-OK with disjoint CiStepId rosters after Bucket A #4055 lands (per proud-pike msg_1d95ba51); partition table dispatched 23:59Z.
- **Alpha-target work** (#4040 Python emit, #4041 Go emit, weather demo verification): parallel to predicate lanes but **NOT part of any v4-done predicate**. Tracked separately per SUPPORTED.md + flavor (iv).
- **Wave 3 framing (Wave F)**: operator-ratified 22:55Z; W3.1 (P1/T-24), W3.2 (CI active-skip), W3.3 (cross-target), W3.4 (preservation laws), W3.5 (self-emit fixpoint = P4 work), W3.6 (corpus exec = P5 work), W3.7 (lenses).

## §6. Realistic work-units estimate (no time projection per CODING.md / TASKS.md "no timelines")

| Stream | Open work-units | In-flight | Done |
|---|---|---|---|
| Wave 2 closure (substrate) | 1 (W2.3 worker landing) | deep-boar-656 | SG-2 + SG-5 + SG-7 + Upsert + CiUpsertStep + W2.5 + W2.6a + W2.6b-closure |
| P2 lane (compiler-of-record + bridge deletion, P2-A through P2-C) | 2 + ledger | P2-A probe dispatched 23:59Z (smart-stag-871) | — |
| P3 lane (SG-1 → M1 probe → SG-6 → per-class → emit-binary, P3-A through P3-F) | elastic (per-class count post-M1) | zesty-carp-242 on #3956 SG-1 cursor RC | substrate above |
| Alpha-target Python + Go emit (NOT P3) | 2 + verification | smart-stag-871 sub-workers (#4040, #4041) | — |
| P5 lane — minimum-viable (rung4 transport + first PASS\|FAIL) | 1 + roster validation | smart-stag-885 (#4047) | runtime-value rosters #4046 |
| P5 lane — PROVEN (fixture widening + structural-bridge deletion) | substantial; needs P5-minimum-viable first | (sequenced after) | — |
| P4 lane (P4-A Wave 2 close → P4-B bootstrap → P4-C fixpoint) | substantial; needs P2 + P3 PROVEN | (hard-gated) | — |
| P1 roster + per-task close (P1-A through P1-C) | bounded after P1-A roster | P1-A dispatched 23:59Z (sharp-otter-407) | — |
| P6 reproduction + hand-Rust detection | 2 + lens activation | (hard-gated on P4 + P3) | — |
| Wave 3 W3.1 / T-24 ci.yml emit (P1-homed, not P4) | substantial; needs Wave 2 closure | (positioned post-Wave-2) | — |

## §7. PM-side immediately-dispatchable items (not waiting on anything)

Items that can be dispatched NOW that are NOT on any in-flight worker's lane:

1. **P1-A every-other-task roster** — Close/Receipt (sharp-otter-407) audit of TASKS.md for non-six-predicate task list. Dispatching this kicks off P1 measurement, which has not been started.
2. **W2.3 Bucket B/C/D parallel dispatch** — after #4055 Bucket A lands (per proud-pike adjudication), spawn 3 sibling workers with disjoint CiStepId rosters.

Everything else is in flight OR genuinely gated by upstream substrate landing.

## §8. Cross-refs

- Authoritative predicate state: `docs/planning/v4-done-predicate-burn-down-2026-05-30.md` (merry-badger-222)
- Authoritative MW-D8 ledger: `docs/planning/v4-mw-d8-wave1-exit-ledger-2026-05-30.md` (sharp-otter-407)
- Merge-wave plan: `docs/planning/v4-merge-wave-and-next-waves-2026-05-30.md` (#3983)
- Correctness ladder + manager lanes: PR #3938
- Wave 3 framing decisions (Wave F operator-ratified): see PM relay messages 22:50–22:55Z + manager state reports
- v0.1.0 release posture (flavor iv): `docs/RELEASE_v0.1.0.md` (#3991) + `docs/SUPPORTED.md` (#4025)

## §9. Honest framing

- **No time estimate**. Per TASKS.md discipline (no day-estimates) and CODING.md, this doc names work-units and dependencies. Operator + manager bandwidth + external review cadence + substrate complexity all combine to actual elapsed time.
- **P3-D is the elastic core**. After SG-1 lands and the M1 probe reclassifies the tail, the per-class work-unit count for the remaining Rust-emit blockers becomes measurable. Until then, that fraction of the graph is "many iterations of unknown count."
- **P4-B is gated on substrate-PROVEN bars** (compiler-of-record + Rust→binary). Compiler self-bootstrap is meaningful only after P2 + P3 PROVEN — there is no shortcut.
- **P5-minimum-viable is the most-decoupled milestone**. P5-minimum-viable (one fixture zero-Deferred) can land well before the P2/P3/P4 chain closes because it operates on a single landed fixture's emitted code. **P5 PROVEN itself is suite-wide + structural-bridge-deleted** — that follows after fixture widening + bridge deletion, not at P5-minimum-viable.
- **Release-state framing** (per TASKS.md no-timeline discipline, no date claim made here): per flavor (iv), v0.1.0 ships with predicates at whatever state they hold; honest alpha labeling absorbs the gap. The path to 6/6 is the predicate dependency graph above, not the release calendar.

## §10. Open decisions (operator-flagged 00:30Z)

| D | Decision | PM recommendation | Status |
|---|---|---|---|
| D1 | Is P2 scoped to compiler/*.dag only, or full v4 corpus? | compiler/*.dag only (full-tree rustc is P3 evidence, not P2 bar) | Adopted in doc |
| D2 | Is generated ci.yml part of P4 or P1/T-24? | P1/T-24 (not P4) unless self-host harness explicitly consumes generated CI | Adopted in doc |
| D3 | Does P5 include rung 6 algebra-law preservation? | Only if part of authoritative TestClaim suite roster; else rung-6 not P5 | Reflected in P5-D conditional |
| D4 | What is the first P5 minimum? | P5-minimum-viable = one fixture zero-Deferred; P5-PROVEN = full suite + structural-bridge deleted | Split in §3.5 dependency-list |
| D5 | Does P1-A have authority to mark items out-of-scope? | P1-A proposes; operator/Close-Receipt ratifies. P1-A cannot expand v4 scope without operator decision. | Reflected in dispatch brief (msg_3df7dcdb) |

## §11. Worker brief template (per operator critique 00:30Z)

When dispatching a worker for any item above, the brief should use this shape:

```
predicate:               # which v4-done predicate this advances (or "none / alpha-target")
work-unit:               # which P*-* sub-item
blocking_receipt_to_flip: # what observable state changes when this lands
upstream_dependencies:   # what must already be PROVEN / landed
must_not_change:         # invariant that the work must not violate
fresh_measurement:       # what re-measurement follows landing (if any)
```

Examples (each maps to an in-flight dispatch):

```
predicate: P3
work-unit: SG-1 TargetAtomRealization (P3-A)
blocking_receipt_to_flip: Symbol/Atom realization no longer emitted as callable Symbol(...)
upstream_dependencies: substrate already on main (SG-2 + SG-5 + Upsert<T> + CiUpsertStep)
must_not_change: do not relax MW-D8 / change predicate bars
fresh_measurement: M1 rustc catalog after merge (P3-B)
```

```
predicate: P5 (minimum-viable)
work-unit: run_emit_host_rust transport (P5-A)
blocking_receipt_to_flip: rung4 fixture Rust cell SKIP → PASS|FAIL
upstream_dependencies: #4046 runtime-value rosters (LANDED ✓)
must_not_change: do not broaden TestClaim suite definition; T-22 eval contract intact
fresh_measurement: first VerdictTally entry for fixture
```

```
predicate: MW-D8 C4 / CI dependency management
work-unit: ci_selection_receipt_shadow (W1.5)
blocking_receipt_to_flip: C4 GAP → PROVEN
upstream_dependencies: Upsert<T> + CiUpsertStep substrate on main
must_not_change: no active skip, no heuristic run-modes (operator no-overclaim policy)
fresh_measurement: receipt-on-PR demo run
```

```
predicate: P2
work-unit: compiler-of-record probe on src/v4/compiler/*.dag (P2-A)
blocking_receipt_to_flip: pipeline traversal log shows no resolve-posture-bridge fallback
upstream_dependencies: compiler-subset substrate (NOT full-corpus)
must_not_change: do NOT use rustc-count as P2 closure proxy
fresh_measurement: named-blocker list if pipeline fails (concrete dispatch targets)
```
