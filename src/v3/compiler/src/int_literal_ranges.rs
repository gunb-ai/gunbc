use crate::dag::{
    AtomPayload, Behavior, Dag, DeclarationId, FieldValue, LiteralBits, PortId, TypeConnective,
    ValueBody,
};
use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::types::TypeShape;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IntegerRange {
    pub(crate) target_name: String,
    pub(crate) min_decimal: String,
    pub(crate) max_decimal: String,
    min: i128,
    max: i128,
}

impl IntegerRange {
    /// Current reconciliation receives only source literals that already
    /// fit the existing `LiteralBits::Int(i64)` carrier. The declared
    /// range facts remain full Rust target ranges (including u64's upper
    /// half); literals above `i64::MAX` are rejected earlier by the
    /// tokenizer until the deferred carrier-widening lane replaces the
    /// source literal carrier.
    pub(crate) fn contains_i64(&self, value: i64) -> bool {
        let value = i128::from(value);
        self.min <= value && value <= self.max
    }
}

pub(crate) enum IntegerRangeLookup {
    Found(IntegerRange),
    Missing,
    Invalid(Diagnostic),
}

/// Structural witness for routing integer literals: `IntegerAlgebra` and
/// `TargetCarrier` variant **payload type** ids (the `constructor` field on
/// `FieldValue::Variant`), derived from std `OrderedRing<C>` / `Semiring<C>`
/// by `DeclarationId` equality on template and carrier type declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntegerRoutingWitness {
    pub(crate) algebra_variant_ty: DeclarationId,
    pub(crate) carrier_variant_ty: DeclarationId,
}

/// Integration / structural tests: witness for `decl`'s resolved integer
/// instantiation (`OrderedRing` / `Semiring` + word carrier), if any.
pub(crate) fn integer_routing_witness_for_decl(
    dag: &Dag,
    decl: DeclarationId,
) -> Option<IntegerRoutingWitness> {
    integer_routing_witness_walk(dag, decl, 0)
}

pub(crate) fn integer_range_for_decl(dag: &Dag, decl: DeclarationId) -> IntegerRangeLookup {
    let Some(witness) = integer_routing_witness_walk(dag, decl, 0) else {
        return IntegerRangeLookup::Missing;
    };
    let Some(pilot) = dag.rust_pilot_primitives() else {
        return IntegerRangeLookup::Missing;
    };
    let Some(body) = pilot.value_body.as_ref() else {
        return IntegerRangeLookup::Missing;
    };
    let ValueBody::List(elements) = body else {
        return IntegerRangeLookup::Missing;
    };

    let integer_primitive_ctor = match rust_primitive_integer_variant_ty(dag) {
        Some(id) => id,
        None => {
            return IntegerRangeLookup::Invalid(malformed_integer_range_fact(
                "bootstrap: RustPrimitive.IntegerPrimitive variant type is unavailable".to_string(),
                pilot.span.clone(),
            ));
        }
    };

    let mut matches: Vec<PilotIntegerMatch> = Vec::new();
    for element in elements {
        let FieldValue::Variant {
            constructor,
            payload,
        } = element
        else {
            continue;
        };
        if *constructor != integer_primitive_ctor {
            continue;
        }
        match pilot_integer_row(witness, payload, pilot.span.clone()) {
            Ok(Some(m)) => matches.push(m),
            Ok(None) => {}
            Err(diag) => return IntegerRangeLookup::Invalid(diag),
        }
    }

    if matches.is_empty() {
        return IntegerRangeLookup::Missing;
    }
    if let Some(row) = matches.iter().find(|row| row.range.is_none()) {
        return IntegerRangeLookup::Invalid(malformed_integer_range_fact(
            format!(
                "malformed rust_pilot_primitives IntegerPrimitive row for routing witness {:?}; integer literal range narrowing is unavailable",
                (witness.algebra_variant_ty, witness.carrier_variant_ty)
            ),
            row.span.clone(),
        ));
    }
    if matches.len() > 1 {
        return IntegerRangeLookup::Invalid(malformed_integer_range_fact(
            format!(
                "duplicate rust_pilot_primitives IntegerPrimitive rows for routing witness {:?}; integer literal range narrowing is ambiguous",
                (witness.algebra_variant_ty, witness.carrier_variant_ty)
            ),
            matches[1].span.clone(),
        ));
    }
    let row = matches.remove(0);
    match row.range {
        Some(range) => IntegerRangeLookup::Found(range),
        None => unreachable!("malformed matching rows are handled before duplicate detection"),
    }
}

struct PilotIntegerMatch {
    range: Option<IntegerRange>,
    span: SourceSpan,
}

fn integer_routing_witness_walk(
    dag: &Dag,
    decl: DeclarationId,
    depth: usize,
) -> Option<IntegerRoutingWitness> {
    if depth >= 32 {
        return None;
    }
    let declaration = dag.declaration(decl);
    match &declaration.connective {
        TypeConnective::Atom(AtomPayload::ResolvedByName(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByStructure(next)) => {
            integer_routing_witness_walk(dag, *next, depth + 1)
        }
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            if arguments.is_empty() {
                return integer_routing_witness_walk(dag, *template, depth + 1);
            }
            let carrier = arguments.first()?.value;
            integer_instantiation_witness(dag, *template, carrier)
        }
        _ => None,
    }
}

fn integer_instantiation_witness(
    dag: &Dag,
    template: DeclarationId,
    carrier_decl: DeclarationId,
) -> Option<IntegerRoutingWitness> {
    let ordered_ring = dag.declaration_by_name("OrderedRing")?.id;
    let semiring = dag.declaration_by_name("Semiring")?.id;
    let algebra_variant_ty = if template == ordered_ring {
        disj_variant_payload_ty(dag, "IntegerAlgebra", "OrderedRingAlgebra")?
    } else if template == semiring {
        disj_variant_payload_ty(dag, "IntegerAlgebra", "SemiringAlgebra")?
    } else {
        return None;
    };
    let carrier_variant_ty = std_word_carrier_to_target_carrier_variant_ty(dag, carrier_decl)?;
    Some(IntegerRoutingWitness {
        algebra_variant_ty,
        carrier_variant_ty,
    })
}

fn disj_variant_payload_ty(
    dag: &Dag,
    sum_name: &str,
    variant_label: &str,
) -> Option<DeclarationId> {
    let decl = dag.declaration_by_name(sum_name)?;
    let TypeConnective::Disj { variants } = &decl.connective else {
        return None;
    };
    variants
        .iter()
        .find(|v| v.label == variant_label)
        .map(|v| v.ty)
}

fn std_word_carrier_to_target_carrier_variant_ty(
    dag: &Dag,
    carrier_decl: DeclarationId,
) -> Option<DeclarationId> {
    let byte = dag.declaration_by_name("Byte")?.id;
    let word16 = dag.declaration_by_name("Word16")?.id;
    let word32 = dag.declaration_by_name("Word32")?.id;
    let word64 = dag.declaration_by_name("Word64")?.id;
    let label = if carrier_decl == byte {
        "ByteCarrier"
    } else if carrier_decl == word16 {
        "Word16Carrier"
    } else if carrier_decl == word32 {
        "Word32Carrier"
    } else if carrier_decl == word64 {
        "Word64Carrier"
    } else {
        return None;
    };
    disj_variant_payload_ty(dag, "TargetCarrier", label)
}

fn rust_primitive_integer_variant_ty(dag: &Dag) -> Option<DeclarationId> {
    let rust_primitive = dag.declaration_by_name("RustPrimitive")?;
    let TypeConnective::Disj { variants } = &rust_primitive.connective else {
        return None;
    };
    variants
        .iter()
        .find(|v| v.label == "IntegerPrimitive")
        .map(|v| v.ty)
}

fn pilot_integer_row(
    witness: IntegerRoutingWitness,
    payload: &[FieldValue],
    default_span: SourceSpan,
) -> Result<Option<PilotIntegerMatch>, Diagnostic> {
    // IntegerPrimitive field order: target_name, algebra, carrier, range_*, is_copy, overflow
    if payload.len() < 5 {
        return Ok(None);
    }
    let FieldValue::Variant {
        constructor: algebra_ctor,
        ..
    } = &payload[1]
    else {
        return Err(malformed_integer_range_fact(
            "rust_pilot_primitives IntegerPrimitive `algebra` must be a variant value".to_string(),
            default_span.clone(),
        ));
    };
    let FieldValue::Variant {
        constructor: carrier_ctor,
        ..
    } = &payload[2]
    else {
        return Err(malformed_integer_range_fact(
            "rust_pilot_primitives IntegerPrimitive `carrier` must be a variant value".to_string(),
            default_span.clone(),
        ));
    };
    if *algebra_ctor != witness.algebra_variant_ty || *carrier_ctor != witness.carrier_variant_ty {
        return Ok(None);
    }

    let range = (|| {
        let min_decimal = literal_string(payload.get(3)?)?;
        let max_decimal = literal_string(payload.get(4)?)?;
        let min = min_decimal.parse().ok()?;
        let max = max_decimal.parse().ok()?;
        if min > max {
            return None;
        }
        Some(IntegerRange {
            target_name: literal_string(payload.first()?)?,
            min_decimal,
            max_decimal,
            min,
            max,
        })
    })();

    Ok(Some(PilotIntegerMatch {
        range,
        span: default_span,
    }))
}

fn literal_string(value: &FieldValue) -> Option<String> {
    match value {
        FieldValue::Literal(LiteralBits::String(value)) => Some(value.clone()),
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
) -> Result<Option<bool>, Diagnostic> {
    match integer_range_for_decl(dag, expected) {
        IntegerRangeLookup::Found(range) => Ok(Some(range.contains_i64(literal))),
        IntegerRangeLookup::Missing => Ok(None),
        IntegerRangeLookup::Invalid(diag) => Err(diag),
    }
}

pub(crate) fn magnitude_out_of_range(
    literal: i64,
    expected: TypeShape,
    range: IntegerRange,
    span: SourceSpan,
) -> Diagnostic {
    Diagnostic::MagnitudeOutOfRange {
        literal: literal.to_string(),
        target: range.target_name.to_string(),
        range_min_inclusive: range.min_decimal.to_string(),
        range_max_inclusive: range.max_decimal.to_string(),
        expected,
        span,
        fixes: Vec::new(),
    }
}

fn malformed_integer_range_fact(message: String, span: SourceSpan) -> Diagnostic {
    Diagnostic::MalformedIntegerRangeFact {
        message,
        span,
        fixes: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uint8_witness_matches_u8_pilot_row_constructors() {
        let dag = Dag::new();
        assert!(
            dag.diagnostics().is_empty(),
            "bootstrap diagnostics: {:?}",
            dag.diagnostics()
        );
        let uint8 = dag
            .declaration_by_name("UInt8")
            .expect("UInt8 in bootstrap")
            .id;
        let witness = integer_routing_witness_for_decl(&dag, uint8).expect("UInt8 witness");
        let pilot = dag.rust_pilot_primitives().expect("pilot");
        let ValueBody::List(elements) = pilot.value_body.as_ref().expect("list body") else {
            panic!("expected list");
        };
        let integer_primitive_ctor = rust_primitive_integer_variant_ty(&dag).expect("ctor");
        let mut matched = 0usize;
        for element in elements {
            let FieldValue::Variant {
                constructor,
                payload,
            } = element
            else {
                continue;
            };
            if *constructor != integer_primitive_ctor {
                continue;
            }
            let FieldValue::Variant { constructor: a, .. } = &payload[1] else {
                continue;
            };
            let FieldValue::Variant { constructor: c, .. } = &payload[2] else {
                continue;
            };
            if *a == witness.algebra_variant_ty && *c == witness.carrier_variant_ty {
                matched += 1;
            }
        }
        assert_eq!(
            matched, 1,
            "exactly one pilot IntegerPrimitive row for UInt8 witness"
        );
    }
}
