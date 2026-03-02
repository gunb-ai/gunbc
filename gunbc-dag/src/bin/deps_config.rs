//! gunbc-deps-config main entry point.
//!
//! Ensures or verifies `deps.toml` against the canonical rendering derived
//! from the tool registry.

#![deny(dead_code)]

use std::process;

use gunbc_cli::{parse, CliParam, ParamType};
use gunbc_deps::{generate_deps_toml_from_registry, DEFAULT_MANIFEST_FILENAME};
use gunbc_ir::resource::ExecMode;
use gunbc_ir::Value;

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let parsed = match parse(
        &argv,
        &[CliParam::new("mode", ParamType::Str).default("ensure")],
    ) {
        Ok(parsed) => parsed,
        Err(error) => {
            eprintln!("error: failed to parse arguments: {error}");
            process::exit(1);
        }
    };

    if parsed.help {
        print_help();
        return;
    }

    let mode_raw = parsed
        .values
        .get("mode")
        .and_then(Value::as_str)
        .unwrap_or("ensure");
    let mode = ExecMode::parse_strict(mode_raw).unwrap_or_else(|_| {
        eprintln!("error: invalid --mode value '{mode_raw}', expected ensure|verify");
        process::exit(1);
    });

    let desired = generate_deps_toml_from_registry();
    let existing = match read_existing(DEFAULT_MANIFEST_FILENAME) {
        Ok(existing) => existing,
        Err(error) => {
            eprintln!(
                "error: failed to read {}: {error}",
                DEFAULT_MANIFEST_FILENAME
            );
            process::exit(1);
        }
    };
    let is_fresh = existing.as_deref() == Some(desired.as_str());

    match mode {
        ExecMode::Verify => {
            if is_fresh {
                println!("ok: {} is up to date", DEFAULT_MANIFEST_FILENAME);
                return;
            }
            eprintln!(
                "error: {} is stale or missing (run with --mode=ensure)",
                DEFAULT_MANIFEST_FILENAME
            );
            process::exit(1);
        }
        ExecMode::Ensure => {
            if is_fresh {
                println!("ok: {} is up to date", DEFAULT_MANIFEST_FILENAME);
                return;
            }
            if parsed.dry_run {
                println!(
                    "dry-run: would update {} from canonical tool registry rendering",
                    DEFAULT_MANIFEST_FILENAME
                );
                return;
            }
            if let Err(error) = write_manifest(DEFAULT_MANIFEST_FILENAME, &desired) {
                eprintln!(
                    "error: failed to write {}: {error}",
                    DEFAULT_MANIFEST_FILENAME
                );
                process::exit(1);
            }
            println!("updated: {}", DEFAULT_MANIFEST_FILENAME);
        }
    }
}

#[allow(clippy::disallowed_methods)] // Bootstrap/config binary owns deps.toml.
fn read_existing(path: &str) -> Result<Option<String>, std::io::Error> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

#[allow(clippy::disallowed_methods)] // Bootstrap/config binary owns deps.toml.
fn write_manifest(path: &str, content: &str) -> Result<(), std::io::Error> {
    std::fs::write(path, content)
}

fn print_help() {
    println!("gunbc-deps-config - ensure or verify deps.toml");
    println!();
    println!("USAGE:");
    println!("    gunbc-deps-config [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --mode=MODE      ensure (default) or verify");
    println!("    -n, --dry-run    Print intended action without writing");
    println!("    -h, --help       Print this help");
}
