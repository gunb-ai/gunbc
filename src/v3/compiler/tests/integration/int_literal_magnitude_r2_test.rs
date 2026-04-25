//! **Layer:** integration
//!
//! R2 T-Substrate: int literal carries i128 magnitude; reconciliation
//! enforces `std/integer.dag` width bounds (`UInt8`, `Int`, etc.).

use v3_compiler::dag::{Dag, LiteralBits, ValueBody};
use v3_compiler::diagnostics::Diagnostic;
use v3_compiler::{compile_to_dag, CompileError};

fn first_magnitude_out_of_range(dag: &Dag) -> Option<(i128, i128, i128, String)> {
    dag.diagnostics()
        .iter()
        .find_map(|(_, d)| match d {
            Diagnostic::MagnitudeOutOfRange {
                value,
                min,
                max,
                target,
                ..
            } => Some((*value, *min, *max, target.clone())),
            _ => None,
        })
}

#[test]
fn data_int_accepts_i64_min_literal() {
    let src = "data min_i64: Int = -9223372036854775808\n";
    let dag = compile_to_dag(src, "int_i64min.v3").expect("compile");
    let decl = dag
        .declarations()
        .iter()
        .find(|d| d.name.as_deref() == Some("min_i64"))
        .expect("data decl");
    let body = decl.value_body.as_ref().expect("value body");
    let ValueBody::Scalar(LiteralBits::Int(v)) = body else {
        panic!("expected scalar int, got {body:?}");
    };
    assert_eq!(*v, i64::MIN as i128);
}

#[test]
fn data_uint8_256_is_magnitude_out_of_range() {
    let src = "data oob: UInt8 = 256\n";
    let err = compile_to_dag(src, "uint8_oob.v3").expect_err("should fail");
    let dag = match err {
        CompileError::Semantic(d) => d,
        other => panic!("expected semantic error, got {other:?}"),
    };
    let got = first_magnitude_out_of_range(&dag).expect("MagnitudeOutOfRange");
    assert_eq!(got.0, 256);
    assert_eq!(got.1, 0);
    assert_eq!(got.2, 255);
    assert!(!got.3.is_empty());
}

#[test]
fn let_annotated_uint8_in_range_passes() {
    let src = "let x: UInt8 = 5\n";
    let dag = compile_to_dag(src, "u8_let.v3").expect("compile");
    assert!(dag.diagnostics().is_empty(), "{:?}", dag.diagnostics().iter().collect::<Vec<_>>());
}

#[test]
fn let_annotated_uint8_256_magnitude_error() {
    let src = "let x: UInt8 = 256\n";
    let err = compile_to_dag(src, "u8_let_oob.v3").expect_err("expected failure");
    let dag = match err {
        CompileError::Semantic(d) => d,
        other => panic!("expected semantic, got {other:?}"),
    };
    assert!(first_magnitude_out_of_range(&dag).is_some());
}
