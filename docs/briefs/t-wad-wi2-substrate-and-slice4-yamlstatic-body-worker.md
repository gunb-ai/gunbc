# T-WAD WI-2 Substrate Reattempt + Slice 4 YamlStatic Body — Worker Brief

**Owner**: clever-lark-568 (T-WAD Slice 4 lane child of warm-wolf-698)
**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Authoring date**: 2026-05-12
**Bundle rationale**: per `feedback_bundle_workstreams_per_pr` + `feedback_single_bundle_ratification_uniform_substrate_cause` — substrate-cause is uniform (landing the WAD-emitter authority); splitting risks WI-2-only PR landing as a TODO-skeleton with no consumer (P5 scaffold-with-no-consumer reviewer pressure).

---

## §0. Status — DISPATCH BLOCKED on prerequisites

**Do not start authoring until both prerequisites clear**:

1. **PR #2736 (`gate #56 ci_workflow_modeled_as_dag` / neat-badger-30) merged.** `CIWorkflowDag` / `CIGateNode` / `CIGateEdge` carriers in `dsl/gunbc/ci.dag` are the input-domain for `project_github_actions`. Current state (2026-05-12T~15Z): mergeable=CONFLICTING, codex RC stale. Track via `gh pr view 2736 --json mergeable,reviewDecision`.
2. **§5.5 audit in PR #2751** (already merged 2026-05-12T~13Z) — gives the canonical 18-site Expression-substrate scope your YamlStatic body consumes; cross-reference at `docs/design-gh-actions-expression-substrate-2026-05-12.md` §5.5 in main.

**If PR #2736 remains blocked >30min from this brief's dispatch timestamp**, ping warm-wolf-698 — operator-tier merge-bypass path may apply.

---

## §1. Scope

Two coupled deliverables in one PR:

### Phase A — WI-2 substrate creation (substrate-shape)

NEW file `dsl/gunbc/ci_emission.dag` declaring:

1. **`WorkflowRuntime` open enum** with **3 initial arms** per the ratified emitter-dispatch canvas (`docs/design-ci-workflow-emitter-dispatch.md:126` in main):
   - `YamlStatic` — emit a static YAML artifact
   - `BinaryShim` — emit a thin YAML shim invoking a compiled binary entry-point
   - `PythonShim` — emit a thin YAML shim invoking an emitted Python CI runner (parallel to BinaryShim; proves target shape generalizes)

   **DO NOT** add `InlineGunbc` arm — `InlineGunbc` is DESIGN-ONLY per emitter-dispatch canvas §5.4 (no enum variant until a real runtime consumer exists). An earlier draft of this brief erroneously demoted `PythonShim` to DESIGN-ONLY; operator BLOCKING on PR #2762 at :17 (2026-05-12T17:25:05Z) caught the stale 2-arm framing that survived squash-merge of PR #2762 (commit 0d95ba09d) despite later sections already saying 3-arm. This fix-forward PR closes the internal inconsistency.

2. **`project_github_actions` function signature**:
   ```
   func project_github_actions(ci_workflow_dag: CIWorkflowDag, runtime: WorkflowRuntime) -> extdeps.github.actions.Workflow
   ```
   Per-arm bodies in Phase B (YamlStatic only; BinaryShim + PythonShim TODO-stubbed).

3. **Practice 4 receipt at declaration site** — 🟡 YELLOW classification with sum-of-tagged-coordinates carrier (NOT Cartesian product). Reference shape from PR #2744 commit `e6d302352` / WI-1 canvas §3:
   ```
   // 🟡 YELLOW (scaffold) — coproduct-dissolution receipt.
   // sum-of-tagged-coordinates carrier shape (NOT Cartesian product
   // of (EmissionArtifactShape × ShimRunnerKind)):
   //   Static(EmissionArtifactShape) | Shim { runner: ShimRunnerKind }
   // Dissolution triggers: (a) 4th arm pressure (InlineGunbc design-only
   // pending real runtime consumer per emitter-dispatch canvas §5.4);
   // (b) per-arm body axis discovery during Slice 4/5 implementation;
   // (c) consumer-side dimension extraction.
   ```

4. **NO pinned `gunbc_ci_yml_workflow` binding** at this stage. Pinning is DEFERRED per WI-2 brief §3 until the canonical `CIWorkflowDag` instance lands with the Slice 4 projection-arm body — which is in this same PR (Phase B), so pinning **may** land here if Phase B succeeds. If pinning lands, declare it in `dsl/gunbc/ci_emission.dag` with explicit reference to the input `CIWorkflowDag` instance.

5. **Imports**: import `CIWorkflowDag` from `gunbc.ci` (post-PR-#2736); import `Workflow` from `extdeps.github.actions`.

**Closes gates**:
- `workflow_runtime_open_enum_landed` (gate 99)
- `project_github_actions_landed` (gate 100)

### Phase B — Slice 4 YamlStatic projection-arm body (enabling deliverable for Slice 8)

Implement the `YamlStatic` arm body of `project_github_actions`:

1. **Walk `CIWorkflowDag`** (the gate-dependency graph from PR #2736) — gates become Steps, edges become `Job.needs` ordering.
2. **Emit a `Workflow` value** (the `extdeps.github.actions.Workflow` carrier) — **strict single-authority derivation**: every `Workflow` / `Job` / `Step` field MUST come from a value present on the `CIWorkflowDag` input (or transitively reachable via an already-modeled `gunbc.ci.*` carrier such as `CIGate`). **No structural defaults, no fabricated values, no second source of truth.** If a target field on `Workflow` has no source in the input domain at authoring time:
   - **STOP authoring.** Do NOT invent a default, hardcode a literal, or import a value from `.github/workflows/ci.yml`.
   - Surface the carrier-gap to warm-wolf-698 (see §1 Phase B carrier-gap protocol below) — gap resolution is a hard prerequisite, not a side-channel.
   - This applies in particular to: `name`, `on` triggers, `env`, `permissions`, and any per-`Job` / per-`Step` field whose value isn't carried by the input. **P2/P3 single-authority bar**: a `Workflow` value with any fabricated field violates INVARIANTS P2 (single authority) and P3 (no second source of truth) per codex review 10208 on PR #2762.
   - **18-site Expression-substrate consumption** per PR #2751 §5.5 (live canvas authority):
     - 17 string-container sites (exact enumeration per PR #2751 §5.5.1, NOT shorthand — `RunStep.*` / `UsesStep.*` glob would incorrectly pull in `UsesStep.uses` (literal-only per GH workflow-syntax) + typed-HOLD fields per operator BLOCKING PR #2768 :70):
       - `Workflow.env`
       - `Job.name`, `Job.if_condition`, `Job.env`, `Job.concurrency.group`
       - `RunStep.name`, `RunStep.run`, `RunStep.env`, `RunStep.working_directory`, `RunStep.if_condition` (5 RunStep fields; NOT `timeout_minutes`/`continue_on_error` — typed HOLD per §6 Q#4)
       - `UsesStep.name`, `UsesStep.with`, `UsesStep.env`, `UsesStep.if_condition` (4 UsesStep fields; NOT `uses` — literal-only; NOT `timeout_minutes`/`continue_on_error` — typed HOLD)
       - `MatrixStrategy.dimensions`, `MatrixStrategy.include`, `MatrixStrategy.exclude` (3 MatrixStrategy fields; NOT `fail_fast`/`max_parallel` — typed HOLD)
     - 1 enum-extension site (Job.runner: RunnerSpec — scalar-expression case only via `ExpressionRunner { expr: Expression }`)
     - At each site: emit `Expression::OpaqueString(s)` variant by unwrapping `s` and emitting verbatim into YAML (single-arm pattern match, no expression grammar engine needed)
   - **DO NOT** migrate these (out-of-scope per §6 deferrals):
     - `DispatchInput.default` — carrier-split-BLOCKED per §6 Q#5
     - 9 typed-field HOLD sites — pending §6 Q#4 ratification
     - Array/object `runs-on` cases — pending §6 Q#6 carrier-split
3. **Carrier-gap audit during authoring — STOP CONDITION**: 4 candidate substrate-prereqs are pre-surfaced in WI-2 brief §Reference materials (Workflow.concurrency, PullRequestActivity.ReadyForReview, Push.paths-Optional, WorkflowPermissions.*-Optional); additional gaps may surface during authoring. For each gap encountered:
   - **STOP authoring this PR.** Do NOT continue the YamlStatic body with the gap unaddressed; do NOT land a substrate prereq inside this PR (separate substrate-prereq lane).
   - Surface the gap to warm-wolf-698 via internal message: which carrier, which field, which CIWorkflowDag value source was missing, what `.github/workflows/ci.yml` semantics need it.
   - **Wait for warm-wolf-698 resolution** — either (a) substrate-prereq PR lands in a separate lane and you rebase, (b) carrier-gap is judged out-of-scope-for-Slice-4 and the corresponding `.github/workflows/ci.yml` semantics get explicitly out-of-scoped + Slice 4 acceptance narrowed, or (c) brief is revised with explicit guidance.
   - **Do NOT resume Slice 4 body authoring** until the gap is resolved by one of the three paths above. Continuing with a gap = fabricated authority = P2/P3 violation (codex review 10208).
   - If the gap is NOT hit during authoring (the CIWorkflowDag instance for the current `.github/workflows/ci.yml` doesn't exercise that surface), explicitly note in PR body which pre-surfaced gaps remain unaddressed at landing time.
4. **Acceptance**: deterministic YAML encoding **semantically equivalent** to current `.github/workflows/ci.yml`. Regression-guard byte-identity is to **fresh projection output**, NOT to legacy hand-authored YAML (internal byte-identity per PR #2744 / §3 Slice 4 framing). Field-ordering rules, indentation, list/scalar conventions: match current ci.yml.
5. **BinaryShim + PythonShim arm bodies**: TODO-stubbed (Phase 5 BinaryShim lane handles BinaryShim; PythonShim body lands in a separate worker post-BinaryShim per emitter-dispatch canvas §5.3).

**Closes gates**: NONE for Phase B in isolation. Phase B is **enabling deliverable** for Slice 8 gate `ci_yml_hand_authority_dissolved` (gate 98) — Slice 8 owns the actual closure via artifact-swap with regression-guard.

---

## §2. PR body framing

Title: `T-WAD R3: WI-2 substrate (WorkflowRuntime + project_github_actions) + Slice 4 YamlStatic body`

Body must explicitly enumerate gate closures (per `feedback_one_canonical_subissue_per_workitem`):

```
Closes gate 99 `workflow_runtime_open_enum_landed` (WorkflowRuntime open enum declared).
Closes gate 100 `project_github_actions_landed` (projection function signature declared).

Slice 4 YamlStatic projection-arm body lands as enabling substrate for Slice 8 gate 98
`ci_yml_hand_authority_dissolved` (Slice 8 owns closure via artifact-swap with regression-guard).
```

PR body must cite:
- PR #2751 §5.5 as 18-site Expression-substrate authority (NOT the WI-1 brief's stale "5-site" framing)
- PR #2744 commit e6d302352 as Practice 4 sum-of-tagged-coordinates precedent
- `docs/design-ci-workflow-emitter-dispatch.md:126` (in main) as 3-arm `WorkflowRuntime = YamlStatic | BinaryShim | PythonShim` ratified scope; §5.4 keeps `InlineGunbc` design-only pending real runtime consumer
- PR #2736 as input-domain carrier source

---

## §3. Verification before PR-ready flip

Before `gh pr ready`:

1. `cargo test --workspace` green
2. `cargo clippy --all-targets -- -D warnings` clean
3. `cargo fmt --all --check` clean
4. **YAML equivalence test** authored against current `.github/workflows/ci.yml` (fresh-projection byte-identity, not legacy-hand-authored byte-identity)
5. **4-axis grep audit** of the brief itself:
   - WorkflowRuntime — already collision-cleared (PR #2749 §7.3.3; PR #2756 cascade rename)
   - project_github_actions — no collisions in `src/v3/SELF_HOSTING.md` or `dsl/std/`
   - Practice 4 sum-of-tagged-coordinates — PR #2744 e6d302352 verbatim
   - 18-site scope — PR #2751 §5.5 verbatim

Surface any of these failing to warm-wolf-698 before PR-ready flip.

---

## §4. Out of scope

- Slice 5 BinaryShim body (neat-crane-827 lane)
- Slice 8 ci.yml artifact-swap with regression-guard (separate worker, depends on this PR's Phase B landing)
- Substrate-prereq PRs for carrier gaps (Workflow.concurrency / PullRequestActivity.ReadyForReview / Push.paths-Optional / WorkflowPermissions.*-Optional) — surface to warm-wolf-698 if hit during authoring; bundle decision deferred
- `DispatchInput` carrier-split per §6 Q#5 (separate prereq lane)
- `RunnerSpec` runs-on grammar carrier-split per §6 Q#6 (separate prereq lane)
- Typed-field migration shape per §6 Q#4 (Director-tier question)

---

## §5. Reference

- `dsl/extdeps/github/actions.dag` — `Workflow` / `Job` / `Step` / `RunStep` / `UsesStep` / `RunnerSpec` / `ConcurrencySpec` / `MatrixStrategy` / `DispatchInput` carriers; header lines 1-12 set platform-vs-CI-logic discriminator
- `dsl/gunbc/ci.dag` — `CIGate` / `CIPipeline` existing carriers; `CIWorkflowDag` lands via PR #2736
- `docs/design-gh-actions-expression-substrate-2026-05-12.md` (in main post-PR #2751) — §5.5 audit table is the 18-site authority
- `docs/design-ci-workflow-substrate-shape-2026-05-12.md` (in main post-PR #2749 + #2756) — §7.3 ratification of (c-refined) substrate shape
- `docs/design-ci-workflow-emitter-dispatch.md` (in main post-PR #2746 + #2756) — WI-1 emitter dispatch framing
- `src/v3/SELF_HOSTING.md:609` — `EmissionTarget` Shape-A authority (do NOT collide with this name; we use `WorkflowRuntime`)
