//! Emitter construction wall: modeled `Symbol` must ground to host `String` at the
//! single opaque-kernel alias authority (`coerce_primitive_type`), not as a parallel
//! newtype carrier that disagrees with `LitSymbol` / `^atom` value emission.

use crate::helpers::{assert_no_diagnostics, compile_dag_named};
use v1_compiler::v1_compiler_artifact::RenderTarget;

const FIXTURE: &str = concat!(
    "module symbol.fixture\n",
    "type Symbol\n",
    "type DiffId { id: Symbol }\n",
    "data root_anchor: Symbol = root_anchor\n",
    "fn symbol_param(x: Symbol) -> Symbol { x }\n",
    "fn string_param(x: String) -> String { x }\n"
);

#[test]
fn symbol_opaque_kernel_alias_grounds_type_and_value_to_string() {
    let result = compile_dag_named(
        "src/v1/symbol_opaque_kernel_alias_fixture.dag",
        FIXTURE,
        RenderTarget::Rust,
    );
    assert_no_diagnostics(&result);
    let emitted = result
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        emitted.contains("pub type Symbol = String;"),
        "Symbol decl must alias host String at coerce authority, got:\n{emitted}"
    );
    assert!(
        !emitted.contains("pub struct Symbol(pub String)"),
        "Symbol must not emit a parallel newtype carrier, got:\n{emitted}"
    );
    assert!(
        emitted.contains("pub id: String,") || emitted.contains("pub id: String"),
        "Symbol field types must ground to String, got:\n{emitted}"
    );
    assert!(
        emitted.contains("fn symbol_param(x: String) -> String"),
        "Symbol fn sig params must ground to String, got:\n{emitted}"
    );
}
