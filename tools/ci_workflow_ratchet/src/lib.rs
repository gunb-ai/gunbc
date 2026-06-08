//! Host CI workflow ratchet — gate-57 / gunbc.ci compile receipts.
//!
//! PR-A relocation from frozen `src/v3/compiler/tests/integration/` (substitute/extract).
//! `compile_to_dag` usages in `lens_gate57/` tests relocate as-is; route-through-v2
//! re-point is tracked follow-up (ctrl#1467 frame).

pub mod support;
