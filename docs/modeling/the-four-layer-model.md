### The four-layer model

The compiler operates at four levels. Each layer is built on the one
below. No layer skips.

```
Surface sugar:      service, fn, type, operation    (user intent)
Composition layer:  Node, children, edges           (how things connect)
Semantic kernel:    types, effects, contracts        (what flows through nodes)
Foundation:         logical algebra                  (why it's sound)
```

**Foundation** — classical logic: truth values plus connectives (AND,
OR, NOT, IMPLIES) plus rules (associativity, commutativity, entailment).
This is the denotational ground truth. Not "a bit" (which is a carrier)
but a logical algebra over truth-valued structure. Everything else is
encoded composition.

**Semantic kernel** — types, effects, contracts: the structural algebra
that the compiler reasons about. Product (AND), Coproduct (OR), Refined
(AND with constraint), Function (IMPLIES). This is richer than raw logic
— the compiler works at this level, not at the bit level. But every
construct in the kernel is justified by the foundation.

**Composition layer** — Nodes and edges: the universal container for
connecting things. A Node composes semantic kernel objects (types, values,
effects) into a graph. The composition layer says HOW things connect.
The semantic kernel says WHAT is flowing through the connections.

**Surface sugar** — keywords and syntax: how the user expresses intent.
`service`, `fn`, `type` are ergonomic ways to say "build me a Node with
these structural properties." The sugar informs the parser what fields
to expect. It does not flow into the compiler core as identity.

