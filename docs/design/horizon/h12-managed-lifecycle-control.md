# H12 Design: Managed Lifecycle Control

## Problem

gunbc already models **execution lifecycle** well:

- acquire
- use
- release

That is the lifecycle of a handle **within a run**.

It does **not** yet model **managed existence lifecycle** well:

- ensure something is present and serving,
- disable new ingress,
- drain in-flight work,
- destroy it,
- verify it is absent.

This gap matters for production operation. Today, the repo has:

- `upsert` and `content_upsert` for presence/creation patterns,
- deployment DAGs that create/update infrastructure,
- SDLC design contracts that mention drain mode,
- ad-hoc rollback/removal in a few handwritten places.

What it does **not** have is a first-class language/compiler contract that says:

> if a workflow can turn a managed thing on, it must be able to turn it off in a
> structured, testable, code-generated way.

Without that, lifecycle control becomes ad-hoc:

- graceful shutdown lives in prose,
- destruction lives in shell commands or handwritten cleanup code,
- codegen only knows how to upsert,
- tests prove creation but not teardown,
- "turn off this section of the pipeline" becomes operational folklore.

## Decision

Introduce a **managed lifecycle** model at the language/compiler level that is
distinct from run-scope acquire/release.

The system should support two destruction paths:

1. **Graceful**: `disable -> drain -> destroy -> verify_absent`
2. **Brutal**: `destroy -> verify_absent`

This is **not** a global workflow stage machine. It is a local lifecycle contract
for managed units.

## Core Distinction

### Execution lifecycle

Existing model:

```text
acquire -> use -> release
```

This is about capabilities or handles inside one execution.

Examples:

- filesystem handle
- auth token
- claim lease
- network capability

### Managed existence lifecycle

New model:

```text
ensure_present
disable
drain
destroy
verify_absent
```

This is about whether a managed thing exists and is actively participating in the
system across runs.

Examples:

- worker service
- webhook ingress
- signal subscription
- queue consumer
- deployment lane
- generated artifact set

These are different concerns and should not be collapsed.

## Managed Unit Model

The smallest useful abstraction is a **managed unit**: anything the system can
bring into service and take out of service.

Candidate unit classes:

- ingress units: webhook handlers, schedulers, signal publishers
- processing units: worker fleets, reconcilers, agents
- storage units: buckets, ledgers, marker stores
- generated artifact sets: codegen outputs, rendered configs
- whole workflow sections: a stage or lane that can be disabled independently

## Lifecycle Surface

### Required lifecycle verbs

Every managed unit must declare which of these it supports:

- `ensure_present`
- `disable`
- `drain`
- `destroy`
- `verify_present`
- `verify_absent`

Not every unit needs every verb, but the declaration must be explicit.

### Destroy modes

```text
DestroyMode =
  Graceful
  Brutal
```

Semantics:

- `Graceful` means no new ingress, no new claims, in-flight work quiesced first.
- `Brutal` means immediate teardown is allowed and explicitly requested.

`Brutal` must never be the default.

## Proposed Language Direction

Current `resource` declarations are run-scope:

```text
resource X {
  acquire {}
  release {}
}
```

For managed units, extend the model with a second lifecycle surface.

This is now the chosen direction for the DSL:

```text
resource WorkerService {
  kind: Capability
  mode: Exclusive

  acquire {}
  release {}

  managed {
    destroy_support: GracefulOnly | GracefulAndBrutal | BrutalOnly | Unsupported

    ensure_present {}
    verify_present {}

    disable {}
    drain {}

    destroy {}
    verify_absent {}
  }
}
```

Semantic rules:

1. run-scope lifecycle remains separate,
2. managed lifecycle is explicit,
3. the compiler can generate lifecycle workflows and tests from it.

Validation rules:

1. `ensure_present` requires `verify_present`.
2. Any destroy-support mode other than `Unsupported` requires `destroy` and
   `verify_absent`.
3. `GracefulOnly` and `GracefulAndBrutal` require both `disable` and `drain`.
4. `Brutal` destroy is generated only when `destroy_support` explicitly allows it.
5. `destroy_support` defaults to `Unsupported`; destroy is never inferred from
   unrelated verbs.

Transitional rule:

1. existing infra resources may temporarily keep ensure logic in `acquire`,
2. the canonical destination is to move long-lived lifecycle behavior into
   `managed`,
3. `acquire`/`release` remain the handle lifecycle, not the long-lived service
   lifecycle.

## Canonical Patterns

### 1. Ensure-present

The current `upsert` family already covers most of this:

```text
check -> create/update -> resolve
```

### 2. Ensure-absent

The missing opposite of upsert:

```text
check_present -> destroy_if_present -> verify_absent
```

This should become a first-class pattern, not ad-hoc delete logic.

### 3. Graceful shutdown

The safe multi-step form:

```text
check_present -> disable -> drain -> destroy -> verify_absent
```

### 4. Brutal shutdown

Explicit, auditable fast path:

```text
check_present -> destroy -> verify_absent
```

## Compiler Responsibilities

If managed lifecycle is first-class, the compiler/codegen should own:

1. lifecycle command generation
2. lifecycle validation
3. lifecycle test obligations
4. lifecycle reporting/audit artifacts

Concrete implications:

- generated CLIs should expose `ensure`, `disable`, `drain`, `destroy`, `status`
- lifecycle verbs must lower as explicit graph structure, not opaque metadata or
  handwritten side channels
- missing `verify_absent` for destructive flows is a modeling error
- `Brutal` destroy paths require explicit invocation and distinct audit output
- lifecycle operations for a workflow section should be composable from its units

## SDLC Consequences

For SDLC, this means the following should be separately controllable:

- intake
- webhook ingress
- signal ingress
- worker claim acquisition
- reconciler
- selected stages or lanes
- cloud infrastructure backing the system

The design contract in the SDLC docs already says workers must respect durable
drain mode. This doc raises that from prose to language/compiler intent.

## Resolved Unit Granularity

The runtime disable target is a **managed unit**, not raw config.

Config still matters, but only for:

1. default desired state,
2. instantiation parameters,
3. whether a controllable unit exists at all.

The act of disabling, draining, or destroying is always targeted at a managed unit
with a stable unit id.

Canonical SDLC managed-unit classes:

1. `IngressUnit`
2. `SignalIngressUnit`
3. `WorkerFleet`
4. `ReconcilerUnit`
5. `StageGate`
6. `LaneGate`
7. `StoreUnit`
8. `ArtifactSet`

For workflow sections:

1. a named stage or lane is represented as a `StageGate` or `LaneGate`,
2. the compiler should synthesize stable gate ids from workflow/stage names,
3. generated lifecycle commands can target those gate ids directly.

This resolves the "arbitrary section of the pipeline" question: the section is not
just a config flag. It is a managed gate with lifecycle.

### Disable semantics for stage and lane gates

Disabling a gate means:

1. no new work may enter that stage or lane,
2. in-flight work already inside the gate may finish unless brutal destroy is
   requested,
3. transitions targeting the disabled gate must park durably rather than fail
   silently.

Operational consequence:

1. workers must not claim new work for disabled gates,
2. transitions into disabled sections should record a non-terminal
   `BlockedByDisable` result and emit a recheck/reconcile signal,
3. claims, ledgers, and artifact markers must remain intact while the gate is
   disabled.

## Invariants

1. A managed unit that can be ensured present must explicitly declare whether it
   supports graceful destroy, brutal destroy, or neither.
2. Graceful destroy is modeled as structured disable/drain/destroy, not as a
   handwritten convention.
3. Destroy is not complete until `verify_absent` succeeds or fails closed.
4. `Brutal` destroy must be explicit in the invocation and machine-readable in
   the execution report.
5. Disabling one section of a pipeline must not silently corrupt claims, ledgers,
   or audit markers owned by another section.
6. Code generation is incomplete if it can emit create/apply commands but not the
   corresponding disable/drain/destroy commands for the same managed unit class.
7. Runtime disable targets are managed units with stable ids, not anonymous config
   toggles.

## Testing Obligations

Every managed lifecycle declaration should derive tests for:

1. ensure-present idempotency
2. disable idempotency
3. drain quiescence behavior
4. graceful destroy path
5. brutal destroy path
6. verify-absent correctness
7. re-enable after graceful disable, when supported

This is where the language-level requirement matters: lifecycle support should not
be "documented but untested." It should generate proof obligations the same way
other contracts do.

## Migration Plan

1. Introduce a design-level distinction between execution lifecycle and managed
   lifecycle.
2. Add a first-class `ensure_absent` pattern to the DSL.
3. Define a minimal managed-lifecycle declaration surface for resources or
   workflow units.
4. Teach lower/codegen/testgen to derive lifecycle graphs and tests.
5. Migrate SDLC ingress/worker/reconciler/deploy units to the new surface.
6. Remove handwritten cleanup/drain paths once lifecycle generation is complete.

## Follow-up Implementation Tasks

- `H12.1` Define managed lifecycle vocabulary in the DSL design (`ensure_present`, `disable`, `drain`, `destroy`, `verify_absent`).
- `H12.2` Add a first-class `ensure_absent` pattern to `dsl/std/patterns.dag`.
- `H12.3` Extend lower/IR to preserve managed lifecycle semantics as explicit nodes or metadata with validation.
- `H12.4` Generate lifecycle commands for codegen/infrastructure targets, not just upsert/apply commands.
- `H12.5` Add compiler/testgen obligations for graceful destroy, brutal destroy, and verify-absent.
- `H12.6` Model SDLC worker/webhook/reconciler shutdown using the managed lifecycle surface.
- `H12.7` Delete handwritten rollback/cleanup paths that become redundant once lifecycle generation is real.
