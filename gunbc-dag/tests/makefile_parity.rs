//! Golden snapshot test for Makefile rendering.
//!
//! Gates all phases of the fn-level evaluation migration. If this test
//! fails, the rendered Makefile output has drifted from the golden snapshot.
//!
//! To update the snapshot: `UPDATE_GOLDEN=1 cargo test -p gunbc-dag --test makefile_parity`

use gunbc_dag::dsl_builder::build_dsl_graph_for_entry;
use gunbc_dag::makegen::{render_makefile, ToolRegistry};

/// Golden snapshot embedded at compile time.
const GOLDEN: &str = include_str!("fixtures/makefile_golden.txt");

#[test]
fn makefile_output_matches_golden_snapshot() {
    let registry = ToolRegistry::default_registry();
    let actual = render_makefile(&registry);

    // Update mode: write the fixture and return.
    // This is a developer-only codepath (requires explicit UPDATE_GOLDEN=1),
    // not runtime I/O, so the direct fs write is appropriate.
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/makefile_golden.txt"
        );
        #[allow(clippy::disallowed_methods)]
        std::fs::write(path, &actual).expect("failed to write golden snapshot");
        eprintln!("Golden snapshot updated at {path}");
        return;
    }

    assert_output_matches_golden(&actual);
}

/// Verify the DSL rendering graph builds successfully.
///
/// This ensures the compiler wires data declarations and fn call arguments
/// correctly after removing the `render_makefile_content` extern bridge.
#[test]
fn makegen_dsl_graph_builds() {
    let dag = build_dsl_graph_for_entry("tools/makegen.dag", "tools.makegen::makegen")
        .expect("makegen graph should build after extern bridge removal");

    // Verify the graph has the expected shape: discover_tools, fn body delegate,
    // content_upsert, etc.
    assert!(
        dag.nodes.len() >= 5,
        "makegen graph should have at least 5 nodes, got {}",
        dag.nodes.len()
    );
}

fn assert_output_matches_golden(actual: &str) {
    if actual != GOLDEN {
        // Find first differing line for a helpful error message
        let actual_lines: Vec<&str> = actual.lines().collect();
        let expected_lines: Vec<&str> = GOLDEN.lines().collect();

        let first_diff = actual_lines
            .iter()
            .zip(expected_lines.iter())
            .enumerate()
            .find(|(_, (a, e))| a != e);

        let diff_msg = if let Some((i, (actual_line, expected_line))) = first_diff {
            format!(
                "First difference at line {}:\n  expected: {:?}\n  actual:   {:?}",
                i + 1,
                expected_line,
                actual_line,
            )
        } else if actual_lines.len() != expected_lines.len() {
            format!(
                "Line count mismatch: expected {} lines, got {} lines",
                expected_lines.len(),
                actual_lines.len(),
            )
        } else {
            "Content differs (trailing whitespace or encoding difference)".to_string()
        };

        panic!(
            "Makefile output does not match golden snapshot.\n\
             {diff_msg}\n\n\
             To update: UPDATE_GOLDEN=1 cargo test -p gunbc-dag --test makefile_parity"
        );
    }
}
