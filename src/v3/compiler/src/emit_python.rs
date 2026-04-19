pub use crate::emit::python_target::EmitPythonError;
use crate::emit::{emit, emit_module, EmitDispatchError, EmitTarget};
use crate::Dag;

pub fn emit_python(dag: &Dag) -> Result<String, EmitPythonError> {
    match emit(dag, EmitTarget::Python) {
        Ok(source) => Ok(source.text),
        Err(EmitDispatchError::Python(error)) => Err(error),
        Err(EmitDispatchError::Core(_)) => {
            unreachable!("EmitTarget::Python cannot yield a core emission error")
        }
    }
}

pub fn emit_python_module(dag: &Dag) -> Result<String, EmitPythonError> {
    match emit_module(dag, EmitTarget::Python) {
        Ok(source) => Ok(source.text),
        Err(EmitDispatchError::Python(error)) => Err(error),
        Err(EmitDispatchError::Core(_)) => {
            unreachable!("EmitTarget::Python cannot yield a core emission error")
        }
    }
}
