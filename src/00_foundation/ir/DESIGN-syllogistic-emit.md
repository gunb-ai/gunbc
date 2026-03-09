# Syllogistic Type Emission

## Problem

The emit layer currently maintains hardcoded name-match tables (`emit_identity_type`)
that map DSL type names to target-language syntax. These tables must be replicated
per backend and updated in lockstep with the type vocabulary. This violates the
project invariant against hardcoded lists and prevents adding new backends without
modifying the compiler.

The structural predicate path (`emit_platform_type`) already works for numeric types
— Width + Signed + Arithmetic predicates derive correct syntax for Rust, Go, and C
without name matching. But non-numeric types (String, Bool, Bytes, containers) still
fall through to the name-match table because they lack structural predicates.

## Goal

Given any structural type DAG and any target backend, emit valid native syntax.
No name-match tables. The type's structure determines its representation; the
backend determines the syntax.

## Core Principle: Derivation Chains

Every type is a structural derivation from `Classical` (the two-valued truth type):

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

This already exists in `dsl/std/`. The derivation chain is the type.

## Backend Emission: Structural Pattern Matching

A backend does not map names. It pattern-matches on structure and selects the
highest-level native construct that covers the structural pattern.

### Worked Example 1: Int32

**Structure:** Word32 where signed, arithmetic
**Resolved predicates:** Width(32), Signed, Arithmetic

| Backend   | Pattern match                        | Emits      |
|-----------|--------------------------------------|------------|
| Rust      | Arithmetic + Signed + Width(32)      | `i32`      |
| Go        | Arithmetic + Signed + Width(32)      | `int32`    |
| C         | Arithmetic + Signed + Width(32)      | `int32_t`  |
| Verilog   | Signed + Width(32)                   | `reg signed [31:0]` |
| MIPS      | Width(32)                            | `$t0` (word register) |

**Current state:** Works via `emit_platform_type`. No name matching needed.

### Worked Example 2: Float64

**Structure:** Word64 where domain("ieee754_binary64"), arithmetic
**Resolved predicates:** Width(64), Domain("ieee754_binary64"), Arithmetic

| Backend   | Pattern match                        | Emits      |
|-----------|--------------------------------------|------------|
| Rust      | Domain(ieee754) + Width(64)          | `f64`      |
| Go        | Domain(ieee754) + Width(64)          | `float64`  |
| C         | Domain(ieee754) + Width(64)          | `double`   |
| Verilog   | Domain(ieee754) + Width(64)          | `real`     |

**Current state:** Works via `emit_platform_type`. No name matching needed.

### Worked Example 3: String

**Structure:** `{ bytes: List<Byte>, encoding: Encoding }`
**Resolved shape:** Product with fields `bytes: List<Width(8)+Unsigned>`, `encoding: Coproduct(Encoding)`

A backend recognizes this pattern as "encoded byte sequence" and emits its native
string type:

| Backend   | Pattern match                             | Emits         |
|-----------|-------------------------------------------|---------------|
| Rust      | Product(bytes: List<u8>, encoding: _)     | `String`      |
| Go        | Product(bytes: List<u8>, encoding: _)     | `string`      |
| C         | Product(bytes: List<u8>, encoding: _)     | `const char*` |
| Verilog   | (no native string)                        | decompose to `reg [7:0] mem []` |

**Current state:** Falls through to `emit_identity_type("String")` name match.
The structural information exists in `string_type.dag` but the emitter doesn't
walk it.

### Worked Example 4: Bool

**Structure:** `Classical` = `True | False` (coproduct with two unit variants)
**Resolved shape:** Coproduct with 2 unit variants

A backend recognizes "two-variant unit coproduct" as boolean:

| Backend   | Pattern match                        | Emits    |
|-----------|--------------------------------------|----------|
| Rust      | Coproduct(2 units)                   | `bool`   |
| Go        | Coproduct(2 units)                   | `bool`   |
| C         | Coproduct(2 units)                   | `bool`   |
| Verilog   | Coproduct(2 units) = Width(1)        | `wire`   |

**Current state:** Falls through to `emit_identity_type("Bool")` name match.
`Classical` is defined in `logic.dag` as `True | False` but the emitter doesn't
recognize it structurally.

### Worked Example 5: Bytes

**Structure:** `List<Byte>` = `List<{ bits: List<Bit> where length(8) }>`
**Resolved shape:** Container(List) with element Width(8) + Unsigned

| Backend   | Pattern match                        | Emits      |
|-----------|--------------------------------------|------------|
| Rust      | List<Width(8)+Unsigned>              | `Vec<u8>`  |
| Go        | List<Width(8)+Unsigned>              | `[]byte`   |
| C         | List<Width(8)+Unsigned>              | `uint8_t*` |

**Current state:** Falls through to `emit_identity_type("Bytes")` name match.

### Worked Example 6: Optional<Int32>

**Structure:** Container(Optional) wrapping Width(32) + Signed + Arithmetic
**Resolved shape:** Optional(Platform(i32))

| Backend   | Pattern match                        | Emits            |
|-----------|--------------------------------------|------------------|
| Rust      | Optional(T)                          | `Option<i32>`    |
| Go        | Optional(T)                          | `*int32`         |
| C         | Optional(T)                          | `int32_t*` (nullable pointer) |

**Current state:** Container wrapping works. Inner type resolution depends on
whether the registry is threaded through (currently not in production paths).

### Worked Example 7: List<String>

**Structure:** Container(List) wrapping Product(bytes: List<Byte>, encoding: Encoding)

| Backend   | Pattern match                             | Emits           |
|-----------|-------------------------------------------|-----------------|
| Rust      | List<Product(bytes+encoding)>             | `Vec<String>`   |
| Go        | List<Product(bytes+encoding)>             | `[]string`      |
| C         | List<Product(bytes+encoding)>             | `const char**`  |

**Current state:** Falls through to name matching for the inner `String`.

## What the Backend Needs to Know

A backend is a function:

```
emit: (TypeShape, BackendRules) -> NativeSyntax
```

`BackendRules` is a set of structural pattern recognizers, ordered by priority:

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

The emitter walks the rules top-down. First match wins. If no rule matches,
decompose the type one level and try again on its constituents.

This is the key insight: **backends are configurable rule sets, not hardcoded
match arms in the compiler**. A Verilog backend would have different rules
(no native string, no native containers) and would decompose further down
the chain.

## Requirements

### R1: Structural identity for all kernel types

Every type currently in `register_kernel_types()` (`String`, `Bool`, `Int`,
`Float`, `Bytes`, `Secret`) must have a structural DAG, not an identity
placeholder. The DSL files already define these structures — the registry
must compose them.

- `String` = `string_type.dag` definition (Product with bytes + encoding)
- `Bool` = `logic.dag` Classical (Coproduct True|False)
- `Int` = `integer.dag` Int64 alias chain
- `Float` = `float.dag` Float64 alias chain
- `Bytes` = `List<Byte>` (container wrapping bit.dag Byte)
- `Secret` = `String where brand("Secret")` (branded string)

### R2: Backend as a rule set, not a match table

Replace `emit_identity_type()` (closed name-match per backend) with a
pattern-matching engine that walks `TypeShape` against backend-specific
structural rules.

Backend rules should be data, not code. Ideally expressible in `.dag` or
at minimum in a declarative Rust structure — not a `match name { ... }`
block.

### R3: Recursive decomposition

When no backend rule matches a type's top-level shape, the emitter must
decompose one structural level and try again. This handles cases like
Verilog encountering `String` — no native pattern matches, so it decomposes
to `List<Byte>` (still no match), then to an array of `reg [7:0]`.

### R4: Eliminate `emit_identity_type` entirely

After R1-R3, the name-match table has no remaining callers. Delete it.
All type emission flows through structural pattern matching.

### R5: Eliminate `map_primitive` and `try_refined_to_rust` in type_codegen.rs

These are the codegen-layer copies of the same name-match logic. Once the
registry is threaded through `type_expr_to_rust_with_registry`, these become
dead code.

### R6: Eliminate `map_to_c_type_static` in lower_c.rs

Same treatment as R5 for the C backend's duplicate table.

### R7: Transport rewrite tables (separate concern)

The `rewrite_transport_call` tables in each backend (15 name pairs × 3
backends) are a separate domain-modeling opportunity. These map abstract
transport operation names to target-language runtime function names. Could
be modeled as data declarations per backend rather than match arms, but
this is orthogonal to type emission and can be addressed separately.

## Static-Analysis-Only Predicates

Some predicates exist for algebraic reasoning and do not affect emission:

- **Length**: `List<Bit> where length(8)` — the length constraint is for
  type-level cardinality checking, not for emitting a different type
- **Range**: `Int where range(0, 1114111)` — bounds checking, not emission
- **Unique**: `List<T> where unique` — distinguishes Set from List at the
  type level but may emit the same native type

These predicates participate in type compatibility checking and static
analysis but are transparent to the emitter. The emitter sees them, ignores
them, and emits based on the structural shape underneath.

## Non-Requirements

- **DSL-defined backend rules in this PR**: Backend rules can start as
  declarative Rust structures. Moving them to `.dag` files is future work.
- **Full Verilog/MIPS backend**: These are motivating examples, not
  deliverables.
- **Removing kernel type bootstrap**: `register_kernel_types()` can still
  exist for the compiler bootstrap sequence. The requirement is that the
  registered DAGs are structural, not identity placeholders.

## Current State vs Target

| Type    | Current DAG            | Target DAG                     | Blocks   |
|---------|------------------------|--------------------------------|----------|
| String  | `identity("String")`   | Product(bytes, encoding)       | R1       |
| Bool    | `identity("Bool")`     | Coproduct(True, False)         | R1       |
| Int     | `identity("Int")`      | Int64 alias → Word64 + Signed  | R1       |
| Float   | `identity("Float")`    | Float64 alias → Word64 + ieee  | R1       |
| Bytes   | `identity("Bytes")`    | List<Byte>                     | R1       |
| Secret  | `identity("Secret")`   | String + Brand("Secret")       | R1       |
| Int32   | structural (predicates)| (already correct)              | --       |
| Float64 | structural (predicates)| (already correct)              | --       |

## Verification

After implementation:

1. `emit_identity_type` has zero callers — deleted
2. `map_primitive` has zero callers — deleted
3. `map_to_c_type_static` has zero callers — deleted
4. Adding a new backend requires only defining a `BackendRules` set
5. Adding a new type requires only a `.dag` definition — no compiler changes
6. `cargo test --workspace` green
7. All existing emit tests produce identical output (no behavioral regression)
