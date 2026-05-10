//! T-Ground-Engine: Rust target primitive validation against the bootstrap Dag.
//!
//! **Phase 1** walks the loaded `Declaration` graph for `RustPrimitive`
//! type structure (sum partition, field shapes, nested tag enums).
//!
//! **Phase 2 (sharpened-(b), enumeration slice)** walks
//! `rust_pilot_primitives.value_body` as [`ValueBody::List`] and checks that
//! the first pilot row matches the authority ordering in
//! `dsl/extdeps/languages/rust/primitives.dag` (asserted here via the pilot
//! crate's `RUST_PILOT_PRIMITIVES` mirror until mirror retirement completes).

use std::collections::{BTreeMap, BTreeSet};

use v3_compiler::dag::{
    Dag, Declaration, DeclarationId, Field, FieldValue, LiteralBits, TypeConnective, ValueBody,
};
use v3_grounding_pilot::{
    IntegerAlgebra as PilotIntegerAlgebra, IntegerOverflow as PilotIntegerOverflow,
    NonIntegerAlgebra as PilotNonIntegerAlgebra, RustPrimitive as PilotRustPrimitive,
    TargetCarrier as PilotTargetCarrier, RUST_PILOT_PRIMITIVES,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructureMismatch {
    pub location: String,
    pub expected: String,
    pub actual: String,
}

pub type StructureResult<T> = Result<T, StructureMismatch>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumShape {
    pub name: String,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantShape {
    pub name: String,
    pub fields: Vec<FieldShape>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldShape {
    pub name: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RustPrimitiveTypeShape {
    pub variants: Vec<VariantShape>,
    pub integer_algebra: EnumShape,
    pub non_integer_algebra: EnumShape,
    pub target_carrier: EnumShape,
    pub integer_overflow: EnumShape,
}

pub fn validate_loaded_rust_primitive_type_structure() -> StructureResult<RustPrimitiveTypeShape> {
    let dag = Dag::new();
    let root = dag
        .rust_pilot_primitives()
        .ok_or_else(|| StructureMismatch {
            location: "Dag::rust_pilot_primitives".to_string(),
            expected: "loaded rust_pilot_primitives declaration".to_string(),
            actual: "missing".to_string(),
        })?;
    validate_rust_primitive_type_structure(&dag, root)
}

pub fn validate_rust_primitive_type_structure(
    dag: &Dag,
    pilot_list_decl: &Declaration,
) -> StructureResult<RustPrimitiveTypeShape> {
    let rust_primitive_id = rust_primitive_element_id(pilot_list_decl)?;
    let rust_primitive = dag.declaration(rust_primitive_id);
    expect_named_decl(
        rust_primitive,
        "RustPrimitive",
        "rust_pilot_primitives element",
    )?;

    let variant_fields = expect_disj_variants(
        rust_primitive,
        "RustPrimitive",
        &["IntegerPrimitive", "NonIntegerPrimitive"],
    )?;

    let integer_variant = variant_decl(dag, &variant_fields, "IntegerPrimitive")?;
    let non_integer_variant = variant_decl(dag, &variant_fields, "NonIntegerPrimitive")?;

    let integer = expect_variant_shape(
        dag,
        "RustPrimitive.IntegerPrimitive",
        integer_variant,
        &[
            ("target_name", "String"),
            ("algebra", "IntegerAlgebra"),
            ("carrier", "TargetCarrier"),
            ("range_min_inclusive", "String"),
            ("range_max_inclusive", "String"),
            ("is_copy", "Bool"),
            ("overflow", "IntegerOverflow"),
        ],
    )?;
    let non_integer = expect_variant_shape(
        dag,
        "RustPrimitive.NonIntegerPrimitive",
        non_integer_variant,
        &[
            ("target_name", "String"),
            ("algebra", "NonIntegerAlgebra"),
            ("carrier", "TargetCarrier"),
            ("is_copy", "Bool"),
        ],
    )?;

    require_field_absent(&non_integer, "overflow")?;

    let integer_algebra = expect_enum_shape(
        dag,
        "IntegerAlgebra",
        &["OrderedRingAlgebra", "SemiringAlgebra"],
    )?;
    let non_integer_algebra = expect_enum_shape(
        dag,
        "NonIntegerAlgebra",
        &[
            "ApproximateFieldAlgebra",
            "BooleanAlgebraAlgebra",
            "TerminalAlgebra",
        ],
    )?;
    let target_carrier = expect_enum_shape(
        dag,
        "TargetCarrier",
        &[
            "BitCarrier",
            "ByteCarrier",
            "Word16Carrier",
            "Word32Carrier",
            "Word64Carrier",
            "Word128Carrier",
            "TerminalCarrier",
        ],
    )?;
    let integer_overflow = expect_enum_shape(
        dag,
        "IntegerOverflow",
        &["TwoComplementWrap", "Saturating", "Trap"],
    )?;

    Ok(RustPrimitiveTypeShape {
        variants: vec![integer, non_integer],
        integer_algebra,
        non_integer_algebra,
        target_carrier,
        integer_overflow,
    })
}

pub fn validate_mirror_consistency() -> StructureResult<()> {
    let shape = validate_loaded_rust_primitive_type_structure()?;
    if shape != expected_mirror_shape() {
        return Err(StructureMismatch {
            location: "RUST_PILOT_PRIMITIVES implicit shape".to_string(),
            expected: format!("{:?}", expected_mirror_shape()),
            actual: format!("{shape:?}"),
        });
    }
    validate_pilot_values_with_shape(&shape)?;
    validate_first_rust_pilot_row_matches_mirror()
}

/// Phase 2: the first lowered `rust_pilot_primitives` list element must match
/// `RUST_PILOT_PRIMITIVES[0]` (authority ordering in `primitives.dag`).
pub fn validate_first_rust_pilot_row_matches_mirror() -> StructureResult<()> {
    let dag = Dag::new();
    let pilot_list = dag
        .rust_pilot_primitives()
        .ok_or_else(|| StructureMismatch {
            location: "Dag::rust_pilot_primitives".to_string(),
            expected: "loaded rust_pilot_primitives declaration".to_string(),
            actual: "missing".to_string(),
        })?;

    let rust_primitive_id = rust_primitive_element_id(pilot_list)?;
    let rust_primitive = dag.declaration(rust_primitive_id);
    let disj_variants = expect_disj_variants(
        rust_primitive,
        "RustPrimitive",
        &["IntegerPrimitive", "NonIntegerPrimitive"],
    )?;

    let body = pilot_list
        .value_body
        .as_ref()
        .ok_or_else(|| StructureMismatch {
            location: "rust_pilot_primitives.value_body".to_string(),
            expected: "data body present".to_string(),
            actual: "None".to_string(),
        })?;
    let ValueBody::List(elements) = body else {
        return Err(StructureMismatch {
            location: "rust_pilot_primitives.value_body".to_string(),
            expected: "ValueBody::List".to_string(),
            actual: value_body_kind(body),
        });
    };
    let first = elements.first().ok_or_else(|| StructureMismatch {
        location: "rust_pilot_primitives.value_body".to_string(),
        expected: "non-empty pilot list".to_string(),
        actual: "empty list".to_string(),
    })?;
    let FieldValue::Variant {
        constructor,
        payload,
    } = first
    else {
        return Err(StructureMismatch {
            location: "rust_pilot_primitives[0]".to_string(),
            expected: "FieldValue::Variant".to_string(),
            actual: field_value_kind(first),
        });
    };

    let variant_label = rust_primitive_variant_label(&disj_variants, *constructor)?;
    let mirror0 = RUST_PILOT_PRIMITIVES
        .first()
        .ok_or_else(|| StructureMismatch {
            location: "RUST_PILOT_PRIMITIVES".to_string(),
            expected: "non-empty mirror slice".to_string(),
            actual: "empty".to_string(),
        })?;

    let label = variant_label.as_str();
    match mirror0 {
        PilotRustPrimitive::IntegerPrimitive { .. } => {
            if label != "IntegerPrimitive" {
                return Err(StructureMismatch {
                    location: "rust_pilot_primitives[0]".to_string(),
                    expected: "IntegerPrimitive (first authority row)".to_string(),
                    actual: label.to_string(),
                });
            }
            assert_integer_primitive_payload_matches(&dag, payload, mirror0)
        }
        PilotRustPrimitive::NonIntegerPrimitive { .. } => {
            if label != "NonIntegerPrimitive" {
                return Err(StructureMismatch {
                    location: "rust_pilot_primitives[0]".to_string(),
                    expected: "NonIntegerPrimitive (first authority row)".to_string(),
                    actual: label.to_string(),
                });
            }
            assert_non_integer_primitive_payload_matches(&dag, payload, mirror0)
        }
    }
}

pub fn require_field_absent(variant: &VariantShape, field: &str) -> StructureResult<()> {
    if variant
        .fields
        .iter()
        .any(|candidate| candidate.name == field)
    {
        return Err(StructureMismatch {
            location: variant.name.clone(),
            expected: format!("no field `{field}`"),
            actual: format!("field `{field}` present"),
        });
    }
    Ok(())
}

fn rust_primitive_element_id(decl: &Declaration) -> StructureResult<DeclarationId> {
    match &decl.connective {
        TypeConnective::Instantiation { arguments, .. } if arguments.len() == 1 => {
            Ok(arguments[0].value)
        }
        TypeConnective::Instantiation { arguments, .. } => Err(StructureMismatch {
            location: "rust_pilot_primitives".to_string(),
            expected: "List<RustPrimitive> with one template argument".to_string(),
            actual: format!("Instantiation with {} template arguments", arguments.len()),
        }),
        other => Err(StructureMismatch {
            location: "rust_pilot_primitives".to_string(),
            expected: "List<RustPrimitive> instantiation".to_string(),
            actual: connective_name(other).to_string(),
        }),
    }
}

fn expect_named_decl(decl: &Declaration, expected: &str, location: &str) -> StructureResult<()> {
    if decl.name.as_deref() == Some(expected) {
        return Ok(());
    }
    Err(StructureMismatch {
        location: location.to_string(),
        expected: expected.to_string(),
        actual: decl.name.as_deref().unwrap_or("<anonymous>").to_string(),
    })
}

fn expect_disj_variants(
    decl: &Declaration,
    location: &str,
    expected: &[&str],
) -> StructureResult<Vec<Field>> {
    let TypeConnective::Disj { variants } = &decl.connective else {
        return Err(StructureMismatch {
            location: location.to_string(),
            expected: "Disj".to_string(),
            actual: connective_name(&decl.connective).to_string(),
        });
    };
    let actual: Vec<&str> = variants.iter().map(|field| field.label.as_str()).collect();
    if actual == expected {
        Ok(variants.clone())
    } else {
        Err(StructureMismatch {
            location: location.to_string(),
            expected: format!("{expected:?}"),
            actual: format!("{actual:?}"),
        })
    }
}

fn variant_decl<'a>(
    dag: &'a Dag,
    variants: &[Field],
    label: &str,
) -> StructureResult<&'a Declaration> {
    variants
        .iter()
        .find(|field| field.label == label)
        .map(|field| dag.declaration(field.ty))
        .ok_or_else(|| StructureMismatch {
            location: "RustPrimitive".to_string(),
            expected: format!("variant `{label}`"),
            actual: "missing".to_string(),
        })
}

fn expect_variant_shape(
    dag: &Dag,
    location: &str,
    decl: &Declaration,
    expected: &[(&str, &str)],
) -> StructureResult<VariantShape> {
    let TypeConnective::Conj { children } = &decl.connective else {
        return Err(StructureMismatch {
            location: location.to_string(),
            expected: "Conj variant payload".to_string(),
            actual: connective_name(&decl.connective).to_string(),
        });
    };

    let mut fields = Vec::with_capacity(children.len());
    for child in children {
        let ty = type_name(dag, child.ty);
        fields.push(FieldShape {
            name: child.label.clone(),
            ty,
        });
    }

    let actual: Vec<(&str, &str)> = fields
        .iter()
        .map(|field| (field.name.as_str(), field.ty.as_str()))
        .collect();
    if actual == expected {
        Ok(VariantShape {
            name: location.to_string(),
            fields,
        })
    } else {
        Err(StructureMismatch {
            location: location.to_string(),
            expected: format!("{expected:?}"),
            actual: format!("{actual:?}"),
        })
    }
}

fn expect_enum_shape(dag: &Dag, name: &str, expected: &[&str]) -> StructureResult<EnumShape> {
    let decl = dag
        .declaration_by_name(name)
        .ok_or_else(|| StructureMismatch {
            location: name.to_string(),
            expected: "declared enum".to_string(),
            actual: "missing".to_string(),
        })?;
    expect_named_decl(decl, name, name)?;
    let variants = expect_disj_variants(decl, name, expected)?
        .into_iter()
        .map(|field| field.label)
        .collect();
    Ok(EnumShape {
        name: name.to_string(),
        variants,
    })
}

fn type_name(dag: &Dag, id: DeclarationId) -> String {
    dag.declaration(id)
        .name
        .clone()
        .unwrap_or_else(|| format!("<anonymous:{}>", id.raw()))
}

fn connective_name(connective: &TypeConnective) -> &'static str {
    match connective {
        TypeConnective::Atom(_) => "Atom",
        TypeConnective::Conj { .. } => "Conj",
        TypeConnective::Disj { .. } => "Disj",
        TypeConnective::Arrow { .. } => "Arrow",
        TypeConnective::Cardinality { .. } => "Cardinality",
        TypeConnective::Instantiation { .. } => "Instantiation",
    }
}

fn expected_mirror_shape() -> RustPrimitiveTypeShape {
    RustPrimitiveTypeShape {
        variants: vec![
            VariantShape {
                name: "RustPrimitive.IntegerPrimitive".to_string(),
                fields: vec![
                    field("target_name", "String"),
                    field("algebra", "IntegerAlgebra"),
                    field("carrier", "TargetCarrier"),
                    field("range_min_inclusive", "String"),
                    field("range_max_inclusive", "String"),
                    field("is_copy", "Bool"),
                    field("overflow", "IntegerOverflow"),
                ],
            },
            VariantShape {
                name: "RustPrimitive.NonIntegerPrimitive".to_string(),
                fields: vec![
                    field("target_name", "String"),
                    field("algebra", "NonIntegerAlgebra"),
                    field("carrier", "TargetCarrier"),
                    field("is_copy", "Bool"),
                ],
            },
        ],
        integer_algebra: enum_shape("IntegerAlgebra", &["OrderedRingAlgebra", "SemiringAlgebra"]),
        non_integer_algebra: enum_shape(
            "NonIntegerAlgebra",
            &[
                "ApproximateFieldAlgebra",
                "BooleanAlgebraAlgebra",
                "TerminalAlgebra",
            ],
        ),
        target_carrier: enum_shape(
            "TargetCarrier",
            &[
                "BitCarrier",
                "ByteCarrier",
                "Word16Carrier",
                "Word32Carrier",
                "Word64Carrier",
                "Word128Carrier",
                "TerminalCarrier",
            ],
        ),
        integer_overflow: enum_shape(
            "IntegerOverflow",
            &["TwoComplementWrap", "Saturating", "Trap"],
        ),
    }
}

fn field(name: &str, ty: &str) -> FieldShape {
    FieldShape {
        name: name.to_string(),
        ty: ty.to_string(),
    }
}

fn enum_shape(name: &str, variants: &[&str]) -> EnumShape {
    EnumShape {
        name: name.to_string(),
        variants: variants.iter().map(|variant| variant.to_string()).collect(),
    }
}

fn validate_pilot_values_with_shape(shape: &RustPrimitiveTypeShape) -> StructureResult<()> {
    let variant_names: BTreeSet<&str> = shape
        .variants
        .iter()
        .map(|variant| variant.name.as_str())
        .collect();
    let integer_algebras = variant_set(&shape.integer_algebra);
    let non_integer_algebras = variant_set(&shape.non_integer_algebra);
    let carriers = variant_set(&shape.target_carrier);
    let overflows = variant_set(&shape.integer_overflow);

    for primitive in RUST_PILOT_PRIMITIVES {
        match primitive {
            PilotRustPrimitive::IntegerPrimitive {
                algebra,
                carrier,
                overflow,
                ..
            } => {
                require_variant_known(&variant_names, "RustPrimitive.IntegerPrimitive")?;
                require_tag_known(
                    &integer_algebras,
                    pilot_integer_algebra_name(*algebra),
                    "IntegerAlgebra",
                )?;
                require_tag_known(&carriers, pilot_carrier_name(*carrier), "TargetCarrier")?;
                require_tag_known(
                    &overflows,
                    pilot_overflow_name(*overflow),
                    "IntegerOverflow",
                )?;
            }
            PilotRustPrimitive::NonIntegerPrimitive {
                algebra, carrier, ..
            } => {
                require_variant_known(&variant_names, "RustPrimitive.NonIntegerPrimitive")?;
                require_tag_known(
                    &non_integer_algebras,
                    pilot_non_integer_algebra_name(*algebra),
                    "NonIntegerAlgebra",
                )?;
                require_tag_known(&carriers, pilot_carrier_name(*carrier), "TargetCarrier")?;
            }
        }
    }
    Ok(())
}

fn variant_set(shape: &EnumShape) -> BTreeMap<&str, ()> {
    shape
        .variants
        .iter()
        .map(|variant| (variant.as_str(), ()))
        .collect()
}

fn require_variant_known(variants: &BTreeSet<&str>, variant: &str) -> StructureResult<()> {
    if variants.contains(variant) {
        Ok(())
    } else {
        Err(StructureMismatch {
            location: "RUST_PILOT_PRIMITIVES".to_string(),
            expected: format!("loaded shape contains `{variant}`"),
            actual: "missing".to_string(),
        })
    }
}

fn require_tag_known(tags: &BTreeMap<&str, ()>, tag: &str, enum_name: &str) -> StructureResult<()> {
    if tags.contains_key(tag) {
        Ok(())
    } else {
        Err(StructureMismatch {
            location: enum_name.to_string(),
            expected: format!("loaded shape contains `{tag}`"),
            actual: "missing".to_string(),
        })
    }
}

fn pilot_integer_algebra_name(algebra: PilotIntegerAlgebra) -> &'static str {
    match algebra {
        PilotIntegerAlgebra::OrderedRing => "OrderedRingAlgebra",
        PilotIntegerAlgebra::Semiring => "SemiringAlgebra",
    }
}

fn pilot_non_integer_algebra_name(algebra: PilotNonIntegerAlgebra) -> &'static str {
    match algebra {
        PilotNonIntegerAlgebra::ApproximateField => "ApproximateFieldAlgebra",
        PilotNonIntegerAlgebra::BooleanAlgebra => "BooleanAlgebraAlgebra",
        PilotNonIntegerAlgebra::Terminal => "TerminalAlgebra",
    }
}

fn pilot_carrier_name(carrier: PilotTargetCarrier) -> &'static str {
    match carrier {
        PilotTargetCarrier::Bit => "BitCarrier",
        PilotTargetCarrier::Byte => "ByteCarrier",
        PilotTargetCarrier::Word16 => "Word16Carrier",
        PilotTargetCarrier::Word32 => "Word32Carrier",
        PilotTargetCarrier::Word64 => "Word64Carrier",
        PilotTargetCarrier::Word128 => "Word128Carrier",
        PilotTargetCarrier::Terminal => "TerminalCarrier",
    }
}

fn pilot_overflow_name(overflow: PilotIntegerOverflow) -> &'static str {
    match overflow {
        PilotIntegerOverflow::TwoComplementWrap => "TwoComplementWrap",
        PilotIntegerOverflow::Saturating => "Saturating",
        PilotIntegerOverflow::Trap => "Trap",
    }
}

fn value_body_kind(body: &ValueBody) -> String {
    match body {
        ValueBody::Unparsed(_) => "ValueBody::Unparsed".to_string(),
        ValueBody::Structural { .. } => "ValueBody::Structural".to_string(),
        ValueBody::Scalar(_) => "ValueBody::Scalar".to_string(),
        ValueBody::List(_) => "ValueBody::List".to_string(),
        ValueBody::Map(_) => "ValueBody::Map".to_string(),
    }
}

fn field_value_kind(value: &FieldValue) -> String {
    match value {
        FieldValue::Literal(_) => "FieldValue::Literal".to_string(),
        FieldValue::Reference(_) => "FieldValue::Reference".to_string(),
        FieldValue::Record(_) => "FieldValue::Record".to_string(),
        FieldValue::List(_) => "FieldValue::List".to_string(),
        FieldValue::Map(_) => "FieldValue::Map".to_string(),
        FieldValue::Variant { .. } => "FieldValue::Variant".to_string(),
    }
}

fn rust_primitive_variant_label(
    disj_variants: &[Field],
    constructor: DeclarationId,
) -> StructureResult<String> {
    disj_variants
        .iter()
        .find(|variant| variant.ty == constructor)
        .map(|variant| variant.label.clone())
        .ok_or_else(|| StructureMismatch {
            location: "rust_pilot_primitives[0].constructor".to_string(),
            expected: "constructor id belonging to RustPrimitive".to_string(),
            actual: "unknown constructor".to_string(),
        })
}

fn expect_literal_string_at(
    payload: &[FieldValue],
    idx: usize,
    ctx: &str,
) -> StructureResult<String> {
    let value = payload.get(idx).ok_or_else(|| StructureMismatch {
        location: ctx.to_string(),
        expected: format!("payload field index {idx}"),
        actual: format!("len {}", payload.len()),
    })?;
    let FieldValue::Literal(LiteralBits::String(s)) = value else {
        return Err(StructureMismatch {
            location: format!("{ctx}[{idx}]"),
            expected: "Literal String".to_string(),
            actual: field_value_kind(value),
        });
    };
    Ok(s.clone())
}

fn expect_literal_bool_at(payload: &[FieldValue], idx: usize, ctx: &str) -> StructureResult<bool> {
    let value = payload.get(idx).ok_or_else(|| StructureMismatch {
        location: ctx.to_string(),
        expected: format!("payload field index {idx}"),
        actual: format!("len {}", payload.len()),
    })?;
    let FieldValue::Literal(LiteralBits::Bool(b)) = value else {
        return Err(StructureMismatch {
            location: format!("{ctx}[{idx}]"),
            expected: "Literal Bool".to_string(),
            actual: field_value_kind(value),
        });
    };
    Ok(*b)
}

fn expect_nullary_variant_label(
    dag: &Dag,
    enum_name: &str,
    expected_variants: &[&str],
    value: &FieldValue,
    ctx: &str,
) -> StructureResult<String> {
    let FieldValue::Variant {
        constructor,
        payload,
    } = value
    else {
        return Err(StructureMismatch {
            location: ctx.to_string(),
            expected: "FieldValue::Variant".to_string(),
            actual: field_value_kind(value),
        });
    };
    if !payload.is_empty() {
        return Err(StructureMismatch {
            location: ctx.to_string(),
            expected: "nullary enum constructor (empty payload)".to_string(),
            actual: format!("payload len {}", payload.len()),
        });
    }
    let enum_decl = dag
        .declaration_by_name(enum_name)
        .ok_or_else(|| StructureMismatch {
            location: ctx.to_string(),
            expected: format!("declared enum `{enum_name}`"),
            actual: "missing".to_string(),
        })?;
    let variants = expect_disj_variants(enum_decl, enum_name, expected_variants)?;
    variants
        .iter()
        .find(|variant| variant.ty == *constructor)
        .map(|variant| variant.label.clone())
        .ok_or_else(|| StructureMismatch {
            location: ctx.to_string(),
            expected: format!("constructor id belonging to {enum_name}"),
            actual: "unknown constructor".to_string(),
        })
}

fn require_string_eq(actual: &str, expected: &str, field: &str) -> StructureResult<()> {
    if actual == expected {
        Ok(())
    } else {
        Err(StructureMismatch {
            location: field.to_string(),
            expected: expected.to_string(),
            actual: actual.to_string(),
        })
    }
}

fn assert_integer_primitive_payload_matches(
    dag: &Dag,
    payload: &[FieldValue],
    pilot: &PilotRustPrimitive,
) -> StructureResult<()> {
    let PilotRustPrimitive::IntegerPrimitive {
        target_name,
        algebra,
        carrier,
        range_min_inclusive,
        range_max_inclusive,
        is_copy,
        overflow,
    } = pilot
    else {
        return Err(StructureMismatch {
            location: "validate_first_rust_pilot_row_matches_mirror".to_string(),
            expected: "IntegerPrimitive mirror row".to_string(),
            actual: "NonIntegerPrimitive".to_string(),
        });
    };
    let ctx = "RustPrimitive.IntegerPrimitive payload";
    if payload.len() != 7 {
        return Err(StructureMismatch {
            location: ctx.to_string(),
            expected: "7 positional fields".to_string(),
            actual: format!("{}", payload.len()),
        });
    }
    let tn = expect_literal_string_at(payload, 0, ctx)?;
    require_string_eq(&tn, target_name, "target_name")?;
    let alg = expect_nullary_variant_label(
        dag,
        "IntegerAlgebra",
        &["OrderedRingAlgebra", "SemiringAlgebra"],
        &payload[1],
        &format!("{ctx}.algebra"),
    )?;
    require_string_eq(&alg, pilot_integer_algebra_name(*algebra), "algebra")?;
    let car = expect_nullary_variant_label(
        dag,
        "TargetCarrier",
        &[
            "BitCarrier",
            "ByteCarrier",
            "Word16Carrier",
            "Word32Carrier",
            "Word64Carrier",
            "Word128Carrier",
            "TerminalCarrier",
        ],
        &payload[2],
        &format!("{ctx}.carrier"),
    )?;
    require_string_eq(&car, pilot_carrier_name(*carrier), "carrier")?;
    let rmin = expect_literal_string_at(payload, 3, ctx)?;
    require_string_eq(&rmin, range_min_inclusive, "range_min_inclusive")?;
    let rmax = expect_literal_string_at(payload, 4, ctx)?;
    require_string_eq(&rmax, range_max_inclusive, "range_max_inclusive")?;
    let ic = expect_literal_bool_at(payload, 5, ctx)?;
    if ic != *is_copy {
        return Err(StructureMismatch {
            location: format!("{ctx}.is_copy"),
            expected: format!("{is_copy}"),
            actual: format!("{ic}"),
        });
    }
    let ov = expect_nullary_variant_label(
        dag,
        "IntegerOverflow",
        &["TwoComplementWrap", "Saturating", "Trap"],
        &payload[6],
        &format!("{ctx}.overflow"),
    )?;
    require_string_eq(&ov, pilot_overflow_name(*overflow), "overflow")?;
    Ok(())
}

fn assert_non_integer_primitive_payload_matches(
    dag: &Dag,
    payload: &[FieldValue],
    pilot: &PilotRustPrimitive,
) -> StructureResult<()> {
    let PilotRustPrimitive::NonIntegerPrimitive {
        target_name,
        algebra,
        carrier,
        is_copy,
    } = pilot
    else {
        return Err(StructureMismatch {
            location: "validate_first_rust_pilot_row_matches_mirror".to_string(),
            expected: "NonIntegerPrimitive mirror row".to_string(),
            actual: "IntegerPrimitive".to_string(),
        });
    };
    let ctx = "RustPrimitive.NonIntegerPrimitive payload";
    if payload.len() != 4 {
        return Err(StructureMismatch {
            location: ctx.to_string(),
            expected: "4 positional fields".to_string(),
            actual: format!("{}", payload.len()),
        });
    }
    let tn = expect_literal_string_at(payload, 0, ctx)?;
    require_string_eq(&tn, target_name, "target_name")?;
    let alg = expect_nullary_variant_label(
        dag,
        "NonIntegerAlgebra",
        &[
            "ApproximateFieldAlgebra",
            "BooleanAlgebraAlgebra",
            "TerminalAlgebra",
        ],
        &payload[1],
        &format!("{ctx}.algebra"),
    )?;
    require_string_eq(&alg, pilot_non_integer_algebra_name(*algebra), "algebra")?;
    let car = expect_nullary_variant_label(
        dag,
        "TargetCarrier",
        &[
            "BitCarrier",
            "ByteCarrier",
            "Word16Carrier",
            "Word32Carrier",
            "Word64Carrier",
            "Word128Carrier",
            "TerminalCarrier",
        ],
        &payload[2],
        &format!("{ctx}.carrier"),
    )?;
    require_string_eq(&car, pilot_carrier_name(*carrier), "carrier")?;
    let ic = expect_literal_bool_at(payload, 3, ctx)?;
    if ic != *is_copy {
        return Err(StructureMismatch {
            location: format!("{ctx}.is_copy"),
            expected: format!("{is_copy}"),
            actual: format!("{ic}"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_structure_parity_matches_rust_primitive_authority() {
        let shape = validate_loaded_rust_primitive_type_structure().expect("shape validates");
        assert_eq!(shape, expected_mirror_shape());
    }

    #[test]
    fn mirror_consistency_links_loader_shape_to_pilot_constants() {
        validate_mirror_consistency().expect("pilot mirror matches loaded type shape");
    }

    #[test]
    fn first_enumerated_pilot_row_matches_mirror_i8() {
        validate_first_rust_pilot_row_matches_mirror()
            .expect("first lowered list row matches RUST_PILOT_PRIMITIVES[0] (i8)");
    }

    #[test]
    fn state_space_keeps_overflow_off_non_integer_variant() {
        let shape = validate_loaded_rust_primitive_type_structure().expect("shape validates");
        let non_integer = shape
            .variants
            .iter()
            .find(|variant| variant.name == "RustPrimitive.NonIntegerPrimitive")
            .expect("non-integer variant present");
        require_field_absent(non_integer, "overflow").expect("overflow is absent");
    }

    #[test]
    fn state_space_reports_mismatch_when_overflow_is_required_on_non_integer_variant() {
        let shape = validate_loaded_rust_primitive_type_structure().expect("shape validates");
        let non_integer = shape
            .variants
            .iter()
            .find(|variant| variant.name == "RustPrimitive.NonIntegerPrimitive")
            .expect("non-integer variant present");
        let actual_fields: Vec<&str> = non_integer
            .fields
            .iter()
            .map(|field| field.name.as_str())
            .collect();
        let err = StructureMismatch {
            location: non_integer.name.clone(),
            expected: "field `overflow`".to_string(),
            actual: format!("{actual_fields:?}"),
        };
        assert_eq!(err.location, "RustPrimitive.NonIntegerPrimitive");
        assert!(err.expected.contains("overflow"));
        assert!(err.actual.contains("target_name"));
    }

    #[test]
    fn diagnostic_quality_names_location_expected_and_actual_shape() {
        let bad_variant = VariantShape {
            name: "RustPrimitive.NonIntegerPrimitive".to_string(),
            fields: vec![field("overflow", "IntegerOverflow")],
        };
        let err = require_field_absent(&bad_variant, "overflow").expect_err("must fail closed");
        assert_eq!(err.location, "RustPrimitive.NonIntegerPrimitive");
        assert_eq!(err.expected, "no field `overflow`");
        assert_eq!(err.actual, "field `overflow` present");
    }
}
