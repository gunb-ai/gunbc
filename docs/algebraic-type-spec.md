> Part of: [THESIS.md](../THESIS.md) > [docs/architecture.md](architecture.md)

# Algebraic Type Specification

This document pins the denotational semantics, algebraic representations,
and cardinality model for the `.dag` type system. It is the authoritative
reference for Phase 3 (P3.6) `.dag` declarations and Phase 1 (P1.4)
cardinality dissolution.

## 1. Primitives

Primitives are opaque to the compiler. Their internal structure (e.g.,
`Int` as `List<List<Bit>>`) is a post-Phase 5 concern. For now:

| Name | Inhabitants | Notes |
|------|-----------|-------|
| `Unit` | 1 | The single zero-information value |
| `Bool` | 2 | `True \| False` |
| `Int` | countably infinite | Machine integers; overflow semantics TBD |
| `Float` | IEEE 754 subset | |
| `String` | countably infinite | UTF-8 sequences |
| `Bytes` | countably infinite | Raw byte sequences |
| `Secret` | same as `String` | Marked for credential handling |
| `Json` | JSON values | Structural JSON tree |

## 2. Collection Denotational Model

Each collection type is characterized by a carrier/index space and an
annotation algebra:

```
Set<A>   = A -> Bool         (finite support — membership)
Bag<A>   = A -> Nat          (finite support — multiplicity)
Map<K,V> = K -> (1 + V)      (finite support — keyed partial function)
List<A>  = Sigma n. Fin(n) -> A   (length + value per position)
```

### List

`List<A>` is **dense**: support is exactly `{0..n-1}`. Holes are
unrepresentable. Sparse sequences are a distinct type. Duplicates are
natural (carrier is positions, not elements). Order is positional, not
element-ordered.

### Set

`Set<A>` is a membership function. Uniqueness falls out of the
denotation (a function returns one answer per input). Extensional
equality: two sets with the same members are equal.

### Bag

`Bag<A>` is a multiplicity function. `(Bag, +, empty)` is a commutative
monoid where `union(B1, B2)(a) = B1(a) + B2(a)` (sum, not max).
Max-union is available as a separate operation.

### Map

`Map<K,V>` is a finite partial function. Key uniqueness falls out of
the denotation. Lookup has return cardinality `0..1` (the `1 + V` in
the denotation). Unordered by default; `SortedMap`/`OrderedMap` are
distinct types.

## 3. Algebra Laws

### Set Laws (from `A -> Bool` pointwise)

| Operation | Definition | Properties |
|-----------|-----------|------------|
| Union | `(S union T)(a) = S(a) or T(a)` | Commutative, associative, idempotent |
| Intersection | `(S inter T)(a) = S(a) and T(a)` | Commutative, associative, idempotent |
| Difference | `(S \ T)(a) = S(a) and not T(a)` | Not commutative |
| Absorption | `S union (S inter T) = S` | Standard lattice |
| Distributivity | `S inter (T union U) = (S inter T) union (S inter U)` | Boolean algebra |

**Complement is not closed** under finite-support sets (complement has
infinite support unless the carrier is finite). No general complement
operation. Use `complement_in(universe)` for finite-universe complement.

### Bag Laws

| Operation | Definition | Properties |
|-----------|-----------|------------|
| Sum-union | `(B1 + B2)(a) = B1(a) + B2(a)` | Commutative monoid (identity: empty bag) |
| Max-union | `(B1 max B2)(a) = max(B1(a), B2(a))` | Semilattice (separate operation) |
| Intersection | `(B1 inter B2)(a) = min(B1(a), B2(a))` | |

### Map Laws

**Merge requires explicit conflict resolution:**
- `merge(m1, m2, f: (V, V) -> V)` — explicit conflict function
- `merge_left(m1, m2)` — keep first on conflict
- `merge_right(m1, m2)` — keep second on conflict
- Bare `merge(m1, m2)` without conflict semantics is a compile error

### Decidable Equality

All runtime values have structural decidable equality. The DAG model
gives every value a structural form; no value is "unhashable." This
is a consequence of the model, not an imposed constraint.

## 4. Occurrence and Cardinality Model

Four concepts that must stay separate:

| Concept | Meaning | Representation |
|---------|---------|----------------|
| **Absence** | Binding may be missing | `Field.cardinality: Cardinality` (0..1) |
| **Nullability** | Value is `Present(v)` or `Null` | Coproduct `1 + T` in type graph |
| **Unknownness** | Compiler can't determine type | `InferredNode.CompilerError` (P1.9 done) |
| **Defaultability** | Field omitted, default fills it | `Field.default_value: Node?` |

### Cardinality

```
type Cardinality = Required | Optional
```

- `T` = cardinality Required (exactly 1)
- `T?` = cardinality Optional (0..1) on the binding site
- `Optional<T>` does not exist as a type constructor
- Emit reads cardinality from bindings, not type-node names

### Return Cardinality

Return cardinality lives as a separate field on the expression node:

```
Node.return_type: InferredNode?          // the type (or failure)
Node.return_cardinality: Cardinality     // Required or Optional
```

Default is `Required`. Inference sets `Optional` for operations like
`map.get(key)` where the result may be absent.

### Default Elaboration

Default values on fields are elaborated during normalization (P1.14).
The normalizer fills in defaults before inference runs.

## 5. Algebraic Representations (Phase 3 `.dag` declarations)

These are the recursive algebraic forms that will become real `.dag`
type declarations when generics (P3.6) land:

```
type List<element> = Nil | Cons { head: element, tail: List<element> }
type Map<key, value>     // finite partial function: key -> (1 + value)
type Set<element>        // membership: element -> Bool (finite support)
```

Slot names (`element`, `key`, `value`) are structural positions, not
type names. `Map` and `Set` are declared with slots but without a body
that reduces them to other types — their denotational semantics define
what operations are well-typed.

### Arity Bridge (current state)

Until P3.6 lands:
- `Map` -> 2 children (key, value)
- `List` -> 1 child (element)
- `Set` -> 1 child (element)

Defined in `00_core.dag` as `parameterized_type_arity`. Deleted when
real declarations exist (P3.7).

## 6. Open Questions (Beyond Phase 3)

| Question | Notes |
|----------|-------|
| Higher-kinded slots (`Functor<F<_>>`) | Post-Phase 3 |
| Named composition syntax (`Map<key: String, value: Int>`) | Decide in Phase 3 |
| `Bag` algebraic form | Not yet needed; `Bag<A> = A -> Nat` pins semantics |
| Constraint syntax | Likely unnecessary; all types have structural equality |
