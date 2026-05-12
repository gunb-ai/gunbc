# CI Workflow Substrate Shape — Mgr Comparison Canvas

**Status:** Substrate Mgr (warm-wolf-698) comparison canvas for gate #56
`ci_workflow_modeled_as_dag` under T-WAD FULL R3-close elevation. Surfaces
substrate-shape options + recommendation for Director ratification.

**Authority:** PM relay `msg_a945b141` (deep-wolf-155) routing Director
substrate-shape question per `feedback_substrate_shape_belongs_in_mgr_canvas`
discipline; Director (zesty-bear-812) `msg_34e9a381` refused to pre-author
(a)/(b)/(c) selection and routed canvas authoring to this lane.

**Scope:** Compare three substrate-shape approaches to gate #56 + identify
WI-1 (PR #2746) assumption breakages + Slice 4/5/8 sequencing implications.
This canvas does **not** itself ratify a shape; Director ratifies the surfaced
recommendation.

---

## §0. Source-claim grep verification

Director (`msg_34e9a381`) flagged the **PR #2736 body claim** against
substrate state and asked this canvas to grep-verify before any comparative
reasoning. Per `feedback_root_cause_evidence_before_pattern_hypothesis` and
`feedback_grep_verify_attribution_in_incoming_messages`, the canvas opens
with the verification rather than treating the PR body as authoritative.

**PR #2736 body claim** (verbatim):

> The previously hand-authored GitHub Actions transport copy was removed so
> the provider-neutral graph remains the single modeled authority until a
> real projection function exists.

**Grep verification** (`gh pr view 2736 --json files` + `gh pr diff 2736`):

PR #2736 files-changed set:

| File | Status | Δ |
|---|---|---|
| `dsl/gunbc/ci.dag` | MODIFIED | +39 / -0 |
| `src/v3/std/t_ci_workflow_as_data_demo.dag` | MODIFIED | +36 / -0 |
| `src/v3/compiler/tests/integration/t_ci_workflow_as_data_demo_test.rs` | MODIFIED | +125 / -2 |
| `src/v3/compiler/src/bootstrap_generated.rs` | MODIFIED | +5554 / -5305 |
| `src/v3/compiler/src/bootstrap_generated_without_parse_surface.rs` | MODIFIED | +5280 / -5031 |

**`dsl/extdeps/github/actions.dag` is NOT in the PR #2736 files-changed set.**
Grep against worktree HEAD (`grep -n "type Workflow\|type Job\|type Step\|^ *steps:\|^ *needs:" dsl/extdeps/github/actions.dag`)
confirms the carriers cited by PM relay are intact:

- `type Workflow` at `dsl/extdeps/github/actions.dag:21`
- `type Job` at `:110` with `steps: List<Step>` at `:114` and `needs: List<String>` at `:115`
- `type Step` at `:147`
- `type WorkflowPermissions` at `:29`
- `type WorkflowTrigger` at `:41`
- `type WorkflowSecret` at `:102`

**Finding**: the PR #2736 body claim is **not supported by the diff**. No
GitHub Actions transport carriers were removed. The PR adds `CIGateNode` /
`CIGateEdge` / `CIWorkflowDag` to `dsl/gunbc/ci.dag` (a provider-neutral
gate-dependency DAG wrapping the existing `CIGate` carrier) and adds a demo
mirror at `src/v3/std/t_ci_workflow_as_data_demo.dag`. The actions.dag
GitHub-specific carriers continue to coexist with PR #2736's additions.

**Per `feedback_verify_dup_vs_fix_forward`** and the broader
"verify-before-acting" family: the substrate at HEAD-after-PR-#2736 is
**dual-authority**, not single-authority. This dual-authority condition is
the substrate fact the comparison below must reason against. The PR body's
single-authority framing is aspirational, not realized.

**PR body correction is a precondition** to PR #2736 merge regardless of
which option this canvas recommends. Either the body is corrected to match
the diff (no actions.dag deletion; `CIWorkflowDag` is **additive** to a still-
extant actions.dag carrier set), or the diff is extended to actually retire
actions.dag carriers — both options resolve the body/diff mismatch, but they
imply different substrate shapes (option (a) vs implicit option (a')).

---

## §1. Three substrate-shape options

The three shapes compared below correspond to the Director-listed (a)/(b)/(c):

### Option (a) — Provider-neutral semantic DAG (PR #2736 as-merged)

**Carrier surface**: `dsl/gunbc/ci.dag` adds `CIGateNode { id, gate: CIGate }`
+ `CIGateEdge { from, to }` + `CIWorkflowDag { name, nodes, edges }`. Nodes
are gates (`CIGate` from existing module); edges are prerequisite relations
between gate ids. No reference to GitHub Actions, runners, triggers,
permissions, jobs, or steps.

**Authority claim**: `CIWorkflowDag` is the modeled authority for gate
dependency shape; provider projection (to ci.yml or any other transport) is
deferred to a future projection function not yet implemented.

**Actions.dag relationship**: orthogonal — `CIWorkflowDag` does not reference
or compose `Workflow`/`Job`/`Step`. The actions.dag carriers remain live at
HEAD (per §0 grep verification) but unused by `CIWorkflowDag`. The PR body
implies a deletion that the diff does not perform.

### Option (b) — Provider-specific concrete carriers (WI-1 canvas, PR #2746)

**Carrier surface**: `dsl/extdeps/github/actions.dag`'s existing `Workflow`
gains an `emission_target: EmissionTarget?` field; `EmissionTarget` is a
new sum type `= YamlStatic | BinaryShim | PythonShim | InlineGunbc`. Same
canvas (PR #2746 `docs/design-ci-workflow-emitter-dispatch.md` §3) rejects
sibling `WorkflowEmission { workflow, target }` wrappers (Option C in that
canvas) and rejects placing the field on `gunbc.ci.CIPipeline` (Option B in
that canvas).

**Authority claim**: the existing `Workflow` carrier (Job/Step nested) is the
modeled authority for the workflow artifact; `emission_target` selects which
projection shape (YamlStatic / BinaryShim / PythonShim / InlineGunbc) the
single-emitter renders from that one `Workflow` value.

**Gate-dependency surface**: not modeled directly — `Job.needs: List<String>`
encodes job-level ordering by name, but gate-as-DAG-node structural framing
is absent. `CIPipeline.gates: List<CIGate>` (existing) gives a flat gate list
without an explicit node/edge prerequisite graph.

### Option (c) — Hybrid: provider-neutral semantic source + actions.dag-composing projection layer

**Carrier surface**: option (a)'s `CIWorkflowDag` retained as semantic
gate-dependency authority **plus** option (b)'s `emission_target: EmissionTarget?`
retained on `extdeps.github.actions.Workflow`. The two carriers occupy
**different concept layers**:

- `gunbc.ci.CIWorkflowDag` — semantic source: which gates exist + how they
  depend (provider-neutral; usable by any CI provider or by `dag run` directly)
- `extdeps.github.actions.Workflow` — provider transport: GitHub Actions
  workflow artifact, with `emission_target` selecting which shape the
  single-emitter renders

**Authority claim**: each carrier is single-authority **for its layer**.
`CIWorkflowDag` is the only modeled authority for gate-dependency facts.
`Workflow` is the only modeled authority for the GitHub Actions transport
artifact. The relation between them is a **projection function**
`project_github_actions: CIWorkflowDag → Workflow` that lives in the emission
layer (per `docs/design-emission-model.md`: "coercion = emission", structural
projection only).

**WI-2 (PR #2745) scope under (c)**: there is **no** separately-declared
`Workflow` value in the modeled authority surface. The only `Workflow` value
that exists is the **output of the projection function** —
`project_github_actions(ci_workflow_dag, target)` — derived structurally
from a single source. Earlier framings of this canvas referring to a
"pinned `Workflow` value the projection is validated against" are retracted
(see §8 retraction); they would have re-introduced a second authority for
the workflow artifact (one derived, one hand-declared) — exactly the dual-
authority condition this canvas was supposed to dissolve.

WI-2's existing draft (`gunbc_ci_yml_workflow` in `dsl/gunbc/ci.dag`) is
re-scoped under (c) / (c-refined) to **either** (i) the projection-function
declaration itself, **or** (ii) the projection-function invocation result
bound to a name (`data gunbc_ci_yml_workflow: Workflow = project_github_actions(ci_workflow_dag, YamlStatic)`).
Both are single-authority shapes: the name is a binding to the derived
result, not an independent declaration. A regression test that compares
the emitted YAML byte-for-byte against a checked-in fixture is a test
artifact (lives in `tests/`, not `dsl/`) and is explicitly **not** part of
the modeled authority surface — it is a receipt of past projection output
for drift detection, not a fact the emitter reads.

---

## §2. Comparison

The substrate-shape question is governed by four discipline axes drawn from
the existing canvases this work composes (per `feedback_substrate_grep_before_authoring`
+ `feedback_canvas_two_axis_verification`):

| Axis | Source | What it tests |
|---|---|---|
| Cost-of-change | `CLAUDE.md`, `INVARIANTS.md` P2 | Adding one CI provider / one emission target — how many files edit? |
| Single-authority | `INVARIANTS.md` P2, `feedback_import_not_redeclare_carriers` | Does each substrate fact live in exactly one carrier? |
| Concept layering (M9) | `MODELING.md` M9, `docs/design-emission-model.md` | Are gate-dependency facts and provider-transport facts in their right concept layer? |
| Sibling-decision cost | `feedback_canvas_finding_taxonomy` Option C-pattern | Does the shape introduce a sibling carrier whose existence forces a join/coherence relation? |

### §2.1 Option (a) evaluation

**Cost-of-change**: low for new CI providers (each provider adds its own
projection function consuming `CIWorkflowDag`). **High for emission target
selection** — option (a) has no field selecting which projection of the
semantic DAG runs; either the build system picks (violates
`docs/design-emission-model.md`: "the build system does not choose a hidden
emitter path") or a parallel selection authority must be added.

**Single-authority**: holds for gate-dependency facts. **Fails** for the
workflow artifact: actions.dag's `Workflow`/`Job`/`Step` carriers remain live
(per §0) and unreconciled — either dead, or implicitly the actual transport
authority while `CIWorkflowDag` claims to be it. The PR body's "removed so
the provider-neutral graph remains the single modeled authority" is
aspirational; the diff leaves dual-authority.

**Concept layering**: clean at the semantic layer. **Absent at the transport
layer** — option (a) does not model the GitHub Actions workflow artifact at
all, leaving `.github/workflows/ci.yml` (the Slice 8 deletion target) without
a modeled counterpart to derive from.

**Sibling-decision cost**: zero (no sibling carriers introduced).

**Conclusion**: option (a) is necessary but **not sufficient** for FULL
R3-close. It provides the semantic source but leaves the workflow artifact +
emission target selection unmodeled. Slice 4/5 (emitter implementations) and
Slice 8 (ci.yml deletion) cannot ground without a workflow-artifact carrier.

### §2.2 Option (b) evaluation

**Cost-of-change**: low for new emission targets (one `EmissionTarget`
variant + one emitter consumer; explicitly stated in PR #2746 canvas §3.1).
**High for new CI providers** — option (b) is GitHub-Actions-specific by
construction; another provider needs its own concrete carriers (GitLab CI's
`.gitlab-ci.yml`, Buildkite, etc.). This is acceptable under
`feedback_strict_mirror_vs_novel_substrate_fact` if other providers are
ratified-out-of-scope, but it forecloses option (a)'s provider-neutral lever.

**Single-authority**: holds for the GitHub Actions workflow artifact.
**Fails** for gate-dependency: `Job.needs: List<String>` is a transport-level
shadow of the prerequisite relation, not a typed gate-dependency surface.
`CIPipeline.gates: List<CIGate>` is a flat list. Reasoning about "is gate X
upstream of gate Y?" requires walking `Job.needs` strings + cross-referencing
job-to-gate name conventions — provider-syntax-coupled, not substrate.

**Concept layering**: option (b) collapses gate-dependency (semantic) and
workflow-artifact (transport) into a single layer (actions.dag). Per
`MODELING.md` M9 (DFS the concept DAG), gate-dependency belongs at the
`gunbc.ci` layer (provider-neutral), not at the `extdeps.github.actions`
layer. WI-1 canvas §3 §B-rejection acknowledges this asymmetry ("CIPipeline
is gate-centric, not workflow-artifact-centric"); the symmetric problem
applies: `Workflow` is workflow-artifact-centric, not gate-dependency-centric.

**Sibling-decision cost**: zero in option (b)'s scope, but the absent
gate-dependency layer means a sibling decision is **deferred**, not
eliminated.

**Conclusion**: option (b) is necessary but **not sufficient** for FULL
R3-close. It models the workflow artifact + emission target but leaves
gate-dependency unmodeled at the semantic layer. Gate #56 demonstration
proves a workflow artifact; FULL R3-close demands the gate-dependency-as-data
property too (else any new CI provider re-shadows the same fact).

### §2.3 Option (c) evaluation

**Cost-of-change**: low across both axes — new provider = new projection
function (semantic source unchanged); new emission target = one `EmissionTarget`
variant + emitter consumer (semantic source unchanged). The two
cost-of-change axes are **decoupled** because the carriers live on different
concept layers.

**Single-authority**: holds for both layers — `CIWorkflowDag` is the only
gate-dependency authority; `Workflow` is the only GitHub Actions transport
authority. The projection function maps one to the other; emission validates
the declared `Workflow` against the projection per
`docs/design-clean-emission-contract.md`. No fact lives in two places.

**Concept layering**: clean — gate-dependency at `gunbc.ci`; provider
transport at `extdeps.github.actions`; emission target selector on the
transport carrier (where it belongs — see PR #2746 canvas §3 rationale).
Per M9, this matches the existing concept DAG: `gunbc.ci.CIPipeline` already
imports `gunbc.compiler` for command construction; `gunbc.ci.CIWorkflowDag`
imports `gunbc.ci.CIGate`; `extdeps.github.actions.Workflow` is a separate
authority for the platform model. The projection function composes them; it
does not collapse them.

**Sibling-decision cost**: zero — `CIWorkflowDag` and `Workflow` are not
siblings (they live on different layers). The projection function is not a
sibling carrier; it is a structural fold per `docs/design-emission-model.md`.

**Conclusion**: option (c) satisfies FULL R3-close substrate requirements
without introducing parallel authorities. The Slice 4/5 emitters consume the
projection result; Slice 8 ci.yml deletion proves the projection function
generates the workflow artifact; gate #56 PASSING requires the projection
function exists and is exercised by a Workflow declaration that mirrors
ci.yml under the YamlStatic emission target.

### §2.4 Summary table

| Axis | (a) | (b) | (c) |
|---|---|---|---|
| Gate-dependency authority | ✓ `CIWorkflowDag` | ✗ implicit in `Job.needs` strings | ✓ `CIWorkflowDag` |
| Workflow-artifact authority | ✗ unmodeled | ✓ `Workflow` | ✓ `Workflow` |
| Emission-target selector | ✗ build-system implicit | ✓ `emission_target` field | ✓ `emission_target` field |
| Single-authority per layer | ✗ dual-authority unresolved | ✓ (transport only) | ✓ both layers |
| Cost-of-change: new provider | low | high (provider-specific) | low |
| Cost-of-change: new target | high (no selector) | low | low |
| Concept layering (M9) | partial | collapsed | clean |
| Sibling-decision cost | zero | zero (deferred) | zero |
| Sufficient for FULL R3-close | no | no | yes |

---

## §3. WI-1 canvas (PR #2746) assumption-breakage under (c)

PR #2746 was authored under the implicit framing that `extdeps.github.actions.Workflow`
is the modeled authority for the CI workflow (option (b) framing). Under
option (c), `Workflow` remains the modeled authority **for the GitHub Actions
transport artifact**, but is no longer the modeled authority for the CI
workflow as a whole — `CIWorkflowDag` owns gate-dependency.

The assumptions that hold:

- **`emission_target` field on `Workflow` is correct placement** — per PR #2746 §3,
  the field selects artifact projection shape, which is a property of the
  transport artifact. (c) does not move this field.
- **`EmissionTarget` sum is correct shape** — `YamlStatic | BinaryShim | PythonShim | InlineGunbc`
  describes projection-target choices on the artifact; orthogonal to the
  semantic gate-dependency DAG.
- **Optional field + `none` → `YamlStatic` migration normalization is correct** —
  preserves backward compatibility while Slice 4 lands.
- **PR #2746 §3 rejection of sibling `WorkflowEmission { workflow, target }`** —
  remains correct under (c) for the same reason (would create implicit join
  between the wrapper and the projection function).
- **PR #2746 §3 rejection of `emission_target` on `CIPipeline`** — remains
  correct: `CIPipeline` is gate-centric, not artifact-centric.

The assumptions that break:

- **"`Workflow` is the load-bearing workflow data carrier"** — PR #2746 §3
  intro frames `Workflow` as carrying "the same workflow data" that the
  emitter projects. Under (c), `Workflow` is a **projection result**, not the
  semantic source. The semantic source is `CIWorkflowDag`; the emitter
  projects `CIWorkflowDag → Workflow` and then renders `Workflow → ci.yml`.
- **"Workflow data chooses its emission target" framing** — PR #2746 §0 reads
  as if a single `Workflow` value chooses its target. Under (c), the
  projection function chooses (or is parameterized by) the emission target;
  the declared `Workflow` is the validated projection result for one
  emission target. Multi-target emission means multiple projection function
  invocations, not multiple `emission_target` fields on one `Workflow`.
- **Slice 4/5 acceptance contract framing** — PR #2746 §4 (per body summary)
  describes emitter implementations consuming `Workflow`. Under (c), the
  implementations consume the **projection function output** for the
  selected target; the declared `Workflow` value is the validation target,
  not the input.

The breakages are **scope-clarifying, not scope-invalidating**. PR #2746
remains correct as a substrate-shape canvas for the **artifact-projection
selector field**; the canvas's framing of `Workflow` as the workflow-data
authority is the part that needs narrowing to "workflow-artifact authority
(transport layer)".

**Recommendation for PR #2746 disposition**: merge with a one-paragraph
clarifying note at §0 (or in PR body) tying `Workflow` to the transport
layer and pointing to this canvas for the gate-dependency authority. The
`EmissionTarget` substrate decision itself stands; only the surrounding
framing narrows.

---

## §4. Slice 4 / 5 / 8 sequencing under (c)

The Director-ratified gate-additions (per PM relay `msg_93e14076`):

- `workflow_emission_target_toggle_proven` (substrate-shape) — Slice 4 + Slice 5
- `ci_yml_dissolved` (state-check) — Slice 8
- `ci_uses_affected_set_selection` (state-check) — Slice 7 (Verification Mgr lane)
- `test_cost_dimension_landed` (substrate-shape + state-check) — Slice 6 (Debt-Paydown Mgr lane)
- Gate #56 stays demonstration-class (existing scope) — possibly PASSING-promotable via separate sweep after (c) ratifies

Under option (c) the lane-absorbed gate-set sequences as:

**S0 — Substrate landing (this canvas + ratification)**
- This canvas ratifies → PR #2746 (WI-1) merges with framing narrowed
- PR #2736 body corrected (or augmented) to match diff; merges as semantic-source
  carrier landing (`CIWorkflowDag` only; no actions.dag deletion implied)
- PR #2745 (WI-2 cool-carp-720) scope **changes** under (c): the existing
  draft adds a `gunbc_ci_yml_workflow` value to `dsl/gunbc/ci.dag` directly.
  Under (c), the only `Workflow` value in modeled authority is the
  projection-function output — i.e., `data gunbc_ci_yml_workflow: Workflow =
  project_github_actions(ci_workflow_dag, YamlStatic)`. The name is a
  binding to the derived result, not an independent declaration. Re-brief
  WI-2 to author the projection function + its bound invocation result
  (or hold WI-2 until projection-function placement decision lands).

**S1 — Projection function (new sub-slice; not yet in Director-ratified gate-set)**
- New brief: `project_github_actions: CIWorkflowDag → Workflow` lives at
  `dsl/gunbc/ci_emission.dag` (or similar) and consumes `CIWorkflowDag` from
  `gunbc.ci` + emits a `Workflow` value parameterized by `EmissionTarget`.
- This sub-slice is the load-bearing substrate fact that ties option (a) +
  option (b) carriers into option (c)'s hybrid. **Without it, (c) reduces to
  dual-authority (a)+(b) coexistence.**
- Whether the projection function is itself a new §1.8 gate or part of
  `workflow_emission_target_toggle_proven` acceptance is a Director call;
  this canvas surfaces the question without selecting.

**S2 — Slice 4 (YamlStatic emitter)**
- Implements emitter consuming `project_github_actions(ci_workflow_dag, YamlStatic)` → ci.yml-equivalent artifact
- Acceptance: emitter output structurally equivalent to current `.github/workflows/ci.yml` (workflow-semantics, not byte equality — per PR #2746 §2)
- Half of `workflow_emission_target_toggle_proven` (one target proven)

**S3 — Slice 5 (BinaryShim emitter)**
- Implements emitter consuming `project_github_actions(ci_workflow_dag, BinaryShim)` → thin YAML shim invoking compiled gunbc CI binary
- Acceptance: BinaryShim output executes the same `CIWorkflowDag` gates that YamlStatic would
- Completes `workflow_emission_target_toggle_proven` (toggle proven across two targets from same `CIWorkflowDag`)

**S4 — Slice 8 (ci.yml dissolution)**
- Delete hand-authored `.github/workflows/ci.yml`; replace with emitted artifact (Slice 4 YamlStatic output) or BinaryShim (Slice 5 output)
- Acceptance: regression guard test asserts `.github/workflows/ci.yml` absent OR equals current-target-emitter output
- `ci_yml_dissolved` gate PASSING

**S5 — Verification Mgr lane (Slice 7, `ci_uses_affected_set_selection`)**
- Routed elsewhere; depends on Slice 5 BinaryShim landing (BinaryShim consumes PR #2713 affected-set lens)

**S6 — Debt-Paydown Mgr lane (Slice 6, `test_cost_dimension_landed`)**
- Routed elsewhere; orthogonal to (c) — depends on Cost dimension landing on test nodes (not gated by emission shape)

**Critical-path sequencing**: S0 → S1 → (S2 ∥ S3) → S4. S5 depends on S3.
S6 is fully orthogonal. Slice 4 ∥ Slice 5 can dispatch in parallel once S1
(projection function) lands.

---

## §5. Recommendation

**Adopt option (c) — hybrid: provider-neutral semantic source (`CIWorkflowDag`
at `dsl/gunbc/ci.dag`) + actions.dag-composing projection layer (`Workflow.emission_target`
at `dsl/extdeps/github/actions.dag`) + projection function (`project_github_actions:
CIWorkflowDag → Workflow`).**

**Reasoning**:

1. Only (c) satisfies single-authority per layer + clean concept layering
   simultaneously (per `MODELING.md` M9, `INVARIANTS.md` P2,
   `feedback_import_not_redeclare_carriers`).
2. Only (c) decouples cost-of-change axes (new provider vs new emission
   target) — a property the FULL R3-close scope explicitly requires (per PM
   scope doc §1 elevation framing).
3. Only (c) preserves both already-authored PRs' substrate contributions:
   PR #2736's `CIWorkflowDag` retained as semantic source; PR #2746's
   `EmissionTarget` retained as transport-artifact selector. The
   reconciliation is a framing narrowing on PR #2746 (not a substrate
   retraction) + a PR body correction on PR #2736 (single-authority claim
   narrowed to the gate-dependency layer).
4. (c) aligns with the single-emitter discipline at `docs/design-emission-model.md`:
   the projection function IS the emitter; there is no separate selection
   engine. Emission target is a substrate fact on the transport carrier;
   the projection function reads it.
5. (c) makes WI-2 scope (PR #2745) coherent: rather than adding
   `gunbc_ci_yml_workflow` as a bare value in `gunbc.ci` (the current
   draft's shape), WI-2 authors the projection function + a single bound
   invocation result (`data gunbc_ci_yml_workflow: Workflow =
   project_github_actions(...)`). The name binds to the derived result; no
   separate hand-declared `Workflow` value exists in modeled authority.
   This is closer to the substrate-shape work the WI-2 brief intended.

**Director ratification ask**:

1. **Ratify option (c) as gate #56 substrate-shape** (under T-CI-WAD program-tag).
2. **Ratify PR #2746 merge readiness** with framing-narrowing addendum (no
   substrate retraction). Mgr-tier APPROVE on PR #2746 already posted at
   `https://github.com/gunb-ai/gunbc/pull/2746#issuecomment-4427973232`; Director
   sign-off remains the gate.
3. **Direct PR #2736 body correction** before merge: replace the "removed
   so the provider-neutral graph remains the single modeled authority"
   sentence with "added as the single modeled authority for gate-dependency
   shape; `dsl/extdeps/github/actions.dag` Workflow/Job/Step retain their
   role as the GitHub Actions transport-artifact authority, consumed via a
   future projection function (S1 above)". Receipt: PR body matches diff.
4. **Re-brief PR #2745 (WI-2)** under (c) — author projection function +
   pinned `Workflow` value, not a bare `gunbc_ci_yml_workflow` value at
   `gunbc.ci`. Or hold PR #2745 closure pending S1 brief.
5. **Decide whether S1 (projection function) is a new §1.8 gate** or part
   of `workflow_emission_target_toggle_proven` acceptance.

**If Director rejects (c)** in favor of (a) or (b): the dual-authority
condition at HEAD remains and needs separate retraction work. Under (a),
PR #2746 substrate ratification needs withdrawal + actions.dag carrier
retirement plan. Under (b), PR #2736's `CIWorkflowDag` needs withdrawal +
re-expression of gate-dependency through `Job.needs` (lossy; provider-coupled).
Both rejections cost more substrate churn than (c) ratification.

---

## §6. Open questions surfaced (not pre-authored)

1. **S1 projection function placement**: `dsl/gunbc/ci_emission.dag`?
   `dsl/gunbc/ci.dag` itself? A new module? Director-tier concept-layering
   call.
2. **Projection function signature**: take `EmissionTarget` as parameter, or
   read it from the input `CIWorkflowDag` (would require adding the field
   there, contradicting M9)? This canvas defers — PR #2746 places the field
   on `Workflow` (the projection output), so the projection function
   signature is likely `(CIWorkflowDag, EmissionTarget) → Workflow`.
3. **Multi-provider extension path**: does (c) imply that future GitLab-CI
   support adds `dsl/extdeps/gitlab/ci.dag` + `project_gitlab_ci: CIWorkflowDag → GitLabPipeline`?
   This canvas asserts yes (per the cost-of-change decoupling argument) but
   does not pre-author the GitLab carriers.
4. **Clean-emission contract relation**: how does the projection function
   interact with `docs/design-clean-emission-contract.md` declarations on
   ci.yml? Likely: the contract declares ci.yml's required structure; the
   projection function output satisfies the contract by construction; Slice
   8 acceptance asserts the satisfied contract is the actual ci.yml (or
   ci.yml is absent).

---

## §7. Addendum — INVARIANTS P2/P1 BLOCKING reframes (c) (2026-05-12 ~06:54Z)

**Trigger**: BLOCKING inline review on PR #2746 at
`docs/design-ci-workflow-emitter-dispatch.md:360` from briansrls
2026-05-12T06:44:13Z (comment c#4427988541), relayed by PM
`msg_31090356`:

> Adding `EmissionTarget` to `dsl/extdeps/github/actions.dag` puts a gunbc
> emission/runtime choice into the GitHub Actions platform model, violating
> extdeps fidelity and INVARIANTS P1.

**Discriminator** (verbatim, `dsl/extdeps/github/actions.dag:1-12` header;
memorialized as `feedback_extdeps_header_discriminator_before_field_placement`
per Director, 2026-05-12):
> Models the GitHub Actions CI/CD platform as external dependency facts...
> These are platform constraints — what GH Actions provides and requires —
> not CI logic (that lives in `gunbc/ci.dag`).

### §7.1 Argument acceptance

The BLOCKING finding is **structurally correct**. The extdeps fidelity
boundary is explicit at the file header: `extdeps.github.actions` models
**platform facts** (what the runtime consumes); `gunbc/ci.dag` models **CI
logic** (what gunbc emits/dispatches). `EmissionTarget` (`YamlStatic |
BinaryShim | PythonShim | InlineGunbc`) is a gunbc emission-policy fact —
the runtime does not consume "emission target"; it consumes whatever YAML
or shim ends up at `.github/workflows/ci.yml`. The selection between
projection shapes is a gunbc decision about which artifact gunbc renders.

**Authority citation correction (per gpt-5.5-pro review 9972)**: the
single-authority / "every fact lives in exactly one place" principle is
`INVARIANTS.md` **P2 Boundary Discipline**, not P1. P1 is Modeling
Faithfulness / grounding. The operator BLOCKING quote above invokes "P1"
verbatim; both principles are in play here — P1 because adding gunbc-policy
state to an extdeps carrier makes the carrier no longer faithful to the
extdeps header's "platform facts only" claim, and P2 because emission-policy
authority must live in exactly one place (`gunbc/ci.dag`), not split across
extdeps and gunbc. Subsequent canvas references to single-authority cite
**P2**; the extdeps-fidelity dimension cites **P1**.

Per `INVARIANTS.md` P2 (Boundary Discipline): emission-policy authority
belongs to `gunbc/ci.dag` (single authority), **not** `extdeps.github.actions`
(which holds platform authority). Placing `emission_target` on
`extdeps.github.actions.Workflow` creates a dual-authority condition where
the extdeps file holds a gunbc-policy fact alongside its platform facts.
Per P1 (Modeling Faithfulness): the extdeps carrier would no longer
faithfully model what its header claims to model ("platform constraints,
not CI logic").

PR #2746 canvas §3.1 explicitly weighed and rejected the parallel objection
("`extdeps.github.actions` describes platform constraints, not gunbc CI
policy"), but the rebuttal — "the workflow carrier already describes a
provider artifact" — **conflates** two distinct things:

- **The platform Workflow value** — what GH Actions consumes (jobs, steps,
  triggers, permissions, runners). This IS a platform fact and correctly
  lives in extdeps.
- **The emitted artifact shape** — `.github/workflows/ci.yml` vs a shim
  invoking a compiled binary vs a Python runner. The SELECTION between
  these is a gunbc emission-policy decision.

The platform Workflow is the runtime's input contract. The emission-target
choice is gunbc's output contract for which artifact realizes that input
contract. These are different authorities on different sides of the
runtime/emitter boundary.

### §7.2 Implications for §1–§6

This addendum **disqualifies** §1 option (b) AS-AUTHORED and **partially
invalidates** §1 option (c) as authored above:

- **Option (b) — `EmissionTarget` on `actions.Workflow`**: DISQUALIFIED per
  §7.1. The substrate-decision in PR #2746 stands as a SHAPE (sum type
  needed; sibling wrapper still rejected for the same join-cost reason),
  but the PLACEMENT moves out of extdeps.
- **Option (c) — hybrid retaining `Workflow.emission_target`**: PARTIALLY
  INVALIDATED. The two-layer concept-layering argument (semantic source at
  `gunbc.ci.CIWorkflowDag` + transport at `extdeps.github.actions.Workflow`)
  REMAINS CORRECT. The `EmissionTarget` placement on the transport carrier
  is what fails P1.

The §2 four-axis comparison conclusions REMAIN VALID for the layering
question: only a hybrid satisfies single-authority per layer + concept
layering simultaneously. What changes is **where `EmissionTarget` lives**
within the hybrid.

### §7.3 Refined option (c) — "(c-refined)"

`EmissionTarget` lives in **`gunbc/ci.dag`** (gunbc namespace; emission-
policy authority) as a sum type. It is **not** a field on any extdeps
carrier. The projection function takes it as a parameter:

```dag
// in dsl/gunbc/ci.dag (or dsl/gunbc/ci_emission.dag — placement open)
type EmissionTarget
  = YamlStatic
  | BinaryShim
  | PythonShim
  | InlineGunbc

// project_github_actions: CIWorkflowDag x EmissionTarget -> Workflow
//   Pure structural projection (per docs/design-emission-model.md):
//   reads gate-dependency DAG + target selector, emits actions.Workflow
//   value that the emitter renders to the chosen artifact shape.
fn project_github_actions(
  ci_workflow_dag: CIWorkflowDag,
  target: EmissionTarget,
) -> Workflow
```

`extdeps.github.actions.Workflow` is **unmodified** — no `emission_target`
field added. The platform model stays bounded to platform facts.

The `Workflow` value WI-2 was authoring at PR #2745 (the structural mirror
of ci.yml) is, under (c-refined), the **bound result of the projection
function** — `data gunbc_ci_yml_workflow: Workflow = project_github_actions(ci_workflow_dag, YamlStatic)`
— living in **gunbc namespace**, not in `dsl/extdeps/github/ci.dag` (the
path PR #2745 currently uses, which is also extdeps-bounded). The name is
a binding to the derived result; there is no separate hand-declared
`Workflow` value as modeled authority (see §8 retraction). This re-routes
WI-2 substrate work to gunbc namespace under (c-refined) AND tightens it
to single-authority projection-output binding.

### §7.4 Comparison table revised

Replacing the §2.4 row "Workflow-artifact authority":

| Axis | (a) | (b) AS-AUTHORED | (c) AS-AUTHORED | **(c-refined)** |
|---|---|---|---|---|
| Gate-dependency authority | ✓ `CIWorkflowDag` | ✗ implicit `Job.needs` | ✓ `CIWorkflowDag` | ✓ `CIWorkflowDag` |
| Workflow-artifact authority | ✗ unmodeled | ✓ `Workflow` (platform-coupled emission-policy) | ✓ `Workflow` (same) | ✓ `Workflow` (platform-pure) + projection-fn in gunbc |
| Emission-target carrier | ✗ build-system implicit | ✗ on extdeps (P1 violation) | ✗ on extdeps (P1 violation) | ✓ `gunbc.ci.EmissionTarget` (gunbc-namespace) |
| INVARIANTS P2 (Boundary Discipline / single authority) | ✗ dual-authority unresolved | ✗ extdeps holds gunbc-policy fact | ✗ extdeps holds gunbc-policy fact | ✓ each authority single-sourced |
| Extdeps fidelity (file header :1-12) | n/a | ✗ violated | ✗ violated | ✓ preserved |
| Sufficient for FULL R3-close | no | no | no (P1) | **yes** |

### §7.5 Revised recommendation

**Adopt option (c-refined): `EmissionTarget` lives in `gunbc/ci.dag` as a
parameter to `project_github_actions(ci_workflow_dag, target) → Workflow`;
`extdeps.github.actions.Workflow` stays unmodified; the only `Workflow`
values in modeled authority are bindings to the projection function's
output (e.g., `data gunbc_ci_yml_workflow: Workflow = project_github_actions(...)`).
No separately-declared `Workflow` value exists as modeled authority.**

**Ratification asks revised** (supersedes §5):

1. **Ratify (c-refined)** as gate #56 substrate-shape under T-CI-WAD program-tag.
2. **PR #2746 disposition changes from "merge with framing-narrowing
   addendum" to "substantive substrate retraction"**: the `EmissionTarget?
   on actions.Workflow` field placement is withdrawn. PR #2746 canvas
   §3.1 reasoning narrows to "`EmissionTarget` substrate-decision
   (sum-type, not sibling-wrapper) is correct; placement moves from
   extdeps to gunbc namespace per INVARIANTS P2 (boundary discipline) +
   P1 (extdeps-carrier modeling faithfulness)". Suggest PR #2746
   authors a §7 addendum mirroring this canvas's §7 + revises §3.1/§3
   accordingly, OR PR #2746 is held while (c-refined) is implemented as a
   replacement canvas. Director call.
3. **PR #2736 body correction still required** per §5.3 (independent of §7;
   the body/diff mismatch is orthogonal to the P1 finding).
4. **PR #2745 (WI-2) re-brief under (c-refined)**: relocate
   `gunbc_ci_yml_workflow` from `dsl/extdeps/github/ci.dag` to a gunbc-
   namespace module (e.g., `dsl/gunbc/ci_emission.dag`); add the
   projection function declaration; pinned `Workflow` value is the
   projection invocation result. This change is larger than §5.4 implied.
5. **Decide S1 (projection function) gate status** per §5.5 — unchanged.

**Pre-canvas Director ratification reconciliation**: Director ratified PR
#2746's substrate shape at `msg_237bde05` before this BLOCKING finding
surfaced. Per `feedback_pre_compaction_framings_self_supersede` and the
broader operator-canvas-supersedes-prior-Director-ratification pattern
when an INVARIANTS violation is identified: this canvas's (c-refined)
recommendation supersedes the pre-canvas Director ratification. Director
ratifies the surfaced shape (per
`feedback_substrate_shape_belongs_in_mgr_canvas`), not the pre-surfaced
shape.

### §7.6 Why (c-refined) is not PM's "(b)" or "(c)" in `msg_31090356`

PM relay `msg_31090356` proposed:
- PM (b) — `EmissionTarget` on `gunbc.ci.CIPipeline`
- PM (c) — Hybrid wrapper `WorkflowEmission { workflow: actions.Workflow, target }` in gunbc namespace

(c-refined) is neither:

- **vs PM (b)**: putting `EmissionTarget` on `CIPipeline` couples emission-
  policy to the gate-list carrier. The same join-cost problem WI-1 canvas
  §3 Option B already flagged applies — `CIPipeline` is gate-centric, not
  emission-artifact-centric. PM (b) trades one P1 problem for a different
  M9 concept-layering problem.
- **vs PM (c)**: a `WorkflowEmission { workflow, target }` wrapper IS the
  sibling-wrapper PR #2746 §3 Option C explicitly rejected — preserves
  separation superficially but introduces an implicit join (which
  `Workflow` corresponds to which emission target) and a duplicate
  authority surface.

(c-refined) instead expresses the emission-target choice **at the
projection invocation**, not as a field on any carrier. This is the
shape `docs/design-emission-model.md` prescribes: emission is structural
projection; the projection function reads target spec + source; there is
no carrier-level "this Workflow is targeted at X" fact because emission
choice is a property of the emit-call, not a property of the value.

The `EmissionTarget` sum-type itself lives in gunbc namespace (as a
named type usable anywhere gunbc CI logic needs to reference emission
targets), but no `EmissionTarget` **field** lives on any existing
carrier. This eliminates both P1 (no extdeps modification) and M9 (no
gate-centric carrier extension).

---

## §8. Retraction — pinned `Workflow` as separate authority (2026-05-12 ~06:57Z)

**Trigger**: codex/codex-default REQUEST_CHANGES on PR #2749
(`/api/reviews/9970/artifacts/stdout.log`) flagged option (c) as written:

> The option (c) design reintroduces a second authority for the GitHub
> Actions artifact by proposing both a projection function
> `CIWorkflowDag → Workflow` and a hand-declared `Workflow` value that
> emission "validates against" as a pinned target. That conflicts with
> `INVARIANTS.md` P2 / `docs/modeling-discipline.md` Practice 5: the
> workflow artifact fact would now live in two places, with coherence
> enforced behaviorally by comparison instead of structurally by
> deriving the artifact from one source.

**Finding accepted**. Earlier framings in §1 (option (c) intro), §4 (S0,
S1), §5 ratification ask #4, §7.3, and §7.5 referred to a "pinned
`Workflow` value the projection is validated against" as a substrate
fact. That formulation creates exactly the P2 dual-authority condition
this canvas was supposed to dissolve: one `Workflow` value derived from
`project_github_actions`, another hand-declared, with coherence enforced
by behavioral comparison rather than structural derivation.

**Retracted framing**: any sentence implying that WI-2 (or any future
work) authors a hand-declared `Workflow` value as **modeled authority**
which the projection output is compared against.

**Replacement framing** (already applied in-place to §1, §4 S0, §5
ask #5, §7.3, §7.5):

- The only `Workflow` value in modeled authority is the **output of
  `project_github_actions(ci_workflow_dag, target)`**, structurally
  derived from a single source.
- WI-2's existing `gunbc_ci_yml_workflow` draft is re-scoped to a **name
  binding to the derived result** — `data gunbc_ci_yml_workflow:
  Workflow = project_github_actions(ci_workflow_dag, YamlStatic)`. The
  name binds to the derived result; it is not an independent
  declaration.
- A byte-level regression fixture comparing emitted YAML against a
  checked-in expected-output file is a **test artifact** (lives in
  `tests/`, not `dsl/`); it is a receipt of past projection output for
  drift detection, **not** part of the modeled authority surface, and
  not consumed by emission.

The §2 / §7 layering argument is unchanged: gate-dependency lives at
`gunbc.ci.CIWorkflowDag`; transport platform model lives at
`extdeps.github.actions.Workflow`; emission policy
(`EmissionTarget` + `project_github_actions`) lives in gunbc namespace;
the workflow artifact is derived by the projection function from a
single source.

---

**Authored by**: warm-wolf-698 (R3 Substrate Mgr) per Director (zesty-bear-812)
canvas-authoring directive 2026-05-12 ~06:50Z via PM (deep-wolf-155) relay
`msg_a945b141`. §7 addendum 2026-05-12 ~06:54Z per BLOCKING relay
`msg_31090356`. §8 retraction 2026-05-12 ~06:57Z per codex
REQUEST_CHANGES on PR #2749 (review 9970).

**Canvas readiness for Director ratification**: SURFACED with §7 INVARIANTS
P1 addendum revising recommendation to **option (c-refined)**: `EmissionTarget`
as projection-function parameter in gunbc namespace; `extdeps.github.actions.Workflow`
unmodified. Director ratifies surfaced shape; Mgr proceeds with downstream
brief authoring per ratified shape.
