//! The v2-native ROUTE's host harness (route authority: `gunbc.witness_v2_native_route`),
//! reached as `claim_executor --v2-native-route`.
//!
//! IT IS NOT A REQUIRED LANE, and the prefixes below say so. The required-v2-native CI job was
//! deleted by the 2026-09-11 operator ruling (#11003) because its wall did not fit the acceptance
//! path; the route survives as an operator-invoked instrument under the declared drop
//! `gunbc.rung_drop` `v2_native_route_off_the_merge_path`, whose restoration trigger is a required
//! native-route job designed against an operator-agreed contract rather than this one re-added.
//!
//! WHAT THIS ROUTE CLAIMS, AND WHAT IT DOES NOT. It claims NATIVE UNIVERSE DERIVATION, NATIVE
//! EVALUATION, NATIVE RECEIPT CONSTRUCTION AND NATIVE ADMISSION over a SEED-PREPARED COMPILER
//! ARTIFACT. It does NOT claim to be interpreter-free end to end, and must not be described that
//! way while `prepare_emitted_compiler` below calls `compile_entry_emission` in the v1 process
//! (operator design review 2026-09-11, C1). The seed is never the ordinary miss path: the intended
//! ancestry is one genesis (V1SeedEmitter -> NativeGeneration0) and thereafter only
//! V2EmitterNative(N) -> N+1, with a missing native ancestor a typed refusal. Replacing the call
//! below is the native ancestry acquisition lane's subject, not this one's; nothing here forecloses
//! it.
//!
//! THE SEED PREPARES; THE EMITTED COMPILER DECIDES. This harness emits the compiler closure once,
//! builds it with cargo, withdraws the old-route CLI, and spawns the emitted binary — twice: once
//! in `census` mode over the malformed specimen's scratch root, and once in `adjudicate` mode over
//! the real source roots. Everything semantic happens inside that binary: it derives the
//! `v2.test.*` universe with the floor's own per-file discovery producer, executes every derived
//! identity plus the authority's named live-verdict controls, mints the `NativeRouteReceipt`, and
//! evaluates `native_route_admission` and `native_route_admission_summary` over it
//! (`v2.compiler.compile` `native_lane_run`).
//!
//! WHAT THIS MODULE NO LONGER DOES, AND WHY THE DELETION IS THE POINT. It used to bracket the
//! route with the v1 interpreter on both ends: a Wet interpreter context folding
//! `discover_floor_rows_for_source` over ~5300 sources to derive the universe (~4 minutes
//! interpreted), and a second context evaluating the admission authority over a receipt built
//! here as interpreter `Value`s. The lane's whole claim is that the emitted compiler answers, so
//! both brackets are deleted rather than kept as a fallback (DESIGN section 3: a replacement
//! migration cuts at the root; a surviving X is an attractor). If the emitted closure cannot
//! carry the admission the emission refuses by name — there is no interpreted arm to fall back
//! to. The one v1 evaluation left in the route is the PREPARATION above, and it is named rather
//! than counted as absent.
//!
//! WHAT THE OLD-ROUTE CONTROL PROVES HERE, AND WHAT IT DOES NOT. The verdicts are the spawned
//! binary's own stdout, so the execution route is a process boundary, not a call convention.
//! Withdrawing the seed's `gunbc` binary for the spawn window proves no subprocess fallback to
//! the seed CLI occurred; the receipt's route identity (binary path + content hash) is what
//! admission checks. This process now interprets nothing at all during the lane.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The compiler entry whose closure becomes the lane's emitted-native compiler. Its
/// `compiler_pipeline_entry` is `SourceRootEvalDriver`, so the emitted crate's `main.rs` is the
/// whole-source-root Eval driver this lane exists to route through — and, since the admission
/// authority is now inside that closure, the binary judges its own receipt.
const NATIVE_COMPILE_ENTRY: &str = "src/v2/compiler/00_compile.dag";

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

/// The emitted compiler, prepared: where the binary is, what its bytes are, and the identity of
/// the closure it was emitted from.
struct EmittedPreparation {
    binary_path: PathBuf,
    binary_identity: String,
    closure_identity: String,
    seed_identity: String,
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
    eprintln!("v2-native-route: emitting {NATIVE_COMPILE_ENTRY} (seed, in-process)");
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
        "v2-native-route: emitted {written} files into {} (closure {closure_identity}); \
         emission arena released (rss_kb_before={rss_before_kb:?} trim_reclaimed_kb={trim_reclaimed_kb:?} \
         rss_kb_after={rss_after_kb:?}); cargo build",
        crate_dir.display()
    );
    let verdict = super::emitted_closure_compile_host::run_cargo(
        &crate_dir,
        &workspace,
        "v2_native_lane_carries_no_mutation_probe",
    );
    if !super::emitted_closure_compile_host::cargo_verdict_compiled(&verdict) {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=EmittedCompilerBuildFailed — {}",
            super::emitted_closure_compile_host::cargo_verdict_summary(&verdict)
        ));
    }
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
        "v2-native-route: emitted compiler at {} (sha256 {binary_identity})",
        binary_path.display()
    );
    Ok(EmittedPreparation {
        binary_path,
        binary_identity,
        closure_identity,
        seed_identity,
    })
}

/// THE OLD-ROUTE WITHDRAWAL, AS A GUARD SO A REFUSAL PATH CANNOT SKIP THE RESTORE. The seed's
/// `gunbc` binary — the old compiler/interpreter route's CLI — is renamed out of the way before
/// the first native spawn and restored when the guard drops, on every exit path.
///
/// ITS ABSENCE IS AN OBSERVATION, NOT A REFUSAL (operator design review 2026-09-11, C5). This
/// harness used to REQUIRE the old route to exist so that it could move it, which made a property
/// of the developer's target directory a precondition of the route's contract — and reads exactly
/// backwards, since a route that is not there is one the spawns could not have reached. The guard
/// therefore carries which control held, and the receipt records the authority's own arm:
/// `OldRouteWithdrawn` when a file was moved aside, `OldRouteNotPresentAtWindow` when the path
/// held nothing for the whole spawn window. Neither claims the stronger
/// `OldRouteAbsentByConstruction` — no v1 emitter or interpreter reachable at all — which the
/// native ancestry acquisition lane introduces with the producer that can establish it.
struct OldRouteWithdrawalGuard {
    original: PathBuf,
    withdrawn: Option<PathBuf>,
}

/// Which old-route control the window actually held, as the two host-fact rows the emitted binary
/// decodes into the authority's coproduct.
struct OldRouteControlFacts {
    disposition: &'static str,
    executable: String,
}

fn withdraw_old_route(workspace: &Path) -> Result<OldRouteWithdrawalGuard, String> {
    let original = workspace.join("target").join("release").join("gunbc");
    if !original.is_file() {
        eprintln!(
            "v2-native-route: old route not present at {} for the spawn window (nothing to \
             withdraw; the receipt records the path it looked for)",
            original.display()
        );
        return Ok(OldRouteWithdrawalGuard {
            original,
            withdrawn: std::option::Option::None,
        });
    }
    let withdrawn = workspace
        .join("target")
        .join("release")
        .join("gunbc.withdrawn-v2-native");
    std::fs::rename(&original, &withdrawn).map_err(|e| {
        format!(
            "V2-NATIVE REFUSAL cause=OldRouteWithdrawalImpossible — renaming {}: {e}",
            original.display()
        )
    })?;
    eprintln!(
        "v2-native-route: old route withdrawn ({} moved aside for the native spawns)",
        original.display()
    );
    Ok(OldRouteWithdrawalGuard {
        original,
        withdrawn: Some(withdrawn),
    })
}

impl OldRouteWithdrawalGuard {
    fn control_facts(&self) -> OldRouteControlFacts {
        match &self.withdrawn {
            Some(_) => OldRouteControlFacts {
                disposition: "withdrawn",
                executable: self.original.display().to_string(),
            },
            std::option::Option::None => OldRouteControlFacts {
                disposition: "not_present_at_window",
                executable: self.original.display().to_string(),
            },
        }
    }
}

impl Drop for OldRouteWithdrawalGuard {
    fn drop(&mut self) {
        let Some(withdrawn) = self.withdrawn.as_ref() else {
            return;
        };
        if let Err(e) = std::fs::rename(withdrawn, &self.original) {
            eprintln!(
                "v2-native-route: WARNING — restoring the old route failed ({}): {e}",
                self.original.display()
            );
        }
    }
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

/// One per-file front-end refusal the emitted binary's collecting context fold observed. The
/// census run's only consumed output.
struct NativeFileRefusalObserved {
    path: String,
    fatal_reason: String,
}

/// The spawned run's terminal marker, decoded. THE MARKER IS THE VERDICT SURFACE: the binary
/// prints one, carrying the authority's own admission summary, and its absence is a refusal — a
/// crash mid-population cannot be read as a quiet green.
struct NativeTerminalMarker {
    mode: String,
    rows: u64,
    universe: u64,
    file_refusals: u64,
    admitted: bool,
    summary: String,
}

struct NativeRunOutput {
    file_refusals: Vec<NativeFileRefusalObserved>,
    terminal: NativeTerminalMarker,
}

/// The host reads exactly two kinds of line: a per-file refusal row and the terminal marker.
/// Population rows are the binary's own evidence surface and are counted by the marker, so they
/// are passed through to the log rather than re-decoded here — decoding them would be this
/// harness re-forming a receipt the authority has already judged.
fn parse_native_run_output(stdout: &str) -> Result<NativeRunOutput, String> {
    let mut file_refusals = Vec::new();
    let mut terminal: Option<NativeTerminalMarker> = None;
    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line)
            .map_err(|e| format!("unparseable line on the native run's stdout: {e}: {line}"))?;
        if value.get("_terminal").is_some() {
            if value.get("_terminal").and_then(|t| t.as_str()) != Some("complete") {
                return Err(format!("terminal marker is not complete: {line}"));
            }
            terminal = Some(NativeTerminalMarker {
                mode: value
                    .get("mode")
                    .and_then(|m| m.as_str())
                    .unwrap_or_default()
                    .to_string(),
                rows: value.get("rows").and_then(|r| r.as_u64()).unwrap_or(0),
                universe: value.get("universe").and_then(|r| r.as_u64()).unwrap_or(0),
                file_refusals: value
                    .get("file_refusals")
                    .and_then(|r| r.as_u64())
                    .unwrap_or(0),
                admitted: value
                    .get("admitted")
                    .and_then(|a| a.as_bool())
                    .unwrap_or(false),
                summary: value
                    .get("summary")
                    .and_then(|s| s.as_str())
                    .unwrap_or_default()
                    .to_string(),
            });
            continue;
        }
        if let Some(refusal) = value.get("file_refusal") {
            let path = refusal
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("file refusal carries no path: {line}"))?;
            let fatal_reason = refusal
                .get("fatal_reason")
                .and_then(|v| v.as_str())
                .ok_or_else(|| format!("file refusal carries no fatal_reason: {line}"))?;
            file_refusals.push(NativeFileRefusalObserved {
                path: path.to_string(),
                fatal_reason: fatal_reason.to_string(),
            });
        }
    }
    let terminal =
        terminal.ok_or_else(|| "the native run printed no terminal marker".to_string())?;
    Ok(NativeRunOutput {
        file_refusals,
        terminal,
    })
}

/// Spawn the emitted binary once, by explicit path, and decode its stdout. A NON-ZERO EXIT IS
/// NOT ITSELF THE ANSWER: `adjudicate` exits 1 on a refused admission after printing the
/// authority's summary, so the stdout is parsed first and a marker-carrying refusal is reported
/// with the cause the authority gave. A non-zero exit with no marker is a lane refusal carrying
/// the process's own stderr — never a truncated green.
fn run_native_binary(binary: &Path, args: &[String]) -> Result<NativeRunOutput, String> {
    let output = Command::new(binary).args(args).output().map_err(|e| {
        format!(
            "V2-NATIVE REFUSAL cause=NativeRunSpawnFailed — spawning {}: {e}",
            binary.display()
        )
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    // THE CHILD'S STDERR IS RELAYED, NOT SWALLOWED. The spawned binary commits its per-phase
    // `[cost-partition]` receipt lines to stderr as each phase finishes, precisely so a cancelled
    // run keeps the phases it already paid for; `Command::output` buffers them, so without this
    // relay the route's own cost receipt would exist inside the child and reach nobody. Written
    // through before the verdict is decided, so a refusal path still carries the phases that led
    // to it.
    for line in String::from_utf8_lossy(&output.stderr).lines() {
        eprintln!("{line}");
    }
    match parse_native_run_output(&stdout) {
        Ok(parsed) => Ok(parsed),
        Err(parse_cause) => Err(format!(
            "V2-NATIVE REFUSAL cause=NativeRunFailed status={:?} — {} — {parse_cause}\n{}",
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
        )),
    }
}

/// THE HOST FACTS ARE EXACTLY WHAT THE ROUTE CANNOT OBSERVE OF ITSELF, and nothing else. Each row
/// is `key<TAB>value`; the binary refuses a missing or duplicated key rather than defaulting one,
/// so a harness that forgets to record an identity fails loudly instead of minting a receipt with
/// an empty field the authority would then refuse for the wrong reason.
fn write_host_facts(
    path: &Path,
    tested_tree: &str,
    preparation: &EmittedPreparation,
    old_route: &OldRouteControlFacts,
    malformed_control: &(String, String),
) -> Result<(), String> {
    let rows = [
        ("tested_tree", tested_tree),
        (
            "preparation_seed_identity",
            preparation.seed_identity.as_str(),
        ),
        (
            "emitted_closure_identity",
            preparation.closure_identity.as_str(),
        ),
        ("executable_identity", preparation.binary_identity.as_str()),
        ("old_route_disposition", old_route.disposition),
        ("old_route_executable", old_route.executable.as_str()),
        ("malformed_control_path", malformed_control.0.as_str()),
        ("malformed_control_reason", malformed_control.1.as_str()),
    ];
    let mut text = String::new();
    for (key, value) in rows {
        text.push_str(key);
        text.push('\t');
        text.push_str(value);
        text.push('\n');
    }
    text.push_str("executable_path\t");
    text.push_str(&preparation.binary_path.display().to_string());
    text.push('\n');
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("creating {}: {e}", parent.display()))?;
    }
    std::fs::write(path, text).map_err(|e| format!("writing {}: {e}", path.display()))
}

/// The lane's one phase. Green exactly when the emitted binary's own admission admitted the
/// receipt it minted; every earlier failure is a located refusal that stops the line.
pub fn run_required_v2_native(source_roots: &[String]) -> Result<(), String> {
    let workspace = super::process_workspace_root();
    let tested_tree = std::env::var("GITHUB_SHA").unwrap_or_else(|_| "local".to_string());

    // 1. PREPARATION. The seed emits the compiler closure once, in-process; cargo builds the
    // emitted crate; the receipt records all three identities. This is the seed's whole job.
    let preparation = prepare_emitted_compiler(source_roots)?;

    // 2. THE OLD ROUTE IS WITHDRAWN for the whole spawn window (census and adjudication alike);
    // the guard restores it on every exit path.
    let withdrawal = withdraw_old_route(&workspace)?;
    let old_route = withdrawal.control_facts();

    // 3. THE MALFORMED CONTROL, FIRST, because its observed refusal is a host fact the
    // adjudicating run records in the receipt. The same binary in `census` mode over the
    // materialized specimen's scratch root ALONE: the run's only consumed output is the poison
    // path's per-file refusal, and rooting the control at the full corpus paid a second
    // whole-corpus context fold — the lane's dominant cost — to produce it. Tokenization is
    // per-file and deterministic, so the observed refusal is identical under the restricted root.
    let control_root = materialize_malformed_specimen(&workspace)?;
    let control_output = run_native_binary(
        &preparation.binary_path,
        &["census".to_string(), control_root],
    )?;
    if control_output.terminal.mode != "census" {
        drop(withdrawal);
        return Err(format!(
            "V2-NATIVE REFUSAL cause=NativeRunFailed — the control run reported mode {}, not census",
            control_output.terminal.mode
        ));
    }
    let malformed_control = match control_output
        .file_refusals
        .iter()
        .find(|fr| fr.path.contains(MALFORMED_MATERIALIZED_PATH))
    {
        Some(fr) => (fr.path.clone(), fr.fatal_reason.clone()),
        None => (String::new(), String::new()),
    };
    eprintln!(
        "v2-native-route: malformed control observed path=\"{}\" reason=\"{}\"",
        malformed_control.0, malformed_control.1
    );

    // 4. THE HOST FACTS, WRITTEN. Everything else the receipt carries is the binary's own
    // observation.
    let facts_file = workspace
        .join("target")
        .join("v2-native-lane")
        .join("host-facts.tsv");
    write_host_facts(
        &facts_file,
        &tested_tree,
        &preparation,
        &old_route,
        &malformed_control,
    )?;

    // 5. THE LANE RUN. The emitted binary, by explicit path, over the real source roots: it
    // derives the universe, executes it, mints the receipt and judges it.
    eprintln!("v2-native-route: adjudicating through the emitted compiler");
    let mut args = vec!["adjudicate".to_string(), facts_file.display().to_string()];
    args.extend(source_roots.iter().cloned());
    let run = run_native_binary(&preparation.binary_path, &args);
    drop(withdrawal);
    eprintln!("v2-native-route: old route restored");
    let run = run?;
    if run.terminal.mode != "adjudicate" {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=NativeRunFailed — the lane run reported mode {}, not \
             adjudicate",
            run.terminal.mode
        ));
    }
    eprintln!(
        "v2-native-route: universe={} population={} file_refusals={}",
        run.terminal.universe, run.terminal.rows, run.terminal.file_refusals
    );

    // 6. THE VERDICT IS THE AUTHORITY'S, REPORTED AS GIVEN. The summary and the admission are one
    // value inside the binary (`native_lane_run` derives the summary from the admission it
    // returns), so the terminal line and the admission cannot disagree about what was decided.
    eprintln!("v2-native-route: admission {}", run.terminal.summary);
    if run.terminal.admitted {
        Ok(())
    } else {
        Err(format!(
            "V2-NATIVE REFUSAL cause=AdmissionRefused — {}",
            run.terminal.summary
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// THE MARKER IS THE VERDICT SURFACE, AND A REFUSED ADMISSION MUST SURVIVE THE PARSE. The
    /// binary exits non-zero on a refused admission after printing its summary, so the host must
    /// read the marker rather than treat the exit status as the whole answer.
    #[test]
    fn terminal_marker_carries_the_refused_admission_summary() {
        let stdout = concat!(
            "{\"file_refusal\":{\"path\":\"a.dag\",\"head_reason\":\"h\",\"fatal_reason\":\"f\"}}\n",
            "{\"_terminal\":\"complete\",\"mode\":\"adjudicate\",\"rows\":3,\"universe\":3,",
            "\"file_refusals\":1,\"admitted\":false,\"summary\":\"REFUSED: population_omissions_present\"}\n"
        );
        let parsed = parse_native_run_output(stdout).expect("the marker parses");
        assert!(!parsed.terminal.admitted);
        assert_eq!(
            parsed.terminal.summary,
            "REFUSED: population_omissions_present"
        );
        assert_eq!(parsed.file_refusals.len(), 1);
        assert_eq!(parsed.file_refusals[0].fatal_reason, "f");
    }

    /// NO MARKER IS A REFUSAL, NEVER A GREEN. A run that crashed mid-population prints rows and
    /// then stops; the absence of the terminal line is the only evidence of the truncation, so
    /// the parse must refuse it.
    #[test]
    fn a_missing_terminal_marker_refuses() {
        let stdout = "{\"identity\":{\"module\":\"v2.test.a\",\"declaration\":\"t\"}}\n";
        assert!(parse_native_run_output(stdout).is_err());
    }
}
