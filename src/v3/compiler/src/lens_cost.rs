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
        match generated::cost_of(self.dag, &port) {
            generated::CostLookup::FoundCost { _0: cost } => usize::try_from(cost)
                .expect("compiled complexity lens should emit a non-negative cost"),
            generated::CostLookup::MissingCost => panic!(
                "complexity lens returned MissingCost for port {port:?}; \
                 the DAG references a port whose producer is not in `d.nodes` \
                 and is not a bind parameter — malformed substrate"
            ),
        }
    }
}
