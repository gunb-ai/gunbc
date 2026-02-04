# Cardinality-Transparent Execution

**Status**: Design
**Date**: 2026-02-03

## Goal

Two things:

1. Make cardinality transparent to ops — the engine handles null/scalar/list
   automatically, so ops never think about cardinality.
2. Model cardinality as a real type, not an enum — composable, testable,
   with derived algebra instead of hand-written truth tables.

## Problem Statement

### The enum problem

Today `Cardinality` is an enum with 5 variants:

```rust
pub enum Cardinality { Zero, One, ZeroOrOne, ZeroOrMore, OneOrMore }
```

This is a label, not a model. `satisfies()` is a 25-case match someone
typed by hand. Adding a new cardinality (e.g., `ExactlyTwo`, `TwoOrMore`,
`[3, 7]`) requires adding match arms to every function. The boolean
queries (`allows_empty`, `allows_many`, etc.) are also hand-written
per-variant.

Enums encode *names*. We need to encode *structure*.

### The engine problem

The execution engine (`execute.rs:429-434`) gathers inputs with
`HashMap::insert` — last-writer-wins. It has no concept of cardinality
at runtime. Ops like `MergeOutputs` must defensively normalize
null/object/array themselves. Every merge-like op repeats this.

### What exists (build-time only)

- `Cardinality::satisfies()` — compatibility check (25-case match)
- `Port::list()`, `Port::optional()`, etc. — constructors
- Contract tower — L1 is cardinality
- Type registry infers cardinality from type structure
- Builder validates cardinality on `add_edge()`
- Canonical edge ordering for deterministic fan-in

All compile-time. The runtime ignores it.

## Design

### Part 1: Cardinality as a real model

#### The interval representation

A cardinality is a closed interval `[min, max]` on ℕ ∪ {∞}:

```rust
/// Cardinality as a bounded interval on the natural numbers.
///
/// This is a real model, not an enum. All operations (satisfies, join,
/// meet, product, coerce) derive from interval arithmetic. Named
/// cardinalities are constants, not variants.
///
/// # Mathematical basis
///
/// A cardinality `[min, max]` represents the set of valid multiplicities.
/// `[1, 1]` means "exactly one value". `[0, ∞)` means "any number of
/// values". This is the same concept as regex quantifiers: `{min,max}`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cardinality {
    /// Minimum number of values (inclusive).
    pub min: u32,
    /// Maximum number of values (inclusive). `None` = unbounded.
    pub max: Option<u32>,
}
```

#### Named constants (not variants)

```rust
impl Cardinality {
    /// ∅ — signal-only, no data.
    pub const ZERO: Self = Self { min: 0, max: Some(0) };

    /// {x} — exactly one (scalar). Default for most ports.
    pub const ONE: Self = Self { min: 1, max: Some(1) };

    /// {x}? — optional (zero or one).
    pub const ZERO_OR_ONE: Self = Self { min: 0, max: Some(1) };

    /// {x}* — Kleene star (zero or more, list).
    pub const ZERO_OR_MORE: Self = Self { min: 0, max: None };

    /// {x}+ — Kleene plus (one or more, non-empty list).
    pub const ONE_OR_MORE: Self = Self { min: 1, max: None };
}
```

These replace the 5 enum variants. But now `Cardinality::new(2, Some(5))`
is also valid without any code changes.

#### Derived queries (not hand-written)

```rust
impl Cardinality {
    pub fn allows_empty(&self) -> bool { self.min == 0 }
    pub fn allows_one(&self) -> bool { self.max_at_least(1) }
    pub fn allows_many(&self) -> bool { self.max_at_least(2) }
    pub fn requires_one(&self) -> bool { self.min >= 1 }
    pub fn is_bounded(&self) -> bool { self.max.is_some() }
    pub fn is_scalar(&self) -> bool { self.min == 1 && self.max == Some(1) }
    pub fn is_list(&self) -> bool { self.allows_many() }

    fn max_at_least(&self, n: u32) -> bool {
        self.max.map_or(true, |m| m >= n)
    }
}
```

No match statements. Each query is a predicate on bounds.

#### Derived satisfaction (not a truth table)

```rust
impl Cardinality {
    /// Can every value this cardinality produces be accepted by `target`?
    ///
    /// This is subset containment: self ⊆ target.
    /// `[1,1]` satisfies `[0,∞)` because {1} ⊆ {0,1,2,...}.
    /// `[0,∞)` does NOT satisfy `[1,1]` because 0 ∉ {1}.
    pub fn satisfies(&self, target: Cardinality) -> bool {
        self.min >= target.min && self.max_leq(target.max)
    }

    fn max_leq(&self, target_max: Option<u32>) -> bool {
        match (self.max, target_max) {
            (_, None) => true,               // target is unbounded, anything fits
            (None, Some(_)) => false,         // we're unbounded, target is bounded
            (Some(a), Some(b)) => a <= b,     // both bounded, compare
        }
    }
}
```

One function. No 25-case match. Correct for any `[min, max]` pair, not
just the 5 named ones.

#### Lattice algebra

```rust
impl Cardinality {
    /// Join (least upper bound): union of possibilities.
    ///
    /// "What cardinality can hold values from either self or other?"
    /// [1,1] ∨ [0,1] = [0,1]  — either scalar or optional → optional
    /// [1,1] ∨ [1,∞) = [1,∞)  — either one or many → one-or-more
    /// [0,1] ∨ [1,∞) = [0,∞)  — either optional or non-empty → any
    pub fn join(self, other: Cardinality) -> Cardinality {
        Cardinality {
            min: self.min.min(other.min),
            max: match (self.max, other.max) {
                (None, _) | (_, None) => None,
                (Some(a), Some(b)) => Some(a.max(b)),
            },
        }
    }

    /// Meet (greatest lower bound): intersection of constraints.
    ///
    /// "What cardinality satisfies both self and other?"
    /// [0,1] ∧ [1,∞) = [1,1]  — must be optional AND non-empty → exactly one
    /// [0,∞) ∧ [1,1] = [1,1]  — any AND scalar → scalar
    ///
    /// Returns None if the intersection is empty (min > max).
    pub fn meet(self, other: Cardinality) -> Option<Cardinality> {
        let min = self.min.max(other.min);
        let max = match (self.max, other.max) {
            (None, x) | (x, None) => x,
            (Some(a), Some(b)) => Some(a.min(b)),
        };
        // Check if interval is valid (min <= max)
        match max {
            Some(m) if min > m => None,  // empty intersection
            _ => Some(Cardinality { min, max }),
        }
    }

    /// Product: cardinality of nested iteration.
    ///
    /// If outer produces [a,b] items each containing [c,d] items,
    /// the flattened result has [a*c, b*d] items.
    ///
    /// [1,∞) × [1,1] = [1,∞)   — many scalars → many
    /// [0,∞) × [1,∞) = [0,∞)   — optional many × non-empty → any
    /// [1,1] × [1,1] = [1,1]    — one scalar → one scalar
    pub fn product(self, other: Cardinality) -> Cardinality {
        Cardinality {
            min: self.min.saturating_mul(other.min),
            max: match (self.max, other.max) {
                (Some(0), _) | (_, Some(0)) => Some(0),
                (None, _) | (_, None) => None,
                (Some(a), Some(b)) => a.checked_mul(b),  // None on overflow → unbounded
            },
        }
    }

    /// Can self be safely coerced to target without data loss?
    ///
    /// Unlike satisfies() (which checks subset containment), this checks
    /// whether there exists a lossless injection self ↪ target.
    ///
    /// [1,1] → [0,∞): yes (wrap in [x])
    /// [0,∞) → [1,1]: no (might lose elements or fail on empty)
    /// [0,1] → [0,∞): yes (None→[], Some(x)→[x])
    pub fn can_coerce_to(self, target: Cardinality) -> bool {
        if self.satisfies(target) {
            return true;  // already compatible, trivial coercion
        }
        // Safe widening: we can always embed into a larger container.
        // [1,1] ↪ [0,∞) by wrapping as [x].
        // This works when target.min <= self.min (target accepts fewer)
        // and target.max >= self.max (target accepts more).
        // The only unsafe direction is narrowing (target stricter than self).
        target.min <= self.min && self.max_leq(target.max)
    }
}
```

#### Properties you can test

Because these are arithmetic, not case analysis, you get property-based
tests for free:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn arb_cardinality() -> impl Strategy<Value = Cardinality> {
        (0u32..10, prop::option::of(0u32..10)).prop_map(|(min, max)| {
            let max = max.map(|m| m.max(min));  // ensure valid interval
            Cardinality { min, max }
        })
    }

    proptest! {
        // Reflexivity: everything satisfies itself
        #[test]
        fn satisfies_reflexive(c in arb_cardinality()) {
            prop_assert!(c.satisfies(c));
        }

        // Transitivity: if a satisfies b and b satisfies c, then a satisfies c
        #[test]
        fn satisfies_transitive(
            a in arb_cardinality(),
            b in arb_cardinality(),
            c in arb_cardinality()
        ) {
            if a.satisfies(b) && b.satisfies(c) {
                prop_assert!(a.satisfies(c));
            }
        }

        // Join is commutative
        #[test]
        fn join_commutative(a in arb_cardinality(), b in arb_cardinality()) {
            prop_assert_eq!(a.join(b), b.join(a));
        }

        // Join is upper bound: a.join(b) accepts both a and b
        #[test]
        fn join_is_upper_bound(a in arb_cardinality(), b in arb_cardinality()) {
            prop_assert!(a.satisfies(a.join(b)));
            prop_assert!(b.satisfies(a.join(b)));
        }

        // Meet is commutative
        #[test]
        fn meet_commutative(a in arb_cardinality(), b in arb_cardinality()) {
            prop_assert_eq!(a.meet(b), b.meet(a));
        }

        // Meet is lower bound: meet(a,b) satisfies both a and b
        #[test]
        fn meet_is_lower_bound(a in arb_cardinality(), b in arb_cardinality()) {
            if let Some(m) = a.meet(b) {
                prop_assert!(m.satisfies(a));
                prop_assert!(m.satisfies(b));
            }
        }

        // Product with ONE is identity
        #[test]
        fn product_identity(c in arb_cardinality()) {
            prop_assert_eq!(c.product(Cardinality::ONE), c);
            prop_assert_eq!(Cardinality::ONE.product(c), c);
        }
    }
}
```

The enum version can't do this — you'd be testing a lookup table against
itself. With the interval model you're testing algebraic laws.

#### Serde compatibility

The named constants can serialize to friendly names for backward compat:

```rust
impl Serialize for Cardinality {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match *self {
            Self::ZERO => s.serialize_str("Zero"),
            Self::ONE => s.serialize_str("One"),
            Self::ZERO_OR_ONE => s.serialize_str("ZeroOrOne"),
            Self::ZERO_OR_MORE => s.serialize_str("ZeroOrMore"),
            Self::ONE_OR_MORE => s.serialize_str("OneOrMore"),
            _ => {
                // Non-standard cardinality: serialize as {min, max}
                let mut map = s.serialize_map(Some(2))?;
                map.serialize_entry("min", &self.min)?;
                map.serialize_entry("max", &self.max)?;
                map.end()
            }
        }
    }
}
```

Deserialization accepts both the string names and the `{min, max}` form.

#### Display

```rust
impl fmt::Display for Cardinality {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.min, self.max) {
            (0, Some(0)) => write!(f, "0"),
            (1, Some(1)) => write!(f, "1"),
            (n, Some(m)) if n == m => write!(f, "{}", n),
            (n, Some(m)) => write!(f, "{}..{}", n, m),
            (0, None) => write!(f, "0..*"),
            (n, None) => write!(f, "{}..*", n),
        }
    }
}
```

Again, no enum match — works for any `[min, max]`.

### Part 2: Cardinality-transparent execution

Once cardinality is a real model, the engine can use it.

#### Track A: Cardinality-aware input gathering

Replace `HashMap::insert` with cardinality-aware collection in
`execute.rs`:

```rust
let mut inputs: HashMap<String, Value> = HashMap::new();
let mut fan_in: HashMap<String, Vec<Value>> = HashMap::new();

for edge in canonical_edge_order(&dag.edges) {
    if edge.to_node == *node_id {
        let port = node.input_port(&edge.to_port);
        if let Some(val) = upstream.get(&edge.from_port.0) {
            if port.cardinality.is_list() {
                fan_in.entry(edge.to_port.0.clone())
                    .or_default()
                    .push(val.clone());
            } else {
                inputs.insert(edge.to_port.0.clone(), val.clone());
            }
        }
    }
}

for (port_name, values) in fan_in {
    inputs.insert(port_name, Value::from_list(values));
}
```

- List ports collect fan-in into `Value::List`
- Scalar ports take single values (error on duplicate)
- Edge ordering follows `canonical_edge_order()` (already exists)

#### Track B: Value::List variant

```rust
pub enum Value {
    Str(String),
    Json(serde_json::Value),
    MapStrStr(BTreeMap<String, String>),
    Bool(bool),
    Int(i64),
    Skipped,
    List(Vec<Value>),  // new
}

impl Value {
    /// Coerce to list: Skipped→[], scalar→[scalar], List→List.
    pub fn to_list(self) -> Vec<Value> {
        match self {
            Value::Skipped => vec![],
            Value::List(vs) => vs,
            other => vec![other],
        }
    }
}
```

#### Track C: Safe coercion compiler pass

Uses `can_coerce_to()` to insert bridging where safe:

```rust
fn insert_cardinality_coercions(dag: &mut Dag<T>) {
    for edge in &dag.edges {
        let from = output_port_cardinality(edge);
        let to = input_port_cardinality(edge);
        if !from.satisfies(to) && from.can_coerce_to(to) {
            insert_coercion_node(dag, edge, from, to);
        }
    }
}
```

Because `can_coerce_to` is interval arithmetic, this works for any
cardinality pair — not just the 5 named ones.

#### Track D: Map semantics (auto-iteration)

When a scalar op receives a list, execute per-element and collect. This
is the "for-loop" model.

Scope TBD — this changes execution semantics fundamentally. Consider
explicit opt-in vs. implicit (APL-style).

### Part 3: Cardinality as fundamental to all types

Today, cardinality is L1 in the contract tower but detached from the
type itself. The goal is to make every type carry its cardinality
structurally, so the engine/compiler can reason about it without
separate lookups.

This means `Port` doesn't need a separate `cardinality` field — it's
part of the type. The type registry doesn't need `infer_cardinality()` —
the type already knows. The contract tower's L1 isn't extracted, it's
intrinsic.

How this interacts with the type DAG (`Dag<TypeOp>`):

```
Current:  TypeOp::Wrap(List) wraps an inner type → infer ZeroOrMore
Proposed: The type itself carries Cardinality { min: 0, max: None }

Current:  Port { type_id: "String", cardinality: One }
Proposed: Port { type_id: TypeWithCardinality("String", [1,1]) }
          or equivalently: type_id carries cardinality intrinsically
```

This is a larger refactor. The interval model is a prerequisite — you
need the real model before you can embed it in types. Sequence:

1. Replace enum with struct (Part 1)
2. Engine uses cardinality (Part 2)
3. Embed cardinality in types (Part 3)

## Migration

### Backward compatibility

The 5 named constants (`ONE`, `ZERO_OR_MORE`, etc.) cover all current
usage. Existing code that writes `Cardinality::One` becomes
`Cardinality::ONE`. The `satisfies()` call sites don't change — same
method name, same semantics, but now it's one line of arithmetic instead
of 25 match arms.

### Migration steps

1. Add `Cardinality` struct alongside existing enum (shadow type)
2. Verify all 25 satisfaction cases pass with interval arithmetic
3. Verify lattice laws with property tests
4. Swap the real type, update call sites (`One` → `ONE`, etc.)
5. Remove the enum
6. Add `Value::List`
7. Update execution engine input gathering
8. Add coercion compiler pass

### What changes per crate

| Crate | Changes |
|-------|---------|
| `core/ir` | Replace enum, add algebra, update Port/Signature |
| `core/exec` | Cardinality-aware input gathering, Value::List |
| `core/testgen` | Use `test_cases()` (interface unchanged) |
| `core/codegen` | Minimal — uses Port which carries cardinality |
| `lib/*` | `Cardinality::One` → `Cardinality::ONE` (mechanical) |

## Impact on test generation

### Current state

Testgen *collects* cardinality but doesn't use it:

- `analyze_port_cardinalities()` extracts `PortCardinalityInfo` with
  `test_cases: Vec<CardinalityCase>` for every port
- `CardinalityTestInput` and `Mockable::cardinality_inputs()` exist in
  `core/test/src/mockable.rs`
- **But `codegen.rs` ignores `port_cardinalities` entirely** — the
  analysis parameter is prefixed with `_` (unused)
- Only `deps/ops.rs` implements `cardinality_inputs()` non-trivially;
  `makegen/ops.rs` returns `vec![]`

The rationale is "proven by construction": the builder validates
cardinality on `add_edge()`, so testing it is tautological.

### Why that's insufficient

The builder catches *static* mismatches. It can't catch *runtime*
cardinality divergence:

1. **Guarded/skipped nodes**: A `One` port receives `Skipped` when its
   upstream is guard-skipped. The builder can't know this at build time.
2. **Fan-in last-writer-wins**: Multiple edges to one port silently
   overwrite. The builder allows fan-in to list ports but the engine
   doesn't collect — so the declared cardinality is a lie.
3. **Defensive deserialization**: Ops like `MergeOutputs` must handle
   null/object/array because the engine doesn't enforce port cardinality.
   This is exactly the bug we hit — the graph was valid, but runtime
   values didn't match.

"Proven by construction" only proves the *graph* is valid. It doesn't
prove *execution* respects cardinality.

### What the interval model unlocks

**1. Boundary-value test generation from intervals**

The enum generates coarse test cases: `Empty`, `One`, `Many`. An
interval `[2, 5]` generates boundary cases: `{2, 3, 5}` — min, min+1,
max. This is standard boundary-value analysis but derived from the model
instead of hand-written:

```rust
impl Cardinality {
    pub fn test_cases(&self) -> Vec<u32> {
        let mut cases = vec![self.min];
        if self.min > 0 {
            cases.push(self.min - 1);  // below min (should fail/skip)
        }
        if let Some(max) = self.max {
            cases.push(max);
            if max > self.min {
                cases.push(self.min + 1);  // just above min
            }
            cases.push(max + 1);  // above max (should fail/skip)
        } else {
            // Unbounded: test with min, min+1, and a "large" value
            cases.push(self.min + 1);
            cases.push(self.min + 10);
        }
        cases.sort();
        cases.dedup();
        cases
    }
}
```

This replaces the hand-written `Empty/One/Many` enum with computed
boundary values. Any cardinality — including `[2, 5]` — gets meaningful
test cases automatically.

**2. Cardinality as a proof obligation**

Today, cardinality isn't in the obligation model (`obligation.rs`). With
the interval model, we can add a new obligation class:

```
Obligation: "port X on node Y handles cardinality [min, max]"
Discharge:
  - [min=0]: test with 0 inputs → node skips or returns empty
  - [min=1]: test with 1 input → node executes once
  - [max=N]: test with N inputs → node executes N times
  - [max+1]: test with N+1 inputs → rejected or handled gracefully
```

This is not tautological — it tests *runtime behavior*, not graph
validity. The builder proves the graph wiring is valid. The obligation
proves the op *handles* the cardinality it declared.

**3. Coercion coverage**

The coercion compiler pass (Track C) introduces bridging nodes. These
are generated code — they need tests. For each coercion `[a,b] → [c,d]`:

- Input at boundary values of `[a,b]`
- Verify output satisfies `[c,d]`
- Property: `coerce(x).cardinality.satisfies(target)` for all valid `x`

With the interval model, this is a property test. With the enum, you'd
hand-write cases.

**4. Fan-in collection testing**

Once the engine collects fan-in into `Value::List` (Track A), testgen
should verify:

- 0 edges to list port → empty list
- 1 edge to list port → single-element list
- N edges to list port → N-element list (in canonical order)
- 1 edge to scalar port → value
- 2 edges to scalar port → error (not silent overwrite)

These are engine-level tests, not op-level. The interval model makes the
expected behavior derivable from port declarations.

**5. Catching real failures**

Concrete failure classes this would catch:

| Failure | How caught |
|---------|------------|
| MergeOutputs null/object/array mismatch | Boundary test: 0 inputs to list port |
| Fan-in silent overwrite | Test: 2 edges to scalar port → error |
| Guarded node produces Skipped for required port | Test: 0 inputs to `[1,1]` port |
| Coercion node wraps incorrectly | Property: output satisfies target cardinality |
| Op assumes scalar but port is list | Obligation: test with Many → must handle |

## Tasks

- [x] Implement `Cardinality` struct with interval representation
- [x] Implement derived queries (allows_empty, is_list, etc.)
- [x] Implement `satisfies()` as interval containment
- [x] Implement lattice algebra (join, meet, product)
- [x] Implement `can_coerce_to()`
- [x] Property-based tests for algebraic laws
- [x] Serde compat (named strings + {min, max} fallback)
- [x] Add `Value::List` variant
- [x] Cardinality-aware input gathering in execute.rs
- [x] Coercion compiler pass
- [ ] Map semantics design (Track D scope decision)
- [ ] Embed cardinality in type system (Part 3)
- [x] Migrate call sites (One → ONE, etc.)
- [x] Migrate `MergeOutputs` to use engine-guaranteed list inputs
- [x] Replace `CardinalityCase` enum with computed boundary values
- [x] Add cardinality obligations to testgen obligation model
- [x] Wire `port_cardinalities` into codegen (currently unused/underscore)
- [x] Generate coercion coverage tests for compiler pass output

## Notes

- The interval model is strictly more expressive: `[2, 5]` just works.
  The enum can't express it without a new variant + match arms everywhere.
- `satisfies()` as interval containment is O(1) with no branching on
  variant count. The enum version is O(variants²).
- The lattice laws (commutativity, associativity, absorption) are
  provable from interval arithmetic. With the enum they're just hoped-for.
- Product distributes over join: `a × (b ∨ c) = (a × b) ∨ (a × c)`.
  This falls out naturally from min/max arithmetic.
- The `Cardinality` struct models the same thing as regex quantifiers
  `{min,max}`. This is well-understood theory.

## Related Files

- `core/ir/src/types.rs` — current Cardinality enum (to be replaced)
- `core/ir/src/contract.rs` — contract tower, L1 extraction
- `core/ir/src/type_lib.rs` — container types, cardinality inference
- `core/ir/src/type_registry.rs` — type-driven cardinality inference
- `core/ir/src/dag.rs` — Port, canonical_edge_order, fan_in_ports
- `core/ir/src/builder.rs` — add_edge cardinality validation
- `core/exec/src/execute.rs` — input gathering (lines 429-438)
- `core/ir/src/value.rs` — Value enum (needs List variant)
- `lib/review/src/lib.rs` — MergeOutputs (current defensive pattern)
- `core/testgen/src/analyze.rs` — collects PortCardinalityInfo (unused downstream)
- `core/testgen/src/codegen.rs` — ignores port_cardinalities (underscore param)
- `core/testgen/src/obligation.rs` — obligation model (no cardinality obligations)
- `core/test/src/mockable.rs` — CardinalityTestInput, Mockable trait
