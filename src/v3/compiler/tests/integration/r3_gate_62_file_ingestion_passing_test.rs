//! **Layer:** integration
//!
//! R3 §1.8 gate **#62** `substrate_gap_file_ingestion_closed` — PASSING
//! receipt. CONSUMER_LANDED already supplied via the `FileAttachment`
//! carrier landing in `src/v3/std/timing_lens.dag` plus the structural
//! ratchet in `file_attachment_substrate_carrier_test.rs` and the
//! `gate_62_file_attachment_demo_record` existence proof (PR #2823).
//!
//! §Acceptance per `docs/r3-program-plan.md` §1.8 row #62: ".dag program
//! ingests external file w/o `include_str!`". The gate is forward-looking
//! (Director-verified at ratification time on PR #2820 canvas — no
//! `include_str!` matches under `dsl/`). This test promotes that
//! one-time grep into a CI-visible ratchet on tree state, distinct from
//! grep-on-doc-comment textual-enforcement (`feedback_no_textual_enforcement_bridges`):
//! the predicate is over the file tree's authoritative substrate body,
//! not over comments or briefs that mention the string.
//!
//! STOP-AND-PING trigger (per parent Mgr brief msg_210620aa): ANY match
//! at HEAD is a substrate-gap regression, not a paper-over — investigate
//! and route through the workflow-substrate carrier (`FileAttachment`)
//! before re-greening this test.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

const DSL_ROOT: &str = "dsl";
const NEEDLE: &str = "include_str!";

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(3)
        .expect("workspace root is three ancestors above src/v3/compiler/")
        .to_path_buf()
}

fn walk_dag_files(root: &Path, out: &mut BTreeSet<PathBuf>) {
    let entries = fs::read_dir(root).unwrap_or_else(|e| panic!("read_dir {}: {e}", root.display()));
    for entry in entries {
        let entry = entry.expect("read_dir entry");
        let path = entry.path();
        if path.is_dir() {
            walk_dag_files(&path, out);
        } else if path.extension() == Some(OsStr::new("dag"))
            || path.extension() == Some(OsStr::new("v3"))
        {
            out.insert(path);
        }
    }
}

#[test]
fn r3_gate_62_no_include_str_in_dsl() {
    let ws = workspace_root();
    let dsl_root = ws.join(DSL_ROOT);
    assert!(
        dsl_root.is_dir(),
        "expected workspace `{DSL_ROOT}/` directory at {}",
        dsl_root.display()
    );

    let mut files = BTreeSet::new();
    walk_dag_files(&dsl_root, &mut files);
    assert!(
        !files.is_empty(),
        "expected at least one `.dag`/`.v3` file under `{DSL_ROOT}/`"
    );

    let mut hits: Vec<String> = Vec::new();
    for path in &files {
        let body =
            fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        if body.contains(NEEDLE) {
            let rel = path.strip_prefix(&ws).unwrap_or(path).display().to_string();
            hits.push(rel);
        }
    }

    assert!(
        hits.is_empty(),
        "R3 gate #62 (`substrate_gap_file_ingestion_closed`) requires zero \
         `include_str!` occurrences under `{DSL_ROOT}/`. Workflow-substrate \
         file ingestion belongs on the ratified `FileAttachment` carrier \
         (`src/v3/std/timing_lens.dag`; Refined-B-1 per PR #2820 canvas / \
         PR #2823 land). Offending files:\n  {}",
        hits.join("\n  ")
    );
}
