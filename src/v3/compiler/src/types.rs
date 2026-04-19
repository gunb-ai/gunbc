// Types flowing through Ports.
//
// **M1(2.6) terminal shape.** `TypeShape` is a thin newtype around
// `DeclarationId`: every port carries the identity of the declaration
// it refers to, and consumers walk the declaration table for any
// further information. There is no second type-identity layer; no
// `Prim::{Int, Bool, String}` enum, no name-keyed bridge back from
// declarations to primitive tags.
//
// History: M0 shipped `TypeShape::Primitive(Prim)` as a coarse
// port-level tag because the declaration table did not yet exist.
// M1(2.5) built the declaration table but did not touch the port
// representation, leaving `declaration_to_type_shape` as a name-keyed
// collapse at the port boundary. M1(2.6) eliminates that bridge by
// making `TypeShape` carry the declaration identity directly — the
// boundary is now structural on both sides.
//
// Dissolution ledger (4-pattern check, post-refactor):
// - Pattern 1 (fact placement): fails. Port types are load-bearing for
//   inference; scattering them off Port.state would duplicate dispatch.
// - Pattern 2 (variant-is-data): fails. TypeShape is one variant (a
//   newtype over DeclarationId); the underlying declaration carries
//   the structural shape.
// - Pattern 3 (algebraic form): the shape IS the declaration. Port
//   types are a pointer into the type substrate, not a separate
//   algebra.
// - Pattern 4 (dimensional): N/A (single-variant).
//
// Verdict: terminal at M1(2.6). Adding new port-level constructors
// (e.g., for linearity annotations, effect rows) would require
// re-opening the declaration-table-is-authority invariant.

use crate::dag::DeclarationId;

include!("types_generated.rs");
