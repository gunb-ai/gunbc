# T-CI-WAD CI projection policy sketch

**Status**: PREP ONLY — consumes the pending
`docs/design-ci-workflow-substrate-shape-2026-05-12.md` canvas. This file does
not ratify a carrier shape.

The retracted option placed `EmissionTarget` on
`dsl/extdeps/github/actions.dag::Workflow`. That violates the file's platform
scope: GitHub Actions facts describe provider constraints, while emission
policy is gunbc CI intent. Any surviving shape keeps the emission target under
`dsl/gunbc/`.

Terminology is deliberately **not** Shape-A compiler `EmissionTarget` here.
`src/v3/SELF_HOSTING.md` distinguishes compiler emission targets from Shape-B
artifact generation: YAML is a Shape-B artifact produced by a `.dag` program,
never a compiler target value. The names below are a CI projection policy
placeholder for workflow artifact generation, not compiler target selection.

**Coproduct classification**: 🟡 SCAFFOLD. The current variants name the
minimum CI projection modes needed for T-CI-WAD planning. The dissolution
trigger is the ratified `(c-refined)` substrate landing: replace this prep
placeholder with the canonical gunbc-namespace sum type and
`project_github_actions(CIWorkflowDag, <projection-policy>) -> Workflow`
signature, preserving the Shape-B/Shape-A distinction.

## Option B — Field on `CIPipeline`

```dag
type CIProjectionMode = YamlStatic | BinaryShim | PythonShim

type CIPipeline {
  name: String
  gates: List<CIGate>
  projection_mode: CIProjectionMode?
}
```

This is the smallest CI-intent-level placement. It is appropriate if the
canvas ratifies `CIPipeline` as the authority that selects how gunbc projects
its own CI.

## Option C — Wrapper in `gunbc.ci`

```dag
type CIProjectionMode = YamlStatic | BinaryShim | PythonShim

type WorkflowEmission {
  pipeline: CIPipeline
  mode: CIProjectionMode
}
```

This keeps `CIPipeline` unchanged and makes emission selection an explicit
projection row. It is appropriate if the canvas wants a separate join point
between provider-neutral CI intent and emitted transport.

## Shared Ratchet Shape

Whichever option wins:

- The CI projection policy has a single authority in `dsl/gunbc/`.
- No CI projection policy field or variant is introduced in `dsl/extdeps/`.
- The focused T-CI-WAD test should assert the selected carrier exists and that
  the initial variants match the ratified `(c-refined)` substrate.
