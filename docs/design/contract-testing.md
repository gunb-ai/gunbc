# Contract Testing: Upgrading @contract to Real Proof Obligations

## Problem Statement

Interfaces define capabilities with typed I/O. Multiple providers implement
each interface (e.g., `StubIssueProvider`, `GitHubIssueProvider`). The
existing infrastructure verifies:

- **IS-\***: Modules compile without profiles via interface stub transport
- **PT-\***: Modules compile with each applicable profile (per-profile live tests)
- **Testgen**: DryRun completion, boundary interception, failure scenarios

**Missing**: No verification that a provider's behavior satisfies the
interface's semantic contract. Nothing ensures `StubIssueProvider` and
`GitHubIssueProvider` behave equivalently for the operations they both
claim to implement.

## Current State

`@contract` appears only in comments across the codebase — no actual
annotations are parsed or enforced. The DSL spec defines the syntax:

```dag
@contract: get(id) after create(...) => found
```

But this is aspirational — no implementation exists.

## Design: Three Phases

### Phase 1 — Contract IR

Add `@contract` annotation support to interface capability definitions:

```dag
interface ObjectStorage {
  capability put(key: String, value: Bytes) -> { ok: Bool }
    @contract: get(key) after put(key, value) => { found: true, value: value }

  capability get(key: String) -> { value: Bytes, found: Bool }
    @contract: get(key) => get(key)  // idempotent

  capability delete(key: String) -> { ok: Bool }
    @contract: get(key) after delete(key) => { found: false }
}
```

**Contract obligation types**:

| Type | Syntax | Meaning |
|------|--------|---------|
| Sequence | `B(args) after A(args) => expected` | After A, calling B produces expected |
| Idempotent | `A(args) => A(args)` | Calling A twice yields same result |
| Destructive | `B(args) after A(args) => expected` | A invalidates prior B results |
| Invariant | `A(args) => { field: constraint }` | Every call satisfies constraint |

**IR representation** (new type in `core/ir`):

```rust
pub struct ContractObligation {
    pub interface_name: String,
    pub capability_name: String,
    pub kind: ContractKind,
    pub setup: Vec<ContractStep>,    // operations to call before assertion
    pub assertion: ContractStep,      // operation to call + expected outputs
}

pub enum ContractKind {
    Sequence,      // A then B => expected
    Idempotent,    // A => A (same result)
    Destructive,   // A then B => different result
    Invariant,     // A => constraint (always)
}

pub struct ContractStep {
    pub capability: String,
    pub args: BTreeMap<String, Value>,
    pub expected: Option<BTreeMap<String, Value>>,
}
```

### Phase 2 — Contract Test Generation

For each interface with `@contract` annotations, testgen emits a
**parameterized contract compliance test suite**:

```rust
// Generated: contract_object_storage.rs
fn contract_suite(binding: ServiceBinding) {
    // From: @contract: get(key) after put(key, value) => { found: true }
    #[test]
    fn contract_get_after_put() {
        let put_result = binding.call("put", &[("key", "test"), ("value", b"data")]);
        assert!(put_result.ok);
        let get_result = binding.call("get", &[("key", "test")]);
        assert_eq!(get_result.found, true);
        assert_eq!(get_result.value, b"data");
    }

    // From: @contract: get(key) => get(key)
    #[test]
    fn contract_get_idempotent() {
        let first = binding.call("get", &[("key", "test")]);
        let second = binding.call("get", &[("key", "test")]);
        assert_eq!(first, second);
    }
}
```

### Phase 3 — Provider Compliance

For each (profile, interface, provider) triple:

1. Profile binds interface to concrete provider
2. Testgen instantiates the contract suite with that provider's binding
3. Hermetic profiles (unit_test): run against stubs, fast, always
4. Integration profiles (local, cloud_run): env-gated, real I/O

A provider failing a contract test is a bug detectable without manual
test authoring.

## Integration with Existing Infrastructure

- **PT-\***: Per-profile live tests already compile with profiles and run
  DryRun. Contract tests extend this to run behavioral sequences.
- **IS-\***: Interface stubs should satisfy contracts by construction.
  Contract tests on stubs verify the stubs themselves are correct.
- **Testgen auto-discovery**: Contract suites are discovered alongside
  module tests. No manual registration needed.

## What This Replaces

With contract testing, hand-written behavioral tests for providers become
unnecessary. Instead of:

```rust
// Hand-written: test that stub IssueProvider returns issues
#[test]
fn stub_issue_provider_discover() {
    let result = stub.discover(owner, repo, "sdlc:");
    assert!(!result.issues.is_empty());
}
```

The contract on `IssueProvider.discover` generates equivalent tests for
ALL providers automatically.

## Implementation Tasks

See `tasks.md` for CT-1 through CT-5 task entries.
