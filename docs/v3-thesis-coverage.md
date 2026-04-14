# v3 Thesis Coverage

> Part of: [THESIS.md](../THESIS.md)
> Single authority: [docs/v3-spec.md](v3-spec.md)

The spec is the authority. This doc tracks which thesis claims
are covered and which have gaps. See the spec for the design.

## Coverage

| Thesis claim | How the spec covers it | Gap? |
|---|---|---|
| **Type safety** | Carrier type on every Port | None |
| **Termination** | Loop requires a Bound. Recursion = Loop. Once DAG is built, termination is structural. | Lowering boundary: classifying recursion, determining bounds is real work |
| **Complexity (cost)** | Cost lens reads per-behavior cost from rule table, composes structurally | None |
| **Ownership** | Ownership lens reads fan-out from Port edges. Exclusive branches don't double-count. | None |
| **Side effects** | Effect lens reads per-behavior effect, composes as lattice join | None |
| **Purity** | Derived: effect lens returns Pure for all behaviors in subgraph | None |
| **Idempotence** | Algebra lens reads algebraic properties from rule table | None |
| **Space bounds** | Space lens, analogous to cost lens but tracks allocation | Needs rule table entries |
| **Coercion = emission** | TypeCast is a Transform rule. Emission reads LanguageSpec. | None |
| **Parallelism** | Compiler raises independent iterations to Map (L2). Waves from DAG structure. | None |
| **Memoization** | Derived: Pure + cost > threshold + finite types → emitter can memoize | Emission strategy not specified |
| **Algebraic simplification** | Normalization during DAG construction. Compiler reads algebra table, simplifies before nodes enter DAG. | None |
| **Tier 2 runtime safety** | Carrier refinement on Ports (NonZero, InBounds). No unsafe unwrap — must Branch. | Refinement types need design |
| **L4-L7 verification** | Closed system → compiler generates witnesses, evaluates kernel, compares to emitted code | Implementation not started |
| **User-defined dimensions** | User defines a lens: what to measure, how it composes, what to check. Same mechanism as built-in. | Deferred to v3.1 |
| **Diagnostics** | Corrections, not errors. 4 levels: show → show fix → emit fix → apply fix. Emission to .dag LanguageSpec. | None |
| **Non-consensual optionality** | Optional<T> is structural. Branch is the only access. No unwrap. | None |
| **Bootstrap** | Architecture, not tooling. Regen → diff → empty. | Needs integration with spec |

## E2E scenarios in the spec

6 scenarios prove the design works frontend → emission:

1. Nested optional map lookup — 3 layers of Optional, zero clones
2. Mutual recursion → Loop → automatic TCO
3. Imperative loop → raised to Map → parallel emission
4. Clone/elision pipeline — ownership lens, zero defensive clones
5. Recursive generics — Tree<T>, Tree<Tree<Int>>, same bounded law
6. Generics + optionality + ownership combined — the hardest case

## What's not covered yet

- Carrier refinement types (Tier 2 safety) — design needed
- Space bound rule table entries — straightforward, not done
- L4-L7 test generation implementation
- User-defined lenses (v3.1)
- Bootstrap integration with v3 spec
