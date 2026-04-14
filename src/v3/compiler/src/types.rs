// Types flowing through Ports.
//
// Checkpoint C3.
//
// Started at 1 variant for M0.1 (Primitive only). Test 1 only needs
// Primitive(Int); Record/Sum/List/Function are deferred until a later
// test forces them.
//
// Highest-probability dissolution target:
//   { connective: Conn, children: Vec<(Label, TypeShape)> }
// with Primitive reinterpreted as { connective: Atom, children: [] }.
// That dissolution becomes valid only when std/ declares
// Product / Coproduct / Function as first-class structures — M1 work.
//
// STOP SIGNAL: adding Record, Sum, List, or Function variants. At that
// moment, pause and ask: is std/ ready to dissolve to
// { connective, children }, or does the scaffold extend by one?
// Neither answer is wrong; making the decision is what matters.
//
// DO NOT add Rust type-width assumptions (`Prim::Int` is symbolic, not
// `i64`). Target-agnostic types is guardrail G1.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeShape {
    Primitive(Prim),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prim {
    Int,
    Bool,
    String,
}
