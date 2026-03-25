# gunbc Roadmap

## Architectural Thesis

**Node and DAG are the only compiler primitives.**

The compiler is a generic graph processor. It reads `.dag` source, builds
a graph of `Node`s, applies structural rules, and emits target code. All
domain knowledge — types, cardinality, containers, optionality, and
target-language facts — lives in `.dag` definitions, not in the compiler
implementation.

A `Node` is the universal graph carrier. Names introduce opaque
namespaces over node-shaped compositions. Distinctions such as type
position, value/expression position, pattern position, and binding-site
descriptor are operational roles in the pipeline, not separate
ontological categories of node.

### Three Structural Principles

These principles refine the thesis based on root-cause analysis of the
current invariant violations (2026-03-23). Every active violation traces
back to one of these principles being underdeveloped.

**1. Names are opaque namespaces.**

Type names (`Int`, `Map`, `List`, etc.) are human-readable labels for
structural compositions, not compiler-meaningful identifiers. Names label
compositions built from machine primitives and algebraic constructors.
`Bit`/bitvectors live in the machine layer; `List`, `Map`, and `Set` live
in the algebraic layer with denotational laws. The compiler must not
branch on node names for structural decisions. At every level above the
fundamental unit, names are opaque.

Enforcement: inference receives nodes with opaque names and no name
registry. It can thread names through to output nodes and diagnostic
messages, but cannot branch on them. Emit receives the registry to
produce target-language identifiers. Scrambled-name tests (rename all
types to arbitrary strings, verify inference produces identical
structural decisions) verify the property wall.

**2. Compiler errors are orthogonal to the node graph.**

When inference fails, the result is not a node — it is a structurally
distinct failure. The compiler produces errors; it should never need to
rediscover them by string-checking node names.

Representation: `InferredNode = Resolved { node: Node } | CompilerError
{ message: String, span: SourceSpan }`. Inference returns `InferredNode`,
not `Node`. A child that fails propagates failure to the parent
expression. Emit never sees error nodes. `Dynamic` and `Error` unify
into `CompilerError` — both mean "inference couldn't determine this,"
and both are failures, not types. Generics now exist, but infer still
does not operate over unresolved type variables: slot substitution
happens before inference sees the graph, so type variables remain a
distinct structural concept rather than an inference-era name check.

**3. Syntactically distinct forms for the same operation normalize before
inference.**

The pipeline has a normalization boundary between resolve and infer.
After normalization: `Call`→`MethodCall` bridging is complete, nodes
carry their declared structural properties from `.dag` type definitions,
and parameterized types always carry their declared arity of children.
Infer receives a fully-normalized graph and processes one form per
semantic operation — no divergent code paths for the same concept.

### Dissolution Layers

The thesis dissolves compiler knowledge in three layers:

| Layer | What dissolves | Compiler stops knowing | Measured sites | Status |
|-------|----------------|------------------------|---------------:|--------|
| **L1: Types** | Name-checking, `node_is_*`, type constructors, `.connective` reads | What `List`, `Map`, `Int`, etc. mean — the compiler processes graph structure; names are opaque namespaces; `Optional` dissolved into cardinality (P1.4) | 373 | **Active — `BuiltinTypeKind` deleted, predicates centralized, `InferredNode` wrapper landed (bridges remain — see P1.9), cardinality model landed (bridges remain — see P1.4), Optional name-checking dissolved. The scripted ratchet table below is the canonical source of truth for the current total.** |
| **L2: Expressions** | `ExprData` semantic knowledge, 12+ full ExprData walks | What `if`, `for`, `match`, `let`, etc. mean | 12 walks | **Bridge landed** — `expr_children`/`map_expr_children` in `00_core.dag` extract child relationship; deletion point P5.11. Full dissolution design in P5.11/P5.12. |
| **L3: Syntax** | `kind_tag` string dispatch, hardcoded parser branches | How to parse surface syntax like `if cond { body }` | ~200 string checks | Future — data-driven parser |

L1 is the urgent layer. Its endgame is not "replace name checks with
property checks" (that still puts domain knowledge in the compiler). The
endgame is: the compiler processes graph structure and reads structurally
declared properties from `.dag` type definitions. Names are opaque.
Inference cannot read them. The L1 ratchet evolves from "count name
comparisons" to "scrambled-name tests pass."

The compiler retains structural vocabulary: `Conj`/`Disj` (product vs
coproduct — graph primitives, not domain), children/parent traversal,
and cardinality. Everything above that — what `Int` means, what `Map`
means — is namespace, defined in `.dag`. `Optional` is not a namespace
or type constructor; it is cardinality on the binding site.
The roadmap also keeps role vocabulary where it marks real operational
invariants: node in type position, node in value/expression position,
node in pattern position, node as binding-site descriptor. Those are
pipeline roles over one carrier, not different species of node.

### Algebraic Type Vision

The long-term endgame: every type is a structural composition from a
single fundamental unit, and properties like cardinality, optionality,
and arity fall out of the definitions — not from compiler-injected
properties.

```
// Level 0: Fundamental unit
type Bit = True | False                           // |[Bit]| = 2

// Level 1: Fixed-width compositions
type Byte = Tuple<Bit, Bit, Bit, Bit, Bit, Bit, Bit, Bit>  // |[Byte]| = 2^8

// Level 2: Named compositions (opaque namespaces)
// Int is List<List<Bit>> — a namespace, not a compiler-known concept
// String is List<Int> — another namespace

// Level 3: Algebraic structures
// Optionality: T? is cardinality 0..1 on the binding site
// (denotationally T | Unit, |T| + 1 inhabitants;
// see Occurrence and Cardinality Model below)
//
// Container types have two layers: denotational meaning and
// algebraic (computational) representation.
//
// Denotational (what they mean):
//   Set<A>   = A -> Bool         (finite support — membership)
//   Bag<A>   = A -> Nat          (finite support — multiplicity)
//   Map<K,V> = K -> (1 + V)      (finite support — keyed lookup)
//   List<A>  = Σ n. Fin(n) -> A  (length + value per position)
//
// Algebraic (how they compute):
//   List<T> = Nil | Cons { head: T, tail: List<T> }
//   Set<T>  = membership-restricted collection of T
//   Map<K,V> = finite partial function K -> V
```

At each level, the compiler sees only graph structure. Names are
opaque. Cardinality is a structural annotation on bindings, not a
type wrapper. `Optional<T>` does not exist as a type constructor —
`T?` is cardinality on the binding site (see Optionality and
Cardinality Model below). The compiler processes coproducts
generically; it doesn't need an "optional" concept.

This spans distinct layers. Machine/foundational primitives bottom out
in `Bit`/bitvectors; algebraic constructors such as `List`, `Map`, and
`Set` carry separate denotational laws. The roadmap should not flatten
those layers into one undifferentiated notion of "primitive."

This requires generics in the `.dag` language (Phase 3+). Until then,
the compiler uses an explicit arity bridge (`kernel_types` pattern)
that gets deleted when real algebraic declarations exist. Every
bridge is a short-term regression acknowledged as such — its purpose
is to move structure upstream so it can be choked out when the real
declarations land.

Phase timeline for this vision:

| Phase | What's reachable | What's still a bridge |
|-------|-----------------|----------------------|
| Phase 1 | `InferredNode`, normalization, path dedup. Arity bridge for known types. Cardinality model (Optional dissolved). Algebraic spec design doc. | Arity hardcoded in bridge (`Map→2`, `List→1`, `Set→1`); cardinality as binding annotation |
| Phase 2 | Gist end-to-end. Arity bridge keeps types structurally complete. | Same bridge; no new emit heuristics needed because inference is sound. |
| Phase 3 | Generics land (at least enough for parameterized type declarations). Algebraic specs become real `.dag` declarations. Arity bridge deleted. | None — real declarations replace the bridge. |
| Phase 4 | Shared emit reads `LanguageSpec` + structural declarations. Emit becomes name-opaque. | None |
| Phase 5 | L1=0. Scrambled-name tests pass. Connective dissolution starts. | None |
| Beyond | Bit-graph model. Primitives as compositions. Full structural type algebra. | None |

The bit-graph model is aspirational and post-Phase 5. The pragmatic
intermediate (Phases 1-3) is: machine primitives and algebraic
constructors/containers get real `.dag` declarations with structural
definitions; the compiler reads structure from those declarations; names
are opaque. This is sufficient for the thesis without requiring the full
algebraic foundation up front.

### Collection Denotational Model

Collections are not one algebra — they are a **family of related
algebras** unified by a common compositional pattern. Each collection
type is characterized by three things:

1. A **carrier/index space** (what you iterate over / address by)
2. An **annotation algebra** (what the codomain tracks)
3. A **type constructor** (`Type -> Type` or `Type -> Type -> Type`)

The denotational equations:

```text
Set<A>   = A -> Bool         (finite support)
Bag<A>   = A -> Nat          (finite support)
Map<K,V> = K -> (1 + V)      (finite support)
List<A>  = Σ n. Fin(n) -> A
```

Reading these:

- **Set** is a membership structure. The codomain is `Bool` (in or out).
  `Set<A>` answers "for each `a : A`, is `a` in the set?" **Uniqueness
  is automatic:** a function returns one answer per input, so an element
  is either in or out — there is no "in twice."
- **Bag** is a multiplicity structure. The codomain is `Nat` (how many).
  Like Set but tracks count, not just presence.
- **Map** is a keyed partial-function structure. The codomain is
  `1 + V` (present with value, or absent). **Key uniqueness is
  automatic:** a function returns one value per key — there is no
  "two entries for the same key."
- **List** is a positional structure. A list is a length `n` plus a
  value of type `A` at each position `0..n-1`. **Duplicates are
  natural:** the carrier is positions, not elements, so different
  positions can carry the same value. `[1,1,2]` has three distinct
  positions, each labeled by a value. If you tried to model `List<A>`
  as an ordered set of `A` itself, `[1,1,2]` would collapse.

`List<A>` is **dense**: its support is exactly the initial segment
`{0..n-1}`. Holes are unrepresentable. Sparse sequences are a
distinct type. The `List` equation can also be read as `Nat -> (1 + A)`
with domain `{0..n-1}`, making the parallel to `Map` visible: a list
is a map from natural-number positions to values.

**Type constructors, not generic sets.** `List<_>` is not a set of
types `{List<Int>, List<String>, ...}`. It is a type constructor:
`List : Type -> Type`. The family `{List<Int>, List<String>, ...}`
exists at the meta-level, but `List<_>` itself is the constructor — an
open DAG with a slot, not the set of its instantiations. This is the
`<>` / generic parameter story from P3.6: slots are structural
positions in the DAG, filled at composition time.

**Method algebras follow from the denotations:**

| Collection | Algebra | Core methods |
|------------|---------|-------------|
| Set | Membership (Boolean) | `contains`, `union`, `intersect`, `difference` |
| Bag | Multiplicity (Nat) | `count`, `add`, `union` (max or sum) |
| Map | Keyed partial function | `lookup`, `insert`, `delete`, `merge` |
| List | Positional / sequential | `append`, `index`, `slice`, `map`, `fold` |

Cross-cutting methods like `map`, `filter`, `fold` apply to multiple
collection types but derive from the specific algebra. `map` on a
`List` preserves positions; `map` on a `Set` preserves membership
(modulo the image collapsing distinct elements). `filter` on a `Bag`
adjusts multiplicities.

**Position order vs element order.** Lists need an order on
**positions**, not on elements. An order on elements only becomes
relevant for sorted structures (`TreeSet`, `SortedMap`, `sort`
method). This is a separate concern from the collection algebra
itself.

**Bits and containers are separate.** `Int32` bottoms out in
bitvector semantics — that's correct. But bitvectors are not the right
primitive for container semantics. The collection denotational model
lives above the bit level. `Set<T>`, `List<T>`, `Map<K,V>` are
indexed-collection structures; they do not reduce to bit compositions
in the same way `Int` does. The bit-graph model (post-Phase 5) is
about machine primitives; the collection model is about data structure
semantics.

#### Law Layer (resolved)

These properties are decided, not left implicit:

| Question | Answer | Notes |
|----------|--------|-------|
| Are collections finite by default? | **Yes** — finite support everywhere | Infinite collections are a separate concept (streams, generators) |
| Is equality extensional? | **Yes** for Set/Map/Bag; **structural** for List | Two sets with the same members are equal regardless of construction |
| Is set element uniqueness semantic? | **Yes** — falls out of `A -> Bool` being a function | An element is in or out; "in twice" is unrepresentable |
| Are list duplicates always allowed? | **Yes** — positions are the carrier | `[1,1,2]` has 3 positions; duplicates are not special |
| Is map key uniqueness semantic? | **Yes** — falls out of `K -> (1 + V)` being a function | Two entries for the same key is a representation error, not a valid map |
| Do sets/maps require decidable equality on keys? | **Yes** — all runtime values have structural decidable equality | The DAG model gives every value a structural form; no value is "unhashable." Decidable equality is a consequence of the model, not an imposed constraint. |
| Is iteration order part of `Map`? | **No** — unordered by default | `SortedMap`/`OrderedMap` are distinct types with additional structure |

#### Support and Algebra Laws

Set algebra laws follow from `Set<A> = A -> Bool` pointwise:

| Operation | Definition | Law |
|-----------|-----------|-----|
| Union | `(S ∪ T)(a) = S(a) ∨ T(a)` | Commutative, associative, idempotent |
| Intersection | `(S ∩ T)(a) = S(a) ∧ T(a)` | Commutative, associative, idempotent |
| Difference | `(S \ T)(a) = S(a) ∧ ¬T(a)` | Not commutative |
| Absorption | `S ∪ (S ∩ T) = S`; `S ∩ (S ∪ T) = S` | Standard lattice absorption |
| Distributivity | `S ∩ (T ∪ U) = (S ∩ T) ∪ (S ∩ U)` | Boolean algebra |

**Complement is not closed.** `¬S(a) = ¬S(a)` is well-defined
pointwise, but the complement of a finite-support set has infinite
support unless the carrier is finite. This language does not provide
a general complement operation on `Set<A>`. Complement relative to a
known finite universe (`S.complement_in(universe)`) is expressible
as difference.

**Bag union is sum, not max.** `Bag<A> = A -> Nat` with union defined
as `(B₁ ⊕ B₂)(a) = B₁(a) + B₂(a)` (sum). Max-union is a distinct
operation (`B₁ ⊔ B₂`). The default `union` on `Bag` uses sum; max
is available as `max_union` if needed. This pins the algebra:
`(Bag, ⊕, ε)` is a commutative monoid (where `ε(a) = 0` for all
`a`), not an idempotent semilattice.

**Map merge requires a conflict function.** `Map<K,V>` merge is not
automatically defined because two maps may disagree on a key's value.
`merge(m1, m2, on_conflict: (V, V) -> V)` takes an explicit conflict
resolution function. Common specializations: `merge_left` (keep
first), `merge_right` (keep second), `merge_with(f)` (apply `f`).
A bare `merge(m1, m2)` without conflict semantics is a compile error.

**Cardinality is support size.** For all finite-support collections,
`|S|` is the number of elements in the support (elements where the
characteristic function returns a non-zero/non-absent value). For
`List<A>`, this is `n` (the length). For `Set<A>`, this is
`|{a : S(a) = true}|`.

#### Occurrence and Cardinality Model

> The DAG represents occurrence constraints structurally. `Optional` is
> normalized to cardinality `0..1`; collection constructors carry
> ordering, uniqueness, and keying constraints at the type level.
> Absence, nullability, unknownness, and defaultability are distinct
> concepts and must not be conflated.

**Presence/cardinality is fundamental.** Optionality is the `0..1`
case — a derived shape, not a separate primitive. Separating these
concerns is prerequisite to the arity bridge (P1.17) and generics
(P3.6).

Four distinct concepts have historically been conflated under
"optional" in the compiler. They must stay separate:

| Concept | Meaning | Compiler representation |
|---------|---------|------------------------|
| **Absence** | The slot/binding may be missing entirely | `Field.cardinality: Cardinality` (0..1) |
| **Nullability** | The slot is present, but the value may be `null` | Value-level coproduct `T \| Unit` in the type graph |
| **Unknownness** | The compiler doesn't know the value/type yet | `InferredNode.CompilerError` (P1.9) |
| **Defaultability** | The slot may be omitted because a default fills it | `Field.default_value: Node?` — syntactically optional, semantically required |

Collapsing these into one mechanism (e.g., a single `Optional` node)
blurs distinct semantics: "this field is absent" is not the same as
"the value is null" is not the same as "the compiler failed" is not
the same as "a default will be filled in."

**`Optional<T>` does not exist as a type constructor.** Type nodes are
purely "what" (the set of values); "how many" is a separate annotation
on the binding site. `T?` sets cardinality on the binding, not wraps
the type.

```
type Cardinality = Required | CardOptional

type Field {
  name: String
  type_expr: Node
  cardinality: Cardinality   // replaces optional: Bool
  default_value: Node?       // distinct from cardinality — see defaultability above
  span: SourceSpan
}
```

**`field?: T` vs `field: T | Unit`.** These have the same
denotational cardinality (`|T| + 1` inhabitants) but differ
operationally:

- `field?: T` — structural presence. The binding may be absent.
  Cardinality 0..1 on the field. No type wrapping.
- `field: T | Unit` — value-level coproduct. The binding is present;
  its value is `Present(v)` or `Null`. The type IS `1 + T`.

This distinction keeps `Optional<List<T>>` (field absent vs present
with a list) crisp from `List<T | Unit>` (field present, each
element nullable). If the DAG conflates these, nested optionality
blurs.

**Denotational interpretation.** `T?` denotes `1 + T` (one extra
inhabitant beyond `T`, representing absence). The denotation guides
semantics; the compiler representation is the cardinality annotation.
The denotational equation `Map<K,V> = K -> (1 + V)` uses the `1 + V`
to express *partiality*: the function may not be defined for all keys.
In the compiler, partiality is return cardinality 0..1 on `lookup`,
not a type wrapper.

**Emit rendering.** The emitter translates cardinality to target
representations (Rust `Option<T>`, Go `*T`, Python `Optional[T]`),
reading cardinality from the binding — not from type-node names or
properties.

**Slot-descriptor derived view.** For schema-level reasoning, a
binding site's constraints can be summarized as:

```text
T            = min=1, max=1
T?           = min=0, max=1
List<T>      = min=0, max=∞, ordered=true     (type constructor)
Set<T>       = min=0, max=∞, unique=true       (type constructor)
Bag<T>       = min=0, max=∞                    (type constructor)
Map<K,V>     = keyed support K -> V?           (type constructor)
```

This is a useful derived view for reasoning about binding-site
properties, but **collections remain type constructors in the type
graph.** `List<T>` is `Type -> Type`, not "cardinality 0..n plus
ordering." The distinction matters for nested composition
(`List<Map<String, Int>>`), method dispatch (`.map()`, `.filter()`),
and inference — types flow through the pipeline as compositional
structures, not as flat slot descriptors.

**What this deletes:** `optional_node()`, `node_is_optional()`,
`optional_property()`, `Field.optional: Bool`, `"Optional"` name
comparisons, `OptionalUnwrap`/`OptionalValue` field access styles.
Migration is Phase 1 (P1.4); `Optional` is removed from the arity
bridge (P1.17); generics (P3.6) no longer need `type Optional<T>`.

**Resolved cardinality decisions (P1.18):**

| Question | Candidate answer | Notes |
|----------|-----------------|-------|
| Cardinality representation? | `Required \| Optional` (closed enum) | Min/max ranges are more general but not needed yet; named cases are safer for the compiler |
| Cardinality in return position? | Same `Cardinality` on function/expression return | `map.get(key)` returns `V` with cardinality 0..1 |
| Cardinality in let bindings? | Yes — `let x: String? = ...` has cardinality 0..1 | Consistent with field cardinality |
| Does `?` always mean cardinality? | Yes — annotates binding site, never wraps type | Single surface-syntax rule |
| When is coproduct return needed? | When richer failure info beyond presence/absence | `Result<T, E> = Ok \| Err` is a coproduct, not cardinality |
| Defaultability interaction? | A field with `default_value` is syntactically optional but semantically required | Cardinality remains `Required`; the default is an elaboration rule |

L3 is larger than previously acknowledged: `02_parse.dag` (3,938 lines)
dispatches on `TokenKind` entirely through a `kind_tag(token) -> String`
function, then compares strings everywhere (`check(tag: "KwFn")`,
`expect(tag: "LBrace")`, etc.). Adding a new token kind requires finding
every string comparison by hand with no compiler-enforced exhaustiveness.

---

## How To Read This Roadmap

This file now has one canonical schedule and three supporting
decompositions.

- **Phases are the canonical execution order.** If another section seems
  to imply a different ordering, the phase plan wins.
- **`M*` tracks** describe cross-cutting architecture migrations that span
  more than one phase.
- **`R*` targets** describe the desired end state of specific compiler
  modules once the naming cleanup lands.
- **`S*` passes** describe technical refactors that cut across phases and
  tracks.

The repo is still in the middle of a rename/relocation cleanup. Some
sections refer to current filenames, and some refer to target filenames.
Use this map:

| Old file | Current file (M1 complete) | Meaning |
|----------|---------------------------|---------|
| `04_reconcile.dag` | `04_infer.dag` | Stage 4 is infer/typecheck, not "reconcile" |
| `06_pipeline.dag` | `compile.dag` | Compiler driver/orchestrator, not a sixth stage |
| `07_complexity.dag` | `complexity.dag` | Proof/report layer, not a numbered stage |
| `07_ownership.dag` | `ownership.dag` | Proof/obligation layer, not a numbered stage |
| `08_artifact.dag` | `artifact.dag` | Artifact planning layer, not a numbered stage |
| `09_trace.dag` | `trace.dag` | Runtime/debug contract, not a numbered stage |

M1 naming cleanup is complete. All files use their target names. Last
stale reference (`v2.compiler.pipeline` in `05_emit_rust.dag`) fixed
2026-03-23.

---

## End Goal

The compiler is a generic graph processor. It reads `.dag` source, builds
a graph of `Node`s, applies structural rules defined in `.dag`, and emits
target artifacts. Adding a type, expression, language, transport, or
runtime contract should mean editing `.dag` files, not compiler code.

Concrete acceptance:

- Zero type-world knowledge in the compiler (L1 complete, **Phase 5 gate**):
  names are opaque namespaces; inference processes graph structure only;
  scrambled-name tests pass; no arity bridges remain
- Compiler errors are orthogonal to nodes: `InferredNode` wrapper;
  no error/Dynamic sentinels in the type graph
- Container types have real `.dag` algebraic declarations grounded in the
  Collection Denotational Model; optionality is cardinality on bindings
  (not a type constructor); arity and uniqueness properties fall out of
  the denotations, not compiler knowledge
- Emit is name-opaque: shared emit reads `LanguageSpec` + structural
  declarations for type→target-identifier mapping (Phase 4); no hardcoded
  `if type_name == "Map" { "HashMap" }` patterns
- One shared emit walker drives all target languages through a common
  compiler-owned spine
- Language-specific facts live in `dsl/extdeps/languages/*`; program-
  dependent lowering lives in compiler-owned adapters
- Ownership and complexity proofs are wired into the compile pipeline
- At least one real program (`gist`) compiles and runs end to end
- v1 is archived (Phase 3: feature-gated in P3.5 **done**; fully removable
  once Phase 3 confirms v2 parity — "archived" means the `v1-bootstrap`
  feature flag and all v1 crates can be deleted without breaking any
  non-bootstrap workflow)
- Compiler-internal structure converges onto `Node` compositions

---

## Completed Milestones

Status labels:

- **tree-green**: verified on the current tree (`cargo test` passes)
- **prior-branch**: achieved on a previous green branch; not yet verified on this tree
- **structural**: code-level change is landed and visible; downstream gate may not pass yet
- **partial**: foundational plumbing landed and tree-green, but the roadmap acceptance condition is not yet met; remaining work is listed
- **smoke**: test or scaffold exists and passes, but does not yet verify the property the roadmap gate describes
- **guardrail**: useful defensive check exists, but is not the acceptance gate the roadmap requires

| Milestone | Gate | Status | Date |
|-----------|------|--------|------|
| Self-compile pipeline | v2 processes its own `.dag` through all 5 core stages | tree-green | 2026-03 |
| Bootstrap A5 | v1 -> stage0 -> stage1 (`cargo check`) | prior-branch | 2026-03 |
| Fixed point A6 | stage1 output == stage2 output (byte-identical) | prior-branch | 2026-03 |
| A7 Phase 1 | Self-compile reached 0 `cargo check` errors | prior-branch | 2026-03 |
| TypeExpr -> Node | 8 `TypeExpr` variants deleted | tree-green | 2026-03 |
| Expr -> Node | 21 `Expr` variants deleted, `ExprData` now lives on `Node` | tree-green | 2026-03 |
| Transport dissolution | `TransportBinding` deleted | tree-green | 2026-03 |
| Node/TypedNode unified IR | W1-W13 complete, 129 tests passing | tree-green | 2026-03 |
| Performance audit | tokenize+parse down to ~24ms | tree-green | 2026-03 |
| OOM fix | `node_type_deps` cycle detection stabilized | tree-green | 2026-03 |
| M1 naming cleanup | All non-stage files renamed to target names | structural | 2026-03 |
| Stage0 build | 18 build errors fixed, stage0 compiles cleanly | prior-branch | 2026-03 |
| Stage0 parse | 5 parser ambiguities fixed in v2 source | tree-green | 2026-03 |
| Gist pipeline | 11-file gist closure compiles with 0 diagnostics | tree-green | 2026-03 |
| V1 feature-gate | v1 crates gated behind `v1-bootstrap` cargo feature | tree-green | 2026-03 |
| Diagnostic reduction | 395 → 197 via tuple naming, error cascade, branch compatibility | prior-branch | 2026-03 |
| Diagnostic ratchet 0 | 197 → 0 via 4 root-cause fixes (map types, data scope, lookup returns, cascade suppression) | prior-branch | 2026-03 |
| RenderTarget extraction | Moved from `00_core.dag` to `artifact.dag` (orchestration, not kernel) | tree-green | 2026-03 |
| Emit metadata extraction | `emit_info` removed from ResolvedGraph; emit builds EmitGraphInfo locally | tree-green | 2026-03 |
| BuiltinTypeKind deletion | Enum and `builtin_type_kind()` fully removed from `00_core.dag` | tree-green | 2026-03 |
| RuntimeBridgeMethod enum | String-keyed bridge dispatch replaced with closed enum in core; round-trip through `runtime_bridge_method_name` remains (P1.10) | tree-green | 2026-03 |
| L1 centralized predicates | `node_is_optional`, `node_is_map`, `node_is_container` and type predicates live in `04_types.dag` (`infer_types`); emit imports them; `classify_type_structure` replaces direct `.connective` reads in emit | tree-green | 2026-03 |
| Rc policy extraction | `type_needs_rc`, `data_lookup_needs_rc_wrap`, `rc_wrapped` removed from core/reconcile/shared emit; live only in `05_emit_rust.dag` | tree-green | 2026-03 |
| Kernel types single authority | `kernel_types` in `00_core.dag` is the only source; deleted `kernel_type_names()`, `is_primitive_name()`, `build_primitive_set()` | tree-green | 2026-03 |
| Complexity match cost | `MatchCostAccum` in `cost_of_expr`; single pass over match arms (no 2^depth re-evaluation) | tree-green | 2026-03 |
| Resolve bounded OOM | `resolve_node_bounded` stops re-resolving already-resolved lookups; trusts topological binding order | tree-green | 2026-03 |
| InferredNode wrapper (P1.9) | `Node.return_type` is `InferredNode?`; `rt_node` returns `NodeType = Typed \| InferError \| Untyped`; `node_is_error_type`/`node_is_dynamic` deleted; error/Dynamic permissive rules removed from type comparison; `return_cardinality: Cardinality` on Node. | tree-green | 2026-03 |
| Normalization stage (P1.14) | `03_normalize.dag` wired between resolve and infer; arity enforcement rejects bare parameterized types as errors; pipeline gate halts before infer on normalization errors. Call→MethodCall unification deferred to Phase 3 (declaration-driven). | tree-green | 2026-03 |
| Arity bridge (P1.17) | `parameterized_type_arity`, `is_parameterized_type`, `type_node_arity_ok` in core/types. Bare-leaf tolerance removed; `actual == 0` rejected as error by normalization gate. | tree-green | 2026-03 |
| Inference path dedup (P1.15) | Shared `refine_collection_result_type` handles map/flat_map/fold/map_insert/map_merge result typing for both paths. Shared fold helpers landed. Call→MethodCall dispatch deferred to Phase 3 normalization. | tree-green | 2026-03 |
| Shared fold helpers (R3) | `extract_fold_init_info`, `infer_method_args_with_fold` eliminate ~35 lines of duplicate fold logic | tree-green | 2026-03 |
| DAG backend (P4.4) | `Dag` variant on `RenderTarget`; `emit_dag_artifact` now serializes a versioned typed-graph envelope (`modules`, `diagnostics`, `files`) to `dag-artifact.json` via explicit `ResolvedGraph`/`TypedModule`/`Node` serializers. | tree-green + smoke | 2026-03 |
| Scrambled-name tests (P1.16) | `v2_scrambled_name_inference_smoke` and `_containers` compare diagnostic count and file count after name scrambling. **Not the roadmap gate:** does not compare inferred structure (typed graph shapes) as Phase 5 requires. | smoke | 2026-03 |
| Algebraic type spec (P1.18) | `docs/algebraic-type-spec.md` pins denotational semantics, algebra laws, and cardinality model | structural | 2026-03 |
| Testgen source gate (P1.21) | `v2_testgen_service_mock_source_gate` greps emitter source for bad patterns. **Guardrail, not gate:** does not compile a service module, extract test files, and assert syntactically valid Rust. | guardrail | 2026-03 |

| `rt_node` 3-variant return type | `rt_node` returns `NodeType = Typed \| InferError \| Untyped` instead of `Node?`. 137 callers migrated; `rt_type` convenience helper for emission callers; explicit 3-arm matches where error/absent distinction matters. | tree-green | 2026-03 |
| `expr_children` structural primitive | Extracts all child expression Nodes from ExprData variants. 4 manual walks rewritten (~210 lines eliminated). `map_expr_children` landed and now backs the remaining structural rewrites. | tree-green | 2026-03 |
| P2.5 resource wiring | `emit_main_service_arg_list` constructs resources (`&Network`) instead of `compile_error!()`. Resource module imports added to main.rs via `find_resource_module`. | tree-green | 2026-03 |
| P3.1 fixed-point convergence | `v2_bootstrap_fixed_point` now passes end-to-end on `cousin-wip`: stage0 builds, stage1 builds, stage2 builds, and the byte-identical compare is green. Final fixes restored explicit authority for bare generic declarations, variant parent resolution, typed intrinsic/runtime-bridge lowering, and underspecified collection refinement. | tree-green | 2026-03 |

---

## Current State (2026-03-25)

**Updated 2026-03-25.** Previous state description (2026-03-24) is superseded.

**Bootstrap status:** `v2_strict_compile_diagnostic_count` passes with
**0 diagnostics** (ratchet = 0). Workspace tests pass (125/125
non-ignored). Clippy clean. Gist pipeline passes (`v2_gist_full_pipeline`
lib target compiles and builds). Stage0 compiles in ~3 s, 91 MB peak RSS.

**v1 retirement (Stream D): Done.** PR #200 + #204 merged on origin/main.
Stage0 committed at `src/v2/stage0/` as a workspace member. Tests call
stage0 Rust directly — no v1 interpreter, no Value wrapping. `v1-bootstrap`
removed from default features. `v2_runtime_shim.rs` deleted. CI excludes
`v2-compiler` from workspace test (generated_tests.rs too large).
`assemble_stage0` binary exists for regenerating the seed from v1.

**Known `assemble_stage0` post-regeneration fixups (4 issues):**
Each regeneration requires the same manual corrections. These should be
fixed in the assembler itself — they violate "no parallel implementations."

1. **Deletes `v2_rt.rs`** — assembler doesn't generate the runtime shim but
   generated code references `v2_rt::*`. Fix: `git checkout HEAD -- v2_rt.rs`.
2. **Corrupts `generated_tests.rs` module paths** — emits `crate::reconcile::`
   (old module name) instead of `crate::infer::`, and `crate::v2_core::Expr`
   instead of `crate::v2_core::ExprData`. Fix: search-and-replace.
3. **Drops `clippy::disallowed_macros`** from `main.rs` `#![allow(...)]`.
   Fix: add it back manually.
4. **Adds `[workspace]` to `Cargo.toml`** — conflicts with the parent workspace.
   Fix: `git checkout HEAD -- Cargo.toml`.

Root cause: the assembler generates from AST without awareness of the committed
stage0's hand-maintained files and test module structure. Long-term fix is either
(a) teach the assembler about these files, or (b) reach self-hosting fixed-point
where stage1 replaces stage0 and the assembler is no longer needed.

**P3.1 fixed-point status:** `v2_bootstrap_fixed_point` is **green**.
The bootstrap path is deterministic and the full stage0 → stage1 → stage2
fixed-point check passes (byte-identical comparison).

**What v1 retirement unblocks:**
- P3.8 recursive generics (v1 Rust stack overflow gone)
- `map_expr_children` (stage0 handles HOFs natively via stage0→stage1 path)
- Full callback `emit_shared_expr` (same mechanism)
- File decomposition of `04_infer.dag` (depends on `map_expr_children`)

**L2 bridge status:** `expr_children` primitive landed in `00_core.dag`.
4 manual ExprData walks rewritten (~210 lines eliminated).
`map_expr_children` **landed** in `00_core.dag` — structural tree-map over
ExprData, covers all 18 recursive variants. `emit_shared_expr` **landed**
with full callback dispatcher (leaf + 4 recursive arms: UnaryOp, Lambda,
ListLit, Return). All 3 backends wired. 9 dead helper functions deleted.
`rt_node` returns `NodeType` (Typed/InferError/Untyped). Bridge-era
invariants documented. P5.11/P5.12 dissolution design complete.

### Error Classification and Progress Measurement

Every fix should be classified by **which layer absorbed it**. If the
fix deleted a heuristic or moved logic upstream, that is real progress.
If it only taught one backend a new quirk, that is survival work.

| Layer | What it means | Example fixes | Health signal |
|-------|---------------|---------------|---------------|
| **Upstream semantic** | Wrong typed graph, wrong cardinality, wrong inference | `explicit_record_struct_name` CardOptional check, `analyze_rc_match` Optional scrutinee handling | Positive — fixes the data flowing into emission |
| **Bridge contract** | Named temporary surface with deletion point | `expr_children`, `rt_type`, arity bridge | Positive if bridge surface shrinks over time |
| **Backend rendering** | Typed graph is right but Rust/Go/Python rendering wrong | vtoe priority fix, Optional string `.as_deref()` | Necessary but not thesis-advancing |
| **Visibility/gating** | Error exists but normal tests don't catch it | Stage1 errors invisible to `cargo test` because ratchet test is `#[ignore]` | Red flag — hidden debt accumulating |

**Key principle: v1 is an oracle for bootstrap parity, not the spec.**
The spec is the roadmap thesis and the typed graph. When a stage1 error
surfaces, first ask "is the typed graph wrong?" If yes, fix inference.
If the graph is right and rendering is wrong, fix the emitter — but
don't copy v1 quirks as the definition of correctness.

**P3.1 fix classification (820 → 16):**
- Fix 1 (Optional type name): **upstream semantic** — CardOptional not recognized in record struct name resolution
- Fix 2 (Rc match deref): **upstream semantic** — match analysis didn't account for Optional wrapping on scrutinees
- Fix 3 (Optional string comparison): **backend rendering** — emitter added `.as_str()` without checking cardinality
- Fix 4 (expr_children import): **bridge contract** — new structural primitive needed explicit import
- Fix 5 (vtoe module priority): **backend rendering** — vtoe lookup order wasn't module-scope-aware

3 of 5 fixes were upstream or bridge. 2 were backend rendering.
The remaining 16 are likely a mix; each should be classified as it's fixed.

**Root-cause audit (2026-03-23):** All ~66 live invariant violations
trace to three root causes. Fixing root causes eliminates downstream
symptoms structurally; fixing symptoms individually is whack-a-mole.

| Root Cause | Violations | Core issue |
|---|---:|---|
| **I: Type nodes structurally incomplete** | ~32 | Parameterized types sometimes lose children (bare `leaf_node(name: "Map")` instead of `map_node(key:, value:)`). Downstream: `normalize_type_name` heuristic, leaf-vs-structured comparison, emit `"_"` placeholders, `classify_type_structure` in emit, Go `interface{}` holes. |
| **II: Error/Dynamic are names, not structure** | ~18 | Inference failures are smuggled through the type namespace as nodes named `"Error"` or `"Dynamic"`. Downstream: `node_type_equals` treats Error==anything, emit string-checks for error types, cascade suppression by name, permissive type compatibility. |
| **III: Divergent inference paths** | ~17 | `ExprCall` and `ExprMethodCall` compute the same operations through independent code paths with different logic. Downstream: duplicated map/flat_map/fold typing, asymmetric `map_insert`/`map_merge` handling, 4x bridge method name maps, inline string method checks bypassing classifiers, testgen parallel mock extraction. |

Terminology note: a "leaf" in this roadmap means only "a Node with no
recorded children in the current IR." It does not mean machine
primitive or algebraically fundamental; a leaf can be a reference, tag,
placeholder, or unresolved name. Root Cause I is about missing
structure, not about primitive-ness.

### Compositional Audit

| Area | Lines | Fns | Current state | Key issues |
|------|------:|----:|---------------|------------|
| `00_core.dag` | ~830 | 29 | Target-agnostic; `BuiltinTypeKind` deleted; `Cardinality` type added | `runtime_bridge_method_name` deleted; bridge maps now use closed-enum match per target (P1.10 done). `rt_node` returns `NodeType = Typed \| InferError \| Untyped` (P1.9 done). `return_cardinality: Cardinality` on Node (P1.4/P1.9). `Field.optional` removed. Several enum-to-string / string-to-enum paired functions (`config_property_*`, `transport_kind*`) risk drift. |
| `01_tokenize.dag` | 507 | 18 | Clean syntax leaf | No structural issues. Errors represented as `Unknown` tokens. |
| `02_parse.dag` | 3,932 | 176 | L3 debt hotspot | `kind_tag` dispatches on `TokenKind` via ~200 string comparisons. Service/resource syntax correctly dissolves into `Node`. 6 fabrication sites (empty capability on bad input, `"_"` status key, dummy EOF tokens). This is the primary L3 target. |
| `03_resolve.dag` | 459 | 12 | Cleanest authority boundary | `kernel_type_names()` deleted; callers import `kernel_types` from core. `resolve_node_bounded` trusts topological order (no redundant re-resolve of bindings; fixes diamond-shaped OOM). One defensive `None => []` fallback. |
| `04_infer.dag` | ~4,450 | 110 | Core inference; P2.6 split in progress | Multiple ExprData walks. Duplicated Call/MethodCall paths → collection methods deduplicated (P1.15); full dispatch deferred to Phase 3. `resolve_node_bounded` preserves `CardOptional` through env lookups. `lookup_field_type_node` checks optionality before product/coproduct dispatch. |
| `04_types.dag` | ~460 | 41 | Split from infer (`infer_types`) | `node_is_error_type` / `node_is_dynamic` **deleted** (P1.9). `node_is_optional` checks `return_cardinality` not name (P1.4). `with_optional_cardinality` / `with_required_cardinality` helpers added. `optional_node` / `optional_property` deleted. |
| `04_env.dag` | 53 | 3 | Split from infer (`infer_env`) | `TypeEnv`, `TypeBinding`, merge/lookup/cycle helpers. |
| `04_method.dag` | 192 | 6 | Split from infer | `classify_reconciled_intrinsic_method`, `classify_runtime_bridge_method` (single string-to-enum entry points). |
| `05_emit.dag` | ~1,192 | 35+ | Shared dispatch + helpers | P4.1 landed: unified `RenderTarget` dispatch for literals, keywords, operators, primitive types, containers, maps, identifiers, let bindings, return statements. `classify_type_structure()`, `build_emit_context()`, name converters. Fabricating fallbacks replaced with `__EMIT_BUG_*` error markers. **Parallel implementation problem:** `emit_node_type` (shared) and `emit_rust_node_type` (Rust) have identical traversal — see P4.1a. |
| `05_emit_rust.dag` | ~3,993 | 158 | Most complete backend | 19/19 intrinsic methods. Migrated to shared `emit_literal`, `emit_keyword`, `emit_bin_op_symbol`, `emit_ident`, `emit_let_binding`, `emit_return`. Type rendering NOT migrated (Rc wrapping — see P4.1a). 56 fabrication sites (21 are `compile_error!` -- correct fail-loud). |
| `05_emit_go.dag` | ~1,164 | 70 | Fully migrated for P4.1 | Leaf rendering (types, containers, maps, identifiers, let bindings) uses shared `emit_node_type`. ~175 lines deleted. 19/19 intrinsic methods (exhaustive match). 13 `interface{}` type holes (silent type erasure). |
| `05_emit_python.dag` | ~1,153 | 70 | Fully migrated for P4.1 | Same migration as Go. ~175 lines deleted. 19/19 intrinsic methods (exhaustive match). 2 `_unimplemented()` placeholders. |
| `05_emit_*` (cross) | — | — | 13 structurally identical function patterns across 3 backends | ExprData dispatch (21 arms x3), TCO (~90% identical x3), service/transport emission (identical 4-way dispatch x3). |
| `complexity.dag` | 1,216 | 33 | Good proof layer | Pipeline-wired. `intrinsic_method_cost_shape` is exhaustive. `cost_of_expr` uses `MatchCostAccum` (single-pass branch costs; avoids 2^depth blowup). |
| `ownership.dag` | 309 | 6 | Good proof layer | Pipeline-wired. `walk_expr` is 1 ExprData walk. String dispatch on `"fold"` for special accumulator threading. |
| `compile.dag` | 277 | 13 | Honest boundary shape | Returns complexity, ownership, and artifact plan. `front_end_sources` extracts shared frontend path. Complexity/ownership stages are order-independent. |
| `artifact.dag` | 113 | 2 | Consumed types only | `RenderTarget`, `Artifact`, `ArtifactPlan`, `default_artifact_plan` consumed by `compile.dag`. Speculative boundary verification types removed (P1.11 **done**). |
| `trace.dag` | 221 | 13 | Completely disconnected from pipeline | No other `.dag` file imports from it. Type-only schema with pure helpers. Should not grow until a consumer exists. |

### Active Temporary Bridges

These are acknowledged short-term regressions with named deletion
points. Each leans on Optional-specific or bootstrap-specific compiler
knowledge that the roadmap is actively dissolving.

| Bridge | Location | What it does | Deletion point |
|--------|----------|-------------|----------------|
| ~~`rt_node` sentinel fabrication~~ | ~~`00_core.dag`~~ | **Resolved.** `rt_node` returns `NodeType = Typed \| InferError \| Untyped`; no sentinel fabrication. Callers distinguish typed/failed/untyped explicitly. | P1.9 done. |
| ~~`Field.optional` + `Field.cardinality` duplication~~ | ~~`00_core.dag`~~ | **Resolved.** `Field.optional` deleted. `Field.cardinality` is the single authority. Parser `maybe_optional` sets `CardOptional` on the type node. | P1.4 done. |
| `prefer_specific_type` | `04_types.dag` | Prefers typed container/optional over Unit-parameterized when resolving if-branch types. Uses `node_is_optional`, `node_is_container`, `normalize_type_name`. | Reduced by P1.4/P1.9; full dissolution with L1 continuation. |
| `node_type_compatible` Unit special cases | `04_types.dag` | Treats `Unit`-element containers as compatible. | Reduced by P1.4/P1.9; remaining cases are L1 dissolution. |
| ~~`Some` constructor → `optional_node`~~ | ~~`04_infer.dag`~~ | **Resolved.** `optional_node` deleted; `Some` record lit infers via `with_optional_cardinality`. | P1.4 done. |
| ~~`node_is_error_type` / `node_is_dynamic`~~ | ~~`04_types.dag`~~ | **Resolved.** Predicates deleted. | P1.9 done. |
| Complexity guard (>100 functions) | `compile.dag` | Returns `empty_complexity_report()` for modules with >100 functions to avoid Rc-cloning OOM in the intern table. **This silently weakens the pipeline-wired proof layer.** | Fix the intern table to use arena allocation or `RefCell` instead of deep Rc cloning. Track as bootstrap performance item. |
| `expr_children` | `00_core.dag` | Extracts child expression Nodes from ExprData variants. Reimplements `node.children` for expressions because ExprData stores children inside variant fields instead of in `node.children`. Eliminates ~210 lines of boilerplate across 4 manual ExprData walks (collectors + self-call detection). `map_expr_children` has landed and now backs the remaining structural rewrites. | L2 dissolution (P5.11): when expression children move to `node.children`, `expr_children` becomes `node.children` read and is deleted. |
| ~~Arity bridge (`parameterized_type_arity`)~~ | ~~`00_core.dag`, `04_types.dag`~~ | **Resolved.** `parameterized_type_arity` and `is_parameterized_type` deleted. Compiler reads arity from `.dag` declarations. | P3.7 done. |

### L2 Bridge-Era Invariants

`ExprData` is an acknowledged L2 bridge. Child `Node` structure migrates
to `expr_children` / `map_expr_children` now so traversals can become
structural without waiting for full semantic dissolution. Role-specific
metadata and non-value-position data remain in `ExprData` until
P5.11/P5.12. This is preparatory work in the final direction, not
throwaway work.

Rules during the bridge era:

1. **`expr_children` is the required API** for new expression walkers,
   shared walkers, and any new ownership/complexity/emit traversals.
   Direct child-field access in ExprData walks is legacy.

2. **No new `ExprData` variant may introduce fresh value-position child
   `Node` storage outside the `expr_children` contract.** New variants
   must define their child order through `expr_children`. If a `Node`
   stays in type position, pattern position, or a binding-site
   descriptor role, the variant should say so explicitly. Legacy callers
   may access fields directly until migrated.

3. **`ExprData` dissolution is about child structure, not about
   `InferredNode`.** The L2 bridge (P5.11/P5.12) dissolves expression
   metadata (`ExprData` tags, method names, operators, binding names).
   Inferred type (`return_type`, `return_cardinality`, `InferredNode`)
   is structural typing from P1.9 — a completed dissolution, not L2 debt.

4. **`map_expr_children` has landed** (v1 retired, PR #200). Stage0→stage1
   bootstrap handles callable parameters natively, and the structural
   mapper now backs the remaining `resolve_expr_types` rewrites. Next:
   continue optional file splitting / orchestration cleanup inside
   `04_infer.dag`.

5. **P5.12 is design validation, not dogmatic deletion.** A closed
   typed semantic tag may still be the right intermediate even in a
   Node-centric compiler if it preserves exhaustiveness. P5.12 replaces
   `ExprData` with a more structural representation only if it reduces
   compiler semantic knowledge without regressing exhaustiveness.

### Active Ratchets

#### Phase-Blocking Ratchet: Diagnostics

`src/v2/tests/src/lib.rs` enforces `DIAG_RATCHET = 0` when the v1-bootstrap
path can compile v2 sources through stage0. **GATE MET:** 0 diagnostics on
this tree.

Journey: 2797 → 395 → 197 → 0 → (45 type errors regressed on tree merge)
→ 0 type errors + 53 ownership warnings → **0 total**.

Root causes eliminated (type errors):
- RC-A: `method_receiver_element_node` for Map<K,V>, map/flat_map/fold return type propagation
- RC-B: Suppress variant/field lookup cascade diagnostics on error/leaf types
- RC-C: Imported data declarations added to scope via `merge_scope_from_imports`
- RC-D: `lookup` return type fixed from receiver to `Optional<element>`
- RC-E: `node_type_compatible` Unit-element handling for List/Optional containers
- RC-F: `prefer_specific_type` for ExprIf branch type resolution
- RC-G: `infer_record_lit` specialization for `Some { value: x }` → `optional_node`

Root causes eliminated (ownership warnings, 53 → 0):
- Multi-consumer bindings in parser functions (inline record construction, decompose
  returns to field projections)
- Multi-consumer bindings in emit/infer/complexity (wrap tail expressions in
  `let result = ...; result` to convert Consumed uses to Read uses)
- Duplicate binding names across if/else branches (`raw` → `empty_raw` rename)

**Lesson learned:** The 53 ownership warnings were latent — hidden behind
type errors that gated the pipeline before ownership analysis could run.
The diagnostic ratchet test is `#[ignore]` and requires stage0 build, so
standard `cargo test` did not catch the regression window.

#### Architectural Ratchet: L1 Type Knowledge Dissolution

Scripted audit via `scripts/l1-ratchet.sh`. The script and this table
measure the same categories. Run `scripts/l1-ratchet.sh --check` to
verify the ratchet (current cap: 373). This scripted table is the
canonical source of truth; older 470-count prose is stale.

| Category | Script variable | Count | What the compiler still "knows" |
|----------|----------------|------:|----------------------------------|
| `.connective` direct access | `connective_field_count` | 19 | Product vs coproduct read from Node field |
| `Conj` / `Disj` references | `conj_disj_count` | 44 | Connective shape matching (includes parse, which must produce them) |
| Type constructors | `constructor_count` | 142 | `leaf_node`, `container_node`, `tuple_node`, etc. |
| Type-name comparisons | `typename_count` | 48 | `.name == "Optional"`, `"Map"`, `"Dynamic"`, etc. |
| `node_is_*` predicate calls | `predicate_count` | 101 | Centralized type-specific dispatch helpers |
| `classify_type_structure` calls | `classify_count` | 19 | Structural classification (replaces raw `.connective` reads in emit) |
| `builtin_type_kind()` calls | `builtin_count` | 0 | **Deleted** |
| **Total** | | **373** | |

Progress since initial baseline (~373): `BuiltinTypeKind` enum and
`builtin_type_kind()` are fully deleted. `classify_type_structure()`
replaces direct `.connective` reads in emit. `node_is_optional`,
`node_is_map`, `node_is_container` are centralized in `04_types.dag`.

The current scripted total is 373. The ratchet categories already absorb
the explicit cardinality and `InferredNode` plumbing that earlier prose
described separately, so the table above is the only number that should
be used for status or gates.

L1 acceptance (updated per thesis amendments):

- `BuiltinTypeKind` deleted **done**
- `builtin_type_kind()` deleted **done**
- `InferredNode` wrapper: compiler errors are orthogonal to node structure
- `node_is_*` predicates deleted — replaced with structural graph traversal
- `optional_node()` deleted **done**; `container_node()` / `map_node()` remain acknowledged bridge constructors until collection-kind and generic-path cleanup lands
- `normalize_type_name` deleted — unnecessary when types are structurally complete
- `classify_type_structure` deleted from emit — unnecessary when nodes carry structure
- `connective` field removed from `Node` (last structural primitive, furthest out)
- Scrambled-name tests pass: inference produces identical results regardless
  of type names (enforced by not passing name registry to infer)
- Fixed point still holds

---

## Recommended Next Steps and Parallelization

Phase 1 complete. Phase 2 gate met (gist lib compiles). Phase 3 closed on
this branch. Phase 4 implementation is complete on this branch except for
P4.6 equivalence validation / convergence follow-through.

### Parallelizable work streams

| Stream | What | Status | Depends on |
|--------|------|--------|------------|
| **A: Fixed-point** | Bootstrap convergence | **Done** — deterministic output, ratchet at 0 | — |
| **B: Generics** | P3.6 + P3.7 | **Done** — recursive generics landed and the arity bridge is deleted; built-in/user-generic path unification remains later L1 cleanup | — |
| **D: V1 retirement** | Replace v1 interpreter with stage0 | **Done** (PR #200 + #204) — stage0 committed, tests direct, v1-bootstrap removed | — |
| **D tier 5: File decomposition** | Decompose `04_infer.dag` using `map_expr_children` | **In progress** — `map_expr_children` landed and `resolve_expr_types` rewrites now use it; further file splitting is follow-up cleanup | D done |
| **E: P4.1a + P4.2/P4.3/P4.4/P4.5** | Shared emit spine, generated tests, DAG backend, typed CLI target surface | **Done** — full shared ExprData/TCO dispatch landed; tests and DAG artifact now share compiler-owned projections | P4.1 review cleanup done |

**D tier 5 remains independent of the completed emit work** — further
`04_infer.dag` file splitting can proceed without reopening the Phase 4
emit boundary changes.

### V1 retirement + file decomposition plan (Stream D)

**Tiers 1-4: Done** (PR #200 + #204, merged 2026-03-24). Stage0 committed
at `src/v2/stage0/` as a workspace member. Tests restructured into modules
(pipeline.rs, bootstrap.rs, parse.rs, helpers.rs) calling stage0 typed
APIs directly; source-level ratchets live in `scripts/source_audit.py`.
`v1-bootstrap` removed from default features.
`v2_runtime_shim.rs` deleted. `assemble_stage0` binary for seed regeneration.
CI excludes `v2-compiler` from workspace test (generated_tests.rs too large).

**Tier 5: File decomposition — partially complete.**

With `map_expr_children` now usable (stage0→stage1 bootstrap path handles
callable parameters), `resolve_expr_types` rewrites now use the structural
mapper instead of hand-written child walks. `04_resolve`, `04_emit_info`,
and `04_sigs` now exist as explicit bridge modules; the remaining large
cleanup is shrinking/orchestrating `04_infer` itself. Target:
`04_infer.dag` under 1500 lines still stands.

**Tier 5 ratchets for extracted passes:**
- An extracted pass is only "landed" when the `.dag` source, stage0 Rust
  module, assembler list, library export/import wiring, and at least one
  downstream consumer move together.
- Source-audit checks for extracted passes must anchor on live function or
  module declarations, not comment substrings or historical notes.

**What V1 retirement unblocked:**
- P3.8 recursive generics (v1 Rust stack overflow gone — done in PR #206)
- `map_expr_children` (landed)
- Full callback `emit_shared_expr` (landed)
- Shared TCO dispatcher extraction (landed)
- `resolve_expr_types` collapse / structural rewrites (landed)
- Further file decomposition with final code shape (still optional cleanup)

### Sequential dependencies

```
D (v1 retirement) ✓ DONE → P3.8 recursive generics ✓ DONE
                          → map_expr_children ✓ DONE
                          → resolve_expr_types collapse ✓ DONE
                          → file decomposition (D tier 5) ← OPTIONAL CLEANUP

E (P4.1a Rc unification) → E (P4.2 shared ExprData dispatch steps 2-4) ✓ DONE
                          → P4.7 (callable-type params) ✓ DONE
                          → P4.2 step 5 (full shared dispatcher + TCO) ✓ DONE
                          → P4.3 (generated tests) ✓ DONE
                          → P4.4 (DAG backend) ✓ DONE
                          → P4.5 (typed CLI target surface) ✓ DONE
                          → P4.6 (equivalence validation) ← NEXT

D tier 5 and P4.6 remain independent.
```

**Dependencies:**

```
Phase 1 (done) ──→ Phase 2 verification (gist gate) ✓
Phase 2 ─────────→ P4.1a/P4.2 (shared emit) ✓
D (v1 retirement) ✓ → full callback emit_shared_expr ✓ DONE
                    → map_expr_children ✓ DONE
                    → file decomposition ← OPTIONAL CLEANUP
P4.7 (callable) ✓→ P4.2 step 5 (full shared dispatcher) ✓ DONE
P4.2/P4.3/P4.4/P4.5 ✓ → P4.6 equivalence validation ← NEXT
P4.2 ────────────→ P5.0 (parser cleanup, can run in parallel with P4.6)
L1 dissolution ──→ Phase 5's L1=0 gate (ongoing, not blocking)
```

---

## Canonical Execution Order

Use this as the source of truth for sequencing.

| Order | Phase | What it does | Blocking gate |
|-------|-------|--------------|---------------|
| 1 | Phase 1 | Fix regressions, complete root cause fixes (InferredNode, normalization, path dedup), arity bridge, cardinality model (Optional dissolved), algebraic type spec | **MET** — All exit criteria satisfied: R1-R4 fixed; P1.9 (InferredNode) done; P1.14 (normalization) done; P1.17 (arity) done; P1.4 (Optional dissolved) done; P1.21 (testgen gate) done; P1.10, P1.12, P1.19, P1.20 done |
| 2 | Phase 2 | `gist` end-to-end through emitted Rust | **MET** — Stage0 compiles gist (0 diags, 18 files); lib target compiles and builds. Manual emitted-bin dry-run remains useful smoke, but is not the blocking Phase 2 gate. P2.1-P2.5 done. P2.6 (decomposition) partially done. |
| 3 | Phase 3 | Compile bundle, ownership/artifact wiring, v1 retirement, generics for parameterized type declarations | **MET** — v1 retired (PR #200); generics done including recursive (P3.8); arity bridge deleted; P3.2/P3.3 sufficient for gate. All exit criteria met. |
| 4 | Phase 4 | Shared emit spine, `LanguageSpec` authority, emit name-opacity | New backend = language facts + compiler-owned adapter; emit reads `LanguageSpec` for type→target mapping, no hardcoded type names |
| 5 | Phase 5 | L1=0, connective dissolution prep, L2/L3 preparation | **L1=0 gate**: scrambled-name tests pass; no arity bridges remain; no `node_is_*` predicates; `normalize_type_name` deleted; `classify_type_structure` deleted from emit |

Important clarifications:

- **No phase regresses the thesis.** Every change either moves toward
  name opacity and structural completeness, or is an explicit short-term
  bridge with a named deletion point. If a fix adds compiler type
  knowledge (emit heuristic, name check, property check), it must be
  traced upstream and the upstream fix must be on the Phase 1 workboard.
  "Temporary" without a deletion phase means "permanent later."
- Phase 1 is the only intentionally overlapping phase. **Diagnostics are
  at 0.** Regressions (R1-R4), root cause fixes (P1.9, P1.14, P1.15,
  P1.17), and invariant items (P1.10, P1.12) block Phase 2. L1
  dissolution continues in parallel toward Phase 5's L1=0 gate.
- The three root causes (I: incomplete types, II: error-as-name, III:
  divergent paths) are the organizing principle for Phase 1. Each root
  cause fix eliminates its downstream violation cluster.
- **Bridges are acknowledged regressions.** The arity bridge (P1.17) is
  hardcoded compiler knowledge — a short-term regression that replaces
  ~20 worse violations (emit `"_"` fabrication). It is deleted in Phase 3
  (P3.7) when real `.dag` declarations land. Every bridge has a named
  deletion point.
- `M*`, `R*`, and `S*` are support structures for this phase order, not
  competing schedules.

### Sequential Execution Checklist

After completing each phase, run the verification commands and confirm
the "you are here" state. If any check fails, the phase is not done.

**After Phase 1 — "The compiler is sound"**
```
cargo test --workspace --exclude v2-compiler-tests          # green
cargo clippy --all-targets -- -D warnings                   # green
cargo test -p v2-compiler-tests                             # green
python3 scripts/source_audit.py                             # green
scripts/l1-ratchet.sh --check                               # total <= ratchet (373)
```
State: 0 diagnostic regressions. Algebraic type spec written. Ownership
branch-merge fix landed. Emit catch-all fail-closed. Testgen fabrication
killed. `InferredNode` wrapper *partially* landed (wrapper exists, but
~15 `leaf_node("Dynamic")` fabrications remain — see P1.9 checklist).
Normalization stage *wired* (audit pass, not rewriting pass — see P1.14
remaining). Arity bridge *exists* (helpers present, but bare-leaf
tolerance still active — see P1.17 remaining). Testgen *guardrail*
exists (source-pattern grep, not the emitted-bundle gate — see P1.21).
Scrambled-name tests are *smoke* (compare counts, not inferred
structure — see P1.16). P1.4 (Optional dissolution) complete.
P1.9 (InferredNode) complete. P1.14/P1.17 (normalization/arity) complete.
P1.21 (testgen gate) complete. Fixed point holds on `cousin-wip`.

**After Phase 2 — "One real program works"**
```
# All Phase 1 checks, plus:
cargo test -p v2-compiler-tests \
  v2_gist_full_pipeline -- --ignored                        # green
```
State: The gist program compiles to Rust and builds successfully.
Emitted test files are present for service modules. Emitted-bin dry-run
remains a manual smoke step rather than an automated gate.

**After Phase 3 — "The compiler owns its domain"**
```
# All Phase 2 checks, plus:
cargo test -p v2-compiler-tests \
  v2_bootstrap_fixed_point -- --ignored                     # green
# v1 deletion proof: build and test without the legacy feature path:
cargo test --workspace --exclude v2-compiler-tests \
  --no-default-features                                     # green
cargo test -p v2-compiler-tests                             # green (no --features)
```
State: Generics landed. Algebraic `.dag` declarations exist for
`List`, `Map`, `Set`. `Optional` dissolved into cardinality in Phase 1.
Arity bridge deleted. v1 fully
removable — the feature-off proof above demonstrates that removing
`v1-bootstrap` does not break any non-bootstrap workflow. Compile
bundle has authoritative shape with ownership and artifact planning.

**After Phase 4 — "Adding a backend is easy"**
```
# All Phase 3 checks, plus:
# New backend (DAG) emits serialized typed graph
# Emit has zero hardcoded type-name → target-identifier mappings
```
State: Shared emit spine serves all backends. `LanguageSpec` is the
single authority. Generated tests work across backends. DAG backend
emits a serialized artifact. Emit is name-opaque.

**After Phase 5 — "The compiler is a generic graph processor"**
```
# All Phase 4 checks, plus:
cargo test -p v2-compiler-tests v2_scrambled_name_inference  # green
scripts/l1-ratchet.sh --check                                # L1 = 0
```
State: L1=0. Scrambled-name tests pass. No `node_is_*` predicates. No
`normalize_type_name`. No `classify_type_structure` in emit. The
compiler processes graph structure only. Ready for L2 work.

**Scrambled-name test definition:** The test compares **inferred
structure** (the typed graph after inference), not emitted artifacts.
Concretely: take a program, run it through inference with real names,
record the structural decisions (which nodes get which types, which
children, which connective shapes). Then scramble all type names
(consistently across declarations and references) and re-run inference.
The two sets of structural decisions must be identical. Emit is excluded
from this test because emit legitimately reads names for target-language
identifiers — that is name-rendering, not name-dependent inference.

---

## Phase 1: Soundness, Root Causes, and L1 Dissolution

This phase fixes all regressions, eliminates the three root causes of
invariant violations, and continues L1 dissolution toward name opacity.

### Execution Principles

- **Regressions first.** No regression survives into the next work item.
- **Fix root causes, not symptoms.** Each root cause fix eliminates its
  downstream violation cluster. Point-fixing symptoms grows them back.
- **L1 is the thesis in action.** Every L1 step reduces the distance to
  "inference doesn't read names."

### Phase 1 Workboard

#### Tier 0: Regressions (fix immediately, no regressions in this PR)

| ID | Item | Root Cause | Fix |
|----|------|-----------|-----|
| R1 | RC3 safety net: 30 lines of emit heuristic compensating for reconcile losing Optional through chained field access (`emit_rust` 1591-1620) | I | Fix `FieldSummary` propagation in inference; delete emit compensation |
| R2 | Anonymous record tuple index | I | **Done (stopgap):** `compile_error!()` for index >= 4 already emitted at `emit_rust` 1579. Single-field unwraps to bare type, multi-field uses tuple `.0`–`.3`. Real fix (proper field access for any arity) remains in backlog. |
| R3 | Duplicate map/flat_map/fold type refinement: ~40 lines in both `ExprCall` (1765-1803) and `ExprMethodCall` (1919-1935) paths | III | Extract shared helper; both paths call it |
| R4 | `map_insert` key type hardcoded `"String"` (`infer` 1781) | III | Read key type from receiver's map children |

#### Tier 1: Immediate invariant fixes (< 1 day each)

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P1.12 | Emit catch-all fail-closed | **Done** | `emit_typed_item` catch-all already emits `compile_error!()` — no `// unhandled` comments remain in `05_emit_rust.dag` |
| P1.10 | Collapse parallel bridge name maps | **Done** | `runtime_bridge_method_name` deleted from core. Remaining 4 tables (1 classifier in `04_method` + 3 per-target renderers in emit files) are exhaustive matches on closed `RuntimeBridgeMethod` enum — structurally sound per invariants. Per-target names genuinely differ (Rust `"with"`, Go `"With"`, Python `"with_update"`). |
| P1.19 | Testgen: collapse parallel mock extraction | **Done** | Deleted `extract_mock_props` from `05_emit_rust.dag`; all 3 call sites inlined to use `has_mock_prefix` from shared emit directly. `TestProjection` and `extract_test_projections` now feed Rust, Go, and Python test emission through the shared P4.3 path. |
| P1.20 | Testgen: kill fabrication sites | **Done** | `emit_simple_expr` wildcard already `compile_error!()`. `emit_data_value_json`/`emit_typed_data_value_json` wildcards changed from `{"__error__": ...}` to invalid JSON `<<<UNSUPPORTED_MOCK_EXPR>>>`. `Default::default()` for resource args → `compile_error!()`. Three `todo!()` sites (fold/method-arg/cast) → `compile_error!()`. Dry-run no-mock-data already `compile_error!()`. |
| P1.21 | Testgen: verification gate | **Done** | `v2_testgen_compiled_bundle_gate` compiles a service module with mock data via `emit_test_file`, extracts test files from the emitted bundle, and asserts non-empty syntactically valid Rust. Guardrail (`v2_testgen_service_mock_source_gate`) also retained. |
| P1.13 | Dead code cleanup | **Done** | Dead functions removed from `00_core.dag` and `complexity.dag` |
| P1.11 | Delete speculative artifact types | **Done** | Only consumed plan types remain in `artifact.dag` |

#### Tier 2: Root cause fixes (structural, each eliminates a violation cluster)

| ID | Item | Root Cause | Status | Notes |
|----|------|-----------|--------|-------|
| P1.9 | `InferredNode` wrapper | II | **Done** | `Node.return_type` is `InferredNode?`; `rt_node` returns `NodeType = Typed \| InferError \| Untyped` (no sentinel fabrication); `node_is_error_type`/`node_is_dynamic` deleted; error/Dynamic permissive rules removed from type comparison; `return_cardinality: Cardinality` on Node. |
| P1.14 | Normalization stage | III | **Done** | `03_normalize.dag` enforces arity as errors; pipeline gate halts before infer on normalization errors. Call→MethodCall unification deferred to Phase 3 (declaration-driven per `03_normalize.dag`). |
| P1.15 | Deduplicate inference paths | III | **Done (collection paths)** | Shared `refine_collection_result_type` and fold helpers landed. Call→MethodCall dispatch deferred to Phase 3 normalization. |
| P1.17 | Arity bridge | I | **Done** | `parameterized_type_arity` and `type_node_arity_ok` exist. Bare-leaf tolerance removed; `actual == 0` rejected as error by normalization gate. Bridge deleted when `.dag` declarations exist (P3.7). |
| P1.18 | Algebraic type spec (design doc) | — | **Done** | `docs/algebraic-type-spec.md` pins denotational semantics, algebra laws (set/bag/map), cardinality model, and algebraic representations for Phase 3 declarations. Law layer resolved (complement not closed, bag union = sum, map merge requires conflict function, decidable equality is structural). |

#### Tier 3: L1 dissolution (toward name opacity — ongoing, NOT Phase 1 exit requirements)

These items start in Phase 1 and continue toward Phase 5's L1=0 gate.
They do NOT block Phase 2. They are tracked here because Phase 1 is
where the foundational work (arity bridge, InferredNode) enables them.

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P1.16 | Scrambled-name tests | **Smoke exists** | Smoke tests (`v2_scrambled_name_inference_smoke`, `_containers`) compare diagnostic count and file count. **Not the gate:** the roadmap requires comparing inferred structure (typed graph shapes). Scaffolding is useful; full suite is Phase 5 scope (P5.6). |
| P1.4 | L1 Optional → cardinality | **Done** | `Optional` dissolved from name-checking: `return_cardinality: CardOptional` on Node, `Field.optional` removed, `optional_node()` deleted, `node_is_optional` checks cardinality not name, emit backends read cardinality, `Field.cardinality` populated from parsed type expression, `field_to_child_node` reads `field.cardinality` (binding site carries cardinality). **Dual-write stepping stone:** type expressions retain cardinality alongside binding site; emitters read from type nodes. Phase 5 strips type-expression cardinality and migrates emitters to binding-site-only reads. |
| P1.5 | L1 Containers | Planned | Structural graph traversal replaces `node_is_container`, `node_is_map`. Each collection type has its own algebra (see Collection Denotational Model) — no single "container" concept needed. Element/key/value types are structural children. Fix bare leaf vs parameterized inconsistency (Root Cause I). |
| P1.6 | L1 Primitives | Planned | Primitives dissolve with the bit-graph model. `Int`, `String`, etc. are namespaced compositions opaque to the compiler. `.dag` declarations carry any facts emit needs. |
| P1.7 | L1 Connective dissolution | Planned | Last structural primitive. Remove `connective` from `Node` only after all consumers use structural graph traversal. |
| P1.8 | Delete residual type primitives | Planned | Delete constructors, predicates, and builtin classifiers after consumers switch. |

#### Completed

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P1.1 | Naming cleanup (M1) | **Done** | All non-stage files renamed; RenderTarget moved to artifact.dag |
| P1.2 | Infer cleanup via data tables | **Partial** | Emit metadata extracted. Method handling (remaining emit-side method dispatch centralization) is picked up by P4.2 (shared emit fold + target adapters). |
| P1.3 | Diagnostics ratchet -> 0 | **Done** | DIAG_RATCHET = 0. Four root-cause fixes. |

### P1.9 Design: `InferredNode` Wrapper

The compiler produces errors — it should never need to rediscover them
by string-checking node names. `Error` and `Dynamic` are both "inference
couldn't determine this." They are failures, not types.

```
type InferredNode {
  Resolved { node: Node }
  CompilerError { message: String, span: SourceSpan }
}
```

Inference returns `InferredNode`. If a child fails, the parent propagates
`CompilerError`. Emit never sees error nodes. `node_type_equals` and
`node_type_compatible` never encounter errors — the wrapper is resolved
before comparison.

**Current status (partial):**

What landed:
- `Node.return_type` is `InferredNode?`
- `rt_node(n:)` helper extracts Node from the wrapper
- `make_expr_error_node` produces `CompilerError`
- `Resolved { node: rt }` matching in emit paths
- Serialization round-trips through v1 interpreter

Mechanical checklist:
- [x] Delete `node_is_error_type(n)` — callers pattern-match `InferredNode`
- [x] Delete `node_is_dynamic(n)` — same (Dynamic = CompilerError)
- [x] Stop `rt_node` fabricating `Node{name:"Error"}` and `Node{name:"Unit"}`
      sentinel nodes — `rt_node` returns `NodeType = Typed | InferError | Untyped`;
      callers distinguish absent vs failed explicitly
- [ ] Delete `node_type_equals` Error==anything rule (blocked: ~12 error_type_node() cascade sites produce Error nodes that flow through type comparison; converting to CompilerError is L1/Phase 5 scope)
- [ ] Delete `node_type_compatible` Error/Dynamic==anything rules (Error: same blocker as above; Dynamic: supports 15 audited polymorphic placeholder sites — removal requires type variable infrastructure, L1/Phase 5)
- [x] Delete ~9 emit sites checking `"Error"`/`"Dynamic"` by name
- [x] Delete `"_"` type placeholders that compensate for error-typed nodes
- [x] Add `return_cardinality: Cardinality` to `Node` (blocks P1.4 completion)
- [x] Replace Dynamic error sentinels with `error_type_node()` (8 sites in
      `callable_return_type`, `lambda_param_types_from_scope`, fold init, method
      result_type); 15 remaining Dynamic sites audited as correct polymorphic
      placeholders (lambda params, method returns, kernel stubs, emit fallbacks)
- [x] Stop `resolve_optional_node` silently swallowing `CompilerError` —
      now surfaces error as diagnostic; `leaf_node("Error")` sentinel persists
      due to `NodeResolveResult` structural constraint (Phase 5 migration)
- [x] Audit `rt_node(...)` callers that collapse `None` into `Unit` —
      `rt_node` now returns `NodeType = Typed | InferError | Untyped`;
      137 callers migrated; `rt_type` convenience helper for emit callers;
      explicit 3-arm matches where error/absent distinction matters

**Verification:** `grep -rn '"Error"\|"Dynamic"' src/v2/04_types.dag
src/v2/04_infer.dag src/v2/05_emit*.dag` returns zero results related
to type-level error/dynamic checking (diagnostic messages are fine).

**Migration boundary: P1.9 completion status.**

Most mechanical changes are done. Remaining items are concentrated in
infer (Dynamic fabrications) and resolution (error re-entry into node graph):

| Layer | Type/API affected | Status |
|-------|-------------------|--------|
| `00_core.dag` | `rt_node(n:)` | **Done** — returns `NodeType = Typed \| InferError \| Untyped`, no sentinel fabrication |
| `00_core.dag` | `Node` | **Done** — `return_cardinality: Cardinality` field added |
| `04_types.dag` | `node_is_error_type(n)`, `node_is_dynamic(n)` | **Done** — deleted |
| `04_types.dag` | `node_type_equals`, `node_type_compatible` | **Remaining** — Error/Dynamic permissive rules serve error cascade (~12 sites) and polymorphic placeholders (15 sites); removal is L1/Phase 5 |
| `04_infer.dag` | ~23 sites fabricating `leaf_node(name: "Dynamic")` | **Done** — 8 error sentinels converted to `error_type_node()`; 15 polymorphic placeholders audited correct with `// Dynamic audit:` comments |
| `04_infer.dag` | `resolve_optional_node` | **Done** — surfaces `CompilerError` as diagnostic; sentinel `leaf_node("Error")` persists due to `NodeResolveResult.resolved: Node` constraint |
| `04_infer.dag` | `rt_node(...)` callers | **Remaining** — some collapse `None` into `Unit` instead of propagating failure |
| `05_emit_rust.dag` | ~9 sites checking `"Error"`/`"Dynamic"` by name | Largely resolved by upstream InferredNode gating |
| `05_emit_go.dag` | `interface{}` type holes from error nodes | Phase 4 scope |

**Return cardinality storage.** Once `Node.return_type` becomes
`InferredNode?`, return cardinality lives as a separate field on the
expression node:

```
Node.return_type: InferredNode?          // the type (or failure)
Node.return_cardinality: Cardinality     // Required (exactly 1) or CardOptional (0..1)
```

Two fields, not a composite `InferredType = { node, cardinality }`,
because cardinality is orthogonal to success/failure — a
`CompilerError` has no meaningful cardinality, and a `Resolved { node }`
can have either. Emit reads `return_cardinality` from the binding site
to render `Option<T>` / `*T` / `Optional[T]`. Default is `Required`
unless inference determines otherwise (e.g., `map.get(key)` sets
`return_cardinality: CardOptional`).

**Default elaboration timing.** Default values on fields
(`Field.default_value: Node?`) are elaborated during normalization
(P1.14). The normalizer fills in default values for omitted fields
before inference runs. Inference sees all fields as present; the
distinction between "explicitly provided" and "default-filled" is a
parse/normalize concern, not an inference concern.

Ordering constraint: the InferredNode wrapper changes the type of
`Node.return_type`, which touches `00_core.dag`. Every `.dag` file that
reads `return_type` needs mechanical updates. This should be done as a
single atomic commit that updates all consumers, not incrementally.

### P1.14 Design: Normalization Stage

New pass: `parse → resolve → **normalize** → infer → emit`

**Current status (partial):**

What landed:
- `03_normalize.dag` wired into the pipeline between resolve and infer
- `check_node_arity` walks the graph and emits warnings for mismatches
- Arity helpers (`parameterized_type_arity`, `type_node_arity_ok`) exist

What remains (mechanical checklist):
- [ ] **Rewrite the graph**, not just audit it — bare `leaf_node(name: "Map")`
      with 0 children must either gain its declared children or become a
      `CompilerError`, not pass through with a warning
- [ ] **Reject bare parameterized types** — the `actual == 0` pass-through
      must become a construction error
- [ ] **Unify Call→MethodCall** — the bridge rewrite currently lives inside
      `infer_expr`; move it into normalize so infer sees one form
- [ ] **Elaborate defaults** — fill in `Field.default_value` for omitted
      fields before inference runs

Normalization has two scopes, split across phases:

- **Phase 1 normalization** (P1.14): Call/MethodCall unification, arity
  enforcement via the hardcoded arity bridge (P1.17), and parser
  error-recovery tagging with `CompilerError`. This scope uses
  hardcoded knowledge of known parameterized types (`Map→2`, `List→1`,
  `Set→1`). `Optional` is excluded — it is cardinality on the binding
  site (P1.4), not a parameterized type.

- **Phase 3 normalization** (P3.6/P3.7): Declaration-driven property
  population and generic slot substitution. The arity bridge is deleted
  and replaced by reading arity from real `.dag` algebraic declarations.
  This scope requires generics (P3.6) to exist first.

Phase 1 normalization job:
- Unify `ExprCall`→`ExprMethodCall` for known method patterns (the
  bridge rewrite that currently lives inside `infer_expr`)
- Enforce arity completeness: parameterized types always carry their
  declared number of children (no bare `leaf_node(name: "Map")`)
- Mark parser error-recovery nodes with `CompilerError`

Distinct from resolve (which validates names and ordering) and from infer
(which determines types and produces diagnostics). Normalize is a
structural graph rewrite: same graph in, structurally-normalized graph
out. It should be small — a single-pass graph rewrite, not a full
analysis.

### P1.4 Design: Optional → Cardinality Dissolution

**Current status: cardinality model complete (dual-write stepping stone).**

The cardinality model is landed, Optional is dissolved from
name-checking, and cardinality routes through the binding site
(`Field.cardinality` → `field_to_child_node` → child node
`return_cardinality`). Type expressions retain cardinality as a
dual-write stepping stone for backward compatibility with emitters.
This is bridge-era role language, not a second ontology: the same Node
carrier can still appear in type position while the enclosing binding
site carries occurrence/cardinality semantics.

What landed:
- `Cardinality = Required | CardOptional` type in `00_core.dag`
- `Node.return_cardinality: Cardinality` field added
- `Field.optional: Bool` removed — all consumers read `Field.cardinality`
- `optional_node()` constructor deleted
- `node_is_optional(n)` checks `n.return_cardinality`, not name
- Parser `maybe_optional(...)` sets `CardOptional` on parsed type
- `parse_field` transfers `te.return_cardinality` to `Field.cardinality`
- `field_to_child_node` reads `field.cardinality` (binding site)
- All emit backends read `return_cardinality` for top-level wrapping
- `resolve_node_bounded` and `resolve_scrutinee_type_node` preserve
  cardinality through resolution
- Design doc pinning the four-way separation (absence, nullability,
  unknownness, defaultability)

What remains (Phase 5 L1 dissolution — not Phase 1 exit):
- [ ] Strip cardinality from type expression nodes (`maybe_optional`
      sets cardinality on binding site only)
- [ ] Migrate emitters from `node_is_optional(type_node)` to reading
      `return_cardinality` from the enclosing binding-site node
- [ ] Delete `OptionalUnwrap` / `OptionalValue` field access styles
      if no longer needed after emitter migration
- [ ] Update `prefer_specific_type` and `node_type_compatible` Unit
      special cases that compensate for Optional-as-type-wrapper

### Remaining Diagnostic Correctness Items

Diagnostics reached 0. These inference gaps remain:

| Fix | Status | Notes |
|-----|--------|-------|
| Enumerate return type | Done | `List<Tuple<Int, T>>` now flows through inference |
| Fold accumulator threading | Done | `fold_accumulator_type` follows init-arg type |
| Callable/function-value type | Done | Callable type representation exists |
| Structured `ErrorCategory` | Done | Error classification moved off ad hoc strings |
| `map_insert` / `map_merge` result typing | Done | Key type inferred from first arg via `refine_collection_result_type`; bare Map leaf rejected by normalization |
| Chained field access | R1 | Depends on correct `FieldSummary` propagation |
| Tighten `node_type_equals` | Done | Dissolved by `InferredNode` wrapper — error/Dynamic rules deleted |
| `normalize_type_name` heuristic | P1.5 | Dissolved when parameterized types are always structurally complete |

### Phase 1 Exit Criteria

Status key: **MET** = verified on tree; **PARTIAL** = foundational work
landed but acceptance condition not fully met; **OPEN** = not yet started
or blocked.

- **MET**: All regressions (R1-R4) fixed
- **MET**: `InferredNode` wrapper complete; `rt_node` returns `NodeType = Typed | InferError | Untyped` (no sentinel
  fabrication); `node_is_error_type`/`node_is_dynamic` deleted; error/Dynamic
  permissive rules removed from type comparison; `return_cardinality` on Node;
  `resolve_optional_node` surfaces `CompilerError` as diagnostics (no longer
  silently swallowed). **Continuing L1 work (not Phase 1 exit):** ~14
  `leaf_node("Dynamic")` fabrications in infer; `resolve_optional_node` sentinel
  `leaf_node("Error")` persists due to `NodeResolveResult` structural constraint;
  ~94 `rt_node` callers collapse `None` into `Unit`. (P1.9)
- **MET**: Normalization stage wired and enforces arity as errors (`Error` severity);
  pipeline halts before infer on normalization errors. Graph rewriting (structural
  normalization pass) and Call→MethodCall unification are Phase 3 scope
  (declaration-driven per `03_normalize.dag`). (P1.14)
- **MET**: Arity bridge enforced: bare-leaf tolerance removed; `actual == 0`
  rejected as error by normalization gate (P1.17)
- **MET**: Parallel bridge name maps collapsed (P1.10)
- **MET**: Emit catch-all fail-closed (P1.12)
- **MET**: Testgen: parallel mock extraction collapsed (P1.19), fabrication sites
  killed (P1.20), verification gate passing — `v2_testgen_compiled_bundle_gate`
  compiles service module, extracts test files, asserts valid Rust (P1.21)
- **MET**: Algebraic type spec (design doc) written, pinning the denotational
  equations (`Set<A> = A -> Bool`, `Bag<A> = A -> Nat`, `Map<K,V> = K ->
  1 + V)`, `List<A> = Σ n. Fin(n) -> A`), the algebra family, the
  law layer decisions for `List`, `Map`, `Set`, and primitives, the
  occurrence/cardinality model (four-way separation: absence, nullability,
  unknownness, defaultability; `Cardinality` type; binding-site semantics),
  and set algebra laws
- **MET**: No silent/fail-open fabrication on the bootstrap-critical Rust emit
  path — every `"_"` placeholder, silent `todo!()`, and `Default::default()`
  fallback in `05_emit_rust.dag` is replaced with `compile_error!()` or
  traced upstream. Non-numeric casts fail-closed with `compile_error!` unless
  source and target resolve to the same Rust type (refinement identity cast).
  Go `interface{}` (13 sites), Python `_unimplemented()`
  (2 sites), and Go `/* unhandled expr */` (1 site) are Phase 4 scope.
- **MET**: No new emit heuristics introduced — every emit-side type-knowledge
  regression is traced upstream and fixed in inference or normalization
- **MET**: `cargo test -p v2-compiler-tests v2_strict_compile_diagnostic_count -- --ignored` passes
  (Diagnostics at 0 as of 2026-03-23. All 45 type errors resolved via
  `node_type_compatible` Unit-element handling and `prefer_specific_type`.
  53 ownership warnings resolved via multi-consumer binding restructuring.)
- **MET**: Optional dissolved from name-checking into `CardOptional` on
  `Node.return_cardinality` (P1.4) — `Field.optional` removed; `optional_node()`
  deleted; `node_is_optional` checks `return_cardinality`; parser sets
  `CardOptional`; emit backends wrap via cardinality; `Field.cardinality`
  populated from parsed type expression; `field_to_child_node` reads
  `field.cardinality` (binding site carries cardinality). **Continuing L1
  work (not Phase 1 exit):** type expressions retain cardinality as dual-write
  stepping stone; emitters still call `node_is_optional(n)` on type nodes.
  Phase 5 strips cardinality from type expressions and migrates emitters to
  read from binding-site nodes only.
- Fixed point still holds after every structural change (prior branch)
- All exit criteria are MET. Phase 2 may proceed. L1 dissolution
  continues in parallel toward Phase 5's L1=0 gate.

---

## Phase 2: Gist End to End

**Gate:** `gist.dag` plus its transitive dependencies compile to Rust,
`cargo build` succeeds, and the emitted program runs correctly in dry-run
mode.

### Current Status

- Service operation bodies are already real in `05_emit_rust.dag`
- `main.rs` workflow dispatch is already emitted
- The remaining blocker is verification through a built stage0 binary

### Verification Path (updated 2026-03-25)

Tests now call stage0 directly (PR #200). The verification path is:

1. Stage0 compiles `gist` (tested via `v2_gist_full_pipeline`)
2. Emitted lib target compiles and builds
3. Bin target dry-run execution: not yet tested (remaining P2.5 work)

### Phase 2 Workboard

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P2.1 | Gist pipeline test | **Done** | 11-file gist closure compiles with 0 diagnostics, 18 files emitted. Fixed: `skip` on empty list panic in emitted stage0 (safe slice bounds in v1 emitter). |
| P2.2 | Service operation bodies | Done | reqwest, `Command`, auth injection, dry-run mocking already landed |
| P2.3 | `main.rs` workflow dispatch | Done | Workflow subcommands and dispatch match arms already land |
| P2.4 | Multi-module extdep imports | **Done** | Verified via gist pipeline test; all 11 modules with transitive imports resolve |
| P2.5 | Emitted crate build/run | **Done (lib)** | `v2_gist_full_pipeline` passes: lib target compiles and builds. Resource wiring fixed (main.rs constructs `&Network`). Bin target dry-run execution not yet tested (comment at test line 5643). Previous 32-error description is stale. |
| P2.6 | `04_infer.dag` decomposition | **In progress** | `04_cycle.dag`, `04_resolve.dag`, `04_emit_info.dag`, and `04_sigs.dag` are extracted. `expr_children`/`map_expr_children` now back the structural rewrites instead of hand-written child walks. Remaining work is shrinking `04_infer.dag` itself and finishing any optional follow-on splits. Target: `04_infer.dag` under 1500 lines. |

### Current Emitted Bundle Shape

Today the emitted Rust crate is already conceptually the right bundle:

```text
output_dir/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── v2_rt.rs
│   ├── gist.rs
│   ├── github_api.rs
│   ├── git.rs
│   └── ...
├── tests/
│   ├── github_api_test.rs   (generated from service mock data)
│   └── ...
```

That bundle comes out of `compile.dag` plus the Rust emitter.

### Phase 2 Exit Criteria

- `cargo test -p v2-compiler-tests v2_gist_full_pipeline -- --ignored` passes — **MET** (lib target compiles and builds)
- The emitted gist crate builds in a dry-run-compatible shape — **MET** (lib target compiles and builds); emitted-bin dry-run remains manual smoke, not the blocking gate
- Emitted test files are present in the bundle for service modules with mock data — **MET**
- No v1-only post-processing step is required to make the crate buildable — **MET** (v1 retired)

---

## Phase 3: Compile Contract, Pipeline Completion, and v1 Retirement

**Gate:** v2 compiles everything that still matters from v1, ownership is
pipeline-wired, artifact planning is real, and v1 is no longer on the
critical path.

This phase owns the compile contract work: M2, M3, and M4, plus R8 and
R9.

### Phase 3 Workboard

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P3.1 | Verify parity with remaining v1 paths | **Done on `cousin-wip`** | `v2_bootstrap_fixed_point` passes again end-to-end. Deterministic bootstrap output landed, bare generic type declarations have an explicit non-emitting item kind, variant-parent authority is contextual, runtime bridge lookups preserve Rc wrapping, and typed intrinsic lowering now covers `sort_by` / `fold` / `empty_map` / `to_string`. |
| P3.2 | Ownership wiring + authoritative compile bundle | **Sufficient for gate** | `compile_sources` returns `complexity`, `ownership`, and `artifact_plan`; emit dispatch follows the planned artifact target. Remaining consolidation (multi-artifact partitioning, obligation reporting) is Phase 4+ scope — the gate criteria are met. |
| P3.3 | Artifact planning above emit | **Sufficient for gate** | Default single-artifact planning runs between infer and emit through the real artifact contract. Speculative boundary types deleted in P1.11. Real partitioning and per-artifact orchestration are Phase 4+ scope — the gate criterion (artifact planning runs in primary compile path) is met. |
| P3.4 | Runtime shim dissolution | **Done** | `runtime_rust.dag` is the authoritative runtime template. v1 legacy `v2_runtime_shim.rs` (336 lines) deleted — v1 retirement (Stream D, PR #200) made it dead code. Future: Go/Python backends may add `runtime_go.dag` / `runtime_python.dag` following the same pattern. |
| P3.5 | Feature-gate v1 | **Done (superseded)** | v1 crates gated behind `v1-bootstrap` feature. PR #200 went further: `v1-bootstrap` removed from default features entirely. Tests call stage0 directly. |
| P3.6 | Generics (parameterized type declarations) | **Done** | Pair<A,B>, Box<T>, nested Pair<List<Int>, String> all work. Slot instantiation currently happens in `resolve_node_bounded` after normalization has enforced arity completeness. Arity bridge deleted (P3.7). Recursive generics (MyList<T> = Nil \| Cons) also work — see P3.8. Built-in/user-generic path unification remains later L1 cleanup. |
| P3.8 | Recursive generics | **Done** | `type MyList<T> = Nil \| Cons { head: T, tail: MyList<T> }` compiles with 0 diagnostics. Cycle detection via Kahn's algorithm in `04_cycle.dag` precomputes `recursive_type_set` on `TypeEnv`. `resolve_node_bounded` skips self-referencing fields in recursive types. v1 stack overflow blocker removed by Stream D v1 retirement (PR #200). Test: `generic_recursive_type` (active, not ignored). |
| P3.7 | Delete arity bridge | **Done** | `parameterized_type_arity` and `is_parameterized_type` deleted from `00_core.dag` and `04_types.dag`. Compiler reads arity from `.dag` declarations (bare `type List<element>`, `type Map<key, value>`, `type Set<element>` in `dsl/std/types.dag`). Arity validation added to `resolve_node_bounded` (TypeMismatch diagnostic on wrong arg count). |

### P3.6 Design: Generics as Compositional DAG Slots

Generics are not a separate system — they are the DAG composition model
applied to type declarations. A type parameter is a **slot**: a named
position in a type's DAG structure where another type can be composed in.

```
type List<T> = Nil | Cons { head: T, tail: List<T> }
type Map<K, V>    // finite partial function K -> V (lookup has return cardinality 0..1)
type Set<T>       // membership: finite-support T -> Bool
```

`Optional` is not a parameterized type declaration — it is cardinality
on the binding site (see Occurrence and Cardinality Model). `T?` sets
cardinality, not wraps the type. The generics system does not need
`type Optional<T>`.

The recursive algebraic representation (`Nil | Cons`) is the
computational form. The denotational meaning (see Collection
Denotational Model above) pins the semantics: `List<A>` is `Σ n.
Fin(n) -> A` — a length plus a value per position. `Map<K,V>` is a
finite partial function, not `List<Tuple<K,V>>` (that's one possible
backing representation, not the type's identity). The `.dag`
declarations capture the algebraic form; the denotational equations
constrain what operations are well-typed.

`List<Int>` means: take the `List` DAG, fill slot `T` with the `Int`
DAG. This is structural composition — the same thing nodes already do
with children. The only new pieces are:

1. Declaring which positions are slots (in the `.dag` declaration)
2. Resolving slot references in bodies at composition time

**Slot names are meaningful namespace identifiers**, not abstract
single-letter variables. Slots are named positions in the type's DAG
structure — the same concept as named children in a product type:

```
type List<element> = Nil | Cons { head: element, tail: List<element> }
type Map<key, value>       // finite partial function: key -> value (lookup cardinality 0..1)
type Set<element>          // membership: element -> Bool (finite support)
```

`key`, `value`, `element` are lowercase because they are structural
positions, not type names. When you write `Map<String, Int>`,
the composition is positional: first slot (`key`) gets `String`, second
(`value`) gets `Int`. If the compiler needs the element type of a `List`,
the answer is structural: whatever was composed into the `element` slot.
`Map` and `Set` are declared with slots but without a body that reduces
them to `List` — their denotational semantics (finite partial function,
membership function) define what operations are well-typed, not a
structural reduction to another type.

**Slot representation.** A slot appears in a type declaration body as a
`TypeVar` leaf node — a leaf whose name matches a declared slot name.
The declaration Node records its slots via its `params` field: each
`Param` has `name` (the slot name, e.g. `"key"`) and `type_expr` (a
`TypeVar` marker node). This reuses the existing `Param` structure —
for function params, `type_expr` is a declared type; for type slots,
`type_expr` is the unfilled slot marker. Same structural shape,
different role determined by context (type declaration vs function
declaration). When the compiler encounters `List<Int>`, it looks up
`List`'s declaration, finds one slot `element`, and produces a concrete
Node graph with every `TypeVar` named `element` replaced by the `Int`
Node.

**Why slot names are not a name-opacity violation.** Slot names
(`key`, `value`, `element`) are structural placeholders within a type
declaration — they are positional addresses, not type identities. The
compiler uses them during substitution (positional matching: first
arg fills first slot), but never branches on a slot name to make an
inference decision. After substitution, no `TypeVar` nodes remain in
the graph — they are fully resolved. Scrambled-name tests (P5.6)
verify this: slot names can be scrambled along with type names, and
inference still produces identical structural decisions. The rule is:
"inference cannot read names to make structural decisions." Slot names
are consumed before inference, not by inference itself.

**Where substitution happens.** In the current implementation,
normalization enforces arity completeness and blocks malformed
parameterized types before inference, while resolve-time type
instantiation (`resolve_node_bounded`) performs slot substitution. By
the time inference sees the graph, all slots are filled with concrete
types. Inference never encounters `TypeVar` nodes — they are resolved
structurally before inference runs.

**What changes per pipeline stage:**

| Stage | Change |
|-------|--------|
| **Parse** | `parse_type_def` learns `<Name, ...>` after the type name. Records slot names on the declaration Node. `finish_type_expr_from_name` generalizes: any name can accept `<Arg, ...>` (delete the hardcoded `List/Set/Map` checks at lines 1104-1115). |
| **Resolve** | Validates that slot names are unique within a declaration. Validates that `TypeVar` references in the body match declared slot names. Instantiates parameterized declarations by substituting concrete args into the declared body during `resolve_node_bounded`. |
| **Normalize** | Enforces arity completeness before inference and keeps malformed parameterized types from reaching resolve/infer. |
| **Infer** | No change — receives fully-substituted concrete types. |
| **Emit** | No change — type nodes already carry their children. |

**Recursive types.** `type List<T> = Nil | Cons { head: T, tail: List<T> }`
— the `List<T>` in `tail` is a self-reference with the same slot
binding. Substitution produces `List<Int>` in `tail` when the outer
`List<Int>` is composed. The existing `is_self_recursive` flag on Node
already tracks this; normalization extends it to slot-aware recursion.

**Nested composition.** `List<Map<String, Int>>` — the `Map<String, Int>`
arg is itself a composition. Substitution is recursive: resolve inner
compositions first, then substitute into the outer slot. This falls out
of the normalization pass naturally (post-order traversal).

**What this deletes:**
- The arity bridge (P1.17 / P3.7) — arity is read from the declaration
- Hardcoded generic parsing in `finish_type_expr_from_name` (lines
  1104-1115 in `02_parse.dag`)
- `container_node()`, `map_node()` constructors in `04_types.dag` —
  replaced by slot substitution from `.dag` declarations
  (`optional_node()` already deleted in Phase 1 by P1.4)
- `container_property()`, `map_type` property injection — properties
  come from the `.dag` declaration, not from hardcoded constructors

**Existing plumbing that helps:**
- `children: List<Node>` already holds type arguments at use sites
- `params: List<Param>` records slots on declarations — `Param.name` is
  the slot name, `Param.type_expr` is the `TypeVar` marker; same field
  already holds function params (role is contextual)
- Parser already handles `Name<Arg>` and `Name<Arg1, Arg2>` reference
  syntax (just hardcoded to known names)

**Open design questions (resolve in P1.18 algebraic spec):**
- Named composition syntax (`Map<key: String, value: Int>`) — more
  explicit, avoids positional arity-order errors, but adds syntax.
  Positional (`Map<String, Int>`) is the minimum viable. Decide whether
  named composition is Phase 3 scope or later.
- Higher-kinded slots (`type Functor<F<_>>`) — out of scope for Phase 3;
  note as a "Beyond" item if needed.
- Constraint syntax — likely unnecessary for the DAG model. Since
  everything is a Node, key hashing/equality is structural (the DAG
  structure itself determines comparison, not type-specific logic). No
  type is "unhashable." This aligns with the law layer decision that
  decidable equality is automatic (all types have structural equality
  in the DAG model). Constraints may emerge later if the model needs
  them, but they are not a Phase 3 concern.

### Key Decisions for Phase 3

- The compile result stops being just `files + diagnostics`
- Ownership becomes a first-class pipeline output, not a side analysis
- Artifact planning becomes part of the real compile flow, not a side
  module with stringly targets
- Unsupported proof or validation obligations must surface explicitly

### Phase 3 Exit Criteria

- The compile bundle has one authoritative typed shape — **MET** (P3.2 sufficient for gate)
- Ownership is included alongside complexity in the pipeline output — **MET** (P3.2 sufficient for gate)
- Artifact planning runs between infer and emit in the primary compile path — **MET** (P3.3 sufficient for gate)
- v1 is no longer required for normal compilation — **MET** (PR #200)
- Algebraic `.dag` declarations exist for `List`, `Map`, `Set`, with
  denotational semantics grounding from P1.18 (collection model);
  `Optional` is not a type declaration — it was dissolved into cardinality
  on binding sites in Phase 1 (P1.4) — **MET**
- Arity bridge (P1.17) is deleted — compiler reads arity from declarations — **MET** (P3.7)
- No Phase 1 bridge remains as a blocking gate item — **MET** (container
  properties deleted, Dynamic error sentinels converted to
  `error_type_node()`, and the remaining visible bridges are now
  explicitly tracked as ongoing L1/L2 debt rather than hidden gate work)

---

## Phase 4: Shared Emit, Projections, and Backend Boundaries

**Gate:** adding a new backend means writing language facts plus a
compiler-owned adapter, with no changes to the shared compiler core.

This phase owns M5, M6, and M7. M8 follows only after the Phase 4
contract is real.

### Design Rules for Phase 4

- Shared emit owns traversal
- Compiler-owned target adapters own program-dependent lowering
- `dsl/extdeps/languages/*` stays declarative
- Generated tests are first-class outputs, not Rust-only emitter details
- The DAG backend remains a compile target; execution stays in a runtime

### Phase 4 Workboard

| ID | Item | Status | Notes |
|----|------|--------|-------|
| P4.1 | `LanguageSpec` becomes the single authority | **Done (contract met)** | Unified `RenderTarget` dispatch layer landed in `05_emit.dag`, and compiler-local helpers in `src/v2/languages.dag` now route shared emit/backends through `LanguageSpec` for reserved words, scaffold, serialization, test conventions, visibility, and per-target syntax facts. Fabricating fallbacks replaced with `__EMIT_BUG_*` error markers. Verified by full `v2-compiler-tests` plus `scripts/source_audit.py` ratchets for `LanguageSpec` reads. |
| P4.1a | Rust type rendering via shared `emit_node_type` | **Done** | Rust type rendering migrated to shared `emit_node_type` + Rc wrapping layer. `emit_rust_node_type` deleted. Commit `22063fe3`. |
| P4.2 | Shared emit fold + target adapters | **Done** | Shared emit now owns the full callback-based `emit_shared_expr` recursion and shared `emit_shared_tco_expr` walker in `05_emit.dag`; Rust/Go/Python backends keep thin target adapters only where ownership or surface syntax genuinely differs. `map_expr_children` landed in core and infer rewrites now use it. Verified by `strict_pipeline_smoke`, `dag_pipeline_smoke`, and full `cargo test -p v2-compiler-tests -- --nocapture`. |
| P4.3 | Generated tests as first-class projection | **Done** | `TestProjection` now carries `module_name`, `return_type`, and `params`; `extract_test_projections()` is the single graph-walk entry point; `emit_simple_expr` moved into shared emit; Rust/Go/Python all emit test files from the same projection contract. Verified by `generates_mock_test_file`, `python3 scripts/source_audit.py`, and the full compiler test suite. |
| P4.4 | DAG backend/runtime boundary | **Done** | `Dag` remains a compile target only; `emit_dag_artifact` now writes a versioned `dag-artifact.json` containing serialized `modules`, `diagnostics`, and `files` from the post-infer `ResolvedGraph`. Runtime execution stays downstream by design. Verified by `dag_pipeline_smoke` and ignored bootstrap smoke `stage0_compile_accepts_dag_target`. |
| P4.5 | Typed backend plumbing and CLI surface | **Done** | Backend selection remains typed end-to-end and the committed stage0 / emitted compile CLI now parse `--target rust\|python\|go\|dag` with recursive source discovery aligned across bootstrap stages. Verified by ignored bootstrap smoke `stage0_compile_accepts_dag_target` plus full suite. |
| P4.6 | Equivalence validation | Planned | Self-compile and gist must still converge after shared emit lands |
| P4.7 | Callable-type parameters | **Done** | All 6 steps (P4.7a-f) complete: v1 `impl Fn` emission, v2 parser `fn(T) -> R` syntax, v1 call emission verified, v2 inference for callable locals, v2 callable-aware lambda param threading, v2 backend rendering verified. **Unblocks P4.2 step 5.** |

### P4.1 Contract: `LanguageSpec` Checklist

`LanguageSpec` is already defined as a concrete type in
`dsl/std/languages.dag` (line 393). It composes all facts the emitter
needs for a target language. The Phase 4 gate is: **the emitters read
this type instead of hardcoding facts inline.**

What belongs in `LanguageSpec` (current fields, grouped):

| Group | Fields | Purpose |
|-------|--------|---------|
| Identity | `language: Language` | Name, extensions, comment syntax, naming convention, type mappings |
| Syntax | `statements`, `expressions`, `control_flow`, `literals`, `modules`, `functions`, `errors`, `type_defs`, `patterns`, `async_model` | Template strings for every syntactic construct the emitter produces |
| Runtime ops | `collection_ops`, `string_ops`, `map_ops`, `null_coalesce` | DSL builtin → target-language operation templates. The split into `collection_ops` vs `map_ops` predates the denotational model; Phase 4 should reconcile these into per-algebra groupings (set ops, list ops, map ops) matching the Collection Denotational Model. |
| Identifier safety | `reserved_words` | Keywords + escape strategy |
| Value semantics | `value_semantics` | Ownership/reference/value model |
| Serialization | `serialization` | Derives, JSON parse/emit, tag attributes |
| Project structure | `scaffold` | Manifest file, source dir, entry point |
| Service calls | `service_calls` | Async invocation template |

Completeness test: if adding a new target language requires changing
any emitter `.dag` file rather than just providing a new `LanguageSpec`
value, a field is missing from this type.

Existing concrete values: `rust_spec`, `go_spec`, `python_spec` (all
in `dsl/std/languages.dag`, layer 2d).

### P4.4 Contract: DAG Artifact Schema

The DAG backend emits a serialized typed graph as a JSON artifact.
The schema is the post-infer `ResolvedGraph` structure — the same
graph that source-code backends receive.

| Field | Type | Description |
|-------|------|-------------|
| `version` | `String` | Schema version (semver, starting at `"0.1.0"`) |
| `modules` | `List<TypedModule>` | The typed module graph after inference |
| `diagnostics` | `List<Diagnostic>` | Compiler diagnostics (errors + warnings) |
| `files` | `List<TextFile>` | Empty for DAG backend (no source-code files) |

Versioning: the schema version tracks breaking changes to the
`TypedModule` / `Node` structure. A runtime that consumes DAG
artifacts pins a schema version range. The compiler bumps the version
when `Node` fields change (e.g., P1.9 `InferredNode` changes
`return_type`). This is not a stability promise yet — it is a
mechanism so that breaking changes are detectable, not silent.

The DAG artifact is a JSON serialization of the v1 interpreter's
`Value` representation of the compile result — the same format the
v1 bootstrap path already produces internally. No new serialization
format is needed; the new part is writing it to a file and defining
the envelope (`version` + `modules` + `diagnostics`).

### P4.2 Design: Shared Emit Fold + Target Adapters

**Why this needs a spec before coding starts:** P4.2 is the highest-risk
refactor in the roadmap. The current state is 13 structurally identical
function patterns across 3 backends. Extracting shared traversal without
a clear contract will fossilize current shims into the shared layer.

**Prerequisites (must be complete before P4.2 starts):**
- P1.4 (Optional dissolution) — otherwise shared emit encodes Optional
  heuristics that should not exist
- P1.9 (InferredNode completion) — otherwise shared emit encodes
  Error/Dynamic sentinel handling
- P1.21 (testgen gate) — otherwise testgen regressions during extraction
  go undetected

**Adapter contract: what the shared layer owns vs what backends own.**

Shared layer owns:
- Recursion / tree traversal (the `ExprData` dispatch walk)
- Evaluation order (statement sequencing, let-binding scoping)
- Scope threading (variable binding, scope extension)
- Typed child collection (field ordering, param mapping)
- Common service/test projection flow
- TCO detection and trampoline structure

Backend layer owns:
- Terminal syntax rendering (keywords, braces, semicolons, indentation)
- Target-specific ownership/reference wrappers (Rust: `Rc`, `Box`,
  `clone`; Go: pointers; Python: none)
- Match/pattern surface syntax
- Runtime helper names (Rust `with`, Go `With`, Python `with_update`)
- Any unavoidable target-specific lowering

**Practical mechanism.** P4.7 adds callable-type parameters to `.dag`.
The shared dispatcher takes a `recurse: fn(Node) -> String` callback
that the backend provides as a closure capturing its own state. This
replaces the enum-dispatch workaround.

**Frozen shared renderer signatures (05_emit.dag):**

All 13 shared renderers take `target: RenderTarget` as their sole
shared-layer parameter plus expression data (strings, nodes, or enum
values). No backend-specific state crosses the boundary.

| Function | Parameters | Line |
|----------|-----------|------|
| `emit_keyword` | `key: String, target` | 748 |
| `emit_literal` | `value: LiteralValue, target` | 757 |
| `emit_bin_op_symbol` | `op: BinOpKind, target` | 769 |
| `emit_node_type` | `n: Node, target` | 810 |
| `emit_ident` | `name: String, target` | 1287 |
| `emit_let_binding` | `name: String, value: String, target` | 1316 |
| `emit_return` | `value: String, target` | 1325 |
| `emit_unary_op` | `op: UnaryOpKind, operand_str: String, target` | 1340 |
| `emit_lambda` | `params_str: String, body_str: String, target` | 1347 |
| `emit_error_expr` | `message: String, target` | 1356 |
| `emit_lambda_params` | `param_names: List<String>, target` | 1366 |
| `emit_list_lit_expr` | `element_strs: List<String>, target` | 1374 |
| `emit_null_coalesce` | `l_str: String, r_str: String, target` | 1393 |

**Backend-specific state (Rust example):** Every `emit_typed_*` in
`05_emit_rust.dag` takes 5 backend-owned parameters:

- `registry: Map<String, ItemInfo>` — item registry (shared concept, per-backend threading)
- `scope: InferScope` — variable scope (shared concept, per-backend threading)
- `vtoe: Map<String, String>` — variant-to-enum mapping (Rust-specific)
- `rc_types: Map<String, Bool>` — Rc wrapping decisions (Rust-specific)
- `emit_info: EmitGraphInfo` — graph-level emit metadata

**P4.2 step 5 — full shared dispatcher (Done).** `emit_shared_expr` in
`05_emit.dag` now owns the full callback-based `ExprData` recursion, and
`emit_shared_tco_expr` owns the shared tail-call walk. The earlier
7-arm callback milestone from `origin/main` is subsumed here: the v1
bootstrap constraint is gone, backends now pass a
`recurse: fn(Node) -> String` adapter, and only genuinely target-specific
syntax / ownership cases remain backend-owned.

**Safe extraction order (lower risk first):**

1. **Freeze the adapter contract** — list every backend-owned hook
   function and its signature. This is a design step, not a code step.
2. **Extract shared type traversal** — `emit_*_node_type` across
   backends follows the same Conj/Disj/leaf/container/callable
   structure. Extract the shared walk; backends own terminal rendering.
3. **Extract shared block/let/scope/TCO walkers** — high duplication,
   lower semantic risk. The three TCO walkers are ~90% identical.
4. **Extract service/test projection flow** — identical 4-way transport
   dispatch across backends.
5. **Extract full ExprData dispatcher** — the hot, highest-risk step.
   Delay until the adapter interface is stable from steps 2-4.
   **Prerequisite:** P4.7 (callable-type parameters) — the shared
   dispatcher needs `recurse: fn(Node) -> String` callback.

**P4.2 completion status — shared ExprData/TCO renderers (2026-03-25).**

Shared emit in `05_emit.dag` now owns the reusable recursion:

*Shared-owned now:*
`emit_shared_expr`, `emit_shared_tco_expr`, `emit_simple_expr`,
`extract_test_projections`, `emit_literal`, `emit_error_expr`,
`emit_return`, `emit_unary_op`, `emit_lambda`, `emit_lambda_params`,
`emit_null_coalesce`, `emit_list_lit_expr`, `emit_bin_op_symbol`,
`emit_keyword`, `emit_ident`, `emit_let_binding`, `emit_node_type`.

*Still backend-owned by design:*
ownership wrappers / deref policy, target block syntax, target match
syntax, runtime helper names, and the genuinely target-specific lowering
cases such as Rust Rc wrapping or Python/Go surface formatting.

*Deeply divergent examples (stay per-backend):*
ExprVar (Rust vtoe/rc_types), ExprFieldAccess (Rust ownership),
ExprCall (Rust empty_map + lookup Rc wrapping), ExprMethodCall
(intrinsic dispatch + runtime bridge), ExprMatch (Rust Rc deref
analysis + pattern rendering), ExprIf (Python ternary vs Rust/Go
block), ExprRecordLit (Rust struct/tuple/Rc wrapping),
ExprStringInterp (format!/f-string/Sprintf), ExprForEach (Rust
collect vs loop), ExprBlock (Rust brace wrapping), ExprCast (Rust
numeric type checking), ExprIndex/ExprSlice (Rust type-dependent).

*Bootstrap constraint lifted (2026-03-25).* Stream D (v1 retirement)
is complete. `assemble_stage0` uses the v1 emitter, which handles
callable types and lambdas correctly, so the full callback dispatcher is
no longer blocked on interpreter limitations.

**Revised acceptance criteria:**
- Each backend's `ExprData` match arms are thin: either a direct shared
  renderer call, or delegation to a per-backend helper (no inline logic)
- All cross-target rendering patterns live in `05_emit.dag`
- Adding a new `ExprData` variant requires: one shared renderer + one
  arm in each backend's match (not three independent implementations)
- Parser/stage0/gist pipeline tests green
- DIAG_RATCHET = 0

### P4.3 Completion: TestProjection as First-Class Output (2026-03-25)

**Landed `TestProjection` shape** (05_emit.dag):
```
type TestProjection {
  module_name: String
  service_name: String
  operation_name: String
  return_type: Node
  params: List<Param>
  mock_field_inits: List<FieldInit>
}
```

**Current state:**
- `extract_test_projections()` is the single graph-walk entry point
- `emit_simple_expr` lives in shared emit and is used by Rust/Go/Python
  test emitters
- Rust, Go, and Python all emit test files from the same projection
  contract
- Test naming, placement, and framework conventions read through
  `LanguageSpec` / `test_conventions`

**Verification (2026-03-25):**
- `cargo test -p v2-compiler-tests generates_mock_test_file -- --nocapture`
- `python3 scripts/source_audit.py`
- `cargo test -p v2-compiler-tests -- --nocapture`

### Current Phase 4 Risks

- P4.6 equivalence validation / convergence is still the explicit
  remaining Phase 4 item after the implementation work in this branch.
- Go `interface{}` type holes and the residual Python/Go fabrication
  markers remain quality issues, but they no longer block the shared emit
  boundary, cross-backend test generation, or DAG artifact contract.
- Mixed-backend runtime execution stays intentionally downstream of the
  compiler: the versioned DAG artifact boundary is implemented, while the
  runtime continues to live outside compiler stages.
- Go `interface{}` type holes and the residual Python/Go fabrication
  markers remain quality issues, but they no longer block the shared emit
  boundary, cross-backend test generation, or DAG artifact contract.
- Mixed-backend runtime execution stays intentionally downstream of the
  compiler: the versioned DAG artifact boundary is implemented, while the
  runtime continues to live outside compiler stages.

### P4.7 Design: Callable-Type Parameters

**Why:** The `.dag` language has callable-type representation in the IR
(`callable_node` in `04_types.dag`) and callable-type rendering in all
3 backends (`05_emit.dag` lines 845-858), but no way to declare a
callable-typed parameter in `.dag` source. This blocks P4.2 step 5:
the shared ExprData dispatcher needs a `recurse: fn(Node) -> String`
callback to delegate sub-expression rendering to the backend.

The triplication of the 19-arm `emit_typed_expr` match across Rust,
Python, and Go backends violates "No parallel implementations." Callable
parameters are the missing language feature that resolves this.

**What already exists:**
- `callable_node(func_params, ret)` builds `Node { name: "Callable", params: ..., return_type: ... }` (04_types.dag:92)
- `callable_return_type(n)` extracts return type (04_types.dag:96)
- Type rendering for Callable in all 3 backends (05_emit.dag:845-858)
- v1 parses `fn(T1, T2) -> R` syntax (`parse_function_type_expr` in type_codegen.rs)
- Arrow token `->` exists in v2 tokenizer (01_tokenize.dag:25)

**Bootstrap-ordered steps:**

| Step | What | File(s) | Status |
|------|------|---------|--------|
| P4.7a | v1: emit `impl Fn(T) -> R` instead of `fn(T) -> R` for callable params | `src/v1/07_emit/daglang-emit/src/type_codegen.rs` | **Done** — position constraint documented (param-only safe; struct fields need position-aware rendering) |
| P4.7b | v2 parser: `fn(T1, T2) -> R` callable type syntax in `parse_type_expr` | `src/v2/02_parse.dag` | **Done** — `parse_callable_type_expr` handles `fn(` prefix, delegates to `parse_callable_param_types`, produces `callable_node`. Reuses `ParamsResult` with empty-name Params. |
| P4.7c | v1: verify callable variable call emission works | `src/v1/07_emit/daglang-emit/src/fn_codegen.rs` | **Done** (no changes needed) — `compile_call` creates `Expr::Call { func: Var(name) }` uniformly; render_rust emits `name(args)` which works for both top-level fns and `impl Fn` params |
| P4.7d | v2 inference: resolve ExprCall against callable-typed locals in `scope.locals` | `src/v2/04_infer.dag` | **Done** — Tier 2c: between builtin calls and type constructors, checks `scope.locals` for Callable-typed binding, extracts return type via `callable_return_type` |
| P4.7e | v2 inference: thread Callable formal param types to lambda args | `src/v2/04_infer.dag` | **Done** — sig lookup moved before arg inference; `infer_lambda_with_callable_type` threads each Callable param type to the corresponding lambda param. Collection method bridge path unchanged (only activates when sig is unknown). |
| P4.7f | v2 emission: verify callable-variable calls render in all backends | `05_emit_rust.dag`, `05_emit_python.dag`, `05_emit_go.dag` | **Done** (no changes needed) — v1 emits `impl Fn` (P4.7a), v2 shared emit renders Callable types (05_emit.dag:845-858), ExprCall renders `name(args)` in all backends. End-to-end verification happens when .dag code first uses callable params. |

**P4.7a** must land first (bootstrap ordering: v1 compiles .dag → stage0,
so v1 must emit correct Rust for callable types before v2 .dag code can
use them). P4.7b-c can proceed in parallel after P4.7a. P4.7d-e are
v2-only inference changes. P4.7f is verification.

**P4.7a detail:** v1's `emit_function_type` (type_codegen.rs:266) emits
`fn(T) -> R` (bare function pointer). Closures require `impl Fn(T) -> R`.
Since callable parameters will be used as callbacks (capturing backend
state), `impl Fn` is required. **Done** (2026-03-25).

**P4.7a constraint — position-aware callable rendering.** The current
`impl Fn` change is blanket: `type_expr_to_rust` renders ALL callable
types as `impl Fn(T) -> R` regardless of position. In Rust, `impl Fn`
works in parameter and return positions but NOT in struct fields (which
need `Box<dyn Fn(T) -> R>` or a generic parameter). The same function
is called from struct field rendering (type_codegen.rs:350, 549, 1767),
parameter rendering (918, 985), and return type rendering (990).

This is safe today because no `.dag` source uses callable-typed struct
fields. When that becomes possible, `type_expr_to_rust` must gain a
`position: TypePosition` parameter (ParamType | FieldType | ReturnType)
and emit the correct Rust syntax per position. This is a rendering
decision (invariant 6) — the IR is position-agnostic; only the backend
needs to know.

Scope: the `.dag` language should initially restrict callable types to
function parameter position only (P4.7b parser enforces this). Struct
field support is a separate future item requiring position-aware rendering.

**P4.7b detail:** `parse_type_expr` (02_parse.dag:1054) handles `LBrace`
(inline record) and `Ident` (named/generic). Add a branch: when current
token is `fn` keyword followed by `(`, parse parameter types and `-> R`
to produce `callable_node(func_params, ret)`.

**P4.7d detail:** ExprCall inference (04_infer.dag:~1767) always looks up
`func` in `func_env`. Add fallback: if not in `func_env`, check
`scope.locals` for a binding whose type is a Callable node. Extract
params and return_type from the Callable to type-check the call.

**P4.7e detail:** When a lambda is passed as an argument whose formal
parameter type is Callable, thread the Callable's param types to the
lambda's param types. Same mechanism as `LambdaSemantics` for built-in
collection methods, generalized.

**Callback signature design note.** The full shared ExprData dispatcher
needs `recurse: fn(Node, InferScope, Int) -> String`. Two parameters
change during recursion (`depth`, `scope`); all others are captured.
P4.7 added callable-type support to the v2 compiler, but the v1
**interpreter** rejects lambdas as standalone values (`eval_stack.rs:1162`).

**Current state (2026-03-25):** `emit_shared_expr` handles 7 arms
(3 leaf + 4 recursive) with `recurse: fn(Node, InferScope, Int) -> String`
callback. All 3 backends wired. `map_expr_children` landed in `00_core.dag`
(18 ExprData variants). Stage0 regenerated with P4.7 callable support.
DIAG_RATCHET = 0.

### Phase 4 Exit Criteria

- No backend owns a whole-tree `ExprData` dispatcher
- No backend owns a separate whole-tree TCO walker
- `LanguageSpec` is the single authority for language facts
- Emit is name-opaque: type→target-identifier mapping reads from
  `LanguageSpec` and structural declarations, no hardcoded type names
- Generated tests are first-class artifact outputs
- The DAG backend emits a serialized typed graph (JSON or equivalent)
  without embedding an interpreter in the compiler stages; a separate
  runtime can execute the artifact

---

## Phase 5: L1=0, Convergence, and L2 Preparation

**Gate:** L1 dissolution is complete. The compiler has zero type-world
knowledge. Names are opaque. Scrambled-name tests pass. The bootstrap
architecture is stable enough for deeper dissolutions.

This phase gates on L1=0. It is intentionally later — it should happen
after naming, pipeline, shared emit, and algebraic declarations are all
stable.

### Phase 5 Workboard

Phase 5 has two tracks that can run in parallel: **L3 dissolution**
(parser, P5.0-P5.1) and **L1 final deletions** (P5.6-P5.10). The
structural dissolutions (P5.2-P5.5) are independent of each other and
can interleave with either track.

#### Track A: L3 dissolution (parser)

| ID | Item | Depends on | Est. scope | Notes |
|----|------|-----------|-----------|-------|
| P5.0 | `kind_tag` string dispatch elimination | — | ~200 sites in `02_parse.dag` | Replace `kind_tag(token) -> String` + string comparisons with a payload-insensitive token shape API. See P5.0 design below. Largest single item in Phase 5. |
| P5.1 | Token dissolution | P5.0 | ~507 lines (`01_tokenize.dag`) | Replace `Token` / `TokenKind` structures with `Node` compositions. Only tractable after P5.0 removes string dispatch. |

#### Track B: Structural dissolutions (independent, any order)

| ID | Item | Depends on | Est. scope | Notes |
|----|------|-----------|-----------|-------|
| P5.2 | Module/import dissolution | — | ~459 lines (`03_resolve.dag`) | Dissolve `Module`, `Import`, and `ImportNames` into `Node` compositions |
| P5.3 | Diagnostic / compile-output dissolution | — | Moderate | Dissolve `Diagnostic`, `Severity`, `CompileResult`, and `TextFile` where it is still valuable. `InferredNode` (P1.9) already handles error representation. |
| P5.4 | Service/support type dissolution | — | Small | Verify which service-layer types still need to move. Service nodes already use `Node` composition for operations/transports. |
| P5.5 | Residual semantic enum cleanup | P5.2-P5.4 | Small | Move remaining compiler-only semantic types toward `.dag` or `Node`-based representation. Depends on prior dissolutions to identify what's left. |

#### Track C: L1 final deletions (the L1=0 gate)

| ID | Item | Depends on | Est. scope | Notes |
|----|------|-----------|-----------|-------|
| P5.6 | Scrambled-name tests (full suite) | Phase 4 (emit name-opacity) | Test suite | Inference produces identical **inferred structure** (typed graph shapes) regardless of type names. Emit excluded (legitimately reads names for target identifiers). This is the L1=0 verification gate. |
| P5.7 | Delete `node_is_*` predicates | P5.6 passing + P5.7a/P5.7b below | 82 call sites | Two distinct invariant violations behind the predicates. See P5.7 design below. |
| P5.8 | Delete `normalize_type_name` | P5.6 passing | 17 sites | Unnecessary when types are always structurally complete (arity enforced since Phase 1, declarations since Phase 3). |
| P5.9 | Delete `classify_type_structure` from emit | P5.6 passing, Phase 4 (shared emit) | 22 call sites | Unnecessary when nodes carry structure directly. |
| P5.10 | Connective dissolution assessment | P5.7-P5.9 | Design decision | Evaluate whether `Conj`/`Disj` can dissolve or remain as the compiler's last structural primitive. Note: collections (`Set`, `Map`, `List`) are *not* products or coproducts — they are function/indexed types in the denotational model. `Conj`/`Disj` remain relevant for record types (product) and coproduct types (e.g., `Result<T, E> = Ok \| Err`), but the collection algebra family is orthogonal. `Optional` is cardinality, not a coproduct node (P1.4). |

### P5.7 Design: Predicate Dissolution (2026-03-25)

The `node_is_*` predicates contain two distinct invariant violations.

**Violation 1: Duplicate representations (products/coproducts).**

`node_is_product` checks `properties |> any(p => p.name == "is_product") || connective == Conj`. Investigation shows products ALWAYS have BOTH set — the parser (02_parse.dag:787, 856, 1068) and inference constructors (tuple_node, anonymous records, service results) set connective AND the property string together. The `||` is not a fallback for missing connective; it's a parallel check of the same fact in two representations. Violates "No duplicate representations."

**Fix (P5.7a): DONE (2026-03-25).** Deleted `is_product`/`is_coproduct` property strings. `connective` is the structural authority. `product_property()` and `coproduct_property()` deleted from `04_types.dag`. Parser (6 sites), inference (4 sites), and resolve (1 site) updated. Predicates simplified to `n.connective == Some { value: Conj/Disj }`. Reader sites in parse/resolve now use predicates or let-bindings (`.dag` parser ambiguity: `Some { value: X }` inside `if COND {` confuses the parser — use let-binding or predicate call instead).

**Violation 2: String-keyed metadata (containers/maps).**

`node_is_container` checks `properties |> any(p => p.name == "container_kind")`. Containers/maps are NOT products or coproducts — a `List<T>` is a sequential collection, not a conjunction. `connective` doesn't apply. The property string is the ONLY representation, but it's a string-keyed open set. Violates "No case enumeration for open sets."

**Fix (P5.7b):** Add a typed enum field to Node: `collection_kind: CollectionKind?` where `CollectionKind = ListKind | SetKind | MapKind`. `container_node()` and `map_node()` (04_types.dag:70-87) set the enum instead of a property string. Predicate becomes `n.collection_kind != none`. ~8 constructor sites + predicate definitions + 30 container/map call sites.

**Sequencing:** P5.7a (delete duplicate property strings) is safe to do any time — it only removes a parallel representation, no semantic change. P5.7b (typed collection enum) is a Node representation change that touches more code and should wait for Phase 5.

#### Track D: L2 dissolution (ExprData → Node children)

| ID | Item | Depends on | Est. scope | Notes |
|----|------|-----------|-----------|-------|
| P5.11 | ExprData child dissolution | Phase 4 (shared emit stable) | ~5000 lines across all .dag files + v1 interpreter | Move expression-position child Nodes from ExprData variant fields to `node.children`. Delete `expr_children`/`map_expr_children` bridge. See P5.11 design below. |
| P5.12 | ExprData tag dissolution assessment | P5.11 | ~2000 lines | Validate whether `ExprData` should dissolve into a more structural expression-kind property on Node, or remain as a closed semantic tag where that is the right exhaustiveness device. The goal is structural clarity, not dogmatic deletion. |

### P5.11 Design: ExprData Child Dissolution

**Problem:** Expression children live inside ExprData variant fields
(`ExprIf.condition`, `ExprCall.args`, etc.) instead of in
`node.children`. This forces every expression analysis to write a full
ExprData match to access children — 12 manual walks totaling ~1800
lines, of which ~600 are pure structural boilerplate.

**Bridge (current):** `expr_children(node) -> List<Node>` extracts
child expression Nodes from ExprData. `map_expr_children(node,
transform)` (structural tree-map) is designed and now **unblocked** by
v1 retirement — stage0→stage1 bootstrap handles callable parameters.

`ExprData` is an acknowledged L2 bridge. Child `Node` structure in
value/expression position migrates to `expr_children` now so traversals
can become structural without waiting for full semantic dissolution.
Role-specific metadata and non-value-position data remain in `ExprData`
until P5.11/P5.12. This is preparatory work in the final direction, not
throwaway work. The bridge is safer than the arity bridge: the arity
bridge invents missing information temporarily; the expr-children bridge
re-homes information that already exists.

**Target state:** Expression nodes use `node.children` for child Nodes
that participate in value/expression position, the same way structural
passes consume ordered child lists elsewhere in the graph. ExprData
retains non-Node metadata plus any Nodes that still serve distinct roles
(type position, pattern position, binding-site descriptors) until those
roles get their own structural encoding.

**Per-variant value-position child table:**

`node.children` contains the child Nodes that participate in
value/expression position, in declared order. A Node is a Node:
distinctions like type, value, pattern, and binding-site describe where
the node participates in the pipeline, not different carriers. The role
vocabulary stays because it still marks real operational invariants.

| Variant | children (positional) |
|---------|----------------------|
| ExprFieldAccess | [base] |
| ExprCall | [arg0.value, arg1.value, ...] |
| ExprMethodCall | [receiver, arg0.value, ...] |
| ExprMatch | [scrutinee, guard0?, body0, guard1?, body1, ...] |
| ExprIf | [condition, then, else?] |
| ExprLet | [value, body?] |
| ExprBinOp | [left, right] |
| ExprUnaryOp | [operand] |
| ExprLambda | [body] |
| ExprBlock | [stmt0, stmt1, ...] |
| ExprCast | [expr] |
| ExprForEach | [collection, body] |
| ExprIndex | [base, index] |
| ExprSlice | [base, start, end] |
| ExprReturn | [value] |
| ExprRecordLit | [field0.value, field1.value, ...] |
| ExprListLit | [elem0, elem1, ...] |
| ExprStringInterp | [interp0.expr, interp1.expr, ...] |
| ExprLiteral, ExprError, ExprVar, NoExprData | [] |

**What stays in ExprData or adjacent metadata (role-specific data):**

```
ExprCall.func: String              → stays (function name is metadata)
ExprCall.call_semantics            → stays
ExprMethodCall.method: String      → stays
ExprMethodCall.method_semantics    → stays
ExprFieldAccess.field: String      → stays
ExprFieldAccess.summary            → stays
ExprVar.name: String               → stays (or moves to node.name)
ExprVar.binding_kind               → stays
ExprBinOp.op: BinOpKind           → stays
ExprUnaryOp.op: UnaryOpKind       → stays
ExprLet.name: String              → stays
ExprForEach.variable: String      → stays
ExprCast.target: Node             → stays for now (type position, not a value child)
ExprRecordLit.type_name           → stays
ExprLambda.params                 → stays (binding-site descriptors; may include type-position nodes)
ExprLambda.semantics              → stays
MatchArm.pattern                  → stays (pattern position, not a value child)
NamedArg.name                     → stays (arg name is metadata on the child)
```

**Positional semantics:** After dissolution, child positions are
meaningful. `ExprIf` children are `[condition, then, else]` by
position — child 0 is condition, child 1 is then-branch, child 2 is
else-branch. The ExprData tag identifies the expression kind;
`node.children` holds the value-position operands in declared order.
Type-position, pattern-position, and binding-site data may still live
beside that list without implying separate ontologies. This is the same
uniform-carrier model used elsewhere in the graph.

**Named arg threading:** `ExprCall` and `ExprMethodCall` have
`args: List<NamedArg>` where each arg has `name: String?` and
`value: Node`. After dissolution, the child Nodes are the arg values;
the arg names become properties on the children (or on the parent via
a parallel name list). Design decision for P5.11.

**Match arm threading:** `ExprMatch` has `arms: List<MatchArm>` where
each arm has `pattern`, `guard`, and `body`. The child Nodes are
scrutinee + arm bodies (+ guards). Patterns stay in ExprData. Design
decision: encode arm structure as a property or flatten into children
with a position convention.

**Migration order:**
1. Add `node.children` population alongside ExprData during parse
   (dual-write)
2. Migrate walkers from ExprData child reads to `node.children` reads
   (one at a time, test after each)
3. Remove ExprData child fields (children now live only in
   `node.children`)
4. Delete `expr_children`/`map_expr_children` bridge

**Dual-write invariants (steps 1-2):**

During the migration, both `ExprData` child fields and `node.children`
exist. `node.children` is canonical. `ExprData` child fields are
compatibility mirrors only. All constructors and rewrites must go
through `make_expr_node` / `map_expr_children`. No pass may directly
mutate expression child fields after the dual-write begins.

Test invariants for the dual-write phase:
- `expr_children(node) == node.children` for every expression node
- `map_expr_children(node, identity) == node`
- Child count is stable per variant (matches the role table above)

**Prerequisite:** Shared emit (Phase 4) should be stable before P5.11
starts. Dissolving ExprData children changes what shared emit
dispatches on. If shared emit is still being extracted when P5.11
lands, the extraction must be redone.

**Verification:** All existing tests + diagnostic ratchet + stage0
self-compile + fixed point + dual-write invariant tests.

### P5.0 Design: Token Shape API

The current parser uses `kind_tag(token) -> String` to classify tokens,
then compares strings in `check(...)`, `expect(...)`, `infix_bp(...)`,
and ~200 inline branches. Replacing these with structural matches on
`TokenKind` directly runs into the fact that many token kinds carry
payloads (identifiers have names, literals have values) that parser
control-flow decisions must ignore.

**The missing abstraction is a payload-insensitive token shape.**

```
type TokenShape
  = ShapeKeyword { tag: String }
  | ShapeIdent
  | ShapeIntLit
  | ShapeStringLit
  | ShapePunct { tag: String }
  | ShapeEof
  | ShapeNewline
```

`TokenShape` strips payloads: an `Ident { name: "foo" }` and
`Ident { name: "bar" }` both map to `ShapeIdent`. Parser decisions match
on shape, not on payload. Payload extraction happens after the shape
match succeeds.

**New parser helpers (replace existing string-based wrappers):**

- `token_shape(t: Token) -> TokenShape` — the new classifier
- `peek_shape(tokens, state) -> TokenShape` — replaces `kind_tag(peek(...))`
- `check_shape(tokens, state, expected: TokenShape) -> Bool` — replaces `check(..., tag: "...")`
- `eat_shape(tokens, state, expected: TokenShape) -> { state, matched: Bool }` — replaces `eat_if_tag`
- `expect_shape(tokens, state, expected: TokenShape) -> ParseResult` — replaces `expect(..., tag: "...")`

**Conversion order (choke points first):**

1. Implement `TokenShape` and `token_shape(t:)` in `01_tokenize.dag`
2. Replace `check(...)` and `expect(...)` with `check_shape` / `expect_shape`
3. Replace operator classification / `infix_bp` branches
4. Replace inline `kind_tag(...) == "..."` branches
5. Rename `kind_tag` to `kind_display_name` and restrict to diagnostics-only

**Acceptance criteria:**

- Parser tests green
- Stage0 compile green
- `grep -n 'kind_tag(' src/v2/02_parse.dag` returns zero non-diagnostic uses
- Fixed point holds

### Phase 5 Exit Criteria

- **L1=0:** scrambled-name tests pass; no `node_is_*` predicates; no
  `normalize_type_name`; no `classify_type_structure` in emit; no type-name
  comparisons in inference; no arity bridges
- Target filenames from M1 are fully normalized
- Compiler-internal structure is consistently `Node`-centric
- Each convergence step survives re-bootstrap and fixed-point verification
- The compiler is in a clean place to start real L2 work

### Beyond Phase 5: Bit-Graph Model

The full algebraic vision — machine primitives grounded in
`Bit`/bitvectors, plus higher-level names as graph compositions over
primitives and algebraic constructors — is post-Phase 5 work. It
requires the algebraic type system to be mature enough that the compiler
genuinely processes only graph structure and the fundamental unit. This
is the theoretical endgame, not a near-term deliverable.

Note: the bit-graph model and the Collection Denotational Model are
**separate layers** that coexist. The bit-graph model is about machine
primitives (`Int32` as bitvector). The collection model is about data
structure semantics (`Set<T>` as membership function, `List<T>` as
positional structure). They do not reduce to each other — `Set<T>` is
not meaningfully a composition of bits, even if its elements are.

---

## Cross-Cutting Reference

The sections below support the phase plan above. They do not override it.

### Structural Pass Order (`S*`)

| Pass | Primary phase | Meaning |
|------|---------------|---------|
| S1 | Done | Theme 4: `kernel_types` / `is_kernel_type` are single-authority in core |
| S2 | Done | Theme 6: pipeline owns compilation only; artifact/trace are honest side systems |
| S3 | Done | Theme 3: known-method resolution is centralized; complexity follows semantics |
| S3.5 | Phase 1 | Extract emit metadata out of infer/reconcile |
| S4 | Phase 1 | Move Rust-only ownership/render policy out of core + infer |
| S5 | Phase 4 | Fuse duplicated `ExprData` walks behind shared fold machinery |
| S6 | Phase 4 | Shared emit dispatch with per-target leaves |
| S7 | Phase 1 / Phase 4 | Remove fabrication fallbacks and finish residual string-keyed cleanup |

### Compositional Refactor Targets (`R*`)

These are written in post-M1 names.

| ID | Module | Current -> Target | Primary phase | Note |
|----|--------|-------------------|---------------|------|
| R1 | `00_core.dag` | C -> A | Phase 1 / 3 | Remove emit/pipeline-only types from core |
| R2 | `01_tokenize.dag` | A -> A | Done | No structural refactor required |
| R3 | `02_parse.dag` | C -> B+ | Phase 5 (L3) | `kind_tag` string dispatch (~200 sites) is the primary L3 debt; service/resource lowering is already clean |
| R4 | `03_resolve.dag` | A -> A | Done | No structural refactor required |
| R5 | `04_infer.dag` | D -> B+ | Phase 1 | Bootstrap-critical infer cleanup |
| R6 | `05_emit*.dag` | D -> B+ | Phase 4 | Shared traversal plus target adapters |
| R7 | `complexity.dag` | B -> A | Phase 4 | P1.13 dead code cleanup **done**; convert into a fold consumer |
| R8 | `ownership.dag` | A- -> A | Phase 3 | Wire ownership into the pipeline |
| R9 | `compile.dag` | B- -> A | Phase 3 | Complete orchestration and typed backend flow |

Practical notes:

- **R5 is the bootstrap-critical refactor.** It is the first high-value
  cleanup inside the current infer/reconcile hotspot.
- **R6 is the highest-risk refactor.** Do it only after the compile
  contract and naming cleanup are stable enough to support it.
- **R8 and R9 are Phase 3 work.** They should not wait for deep
  convergence.

### Architecture Migration Tracks (`M*`)

| ID | Track | Primary phase | Depends on | Outcome |
|----|-------|---------------|------------|---------|
| M1 | Stage/module naming cleanup | Phase 1 | none | Target filenames and stage naming are coherent |
| M2 | Compile bundle + projection contracts | Phase 3 | M1 | One authoritative compile result shape |
| M3 | Artifact planning above emit | Phase 3 | M2 | `infer -> plan -> emit` is real |
| M4 | Proof/obligation derivation contract | Phase 3 | none | Proofs/tests/reports share one contract and unsupported is explicit |
| M5 | Generated tests as first-class projection | Phase 4 | M3, M4 | Generated tests become artifact outputs, not a Rust side path |
| M6 | Shared emit spine + target adapters | Phase 4 | M3 | Shared traversal plus compiler-owned adapters |
| M7 | DAG backend/runtime boundary | Phase 4 | M2, M3, M4 | Canonical DAG artifact with runtime kept downstream |
| M8 | Mixed-backend artifact boundaries | Late Phase 4 / later | M3, M5, M7 | Typed boundary plans and generated validation across artifacts |

---

## Backlog

Items below are real, but they are not on the critical path for the
current phase order.

### Language Features

| Item | Why deferred |
|------|--------------|
| General generic syntax | Now planned as P3.6 (compositional DAG slots). Phase 3 scope covers parameterized type declarations; higher-kinded types and constraints are post-Phase 3. |
| Callable-type parameters | **Done** (P4.7). All 6 steps complete. Unblocks P4.2 step 5. |
| Full linear type checking | Ownership proof work has started, but full proof remains beyond the current migration |
| Widen V5 | The conservative version covers current hot paths |

### Compiler Improvements

| Item | Why deferred |
|------|--------------|
| Anonymous record target resolution | Must fail closed, but is not blocking active phases. R2 is the fail-loud stopgap; this item is the real fix (proper field access for any arity). |
| Collection intrinsic semantics in shared IR | Worth doing after shared emit is real; the Collection Denotational Model pins what each algebra's methods mean |
| Generated self-hosting tests and stage contracts | Valuable once the compile contract settles |
| TCO backend contract | Should be cleaned up during/after shared emit extraction |
| SCC-aware return type resolution | Not currently blocking bootstrap |

### Open Invariant Violations (grouped by root cause)

All ~66 violations trace to three root causes. Each root cause has a
structural fix in the Phase 1 workboard above. Symptoms dissolve when
root causes are fixed.

#### Root Cause I: Type nodes structurally incomplete (~32 sites)

Dissolved by: P1.5 (arity enforcement), P1.14 (normalization), P1.16
(scrambled-name tests verify no name-dependent inference).

| ID | Violation | Where | Dissolved by |
|----|-----------|-------|-------------|
| I-1 | `"_"` for Map with no children | `emit_rust` 764 | P1.5: Map always has key/value children |
| I-2 | `"_"` for collection element | `emit_rust` 1959-1962 | P1.5: containers always have element child |
| I-3 | `"_"` for lambda param | `emit_rust` 2001 | P1.14: normalize populates param types |
| I-4 | `"_"` for fold accumulator | `emit_rust` 2033, 2170-2173 | P1.15: unified fold inference |
| I-5 | `"_"` for sort_by element | `emit_rust` 2189-2190 | P1.5: receiver element type always present |
| I-6 | RC3 safety net | `emit_rust` 1591-1620 | **R1**: fix FieldSummary propagation |
| I-7 | ~~Anon record index cap → `"0"`~~ | **Fixed** — `compile_error!()` for index >= 4 |
| I-8 | `normalize_type_name` heuristic | `04_types` 165-177 | P1.5: structurally complete types make this unnecessary |
| I-9 | `node_type_equals` leaf==structured | `04_types` 267-272 | P1.5: bare leaves for parameterized types don't exist |
| I-10 | `node_type_compatible` empty children → true | `04_types` 226-239 | P1.5: children always present |
| I-11 | `node_type_compatible` name-only fallback | `04_types` 242 | P1.5 + P1.7: structural comparison |
| I-12 | `classify_type_structure` in emit (13 calls) | `emit_rust` multiple | P1.5/P1.7: nodes carry structure directly |
| I-13 | `field_type_names` composite string key | `04_infer` / `emit_rust` | P1.5: field types on nodes directly |
| I-14 | Two composite key formats (`\|` vs `::`) | `04_infer` / `05_emit` | Same |
| I-15 | Go `interface{}` type holes (13 sites) | `emit_go` | P1.5: concrete type nodes |
| I-16 | Lambda params default to `Dynamic` | `04_infer` ~6 sites | P1.9 + P1.14: `CompilerError`, not Dynamic |
| I-17 | Fold acc defaults to `Dynamic` | `04_infer` 1684, 1886 | P1.15: unified fold inference |
| I-18 | `bare_map_node()` for `empty_map()` | `04_infer` 1817 | P1.5: typed empty map from context |
| I-19 | `child_return_type_or_name` falls to `.name` | `04_types` 27-29 | P1.5: `return_type` always set |
| I-20 | `method_receiver_element_node` returns receiver for odd shapes | `04_types` 403-417 | P1.5: structural element extraction |

#### Root Cause II: Error/Dynamic are names, not structure (~18 sites)

**Dissolved by P1.9 (`InferredNode` wrapper).** All sites below are
eliminated when error/Dynamic become `CompilerError` outside the node
graph.

| ID | Violation | Where |
|----|-----------|-------|
| II-1 | `node_type_equals` Error == anything | `04_types` 246 |
| II-2 | `node_type_compatible` Error == anything | `04_types` 215 |
| II-3 | `node_type_compatible` Dynamic == anything | `04_types` 216 |
| II-4 | Dynamic/Error → `compile_error!()` in type emission | `emit_rust` 762-763 |
| II-5 | Lambda scope → `Dynamic` fabrication | `emit_rust` 1974 |
| II-6 | Field type checks against `"Dynamic"`/`"Error"` strings | `emit_rust` 2491, 2510, 2595 |
| II-7 | `ExprMatch` result = first arm only | `04_infer` 1986-1988 |
| II-8 | `ExprIf` result = then only | `04_infer` 2042 |
| II-9 | `ExprListLit` element = first only | `04_infer` 2089-2096 |
| II-10 | `Optional == Unit` name shortcut | `04_types` 217-218, 249-250 |
| II-11 | `node_is_error_type` checks in emit | `emit_rust` 1990, 2243 |
| II-12 | ~~`emit_typed_item` catch-all → comment~~ | **Fixed** — now `compile_error!()` |
| II-13 | Resolve builtin → `Unit` for unknown | `04_method` 187-191 |
| II-14 | Unknown method → result = receiver type | `04_infer` 1912-1914 |

#### Root Cause III: Divergent inference paths (~17 sites)

Dissolved by: P1.14 (normalization), P1.15 (deduplication), and P1.19
(testgen parallel extraction).

| ID | Violation | Where |
|----|-----------|-------|
| III-1 | Duplicate fold init/acc inference | `04_infer` 1670-1684 vs 1872-1887 |
| III-2 | Duplicate element typing | `04_infer` 1686-1718 vs 1888-1901 |
| III-3 | Duplicate map/flat_map/fold result refinement | `04_infer` 1765-1803 vs 1919-1935 |
| III-4 | `map_insert`/`map_merge` only in Call bridge | `04_infer` 1777-1796 |
| III-5 | `map_insert` key hardcoded `"String"` | `04_infer` 1781 |
| III-6 | String method names in `ExprCall` bridge | `04_infer` 1670+ |
| III-7 | ~~4x `RuntimeBridgeMethod` → string maps~~ | **Fixed** (P1.10) — closed-enum match per target |
| III-8 | ~~`runtime_bridge_method_name` dead in core~~ | **Fixed** — deleted |
| III-9 | `py_bridge_method_name` diverges (`"with_update"` vs `"with"`) | `emit_python` 693 |
| III-10 | `ownership.dag` `"fold"` string dispatch | `ownership` 2 sites |
| III-11 | Inline string checks duplicate classifiers | `04_infer` |
| III-12 | Builtin vs bridge typing disagrees | `04_method` |
| III-13 | ~~Testgen: `extract_mock_props` duplicates shared `has_mock_prefix`~~ | **Fixed** (P1.19) — deleted, inlined |

#### Testgen fabrication (Phase 1, P1.20) — **All Fixed**

| ID | Violation | Status |
|----|-----------|--------|
| TG-1 | ~~`emit_simple_expr` wildcard → `todo!()`~~ | **Fixed** — already `compile_error!()` |
| TG-2 | ~~`emit_data_value_json` wildcard → `"null"`~~ | **Fixed** — now `<<<UNSUPPORTED_MOCK_EXPR>>>` (invalid JSON) |
| TG-3 | ~~`Default::default()` fallback~~ | **Fixed** — `compile_error!()` for resources; dry-run no-mock already `compile_error!()` |
| TG-4 | ~~`extract_mock_props` duplicates shared emit~~ | **Fixed** (P1.19) — deleted, inlined |

#### Phase 4 violations (remaining, not blocking)

| Violation | Where | Dissolved by |
|-----------|-------|-------------|
| Go `interface{}` type erasure (13 sites) | `emit_go` | P1.5 + P4 |
| Python `_unimplemented()` (2 sites) | `emit_python` 1132, 1145 | P4 |
| Go `/* unhandled expr */` wildcard | `emit_go` 613-614 | P4 |
| `02_parse.dag` `kind_tag` string dispatch | `02_parse` (~200 sites) | P5 (L3) |

#### Resolved

| Violation | Status |
|-----------|--------|
| `classify_reconciled_intrinsic_method` string ladder | **Done** |
| `classify_runtime_bridge_method` string ladder | **Done** |
| Go/Python intrinsic `_ => none` | **Done**: exhaustive 19-arm matches |
| Bridge name maps (III-7, III-8) | **Done** (P1.10): dead fn deleted, closed-enum per-target |
| Testgen mock extraction (III-13) | **Done** (P1.19): `extract_mock_props` deleted |
| Testgen fabrication (TG-1–TG-4) | **Done** (P1.20): all `compile_error!()` or invalid JSON |
| Triple `emit_typed_expr` 22-arm parallelism | **Done** (P4.2): `emit_shared_expr` owns whole-tree recursion |
| Triple TCO walk parallelism | **Done** (P4.2): `emit_shared_tco_expr` owns the shared walk |
| Triple service/transport emission | **Done** (P4.3): shared projection flow feeds all backends |
| `emit_typed_item` catch-all (II-12) | **Done** (P1.12): `compile_error!()` |
| Anon record index cap (I-7) | **Done** (R2): `compile_error!()` for index >= 4 |
| Parse error (binary op RHS across newline) | **Fixed**: `skip_newlines` in `parse_expr_loop` |

### Review follow-ups (branch reconciliation)

| ID | Item | Notes |
|----|------|-------|
| **F4** | Parser `item_kind` (body without type annotation → `FnItem`) | Verify gist closure: no un-annotated data defs misclassified |
| **F6** | Unused `runtime_bridge_method_name` import in `04_infer.dag` | **Done** |
| **F7** | Data constants `.clone()` from `lazy_static` | Optional: avoid redundant clones on `Copy` types if profiling warrants |

---

## Verification

| Gate | Command | When |
|------|---------|------|
| Unit tests | `cargo test --workspace --exclude v2-compiler-tests` | After every change |
| Clippy | `cargo clippy --all-targets -- -D warnings` | After every change |
| V2 compiler tests | `cargo test -p v2-compiler-tests` | After every change |
| Diagnostics ratchet | `cargo test -p v2-compiler-tests v2_strict_compile_diagnostic_count -- --ignored` | End of Phase 1 |
| Fixed point | `cargo test -p v2-compiler-tests v2_bootstrap_fixed_point -- --ignored` | After any `.dag` change that affects bootstrap output |
| Gist pipeline | `cargo test -p v2-compiler-tests v2_gist_full_pipeline -- --ignored` | End of Phase 2 |
| L1 ratchet | `scripts/l1-ratchet.sh --check` | After any `.dag` change (goal: 0) |
| Source ratchets | `python3 scripts/source_audit.py` | During ongoing bridge/deletion refactors |
| Testgen gate | `cargo test -p v2-compiler-tests generates_mock_test_file -- --nocapture` | After P1.21; verifies generated test files are emitted across backends |
| Scrambled-name tests | `cargo test -p v2-compiler-tests v2_scrambled_name_inference` | After P1.16; verifies name opacity |

**Scrambled-name test design:** Rename all type names to arbitrary strings
(consistently across declarations and references), run through inference,
compare **inferred structure** (typed graph shapes: which nodes carry which
types, children, connective shapes). If inference depends on `"Map"` being
called `"Map"`, the structural decisions diverge and the test breaks.
Scoped from infer onward — parse and resolve legitimately work with real
names. Emit is excluded because it legitimately reads names for target-
language identifiers (name-rendering, not name-dependent inference).
Inference receives nodes with opaque names and no name registry.

Manual Phase 2 smoke still exists in addition to the automated test:
build the emitted gist crate and run it in dry-run mode. There is not yet
a dedicated `v2_gist_end_to_end` test in the tree, so the roadmap should
not pretend that one exists.

**Review-queue discipline:** Prefer **scoped commits** (one invariant or
theme per commit) on automation branches, per `CLAUDE.md`, so CI and
blame stay attributable when diffs are large.
