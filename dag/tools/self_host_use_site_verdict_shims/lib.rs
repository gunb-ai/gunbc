// seed-linked pilot lib — entry module self-emitted, seed-retained dep shim
#![allow(clippy::all, dead_code, unused_imports)]
#![recursion_limit = "256"]
pub mod v1_rt;
pub mod v2_std_node;
pub mod v2_std_algebra;
pub mod v2_std_collection;
pub mod v2_compiler_use_site_verdict;
pub use v1_compiler::NonEmptyVec;
pub use v1_compiler::NonEmptyBTreeSet;
