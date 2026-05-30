# v4 progress snapshot — 2026-05-30T22Z

PM-authored visibility doc per operator ask 22:30Z. Complements `docs/planning/v4-done-six-predicate-burn-down-2026-05-30.md` (merry-badger-222's predicate-level tracker) with a wave-level + immediate-momentum lens.

For deeper per-predicate analysis with fresh rustc error count, see merry-badger-222's incoming amendment to the burn-down (dispatched via msg_bc66155b, ETA ~1h from 21:54Z).

## Headline

- **0 / 6 v4-done predicates PROVEN** (per #4021 / #4030 burn-down). 5 YELLOW (P1–P5 advancing), 1 GRAY (P6 gated on P4 + P3).
- **~7,951 rustc errors** baseline on full-tree v4 emit (per `docs/audit/v4-rustc-error-catalog-2026-05-29.md`). Today's Wave 2 substrate landings (SG-2 #3962, SG-7 #4014, Upsert<T> #3981, CiUpsertStep #3989, SG-5 #3957) unblock the dominant closers; SG-1 #3956 is the largest unlanded Pareto closer (~2,978 E0423 errors expected closed when it lands).
- Wave 1 MW-D8 exit conditions: **4 / 5 MET** (only `ci_selection_receipt_shadow` W1.5 remaining; smart-stag-871's queue).
- Wave 2 closure: **W2.1 SG-1 in flight (draft) • W2.2 SG-5 ✓ MERGED 22:23Z • W2.3 worksheet authoring (proud-pike) • W2.4 gated on W2.3 • W2.5 ✓ COMPLETE (3 phase4 fixtures × rungs 0-2/3/4) • W2.6 ✓ MERGED (python.dag verification)**.
- Release posture: **flavor (iv) — alpha / WIP labeled, no Wave-2 gating**. v4 ships public AS-IS in v0.1.0 with honest disclaimers per `docs/RELEASE_v0.1.0.md` (#3991 merged), `docs/SUPPORTED.md` (#4025 merged), and README swap-line disclaimer (#4031 merged).

## Wave 2 status (per merge wave doc §5)

| # | Item | Owner | State | Evidence |
|---|------|-------|-------|----------|
| W2.1 | SG-1 TargetAtomRealization | Target Realization (keen-heron / zesty-carp-242) | IN FLIGHT, draft | #3956 (draft, 10 CI passing, no approvals yet) |
| W2.2 | SG-5 TargetCollectionRealization | Target Realization (keen-heron / vivid-heron-767) | ✓ MERGED 22:23Z | #3957 |
| W2.2 | SG-6 BoundedLattice realization | Target Realization | NOT DISPATCHED (gated on SG-1) | — |
| W2.3 | Phase 1.5 CiUpsertStep migration worksheet | Modeling DFS (proud-pike-680) | IN FLIGHT, worksheet ~4h ETA | — (worker dispatch follows) |
| W2.4 | Phase 1b A3–A14 atom migration | Compiler Spine (smart-stag-871) | GATED on W2.3 maturity | — |
| W2.5 | Phase 4 fixture widening | Ladder/Fixture (wise-seal-69 → archived; royal-wolf-898 → archived) | ✓ COMPLETE | #4018 + #4028 + #4034 + #4035 + #4036 + #4037 + #4038 + #4039 (3 fixtures × rungs 0-2/3/4) |
| W2.6 | Cross-target python.dag leaf-model verification | Modeling DFS + TR + Runtime/TestClaim (quick-tern-735 / warm-tern-791) | ✓ MERGED 21:05Z | #4022 |

Wave 2 is ~85% closed. Remaining: SG-1 #3956 to land, SG-6 to dispatch (after SG-1), W2.3 worksheet + worker dispatch + landing, W2.4 (gated on W2.3 maturity, deepest dependency chain).

## MW-D8 Wave 1 exit conditions

| # | Condition | Status | Evidence |
|---|-----------|--------|----------|
| 1 | R1 produces leaf-model verdict (rust.dag R1 → rustc → Verdict<R1>) | ✓ MET | #3972 |
| 2 | SG-7 ci.dag recursion dissolved | ✓ MET | #4014 (implementation; #3977 worksheet earlier) |
| 3 | Upsert<T> landed as substrate primitive | ✓ MET | #3981 + #3989 (Phase 1.4 + 1.5) |
| 4 | `ci_selection_receipt_shadow` generatable for ≥1 PR fixture | ✗ NOT STARTED | smart-stag-871 queue (W1.5) |
| 5 | R2a/R2b/R3-external/R3-internal authoring ready-to-run or named-blocked | ✓ MET | #4000 (R2a/R2b/R3-external landed; R3-internal explicitly-blocked-and-named on SG-1) |

Wave 1 exits to MW-D8-CLOSED when W1.5 ci_selection_receipt_shadow ships (the only remaining condition).

## v4-done six-predicate burn-down (per `docs/planning/v4-done-six-predicate-burn-down-2026-05-30.md`)

All status as of #4030 + landings since:

| Predicate (TASKS.md:801–815) | Status | Recent advances | Named blockers |
|---|---|---|---|
| P1 every-other-task | YELLOW | substrate widening across multiple lanes today | broad cross-task closure not yet measured |
| P2 corpus-compiles (rustc on full-tree v4 emit) | YELLOW | SG-2 #3962 + SG-5 #3957 substrate landed; SG-1 #3956 imminent | SG-1 landing → ~2,978 E0423 closed; remaining error tail unmeasured until fresh `cargo check` run |
| P3 emit-compiles (per target) | YELLOW | python.dag verification path landed (#4022); R1 rust verdict (#3972) | weather → Python/Go currently fails (PRs #4040 / #4041 in flight to fix) |
| P4 bit-identical-self-output (T-15) | YELLOW | unchanged today | T-15 self-host fixed-point work; not on critical path today |
| P5 TestClaim-suite-passes | YELLOW | R2a/R2b/R3-external verdicts (#4000); python.dag R1 verdict (#4022); fixture widening (#4018 + #4028) | broader suite-pass measurement unmeasured |
| P6 hand-Rust-not-editable-authority-proven-by-reproduction | GRAY | gated on P4 + P3 | — |

**Realistic Jun 1 GREEN forecast**: 0 / 6 by tag time. Per flavor (iv), this is honest alpha framing, not a release blocker.

## Today's substrate landings (Wave 1 + Wave 2)

PRs merged 2026-05-30 in chronological order. This is the substrate that ships with v0.1.0 alpha / WIP:

- Wave 1 closure: #3947 (Compiler Spine × Runtime min runner interface) + #3958 (W2 host harness rungs 3–4) + #3972 (R1 leaf-model rust verification) + #3977 (SG-7 worksheet) + #3981 (Upsert<T>) + #3989 (CiUpsertStep) + #3960 (RoundTripClaim eval path) + #4000 (R2a/R2b/R3-external widening) + #4014 (SG-7 ci.dag dissolution) + #4015 (nat_semiring rung gate alignment) + #3996 (emit_data_def Rc-wrap fix)
- Wave 2 substrate: #3962 (SG-2 TargetTypeExpressionProjection) + #3957 (SG-5 TargetCollectionRealization) + #3989 (Phase 1.5 CiUpsertStep substrate) + #3998 + #3999 + #4001 (M6 enforcement composite: review-gate doc + structural_similarity lens + IdenticalVariantPayload sub-signature) + #4022 (W2.6 python.dag verification) + #4018 + #4028 + #4034–#4039 (W2.5 Phase 4 fixture widening, 3 fixtures × rungs 0-2/3/4)
- Release prep: #3859 (release.yml + 6-target binary matrix, earlier) + #3911 (install.dag substrate) + #3992 (install.sh resurrection) + #3994 (README weather hero) + #4024 (src/v4 PM-jargon scrub) + #4023 (v4 ship-disposition supplement) + #4025 (SUPPORTED.md) + #4031 (README swap-line disclaimer) + #4032 (close/receipt §1.5 run-status discipline + weather verification) + #3991 (v0.1.0 consolidated state snapshot, flavor (iv)) + #4009 (RELEASE_TODO strip) + #4017 (MW-D8 ledger)
- Tracking docs: #4013 + #4021 (v4-done burn-down) + #4030 (Wave 2 maintenance burn-down) + #3983 (merge wave plan) + #3938 (correctness ladder + manager lane architecture) + #3946 (rung 0-2 specs) + #4003 (rung 3 spec) + #3990 (rung 4 spec)

That is roughly 40 v4-relevant PRs landed on 2026-05-30, of which the Wave 1 + Wave 2 substrate + release prep are the load-bearing fraction.

## What's in flight right now

- PR #4040 Python emit fix (`session/emit-python-tco-fix` — match-as-expression + TCO temp-decl) — DRAFT, 10 CI passing. Workers gate on weather.dag + nat_semiring per smart-stag's brief. ETA 2-3 working days. Operator picked option (X) — README disclaimer in place now; fix lands in v0.1.1.
- PR #4041 Go emit fix (`session/emit-go-layout-fix` — multi-file layout + := scope) — DRAFT, 10 CI passing. Same gates. ETA 1-2 working days.
- PR #3956 SG-1 TargetAtomRealization (`session/zesty-carp-242-sg1-target-atom-realization`) — DRAFT, 10 CI passing, 0 approvals. Dominant Pareto error-closer when it lands.
- Wave 2 worksheets: W2.3 Phase 1.5 CiUpsertStep migration (proud-pike) + W2.6 python.dag leaf-model spec (proud-pike). Worksheets gate worker dispatch.
- W1.5 `ci_selection_receipt_shadow` (smart-stag) — last MW-D8 condition unmet. Drafts pending.

Nothing else is dispatch-ready until either (a) W2.3 worksheet lands → worker dispatch on Phase 1.5 step migration (W2.3 / W2.4) OR (b) SG-1 #3956 lands → SG-6 dispatch (W2.2 closure) OR (c) Wave 3 framing ratified.

## Wave 3 candidates (per merge wave doc §6)

None dispatchable until Wave 2 closes (W2.3 + W2.4 outstanding; SG-1 outstanding). Listed for visibility:

| # | Item | Gated on |
|---|------|----------|
| W3.1 | Phase 2 (T-24): Shape-B ci.yml emitted from CiPipeline; all hand-authored YAML deleted | W2.4 (A0–A14 atoms ported) |
| W3.2 | Phase 2.5: affected-set intersection gate firing | W1.2 + W2.3 + W3.1 |
| W3.3 | Cross-target equivalence on substantial fixture set (rung 5 closure) | W1.3 + W1.7 ✓ ready; scope decision (which targets, how many fixtures) outstanding |
| W3.4 | L7 algebraic preservation post-emit (rung 6 closure) | per-fixture per-target per-algebra; combinatorial |
| W3.5 | Self-emit fixpoint (rung 7) — T-15 close | every Wave 1-2 item + W3.1 + a binary that compiles compiler.dag to itself bit-identically |
| W3.6 | TestClaim corpus actually executes (rung 8) | runner + cache + all SG fixes |
| W3.7 | Lenses gate PRs (rung 9): complexity / ownership / idempotency / grounding / synthesis | per-lens activation; substrate-rich/activation-poor pattern at its widest scope |

Wave 3 framing is a decision conversation, not a dispatch action. Recommend ratifying after SG-1 lands + W2.3 worksheet ships.

## Tree health snapshot (22:30Z)

Active managers under PM (nimble-dove-733):

| Manager | Lane | Children | Recent activity |
|---|---|---|---|
| proud-pike-680 | Modeling DFS | 0 | Authoring W2.3 + W2.6 worksheets (manager personally) |
| keen-heron-687 | Target Realization | 2 (zesty-carp + vivid-heron) | SG-5 #3957 just merged; SG-1 #3956 draft |
| sharp-otter-407 | Close/Receipt | 0 | #4032 merged 21:30Z; ledger #4017 maintained |
| smart-stag-871 | Compiler Spine | 0 visible; #4040 + #4041 in sub-worker sessions outside graph view | SG-7 #4014 merged today; emit-fix workers in flight |
| quick-tern-735 | Runtime/TestClaim | 1 | W2.6 #4022 landed; sharp-swift-715 archived clean |
| wise-bear-350 | PM closeout NO-OP | 0 | placeholder |
| zesty-ibex-231 → archived → respawn → archived → … | Ladder/Fixture | role-node cycles | W2.5 complete; role-node respawning each archive (dashboard-tooling pattern) |
| merry-badger-222 | Self-host/Release | 0 | #4021 + #4030 merged; v4 progress doc in flight per operator dispatch (~1h ETA) |

Cron job `c3a6bc78` (PM 30-min check-in) firing at :13 + :43 each hour, session-only, 7-day expiry.

## Release readiness summary (v0.1.0 alpha / WIP per flavor iv)

- ✓ Public snapshot generator (`scripts/publish-snapshot.sh`) on main
- ✓ Release workflow (`.github/workflows/release.yml`) + 6-target matrix on main
- ✓ install.sh on main (#3992)
- ✓ README weather hero (#3994) with honest swap-line disclaimer (#4031)
- ✓ SUPPORTED.md (#4025) with per-target confidence + alpha disclaimers for v3 / v4
- ✓ RELEASE_v0.1.0.md (#3991) flavor (iv) framing on main
- ✓ src/v4/* PM-jargon scrub (#4024)
- ✓ RELEASE_TODO + WISHLIST stripped from public snapshot (#4009)
- Remaining for tag: pre-tag dry-run of release.yml + per-target binary drop decisions + visibility verification (mac-mini.tailecbe08.ts.net:8443 → public)

## Open questions for operator

1. **Wave 3 framing conversation** — when to start. Recommend after SG-1 #3956 lands + smart-stag's emit-fix PRs flip ready.
2. **Pre-tag dry-run timing** — operator decides when to trigger release.yml dry-run on a throwaway tag.
3. **Website (mac-mini.tailecbe08.ts.net:8443 → public)** — flip status. snappy-bee-513 lane.
4. **Ladder/Fixture role-node closure** — the respawn cycle observed today suggests the role-node needs an explicit "lane done for now" closure mechanism, OR Wave 3 fixture/rung dispatch needs to land to give it actionable work.

## Cross-refs

- `docs/planning/v4-done-six-predicate-burn-down-2026-05-30.md` — per-predicate burn-down (merry-badger-222's amendments incoming)
- `docs/planning/v4-mw-d8-wave1-exit-ledger-2026-05-30.md` — MW-D8 exit-condition ledger (sharp-otter-407)
- `docs/planning/v4-merge-wave-and-next-waves-2026-05-30.md` — merge-wave plan + Wave 1-3 framing
- `docs/RELEASE_v0.1.0.md` — v0.1.0 release scope (flavor (iv))
- `docs/SUPPORTED.md` — per-target support contract
