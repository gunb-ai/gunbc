# R3 Cluster M Phase 2 — Verification Discipline #87 Dispatch (2026-05-09)

**Owner**: Verification Mgr (wise-bear-525 / gunbc#2075)
**Authority**: PM-tier dispatch coordination per Director ratification at gunbc#846 #issuecomment-4412309986 (Director answered Ask 1 with "(γ) hybrid"). Sequencing structure lives at [`docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md`](../audit/r3-cluster-m-sequencing-plan-2026-05-09.md) §4 (in-flight via concurrent PR #2361; this brief is the dispatch overlay — substantive content here is self-contained and grounded in locked-design + live ledger authorities below). This brief is a light-touch dispatch surface.

---

## §0. Scope

**Gate #87** `lens_cementing_test_discipline_complete` — Cluster M Phase 2. Verification Mgr authors discipline-pattern + first cementing-test migration receipt.

## §1. Substrate

**Locked design authority**: [`docs/design-tests-as-data-completeness.md`](../design-tests-as-data-completeness.md) §C5 (Cementing v2 oracle) — canonical predicate set for cementing-test discipline.

**Existing brief at HEAD**: [`docs/briefs/r3-v-tests-as-data-v1-worker.md`](r3-v-tests-as-data-v1-worker.md) — **PRE-AUTH DISPATCH-READY** (tier-1 queue gunbc#1859). Covers all four Cluster M gates (#84/#85/#86/#87) in single-coordinator framing. Verified live on main: `git ls-tree origin/main -- docs/briefs/r3-v-tests-as-data-v1-worker.md` → blob `4ff9abcb1b8b`.

**This dispatch shape (γ) splits the existing brief by gate-class**:
- #85/#86 → Substrate Mgr canvas-tier (authored under standing authority; see [`docs/briefs/r3-cluster-m-dispatch-substrate-canvas-asks-2026-05-09.md`](r3-cluster-m-dispatch-substrate-canvas-asks-2026-05-09.md))
- **#87 → Verification Mgr discipline-authoring (this brief)**
- #84 → Verification Mgr bulk-port coordinator (see [`docs/briefs/r3-cluster-m-dispatch-verification-bulkport-84-2026-05-09.md`](r3-cluster-m-dispatch-verification-bulkport-84-2026-05-09.md))

The existing `r3-v-tests-as-data-v1-worker.md` is cited as substrate-of-truth for #87 substantive content; this dispatch brief is the (γ)-hybrid coordination overlay.

## §2. Dispatch trigger

Verification Mgr dispatches #87 discipline-authoring **on Phase 1 partial-land** — i.e., once at least one of #85/#86 substrate canvases has Director ratification on carrier shape (does NOT require both #85 and #86 carriers to fully land; Director-ratified shape questions are sufficient to begin discipline-pattern authoring).

Pattern precedent: discipline patterns can be authored against substrate-shape ratifications before substrate-implementation lands; the discipline pattern *consumes* the carrier shape, not its implementation.

## §3. Authoring scope

Per `r3-v-tests-as-data-v1-worker.md` substrate + Cluster M sequencing plan §4:

- **Discipline pattern**: how every `.dag` lens has at least one cementing test in `.dag` form using #85 `Quantifier` + `QuantifiedTestClaim` carriers (per locked design §2.2) + #86 `ProgramGenerator` carrier (per locked design §2.1)
- **First migration target**: smallest hand-Rust cementing test (e.g., `tests/integration/cementing/cementing_lens_registry_dispatch_test.rs`) — proof-of-concept migration from hand-Rust `#[test] fn test_X` with v2-oracle assert → `.dag` `TestClaim` with **same-source v2-vs-v3 comparison** via `DifferentialEquals` or `LensOutputEquals` predicate (per locked design `docs/design-tests-as-data-completeness.md` §C5 row, §"C5: Cementing (v2 oracle)" — "v2 oracle and v3 lens produce equal output on source S"). `BinaryDimensionReportEquals` is for Pattern-A DimensionReport comparisons (TC1/TC2/TC3 family), NOT for cementing — different predicate axis.
- **Receipt**: state-check gate fires when audit confirms 100% lens coverage in cementing-test form using the locked-design predicates

## §4. Receipt

- #87 status moves DECLARED → CONSUMER_LANDED on first migration receipt (proof-of-concept migration lands)
- #87 status moves CONSUMER_LANDED → PASSING when audit confirms 100% lens coverage in `.dag` cementing-test form
- §1.8 ledger Status updated by Verification Mgr per §10 cadence

## §5. Velocity context

Per [`docs/audit/r3-pb0-velocity-walk-2026-05-09.md`](../audit/r3-pb0-velocity-walk-2026-05-09.md) §3.3: ~20-25 of the test entries in `EXPECTED_HAND_AUTHORED_TEST` (live count authoritative at `src/v3/compiler/tests/integration/sg0_census_test.rs`) dissolve via #87 cementing-test discipline landing (cementing-test class is the largest single class). Phase 2 is therefore a substantial bulk-dissolution sub-event en route to #84 close.

Phase 2 sequencing-criticality: M → B → E (per PR #2300 §4 Risk 2 + this audit §5 Risk 4) — B's cementing tests must capture frozen v2-oracle snapshot via M's discipline before E (v2 retirement) fires.

---

**End of brief.**
