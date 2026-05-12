# Worker brief skeleton — T-CI-WAD Slice 4 YamlStatic projection

**Status**: PREP — supersedes prior option-neutral skeleton. Updated
2026-05-12 to reflect `(c-refined)` ratification per PR #2749 §7.3-§7.5
and Director msg_4f7f536d.

**Program tag**: `t_ci_wad_full_r3_close`

**Scope claim**: Slice 4 implements the YamlStatic projection path over the
ratified gunbc-namespace substrate. The target workflow value is derived as:

```dag
data gunbc_ci_yml_workflow: Workflow =
  project_github_actions(ci_workflow_dag, YamlStatic)
```

No workflow authority is added to `dsl/extdeps/`; `Workflow` remains the GitHub
Actions platform output carrier and `ci_workflow_dag` remains the gunbc CI
semantic authority.

## Authority

- PR #2749 §7.3-§7.5 and Director msg_4f7f536d: `(c-refined)` substrate shape.
- PR #2751 / Director msg_168005e1: GitHub Actions `Expression` substrate.
- Aggregator pattern: `t_ci_wad_full_r3_close` derives from constituent gate
  statuses by lattice meet; this brief does not hand-set aggregator status.

## Inputs

- `dsl/gunbc/ci_emission.dag` once WI-2 lands.
- `CIWorkflowDag` authority for gunbc CI topology.
- `EmissionTarget` standalone gunbc sum type.
- `project_github_actions(CIWorkflowDag, EmissionTarget) -> Workflow`.
- `Expression::OpaqueString` for GitHub Actions expressions that remain opaque
  until a non-YamlStatic consumer needs structural evaluation.

## Slice 4 Deliverables

- YamlStatic emitter consumes
  `project_github_actions(ci_workflow_dag, YamlStatic)`.
- The emitted `.github/workflows/ci.yml` is derived from the projected
  `Workflow` value.
- Checkable ratchet proves the emitted artifact is not hand-authoritative.
- Bootstrap regeneration, if `.dag` authorities change.

## Acceptance

- No `EmissionTarget` field in `dsl/extdeps/github/actions.dag`.
- No independent hand-authored `Workflow` copy for gunbc CI.
- `Expression::OpaqueString` is emitted verbatim to YAML.
- Focused T-CI-WAD tests pass.
- `regen_bootstrap --verify` passes after any intentional authority edit.

## Deferred Until WI-2 Lands

- Exact file list and line-level references.
- Concrete YamlStatic emitter entrypoint.
- BinaryShim/PythonShim runtime handoff details.
