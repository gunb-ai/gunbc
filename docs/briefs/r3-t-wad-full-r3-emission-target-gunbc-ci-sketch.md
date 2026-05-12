# T-CI-WAD emission target placement sketch

**Status**: PREP ONLY — consumes the pending
`docs/design-ci-workflow-substrate-shape-2026-05-12.md` canvas. This file does
not ratify a carrier shape.

The retracted option placed `EmissionTarget` on
`dsl/extdeps/github/actions.dag::Workflow`. That violates the file's platform
scope: GitHub Actions facts describe provider constraints, while emission
policy is gunbc CI intent. Any surviving shape keeps the emission target under
`dsl/gunbc/`.

## Option B — Field on `CIPipeline`

```dag
type EmissionTarget = YamlStatic | BinaryShim | PythonShim

type CIPipeline {
  name: String
  gates: List<CIGate>
  emission_target: EmissionTarget?
}
```

This is the smallest CI-intent-level placement. It is appropriate if the
canvas ratifies `CIPipeline` as the authority that selects how gunbc projects
its own CI.

## Option C — Wrapper in `gunbc.ci`

```dag
type EmissionTarget = YamlStatic | BinaryShim | PythonShim

type WorkflowEmission {
  pipeline: CIPipeline
  target: EmissionTarget
}
```

This keeps `CIPipeline` unchanged and makes emission selection an explicit
projection row. It is appropriate if the canvas wants a separate join point
between provider-neutral CI intent and emitted transport.

## Shared Ratchet Shape

Whichever option wins:

- `EmissionTarget` has a single authority in `dsl/gunbc/ci.dag`.
- No `EmissionTarget` field or variant is introduced in `dsl/extdeps/`.
- The focused T-CI-WAD test should assert the selected carrier exists and that
  `YamlStatic`, `BinaryShim`, and `PythonShim` are the only initial variants.
