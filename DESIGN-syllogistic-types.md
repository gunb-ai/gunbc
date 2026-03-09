# Design: Syllogistic Type System

## The Unifying Idea

Types, domain models, and workflows should be the same structure.

Today, extdeps modeling and workflow DAGs already share a common form:
both are `Dag<Op>` defined in `.dag` files, composed through layers,
processed by the same infrastructure. The type system is the outlier —
it uses `TypeId(String)` backed by hardcoded Rust enums, living in a
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
| **Token** | Opaque ordering type for channel I/O | No equivalent — could be `type Token = Unit where brand("Token")` with control edges | **Gap** |

**Assessment**: The syllogistic type system covers XLS's type surface
almost completely. The main gaps are width parametrics (writing
`fn add<N>(a: bits[N], b: bits[N]) -> bits[N]` where N is inferred
from call site) and the token type for channel ordering.

### Layer 2: Operations (~60 XLS IR Opcodes)

XLS has ~60 opcodes. Here's every category and how it maps.

#### Arithmetic (XLS: `add`, `sub`, `neg`, `umul`, `smul`, `udiv`, `sdiv`, `umod`, `smod`)

| Op | XLS | gunbc After Migration |
|----|-----|----------------------|
| Add | `add(x, y)` | `x + y` — `Int implements CommutativeRing { add }` | 
| Sub | `sub(x, y)` | `x - y` — `Ring { add(a, neg(b)) }` |
| Neg | `neg(x)` | `-x` — `Ring { neg }` |
| Mul | `umul/smul` | `x * y` — `Ring { mul }` |
| Div | `udiv/sdiv` | `x / y` — needs `EuclideanDomain` or `DivisionAlgebra` behavior |
| Mod | `umod/smod` | `x % y` — same as div |

**Gap**: Division and modulus. The current `CommutativeRing` behavior
doesn't include division. Need to either add a `EuclideanDomain`
behavior or a simpler `DivMod` behavior. Small addition to
`std/algebra.dag`.

#### Bitwise (XLS: `and`, `or`, `xor`, `not`, `nand`, `nor`)

| Op | XLS | gunbc After Migration |
|----|-----|----------------------|
| And | `and(x, y)` | `x & y` — `Bit implements BooleanAlgebra { meet }` |
| Or | `or(x, y)` | `x \| y` — `BooleanAlgebra { join }` |
| Not | `not(x)` | `~x` — `BooleanAlgebra { complement }` |
| Xor | `xor(x, y)` | `x ^ y` — `BooleanAlgebra { join(meet(a, complement(b)), meet(complement(a), b)) }` or explicit |
| Nand | `nand(x, y)` | `complement(meet(x, y))` — derived from BooleanAlgebra |
| Nor | `nor(x, y)` | `complement(join(x, y))` — derived |

**Gap**: None for single bits. For multi-bit words, need **bitwise
extension** — applying BooleanAlgebra element-wise across a
`List<Bit>`. This is a `behavior Bitwise` that lifts single-bit
BooleanAlgebra to words. Add to `std/bit.dag`.

#### Shifts (XLS: `shll`, `shrl`, `shra`)

| Op | XLS | gunbc After Migration |
|----|-----|----------------------|
| Shift left | `shll(x, amount)` | Needs `Bitwise { shift_left }` behavior |
| Shift right logical | `shrl(x, amount)` | Needs `Bitwise { shift_right_logical }` |
| Shift right arithmetic | `shra(x, amount)` | Needs `Bitwise { shift_right_arithmetic }` |

**Gap**: Shift operations. Not derivable from BooleanAlgebra alone.
Need explicit shift operations in a `Bitwise` behavior extension.

#### Bit Manipulation (XLS: `bit_slice`, `bit_slice_update`, `concat`, `reverse`, `sign_ext`, `zero_ext`, `encode`, `decode`)

| Op | XLS | gunbc After Migration |
|----|-----|----------------------|
| Bit slice `x[3:0]` | `bit_slice(x, start, width)` | Needs `Bitwise { slice }` — structural on `List<Bit>` |
| Bit slice update | `bit_slice_update(x, start, value)` | Needs `Bitwise { slice_update }` |
| Concat | `concat(x, y)` | List append on `List<Bit>` — structural |
| Reverse | `reverse(x)` | List reverse on `List<Bit>` — structural |
| Sign extend | `sign_ext(x, new_width)` | Needs `Signed { sign_extend }` behavior |
| Zero extend | `zero_ext(x, new_width)` | List prepend zeros — structural |
| Encode | `encode(x)` | Needs `Encoding` behavior (one-hot → binary) |
| Decode | `decode(x, width)` | Needs `Encoding` behavior (binary → one-hot) |

**Gap**: Bit slicing, sign extension, encode/decode. These are
specific hardware operations that need explicit behavior definitions.
Concat and reverse are structural on `List<Bit>`.

#### Comparison (XLS: `eq`, `ne`, `ult`, `ugt`, `ule`, `uge`, `slt`, `sgt`, `sle`, `sge`)

| Op | XLS | gunbc After Migration |
|----|-----|----------------------|
| Eq/Ne | `eq(x, y)`, `ne(x, y)` | `x == y`, `x != y` — structural equality |
| Unsigned compare | `ult`, `ugt`, `ule`, `uge` | `UInt implements TotalOrder { leq }` |
| Signed compare | `slt`, `sgt`, `sle`, `sge` | `Int implements TotalOrder { leq }` |

**Gap**: None. `TotalOrder` from `std/algebra.dag` covers all
comparison operations. Signed vs unsigned is distinguished by the
type's signedness predicate.

#### Selection (XLS: `sel`, `one_hot`, `one_hot_sel`, `priority_sel`)

| Op | XLS | gunbc After Migration |
|----|-----|----------------------|
| Select (mux) | `sel(selector, cases, default)` | `match selector { ... }` — existing DSL construct |
| One-hot | `one_hot(input, lsb_prio)` | Needs hardware `Multiplexing` behavior |
| One-hot select | `one_hot_sel(selector, cases)` | Needs hardware `Multiplexing` behavior |
| Priority select | `priority_sel(selector, cases)` | Needs hardware `Multiplexing` behavior |

**Gap**: Hardware multiplexing primitives (one-hot encoding/decoding,
priority selection). These are hardware-domain operations that need
an `extdeps/hardware/multiplexing.dag` module — defined as behaviors
the same way cloud APIs are defined as extdeps.

#### Array/Tuple (XLS: `array`, `array_index`, `array_update`, `array_concat`, `array_slice`, `tuple`, `tuple_index`)

| Op | XLS | gunbc After Migration |
|----|-----|----------------------|
| Array construct | `[a, b, c]` | `[a, b, c]` — existing DSL syntax |
| Array index | `a[i]` | List index — structural |
| Array update | `update(a, i, v)` | Needs list update operation |
| Array concat | `array_concat(a, b)` | List append — structural |
| Array slice | `a[start:end]` | Needs list slice operation |
| Tuple construct | `(a, b)` | `{ a: x, b: y }` — anonymous product |
| Tuple index | `t.0` | `t.a` — field access |

**Gap**: Array/list update and slice. Minor — these are standard
collection operations, not hardware-specific.

#### Channel/State (XLS: `send`, `receive`, `after_all`, Proc state)

| Op | XLS | gunbc After Migration |
|----|-----|----------------------|
| Send | `send(tok, channel, data)` | No direct equivalent today |
| Receive | `recv(tok, channel)` | No direct equivalent today |
| After all | `after_all(tok1, tok2)` | Control edge ordering in DAG — structural |
| Proc state | `next(state) -> state` | `func` with `uses` clauses — conceptual parallel |

**Gap**: **Channels and procs are the biggest gap.** XLS's proc model
(stateful processes communicating via channels with ready/valid
handshake) has no direct equivalent in gunbc. However, this maps
naturally to the existing `func ... uses` pattern:

```dag
// Hypothetical hardware proc in gunbc syntax
func counter() -> { count: UInt32 }
  uses clk: Clock
  state { count: UInt32 = 0 }
  channels {
    increment: chan<Bool> in
    value: chan<UInt32> out
  }
{
  inc = receive(increment)
  next_count = if inc { count + 1 } else { count }
  send(value, next_count)
  return { count: next_count }
}
```

This uses existing DSL constructs (`func`, `uses`, `state`, `return`)
with new `channels` syntax. The `state` block parallels XLS's
`init`/`next` pattern. The `channels` block parallels XLS's channel
declarations. The compiler would lower this to a DAG with
`DataFlow` and `Control` edges — which already exist.

### Layer 3: Scheduling and Timing

| XLS Capability | XLS Implementation | gunbc Equivalent | Gap? |
|---|---|---|---|
| **Clock period** | `--clock_period_ps` flag | No equivalent | **Gap** |
| **Pipeline stages** | `--pipeline_stages` flag | No equivalent | **Gap** |
| **Delay model** | `--delay_model=asap7\|sky130` | No equivalent | **Gap** |
| **Operation latency** | Per-op delay in cycles/picoseconds | No equivalent | **Gap** |
| **Register insertion** | Automatic at stage boundaries | No equivalent | **Gap** |
| **Schedule optimization** | Minimize pipeline registers | No equivalent | **Gap** |

**Gap**: This is the hardest layer. XLS's scheduler is sophisticated —
it uses per-operation delay models (from ASIC libraries like ASAP7 or
SKY130), assigns operations to clock cycles, and inserts pipeline
registers at stage boundaries.

**However**, the syllogistic approach makes this more tractable than
building it from scratch:

1. **Delay models are extdeps.** A delay model IS an external
   specification: "an ASAP7 adder takes 150ps." This is the same
   pattern as "a GCP Secret Manager access is ReadOnly with 200ms
   latency." It belongs in `extdeps/hardware/asap7.dag`:

```dag
module extdeps.hardware.asap7

// Ref: ASAP7 PDK — Arizona State University
// https://github.com/The-OpenROAD-Project/asap7

data gate_delays: List<GateDelay> = [
  { gate: "add",    width: 32, delay_ps: 150 },
  { gate: "mul",    width: 32, delay_ps: 400 },
  { gate: "and",    width: 1,  delay_ps: 20 },
  { gate: "or",     width: 1,  delay_ps: 20 },
  { gate: "mux2",   width: 1,  delay_ps: 30 },
  { gate: "reg",    width: 1,  delay_ps: 50, is_sequential: true },
]
```

2. **Scheduling is a backend concern.** The scheduler reads the
   DAG topology (which operations depend on which), reads the delay
   model (from extdeps), and partitions operations into pipeline
   stages. This is a Verilog backend pass, not a core compiler
   feature. The DAG structure (nodes, edges, data flow) already
   provides the dependency information the scheduler needs.

3. **Pipeline stages map to existing DAG constructs.** gunbc
   already has `Pipeline { stages, stage_names }` in `LoweredOp`.
   Today these are workflow stages. For hardware, they'd be clock-
   cycle stages. Same structure, different domain.

### Layer 4: Verilog Codegen

| XLS Capability | XLS Implementation | gunbc Equivalent | Gap? |
|---|---|---|---|
| **Module declaration** | `module name(ports)` | Product type → module ports (structural) | **Covered** by type system |
| **Port widths** | Derived from `bits[N]` types | Derived from `width(N)` predicate | **Covered** |
| **Wire/reg** | Inferred from combinational vs sequential | Needs `Combinational`/`Sequential` behavior markers | **Gap** (small) |
| **`always_comb`** | For purely combinational logic | `fn` (pure functions) → combinational | **Covered** conceptually |
| **`always_ff`** | For registered (stateful) logic | `func` with `state` → sequential | **Gap** (needs `state` syntax) |
| **Reset logic** | `--reset` / `--reset_active_low` flags | Needs reset behavior on state types | **Gap** (small) |
| **Clock gating** | Automatic for unused registers | Backend optimization pass | **Gap** (backend concern) |
| **Instantiation** | Sub-module instantiation | SubDag → sub-module | **Covered** structurally |

### Summary: The Gap Map

| Category | XLS Opcodes | Covered by Syllogistic Types | Needs New Behavior | Needs Backend Work |
|----------|-------------|-------|------|------|
| **Type system** | `bits[N]`, signed/unsigned, arrays, tuples, structs, enums | 9/10 | Parametric widths | — |
| **Arithmetic** | add, sub, neg, mul, div, mod | 4/6 | DivMod behavior | — |
| **Bitwise** | and, or, xor, not, nand, nor | 6/6 via BooleanAlgebra | Bitwise word-lift | — |
| **Shifts** | shll, shrl, shra | 0/3 | Bitwise { shift } | — |
| **Bit manipulation** | slice, update, concat, reverse, sign_ext, zero_ext, encode, decode | 2/8 structural | 6 ops in Bitwise/Signed | — |
| **Comparison** | eq, ne, unsigned, signed (10 ops) | 10/10 via TotalOrder | — | — |
| **Selection** | sel, one_hot, one_hot_sel, priority_sel | 1/4 (match) | Multiplexing behavior | — |
| **Array/Tuple** | construct, index, update, concat, slice, tuple_index | 4/7 structural | update, slice | — |
| **Channels/Procs** | send, receive, after_all, proc state | 0/4 | Channel behavior + state syntax | — |
| **Scheduling** | clock, pipeline, delay model, register insertion | 0/4 | — | Full backend pass |
| **Codegen** | module, ports, wire/reg, always_comb/ff, reset | 3/7 structural | wire/reg, state, reset | Verilog emitter |

### What This Tells Us

**The syllogistic type system covers ~60% of XLS's capability surface
for free** — types, arithmetic, comparison, basic structure. This is
the part that falls out of the type-by-type migration without any
hardware-specific work.

**Another ~25% is new behaviors** (`Bitwise`, `DivMod`,
`Multiplexing`, channels) that follow the same pattern as everything
else: `.dag` files declaring tautological operations with algebraic
laws. These are `extdeps/hardware/*.dag` modules, no different from
`extdeps/cloud/gcp/*.dag`.

**The remaining ~15% is genuine backend work**: a scheduling pass
(reading delay models from extdeps, partitioning into pipeline
stages), a Verilog emitter (rendering DAGs as `module` declarations
with `always_comb`/`always_ff` blocks), and state/reset handling.

Crucially, that ~15% is *localized to the backend*. It doesn't
require changes to the compiler core, the type system, or the DAG
infrastructure. A Verilog backend is a new emitter — the same way
the Rust, Go, C, and MIPS backends are emitters — that reads DAGs
and renders them as Verilog.

### Path to Parity

Given the execution plan, here's when each XLS capability becomes
available:

| After Step | XLS Parity Gained |
|-----------|-------------------|
| Step 4 (Int/Float) | Fixed-width types, signed/unsigned, arithmetic, comparison |
| Step 6 (containers) | Arrays, tuples, structural concat/reverse |
| Step 7 (emit reads structure) | Backends can derive port widths from type DAGs |
| **New: Hardware behaviors** | Bitwise, shifts, bit slicing, DivMod, Multiplexing |
| **New: Channel/state syntax** | Procs, channels, stateful logic |
| **New: Delay extdeps** | extdeps/hardware/asap7.dag, sky130.dag |
| **New: Verilog backend** | Scheduling pass, Verilog emitter |

Steps 0–10 from the execution plan give us the foundation. After
that, hardware support is additive:

1. **`extdeps/hardware/` modules** (~2–4 weeks): Define `Bitwise`,
   `Multiplexing`, `Schedulable` behaviors. Define delay models for
   ASAP7/SKY130 as extdeps data. Define `Clock`, `Reset` as resource
   types.

2. **Channel and state syntax** (~2 weeks): Add `state { }` and
   `channels { }` blocks to `func`. Parse, typecheck, lower to DAG
   nodes with appropriate edges.

3. **Verilog emitter** (~4–6 weeks): A new emit backend that reads
   DAGs and produces `.v` files. Includes scheduling pass that reads
   delay extdeps to assign pipeline stages.

**Total additional work beyond the type migration: ~8–12 weeks.**
Combined with the ~10–14 week type migration, the full path from
today to Verilog emission is **~20–26 weeks**.

For comparison, XLS has been in development since ~2020 with a
multi-person team. The syllogistic approach gets to parity faster
because the type system, DAG infrastructure, and DSL surface already
exist — we're adding a domain (hardware) to an existing framework,
not building a hardware compiler from scratch.

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

## No Metadata: Structure or Nothing

### The Anti-Pattern

The current IR has a `TypeOp::Meta(MetadataPayload)` node that carries
"inert" information:

```rust
pub enum MetadataPayload {
    SystemId(String),
    SystemKind(String),
    BehaviorId(String),
    Invocation(String),
    Property(String),
    InputContract { name, type_id, required },
    OutputContract { name, type_id },
    PlatformRepr(PlatformRepr),   // bits, signed, float, discrete
}
```

`Meta` is explicitly documented as "non-semantic, non-failing" — it's
traversable but must not change runtime behavior. This is the escape
hatch. When something is true about a type but can't be expressed as
structure, it gets stuffed into `Meta`.

`PlatformRepr` is the clearest example. It carries `{ bits: 64,
signed: true, float: false, discrete: true }` — four fields that
tell backends "this is `i64`." But this information is *metadata*,
not *structure*. It's an annotation someone attached, not a
consequence of composition. Nothing in the type DAG *derives*
`bits: 64` from the type's structural definition. It's just... there.

`SystemId`, `SystemKind`, `BehaviorId`, `Invocation`, `Property` —
these are all the same problem. They're truths about a node that
aren't expressed as DAG structure.

### The Principle

**If it's true, it's structure. If it's structure, it's in the DAG.**
There is no metadata. There is no sidecar. If a type is 64 bits wide,
that fact is a `width(64)` predicate on a structural node, derivable
from the composition: `Word64 = 8 × Byte = 8 × (8 × Bit) = 64 × Bit`.
If a system has `ReadOnly` behavior, that fact is an `implements`
clause on the service type, not a `Property(String)` metadata blob.

The `Meta` node variant should be eliminated. Everything currently in
`MetadataPayload` either:

1. **Becomes structural** — `PlatformRepr` becomes derivable from
   type composition (width × signedness × domain). `SystemKind`
   becomes a type relationship (service type implements a behavior).

2. **Moves to the DSL** — `BehaviorId`, `Invocation`, `Property`
   become `data` declarations or `behavior` implementations in `.dag`
   files, same as extdeps behavioral data.

3. **Disappears** — if a piece of information can't be expressed
   structurally or as a DSL declaration, it probably shouldn't exist.

### `PlatformRepr` Specifically

Today:

```rust
PlatformRepr { bits: 64, signed: true, float: false, discrete: true }
```

This is metadata attached to `Int` to tell backends "emit `i64`."

After the migration, there is no `PlatformRepr`. Instead:

```dag
type Int64 = Word64 where signed(twos_complement)
```

The backend walks the type DAG:
- `Int64` → `Word64` → `8 × Byte` → `8 × (8 × Bit)` → width is
  `8 × 8 = 64` (derived from structure)
- `signed(twos_complement)` → signed (declared as predicate)
- `Word64` → discrete (bit-based types are always discrete)
- Not float (no float-domain marker in the chain)

Every field of `PlatformRepr` is now a *derived consequence* of
structure, not an opaque annotation. The backend computes the same
`{ bits: 64, signed: true, float: false, discrete: true }` tuple,
but it computes it by walking the DAG, not by reading a metadata blob.

## Stacking Tautologies: The General Mechanism

### The Insight

Cardinality, set algebra, content encoding, predicate entailment —
these aren't special cases to be individually migrated. They're
*examples* of a general capability: **any tautological behavior should
be stackable onto a DAG node, and the system should enforce it
through testing**.

Today the codebase has ~14 algebraic structures (cardinality lattice,
content encoding lattice, predicate entailment, presence ordering,
access modes, guards, fermi depth, etc.). Each is implemented as a
bespoke Rust type with hand-written algebraic laws. The algebra
module (`algebra.rs`) defines the right traits — `PartialOrder`,
`JoinSemilattice`, `MeetSemilattice`, `Lattice`, `BoundedLattice` —
but only two types implement them (`Cardinality` and
`ContentEncoding`). Everything else has implicit algebraic structure
encoded in ad-hoc match statements.

The problem isn't that cardinality is "outside the DAG." The problem
is that the system has no *general mechanism* for:

1. Declaring a tautological behavior in the DSL
2. Attaching it to a type (or a node, or an edge)
3. Having the compiler enforce it
4. Having tests verify the algebraic laws automatically

Cardinality is just one behavior. Width is another. Signedness is
another. Ordering is another. The system should handle all of them
the same way — as stackable tautologies on the DAG.

### What a Tautology Is

In the extdeps model, a tautology is a definition that's true by
construction: "a CAS mechanism is one of: generation-based,
ETag-based, version ID, or row version." You don't prove it. You
define it. Everything downstream inherits it.

The same applies to type behaviors:

- "Cardinality is an interval [min, max] on ℕ ∪ {∞} with join,
  meet, product, and sum operations." — tautology
- "A bit has width 1." — tautology
- "Signed arithmetic uses two's complement." — tautology
- "ASCII is a subtype of UTF8." — tautology
- "A list has cardinality [0, ∞)." — tautology

Each of these is a *truth you attach to a type DAG*. Each carries
algebraic laws (lattice, ordering, entailment). Each should be
expressible in the DSL and enforceable by the compiler.

### The Mechanism: Behaviors as DAG-Attached Truths

A behavior is a named set of algebraic laws attached to a type or
type family. In the DSL:

```dag
// See std/algebra.dag in the Future State section for the complete
// hierarchy: Magma → Semigroup → Monoid → Group → AbelianGroup,
// Ring → CommutativeRing → IntegralDomain → Field,
// PartialOrder → Lattice → BoundedLattice → BooleanAlgebra.
//
// References: Lang "Algebra" (2002), Davey & Priestley "Introduction
// to Lattices and Order" (2002), Lean mathlib naming conventions.
```

These are the standard algebraic laws from graduate mathematics,
faithfully transcribed. The difference from today: they'd be
expressed *in the DSL* as `behavior` declarations, not in Rust trait
definitions. See the Future State section for the full hierarchy.

### Attaching Behaviors to Types

A type acquires behaviors by declaration. The behaviors stack:

```dag
module std.cardinality

import std.algebra { BoundedLattice, Semiring }

// Cardinality carries BoundedLattice + Semiring behaviors.
// The compiler enforces all laws from both behaviors.
type Cardinality {
  min: Int where range(min: 0)
  max: Int?
}
  implements BoundedLattice {
    join(a, b) = { min: min(a.min, b.min), max: interval_max(a.max, b.max) }
    meet(a, b) = ...
    top = { min: 0, max: null }
  }
  implements Semiring {
    product(a, b) = { min: a.min * b.min, max: interval_mul(a.max, b.max) }
    sum(a, b) = { min: a.min + b.min, max: interval_add(a.max, b.max) }
    one = { min: 1, max: 1 }
    zero = { min: 0, max: 0 }
  }
```

The same mechanism works for any domain:

```dag
module std.encoding

import std.algebra { BoundedLattice }

type ContentEncoding
  = ASCII | UTF8 | Latin1 | Text | Binary | Unknown
  implements BoundedLattice {
    // Subtype ordering declared as data:
    ordering = [
      ASCII <= UTF8,
      UTF8 <= Text,
      Latin1 <= Text,
      Text <= Unknown,
      Binary <= Unknown,
    ]
    top = Unknown
  }
```

And for entirely new domains that don't exist yet:

```dag
module hardware.timing

import std.algebra { PartialOrder }

type Latency {
  cycles: Int where range(min: 0)
  pipeline_stages: Int where range(min: 1)
}
  implements PartialOrder {
    a <= b = a.cycles <= b.cycles
  }

behavior Schedulable {
  operation delay(self) -> Latency
  law bounded: delay(self).cycles >= 0
  law composable: delay(a >> b).cycles == delay(a).cycles + delay(b).cycles
}
```

### Test Generation from Behaviors

This is where "enforced via testing" comes in. When a type declares
`implements BoundedLattice`, the compiler can *automatically generate*
the property-based tests that `algebra.rs` currently has by hand:

```rust
// AUTO-GENERATED from: Cardinality implements BoundedLattice
proptest! {
    #[test]
    fn cardinality_partial_order_reflexive(a in arb_cardinality()) {
        prop_assert!(a.leq(&a));
    }
    #[test]
    fn cardinality_join_commutative(a in arb_cardinality(), b in arb_cardinality()) {
        prop_assert_eq!(a.join(b), b.join(a));
    }
    #[test]
    fn cardinality_top_is_top(a in arb_cardinality()) {
        prop_assert!(a.leq(&Cardinality::top()));
    }
    // ... all BoundedLattice + Semiring laws ...
}
```

Today these tests are written by hand in `algebra.rs` and
`type_op.rs`. With declarative behaviors, the *behavior definition*
carries the laws, and the *test generator* produces the property tests
from any `implements` declaration. Adding a new behavior to an
existing type automatically generates and runs the corresponding
tests.

This is the same pattern as testgen for service operations: the
`OperationBehavior` data in extdeps drives test generation for
transport contracts. Algebraic behaviors would drive test generation
for type contracts.

### What "Stacking Tautologies" Looks Like

A fully compositional type stacks multiple tautological behaviors:

```dag
type Int64 = Word64
  where signed(twos_complement), arithmetic
  implements BoundedLattice {
    // Integer ordering: the usual <=
    top = max_int64
  }
  implements Semiring {
    // Arithmetic: +, *
    product(a, b) = a * b
    sum(a, b) = a + b
    one = 1
    zero = 0
  }
  implements Bitwise {
    // Bit operations
    and(a, b) = ...
    or(a, b) = ...
    xor(a, b) = ...
    shift_left(a, n) = ...
    shift_right(a, n) = ...
  }
```

Each `implements` clause is a tautology stacked on top of the
structural definition. The structural definition says "64 bits, signed,
two's complement." The behaviors say "you can order these, do
arithmetic, and do bitwise operations." The compiler enforces all
laws from all behaviors. The backends read the structural properties
AND the behaviors to decide how to emit.

A Verilog backend sees `Bitwise` and emits `&`, `|`, `^`, `<<`, `>>`.
A Rust backend sees the same and emits the same operators with
different syntax. A test generator sees `Semiring` and generates
identity/absorbing/commutativity tests. Nobody hardcodes what `Int64`
can do — the `.dag` file declares it, and everything follows.

### The Current State as a Starting Point

The codebase already has the *right ideas* in the wrong places:

| What exists | Where | What it should become |
|-------------|-------|---------------------|
| `algebra.rs` traits | Rust trait definitions | `std/algebra.dag` behavior definitions |
| `Cardinality` lattice impl | Rust `impl` blocks | `Cardinality implements BoundedLattice` in `.dag` |
| `ContentEncoding` lattice impl | Rust `impl` + match arms | `ContentEncoding implements BoundedLattice` in `.dag` |
| `Predicate::entails()` | Rust method | `Predicate implements PartialOrder` in `.dag` |
| Property-based tests in `algebra.rs` | Hand-written proptest | Auto-generated from `behavior` law declarations |
| `TypeContract` composite lattice | Rust struct + impls | Emerges from stacked behaviors on types |

The Rust `algebra.rs` traits don't go away. They become the
*compiled representation* of what the `.dag` behaviors declare — the
same way `Dag<LoweredOp>` is the compiled representation of `.dag`
workflow files. The DSL declares the truths. The compiler verifies and
compiles them. The Rust types are the efficient runtime form.

### Relationship to Cardinality Specifically

Cardinality is the clearest example of the current split. Today it
exists in three places that can drift:

1. `Port.cardinality` — manually set at construction
2. `TypeOp::Wrap(WrapperKind)` — structural truth in the type DAG
3. `TypeRegistry::infer_cardinality()` — bridge that derives (1) from (2)

The system even has `audit_cardinality_drift()` to detect when these
disagree.

In the tautology-stacking model, this collapse is straightforward.
`List<T>` is a type whose DAG says "collection of T with cardinality
[0, ∞)." The cardinality isn't a separate annotation — it's a
structural consequence of the type's behavior declarations. The
`Cardinality` struct stays as the efficient internal kernel, but it's
always *derived* from the DAG, never independently declared.

The `audit_cardinality_drift()` function becomes unnecessary. The
dual-source problem disappears. There's one truth (the type DAG with
its stacked behaviors) and one derivation path.

### Guards and Predicates: Same Tautology

Guards on edges and predicates on type DAGs are the same thing:
truth assertions that gate data flow. Today `Guard` has only
`Eq(Value)` / `NotEq(Value)`, while `Predicate` has the full algebra
(`InRange`, `Matches`, `And`, `Or`, `Not`, `Content`, `NonEmpty`).

In the tautology model, there's no distinction. An edge carries a
predicate. A type node carries a predicate. Both assert a truth.
Both use the same algebra. Both are tested by the same mechanism.
`Guard` as a separate concept disappears.

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

#### Phase 8: Behavior Declarations in DSL (2 weeks)

Add `behavior` and `implements` as DSL constructs. Parse `behavior`
definitions with `law` and `operation` clauses. Parse `implements`
clauses on type definitions.

```
Touches: parser.rs (new syntax), lib.rs (AST nodes), typecheck
         (validate implements clauses against behavior contracts)
Lines changed: ~400
Risk: Medium — new syntax, but purely additive
Test: behavior/implements parse and validate; no codegen yet
```

This is the foundation. Once behaviors are expressible in the DSL,
everything else is incremental.

#### Phase 9: Test Generation from Behaviors (2 weeks)

The compiler reads `behavior` law declarations and `implements`
clauses, then auto-generates property-based tests verifying the
algebraic laws. Replaces the hand-written proptests in `algebra.rs`.

```
Touches: testgen (new law-driven test generator), algebra.rs
         (hand-written tests become auto-generated)
Lines changed: ~500 (new generator), ~300 (deleted hand-written tests)
Risk: Medium — generated tests must match hand-written coverage
Test: Auto-generated tests for Cardinality and ContentEncoding
      produce equivalent coverage to existing hand-written tests
```

After this, declaring `implements BoundedLattice` on a type
automatically produces and runs reflexivity, transitivity,
commutativity, absorption, and top-element tests.

#### Phase 10: Migrate Existing Algebras to Behaviors (1 week)

Move `Cardinality`, `ContentEncoding`, and `Predicate` lattice
implementations from Rust trait impls to DSL `implements` clauses.
Unify `Guard` with `Predicate`. Derive port cardinality from type
DAGs.

```
Touches: algebra.rs (Rust impls become thin wrappers over DAG-derived
         behavior), type_op.rs (ContentEncoding lattice → .dag),
         dag.rs (Guard → Predicate, Port.cardinality derived)
Lines changed: ~400 (mostly moving, some deleting)
Risk: Medium — behavioral equivalence, different source of truth
Test: All existing algebraic property tests pass via auto-generation
```

After this, the 14 ad-hoc algebraic structures are expressed as
stacked tautologies in `.dag` files, enforced by auto-generated
tests, and compiled to efficient Rust representations.

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
| 8. Behavior declarations in DSL | 2 weeks | Medium | Phase 1 |
| 9. Test generation from behaviors | 2 weeks | Medium | Phase 8 |
| 10. Migrate existing algebras to behaviors | 1 week | Medium | Phase 9 |
| **Total** | **~15 weeks** | | |

Phases 1-2 are safe, incremental, and independently valuable. They
can ship without committing to the full migration.

Phases 3-4 are the structural watershed. Once Products embed their
fields as SubDags and TypedBinding carries TypeId, the system is
fundamentally DAG-structural. Everything after is cleanup and payoff.

Phases 5-7 are where the payoff materializes: backends become
domain-agnostic, primitives move to the DSL, and string matching
disappears.

Phases 8-10 are the generalization: the system gains a universal
mechanism for attaching and enforcing arbitrary tautological
behaviors. Cardinality, encoding, ordering, arithmetic — anything
with algebraic laws — becomes a stackable behavior declaration in
`.dag`, enforced by auto-generated tests, compiled to efficient Rust.
Phase 8 (behavior declarations) can start as early as Phase 1 since
it's additive syntax.

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

- **Arbitrary behaviors stack as tautologies.** Cardinality, encoding,
  ordering, arithmetic, bitwise operations — any algebraic behavior
  is a `behavior` declaration in `.dag` with `law` clauses. Types
  acquire behaviors via `implements`. The compiler auto-generates
  property-based tests from the laws. Adding a new algebra to an
  existing type means adding an `implements` clause and getting
  tests for free. No Rust changes.

- **Guards and predicates are the same thing.** Type validation
  ("this string must match a URL pattern") and edge gating ("this
  branch fires when condition is true") use the same `Predicate`
  algebra. One mechanism for all truth assertions.

- **No metadata.** There is no `Meta` node, no `PlatformRepr`, no
  `MetadataPayload`. If something is true, it's structure. If it's
  structure, it's in the DAG. Backends derive everything by walking
  the DAG — bit width from composition, signedness from predicates,
  domain from type ancestry.

---

## Future State: What the `.dag` World Looks Like

The existing `.dag` files are not sacred. If this design is right,
they can be rewritten to match. What follows is a sketch of the
end state — the full stack from foundational logic through to a
tool like `gist.dag`, with no metadata, no hardcoded primitives, and
every truth expressed as DAG structure.

The foundational layers (logic, algebra) reference real mathematical
standards. This isn't a toy sketch — it should compose correctly
when we eventually build out multi-valued logics, algebraic number
theory, or hardware timing algebras. Getting the foundations right
means those extensions are instantiation, not invention.

### Layer 0: Logic (`std/logic.dag`)

```dag
module std.logic

// Propositional logic foundations.
//
// References:
//   Classical logic: Enderton, "A Mathematical Introduction to Logic" (2001)
//   Belnap four-valued: Belnap, "A useful four-valued logic" (1977)
//   Kleene three-valued: Kleene, "Introduction to Metamathematics" (1952)
//
// We define multiple logic systems as tautological types. Each is a
// truth value domain with its own connectives. Classical is the default.
// Multi-valued logics compose with classical — a Belnap value can be
// projected to classical by collapsing its information ordering.

// ── Classical (two-valued) logic ────────────────────────────────────
// The standard Boolean domain: {⊤, ⊥} with ¬, ∧, ∨.
// Ref: Enderton §1.1 "Sentential Logic"

type Classical = True | False

// Classical connectives. Each is a total function on Classical values.
fn classical_not(a: Classical) -> Classical {
  match a { True => False, False => True }
}
fn classical_and(a: Classical, b: Classical) -> Classical {
  match (a, b) { (True, True) => True, _ => False }
}
fn classical_or(a: Classical, b: Classical) -> Classical {
  match (a, b) { (False, False) => False, _ => True }
}
fn classical_xor(a: Classical, b: Classical) -> Classical {
  match (a, b) { (True, False) => True, (False, True) => True, _ => False }
}
fn classical_nand(a: Classical, b: Classical) -> Classical {
  classical_not(classical_and(a, b))
}
fn classical_implies(a: Classical, b: Classical) -> Classical {
  classical_or(classical_not(a), b)
}

// ── Kleene (three-valued) logic ─────────────────────────────────────
// Adds Unknown (⊥_k) for partial information.
// Ref: Kleene (1952) §64 "Three-valued logic"
//
// Truth table for ∧:  T∧U=U, U∧F=F, U∧U=U
// This models "we don't know yet" — useful for hardware X-states,
// uninitialized memory, and speculative evaluation.

type Kleene = KTrue | KFalse | KUnknown

// ── Belnap–Dunn (four-valued) logic ─────────────────────────────────
// Adds Both (⊤_b) for contradictory information.
// Ref: Belnap, "A useful four-valued logic" in Dunn & Epstein (1977)
//
// Four values arranged in two orderings:
//   Truth ordering:    False ≤ {Unknown, Both} ≤ True
//   Information ordering: Unknown ≤ {True, False} ≤ Both
//
// Useful for: database nullability, sensor fusion, conflict detection.

type Belnap = BTrue | BFalse | BUnknown | BBoth
```

This is the real bottom. Classical logic is the standard two-valued
system. Kleene and Belnap extend it for domains where partial or
contradictory information matters (hardware X-states, database nulls,
speculative execution). Each is a tautological definition. They
compose: a Belnap value projects to Classical by mapping `BUnknown`
and `BBoth` to `False` (conservative) or by requiring resolution.

### Layer 0: Algebra (`std/algebra.dag`)

```dag
module std.algebra

// Standard algebraic structure hierarchy.
//
// References:
//   General: Lang, "Algebra" (2002), Chapters I–IV
//   Lattice theory: Davey & Priestley, "Introduction to Lattices
//     and Order" (2002)
//   Universal algebra: Burris & Sankappanavar, "A Course in
//     Universal Algebra" (1981), freely available
//   Conventions: following the Lean mathlib naming where possible
//     (https://leanprover-community.github.io/mathlib4_docs/)
//
// The hierarchy:
//
//   Magma                (closed binary operation)
//     │
//   Semigroup            (+ associativity)
//     │
//   Monoid               (+ identity element)
//     │
//   Group                (+ inverse element)
//     │
//   AbelianGroup         (+ commutativity)
//
//   Semiring             (two operations: additive AbelianMonoid +
//                         multiplicative Monoid + distributivity)
//     │
//   Ring                 (additive AbelianGroup + multiplicative Monoid)
//     │
//   CommutativeRing      (+ multiplicative commutativity)
//     │
//   IntegralDomain       (+ no zero divisors)
//     │
//   Field                (+ multiplicative inverse for nonzero)
//
//   PartialOrder         (reflexive, transitive, antisymmetric)
//     │
//   JoinSemilattice      (+ least upper bound)
//     │
//   MeetSemilattice      (+ greatest lower bound)
//     │
//   Lattice              (both join and meet + absorption)
//     │
//   BoundedLattice       (+ top and/or bottom elements)
//     │
//   BooleanAlgebra       (+ complement + distributivity)

// ── Order structures ────────────────────────────────────────────────

// Ref: Davey & Priestley §1.1–1.3
behavior Preorder {
  operation leq(a, b) -> Bool
  law reflexive:  leq(a, a)
  law transitive: leq(a, b), leq(b, c)  implies  leq(a, c)
}

// Ref: Davey & Priestley §1.4
behavior PartialOrder extends Preorder {
  law antisymmetric: leq(a, b), leq(b, a)  implies  a == b
}

behavior TotalOrder extends PartialOrder {
  law total: leq(a, b) or leq(b, a)
}

// ── Lattice structures ──────────────────────────────────────────────

// Ref: Davey & Priestley §2.1–2.3
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

// Ref: Davey & Priestley §2.8
behavior Lattice extends JoinSemilattice, MeetSemilattice {
  law absorption_join: join(a, meet(a, b)) == a  when meet(a, b) exists
  law absorption_meet: meet(a, join(a, b)) == Some(a)
}

// Ref: Davey & Priestley §2.10
behavior BoundedLattice extends Lattice {
  element top
  element bottom
  law top_is_top:       leq(a, top)
  law bottom_is_bottom: leq(bottom, a)
}

// Ref: Davey & Priestley §4.5 — Boolean algebras
behavior BooleanAlgebra extends BoundedLattice {
  operation complement(a) -> Self
  law complement_join: join(a, complement(a)) == top
  law complement_meet: meet(a, complement(a)) == Some(bottom)
  law distributive: meet(a, join(b, c)) == join(meet(a, b), meet(a, c))
}

// ── Algebraic structures ────────────────────────────────────────────

// Ref: Lang §I.1
behavior Magma {
  operation op(a, b) -> Self
  law closed: true   // closure is structural (op returns Self)
}

// Ref: Lang §I.1
behavior Semigroup extends Magma {
  law associative: op(op(a, b), c) == op(a, op(b, c))
}

// Ref: Lang §I.2
behavior Monoid extends Semigroup {
  element identity
  law left_identity:  op(identity, a) == a
  law right_identity: op(a, identity) == a
}

// Ref: Lang §I.2
behavior Group extends Monoid {
  operation inverse(a) -> Self
  law left_inverse:  op(inverse(a), a) == identity
  law right_inverse: op(a, inverse(a)) == identity
}

// Ref: Lang §I.2
behavior AbelianGroup extends Group {
  law commutative: op(a, b) == op(b, a)
}

// ── Ring-like structures ────────────────────────────────────────────

// Ref: Lang §II.1
// A ring has two operations: (R, +, 0) is an abelian group,
// (R, *, 1) is a monoid, and * distributes over +.
behavior Ring {
  operation add(a, b) -> Self
  operation mul(a, b) -> Self
  operation neg(a) -> Self
  element zero
  element one
  // Additive abelian group
  law add_commutative:  add(a, b) == add(b, a)
  law add_associative:  add(add(a, b), c) == add(a, add(b, c))
  law add_identity:     add(a, zero) == a
  law add_inverse:      add(a, neg(a)) == zero
  // Multiplicative monoid
  law mul_associative:  mul(mul(a, b), c) == mul(a, mul(b, c))
  law mul_identity:     mul(a, one) == a
  // Distributivity
  law left_distribute:  mul(a, add(b, c)) == add(mul(a, b), mul(a, c))
  law right_distribute: mul(add(a, b), c) == add(mul(a, c), mul(b, c))
}

// Ref: Lang §II.1
behavior CommutativeRing extends Ring {
  law mul_commutative: mul(a, b) == mul(b, a)
}

// Ref: Lang §II.2
behavior IntegralDomain extends CommutativeRing {
  law no_zero_divisors: mul(a, b) == zero implies (a == zero or b == zero)
}

// Ref: Lang §II.2
behavior Field extends IntegralDomain {
  operation reciprocal(a) -> Self   // partial: undefined for zero
  law mul_inverse: a != zero implies mul(a, reciprocal(a)) == one
}
```

This is the standard algebraic hierarchy from any graduate algebra
textbook, faithfully transcribed. The `behavior` declarations
reference Lang (2002) for algebraic structures and Davey & Priestley
(2002) for order/lattice theory. The naming follows Lean's mathlib
conventions where possible for interoperability with formal methods.

The hierarchy composes correctly:
- `BooleanAlgebra` extends `BoundedLattice` — this is how hardware
  combinational logic works (classical propositional logic IS a
  Boolean algebra)
- `Field` extends `IntegralDomain` extends `CommutativeRing` extends
  `Ring` — this is how floating-point types would be modeled
- `AbelianGroup` appears as the additive structure inside `Ring` —
  integer addition is an abelian group

### Layer 1: Bits (`std/bit.dag`)

```dag
module std.bit

import std.logic { Classical }
import std.algebra { BooleanAlgebra }

// Ref: IEEE 1364 (Verilog) §3.1 "Value set" — the four-valued
// system {0, 1, x, z}. For classical logic we use the two-valued
// subset {0, 1}.
//
// A bit is a classical truth value with physical width 1.
// It inherits BooleanAlgebra — the standard algebraic structure
// for combinational logic.

type Bit = Classical where width(1)
  implements BooleanAlgebra {
    join(a, b) = classical_or(a, b)
    meet(a, b) = classical_and(a, b)
    complement(a) = classical_not(a)
    top = True
    bottom = False
  }

// Fixed-size aggregates. Width is derivable from composition.
type Nibble  = { bits: List<Bit> where length(4) }    // width: 4
type Byte    = { bits: List<Bit> where length(8) }    // width: 8
type Word16  = { bytes: List<Byte> where length(2) }  // width: 16
type Word32  = { bytes: List<Byte> where length(4) }  // width: 32
type Word64  = { bytes: List<Byte> where length(8) }  // width: 64
type Word128 = { bytes: List<Byte> where length(16) } // width: 128
```

Because `Bit` implements `BooleanAlgebra`, the compiler auto-generates
tests verifying complement, distributivity, De Morgan's laws, etc.
A Verilog backend sees `BooleanAlgebra` and emits `&`, `|`, `~`. This
isn't special-cased — it follows from the algebra.

### Layer 1: Encoding (`std/encoding.dag`)

```dag
module std.encoding

import std.algebra { BoundedLattice }

// Ref: Unicode Standard §2.3 "Encoding Forms" — UTF-8, UTF-16, etc.
// Ref: ISO 8859-1:1998 — Latin-1
// Ref: RFC 20 / ANSI X3.4 — ASCII

type Encoding = ASCII | UTF8 | Latin1 | Text | Binary | Unknown
  implements BoundedLattice {
    // Subtype ordering. ASCII is a strict subset of UTF8, etc.
    ordering = [
      ASCII <= UTF8, UTF8 <= Text,
      Latin1 <= Text,
      Text <= Unknown, Binary <= Unknown,
    ]
    top = Unknown
    bottom = ASCII
  }
```

### Layer 2: Integers (`std/integer.dag`)

```dag
module std.integer

import std.bit { Byte, Word16, Word32, Word64 }
import std.algebra { CommutativeRing, TotalOrder }

// Ref: ISO/IEC 10967:2012 "Language independent arithmetic"
//   Part 1: Integer and floating-point arithmetic
// Ref: Two's complement: adopted as the only signed representation
//   in C2x (ISO/IEC 9899:2023 §6.2.6.2)

type UInt8  = Byte   where unsigned
type UInt16 = Word16 where unsigned
type UInt32 = Word32 where unsigned
type UInt64 = Word64 where unsigned

type Int8  = Byte   where signed(twos_complement)
type Int16 = Word16 where signed(twos_complement)
type Int32 = Word32 where signed(twos_complement)
type Int64 = Word64 where signed(twos_complement)

// Integers form a commutative ring (addition, multiplication,
// negation, but no general multiplicative inverse).
// They also have a total order.
type Int = Int64
  implements CommutativeRing {
    add(a, b) = intrinsic_add(a, b)
    mul(a, b) = intrinsic_mul(a, b)
    neg(a) = intrinsic_neg(a)
    zero = 0
    one = 1
  }
  implements TotalOrder

// Unsigned integers: a commutative semiring (no negation).
type UInt = UInt64
  implements TotalOrder
```

### Layer 2: Floating Point (`std/float.dag`)

```dag
module std.float

import std.bit { Word32, Word64 }
import std.algebra { Field, TotalOrder }

// Ref: IEEE 754-2019 "Floating-Point Arithmetic"
//   §3.3 "Binary interchange format"
//   binary32: 1 sign + 8 exponent + 23 significand
//   binary64: 1 sign + 11 exponent + 52 significand

type Float32 = Word32 where ieee754(binary32)
type Float64 = Word64 where ieee754(binary64)

// Floats approximate a field. IEEE 754 arithmetic is not associative
// (due to rounding), so Field laws are approximate — the compiler
// generates tests with epsilon tolerance.
type Float = Float64
  implements Field {
    add(a, b) = intrinsic_fadd(a, b)
    mul(a, b) = intrinsic_fmul(a, b)
    neg(a) = intrinsic_fneg(a)
    reciprocal(a) = intrinsic_fdiv(one, a)
    zero = 0.0
    one = 1.0
    approximate = true   // laws hold within epsilon
  }
```

### Layer 2: Strings and Containers

```dag
module std.string

import std.bit { Byte }
import std.encoding { Encoding }

// Ref: Unicode Standard §2.7 "Unicode Strings"
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

import std.integer { Int, UInt8 }
import std.string { String, Char }
import std.containers { List, Option, Map }
import std.bit { Byte }
import std.logic { Classical }

type Bool = Classical
type Bytes = List<UInt8>
type Json = String | Int | Bool | Float | List<Json> | Map<String, Json>

type Url        = String where non_empty, pattern("^https?://")
type FilePath   = String where non_empty
type Email      = String where pattern("^[^@]+@[^@]+\\.[^@]+$")
type Port       = Int where range(min: 1, max: 65535)
type HttpStatus = Int where range(min: 100, max: 599)
type CommitSha  = String where pattern("^[a-f0-9]{40}$")
type Secret     = String where brand("Secret")
```

### Layer 5: Tools (`tools/gist.dag` — unchanged)

```dag
module tools.gist

import extdeps.git
import extdeps.github.auth { github_token }
import extdeps.github.gists
import std.resources { Network }
import std.types { CommitSha, Url, Bool }

func gist(public: Bool = false) -> { url: Url }
  uses net: Network
{
  branch_info = git.Core.CurrentBranch()
  listing = git.Core.LsFiles()
  // ... exactly the same as today ...
}
```

**`gist.dag` doesn't change.** The tool layer imports types and uses
them. The revolution is below — `Bool` resolves to `Classical`
(a coproduct of `True | False` from `std/logic.dag`), `Int` resolves
through `Int64 → Word64 → 8 × Byte → 64 × Bit → Classical`, `String`
is a sequence of bytes with an encoding. The tool code is untouched.

### What a Verilog Tool Would Look Like

```dag
module tools.blink

import std.bit { Bit }
import std.integer { UInt32 }
import extdeps.hardware.clock { Clock }

func blink(period: UInt32) -> { led: Bit }
  uses clk: Clock
{
  counter = UInt32 where range(min: 0, max: period)
  tick = clk.Tick()
  next_count = if counter == period {
    { count: 0, toggle: true }
  } else {
    { count: counter + 1, toggle: false }
  }
  counter = next_count.count
  led = if next_count.toggle { complement(led) } else { led }
  return { led: led }
}
```

`complement(led)` works because `Bit` implements `BooleanAlgebra`.
The Verilog backend sees `BooleanAlgebra.complement` and emits `~`.
The Rust backend sees the same and emits `!`. The `.dag` file doesn't
name a backend — it uses algebraic operations that any backend can
render.

### Composition Guarantee

The mathematical foundations compose correctly because they follow
real algebraic conventions:

- `Bit` implements `BooleanAlgebra` → hardware combinational logic
- `Int64` implements `CommutativeRing` → integer arithmetic
- `Float64` implements `Field` (approximate) → floating-point math
- `ContentEncoding` implements `BoundedLattice` → subtype hierarchies
- `Cardinality` implements `BoundedLattice` + `Semiring` semantics →
  multiplicity algebra

If we later add `Complex = { real: Float64, imag: Float64 }`, it
would implement `Field` and get all the field laws tested for free.
If we add `Matrix<N, M, T>` where `T` implements `Ring`, it would
implement `Ring` itself (matrix multiplication is a ring). The
algebraic hierarchy doesn't need to be extended — these are standard
instantiations of the structures already defined.

The logic systems compose the same way. A Belnap four-valued bit
(`BelnapBit = Belnap where width(1)`) could model Verilog's X/Z
states. It wouldn't implement `BooleanAlgebra` (Belnap logic isn't
Boolean) but it would implement `BoundedLattice` on the information
ordering. A simulator backend would read the lattice structure and
generate the correct four-valued truth tables. No compiler changes —
just a new `.dag` file with different algebra attachments.

---

## Execution Plan: Type-by-Type Migration

The earlier migration scope describes the abstract phases. This
section is the concrete plan: which compiler changes come first,
which types migrate in which order, and what "done" looks like at
each step. The strategy is compiler-first, type-by-type, always
green.

### What the Current `.dag` Code Actually Uses

Audit of all `.dag` files shows the minimal type surface:

**8 primitives** (by frequency): String (~350), Int (~120),
Bool (~80), Secret (~45), Json (~25), Float (~15), Bytes (~8),
Unit (0 — unused)

**3 containers**: `List<T>` (~95), `T?` / `Option<T>` (~60),
`Map<K,V>` (~5)

**~14 refinements** (all `T where pred`): FilePath, NonEmptyStr,
Timestamp, Url, CommitSha, Milliseconds, GitRef, Char, ProjectId,
ServiceAccountEmail, GistId, MimeType, TextFilePath, BinaryFilePath

**~5 operations on values**: string ops (split, join, interpolation),
int arithmetic (+, -, *, /, comparisons), bool logic (if, &&, ||, !),
list ops (map, filter, fold, for, join, count, contains), record
field access/construction

**~6 sum types used heavily**: FermiDepth, ContentEncoding, EntryKind,
AuthScheme, Tier, Platform

This is the scope. Everything else (OperationBehavior, GCP types,
LLM types, etc.) is domain products/coproducts that compose from these
foundations. They don't need special migration — once the foundations
work, they come along for free.

### The Minimal Foundation Files

For today's codebase, we need exactly this much foundation:

```
std/logic.dag        Classical only (Bool = Classical)
std/algebra.dag      PartialOrder, TotalOrder, CommutativeRing
                     (just enough for Int ordering and arithmetic)
std/bit.dag          Bit, Byte, Word32, Word64
std/integer.dag      Int8..64, UInt8..64, Int = Int64
std/float.dag        Float64, Float = Float64
std/string.dag       String, Char
std/containers.dag   List<T>, Option<T>, Map<K,V>
std/encoding.dag     ContentEncoding implements BoundedLattice
```

Kleene, Belnap, Group, Field, BooleanAlgebra — all of that stays
in the design as documented but isn't needed until we target hardware
or formal verification. The foundation files are structured so those
extensions are additive.

### Migration Order: Compiler First, Then Types

Each step produces a compiling, testing, green codebase. No big bang.

#### Step 0: Parser accepts new predicates (no behavior change)

Add `width`, `length`, `signed`, `unsigned`, `domain` as first-class
`Refinement` variants in the parser. They already parse today via the
generic `Predicate(String)` fallback — this just promotes them.

```
Change: parser.rs (~30 lines), lib.rs AST (~6 variants)
Test: new predicates parse; existing code unaffected
Duration: 1–2 days
```

#### Step 1: Alias types register in TypeRegistry

Today `collect_dsl_type_registry()` skips `TypeBody::Alias`. Fix this
so `type Url = String where pattern(...)` produces a proper
`Dag<TypeOp>` in the registry with `Identity → Validate(Matches)`.

```
Change: typecheck/lib.rs (~50 lines)
Test: registry.get("Url") returns a multi-node DAG
Duration: 2–3 days
```

After this, every type defined in `std/types.dag` — including
refinements — has a structural DAG in the registry. This is
independently valuable even without any further migration.

#### Step 2: Product/Coproduct fields become SubDags

Change `TypeOp::Product(Vec<(String, TypeId)>)` to use SubDag
children for field types. Change `TypeOp::Coproduct` the same way.

```
Change: type_op.rs, type_lib.rs, type_registry.rs, contract.rs,
        type_shape.rs, typecheck
Duration: 1–2 weeks
```

This is the hardest structural change. After it, a `Product` is a
self-contained DAG — you can walk from `Summary` through its fields
(`total: Int`, `passed: Int`, `failed: Int`) without registry lookups.

#### Step 3: Migrate `Bool` — the simplest primitive

Write `std/logic.dag` with `type Classical = True | False`.
In `std/types.dag`, change `Bool` from a compiler built-in to:

```dag
import std.logic { Classical }
type Bool = Classical
```

Compiler change: `TypeRegistry::register_primitives()` no longer
hardcodes `Bool`. Instead, the registry loads it from
`std/logic.dag` → `std/types.dag`. The `BaseType::Bool` enum variant
is still used internally as a cache, but it's *derived* from the
DAG, not the source of truth.

```
Change: type_registry.rs, type_lib.rs, std/logic.dag, std/types.dag
Test: all existing Bool tests pass; registry.get("Bool") returns
      a DAG with Coproduct(True, False)
Duration: 3–5 days
```

Why Bool first: it's the simplest primitive (2 values, no arithmetic,
no width). It proves the full round-trip: DSL definition → parse →
register → typecheck → lower → emit, without needing any new
predicates.

#### Step 4: Migrate `Int` and `Float` — width + arithmetic

Write `std/bit.dag` (Bit, Byte, Word32, Word64), `std/integer.dag`
(Int8..64, UInt8..64), `std/float.dag` (Float32, Float64).

Compiler change: the IR `Predicate` enum gets `Width(u16)`,
`Length(u64)`, `Signed(Option<String>)`, `Unsigned`. The lowerer and
emitter use these to derive what today lives in `PlatformRepr`.

```dag
// std/bit.dag
type Bit = Classical where width(1)
type Byte = { bits: List<Bit> where length(8) }
type Word64 = { bytes: List<Byte> where length(8) }

// std/integer.dag
type Int64 = Word64 where signed(twos_complement)
type Int = Int64

// std/float.dag
type Float64 = Word64 where ieee754(binary64)
type Float = Float64
```

```
Change: type_op.rs (new Predicate variants), type_registry.rs,
        type_lib.rs, emit backends (derive width from DAG instead
        of PlatformRepr), new .dag files
Test: emit("Int") still produces "i64" in Rust, "int64" in Go —
      now derived from width(64) + signed, not string matching
Duration: 1–2 weeks
```

Why Int next: it's the highest-frequency primitive after String,
and it exercises the new predicates (width, signed). Float follows
immediately since it's the same structure with `ieee754` instead of
`signed`.

#### Step 5: Migrate `String` and `Bytes`

Write `std/string.dag`. String becomes a structural type
(sequence of bytes with encoding). `Bytes` becomes `List<UInt8>`.

```dag
// std/string.dag
type String = { bytes: List<Byte>, encoding: Encoding }
type Char = Int where range(min: 0, max: 1114111), brand("Char")
```

```
Change: type_registry.rs, std/string.dag, std/encoding.dag
Test: all string operations still work; encoding lattice is
      DAG-derived instead of Rust match arms
Duration: 1 week
```

#### Step 6: Migrate containers (`List`, `Option`, `Map`)

Write `std/containers.dag`. Containers become structural types
where cardinality is a derived consequence.

```
Change: type_registry.rs (remove WrapperKind hardcoding),
        std/containers.dag
Test: List<String> still infers cardinality [0,∞)
Duration: 1 week
```

#### Step 7: Emit backends read structure

Replace `map_abstract_type(string)` with DAG walkers. The three
divergent backend mapping strategies unify into one.

```
Change: type_mapping.rs, type_codegen.rs, lower_to_ir.rs, lower_c.rs
Test: all codegen tests pass with structural resolution
Duration: 1–2 weeks
```

#### Step 8: Eliminate `BaseType` enum and string classification

Delete `BaseType`, `semantic_carrier_kind_for_type_id()`,
string-based `TypeCategory`. Everything is DAG-structural now.

```
Change: types.rs, type_registry.rs (~300 lines deleted)
Test: all tests pass without string matching
Duration: 3–5 days
```

#### Step 9: Cardinality derived, Guard unified with Predicate

Remove `Port.cardinality` as independently set. Derive it from type
DAGs. Replace `Guard` with `Predicate` on edges.

```
Change: dag.rs, lowerer, executor
Test: audit_cardinality_drift() reports zero drift (then delete it)
Duration: 1 week
```

#### Step 10: Delete `MetadataPayload` and `PlatformRepr`

Everything formerly in metadata is now structural. Delete the escape
hatches.

```
Change: type_op.rs, system_model.rs, type_shape.rs
Test: all tests pass without Meta nodes
Duration: 3–5 days
```

### Summary: The Migration as a Table

| Step | What | Compiler change | DSL change | Duration |
|------|------|----------------|------------|----------|
| 0 | Parser accepts new predicates | parser.rs | — | 1–2 days |
| 1 | Alias types register in TypeRegistry | typecheck | — | 2–3 days |
| 2 | Product/Coproduct fields → SubDags | IR foundation | — | 1–2 weeks |
| 3 | Migrate Bool | registry, type_lib | std/logic.dag | 3–5 days |
| 4 | Migrate Int, Float | IR predicates, emit | std/bit.dag, std/integer.dag, std/float.dag | 1–2 weeks |
| 5 | Migrate String, Bytes | registry | std/string.dag, std/encoding.dag | 1 week |
| 6 | Migrate containers | registry | std/containers.dag | 1 week |
| 7 | Emit reads structure | emit backends | — | 1–2 weeks |
| 8 | Delete BaseType, string classification | types.rs | — | 3–5 days |
| 9 | Derive cardinality, unify Guard/Predicate | dag.rs, executor | — | 1 week |
| 10 | Delete MetadataPayload, PlatformRepr | type_op.rs | — | 3–5 days |
| **Total** | | | | **~10–14 weeks** |

Steps 0–2 are pure compiler infrastructure. No `.dag` files change.
No existing behavior changes. These can ship as normal PRs.

Steps 3–6 are the type-by-type migration. Each introduces one
foundation `.dag` file and removes the corresponding hardcoded Rust.
Each step produces a green codebase. The order (Bool → Int/Float →
String → containers) follows dependency: containers need types,
types need integers, integers need bits, bits need logic.

Steps 7–10 are cleanup. The structural path is primary. The string
path is vestigial. Delete it.

---

## Task Sheet

Each task has: what to create, what to change, what to delete, and
acceptance criteria. Every task produces a green codebase. Tasks are
ordered by dependency — a task's prerequisites are all prior tasks.

The acceptance test for the entire migration: `cargo test` passes,
`cargo clippy --all-targets -- -D warnings` passes, `tools/gist.dag`
compiles and emits identical output to today, and every item in the
"Must Be Deleted" column is gone.

---

### Task 0: Parser Accepts New Predicates

**Create:**
- Nothing

**Change:**
- `src/03_source/daglang-syntax/src/lib.rs` — add `Refinement` variants: `Width(Expr)`, `Length(Expr)`, `Unsigned`, `Signed(Option<String>)`, `Arithmetic`, `Domain(String)`
- `src/03_source/daglang-syntax/src/parser.rs` — add 6 match arms in `parse_refinement()` before the `other =>` fallback (~30 lines)

**Delete:**
- Nothing

**Acceptance:**
- `type Bit = Bool where width(1)` parses to `TypeExpr::Refined(Named("Bool"), [Width(1)])`
- `type Int64 = Word64 where signed(twos_complement)` parses to `Refined(Named("Word64"), [Signed(Some("twos_complement"))])`
- `type Byte = { bits: List<Bit> where length(8) }` parses correctly
- All existing parser tests pass unchanged
- `cargo test -p daglang-syntax` green

---

### Task 1: Alias Types Register in TypeRegistry

**Create:**
- Nothing

**Change:**
- `src/04_semantics/daglang-typecheck/src/lib.rs` — in `collect_dsl_type_registry()`, handle `TypeBody::Alias(type_expr)` (currently `TypeBody::Alias(_) => {}`): construct `Dag<TypeOp>` with Identity + Validate nodes from refinements, register in `TypeRegistry`
- Map `Refinement::Pattern` → `Predicate::Matches`, `Refinement::Range` → `Predicate::InRange`, `Refinement::NonEmpty` → `Predicate::NonEmpty`, `Refinement::Brand` → `TypeOp::Brand`, `Refinement::Content` → `Predicate::Content`, new refinements → new predicates (from Task 0)

**Delete:**
- Nothing

**Acceptance:**
- `type Url = String where non_empty, pattern("^https?://")` produces a 3-node DAG: `Identity("String") → Validate(NonEmpty) → Validate(Matches("^https?://"))`
- `type Port = Int where range(min: 1, max: 65535)` produces a 2-node DAG: `Identity("Int") → Validate(InRange(1, 65535))`
- `type Char = Int where brand("Char"), range(min: 0, max: 1114111)` produces the right Brand + Validate DAG
- `registry.get("Url")` returns `Some(dag)` with 3 nodes (previously returned `None`)
- All existing typecheck tests pass unchanged
- `cargo test -p daglang-typecheck` green

---

### Task 2: Product/Coproduct Fields Become SubDags

**Create:**
- Nothing

**Change:**
- `src/00_foundation/ir/src/type_op.rs` — change `Product(Vec<(String, TypeId)>)` to `Product(Vec<String>)` with field type DAGs as SubDag children; same for `Coproduct`
- `src/00_foundation/ir/src/type_lib.rs` — `product()` and `coproduct()` builders add SubDag nodes for each field type (resolved from registry)
- `src/00_foundation/ir/src/type_registry.rs` — `register_core_types()` updated for new Product/Coproduct shape
- `src/00_foundation/ir/src/contract.rs` — `TypeContract::from_type_dag()` walks SubDag children for field types
- `src/00_foundation/ir/src/type_shape.rs` — `type_shape()` recurses into SubDag children
- `src/04_semantics/daglang-typecheck/src/lib.rs` — `collect_dsl_type_registry()` builds Products/Coproducts with SubDag children

**Delete:**
- Nothing yet (old `TypeId` references in Product/Coproduct removed)

**Acceptance:**
- `type Summary { total: Int, passed: Int, failed: Int }` produces a Product DAG with 3 SubDag children, each containing the field's type DAG
- `type Platform = Linux | Macos | Windows` produces a Coproduct DAG with 3 SubDag children
- Walking the DAG from `Summary` reaches `Int`'s Identity DAG without registry lookups
- `TypeContract::from_type_dag()` correctly extracts field types from SubDags
- All existing tests pass (contract tests, shape tests, typecheck tests)
- `cargo test` green (full suite)

---

### Task 3: Migrate Bool

**Create:**
- `dsl/std/logic.dag` — `module std.logic` with `type Classical = True | False`

**Change:**
- `dsl/std/types.dag` — add `import std.logic { Classical }` and `type Bool = Classical`
- `src/00_foundation/ir/src/type_registry.rs` — `register_primitives()` no longer hardcodes `Bool`; it's loaded from the DSL type registry (populated by typecheck from `std/logic.dag` → `std/types.dag`)
- `src/00_foundation/ir/src/type_lib.rs` — `bool()` returns the registry-resolved DAG instead of `identity("Bool")`

**Delete:**
- The `"Bool"` entry in `register_primitives()` (registration moves to DSL)

**Acceptance:**
- `registry.get("Bool")` returns a DAG equivalent to `Coproduct(True, False)` — resolved from `std/logic.dag`, not from Rust
- `registry.get("Classical")` returns the same DAG
- `registry.is_compatible(&TypeId::from("Bool"), &TypeId::from("Classical"))` is true
- All existing tests that use `Bool` pass unchanged
- `tools/gist.dag` compiles successfully (uses `Bool` in `public: Bool = false`)
- `cargo test` green

---

### Task 4: Migrate Int, Float

**Create:**
- `dsl/std/bit.dag` — `type Bit = Classical where width(1)`, `type Byte = { bits: List<Bit> where length(8) }`, `type Word32`, `type Word64`
- `dsl/std/integer.dag` — `type Int8..64`, `type UInt8..64`, `type Int = Int64`
- `dsl/std/float.dag` — `type Float64 = Word64 where ieee754(binary64)`, `type Float = Float64`

**Change:**
- `src/00_foundation/ir/src/type_op.rs` — add `Predicate::Width(u16)`, `Predicate::Length(u64)`, `Predicate::Signed(Option<String>)`, `Predicate::Unsigned` to IR `Predicate` enum
- `src/00_foundation/ir/src/type_registry.rs` — `register_primitives()` no longer hardcodes `Int`, `Float`, `Bytes`
- `src/07_emit/daglang-emit/src/type_mapping.rs` — begin reading width/signedness from type DAGs for Rust/Go mapping (e.g., `width(64) + signed → i64`)
- `src/07_emit/daglang-emit/src/type_codegen.rs` — inspect refinements instead of stripping them

**Delete:**
- `"Int"`, `"Float"`, `"Bytes"` entries from `register_primitives()`

**Acceptance:**
- `registry.get("Int")` returns a DAG chain: `Int → Int64 → Word64 → {List<Byte> where length(8)} → ...`
- `registry.get("Bit")` returns a DAG with `width(1)` predicate
- Walking Int's DAG and extracting all `Width` predicates yields total width 64
- Walking Int's DAG finds `Signed(Some("twos_complement"))`
- Emit for Int still produces `i64` (Rust), `int64` (Go), `int64_t` (C) — now derived from DAG, not string table
- Emit for Float still produces `f64` (Rust), `float64` (Go), `double` (C)
- `tools/gist.dag` compiles (uses `Int` in skip count, `Bool` in conditions)
- `cargo test` green

---

### Task 5: Migrate String, Bytes

**Create:**
- `dsl/std/string.dag` — `type String = { bytes: List<Byte>, encoding: Encoding }`, `type Char`
- `dsl/std/encoding.dag` — `type Encoding = ASCII | UTF8 | Latin1 | Text | Binary | Unknown` with lattice ordering declared in DSL

**Change:**
- `src/00_foundation/ir/src/type_registry.rs` — `register_primitives()` no longer hardcodes `String`, `Secret`, `Bytes`; `register_core_types()` no longer hardcodes `ContentEncoding` lattice — it's read from `std/encoding.dag`
- `src/00_foundation/ir/src/type_op.rs` — `ContentEncoding` lattice ordering driven by DSL declarations (the Rust enum may remain as a cache but `is_subtype_of` reads from registry)

**Delete:**
- `"String"`, `"Secret"`, `"Bytes"` entries from `register_primitives()`
- Hardcoded `ContentEncoding` lattice match arms in `type_op.rs` (replaced by DSL-declared ordering)

**Acceptance:**
- `registry.get("String")` returns a structural DAG (not an Identity node)
- `registry.get("Encoding")` returns a Coproduct with lattice metadata
- `ContentEncoding::ASCII.is_subtype_of(&ContentEncoding::UTF8)` still returns `true` — now derived from the DSL-declared ordering
- `Predicate::Content(ASCII).entails(&Predicate::Content(UTF8))` still works
- All content encoding lattice tests pass (join, meet, absorption)
- All string operation tests pass
- `tools/gist.dag` compiles (heavy String usage throughout)
- `cargo test` green

---

### Task 6: Migrate Containers

**Create:**
- `dsl/std/containers.dag` — `type List<T>`, `type Option<T>`, `type Map<K,V>`, `type Set<T>`

**Change:**
- `src/00_foundation/ir/src/type_registry.rs` — container types loaded from DSL; `WrapperKind` enum may remain internally but is derived from the container type DAGs
- `src/00_foundation/ir/src/type_lib.rs` — `list()`, `optional()`, `map()`, `set()` become registry lookups, not hardcoded DAG builders
- `src/00_foundation/ir/src/contract.rs` — `cardinality()` derives from structural container type DAGs

**Delete:**
- Container registrations in `register_core_types()` (`OptionalString`, `StringList`, `IntList`, etc. — ~20 entries)

**Acceptance:**
- `registry.resolve_type(&TypeId::from("List<String>"))` returns a structural DAG
- `contract::cardinality(&list_string_dag)` returns `ZERO_OR_MORE`
- `registry.infer_cardinality(&TypeId::from("Optional<Int>"))` returns `Some(ZERO_OR_ONE)`
- All cardinality algebra tests pass (join, meet, product, sum, satisfies)
- All coercion tests pass
- `tools/gist.dag` compiles (uses `List<{ path, content }>`, `List<String>`)
- `cargo test` green

---

### Task 7: Emit Backends Read Structure

**Create:**
- Nothing

**Change:**
- `src/07_emit/daglang-emit/src/type_mapping.rs` — replace `map_abstract_type(&str)` with `emit_type(&Dag<TypeOp>)` that pattern-matches on `TypeShape`; delete `RUST_TYPE_MAPPING` and `GO_TYPE_MAPPING` static tables
- `src/07_emit/daglang-emit/src/type_codegen.rs` — stop stripping refinements (`Refined(inner, _) => inner`); inspect predicates for width/signedness to derive target types
- `src/07_emit/daglang-emit/src/lower_to_ir.rs` — use shared structural mapper, delete duplicate mapping
- `src/07_emit/daglang-emit/src/lower_c.rs` — use shared structural mapper, delete inline C mapping
- `src/07_emit/daglang-emit/src/lower_go.rs` — use shared structural mapper

**Delete:**
- `RUST_TYPE_MAPPING` static table (type_mapping.rs)
- `GO_TYPE_MAPPING` static table (type_mapping.rs)
- Duplicate type mapping in `lower_to_ir.rs`
- Inline C type mapping in `lower_c.rs`
- `map_abstract_type()` function

**Acceptance:**
- No function in the emit crate takes a type name string and returns a target type string
- All type mapping goes through `Dag<TypeOp>` → `TypeShape` → target string
- `emit_type(registry.resolve("Int"))` returns `"i64"` for Rust backend
- `emit_type(registry.resolve("Optional<String>"))` returns `"Option<String>"` for Rust backend
- All codegen tests pass (Rust, Go, C, MIPS emit identical output)
- `tools/gist.dag` produces identical Rust output to pre-migration
- `cargo test` green

---

### Task 8: Eliminate BaseType Enum and String Classification

**Create:**
- Nothing

**Change:**
- `src/00_foundation/ir/src/types.rs` — replace `semantic_carrier_kind_for_type_id()` with DAG-structural queries; replace `TypeId::category()` string matching with registry-based category derivation

**Delete:**
- `BaseType` enum (type_op.rs) — 11 variants, ~21 references
- `semantic_carrier_kind_for_type_id()` (types.rs) — ~70-line match statement
- `semantic_carrier_class_for_type_id()` (types.rs)
- `seed_placeholder_policy_for_type_id()` (types.rs)
- `TypeCategory` enum and `TypeId::category()` string matching (types.rs) — ~20 references
- `value_backing_for_type_id()` free function (types.rs) — replaced by `TypeRegistry::value_backing()` which already exists

**Acceptance:**
- `BaseType` does not appear anywhere in the codebase (`rg 'BaseType' --type rust` returns 0 results)
- `semantic_carrier_kind_for_type_id` does not appear (`rg 'semantic_carrier_kind_for_type_id' --type rust` returns 0 results outside of tests that verify the replacement)
- `TypeCategory` does not appear
- All semantic carrier compatibility tests pass via structural queries
- All seed placeholder policy tests pass via structural queries
- `cargo test` green

---

### Task 9: Cardinality Derived, Guard Unified with Predicate

**Create:**
- Nothing

**Change:**
- `src/00_foundation/ir/src/dag.rs` — `Port.cardinality` becomes a derived method (reads from type DAG via registry) instead of a stored field; `Guard` enum replaced by `Predicate` on ports
- `src/05_graph/daglang-lower/src/lib.rs` — all `Port::scalar()`, `Port::list()`, `Port::optional()` calls simplified: cardinality no longer passed, derived from type
- `src/06_artifacts/daglang-derive/src/lib.rs` — same port simplification
- `src/09_execute/exec/` — guard evaluation uses `Predicate::evaluate()` instead of `Guard::evaluate()`

**Delete:**
- `Port.cardinality` as a stored field (dag.rs)
- `Port::with_cardinality()` constructor (dag.rs)
- `Port::scalar()`, `Port::list()`, `Port::optional()`, `Port::non_empty_list()` as cardinality-stamping constructors — replaced by `Port::new(name, type_id)` with cardinality derived
- `Guard` enum (dag.rs)
- `audit_cardinality_drift()` function (coerce.rs) — no longer needed
- `CardinalityDrift` struct (coerce.rs)

**Acceptance:**
- `Guard` does not appear anywhere in the codebase (`rg 'Guard' --type rust` returns 0 results, excluding test comments)
- `audit_cardinality_drift` does not appear
- `Port::with_cardinality` does not appear
- Constructing `Port::new("items", "List<String>")` and querying its cardinality via the registry returns `ZERO_OR_MORE`
- All cardinality satisfaction tests pass
- All conditional/match DAG execution tests pass with Predicate-based gating
- `cargo test` green

---

### Task 10: Delete MetadataPayload and PlatformRepr

**Create:**
- Nothing

**Change:**
- `src/00_foundation/ir/src/type_shape.rs` — derive platform properties from structural predicates (width, signed, float-domain) instead of reading `MetadataPayload::PlatformRepr`

**Delete:**
- `MetadataPayload` enum (type_op.rs)
- `PlatformRepr` struct (type_op.rs)
- `TypeOp::Meta(MetadataPayload)` variant (type_op.rs)
- All `MetadataPayload::SystemId`, `SystemKind`, `BehaviorId`, `Invocation`, `Property`, `InputContract`, `OutputContract` usages in `system_model.rs` (~25 references)
- `Meta` node construction in any DAG builder

**Acceptance:**
- `MetadataPayload` does not appear anywhere in the codebase (`rg 'MetadataPayload' --type rust` returns 0 results)
- `PlatformRepr` does not appear (`rg 'PlatformRepr' --type rust` returns 0 results)
- `TypeOp::Meta` does not appear (`rg 'TypeOp::Meta' --type rust` returns 0 results)
- `type_shape()` for Int returns `TypeShape::Platform` with correct properties (bits=64, signed=true) — derived from DAG predicates
- All type shape tests pass
- All system model tests pass
- `cargo test` green

---

### Final Acceptance: Migration Complete

When all 11 tasks are done, the following invariants hold:

**Deletions verified** (all return 0 results from `rg --type rust`):
- `BaseType::`
- `MetadataPayload`
- `PlatformRepr`
- `TypeOp::Meta`
- `Guard::`
- `audit_cardinality_drift`
- `semantic_carrier_kind_for_type_id`
- `seed_placeholder_policy_for_type_id`
- `map_abstract_type`
- `RUST_TYPE_MAPPING`
- `GO_TYPE_MAPPING`
- `Port::with_cardinality`

**Structural invariants:**
- Every type referenced in any `.dag` file resolves to a `Dag<TypeOp>` via the registry
- Every `Dag<TypeOp>` for a primitive type contains structural nodes (not a single Identity node) traceable to `std/logic.dag` or `std/bit.dag`
- No type's properties (width, signedness, cardinality) are determined by string matching — all derived from DAG walks
- Emit backends produce identical output to pre-migration for all existing `.dag` files

**Behavioral invariants:**
- `cargo test` passes (full suite)
- `cargo clippy --all-targets -- -D warnings` passes
- `tools/gist.dag` compiles and emits identical Rust/Go/C output
- All codegen tests produce identical output
- All cardinality algebra property tests pass
- All content encoding lattice property tests pass
- All predicate entailment tests pass
