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

### FIXED: Sum Types Were Registered With "String" Payload Placeholders

**Problem:** `register_type_def()` mapped every `TypeBody::Sum` variant
to `("variant_name", "String")`, regardless of whether the variant was
a unit variant or carried payload fields. So `Classical = True | False`
was registered as a coproduct with two String-backed variants, not as
a pure truth-value coproduct with two unit variants.

**Fix (this PR):** `register_type_def()` now distinguishes:
- Unit variants (no fields) → `type_lib::unit()` DAG
- Single-field payload variants → resolved field type DAG
- Multi-field payload variants → anonymous product DAG

### FIXED: Merge Order / Stale-Child Problem

**Problem:** `merged_type_registry()` previously registered core types
BEFORE the DSL merge, so Rust-side coproduct registrations clobbered
DSL structural types.

**Fix:** `merged_type_registry()` now uses the sequence:
kernel → core types → DSL merge. Rust-side core types are registered
FIRST as bootstrap fallbacks, then `merge_dsl_types()` overrides them
with DSL structural definitions. DSL always wins.

### Gap 1: Kernel Types — Partially Resolved

`register_kernel_types()` registers `String`, `Int`, `Float` as
identity placeholders. These are overridden by DSL merge
(`string_type.dag`, `integer.dag`, `float.dag`), so post-merge they
are structural.

`Bool`, `Bytes`, `Secret` are already structural in the kernel:
- `Bool` → Coproduct(True: Unit, False: Unit)
- `Bytes` → List\<Byte\>
- `Secret` → Branded\<String\>

| Type   | Kernel DAG           | After DSL merge                  |
|--------|----------------------|----------------------------------|
| String | `identity("String")` | Product(bytes: List\<Byte\>, encoding: Encoding) |
| Bool   | Coproduct(True, False) | Same (no DSL override)         |
| Int    | `identity("Int")`    | Int64 alias → Word64 + Signed + Arithmetic |
| Float  | `identity("Float")`  | Float64 alias → Word64 + Domain(ieee754) |
| Bytes  | List\<Byte\>          | Same (no DSL override)          |
| Secret | Branded\<String\>    | Same (no DSL override)           |

**Remaining gap:** `String`, `Int`, `Float` are identity in kernel-only
contexts (`with_core_types()` without DSL merge). This is acceptable
for bootstrapping — the ratchet test documents the baseline.

### Gap 2: Hardcoded Name-Match Tables in Emit Layer

`emit_identity_type()` in `type_mapping.rs` has a match statement
per backend (Rust, Go, C) that maps type names to target-language
syntax. `map_to_c_type_static()` has been deleted.

**Impact:** Adding a type requires updating the match arms. Adding
a backend requires adding another arm.

**Root cause:** Named products/coproducts still route through
`emit_identity_type()` (see `emit_shape` arms for `Product(Some(name))`
and `Coproduct(Some(name))`). This is correct for language primitives
(String→String, Bool→bool) and opaque runtime types, but it means
the emit path is not fully structural yet.

**Fix (deferred):** Wire `CompileOutput::merged_type_registry()` to
production emit paths (Gap 3), then backend rule sets can replace
name-match tables. See Phase D.

### Gap 3: Registry Not Threaded to Emit Layer in Production

Every production call site uses `lower_to_*_with_registry(None)`. The
`_with_registry(Some(reg))` path exists and is tested, but no caller
passes a real registry. The structural emit path is scaffolding only.

**Fix:** Thread `CompileOutput::merged_type_registry()` through the
codegen pipeline to the emit backends.

### ~~Gap 4: `Guard` Still Exists in Executor~~ (RESOLVED)

Guard type is already `Option<Predicate>` in the IR. No separate Guard
enum exists.

### ~~Gap 5: Port Cardinality Stored-and-Mutated~~ (RESOLVED)

`Port::with_cardinality` is already `pub(crate)` — no post-construction
mutation from outside the IR crate.

### ~~Gap 6: `register_core_types()` Coproduct Lists~~ (RESOLVED)

`register_core_types()` hardcodes variant lists for 11 coproducts.
All now use `"Unit"` variant payloads (matching DSL unit variants),
and all have corresponding DSL definitions in `.dag` files:
- ContentEncoding → `std/types.dag`
- SemanticColor, Tier, SymbolId → `std/symbols.dag`
- FermiDepth → `std/types.dag`
- TransportClass, TestClass → `std/fidelity.dag`
- DisplayWidth → `std/unicode.dag`
- WarningPolicy → `std/policy.dag`
- CloudRuntime, AuthScheme → `std/cloud.dag`

The Rust-side registrations are bootstrap fallbacks for contexts
without DSL compilation. `merge_dsl_types()` overrides them.
Merge order (core types first, DSL merge second) ensures DSL wins.

### Gap 7: Transport Rewrite Tables

`rewrite_transport_call()` in each backend (Rust, Go, C) has ~15
hardcoded name→function pairs mapping abstract transport operations
to target-language runtime functions. These are replicated across
backends with minor syntax differences.

### Gap 8: Generic/Container Composition (PARTIALLY RESOLVED)

`resolve_field_type_dag()` now handles `TypeExpr::Generic(...)` for
`List<T>`, `Option<T>`, `Set<T>`, and `Map<K,V>`. Container types
produce structural DAGs, not identity strings.

**Resolved:** `List<Bit>` → `type_lib::list(resolve(Bit))`, etc.
`TypeExpr::Optional` and `TypeExpr::Refined` also structurally resolve.

**Remaining gap: Map key erasure.** `Map<K,V>` resolves only the value
type — `type_lib::map(val)` drops the key. `Map<String, Int>` and
`Map<Int, Int>` are structurally indistinguishable. Fix requires
extending `type_lib::map()` to accept key + value DAGs.

**Remaining gap: Refinement predicates on containers.** `List<Bit>
where length(8)` does not yet apply the `length(8)` predicate to the
container DAG. The derivation chain `Byte.bits → List<Bit> where
length(8)` is resolved as a plain `List<Bit>` without the constraint.

### Gap 9: No Compositional Width Derivation (NEW)

`derive_structural_properties()` copies explicit `Width`, `Length`,
`Domain` predicates from DAG nodes but does not compose them. Width
is only derived when an explicit `Predicate::Width(N)` node exists.

**Impact:**
- `Bit = Classical where width(1)` → width(1) ✓ (explicit predicate)
- `Byte = { bits: List<Bit> where length(8) }` → width(?) ✗ (no explicit
  width predicate; should derive width = 8 × 1 = 8)
- `Word32 = { bytes: List<Byte> where length(4) }` → width(?) ✗ (should
  derive width = 4 × 8 = 32)
- `UInt8 = Byte where unsigned, arithmetic` → width(?) ✗ (no derived
  width from Byte)
- `Float32 = Word32 where domain("ieee754_binary32")` → width(?) ✗
  (should derive width = 32 from Word32)

This means the structural emit path (`emit_platform_type`) cannot
determine the width of `UInt8` or `Float32` from their type DAGs
alone, defeating the purpose of the derivation chain.

**Fix:** `derive_structural_properties()` must compose width:
1. For products with a single `List<T> where length(N)` field where
   `T` has known width `W`: derived width = `N × W`
2. For alias types whose base has known width: inherit width
3. This is a recursive walk — `Byte` gets width 8 from its field
   `List<Bit> where length(8)` where `Bit` has width 1.

### Gap 10: Unresolved Refs Still Silently Degrade (NEW)

Multiple authoritative paths silently fall back to identity/nominal
wrappers instead of failing on unresolved structure:

- `resolve_field_type_dag()` falls back to `identity(name)`
- `register_product()` / `register_coproduct()` silently use identity
  for unresolved child types
- Topological sort cycle breaker skips cycles without error

The `register_product_checked()` / `register_coproduct_checked()`
helpers exist and return errors, but the primary registration paths
still use the silent versions.

**Fix:** Adopt a "fail-loud" policy on the structural path:
1. Replace `register_product` / `register_coproduct` calls in
   `register_type_def` with their `_checked` variants
2. Make `resolve_field_type_dag` return `Result`, surfacing unresolved
   types as compile errors
3. Surface topological sort cycles as structural errors, not silent
   skips
4. Keep the silent paths only for the kernel bootstrap (where identity
   placeholders are intentional)

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

### Backend as a Rule Set (Intermediate Form)

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

This flat rule-set model is a useful stepping stone and captures the
mechanical behavior correctly. But it doesn't model *why* Rust and Go
share `i32`/`int32` for the same structural pattern while Verilog
doesn't — that relationship is implicit in the rule ordering. The
backend language model (see next section) gives these relationships
structure.

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

## Backend Language Models: Hierarchical Target Modeling

### The Deeper Point

The flat rule-set model above tells the emitter *what to do* but not
*what the target is*. Each backend is an opaque list of patterns. If
Rust, Go, and C all emit `i32`/`int32`/`int32_t` for the same
structural pattern, that's not a coincidence — they share a common
ancestry in how they represent machine integers. The rule-set model
can't express that; it just happens to have similar entries.

The type system got this right: types are structural DAGs, not
opaque names. `Int32` isn't a magic string — it's `Word32 where
signed, arithmetic`, which is `{ bytes: List<Byte> where length(4) }`
all the way down to `Classical`. The structure carries the knowledge.

Backend languages should follow the same pattern. A compilation
target is a *structure* — a set of representational capabilities
with hierarchical relationships. Rust's `i32` IS a signed 32-bit
machine integer with Rust-specific syntax. Go's `int32` IS the same
machine integer with Go-specific syntax. They share that structure
because both ultimately target the same ISA registers — that's an
objective fact documented in their respective specs and the
compilation chains that connect them to the hardware.

If we model this hierarchy, then emitting a type into a backend is
not a lookup — it's structural resolution. The source type's
predicates (width=32, signed, arithmetic) resolve against the
target language's type hierarchy. The type unpacks itself into the
backend's structure the same way it unpacks itself into the source's
derivation chain.

### The Three Hierarchies

The system has three parallel hierarchies, all following the same
pattern: tautological foundations at the bottom, composition at the
top, structure all the way through.

```
Types:       Classical → Bit → Byte → Word → Int32
Targets:     ISA → C → Rust / Go / C++
Extdeps:     behavioral → cloud → gcp → SecretManager
```

Types model *what data is*. Targets model *how data renders*.
Extdeps model *where data lives*. All three are compositional,
layered, and (eventually) DAG-expressible. The emit function sits
at the intersection of the first two: given a type DAG and a target
DAG, find where they align.

### The Organizing Principle: Compilation Target Chains

The extdeps system layers external API knowledge, and every layer
points to a concrete specification:

```
Layer 0  std/behavioral.dag     "What is idempotency?"     (tautology)
Layer 1  cloud/cloud.dag        "What is a cloud provider?" (abstract)
Layer 2  cloud/gcp/gcp.dag      "What is GCP?"              (instantiation)
Layer 3  cloud/gcp/sm.dag       "What is Secret Manager?"   (composition)
```

Adding Stripe means instantiating existing vocabulary with Stripe's
documented facts. No Rust changes.

Backend targets should follow the same rigor. But the organizing
relationship isn't "family membership" (that's vague and subjective).
It's the **compilation target chain**: each language compiles to a
lower-level representation, and that relationship is objective,
verifiable, and backed by a concrete specification.

```
Rust (The Rust Reference)
  compiles via rustc to → LLVM IR (LLVM Language Reference Manual)
    compiles via llc to → x86-64 assembly
      assembles to → x86-64 machine code (Intel® 64 and IA-32 SDM)

Go (The Go Programming Language Specification)
  compiles via gc to → machine code (architecture-specific)

C (ISO/IEC 9899:2018 — C17)
  compiles via gcc/clang to → assembly / LLVM IR → machine code

Verilog (IEEE 1364-2005)
  synthesizes to → netlist → FPGA bitstream / ASIC layout
```

The compilation chain explains *why* languages share type
representations: Rust and C both have 32-bit signed integers because
both ultimately target machines with 32-bit registers. They differ
on tagged unions because Rust's spec adds them while C's spec
doesn't. Verilog shares widths and signedness with the ISA level
but has no pointers or heap because synthesis targets hardware, not
a Von Neumann machine.

These are objective, citable relationships — not taxonomic opinions.
Every node in the hierarchy is a real specification document.

### The Hierarchy

```
Layer 0  backend/isa.dag            Ref: ISA manuals (x86-64 SDM, ARM ARM, RISC-V spec)
         "What can a machine register hold?"

Layer 1  backend/c.dag              Ref: ISO/IEC 9899:2018 (C17)
         "What does ISO C define over the ISA?"
         backend/verilog.dag        Ref: IEEE 1364-2005
         "What does Verilog define over the ISA?"

Layer 2  backend/rust.dag           Ref: The Rust Reference
         "What does Rust define over C's model?"
         backend/go.dag             Ref: The Go Programming Language Specification
         "What does Go define over its machine target?"
         backend/cpp.dag            Ref: ISO/IEC 14882:2020 (C++20)
         "What does C++ define over C's model?"
```

The "over" relationships are compiler I/O chains:
- Rust compiles through LLVM, which targets the same machine model
  that C targets. Rust's type capabilities are a superset of C's
  for the same reason: it targets the same ISA, through a compatible
  intermediate representation.
- Go compiles to machine code via its own backend, but its scalar
  types still reflect the ISA's register widths.
- C++ is literally a superset of C (modulo edge cases), targeting
  the same backends.
- Verilog branches from the ISA level directly — it targets hardware
  synthesis, not a Von Neumann instruction stream.

### Layer 0: ISA — Instruction Set Architecture

What the machine physically provides. Every software compilation
target eventually produces instructions for one of these. Every
hardware target eventually produces a circuit that implements
equivalent logic.

Ref: Intel® 64 and IA-32 Architectures Software Developer's Manual;
ARM Architecture Reference Manual; The RISC-V Instruction Set Manual

```dag
module backend.isa

// Ref: x86-64 SDM Vol. 1 §3.1 — Fundamental Data Types
// A machine provides fixed-width general-purpose registers.
type GPR {
    width: Int
}

// Ref: x86-64 SDM Vol. 1 §3.1.2
// Signedness is an interpretation of the bit pattern.
type SignedGPR = GPR where signed
type UnsignedGPR = GPR where unsigned

// Ref: IEEE 754-2019 §3.3; x86-64 SDM Vol. 1 §4.2.2
// The machine provides IEEE 754 floating-point via SSE/AVX registers.
type FPR = GPR where domain("ieee754")

// Ref: x86-64 SDM Vol. 1 §3.4.1
// Standard register widths available on x86-64.
data register_widths: List<Int> = [8, 16, 32, 64]
```

This layer defines what the silicon provides: registers with widths,
two interpretations (signed/unsigned), and IEEE 754 float support.
Nothing about `struct`, `enum`, pointers, heap, or syntax — those
are language-level concepts that don't exist at the ISA level.

### Layer 1: C and Verilog — Direct ISA Abstractions

These languages are the first abstraction layer over the ISA. They
add named types, composite structures, and (for C) pointer-based
memory access. Each has a concrete specification that defines
exactly what it provides.

**C** (Ref: ISO/IEC 9899:2018 — C17):

```dag
module backend.c

import backend.isa { GPR, SignedGPR, UnsignedGPR, FPR }

// Ref: C17 §7.20.1.1 — Exact-width integer types
type int8_t   = SignedGPR   where width(8),  syntax("int8_t")
type int16_t  = SignedGPR   where width(16), syntax("int16_t")
type int32_t  = SignedGPR   where width(32), syntax("int32_t")
type int64_t  = SignedGPR   where width(64), syntax("int64_t")
type uint8_t  = UnsignedGPR where width(8),  syntax("uint8_t")
type uint16_t = UnsignedGPR where width(16), syntax("uint16_t")
type uint32_t = UnsignedGPR where width(32), syntax("uint32_t")
type uint64_t = UnsignedGPR where width(64), syntax("uint64_t")

// Ref: C17 §6.2.5 ¶10 — Real floating types
type c_float  = FPR where width(32), domain("ieee754_binary32"), syntax("float")
type c_double = FPR where width(64), domain("ieee754_binary64"), syntax("double")

// Ref: C17 §6.7.2.1 — Structure specifiers
type c_struct {
    syntax_template: "struct {name} { {fields} }"
}

// Ref: C17 §6.7.2.2 — Enumeration specifiers
// C enums are untagged integer constants — no payload support.
type c_enum {
    syntax_template: "enum {name} { {variants} }"
    payload_support: false
}

// Ref: C17 §6.7.6 — Pointer declarators
type c_pointer {
    syntax_template: "{T}*"
}

// C has no generic containers, no tagged unions, no GC.
// Containers decompose to pointer arithmetic.
type c_bool where syntax("bool")         // Ref: C17 §7.18 (stdbool.h)
type c_void where syntax("void")
type c_string where syntax("const char*")
```

**Verilog** (Ref: IEEE 1364-2005):

```dag
module backend.verilog

import backend.isa { GPR, FPR }

// Ref: IEEE 1364-2005 §3.2.2 — Nets
type wire = GPR where kind("combinational"),
    syntax("wire [{W-1}:0] {name}")

// Ref: IEEE 1364-2005 §3.2.4 — Regs
type reg = GPR where kind("sequential"),
    syntax("reg [{W-1}:0] {name}")
type reg_signed = GPR where kind("sequential"), signed,
    syntax("reg signed [{W-1}:0] {name}")

// Ref: IEEE 1364-2005 §3.9 — Real, realtime
type real = FPR where syntax("real")

// Verilog has no pointers, no heap, no containers.
// Composite data flattens to bit vectors or port bundles.
// Ref: IEEE 1364-2005 §12 — Hierarchical structures (modules, ports)
```

C and Verilog both sit directly on top of the ISA, but they model
different aspects: C models the Von Neumann execution model
(sequential instructions, memory, pointers), Verilog models the
hardware fabric (combinational logic, registers, clock domains).
The branching is not a taxonomy — it's a consequence of targeting
different physical substrates from the same ISA-level primitives.

### Layer 2: Rust, Go, C++ — Higher-Level Targets

These languages compile through (or alongside) the Layer 1 targets.
Their type systems are supersets of what the ISA provides, adding
language-specific constructs. Each addition is documented in the
language's specification.

**Rust** (Ref: The Rust Reference, doc.rust-lang.org/reference):

```dag
module backend.rust

import backend.c { int8_t, uint8_t, c_float, c_double, c_struct, ... }

// Ref: Rust Reference §6.1.1 — Integer types
// Rust's integer types map to the same ISA registers as C's,
// through LLVM IR (LLVM Language Reference Manual).
type i8  = int8_t   where syntax("i8")
type i16 = int16_t  where syntax("i16")
type i32 = int32_t  where syntax("i32")
type i64 = int64_t  where syntax("i64")
type u8  = uint8_t  where syntax("u8")
type u16 = uint16_t where syntax("u16")
type u32 = uint32_t where syntax("u32")
type u64 = uint64_t where syntax("u64")

// Ref: Rust Reference §6.1.2 — Floating-point types
type f32 = c_float  where syntax("f32")
type f64 = c_double where syntax("f64")

// Ref: Rust Reference §6.1.9 — Struct types
type rust_struct = c_struct where syntax("struct {name} { {fields} }")

// Ref: Rust Reference §6.1.10 — Enumerated types
// Rust extends C's enum with tagged payloads (algebraic data types).
// This is a capability C does not have — it's in the Rust spec, not the C spec.
type rust_enum {
    syntax_template: "enum {name} { {variants} }"
    payload_support: true
}

// Ref: Rust Reference §8.1 — std::vec::Vec
type rust_vec     { syntax_template: "Vec<{T}>" }
type rust_option  { syntax_template: "Option<{T}>" }
type rust_hashmap { syntax_template: "HashMap<{K}, {V}>" }
type rust_hashset { syntax_template: "HashSet<{T}>" }

// Composite patterns: structural shapes that map to Rust builtins.
type rust_string = Product(bytes: List<Width(8)>, encoding: _)
    where syntax("String")
type rust_bool = Coproduct(2, all_unit) where syntax("bool")
type rust_unit where syntax("()")
```

**Go** (Ref: The Go Programming Language Specification, go.dev/ref/spec):

```dag
module backend.go

import backend.isa { SignedGPR, UnsignedGPR, FPR }

// Ref: Go spec §Numeric types
// Go targets machine code via its own compiler (gc), not through C.
// Its integer types still reflect ISA register widths.
type go_int8   = SignedGPR   where width(8),  syntax("int8")
type go_int16  = SignedGPR   where width(16), syntax("int16")
type go_int32  = SignedGPR   where width(32), syntax("int32")
type go_int64  = SignedGPR   where width(64), syntax("int64")
type go_uint8  = UnsignedGPR where width(8),  syntax("uint8")
type go_uint16 = UnsignedGPR where width(16), syntax("uint16")
type go_uint32 = UnsignedGPR where width(32), syntax("uint32")
type go_uint64 = UnsignedGPR where width(64), syntax("uint64")

// Ref: Go spec §Numeric types — float32, float64
type go_float32 = FPR where width(32), domain("ieee754_binary32"), syntax("float32")
type go_float64 = FPR where width(64), domain("ieee754_binary64"), syntax("float64")

// Ref: Go spec §Struct types
type go_struct { syntax_template: "type {name} struct { {fields} }" }

// Go has no algebraic enums. Coproducts decompose to interface + variants.
// Ref: Go spec §Interface types

// Ref: Go spec §Slice types, Map types
type go_slice { syntax_template: "[]{T}" }
type go_map   { syntax_template: "map[{K}]{V}" }

type go_string = Product(bytes: List<Width(8)>, encoding: _)
    where syntax("string")
type go_bool = Coproduct(2, all_unit) where syntax("bool")
type go_unit where syntax("struct{}")
```

Note that Go imports from `backend.isa` directly, not from
`backend.c`. This reflects reality: Go's compiler (`gc`) has its
own backend and does not compile through C. It shares scalar
representations with C because both target the same ISA registers,
not because Go "extends" C. The hierarchy captures this by letting
both C and Go import from the ISA layer independently.

### Why the Compilation Chain Matters

The compilation target chain explains three things that a subjective
taxonomy cannot:

1. **Why types are shared.** Rust, Go, and C all have a 32-bit
   signed integer because all three ultimately target machines with
   32-bit registers. The ISA spec is the shared ancestor — that's
   a citable fact, not a classification judgment.

2. **Why types diverge.** Rust has `enum` with payloads because the
   Rust Reference §6.1.10 defines them. C does not because ISO
   C §6.7.2.2 defines enums as integer constants only. The
   divergence is traceable to specific sections of specific specs.

3. **Where decomposition boundaries fall.** When emitting `String`
   to Verilog, the emitter decomposes because IEEE 1364-2005
   defines no string type. This is not a heuristic — it's a
   consequence of what the Verilog spec provides. The language
   model's contents are derived from the spec; the emitter reads
   the model; the decomposition follows.

Where the hierarchy should NOT be forced:

- Some targets don't fit a clean chain. TypeScript (ECMA-262)
  targets a VM with no fixed-width integers — it may import from
  the ISA layer for floats (IEEE 754 via `Number`) but not for
  integers. The hierarchy should reflect that honestly rather than
  inventing an intermediate node.
- Container representations vary across languages even when scalar
  types align. That's fine — each language model declares its own
  container syntax. Shared structure exists at the ISA/scalar level;
  divergent structure exists at the language level.
- The compilation chain is the *primary* organizing relationship
  but not the only one. A future `backend/wasm.dag` (Ref: WebAssembly
  Core Specification) would import ISA-level concepts but represent
  a virtual machine, not physical silicon. The model accommodates
  this without forcing WASM into a physical-ISA chain.

### How Resolution Works

Emission becomes structural resolution between two DAGs — the
source type DAG and the target language DAG. The algorithm:

```
resolve(source_type, language_model):
  1. Extract StructuralProperties from source_type
  2. Walk the language_model's type entries
  3. Find the most specific language type whose structural
     predicates are satisfied by the source's properties
  4. Instantiate the syntax template with resolved values
  5. If no match: decompose source one structural level, retry
```

This is the same operation as type coercion (structural DAG walk
to find a compatible target) and extdeps resolution (structural
matching of API signatures to provider implementations). One
mechanism across all three domains.

#### Resolution Example: UInt32 → Rust

```
Source: UInt32
  → StructuralProperties { width: 32, signed: false, arithmetic: true }

Language model walk (backend.rust):
  u32
    = uint32_t (C17 §7.20.1.1) where syntax("u32")
    = UnsignedGPR where width(32) (ISA: 32-bit register)
  Match: width(32) ✓, unsigned ✓, arithmetic ✓

Result: "u32"
```

No match table consulted. The source type's properties structurally
unify with the language model's type definition, traceable through
the compilation chain all the way to the ISA.

#### Resolution Example: String → Verilog

```
Source: String
  → TypeShape::Product(bytes: List<Width(8)>, encoding: Coproduct)

Language model walk (backend.verilog):
  wire: scalar only — no match
  reg: scalar only — no match
  (no composite patterns — IEEE 1364-2005 has no native string)

Decompose one level:
  bytes field: List<Byte>
    → reg where width(8), array syntax
    → "reg [7:0] {name}_bytes []"
  encoding field: Coproduct(6 variants)
    → flatten to bit vector, width = ceil(log2(6)) = 3
    → "reg [2:0] {name}_encoding"
```

Decomposition happens because the Verilog spec (IEEE 1364-2005)
defines no string type. The language model faithfully reflects the
spec; the emitter reads the model; the decomposition follows.

### Bootstrap: Language Models as IR

The compiler needs language models to emit code. If language models
are `.dag` files compiled by the compiler, there's a bootstrap loop.
But `.dag` is sugar over IR. The IR (`Dag<TypeOp>`) is the canonical
representation, and it can be constructed and consumed without the
`.dag` pipeline.

This is the key insight: **language models can be expressed directly
as IR**, using the same `Dag<TypeOp>` vocabulary that type DAGs use.
No separate `LanguageModel` struct, no parallel type system. The
language model for Rust IS a `Dag<TypeOp>` — a type DAG whose
predicates describe Rust's representational capabilities.

The infrastructure already supports this:

- `Dag<TypeOp>` derives `Serialize`/`Deserialize` — language models
  can be serialized as JSON and loaded at startup
- `type_lib` provides constructors (`refined()`, `product_resolved()`,
  `coproduct_resolved()`, `branded()`) for building DAGs
  programmatically
- `register_kernel_types()` already constructs structural type DAGs
  in pure Rust — `Bool` as `coproduct_resolved("Bool", [("True",
  unit()), ("False", unit())])`, `Bytes` as `list(identity("Byte"))`,
  etc.
- The `Predicate` enum already has `Width`, `Signed`, `Domain`,
  `Arithmetic` — exactly the predicates language models need

A language model entry for Rust's `i32` is a `Dag<TypeOp>` with:
- `Validate(Width(32))` node
- `Validate(Signed)` node
- `Validate(Arithmetic)` node
- A `Brand("i32")` node carrying the syntax string

This is structurally identical to our source type `Int32` — because
it IS the same statement. `Int32` says "I am a 32-bit signed
arithmetic type." Rust's `i32` says "I represent 32-bit signed
arithmetic types as `i32`." Resolution is structural matching of
two `Dag<TypeOp>` instances.

**Circularity is broken** because `Dag<TypeOp>` consumption is
independent of the `.dag` compilation pipeline. The compiler loads
language model DAGs the same way it loads kernel type DAGs: directly
from Rust code or deserialized data, not through parsing and
typechecking.

**Migration path:**

1. **Now:** Construct language model DAGs in Rust using `type_lib`
   helpers and `Dag::new()`. Register them in a `LanguageModelRegistry`
   alongside the `TypeRegistry`. Same pattern as
   `register_kernel_types()`.

2. **Next:** Serialize language models as JSON IR. Load from
   `backend/rust.ir.json`, `backend/go.ir.json`, etc. Source of
   truth moves out of Rust code into data files.

3. **Later:** Write `.dag` files that compile to the same IR.
   `backend/rust.dag` becomes sugar for `backend/rust.ir.json`.
   The `.dag` pipeline produces the IR; the compiler consumes
   the IR. No circularity at any stage.

The same DAG-to-DAG resolution that matches source types against
language models is the same mechanism that will eventually power
cross-language coercion, structural testgen, and backend-specific
transport rewrites. One IR, one resolution mechanism, all domains.

### Connections to Coercion and Testgen

**Coercion.** If backend language models describe structural
relationships between target types (e.g., Go's `int` is platform-
dependent width while Rust's `i64` is fixed-width 64), then cross-
language coercion becomes structural too. "Can this Go output feed
into this Rust input?" is a resolution against both language models.
The coercion search space is the intersection of the two target
hierarchies.

**Testgen.** With structural language models, test generation gains
per-target precision. If the Rust model declares `i32` IS `{ width:
32, signed: true }` and the source type structurally matches, that's
an identity — no coercion test needed. If the source is `Width(64)`
targeting a backend where the best match is `Width(32)`, the model
tells testgen it's a narrowing conversion and boundary tests are
generated. The language model carries the information that drives
test decisions, replacing the current flat heuristics.

**Transport rewrite tables.** The three `rewrite_transport_call`
functions (Rust, Go, C) with ~15 hardcoded name→function pairs each
follow the same pattern. Transport operations can be modeled in the
language model: each language declares how abstract transport ops
(file read, HTTP request, shell exec) map to its runtime library.
The rewrite becomes a resolution against the language model's
transport vocabulary, not a per-backend match table.

---

## Coercion: DAG → DAG Translation

### The Principle

Everything is a DAG. Converting any type to any other type is a
structural DAG-to-DAG translation. The derivation chain in `std/`
*is* the coercion graph. No separate coercion registry, no tri-level
checks, no hardcoded upcast tables.

Given two type DAGs, the compiler walks the structural derivation
chain between them. If a path exists, coercion is possible. The
direction determines safety.

### Upcast (Safe)

An upcast walks *downstream* in the derivation chain — from a simpler
type toward a more composed type. Each step either:

- **Embeds** a value into a richer structure (Bit → Byte: the bit
  becomes one element of a product), or
- **Adds predicates** that narrow the domain (Byte → UInt8: adds
  `unsigned`, `arithmetic`)

Upcasts are safe because every step is information-adding. The
compiler can derive them automatically from the type DAG structure —
if `Byte = { bits: List<Bit> where length(8) }`, then `Bit → Byte`
is an upcast derivable from the structural relationship.

```
Upcast chain: Bit → Byte → UInt8
                         → Word32 → Int32
                                  → Float32
```

### Downcast (Lossy, Must Be Acknowledged)

A downcast walks *upstream* — reversing the derivation chain. Each
step either:

- **Extracts** a component from a composite (Byte → Bit: which of
  the 8 bits? information is lost), or
- **Strips predicates** that were constraining the domain (UInt8 →
  Byte: the value is no longer guaranteed unsigned)

Downcasts are mechanically reversible — the compiler can unwind the
derivation chain — but they are lossy and must be explicitly
acknowledged in the `.dag` file. Without acknowledgment, the compiler
refuses the coercion.

```
Downcast chain: Int32 → Word32 → Byte → Bit (each step lossy)
```

### Worked Examples

#### Bit → UInt8 (upcast, safe)

The compiler walks the derivation chain:
1. `Bit` → embed into `Byte.bits[0]` (structural composition)
2. `Byte` → refine to `UInt8` (add unsigned + arithmetic)

Each step is structurally derivable from the std/ type DAGs. No
manual registration needed.

#### UInt8 → Bit (downcast, lossy)

The compiler reverses:
1. `UInt8` → strip `unsigned`, `arithmetic` → `Byte`
2. `Byte` → extract from `bits` field → `Bit` (which one? truncation)

The `.dag` must acknowledge:
```dag
// Possible future syntax:
coerce UInt8 -> Bit where lossy("truncates to least significant bit")
```

#### Int32 → Float32 (lateral, same width)

Both derive from `Word32` — same structural base, different
predicates. This is neither strictly upcast nor downcast. The compiler
sees:
1. `Int32` → strip `signed`, `arithmetic` → `Word32`
2. `Word32` → add `domain("ieee754_binary32")`, `arithmetic` → `Float32`

This is a **reinterpretation** — same bits, different semantics.
Requires acknowledgment (semantically lossy even if bit-preserving).

### Workflow Coercion: Types Are Not Special

The coercion model applies identically to workflows because types and
workflows are the same substrate: `Dag<T>` parameterized over
operation type. `Dag<TypeOp>` and `Dag<LoweredOp>` share `Node`,
`Port`, `Edge`, `Cardinality`, `SubDag` composition — no seams.

A workflow has a structural shape: inputs, outputs, composition
depth, internal causal chain. That shape IS a type. A type's
validation chain IS a workflow (validate non-empty, then validate
pattern, then validate range — that's a causal sequence).

#### Example: gist → extended_gist (workflow upcast)

```
gist:
  repo_path → ls_files → for_each(show) → build_content → create_gist → gist_url

extended_gist:
  repo_path → ls_files → for_each(show) → build_content → format_md → write_file → (gist_url, local_path)
```

**Upcast** (gist → extended_gist): gist's structure embeds into
extended_gist. The first four steps are structurally identical. The
extended process adds steps after `build_content` — it *contains*
gist's causal chain as a prefix. This is the same relationship as
Bit → Byte: the simpler structure is a component of the richer one.

**Downcast** (extended_gist → gist): strip `format_md`, `write_file`,
truncate outputs to `gist_url` only. Lossy — the markdown formatting
and local file are lost. Must acknowledge.

The compiler walks the DAG structure the same way for both:
- For types: walk SubDag children, compare predicates, check embedding
- For workflows: walk SubDag children, compare port shapes, check
  that the source's causal chain is a structural sub-path of the
  target's

#### Why this matters

If coercion only worked on types, we'd have two systems: one for
"data compatibility" and one for "process compatibility." But because
both are `Dag<T>`, there's one mechanism. A workflow's output ports
have type shapes. If workflow A's output shape structurally upcasts
to workflow B's input shape, the coercion is safe — regardless of
whether A and B are "types" or "processes." The distinction doesn't
exist at the DAG level.

This means:
- A tool's output type can be coerced into another tool's input type
  by the same structural walk that coerces Int32 to Float32
- A workflow can be extended (upcast) by embedding it as a sub-DAG
  in a richer workflow, same as Bit embedding in Byte
- A workflow can be narrowed (downcast) by truncating to a sub-path,
  same as Byte → Bit, and requires the same explicit acknowledgment

### What This Replaces

The current coercion system has several layers of nominal machinery
that this model eliminates:

| Current (legacy)                        | Structural replacement                    |
|-----------------------------------------|-------------------------------------------|
| `CoercionEdge { to, transform }`        | Derivation chain walk                     |
| `register_coercion_edge(from, to)`      | Automatic from type DAG structure         |
| `coercion_edges: HashMap<TypeId, Vec>`  | No separate registry — the type DAGs ARE the graph |
| `coercion_path()` BFS on manual edges   | Structural DAG walk through std/ hierarchy |
| `coercion_neighbors()` with Json top    | Structural ancestry from type composition |
| `TypeContract` L1/L2/L3 model           | Two DAGs in, structural path out          |
| `can_safely_coerce_to_with()` tri-check | Direction of derivation chain walk        |
| `base_type_upcasts_to()` hardcoded match| Derived from std/ type DAG relationships  |
| `CoercionStrategy::ValidateTo`          | Downcast acknowledgment in .dag           |

### Status

The current system still uses the old nominal model. All items in
the left column above exist in the codebase. Migration path:

1. **Phase 1:** Make the structural derivation chain walkable
   (requires Gap 8 — structural containers — so the chain doesn't
   break at product/list boundaries)
2. **Phase 2:** Implement `structural_coercion_path(from_dag, to_dag)`
   that finds the derivation path between two type DAGs
3. **Phase 3:** Replace `is_compatible()` with structural path check
4. **Phase 4:** Add `.dag` syntax for downcast acknowledgment
5. **Phase 5:** Delete `CoercionEdge`, `coercion_edges`,
   `register_coercion_edge`, `TypeContract`, `base_type_upcasts_to`,
   `coercion_neighbors`

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

## DAG Universality: Gap Analysis

### The Principle

Everything is a DAG. Types, workflows, policies, configurations,
coercions, backend rules — all domain knowledge should live in the
DAG substrate. The compiler's Rust code provides DAG *infrastructure*
(the `Dag<T>` struct, the executor loop, registry lookups). Domain
*knowledge* lives in DAGs.

The terminology should converge: "type" and "workflow" and "policy"
are all just DAGs with different operation vocabularies. Eventually,
we should refer to everything as a "DAG" or "workflow" — no special
status for types.

### What's Already DAG-Expressed (correct pattern)

| Domain | Representation | Status |
|--------|---------------|--------|
| Types | `Dag<TypeOp>` — validation chains, products, coproducts | Done |
| Workflows | `Dag<LoweredOp>` — service calls, loops, branches | Done |
| System models | Behavior catalogs as `Dag<TypeOp>` with predicates | Done |
| Extdeps | `.dag` files — tautological API definitions | Done |
| Test policy | `test_policy.dag` — classification via DSL evaluation | Done |
| Tool rendering | `makegen.dag` — Makefile generation via DSL fns | Done |

### Domain Knowledge Still in Rust (should migrate to DAGs)

#### Tier 1: Direct blockers for structural watershed

| Knowledge | Location | Current Form | DAG Form |
|-----------|----------|-------------|----------|
| Backend type mappings | `type_mapping.rs:164-222` | 3× duplicated match tables (Rust/Go/C) | Language model DAGs organized by compilation target chain (ISA → C → Rust/Go); structural resolution replaces match tables (see Backend Language Models section) |
| Coercion graph | `type_registry.rs` (`CoercionEdge`, `coercion_path`) | Manual edge registration + BFS | Structural derivation chain walk (see Coercion section) |
| Base type lattice | `contract.rs:1827-1838` | Hardcoded `base_type_upcasts_to()` match | Derived from std/ type DAG hierarchy |
| TypeContract L1/L2/L3 | `contract.rs:1690-1804` | Tri-level struct with `can_safely_coerce_to_with()` | Two DAGs in, structural path out |

#### Tier 2: Algebraic structures as enums

| Knowledge | Location | Current Form | DAG Form |
|-----------|----------|-------------|----------|
| ContentEncoding lattice | `type_op.rs:98-148` | 6-variant enum with `is_subtype_of()` match | Coproduct type DAG with subtype predicates; already in `encoding.dag` |
| AccessMode conflicts | `resource/mod.rs:118-142` | 3-variant enum with `conflicts_with()` match | Conflict DAG or behavior declaration |
| Cardinality algebra | `types.rs:53-149`, `algebra.rs` | Struct with lattice trait impls | Boundary — traits are infrastructure; constants could be DAG-discovered |
| FermiDepth/TestClass | `fermi.rs:8-80` | Enums with timeout match tables | Already partially in `test_policy.dag`; complete migration pending |
| TransportClass | `transport_types.rs:6-143` | 9-variant enum with query methods | Transport catalog DAG or system model extension |
| EndpointBehavior | `transport_types.rs:80-143` | 8-variant enum with boolean queries | Behavior predicates on system model DAGs |

#### Tier 3: Configuration as Rust structs

| Knowledge | Location | Current Form | DAG Form |
|-----------|----------|-------------|----------|
| ToolDef metadata | `codegen/registry.rs:16-60` | Rust struct with static fields | DSL declarations (partially done via entrypoint inference) |
| TestgenTargetDef | `codegen/registry.rs:62-113` | Rust struct with 10+ fields | DSL declarations in testgen spec |
| SystemKind taxonomy | `system_model.rs:18-30` | 10-variant enum | System model catalog DAG |
| Property taxonomy | `system_model.rs:32-53` | 11-variant enum | Behavior predicates |

### Legitimately Rust (infrastructure, not domain knowledge)

These should stay as Rust code because they ARE the DAG machinery:

- `Dag<T>`, `Node<T>`, `Port`, `Edge` — the substrate itself
- Executor engine (`execute_dag`, `ExecutionMode`, topo-sort)
- Primitive ops (`GuardOp`, `LoopOp`, `BranchOp`) — execution atoms
- Registry lookup mechanics (HashMap indexing)
- Parser, lexer, lowerer — compiler pipeline stages
- Serde serialization/deserialization
- CLI argument handling

The distinction: **infrastructure** provides the DAG machinery.
**Domain knowledge** lives *in* DAGs expressed through that machinery.
Rust match tables encoding domain knowledge are the migration target.

### Migration Priority

The highest-value migrations are Tier 1 — they directly block the
structural watershed. Tier 2 items are algebraic structures that
the `behavior` / `implements` DSL constructs (future Phase G) will
address. Tier 3 items are configuration that migrates incrementally
as DSL tooling matures.

---

## Structural Testing: What the DAG Guarantees

### The Principle

**Structurally guaranteed → validated → test generated.** Tests
should only cover what the structure cannot guarantee. If the DAG
substrate makes a property structurally impossible to violate, no
test is needed for it. If the DAG can validate a property at compile
time, a compile-time check replaces a runtime test. Tests are reserved
for properties that can only be verified at runtime.

This is the testing analog of "no metadata" — if it's true, it's
structure; if it's structure, the compiler enforces it; if the
compiler enforces it, no test is needed.

### What the DAG Already Guarantees (no tests needed)

These properties are structurally enforced by the `Dag<T>` substrate
and cannot be violated at runtime:

| Property | How the DAG guarantees it | Previously tested? |
|----------|--------------------------|-------------------|
| **Port connectivity** | `Edge` connects named ports; missing ports are compile errors | Yes — redundant connectivity tests exist |
| **Acyclicity** | Topo-sort at execution time; cycles are structural errors | Yes — cycle detection tests exist but the executor inherently prevents cycles |
| **Node existence** | Edges reference node IDs; invalid IDs fail at DAG construction | Yes — some tests verify "node exists" |
| **Type identity** | `Port.type_id` is a `TypeId`; the registry resolves or fails | Partially — type resolution tests |

### What the DAG Can Validate at Compile Time (tests → compiler checks)

With structural containers (Gap 8) and compositional derivation
(Gap 9), the compiler can check these at compile time instead of
generating runtime tests:

| Property | Current test approach | Structural approach |
|----------|----------------------|---------------------|
| **Cardinality compatibility** | Generated coercion tests per edge | Compiler validates `Port.cardinality` satisfies connected port — structural guarantee once cardinality is derived from type DAG |
| **Type coercion safety** | Generated tests per type pair | Structural derivation chain walk — if upcast path exists, coercion is safe by construction |
| **Width consistency** | No systematic check | Compositional width derivation (Phase B) makes width a structural property — Int32 MUST be 32 bits because Word32 = 4×Byte = 4×8×Bit |
| **Predicate entailment** | Test-per-predicate-pair | Compiler walks predicate DAG; entailment is structural |
| **Resource conflict detection** | Generated tests for access mode pairs | Once AccessMode is a DAG, conflict detection is a structural walk |

### What Still Needs Generated Tests (runtime properties)

These properties cannot be guaranteed structurally and require
runtime verification:

| Property | Why tests are needed | Structural information available |
|----------|---------------------|--------------------------------|
| **Transport correctness** | HTTP responses depend on external services | Transport class, endpoint behavior, retry policy — the *contract* is structural, the *response* is runtime |
| **Value domain correctness** | `Int where range(1, 65535)` — the constraint is structural but the value is runtime | Predicate DAG provides the constraint; test generates boundary values |
| **Semantic equivalence** | Two workflows producing "the same" output | Output type shape is structural; content equivalence is runtime |
| **Performance / Fermi cost** | Execution time depends on environment | FermiDepth classification is structural; actual timing is runtime |
| **Idempotency** | Whether re-execution produces same result | `Idempotent` predicate is structural; verification requires re-execution |

### Orthogonality and Independent Testing

The DAG structure reveals which properties are orthogonal — they can
be tested independently because they don't interact structurally:

| Property A | Property B | Orthogonal? | Why |
|-----------|-----------|-------------|-----|
| Cardinality | Type compatibility | Yes | Cardinality is an interval lattice on ℕ; type compatibility is a DAG walk. Different algebras, no interaction. Already tested independently. |
| Width | Signedness | Yes | Width is a natural number predicate; signedness is a boolean predicate. `width(32) + signed` and `width(32) + unsigned` are independent refinements. |
| Content encoding | File path validity | Yes | Encoding is a lattice (ASCII ⊆ UTF8 ⊆ Text); path validity is a string predicate. No structural interaction. |
| Transport class | Endpoint behavior | Partially | Transport class constrains which behaviors are possible (e.g., Shell cannot be Paginated). The constraint is structural — discoverable from the transport catalog DAG. |
| Resource access mode | Workflow topology | No | Access conflicts depend on which nodes run concurrently, which depends on DAG topology. Must be tested together via `validate_resource_wiring_recursive()`. |

**Key insight:** Orthogonal properties reduce the test space
multiplicatively. If width has W values, signedness has S values,
and domain has D values, non-orthogonal testing requires W×S×D tests.
Orthogonal testing requires W+S+D tests. The DAG structure tells you
which are orthogonal.

### Current Testgen Gap Analysis

The existing testgen system generates tests from the workflow DAG
structure. With structural types and coercion, several test categories
become redundant or can be strengthened:

| Current test category | Count | After structural types |
|----------------------|-------|----------------------|
| **Coercion tests** (type A → type B compatibility) | ~200 per module | Most become compile-time checks. Only lateral/downcast coercions need runtime tests. |
| **Boundary tests** (each callable is independently mockable) | ~10 per callable | Keep — boundary isolation is a runtime property |
| **Skip propagation** (Skipped values propagate correctly) | ~5 per transport | Keep — skip behavior depends on runtime execution order |
| **Flow tests** (end-to-end DAG execution) | 1 per module | Keep — integration coverage |
| **DryRun completion** (all mocks exercised) | 1 per module | Keep — mock completeness is runtime |
| **Mock spec consistency** | 1 per module | Partially structural — type shapes can validate mock signatures at compile time |

**Net effect:** Structural types should eliminate ~40-60% of generated
coercion tests (those verifiable by structural derivation chain walk)
while strengthening the remaining tests with structural information
(e.g., boundary tests can use type DAG shapes to generate smarter
mock values).

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
| Task 3: Migrate Bool (DSL definition) | **DONE** (logic.dag exists; kernel has structural Coproduct(True, False)) |
| Task 4: Migrate Int, Float (DSL definitions) | **DONE** (bit.dag, integer.dag, float.dag exist; structural emit works for predicates) |
| Task 5: Migrate String, Bytes (DSL definitions) | **DONE** (string_type.dag, encoding.dag exist; kernel identity overridden by DSL merge) |
| Task 6: Migrate containers (DSL definitions) | **PARTIAL** (`resolve_field_type_dag` handles List/Option/Set/Map structurally; Map key erased; container refinement predicates not applied) |
| Task 7: Emit backends read structure | **PARTIAL** (structural path exists but not wired in production — Gap 3) |
| Task 8: Delete BaseType, string classification | **DONE** |
| Task 9: Cardinality derived, Guard→Predicate | **PARTIAL** (edge guards migrated; cardinality still stored on ports) |
| Task 10: Delete MetadataPayload, PlatformRepr | **DONE** (MetadataPayload deleted; PlatformRepr → StructuralProperties) |
| Sum type registration | **DONE** (unit variants get Unit DAG, payload variants get resolved type DAGs) |
| Coproduct variant payloads | **DONE** (all 11 Rust-side coproducts use "Unit" payloads, matching DSL unit variants) |
| Merge order / stale-child | **DONE** (merged_type_registry: kernel → core types → DSL merge; DSL wins) |
| Identity type ratchet | **DONE** (ratchet test: 13 allowed identity types in with_core_types baseline) |
| Value compatibility tightening | **DONE** (Bool accepts only "True"/"False"; Platform accepts only canonical variants) |

### Honest Assessment: Substrate vs. Authority

The structural substrate is real. `Dag<TypeOp>` with predicate-based
classification, SubDag field embedding, and predicate-driven emission
— that infrastructure works and is tested. But the system is still
hybrid, not fully structural end-to-end.

**What's real:**

- Foundational scalar/type DSL definitions exist and compile
- Sum-type registration handles unit and payload variants structurally
- Core coproduct catalogs have `.dag` definitions
- Product/Coproduct fields are SubDag children, not TypeId strings
- The two-phase bootstrap (kernel → DSL merge) works correctly
- Structural emit works for numeric types with explicit predicates

**What's still nominal or mixed:**

- `String`, `Int`, `Float` are identity placeholders in kernel-only
  contexts (`with_core_types()` without DSL merge) — the ratchet
  test documents 13 allowed identity types as the baseline
- Containers are partially structural: `resolve_field_type_dag()`
  handles generics, but `Map<K,V>` erases keys, container refinement
  predicates (`where length(8)`) aren't applied, and `containers.dag`
  is comments-only
- Compositional width derivation doesn't exist yet — `Byte` has no
  derived width(8), `Word32` has no derived width(32), so aliases
  like `UInt8` and `Float32` can't recover their width from structure
  alone
- Production emit still routes through `emit_identity_type()` with
  per-backend match arms — the structural path exists but callers
  pass `None` for the registry
- Cardinality is stored on ports, not derived from type DAGs
- Structural coercion is a design, not an implementation — the legacy
  `CoercionEdge`/`TypeContract` model is still live

**The framing:**

Not half-baked as a substrate. Half-migrated as a source of truth.

The substrate is real. The authority path is still mixed. The line
between "we have a structural type system in principle" and "the
compiler actually lives on it" is the remaining work below.

### Design Stance: Minimal Structural Kernel

The compiler owns a minimal non-DSL kernel for bootstrapping. This is
explicit and intentional. The kernel provides DAG infrastructure, not
types. DSL-defined types are the library built on top of that kernel.

The kernel registers identity placeholders so the DSL typechecker can
reference primitive names during compilation. Core types
(`register_core_types()`) are registered as Rust-side fallbacks, then
`merge_dsl_types()` overwrites them with structural definitions from
the compiled `.dag` files. DSL always takes precedence.

The goal is NOT "the compiler must read DSL for every primitive
immediately" but "the compiler's kernel shrinks over time as more
structure moves to DSL definitions." Identity placeholders are
transitional debt, not the end state.

### Next Milestones: Containers → Derivation → Authority

The remaining work is the actual hard part of the design. The first
PR landed the substrate; the next milestones land the authority.

The critical path, in order of dependency:

1. **Finish structural containers** — `Map<K,V>` keys, container
   refinement predicates, real `containers.dag` definitions. Without
   `List<Bit> where length(8)` carrying its constraint, nothing
   downstream can compose width.

2. **Make width/scalar-kind/encoding derivation compositional** —
   `Byte` gets width(8) from `8 × Bit`, `Word32` gets width(32)
   from `4 × Byte`, aliases inherit. This is the structural
   watershed: the system can recover all backend-relevant facts
   from composition alone.

3. **Wire the merged registry into production emit and replace
   nominal fallback tables with language-model resolution** — the
   compiler crosses the line from "structural path exists" to
   "structural path is the only path." Language models as IR
   (see Backend Language Models section) replace `emit_identity_type`.

4. **Derive cardinality from type DAGs** instead of storing it on
   ports.

5. **Finish structural coercion** — explicit translation rules,
   brand policy, downcast acknowledgment syntax.

6. **Shrink the kernel** — `String`, `Int`, `Float` stop being
   identity placeholders even in kernel-only contexts.

That is the line between "we have a structural type system in
principle" and "the compiler actually lives on it."

#### Phase A: Structural Containers (Gap 8) — PARTIALLY DONE

**A1. DONE.** `resolve_field_type_dag()` handles
`TypeExpr::Generic(name, params)`:
- `List<T>` → `type_lib::list(resolve_field_type_dag(T, registry))`
- `Option<T>` → `type_lib::optional(resolve_field_type_dag(T, registry))`
- `Map<K,V>` → `type_lib::map(resolve(V))` (**key erased — A1a**)
- `Set<T>` → `type_lib::set(resolve_field_type_dag(T, registry))`

**A1a. DEFERRED.** Map key type erasure. `type_lib::map()` only
accepts a value DAG. Fix requires extending to `map(key_dag, val_dag)`
and updating `ContainerShape::Map`, `TypeShape`, structural
compatibility, and all downstream consumers.

**A2. DEFERRED.** Refinement predicates on containers.
`List<Bit> where length(8)` → list DAG without Length(8) constraint.
Needed for compositional width derivation (Phase B).

**A3. DEFERRED.** Replace `containers.dag` documentation-only comments
with structural definitions.

**A4. DEFERRED.** Verify `Byte.bits` resolves to structural
`List(element: Bit(width(1))) where Length(8)` — blocked on A2.

#### Phase B: Compositional Width Derivation (Gap 9)

Make `derive_structural_properties()` compose width from structure.

**B1.** Width from fixed-length homogeneous containers:
- Product with single field `List<T> where length(N)` where T has
  width W → derived width = N × W
- This gives `Byte` width 8, `Word32` width 32, etc.

**B2.** Width inheritance through aliases:
- `UInt8 = Byte where unsigned, arithmetic` → inherit Byte's width 8
- `Float32 = Word32 where domain(...)` → inherit Word32's width 32

**B3.** Recursive width resolution:
- `derive_structural_properties()` already recurses into SubDags
- After Phase A, SubDags contain structural container DAGs
- Width derivation follows: `Word32` → field `bytes: List<Byte>
  where length(4)` → element `Byte` → field `bits: List<Bit> where
  length(8)` → element `Bit` → width(1) → Byte width = 8 × 1 = 8
  → Word32 width = 4 × 8 = 32

**B4.** Verify: `derive_structural_properties()` on the merged
registry's `UInt8` returns `width: Some(8), signed: Some(false),
arithmetic: true`. Same for `Float32` → `width: Some(32)`.

#### Phase C: Structural Kernel Types (Gap 1) — PARTIALLY DONE

**C1. PARTIAL.** `register_kernel_types()` has structural DAGs for:
- `Bool` → Coproduct(True: Unit, False: Unit)
- `Bytes` → List\<Byte\>
- `Secret` → Branded\<String\>

`String`, `Int`, `Float` remain identity in kernel but are overridden
by DSL merge (`string_type.dag`, `integer.dag`, `float.dag`).

**C2. DONE.** `ratchet_identity_types_in_core_registry` test documents
the baseline: 13 allowed identity types in `with_core_types()`. The
merged registry has fewer (DSL overrides String, Int, Float, Credential).

**C3. DONE.** `register_core_types()` identity placeholders reduced:
Platform → coproduct, Timestamp → refined Int, NetworkHandle → Unit,
ShellResponse/FileResponse/RestResponse/HttpResponse → products.

#### Phase D: Backend Language Models + Registry Wiring (Gaps 2-3)

**D1.** Thread `CompileOutput::merged_type_registry()` through the
codegen pipeline so production callers use
`lower_to_*_with_registry(Some(reg))`.

**D2.** Construct language models as `Dag<TypeOp>` instances using
`type_lib` helpers. Each language model is a collection of type DAGs
whose predicates describe the target language's representational
capabilities. No separate `LanguageModel` struct — the IR is the
model. See "Bootstrap: Language Models as IR" in the Backend
Language Models section.

Hierarchy (following compilation target chains — each node is a
citable spec):
- ISA layer: `Dag<TypeOp>` with Width/Signed/Domain predicates
  (Ref: x86-64 SDM, ARM ARM, RISC-V spec)
- C / Verilog: extend ISA with language-specific type constructs
  (Ref: ISO/IEC 9899:2018, IEEE 1364-2005)
- Rust / Go / C++: extend C or ISA with higher-level constructs
  (Ref: The Rust Reference, Go Language Spec, ISO/IEC 14882:2020)

**D3.** Populate language models for Rust, Go, C by mechanical
extraction from the existing match tables in `emit_identity_type()`,
`emit_platform_type()`, and container mapping. Each entry becomes a
`Dag<TypeOp>` with:
- Predicate nodes (Width, Signed, Domain, Arithmetic) defining what
  the target type represents
- A Brand or Identity node carrying the syntax string
- SubDag composition for containers and composite patterns

**D4.** Implement structural resolution:
`resolve(source_dag: &Dag<TypeOp>, model: &[Dag<TypeOp>]) -> String`.
The resolver structurally matches the source type's predicates
against each language model entry's predicates, finds the most
specific match, and returns the syntax string. Recursive
decomposition: if no match at the current structural level, peel
one level from the source type and retry.

**D5.** Replace `emit_platform_type()` and `emit_identity_type()`
with calls to the structural resolver.

**D6.** Delete `emit_identity_type()`, `map_primitive()`,
`try_refined_to_rust()`, `map_to_c_type_static()`.

**D7.** Serialize language models as JSON IR (`backend/rust.ir.json`,
etc.) so they can be loaded as data rather than constructed in Rust
code. This moves the source of truth from Rust to data files.

#### Phase E: Structural Coercion (replaces legacy coercion model)

**Open design decisions (deferred to next PR):**

1. **Derivation ≠ coercion.** A derivation path (Bit → Byte) does not
   imply a free cast. `Bit → Byte` is composition (8 bits), not
   supertype. `Int32 → Float32` needs conversion semantics (same width,
   different interpretation). The coercion search space is the derivation
   graph, but each edge needs an explicit structural translation rule.

2. **Brand semantics.** `structural_shapes_compatible()` currently
   strips brands when comparing `Brand(_, inner) → other`. This makes
   `Secret → String` structurally compatible, which may be wrong for
   sensitive types. Options: (a) brands are one-way refinements
   (current), (b) brands are strict nominal barriers, (c) per-brand
   policy. Decision needed before implementing coercion.

3. **Post-hoc validation.** Legacy coercion edges are gone. The new
   `is_compatible()` uses structural shape comparison + predicate
   entailment. If any graph construction path bypasses the builder
   (deserialization, manual DAG construction, test fixtures), there
   is no post-hoc coercion safety net. Audit needed.

**E1.** Implement `structural_coercion_path(from_dag, to_dag)` using
explicit translation rules over structure, not automatic derivation
path casts.

**E2.** Replace `is_compatible()` with structural coercion check.

**E3.** Add `.dag` syntax for downcast acknowledgment.

**E4.** Delete remaining legacy coercion infrastructure.

#### Phase F: Fail-Loud + Cleanup (Gaps 4-7, 10)

**F1.** Replace silent identity fallbacks with `Result` returns on
the structural path. Keep silent fallbacks only in kernel bootstrap.

**F2.** Eliminate remaining `Guard` references in executor.

**F3.** Derive port cardinality from type DAGs; restrict
`Port.cardinality` to `pub(crate)`.

**F4.** Replace `register_core_types()` coproduct lists with
DSL-sourced registration via `merge_dsl_types()`.

**F5.** Surface topological sort cycles as structural errors.

**F6.** Model transport rewrite tables as data.

#### Future: Behavior Declarations (not this PR)

**G1.** Add `behavior` and `implements` as DSL constructs.
**G2.** Auto-generate property-based tests from `law` declarations.
**G3.** Migrate `Cardinality`, `ContentEncoding`, `Predicate` lattice
impls from Rust to DSL `implements` clauses.

---

## Verification: Final Acceptance Criteria

When Phases A-F are complete:

**Deletions verified** (all return 0 results from `rg --type rust`):
- `emit_identity_type`
- `map_primitive`
- `try_refined_to_rust`
- `map_to_c_type_static`
- `Guard::` (in non-test code)
- `Port::with_cardinality`
- `CoercionEdge`
- `register_coercion_edge`
- `TypeContract`
- `base_type_upcasts_to`

**Structural invariants:**
- Every type in `register_kernel_types()` resolves to a multi-node
  structural DAG (not a single Identity node)
- `resolve_field_type_dag()` handles `TypeExpr::Generic` and produces
  structural container DAGs (not identity strings)
- `derive_structural_properties()` derives width compositionally
  (Byte → 8, Word32 → 32, UInt8 → 8, Float32 → 32)
- All type emission flows through structural pattern matching on
  `TypeShape`, not string-name matching
- Type coercion is determined by structural derivation chain walk,
  not by manual edge registration or hardcoded name matching
- Downcasts require explicit `.dag` acknowledgment
- Adding a new backend requires only defining a language model
  that imports from the appropriate level of the compilation chain
- Adding a new type requires only a `.dag` definition
- No type's properties (width, signedness, cardinality) are
  determined by string matching
- Unresolved type references fail compilation on the structural path

**Behavioral invariants:**
- `cargo test --workspace` passes
- `cargo clippy --all-targets -- -D warnings` passes
- All codegen tests produce identical output
- All existing `.dag` files compile without warnings
