pub use crate::emit::rust_target::{EmitError, RealizationCategory, SubstrateMarkerRole};
use crate::emit::{emit, emit_module, EmitDispatchError, EmitTarget};
use crate::Dag;

pub fn emit_rust(dag: &Dag) -> Result<String, EmitError> {
    match emit(dag, EmitTarget::Rust) {
        Ok(source) => Ok(source.text),
        Err(EmitDispatchError::Core(error)) => Err(error),
        Err(EmitDispatchError::Python(_)) => {
            unreachable!("EmitTarget::Rust cannot yield a Python emission error")
        }
    }
}

pub fn emit_rust_module(dag: &Dag) -> Result<String, EmitError> {
    match emit_module(dag, EmitTarget::Rust) {
        Ok(source) => Ok(source.text),
        Err(EmitDispatchError::Core(error)) => Err(error),
        Err(EmitDispatchError::Python(_)) => {
            unreachable!("EmitTarget::Rust cannot yield a Python emission error")
        }
    }
}
