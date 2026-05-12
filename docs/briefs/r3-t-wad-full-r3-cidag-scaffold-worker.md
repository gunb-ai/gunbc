# Worker brief — T-WAD FULL R3 ci_emission.dag substrate scaffold

**Authority**: PM scoping doc `docs/r3-t-workflow-as-data-full-r3-close-scope.md` §6 WI-2; operator FULL elevation 2026-05-12; Director ratification msg_5cbdad24 + (c-refined) ratification msg_237bde05 / msg_f9fd669e 2026-05-12.
**Parent**: T-Workflow-As-Data lane (Substrate Mgr warm-wolf-698 lane-absorbed Slices 4-5/8 per Director); this WI lands the projection-function substrate that enables Slice 4 emitter implementation.
**Closure predicate**: `dsl/gunbc/ci_emission.dag` declares `WorkflowRuntime` open enum + Practice 4 coproduct-dissolution receipt (per acceptance gate 3) + `project_github_actions: (CIWorkflowDag, WorkflowRuntime) -> Workflow` projection function signature with TODO-marked total-handling skeleton. The `gunbc_ci_yml_workflow` pinned-projection data binding is **DEFERRED to Slice 4** per §3 + acceptance gate 5 (P2 single-authority lockdown — the binding requires a canonical `CIWorkflowDag` instance which Slice 4 authors alongside the binding; WI-2 landing the binding now would force placeholder-or-conversion authority creation per briansrls BLOCKING c#3224878308 fix in commit `f830b988f`). Downstream Slice 4 YamlStatic emitter consumes the projection and authors the pinned binding; downstream Slice 8 dissolves hand-authority over `.github/workflows/ci.yml` (per `ci_yml_hand_authority_dissolved` gate — see scope doc §1).

## Substrate-shape ratification anchor

The (c-refined) shape was ratified by Director at msg_237bde05 + msg_f9fd669e and self-corrected to via PR #2749 §7 (warm-wolf-698). The earlier (a) shape (`WorkflowRuntime?` field on `dsl/extdeps/github/actions.dag::Workflow`) was RETRACTED at Director msg_b4151f45 + codex BLOCKING #9970 on PR #2749 for INVARIANTS P1 violation (CI logic placed on platform carrier).

**The ratified shape**: `WorkflowRuntime` is a parameter to a projection function in `dsl/gunbc/`, NOT a field on any carrier. The function's invocation pin produces the `Workflow` value the emitter consumes.

## Output

**Create NEW file `dsl/gunbc/ci_emission.dag`** declaring:

1. **`WorkflowRuntime` open enum** (sum-type with named arms; OPEN per design):

   ```
   WorkflowRuntime = YamlStatic
                  | BinaryShim
                  | ...           // open enum — additional targets land when real consumers exist
   ```

   Each arm is a tag indicating the emission strategy. **Initial set: 2 arms** (`YamlStatic`, `BinaryShim`) — both have concrete consumer-paired slices (Slice 4 / Slice 5). **`PythonShim` AND `InlineGunbc` are DESIGN-ONLY** held out of the initial enum until real runtime consumers exist. Per INVARIANTS P5 / Pure Bootstrap discipline + sibling WI-1 brief framing (PythonShim "Future; sketch only" — no concrete Slice consumer in scope) + PR #2746 §5.4 (InlineGunbc) + openai-pro BLOCKING on PR #2744 (2026-05-12 ~07:55Z) for InlineGunbc + briansrls BLOCKING on PR #2744 (2026-05-12T10:46:59Z) for PythonShim asymmetry: NO enum variant or emitter arm lands without a concrete consumer-paired slice. Open-enum discipline: the projection function must total-handle the declared arms; unknown arms compile-fail (per fail-closed discipline `feedback_fail_closed_discipline`); future arms (`PythonShim`, `InlineGunbc`, and any others) land via separate substrate-prereq PRs paired with their consumers.

2. **`project_github_actions` projection function declaration**:

   ```
   project_github_actions: (CIWorkflowDag, WorkflowRuntime) -> Workflow
   ```

   - Input domain: `(ci_workflow_dag: CIWorkflowDag, target: WorkflowRuntime)`
   - Output codomain: `Workflow` (the `dsl/extdeps/github/actions.dag::Workflow` platform carrier)
   - **This WI lands the DECLARATION + signature**, not the per-arm implementation. Per-arm bodies (YamlStatic projection, BinaryShim projection, etc.) are Slice 4+ work owned by Substrate Mgr.
   - Leave function body as `// TODO Slice 4-5: per-arm projection implementations` with structural total-handling skeleton (match on `target` arms; each arm body marked TODO).

3. **`gunbc_ci_yml_workflow` pinned-projection data binding** — **DEFERRED TO SLICE 4** (P2 single-authority lockdown):

   The binding will eventually take this form:

   ```
   gunbc_ci_yml_workflow: Workflow = project_github_actions(<canonical_ci_workflow_dag>, YamlStatic)
   ```

   But WI-2 does **NOT** land this binding. Rationale (per briansrls BLOCKING on PR #2744 2026-05-12T08:30:15Z c#3224878308): the binding requires a concrete `CIWorkflowDag` *instance* whose authority is locked to a canonical source. No such canonical instance exists in main today — `CIWorkflowDag` was introduced as a carrier (type) by PR #2736 (neat-badger-30), but the first canonical *value* (the CI-workflow-as-data corresponding to current `.github/workflows/ci.yml`) lands as part of **Slice 4** (YamlStatic projection-arm implementation, Substrate Mgr warm-wolf-698 lane). If WI-2 landed the binding now, the worker would have to either (a) invent a placeholder `CIWorkflowDag` value inline in `ci_emission.dag` (creating a parallel authority alongside the eventual Slice 4 canonical instance — P2 violation), or (b) build a `CIWorkflowDag` value from `CIPipeline` via inline conversion (reopens Path (a) authority despite the explicit Path (b) rejection below — P2 violation), or (c) leave the binding as a non-compiling forward-reference.

   **WI-2 scope therefore lands only items 1 (`WorkflowRuntime` enum) + 2 (function signature)**. The pinned-projection binding is a Slice 4 deliverable, not a WI-2 deliverable. Slice 4 authors the canonical `CIWorkflowDag` instance and the pinned-projection binding together, sourcing the binding's `ci_workflow_dag` argument from that canonical instance (single authority, P2-clean).

   Forward reference for Slice 4 (informational, not WI-2 work): the pinned binding will be the canonical YAML-emission point Slice 4's YamlStatic emitter walks to produce `.github/workflows/ci.yml`-equivalent output (per scope-doc §3 Slice 4).

## CIWorkflowDag dependency sequencing — Path (b) REQUIRED (Substrate Mgr clarification 2026-05-12 msg_27d99080)

**The projection function takes a `CIWorkflowDag` input.** Substrate Mgr (warm-wolf-698) clarified at msg_27d99080 that Path (a) is INSUFFICIENT:

**Path (a) — reuse existing `dsl/gunbc/ci.dag::CIPipeline`** — REJECTED. `CIPipeline { name, gates: List<CIGate> }` is a FLAT gates list without edge/dependency structure. `project_github_actions` consumes gate-DEPENDENCY (which gates depend on which); flat `List<CIGate>` cannot serve as the projection input. Path (a) is INSUFFICIENT for the projection's input requirements.

**Path (b) — use `CIWorkflowDag` carrier from PR #2736 (neat-badger-30)** — REQUIRED. `CIWorkflowDag { name, nodes: List<CIGateNode>, edges: List<CIGateEdge> }` is the load-bearing semantic carrier carrying gate-dependency structure. This is **ALREADY canvas-tier ratified** (PR #2749 §1 option (a) discussion + §2.4 / §7.4 tables explicitly position CIWorkflowDag as the gate-dependency authority on which (c-refined) composes) AND Director-ratified (msg_4f7f536d). PR #2736 is the carrier-introduction PR; canvas + Director ratification covers Path (b) — **no additional Mgr ratification needed for the CIWorkflowDag choice itself**.

**Disposition**: Proceed Path (b) directly. Use `CIWorkflowDag` from PR #2736 as the projection function's input domain.

**Sequencing implication** (per Substrate Mgr msg_27d99080):
- WI-2 implementation **depends on PR #2736 merge** OR references the in-flight `CIWorkflowDag` type
- If PR #2736 is still HOLD-merging when cool-carp-720 opens WI-2: either (1) WI-2 holds until PR #2736 merges, OR (2) WI-2's PR rebases on PR #2736's branch (`session/neat-badger-30`)
- PR #2736 mergeable=CLEAN with all CI checks SUCCESS as of 2026-05-12 (no structural blocker; awaiting review tally)

**STOP-and-route discipline** (revised):
- The CIWorkflowDag CARRIER CHOICE is canvas+Director-ratified — no STOP needed for this dimension.
- STOP still applies for OTHER substrate-shape questions (e.g., `WorkflowRuntime` open-enum vocabulary support, function-as-projection declaration vocabulary support) — see STOP / PING criteria below.
- PING-on-PR-open ratification covers shape (signature, derived-binding form, module placement), NOT the CIWorkflowDag choice in isolation.

## Scope boundaries (DO / DON'T)

**DO**:
- Create NEW file `dsl/gunbc/ci_emission.dag` (the projection-substrate file).
- Declare `WorkflowRuntime` open enum with **2** named arms (`YamlStatic`, `BinaryShim`) + open marker. **`PythonShim` AND `InlineGunbc` are DESIGN-ONLY** — do NOT add either as enum arm (PythonShim per briansrls BLOCKING on PR #2744 2026-05-12T10:46:59Z; InlineGunbc per WI-1 brief §scope + PR #2746 §5.4 + openai-pro BLOCKING on PR #2744). Both land via separate substrate-prereq PRs paired with their concrete runtime consumers.
- Declare `project_github_actions` function signature with TODO-marked total-handling skeleton.
- Author Practice 4 coproduct-dissolution receipt for `WorkflowRuntime` (🟡 YELLOW classification + named dissolution trigger + coordinate-dissolution sketch per WI-1 brief discipline; see acceptance gate 3).
- Reference existing `Workflow` carrier from `dsl/extdeps/github/actions.dag` as the codomain type.
- Reference `CIWorkflowDag` carrier from PR #2736 as the input domain type (canvas + Director-ratified per msg_4f7f536d).
- Sequence WI-2 PR after PR #2736 merge OR rebase WI-2's branch on PR #2736's `session/neat-badger-30` if PR #2736 is still HOLD-merging.

**DON'T**:
- Do NOT add fields to `dsl/extdeps/github/actions.dag` carriers (Workflow/Job/Step) — this is the INVARIANTS P1 violation the (c-refined) shape resolves.
- Do NOT extend `dsl/gunbc/ci.dag` with new substrate types — extension belongs to Mgr canvas tier.
- Do NOT use `CIPipeline` from `dsl/gunbc/ci.dag` as the projection input — it is flat without edge structure and INSUFFICIENT for `project_github_actions` (per Substrate Mgr clarification msg_27d99080).
- Do NOT implement per-arm projection bodies — those are Slice 4-5 work owned by Substrate Mgr after canvas ratification.
- Do NOT modify `.github/workflows/ci.yml` — Slice 8 dissolves hand-authority; this WI only lands the projection substrate.
- Do NOT introduce ALTERNATE carriers for the projection input — `CIWorkflowDag` from PR #2736 is the ratified authority.
- Do NOT land the `gunbc_ci_yml_workflow` pinned-projection data binding in this PR — DEFERRED to Slice 4 per §3 + acceptance gate 5 (P2 single-authority lockdown; the binding requires a canonical `CIWorkflowDag` instance, which Slice 4 authors alongside the binding).
- Do NOT author a placeholder `CIWorkflowDag` instance to satisfy the binding shape — that creates parallel authority alongside Slice 4's canonical instance (P2 violation per briansrls BLOCKING c#3224878308).
- Do NOT land the `WorkflowRuntime` enum WITHOUT its Practice 4 receipt — receipt is a co-equal substrate authoring artifact, not optional documentation (per briansrls BLOCKING c#3224878313).

## Acceptance gates

1. New file `dsl/gunbc/ci_emission.dag` exists with valid `.dag` syntax per existing v3 parser.
2. `WorkflowRuntime` open enum declared with **2** named arms (`YamlStatic`, `BinaryShim`) + open-enum marker. **NEITHER `PythonShim` NOR `InlineGunbc` arms** — both DESIGN-ONLY future targets (PythonShim per briansrls BLOCKING on PR #2744 2026-05-12T10:46:59Z; InlineGunbc per PR #2746 §5.4); each lands via separate substrate-prereq PR paired with its concrete runtime consumer per INVARIANTS P5.
3. **Practice 4 coproduct-dissolution receipt for `WorkflowRuntime`** (consistent with WI-1 brief discipline; per `feedback_coproduct_dissolution` + `modeling-discipline.md` Practice 4; addresses briansrls BLOCKING c#3224878313 on PR #2744 2026-05-12T08:30:15Z):
   - Receipt classification: **🟡 YELLOW scaffold** (flat enum with named dimensions noted: artifact-shape `StaticYaml` vs `ThinShim` × runner-realization `CompiledBinary` vs `EmittedPython`)
   - Named dissolution trigger: first additional shim runtime OR first need to share runner metadata across shim targets
   - Coordinate-dissolution sketch (in-doc, not code): the eventual factoring is `EmissionArtifactShape × ShimRunnerKind` per WI-1 canvas §3 (PR #2746 merged 08:29:09Z); declare the receipt as a `.dag` comment block at the `WorkflowRuntime` declaration site OR as a `modeling-discipline.md`-style receipt header in `ci_emission.dag` preamble. WI-2 worker MUST NOT land the enum without the receipt — receipt is part of substrate authoring discipline, not optional documentation.
4. `project_github_actions: (CIWorkflowDag, WorkflowRuntime) -> Workflow` function signature declared with structural total-handling skeleton (match on `target`; each arm body TODO-marked).
5. **NO pinned-projection data binding** (`gunbc_ci_yml_workflow`) in WI-2's PR — DEFERRED to Slice 4 per §3 above (P2 single-authority lockdown; canonical `CIWorkflowDag` instance lands with Slice 4, binding lands with the canonical instance).
6. NO modifications to `dsl/extdeps/github/actions.dag` (INVARIANTS P1).
7. NO new fields on existing `dsl/gunbc/ci.dag` carriers (`CIGate` / `CIPipeline` / `GateSource`).
8. NO new substrate types introduced without Mgr ratification (per `feedback_substrate_shape_belongs_in_mgr_canvas`).
9. `cargo test --workspace` green (no breakage of existing tests including `t_ci_workflow_as_data_demo`).
10. `cargo clippy --all-targets -- -D warnings` clean.
11. `cargo fmt --all --check` clean.

## STOP / PING criteria

- **STOP** if `WorkflowRuntime` open-enum declaration requires substrate-shape features not yet supported by current v3 surface (e.g., open-enum vocabulary missing) — surface to Substrate Mgr.
- **STOP** if `project_github_actions` function signature requires substrate features not yet supported (e.g., function-as-projection declaration vocabulary missing) — surface to Substrate Mgr; canvas-tier decision.
- **STOP** if PR #2736 (CIWorkflowDag introduction) is BLOCKED on something WI-2 implementation reveals (e.g., shape mismatch with what `project_github_actions` actually needs) — surface to Substrate Mgr; CIWorkflowDag carrier shape ratified-but-revisable.
- **PING** PM (deep-wolf-155) on PR-open for review-routing.
- **PING** Substrate Mgr (warm-wolf-698) on PR-open for Mgr-tier ratification of overall shape (signature, derived-binding form, module placement; NOT CIWorkflowDag carrier choice — that's pre-ratified).
- **COORDINATE** with sibling still-heron-763 (WI-1 emitter-dispatch canvas at PR #2746) — the canvas declares emitter-dispatch architecture this WI's projection function plugs into. PR #2749 (warm-wolf-698's adjacent substrate-shape comparison canvas) §7 is the (c-refined) ratification anchor; if either canvas re-ratifies the shape, this WI must follow.
- **COORDINATE** with neat-badger-30 (PR #2736 CIWorkflowDag introduction) if shape questions arise about the input carrier itself.

## Reference materials

- `docs/r3-t-workflow-as-data-full-r3-close-scope.md` — PM FULL scope (this brief's parent); §1 gate `project_github_actions_landed` is the closure-gate for this WI.
- `docs/briefs/r3-t-wad-full-r3-emitter-dispatch-canvas-worker.md` — sibling WI-1 brief (declares the (c-refined) substrate shape this WI implements).
- PR #2746 (still-heron-763 WI-1 emitter-dispatch canvas, `docs/design-ci-workflow-emitter-dispatch.md`) — the primary WI-1 canvas; declares emitter-dispatch architecture this WI's projection function plugs into.
- PR #2749 (warm-wolf-698 substrate-shape comparison canvas, `session/warm-wolf-698-substrate-shape-canvas`) §7 — (c-refined) shape self-correction; THIS WI implements that shape. PR #2749 §1 + §2.4 + §7.4 — canvas-tier ratification of `CIWorkflowDag` as the gate-dependency authority on which (c-refined) composes.
- PR #2736 (neat-badger-30) — `CIWorkflowDag` carrier introduction; MERGEABLE with all checks SUCCESS as of 2026-05-12; awaiting review tally; this WI consumes `CIWorkflowDag` as the projection input domain.
- Director ratification msg_4f7f536d — covers Path (b) (CIWorkflowDag-as-input) per Substrate Mgr clarification msg_27d99080.
- `dsl/extdeps/github/actions.dag:1-12` — platform-carrier scope header ("platform constraints, not CI logic (that lives in gunbc/ci.dag)"); the discriminator that grounds the (c-refined) shape.
- `dsl/extdeps/github/actions.dag:21` — `Workflow` carrier (codomain type for `project_github_actions`).
- `dsl/gunbc/ci.dag::CIPipeline` — existing gate-centric CI substrate; INSUFFICIENT as projection input (flat without edge structure) per Substrate Mgr msg_27d99080. Do NOT use as `project_github_actions` input.
- `feedback_audit_extdeps_header_for_logic_vs_platform_discriminator.md` — the discipline that catches (a)-shape mis-placements at canvas-authoring time.

## Sequencing

- **Depends on PR #2736 merge** (CIWorkflowDag carrier introduction by neat-badger-30) — currently MERGEABLE, awaiting review tally. WI-2 implementation either holds for PR #2736 merge OR rebases on `session/neat-badger-30` branch.
- `dsl/extdeps/github/actions.dag::Workflow` already present — no upstream dependency for codomain type.
- Slice 1 substrate LANDED (PR #2160) — `WorkflowSecret` + `CronSchedule` available; consumed downstream by Slice 4 emitter, not this WI.
- Slice 3 demo LANDED (PR #2371) — `t_ci_workflow_as_data_demo.dag` exists; this WI extends scope from demo-of-evaluation into projection-substrate-declaration.
- This WI lands the projection-function substrate; Slice 4 implements per-arm bodies; Slice 8 dissolves hand-authority over `.github/workflows/ci.yml` (per renamed gate `ci_yml_hand_authority_dissolved` — see scope doc §1 fix 2026-05-12).

## Propagated substrate-fidelity concerns (Slice 4-5 — Substrate Mgr canvas)

These concerns are NOT acceptance gates for THIS WI (declaration-only scope). They are propagated to Slice 4-5 canvas as MUST-address-before-per-arm-body-PRs.

Authority: briansrls BLOCKING inline reviews on PR #2744 (2026-05-12T06:58:55Z) + codex BLOCKING scheduled review on cc82ec4c (2026-05-12T~07:14Z) flagged the audit as incomplete and requested "key-by-key inventory and STOP/reroute missing carriers before WI-2." This section provides the exhaustive top-level inventory.

### Exhaustive top-level inventory (current ci.yml → actions.dag carriers)

Verified 2026-05-12 against `.github/workflows/ci.yml` HEAD + `dsl/extdeps/github/actions.dag` HEAD:

| ci.yml field | actions.dag carrier | Status |
|---|---|---|
| `name: ci` | `Workflow.name: String` | ✓ covered |
| `on.push.branches: [main]` | `Push { branches: List<String>, paths: List<String> }` | ⚠ `Push.paths` REQUIRED in carrier; ci.yml omits — needs Optional |
| `on.pull_request.branches: [main]` | `PullRequest { branches, types }` | ✓ covered |
| `on.pull_request.types: [opened, synchronize, reopened, ready_for_review]` | `PullRequestActivity = Opened \| Synchronize \| Reopened \| Closed` | ⚠ **MISSING `ReadyForReview` arm** |
| `permissions.contents: read` | `WorkflowPermissions.contents: PermissionLevel` | ✓ covered |
| `permissions.pull-requests: read` | `WorkflowPermissions.pull_requests: PermissionLevel` | ✓ covered |
| (ci.yml omits `permissions.issues`) | `WorkflowPermissions.issues: PermissionLevel` REQUIRED | ⚠ field required; ci.yml omits — needs Optional OR `PermUnset` arm |
| (ci.yml omits `permissions.actions`) | `WorkflowPermissions.actions: PermissionLevel` REQUIRED | ⚠ same as issues |
| `concurrency.group: <expr>` | NO `Workflow.concurrency` field — only `Job.concurrency: ConcurrencySpec?` | ⚠ **MISSING `Workflow.concurrency`** |
| `concurrency.cancel-in-progress: true` | `ConcurrencySpec.cancel_in_progress: Bool` | (depends on adding `Workflow.concurrency`) |
| `env.CARGO_TERM_COLOR: always` | `Workflow.env: Map<String, String>` | ✓ covered |
| `env.RUSTFLAGS: -D warnings` | `Workflow.env: Map<String, String>` | ✓ covered |

### Per-job inventory (representative — `fmt` job from ci.yml)

| ci.yml field | actions.dag carrier | Status |
|---|---|---|
| `if: github.event.pull_request.draft != true` | `Job.if_condition: String?` (opaque expression string) | ✓ covered |
| `runs-on: ${{ vars.CI_RUNNER \|\| 'ubuntu-latest' }}` | `Job.runner: RunnerSpec = HostedRunner { RunnerLabel } \| SelfHosted { labels }` | ⚠ **EXPRESSION-SYNTAX GAP** — `RunnerLabel` is enum literal only; cannot represent GH Actions expression `${{ vars.X \|\| 'fallback' }}` |
| `timeout-minutes: 5` | `Job.timeout_minutes: Int?` | ✓ covered |
| `steps: [...]` | `Job.steps: List<Step>` | covered (per-step shape ratified) |

### Per-step inventory (representative — `actions/checkout@v4` step from ci.yml)

| ci.yml field | actions.dag carrier | Status |
|---|---|---|
| `name: Checkout` | `UsesStep.name: String?` | ✓ covered |
| `uses: actions/checkout@v4` | `UsesStep.uses: ActionRef { owner, repo, ref }` | ✓ covered |
| `with: <map>` | `UsesStep.with: Map<String, String>` | ✓ covered |

### 5 substantive carrier gaps requiring Slice 4 substrate-prereq PRs (cited in propagated concerns below)

1. **`Workflow.concurrency` absence**: ci.yml uses top-level `concurrency:`; `dsl/extdeps/github/actions.dag::Workflow` does NOT model `concurrency` (only `Job.concurrency` exists). P1/P2 fidelity requires extension (`Workflow.concurrency: ConcurrencySpec?`).
2. **`PullRequestActivity.ReadyForReview` arm absence**: ci.yml uses `types: [..., ready_for_review]`; carrier only has `Opened | Synchronize | Reopened | Closed`. Audit + extend.
3. **`Push.paths` required-but-omitted**: carrier requires both `branches` and `paths`; ci.yml only declares `branches`. Make `paths` optional with default empty list OR document the existing-default semantic.
4. **`WorkflowPermissions.issues/.actions` required-but-omitted**: carrier requires all 4 permission fields; ci.yml only declares `contents` + `pull-requests`. Either (a) make fields Optional, OR (b) introduce `PermUnset` arm to represent "no permission scope set" structurally.
5. **`RunnerSpec` expression-syntax modeling**: ci.yml uses `runs-on: ${{ vars.CI_RUNNER || 'ubuntu-latest' }}` — a GH Actions expression with fallback. Current `RunnerLabel` is enum literal only; cannot represent expression sites. Decide: (a) generalize `RunnerSpec` to allow opaque expression strings (parallel to `if_condition: String?`), (b) introduce expression-AST carrier covering all expression sites (`concurrency.group`, `if_condition`, `with` values, `env` values, `runner`), or (c) declare expression-string opaque for runner only as minimal-surface fix. Choice IS substrate-shape — Mgr canvas-tier decision.

### Carry-forward authoring concerns (Slice 4 acceptance discipline)

6. **Trigger fidelity — NO fabrication**: current ci.yml declares only `push` and `pull_request` triggers — NOT `schedule`. Slice 4 YamlStatic emitter MUST emit faithfully (push + PR only); `Schedule` trigger arm exists in the carrier but is NOT instantiated for current ci.yml. **No fabrication of triggers absent from source.**
7. **Step body + action input completeness as MUST (not SHOULD / NICE-TO-HAVE)**: Slice 4 ci.yml-equivalent regeneration requires executable step bodies (`RunStep.run`, `RunStep.shell`, `RunStep.env`) and complete action references (`UsesStep.uses: ActionRef`, `UsesStep.with: Map<String, String>`) as MUST acceptance. P1 modeling faithfulness fails if these are partial.
8. **Exhaustive inventory must extend per-job and per-step coverage**: this brief covers top-level + ONE representative job (`fmt`) + ONE representative step. Slice 4 canvas MUST inventory EVERY job (`fmt`, `ci`, `changes`, `v3`, `self_host_ratchet`, etc.) and EVERY step within each. STOP/reroute discipline applies per-job-per-step.

These concerns will be encoded into the Slice 4 brief as MUST-address acceptance gates when Substrate Mgr (warm-wolf-698) authors post-canvas-merge.

---

— Authored by deep-wolf-155 (PM) 2026-05-12 per operator FULL elevation directive + Director ratification msg_5cbdad24 + (c-refined) ratification msg_237bde05 / msg_f9fd669e; revised after codex BLOCKING #9970 on PR #2749 (INVARIANTS P1: WorkflowRuntime belongs in gunbc-substrate projection function, not as field on actions.dag::Workflow).
