# R3 Gate 87 Lens Cementing Test-Discipline Decomposition — 2026-05-13

**Owner:** Verification Mgr lane, decomposition node `adhoc-b75b3d90-3d0`.

**Purpose:** dispatch concrete follow-on work that keeps gate #87
`lens_cementing_test_discipline_complete` honest after the initial
`CONSUMER_LANDED + PASSING` receipt. This packet does not reopen gate #87;
it decomposes the remaining test-discipline cementing work around the
lens-completeness invariant.

**Authorities:**
- `TESTING.md` section "Cementing tests (Band C — lens subsumption)".
- `docs/design-tests-as-data-completeness.md` §5 / §C5 / §8.3.
- `docs/v3-lens-capability-register.md` Discipline section.
- `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md`.
- `src/v3/std/verification.dag` `LensCapabilityRegisterRow`,
  `TestPredicate`, and `CementingDispatchMatchesProjection`.
- `src/v3/compiler/regen.dag` `LensRegistryEntry` rows.
- `src/v3/compiler/tests/dag/cementing_dispatch.dag`.
- `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`.

## Invariant

A lens row may claim `BEHAVIORALLY COMPLETE` only when the same change that
creates or preserves that claim also lands a Band-C cementing receipt:

- real v2 counterpart: a same-source v2/v3 differential receipt, or a frozen
  reviewed v2 projection while direct `.dag` carrier equality is blocked;
- v3-native lens: a receipt for the published v3 behavior, preferably
  `LensOutputEquals`, with a named temporary Rust receipt only when the output
  carrier is not authorable as `.dag` expected data yet;
- helper / `N/A` rows: explicit non-complete or non-behavioral classification,
  not an implicit exception.

The completeness claim, register row, runner inventory, `.dag` receipt, and any
temporary Rust receipt must move together. Drift must fail closed.

## Dispatch Items

### G87-A — Register / Regen Drift Ratchet

**Scope:** strengthen the existing dispatch projection around the canonical
register and regen registry.

**Files to inspect first:**
- `src/v3/std/verification.dag`
- `src/v3/compiler/regen.dag`
- `src/v3/compiler/tests/dag/cementing_dispatch.dag`
- `src/v3/compiler/src/cementing_dispatch.rs`
- `src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs`

**Acceptance:**
1. Adding a behaviorally complete real-v2 `LensRegistryEntry` without a Band-C
   receipt fails a focused test.
2. Adding a receipt whose `registry_name`, `module_stem`, or `kind` does not
   match the closed projection fails a focused test.
3. Existing `cargo test -p v3-compiler r3_gate_87` remains green.
4. No new prose-only inventory is introduced.

### G87-B — Temporary Rust Receipt Dissolution Plan

**Scope:** replace temporary Rust receipts with `.dag` claims where the expected
carrier is now authorable; for still-blocked rows, leave one named blocker and
owning lane.

**Files to inspect first:**
- `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md` §3
- `src/v3/compiler/tests/integration/cementing/`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag`
- `src/v3/compiler/tests/integration/sg0_census_test.rs`

**Acceptance:**
1. At least one eligible temporary Rust receipt is deleted and replaced by a
   `.dag` `TestClaim`, or the worker proves none are currently eligible and
   updates the disposition table with precise blocker evidence.
2. Any deletion decrements the SG-0 hand-authored census in the same PR.
3. Each still-temporary Rust receipt names the missing carrier/predicate support
   and owning lane in exactly one place.
4. No Rust receipt remains as a parallel duplicate of an executable `.dag`
   behavioral claim.

### G87-C — Non-`regen.dag` Complete-Lens Sweep

**Scope:** make sure complete lenses outside the generated-lens registry are
covered by the same Band-C discipline through the capability register rather
than silently bypassing gate #87's enumeration boundary.

**Files to inspect first:**
- `docs/v3-lens-capability-register.md`
- `src/v3/std/verification.dag`
- `src/v3/lenses/*.dag`
- `src/v3/compiler/tests/integration/lens_register_correspondence_test.rs`
- `src/v3/compiler/tests/integration/canonical_lens_bridge_ratchet_test.rs`

**Acceptance:**
1. Every `STRUCTURALLY TERMINAL` + `BEHAVIORALLY COMPLETE` lens not represented
   by `regen.dag` has either a Band-C receipt or an explicit `N/A` rationale
   tied to the canonical register row.
2. The sweep does not broaden gate #87's `regen.dag` runner corpus by accident;
   non-registry rows stay on the register-correspondence ratchet.
3. Any newly discovered complete-row gap becomes a same-PR receipt or a
   dashboard blocker with the owning lane named.

### G87-D — Worker-Facing Cementing Port Template

**Scope:** author a small reusable template for future lens-completeness flips
so workers know which files must change together.

**Files to inspect first:**
- `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md`
- `docs/briefs/r3-v-cluster-m-87-cementing-worker.md`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_cost.dag`
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_provenance.dag`

**Acceptance:**
1. Template covers real-v2, v3-native, helper/`N/A`, and temporary Rust blocker
   cases.
2. Template requires same-PR updates to register data, prose mirror, runner
   inventory, `.dag` receipt, temporary Rust receipt, and SG-0 census when
   applicable.
3. Template explicitly forbids creating a second cementing inventory outside the
   register / dispatch projection.

## Sequencing

G87-A and G87-D can run immediately and independently. G87-B should consume
G87-D if the template lands first, but it does not block on it. G87-C can run in
parallel with G87-A because it audits non-`regen.dag` complete rows and should
not edit the dispatch projection unless it finds a real register mismatch.

## Verification

Docs-only decomposition changes do not require a cargo run. Any worker that
edits runner code, `.dag` claims, Rust receipts, or the SG-0 census must run:

```bash
cargo test -p v3-compiler r3_gate_87
```

Broader worker PRs should also run the smallest affected integration test named
by their changed file.

---

**End of decomposition.**
