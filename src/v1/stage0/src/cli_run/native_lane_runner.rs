//! The required-v2-native lane's host harness (lane authority:
//! `gunbc.witness_v2_native_route`; roster row: `gunbc.required_ci_phase_roster`
//! `V2NativePhase`).
//!
//! THE SUBJECT IS THE ROUTE, NOT THE MODEL. The `.dag` authority owns the admission contract;
//! this module is the harness that MINTS the receipt the contract admits or refuses: it derives
//! the `v2.test.*` universe with the floor's own discovery producer, prepares the emitted-native
//! compiler through the emit-compile phase's one crate writer, invokes that binary by explicit
//! path over the universe plus the controls, withdraws the old-route CLI for the duration of
//! every native spawn, and binds the observed verdicts into a `NativeRouteReceipt` that is then
//! admitted — or refused — by evaluating `native_route_admission` itself. Nothing here decides
//! admission in Rust: the host observes, the substrate judges.
//!
//! WHAT THE OLD-ROUTE CONTROL PROVES HERE, AND WHAT IT DOES NOT. The lane's verdicts are parsed
//! from the spawned emitted binary's stdout, so the execution route is a process boundary, not a
//! call convention. Withdrawing the seed's `gunbc` binary for the spawn window proves no
//! subprocess fallback to the seed CLI occurred; it does not prove this process never
//! interpreted a test — that discipline is the code's own (the interpreter contexts below
//! evaluate ONLY the discovery producer and the admission authority, never a universe test),
//! and the receipt's route identity (binary path + content hash) is what admission checks.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

use crate::v1_interpreter::{self, str_value, Value};

/// The compiler entry whose closure becomes the lane's emitted-native compiler. Its
/// `compiler_pipeline_entry` is `SourceRootEvalDriver`, so the emitted crate's `main.rs` is the
/// whole-source-root Eval driver this lane exists to route through.
const NATIVE_COMPILE_ENTRY: &str = "src/v2/compiler/00_compile.dag";

/// The universe prefix — the same `v2.test.` the required floor gates on. The derivation below
/// filters the floor discovery producer's rows to it; the admission authority independently
/// checks prefix closure, so a drift between the two is a red, not a silent widening.
const UNIVERSE_PREFIX: &str = "v2.test.";

/// The live-verdict control pair (v2.native_lane_fixture.control): plain `fn` declarations
/// outside the universe prefix, so floor discovery enrolls no rows for them and this harness
/// names them to the binary explicitly, beside the derived universe.
const CONTROL_MODULE: &str = "v2.native_lane_fixture.control";
const FALSE_CONTROL_DECL: &str = "native_lane_false_control";
const TRUE_CONTROL_DECL: &str = "native_lane_true_control";

/// The malformed control: deliberately unterminating bytes any honest front-end must refuse at
/// tokenize. THE COMMITTED CARRIER IS NOT A `.dag` FILE — the bytes are not a dag program, and
/// carrying the extension put them in the changed-witness observation's parse path, where the
/// deliberate tokenize failure refused the whole floor lane (run 34471447387). The harness
/// materializes the specimen as `poison.dag` under the lane's scratch root, and only that
/// materialized copy ever enters an ingest — the control run's own source root.
const MALFORMED_SPECIMEN_COMMITTED: &str = "fixtures/native_lane_malformed/poison.dag.poisoned";
/// The control run's scratch source root and the materialized specimen's path under it,
/// workspace-relative like every path this harness hands the binary (its working directory is
/// the workspace root). The receipt records the observed refusal path; the admission authority
/// requires it non-empty, and this harness requires it to name the materialized specimen.
const MALFORMED_CONTROL_ROOT: &str = "target/v2-native-lane/malformed-control-root";
const MALFORMED_MATERIALIZED_PATH: &str = "target/v2-native-lane/malformed-control-root/poison.dag";

/// The admission authority's entry file, for the interpreter context that judges the receipt.
const NATIVE_ROUTE_AUTHORITY_ENTRY: &str = "dag/gunbc/witness/v2_native_route.dag";

/// One observed verdict row, decoded from the emitted binary's stdout at the same grain the
/// receipt carries: qualified identity plus the driver's own verdict vocabulary.
struct NativeObservation {
    module: String,
    declaration: String,
    verdict: NativeVerdictObserved,
}

enum NativeVerdictObserved {
    Passed,
    ReturnedFalse,
    ReturnedOther,
    Refused { stage: String, reason: String },
}

/// One per-file front-end refusal the emitted binary's collecting context fold observed.
struct NativeFileRefusalObserved {
    path: String,
    head_reason: String,
    fatal_reason: String,
}

/// The emitted compiler, prepared: where the binary is, what its bytes are, and the identity of
/// the closure it was emitted from.
/// Seed mirror of `gunbc.witness_v2_native_route` `native_lane_emitted_compiler_rustflags`.
const NATIVE_LANE_EMITTED_COMPILER_RUSTFLAGS: &str = "-D warnings";

struct EmittedPreparation {
    binary_path: PathBuf,
    binary_identity: String,
    closure_identity: String,
    seed_identity: String,
    cargo_command: String,
    effective_rustflags: String,
    rustc_identity: String,
    cargo_exit_status: i32,
    warning_count: u32,
}

fn sha256_file(path: &Path) -> Result<String, String> {
    use sha2::Digest;
    let bytes = std::fs::read(path).map_err(|e| {
        format!(
            "could not read {} for its content identity: {e}",
            path.display()
        )
    })?;
    Ok(format!("{:x}", sha2::Sha256::digest(&bytes)))
}

/// The emitted closure's identity: one digest over the crate's emitted sources, path and
/// content in sorted order, so the receipt names WHAT was compiled, not merely that something
/// was.
fn emitted_closure_identity(crate_dir: &Path) -> Result<String, String> {
    use sha2::Digest;
    let src_dir = crate_dir.join("src");
    let mut files: Vec<PathBuf> = std::fs::read_dir(&src_dir)
        .map_err(|e| format!("could not list {}: {e}", src_dir.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.extension().map(|ext| ext == "rs").unwrap_or(false))
        .collect();
    files.sort();
    let mut hasher = sha2::Sha256::new();
    for path in &files {
        let bytes =
            std::fs::read(path).map_err(|e| format!("could not read {}: {e}", path.display()))?;
        hasher.update(
            path.file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default()
                .as_bytes(),
        );
        hasher.update(&bytes);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// The derived universe plus BOTH directions of the module index's path relation. The
/// discovery-row join needs path → module (producer rows key on the entry path); the context
/// reclassification needs module → path (observations key on the module). The module → path
/// direction carries no collision refusal of its own: `ModuleSourceIndex` is keyed by module
/// and its construction already refuses two files declaring one module, so the relation is a
/// function by construction — a second check here would have an unauthorable RED.
struct NativeUniverseDerivation {
    universe: Vec<(String, String)>,
    module_for_path: HashMap<String, String>,
    path_for_module: HashMap<String, String>,
}

/// THE UNIVERSE IS DERIVED BY THE FLOOR'S OWN PRODUCER, NOT BY A SECOND SCAN. The production
/// required floor folds `discover_floor_rows_for_source` over the full module inventory and
/// finalizes with `floor_discovery_finalize_source_outcomes`; this derivation makes the same
/// two calls over the same inventory and keeps the rows whose AUTHORED module (read off the
/// `module` header by the index, never path-derived) carries the universe prefix. A second
/// discovery beside the producer would be free to disagree with the floor about what a test is.
fn derive_native_universe(source_roots: &[String]) -> Result<NativeUniverseDerivation, String> {
    let (graph, indices) =
        super::resolve_workspace_entry(source_roots, super::FLOOR_DISCOVERY_PRODUCER_ENTRY)?;
    let ctx = super::make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let index = super::try_build_module_index(source_roots)?;
    let mut inventory: Vec<(String, String, String)> = index
        .iter()
        .map(|(module_path, source)| {
            (
                source.path.replace('\\', "/"),
                module_path.clone(),
                source.content.clone(),
            )
        })
        .collect();
    inventory.sort();
    let module_for_path: HashMap<String, String> = inventory
        .iter()
        .map(|(path, module, _)| (path.clone(), module.clone()))
        .collect();
    let path_for_module: HashMap<String, String> = inventory
        .iter()
        .map(|(path, module, _)| (module.clone(), path.clone()))
        .collect();
    eprintln!(
        "required-ci: v2-native deriving the universe — {} sources through the floor discovery producer",
        inventory.len()
    );
    let mut outcomes: Vec<Value> = Vec::with_capacity(inventory.len());
    for (path, _, content) in &inventory {
        let args = [
            (Some("repo_path".to_string()), str_value(path)),
            (Some("content".to_string()), str_value(content)),
        ];
        let outcome = v1_interpreter::run_in_context_with_args(
            &ctx,
            "v2.workflow.floor_discovery_producer.discover_floor_rows_for_source",
            &args,
            false,
        )
        .map_err(|e| {
            format!(
                "V2-NATIVE REFUSAL cause=UniverseDerivationUnevaluable source={path} — \
                 discover_floor_rows_for_source: {e}"
            )
        })?;
        outcomes.push(outcome);
    }
    let finalized = v1_interpreter::run_in_context_with_args(
        &ctx,
        "v2.workflow.floor_discovery_producer.floor_discovery_finalize_source_outcomes",
        &[(
            Some("outcomes".to_string()),
            super::list_value_from_vec(outcomes),
        )],
        false,
    )
    .map_err(|e| {
        format!(
            "V2-NATIVE REFUSAL cause=UniverseDerivationUnevaluable — \
             floor_discovery_finalize_source_outcomes: {e}"
        )
    })?;
    let rows =
        super::parse_floor_discovery_producer_result(&ctx, &finalized).map_err(|reason| {
            format!("V2-NATIVE REFUSAL cause=UniverseDerivationRefused — {reason}")
        })?;
    let mut universe: Vec<(String, String)> = Vec::new();
    for row in &rows {
        let Some(module) = module_for_path.get(row.entry.as_str()) else {
            return Err(format!(
                "V2-NATIVE REFUSAL cause=UniverseEntryOutsideSubject entry={} — the discovery \
                 authority enrolled an entry the module index does not hold",
                row.entry
            ));
        };
        if module.starts_with(UNIVERSE_PREFIX) {
            universe.push((module.clone(), row.function.clone()));
        }
    }
    // SORTED FOR CANONICAL TRANSPORT, NEVER DEDUPLICATED. A duplicated identity is a harness
    // defect the admission authority refuses by name (UniverseDuplicatesPresent); normalizing
    // it away here would erase the evidence before the authority can judge it.
    universe.sort();
    if universe.is_empty() {
        return Err(
            "V2-NATIVE REFUSAL cause=UniverseUnderived — the floor discovery producer enrolled \
             no v2.test.* rows over the full module inventory"
                .to_string(),
        );
    }
    Ok(NativeUniverseDerivation {
        universe,
        module_for_path,
        path_for_module,
    })
}

/// PREPARATION IS THE EMIT-COMPILE PHASE'S OWN MACHINERY, REUSED. The same emission entry
/// point, the same crate writer, the same cargo invocation the required emit-compile probes use
/// — a second emit-or-build path beside them would be free to disagree about what "the emitted
/// compiler" is. The seed is used exactly once here, in-process, to emit; the receipt records
/// the seed's identity honestly, and this job claims no native bootstrap.
fn prepare_emitted_compiler(source_roots: &[String]) -> Result<EmittedPreparation, String> {
    let workspace = super::process_workspace_root();
    // The probe root follows the declared execution environment (per-job runner temp in CI,
    // host temp locally) — the selection's authority and its receipt live beside the required
    // phase's own root policy in `emitted_closure_compile_host`.
    let probe_root = super::lane_emit_compile_probe_root();
    eprintln!("required-ci: v2-native emitting {NATIVE_COMPILE_ENTRY} (seed, in-process)");
    let run = super::compile_entry_emission(
        source_roots,
        NATIVE_COMPILE_ENTRY,
        true,
        crate::v1_compiler_artifact::RenderTarget::Rust,
    );
    match &run.disposition {
        super::CompileDisposition::Completed { .. } => {}
        super::CompileDisposition::Refused { phase, cause } => {
            return Err(format!(
                "V2-NATIVE REFUSAL cause=EmissionRefused stage={phase} — {cause}"
            ))
        }
        super::CompileDisposition::NotExecuted {
            earlier_phase,
            cause,
        } => {
            return Err(format!(
                "V2-NATIVE REFUSAL cause=EmissionRefused stage={earlier_phase} — {cause}"
            ))
        }
    }
    let (crate_dir, written) = super::emitted_closure_compile_host::write_probe_crate(
        &run,
        &probe_root,
        NATIVE_COMPILE_ENTRY,
    )
    .map_err(|cause| format!("V2-NATIVE REFUSAL cause=EmittedCrateNotWritten — {cause}"))?;
    let closure_identity = emitted_closure_identity(&crate_dir)?;
    // THE BUILD'S PEAK MUST NOT STACK ON THE EMISSION'S RETAINED ARENA. The emission's resolved
    // graph died inside `compile_entry_emission` and the emitted file texts die with `run` here,
    // but glibc retains the freed arena — and the cargo build below needs gigabytes beside this
    // process. Measured: the lane's first run held ~15GiB RSS into the build and was SIGKILLed
    // (rc=137, no diagnostic). Drop, trim, and report in the same motion — the floor runner's
    // full-inventory release is the pattern, and a trim that cannot release live memory doubles
    // as the measurement that nothing here is still held.
    drop(run);
    let rss_before_kb = super::current_rss_bytes().map(|b| b / 1024);
    let trim_reclaimed_kb = super::trim_retained_heap();
    let rss_after_kb = super::current_rss_bytes().map(|b| b / 1024);
    eprintln!(
        "required-ci: v2-native emitted {written} files into {} (closure {closure_identity}); \
         emission arena released (rss_kb_before={rss_before_kb:?} trim_reclaimed_kb={trim_reclaimed_kb:?} \
         rss_kb_after={rss_after_kb:?}); cargo build",
        crate_dir.display()
    );
    let cargo_run = super::emitted_closure_compile_host::run_cargo_recorded(
        &crate_dir,
        &workspace,
        "v2_native_lane_carries_no_mutation_probe",
        Some(NATIVE_LANE_EMITTED_COMPILER_RUSTFLAGS),
    );
    let cargo_exit_status = match &cargo_run.verdict {
        super::emitted_closure_compile_host::CargoVerdict::Completed { status, .. } => *status,
        _ => -1,
    };
    if !super::emitted_closure_compile_host::cargo_verdict_compiled(&cargo_run.verdict) {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=EmittedCompilerBuildFailed — {}",
            super::emitted_closure_compile_host::cargo_verdict_summary(&cargo_run.verdict)
        ));
    }
    let rustc_identity = rustc_verbose_identity();
    eprintln!(
        "required-ci: v2-native cargo_command={} effective_RUSTFLAGS={} rustc_identity={} \
         cargo_exit_status={cargo_exit_status} warning_count={}",
        cargo_run.command, cargo_run.rustflags, rustc_identity, cargo_run.warning_count
    );
    let binary_path = workspace.join("target").join("release").join(
        super::emitted_closure_compile_host::probe_package_name(NATIVE_COMPILE_ENTRY),
    );
    if !binary_path.is_file() {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=EmittedCompilerAbsent — cargo reported success but {} is \
             not a file",
            binary_path.display()
        ));
    }
    let binary_identity = sha256_file(&binary_path)?;
    let seed_identity = sha256_file(&std::env::current_exe().map_err(|e| {
        format!("V2-NATIVE REFUSAL cause=SeedIdentityUnreadable — current_exe: {e}")
    })?)?;
    eprintln!(
        "required-ci: v2-native emitted compiler at {} (sha256 {binary_identity})",
        binary_path.display()
    );
    Ok(EmittedPreparation {
        binary_path,
        binary_identity,
        closure_identity,
        seed_identity,
        cargo_command: cargo_run.command,
        effective_rustflags: cargo_run.rustflags,
        rustc_identity,
        cargo_exit_status,
        warning_count: cargo_run.warning_count,
    })
}

fn rustc_verbose_identity() -> String {
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    match Command::new(&rustc).arg("-vV").output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .unwrap_or("rustc -vV produced no lines")
            .to_string(),
        Err(e) => format!("rustc -vV failed: {e}"),
    }
}

/// THE OLD-ROUTE WITHDRAWAL, AS A GUARD SO A REFUSAL PATH CANNOT SKIP THE RESTORE. The seed's
/// `gunbc` binary — the old compiler/interpreter route's CLI — is renamed out of the way before
/// the first native spawn and restored when the guard drops, on every exit path. The receipt
/// records the withdrawn executable's path; the window is what admission's control clause names.
struct OldRouteWithdrawalGuard {
    original: PathBuf,
    withdrawn: PathBuf,
}

fn withdraw_old_route(workspace: &Path) -> Result<OldRouteWithdrawalGuard, String> {
    let original = workspace.join("target").join("release").join("gunbc");
    let withdrawn = workspace
        .join("target")
        .join("release")
        .join("gunbc.withdrawn-v2-native");
    if !original.is_file() {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=OldRouteWithdrawalImpossible — {} is not a file; the \
             control cannot withdraw what is absent",
            original.display()
        ));
    }
    std::fs::rename(&original, &withdrawn).map_err(|e| {
        format!(
            "V2-NATIVE REFUSAL cause=OldRouteWithdrawalImpossible — renaming {}: {e}",
            original.display()
        )
    })?;
    eprintln!(
        "required-ci: v2-native old route withdrawn ({} moved aside for the native spawns)",
        original.display()
    );
    Ok(OldRouteWithdrawalGuard {
        original,
        withdrawn,
    })
}

impl Drop for OldRouteWithdrawalGuard {
    fn drop(&mut self) {
        if let Err(e) = std::fs::rename(&self.withdrawn, &self.original) {
            eprintln!(
                "required-ci: v2-native WARNING — restoring the old route failed ({}): {e}",
                self.original.display()
            );
        }
    }
}

/// The universe file the emitted binary reads: `module<TAB>declaration` per line, the derived
/// universe followed by the two live-verdict controls (which the harness names explicitly —
/// they are controls, never universe members, and the receipt's population join would refuse
/// them as foreign rows if they reached it).
fn write_universe_file(path: &Path, universe: &[(String, String)]) -> Result<(), String> {
    let mut text = String::new();
    for (module, declaration) in universe {
        text.push_str(module);
        text.push('\t');
        text.push_str(declaration);
        text.push('\n');
    }
    for declaration in [FALSE_CONTROL_DECL, TRUE_CONTROL_DECL] {
        text.push_str(CONTROL_MODULE);
        text.push('\t');
        text.push_str(declaration);
        text.push('\n');
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("creating {}: {e}", parent.display()))?;
    }
    std::fs::write(path, text).map_err(|e| format!("writing {}: {e}", path.display()))
}

/// One controls-only universe file for the malformed-specimen run: the poisoned source root is
/// the subject. The control rows stay named because the driver refuses an empty universe file;
/// under the control run's fixture-only roots they resolve to nothing, and their prepare-stage
/// refusal observations are not consumed — the run's only consumed output is the poison path's
/// per-file refusal.
fn write_control_universe_file(path: &Path) -> Result<(), String> {
    write_universe_file(path, &[])
}

/// Build the malformed control's scratch source root: exactly the committed specimen,
/// materialized under its `.dag` name. The root is REMOVED first — a scratch root that
/// accumulates whatever earlier runs left behind would let a stale second file into the
/// control's ingest, and the control's whole value is that its root holds exactly one poisoned
/// file. Returns the workspace-relative root the binary is spawned with.
fn materialize_malformed_specimen(workspace: &Path) -> Result<String, String> {
    let specimen = workspace.join(MALFORMED_SPECIMEN_COMMITTED);
    let bytes = std::fs::read(&specimen).map_err(|e| {
        format!(
            "V2-NATIVE REFUSAL cause=MalformedSpecimenUnreadable — reading {}: {e}",
            specimen.display()
        )
    })?;
    let root = workspace.join(MALFORMED_CONTROL_ROOT);
    if root.exists() {
        std::fs::remove_dir_all(&root).map_err(|e| format!("clearing {}: {e}", root.display()))?;
    }
    let materialized = workspace.join(MALFORMED_MATERIALIZED_PATH);
    let parent = materialized
        .parent()
        .expect("the materialized path names a file under the control root");
    std::fs::create_dir_all(parent).map_err(|e| format!("creating {}: {e}", parent.display()))?;
    std::fs::write(&materialized, bytes)
        .map_err(|e| format!("writing {}: {e}", materialized.display()))?;
    Ok(MALFORMED_CONTROL_ROOT.to_string())
}

/// The spawned run's decoded observations: one row per printed verdict, the per-file refusals
/// the context fold collected, and the terminal marker's row count. A line that is neither an
/// observation, nor a file refusal, nor the terminal marker refuses the parse — the binary's
/// stdout is a receipt surface, and an unrecognized line on it is a harness defect, not noise.
struct NativeRunOutput {
    observations: Vec<NativeObservation>,
    file_refusals: Vec<NativeFileRefusalObserved>,
    terminal_rows: u64,
}

fn decode_verdict_json(value: &serde_json::Value) -> Result<NativeVerdictObserved, String> {
    let variant = value
        .get("_variant")
        .and_then(|v| v.as_str())
        .ok_or_else(|| format!("verdict carries no _variant tag: {value}"))?;
    match variant {
        "NativeTestPassed" => Ok(NativeVerdictObserved::Passed),
        "NativeTestReturnedFalse" => Ok(NativeVerdictObserved::ReturnedFalse),
        "NativeTestReturnedOther" => Ok(NativeVerdictObserved::ReturnedOther),
        "NativeTestRefused" => {
            let stage = value
                .get("stage")
                .and_then(|s| s.get("_variant"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("refusal verdict carries no stage tag: {value}"))?;
            let reason = value
                .get("reason")
                .and_then(|r| r.as_str())
                .ok_or_else(|| format!("refusal verdict carries no reason: {value}"))?;
            Ok(NativeVerdictObserved::Refused {
                stage: stage.to_string(),
                reason: reason.to_string(),
            })
        }
        other => Err(format!("unknown verdict variant: {other}")),
    }
}

fn parse_native_run_output(stdout: &str) -> Result<NativeRunOutput, String> {
    let mut observations = Vec::new();
    let mut file_refusals = Vec::new();
    let mut terminal_rows = None;
    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line)
            .map_err(|e| format!("unparseable line on the native run's stdout: {e}: {line}"))?;
        if let Some(terminal) = value.get("_terminal") {
            if terminal.as_str() != Some("complete") {
                return Err(format!("terminal marker is not complete: {line}"));
            }
            terminal_rows = value.get("rows").and_then(|r| r.as_u64());
            continue;
        }
        if let Some(refusal) = value.get("file_refusal") {
            let path = refusal
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("file refusal carries no path: {line}"))?;
            let head_reason = refusal
                .get("head_reason")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("file refusal carries no head_reason: {line}"))?;
            let fatal_reason = refusal
                .get("fatal_reason")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("file refusal carries no fatal_reason: {line}"))?;
            file_refusals.push(NativeFileRefusalObserved {
                path: path.to_string(),
                head_reason: head_reason.to_string(),
                fatal_reason: fatal_reason.to_string(),
            });
            continue;
        }
        let module: Vec<String> = value
            .get("module")
            .and_then(|m| serde_json::from_value(m.clone()).ok())
            .ok_or_else(|| format!("observation carries no module segments: {line}"))?;
        let declaration = value
            .get("declaration")
            .and_then(|d| d.as_str())
            .ok_or_else(|| format!("observation carries no declaration: {line}"))?;
        let verdict = decode_verdict_json(
            value
                .get("verdict")
                .ok_or_else(|| format!("observation carries no verdict: {line}"))?,
        )?;
        observations.push(NativeObservation {
            module: module.join("."),
            declaration: declaration.to_string(),
            verdict,
        });
    }
    let terminal_rows =
        terminal_rows.ok_or_else(|| "the native run printed no terminal marker".to_string())?;
    Ok(NativeRunOutput {
        observations,
        file_refusals,
        terminal_rows,
    })
}

/// Spawn the emitted binary once, by explicit path, and decode its stdout. A nonzero exit or an
/// unparseable stdout refuses the lane — the run's output is the receipt's evidence, and a
/// partial or garbled evidence stream is a red, never a truncated green.
fn run_native_binary(
    binary: &Path,
    universe_file: &Path,
    source_roots: &[String],
) -> Result<NativeRunOutput, String> {
    let output = Command::new(binary)
        .arg(universe_file)
        .args(source_roots)
        .output()
        .map_err(|e| {
            format!(
                "V2-NATIVE REFUSAL cause=NativeRunSpawnFailed — spawning {}: {e}",
                binary.display()
            )
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    if !output.status.success() {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=NativeRunFailed status={:?} — {}\n{}",
            output.status.code(),
            binary.display(),
            String::from_utf8_lossy(&output.stderr)
                .lines()
                .rev()
                .take(20)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n")
        ));
    }
    parse_native_run_output(&stdout)
}

/// A CONTEXT-STAGE ROW IS A DERIVED CLASSIFICATION, AND IT IS MINTED ONLY FROM ITS OBSERVATION.
/// A module whose file the context fold refused never reached preparation, so the binary's row
/// for it is a prepare-stage resolver refusal; the harness rewrites exactly those rows to the
/// Context stage carrying the FILE'S fatal reason — the cause that actually refused the file —
/// and admission's context_refusals_backed clause refuses any Context row whose (module, path,
/// reason) triple no observed file refusal backs through the receipt's module-source index. A
/// prepare refusal whose module's file loaded stays a prepare refusal with the resolver's own
/// reason.
///
/// THE JOIN KEY IS module → path, NEVER path → module: the observation names its module, and
/// the file refusal names its path, so the relation is queried in the module → path direction.
/// (An earlier draft queried the path-keyed map with the module, which type-checks at
/// `HashMap<String, String>` and matches nothing — the reclassification silently never fired.)
fn reclassify_context_refusals(
    observations: &mut [NativeObservation],
    file_refusals: &[NativeFileRefusalObserved],
    path_for_module: &HashMap<String, String>,
) {
    let refused_by_path: HashMap<&str, &NativeFileRefusalObserved> = file_refusals
        .iter()
        .map(|fr| (fr.path.as_str(), fr))
        .collect();
    for observation in observations.iter_mut() {
        let Some(path) = path_for_module.get(observation.module.as_str()) else {
            continue;
        };
        let Some(file_refusal) = refused_by_path.get(path.as_str()) else {
            continue;
        };
        if matches!(observation.verdict, NativeVerdictObserved::Refused { .. }) {
            observation.verdict = NativeVerdictObserved::Refused {
                stage: "NativeTestStageContext".to_string(),
                reason: file_refusal.fatal_reason.clone(),
            };
        }
    }
}

// ── the receipt, as interpreter Values ──────────────────────────────────────────────────────
//
// The receipt is built as the `.dag` authority's own types — Symbol fields enter as Str (the
// interpreter's symbol literal value), records and variants name their types, and the Optional
// controls use the Present/Absent encoding every host bridge in this crate shares.

fn identity_value(ctx: &v1_interpreter::InterpContext, module: &str, declaration: &str) -> Value {
    Value::Record {
        type_name: ctx.sym("NativeRouteTestIdentity"),
        fields: Rc::new(vec![
            (ctx.sym("module"), str_value(module)),
            (ctx.sym("declaration"), str_value(declaration)),
        ]),
    }
}

fn verdict_value(ctx: &v1_interpreter::InterpContext, verdict: &NativeVerdictObserved) -> Value {
    match verdict {
        NativeVerdictObserved::Passed => Value::Variant {
            type_name: ctx.sym("NativeTestVerdict"),
            variant_name: ctx.sym("NativeTestPassed"),
            fields: Rc::new(vec![]),
        },
        NativeVerdictObserved::ReturnedFalse => Value::Variant {
            type_name: ctx.sym("NativeTestVerdict"),
            variant_name: ctx.sym("NativeTestReturnedFalse"),
            fields: Rc::new(vec![]),
        },
        NativeVerdictObserved::ReturnedOther => Value::Variant {
            type_name: ctx.sym("NativeTestVerdict"),
            variant_name: ctx.sym("NativeTestReturnedOther"),
            fields: Rc::new(vec![]),
        },
        NativeVerdictObserved::Refused { stage, reason } => Value::Variant {
            type_name: ctx.sym("NativeTestVerdict"),
            variant_name: ctx.sym("NativeTestRefused"),
            fields: Rc::new(vec![
                (
                    ctx.sym("stage"),
                    Value::Variant {
                        type_name: ctx.sym("NativeTestStage"),
                        variant_name: ctx.sym(stage),
                        fields: Rc::new(vec![]),
                    },
                ),
                (ctx.sym("reason"), str_value(reason)),
            ]),
        },
    }
}

fn optional_verdict_value(
    ctx: &v1_interpreter::InterpContext,
    verdict: Option<&NativeVerdictObserved>,
) -> Value {
    match verdict {
        Some(v) => Value::Variant {
            type_name: ctx.sym("Optional"),
            variant_name: ctx.sym("Present"),
            fields: Rc::new(vec![(ctx.sym("value"), verdict_value(ctx, v))]),
        },
        None => Value::Variant {
            type_name: ctx.sym("Optional"),
            variant_name: ctx.sym("Absent"),
            fields: Rc::new(vec![]),
        },
    }
}

fn optional_int(ctx: &v1_interpreter::InterpContext, value: Option<i64>) -> Value {
    match value {
        Some(n) => Value::Variant {
            type_name: ctx.sym("Optional"),
            variant_name: ctx.sym("Present"),
            fields: Rc::new(vec![(ctx.sym("value"), Value::Int(n))]),
        },
        None => Value::Variant {
            type_name: ctx.sym("Optional"),
            variant_name: ctx.sym("Absent"),
            fields: Rc::new(vec![]),
        },
    }
}

fn operability_value(
    ctx: &v1_interpreter::InterpContext,
    preparation: &EmittedPreparation,
    route_wall_ms: i64,
    peak_rss_bytes: Option<i64>,
    admitted_budget_bytes: Option<i64>,
    budget_source: &str,
) -> Value {
    Value::Record {
        type_name: ctx.sym("NativeRouteOperability"),
        fields: Rc::new(vec![
            (
                ctx.sym("cargo_command"),
                str_value(&preparation.cargo_command),
            ),
            (
                ctx.sym("effective_rustflags"),
                str_value(&preparation.effective_rustflags),
            ),
            (
                ctx.sym("rustc_identity"),
                str_value(&preparation.rustc_identity),
            ),
            (
                ctx.sym("cargo_exit_status"),
                Value::Int(i64::from(preparation.cargo_exit_status)),
            ),
            (
                ctx.sym("warning_count"),
                Value::Int(i64::from(preparation.warning_count)),
            ),
            (ctx.sym("route_wall_ms"), Value::Int(route_wall_ms)),
            (ctx.sym("peak_rss_bytes"), optional_int(ctx, peak_rss_bytes)),
            (
                ctx.sym("admitted_budget_bytes"),
                optional_int(ctx, admitted_budget_bytes),
            ),
            (ctx.sym("budget_source"), str_value(budget_source)),
        ]),
    }
}

#[allow(clippy::too_many_arguments)]
fn receipt_value(
    ctx: &v1_interpreter::InterpContext,
    tested_tree: &str,
    preparation: &EmittedPreparation,
    universe: &[(String, String)],
    module_source_index: &[(String, String)],
    population: &[NativeObservation],
    file_refusals: &[NativeFileRefusalObserved],
    false_control: Option<&NativeVerdictObserved>,
    true_control: Option<&NativeVerdictObserved>,
    malformed_control: Value,
    withdrawn_executable: &str,
    route_wall_ms: i64,
    peak_rss_bytes: Option<i64>,
    admitted_budget_bytes: Option<i64>,
    budget_source: &str,
) -> Value {
    let universe_values: Vec<Value> = universe
        .iter()
        .map(|(module, declaration)| identity_value(ctx, module, declaration))
        .collect();
    let population_values: Vec<Value> = population
        .iter()
        .map(|row| Value::Record {
            type_name: ctx.sym("NativeRouteMemberRow"),
            fields: Rc::new(vec![
                (
                    ctx.sym("identity"),
                    identity_value(ctx, &row.module, &row.declaration),
                ),
                (ctx.sym("verdict"), verdict_value(ctx, &row.verdict)),
            ]),
        })
        .collect();
    let file_refusal_values: Vec<Value> = file_refusals
        .iter()
        .map(|fr| Value::Record {
            type_name: ctx.sym("NativeTestFileRefusal"),
            fields: Rc::new(vec![
                (ctx.sym("path"), str_value(&fr.path)),
                (ctx.sym("head_reason"), str_value(&fr.head_reason)),
                (ctx.sym("fatal_reason"), str_value(&fr.fatal_reason)),
            ]),
        })
        .collect();
    let module_source_values: Vec<Value> = module_source_index
        .iter()
        .map(|(module, path)| Value::Record {
            type_name: ctx.sym("NativeRouteModuleSource"),
            fields: Rc::new(vec![
                (ctx.sym("module"), str_value(module)),
                (ctx.sym("path"), str_value(path)),
            ]),
        })
        .collect();
    Value::Record {
        type_name: ctx.sym("NativeRouteReceipt"),
        fields: Rc::new(vec![
            (ctx.sym("job"), str_value("required-v2-native")),
            (ctx.sym("tested_tree"), str_value(tested_tree)),
            (
                ctx.sym("preparation_seed_identity"),
                str_value(&preparation.seed_identity),
            ),
            (
                ctx.sym("emitted_closure_identity"),
                str_value(&preparation.closure_identity),
            ),
            (
                ctx.sym("executable_path"),
                str_value(preparation.binary_path.display().to_string()),
            ),
            (
                ctx.sym("executable_identity"),
                str_value(&preparation.binary_identity),
            ),
            (
                ctx.sym("universe"),
                super::list_value_from_vec(universe_values),
            ),
            (
                ctx.sym("module_source_index"),
                super::list_value_from_vec(module_source_values),
            ),
            (
                ctx.sym("population"),
                super::list_value_from_vec(population_values),
            ),
            (
                ctx.sym("file_refusals"),
                super::list_value_from_vec(file_refusal_values),
            ),
            (
                ctx.sym("false_control"),
                optional_verdict_value(ctx, false_control),
            ),
            (
                ctx.sym("true_control"),
                optional_verdict_value(ctx, true_control),
            ),
            (ctx.sym("malformed_control"), malformed_control),
            (
                ctx.sym("old_route_control"),
                Value::Variant {
                    type_name: ctx.sym("NativeRouteOldRouteControl"),
                    variant_name: ctx.sym("OldRouteWithdrawn"),
                    fields: Rc::new(vec![(
                        ctx.sym("withdrawn_executable"),
                        str_value(withdrawn_executable),
                    )]),
                },
            ),
            (
                ctx.sym("operability"),
                operability_value(
                    ctx,
                    preparation,
                    route_wall_ms,
                    peak_rss_bytes,
                    admitted_budget_bytes,
                    budget_source,
                ),
            ),
        ]),
    }
}

/// The lane's one phase. Green exactly when the admission authority admits the minted receipt;
/// every earlier failure is a located refusal that stops the line. THE VERDICT IS THE
/// AUTHORITY'S OWN, EVALUATED — never re-decided in Rust: the receipt Value is handed to
/// `native_route_admission` in the authority's closure, and the summary the lane prints is
/// `native_route_admission_summary` of that same value, so the terminal line and the admission
/// fold cannot disagree about what was decided.
pub fn run_required_v2_native(source_roots: &[String]) -> Result<(), String> {
    let lane_started = std::time::Instant::now();
    let workspace = super::process_workspace_root();
    let tested_tree = std::env::var("GITHUB_SHA").unwrap_or_else(|_| "local".to_string());

    // 1. THE UNIVERSE, DERIVED. The floor's producer over the full module inventory, filtered
    // to the universe prefix, plus both directions of the module/path relation: the entry join
    // consumed path → module during derivation, and the context reclassification consumes
    // module → path below.
    let derivation = derive_native_universe(source_roots)?;
    let universe = derivation.universe;
    eprintln!(
        "required-ci: v2-native universe derived — {} v2.test.* identities",
        universe.len()
    );
    // THE MODULE-SOURCE INDEX THE RECEIPT CARRIES, restricted to the universe's own modules —
    // the domain the context reclassification can touch and the admission join reasons over.
    // Built here, where a missing row is a located refusal, rather than inside receipt minting
    // where it could only panic or default.
    let mut universe_modules: Vec<String> =
        universe.iter().map(|(module, _)| module.clone()).collect();
    universe_modules.dedup();
    let mut module_source_index: Vec<(String, String)> = Vec::with_capacity(universe_modules.len());
    for module in &universe_modules {
        let Some(path) = derivation.path_for_module.get(module) else {
            return Err(format!(
                "V2-NATIVE REFUSAL cause=UniverseEntryOutsideSubject module={module} — the \
                 derived universe names a module the module index does not hold"
            ));
        };
        module_source_index.push((module.clone(), path.clone()));
    }
    // The derivation's interpreter context and the full inventory's source bytes died with the
    // call above; return the retained arena before the emission peaks beside it (same release
    // discipline as the emission-to-build handoff below).
    let derivation_trim_kb = super::trim_retained_heap();
    eprintln!("required-ci: v2-native derivation arena released (trim_reclaimed_kb={derivation_trim_kb:?})");

    // 2. PREPARATION. The seed emits the compiler closure once, in-process; cargo builds the
    // emitted crate; the receipt records all three identities.
    let preparation = prepare_emitted_compiler(source_roots)?;

    // 3. THE UNIVERSE FILE — the derived universe plus the two named controls.
    let universe_file = workspace
        .join("target")
        .join("v2-native-lane")
        .join("universe.tsv");
    write_universe_file(&universe_file, &universe)?;

    // 4. THE OLD ROUTE IS WITHDRAWN for the whole spawn window (universe run and malformed
    // control alike); the guard restores it on every exit path.
    let withdrawal = withdraw_old_route(&workspace)?;
    let withdrawn_executable = withdrawal.original.display().to_string();

    // 5. THE NATIVE RUN. The emitted binary, by explicit path, over the derived universe.
    eprintln!("required-ci: v2-native running the emitted compiler over the universe");
    let main_output = run_native_binary(&preparation.binary_path, &universe_file, source_roots)?;
    eprintln!(
        "required-ci: v2-native native run complete — {} verdict rows, {} file refusals",
        main_output.terminal_rows,
        main_output.file_refusals.len()
    );

    // 6. THE MALFORMED CONTROL. The same binary over the materialized specimen's scratch root
    // ALONE. The run's only consumed output is the poison path's per-file refusal, and rooting
    // the control at the full corpus paid a second whole-corpus context fold — the lane's
    // dominant cost — to produce it. Tokenization is per-file and deterministic, so the
    // observed refusal is identical under the restricted roots. The committed specimen is not a
    // `.dag` file (its constant carries the reason), so the control's source root is built here
    // — exactly the specimen, rebuilt fresh, never whatever a previous run left behind.
    let control_universe_file = workspace
        .join("target")
        .join("v2-native-lane")
        .join("controls-universe.tsv");
    write_control_universe_file(&control_universe_file)?;
    let control_root = materialize_malformed_specimen(&workspace)?;
    let control_roots = vec![control_root];
    let control_output = run_native_binary(
        &preparation.binary_path,
        &control_universe_file,
        &control_roots,
    )?;
    drop(withdrawal);
    eprintln!("required-ci: v2-native old route restored");

    // 7. THE RECEIPT. Controls are lifted out of the observed population (they are not universe
    // members and the exact join would refuse them as foreign), context refusals are reclassified
    // against the observed file refusals, and the malformed control records what the poisoned
    // run observed.
    let mut population: Vec<NativeObservation> = Vec::new();
    let mut false_control: Option<NativeVerdictObserved> = None;
    let mut true_control: Option<NativeVerdictObserved> = None;
    for observation in main_output.observations {
        if observation.module == CONTROL_MODULE && observation.declaration == FALSE_CONTROL_DECL {
            false_control = Some(observation.verdict);
        } else if observation.module == CONTROL_MODULE
            && observation.declaration == TRUE_CONTROL_DECL
        {
            true_control = Some(observation.verdict);
        } else {
            population.push(observation);
        }
    }
    reclassify_context_refusals(
        &mut population,
        &main_output.file_refusals,
        &derivation.path_for_module,
    );
    let malformed_control = match control_output
        .file_refusals
        .iter()
        .find(|fr| fr.path.contains(MALFORMED_MATERIALIZED_PATH))
    {
        Some(fr) => {
            let reason = fr.fatal_reason.clone();
            let path = fr.path.clone();
            // Built late because it needs the admission context's symbols; carried as data here.
            (path, reason)
        }
        None => (String::new(), String::new()),
    };

    let (graph, indices) = super::resolve_workspace_entry(
        source_roots,
        super::WorkspaceRootRelativeEntry(NATIVE_ROUTE_AUTHORITY_ENTRY),
    )?;
    let route_ctx = super::make_eval_context(&graph, indices, v1_interpreter::ExecutionMode::Wet);
    let malformed_value = if malformed_control.0.is_empty() {
        Value::Variant {
            type_name: route_ctx.sym("NativeRouteMalformedControl"),
            variant_name: route_ctx.sym("MalformedSpecimenAccepted"),
            fields: Rc::new(vec![]),
        }
    } else {
        Value::Variant {
            type_name: route_ctx.sym("NativeRouteMalformedControl"),
            variant_name: route_ctx.sym("MalformedSpecimenRefused"),
            fields: Rc::new(vec![
                (route_ctx.sym("path"), str_value(&malformed_control.0)),
                (route_ctx.sym("reason"), str_value(&malformed_control.1)),
            ]),
        }
    };
    let route_wall_ms = i64::try_from(lane_started.elapsed().as_millis()).unwrap_or(i64::MAX);
    let (_, cgroup_peak) = super::p1_cohort::p1_cohort_cgroup_memory();
    let peak_rss_bytes = cgroup_peak
        .or_else(super::peak_rss_vhwm_bytes)
        .and_then(|b| i64::try_from(b).ok());
    let budget_resolution = crate::memory_governor::read_host_budget_resolution();
    let admitted_budget_bytes = budget_resolution
        .bytes()
        .and_then(|b| i64::try_from(b).ok());
    let budget_source = budget_resolution.label();
    eprintln!(
        "required-ci: v2-native receipt fields closure_digest={} emitted_binary_digest={} \
         cargo_command={} effective_RUSTFLAGS={} rustc_identity={} cargo_exit_status={} \
         warning_count={} route_wall_ms={route_wall_ms} peak_rss_bytes={peak_rss_bytes:?} \
         admitted_budget_bytes={admitted_budget_bytes:?} budget_source={budget_source} \
         (budget authority gunbc.ci_floor_measurement.gunbc_ci_runner_cgroup_memory_cap)",
        preparation.closure_identity,
        preparation.binary_identity,
        preparation.cargo_command,
        preparation.effective_rustflags,
        preparation.rustc_identity,
        preparation.cargo_exit_status,
        preparation.warning_count
    );
    let receipt = receipt_value(
        &route_ctx,
        &tested_tree,
        &preparation,
        &universe,
        &module_source_index,
        &population,
        &main_output.file_refusals,
        false_control.as_ref(),
        true_control.as_ref(),
        malformed_value,
        &withdrawn_executable,
        route_wall_ms,
        peak_rss_bytes,
        admitted_budget_bytes,
        &budget_source,
    );

    // 8. ADMISSION, BY THE AUTHORITY. The lane prints the authority's own summary either way.
    let admission = v1_interpreter::run_in_context_with_args(
        &route_ctx,
        "gunbc.witness_v2_native_route.native_route_admission",
        &[(Some("receipt".to_string()), receipt)],
        false,
    )
    .map_err(|e| format!("V2-NATIVE REFUSAL cause=AdmissionUnevaluable — {e}"))?;
    let summary = v1_interpreter::run_in_context_with_args(
        &route_ctx,
        "gunbc.witness_v2_native_route.native_route_admission_summary",
        &[(Some("admission".to_string()), admission.clone())],
        false,
    )
    .map_err(|e| format!("V2-NATIVE REFUSAL cause=AdmissionUnevaluable — summary: {e}"))?;
    let summary_text = match &summary {
        Value::Str(s) => s.to_string(),
        other => {
            return Err(format!(
                "V2-NATIVE REFUSAL cause=AdmissionUnevaluable — the summary returned {}, not a \
                 String",
                super::output_policy_value_shape(other)
            ))
        }
    };
    let admitted = match &admission {
        Value::Variant { variant_name, .. } => {
            route_ctx.sym_eq(*variant_name, "NativeRouteAdmitted")
        }
        other => {
            return Err(format!(
                "V2-NATIVE REFUSAL cause=AdmissionUnevaluable — native_route_admission returned \
                 {}, not a NativeRouteAdmission variant",
                super::output_policy_value_shape(other)
            ))
        }
    };
    eprintln!("required-ci: v2-native admission {summary_text}");
    if admitted {
        Ok(())
    } else {
        Err(format!(
            "V2-NATIVE REFUSAL cause=AdmissionRefused — {summary_text}"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn refused_observation(
        module: &str,
        declaration: &str,
        stage: &str,
        reason: &str,
    ) -> NativeObservation {
        NativeObservation {
            module: module.to_string(),
            declaration: declaration.to_string(),
            verdict: NativeVerdictObserved::Refused {
                stage: stage.to_string(),
                reason: reason.to_string(),
            },
        }
    }

    fn refusal_shape(observation: &NativeObservation) -> Option<(&str, &str)> {
        match &observation.verdict {
            NativeVerdictObserved::Refused { stage, reason } => Some((stage, reason)),
            _ => None,
        }
    }

    /// THE PRODUCTION-PATH RECLASSIFICATION, AT ITS EXACT JOIN GRAIN. A test module whose own
    /// file the context fold refused (here: an unterminated string at tokenize, the poison
    /// class the malformed control carries) surfaces in the binary's rows as a Prepare-stage
    /// resolver refusal; the reclassification must rewrite THAT identity — keyed by its module,
    /// through the module → path direction of the index relation — to the Context stage
    /// carrying the file's fatal reason. A second module whose file loaded keeps its Prepare
    /// refusal untouched: the join is per-subject, never "some file anywhere".
    #[test]
    fn context_reclassification_joins_module_to_its_own_files_refusal() {
        let mut observations = vec![
            refused_observation(
                "v2.test.poisoned",
                "never_ran",
                "NativeTestStagePrepare",
                "resolve_module_not_found",
            ),
            refused_observation(
                "v2.test.healthy",
                "ran_and_refused",
                "NativeTestStagePrepare",
                "resolve_reason_unbound_symbol",
            ),
        ];
        let file_refusals = vec![NativeFileRefusalObserved {
            path: "src/v2/test/poisoned_test.dag".to_string(),
            head_reason: "parse_g0_tokens_remain".to_string(),
            fatal_reason: "tokenize_lex_e1_unterminated_string".to_string(),
        }];
        let path_for_module: HashMap<String, String> = [
            (
                "v2.test.poisoned".to_string(),
                "src/v2/test/poisoned_test.dag".to_string(),
            ),
            (
                "v2.test.healthy".to_string(),
                "src/v2/test/healthy_test.dag".to_string(),
            ),
        ]
        .into_iter()
        .collect();

        reclassify_context_refusals(&mut observations, &file_refusals, &path_for_module);

        assert_eq!(
            refusal_shape(&observations[0]),
            Some((
                "NativeTestStageContext",
                "tokenize_lex_e1_unterminated_string"
            )),
            "the poisoned module's exact identity must receive its own file's Context-stage \
             fatal reason"
        );
        assert_eq!(
            refusal_shape(&observations[1]),
            Some(("NativeTestStagePrepare", "resolve_reason_unbound_symbol")),
            "a module whose file loaded keeps the resolver's own Prepare-stage reason"
        );
    }

    /// THE DIRECTION IS LOAD-BEARING. Handed the relation in its REVERSED shape — paths as
    /// keys, the exact pre-repair defect — the lookup by module must miss and leave the row a
    /// Prepare refusal: no module → path row means no reclassification, never a guessed join.
    #[test]
    fn context_reclassification_misses_when_queried_off_key() {
        let mut observations = vec![refused_observation(
            "v2.test.poisoned",
            "never_ran",
            "NativeTestStagePrepare",
            "resolve_module_not_found",
        )];
        let file_refusals = vec![NativeFileRefusalObserved {
            path: "src/v2/test/poisoned_test.dag".to_string(),
            head_reason: "parse_g0_tokens_remain".to_string(),
            fatal_reason: "tokenize_lex_e1_unterminated_string".to_string(),
        }];
        let reversed_relation: HashMap<String, String> = [(
            "src/v2/test/poisoned_test.dag".to_string(),
            "v2.test.poisoned".to_string(),
        )]
        .into_iter()
        .collect();

        reclassify_context_refusals(&mut observations, &file_refusals, &reversed_relation);

        assert_eq!(
            refusal_shape(&observations[0]),
            Some(("NativeTestStagePrepare", "resolve_module_not_found")),
            "a path-keyed map cannot answer the module-keyed question"
        );
    }
}
