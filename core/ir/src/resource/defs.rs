//! Built-in resource definitions.
//!
//! These are shared resource declarations used across the repo to ensure
//! hashing inputs stay centralized and consistent.

use super::{InputPattern, ResourceDef};
use crate::ResourceId;

/// Input globs that affect codegen outputs.
pub const CODEGEN_INPUT_GLOBS: &[&str] = &[
    "core/codegen/src/**/*.rs",
    "core/ir/src/**/*.rs",
];

/// Individual files that affect codegen outputs.
pub const CODEGEN_INPUT_FILES: &[&str] = &[
    "core/codegen/Cargo.toml",
    "core/ir/Cargo.toml",
];

/// Environment variables that affect codegen outputs.
pub const CODEGEN_INPUT_ENVS: &[&str] = &["RUSTC_VERSION"];

/// Resource definition for codegen outputs (`build:generated_cli`).
pub fn codegen_resource_def() -> ResourceDef {
    let mut def = ResourceDef::new(ResourceId::build("generated_cli"));

    for pattern in CODEGEN_INPUT_GLOBS {
        def = def.with_input(InputPattern::glob(*pattern));
    }
    for path in CODEGEN_INPUT_FILES {
        def = def.with_input(InputPattern::file(*path));
    }
    for var in CODEGEN_INPUT_ENVS {
        def = def.with_input(InputPattern::env(*var));
    }

    def
}
