//! **Layer:** integration
//!
//! get-off-v3 — by-EXECUTION caller census + ratchet for
//! `v3_compiler::compile_to_dag`.
//!
//! ## Why this exists
//!
//! `compile_to_dag` (defined at `src/v3/compiler/src/lib.rs`) is the v3
//! whole-source compile entry point. The self-hosting thesis shrinks the
//! hand-authored v3 Rust surface toward zero (`src/v3/SELF_HOSTING.md`),
//! and every direct call to `compile_to_dag` is one more site that has to
//! migrate before v3 can be retired. To drive that set down you first have
//! to *see* it — and see it honestly.
//!
//! ## By execution, not grep-and-pin
//!
//! INVARIANTS.md E-10 is explicit: **"Done" means a consumer running green
//! by execution — never typecheck-plus-grep. `.contains()` greps are not
//! consumers; they pass whether or not the code runs.** A census that
//! checked in a hand-counted number per caller file (the
//! `grep + hand-count-pin` shape retired by #4633 for glob discovery) is a
//! parallel ledger that drifts: the pinned number and the real source go
//! out of sync silently, and nobody notices until an audit.
//!
//! This census instead **discovers the caller set by execution**: at test
//! time it walks the live source tree, parses each `.rs` file, and counts
//! the *direct* `compile_to_dag(` call sites it actually finds. The number
//! is whatever the scan returns — there is no per-file pinned count to
//! maintain. The only checked-in number is a single **ceiling**
//! ([`COMPILE_TO_DAG_CALLER_CEILING`]) that the discovered total must stay
//! at or below. The ceiling is a high-water mark, not a mirror of the
//! count: it ratchets **down only** as callers are migrated off v3, exactly
//! like the numeric `Arbitrary` / wall-clock ratchets in this repo
//! (`src/v3/SELF_HOSTING.md` — "numeric ratchet monotonically decreasing
//! toward zero").
//!
//! ## The consumer
//!
//! [`get_off_v3_compile_to_dag_caller_count_is_at_or_below_ceiling`] is a
//! real consumer in E-10's sense: when a new direct `compile_to_dag`
//! caller lands, **executing this test breaks**. It also emits the full
//! per-file census as an artifact (printed, and written to
//! `target/get-off-v3/compile_to_dag_caller_census.tsv`) so the tracking
//! surface is visible — run with `--nocapture` to read it.
//!
//! ## Where this runs
//!
//! This is a **CI gate** as of PR #4659: the test is wired into gate-3
//! (`scripts/v4-affected-tests-gate.sh` via `run_ci_pipeline`). A 477th
//! direct `compile_to_dag(` caller
//! will fail CI. It also runs locally via `cargo test -p v3-compiler` and
//! in review/dev runs, and emits the full per-file census as an artifact
//! (printed under `--nocapture`, written to
//! `target/get-off-v3/compile_to_dag_caller_census.tsv`).
//!
//! ## What counts as a caller
//!
//! A *direct* call to the `compile_to_dag` free function. The scanner
//! deliberately does **not** count:
//!   - `cached_compile_to_dag(` — the in-binary memoizing test helper
//!     (`tests/integration/common/cached_compile.rs`); routing a test onto
//!     it is itself part of getting off the direct call.
//!   - `compile_to_dag_modules_in_order(` — the multi-module variant
//!     (a distinct identifier).
//!   - the `fn compile_to_dag(` definition itself.
//! Matching mirrors the line-oriented regex
//! `(?<![A-Za-z0-9_])(?<!fn )compile_to_dag\s*\(` so the count is
//! reproducible against `grep -P` from a shell.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// High-water ceiling on the total number of direct `compile_to_dag`
/// call sites across the scanned roots. **Ratchet down only.**
///
/// When you migrate callers off `compile_to_dag`, re-run this test, read
/// the new total it reports, and lower this constant to match in the same
/// PR. Never raise it: a new direct caller must either be removed before
/// merge or land with explicit get-off-v3 sign-off (and a paired raise
/// documented here). The north star is `0` — at which point `v3` has no
/// direct whole-source compile callers left.
const COMPILE_TO_DAG_CALLER_CEILING: usize = 375;

/// The identifier whose direct call sites this census tracks.
const TARGET_IDENT: &str = "compile_to_dag";

/// Workspace-relative roots scanned for callers. `src` covers the entire
/// `v3` surface (and proves there are no hidden callers in sibling
/// versions, which contribute 0); `tools` catches the CI/ratchet helpers.
const SCAN_ROOTS: &[&str] = &["src", "tools"];

/// This census file is the discovery *instrument*, not a caller. Its own
/// source carries literal `compile_to_dag(` text (the scanner unit-test
/// cases and doc comments) that the scanner would otherwise count against
/// the budget. Exclude it to keep discovery acyclic — the same posture
/// `discover_owned_data` takes toward its own manifest/consumers (#4633).
const SELF_EXCLUDE_REL: &str =
    "src/v3/compiler/tests/integration/get_off_v3_compile_to_dag_census_test.rs";

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Count direct `compile_to_dag(` call sites in one file's text.
///
/// Reproduces, line by line, the matches of
/// `(?<![A-Za-z0-9_])(?<!fn )compile_to_dag\s*\(`:
///   - the identifier `compile_to_dag` appears,
///   - the byte before it is not a word byte (excludes `cached_…`),
///   - the identifier is not immediately preceded by `"fn "` (excludes the
///     definition),
///   - after the identifier, skipping only spaces/tabs (line-oriented, so
///     no newline crossing — matching `grep`), the next byte is `(`.
/// The trailing `(` check also excludes `compile_to_dag_modules_in_order(`,
/// whose next byte is `_` rather than `(` or whitespace.
fn count_direct_callers_in_text(text: &str) -> usize {
    let bytes = text.as_bytes();
    let needle = TARGET_IDENT.as_bytes();
    let mut count = 0usize;
    let mut i = 0usize;
    while i + needle.len() <= bytes.len() {
        if &bytes[i..i + needle.len()] != needle {
            i += 1;
            continue;
        }
        // Preceding byte must not be a word byte (so `cached_compile_to_dag`
        // — preceded by `_` — does not match).
        let preceded_by_word = i > 0 && is_word_byte(bytes[i - 1]);
        // Not the `fn compile_to_dag` definition.
        let is_definition = i >= 3 && &bytes[i - 3..i] == b"fn ";
        if preceded_by_word || is_definition {
            i += 1;
            continue;
        }
        // After the identifier, skip spaces/tabs only (line-oriented) and
        // require an opening paren.
        let mut j = i + needle.len();
        while j < bytes.len() && (bytes[j] == b' ' || bytes[j] == b'\t') {
            j += 1;
        }
        if j < bytes.len() && bytes[j] == b'(' {
            count += 1;
        }
        i += needle.len();
    }
    count
}

/// Recursively collect `.rs` files under `dir`.
fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            collect_rs_files(&path, out);
        } else if file_type.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// A path is on the **test surface** iff a `tests` directory appears in it
/// (`tests/integration/…`, `tests/boundary/…`, `determinism_test.rs` at the
/// `tests/` root). Everything else — production `src/…` modules and `tools`
/// helpers — is the non-test surface. The brief notes callers are
/// concentrated on the test surface; the census reports both partitions but
/// ratchets the **total**, keeping a single source of truth for the budget.
fn is_test_surface(rel: &str) -> bool {
    rel.split('/').any(|seg| seg == "tests")
}

struct Census {
    /// Workspace-relative path -> direct caller count (only files with ≥1).
    per_file: BTreeMap<String, usize>,
    total: usize,
    test_surface_total: usize,
    non_test_total: usize,
}

fn discover_census() -> Census {
    let root = workspace_root();
    let mut files = Vec::new();
    for scan_root in SCAN_ROOTS {
        collect_rs_files(&root.join(scan_root), &mut files);
    }

    let mut per_file = BTreeMap::new();
    let mut total = 0usize;
    let mut test_surface_total = 0usize;
    let mut non_test_total = 0usize;

    for path in files {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let rel = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if rel == SELF_EXCLUDE_REL {
            continue;
        }
        let n = count_direct_callers_in_text(&text);
        if n == 0 {
            continue;
        }
        total += n;
        if is_test_surface(&rel) {
            test_surface_total += n;
        } else {
            non_test_total += n;
        }
        per_file.insert(rel, n);
    }

    Census {
        per_file,
        total,
        test_surface_total,
        non_test_total,
    }
}

/// Render the census as TSV. First line is a `#`-prefixed summary; each
/// remaining line is `count\tworkspace_relative_path`, sorted by descending
/// count then path so the heaviest callers sit at the top.
fn render_tsv(census: &Census) -> String {
    let mut rows: Vec<(&String, &usize)> = census.per_file.iter().collect();
    rows.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
    let mut out = format!(
        "# get-off-v3 compile_to_dag caller census (by execution)\n\
         # total={}\tceiling={}\ttest_surface={}\tnon_test={}\tcaller_files={}\n",
        census.total,
        COMPILE_TO_DAG_CALLER_CEILING,
        census.test_surface_total,
        census.non_test_total,
        census.per_file.len(),
    );
    for (path, count) in rows {
        out.push_str(&format!("{count}\t{path}\n"));
    }
    out
}

/// Emit the census artifact to `target/get-off-v3/…` and return its path.
/// Writing it is itself a (weak) consumer: a failure here fails the test.
fn emit_artifact(tsv: &str) -> PathBuf {
    let dir = workspace_root().join("target").join("get-off-v3");
    fs::create_dir_all(&dir)
        .unwrap_or_else(|e| panic!("create census artifact dir {}: {e}", dir.display()));
    let path = dir.join("compile_to_dag_caller_census.tsv");
    fs::write(&path, tsv)
        .unwrap_or_else(|e| panic!("write census artifact {}: {e}", path.display()));
    path
}

#[test]
fn get_off_v3_compile_to_dag_caller_count_is_at_or_below_ceiling() {
    let census = discover_census();
    let tsv = render_tsv(&census);
    let artifact = emit_artifact(&tsv);

    // Print the artifact so the tracking surface is visible under
    // `--nocapture`. This is a by-execution view, not a checked-in ledger.
    println!(
        "get-off-v3 compile_to_dag caller census (artifact: {})\n{}",
        artifact.display(),
        tsv,
    );

    // Self-check: the partition totals reconcile with the per-file sum, so
    // the discovery scan genuinely executed over every counted file.
    let summed: usize = census.per_file.values().copied().sum();
    assert_eq!(
        summed, census.total,
        "census internal inconsistency: per-file sum {summed} != reported total {}",
        census.total,
    );
    assert_eq!(
        census.test_surface_total + census.non_test_total,
        census.total,
        "partition totals must reconcile with the grand total",
    );

    // The ratchet. Ratchet DOWN only.
    assert!(
        census.total <= COMPILE_TO_DAG_CALLER_CEILING,
        "get-off-v3 ratchet breach: discovered {} direct `compile_to_dag` call site(s) \
         ({} on the test surface, {} non-test), but the ceiling is {}.\n\
         A new direct caller of `v3_compiler::compile_to_dag` landed. Every direct call \
         is a site that must migrate before v3 can be retired (src/v3/SELF_HOSTING.md). \
         Either remove the new call before merge (prefer routing tests onto \
         `cached_compile_to_dag`, or onto the .dag substrate), or — if the caller is \
         genuinely irreducible right now — raise this ceiling in the same PR with \
         get-off-v3 sign-off and a note here. The artifact at {} lists every caller.",
        census.total,
        census.test_surface_total,
        census.non_test_total,
        COMPILE_TO_DAG_CALLER_CEILING,
        artifact.display(),
    );

    // Slack guard: if the discovered total has dropped strictly below the
    // ceiling, the ceiling is now slack and should be tightened to lock in
    // the win. This fails loudly (with the exact number to write) rather
    // than letting reductions silently re-open headroom for new callers —
    // the ratchet only bites if it tracks the real high-water mark.
    assert_eq!(
        census.total, COMPILE_TO_DAG_CALLER_CEILING,
        "get-off-v3 progress! Direct `compile_to_dag` callers dropped to {}, below the \
         ceiling {}. Lower `COMPILE_TO_DAG_CALLER_CEILING` to {} in this PR to lock in the \
         reduction (the ratchet must track the real high-water mark to keep biting).",
        census.total, COMPILE_TO_DAG_CALLER_CEILING, census.total,
    );
}

#[test]
fn get_off_v3_census_scanner_matches_known_shapes() {
    // Unit-level guard on the scanner semantics so the census number is
    // trustworthy: the discovery is only as honest as what it counts.
    let cases: &[(&str, usize)] = &[
        ("compile_to_dag(src, file)", 1),
        ("v3_compiler::compile_to_dag(src, file)", 1),
        ("let d = compile_to_dag (src, file);", 1), // space before paren
        ("cached_compile_to_dag(src, file)", 0),    // wrapper helper
        ("compile_to_dag_modules_in_order(mods)", 0), // distinct variant
        ("pub fn compile_to_dag(source: &str) {}", 0), // the definition
        ("fn compile_to_dag(s: &str)", 0),          // definition (no `pub`)
        ("// see compile_to_dag for details", 0),   // mention, no call paren
        ("a_compile_to_dag(x)", 0),                 // word-prefixed identifier
        (
            "compile_to_dag(a);\n  compile_to_dag(b);\n  cached_compile_to_dag(c);",
            2,
        ),
    ];
    for (text, expected) in cases {
        assert_eq!(
            count_direct_callers_in_text(text),
            *expected,
            "scanner mismatch on {text:?}",
        );
    }
}
