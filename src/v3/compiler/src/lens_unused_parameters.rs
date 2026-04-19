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
    }
}
