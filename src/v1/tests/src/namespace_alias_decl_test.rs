//! §13 namespace alias-decl parse + typecheck-env binding witnesses.
//!
//! GREEN: `alias Binding = qualified.path` parses and binds without importing the
//! target module directly — bare references resolve through the alias.
//! RED: a dangling alias target is fail-closed (no fabricated binding).

use crate::helpers::{compile_multi_target, diagnostic_messages};
use v1_compiler::v1_compiler_compile::RenderTarget;
use v1_compiler::v1_compiler_infer::is_error_diagnostic;

const TARGET_LIB: &str = "module alias_fixture.target\n\
    type Carrier { x: Int }\n";

const GREEN_CONSUMER: &str = "module alias_fixture.consumer\n\
    alias CarrierAlias = alias_fixture.target.Carrier\n\
    fn echo(c: CarrierAlias) -> CarrierAlias { c }\n";

const DANGLING_ALIAS: &str = "module alias_fixture.dangling\n\
    alias MissingAlias = alias_fixture.nowhere.Missing\n\
    fn noop() -> Int { 0 }\n";

fn error_messages(result: &v1_compiler::v1_compiler_compile::PipelineResult) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| format!("{:?}", d.diagnostic))
        .collect()
}

#[test]
fn namespace_alias_resolves_without_direct_import() {
    let result = compile_multi_target(
        &[
            ("target.dag", TARGET_LIB),
            ("consumer.dag", GREEN_CONSUMER),
        ],
        RenderTarget::Dag,
    );
    let errors = error_messages(&result);
    assert!(
        errors.is_empty(),
        "alias to a type in a module not imported directly must typecheck; errors:\n{}\nall:\n{}",
        errors.join("\n"),
        diagnostic_messages(&result).join("\n")
    );
}

#[test]
fn namespace_alias_dangling_target_refuses() {
    let result = compile_multi_target(
        &[("target.dag", TARGET_LIB), ("dangling.dag", DANGLING_ALIAS)],
        RenderTarget::Dag,
    );
    let errors = error_messages(&result);
    assert!(
        !errors.is_empty(),
        "dangling alias target must refuse (fail-closed), got clean compile"
    );
}
