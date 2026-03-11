# v2 Compiler: Target Domain Models

## Purpose

This document defines the **target** `.dag` type models for the v2
compiler, designed from first principles using the same compositional
discipline as the extdeps system. The existing compiler code implements
a working pipeline; this document specifies what that pipeline's types
*should* be, so we can converge toward them incrementally.

The extdeps invariant applies inward: **a compiler module implements a
specification of compiler semantics, not an abstraction of how the Rust
implementation happens to work.**

There are two orthogonal axes of layering in this document:

1. **Pipeline stages** (tokenize → parse → resolve → typecheck → emit)
   — the linear data flow of compilation. This is already well-shaped.

2. **Concept decomposition** — the types that flow THROUGH those stages
   should themselves be layered compositionally, from tautologies upward.
   This is where the model is still centralized and needs work.

**The pipeline stays linear. The model becomes layered.**

Algorithm scratch state (ParserState, KahnState) stays local and is
never exported.

---

## Foundations: Set-Algebraic Type Theory

The type system's semantics are set-theoretic (following `std/types.dag`):

```
Every type T denotes a set ⟦T⟧ of values.
Subtyping is set inclusion:  A <: B  iff  ⟦A⟧ ⊆ ⟦B⟧
```

The type algebra has six operations, corresponding to classical logic:

| Type Constructor | Set Operation | Logic | Notation |
|-----------------|---------------|-------|----------|
| **Atom** | Generator set | Axiom | `⟦String⟧ = Σ*` |
| **Product** | Cartesian product | Conjunction (∧) | `⟦{a: A, b: B}⟧ = ⟦A⟧ × ⟦B⟧` |
| **Coproduct** | Disjoint union | Disjunction (∨) | `⟦A \| B⟧ = ⟦A⟧ ⊔ ⟦B⟧` |
| **Application** | Functor application | Quantification | `⟦F<A>⟧ = F(⟦A⟧)` |
| **Refinement** | Subset comprehension | Predicate restriction | `⟦T where P⟧ = {x ∈ ⟦T⟧ \| P(x)}` |
| **Reference** | Indirection (pre-resolution) | Variable | `⟦Named("Foo")⟧ = ⟦Foo⟧` (by lookup) |

These six constructors are **complete** for the language's type system.
Every type expressible in `.dag` syntax decomposes into these six forms.

### Subsumption rules

Application subsumes three current TypeExpr variants:

```
Container { kind: List, element: T }  =  App { name: "List", args: [T] }
Container { kind: Set, element: T }   =  App { name: "Set", args: [T] }
MapType { key: K, value: V }          =  App { name: "Map", args: [K, V] }
Optional { inner: T }                 =  App { name: "Option", args: [T] }
```

This is not a modeling choice — it is set-theoretically forced. `List<T>`
IS the application of the `List` constructor to `T`. Having both `App`
and `Container` as peers means the same set has two representations,
which violates canonicality.

### The kernel types as atoms

The type universe is a bounded lattice with 8 atoms:

```
⊤ = Json                  (universal structured value)
⊥ = Unit = {}             (empty/void)

Atoms = { String, Int, Bool, Float, Secret, Bytes, Unit, Json }
```

Every atom is a primitive type with no structural expansion. All other
types are built from atoms by product, coproduct, application, and
refinement.

### Optionality and cardinality

In the surface syntax, `T?` is sugar. In the type algebra:

```
⟦T?⟧ = ⟦T⟧ ∪ {none}
```

The deeper question is whether optionality is a **type constructor**
(a transformation on ⟦T⟧) or a **cardinality constraint** on a binding
site (how many values of type T this site holds).

Currently the model has THREE representations of "might be absent":

1. `Field.optional: Bool` — cardinality on the binding
2. `TypeExpr::Optional { inner: T }` — optionality as a type wrapper
3. `App("Option", [T])` — optionality as type application (proposed)

This proliferation is the problem. The extdeps approach would be:
define one concept, derive the rest.

**Direction: cardinality as the primitive.** Optionality is not a
property of the type — it's a property of how the type is used at a
binding site. A `String` is a `String`; whether a particular field
CAN be absent is a cardinality constraint on that field, not a
different type.

Set-algebraically, cardinality governs the multiplicity of the fiber
over a binding site:

```
⟦Required⟧  : exactly 1 value from ⟦T⟧      (1..1)
⟦Optional⟧  : 0 or 1 value from ⟦T⟧         (0..1)
⟦Many⟧      : 0..n values from ⟦T⟧           (0..n, = ⟦T⟧*)
⟦AtLeastOne⟧: 1..n values from ⟦T⟧           (1..n, = ⟦T⟧⁺)
```

This means:
- `Optional` exits TypeExpr (not collapsed to App — **removed**)
- `Field.optional: Bool` becomes `Field.cardinality: Cardinality`
- TypeExpr is purely "what" (the set of values), never "how many"
- Return type positions, params, etc. also carry cardinality

**Boundary**: Cardinality governs the outermost multiplicity of a
binding site. `List<T>`, `Set<T>`, `Map<K,V>` remain as type
constructors via `App` — they describe the *structure* of the value,
not the multiplicity of the binding. You can have `List<List<String>>`
(nested type construction) and you can have a field of type `List<T>`
with cardinality Required (exactly one list) or Optional (maybe no
list). These are orthogonal.

**Not yet decided**: The precise surface syntax for cardinality
annotations, whether `Cardinality` is a finite coproduct (Required |
Optional | Many | AtLeastOne) or a min/max range `{ min: Int, max:
Int? }`, and the recursive self-application question (the `Cardinality`
field on `Field` is itself a required binding — cardinality all the way
down, bottoming out at the kernel types). These need further design
work.

---

## Layer 0: Compiler Primitives

*"What is a diagnostic? A source location? A compiler output?"*

These are the atoms of compiler infrastructure — they have no knowledge
of tokens, AST nodes, types, or any language-level concept. They are
to the compiler what `std/types.dag` is to the extdeps system.

```dag
// Source locations (already in std.types, imported here)
// ⟦SourceSpan⟧ = ⟦Int⟧ × ⟦Int⟧  (byte offsets)
type SourceSpan { start: Int, end: Int }

// Diagnostic severity: a 3-element set.
// ⟦Severity⟧ = {Error, Warning, Info}  (|⟦Severity⟧| = 3)
type Severity = Error | Warning | Info

// A diagnostic is a located message with severity.
// ⟦Diagnostic⟧ = ⟦Severity⟧ × ⟦String⟧ × ⟦SourceSpan?⟧ × ⟦String?⟧
type Diagnostic {
  severity: Severity
  message: String
  span: SourceSpan?
  module_name: String?
}

// The compiler's final output: files and diagnostics.
// ⟦CompileResult⟧ = ⟦List<TextFile>⟧ × ⟦List<Diagnostic>⟧
type CompileResult {
  files: List<TextFile>
  diagnostics: List<Diagnostic>
}

type TextFile {
  path: String    // relative output path
  content: String
}
```

### The pass contract

Every compiler pass is a function of the form:

```
pass : Input → (Output × List<Diagnostic>)
```

This is the Writer monad over `List<Diagnostic>`. The DSL cannot express
parametric polymorphism, so we cannot write `WithDiag<T>`. Instead, the
contract is **structural**: every pass function returns its primary
value directly, with a `diagnostics: List<Diagnostic>` field alongside
it. No named wrapper types.

**Current problem**: typecheck.dag defines 8 named wrappers
(`ResolveResult`, `ItemResult`, `FieldResult`, `VariantResult`,
`ParamResult`, `OperationResult`, `CapabilityResult`,
`ResourceUseResult`, `TypeBodyResult`, `TypecheckModuleResult`,
`EnvResolveResult`). Each is `{ value: T, diagnostics: List<Diagnostic> }`
with a different name for `value`.

**Target**: Eliminate all named wrappers. Functions return anonymous
records:

```dag
// CURRENT (11 named wrapper types):
fn resolve_field(field: Field, env: TypeEnv, ...) -> FieldResult

// TARGET (zero wrapper types):
fn resolve_field(field: Field, env: TypeEnv, ...) -> { field: Field, diagnostics: List<Diagnostic> }
```

The naming of the value field varies per function (`field`, `variant`,
`param`, `item`, etc.) — this is correct because the field name
documents what the value IS, not that it is "a result." The wrapper
type was adding a name to the pair without adding information.

**Exception**: Top-level pass outputs (`ModuleGraph`, `TypedGraph`) are
real semantic types, not wrappers. They carry structure beyond
`{ value, diagnostics }`. These belong in the model and stay named.

---

## Layer 1: Type Algebra

*"What is a type in this language?"*

This layer defines the compiler's internal representation of types.
It imports Layer 0 (SourceSpan) and nothing else. It knows nothing
about tokens, parsing, services, or code emission.

### Target TypeExpr (5 variants, down from 9)

```dag
// A type expression — the compiler's representation of a type.
//
// Set-theoretically: TypeExpr is the free algebra generated by
// the five constructors below, quotiented by the subsumption rules
// in the Foundations section.
//
// TypeExpr answers "WHAT set of values?" — never "how many?"
// Multiplicity is governed by Cardinality at binding sites.
//
// Invariant (post-typecheck): no Ref variants survive except
// recursive cycle-breakers that are guaranteed to be defined.

type TypeExpr
  // Pre-resolution reference. Goes away after typechecking.
  // ⟦Ref⟧ = lookup in type environment
  = Ref { name: String, span: SourceSpan }

  // Kernel atom. No structural expansion.
  // ⟦Atom⟧ = one of the 8 kernel primitive sets
  | Atom { name: String, span: SourceSpan }

  // Labeled Cartesian product.
  // ⟦Product { fields: [f₁:T₁, ..., fₙ:Tₙ] }⟧ = ⟦T₁⟧ × ... × ⟦Tₙ⟧
  //   (with labels for projection)
  | Product { name: String?, fields: List<Field>, span: SourceSpan }

  // Labeled disjoint union.
  // ⟦Coproduct { variants: [V₁, ..., Vₙ] }⟧ = ⟦V₁⟧ ⊔ ... ⊔ ⟦Vₙ⟧
  //   (with tags for injection/case analysis)
  | Coproduct { name: String?, variants: List<Variant>, span: SourceSpan }

  // Type constructor application (subsumes Container, MapType).
  // ⟦App { name: F, args: [A₁, ..., Aₙ] }⟧ = F(⟦A₁⟧, ..., ⟦Aₙ⟧)
  //
  // Built-in constructors:
  //   "List"         : * → *           Vec<T>
  //   "Set"          : * → *           BTreeSet<T>
  //   "Map"          : * → * → *       BTreeMap<K, V>
  //   "NonEmptyList" : * → *           Vec<T>  (with runtime invariant)
  //   "NonEmptySet"  : * → *           BTreeSet<T>  (with runtime invariant)
  //
  // Note: "Option" is NOT a type constructor. Optionality is a
  // cardinality constraint (0..1) on the binding site, not a type
  // transformation. See Foundations § "Optionality and cardinality."
  //
  // User-defined type constructors (post-generics) use the same form.
  | App { name: String, args: List<TypeExpr>, span: SourceSpan }

  // Subset comprehension (refinement type).
  // ⟦Refine { base: T, predicates: [P₁, ..., Pₙ] }⟧
  //   = {x ∈ ⟦T⟧ | P₁(x) ∧ ... ∧ Pₙ(x)}
  | Refine { base: TypeExpr, predicates: List<Predicate>, span: SourceSpan }
```

### What was eliminated and why

| Removed | Replaced by | Set-theoretic justification |
|---------|-------------|---------------------------|
| `Named` | `Ref` | Renamed for clarity: it is a reference, not a type |
| `Primitive` | `Atom` | Renamed: these are the atoms of the type algebra |
| `Container` | `App` | `List<T>` IS `App("List", [T])` — same set, one representation |
| `MapType` | `App` | `Map<K,V>` IS `App("Map", [K, V])` |
| `Optional` | Cardinality | `T?` is not a type — it's type T with cardinality 0..1 on the binding site |
| `Refined` | `Refine` | Renamed for verb form consistency |
| `TypeApp` | `App` | Renamed: shorter, and now the only application form |

**Key decision: Optional removed entirely, not collapsed to App.**

The earlier draft proposed collapsing `Optional { inner: T }` to
`App("Option", [T])`. The direction is more principled than that:
optionality is not a type at all. It is a cardinality constraint on
the binding site. `T?` in a field declaration means "this field has
type T and cardinality 0..1" — the type is still T, the multiplicity
is on the field. This eliminates Optional from TypeExpr without
introducing "Option" as a type constructor, and unifies it with the
existing `Field.optional: Bool` (which also encodes multiplicity).

### Predicate (unchanged, already clean)

```dag
type Predicate
  = NonEmpty
  | Pattern { regex: String }
  | Range { min: Int?, max: Int? }
  | Brand { name: String }
  | ContentEncoding { encoding: String }
  | Format { name: String }
  | Domain { name: String }
```

This is already compositional. `Domain { name }` is the open extension
point — it names a domain-specific predicate without the compiler
needing to know what it means. The compiler verifies the predicate
is syntactically well-formed; domain semantics are the backend's problem.

### TypeBody (unchanged)

```dag
type TypeBody
  = Record { fields: List<Field> }
  | Sum { variants: List<Variant> }
  | Alias { base: TypeExpr }
```

This is the right decomposition. A type definition's body is one of
three structural forms. `Record` and `Sum` map to `Product` and
`Coproduct` in the type algebra; `Alias` is transparent indirection.

### Field, Variant, Param

```dag
type Field {
  name: String
  type_expr: TypeExpr         // "what" — the set of values
  cardinality: Cardinality    // "how many" — replaces optional: Bool
  default_value: Expr?        // see note below
  span: SourceSpan
}

type Variant {
  name: String
  fields: List<Field>
  span: SourceSpan
}

type Param {
  name: String
  type_expr: TypeExpr
  cardinality: Cardinality    // params can be optional too
  default_value: Expr?
  span: SourceSpan
}
```

A variant with no fields is a unit variant (tag-only injection into the
coproduct). A variant with fields is a payload variant (tagged product
within the coproduct). Variant itself is unchanged.

**Note on `default_value: Expr?`**: This field declaration uses `?`
(cardinality 0..1) on the Field type itself. Cardinality applies at
every binding site, recursively — bottoming out at the kernel types.
The representation of "Expr?" on Field's own definition is an instance
of the same concept: the default_value binding has type Expr and
cardinality Optional.

---

## Layer 2: Syntactic Model

*"What is a token? What is an AST node?"*

This layer defines the source-faithful representation of `.dag` programs.
It imports Layer 0 (SourceSpan, Diagnostic) and Layer 1 (TypeExpr,
Field, Variant, Param, Predicate). It knows nothing about resolution,
typechecking, or code emission.

### Tokens (unchanged, already clean)

TokenKind is a finite coproduct (|⟦TokenKind⟧| ≈ 75). Each variant
is either a unit tag (keyword/punctuation) or carries a payload
(LitStr, LitInt, Ident). The current definition is correct.

### AST Items

```dag
type Module {
  name: String
  imports: List<Import>
  items: List<Item>
  span: SourceSpan
}

type Import {
  module_path: String
  names: List<String>
  span: SourceSpan
}

type Item
  = TypeDef { name: String, body: TypeBody, span: SourceSpan }
  | FuncDef { name: String, params: List<Param>, return_type: TypeExpr?,
              uses: List<ResourceUse>, body: Expr, span: SourceSpan }
  | FnDef { name: String, params: List<Param>, return_type: TypeExpr?,
            body: Expr, span: SourceSpan }
  | ServiceDef { name: String, transport: TransportBinding,
                 config: ServiceConfig?, operations: List<OperationDef>,
                 span: SourceSpan }
  | ResourceDef { name: String, capabilities: List<CapabilityDef>,
                  span: SourceSpan }
  | DataDef { name: String, type_expr: TypeExpr, value: Expr,
              span: SourceSpan }
  | ExternFuncDecl { name: String, params: List<Param>,
                     return_type: TypeExpr, span: SourceSpan }
```

This is source-faithful. Item is the coproduct of everything that
can appear at the top level of a module. Each variant records exactly
what the parser saw, nothing more.

### Expressions (unchanged, already clean)

The Expr type is a 16-variant coproduct. Each variant corresponds to
a syntactic form. No semantic interpretation. The current definition
in core.dag is correct.

### Match Patterns

```dag
type MatchPattern
  = Bind { name: String }
  | LitPattern { value: LiteralValue }
  | VariantPattern { name: String, field_bindings: List<FieldBinding> }
  | Wildcard

type FieldBinding {
  field_name: String
  binding: MatchPattern
}
```

This is the target. `FieldBinding` enables destructuring:
`Some { value: x }` binds `x` to the `value` field of the `Some`
variant. The current core.dag definition matches this target.

**Current gap**: parse.dag still constructs `VariantPattern` using the
old binding-list shape without field_bindings. The parser must be
updated to produce `FieldBinding` records.

### Service/Transport (AST level)

```dag
// Transport binding — how a service communicates.
// This is a typed coproduct (not a string-keyed bag).
// Already correct in current core.dag.
type TransportBinding
  = RestBinding { base_url: Expr, auth: AuthConfig?, headers: List<HeaderDef> }
  | ShellBinding { argv: List<Expr>, env: List<EnvDef> }
  | FileBinding { base_path: Expr }
  | LocalBinding

// Service configuration — CURRENT (ad-hoc, ungrounded)
type ServiceConfig {
  endpoint: Expr
  auth: Expr?
  rate_limit: Expr?
  retry: Expr?
}
```

**Problem with ServiceConfig**: This record exists at the AST level
but its fields are unresolved `Expr` values with no type constraints.
It doesn't implement a specification — it's a grab-bag of "things a
service might need."

**Target ServiceConfig**: At the AST level, ServiceConfig should record
exactly what the parser saw, which means `Expr` fields are correct
(they are unresolved source expressions). But the field set should
match the actual grammar, not be speculative. If the grammar doesn't
support `rate_limit` declarations yet, the field shouldn't exist.

This is Layer 2 (syntactic) — the semantic interpretation of service
config belongs in Layer 3.

### Operation definition (AST level)

```dag
type OperationDef {
  name: String
  inputs: List<Field>
  outputs: List<Field>
  response: List<ResponseMapping>
  mock_response: List<MockResponseDef>
  modifiers: List<OperationModifier>
  span: SourceSpan
}

type OperationModifier = Idempotent | Readonly | Hermetic

type ResponseMapping { status: Expr, type_expr: TypeExpr }
type MockResponseDef { status: Expr, body: Expr, description: String? }
```

This is source-faithful. At the AST level, operations record what was
declared: inputs, outputs, response mappings, mock data, modifiers.
The semantic meaning of these (e.g., that `Idempotent` implies safe
retry) belongs in Layer 3.

---

## Layer 3: Semantic Model

*"What is a resolved module? A typed module? A service's semantics?"*

This layer defines the compiler's understanding of a program after
analysis. It imports Layer 0 (Diagnostic), Layer 1 (TypeExpr, etc.),
and Layer 2 (Module, Item, etc.). It knows nothing about code emission
or target languages.

### Canonical homes

The following types are defined ONCE in core.dag and imported by both
typecheck.dag and emit.dag. No local redeclarations.

**Current problem**: `TypedGraph`, `TypedModule`, `TypeEnv`,
`TypeBinding` are defined independently in both typecheck.dag and
emit.dag. `ModuleGraph`, `ResolvedModule`, `ResolvedImport` are
forward-declared in typecheck.dag with a comment saying "remove once
resolve module exists" — but resolve.dag does exist now.

**Target**: All semantic types live in core.dag:

```dag
// === Resolution output (produced by resolve, consumed by typecheck) ===

type ModuleGraph {
  modules: List<ResolvedModule>
  diagnostics: List<Diagnostic>
}

type ResolvedModule {
  module: Module
  resolved_imports: List<ResolvedImport>
  dep_order: Int
}

type ResolvedImport {
  module_path: String
  names: List<String>
  target_module: Module?
}

// === Typecheck output (produced by typecheck, consumed by emit) ===

type TypedGraph {
  modules: List<TypedModule>
  diagnostics: List<Diagnostic>
}

type TypedModule {
  module: Module       // items now have resolved TypeExprs
  type_env: TypeEnv
}

type TypeEnv {
  bindings: List<TypeBinding>
}

type TypeBinding {
  name: String
  resolved: TypeExpr
}
```

resolve.dag, typecheck.dag, and emit.dag all import these from core.dag.
No local copies. Single source of truth.

### Service semantics (post-typecheck)

At the AST level (Layer 2), `TransportBinding` and `ServiceConfig` are
syntactic — they record what was written. After typechecking, we need
a semantic model of what a service operation MEANS.

The extdeps system has vocabulary for this domain (`SideEffects`,
`OperationBehavior`, `RetryPolicy`, `CloudAuthScheme`). The compiler
defines its own types using compatible shapes — same names where they
overlap, independently owned. This is not a sketch yet, just the
direction:

- The emitter should consume typed semantic records, not raw AST
- Behavioral properties (idempotent, readonly, side effects) should
  be explicit fields, not inferred from modifier tags at emit time
- The semantic model is backend-neutral — it says WHAT, not HOW
- A second backend consumes the same semantic model

**Deferred**: The concrete `OperationSemantics` type and the
`OperationPlan`/`BackendPlan` intermediate layer. This is post-
self-hosting work. The priority is getting the type algebra (Layer 1)
and canonical homes (Layer 3 basics) right first.

---

## Layer 4: Emission Model

*"What does the emitter need to know?"*

The emitter consumes Layer 3 (TypedGraph, OperationSemantics) and
produces Layer 0 (List<TextFile>). It is the only layer that knows
about target languages.

### Backend selection

```dag
type Backend = Rust | Python
```

This is a closed coproduct. Adding a backend means adding a variant
and implementing the emission functions. The pipeline selects the
backend; the emitter dispatches.

### Emission structure

The emitter is a collection of pure functions:

```
emit_module : TypedModule → TextFile
emit_type   : TypeExpr → String        (Rust type syntax)
emit_expr   : Expr → String            (Rust expression syntax)
emit_item   : Item → String            (Rust item syntax)
```

These functions are target-language-specific. They live in emit.dag
(for Rust) or a future emit_python.dag. They do NOT define new types
for their intermediate representations — they produce strings directly.

**Current problem**: emit.dag locally redefines `TypedGraph`,
`TypedModule`, `TypeEnv`, `TypeBinding`. After Layer 3 types move to
core.dag, emit.dag imports them like every other consumer.

### Built-in type constructor mapping

The emitter pattern-matches on `App.name` for built-in constructors:

```dag
fn emit_type_app(name: String, args: List<TypeExpr>) -> String {
  match name {
    "List"         => "Vec<" + emit_type(args[0]) + ">"
    "Set"          => "BTreeSet<" + emit_type(args[0]) + ">"
    "Map"          => "BTreeMap<" + emit_type(args[0]) + ", " + emit_type(args[1]) + ">"
    "Option"       => "Option<" + emit_type(args[0]) + ">"
    "NonEmptyList" => "Vec<" + emit_type(args[0]) + ">"
    "NonEmptySet"  => "BTreeSet<" + emit_type(args[0]) + ">"
    _              => name + "<" + join(map(args, emit_type), ", ") + ">"
  }
}
```

This replaces three separate match arms (Container, MapType, Optional)
with one (App), and the built-in constructor table is explicit and
exhaustive. A Python backend would have a parallel table:
`List` → `list[T]`, `Map` → `dict[K, V]`, etc.

---

## Layer 5: Pipeline Composition

*"How do stages compose?"*

### Current state

pipeline.dag defines three monomorphic result types:

```dag
type StageResult { value: List<Module>, diagnostics: List<Diagnostic> }
type TokenizeResult { tokens: List<Token>, diagnostics: List<Diagnostic> }
type ParseStageResult { modules: List<Module>, diagnostics: List<Diagnostic> }
```

And `compile_sources` manually chains stages with early-return on error.

### Target

Eliminate `StageResult`, `TokenizeResult`, `ParseStageResult`. Each
stage function already has a well-typed return:

```
tokenize        : String → List<Token>
parse           : List<Token> → ParseResult         (Module? × Diagnostic?)
resolve_modules : List<Module> → ModuleGraph         (has .diagnostics)
typecheck       : ModuleGraph → TypedGraph            (has .diagnostics)
emit_rust       : TypedGraph → List<TextFile>
```

The pipeline is a linear composition of these functions. Diagnostics
accumulate via the `diagnostics` fields on `ModuleGraph` and
`TypedGraph`. The pipeline's job is:

1. Map `tokenize` over source files
2. Map `parse` over token lists
3. Collect parse diagnostics, bail if errors
4. `resolve_modules` on the parsed modules
5. `typecheck` on the module graph
6. `emit_rust` (or `emit_python`) on the typed graph
7. Merge all diagnostics into `CompileResult`

This is what `compile_sources` already does. The only change is
removing the wrapper types that duplicate what the stage functions
already return.

---

## Concept Layering: From Tautologies to Services

The pipeline layers above (Layer 0-5) describe the compiler's stages.
This section describes a different axis: how the **concepts themselves**
should be decomposed, independent of which stage uses them.

### The current problem

`v2/std/core.dag` is a big central bundle. It defines token kinds,
AST nodes, type expressions, service/transport types, operation
definitions, and compiler output — all in one 333-line module. This
is pragmatic for bootstrap but it is not a layered explanation of why
things are the way they are.

A concrete example: `RestBinding { base_url, auth, headers }` is
typed (good), but it collapses several layers of truth into one record:

- Raw HTTP concerns (methods, headers, status codes)
- REST-style concerns (path templates, content negotiation)
- Provider-specific conventions (GitHub's Accept header, API version)
- Service-specific operation facts (this endpoint, these params)

In extdeps terms, that's like jumping from "TCP exists" straight to
"GitHub Gists create request" in one hop. It's cleaner than strings,
but it's not a stack of truths.

### The key rule

**A higher layer may name a concept, but it should not explain it
from scratch if a lower layer already can.**

- `GitHub` should not redefine what an HTTP header is.
- `Gists` should not redefine what REST is.
- `tool.gist` should not redefine what GitHub auth is.
- `emit_rust` should not redefine what a function or struct is if
  the type/syntax vocabulary already did.

This is the same invariant as extdeps: model the spec, not an idea
of the spec; each layer only knows lower layers; higher layers compose
rather than invent.

### Target concept modules

The direction is to split `v2.std.core` into smaller modules, each
answering ONE question from tautological definitions upward:

**Concept Layer 0: Universal vocabulary (tautologies)**

These define concepts that are true by construction — no knowledge of
any specific domain, provider, or language.

```
v2.std.span          "What is a source location?"
v2.std.lex           "What are the token categories?"
v2.std.types         "What is a type? A field? A predicate?"
v2.std.behavioral    "What is idempotency? Readonly? Purity?"
v2.std.http          "What is an HTTP method? A header? A status code?"
```

Analogous to extdeps `std/errors`, `std/behavioral`, `std/rate_limit`.

**Concept Layer 1: Domain styles / protocol families**

These define what *kind of thing* something is — abstract vocabulary,
still not provider- or backend-specific.

```
v2.std.rest          "What is a REST operation?" (composes http)
v2.std.resource      "What is a resource/capability lifecycle?"
v2.std.service       "What is a generic service/operation?"
v2.std.backend       "What is a target backend artifact?"
```

Analogous to extdeps `cloud/cloud.dag` ("What is a cloud provider?").

**Concept Layer 2: Provider / backend facts**

These instantiate Layer 1 vocabulary with real facts.

```
v2.targets.rust      Rust naming, module/file mapping, ownership, derives
v2.targets.python    Python emission conventions
v2.providers.github  Base URL, auth, standard headers, pagination style
v2.providers.gcp     OAuth/SA conventions, resource naming, error shape
```

Analogous to extdeps `cloud/gcp/gcp.dag` ("What is GCP?").

**Concept Layer 3: Concrete services / concrete compiler constructs**

These compose Layer 1 vocabulary with Layer 2 facts.

```
v2.providers.github.gists     "Given REST + GitHub facts + these schemas, this is Gists"
v2.targets.rust.type_emit     "Given Rust facts + TypeExpr, this is Rust type emission"
v2.targets.rust.service_emit  "Given Rust facts + REST + service, this is transport code"
```

Analogous to extdeps `cloud/gcp/secret_manager.dag`.

**Concept Layer 4: Workflows / tools / pipeline**

These only compose, never redefine.

```
v2.compiler.pipeline    Wires stages together
v2.tools.codegen        Invokes the compiler
```

### How concept layers intersect pipeline stages

The concept layers and pipeline stages are orthogonal:

```
                    concept layer 0   concept layer 1   concept layer 2
                    (tautologies)     (domain styles)   (provider facts)
                    ─────────────     ───────────────   ───────────────
pipeline:tokenize   lex vocabulary      —                  —
pipeline:parse      syntax, types       —                  —
pipeline:typecheck  types, behavioral   service, resource  —
pipeline:emit       types               backend            rust/python facts
```

Every pipeline stage consumes types from the concept layers it needs.
The concept layers do not know about pipeline stages. This means you
can change how the emitter works without changing what REST means,
and you can add a new provider without changing the typechecker.

### What this does NOT mean

This is **not** "everything is a DAG node again." The v2 direction is
right that the compiler's primary model is a typed domain model, not
an executable DAG IR. The extdeps analogy is about **layered truths**,
not about forcing every truth into one runtime graph representation.

Preserve:
- Linear compiler stages
- Typed domain models
- File-to-file compiler boundary

Import from extdeps only the thing that actually matters:
**concepts should be layered compositionally from tautologies upward.**

### Concrete example: RestBinding before and after

**Today** (centralized in core.dag):

```dag
type TransportBinding
  = RestBinding { base_url: Expr, auth: AuthConfig?, headers: List<HeaderDef> }
  | ShellBinding { argv: List<Expr>, env: List<EnvDef> }
  | FileBinding { base_path: Expr }
  | LocalBinding

type AuthConfig {
  scheme: String        // soft string — not structural
  header: String        // soft string — not structural
  token_expr: Expr
}
```

**Direction** (layered):

```dag
// v2.std.http — tautological HTTP vocabulary
type HttpMethod = Get | Post | Put | Patch | Delete
type HeaderName = String where non_empty
type HeaderDef { name: HeaderName, value: Expr }
type AuthScheme = BearerToken | BasicAuth | ApiKeyHeader { header: HeaderName }

// v2.std.rest — what a REST operation shape looks like
type RestRequestShape {
  method: HttpMethod
  path_template: String
  headers: List<HeaderDef>
  auth: AuthScheme
}

// v2.providers.github.core — GitHub-specific facts
data github_defaults = {
  base_url: "https://api.github.com",
  auth: BearerToken,
  headers: [
    { name: "Accept", value: "application/vnd.github+json" },
    { name: "X-GitHub-Api-Version", value: "2022-11-28" }
  ]
}
```

The AST-level `TransportBinding` still exists — it records what the
parser saw. But now `AuthConfig.scheme: String` becomes
`AuthScheme` (a typed coproduct), and lower-layer vocabulary exists
for the emitter and typechecker to reference instead of inventing
ad-hoc representations.

### Migration note

Splitting `core.dag` into 5-8 modules is not Phase 0 work. Phase 0
is canonical type homes + TypeExpr consolidation + cardinality. The
concept layering is a longer arc that runs alongside self-hosting,
not before it. But the direction should inform how we add new types
— new concepts go in the right layer, not appended to core.dag.

---

## Cross-Cutting: Relationship to extdeps

### Same algebra, independent namespaces

The v2 compiler and the extdeps system use the same type algebra
(Atom, Product, Coproduct, App, Refine). The compiler processes that
algebra; the extdeps are expressed in it. They share constructors the
way two programs share a programming language — not by importing each
other's types, but by speaking the same structural language.

| Surface syntax | TypeExpr | extdeps usage |
|---------------|----------|---------------|
| `String` | `Atom { name: "String" }` | Field types in all extdeps |
| `List<T>` | `App { name: "List", args: [T] }` | `List<GcpScope>` etc. |
| `Map<K,V>` | `App { name: "Map", args: [K,V] }` | `Map<String, String>` etc. |
| `T?` | Cardinality Optional on binding | Optional fields everywhere |
| `Int where range(...)` | `Refine { base: Atom, ... }` | `HttpStatus`, `Port` etc. |
| `A \| B \| C` | `Coproduct { variants: [...] }` | `CloudAuthScheme` etc. |
| `{ a: A, b: B }` | `Product { fields: [...] }` | `ServiceEndpoint` etc. |

### Compositional discipline as template

The extdeps 5-layer architecture (Layer 0 universal primitives →
Layer 5 tool integration) is the template, not the dependency. The
compiler's own layers (Layer 0 compiler primitives → Layer 5 pipeline)
follow the same structural rules:

- Each layer only imports downward
- Types answer "what is X?" not "how is X implemented?"
- Algorithm scratch stays local
- No type duplication across module boundaries

### Behavioral compatibility

The compiler's `OperationModifier` (Idempotent | Readonly | Hermetic)
and the extdeps `SideEffects` / `OperationBehavior` describe the same
domain. The compiler defines its own types using the same shapes and
names where they overlap. A reader of `std/behavioral.dag` immediately
understands the compiler's behavioral annotations, and vice versa.

If a shared stable `std/` foundation layer emerges that both the
compiler and extdeps import from, this compatibility becomes structural
sharing. Until then, it's convention — enforced by the modeling
discipline, not the import graph.

### Where we are vs where we're going

**Today**: v2 is structurally typed but semantically centralized.
`core.dag` is a useful bootstrap aggregate, but one big module
defining tokens + AST + types + services + output is not a layered
explanation of compiler concepts.

**Direction**: Structurally typed AND semantically layered. The
pipeline stays linear. The concepts that flow through the pipeline
decompose into tautological vocabulary → domain styles → provider/
backend facts → concrete composition. Each concept is a stack of
truths, not a flat record with a pile of fields.

---

## Migration Plan

### Phase 0a: Canonical type homes

Move `ModuleGraph`, `ResolvedModule`, `ResolvedImport`, `TypedGraph`,
`TypedModule`, `TypeEnv`, `TypeBinding` to core.dag. Update imports
in resolve.dag, typecheck.dag, emit.dag. Delete local redeclarations.

*Estimated: 1 session. Zero semantic change.*

### Phase 0b: TypeExpr consolidation

Rename: `Named` → `Ref`, `Primitive` → `Atom`, `Refined` → `Refine`,
`TypeApp` → `App`. Remove: `Container`, `MapType`, `Optional`.

Update parser to produce `App` for `List<T>`, `Set<T>`, `Map<K,V>`,
`T?`. Update typechecker and emitter match arms. Delete `ContainerKind`.

*Estimated: 1-2 sessions. Semantics-preserving refactor.*

### Phase 0c: Eliminate pass-local wrapper types

Delete `ResolveResult`, `ItemResult`, `FieldResult`, `VariantResult`,
`ParamResult`, `OperationResult`, `CapabilityResult`,
`ResourceUseResult`, `TypeBodyResult`, `TypecheckModuleResult`,
`EnvResolveResult` from typecheck.dag. Delete `StageResult`,
`TokenizeResult`, `ParseStageResult` from pipeline.dag.

Return anonymous records from all functions.

*Estimated: 1 session. Zero semantic change.*

### Phase 0d: OperationSemantics (deferred)

Add `OperationSemantics` to core.dag. Build it in typecheck from
`OperationDef` + modifiers. Consume in emitter instead of raw AST.

*Deferred until after self-hosting bootstrap.*

---

## Sustainability of the Foundation

The deepest risk to this model is not getting it wrong initially — it's
**forgetting** the primitives later. If Cardinality is the concept but
someone can still write `optional: Bool` or `TypeExpr::Optional` or
`App("Option", [T])`, the discipline breaks down through accumulated
drift. This section addresses that.

### Make the right thing structural, the wrong thing unrepresentable

The core defense is: **if TypeExpr has no Optional variant, you cannot
express optionality as a type.** The compiler literally does not have
the construct. If you want "might be absent," the only path is a
Cardinality annotation on the binding site. This is not convention —
it's structural impossibility of the alternative.

Similarly: if `Field` requires `cardinality: Cardinality` and there is
no `optional: Bool`, you cannot forget cardinality. You can only choose
a value for it (Required, Optional, etc.). The default is Required, so
the common case requires no annotation.

This is the "make illegal states unrepresentable" principle applied to
the type model itself.

### Foundation types must come first

Cardinality, Diagnostic, Severity, SourceSpan — these are kernel
concepts that everything else is built on. They need to exist at a
layer so fundamental that both the compiler and the extdeps can
reference them without circular dependency.

Current state: `std/types.dag` mixes kernel primitives (SourceSpan)
with domain-specific types (GcpProjectId, CredentialFlow, ToolEntry).
This means importing SourceSpan also pulls in 500 lines of domain
vocabulary.

**Direction**: A small, stable kernel module that defines ONLY the
structural primitives:

```
kernel candidates:
  SourceSpan          — "where in the source"
  Cardinality         — "how many at a binding site"
  Severity            — "how bad is this diagnostic"
  Diagnostic          — "a located compiler message"
```

This kernel is imported by everything — the compiler, the extdeps,
the std library. It changes almost never. Domain-specific types
(tokens, AST, cloud providers, etc.) live in higher layers that
import the kernel.

Whether this becomes a literal `std/kernel.dag` file or a section of
`std/types.dag` with clear layering is a representation question, not
a modeling question. The modeling question is: what are the types that
MUST exist before anything else can be defined?

### Refactoring for sustainability

The existing types are not wrong — they're incomplete. The path is
refinement, not redesign:

1. `Field.optional: Bool` → `Field.cardinality: Cardinality`
   (Bool is a degenerate cardinality; upgrade it)
2. `TypeExpr::Optional { inner: T }` → removed
   (optionality exits the type algebra entirely)
3. `TypeExpr::Container { kind, element }` → `TypeExpr::App`
   (containers are type application, not a special category)
4. Duplicate type definitions across modules → single definition
   in core.dag, imported by consumers

Each step is a narrowing: fewer ways to express the same concept,
converging toward one canonical representation per concept. The
sustainability improvement is not adding new capability — it's
removing redundant paths that would accumulate drift.

---

## Design Philosophy

These are the principles guiding this model. No final decisions yet —
the specific representations need further design work. But the
direction is set.

### P1: Everything typed and structured — but sustainably

Untyping something (e.g., `PR { val: Map }` in the parser) is not a
sustainability hack. It is the opposite: it defers breakage to runtime
and silently permits malformed construction. The parser should be
typed throughout.

The sustainability constraint is real though. The parser has ~100
functions, and uniform error propagation (`if is_err(pr: r) { return r }`)
currently requires a uniform return type. Typing the parser means
finding a shape that:

- Gives every production a compile-checked output type
- Preserves uniform error propagation (or replaces it with something
  equally ergonomic)
- Does not create 100 throwaway wrapper types

The actual output shapes cluster into ~12-15 categories (the parser
produces Items, Exprs, TypeExprs, Fields, Variants, Params, various
lists thereof, and a handful of intermediate values). The de facto
types already exist — `r.val.item`, `r.val.expr`, `r.val.type_expr`
are consistent field-name conventions that correspond to the AST types
in core.dag. The work is making these implicit conventions explicit and
compiler-checked.

This may take several design iterations to get right. That's fine.
The direction is: typed, not untyped.

### P2: Cardinality over proliferating optional forms

Optionality is a cardinality constraint on a binding site, not a type
constructor. The model should have ONE concept for multiplicity
(Cardinality), not three overlapping representations.

This means TypeExpr is purely "what" (the set of values). "How many"
lives at the binding site — fields, params, return positions.
`List<T>`, `Set<T>`, `Map<K,V>` are type constructors (they describe
value structure), not cardinality (they don't describe binding
multiplicity). These are orthogonal.

The precise representation of Cardinality (finite coproduct vs min/max
range) and the surface syntax for cardinality annotations are not yet
decided.

### P3: Compiler defines its own types, compositionally

The v2 compiler does not literally import from extdeps. It defines its
own domain model from scratch. But it follows the same compositional
discipline:

- **Layered**: each layer imports only downward
- **Grounded**: each type answers "what is X?" in terms of compiler
  semantics, not implementation convenience
- **Compositional**: product, coproduct, application, refinement —
  the same constructors the extdeps use
- **Single source of truth**: semantic types defined once, imported
  by consumers — no local redeclarations

A stable `std/` foundation (SourceSpan, Diagnostic, Severity,
potentially Cardinality) may eventually sit below both the compiler
and the extdeps system. The compiler is one domain in the ecosystem,
not a privileged meta-system exempt from modeling discipline.

### P4: Behavioral vocabulary compatibility (not dependency)

The compiler's service/operation model should be *compatible* with the
extdeps behavioral vocabulary (SideEffects, Determinism,
OperationBehavior) without literally importing it. The compiler defines
its own behavioral types using the same shapes and names where they
overlap. This means the compiler and extdeps can evolve independently,
but a reader familiar with one immediately understands the other.

If a shared stable `std/` layer emerges that both reference, great.
But the compiler should not be blocked on that.

## Open Design Work

### OD1: Typed parser result shape

Need to design the concrete mechanism. The leading candidate is a
typed coproduct (`ParseVal`) replacing `Map` in `PR.val`, giving ~25
variants that cover all production output categories. But this needs
to be prototyped against the actual error-propagation patterns in
parse.dag to verify it's ergonomic enough to be sustainable at 2,500
lines.

### OD2: Cardinality representation

Two candidates:

```dag
// A: Finite coproduct (simple, covers known cases)
type Cardinality = Required | Optional | Many | AtLeastOne

// B: Min/max range (more general, extensible)
type Cardinality { min: Int, max: Int? }
//   Required   = { min: 1, max: 1 }
//   Optional   = { min: 0, max: 1 }
//   Many       = { min: 0, max: none }
//   AtLeastOne = { min: 1, max: none }
```

Option A is simpler and covers all current uses. Option B is more
general (e.g., `{ min: 2, max: 5 }` for fixed-arity tuples) but may
be over-engineering for the compiler's own model. Need to survey
actual usage patterns.

### OD3: Cardinality in return position

Fields and params have natural binding sites for cardinality. Function
return types are less obvious: `fn lookup(...) -> TypeExpr?` — where
does the cardinality annotation live? Options:

- On the function definition: `return_cardinality: Cardinality`
- As part of a return-type record: `return: { type: TypeExpr, cardinality: Cardinality }`
- Implicit: return types are always Required; "optional return" is
  modeled as returning a coproduct `Found { value: T } | NotFound`

### OD4: ServiceConfig grounding

ServiceConfig needs to either faithfully model the grammar (AST level)
or be removed in favor of fields directly on ServiceDef. Currently it
contains speculative fields. The grammar should be the spec.
