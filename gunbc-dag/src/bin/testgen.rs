//! gunbc-testgen main entry point.
//!
//! Generates test files from DAG structures and MockSpecs.
//!
//! Usage:
//!     cargo run -p gunbc-dag --bin gunbc-testgen
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --dry-run
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --output-dir /path/to/output

use gunbc_testgen::{TestConfig, TestGenerator};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

/// A test generation target: a DAG + MockSpec + output path.
struct TestgenTarget {
    /// Human-readable name
    name: &'static str,
    /// Relative path from workspace root where the generated file goes
    output_path: &'static str,
    /// Function to generate the test code
    generate: fn() -> String,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut output_dir = PathBuf::from(".");
    let mut dry_run = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output-dir" => {
                i += 1;
                if i < args.len() {
                    output_dir = PathBuf::from(&args[i]);
                }
            }
            "-n" | "--dry-run" => dry_run = true,
            "-h" | "--help" => {
                print_help();
                return;
            }
            _ => {}
        }
        i += 1;
    }

    println!("testgen");
    println!("  output_dir: {}", output_dir.display());
    println!("  mode: {}", if dry_run { "dry-run" } else { "real" });
    println!();

    let targets = all_targets();

    let mut generated = 0;
    let mut errors = 0;

    for target in &targets {
        let code = (target.generate)();
        let output_path = output_dir.join(target.output_path);

        if dry_run {
            println!("[{}] would write to: {}", target.name, output_path.display());
            println!("  {} bytes, {} lines", code.len(), code.lines().count());
        } else {
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
    }

    println!();
    if dry_run {
        println!("dry-run complete: {} targets", targets.len());
    } else {
        println!(
            "generated: {} files, {} errors",
            generated, errors
        );
    }

    if errors > 0 {
        process::exit(1);
    }
}

/// All testgen targets in the project.
fn all_targets() -> Vec<TestgenTarget> {
    vec![
        // gunbc-dag internal DAGs
        // NOTE: boundary_tests disabled until testgen supports entrypoint input mocking
        // (dry-run only intercepts transport executors; pure nodes still need inputs)
        TestgenTarget {
            name: "bootstrap",
            output_path: "gunbc-dag/src/bootstrap/generated_tests.rs",
            generate: || {
                let dag = gunbc_dag::build_bootstrap_graph().unwrap();
                let spec = gunbc_dag::bootstrap::graph_mock::bootstrap_mock_spec();
                let config = TestConfig { boundary_tests: false, ..TestConfig::default() };
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
                let config = TestConfig { boundary_tests: false, ..TestConfig::default() };
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
                let config = TestConfig { boundary_tests: false, ..TestConfig::default() };
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
    println!("    -h, --help              Print this help");
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
