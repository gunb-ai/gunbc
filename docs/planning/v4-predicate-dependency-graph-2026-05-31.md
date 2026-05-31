# v4 predicate dependency graph — 2026-05-31T06:30Z (updated from #4058)

Forward projection of remaining work to v4-done. Supersedes the 2026-05-30 snapshot (#4058) for current state; #4058 retained for historical baseline. Authoritative predicate definitions at `src/v4/TASKS.md:806-817`.

## §1. Six v4-done predicates — current state

| # | Predicate (TASKS.md:806-817) | State |
|---|---|---|
| P1 | Every other scheduled task complete | YELLOW (8/53 PROVEN per sharp-otter #4065 roster + #4060 distribution; 45 GAP items remain) |
| P2 | v4 compiles `src/v4/compiler/*.dag` end-to-end | **TECHNICAL-PROVEN / AUTHORITY-BLOCKED**: scope (a) PROVEN per valiant-moth-559 probe (44-source compiler closure, 0 dag+rust diagnostics, no resolve-posture bridge needed). Full predicate gated on P2-B bridge deletion (`ci.yml:378-385` + `scripts/v4-bootstrap-resolve-posture-gate.sh`) — operator-authorization-blocked. |
| P3 | v4 emits Rust source that compiles to a binary | YELLOW. Post-SG-1 #3956 measurement: 6991 rustc errors (was 7951; SG-1 dissolved E0423 class -2978; SG-7 cleared). 8 active receipt-producing classes routed per #4086 catalog. |
| P4 | Binary on `src/v4/compiler/*.dag` produces bit-identical output | RED — hard-gated on P2 full deletion + P3 PROVEN |
| P5 | TestClaim suite passes | YELLOW. **P5-minimum-viable LANDED** (#4063 + #4064). **P5-PROVEN gates split into two bundles. Layer 1 fixture/law bundle: 3/3 CLOSED** (Wa-1 #4079 + Wa-2 #4080 + P5-D tranche-2 #4089). **Layer 2 authority gate: OPEN** (structural-bridge `scripts/v4-testclaim-corpus-gate.sh` replacement+deletion). **P5 PROVEN requires BOTH layers closed.** Don't read "Wa-2 LANDED → P5 GREEN". |
| P6 | Hand-authored Rust not editable authority (proven by REPRODUCTION) | RED — hard-gated on P4 + P3 PROVEN |

## §2. What closed since #4058 (2026-05-30T00:48Z baseline)

Receipt-producing landings (NOT activity counting — these flipped or partial-flipped a predicate or exit gate):

- **SG-1 #3956** — dominant Pareto closer; -2978 E0423 errors
- **#4086 sharp-otter post-SG-1 catalog** — P3-D elastic core has concrete count: 8 classes routed (see §3.3)
- **W2.3 Bucket E #4078** — 5 GateStep CiUpsertStep rows; full `ci_pipeline_step_ids_shadow` bijection; W2.3 sequence A+B+C+E complete
- **W1.5 shadow receipt #4073 + #4082** — MW-D8 condition C4 PROVEN → Wave 1 EXIT achieved
- **Wc cross-target #4081** — L5 cross-target equivalence gate first-fire (18/18 rows Rust+Python+Go on nat_semiring)
- **Wa-1 #4079** — branch_dispatch complete-roster rung-8; second fixture flips zero-Deferred
- **P5-D tranche-2 #4089** — nat_semiring semiring multiplicative + annihilator (full ring-structure × all 3 targets)
- **P2-A probe** (valiant-moth-559) — compiler closure verifies P2 scope (a)
- **#4076 Go emit fix** — alpha-target unblock (v0.1.0 release scope)
- **#4074 + #4092 CI runtime drops** — ~14min CI runtime savings landed (operator CI-overhaul gap actively closing)

## §3. Per-predicate remaining work — dependency list

### §3.1 P1 every-other-task complete (8/53 PROVEN)

Per sharp-otter #4060 roster + per-lane close work. NOT a critical path for v0.1.0 release; spans full plan minus T-15. Per-lane workers maintain (deep-ferret rebase wave, T-22 etc.).

### §3.2 P2 — TECHNICAL-PROVEN / AUTHORITY-BLOCKED

**Layer 1 — Technical proof (DONE):** v4 compiler closure (44 sources) compiles end-to-end with 0 dag+rust diagnostics via P2-A probe (valiant-moth-559). No resolve-posture bridge invoked.

**Layer 2 — Authority deletion (OPEN, operator-decision-node):**
- **P2-B**: delete `ci.yml:378-385` resolve-posture bridge + `scripts/v4-bootstrap-resolve-posture-gate.sh`. Decision-node owner: operator/PM. Output: authorize deletion OR explicitly keep bridge with named follow-up receipt. Smart-stag flagged 2026-05-30; not authorized. **Blocks: P2 full PROVEN flip; P4 cascade.**

### §3.3 P3 Rust source → binary — 8 active receipt-producing classes

Per #4086 catalog + proud-pike routing (msg_6db2dc9e):

| Class | ~Errors | State | Routing |
|---|---|---|---|
| SG-1b function-signature String↔Symbol | (subset of 6991) | worksheet-needed | NEW sibling worksheet — TR lane (keen-heron) implements |
| SG-RC-LAYERING | ~700 (10%) | authoring | NEW §10.0 worksheet — **proud-pike authoring** |
| SG-2 | (extend existing) | routed | proud-pike's SG-2 worksheet; worker dispatch when ready |
| SG-8 | (extend existing) | routed | extend existing worksheet |
| SG-3-CASCADE | (extend existing) | routed | extend existing worksheet |
| SG-5 | (extend existing) | routed | extend existing worksheet |
| SG-6 | (extend existing) | in-review | #4085 SG-6 in cursor RC cycle (lively-eagle) |
| SG-1-FOLLOWON | (extend existing) | routed | worker-only follow-on |
| SG-COLLECTION-PROJECTION | ~170 | route-pending | amend SG-5/SG-6 first; escalate to NEW only if single-authority-fact fails |

(Table lists 9 routing rows for 8 distinct receipt-producing classes — SG-1b + SG-1-FOLLOWON are two routing paths for one class per #4086 catalog. `state` legend: `worksheet-needed` = no §10.0 yet; `authoring` = §10.0 being written; `routed` = §10.0 exists, worker dispatchable; `in-review` = worker PR open; `route-pending` = decision needed on extend-vs-new.)

P3 PROVEN bar: **Rust source compiles to a binary** (not zero rustc errors — that is a measurement proxy; not Python+Go either — those are v0.1.0 alpha scope per Wave F F3). Each named class closure shrinks the rustc error count; when residual errors → 0 on the Rust compile path, the binary builds.

### §3.4 P4 bit-identical fixpoint

Hard-gated on P2 bridge deletion + P3 binary build. No active worker dispatchable until those land.

### §3.5 P5 TestClaim suite passes — two-layer gate split

**Layer 1 — Fixture/law bundle (technical proofs): 3/3 CLOSED ✓**
- Fixture-roster widening axis 1: Wa-1 #4079 branch_dispatch complete-roster LANDED
- Fixture-roster widening axis 2: **Wa-2 #4080 loop_linear_bound complete-roster LANDED dca5ce7a** (zero-Deferred now on three fixtures: nat_semiring + branch_dispatch + loop_linear_bound)
- P5-D tranche-2: nat_semiring multiplicative + annihilator LANDED #4089

**Layer 2 — Authority gate (substrate deletion): OPEN**
- **Structural-bridge deletion**: `scripts/v4-testclaim-corpus-gate.sh` removal. Per quick-tern's no-new-shell read, this needs a **positive-Y modeled CI-step authoring** (likely a modeled `ci.dag` `CiUpsertStep` row that replaces the shell call) before dispatch. **Not currently spawned.** The shape of the replacement must align with the elastic CI design currently being fleshed out in [#4091](https://github.com/gunb-ai/gunbc/pull/4091) (`docs/planning/elastic-ci-redesign-exploration-2026-05-31.md`) — specifically §3 Upsert<T> substrate state + the four-compile-redundancy reduction that informs where T-22 corpus gate steps belong in the redesigned graph.

**P5 PROVEN requires BOTH layers closed.** Layer 1 done; Layer 2 still open. Reading "Wa-2 LANDED → P5 GREEN" is a category error.

### §3.6 P6 hand-Rust REPRODUCTION

Hard-gated on P4 + P3 PROVEN. No worker until those land.

## §4. Critical paths forward

**Path A: P5 final-flip (shortest)**
1. ✓ Wa-2 #4080 LANDED (fixture/law bundle 3/3 complete)
2. Structural-bridge replacement authored (positive-Y CiUpsertStep in ci.dag, citing #4091 §3) → bridge deletion → P5-PROVEN flips GREEN

**Path B: P2 bridge deletion (operator-gated)**
1. Operator authorizes ci.yml:378-385 + scripts/v4-bootstrap-resolve-posture-gate.sh deletion
2. Smart-stag implements
3. P2 PROVEN flips GREEN

**Path C: P3 binary build (longest, parallel)**
1. proud-pike authors SG-RC-LAYERING §10.0 worksheet
2. keen-heron dispatches SG-1b sibling worker (TR lane)
3. Workers extend SG-2/SG-3/SG-5/SG-6/SG-8/SG-1-FOLLOWON in parallel
4. Each class closure shrinks rustc residual; binary builds when residual hits 0
5. P3 PROVEN flips GREEN

**Path D: P1 P4 P6 cascade**
1. P1: continue per-lane close work; not blocking v0.1.0 release per operator framework
2. P4 + P6: gate on P2 + P3 PROVEN above

## §5. Parallelizable NOW (no dispatch blockers)

| Worker | Work |
|---|---|
| valiant-cat-623 | Wa-2 #4080 LANDED dca5ce7a; worker archived |
| lively-eagle-677 | SG-6 #4085 — address cursor RC |
| proud-pike-680 | Author SG-RC-LAYERING §10.0 worksheet |
| keen-heron-687 | SG-1b sibling worksheet relay → TR worker dispatch when proud-pike ratifies |
| sharp-otter-407 | Standing by post-routing |
| smart-stag-871 | Standing by; P2-B bridge deletion blocked-on-decision |
| quick-tern-735 | Tranche-3 dispatch when Wa-2 lands; structural-bridge replacement pending positive-Y authoring |
| merry-badger-222 | Post-cascade burn-down refresh |
| deep-ferret-235 | CI rebase wave continuing |

## §6. Active blockers — what's stuck

| Blocker | Owner action |
|---|---|
| **P2-B bridge deletion authorization** | Operator decision. Smart-stag flagged 2026-05-30; bridge at `ci.yml:378-385` + `scripts/v4-bootstrap-resolve-posture-gate.sh`. |
| **Structural-bridge replacement** (P5 final gate) | Modeling DFS — positive-Y `CiUpsertStep` in `ci.dag` to replace `scripts/v4-testclaim-corpus-gate.sh` shell call. Not currently spawned. Authoring should cite [#4091](https://github.com/gunb-ai/gunbc/pull/4091) §3 substrate state to avoid drifting from the in-flight elastic CI design. |
| **SG-RC-LAYERING worksheet** | proud-pike — authoring next; downstream worker auto-spawns once §10.0 published |
| **#4085 SG-6 cursor RC** | lively-eagle authoring fix |

## §6.5 Current Dispatch Board

| Priority | Item | State | Owner | Exit receipt |
|---|---|---|---|---|
| 1 | Wa-2 #4080 | LANDED dca5ce7a ✓ | valiant-cat (archived) | fixture/law bundle 3/3 closed |
| 2 | Structural-bridge replacement DFS | **unstaffed** | Modeling DFS (proud-pike) | §10.0 worksheet for positive-Y CiUpsertStep replacing scripts/v4-testclaim-corpus-gate.sh |
| 3 | P2-B deletion authorization | **operator-gated decision-node** | operator/PM | bridge deletion authorized OR explicitly kept with named follow-up |
| 4 | SG-RC-LAYERING worksheet | authoring | proud-pike | §10.0 worksheet published |
| 5 | SG-6 #4085 | cursor RC | lively-eagle | class merged |
| 6 | SG-1b sibling | routed | Target Realization (keen-heron) | worker dispatched |
| 7 | #4091 elastic CI ratification | draft / exploration | operator | ratification posture decided |

## §7. PM-side actionable items

**No implementation dispatches possible until two authoring blockers clear:**
1. SG-RC-LAYERING §10.0 worksheet publishes (proud-pike) → downstream P3 worker auto-dispatchable
2. Structural-bridge replacement DFS worksheet publishes (Modeling DFS, **currently unstaffed**) → downstream P5 worker dispatchable

**Both authoring items ARE active blockers — monitor and escalate if no movement.**

**Operator-gated decisions also blocking dispatch:**
- P2-B bridge deletion authorization (without it P2 stays AUTHORITY-BLOCKED → P4 + P6 cascade)
- #4091 elastic CI ratification posture (without it structural-bridge replacement substrate framing is uncertain → risks re-shape post-ratification)

## §8. Risk / honesty section

- **P5 Layer 1 fixture/law bundle is 3/3 done** (Wa-1 + Wa-2 + P5-D tranche-2). Layer 2 substrate gate is the live risk now.
- **Structural-bridge replacement is unstaffed.** Per quick-tern's no-new-shell-substrate read, it needs DFS authoring before worker dispatch. This is the latent gate that could surprise — and per operator framing it should be the **first concrete CiUpsertStep replacement under #4091** rather than a one-off shell swap.
- **P2-B bridge deletion is operator-gated and not authorized.** Without it, P2 stays YELLOW even though scope (a) is PROVEN.
- **#4092 saved ~14m CI runtime** but ci_v4 step still sometimes times out (#4074 rerun was a 32min nailbiter); CI overhaul gap is partially closing, not fully. The full elastic redesign is being explored in [#4091](https://github.com/gunb-ai/gunbc/pull/4091) (operator-authored, draft), anchored on the same #4074 run profile that surfaced the four-compile-redundancy (M1 rust emit + v2→v4 bootstrap + T-22 corpus rust + T-22 corpus dag, all on the same 332-source closure). Until that lands, partial wins like #4092 + #4074 are the per-step amortizations; structural overhaul awaits ratification.
- **6991 rustc errors → 0 is a long-horizon path.** Per-class closures (SG-RC-LAYERING ~700, SG-COLLECTION-PROJECTION ~170, etc.) accumulate — the visible single-class wins from SG-1 (~37%) won't recur at that magnitude.

## §9. Cross-refs (authoritative state)

- merry-badger burn-down: `docs/planning/v4-done-predicate-burn-down-2026-05-30.md`
- sharp-otter MW-D8 ledger: `docs/planning/v4-mw-d8-wave1-exit-ledger-2026-05-30.md`
- sharp-otter P1 roster: `docs/planning/v4-p1-other-task-roster-2026-05-30.md`
- sharp-otter post-SG-1 catalog: `docs/audit/v4-rustc-error-catalog-2026-05-31.md` (PR #4086)
- prior dep graph (baseline): `docs/planning/v4-predicate-dependency-graph-2026-05-30.md` (#4058)
- elastic CI redesign exploration (in flight, draft): `docs/planning/elastic-ci-redesign-exploration-2026-05-31.md` ([#4091](https://github.com/gunb-ai/gunbc/pull/4091)) — anchor for any structural-bridge replacement authoring + framing for the CI overhaul gap
- Definitions: `src/v4/TASKS.md:806-817`

## §10. Watchlist (receipt-state, not time-based)

**Likely next-flip** (high confidence, named workers in flight; expressed as receipts not deadlines):
- SG-6 #4085 lands once cursor RC addressed → P3 class closure increment
- merry-badger burn-down refresh post-cascade reflecting Wave 1 EXIT + Wa-2 landing
- proud-pike SG-RC-LAYERING §10.0 worksheet authored → downstream P3 worker dispatchable
- (Wa-2 #4080 LANDED dca5ce7a — already on main; fixture/law bundle 3/3)

**Decisions blocking forward progress** (no PM authority to resolve):
- **P2-B bridge deletion authorization** — without it P2 stays AUTHORITY-BLOCKED, P4+P6 cascade gated
- **Structural-bridge replacement dispatch authorization** — needs DFS authoring of positive-Y CiUpsertStep replacing scripts/v4-testclaim-corpus-gate.sh; should align with #4091 §3 substrate state (operator framing)
- **#4091 ratification posture** — currently draft+exploration; flipping to ratified would unblock structural-bridge replacement framing
- **SG-RC-LAYERING vs SG-1b prioritization** — proud-pike authoring SG-RC-LAYERING; SG-1b sibling needs separate keen-heron-relayed worksheet (operator: parallel or serial?)
- **CI risk posture** — ci_v4 timeout (#4074 was a 32min nailbiter; #4092 saved ~14m but partial); operational nuisance vs merge-blocker until #4091 lands
