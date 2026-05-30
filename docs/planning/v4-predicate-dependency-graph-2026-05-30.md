# v4 predicate dependency graph — from 0/6 to 6/6 PROVEN

PM-authored forward-projection. Maps the labour-graph from current state (0/6 v4-done predicates PROVEN per `docs/planning/v4-done-predicate-burn-down-2026-05-30.md`) to all six PROVEN.

**State-of-truth deferral**: this doc forward-projects labour and dependencies. It does NOT re-author predicate state. The authoritative predicate-state source is merry-badger-222's `docs/planning/v4-done-predicate-burn-down-2026-05-30.md`; the authoritative MW-D8 source is sharp-otter-407's `docs/planning/v4-mw-d8-wave1-exit-ledger-2026-05-30.md`.

**Scope**: not a schedule, not a timeline. Each work-unit is named by the receipt it produces, the gating dependencies, and the responsible manager lane.

## §1. The six predicates (TASKS.md:801–815)

Per PR #3938 §8 D4: v4-done = ALL six PROVEN collectively. Any single predicate at non-PROVEN blocks v4-done close.

| # | Predicate | What "PROVEN" requires |
|---|---|---|
| P1 | every-other-task | Each TASKS.md "other" task either PROVEN, named-blocked, or explicitly out-of-scope; no hidden incomplete tasks |
| P2 | corpus-compiles | Full-tree `src/v4` emit → `cargo check` clean (zero rustc errors on emitted Rust over full v4 corpus) |
| P3 | emit-compiles | Each supported emit target (Rust + Python + Go per D-REL-3) → target-toolchain check clean for the documented corpus |
| P4 | bit-identical-self-output | T-15: compiler.dag compiles itself to byte-identical output across two consecutive runs (fixed-point) |
| P5 | TestClaim-suite-passes | Per quick-tern F5 ratified 22:54Z: ≥1 fixture's complete claim roster executes T-22 → TestClaimRun → CorpusEvalReport → VerdictTally with zero `Deferred` verdicts. Widen fixture-by-fixture until full corpus pass. |
| P6 | hand-Rust-not-editable-authority-proven-by-reproduction | Reproduction-from-`.dag` regenerates the compiler without hand-Rust edits; bit-identical to current. Gated on P4 + P3. |

## §2. Dependency graph

ASCII graph reading bottom-up (leaves are merged-substrate; root is `v4-done = 6/6 PROVEN`):

```
                           v4-done = 6/6 PROVEN
                                  ↑
            ┌─────────┬──────────┼──────────┬──────────┐
            │         │          │          │          │
            P1        P2         P3         P4         P5        P6
   every-other-task corpus-   emit-      bit-id     TestClaim   hand-Rust
                    compiles  compiles   self-output  passes    reproduction
                                                                (gated P4+P3)
            │         │          │          │          │          │
            │         │          │          │          │          │
            │         │          │       ┌──┴──┐       │      ┌───┴───┐
            │         │          │       │     │       │      │       │
            │         │     ┌────┴────┐  │  P2 │       │      P4      P3
            │         │     │         │  │  P3 │       │   PROVEN  PROVEN
            │         │     │   Rust  │  │PROVEN│      │      ↑       ↑
            │         │     │ emit OK │  └──┬──┘       │  [see P4]  [see P3]
            │         │     │         │     │          │
            │         │     │   ┌─────┴───┐ │          │   ┌──────────────┐
            │         │     │   │ Python  │ │      ┌───┴───┴┐             │
            │         │     │   │ emit OK │ │      │ rung4-9│             │
            │         │     │   └────┬────┘ │      │ fixtures│             │
            │         │     │        │      │      └────┬───┘             │
            │         │     │        │      │           │                 │
            │         │     │   ┌────┴───┐  │      ┌────┴────┐            │
            │         │     │   │ Go     │  │      │ rung4   │            │
            │         │     │   │ emit OK│  │      │ first   │            │
            │         │     │   └────┬───┘  │      │ PASS|FAIL│           │
            │         │     │        │      │      └────┬────┘            │
            │         │     │        │      │           │                 │
            │         │  ┌──┴────────┴──────┴┐     ┌────┴────────────┐    │
            │         │  │ All SG (Symbol/   │     │ run_emit_host   │    │
            │         │  │ TypeExpr/Coll/    │     │ transport       │    │
            │         │  │ Lattice/etc.)     │     │ (#4047 in flight)│   │
            │         │  │ TR carriers       │     └────┬────────────┘    │
            │         │  │ landed            │          │                 │
            │         │  └──┬────────────────┘     ┌────┴────────────┐    │
            │         │     │                      │ runtime-value   │    │
            │         │     │                      │ rosters (#4046  │    │
            │         │     │                      │ MERGED ✓)       │    │
            │         │  ┌──┴───────┐              └─────────────────┘    │
            │         │  │ SG-1 #3956│                                    │
            │         │  │ (in flight)│                                   │
            │         │  └──┬───────┘                                    │
            │         │     │                                            │
            │         │  ┌──┴────────────────┐                           │
            │         │  │ SG-5 ✓ + SG-2 ✓   │                           │
            │         │  │ + Upsert<T> ✓ +   │                           │
            │         │  │ CiUpsertStep ✓ +  │                           │
            │         │  │ SG-7 dissolved ✓  │                           │
            │         │  └───────────────────┘                           │
            │         │                                                  │
        ┌───┴───┐ ┌───┴────────────┐                                     │
        │"every"│ │ fresh M1 probe │                                     │
        │ defn  │ │ + reclassify   │                                     │
        └───────┘ │ tail by missing│                                     │
                  │ modeled fact   │                                     │
                  └────────────────┘                                     │
                                                                         │
                                              ┌──────────────────────────┘
                                              │ Wave 3 closure: W3.1
                                              │ (T-24 ci.yml emit) +
                                              │ W3.5 (self-emit fixpoint
                                              │ rung 7 / T-15 close)
                                              └──────────────────────────
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

### §3.2. P2 — corpus-compiles (rustc on full-tree v4 emit)

**Status**: YELLOW. Current baseline ~7,951 rustc errors per `docs/audit/v4-rustc-error-catalog-2026-05-29.md`. Critical path Pareto: E0423 (~2,978) closed by SG-1 #3956.

**Work units required**:
1. **P2-A — SG-1 TargetAtomRealization lands**: closes ~2,978 E0423 (Symbol-as-callable). 
   - Lane: Target Realization (keen-heron / zesty-carp-242)
   - Gating: cursor RC clearance on #3956
   - Receipt: #3956 merged + sub-class closure
2. **P2-B — fresh M1 probe + tail reclassification**: re-run full-tree v4 → cargo check; reclassify residuals by missing modeled fact (not by error count).
   - Lane: Close/Receipt + Modeling DFS
   - Gating: P2-A landed
   - Receipt: updated `docs/audit/v4-rustc-error-catalog-YYYY-MM-DD.md` + per-class DFS worksheet routing
3. **P2-C — SG-6 BoundedLattice realization**: per merge-wave §5 W2.2 (gated on SG-1).
   - Lane: Target Realization
   - Gating: P2-A landed
   - Receipt: SG-6 PR merged
4. **P2-D — each remaining error class closes**: per P2-B reclassification, each class gets a DFS worksheet (Modeling DFS lane) + Target Realization carrier + per-class PR cycle.
   - Lane: Modeling DFS + Target Realization (per-class)
   - Gating: P2-B reclassification per class
   - Receipt: per-class PR cycles; each closes a measurable rustc subset
5. **P2-E — full-tree cargo check ZERO errors**: final receipt; flips P2 PROVEN.
   - Lane: Close/Receipt (adjudication)
   - Gating: all P2-D classes closed
   - Receipt: ledger entry: P2 PROVEN

### §3.3. P3 — emit-compiles (per target)

**Status**: YELLOW. Rust subset of P2; Python (#4040 in flight) + Go (#4041 in flight) emit fixes for weather demo.

**Work units required**:
1. **P3-A — weather.dag emit-to-Rust verified end-to-end**: clean checkout → cargo build gunbc → gunbc compile weather → cargo check + cargo run.
   - Lane: snappy-bee-513 (release subtree) — verification worker already dispatched
   - Gating: gunbc binary compiles + weather emits clean (current state)
   - Receipt: verification log on `docs/release/weather-e2e-receipt.md`
2. **P3-B — Python emit fix lands (#4040)**: smart-stag sub-worker on `session/emit-python-tco-fix`. Match/case arms + TCO temp-decl.
   - Lane: Compiler Spine (smart-stag-871 / sub-worker)
   - Gating: worker iteration on draft #4040
   - Receipt: #4040 merged + weather → Python passes `py_compile`
3. **P3-C — Go emit fix lands (#4041)**: smart-stag sub-worker on `session/emit-go-layout-fix`. Multi-file layout + `:=` scope.
   - Lane: Compiler Spine
   - Gating: worker iteration on draft #4041
   - Receipt: #4041 merged + weather → Go passes `go build`
4. **P3-D — per-target test suite pass**: every documented example for each of Rust + Python + Go runs target-toolchain check clean.
   - Lane: Runtime/TestClaim (quick-tern-735) + Close/Receipt
   - Gating: P3-A + P3-B + P3-C
   - Receipt: per-target test ledger entries
5. **P3-E — P3 PROVEN ledger entry**: closure adjudication.
   - Lane: Close/Receipt
   - Gating: P3-D all-targets-green
   - Receipt: P3 PROVEN

**P3 also depends on P2 for the Rust path** (full-tree v4 emit Rust = subset of P3-Rust). If P2 lands, P3-Rust comes for free.

### §3.4. P4 — bit-identical-self-output (T-15)

**Status**: YELLOW. Heaviest predicate; requires full v4 compile path stable.

**Work units required**:
1. **P4-A — Wave 2 closure**: SG-1 + W2.3 worker + W2.4 + W2.6c all land.
   - Multiple lanes
   - Receipt: Wave 2 ledger entry: CLOSED
2. **P4-B — Wave 3 W3.1 land (T-24 ci.yml emit substrate)**: per Wave F F1 ratified. `.github/workflows/ci.yml` emitted SOLELY from `ci_pipeline`; hand YAML deleted.
   - Lane: Compiler Spine + Modeling DFS (joint)
   - Gating: P4-A
   - Receipt: ci.yml byte-identical to YamlStatic emission from ci_pipeline
3. **P4-C — compiler.dag bootstraps from .dag source**: gunbc binary compiles itself starting from `src/v4/compiler/`.
   - Lane: Compiler Spine + Self-host/Release
   - Gating: P3 (Rust emit-compiles for full v4 corpus) + P2 (zero rustc errors)
   - Receipt: bootstrap-from-source receipt
4. **P4-D — fixed-point harness: two consecutive bootstrap runs produce bit-identical output**.
   - Lane: Self-host/Release (merry-badger-222 successor)
   - Gating: P4-C
   - Receipt: `docs/audit/v4-self-host-fixpoint-receipt-YYYY-MM-DD.md`
5. **P4-E — P4 PROVEN ledger entry**.
   - Lane: Close/Receipt
   - Gating: P4-D
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
5. **P5-E — P5 PROVEN ledger entry**: full corpus VerdictTally zero-Deferred OR explicit subset proven + named-out-of-scope.
   - Lane: Close/Receipt
   - Gating: P5-C + P5-D
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

## §4. Critical path

The longest single dependency chain to 6/6:

```
SG-1 #3956 lands
  → fresh M1 probe (P2-B)
  → per-class DFS worksheets + TR carriers (P2-D, many iterations)
  → P2 zero-error full-tree corpus
  → Rust emit clean (P3-Rust subset of P2)
  → Python + Go emit fixes (P3-B + P3-C, parallel)
  → P3 PROVEN
  → Wave 2 + Wave 3 W3.1 (T-24 ci.yml emit) land
  → compiler.dag bootstraps from .dag (P4-C)
  → fixed-point harness (P4-D)
  → P4 PROVEN
  → reproduction-from-.dag (P6-A)
  → hand-Rust edit detection (P6-B)
  → P6 PROVEN
```

**Critical path length**: P2-D is the elastic stage — the number of iterations depends on tail-classification of remaining errors after SG-1 + SG-5 lands. Honest read: P2 closure is the longest unknown.

## §5. Parallelizable lanes

The graph above is NOT entirely serial. These streams run in parallel with the critical path:

- **P5 (TestClaim suite)**: parallel from rung4 transport onwards. P5-A through P5-D do NOT block on P2/P3 closure (uses already-emitted Rust for nat_semiring rung4 fixture). P5 could PROVEN before P4 or P3 if the corpus is widened.
- **P3 Python + Go fixes (P3-B / P3-C)**: parallel to P2 Rust closure. Each emit-fix worker is independent.
- **W2.3 Buckets B/C/D**: parallel after bucket A lands (per proud-pike's adjudication msg_1d95ba51).
- **P1-A roster authoring**: can dispatch now, parallel to everything else.
- **Wave 3 framing decisions (Wave F)**: already operator-ratified.

## §6. Realistic work-units estimate (no time projection per CODING.md / TASKS.md "no timelines")

| Stream | Open work-units | In-flight | Done |
|---|---|---|---|
| Wave 2 closure (substrate) | 1 (W2.3 worker landing) | deep-boar-656 | SG-2 + SG-5 + SG-7 + Upsert + CiUpsertStep + W2.5 + W2.6a + W2.6b-closure |
| SG-1 #3956 + tail (P2-A through P2-E) | many (post-SG-1 reclassification dependent) | zesty-carp-242 cursor RC | substrate landings above |
| Python + Go emit (P3-B + P3-C) | 2 + verification | smart-stag-871 sub-workers (#4040, #4041) | — |
| Rung4 transport + first PASS\|FAIL (P5-A + P5-B) | 1 + roster validation | smart-stag-885 (#4047) | runtime-value rosters #4046 |
| Wave 3 W3.1 (P4-B) | substantial; needs Wave 2 closure first | (gated) | — |
| Self-host fixed-point (P4-C, P4-D) | substantial; needs P2 + P3 PROVEN | (gated) | — |
| P1 roster + per-task close | unknown until roster authored | (P1-A not yet dispatched) | — |
| P6 reproduction + hand-Rust detection | 2 + lens activation | (gated on P4 + P3) | — |

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
- **P2-D is the elastic core**. After SG-1 lands and the M1 probe reclassifies the tail, the work-unit count for the remaining error classes becomes measurable. Until then, that fraction of the graph is "many iterations of unknown count."
- **P4-C is gated on substrate**. Compiler self-bootstrap is meaningful only after P2 + P3 cargo-clean — there is no shortcut.
- **P5 is the most-decoupled lane**. It can PROVEN well before the P2/P3/P4 chain closes because it operates on a single landed fixture's emitted code.
- **0/6 by Jun 1**. Per flavor (iv) honest alpha framing, v0.1.0 ships with all predicates YELLOW or GRAY. The path to 6/6 is post-launch.
