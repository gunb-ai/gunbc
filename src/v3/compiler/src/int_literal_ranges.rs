use crate::dag::{AtomPayload, Behavior, Dag, DeclarationId, LiteralBits, PortId, TypeConnective};
use crate::diagnostics::{Correction, Diagnostic, SourceSpan};
use crate::types::TypeShape;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntegerRange {
    pub(crate) target_name: &'static str,
    pub(crate) min_decimal: &'static str,
    pub(crate) max_decimal: &'static str,
}

impl IntegerRange {
    fn min(self) -> i128 {
        self.min_decimal
            .parse()
            .expect("checked-in integer range minimum must parse as i128")
    }

    fn max(self) -> i128 {
        self.max_decimal
            .parse()
            .expect("checked-in integer range maximum must parse as i128")
    }

    pub(crate) fn contains_i64(self, value: i64) -> bool {
        let value = i128::from(value);
        self.min() <= value && value <= self.max()
    }
}

pub(crate) fn integer_range_for_decl(dag: &Dag, decl: DeclarationId) -> Option<IntegerRange> {
    let name = integer_decl_name(dag, decl, 0)?;
    match name {
        "Int8" => Some(IntegerRange {
            target_name: "Int8",
            min_decimal: "-128",
            max_decimal: "127",
        }),
        "Int16" => Some(IntegerRange {
            target_name: "Int16",
            min_decimal: "-32768",
            max_decimal: "32767",
        }),
        "Int32" => Some(IntegerRange {
            target_name: "Int32",
            min_decimal: "-2147483648",
            max_decimal: "2147483647",
        }),
        "Int" | "Int64" => Some(IntegerRange {
            target_name: "Int64",
            min_decimal: "-9223372036854775808",
            max_decimal: "9223372036854775807",
        }),
        "UInt8" => Some(IntegerRange {
            target_name: "UInt8",
            min_decimal: "0",
            max_decimal: "255",
        }),
        "UInt16" => Some(IntegerRange {
            target_name: "UInt16",
            min_decimal: "0",
            max_decimal: "65535",
        }),
        "UInt32" => Some(IntegerRange {
            target_name: "UInt32",
            min_decimal: "0",
            max_decimal: "4294967295",
        }),
        "UInt" | "UInt64" => Some(IntegerRange {
            target_name: "UInt64",
            min_decimal: "0",
            max_decimal: "18446744073709551615",
        }),
        _ => None,
    }
}

fn integer_decl_name(dag: &Dag, decl: DeclarationId, depth: usize) -> Option<&str> {
    if depth >= 32 {
        return None;
    }
    let declaration = dag.declaration(decl);
    if let Some(name) = declaration.name.as_deref() {
        return Some(name);
    }
    match &declaration.connective {
        TypeConnective::Atom(AtomPayload::ResolvedByName(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByStructure(next)) => {
            integer_decl_name(dag, *next, depth + 1)
        }
        TypeConnective::Instantiation { template, .. } => {
            integer_decl_name(dag, *template, depth + 1)
        }
        _ => None,
    }
}

pub(crate) fn literal_int_at(dag: &Dag, port: PortId) -> Option<i64> {
    match dag.resolve_producer_opt(&port)? {
        Behavior::Value(value) => match &value.data {
            LiteralBits::Int(n) => Some(*n),
            _ => None,
        },
        _ => None,
    }
}

pub(crate) fn int_literal_fits_expected_type(
    dag: &Dag,
    literal: i64,
    expected: DeclarationId,
) -> Option<bool> {
    integer_range_for_decl(dag, expected).map(|range| range.contains_i64(literal))
}

pub(crate) fn magnitude_out_of_range(
    literal: i64,
    expected: TypeShape,
    range: IntegerRange,
    span: SourceSpan,
) -> Diagnostic {
    let wider = wider_integer_target(literal);
    let hint = format!(
        "use a wider integer target such as `{wider}` or choose a literal in {}..={}",
        range.min_decimal, range.max_decimal
    );
    Diagnostic::MagnitudeOutOfRange {
        literal: literal.to_string(),
        target: range.target_name.to_string(),
        range_min_inclusive: range.min_decimal.to_string(),
        range_max_inclusive: range.max_decimal.to_string(),
        expected,
        span,
        fixes: vec![Correction {
            description: hint,
            span: SourceSpan::new("<int-literal-range-hint>", 0, 0),
            new_source: String::new(),
        }],
    }
}

fn wider_integer_target(literal: i64) -> &'static str {
    if literal < 0 {
        if i16::MIN as i64 <= literal && literal <= i16::MAX as i64 {
            "Int16"
        } else if i32::MIN as i64 <= literal && literal <= i32::MAX as i64 {
            "Int32"
        } else {
            "Int64"
        }
    } else if literal <= u16::MAX as i64 {
        "UInt16"
    } else if literal <= u32::MAX as i64 {
        "UInt32"
    } else {
        "UInt64"
    }
}
