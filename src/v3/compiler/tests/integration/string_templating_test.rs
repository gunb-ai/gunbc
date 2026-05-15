//! **Layer:** integration
//!
//! Path B Brief 2: executable `.dag` string templating and primitive
//! conversions. The fixtures compile through `.dag`, emit through the normal
//! Rust target, and run as standalone programs.

use crate::common::{HarnessLinkMode, RustcHarness};
use v3_compiler::{compile_to_dag, emit_rust::emit_rust};

fn rust_program_stdout(source: &str, file: &str) -> String {
    let dag = compile_to_dag(source, file).expect("fixture compiles");
    let rust = emit_rust(&dag).expect("fixture emits to Rust");
    let harness = RustcHarness::new("string_templating");
    let bin = harness.compile(&rust, "string_templating", HarnessLinkMode::Standalone);
    RustcHarness::run(&bin, &[])
}

fn rust_program_raw_stdout(source: &str, file: &str) -> String {
    rust_program_stdout(source, file)
}

#[test]
fn int_to_string_outputs_decimal_string() {
    let out = rust_program_stdout(
        r#"
import std.formatting { int_to_string }

let msg: String = int_to_string(42)
"#,
        "string_templating_int_to_string.v3",
    );
    assert_eq!(out, "42");
}

#[test]
fn char_to_string_outputs_single_char_string() {
    let out = rust_program_stdout(
        r#"
import std.formatting { char_to_string }
import std.types { Char }

let ch: Char = 65
let msg: String = char_to_string(ch)
"#,
        "string_templating_char_to_string.v3",
    );
    assert_eq!(out, "A");
}

#[test]
fn bool_to_string_outputs_lowercase_bool_strings() {
    let true_out = rust_program_stdout(
        r#"
import std.formatting { bool_to_string }

let msg: String = bool_to_string(true)
"#,
        "string_templating_bool_true.v3",
    );
    assert_eq!(true_out, "true");

    let false_out = rust_program_stdout(
        r#"
import std.formatting { bool_to_string }

let msg: String = bool_to_string(false)
"#,
        "string_templating_bool_false.v3",
    );
    assert_eq!(false_out, "false");
}

#[test]
fn format_substitutes_indexed_placeholders() {
    let out = rust_program_stdout(
        r#"
import std.formatting { format, int_to_string }

let msg = format("hello {0}! count={1}", ["world", int_to_string(42)])
"#,
        "string_templating_format.v3",
    );
    assert!(
        out.contains("Ok") && out.contains("hello world! count=42"),
        "format should return structural Ok with substituted string, got {out:?}"
    );
}

#[test]
fn format_missing_argument_returns_structural_error() {
    let out = rust_program_raw_stdout(
        r#"
import std.formatting { format }

let msg = format("{1}", ["world"])
"#,
        "string_templating_format_oob.v3",
    );
    assert!(
        out.contains("Err") && out.contains("PlaceholderIndexOutOfBounds"),
        "out-of-bounds format placeholder must return a structural error, got {out:?}"
    );
}
