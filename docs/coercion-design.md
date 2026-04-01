# Coercion Design: Algebra Inhabitants and Type Sidecast

**Status:** Design sketch (no compiler changes)
**Branch:** `design/coercion-inhabitants`
**Depends on:** M2 (working Rust codegen), TypeRendering (E0c), Lane 1 Tier 3
**Feeds into:** TypeRendering (E0c) — coercion data is consumed by `build_type_rendering`

---

## Resolved Questions

**RQ-1 (was OQ-1): Shared coercion vocabulary.**
Resolved: shared types live in `std/coercion.dag`. Per-language files import
`TypeCheckpoint`, `InhabitantDecl`, `CallableRepr` and instantiate data only.
Language-specific extension fields (Rust's `is_copy`) are optional fields on
the shared types — `none` for languages where the concept doesn't exist.

**RQ-2 (was OQ-2): Scalar vs collection FreeMonoid.**
Resolved: checkpoint-first. `String` hits the checkpoint table before algebra
resolution. `FreeMonoid<Char>` as a user type correctly coerces to `Vec<char>`
via the algebra inhabitant — this is correct, not a bug. No `scalar: Bool` flag.

**RQ-3 (was OQ-3): NonEmptyList / NonEmptySet.**
Resolved: refinements of their base containers, not separate inhabitant families.
`NonEmptyList<T>` = `List<T> where non_empty`. The coerced type is `Vec<T>`;
the non-empty constraint is a refinement validation obligation (see Refinement
Contract below). Separate inhabitants would grow the authority surface.

**RQ-4 (was OQ-4): Callable context decisions.**
Resolved: derive from ValueContext + resolved capture/ownership facts. The
shared `CallableRepr` declares the base syntax (`fn(T) -> U` in Rust,
`Callable[[T], U]` in Python, `func(T) U` in Go). Context-dependent wrapping
(Rust's `Box<dyn Fn>` for owned closures, `Rc<dyn Fn>` for stored closures)
is applied by the emitter based on ValueContext, not declared as template
variants in the data.

**RQ-5 (was OQ-5): Operation coercion.**
Resolved: attach per-language operation renderings to algebra operation identity
from `std/algebra.dag`. Do NOT create a second free-form operation catalog.
Types first, operations second. Operation templates reference the algebra
operation's declaration identity, preventing parallel authorities.

**RQ-6 (was OQ-6): Refinement validation.**
Resolved: one contract. See "Refinement Contract" section below.

**RQ-7 (was OQ-7): TypeRendering coexistence.**
Resolved: TypeRendering IS the migration boundary. `build_type_rendering` reads
inhabitant data from `std/coercion.dag` instances. Coercion feeds TypeRendering
rather than replacing it. This matches the roadmap and deletes the current
heuristic type rendering path without a big-bang rewrite.

**RQ-8 (was OQ-8): Property-based test generation.**
Resolved: auto-generate refusal tests for every missing inhabitant. Auto-generate
positive law tests ONLY from explicit law metadata in `std/algebra.dag` — each
algebra field → law assertion. Hand-written tests reserved for exceptional cases.
`CoercionTest` as a manual per-language table is deleted.

---

## Remaining Open Questions

**OQ-1: Law metadata in std/algebra.dag.**
The test generation algorithm requires each algebra to declare its laws explicitly
(e.g., `is_associative`, `has_identity`, `is_commutative`). Currently
`std/algebra.dag` declares operations but not law metadata. What is the minimal
law annotation surface needed to auto-generate correct positive tests?

**OQ-2: TypeRendering variant coverage.**
The roadmap's E0c defines `TypeRendering` variants (`PrimitiveRendering`,
`ContainerRendering`, `MapRendering`, etc.). Does the set of `InhabitantDecl`
entries in a language file need to cover every `TypeRendering` variant, or can
some variants be structurally derived without an explicit inhabitant?

---

## Core Thesis

Type rendering is coercion. The compiler walks a .dag type composition tree
and sidecasts to the target language's type at the first level the language
declares an inhabitant for. Each language declares its algebra inhabitants
as .dag extdep data. The compiler sidecasts mechanically. Fail-loud when
no inhabitant exists.

**Scope of this design:** This covers the type-coercion slice of language
backends. The full backend still requires `LanguageSpec` (statement terminators,
declaration keywords, assignment syntax, lambda syntax, null-coalescing,
string interpolation, container syntax, indentation, lint/formatting rules).
Type coercion is data-driven; the full backend is broader.

## The Coercion Model

```
.dag type composition tree:

   Bit
    │
   BitWord<64>
    │
   Int ← OrderedRing<Word64>        ← Rust checkpoints here: i64
    │
   NonNegativeInt                     ← refinement: range(min: 0)
    │
   Port                               ← refinement: range(min: 1, max: 65535)
```

Rust says "I know Int → i64" and skips the intermediate composition.
SPICE might say "I know BitWord<64> → wire[63:0]" and checkpoint lower.
English might say "I know Int → 'integer'" and checkpoint at the same level
as Rust but with different representation.

### Resolution Order

```
For each .dag type T in target language L:

1. CHECKPOINT: Is T.name in L's type checkpoint table?
   YES → use declared type. DONE.

2. ALGEBRA: Does T inhabit an algebra A (from std/algebra.dag)?
   Is A in L's algebra inhabitant table?
   YES → apply template with T's type params (recursively coerced). DONE.

3. STRUCTURAL: Is T a Product (struct) or Coproduct (enum)?
   YES → coerce each field/variant recursively. DONE.

4. REFINEMENT: Is T a refinement of base type B?
   YES → coerce B (recursively). Attach validation per Refinement Contract. DONE.

5. FAIL: No coercion path found.
   → COMPILE ERROR: "Language L has no representation for type T
     (algebra: A). Add an inhabitant to extdeps/languages/L/types.dag."
```

### Identity Model (transitional)

The resolution algorithm currently keys on `T.name` (String) for checkpoints
and `algebra` (String) for inhabitants. This is the proxy identity model
that M4/Lane 1 is dissolving. The correct end state:

1. Resolution phase (at startup) converts `dag_name` strings to declaration
   node references — structural identity, not string equality.
2. After resolution, no semantic decision in the emitter or coercion engine
   uses `T.name` or string comparison. All decisions flow from resolved
   declaration edges.
3. The `String` fields in `TypeCheckpoint` and `InhabitantDecl` are the
   transitional form. M4/Lane 1 Tier 3 replaces them with `Node` edges
   to actual `.dag` type declarations.

The shared schema in `std/coercion.dag` documents this explicitly.

## Coercion Categories

### Category 1: Primitive Scalar Coercion

**What:** .dag abstract type → target language concrete type (direct name match)

**Mechanism:** Named type checkpoints (level 1 resolution)

**.dag source:**
```
fn add(a: Int, b: Int) -> Int { a + b }
```

**Resolution path:**
```
"Int" → checkpoint table → "i64"
```

**Target outputs:**

| Language | Type | Mechanism |
|----------|------|-----------|
| Rust | `i64` | Checkpoint: `Int → i64` |
| Python | `int` | Checkpoint: `Int → int` |
| Go | `int64` | Checkpoint: `Int → int64` |
| SPICE | `real` | Checkpoint: `Int → real` (or `wire[63:0]` for bit-level) |
| English | `integer` | Checkpoint: `Int → integer` |

**What this replaces:** `rust_type_map` / `python_type_map` / `go_type_map` in emit.dag.
Those were untyped `Map<String, String>` with no Copy semantics, no default
expressions, and silent pass-through on miss.

**Algebra grounding:**

| .dag type | Algebra | Structure |
|-----------|---------|-----------|
| `Int` | `OrderedRing<Word64>` | Arithmetic from Ring, comparison from Order |
| `Float` | `ApproximateField<Word64>` | Ring + division, approximate (IEEE 754) |
| `Bool` | `BooleanAlgebra<{True,False}>` | and/or/not from Lattice |
| `String` | `FreeMonoid<Char>` | concat/length from Monoid |
| `Unit` | Terminal object | Single inhabitant, zero fields |

---

### Category 2: Parameterized Container Coercion

**What:** .dag algebraic structure → target language parameterized type

**Mechanism:** Algebra inhabitant table (level 2 resolution)

**.dag source:**
```
fn lengths(items: List<String>) -> List<Int> {
  items |> map(s => s.length)
}
```

**Resolution path:**
```
List<String>
  → List<T> = FreeMonoid<T>     (from std/types.dag)
  → FreeMonoid inhabitant: "Vec<{0}>"  (arity 1)
  → {0} = coerce(String) = "String"   (checkpoint)
  → Vec<String>
```

**Target outputs:**

| Language | List\<T> | Set\<A> | Map\<K,V> |
|----------|----------|---------|-----------|
| Rust | `Vec<T>` | `BTreeSet<A>` | `HashMap<K,V>` |
| Python | `list[T]` | `set[A]` | `dict[K,V]` |
| Go | `[]T` | `map[A]struct{}` | `map[K]V` |
| SPICE | N/A (error) | N/A (error) | N/A (error) |
| English | `list of T` | `set of A` | `mapping from K to V` |

**The Set/NonEmptySet bug this prevents:**

Set<A> inhabits BooleanAlgebra<A>, not FreeMonoid<A>. The old code
mapped both to `FreeMonoidCollectionProfile`, which gave sets list
operations (append, sort_by, fold) instead of set operations (union,
intersect, diff).

With algebra-keyed inhabitants:
- `FreeMonoid<T>` → `Vec<T>` (list operations: append, map, fold)
- `BooleanAlgebra<A>` → `BTreeSet<A>` (set operations: union, intersect, diff)

---

### Category 3: Optional Coercion

**What:** .dag nullable annotation → target language optional type

**Mechanism:** Structural (cardinality annotation, not algebraic)

| Language | T? representation | None literal |
|----------|-------------------|-------------|
| Rust | `Option<T>` | `None` |
| Python | `T \| None` | `None` |
| Go | `*T` | `nil` |

---

### Category 4: Callable Coercion

**What:** .dag function type → target language callable type

**Mechanism:** Base callable template from `CallableRepr`, context-dependent
wrapping derived from ValueContext at emit time.

**.dag source:**
```
type Monoid<T> {
  op: fn(T, T) -> T
  identity: T
}
```

**Base rendering:** `CallableRepr.template` gives the bare function type syntax.
For Rust: `fn(T, T) -> T`. For Python: `Callable[[T, T], T]`. For Go: `func(T, T) T`.

**Context-dependent wrapping (Rust only, derived from ValueContext):**
- Struct field + stored → `Rc<dyn Fn(T, T) -> T>` (shared closure)
- Function parameter → `impl Fn(T) -> U` (generic bound)
- Data declaration → `fn(T) -> U` (static, known at compile time)
- Return type → `Box<dyn Fn(T) -> U>` (owned closure)

These are NOT declared as template variants. They are emitter decisions
derived from `ValueContext.has_fn_fields`, capture analysis, and resolved
ownership facts — matching "emission is translation, not decision-making."

**The Callable → "String" bug this prevents:**

The old code had `"Callable": "String"` in `rust_type_map`. Algebra method
fields with type `Callable` produced "String", causing 9 E0425 errors.

---

### Category 5: Sharing/Ownership Coercion

**What:** .dag value semantics → target language memory model

**Mechanism:** Coercion EFFECT applied after type coercion, derived from
ValueContext — NOT from a declared `SharingStrategy` configuration surface.

**Sharing decisions are structural facts, not language config:**

| Fact | Source | Decision |
|------|--------|----------|
| `ValueContext.is_constant = true` | Resolved graph | No Rc (const data) |
| `ValueContext.has_fn_fields = true` | Resolved graph | Skip PartialEq/Debug derives |
| `is_copy = true` (from checkpoint/inhabitant) | Coercion data | No Rc (value semantics) |
| Target language has GC | LanguageSpec | No wrapper needed |
| Recursive type | Structural analysis | Box<T> for indirection |
| Otherwise (Rust, non-Copy, runtime) | Default | Rc<T> wrapping |

Previous design had `SharingStrategy` as a per-language config type with
`wrapper_template`, `exempt_predicate`, `size_threshold_bytes`, and
`reference_types: List<String>`. Those were new heuristic side tables —
exactly the kind of mini-language the invariants say not to maintain.
Deleted in favor of structural derivation from ValueContext.

**The lazy_static + Rc bug this prevents:**

Constants skip Rc wrapping because `ValueContext.is_constant = true`.
No policy string needed — the fact is already in the graph.

---

### Category 6: Refinement Coercion

**What:** .dag refined type → target language base type + validation

**Mechanism:** Chain through refinement to base type checkpoint

### Refinement Contract

One authoritative contract (not per-target emitter discretion):

1. **Compile-time proven refinements → erased.** If the .dag type checker
   can prove the refinement holds (e.g., a literal `5` satisfies
   `NonNegativeInt`), the refinement erases to the base type with no
   runtime cost. The proof is the authority.

2. **Unproven refinements → explicit boundary constructor.** If the .dag
   type checker cannot prove the refinement (e.g., user input flowing
   into `NonNegativeInt`), validation happens at a **named constructor
   boundary** — not at ad-hoc assignment sites, not as `debug_assert!`
   that disappears in release builds.

   The constructor is the single place where unvalidated base values
   enter the refined type. This matches INVARIANTS.md: "illegal states
   made unrepresentable at the right boundary" and "errors surfaced
   where the owning stage can see them."

**Target outputs:**

| Language | Proven refinement | Unproven refinement |
|----------|-------------------|---------------------|
| Rust | `i64` (erased) | `NonNegativeInt::new(x)?` (constructor) |
| Python | `int` (erased) | `NonNegativeInt(x)` (constructor with `assert`) |
| Go | `int64` (erased) | `NewNonNegativeInt(x)` (constructor returning error) |

---

### Category 7: Structural Coercion (Product / Coproduct)

**What:** .dag user-defined types → target language structs and enums

**Mechanism:** Recursive structural traversal. Each field/variant is
recursively coerced. Derive strategy data is per-language.

---

### Category 8: Algebraic Operation Coercion

**What:** .dag algebra method calls → target language expressions

**Mechanism:** Per-language operation renderings attached to algebra operation
identity from `std/algebra.dag`. NOT a separate free-form operation catalog.

When a .dag expression calls `items |> map(f)`:
1. Resolve `items: List<Int>` → `FreeMonoid<Int>`
2. `map` is `FreeMonoid.map` (algebra operation identity)
3. Look up the target language's rendering for `FreeMonoid.map`
4. Apply: `items.iter().map(f).collect::<Vec<_>>()`

**The parallel authority risk:**

If operation templates become a free-standing catalog separate from
`std/algebra.dag` operation declarations, they recreate the same
"parallel authority" problem the roadmap calls out in
`free_monoid_collection_templates` and `partial_function_templates`.

**The performance risk:**

Operation templates that bake in clone-heavy expressions
(`.cloned().collect()`, `m.clone(); m.insert(...)`) may violate the
performance invariant by hardcoding conservative copying instead of
exploiting source guarantees around purity, fan-out, and shared ownership.
Operation renderings should be parameterized by ownership context, not
hard-coded as worst-case copies.

---

## Fail-Closed Contract

> **If a target language does not declare an inhabitant for an algebra,
> compiling any .dag type that requires that algebra for that target
> MUST produce a compile error.**

### How to test it

For each algebra A in std/algebra.dag:
- If this language file declares an inhabitant for A: generate algebra
  law tests (positive tests, from explicit law metadata in the algebra)
- If this language file does NOT declare an inhabitant for A: generate
  a coercion refusal test (negative test)

The refusal test compiles a .dag source containing `type T = A<Int>` and
asserts the compiler produces a diagnostic matching:

```
ERROR: No <LANGUAGE> inhabitant for algebra <ALGEBRA>.
       Add an inhabitant declaration to extdeps/languages/<LANGUAGE>/types.dag.
```

### Cross-language completeness matrix

| Algebra | Rust | Python | Go | SPICE | English |
|---------|------|--------|-----|-------|---------|
| OrderedRing | `i64` | `int` | `int64` | `real` | `integer` |
| ApproximateField | `f64` | `float` | `float64` | — (ERROR) | `decimal` |
| BooleanAlgebra (scalar) | `bool` | `bool` | `bool` | `wire` | `truth value` |
| BooleanAlgebra (collection) | `BTreeSet<A>` | `set[A]` | `map[A]struct{}` | — (ERROR) | `set of A` |
| FreeMonoid (scalar) | `String` | `str` | `string` | — (ERROR) | `text` |
| FreeMonoid (collection) | `Vec<T>` | `list[T]` | `[]T` | — (ERROR) | `list of T` |
| PartialFunction | `HashMap<K,V>` | `dict[K,V]` | `map[K]V` | — (ERROR) | `mapping` |

---

## Test Generation Strategy

### Positive tests: from explicit law metadata

```
for each InhabitantDecl D in language L:
  A = lookup_algebra(D.algebra) in std/algebra.dag

  for each law declared in A (from explicit law metadata):
    emit law test using D.identity_expr and D.template
    e.g., Monoid associativity → assert_eq!((a ⊕ b) ⊕ c, a ⊕ (b ⊕ c))

  emit round-trip test (serialize → deserialize = identity)
  emit boundary tests from D.identity_expr and type bounds
```

Positive law tests are generated ONLY from explicit law metadata in
`std/algebra.dag`. Generating associativity/identity/etc. from field shape
alone is a heuristic — the algebra must explicitly declare what laws it
promises. Hand-written tests are reserved for exceptional cases.

### Negative tests: refusal (auto-generated for every missing inhabitant)

```
for each algebra A in std/algebra.dag:
  if language L has NO InhabitantDecl for A:
    emit test: compile "type T = A<Int>" for target L
    assert compiler output contains error diagnostic
```

Refusal tests are exhaustive and fully automatic.

---

## TypeRendering as Migration Boundary

TypeRendering (E0c) is the normalized boundary between resolution and emit.
Coercion data **feeds** TypeRendering — it does not replace it.

**Migration path:**

1. `build_type_rendering` reads from `InhabitantDecl` and `TypeCheckpoint`
   data instead of hardcoded container/primitive maps.
2. TypeRendering's named edges (`element`, `key`, `value`, `params`,
   `return_type`, `inner`, `shared`, `boxed`) ARE the algebraic
   relationships, resolved through inhabitant declarations.
3. The emitter consumes `TypeRendering × LanguageSpec` — pure translation.
4. Current heuristic type rendering (node shape guessing) dissolves
   incrementally as `build_type_rendering` gains inhabitant-backed
   resolution.

This is incremental. No big-bang rewrite.

**What dissolves into TypeRendering + coercion data:**

| Current | Feeds into | Mechanism |
|---------|-----------|-----------|
| `rust_type_map` | `TypeCheckpoint` → `build_type_rendering` | Structured checkpoint with Copy/default |
| `rust_container_templates` | `InhabitantDecl` → `build_type_rendering` | Keyed by algebra, not container name |
| `build_rc_types` | ValueContext × `is_copy` | Structural derivation, not name-based |
| `emit_node_type_rc` | `TypeRendering` match | Pure translation from resolved descriptor |
| `emit_primitive_type` | `TypeCheckpoint` lookup | Fail-closed, no pass-through |
| `emit_container` | `InhabitantDecl` template application | `apply_template(D.template, inner_types)` |

---

## New Language Burden

Adding a new target language requires, for the **type coercion slice:**

1. Create `dsl/extdeps/languages/<name>/types.dag`
2. Import `TypeCheckpoint`, `InhabitantDecl`, `CallableRepr` from `std/coercion.dag`
3. Declare checkpoint entries for primitives the language knows
4. Declare inhabitant entries for each algebra the language supports
5. Declare a `CallableRepr` for function type syntax
6. Declare optional rendering templates

The compiler immediately generates refusal tests for every undeclared algebra.

**What this does NOT cover:** The full language backend also requires
`LanguageSpec` declarations: statement terminators, declaration keywords,
assignment syntax, lambda syntax, null-coalescing, string interpolation,
tuple/container syntax, indentation width, per-`ValueContext` templates,
and lint/formatting rules. Type coercion is one slice of the backend,
not the whole thing.

---

## Schema Architecture

```
std/coercion.dag            ← shared types (single authority)
  TypeCheckpoint
  InhabitantDecl
  CallableRepr

extdeps/languages/rust/types.dag    ← Rust data instances
  import std.coercion { TypeCheckpoint, InhabitantDecl, CallableRepr }
  data rust_type_checkpoints: List<TypeCheckpoint> = [...]
  data rust_algebra_inhabitants: List<InhabitantDecl> = [...]
  data rust_callable: CallableRepr = ...

extdeps/languages/python/types.dag  ← Python data instances (same schema)
extdeps/languages/go/types.dag      ← Go data instances (same schema)
```

No per-language type families. No schema drift. One authority for the
coercion vocabulary; per-language files are pure data.
