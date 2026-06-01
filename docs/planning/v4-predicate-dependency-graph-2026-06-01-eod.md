# v4 predicate dependency graph — 2026-06-01T00:30Z (Jun 1 release day EOD)

Forward projection of remaining work to v4-done as of Jun 1 release day end-of-day. Supersedes `docs/planning/v4-predicate-dependency-graph-2026-05-31.md` (#4096) for current state. Predicate definitions at `src/v4/TASKS.md:806-817`.

## §1. Six v4-done predicates — current state

| # | Predicate (TASKS.md:806-817) | State |
|---|---|---|
| P1 | Every other scheduled task complete | YELLOW (~8-12/53 PROVEN; R3-internal added; per-lane GAP continues; no near-term GREEN candidate) |
| P2 | v4 compiles `src/v4/compiler/*.dag` end-to-end | **TECHNICAL-PROVEN; DELETION IN FLIGHT**. P2-A probe (44-source closure 0 diagnostics) verified scope (a). M2 probe (#4097) confirmed bridge dead-weight. **Operator authorized early deletion 2026-06-01T00:28Z** (skipping 14-day window); work-item `adhoc-4cad7e9b-558` spawned for `scripts/v4-bootstrap-resolve-posture-gate.sh` + `ci.yml` step removal. P2 flips fully GREEN on that PR merge. |
| P3 | v4 emits Rust source that compiles to a binary | YELLOW (downstream-cascading). **9 of 9 routed classes closed or worksheet-on-main** today: SG-1 ✓ SG-7 ✓ SG-5 ✓ SG-6 ✓ SG-1b ✓ SG-2 ✓ SG-RC-LAYERING ✓ SG-3-cascade-retired ✓ SG-8 worksheet ✓ (impl gated on §8 sign-off). Residual rustc count stale at 7,175 (#4122 post-#4115 catalog); fresh probe needed post-cascade. Two minor follow-ons remain: SG-1-FOLLOWON (per amend), SG-COLLECTION-PROJECTION (~170, deferred amendment). |
| P4 | Binary on `src/v4/compiler/*.dag` produces bit-identical output | RED — hard-gated on P2 full PROVEN + P3 binary builds |
| P5 | TestClaim suite passes | **Layer 1 + Layer 2 BOTH CLOSED** via #4115 (P5 structural-bridge replaced + deleted). Strict P5 "suite passes" still gates on runtime-execution path: upward debt `node://adhoc-f8699326-d69` (operator decision: M1 cargo-clean subset OR new bootstrap-evaluator corpus runtime per SELF_HOSTING). T-38-PR2 verdict-SURFACE migration LANDED #4120 (closes one of three closeout-leaf conditions for stern-lynx). |
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

### §3.2 P2 — TECHNICAL-PROVEN / AUTHORITY-BLOCKED

**Layer 1 (technical):** PROVEN via P2-A probe.

**Layer 2 (authority):** OPEN. Two paths to GREEN:
- **(a) Operator authorization**: explicit `gh pr` deleting `ci.yml:378-385` + `scripts/v4-bootstrap-resolve-posture-gate.sh`. M2 probe (#4097) provides safety-net receipt. Today is day 1 of 14 per script header.
- **(b) Wait for 14-day window**: removal trigger met on main CI for 14 consecutive days (either v2-compiler typed resolve gate OR v4 emit reaches `compiled:` without SIGTERM).

PM cannot resolve; operator decision.

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
| SG-COLLECTION-PROJECTION | deferred | amend SG-5/SG-6 first | ~170 errors; tactical follow-on |

**P3 PROVEN bar reminder**: Rust source compiles to a binary (not zero rustc errors as goal; not Python+Go alpha-targets). Each closure shrinks residual; binary builds when residual hits 0 on Rust path.

**Stale residual count**: 7,175 errors (#4122 catalog at 16:18Z, before SG-1b + SG-2 + SG-RC-LAYERING + SG-8 worksheet landed). **Fresh probe needed** to reflect post-cascade state — likely meaningfully lower.

### §3.4 P4 bit-identical fixpoint

Hard-gated on P2 full PROVEN + P3 binary builds. No active worker dispatchable.

### §3.5 P5 TestClaim suite passes — Layer 1 + Layer 2 BOTH CLOSED

**Layer 1 — Fixture/law bundle: 3/3 CLOSED** (Wa-1 #4079 + Wa-2 #4080 + P5-D tranche-2 #4089).

**Layer 2 — Authority gate: CLOSED via #4115** (positive-Y CiUpsertStep replaces shell; `scripts/v4-testclaim-corpus-gate.sh` deleted).

**T-38-PR2 verdict-surface migration LANDED #4120**: `blocked_m1_subset` string retired; `host_verdict_surface_receipt_v3` with per-row Pass/Fail/Deferred counts.

**Strict P5 "suite passes" reading still has upward debt** (`node://adhoc-f8699326-d69`): true runtime corpus execution gated on EITHER (i) M1 cargo-clean emitted subset OR (ii) new bootstrap-evaluator corpus runtime per SELF_HOSTING. Operator decision pending. (i) is downstream of P3 cascade closures; (ii) is load-bearing substrate work.

### §3.6 P6 hand-Rust REPRODUCTION

Hard-gated on P4 + P3.

## §4. Critical paths forward — post-cascade

**Path A: P5 strict "suite passes" (runtime gate)** — operator decision required between (i) and (ii). (i) is patient-wait-for-P3-cascade. (ii) is dedicated DFS worksheet authoring. Either flips strict P5 GREEN.

**Path B: P2 GREEN flip** — operator decides whether to authorize bridge deletion now (with M2 probe as safety-net receipt) OR wait for 14-day window. Authorization-only; no implementation work needed.

**Path C: P3 binary build (residual march)** — SG-8 impl worker can spawn once §8 sign-off resolves; SG-1-FOLLOWON + SG-COLLECTION-PROJECTION minor follow-ons; fresh M1 probe to reflect actual post-cascade count. Each class closure shrinks residual; binary builds when ≈0 on Rust path. Long-horizon.

**Path D: P1 + P4 + P6 cascade** — all gated on P2 + P3 above.

## §5. Active workers / no work

Tree is at 0 active subtree work as of 00:30Z 2026-06-01. All five overnight subtree PRs (#4116, #4127, #4133, #4135, #4101) merged; #4112 operator-landed. Managers + closeout-leaf placeholders idle. Subtree health: 0 working / 7 idle / 0 blocked / 0 active PRs.

## §6. Active blockers (post-EOD)

| Blocker | Owner action |
|---|---|
| **P2-B bridge deletion authorization** | Operator decision OR wait for 14-day window (day 1 = today via #4097 M2 probe) |
| **P5 runtime gate** | Operator decision on `node://adhoc-f8699326-d69` choice (i) vs (ii) |
| **SG-8 §8 sign-off** | Modeling DFS authority ambiguous post-proud-pike-archival; need operator routing decision OR PM/keen-heron to act as DFS reviewer |
| **Fresh M1 probe post-cascade** | sharp-otter to re-measure after today's 7 class closures landed |

## §6.5 Current Dispatch Board

| Priority | Item | State | Owner | Exit receipt |
|---|---|---|---|---|
| 1 | P5 runtime gate decision | operator-decision-node | operator | choice (i) or (ii) on adhoc-f8699326-d69 |
| 2 | P2-B bridge deletion authorization | operator-decision-node | operator | authorize delete OR wait window |
| 3 | SG-8 §8 sign-off authority | structurally ambiguous | operator routing OR PM/keen-heron-acting | §8 ratification → impl worker dispatchable |
| 4 | Fresh M1 probe post-cascade | PM dispatchable | sharp-otter | new rustc residual count + per-class delta |
| 5 | SG-1-FOLLOWON impl | routed | TR lane (keen-heron) | minor follow-on close |
| 6 | SG-COLLECTION-PROJECTION amend | deferred | TR/proud-pike-successor | ~170 errors |

## §7. PM-side actionable items

**No worker implementation dispatches blocked** — most P3 classes closed; SG-8 impl needs §8 routing decision (operator) before spawn.

**One PM dispatch available** if operator authorizes: fresh M1 probe post-cascade (sharp-otter). This would give a clean post-cascade rustc residual count to inform P3 path-C planning.

**Operator decisions blocking forward progress**:
- P5 runtime gate (i) vs (ii) on adhoc-f8699326-d69
- P2-B bridge deletion authorization (or rely on 14-day window)
- SG-8 §8 sign-off authority routing (PM acts as DFS / spawn new DFS Manager / ratify keen-heron expanded scope)

## §8. Risk / honesty

- **P3 cascade was massive but residual is still significant**: 7,175 rustc errors at last probe (16:18Z), before SG-2 + SG-1b + SG-RC + SG-8 worksheet landed. Per-class wins are smaller than SG-1's 37% magnitude per #4096 §8 honesty. Realistic horizon: weeks-to-months for residual → 0.
- **P5 single authority**: P5 is **GREEN only when the canonical TASKS.md predicate "TestClaim suite passes" (`src/v4/TASKS.md:816`) is satisfied** — which requires the runtime-execution gate to close. Today's Layer 1 + Layer 2 closures are **partial progress toward P5**, not an alternate definition of GREEN. Surfaced explicitly by T-38-PR2 worker's honest scope analysis (cool-boar-841 identified 3 readings, chose surface migration + escalated runtime gate as upward debt at `node://adhoc-f8699326-d69`; correct discipline per project_spirit "stop and escalate"). Do not treat partial layer closure as P5 GREEN.
- **proud-pike archival mid-day** consolidated Modeling DFS lane under keen-heron de facto. §8 sign-off authority needs explicit operator routing for SG-8 impl worker dispatch.
- **#4112 operator-landed manually** despite dashboard recommending self-merge — operator chose to handle that specific PR, others all self-merged per current policy.
- **One major gap**: no fresh post-cascade rustc residual count. Without #4122-successor probe, we don't know the real P3 residual after today's closures.

## §8.5 Operator sign-offs queue (consolidated)

Captures everything PM is waiting on operator to confirm/decide. Updated as items resolve.

| # | Decision | Status | Notes |
|---|---|---|---|
| 1 | P2-B bridge deletion | **AUTHORIZED 2026-06-01T00:28Z** (early deletion, skip 14-day window) — work-item `adhoc-4cad7e9b-558` in flight | Flips P2 GREEN on merge |
| 2 | Fresh M1 probe post-cascade | **AUTHORIZED 2026-06-01T00:28Z** — work-item `adhoc-7b46e080-3cd` in flight (sharp-otter archived; fresh worker spawned) | Gives clean post-cascade P3 residual count |
| 3 | P5 runtime gate (i) vs (ii) | PENDING | Choice (i) wait for P3 cascade → M1 cargo-clean subset emerges naturally; (ii) author bootstrap-evaluator corpus runtime (load-bearing per SELF_HOSTING; needs ratified Modeling DFS worksheet first). On `node://adhoc-f8699326-d69` |
| 4 | SG-8 §8 sign-off authority routing | PENDING | Post-proud-pike archival. (a) PM acts as Modeling DFS; (b) spawn fresh Modeling DFS Manager session; (c) ratify keen-heron-687 expanded scope (Modeling DFS + TR). Blocks SG-8 impl worker dispatch. |
| 5 | Modeling DFS Manager succession plan | PENDING (same as #4 one level up) | Once #4 resolved, this is implicit |
| 6 | TypeScript / non-release-minimum work plan | PENDING | Operator authorization to dispatch TS class-closure work in parallel with Rust residual (TR-lane capacity); per Wave F F3 TS is v4-alpha-only |
| 7 | stern-lynx Runtime/TestClaim Manager closeout | PENDING | Cond (a) ✓ + (b) ✓ already; only (c) "operator confirms role complete" remains |

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
- Predicate defs: `src/v4/TASKS.md:806-817`

## §10. Watchlist (Jun 1 release-day eod, no time-based ETAs)

**Operator decisions pending (gate forward progress):**
- P2-B bridge deletion authorization
- P5 runtime gate choice (i) vs (ii) on `adhoc-f8699326-d69`
- SG-8 §8 sign-off authority routing

**PM dispatches available pending operator authorization:**
- Fresh M1 probe post-cascade (sharp-otter)

**Receipts available for review:**
- All today's class closures on main; predicate-state movement substantial across P3 + P5 layers
- No active worker awaiting attention

**Jun 1 release readiness summary:**
- v4 ships alpha per Flavor (iv) operator framing (per session memory `project_june1_release`)
- Substrate landings: massive day (SG-1/2/5/6/1b/RC-LAYERING + SG-3-retired + SG-8-worksheet + #4115 P5 + #4120 T-38-PR2 + W2.3 Bucket E + #4091 elastic CI ratification + compute fabric + cache substrate)
- **P5 (canonical "TestClaim suite passes" per TASKS.md): NOT fully GREEN today** — Layer 1 + Layer 2 closed but runtime-execution gate remains open at `node://adhoc-f8699326-d69` (operator decision pending on path (i) M1 cargo-clean subset vs (ii) bootstrap-evaluator corpus runtime). P5 flips fully GREEN only when runtime-execution gate closes; do NOT treat as done.
- P2 awaits operator authorization OR 14-day window completion
- P3 multi-class cascade closed today but residual rustc count significant; long-horizon to binary builds
