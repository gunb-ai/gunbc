//! **Layer:** integration
//!
//! Unit tests for `tests/integration.rs` wiring scanners shared via `crate::common`
//! (`integration_rs_active_line_contains`, `integration_rs_cementing_path_attr_binds_mod_stem`).

use crate::common::{
    integration_rs_active_line_contains, integration_rs_cementing_path_attr_binds_mod_stem,
};

fn md_table_cells(line: &str) -> Vec<String> {
    let tmp = line.replace("\\|", "\u{241f}");
    tmp.split('|')
        .map(|s| s.replace('\u{241f}', "|").trim().to_string())
        .collect()
}

fn register_row_has_real_v2_counterpart(v2_cell: &str) -> bool {
    let v2 = v2_cell.trim().trim_matches('`');
    !(v2.contains("None (v3-native)") || v2 == "N/A")
}

#[test]
fn register_row_has_real_v2_counterpart_matches_testing_md_band_c() {
    assert!(!register_row_has_real_v2_counterpart("None (v3-native)"));
    assert!(!register_row_has_real_v2_counterpart(
        "  None (v3-native)  "
    ));
    assert!(!register_row_has_real_v2_counterpart("N/A"));
    assert!(!register_row_has_real_v2_counterpart("  `N/A`  "));
    assert!(register_row_has_real_v2_counterpart(
        "src/v3/lenses/complexity.dag (586L)"
    ));
}

#[test]
fn integration_rs_active_line_contains_rejects_commented_cementing_path() {
    let src = r#"
// #[path = "integration/cementing/ghost.rs"]
// mod ghost;
#[path = "integration/cementing/real.rs"]
mod real;
"#;
    assert!(!integration_rs_active_line_contains(
        src,
        r#"#[path = "integration/cementing/ghost.rs"]"#,
    ));
    assert!(integration_rs_active_line_contains(
        src,
        r#"#[path = "integration/cementing/real.rs"]"#,
    ));
}

#[test]
fn integration_rs_active_line_contains_rejects_block_commented_cementing_path() {
    let src = r#"/*
#[path = "integration/cementing/ghost.rs"]
mod ghost;
*/
#[path = "integration/cementing/real.rs"]
mod real;
"#;
    assert!(!integration_rs_active_line_contains(
        src,
        r#"#[path = "integration/cementing/ghost.rs"]"#,
    ));
    assert!(integration_rs_active_line_contains(
        src,
        r#"#[path = "integration/cementing/real.rs"]"#,
    ));
}

#[test]
#[should_panic(expected = "raw/byte string literal")]
fn integration_rs_active_line_contains_panics_on_raw_string_in_code() {
    let src = concat!("fn f() { r#\"", "hi", "\"#; }");
    integration_rs_active_line_contains(src, "nope");
}

#[test]
fn integration_rs_cementing_path_attr_binds_mod_accepts_multiline() {
    let src = concat!(
        "#[path = \"integration/cementing/real.rs\"]\n",
        "mod real;\n",
    );
    assert!(integration_rs_cementing_path_attr_binds_mod_stem(
        src, "real"
    ));
}

#[test]
fn integration_rs_cementing_path_attr_binds_mod_accepts_same_line() {
    let src = "#[path = \"integration/cementing/real.rs\"] mod real;\n";
    assert!(integration_rs_cementing_path_attr_binds_mod_stem(
        src, "real"
    ));
}

#[test]
fn integration_rs_cementing_path_attr_binds_mod_accepts_interleaved_attribute() {
    let src = concat!(
        "#[path = \"integration/cementing/real.rs\"]\n",
        "#[allow(dead_code)]\n",
        "mod real;\n",
    );
    assert!(integration_rs_cementing_path_attr_binds_mod_stem(
        src, "real"
    ));
}

#[test]
fn integration_rs_cementing_path_attr_binds_mod_rejects_mismatched_mod_name() {
    let src = concat!(
        "#[path = \"integration/cementing/real.rs\"]\n",
        "mod decoy;\n\n",
        "mod real;\n",
    );
    assert!(!integration_rs_cementing_path_attr_binds_mod_stem(
        src, "real"
    ));
}

#[test]
fn integration_rs_cementing_path_attr_binds_mod_rejects_stem_extra_mod_name() {
    let src = concat!(
        "#[path = \"integration/cementing/stem.rs\"]\n",
        "mod stem_extra;\n",
    );
    assert!(!integration_rs_cementing_path_attr_binds_mod_stem(
        src, "stem"
    ));
}

#[test]
fn integration_rs_active_line_ignores_needle_inside_string_literal() {
    let src = concat!(
        "let _ = \"#[path = \\\"integration/cementing/decoy.rs\\\"]\";\n",
        "#[path = \"integration/cementing/real.rs\"]\n",
        "mod real;\n",
    );
    assert!(!integration_rs_active_line_contains(
        src,
        r#"#[path = "integration/cementing/decoy.rs"]"#,
    ));
    assert!(integration_rs_active_line_contains(
        src,
        r#"#[path = "integration/cementing/real.rs"]"#,
    ));
}

#[test]
fn md_table_cells_preserves_escaped_pipes_for_register_capability_rows() {
    let row = "| a.dag | TERMINAL | COMPLETE | v2 path | FoundCost(Int) \\| MissingCost | note |";
    let cells = md_table_cells(row);
    assert!(
        cells.len() >= 5,
        "capability ratchet expects Lens, Structural, Behavioral, v2, … columns; got {cells:?}"
    );
    assert_eq!(cells[1].trim(), "a.dag");
    assert_eq!(cells[3].trim(), "COMPLETE");
    assert_eq!(cells[4].trim(), "v2 path");
    assert_eq!(
        cells[5].trim(),
        "FoundCost(Int) | MissingCost",
        "cell-internal `|` must survive `\\|` escaping (v3 output column)"
    );
}
