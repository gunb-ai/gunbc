# `dsl/ctrl/` — ctrl/ subsystem `.dag` contracts (Phase 1.5)

This directory hosts the **subsystem-modeling artifacts** for the `ctrl/` → `.dag` migration program. Each `.dag` file here is a type-only service contract for one `ctrl/` subsystem; the corresponding TS implementation in [the `ctrl/` repo](https://github.com/gunb-ai/ctrl) remains authoritative until cut-over (Phase 4).

**Authority**: every file in this directory is `🟡 STAGED`, not `🟢 AUTHORITY`, per INVARIANTS P2. The dissolution trigger that fires `STAGED → AUTHORITY` is per-subsystem trio convergence (algebra ✓ + Phase 1.5 modeling PR ✓ + Phase 3 emission target ✓ + parity test ✓), as authored in [`docs/r4-ctrl-dag-migration-project-plan.md`](../../docs/r4-ctrl-dag-migration-project-plan.md) §6 and operationalized in [`docs/briefs/r4-ctrl-migration-subsystem-modeling-manager.md`](../../docs/briefs/r4-ctrl-migration-subsystem-modeling-manager.md).

## Conventions

- **Path**: `dsl/ctrl/<subsystem>.dag` — one file per catalog row (project-plan §3). Sub-subsystem splits live as separate `.dag` files (e.g. `pools_billing.dag` + `pools_dispatch.dag`) only when a single file would exceed cost-of-change-1.
- **Module name**: `module ctrl.<subsystem>` matching the file basename.
- **Shape**: type-only modeling — carriers, closed/open enums, projections, `service ... { fn ... }` signatures. No transport blocks in Phase 1.5 (those land in `dsl/extdeps/*.dag` under the Emission-Targets Mgr lane).
- **Practice-4 receipts**: every enum/sum with ≥2 variants carries an inline classification (`🟢 TERMINAL` / `🟡 STAGED` / `🔴 NEEDS-DISSOLUTION`) and, if non-terminal, a named dissolution trigger. See manager brief §"Per-worker brief template" item 4 for the required receipt format.
- **Consumer-receipt header**: each `.dag` file's module header must cite (a) the ctrl/ TS file(s) currently authoritative and (b) the future emission target whose landing fires `STAGED → AUTHORITY`.

## Index

(populated as worker PRs land; entries here index files in this directory)

_no files yet — Wave-1 dispatch pending #2775 merge per Mgr brief §"Working state"._

## Discipline carry-overs

Workers landing files here MUST grep BEFORE naming new carriers:
- `src/v3/SELF_HOSTING.md` and ctrl's authority docs (per `feedback_self_hosting_md_authority_audit_before_naming.md`)
- existing `dsl/std/` for primitives reuse (per MODELING.md M9 DFS-the-concept-DAG)
- `dsl/extdeps/` headers for emission/policy fact placement (per `feedback_extdeps_header_discriminator_before_field_placement.md`)
