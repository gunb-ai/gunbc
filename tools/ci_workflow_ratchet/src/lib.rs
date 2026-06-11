//! Host CI workflow ratchet — byte-mirror, SHA pin, ci.yml binding, dag↔Rust parity.
//!
//! PR-A relocation from frozen `src/v3/compiler/tests/integration/` (substitute/extract).
//! `compile_to_dag` usages in `lens_gate57/` tests relocate as-is; route-through-v2
//! re-point is tracked follow-up (ctrl#1467 frame).

pub mod support;
