use crate::dag::Dag;
use crate::emit::{emit, emit_module, EmitDispatchError, EmitTarget};
use crate::emit_rust::EmitError;

pub fn emit_go(dag: &Dag) -> Result<String, EmitError> {
    match emit(dag, EmitTarget::Go) {
        Ok(source) => Ok(source.text),
        Err(EmitDispatchError::Core(error)) => Err(error),
        Err(EmitDispatchError::Python(_)) => {
            unreachable!("EmitTarget::Go cannot yield a Python emission error")
        }
    }
}

pub fn emit_go_module(dag: &Dag) -> Result<String, EmitError> {
    match emit_module(dag, EmitTarget::Go) {
        Ok(source) => Ok(source.text),
        Err(EmitDispatchError::Core(error)) => Err(error),
        Err(EmitDispatchError::Python(_)) => {
            unreachable!("EmitTarget::Go cannot yield a Python emission error")
        }
    }
}
