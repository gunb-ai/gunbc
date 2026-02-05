//! gunbc-testgen main entry point.
//!
//! Generates test files from DAG structures and MockSpecs.
//!
//! Usage:
//!     cargo run -p gunbc-dag --bin gunbc-testgen
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --dry-run
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --check
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --output-dir /path/to/output

use gunbc_codegen::testgen::{TestConfig, TestGenerator};
use gunbc_codegen::{FileWriter, TestgenTargetDef};
use gunbc_dag::testgen_resource_def;
use gunbc_exec::Executable;
use gunbc_ir::resource::{
    compute_key_from_def, ExecMode, ManagedResource, ManifestEntry, ResourceDef, ResourceError,
    ResourceManifest,
};
use gunbc_ir::Dag;
use gunbc_test::MockSpec;
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

/// A test generation target: config metadata + a function that builds DAG and MockSpec.
struct TestgenTarget {
    /// Configuration from the registry (output path, module name, etc.)
    config: TestgenTargetDef,
    /// Function to generate the test code
    generate: fn(&TestgenTargetDef) -> String,
}

// ============================================================================
// Target Registration
//
// Single registration site: add new DAGs here.
// Each expression is written once — the macro both calls it (for testgen)
// and stringifies it (for the generated test code).
// ============================================================================

/// Define a testgen target with zero duplication.
///
/// The `dag` and `mock` expressions are written once. The macro:
/// 1. Calls them directly (testgen executes them to analyze the DAG)
/// 2. Stringifies them and replaces the crate prefix with `crate::`
///    (generated test files live inside the target crate)
macro_rules! target {
    (
        $name:expr, $output:expr, $module:expr,
        $krate:expr,
        dag: $dag:expr,
        mock: $mock:expr,
        signature: $signature:expr
        $(, $config:ident)*
    ) => {
        TestgenTarget {
            config: TestgenTargetDef::new($name, $output, $module)
                .dag_builder(&to_crate_path(stringify!($dag), $krate))
                .mock_spec(&to_crate_path(stringify!($mock), $krate))
                .signature(&to_crate_path(stringify!($signature), $krate))
                $(.$config())*,
            generate: |c| generate_target(c, $dag, $mock),
        }
    };
    (
        $name:expr, $output:expr, $module:expr,
        $krate:expr,
        dag: $dag:expr,
        mock: $mock:expr
        $(, $config:ident)*
    ) => {
        TestgenTarget {
            config: TestgenTargetDef::new($name, $output, $module)
                .dag_builder(&to_crate_path(stringify!($dag), $krate))
                .mock_spec(&to_crate_path(stringify!($mock), $krate))
                $(.$config())*,
            generate: |c| generate_target(c, $dag, $mock),
        }
    };
}

/// Replace an external crate name with `crate::` for generated test code.
///
/// Generated test files live inside the target crate (e.g., gunbc-dag),
/// so `gunbc_dag::foo()` becomes `crate::foo()` in the emitted code.
fn to_crate_path(stringified: &str, krate: &str) -> String {
    let prefix = format!("{}::", krate);
    stringified.replacen(&prefix, "crate::", 1)
}

/// Build all testgen targets.
///
/// Adding a new target: add a `target!()` entry below.
/// The DAG builder and MockSpec expressions are each written once.
fn build_targets() -> Vec<TestgenTarget> {
    vec![
        // ====================================================================
        // Internal gunbc-dag DAGs (flow tests enabled)
        // ====================================================================
        target!("bootstrap", "gunbc-dag/src/bootstrap/generated_tests.rs", "bootstrap_generated_tests",
            "gunbc_dag",
            dag: gunbc_dag::build_bootstrap_graph().unwrap(),
            mock: gunbc_dag::bootstrap::graph_mock::bootstrap_mock_spec(),
            signature: gunbc_dag::bootstrap_signature(),
            flow_tests
        ),
        target!("ci", "gunbc-dag/src/ci/generated_tests.rs", "ci_generated_tests",
            "gunbc_dag",
            dag: gunbc_dag::build_ci_graph().unwrap(),
            mock: gunbc_dag::ci::graph_mock::ci_mock_spec(),
            signature: gunbc_dag::ci_signature(),
            flow_tests
        ),
        target!("makegen", "gunbc-dag/src/makegen/generated_tests.rs", "makegen_generated_tests",
            "gunbc_dag",
            dag: gunbc_dag::build_makegen_graph().unwrap(),
            mock: gunbc_dag::makegen::graph_mock::makegen_mock_spec(),
            signature: gunbc_dag::makegen_signature(),
            flow_tests
        ),
        // ====================================================================
        // Library DAGs (composable sub-DAGs)
        // ====================================================================
        target!("llm-openai", "lib/llm-ops/src/generated_tests.rs", "llm_openai_generated_tests",
            "gunbc_lib_llm_ops",
            dag: gunbc_lib_llm_ops::graph::build_chat_completion_graph(),
            mock: gunbc_lib_llm_ops::graph_mock::openai_mock_spec(),
            no_boundary_tests
        ),
        target!("llm-anthropic", "lib/llm-ops/src/generated_tests_anthropic.rs", "llm_anthropic_generated_tests",
            "gunbc_lib_llm_ops",
            dag: gunbc_lib_llm_ops::graph::build_chat_completion_graph(),
            mock: gunbc_lib_llm_ops::graph_mock::anthropic_mock_spec(),
            no_boundary_tests
        ),
        target!("llm-code-review", "lib/llm-ops/src/generated_tests_code_review.rs", "llm_code_review_generated_tests",
            "gunbc_lib_llm_ops",
            dag: gunbc_lib_llm_ops::graph::build_chat_completion_graph(),
            mock: gunbc_lib_llm_ops::graph_mock::code_review_mock_spec(),
            no_boundary_tests
        ),
        target!("llm-secrets", "lib/llm-ops/src/generated_tests_secrets.rs", "llm_secrets_generated_tests",
            "gunbc_lib_llm_ops",
            dag: gunbc_lib_llm_ops::graph::build_chat_completion_graph(),
            mock: gunbc_lib_llm_ops::graph_mock::secret_api_key_mock_spec(),
            no_boundary_tests
        ),
    ]
}

/// Generic test generation: builds test code from a DAG + MockSpec + config.
///
/// This is the single codegen path — all targets use this function.
/// Per-target variation is only in which DAG and MockSpec are provided.
fn generate_target<T: Executable + Clone>(
    config: &TestgenTargetDef,
    dag: Dag<T>,
    spec: MockSpec,
) -> String {
    let test_config = TestConfig {
        boundary_tests: config.boundary_tests,
        chain_tests: config.chain_tests,
        flow_tests: config.flow_tests,
        window_max_nodes: config.window_max_nodes,
        ..TestConfig::default()
    };
    let mut generator = TestGenerator::new(&dag)
        .with_config(test_config)
        .with_mock_spec(spec)
        .with_mock_spec_fn(&config.mock_spec_path);
    if let Some(signature_fn) = &config.signature_path {
        generator = generator.with_signature_fn(signature_fn);
    }
    generator.generate_test_module(&config.module_name, &config.dag_builder_call)
}

// ============================================================================
// Main
// ============================================================================

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
    // Check mode is read-only (like dry-run) — we only compare, never write.
    let writer = FileWriter::new(mode != Mode::Generate);

    let mut ok = 0;
    let mut stale = 0;
    let mut errors = 0;
    let mut stale_files = Vec::new();

    for target in &targets {
        let code = (target.generate)(&target.config);
        let output_path = output_dir.join(&target.config.output_path);

        match writer.write(&output_path, &code) {
            Ok(result) => {
                if mode == Mode::Check {
                    // Check mode: report stale/ok without writing
                    if result.changed {
                        println!("[{}] ✗ STALE - needs regeneration", target.config.name);
                        stale += 1;
                        stale_files.push(target.config.output_path.clone());
                    } else {
                        println!("[{}] ✓ up to date", target.config.name);
                        ok += 1;
                    }
                } else if result.written {
                    println!(
                        "[{}] wrote {} ({} bytes)",
                        target.config.name,
                        output_path.display(),
                        code.len()
                    );
                    ok += 1;
                } else {
                    // Dry-run
                    println!(
                        "[{}] would write to: {}",
                        target.config.name,
                        output_path.display()
                    );
                    println!("  {} bytes, {} lines", code.len(), code.lines().count());
                    ok += 1;
                }
            }
            Err(e) => {
                eprintln!(
                    "[{}] error: {}: {}",
                    target.config.name,
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
            let (key, file_count) = compute_key_from_def(&self.def, manifest)?;
            Ok(ManifestEntry::new(key, file_count).with_outputs(self.outputs.clone()))
        }
    }

    let def = testgen_resource_def();
    let mut manifest = match ResourceManifest::load_default() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("  ERROR: Could not load manifest: {}", e);
            eprintln!("  Testgen outputs exist but freshness cannot be verified.");
            eprintln!("  CI --mode=verify will fail until manifest is written.");
            return;
        }
    };

    // Collect output paths from all targets
    let targets = build_targets();
    let outputs: Vec<PathBuf> = targets
        .iter()
        .map(|t| PathBuf::from(&t.config.output_path))
        .collect();

    let resource = TestgenResource { def, outputs };
    match resource.acquire(ExecMode::Ensure, &mut manifest) {
        Ok(_) => {
            if let Err(e) = manifest.save_default() {
                eprintln!("  ERROR: Could not write manifest: {}", e);
                eprintln!("  Testgen outputs exist but freshness cannot be verified.");
                eprintln!("  CI --mode=verify will fail until manifest is written.");
            } else {
                println!("  Updated resource manifest: target/.resource-manifest.json");
            }
        }
        Err(e) => {
            eprintln!("  ERROR: Could not update manifest: {}", e);
            eprintln!("  Testgen outputs exist but freshness cannot be verified.");
            eprintln!("  CI --mode=verify will fail until manifest is written.");
        }
    }
}

fn print_help() {
    let targets = build_targets();

    println!("testgen - Generate tests from DAG structures and MockSpecs");
    println!();
    println!("USAGE:");
    println!("    gunbc-testgen [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -o, --output-dir <DIR>  Output base directory (default: .)");
    println!("    -n, --dry-run           Don't write files, just show what would be generated");
    println!("    -c, --check             Check if generated files are stale (CI mode)");
    println!("    -h, --help              Print this help");
    println!();
    println!("MODES:");
    println!("    (default)   Generate and write test files");
    println!(
        "    --check     Verify existing files match what would be generated (fails if stale)"
    );
    println!("    --dry-run   Show what would be generated without writing");
    println!();
    println!("REGISTERED DAGS ({} targets):", targets.len());
    for target in &targets {
        println!("  {}: {}", target.config.name, target.config.output_path);
    }
}
