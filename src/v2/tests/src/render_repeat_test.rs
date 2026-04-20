//! Regression: dsl/std/render.dag repeat_string must repeat n times (P0-A).

use crate::helpers::{assert_no_diagnostics, compile_dag, emitted_file_paths, find_file};

#[test]
fn repeat_string_returns_n_copies_compiles() {
    let src = r#"module test.repeat_string_regression
import std.render { repeat_string, indent_text }
fn repeat_string_returns_n_copies() -> String { repeat_string(s: "x", n: 3) }
fn indent_text_produces_level_times_unit() -> String { indent_text(level: 2, unit: " ", text: "a") }
"#;
    let result = compile_dag(src);
    assert_no_diagnostics(&result);

    // Emitted Rust must implement a descending counter (not singleton-fold).
    let mut saw_loop = false;
    for path in emitted_file_paths(&result) {
        let body = find_file(&result, &path);
        if body.contains("repeat_string_loop") && body.contains("remaining") {
            saw_loop = true;
            break;
        }
    }
    assert!(
        saw_loop,
        "expected emitted crate to contain repeat_string_loop/remaining; paths: {:?}",
        emitted_file_paths(&result)
    );
}
