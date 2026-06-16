# src/v2/experimental — quarantine for consumer-less models

Per DESIGN.md §5 (fail-closed — a typecheck/`.contains()` is not a consumer): a model /
function / type / field with **no real consumer** — nothing that breaks when its behavior is
wrong — does not live in the active tree. It is quarantined here until a consumer exists, then
promoted back out (with that consumer, in the same change).

This is **experimental / less-supported** work: not gated, not relied upon, and possibly wrong
— it has never had to run correctly. Treat it as a map (a design sketch), not territory.

Rules:
- **Nothing in the active tree may import from `experimental/`.** If active code needs something
  here, promote it out *with its consumer* in the same change.
- **No module-path collisions with the active tree.** Experimental `.dag` use the
  `v2.experimental.*` module namespace so the bootstrap/index never confuses them with live modules.
- **Routing here is git-recoverable and requires no reasoning.** Prefer it over auditing or
  porting consumer-less code (the trap).
- A review that finds a new model with no consumer **routes the code, blocks the claim**: the
  author either wires a real consumer, moves the model here, or escalates to the operator to
  force it into the active tree.
