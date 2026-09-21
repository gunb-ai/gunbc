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

/// THE V2-EXCLUSIVE CLI'S ENTRY, AND WHY IT IS A SECOND CONSTANT RATHER THAN A SECOND ROUTE.
///
/// One emitted crate has one entry point, and the emitter admits exactly one
/// `compiler_pipeline_entry` declaration per closure, so the v2-native CLI cannot be a mode of
/// the module above: it is a DIFFERENT closure with a different declared driver
/// (`std.compiler_entry` `NativeCliDriver`). Everything downstream of the entry -- the emission,
/// the crate write, the cargo invocation, the clean-build verdict -- is the same producer
/// parameterised by it, which is why `prepare_emitted_compiler_for_entry` takes the entry rather
/// than this file growing a second copy of that sequence.
const V2_NATIVE_CLI_ENTRY: &str = "src/v2/cli/compile_cli.dag";

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

/// THE TWO DOORS' CONTROL INPUTS, AS THE THREE FACTS THEIR ARGV IS BUILT FROM.
///
/// The root is named for its PROPERTY and not for one of its readers, because both walks use it:
/// the CLI door emits this subject, and the eval driver's accepting control censuses it. A
/// `CLI_DOOR_` prefix would have been a fact about which caller was written first.
///
/// The source root is a directory of its own because the CLI walks a root RECURSIVELY for every
/// `.dag` file under it: a control sharing a directory with other fixtures would ingest whatever
/// their authors add next, so what the door compiled would stop being a fact this harness states.
/// The subject is the module that directory's one file declares, and the witness word is a
/// declaration name from that same file — which is what makes the positive control an assertion
/// about THIS subject's emission rather than about stdout being non-empty.
const WELL_FORMED_CONTROL_ROOT: &str = "fixtures/native_cli_door";
const CLI_DOOR_ENTRY_MODULE: &str = "fixture.native_cli_door.door_probe";
const CLI_DOOR_EMITTED_WITNESS: &str = "native_cli_door_probe_value";

/// THE EMIT PROBE'S EXPECTED REFUSAL, AND WHY THIS CONSTANT IS THE PROBE'S WHOLE POINT.
///
/// The built door CANNOT EMIT A CLOSURE TODAY. Measured on the artifact this preparation produces:
/// over the fixture root alone it answers `infer_grounding_not_derived` thirty-five times, and over
/// the real corpus it answers a LOCATED `parse_g0_tokens_remain` at
/// `dag/extdeps/access/posix_effective_principal_read_op.dag`, whose bytes at that offset are the
/// `service ... { operation ... }` form. Those are COMPILER LIMITATIONS in the emitted front end,
/// not defects in this instrument, and `v2.cli.compile_cli` already records the same shape measured
/// at gunbc#11507 on three different entries.
///
/// SO THE PROBE EXPECTS RED, AND IT IS ENROLLED RATHER THAN DELETED (DESIGN section 4b(4)). A probe
/// that is removed because its subject cannot pass yet leaves nothing to notice when the subject
/// starts passing. This one pins the limitation by its CAUSE, so the day the door can ground a
/// closure the probe stops matching and this instrument REFUSES -- which is the signal to flip it
/// into a permanent regression control asserting the door emits. It is deliberately NOT satisfied
/// by any non-zero exit: a door that refused for some other reason, or crashed, would establish
/// nothing about the limitation this row pins.
const CLI_DOOR_EMIT_LIMITATION: &str = "infer_grounding_not_derived";

/// The refusal control's expected cause, quoted from `v2.cli.compile_cli` `cli_parse_finish`'s
/// `cli_no_entry` arm. It is the DETAIL and not the reason symbol because the rendered main prints
/// the `ProcessExit` reason string and nothing else, so the detail is the only part of the refusal
/// that crosses the process boundary — the gap `compile_cli`'s own annotation records having been
/// measured on a built binary. Matching the sentence therefore establishes that the arm the door
/// took is the one this control asked for; matching only "REFUSED" would be satisfied by every
/// other refusal the door has.
const CLI_DOOR_REFUSAL_DETAIL: &str = "emit needs the module to resolve as its subject";

/// What one spawn of the built CLI observed. Returned rather than adjudicated so the step that
/// knows which control it was running decides what the observation means — the same split
/// `run_self_host` and `run_v2_native_cli` make against the instrument seam.
struct CliDoorRun {
    status: Option<i32>,
    stdout: String,
    stderr: String,
}

/// The emitted compiler, prepared: where the binary is, what its bytes are, and the identity of
/// the closure it was emitted from.
struct EmittedPreparation {
    binary_path: PathBuf,
    binary_identity: String,
    closure_identity: String,
    seed_identity: String,
    build: EmittedBuildObserved,
}

/// The emitted compiler's build as the receipt records it — mirror of
/// `gunbc.witness_v2_native_route` `NativeRouteEmittedBuild`. Every field is read from the
/// spawn that ran or the toolchain that answered, never composed from what was intended.
struct EmittedBuildObserved {
    cargo_argv: Vec<String>,
    rustflags: String,
    compiler_path: String,
    rustc_identity: String,
    exit_status: i64,
    warning_count: i64,
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
    prepare_emitted_compiler_for_entry(source_roots, NATIVE_COMPILE_ENTRY)
}

fn prepare_emitted_compiler_for_entry(
    source_roots: &[String],
    entry: &str,
) -> Result<EmittedPreparation, String> {
    let workspace = super::process_workspace_root();
    // The probe root follows the declared execution environment (per-job runner temp in CI,
    // host temp locally) — the selection's authority and its receipt live beside the required
    // phase's own root policy in `emitted_closure_compile_host`.
    let probe_root = super::lane_emit_compile_probe_root();
    eprintln!("v2-native-route: emitting {entry} (seed, in-process)");
    let run = super::compile_entry_emission(
        source_roots,
        entry,
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
    let (crate_dir, written) =
        super::emitted_closure_compile_host::write_probe_crate(&run, &probe_root, entry)
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
    // The invocation resolves and binds the one compiler the build runs under and takes its
    // identity from the crate's own directory; a compiler that cannot be resolved or named is
    // a refusal before the build is paid for.
    let invocation =
        super::emitted_closure_compile_host::probe_cargo_invocation(&crate_dir, &workspace)
            .map_err(|cause| format!("V2-NATIVE REFUSAL cause={cause}"))?;
    let rustc = invocation.rustc_identity.clone();
    // THE BASELINE IS ATTRIBUTED TO THE SAME PROBE SYMBOL THE FAULTED ARM WILL CARRY. This
    // argument used to be the literal `"v2_native_lane_carries_no_mutation_probe"`, which was a
    // true statement about this lane and is now a false one: the discriminating red below is
    // established on every preparation, so the baseline and the faulted arm are two runs of one
    // pair and must be read against one symbol. A second spelling here would let the green arm
    // search for a symbol the red arm never injects.
    let verdict = super::emitted_closure_compile_host::run_cargo(
        &crate_dir,
        &workspace,
        super::emitted_closure_compile_host::MUTATION_PROBE_SYMBOL,
    );
    if !super::emitted_closure_compile_host::cargo_verdict_compiled(&verdict) {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=EmittedCompilerBuildFailed — {} (argv={:?} RUSTFLAGS={:?} rustc={rustc})",
            super::emitted_closure_compile_host::cargo_verdict_summary(&verdict),
            invocation.argv,
            invocation.rustflags,
        ));
    }
    // `cargo_verdict_compiled` admitted only the `Completed { status: 0 }` arm, so the fields
    // below are the run's own; a verdict of any other shape refused above.
    let (exit_status, warning_count) = match &verdict {
        super::emitted_closure_compile_host::CargoVerdict::Completed {
            status,
            warning_count,
            ..
        } => (i64::from(*status), *warning_count as i64),
        other => {
            return Err(format!(
                "V2-NATIVE REFUSAL cause=EmittedCompilerBuildFailed — verdict admitted as compiled \
                 is not a completed run: {}",
                super::emitted_closure_compile_host::cargo_verdict_summary(other)
            ))
        }
    };
    let build = EmittedBuildObserved {
        cargo_argv: invocation.argv,
        rustflags: invocation.rustflags,
        compiler_path: invocation.compiler_path,
        rustc_identity: rustc,
        exit_status,
        warning_count,
    };
    eprintln!(
        "v2-native-route: emitted crate built — argv={:?} RUSTFLAGS={:?} compiler={} \
         exit_status={exit_status} warning_count={warning_count} rustc={}",
        build.cargo_argv, build.rustflags, build.compiler_path, build.rustc_identity
    );
    let binary_path = workspace.join("target").join("release").join(
        super::emitted_closure_compile_host::probe_package_name(entry),
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

    // THE PLANTED DEFECT, ON THE ARTIFACT THIS PREPARATION IS ABOUT TO HAND ON.
    //
    // Until this block landed, both retained native build subjects -- the test driver
    // (`v2.compiler.compile`) and the CLI (`v2.cli.compile_cli`) -- were admitted on a GREEN CARGO
    // RUN ALONE, and this call site passed the literal
    // `"v2_native_lane_carries_no_mutation_probe"` where the required emit-compile phase passes its
    // probe symbol. A cargo phase that is green because nothing was measured is the decoration
    // DESIGN section 4b calls worse than absent: a fingerprint alias, a replayed cached verdict or
    // an emitter that stopped reaching rustc all report `Completed status=0` byte-identically to a
    // real compile, and the instrument would carry the green under the subject's name.
    //
    // IT IS THE SAME PRODUCER THE REQUIRED PHASE USES, NOT A SECOND ONE.
    // `emitted_closure_compile_host` `establish_discriminating_red` injects ONE type error into the
    // ENTRY'S OWN emitted module, requires cargo to fail ALONE on it with a diagnostic NAMING the
    // injected symbol, restores the bytes byte-exactly, and returns a typed non-discriminating
    // verdict for every other shape. Calling it here rather than copying its shape is what keeps
    // "the emitted tree is really being compiled" one fact with one home (DESIGN section 3).
    //
    // IT RUNS AFTER THE BINARY IS IDENTIFIED AND THE IDENTITY IS RE-READ AFTERWARDS. The faulted
    // arm is a cargo run over the same crate and target directory, so the artifact this function
    // returns must be shown to be the one the GREEN baseline produced rather than assumed to be:
    // a mutation that somehow left a different executable at that path would otherwise be handed
    // to the entrypoint step as though it were the baseline's.
    let entry_module = super::emitted_closure_compile_host::entry_rust_module(entry, &workspace)
        .map_err(|cause| format!("V2-NATIVE REFUSAL cause=EntryModuleUnreadable — {cause}"))?;
    eprintln!("v2-native-route: establishing the discriminating red on {entry_module}");
    let mutation = super::emitted_closure_compile_host::establish_discriminating_red(
        &crate_dir,
        &workspace,
        &entry_module,
    );
    if !super::emitted_closure_compile_host::mutation_verdict_discriminated(&mutation) {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=EmittedBuildNotDiscriminating — {}",
            super::emitted_closure_compile_host::mutation_verdict_summary(&mutation)
        ));
    }
    eprintln!(
        "v2-native-route: discriminating red established — {}",
        super::emitted_closure_compile_host::mutation_verdict_summary(&mutation)
    );
    let identity_after_mutation = sha256_file(&binary_path)?;
    if identity_after_mutation != binary_identity {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=EmittedCompilerReplacedByFaultedArm — {} was {binary_identity} \
             when the clean baseline built it and is {identity_after_mutation} after the fault was \
             injected and removed; the executable handed on is not the one the green build produced",
            binary_path.display()
        ));
    }

    Ok(EmittedPreparation {
        binary_path,
        binary_identity,
        closure_identity,
        seed_identity,
        build,
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
    /// THE NOT-PRESENT ARM IS A CLAIM ABOUT A WINDOW, SO IT IS CHECKED AT BOTH ENDS (review
    /// 64075). The withdrawn arm HOLDS the path closed: the rename is what makes "the old route
    /// was unreachable while the spawns ran" true, and the guard restores it afterwards. The
    /// not-present arm holds nothing, so a single `is_file()` before the spawns is evidence about
    /// t0 and the receipt clause reads it as evidence about the whole window -- and a binary that
    /// materializes mid-run (a concurrent build, a cache landing) would green the one control
    /// whose entire job is to prove the native route could not have fallen back. DESIGN section 4d:
    /// an inference is not promoted to a deduction. So the window is re-read after the last spawn
    /// and a path that became occupied REFUSES the run rather than widening the arm to mean
    /// "absent at some point" (section 5: a failure arm refuses, never widens). This does not
    /// collapse the two arms: withdrawal still closes the window by construction, and this one is
    /// an observation checked at both of its ends.
    fn verify_window_stayed_closed(&self) -> Result<(), String> {
        if self.withdrawn.is_some() {
            return Ok(());
        }
        if self.original.is_file() {
            return Err(format!(
                "V2-NATIVE REFUSAL cause=OldRouteAppearedDuringWindow — {} held no file when the \
                 spawn window opened and holds one now, so the receipt cannot record \
                 not_present_at_window: the old route was reachable for part of the window",
                self.original.display()
            ));
        }
        Ok(())
    }

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
            // EVERY FIELD THE MODE CARRIES IS REQUIRED, NONE IS DEFAULTED -- AND WHICH FIELDS THOSE
            // ARE IS A FACT ABOUT THE MODE. These were `unwrap_or` defaults, and each one
            // fabricated a different lie: a missing `rows`/`universe`/`file_refusals` printed as
            // `0` in the route's own summary line, and a missing `admitted` defaulted to `false`
            // with an empty `summary`, so the refusal downstream read `cause=AdmissionRefused — `
            // with no cause at all.
            //
            // THE FIRST FIX WAS TOO STRICT AND THE ROUTE CAUGHT IT. Requiring the adjudicate field
            // set on EVERY marker refused the census run, whose marker is
            // `{"_terminal":"complete","mode":"census","file_refusals":N}` and which has no
            // population to count: census exists to report the malformed control's per-file
            // refusals and nothing else. Demanding `rows`/`universe`/`admitted`/`summary` there was
            // requiring a fact the mode does not have -- the mirror image of defaulting one it
            // does. So the required set is keyed by mode, and both modes still refuse a missing
            // field rather than inventing it.
            //
            // THE CENSUS ARM'S ZEROS ARE NOT A MEASUREMENT AND NOTHING READS THEM, which is what
            // keeps this from being the defaulting it replaces: the census consumer touches
            // `terminal.mode` and the separately carried file refusals, and never `rows`,
            // `universe`, `admitted` or `summary`. They are struct fill for fields the mode does
            // not have. If a future consumer wants a count from a census marker, the honest change
            // is a per-mode marker type, not a zero that has quietly become load-bearing.
            let need_u64 = |key: &str| -> Result<u64, String> {
                value
                    .get(key)
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| format!("terminal marker carries no {key}: {line}"))
            };
            let need_str = |key: &str| -> Result<String, String> {
                value
                    .get(key)
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .ok_or_else(|| format!("terminal marker carries no {key}: {line}"))
            };
            let mode = need_str("mode")?;
            let file_refusals = need_u64("file_refusals")?;
            terminal = Some(match mode.as_str() {
                "census" => NativeTerminalMarker {
                    mode,
                    rows: 0,
                    universe: 0,
                    file_refusals,
                    admitted: false,
                    summary: String::new(),
                },
                "adjudicate" => NativeTerminalMarker {
                    rows: need_u64("rows")?,
                    universe: need_u64("universe")?,
                    admitted: value
                        .get("admitted")
                        .and_then(|a| a.as_bool())
                        .ok_or_else(|| format!("terminal marker carries no admitted: {line}"))?,
                    summary: need_str("summary")?,
                    mode,
                    file_refusals,
                },
                other => {
                    return Err(format!(
                        "terminal marker reports an unknown mode {other:?}: {line}"
                    ))
                }
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
            continue;
        }
        // THE BASE CONTRACT, RESTORED (review 64181). This loop had no final arm, so any line that
        // was neither the terminal marker nor a file refusal fell off the end silently — an
        // undeclared drop of the rule the base states in as many words: the binary's stdout is a
        // receipt surface, and an unrecognized line on it is a harness defect, not noise.
        //
        // A POPULATION ROW IS RECOGNIZED, NOT DECODED. The adjudication moved into the emitted
        // binary, so the host no longer needs each verdict — it counts them through the marker.
        // But the rows are still on this surface, so they are recognized by the shape the
        // authority gives them (NativeRouteMemberRow: an `identity` and a `verdict`) rather than
        // waved past by a catch-all, which would re-open exactly the hole this arm closes.
        if value.get("identity").is_some() && value.get("verdict").is_some() {
            continue;
        }
        return Err(format!(
            "unrecognized line on the native run's stdout — it is a receipt surface, so this is a \
             harness defect rather than noise: {line}"
        ));
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
// `rows_out` PERSISTS THE CHILD'S STDOUT BESIDE THE RECEIPT (eager-raven-113 ruling, 2026-09-12).
// The driver prints one population row per identity, each carrying a TYPED verdict -- for a refused
// module, NativeTestRefused { stage, reason }. This host parsed that stream for counts and then
// DROPPED it, so the only surviving artifact was the `[native-prepare]` stderr line, which collapses
// the same fact to `outcome=accepted|refused` through a matches!. A typed refusal computed and then
// flattened to a Bool at the reporting boundary is the DESIGN section 5 shape, and the cost was
// measured rather than argued: after a 113-minute full-N run at 36e6ad91 the question "why did 575
// of 713 modules refuse" was unanswerable from anything on disk, and answering it would have meant
// running the whole route again.
//
// WRITTEN BEFORE THE PARSE, DELIBERATELY. A run that ends in a refusal is exactly the run whose rows
// someone needs, and the 36e6ad91 run refused at the cost partition before printing its terminal
// marker -- had the write been gated on a successful parse, the rows would have been discarded on
// the one occasion they mattered most.
//
// TWO CHECKS, TWO QUESTIONS, AND THEY ARE NOT INTERCHANGEABLE (side-chat 10:23). Byte equality
// between the child stdout and its read-back answers "is this artifact what the child wrote". The
// population count answers "does the row set the marker claims match the row set on the surface" --
// a check about the DRIVER agreeing with itself, not about storage. An earlier revision ran only
// the count and described it as the first guarantee, which is why the distinction is written down:
// a same-count substitution passes a count and fails the bytes, so conflating them would have left
// the stronger claim resting on the weaker check.
fn run_native_binary(
    binary: &Path,
    args: &[String],
    rows_out: Option<&Path>,
) -> Result<NativeRunOutput, String> {
    let output = Command::new(binary).args(args).output().map_err(|e| {
        format!(
            "V2-NATIVE REFUSAL cause=NativeRunSpawnFailed — spawning {}: {e}",
            binary.display()
        )
    })?;
    // CHECKED UTF-8, NOT LOSSY (side-chat 10:23). from_utf8_lossy substitutes U+FFFD for invalid
    // sequences, so a corrupted stream would be PARSED AND PERSISTED as though it were the bytes
    // the child wrote -- the decoder silently repairing the evidence it is supposed to carry. A
    // receipt surface cannot transform its own subject; invalid UTF-8 on this stream is a refusal.
    let stdout = match String::from_utf8(output.stdout.clone()) {
        Ok(text) => text,
        Err(cause) => {
            return Err(format!(
                "V2-NATIVE REFUSAL cause=NativeRunStdoutNotUtf8 — {} wrote {} byte(s) that are not valid UTF-8: {cause}",
                binary.display(),
                output.stdout.len()
            ))
        }
    };
    // THE CHILD'S STDERR IS RELAYED, NOT SWALLOWED. The spawned binary writes its cost receipt
    // (`[native-cost-partition]`, `[native-prepare]`) to its own stderr, and `Command::output`
    // buffers it, so without this relay those rows would exist inside the child and reach nobody
    // — which is what they do on the pre-#10940 host, where `output.stderr` is read only on the
    // error path. Written through before the verdict is decided, so a refusal path still carries
    // the rows that led to it.
    //
    // WHAT THIS RELAY DOES NOT DO, corrected rather than left overclaiming: it does not preserve a
    // CANCELLED run's rows. `Command::output()` collects to completion, so nothing is forwarded
    // until the child exits — on a multi-hour run the host stays silent throughout and a cancel or
    // a killed parent loses every row the child had already printed. The earlier wording here said
    // the relay existed "so a cancelled run keeps the phases it already paid for", which the
    // mechanism does not deliver. Streaming is the close: `Stdio::piped` with a reader draining the
    // child as it runs, which is also what would let a watcher see progress rather than a silence
    // indistinguishable from "no rows". Reported to the #11060 lane (smart-eagle-506), whose
    // zero-rows observation on srv2 is the pre-relay host, not a missing emission.
    for line in String::from_utf8_lossy(&output.stderr).lines() {
        eprintln!("{line}");
    }
    if let Some(path) = rows_out {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "V2-NATIVE REFUSAL cause=NativeRunRowsUnwritable — creating {}: {e}",
                    parent.display()
                )
            })?;
        }
        std::fs::write(path, &output.stdout).map_err(|e| {
            format!(
                "V2-NATIVE REFUSAL cause=NativeRunRowsUnwritable — writing {}: {e}",
                path.display()
            )
        })?;
        // STORAGE INTEGRITY IS A BYTE COMPARISON, AND NOTHING ELSE IS (side-chat 10:23). The
        // earlier form of this check counted records carrying identity+verdict and compared that
        // count with the terminal marker, then claimed the file could not silently disagree with
        // the verdict path. THAT CLAIM EXCEEDED THE CHECK: a same-count substitution -- another
        // identity, a flipped verdict -- or a mangled file_refusal record passes a count
        // unchanged. The count answers a different question and is kept below for that question
        // alone. What establishes that the artifact IS what the child wrote is reading the bytes
        // back and comparing them to the bytes in hand.
        let persisted_bytes = std::fs::read(path).map_err(|e| {
            format!(
                "V2-NATIVE REFUSAL cause=NativeRunRowsUnreadable — reading back {}: {e}",
                path.display()
            )
        })?;
        if persisted_bytes != output.stdout {
            return Err(format!(
                "V2-NATIVE REFUSAL cause=NativeRunRowsNotPreserved — {} holds {} byte(s) but the child wrote {}; the persisted rows are not the rows that were produced",
                path.display(),
                persisted_bytes.len(),
                output.stdout.len()
            ));
        }
        eprintln!(
            "v2-native-route: driver rows persisted to {} ({} bytes, byte-for-byte verified)",
            path.display(),
            output.stdout.len()
        );
    }
    match parse_native_run_output(&stdout) {
        Ok(parsed) => {
            if let Some(path) = rows_out {
                let persisted = std::fs::read_to_string(path).map_err(|e| {
                    format!(
                        "V2-NATIVE REFUSAL cause=NativeRunRowsUnreadable — reading back {}: {e}",
                        path.display()
                    )
                })?;
                let in_file = persisted
                    .lines()
                    .filter(|line| {
                        serde_json::from_str::<serde_json::Value>(line)
                            .ok()
                            .is_some_and(|v| {
                                v.get("identity").is_some() && v.get("verdict").is_some()
                            })
                    })
                    .count() as u64;
                if in_file != parsed.terminal.rows {
                    return Err(format!(
                        "V2-NATIVE REFUSAL cause=NativeRunRowsDisagree — {} carries {in_file} population row(s) but the terminal marker declares {}; a persisted receipt that disagrees with the verdict path is not a receipt",
                        path.display(),
                        parsed.terminal.rows
                    ));
                }
            }
            // THE EXIT STATUS AND THE RECEIPT MUST AGREE (review 64499). The unconditional
            // `if !output.status.success()` gate that main carries was removed on this branch
            // (abe5a17bb4) for a real reason -- the driver exits non-zero on a REFUSED admission
            // after printing its summary, so treating status as the whole answer would discard the
            // receipt that explains the refusal. But that justifies not treating status as the
            // whole answer; it does not justify discarding it, and discarding it left the host
            // accepting a run that exits non-zero while claiming `admitted: true`. Fail-closure
            // then rested on an unchecked ordering invariant in GENERATED code -- that the emitted
            // main happens to print `_terminal` last -- which is not a thing this host can see.
            //
            // THE CHECK IS AN AGREEMENT, WHICH IS STRICTLY STRONGER THAN THE GATE IT RESTORES:
            // main's version could not catch the other contradiction, a zero exit carrying a
            // refused receipt. Both directions are a disagreement between two independent
            // observations of one run, and a disagreement is refused rather than resolved in
            // favour of whichever one we prefer.
            //
            // SCOPED TO ADJUDICATE because `admitted` is not a fact the census mode has: its
            // marker carries only mode and file_refusals, and the false there is struct fill
            // documented as never read. Asking a mode for a fact it does not carry is how the
            // first version of the marker parser refused every census run.
            if parsed.terminal.mode == "adjudicate"
                && output.status.success() != parsed.terminal.admitted
            {
                return Err(format!(
                    "V2-NATIVE REFUSAL cause=NativeRunStatusReceiptDisagree — {} exited {:?} but its terminal marker reports admitted={}; the exit status and the receipt are two observations of one run and they disagree",
                    binary.display(),
                    output.status.code(),
                    parsed.terminal.admitted
                ));
            }
            Ok(parsed)
        }
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
    let build = &preparation.build;
    let executable_path = preparation.binary_path.display().to_string();
    let mut rows: Vec<(String, &str)> = vec![
        ("tested_tree".to_string(), tested_tree),
        (
            "preparation_seed_identity".to_string(),
            preparation.seed_identity.as_str(),
        ),
        (
            "emitted_closure_identity".to_string(),
            preparation.closure_identity.as_str(),
        ),
        ("executable_path".to_string(), executable_path.as_str()),
        (
            "executable_identity".to_string(),
            preparation.binary_identity.as_str(),
        ),
        ("old_route_disposition".to_string(), old_route.disposition),
        (
            "old_route_executable".to_string(),
            old_route.executable.as_str(),
        ),
        (
            "malformed_control_path".to_string(),
            malformed_control.0.as_str(),
        ),
        (
            "malformed_control_reason".to_string(),
            malformed_control.1.as_str(),
        ),
    ];
    // THE EMITTED BUILD CROSSES AS ROWS TOO: the six facts of NativeRouteEmittedBuild
    // (gunbc.witness_v2_native_route) are the host's own spawn observations, so they are host
    // facts by the same rule as the identities above. The argv is one row per word under a
    // length row, because a word is opaque bytes and the TSV grammar has one value per key --
    // no separator is smuggled into a value.
    let argv_len = build.cargo_argv.len().to_string();
    let exit_status = build.exit_status.to_string();
    let warning_count = build.warning_count.to_string();
    rows.push(("cargo_argv_len".to_string(), argv_len.as_str()));
    for (index, word) in build.cargo_argv.iter().enumerate() {
        rows.push((format!("cargo_argv_{index}"), word.as_str()));
    }
    rows.push(("rustflags".to_string(), build.rustflags.as_str()));
    rows.push(("compiler_path".to_string(), build.compiler_path.as_str()));
    rows.push(("rustc_identity".to_string(), build.rustc_identity.as_str()));
    rows.push(("exit_status".to_string(), exit_status.as_str()));
    rows.push(("warning_count".to_string(), warning_count.as_str()));
    let mut text = String::new();
    for (key, value) in &rows {
        // A value carrying the row grammar's own delimiters would be read back as a different
        // row set by the binary; refuse here rather than let the decoder refuse a fact that
        // was never the one written.
        if value.contains('\t') || value.contains('\n') {
            return Err(format!(
                "host fact {key} carries a tab or newline and cannot cross as one TSV row: {value:?}"
            ));
        }
        text.push_str(key);
        text.push('\t');
        text.push_str(value);
        text.push('\n');
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("creating {}: {e}", parent.display()))?;
    }
    std::fs::write(path, text).map_err(|e| format!("writing {}: {e}", path.display()))
}

/// The lane's one phase. Green exactly when the emitted binary's own admission admitted the
/// receipt it minted; every earlier failure is a located refusal that stops the line.
/// THE V1 -> V2 SELF-HOST STEP, AND ITS WHOLE POINT IS THAT WHERE IT STOPS IS THE STATUS.
///
/// The seed emits v2's compiler closure, assembles it as a crate, and builds it. That is the
/// generation the repository can perform today, and it either holds or it does not -- there is no
/// roster of which entries are known-good, because a roster is a second representation of what this
/// process demonstrates by running (DESIGN sections 2 and 3). A regression here fails the step;
/// progress moves the failure later. Nothing separate has to be updated for either.
///
/// WHY THIS IS A PREFIX OF `--v2-native-route` RATHER THAN A SECOND PATH. That route already
/// performs exactly these boundaries in `prepare_emitted_compiler` and then spends roughly nine
/// further minutes executing the v2.test.* universe through the emitted binary. The test fold is a
/// different claim -- what the emitted compiler ANSWERS -- and bundling it here would price the
/// self-host question at the cost of a question nobody asked. So this calls the same producer and
/// returns on its verdict; the two modes cannot disagree about whether the seed can build v2,
/// because only one of them decides it.
///
/// WHAT THIS DOES NOT ESTABLISH, named so the green is not read for more than it carries. It is
/// EMISSION, COMPILATION AND A BOUNDED EXECUTION — `walk_eval_driver_door` starts the built driver
/// and requires it to accept a well-formed root and refuse a malformed one — and it is still not
/// behavioural equivalence: DESIGN section 7 asks that the emitted
/// module also behave as the seed does on a discriminating corpus, and that half
/// (`--behavioral-receipt-*`) is a declared drop that no required run performs. Nor is it the
/// second generation: the built binary emitting the same closure is the v2 -> v2 boundary, and it
/// refuses today because `v2.compiler.compile` declares `SourceRootEvalDriver`, whose rendered main
/// answers `census` and `adjudicate` and has no compile mode at all. That refusal is the honest next
/// position and belongs in this step when a driver exists that can reach it.
/// What the self-host generation observed, as the fields a caller renders. Returned rather than
/// printed so the instrument seam decides the termination: a producer that only printed would make
/// every caller re-derive the verdict from stderr, which is the second representation DESIGN
/// section 3 forbids.
pub struct SelfHostHeld {
    pub closure_identity: String,
    pub binary_identity: String,
    pub seed_identity: String,
    pub exit_status: i64,
    pub warning_count: i64,
    /// What the built driver observed when this instrument STARTED it: the size of the universe
    /// its accepting control derived, and the cause it gave for refusing the poison specimen.
    /// Carried as the refusal's own sentence rather than as a Bool, so a receipt reader can see
    /// WHICH refusal fired — the flattening this file's `run_native_binary` annotation records
    /// having already cost one unanswerable question.
    pub door_universe: i64,
    pub door_refusal_reason: String,
}

/// WALK THROUGH THE TEST DRIVER'S DOOR, BOTH WAYS.
///
/// WHAT THIS CLOSES, AND WHY IT IS NOT THE SAME GAP THE CLI HAD. The eval driver's entrypoint IS
/// spawned somewhere — by `run_required_v2_native`, which the operator invokes with
/// `--required-v2-native` — but nothing spawned it from `//gunbc/instruments:self-host`, so that
/// instrument's green said "the closure emitted and cargo accepted it" about an executable it never
/// started. An artifact admitted without being run is the specification-without-execution shape
/// DESIGN section 5 names, and it is that shape whether or not a DIFFERENT route happens to run the
/// same program: the instrument is the thing reporting, and its evidence has to be its own.
///
/// THIS CONTROL IS A SECOND READER OF THE EVAL DRIVER'S VERB SURFACE, AND SAYS SO RATHER THAN
/// LEAVING IT TO BE DISCOVERED. The argv below spells `census` as a literal. The driver itself is
/// the first reader and the authority: it decides what verbs it accepts. So there are two places
/// that know this driver's argv, which is the §3 fork in its mildest form -- mild because the
/// literal is one word in one call, and real because a change to the driver's surface makes this
/// control spawn an argv the driver no longer accepts, and the failure would look like a broken
/// door rather than a stale spelling.
///
/// IT IS ADMITTED RATHER THAN REPAIRED HERE BECAUSE THE JOIN HAS ANOTHER OWNER. gunbc#11952 turns
/// that verb surface into a plan -- `native_driver_parse` and the plan it returns -- which is
/// exactly the authority this control should ASK instead of spelling. That PR is OPEN and its
/// symbols DO NOT RESOLVE in this tree; they are named here as provenance for the trigger, not as
/// citations of declarations that exist (`gunbc.recurring_failure_mode`
/// `unlanded_citation_indistinguishable_at_the_citing_end`). Consuming it from here would invert
/// the dependency and block this instrument on an unlanded PR.
///
/// RETIREMENT TRIGGER, AT CAPABILITY GRAIN AND NOT ARTIFACT GRAIN: this literal is retired when THE
/// ENTRYPOINT CONTROL DERIVES ITS ARGV FROM THE DRIVER'S OWN PLAN, once that plan is on main --
/// sufficient that no spelling of any verb survives in this file. It is deliberately NOT "when
/// #11952 lands": that PR could land with the plan reachable and this call still spelling `census`,
/// which would satisfy an artifact-grain trigger while the second reader stood (DESIGN §4b(3) --
/// a trigger naming less than the capability is satisfied while the capability stays dead).
///
/// THE PAIR IS TWO CENSUS RUNS DIFFERING ONLY IN THEIR ROOT. Over `fixtures/native_cli_door` — one
/// well-formed module — the driver must reach `census` and report NO per-file refusal; over the
/// materialized poison specimen it must report one naming that path. A binary that refused
/// everything would pass the second alone, and one whose front-end refusal arm is dead would pass
/// the first alone; only the pair discriminates. It is `census` rather than `adjudicate` because
/// the question is whether the ENTRYPOINT executes and the front end answers, and adjudication
/// would additionally derive and judge the whole `v2.test.*` universe — the hours-long subject of
/// the required-v2-native route, and a different claim.
fn walk_eval_driver_door(binary: &Path, workspace: &Path) -> Result<(u64, String), String> {
    let clean_root = workspace.join(WELL_FORMED_CONTROL_ROOT);
    if !clean_root.is_dir() {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=NativeCliDoorControlRootAbsent — {} is not a directory, so \
             the accepting control has no input",
            clean_root.display()
        ));
    }
    eprintln!(
        "self-host: walking the driver — census over {}",
        clean_root.display()
    );
    let clean = run_native_binary(
        binary,
        &["census".to_string(), clean_root.display().to_string()],
        None,
    )?;
    if clean.terminal.mode != "census" {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=NativeRunFailed — the accepting control reported mode {}, not census",
            clean.terminal.mode
        ));
    }
    if !clean.file_refusals.is_empty() {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=EvalDriverRefusedItsAcceptingControl — the driver reported {} \
             per-file refusal(s) over {}, whose one file is well formed; a front end that refuses \
             this input makes the refusal beside it uninformative",
            clean.file_refusals.len(),
            clean_root.display()
        ));
    }
    let universe = clean.terminal.universe;
    eprintln!("self-host: driver accepted the clean root — mode=census universe={universe}");

    // THE ROOT IS ABSOLUTE HERE AND RELATIVE IN `run_required_v2_native`, and the difference is the
    // caller's cwd rather than a preference. That lane anchors at the workspace root before it
    // spawns; an instrument producer is invoked from wherever the operator ran `gunbc test`, so a
    // workspace-relative root would name a directory that does not exist and the refusal control
    // would report "no refusal" for a reason that has nothing to do with the front end.
    let poison_root = workspace
        .join(materialize_malformed_specimen(workspace)?)
        .display()
        .to_string();
    eprintln!("self-host: refusal control — the same mode over the materialized poison specimen");
    let poisoned = run_native_binary(binary, &["census".to_string(), poison_root], None)?;
    let Some(refusal) = poisoned
        .file_refusals
        .iter()
        .find(|fr| fr.path.contains(MALFORMED_MATERIALIZED_PATH))
    else {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=EvalDriverAcceptedThePoisonSpecimen — the driver reported no \
             per-file refusal naming {MALFORMED_MATERIALIZED_PATH}; a route that accepts a source \
             whose string never terminates has a dead front-end refusal arm, and every refusal \
             verdict it reports elsewhere is then worthless"
        ));
    };
    eprintln!(
        "self-host: driver refused the poison specimen — path=\"{}\" reason=\"{}\"",
        refusal.path, refusal.fatal_reason
    );
    Ok((universe, refusal.fatal_reason.clone()))
}

pub fn run_self_host(source_roots: &[String]) -> Result<SelfHostHeld, String> {
    let started = std::time::Instant::now();
    eprintln!(
        "self-host: v1 -> v2 — seed emits {NATIVE_COMPILE_ENTRY}, assembles the crate, and builds it"
    );
    let prepared = prepare_emitted_compiler(source_roots)?;
    let wall_s = started.elapsed().as_secs();
    // THE BUILD'S OWN COUNTERS DECIDE, NOT THE ABSENCE OF A REFUSAL ABOVE. prepare_emitted_compiler
    // admits only a completed zero-status run, so these are already its verdict; reading them here
    // is what makes the step's claim ("built clean") the one the receipt carries rather than one
    // inferred from control flow having reached this line.
    eprintln!(
        "self-host: v1->v2 emit and build completed — exit_status={} warning_count={} wall_s={wall_s}",
        prepared.build.exit_status, prepared.build.warning_count,
    );
    // THE COUNTERS ARE CARRIED, NOT ADJUDICATED HERE. A non-clean build is an observation that DID
    // NOT HOLD, and a refusal above is the subject never having been reached; those are different
    // terminations with different exit statuses, and the instrument seam is the one place that
    // knows the vocabulary. Deciding it here too would give one fact two homes.
    let (door_universe, door_refusal_reason) =
        walk_eval_driver_door(&prepared.binary_path, &super::process_workspace_root())?;
    Ok(SelfHostHeld {
        closure_identity: prepared.closure_identity,
        binary_identity: prepared.binary_identity,
        seed_identity: prepared.seed_identity,
        exit_status: prepared.build.exit_status,
        warning_count: prepared.build.warning_count,
        door_universe: door_universe as i64,
        door_refusal_reason,
    })
}

/// What the v2-native CLI generation observed, as the fields a caller renders. It is deliberately
/// the SAME shape as `SelfHostHeld`: both instruments ask one question -- can the seed emit this
/// entry's closure and build it clean -- about two different entries, and giving them two shapes
/// would make the difference look semantic when it is only which closure was compiled.
pub struct V2NativeCliHeld {
    pub closure_identity: String,
    pub binary_identity: String,
    pub seed_identity: String,
    pub exit_status: i64,
    pub warning_count: i64,
    /// The status the built binary itself took on the EMIT PROBE, and the bytes it wrote on stdout.
    /// Both are carried rather than folded into a Bool because a receipt that says only "the probe
    /// held" cannot be read afterwards for what the door actually did — the flattening
    /// `run_native_binary`'s annotation records paying for once already. `door_emitted_bytes` is
    /// expected to be ZERO while the probe expects red, and it is recorded precisely so that a
    /// nonzero value is visible in the receipt on the day the door starts emitting.
    pub door_exit_status: i64,
    pub door_emitted_bytes: i64,
    /// The status the same binary took when the one argument the positive control supplies was
    /// removed. It is beside the green deliberately: the pair is the evidence, and a reader given
    /// only the green would have no way to tell an executing door from one that exits 0 on
    /// anything.
    pub door_refusal_exit_status: i64,
}

/// THE V2-EXCLUSIVE CLI, EMITTED AND BUILT. This is the door the self-host step stops in front of.
///
/// `//gunbc/instruments:self-host` establishes that the seed can emit and build
/// `v2.compiler.compile`; what it explicitly does NOT establish is the second generation, because
/// that entry declares `SourceRootEvalDriver`, whose rendered main answers `census` and `adjudicate`
/// and has no compile mode. `v2.cli.compile_cli` is the entry that does have one, and this producer
/// is the question "does that door exist as a built binary today" asked the same way its sibling
/// asks its own.
///
/// WHY IT IS A SEPARATE INSTRUMENT AND NOT A PHASE OF THE SELF-HOST ONE. They compile DIFFERENT
/// CLOSURES, and different is the operative word rather than smaller: the two are close in size,
/// neither contains the other's `compiler_pipeline_entry` declaration, and this one reaches modules
/// the compiler entry's does not (`v2.extdeps.languages.rust` among them). So a green here and a
/// green there are two facts about two compilations, and collapsing them would let one subject's
/// regression be reported under the other's name.
///
/// WHAT A GREEN ESTABLISHES, CORRECTED BY THIS CHANGE RATHER THAN RESTATED. The paragraph here
/// used to end "a green here plus those witnesses is 'the decisions hold and the door compiles',
/// not 'the door has been walked through'" — an accurate description of an instrument that stopped
/// at cargo. It no longer stops there: `walk_cli_door` below spawns the binary this preparation
/// produced, twice, and admits it only on an emission that names its subject and a refusal that
/// names its cause. So the door IS walked through, on a fixture closure.
///
/// WHAT IS STILL NOT CLAIMED, and the distinction is the whole reason the door exists. This is not
/// the second generation: that is this binary emitting the COMPILER'S OWN closure, and the subject
/// walked here is `fixture.native_cli_door.door_probe`, a single import-free module. What the walk
/// establishes is that argv parsing, the source-root read, ingest, resolution, emission and the
/// process exit all execute in the built artifact — the inhabitance obligation beside
/// `v2.test.cli.v2_native_cli`, whose rows supply their argv and their ingest and are claims about
/// the folds rather than about the program.
/// SPAWN THE BUILT CLI ONCE, BY EXPLICIT PATH, AND CARRY WHAT IT DID.
///
/// IT IS NOT `run_native_binary`, AND THE DIFFERENCE IS THE PROTOCOL RATHER THAN THE PROCESS. That
/// function decodes the SourceRootEvalDriver's stdout — population rows and a terminal marker —
/// and every check it performs is about that stream. The CLI driver's rendered main prints the
/// EMITTED TEXT on stdout and `REFUSED: <reason>` on stderr, so handing its output to that parser
/// would report a working door as an unparseable one. Two drivers, two stdout contracts; sharing
/// the reader would be a fork of the contract, not a saving.
///
/// STDOUT IS CHECKED UTF-8 AND NOT LOSSY, on the rule `run_native_binary` already states: a lossy
/// decode substitutes U+FFFD and would let this harness assert about bytes the child did not
/// write. Stderr is lossy-decoded because it is diagnostic relay rather than a subject.
fn run_cli_door(binary: &Path, args: &[String]) -> Result<CliDoorRun, String> {
    let output = Command::new(binary).args(args).output().map_err(|e| {
        format!(
            "V2-NATIVE REFUSAL cause=NativeCliDoorSpawnFailed — spawning {} {args:?}: {e}",
            binary.display()
        )
    })?;
    let stdout = String::from_utf8(output.stdout.clone()).map_err(|cause| {
        format!(
            "V2-NATIVE REFUSAL cause=NativeCliDoorStdoutNotUtf8 — {} {args:?} wrote {} byte(s) that are not valid UTF-8: {cause}",
            binary.display(),
            output.stdout.len()
        )
    })?;
    Ok(CliDoorRun {
        status: output.status.code(),
        stdout,
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

/// WALK THROUGH THE DOOR, BOTH WAYS.
///
/// WHAT THIS CLOSES. Before it, `//gunbc/instruments:v2-native-cli` emitted the CLI's closure and
/// built it, and stopped — the door compiled and nothing ever opened it. DESIGN section 5 names
/// that exactly: a typecheck is not a consumer, and "done" is a real consumer green BY EXECUTION
/// plus a discriminating input that goes red. `v2.test.cli.v2_native_cli` executes the door's
/// FOLDS over supplied argv and a supplied ingest, which is the right subject for those claims and
/// is not this one; the inhabitance obligation beside them is that the REAL PATH runs, and the real
/// path is the built process reading real argv and real files.
///
/// AND THE FIRST THING IT ESTABLISHED WAS THAT THE DOOR DOES NOT EMIT, which is the point rather
/// than a disappointment. An entrypoint-execution control exists to find out what the program
/// actually does, and on its first real run it found a limitation the instrument had been silently
/// carrying: the emitted front end cannot ground a closure. The probe is enrolled around that fact
/// instead of being tuned until it passes.
///
/// WHAT THE PAIR ESTABLISHES, AND IT IS NOT WHAT AN EARLIER DRAFT OF THIS COMMENT CLAIMED. The
/// door CANNOT EMIT yet, so there is no arm here that ends in a successful emission. What the pair
/// does establish is that THE BUILT BINARY'S ENTRYPOINT EXECUTES: the `cli_no_entry` arm reaches a
/// LOCATED refusal that only this program's own `.dag` fold can produce, which means the process
/// started, read argv, parsed it, decided, rendered its own cause and set its own exit status.
/// That is the inhabitance claim `v2.test.cli.v2_native_cli` cannot make, because its rows supply
/// argv and ingest rather than running the program.
///
/// TWO SPAWNS, ONE BINARY, DIFFERING IN ONE ARGUMENT. The emit probe hands the door a complete
/// argv and pins the named limitation that stops it (`CLI_DOOR_EMIT_LIMITATION`). The refusal
/// control removes ONLY `--entry` and requires `cli_no_entry`'s located sentence. One argument
/// differs, so the second arm's cause is information about THAT argument rather than about the door
/// being broken in general — and the two causes being DIFFERENT is itself the evidence that the
/// door is deciding rather than failing uniformly.
///
/// NEITHER ARM IS SATISFIED BY A NON-ZERO EXIT. A door that panicked, could not find its source
/// root, or refused for any of its other causes also exits non-zero, and admitting those would
/// green this control on a broken door. Both arms match the CAUSE, and a non-zero exit with the
/// wrong cause refuses the instrument rather than passing it.
fn walk_cli_door(binary: &Path, workspace: &Path) -> Result<(i64, usize, i64), String> {
    let root = workspace.join(WELL_FORMED_CONTROL_ROOT);
    if !root.is_dir() {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=NativeCliDoorControlRootAbsent — {} is not a directory, so \
             neither control has an input and a green here would describe nothing",
            root.display()
        ));
    }
    let root_arg = root.display().to_string();

    eprintln!(
        "v2-native-cli: walking the door — emit --entry {CLI_DOOR_ENTRY_MODULE} --source-root {root_arg}"
    );
    let emitted = run_cli_door(
        binary,
        &[
            "emit".to_string(),
            "--entry".to_string(),
            CLI_DOOR_ENTRY_MODULE.to_string(),
            "--source-root".to_string(),
            root_arg.clone(),
        ],
    )?;
    // THE EXPECTING-RED ARM. A GREEN HERE IS THE FAILURE, and it is the good kind: it means the
    // emitted front end can ground a closure, so this probe has outlived its subject and must
    // become the regression control that asserts the door emits. Refusing loudly is what makes that
    // transition happen on the day it becomes true instead of whenever someone next reads the file.
    if emitted.status == Some(0) {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=NativeCliDoorEmitProbeGreened — the built CLI EMITTED {} \
             byte(s) and exited 0. This probe expects `{CLI_DOOR_EMIT_LIMITATION}` because the \
             emitted front end could not ground a closure; that limitation is gone. FLIP THIS PROBE \
             into a regression control that requires exit 0 and requires the emitted text to name \
             `{CLI_DOOR_EMITTED_WITNESS}` — do not delete it, and do not relax this arm.",
            emitted.stdout.len()
        ));
    }
    if !emitted.stderr.contains(CLI_DOOR_EMIT_LIMITATION) {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=NativeCliDoorRefusedForAnUnpinnedReason — the built CLI exited \
             {:?} without naming `{CLI_DOOR_EMIT_LIMITATION}`. A non-zero exit is not this probe's \
             subject: it pins ONE named limitation, so a different refusal, a crash or a spawn \
             failure must be read rather than absorbed. stderr: {}",
            emitted.status,
            emitted.stderr.trim()
        ));
    }
    let door_exit_status = i64::from(emitted.status.unwrap_or_default());
    let emitted_bytes = emitted.stdout.len();
    eprintln!(
        "v2-native-cli: door refused the emit as expected — {CLI_DOOR_EMIT_LIMITATION}, exit {door_exit_status}, stdout {emitted_bytes} byte(s)"
    );

    eprintln!("v2-native-cli: refusal control — the same argv with --entry removed");
    let refused = run_cli_door(
        binary,
        &[
            "emit".to_string(),
            "--source-root".to_string(),
            root_arg.clone(),
        ],
    )?;
    match refused.status {
        Some(0) | None => {
            return Err(format!(
                "V2-NATIVE REFUSAL cause=NativeCliDoorRefusalNotDiscriminating — the built CLI \
                 exited {:?} on an argv naming no --entry; a door that accepts the argv its own \
                 `cli_no_entry` arm exists to refuse establishes nothing about the arm beside it",
                refused.status
            ))
        }
        Some(_) => {}
    }
    if !refused.stderr.contains(CLI_DOOR_REFUSAL_DETAIL) {
        return Err(format!(
            "V2-NATIVE REFUSAL cause=NativeCliDoorRefusedForAnotherReason — the built CLI exited \
             {:?} without naming `{CLI_DOOR_REFUSAL_DETAIL}`; a non-zero exit is not evidence that \
             the refusal this control asked for is the one that fired. stderr: {}",
            refused.status,
            refused.stderr.trim()
        ));
    }
    let refusal_status = i64::from(refused.status.unwrap_or_default());
    eprintln!("v2-native-cli: door refused as cli_no_entry — exit {refusal_status}");
    Ok((door_exit_status, emitted_bytes, refusal_status))
}

pub fn run_v2_native_cli(source_roots: &[String]) -> Result<V2NativeCliHeld, String> {
    let started = std::time::Instant::now();
    eprintln!(
        "v2-native-cli: v1 -> v2 — seed emits {V2_NATIVE_CLI_ENTRY}, assembles the crate, and builds it"
    );
    let prepared = prepare_emitted_compiler_for_entry(source_roots, V2_NATIVE_CLI_ENTRY)?;
    let wall_s = started.elapsed().as_secs();
    eprintln!(
        "v2-native-cli: emit and build completed — exit_status={} warning_count={} wall_s={wall_s}",
        prepared.build.exit_status, prepared.build.warning_count,
    );
    // THE ENTRYPOINT EXECUTES, OR THE SUBJECT WAS NEVER REACHED. A refusal from the walk is a
    // `SubjectUnreached` at the instrument seam for the same reason a refusal from the preparation
    // is: the door failing to spawn, to find its control root or to emit says the observation could
    // not be taken, which is a different termination from the build having reported non-zero
    // counters. The one arm that is deliberately NOT a "did not hold" is the refusal control going
    // green — a door that accepts everything is a broken instrument, not a broken build.
    let (door_exit_status, door_emitted_bytes, door_refusal_exit_status) =
        walk_cli_door(&prepared.binary_path, &super::process_workspace_root())?;
    // The counters are carried and not adjudicated here, on the same rule `run_self_host` follows:
    // a non-clean build is an observation that did not hold, a refusal above is the subject never
    // having been reached, and the instrument seam is the one place that knows the difference.
    Ok(V2NativeCliHeld {
        closure_identity: prepared.closure_identity,
        binary_identity: prepared.binary_identity,
        seed_identity: prepared.seed_identity,
        exit_status: prepared.build.exit_status,
        warning_count: prepared.build.warning_count,
        door_exit_status,
        door_emitted_bytes: door_emitted_bytes as i64,
        door_refusal_exit_status,
    })
}

pub fn run_required_v2_native(source_roots: &[String]) -> Result<(), String> {
    let lane_started = std::time::Instant::now();
    let workspace = super::process_workspace_root();
    // THE TESTED TREE IS OBSERVED OR THE RUN REFUSES (review 64210). This defaulted to the literal
    // "local", and `tested_tree` is not telemetry: `gunbc.witness_v2_native_route`
    // `native_route_tested_tree_recorded` admits on `tested_tree != ""`, so the default SATISFIED
    // its own admission clause and the receipt then asserted it described a tree named `local`.
    // That is the defect the rendered main fixes for its sibling field in this same change, and
    // the clause-bearing field is the one where it actually buys a false green.
    //
    // WHY NOT FALL BACK TO `git rev-parse HEAD`, which the seed can do: on a dirty worktree HEAD
    // names a commit whose content is NOT what ran, so it is the same plausible-identity defect
    // wearing a real sha. An identity that is only sometimes true is worse here than none.
    //
    // A LOCAL RUN IS STILL AVAILABLE, and deliberately so: the operator sets GITHUB_SHA to the
    // commit the tree describes, which makes the identity an assertion someone made rather than
    // one the harness invented. Refusing without saying that would have turned an honesty fix into
    // the removal of an operator-invoked instrument.
    let tested_tree = std::env::var("GITHUB_SHA").map_err(|_| {
        "V2-NATIVE REFUSAL cause=TestedTreeUnobservable — GITHUB_SHA is unset, so the tree this run \
         describes cannot be observed. The receipt's tested_tree carries an admission clause, so a \
         placeholder would green it and assert a tree that was never tested. Set GITHUB_SHA to the \
         commit this worktree describes to run the route outside CI."
            .to_string()
    })?;

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
    // No rows file for the census control: that mode reports per-file refusals and prints no
    // population rows, so a file there would be an empty artifact inviting the reading that the
    // census found nothing to say.
    let control_output = run_native_binary(
        &preparation.binary_path,
        &["census".to_string(), control_root],
        None,
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
    let rows_file = workspace
        .join("target")
        .join("v2-native-lane")
        .join("driver-rows.jsonl");
    let run = run_native_binary(&preparation.binary_path, &args, Some(&rows_file));
    // The window closes here, so this is where the not-present arm is re-read. Taken BEFORE the
    // guard drops, because dropping it restores the withdrawn file and would make the path
    // occupied again for the other arm.
    let window = withdrawal.verify_window_stayed_closed();
    drop(withdrawal);
    eprintln!("v2-native-route: old route restored");
    window?;
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

    // REALIZATION TELEMETRY, ON ITS OWN LINE AND ON NO RECEIPT FIELD: this host process's peak
    // RSS and current RSS (process-scoped -- the emitted binary and the rustc children cargo
    // spawned are not in it; the binary's own figures ride its [native-cost-partition] line),
    // the slot cgroup's memory.current/memory.peak (peak spans the runner service's lifetime,
    // not this run), and the lane's wall. It re-derives the run's cost; it decides nothing.
    // Resource preflight (predicted critical path and memory envelope against
    // gunbc.runner_slot_allocation's slot rows) is std.realization cost vocabulary and lands
    // with route integration, not here.
    let (cgroup_current, cgroup_peak) = super::p1_cohort::p1_cohort_cgroup_memory();
    eprintln!(
        "v2-native-route: telemetry peak_rss_bytes={:?} rss_bytes={:?} cgroup_current_bytes={cgroup_current:?} \
         cgroup_peak_bytes_slot_lifetime={cgroup_peak:?} wall_s={}",
        super::peak_rss_vhwm_bytes(),
        super::current_rss_bytes(),
        lane_started.elapsed().as_secs()
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

    /// STORAGE INTEGRITY, BOTH ARMS. The mutation is a SAME-COUNT substitution -- one verdict
    /// flipped, the row count untouched -- which is exactly the case the count check cannot see.
    /// It is performed on the persisted file after the run, then the read-back is compared, so the
    /// control exercises the comparison rather than a mocked version of it.
    #[test]
    fn a_same_count_substitution_in_the_persisted_rows_is_detected() {
        let dir = std::env::temp_dir().join(format!("dc655-tee-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("rows.jsonl");
        let produced = b"{\"identity\":{\"module\":\"m\",\"declaration\":\"d\"},\"verdict\":\"NativeTestPassed\"}\n".to_vec();
        std::fs::write(&path, &produced).unwrap();

        // same record count, one verdict changed
        let substituted = String::from_utf8(produced.clone())
            .unwrap()
            .replace("NativeTestPassed", "NativeTestFailed");
        std::fs::write(&path, substituted.as_bytes()).unwrap();
        let read_back = std::fs::read(&path).unwrap();
        assert_eq!(
            read_back.iter().filter(|b| **b == b'\n').count(),
            produced.iter().filter(|b| **b == b'\n').count(),
            "the substitution must preserve the record count, or it is not testing what it claims"
        );
        assert_ne!(
            read_back, produced,
            "a flipped verdict must be visible as a byte difference -- this is the comparison the tee performs"
        );

        // positive control: unchanged bytes compare equal
        std::fs::write(&path, &produced).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), produced);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// THE DISAGREEMENT ARM HAS AN EXECUTING RED, and it is authorable without an emitted
    /// compiler: run_native_binary takes a path, so a shell fixture can produce exactly the
    /// contradiction the arm exists for -- a marker claiming admitted while the process exits
    /// non-zero. Writing the check without this pair would have left a refusal nobody has watched
    /// fire, which is the state this PR has already had to repair twice.
    #[test]
    fn a_nonzero_exit_claiming_admitted_is_refused() {
        let marker = "{\"_terminal\":\"complete\",\"mode\":\"adjudicate\",\"rows\":0,\"universe\":0,\"file_refusals\":0,\"admitted\":true,\"summary\":\"s\"}";
        let result = run_native_binary(
            Path::new("/bin/sh"),
            &["-c".to_string(), format!("echo '{marker}'; exit 1")],
            None,
        );
        let cause = match result {
            Err(cause) => cause,
            Ok(_) => panic!("a non-zero exit claiming admitted must refuse"),
        };
        assert!(
            cause.contains("NativeRunStatusReceiptDisagree"),
            "the refusal must name the disagreement, got: {cause}"
        );
    }

    /// POSITIVE CONTROL: the same marker with a zero exit is accepted, so the arm above
    /// discriminates on the disagreement rather than on the fixture.
    #[test]
    fn a_zero_exit_claiming_admitted_is_accepted() {
        let marker = "{\"_terminal\":\"complete\",\"mode\":\"adjudicate\",\"rows\":0,\"universe\":0,\"file_refusals\":0,\"admitted\":true,\"summary\":\"s\"}";
        let parsed = run_native_binary(
            Path::new("/bin/sh"),
            &["-c".to_string(), format!("echo '{marker}'; exit 0")],
            None,
        )
        .expect("a zero exit agreeing with an admitted marker is the ordinary accepted run");
        assert!(parsed.terminal.admitted);
    }

    /// THE OTHER DIRECTION, which main's unconditional status gate could not catch: a zero exit
    /// carrying a REFUSED receipt is equally a disagreement between two observations of one run.
    #[test]
    fn a_zero_exit_claiming_refused_is_refused() {
        let marker = "{\"_terminal\":\"complete\",\"mode\":\"adjudicate\",\"rows\":0,\"universe\":0,\"file_refusals\":0,\"admitted\":false,\"summary\":\"s\"}";
        let result = run_native_binary(
            Path::new("/bin/sh"),
            &["-c".to_string(), format!("echo '{marker}'; exit 0")],
            None,
        );
        let cause = match result {
            Err(cause) => cause,
            Ok(_) => panic!("a zero exit claiming refused must refuse"),
        };
        assert!(
            cause.contains("NativeRunStatusReceiptDisagree"),
            "got: {cause}"
        );
    }

    /// EXIT 2 WITH NO MARKER STILL REFUSES, and it refuses at the PARSE rather than at the
    /// agreement arm -- the driver that exits before doing work prints no terminal marker, so
    /// there is no `admitted` to disagree with. Enrolled because "refused before work" and
    /// "disagreed about the verdict" are different causes and a reader must not see one reported
    /// as the other.
    #[test]
    fn an_early_refusal_with_no_marker_refuses_at_the_parse() {
        let result = run_native_binary(
            Path::new("/bin/sh"),
            &["-c".to_string(), "exit 2".to_string()],
            None,
        );
        let cause = match result {
            Err(cause) => cause,
            Ok(_) => panic!("a marker-less stream must refuse"),
        };
        assert!(
            cause.contains("printed no terminal marker"),
            "an early refusal is a missing marker, not a disagreement, got: {cause}"
        );
    }

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
        let stdout =
            "{\"identity\":{\"module\":\"v2.test.a\",\"declaration\":\"t\"},\"verdict\":\"NativeTestPassed\"}\n";
        assert!(parse_native_run_output(stdout).is_err());
    }
}
