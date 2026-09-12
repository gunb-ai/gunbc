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
    // The invocation resolves and binds the one compiler the build runs under and takes its
    // identity from the crate's own directory; a compiler that cannot be resolved or named is
    // a refusal before the build is paid for.
    let invocation =
        super::emitted_closure_compile_host::probe_cargo_invocation(&crate_dir, &workspace)
            .map_err(|cause| format!("V2-NATIVE REFUSAL cause={cause}"))?;
    let rustc = invocation.rustc_identity.clone();
    let verdict = super::emitted_closure_compile_host::run_cargo(
        &crate_dir,
        &workspace,
        "v2_native_lane_carries_no_mutation_probe",
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
// THE TEE IS ITS OWN CONTROL. A persisted receipt that disagrees with the verdict path is worse
// than none, because it will be read as authoritative: the row count recovered FROM THE FILE is
// compared against the count the terminal marker declares, and a mismatch REFUSES rather than
// reporting the smaller number. That also means this cannot silently become a second, divergent
// source of truth -- it agrees with the parse or it stops the line.
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
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
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
        std::fs::write(path, &stdout).map_err(|e| {
            format!(
                "V2-NATIVE REFUSAL cause=NativeRunRowsUnwritable — writing {}: {e}",
                path.display()
            )
        })?;
        eprintln!(
            "v2-native-route: driver rows persisted to {} ({} bytes)",
            path.display(),
            stdout.len()
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
