//! Thin wrapper: delegates to `gunbc_resolve::resolve` with app-specific
//! extern symbol resolution.

use daglang_lower::LoweredOp;
use gunbc_exec::DynOp;
use gunbc_ir::Dag;

pub use gunbc_resolve::ResolveError;

/// Resolve a lowered DAG to concrete `DynOp` implementations.
///
/// Uses the app-specific runtime binding table for extern symbol dispatch.
pub fn resolve_lowered_dag(dag: &Dag<LoweredOp>) -> Result<Dag<DynOp>, ResolveError> {
    gunbc_resolve::resolve_lowered_dag_with(dag, crate::extern_ops::gunbc_runtime_bindings())
}
