//! gunbc-testgen main entry point.
//!
//! Generates test files from DAG structures and MockSpecs.
//! Progress display is automatic based on terminal capabilities.
//!
//! Usage:
//!     cargo run -p gunbc-dag --bin gunbc-testgen
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --dry-run
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --mode=verify
//!     cargo run -p gunbc-dag --bin gunbc-testgen -- --output-dir /path/to/output

#![deny(dead_code)]
use gunbc_cli::BinaryArgs;
use gunbc_dag::{print_tool_header, testgen_resource_def};
use gunbc_exec::{print_attention, AttentionLevel};
use gunbc_ir::resource::{
    update_resource_manifest, ExecMode, ManagedResource, ManifestEntry, ManifestUpdateError,
    ResourceDef, ResourceError, ResourceIo, ResourceManifest,
};
use gunbc_lib_transport::TransportIo;
// Force-link crates so inventory-driven testgen target registrations are retained.
// The `_` alias makes the side-effect-only intent explicit.
use gunbc_deps as _;
use gunbc_gist as _;
use gunbc_lib_llm_ops as _;
use gunbc_lib_review as _;
use gunbc_testgen_registry::{iter_dag_specs, DagSpecDef};
use std::fmt::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::process;

#[derive(Clone)]
struct TargetPlan {
    name: String,
    output_path: PathBuf,
    spec: &'static DagSpecDef,
}

/// Build all testgen targets from the auto-discovery registry.
fn build_targets() -> Vec<&'static DagSpecDef> {
    let mut targets: Vec<&'static DagSpecDef> = iter_dag_specs().collect();
    targets.sort_by(|a, b| a.name.cmp(b.name));
    targets
}

fn build_target_plans(targets: &[&'static DagSpecDef], output_dir: &Path) -> Vec<TargetPlan> {
    targets
        .iter()
        .map(|spec| {
            let config = spec.to_def();
            TargetPlan {
                name: config.name.to_string(),
                output_path: output_dir.join(config.output_path.as_ref()),
                spec,
            }
        })
        .collect()
}

fn read_existing_file(io: &dyn ResourceIo, path: &Path) -> Result<Option<String>, String> {
    let exists = io
        .file_exists(path)
        .map_err(|e| format!("failed checking {}: {e}", path.display()))?;
    if !exists {
        return Ok(None);
    }

    let bytes = io
        .read_file(path)
        .map_err(|e| format!("failed reading {}: {e}", path.display()))?;
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|e| format!("{} is not valid UTF-8: {e}", path.display()))
}

fn render_target_content(target: &TargetPlan) -> Result<String, String> {
    let config = target.spec.to_def();
    let generate_fn = target.spec.generate;
    match catch_unwind(AssertUnwindSafe(|| (generate_fn)(&config))) {
        Ok(content) => Ok(content),
        Err(payload) => {
            let message = if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "unknown panic".to_string()
            };
            Err(format!("generate '{}' failed:\n{}", target.name, message))
        }
    }
}

fn check_targets(plans: &[TargetPlan], io: &dyn ResourceIo) -> Result<Vec<String>, String> {
    let mut ok_count = 0usize;
    let mut stale = Vec::new();

    for plan in plans {
        let content = render_target_content(plan)?;
        let fresh = read_existing_file(io, &plan.output_path)?
            .map(|existing| existing == content)
            .unwrap_or(false);

        if fresh {
            println!("[{}] up to date", plan.name);
            ok_count += 1;
        } else {
            println!("[{}] STALE - needs regeneration", plan.name);
            stale.push(plan.output_path.display().to_string());
        }
    }

    println!();
    println!("check complete: {} ok, {} stale", ok_count, stale.len());
    Ok(stale)
}

fn generate_targets(
    plans: &[TargetPlan],
    dry_run: bool,
    io: &dyn ResourceIo,
) -> Result<(), String> {
    for plan in plans {
        let content = render_target_content(plan)?;
        let fresh = read_existing_file(io, &plan.output_path)?
            .map(|existing| existing == content)
            .unwrap_or(false);

        if fresh {
            println!("[{}] up to date", plan.name);
            continue;
        }

        if dry_run {
            println!("[{}] would write {}", plan.name, plan.output_path.display());
            continue;
        }

        io.write_file(&plan.output_path, content.as_bytes())
            .map_err(|e| format!("failed to write {}: {e}", plan.output_path.display()))?;
        println!("[{}] wrote {}", plan.name, plan.output_path.display());
    }
    Ok(())
}

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

    let targets = build_targets();
    if targets.is_empty() {
        print_attention(
            AttentionLevel::Error,
            "No testgen targets",
            "No testgen targets registered.",
        );
        process::exit(1);
    }

    let plans = build_target_plans(&targets, &output_dir);
    let io = TransportIo::new();

    if resource_mode == ExecMode::Verify {
        match check_targets(&plans, &io) {
            Ok(stale) => {
                if !stale.is_empty() {
                    let mut body = String::new();
                    body.push_str("Run `make testgen` to regenerate:\n");
                    for path in &stale {
                        writeln!(body, "  {path}").expect("string write should not fail");
                    }
                    print_attention(
                        AttentionLevel::Error,
                        "testgen --mode=verify: generated tests are out of date",
                        body.trim_end(),
                    );
                    process::exit(1);
                }
            }
            Err(e) => {
                print_attention(AttentionLevel::Error, "testgen --mode=verify failed", &e);
                process::exit(1);
            }
        }
    } else {
        print_tool_header(
            "testgen",
            &[
                ("output_dir", output_dir.display().to_string()),
                ("mode", if dry_run { "dry-run" } else { "real" }.to_string()),
                ("resource_mode", resource_mode.to_string()),
                ("targets", targets.len().to_string()),
            ],
        );
        if let Err(e) = generate_targets(&plans, dry_run, &io) {
            print_attention(AttentionLevel::Error, "testgen generation failed", &e);
            process::exit(1);
        }

        // Update manifest after successful generation (not in DAG - post-execution step)
        if !dry_run && resource_mode == ExecMode::Ensure {
            update_manifest_after_testgen();
        }
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
    println!("testgen - Generate tests from DAG structures and MockSpecs");
    println!("Usage:");
    println!("    gunbc-testgen [OPTIONS]");
    println!();
    println!("Options:");
    println!("    -o, --output-dir <DIR>  Output directory (default: current)");
    println!("    -n, --dry-run           Show what would be generated without writing");
    println!("    --mode=MODE             Resource mode: verify (CI) or ensure (default)");
    println!("    -h, --help              Show this help message");
    println!();
    println!("Progress display is automatic based on terminal capabilities.");
}
