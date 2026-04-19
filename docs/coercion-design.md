> Part of: [THESIS.md](../THESIS.md) > [ROADMAP.md](../ROADMAP.md) > **Tier 1: Structural** (coercion completeness)

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

## Resolved Questions (cont.)

**RQ-9 (was OQ-1): Algebra laws in std/algebra.dag.**
Resolved: laws are `.dag` functions, not metadata annotations. The `.dag`
language IS the meta-language for modeling facts — adding a string-based
annotation layer (`assertion: "op(op(a,b),c) == ..."`) would introduce a
meta-meta-language, another dimension of intersubjectivity. Instead, laws
are modeled as computable predicates using the language's own constructs:

```
fn is_associative<T>(op: fn(T, T) -> T, a: T, b: T, c: T) -> Bool {
  op(op(a, b), c) == op(a, op(b, c))
}

fn has_left_identity<T>(op: fn(T, T) -> T, identity: T, a: T) -> Bool {
  op(identity, a) == a
}

fn has_right_identity<T>(op: fn(T, T) -> T, identity: T, a: T) -> Bool {
  op(a, identity) == a
}
```

These are structural declarations. The compiler can type-check them, compile
them to target-language test assertions, and apply them to any inhabitant by
substituting concrete operations. No `AlgebraLaw` metadata type. No string
assertions. The `.dag` language models the mathematical facts directly.

Test generation: for each `InhabitantDecl` D, look up the law predicates for
D's algebra, compile each to the target language with the inhabitant's concrete
types/operations substituted, and generate property-based tests. An
`InhabitantDecl` without corresponding law verification is an unproven theorem.

**RQ-10 (was OQ-2): TypeRendering variant coverage.**
Resolved: coverage follows from the grounding fact. TypeRendering variants
backed by algebraic facts (FreeMonoid, PartialFunction, BooleanAlgebra) need
explicit `InhabitantDecl` entries — the algebraic fact is what validates the
coercion. Variants that are structural (Product, Coproduct, Tuple) are derived
from the type definition itself — the structural fact IS the definition.
Callables and optionals are per-language rendering data. No gap exists:

| TypeRendering variant | Grounding fact | Coercion source |
|---|---|---|
| `PrimitiveRendering` | TypeCheckpoint declaration | Checkpoint lookup |
| `ContainerRendering` | Algebra axioms (FreeMonoid) | InhabitantDecl |
| `MapRendering` | Algebra axioms (PartialFunction) | InhabitantDecl |
| `SetRendering` | Algebra axioms (BooleanAlgebra) | InhabitantDecl |
| `ProductRendering` | Type definition (struct) | Structural recursion |
| `CoproductRendering` | Type definition (enum) | Structural recursion |
| `CallableRendering` | CallableRepr data | Per-language rendering |
| `OptionalRendering` | Cardinality annotation | Per-language template |
| `TupleRendering` | Type definition (anonymous product) | Structural recursion |

---

## Chief Invariant: Faithful Fact Modeling

See [`MODELING.md`](../MODELING.md) for the full principle. The core
rule: every construct must be grounded in an identifiable external fact.
No annotations, no meta-language on top of `.dag`, no ungrounded
abstractions.

Applied to coercion specifically:

- **Algebras are their axioms.** `Monoid` is the axiom set {associativity,
  identity}, not a name. `std/algebra.dag` formalizes these agreements.

- **Inhabitant declarations are theorem statements.** `Vec<T>` inhabiting
  `FreeMonoid<T>` is a claim that `Vec<T>` satisfies the axioms. Tests
  verify the theorems.

- **Fail-closed is epistemologically necessary.** No declared inhabitant
  means no factual basis for coercion. Silent fallback would be emitting
  code without evidence.

- **Cross-language correctness is grounded in shared algebra.** Rust's
  `Vec<i64>` and Python's `list[int]` serialize compatibly because both
  inhabit `FreeMonoid<OrderedRing>`.

- **String identity is a faithfulness problem.** The identity of
  `OrderedRing` is its axiom set, not the string. M4/Lane 1 replaces
  string keys with declaration edges for this reason.

---

## Core Thesis

Type rendering is coercion grounded in algebraic facts. The compiler walks
a `.dag` type composition tree and sidecasts to the target language's type
at the first level the language declares a fact-backed inhabitant for. Each
inhabitant declaration is a theorem: "this target type satisfies these
algebra axioms." The compiler sidecasts mechanically. Tests verify the
theorems. Fail-loud when no fact exists.

**Scope of this design:** This covers the type-coercion slice of language
backends. The full backend still requires `LanguageSpec` (statement terminators,
declaration keywords, assignment syntax, lambda syntax, null-coalescing,
string interpolation, container syntax, indentation, lint/formatting rules).
Type coercion is data-driven; the full backend is broader.

**Transitional state:** This document describes the target architecture.
The compiler is still partially transitional on some identity surfaces:
checkpoint and inhabitant keys are strings (`dag_name`, `algebra`), not
declaration edges. The v3 `Node.name` deletion is already landed; the
remaining transition is Lane 1's string-keyed coercion identity. The
coercion algorithm described here is designed for the post-transition
world where identity flows from declarations, but the initial
implementation (M5-early, via E0c `TypeRendering`) will keep the
transitional string keys until Lane 1 completes. The algorithm shape is
the same; only the key resolution mechanism changes.

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
| `render_type` | `TypeRendering` match | Pure translation from resolved descriptor |
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

---

## Appendix A: End-to-End Coercion Walks

This appendix works through complete coercion examples — from `.dag` source
through the type composition tree, through the resolution algorithm, to
target language output. Each example shows the DAG node structure, the
resolution walk at every node, and the final emitted type in every target.

---

### A.1: Sidecast — Same Algebra, Different Representation

A sidecast moves between inhabitants of the same algebra. `List<Int>` is
`FreeMonoid<Int>` in `.dag`. Each target language has a different inhabitant
for `FreeMonoid`, so the coercion is a sidecast from the algebraic identity
to the target-specific representation.

**.dag source:**
```
fn double_all(items: List<Int>) -> List<Int> {
  items |> map(x => x * 2)
}
```

**Type composition DAG for `List<Int>`:**
```
              List<Int>
              │
              ├─ algebra: FreeMonoid<T>   (from std/types.dag: List<T> = FreeMonoid<T>)
              │
              └─ type param [0]: Int
                                  │
                                  └─ algebra: OrderedRing<Word64>
                                     checkpoint: Rust→"i64", Python→"int", Go→"int64"
```

**Resolution walk (Rust):**
```
resolve(List<Int>, Rust):
  1. CHECKPOINT: "List" in rust_type_checkpoints? NO
  2. ALGEBRA: List inhabits FreeMonoid (from std/types.dag)
     FreeMonoid in rust_algebra_inhabitants? YES → template "Vec<{0}>", arity 1
     Recurse into type param [0]:
       resolve(Int, Rust):
         1. CHECKPOINT: "Int" in rust_type_checkpoints? YES → "i64"
         DONE → "i64"
     Apply template: "Vec<{0}>" with {0}="i64" → "Vec<i64>"
  DONE → "Vec<i64>"
```

**Resolution walk (SPICE):**
```
resolve(List<Int>, SPICE):
  1. CHECKPOINT: "List" in spice_type_checkpoints? NO
  2. ALGEBRA: List inhabits FreeMonoid
     FreeMonoid in spice_algebra_inhabitants? NO
  5. FAIL → COMPILE ERROR:
     "No SPICE inhabitant for algebra FreeMonoid.
      Add an inhabitant to extdeps/languages/spice/types.dag."
```

**All targets:**

| Target | `List<Int>` | `items |> map(f)` | Notes |
|--------|-------------|-------------------|-------|
| Rust | `Vec<i64>` | `items.iter().map(f).collect()` | FreeMonoid → Vec |
| Python | `list[int]` | `[f(x) for x in items]` | FreeMonoid → list |
| Go | `[]int64` | `for _, x := range items { ... }` | FreeMonoid → slice |
| SPICE | **ERROR** | — | No FreeMonoid inhabitant |
| English | `list of integers` | `apply f to each item` | FreeMonoid → "list of" |

The sidecast: `FreeMonoid<Int>` is the single algebraic identity. `Vec<i64>`,
`list[int]`, `[]int64` are all different inhabitants of the same algebra.
Moving between them is a sidecast — the structure is preserved, only the
representation changes.

---

### A.2: Downcast — Refinement Narrowing

A downcast narrows a type through refinement. The composition tree has
refinement nodes that add constraints.

**.dag source:**
```
type NonNegativeInt = Int where range(min: 0)
type Port = NonNegativeInt where range(min: 1, max: 65535)

fn connect(host: String, port: Port) -> Unit { ... }
```

**Type composition DAG for `Port`:**
```
              Port
              │
              ├─ refinement of: NonNegativeInt
              │                 │
              │                 ├─ refinement of: Int
              │                 │                 │
              │                 │                 └─ algebra: OrderedRing<Word64>
              │                 │                    checkpoint: Rust→"i64"
              │                 │
              │                 └─ predicate: range(min: 0)
              │
              └─ predicate: range(min: 1, max: 65535)
```

**Resolution walk (Rust):**
```
resolve(Port, Rust):
  1. CHECKPOINT: "Port" in rust_type_checkpoints? NO
  2. ALGEBRA: Port has no direct algebra
  4. REFINEMENT: Port refines NonNegativeInt
     resolve(NonNegativeInt, Rust):
       1. CHECKPOINT: "NonNegativeInt" in rust_type_checkpoints? NO
       2. ALGEBRA: NonNegativeInt has no direct algebra
       4. REFINEMENT: NonNegativeInt refines Int
          resolve(Int, Rust):
            1. CHECKPOINT: "Int" in rust_type_checkpoints? YES → "i64"
            DONE → "i64"
       DONE → "i64" (with validation: x >= 0)
     DONE → "i64" (with validation: 1 <= x <= 65535)
  Refinement Contract:
    Compile-time provable (literal 443)? → erased, bare "i64"
    Runtime input (user-provided)? → Port::new(x) → Result<Port, ValidationError>
```

**All targets:**

| Target | `Port` (proven) | `Port` (unproven) | Validation |
|--------|-----------------|-------------------|------------|
| Rust | `i64` | `Port::new(x)?` returns `Result<Port, _>` | Boundary constructor |
| Python | `int` | `Port(x)` raises `ValueError` | `__init__` guard |
| Go | `int64` | `NewPort(x)` returns `(Port, error)` | Constructor |
| SPICE | `real` | `real` (constraints in `.param` bounds) | Simulator limits |
| English | `port number` | `port number (1–65535)` | Prose constraint |

The downcast: `Int` → `NonNegativeInt` → `Port`. Each step narrows the
value set. The target type is always the base type's coercion (`i64`).
The refinement predicates become validation obligations at named boundaries.

---

### A.3: Upcast — Widening to Base Type

An upcast widens a refined type to its base. Always safe — no validation needed.

**.dag source:**
```
fn port_to_int(p: Port) -> Int { p }
fn stringify(n: Int) -> String { int_to_string(n) }

// Upcast chain: Port → NonNegativeInt → Int (always valid)
fn example(p: Port) -> String {
  stringify(p)  // Port <: Int, upcast is implicit
}
```

**Resolution:** `Port` and `Int` both coerce to `i64` in Rust. The upcast
is invisible at the target level — both types have the same representation.
The .dag type checker proves `Port <: Int` (every port is an integer), so
no runtime check is needed.

| Target | `port_to_int(p: Port) -> Int` | Notes |
|--------|-------------------------------|-------|
| Rust | `fn port_to_int(p: i64) -> i64 { p }` | Same type, identity |
| Python | `def port_to_int(p: int) -> int: return p` | Same type |
| Go | `func portToInt(p int64) int64 { return p }` | Same type |

---

### A.4: Deep Composition — `Map<String, List<Int?>>`

This is the stress test: three algebra layers plus an optional annotation.

**.dag source:**
```
type QueryResults = Map<String, List<Int?>>

fn empty_results() -> QueryResults { {} }
fn lookup(results: QueryResults, key: String) -> List<Int?> {
  match map_get(results, key) {
    Some { value: vs } => vs
    None => []
  }
}
```

**Type composition DAG:**
```
Map<String, List<Int?>>
│
├─ algebra: PartialFunction<K, V>     (from std/types.dag: Map<K,V> = PartialFunction<K,V>)
│
├─ type param [0] (K): String
│                       │
│                       └─ checkpoint: Rust→"String", Python→"str", Go→"string"
│
└─ type param [1] (V): List<Int?>
                        │
                        ├─ algebra: FreeMonoid<T>
                        │
                        └─ type param [0]: Int?
                                           │
                                           ├─ cardinality: Optional
                                           │
                                           └─ base: Int
                                                    │
                                                    └─ checkpoint: Rust→"i64", Python→"int", Go→"int64"
```

**Resolution walk (Rust) — full trace:**
```
resolve(Map<String, List<Int?>>, Rust):
  1. CHECKPOINT: "Map" in rust_type_checkpoints? NO
  2. ALGEBRA: Map inhabits PartialFunction (from std/types.dag)
     PartialFunction in rust_algebra_inhabitants? YES
       → template "HashMap<{0}, {1}>", arity 2

     Recurse into [0] (K = String):
       resolve(String, Rust):
         1. CHECKPOINT: "String" in rust_type_checkpoints? YES → "String"
         DONE → "String"

     Recurse into [1] (V = List<Int?>):
       resolve(List<Int?>, Rust):
         1. CHECKPOINT: "List" in rust_type_checkpoints? NO
         2. ALGEBRA: List inhabits FreeMonoid
            FreeMonoid in rust_algebra_inhabitants? YES
              → template "Vec<{0}>", arity 1

            Recurse into [0] (T = Int?):
              resolve(Int?, Rust):
                OPTIONAL: cardinality = Optional, base = Int
                  resolve(Int, Rust):
                    1. CHECKPOINT: "Int" → "i64"
                    DONE → "i64"
                Apply optional: "Option<{0}>" with {0}="i64" → "Option<i64>"
                DONE → "Option<i64>"

            Apply template: "Vec<{0}>" with {0}="Option<i64>"
            DONE → "Vec<Option<i64>>"

     Apply template: "HashMap<{0}, {1}>" with {0}="String", {1}="Vec<Option<i64>>"
     DONE → "HashMap<String, Vec<Option<i64>>>"
```

**Resolution walk (Python):**
```
resolve(Map<String, List<Int?>>, Python):
  Map → PartialFunction → "dict[{0}, {1}]"
    [0]: String → checkpoint → "str"
    [1]: List<Int?> → FreeMonoid → "list[{0}]"
           [0]: Int? → Optional → "int | None"   (PEP 604)
                → "list[int | None]"
    → "dict[str, list[int | None]]"
```

**Resolution walk (Go):**
```
resolve(Map<String, List<Int?>>, Go):
  Map → PartialFunction → "map[{0}]{1}"
    [0]: String → checkpoint → "string"
    [1]: List<Int?> → FreeMonoid → "[]{0}"
           [0]: Int? → Optional → "*int64"  (pointer for nil = absent)
                → "[]*int64"
    → "map[string][]*int64"
```

**Resolution walk (SPICE):**
```
resolve(Map<String, List<Int?>>, SPICE):
  Map → PartialFunction → NO INHABITANT
  FAIL → COMPILE ERROR:
    "No SPICE inhabitant for algebra PartialFunction.
     SPICE has no keyed data structures — partial functions are
     not representable in circuit description languages."
```

**Resolution walk (English):**
```
resolve(Map<String, List<Int?>>, English):
  Map → PartialFunction → "mapping from {0} to {1}"
    [0]: String → checkpoint → "text"
    [1]: List<Int?> → FreeMonoid → "list of {0}"
           [0]: Int? → Optional → "optional integer"
                → "list of optional integers"
    → "mapping from text to list of optional integers"
```

**All targets:**

| Target | `Map<String, List<Int?>>` | `empty_results()` |
|--------|---------------------------|-------------------|
| Rust | `HashMap<String, Vec<Option<i64>>>` | `HashMap::new()` |
| Python | `dict[str, list[int \| None]]` | `{}` |
| Go | `map[string][]*int64` | `nil` |
| SPICE | **COMPILE ERROR** | — |
| English | `mapping from text to list of optional integers` | `empty mapping` |

**Optionality interpretation:** `.dag` lists are dense (`FreeMonoid`
preserves ordering and has no holes). A `List<Int?>` is NOT a sparse
sequence with absent positions — every index `0..n-1` is occupied. The
`?` means each element's VALUE is nullable: denotationally `List<Int | Unit>`
where `Unit` represents null/absent. This is consistent with
`constructors.dag` ("Cardinality is orthogonal to constructors... annotation
on the binding, not a type wrapper") and `algebraic-type-spec.md`
("`T?` = cardinality Optional on the binding site, `Optional<T>` does not
exist as a type constructor").

Concretely: the list has exactly `n` elements. Each element binding has
optional cardinality, meaning the value at that position is either an `Int`
or null. The coercion engine resolves this structurally: coerce the base
type (`Int` → `i64`), wrap in the target's nullable template
(`Option<i64>`, `int | None`, `*int64`), then wrap in the container
template (`Vec<Option<i64>>`). The resulting target type is a dense
collection of nullable values, not a partial-index map or sparse sequence.

---

### A.5: Deep Composition — `List<Map<String, Set<String>>>`

Three different algebra inhabitants nested: FreeMonoid wrapping PartialFunction
wrapping BooleanAlgebra.

**.dag source:**
```
type TagIndex = List<Map<String, Set<String>>>

fn merge_indices(a: TagIndex, b: TagIndex) -> TagIndex {
  concat(a, b)
}
```

**Type composition DAG:**
```
List<Map<String, Set<String>>>
│
├─ FreeMonoid<T>
│
└─ [0]: Map<String, Set<String>>
         │
         ├─ PartialFunction<K, V>
         │
         ├─ [0] K: String → checkpoint
         │
         └─ [1] V: Set<String>
                    │
                    ├─ BooleanAlgebra<A>
                    │
                    └─ [0] A: String → checkpoint
```

**All targets (resolved):**

| Target | `List<Map<String, Set<String>>>` |
|--------|----------------------------------|
| Rust | `Vec<HashMap<String, BTreeSet<String>>>` |
| Python | `list[dict[str, set[str]]]` |
| Go | `[]map[string]map[string]struct{}` |
| SPICE | **ERROR** (FreeMonoid) |
| English | `list of mappings from text to sets of text` |

**Sharing effect (Rust):**

The fully coerced type is `Vec<HashMap<String, BTreeSet<String>>>`. None of
these are Copy. In a shared context (struct field, multiple references):

```rust
pub struct SearchEngine {
    pub indices: Rc<Vec<HashMap<String, BTreeSet<String>>>>,
    //           ^^^ sharing applied at outermost level only
    //               inner types are owned by the outer container
}
```

ValueContext determines the wrapping: `is_constant = false`, type is `!Copy`
→ `Rc<T>`. The `Rc` wraps the outermost type; inner containers are owned
values within the outer `Vec`.

---

### A.6: Struct with Mixed Complexity — Full Service Type

A realistic service definition combining all coercion categories. Presented
in two phases: first pure type coercion (what each field becomes in the
target), then contextual rendering effects (sharing, derives) applied
downstream from the facts.

**.dag source:**
```
type Email = String where pattern("^[^@]+@[^@]+\\.[^@]+$")
type Timestamp = Int where range(min: 0)

type UserProfile {
  id: Int
  name: String
  email: Email
  tags: Set<String>
  metadata: Map<String, Json>
  last_login: Timestamp?
  friends: List<Int>
  on_update: fn(UserProfile) -> Unit
}
```

#### Phase 1: Pure field-type coercion

Each field resolves independently through the coercion algorithm. No
sharing, ownership, or derive decisions — just algebraic/structural facts.

**Per-field resolution (Rust):**

| Field | .dag type | Resolution path | Coerced type |
|-------|-----------|-----------------|-----------|
| `id` | `Int` | checkpoint | `i64` |
| `name` | `String` | checkpoint | `String` |
| `email` | `Email` | refinement → String → checkpoint | `String` |
| `tags` | `Set<String>` | BooleanAlgebra → BTreeSet, String → checkpoint | `BTreeSet<String>` |
| `metadata` | `Map<String, Json>` | PartialFunction → HashMap, String/Json → checkpoint | `HashMap<String, serde_json::Value>` |
| `last_login` | `Timestamp?` | optional + refinement → Int → checkpoint | `Option<i64>` |
| `friends` | `List<Int>` | FreeMonoid → Vec, Int → checkpoint | `Vec<i64>` |
| `on_update` | `fn(UserProfile) -> Unit` | callable base template | `fn(UserProfile) -> ()` |

Phase 1 produces the BASE callable type from `CallableRepr.template`.
For `on_update`, this is the bare function type — context-dependent
wrapping (struct field → `Rc<dyn Fn>`, parameter → `impl Fn`, etc.)
happens in Phase 2 based on `ValueContext`.

**Cross-target coercion (pure types, no sharing):**

| Field | Rust | Python | Go |
|-------|------|--------|----|
| `id` | `i64` | `int` | `int64` |
| `name` | `String` | `str` | `string` |
| `email` | `String` | `str` | `string` |
| `tags` | `BTreeSet<String>` | `set[str]` | `map[string]struct{}` |
| `metadata` | `HashMap<String, serde_json::Value>` | `dict[str, dict]` | `map[string]interface{}` |
| `last_login` | `Option<i64>` | `int \| None` | `*int64` |
| `friends` | `Vec<i64>` | `list[int]` | `[]int64` |
| `on_update` | `fn(UserProfile) -> ()` | `Callable[[UserProfile], None]` | `func(UserProfile)` |

This is the coercion engine's output. Every cell is determined by the
type's algebraic or structural facts. No language-specific heuristics.
Note: `on_update` shows the base callable template; Rust context wrapping
is applied in Phase 2.

#### Phase 2: ValueContext rendering effects (downstream from coercion)

After coercion resolves the target types, `ValueContext` applies
language-specific rendering policy. This is "emission is translation"
— the emitter reads precomputed facts, it doesn't make decisions.

**Rust — ValueContext analysis:**
```
UserProfile:
  has_fn_fields = true  (on_update is a callable field)
    → skip PartialEq, Debug derives (dyn Fn doesn't impl these)
    → derive only: Clone, Serialize, Deserialize

  callable context:
    on_update: fn(UserProfile) -> Unit
      context = struct field (stored callable)
      → Rc<dyn Fn(UserProfile)>  (shared closure, per Category 4)
      (If this were a data declaration → fn(UserProfile) -> (),
       a noncapturing static function pointer. The .dag type is
       the same; the rendering depends on ValueContext.)

  per-field is_copy:
    id: i64              → Copy (checkpoint declares is_copy: true)
    name: String         → !Copy → Rc<String>
    email: String        → !Copy → Rc<String>
    tags: BTreeSet<...>  → !Copy → Rc<BTreeSet<String>>
    metadata: HashMap<...> → !Copy → Rc<HashMap<...>>
    last_login: Option<i64> → Copy (Option<Copy> = Copy)
    friends: Vec<i64>    → !Copy → Rc<Vec<i64>>
    on_update: Rc<dyn Fn(...)> → !Copy → already Rc
```

**Rust output (coercion + ValueContext combined):**

```rust
use std::collections::{BTreeSet, HashMap};

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct UserProfile {
    pub id: i64,
    pub name: Rc<String>,
    pub email: Rc<String>,
    pub tags: Rc<BTreeSet<String>>,
    pub metadata: Rc<HashMap<String, serde_json::Value>>,
    pub last_login: Option<i64>,
    pub friends: Rc<Vec<i64>>,
    pub on_update: Rc<dyn Fn(UserProfile)>,
}
```

**Python — no ValueContext effects.** Python is GC; no sharing wrapping,
no Copy/!Copy distinction. The Phase 1 coerced types are the final output:

```python
from dataclasses import dataclass
from typing import Callable

@dataclass
class UserProfile:
    id: int
    name: str
    email: str
    tags: set[str]
    metadata: dict[str, dict]
    last_login: int | None
    friends: list[int]
    on_update: Callable[[UserProfile], None]
```

**Go — no ValueContext effects.** Go is GC; Phase 1 types are final:

```go
type UserProfile struct {
    ID        int64
    Name      string
    Email     string
    Tags      map[string]struct{}
    Metadata  map[string]interface{}
    LastLogin *int64
    Friends   []int64
    OnUpdate  func(UserProfile)
}
```

**English output:**

```
User Profile:
  id: integer
  name: text
  email: email address (text matching email pattern)
  tags: set of text
  metadata: mapping from text to structured data
  last login: optional timestamp (non-negative integer)
  friends: list of integers
  on update: function taking a user profile, returning nothing
```

**SPICE output (partial — fails closed):**

```
ERROR: No SPICE inhabitant for algebra FreeMonoid.
  Field 'friends' has type List<Int> which requires FreeMonoid.
  SPICE cannot represent ordered sequences.

ERROR: No SPICE inhabitant for algebra PartialFunction.
  Field 'metadata' has type Map<String, Json> which requires PartialFunction.

ERROR: No SPICE inhabitant for algebra BooleanAlgebra (collection).
  Field 'tags' has type Set<String> which requires BooleanAlgebra.
```

The two-phase structure matches the invariant: coercion resolves target
types from algebraic/structural facts (Phase 1). Rendering policy — sharing,
derives, naming conventions — is applied downstream from `ValueContext`
(Phase 2). The emitter never makes type decisions; it reads them.

---

### A.7: Sidecast Between Language Targets

The same `.dag` value can be serialized and deserialized across targets
because the algebraic identity is preserved. This is the interop story.

**.dag source:**
```
data config: Map<String, List<Int>> = {
  "primes": [2, 3, 5, 7, 11],
  "fibonacci": [1, 1, 2, 3, 5, 8]
}
```

**Resolution (same for all targets):**
```
Map<String, List<Int>>
  → PartialFunction<String, FreeMonoid<Int>>
  → each target applies its own inhabitants
```

**Target representations of the same value:**

| Target | Type | Value |
|--------|------|-------|
| Rust | `HashMap<String, Vec<i64>>` | `{("primes".into(), vec![2,3,5,7,11]), ...}` |
| Python | `dict[str, list[int]]` | `{"primes": [2,3,5,7,11], "fibonacci": [1,1,2,3,5,8]}` |
| Go | `map[string][]int64` | `map[string][]int64{"primes": {2,3,5,7,11}, ...}` |
| English | `mapping from text to lists of integers` | `"primes" maps to the list 2, 3, 5, 7, 11; ...` |

The JSON serialization is identical across all targets:
```json
{"primes": [2, 3, 5, 7, 11], "fibonacci": [1, 1, 2, 3, 5, 8]}
```

This is the sidecast property: the algebraic structure (PartialFunction
containing FreeMonoid containing OrderedRing) is preserved through
serialization. Any target that has inhabitants for all three algebras
can round-trip the data.

**Sharing effect:** This is `data` (constant). `ValueContext.is_constant = true`.
No `Rc` wrapping in Rust. The emitter generates a constructor function:

```rust
pub fn config() -> HashMap<String, Vec<i64>> {
    let mut m = HashMap::new();
    m.insert("primes".into(), vec![2, 3, 5, 7, 11]);
    m.insert("fibonacci".into(), vec![1, 1, 2, 3, 5, 8]);
    m
}
```

Not `lazy_static! { static ref CONFIG: Rc<HashMap<...>> = ... }` — which
would cause E0277 Send/Sync.

---

### A.8: Programmatic Coercion — User-Defined Algebra Types

When a programmer defines a new type as inhabiting an algebra, coercion
applies automatically.

**.dag source:**
```
// User defines a type that inhabits an existing algebra
type Seq<T> = FreeMonoid<T>
type Lookup<K, V> = PartialFunction<K, V>
type Flags<A> = BooleanAlgebra<A>

// These compose exactly like List, Map, Set
type Registry = Lookup<String, Seq<Flags<Int>>>
```

**Resolution walk (Rust):**
```
resolve(Registry, Rust):
  Registry = Lookup<String, Seq<Flags<Int>>>
  1. CHECKPOINT: "Registry" not in checkpoints
  2. Lookup = PartialFunction<K,V>
     PartialFunction → "HashMap<{0}, {1}>"
       [0] K = String → checkpoint → "String"
       [1] V = Seq<Flags<Int>>
           Seq = FreeMonoid<T>
           FreeMonoid → "Vec<{0}>"
             [0] T = Flags<Int>
                 Flags = BooleanAlgebra<A>
                 BooleanAlgebra → "BTreeSet<{0}>"
                   [0] A = Int → checkpoint → "i64"
                 → "BTreeSet<i64>"
             → "Vec<BTreeSet<i64>>"
       → "HashMap<String, Vec<BTreeSet<i64>>>"
```

**All targets:**

| Target | `Lookup<String, Seq<Flags<Int>>>` |
|--------|-----------------------------------|
| Rust | `HashMap<String, Vec<BTreeSet<i64>>>` |
| Python | `dict[str, list[set[int]]]` |
| Go | `map[string][]map[int64]struct{}` |
| English | `mapping from text to list of sets of integers` |

The programmer never wrote a type checkpoint or inhabitant for `Seq`,
`Lookup`, or `Flags`. The coercion engine resolved them through their
algebra definitions. New algebra aliases compose freely — that's the
payoff of algebra-keyed inhabitants over name-keyed container templates.

---

### A.9: Unified Authority — Type, Identity, and Operation from One Algebra

The previous examples prove type coercion. This example proves that the
same algebra declaration is the single authority for three things: the
target type, the identity/empty literal, and operation rendering. All three
come from `FreeMonoid`'s `InhabitantDecl` and its operation identity in
`std/algebra.dag`.

**.dag source:**
```
fn build_sequence(seed: Int, items: List<Int>) -> List<Int> {
  let start = []                    // identity element (empty FreeMonoid)
  let base = append(start, seed)    // FreeMonoid.append operation
  concat(base, items)               // FreeMonoid.concat operation
}
```

**Three facts, one authority (Rust):**

```
Authority: InhabitantDecl { algebra: FreeMonoid, template: "Vec<{0}>", identity_expr: "Vec::new()" }

1. TYPE coercion:
   List<Int> → FreeMonoid<Int> → "Vec<{0}>" with Int→"i64"
   → Vec<i64>

2. IDENTITY literal:
   [] → FreeMonoid identity → identity_expr → Vec::new()

3. OPERATION rendering (from std/algebra.dag operation identity):
   append(xs, x) → FreeMonoid.append → { let mut v = xs; v.push(x); v }
   concat(xs, ys) → FreeMonoid.concat → { let mut v = xs; v.extend(ys); v }
```

**Rust output:**
```rust
fn build_sequence(seed: i64, items: Vec<i64>) -> Vec<i64> {
    let start: Vec<i64> = Vec::new();
    let mut base = start;
    base.push(seed);
    let mut result = base;
    result.extend(items);
    result
}
```

**Python — same three facts, different inhabitants:**
```
Authority: InhabitantDecl { algebra: FreeMonoid, template: "list[{0}]", identity_expr: "[]" }

1. TYPE: list[int]
2. IDENTITY: []
3. OPERATIONS: append → xs + [x], concat → xs + ys
```

```python
def build_sequence(seed: int, items: list[int]) -> list[int]:
    start: list[int] = []
    base = start + [seed]
    return base + items
```

**Go — same three facts, different inhabitants:**
```
Authority: InhabitantDecl { algebra: FreeMonoid, template: "[]{0}", identity_expr: "nil" }

1. TYPE: []int64
2. IDENTITY: nil (empty slice)
3. OPERATIONS: append → append(xs, x), concat → append(xs, ys...)
```

```go
func buildSequence(seed int64, items []int64) []int64 {
    start := []int64(nil)
    base := append(start, seed)
    return append(base, items...)
}
```

**English — same three facts:**
```
1. TYPE: list of integers
2. IDENTITY: empty list
3. OPERATIONS: append → "add seed to the list", concat → "combine base with items"
```

```
To build a sequence from a seed and a list of integers:
  start with an empty list,
  add the seed to the list,
  then combine it with the given items.
```

The key: type template and identity expression come from the `FreeMonoid`
`InhabitantDecl`. Operation rendering comes from `FreeMonoid`'s operation
identity in `std/algebra.dag` plus the per-language rendering attached to
that operation. These are not literally one record holding all three facts,
but they form a single algebraic authority chain — no parallel catalogs,
no free-standing operation tables. If any link in the chain is missing
(no inhabitant, no operation rendering), the compiler fails closed.

---

### A.10: Mixed Structural and Algebraic — `List<Point>`

The previous examples compose algebras inside algebras. This example composes
a structural product (no algebra) inside an algebra container, proving the
same recursive algorithm handles both without a second mechanism.

**.dag source:**
```
type Point { x: Int, y: Int }
type Shape = Circle { center: Point, radius: Float }
           | Rect { top_left: Point, bottom_right: Point }

type Canvas = Map<String, List<Shape>>
```

**Type composition DAG for `Canvas`:**
```
Map<String, List<Shape>>
│
├─ algebra: PartialFunction<K, V>
│
├─ [0] K: String → checkpoint
│
└─ [1] V: List<Shape>
         │
         ├─ algebra: FreeMonoid<T>
         │
         └─ [0] T: Shape
                    │
                    ├─ NO checkpoint, NO algebra → STRUCTURAL (coproduct)
                    │
                    ├─ variant Circle { center: Point, radius: Float }
                    │                    │              │
                    │                    │              └─ checkpoint
                    │                    │
                    │                    └─ Point: STRUCTURAL (product)
                    │                        ├─ x: Int → checkpoint
                    │                        └─ y: Int → checkpoint
                    │
                    └─ variant Rect { top_left: Point, bottom_right: Point }
                                      │                 │
                                      └─ Point ─────────┘  (same structural product)
```

**Resolution walk (Rust):**
```
resolve(Canvas, Rust):
  Canvas = Map<String, List<Shape>>
  Map → PartialFunction → "HashMap<{0}, {1}>"
    [0] String → checkpoint → "String"
    [1] List<Shape> → FreeMonoid → "Vec<{0}>"
          [0] Shape →
            NO checkpoint. NO algebra.
            → STRUCTURAL: coproduct → CoproductRendering
              variant Circle:
                center: Point → STRUCTURAL: product → ProductRendering
                  x: Int → checkpoint → "i64"
                  y: Int → checkpoint → "i64"
                → struct Point { x: i64, y: i64 }
                radius: Float → checkpoint → "f64"
              variant Rect:
                top_left: Point → (same as above)
                bottom_right: Point → (same as above)
              → enum Shape { Circle { center: Point, radius: f64 },
                             Rect { top_left: Point, bottom_right: Point } }
          → "Vec<Shape>"
    → "HashMap<String, Vec<Shape>>"
```

**All targets:**

| Target | `Point` | `Shape` | `Canvas` |
|--------|---------|---------|----------|
| Rust | `struct Point { x: i64, y: i64 }` | `enum Shape { Circle {...}, Rect {...} }` | `HashMap<String, Vec<Shape>>` |
| Python | `@dataclass Point: x: int; y: int` | `Shape = Circle \| Rect` | `dict[str, list[Shape]]` |
| Go | `type Point struct { X, Y int64 }` | `type Shape interface {...}` | `map[string][]Shape` |
| English | `point (x: integer, y: integer)` | `shape: circle or rectangle` | `mapping from text to list of shapes` |

The structural types (`Point`, `Shape`) fall through to `ProductRendering`
and `CoproductRendering` — no inhabitant declarations needed, no algebra
lookups. The algebra containers (`Map`, `List`) resolve through their
inhabitants as usual. One recursive algorithm, no mode switch between
algebraic and structural types.
