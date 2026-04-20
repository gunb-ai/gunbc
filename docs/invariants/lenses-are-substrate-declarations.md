## Lenses Are Substrate Declarations

The canonical form of a lens is a `.dag` declaration operating over the
reflected substrate, not a permanently hand-written Rust module. Rust
lenses are tolerated only as bootstrap scaffolds while the remaining
execution path for compiled lenses is being finished.

This invariant has two operational consequences:

- New lens features should prefer substrate-level facts and realization
  bindings over Rust-side helper APIs.
- The number of Rust bootstrap lenses is itself a ratchet. Deleting a
  Rust lens in favor of its `.dag` form is forward progress; adding a
  new permanent Rust-only lens is not.

The point is not aesthetic. A lens declared in the substrate can be
type-checked, realized, diffed, and eventually analyzed by other lenses.
A Rust lens is opaque to the substrate and therefore a standing hole in
the self-inspection claim.

### Reflected facts: when a boundary counts as “landed”

A reflected substrate fact (field + accessor + realization) may be marked
**shipped** / **debt-cleared** only when **all** of the following hold:

1. **Declaration + realization** — `.dag` authority and target spec
   bindings (e.g. `rust.dag` / `SubstrateAccessorBinding`) for the active
   language(s), with **no storage-shape leak** at the realization boundary
   (see “Explicit boundary contracts”).
2. **Generated consumer proof** — at least one regression that **emits**
   (or otherwise compiles) a **declared** `.dag` consumer and **runs** it
   linked against the compiler so the **declared** accessor or field path is
   exercised end-to-end. Binding-count-only smoke or **only** hand-written
   calls into a crate oracle (without going through emitted lens code) do
   **not** satisfy this bar.

This matches the “new semantic boundaries must land with a real downstream
consumer” discipline: **declaration alone is not consumption.**

