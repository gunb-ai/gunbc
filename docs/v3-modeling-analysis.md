# v3 Modeling Analysis

> Purpose: audit every type in the v3 spec and current std/, determine
> where each belongs, and show how TransformRule dissolves into algebra.
>
> This doc is the design phase. Code changes follow from decisions here.

## Principle

Two buckets:

1. **std/** — general concepts from math and computing. True regardless
   of this compiler. Algebra, iteration, type constructors, data flow.

2. **compiler lib/** — implementation details specific to THIS compiler.
   Organized per layer (parse, resolve, infer, emit) to prevent
   contamination. The compiler imports from std/ but std/ never imports
   from the compiler.

If a type traces to a math concept or a universal computing concept,
it belongs in std/. If it's about how this specific compiler works,
it belongs in compiler lib/.

---

## V3 Spec Types — Where They Belong

### Value — std/

A known thing. No inputs, one output. Literal, constant, data.

**Traces to:** terminal morphism in category theory. The unique
morphism from the terminal object (unit) to a type. "3" is the
morphism Unit → Int that picks 3.

**Already modeled:** std/syntax.dag has `LiteralValue` (the data).
std/constructors.dag has `Terminal` (the structural shape). Value
composes these: a Terminal-shaped node carrying a LiteralValue.

### Transform — std/

Do something to inputs, get an output. The workhorse.

**Traces to:** morphism. Every transform is a morphism in some
algebraic category. What category and what morphism is determined
by the type's algebraic structure.

**The rule is NOT an enum.** See "TransformRule Dissolution" below.

### Branch — std/

Look at something, take a path. `if`, `match`, pattern matching.

**Traces to:** Coproduct elimination. std/constructors.dag already
declares Coproduct. Branch is its elimination form — the universal
way to consume a coproduct is to handle each variant.

**Already modeled:** constructors.dag has `Coproduct`. What's missing:
the explicit connection between Coproduct and Branch (the elimination
form).

### Loop — std/

Repeat something, bounded.

**Traces to:** bounded recursion / iteration. std/iteration.dag
already declares fold, descend, repeat. Loop is the general form
that these are instances of.

**Already modeled:** iteration.dag has the three primitives.
computation.dag has `SizeBound` and the lowering table. Loop
composes: an iteration primitive with a bound and a body.

### Bind — std/

Give something a name. Not computation — just wiring.

**Traces to:** variable binding / let-abstraction. Universal in
lambda calculus, type theory, every programming language.

**Already modeled:** std/binding.dag has `Binding { key, value }`.
Bind extends this with scope.

### Port — std/

Typed connection between behaviors. Data flows forward through ports.

**Traces to:** typed edge in a directed acyclic graph. Ports are
the edges; behaviors are the nodes. This is the fundamental data
flow concept.

**Not yet modeled.** Needs a std/ definition. Relates to:
- constructors.dag (port carries a value of some Product/Coproduct shape)
- types.dag (port has a value_type from the type system)

### Bound — std/

How many times a Loop repeats. An integer — finite by construction.

**Traces to:** well-founded measure / termination measure.

**Already modeled:** computation.dag has `SizeBound` with 5 variants
(CollectionSize, TreeSize, ArithmeticParam, ExplicitCount, Forever).
This is the right concept but may be over-specified — the compiler
determines the source; the bound itself is just "a finite number."

### Path — std/

One arm of a Branch. Pattern + bindings + body.

**Traces to:** one case of a Coproduct elimination. The pattern
identifies which variant, bindings extract the components, body
handles it.

**Not yet modeled as a standalone concept.** Relates to Branch.

### Pattern — std/

What activates a path in a Branch. Structural match.

**Traces to:** destructuring / Coproduct case discrimination.

**Partially modeled:** std/patterns.dag exists but is about DAG
workflow patterns (ensure, upsert), not pattern matching. Pattern
matching is a different concept that needs its own modeling.

---

## Reference Types — Where They Belong

| Type | Bucket | Reason |
|------|--------|--------|
| NodeId | compiler lib/ | this compiler's identity scheme |
| PortId | compiler lib/ | this compiler's identity scheme |
| TypeShape | split | constructors (Product/Coproduct/Terminal) are std/; representation is compiler lib/ |
| LiteralValue | std/ | already in std/syntax.dag |
| BinOp | std/ | already in std/syntax.dag; should trace to algebra operations |
| UnaryOpKind | std/ | should trace to algebra operations |
| FieldRef | compiler lib/ | this compiler's reference to a product field |
| MethodRef | compiler lib/ | this compiler's dispatched application target |
| NodeRef | compiler lib/ | this compiler's reference to a subgraph |
| TypeRef | compiler lib/ | this compiler's reference to a type declaration |
| BindingId | compiler lib/ | this compiler's naming scheme |
| BuiltinKind | compiler lib/ | this compiler's set of primitives |
| ParamPort | std/ | a port with a name — parameter |

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
itself has zero edits.

### What's missing from std/

The **function space** (exponential object) isn't modeled:

```
Function space A → B:
  Introduction (Define): create a function value (params → body)
  Elimination (Apply):   call a function with arguments
```

This is the same intro/elim pattern as Product and FreeMonoid. It
should be declared in std/ alongside them. Then Define, Call, Method,
and Builtin all trace to it.

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

## std/ Audit — What Doesn't Belong

Types currently in std/ that are about THIS compiler, not general concepts:

### node.dag — compiler-specific

Declares InductiveField entries for Node, InferredNode, MatchPattern,
MethodSemantics. These are this compiler's data structures.

**Move to:** compiler lib/ (analysis layer). The CONCEPT of inductive
fields and recursion shapes is general (and stays in std/induction.dag).
The INSTANCES for specific compiler types are compiler data.

### graph.dag — mixed

The graph algorithms (DFS, Kosaraju SCC) are general. But the specific
types (CallGraph with String keys, imports from termination.dag's
ProofEdge) are compiler-specific.

**Split:**
- General graph algorithms → stay in std/ (generalized)
- CallGraph, proof validation → compiler lib/ (analysis layer)

### computation.dag — mixed

SizeBound, IterationPrimitive, IterationDimension are general concepts.
CallPattern (ChildAccessorCall, CollectionShrinkCall, etc.) is about
how THIS compiler classifies recursive calls.

**Split:**
- SizeBound, IterationPrimitive, IterationDimension → stay in std/
- CallPattern, LoweringTarget, lower_call_pattern → compiler lib/

### fidelity.dag — compiler-specific

TransportClass, TestClass, DerivedClassification — about how this
compiler classifies tests and transport declarations.

**Move to:** compiler lib/ (analysis layer).

### effects.dag — mixed

EffectShape (Pure, ServiceCall, etc.) is a general concept. The
specific composition rules (ComposedEffect, ModifierAgreement) are
this compiler's policy.

**Split:**
- Effect vocabulary (Pure, Read, Write, etc.) → std/
- Composition policy → compiler lib/

---

## std/ Audit — What's Missing

### Function space (exponential)

The intro/elim structure for functions. Every language has:
- Define: create a callable (function, lambda, closure)
- Apply: call a callable with arguments
- Capture: a define that references outer scope (closure)

Should live in std/ alongside constructors.dag's Product/Coproduct.

### Port / typed data flow

The concept of a typed, directed edge between computation nodes.
Forward-only. No cycles. Carries a value of a declared type.

This is the substrate that connects behaviors. Currently only in
the v3 spec as design notation.

### Intro/elim declarations on algebraic structures

algebra.dag declares the algebraic hierarchy (Magma → Ring → Field).
It declares the OPERATIONS that emerge. It does NOT declare the
INTRODUCTION and ELIMINATION forms.

Adding these makes TransformRule dissolve:
- Product: intro = construct from fields, elim = project field
- FreeMonoid: intro = empty + cons/concat, elim = index/fold/length
- Ring: ops already declared (add, sub, mul, negate)
- BooleanAlgebra: ops already declared (and, or, not)
- Function: intro = define, elim = apply

### Pattern matching

Pattern matching as Coproduct elimination. Currently std/patterns.dag
is about workflow patterns (different concept). Match patterns need:
- Wildcard: match anything
- Literal: match specific value
- Variant: match coproduct variant, bind fields
- Nested: compose patterns

---

## Lens Types — Where They Belong

The v3 spec describes 7 lenses. Their CONCEPTS are general (cost
composition, ownership tracking, effect classification). Their
IMPLEMENTATION reads compiler-specific DAG structure.

| Lens | Concept (std/) | Implementation (compiler lib/) |
|------|------|------|
| Cost | CostAlgebra: add for sequence, mul for loops, max for branches | Reads specific DAG behaviors |
| Ownership | FanOut: count of consumers per port | Reads specific port edges |
| Effect | EffectLattice: Pure < Read < Write < Service | Reads specific behaviors |
| Termination | WellFoundedOrder: every Loop has a bound | Reads Loop nodes |
| Provenance | produced_by: follow edges backward | Reads Port.produced_by |
| Algebra | IntroElimLaws: involution, inverse, fusion | Reads adjacent transforms |
| Space | same composition as Cost but tracks allocation | Reads specific DAG behaviors |

The composition rules (add, max, multiply for cost; lattice join for
effects) are math — they belong in std/. The specific traversal of
THIS compiler's DAG structure is compiler lib/.

---

## Summary: What Changes

### std/ gains:
1. L1 behavior type declarations (Value, Transform, Branch, Loop, Bind)
2. Port / typed data flow
3. Function space (exponential) — intro/elim
4. Explicit intro/elim declarations on existing algebraic structures
5. Pattern matching types (Coproduct elimination)
6. Lens composition algebras (cost, ownership, effect lattices)

### std/ loses:
1. node.dag compiler-type instances → compiler lib/
2. graph.dag compiler-specific types → compiler lib/
3. computation.dag CallPattern/LoweringTarget → compiler lib/
4. fidelity.dag → compiler lib/
5. effects.dag composition policy → compiler lib/

### TransformRule dissolves:
From 14-variant enum → algebraic structure + form (intro/elim/op).
The compiler reads std/ declarations to know what to do. No enum
to maintain. No match arms to add when a new structure appears.

### Compiler gets a lib/:
Organized by layer. Each layer imports from std/ but not from other
layers. Prevents the contamination where parse types leak into emit.

---

## Open Questions

1. **How does the compiler look up intro/elim forms at compile time?**
   The std/ declarations are .dag data. The compiler reads them at
   startup and builds an index: given a type's algebra profile, what
   are the valid intro/elim/operation forms? This index replaces the
   TransformRule enum.

2. **What about operations that don't trace to a single algebraic structure?**
   Example: string interpolation builds a String but uses values from
   multiple types. Is this a FreeMonoid introduction with heterogeneous
   inputs (each coerced to Char via the algebra)? Or something else?

3. **How much of this is v3 vs v2-compatible?**
   The std/ declarations (intro/elim on structures, function space,
   port) can be added now without changing v2. The compiler reading
   them is v3 work. The std/ audit (moving compiler types out) can
   happen incrementally.

4. **Where exactly does compiler lib/ live?**
   Options: src/v3/lib/, dsl/compiler/, or organized within src/v3/
   by stage. Need to decide before moving types.

5. **Do BinOp/UnaryOp survive as std/ types or dissolve into algebra?**
   Currently in std/syntax.dag. BinOp { Add, Sub, Mul, ... } traces
   to Ring operations. But BinOp is also a SYNTAX concept (the parser
   sees `+` and produces a BinOp). Should syntax.dag keep BinOp as
   surface syntax, with the compiler resolving it to the algebra's
   operation at construction time?
