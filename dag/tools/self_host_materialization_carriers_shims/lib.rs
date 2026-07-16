// seed-linked pilot lib — gunbc-emitted entry + seed std re-exports + emit-only dep modules
#![allow(clippy::all, dead_code, unused_imports)]
#![recursion_limit = "256"]

pub mod v1_rt;
pub mod v2_std_text;

pub use v1_compiler::{
    extdeps_external_authority, extdeps_uri, std_algebra, std_content_hash, std_decl_ref,
    std_disposition, std_effects, std_measure, std_pareto, std_realization_schedule, std_types,
    NonEmptyBTreeSet, NonEmptyVec,
};

pub mod extdeps_cache_catalog_io;
pub mod extdeps_cache_catalog_placement;
pub mod extdeps_cache_materialization;
pub mod extdeps_cache_types;
pub mod extdeps_realization_compile_stage_memo;
pub mod extdeps_realization_parse_table_memo;
pub mod std_cache_identity;
pub mod std_cache_interface;
pub mod std_magnitude;
pub mod std_materialization_ladder;
pub mod std_nat;
pub mod std_realization;
pub mod std_realization_measurement;
pub mod std_realization_width;
pub mod std_verification;
pub mod v2_std_diagnostic;
pub mod v2_std_node;
pub mod v2_std_staging;
pub mod v2_compiler_materialization_carriers;
