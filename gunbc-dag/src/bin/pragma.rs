//! gunbc-pragma: Generate repo pragma artifacts (clippy.toml + allowlists).

#![forbid(dead_code)]

use gunbc_codegen::FileWriter;
use gunbc_dag::policy::pragma::{
    clippy_renderer, render_disallowed_methods_allowlist, render_pragma_lint_policy,
    PRAGMA_REGENERATE_CMD,
};
use gunbc_ir::Renderable;
use std::env;
use std::path::Path;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut dry_run = false;
    let mut check = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--dry-run" => dry_run = true,
            "-c" | "--check" => check = true,
            "-h" | "--help" => {
                print_help();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    if check {
        dry_run = true;
    }

    println!("pragma");
    println!(
        "  mode: {}",
        if check {
            "check"
        } else if dry_run {
            "dry-run"
        } else {
            "real"
        }
    );
    println!();

    let writer = FileWriter::new(dry_run);

    let outputs = [
        ("clippy.toml", Path::new("clippy.toml"), clippy_renderer().render()),
        (
            "disallowed-methods-allowlist",
            Path::new("tools/disallowed-methods-allowlist.txt"),
            render_disallowed_methods_allowlist(),
        ),
        (
            "pragma-lint-policy",
            Path::new("tools/pragma-lint-policy.txt"),
            render_pragma_lint_policy(),
        ),
    ];

    let mut any_changed = false;
    for (label, path, content) in outputs {
        match writer.write(path, content) {
            Ok(result) => {
                any_changed |= result.changed;
                let status = if result.written {
                    if result.changed {
                        "written"
                    } else {
                        "unchanged"
                    }
                } else if result.changed {
                    "would-change"
                } else {
                    "unchanged"
                };
                println!("  [{}] {} ({})", label, path.display(), status);
            }
            Err(e) => {
                eprintln!("  [{}] {} ERROR: {}", label, path.display(), e);
                process::exit(1);
            }
        }
    }

    if check && any_changed {
        eprintln!();
        eprintln!("ERROR: pragma artifacts are out of date.");
        eprintln!("Run: {}", PRAGMA_REGENERATE_CMD);
        process::exit(1);
    }

    println!();
    println!("Generated: pragma artifacts");
}

fn print_help() {
    println!("pragma - Generate clippy.toml and pragma allowlists");
    println!();
    println!("USAGE:");
    println!("    gunbc-pragma [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run        Show what would be generated");
    println!("    -c, --check          Fail if generated files are stale");
    println!("    -h, --help           Print this help");
    println!();
    println!("Regenerate command:");
    println!("    {}", PRAGMA_REGENERATE_CMD);
}
