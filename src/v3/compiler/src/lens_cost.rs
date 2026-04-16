use crate::dag::PortId;
use crate::Dag;

mod generated {
    #![allow(
        dead_code,
        unused_imports,
        unused_parens,
        unused_variables,
        clippy::clone_on_copy,
        clippy::collapsible_else_if
    )]

    use crate::dag::*;
    use crate::diagnostics::*;

    include!("lens_cost_generated.rs");
}

pub struct CostLens<'a> {
    dag: &'a Dag,
}

impl<'a> CostLens<'a> {
    pub fn new(dag: &'a Dag) -> Self {
        Self { dag }
    }

    pub fn cost_of(&self, port: PortId) -> usize {
        usize::try_from(generated::cost_of(self.dag, &port))
            .expect("compiled complexity lens should emit a non-negative cost")
    }
}
