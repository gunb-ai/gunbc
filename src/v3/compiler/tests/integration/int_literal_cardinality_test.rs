//! **Layer:** integration

use v3_compiler::dag::{Behavior, LiteralBits, PortState, ValueBody};
use v3_compiler::emit_rust;
use v3_compiler::{compile_to_dag, CompileError};

fn assert_data_value_scalar_typed_u8(
    dag: &v3_compiler::dag::Dag,
    name: &str,
    literal: i64,
    context: &str,
) {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("{context}: no declaration named `{name}`"));
    assert!(
        matches!(&decl.value_body, Some(ValueBody::Scalar(LiteralBits::Int(n))) if *n == literal),
        "{context}: {name} value should be int literal {literal}, got {:?}",
        decl.value_body
    );
    // `lower_data_item` stores the annotation on `connective` + a `meta_tag` edge; there is
    // no `inhabits` link for scalar `data` items today.
    let ty = decl
        .meta_tag
        .unwrap_or_else(|| panic!("{context}: {name} missing meta_tag to type decl"));
    assert_eq!(
        dag.declaration(ty).name.as_deref(),
        Some("UInt8"),
        "{context}: data `meta_tag` should point at the `UInt8` type declaration"
    );
}

fn assert_int_value_port_resolves_to_uint8(
    dag: &v3_compiler::dag::Dag,
    literal: i64,
    context: &str,
) {
    let value = dag
        .nodes()
        .iter()
        .find_map(|node| match node {
            Behavior::Value(v) if v.data == LiteralBits::Int(literal) => Some(v),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!("{context}: int literal {literal} value node not found in DAG");
        });
    let ty = match dag.port(value.output).state() {
        PortState::Resolved(ty) => ty,
        other => panic!("{context}: literal {literal} should resolve, got {other:?}"),
    };
    assert_eq!(
        dag.declaration(ty.declaration).name.as_deref(),
        Some("UInt8"),
        "{context}: in-range u8 should resolve the value port to UInt8, not default Int"
    );
}

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

/// Regression: lowering pre-seeds the value port to a narrow int annotation, while
/// `decide(Behavior::Value)` still stamps the default `Int` shape. Inference must
/// reconcile (in-range) or `MagnitudeOutOfRange` (OOB) — not a `TypeMismatch`.
#[test]
fn let_annotated_uint8_in_range_literal_narrows_against_preseed() {
    let dag = compile_to_dag("let x: UInt8 = 5", "let_u8_in_range.v3")
        .expect("in-range annotated u8 `let` must not spuriously report Int vs narrow mismatch");
    assert!(dag.diagnostics().is_empty(), "{:?}", dag.diagnostics());
    assert_int_value_port_resolves_to_uint8(&dag, 5, "let u8 in-range");
}

#[test]
fn data_annotated_uint8_in_range_literal_narrows_against_preseed() {
    let dag = compile_to_dag("data d: UInt8 = 5", "data_u8_in_range.v3")
        .expect("in-range annotated u8 `data` must not spuriously report Int vs narrow mismatch");
    assert!(dag.diagnostics().is_empty(), "{:?}", dag.diagnostics());
    // Data bodies use declaration `value` nodes / `inhabits` edges — not always a
    // top-level `Behavior::Value` with the same wiring as `let`.
    assert_data_value_scalar_typed_u8(&dag, "d", 5, "data u8 in-range");
}

/// Call-site literal: `decide_transform` must narrow the argument `7` to `UInt8` when
/// the callee parameter is `UInt8` (same range facts as `let` / `data`, different site).
#[test]
fn call_site_u8_literal_narrows_against_uint8_parameter() {
    // Avoid `id8` / `id_u8` name collisions with std/bootstrap templates.
    let dag = compile_to_dag(
        "fn u8_id_for_call_site_test(x: UInt8) -> UInt8 = x\n\
         let r: UInt8 = u8_id_for_call_site_test(7)\n",
        "call_u8_narrow.v3",
    )
    .expect("call with u8-sized literal at UInt8 parameter should compile");
    assert!(dag.diagnostics().is_empty(), "{:?}", dag.diagnostics());
    assert_int_value_port_resolves_to_uint8(&dag, 7, "call id_u8(7) argument literal");
}

/// OOB for `data` is covered in `out_of_range_uint8_literal_emits_magnitude_diagnostic`.
/// This pins the same `MagnitudeOutOfRange` contract for a **let** (pre-seeded path).
#[test]
fn let_annotated_uint8_out_of_range_emits_magnitude_diagnostic() {
    let err = compile_to_dag("let x: UInt8 = 256", "int_literal_u8_oob_let.v3")
        .expect_err("let UInt8 OOB must fail closed");
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
        "MagnitudeOutOfRange for let should match `data` OOB shape"
    );
}

/// Emit must surface narrow Rust backing (`u8`) for UInt8 — this ratchets the
/// `TypeConnective::Cardinality` + rust primitive bridge without a full `rustc` roundtrip.
#[test]
fn emit_rust_uint8_let_mentions_rust_u8() {
    let dag =
        compile_to_dag("let x: UInt8 = 5", "emit_rust_u8_let.v3").expect("emit u8: let compiles");
    let out = emit_rust::emit_rust(&dag).expect("emit");
    assert!(
        out.contains("u8") || out.contains("UInt8"),
        "expected `u8` (or v3 `UInt8` trace) in emit output; got: {}",
        &out.chars().take(800).collect::<String>()
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
fn duplicate_integer_range_fact_fails_closed() {
    let err = compile_to_dag(
        "data duplicate_u8_range: IntegerRangeFact = {\n\
           target_name: \"u8-duplicate\",\n\
           algebra: SemiringAlgebra,\n\
           carrier: ByteCarrier,\n\
           range_min_inclusive: \"0\",\n\
           range_max_inclusive: \"255\"\n\
         }\n\
         data x: UInt8 = 255",
        "int_literal_duplicate_range_fact.v3",
    )
    .expect_err("duplicate range facts for a routing key must fail closed");
    let CompileError::Semantic(dag) = err else {
        panic!("expected semantic diagnostic, got {err:?}");
    };
    let messages: Vec<String> = dag
        .diagnostics()
        .iter()
        .map(|(_, diagnostic)| diagnostic.message())
        .collect();
    assert!(
        dag.diagnostics().iter().any(|(_, diagnostic)| {
            matches!(
                diagnostic,
                v3_compiler::diagnostics::Diagnostic::MalformedIntegerRangeFact {
                    message,
                    fixes,
                    ..
                } if message.contains("duplicate IntegerRangeFact")
                    && fixes.is_empty()
            )
        }),
        "duplicate range key should emit malformed fact diagnostic, got {messages:?}"
    );
}

#[test]
fn malformed_integer_range_fact_fails_closed() {
    let err = compile_to_dag(
        "data malformed_u8_range: IntegerRangeFact = {\n\
           target_name: \"u8-malformed\",\n\
           algebra: SemiringAlgebra,\n\
           carrier: ByteCarrier,\n\
           range_min_inclusive: \"0\",\n\
           range_max_inclusive: \"not-a-number\"\n\
         }\n\
         data x: UInt8 = 255",
        "int_literal_malformed_range_fact.v3",
    )
    .expect_err("malformed range fact for a routing key must fail closed");
    let CompileError::Semantic(dag) = err else {
        panic!("expected semantic diagnostic, got {err:?}");
    };
    let messages: Vec<String> = dag
        .diagnostics()
        .iter()
        .map(|(_, diagnostic)| diagnostic.message())
        .collect();
    assert!(
        dag.diagnostics().iter().any(|(_, diagnostic)| {
            matches!(
                diagnostic,
                v3_compiler::diagnostics::Diagnostic::MalformedIntegerRangeFact {
                    message,
                    fixes,
                    ..
                } if message.contains("malformed IntegerRangeFact")
                    && fixes.is_empty()
            )
        }),
        "malformed range key should emit malformed fact diagnostic, got {messages:?}"
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
