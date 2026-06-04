//! Crate boundary proof for generated v2 emit core support.
//!
//! The semantic bodies remain generated from `src/v2/05_emit_core_support.dag`
//! into the stage0 seed. This crate exposes that generated support surface
//! through the same type authorities as `v2-compiler`, avoiding copied Rust
//! semantics while the lower stage0 core crates are still unsplit.

#![allow(
    unused_imports,
    unused_variables,
    unused_mut,
    unused_parens,
    dead_code,
    unreachable_patterns,
    non_shorthand_field_patterns,
    suspicious_double_ref_op,
    clippy::all
)]

pub use v2_compiler::{NonEmptyBTreeSet, NonEmptyVec};

pub mod v2_compiler_artifact {
    pub use v2_compiler::v2_compiler_artifact::*;
}

pub mod v2_compiler_infer_items {
    pub use v2_compiler::v2_compiler_infer_items::*;
}

pub mod v2_compiler_infer_service {
    pub use v2_compiler::v2_compiler_infer_service::*;
}

pub mod v2_compiler_infer_types {
    pub use v2_compiler::v2_compiler_infer_types::*;
}

pub mod v2_compiler_languages {
    pub use v2_compiler::v2_compiler_languages::*;
}

pub mod v2_rt {
    pub use v2_compiler::v2_rt::*;
}

pub mod v2_std_core {
    pub use v2_compiler::v2_std_core::*;
}

#[path = "../../stage0/src/v2_compiler_emit_core_support.rs"]
pub mod v2_compiler_emit_core_support;

pub use v2_compiler_emit_core_support::*;
