//! A4 OPACITY (sharp-gull-122) — CharOffset != ByteOffset brand-twin enforcement
//! via the A3 direct-call arg relation.
//!
//! The canonical "two integers you must not mix" opacity case: CharOffset and
//! ByteOffset are distinct brands over a shared numeric carrier. Passing a
//! CharOffset where a ByteOffset is expected (and vice versa) is the classic
//! offset-confusion bug. This suite pins the A3 gate (node_type_compatible,
//! direct-call ExprCall arm) on that opacity idiom:
//!   - opacity twin (CharOffset for ByteOffset)  -> REJECT
//!   - same brand   (ByteOffset for ByteOffset)  -> ACCEPT
//!
//! These are probes against a USER module (the v2.* / v1.compiler.* substrate is
//! skip-listed by module_skips_direct_call_arg_check, PD-3-DOGFOOD deferral), so
//! the hook is live here. A red is a real enforcement gap.

use v1_compiler::v1_std_core::CompilerDiagnostic;

fn has_type_mismatch(result: &v1_compiler::v1_compiler_compile::PipelineResult) -> bool {
    result
        .diagnostics
        .iter()
        .any(|d| matches!(&*d.diagnostic, CompilerDiagnostic::TypeMismatch { .. }))
}

// ── opacity twin over a Refined<Int> carrier must REJECT ──

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

// Reverse direction — ByteOffset for CharOffset must also REJECT.
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

// ── same brand must ACCEPT (gate is live, not always-reject) ──

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
