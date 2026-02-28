//! DSL-backed deps tool — thin delegate to `dsl_builder::build_tool_graph`.

use gunbc_exec::DynOp;
use gunbc_ir::{BuilderError, Dag};

pub type DepsGraphOp = DynOp;

pub fn build_deps_graph() -> Result<Dag<DepsGraphOp>, BuilderError> {
    crate::dsl_builder::build_tool_graph("deps")
}
