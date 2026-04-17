> Part of: [THESIS.md](../THESIS.md) > [src/v3/ROADMAP.md](../src/v3/ROADMAP.md)

# Post-L1.5 Phase Plan

**Status:** Plan. Phase 1 lanes detailed below; P2–P4 stubbed.
**Owner:** Sequential (one phase at a time).
**Cadence:** 3 lanes per phase, ~3 weeks per phase, ~12 weeks total.

Detailed design docs for each phase are written at the **start of the
prior phase**, so they benefit from what the running phase learns
without going stale.

---

## Phase map

| Phase | Weeks | Lanes | Theme |
|---|---|---|---|
| **P1 Foundation** | 1–3 | 3 | Close L1.5 tail, establish clean-emission invariant, design consolidation |
| **P2 Consolidation** | 4–6 | 3 | Collapse `emit_*.rs` into generic walker + target specs |
| **P3 Self-host + symbolic** | 7–9 | 3 | Self-hosting cycle through consolidated emitter, L2 M1 symbolic bounds |
| **P4 Diagnostics + breadth** | 10–12 | 3 | Diagnostics-as-corrections, Verilog/SPICE targets |

**Hard sequencing:** P2 before P3 (self-hosting inherits emitter
fragmentation). P1 before P2 (consolidation needs the clean-emission
contract as its north star).

**Soft sequencing:** Within a phase, lanes may run in parallel where
dependencies allow. Cross-lane coordination must be explicit — if a
lane blocks another, surface it; don't wait silently.

---

## P1 Lane overviews

Full design docs linked per lane. Each design doc carries scope, direction,
escalation criteria, and acceptance gates.

### [P1-L1: L1.5 tail — ownership Phase 2 + ignore-hygiene](./phase1-lane1-l15-tail.md)
Close the remaining L1.5 ownership work and unignore the 6 explicitly-gated
tests (4 Go cross-target placeholders + 2 Phase 2 Consumed contracts). Also
audit receipts on new coproducts and close m1_3 compile-once harness coverage.

### [P1-L2: Clean-emission invariant](./phase1-lane2-clean-emission-invariant.md)
**The novel piece.** Establish warnings-by-construction as a structural
invariant. Each target spec declares clean-code constraints; emission
respects them by construction. Pilot with one concrete emitter fix to
validate the shape before P2 codifies it across all targets.

### [P1-L3: Single-emitter consolidation build plan](./phase1-lane3-consolidation-build-plan.md)
Pure design. Produce the file-by-file map for collapsing `emit_rust.rs`,
`emit_go.rs`, `emit_python.rs` into one generic walker + three target
specs. Choose P2's pilot target (SPICE or English). Enumerate substrate
gaps before any P2 code starts.

---

## P2–P4 lane stubs

These lanes will get design docs at the start of P1. Placeholder below so
the shape is visible.

### Phase 2 — Consolidation (Weeks 4–6)
- **P2-L1:** Pilot target implementation (SPICE or English from P1-L3).
  Validates the generic walker against the simplest target before
  tackling Rust/Go/Python.
- **P2-L2:** Rust consolidation. `emit_rust.rs` dissolves into spec
  additions + walker dispatch.
- **P2-L3:** Go + Python consolidation. Inherits from L2; largely
  mechanical once the Rust case proves the pattern.

### Phase 3 — Self-hosting + symbolic (Weeks 7–9)
- **P3-L1:** Self-hosting cycle through consolidated emitter.
  Acceptance: fixed-point Rust output from `compiler.dag`.
- **P3-L2:** L2 M1 symbolic bounds. Unignores
  `kf_1_lambda_body_cost_contributes_to_fold`. Adds O(n) vs O(n²)
  diagnostics.
- **P3-L3:** Parallelism-as-lens. Unignores
  `parallel_fold_on_commutative_monoid_is_reducible`. The "promotable
  to map" diagnostic from the thesis doc becomes a lens output.

### Phase 4 — Diagnostics + breadth (Weeks 10–12)
- **P4-L1:** Diagnostics-as-corrections. Every diagnostic carries a
  literal fix snippet.
- **P4-L2:** Verilog target. First hardware validation of the
  consolidated emitter.
- **P4-L3:** Second non-programmatic target (SPICE or English,
  whichever P2-L1 did NOT pilot). Completes the thesis's
  cross-domain emission claim.

---

## Coordination protocol

**Per-lane kickoff:** implementer reads the lane's design doc end-to-end
and confirms scope + escalation criteria before writing any code.

**Mid-lane:** if a lane hits an escalation trigger, stop work and surface
the issue. Do NOT expand scope silently. Expanding scope inside a lane
breaks the phase's time budget.

**Per-phase wrap:** before starting the next phase, all lanes green, all
acceptance gates hit, next phase's design docs reviewed.

**Cross-phase:** if a phase overruns by more than one lane-week, reassess
the remaining phases' scope. A consistent overrun pattern means the
lanes are too large.

---

## What NOT to do

- **Do not add a fourth per-language emit file.** Any new target
  language must wait until P2 completes. Adding `emit_verilog.rs` or
  `emit_spice.rs` before consolidation makes the consolidation
  proportionally harder.
- **Do not suppress warnings at consumer sites.** The clean-emission
  invariant (P1-L2) forbids `#[allow(...)]` escape hatches in emitted
  code. Band-aids block the structural fix.
- **Do not start implementing P2 before P1-L3 completes.** The build
  plan exists specifically to catch substrate gaps before they become
  code you have to rewrite.
- **Do not defer receipts on new coproducts.** Every new sum type
  added anywhere in the compiler gets a 🟢/🟡/🔴 receipt at declaration
  time. This is not P1-only; it's a standing rule.
