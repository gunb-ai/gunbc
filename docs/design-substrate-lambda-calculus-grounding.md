# Substrate grounding in lambda calculus

**Status:** LIVE 2026-04-29 (foundational reference). Authored as a P1-Modeling-Faithfulness grounding doc — gunbc's substrate is a *typed total* fragment of lambda calculus + structural extensions; this doc names the mapping explicitly so reviewers can verify each claim against external mathematical consensus rather than against internal taxonomy.

## What this doc does

Maps gunbc's 5 substrate `Behavior` variants (`src/v3/std/substrate.dag`) onto their lambda-calculus / type-theory equivalents. Names the 3 intentional divergences from pure lambda calculus that make gunbc *typed total* (decidable) rather than *untyped Turing-complete*. Names what gunbc adds *above* the lambda calculus kernel (analyses on terms, not citizens of the calculus). Names 3 research-level open questions worth investigating post-R2.

External authority: Church (1932/1936), Curry-Howard correspondence, System F (Girard 1972 / Reynolds 1974), Calculus of Inductive Constructions (Coquand & Huet 1986), Agda's totality discipline, size-change termination (Lee, Jones, Ben-Amram 2001).

## Direct mapping (5 substrate behaviors → λ-calculus)

The 5 `Behavior` variants in `src/v3/std/substrate.dag` map onto canonical lambda-calculus / typed-lambda-calculus constructs:

| `Behavior` variant | λ-calculus equivalent | Mechanism |
|---|---|---|
| `Behavior::Value(ValueNode)` | Constant / 0-ary application | `payload: LiteralBits` carries the structural value; reflects as a leaf `FieldValue`. |
| `Behavior::Transform(TransformNode)` | Application `(M N)` | `target: TransformTarget = Callable(DeclarationId) \| FieldProject \| Operator` discriminates the function being applied; `inputs: List<PortId>` are the arguments. |
| `Behavior::Bind(BindNode)` | Let-binding `let x = M in N` | Sugar for `(λx.N) M`. The bound name is reified as a `NodeId`; the body is a sub-DAG. |
| `Behavior::Branch(BranchNode)` | Pattern-match on a coproduct (System F + sums) | Each `BranchPath` is one arm; `pattern: BranchPattern` is the variant discriminator; `binding: PayloadBinding?` extracts the payload. |
| `Behavior::Loop(LoopNode)` | Bounded primitive recursion | `body: NodeId` is the per-iteration body; `loop_bound: LoopBound` is the explicit bound carrier (replaces Y-combinator-based fixed-point). |

Function declarations themselves materialize via `Arrow.body` (referenced by `INVARIANTS.md` DB-14 — "External Primitives Materialize Through Declared Arrow.body Plus Target Bindings"). `Arrow` is gunbc's name for the typed lambda abstraction — `Arrow.body` is structurally `λ(params).body`.

### α-conversion is satisfied by construction

Pure lambda calculus's α-conversion rule (`λx.M ≡ λy.M[x:=y]`) is the equivalence of bound-variable renaming. gunbc *doesn't have bound variable names at substrate level* — variable references are `NodeId` / `PortId` references. NodeId is the canonical identity; renaming would create a different value but you'd never have a reason to. **α-conversion is structurally trivial in gunbc**, which is cleaner than the formal calculus's name-based binding.

### β-reduction is being reified

Pure lambda calculus's β-reduction (`(λx.M) N → M[x := N]`) is the *operational* rule of evaluation — substitution of arguments into the body. In gunbc, β-reduction is being made an explicit substrate citizen via the **PB-Runtime interpreter-as-data** work in [`docs/design-pb-runtime-interpreter.md`](design-pb-runtime-interpreter.md) (PR #1176). The R2-Evaluator + PB-Runtime co-design IS the explicit reification of β-reduction over the 5-Behavior substrate.

### Capture-avoiding substitution is structurally satisfied

Pure lambda calculus's substitution `M[x := N]` requires capture-avoidance machinery (rename bound vars to avoid free-variable capture). gunbc's DAG structure handles this implicitly: every NodeId reference is a direct pointer to its declaration site; there's no name-based shadowing to avoid. **No capture problem because no name-based scope.**

## Three intentional divergences from pure λ-calculus

Each of these is a deliberate design choice grounded in external consensus.

### 1. Loop replaces Y-combinator (totality choice)

Pure untyped λ-calculus is Turing-complete via the Y-combinator: `Y = λf.(λx.f(x x))(λx.f(x x))`. This is what gives lambda calculus its expressive power but also its undecidability. gunbc rejects this in favor of `Behavior::Loop` with explicit `loop_bound: LoopBound`. Bounded primitive recursion is decidable; programs always terminate.

**External grounding:** this is the *typed total* position — the same choice made by Coq's Calculus of Inductive Constructions, Agda's totality checker, Idris's `total` keyword, and System T (Gödel 1958). Trades Turing-completeness for decidability; well-studied.

**Authority:** `INVARIANTS.md` §P4 (Decidability) — "all `.dag` code must be decidable" — names this directly. The decidability invariant IS the rejection of Y-combinator-based recursion.

### 2. Coproducts are primitive (System F + sums)

Pure λ-calculus Church-encodes sum types: `Either a b = (a → r) → (b → r) → r`. Pattern-matching becomes function application; structural information is lost (you can't observe "this is the left arm" without applying both eliminator functions).

gunbc has `Behavior::Branch` as a substrate primitive — pattern matching is structural. Lens analyses fold over Branch arms directly without re-extracting them from Church encodings. This is a System F + sums extension; same as Haskell, ML, Rust — most modern functional languages.

**Why it's load-bearing for gunbc:** lens analyses (`src/v3/lenses/cost.dag` etc.) need to introspect Branch arms structurally. Church-encoded sums would force the lens framework to defeat its own structural-fold discipline.

### 3. n-ary application

Pure λ-calculus is curried: every function takes exactly one argument. `f(x, y)` desugars to `(f x) y`. gunbc's `TransformNode.inputs: List<PortId>` is n-ary — multiple arguments simultaneously.

This is **cosmetic** — n-ary ↔ curried is structurally equivalent. n-ary is more direct for codegen + cost analysis (multiple inputs evaluated in some order); curried is cleaner for theory. The structural fact is preserved either way.

## What gunbc adds *above* λ-calculus (analyses, not citizens)

The following are **not** part of pure λ-calculus. They're **analyses on terms** that gunbc's substrate makes structurally first-class:

- **`Witness<C>`** (`src/v3/std/dimensions.dag`) — per-Behavior read-channel proof / failure carrier. Not a λ-calculus citizen; a property *about* terms.
- **`Lens<C>`** (`docs/design-lens-framework.md`) — categorical fold over the 5 behaviors with structural inhabitance of `Monoid<C>`. Categorical-effect-handler-shaped (modern type theory; Plotkin & Pretnar 2009).
- **`DimensionReport<C>`** — analysis result with violations list. Compositional fold output, not a calculus term.
- **`Diagnostic`** with kinds (Q6.5 two-layer authority per `docs/design-lens-framework.md`) — structural error-reporting carrier.
- **Cost / complexity / capability / IFC lens instances** — structural properties of programs derived by folding the Lens framework over substrate behaviors.

These are **analyses on top of a typed total λ-calculus** — what would normally be done with structural recursion in a paper, executed compositionally over the substrate via the lens framework.

## Three open questions / research-level gaps

These don't block R2; they're worth investigating post-R2 as TestClaims that verify gunbc's behavior matches λ-calculus theorems.

### 1. η-equivalence at substrate level

Pure λ-calculus has η-conversion: `λx.(f x) ≡ f` (extensional equality). If gunbc's DAG can carry both `f` and an η-expanded `λx.apply(f, [x])` as distinct structures, lens analyses might double-count or disagree on observationally equivalent forms.

**Question:** does the substrate normalize to a canonical form (always-η-reduced)? Or do reflection / lens analyses see η-expanded forms as different?

**Suggested TestClaim** (to land post-Substrate-Lens-Primitive):
```
test_claim eta_equivalent_dag_forms_yield_identical_lens_results {
  // For any Lens<C>, applying it to `f` and to `λx.f(x)`
  // yields identical DimensionReport<C>.
}
```

**Probable owner:** R2 Substrate Manager (`jolly-ram-908`) — substrate-shape question; depends on Substrate-Lens-Primitive landing.

### 2. Confluence (Church-Rosser) for evaluation

Pure λ-calculus has the Church-Rosser property: any two reduction paths converge to the same normal form. gunbc's evaluation order is being specified by `docs/design-pb-runtime-interpreter.md` (PR #1176), but no explicit confluence theorem is currently stated.

**Question:** does the locked evaluation order in PB-Runtime guarantee that two valid evaluation strategies (e.g., applicative-order vs normal-order) produce the same lens-output for any program?

**Suggested TestClaim** (to land post-PB-Runtime):
```
test_claim evaluation_order_independent_lens_results {
  // For any program + Lens<C>, two valid evaluation strategies
  // (applicative vs normal order; left-first vs right-first
  // input evaluation in n-ary Transform) yield identical
  // DimensionReport<C>.
}
```

**Probable owner:** R2 Evaluator Manager (`snappy-moth-795`) since they own the interpreter spec; alternatively R2 Pure Bootstrap Manager (`cool-stag-230`) since they own PB-Runtime authoring per `docs/design-pb-runtime-interpreter.md`. Cross-program coordination at lane spin-up.

### 3. Strong normalization for the typed fragment

Typed λ-calculi (STLC, System F, CIC) have strong normalization: every reduction sequence terminates. gunbc's `LoopBound` + `feedback_decidability_invariant` rejects unbounded recursion structurally — strong normalization should hold by construction. But there's no explicit theorem cited.

**Question:** is strong normalization stated formally somewhere, or only implied by the decidability invariant + LoopBound?

**Suggested TestClaim** (post-Substrate-Lens-Primitive + post-Loop-construction-closure-audit per B5 from `docs/briefs/r2-release-b5-loop-construction-closure-audit-worker.md`):
```
test_claim every_typed_dag_program_terminates_in_bounded_steps {
  // For any well-typed .dag program with declared LoopBound,
  // evaluation terminates in O(loop_bound) reduction steps.
  // Sufficient: structural induction on Behavior variants
  //   + LoopBound's BoundedLattice structure.
}
```

**Probable owner:** R2 Pure Bootstrap Manager (`cool-stag-230`) since this is structural-property-of-the-evaluator territory; alternatively R3 Verification Manager (when spawned) since strong normalization is a verification-surface claim.

## Cross-references

- **Substrate authority:** `src/v3/std/substrate.dag` — the 5-Behavior coproduct + Arrow + supporting types.
- **INVARIANTS:**
  - §P1 (Modeling Faithfulness) — this doc IS a P1 grounding artifact.
  - §P4 (Decidability) — names the totality choice that justifies the Loop-replaces-Y divergence.
  - DB-14 — Arrow.body materialization mechanism.
- **Lens framework:** `docs/design-lens-framework.md` — categorical fold over the 5 behaviors with `Lens<C>`.
- **Reflection:** `docs/design-reflection-completeness.md` — what "complete reflection" means for the 5 Behaviors when fed to lens-analysis.
- **PB-Runtime interpreter (β-reduction reified):** `docs/design-pb-runtime-interpreter.md` (PR #1176).
- **Cost / complexity lenses:** `src/v3/lenses/cost.dag`, `src/v3/lenses/complexity.dag` — instances of `Lens<C>` over the 5 behaviors.

## What's NOT in this doc (out of scope)

- **Type system details:** type checking / inference / unification belongs in `docs/design-emission-model.md` and the parser docs. This doc is about the kernel calculus, not the type machinery on top.
- **Implementation choices:** Rust pass structure, cost models, parser architecture all live elsewhere.
- **Comparison with specific languages** (Haskell, Rust, OCaml): this doc grounds in the formal calculus, not in language ergonomics.

## Summary

gunbc's substrate is a **typed total lambda calculus + structural coproducts + reified evaluation**. The 5-Behavior substrate maps cleanly onto System F + sums + bounded recursion (CIC-shaped). α-conversion is satisfied by construction (NodeId-based binding); β-reduction is being reified via PB-Runtime; capture-avoiding substitution is satisfied by DAG-reference structure.

Three intentional divergences (Loop / Branch / n-ary) are well-grounded design choices matching modern dependently-typed languages. Three open research-level questions (η-equivalence, confluence, strong normalization) are worth investigating post-R2 as TestClaims; none block current work.

The lens framework + Witness/DimensionReport/Diagnostic are **analyses on top of the calculus**, not calculus citizens — same shape as algebraic effects in modern type theory.

This grounding makes the substrate's claims verifiable against ~90 years of mathematical and CS consensus rather than against internal gunbc taxonomy alone — which is what `INVARIANTS.md` §P1 demands.
