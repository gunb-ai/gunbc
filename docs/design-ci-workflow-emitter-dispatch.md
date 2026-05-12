# CI Workflow Emitter Dispatch - Design Canvas

**Status:** Design canvas for T-Workflow-As-Data FULL R3-close; consumes the
placement decision from substrate-shape comparison PR #2749.
**Authority:** Worker output for PR #2744 WI-1; substrate-shape comparison
canvas PR #2749; final lane absorption proceeds through the T-WAD owner.
**Scope:** Workflow emission target dispatch only. This document does not
implement carriers or emitters.

---

## 0. Purpose

T-Workflow-As-Data was originally scoped to prove that at least one CI
workflow can be represented as `.dag` data. The FULL R3-close elevation makes
the requirement stronger: the repository CI workflow must be authored as
`.dag`, and the checked-in GitHub Actions workflow must become an emitted
artifact or a thin shim.

The load-bearing design choice is:

```
CI workflow data is projected through a modeled emission target.
The build system does not choose a hidden emitter path.
```

The modeled value that selects the projection is part of gunbc CI/emission
substrate. The emitter reads that value and mechanically projects the same
workflow data into one of several target artifacts:

- `YamlStatic`: a complete `.github/workflows/ci.yml`
- `BinaryShim`: a thin YAML shim that invokes a compiled gunbc CI binary
- `PythonShim`: a thin YAML shim that invokes an emitted Python CI runner
- `InlineGunbc`: a longer-term direct gunbc runtime orchestrator

This extends the single-emitter thesis. It does not introduce a separate
"workflow generator engine." The emitter still reads declared data and renders
target syntax. The target is a workflow runtime shape, not a compiler policy.

## 1. Non-Goals

This canvas does not:

- implement `EmissionTarget`
- rewrite `.github/workflows/ci.yml`
- introduce a new `MatrixSpec`, `WorkflowRuntime`, or sibling workflow carrier
- extend `dsl/gunbc/ci.dag` to full workflow coverage
- implement affected-set lens evaluation
- replace current Rust/Python/Go program emitters

The sibling WI-2 work item owns extending the existing `dsl/gunbc/ci.dag`
surface to full `.github/workflows/ci.yml` coverage. Slice 4 and Slice 5 own
emitter implementation after this shape is ratified.

## 2. Existing Authorities

This canvas composes existing documents and carriers:

| Authority | Role |
|---|---|
| `dsl/extdeps/github/actions.dag` | Existing GitHub Actions platform model: `Workflow`, `Job`, `Step`, `MatrixStrategy`, `RunnerSpec`, triggers, permissions. |
| `docs/design-emission-model.md` | Emission is structural projection, not a decision engine. |
| `docs/single-emitter-design.md` | End-state emitter reads target specs and emits mechanically. |
| `docs/design-affected-set-lens.md` | Affected-set output is a structural lens result over `Dag_before` and `Dag_after`; CI consumes it as selection input. |
| `dsl/gunbc/ci.dag` | Existing gate-centric CI intent declarations: `CIPipeline`, `CIGate`, `GateSource`. |
| `.github/workflows/ci.yml` | Current hand-authored target artifact and Slice 8 deletion target. |
| `docs/design-clean-emission-contract.md` | Target artifacts must satisfy declared clean-emission contracts by construction. |

## 3. Carrier Shape

Placement authority lives in PR #2749, the canonical substrate-shape comparison
canvas. This document consumes that ratified placement and authors the
per-target emission semantics and acceptance contracts on top of it.

PR #2749 evaluated the carrier-placement question:

| Option | Shape | Result |
|---|---|---|
| A | Add `emission_target` to `extdeps.github.actions.Workflow` | Rejected after comparison-canvas review: violates extdeps fidelity by adding gunbc projection policy to a GitHub Actions platform carrier. |
| B | Add `emission_target` to `gunbc.ci.CIPipeline` | Rejected for this gate: `CIPipeline` is gate-centric CI intent, while the target controls projection of the GitHub Actions workflow artifact. |
| C | Add `WorkflowEmission { workflow, target }` wrapper | Rejected in wrapper form: preserves separation superficially but introduces an implicit join and duplicate authority. |
| C-refined | Put `EmissionTarget` in gunbc namespace and pass it to `project_github_actions(ci_workflow_dag, target) -> Workflow` | **Consumed here.** The target choice is invocation-time modeled data, extdeps stays provider-faithful, and the emitted `Workflow` is derived from one source. |

The recommended substrate shape is a gunbc-owned enum plus projection function:

```dag
type EmissionTarget
  = YamlStatic
  | BinaryShim
  | PythonShim
  | InlineGunbc

fn project_github_actions(source: CIWorkflowDag, target: EmissionTarget?) -> Workflow
```

`EmissionTarget` is a normal `.dag` sum type. It is "open" only in the same
operational sense as other R3 extension surfaces: new variants are added to
this single authority when a real consumer lands. A new variant must not create
a sibling workflow carrier.

The projection argument is optional to preserve migration compatibility:

```
normalize_target(target) =
  YamlStatic if target is none
  the contained target otherwise
```

This lets existing projection call sites remain valid before Slice 4 lands the
enum and emitter consumers. The final authored projection call should still
carry the target explicitly once the emitter consumes it.

Optionality is migration-only. The retirement trigger is Slice 8
`ci_yml_dissolved`: once hand-authored `.github/workflows/ci.yml` is gone and
the workflow artifact is emitted from `.dag`, every authoritative projection
call must carry an explicit `EmissionTarget`. At that point
`normalize_target(none) = YamlStatic` becomes a compatibility reader for older
fixtures only, and a ratchet should reject new authoritative projection calls
that omit the target.

### 3.1 Placement Evaluation Summary

#### Option A: Field on `Workflow`

This option is rejected. It would put the target choice on
`extdeps.github.actions.Workflow`.

The objection is that `dsl/extdeps/github/actions.dag` describes platform
constraints, not gunbc CI policy. That objection is load-bearing:
`EmissionTarget` is not a GitHub Actions provider fact and must not become a
field rendered into, or normalized as part of, the provider `Workflow` type.

The workflow carrier already describes a provider artifact:

```
Workflow -> GitHub Actions workflow file or shim entrypoint
Job      -> GitHub Actions job
Step     -> GitHub Actions step
```

The platform carrier should remain provider-faithful. The target choice belongs
to the gunbc projection call that produces a `Workflow`, not to the extdeps
`Workflow` value itself.

#### Option B: Field on `CIPipeline`

`dsl/gunbc/ci.dag` already carries CI-intent declarations:

```dag
type CIPipeline {
  name: String
  gates: List<CIGate>
}
```

This is attractive because `emission_target` can be read as CI-level policy,
and `actions.dag` explicitly names `gunbc/ci.dag` as a consumer. It also keeps
provider platform facts free of gunbc-specific emission choices.

The problem is that `CIPipeline` is gate-centric, not workflow-artifact-centric.
It names `CIGate`s and their structural gate sources. It does not own:

- GitHub triggers
- workflow permissions
- top-level workflow environment
- job runner selection
- job dependency graph
- step rendering
- provider-specific syntax obligations

Putting `emission_target` on `CIPipeline` would require the emitter to join a
pipeline declaration to a separate `Workflow` declaration. That join becomes a
new coherence surface: which pipeline emits which workflow, which value wins if
multiple pipelines reference the same workflow, and how a target-specific shim
observes workflow-level fields. Those are not CI gates; they are artifact
projection facts.

If later work reshapes `CIPipeline` so that it owns the full workflow artifact,
this question can be reopened. At HEAD, `CIPipeline` is the wrong placement for
Slice 4 and Slice 5.

#### Option C: Wrapper Node

A wrapper shape would be:

```dag
type WorkflowEmission {
  workflow: Workflow
  target: EmissionTarget
}
```

This keeps `actions.dag` pure and avoids changing `Workflow`, but it makes the
emission target a separate fact that must be joined to a workflow. That is
parallel-representation debt:

- one workflow can be paired with multiple wrappers without declaring whether
  that is intentional A/B emission or drift
- wrapper identity can diverge from workflow identity
- freshness checks must decide whether edits to `Workflow` or
  `WorkflowEmission` are the source of truth
- target-specific consumers can accidentally read wrapper fields while static
  emitters read workflow fields

The operator requirement is "modeled data," not "external wrapper." A field on
the workflow would have satisfied that requirement without a join, but extdeps
fidelity rules it out. The refined answer is a gunbc-owned projection function
parameter: the target is modeled data on the call, not an external wrapper
around an already-authored provider workflow.

#### Option C-refined: Projection Parameter

The refined recommendation is:

```dag
data ci_workflow_dag: CIWorkflowDag = ...
data gunbc_ci_yml_workflow: Workflow =
  project_github_actions(ci_workflow_dag, YamlStatic)
```

`CIWorkflowDag` is the single semantic source for the gunbc CI workflow.
`project_github_actions` is the structural fold invoked with that source and
the emission target argument, then produces the provider-faithful GitHub
Actions `Workflow` value. There is no independent hand-authored `Workflow`
authority for the emitter to join against.

This keeps all three layers distinct:

| Layer | Authority |
|---|---|
| CI semantics | `gunbc.ci` workflow/gate graph |
| Emission choice | invocation-time `EmissionTarget` argument |
| Provider artifact | derived `extdeps.github.actions.Workflow` |

The cost of adding a target remains bounded: add one `EmissionTarget` variant
and one projection consumer. The extdeps carrier remains unchanged.

### 3.2 Carrier Reuse Audit

The existing GitHub Actions carrier hierarchy is sufficient for this canvas:

```
Workflow {
  name: String
  on: List<WorkflowTrigger>
  jobs: List<Job>
  env: Map<String, String>
  permissions: WorkflowPermissions?
}

Job {
  id: String
  runner: RunnerSpec
  steps: List<Step>
  needs: List<String>
  strategy: MatrixStrategy?
  ...
}

Step = RunStep | UsesStep
```

`Workflow` is concrete, not generic. It contains jobs, jobs contain steps, and
`Job.needs` is a list of job-id references. The dispatch canvas does not need a
new step carrier, trigger carrier, or matrix carrier.

### 3.3 No New Matrix Carrier

The existing `MatrixStrategy` carrier is enough for static matrix structure.
Dynamic matrix selection is not a new substrate fact. It is the runtime value
of the same workflow selection problem:

- `YamlStatic` renders the declared matrix directly into YAML.
- `BinaryShim` computes the selected subset at runtime, then writes the matrix
  payload expected by the shim protocol.
- `PythonShim` does the same through Python.

If future work proves that GitHub's dynamic matrix protocol needs a typed
carrier, that is a separate substrate question. This canvas does not require
it for Slice 4 or Slice 5.

## 4. Dispatch Semantics

The workflow emitter has one structural responsibility:

```
(CI workflow graph, EmissionTarget, GitHub Actions platform facts,
 target language/runtime facts)
  -> emitted workflow artifact(s)
```

It must not infer policy from repository paths, CI environment variables, or
hard-coded branch names except where those facts are already declared in the
workflow data.

The dispatch steps are:

1. Normalize the target.
2. Validate the workflow against GitHub Actions platform facts.
3. Validate target-specific prerequisites.
4. Render deterministic artifacts.
5. Emit a typed diagnostic if any required fact is absent.

Pseudocode:

```dag
fn emit_workflow(source: CIWorkflowDag, projection_target: EmissionTarget?) -> WorkflowEmissionResult =
  let target = normalize_target(projection_target)
  let workflow = project_github_actions(source, target)
  match target {
    YamlStatic =>
      emit_yaml_static(workflow)
    BinaryShim =>
      emit_binary_shim(workflow, source)
    PythonShim =>
      emit_python_shim(workflow, source)
    InlineGunbc =>
      emit_inline_gunbc(source)
  }
```

The `match` is not an engine decision. It is the mechanical projection of an
authored projection argument. Each arm must consume the same CI workflow source
and the derived provider `Workflow`, not a target-private copy of workflow data.

## 5. Target Semantics

### 5.1 `YamlStatic`

`YamlStatic` emits a complete GitHub Actions workflow file.

Output:

```
.github/workflows/ci.yml
```

Semantics:

- Every `WorkflowTrigger` renders into `on:`.
- `WorkflowPermissions` renders into `permissions:`.
- Workflow `env` renders into top-level `env:`.
- Each `Job` renders into one `jobs.<id>` entry.
- Each `Step` renders into either `run:` or `uses:`.
- `MatrixStrategy` renders into `strategy.matrix`.
- `needs`, `if_condition`, `timeout_minutes`, `continue_on_error`, and
  `concurrency` render directly from the existing carrier fields.

Acceptance contract:

- Emission is deterministic: same `Workflow` graph yields byte-identical YAML.
- The emitted file validates against GitHub Actions syntax.
- The emitted file is functionally equivalent to the current hand-authored
  `.github/workflows/ci.yml` for the modeled subset.
- No emitter arm reads repository-local path-regex policy outside `Workflow`
  data. The current Layer 1/Layer 2 path filters are migration scaffolding and
  must dissolve as affected-set selection lands.

`YamlStatic` is the default because it preserves today's GitHub execution
model and is the safest first emitter implementation.

### 5.2 `BinaryShim`

`BinaryShim` emits two artifacts:

```
.github/workflows/ci.yml        # thin shim
target ci binary artifact       # emitted/compiled runner
```

The YAML shim should be intentionally small. Its job is to bootstrap the
runtime, not to encode CI policy. A representative shape:

```yaml
name: ci
on: [push, pull_request]
jobs:
  ci:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: ./gunbc-ci --workflow ci --event "$GITHUB_EVENT_PATH"
```

The exact bootstrap can differ once implementation lands, but the invariant is
that the shim delegates CI decisions to the emitted binary. A large YAML file
with reimplemented path conditions is not `BinaryShim`; it is a static workflow
with a binary step attached.

Runtime semantics:

1. Read the GitHub event payload and checked-out repository state.
2. Compute `Dag_before` and `Dag_after` when the event is a PR.
3. Run `affected_set(Dag_before, Dag_after)` per dimension.
4. Intersect the aggregate affected set with test/workflow declarations.
5. Execute or dispatch only the selected jobs/tests.
6. Fail closed when affectedness cannot be proven narrow.

Acceptance contract:

- Same CI workflow source and projection target surface are the input to both
  `YamlStatic` and `BinaryShim`; the provider `Workflow` is derived.
- The binary reads affected-set output as structural data, not as file-path
  regexes.
- When affected-set computation is unavailable, unknown, or diagnostic-bearing,
  the binary runs the conservative superset.
- The shim YAML contains no durable CI policy beyond runner bootstrap and
  checkout/setup required to invoke the binary.
- The emitted binary exposes a stable local reproduction command.

`BinaryShim` is the FULL R3 unlock for affected-set CI because GitHub's static
YAML expression language is not the right place to run dimension-aware graph
selection.

### 5.3 `PythonShim`

`PythonShim` is the same target shape as `BinaryShim`, but the runtime
orchestrator is emitted Python.

Output:

```
.github/workflows/ci.yml        # thin shim
generated ci runner .py files
```

Semantics:

- The YAML shim installs or uses Python.
- The generated Python runner reads the same `Workflow` data projection.
- Affected-set selection follows the same fail-closed rules as `BinaryShim`.

Acceptance contract:

- Python runner behavior is semantically equivalent to the binary runner for
  the same workflow and affected-set inputs.
- Python-specific syntax and packaging facts live in the Python target spec,
  not in workflow carriers.
- Missing Python runtime facts produce diagnostics, not silent fallback to
  static YAML.

`PythonShim` is useful as a second shim target because it proves the target
toggle is not secretly "Rust binary vs YAML" hard-coding.

### 5.4 `InlineGunbc`

`InlineGunbc` is a longer-term target where gunbc itself is the CI runtime
orchestrator. GitHub Actions becomes a minimal host that invokes gunbc's
workflow interpreter directly.

This target is intentionally sketched only. It should not block Slice 4 or
Slice 5. Its value is architectural: the `EmissionTarget` surface can express
"workflow stays inside gunbc runtime" without inventing a second workflow
model.

Acceptance contract before implementation:

- No `InlineGunbc` emitter arm lands until a real runtime consumer exists.
- The variant must not become a placeholder that accepts and discards workflow
  semantics.

## 6. Affected-Set Integration

`docs/design-affected-set-lens.md` Section 5 defers production CI
integration. FULL R3-close makes that integration part of T-WAD.

The integration point is `BinaryShim` or `PythonShim`, not `YamlStatic`.

Static YAML can express coarse `if:` conditions, but it cannot honestly run
the affected-set lens:

```
affected_set(Dag_before, Dag_after) =
  union over dimensions:
    affected_set(Dag_before, Dag_after, dim)
```

That computation requires compiled graph state, dimension lenses, fail-closed
receipts, and intersection with test declarations. Encoding it as GitHub
Actions expressions would recreate a parallel build system in YAML.

### 6.1 Runtime Decision Surface

The shim runtime consumes:

| Input | Source |
|---|---|
| GitHub event payload | `$GITHUB_EVENT_PATH` or equivalent |
| Before ref | PR base SHA or main branch ref |
| After ref | PR head SHA or push SHA |
| Workflow declaration | emitted/embedded `ci.dag` projection |
| Affected-set lens | compiled gunbc runtime / generated runner |
| Test/workflow declarations | `.dag` TestClaim and workflow data |

The runtime produces one of these target-native decision surfaces:

| Surface | Use |
|---|---|
| selected job list | Run named job functions in-process. |
| selected test list | Pass test names to cargo/test runner. |
| dynamic matrix JSON | Feed a GitHub matrix job through step output. |
| skip receipt | Emit a GitHub notice explaining why a job/test was skipped. |

The first implementation should prefer selected test/job lists because they
keep the decision inside the runner. Dynamic matrix JSON is only necessary
when GitHub-level fanout is required.

### 6.2 Fail-Closed Rule

The runtime must include a candidate when any of the following is true:

- the lens reports an unknown dimension delta
- the lens lacks a proof receipt for an exclusion
- `Dag_before` or `Dag_after` cannot be constructed
- a TestClaim does not declare the dimensions it asserts
- the CI workflow source references a job not mapped to a test/workflow node

This is the same discipline as the affected-set design: no silent exclusions.
CI may run too much; it must not skip a structurally affected check.

### 6.3 Dissolving Current Path Bridges

The current CI contains coarse path-based mitigation comments and Layer 2
path-regex bridge work. Those are migration scaffolds. The dissolution path is:

1. Model current jobs and steps as workflow data.
2. Emit `YamlStatic` equivalent to current CI.
3. Land `BinaryShim` with conservative all-run behavior.
4. Wire affected-set selection into the binary.
5. Remove path-regex `if:` policy from emitted YAML.
6. Ratchet that CI selection reads affected-set output, not path globs.

## 7. Equivalence Claim

The core proof obligation is not byte equality. Different targets have
different operational shapes. The claim is workflow-semantics equivalence.

For a workflow `W`, targets `A` and `B` are equivalent when, for the same event
and repository state:

1. They observe the same triggers.
2. They enforce the same permissions envelope.
3. They evaluate the same job dependency graph.
4. They select the same required checks when affected-set is disabled.
5. They select a subset only when affected-set proof receipts justify it.
6. They report failure if any selected required check fails.
7. They fail closed to the conservative superset when runtime selection is
   underdetermined.

This yields two TestClaim shapes:

```dag
data workflow_emission_target_toggle_proven: TestClaim = ...
data workflow_target_semantics_equivalent: TestClaim = ...
```

The first proves both emitters execute from the same CI workflow source and
modeled projection target surface. The second proves semantic equivalence on
representative event fixtures.

Suggested fixtures:

| Fixture | Expected result |
|---|---|
| Draft PR | workflow skipped consistently |
| Ready PR with docs-only change | selected minimal safe checks, or all checks before affected-set lands |
| Source change affecting v3 compiler | v3 compiler checks selected |
| Workflow edit | conservative all-run selected |
| Unknown affected-set receipt | conservative all-run selected |

`YamlStatic` vs `BinaryShim` cannot be required to select the same reduced set
after affected-set lands. `YamlStatic` has no dynamic selection runtime. The
equivalence relation is therefore parameterized:

- with affected-set disabled: same required checks
- with affected-set enabled: `BinaryShim` may run a proven-safe subset;
  `YamlStatic` remains the conservative superset

## 8. Clean Emission Contract

Workflow artifacts must satisfy clean emission by construction.

For `YamlStatic`, the verifier is a GitHub Actions workflow syntax check plus
canonical formatting. For shim targets, the contract covers both files:

- YAML shim is minimal, deterministic, and syntax-valid.
- Runtime artifact compiles or passes its target verifier.
- No dead target arm emits placeholder success.
- No warning suppression is introduced as a substitute for constructive
  emission.

The target-specific verifier commands belong in target specs or workflow
emission spec data. The workflow emitter reads those facts. It does not shell
out to whatever command happens to be convenient.

## 9. Dependency Shape

This canvas keeps Slice 4 and Slice 5 small:

| Slice | Dependency |
|---|---|
| Slice 4: projection + `YamlStatic` | gunbc-owned `EmissionTarget` plus `project_github_actions(..., YamlStatic)` |
| Slice 5: `BinaryShim` | Slice 4 target dispatch plus compiled runner surface |
| Slice 7: affected-set CI | Slice 5 plus affected-set lens |
| Slice 8: `ci.yml` deletion | Slice 4-7 accepted |

No Slice 4 implementation should wait on timing carriers. Timing is a workflow
dimension and can be consumed later by the same workflow data.

No Slice 5 implementation should wait on perfect affected-set narrowing. It can
land with conservative all-run behavior as long as the runtime decision surface
is shaped to consume affected-set receipts when they become available.

## 10. Migration Plan

### Phase A: Ratify Shape

- Land this canvas.
- Ratify `EmissionTarget` placement in gunbc namespace as the projection
  argument to `project_github_actions`.
- Ratify `YamlStatic` default for absent projection target during migration.

### Phase B: Static Parity

- Add `EmissionTarget` to gunbc CI/emission substrate, not to extdeps.
- Model current CI in `dsl/gunbc/ci.dag` as the semantic source.
- Emit `.github/workflows/ci.yml` from
  `project_github_actions(ci_workflow_dag, YamlStatic)`.
- Keep the checked-in YAML as a generated artifact with freshness checking.

### Phase C: Shim Runtime

- Add `BinaryShim` emitter arm.
- Emit a thin YAML shim and compiled CI runner.
- Initially run the conservative full job/test set.
- Prove target toggle on the same workflow input.

### Phase D: Affected-Set Selection

- Build `Dag_before` and `Dag_after` in the shim runtime.
- Run affected-set per dimension.
- Intersect with workflow jobs and TestClaims.
- Emit skip receipts and run selected checks.
- Remove path-regex bridge policy.

### Phase E: Delete Hand CI

- Replace hand-authored `.github/workflows/ci.yml` with emitted static artifact
  or thin shim.
- Add a ratchet that rejects manual workflow policy edits outside `ci.dag`.
- Add a ratchet that rejects missing projection target on authoritative
  projection calls; the optional target remains only for historical fixture
  compatibility.
- Mark `ci_yml_dissolved` passing only when the source of truth is `.dag`.

## 11. Ratchets

Recommended ratchets:

| Ratchet | Purpose |
|---|---|
| `workflow_emission_target_consumed` | `EmissionTarget` is read by the workflow projection/emitter. |
| `workflow_yaml_static_fresh` | emitted YAML matches checked-in artifact while static artifact remains checked in. |
| `workflow_binary_shim_is_thin` | shim YAML contains only bootstrap/checkout/setup/invoke steps. |
| `workflow_no_path_regex_policy` | no durable CI selection policy remains in YAML path regexes after affected-set lands. |
| `workflow_affected_set_fail_closed` | unknown affected-set receipts select conservative superset. |
| `workflow_target_semantics_equivalent` | representative fixtures prove target semantics equivalence. |

The ratchets should live with the T-WAD lane and should cite this canvas as
design authority once ratified.

## 12. Review Questions

The following questions should be answered by ratification, not by worker
implementation:

1. Should `EmissionTarget` be optional during the migration or required
   immediately with every projection call updated in one PR?
2. Should `BinaryShim` first run jobs in-process, or should it emit dynamic
   matrix JSON for GitHub fanout?
3. What is the minimal verifier for GitHub Actions YAML in CI before a full
   GitHub workflow dry-run exists?
4. Does `InlineGunbc` stay in the first carrier as a declared future variant,
   or should it be added only when the runtime consumer lands?

This canvas recommends:

- optional field during migration, required at Slice 8 `ci_yml_dissolved`
- in-process selected job/test execution first
- syntax validation plus freshness check for YAML
- keep `InlineGunbc` in the design, but do not land the variant in substrate
  until a consumer exists

## 13. Acceptance

This design is ready for downstream implementation when:

- `EmissionTarget` lives in gunbc namespace as a projection argument, with
  `YamlStatic` default for absence during migration.
- `YamlStatic`, `BinaryShim`, and `PythonShim` have explicit semantics and
  acceptance contracts.
- Affected-set integration is assigned to shim runtime targets and remains
  fail-closed.
- Equivalence is defined as workflow-semantics equivalence, not byte equality.
- No independent provider `Workflow` declaration duplicates the derived
  projection output.
- No new matrix carrier is required for Slice 4 or Slice 5.

---

**End of design canvas.**
