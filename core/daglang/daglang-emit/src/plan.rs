//! Topo-ordered emission plan with data flow bindings.
//!
//! Built from a lowered DAG + Computation classifications.
//! Every codegen path consumes the same `EmitPlan`.
//!
//! **Owned by**: Task 4 (dsl-codegen-tasks.md)
