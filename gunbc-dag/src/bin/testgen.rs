//! gunbc-testgen main entry point.
//!
//! Generates test files from DAG structures and MockSpecs.
//!
//! Usage:
//!     cargo run -p gunbc-dag --bin gunbc-testgen
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --dry-run
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --check
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --output-dir /path/to/output

use gunbc_testgen::{TestConfig, TestGenerator};
use std::env;
use std::fs;
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

/// A test generation target: a DAG + MockSpec + output path.
struct TestgenTarget {
    /// Human-readable name
    name: &'static str,
    /// Relative path from workspace root where the generated file goes
    output_path: &'static str,
    /// Function to generate the test code
    generate: fn() -> String,
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

    let targets = all_targets();

    match mode {
        Mode::Generate => run_generate(&targets, &output_dir),
        Mode::Check => run_check(&targets, &output_dir),
        Mode::DryRun => run_dry_run(&targets, &output_dir),
    }
}

/// Generate and write test files.
#[allow(clippy::disallowed_methods)]
fn run_generate(targets: &[TestgenTarget], output_dir: &PathBuf) {
    let mut generated = 0;
    let mut errors = 0;

    for target in targets {
        let code = (target.generate)();
        let output_path = output_dir.join(target.output_path);

        // Ensure parent directory exists
        if let Some(parent) = output_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("[{}] error creating dir {}: {}", target.name, parent.display(), e);
                errors += 1;
                continue;
            }
        }

        match fs::write(&output_path, &code) {
            Ok(_) => {
                println!(
                    "[{}] wrote {} ({} bytes)",
                    target.name,
                    output_path.display(),
                    code.len()
                );
                generated += 1;
            }
            Err(e) => {
                eprintln!("[{}] error writing {}: {}", target.name, output_path.display(), e);
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
fn run_check(targets: &[TestgenTarget], output_dir: &PathBuf) {
    let mut ok = 0;
    let mut stale = 0;
    let mut missing = 0;
    let mut stale_files = Vec::new();

    for target in targets {
        let expected = (target.generate)();
        let output_path = output_dir.join(target.output_path);

        match fs::read_to_string(&output_path) {
            Ok(actual) => {
                if actual == expected {
                    println!("[{}] ✓ up to date", target.name);
                    ok += 1;
                } else {
                    println!("[{}] ✗ STALE - needs regeneration", target.name);
                    stale += 1;
                    stale_files.push(target.output_path);
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!("[{}] ✗ MISSING - {}", target.name, output_path.display());
                missing += 1;
                stale_files.push(target.output_path);
            }
            Err(e) => {
                eprintln!("[{}] error reading {}: {}", target.name, output_path.display(), e);
                stale += 1;
                stale_files.push(target.output_path);
            }
        }
    }

    println!();
    println!("check complete: {} ok, {} stale, {} missing", ok, stale, missing);

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
fn run_dry_run(targets: &[TestgenTarget], output_dir: &PathBuf) {
    for target in targets {
        let code = (target.generate)();
        let output_path = output_dir.join(target.output_path);

        println!("[{}] would write to: {}", target.name, output_path.display());
        println!("  {} bytes, {} lines", code.len(), code.lines().count());
    }

    println!();
    println!("dry-run complete: {} targets", targets.len());
}

/// All testgen targets in the project.
fn all_targets() -> Vec<TestgenTarget> {
    vec![
        // gunbc-dag internal DAGs — flow tests enabled (Phase 1)
        // These DAGs have no external inputs and can run DryRun today.
        // Chain tests (self-consistency) replaced by flow verification.
        TestgenTarget {
            name: "bootstrap",
            output_path: "gunbc-dag/src/bootstrap/generated_tests.rs",
            generate: || {
                let dag = gunbc_dag::build_bootstrap_graph().unwrap();
                let spec = gunbc_dag::bootstrap::graph_mock::bootstrap_mock_spec();
                let config = TestConfig {
                    boundary_tests: false,
                    chain_tests: false,
                    flow_tests: true,
                    ..TestConfig::default()
                };
                TestGenerator::new(&dag)
                    .with_config(config)
                    .with_mock_spec(spec)
                    .generate_test_module("bootstrap_generated_tests", "crate::build_bootstrap_graph().unwrap()")
            },
        },
        TestgenTarget {
            name: "ci",
            output_path: "gunbc-dag/src/ci/generated_tests.rs",
            generate: || {
                let dag = gunbc_dag::build_ci_graph().unwrap();
                let spec = gunbc_dag::ci::graph_mock::ci_mock_spec();
                let config = TestConfig {
                    boundary_tests: false,
                    chain_tests: false,
                    flow_tests: true,
                    ..TestConfig::default()
                };
                TestGenerator::new(&dag)
                    .with_config(config)
                    .with_mock_spec(spec)
                    .generate_test_module("ci_generated_tests", "crate::build_ci_graph().unwrap()")
            },
        },
        TestgenTarget {
            name: "makegen",
            output_path: "gunbc-dag/src/makegen/generated_tests.rs",
            generate: || {
                let dag = gunbc_dag::build_makegen_graph().unwrap();
                let spec = gunbc_dag::makegen::graph_mock::makegen_mock_spec();
                let config = TestConfig {
                    boundary_tests: false,
                    chain_tests: false,
                    flow_tests: true,
                    ..TestConfig::default()
                };
                TestGenerator::new(&dag)
                    .with_config(config)
                    .with_mock_spec(spec)
                    .generate_test_module("makegen_generated_tests", "crate::build_makegen_graph().unwrap()")
            },
        },
        // LLM ops (composable sub-DAGs — no standalone boundary execution tests)
        TestgenTarget {
            name: "llm-openai",
            output_path: "lib/llm-ops/src/generated_tests.rs",
            generate: || {
                let dag = gunbc_lib_llm_ops::graph::build_chat_completion_graph();
                let spec = gunbc_lib_llm_ops::graph_mock::openai_mock_spec();
                let config = TestConfig { boundary_tests: false, ..TestConfig::default() };
                TestGenerator::new(&dag)
                    .with_config(config)
                    .with_mock_spec(spec)
                    .generate_test_module("llm_openai_generated_tests", "crate::graph::build_chat_completion_graph()")
            },
        },
        TestgenTarget {
            name: "llm-anthropic",
            output_path: "lib/llm-ops/src/generated_tests_anthropic.rs",
            generate: || {
                let dag = gunbc_lib_llm_ops::graph::build_chat_completion_graph();
                let spec = gunbc_lib_llm_ops::graph_mock::anthropic_mock_spec();
                let config = TestConfig { boundary_tests: false, ..TestConfig::default() };
                TestGenerator::new(&dag)
                    .with_config(config)
                    .with_mock_spec(spec)
                    .generate_test_module("llm_anthropic_generated_tests", "crate::graph::build_chat_completion_graph()")
            },
        },
        TestgenTarget {
            name: "llm-code-review",
            output_path: "lib/llm-ops/src/generated_tests_code_review.rs",
            generate: || {
                let dag = gunbc_lib_llm_ops::graph::build_chat_completion_graph();
                let spec = gunbc_lib_llm_ops::graph_mock::code_review_mock_spec();
                let config = TestConfig { boundary_tests: false, ..TestConfig::default() };
                TestGenerator::new(&dag)
                    .with_config(config)
                    .with_mock_spec(spec)
                    .generate_test_module("llm_code_review_generated_tests", "crate::graph::build_chat_completion_graph()")
            },
        },
        TestgenTarget {
            name: "llm-secrets",
            output_path: "lib/llm-ops/src/generated_tests_secrets.rs",
            generate: || {
                let dag = gunbc_lib_llm_ops::graph::build_chat_completion_graph();
                let spec = gunbc_lib_llm_ops::graph_mock::secret_api_key_mock_spec();
                let config = TestConfig { boundary_tests: false, ..TestConfig::default() };
                TestGenerator::new(&dag)
                    .with_config(config)
                    .with_mock_spec(spec)
                    .generate_test_module("llm_secrets_generated_tests", "crate::graph::build_chat_completion_graph()")
            },
        },
    ]
}

fn print_help() {
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
    println!("Generated test files:");
    println!("  gunbc-dag/src/bootstrap/generated_tests.rs");
    println!("  gunbc-dag/src/ci/generated_tests.rs");
    println!("  gunbc-dag/src/makegen/generated_tests.rs");
    println!("  lib/llm-ops/src/generated_tests.rs");
    println!("  lib/llm-ops/src/generated_tests_anthropic.rs");
    println!("  lib/llm-ops/src/generated_tests_code_review.rs");
    println!("  lib/llm-ops/src/generated_tests_secrets.rs");
}
