pub mod builder;
pub mod dry_run;
pub mod fs_env;
pub mod resolve;
pub mod service_ops;

pub use builder::{BuildOpts, DslGraphResult};
pub use dry_run::wire_fs_env_write_mock;
pub use fs_env::{add_fs_env_root_node, wire_fs_env_write_edges};
pub use resolve::{resolve_lowered_dag_with, ResolveError};

use gunbc_exec::DynOp;

/// Trait for resolving extern symbols to concrete `DynOp` implementations.
///
/// The resolver is called for:
/// 1. Domain-specific callables in `resolve_domain()` (module + name)
/// 2. `ExternCall` resolution in `resolve_extern_call()` (module + name)
///
/// Return `None` to fall through to the default passthrough handling.
pub trait ExternResolver: Send + Sync {
    fn resolve(&self, module: &str, name: &str) -> Option<DynOp>;
}

/// Resolver that never resolves any extern symbols.
pub struct NullExternResolver;

impl ExternResolver for NullExternResolver {
    fn resolve(&self, _module: &str, _name: &str) -> Option<DynOp> {
        None
    }
}
