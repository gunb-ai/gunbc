# R3 Deferral Anti-Pattern Audit — 2026-05-11

**Status:** PROPOSAL (Director-authored)
**Audience:** PM (deep-wolf-155) + standing Mgrs (Substrate / PB / Verification + future Mgrs) for ratification and dispatch
**Audit basis:** operator-directive 2026-05-11 — "by default, let these fail, would NOT invent a Miss case to support them; default is reject construction entirely. Miss should go away entirely; if something in substrate defines a Miss it should fail and be investigated asap."

## §0. Why this audit

The cost-lens `Lookup<SymbolicCost>::Miss` discussion surfaced a broader class problem: **deferral-via-wrapper-variant**. The pattern: when a substrate function can't or won't decide an answer, it returns a wrapper variant (Miss / None / Unknown / Unparsed / Pending / NotYetImplemented / etc.) that the caller must handle. This is "better than silently passing" but is itself a deferral of the real design decision.

Operator framing: deferral wrappers should not exist by default. Two acceptable outcomes for **Miss-class deferral**:

1. **Construction makes the case impossible** — the substrate shape doesn't permit constructing programs / data that would land in the deferral arm. The wrapper variant dissolves; the type narrows.
2. **Fail-closed Diagnostic** at the boundary — if the case is reachable from user input, surface a typed Diagnostic at the input boundary (per `feedback_fail_closed_discipline`), not a silently-Maybe-typed wrapper that callers can mishandle.

Anything else is **slacking on the design decision**. Operator notes this is "something I want to snuff out in R3" and "should really be escalated/caught during review."

**Scope qualifier (per openai-pro review 2026-05-11)**: "Miss-class deferral" ≠ "all `Option<T>` returns." Per `modeling-discipline.md:41-50` + `CODING.md:95-97`, `Option<T>` is **explicitly allowed when absence is a legitimate non-error state**. The Miss-class violation pattern is: `None`-on-error without a diagnostic write, panics in interior substrate flow (not boundary tooling), construction-time deferral surfaces ("Unknown/Pending/Missing" variants that capture "I haven't decided this yet"). Compliant wrappers — legitimate-absence `Option<T>`, fail-closed lattice bottoms, deliberate-default catch-alls — survive. Per-site classification required; bulk conversion would itself be a discipline violation.

## §1. Anti-pattern survey — counts at HEAD `origin/main`

Grep-verified instances. Each instance is a site where the substrate admits "I don't have a structural answer for this case."

| # | Anti-pattern | Sites | Notes |
|---|---|---|---|
| 1 | `Option<T>` returns in substrate `dag.rs` | **83** | Triage candidates — but **not** uniformly Miss-class. Per `modeling-discipline.md:41-50` + `CODING.md:95-97`, `Option<T>` is **allowed when absence is a legitimate non-error state**. Miss-class violation = `None`-on-error without a diagnostic write. The 83 sites need per-call audit: which are error-None (Miss-class, must dissolve) vs legitimate-absence (compliant). Don't bulk-convert. |
| 2 | `panic!` calls in production src | **244** | Triage candidates — boundary tooling (regen/bootstrap entrypoints) is legitimate per `CODING.md:307-309`; interior substrate-flow panics dissolve to typed Diagnostic per C-8. Per-site classification (§2.2). |
| 3 | `.expect(...)` calls in production src | **665** | Same boundary-vs-interior triage as #2 per `CODING.md:307-309`. |
| 4 | `.unwrap()` calls in production src | **35** | Same boundary-vs-interior triage as #2. |
| 5 | Catch-all `_ =>` match arms | **406** | Triage — distinguishes closed-enum non-exhaustiveness (must dissolve) from deliberate default-arm on open enums or terminal "shouldn't happen" arms paired with explicit Diagnostic. Per-site review. |
| 6 | `todo!() / unimplemented!() / unreachable!()` macros | **43** | Triage — `unreachable!()` paired with an invariant proof is legitimate; `todo!()` / `unimplemented!()` are explicit deferrals. Per-site disposition (§2.2). |
| 7 | Opaque error types (`Result<*, String>`, `Box<dyn Error>`) | **69** | Triage candidate — erases structural failure shape at the type level; Mgr-canvas audit per consumer to determine which warrant typed-Diagnostic conversion. |
| 8 | `ClaimResult::NotYetImplemented` | **21** (9 src + 12 tests) | Explicit gate-deferral — per-gate disposition (§3.5): substrate-land OR R3-carve. |
| 9 | `Lookup<T>::Miss` variant in generated code | **4 sites** | Empty-list-as-Miss pattern in `lens_cost_symbolic_generated.rs` (3) + `infer_helpers_generated.rs` (1). Miss-class; ratified for R3 dissolution per operator 2026-05-11. |
| 10 | `DescentEvidence::DescentUnknown` | substrate lattice bottom | **Authority-conflicting per `INVARIANTS.md:63-66`** — currently load-bearing as BoundedLattice fail-closed bottom. **Requires PM ratification before classification** (Miss-class vs lattice-element); see §3.2. |
| 11 | `EvidenceUnknown / EvidenceIncomplete` in descent_execution_proof residual | substrate residual carrier | Miss-shape candidate; see §3.3. |
| 12 | `ArrowBody::Pending` / `LensSurfacePending` | 4 enum variants | In-progress states baked into the substrate type; see §3.6 — substantial substrate-shape change, R3-load-bearing-ness needs PM ratification. |
| 13 | `structural_coverage_gap_*` named gates | **10+** | Tracking-only ratchets — per §3.8, each promotes (R3-load-bearing) or carves (post-R3); not classified as deferral by default. |

**Aggregate:** ≈1600+ instances of deferral-shape patterns in the v3 compiler's production surface. The cost-lens `Miss` work is one specific case in a much larger class.

## §2. Why each instance is "slacking"

Not every instance is equally bad. The taxonomy:

### §2.1 Miss-class deferral

Cases where the wrapper variant captures "I don't have an answer" and the answer is required for correctness:

- `Lookup<SymbolicCost>::Miss` — already committed to R3 dissolution per operator-ratified 2026-05-11. See §3.1.
- `EvidenceUnknown / EvidenceIncomplete` in descent residual — Miss-shape candidate; see §3.3.
- Empty-list-as-Miss in generated code — see §3.4.
- `ClaimResult::NotYetImplemented` — explicit deferral of gate execution. See §3.5.
- `ArrowBody::Pending` / `LensSurfacePending` — in-progress states baked into substrate type; **substantive substrate-shape change** — see §3.6.
- `DescentEvidence::DescentUnknown` — **NOT auto-classified as Miss-class**. Currently load-bearing as BoundedLattice fail-closed bottom per `INVARIANTS.md:63-66`. Requires PM ratification before classification; see §3.2.

### §2.2 Boundary tools used in interior — **triage candidates, NOT uniform "abuse"**

**Correction per openai-pro review**: the original framing called all 1100+ sites "Rust idiom abuse." That overgeneralized. Per `modeling-discipline.md:41-50` and `CODING.md:95-97`, `Option<T>` / `Result<T, E>` are explicitly **allowed** when absence/failure is a meaningful structural state. The violation pattern is `None`-on-error without a diagnostic write, and runtime panics in production substrate flow (vs. legitimate panics in regen binaries / bootstrap / boundary tooling).

Triage candidates (per-site audit, not bulk-conversion):

- 83 `Option<T>` returns in `dag.rs` — classify: error-None (must dissolve) vs legitimate-absence (compliant per `modeling-discipline.md:49-50`).
- 244 `panic!` calls — classify: boundary-tooling (regen, bootstrap, setup; legitimate) vs interior substrate flow (must dissolve to typed Diagnostic per C-8).
- 665 `.expect()` / 35 `.unwrap()` calls — same as panic: per `CODING.md:307-309`, panics/unwraps in library code are contract violations; the boundary subset (regen entrypoints) is acceptable.
- 69 opaque `Result<*, String>` / `Box<dyn Error>` — erases structural failure shape; Mgr-canvas audit per consumer.
- 406 catch-all `_ =>` match arms — each one needs review: does it admit non-exhaustiveness, or is it deliberate fall-through (e.g., default-arm for an open enum)?
- 43 `todo!() / unimplemented!() / unreachable!()` macros — explicit deferral; per-site disposition.

The Mgr-canvas audit (§3.7) should be a **per-file production-flow inventory** with each candidate classified by "boundary tooling vs interior substrate flow" before any conversion work. Counts are scope-signals for canvas authoring, not ratchet targets.

### §2.3 Tracking-only artifacts (must promote or carve)

- `structural_coverage_gap_*` named gates — these are R3 ledger rows that admit "we know this isn't covered" without forcing the dissolution decision. Either they're load-bearing (must close in R3) or they're not (must be explicitly carved out and post-R3-scheduled).

## §3. R3-close dissolutions (per-category direction)

### §3.1 Cost-lens `Lookup<SymbolicCost>::Miss` — **ALREADY RATIFIED 2026-05-11**

5 sub-cases; see Director ratification message to Substrate Mgr at warm-wolf-698. Net: `Lookup<SymbolicCost>` type collapses to `SymbolicCost`.

### §3.2 `DescentEvidence::DescentUnknown` — requires authority-update precondition (NOT direct dissolution)

**Correction per openai-pro review**: my original framing ("same shape as Miss; remove the variant") conflated a Miss-class deferral with a fail-closed lattice bottom. They're different. `INVARIANTS.md:63-66` currently establishes:

> `DescentEvidence` = `Strict | NonIncreasing | DescentUnknown`, with `BoundedLattice` top = `Strict`, bottom = `DescentUnknown` (fail-closed), meet = conservative branch merge, join = optimistic branch merge.

`DescentUnknown` as fail-closed bottom is a load-bearing lattice element, not a Miss surface. Per `feedback_construction_over_ratchets`: the dissolution question is whether the design intent still wants a 3-variant lattice (with `DescentUnknown` as conservative bottom for branch-merge semantics) or a 2-variant lattice (with construction-time rejection of programs that can't prove `Strict` or `NonIncreasing`).

**Revised proposal**: dispatch the design question to PM + Substrate Mgr for ratification BEFORE substrate change:

1. **(a) Keep 3-variant lattice; redirect Miss-shape concerns**: `DescentUnknown` stays as the fail-closed merge bottom (per current INVARIANTS authority). What dissolves is **construction**: programs that lower to `DescentEvidence::DescentUnknown` at a callsite become compile-time Diagnostic at the producer side — the lattice element survives, but reaching it during well-typed program execution is impossible. Producer-side path-narrowing, not lattice-shape change.
2. **(b) Collapse to 2-variant lattice**: requires explicit `INVARIANTS.md` authority update first. PM-tier ratification: is `DescentUnknown` truly redundant once construction-time rejection lands, or is the conservative-branch-merge bottom still needed for sound lattice composition (joins of partial program fragments)?

**Pre-dispatch requirement**: PM ratification on (a) vs (b). Until ratified, no worker dispatch on this dissolution. This is the discipline gap operator called out — review-tier should catch "audit doc proposes substrate-shape change before authority-doc update" before it lands.

Consumer impact (either path): `merge_evidence`, `join_evidence`, `evidence_rank`, etc. in `dag.rs` (the older versions were already retired in earlier Cluster K work) — surviving consumers need to align with whichever path PM ratifies.

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

Proposal: extend the PR review checklist (per `feedback_pre_authored_brief_queue` discipline) with **anti-pattern justification callouts** — flag for JUSTIFICATION review, not for "convert by default":

1. Does this PR add a new `Option<T>` return in substrate-tier code? → flag for **justification**: is absence a meaningful structural state (compliant per `modeling-discipline.md:49-50` + `CODING.md:95-97`), or is it error-None deferring a diagnostic write? Reviewer asks; author justifies; non-compliant cases convert.
2. Does this PR add a new `panic!` / `.expect()` / `.unwrap()` in **library / substrate-flow** code (not in regen/bootstrap entrypoints)? → flag for Diagnostic-conversion review per `CODING.md:307-309`.
3. Does this PR add a new enum variant whose name contains `Unknown / Pending / Missing / Incomplete / Maybe`? → flag for **construction-impossibility OR fail-closed-lattice review**: is the variant a deferral surface (must dissolve) or a load-bearing lattice element (compliant per `INVARIANTS.md` authority)?
4. Does this PR add a `NotYetImplemented` predicate? → flag for substrate-readiness review.
5. Does this PR add a new `_ =>` catch-all in an enum match? → flag for **exhaustiveness OR deliberate-default review**: is the enum closed (catch-all admits non-exhaustiveness) or open (catch-all is the default arm)?

These checklist items belong in `.github/PULL_REQUEST_TEMPLATE.md` (or whichever PR description ratchet is canonical). Per `feedback_construction_over_ratchets`: prefer structural enforcement (lint rule) over textual checklist, but checklist is a transitional state until lints land.

**The discipline shift**: from "deferral wrappers must die" to "deferral wrappers require explicit justification at construction." Compliant wrappers (legitimate-absence Option, fail-closed-lattice bottoms, deliberate-default catch-alls) survive; deferral-wrapper-as-design-laziness gets caught at review.

## §5. Sequencing recommendation

R3-close commitment scope (per operator-directive 2026-05-11). **Dispatch order respects authority gates established in §3 — items requiring PM-tier authority update are explicitly blocked until that update lands.**

1. **§3.1 (cost-lens Miss)** — RATIFIED, dispatched to Substrate Mgr 2026-05-11.
2. **§3.2 (DescentUnknown)** — **BLOCKED on PM ratification of path (a) vs (b)** per §3.2. No dispatch until PM decides whether substrate keeps 3-variant lattice + construction-side narrowing, or authority update precedes 2-variant collapse. Same-batch with §3.1 only valid under path (a); path (b) requires `INVARIANTS.md:63-66` edit landing first.
3. **§3.3 (descent-execution-proof residual)** — propose same-batch dispatch with §3.1 once §3.2 path is ratified (the residual carrier sits downstream of `DescentEvidence` shape).
4. **§3.4 (empty-list-as-Miss)** — generated-code regen needed; folds into §3.1 + §3.5 dispatch.
5. **§3.5 (NotYetImplemented audit)** — per-gate Mgr-tier dispatch; recommend Verification Mgr re-spawn (currently archived per overnight cascade) authors the audit.
6. **§3.6 (ArrowBody::Pending)** — larger substrate-shape work; **R3-load-bearing-ness needs PM ratification before dispatch** (may need post-R3 carve).
7. **§3.7 (boundary-tool-in-interior)** — per-file Mgr-canvas authoring; spread across Mgrs. Each canvas owns its per-site classification (boundary tooling vs interior substrate flow) before any conversion work.
8. **§3.8 (structural_coverage_gap)** — promote/carve audit; PM-coordinated.
9. **§4 (review-process)** — PM authors the PR-template ratchet update; cross-Mgr coordination.

**Authority-gate summary**: items 2 (DescentUnknown) and 6 (ArrowBody::Pending) are explicitly PM-blocked until authority disposition. Items 1/3/4/5/7/8/9 may dispatch on the standing Mgr-canvas authority once R3-scope-ratified.

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
