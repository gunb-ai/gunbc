//! gunbc-testgen main entry point.
//!
//! Generates test files from DAG structures and MockSpecs.
//!
//! Usage:
//!     cargo run -p gunbc-dag --bin gunbc-testgen
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --dry-run
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --check
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --output-dir /path/to/output

#![forbid(dead_code)]
use gunbc_codegen::FileWriter;
use gunbc_dag::testgen_resource_def;
use gunbc_ir::resource::{
    update_resource_manifest, ManagedResource, ManifestEntry, ManifestUpdateError, ResourceDef,
    ResourceError, ResourceManifest,
};
// Force-link crates that register testgen targets.
use gunbc_deps as _;
use gunbc_gist as _;
use gunbc_lib_llm_ops as _;
use gunbc_lib_review as _;
use gunbc_testgen_registry::{iter_targets, TestgenTarget};
use std::env;
use std::path::PathBuf;
use std::process;

/// Execution mode for testgen.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Generate and write files
    Generate,
    /// Check if generated files are stale (CI mode)
    Check,
    /// Show what would be generated without writing
    DryRun,
}

/// Build all testgen targets from the auto-discovery registry.
fn build_targets() -> Vec<&'static TestgenTarget> {
    let mut targets: Vec<&'static TestgenTarget> = iter_targets().collect();
    targets.sort_by(|a, b| a.name.cmp(b.name));
    targets
}

// Code generator — needs direct filesystem access (same exemption as gunbc-codegen).
#[allow(clippy::disallowed_methods)]
fn main() {
    let args: Vec<String> = env::args().collect();

    let mut output_dir = PathBuf::from(".");
    let mut mode = Mode::Generate;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output-dir" => {
                i += 1;
                if i < args.len() {
                    output_dir = PathBuf::from(&args[i]);
                }
            }
            "-n" | "--dry-run" => mode = Mode::DryRun,
            "-c" | "--check" => mode = Mode::Check,
            "-h" | "--help" => {
                print_help();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    let mode_str = match mode {
        Mode::Generate => "generate",
        Mode::Check => "check",
        Mode::DryRun => "dry-run",
    };

    println!("testgen");
    println!("  output_dir: {}", output_dir.display());
    println!("  mode: {}", mode_str);
    println!();

    let targets = build_targets();
    if targets.is_empty() {
        eprintln!("No testgen targets registered.");
        process::exit(1);
    }

    // Check mode is read-only (like dry-run) — we only compare, never write.
    let writer = FileWriter::new(mode != Mode::Generate);

    let mut ok = 0;
    let mut stale = 0;
    let mut errors = 0;
    let mut stale_files = Vec::new();

    for target in &targets {
        let config = target.to_def();
        let code = (target.generate)(&config);
        let output_path = output_dir.join(&config.output_path);

        match writer.write(&output_path, &code) {
            Ok(result) => {
                if mode == Mode::Check {
                    // Check mode: report stale/ok without writing
                    if result.changed {
                        println!("[{}] ✗ STALE - needs regeneration", config.name);
                        stale += 1;
                        stale_files.push(config.output_path.clone());
                    } else {
                        println!("[{}] ✓ up to date", config.name);
                        ok += 1;
                    }
                } else if result.written {
                    println!(
                        "[{}] wrote {} ({} bytes)",
                        config.name,
                        output_path.display(),
                        code.len()
                    );
                    ok += 1;
                } else {
                    // Dry-run
                    println!(
                        "[{}] would write to: {}",
                        config.name,
                        output_path.display()
                    );
                    println!("  {} bytes, {} lines", code.len(), code.lines().count());
                    ok += 1;
                }
            }
            Err(e) => {
                eprintln!(
                    "[{}] error: {}: {}",
                    config.name,
                    output_path.display(),
                    e
                );
                errors += 1;
            }
        }
    }

    println!();

    match mode {
        Mode::Generate => {
            println!("generated: {} files, {} errors", ok, errors);
        }
        Mode::Check => {
            println!("check complete: {} ok, {} stale", ok, stale);
            if stale > 0 {
                println!();
                println!("Generated tests are out of date. Run `make testgen` to regenerate:");
                for file in &stale_files {
                    println!("  {}", file);
                }
            }
        }
        Mode::DryRun => {
            println!("dry-run complete: {} targets", targets.len());
        }
    }

    // Update manifest after successful generation
    if errors == 0 && mode == Mode::Generate {
        update_manifest_after_testgen();
    }

    if errors > 0 || stale > 0 {
        process::exit(1);
    }
}

// ============================================================================
// Resource Manifest Support
// ============================================================================

/// Update the resource manifest after successful testgen.
fn update_manifest_after_testgen() {
    println!();
    println!("Updating resource manifest...");

    #[derive(Clone)]
    struct TestgenResource {
        def: ResourceDef,
        outputs: Vec<PathBuf>,
    }

    impl ManagedResource for TestgenResource {
        fn definition(&self) -> &ResourceDef {
            &self.def
        }

        fn create(&self, manifest: &ResourceManifest) -> Result<ManifestEntry, ResourceError> {
            let (key, file_count) = self.compute_key_with_stats(manifest)?;
            Ok(ManifestEntry::new(key, file_count).with_outputs(self.outputs.clone()))
        }
    }

    let def = testgen_resource_def();
    let resource = TestgenResource {
        def: def.clone(),
        outputs: Vec::new(),
    };

    match update_resource_manifest(&resource) {
        Ok(()) => {
            println!("Resource manifest updated.");
        }
        Err(ManifestUpdateError::Load(e)) => {
            eprintln!("Failed to load manifest: {e}");
        }
        Err(ManifestUpdateError::Save(e)) => {
            eprintln!("Failed to write manifest: {e}");
        }
        Err(ManifestUpdateError::Acquire(e)) => {
            eprintln!("Failed to update manifest: {e}");
        }
    }
}

fn print_help() {
    println!("testgen - Generate tests from DAG structures and MockSpecs");
    println!("Usage:");
    println!("    gunbc-testgen [OPTIONS]");
    println!();
    println!("Options:");
    println!("    -o, --output-dir <DIR>  Output directory (default: current)");
    println!("    -n, --dry-run           Show what would be generated without writing");
    println!("    -c, --check             Check if generated tests are up to date (CI mode)");
    println!("    -h, --help              Show this help message");
}
