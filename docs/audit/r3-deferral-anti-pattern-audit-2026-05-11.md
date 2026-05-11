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
| 11 | `EvidenceUnknown / EvidenceIncomplete` in descent_execution_proof residual | substrate residual carrier | **Authority-conflicting per `docs/briefs/r3-substrate-descent-execution-proof-worker.md` Director-ratified γ-shape** — compliant as written today. See §3.3 corrected disposition. |
| 12 | `ArrowBody::Pending` | declaration-tier substrate variant | Pre-lowering transitional state on `TypeConnective::Arrow.body`; see §3.6. **Note**: `LensSurfacePending` (originally lumped here) is explicitly **terminal** per `dag/effects.rs:197` (a `ParallelismUnsupportedKind` variant — terminal unsupported-reason payload, NOT a Pending-shape in-progress state). Misleading suffix; do not classify as Miss-class deferral. |
| 13 | `structural_coverage_gap_*` named gates | **10+** | Tracking-only ratchets — per §3.8, each promotes (R3-load-bearing) or carves (post-R3); not classified as deferral by default. |

**Aggregate:** ≈1600+ instances of deferral-shape patterns in the v3 compiler's production surface. The cost-lens `Miss` work is one specific case in a much larger class.

## §2. Why each instance is "slacking"

Not every instance is equally bad. The taxonomy:

### §2.1 Miss-class deferral

Cases where the wrapper variant captures "I don't have an answer" and the answer is required for correctness:

- `Lookup<SymbolicCost>::Miss` — already committed to R3 dissolution per operator-ratified 2026-05-11. See §3.1.
- Empty-list-as-Miss in generated code — see §3.4.
- `ClaimResult::NotYetImplemented` — explicit deferral of gate execution. See §3.5.
- `ArrowBody::Pending` — pre-lowering transitional state on declaration-tier `TypeConnective::Arrow.body`; **substantive substrate-shape change** — see §3.6.
- `LensSurfacePending` — **NOT auto-classified as Miss-class**. Per `dag/effects.rs:197` (🟢 TERMINAL), this is a `ParallelismUnsupportedKind` variant — terminal unsupported-reason payload, not a Pending-shape in-progress state. Misleading suffix; do not dissolve.
- `DescentEvidence::DescentUnknown` — **NOT auto-classified as Miss-class**. Currently load-bearing as BoundedLattice fail-closed bottom per `INVARIANTS.md:63-66`. Requires PM ratification before classification; see §3.2.
- `EvidenceUnknown / EvidenceIncomplete` in descent_execution_proof residual — **NOT auto-classified as Miss-class**. Compliant per Director-ratified γ-shape at `docs/briefs/r3-substrate-descent-execution-proof-worker.md` (ratification at gunbc#828 #issuecomment-4395060514). Any dissolution proposal requires authority-reconciliation precondition; see §3.3 corrected disposition.

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

### §3.3 Descent-execution-proof residual `EvidenceUnknown / EvidenceIncomplete` — requires authority-reconciliation precondition (NOT direct dissolution)

**Correction per inline review finding 2026-05-11**: my original framing called the residual "Miss-shape" and proposed replacing the carrier without reconciling against the **Director-ratified residual shape** at `docs/briefs/r3-substrate-descent-execution-proof-worker.md:20-27` (per `r3-program-plan.md` Q-EVAL-Descent-Termination-Contract ratification at gunbc#828 #issuecomment-4395060514). That violates P1 modeling-faithfulness + locked-decision discipline.

Currently: `DescentResidual = EvidenceUnknown(NonStrictEvidence) | EvidenceIncomplete`, where the prior 4→2-variant narrowing was specifically the **Director-ratified illegal-states-unrepresentable rationale** (making `EvidenceUnknown(Strict)` unconstructible via the typed `NonStrictEvidence` subset). The residual is not a Miss-class deferral — it's a typed witness of "descent execution attempted, here's why we don't have a per-path Strict witness." The "Miss-shape" framing in my original audit conflated the analyzer's runtime-failure surface with a design-laziness deferral.

**Pre-dispatch requirement**: any further dissolution proposal on this carrier MUST start from a grep-verified read of:
- `dsl/std/termination.dag` (substrate carrier authority)
- `docs/briefs/r3-substrate-descent-execution-proof-worker.md` (Director-ratified γ-shape rationale)
- `r3-program-plan.md` Q-EVAL-Descent-Termination-Contract (ratification commit)

And produce: (i) a concrete reason why the existing typed-residual shape is insufficient (i.e., what dispatch-level harm follows from keeping it); (ii) which authority doc would need amendment (if any); (iii) PM ratification before any worker dispatch.

**As written today**, the residual is compliant per existing Director ratification. The audit was wrong to label it Miss-class.

### §3.4 Empty-list `[] => Lookup::Miss` in generated code (4 sites)

Per operator framing on cost lens Case 5: lookups should not fail silently. Either use typed-key references (cost-table-with-guaranteed-presence) or fail-closed Diagnostic. Default for `lookup_cost([])` is `Diagnostic`, not `ConstantCost(0)`. Affects `lens_cost_symbolic_generated.rs` (3 sites) + `infer_helpers_generated.rs` (1 site).

### §3.5 `ClaimResult::NotYetImplemented` (21 sites) — per-gate disposition

Each `NotYetImplemented` is a gate-tier deferral. For R3 close:

- **Tier-1**: If the gate is in §1.8 R3-load-bearing scope (96 enumerated), the `NotYetImplemented` must be replaced with a real predicate evaluator OR the gate must be explicitly carved post-R3.
- **Tier-2**: Audit each of the 21 sites and route to corresponding Mgr (Verification owns most via TestRunner; lower.rs + test_runner.rs predicate sites).

Per `feedback_construction_over_ratchets` — `NotYetImplemented` is a textual ratchet that should dissolve when the predicate substrate lands.

### §3.6 `ArrowBody::Pending` — pre-lowering transitional state at declaration-tier

**Scope correction per inline review finding 2026-05-11**: `LensSurfacePending` (originally lumped here) is **terminal**, not in-progress. Per `src/v3/compiler/src/dag/effects.rs:197`, `LensSurfacePending` is a variant of `ParallelismUnsupportedKind` (explicitly marked 🟢 TERMINAL in code comments), representing an explicit unsupported-reason payload for the parallelism lens. The "Pending" suffix is misleading; the variant is a terminal classification ("this case is not supported, here's the named reason"), not a transitional state. It does NOT belong in this section and is removed.

The remainder of this section applies to `ArrowBody::Pending` only.



**Correction per inline review finding 2026-05-11**: my original framing said `ArrowBody::Pending` is on `Behavior::Transform.body`. That's **factually wrong**. Verified via grep at HEAD:
- `ArrowBody` enum lives at `src/v3/compiler/src/dag.rs:1092`
- It's used at `TypeConnective::Arrow { body, .. }` (see `bootstrap.rs:288`) — i.e., on the **type-connective `Arrow` carrier** that lives on `Declaration.connective`, NOT on `Behavior::Transform.body`.
- All `ArrowBody::Unparsed(...)` literal sites in `bootstrap_generated.rs` are inside `TypeConnective::Arrow { body: ArrowBody::Unparsed(...), .. }` patterns.

So the proposed dissolution (separate `UnresolvedSignature` from `ResolvedTransform`) targets the wrong substrate boundary. The actual boundary is **declaration-tier type-connective**: a `Declaration` whose `connective: TypeConnective::Arrow { body: ArrowBody::Pending }` has had its signature parsed but the body hasn't been lowered to a `Behavior` yet. Walkers that operate on `Behavior::Transform` already see only resolved bodies; the "paper-over" cost is at the type-connective-walking layer, not the Behavior walker layer.

**Revised proposal**: the substrate-shape question is whether the declaration-tier `TypeConnective::Arrow.body` should be a sum that includes `Pending` (allowing pre-lowering declarations to coexist with post-lowering ones), or whether two separate substrate types should partition the pre-/post-lowering states. This decision lives in the v3 dag substrate authority (`src/v3/M1_DESIGN.md` + `dag.rs`), is substantial substrate-shape work, and per `feedback_substrate_shape_belongs_in_mgr_canvas` belongs in a Substrate Mgr canvas with PM ratification on R3-load-bearing-ness.

**Pre-dispatch requirement**: PM ratification on R3-load-bearing-ness; Substrate Mgr canvas authoring on the partition-vs-sum-with-Pending design question, citing `M1_DESIGN.md` authority + per-walker impact (which walkers/lenses actually touch the type-connective layer vs only Behavior).

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
3. **§3.3 (descent-execution-proof residual)** — **BLOCKED on authority-reconciliation** per §3.3 corrected disposition. The carrier is compliant per Director-ratified γ-shape at `docs/briefs/r3-substrate-descent-execution-proof-worker.md`. Any dissolution requires reading the existing authority + producing a grep-verified reason the typed-residual shape is insufficient, then PM ratification. No same-batch dispatch with §3.1.
4. **§3.4 (empty-list-as-Miss)** — generated-code regen needed; folds into §3.1 + §3.5 dispatch.
5. **§3.5 (NotYetImplemented audit)** — per-gate Mgr-tier dispatch; recommend Verification Mgr re-spawn (currently archived per overnight cascade) authors the audit.
6. **§3.6 (ArrowBody::Pending)** — larger substrate-shape work; **R3-load-bearing-ness needs PM ratification before dispatch** (may need post-R3 carve).
7. **§3.7 (boundary-tool-in-interior)** — per-file Mgr-canvas authoring; spread across Mgrs. Each canvas owns its per-site classification (boundary tooling vs interior substrate flow) before any conversion work.
8. **§3.8 (structural_coverage_gap)** — promote/carve audit; PM-coordinated.
9. **§4 (review-process)** — PM authors the PR-template ratchet update; cross-Mgr coordination.

**Authority-gate summary**: items 2 (DescentUnknown), 3 (descent-execution-proof residual), and 6 (ArrowBody::Pending) are explicitly authority-blocked until reconciliation. Items 1/4/5/7/8/9 may dispatch on the standing Mgr-canvas authority once R3-scope-ratified.

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
