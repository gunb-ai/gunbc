# Design: Syllogistic Type System

## Problem Statement

gunbc's type system has a split personality.

The *design* says types are DAGs (`Dag<TypeOp>`). The doc header of
`std/types.dag` says it plainly:

> TYPES ARE DAGS: Every type T is itself a Dag<TypeOp>.

The *implementation* disagrees. In practice, most of the compiler
pipeline works on `TypeId(String)` — a string like `"Int"` or
`"List<String>"` — and resolves it through hardcoded enums and string
matching. The eight primitives (`Bool`, `String`, `Int`, `Float`,
`Bytes`, `Unit`, `Json`, `Secret`) are axioms baked into Rust code.
Products and coproducts reference their fields by `TypeId` string, not
by nested DAG. The type checker, lowerer, and emitter all traffic in
strings.

This creates two problems:

1. **The primitives are opaque.** `Int` is a magic word. The compiler
   knows it exists because `BaseType::Int` is an enum variant in Rust
   and `type_lib::int()` returns a single-node Identity DAG. There is
   no structural definition of what an integer *is* — what it's made
   of, what operations it supports, what bit widths it permits. It's a
   label, not a construction.

2. **The type system can't express new domains.** If we wanted to
   define `Bit`, `Byte`, `Word`, `Register` — or `Qubit`, `Gate`,
   `Circuit` — we'd need to add new Rust enum variants, new hardcoded
   string matches, new special cases in every emission backend. The
   types can't teach the compiler what they are; the compiler must
   already know.

The extdeps system solved an analogous problem for external services.
The type system should follow the same pattern.

## The Analogy: Extdeps Got This Right

The extdeps system models external APIs *syllogistically*: tautological
definitions at the bottom, instantiation in the middle, composition at
the top. Nothing is hardcoded in Rust. The DSL is the source of truth.

```
Layer 0  std/behavioral.dag     "What is a side effect?"      (tautology)
Layer 1  cloud/cloud.dag        "What is a cloud provider?"   (abstract vocabulary)
Layer 2  cloud/gcp/gcp.dag      "What is GCP?"                (instantiation with facts)
Layer 3  cloud/gcp/sm.dag       "What is Secret Manager?"     (composition)
```

Adding Stripe doesn't require changing Rust code. You instantiate
existing vocabulary (`OperationBehavior`, `HttpErrorShape`,
`RetryPolicy`) with Stripe's documented values.

The type system today is the *opposite*. Adding a new base type requires
changing Rust code: `BaseType` enum, `TypeId` constructors, string
matching in `semantic_carrier_kind_for_type_id()`, emission tables in
every backend. The DSL can refine existing types and compose containers,
but it cannot *define* new structural primitives.

## The Vision: Types as Syllogisms

The same layered, tautological approach should work for types.

### Layer 0: Logical Foundations (Tautologies)

Define what *logic itself* is — not in Rust, but in the DSL:

```dag
module std.logic

// A classical proposition is either true or false.
// This is not Bool. This is the *definition* of classical logic.
type ClassicalProposition = True | False

// A classical gate is a function from propositions to propositions.
type ClassicalGate
  = Not   { input: ClassicalProposition }
  | And   { a: ClassicalProposition, b: ClassicalProposition }
  | Or    { a: ClassicalProposition, b: ClassicalProposition }
  | Xor   { a: ClassicalProposition, b: ClassicalProposition }
  | Nand  { a: ClassicalProposition, b: ClassicalProposition }
  | Nor   { a: ClassicalProposition, b: ClassicalProposition }
```

These are tautological. "A classical proposition is true or false" is
true by definition. It does not reference any external system, any
runtime, any bit width.

### Layer 1: Structural Primitives (Instantiation)

Build physical data representations from logical foundations:

```dag
module std.bit

import std.logic { ClassicalProposition, ClassicalGate }

// A bit IS a classical proposition given physical representation.
// It carries exactly the information needed to emit it:
//   - its logical basis (ClassicalProposition)
//   - its width (1)
//   - its domain (digital logic)
type Bit = ClassicalProposition where width(1)

// A byte is 8 bits. Not a magic number — a structural composition.
type Byte {
  bits: List<Bit> where length(8)
}

// A word is a parameterized composition.
type Word16 { bytes: List<Byte> where length(2) }
type Word32 { bytes: List<Byte> where length(4) }
type Word64 { bytes: List<Byte> where length(8) }
```

Key insight: `Byte` is not a primitive. It's a product of 8 `Bit`s.
`Bit` is not a primitive. It's a `ClassicalProposition` with a width
constraint. The compiler never needs a `BaseType::Bit` enum variant —
it can *derive* what a bit is by walking the type DAG.

### Layer 2: Arithmetic Types (Composition)

Build the familiar software types from structural primitives:

```dag
module std.integer

import std.bit { Word8, Word16, Word32, Word64 }

// An unsigned 8-bit integer is a Word8 with arithmetic semantics.
type UInt8  = Word8  where unsigned, arithmetic
type UInt16 = Word16 where unsigned, arithmetic
type UInt32 = Word32 where unsigned, arithmetic
type UInt64 = Word64 where unsigned, arithmetic

// A signed integer uses two's complement.
type Int8  = Word8  where signed(twos_complement), arithmetic
type Int16 = Word16 where signed(twos_complement), arithmetic
type Int32 = Word32 where signed(twos_complement), arithmetic
type Int64 = Word64 where signed(twos_complement), arithmetic

// The current "Int" becomes an alias, not a primitive.
type Int = Int64
```

### Layer 3: Domain Types (Further Composition)

Higher-level types compose Layer 2 types, exactly as they do today:

```dag
module std.types

import std.integer { Int, UInt8 }

type Port = Int where range(min: 1, max: 65535)
type Char = Int where range(min: 0, max: 1114111), brand("Char")
type Bytes = List<UInt8>
```

## What Falls Out

The critical consequence: **if the type DAG contains enough structural
information to define what something is, it contains enough information
to emit that thing in any target.**

### Verilog

A `Bit` is a `ClassicalProposition` with `width(1)`. A Verilog backend
sees the type DAG, finds the width constraint and the logical-domain
marker, and emits:

```verilog
wire [0:0] my_bit;
```

A `Word32` is a product of 4 `Byte`s, each a product of 8 `Bit`s. The
backend flattens the composition and emits:

```verilog
wire [31:0] my_word;
```

A `ClassicalGate::And` with two `Bit` inputs emits:

```verilog
assign out = a & b;
```

No special Verilog-awareness needed in the type system. The type DAG
*is* the specification. The backend *reads* the specification.

### Software Targets (Rust, Go, C)

The same type DAGs drive software emission, as they do today, but
through structural traversal rather than string matching:

- `Int64` → walk DAG → find `Word64` → find `signed(twos_complement)`
  → emit `i64` (Rust), `int64` (Go), `int64_t` (C)
- `UInt8` → walk DAG → find `Word8` → find `unsigned` → emit `u8`,
  `uint8`, `uint8_t`
- `Bytes` → walk DAG → find `List<UInt8>` → emit `Vec<u8>`, `[]byte`,
  `uint8_t*`

### Future Domains

The same pattern extends to domains that don't exist yet:

```dag
module std.quantum

type Qubit = QuantumProposition where width(1)
type QuantumGate
  = Hadamard { input: Qubit }
  | CNot { control: Qubit, target: Qubit }
  | PauliX { input: Qubit }
```

A QASM backend would read the quantum type DAGs and emit:

```qasm
qubit[1] q0;
h q0;
```

The compiler doesn't need to know about quantum computing. The type
definitions *are* the knowledge.

## Gap Analysis: Where We Are Today

### What works (keep these)

| Feature | Status |
|---------|--------|
| `Dag<TypeOp>` infrastructure | Solid — same DAG model as workflows |
| Container composition via `SubDag` | Works — `Optional`, `List`, `Set`, `Map` are truly recursive |
| Refinement chains via `Validate` nodes | Works — `Url` = `Identity → NonEmpty → Matches(regex)` |
| Brands via `Brand` node + `SubDag` | Works — `TextFilePath` is nominally distinct from `FilePath` |
| `TypeRegistry` stores `Dag<TypeOp>` | Works — lookup, compatibility, coercion paths |
| `PlatformRepr` (bits, signed, float) | Exists — the right idea, wrong location (metadata vs structural) |
| Cardinality algebra (lattice + semiring) | Excellent — mathematically rigorous |
| Predicate entailment | Works — `NonEmpty ∧ Matches(url) ⊢ NonEmpty` |

### What breaks the vision

| Gap | Impact | Severity |
|-----|--------|----------|
| **Primitives are Rust enum variants** | `BaseType::Int` is hardcoded, not derived from a DAG | High |
| **Products/Coproducts use `TypeId` strings for fields** | Structural recursion stops at record boundaries | High |
| **Pipeline carries `TypeId` strings, not DAGs** | Typecheck, lower, emit all work on strings | High |
| **`PlatformRepr` is metadata, not structure** | Bit widths are hints, not derivable from composition | Medium |
| **String-matching classification** | `semantic_carrier_kind_for_type_id()` is a 70-line match on names | Medium |
| **No width/length predicates** | Can't express `width(1)` or `length(8)` in the predicate system | Medium |
| **No domain predicates** | Can't express `unsigned`, `signed(twos_complement)`, `arithmetic` | Medium |

## Migration Path

### Phase 1: Structural Products and Coproducts

Make `TypeOp::Product` and `TypeOp::Coproduct` use `SubDag` for field
types instead of `TypeId` strings.

Before:
```rust
Product(Vec<(String, TypeId)>)
```

After:
```rust
Product(Vec<(String, Dag<TypeOp>)>)
// or equivalently:
Product(Vec<String>)  // field names only; field types are SubDag children
```

This is the keystone change. It makes the entire type tree a single
self-contained DAG, traversable without registry lookups.

**Constraint**: the `TypeRegistry` can still intern and deduplicate —
a `Product` can reference a registered DAG by embedding it as a
`SubDag`. But the structure must be *there*, not deferred to a string
lookup.

### Phase 2: Domain and Width Predicates

Extend `Predicate` with structural predicates that express physical
properties:

```rust
pub enum Predicate {
    // ... existing ...
    Width(u16),                       // bit width
    Length(u32),                       // fixed collection length
    Domain(DomainMarker),             // logical/physical domain
    Signed(SignednessKind),           // unsigned, twos_complement, ...
    Arithmetic,                       // supports +, -, *, /
}

pub enum DomainMarker {
    ClassicalLogic,
    QuantumLogic,
    FloatingPoint,
    // extensible via DSL — these become tautological definitions
}
```

These predicates are what backends read to determine how to emit a type.

### Phase 3: Define Primitives in `.dag`

Move `Bool`, `Int`, `Float`, etc. from Rust code into `std/logic.dag`,
`std/bit.dag`, `std/integer.dag`. The compiler boots with a minimal
kernel (the DAG infrastructure itself) and loads everything else from
the DSL.

The existing `type_lib.rs` functions (`string()`, `int()`, `bool()`)
become thin wrappers that parse the corresponding `.dag` definitions
rather than constructing hardcoded DAGs.

### Phase 4: Thread DAGs Through the Pipeline

Replace `TypeId` on ports with a structural reference to (or interned
key into) the full type DAG. The type checker, lowerer, and emitter
work on structural DAG data instead of string lookups.

This is the largest change. It can be done incrementally: start by
keeping `TypeId` as a cache key but always resolving to the DAG before
making decisions. Over time, the string-based paths become dead code.

### Phase 5: Emission Backends Read Structure

Replace the per-backend type mapping tables with DAG walkers. Instead
of:

```rust
match type_id.as_str() {
    "Int" => "i64",
    "String" => "String",
    ...
}
```

The backend walks the type DAG:

```rust
fn emit_type(dag: &Dag<TypeOp>) -> String {
    let shape = type_shape(dag);
    match shape {
        TypeShape::Scalar { width, signed, arithmetic, .. } => {
            // derive "i64", "u8", "wire [31:0]", etc. from structure
        }
        TypeShape::Product { fields } => {
            // derive struct / module ports from field DAGs
        }
        // ...
    }
}
```

A Verilog backend and a Rust backend read the *same* type DAGs. They
disagree only on how to render what they find.

## Design Principles

These follow directly from the extdeps invariants:

1. **No type is a primitive.** Every type is a composition of simpler
   types, bottoming out at tautological definitions in the DSL. The
   compiler kernel provides the DAG infrastructure, not the types.

2. **Types are specifications.** A type DAG contains all information
   needed to validate, emit, and test values of that type. If the
   type DAG doesn't say what width an integer has, no backend can
   know.

3. **Backends read, they don't invent.** A Verilog backend does not
   know what a "bit" is. It knows how to emit things with `width(1)`
   and `domain(ClassicalLogic)`. The *type definition* carries the
   knowledge; the backend carries the rendering rules.

4. **Layering is one-way.** `std/logic.dag` does not know bits exist.
   `std/bit.dag` does not know integers exist. `std/integer.dag` does
   not know Verilog exists. Each layer only imports from below.

5. **Adding a domain means adding `.dag` files, not Rust code.**
   Quantum computing, analog circuits, neural network layers — each
   is a new set of tautological definitions and compositions in the
   DSL. The compiler and its backends never change.

## Relationship to XLS

Google's XLS project generates Verilog from a Rust-like DSL (DSLX).
XLS achieves this by building hardware knowledge into the compiler:
~60 opcodes for bit-level operations, a scheduling pass for pipeline
stages, a delay model for timing, and a dedicated Block IR for RTL.

The syllogistic approach is philosophically different. Instead of
teaching the compiler about hardware, we teach the *type system* about
hardware — and the compiler remains domain-agnostic. If `Bit` is
defined as `ClassicalProposition where width(1)`, and `ClassicalGate`
is defined as a coproduct of `Not | And | Or | ...`, then a Verilog
backend has everything it needs without the compiler containing any
hardware-specific IR.

This is not a shortcut. XLS's scheduling pass, delay models, and
pipeline register insertion are genuinely hard problems that would
still need solutions. But the *type-level* foundation — "what is a
bit, what is a gate, what is a register" — falls out of the
syllogistic type system for free. The remaining problems (scheduling,
timing, optimization) are concerns of a specific backend, not of the
type system or the core compiler.

## Summary

The type system should work like extdeps: tautological definitions at
the bottom, structural composition in the middle, domain-specific
instantiation at the top. No hardcoded primitives. No string matching.
No Rust enum variants for base types.

If the type DAG for `Bit` says "I am a classical proposition with
width 1", then any backend that understands classical logic and widths
can emit a bit — whether that's `wire [0:0]` in Verilog, `bool` in
Rust, or `uint8_t` in C.

The types carry the knowledge. The compiler carries the infrastructure.
The backends carry the rendering rules. Nothing else is needed.
