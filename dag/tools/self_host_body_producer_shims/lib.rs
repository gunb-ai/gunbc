// seed-linked pilot lib — entry module gunbc-emitted, seed-retained dep shims
#![allow(clippy::all, dead_code, unused_imports)]
#![recursion_limit = "256"]
pub mod v1_rt;
pub mod v2_std_algebra;
pub mod v2_std_diagnostic;
pub mod v2_std_node;
pub mod v2_compiler_body_producer;
pub use v1_compiler::NonEmptyVec;
pub use v1_compiler::NonEmptyBTreeSet;
