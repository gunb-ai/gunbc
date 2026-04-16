use crate::dag::{NodeId, PortId};
use crate::diagnostics::SourceSpan;
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

    include!("lens_unused_parameters_generated.rs");
}

#[derive(Debug, Clone, Default)]
pub struct UnusedParametersConfig {}

#[derive(Debug, Clone)]
pub struct UnusedParameter {
    pub function: NodeId,
    pub parameter: PortId,
    pub parameter_index: usize,
    pub function_span: SourceSpan,
}

pub struct UnusedParametersLens<'a> {
    dag: &'a Dag,
}

impl<'a> UnusedParametersLens<'a> {
    pub fn new(dag: &'a Dag) -> Self {
        Self { dag }
    }

    pub fn query(&self, _config: &UnusedParametersConfig) -> Vec<UnusedParameter> {
        generated::check(self.dag)
            .into_iter()
            .map(|violation| UnusedParameter {
                function: violation.function,
                parameter: violation.parameter,
                parameter_index: usize::try_from(violation.parameter_index)
                    .expect("compiled lens should emit non-negative parameter indexes"),
                function_span: violation.function_span,
            })
            .collect()
    }
}
