# Design: Workflow Credentialing and Interactive Acquisition

## Summary

Credentialing should be modeled as a workflow, not as a static lookup plus a
manual instruction to the user.

Today the system is good enough to prove the basic path works:

- `gunbc.tools.gist` asks for a GitHub token
- `extdeps.github.auth::github_token()` fetches that token from GCP Secret
  Manager
- local GCP auth can succeed via `gcloud auth login --update-adc`

But the dev/user workflow is still wrong. When local auth is missing or stale,
the program errors, tells the user to run a command, and expects a second
invocation. The program should own that step. If the workflow is interactive
and policy allows it, the command should be run by the program as part of
credential acquisition.

The right mental model is:

- services declare credential intent
- the auth layer resolves that intent
- workflow/domain context selects the acquisition path
- local interactive login is an upsert/repair of auth state, not an external
  TODO for the user

## Gap Analysis

Before changing behavior further, the important thing is to separate what
already exists from what is still missing.

### What Already Exists

- `gunbc` now has its own auth namespace:
  `dsl/gunbc/auth/credentials.dag` and `dsl/gunbc/auth/patterns.dag`
- service-specific auth is still factored separately:
  `dsl/extdeps/github/auth.dag`
- the local GCP auth primitives already exist:
  `gcloud.Auth.Login`, `shell.GCloud.AuthPrintAccessToken`,
  `shell.GCloud.SecretManagerAccessVersion`
- the system already has a provider-neutral credential-intent concept on the
  Rust side in `src/00_foundation/ir/src/transport/scope.rs`
- `gunbc` already has some domain/runtime vocabulary in
  `dsl/gunbc/workflow/types.dag`

So the repo is not starting from zero. The missing piece is not "auth support
exists or does not exist". The missing piece is that the existing parts are not
yet joined into one workflow-owned credential lifecycle.

### What Is Missing

- no single `gunbc` auth workflow/resolver that takes intent plus context and
  chooses the acquisition path
- no explicit environment-tier model for credential selection
- no explicit separation between:
  environment tier,
  execution surface,
  and interaction policy
- no first-class mapping from environment tier to:
  GCP project,
  secret namespace,
  and allowed acquisition methods
- no same-invocation repair flow for local interactive auth
- no first-class terminal progress interrupt for:
  pause animated progress,
  show "Action Required",
  run the interactive repair step,
  then continue in the same invocation
- no clean guardrail preventing local/dev acquisition behavior from running in
  prod-like contexts

### What This Means for the First Increment

The first increment should be deliberately narrower than the full design:

- implement only the `dev` workflow first
- keep the source of truth in Secret Manager
- allow interactive `gcloud` repair only for explicitly local/dev contexts
- do not solve prod or CI policy in the same change

That gets the user workflow right without pretending the full environment model
already exists.

## Current State

Current seams:

- `dsl/gunbc/tools/gist.dag`
  `gist`, `gist_diff`, and `gist_recent` call `github_token()`
- `dsl/extdeps/github/auth.dag`
  `github_token()` currently delegates straight to
  `gunbc.auth.credentials::gcp_secret_credential(...)`
- `dsl/gunbc/auth/credentials.dag`
  `gcp_secret_credential(...)` reads from Secret Manager through
  `shell.GCloud.SecretManagerAccessVersion`
- `dsl/gunbc/auth/patterns.dag`
  `local_auth()` can read ADC / refresh / print an access token, but it does
  not yet model "repair local auth by running login and continue"
- `dsl/extdeps/cloud/gcp/gcp.dag`
  already defines `gcloud.Auth.Login` and `shell.GCloud.AuthPrintAccessToken`
- `src/00_foundation/ir/src/transport/scope.rs`
  already has a useful provider-neutral `CredentialIntent`

What is missing is the connection between these pieces.

One important clarification: this PR established a `gunbc.auth` home for these
concepts, but it did not yet establish the final auth workflow model. What we
have now is a better namespace split plus some concrete acquisition bindings.
What we still need is the resolver/policy layer.

## Problem

We currently collapse three different concerns into one ad hoc path:

1. What credential does a service need?
2. What runtime/workflow are we in?
3. What acquisition steps are allowed for that workflow?

That causes the wrong behavior for local dev:

- `gist` needs a GitHub credential
- the GitHub credential is stored in GCP Secret Manager
- accessing Secret Manager requires local GCP auth
- if local GCP auth is absent, the system should repair it
- instead, the user is told to run a command manually and try again

This is structurally the same kind of problem as other resource acquisition
problems. We do not want "missing local auth" to live as a free-floating user
instruction. We want it modeled as a first-class acquisition workflow.

## Design Goals

- Service DAGs should declare credential needs, not local login mechanics.
- Local interactive workflows should be single-invocation when possible.
- Non-interactive workflows must fail closed and never launch interactive auth.
- The auth layer should own retries after acquisition/repair, not require the
  caller to rerun.
- Secret Manager lookup, token refresh, and local login should compose as one
  acquisition pipeline.
- The same model should work for dev/user workflows now and CI/agent workflows
  later.
- Interactive repair should have an explicit terminal UX:
  pause progress,
  render an attention/info box,
  run the repair,
  then resume or complete with clear status.

## Non-Goals

- This design does not try to solve every cloud/provider at once.
- This design does not require removing Secret Manager as the source of truth.
- This design does not require collapsing all auth concerns into `std/`.
- This design does not require a perfect final UX in the first step.

## Core Model

### 1. Services Declare Intent, Not Acquisition Mechanics

Service-facing modules should describe the credential they need:

- provider
- service
- scheme/header
- required scopes
- secret binding name, when relevant

This is already close to the Rust-side `CredentialIntent` in
`src/00_foundation/ir/src/transport/scope.rs`.

For example, the `gist` path should conceptually say:

- I need a GitHub API credential
- for gist creation
- using bearer auth
- with `gist` write scope

It should not directly decide:

- whether we are in local dev, CI, or fleet
- whether `gcloud auth login` is allowed
- whether we should refresh ADC, read metadata, use WIF, or fail

### 2. Workflow Context Selects the Acquisition Path

Credential resolution should take a workflow context, not just an intent.

Proposed context shape:

```dag
type InteractionMode
  = Interactive
  | NonInteractive

type ExecutionSurface
  = UserCli
  | LocalAgent
  | FleetWorker

type AuthWorkflowContext {
  environment_tier: EnvironmentTier
  runtime: CloudRuntime
  interaction_mode: InteractionMode
  execution_surface: ExecutionSurface
  dry_run: Bool
}
```

This context answers the question:

"Given this credential need, what acquisition workflow is legal here?"

Examples:

- local CLI + interactive:
  login repair is allowed
- local CLI + dry-run:
  login repair is described, not executed
- CI / fleet:
  login repair is forbidden; use existing non-interactive credentials or fail

The key point is that these are separate axes.

- `environment_tier` answers:
  which secrets/project/bindings should this workflow use?
- `execution_surface` answers:
  am I a human-driven CLI, CI runner, local agent, or fleet worker?
- `interaction_mode` answers:
  may I launch interactive repair?

We should not collapse those into one boolean.

### 2a. Environment Separation Must Be Explicit

For now, the practical environment progression should be:

```dag
type EnvironmentTier
  = Local
  | Test
  | Dev
  | Prod
```

And CI should be modeled as an execution surface, not as a replacement for
environment tier:

```dag
type ExecutionSurface
  = UserCli
  | Ci
  | LocalAgent
  | FleetWorker
```

That lets us represent cases like:

- local CLI against `Dev`
- CI against `Test`
- fleet worker against `Prod`

This is cleaner than treating "CI" as its own environment, because CI can run
against multiple actual environments.

### 2b. Fermi Can Help, But Should Not Be the Primary Boundary

Using a Fermi-like classification as an additional safety/risk gate is
reasonable, but it should not be the sole prod/non-prod separator.

`FermiDepth` in this repo is currently a magnitude/cost vocabulary. It is good
for expressing things like:

- how expensive a workflow is
- how cautious the system should be about mutation
- whether an auth acquisition path has "small local impact" vs "large prod
  impact"

But it is too indirect to be the primary environment discriminator. The
security boundary should be explicit:

- `environment_tier: Prod` means prod
- `environment_tier: Dev` means dev

If we want an additional safety gate, we can layer one on top, for example:

```dag
type AuthRiskClass
  = LocalOnly
  | NonProd
  | Prod
```

or a Fermi-derived budget rule. But the environment split itself should stay
named and concrete.

### 3. Interactive Login Is an Upsert of Local Auth State

For local dev, `gcloud auth login --update-adc` is not an out-of-band setup
instruction. It is a mutating acquisition step on local auth state.

That means it should be modeled like an upsert/repair:

- if local auth state is present and valid, no-op
- if it is present but stale, repair
- if it is missing and interaction is allowed, create it
- if it is missing and interaction is not allowed, fail with a clear error

In other words:

- `print-access-token` is a read
- `auth login --update-adc` is a write/upsert
- the overall workflow is `ensure_local_gcloud_auth`

The caller should not need to rerun after that upsert. The workflow should
continue in the same invocation.

### 4. Auth Is a Layered Workflow

For a local `gist` invocation, the credential chain is actually:

1. Need GitHub credential for gist creation
2. GitHub credential source-of-truth is GCP Secret Manager
3. Secret Manager access needs GCP auth
4. Local GCP auth may require repair/upsert
5. Once GCP auth is present, read secret
6. Construct GitHub credential
7. Execute GitHub call

This should be explicit in the auth layer, not smeared across service wrappers
and error hints.

## Proposed Architecture

### A. Keep Service Definitions Thin

`extdeps.github.auth` should keep service-specific knowledge:

- secret binding name
- scopes
- auth scheme
- provider/service identity

But it should stop hardcoding a single acquisition mechanism such as
`gcp_secret_credential(...)`.

Instead, it should construct a credential intent and delegate to a workflow
resolver in `gunbc.auth`.

### B. Introduce an Auth Resolver at the Gunbc Layer

Add a gunbc-specific resolver along these lines:

```dag
func resolve_credential(
  intent: CredentialIntent,
  ctx: AuthWorkflowContext
) -> { credential: Credential }
```

Responsibilities:

- choose local-dev vs headless vs metadata/WIF path
- decide whether interactive repair is allowed
- ensure prerequisite auth state exists
- fetch source-of-truth secret/token
- return a transport-usable credential

### C. Split "Ensure Access to Secret Store" from "Read Secret"

Today `gcp_secret_credential(...)` effectively assumes Secret Manager access
already works.

We should split that into two ideas:

- `ensure_gcp_secret_store_access(ctx)`:
  acquire/repair the auth needed to talk to Secret Manager
- `read_secret_manager_secret(...)`:
  read the secret once access exists

This is especially important because the dev/user problem is not "reading the
GitHub token"; it is "ensuring local GCP auth exists so the token can be read".

### C1. Gunbc Must Own the Binding from Environment to Secret Location

For the first real workflow, `gunbc` should explicitly bind:

- environment tier
- GCP project ID
- secret name / secret namespace
- allowed acquisition path

For example, conceptually:

```dag
type AuthBinding {
  environment_tier: EnvironmentTier
  provider: String
  service: String
  project_id: ProjectId
  secret_name: NonEmptyStr
  interactive_login_allowed: Bool
}
```

This binding is gunbc-specific policy. It does not belong in `std/`, and it
does not belong in a generic extdep module.

For now, the expected shape is something like:

- `Local`:
  local/dev-like secret binding, interactive repair allowed
- `Test`:
  isolated non-prod binding, interactive repair usually disallowed except in
  explicit local test contexts
- `Dev`:
  shared dev binding, typically non-interactive in automation
- `Prod`:
  prod binding, never interactive

The exact project/secret naming can vary, but the policy mapping should be
explicitly modeled by gunbc rather than inferred ad hoc from secret strings.

### D. Model Local GCP Auth as a Resource

We already have `AuthContext` as a resource shape. Local auth repair should be
modeled as acquisition of a more specific auth resource, conceptually something
like:

```dag
resource LocalGcpAuthState {
  account: String?
  adc_path: FilePath
  valid: Bool
}
```

The exact type can change, but the important point is that local auth becomes a
managed resource with read/repair semantics, not a hidden ambient assumption.

### E. Treat Interactive Auth Repair as a Progress Interrupt

`gunbc` already has the Rust-side progress renderer and box primitives, but the
credential workflow should make the interactive seam explicit rather than
surfacing a generic error that tells the user to rerun manually.

The target behavior is closer to the old `gunb.ai` flow:

- run normal preflight/acquisition work under progress
- if interactive repair is needed, pause the animated progress display
- render a clear boxed prompt such as `Action Required`
- run the login/upsert step from the program
- resume or complete the workflow in the same invocation

This is important enough to treat as part of the feature contract, not as
optional polish.

## Proposed Local Dev Flow

### `gist` in interactive local CLI

1. `gunbc.tools.gist` asks for GitHub credential intent
2. `gunbc.auth.resolve_credential(...)` sees:
   - runtime: `LocalDev`
   - interaction: `Interactive`
   - source of truth: GCP Secret Manager
3. Resolver tries to ensure GCP auth:
   - read ADC if present
   - refresh or print access token if possible
4. If that fails and interactive repair is allowed:
   - run `gcloud auth login --update-adc`
   - re-read / re-validate auth state
5. Use the repaired auth state to read the GitHub token from Secret Manager
6. Return the GitHub credential to the gist service
7. Continue the gist workflow in the same invocation

### `gist` in non-interactive CI/fleet

1. Same GitHub credential intent
2. Resolver sees:
   - runtime: non-local or non-interactive
   - interactive repair forbidden
3. Use metadata / WIF / existing non-interactive credential chain
4. If unavailable, fail closed with a clear diagnostic
5. Never launch browser login

## Recommended Environment Model

For the near term, use this matrix:

| Environment tier | Typical purpose | Interactive repair | Secret binding owner |
|---|---|---|---|
| `Local` | developer-owned local workflows | allowed | gunbc local policy |
| `Test` | isolated test/sandbox env | usually no | gunbc test policy |
| `Dev` | shared development env | no for automation | gunbc dev policy |
| `Prod` | production env | never | gunbc prod policy |

And combine it with:

- `ExecutionSurface = UserCli | Ci | LocalAgent | FleetWorker`
- `InteractionMode = Interactive | NonInteractive`

Example interpretations:

- `Local + UserCli + Interactive`
  the program may run `gcloud auth login --update-adc`
- `Test + Ci + NonInteractive`
  use non-interactive credentials only
- `Dev + UserCli + Interactive`
  likely okay for early rollout, but should still use the explicit dev secret
  binding
- `Prod + FleetWorker + NonInteractive`
  never use local login repair; only workload/service credentials

## First Target: Dev Workflow

The first concrete workflow should be the `dev` path, not the whole matrix.

That means:

- `gunbc` owns one explicit `Dev` auth binding for GitHub
- the binding names the Secret Manager location and secret name
- the `Dev` workflow can be invoked from a local interactive surface
- if local GCP auth is missing, the program repairs it and continues
- the same workflow must fail closed when invoked from non-interactive
  automation

This gives us a useful real workflow without prematurely generalizing prod.

## Why This Is Better

- It matches how users think:
  "I ran `make gist`; the tool should do what it needs to do."
- It localizes auth policy in the auth/workflow layer instead of leaking it
  into service wrappers.
- It gives us one place to express local-dev vs CI differences.
- It keeps interactive behavior explicit and policy-driven.
- It lets us reuse the same model for more than gist.

## Concrete Changes

### Phase 1: Fix the Local Dev Path

- add a local-dev auth-upsert pattern in `gunbc.auth`
- call `gcloud.Auth.Login(update_adc: true)` from the program when policy
  allows it
- retry acquisition inside the same invocation
- route `extdeps.github.auth::github_token()` through that resolver path
- add an explicit progress-display interrupt/prompt/resume path for interactive
  repair, rather than erroring and asking the user to rerun

This phase can remain GCP/Secret-Manager-specific.

### Phase 1 Test Requirements

- hermetic integration test:
  missing local auth triggers the interactive repair path and the overall
  command succeeds in one invocation
- progress UX test:
  the display pauses and emits an explicit action-required/info box for the
  interactive step
- policy test:
  the same workflow in non-interactive mode fails closed and does not launch
  interactive login
- environment test:
  local/dev behavior cannot silently run when the workflow context is `Prod`

These should be treated as feature gates, not follow-up cleanup.

### Phase 2: Make Intent + Context First-Class

- define a DSL-side `AuthWorkflowContext`
- align service auth wrappers with the provider-neutral `CredentialIntent`
  concept already present in Rust
- make service wrappers produce intent and delegate to a common resolver

### Phase 3: Generalize Across Workflows

- use the same resolver for other dev/user tools
- teach planner/executor/domain layers to provide context explicitly
- unify dry-run behavior so interactive auth is previewed, not executed

## Important Constraints

- Interactive acquisition must be opt-in via workflow context.
- Dry-run must not mutate local auth state.
- The resolver must own retry-after-acquire behavior.
- Services must remain ignorant of local repair mechanics.
- Headless workflows must never silently fall back to interactive login.

## Open Questions

- Should the primary local repair step be `gcloud auth login --update-adc`,
  `gcloud auth application-default login`, or a policy-selected command?
- Should local auth state be modeled as a dedicated resource, or remain an
  auth-specific pattern with resource-like semantics?
- Should `CredentialIntent.interactive_allowed` remain on the intent, or be
  derived solely from workflow context plus policy?
- Do we want one resolver that returns final `Credential`, or a two-step model:
  `resolve auth context` then `materialize service credential`?

## Recommendation

Advance in this order:

1. treat local GCP login as a program-owned upsert/repair step
2. keep `gist` and other services thin by moving path selection into
   `gunbc.auth`
3. make workflow context explicit so user/dev vs CI behavior is chosen by
   domain policy, not ad hoc conditionals

That gets the user-facing behavior right first, while still moving toward the
more principled model: services declare intent, auth resolves it, and workflow
context chooses the path.
