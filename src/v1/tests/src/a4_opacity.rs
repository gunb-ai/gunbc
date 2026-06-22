use v1_compiler::v1_std_core::CompilerDiagnostic;

fn has_type_mismatch(result: &v1_compiler::v1_compiler_compile::PipelineResult) -> bool {
    result
        .diagnostics
        .iter()
        .any(|d| matches!(&*d.diagnostic, CompilerDiagnostic::TypeMismatch { .. }))
}

#[test]
fn opacity_charoffset_for_byteoffset_must_reject() {
    let source = r#"
module a4opacity.refined_twin

type Refined<T> {
  base: T
}
type ByteOffset = Refined<Int>
type CharOffset = Refined<Int>

fn at_byte(b: ByteOffset) -> String {
  ""
}

fn caller(c: CharOffset) -> String {
  at_byte(c)
}
"#;
    let result = crate::helpers::compile_dag(source);
    assert!(
        has_type_mismatch(&result),
        "A4: CharOffset passed where ByteOffset expected must be rejected, got: {:?}",
        crate::helpers::diagnostic_messages(&result)
    );
}

#[test]
fn opacity_byteoffset_for_charoffset_must_reject() {
    let source = r#"
module a4opacity.refined_twin_reverse

type Refined<T> {
  base: T
}
type ByteOffset = Refined<Int>
type CharOffset = Refined<Int>

fn at_char(c: CharOffset) -> String {
  ""
}

fn caller(b: ByteOffset) -> String {
  at_char(b)
}
"#;
    let result = crate::helpers::compile_dag(source);
    assert!(
        has_type_mismatch(&result),
        "A4: ByteOffset passed where CharOffset expected must be rejected, got: {:?}",
        crate::helpers::diagnostic_messages(&result)
    );
}

#[test]
fn opacity_same_byteoffset_must_accept() {
    let source = r#"
module a4opacity.same_brand

type Refined<T> {
  base: T
}
type ByteOffset = Refined<Int>

fn at_byte(b: ByteOffset) -> String {
  ""
}

fn caller(b: ByteOffset) -> String {
  at_byte(b)
}
"#;
    let result = crate::helpers::compile_dag(source);
    crate::helpers::assert_no_diagnostics(&result);
}
