# v2 Infer Consolidation Calibration - 2026-05-19

## Scope

This audit records the v2 calibration slice for the v4 T-9 single-file infer discipline.

The following former helper modules were consolidated into `src/v2/04_infer.dag`:

- `04_access.dag`
- `04_cycle.dag`
- `04_emit_info.dag`
- `04_env.dag`
- `04_items.dag`
- `04_lookup.dag`
- `04_method.dag`
- `04_patterns.dag`
- `04_resolve.dag`
- `04_service.dag`
- `04_sigs.dag`
- `04_types.dag`

Together with the pre-existing `04_infer.dag`, this collapses the v2 infer-stage reference from 13 source files to one module authority: `v2.compiler.infer`.

## Calibration Result

The consolidation is intentionally behavioral-preserving for v2. It does not claim the v4 infer shape should own name resolution, scope lookup, service wiring, pattern validation, or emit-summary construction. It makes the old pressure visible in one place so v4 can enforce the opposite boundary:

- `03_resolve` owns binding and symbol precision.
- `04_infer` owns bounded homomorphism find, cardinality propagation, and diagnostic precision.
- `05_emit` owns mechanical projection from inferred facts.

The deleted helper modules are therefore a calibration receipt, not a v4 design template.

## Mechanical Changes

- Downstream v2 imports now read all former helper declarations from `v2.compiler.infer`.
- Duplicate local `is_type_variable` / `type_variable_node` helper definitions collapsed to the existing `04_infer.dag` definitions.
- Source-audit checks that previously targeted deleted helper files now target `04_infer.dag`.

## Residual Boundary

The committed stage0 Rust mirror still contains generated Rust modules named after the former helper modules. That mirror remains bootstrap seed output, not `.dag` authority. The `.dag` authority for this calibration slice is now the single consolidated source file.
