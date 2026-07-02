# `dag/ctrl/` — ctrl/ subsystem `.dag` contracts (Phase 1.5)

This directory hosts the **subsystem-modeling artifacts** for the `ctrl/` → `.dag` migration program. Each `.dag` file here is a type-only service contract for one `ctrl/` subsystem; the corresponding TS implementation in [the `ctrl/` repo](https://github.com/gunb-ai/ctrl) remains authoritative until cut-over (Phase 4).

**Authority**: every file in this directory is `🟡 STAGED`, not `🟢 AUTHORITY`, per single-authority / boundary discipline (DESIGN.md §3). The TS implementation in [the `ctrl/` repo](https://github.com/gunb-ai/ctrl) is the sole `🟢 AUTHORITY` while a subsystem is staged here. **Two distinct gates** govern the lifecycle — kept separate per single-trigger discipline (DESIGN.md §3):

1. **Parity-proof gate (readiness)**: trio convergence — `algebra_landed` ✓ + `phase15_pr_merged` ✓ + `phase3_emission_landed` ✓ + `parity_passed` ✓, using the ledger's column semantics (`algebra_landed` is satisfied by `true` OR `—` for non-consumer rows). Crossing this gate *proves* the .dag substrate can stand in for the TS implementation; it does NOT itself flip authority. Workers and Mgrs cite this gate when judging whether a subsystem is ready for Phase 4 dispatch.
2. **Source-authority deletion gate (`STAGED → AUTHORITY` flip)**: the **ctrl PR cut-over** that deletes the corresponding TS file(s). This is the *only* event that flips a subsystem from `🟡 STAGED` to `🟢 AUTHORITY`. The PR's existence + merge IS the deletion receipt.

The parity-proof gate is a **precondition** for cut-over dispatch but is not sufficient by itself; the cut-over PR must still land. This collapses-then-splits ordering avoids the competing-authority failure mode (no overlap window where two artifacts claim authority simultaneously).

The migration project plan and the manager brief that define these gates live in the ctrl repo (`gunb-ai/ctrl`).

## Conventions

- **Path**: `dag/ctrl/<subsystem>.dag` — one file per catalog row (project-plan §3). Sub-subsystem splits live as separate `.dag` files (e.g. `pools_billing.dag` + `pools_dispatch.dag`) only when a single file would exceed cost-of-change-1.
- **Module name**: `module ctrl.<subsystem>` matching the file basename.
- **Shape**: type-only modeling — carriers, closed/open enums, projections, `service ... { operation ... { input / output } readonly }` signatures (no transport in Phase 1.5; those land in `dag/extdeps/*.dag` under the Emission-Targets Mgr lane).
- **Practice-4 receipts**: every enum/sum with ≥2 variants carries an inline classification (`🟢 TERMINAL` / `🟡 STAGED` / `🔴 NEEDS-DISSOLUTION`) and, if non-terminal, a named dissolution trigger. See manager brief §"Per-worker brief template" item 4 for the required receipt format.
- **Consumer-receipt header**: each `.dag` file's module header must cite (a) the ctrl/ TS file(s) currently authoritative and (b) the future emission target whose landing fires `STAGED → AUTHORITY`. The emission target's **placement** distinguishes:
  - `dag/extdeps/<platform>/*.dag` — third-party source-of-record facts (GitHub-API shapes, SQL dialects, HTTP wire formats); owned by Emission-Targets Mgr; **no gunbc emission/policy facts allowed here** per faithful-modeling discipline (DESIGN.md §3).
  - `dag/gunbc/*.dag` — gunbc-owned rendering / projection / policy surfaces (e.g. `digest_render.dag`, `workflow/*.dag`); composes `dag/extdeps/*` carriers + `dag/std/render.dag` primitives.
  - `dag/std/*.dag` — domain-agnostic primitives (render IR, algebra, integers, etc.).
  Worker briefs must name the correct placement before dispatch; see `feedback_extdeps_header_discriminator_before_field_placement.md`.

## Index

- [`review_verdict.dag`](review_verdict.dag) — PR review verdict / tally staging (`ctrl.review_verdict`).
- [`pr_digests.dag`](pr_digests.dag) — PR digest helpers staging (`ctrl.pr_digests`), catalog #8.
- [`code_change_workflow.dag`](code_change_workflow.dag) — code-change node lifecycle FSM staging (`ctrl.code_change_workflow`), catalog M2-PR.
- [`process_algebra.dag`](process_algebra.dag) — process decomposition algebra staging (`ctrl.process_algebra`).

## Discipline carry-overs

Workers landing files here MUST grep BEFORE naming new carriers:
- ctrl's authority docs (per `feedback_self_hosting_md_authority_audit_before_naming.md`)
- existing `dag/std/` for primitives reuse (per DFS-the-concept-DAG, DESIGN.md §2/§6)
- `dag/extdeps/` headers for emission/policy fact placement (per `feedback_extdeps_header_discriminator_before_field_placement.md`)
