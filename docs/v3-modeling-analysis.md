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

## Coproduct Dissolution Principle

### The t-shirt analogy

A coproduct is a t-shirt size — a categorical compression of a
richer coordinate space. S/M/L compresses (chest × length ×
shoulder) into a tag because garment manufacturers don't own both
sides of the transaction: they don't know who buys the shirts,
and the cost of communicating full dimensions exceeds the benefit.

**We own both sides.** We are the only manufacturer (the compiler
generates the code) and the only consumer (the compiler reads its
own model). Communication cost is zero because the model generates
the access code. So every economic reason for categorical
compression disappears.

Every coproduct in v2 is a t-shirt size laid over a space we
defined:
- TokenShape::ShPlus is the "M" of (operator family × arity ×
  algebraic structure × precedence)
- InferredNode::CompilerError is the "L" of (inference succeeded ×
  failure kind × who needs to know)
- SizeBound::CollectionSize is the "XL" of (algebraic structure ×
  parameter × count)

The t-shirt sizes exist not because they're right, but because
whoever designed them was importing habits from codebases where
categorical compression was rational — where someone else would
maintain the matching code. In our system, that instinct is wrong.

### The dissolution test

When encountering a coproduct, ask: **what's the underlying
coordinate space this is a tag system over?**

If you can name the space — "this is really (family × arity ×
position)" — the coproduct dissolves into the fact-level model
of that space.

If you can't name any underlying space — the variants truly are
an unordered set of distinct things with no common coordinates —
it's a genuine terminal.

### The stopping rule

**Dissolve until you hit the user-input boundary.**

Anything on the user's side of that boundary is terminal because
it's not ours to define: identifiers (arbitrary strings the user
chose), literal values (arbitrary bits the user wrote), source
spans (byte offsets into files). These are atoms because the
user's choice is input from outside our closed world.

Anything on the compiler's side is structural because we defined
it. Structure we defined is always coordinate-expressible. So
everything on the compiler side dissolves.

### Watch for lossy compression

Some categorical compressions lose information that matters.
v2's TypeBinding = {name, resolved} compressed away provenance
during inference, and complexity.dag rebuilt it from heuristics —
5,000 lines of reconstruction because the coordinate-level fact
was compressed away at source. The audit should ask: **what
information is this coproduct losing compared to the coordinate
model?** If the lost information is being reconstructed downstream,
dissolution isn't just cleaner — it's required for correctness.

### Why people keep doing it

Categorical thinking is cognitively cheaper than coordinate
thinking. "It's an L" is easier to hold in your head than "42"
chest × 28" length × 18" shoulder." The compression is valuable
to the designer at design time, even when it's costly to the
system at runtime.

In a generated-code system, the coordinate model lives in the
spec (expensive to design once), the categorical ergonomics live
in query helpers on top of it (cheap to generate), and nobody
holds both in their head at once. The hard work happens once.

### Three dissolution patterns

**1. Fact placement** — "one field, multiple consumers, different
needs." The coproduct exists because the substrate didn't give
each concern a separate home. Add the substrate (ports, tables,
algorithm-internal state) and the coproduct dissolves.

Example: InferredNode (Resolved | CompilerError | TypeVariable)
→ Port.value_type carries the type, diagnostic table carries
errors, unification is algorithm-internal. Three questions, three
places, no coproduct.

Test: if a coproduct's variants serve different consumers or answer
different questions, it's a missed fact placement.

**2. Variant-is-data** — "one consumer, one question, but variants
are really data on a single structural shape." The variants differ
in WHAT (which operator, which keyword) but not in HOW (same parse
dispatch, same emission pattern). Compress to one structural shape
with the variation as data in a table.

Example: TransformRule 14 variants → algebraic structure + form.
Keywords → ShKeyword with identity in Token.text.

Test: if consumers handle every variant identically except for a
data lookup, the variant is data, not structure.

**3. Algebraic-form** — "the variants trace to introduction or
elimination forms of known algebraic structures." The structures'
std/ declarations generate the dispatch. This is variant-is-data
specialized to algebra.

Example: Construct = Product intro, FieldAccess = Product elim,
ListBuild = FreeMonoid intro. All trace to std/algebra.dag.

**4. Dimensional** — "a flat coproduct of N variants hides an
M-dimensional structure." The variants are points in an M-space.
Replace the flat enum with a record whose fields are the
dimensions, each dimension being a small coproduct. Queries along
any single dimension become field reads instead of N-way matches,
and the structure grows along known axes, not unboundedly.

Example: 6 delimiter variants (LBrace, RBrace, LParen, RParen,
LBracket, RBracket) → Delimiter { shape: BracketShape, side: Side }
where BracketShape = Curly | Round | Square and Side = Open | Close.
"Is this any closer?" becomes `side == Close` instead of matching
three variants.

Each dissolution needs the right pattern. Running the wrong pattern
gives a worse result — e.g., applying fact placement to
TransformRule would scatter transform rules across the DAG and
lose their shared machinery.

### Default: dissolve at first sight

Every coproduct is dissolvable until proven otherwise. The burden
is on PROVING irreducibility with a structural argument ("these
cases share no common fields or dimensions"), not an operational
one ("the parser does different things with them"). Operational
arguments describe current code; structural arguments describe
the model. The project is about the second.

**"Single consumer" is not a defer signal — it's an urgency signal.**

In hand-written codebases, a single-consumer coproduct looks safe
to defer: low pain, low priority. But in a generated-code system,
this reasoning is inverted:

- Dissolution cost = model editing (cheap, constant over time)
- Deferral cost = consumer accretion (expensive, monotonically
  increasing — new consumers bolt on because the coproduct exists)
- Therefore: the cheapest moment to dissolve is always NOW

ExprData is the existence proof. It started as the parser's
discriminator. One consumer. Looked fine. Five years later:
22 variants, 665 match arms, 7 consumers, thousands of lines
to unwind. Nobody designed a god-coproduct — it accreted because
each new consumer was marginally easier to bolt on than to
redesign. "Single consumer" was a temporal accident, not a
design property.

**Priority order (opposite of instinct):**
1. Single-consumer coproducts first — cheapest, highest risk
2. Few-consumer coproducts next — before they grow
3. God-coproducts last — wait for big-bang moments (like v2→v3)

**Criterion:** not "is this a design blocker?" (that only catches
things that stop v3 from being built). The criterion is: "does
the argument for keeping this rely on current consumer count, or
on structural irreducibility?" If the former, dissolve. If the
latter, accept.

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
a 13-variant enum (v3-spec.md lines 82-119). In v2, ExprData was a
22-variant enum and caused 665 match arms across the codebase.
TransformRule is the same disease at smaller scale.

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
forms, the compiler can handle transforms involving that structure
by reading the declaration. The matching code is generated from the
algebraic structure declarations, not hand-written per variant.

**v2 disease:** add ListBuild → edit 8-11 files, add ~30 match arms.

**v3 target:** add a new transform WITHIN an existing algebraic
structure → zero consumer edits (validated by Experiment 3). Adding
an entirely new algebraic structure is a stronger claim — it requires
the structure to map to one of the three iteration primitives (fold,
descend, repeat) and to have intro/elim forms that the emitter can
render via LanguageSpec. This is the expected path but is not yet
validated by experiment.

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

**fidelity.dag — stays in std/ (reviewer-corrected).** TransportClass
and TestClass are shared transport vocabulary, not compiler policy.
Part of extdeps layer model (fermi → fidelity → test_policy). The
file explicitly says "no repo-specific policy here." Stays in std/.

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

### Diagnostic table invariant

Port.value_type is `Option<TypeShape>`. None means "inference did
not produce a type for this port." The invariant:

> **If Port.value_type is None, the diagnostic table MUST have an
> entry for the node that produced this port.**

Emit never reads value_type without this invariant guaranteed
upstream. There is no "check the table and see" — the invariant
is structural. If value_type is Some, the type is trustworthy.
If it's None, the diagnostic explains why. No third state.

This prevents the "two sources of truth" problem: value_type and
the diagnostic table are not independent — they're linked by this
invariant. Upstream (infer) enforces it. Downstream (emit) relies
on it.

TypeVariable is algorithm-internal scratch state. It never appears
on any Port or in the diagnostic table. If unification fails,
the algorithm writes a diagnostic and leaves value_type as None.
Debugging unification failures uses the diagnostic table, not IR
inspection — different affordance than v2, but the diagnostic
message should include the unification state that failed.

### Parse vs resolve boundary

The doc says parse emits L1 behaviors by looking up type
declarations, and resolve builds the type environment. This is
NOT a simple left-to-right pipeline. Concretely:

1. **Parse (surface):** tokens → surface AST. No type knowledge.
   Produces structural nodes (expressions, declarations, patterns).

2. **Resolve:** surface AST → resolved AST. Maps names to
   declarations, builds type environment, wires imports. This is
   where the compiler learns WHAT types exist and what algebraic
   structures they inhabit.

3. **Lower (part of resolve or a sub-step):** resolved AST → DAG
   with L1 behaviors. NOW the compiler knows the algebraic
   structures and can emit intro/elim forms. `[1,2,3]` becomes
   FreeMonoid introduction because resolve established that List
   inhabits FreeMonoid.

4. **Infer:** DAG → typed DAG. Propagates types through ports.

The key: parse does NOT look up algebraic structures. Resolve does.
Parse produces surface structure; resolve lowers to L1 behaviors
using the type environment it built. This avoids the question of
"who owns type knowledge" — resolve owns it, exclusively.

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
- TokenShape — temporary carry-forward for bootstrap continuity.
  Subject to structural decompression (35 → ~9) before v3
  canonizes it. See coproduct audit.
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

### parse — tokens → surface AST

**Owns:** ParseContext, ParseResult, and per-construct result types

**Imports from std/:**
- SyntaxSpec — grammar tables, operator specs

**The parser's job in v3:**
Tokens → surface AST. No type knowledge. Produces structural nodes
(expressions, declarations, patterns). The parser does NOT look up
algebraic structures — that's resolve's job.

**v2 parser rework needed:**
v2's parser uses integer position into a token list. The structural
parser design (parser-design.md) proposes list consumption instead,
which gives the complexity analyzer structural descent evidence.
v3 should start with list consumption from day one.

### resolve — names → declarations → L1 lowering

**Owns:** ModuleGraph, ResolvedModule, ResolvedImport, type environment

**Imports from std/:**
- L1 behavior types (Value, Transform, Branch, Loop, Bind)
- Port — typed data flow edges
- Algebraic structure declarations — to determine intro/elim forms
- Module/import vocabulary

**The resolve layer's job in v3:**
1. Map names to declarations, wire imports, detect cycles
2. Build the type environment (learn what types exist, what
   algebraic structures they inhabit)
3. Lower surface AST → DAG with L1 behaviors, using the type
   environment to determine algebraic forms:
   - `Person { name: "alice" }` → Product introduction
   - `person.name` → Product elimination
   - `[1, 2, 3]` → FreeMonoid introduction
   - `items[0]` → FreeMonoid elimination
   - `x + y` → resolve `+` via OperatorSpec → Ring operation
   - `x => x + 1` → Function space introduction
   - `f(x)` → Function space elimination
   - `if cond then a else b` → Branch on Bool

No TransformRule enum. Resolve reads std/ declarations.

Resolve owns type knowledge exclusively. Parse doesn't know types.
Infer doesn't discover types — it propagates them through ports.

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
| TokenShape | 35 | Dimensional dissolution (pattern 4). See analysis below. |
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

### TokenShape dimensional dissolution (pattern 4)

TokenShape has 35 flat variants. One consumer (parser), one question
("what kind of token?"). Not a fact placement problem. But the flat
list hides dimensional structure — many variants are points in a
small M-dimensional space, not genuinely atomic.

**Operators (18 variants → 1).** Pattern 2 (variant-is-data).
The parser already dispatches operators by text via
`find_operator_bp(symbol: t.text)`, not by shape. ShPlus, ShMinus,
ShStar, etc. are dead structural weight — the real dispatch is
OperatorSpec. Collapse to ShOperator; the text + OperatorSpec table
does the work.

**Delimiters (6 variants → 1).** Pattern 4 (dimensional).
LBrace/RBrace/LParen/RParen/LBracket/RBracket are a 3×2 space:
```
Delimiter { shape: BracketShape, side: Side }
BracketShape = Curly | Round | Square
Side = Open | Close
```
"Is this any closer?" → `side == Close` (field read, not 3-way match).
"Does this closer match that opener?" → `opener.shape == closer.shape`.
Currently requires 3 separate `is_rbrace/rparen/rbracket` predicates.

**Literals (3 variants → 1).** Pattern 4 (dimensional).
LitStr/LitInt/LitFloat share the same structure — a token whose
text carries a literal value. The parser's job is to produce a
LiteralValue (already in std/syntax.dag). Collapse to:
```
Literal { kind: LiteralKind }
LiteralKind = Str | Int | Float
```
"Is this any literal?" → `match t { Literal(_) => ... }` (one arm).
Currently requires 3 separate `is_lit_*_shape` predicates.

**String parts (3 variants → 1).** Pattern 4 (dimensional).
StrBegin/StrMid/StrEnd are position tags on a string template:
```
StrPart { position: StrPosition }
StrPosition = Begin | Mid | End
```
"Is this any string part?" → `match t { StrPart(_) => ... }`.
In v3 where string interpolation dissolves into FreeMonoid concat,
these tokens exist only to signal grouping — the position tag
makes that explicit.

**Boundary markers (2 variants → 1).** Pattern 4 (dimensional).
Newline and Eof both signal boundaries in the token stream —
statement end vs file end. They share the structural role of
"position marker, not content carrier":
```
Marker { kind: MarkerKind }
MarkerKind = StatementEnd | FileEnd
```
"Is this any boundary marker?" → `match t { Marker(_) => ... }`.

**Unknown → fact placement (#1), not a token at all.**
Unknown is the tokenizer's version of InferredNode's CompilerError:
an error state mixed into a success type. Structurally, it's not
a token — it's the absence of a successful tokenization. By the
fact-placement rule, it belongs in a tokenization diagnostic table,
not in the token stream. The token stream should contain only real
tokens.

**Remaining atomic tokens (~8), with structural justifications:**
- ShIdent — terminal carrier for user-chosen names. Names have no
  compile-time structural properties. Genuinely atomic.
- ShKeyword — already dissolved by variant-is-data (text carries
  identity). Single variant by that rule, not by "irreducible."
- ShColon — binding separator (name:type, key:value). No shared
  dimension with other punctuation.
- ShComma — element separator (list items, fields). No shared
  dimension with Colon — different grammatical role.
- ShDot — field access operator. Related to DotDot (range), but
  the parser uses them in completely different grammatical contexts
  (postfix access vs infix range). No useful cross-variant query.
- ShDotDot — range operator. See Dot.
- ShArrow — return type annotation (`-> Type`). Related to FatArrow
  but different grammatical role (type annotation vs match body).
- ShFatArrow — match arm body (`=> expr`). See Arrow.
- ShEq — assignment/binding form. Structurally distinct from EqEq
  (Ring/BooleanAlgebra equality operation). Assignment is a binding
  form with no dimensional structure.

**Stopping rule:** stop dissolving when remaining variants have no
cross-variant query that would be cleaner as a field read. Each
atom above has no useful "is this any kind of X?" query that groups
it with other atoms.

**Result: 35 → ~9 top-level variants + sidecar relocation.**

| Category | Current | Proposed | Pattern |
|---|---|---|---|
| Operators | 18 | 1 (ShOperator + OperatorSpec) | variant-is-data |
| Delimiters | 6 | 1 (ShDelimiter { shape, side }) | dimensional |
| Literals | 3 | 1 (ShLiteral { kind }) | dimensional |
| String parts | 3 | 1 (ShStrPart { position }) | dimensional |
| Ident | 1 | 1 | atomic |
| Keyword | 1 | 1 | already dissolved (text) |
| Punctuation | 6 | 4-6 (Colon, Comma, Dot, DotDot, Arrow, FatArrow) | mostly atomic |
| Control | 2 | 2 (Newline, Eof) | atomic |
| Assignment | 1 | 1 (Eq) | atomic |
| Unknown | 1 | 1 | atomic |
| Boundary markers | 2 | 1 (ShMarker { kind }) | dimensional |
| Unknown | 1 | 0 (→ diagnostic table) | fact placement |
| **Total** | **35** | **~9** | |

The internal mini-coproducts (BracketShape=3, Side=2, LiteralKind=3,
StrPosition=3) are all small, coherent, and structurally stable —
they won't grow over time.

This is an architectural win, not an optimization. Every parser
site that asks "is this any kind of X?" goes from N-way match to
one-arm match or field read. Same kind of win Experiment 5 measured
for ExprData (~30 arms per new variant).

### Full coproduct inventory — dissolution status

~280 coproducts across the codebase. Categorized by what v3 needs
to address vs what's fine.

**ALREADY DISSOLVED (design complete, 11 types):**

| Type | Variants | Pattern | Status |
|---|---|---|---|
| ExprData | 22 | L1 behaviors + algebraic forms | design done |
| TransformRule (spec) | 14 | algebraic-form (#3) | design done |
| TokenShape | 35 | dimensional (#4) → ~12 | design done |
| Connective | 4 | → Product/Coproduct/Terminal/Function | dissolves with std/ |
| FieldAccessStyle | 5 | algebraic-form (#3) → intro/elim instances | dissolves with std/ |
| InferredNode | 3 | fact placement (#1) → Port + diagnostics | design done |
| SizeBound | 5 | → Bound { produced_by, count } | design done |
| BinOp | 14 | variant-is-data (#2) → algebra resolution | design done |
| UnaryOpKind | 2 | → algebra resolution (same as BinOp) | design done |
| VarBindingKind | 4 | variant-is-data (#2) → origin as data | design done |
| ExprErrorKind | 3 | → unified diagnostic type | design done |

**DISSOLVES WHEN V3 LANDS (no design needed, 8 types):**

| Type | Variants | Why it dissolves |
|---|---|---|
| MethodSemantics | 3 | algebra elim vs transport elim — falls out of function space |
| CallSemantics | 2 | Plain vs Lookup — falls out when method resolution reads algebra |
| StringPart | 2 | dissolves when string interp → FreeMonoid concat |
| OperationModifier | 3 | becomes boolean fields on effect descriptor |
| ExprCategory | 5 | classification of ExprData — gone when ExprData gone |
| FuncBodyShape | 3 | classification of func bodies — gone when emit reads L1 behaviors |
| TcoExprShape | 5 | TCO classification — gone when recursion → Loop lowering |
| BackendCapability | 5 | capability flags — becomes boolean fields on LanguageSpec |

**DESIGN DECISIONS RESOLVED (4 non-trivial, 4 mechanical):**

The 4 non-trivial cases have concrete v3 type declarations.
The 4 mechanical cases (NodeFieldRole, FunctionSizeEffect, ItemKind,
AliasKind) dissolve automatically when their substrate changes land.

#### OwnershipDecision — DELETED (fact placement #1)

No type declaration needed. Ownership is a derived query on Ports:

```
fan_out(port) == 1                            → move-eligible
fan_out(port) > 1                             → shared; diagnostic if uniqueness required
fan_out(port) == 0 AND node is pure           → dead code diagnostic
fan_out(port) == 0 AND node has effect        → valid (effect is the "use")
```

Consumers that asked "is this SoleOwner?" now ask `fan_out(port) == 1`.
Errors go to the diagnostic table. The Port.value_type = None invariant
extends: if a port has fan_out > 1 in a uniqueness context, a diagnostic
is written with binding, consumer_count, and sites as structured data.

Note: fan_out=0 rule accounts for effects — effectful ports with no
consumers are valid (the effect IS the use), not dead code.

#### EdgeKind — dimensional (SourceEffect × ControlRole)

```
type SourceEffect = Intact | Partial | Consumed
type ControlRole  = DataFlow | IterationCarry

type Edge {
  source_port: PortId
  consumer_port: PortId
  source_effect: SourceEffect
  control_role: ControlRole
}
```

v2's Threaded = (Consumed, IterationCarry). Consumer migration:
- `semantic_consumer_count` → `source_effect == Consumed && control_role == DataFlow`
- `binding_fan_out` → `control_role == DataFlow`
- `build_read_only_params` → `source_effect != Consumed OR control_role == IterationCarry`

The third filter mixes dimensions — a hint that "read-only params" may
be a v2-ism that mixed two concepts. Flag for audit during v3 ownership.

Naming: "control_role" replaces "administrative" — structural property
of the edge in the data-flow graph, not compiler bookkeeping category.

#### MatchPattern — dimensional (Activation × BindingSet)

```
type Activation =
    Always
  | Literal { value: LiteralValue }
  | Variant { variant_ref: VariantRef }

type BindingSet =
    NoBinding
  | Single { name: Name }
  | PerField { fields: List<(FieldName, Pattern)> }

type Pattern {
  activation: Activation
  binding: BindingSet
  span: SourceSpan
}
```

v2's 4 variants are specific points in a 3×3 space:
- Wildcard       = (Always, NoBinding)
- Bind           = (Always, Single)
- LitPattern     = (Literal, NoBinding)
- VariantPattern = (Variant, PerField)

Uninhabited: (Always, PerField) and (Literal, PerField) — "destructure
anything" and "destructure a literal" are nonsensical. Enforced by
validity rule, not type structure.

Free extensions at zero consumer cost: at-patterns like `n @ 42`
(Literal, Single) and `s @ Some(_)` (Variant, Single).

PerField uses named field bindings `(FieldName, Pattern)` rather than
positional — prevents wrong-field bugs. Change from v2's positional model.

Exhaustiveness checking simplifies: the checker folds over patterns
along the activation axis. "Is this exhaustive?" = "does the set of
activations cover all values of the scrutinee type?"

#### CompilerDiagnostic — 5-field record with residual coproducts

```
type Severity = Error | Warning | Info

type Category = Module | Type | Pattern | Parse | Analysis | Internal

type Subject =
    OfName     { name: Name }
  | OfType     { type_ref: TypeRef }
  | OfField    { field_ref: FieldRef }
  | OfModule   { module_path: ModulePath }
  | OfFunction { function_ref: FunctionRef }
  | OfBinding  { binding_ref: BindingRef }
  | OfVariant  { variant_ref: VariantRef }

type Detail =
    NoDetail
  | Comparison    { expected: TypeRef, got: TypeRef }
  | ArityCompare  { expected: Int, got: Int }
  | MissingItems  { missing: List<Name> }
  | ItemChain     { chain: List<ModulePath> }
  | ConsumerList  { consumers: List<BindingRef> }
  | Reason        { text: String }

type Diagnostic {
  span: SourceSpan
  severity: Severity
  category: Category
  subject: Subject
  detail: Detail
  producing_node: NodeId
}
```

**Subject is a residual coproduct, NOT terminal.** The richer source
(typed program references with uniform identity) doesn't fully exist
yet. When typed references land, Subject decomposes to dimensional
form: `Reference { ref_kind, identity }`. This is honest deferral
with a named trigger, not accepted compression.

**Detail is a terminal coproduct.** The variants carry genuinely
different structured payloads (two TypeRefs vs a List<Name> vs Int
pair). No coordinate space exists — they're different small record
types. The smallest possible set of genuinely different shapes.

**Reason is an escape hatch.** Prefer structured Detail variants.
Every Reason use is a deferred modeling task. Over time, Reason
uses should decrease as structured variants are added.

Consumer wins:
- `diagnostic_to_span(d)` → `d.span` (field read, was 16-way match)
- `is_error_diagnostic(d)` → `d.severity == Error` (was hand-maintained list)
- `diagnostic_to_message(d)` → formatted by category + detail kind,
  driven by a template table. New diagnostic = new row, not new branch.

v2 → v3 migration (sample):
```
v2: TypeMismatch { expected, got, span }
v3: Diagnostic { span, severity: Error, category: Type,
     subject: OfType { type_ref: expected },
     detail: Comparison { expected, got },
     producing_node: <node_id> }
```

#### Remaining 4 mechanical dissolutions

| Type | Variants | Dissolution |
|---|---|---|
| NodeFieldRole | 3 | Dissolves when Node moves to std/ — fields declared with inductive structure (already in node.dag) |
| FunctionSizeEffect | 3 | Dissolves when cost lens reads algebraic properties instead of function-specific tables |
| ItemKind | 7 | Variant-is-data: all items have same structure, ItemForm drives differences. Already modeled in SyntaxSpec.item_forms |
| AliasKind | 3 | Variant-is-data: all aliases are name → type, kind determines resolution strategy |

**DOMAIN MODELING (external facts, ~70 types):**

Types that model external systems (GitHub Actions, GCP, LLM APIs,
Cargo, Git, Bash). These are dictated by the external API shapes
and are correct as-is. They describe the real world's structure,
not compression artifacts.

Examples: HttpMethod (7), Platform (3), Arch (11), Role (4),
GitFileStatus (6), CargoCommand (8), ShellType (5).

Rule: if the variants come from an external spec, they're facts
about the world. Don't dissolve facts.

**STD/ MODELING (language concepts, ~40 types):**

Types modeling general computing concepts. Most are small (2-4
variants) and structurally coherent. Worth auditing:

| Type | Variants | Assessment |
|---|---|---|
| LiteralValue | 5 | Dimensional: Literal { kind, value }? But value types differ per kind (String vs Int vs Float). Probably irreducible — the kinds carry different data. |
| BodyKind | 7 | Variant-is-data: all bodies are "contents of a declaration." The kind determines parse shape. Maps to SyntaxSpec. |
| ItemFormKind | 6 | Same as BodyKind — maps to SyntaxSpec item_forms. |
| AlgebraProfile | 7 | Each profile carries different operation sets. Dimensional? The profile IS the algebraic structure description. Probably irreducible — each algebra genuinely has different laws. |
| AlgebraFieldKind | 5 | Variant-is-data: names of algebraic operations. Could be text in a table. |
| CostBound | 5 | Dimensional: bound has { complexity_class, multiplier }? Or irreducible because the classes have genuinely different composition rules? |
| AtomicCost | 5 | Same question as CostBound. |
| RecursionShape | 5 | induction.dag shapes. Dimensional: { direction, wrapping }? |
| Certainty | 4 | Severity-like scale. Could be an ordered Int. |
| DescentEvidence | 3 | Ordered lattice: Strict > NonIncreasing > Unknown. Could be an Int with ordering. |
| Fragment | 10 | Render fragments. Each carries different content. Dimensional? Category + content? Some share structure (Text, Comment, Heading all carry String). |
| Ordering | 3 | Less/Equal/Greater — mathematical. Genuinely irreducible. |
| Cardinality | 2 | Required/Optional — genuinely irreducible (it IS the information). |
| Classical | 2 | True/False — genuinely irreducible (it IS the Bit). |

**COMPILER INFRASTRUCTURE (parse/infer/emit internals, ~30 types):**

Types internal to v2's compiler pipeline. Most are result types
(ParseResult variants, lookup statuses) or classification types
for specific stages. These don't carry forward to v3 — they're
v2 implementation details that v3 replaces entirely.

Examples: AdvanceResult (2), EatResult (2), ExpectedToken (4),
NodeLookupStatus (3), PatternSubject (3), CoercionAssertion (3),
StringScanResult (3), TypeRepr (2).

Rule: don't dissolve v2 infrastructure that v3 replaces. Focus
dissolution energy on types that v3 inherits.

### Summary counts

| Category | Types | Action |
|---|---|---|
| Already dissolved | 15 | design done (11 prior + 4 resolved) |
| Dissolves with v3 | 7 | no action needed (BackendCapability → delete, dead code) |
| Mechanical (v3 substrate) | 4 | NodeFieldRole, FunctionSizeEffect, ItemKind, AliasKind |
| Domain modeling | ~70 | external facts (spot-check 5 pending) |
| std/ modeling | ~40 | audit individually |
| Compiler infrastructure | ~30 | v3 replaces entirely |
| **Total** | **~280** | |

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

**Experiment 2 gate: CLEARED.**

The classify_let_value reconstruction path has been deleted for
ExprVar and ExprFieldAccess cases. scope_locals (carried facts)
covers these paths. 415 tests pass, CX ratchet 0 diagnostics
(stable), no compensating skip annotations.

**Experiment 2 completion proves the physics-and-lens architecture
holds when tested.** One reconstruction path has been deleted with
zero CX regression and no compensating mechanism. The remaining
20 migrations are incremental work under the same pattern. This is
the first banked dividend in the project, and it confirms that
dissolution removes code rather than relocating complexity.

The 20 remaining sub_value_vars reads (classify_argument,
resolve_collection_field, annotate_descent match/lambda/foreach)
are a parallel track alongside v3 M0. Each migration is
independently testable with the same pattern. If two consecutive
migrations find the pattern doesn't apply cleanly, pause and
investigate — that's the signal of a sub-case the first migration
didn't teach us.

### Mechanism vs dividend discipline

Use this as a standing discipline for v3 work:

| Claim | Mechanism | Dividend | Cost to bank |
|---|---|---|---|
| Rule-table transforms | validated (Exp 3) | banked | done |
| Observational lenses | validated (Exp 4) | banked | done |
| Facts carried through bindings | validated (Exp 2) | **BANKED** — classify_let_value reconstruction deleted, CX stable | done (remaining: 20 incremental migrations) |
| Lambda = function | validated (Exp 1) | partially banked — closure rule identified | small: closure rule is 2 fields on Define (capture_context, iteration_bound). Not a redesign — the physics model already supports it. |
| ExprData tax is real | measured (Exp 5) | target: 8-11x → 1x | v3 construction (the whole project) |

The spec only claims credit when mechanism AND dividend are green.
Exp 3+4 justify the TransformRule dissolution and lens mechanism.
Exp 2 justifies the "physics carries facts" claim ONLY when the
old reconstruction is gone. The cost column prevents "partially
banked" from hiding material vs trivial remaining work.

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

### std/ stays (reviewer-confirmed):
1. computation.dag CallPattern/LoweringTarget — language guarantee
2. effects.dag ComposedEffect/ModifierAgreement — algebraic
3. fidelity.dag — shared transport vocabulary (extdeps layer model)

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
