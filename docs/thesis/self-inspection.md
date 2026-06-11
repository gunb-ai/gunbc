### Self-inspection: the substrate is its own subject

> Related: [epistemic-stacking.md](epistemic-stacking.md), [the-substrate-two-coordinated-shapes.md](the-substrate-two-coordinated-shapes.md), [compiler-std-consolidation.md](compiler-std-consolidation.md), [two-groundings-static-validation-vs-efficient-realization.md](two-groundings-static-validation-vs-efficient-realization.md)

The substrate's structural type layer is now declared inside the
substrate itself. `src/v3/std/substrate.dag` names `Dag`,
`Declaration`, `Behavior`, `TypeConnective`, the identity handles,
and the behavior payload records as ordinary `.dag` declarations,
and `src/v3/spec/rust.dag` attaches Rust-side realizations to those
same declarations. There is no separate meta-layer and no
`substrate_query.rs` bridge module. The substrate is described as
data inside itself, and the Rust compiler structs are one
realization of that data.

This matters because self-hosting is not only about compiling user
programs. The compiler has to inspect its own shape, enforce its own
invariants, and eventually express more of its own passes in `.dag`.
Once the substrate's own types are visible as declarations, every
consumer above them can read them through the same structural path:
typed declaration edges, field access, and realization bindings.
That is the fixed point the thesis has been aiming at. A lens over
the compiler substrate is no longer "special tooling" living
outside the model; it is an ordinary `.dag` consumer walking facts
the substrate already carries.

Self-inspection also tightens the two-groundings story. Static
grounding now includes the substrate's own shape, not only user
concepts. Realization grounding now includes the bridge from
reflected `.dag` fields to concrete Rust fields and accessor
methods. The consequence is practical: adding a new reflected
substrate field is a `.dag` declaration change plus a realization
binding, not a bespoke Rust-side query helper. The compiler grows by
surfacing facts structurally, not by accreting meta APIs.

