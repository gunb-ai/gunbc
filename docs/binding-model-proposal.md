> Part of: [THESIS.md](../THESIS.md) > **Correctness dimensions** + **Binding unification**
> See also: [binding-unification-design.md](binding-unification-design.md), [dimensions-design.md](../src/v2/dimensions-design.md)

# Proposed .dag modeling: bindings, provenance, ownership

This document proposes the structural `.dag` types for the binding
unification (7 → 2), the unified provenance interface, and ownership
as a dimension. These are **proposals** — the types will land in
`dsl/std/` when the corresponding implementation work begins.

---

## 1. Binding forms (`std/binding.dag` — proposed extension)

### Concept DAG attachment

```
constructors.dag (Product, Coproduct)
  └── binding.dag — a binding is Product(name, value)
        with Coproduct(Parameter, LetBinding) discriminant
```

External authority: lambda calculus — abstraction (parameter) and
let-binding (substitution). All binding mechanisms in typed lambda
calculi reduce to these two.

### Proposed types

```dag
// BindingForm: how a name enters scope.
//
// Parameter: value provided by caller. The binding site does not
//   determine what it receives — the call site does. Provenance,
//   ownership, and other dimension values flow FROM the caller
//   TO the parameter.
//
// LetBinding: value computed by an expression. The binding site
//   names the result. Dimension values derive from the expression's
//   structure (field access, method call, constructor, etc.).
type BindingForm
  = Parameter
  | LetBinding
```

This is a coproduct (OR): a binding form is EITHER a Parameter OR
a LetBinding. The `|` syntax is .dag's representation of the
`Coproduct` concept from `constructors.dag`. Pattern matching
dispatches on which branch holds:

```dag
match form {
  Parameter  => ...   // value came from caller
  LetBinding => ...   // value came from expression
}
```

### Surface syntax (metadata, not branching)

```dag
// BindingSurface: the original syntactic form.
//
// Downstream code (CX, ownership, emission) does NOT branch on this.
// It exists for error messages, idiomatic emission, source location.
//
// Each surface maps to exactly one BindingForm:
//   FunctionParam     → Parameter
//   LambdaParam       → Parameter
//   ForEachVariable   → Parameter (for x in xs ≡ fold)
//   LetDeclaration    → LetBinding
//   MatchArmBind      → LetBinding (Foo{x,y} ≡ let x = scrutinee.Foo.x)
//   BlockLetBind      → LetBinding
type BindingSurface
  = FunctionParam
  | LambdaParam
  | ForEachVariable
  | LetDeclaration
  | MatchArmBind
  | BlockLetBind

fn binding_form(surface: BindingSurface) -> BindingForm {
  match surface {
    FunctionParam   => Parameter
    LambdaParam     => Parameter
    ForEachVariable => Parameter
    LetDeclaration  => LetBinding
    MatchArmBind    => LetBinding
    BlockLetBind    => LetBinding
  }
}
```

### Caller context (what the call site contributes to a parameter)

```dag
// CallerContext: the structural relationship between a parameter's
// value and its source.
//
// This is a property of the CALL SITE, not the binding. A lambda
// parameter doesn't need to know whether it's inside a fold, a
// descend, or a direct call — it reads whatever dimension values
// the caller provides.
//
// Maps to SubValueRelation (provenance dimension):
//   CollectionIteration → IteratedSubValue
//   TreeDescend         → StrictSubValue
//   CallableContract    → read from callee contract
//   DirectArgument      → PreservedValue
//   UnknownContext       → SubValueUnknown (bottom)
type CallerContext
  = CollectionIteration
  | TreeDescend
  | CallableContract
  | DirectArgument
  | UnknownContext
```

### Access shape (how a let-binding's value relates to its source)

```dag
// AccessShape: the structural relationship between a let-binding's
// value and the expression it names.
//
// Every dimension computes at a LetBinding site by composing the
// source's dimension value with the access shape. The composition
// is dimension-specific:
//
//   Provenance (SubValueRelation):
//     DirectValue  → source provenance unchanged
//     FieldAccess  → compose with InductiveField (structural descent)
//     Iteration    → IteratedSubValue (element of collection)
//     Arithmetic   → ArithmeticDescent (n-1, n/2, etc.)
//     Construction → SubValueUnknown (new value, not a sub-value)
//
//   Ownership (OwnershipKind):
//     DirectValue  → same ownership as source
//     FieldAccess  → Projected (borrow into structure)
//     Iteration    → element ownership from collection
//     Arithmetic   → Owned (new value)
//     Construction → Owned (new value)
//
// This table is the interface contract for dimension implementors:
// implement one rule per AccessShape, not one per syntactic form.
type AccessShape
  = DirectValue
  | FieldAccess
  | Iteration
  | Arithmetic
  | Construction
```

### Iteration binding contracts

```dag
// IterationBinding: how an iteration primitive introduces parameter
// bindings in its body.
//
// fold(collection, init, (acc, elem) => body):
//   acc  → Parameter with CallerContext = DirectArgument
//   elem → Parameter with CallerContext = CollectionIteration
//
// descend(node, (child) => body):
//   child → Parameter with CallerContext = TreeDescend
//
// repeat(n, init, (i, acc) => body):
//   i   → Parameter (iteration counter)
//   acc → Parameter with CallerContext = DirectArgument
//
// for x in collection { body }:
//   desugars to fold(collection, (), (_, x) => body; ())
//   x → Parameter with CallerContext = CollectionIteration
type IterationBinding
  = FoldAccumulator  { cardinality: Cardinality }
  | FoldElement
  | DescendChild
  | RepeatCounter
  | RepeatAccumulator

fn iteration_caller_context(ib: IterationBinding) -> CallerContext {
  match ib {
    FoldAccumulator { cardinality: _ } => DirectArgument
    FoldElement                        => CollectionIteration
    DescendChild                       => TreeDescend
    RepeatCounter                      => UnknownContext
    RepeatAccumulator                  => DirectArgument
  }
}
```

### Dimension interface (the generic contract)

For a dimension D (provenance, ownership, effects, ...):

- **Parameter binding:** `D(param) = caller_provided_value`.
  The parameter reads what it receives.

- **LetBinding:** `D(let x = expr) = compose_dimension(source_D, access_shape)`.
  The derivation composes the source value's D with the AccessShape.

The compose_dimension function is dimension-specific:
- SubValueRelation: `compose_sub_value_relations` in std/induction.dag
- OwnershipKind: `compose_ownership` in std/ownership.dag (planned)
- EffectLevel: `compose_effects` in std/effects.dag (planned)

When compose_dimension cannot determine the result: fail-closed
to `D.bottom` (BoundedLattice.bottom). Never approximate upward.

---

## 2. Ownership as dimension (`std/ownership.dag` — proposed new file)

### Concept DAG attachment

```
algebra.dag (BoundedLattice)
  └── ownership.dag — OwnershipKind inhabits BoundedLattice
binding.dag (BindingForm, AccessShape)
  └── ownership.dag — compute rules per BindingForm + AccessShape
```

External authority: linear/affine types (Girard 1987), Rust
ownership model (Matsakis, Klock 2014).

### Proposed types

```dag
// OwnershipKind: how exclusively a binding holds its value.
//
// Lattice ordering: Owned > Borrowed > Shared
//   Owned:    unique holder — can move, consume, or destroy
//   Borrowed: temporary read access — can read, not consume
//   Shared:   reference-counted — can clone, not move
//
// Meet: at join points, take the LOWER ownership (conservative).
type OwnershipKind = Owned | Borrowed | Shared

// BoundedLattice<OwnershipKind>:
//   bottom: Shared, top: Owned
//   meet: min (weaker wins), join: max (stronger wins)
fn ownership_meet(a: OwnershipKind, b: OwnershipKind) -> OwnershipKind
fn ownership_join(a: OwnershipKind, b: OwnershipKind) -> OwnershipKind
data ownership_bottom: OwnershipKind = Shared
data ownership_top: OwnershipKind = Owned
```

### Use-site edges

```dag
// UsageEdge: how a value is consumed at each reference.
// Orthogonal to OwnershipKind (what the binding CAN do vs what
// each USE SITE actually does).
type UsageEdge
  = Consumed   // moved / ownership transferred
  | Read       // read but not consumed (borrow candidate)
  | Projected  // accessed via field / index (structural borrow)
  | Threaded   // passed through unchanged (fold accumulator)
```

### Binding-site computation rule

```dag
// How OwnershipKind is computed at a LetBinding + AccessShape:
fn ownership_at_let(source: OwnershipKind, access: AccessShape) -> OwnershipKind {
  match access {
    DirectValue  => source      // naming doesn't change ownership
    FieldAccess  => Borrowed    // borrowing into structure
    Iteration    => Owned       // element extracted from collection
    Arithmetic   => Owned       // new value
    Construction => Owned       // newly constructed
  }
}
```

### Fold accumulator contract

```dag
// AccumulatorOwnership: fold guarantees about accumulator aliasing.
type AccumulatorOwnership
  = ThreadedOwned    // unique access per iteration (move semantics)
  | ThreadedShared   // cannot guarantee (aliased in body)
```

---

## 3. Provenance dimension interface (induction.dag — proposed comment extension)

SubValueRelation already exists. The proposed extension documents
the binding-site computation rules:

**Parameter binding** (provenance = what the caller provides):
- `DirectArgument` → `PreservedValue`
- `CollectionIteration` → `IteratedSubValue { field }`
- `TreeDescend` → `StrictSubValue { field, factor: UnitShrink }`
- `CallableContract` → read from `DeclaredFuncSig`
- `UnknownContext` → `SubValueUnknown` (bottom)

**LetBinding** (provenance = compose source with access shape):
- `DirectValue` → source provenance unchanged
- `FieldAccess` → `compose_sub_value(source, inductive_field)`
- `Iteration` → `IteratedSubValue { field }`
- `Arithmetic` → `ArithmeticDescent { param, factor }`
- `Construction` → `SubValueUnknown` (new value)

This replaces the triple classification system:
- `classify_binding_provenance` (04_infer.dag ~2616)
- `classify_let_value` / `classify_argument` (04_infer.dag ~2649-3104)
- `classify_body_provenance` (04_infer.dag ~3851)

---

## Open design questions

1. **`FieldAccess` ownership** — the proposal says `Borrowed`
   unconditionally. But if the source struct is being moved
   (destructured), the field could be `Owned`. The rule may need
   to consider source ownership: `FieldAccess + Owned source →
   Owned` vs `FieldAccess + Borrowed source → Borrowed`.

2. **`RepeatCounter` caller context** — proposed as `UnknownContext`.
   The counter is a known bounded arithmetic sequence (0..n). Should
   this be a new `ArithmeticContext` variant, or is `UnknownContext`
   correct because the counter is not a sub-value of any input?

3. **`Iteration` ownership** — proposed as `Owned` unconditionally.
   But if the collection is borrowed, the element might also be
   borrowed. May need: `Iteration + Owned source → Owned`,
   `Iteration + Borrowed source → Borrowed`.

---

## Status

**Proposal only.** These types will land in `dsl/std/` when the
corresponding implementation work begins (Theme 1 for binding/
provenance, Theme 3 for ownership). The roadmap references these
proposals. Next step: first implementation PR (for-each → fold
desugaring in Theme 1).
