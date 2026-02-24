//! gunbc-testgen main entry point.
//!
//! Generates test files from DAG structures and MockSpecs.
//! Discovers test blocks directly from `.dag` files at runtime — no intermediate
//! `graph_mock.rs` codegen step. Tier, hermetic, and fermi metadata are inferred
//! from DAG topology by `generate_target()`.
//!
//! Usage:
//!     cargo run -p gunbc-dag --bin gunbc-testgen
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --dry-run
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --mode=verify
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --output-dir /path/to/output

#![deny(dead_code)]
use gunbc_cli::BinaryArgs;
use gunbc_codegen::FileWriter;
use gunbc_dag::testgen_dag::{
    build_mock_spec_from_test, build_testgen_target_def, compile_dag_for_test,
    dag_builder_call_for_module, discover_dag_tests,
};
use gunbc_dag::{print_tool_header, testgen_resource_def};
use gunbc_exec::{print_attention, AttentionLevel};
use gunbc_ir::resource::{
    update_resource_manifest, ExecMode, ManagedResource, ManifestEntry, ManifestUpdateError,
    ResourceDef, ResourceError, ResourceIo, ResourceManifest,
};
use gunbc_ir::WorkspaceLayout;
use gunbc_lib_transport::TransportIo;
use gunbc_testgen_registry::generate_target;
use std::fmt::Write;
use std::path::PathBuf;
use std::process;

fn main() {
    let parsed = BinaryArgs::new()
        .with_mode()
        .with_string_param("output_dir", Some('o'), Some("."))
        .parse_env();
    if parsed.help {
        print_help();
        return;
    }
    let dry_run = parsed.dry_run;
    let resource_mode = parsed.resource_mode.unwrap_or(ExecMode::Ensure);
    let output_dir = PathBuf::from(parsed.get_string("output_dir").unwrap_or("."));

    // Resolve workspace root → dsl directory
    let layout = match WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
    {
        Ok(l) => l,
        Err(e) => {
            print_attention(
                AttentionLevel::Error,
                "Workspace layout",
                &format!("Failed to resolve workspace layout: {e}"),
            );
            process::exit(1);
        }
    };
    let dsl_root = layout.workspace_root.join("dsl");

    // Discover test targets from .dag files
    let targets = discover_dag_tests(&dsl_root);
    if targets.is_empty() {
        print_attention(
            AttentionLevel::Warning,
            "No testgen targets",
            "No .dag test blocks found; skipping test generation.",
        );
        return;
    }

    let is_verify = resource_mode == ExecMode::Verify;

    print_tool_header(
        "testgen",
        &[
            ("output_dir", output_dir.display().to_string()),
            (
                "mode",
                if is_verify {
                    "verify"
                } else if dry_run {
                    "dry-run"
                } else {
                    "real"
                }
                .to_string(),
            ),
            ("targets", targets.len().to_string()),
        ],
    );

    let io = TransportIo::new();
    let writer = FileWriter::new(dry_run || is_verify, &io);

    let mut ok_count = 0;
    let mut written_count = 0;
    let mut stale = Vec::new();
    let mut errors = Vec::new();

    for target in &targets {
        // Compile the DAG from .dag source
        let dag = match compile_dag_for_test(&target.dsl_module) {
            Ok(d) => d,
            Err(e) => {
                errors.push(format!(
                    "{}: failed to compile DAG: {e}",
                    target.test_name
                ));
                continue;
            }
        };

        // Build MockSpec from test block
        let spec = build_mock_spec_from_test(&dag, target);

        // Build target definition
        let dag_builder_call = dag_builder_call_for_module(&target.dsl_module);
        let config = build_testgen_target_def(target, &output_dir, &dag_builder_call);

        // Generate test code (catch panics from mock validation)
        let test_code = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            generate_target(&config, dag, spec)
        })) {
            Ok(code) => code,
            Err(e) => {
                let msg = e
                    .downcast_ref::<String>()
                    .map(|s| s.as_str())
                    .or_else(|| e.downcast_ref::<&str>().copied())
                    .unwrap_or("unknown panic");
                errors.push(format!("{}: {msg}", target.test_name));
                continue;
            }
        };

        // Content upsert
        let output_path = config.output_path.as_ref();
        match writer.write_if_changed(output_path, &test_code) {
            Ok(result) => {
                if result.changed {
                    if is_verify {
                        println!("[{}] STALE - needs regeneration", target.test_name);
                        stale.push(output_path.to_string());
                    } else if dry_run {
                        println!("[{}] would write {}", target.test_name, output_path);
                        written_count += 1;
                    } else {
                        println!("[{}] updated {}", target.test_name, output_path);
                        written_count += 1;
                    }
                } else {
                    println!("[{}] up to date", target.test_name);
                    ok_count += 1;
                }
            }
            Err(e) => {
                errors.push(format!("{}: write failed: {e}", target.test_name));
            }
        }
    }

    // Summary
    println!();
    if !errors.is_empty() {
        let mut body = String::new();
        for err in &errors {
            writeln!(body, "  {err}").unwrap();
        }
        print_attention(AttentionLevel::Error, "testgen errors", body.trim_end());
    }

    if is_verify {
        println!(
            "check complete: {} ok, {} stale",
            ok_count,
            stale.len()
        );
        if !stale.is_empty() {
            let mut body = String::new();
            body.push_str("Run `make testgen` to regenerate:\n");
            for path in &stale {
                writeln!(body, "  {path}").unwrap();
            }
            print_attention(
                AttentionLevel::Error,
                "testgen --mode=verify: generated tests are out of date",
                body.trim_end(),
            );
            process::exit(1);
        }
    } else {
        println!(
            "testgen complete: {} up to date, {} {}",
            ok_count,
            written_count,
            if dry_run { "would write" } else { "written" }
        );
    }

    if !errors.is_empty() {
        process::exit(1);
    }

    // Update manifest after successful generation (not in DAG - post-execution step)
    if !dry_run && !is_verify {
        update_manifest_after_testgen();
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

        fn create(
            &self,
            manifest: &ResourceManifest,
            io: &dyn ResourceIo,
        ) -> Result<ManifestEntry, ResourceError> {
            let (key, file_count, input_files) = self.compute_key_with_file_list(manifest, io)?;
            Ok(ManifestEntry::new(key, file_count)
                .with_outputs(self.outputs.clone())
                .with_input_files(input_files))
        }
    }

    let def = testgen_resource_def();
    let resource = TestgenResource {
        def: def.clone(),
        outputs: Vec::new(),
    };

    let io = TransportIo::new();
    match update_resource_manifest(&resource, &io) {
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
    println!("testgen - Generate tests from .dag test blocks");
    println!("Usage:");
    println!("    gunbc-testgen [OPTIONS]");
    println!();
    println!("Options:");
    println!("    -o, --output-dir <DIR>  Output directory (default: current)");
    println!("    -n, --dry-run           Show what would be generated without writing");
    println!("    --mode=MODE             Resource mode: verify (CI) or ensure (default)");
    println!("    -h, --help              Show this help message");
    println!();
    println!("Discovers test blocks from dsl/tools/*.dag files at runtime.");
    println!("Tier, hermetic, and fermi metadata are inferred from DAG topology.");
}
