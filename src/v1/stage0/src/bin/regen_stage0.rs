use std::collections::{BTreeSet, HashMap};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::rc::Rc;
use std::time::Instant;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::stage0_self_compile_refusal_message;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};

const BOOTSTRAP_TIMING_RECEIPT_VERSION: u32 = 2;
const BOOTSTRAP_TIMING_RECEIPT_SCHEMA: &str = "gunbc.bootstrap_timing_receipt.v2";
const BOOTSTRAP_TIMING_RECEIPT_ENV: &str = "GUNBC_BOOTSTRAP_TIMING_RECEIPT";
const DEFAULT_BOOTSTRAP_TIMING_RECEIPT: &str =
    "target/bootstrap_timing/v1_regen_stage0_receipt.json";

// Registry authority: gunbc.stage0_emit_model.generated_stage0_files.
// Dissolve-on: regen_stage0 reads emitted gunbc_stage0_emit_model.rs roster.
const GENERATED_STAGE0_FILES: &[&str] = &[
    "compiler_tests.rs",
    "extdeps_cargo.rs",
    "extdeps_cargo_version.rs",
    "extdeps_external_authority.rs",
    "extdeps_languages_dag_emit.rs",
    "extdeps_languages_dag_syntax.rs",
    "extdeps_languages_dag_types.rs",
    "extdeps_languages_go_emit.rs",
    "extdeps_languages_go_syntax.rs",
    "extdeps_languages_go_types.rs",
    "extdeps_languages_python_emit.rs",
    "extdeps_languages_python_syntax.rs",
    "extdeps_languages_python_types.rs",
    "extdeps_languages_rust_emit.rs",
    "extdeps_languages_rust_syntax.rs",
    "extdeps_languages_rust_types.rs",
    "extdeps_uri.rs",
    "extdeps_uri_path.rs",
    "extdeps_version.rs",
    "extdeps_version_semver.rs",
    "lib.rs",
    "std_algebra.rs",
    "std_coercion.rs",
    "std_computation.rs",
    "std_constructors.rs",
    "std_decl_ref.rs",
    "std_effects.rs",
    "std_emit_model.rs",
    "std_error_primitives.rs",
    "std_graph.rs",
    "std_http_path.rs",
    "std_induction.rs",
    "std_integer.rs",
    "std_iteration.rs",
    "std_lens_verdict.rs",
    "std_list.rs",
    "std_logic.rs",
    "std_machine_constraints.rs",
    "std_magnitude.rs",
    "std_measure.rs",
    "std_nat.rs",
    "std_node.rs",
    "std_pareto.rs",
    "std_realization_schedule.rs",
    "std_serialization.rs",
    "std_syntax.rs",
    "std_termination.rs",
    "std_types.rs",
    "v1_compiler_artifact.rs",
    "v1_compiler_coercion.rs",
    "v1_compiler_compile.rs",
    "v1_compiler_compiler_tests_rust.rs",
    "v1_compiler_complexity.rs",
    "v1_compiler_dag_collect.rs",
    "v1_compiler_dag_collect_support.rs",
    "v1_compiler_effect_derivation.rs",
    "v1_compiler_emit.rs",
    "v1_compiler_emit_core_support.rs",
    "v1_compiler_emit_go.rs",
    "v1_compiler_emit_python.rs",
    "v1_compiler_emit_rust.rs",
    "v1_compiler_infer.rs",
    "v1_compiler_infer_access.rs",
    "v1_compiler_infer_cycle.rs",
    "v1_compiler_infer_emit_info.rs",
    "v1_compiler_infer_env.rs",
    "v1_compiler_infer_items.rs",
    "v1_compiler_infer_lookup.rs",
    "v1_compiler_infer_method.rs",
    "v1_compiler_infer_patterns.rs",
    "v1_compiler_infer_resolve.rs",
    "v1_compiler_infer_service.rs",
    "v1_compiler_infer_sigs.rs",
    "v1_compiler_infer_types.rs",
    "v1_compiler_languages.rs",
    "v1_compiler_normalize.rs",
    "v1_compiler_ownership.rs",
    "v1_compiler_parse.rs",
    "v1_compiler_resolve.rs",
    "v1_compiler_runtime_go.rs",
    "v1_compiler_runtime_rust.rs",
    "v1_compiler_stage0_crates.rs",
    "v1_compiler_tokenize.rs",
    "v1_compiler_trace.rs",
    "v1_compiler_workspace_members.rs",
    "v1_probe_emit_interp.rs",
    "v1_rt.rs",
    "v1_std_core.rs",
    "v1_test_non_ascii_perf_fixture.rs",
    "wt_a.rs",
    "wt_b.rs",
    "wt_common.rs",
];

const HAND_MAINTAINED_STAGE0_FILES: &[&str] = &[
    "cli_run.rs",
    "coproduct_reflection.rs",
    "recorded_fixture.rs",
    "resolved_graph_cache.rs",
    // dag_collect split (#6053) removed Ci-subcommand / extract_module_path emission from
    // 05_emit_rust.dag; without those, the emitter no longer produces byte-identical main.rs,
    // so main.rs parks here until those gaps are restored in .dag. Registry flip deferred until
    // regen_stage0 --verify confirms byte-identical output vs this committed file.
    // Dissolution: run regen_stage0 --verify; on green, move to GENERATED_STAGE0_FILES.
    "main.rs",
    "v1_interpreter.rs",
    // Phase-local heartbeat for long floor walks (2026-07-04); its own header
    // carries the dissolution trigger (realization_measurement_loop Phase 0).
    "phase_profile.rs",
];

/// Hand-maintained stage0 support living in subdirectories (not flat `.rs` files).
const HAND_MAINTAINED_STAGE0_DIRS: &[&str] = &["module_path_index"];

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            let _ = writeln!(io::stderr(), "{message}");
            ExitCode::FAILURE
        }
    }
}

struct VerifyFinishInput<'a> {
    receipt_path: &'a Path,
    workspace: &'a Path,
    manifest_dir: &'a Path,
    stage0_src: &'a Path,
    fresh_dir: &'a Path,
    verify_only: bool,
    generated_file_count: usize,
    emitted_file_count: usize,
    phases: &'a mut Vec<BootstrapTimingPhase>,
    run_started: Instant,
    preserve_fresh_dir: bool,
}

fn finish_verify_checks(input: VerifyFinishInput<'_>) -> Result<(), String> {
    let VerifyFinishInput {
        receipt_path,
        workspace,
        manifest_dir,
        stage0_src,
        fresh_dir,
        verify_only,
        generated_file_count,
        emitted_file_count,
        phases,
        run_started,
        preserve_fresh_dir,
    } = input;
    let fresh_src = fresh_dir.join("src");
    let verify_result = time_phase(phases, "verify_stage0_matches", || {
        verify_stage0_matches(stage0_src, &fresh_src)
    });
    if let Err(message) = verify_result {
        let changed_generated_files = changed_registered_outputs(&fresh_src, stage0_src)?;
        write_bootstrap_timing_receipt(BootstrapTimingReceiptInput {
            path: receipt_path,
            workspace,
            manifest_dir,
            verify_only,
            status: "failed_stage0_stale",
            generated_file_count,
            emitted_file_count,
            phases: std::mem::take(phases),
            elapsed_ms: elapsed_ms(run_started),
            changed_generated_files,
        })?;
        if !preserve_fresh_dir {
            let _ = fs::remove_dir_all(fresh_dir);
        }
        return Err(message);
    }
    if let Err(message) = time_phase(phases, "verify_stage0_split_crate_boundaries", || {
        verify_stage0_split_crate_boundaries(workspace)
    }) {
        write_bootstrap_timing_receipt(BootstrapTimingReceiptInput {
            path: receipt_path,
            workspace,
            manifest_dir,
            verify_only,
            status: "failed_stage0_split_crate_stale",
            generated_file_count,
            emitted_file_count,
            phases: std::mem::take(phases),
            elapsed_ms: elapsed_ms(run_started),
            changed_generated_files: Vec::new(),
        })?;
        if !preserve_fresh_dir {
            let _ = fs::remove_dir_all(fresh_dir);
        }
        return Err(message);
    }
    if let Err(message) = time_phase(phases, "verify_workspace_members", || {
        verify_workspace_members(workspace)
    }) {
        write_bootstrap_timing_receipt(BootstrapTimingReceiptInput {
            path: receipt_path,
            workspace,
            manifest_dir,
            verify_only,
            status: "failed_workspace_members_stale",
            generated_file_count,
            emitted_file_count,
            phases: std::mem::take(phases),
            elapsed_ms: elapsed_ms(run_started),
            changed_generated_files: Vec::new(),
        })?;
        if !preserve_fresh_dir {
            let _ = fs::remove_dir_all(fresh_dir);
        }
        return Err(message);
    }
    write_bootstrap_timing_receipt(BootstrapTimingReceiptInput {
        path: receipt_path,
        workspace,
        manifest_dir,
        verify_only,
        status: if preserve_fresh_dir {
            "completed_emit_fresh_verify"
        } else {
            "completed"
        },
        generated_file_count,
        emitted_file_count,
        phases: std::mem::take(phases),
        elapsed_ms: elapsed_ms(run_started),
        changed_generated_files: Vec::new(),
    })?;
    if !preserve_fresh_dir {
        let _ = fs::remove_dir_all(fresh_dir);
    }
    if preserve_fresh_dir {
        println!(
            "regen_stage0 --emit-fresh --verify: committed stage0 matches fresh self-compile; artifacts at {}",
            fresh_dir.display()
        );
    } else {
        println!("regen_stage0 --verify: committed stage0 matches fresh self-compile.");
    }
    Ok(())
}

fn run() -> Result<(), String> {
    let run_started = Instant::now();
    let args: Vec<String> = env::args().skip(1).collect();
    // `--emit-fresh <dir>` runs the assembly phases (emit closure + copy hand-
    // maintained periphery + patches + rustfmt) into a STABLE, caller-named dir and
    // STOPS -- no copy-back into the committed seed, no temp cleanup. It is the
    // non-destructive re-baseline harness for the deferred regen-fixpoint lane
    // (carrier mark above): a faithful full regen WIRES every emitted module
    // (including the std-tower modules unwired in the committed lib.rs), so the
    // assembled crate is the "REAL" emitted seed whose emitter-completeness gaps
    // surface as cargo errors. Build it to measure them; the committed seed is
    // never touched.
    let mut emit_fresh: Option<PathBuf> = None;
    let mut write_manifest: Option<PathBuf> = None;
    let mut verify_only = false;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--verify" => {
                verify_only = true;
                index += 1;
            }
            "--emit-fresh" => {
                let dir = args
                    .get(index + 1)
                    .ok_or_else(|| "regen_stage0: --emit-fresh requires <dir>".to_string())?;
                emit_fresh = Some(PathBuf::from(dir));
                index += 2;
            }
            "--write-manifest" => {
                let path = args
                    .get(index + 1)
                    .ok_or_else(|| "regen_stage0: --write-manifest requires <path>".to_string())?;
                write_manifest = Some(PathBuf::from(path));
                index += 2;
            }
            unexpected => {
                return Err(format!(
                    "regen_stage0: unexpected argument: {unexpected:?}\n\
                     Usage: regen_stage0 [--verify | --emit-fresh <dir> [--write-manifest <path>] [--verify]]\n\
                     Omit flags to write stage0; pass `--verify` to check without writing;\n\
                     pass `--emit-fresh <dir>` to assemble the faithful emitted crate into <dir> and stop;\n\
                     add `--write-manifest <path>` to also write the GENERATED_STAGE0_FILES roster there;\n\
                     combine `--emit-fresh` with `--verify` to leave the assembled crate in place after checking."
                ));
            }
        }
    }
    if write_manifest.is_some() && emit_fresh.is_none() {
        return Err("regen_stage0: --write-manifest requires --emit-fresh".to_string());
    }

    assert_registry_is_partitioned()?;
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = workspace_root(&manifest_dir)?;
    let receipt_path = bootstrap_timing_receipt_path(&workspace);
    let stage0_src = manifest_dir.join("src");
    let fresh_dir = match &emit_fresh {
        Some(dir) => dir.clone(),
        None => temp_dir("v2-regen-stage0-fresh"),
    };
    let _ = fs::remove_dir_all(&fresh_dir);
    fs::create_dir_all(fresh_dir.join("src"))
        .map_err(|e| format!("create {}: {e}", fresh_dir.display()))?;

    let mut phases = Vec::new();
    let emitted = time_phase(&mut phases, "compile_stage0", || compile_stage0(&workspace))?;
    // Hand-maintained verification: diff each hand file against its fresh emit candidate
    // (without overwriting it) so the gate's exclusion of hand files stops being silent.
    // Runs before the crate assembly / registry asserts so the report is always visible on
    // an emit-fresh run, independent of later failures.
    if emit_fresh.is_some() {
        let report = time_phase(&mut phases, "verify_hand_maintained_candidates", || {
            verify_hand_maintained_candidates(&emitted, &stage0_src, &fresh_dir.join(".handverify"))
        })?;
        print_hand_verify_report(&report);
    }
    time_phase(&mut phases, "write_emitted_crate", || {
        write_emitted_crate(&fresh_dir, &emitted)
    })?;
    time_phase(&mut phases, "copy_hand_maintained_support", || {
        copy_hand_maintained_support(&stage0_src, &fresh_dir.join("src"))
    })?;
    time_phase(&mut phases, "assert_bootstrap_emit_core_support", || {
        assert_bootstrap_emit_core_support(&fresh_dir.join("src"))
    })?;
    time_phase(&mut phases, "rustfmt_generated_crate", || {
        rustfmt_generated_crate(&fresh_dir)
    })?;
    time_phase(&mut phases, "assert_output_set_matches_registry", || {
        assert_output_set_matches_registry(&stage0_src, &fresh_dir.join("src"))
    })?;

    if emit_fresh.is_some() {
        if let Some(manifest_path) = &write_manifest {
            let content = GENERATED_STAGE0_FILES.join("\n");
            fs::write(manifest_path, content)
                .map_err(|e| format!("write roster manifest {}: {e}", manifest_path.display()))?;
            println!(
                "regen_stage0 --write-manifest: wrote {} file entries to {}",
                GENERATED_STAGE0_FILES.len(),
                manifest_path.display()
            );
        }
        if verify_only {
            return finish_verify_checks(VerifyFinishInput {
                receipt_path: &receipt_path,
                workspace: &workspace,
                manifest_dir: &manifest_dir,
                stage0_src: &stage0_src,
                fresh_dir: &fresh_dir,
                verify_only,
                generated_file_count: GENERATED_STAGE0_FILES.len(),
                emitted_file_count: emitted.len(),
                phases: &mut phases,
                run_started,
                preserve_fresh_dir: true,
            });
        }
        write_bootstrap_timing_receipt(BootstrapTimingReceiptInput {
            path: &receipt_path,
            workspace: &workspace,
            manifest_dir: &manifest_dir,
            verify_only,
            status: "completed_emit_fresh",
            generated_file_count: GENERATED_STAGE0_FILES.len(),
            emitted_file_count: emitted.len(),
            phases,
            elapsed_ms: elapsed_ms(run_started),
            changed_generated_files: Vec::new(),
        })?;
        println!(
            "regen_stage0 --emit-fresh: assembled faithful emitted crate at {}",
            fresh_dir.display()
        );
        return Ok(());
    }

    if verify_only {
        return finish_verify_checks(VerifyFinishInput {
            receipt_path: &receipt_path,
            workspace: &workspace,
            manifest_dir: &manifest_dir,
            stage0_src: &stage0_src,
            fresh_dir: &fresh_dir,
            verify_only,
            generated_file_count: GENERATED_STAGE0_FILES.len(),
            emitted_file_count: emitted.len(),
            phases: &mut phases,
            run_started,
            preserve_fresh_dir: false,
        });
    }

    let changed_generated_files = changed_registered_outputs(&fresh_dir.join("src"), &stage0_src)?;
    time_phase(&mut phases, "write_registered_outputs", || {
        write_registered_outputs(&fresh_dir.join("src"), &stage0_src)
    })?;
    time_phase(&mut phases, "write_stage0_split_crate_boundaries", || {
        write_stage0_split_crate_boundaries(&workspace)
    })?;
    time_phase(&mut phases, "write_workspace_members", || {
        write_workspace_members(&workspace)
    })?;
    time_phase(&mut phases, "rustfmt_workspace", || {
        rustfmt_workspace(&manifest_dir)
    })?;
    write_bootstrap_timing_receipt(BootstrapTimingReceiptInput {
        path: &receipt_path,
        workspace: &workspace,
        manifest_dir: &manifest_dir,
        verify_only,
        status: "completed",
        generated_file_count: GENERATED_STAGE0_FILES.len(),
        emitted_file_count: emitted.len(),
        phases,
        elapsed_ms: elapsed_ms(run_started),
        changed_generated_files,
    })?;
    let _ = fs::remove_dir_all(&fresh_dir);
    println!(
        "regen_stage0: wrote {} generated stage0 files.",
        GENERATED_STAGE0_FILES.len()
    );
    Ok(())
}

fn workspace_root(manifest_dir: &Path) -> Result<PathBuf, String> {
    manifest_dir
        .ancestors()
        .nth(3)
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            format!(
                "could not find workspace root from {}",
                manifest_dir.display()
            )
        })
}

fn time_phase<T, F>(phases: &mut Vec<BootstrapTimingPhase>, name: &str, f: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String>,
{
    let started = Instant::now();
    let result = f();
    phases.push(BootstrapTimingPhase {
        name: name.to_string(),
        elapsed_ms: elapsed_ms(started),
    });
    result
}

fn elapsed_ms(started: Instant) -> u128 {
    started.elapsed().as_millis()
}

#[derive(serde::Serialize)]
struct BootstrapTimingReceipt {
    schema: &'static str,
    version: u32,
    subject: &'static str,
    mode: &'static str,
    status: &'static str,
    elapsed_ms: u128,
    phases: Vec<BootstrapTimingPhase>,
    generated_file_count: usize,
    emitted_file_count: usize,
    changed_generated_file_count: usize,
    changed_generated_files: Vec<String>,
    cargo_invalidation: CargoInvalidationReceipt,
}

#[derive(serde::Serialize)]
struct BootstrapTimingPhase {
    name: String,
    elapsed_ms: u128,
}

#[derive(serde::Serialize)]
struct CargoInvalidationReceipt {
    strategy: &'static str,
    inputs: Vec<CargoInvalidationInput>,
}

#[derive(serde::Serialize)]
struct CargoInvalidationInput {
    path: String,
    content_hash: String,
    len_bytes: u64,
}

struct BootstrapTimingReceiptInput<'a> {
    path: &'a Path,
    workspace: &'a Path,
    manifest_dir: &'a Path,
    verify_only: bool,
    status: &'static str,
    generated_file_count: usize,
    emitted_file_count: usize,
    phases: Vec<BootstrapTimingPhase>,
    elapsed_ms: u128,
    changed_generated_files: Vec<String>,
}

fn bootstrap_timing_receipt_path(workspace: &Path) -> PathBuf {
    env::var_os(BOOTSTRAP_TIMING_RECEIPT_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace.join(DEFAULT_BOOTSTRAP_TIMING_RECEIPT))
}

fn write_bootstrap_timing_receipt(input: BootstrapTimingReceiptInput<'_>) -> Result<(), String> {
    let receipt = BootstrapTimingReceipt {
        schema: BOOTSTRAP_TIMING_RECEIPT_SCHEMA,
        version: BOOTSTRAP_TIMING_RECEIPT_VERSION,
        subject: "v1_regen_stage0",
        mode: if input.verify_only { "verify" } else { "write" },
        status: input.status,
        elapsed_ms: input.elapsed_ms,
        phases: input.phases,
        generated_file_count: input.generated_file_count,
        emitted_file_count: input.emitted_file_count,
        changed_generated_file_count: input.changed_generated_files.len(),
        changed_generated_files: input.changed_generated_files,
        cargo_invalidation: cargo_invalidation_receipt(input.workspace, input.manifest_dir)?,
    };
    let body = serde_json::to_string_pretty(&receipt)
        .map_err(|e| format!("serialize bootstrap timing receipt: {e}"))?;
    if let Some(parent) = input.path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    fs::write(input.path, format!("{body}\n"))
        .map_err(|e| format!("write {}: {e}", input.path.display()))
}

fn cargo_invalidation_receipt(
    workspace: &Path,
    manifest_dir: &Path,
) -> Result<CargoInvalidationReceipt, String> {
    let paths = [
        workspace.join("Cargo.toml"),
        workspace.join("Cargo.lock"),
        manifest_dir.join("Cargo.toml"),
    ];
    let mut inputs = Vec::new();
    for path in paths {
        inputs.push(cargo_invalidation_input(workspace, &path)?);
    }
    Ok(CargoInvalidationReceipt {
        strategy: "content_hashes_for_workspace_manifest_lockfile_and_stage0_manifest",
        inputs,
    })
}

fn cargo_invalidation_input(
    workspace: &Path,
    path: &Path,
) -> Result<CargoInvalidationInput, String> {
    let bytes = fs::read(path)
        .map_err(|e| format!("read cargo invalidation input {}: {e}", path.display()))?;
    Ok(CargoInvalidationInput {
        path: display_source_path(path, workspace),
        content_hash: format!("fnv1a64:{:016x}", fnv1a64(&bytes)),
        len_bytes: bytes.len() as u64,
    })
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn temp_dir(name: &str) -> PathBuf {
    let unique = format!(
        "{name}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos()
    );
    env::temp_dir().join(unique)
}

fn assert_registry_is_partitioned() -> Result<(), String> {
    for generated in GENERATED_STAGE0_FILES {
        if HAND_MAINTAINED_STAGE0_FILES.contains(generated) {
            return Err(format!(
                "`{generated}` appears in both generated and hand-maintained stage0 registries"
            ));
        }
    }
    Ok(())
}

fn compile_stage0(workspace: &Path) -> Result<HashMap<String, String>, String> {
    let roots = vec![workspace.join("src/v1"), workspace.join("dag")];
    let sources = source_files_for_roots(&roots, workspace)?;
    let result = compile_sources(Rc::new(sources), RenderTarget::Rust);
    if let Some(message) = stage0_self_compile_refusal_message(result.clone()) {
        return Err(message);
    }

    let mut out = HashMap::new();
    for file in result.files.iter() {
        out.insert(file.path.clone(), file.content.clone());
    }
    Ok(out)
}

fn source_files_for_roots(
    roots: &[PathBuf],
    workspace: &Path,
) -> Result<Vec<Rc<SourceFile>>, String> {
    let index = build_module_index(roots)?;
    let entry_root = roots
        .first()
        .ok_or_else(|| "source root list must not be empty".to_string())?;
    let mut entry_files = Vec::new();
    let mut dag_paths = Vec::new();
    collect_dag_files(entry_root, &mut dag_paths)?;
    for path in dag_paths {
        let content =
            fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        entry_files.push((display_source_path(&path, workspace), content));
    }

    let mut seen: HashMap<String, Rc<SourceFile>> = HashMap::new();
    let mut queue = Vec::new();
    for (path, content) in &entry_files {
        if let Some(module_path) = extract_module_path(content) {
            seen.insert(
                module_path,
                Rc::new(SourceFile {
                    path: path.clone(),
                    content: content.clone(),
                }),
            );
        }
        queue.push((path.clone(), content.clone()));
    }

    while let Some((_path, content)) = queue.pop() {
        for module_path in extract_import_paths(&content) {
            if seen.contains_key(&module_path) {
                continue;
            }
            if let Some(file_path) = index.get(&module_path) {
                let file_content = fs::read_to_string(file_path)
                    .map_err(|e| format!("read imported module {}: {e}", file_path.display()))?;
                let rel_path = display_source_path(file_path, workspace);
                seen.insert(
                    module_path,
                    Rc::new(SourceFile {
                        path: rel_path.clone(),
                        content: file_content.clone(),
                    }),
                );
                queue.push((rel_path, file_content));
            }
        }
    }

    let mut result: Vec<Rc<SourceFile>> = seen.into_values().collect();
    result.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(result)
}

fn build_module_index(roots: &[PathBuf]) -> Result<HashMap<String, PathBuf>, String> {
    let mut index: HashMap<String, PathBuf> = HashMap::new();
    for root in roots {
        if !root.exists() {
            return Err(format!("source root does not exist: {}", root.display()));
        }
        let mut dag_paths = Vec::new();
        collect_dag_files(root, &mut dag_paths)?;
        for path in dag_paths {
            let content =
                fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
            if let Some(module_path) = extract_module_path(&content) {
                if let Some(existing) = index.get(&module_path) {
                    return Err(format!(
                        "duplicate module path `{module_path}`: {} and {}",
                        existing.display(),
                        path.display()
                    ));
                }
                index.insert(module_path, path);
            }
        }
    }
    Ok(index)
}

fn collect_dag_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| format!("read dir {}: {e}", dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("read dir entry in {}: {e}", dir.display()))?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files(&path, files)?;
        } else if path.extension().is_some_and(|ext| ext == "dag") {
            files.push(path);
        }
    }
    Ok(())
}

fn extract_module_path(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("module ") {
            return Some(rest.trim().to_string());
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            break;
        }
    }
    None
}

fn extract_import_paths(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("import ") {
            let module_path = rest.split('{').next().unwrap_or(rest).trim();
            if !module_path.is_empty() {
                imports.push(module_path.to_string());
            }
        }
    }
    imports
}

fn display_source_path(path: &Path, workspace: &Path) -> String {
    path.strip_prefix(workspace)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn write_emitted_crate(dir: &Path, files: &HashMap<String, String>) -> Result<(), String> {
    for (path, content) in files {
        let out_path = dir.join(path);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        fs::write(&out_path, content).map_err(|e| format!("write {}: {e}", out_path.display()))?;
    }
    Ok(())
}

fn copy_hand_maintained_support(stage0_src: &Path, dest_src: &Path) -> Result<(), String> {
    for file_name in HAND_MAINTAINED_STAGE0_FILES {
        let source = stage0_src.join(file_name);
        if source.exists() {
            fs::copy(&source, dest_src.join(file_name))
                .map_err(|e| format!("copy {}: {e}", source.display()))?;
        }
    }
    for dir_name in HAND_MAINTAINED_STAGE0_DIRS {
        let source = stage0_src.join(dir_name);
        if source.is_dir() {
            copy_dir_recursive(&source, &dest_src.join(dir_name))?;
        }
    }
    Ok(())
}

fn copy_dir_recursive(source: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| format!("create {}: {e}", dest.display()))?;
    for entry in fs::read_dir(source).map_err(|e| format!("read dir {}: {e}", source.display()))? {
        let entry = entry.map_err(|e| format!("read dir entry in {}: {e}", source.display()))?;
        let src_path = entry.path();
        let dst_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).map_err(|e| {
                format!("copy {} -> {}: {e}", src_path.display(), dst_path.display())
            })?;
        }
    }
    Ok(())
}

fn assert_bootstrap_emit_core_support(src_dir: &Path) -> Result<(), String> {
    let support_path = src_dir.join("v1_compiler_emit_core_support.rs");
    let support_text = fs::read_to_string(&support_path)
        .map_err(|e| format!("read {}: {e}", support_path.display()))?;
    if !support_text.contains("// Source module: v1.compiler.emit_core_support") {
        return Err(format!(
            "{} must be emitted from v1.compiler.emit_core_support, not postprocessed",
            support_path.display()
        ));
    }
    for file_name in [
        "v1_compiler_emit_go.rs",
        "v1_compiler_emit_python.rs",
        "v1_compiler_emit_rust.rs",
    ] {
        let path = src_dir.join(file_name);
        let text =
            fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        if !text.contains("pub use crate::v1_compiler_emit_core_support::{") {
            return Err(format!(
                "{} must import backend-shared helpers from v1_compiler_emit_core_support",
                path.display()
            ));
        }
    }
    Ok(())
}

fn rustfmt_generated_crate(dir: &Path) -> Result<(), String> {
    let output = Command::new("cargo")
        .arg("fmt")
        .arg("--all")
        .arg("--manifest-path")
        .arg(dir.join("Cargo.toml"))
        .output()
        .map_err(|e| format!("spawn cargo fmt for {}: {e}", dir.display()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "cargo fmt failed for {}:\n{}",
            dir.display(),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

/// A HAND_MAINTAINED file's status relative to what the emitter would produce for it.
/// The regen gate copies hand files verbatim and never diffs them against a fresh emit,
/// so a bad hand-sync -- or an emitter that has quietly caught up -- is invisible. This
/// classification turns that silent exclusion into a countable, located signal.
struct HandVerifyReport {
    /// Emitter produces a candidate byte-identical (after identical rustfmt normalization)
    /// to the committed hand file: the HAND entry is now a dead scaffold and the file
    /// should be flipped into GENERATED_STAGE0_FILES.
    matches: Vec<String>,
    /// Emitter produces a candidate that differs from committed: either the known emitter
    /// gap that justifies hand-maintenance, or an unintended hand-edit. Inspect the diff.
    drifts: Vec<String>,
    /// No emit candidate exists (a pure host-physics pin with no `.dag` source): not
    /// regen-verifiable, covered only by the fresh-crate cargo-green check.
    no_candidate: Vec<String>,
}

/// rustfmt `content` standalone (edition 2021), returning the formatted text. Emitting to
/// stdout leaves no file behind. Returns Err with rustfmt's stderr when the content does
/// not parse -- an incomplete emit candidate is itself drift, so callers treat Err as drift.
fn rustfmt_normalize(content: &str, work_dir: &Path, tag: &str) -> Result<String, String> {
    fs::create_dir_all(work_dir).map_err(|e| format!("create {}: {e}", work_dir.display()))?;
    let path = work_dir.join(format!("{tag}.rs"));
    fs::write(&path, content).map_err(|e| format!("write {}: {e}", path.display()))?;
    let output = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .arg("--emit")
        .arg("stdout")
        .arg(&path)
        .output()
        .map_err(|e| format!("spawn rustfmt for {}: {e}", path.display()))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

/// Verify the hand-maintained files against what the emitter would produce, WITHOUT
/// overwriting them. Both the emit candidate and the committed file are pushed through the
/// same standalone rustfmt so the comparison isolates content drift from formatting noise.
fn verify_hand_maintained_candidates(
    emitted: &HashMap<String, String>,
    stage0_src: &Path,
    work_dir: &Path,
) -> Result<HandVerifyReport, String> {
    let mut report = HandVerifyReport {
        matches: Vec::new(),
        drifts: Vec::new(),
        no_candidate: Vec::new(),
    };
    for file_name in HAND_MAINTAINED_STAGE0_FILES {
        // compile_stage0's emitted map is keyed by output path (e.g. "src/main.rs"),
        // while the roster lists bare filenames; look up both so a genuinely-emitted
        // hand file is not misreported as absent.
        let candidate = emitted
            .get(&format!("src/{file_name}"))
            .or_else(|| emitted.get(*file_name));
        let Some(candidate) = candidate else {
            report.no_candidate.push((*file_name).to_string());
            continue;
        };
        let committed_path = stage0_src.join(file_name);
        let committed = fs::read_to_string(&committed_path)
            .map_err(|e| format!("read committed hand file {}: {e}", committed_path.display()))?;
        let committed_norm = rustfmt_normalize(&committed, work_dir, "committed")
            .map_err(|e| format!("rustfmt committed hand file {file_name}: {e}"))?;
        match rustfmt_normalize(candidate, work_dir, "candidate") {
            Ok(candidate_norm) if candidate_norm == committed_norm => {
                report.matches.push((*file_name).to_string())
            }
            _ => report.drifts.push((*file_name).to_string()),
        }
    }
    // Directory pins (e.g. module_path_index) have no single emit candidate keyed by path.
    for dir_name in HAND_MAINTAINED_STAGE0_DIRS {
        report.no_candidate.push(format!("{dir_name}/ (dir pin)"));
    }
    Ok(report)
}

/// Print the hand-maintained verification report. MATCH is the actionable signal (flip to
/// GENERATED); DRIFT is expected for files parked on a known emitter gap; NO CANDIDATE marks
/// the terminal-kernel host-physics pins that regen cannot cross-check.
fn print_hand_verify_report(report: &HandVerifyReport) {
    println!(
        "regen_stage0 hand-maintained verification: {} match / {} drift / {} no-candidate",
        report.matches.len(),
        report.drifts.len(),
        report.no_candidate.len()
    );
    if !report.matches.is_empty() {
        println!(
            "  MATCH (emitter reproduces these -- flip to GENERATED_STAGE0_FILES): {}",
            report.matches.join(", ")
        );
    }
    if !report.drifts.is_empty() {
        println!(
            "  DRIFT (emit candidate differs -- known emitter gap or hand-edit, inspect): {}",
            report.drifts.join(", ")
        );
    }
    if !report.no_candidate.is_empty() {
        println!(
            "  NO CANDIDATE (not in the fresh emit closure -- a host-physics pin with no \
             .dag source, or a module the closure does not reach; cargo-green only): {}",
            report.no_candidate.join(", ")
        );
    }
}

fn rustfmt_workspace(manifest_dir: &Path) -> Result<(), String> {
    let output = Command::new("cargo")
        .arg("fmt")
        .arg("--all")
        .arg("--manifest-path")
        .arg(manifest_dir.join("Cargo.toml"))
        .output()
        .map_err(|e| format!("spawn cargo fmt: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "cargo fmt failed after writing stage0:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn stage0_split_crate_boundaries(workspace: &Path) -> Vec<(PathBuf, String)> {
    v1_compiler::v1_compiler_stage0_crates::stage0_crate_boundary_files()
        .iter()
        .map(|file| (workspace.join(&file.path), file.content.clone()))
        .collect()
}

fn verify_stage0_split_crate_boundaries(workspace: &Path) -> Result<(), String> {
    let mut mismatches = Vec::new();
    for (path, expected) in stage0_split_crate_boundaries(workspace) {
        let committed =
            fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        if committed != expected {
            mismatches.push(display_source_path(&path, workspace));
        }
    }
    if mismatches.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Stage0 split crate boundary files are stale. Run `cargo run -p v1-compiler --bin regen_stage0` to regenerate. Changed file(s): {}",
            mismatches.join(", ")
        ))
    }
}

fn write_stage0_split_crate_boundaries(workspace: &Path) -> Result<(), String> {
    for (path, contents) in stage0_split_crate_boundaries(workspace) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        fs::write(&path, contents).map_err(|e| format!("write {}: {e}", path.display()))?;
    }
    Ok(())
}

const WORKSPACE_MEMBERS_REGEN_HINT: &str =
    "Run `cargo run -p v1-compiler --bin regen_stage0` to regenerate.";

fn workspace_members_markers_and_region() -> (String, String, String) {
    (
        v1_compiler::v1_compiler_workspace_members::workspace_members_region_begin_marker(),
        v1_compiler::v1_compiler_workspace_members::workspace_members_region_end_marker(),
        v1_compiler::v1_compiler_workspace_members::stage0_workspace_member_region(),
    )
}

fn locate_workspace_members_region<'a>(
    content: &'a str,
    begin_marker: &str,
    end_marker: &str,
) -> Result<(&'a str, &'a str, &'a str), String> {
    let begin = content.find(begin_marker).ok_or_else(|| {
        format!("top-level Cargo.toml is missing the generated-members BEGIN marker. {WORKSPACE_MEMBERS_REGEN_HINT}")
    })?;
    let end = content.find(end_marker).ok_or_else(|| {
        format!("top-level Cargo.toml is missing the generated-members END marker. {WORKSPACE_MEMBERS_REGEN_HINT}")
    })?;
    if begin >= end {
        return Err(
            "generated-members markers in top-level Cargo.toml are out of order".to_string(),
        );
    }
    let after_begin_marker = begin + begin_marker.len();
    let body_start = match content[after_begin_marker..].find('\n') {
        Some(rel) => after_begin_marker + rel + 1,
        None => return Err(
            "generated-members BEGIN marker in top-level Cargo.toml is not followed by a newline"
                .to_string(),
        ),
    };
    Ok((
        &content[..body_start],
        &content[body_start..end],
        &content[end..],
    ))
}

fn verify_workspace_members(workspace: &Path) -> Result<(), String> {
    let (begin, end, region) = workspace_members_markers_and_region();
    let path = workspace.join("Cargo.toml");
    let content = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let (_, body, _) = locate_workspace_members_region(&content, &begin, &end)?;
    if body == format!("{region}\n") {
        Ok(())
    } else {
        Err(format!(
            "Top-level Cargo.toml `[workspace] members` generated region is stale. {WORKSPACE_MEMBERS_REGEN_HINT}"
        ))
    }
}

fn write_workspace_members(workspace: &Path) -> Result<(), String> {
    let (begin, end, region) = workspace_members_markers_and_region();
    let path = workspace.join("Cargo.toml");
    let content = fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let (prefix, _, suffix) = locate_workspace_members_region(&content, &begin, &end)?;
    let updated = format!("{prefix}{region}\n{suffix}");
    if updated != content {
        fs::write(&path, updated).map_err(|e| format!("write {}: {e}", path.display()))?;
    }
    Ok(())
}

fn assert_output_set_matches_registry(
    committed_src: &Path,
    fresh_src: &Path,
) -> Result<(), String> {
    let registered: BTreeSet<&str> = GENERATED_STAGE0_FILES.iter().copied().collect();
    let hand_maintained: BTreeSet<&str> = HAND_MAINTAINED_STAGE0_FILES.iter().copied().collect();
    let fresh_files = direct_rs_file_names(fresh_src)?;

    let unregistered_fresh: Vec<_> = fresh_files
        .iter()
        .filter(|file| {
            !registered.contains(file.as_str()) && !hand_maintained.contains(file.as_str())
        })
        .cloned()
        .collect();
    if !unregistered_fresh.is_empty() {
        return Err(format!(
            "fresh self-compile emitted unregistered stage0 file(s): {}\n\
             Add generated files to GENERATED_STAGE0_FILES or mark hand-maintained files explicitly.",
            unregistered_fresh.join(", ")
        ));
    }

    let missing_fresh: Vec<_> = registered
        .iter()
        .filter(|file| !fresh_files.contains(**file))
        .copied()
        .collect();
    if !missing_fresh.is_empty() {
        return Err(format!(
            "GENERATED_STAGE0_FILES contains file(s) missing from fresh self-compile: {}",
            missing_fresh.join(", ")
        ));
    }

    let committed_files = direct_rs_file_names(committed_src)?;
    let mut unregistered_committed_generated = Vec::new();
    for file_name in committed_files {
        if registered.contains(file_name.as_str()) || hand_maintained.contains(file_name.as_str()) {
            continue;
        }
        let path = committed_src.join(&file_name);
        let text =
            fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
        if text.contains("Generated by v1 compiler -- do not edit")
            || text.contains("Generated by the v2 compiler -- do not edit")
        {
            unregistered_committed_generated.push(file_name);
        }
    }
    if !unregistered_committed_generated.is_empty() {
        return Err(format!(
            "committed generated stage0 file(s) are not registered: {}\n\
             Add them to GENERATED_STAGE0_FILES or remove the stale committed outputs.",
            unregistered_committed_generated.join(", ")
        ));
    }

    Ok(())
}

fn direct_rs_file_names(dir: &Path) -> Result<BTreeSet<String>, String> {
    let mut files = BTreeSet::new();
    let entries = fs::read_dir(dir)
        .map_err(|e| format!("read dir {}: {e}", dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("read dir entry in {}: {e}", dir.display()))?;
    for entry in entries {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            let file_name = path
                .file_name()
                .ok_or_else(|| format!("stage0 file has no name: {}", path.display()))?
                .to_string_lossy()
                .to_string();
            files.insert(file_name);
        }
    }
    Ok(files)
}

fn verify_stage0_matches(committed_src: &Path, fresh_src: &Path) -> Result<(), String> {
    let mut mismatches = Vec::new();
    for file_name in GENERATED_STAGE0_FILES {
        let committed = committed_src.join(file_name);
        let fresh = fresh_src.join(file_name);
        let committed_text = fs::read_to_string(&committed)
            .map_err(|e| format!("read committed {}: {e}", committed.display()))?;
        let fresh_text = fs::read_to_string(&fresh)
            .map_err(|e| format!("read fresh {}: {e}", fresh.display()))?;
        if committed_text != fresh_text {
            mismatches.push((*file_name).to_string());
        }
    }
    if mismatches.is_empty() {
        return Ok(());
    }

    let mut message = format!(
        "Stage0 is stale: {} generated file(s) differ from fresh self-compile.\n\
         Run `cargo run -p v1-compiler --bin regen_stage0` to regenerate.\n\
         Changed generated file(s): {}\n",
        mismatches.len(),
        mismatches.join(", ")
    );
    if let Some(first) = mismatches.first() {
        message.push_str(&diff_hint(
            &committed_src.join(first),
            &fresh_src.join(first),
        ));
    }
    Err(message)
}

fn diff_hint(committed: &Path, fresh: &Path) -> String {
    let output = Command::new("diff")
        .arg("-u")
        .arg(committed)
        .arg(fresh)
        .output();
    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let truncated = &stdout[..stdout.len().min(4000)];
            format!(
                "First mismatch diff ({})\n{}",
                committed.display(),
                truncated
            )
        }
        Err(e) => format!(
            "Could not produce diff hint for {}: {e}",
            committed.display()
        ),
    }
}

fn changed_registered_outputs(
    fresh_src: &Path,
    committed_src: &Path,
) -> Result<Vec<String>, String> {
    let mut changed = Vec::new();
    for file_name in GENERATED_STAGE0_FILES {
        let fresh = fresh_src.join(file_name);
        let committed = committed_src.join(file_name);
        let fresh_text = fs::read_to_string(&fresh)
            .map_err(|e| format!("read fresh {}: {e}", fresh.display()))?;
        let committed_text = match fs::read_to_string(&committed) {
            Ok(text) => text,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                changed.push((*file_name).to_string());
                continue;
            }
            Err(e) => return Err(format!("read committed {}: {e}", committed.display())),
        };
        if fresh_text != committed_text {
            changed.push((*file_name).to_string());
        }
    }
    Ok(changed)
}

fn write_registered_outputs(fresh_src: &Path, committed_src: &Path) -> Result<(), String> {
    for file_name in GENERATED_STAGE0_FILES {
        let fresh = fresh_src.join(file_name);
        let committed = committed_src.join(file_name);
        fs::copy(&fresh, &committed)
            .map_err(|e| format!("copy {} to {}: {e}", fresh.display(), committed.display()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn generated_registry_excludes_hand_maintained_support_files() {
        assert_registry_is_partitioned().expect("stage0 registry partition");
        for file_name in HAND_MAINTAINED_STAGE0_FILES {
            assert!(
                !GENERATED_STAGE0_FILES.contains(file_name),
                "{file_name} must not be touched by regen_stage0"
            );
        }
    }

    #[test]
    fn output_set_rejects_unregistered_fresh_file() {
        let committed = temp_test_dir("committed");
        let fresh = temp_test_dir("fresh");
        seed_registered_files(&committed, "// Generated by v1 compiler -- do not edit\n");
        seed_registered_files(&fresh, "// Generated by v1 compiler -- do not edit\n");
        fs::write(
            fresh.join("new_generated_file.rs"),
            "// Generated by v1 compiler -- do not edit\n",
        )
        .expect("write unregistered fresh file");

        let err = assert_output_set_matches_registry(&committed, &fresh)
            .expect_err("unregistered fresh file must fail");
        assert!(err.contains("fresh self-compile emitted unregistered stage0 file"));

        let _ = fs::remove_dir_all(committed);
        let _ = fs::remove_dir_all(fresh);
    }

    #[test]
    fn output_set_rejects_unregistered_committed_generated_file() {
        let committed = temp_test_dir("committed");
        let fresh = temp_test_dir("fresh");
        seed_registered_files(&committed, "// Generated by v1 compiler -- do not edit\n");
        seed_registered_files(&fresh, "// Generated by v1 compiler -- do not edit\n");
        fs::write(
            committed.join("old_generated_file.rs"),
            "// Generated by v1 compiler -- do not edit\n",
        )
        .expect("write stale committed generated file");

        let err = assert_output_set_matches_registry(&committed, &fresh)
            .expect_err("unregistered committed generated file must fail");
        assert!(err.contains("committed generated stage0 file"));

        let _ = fs::remove_dir_all(committed);
        let _ = fs::remove_dir_all(fresh);
    }

    #[test]
    fn changed_registered_outputs_lists_only_mismatched_generated_files() {
        let committed = temp_test_dir("committed");
        let fresh = temp_test_dir("fresh");
        seed_registered_files(&committed, "same\n");
        seed_registered_files(&fresh, "same\n");
        fs::write(fresh.join(GENERATED_STAGE0_FILES[0]), "changed\n")
            .expect("write changed generated file");

        let changed =
            changed_registered_outputs(&fresh, &committed).expect("changed output receipt");
        assert_eq!(changed, vec![GENERATED_STAGE0_FILES[0].to_string()]);

        let _ = fs::remove_dir_all(committed);
        let _ = fs::remove_dir_all(fresh);
    }

    #[test]
    fn cargo_invalidation_receipt_hashes_workspace_and_stage0_manifests() {
        let workspace = temp_test_dir("workspace");
        let manifest_dir = workspace.join("src").join("v1").join("stage0");
        fs::create_dir_all(&manifest_dir).expect("create manifest dir");
        fs::write(workspace.join("Cargo.toml"), "[workspace]\n").expect("write workspace toml");
        fs::write(workspace.join("Cargo.lock"), "# lock\n").expect("write cargo lock");
        fs::write(
            manifest_dir.join("Cargo.toml"),
            "[package]\nname = \"v1\"\n",
        )
        .expect("write stage0 toml");

        let receipt = cargo_invalidation_receipt(&workspace, &manifest_dir).expect("cargo receipt");
        assert_eq!(
            receipt.strategy,
            "content_hashes_for_workspace_manifest_lockfile_and_stage0_manifest"
        );
        let paths: Vec<_> = receipt
            .inputs
            .iter()
            .map(|input| input.path.as_str())
            .collect();
        assert_eq!(
            paths,
            vec!["Cargo.toml", "Cargo.lock", "src/v1/stage0/Cargo.toml"]
        );
        assert!(receipt
            .inputs
            .iter()
            .all(|input| input.content_hash.starts_with("fnv1a64:") && input.len_bytes > 0));

        let _ = fs::remove_dir_all(workspace);
    }

    #[test]
    fn compile_stage0_uses_shared_refusal_authority() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/bin/regen_stage0.rs");
        let source = fs::read_to_string(path).expect("read regen_stage0.rs");
        assert!(source.contains("stage0_self_compile_refusal_message"));
    }

    #[test]
    fn bootstrap_timing_receipt_json_pins_v2_schema() {
        let workspace = temp_test_dir("receipt-workspace");
        let manifest_dir = workspace.join("src").join("v2").join("stage0");
        let receipt_path = workspace.join("receipt.json");
        fs::create_dir_all(&manifest_dir).expect("create manifest dir");
        fs::write(workspace.join("Cargo.toml"), "[workspace]\n").expect("write workspace toml");
        fs::write(workspace.join("Cargo.lock"), "# lock\n").expect("write cargo lock");
        fs::write(manifest_dir.join("Cargo.toml"), "[package]\n").expect("write stage0 toml");

        write_bootstrap_timing_receipt(BootstrapTimingReceiptInput {
            path: &receipt_path,
            workspace: &workspace,
            manifest_dir: &manifest_dir,
            verify_only: true,
            status: "completed",
            generated_file_count: 2,
            emitted_file_count: 3,
            phases: vec![BootstrapTimingPhase {
                name: "compile_stage0".to_string(),
                elapsed_ms: 7,
            }],
            elapsed_ms: 9,
            changed_generated_files: Vec::new(),
        })
        .expect("write receipt");

        let body = fs::read_to_string(&receipt_path).expect("read receipt");
        let json: serde_json::Value = serde_json::from_str(&body).expect("parse receipt");
        assert_eq!(json["schema"], BOOTSTRAP_TIMING_RECEIPT_SCHEMA);
        assert_eq!(json["version"], BOOTSTRAP_TIMING_RECEIPT_VERSION);
        assert_eq!(json["subject"], "v1_regen_stage0");
        assert_eq!(json["mode"], "verify");
        assert_eq!(
            json["cargo_invalidation"]["inputs"]
                .as_array()
                .unwrap()
                .len(),
            3
        );

        let _ = fs::remove_dir_all(workspace);
    }

    fn seed_registered_files(dir: &Path, contents: &str) {
        fs::create_dir_all(dir).expect("create temp stage0 dir");
        for file_name in GENERATED_STAGE0_FILES {
            fs::write(dir.join(file_name), contents).expect("write registered generated file");
        }
    }

    fn temp_test_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        env::temp_dir().join(format!("regen-stage0-test-{label}-{unique}"))
    }
}
