# SDLC Execution Intent and Binding Plan

Status: Draft — Needs review
Date: 2026-03-05
Parent: [mega-modeling-design.md](mega-modeling-design.md)
Companion planning lane: [../../../tasks.md](../../../tasks.md)
Scope: Define the reusable composition-root and concrete binding/link model that replaces the temporary SDLC profile compatibility path and can be reused by future process families.

## 1. Document Contract

1. This document owns the design for `SM-1` and `SM-2` from `tasks.md` Phase H.
2. `tasks.md` remains the active planning surface and source of prioritization.
3. If `tasks.md` and this document diverge on `SM-1` / `SM-2`, update one or both immediately; they should not drift.

## 2. Problem

The branch currently proves local real-mode SDLC execution through a temporary
`dsl/profiles/sdlc.dag` compatibility path. That path is useful as an unblocker,
but it is not the target architecture.

The missing target design is not "profiles, but renamed." The missing target design is:

1. one thin composition root that says what is true about a run,
2. one reusable mechanism for binding abstract contracts to concrete implementations,
3. one reusable mechanism for linking implementation inputs to objective facts,
4. orthogonal fact models for topology, scope, credentials, authorities, triggers, and safety.

The design must be reusable beyond SDLC. The same modeling shape should be able to
serve infra flows, review flows, and future process families.

## 3. Design Rules

### 3.1 Reusable Facts First

Prefer reusable, objective facts over scenario enums.

Good:

- "runtime is co-located"
- "mutation is prohibited"
- "target repo is `gunb-ai/integration_testing`"
- "claim ledger authority is local dir `target/sdlc/outcomes`"

Bad:

- "`local_real` mode"
- "`cloud_prod` profile"
- scenario-specific branching embedded in runtime code

### 3.2 Thin Composition Root

If we introduce a top-level execution-intent concept, it must stay thin.

It composes reusable fact models. It must not become:

- a renamed SDLC-only profile system
- a bucket for raw secrets
- a place to inline provider-specific configuration
- a giant enum for local/cloud/dev/real combinations

### 3.3 Binding and Linking Are Separate

"What concrete implementation satisfies this contract?" and "where does that
implementation get each required input?" are different questions and must stay
separate in the model.

### 3.4 Deployment Split Is Not Business Logic

Co-located local execution, hosted single-worker execution, and hosted fleet
execution must preserve the same SDLC stage semantics. Deployment topology is a
modeled fact, not a reason to fork stage logic.

## 4. Reuse Existing Vocabulary

The design should reuse or evolve existing types where they already fit:

- `RuntimeProfile`
- `LaunchConfig`
- `InfraIntent`
- `CredentialIntent`
- `CredentialResolution`
- `CredentialBinding`
- `SignalType`
- `IssueBinding`
- `StageOutcome`

Do not introduce parallel local/cloud split types if `RuntimeProfile` /
`LaunchConfig` can be evolved into the final `ExecutionTopology` vocabulary.

## 5. Proposed Model

### 5.1 `ExecutionIntent`

`ExecutionIntent` is the thin composition root for a run, worker invocation, or
execution context.

```text
type BindingPlanId = String @brand("BindingPlanId") @non_empty

type ExecutionIntent {
  topology: ExecutionTopology
  effect_policy: EffectPolicy
  target_scope: TargetScope
  binding_plan_id: BindingPlanId
  trigger_policy: TriggerPolicy
  safety_policy: SafetyPolicy
}
```

Invariants:

1. `ExecutionIntent` does not directly contain provider-specific config.
2. `ExecutionIntent` does not directly contain raw secret values.
3. `ExecutionIntent` does not encode scenario labels such as `local_real`.
4. Changing only topology or authority backing must not change SDLC stage semantics.

### 5.2 `BindingPlan`

`BindingPlan` selects concrete implementations for abstract contracts and then
links implementation inputs to reusable facts.

```text
type ContractRef = String @brand("ContractRef") @non_empty
type ImplementationRef = String @brand("ImplementationRef") @non_empty
type AuthorityId = String @brand("AuthorityId") @non_empty

type BindingPlan {
  plan_id: BindingPlanId
  contract_bindings: List<ContractBinding>
  input_links: List<InputLink>
}

type ContractBinding {
  contract: ContractRef
  implementation: ImplementationRef
}

type InputLink {
  implementation: ImplementationRef
  input_name: NonEmptyStr
  source: LinkSource
}

type LinkSource
  = ScopeField { field_path: NonEmptyStr }
  | AuthorityRef { authority_id: AuthorityId }
  | CredentialIntentRef { intent: CredentialIntent }
  | PolicyField { field_path: NonEmptyStr }
  | Literal { value: Json }
```

Invariants:

1. `ContractBinding` decides implementation selection only.
2. `InputLink` decides input realization only.
3. `BindingPlan` points to objective facts; it does not inline scenario-specific shell logic.
4. Replacing `dsl/profiles/sdlc.dag` should require no handwritten runtime special case; only the selected `BindingPlan` changes.

## 6. Supporting Fact Models

### 6.1 `ExecutionTopology`

```text
type ExecutionTopology {
  runtime_profile: RuntimeProfile
  launch: LaunchConfig
  controller_shape: ControllerShape
}

type ControllerShape
  = CoLocatedLoop
  | WorkerOnly
  | WorkerAndReconciler
  | SplitControllers
```

Purpose:

- describe deployable/controller split
- reuse `RuntimeProfile` and `LaunchConfig`
- keep deployment topology separate from stage logic

### 6.2 `EffectPolicy`

```text
type EffectPolicy {
  effect_mode: EffectMode
  mutation_guard: MutationGuard
}

type EffectMode
  = Hermetic
  | ReadOnlyProbe
  | Mutating

type MutationGuard
  = MutationProhibited
  | ExplicitOptIn { flag_name: NonEmptyStr }
  | ScopedByPolicy
```

Purpose:

- model hermetic vs real effects without provider-specific branching
- make mutation permission explicit and auditable

### 6.3 `TargetScope`

```text
type TargetScope {
  repo_scope: RepoScope?
  cloud_scope: CloudScope?
  local_scope: LocalScope?
  namespace_tag: NonEmptyStr?
}

type RepoScope {
  owner: NonEmptyStr
  repo: NonEmptyStr
  issue_label_prefix: NonEmptyStr?
}

type CloudScope {
  project_id: NonEmptyStr
  region: NonEmptyStr?
}

type LocalScope {
  root_dir: FilePath
}
```

Purpose:

- carry bounded target facts
- keep "dev repo vs prod repo" in scope data, not in binding names

### 6.4 `TriggerPolicy`

```text
type TriggerPolicy {
  work_discovery: WorkDiscoveryPolicy
  reconcile: ReconcilePolicy
}

type WorkDiscoveryPolicy
  = ManualOnly
  | ScanOnly
  | SignalAccelerated

type ReconcilePolicy {
  periodic_tick_enabled: Bool
  tick_window_id: NonEmptyStr?
}
```

Purpose:

- model trigger source separately from stage semantics
- ensure signal-driven and scan-driven execution converge on the same authoritative state

### 6.5 `SafetyPolicy`

```text
type SafetyPolicy {
  preflight: PreflightPolicy
  drain: DrainPolicy
  observability: ObservabilityPolicy
}

type PreflightPolicy {
  require_credentials: Bool
  require_authorities_resolved: Bool
  require_non_prod_scope: Bool
}

type DrainPolicy
  = NoDrainSupport
  | FileFlagDrain
  | UrlFlagDrain

type ObservabilityPolicy {
  require_execution_report: Bool
  require_audit_entries: Bool
}
```

Purpose:

- model rollout safety as reusable contracts rather than ops folklore
- block remote real runs on missing modeled safety, not just on cautionary prose

### 6.6 `AuthorityFact`

```text
type AuthorityFact
  = RepoAuthority { authority_id: AuthorityId, owner: NonEmptyStr, repo: NonEmptyStr }
  | LocalDirAuthority { authority_id: AuthorityId, path: FilePath }
  | GcsBucketAuthority { authority_id: AuthorityId, project_id: NonEmptyStr, bucket: NonEmptyStr }
  | PubSubAuthority { authority_id: AuthorityId, project_id: NonEmptyStr, topic: NonEmptyStr, subscription: NonEmptyStr? }
  | SecretAuthority { authority_id: AuthorityId, project_id: NonEmptyStr, secret_id: NonEmptyStr }
```

Purpose:

- provide objective backing facts any process can reference
- keep binding plans from repeating raw provider config everywhere

## 7. Scenario Presets Are Derived, Not Fundamental

Practical scenarios still matter, but only as presets over the fact models above.

Examples:

- `local_dev_testing`
  - local co-located topology
  - hermetic effect policy
  - mocked/local authorities
  - manual trigger

- `local_real_testing`
  - local co-located topology
  - mutating effect policy with explicit opt-in
  - integration `TargetScope`
  - file-backed authorities
  - real credential realization

- `remote_dev_testing`
  - hosted topology
  - mutating effect policy
  - non-prod `TargetScope`
  - cloud authorities
  - strict `SafetyPolicy`

- `remote_real_runs`
  - hosted fleet topology
  - mutating effect policy
  - prod `TargetScope`
  - cloud authorities
  - strongest `SafetyPolicy`

Invariant:

1. these names are for docs, presets, fixtures, and operator shorthand;
2. executable truth remains the composed fact models.

## 8. Migration Plan

### 8.1 SM-1

1. Add the `ExecutionIntent` design vocabulary to docs/types.
2. Reuse `RuntimeProfile` / `LaunchConfig` as the seed for `ExecutionTopology`.
3. Keep the composition root thin; do not inline binding logic into it.

### 8.2 SM-2

1. Add `BindingPlan` and supporting ref/fact types.
2. Model SDLC local real testing as one explicit `BindingPlan`.
3. Move the SDLC live harness/compiler selection path from `profiles.sdlc.local` to that plan.
4. Delete the temporary profile compatibility module once the plan is active.

### 8.3 Follow-ons

1. `SM-3`: refine credential intent vs credential realization
2. `SM-4`: model backing authorities uniformly across local and hosted stores
3. `SM-5`: attach safety policy to execution intent / scope / authority facts
4. `SM-6`: keep the scenario/proof matrix aligned with real tests

## 9. Acceptance Criteria

`SM-1` is done when:

1. the composition-root type exists with separate topology/scope/effect/trigger/safety concerns
2. no giant scenario enum is introduced
3. the shape is usable by non-SDLC process families

`SM-2` is done when:

1. a reusable binding/link model exists
2. the SDLC active path can select concrete providers without `profiles.sdlc.local`
3. `AUTH-4` / `DM-3A` can close by deleting the temporary compatibility module

## 10. Open Naming Notes

Names are still provisional. The important constraints are structural:

- `ExecutionIntent` could become `RunIntent` or similar
- `BindingPlan` could become `ContractRealizationPlan` or similar
- `AuthorityFact` could become `AuthorityBinding` or similar if the team prefers

Do not bikeshed names before preserving the separation rules.
