//! gunbc-testgen main entry point.
//!
//! Generates test files from DAG structures and MockSpecs.
//!
//! Usage:
//!     cargo run -p gunbc-dag --bin gunbc-testgen
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --dry-run
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --check
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --output-dir /path/to/output

use gunbc_codegen::TestgenTargetDef;
use gunbc_exec::Executable;
use gunbc_ir::Dag;
use gunbc_test::MockSpec;
use gunbc_testgen::{TestConfig, TestGenerator};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
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
// Each entry defines both metadata (output path, module name, etc.) and the
// builder that produces the actual DAG + MockSpec. No separate registry needed.
// ============================================================================

/// Build all testgen targets.
///
/// Adding a new testgen target: add an entry below with:
/// - `TestgenTargetDef::new(name, output_path, module_name)` — metadata
/// - `.dag_builder()` / `.mock_spec()` — string expressions for generated code
/// - Builder closure — actual DAG + MockSpec construction
fn build_targets() -> Vec<TestgenTarget> {
    vec![
        // ====================================================================
        // Internal gunbc-dag DAGs (flow tests enabled)
        // ====================================================================
        TestgenTarget {
            config: TestgenTargetDef::new(
                "bootstrap",
                "gunbc-dag/src/bootstrap/generated_tests.rs",
                "bootstrap_generated_tests",
            )
            .dag_builder("crate::build_bootstrap_graph().unwrap()")
            .mock_spec("crate::bootstrap::graph_mock::bootstrap_mock_spec()")
            .flow_tests(),
            generate: |c| generate_target(c,
                gunbc_dag::build_bootstrap_graph().unwrap(),
                gunbc_dag::bootstrap::graph_mock::bootstrap_mock_spec(),
            ),
        },
        TestgenTarget {
            config: TestgenTargetDef::new(
                "ci",
                "gunbc-dag/src/ci/generated_tests.rs",
                "ci_generated_tests",
            )
            .dag_builder("crate::build_ci_graph().unwrap()")
            .mock_spec("crate::ci::graph_mock::ci_mock_spec()")
            .flow_tests(),
            generate: |c| generate_target(c,
                gunbc_dag::build_ci_graph().unwrap(),
                gunbc_dag::ci::graph_mock::ci_mock_spec(),
            ),
        },
        TestgenTarget {
            config: TestgenTargetDef::new(
                "makegen",
                "gunbc-dag/src/makegen/generated_tests.rs",
                "makegen_generated_tests",
            )
            .dag_builder("crate::build_makegen_graph().unwrap()")
            .mock_spec("crate::makegen::graph_mock::makegen_mock_spec()")
            .flow_tests(),
            generate: |c| generate_target(c,
                gunbc_dag::build_makegen_graph().unwrap(),
                gunbc_dag::makegen::graph_mock::makegen_mock_spec(),
            ),
        },
        // ====================================================================
        // Library DAGs (composable sub-DAGs)
        // ====================================================================
        TestgenTarget {
            config: TestgenTargetDef::new(
                "llm-openai",
                "lib/llm-ops/src/generated_tests.rs",
                "llm_openai_generated_tests",
            )
            .dag_builder("crate::graph::build_chat_completion_graph()")
            .mock_spec("crate::graph_mock::openai_mock_spec()")
            .no_boundary_tests(),
            generate: |c| generate_target(c,
                gunbc_lib_llm_ops::graph::build_chat_completion_graph(),
                gunbc_lib_llm_ops::graph_mock::openai_mock_spec(),
            ),
        },
        TestgenTarget {
            config: TestgenTargetDef::new(
                "llm-anthropic",
                "lib/llm-ops/src/generated_tests_anthropic.rs",
                "llm_anthropic_generated_tests",
            )
            .dag_builder("crate::graph::build_chat_completion_graph()")
            .mock_spec("crate::graph_mock::anthropic_mock_spec()")
            .no_boundary_tests(),
            generate: |c| generate_target(c,
                gunbc_lib_llm_ops::graph::build_chat_completion_graph(),
                gunbc_lib_llm_ops::graph_mock::anthropic_mock_spec(),
            ),
        },
        TestgenTarget {
            config: TestgenTargetDef::new(
                "llm-code-review",
                "lib/llm-ops/src/generated_tests_code_review.rs",
                "llm_code_review_generated_tests",
            )
            .dag_builder("crate::graph::build_chat_completion_graph()")
            .mock_spec("crate::graph_mock::code_review_mock_spec()")
            .no_boundary_tests(),
            generate: |c| generate_target(c,
                gunbc_lib_llm_ops::graph::build_chat_completion_graph(),
                gunbc_lib_llm_ops::graph_mock::code_review_mock_spec(),
            ),
        },
        TestgenTarget {
            config: TestgenTargetDef::new(
                "llm-secrets",
                "lib/llm-ops/src/generated_tests_secrets.rs",
                "llm_secrets_generated_tests",
            )
            .dag_builder("crate::graph::build_chat_completion_graph()")
            .mock_spec("crate::graph_mock::secret_api_key_mock_spec()")
            .no_boundary_tests(),
            generate: |c| generate_target(c,
                gunbc_lib_llm_ops::graph::build_chat_completion_graph(),
                gunbc_lib_llm_ops::graph_mock::secret_api_key_mock_spec(),
            ),
        },
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
        ..TestConfig::default()
    };
    TestGenerator::new(&dag)
        .with_config(test_config)
        .with_mock_spec(spec)
        .with_mock_spec_fn(&config.mock_spec_path)
        .generate_test_module(&config.module_name, &config.dag_builder_call)
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

    match mode {
        Mode::Generate => run_generate(&targets, &output_dir),
        Mode::Check => run_check(&targets, &output_dir),
        Mode::DryRun => run_dry_run(&targets, &output_dir),
    }
}

// ============================================================================
// Execution Functions
// ============================================================================

/// Generate and write test files.
#[allow(clippy::disallowed_methods)]
fn run_generate(targets: &[TestgenTarget], output_dir: &Path) {
    let mut generated = 0;
    let mut errors = 0;

    for target in targets {
        let code = (target.generate)(&target.config);
        let output_path = output_dir.join(&target.config.output_path);

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!(
                    "[{}] error creating dir {}: {}",
                    target.config.name,
                    parent.display(),
                    e
                );
                errors += 1;
                continue;
            }
        }

        match fs::write(&output_path, &code) {
            Ok(_) => {
                println!(
                    "[{}] wrote {} ({} bytes)",
                    target.config.name,
                    output_path.display(),
                    code.len()
                );
                generated += 1;
            }
            Err(e) => {
                eprintln!(
                    "[{}] error writing {}: {}",
                    target.config.name,
                    output_path.display(),
                    e
                );
                errors += 1;
            }
        }
    }

    println!();
    println!("generated: {} files, {} errors", generated, errors);

    if errors > 0 {
        process::exit(1);
    }
}

/// Check if generated files are stale.
#[allow(clippy::disallowed_methods)]
fn run_check(targets: &[TestgenTarget], output_dir: &Path) {
    let mut ok = 0;
    let mut stale = 0;
    let mut missing = 0;
    let mut stale_files = Vec::new();

    for target in targets {
        let expected = (target.generate)(&target.config);
        let output_path = output_dir.join(&target.config.output_path);

        match fs::read_to_string(&output_path) {
            Ok(actual) => {
                if actual == expected {
                    println!("[{}] ✓ up to date", target.config.name);
                    ok += 1;
                } else {
                    println!("[{}] ✗ STALE - needs regeneration", target.config.name);
                    stale += 1;
                    stale_files.push(target.config.output_path.clone());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!(
                    "[{}] ✗ MISSING - {}",
                    target.config.name,
                    output_path.display()
                );
                missing += 1;
                stale_files.push(target.config.output_path.clone());
            }
            Err(e) => {
                eprintln!(
                    "[{}] error reading {}: {}",
                    target.config.name,
                    output_path.display(),
                    e
                );
                stale += 1;
                stale_files.push(target.config.output_path.clone());
            }
        }
    }

    println!();
    println!(
        "check complete: {} ok, {} stale, {} missing",
        ok, stale, missing
    );

    if stale > 0 || missing > 0 {
        println!();
        println!("Generated tests are out of date. Run `make testgen` to regenerate:");
        for file in &stale_files {
            println!("  {}", file);
        }
        process::exit(1);
    }
}

/// Dry run - show what would be generated.
fn run_dry_run(targets: &[TestgenTarget], output_dir: &Path) {
    for target in targets {
        let code = (target.generate)(&target.config);
        let output_path = output_dir.join(&target.config.output_path);

        println!(
            "[{}] would write to: {}",
            target.config.name,
            output_path.display()
        );
        println!("  {} bytes, {} lines", code.len(), code.lines().count());
    }

    println!();
    println!("dry-run complete: {} targets", targets.len());
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
    println!("    --check     Verify existing files match what would be generated (fails if stale)");
    println!("    --dry-run   Show what would be generated without writing");
    println!();
    println!("REGISTERED DAGS ({} targets):", targets.len());
    for target in &targets {
        println!("  {}: {}", target.config.name, target.config.output_path);
    }
}
