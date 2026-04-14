# v3 Roadmap

> Design spec: [docs/v3-spec.md](../../docs/v3-spec.md)
> Validation: [docs/v3-validation-experiments.md](../../docs/v3-validation-experiments.md)
> Lineage: [docs/design-lineage.md](../../docs/design-lineage.md)

## Principles

- Keep it simple. If a file gets large, something is wrong.
- Behaviors compose from std/. Hardcoded rules = missing modeling.
- Every decision should trace to a validation experiment or a v2 lesson.
- v2 is the reference implementation and test oracle.

## Sketch vs Oracle framing (M0–M2)

**The Rust at `src/v3/compiler/` is a sketch, not an oracle.**

During M0–M2, the Rust implementation exists to validate the
substrate design — to discover whether the L1 decomposition, the
port invariants, the lens architecture, and the diagnostic system
actually work when you build against them. Its purpose is
*discovery*, not *specification*. The .dag version (M3) is the
real v3, and it will be written fresh against the same test suite,
using the Rust as a reference for "did we think of this case?" and
"does the architecture hold?" — not as a structural template.

Consequences:
- **Style matches Rust, not .dag.** Imperative patterns (mutable
  Dag, HashMap scope mutation, fixpoint loops with a `changed`
  flag) are fine when they fit Rust's affordances. Style
  consistency with .dag is nice-to-have, not load-bearing.
- **Refactor only where the pattern is structurally gapped.**
  If the Rust pattern relies on something .dag genuinely cannot
  express (mutable references in a recursive parameter position,
  for example), refactor now. If the gap is "functional Rust
  would be prettier," defer to M3.
- **The M0.6 immutable-scope refactor is an example of the first
  category.** `&mut HashMap` as a recursive parameter has no .dag
  analogue, so lower.rs threads scope by value.

**At M3**, the Rust's role transitions. During the port attempt,
re-evaluate which patterns need restructuring before translating.
That's the first time you have enough information to know what
maps mechanically and what requires redesign. Pre-emptive
restructuring at M0 would be guessing about M3's needs.

Do NOT re-litigate "should the Rust be more functional" in every
review. The answer for M0–M2 is: no, it's a sketch. The answer
at M3 is: look at the port attempt.

## Architecture

```
Source text → tokenize → parse → build DAG → emit
                                    │
                                    ├── lenses read the DAG
                                    │   (cost, ownership, effects, ...)
                                    │
                                    └── emitter translates DAG + LanguageSpec → text
```

5 L1 behaviors: Value, Transform, Branch, Loop, Bind.
Transform rules come from compositional modeling in std/, not
from a hardcoded enum. If something needs a new rule variant,
first ask: can the algebra express this?

## Open design questions

1. **Transform rules from algebra, not enums.**
   ListBuild, StringBuild, Construct, IndexAccess, SliceAccess,
   Cast — all should emerge from std/ modeling (FreeMonoid,
   Product, PartialFunction, etc.). TransformRule should be
   minimal: Access, Apply, Arithmetic, Define. Everything else
   is data from the algebra declarations.

2. **Bound source tracking.**
   Bound is currently `count: Port` (just an Int). The compiler
   may need to know WHERE the bound came from (collection size
   vs explicit number) to verify structural descent. TBD during
   implementation.

3. **Closure context rule.**
   When a Define has an edge into a Loop, captures inherit the
   Loop's fan-out and termination context. This is documented in
   the spec but needs to be wired into the ownership and
   termination lenses during implementation.

4. **Carrier refinement (Tier 2 safety).**
   NonZero divisors, InBounds indices, no force-unwrap. Needs
   design before implementation. Likely: refinement predicates
   on Port types, checked at Branch boundaries.

5. **Effect composition.**
   How effects compose across sequential nodes, Branches, and
   Loops. The spec says "pick the strongest" but the details
   (commutativity of service calls, ordering constraints) need
   working out.

## Milestones

### M0: Skeleton
- [ ] Tokenizer (can reuse v2's approach — it's already clean)
- [ ] Parser → produces DAG with 5 behaviors
- [ ] Minimal type system (primitives + records + sums)
- [ ] Can parse and build DAG for a trivial program

### M1: Self-contained compilation

**Ordering note (post-M0 retrospective):** cost lens comes BEFORE
emission, not after. The cost lens is the first writer lens in v3 —
it produces new facts (computed costs per node) that downstream
analyses consume. Building it first forces the "how do lenses store
results" decision under real pressure, rather than guessing at the
answer while building emission. If the cost lens can't be added
without substrate modifications, the substrate is not yet at the
success bar and needs fixing before emission lands.

- [ ] **Cost lens (writer lens #1)** — reads the DAG, writes computed
  costs per node/port. Implementation must live in `lens_cost.rs`
  as a new file with no substrate file changes. If substrate changes
  are required, pause and design the lens-storage mechanism once,
  then proceed. Acceptance: line count of substrate mods = 0.
- [ ] **Success bar validated for writer lenses.** By the end of the
  cost lens work, the question "if we came up with a new lens
  tomorrow, what's the minimum substrate change?" has a confident
  answer of zero. This is the gating acceptance criterion for
  moving on to emission. See `src/v3/M0_RETROSPECTIVE.md` for the
  framing.
- [ ] **Ownership lens (writer lens #2)** — second writer lens.
  Reuses the storage mechanism chosen for cost. If the mechanism
  doesn't generalize, that's a signal to fix it before adding more
  lenses.
- [ ] Emit Rust from DAG (single target, minimal). Only after cost
  + ownership lenses prove the substrate is extensible.
- [ ] Emitted code compiles and runs
- [ ] Transform rules from std/ algebra (not hardcoded enum)

### M2: Feature parity with v2 subset
- [ ] Generics
- [ ] Optional / cardinality
- [ ] Service calls (transport declarations)
- [ ] Pattern matching (Branch with destructuring)
- [ ] Interpreter (dag run)
- [ ] Recursive functions → Loop lowering

### M3: Self-hosting
- [ ] v3 can compile itself
- [ ] Bootstrap: v2 compiles v3 stage0, v3 compiles v3 → fixed point
- [ ] All v2 test programs compile under v3 with same output

### M4: Thesis completion
- [ ] All lenses operational (cost, ownership, effects, termination,
      algebra, space)
- [ ] Diagnostics as corrections (Level 1-2)
- [ ] L4 verification: emitted code matches DAG evaluation
- [ ] User-defined observational lenses

## What NOT to build yet

- Generic dimension mechanism (user-defined optimization lenses)
- Multi-target emission (start with Rust only)
- Omni-emission (multi-artifact from single source)
- Advanced diagnostics (Level 3 auto-fix)
- Async/concurrent emission strategies

These are thesis goals that fall out when the foundation is right.
Don't build them — let them emerge.
