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
<<<<<<< HEAD
    use super::{UnusedParameter, UnusedParametersConfig, UnusedParametersLens};
    use crate::dag::{
        ArithmeticOp, BranchPattern, ComparisonOp, Dag, LiteralBits, OperatorKind, Path, PortId,
        TransformTarget,
    };
    use crate::diagnostics::SourceSpan;

    fn span() -> SourceSpan {
        SourceSpan::new("<lens-unused-parameters-test>", 0, 0)
    }

    fn query(dag: &Dag) -> Vec<UnusedParameter> {
        UnusedParametersLens::new(dag).query(&UnusedParametersConfig::default())
    }

    fn int_param(dag: &mut Dag) -> PortId {
        let int_shape = dag.int_shape().expect("bootstrap Int");
        dag.alloc_port_with_shape(int_shape)
    }

    #[test]
    fn all_used_parameters_report_zero_violations() {
        // fn add(a: Int, b: Int) -> Int = a + b
        let mut dag = Dag::new();
        let a = int_param(&mut dag);
        let b = int_param(&mut dag);
        let body = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Arithmetic(ArithmeticOp::Add)),
            vec![a, b],
            span(),
        );
        let _ = dag.push_bind("add", body, vec![a, b], span());

        assert!(query(&dag).is_empty());
    }

    #[test]
    fn single_unused_parameter_is_reported_at_its_index() {
        // fn first(a: Int, b: Int) -> Int = a — b is unused at index 1.
        let mut dag = Dag::new();
        let a = int_param(&mut dag);
        let b = int_param(&mut dag);
        let bind = dag.push_bind("first", a, vec![a, b], span());

        let violations = query(&dag);
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].function, bind);
        assert_eq!(violations[0].parameter, b);
        assert_eq!(violations[0].parameter_index, 1);
    }

    #[test]
    fn constant_body_reports_every_parameter_in_order() {
        // fn always_one(x, y, z) = 1 — all three params unused.
        let mut dag = Dag::new();
        let x = int_param(&mut dag);
        let y = int_param(&mut dag);
        let z = int_param(&mut dag);
        let body = dag.push_value(LiteralBits::Int(1), span());
        let bind = dag.push_bind("always_one", body, vec![x, y, z], span());

        let violations = query(&dag);
        assert_eq!(violations.len(), 3);
        assert!(violations.iter().all(|v| v.function == bind));
        assert_eq!(violations[0].parameter_index, 0);
        assert_eq!(violations[1].parameter_index, 1);
        assert_eq!(violations[2].parameter_index, 2);
        assert_eq!(violations[0].parameter, x);
        assert_eq!(violations[1].parameter, y);
        assert_eq!(violations[2].parameter, z);
    }

    #[test]
    fn parameters_referenced_only_through_a_branch_arm_are_considered_used() {
        // fn pick(a: Int, b: Int) -> Int = if a > 0 then a else b
        let mut dag = Dag::new();
        let a = int_param(&mut dag);
        let b = int_param(&mut dag);
        let zero = dag.push_value(LiteralBits::Int(0), span());
        let cond = dag.push_transform(
            TransformTarget::Operator(OperatorKind::Comparison(ComparisonOp::Gt)),
            vec![a, zero],
            span(),
        );
        let then_body = dag.push_bind("then_arm", a, Vec::new(), span());
        let else_body = dag.push_bind("else_arm", b, Vec::new(), span());
        let branch = dag.push_branch(
            cond,
            vec![
                Path {
                    body: then_body,
                    output: a,
                    pattern: BranchPattern::UnresolvedVariant {
                        name: "Then".to_string(),
                        span: span(),
                    },
                    binding: None,
                },
                Path {
                    body: else_body,
                    output: b,
                    pattern: BranchPattern::UnresolvedVariant {
                        name: "Else".to_string(),
                        span: span(),
                    },
                    binding: None,
                },
            ],
            span(),
        );
        let _ = dag.push_bind("pick", branch, vec![a, b], span());

        assert!(query(&dag).is_empty());
    }

    #[test]
    fn value_and_transform_bodyless_binds_without_params_are_not_reported() {
        // A bind with zero params must never produce a violation — the
        // emitted lens short-circuits `check_behavior` on empty params.
        let mut dag = Dag::new();
        let body = dag.push_value(LiteralBits::Int(42), span());
        let _ = dag.push_bind("answer", body, Vec::new(), span());

        assert!(query(&dag).is_empty());
=======
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
        dag.push_value(LiteralBits::Int(value), span())
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
>>>>>>> 5ec214d6e (WIP: Worker 7)
    }
}
