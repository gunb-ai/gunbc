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

#[cfg(test)]
mod tests {
    use super::{UnusedParametersConfig, UnusedParametersLens};
    use crate::dag::{BranchPattern, Dag, LiteralBits, LoopBound, Path, PortId, TransformTarget};
    use crate::diagnostics::SourceSpan;
    use crate::operators::{ArithmeticOp, ComparisonOp, OperatorKind};

    const DIRECT_DAG_FILE: &str = "lens_unused_parameters.unit";

    fn span() -> SourceSpan {
        SourceSpan::new(DIRECT_DAG_FILE, 0, 0)
    }

    fn unused_parameter_indexes(dag: &Dag) -> Vec<usize> {
        let lens = UnusedParametersLens::new(dag);
        let mut indexes: Vec<_> = lens
            .query(&UnusedParametersConfig::default())
            .into_iter()
            .map(|violation| violation.parameter_index)
            .collect();
        indexes.sort_unstable();
        indexes
    }

    fn int_value(dag: &mut Dag, value: i64) -> PortId {
        dag.push_value(LiteralBits::Int(value.into()), span())
    }

    fn add(dag: &mut Dag, lhs: PortId, rhs: PortId) -> PortId {
        dag.push_transform(
            TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            vec![lhs, rhs],
            span(),
        )
    }

    fn gt(dag: &mut Dag, lhs: PortId, rhs: PortId) -> PortId {
        dag.push_transform(
            TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Gt)),
            vec![lhs, rhs],
            span(),
        )
    }

    fn int_params(dag: &mut Dag, count: usize) -> Vec<PortId> {
        let int_shape = dag.int_shape().expect("bootstrap Dag has Int");
        (0..count)
            .map(|_| dag.alloc_port_with_shape(int_shape))
            .collect()
    }

    fn producer_or_bind(dag: &mut Dag, name: &str, output: PortId) -> crate::dag::NodeId {
        dag.port(output)
            .produced_by
            .unwrap_or_else(|| dag.push_bind(name, output, Vec::new(), span()))
    }

    fn bind_arm(dag: &mut Dag, name: &str, output: PortId) -> Path {
        Path {
            body: producer_or_bind(dag, name, output),
            output,
            pattern: BranchPattern::UnresolvedVariant {
                name: name.to_string(),
                span: span(),
            },
            binding: None,
        }
    }

    fn function_dag<F>(name: &str, param_count: usize, build_body: F) -> Dag
    where
        F: FnOnce(&mut Dag, &[PortId]) -> PortId,
    {
        let mut dag = Dag::new();
        let params = int_params(&mut dag, param_count);
        let value = build_body(&mut dag, &params);
        dag.push_bind(name, value, params, span());
        dag
    }

    #[test]
    fn unused_params_empty_for_function_using_every_parameter() {
        let dag = function_dag("add", 2, |dag, params| add(dag, params[0], params[1]));

        assert_eq!(unused_parameter_indexes(&dag), Vec::<usize>::new());
    }

    #[test]
    fn unused_params_reports_single_unused_parameter() {
        let dag = function_dag("first", 2, |_dag, params| params[0]);

        assert_eq!(unused_parameter_indexes(&dag), vec![1]);
    }

    #[test]
    fn unused_params_reports_all_parameters_for_constant_body() {
        let dag = function_dag("always_one", 3, |dag, _params| int_value(dag, 1));

        assert_eq!(unused_parameter_indexes(&dag), vec![0, 1, 2]);
    }

    #[test]
    fn unused_params_skips_value_bindings() {
        let mut dag = Dag::new();
        let lhs = int_value(&mut dag, 1);
        let rhs = int_value(&mut dag, 2);
        let value = add(&mut dag, lhs, rhs);
        dag.push_bind("x", value, Vec::new(), span());

        assert_eq!(unused_parameter_indexes(&dag), Vec::<usize>::new());
    }

    #[test]
    fn unused_params_handles_branch_in_body() {
        let dag = function_dag("pick", 2, |dag, params| {
            let zero = int_value(dag, 0);
            let cond = gt(dag, params[0], zero);
            let then_path = bind_arm(dag, "then_arm", params[0]);
            let else_path = bind_arm(dag, "else_arm", params[1]);
            dag.push_branch(cond, vec![then_path, else_path], span())
        });

        assert_eq!(unused_parameter_indexes(&dag), Vec::<usize>::new());
    }

    #[test]
    fn unused_params_bootstrap_baseline_is_empty() {
        let dag = Dag::new();
        assert_eq!(unused_parameter_indexes(&dag), Vec::<usize>::new());
    }

    #[test]
    fn unused_params_reports_unused_in_branch_body() {
        let dag = function_dag("always_a", 2, |dag, params| {
            let zero = int_value(dag, 0);
            let cond = gt(dag, params[0], zero);
            let then_path = bind_arm(dag, "then_arm", params[0]);
            let else_path = bind_arm(dag, "else_arm", params[0]);
            dag.push_branch(cond, vec![then_path, else_path], span())
        });

        assert_eq!(unused_parameter_indexes(&dag), vec![1]);
    }

    #[test]
    fn unused_params_descends_into_loop_body_for_recursive_calls() {
        let dag = function_dag("count_down", 2, |dag, params| {
            let body_output = add(dag, params[0], params[1]);
            let count = int_value(dag, 1);
            let body = producer_or_bind(dag, "loop_body", body_output);
            dag.push_loop(
                params[0],
                params[1],
                body,
                LoopBound::Cardinality { count },
                span(),
            )
        });

        assert_eq!(unused_parameter_indexes(&dag), Vec::<usize>::new());
    }

    #[test]
    fn unused_params_loop_body_descent_finds_param_only_used_in_recursion() {
        let dag = function_dag("count_down", 2, |dag, params| {
            let body_output = add(dag, params[0], params[1]);
            let init = int_value(dag, 0);
            let count = int_value(dag, 1);
            let body = producer_or_bind(dag, "loop_body", body_output);
            dag.push_loop(
                params[0],
                init,
                body,
                LoopBound::Cardinality { count },
                span(),
            )
        });

        assert_eq!(unused_parameter_indexes(&dag), Vec::<usize>::new());
    }
}
