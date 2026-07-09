//! CI-coverage-completeness lens (DESIGN §5/§6 residue).
//!
//! The rust gate runs the WHOLE `v1-compiler-tests` suite (no allowlist filter):
//! coverage = "every test runs" holds BY CONSTRUCTION — cargo's default is
//! run-all-EXCEPT-`#[ignore]`d, so a new test is covered the moment it is written
//! (`dag/gunbc/ci_spec.dag` `ci_rust_gate_test_command`). The single authority for
//! "excused from the gate" is therefore the `#[ignore = "..."]` attribute in the
//! source itself — visible and reviewable.
//!
//! This lens is the genuinely-unstructurable residue: we cannot restructure rustc's
//! `#[ignore]`, so we cannot make a reasonless excuse *unwritable*. What we CAN do is
//! make it loud — fail-closed. Two holes are closed:
//!   1. an `#[ignore]` with no written reason (a silent skip), and
//!   2. a `*.rs` test file not declared `mod` in `lib.rs` (a test that exists but
//!      never compiles/runs).
//!
//! Both leave a test "neither run nor excused-with-reason"; both go RED here.
//!
//! Because this lens is itself a test in the widened suite, the gate covers its own
//! completeness check — there is no separate wiring to drift.

use std::path::{Path, PathBuf};

use crate::helpers::workspace_root;

/// Directory holding this package's test sources.
fn tests_src_dir() -> PathBuf {
    workspace_root().join("src/v1/tests/src")
}

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// THE single-authority detector. Returns the 1-based line numbers of every
/// `#[ignore]` attribute in `source` that does NOT carry a non-empty written
/// reason (`#[ignore = "..."]`). A bare `#[ignore]`, an `#[ignore] // comment`
/// (the reason must live in the attribute, not a comment), and `#[ignore = ""]`
/// all count as reasonless.
///
/// Only physical lines whose trimmed text STARTS WITH `#[ignore` are considered,
/// so prose mentions of the token inside `//` comments or string literals (this
/// file's own fixtures) are not matched.
fn reasonless_ignore_lines(source: &str) -> Vec<usize> {
    let mut hits = Vec::new();
    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("#[ignore") else {
            continue;
        };
        let rest = rest.trim_start();
        let reasoned = if let Some(after_eq) = rest.strip_prefix('=') {
            // Expect a string literal; the reason is its non-empty contents.
            let after_eq = after_eq.trim_start();
            match after_eq.strip_prefix('"') {
                // Closed on this line: reason is the content between the quotes.
                Some(body) if body.contains('"') => {
                    let end = body.find('"').expect("contains('\"') implies find");
                    !body[..end].trim().is_empty()
                }
                // Opened but not closed on this line → a multi-line reason
                // (`\`-continued or a raw string spanning lines). Reasoned iff the
                // content before the line break (sans a trailing `\` continuation)
                // is non-empty — a written reason that simply wraps.
                Some(body) => !body.trim_end().trim_end_matches('\\').trim().is_empty(),
                // `= ` with no opening quote → malformed, reasonless.
                None => false,
            }
        } else {
            // `]` (bare) or anything else (e.g. trailing comment) → reasonless.
            false
        };
        if !reasoned {
            hits.push(i + 1);
        }
    }
    hits
}

/// Module names declared in `lib.rs` (`mod foo;` / `pub mod foo;`).
///
/// ASSUMES a FLAT module tree — every `*_test.rs` is a top-level `mod` in `lib.rs`, which holds
/// today. If anyone nests test modules under subdirs, this top-level-only scan would miss the
/// nested declaration and `every_test_file_is_declared_in_lib` would false-positive; at that point
/// it must recurse into nested `mod { … }` blocks / `foo/mod.rs`. Flagged in review of #5427.
fn declared_modules(lib_rs: &str) -> std::collections::HashSet<String> {
    let mut mods = std::collections::HashSet::new();
    for line in lib_rs.lines() {
        let t = line.trim_start();
        let t = t.strip_prefix("pub ").unwrap_or(t);
        if let Some(rest) = t.strip_prefix("mod ") {
            if let Some(name) = rest.split([';', ' ', '{']).next() {
                if !name.is_empty() {
                    mods.insert(name.to_string());
                }
            }
        }
    }
    mods
}

/// Hole 1: every `#[ignore]` in the suite carries a written reason.
#[test]
fn every_ignore_carries_a_written_reason() {
    let src = tests_src_dir();
    let mut files = Vec::new();
    collect_rs_files(&src, &mut files);
    assert!(
        !files.is_empty(),
        "no .rs files found under {}",
        src.display()
    );

    let mut offenders = Vec::new();
    for file in &files {
        let content = std::fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("read {}: {e}", file.display()));
        for line in reasonless_ignore_lines(&content) {
            offenders.push(format!("{}:{line}", file.display()));
        }
    }

    assert!(
        offenders.is_empty(),
        "CI-coverage-completeness: {} test(s) are skipped with no written reason.\n\
         Every `#[ignore]` must be `#[ignore = \"<reason>\"]` (the reason in the attribute, \
         not a trailing comment) — that attribute is the single authority for 'excused from \
         the rust gate'.\nOffenders:\n  {}",
        offenders.len(),
        offenders.join("\n  ")
    );
}

/// Hole 2: every test `*.rs` is wired into `lib.rs` as a module, so no test file
/// silently exists-but-never-runs.
#[test]
fn every_test_file_is_declared_in_lib() {
    let src = tests_src_dir();
    let lib_rs = std::fs::read_to_string(src.join("lib.rs")).expect("read lib.rs");
    let declared = declared_modules(&lib_rs);

    let mut files = Vec::new();
    collect_rs_files(&src, &mut files);

    let mut undeclared = Vec::new();
    for file in &files {
        let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if stem == "lib" {
            continue;
        }
        if !declared.contains(stem) {
            undeclared.push(file.display().to_string());
        }
    }

    assert!(
        undeclared.is_empty(),
        "CI-coverage-completeness: {} test file(s) are not declared `mod` in lib.rs, so they \
         never compile or run.\nUndeclared:\n  {}",
        undeclared.len(),
        undeclared.join("\n  ")
    );
}

// ── Discriminating controls (the lens has teeth) ─────────────────────────────
// Each control feeds a fixture through the SAME `reasonless_ignore_lines`
// authority used on the live suite — not a re-implementation — so green-on-good
// and red-on-bad are both proven on the real detector (not a tautological grep).

#[test]
fn detector_flags_a_reasonless_ignore() {
    // bare attribute → flagged
    assert_eq!(
        reasonless_ignore_lines("#[test]\n#[ignore]\nfn x() {}"),
        vec![2]
    );
    // reason in a trailing comment, not the attribute → still flagged
    assert_eq!(
        reasonless_ignore_lines("    #[ignore] // expensive: 30s\n    fn y() {}"),
        vec![1]
    );
    // empty reason string → flagged
    assert_eq!(
        reasonless_ignore_lines("#[ignore = \"\"]\nfn z() {}"),
        vec![1]
    );
}

#[test]
fn detector_accepts_a_reasoned_ignore() {
    assert!(
        reasonless_ignore_lines("#[ignore = \"wet-only: live network\"]\nfn x() {}").is_empty()
    );
    // a prose mention of the token (comment / string) is not an attribute
    assert!(reasonless_ignore_lines("//! Most are `#[ignore]` because they are slow.").is_empty());
    // a normal, non-ignored test is covered by construction — nothing to flag
    assert!(reasonless_ignore_lines("#[test]\nfn runs() {}").is_empty());
}

#[test]
fn detector_accepts_a_multiline_reason() {
    // A long reason wrapped with `\` line-continuation across physical lines is
    // still a written reason — the opening line carries non-empty content even
    // though the closing quote lands on a later line. (Regression guard: an
    // earlier detector required the closing `"` on the same line and false-flagged
    // exactly this shape — the `fixtures/v2` ignore in pipeline.rs.)
    let src = "#[ignore = \"failing: a genuinely long reason that explains the \\\n  drift and names the owner, wrapped across lines for readability\"]\nfn x() {}";
    assert!(reasonless_ignore_lines(src).is_empty());
    // ...but a multi-line opener with NO content before the break is still empty → flagged.
    assert_eq!(
        reasonless_ignore_lines("#[ignore = \"\\\n  \"]\nfn y() {}"),
        vec![1]
    );
}

#[test]
fn module_detector_reads_declarations() {
    let lib = "pub mod helpers;\n#[cfg(test)]\nmod bootstrap;\nmod pipeline;\n";
    let mods = declared_modules(lib);
    assert!(mods.contains("helpers"));
    assert!(mods.contains("bootstrap"));
    assert!(mods.contains("pipeline"));
    assert!(!mods.contains("missing"));
}
