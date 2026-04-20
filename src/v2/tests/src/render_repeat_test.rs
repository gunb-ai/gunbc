//! Regression: dsl/std/render.dag repeat_string must repeat n times (P0-A).

use crate::helpers::{assert_no_diagnostics, compile_dag};

#[test]
fn repeat_string_returns_n_copies_compiles() {
    let src = r#"module test.repeat_string_regression
import std.render { repeat_string, indent_text }
fn repeat_string_returns_n_copies() -> String { repeat_string(s: "x", n: 3) }
fn indent_text_produces_level_times_unit() -> String { indent_text(level: 2, unit: " ", text: "a") }
"#;
    let result = compile_dag(src);
    assert_no_diagnostics(&result);
}
