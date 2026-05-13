# R3 Gate 87 Lens-Completeness Test-Discipline Decomposition

Date: 2026-05-13

Owner: Verification Manager lane dispatch (`zesty-bear-119`).

Scope: decompose the remaining test-discipline cementing work around the lens-completeness invariant. Gate #87 itself is already `CONSUMER_LANDED + PASSING` for the `src/v3/compiler/regen.dag` registry corpus. This brief does not reopen that closure; it turns the landed discipline into concrete sub-items for the adjacent lens-completeness invariant, especially rows that are blocked today by carrier authoring or by non-`regen` lens-completeness surfaces.

## Authority

- `docs/r3-structure.md` Acceptance, `lens_cementing_test_discipline_complete`.
- `docs/r3-program-plan.md` section 1.8 rows #79, #80, #81, #82, #83, #87, and #104.
- `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md`.
- `docs/briefs/r3-gate-87-lens-cementing-closure-audit.md`.
- `docs/design-tests-as-data-completeness.md` sections 5.2-5.4.
- `TESTING.md` Band-C cementing discipline.
- `docs/v3-lens-capability-register.md`.
- `src/v3/compiler/regen.dag`.
- `src/v3/compiler/tests/integration/sg0_census_test.rs` `EXPECTED_HAND_AUTHORED_TEST`.

## Lens-Completeness Invariant

Any lens row that claims `BEHAVIORALLY COMPLETE`, or is promoted into the R3 complete-lens set, must have a cementing receipt that would fail on silent semantic drift:

1. Real v2 counterpart: a `DifferentialEquals` or frozen-oracle equivalent over the shared fixture.
2. v3-native lens: a `LensOutputEquals` receipt over minimal `Dag` shapes, or a named temporary Rust receipt where the carrier cannot yet be authored in `.dag`.
3. Helper / N/A scope: explicit `Compiles` plus a paired Rust pin receipt or an explicit non-behavioral disposition.
4. No parallel inventory: the live dispatch source is the SG-0 census plus the gate-87 runner inventory, not a hand-maintained alternative table.

## Dispatch Sub-Items

### G87-D1: Sum-Typed Lens Expected-Carrier Authoring

Goal: unblock `.dag` `LensOutputEquals` cementing receipts for lens outputs whose expected values are currently too rich to author as data.

Owned surfaces:
- `src/v3/std/verification.dag` expected-value / predicate carrier shape.
- Existing gate-87 `.dag` receipts for `provenance` and `variant_payload`.
- Rust pin receipts named in `docs/briefs/r3-cementing-discipline-pattern-2026-05-12.md` section 3.

Concrete acceptance:
- `.dag` TestClaims can express expected values for `Origin` and `VariantPayloadShapeLookup`-class outputs, or the worker records the exact smaller substrate carrier that must land first.
- `t_r3_gate_87_cementing_regen_provenance.dag` and `t_r3_gate_87_cementing_regen_variant_payload.dag` are upgraded from seam/compile placeholders where possible.
- Any replaced Rust receipt is removed from the SG-0 census in the same PR.
- If not fully unblockable, the worker lands a narrow blocker brief with owner lane, target carrier, and the exact Rust receipt that remains.

### G87-D2: Complexity / Cost Report-Predicate Carrier Cementing

Goal: close the cementing gap for complete complexity and cost lens reports without treating hand-Rust frozen receipts as terminal.

Owned surfaces:
- Complexity/cost cementing tests under `src/v3/compiler/tests/integration/cementing/`.
- Gate #73 report-predicate carrier authoring interface.
- Gate-87 `.dag` receipts for `cost` and `cost_symbolic`.

Concrete acceptance:
- `ComplexitySummary`, `SymbolicCost`, `SizeVariable`, and dimension-bearing cost outputs have an authorable `.dag` expected-value path or a documented blocker tied to gate #73.
- Existing Rust frozen-oracle receipts are either replaced by `.dag` `DifferentialEquals` / `SymbolicCostExprEquals` claims or explicitly narrowed to a temporary pin with a dissolution trigger.
- The PR body states the SG-0 census delta and the Band-C predicate class used.

### G87-D3: Complete-Lens Register Ratchet For Promoted R3 Lenses

Goal: keep the lens-capability register, `regen.dag`, and cementing receipts synchronized as parallelism and effect-enumeration move to `BEHAVIORALLY COMPLETE`.

Owned surfaces:
- `docs/v3-lens-capability-register.md`.
- `src/v3/compiler/regen.dag`.
- `src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs`.
- `src/v3/compiler/tests/dag/t_r3_gate_87_cementing_regen_*.dag`.
- `src/v3/compiler/tests/dag/cementing_dispatch.dag`.

Concrete acceptance:
- Add or tighten a ratchet so a `BEHAVIORALLY COMPLETE` `regen.dag` row cannot land without a matching gate-87 runner suite and cementing `TestClaim`.
- Exercise at least one promoted-lens transition path, preferably `effect_enumeration` or `parallelism`, using the correct Band-C predicate class.
- `cargo test -p v3-compiler r3_gate_87` passes or the worker reports the smallest failing command and owner.

### G87-D4: Lens Read Witness Coverage Cementing

Goal: connect gate #104 (`lens_read_witness_shape_dissolved`) to the test-discipline invariant so complete lenses do not regress through `Lookup<C>::Miss`-style read holes.

Owned surfaces:
- `src/v3/lenses/*.dag` read functions.
- Generated lens Rust surfaces under `src/v3/compiler/src/lens_*_generated.rs`.
- TestClaim or grep-ratchet receipts for the terminal no-`Lookup<` / no-`::Miss` condition.

Concrete acceptance:
- Add a test receipt that every complete lens read channel returns `Witness<C>::Inhabits` or `Witness<C>::Violates { reason, at }` and never a bare miss/defer surface.
- If a universal `.dag` quantifier is not available, add the narrowest deterministic ratchet using the existing generated files and name the dissolution trigger.
- Document how this receipt composes with the Band-C cementing receipt rather than replacing it.

### G87-D5: Non-`regen` Complete-Lens Census Reconciliation

Goal: prevent the gate-87 registry corpus from being misread as the full lens universe.

Owned surfaces:
- `docs/v3-lens-capability-register.md`.
- `src/v3/compiler/tests/integration/sg0_census_test.rs`.
- Any non-`regen` lens cementing tests or blockers.

Concrete acceptance:
- Derive the non-`regen` complete-lens slice from `docs/v3-lens-capability-register.md` and the SG-0 census; do not create a separate persistent inventory.
- For each derived row, update the canonical register / census comments with the classification: already cemented, needs Band-C receipt, blocked by carrier authoring, or not a behavioral lens.
- Dispatch concrete follow-on ports for every actionable row and update the canonical census / register surfaces in the same PR where a test moves.

## Dispatch Order

D1 and D2 are carrier-unblockers and can run in parallel. D3 can start immediately as a ratchet-only pass, then absorb concrete promoted-lens receipt edits as they become available. D4 should run alongside the gate #104 migration because it is a shape invariant over lens read channels. D5 is the reconciliation sweep that prevents closure drift after D1-D4 land.

## Manager Closeout Receipt

This decomposition is complete when matching dashboard work items exist for D1-D5 and this brief is merged. Child PRs own their implementation receipts; this manager PR owns only the decomposition and dispatch artifact.
