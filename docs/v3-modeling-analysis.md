# v3 Modeling Analysis

> Purpose: audit every type in the v3 spec and current std/, determine
> where each belongs, and show how TransformRule dissolves into algebra.
>
> This doc is the design phase. Code changes follow from decisions here.
>
> Structure: "Current State" sections describe what exists today.
> "Proposed" sections describe what should change. Clearly separated.

## Principle

Two buckets:

1. **std/** — general concepts from math and computing. True regardless
   of this compiler. Algebra, iteration, type constructors, data flow.

2. **compiler src/v3/** — implementation details specific to THIS compiler.
   Organized per layer (parse, resolve, infer, emit). Shared types that
   multiple layers need go in a common upstream lib (like Google C++ style)
   — no dependency cycles. Each layer can depend on layers above it
   (parse feeds resolve feeds infer feeds emit) but never the reverse.
   Facts flow forward through typed boundaries, computed once.

If a type traces to a math concept or a universal computing concept,
it belongs in std/. If it's about how this specific compiler works,
it belongs in src/v3/.

---

## V3 Spec Types — Where They Belong

### Value — std/ (proposed)

A known thing. No inputs, one output. Literal, constant, data.

**Traces to:** terminal morphism in category theory. The unique
morphism from the terminal object (unit) to a type. "3" is the
morphism Unit → Int that picks 3.

**Current state:** std/syntax.dag has `LiteralValue` (the data).
std/constructors.dag describes Terminal in comments but does NOT
yet declare it as a type. Value needs both: a Terminal declaration
in constructors.dag and composition with LiteralValue.

**First consumer:** v3 parser (DAG construction).
**Deletes:** ExprLitStr, ExprLitInt, ExprLitFloat, ExprLitBool
variants from v2's ExprData enum.

### Transform — std/ (proposed)

Do something to inputs, get an output. The workhorse.

**Traces to:** morphism. Every transform is a morphism in some
algebraic category. What category and what morphism is determined
by the type's algebraic structure.

**The rule is NOT an enum.** See "TransformRule Dissolution" below.

**First consumer:** v3 parser.
**Deletes:** ExprCall, ExprFieldAccess, ExprBinOp, ExprUnaryOp,
ExprRecordLit, ExprListLit, ExprCast, ExprStringInterp — 8 of 22
ExprData variants.

### Branch — std/ (proposed)

Look at something, take a path. `if`, `match`, pattern matching.

**Traces to:** Coproduct elimination. std/constructors.dag already
declares Coproduct. Branch is its elimination form — the universal
way to consume a coproduct is to handle each variant.

**Current state:** constructors.dag has `Coproduct`. What's missing:
the explicit connection between Coproduct and Branch (the elimination
form).

**First consumer:** v3 parser.
**Deletes:** ExprIf, ExprMatch — 2 ExprData variants.

### Loop — std/ (proposed)

Repeat something, bounded.

**Traces to:** bounded recursion / iteration. std/iteration.dag
already declares fold, descend, repeat. Loop is the general form
that these are instances of.

**Current state:** iteration.dag has the three primitives.
computation.dag has the lowering table. Loop composes: an iteration
primitive with a bound and a body.

**First consumer:** v3 parser (recursive function lowering).
**Deletes:** recursive function handling spread across 04_infer.dag,
complexity.dag, ownership.dag.

### Bind — std/ (proposed)

Give something a name. Not computation — just wiring.

**Traces to:** variable binding / let-abstraction. Universal in
lambda calculus, type theory, every programming language.

**Current state:** std/binding.dag has `Binding { key, value }`.
Bind extends this with scope.

**First consumer:** v3 parser.
**Deletes:** ExprLet, ExprVar — 2 ExprData variants.

### Port — std/ (proposed)

Typed connection between behaviors. Data flows forward through ports.

**Traces to:** typed edge in a directed acyclic graph. Ports are
the edges; behaviors are the nodes. This is the fundamental data
flow concept.

**Current state:** not yet modeled. Needs a std/ definition.
Note: std/types.dag has `type Port = Int` for network ports.
Module namespacing disambiguates (`std.types.Port` vs the
data flow Port in a different module). No rename needed.

**First consumer:** v3 DAG construction + all lenses.
**Deletes:** ad-hoc edge tracking in v2's ownership.dag and
complexity.dag.

### Bound — std/ (proposed, revision)

How many times a Loop repeats. Finite by construction.

**Traces to:** well-founded measure / termination measure.

**Current state:** computation.dag has `SizeBound` — a 5-variant
coproduct (CollectionSize, TreeSize, ArithmeticParam, ExplicitCount,
Forever). This is the right concept but the coproduct should dissolve.

**Proposed dissolution:** SizeBound's 5 variants encode two facts:
a dimension (what's being counted) and a source (which parameter).
The dimension is DERIVABLE from the source's type via its algebra
profile — computation.dag already has `algebra_profile_to_dimension`
for this. Since it's derivable, don't store it:

```
type Bound {
  produced_by: PortId             // typed reference, not String
  count: Port                     // the actual number
}
```

The dimension is a FUNCTION of the type at produced_by, computed
when the cost lens needs it:
- FreeMonoid (List, String, Set, Map) → collection length
- Inductive structure (Tree) → tree node count
- OrderedRing (Int) → numeric magnitude
- Literal → constant (input-independent)
- System → Forever (externally terminated)

**produced_by is a typed reference, not a String.** The v2
retrospective identified "string-based name matching (45+ sites)"
as an architectural anti-pattern. Using `origin: String` would
reintroduce that disease — six months later the cost lens would
be doing `if origin == "items" then ...`. PortId (or BindingId)
makes "follow the edge backward" a graph operation on typed
identity.

Uses `produced_by` for consistency with Port.produced_by — same
concept ("where did this come from?") should use the same name.

**Why immediate produced_by, not full path:** if a consumer needs
to trace further back, it follows edges in the DAG. Storing the
full path would duplicate the graph structure.

### Path — std/ (proposed)

One arm of a Branch. Pattern + bindings + body.

**Traces to:** one case of a Coproduct elimination. The pattern
identifies which variant, bindings extract the components, body
handles it.

**First consumer:** v3 parser (Branch construction).

### Pattern — std/ (proposed)

What activates a path in a Branch. Structural match.

**Traces to:** destructuring / Coproduct case discrimination.

Note: std/patterns.dag is about DAG workflow patterns (ensure,
upsert) — a different concept. Pattern matching needs its own
modeling.

**First consumer:** v3 parser (match expressions).

---

## Reference Types — Where They Belong

| Type | Bucket | Reason |
|------|--------|--------|
| NodeId | src/v3/ | this compiler's identity scheme |
| PortId | src/v3/ | this compiler's identity scheme |
| TypeShape | split | constructors (Product/Coproduct) are std/; representation is src/v3/ |
| LiteralValue | std/ | already in std/syntax.dag |
| BinOp | dissolves | syntax token stays in syntax.dag; semantic operation resolves to algebra |
| UnaryOpKind | dissolves | same as BinOp |
| FieldRef | src/v3/ | this compiler's reference to a product field |
| MethodRef | src/v3/ | this compiler's dispatched application target |
| NodeRef | src/v3/ | this compiler's reference to a subgraph |
| TypeRef | src/v3/ | this compiler's reference to a type declaration |
| BindingId | src/v3/ | this compiler's naming scheme |
| BuiltinKind | src/v3/ | this compiler's set of primitives |
| ParamPort | std/ | a port with a name — parameter |

### BinOp/UnaryOp dissolution

`+` is syntax (the parser sees a token). The semantic operation is
"Ring addition" — determined by the operand type's algebra profile.
std/syntax.dag keeps OperatorSpec (token → binding power → algebra
field mapping). The compiler IR references the algebra operation,
not a BinOp enum. The parser resolves syntax to algebra at
construction time.

---

## TransformRule Dissolution

This is the most important section. TransformRule in the v3 spec is
a 14-variant enum. In v2, ExprData was a 22-variant enum and caused
665 match arms across the codebase. TransformRule is the same disease
at smaller scale.

### The insight

Every "transform rule" is the **introduction or elimination form**
of an algebraic structure that already exists in std/:

| Spec variant | Algebraic structure | Form | std/ source |
|---|---|---|---|
| Construct | Product | introduction | constructors.dag |
| FieldAccess | Product | elimination | constructors.dag |
| ListBuild | FreeMonoid | introduction | algebra.dag |
| StringBuild | FreeMonoid | introduction | algebra.dag |
| IndexAccess | FreeMonoid | elimination | algebra.dag |
| SliceAccess | FreeMonoid | elimination | algebra.dag |
| BinaryOp | Ring / Field / BooleanAlgebra | operation | algebra.dag |
| UnaryOp | Ring / BooleanAlgebra | operation | algebra.dag |
| Define | Function space | introduction | (needs modeling) |
| Call | Function space | elimination | (needs modeling) |
| Method | Function space | elimination (dispatched) | (needs modeling) |
| Builtin | Function space | elimination (primitive) | (needs modeling) |
| Cast | Type morphism | coercion | coercion.dag |

### What the compiler actually needs

Not a TransformRule enum. Instead, two facts:

1. **Which algebraic structure** is involved?
   Already declared in std/algebra.dag and std/constructors.dag.
   The type's algebra profile tells the compiler this.

2. **What form** — introduction, elimination, or operation?
   This is a 3-value structural distinction, not a 14-variant enum:
   - **Introduction:** constructing a value of this type
   - **Elimination:** deconstructing / accessing into this type
   - **Operation:** using the algebra's laws (add, negate, etc.)

The refinement details (which field? what operator? which function?)
are typed by the algebraic structure, not by a flat enum:
- Product elimination → needs field name
- FreeMonoid elimination → needs index
- Ring operation → needs operator (from algebra emergence table)
- Function elimination → needs target reference

### Why this dissolves matching code

When std/ declares a new algebraic structure with its intro/elim
forms, the compiler automatically knows how to handle transforms
involving that structure. The matching code is GENERATED from the
algebraic structure declarations, not hand-written per variant.

**v2 disease:** add ListBuild → edit 8-11 files, add ~30 match arms.

**v3 target:** add a new algebraic structure to std/ → the compiler
reads the declaration and generates the matching code. The compiler
itself has zero edits for the new structure.

Note: the 3-way intro/elim/op distinction IS still a small enum.
It's probably irreducible — these are the three fundamental things
you can do with any algebraic structure. The win is not "no enum"
but "the enum is fixed at 3 and growth happens in std/ declarations,
not compiler match arms."

### What's missing from std/ (proposed additions)

The **function space** (exponential object) isn't modeled:

```
Function space A → B:
  Introduction (Define): create a function value (params → body)
  Elimination (Apply):   call a function with arguments
```

This is the same intro/elim pattern as Product and FreeMonoid. It
should be declared in std/ alongside them. Then Define, Call, Method,
and Builtin all trace to it.

**First consumer:** v3 parser (function/lambda construction).
**Deletes:** ExprLambda variant + lambda-specific paths in v2.

### The general pattern

Every algebraic structure in std/ should declare:

```
For structure S:
  introduction:  how to construct a value of type S
  elimination:   how to deconstruct / access a value of type S
  operations:    what laws/operations emerge from S's axioms
```

Product already does this implicitly (construct = provide all fields,
project = access one field). FreeMonoid already does this implicitly
(build = unit + concat, access = index/fold). Making this EXPLICIT
in std/ is what dissolves TransformRule.

---

## std/ Audit

### What doesn't belong (current state)

Types currently in std/ that are about THIS compiler, not general:

**node.dag — compiler-specific instances.** Declares InductiveField
entries for Node, InferredNode, MatchPattern, MethodSemantics.
The CONCEPT of inductive fields is general (stays in induction.dag).
The INSTANCES for specific compiler types are compiler data.
Move to: src/v3/ analysis layer.

**graph.dag — mixed.** Graph algorithms (DFS, Kosaraju SCC) are
general. CallGraph with String keys + ProofEdge imports are
compiler-specific. Split: generalize algorithms, move specific
types to src/v3/.

**fidelity.dag — compiler-specific.** TransportClass, TestClass,
DerivedClassification. Move to: src/v3/.

### What stays (reviewer-corrected)

**computation.dag CallPattern/LoweringTarget — stays in std/.**
The bounded lowering table (every recursive pattern → bounded
primitive) is a LANGUAGE GUARANTEE (decidability), not compiler
policy. The model stays. Only compiler-specific evidence/
classification plumbing moves to src/v3/.

**effects.dag ComposedEffect/ModifierAgreement — stays in std/.**
Effect composition is algebraic (idempotency, cancellation,
redundancy are one mechanism). Track 17 says wire these into
real operations, not hollow them out. The algebra stays in std/.
Only compiler-specific reporting wrappers move.

### What's missing (proposed additions)

1. **Terminal declaration** in constructors.dag (described in
   comments but not declared as a type)
2. **Function space** (exponential) — intro/elim for functions
3. **Port** — typed data flow edge
4. **Intro/elim declarations** on existing algebraic structures
5. **Pattern matching types** (Coproduct elimination vocabulary)
6. **Lens composition algebras** (cost, ownership, effect lattices)

Each needs a first consumer and a concrete deletion target before
implementation. A type with zero consumers is a paper exercise.

---

## Lens Types — Where They Belong

The v3 spec describes 7 lenses. Their CONCEPTS are general (cost
composition, ownership tracking, effect classification). Their
IMPLEMENTATION reads compiler-specific DAG structure.

| Lens | Concept (std/) | Implementation (src/v3/) |
|------|------|------|
| Cost | CostAlgebra: add for sequence, mul for loops, max for branches | Reads specific DAG behaviors |
| Ownership | FanOut: count of consumers per port | Reads specific port edges |
| Effect | EffectLattice: Pure < Read < Write < Service | Reads specific behaviors |
| Termination | WellFoundedOrder: every Loop has a bound | Reads Loop nodes |
| Origin | produced_by: follow edges backward | Reads Port.produced_by |
| Algebra | IntroElimLaws: involution, inverse, fusion | Reads adjacent transforms |
| Space | same composition as Cost but tracks allocation | Reads specific DAG behaviors |

The composition rules (add, max, multiply for cost; lattice join for
effects) are math — they belong in std/. The specific traversal of
THIS compiler's DAG structure is src/v3/.

### Closure rule (from Experiment 1)

When a Define has an edge into a Loop, the Define's free variables
inherit the Loop's bound for fan-out, and the Define's body is
evaluated under the Loop's termination context, not the enclosing
function's.

This affects two lenses:
- **Ownership:** captured values have fan-out = Loop's bound (N),
  not 1. They're used N times → borrow or clone, not move.
- **Termination:** self-calls inside the Define are bounded by
  the Loop, not the enclosing function's recursion.

Without this rule, closures compile but the ownership lens gets
fan-out wrong — treats captures as fan-out=1 instead of fan-out=N.
This manifests as either over-cloning (defensive) or aliased
mutation (unsound). Neither is acceptable.

### Where do lenses live in the layering?

Lenses are NOT a separate post-infer pass. They are computed at
binding sites during DAG construction (option b from the retro).
This is the Experiment 2 lesson: carry facts through bindings,
don't reconstruct them afterward.

Concretely: when the parser/infer layer creates a Bind, it
computes the lens facts (fan-out, effect, cost) for that binding
and stores them on the Port. Lenses are fields, not passes. A
downstream consumer reads them — no re-derivation.

If lenses were a separate pass (option a), we'd recreate the v2
problem: construct facts during inference, discard them, then
rebuild them in a 5,000-line analysis. If they were external
scripts (option c), compile-time enforcement claims fall apart.

This means the infer layer owns the lens COMPUTATION, and the
emit layer READS the results. The lens algebra (how costs compose,
how effects join) lives in std/. The specific computation lives
in src/v3/infer.

---

## Compiler Layer Breakdown (src/v3/)

Layers depend downward only (parse → resolve → infer → emit).
Shared types that multiple layers need go in a common upstream lib.
No dependency cycles. All layers import from std/.

### tokenize — source text → tokens

**Owns:** TokenizerState, ScanResult

**Imports from std/:**
- SourceSpan (types.dag) — byte offsets for source locations
- LiteralValue (syntax.dag) — classified literal data
- OperatorSpec, SyntaxSpec (syntax.dag) — operator/keyword tables

**Reusable from v2 (proven, no rework needed):**
- Token { text, span, shape } — payload-insensitive shape + text
- TokenShape — flat enum of structural classifiers (35 variants)
- SourceRef { file, text, source_chars } — pre-decomposed code points
- NewlineIndex — O(log n) byte-offset → line:col translation
- String interpolation multi-token model (StrBegin/StrMid/StrEnd)
- Keyword open-set via SyntaxSpec map lookup

**Unicode handling (solved in v2):**
The tokenizer pre-decomposes UTF-8 to a List<Int> of code points
at the I/O boundary (once, O(n)). All internal scanning uses O(1)
list indexing on code points. This eliminated O(n^2) behavior on
non-ASCII input that caused OOM on large files. The pattern is:
compute encoding facts once at boundary, never re-derive.

Character positions (for scanning) are code point indices.
SourceSpan offsets are byte positions (for source text slicing).
Conversion happens at tokenize() entry. No mixing.

### parse — tokens → DAG with L1 behaviors

**Owns:** ParseContext, ParseResult, and per-construct result types

**Imports from std/:**
- L1 behavior types (Value, Transform, Branch, Loop, Bind)
- Port — typed data flow edges
- Algebraic structure declarations — to determine intro/elim forms
- SyntaxSpec — grammar tables

**The parser's job in v3:**
Read tokens, look up type declarations from std/ to determine
algebraic structure, and emit L1 behavior nodes with Ports:
- `Person { name: "alice" }` → Product introduction (constructors.dag)
- `person.name` → Product elimination (constructors.dag)
- `[1, 2, 3]` → FreeMonoid introduction (algebra.dag)
- `items[0]` → FreeMonoid elimination (algebra.dag)
- `x + y` → resolve `+` via OperatorSpec → algebra field → Ring operation
- `x => x + 1` → Function space introduction
- `f(x)` → Function space elimination
- `if cond then a else b` → Coproduct elimination (Branch on Bool)

No TransformRule enum. The parser reads std/ declarations.

**v2 parser rework needed:**
v2's parser uses integer position into a token list. The structural
parser design (parser-design.md) proposes list consumption instead,
which gives the complexity analyzer structural descent evidence.
v3 should start with list consumption from day one.

### resolve — names → declarations

**Owns:** ModuleGraph, ResolvedModule, ResolvedImport

**Imports from std/:**
- Module/import vocabulary
- Type declarations (to build the type environment)

**Job:** map string names to actual type/function declarations.
Wire imports. Detect cycles. Build the type environment that
parse and infer read from.

### infer — propagate types through ports

**Owns:** InferScope, TypeBinding, inference result types

**Imports from std/:**
- Algebraic structure profiles — to determine valid operations
- Coercion vocabulary — to validate casts
- Type constructors — to check structural compatibility

**Job:** every Port gets a value_type. The DAG already has the
structure from parse; inference fills in the types and validates
that every port connection is type-compatible.

### emit — DAG + LanguageSpec → target source code

**Owns:** EmitResult, rendering state, per-target formatting

**Imports from std/:**
- LanguageSpec (languages.dag) — syntax templates per target
- Coercion data (coercion.dag) — type rendering rules
- AlgebraFieldTemplate (algebra.dag) — operation rendering

**Job:** read behaviors, read lens results, render target code.
Pure translation — no semantic decisions. The emitter asks:
"how does this target language spell Product introduction?"
and reads the answer from LanguageSpec data.

---

## Intro/Elim and Coercion

Intro/elim forms and coercion are related but distinct:

**Intro/elim** = structural operations on a type's algebraic form.
"How do I build a List?" (FreeMonoid introduction.)
"How do I access a struct field?" (Product elimination.)

**Coercion** = mapping between types, especially across the
.dag → target language boundary.
"How does .dag Int render in Rust?" (TypeCheckpoint: i64.)
"How does .dag List<T> render in Rust?" (InhabitantDecl: Vec<{0}>.)

**The connection:** when the emitter sees a Product introduction,
it needs to know how to render the constructed type in the target.
The intro/elim form tells it WHAT operation. The coercion system
tells it HOW to render the types involved.

### v2's coercion system (current state — already data-driven)

v2 uses a two-stage data lookup:

1. **TypeCheckpoint (fast path):** direct .dag name → target type.
   `Int → i64`, `String → String`, `Bool → bool`.
   Includes metadata: is_copy, literal_suffix, default_expr.
   Lives in extdeps/languages/*/types.dag — per-target instances
   of schema defined in std/coercion.dag.

2. **InhabitantDecl (algebra fallback):** when no checkpoint matches,
   resolve via the type's algebra profile.
   `List<T>` inhabits FreeMonoid → renders as `Vec<{0}>` in Rust.
   `Set<A>` inhabits BooleanAlgebra → renders as `BTreeSet<{0}>`.
   Includes identity_expr, import_path, arity.

3. **Cast validation:** two levels.
   - .dag level: dag_cast_rules in std/coercion.dag (numeric types only)
   - Per-target: CastSyntax.cast_rules (Rust strict, Python/Go fail-open)

**What v3 gets for free from the DAG:**
Since every transform traces to an algebraic structure, and the
coercion system maps algebras to target types, the rendering chain
is automatic:

```
Parser sees [1, 2, 3]
  → FreeMonoid introduction (from algebra.dag)
  → element type Int (from inference)
  → FreeMonoid<Int> (structural)
  → InhabitantDecl: Vec<{0}> with {0}=i64 (from coercion)
  → Rust: vec![1_i64, 2_i64, 3_i64] (from LanguageSpec)
```

No special "ListBuild" emit path. The algebra + coercion data
drives the entire chain.

---

## String Interpolation — Dissolves Into Algebra

String interpolation is not a special operation. It is FreeMonoid
concat with coercion. No special ExprData variant, no dedicated
emit path.

### How v2 handles it (the disease)

v2 treats `ExprStringInterp` as one of 22 ExprData variants. It has
dedicated match arms in emit (2 rendering functions), ownership,
and complexity. The tokenizer emits StrBegin/StrMid/StrEnd tokens.
The parser builds a `StringPart` list (Text | Interpolation).
The emitter reads a `string_interp` template from LanguageSpec.

The tokenizer model (StrBegin/StrMid/StrEnd) is reusable — it's
just token classification. The ExprStringInterp variant is not.

### How v3 handles it (the dissolution)

The parser desugars string interpolation to FreeMonoid concat +
coercion transforms during DAG construction. No special variant.

**Example 1: all parts are String**

```
let name = "Alice"
let msg = "Hello {name}"
```

→ Two String values, FreeMonoid concat:
```
Value("Hello ") → port<String>
Bind(name) → port<String>
Transform(FreeMonoid concat, [port<String>, port<String>]) → port<String>
```

No coercion needed. Both parts already inhabit FreeMonoid<Char>.

**Example 2: mixed types (Int → String coercion)**

```
let age = 30
let msg = "age is {age}"
```

→ `age` is Int (OrderedRing), target is String (FreeMonoid<Char>).
Int's algebra provides `to_string` (Ring → FreeMonoid morphism):
```
Value("age is ") → port<String>
Bind(age) → port<Int>
Transform(Ring→FreeMonoid coercion, [port<Int>]) → port<String>
Transform(FreeMonoid concat, [port<String>, port<String>]) → port<String>
```

The coercion is a transform like any other. The coercion system
already handles Int→String via algebra profiles.

**Example 3: nested expression**

```
let msg = "total: {a + b}"
```

→ `a + b` is a Ring operation producing Int, then coerced:
```
Bind(a) → port<Int>
Bind(b) → port<Int>
Transform(Ring addition, [port<Int>, port<Int>]) → port<Int>
Transform(Ring→FreeMonoid coercion, [port<Int>]) → port<String>
Value("total: ") → port<String>
Transform(FreeMonoid concat, [port<String>, port<String>]) → port<String>
```

Three transforms, all from existing algebra. No special case.

**Example 4: multiple interpolations**

```
let msg = "Hello {name}, you are {age} years old"
```

→ Desugars to a chain of FreeMonoid concats:
```
concat("Hello ", name, ", you are ", to_string(age), " years old")
```

Which is associative (FreeMonoid axiom), so the compiler can
group however it wants. Left-to-right fold, balanced tree, etc.

### Rendering optimization

Target languages have optimized string interpolation syntax:
- Rust: `format!("{}{}", ...)`
- Python: `f"Hello {name}"`
- Go: `fmt.Sprintf("Hello %v", name)`

The emitter can recognize "a chain of FreeMonoid concats where
some inputs are coerced" and use the optimized syntax. This is a
rendering optimization in LanguageSpec, not a semantic distinction.
The DAG is still just concat + coercion.

### Bootstrapping

No bootstrapping issue. v2's tokenizer (StrBegin/StrMid/StrEnd)
feeds v3's parser unchanged. The difference is entirely in how
the parser builds the DAG from those tokens — v3 desugars to
concat + coercion instead of building ExprStringInterp nodes.

---

## Forever and Bounded Iteration

### What "forever" means

In a Bit/Word64 system, there is no infinity. `true` is a Bit
(a signal, a data value), not a philosophical statement about
truth. Classical logic True (a proposition that holds) exists at
a different layer — it's in the logic, not the data.

A "forever" loop means: "run until externally terminated." The
developer wants "until the heat death of the universe" — this is
an engineering system, not a mathematical proof. The system models
this honestly: Forever is the largest representable bound, which
means "the system guarantees the loop body executes; termination
comes from outside (process killed, signal received, resources
exhausted)."

Forever is NOT "iterate exactly 2^63 times." It is a named
sentinel meaning "externally terminated." The cost algebra uses
it for worst-case reasoning, not as an iteration target.

### Why not float infinity?

Float infinity would lose information:
- infinity × body_cost = infinity (can't distinguish cheap vs
  expensive forever loops)
- infinity × infinity = infinity (nested costs collapse)
- 2^63 × 2^63 = 2^126 is a meaningful number for nested analysis
- infinity breaks decidability (not representable in Word64)

The integer sentinel preserves cost algebra precision. The cost
lens can report "this forever loop costs Forever × O(n) per
iteration" — meaningful for comparing alternatives.

### Parallelism safety

Can the system accidentally parallelize a forever loop and "finish"
it? No — but the guarantee is structural + effect, not structural
alone.

**Structural argument (covers most cases):**
1. Parallelism requires independence — no shared edges between
   iterations in the DAG.
2. Forever loops typically have accumulator state (server state,
   event loop state) that persists across iterations.
3. If iterations WERE independent, the compiler raises to Map.
   But Map requires a finite source collection. Forever isn't a
   collection — it's a repeat bound.
4. `repeat(Forever, f)` is sequential by construction — each
   iteration feeds the next.

**Edge case: stateless effectful forever loop.**
`repeat(Forever, fn() { handle_request() })` where handle_request
reads from a shared queue. There's no value-level accumulator —
each iteration is independent at the data flow level. The
dependency is effectful (the queue), not structural.

The safety net here is the **effect lens**: handle_request has a
ServiceCall or Mutation effect, so the compiler refuses to
parallelize because effects don't generally commute.

This means parallelism safety for forever loops DEPENDS on the
effect lens being correct. If the effect lens has a hole,
parallelism over a forever loop becomes possible. The effect lens
is load-bearing for parallelism — not optional.

The fully pure case (`repeat(Forever, fn() { 1 + 1 })`) is safe
even if "parallelized" — there's nothing to compute and no
observable effect. Dead code.

### Composition cases

- **Forever containing fold:** `repeat(Forever, fn(state) { items |> fold(...) })`
  Cost = Forever × |items| × body. Sequential outer, inner fold
  may parallelize if independent.

- **Forever inside branch:** `if cond then repeat(Forever, ...) else x`
  Cost = max(Forever × body, cost(x)). Branch is exclusive.

- **Fold containing forever:** structurally invalid. You can't fold
  over elements where each runs forever — the inner loop must
  terminate to produce a value for the accumulator.

- **Map over forever:** impossible. Map requires a finite source.

### Recursion and forever

Mutual recursion lowers to a single Loop with a phase tag (v3 spec
Scenario 2). So mutual recursion composes with the same primitives
as everything else. The phase tag is data, not structure.

Self-call with same argument → `repeat(Forever, ...)`. The bound
is honest: this might run for 292 years at 1ns/iteration. The cost
lens reports it.

**Testing strategy:** the lowering boundary (surface syntax → DAG
with Loop+Bound) is where the real complexity lives. Two layers:

**Seed cases (hand-written TDD, for understanding):**
- Single self-call → Loop with correct bound
- Mutual recursion (A→B→A) → single Loop with phase
- 3-way mutual recursion (A→B→C→A) → single Loop with 3-phase
- Self-call with same argument → repeat(Forever)
- Mutual recursion with same argument (ping/pong with no descent)
  → repeat(Forever) — must detect the implicit forever loop
- Recursion with accumulator vs without (different ownership)
- Recursion inside a fold body — legal (bounded by fold × descent)
  but composition must be correct
- Mixed: mutual recursion where one branch terminates → Loop + Branch

**Property tests (randomized, the real safety net):**

Hand-written cases are seeds for understanding. Randomized property
tests cover the space that hand-written cases miss — N-way cycles,
mixed descent patterns, deeply nested compositions.

Properties to verify:

1. **Lowering totality:** for any randomly generated call graph
   (N functions, random call edges, random descent/non-descent),
   the lowering step ALWAYS produces a Loop with a finite Bound.
   Never crashes, never returns "unknown."

2. **Bound correctness:** if all calls in a cycle pass arguments
   unchanged → bound = Forever. If any call descends on its
   argument → bound = TreeSize or CollectionSize. The bound type
   matches the call pattern.

3. **Cost composition finiteness:** for any random nesting of
   Loop/Branch/Transform, the cost algebra produces a finite cost
   expression. No infinity, no NaN, no overflow in the algebra.

4. **Parallelism conservatism:** for any randomly generated DAG,
   if the system marks something parallelizable, every iteration
   is truly independent (no shared edges to accumulator or prior
   iterations). False negatives (missing parallelism) are safe.
   False positives (incorrect parallelism) are bugs.

5. **Ownership consistency:** for any random DAG, fan-out counts
   are consistent with actual edge counts. No port has fan-out=1
   (move) but is actually used twice.

Generation strategy: build random call graphs by:
- Picking N functions (2-20)
- For each, randomly choosing 0-3 call targets from the set
- For each call, randomly choosing descent (pass child) or
  non-descent (pass same arg or unrelated arg)
- Randomly nesting some functions inside fold/branch contexts

This covers N-way mutual recursion, mixed patterns, and edge
cases far better than enumeration. The hand-written seed cases
verify human understanding; the property tests verify the system.

---

## V2 Coproducts to Dissolve in V3

The doc audited new L1 behaviors but not existing v2 coproducts.
If v3 inherits these, the same audit happens again under "v4."
Here's the inventory from 00_core.dag:

| Type | Variants | Dissolution path |
|---|---|---|
| ExprData | 22 | Already addressed — becomes L1 behaviors + algebraic forms |
| TokenShape | 35 | Doc says "reusable" — but 35 variants deserves audit. Keyword tail may compress. Revisit during v3 tokenizer. |
| Connective | 4 (Conj, Disj, NoConnective, Arrow) | Traces to Product, Coproduct, Terminal, Function. Once those are first-class in std/, Connective is redundant. |
| FieldAccessStyle | 5 (StoredField, EnumAccessor, OptionalUnwrap, TupleFirst, TupleSecond) | All 5 are intro/elim instances: TupleFirst/TupleSecond = positional Product elimination, StoredField = named Product elimination, OptionalUnwrap = Optional elimination, EnumAccessor = Coproduct projection. Dissolves via the same intro/elim mechanism. |
| MethodSemantics | 3 (Plain, Algebra, Service) | God-coproduct in miniature. AlgebraMethodSemantics carries 4 fields, ServiceMethodSemantics carries 2. Algebra method = elimination of an algebraic structure, Service method = elimination across a transport boundary. |
| InferredNode | 3 (Resolved, CompilerError, TypeVariable) | Three states of one process. CompilerError doesn't belong in the result type — should live in a diagnostic table keyed by NodeId. TypeVariable is unification scratch state. Probably reduces to Optional<Node> plus side tables. |
| VarBindingKind | 4 (Local, Function, Variant, MatchBound) | Origin tags — structurally indistinguishable, different labels. "Label is data, not structure" pattern. Dissolves into a single binding type with origin as data. |
| ExprErrorKind | 3 | Classification metadata. Single field on unified error type. |

**Priority for v3:** Connective and FieldAccessStyle dissolve
naturally when intro/elim forms land (they're just unmodeled
instances). MethodSemantics dissolves when algebraic and transport
operations are unified under the function space. InferredNode and
VarBindingKind dissolve when diagnostics and bindings are redesigned.

Not all need to dissolve on day one. But they need to be tracked
so they don't get inherited by default.

### Why 04_infer fragmented into 13 files

v2's stage 4 has 13 sub-files (access, cycle, emit_info, env,
infer, items, lookup, method, patterns, resolve, service, sigs,
types). This happened because infer has too many concerns:

- Name resolution (should be a separate layer — it is in v3)
- Type checking (the actual job)
- Method dispatch (really algebra profile lookup)
- Pattern matching validation (really Coproduct elimination check)
- Service wiring (really transport boundary check)
- Cycle detection (really call graph analysis)
- Emit info construction (should be emit's job)

v3 splits this: resolve is its own layer, method dispatch reads
algebra profiles, pattern matching reads Coproduct structure,
service wiring reads transport declarations. What remains in infer
is type propagation through ports — a focused job. If v3's infer
starts growing sub-files, it's a signal that a concern belongs
elsewhere.

---

## Experiment Status

These experiments (docs/v3-validation-experiments.md) validated
the v3 direction. Status matters because the doc builds on them:

| # | Experiment | Result | Status |
|---|---|---|---|
| 1 | Lambda → Bind + Define | **PASS (partial)** | LambdaSemantics deleted, -30 lines. 43 ExprLambda refs remain — justified by closure semantics (fresh scope, capture fan-out, iteration context). Lesson: the closure/Define-in-Loop rule (above). |
| 2 | Provenance on binding | **PASS (partial)** | Carry path works: classify_let_value reads scope_locals first (carried facts) before sub_value_vars (reconstruction). **Reconstruction not yet fully deleted** — the old path runs in parallel. This is a parallel implementation, not a dissolved heuristic. The dividend is enabled but not banked. |
| 3 | Add clamp builtin | **PASS (full)** | 3 files edited, zero consumer edits. Validates rule-table mechanism. |
| 4 | Purity lens | **PASS (full)** | 1 new file, zero compiler changes, 3117 pure / 36 effectful. Validates observational lens mechanism. |
| 5 | ExprData variant cost | **MEASURED** | 8-11 files, ~30 match arms per new variant. Validates the disease is real. |

**Experiment 2 is the honest partial.** The carry path works at
the substrate level, but the old reconstruction hasn't been deleted.
Until at least one reconstruction function is DELETED (not just
bypassed), the v3 spec's "lenses read physics" claim rests on a
half-proof. The mechanism is validated; the full dividend is not.

All experiments keep 415 tests green and CX ratchet stable.

---

## Summary: What Changes

### std/ gains (proposed, each needs first consumer):
1. Terminal declaration in constructors.dag
2. L1 behavior type declarations (Value, Transform, Branch, Loop, Bind)
3. Port — typed data flow edge
4. Function space (exponential) — intro/elim for functions
5. Explicit intro/elim declarations on existing algebraic structures
6. Pattern matching types (Coproduct elimination vocabulary)
7. Lens composition algebras (cost, ownership, effect lattices)

### std/ loses (proposed):
1. node.dag compiler-type instances → src/v3/
2. graph.dag compiler-specific types → src/v3/
3. fidelity.dag → src/v3/

### std/ stays (reviewer-confirmed):
1. computation.dag CallPattern/LoweringTarget — language guarantee
2. effects.dag ComposedEffect/ModifierAgreement — algebraic

### TransformRule dissolves:
From 14-variant enum → algebraic structure + form (intro/elim/op).
The 3-way form distinction is a small, irreducible enum. The win:
growth happens in std/ declarations, not compiler match arms.

### SizeBound dissolves:
From 5-variant coproduct → Bound { produced_by: PortId, count: Port }.
Dimension is derivable from the type at produced_by — it's a
function, not a stored field. produced_by is a typed reference,
not a String (avoids reintroducing string-matching anti-pattern).

### BinOp/UnaryOp dissolve:
Syntax token stays in syntax.dag. Semantic operation resolves to
algebra at construction time.

### Compiler lives under src/v3/:
Layers depend downward only. Shared types in upstream lib. No
dependency cycles. Facts flow forward, computed once.

---

## Open Questions

1. **How much of this is v3 vs v2-compatible?**
   The std/ declarations (intro/elim on structures, function space,
   port) can be added now without changing v2. The compiler reading
   them is v3 work. The std/ audit (moving compiler types out) can
   happen incrementally.

2. **Parser state model:** v2 uses integer position. v3 should use
   list consumption for structural descent evidence. Design the
   ParseState type before implementing.

3. **Coproduct minimization:** any remaining coproducts in std/
   should be audited for dissolution opportunities. If a coproduct's
   variants trace to different algebraic structures, it can likely
   dissolve into algebra-derived data.
