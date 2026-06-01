# CI Selection Receipt — Wave 3 claim/testgen projection (§10.0)

> **Status:** WORKSHEET APPROVED — Modeling DFS Arbiter §8 sign-off 2026-06-01 (`proud-fox-405`; Phase 1 landed PR #4174).  
> **Author:** loyal-carp-103 (CI Manager Wave 3 lane; smart-newt-797 approved EXTEND attachment 2026-06-01).  
> **Charter:** `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` §11.7.4 (Wave 3 shadow), §11.7.6 (honesty).  
> **Blocked implementation until:** (a) this worksheet §8 sign-off, (b) neat-hawk-413 population/eval entry for `affected_set_from_diff` on CI, (c) neat-hawk mapping API for testgen roster.

---

## §10.0-adapted worksheet

```text
Schema-migration class:     Wave 3 shadow receipt — extend CiSelectionReceipt (no parallel receipt genus)
Representative failure:     CI `affected` job emits path-bucket CiComponentAffected only; no structured
                            witness of which TestClaims / testgen slots a genuine RerunNodeSet would
                            select. Operator cannot compare shadow selection vs §11.7.1 Class A floor
                            over N PRs (Wave 4 gate).

Immediate local patch:
  - Mint CiWave3ShadowReceipt duplicating pr + affected.
  - Store floor_skip: Bool and mode: CiSelectionMode as independent writable fields both set in host.
  - Use reason: Symbol on claim rows (unbounded string-keyed heuristics).
  - Reimplement affected_set_from_diff in tools/ci_affected_components Rust mirror.

Why forbidden:
  - Parallel receipt violates INVARIANTS P2 / MODELING.md M9 single-authority (same selection event,
    two carriers; host merges forever).
  - mode + floor_skip as free-standing fields that must stay in lockstep is a divergence hazard unless
    one derives from the other or they name genuinely independent facts (arbiter ruling required).
  - Free Symbol reason reintroduces heuristic CI policy beside T-21 lens authority.
  - Rust lens reimplementation diverges from v4.lens.affected_set and calcifies path buckets as frontier.

DFS path:
  std/ authority (CONSUME):
    - v4.std.change — ChangeSet, AffectedSet, AffectedSetFailClosed | AffectedSetProduced
    - v4.std.verification — TestClaim, test_claim_label, ClaimAnchorKey, content_hash on claims
  lens authority (CONSUME):
    - v4.lens.affected_set — affected_set_from_diff, affected_set_rerun_nodes, RerunNodeSet
    - v4.lens.testgen — Generator<TestgenConcept>, testgen_scheduled_*, testgen_emit_*_claim
  workflow authority (AMEND):
    - v4.workflow.ci — CiSelectionReceipt, ci_selection_receipt_shadow, ci_select_from_affected_set
  test claims (CONSUME):
    - v4.test.claim.workflow.affected_set_ci_runner — fail-closed superset + narrow frontier claims
  host transport (AMEND — thin invoke only):
    - tools/ci_affected_components — emit-ci-wave3-shadow-receipt bin after eval entry lands
  existing scaffold:
    - docs/planning/compiler-spine-ci-selection-receipt-shadow-2026-05-30.md (step grain)
    - ci.dag ci_selection_decision_for_step reason symbols (closed set at data layer today)

Deepest unsound boundary:
  CiSelectionReceipt models step-level decisions only; claim-level and testgen-slot projections are
  missing from the receipt carrier even though ci_select_from_affected_set is already the IRT-1
  authority for TestClaimCorpusEvalCommand. Live CI cannot evaluate affected_set_from_diff without
  bootstrap eval entry (neat-hawk / Compiler Spine routing).

Systemic fix (single-authority fact):
  EXTEND CiSelectionReceipt with claim + testgen projections and shadow policy fields; one population
  entry ci_selection_receipt_shadow_from_git_diff(git_diff) -> Outcome<CiSelectionReceipt>.
  New sibling row types attach under that receipt (not a second top-level receipt):
    - CiSelectionMode (Shadow | Active) — policy dimension for Wave 3 vs Wave 4 active skip
    - CiTestClaimSelection — per-claim row in selected roster (parallel grain to CiStepSelection)
    - TestgenSlotSelection — scheduled generator slot linked to emitted claim(s)
  CiComponentAffected retained only as component_affected_comparison (explicitly non-authoritative).

Non-goals:
  - New top-level CiWave3ShadowReceipt type
  - Active skip / floor gating (Wave 4)
  - T-38 per-row TestClaimRun runtime verdicts in shadow receipt (neat-hawk P5 lane)
  - ci_selection_receipt_shadow step-decision live emission in v1 (follow-up unless trivial)
  - Host-side reimplementation of affected_set lens

Falsification probe (implementation PR):
  F1 — Receipt JSON schema includes testclaim_decisions + testgen_slots populated from modeled eval,
       not from CiComponentAffected alone.
  F2 — Fail-closed diff: AffectedSetFailClosed => testclaim_decisions superset equals full shadow
       roster (parity with affected_set_ci_runner.dag).
  F3 — Narrow diff fixture: single touched claim node => exactly one testclaim_decisions row selected.
  F4 — v4_workflow_ci_wave3_* smoke: ci.yml affected job has emit step; ci_floor/ci never needs affected.
  F5 — Honesty ledger §Wave 3 row names structured receipt + comparison-only component_affected.
  F6 — mode/floor_skip: no host path can emit mode=Shadow with floor_skip=true (per arbiter ruling).

Metric allowed only as secondary:
  Count of selected claims per PR; wall-clock of shadow job — not acceptance.
```

---

## §1 Single-authority fact

| Field | Value |
| ----- | ----- |
| **Fact name** | `CiSelectionReceipt` claim + testgen projection (Wave 3 shadow) |
| **Authority home** | `src/v4/workflow/ci.dag` (extend existing `CiSelectionReceipt`; population fn co-owned with Runtime/TestClaim for roster + testgen filter) |
| **Consumers** | `emit-ci-wave3-shadow-receipt` host transport; operator shadow-vs-floor comparison; Wave 4 active-skip (mode `Active`) |
| **Dissolves** | Ad-hoc JSON shapes, parallel `CiWave3ShadowReceipt`, component-flag-as-frontier stand-ins |

---

## §2 Proposed carriers (arbiter review — not authored until §8 sign-off)

### 2.1 Extend `CiSelectionReceipt`

**Landed Phase 1 sketch** — matches `src/v4/workflow/ci.dag` on PR #4174 (authoritative over pre-sign-off Symbol-flatten draft).

```dag
import v4.lens.testgen { Generator, TestgenConcept }

type CiActiveFloorSkipEvidence {
  cached_verdict_digest: Hash
}

type CiSelectionMode
  = Shadow
  | Active { skip_evidence: CiActiveFloorSkipEvidence }

type CiSelectionReceiptProvenance
  = FixtureReceipt
  | LivePrGitDiff   // coproduct-only until ci_selection_receipt_shadow_from_git_diff

type CiClaimSelectionReason
  = FailClosedSuperset
  | AffectedFrontierEmpty              // global rerun frontier empty
  | AffectedIntersectionNonempty       // selected: claim ∈ rerun frontier
  | AffectedIntersectionEmpty          // not selected: frontier nonempty, claim outside
  | TestgenSlotMapped
  | ShadowObserveOnly
  | FloorStepCarveout

type CiSelectionReceipt {
  pr: ChangeSet
  affected: AffectedSet
  mode: CiSelectionMode
  provenance: CiSelectionReceiptProvenance
  decisions: List<CiStepSelection>
  testclaim_decisions: List<CiTestClaimSelection>
  testgen_slots: List<TestgenSlotSelection>
  component_affected_comparison: CiComponentAffected
}

type CiTestClaimSelection {
  anchor: ClaimAnchorKey
  label: String
  coproduct_variant: TestClaimCoproductVariant
  claim_projection_hash: Hash          // IRT-4 test_claim_claim_hash_digest(c)
  selected: Bool
  reason: CiClaimSelectionReason
}

type TestgenSlotSelection {
  generator: Generator<TestgenConcept>   // v4.lens.testgen authority — NOT Symbol flattening
  emits_claim_anchor: ClaimAnchorKey
  selected: Bool
  reason: CiClaimSelectionReason
}

// W1.5 step-only shadow: hardcodes FixtureReceipt; empty claim/testgen until live entry.
// Wave 3 fixture receipt: ci_selection_receipt_mk + ci_wave3_shadow_testclaim_selection_rows.
```

### 2.2 Population entry (single modeled surface)

```dag
fn ci_selection_receipt_shadow_from_git_diff(
  git_diff: GitDiffNameOnly
) -> Outcome<CiSelectionReceipt>
```

**Requires:** `affected_set_from_diff` + shadow claim roster + testgen filter (neat-hawk bootstrap-eval or eval hook on CI). Until eval lands, **no implementation PR** from CI lane.

---

## §3 Modeling questions for proud-fox-405 (required before type land)

### 3.1 `mode: CiSelectionMode` vs `floor_skip: Bool` — same fact twice?

**Observation:** For Wave 3, operator intent is fixed: shadow mode ⟺ floor is never skipped by this receipt. Storing both `mode = Shadow` and `floor_skip = false` as independent writable fields creates a lockstep pair — if they diverge, receipts lie about §11.7.5 compliance.

**Options (request ruling):**

| Option | Shape | Tradeoff |
| ------ | ----- | -------- |
| **A. Derive** | Keep `mode` only; `floor_skip` is projection `mode != Active` (or const false when `mode == Shadow`) | Single authority; host cannot emit contradictory receipt |
| **B. Independent facts** | Keep both only if Wave 4 needs `mode = Active` while some claims still cannot skip floor (name the independent fact each carries) | Requires explicit invariant fn in substrate, not host convention |
| **C. Rename** | Replace pair with one carrier e.g. `CiScheduleAuthority { receipt: SelectionReceiptOnly, executable_skip: Bool }` | Clearest semantics; slightly wider schema change |

**Worker recommendation:** Option **A** for Wave 3 unless Wave 4 design already needs Option B. Do **not** ship both as free-standing host-writable fields without arbiter ruling.

### 3.2 `reason` — closed set vs free `Symbol`

**Observation:** `CiStepSelection.reason` today uses named `data ci_*_reason: Symbol` atoms (e.g. `ci_receipt_inputs_fail_closed_reason`, `ci_shadow_git_diff_proxy_nonempty_reason`) — a de facto closed set at the data layer, but typed as `Symbol`.

**Request ruling:** Promote to explicit coproduct for claim/testgen rows (and eventually step rows):

```dag
// Historical §3.2 sketch — landed name is CiClaimSelectionReason (arbiter §8).
type CiClaimSelectionReason
  = ReceiptInputsFailClosed
  | AffectedIntersectionNonempty
  | AffectedIntersectionEmptyCacheValid
  | CacheDigestProjectionFailClosed
  | CarveoutMatched { carveout_reason: Symbol }
  | TestClaimInRerunFrontier
  | TestClaimFailClosedSuperset
  | TestgenSlotSchedulesForFrontier
  | TestgenSlotNotInFrontier
```

Forbidden: host-supplied arbitrary `Symbol` strings as selection justification (heuristic policy channel).

### 3.3 Carrier homes

| Carrier | Proposed home | Alternative |
| ------- | ------------- | ----------- |
| `CiTestClaimSelection` | `v4.workflow.ci` | `v4.std.verification` if claim-selection is cross-workflow |
| `TestgenSlotSelection` | `v4.workflow.ci` or `v4.lens.testgen` | testgen.dag if slot selection is lens-owned |
| `CiSelectionMode` | `v4.workflow.ci` | — |

**Worker default:** all three in `ci.dag` beside `CiStepSelection` unless arbiter routes testgen slot to `v4.lens.testgen`.

---

## §4 Dependencies (not worksheet scope — manager-routed)

| Dependency | Owner | Blocks |
| ---------- | ----- | ------ |
| `ci_selection_receipt_shadow_from_git_diff` eval on CI | neat-hawk-413 (+ Compiler Spine bootstrap entry) | host transport + genuine `affected` / `rerun_frontier` |
| Shadow claim roster + testgen→frontier filter fns | neat-hawk-413 | `testclaim_decisions`, `testgen_slots` population |
| Wave 3 CI wiring + honesty ledger | loyal-carp-103 | after (a) §8 + (b)(c) |

---

## §5 Implementation sequencing (post sign-off)

1. Arbiter §8 sign-off on §3 rulings (mode/floor_skip, reason coproduct, carrier homes).
2. neat-hawk lands eval/print entry + mapping API folded into `ci_selection_receipt_shadow_from_git_diff`.
3. loyal-carp-103: type extension in `ci.dag` + `emit-ci-wave3-shadow-receipt` + `ci.yml` shadow step + `v4_workflow_ci_wave3_*` + honesty ledger §Wave 3.

---

## §6 Non-goals (worksheet boundary)

- Wave 4 active skip and cached verdict shape.
- Merging `CiSelectionReceipt` with `CiComponentAffected` authority.
- §10.0 worksheet for a **new top-level** receipt type (not requested — EXTEND only).

---

## §7 Falsification table (implementation PR)

| ID | Probe | Pass criterion |
| -- | ----- | -------------- |
| F1 | Receipt populated from modeled eval | `testclaim_decisions` / `testgen_slots` non-empty on touched-claim fixture PR; not derivable from `component_affected_comparison` alone |
| F2 | Fail-closed superset | `AffectedSetFailClosed` ⇒ full roster selected (matches `affected_set_ci_runner.dag`) |
| F3 | Narrow frontier | Single-node touch ⇒ one selected claim row |
| F4 | CI graph | `affected` has emit step; `ci_floor` / `ci` never `needs: affected` |
| F5 | Honesty ledger | §Wave 3 row documents structured receipt |
| F6 | mode/floor_skip invariant | Per §3.1 arbiter ruling — no contradictory receipt bodies |

---

## §8 Arbiter checklist (proud-fox-405)

- [x] EXTEND `CiSelectionReceipt` approved (no parallel `CiWave3ShadowReceipt`).
- [x] §3.1 ruled: **`CiSelectionMode` only** — do not persist `floor_skip`; derive `ci_floor_held(receipt) := mode == Shadow`.
- [x] §3.2 ruled: **`CiClaimSelectionReason` closed coproduct** on claim rows; step-level `CiStepSelection.reason` stays `Symbol` for now.
- [x] §3.3 carrier homes: `CiTestClaimSelection`, `TestgenSlotSelection`, `CiSelectionMode` in `src/v4/workflow/ci.dag`; roster in `wave3_shadow_roster.dag`.
- [x] Phase 1 dispatched: fixture receipt + smoke tests on PR #4174; **live CI host transport** deferred on `node://adhoc-331899f9-19a`.

---

## §9 References

- `docs/planning/v4-predicate-dependency-graph-2026-06-01-eod.md` §11.7.4–§11.7.6
- `docs/planning/compiler-spine-ci-selection-receipt-shadow-2026-05-30.md`
- `docs/planning/ci-required-surface-cut-2026-06-01.md`
- `docs/design-ci-bankruptcy-rebuild.md` §2.1 I1 (frontier + `ci_select_*`)
- `src/v4/test/claim/workflow/affected_set_ci_runner.dag`
- `MODELING.md` M9
