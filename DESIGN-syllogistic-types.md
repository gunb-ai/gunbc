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

---

## Migration Scope: Complete Audit

The following is a concrete, measured accounting of what it takes to
fully migrate to the syllogistic type system. Every number comes from
the actual codebase.

### The Core Insight

The type system, the domain model, and the workflow DAGs should be the
same structure. Today they diverge:

| Concern | Representation | Source of truth |
|---------|---------------|-----------------|
| Workflows | `Dag<LoweredOp>` | `.dag` files |
| Domain models | `Dag<LoweredOp>` | `.dag` files (extdeps) |
| Types | `TypeId(String)` + hardcoded Rust enums | Rust code (`type_lib.rs`, `type_registry.rs`) |

After migration, all three use `Dag<TypeOp>` defined in `.dag` files.
The compiler is domain-agnostic infrastructure.

### Where Type Information Lives Today

```
Phase           Representation          What's lost
─────           ──────────────          ────────────
Syntax          TypeExpr (AST enum)     nothing yet
Resolve         TypeExpr (passthrough)  nothing yet
Typecheck       → TypedBinding.ty: String    ← REFINEMENTS LOST HERE
                  TypeRegistry: Dag<TypeOp>  (sidecar, not primary path)
Lower           Port.type_id: TypeId(String) ← STRUCTURE LOST HERE
Derive          Port.type_id (passthrough)   (type-transparent)
Emit            map_abstract_type(string)    ← STRING TABLES HERE
Execute         Value (dynamic)              (type-transparent)
```

The two critical lossy conversions:

1. **Typecheck → Lower**: `type_expr_to_string()` flattens
   `TypeExpr::Refined(Int, [range(1,65535)])` to just `"Int"`,
   discarding all predicates. Called 19 times in the lowerer,
   22 times in the typechecker.

2. **Lower → Emit**: `Port.type_id` is a bare string. The emitter's
   `map_abstract_type()` does string matching (`"String" → "String"`,
   `"Int" → "i64"`) with no access to structural type information.

### File-by-File Impact

#### Foundation IR (`src/00_foundation/ir/`) — The Type Kernel

| File | Lines | What changes | Effort |
|------|-------|-------------|--------|
| `type_op.rs` | 673 | Add `Width`, `Length`, `Domain`, `Signed`, `Arithmetic` predicates. Extend `Predicate` enum (~6 variants). | Small |
| `types.rs` | 2,123 | Eliminate `BaseType` enum (11 variants, 21 uses). Replace `semantic_carrier_kind_for_type_id()` (~70-line string match) with DAG metadata queries. Replace `TypeCategory` string matching (~20 uses). | Medium |
| `type_lib.rs` | 605 | Make primitives load from `.dag` instead of hardcoded `identity("String")` constructors. 14 `type_lib::` calls in this file become registry lookups. | Medium |
| `type_registry.rs` | 1,707 | Replace 102 `type_lib::` calls in `register_core_types()` with `.dag`-driven registration. The registry itself stays — it becomes a cache over DSL definitions. | Medium |
| `type_shape.rs` | 573 | Extend `TypeShape` to derive properties from new predicates (`Width`, `Domain`, etc.). 24 `PlatformRepr` references, 6 `type_lib::` calls. | Small |
| `contract.rs` | 3,726 | Extend `TypeContract::from_type_dag()` for new predicates. 56 `type_lib::` calls, 33 `WrapperKind` uses, 10 `PlatformRepr` uses. | Medium |
| `dag.rs` | — | No changes. DAG infrastructure is already correct. | None |
| `node.rs` | — | No changes. | None |

**Subtotal**: ~9,407 lines across 6 files. ~60% is test code.

#### Syntax (`src/03_source/daglang-syntax/`) — Already Almost Ready

| File | Lines | What changes | Effort |
|------|-------|-------------|--------|
| `lib.rs` | 1,262 | Add 6 new `Refinement` variants (`Width`, `Length`, `Unsigned`, `Signed`, `Arithmetic`, `Domain`). | Tiny |
| `parser.rs` | 5,019 | Add 6 match arms in `parse_refinement()` (~30 lines). The generic `Predicate` fallback already catches these — we're just promoting them to first-class. | Tiny |
| `ast_utils.rs` | — | `type_expr_to_string()` stays for display, but stops being the primary type transport. 4 call sites. | Tiny |

**Subtotal**: ~6,281 lines, but actual changes are ~50 lines.

**Good news**: `width(1)`, `length(8)`, `domain(ClassicalLogic)` etc.
already parse today via the generic `Refinement::Predicate(String)`
fallback. The parser change is a promotion, not an invention.

#### Resolve (`src/03_source/daglang-resolve/`) — No Changes

Type-transparent. Handles module discovery and import ordering only.

#### Typecheck (`src/04_semantics/daglang-typecheck/`) — The Bridge

| File | Lines | What changes | Effort |
|------|-------|-------------|--------|
| `lib.rs` | 3,920 | **Three changes**: (1) `collect_dsl_type_registry()` must handle `TypeBody::Alias` refined types (currently skipped — `TypeBody::Alias(_) => {}`). (2) `TypedBinding.ty: String` becomes `TypedBinding.ty: TypeId` with registry-backed resolution. (3) Add validation for new refinement predicates. 22 `type_expr_to_string()` calls, 10 `TypedBinding` uses. | Large |

**Subtotal**: 3,920 lines, significant refactor of the signature collection path.

This is the **keystone change**. Once `TypedBinding.ty` carries a
`TypeId` that resolves to a full `Dag<TypeOp>` through the registry,
every downstream phase can access structural type information.

#### Lower (`src/05_graph/daglang-lower/`) — The Largest Codebase

| File | Lines | What changes | Effort |
|------|-------|-------------|--------|
| `lib.rs` | 12,482 | 62 `Port::scalar()` calls with string type IDs. 19 `type_expr_to_string()` calls. Port construction switches from `Port::scalar("path", "String")` to `Port::scalar("path", type_registry.resolve("String"))` or similar. **No semantic change** — the lowerer stamps type IDs, it doesn't interpret them. | Medium-Large |
| `tests.rs` | — | 12 `Port::scalar()` calls in tests. | Small |

**Subtotal**: ~12,500 lines, but the change is mechanical — find every
`Port::scalar("name", "TypeName")` and replace the string with a
registry lookup. No logic changes.

#### Derive (`src/06_artifacts/daglang-derive/`) — Minimal

| File | Lines | What changes | Effort |
|------|-------|-------------|--------|
| `lib.rs` | 1,505 | 29 `Port::scalar()` calls in manifest/test construction. Same mechanical port-type change as lower. | Small |

#### Emit (`src/07_emit/daglang-emit/`) — Three Backends to Unify

| File | Lines | What changes | Effort |
|------|-------|-------------|--------|
| `type_mapping.rs` | 345 | Replace `map_abstract_type(string)` with `emit_type(Dag<TypeOp>)`. The 30 call sites switch from string tables to DAG walkers. Static mapping tables (`RUST_TYPE_MAPPING`, `GO_TYPE_MAPPING`) become pattern matches on `TypeShape`. | Medium |
| `type_codegen.rs` | 1,434 | Currently strips refinements: `TypeExpr::Refined(inner, _) => type_expr_to_rust(inner)`. Must inspect refinements to derive Rust types (e.g., `width(8) + unsigned → u8`). | Medium |
| `lower_to_ir.rs` | 801 | 3 `map_abstract_type()` calls. Duplicate type mapping that should use shared `type_mapping.rs`. | Small |
| `lower_c.rs` | 1,129 | 1 `map_abstract_type()` call. Independent inline C type mapping. | Small |
| `lower_go.rs` | — | 2 `map_abstract_type()` calls. | Tiny |
| `lower_rust.rs` | — | 2 `map_abstract_type()` calls. | Tiny |
| `plan.rs` | 1,071 | 35 `Port::scalar()` calls. Mechanical port-type change. | Small |
| `lib.rs` | 1,771 | 21 `Port::scalar()` calls. Mechanical port-type change. | Small |
| `service_emit.rs` | — | 1 `map_abstract_type()` call. | Tiny |

**Subtotal**: ~6,551 lines across 9 files. Main effort is unifying
the three divergent type-mapping strategies into one DAG-structural
approach.

#### Materialize/Execute — Minimal

| Area | What changes | Effort |
|------|-------------|--------|
| Transport dispatch | Uses `Port.type_id` as opaque labels. No semantic change needed. | None |
| Executor | Uses `Value` (dynamic). `ValueBacking` inference already goes through `TypeRegistry`. | Tiny |
| Auto-mock/test | 3 `value_backing_for_type_id()` calls, 3 in test_gen. Switch to registry-structural path. | Tiny |

#### DSL Files — The New Primitive Definitions

| File | What changes | Effort |
|------|-------------|--------|
| `dsl/std/types.dag` | Currently documents primitives as comments ("built-in to the compiler"). After migration, primitives are defined here structurally. | Medium |
| `dsl/std/logic.dag` | **New file**. Defines `ClassicalProposition`, `ClassicalGate`. ~30 lines. | Small |
| `dsl/std/bit.dag` | **New file**. Defines `Bit`, `Byte`, `Word16/32/64`. ~40 lines. | Small |
| `dsl/std/integer.dag` | **New file**. Defines `UInt8..64`, `Int8..64`, aliases `Int = Int64`. ~50 lines. | Small |
| `dsl/std/float.dag` | **New file**. Defines `Float32`, `Float64`, alias `Float = Float64`. ~20 lines. | Small |

### Quantified Summary

| Category | Files touched | Lines in scope | Actual change estimate |
|----------|--------------|---------------|----------------------|
| IR foundation | 6 | 9,407 | ~800 lines changed |
| Parser/syntax | 2 | 6,281 | ~50 lines changed |
| Typecheck | 1 | 3,920 | ~300 lines changed |
| Lower | 1 | 12,482 | ~200 lines changed (mechanical) |
| Derive | 1 | 1,505 | ~50 lines changed (mechanical) |
| Emit | 9 | 6,551 | ~500 lines changed |
| Materialize/Execute | ~3 | ~7,500 | ~30 lines changed |
| New DSL files | 4 new | 0 → ~140 | ~140 lines new |
| **Total** | **~27 files** | **~47,646 in scope** | **~2,070 lines changed** |

### Migration Phases (Ordered by Dependency)

#### Phase 1: Predicate Extension (1 week)

Add `Width`, `Length`, `Domain`, `Signed`, `Unsigned`, `Arithmetic`
to the predicate system.

```
Touches: type_op.rs, lib.rs (syntax), parser.rs, lib.rs (typecheck)
Lines changed: ~100
Risk: Low — additive, nothing breaks
Test: New predicate variants parse and validate
```

This is pure addition. Nothing existing changes.

#### Phase 2: Alias Registration (1 week)

Make `collect_dsl_type_registry()` in typecheck handle
`TypeBody::Alias` by constructing `Dag<TypeOp>` with appropriate
`Validate` nodes for refinements.

```
Touches: lib.rs (typecheck), type_lib.rs (may need new builders)
Lines changed: ~150
Risk: Medium — currently aliases are silently skipped
Test: `type Url = String where pattern(...)` resolves to a 3-node
      validation DAG in the registry
```

After this, every type definition in `.dag` files — including aliases
with refinements — produces a `Dag<TypeOp>` in the registry.

#### Phase 3: Product/Coproduct Structural Recursion (2 weeks)

Change `TypeOp::Product` and `TypeOp::Coproduct` to embed field type
DAGs as `SubDag` children instead of `TypeId` strings.

```
Touches: type_op.rs, type_lib.rs, type_registry.rs, contract.rs,
         type_shape.rs, typecheck
Lines changed: ~500
Risk: High — changes the core type representation
Test: Product type DAGs are self-contained; field types traversable
      without registry lookup
```

This is the structural keystone. It makes a `Product` a true DAG
containing its fields' types, just as `List<T>` already contains its
element type as a `SubDag`.

#### Phase 4: TypedBinding Carries TypeId (2 weeks)

Replace `TypedBinding.ty: String` with `TypedBinding.ty: TypeId` in
the typecheck → lower boundary. The lowerer resolves `TypeId` through
the registry when constructing ports.

```
Touches: lib.rs (typecheck), lib.rs (lower), port construction
Lines changed: ~400
Risk: High — changes the primary type transport across phases
Test: Round-trip: DSL type → TypeExpr → TypeId → Dag<TypeOp> →
      Port.type_id
```

After this, the lossy `type_expr_to_string()` conversion is no longer
on the critical path. Types flow as structured identifiers.

#### Phase 5: Emit Backends Read Structure (2 weeks)

Replace `map_abstract_type(string)` with structural `TypeShape` /
`Dag<TypeOp>` walkers. Unify the three divergent backend mapping
strategies.

```
Touches: type_mapping.rs, type_codegen.rs, lower_to_ir.rs, lower_c.rs,
         lower_go.rs, lower_rust.rs
Lines changed: ~500
Risk: Medium — functional behavior preserved, implementation changes
Test: All existing codegen tests pass with structural type resolution
```

After this, type mapping is DAG-driven. Adding `UInt8` to the DSL
automatically maps to `u8` / `uint8` / `uint8_t` without touching
Rust code — the backend reads `width(8) + unsigned` from the DAG.

#### Phase 6: DSL-Defined Primitives (1 week)

Write `std/logic.dag`, `std/bit.dag`, `std/integer.dag`,
`std/float.dag`. Make `TypeRegistry::register_primitives()` load from
these files instead of hardcoding.

```
Touches: type_registry.rs, type_lib.rs, new .dag files
Lines changed: ~300 (mostly deleting hardcoded registrations)
Risk: Medium — boot order matters (registry must load .dag before
      anything references types)
Test: `TypeRegistry::with_core_types()` produces identical DAGs
      whether loaded from .dag or from hardcoded Rust
```

After this, the primitives are DSL-defined. `Int` is
`Int64 = Word64 where signed(twos_complement), arithmetic`, not a
Rust enum variant.

#### Phase 7: Eliminate String-Based Classification (1 week)

Delete `BaseType` enum, `semantic_carrier_kind_for_type_id()`,
string-based `TypeCategory`, and `value_backing_for_type_id()`.
Replace with DAG-structural queries.

```
Touches: types.rs, type_registry.rs, auto_mock, test_gen
Lines changed: ~300 (mostly deletion)
Risk: Low by this point — all consumers already use structural paths
Test: All classification tests pass with DAG-structural queries
```

This is cleanup. By Phase 6, the string-based paths are vestigial.

### Total Effort Estimate

| Phase | Duration | Risk | Prerequisite |
|-------|----------|------|-------------|
| 1. Predicate extension | 1 week | Low | None |
| 2. Alias registration | 1 week | Medium | Phase 1 |
| 3. Structural Products/Coproducts | 2 weeks | High | Phase 2 |
| 4. TypedBinding carries TypeId | 2 weeks | High | Phase 3 |
| 5. Emit reads structure | 2 weeks | Medium | Phase 4 |
| 6. DSL-defined primitives | 1 week | Medium | Phase 5 |
| 7. Eliminate string classification | 1 week | Low | Phase 6 |
| **Total** | **~10 weeks** | | |

Phases 1-2 are safe, incremental, and independently valuable. They
can ship without committing to the full migration.

Phases 3-4 are the structural watershed. Once Products embed their
fields as SubDags and TypedBinding carries TypeId, the system is
fundamentally DAG-structural. Everything after is cleanup and payoff.

Phases 5-7 are where the payoff materializes: backends become
domain-agnostic, primitives move to the DSL, and string matching
disappears.

### What This Enables

After all seven phases:

- **Types, domain models, and workflows are the same structure.** All
  are `Dag<Op>` defined in `.dag` files, composed through layers.

- **Adding a new domain means adding `.dag` files.** Quantum types,
  hardware types, analog types — no Rust changes.

- **Verilog falls out.** A Verilog backend reads `width(1)` +
  `domain(ClassicalLogic)` and emits `wire [0:0]`. It reads a Product
  with width-annotated fields and emits `module` port declarations.
  The type definitions *are* the hardware specification.

- **The compiler is domain-agnostic infrastructure.** It processes
  DAGs. It doesn't know what a bit is, what a secret is, or what an
  HTTP error is. The `.dag` files carry all domain knowledge.
