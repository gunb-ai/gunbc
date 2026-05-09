//! **Layer:** integration

use std::collections::HashSet;

use v3_compiler::dag::{
    literal_bits_int, Behavior, CardinalityBound, FieldValue, LiteralBits, PortState,
    TypeConnective, ValueBody,
};
use v3_compiler::emit_rust;
use v3_compiler::{compile_to_dag, integer_literal_routing_witness, CompileError};

#[derive(Debug, Clone, Copy)]
struct IntegerOverflowCase {
    ty: &'static str,
    target: &'static str,
    literal: &'static str,
    min: &'static str,
    max: &'static str,
    check_alias: bool,
}

fn assert_magnitude_out_of_range(source: String, file: &str, case: IntegerOverflowCase) {
    let err = match compile_to_dag(&source, file) {
        Ok(_) => panic!("{file}: `{}` overflow must fail closed", case.ty),
        Err(err) => err,
    };
    let CompileError::Semantic(dag) = err else {
        panic!("{file}: expected semantic diagnostic, got {err:?}");
    };
    assert_eq!(
        dag.diagnostics().len(),
        1,
        "{file}: out-of-range integer literal should emit one root-cause diagnostic, got {:#?}",
        dag.diagnostics()
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
                } if literal == case.literal
                    && target == case.target
                    && range_min_inclusive == case.min
                    && range_max_inclusive == case.max
                    && fixes.is_empty()
            )
        }),
        "{file}: expected MagnitudeOutOfRange for {case:?}, got {:#?}",
        dag.diagnostics()
    );
}

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
        matches!(&decl.value_body, Some(ValueBody::Scalar(LiteralBits::Int(n))) if n.parse::<i64>().ok() == Some(literal)),
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
    assert_int_value_port_resolves_to_uint8_in_file(dag, literal, context, None)
}

fn assert_int_value_port_resolves_to_uint8_in_file(
    dag: &v3_compiler::dag::Dag,
    literal: i64,
    context: &str,
    span_file: Option<&str>,
) {
    // S9 Slice 2.5: registry-driven `range` body synthesis creates additional
    // `Value(LiteralBits::Int(_))` nodes in the DAG for each types.dag
    // declaration's record fields (e.g. `RetryCount = Int where range(min:
    // 1, max: 5)` produces Int(1) and Int(5) nodes). When the user-literal
    // and a synthesized literal share the same bit-pattern, find the one in
    // the user's source file via `span.file`. Falls back to first match if
    // no file filter is provided (preserves prior behavior for tests where
    // the literal is unambiguous).
    let value = dag
        .nodes()
        .iter()
        .find_map(|node| match node {
            Behavior::Value(v)
                if v.data == literal_bits_int(literal)
                    && span_file.is_none_or(|f| v.span.file == f) =>
            {
                Some(v)
            }
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!(
                "{context}: int literal {literal} value node not found in DAG \
                 (filter span.file={span_file:?})"
            );
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

/// R2 Modeling Manager structural acceptance — `int_lit_magnitude_overflow_compile_error`:
/// out-of-range literal vs fixed `ExactInterval` bounds surfaces [`Diagnostic::MagnitudeOutOfRange`].
#[test]
fn int_lit_magnitude_overflow_compile_error() {
    let err = compile_to_dag("data x: UInt8 = 256", "int_lit_gate_u8_oob.v3").expect_err("OOB");
    let CompileError::Semantic(dag) = err else {
        panic!("expected semantic failure, got {err:?}");
    };
    assert!(
        dag.diagnostics().iter().any(|(_, d)| {
            matches!(
                d,
                v3_compiler::diagnostics::Diagnostic::MagnitudeOutOfRange { literal, .. }
                    if literal == "256"
            )
        }),
        "expected MagnitudeOutOfRange, got {:?}",
        dag.diagnostics()
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
            v3_compiler::dag::Behavior::Value(value) if value.data == literal_bits_int(5) => {
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
    // Filter to the user-source span — registry-driven `range` body
    // synthesis adds Int(5) literal nodes from `dsl/std/types.dag`
    // declarations like `RetryCount = Int where range(min: 1, max: 5)`.
    assert_int_value_port_resolves_to_uint8_in_file(
        &dag,
        5,
        "let u8 in-range",
        Some("let_u8_in_range.v3"),
    );
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
fn let_annotated_uint8_literal_resolves_to_narrow_type() {
    let dag = compile_to_dag("let x: UInt8 = 5\n", "let_u8_narrow.v3").expect("compiles");
    // S9 Slice 2.5: synthesized `range(max: 5)` predicate bodies in
    // `dsl/std/types.dag` (e.g. `RetryCount`) also create `Int(5)` literal
    // nodes; filter by the user-source file to find the right one.
    let value = dag
        .nodes()
        .iter()
        .find_map(|node| match node {
            v3_compiler::dag::Behavior::Value(v)
                if v.data == literal_bits_int(5) && v.span.file == "let_u8_narrow.v3" =>
            {
                Some(v)
            }
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
            v3_compiler::dag::Behavior::Value(v) if v.data == literal_bits_int(7) => Some(v),
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
    assert_eq!(
        dag.diagnostics().len(),
        1,
        "out-of-range integer literal should emit one root-cause diagnostic, got {:#?}",
        dag.diagnostics()
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
        "MagnitudeOutOfRange should carry typed bounds and no fabricated correction, got {:#?}",
        dag.diagnostics()
    );
}

/// R3 gate #21 — `int_refinement_overflow_proven_parametric`.
///
/// The proof obligation is not "UInt8 has a one-off overflow check"; every
/// fixed-width integer refinement with a source-representable out-of-range
/// literal must route through the same structural range facts and produce the
/// same typed [`MagnitudeOutOfRange`](v3_compiler::diagnostics::Diagnostic::MagnitudeOutOfRange)
/// diagnostic. Upper-half `UInt64` / `UInt128` and full `Int128` overflow
/// source literals remain blocked by the current i64 source literal carrier,
/// which is covered separately by `uint64_upper_half_literals_are_tracked_carrier_limitation`.
/// `UInt128` is still included for its source-representable lower-bound
/// overflow (`-1`). Alias coverage is representative rather than exhaustive
/// so this receipt stays under the CI per-test wall-clock ratchet.
#[test]
fn int_refinement_overflow_is_proven_parametric_for_representable_widths() {
    let cases = [
        IntegerOverflowCase {
            ty: "Int8",
            target: "i8",
            literal: "128",
            min: "-128",
            max: "127",
            check_alias: false,
        },
        IntegerOverflowCase {
            ty: "Int8",
            target: "i8",
            literal: "-129",
            min: "-128",
            max: "127",
            check_alias: false,
        },
        IntegerOverflowCase {
            ty: "Int16",
            target: "i16",
            literal: "32768",
            min: "-32768",
            max: "32767",
            check_alias: false,
        },
        IntegerOverflowCase {
            ty: "Int32",
            target: "i32",
            literal: "2147483648",
            min: "-2147483648",
            max: "2147483647",
            check_alias: true,
        },
        IntegerOverflowCase {
            ty: "UInt8",
            target: "u8",
            literal: "256",
            min: "0",
            max: "255",
            check_alias: false,
        },
        IntegerOverflowCase {
            ty: "UInt8",
            target: "u8",
            literal: "-1",
            min: "0",
            max: "255",
            check_alias: false,
        },
        IntegerOverflowCase {
            ty: "UInt16",
            target: "u16",
            literal: "65536",
            min: "0",
            max: "65535",
            check_alias: false,
        },
        IntegerOverflowCase {
            ty: "UInt32",
            target: "u32",
            literal: "4294967296",
            min: "0",
            max: "4294967295",
            check_alias: true,
        },
        IntegerOverflowCase {
            ty: "UInt64",
            target: "u64",
            literal: "-1",
            min: "0",
            max: "18446744073709551615",
            check_alias: false,
        },
        IntegerOverflowCase {
            ty: "UInt128",
            target: "u128",
            literal: "-1",
            min: "0",
            max: "340282366920938463463374607431768211455",
            check_alias: false,
        },
    ];

    for case in cases {
        assert_magnitude_out_of_range(
            format!("data x: {} = {}", case.ty, case.literal),
            &format!(
                "parametric_overflow_{}_{}.v3",
                case.ty,
                case.literal.replace('-', "neg")
            ),
            case,
        );
        if case.check_alias {
            assert_magnitude_out_of_range(
                format!("type Alias = {}\ndata x: Alias = {}", case.ty, case.literal),
                &format!(
                    "parametric_alias_overflow_{}_{}.v3",
                    case.ty,
                    case.literal.replace('-', "neg")
                ),
                case,
            );
        }
    }
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
    assert_eq!(
        dag.diagnostics().len(),
        1,
        "aliased out-of-range integer literal should emit one root-cause diagnostic, got {:#?}",
        dag.diagnostics()
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
        "expected aliased MagnitudeOutOfRange details, got {:#?}",
        dag.diagnostics()
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
        10,
        "pilot carries ten distinct integer primitive witnesses (i8..i64, i128, u8..u64, u128); \
         u128 row unblocked by R3 Phase A `IntervalInt::ExactInterval` BigInt host repr widening \
         per gunbc#1739 #issuecomment-4392731264 + Option (ii) at #issuecomment-4393145631"
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
    let rust_primitive = dag
        .declaration_by_name("RustPrimitive")
        .expect("RustPrimitive");
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
        let FieldValue::Variant { constructor: a, .. } = &payload[1] else {
            continue;
        };
        let FieldValue::Variant { constructor: c, .. } = &payload[2] else {
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

/// T-ImpossibleBugs nested-optional flatten: `AtMostOne ∧ AtMostOne = AtMostOne`
/// must hold for every `TypeConnective::Cardinality` declaration in the DAG,
/// regardless of whether the cardinality was minted via `alloc_cardinality_decl`,
/// the non-allocating `type_connective_cardinality` helper, or via generic
/// substitution that walks `resolve_decl_with_subst` on a `Cardinality` node.
fn assert_no_nested_at_most_one(dag: &v3_compiler::dag::Dag, context: &str) {
    for decl in dag.declarations() {
        let TypeConnective::Cardinality(payload) = &decl.connective else {
            continue;
        };
        if payload.bound() != CardinalityBound::AtMostOne {
            continue;
        }
        let inner = dag.declaration(payload.element());
        if let TypeConnective::Cardinality(inner_payload) = &inner.connective {
            assert!(
                inner_payload.bound() != CardinalityBound::AtMostOne,
                "{context}: declaration#{outer} (name={outer_name:?}) wraps \
                 declaration#{inner} (name={inner_name:?}) in AtMostOne, but the \
                 inner declaration is itself Cardinality(AtMostOne, …) — \
                 the idempotence rule was bypassed",
                outer = decl.id.raw(),
                outer_name = decl.name,
                inner = inner.id.raw(),
                inner_name = inner.name,
            );
        }
    }
}

#[test]
fn nested_optional_flatten_holds_in_bootstrap_dag() {
    let dag = compile_to_dag("data probe: Int = 0\n", "nested_optional_bootstrap.v3")
        .expect("trivial program compiles");
    assert_no_nested_at_most_one(&dag, "bootstrap");
}

#[test]
fn nested_optional_flatten_holds_for_surface_double_question() {
    let src = "\
fn flatten_probe(x: Int??) -> Int? = x
";
    let dag = compile_to_dag(src, "nested_optional_surface_double_question.v3")
        .expect("surface Int?? should silently normalize to Int?");
    assert!(
        dag.diagnostics().is_empty(),
        "surface nested optional should be diagnostic-free, got: {:?}",
        dag.diagnostics()
    );
    assert_no_nested_at_most_one(&dag, "surface T??");
}

#[test]
fn nested_optional_flatten_via_generic_specialization() {
    // `unwrap_id` is generic over T and takes/returns `T?`. Calling it with
    // `Int?` makes substitution ask for `Cardinality(AtMostOne, Int?-decl)`,
    // where `Int?-decl` is itself `Cardinality(AtMostOne, Int)`. Without
    // the idempotence rule wired into `resolve_decl_with_subst`, that walk
    // would mint or re-use a nested `AtMostOne` declaration; with it, the
    // walk lands on the existing single-AtMostOne declaration.
    let src = "\
fn unwrap_id<T>(x: T?) -> T? = x
fn use_it(o: Int?) -> Int? = unwrap_id(o)
";
    let dag = compile_to_dag(src, "nested_optional_generic_specialization.v3").expect("compiles");
    assert!(
        dag.diagnostics().is_empty(),
        "generic optional specialization should be diagnostic-free, got: {:?}",
        dag.diagnostics()
    );
    assert_no_nested_at_most_one(&dag, "generic specialization");
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
