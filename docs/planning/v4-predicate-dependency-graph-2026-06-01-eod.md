# v4 predicate dependency graph — 2026-06-01T00:30Z (Jun 1 release day EOD)

Forward projection of remaining work to v4-done as of Jun 1 release day end-of-day. Supersedes `docs/planning/v4-predicate-dependency-graph-2026-05-31.md` (#4096) for current state. Predicate definitions at `src/v4/TASKS.md` section "Definition of v4-done".

## §1. Six v4-done predicates — current state

| # | Predicate (per `src/v4/TASKS.md` "Definition of v4-done") | State |
|---|---|---|
| P1 | Every other scheduled task complete | YELLOW (~8-12/53 PROVEN; R3-internal added; per-lane GAP continues; no near-term GREEN candidate) |
| P2 | v4 compiles `src/v4/compiler/*.dag` end-to-end | **GREEN.** P2-A probe (44-source closure 0 diagnostics) verified scope (a). M2 probe (#4097) confirmed bridge dead-weight. **#4139 MERGED 00:50Z (squash 12d1aa68c)** — `scripts/v4-bootstrap-resolve-posture-gate.sh` deleted from main; `.github/workflows/ci.yml` bridge step removed. P2 now fully GREEN. |
| P3 | v4 emits Rust source that compiles to a binary | YELLOW (downstream-cascading). **9 of 9 routed classes closed or worksheet-on-main** today: SG-1 ✓ SG-7 ✓ SG-5 ✓ SG-6 ✓ SG-1b ✓ SG-2 ✓ SG-RC-LAYERING ✓ SG-3-cascade-retired ✓ SG-8 worksheet ✓ (impl gated on §8 sign-off). **Residual rustc count: 7,724** per fresh post-cascade probe (sleek-heron-13 PR #4140; +549 vs #4122's 7,175 — substrate landings introduced new diagnostics absorbed back into existing classes). Per-class delta on review. Two minor follow-ons remain: SG-1-FOLLOWON (per amend), SG-COLLECTION-PROJECTION (~170, worksheet drafted for implementation routing). |
| P4 | Binary on `src/v4/compiler/*.dag` produces bit-identical output | RED — hard-gated on P3 binary builds (P2 now GREEN per #4139) |
| P5 | TestClaim suite passes | **Layer 1 + Layer 2 BOTH CLOSED** via #4115 (P5 structural-bridge replaced + deleted). Strict P5 "suite passes" still gates on runtime-execution path: **option (ii) bootstrap-evaluator corpus runtime authorized 2026-06-01T01 as WORKSHEET** (not impl) per §8.5 #3; Modeling DFS Arbiter (to spawn) authors the worksheet. P3 cascade continues independently. T-38-PR2 verdict-SURFACE migration LANDED #4120. |
| P6 | Hand-authored Rust not editable authority (proven by REPRODUCTION) | RED — hard-gated on P4 + P3 PROVEN |

## §2. What closed today (2026-05-31 since 06:30Z)

This was the biggest single-day cascade of the project. Receipt-producing landings (predicate state movement, not activity counting):

**P3 class closures:**
- SG-1 #3956 (-2978 E0423 errors)
- SG-7 cleared (per #4086 catalog)
- SG-5/SG-6 #4121 (worksheet §6 closure + falsification receipt)
- SG-1b #4118 + #4128 (TR-lane impl + fail-closed hardening)
- SG-2 #4124 (worksheet §6 closure + P5 smoke receipts)
- SG-RC-LAYERING #4116 + #4133 + #4135 (~700 errors / ~10% addressed)
- SG-3-CASCADE #4126 (cascade-only determination — retired, no §10.0)
- SG-8 worksheet #4127 (impl pending §8 sign-off)

**P5 closure:**
- P5 structural-bridge worksheet #4114 + implementation #4115 (positive-Y CiUpsertStep replaces shell, bridge deleted)
- T-38-PR2 verdict-surface migration #4120 (host_verdict_surface_receipt_v3 with per-row Pass/Fail/Deferred counts)

**Infra + adjacent receipts:**
- #4097 M2 probe receipt (P2-B safety-net, day 1 of 14)
- #4101 CI bankruptcy B0/B1 (Tier-0 fold; legacy v2/v3/v4/self_host_ratchet jobs deleted)
- #4112 + #4111 + #4109 + #4087 + #4084 + #4077 + #4075 burn-down ratchets (merry-badger maintenance cadence)
- #4117 Python R2a/R2b/R3-external widening (MW-D3 Rust+Python parity)
- #4122 fresh M1 catalog (7175 baseline)
- #4126 + #4130 + #4131 P3 receipt docs
- #4127 SG-8 worksheet
- #4129 v2-compiler tokenize API fix
- #4099 + #4100 SG-1b + SG-RC worksheets
- #4094 + #4096 dep graph refreshes (this doc's predecessors)
- #4091 elastic CI redesign ratified
- #4095 + #4105 + #4107 elastic compute fabric + cache substrate (Worksheet A + B)
- #4078 W2.3 Bucket E (5 GateStep CiUpsertStep rows)

**Org changes:**
- proud-pike-680 (Modeling DFS Manager) ARCHIVED mid-day; children reparented under keen-heron-687
- Workers archived: valiant-ferret, witty-lark, sleek-bat, sharp-lark, sleek-badger, keen-bat, sharp-wolf-children (still-koi, quick-heron), cool-boar, witty-fox, valiant-eagle, sharp-raven, sunny-ibex, sunny-cat, eager-ferret
- Per-day pattern: child worker spawn → ~7-30 min round-trip to merged PR → archive

## §3. Per-predicate remaining work — dependency list

### §3.1 P1 every-other-task complete

Per-lane maintenance continues. Not Jun 1 GREEN candidate. R3-internal addition + scattered closures throughout the day; total count needs fresh sharp-otter roster refresh.

### §3.2 P2 — GREEN (TECHNICAL-PROVEN + AUTHORITY-DELETED)

**Layer 1 (technical):** PROVEN via P2-A probe.

**Layer 2 (authority):** **DELETED via #4139 MERGED 2026-06-01T00:50Z (squash 12d1aa68c)** — `scripts/v4-bootstrap-resolve-posture-gate.sh` removed from main; `.github/workflows/ci.yml` bridge step removed. Operator authorized early deletion at 00:28Z (skipping 14-day window) on M2 probe (#4097) safety-net receipt; codex worker shipped the deletion PR within 22 minutes. P2 fully GREEN.

### §3.3 P3 — 9 routed classes; 8 closed/retired/worksheet-on-main + 2 follow-on stragglers

| Class | State | Routing | EOD note |
|---|---|---|---|
| SG-1 | **CLOSED** ✓ | #3956 squash 78b9698a | -2978 E0423 errors |
| SG-7 | **CLOSED** ✓ | per #4086 catalog | residual cleared |
| SG-5 | **CLOSED** ✓ | #4121 squash f7ce371c | worksheet §6 closure |
| SG-6 | **CLOSED** ✓ | #4085 squash 2ac52f26 | BoundedLattice instance-completeness gate |
| SG-1b | **CLOSED** ✓ | #4118 + #4128 (TR-lane) | function-signature realization rows + fail-closed hardening |
| SG-2 | **CLOSED** ✓ | #4124 squash 9414fc22 | TargetTypeExpressionProjection §6 |
| SG-RC-LAYERING | **CLOSED** ✓ | #4116 + #4133 + #4135 | ~700 errors / ~10% closed via single-authority-fact substrate |
| SG-3-CASCADE | **RETIRED** ✓ | #4126 squash 610be95a + #4130 | cascade-only determination; no §10.0; bands owned by primaries |
| SG-8 | **worksheet on main, impl gated** | #4127 squash 8c268005 | §8 sign-off pending — see §6.5 blocker |
| SG-1-FOLLOWON | routed (worker-only) | extend SG-1 worksheet | minor; not separately tracked |
| SG-COLLECTION-PROJECTION | worksheet-drafted | `docs/planning/v4-sg-collection-projection-worksheet-2026-06-01.md` | ~170 errors; FreeMonoid boundary projection stays separate from SG-5/SG-6 Set realization |

**P3 PROVEN bar reminder**: Rust source compiles to a binary (not zero rustc errors as goal; not Python+Go alpha-targets). Each closure shrinks residual; binary builds when residual hits 0 on Rust path.

**Residual rustc count**: 7,724 per fresh post-cascade probe (sleek-heron-13 PR #4140; +549 vs #4122's 7,175 baseline). Substrate landings introduced new diagnostics absorbed into existing classes (no new class signatures); per-class delta on review will inform next-wave routing.

### §3.4 P4 bit-identical fixpoint

Hard-gated on P3 binary builds (P2 fully GREEN per #4139 MERGED 00:50Z). No active worker dispatchable until P3 residual reaches binary-build threshold.

### §3.5 P5 TestClaim suite passes — Layer 1 + Layer 2 BOTH CLOSED

**Layer 1 — Fixture/law bundle: 3/3 CLOSED** (Wa-1 #4079 + Wa-2 #4080 + P5-D tranche-2 #4089).

**Layer 2 — Authority gate: CLOSED via #4115** (positive-Y CiUpsertStep replaces shell; `scripts/v4-testclaim-corpus-gate.sh` deleted).

**T-38-PR2 verdict-surface migration LANDED #4120**: `blocked_m1_subset` string retired; `host_verdict_surface_receipt_v3` with per-row Pass/Fail/Deferred counts.

**Strict P5 "suite passes" reading still has upward debt** (`node://adhoc-f8699326-d69`): true runtime corpus execution gated on EITHER (i) M1 cargo-clean emitted subset OR (ii) new bootstrap-evaluator corpus runtime per SELF_HOSTING. **Routing decision RESOLVED 2026-06-01T01 per §8.5 #3: option (ii) authorized as WORKSHEET** (Modeling DFS Arbiter authors; not yet implemented). (i) remains the patient-wait path (downstream of P3 cascade closures). Gate itself stays OPEN until runtime ships + suite executes + passes.

### §3.6 P6 hand-Rust REPRODUCTION

Hard-gated on P4 + P3.

## §4. Critical paths forward — post-cascade

**Path A: P5 strict "suite passes" (runtime gate)** — **option (ii) WORKSHEET authorized 2026-06-01T01** per §8.5 #3. Modeling DFS Arbiter (to spawn) authors bootstrap-evaluator corpus runtime worksheet; P3 cascade continues independently. P5 strict GREEN still requires `src/v4/TASKS.md` "TestClaim suite passes" satisfied by actual suite execution receipt — worksheet authorization unblocks the path, not the predicate.

**Path B: P2 GREEN flip** — **DONE.** #4139 MERGED 2026-06-01T00:50Z (squash 12d1aa68c); bridge script + ci.yml step removed from main.

**Path C: P3 binary build (residual march)** — SG-8 impl worker can spawn once §8 sign-off resolves; SG-1-FOLLOWON + SG-COLLECTION-PROJECTION minor follow-ons; fresh M1 probe to reflect actual post-cascade count. Each class closure shrinks residual; binary builds when ≈0 on Rust path. Long-horizon.

**Path D: P1 + P4 + P6 cascade** — all gated on P2 + P3 above.

## §5. Active workers / no work

Tree is at 0 active subtree work as of 00:30Z 2026-06-01. All five overnight subtree PRs (#4116, #4127, #4133, #4135, #4101) merged; #4112 operator-landed. Managers + closeout-leaf placeholders idle. Subtree health: 0 working / 7 idle / 0 blocked / 0 active PRs.

## §6. Active blockers (post-EOD)

| Blocker | Owner action |
|---|---|
| ~~P2-B bridge deletion authorization~~ | **RESOLVED + DELIVERED 2026-06-01T00:50Z** — #4139 MERGED (squash 12d1aa68c); script + ci.yml step gone from main |
| **P5 runtime gate** (still OPEN) | Routing decision RESOLVED 01Z (option (ii) WORKSHEET) but gate itself OPEN — runtime not yet shipped, suite not yet executed. Modeling DFS Arbiter (to spawn) authors worksheet; impl follows. |
| **SG-8 §8 sign-off** (authority routing resolved; ratification still open) | Authority RESOLVED 01Z per §8.5 #4 (dedicated Modeling DFS Manager to spawn). Actual §8 ratification on SG-8 worksheet still gates impl dispatch — happens after Arbiter spawn + review. |
| ~~Fresh M1 probe post-cascade~~ | **RESOLVED 2026-06-01T00:28Z** — authorized; work-item `adhoc-7b46e080-3cd` in flight (sleek-heron-13 auto-spawned; PR #4140 opened) |

## §6.5 Current Dispatch Board

| Priority | Item | State | Owner | Exit receipt |
|---|---|---|---|---|
| 1 | P5 runtime gate (gate OPEN; routing RESOLVED) | Routing RESOLVED 01Z per §8.5 #3 — option (ii) WORKSHEET authorized | Modeling DFS Arbiter (to spawn) | bootstrap-evaluator corpus runtime worksheet authored + §8 approved + impl ships + suite executes |
| 2 | P2-B bridge deletion | **DONE 2026-06-01T00:50Z** | #4139 MERGED (squash 12d1aa68c) | P2 fully GREEN |
| 3 | SG-8 §8 sign-off **routing** (NOT the ratification itself) | Routing **RESOLVED 2026-06-01T01** per §8.5 #4 — dedicated Modeling DFS Manager to spawn. The actual §8 ratification on SG-8 worksheet remains PENDING and still gates impl dispatch (sequenced: Arbiter spawn → §8 ratification → SG-8 impl). | Modeling DFS Arbiter (to spawn) | §8 ratification → SG-8 impl worker dispatchable |
| 4 | Fresh M1 probe post-cascade | **AUTHORIZED + in flight** (PR #4140 sleek-heron-13) | work-item `adhoc-7b46e080-3cd` | residual = 7,724 (+549 vs #4122); per-class delta on review |
| 5 | SG-1-FOLLOWON impl | routed | TR lane (keen-heron) | minor follow-on close |
| 6 | SG-COLLECTION-PROJECTION amend | worksheet-drafted | TR/proud-pike-successor | `docs/planning/v4-sg-collection-projection-worksheet-2026-06-01.md`; ~170 errors |

## §7. PM-side actionable items

**Worker implementation dispatch state (aligned with §6.5 dispatch board + §8.5 resolutions):**
- **SG-8 impl: pending Modeling DFS Arbiter spawn** then §8 sign-off (operator decision #4 RESOLVED 01Z; Arbiter spawn is now the next step)
- **P5 runtime gate impl: pending worksheet authoring** (operator decision #3 RESOLVED 01Z — option (ii) WORKSHEET authorized; Arbiter writes the worksheet, then impl spawns post-§8)
- All other P3 classes either closed today (per §3.3) OR have follow-on routing (SG-1-FOLLOWON minor, SG-COLLECTION-PROJECTION worksheet drafted)

**PM dispatches from 2026-06-01T00:28Z operator authorization:** P2-B deletion DELIVERED via #4139 MERGED 00:50Z; fresh M1 probe in flight via PR #4140 (sleek-heron-13).

**Operator decisions still blocking forward progress** (per consolidated §8.5):
- ~~P5 runtime gate (i) vs (ii)~~ **RESOLVED 01Z**: option (ii) WORKSHEET authorized (see §8.5 #3)
- ~~SG-8 §8 sign-off authority routing~~ **RESOLVED 01Z**: spawn dedicated Modeling DFS Manager (see §8.5 #4)
- ~~Modeling DFS Manager succession plan~~ **RESOLVED 01Z**: dedicated manager going forward (see §8.5 #5)
- ~~TypeScript work plan authorization~~ **RESOLVED 01Z**: alpha/preview lane authorized (see §8.5 #6)
- stern-lynx closeout cond (c) — still pending (low-priority)

## §8. Risk / honesty

- **P3 cascade was massive but residual is still significant**: 7,724 rustc errors per fresh probe (sleek-heron-13 PR #4140; +549 vs #4122's 7,175 baseline — substrate landings introduced new diagnostics absorbed into existing classes, not new class signatures). Per-class wins are smaller than SG-1's 37% magnitude per #4096 §8 honesty. Realistic horizon: weeks-to-months for residual → 0.
- **P5 single authority**: P5 is **GREEN only when the canonical `src/v4/TASKS.md` "Definition of v4-done" bullet "TestClaim suite passes" is satisfied** — which requires the runtime-execution gate to close. Today's Layer 1 + Layer 2 closures are **partial progress toward P5**, not an alternate definition of GREEN. Surfaced explicitly by T-38-PR2 worker's honest scope analysis (cool-boar-841 identified 3 readings, chose surface migration + escalated runtime gate as upward debt at `node://adhoc-f8699326-d69`; correct discipline per project_spirit "stop and escalate"). Do not treat partial layer closure as P5 GREEN.
- **proud-pike archival mid-day** consolidated Modeling DFS lane under keen-heron de facto. §8 sign-off authority needs explicit operator routing for SG-8 impl worker dispatch.
- **#4112 operator-landed manually** despite dashboard recommending self-merge — operator chose to handle that specific PR, others all self-merged per current policy.
- **Per-class delta for the 7,724 residual is still pending review** (sleek-heron-13 PR #4140 captures total + acknowledges substrate-absorbed-into-existing-classes pattern; granular per-class breakdown surfaces on PR review and informs whether new dominant classes need §10.0 worksheets).

## §8.5 Operator sign-offs queue (consolidated)

Captures everything PM is waiting on operator to confirm/decide. Updated as items resolve.

| # | Decision | Status | Notes |
|---|---|---|---|
| 1 | P2-B bridge deletion | **DONE 2026-06-01T00:50Z** (early deletion, 14-day window skipped) — #4139 MERGED (squash 12d1aa68c); script + ci.yml step removed from main | P2 flipped fully GREEN |
| 2 | Fresh M1 probe post-cascade | **AUTHORIZED 2026-06-01T00:28Z** — work-item `adhoc-7b46e080-3cd` in flight (sharp-otter archived; fresh worker spawned) | Gives clean post-cascade P3 residual count |
| 3 | P5 runtime gate **routing decision** (i) vs (ii) | **DECISION RESOLVED 2026-06-01T01** — option (ii) authorized as **WORKSHEET** (not implementation). The P5 runtime-execution **gate itself remains OPEN** — gate closes only when runtime ships + suite executes + passes (per §1 P5 row + §3.5 + §10). Worksheet authoring is the next step toward gate closure, not the closure itself. |
| 4 | SG-8 §8 sign-off **authority routing** | **ROUTING RESOLVED 2026-06-01T01** — dedicated Modeling DFS Manager to spawn; PM does NOT act as Modeling DFS; keen-heron remains TR. Note: actual §8 ratification on SG-8 worksheet is a separate step that happens after Arbiter spawn + worksheet review — gates SG-8 impl dispatch until then. |
| 5 | Modeling DFS Manager succession plan | **RESOLVED 2026-06-01T01** — dedicated manager going forward (per #4). |
| 6 | TypeScript / non-release-minimum work plan | **RESOLVED 2026-06-01T01** — authorized as alpha/preview language lane (NOT release-minimum per Wave F F3); spawn TS RCA Manager per §11.8 runway. |
| 7 | Go / Python language RCA managers | **RESOLVED 2026-06-01T01** — authorized for self-compile runway (Rust+Python+Go release-minimum family per §11.8; rungs L0-L4 sequenced). |
| 8 | CI Coverage Bankruptcy Wave 1 | **RESOLVED 2026-06-01T01** — APPROVED; spawn CI Manager to ship bankruptcy PR + four-compile collapse in parallel. |
| 9 | stern-lynx Runtime/TestClaim Manager closeout | PENDING | Cond (a) ✓ + (b) ✓ already; only (c) "operator confirms role complete" remains (low-priority since Runtime/TestClaim role is being reconstituted under new structure anyway) |

## §8.6 Remaining P3 residual cascade — worksheet protocol clarity

After fresh M1 probe lands (#2 in §8.5), the protocol is:
1. Probe identifies residual classes + counts
2. **Acting Modeling DFS** (per routing decision #4 in §8.5) reviews classes for new single-authority-fact emergence
3. If existing worksheets cover the residual → spawn impl workers per existing pattern (the 8 closures today were per this pattern)
4. If new classes emerged → §10.0 worksheet authoring under acting Modeling DFS → impl after §8 sign-off
5. Repeat: each merged closure → fresh probe → re-evaluate

**Manager**: keen-heron-687 is de facto current owner of all reparented Modeling DFS workers (sleek-bat / sunny-cat / etc post-proud-pike archival). Routing decision #4 will formalize.

**Mechanical rule** (unchanged from `INVARIANTS.md` + proud-pike's role): **NO worker dispatched on SG-class work until §10.0 worksheet identifies the single-authority fact to add or consume.** This is what prevented "fix 6991 errors" spot-fix dispatches today.

## §8.7 TypeScript / cross-target parallel opportunities

Per Wave F F3 framing, TS is v4-alpha-only (not release-minimum). Parallel TS class-closure work is fine — does NOT share single-authority-fact deps with Rust residual cascade. Concrete opportunities, mirroring today's Python widening (#4117) and earlier Rust patterns:

- **TS leaf-model R2a/R2b/R3-external widening** — direct analog of #4117 Python (algebra ops accepted by TS runtime, error falsification, atom realization projection); would close cross-target MW-D3 for TS
- **TS atom realization rows** (TargetAtomRealization for TS) — analog of SG-1 Rust closure pattern
- **TS type-expression projection** (TargetTypeExpressionProjection for TS) — analog of SG-2 #4124 pattern
- **TS algebra inhabitance widening** — analog of W1.7 / #4000 rust.dag widening
- **#3850 grammar-inverse TestClaims for python/go/cpp/typescript Shape-A targets** — revive if still relevant post-#4091 ratification

**Each TS class needs its own §10.0 worksheet** per single-authority fact rule. TR-lane (keen-heron-687) implements the realization rows once worksheets ratified.

**Operator decision #6 in §8.5 covers authorization to spawn this work plan.**

## §9. Cross-refs

- Prior dep graphs: `docs/planning/v4-predicate-dependency-graph-2026-05-30.md` (#4058), `docs/planning/v4-predicate-dependency-graph-2026-05-31.md` (#4096; updated in-place with SG-3 retirement)
- Burn-down: `docs/planning/v4-done-predicate-burn-down-2026-05-30.md` (merry-badger; multiple ratchets through the day)
- Post-SG-1 catalog: `docs/audit/v4-rustc-error-catalog-2026-05-31.md` (#4086) + post-#4115 (#4122)
- Elastic CI: `docs/planning/elastic-ci-redesign-exploration-2026-05-31.md` (#4091 ratified)
- P5 structural-bridge worksheet: `docs/planning/v4-p5-structural-bridge-replacement-worksheet-2026-05-30.md` (#4114)
- SG-RC-LAYERING worksheet: `docs/planning/v4-sg-rc-layering-worksheet-*.md` (#4100)
- SG-1b worksheet: `docs/planning/v4-sg-1b-function-signature-realization-worksheet-2026-05-30.md` (#4099)
- SG-8 worksheet: `docs/planning/v4-sg8-module-graph-carrier-reexports-worksheet-2026-05-31.md` (#4127)
- SG-3-CASCADE receipt: `docs/audit/v4-sg3-cascade-only-receipt-2026-05-31.md` (#4126)
- M2 probe: `docs/audit/v4-p2b-bridge-removal-probe-2026-05-31.md` (#4097)
- T-38-PR2: `docs/planning/...` per #4120
- Predicate defs: `src/v4/TASKS.md` section "Definition of v4-done"

## §10. Watchlist (Jun 1 release-day eod, no time-based ETAs)

**Operator decisions** — see §8.5 for full table (decisions #3-#8 RESOLVED 2026-06-01T01; only #9 stern-lynx closeout (c) remains pending, low-priority):
- ~~P5 runtime gate (i) vs (ii)~~ RESOLVED: option (ii) WORKSHEET authorized
- ~~SG-8 §8 sign-off authority routing~~ RESOLVED: dedicated Modeling DFS Manager to spawn
- ~~Modeling DFS Manager succession plan~~ RESOLVED: dedicated manager going forward
- ~~TypeScript work plan authorization~~ RESOLVED: alpha/preview lane authorized
- ~~Go/Python language RCA managers~~ RESOLVED: authorized for self-compile runway per §11.8
- ~~CI Coverage Bankruptcy Wave 1~~ RESOLVED: APPROVED; spawn CI Manager
- stern-lynx closeout cond (c) — still pending (low-priority)

**Next dispatch actions (post-#4137 merge)**:
- Spawn Modeling DFS Arbiter (per §8.5 #4-#5)
- Spawn CI Manager (per §8.5 #8) — Wave 1 bankruptcy PR + four-compile collapse
- Spawn TypeScript/Go/Python language RCA managers (per §8.5 #6-#7 + §11.8 runway)
- Modeling DFS Arbiter authors P5 runtime gate option (ii) worksheet (per §8.5 #3)

**PM dispatches from 2026-06-01T00:28Z operator authorization:**
- P2-B deletion DELIVERED via #4139 MERGED 00:50Z (squash 12d1aa68c) → P2 fully GREEN
- Fresh M1 probe in flight via PR #4140 (sleek-heron-13; residual = 7,724)

**Receipts available for review:**
- All today's class closures on main; predicate-state movement substantial across P3 + P5 layers

**Jun 1 release readiness summary:**
- v4 ships alpha per Flavor (iv) operator framing (per session memory `project_june1_release`)
- Substrate landings: massive day (SG-1/2/5/6/1b/RC-LAYERING + SG-3-retired + SG-8-worksheet + #4115 P5 + #4120 T-38-PR2 + W2.3 Bucket E + #4091 elastic CI ratification + compute fabric + cache substrate)
- **P5 (canonical "TestClaim suite passes" per TASKS.md): NOT fully GREEN today** — Layer 1 + Layer 2 closed but runtime-execution gate remains open at `node://adhoc-f8699326-d69`. P5 flips fully GREEN only when runtime-execution gate closes; do NOT treat as done.
- **P2: fully GREEN** — #4139 MERGED 00:50Z (squash 12d1aa68c); bridge script + ci.yml step removed from main
- P3 multi-class cascade closed today but residual rustc count significant; long-horizon to binary builds

## §11. Proposed next-iteration management structure

This section captures the operator-ratified (pending re-review) management redesign for the post-Jun-1 next-iteration. Replaces the dissolved structure (proud-pike Modeling DFS Manager archived; sharp-otter Close/Receipt Manager archived; quick-tern Runtime/TestClaim Manager archived; keen-heron TR Manager archived; merry-badger burn-down remains active for #4141; sleek-heron remains active for #4140).

### §11.1 Structural principle

**Fan out RCA + worksheet authoring; centralize single-authority-fact approval.** Per `INVARIANTS.md` P2 (single authority), no implementation worker is dispatched until a §10.0 worksheet identifies the missing modeled fact. The bottleneck was historically a single Modeling DFS lane; the new structure parallelizes worksheet drafting across language/substrate domains while keeping approval centralized in one Modeling DFS Arbiter.

### §11.2 Manager roles

**Modeling DFS Arbiter** (NEW, fresh-spawn — NOT PM):
- Scope: approves §10.0 single-authority-fact worksheets; rejects spot-fixes; detects cross-language shared facts; merges duplicate worksheets; decides whether a fix belongs in `std/`, target_model, `extdeps/language`, or compiler-spine
- Deliverable by end of tenure: every active rustc residual class has either (a) a single-authority-fact-identified §10.0 worksheet on main + named implementation worker, OR (b) explicit retirement receipt
- Exit criteria: P3 residual reaches the "binary builds" threshold (rustc errors near-zero on Rust compile path) OR operator dissolves role

**Per-language RCA Managers** (5 lanes, parallel):

| Manager | Scope | Deliverable by end of tenure | Exit criteria |
|---|---|---|---|
| **Rust RCA Manager** | Probe + residual cluster + worksheet draft + impl fanout for Rust target; coordinate with Modeling DFS Arbiter on shared facts | All rustc residual families have routed worksheets (existing or new); SG-8 + E0308 stratification + SG-2 residual + ownership/use-site + collection projection all under named worksheets | P3 rustc residual reaches binary-build threshold |
| **Python RCA Manager** | Same shape, Python target | MW-D3 Python parity complete (R1+R2a+R2b+R3-external on main per #4117); Python suite passes against modeled emit | Python alpha-release-minimum bar met |
| **Go RCA Manager** | Same shape, Go target | MW-D3 Go parity complete (R1+R2a+R2b+R3-external analog); Go suite passes against modeled emit; Go emit fix #4076 follow-ons closed | Go alpha-release-minimum bar met |
| **TypeScript RCA Manager** (alpha/preview) | Same shape, TS target | TS leaf-model R2a/R2b/R3-external widening; TS TargetAtomRealization rows; TS TargetTypeExpressionProjection; TS algebra inhabitance; TS grammar-inverse TestClaims | TS alpha-preview-coverage bar met (operator-defined when authorized) |
| **C++ RCA Manager** (LATER / capacity-permitting) | Same shape, C++ target | C++ language residual matrix produced; no impl until shared substrate authority resolved | Operator-defined when authorized |

**Shared substrate managers** (6 lanes; named single-authority owners):

| Manager | Scope | Deliverable by end of tenure | Exit criteria |
|---|---|---|---|
| **ModuleGraph / Import-Export Manager** | Owns module-graph / import-export / re-export authority across all language targets; first responder for SG-8 residual family (E0425/E0432/E0433) | SG-8 worksheet implementation on main; module/import authority projection lands; cross-language import-graph contract ratified | SG-8 residual = 0; no E0425/E0432/E0433 emitted in any target |
| **TargetTypeExpression Manager** | Owns TargetTypeExpressionProjection across all targets; first responder for SG-2 residual family (E0107/E0282) | SG-2 residual worksheet (generic instantiation preservation across aliases, caches, signatures); higher-kinded-ish shape projection | SG-2 residual = 0 |
| **TargetAtom / Primitive Realization Manager** | Owns TargetAtomRealization for all targets (carries: Symbol, Bool, Char, Int, etc); first responder for atom-related E0308 clusters | SG-1b follow-on closure; function-signature realization for atom-typed params/returns; cross-target atom contract ratified | Atom realization E0308 cluster = 0 |
| **Collection / Algebra / Law Manager** | Owns TargetCollectionRealization + FreeMonoid/Vec/Rc boundary; algebra inhabitance + law preservation across targets | SG-COLLECTION-PROJECTION worksheet + impl; collection boundary projection row; law preservation receipts per target | Collection-boundary E0308 cluster = 0; law preservation receipts on main |
| **Runtime / TestClaim Manager** | Owns T-22 eval execution + T-38 structured TestClaimRun verdicts + emitted-code run harness + falsification verdict receipts | P5 strict "suite passes" — runtime-execution gate closed via path (i) or (ii); bootstrap-evaluator corpus runtime modeled and implemented | P5 strict GREEN per `src/v4/TASKS.md` "TestClaim suite passes" |
| **CI Manager** (NEW, explicit named role) | Owns `src/v4/workflow/ci.dag` substrate maintenance + extension; remaining shell-to-CiUpsertStep migrations (per standing `project_no_new_shell` directive); CI runtime drops + elastic redesign implementation per #4091; coordinates with Compiler Spine + Modeling DFS Arbiter | (a) Four-compile-redundancy in ci_v4 collapsed to ≤1; (b) every YAML step backed by `CiUpsertStep` row with receipt parity; (c) affected-set testing/building running via evaluator (not YAML+shell); (d) remaining shell scripts retired per dissolution policy | (a)+(b)+(c)+(d) all on main; CI runtime ≤10min wall on cold cache for v4 affected-set |

### §11.3 Coordination protocol

```
1. Fresh probe lands (Close/Receipt-style worker; reports per-class delta).
2. Modeling DFS Arbiter classifies residual families per shared substrate domain.
3. Per-language RCA Managers + Shared substrate managers draft §10.0 worksheets in parallel.
4. Arbiter reviews + approves single-authority facts (centralized chokepoint).
5. Approved worksheets spawn implementation workers in batches (parallel within a wave).
6. Wave merges → re-probe (not after every PR).
7. Repeat.
```

**Mechanical rule preserved (unchanged from `INVARIANTS.md`):** NO worker dispatched on SG-class work until §10.0 worksheet identifies the single-authority fact to add or consume.

### §11.4 Immediate next-wave dispatch (post-#4140 + #4137 merge)

Once operator ratifies §11 structure and #4140 lands as measurement receipt:

1. **Spawn Modeling DFS Arbiter** (fresh session)
2. **Spawn 6 shared substrate managers** + **3-4 language RCA managers** in parallel
3. **First wave authorizations** (per #4140 RCA fanout):
   - SG-8 / ModuleGraph manager → impl wave on E0425/E0432/E0433 (biggest delta +420)
   - E0308 stratification → Rust RCA + TargetAtom + Collection + Ownership managers split the cluster
   - SG-2 residual → TargetTypeExpression manager
   - P5 runtime gate path (ii) → Runtime/TestClaim manager authoring worksheet
   - CI Manager → start migrating remaining YAML steps to CiUpsertStep rows
4. **Re-probe after wave** (not per-PR)

### §11.6 CI Manager — concrete migration inventory (per operator request)

Captures what specifically needs to migrate for "affected-set testing/building via evaluator" + the larger CI substrate goal. Verified counts as of 2026-06-01T01:00Z post-#4139 main.

**Headline numbers:**
- `.github/workflows/ci.yml`: **73 YAML steps** across 7 jobs (fmt / discipline / affected / ci_integration / v3 / ci_v4 / ci aggregator / v4 aggregator)
- `src/v4/workflow/ci.dag`: **47 modeled `CiUpsertStep` rows** on main (W2.3 Buckets A+B+C+E + bankruptcy Tier-0 + P5 structural-bridge replacement #4115)
- `scripts/`: **~30 shell scripts** in scripts/; ~15-20 invoked from ci.yml gate/check steps

**Migration scope (~26 YAML steps still without `CiUpsertStep` backing):**

| Category | Count | Notes |
|---|---|---|
| **A. Already modeled + receipt-parity** | 47 rows | On main per #4078 W2.3 Bucket E + bankruptcy Tier-0 + #4115 P5; receipt parity verified via `ci_pipeline_step_ids_shadow` bijection |
| **B. Pending modeling (gate/check steps)** | ~10 | SG-0 net-shrink discipline, R4-carve dissolution, Fabrication sentinel ratchet, T-19 testgen activation, Release-doc authority check, install.sh pinned-version smoke, Manager-brief authority check, Test-timeout ratchet, Rust toolchain single-authority check, Gate #103 path-regex + Layer 1 selection |
| **C. Pending modeling (leaf-model boundary steps)** | ~6 | Phase 1 leaf-model R1 rustc / Python R1 / Python R2a-R2b-R3-external / Rust R2a-R2b-R3-external boundaries (gunbc#846 bypass steps) |
| **D. Pending modeling (toolchain/cache steps)** | ~5 | Isolate toolchain dirs (×4 jobs), Clear inherited GitHub auth header (×4 jobs), Pin global rustup default (×4 jobs), Cache Cargo, Cache gunbc binary — currently per-job duplication, candidates for shared `CiUpsertStep` factoring |
| **E. Already retired via dissolution** | n/a | `scripts/v4-testclaim-corpus-gate.sh` deleted #4115; `scripts/v4-bootstrap-resolve-posture-gate.sh` deleted #4139; legacy v2/v3/v4/self_host_ratchet jobs folded into bankruptcy #4101 |

**Shell scripts pending retirement (per `project_no_new_shell` standing directive; each requires positive-Y `CiUpsertStep` modeling first per the `#4115` pattern):**

```
scripts/v4-bootstrap-viability.sh                     # main v2→v4 bootstrap compile path (invoked from ci_v4)
scripts/v4-testclaim-corpus-eval.sh                   # T-22 corpus eval host transport (partially modeled per #4115; runtime still shell)
scripts/v4-m1-rust-emit-probe.sh                      # M1 rust emit probe step
scripts/v4-mvp1-e2e-gate.sh                           # MVP-1 add.dag e2e gate
scripts/v4-phase1-nat-semiring-rung-gate.sh           # phase1/nat_semiring rungs 0-2 gate
scripts/v4-leaf-model-python-r1-verify.sh             # category C above
scripts/v4-leaf-model-python-r2a-verify.sh
scripts/v4-leaf-model-python-r2b-verify.sh
scripts/v4-leaf-model-python-r3-external-verify.sh
scripts/v4-leaf-model-rust-r3-external-verify.sh
scripts/check-pr-sg0-net-shrink-discipline.sh         # category B (SG-0 net-shrink)
scripts/check-r4-carve-dissolution-discipline.sh      # category B (R4-carve)
scripts/check-rust-toolchain-single-authority.sh      # category B (Rust toolchain)
scripts/check-compiler-std-ratchet.sh                 # compiler-std ratchet
scripts/check-test-timeout.sh                         # test timeout ratchet
scripts/check-release-doc-authority.sh                # release-doc authority
scripts/check-manager-brief-authority.sh              # manager-brief authority
scripts/r1_p0_no_fabrication_sentinel.sh              # fabrication sentinel
scripts/r3-debt-velocity.sh                           # debt velocity
scripts/l1-ratchet.sh                                 # L1 ratchet
scripts/detect-phase1-nat-semiring-gate-scope.sh      # PR diff scope detector
scripts/regenerate-stage0.sh                          # stage0 regen (developer-facing; may stay)
scripts/publish-snapshot.sh                           # snapshot publishing
scripts/release-target-triples.sh                     # release target triples
scripts/install-hooks.sh                              # pre-push hook installer (developer-facing; stays)
scripts/test-check-*.sh                               # self-test scripts (test infrastructure; stays)
```

~22 production shell scripts need positive-Y modeling + retirement; ~8 are developer-facing or test infrastructure that stays.

**Structural redundancies pending resolution (per #4091 §1.2):**
- **Four-compile redundancy in `ci_v4`**: M1 rust emit + v2→v4 bootstrap dag + T-22 corpus rust + T-22 corpus dag — all run against same 332-source closure. Worth ~14m saved if collapsed to 1.
- **Per-job `$HOME` isolation pattern** (per #4091 §2.3): 4 jobs duplicate `Isolate toolchain dirs` + `Clear GitHub auth header` + `Pin rustup default` steps. Candidates for shared substrate.
- **Cache cleanup steps** (`Cache cleanup (gates 3s)` repeated across jobs)

**Runtime authority migration:**
- Today: `.github/workflows/ci.yml` is the runtime authority; GitHub Actions executes the YAML directly
- Target: `src/v4/workflow/ci.dag` `CiPipeline` is the runtime authority; YAML is emitted from `ci.dag` (or replaced by an evaluator-driven runner)
- Path: requires (a) all 73 steps modeled, (b) `ci_pipeline_step_ids_shadow` becomes authority (not shadow), (c) evaluator that executes modeled pipeline OR YAML generator from `ci.dag`
- **Coupled with P5 runtime gate** (`adhoc-f8699326-d69`): the bootstrap-evaluator corpus runtime path (option (ii)) is the same architectural shape as the CI evaluator runtime; both gate on the same SELF_HOSTING-load-bearing substrate work

**Affected-set testing/building specifically:**
- Today: `needs.affected.outputs.v4 == 'true'` (shell+YAML condition), feeds into per-job conditional gates
- Modeled: `ChangeSet` → `AffectedSet` → `AffectedSetProduced` substrate exists in `src/v4/std/change.dag`
- Gap: runtime that consumes the modeled `AffectedSet` instead of the YAML conditional. Same gap as P5 runtime gate.

**CI Manager near-term wins (executable without operator decisions):**
1. **Migrate categories B (~10 gate/check steps) + C (~6 leaf-model boundaries) to `CiUpsertStep` rows** — pattern proven via W2.3 Bucket E (#4078); each row ~30min authoring; receipt parity via shadow bijection
2. **Collapse category D toolchain duplication** — shared substrate factoring; one CiUpsertStep instead of 4×
3. **Collapse four-compile redundancy in ci_v4** — most-impactful runtime drop (~14m on cold cache)

**CI Manager long-pole (gated on operator decision + Modeling DFS Arbiter):**
- Runtime authority migration (YAML→evaluator) — coupled with P5 runtime gate decision

### §11.7 CI Coverage Bankruptcy — operator-ratified aggressive cut plan (2026-06-01T01)

**Core operator framing**: cut shell-driven CI now to a bare safety floor. Keep one broad floor; move everything else to modeled / shadow / nightly / dropped. Do NOT pretend affected-set is ready for exclusive gating; run it in shadow until proven.

**Why now**: CI is bogging the program down. `ci_v4` is 33m46s wall (per #4091 §1.2); 14m40s is duplicated compiler work on the same 332-source closure (four-compile redundancy). 73 YAML steps with 26+ still shell-owned; multiple v3-era ratchets enforcing counts against retired surfaces. The program cannot afford to wait for full modeled-substrate migration before cutting.

#### §11.7.1 BARE MINIMUM SAFETY NET — what we actually need (Class A)

**This is what stays required on every PR.** Everything not on this list is either Class C (shadow/scheduled/manual) or Class D (cut/deleted).

| # | Required gate | Purpose | Notes |
|---|---|---|---|
| 1 | `fmt --check` | Format hygiene | Cheap (~30s); near-universal in Rust projects |
| 2 | Bootstrap minimal viability (v2→v4 compile path) | The tree compiles end-to-end via the path we ship | Currently `v4-bootstrap-viability.sh`; can stay shell short-term per Class A exception; the modeled `CiUpsertStep` replacement is Wave 2 work |
| 3 | **`M1 v4 full-tree rust emit probe`** (single Rust emit/probe) | Single sample of P3 surface ("does v4 emit Rust that rustc can chew on?") | Picked M1 over T-22 corpus rust: M1 is the canonical residual-count probe (drives #4086/#4122/#4140 catalogs informing dispatch); T-22 corpus rust duplicates same closure compile, no unique signal vs M1 → demoted to Class C scheduled |
| 4 | `ci.dag` structural receipt + `CiUpsertStep` schema checks | Ensures modeled substrate stays well-formed | Already cheap; modeled-positive-Y |
| 5 | no-new-shell / shell-retirement ratchet | Enforces `project_no_new_shell` directive | Mechanical check; ensures CI doesn't regress on shell-cut posture |

That's it. **Five required gates.** Cold PR target: ~5-10 minutes (vs current ~30-75min).

#### §11.7.2 The four CI classes (for categorization)

| Class | Required on PR? | What happens now |
|---|---|---|
| **A. Safety floor** | Yes | The 5 items above; nothing else |
| **B. Modeled positive-Y** | **Only if part of current §11.7.1 safety floor OR selected by trusted affected-set/testgen** | "Already modeled" does NOT automatically mean "required on every PR" — that would let the required surface silently grow back. Until affected-set/testgen is trusted (post-Wave 3 receipts), modeled-positive checks outside the 5-gate floor run in shadow or scheduled. |
| **C. Shadow / diagnostic** | No | Runs on demand, scheduled, or as shadow receipt; produces evidence but does not block |
| **D. Cut until modeled** | No | Disabled/deleted from required CI; restoration path named (or permanently deleted for retired-lane items like v3) |

#### §11.7.3 Specifically cut/demote immediately

| Item | Class | Reason |
|---|---|---|
| All 5 leaf-model verify shells (`v4-leaf-model-python-{r1,r2a,r2b,r3-external}-verify.sh` + `v4-leaf-model-rust-r3-external-verify.sh`) | C/D | "Dissolve-on-arrival" per their own headers; Python R1/R2a/R2b/R3-external substrate landed via #4117 |
| `check-pr-sg0-net-shrink-discipline.sh` | D | v3 hand-Rust net-shrink ratchet against retired v3 lane |
| `check-compiler-std-ratchet.sh` | D | counts `^type [A-Z]` in `src/v3/compiler/*.dag` (retired v3) |
| `l1-ratchet.sh` | D | counts L1 type knowledge in `src/v3/*.dag` (retired v3) |
| `r3-debt-velocity.sh` | D | reporting-only; explicitly "does not gate CI" |
| `r1_p0_no_fabrication_sentinel.sh` | D | header itself says "v2 tree removed under T-V2-Retirement G-2" |
| `check-r4-carve-dissolution-discipline.sh` | C | prose-grep doc-hygiene; demote from required |
| `check-rust-toolchain-single-authority.sh` | C | header itself names dissolution: "delete this script if extdeps + workflow toolchain selection are generated" |
| `check-test-timeout.sh` | C | reads pre-captured test log; dev-discipline check, demote from required |
| `check-manager-brief-authority.sh` | C | prose-grep; pre-commit hook material, not CI gate |
| `check-release-doc-authority.sh` | C | prose-grep; pre-commit hook material |
| Four-compile redundancy in `ci_v4` | A/D | collapse to ONE Class-A run; demote the other 3 copies to scheduled |
| **All v3 integration tests** (`ci_integration` Tier-0 I3 `cargo test -p v3-compiler --release --test determinism_test`; full `v3` job: `cargo test --lib --bins`, `clippy --features bootstrap-regen-fresh`, all v3 lib/bin tests) | **D — PERMANENT** | **Operator directive 2026-06-01**: v3 is retired; we aren't supposed to rely on these. Delete from CI permanently (not move to scheduled — DELETE). Per project spirit + #4101 bankruptcy framing. Confidence-on-v3 is not a v4 ship gate. |

#### §11.7.4 The four-wave plan

**Wave 1 — CI Cut / Survival Floor** (CI Manager + Compiler Spine)
- Deliverable: `ci.yml` required path reduced to safety floor (Class A only); all removed/demoted checks recorded in honesty ledger
- Acceptance: cold PR CI target materially shorter; required gates explicit; removed coverage named
- Output: `docs/planning/ci-required-surface-cut-2026-06-01.md` (the bankruptcy doc) + `.github/workflows/ci.yml` (cut) + `src/v4/workflow/ci.dag` (any modeled-receipt updates)
- **This is triage, not a modeling victory** — operator-ratified honest reduction

**Wave 2 — Shell-to-CiUpsertStep migration** (CI Manager)
- Take ~26 unmigrated YAML steps; migrate highest-value first per W2.3 worksheet pattern (#4078)
- Each step → `CiUpsertStep<T>` with typed `UpsertInputRef` inputs, projected verify/create/resolve, derived cache digest
- Forbidden: hand-edited GHA `if:` conditions, parallel selectors, string-keyed deps, authored cache keys, heuristic run modes

**Wave 3 — Testgen / Affected-Set Shadow Integration** (Runtime/TestClaim Manager + CI Manager)
- For every PR diff: affected-set outputs graph regions → testgen maps regions → TestClaims → CI prints selected claims (does NOT skip floor)
- Broad safety floor STILL runs
- Receipt-only mode; comparison with Class A determines trustworthiness over time

**Wave 4 — First Active Modeled Skip** (only after Wave 3 receipts trustworthy)
- Narrow first skip: docs-only OR leaf-model-only OR one narrowly scoped fixture
- Skip must return structured cached/previous verdict, not just "did not run"
- Requires: step deps modeled as `CiUpsertStep` inputs; selected TestClaims correspond to actual affected graph regions; false-negative cases tested; broad floor remains

#### §11.7.5 The temporary CI cut rule (codified)

```
A shell-owned check may remain REQUIRED only if:
  1. it protects the current P2/P3/P5 safety floor,
  2. it has no modeled CiUpsertStep replacement yet,
  3. it is listed in the temporary shell exception table,
  4. it has a named dissolution path.

All other shell-owned checks move to shadow/manual/scheduled or are deleted.
No affected-set-exclusive skip is allowed until shadow receipts prove
the selected TestClaim set against a broad safety floor.
```

#### §11.7.6 Honesty ledger requirement

The bankruptcy PR (Wave 1) must name explicitly:
- Required PR gates after cut (list)
- Moved to shadow/manual/scheduled (list)
- Deleted until modeled (list)
- Restoration condition per item (must return as `CiUpsertStep` or TestClaim/testgen-derived gate)

Same posture as v4 alpha release: **reduced required confidence is acceptable if the doc says required confidence is reduced** — not hidden behind "CI is green".

#### §11.7.7 CI Manager REFRAMED charter (supersedes §11.2 CI row)

**Charter (one sentence)**: get the bare minimum safety net working in CI using affected-set + testgen, instead of maintaining shell-owned coverage.

This is a different job than "migrate every shell step to `CiUpsertStep`". The migration is a means to an end; the end is: **affected-set + testgen-driven safety net IS the CI runtime authority**, and the bare minimum (§11.7.1) is what runs on every PR with that authority providing intelligent selection.

The CI Manager's work lane:

| Deliverable | Order | Why |
|---|---|---|
| 1. **CI Coverage Bankruptcy PR (Wave 1)** | First | Cut required surface to §11.7.1 bare minimum; publish honesty ledger; permanently delete v3 integration tests per operator directive |
| 2. **Four-compile redundancy collapse** | Parallel with Wave 1 | ~14m savings; biggest single runtime drop available without modeling work |
| 3. **Affected-set + testgen shadow integration (Wave 3 with Runtime/TestClaim)** | Second | Shadow mode: for every PR, affected-set outputs graph regions → testgen maps to TestClaims → CI prints selected claims → does NOT skip floor. Build trust over time. |
| 4. **Affected-set + testgen FIRST ACTIVE skip (Wave 4)** | Third | Once Wave 3 receipts are trustworthy, first narrow active skip: docs-only OR leaf-model-only. Skip returns structured cached verdict, not "did not run". |
| 5. **Affected-set + testgen as Class A safety net** | Long horizon | The end state: the bare minimum (§11.7.1) runs PLUS affected-set/testgen selects additional class-B gates per PR diff. The "safety net" IS dynamic — broad on risky changes, narrow on docs-only. Hand-authored `if:` conditions retire. |
| 6. **Shell-to-CiUpsertStep migration** | Continuous background | Modeled-migration of remaining ~26 steps happens INCREMENTALLY in service of Wave 3/4/5. NOT a separate front. Each step migrated when affected-set needs it as a typed dependency. |

**Explicitly NOT the CI Manager's job** (anti-patterns to avoid):
- Migrating every YAML step to `CiUpsertStep` for completeness sake — only migrate what affected-set + testgen needs
- Maintaining shell-owned ratchets, leaf-model verifications, prose-grep checks — those are CUT per §11.7.3, not migrated
- Pretending affected-set is ready for exclusive gating before Wave 3 receipts prove it

**Exit criteria (updated)**:
- (a) Wave 1 bankruptcy cut on main; honesty ledger published (`docs/planning/ci-required-surface-cut-2026-06-01.md`)
- (b) Cold PR CI runtime ≤10min on §11.7.1 bare minimum (was ~30min warm, ~75min cold with rerun)
- (c) Wave 3 shadow receipts trustworthy across N PRs (operator-defined N)
- (d) Wave 4 first active modeled skip on main with structured cached verdict
- (e) Four-compile redundancy = 1 compile
- (f) v3 integration tests permanently deleted from CI (not moved to scheduled)

**Long-pole (gated on P5 runtime gate)**: runtime authority migration (YAML→evaluator); same SELF_HOSTING substrate work as `adhoc-f8699326-d69`. Not on critical path for CI cut; arrives when bootstrap-evaluator corpus runtime ships.

### §11.8 Language self-compile runway (operator-ratified 2026-06-01)

**Goal**: every supported language target has a credible ladder toward "self-compile / compiler-subset execution". Not vague ambition — explicit rungs L0-L4 per language, with per-language manager scope.

**Critical-path framing**: Rust is the only release-critical self-compile target (P3 → P4 → P6 chain). Go is the next-best candidate (clean external verifier via `go build`; static-ish binary). Python is release-minimum parity but with weaker static verification. TypeScript is alpha/preview lane (not release-minimum per Wave F F3).

#### §11.8.1 The five rungs (per-language)

| Rung | Bar | Receipt |
|---|---|---|
| **L0** | Leaf-model verification: language model claims checked against real toolchain | TestClaim receipts on `language.dag` R1/R2a/R2b/R3-external (the pattern landed for Rust via #4000/Python via #4117) |
| **L1** | Compiler-subset emit: emit v4 compiler subset to target language; toolchain typechecks/compiles | rustc/tsc/pyright/go-build receipt on emitted subset (target-toolchain-clean) |
| **L2** | Compiler-subset execution: emitted target compiler runs on small .dag fixture, output matches Rust/.dag eval | Cross-target equivalence receipt (analog of #4081 Wc L5 cross-target on nat_semiring) |
| **L3** | Self-output / fixed point: target-emitted compiler emits itself or declared compiler slice, bit-identical to reference | T-15-style self-host fixed-point receipt for the target |
| **L4** | Full self-compile: target backend is credible compiler-of-record path | P3-analog for the target language; full v4 compiler closure compiles via target toolchain |

#### §11.8.2 Per-language manager scope + current rung

| Language | Manager scope | Current rung | Target rung (by end of next iteration) |
|---|---|---|---|
| **Rust** | Critical-path self-compile target; owns R1/R2a/R2b/R3 + emit residual closures + binary build path | **L1 in progress** (#4140 7,724 rustc residual being driven to 0); L2/L3/L4 gated on P3 binary build | L1 complete (binary builds); L2-L4 sequenced per P3+P4+P6 cascade |
| **Go** | Release-minimum parity + future self-compile candidate; clean external verifier (`go build`); static-ish binary advantage | **L0** (Python R1 analog in flight; Go emit fix #4076 landed) | L0 complete + L1 fixture-scale; sequence toward L2 fixture-execution |
| **Python** | Release-minimum parity; weaker static verifier (use `pyright`/`mypy` + runtime fixtures); cross-target behavioral parity emphasis | **L0 complete** (#4117); **§8 CLOSED** (#4170) — L1/L2 worksheets at `docs/planning/v4-python-rca-manager-worksheets-2026-06-01.md`; dispatch authority: `docs/planning/v4-active-authority-map-2026-06-01.md` (A ready; B/C blocked on Runtime/TestClaim) | L1 fixture-scale + L2 cross-target behavioral parity receipts (vs Rust/Go) |
| **TypeScript** (alpha/preview) | Alpha/preview lane; not release-minimum; needs `tsc` + Node runtime semantics + module-layout modeling | **L0** (worksheets not started) | L0 complete per #4117 pattern: R2a/R2b/R3-external leaf-model widening + TargetAtomRealization + TargetTypeExpressionProjection + grammar-inverse TestClaims |
| **C++** (later) | Capacity-permitting; produce language residual matrix only; no impl until shared substrate authority resolved | **none** | none unless operator authorizes; matrix only |

#### §11.8.3 Mechanical rules (preserved from §11.1)

- Each language class needs its own §10.0 worksheet (single-authority fact rule)
- Shared facts (e.g., `TargetTypeExpressionProjection` appearing in Rust + Python + Go + TS) go through Modeling DFS Arbiter — NOT per-language reinvention
- Language manager finds symptoms; shared substrate managers own shared facts
- Per-language verification fixtures are part of each rung's exit criteria (TestClaim receipts, not "compiles")

### §11.5 PM (this session) scope under new structure

PM remains routing + escalation only:
- Spawn new managers + dispatch their initial briefs
- Receive sign-off requests; route to operator OR Modeling DFS Arbiter
- Run cron tree-health checks; surface anomalies + receipts
- Maintain dep graph snapshot (this doc + successors)
- **DO NOT act as Modeling DFS Arbiter** (single-authority discipline)
- **DO NOT approve substrate** (Arbiter only)
- **DO NOT spawn implementation workers without ratified worksheet** (mechanical rule)
