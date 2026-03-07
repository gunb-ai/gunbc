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
use gunbc_ir::ProgramSymbolId;

/// Concrete extern binding table.
///
/// This is the single runtime binding surface for extern symbols and
/// app-specific callables.
#[derive(Debug, Clone)]
pub struct RuntimeBindings {
    bindings: std::collections::HashMap<ProgramSymbolId, DynOp>,
}

impl RuntimeBindings {
    /// Create an empty binding table.
    pub fn new() -> Self {
        Self {
            bindings: std::collections::HashMap::new(),
        }
    }

    /// Register a concrete operation for a canonical program symbol.
    pub fn register_symbol(&mut self, symbol: impl Into<ProgramSymbolId>, op: DynOp) {
        self.bindings.insert(symbol.into(), op);
    }

    /// Register a concrete operation for an extern symbol.
    pub fn register(&mut self, module: impl Into<String>, name: impl Into<String>, op: DynOp) {
        let module = module.into();
        let name = name.into();
        self.register_symbol(ProgramSymbolId::from_parts(&module, &name), op);
    }

    /// Look up a binding by canonical symbol.
    pub fn get_symbol(&self, symbol: &ProgramSymbolId) -> Option<&DynOp> {
        self.bindings.get(symbol)
    }

    /// Resolve a binding by canonical symbol.
    pub fn resolve_symbol(&self, symbol: &ProgramSymbolId) -> Option<DynOp> {
        self.get_symbol(symbol).cloned()
    }

    /// Look up a binding by (module, name).
    pub fn get(&self, module: &str, name: &str) -> Option<&DynOp> {
        let symbol = ProgramSymbolId::from_parts(module, name);
        self.get_symbol(&symbol)
    }

    /// Resolve a binding by (module, name).
    pub fn resolve(&self, module: &str, name: &str) -> Option<DynOp> {
        self.get(module, name).cloned()
    }

    /// Check if any bindings are registered.
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}

impl Default for RuntimeBindings {
    fn default() -> Self {
        Self::new()
    }
}
