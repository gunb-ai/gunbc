# Design: Type-Aware Safe Coercion

> Resolves: cardinality-transparent-execution.md Task 11 (Map semantics)
> and Task 12 (Embed cardinality in type system).
>
> Principle: **No unsafe coercion.** Every coercion is either a provably
> lossless upcast or an explicit transformation step. The type system
> knows enough about each type's properties to decide at compile time
> (ideally) or runtime whether a conversion is safe.

## 1. Problem Statement

Today, cardinality lives as a separate field on `Port`:

```rust
Port { type_id: "String", cardinality: Cardinality::ONE }
```

And type compatibility is checked by name + cardinality independently:

```rust
// TypeRegistry::is_compatible()
if from_base == to_base && from_card.satisfies(to_card) { true }
```

This has three gaps:

1. **No coercion safety model.** `can_coerce_to()` is currently equivalent
   to `satisfies()` — it can't express "this conversion is safe but
   requires a transformation step."

2. **Cardinality is detached from types.** A `Port` with `type_id: "Json"`
   and `cardinality: ZERO_OR_MORE` is semantically `List<Json>`, but the
   type system doesn't know that. The registry and the port disagree.

3. **No upcast lattice.** `Url` is a refinement of `String` (every URL is
   a string), but `is_compatible("Url", "String")` returns false because
   the names differ.

## 2. Design Principles

From project direction:

- **Coercion is only safe if it's strictly an upcast or there's an actual
  transformation step.** No implicit lossy narrowing.
- **The type system should be aware of a type's properties fundamentally**
  — cardinality, base type, predicates — to infer convertibility.
- **Compile-time inference preferred**, runtime fallback when needed.

## 3. The Coercion Lattice

### 3.1 What We Mean by "Safe Upcast"

A coercion `A → B` is safe when every value of type `A` has a
well-defined, lossless representation as type `B`. This is a
**widening** — the target type's value space is a superset of the source's.

```
Concrete ──────────────────────────────── Abstract
   Int ──────────────→ Json
   Url ──────────────→ String ──────────→ Json
   Bool ─────────────→ Json
   [1,1] ────────────→ [0,∞)     (wrap scalar in list)
   [0,1] ────────────→ [0,∞)     (None→[], Some(x)→[x])
   NonEmpty<T> ──────→ List<T>   (drop the non-empty guarantee)
```

The reverse (narrowing) is **never** implicit:

```
   Json ──✗──→ Int    (might not be numeric)
   String ─✗─→ Url    (might not be valid URL)
   [0,∞) ──✗─→ [1,1]  (might be empty or have many)
```

### 3.2 Three Dimensions of Coercion

Every type has three properties the system can reason about (matching
the existing contract tower L1-L3):

| Level | Property | Example | Upcast Direction |
|-------|----------|---------|------------------|
| L1 | Cardinality | `[1,1]` → `[0,∞)` | Wider interval |
| L2 | Base type | `Int` → `Json` | More general base |
| L3 | Predicates | `NonEmpty ∧ Matches(url)` → `NonEmpty` | Fewer predicates |

A safe coercion requires **all three levels** to be safe:

```
can_safely_coerce(source, target) =
    source.cardinality ⊆ target.cardinality   -- L1: interval containment
  ∧ source.base_type ≤ target.base_type       -- L2: base type lattice
  ∧ source.predicates ⊇ target.predicates     -- L3: predicate entailment
```

### 3.3 Base Type Lattice

```
        Json (top — accepts any structured value)
       / | \
    Int Bool String
               |
              Url (refined String)

    Unit (bottom — no value)
```

Upcast rules (edges go upward):
- `Int → Json`: safe (JSON represents all integers)
- `Bool → Json`: safe
- `String → Json`: safe (JSON string)
- `Url → String`: safe (every URL is a string)
- `Url → Json`: safe (transitive: Url → String → Json)

These are already expressible as `TypeOp::Transform(Coercion { from, to })`
in the type DAG — the missing piece is *using them during edge validation*.

### 3.4 Predicate Entailment as Coercion

A type with more predicates is more constrained. Dropping predicates
is always safe (widening):

```
NonEmpty<String> → String          -- safe: drop NonEmpty
Matches("[0-9]+") → String         -- safe: drop Matches
InRange(0, 100) → Int             -- safe: drop InRange
```

Adding predicates is narrowing and requires explicit validation:

```
String → NonEmpty<String>          -- UNSAFE: might be empty
String → Url                       -- UNSAFE: might not parse as URL
```

This is already partially handled by `check_predicate_entailment()` in
the obligation model — it returns `Verified`, `Unknown`, or `Invalid`.

## 4. Embedding Cardinality in the Type System

### 4.1 Current State

```rust
// Port declares cardinality separately from type
Port { type_id: TypeId("String"), cardinality: Cardinality::ONE }

// Type DAG can also carry cardinality via Wrap nodes
Dag<TypeOp> = [Identity("String"), Wrap(Optional)]  // → [0,1]
```

The dual encoding creates drift: a Port can say `cardinality: ONE` but
its type DAG says `Wrap(Optional)` → `[0,1]`. `infer_cardinality()` tries
to resolve this by preferring the type DAG, but the fallback to the port's
declared cardinality is a source of bugs.

### 4.2 Proposed: TypeContract as the Source of Truth

Instead of `Port { type_id, cardinality }` with two independent fields,
the **type contract extracted from the type DAG** becomes authoritative:

```rust
// The type DAG IS the type. Cardinality is a derived property.
let contract = TypeContract::from_type_dag(registry.get(&port.type_id));
let cardinality = contract.cardinality;  // L1
let base = contract.base_type;           // L2
let preds = contract.predicates;         // L3
```

The port's `cardinality` field becomes a **declaration hint** that is
validated against the type DAG at build time:

```rust
// In DagBuilder::add_edge():
let source_contract = TypeContract::from_source(registry, &from_port);
let target_contract = TypeContract::from_source(registry, &to_port);

// Check all three levels
if !source_contract.can_safely_coerce_to(&target_contract) {
    return Err(BuilderError::IncompatibleTypes { ... });
}
```

### 4.3 TypeContract Coercion Check

```rust
impl TypeContract {
    /// Can this contract's values safely flow to a target contract?
    ///
    /// Checks all three levels of the contract tower:
    /// - L1: Cardinality interval containment
    /// - L2: Base type lattice (upcast only)
    /// - L3: Predicate entailment (source has ≥ target's predicates)
    pub fn can_safely_coerce_to(&self, target: &TypeContract) -> CoercionResult {
        // L1: Cardinality
        let card_ok = self.cardinality.satisfies(target.cardinality);

        // L2: Base type
        let base_ok = match (&self.base_type, &target.base_type) {
            (Some(from), Some(to)) => base_type_upcasts_to(from, to),
            (_, None) => true,   // target has no base type constraint
            (None, Some(_)) => false, // can't prove compatibility
        };

        // L3: Predicates
        let pred_ok = target.predicates.iter().all(|tp| {
            self.predicates.iter().any(|sp| sp.entails(tp))
        });

        if card_ok && base_ok && pred_ok {
            CoercionResult::Safe
        } else if !card_ok && self.cardinality.can_coerce_to(target.cardinality) {
            CoercionResult::RequiresTransform(CoercionKind::from_cardinalities(
                self.cardinality, target.cardinality
            ))
        } else {
            CoercionResult::Incompatible { card_ok, base_ok, pred_ok }
        }
    }
}

enum CoercionResult {
    /// Values flow directly, no transformation needed.
    Safe,
    /// Safe but requires an explicit transformation step.
    RequiresTransform(CoercionKind),
    /// Incompatible — cannot coerce.
    Incompatible { card_ok: bool, base_ok: bool, pred_ok: bool },
}
```

### 4.4 Base Type Upcast Function

```rust
/// Can `from` safely upcast to `to` in the base type lattice?
fn base_type_upcasts_to(from: &str, to: &str) -> bool {
    if from == to { return true; }
    match (from, to) {
        // Everything upcasts to Json (top of lattice)
        (_, "Json") => true,
        // Url is a refinement of String
        ("Url", "String") => true,
        // Custom refinement types upcast to their base
        // (requires registry lookup — see §4.5)
        _ => false,
    }
}
```

### 4.5 Registry-Driven Refinement Discovery

For user-defined types like `Url` (a refined `String`), the registry
already stores the type DAG:

```
Url = [Identity("String"), Validate(Matches("^https?://..."))]
```

The base type is `String` (from the Identity node). The predicate is
`Matches(...)`. So `Url → String` is safe because it drops a predicate.

This means `base_type_upcasts_to` can be enhanced to consult the registry:

```rust
fn base_type_upcasts_to(from: &str, to: &str, registry: &TypeRegistry) -> bool {
    if from == to { return true; }
    // Check if `from` is a refinement of `to`
    if let Some(from_dag) = registry.get_by_name(from) {
        let from_contract = TypeContract::from_type_dag(from_dag);
        if from_contract.base_type.as_deref() == Some(to) {
            return true; // `from` refines `to`
        }
    }
    // Hardcoded lattice for primitive types
    matches!((from, to), (_, "Json"))
}
```

## 5. Map Semantics Resolution

### Decision: Explicit Opt-In, Not APL-Style

Given the principle "no unsafe coercion," implicit auto-iteration is off
the table. When a scalar op receives a list input, this is a **type
mismatch**, not an auto-map opportunity.

The type system handles this naturally:

```
// Scalar op expects [1,1], receives [0,∞) → INCOMPATIBLE
// [0,∞).satisfies([1,1]) = false (might be empty)

// To process list elements individually, use explicit iteration:
// LoopBuilder or map pattern that unwraps list → processes each → collects
```

If a future need arises for map-over-list semantics, it would be:

1. **A pattern, not implicit behavior.** E.g., `MapBuilder` that takes
   a scalar sub-DAG and applies it element-wise.
2. **Type-safe.** The `MapBuilder` declares `List<T>` input and
   `List<U>` output, with the body sub-DAG being `T → U`.
3. **Explicit in the DAG structure.** Visible as a node, not hidden
   in the engine.

This means Task 11 (map semantics) is resolved as: **no implicit map;
use explicit patterns when needed.**

## 6. Implementation Plan

### Phase 1: Contract-Based Edge Validation

- [ ] Add `can_safely_coerce_to()` to `TypeContract`
- [ ] Add `base_type_upcasts_to()` with hardcoded primitive lattice
- [ ] Add `Predicate::entails()` method for predicate subsumption
- [ ] Wire into `DagBuilder::add_edge()` as an optional check
  (alongside existing `satisfies()` check)

### Phase 2: Registry-Driven Refinement

- [ ] Enhance `base_type_upcasts_to()` to consult `TypeRegistry`
- [ ] Add `TypeRegistry::is_refinement_of(from, to) -> bool`
- [ ] Update `check_predicate_entailment()` in obligation model to use
  contract-based coercion instead of ad-hoc string comparison

### Phase 3: Deprecate Dual Encoding

- [ ] Validate port cardinality against type DAG at build time
- [ ] Emit warnings when port cardinality disagrees with type DAG
- [ ] Eventually: derive port cardinality from type DAG only

### Phase 4: CoercionResult in Obligation Model

- [ ] Replace `CoercionKind` detection with `CoercionResult`-based analysis
- [ ] Generate tests for `RequiresTransform` edges (verify engine applies
  the transformation correctly)
- [ ] Surface `Incompatible` results as build-time errors

## 7. Relationship to Existing Code

| Existing | Role in New Design |
|----------|-------------------|
| `TypeContract` | Source of truth for all three levels |
| `TypeOp::Transform(Coercion)` | Explicit base type coercion in type DAGs |
| `Predicate` enum | L3 predicate model (entailment logic needed) |
| `WrapperKind` | L1 cardinality encoding in type DAGs |
| `TypeRegistry::is_compatible()` | Replaced by `TypeContract::can_safely_coerce_to()` |
| `Cardinality::can_coerce_to()` | Used within L1 check of contract coercion |
| `classify_coercion()` (coerce.rs) | Edge-level coercion detection (unchanged) |
| `detect_coercions()` (coerce.rs) | Finds edges needing transformation (unchanged) |

## 8. Open Questions

- [ ] **Q1:** Should the base type lattice be hardcoded or registry-driven?
  Phase 1 proposes hardcoded for primitives, Phase 2 adds registry lookup.
  Could start registry-only if the hardcoded set is too limiting.

- [ ] **Q2:** How should `Map<K,V>` types participate in the lattice?
  `Map<String, Json>` → `Json` is safe, but `Json` → `Map<K,V>` is not.
  Probably follows the same widening rule as other containers.

- [ ] **Q3:** Should coercion results be cached? For large DAGs,
  `TypeContract::from_type_dag()` + `can_safely_coerce_to()` per edge
  could be expensive. The analysis pass already caches port info.

## Checklist

- [ ] Contract-based edge validation (`can_safely_coerce_to`)
- [ ] Base type upcast lattice (primitives)
- [ ] Predicate entailment (`Predicate::entails()`)
- [ ] Wire contract coercion into `DagBuilder::add_edge()`
- [ ] Registry-driven refinement discovery
- [ ] Deprecate dual cardinality encoding
- [ ] Map semantics: resolved as explicit-only (no implicit auto-iteration)
