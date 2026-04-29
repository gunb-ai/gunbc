# R2 Release — B5 Loop construction-closure audit `(S, R2-coupled)`

> **CLOSED — receipt brief.** Construction-closure **holds**; structural gate landed at `src/v3/compiler/tests/integration/r2_b5_loop_construction_closure_test.rs` (2026-04-27). Synthesis Tier 2 §5 and ROADMAP record **RESOLVED**; speculative `LoopKind` marker **retired** per `feedback_construction_over_ratchets`. **Do not dispatch** this file as open audit work — reopen only if lowering regresses (new `Behavior::Loop` sites or test failure).

> **R2 Release Manager dispatch.** Per [`docs/briefs/debt-paydown-synthesis-2026-04-25.md` §"Tier 2"](debt-paydown-synthesis-2026-04-25.md) item 5 + [`docs/r2-structure.md` §"R2 Release Manager"](../r2-structure.md). **MUST start with audit, NOT marker design.** Reports to R2 Release Manager once R2 spawns; pre-spawn authoring per inbox #828 PM portion.

## Read first

- **[`docs/briefs/debt-paydown-synthesis-2026-04-25.md` §"Tier 2" item 5 (lines 300-314)](debt-paydown-synthesis-2026-04-25.md)** — the parent scope statement. Quoted: *"Brief MUST start with a construction-closure audit, NOT with a marker design. ... Both proposed paths in the original PR #809 row are bridges; do not author the marker brief blind. Per `feedback_construction_over_ratchets`, prefer the structural-closure outcome."*
- **`feedback_construction_over_ratchets`** — model first; violations dissolve. Loop-emission marker is a ratchet; construction-closure (paths-route-through-recursive-function-lowering) dissolves the need for the marker.
- **`src/v3/compiler/src/lower.rs`** — search location for `Behavior::Loop` construction sites. The audit's primary surface.
- **`src/v3/std/substrate.dag`** — `Loop` connective declaration. The substrate-side authority on what Loop is.
- **PR #809** — original entry that surfaced the Loop-emission semantic invariant question. Documents the marker / test framing as bridges.
- **[`docs/escalation-paths.md`](../escalation-paths.md)** — escalation channel + decision-artifact discipline.
- **[`MODELING.md`](../../MODELING.md)** + **[`INVARIANTS.md`](../../INVARIANTS.md)** + **[`CODING.md`](../../CODING.md)**.

## Frame — audit-first, marker conditional on audit refusal

The synthesis-doc question: does every `Behavior::Loop` construction site route through recursive-function lowering? If yes, **construction-closure holds** — Loop is structurally only-reachable via the recursive-function lowering path, and the speculative `LoopKind` marker is unneeded. If no, the marker (or test framing) becomes load-bearing because Loop's source is unconstrained.

The brief is audit-first because authoring the marker brief blind is a `feedback_construction_over_ratchets` violation: it ratchets a marker into the substrate before knowing whether closure already obtains.

## Three consumer-side requirements

1. **Construction-closure audit.** Enumerate every `Behavior::Loop` construction site in `lower.rs` (and anywhere else — grep `src/` for `Loop {` / `Loop::new` / `behaviors::Loop` patterns + any explicit `Behavior::Loop` constructor calls). For each site, trace the call path back to its caller. Determine whether the path originates from recursive-function lowering or from another source. **Acceptance:** site enumeration in PR description; per-site path trace; binary verdict (closure-holds vs closure-fails).

2. **Branch on audit outcome:**
   - **If closure holds (preferred per `feedback_construction_over_ratchets`):** deliverable is a **structural integration test** that asserts construction-closure. Test shape: walk the lowered Dag for any program; assert that every `Behavior::Loop` carries provenance traceable to recursive-function lowering. Marker brief is **retired** as a speculative ratchet that didn't need landing. Update the synthesis doc PR #809 entry to mark this audit-resolved.
   - **If closure fails:** brief turns into a `LoopKind` lowering-marker spec. Deliverable: marker-shape design (which sources of Loop need disambiguation; what `LoopKind` variants are needed; lowering invariant the marker enforces). This becomes a substrate-amendment proposal that escalates per the synthesis-doc "STOP-AND-ESCALATE" discipline.

3. **Document outcome.** Either way, update the synthesis-doc Tier 2 §5 entry to reflect the resolved state. If closure holds: row marks RESOLVED (structural test landed). If closure fails: row marks REOPENED (marker brief authored, escalated to substrate-amendment).

## Slice — audit → branch → document

1. Audit construction sites (this is the load-bearing first step).
2. Branch on outcome:
   - Closure-holds path: author structural integration test; land; update synthesis row.
   - Closure-fails path: author marker spec; escalate per synthesis-doc STOP discipline.
3. Update synthesis-doc Tier 2 §5 entry with resolved state.

Single PR (audit + branch + document) for the closure-holds path. Closure-fails path likely needs ≥2 PRs (audit-receipt + marker-spec authoring as separate dispatch).

## Acceptance

- [x] Site enumeration captured in PR description.
- [x] Per-site path trace from `Behavior::Loop` construction back to caller.
- [x] Binary verdict (closure-holds vs closure-fails) explicitly stated — **closure-holds**.
- [x] If closure-holds: structural integration test lands; gate verification clean (`cargo test --workspace --exclude v2-compiler-tests`); marker brief idea retired in synthesis doc.
- [ ] If closure-fails: marker spec authored as separate brief; escalation to substrate-amendment per synthesis-doc STOP discipline; this brief closes as audit-receipt-only. *(N/A — closure-holds path taken.)*
- [x] Synthesis-doc Tier 2 §5 row updated to RESOLVED or REOPENED — **RESOLVED**.
- [x] DB-8 fixed-point converges bit-identically (regardless of audit outcome).

## STOP-AND-ESCALATE

Per [`docs/escalation-paths.md`](../escalation-paths.md):

- **Audit reveals a Loop construction site whose origin is ambiguous** (cannot determine whether it routes through recursive-function lowering) → STOP. Do not declare closure-holds without full coverage. Surface the ambiguous site for design clarification.
- **Closure fails AND the marker spec turns out to require new substrate connective** → STOP. Per synthesis-doc and `feedback_compiler_is_dag_processor`, substrate amendment is C1-class; escalate to Director (Director opens C1 substrate-capability lane if work requires substrate).
- **Audit reveals construction sites I added in this brief's setup** (e.g., temporary scaffold) → STOP. Confirm this is not a self-fulfilling closure case before declaring; closure must be a property of the existing code, not of the audit's own additions.

## Cross-refs

- Parent: [`docs/briefs/debt-paydown-synthesis-2026-04-25.md` §"Tier 2" item 5](debt-paydown-synthesis-2026-04-25.md).
- R2 Release Manager scope: [`docs/r2-structure.md` §"R2 Release Manager"](../r2-structure.md) Goal 5 / B-wave Tier 2.
- Substrate authority: [`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag) — `Loop` connective.
- Discipline anchor: `feedback_construction_over_ratchets`.
- Originating issue: PR #809.
- Escalation discipline: [`docs/escalation-paths.md`](../escalation-paths.md).

## Closure receipt (LANDED 2026-04-27)

**Verdict: closure-holds.** Site enumeration + per-site traces + structural gate landed in commit `3e7696f1c` (`audit(v3): loop construction-closure holds + integration test`). Authoritative rows: [`debt-paydown-synthesis-2026-04-25.md`](debt-paydown-synthesis-2026-04-25.md) Tier 2 item 5 (RESOLVED); [`ROADMAP.md`](../../ROADMAP.md) Loop emission semantic invariant (B5). Integration gate: `src/v3/compiler/tests/integration/r2_b5_loop_construction_closure_test.rs`.
