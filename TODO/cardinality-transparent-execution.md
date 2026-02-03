# Cardinality-Transparent Execution

**Status**: Design
**Date**: 2026-02-03

## Goal

Make cardinality transparent to ops. Today, ops like `MergeOutputs` must
defensively handle null/object/array because the execution engine ignores
cardinality. The engine should handle cardinality the way a for-loop does:
null/0 items skip, 1 item executes once, N items execute N times. Ops never
think about it.

This also means cardinality becomes a first-class part of the type algebra,
enabling the compiler to infer safe coercions across edges without manual
bridging nodes.

## Problem Statement

### What exists

The IR has a well-defined cardinality model:

```
Zero        ∅       signal-only, no data
One         {x}     scalar (default)
ZeroOrOne   {x}?    optional
ZeroOrMore  {x}*    list (Kleene star)
OneOrMore   {x}+    non-empty list (Kleene plus)
```

Build-time infrastructure:

- `Cardinality::satisfies()` — compatibility check with full matrix
- `Port::list()`, `Port::optional()`, etc. — constructors
- `TypeContract` / contract tower — L1 is cardinality
- `type_lib` containers — `Optional<T>`, `List<T>`, `NonEmptyList<T>`
- Type registry infers cardinality from type structure
- Builder validates cardinality on `add_edge()`
- Canonical edge ordering for deterministic fan-in

### What's missing

The execution engine (`execute.rs:429-434`) gathers inputs with
`HashMap::insert` — last-writer-wins. It has no concept of cardinality at
runtime:

```rust
// Current: cardinality-blind
for edge in &dag.edges {
    if edge.to_node == *node_id {
        if let Some(val) = upstream.get(&edge.from_port.0) {
            inputs.insert(edge.to_port.0.clone(), val.clone());
            //     ^^^^^^ overwrites on fan-in
        }
    }
}
```

Consequences:

1. **Fan-in is broken** — multiple edges to one port = last writer wins
2. **Ops do their own cardinality handling** — `MergeOutputs` manually
   normalizes null/object/array. Every merge-like op will repeat this.
3. **No auto-coercion** — `One` → `ZeroOrMore` requires manual wrapper
   nodes (we just removed `WrapArray` by making the op defensive, but
   that pushes the problem into op implementations)
4. **No map semantics** — a scalar op receiving a list can't automatically
   execute per-element

### The deeper issue

Cardinality is defined at the type level but not enforced at the value
level. The engine treats all values as scalars regardless of port
cardinality. Types are epistemic (we know the cardinality from the type
structure) but the runtime doesn't use that knowledge.

## Design

### Principle: Cardinality as transparent iteration

The execution engine should treat cardinality like a for-loop:

| Input cardinality | Behavior |
|-------------------|----------|
| null / missing    | Skip (0 iterations) |
| scalar (One)      | Execute once |
| array (N items)   | Execute N times, collect outputs |

This is analogous to SQL's implicit set operations, APL's rank
polymorphism, or shell `xargs` behavior.

### Track 1: Cardinality-aware input gathering

Replace `HashMap::insert` with cardinality-aware collection:

```rust
// Proposed: cardinality-aware
let mut inputs: HashMap<String, Value> = HashMap::new();
let mut fan_in: HashMap<String, Vec<Value>> = HashMap::new();

for edge in canonical_edge_order(&dag.edges) {
    if edge.to_node == *node_id {
        let port = node.input_port(&edge.to_port);
        if let Some(val) = upstream.get(&edge.from_port.0) {
            if port.cardinality.allows_many() {
                // List port: collect into array
                fan_in.entry(edge.to_port.0.clone())
                    .or_default()
                    .push(val.clone());
            } else {
                // Scalar port: single value (error on duplicate)
                inputs.insert(edge.to_port.0.clone(), val.clone());
            }
        }
    }
}

// Merge fan-in collections into inputs as Value::Json arrays
for (port_name, values) in fan_in {
    inputs.insert(port_name, Value::from_list(values));
}
```

Key decisions:
- List ports (`ZeroOrMore`, `OneOrMore`) collect fan-in into arrays
- Scalar ports (`One`) error on duplicate edges (instead of silent overwrite)
- `ZeroOrOne` ports accept 0 or 1 values
- Edge ordering follows `canonical_edge_order()` (already implemented)

### Track 2: Cardinality algebra

Add algebraic operations to `Cardinality` for type inference:

```rust
impl Cardinality {
    /// Lattice join (least upper bound).
    /// What cardinality can hold values from either input?
    ///
    /// One ∨ ZeroOrOne = ZeroOrOne
    /// One ∨ OneOrMore = OneOrMore
    /// ZeroOrOne ∨ OneOrMore = ZeroOrMore
    pub fn join(self, other: Cardinality) -> Cardinality { ... }

    /// Lattice meet (greatest lower bound).
    /// What cardinality satisfies both constraints?
    pub fn meet(self, other: Cardinality) -> Cardinality { ... }

    /// Product: cardinality of nested iteration.
    /// If outer has N items each containing M items, result has N*M.
    ///
    /// One × One = One
    /// OneOrMore × One = OneOrMore
    /// ZeroOrMore × OneOrMore = ZeroOrMore
    pub fn product(self, other: Cardinality) -> Cardinality { ... }

    /// Can self be safely coerced to target?
    /// Unlike satisfies(), this answers "is there a lossless coercion?"
    ///
    /// One → ZeroOrMore: yes (wrap in single-element list)
    /// ZeroOrMore → One: no (might be 0 or N)
    /// ZeroOrOne → ZeroOrMore: yes (None→[], Some(x)→[x])
    pub fn can_coerce_to(self, target: Cardinality) -> bool { ... }
}
```

The lattice structure:

```
        ZeroOrMore
       /          \
  ZeroOrOne    OneOrMore
       \          /
          One
           |
          Zero
```

`join` goes up, `meet` goes down. This is a bounded lattice with
`Zero` as bottom and `ZeroOrMore` as top.

### Track 3: Safe coercion inference (compiler pass)

A compiler pass that inserts coercion nodes when cardinality mismatches
are safe:

```rust
/// Compiler pass: insert cardinality coercions where safe.
fn insert_cardinality_coercions(dag: &mut Dag<T>) {
    for edge in &dag.edges {
        let from_card = output_port_cardinality(edge);
        let to_card = input_port_cardinality(edge);

        if from_card.satisfies(to_card) {
            continue; // already compatible
        }

        if from_card.can_coerce_to(to_card) {
            // Insert coercion node:
            // One → ZeroOrMore: wrap scalar in [scalar]
            // ZeroOrOne → ZeroOrMore: None→[], Some(x)→[x]
            // One → OneOrMore: wrap scalar in [scalar]
            insert_coercion_node(dag, edge, from_card, to_card);
        }
        // If can't coerce, leave as build error (already caught by builder)
    }
}
```

Safe coercions (lossless, no information destroyed):

| From | To | Coercion |
|------|----|----------|
| One → ZeroOrMore | `x` → `[x]` | wrap |
| One → OneOrMore | `x` → `[x]` | wrap |
| One → ZeroOrOne | `x` → `Some(x)` | trivial (already satisfied) |
| ZeroOrOne → ZeroOrMore | `None` → `[]`, `Some(x)` → `[x]` | option-to-list |
| OneOrMore → ZeroOrMore | identity | trivial (already satisfied) |

Unsafe coercions (lossy, never auto-inferred):

| From | To | Why unsafe |
|------|----|------------|
| ZeroOrMore → One | might be 0 or N | data loss |
| ZeroOrMore → OneOrMore | might be 0 | might fail |
| OneOrMore → One | might be N | which one? |
| ZeroOrOne → One | might be 0 | might fail |

### Track 4: Map semantics (auto-iteration)

When a scalar op receives a list input, the engine automatically maps:

```rust
// If node expects One but receives ZeroOrMore on a port,
// execute the node once per element, collect outputs.
fn execute_with_map(
    node: &Node,
    inputs: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    // Find list-typed inputs on scalar ports
    let map_port = find_map_candidate(node, inputs);

    match map_port {
        None => node.execute(inputs),  // normal scalar execution
        Some((port_name, items)) => {
            // Map: execute once per item, collect outputs
            let mut collected: HashMap<String, Vec<Value>> = HashMap::new();
            for item in items {
                let mut iteration_inputs = inputs.clone();
                iteration_inputs.insert(port_name.clone(), item);
                let outputs = node.execute(&iteration_inputs)?;
                for (k, v) in outputs {
                    collected.entry(k).or_default().push(v);
                }
            }
            // Wrap collected outputs as lists
            Ok(collected.into_iter()
                .map(|(k, vs)| (k, Value::from_list(vs)))
                .collect())
        }
    }
}
```

This is the "for-loop" semantics: cardinality propagates through the graph.
A scalar op in a list context becomes a map operation.

**Scope consideration**: Track 4 is powerful but complex. It changes
execution semantics fundamentally. Consider implementing tracks 1-3 first,
and Track 4 as a separate phase with explicit opt-in (e.g., a node flag
or port annotation).

### Track 5: Value representation

`Value` needs a list variant to carry collected fan-in values:

```rust
pub enum Value {
    Str(String),
    Json(serde_json::Value),
    MapStrStr(BTreeMap<String, String>),
    Bool(bool),
    Int(i64),
    Skipped,
    // New:
    List(Vec<Value>),  // Ordered collection of values
}

impl Value {
    /// Wrap a scalar value in a single-element list.
    pub fn wrap_list(self) -> Value {
        Value::List(vec![self])
    }

    /// Coerce to list: null→[], scalar→[scalar], list→list.
    pub fn to_list(self) -> Vec<Value> {
        match self {
            Value::Skipped => vec![],
            Value::List(vs) => vs,
            other => vec![other],
        }
    }
}
```

This mirrors the cardinality algebra at the value level.

## Relationship to existing work

### Contract tower (L1-L4)

Cardinality is already L1 in the contract tower. This design makes L1
*operational* — the engine respects it, not just the compiler.

### `satisfies()` vs `can_coerce_to()`

`satisfies()` answers "is this statically safe?" (compile-time gate).
`can_coerce_to()` answers "can we bridge this safely?" (compiler
inference). They're complementary:

- `satisfies()` = subtyping (A ⊆ B)
- `can_coerce_to()` = safe injection (A ↪ B)

A satisfying relationship needs no coercion. A coercible relationship
needs a node inserted. An unsatisfiable+uncoercible relationship is a
build error.

### MergeOutputs pattern

With tracks 1-3, `MergeOutputs` simplifies:

```rust
// Before: defensive deserialization in every merge-like op
let outputs: Vec<ReviewOutput> = match inputs.get("outputs")... {
    None => vec![],
    Some(j) if j.is_null() => vec![],
    Some(j) if j.is_array() => ...,
    Some(j) if j.is_object() => vec![...],
};

// After: engine guarantees list input on list port
let outputs: Vec<ReviewOutput> = inputs.get("outputs")
    .unwrap()  // engine guarantees presence for required ports
    .as_list() // engine guarantees list shape for list ports
    ...;
```

### Epistemic types and fungible inference

Types are defined by what we know about them (contract tower levels).
Cardinality is one axis of that knowledge. Because we know cardinality
structurally (from `WrapperKind` in type DAGs), we can compute safe
coercions without runtime checks. The algebra (join, meet, product)
lets us compose cardinality knowledge across edges — this is what makes
inference "fungible" (interchangeable, composable).

## Tasks

- [ ] Track 1: Cardinality-aware input gathering in execute.rs
- [ ] Track 5: Add `Value::List` variant
- [ ] Track 2: Cardinality algebra (join, meet, product, can_coerce_to)
- [ ] Track 3: Compiler pass for safe coercion insertion
- [ ] Track 4: Map semantics (auto-iteration) — scope TBD
- [ ] Migrate `MergeOutputs` to use engine-guaranteed list inputs
- [ ] Migrate other ops that do defensive cardinality handling
- [ ] Update testgen to generate cardinality edge cases

## Notes

- Track 5 (Value::List) should land first — it's the runtime
  representation that tracks 1 and 3 depend on.
- Track 1 is the highest-impact change: it fixes fan-in and makes list
  ports actually work.
- Track 2 is pure algebra — no runtime changes, can land independently.
- Track 3 depends on Track 2 (needs `can_coerce_to`).
- Track 4 is a larger design decision. Auto-mapping changes how graphs
  compose. Consider whether it should be implicit (APL-style) or require
  explicit annotation (SQL-style GROUP BY / UNNEST).
- The cardinality lattice is bounded (Zero bottom, ZeroOrMore top), which
  means join and meet are always defined — no partial order issues.

## Related Files

- `core/ir/src/types.rs` — Cardinality enum, satisfies()
- `core/ir/src/contract.rs` — Contract tower, L1 extraction
- `core/ir/src/type_lib.rs` — Container types, cardinality inference
- `core/ir/src/type_registry.rs` — Type-driven cardinality inference
- `core/ir/src/dag.rs` — Port, canonical_edge_order, fan_in_ports
- `core/ir/src/builder.rs` — add_edge cardinality validation
- `core/exec/src/execute.rs` — Input gathering (lines 429-438)
- `core/ir/src/value.rs` — Value enum (needs List variant)
- `lib/review/src/lib.rs` — MergeOutputs (current defensive pattern)
