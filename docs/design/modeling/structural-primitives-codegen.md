# Design: Structural Primitives for Consistent Cross-Backend Codegen

**Status**: Draft
**Date**: 2026-02-23
**Related tasks**: Lane 1 (Type System), Foundation Close-Out Lane A
**Reference**: `docs/handbook.md` § "Compositional Modeling Philosophy"

## Problem

The four emit backends (Rust, Go, C, MIPS) each contain **independent hardcoded
type mappings** with no shared derivation:

```rust
// lower_rust.rs
fn map_to_rust_type(abstract_type: &str) -> String {
    match abstract_type {
        "String" | "Path" => "String".to_string(),
        "Bool" | "bool" => "bool".to_string(),
        "Int" | "i64" | "I64" => "i64".to_string(),
        other => "serde_json::Value".to_string(),  // ← catch-all
    }
}

// lower_go.rs
fn map_to_go_type(abstract_type: &str) -> String {
    match abstract_type {
        "String" | "Path" => "string".to_string(),
        "Bool" | "bool" => "bool".to_string(),
        "Int" | "i64" | "I64" => "int64".to_string(),
        other => "interface{}".to_string(),          // ← catch-all
    }
}

// lower_c.rs — returns CType, not String
fn map_to_c_type(abstract_type: &str) -> CType {
    match abstract_type {
        "Bool" | "bool" => CType::Int(CIntKind::Int),  // ← C has no bool
        "Int" | "i64" => CType::Int(CIntKind::Fixed(64)),
        other => CType::Ptr(Box::new(CType::Void)),     // ← catch-all
    }
}
```

The MIPS backend receives C IR and computes sizes from CType — it has no
user-facing type mapping at all.

### Consequences

1. **Semantic drift**: Each backend independently decides what `Bool` means.
   Rust maps it to `bool` (1 byte, values `true`/`false`). C maps it to `int`
   (4 bytes, values 0/non-zero). There is no shared contract for what values
   are valid, how comparison works, or what the canonical encoding is.

2. **Catch-all hiding**: When a new type is added to the type registry but not
   to all four backend match statements, it silently falls to `serde_json::Value`
   / `interface{}` / `void*`. There is no compile-time or test-time enforcement
   that all backends handle all types.

3. **No structural derivation**: Product types, coproduct types, branded types,
   and refined types are all registered as `Dag<TypeOp>` in the type registry,
   but the backends ignore that structure — they pattern-match on the string name.
   A `ContentEncoding` coproduct with 6 variants is mapped to `serde_json::Value`
   in Rust instead of being derived as an enum.

4. **Container inconsistency**: Rust handles `List<T>` but not `Optional<T>` or
   `Map<K,V>`. Go handles all three. C handles `List<T>` only. The coverage gap
   is invisible — each backend silently downgrades missing patterns to the catch-all.

## Principle: Derive from structure, don't match on names

The type registry already stores types as `Dag<TypeOp>`. A `Bool` today is:

```rust
pub fn bool() -> Dag<TypeOp> {
    identity("Bool")  // single Identity node, ports typed "Bool"
}
```

This tells the backend nothing. It's an opaque name. The backend must know
independently what "Bool" means. If we instead define Bool structurally:

```rust
pub fn bool() -> Dag<TypeOp> {
    coproduct("Bool", vec![("True", "Unit"), ("False", "Unit")])
}
```

Now the type DAG **carries its own definition**. Any backend can inspect the
DAG, see that Bool is a 2-variant coproduct with no payload, and derive the
appropriate representation:

| Backend | Derived from `Coproduct([True: Unit, False: Unit])` |
|---------|------------------------------------------------------|
| Rust    | `enum Bool { True, False }` or `bool` (optimization) |
| Go      | `type Bool int; const (True Bool = iota; False)` |
| C       | `typedef enum { BOOL_TRUE, BOOL_FALSE } Bool;` |
| MIPS    | `.word` 0/1, fits in single register `$t0` |

The derivation rule is: **Coproduct with N unit variants → language-native enum
or discriminated tag**. The backend doesn't need to know "Bool" — it reads the
structure.

## Design: Structural Primitive Decomposition

### Step 1: Decompose primitives into structural type DAGs

Replace opaque `identity()` primitives with structural definitions that encode
their algebraic shape. Start from the bottom:

```
Bit     = Coproduct([Zero: Unit, One: Unit])
Bool    = Coproduct([True: Unit, False: Unit])   — isomorphic to Bit
Byte    = Product([b7: Bit, b6: Bit, ..., b0: Bit])  — 8 bits
Nat     = Coproduct([Zero: Unit, Succ: Nat])     — Peano (theoretical only)
Int     = Brand("Int", identity("Int"))           — platform-width signed integer
UInt8   = Brand("UInt8", Byte)                    — unsigned 8-bit
Int64   = Brand("Int64", identity("Int64"))       — fixed-width signed 64-bit
Float64 = Brand("Float64", identity("Float64"))   — IEEE 754 double
Char    = Brand("Char", UInt32)                   — Unicode scalar value
String  = List<Char>                              — sequence of characters
Bytes   = List<Byte>                              — sequence of bytes
```

**Practical boundary**: We do NOT recursively decompose `Int64` into 64 `Bit`
products or `Nat` into Peano successors at the codegen level. That would be
correct but impractical — no backend would emit 64 bit-fields for an integer.
Instead, we use a **two-tier** model:

- **Tier 1 (Structural)**: Types whose structure the backend reads and derives
  from. `Bool`, `Byte`, `ContentEncoding`, user-defined products/coproducts.
- **Tier 2 (Platform-primitive)**: Types that are structurally opaque but carry
  a `PlatformHint` annotation telling backends to use the native representation.
  `Int64`, `Float64`, `Char`.

### Step 2: Add `PlatformHint` metadata to TypeOp

A new `MetadataPayload` variant carries the machine representation contract:

```rust
enum MetadataPayload {
    // ... existing variants ...

    /// Platform-level representation hint for code generation.
    ///
    /// Tells backends: "this type maps to a well-known machine primitive
    /// with these properties." Backends derive their native type from
    /// the hint, not from the type name string.
    PlatformRepr(PlatformRepr),
}

/// Machine representation contract for platform-primitive types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlatformRepr {
    /// Minimum bit width required to represent all values.
    pub bits: u16,
    /// Signedness.
    pub signed: bool,
    /// IEEE 754 floating-point (changes representation rules).
    pub float: bool,
    /// Whether the type has exactly 2^bits distinct values (integers)
    /// or a continuous range (floats).
    pub discrete: bool,
}
```

Platform-primitive type DAGs carry this metadata:

```rust
pub fn int64() -> Dag<TypeOp> {
    let mut dag = Dag::new();
    dag.add_node(Node::opaque(
        "repr",
        vec![Port::scalar("in", "Int64")],
        vec![Port::scalar("out", "Int64")],
        TypeOp::Meta(MetadataPayload::PlatformRepr(PlatformRepr {
            bits: 64, signed: true, float: false, discrete: true,
        })),
    ));
    dag
}

pub fn float64() -> Dag<TypeOp> {
    let mut dag = Dag::new();
    dag.add_node(Node::opaque(
        "repr",
        vec![Port::scalar("in", "Float64")],
        vec![Port::scalar("out", "Float64")],
        TypeOp::Meta(MetadataPayload::PlatformRepr(PlatformRepr {
            bits: 64, signed: true, float: true, discrete: false,
        })),
    ));
    dag
}
```

### Step 3: Shared type derivation function

Replace the per-backend `map_to_*_type()` functions with a shared two-phase
derivation:

**Phase 1: Read the type DAG structure (shared)**

```rust
/// Structural classification derived from a type DAG.
pub enum TypeShape {
    /// Platform primitive with machine representation contract.
    Platform(PlatformRepr),
    /// Coproduct (tagged union) with named variants.
    Coproduct(Vec<(String, TypeShape)>),
    /// Product (record) with named fields.
    Product(Vec<(String, TypeShape)>),
    /// Branded wrapper around an inner type.
    Brand(String, Box<TypeShape>),
    /// Container: Optional, List, Set, Map.
    Container(ContainerShape),
    /// Opaque/unresolved (legacy fallback — should shrink over time).
    Opaque(String),
}

pub enum ContainerShape {
    Optional(Box<TypeShape>),
    List(Box<TypeShape>),
    Set(Box<TypeShape>),
    Map(Box<TypeShape>, Box<TypeShape>),
}

/// Extract the structural shape from a registered type DAG.
pub fn type_shape(registry: &TypeRegistry, type_id: &TypeId) -> TypeShape {
    // Walk the Dag<TypeOp>, classify by root node's TypeOp variant.
    // Recurse into Product/Coproduct field types via registry lookup.
    // ...
}
```

**Phase 2: Backend-specific rendering (per-backend)**

Each backend implements a single function that pattern-matches on `TypeShape`:

```rust
// Rust backend
fn render_rust_type(shape: &TypeShape) -> String {
    match shape {
        TypeShape::Platform(repr) => match (repr.bits, repr.signed, repr.float) {
            (64, true, false) => "i64".to_string(),
            (64, true, true)  => "f64".to_string(),
            (32, false, false) => "u32".to_string(),
            (8, false, false)  => "u8".to_string(),
            // ...
        },
        TypeShape::Coproduct(variants) if is_unit_coproduct(variants) => {
            // All-unit variants: enum { A, B, C }
            // Render as Rust enum with no fields.
        },
        TypeShape::Coproduct(variants) => {
            // Mixed variants: enum { A(X), B(Y) }
        },
        TypeShape::Product(fields) => {
            // struct { field: Type, ... }
        },
        TypeShape::Container(ContainerShape::List(inner)) => {
            format!("Vec<{}>", render_rust_type(inner))
        },
        TypeShape::Container(ContainerShape::Optional(inner)) => {
            format!("Option<{}>", render_rust_type(inner))
        },
        TypeShape::Container(ContainerShape::Map(key, val)) => {
            format!("HashMap<{}, {}>", render_rust_type(key), render_rust_type(val))
        },
        TypeShape::Brand(name, inner) => {
            // Newtype pattern: struct Name(inner)
            // Or: type alias if transparent.
        },
        TypeShape::Opaque(name) => format!("serde_json::Value /* {} */", name),
    }
}

// Go backend
fn render_go_type(shape: &TypeShape) -> String {
    match shape {
        TypeShape::Platform(repr) => match (repr.bits, repr.signed, repr.float) {
            (64, true, false) => "int64".to_string(),
            (64, true, true)  => "float64".to_string(),
            (32, false, false) => "uint32".to_string(),
            (8, false, false)  => "byte".to_string(),
            // ...
        },
        TypeShape::Coproduct(variants) if is_unit_coproduct(variants) => {
            // Go: type T int; const (A T = iota; B; C)
        },
        TypeShape::Container(ContainerShape::List(inner)) => {
            format!("[]{}", render_go_type(inner))
        },
        TypeShape::Container(ContainerShape::Optional(inner)) => {
            format!("*{}", render_go_type(inner))
        },
        TypeShape::Container(ContainerShape::Map(key, val)) => {
            format!("map[{}]{}", render_go_type(key), render_go_type(val))
        },
        TypeShape::Opaque(name) => "interface{}".to_string(),
    }
}

// C backend
fn render_c_type(shape: &TypeShape) -> CType {
    match shape {
        TypeShape::Platform(repr) => match (repr.bits, repr.signed, repr.float) {
            (64, true, false) => CType::Int(CIntKind::Fixed(64)), // int64_t
            (64, true, true)  => CType::Float(CFloatKind::Double),
            (8, false, false)  => CType::Char,                    // uint8_t
            _ => CType::Int(CIntKind::Int),
        },
        TypeShape::Coproduct(variants) if is_unit_coproduct(variants) => {
            // C: typedef enum { A, B, C } T;
            CType::Int(CIntKind::Int)
        },
        TypeShape::Container(ContainerShape::List(inner)) => {
            CType::Ptr(Box::new(render_c_type(inner)))
        },
        TypeShape::Opaque(_) => CType::Ptr(Box::new(CType::Void)),
    }
}
```

### Step 4: Bool as the first structural primitive

Start with Bool because it's the simplest non-trivial case and touches all
backends:

**Before** (current):
```rust
// type_lib.rs
pub fn bool() -> Dag<TypeOp> {
    identity("Bool")  // opaque
}

// lower_rust.rs:  "Bool" => "bool"
// lower_go.rs:    "Bool" => "bool"
// lower_c.rs:     "Bool" => CType::Int(CIntKind::Int)
// MIPS:           inherits C's int (4 bytes, 0/non-zero)
```

**After** (structural):
```rust
// type_lib.rs
pub fn bool() -> Dag<TypeOp> {
    coproduct("Bool", vec![("True", "Unit"), ("False", "Unit")])
}
```

The shared derivation reads this as `Coproduct([True: Unit, False: Unit])` —
a 2-variant unit coproduct. Each backend has a well-defined rule for this
shape:

| Backend | Rule for 2-variant unit coproduct | Emitted type |
|---------|-----------------------------------|--------------|
| Rust    | Use native `bool` (compiler optimization: `Coproduct` with ≤256 unit variants → `u8` backing, 2-variant special case → `bool`) | `bool` |
| Go      | `type Bool int; const (True Bool = iota; False)` or native `bool` | `bool` |
| C       | `typedef enum { BOOL_TRUE = 0, BOOL_FALSE = 1 } Bool;` or `_Bool` (C99) | `_Bool` or `int` |
| MIPS    | Single register, 0 = False, 1 = True | `.word` in `$t0` |

**Key insight**: The backend doesn't need to special-case "Bool" by name. It
sees a 2-variant unit coproduct and applies the general rule. If someone defines
`type Bit = Coproduct([Zero, One])`, it gets the **same** representation. This
is the consistency guarantee.

### Step 5: Representation decisions (the hard part)

These are the decisions that require design choices — the things worth
"struggling with up front":

#### 5a. Bool encoding: 0/1 or 0/non-zero?

C traditionally uses `int` where 0 is false and any non-zero is true. Rust uses
`bool` where only 0 and 1 are valid bit patterns. The structural definition
`Coproduct([True, False])` implies exactly 2 valid values, which matches Rust's
`bool` semantics, not C's liberal interpretation.

**Decision**: A Bool Coproduct with 2 variants has exactly 2 valid values.
Backends that use wider representations (C's `int`) must validate on input
boundaries, not at every use site. This is consistent with the compositional
principle: the type's invariants are enforced at the boundary where the value
enters the system.

#### 5b. Coproduct discriminant encoding

A coproduct `Coproduct([A: T1, B: T2, C: T3])` needs a discriminant. Options:

1. **Integer tag** (0, 1, 2): Simple, fixed-size, all backends support it.
2. **String tag** ("A", "B", "C"): Self-describing but expensive.
3. **Bit-packed tag**: Minimal bits (`⌈log2(N)⌉`), optimal for MIPS/C.

**Decision**: Integer tag. The tag width is derived from variant count:
- ≤ 256 variants: `u8` / `uint8_t` / `.byte`
- ≤ 65536 variants: `u16` / `uint16_t` / `.half`
- Otherwise: `u32` / `uint32_t` / `.word`

Backends may optimize (Rust can use `enum` directly; C can use `typedef enum`).
The structural contract is: the discriminant is an unsigned integer whose range
is `[0, N)` where N is the variant count.

#### 5c. Product field layout

A product `Product([name: String, age: Int64, active: Bool])` needs field
ordering. Options:

1. **Declaration order**: Fields laid out in the order declared.
2. **Alignment-optimal**: Fields reordered for minimal padding (C struct packing).
3. **Alphabetical**: Deterministic regardless of declaration order.

**Decision**: Declaration order, with a `Meta(FieldLayout::Packed)` annotation
available for alignment optimization. The default preserves author intent.
C/MIPS backends may add padding per their ABI requirements, but the logical
field order is always declaration order.

#### 5d. String encoding

`String = List<Char>` is the structural truth, but backends disagree:
- Rust: UTF-8 encoded `String` (variable-width, length-prefixed)
- Go: UTF-8 encoded `string` (immutable, length-prefixed)
- C: null-terminated `char*` (ASCII or locale-dependent)
- MIPS: `.asciiz` (null-terminated ASCII)

**Decision**: String carries `PlatformRepr { encoding: UTF8 }` metadata.
Backends that natively support UTF-8 (Rust, Go) use their native string type.
Backends that don't (C, MIPS) use `const char*` / `.asciiz` with a documented
limitation that only ASCII is safe without a runtime UTF-8 library. The
structural definition (`List<Char>`) remains the canonical truth; the platform
hint is an optimization for practical codegen.

#### 5e. Opaque catch-all behavior

Currently, unknown types fall to `serde_json::Value` / `interface{}` / `void*`
silently. This must fail loudly.

**Decision**: `TypeShape::Opaque` is a **diagnostic**, not a silent fallback.
The codegen phase emits a warning (strict mode: error) when it encounters
Opaque. The goal is to shrink Opaque to zero over time as all types get
structural definitions.

## Migration Path

### Phase 0: TypeShape extraction (no behavioral change)

Add `TypeShape` enum and `type_shape()` function to `core/ir`. Wire it into
the emit pipeline alongside (not replacing) the existing `map_to_*_type()`
functions. Add tests proving the shape extraction matches the existing hardcoded
mappings for all registered types.

### Phase 1: Bool decomposition

Change `type_lib::bool()` from `identity("Bool")` to
`coproduct("Bool", vec![("True", "Unit"), ("False", "Unit")])`.

Update `type_shape()` to classify this as a 2-variant unit coproduct. Update
the existing `map_to_*_type()` functions to call `type_shape()` for Bool and
verify they emit the same output. This is a refactor — no behavioral change in
emitted code.

### Phase 2: PlatformRepr for Int/Float

Add `PlatformRepr` metadata. Change `type_lib::int()` and `type_lib::float()`
to carry `Meta(PlatformRepr { ... })`. Update `type_shape()` to read the
metadata. Verify backends derive the same types from the metadata that they
currently hardcode.

### Phase 3: Shared derivation replaces hardcoded mappings

Replace the `map_to_*_type()` match statements with `type_shape()` +
`render_*_type()`. The match-on-string-name functions become dead code.
Add exhaustiveness tests that fail when a registered type produces
`TypeShape::Opaque`.

### Phase 4: Product/Coproduct derivation

With the shared derivation in place, user-defined product and coproduct types
(e.g., `ContentEncoding`, `CliResult`) automatically get correct representations
in all backends instead of falling to the catch-all. This is the payoff: adding
a new type to the registry automatically generates correct Rust enum, Go const
block, C typedef enum, and MIPS layout — with zero per-backend code changes.

## Scope & Non-Goals

### In scope
- Structural decomposition of Bool (and later other primitives)
- `PlatformRepr` metadata for Int/Float/Char
- Shared `TypeShape` derivation used by all emit backends
- Backend-specific rendering from `TypeShape` (replacing hardcoded match)
- Exhaustiveness enforcement (Opaque is a diagnostic, not a silent fallback)

### Not in scope (future work)
- Peano naturals or fully recursive type definitions
- Custom memory allocators or ownership models per type
- Runtime reflection or type metadata in emitted code
- Bit-level layout control (e.g., bit-packing struct fields)

## Relationship to Other Tasks

- **Lane 1 (Type System)**: This design extends Lane 1's type algebra with
  codegen-visible structure. It does not change the TypeOp enum (only adds a
  MetadataPayload variant and decomposes existing Identity DAGs).
- **M8 (metadata separation)**: `PlatformRepr` is carried via `TypeOp::Meta`,
  which M8 introduces. This design depends on M8 being complete.
- **M14 (single inventory authority)**: Type-to-backend mappings are currently
  a form of duplicated inventory. This design consolidates them.
- **Foundation Close-Out Lane A**: This is a Lane A task — it eliminates the
  parallel truth between the type registry's structural definitions and the
  backends' hardcoded string-match tables.

## Examples

### Before: Adding a new enum type

```rust
// 1. Register in type_registry.rs
registry.register("HttpMethod", type_lib::coproduct("HttpMethod", vec![
    ("GET", "Unit"), ("POST", "Unit"), ("PUT", "Unit"), ("DELETE", "Unit"),
]));

// 2. MUST ALSO add to lower_rust.rs:
"HttpMethod" => "HttpMethod".to_string(),  // ← manual, easy to forget

// 3. MUST ALSO add to lower_go.rs:
"HttpMethod" => "HttpMethod".to_string(),  // ← manual, easy to forget

// 4. MUST ALSO add to lower_c.rs:
"HttpMethod" => CType::Int(CIntKind::Int), // ← manual, easy to forget

// 5. If you forget step 2-4, the type silently becomes serde_json::Value / void*
```

### After: Adding a new enum type

```rust
// 1. Register in type_registry.rs (same as before)
registry.register("HttpMethod", type_lib::coproduct("HttpMethod", vec![
    ("GET", "Unit"), ("POST", "Unit"), ("PUT", "Unit"), ("DELETE", "Unit"),
]));

// 2. Done. type_shape() reads the Coproduct, all backends derive:
//    Rust:  enum HttpMethod { Get, Post, Put, Delete }
//    Go:    type HttpMethod int; const (Get HttpMethod = iota; Post; Put; Delete)
//    C:     typedef enum { HTTP_METHOD_GET, HTTP_METHOD_POST, ... } HttpMethod;
//    MIPS:  .word 0/1/2/3 in register
```
