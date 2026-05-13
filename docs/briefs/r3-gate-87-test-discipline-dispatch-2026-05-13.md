# R3 Gate 87 Test-Discipline Cementing Dispatch - 2026-05-13

**Owner:** Verification Mgr lane closeout (`valiant-cat-190`).

**Purpose:** decompose the remaining test-discipline cementing work around gate #87
(`lens_cementing_test_discipline_complete`) into dispatchable sub-items. Gate #87
itself is already `CONSUMER_LANDED + PASSING` for the `regen.dag`
`LensRegistryEntry` corpus; this brief does not reopen that status. It turns the
residual discipline work into concrete child scopes so the lens-completeness
invariant keeps a live path from temporary Rust pins / `Compiles` placeholders to
`.dag` `TestClaim` receipts.

## Authority

- `TESTING.md` section "Cementing tests (Band C - lens subsumption)".
- `docs/design-tests-as-data-completeness.md` section 5 and section C5.
- `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md`.
- `docs/briefs/r3-gate-87-lens-cementing-closure-audit.md`.
- `docs/v3-lens-capability-register.md`.
- `src/v3/compiler/regen.dag`.
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`.
- `src/v3/compiler/tests/dag/cementing_dispatch.dag`.
- `src/v3/compiler/tests/integration/sg0_census_test.rs`
  `EXPECTED_HAND_AUTHORED_TEST`.

## Dispatch Items

### G87-D1 - Carrier-Blocked Rust Cementing Receipts

**Scope:** audit and classify the Rust receipts that remain under
`src/v3/compiler/tests/integration/cementing/` and
`r3_gate_87_lens_cementing_regen_receipts_test.rs` because their expected
carriers are not yet authorable as `.dag` test data.

**Concrete targets:**

- `cementing_provenance_origin_integration_test.rs` - keep blocked on
  sum-typed `Origin` expected values until the existing provenance `.dag` receipt
  can express the per-`Behavior` mirror.
- `complexity_lens_behavioral_completion.rs` - keep blocked on
  `Gate73_ReportPredicateCarriers` until `ComplexitySummary` is authorable in a
  `TestPredicate`.
- `memory_peak_cost_basis_demo.rs` - keep blocked on the parser-level
  `apply_lens(cost, DeclarationScope, Enforce { ... })` consumer.
- `r3_gate_87_lens_cementing_regen_receipts_test.rs` - split only by real
  carrier unblock; do not port the whole host ratchet as one Band-C row.

**Acceptance:** update the live disposition surface in
`r3-cementing-discipline-pattern-2026-05-12.md` if any target has become
unblocked; otherwise land an audit receipt confirming the blockers and owner
lanes still match `sg0_census_test.rs`.

### G87-D2 - Helper Placeholder Dissolution Plan

**Scope:** turn the remaining gate-87 `Compiles` helper placeholders into
explicit dissolve-on-unblock tasks, without treating them as behavioral evidence
today.

**Concrete targets:**

- `t_r3_gate_87_cementing_regen_infer_helpers.dag`.
- `t_r3_gate_87_cementing_regen_lower_helpers.dag`.
- `t_r3_gate_87_cementing_regen_variant_payload.dag`.
- Paired Rust compile / unit pins in
  `r3_gate_87_lens_cementing_regen_receipts_test.rs` and
  `src/v3/compiler/src/lib.rs::variant_payload::tests`.

**Acceptance:** for each placeholder, verify the harness comment names the
stronger replacement predicate, the owning unblock lane, and the paired Rust pin.
If a carrier is now authorable, replace the placeholder with `LensOutputEquals`
and update the runner inventory plus dispatch list in the same PR.

### G87-D3 - Single-Authority Dispatch Ratchet

**Scope:** cement the rule that every new or changed gate-87 registry cementing
row moves through the four single-authority surfaces together.

**Concrete targets:**

- `src/v3/compiler/regen.dag` `LensRegistryEntry` rows.
- `R3_GATE_87_CEMENTING_REGEN_SUITES`.
- `tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag`.
- `tests/dag/cementing_dispatch.dag`.
- `sg0_census_test.rs` classification for temporary Rust pins.

**Acceptance:** add or refresh a lightweight audit/ratchet receipt proving the
runner inventory, dispatch projection, and SG-0 cementing classification agree.
For code or fixture edits, run:

```bash
cargo test -p v3-compiler r3_gate_87
```

Docs-only disposition refreshes may cite the current passing gate and skip the
heavy test run.

## Dispatch Rule

Child workers must not introduce a parallel cementing inventory. The live row
set comes from `regen.dag`, `cementing_dispatch.dag`, the runner table, and
`EXPECTED_HAND_AUTHORED_TEST`. Any replacement that retires a Rust receipt must
ship the `.dag` claim, runner/dispatch updates, and SG-0 census decrement in the
same PR.
