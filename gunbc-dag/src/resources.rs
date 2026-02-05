//! Repo-specific resource definitions for gunbc-dag.

use gunbc_ir::resource::{codegen_resource_def, InputPattern, ResourceDef};
use gunbc_ir::ResourceId;

/// Input globs that affect testgen outputs.
pub const TESTGEN_INPUT_GLOBS: &[&str] = &[
    "gunbc-dag/src/**/*.rs",
    "core/ir/src/**/*.rs",
    "lib/**/*.rs",
];

/// Resource definition for testgen outputs (`build:generated_tests`).
pub fn testgen_resource_def() -> ResourceDef {
    let mut def = ResourceDef::new(ResourceId::build("generated_tests"));

    for pattern in TESTGEN_INPUT_GLOBS {
        def = def.with_input(InputPattern::glob(*pattern));
    }

    // Testgen depends on codegen output key.
    let codegen_id = codegen_resource_def().id;
    def = def.with_input(InputPattern::resource(codegen_id));

    def
}
