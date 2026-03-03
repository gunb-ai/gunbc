//! Thin wrapper: delegates to `gunbc_resolve::resolve` with app-specific
//! extern symbol resolution.

use daglang_lower::LoweredOp;
use gunbc_exec::DynOp;
use gunbc_ir::Dag;
use gunbc_resolve::ExternResolver;

pub use gunbc_resolve::ResolveError;

/// App-specific extern resolver that dispatches to `extern_ops`.
struct GunbcExternResolver;

impl ExternResolver for GunbcExternResolver {
    fn resolve(&self, module: &str, name: &str) -> Option<DynOp> {
        crate::extern_ops::resolve_extern_symbol(module, name)
    }
}

/// Resolve a lowered DAG to concrete `DynOp` implementations.
///
/// Uses the app-specific `GunbcExternResolver` for extern symbol dispatch.
pub fn resolve_lowered_dag(dag: &Dag<LoweredOp>) -> Result<Dag<DynOp>, ResolveError> {
    gunbc_resolve::resolve_lowered_dag_with(dag, &GunbcExternResolver)
}
