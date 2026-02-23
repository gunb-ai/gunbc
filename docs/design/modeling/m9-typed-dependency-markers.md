# M9: Typed Dependency Markers

**Status**: Design
**Lane**: A (Graph Semantics)
**Depends on**: M8 (done)
**Blocks**: M16

## Problem

`SecretDependencyId` values carry string-prefixed conventions like
`"secret:GOOGLE_APPLICATION_CREDENTIALS"`. While `DependencyKind` is already a
typed enum (`System` / `Secret`), the inner ID values encode their kind
redundantly via string prefixes. If DAG materialization ever parses these
strings, the prefix convention would become load-bearing.

## Current State

```rust
pub enum DependencyKind {
    System(SystemDependencyId),  // .0 = "gcp.secret_manager"
    Secret(SecretDependencyId),  // .0 = "secret:GOOGLE_APPLICATION_CREDENTIALS"
}
```

Callers construct dependencies with the prefix baked in:
```rust
Dependency::secret("secret:GOOGLE_APPLICATION_CREDENTIALS")
```

The `validate_dependency_graph_acyclic()` function only inspects
`DependencyKind::System` edges and treats the inner ID as opaque.

## Design

### 1. Strip string-prefix convention from ID values

Change secret dependency IDs to contain only the bare env var name:

```rust
// Before:
Dependency::secret("secret:GOOGLE_APPLICATION_CREDENTIALS")
// After:
Dependency::secret("GOOGLE_APPLICATION_CREDENTIALS")
```

The `DependencyKind::Secret` variant already encodes that this is a secret
dependency. The `"secret:"` prefix in the ID is redundant.

### 2. Add typed constructors with validation

```rust
impl SecretDependencyId {
    pub fn env_var(name: &str) -> Self {
        debug_assert!(!name.starts_with("secret:"), "prefix is redundant");
        Self(name.to_string())
    }
}
```

### 3. Add round-trip tests

- Serialize → deserialize a SystemModel with both System and Secret deps
- Verify the dependency graph validates correctly
- Verify no string-prefix parsing exists in any walker

### 4. Migration

Update all call sites in `lib/gcp-ops/src/system_models.rs` and
`lib/aws-ops/src/system_models.rs` to strip the `"secret:"` prefix.
Update corresponding assertion tests.

## Files

- `core/ir/src/system_model.rs` — `SecretDependencyId` constructor
- `lib/gcp-ops/src/system_models.rs` — 3 dependency declarations + tests
- `lib/aws-ops/src/system_models.rs` — 6 dependency declarations + tests

## References

- `core/ir/src/system_model.rs:174-245` — DependencyKind, SystemDependencyId, SecretDependencyId
- `core/ir/src/system_model.rs:456-505` — validate_dependency_graph_acyclic
