//! **Layer:** integration
//!
//! R3 §1.8 gate **#62** `substrate_gap_file_ingestion_closed` —
//! **negative-bridge audit (supporting evidence; NOT a §Acceptance
//! PASSING receipt)**. CONSUMER_LANDED supplied via the `FileAttachment`
//! carrier landing in `src/v3/std/timing_lens.dag` plus the structural
//! ratchet in `file_attachment_substrate_carrier_test.rs` and the
//! `gate_62_file_attachment_demo_record` existence proof (PR #2823).
//!
//! Per operator BLOCKING on PR #3111 (2026-05-14T19:13:37Z): §1.4/§4.3
//! of `docs/r3-program-plan.md` require a positive `.dag` program that
//! ingests an external file via `FileAttachment`, not just absence of
//! the old bridge (THESIS/P1 modeling faithfulness). This audit
//! ratchets that no `.dag`/`.v3` program body under `dsl/` re-introduces
//! `include_str!` while the positive ingestion-via-`FileAttachment`
//! demonstration is built out; it is paired with that future positive
//! receipt, not a substitute for it.
//!
//! Per codex REQUEST_CHANGES on PR #3111 (2026-05-14 review #11981): the
//! check operates over **program body** text, not raw bytes — line
//! comments (`// …`), block comments (`/* … */`), and string literals
//! (`"…"` / `` `…` ``) are stripped before the substring search so a
//! comment or doc-block discussing the bridge name does not trip the
//! ratchet. The gate-fact is "no `.dag` program body invokes
//! `include_str!`", not "no `.dag` file's bytes contain the substring".
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
        let program_body = strip_comments_and_string_literals(&body);
        if program_body.contains(NEEDLE) {
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

/// Strip `//` line comments, `/* … */` block comments, and `"…"` / `` `…` ``
/// string literals from `src` so the gate-#62 substring check operates over
/// program-body tokens rather than raw bytes. Conservative single-pass scan
/// with `\` escape handling inside string literals. If an opening `"`,
/// `` ` ``, or `/*` has no closing delimiter, the scanner consumes through
/// end-of-input — anything after the unbalanced opener is dropped, not
/// searched. Realistic `.dag` / `.v3` trees would fail parse elsewhere if
/// they contained unbalanced delimiters; the negative-bridge audit therefore
/// relies on lex-level well-formedness of the substrate it walks. A
/// well-formed `include_str!` token outside any literal remains visible to
/// the needle search.
fn strip_comments_and_string_literals(src: &str) -> String {
    let bytes = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        // `//` line comment: drop through end-of-line, keep the newline.
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // `/* … */` block comment: drop through closing `*/`.
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
            continue;
        }
        // `"…"` or `` `…` `` string literal: drop through matching unescaped quote.
        if b == b'"' || b == b'`' {
            let quote = b;
            i += 1;
            while i < bytes.len() && bytes[i] != quote {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            i = (i + 1).min(bytes.len());
            continue;
        }
        out.push(b as char);
        i += 1;
    }
    out
}

#[cfg(test)]
mod strip_tests {
    use super::{strip_comments_and_string_literals, NEEDLE};

    #[test]
    fn line_comment_mentioning_needle_is_stripped() {
        let src = "// retired: include_str! side-channel\nfn ok() = 0\n";
        assert!(!strip_comments_and_string_literals(src).contains(NEEDLE));
    }

    #[test]
    fn block_comment_mentioning_needle_is_stripped() {
        let src = "/* note: include_str! once lived here */\nfn ok() = 0\n";
        assert!(!strip_comments_and_string_literals(src).contains(NEEDLE));
    }

    #[test]
    fn string_literal_mentioning_needle_is_stripped() {
        let src = "fn label() = \"name: include_str!\"\n";
        assert!(!strip_comments_and_string_literals(src).contains(NEEDLE));
    }

    #[test]
    fn body_invocation_survives_stripping() {
        let src = "fn bad() = include_str!(\"path\")\n";
        assert!(strip_comments_and_string_literals(src).contains(NEEDLE));
    }
}
