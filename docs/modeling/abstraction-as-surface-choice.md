### Abstraction as surface choice

The core is fixed: AND, OR, NOT, IMPLIES, composition, grounding.
The surface is a choice. Different communities can work at whatever
level of abstraction they find appropriate, and it all compiles
down to the same logical structure.

```
Surface                  Abstraction level        Compiles to
───────────────────     ─────────────────────    ──────────────────
∀x ∈ S: P(x)            set theory / functions   AND over predicates
all(items, p => valid)   developer / collections  fold with AND
type T = A | B           developer / types        OR of variants
service git.Core { }     developer / services     AND(transport, children)
pipeline build { }       domain / orchestration   chained IMPLIES with guards
drag-and-drop graph      visual / no code         Node + edges
```

The compiler doesn't care which surface produced a Node. It sees
the logical structure. This means:

- **Mathematicians** can operate at set theory / function level.
  `{ x ∈ S | P(x) }` is `filter(S, P)` is `S ∩ P` is AND. They
  work with the foundation directly.

- **Developers** can work with types, functions, services, resources.
  `type`, `fn`, `service` are ergonomic keywords that produce Nodes
  with specific structural properties. They work with the surface.

- **Domain experts** can define their own abstractions. A finance
  team defines `ledger`, `transaction`, `settlement` keywords that
  produce Nodes with domain-appropriate constraints. New surface,
  same core.

- **Visual builders** can compose graphs without text. A node editor
  that connects boxes with wires produces the same Node + children +
  connective structure. Different surface, same compiler.

The key: no level is more "real" than another. `service git.Core`
and `AND(transport, children)` are the same proposition. The surface
determines ergonomics. The core determines semantics. Adding a new
abstraction level means writing a new surface (parser), not changing
the compiler.

This is the same architecture as hardware:
- A physicist models transistor characteristics (foundation)
- A digital designer works in gates and flip-flops (semantic kernel)
- A system architect draws block diagrams (composition)
- An FPGA user configures in a GUI (surface sugar)

All synthesize to the same silicon. The abstraction level is a
human choice. The physics is fixed.

