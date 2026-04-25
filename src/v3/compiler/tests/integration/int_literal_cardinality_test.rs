//! **Layer:** integration

use v3_compiler::dag::LiteralBits;
use v3_compiler::{compile_to_dag, CompileError};

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
