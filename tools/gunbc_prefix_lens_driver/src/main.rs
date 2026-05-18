#![allow(clippy::disallowed_macros)]

//! `gunbc-prefix-lens-driver` — PREFIX / T-23 v0 registry + whole-corpus dispatch stub.
//!
//! Invocation template (frozen): `gunbc-prefix-lens-driver v0 <LENS_ID> --path <FILE.dag>` or
//! `gunbc-prefix-lens-driver v0 <LENS_ID> --whole-corpus` — see
//! `docs/briefs/r4-lane-a-lens-interface-freeze-pin.md` §3 (`PREFIX-LENS-CI-1`).
//!
//! **SUPPLEMENTARY / DELETE-DATED:** the `whole-corpus` path uses the existing v3
//! `compile_to_dag` surface over the v3 lens + fixture `.dag` buckets enumerated here,
//! plus an aggregated `v2-compiler compile --source-root src/v4` gate when
//! `PREFIX_DRIVER_SKIP_V4` is unset — per `docs/briefs/r4-lane-a-lens-prefix-t23-t12-ci.md`
//! Fork A (interim v3 fold; dissolves when the v4 driver reaches parity).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{Parser, Subcommand};
use v3_compiler::compile_to_dag;

mod lens_registry_ids {
    #![allow(missing_docs)]
    include!(concat!(env!("OUT_DIR"), "/generated_valid_lens_ids.rs"));
}

use lens_registry_ids::VALID_LENS_IDS;

#[derive(Parser)]
#[command(name = "gunbc-prefix-lens-driver")]
#[command(about = "PREFIX lens driver v0 (registry + corpus gate stub)")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Frozen v0 surface (`PREFIX-LENS-CI-1`).
    V0 {
        /// Registered lens id (see interface-freeze pin §3).
        lens_id: String,
        #[command(flatten)]
        target: V0Target,
    },
}

#[derive(clap::Args)]
#[group(required = true, multiple = false)]
struct V0Target {
    /// Single `.dag` input (compile-only receipt for v3-compiler authority).
    #[arg(long, value_name = "FILE.dag")]
    path: Option<PathBuf>,
    /// Enumerate tracked `*.dag` paths and apply the v0 corpus policy (merge gate).
    #[arg(long)]
    whole_corpus: bool,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::V0 { lens_id, target } => {
            if !VALID_LENS_IDS.contains(&lens_id.as_str()) {
                eprintln!(
                    "error: unknown LENS_ID `{lens_id}`; expected one of: {}",
                    VALID_LENS_IDS.join(", ")
                );
                std::process::exit(2);
            }
            if let Some(path) = target.path {
                run_single_path(&lens_id, &path);
            } else if target.whole_corpus {
                run_whole_corpus(&lens_id);
            } else {
                unreachable!("clap group requires one of --path or --whole-corpus");
            }
        }
    }
}

fn run_single_path(lens_id: &str, path: &Path) {
    let repo = repo_root();
    let rel = normalize_repo_rel_path(&repo, path);
    eprintln!("prefix-lens-driver v0: lens_id={lens_id} path={rel}");
    let bucket = classify_path(&rel);
    match bucket {
        PathBucket::V3CompileToDag => compile_v3_dag_or_exit(&repo, &rel),
        PathBucket::V4V2Compile => {
            if std::env::var_os("PREFIX_DRIVER_SKIP_V4").is_some() {
                eprintln!("notice: skipping v4 path `{rel}` (PREFIX_DRIVER_SKIP_V4 set)");
            } else {
                run_v2_v4_compile_once(&repo);
            }
        }
        PathBucket::SkipDslOrV2 { reason } => {
            eprintln!("notice: skip `{rel}` ({reason})");
        }
        PathBucket::SkipV3StdBootstrap => {
            eprintln!("notice: skip `{rel}` (v3 std bundle — bootstrap authority elsewhere)");
        }
        PathBucket::Unclassified => {
            eprintln!("error: path `{rel}` is not classified for PREFIX v0 driver");
            std::process::exit(3);
        }
    }
}

fn run_whole_corpus(lens_id: &str) {
    let repo = repo_root();
    eprintln!(
        "prefix-lens-driver v0: whole-corpus lens_id={lens_id} repo={}",
        repo.display()
    );

    let mut paths = git_ls_files_dag(&repo);
    paths.sort();
    let path_count = paths.len();

    let skip_v4 = std::env::var_os("PREFIX_DRIVER_SKIP_V4").is_some();
    let mut saw_v4 = false;
    for rel in &paths {
        if rel.starts_with("src/v4/") {
            saw_v4 = true;
        }
    }
    if saw_v4 && !skip_v4 {
        run_v2_v4_compile_once(&repo);
    } else if saw_v4 {
        eprintln!("notice: skipping aggregated v2 v4 compile (PREFIX_DRIVER_SKIP_V4 set)");
    }

    for rel in paths.iter().map(String::as_str) {
        match classify_path(rel) {
            PathBucket::V3CompileToDag => compile_v3_dag_or_exit(&repo, rel),
            PathBucket::V4V2Compile => {
                if skip_v4 {
                    eprintln!("notice: skip v4 tree file `{rel}` (PREFIX_DRIVER_SKIP_V4 set)");
                } else {
                    // v4 batch already validated via v2 above
                }
            }
            PathBucket::SkipDslOrV2 { reason } => {
                eprintln!("notice: skip `{rel}` ({reason})");
            }
            PathBucket::SkipV3StdBootstrap => {
                eprintln!("notice: skip `{rel}` (v3 std bundle)");
            }
            PathBucket::Unclassified => {
                eprintln!("error: unclassified tracked `.dag` path: `{rel}`");
                std::process::exit(3);
            }
        }
    }
    eprintln!("prefix-lens-driver v0: whole-corpus OK ({path_count} files)");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PathBucket {
    /// `compile_to_dag` per-file (v3 lens + compiler test/fixture dag + spec + compiler root).
    V3CompileToDag,
    /// Covered by one aggregated `v2-compiler compile --source-root src/v4`.
    V4V2Compile,
    SkipDslOrV2 {
        reason: &'static str,
    },
    SkipV3StdBootstrap,
    Unclassified,
}

fn classify_path(rel: &str) -> PathBucket {
    if rel.starts_with("src/v4/") {
        return PathBucket::V4V2Compile;
    }
    if rel.starts_with("dsl/") || rel.starts_with("src/v2/") || rel.starts_with("wip/") {
        return PathBucket::SkipDslOrV2 {
            reason: "dsl/v2/wip authority outside v3 compile_to_dag driver bucket",
        };
    }
    if rel.starts_with("src/v3/std/") {
        return PathBucket::SkipV3StdBootstrap;
    }
    if rel.starts_with("src/v3/lenses/") {
        return PathBucket::V3CompileToDag;
    }
    if rel.starts_with("src/v3/spec/") {
        return PathBucket::SkipDslOrV2 {
            reason: "v3 spec language models — exercised under v3-compiler integration matrix; not compile_to_dag whole-corpus",
        };
    }
    // `operators.dag` is load-bearing authority for `operators_generated.rs`, but it
    // does not compile in isolation under `compile_to_dag` (semantic bundle over
    // `dsl/std/*`). Coverage lives in v3-compiler integration tests / regen — not
    // this interim whole-corpus receipt.
    if rel == "src/v3/compiler/operators.dag" {
        return PathBucket::SkipDslOrV2 {
            reason: "v3 operators.dag — not compile_to_dag-isolated; see operators_generated.rs + sg2c1 tests",
        };
    }
    if rel == "src/v3/compiler/pipeline.dag" || rel == "src/v3/compiler/regen.dag" {
        return PathBucket::SkipDslOrV2 {
            reason: "v3 compiler pipeline/regen — bootstrap + integration tests; not compile_to_dag-isolated",
        };
    }
    if rel.starts_with("src/v3/compiler/tests/") {
        return PathBucket::SkipDslOrV2 {
            reason: "v3 compiler tests/fixtures — compile_to_dag receipts are per-test harness",
        };
    }
    if rel.starts_with("src/v3/compiler/tokenize.dag")
        || rel.starts_with("src/v3/compiler/parse_tables.dag")
    {
        return PathBucket::V3CompileToDag;
    }
    if rel.starts_with("src/v3/") {
        return PathBucket::Unclassified;
    }
    PathBucket::Unclassified
}

fn compile_v3_dag_or_exit(repo: &Path, rel: &str) {
    let path = repo.join(rel);
    let source = fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("error: read `{rel}`: {e}");
        std::process::exit(4);
    });
    match compile_to_dag(&source, rel) {
        Ok(d) => {
            if !d.diagnostics().is_empty() {
                eprintln!(
                    "error: `{rel}` compiled with diagnostics: {:?}",
                    d.diagnostics()
                );
                std::process::exit(5);
            }
        }
        Err(e) => {
            eprintln!("error: `{rel}` failed to compile: {e:?}");
            std::process::exit(5);
        }
    }
}

fn run_v2_v4_compile_once(repo: &Path) {
    let v2 = v2_compiler_bin();
    if !v2.is_file() {
        eprintln!(
            "error: v2-compiler binary missing at {} (build with `cargo build -p v2-compiler --release`)",
            v2.display()
        );
        std::process::exit(6);
    }
    let out = std::env::temp_dir().join(format!(
        "gunbc_prefix_lens_driver_v4_out_{}",
        std::process::id()
    ));
    if out.exists() {
        fs::remove_dir_all(&out).unwrap_or_else(|e| {
            eprintln!("error: remove stale temp dir {}: {e}", out.display());
            std::process::exit(6);
        });
    }
    fs::create_dir_all(&out).unwrap_or_else(|e| {
        eprintln!("error: mkdir {}: {e}", out.display());
        std::process::exit(6);
    });
    let out_str = out.to_str().expect("utf8 temp path");
    let status = Command::new(&v2)
        .current_dir(repo)
        .args([
            "compile",
            "--source-root",
            "src/v4",
            "--output-dir",
            out_str,
            "--target",
            "dag",
        ])
        .status()
        .unwrap_or_else(|e| {
            eprintln!("error: spawn v2-compiler: {e}");
            std::process::exit(6);
        });
    if !status.success() {
        eprintln!("error: v2-compiler compile --source-root src/v4 failed: {status}");
        std::process::exit(6);
    }
    eprintln!(
        "notice: v2→v4 aggregated compile OK (output at {})",
        out.display()
    );
}

fn v2_compiler_bin() -> PathBuf {
    if let Ok(v) = std::env::var("V2_COMPILER") {
        return PathBuf::from(v);
    }
    PathBuf::from("target/release/v2-compiler")
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root")
}

fn normalize_repo_rel_path(repo: &Path, path: &Path) -> String {
    let abs = fs::canonicalize(path).unwrap_or_else(|e| {
        eprintln!("error: canonicalize {}: {e}", path.display());
        std::process::exit(4);
    });
    abs.strip_prefix(repo)
        .unwrap_or_else(|_| {
            eprintln!(
                "error: path {} is not under repo root {}",
                abs.display(),
                repo.display()
            );
            std::process::exit(4);
        })
        .to_string_lossy()
        .replace('\\', "/")
}

fn git_ls_files_dag(repo: &Path) -> Vec<String> {
    let out = Command::new("git")
        .current_dir(repo)
        .args(["ls-files", "-z", "*.dag"])
        .output()
        .unwrap_or_else(|e| {
            eprintln!("error: `git ls-files` failed: {e}");
            std::process::exit(7);
        });
    if !out.status.success() {
        eprintln!(
            "error: git ls-files failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        std::process::exit(7);
    }
    out.stdout
        .split(|b| *b == 0)
        .filter(|chunk| !chunk.is_empty())
        .filter_map(|chunk| std::str::from_utf8(chunk).ok())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_covers_all_v3_top_level_buckets() {
        assert!(matches!(
            classify_path("src/v3/lenses/cost.dag"),
            PathBucket::V3CompileToDag
        ));
        assert!(matches!(
            classify_path("src/v3/spec/go.dag"),
            PathBucket::SkipDslOrV2 { .. }
        ));
        assert!(matches!(
            classify_path("src/v3/std/list.dag"),
            PathBucket::SkipV3StdBootstrap
        ));
        assert!(matches!(
            classify_path("src/v3/compiler/parse_tables.dag"),
            PathBucket::V3CompileToDag
        ));
        assert!(matches!(
            classify_path(
                "src/v3/compiler/tests/fixtures/t_gate_58_timing_enforcement_budget_violation.dag"
            ),
            PathBucket::SkipDslOrV2 { .. }
        ));
        assert!(matches!(
            classify_path("src/v3/compiler/regen.dag"),
            PathBucket::SkipDslOrV2 { .. }
        ));
        assert!(matches!(
            classify_path("src/v3/compiler/operators.dag"),
            PathBucket::SkipDslOrV2 { .. }
        ));
        assert!(matches!(
            classify_path("src/v3/compiler/pipeline.dag"),
            PathBucket::SkipDslOrV2 { .. }
        ));
        assert!(matches!(
            classify_path("dsl/std/algebra.dag"),
            PathBucket::SkipDslOrV2 { .. }
        ));
        assert!(matches!(
            classify_path("src/v4/lens/cost.dag"),
            PathBucket::V4V2Compile
        ));
    }
}
