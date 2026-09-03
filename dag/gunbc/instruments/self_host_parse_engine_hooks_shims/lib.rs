// seed-linked pilot lib — entry module self-emitted (gunbc output), seed-retained dep shim
#![allow(clippy::all, dead_code, unused_imports)]
#![recursion_limit = "256"]
pub mod v1_rt;
pub mod v2_std_node;
pub mod v2_compiler_parse_engine_hooks;
pub use v1_compiler::NonEmptyVec;
pub use v1_compiler::NonEmptyBTreeSet;
