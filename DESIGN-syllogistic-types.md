# Design: Syllogistic Type System

## The Unifying Idea

Types, domain models, and workflows should be the same structure.

Today, extdeps modeling and workflow DAGs already share a common form:
both are `Dag<Op>` defined in `.dag` files, composed through layers,
processed by the same infrastructure. The type system was the outlier —
it used `TypeId(String)` backed by hardcoded Rust enums, living in a
parallel universe from the DAGs it's supposed to describe.

But the deeper point is this: the DAG is not just a data structure for
workflows. It's a substrate for expressing *arbitrary causal truths*.
A workflow DAG says "prepare, then execute, then parse" — that's a
causal chain. A type DAG says "start with String, validate NonEmpty,
validate URL pattern" — that's also a causal chain. An extdeps module
says "HTTP errors have a status, a type, a message" — that's a
structural truth expressed in the same `.dag` vocabulary.

**Types are data.** Specifically, types are syllogistic truths —
premises that compose into conclusions — expressed as DAG structure.
A bit is a classical proposition with width 1. A byte is 8 bits.
An integer is a word with signed arithmetic. Each statement is a
truth. Each truth composes from simpler truths. The type DAG carries
those truths exactly the way a workflow DAG carries causal steps.

This means every algebraic property the system reasons about —
cardinality, ordering, compatibility, encoding, presence — should
live *on* the DAG, not beside it in ad-hoc Rust enums and match
statements.

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

The type system should follow the same pattern.

## The Vision: Types as Syllogisms

The same layered, tautological approach works for types.

### Layer 0: Logical Foundations (Tautologies)

Define what *logic itself* is — not in Rust, but in the DSL:

```dag
module std.logic

// A classical proposition is either true or false.
// This is not Bool. This is the *definition* of classical logic.
type Classical = True | False

fn classical_not(a: Classical) -> Classical {
  match a { True => False, False => True }
}
fn classical_and(a: Classical, b: Classical) -> Classical {
  match a { False => False, True => b }
}
fn classical_or(a: Classical, b: Classical) -> Classical {
  match a { True => True, False => b }
}
```

These are tautological. "A classical proposition is true or false" is
true by definition. It does not reference any external system, any
runtime, any bit width.

### Layer 1: Structural Primitives (Instantiation)

Build physical data representations from logical foundations:

```dag
module std.bit

import std.logic { Classical }

// A bit IS a classical proposition given physical representation.
type Bit = Classical where width(1)

// A byte is 8 bits. Not a magic number — a structural composition.
type Byte {
  bits: List<Bit> where length(8)
}

type Word16 { bytes: List<Byte> where length(2) }
type Word32 { bytes: List<Byte> where length(4) }
type Word64 { bytes: List<Byte> where length(8) }
```

Key insight: `Byte` is not a primitive. It's a product of 8 `Bit`s.
`Bit` is not a primitive. It's a `Classical` with a width constraint.
The compiler never needs a `BaseType::Bit` enum variant — it can
*derive* what a bit is by walking the type DAG.

### Layer 2: Arithmetic Types (Composition)

Build the familiar software types from structural primitives:

```dag
module std.integer

import std.bit { Byte, Word16, Word32, Word64 }

type UInt8  = Byte   where unsigned, arithmetic
type UInt16 = Word16 where unsigned, arithmetic
type UInt32 = Word32 where unsigned, arithmetic
type UInt64 = Word64 where unsigned, arithmetic

type Int8  = Byte   where signed, arithmetic
type Int16 = Word16 where signed, arithmetic
type Int32 = Word32 where signed, arithmetic
type Int64 = Word64 where signed, arithmetic

type Int = Int64
```

### Layer 3: Domain Types (Further Composition)

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

A `Bit` is a `Classical` with `width(1)`. A Verilog backend sees the
type DAG, finds the width constraint and the logical-domain marker,
and emits:

```verilog
wire [0:0] my_bit;
```

A `Word32` is a product of 4 `Byte`s, each a product of 8 `Bit`s. The
backend flattens the composition and emits:

```verilog
wire [31:0] my_word;
```

No special Verilog-awareness needed in the type system. The type DAG
*is* the specification. The backend *reads* the specification.

### Software Targets (Rust, Go, C)

The same type DAGs drive software emission through structural
traversal rather than string matching:

- `Int64` → walk DAG → find `Word64` → find `signed`
  → emit `i64` (Rust), `int64` (Go), `int64_t` (C)
- `UInt8` → walk DAG → find `Byte` → find `unsigned` → emit `u8`,
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

---

## Current State (post-PR `syllogistic-types`)

The first PR landed the structural foundation. Here is what exists.

### Completed

| Task | What was done | Key commits |
|------|--------------|-------------|
| **Parser predicates** | `Width(Expr)`, `Length(Expr)`, `Signed`, `Unsigned`, `Arithmetic`, `Domain(String)` as first-class `Refinement` variants | Step 1 |
| **Alias registration** | `collect_dsl_type_registry()` handles `TypeBody::Alias` — refined types produce structural DAGs in the registry | Step 2 |
| **SubDag field embedding** | `TypeOp::Product(Vec<String>)` and `TypeOp::Coproduct(Vec<String>)` — field/variant type DAGs are SubDag children, wired via edges | Step 2–3 |
| **TypedBinding carries TypeId** | `TypedBinding.ty: TypeId` instead of bare String | Step 6 |
| **Predicate-based type classification** | `StructuralProperties` derived from DAG predicates (Width, Signed, Domain, Arithmetic), not from `PlatformRepr` metadata | Step 13–14 |
| **Guard → Predicate on edges** | `Guard` enum eliminated from edge gating; `Predicate` used everywhere | Step 10 |
| **BaseType enum deleted** | 11 variants gone, all classification is DAG-structural | Step 8 |
| **Coercion struct deleted** | Transform edges use `(String, String)` pairs | Step 8 |
| **PlatformRepr replaced** | `StructuralProperties` derived from predicate DAG walking | Step 10 |
| **MetadataPayload eliminated** | `TypeOp::Meta` → `Predicate::Meta(SystemModelMeta)` | Step 16 |
| **Structural emit for numerics** | `emit_platform_type()` derives Rust/Go/C types from Width + Signed + Domain predicates | Step 14 |
| **DSL type files** | `logic.dag`, `bit.dag`, `integer.dag`, `float.dag`, `string_type.dag`, `encoding.dag`, `containers.dag` all exist | Step 4 |
| **Two-phase registry bootstrap** | `register_kernel_types()` → `merge_dsl_types()` pattern | Step 5 |
| **`TypeShape` recursive inference** | Products/Coproducts/Brands recursively walk SubDag children for structural properties | Step 13 |

### What the Compiler Looks Like Now

```
Phase           Representation                    What's structural
─────           ──────────────                    ──────────────────
Syntax          TypeExpr (AST enum)               Refinements are first-class
Typecheck       TypedBinding.ty: TypeId           Registry-backed, not bare string
                TypeRegistry: Dag<TypeOp>         Products/Coproducts have SubDag fields
Lower           Port.type_id: TypeId(String)      Still string-based (Gap 1)
Emit            resolve_and_emit(name, reg, be)   Structural path exists, but
                emit_platform_type(props, be)     name-match fallback still primary
Execute         Value (dynamic)                   Type-transparent
```

### DSL Foundation Files

These exist in `dsl/std/` and define the derivation chains:

```
std/logic.dag        Classical = True | False                     (2 variants, 3 fns)
std/bit.dag          Bit, Nibble, Byte, Word16..128               (7 types)
std/integer.dag      UInt8..64, Int8..64, Int, UInt               (12 types)
std/float.dag        Float32, Float64, Float                      (3 types)
std/string_type.dag  String { bytes, encoding }, Char             (2 types)
std/encoding.dag     Encoding = ASCII|UTF8|Latin1|Text|Binary|Unknown  (1 coproduct)
std/containers.dag   List, Option, Map, Set                       (comments only, structural in registry)
```

---

## Remaining Gaps

### Gap 1: Kernel Types Are Still Identity Placeholders

`register_kernel_types()` registers `String`, `Bool`, `Int`, `Float`,
`Bytes`, `Secret` as `identity(name)` — single-node passthrough DAGs
with no structural predicates. The DSL files define these types
structurally, but the registry doesn't compose them.

**Impact:** The structural emit path (`emit_platform_type`) works for
refined types like `Int32` that have explicit predicates, but the
default aliases (`Int`, `String`, `Bool`) fall through to the
name-match table because their registry DAGs are identity nodes.

**Fix:** Replace `identity("String")` with the structural DAG from
`string_type.dag`. Replace `identity("Bool")` with the `Classical`
coproduct from `logic.dag`. Etc. The DSL files exist — the registry
just needs to compose them instead of storing placeholders.

| Type   | Current DAG          | Target DAG                       |
|--------|----------------------|----------------------------------|
| String | `identity("String")` | Product(bytes: List\<Byte\>, encoding: Encoding) |
| Bool   | `identity("Bool")`   | Coproduct(True, False)           |
| Int    | `identity("Int")`    | Int64 alias → Word64 + Signed + Arithmetic |
| Float  | `identity("Float")`  | Float64 alias → Word64 + Domain(ieee754) |
| Bytes  | `identity("Bytes")` | List\<Byte\>                      |
| Secret | `identity("Secret")` | String + Brand("Secret")         |

### Gap 2: Hardcoded Name-Match Tables in Emit Layer

`emit_identity_type()` in `type_mapping.rs` has a 60-line match
statement replicated per backend (Rust, Go, C) that maps type names
to target-language syntax. This same list is duplicated in
`map_primitive()` (type_codegen.rs) and `map_to_c_type_static()`
(lower_c.rs) — 5 independent copies of the same knowledge.

**Impact:** Adding a type requires updating up to 5 locations.
Adding a backend requires adding another arm to every match. This is
the exact problem the syllogistic system was designed to eliminate.

**Root cause:** Gap 1. Because kernel types are identity placeholders,
the structural emit path has nothing to work with, so the name-match
table is the only thing that produces correct output.

**Fix:** Eliminate Gap 1 (structural kernel types), then the
structural path handles all types and `emit_identity_type()` becomes
dead code.

### Gap 3: Registry Not Threaded to Emit Layer in Production

Every production call site uses `lower_to_*_with_registry(None)`. The
`_with_registry(Some(reg))` path exists and is tested, but no caller
passes a real registry. The structural emit path is scaffolding only.

**Fix:** Thread `CompileOutput::merged_type_registry()` through the
codegen pipeline to the emit backends.

### Gap 4: `Guard` Still Exists in Executor

`Guard::Eq`/`Guard::NotEq` is still used in executor edge evaluation,
transport ops, and testgen codegen. The edge guards were migrated to
`Predicate` in the IR, but the executor's display and evaluation paths
still reference `Guard`.

**Files:** `exec/src/display.rs`, `transport/src/ops.rs`,
`transport/tests/basic_transports_integration.rs`,
`codegen/src/testgen/codegen.rs`

### Gap 5: Port Cardinality Stored-and-Mutated

`Port.cardinality` is set at construction and then mutated
post-construction in the builder and lowerer. `Port::with_cardinality`
exists in 6 files. Cardinality should be derived from the type DAG,
not independently stamped.

### Gap 6: `register_core_types()` Coproduct Lists

`register_core_types()` hardcodes variant lists for ~10 coproducts
(ContentEncoding, SemanticColor, SymbolId, FermiDepth, etc.). These
types already exist as DSL definitions in `.dag` files, making this a
duplicated source of truth.

### Gap 7: Transport Rewrite Tables

`rewrite_transport_call()` in each backend (Rust, Go, C) has ~15
hardcoded name→function pairs mapping abstract transport operations
to target-language runtime functions. These are replicated across
backends with minor syntax differences.

---

## The Emit Vision: Structural Pattern Matching

This is the key remaining architectural work. The emit layer should
not maintain name-match tables. Given any structural type DAG and any
target backend, it should emit valid native syntax.

### Core Concept: Derivation Chains

Every type is a structural derivation from `Classical`:

```
Classical         -- the atom: True | False
  Bit             -- Classical where width(1)
    Byte          -- { bits: List<Bit> where length(8) }
      Char        -- Int where range(0, 1114111), brand("Char")
        String    -- { bytes: List<Byte>, encoding: Encoding }
      UInt8       -- Byte where unsigned, arithmetic
      Int8        -- Byte where signed, arithmetic
    Word32        -- { bytes: List<Byte> where length(4) }
      UInt32      -- Word32 where unsigned, arithmetic
      Int32       -- Word32 where signed, arithmetic
      Float32     -- Word32 where domain("ieee754_binary32"), arithmetic
    Word64        -- { bytes: List<Byte> where length(8) }
      UInt64      -- Word64 where unsigned, arithmetic
      Int64       -- Word64 where signed, arithmetic
      Float64     -- Word64 where domain("ieee754_binary64"), arithmetic
```

The derivation chain IS the type. A backend doesn't need to know the
*name* — it pattern-matches on *structure* and selects the highest-level
native construct that covers the structural pattern.

### Backend as a Rule Set

A backend is a function:

```
emit: (TypeShape, BackendRules) -> NativeSyntax
```

`BackendRules` is a set of structural pattern recognizers, ordered by
priority:

```
BackendRules = [
  // Highest priority: native composite patterns
  (Product(bytes: List<u8>, encoding: _),  "String"),       // Rust
  (Coproduct(2 units),                     "bool"),         // Rust
  (List<Width(8)+Unsigned>,                "Vec<u8>"),      // Rust

  // Mid priority: containers
  (List<T>,                                "Vec<{T}>"),
  (Optional<T>,                            "Option<{T}>"),
  (Map<K,V>,                               "HashMap<{K},{V}>"),

  // Low priority: platform primitives
  (Arithmetic + Domain(ieee754) + W,       "f{W}"),
  (Arithmetic + Signed + W,                "i{W}"),
  (Arithmetic + Unsigned + W,              "u{W}"),
  (Width(W) + Signed,                      "i{W}"),
  (Width(W) + Unsigned,                    "u{W}"),

  // Fallback: decompose
  (Product(fields),                        "struct { {fields} }"),
  (Coproduct(variants),                    "enum { {variants} }"),
]
```

The emitter walks the rules top-down. First match wins. If no rule
matches, decompose the type one level and try again on its
constituents.

**Backends are configurable rule sets, not hardcoded match arms in
the compiler.** A Verilog backend would have different rules (no
native string, no native containers) and would decompose further down
the chain.

### Recursive Decomposition

When no backend rule matches a type's top-level shape, the emitter
decomposes one structural level and tries again. This handles cases
like Verilog encountering `String` — no native pattern matches, so it
decomposes to `List<Byte>` (still no match), then to an array of
`reg [7:0]`.

### Worked Examples

#### Example 1: Int32 (already works)

**Structure:** Word32 where signed, arithmetic
**Resolved predicates:** Width(32), Signed, Arithmetic

| Backend   | Pattern match                   | Emits              |
|-----------|---------------------------------|---------------------|
| Rust      | Arithmetic + Signed + Width(32) | `i32`               |
| Go        | Arithmetic + Signed + Width(32) | `int32`             |
| C         | Arithmetic + Signed + Width(32) | `int32_t`           |
| Verilog   | Signed + Width(32)              | `reg signed [31:0]` |
| MIPS      | Width(32)                       | `$t0` (word register) |

**Current state:** Works via `emit_platform_type`. No name matching.

#### Example 2: Float64 (already works)

**Structure:** Word64 where domain("ieee754_binary64"), arithmetic
**Resolved predicates:** Width(64), Domain("ieee754_binary64"), Arithmetic

| Backend   | Pattern match               | Emits     |
|-----------|-----------------------------|-----------|
| Rust      | Domain(ieee754) + Width(64) | `f64`     |
| Go        | Domain(ieee754) + Width(64) | `float64` |
| C         | Domain(ieee754) + Width(64) | `double`  |
| Verilog   | Domain(ieee754) + Width(64) | `real`    |

**Current state:** Works via `emit_platform_type`. No name matching.

#### Example 3: String (needs Gap 1 fix)

**Structure:** `{ bytes: List<Byte>, encoding: Encoding }`
**Resolved shape:** Product with fields `bytes: List<Width(8)+Unsigned>`,
`encoding: Coproduct(Encoding)`

A backend recognizes this pattern as "encoded byte sequence" and emits
its native string type:

| Backend   | Pattern match                          | Emits                       |
|-----------|----------------------------------------|-----------------------------|
| Rust      | Product(bytes: List\<u8\>, encoding: _) | `String`                    |
| Go        | Product(bytes: List\<u8\>, encoding: _) | `string`                    |
| C         | Product(bytes: List\<u8\>, encoding: _) | `const char*`               |
| Verilog   | (no native string)                     | decompose to `reg [7:0] mem []` |

**Current state:** Falls through to `emit_identity_type("String")`
name match. The structural information exists in `string_type.dag` but
the emitter doesn't walk it because the registry stores
`identity("String")`.

#### Example 4: Bool (needs Gap 1 fix)

**Structure:** `Classical` = `True | False` (coproduct with two unit
variants)
**Resolved shape:** Coproduct with 2 unit variants

| Backend   | Pattern match          | Emits  |
|-----------|------------------------|--------|
| Rust      | Coproduct(2 units)     | `bool` |
| Go        | Coproduct(2 units)     | `bool` |
| C         | Coproduct(2 units)     | `bool` |
| Verilog   | Coproduct(2 units) = Width(1) | `wire` |

**Current state:** Falls through to `emit_identity_type("Bool")`.

#### Example 5: Bytes (needs Gap 1 fix)

**Structure:** `List<Byte>` = `List<{ bits: List<Bit> where length(8) }>`
**Resolved shape:** Container(List) with element Width(8) + Unsigned

| Backend | Pattern match            | Emits      |
|---------|--------------------------|------------|
| Rust    | List\<Width(8)+Unsigned\>  | `Vec<u8>`  |
| Go      | List\<Width(8)+Unsigned\>  | `[]byte`   |
| C       | List\<Width(8)+Unsigned\>  | `uint8_t*` |

**Current state:** Falls through to `emit_identity_type("Bytes")`.

#### Example 6: Optional\<Int32\>

**Structure:** Container(Optional) wrapping Width(32) + Signed + Arithmetic

| Backend | Pattern match | Emits           |
|---------|---------------|-----------------|
| Rust    | Optional(T)   | `Option<i32>`   |
| Go      | Optional(T)   | `*int32`        |
| C       | Optional(T)   | `int32_t*` (nullable) |

**Current state:** Container wrapping works. Inner type resolution
depends on registry threading (Gap 3).

### Static-Analysis-Only Predicates

Some predicates exist for algebraic reasoning and do not affect
emission:

- **Length**: `List<Bit> where length(8)` — cardinality checking
- **Range**: `Int where range(0, 1114111)` — bounds checking
- **Unique**: `List<T> where unique` — distinguishes Set from List

These participate in type compatibility checking and static analysis
but are transparent to the emitter.

---

## No Metadata: Structure or Nothing

### The Principle

**If it's true, it's structure. If it's structure, it's in the DAG.**
There is no metadata. There is no sidecar. If a type is 64 bits wide,
that fact is a `width(64)` predicate on a structural node, derivable
from the composition: `Word64 = 8 × Byte = 8 × (8 × Bit) = 64 × Bit`.

### Status

The first PR eliminated `MetadataPayload` and `PlatformRepr`.
`TypeOp::Meta` is gone. Platform properties (bits, signed, float) are
now derived by `derive_structural_properties()` walking the DAG's
`Predicate::Width`, `Predicate::Signed`, `Predicate::Domain` nodes.

**Remaining escape hatch:** `Predicate::Meta(SystemModelMeta)` carries
system-model catalog metadata (SystemId, SystemKind, BehaviorId, etc.)
as a predicate rather than a separate `TypeOp::Meta` variant. This is
a deliberate extension point for metadata that genuinely lives at the
system boundary, not the type level. New `SystemModelMeta` variants
require design justification.

---

## Stacking Tautologies: The General Mechanism

### The Insight

Cardinality, set algebra, content encoding, predicate entailment —
these aren't special cases to be individually migrated. They're
*examples* of a general capability: **any tautological behavior should
be stackable onto a DAG node, and the system should enforce it
through testing**.

Today the codebase has ~14 algebraic structures (cardinality lattice,
content encoding lattice, predicate entailment, presence ordering,
access modes, fermi depth, etc.). Each is implemented as a bespoke
Rust type with hand-written algebraic laws. The algebra module
(`algebra.rs`) defines the right traits — `PartialOrder`,
`JoinSemilattice`, `MeetSemilattice`, `Lattice`, `BoundedLattice` —
but only two types implement them (`Cardinality` and
`ContentEncoding`). Everything else has implicit algebraic structure
encoded in ad-hoc match statements.

### The Mechanism: Behaviors as DAG-Attached Truths

A behavior is a named set of algebraic laws attached to a type.
In the DSL:

```dag
behavior PartialOrder extends Preorder {
  law antisymmetric: leq(a, b), leq(b, a)  implies  a == b
}

behavior BoundedLattice extends Lattice {
  element top
  element bottom
  law top_is_top:       leq(a, top)
  law bottom_is_bottom: leq(bottom, a)
}
```

Types acquire behaviors by declaration:

```dag
type ContentEncoding = ASCII | UTF8 | Latin1 | Text | Binary | Unknown
  implements BoundedLattice {
    ordering = [
      ASCII <= UTF8, UTF8 <= Text,
      Latin1 <= Text,
      Text <= Unknown, Binary <= Unknown,
    ]
    top = Unknown
    bottom = ASCII
  }
```

When a type declares `implements BoundedLattice`, the compiler can
auto-generate property-based tests verifying the algebraic laws.
Adding a new behavior to an existing type means adding an `implements`
clause and getting tests for free. No Rust changes.

### Guards and Predicates: Same Tautology

Guards on edges and predicates on type DAGs are the same thing:
truth assertions that gate data flow. The `Guard` enum is partially
eliminated (edge guards use `Predicate` in the IR), but the executor
still references `Guard` in a few files (Gap 4). When fully unified,
there is one mechanism for all truth assertions.

---

## XLS Parity Analysis

Google's XLS project generates synthesizable Verilog from DSLX
(a Rust-like hardware DSL). This section maps every XLS capability
to what gunbc provides after the syllogistic migration, identifies
what's covered, what's missing, and how the syllogistic approach
makes closing each gap easier.

### XLS Architecture (for reference)

```
DSLX source → IR conversion → optimization → scheduling → codegen → Verilog
```

- **DSLX**: Rust-like syntax, `bits[N]` types, parametric generics,
  `for` loops, `match`, structs, enums, procs (stateful), channels
- **IR**: ~60 opcodes, SSA dataflow (sea-of-nodes), three
  abstractions (Function, Proc, Block)
- **Scheduling**: assigns IR ops to clock cycles, delay modeling,
  pipeline register insertion
- **Codegen**: IR → Verilog modules with `always_comb`/`always_ff`,
  port declarations, reset logic

### Layer 1: Type System

| XLS Capability | XLS Implementation | gunbc After Migration | Gap? |
|---|---|---|---|
| **Fixed-width bits** `bits[N]` | Built-in type, N is compile-time constant | `Bit where width(N)` — structural, N from `width` predicate | **Covered** |
| **Signed/unsigned** `sN`, `uN` | Built-in signed/unsigned integer types | `Word where signed(twos_complement)` / `where unsigned` | **Covered** |
| **Arbitrary width** `bits[37]` | Any N from 1 to ~2^16 | `List<Bit> where length(37)` → width(37) derived | **Covered** |
| **Bool** | `bool` = `bits[1]` | `Bool = Classical` where `Bit = Classical where width(1)` | **Covered** |
| **Arrays** `T[N]` | Fixed-size, homogeneous | `List<T> where length(N)` | **Covered** |
| **Tuples** `(T1, T2)` | Fixed-size, heterogeneous | Anonymous product `{ a: T1, b: T2 }` | **Covered** |
| **Structs** | Named products with fields | `type S { field: T }` — same as today | **Covered** |
| **Enums** | Tagged unions | `type E = A \| B { payload }` — same as today | **Covered** |
| **Parametric types** `fn f<N: u32>(x: bits[N])` | Compile-time parametric instantiation | Generic types `List<T>`, `Map<K,V>` exist; **width parametrics need work** | **Partial** |
| **Token** | Opaque ordering type for channel I/O | No equivalent — could be `type Token = Unit where brand("Token")` | **Gap** |

### Layer 2: Operations (~60 XLS IR Opcodes)

| Category | XLS Opcodes | Covered | Needs New Behavior | Needs Backend Work |
|----------|-------------|---------|-------|------|
| **Arithmetic** | add, sub, neg, mul, div, mod | 4/6 | DivMod behavior | — |
| **Bitwise** | and, or, xor, not, nand, nor | 6/6 via BooleanAlgebra | Bitwise word-lift | — |
| **Shifts** | shll, shrl, shra | 0/3 | Bitwise { shift } | — |
| **Bit manipulation** | slice, update, concat, reverse, sign_ext, zero_ext, encode, decode | 2/8 structural | 6 ops in Bitwise/Signed | — |
| **Comparison** | eq, ne, unsigned, signed (10 ops) | 10/10 via TotalOrder | — | — |
| **Selection** | sel, one_hot, one_hot_sel, priority_sel | 1/4 (match) | Multiplexing behavior | — |
| **Array/Tuple** | construct, index, update, concat, slice, tuple_index | 4/7 structural | update, slice | — |
| **Channels/Procs** | send, receive, after_all, proc state | 0/4 | Channel behavior + state syntax | — |

### Layer 3-4: Scheduling, Codegen

| Area | Gap |
|------|-----|
| **Scheduling** (clock, pipeline, delay model, register insertion) | Full backend pass needed |
| **Codegen** (module, ports, wire/reg, always_comb/ff, reset) | Verilog emitter + state/reset behaviors |

**Assessment:** The syllogistic type system covers ~60% of XLS's
capability surface for free. Another ~25% is new behaviors
(`Bitwise`, `DivMod`, `Multiplexing`, channels) as `.dag` modules.
The remaining ~15% is genuine backend work (scheduler + Verilog
emitter). That ~15% is localized — it doesn't require changes to the
compiler core.

---

## Future State: What the `.dag` World Looks Like

### Layer 0: Logic (`std/logic.dag`)

```dag
module std.logic

// Classical (two-valued) logic: {⊤, ⊥} with ¬, ∧, ∨.
// Ref: Enderton, "A Mathematical Introduction to Logic" (2001)
type Classical = True | False

fn classical_not(a: Classical) -> Classical {
  match a { True => False, False => True }
}
fn classical_and(a: Classical, b: Classical) -> Classical {
  match a { False => False, True => b }
}
fn classical_or(a: Classical, b: Classical) -> Classical {
  match a { True => True, False => b }
}

// Kleene (three-valued) logic: adds Unknown for partial information.
// Ref: Kleene, "Introduction to Metamathematics" (1952)
type Kleene = KTrue | KFalse | KUnknown

// Belnap–Dunn (four-valued) logic: adds Both for contradictory info.
// Ref: Belnap, "A useful four-valued logic" (1977)
type Belnap = BTrue | BFalse | BUnknown | BBoth
```

### Layer 0: Algebra (`std/algebra.dag`)

```dag
module std.algebra

// Standard algebraic structure hierarchy.
// Refs: Lang "Algebra" (2002), Davey & Priestley "Introduction to
//   Lattices and Order" (2002), Lean mathlib naming conventions.

// ── Order structures ────────────────────────────────────────────
behavior Preorder {
  operation leq(a, b) -> Bool
  law reflexive:  leq(a, a)
  law transitive: leq(a, b), leq(b, c)  implies  leq(a, c)
}

behavior PartialOrder extends Preorder {
  law antisymmetric: leq(a, b), leq(b, a)  implies  a == b
}

behavior TotalOrder extends PartialOrder {
  law total: leq(a, b) or leq(b, a)
}

// ── Lattice structures ──────────────────────────────────────────
behavior JoinSemilattice extends PartialOrder {
  operation join(a, b) -> Self
  law commutative:  join(a, b) == join(b, a)
  law associative:  join(join(a, b), c) == join(a, join(b, c))
  law idempotent:   join(a, a) == a
  law upper_bound:  leq(a, join(a, b))
}

behavior MeetSemilattice extends PartialOrder {
  operation meet(a, b) -> Self?
  law commutative:  meet(a, b) == meet(b, a)
  law idempotent:   meet(a, a) == Some(a)
  law lower_bound:  meet(a, b) is Some(m) implies leq(m, a)
}

behavior Lattice extends JoinSemilattice, MeetSemilattice {
  law absorption_join: join(a, meet(a, b)) == a  when meet(a, b) exists
  law absorption_meet: meet(a, join(a, b)) == Some(a)
}

behavior BoundedLattice extends Lattice {
  element top
  element bottom
  law top_is_top:       leq(a, top)
  law bottom_is_bottom: leq(bottom, a)
}

behavior BooleanAlgebra extends BoundedLattice {
  operation complement(a) -> Self
  law complement_join: join(a, complement(a)) == top
  law complement_meet: meet(a, complement(a)) == Some(bottom)
  law distributive: meet(a, join(b, c)) == join(meet(a, b), meet(a, c))
}

// ── Algebraic structures ────────────────────────────────────────
behavior Ring {
  operation add(a, b) -> Self
  operation mul(a, b) -> Self
  operation neg(a) -> Self
  element zero
  element one
  law add_commutative:  add(a, b) == add(b, a)
  law add_associative:  add(add(a, b), c) == add(a, add(b, c))
  law add_identity:     add(a, zero) == a
  law add_inverse:      add(a, neg(a)) == zero
  law mul_associative:  mul(mul(a, b), c) == mul(a, mul(b, c))
  law mul_identity:     mul(a, one) == a
  law left_distribute:  mul(a, add(b, c)) == add(mul(a, b), mul(a, c))
  law right_distribute: mul(add(a, b), c) == add(mul(a, c), mul(b, c))
}

behavior CommutativeRing extends Ring {
  law mul_commutative: mul(a, b) == mul(b, a)
}

behavior IntegralDomain extends CommutativeRing {
  law no_zero_divisors: mul(a, b) == zero implies (a == zero or b == zero)
}

behavior Field extends IntegralDomain {
  operation reciprocal(a) -> Self
  law mul_inverse: a != zero implies mul(a, reciprocal(a)) == one
}
```

### Layer 1: Bits (`std/bit.dag`)

```dag
module std.bit

import std.logic { Classical }
import std.algebra { BooleanAlgebra }

type Bit = Classical where width(1)
  implements BooleanAlgebra {
    join(a, b) = classical_or(a, b)
    meet(a, b) = classical_and(a, b)
    complement(a) = classical_not(a)
    top = True
    bottom = False
  }

type Nibble  = { bits: List<Bit> where length(4) }
type Byte    = { bits: List<Bit> where length(8) }
type Word16  = { bytes: List<Byte> where length(2) }
type Word32  = { bytes: List<Byte> where length(4) }
type Word64  = { bytes: List<Byte> where length(8) }
type Word128 = { bytes: List<Byte> where length(16) }
```

### Layer 2: Integers, Floats, Strings

```dag
module std.integer

import std.bit { Byte, Word16, Word32, Word64 }
import std.algebra { CommutativeRing, TotalOrder }

type UInt8  = Byte   where unsigned
type UInt16 = Word16 where unsigned
type UInt32 = Word32 where unsigned
type UInt64 = Word64 where unsigned

type Int8  = Byte   where signed(twos_complement)
type Int16 = Word16 where signed(twos_complement)
type Int32 = Word32 where signed(twos_complement)
type Int64 = Word64 where signed(twos_complement)

type Int = Int64
  implements CommutativeRing {
    add(a, b) = intrinsic_add(a, b)
    mul(a, b) = intrinsic_mul(a, b)
    neg(a) = intrinsic_neg(a)
    zero = 0
    one = 1
  }
  implements TotalOrder

type UInt = UInt64
  implements TotalOrder
```

```dag
module std.float

import std.bit { Word32, Word64 }
import std.algebra { Field }

// Ref: IEEE 754-2019 §3.3
type Float32 = Word32 where ieee754(binary32)
type Float64 = Word64 where ieee754(binary64)

type Float = Float64
  implements Field {
    add(a, b) = intrinsic_fadd(a, b)
    mul(a, b) = intrinsic_fmul(a, b)
    neg(a) = intrinsic_fneg(a)
    reciprocal(a) = intrinsic_fdiv(one, a)
    zero = 0.0
    one = 1.0
    approximate = true
  }
```

```dag
module std.string

import std.bit { Byte }
import std.encoding { Encoding }

// Ref: Unicode Standard §2.7
type String = { bytes: List<Byte>, encoding: Encoding }
type Char = Int where range(min: 0, max: 1114111), brand("Char")
```

```dag
module std.containers

type List<T> = { elements: Collection<T> }
type Option<T> = { value: Optional<T> }
type Map<K, V> = { entries: List<{ key: K, value: V }> }
type Set<T> = { elements: List<T> where unique }
```

### Layer 3: Refinements (`std/types.dag` — rewritten)

```dag
module std.types

import std.integer { Int }
import std.string { String }
import std.logic { Classical }

type Bool = Classical
type Bytes = List<UInt8>
type Json = String | Int | Bool | Float | List<Json> | Map<String, Json>

type Url        = String where non_empty, pattern("^https?://")
type FilePath   = String where non_empty
type Port       = Int where range(min: 1, max: 65535)
type CommitSha  = String where pattern("^[a-f0-9]{40}$")
type Secret     = String where brand("Secret")
```

**`gist.dag` doesn't change.** The tool layer imports types and uses
them. The revolution is below — `Bool` resolves to `Classical`
(a coproduct of `True | False` from `std/logic.dag`), `Int` resolves
through `Int64 → Word64 → 8 × Byte → 64 × Bit → Classical`.

---

## Execution Plan: Remaining Work

### What's Done

| Original Task | Status |
|---------------|--------|
| Task 0: Parser accepts new predicates | **DONE** |
| Task 1: Alias types register in TypeRegistry | **DONE** |
| Task 2: Product/Coproduct fields become SubDags | **DONE** |
| Task 3: Migrate Bool (DSL definition) | **DONE** (logic.dag exists) |
| Task 4: Migrate Int, Float (DSL definitions) | **DONE** (bit.dag, integer.dag, float.dag exist; structural emit works for predicates) |
| Task 5: Migrate String, Bytes (DSL definitions) | **PARTIAL** (string_type.dag, encoding.dag exist; registry still uses identity) |
| Task 6: Migrate containers (DSL definitions) | **PARTIAL** (containers.dag exists as comments; structural in registry) |
| Task 7: Emit backends read structure | **PARTIAL** (structural path exists but not wired in production) |
| Task 8: Delete BaseType, string classification | **DONE** |
| Task 9: Cardinality derived, Guard→Predicate | **PARTIAL** (edge guards migrated; Guard still in executor; cardinality still stored) |
| Task 10: Delete MetadataPayload, PlatformRepr | **DONE** (MetadataPayload deleted; PlatformRepr → StructuralProperties) |

### Next PR: Cross the Structural Watershed

The goal: eliminate `emit_identity_type()` and all hardcoded
name-match tables. After this PR, adding a type or a backend requires
zero compiler changes.

#### Phase A: Structural Kernel Types (Gap 1)

Replace identity placeholders with structural DAGs in the registry.

**A1.** `register_kernel_types()` — replace `identity("String")` with
the structural DAG composed from `string_type.dag`. Same for Bool
(from `logic.dag` Classical), Int (from `integer.dag` Int64 chain),
Float (from `float.dag` Float64 chain), Bytes (List\<Byte\>), Secret
(branded String).

**A2.** Verify: `audit_identity_types()` on the merged registry
returns zero (or a documented baseline for truly opaque types like
`Any`).

**A3.** Two-pass registration in `collect_dsl_type_registry()` —
forward refs resolve to pass-1 placeholders instead of identity
fallback. Topological sort for pass 2 minimizes residual placeholders.

**Verification:** Every type in `register_kernel_types()` resolves to
a multi-node structural DAG. `cargo test --workspace` green.

#### Phase B: Wire Registry to Emit (Gap 3)

Thread `CompileOutput::merged_type_registry()` through the codegen
pipeline so production callers use
`lower_to_*_with_registry(Some(reg))` instead of `None`.

**Verification:** `resolve_and_emit` hits the structural path in
production. `emit_platform_type` handles all numeric types.

#### Phase C: Backend Rule Sets (Gap 2)

Replace `emit_identity_type()` with structural pattern matching.

**C1.** Define `BackendRules` as a declarative Rust structure —
ordered list of (TypeShape pattern → native syntax template) per
backend.

**C2.** Implement recursive decomposition: if no rule matches, peel
one structural level and retry.

**C3.** Add rules for String (Product with bytes+encoding), Bool
(Coproduct with 2 units), Bytes (List\<u8\>).

**C4.** Delete `emit_identity_type()`, `map_primitive()`,
`try_refined_to_rust()`, `map_to_c_type_static()`.

**Verification:** Zero hardcoded type-name match tables in the emit
layer. Adding a new type requires only a `.dag` definition.

#### Phase D: Cleanup (Gaps 4-7)

**D1.** Eliminate remaining `Guard` references in executor.
**D2.** Derive port cardinality from type DAGs; restrict
`Port.cardinality` to `pub(crate)`.
**D3.** Replace `register_core_types()` coproduct lists with
DSL-sourced registration via `merge_dsl_types()`.
**D4.** Model transport rewrite tables as data (separate concern,
can be follow-up).

#### Future: Behavior Declarations (not this PR)

**E1.** Add `behavior` and `implements` as DSL constructs.
**E2.** Auto-generate property-based tests from `law` declarations.
**E3.** Migrate `Cardinality`, `ContentEncoding`, `Predicate` lattice
impls from Rust to DSL `implements` clauses.

---

## Verification: Final Acceptance Criteria

When Phases A-D are complete:

**Deletions verified** (all return 0 results from `rg --type rust`):
- `emit_identity_type`
- `map_primitive`
- `try_refined_to_rust`
- `map_to_c_type_static`
- `Guard::` (in non-test code)
- `Port::with_cardinality`

**Structural invariants:**
- Every type in `register_kernel_types()` resolves to a multi-node
  structural DAG (not a single Identity node)
- All type emission flows through structural pattern matching on
  `TypeShape`, not string-name matching
- Adding a new backend requires only defining a `BackendRules` set
- Adding a new type requires only a `.dag` definition
- No type's properties (width, signedness, cardinality) are
  determined by string matching

**Behavioral invariants:**
- `cargo test --workspace` passes
- `cargo clippy --all-targets -- -D warnings` passes
- All codegen tests produce identical output
- All existing `.dag` files compile without warnings
