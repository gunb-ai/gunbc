# Invariants as DAG Nodes

**Status**: Design Concept
**Date**: 2026-01-30

## Goal

Model system invariants (I1-I8) as first-class DAG nodes that produce **proof tokens**. Nodes requiring an invariant must consume the corresponding proof. This makes invariant satisfaction structural, not external.

## The Insight

Currently, invariants are enforced by:
- **Clippy rules** (I6: no escape hatches)
- **Build system** (I8: -D warnings)
- **Code structure** (I1: pure nodes)
- **Documentation** (human reads and follows)

But the gunbc philosophy says: **If it validates, it is structurally sound.**

Why should invariants be different? They could flow through the graph like any other data.

## Design Concept

### Invariants Produce Proof Tokens

```rust
/// A proof that an invariant is satisfied.
/// Cannot be constructed directly - only by invariant nodes.
pub struct Proof<I: Invariant> {
    _marker: PhantomData<I>,
}

/// Marker trait for invariants
pub trait Invariant {}

// Concrete invariants
pub struct I6NoEscapeHatch;
pub struct I7NoFallback;
pub struct I8NoWarnings;

impl Invariant for I6NoEscapeHatch {}
impl Invariant for I7NoFallback {}
impl Invariant for I8NoWarnings {}
```

### Invariant Nodes

```rust
/// An invariant node that validates a condition and produces a proof.
pub enum InvariantOp {
    /// Validates that a node only uses approved I/O boundaries
    AssertPureOrTransport,
    /// Validates that an operation returns Result, not Option with default
    AssertNoFallback,
    /// Validates that no warnings are suppressed
    AssertNoSuppressedWarnings,
}
```

### Workflow with Invariant Flow

```
                                    ┌─────────────────────┐
                                    │ AssertPureOrTransport│
                                    │ (produces I6 proof) │
                                    └──────────┬──────────┘
                                               │
                                               ▼ proof: Proof<I6>
┌───────────────┐    ┌──────────────┐    ┌─────────────────┐
│ PrepareRead   │───▶│ Execute      │───▶│ ParseResult     │
│ (pure)        │    │ (transport)  │    │ (pure)          │
└───────────────┘    └──────────────┘    └─────────────────┘
                           ▲
                           │ consumes proof
                           │
                     ┌─────┴─────────────┐
                     │ Proof<I6> required │
                     │ to execute I/O    │
                     └───────────────────┘
```

### Capability-Based I/O

The transport execute node could **require** a proof token:

```rust
pub struct TransportExecuteOp;

impl TransportExecuteOp {
    fn inputs() -> Vec<Port> {
        vec![
            port("request", "TransportRequest"),
            port("io_proof", "Proof<I6>"),  // Must prove I6 to do I/O
        ]
    }
}
```

Now I/O can only happen if you have a proof that I6 is satisfied. The proof comes from an invariant node that validates the context.

## Levels of Enforcement

### Level 1: Documentation (Current)
Invariants are documented. Humans follow them.

### Level 2: External Tools (Current)
Clippy, tests, CI check invariants externally.

### Level 3: Structural Proofs (Target)
Invariants produce proof tokens. Nodes consume proofs.
Invalid graphs can't be constructed.

### Level 4: Type-Level Proofs (Advanced)
Proofs are encoded in Rust's type system.
Invalid programs don't compile.

## Example: I7 No Fallback

```rust
/// Validates that an operation doesn't use fallback patterns
pub struct AssertNoFallback;

impl AssertNoFallback {
    fn validate(node: &Node) -> Result<Proof<I7>, InvariantViolation> {
        // Check that the node's outputs are all Result types, not Option
        for port in &node.outputs {
            if port.type_id.contains("Option") && !port.type_id.contains("Result") {
                return Err(InvariantViolation::FallbackDetected {
                    node: node.id.clone(),
                    port: port.name.clone(),
                });
            }
        }
        Ok(Proof::new())
    }
}
```

## Example: Compose Invariants

Invariants themselves can be composed:

```
┌────────────────┐     ┌────────────────┐     ┌────────────────┐
│ AssertPure     │     │ AssertNoFallback│     │ AssertNoWarn   │
│ → Proof<I6>    │     │ → Proof<I7>     │     │ → Proof<I8>    │
└───────┬────────┘     └───────┬─────────┘     └───────┬────────┘
        │                      │                       │
        └──────────────────────┼───────────────────────┘
                               ▼
                    ┌──────────────────────┐
                    │ CombineProofs        │
                    │ → Proof<WellFormed>  │
                    └──────────────────────┘
                               │
                               ▼ (can be used by any node needing all invariants)
```

## Benefits

| Benefit | How |
|---------|-----|
| **Self-documenting** | The graph shows what invariants are required where |
| **Composable** | Invariants can be combined, extended, specialized |
| **Testable** | Invariant nodes can be mocked to test failure paths |
| **Visible** | DryRun/visualization shows invariant flow |
| **Extensible** | New invariants are just new node types |

## Challenges

| Challenge | Mitigation |
|-----------|------------|
| **Complexity** | Start with one invariant (I6), prove the pattern |
| **Performance** | Proofs are zero-cost (PhantomData markers) |
| **Ergonomics** | Builder patterns auto-wire common invariant flows |
| **Chicken/egg** | Bootstrap code is exempt (documented exception) |

## Implementation Sketch

### Phase 1: Define Proof Types

```rust
// core/ir/src/invariant.rs
pub mod invariant {
    pub struct Proof<I>(PhantomData<I>);
    
    impl<I> Proof<I> {
        pub(crate) fn new() -> Self { Self(PhantomData) }
    }
    
    // Invariant markers
    pub struct I6NoEscapeHatch;
    pub struct I7NoFallback;
    pub struct I8NoWarnings;
}
```

### Phase 2: Invariant Ops

```rust
// core/ir/src/patterns/invariant.rs
pub enum InvariantOp {
    AssertPure { allowed_boundaries: Vec<String> },
    AssertNoFallback,
    AssertNoWarnings,
    CombineProofs { required: Vec<String> },
}
```

### Phase 3: Transport Requires Proof

```rust
// lib/transport/src/ops.rs
impl TransportOps {
    pub fn execute_with_proof() -> Node<TransportOps> {
        Node::opaque(
            "execute",
            vec![
                port("request", "TransportRequest"),
                port("io_proof", "Proof<I6>"),
            ],
            vec![port("response", "TransportResponse")],
            TransportOps::Execute,
        )
    }
}
```

## Related Concepts

- **Linear Types**: Proofs that must be consumed exactly once
- **Session Types**: Proofs that encode protocol state
- **Refinement Types**: Types with embedded predicates
- **Dependent Types**: Types that depend on values

## Not In Scope (For Now)

- **Full dependent types**: Too complex for initial implementation
- **Runtime proof verification**: Proofs are compile-time/construction-time
- **Proof automation**: Manual proof wiring initially

## Next Steps

1. Prototype `Proof<I>` types in `core/ir`
2. Create one `AssertPure` invariant node
3. Wire it into the gist graph as proof-of-concept
4. If successful, extend to other invariants

## Related Files

- `core/ir/src/patterns/` — Where invariant patterns would live
- `core/ir/src/contract.rs` — Existing contract/validation concepts
- `docs/design/overview.md#graph-invariants` — Invariant definitions
