//! Serialize [`v3_compiler::dag::SymbolicCost`] into v3 surface syntax for `data …: SymbolicCost = …`
//! rows consumed by `SymbolicCostExprEquals` (gate **#40**).
//!
//! The initializer must parse under the same `std.algebra` / `std.substrate` / `std.list`
//! imports used by hand-authored verification fixtures; keep this serializer aligned with
//! `test_runner::field_value_to_symbolic_cost_eq_pattern` decoding rules.

use v3_compiler::dag::{DegreeAtLeastTwo, NonSingletonList, SymbolicCost};

/// Escape a UTF-8 string for embedding inside a v3 double-quoted `TestClaim.source` literal.
pub fn escape_v3_string_literal_content(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => {
                use std::fmt::Write as _;
                let _ = write!(&mut out, "\\u{{{:04x}}}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

fn degree_at_least_two_v3(d: &DegreeAtLeastTwo) -> String {
    match d {
        DegreeAtLeastTwo::DegreeTwo => "DegreeTwo".to_string(),
        DegreeAtLeastTwo::DegreeSuccessor { previous } => format!(
            "DegreeSuccessor {{ previous: {} }}",
            degree_at_least_two_v3(previous)
        ),
    }
}

fn symbolic_cost_list_tail_v3(terms: &[SymbolicCost]) -> String {
    assert!(
        terms.len() >= 2,
        "NonSingletonList serialization requires at least two terms"
    );
    if terms.len() == 2 {
        format!(
            "two_terms({}, {})",
            symbolic_cost_as_v3_data_initializer(&terms[0]),
            symbolic_cost_as_v3_data_initializer(&terms[1])
        )
    } else {
        let first = symbolic_cost_as_v3_data_initializer(&terms[0]);
        let second = symbolic_cost_as_v3_data_initializer(&terms[1]);
        let mut list = "empty()".to_string();
        for t in terms[2..].iter().rev() {
            list = format!(
                "cons({}, {})",
                symbolic_cost_as_v3_data_initializer(t),
                list
            );
        }
        format!("many_terms({}, {}, {})", first, second, list)
    }
}

/// Lower a runtime [`SymbolicCost`] to a v3 `SymbolicCost` data expression (no trailing semicolon).
pub fn symbolic_cost_as_v3_data_initializer(cost: &SymbolicCost) -> String {
    match cost {
        SymbolicCost::ConstantCost { _0: n } => format!("ConstantCost({n})"),
        SymbolicCost::LinearCost { _0: sv } => format!(
            "LinearCost(unnamed_size_variable(PortId({})))",
            sv.source_port.raw()
        ),
        SymbolicCost::LogCost { _0: sv } => format!(
            "LogCost(unnamed_size_variable(PortId({})))",
            sv.source_port.raw()
        ),
        SymbolicCost::PolynomialCost { var, degree } => format!(
            "PolynomialCost {{ var: unnamed_size_variable(PortId({})), degree: {} }}",
            var.source_port.raw(),
            degree_at_least_two_v3(degree)
        ),
        SymbolicCost::ProductCost { _0: nsl } => {
            let v: Vec<SymbolicCost> = nsl.to_vec().into_iter().map(|b| *b).collect();
            format!("ProductCost({})", symbolic_cost_list_tail_v3(&v))
        }
        SymbolicCost::SumCost { _0: nsl } => {
            let v: Vec<SymbolicCost> = nsl.to_vec().into_iter().map(|b| *b).collect();
            format!("SumCost({})", symbolic_cost_list_tail_v3(&v))
        }
        SymbolicCost::UnknownCost { _0: s } => {
            let esc = escape_v3_string_literal_content(s);
            format!("UnknownCost(\"{esc}\")")
        }
    }
}

#[cfg(test)]
mod symbolic_cost_verification_fixture_tests {
    use super::*;
    use v3_compiler::dag::{Dag, NonSingletonList, SizeVariable, SymbolicCost};

    #[test]
    fn list_tail_two_terms_emits_two_terms_call() {
        let a = SymbolicCost::ConstantCost { _0: 1 };
        let b = SymbolicCost::ConstantCost { _0: 2 };
        assert_eq!(
            symbolic_cost_list_tail_v3(&[a.clone(), b.clone()]),
            "two_terms(ConstantCost(1), ConstantCost(2))"
        );
        let got = symbolic_cost_as_v3_data_initializer(&SymbolicCost::SumCost {
            _0: NonSingletonList::from_vec(vec![Box::new(a), Box::new(b)]).expect("two terms"),
        });
        assert_eq!(got, "SumCost(two_terms(ConstantCost(1), ConstantCost(2)))");
    }

    #[test]
    fn linear_emits_unnamed_size_variable() {
        let dag = Dag::new();
        let p = dag
            .ports()
            .first()
            .expect("empty dag should still allocate ports")
            .id();
        let c = SymbolicCost::LinearCost {
            _0: SizeVariable {
                source_port: p,
                display_name: None,
            },
        };
        let expected = format!("LinearCost(unnamed_size_variable(PortId({})))", p.raw());
        assert_eq!(symbolic_cost_as_v3_data_initializer(&c), expected);
    }
}
