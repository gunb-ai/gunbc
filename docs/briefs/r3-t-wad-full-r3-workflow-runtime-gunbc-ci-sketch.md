# T-CI-WAD CI projection policy sketch

**Status**: PREP — supersedes prior Option B/C framings. Updated
2026-05-12 to reflect `(c-refined)` ratification per PR #2749 §7.3-§7.5
and Director msg_4f7f536d.

`dsl/extdeps/github/actions.dag` remains a GitHub Actions platform model only.
It must not carry gunbc CI emission policy. The CI projection choice lives in
the `dsl/gunbc/` namespace and is supplied at projection invocation time.

Terminology remains deliberately separate from Shape-A compiler target
selection. `src/v3/SELF_HOSTING.md` says YAML is a Shape-B artifact produced by
`.dag` programs, never a compiler `WorkflowRuntime` value. The ratified
T-CI-WAD `WorkflowRuntime` below is a gunbc CI projection mode for workflow
artifact generation, not a compiler target.

**Coproduct classification**: 🟡 SCAFFOLD. The initial variants are the
ratified projection modes needed for the T-CI-WAD sequence. Future variants are
added only at this single gunbc-namespace authority when a real projection
consumer lands.

## Ratified Shape

```dag
type WorkflowRuntime
  = YamlStatic
  | BinaryShim
  | PythonShim
  | InlineGunbc

fn project_github_actions(workflow: CIWorkflowDag, target: WorkflowRuntime) -> Workflow
```

`WorkflowRuntime` is standalone. It is not a field on `CIPipeline`,
`Workflow`, or a wrapper record. The selected target is a property of the
projection call:

```dag
data gunbc_ci_yml_workflow: Workflow =
  project_github_actions(ci_workflow_dag, YamlStatic)
```

The binding name may persist for tests and downstream emitters, but the value
is derived from `ci_workflow_dag`; it is not an independent hand-authored
workflow declaration.

## Rejected Shapes

- `CIPipeline { workflow_runtime }`: rejected because `CIPipeline` is
gate-centric, not emission-artifact-centric; coupling projection policy to the
gate list violates modeling faithfulness.
- `WorkflowEmission { workflow, target }`: rejected because it introduces an
implicit join and duplicate authority between workflow value and target choice.
- `extdeps.github.actions.Workflow { workflow_runtime }`: rejected because it
puts gunbc CI policy into provider platform facts.

## Shared Ratchet Shape

The focused T-CI-WAD ratchet should assert:

- `WorkflowRuntime` and `project_github_actions` live under `dsl/gunbc/`.
- `dsl/extdeps/github/actions.dag` has no CI projection policy field.
- `gunbc_ci_yml_workflow` is a derived projection binding of
  `project_github_actions(ci_workflow_dag, YamlStatic)`.
- The initial variants are `YamlStatic`, `BinaryShim`, `PythonShim`, and
  `InlineGunbc`.
