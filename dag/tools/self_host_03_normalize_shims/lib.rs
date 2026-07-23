// seed-linked pilot lib — narrow closure manifest for 03_normalize wet receipt.
// Post-#7057 closure grew (~48 emitted pub mods); this manifest keeps only the
// modules the witness crate rustc-links, with ABI-bridge shims for seed-retained
// deps and minimal std stubs for normalize's import surface.
#![allow(clippy::all, dead_code, unused_imports)]
#![recursion_limit = "512"]
pub use v1_compiler::NonEmptyVec;
pub use v1_compiler::NonEmptyBTreeSet;
pub use v1_compiler::v1_rt;
pub mod std_algebra;
pub mod std_types;
pub mod v2_std_integer;
pub mod v2_std_algebra;
pub mod v2_std_collection;
pub mod v2_std_grammar;
pub mod v2_std_diagnostic;
pub mod v2_std_node;
pub mod v2_std_compilers_sugar;
pub mod v2_compiler_body_lowering_fold;
pub mod v2_compiler_normalized_tree;
pub mod v2_extdeps_languages_dag;
pub mod v2_compiler_namespace_graft;
pub mod v2_compiler_normalize;
