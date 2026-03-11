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

Each layer below defines "what X is" without leaking how it's used.
Higher layers import from lower layers, never the reverse. Algorithm
scratch state (ParserState, KahnState) stays local and is never exported.

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

### Optionality as union with None

In the surface syntax, `T?` is sugar. In the type algebra:

```
⟦T?⟧ = ⟦T⟧ ∪ {none}
```

This can be represented as `App { name: "Option", args: [T] }` where
`Option` is a built-in type constructor, or as the compiler's internal
knowledge that `T?` means "the set ⟦T⟧ plus the none value." The key
constraint is: **one canonical representation**, not two.

Decision for v2: `Optional { inner: T }` remains a distinct TypeExpr
variant. Rationale: optionality is pervasive in the AST (`return_type:
TypeExpr?`, `default_value: Expr?`), and the `?` suffix syntax maps
cleanly to a dedicated variant. Collapsing to `App("Option", [T])` is
set-theoretically correct but adds syntactic noise in pattern matches
throughout the compiler with no semantic benefit. The emitter already
knows `Optional` means `Option<T>` in Rust.

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

### Target TypeExpr (6 variants, down from 9)

```dag
// A type expression — the compiler's representation of a type.
//
// Set-theoretically: TypeExpr is the free algebra generated by
// the six constructors below, quotiented by the subsumption rules
// in the Foundations section.
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

  // Type constructor application (subsumes Container, MapType, Optional).
  // ⟦App { name: F, args: [A₁, ..., Aₙ] }⟧ = F(⟦A₁⟧, ..., ⟦Aₙ⟧)
  //
  // Built-in constructors:
  //   "List"         : * → *           Vec<T>
  //   "Set"          : * → *           BTreeSet<T>
  //   "Map"          : * → * → *       BTreeMap<K, V>
  //   "Option"       : * → *           Option<T>
  //   "NonEmptyList" : * → *           Vec<T>  (with runtime invariant)
  //   "NonEmptySet"  : * → *           BTreeSet<T>  (with runtime invariant)
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
| `Optional` | `App` | `T?` IS `App("Option", [T])` — see decision note below |
| `Refined` | `Refine` | Renamed for verb form consistency |
| `TypeApp` | `App` | Renamed: shorter, and now the only application form |

**Decision: Optional**

Keeping `Optional` as a distinct variant vs collapsing to
`App("Option", [T])` is the one genuine choice here. Arguments:

*For collapsing (fewer variants):*
- One less match arm in every pass (6 passes × 1 arm = 6 removed)
- Canonicality: one representation per set
- The emitter already maps to `Option<T>` in Rust regardless

*For keeping (source fidelity):*
- `T?` is extremely common in the AST (14 optional fields in core.dag alone)
- `App("Option", [T])` requires string matching where `Optional { inner }` is structural
- The parser produces `Optional` directly from `?` suffix

**Recommendation**: Collapse. The emitter handles `App.name == "Option"`
in the same switch that handles `"List"`, `"Set"`, `"Map"`. One code
path, not two. The parser produces `App("Option", [T])` from `T?` —
this is a trivial desugaring. The 14 optional fields in core.dag are
field declarations (`return_type: TypeExpr?`), not TypeExpr nodes — they
are unaffected.

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

### Field, Variant, Param (unchanged)

```dag
type Field {
  name: String
  type_expr: TypeExpr
  optional: Bool
  default_value: Expr?
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
  default_value: Expr?
  span: SourceSpan
}
```

These are already correct. A variant with no fields is a unit variant
(tag-only injection into the coproduct). A variant with fields is a
payload variant (tagged product within the coproduct).

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

This is where convergence with extdeps modeling matters. The extdeps
system already has vocabulary for service semantics:

```
std/behavioral.dag:  SideEffects, Determinism, OperationBehavior
std/rate_limit.dag:  RetryPolicy, BackoffStrategy
cloud/cloud.dag:     CloudAuthScheme, ServiceEndpoint, RateLimitPolicy
```

The v2 compiler's service model should be compatible with this
vocabulary. Specifically:

```dag
// Post-typecheck service semantics.
//
// This is the compiler's understanding of what an operation does,
// derived from the AST-level OperationDef + modifiers + transport.
// It is backend-neutral — it says WHAT, not HOW.

type OperationSemantics {
  name: String
  // Transport mechanism (from AST, preserved)
  transport: TransportBinding
  // Typed inputs and outputs (TypeExprs resolved)
  inputs: List<Field>
  outputs: List<Field>
  // Behavioral properties (derived from modifiers)
  side_effects: SideEffects       // from std.behavioral
  idempotent: Bool                // from Idempotent modifier
  readonly: Bool                  // from Readonly modifier
  // Mock/test data (from AST, preserved for test emission)
  mock_response: List<MockResponseDef>
}
```

**Why this helps**: When the emitter generates transport code, it
pattern-matches on `OperationSemantics` — a well-typed semantic record
— instead of reaching back into the raw AST. When a second backend
(Python) is added, it consumes the same `OperationSemantics`. The
behavioral vocabulary (`SideEffects`, `idempotent`) comes from the
same `std/behavioral.dag` that the extdeps system uses.

**Deferred**: Full `OperationPlan`/`BackendPlan` intermediate layer.
`OperationSemantics` is the minimal version — it lifts modifier flags
to semantic fields without introducing a new IR. Adding `BackendPlan`
(which would carry Rust-specific details like "use reqwest::Client"
or "use std::process::Command") is deferred until a second backend
is added.

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

## Cross-Cutting: Convergence with extdeps

### Type vocabulary alignment

The v2 compiler's type system and the extdeps type vocabulary should
share the same constructors. After the TypeExpr consolidation:

| Surface syntax | TypeExpr | extdeps usage |
|---------------|----------|---------------|
| `String` | `Atom { name: "String" }` | Field types in all extdeps |
| `List<T>` | `App { name: "List", args: [T] }` | `List<GcpScope>` etc. |
| `Map<K,V>` | `App { name: "Map", args: [K,V] }` | `Map<String, String>` etc. |
| `T?` | `App { name: "Option", args: [T] }` | Optional fields everywhere |
| `Int where range(...)` | `Refine { base: Atom, ... }` | `HttpStatus`, `Port` etc. |
| `A \| B \| C` | `Coproduct { variants: [...] }` | `CloudAuthScheme` etc. |
| `{ a: A, b: B }` | `Product { fields: [...] }` | `ServiceEndpoint` etc. |

The compiler and the extdeps are using the same type algebra. The
compiler is just the system that understands and processes that algebra.

### Behavioral vocabulary reuse

The extdeps define `SideEffects`, `Determinism`, `OperationBehavior`
in `std/behavioral.dag`. The compiler's `OperationModifier` type
(`Idempotent | Readonly | Hermetic`) is a subset of this vocabulary.

After Layer 3 introduces `OperationSemantics`, the compiler imports
and uses the same behavioral types that extdeps do. The operation
modifier `Idempotent` in the AST maps to `idempotent: true` in the
semantics, which the extdeps system already understands.

This means: when the compiler compiles an extdeps module that declares
`idempotent` on a service operation, it produces the same semantic
annotation that it uses for its own operations. One vocabulary, shared.

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

## Open Questions

### Q1: Should `parse.dag`'s `PR { val: Map }` be typed?

The parser's generic result type uses `val: Map` (untyped). This is
the deepest modeling debt — the parser constructs AST nodes through
an untyped bag, which means malformed nodes are not caught until
runtime.

The correct model would be variant-specific return types
(e.g., `parse_type_def` returns `{ typedef: TypeDef, state }`,
`parse_fn_def` returns `{ fndef: FnDef, state }`). But this would
mean ~20 per-production result types, which conflicts with the
"eliminate wrapper types" direction.

Possible resolutions:
1. **Accept the untyped bag** — PR is algorithm scratch, not a semantic
   type. It's the parser's internal affair, not exported.
2. **Type the major productions** — `parse_item` returns
   `{ item: Item, state }` (typed), while leaf parsers keep `PR` for
   internal chaining.
3. **Return Item directly** with state threading via a different
   mechanism (continuation, accumulator).

Decision: Deferred. This is a parse.dag-internal concern that doesn't
affect the other layers. The parser's contract with the rest of the
pipeline is `parse : List<Token> → ParseResult` — as long as ParseResult
is typed, the internal representation is the parser's business.

### Q2: Should `Optional` remain a distinct TypeExpr variant?

See the detailed analysis in Layer 1 above. Recommendation: collapse
to `App("Option", [T])`. This is a modeling decision, not a correctness
issue.

### Q3: How deep should extdeps behavioral vocabulary integration go?

Layer 3 proposes importing `SideEffects` from `std/behavioral.dag`
into the compiler's `OperationSemantics`. This creates a dependency
from the compiler's semantic model to the std library's behavioral
vocabulary.

Alternative: Define `OperationSemantics.side_effects` as an independent
type within the compiler, and note the correspondence in documentation.
This avoids the import dependency but loses the shared vocabulary.

Recommendation: Import. The whole point of compositional modeling is
that vocabulary is shared, not duplicated. The compiler is a consumer
of the std library, not a peer.

### Q4: Where does the line between "AST-level ServiceConfig" and "semantic-level OperationSemantics" belong?

Currently `ServiceConfig` mixes syntactic recording (unresolved `Expr`
fields) with speculative structure (`rate_limit`, `retry` fields that
the grammar may not support yet).

The target is: AST types record exactly what the grammar allows.
Semantic types record what the compiler has derived. If the grammar
adds `rate_limit` declarations, ServiceConfig grows a field. If the
compiler infers retry behavior from modifiers, OperationSemantics
grows a field. The two evolve independently.
