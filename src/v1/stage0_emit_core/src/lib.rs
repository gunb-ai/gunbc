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

pub use v1_stage0_core::{NonEmptyBTreeSet, NonEmptyVec};

pub mod v1_compiler_artifact {
    pub use v1_stage0_core::v1_compiler_artifact::*;
}

pub mod v1_compiler_infer_items {
    pub use v1_stage0_core::v1_compiler_infer_items::*;
}

pub mod v1_compiler_infer_service {
    pub use v1_stage0_core::v1_compiler_infer_service::*;
}

pub mod v1_compiler_infer_types {
    pub use v1_stage0_core::v1_compiler_infer_types::*;
}

pub mod v1_compiler_languages {
    pub use v1_stage0_core::v1_compiler_languages::*;
}

pub mod v1_rt {
    pub use v1_stage0_core::v1_rt::*;
}

pub mod v1_std_core {
    pub use v1_stage0_core::v1_std_core::*;
}

#[path = "../../stage0/src/v1_compiler_emit_core_support.rs"]
pub mod v1_compiler_emit_core_support;

pub use v1_compiler_emit_core_support::*;
