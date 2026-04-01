# Coercion Design: Algebra Inhabitants and Type Sidecast

**Status:** Design sketch (no compiler changes)
**Branch:** `design/coercion-inhabitants`
**Depends on:** M2 (working Rust codegen), TypeRendering (E0c), Lane 1 Tier 3
**Dissolves:** `rust_type_map`, `rust_container_templates`, `build_rc_types`,
`emit_node_type_rc`, TypeRendering (E0c)

---

## Open Questions

**OQ-1: Shared coercion vocabulary type location.**
Each language file currently defines its own `InhabitantDecl` with
language-specific field names (`rust_template`, `python_template`,
`go_template`). Should there be a single shared `InhabitantDecl` in
`std/coercion.dag` with a generic `target_template` field? A shared type
enforces schema consistency across languages; per-language types allow
language-specific metadata (e.g. Rust's `is_copy`, Python's `import_path`).
Candidate resolution: shared base type + per-language extension fields.

**OQ-2: Scalar vs collection FreeMonoid disambiguation.**
String is `FreeMonoid<Char>` and List is `FreeMonoid<T>`. The inhabitant
table has both arity-0 (String → `String`) and arity-1 (List → `Vec<{0}>`)
for FreeMonoid. How does the compiler distinguish which inhabitant to apply?
Arity alone works today, but breaks if someone writes `type Chars = FreeMonoid<Char>`
(arity 1, but conceptually scalar). Options: (a) arity is sufficient and
`Chars` correctly coerces to `Vec<char>`, (b) add a `scalar: Bool` flag
to InhabitantDecl, (c) resolve through the checkpoint table first (String
hits the checkpoint before reaching the algebra level).

**OQ-3: NonEmptyList / NonEmptySet representation.**
These are refinements of List/Set with a non-empty constraint. Currently
they have separate container template entries (`non_empty_list`, `non_empty_set`).
Should they be modeled as: (a) separate algebra inhabitants (current approach),
(b) refinements of their base container type (NonEmptyList = List where non_empty),
or (c) a `NonEmpty<C>` wrapper algebra? Option (b) is cleanest algebraically
but requires the compiler to handle refinement-of-container coercion.

**OQ-4: Callable context decision algorithm.**
`CallableRepr` has owned/shared/static templates, but the design doesn't
specify how the compiler chooses between them. Proposed rule:
- Struct field → `shared_template` (stored, needs Rc for sharing)
- Function parameter → `static_template` if monomorphic, `impl Fn` if generic
- Data declaration → `static_template` (known at compile time)
- Return type → `owned_template` (caller owns the closure)
Does this cover all cases? What about closures that capture mutable state?

**OQ-5: Operation coercion data model.**
Category 8 (algebraic operation coercion — e.g. `FreeMonoid.map` →
`.iter().map(f).collect()`) is described in the design doc but has no
corresponding data declarations in `types.dag`. Should operation templates
live alongside type inhabitants in `types.dag`, in a separate
`operations.dag`, or remain in `emit.dag`? The algebra definitions in
`std/algebra.dag` already list operations per algebra — the question is
where the per-language rendering templates for those operations live.

**OQ-6: Refinement validation emission strategy.**
Refinements coerce to their base type + validation, but the design doesn't
specify WHEN validation is emitted. Options:
- Constructor-time: newtype wrapper with validating `new()` → safest but verbose
- Assignment-time: `debug_assert!` at binding sites → lighter but only in debug
- Boundary-time: validate at module/function entry points → compromise
- Never (trust the type checker): .dag already proved the constraint → no runtime cost
The choice may vary per target language and per optimization level.

**OQ-7: TypeRendering (E0c) coexistence.**
TypeRendering is being implemented on `keen-lynx-513` as a stepping stone.
Should the coercion inhabitant declarations be designed to coexist with
TypeRendering during the transition? Specifically: can `build_type_rendering`
read from `InhabitantDecl` data instead of hardcoded container/primitive
maps, so TypeRendering dissolves incrementally into coercion rather than
requiring a big-bang replacement?

**OQ-8: Property-based test generation.**
The `CoercionTest` declarations use example-based assertions. Should the
compiler also generate property-based tests (proptest / quickcheck style)
from the algebra laws? E.g., for OrderedRing: `proptest!(|(a: i64, b: i64, c: i64)| assert_eq!((a.wrapping_add(b)).wrapping_add(c), a.wrapping_add(b.wrapping_add(c))))`
This would give stronger coverage but introduces a test framework dependency.

---

## Core Thesis

Type rendering is coercion. The compiler walks a .dag type composition tree
and sidecasts to the target language's type at the first level the language
declares an inhabitant for. Each language declares its algebra inhabitants
as .dag extdep data. The compiler sidecasts mechanically. Adding a new
language = declaring inhabitants + sharing strategy. No emission logic.
Fail-loud when no inhabitant exists.

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
   YES → coerce B (recursively). Add validation. DONE.

5. FAIL: No coercion path found.
   → COMPILE ERROR: "Language L has no representation for type T
     (algebra: A). Add an inhabitant to extdeps/languages/L/types.dag."
```

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

**Test obligations per primitive:**

For `Int → i64` (OrderedRing):
```rust
// ALGEBRA LAW: additive identity
assert_eq!(a + 0i64, a);
// ALGEBRA LAW: additive associativity
assert_eq!((a + b) + c, a + (b + c));
// ALGEBRA LAW: additive inverse
assert_eq!(a.wrapping_add(-a), 0); // wrapping to avoid overflow UB
// ALGEBRA LAW: multiplicative identity
assert_eq!(a * 1i64, a);
// ALGEBRA LAW: distributivity
assert_eq!(a * (b + c), a * b + a * c); // subject to overflow
// BOUNDARY: overflow
assert!(i64::MAX.checked_add(1).is_none());
// BOUNDARY: zero annihilation
assert_eq!(0i64 * any_value, 0);
```

For `Bool → bool` (BooleanAlgebra):
```rust
// ALGEBRA LAW: complement involution
assert_eq!(!(!a), a);
// ALGEBRA LAW: De Morgan
assert_eq!(!(a && b), !a || !b);
assert_eq!(!(a || b), !a && !b);
// ALGEBRA LAW: identity
assert_eq!(a && true, a);
assert_eq!(a || false, a);
```

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

The algebra identity is the authority. A set is NOT a list that happens
to have unique elements — it's a fundamentally different algebraic object
with different operations.

**Test obligations per container:**

For `FreeMonoid → Vec<T>`:
```rust
// ALGEBRA LAW: left identity (concat with empty)
assert_eq!([Vec::<i64>::new(), xs.clone()].concat(), xs);
// ALGEBRA LAW: right identity
assert_eq!([xs.clone(), vec![]].concat(), xs);
// ALGEBRA LAW: associativity
let ab_c = [[a.clone(), b.clone()].concat(), c.clone()].concat();
let a_bc = [a.clone(), [b.clone(), c.clone()].concat()].concat();
assert_eq!(ab_c, a_bc);
// ALGEBRA LAW: length homomorphism
assert_eq!([a.clone(), b.clone()].concat().len(), a.len() + b.len());
// FUNCTORIAL LAW: map identity
let mapped: Vec<i64> = xs.iter().map(|x| *x).collect();
assert_eq!(mapped, xs);
// FUNCTORIAL LAW: map composition
let fg: Vec<i64> = xs.iter().map(|x| g(f(*x))).collect();
let f_then_g: Vec<i64> = xs.iter().map(f).map(g).collect();
assert_eq!(fg, f_then_g);
```

For `BooleanAlgebra → BTreeSet<A>`:
```rust
// ALGEBRA LAW: join identity
let result: BTreeSet<_> = s.union(&BTreeSet::new()).cloned().collect();
assert_eq!(result, s);
// ALGEBRA LAW: meet idempotent
let result: BTreeSet<_> = s.intersection(&s).cloned().collect();
assert_eq!(result, s);
// ALGEBRA LAW: absorption
let meet: BTreeSet<_> = s.intersection(&t).cloned().collect();
let result: BTreeSet<_> = s.union(&meet).cloned().collect();
assert_eq!(result, s);
// ALGEBRA LAW: commutativity
let st: BTreeSet<_> = s.union(&t).cloned().collect();
let ts: BTreeSet<_> = t.union(&s).cloned().collect();
assert_eq!(st, ts);
```

For `PartialFunction → HashMap<K,V>`:
```rust
// ALGEBRA LAW: empty lookup
assert_eq!(HashMap::<String, i64>::new().get("key"), None);
// ALGEBRA LAW: insert-lookup roundtrip
let mut m = HashMap::new();
m.insert("key".to_string(), 42i64);
assert_eq!(m.get("key"), Some(&42));
// ALGEBRA LAW: insert-overwrite
m.insert("key".to_string(), 99i64);
assert_eq!(m.get("key"), Some(&99));
// ALGEBRA LAW: merge preserves keys
let mut a = HashMap::new(); a.insert("a".to_string(), 1i64);
let mut b = HashMap::new(); b.insert("b".to_string(), 2i64);
a.extend(b);
assert_eq!(a.len(), 2);
```

---

### Category 3: Optional Coercion

**What:** .dag nullable annotation → target language optional type

**Mechanism:** Structural (cardinality annotation, not algebraic)

**.dag source:**
```
type User { name: String, email: Email? }
```

**Resolution path:**
```
email: Email?
  → cardinality = Optional, base = Email
  → Email is refinement of String → "String" (checkpoint)
  → Optional template: "Option<{0}>" → Option<String>
```

**Target outputs:**

| Language | T? representation | None literal | Pattern match |
|----------|-------------------|-------------|---------------|
| Rust | `Option<T>` | `None` | `match x { Some(v) => ..., None => ... }` |
| Python | `T \| None` | `None` | `if x is not None: ...` |
| Go | `*T` | `nil` | `if x != nil { ... }` |
| English | `optional T` | `absent` | `if present: ...` |

**Test obligations:**
```rust
// ROUND-TRIP: Some wraps and unwraps
assert_eq!(Some(42i64).unwrap(), 42);
// ROUND-TRIP: None is None
assert!(None::<i64>.is_none());
// PATTERN: exhaustive match
match opt {
    Some(v) => { /* v is accessible */ },
    None => { /* handled */ },
}
// No third case exists — compiler enforces exhaustiveness.
```

---

### Category 4: Callable Coercion

**What:** .dag function type → target language callable type

**Mechanism:** Context-dependent (owned/shared/static)

**.dag source:**
```
type Monoid<T> {
  op: fn(T, T) -> T
  identity: T
}
```

**Resolution path:**
```
op: fn(T, T) -> T
  → CallableRepr
  → Context: struct field, stored → shared_template
  → "Rc<dyn Fn(T, T) -> T>"
  → ValueContext.has_fn_fields = true → skip PartialEq/Debug derives
```

**The Callable → "String" bug this prevents:**

The old code had `"Callable": "String"` in `rust_type_map`. When algebra
method fields had type `Callable` (a bare leaf node with no params), the
type_map returned "String". This caused 9 E0425 errors in std_algebra.rs
because the emitted Rust code referenced a type `Callable` that didn't exist.

With proper callable coercion:
- Function-typed fields get the correct Rust representation
- The representation varies by context (owned/shared/static)
- `has_fn_fields` is detectable → derives are adjusted

**Target outputs:**

| Language | fn(T) -> U (parameter) | fn(T) -> U (stored) |
|----------|----------------------|---------------------|
| Rust | `impl Fn(T) -> U` | `Rc<dyn Fn(T) -> U>` |
| Python | `Callable[[T], U]` | `Callable[[T], U]` |
| Go | `func(T) U` | `func(T) U` |

**Test obligations:**
```rust
// ROUND-TRIP: calling a coerced function produces expected result
let f: fn(i64, i64) -> i64 = |a, b| a + b;
assert_eq!(f(2, 3), 5);
// STORED: Rc-wrapped closure is callable
let f: Rc<dyn Fn(i64) -> i64> = Rc::new(|x| x * 2);
assert_eq!(f(21), 42);
// COMPOSITION: function composition through coercion
let f: fn(i64) -> i64 = |x| x + 1;
let g: fn(i64) -> i64 = |x| x * 2;
assert_eq!(g(f(3)), 8);
```

---

### Category 5: Sharing/Ownership Coercion

**What:** .dag value semantics → target language memory model

**Mechanism:** Coercion EFFECT applied after type coercion, gated on ValueContext

**.dag source:**
```
type Config {
  items: List<String>
  max_retries: Int
}
data default_config: Config = Config { items: [], max_retries: 3 }
```

**Resolution path for struct field (`items`):**
```
items: List<String>
  → coerce: Vec<String>
  → ValueContext: runtime struct field (is_constant = false)
  → sharing: Vec<String> is !Copy → Rc<Vec<String>>
```

**Resolution path for data declaration (`default_config`):**
```
default_config: Config
  → coerce: Config struct
  → ValueContext: constant data (is_constant = true)
  → sharing: SKIPPED (constant data, no Rc)
  → Rust: pub fn default_config() -> Config { ... }
```

**The lazy_static + Rc bug this prevents:**

When constant data declarations (like `data keywords = { ... }`) were
wrapped in `Rc<T>` and placed in `lazy_static!`, Rust produced E0277:
`Rc<T>` doesn't implement `Send + Sync`, which `lazy_static!` requires.

The fix: ValueContext distinguishes constant data from runtime values.
Constants skip Rc wrapping entirely.

**The Rc<dyn Fn> → PartialEq bug this prevents:**

When a struct has function-typed fields (like algebra witnesses), wrapping
in `Rc<dyn Fn>` and adding `#[derive(PartialEq)]` fails because function
types don't implement `PartialEq`. The fix: `has_fn_fields` in ValueContext
suppresses the derive.

**Target sharing strategies:**

| Language | Sharing mechanism | Exempt types | Effect on constants |
|----------|------------------|-------------|-------------------|
| Rust | `Rc<T>` | Copy types (i64, bool, f64, ()) | No wrap, `const`/`static` |
| Python | identity (GC) | all (GC handles sharing) | Module-level variable |
| Go | `*T` for large structs | small value types | Package-level `var` |
| English | N/A | all | Inline text |
| SPICE | wire sharing | N/A | `.param` declaration |

**Test obligations:**
```rust
// SHARING: Rc clone is cheap reference increment
let v: Rc<Vec<i64>> = Rc::new(vec![1, 2, 3]);
let v2 = v.clone(); // Rc reference increment, not deep copy
assert_eq!(Rc::strong_count(&v), 2);
assert_eq!(*v, *v2);
// CONSTANT: no Rc for constant data
fn keywords() -> HashMap<String, bool> {
    // Direct construction, no Rc, no lazy_static
    let mut m = HashMap::new();
    m.insert("if".to_string(), true);
    m
}
// FN_FIELDS: struct with fn fields skips PartialEq
// This should compile (no PartialEq derive):
struct Monoid<T> {
    op: fn(T, T) -> T,
    identity: T,
}
// This should NOT compile (PartialEq derive on fn field):
// #[derive(PartialEq)]
// struct Bad { f: fn() -> () }  // E0369
```

---

### Category 6: Refinement Coercion

**What:** .dag refined type → target language base type + validation

**Mechanism:** Chain through refinement to base type checkpoint

**.dag source:**
```
type NonNegativeInt = Int where range(min: 0)
type Email = String where pattern("^[^@]+@[^@]+\\.[^@]+$")
fn process(count: NonNegativeInt, contact: Email) -> Unit { ... }
```

**Resolution path:**
```
NonNegativeInt
  → refinement of Int, predicate: range(min: 0)
  → base: Int → "i64" (checkpoint)
  → Rust: i64 (predicate preserved as runtime validation)

Email
  → refinement of String, predicate: pattern(...)
  → base: String → "String" (checkpoint)
  → Rust: String (pattern preserved as runtime validation)
```

**Target outputs:**

| Language | NonNegativeInt | Email | Validation style |
|----------|---------------|-------|-----------------|
| Rust | `i64` | `String` | `debug_assert!(x >= 0)` or newtype |
| Python | `int` | `str` | `assert x >= 0` or `@validator` |
| Go | `int64` | `string` | `if x < 0 { return err }` |

**Test obligations:**
```rust
// VALID: value satisfying refinement succeeds
let count: i64 = 5; // NonNegativeInt
debug_assert!(count >= 0); // passes

// INVALID: value violating refinement fails
let bad_count: i64 = -1; // should fail NonNegativeInt check
// debug_assert!(bad_count >= 0); // panics

// SUBTYPE: refined value usable as base type
fn takes_int(x: i64) { /* ... */ }
let count: i64 = 5; // NonNegativeInt
takes_int(count); // always valid: NonNegativeInt <: Int

// SUPERTYPE REJECTION: base value NOT usable as refined type without check
// takes_nonneg(arbitrary_int) → must validate first
```

---

### Category 7: Structural Coercion (Product / Coproduct)

**What:** .dag user-defined types → target language structs and enums

**Mechanism:** Recursive structural traversal

**.dag source:**
```
type Shape
  = Circle { radius: Float }
  | Rect { w: Float, h: Float }
  | Point
```

**Resolution path:**
```
Shape = Coproduct [Circle, Rect, Point]
  → Rust enum
  → Circle: Product {radius: Float}
      → radius: Float → f64 (checkpoint, Copy)
  → Rect: Product {w: Float, h: Float}
      → w, h: Float → f64
  → Point: unit variant (no fields)
  → enum has fielded variants → !Copy → Rc in shared context
```

**Rust output:**
```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum Shape {
    Circle { radius: f64 },
    Rect { w: f64, h: f64 },
    Point,
}
```

**Test obligations:**
```rust
// EXHAUSTIVE MATCH: all variants covered
match shape {
    Shape::Circle { radius } => { assert!(radius >= 0.0); },
    Shape::Rect { w, h } => { assert!(w >= 0.0 && h >= 0.0); },
    Shape::Point => { /* unit */ },
}
// ROUND-TRIP: serialize → deserialize = identity
let s = Shape::Circle { radius: 3.14 };
let json = serde_json::to_string(&s).unwrap();
let s2: Shape = serde_json::from_str(&json).unwrap();
assert_eq!(s, s2);
// FIELD ACCESS: each field correctly typed
let Shape::Rect { w, h } = shape else { panic!() };
let area: f64 = w * h; // f64 arithmetic works
```

---

### Category 8: Algebraic Operation Coercion

**What:** .dag algebra method calls → target language expressions

**Mechanism:** Algebra inhabitant × operation → target expression template

When a .dag expression calls an algebra method (like `list.map(f)` or
`a + b`), the compiler must coerce the operation to the target language's
concrete syntax. This is operation coercion, distinct from type coercion.

**.dag source:**
```
fn double_all(items: List<Int>) -> List<Int> {
  items |> map(x => x * 2)
}
```

**Resolution:**
```
items |> map(f)
  → items: List<Int> → FreeMonoid<Int>
  → map is FreeMonoid.map
  → Rust inhabitant for FreeMonoid is Vec<T>
  → Vec.map → .iter().map(f).collect::<Vec<_>>()
  → Rust: items.iter().map(|x| x * 2).collect::<Vec<_>>()
```

**Operation mapping table (Rust):**

| Algebra | .dag operation | Rust expression |
|---------|---------------|-----------------|
| OrderedRing | `a + b` | `a + b` |
| OrderedRing | `negate(a)` | `-a` |
| OrderedRing | `compare(a, b)` | `a.cmp(&b)` |
| FreeMonoid | `concat(a, b)` | `[a, b].concat()` |
| FreeMonoid | `xs.map(f)` | `xs.iter().map(f).collect()` |
| FreeMonoid | `xs.fold(init, f)` | `xs.iter().fold(init, f)` |
| FreeMonoid | `xs.filter(p)` | `xs.iter().filter(p).cloned().collect()` |
| FreeMonoid | `xs.length` | `xs.len()` |
| BooleanAlgebra | `s.union(t)` | `s.union(&t).cloned().collect()` |
| BooleanAlgebra | `s.intersect(t)` | `s.intersection(&t).cloned().collect()` |
| BooleanAlgebra | `s.member(x)` | `s.contains(&x)` |
| PartialFunction | `m.lookup(k)` | `m.get(&k).cloned()` |
| PartialFunction | `m.insert(k, v)` | `{ let mut m2 = m.clone(); m2.insert(k, v); m2 }` |
| PartialFunction | `m.keys` | `m.keys().cloned().collect()` |

---

## Fail-Closed Contract

The fail-closed contract is the safety backbone of the coercion system.
It states:

> **If a target language does not declare an inhabitant for an algebra,
> compiling any .dag type that requires that algebra for that target
> MUST produce a compile error.**

### Why this matters at scale

Silent fallback is lethal for a multi-target compiler. If Rust silently
emits a bare type name when no inhabitant is found, the developer discovers
the bug as a Rust compiler error — far from the root cause. Worse, if the
bare name happens to match an existing Rust type, the code compiles but
behaves incorrectly.

Evidence from the current codebase:
1. `emit_node_type_leaf_rc` emits unrecognized names literally → silently
   wrong output for any type not in a hardcoded list
2. `emit_primitive_type` passes through unknown names → any typo in the
   type_map produces a bare string in the output
3. `_` placeholders in bare containers → `Vec<_>` emitted in struct fields,
   which is invalid Rust but was only caught downstream

### How to test it

For each algebra A in std/algebra.dag:
- If this language file declares an inhabitant for A: generate algebra
  law tests (positive tests)
- If this language file does NOT declare an inhabitant for A: generate
  a coercion refusal test (negative test)

The refusal test compiles a .dag source containing `type T = A<Int>` and
asserts the compiler produces a diagnostic matching:

```
ERROR: No <LANGUAGE> inhabitant for algebra <ALGEBRA>.
       Add an inhabitant declaration to extdeps/languages/<LANGUAGE>/types.dag.
```

This ensures:
1. The error message is actionable (tells you where to fix it)
2. The error is loud (compile error, not warning)
3. New algebras are fail-closed by default (no inhabitant = error)
4. Adding a new algebra to std/algebra.dag immediately forces every
   language to either declare an inhabitant or accept the compile error

### Cross-language completeness matrix

The compiler can generate this matrix from the inhabitant declarations:

| Algebra | Rust | Python | Go | SPICE | English |
|---------|------|--------|-----|-------|---------|
| OrderedRing | `i64` | `int` | `int64` | `real` | `integer` |
| ApproximateField | `f64` | `float` | `float64` | — (ERROR) | `decimal` |
| BooleanAlgebra (scalar) | `bool` | `bool` | `bool` | `wire` | `truth value` |
| BooleanAlgebra (collection) | `BTreeSet<A>` | `set[A]` | `map[A]struct{}` | — (ERROR) | `set of A` |
| FreeMonoid (scalar) | `String` | `str` | `string` | — (ERROR) | `text` |
| FreeMonoid (collection) | `Vec<T>` | `list[T]` | `[]T` | — (ERROR) | `list of T` |
| PartialFunction | `HashMap<K,V>` | `dict[K,V]` | `map[K]V` | — (ERROR) | `mapping` |

Every `— (ERROR)` cell is a test case: compiling List<Int> for SPICE MUST
produce a compile error because SPICE has no FreeMonoid collection inhabitant.

---

## Test Generation Strategy

### Per-declaration test generation

Every `InhabitantDecl` in `types.dag` generates a test suite. The compiler
reads the algebra definition from `std/algebra.dag`, matches it with the
inhabitant declaration, and emits test functions.

**Algorithm:**

```
for each InhabitantDecl D in language L:
  A = lookup_algebra(D.algebra) in std/algebra.dag

  // 1. Algebra law tests
  for each field F in A:
    if F is a binary operation (arity 2, returns Self):
      emit associativity test
      emit identity test (if A.identity exists)
      emit commutativity test (if A is Commutative*)
    if F is a unary operation (arity 1, returns Self):
      emit involution test (if applicable)
    if F is a predicate (returns Bool):
      emit reflexivity / antisymmetry / transitivity as applicable

  // 2. Identity element tests
  if D.identity_expr is not none:
    emit "assert_eq!(op(identity, x), x)" for each binary op

  // 3. Boundary tests
  emit overflow/underflow test (for numeric types)
  emit empty collection test (for container types)
  emit single-element test (for container types)

  // 4. Round-trip tests
  emit ".dag value → Rust value → serialize → deserialize → compare"

  // 5. Composition tests
  for each pair (D1, D2) where D2's type appears inside D1:
    emit nested coercion test (e.g., Vec<HashMap<String, i64>>)
```

### Negative test generation (coercion refusal)

```
for each algebra A in std/algebra.dag:
  if language L has NO InhabitantDecl for A:
    emit test that compiles "type T = A<Int>" for target L
    assert compiler output contains error diagnostic
    assert error message mentions A by name
    assert error message suggests adding inhabitant to L/types.dag
```

### Test coverage guarantee

The test generation is EXHAUSTIVE over the cross product:
- Every algebra × every language = either a law test suite OR a refusal test
- Every checkpoint × every language = a round-trip test
- Every pair of composable inhabitants = a composition test

No inhabitant declaration exists without a test. No algebra exists without
either a test or a documented refusal. This is the "every coercion we define
should be able to easily generate end-to-end comprehensive testing" requirement.

---

## Dissolution Path

When the coercion engine (M5) lands, the following current infrastructure
dissolves into it:

| Current | Dissolves into | Mechanism |
|---------|---------------|-----------|
| `rust_type_map` | `rust_type_checkpoints` | Structured type with Copy/default |
| `rust_container_templates` | `rust_algebra_inhabitants` | Keyed by algebra, not container name |
| `build_rc_types` | `rust_sharing` | Single SharingStrategy declaration |
| `emit_node_type_rc` | `coerce_type` + `apply_sharing` | Walk type tree, sidecast, apply effect |
| `emit_primitive_type` | `resolve_checkpoint` | Lookup in type checkpoint table |
| `TypeRendering` (E0c) | `coerce_type` | TypeRendering's named edges ARE the algebra relationships |
| `emit_container` | algebra inhabitant template application | `apply_type_template(inhabitant.rust_template, inner_types)` |
| `kernel_algebra_profile` | algebra inhabitant lookup | Profile = "which algebra does this type inhabit?" |

The E0c TypeRendering struct maps directly:
```
TypeRendering.element  → FreeMonoid<THIS>
TypeRendering.key      → PartialFunction<THIS, V>
TypeRendering.value    → PartialFunction<K, THIS>
TypeRendering.params   → Callable params
TypeRendering.return_type → Callable return
TypeRendering.inner    → Optional<THIS>
TypeRendering.shared   → SharingStrategy applied
TypeRendering.boxed    → recursive indirection
```

When coercion replaces TypeRendering, every named edge becomes an algebraic
relationship resolved through inhabitant declarations instead of structural
pattern matching.

---

## New Language Burden

Adding a new target language requires:

1. Create `dsl/extdeps/languages/<name>/types.dag`
2. Declare `TypeCheckpoint` entries for primitives the language knows
3. Declare `InhabitantDecl` entries for each algebra the language supports
4. Declare a `SharingStrategy` (or identity for GC languages)
5. Declare a `CallableRepr`

The compiler immediately:
- Generates law tests for every declared inhabitant
- Generates refusal tests for every undeclared algebra
- Reports the coverage matrix showing what works and what doesn't

Start minimal: declare only primitives. Get compile errors for containers.
Add container inhabitants. Get compile errors for callables. Add callable
repr. The compile errors GUIDE the implementation.

Example for a hypothetical SPICE target:

```
// dsl/extdeps/languages/spice/types.dag
module extdeps.languages.spice.types

data spice_type_checkpoints: List<TypeCheckpoint> = [
  { dag_name: "Int",    rust_type: "real",    is_copy: true, default_expr: "0" },
  { dag_name: "Float",  rust_type: "real",    is_copy: true, default_expr: "0.0" },
  { dag_name: "Bool",   rust_type: "wire",    is_copy: true, default_expr: "0" }
  // String: NOT declared → compile error for any .dag type using String
  // List: NOT declared → compile error for any .dag List type
]

data spice_algebra_inhabitants: List<InhabitantDecl> = [
  { algebra: "OrderedRing", rust_template: "real", arity: 0,
    is_copy: true, identity_expr: "0", std_import: none }
  // FreeMonoid: NOT declared → error for List/String
  // PartialFunction: NOT declared → error for Map
  // BooleanAlgebra: NOT declared → error for Set
]

data spice_sharing: SharingStrategy = SharingStrategy {
  wrapper_template: "{0}",  // SPICE is wire-level, no sharing overhead
  box_template: "{0}",
  exempt_predicate: "true"  // everything is exempt (no heap)
}
```

Compiling a .dag program that uses `List<Int>` for SPICE:
```
ERROR: No SPICE inhabitant for algebra FreeMonoid.
       SPICE has no collection types — ordered sequences are not
       representable in circuit description languages.
       Add an inhabitant to extdeps/languages/spice/types.dag
       or remove List usage from the .dag source.
```

This is correct behavior. SPICE genuinely cannot represent collections.
The error message is accurate and actionable.
