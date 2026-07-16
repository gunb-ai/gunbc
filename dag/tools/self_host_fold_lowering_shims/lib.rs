// seed-linked pilot lib — entry module self-emitted (gunbc output), seed-retained dep shims
#![allow(clippy::all, dead_code, unused_imports)]
#![recursion_limit = "256"]
pub mod v1_rt;
pub mod v2_std_node;
pub mod v2_std_collection;
pub mod v2_std_diagnostic;
pub mod v2_std_grammar;
pub mod v2_std_node_query;
pub mod v2_extdeps_languages_dag;
pub mod v2_compiler_fold_lowering {
    // Text-carrier wart: gunbc emits note fns as Rc<FreeMonoid<Char>> initialized from String.
    // Seed-linked host surface resolves here without editing the emitted entry body.
    pub type Char = i64;
    pub type FreeMonoid<T> = String;
    include!("v2_compiler_fold_lowering_emitted.rs");
}
pub use v1_compiler::NonEmptyVec;
pub use v1_compiler::NonEmptyBTreeSet;
