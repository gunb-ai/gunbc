# H7 Design: Resource Abstraction Trait for DAG-Native Management

## Problem

Resource handling (files, creds, network, locks) is inconsistent across ops. Capability checks and lifecycle behavior are scattered.

## Decision

Introduce a capability-oriented resource trait as the shared contract for acquisition, probe, and release.

## Proposed API

```rust
pub trait Resource {
    type Handle;

    fn id(&self) -> &str;
    fn capabilities(&self) -> &'static [ResourceCapability];
    fn acquire(&self, ctx: &ResourceContext) -> Result<Self::Handle, ResourceError>;
    fn probe(&self, ctx: &ResourceContext) -> Result<ResourceHealth, ResourceError>;
    fn release(&self, handle: Self::Handle, ctx: &ResourceContext) -> Result<(), ResourceError>;
}
```

## Capability Model

- `ReadFile`, `WriteFile`
- `ReadCredential`, `ImpersonateCredential`
- `NetworkEgress`
- `SharedLock`, `ExclusiveLock`

## Invariants

- All boundary/resource nodes declare required capabilities.
- DryRun cannot forge capabilities.
- Resource handles are opaque and typed (no stringly-typed downcasts).

## Migration Plan

1. Define trait + capability enums.
2. Implement adapters for current resource types.
3. Enforce capability checks in execution boundary paths.
4. Add probe and release observability in logs.

## Follow-up Implementation Tasks

- `H7.1` Add `Resource` trait and capability enums.
- `H7.2` Implement file/credential/network adapters.
- `H7.3` Add execution-time capability enforcement.
- `H7.4` Add probe/release reporting to execution log.
- `H7.5` Add negative tests for capability forgery attempts.
