//! **Layer:** integration

use std::collections::HashSet;

use v3_compiler::dag::{FieldValue, LiteralBits, TypeConnective, ValueBody};
use v3_compiler::{compile_to_dag, integer_literal_routing_witness, CompileError};

#[test]
fn int_literals_fit_declared_integer_ranges() {
    let source = r#"
data i8_max: Int8 = 127
data i16_max: Int16 = 32767
data i32_max: Int32 = 2147483647
data i64_max: Int64 = 9223372036854775807
type ByteAlias = UInt8
data alias_u8_max: ByteAlias = 255
data u8_max: UInt8 = 255
data u16_max: UInt16 = 65535
data u32_max: UInt32 = 4294967295
"#;

    let dag = compile_to_dag(source, "int_literal_ranges.v3").expect("range literals compile");
    for name in [
        "i8_max",
        "i16_max",
        "i32_max",
        "i64_max",
        "alias_u8_max",
        "u8_max",
        "u16_max",
        "u32_max",
    ] {
        assert!(
            matches!(
                dag.declaration_by_name(name)
                    .and_then(|decl| decl.value_body.as_ref()),
                Some(v3_compiler::dag::ValueBody::Scalar(LiteralBits::Int(_)))
            ),
            "{name} should lower as a scalar int literal"
        );
    }
}

#[test]
fn unconstrained_int_literal_still_defaults_to_int64() {
    let dag = compile_to_dag("let x = 5", "int_literal_default.v3").expect("compiles");
    let value = dag
        .nodes()
        .iter()
        .find_map(|node| match node {
            v3_compiler::dag::Behavior::Value(value) if value.data == LiteralBits::Int(5) => {
                Some(value)
            }
            _ => None,
        })
        .expect("literal value node exists");
    let ty = match dag.port(value.output).state() {
        v3_compiler::dag::PortState::Resolved(ty) => ty,
        other => panic!("literal type should resolve, got {other:?}"),
    };
    assert_eq!(
        dag.declaration(ty.declaration).name.as_deref(),
        Some("Int"),
        "unconstrained literals keep the explicit Int64 default alias"
    );
}

#[test]
fn let_annotated_uint8_literal_resolves_to_narrow_type() {
    let dag = compile_to_dag("let x: UInt8 = 5\n", "let_u8_narrow.v3").expect("compiles");
    let value = dag
        .nodes()
        .iter()
        .find_map(|node| match node {
            v3_compiler::dag::Behavior::Value(v) if v.data == LiteralBits::Int(5) => Some(v),
            _ => None,
        })
        .expect("literal");
    let ty = match dag.port(value.output).state() {
        v3_compiler::dag::PortState::Resolved(ty) => ty,
        other => panic!("expected resolved port, got {other:?}"),
    };
    assert_eq!(
        dag.declaration(ty.declaration).name.as_deref(),
        Some("UInt8"),
        "annotated u8-typed `let` should keep range-backed narrow type at the literal port"
    );
}

#[test]
fn let_annotated_uint8_out_of_range_emits_magnitude_diagnostic() {
    let err = compile_to_dag("let x: UInt8 = 256\n", "let_u8_oob.v3")
        .expect_err("annotated let UInt8 overflow must fail closed");
    let CompileError::Semantic(dag) = err else {
        panic!("expected semantic diagnostic, got {err:?}");
    };
    assert!(
        dag.diagnostics().iter().any(|(_, diagnostic)| {
            matches!(
                diagnostic,
                v3_compiler::diagnostics::Diagnostic::MagnitudeOutOfRange {
                    literal,
                    target,
                    range_min_inclusive,
                    range_max_inclusive,
                    fixes,
                    ..
                } if literal == "256"
                    && target == "u8"
                    && range_min_inclusive == "0"
                    && range_max_inclusive == "255"
                    && fixes.is_empty()
            )
        }),
        "expected MagnitudeOutOfRange for let literal, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn call_site_uint8_literal_narrows() {
    let source = "fn id_u8(p: UInt8) -> UInt8 = p\nlet y: UInt8 = id_u8(7)\n";
    let dag = compile_to_dag(source, "call_u8_narrow.v3").expect("compiles");
    let value = dag
        .nodes()
        .iter()
        .find_map(|node| match node {
            v3_compiler::dag::Behavior::Value(v) if v.data == LiteralBits::Int(7) => Some(v),
            _ => None,
        })
        .expect("call literal 7");
    let ty = match dag.port(value.output).state() {
        v3_compiler::dag::PortState::Resolved(ty) => ty,
        other => panic!("expected resolved port, got {other:?}"),
    };
    assert_eq!(
        dag.declaration(ty.declaration).name.as_deref(),
        Some("UInt8")
    );
}

#[test]
fn emit_let_uint8_uses_narrow_rust_type() {
    use v3_compiler::emit_rust::emit_rust;
    let dag = compile_to_dag("let x: UInt8 = 5\n", "emit_let_u8.v3").expect("compiles");
    let out = emit_rust(&dag).expect("emits");
    assert!(
        out.contains("u8") && out.contains("x") && out.contains("5"),
        "expected u8-annotated let in Rust text, got: {out}"
    );
}

#[test]
fn uint64_upper_half_literals_are_tracked_carrier_limitation() {
    let err = compile_to_dag(
        "data x: UInt64 = 9223372036854775808",
        "uint64_upper_half_literal.v3",
    )
    .expect_err("u64 upper-half literals remain blocked by the i64 source literal carrier");
    assert!(
        matches!(
            err,
            CompileError::Tokenize(v3_compiler::diagnostics::Diagnostic::TokenizerError { .. })
        ),
        "expected tokenizer boundary before range reconciliation, got {err:?}"
    );
}

#[test]
fn out_of_range_uint8_literal_emits_magnitude_diagnostic() {
    let err = compile_to_dag("data x: UInt8 = 256", "int_literal_u8_oob.v3")
        .expect_err("UInt8 overflow must fail closed");
    let CompileError::Semantic(dag) = err else {
        panic!("expected semantic diagnostic, got {err:?}");
    };
    let messages: Vec<String> = dag
        .diagnostics()
        .iter()
        .map(|(_, diagnostic)| diagnostic.message())
        .collect();
    assert_eq!(
        messages.len(),
        1,
        "out-of-range integer literal should emit one root-cause diagnostic, got {messages:?}"
    );
    assert!(
        messages.iter().any(|message| {
            message.contains("integer literal `256`")
                && message.contains("u8")
                && message.contains("0..=255")
                && message.contains("wider target")
        }),
        "expected MagnitudeOutOfRange details, got {messages:?}"
    );
    assert!(
        dag.diagnostics().iter().any(|(_, diagnostic)| {
            matches!(
                diagnostic,
                v3_compiler::diagnostics::Diagnostic::MagnitudeOutOfRange {
                    literal,
                    target,
                    range_min_inclusive,
                    range_max_inclusive,
                    fixes,
                    ..
                } if literal == "256"
                    && target == "u8"
                    && range_min_inclusive == "0"
                    && range_max_inclusive == "255"
                    && fixes.is_empty()
            )
        }),
        "MagnitudeOutOfRange should carry typed bounds and no fabricated correction"
    );
}

#[test]
fn int_literal_ranges_follow_type_aliases() {
    let err = compile_to_dag(
        "type ByteAlias = UInt8\ndata x: ByteAlias = 256",
        "int_literal_alias_u8_oob.v3",
    )
    .expect_err("UInt8 alias overflow must fail closed through the alias chain");
    let CompileError::Semantic(dag) = err else {
        panic!("expected semantic diagnostic, got {err:?}");
    };
    let messages: Vec<String> = dag
        .diagnostics()
        .iter()
        .map(|(_, diagnostic)| diagnostic.message())
        .collect();
    assert_eq!(
        messages.len(),
        1,
        "aliased out-of-range integer literal should emit one root-cause diagnostic, got {messages:?}"
    );
    assert!(
        dag.diagnostics().iter().any(|(_, diagnostic)| {
            matches!(
                diagnostic,
                v3_compiler::diagnostics::Diagnostic::MagnitudeOutOfRange {
                    literal,
                    target,
                    range_min_inclusive,
                    range_max_inclusive,
                    ..
                } if literal == "256"
                    && target == "u8"
                    && range_min_inclusive == "0"
                    && range_max_inclusive == "255"
            )
        }),
        "expected aliased MagnitudeOutOfRange details, got {messages:?}"
    );
}

#[test]
fn rust_pilot_primitives_integer_witnesses_are_unique() {
    let dag = v3_compiler::Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap diagnostics: {:?}",
        dag.diagnostics()
    );
    let pilot = dag.rust_pilot_primitives().expect("rust_pilot_primitives");
    let ValueBody::List(elements) = pilot.value_body.as_ref().expect("value body") else {
        panic!("expected ValueBody::List");
    };
    let rust_primitive = dag
        .declaration_by_name("RustPrimitive")
        .expect("RustPrimitive type");
    let TypeConnective::Disj { variants } = &rust_primitive.connective else {
        panic!("RustPrimitive must be a sum");
    };
    let integer_primitive_ctor = variants
        .iter()
        .find(|v| v.label == "IntegerPrimitive")
        .expect("IntegerPrimitive variant")
        .ty;
    let mut witnesses = HashSet::new();
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
        let FieldValue::Variant {
            constructor: algebra_ctor,
            ..
        } = &payload[1]
        else {
            panic!("algebra field must be a variant");
        };
        let FieldValue::Variant {
            constructor: carrier_ctor,
            ..
        } = &payload[2]
        else {
            panic!("carrier field must be a variant");
        };
        assert!(
            witnesses.insert((*algebra_ctor, *carrier_ctor)),
            "duplicate integer routing witness in pilot list: {:?}",
            (*algebra_ctor, *carrier_ctor)
        );
    }
    assert_eq!(
        witnesses.len(),
        8,
        "pilot carries eight distinct integer primitive witnesses"
    );
}

#[test]
fn int_literal_range_routing_matches_std_type_witness() {
    let dag = compile_to_dag("data x: UInt8 = 5", "int_literal_witness_u8.v3").expect("compiles");
    let uint8 = dag.declaration_by_name("UInt8").expect("UInt8").id;
    let std_witness = integer_literal_routing_witness(&dag, uint8).expect("UInt8 routing witness");
    let pilot = dag.rust_pilot_primitives().expect("pilot");
    let ValueBody::List(elements) = pilot.value_body.as_ref().expect("list") else {
        panic!("expected list");
    };
    let rust_primitive = dag.declaration_by_name("RustPrimitive").expect("RustPrimitive");
    let TypeConnective::Disj { variants } = &rust_primitive.connective else {
        panic!("RustPrimitive sum");
    };
    let integer_primitive_ctor = variants
        .iter()
        .find(|v| v.label == "IntegerPrimitive")
        .unwrap()
        .ty;
    let mut matches = 0usize;
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
        let FieldValue::Variant {
            constructor: a, ..
        } = &payload[1]
        else {
            continue;
        };
        let FieldValue::Variant {
            constructor: c, ..
        } = &payload[2]
        else {
            continue;
        };
        if (*a, *c) == std_witness {
            matches += 1;
        }
    }
    assert_eq!(
        matches, 1,
        "exactly one IntegerPrimitive row matches UInt8's declaration-identity witness"
    );
}

#[test]
fn int_literal_range_narrowing_does_not_bypass_refinement_discharge() {
    let err = compile_to_dag(
        "type PositiveInt = Int where PositiveInt > 0\n\
         fn requires_positive(x: PositiveInt) -> Int = x\n\
         fn bad() -> Int = requires_positive(1)",
        "int_literal_refinement_discharge.v3",
    )
    .expect_err("range-compatible literal must still fail missing refinement discharge");
    let CompileError::Semantic(dag) = err else {
        panic!("expected semantic diagnostic, got {err:?}");
    };
    let messages: Vec<String> = dag
        .diagnostics()
        .iter()
        .map(|(_, diagnostic)| diagnostic.message())
        .collect();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("refinement") || message.contains("no narrowing")),
        "expected refinement discharge failure, got {messages:?}"
    );
}

#[test]
fn data_int_literal_range_narrowing_does_not_bypass_refinement() {
    for (source, file) in [
        (
            "type PositiveInt = Int where PositiveInt > 0\n\
             data x: PositiveInt = 1",
            "data_int_literal_refinement_discharge.v3",
        ),
        (
            "type PositiveInt = Int where PositiveInt > 0\n\
             type LocalPositive = PositiveInt\n\
             data x: LocalPositive = 1",
            "data_int_literal_refined_alias_discharge.v3",
        ),
    ] {
        let err = compile_to_dag(source, file).expect_err(
            "range-compatible data literal must still fail missing refinement evidence",
        );
        let CompileError::Semantic(dag) = err else {
            panic!("expected semantic diagnostic, got {err:?}");
        };
        let messages: Vec<String> = dag
            .diagnostics()
            .iter()
            .map(|(_, diagnostic)| diagnostic.message())
            .collect();
        assert!(
            messages
                .iter()
                .any(|message| message.contains("refinement") || message.contains("no narrowing")),
            "expected refinement failure for {file}, got {messages:?}"
        );
    }
}

#[test]
fn data_bool_string_scalar_literals_do_not_bypass_refinement() {
    for (source, file) in [
        (
            "type TrueAlias = Bool where TrueAlias == true\n\
             data x: TrueAlias = true",
            "data_bool_literal_refinement_discharge.v3",
        ),
        (
            "type NamedString = String where NamedString != \"\"\n\
             data x: NamedString = \"gunbc\"",
            "data_string_literal_refinement_discharge.v3",
        ),
    ] {
        let err = compile_to_dag(source, file)
            .expect_err("scalar data literal must fail missing refinement evidence");
        let CompileError::Semantic(dag) = err else {
            panic!("expected semantic diagnostic, got {err:?}");
        };
        let messages: Vec<String> = dag
            .diagnostics()
            .iter()
            .map(|(_, diagnostic)| diagnostic.message())
            .collect();
        assert!(
            messages
                .iter()
                .any(|message| message.contains("refinement") || message.contains("no narrowing")),
            "expected refinement failure for {file}, got {messages:?}"
        );
    }
}

#[test]
fn structural_data_scalar_fields_do_not_bypass_refinement() {
    let err = compile_to_dag(
        "type PositiveInt = Int where PositiveInt > 0\n\
         type Box { value: PositiveInt }\n\
         data x: Box = { value: 1 }",
        "structural_data_refined_scalar_field.v3",
    )
    .expect_err("structural scalar field must fail missing refinement evidence");
    let CompileError::Semantic(dag) = err else {
        panic!("expected semantic diagnostic, got {err:?}");
    };
    let messages: Vec<String> = dag
        .diagnostics()
        .iter()
        .map(|(_, diagnostic)| diagnostic.message())
        .collect();
    assert!(
        messages
            .iter()
            .any(|message| message.contains("refinement") || message.contains("no narrowing")),
        "expected refinement failure, got {messages:?}"
    );
}
