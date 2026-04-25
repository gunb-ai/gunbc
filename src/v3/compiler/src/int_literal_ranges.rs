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

pub(crate) fn integer_range_for_decl(dag: &Dag, decl: DeclarationId) -> Option<IntegerRange> {
    let key = integer_routing_key_for_decl(dag, decl, 0)?;
    dag.declarations()
        .iter()
        .filter(|decl| is_integer_range_fact(dag, decl.meta_tag))
        .filter_map(|decl| integer_range_fact(dag, decl.value_body.as_ref()?))
        .find(|fact| fact.key == key)
        .map(|fact| fact.range)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IntegerRoutingKey {
    algebra: String,
    carrier: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IntegerRangeFact {
    key: IntegerRoutingKey,
    range: IntegerRange,
}

fn integer_routing_key_for_decl(
    dag: &Dag,
    decl: DeclarationId,
    depth: usize,
) -> Option<IntegerRoutingKey> {
    if depth >= 32 {
        // Alias/connective chains this deep are outside the current
        // reconciliation contract. Treat the range lookup as unavailable
        // and let callers fall through to the existing type diagnostics.
        return None;
    }
    let declaration = dag.declaration(decl);
    match &declaration.connective {
        TypeConnective::Atom(AtomPayload::ResolvedByName(next))
        | TypeConnective::Atom(AtomPayload::ResolvedByStructure(next)) => {
            integer_routing_key_for_decl(dag, *next, depth + 1)
        }
        TypeConnective::Instantiation {
            template,
            arguments,
        } => {
            if arguments.is_empty() {
                return integer_routing_key_for_decl(dag, *template, depth + 1);
            }
            let template_name = dag.declaration(*template).name.as_deref()?;
            let algebra = match template_name {
                "OrderedRing" => "OrderedRingAlgebra",
                "Semiring" => "SemiringAlgebra",
                _ => return None,
            };
            let carrier = arguments
                .first()
                .and_then(|arg| dag.declaration(arg.value).name.as_deref())
                .and_then(carrier_tag_for_std_type)?;
            Some(IntegerRoutingKey {
                algebra: algebra.to_string(),
                carrier: carrier.to_string(),
            })
        }
        _ => None,
    }
}

fn carrier_tag_for_std_type(name: &str) -> Option<&'static str> {
    match name {
        "Byte" => Some("ByteCarrier"),
        "Word16" => Some("Word16Carrier"),
        "Word32" => Some("Word32Carrier"),
        "Word64" => Some("Word64Carrier"),
        _ => None,
    }
}

fn is_integer_range_fact(dag: &Dag, meta_tag: Option<DeclarationId>) -> bool {
    let Some(meta_tag) = meta_tag else {
        return false;
    };
    dag.declaration(meta_tag).name.as_deref() == Some("IntegerRangeFact")
}

fn integer_range_fact(dag: &Dag, value_body: &ValueBody) -> Option<IntegerRangeFact> {
    let ValueBody::Structural { fields } = value_body else {
        return None;
    };
    let min_decimal = literal_string(require_field(fields, "range_min_inclusive")?)?;
    let max_decimal = literal_string(require_field(fields, "range_max_inclusive")?)?;
    let min = min_decimal.parse().ok()?;
    let max = max_decimal.parse().ok()?;
    if min > max {
        return None;
    }

    Some(IntegerRangeFact {
        key: IntegerRoutingKey {
            algebra: variant_label_for_value(dag, require_field(fields, "algebra")?)?,
            carrier: variant_label_for_value(dag, require_field(fields, "carrier")?)?,
        },
        range: IntegerRange {
            target_name: literal_string(require_field(fields, "target_name")?)?,
            min_decimal,
            max_decimal,
            min,
            max,
        },
    })
}

fn require_field<'a>(fields: &'a [(String, FieldValue)], name: &str) -> Option<&'a FieldValue> {
    fields
        .iter()
        .find(|(field_name, _)| field_name == name)
        .map(|(_, value)| value)
}

fn literal_string(value: &FieldValue) -> Option<String> {
    match value {
        FieldValue::Literal(LiteralBits::String(value)) => Some(value.clone()),
        _ => None,
    }
}

fn variant_label_for_value(dag: &Dag, value: &FieldValue) -> Option<String> {
    let FieldValue::Variant { constructor, .. } = value else {
        return None;
    };
    dag.declarations()
        .iter()
        .find_map(|decl| match &decl.connective {
            TypeConnective::Disj { variants } => variants
                .iter()
                .find(|variant| variant.ty == *constructor)
                .map(|variant| variant.label.clone()),
            _ => None,
        })
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
