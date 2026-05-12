# R3 Wave-1 S4 — bridge_retirement_ledger_zero (#36)

**Owner**: Wave-1 Substrate worker
**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Authoring date**: 2026-05-12

---

## §0. Status — DISPATCH-READY

Small, isolated gate. Closes the Bridge-Retirement lane via unified-ledger-zero demonstration.

## §1. Scope

Demonstrate that the unified Bridge-Retirement ledger has zero outstanding entries. This is a verification-shape closure, not new substrate.

### Phase A — Locate the unified ledger

Grep authority: `docs/audit/r3-bridge-retirement-*` + `dsl/std/` for the ledger carrier. Likely shape:
```
data bridge_retirement_ledger: List<BridgeRetirementEntry> = []
```
or equivalent (a list/set carrier currently holding zero entries OR holding entries that all carry a `retired: true` marker).

### Phase B — Verify zero outstanding

Author a test (`.dag` test claim preferred per TESTING.md band-A) that asserts:
- `bridge_retirement_ledger` is empty, OR
- Every entry in the ledger carries the retired-discriminator
- The test is structural (pattern-match on the carrier shape), not just a length check

### Phase C — Close the gate

Wire the test into `§1.8` or the appropriate gate-closure surface; close gate #36 with the receipt.

## §2. STOP conditions

1. **Ledger has outstanding entries** — if the ledger is NOT empty at landing time, **STOP** — this gate's prerequisite (other Bridge-Retirement lane work) isn't complete. Surface to warm-wolf-698 with the entries listed.
2. **Ledger carrier mis-identified** — if multiple candidate carriers could be "the" unified ledger, **STOP** — substrate-authority question. Don't pick one arbitrarily.
3. **Test substrate gap** — if the test claim shape can't be expressed structurally and would need ad-hoc Rust, **STOP** — that's a substrate gap, not a gate-closure step.

## §3. Verification

- `cargo test --workspace`
- Test-claim pattern follows existing band-A precedent in `dsl/std/verification.dag` or equivalent
- PR body cites the specific ledger carrier + zero-receipt mechanism

## §4. PR body framing

- Cite gate #36 closure
- Cite the unified-ledger authority
- Receipt-level test result inline

## §5. Out of scope

- Adding entries to the ledger (this gate is the zero-state assertion, not the population step)
- Other Bridge-Retirement work — that's separate lane history

## §6. Reference

- `docs/r3-remaining-work-dependency-graph.md:128` — gate-row metadata
