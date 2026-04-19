use crate::dag::Dag;
use crate::emit::{emit, emit_module, EmitTarget};
use crate::emit_rust::EmitError;

pub fn emit_go(dag: &Dag) -> Result<String, EmitError> {
    Ok(emit(dag, EmitTarget::Go)?.text)
}

pub fn emit_go_module(dag: &Dag) -> Result<String, EmitError> {
    Ok(emit_module(dag, EmitTarget::Go)?.text)
}
