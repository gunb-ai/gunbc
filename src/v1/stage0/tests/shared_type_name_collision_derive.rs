//! A heap-owning record must not derive Copy because another module declares the same bare
//! name with scalar fields. build_shared_types keys on the bare name, so the colliding
//! declaration drops the name from the shared set and both take the Copy roster.
//! RED today (collision arm); the renamed control is green and discriminates the cause.
//! Run with cargo test -p v1-compiler --test shared_type_name_collision_derive. Required CI
//! does not execute Rust integration tests; direct execution is not scheduled coverage.

use std::rc::Rc;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};

fn module_a() -> &'static str {
    "module rp_a\nimport rp_b { Pair }\ntype Byte { bits: List<Bool> }\ntype Word64 { bytes: List<Byte> }\ntype Word32 { bytes: List<Byte> }\n"
}

fn module_b(scalar_name: &str) -> String {
    format!("module rp_b\ntype {scalar_name} {{ hi: Int, lo: Int }}\ntype Pair {{ x: Int }}\n")
}

fn emitted_a(b: String) -> String {
    let source = |path: &str, content: String| {
        Rc::new(SourceFile {
            path: path.into(),
            content,
        })
    };
    let result = compile_sources(
        Rc::new(vec![source("rp_a.dag", module_a().into()), source("rp_b.dag", b)].into()),
        RenderTarget::Rust,
    );
    let file = result
        .files
        .iter()
        .find(|file| file.path == "src/rp_a.rs")
        .expect("emission produced rp_a");
    file.content.to_string()
}

fn derive_line_before(emitted: &str, declaration: &str) -> String {
    let at = emitted
        .find(declaration)
        .unwrap_or_else(|| panic!("{declaration} absent from emitted bytes"));
    emitted[..at]
        .lines()
        .rev()
        .find(|l| l.contains("derive("))
        .unwrap()
        .to_string()
}

#[test]
fn control_renamed_scalar_declaration_leaves_heap_record_without_copy() {
    let emitted = emitted_a(module_b("Word65"));
    let line = derive_line_before(&emitted, "pub struct Word64");
    assert!(
        !line.contains("Copy"),
        "control arm derived Copy on a heap record: {line}"
    );
}

#[test]
fn colliding_bare_name_must_not_give_heap_record_copy() {
    let emitted = emitted_a(module_b("Word64"));
    let line = derive_line_before(&emitted, "pub struct Word64");
    assert!(
        !line.contains("Copy"),
        "rp_a.Word64 {{ bytes: List<Byte> }} derived Copy because rp_b declares the same bare name: {line}"
    );
}
