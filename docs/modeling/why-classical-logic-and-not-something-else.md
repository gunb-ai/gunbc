### Why classical logic (and not something else)

The primitive depends on the computational model. Classical
bivalent logic is not universally "the" foundation — it's the right
foundation for classical digital systems:

| Computational model | Primitive | Algebra |
|---|---|---|
| Classical digital | truth/false | classical logic |
| Continuous/analog | real quantity | real analysis |
| Probabilistic | degree of belief [0,1] | probability theory |
| Quantum | complex amplitude | linear algebra over Hilbert spaces |

gunbc targets digital computing, so the foundation is classical
logic. But the composition layer (Node, children, connectives) is
**model-independent** — AND/OR/IMPLIES have analogs in every algebra:

- AND: conjunction / min / joint probability / tensor product
- OR: disjunction / max / union probability / direct sum
- IMPLIES: entailment / order / conditional probability / subspace

The DAG structure doesn't care which algebra is underneath. It
composes things with connectives. What those connectives *mean*
depends on the foundation you install. Classical logic is a
parameter of the system, not a hardwired assumption.

This matters because it means the architecture is sound even if
the foundation changes. A probabilistic extension (fuzzy types,
confidence intervals) wouldn't require rearchitecting the
composition layer — it would require a different foundation
algebra and kernel types, but Nodes and children and connectives
would still work.

