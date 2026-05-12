# Worker brief skeleton — T-CI-WAD Slice 4 emission-target implementation

**Status**: SKELETON ONLY — substantive scope waits on
`docs/design-ci-workflow-substrate-shape-2026-05-12.md` ratification.

**Program tag**: `t_ci_wad_full_r3_close`

**Scope claim**: implement the canvas-ratified substrate shape for
`workflow_emission_target_open_enum_landed` without introducing a second
workflow authority. The implementation must preserve the concrete
`Workflow > Job > Step` carrier hierarchy if option (ii) wins, preserve the
provider-neutral node/edge graph as single authority if option (i) wins, or
make the projection boundary explicit if option (iii) wins.

## Authority

- Director hold notice: substrate-shape canvas pending under R3 Substrate Mgr.
- Aggregator pattern: `t_ci_wad_full_r3_close` derives from constituent gate
  statuses by lattice meet; this brief does not hand-set aggregator status.
- Shape canvas: `docs/design-ci-workflow-substrate-shape-2026-05-12.md`
  (pending at skeleton authoring time).

## Inputs

- Gate #56 substrate-shape decision: option (i), (ii), or (iii).
- Existing T-WAD demo receipt and tests.
- Existing `dsl/gunbc/ci.dag` CI intent declarations.
- Existing GitHub Actions platform carriers, if retained by the ratified shape.

## Deliverables

- Ratified substrate carrier or projection-layer edits.
- Checkable ratchet proving the chosen authority is present and singular.
- Bootstrap regeneration, if `.dag` authorities change.
- PR body mapping the implementation to the selected canvas option.

## Acceptance

- No parallel workflow authority.
- No unratified carrier hierarchy changes.
- Focused T-CI-WAD tests pass.
- `regen_bootstrap --verify` passes after any intentional authority edit.

## Deferred Until Ratification

- Concrete file list.
- Exact type names and field placement.
- YamlStatic emitter implementation details.
- BinaryShim handoff boundary.
