# R3 gate #87 — cementing placeholder dissolution ledger

**Authority:** complements `lens_cementing_test_discipline_complete` (see `docs/r3-structure.md` / `docs/r3-program-plan.md` §1.8 row 87). **Per-path dissolution prose** (P5 receipts) stays in `INVARIANTS.md` §"SG-0 hand-authored integration test receipts" and the matching `EXPECTED_HAND_AUTHORED_*` lines in `sg0_census_test.rs`. **This doc** is the merge-visible index of **named placeholders** (temporary `Compiles` / minimal projection harnesses plus the Rust bridge files that must retire with them).

**Ratchet:** `r3_gate_87_lens_cementing_regen_receipts_test::r3_gate_87_placeholder_dissolution_ledger_matches_authority` reads the machine blocks below and asserts they match the table in `v3_compiler::r3_gate_87_cementing_regen_runner_suites::R3_GATE_87_CEMENTING_REGEN_SUITES` plus the fixed Rust path inventory in that test. When a placeholder dissolves, update the harness (or retire the Rust file), shrink the SG-0 census in the **same** PR, update `INVARIANTS.md`, and edit the corresponding line in the marker block.

## `.dag` harness rows — temporary placeholder claims

These `tests/dag/t_r3_gate_87_cementing_regen_<lens>.dag` harnesses intentionally avoid full `LensOutputEquals` / carrier-shaped receipts until the cited substrate or fixture work lands. Keys are `regen.dag` `LensRegistryEntry.name` values.

| Lens registry `name` | Placeholder shape | Dissolution (summary) |
|----------------------|-------------------|------------------------|
| `infer_helpers` | `Compiles` only | Public output carrier authorable as `.dag` data → replace with `LensOutputEquals` + delete Rust `*_lens_source_compiles` receipt. |
| `lower_helpers` | `Compiles` only | Same pattern as `infer_helpers`. |
| `variant_payload` | `Compiles` only | Stable variant fixture + `VariantPayloadShapeLookup` literal in `.dag` → behavior claim + delete Rust receipt. |
| `structural_resolution` | `LensOutputEquals` on `Int` projection | Full `List<UnresolvedArrowBody>` carrier stable in `.dag` → full structural receipt. |
| `unused_parameters` | `LensOutputEquals` on `Int` projection | Full `List<UnusedParameter>` carrier stable in `.dag` → full structural receipt. |

### Machine list — DAG placeholder lens names

<!-- G87_CEMENTING_DAG_PLACEHOLDER_KEYS_BEGIN -->
infer_helpers
lower_helpers
structural_resolution
unused_parameters
variant_payload
<!-- G87_CEMENTING_DAG_PLACEHOLDER_KEYS_END -->

## Rust bridge inventory (SG-0 / Band-C seams)

Sorted workspace-relative paths. Each line must match `EXPECTED_HAND_AUTHORED_NON_TEST` or `EXPECTED_HAND_AUTHORED_TEST` in `sg0_census_test.rs` while the receipt remains hand-authored.

### Machine list — Rust paths

<!-- G87_CEMENTING_RUST_RECEIPT_PATHS_BEGIN -->
src/v3/compiler/src/cementing_dispatch.rs
src/v3/compiler/src/integration_rs_wiring_scan.rs
src/v3/compiler/src/r3_gate_87_cementing_regen_runner_suites.rs
src/v3/compiler/tests/integration/cementing/cementing_provenance_origin_integration_test.rs
src/v3/compiler/tests/integration/cementing/complexity_lens_behavioral_completion.rs
src/v3/compiler/tests/integration/cementing/cost_lens_symbolic_consumer_test.rs
src/v3/compiler/tests/integration/common/wiring_scanner_test.rs
src/v3/compiler/tests/integration/r3_gate_87_lens_cementing_regen_receipts_test.rs
<!-- G87_CEMENTING_RUST_RECEIPT_PATHS_END -->
