# v3 Roadmap

> Design spec: [docs/v3-spec.md](../../docs/v3-spec.md)
> Validation: [docs/v3-validation-experiments.md](../../docs/v3-validation-experiments.md)
> Lineage: [docs/design-lineage.md](../../docs/design-lineage.md)

## Principles

- Keep it simple. If a file gets large, something is wrong.
- Behaviors compose from std/. Hardcoded rules = missing modeling.
- Every decision should trace to a validation experiment or a v2 lesson.
- v2 is the reference implementation and test oracle.

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
- [ ] Emit Rust from DAG (single target, minimal)
- [ ] Emitted code compiles and runs
- [ ] Lenses: cost lens reads the DAG
- [ ] Lenses: ownership lens reads fan-out
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
