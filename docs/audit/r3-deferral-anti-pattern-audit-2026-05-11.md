# R3 Deferral Anti-Pattern Audit — 2026-05-11

**Status:** PROPOSAL (Director-authored)
**Audience:** PM (deep-wolf-155) + standing Mgrs (Substrate / PB / Verification + future Mgrs) for ratification and dispatch
**Audit basis:** operator-directive 2026-05-11 — "by default, let these fail, would NOT invent a Miss case to support them; default is reject construction entirely. Miss should go away entirely; if something in substrate defines a Miss it should fail and be investigated asap."

## §0. Why this audit

The cost-lens `Lookup<SymbolicCost>::Miss` discussion surfaced a broader class problem: **deferral-via-wrapper-variant**. The pattern: when a substrate function can't or won't decide an answer, it returns a wrapper variant (Miss / None / Unknown / Unparsed / Pending / NotYetImplemented / etc.) that the caller must handle. This is "better than silently passing" but is itself a deferral of the real design decision.

Operator framing: deferral wrappers should not exist by default. Two acceptable outcomes:

1. **Construction makes the case impossible** — the substrate shape doesn't permit constructing programs / data that would land in the deferral arm. The wrapper variant dissolves; the type narrows.
2. **Fail-closed Diagnostic** at the boundary — if the case is reachable from user input, surface a typed Diagnostic at the input boundary (per `feedback_fail_closed_discipline`), not a silently-Maybe-typed wrapper that callers can mishandle.

Anything else is **slacking on the design decision**. Operator notes this is "something I want to snuff out in R3" and "should really be escalated/caught during review."

## §1. Anti-pattern survey — counts at HEAD `origin/main`

Grep-verified instances. Each instance is a site where the substrate admits "I don't have a structural answer for this case."

| # | Anti-pattern | Sites | Notes |
|---|---|---|---|
| 1 | `Option<T>` returns in substrate `dag.rs` | **83** | Pure deferral surface — every caller must handle None. |
| 2 | `panic!` calls in production src | **244** | Should be typed Diagnostics, not runtime panics. |
| 3 | `.expect(...)` calls in production src | **665** | Assumes success; same fail-shape as panic. |
| 4 | `.unwrap()` calls in production src | **35** | Same shape as expect; less explicit. |
| 5 | Catch-all `_ =>` match arms | **406** | Each one admits non-exhaustiveness of the matched enum. |
| 6 | `todo!() / unimplemented!() / unreachable!()` macros | **43** | Explicit "I haven't decided this." |
| 7 | Opaque error types (`Result<*, String>`, `Box<dyn Error>`) | **69** | Erases the structural shape of failure. |
| 8 | `ClaimResult::NotYetImplemented` | **21** (9 src + 12 tests) | Explicit "gate exists, substrate doesn't." |
| 9 | `Lookup<T>::Miss` variant in generated code | **4 sites** | Empty-list-as-Miss pattern in `lens_cost_symbolic_generated.rs` (3) + `infer_helpers_generated.rs` (1). |
| 10 | `DescentEvidence::DescentUnknown` | substrate lattice bottom | "We don't know if this descends" — same shape as `SameArgumentCall` Miss. |
| 11 | `EvidenceUnknown / EvidenceIncomplete` in descent_execution_proof residual | substrate residual carrier | "Proof construction failed but maybe just incomplete" — Miss-shape. |
| 12 | `ArrowBody::Pending` / `LensSurfacePending` | 4 enum variants | In-progress states baked into the substrate type. |
| 13 | `structural_coverage_gap_*` named gates | **10+** | Tracking-only ratchets — admits "we know this isn't covered yet" without enforcing dissolution. |

**Aggregate:** ≈1600+ instances of deferral-shape patterns in the v3 compiler's production surface. The cost-lens `Miss` work is one specific case in a much larger class.

## §2. Why each instance is "slacking"

Not every instance is equally bad. The taxonomy:

### §2.1 Pure deferral (must dissolve)

Cases where the wrapper variant captures "I don't have an answer" and the answer is required for correctness:

- `Lookup<SymbolicCost>::Miss` — already committed to R3 dissolution per operator-ratified 2026-05-11. See §3.1.
- `DescentEvidence::DescentUnknown` — same shape; should also dissolve. See §3.2.
- `EvidenceUnknown / EvidenceIncomplete` in descent residual — same shape; see §3.3.
- Empty-list-as-Miss in generated code — see §3.4.
- `ClaimResult::NotYetImplemented` — explicit deferral of gate execution. See §3.5.
- `ArrowBody::Pending` / `LensSurfacePending` — in-progress baked into the substrate, meaning the substrate type allows "I'm half-built" as a valid state. See §3.6.

### §2.2 Boundary tools used in interior (must convert to typed)

Cases where Rust's standard "this might fail" tools (`Option`, `Result`, `panic!`, `.expect()`) are used INSIDE the substrate rather than only at the user-input boundary:

- 83 `Option<T>` returns in `dag.rs`
- 244 `panic!` calls
- 665 `.expect()` calls
- 35 `.unwrap()` calls
- 69 opaque `Result<*, String>` / `Box<dyn Error>`
- 406 catch-all `_ =>` match arms
- 43 `todo!() / unimplemented!() / unreachable!()` macros

These are "Rust idiom" abuse: the language offers these tools because real software has boundaries, but using them inside the substrate (vs. at the boundary) treats every internal function as if it were the boundary. Per `feedback_fail_closed_discipline` (C-8: every detectable problem is a Diagnostic), the substrate's internal flows should be total (typed-impossible to fail) or surface a typed Diagnostic at the boundary, not paper-over via runtime panic.

### §2.3 Tracking-only artifacts (must promote or carve)

- `structural_coverage_gap_*` named gates — these are R3 ledger rows that admit "we know this isn't covered" without forcing the dissolution decision. Either they're load-bearing (must close in R3) or they're not (must be explicitly carved out and post-R3-scheduled).

## §3. R3-close dissolutions (per-category direction)

### §3.1 Cost-lens `Lookup<SymbolicCost>::Miss` — **ALREADY RATIFIED 2026-05-11**

5 sub-cases; see Director ratification message to Substrate Mgr at warm-wolf-698. Net: `Lookup<SymbolicCost>` type collapses to `SymbolicCost`.

### §3.2 `DescentEvidence::DescentUnknown` — proposed dissolution

Currently: lattice bottom for "we can't prove descent." Same shape as `SameArgumentCall` Miss.

Proposal: remove `DescentUnknown` variant from `DescentEvidence` enum. Construction of recursive Transform that can't be proven `Strict` or `NonIncreasing` becomes a compile-time Diagnostic. Lattice collapses from 3 variants to 2 (`Strict | NonIncreasing`).

Consumer impact: `merge_evidence`, `join_evidence`, `evidence_rank`, etc. in `dag.rs` (already retired in earlier work) — surviving consumers must drop the `DescentUnknown` arms.

### §3.3 Descent-execution-proof residual `EvidenceUnknown / EvidenceIncomplete` — proposed dissolution

Currently: `DescentResidual = EvidenceUnknown(NonStrictEvidence) | EvidenceIncomplete`. The work to narrow from 4 to 2 was good, but the residual itself is still a Miss-shape.

Proposal: if descent execution can't complete, that's a typed compile-time Diagnostic at the producer's surface. Replace the residual carrier with `Result<DescentExecutionProof, DescentExecutionDiagnostic>` where `DescentExecutionDiagnostic` is a concrete error type (not a Maybe-coverage Maybe-incomplete wrapper).

### §3.4 Empty-list `[] => Lookup::Miss` in generated code (4 sites)

Per operator framing on cost lens Case 5: lookups should not fail silently. Either use typed-key references (cost-table-with-guaranteed-presence) or fail-closed Diagnostic. Default for `lookup_cost([])` is `Diagnostic`, not `ConstantCost(0)`. Affects `lens_cost_symbolic_generated.rs` (3 sites) + `infer_helpers_generated.rs` (1 site).

### §3.5 `ClaimResult::NotYetImplemented` (21 sites) — per-gate disposition

Each `NotYetImplemented` is a gate-tier deferral. For R3 close:

- **Tier-1**: If the gate is in §1.8 R3-load-bearing scope (96 enumerated), the `NotYetImplemented` must be replaced with a real predicate evaluator OR the gate must be explicitly carved post-R3.
- **Tier-2**: Audit each of the 21 sites and route to corresponding Mgr (Verification owns most via TestRunner; lower.rs + test_runner.rs predicate sites).

Per `feedback_construction_over_ratchets` — `NotYetImplemented` is a textual ratchet that should dissolve when the predicate substrate lands.

### §3.6 `ArrowBody::Pending` / `LensSurfacePending` — in-progress states in substrate

`ArrowBody::Pending` is a `Behavior::Transform.body` variant indicating "this function hasn't been lowered yet." It's a transitional state baked into the substrate type. Consequence: every walker / lens that processes Transform bodies has to handle `Pending` (paper-over).

Proposal: substrate-shape redesign — separate `UnresolvedSignature` (pre-lowering) from `ResolvedTransform` (post-lowering). The lowering pipeline transforms the former into the latter. Walkers / lenses operate on `ResolvedTransform` only. Pending becomes unrepresentable at the post-lowering substrate type level.

### §3.7 Boundary-tool-in-interior cleanup (1100+ sites) — per-Mgr-canvas audit

Volume too large for a single dispatch. Recommend Mgr-canvas authoring per file:
- Substrate Mgr (warm-wolf-698): `dag.rs` 83 `Option<T>` returns audit → typed accessor cleanup
- Substrate Mgr: 244 `panic!` calls audit → typed Diagnostic conversion
- Substrate Mgr: 665 `.expect()` calls audit → similar
- PB Mgr (warm-dove-618): emit-path `unreachable!()` macros audit
- Verification Mgr (post-respawn): `_ =>` catch-all audit (406 sites)

Per-file canvas authoring expected; not a single PR.

### §3.8 `structural_coverage_gap_*` ratchets — promote or carve

10+ named gates in `ROADMAP.md` and worker briefs. Each should be reviewed:
- Load-bearing for R3: close in R3 cycle
- Not load-bearing: explicit carve to post-R3 (no silent tracking)

## §4. Process implication — why review didn't catch this

Operator notes: "this is me slacking — things like this should really be escalated/caught during review." Review-tier process gap.

Proposal: extend the PR review checklist (per `feedback_pre_authored_brief_queue` discipline) with **explicit anti-pattern callouts**:

1. Does this PR add a new `Option<T>` return in substrate-tier code? → flag for typed-accessor / fail-closed Diagnostic review.
2. Does this PR add a new `panic!` / `.expect()` / `.unwrap()` in production code? → flag for Diagnostic-conversion review.
3. Does this PR add a new enum variant whose name contains `Unknown / Pending / Missing / Incomplete / Maybe`? → flag for construction-impossibility review.
4. Does this PR add a `NotYetImplemented` predicate? → flag for substrate-readiness review.
5. Does this PR add a new `_ =>` catch-all in an enum match? → flag for exhaustiveness review.

These checklist items belong in `.github/PULL_REQUEST_TEMPLATE.md` (or whichever PR description ratchet is canonical). Per `feedback_construction_over_ratchets`: prefer structural enforcement (lint rule) over textual checklist, but checklist is a transitional state until lints land.

## §5. Sequencing recommendation

R3-close commitment scope (per operator-directive 2026-05-11):

1. **§3.1 (cost-lens Miss)** — RATIFIED, dispatched to Substrate Mgr 2026-05-11.
2. **§3.2 (DescentUnknown)** — propose same-batch dispatch with §3.1 (same Substrate-tier work).
3. **§3.3 (descent-execution-proof residual)** — propose same-batch dispatch.
4. **§3.4 (empty-list-as-Miss)** — generated-code regen needed; folds into §3.1 + §3.5 dispatch.
5. **§3.5 (NotYetImplemented audit)** — per-gate Mgr-tier dispatch; recommend Verification Mgr re-spawn (currently archived per overnight cascade) authors the audit.
6. **§3.6 (ArrowBody::Pending)** — larger substrate-shape work; PB Mgr or Substrate Mgr canvas decides.
7. **§3.7 (boundary-tool-in-interior)** — per-file Mgr-canvas authoring; spread across Mgrs.
8. **§3.8 (structural_coverage_gap)** — promote/carve audit; PM-coordinated.
9. **§4 (review-process)** — PM authors the PR-template ratchet update; cross-Mgr coordination.

## §6. Open questions for PM ratification

1. Is §3.6 (ArrowBody::Pending dissolution) R3-load-bearing or post-R3? It's a substantial substrate-shape change; pragmatically may need carve-out.
2. Is §3.7 (1100+ boundary-tool sites) realistic for R3 timeline? Or should it be a "no new instances" ratchet with cleanup deferred?
3. Does §4 (PR-template ratchet) need a Verification Mgr re-spawn first, or can PM author standalone?
4. Should §3.8 (structural_coverage_gap audit) be folded into the standing Debt-Paydown Mgr cadence (silent-ram-834 — if alive) or stand alone?

## §7. Related memories / references

- `feedback_construction_over_ratchets` — model first, violations dissolve; never heuristic-patch.
- `feedback_state_space_vs_behavioral_invariants` — type enforcement > API enforcement.
- `feedback_decidability_invariant` — all `.dag` code must be decidable.
- `feedback_fail_closed_discipline` — C-8: every detectable problem is a Diagnostic; no warnings, no silent Nones, no panics.
- `feedback_coproduct_dissolution` — coproducts are categorical compression; dissolve into coordinates.
- `feedback_no_textual_enforcement_bridges` — never propose grep/regex as interim enforcement.
- INVARIANTS.md §C-8 (fail-closed discipline).

## §8. Ratification asks

Per PM disposition request:

- (a) Ratify §3 dissolution directions per category — or surface alternatives.
- (b) Ratify §4 PR-template ratchet authoring authority (PM vs. Verification Mgr).
- (c) Ratify §5 sequencing — same-batch vs. staged.
- (d) Author or delegate §3.7 per-Mgr-canvas audit dispatch.
