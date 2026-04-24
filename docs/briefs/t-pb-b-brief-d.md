# T-PB-B — Brief D (parallel to T-PB-A)

**Status:** pre-landing inventory + draft `TestClaim` fixtures.  
**Authority:** Post-R2 residuals in `TESTING.md` (compiler-internal `#[cfg(test)]` under `src/v3/compiler/src/`; boundary tests invoking external toolchains). **Schema:** `src/v3/std/verification.dag` (`TestClaim`, `TestPredicate`, `requires: List<ResourceReference>`).

**Gates (unchanged):** Do not remove or replace existing Rust integration tests as the source of truth. Do not assert `pb_test_file_generated_from_dag` or `pb_rust_tests_outside_residual_zero` until Testgen signals. Draft `.v3` / `.dag` modules here are **fixtures** for eventual runner wiring.

**Fixtures on disk:** `src/v3/compiler/tests/fixtures/t_pb_b_brief_d/*.v3` — each file is a self-contained v3 module declaring `TestSuite` / `TestClaim` values. **Compile smoke:** `t_pb_b_brief_d_fixture_smoke_test` in `tests/integration/` (lowers cleanly; not a pure-bootstrap gate).

**Duplicate authority (bounded, P2/P5):** The same three claims exist here (`.v3`, suite names `t-pb-b/…`) and under T-PB-B-1 (`.dag`, suite names `t-pb-b-1/…`). Neither path is runner-evaluated yet. **Dissolution trigger (named):** remove or shrink the `.v3` copies once Testgen accepts `src/v3/compiler/tests/dag/t_pb_b_1_*.dag` as the single maintained source for those claims—until then, any edit to claim text must keep both paths aligned (each `.v3` file header points at its `.dag` sibling).

**T-PB-B-1 (landed `.dag` home):** `src/v3/compiler/tests/dag/*.dag` + `docs/briefs/t-pb-b-1.md` + `t_pb_b_1_tests_dag_smoke_test` — first batch as **`data` declarations** in real `.dag` modules (same claims as Brief D fixtures; runner + Rust deletion still gated on Testgen).

---

## Legend (D / G / A / B)

| Tag | Meaning |
|-----|---------|
| **D** | Drop-in `.dag` / `TestClaim` candidate once Testgen runner evaluates today’s `TestPredicate` vocabulary (`Compiles`, `FailsWithDiagnostic`, `OutputEquals`, `PortHasState`, `CostBounded`, plus `BehavioralObservation` / `MockBackedInvariant` when oracles exist). |
| **G** | Predicate or carrier **gap** — intent ports to data, but needs schema extension, lens-backed observation, or substrate facts before Rust can be retired for that assertion. |
| **A** | **Residual A** (meta): harness, repo layout, `include_str!` golden glue, `integration.rs` wiring, SG census — not boundary; still not `.dag`-migratable until declarative census / manifest story exists. |
| **B** | **Residual B** (boundary): rustc / Go / Python / emitted roundtrips — subprocess or external toolchain. |

**v2 suite** (`src/v2/tests/**`): out of scope for T-PB-B unless a director decision adds dual-oracle **B** work.

---

## Extended inventory — `v3-compiler` consolidated harness

Paths are modules under `src/v3/compiler/tests/` unless noted. Primary classification is **first** tag; **G** can combine with **D** (e.g. D+G).

### Boundary directory (`tests/boundary/`)

| Module | Pipeline vs contract | Tags | Forever Rust (when applicable) |
|--------|----------------------|------|--------------------------------|
| `m1_3_emit_go_test` | Contract on emitted Go | B | External `go` toolchain. |
| `m1_3_emit_rust_test` | Contract on emitted Rust | B | `rustc` roundtrip. |
| `m1_4_emit_python_test` | Contract on emitted Python | B | CPython / tooling. |
| `m1_5_emit_omni_demo_test` | Emit demo / matrix | B | Same class as emit tests. |
| `m2_emit_multi_field_struct_variant_test` | Class-5 emit | B | `rustc` / structural emit checks. |

### Standalone test binary

| Module | Pipeline vs contract | Tags | Forever Rust |
|--------|----------------------|------|----------------|
| `determinism_test.rs` | Emit determinism | A + B-flavored | Release emit matrix; host-side 5× replay ratchet stays in Rust until emit + runner are fully declarative. |

### Inline `integration.rs` submodules

| Module | Pipeline vs contract | Tags | Notes |
|--------|----------------------|------|-------|
| `lane2_stage_2f_dimension_test` | Pipeline + bootstrap scan | D + G + A-ish | Dimension scan uses `Dag::new()` introspection; cost alignment uses `compile_to_dag`. |

### `tests/integration/*.rs` (alphabetical by stem)

| Module | Pipeline vs contract | Tags | Forever Rust / gap summary |
|--------|----------------------|------|------------------------------|
| `cementing_lens_registry_dispatch_test` | Meta + parity dispatch | A (+ D/B for future claims) | Markdown + `integration.rs` path parsing; v2 oracle slices → **B** when present. |
| `four_fixture_regression_test` | Pipeline smoke | D + G | Multi-fixture regressions; predicate gap when walking internal graphs. |
| `l1_5_fixed_point_test` | Pipeline fixed-point | D + G | Structural fixed-point claims often **G** until surfaced as declarations. |
| `lane2_stage_2a_effects_smoke` | Pipeline / effects | D + G | |
| `lane2_stage_2b_db18_test` | Pipeline / DB-18 | D + G | |
| `lane2_stage_2c_db15_test` | Contract on `TestClaim.requires` | D + A | Obligation materialization: much **D**; harness assertions **A**. |
| `lane2_stage_2d_symbolic_cost_test` | Contract / lens | D + G | Rich cost carriers → **G** until `CostBounded` or facts suffice. |
| `lane2_stage_2e_parallelism_test` | Contract / lens | D + G | |
| `lane3_stage_3b_db1_test` | Pipeline / DB-1 | D + G | |
| `lens_register_correspondence_test` | Meta + registry | A + D | `regen.dag` correspondence: **A** for file wiring. |
| `m0_acceptance` | Mixed milestone | D + G | Large **G** substrate walks. |
| `m1_3_lens_cost_test` | Contract | D + G | |
| `m1_3_lens_unused_parameters_test` | Contract | D + G | |
| `m1_5_testgen_test` | Meta (testgen) | A + D | Exhaustive / expensive harness → **A**; spot claims → **D**. |
| `m1_5_verification_test` | Contract (`std.verification`) | D | Shape witnesses; bootstrap `Dag::new()` layout checks lean **A** until reflection-only. |
| `m1_fn_external_body_reconciliation_test` | Contract | D + G | |
| `m1_lens_structural_resolution_test` | Contract | D + G | |
| `m1_substrate_test` | Pipeline + substrate | D + G | Major **G** volume (imperative walks). |
| `m2_feature_parity_test` | Contract | D + G | |
| `m2_field_access_binding_test` | Contract | D + G | |
| `m2_lens_cost_migration_test` | Contract + rustc | D + G + B | Emit-linked: **B** for subprocess. |
| `m2_lens_idempotency_emit_test` | Contract + emit | D + G + B | |
| `m2_lens_idempotency_migration_test` | Contract | D + G | |
| `m2_lens_provenance_migration_test` | Contract + rustc | D + G + B | |
| `m2_lens_structural_resolution_migration_test` | Contract | D + G | |
| `m2_lens_unused_parameters_migration_test` | Contract + rustc | D + G + B | |
| `m2_lens_variant_payload_migration_test` | Contract | D + G | |
| `m2_substrate_inhabitance_test` | Contract / substrate | D + G | |
| `p0_std_render_repeat_string_test` | Pipeline / std | D + G | |
| `pb1_bootstrap_full_snapshot_test` | Pipeline / bootstrap | A + D | Snapshot / `include_str!` glue → **A**; digest-style claims → **D**. |
| `pb1_bootstrap_std_snapshot_test` | Pipeline / bootstrap | A + D | Same. |
| `pipe_desugar` | Pipeline lowering | G | Structural `Transform` walks — **G** until new `TestPredicate` or behavioral observation without tautology. |
| `sg0_census_test` | Meta ratchet | A | SG-0 hand-authored lists; T-PB-A can mechanical-split; **not** `.dag` until declarative census. |
| `sg1_tokenize_authority_test` | Pipeline | D + G | Token streams vs **D** `Compiles` on `.dag` sources. |
| `sg2_parse_authority_test` | Pipeline | D + G | |
| `sg2c1_parse_tables_authority_test` | Pipeline | D + G | |
| `sg2c5_soft_keyword_ident_test` | Pipeline | D + G | |
| `sg3_lower_authority_test` | Pipeline | D + G | |
| `sg3_lower_parse_surface_stack_test` | Pipeline | D + G | |
| `sg3_surface_reflection_consumer_test` | Pipeline | D + G | |
| `sg6_hand_authored_census_test` | Meta | A | |
| `sg7_prep_variant_payload_freshness_test` | Pipeline / prep | D + G | |
| `t_pb_b_brief_d_fixture_smoke_test` | Fixture host | A | Intentionally thin: only verifies Brief D `.v3` fixtures compile. |
| `t_pb_b_1_tests_dag_smoke_test` | T-PB-B-1 `tests/dag/` host | A | Compile smoke for landed `.dag` `TestClaim` modules (`src/v3/compiler/tests/dag/`). |
| `thesis_parallelism_test` | Contract (thesis) | D + G | |
| `thesis_validation_test` | Contract (thesis) | D + G | |

### `tests/integration/common/*.rs`

| Module | Tags | Notes |
|--------|------|-------|
| `mod.rs`, `cached_compile.rs`, `budgeted.rs`, `substrate_receipts.rs`, `determinism_fixtures.rs` | A | Shared harness — stays Rust; not user program claims. |

### `tests/unit/` (if present under census)

Unit modules under `tests/unit/` follow **residual A** when they test harness-only shapes; otherwise **D**/**G** same as integration, with bias to **D** once minimal `Dag` builders exist (`TESTING.md`).

---

## First-wave port ordering (when Testgen opens)

1. Pure **D** rows: `Compiles` / `FailsWithDiagnostic` / `PortHasState` / `CostBounded` / `OutputEquals` aligned with existing Rust claims (`thesis_*` subsets, `m2_feature_parity` simple cases, `m1_5_verification`-style witnesses).  
2. **D+G:** migrate after predicate extensions or non-tautological `BehavioralObservation` paths land.  
3. **B:** keep subprocess in Rust; optional `TestClaim.requires` `ResourceReference` hooks when DB-15 materialization wires mocks/toolchains.  
4. **A:** last — or never as user `.dag` claims; may become generated manifest + one shim.

---

## Related

- `docs/briefs/r1-selfhosting-manager.md` — T-PB-B manager checklist.  
- `TESTING.md` — Post-R2 residuals; one-claim-per-test discipline.  
- `src/v3/std/verification.dag` — `TestClaim` / `TestPredicate` schema.  
- Fixtures: `src/v3/compiler/tests/fixtures/t_pb_b_brief_d/`.
