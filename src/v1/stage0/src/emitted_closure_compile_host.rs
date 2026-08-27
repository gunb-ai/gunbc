//! Host realization for the required `emit-compile` phase: emit one entry's closure,
//! write it as a crate, and run cargo over it.
//!
//! WHY THIS MODULE EXISTS. DESIGN's Building-&-checks section carries a declared rung drop
//! headed "A BLOCKING EMIT-STAGE DIAGNOSTIC CAN SIT ON MAIN INDEFINITELY WITH NO REQUIRED
//! PHASE THAT FAILS", and its restoration trigger names a required phase that emits over a
//! closure and compiles it. The `v2-emission` phase beside this one emits and stops: its own
//! header says so in as many words -- what it does not catch is "a rustc error in the emitted
//! tree (nothing here compiles the emission)". This module is that missing conjunct, and it is
//! deliberately the SAME PRODUCER (`compile_entry_emission`), so a green there and a green here
//! cannot be two different facts about two different emissions.
//!
//! THE SUBJECT IS A CLOSURE, NOT A FILE. DESIGN's row measures its own specimen as reachable
//! from an entry whose closure INCLUDES the offending call site and unreachable from the file
//! that HOLDS it, and measures the holding module compiling clean in isolation. So a per-file
//! or per-module check answers a different question; the subject here is the emitted closure of
//! a declared entry, compiled whole.
//!
//! THE DISCRIMINATION IS EXECUTED, NOT ASSERTED. A phase that runs cargo and cannot go red is
//! worse than absent (DESIGN 4b): it gets cited as coverage. So every run of this phase
//! establishes its own red BY MUTATION rather than by inspection -- baseline green, then ONE
//! injected fault in ONE emitted file which must fail ALONE, then a byte-exact restore which
//! must return the tree to green. A run in which the mutated tree still compiles FAILS THE
//! PHASE, because at that point the cargo verdict is known to be insensitive to the bytes it
//! was handed and the green above it carries no information. The restore is as much of the
//! evidence as the failure is: without it, a red could be residue from the emission rather
//! than from the fault. A FAILED RESTORE IS TERMINAL FOR THE RUN and is not recoverable by
//! re-running the phase -- see `run_required_emit_compile`, which stops there and reports every
//! later entry as `NotExecuted` rather than measuring it through a target directory whose state
//! is no longer known.
//!
//! WHAT THIS PHASE IS NOT. It carries no baseline, no diagnostic count and no ratchet. Cargo's
//! own exit status is the whole verdict, and warnings are not errors here. Pinning a
//! diagnostic population measured on the current tree would be the tree-copied oracle DESIGN 5
//! rejects; an identity-grain debt contract over the emitted population is a separate
//! construction with a separate argument.

use std::path::{Path, PathBuf};

use super::{
    ci_layer_roots_authority_content, compile_entry_emission, process_workspace_root,
    string_list_data_from_ci_layer_roots_source, CompileDisposition, CompileRun,
};
use crate::extdeps_cargo::{CargoDepSource, CargoDependency};
use crate::extdeps_cargo_version::render_cargo_package_header_prefix;
use crate::gunbc_stage0_crate_partition_generated::{
    GeneratedPartitionCrateKind, GeneratedPartitionCrateRow,
};
use crate::v1_compiler_stage0_crates::{
    render_stage0_crate_dep, render_stage0_crate_features_section,
    stage0_foundation_runtime_dependencies, stage0_partition_row_features,
};

const REQUIRED_EMIT_COMPILE_ENTRIES_DATA_NAME: &str = "required_emit_compile_entries";

/// The crate name the emitted closure is compiled under — DERIVED PER ENTRY, not shared.
///
/// WHY IT IS NOT ONE NAME FOR EVERY ENTRY, WHICH IS WHAT IT WAS. The entries share one
/// `CARGO_TARGET_DIR` deliberately: that is what keeps dependency artifacts warm across a roster,
/// and rebuilding `im`, `serde` and the seed crate once per entry would multiply the phase's cost
/// by its roster size. But a shared target directory plus one package name and version means the
/// only thing separating two entries' fingerprints is cargo's use of the manifest path. That is
/// an implementation detail of a tool, load-bearing for a merge gate, and nothing here states it.
///
/// THE FAILURE IT WOULD PRODUCE IS FAIL-OPEN, WHICH IS WHY IT IS WORTH A NAME RATHER THAN A
/// COMMENT SAYING CARGO HANDLES IT. If one entry's build were ever judged fresh against another's
/// artifacts, cargo would replay the other's cached diagnostics — and a replayed clean compile is
/// byte-identical in the output to a real one. The arm would report `Completed status=0` for an
/// entry it never compiled, and the gate would go green over it.
///
/// Deriving the name from the entry makes each probe crate its own package, so the fingerprints
/// cannot alias whatever cargo keys on. Dependencies are separate packages and stay shared, so
/// the warmth the shared target dir buys is untouched.
fn probe_package_name(entry: &str) -> String {
    let slug: String = entry
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    format!("gunbc-emitted-closure-{slug}")
}

/// THE INJECTED FAULT. A type error rather than a syntax error, deliberately: a syntax error
/// would also be caught by anything that merely parses the file, so it cannot discriminate a
/// cargo verdict from a cheaper reader. `E0308` requires rustc to have type-checked the
/// module, which is exactly the reach being claimed. The name is unique enough that it cannot
/// collide with emitted output, and the item is `pub` so no dead-code lint can elide it.
/// The symbol the injected item declares. The faulted arm's diagnostics must NAME it: that is
/// what makes the red attributable to this phase's own fault rather than to anything else that
/// happened to be wrong in the emitted tree at the same moment.
const MUTATION_PROBE_SYMBOL: &str = "EMIT_COMPILE_MUTATION_PROBE";

const MUTATION_ITEM: &str =
    "\npub const EMIT_COMPILE_MUTATION_PROBE: u8 = \"the phase's own discriminating red\";\n";

/// The `.dag` entry paths whose emitted closure a required phase compiles, read live from
/// `gunbc.ci_layer_roots` `required_emit_compile_entries`.
///
/// A `List<String>` for the same reason the emission roster is one: the axis is WHICH ENTRIES,
/// so a second entry is a row and never a second host reader.
pub fn required_emit_compile_entries() -> Vec<String> {
    string_list_data_from_ci_layer_roots_source(
        ci_layer_roots_authority_content(),
        REQUIRED_EMIT_COMPILE_ENTRIES_DATA_NAME,
    )
}

/// What cargo did, at the grain the phase can act on.
///
/// The three arms are the ones `PartitionCompileOutcome` already separates in this seed, and
/// for the reason recorded there: a process killed without an exit status reports an empty
/// diagnostic population, which renders identically to a clean build unless the disposition
/// says otherwise.
#[derive(Debug, Clone)]
pub enum CargoVerdict {
    /// The toolchain was never invoked.
    NotAttempted { reason: String },
    /// Launched, and reached no exit status of its own -- killed, or the spawn failed.
    DidNotComplete { detail: String },
    /// Ran to completion and reported its own exit status.
    ///
    /// `probe_line` carries the first diagnostic line naming the injected probe symbol, scanned
    /// from the WHOLE stderr rather than from `stderr_tail`. The two are different questions and
    /// conflating them would reintroduce the defect this field exists for: the tail is the last
    /// 20 lines, kept so a human can read a failure, and a genuine `E0308` for the injected item
    /// can sit well above it when other diagnostics follow. Deciding attribution from the tail
    /// would then fail a run whose fault WAS refused, for the reason that the receipt was short.
    Completed {
        status: i32,
        stderr_tail: String,
        probe_line: Option<String>,
    },
}

/// Only a completed, zero-status run compiled. Every other arm -- including the one that never
/// launched -- is a refusal.
///
/// FREE FUNCTIONS RATHER THAN `impl` METHODS, throughout this module, and it is deliberate: an
/// `impl` method has no `DeclarationRef` spelling (`std.decl_ref` offers `WholeDeclaration` or
/// `NamedField`, and neither names a method on an impl block), so every method would grow the
/// uncitable-item class `gunbc.seed_growth_admission` reports in
/// `seed_growth_uncitable_item_keys`. `v1_compiler.declaration_index` took the same route for
/// the same reason.
pub fn cargo_verdict_compiled(verdict: &CargoVerdict) -> bool {
    matches!(verdict, CargoVerdict::Completed { status: 0, .. })
}

pub fn cargo_verdict_summary(verdict: &CargoVerdict) -> String {
    match verdict {
        CargoVerdict::NotAttempted { reason } => format!("NotAttempted reason={reason}"),
        CargoVerdict::DidNotComplete { detail } => format!("DidNotComplete detail={detail}"),
        CargoVerdict::Completed { status, .. } => format!("Completed status={status}"),
    }
}

/// The diagnostic line naming the injected probe symbol, if the run produced one.
pub fn cargo_verdict_probe_line(verdict: &CargoVerdict) -> Option<&str> {
    match verdict {
        CargoVerdict::Completed { probe_line, .. } => probe_line.as_deref(),
        _ => None,
    }
}

pub fn cargo_verdict_stderr_tail(verdict: &CargoVerdict) -> &str {
    match verdict {
        CargoVerdict::Completed { stderr_tail, .. } => stderr_tail.as_str(),
        _ => "",
    }
}

/// WHICH FILE THE FAULT WENT INTO, carried rather than inferred.
///
/// A closure member is the stronger subject -- it establishes that the cargo verdict reaches
/// past the entry's own bytes into the closure the entry pulled in, which is the property
/// DESIGN's row turns on. `EntryModule` is the honest fallback for a closure whose only member
/// is the entry, and naming it means a reader can tell the weaker measurement from the stronger
/// one instead of assuming the stronger.
#[derive(Debug, Clone)]
pub enum MutationSubject {
    ClosureMember { rust_module: String },
    EntryModule { rust_module: String },
}

pub fn mutation_subject_rust_module(subject: &MutationSubject) -> &str {
    match subject {
        MutationSubject::ClosureMember { rust_module } => rust_module.as_str(),
        MutationSubject::EntryModule { rust_module } => rust_module.as_str(),
    }
}

pub fn mutation_subject_name(subject: &MutationSubject) -> &'static str {
    match subject {
        MutationSubject::ClosureMember { .. } => "ClosureMember",
        MutationSubject::EntryModule { .. } => "EntryModule",
    }
}

/// WHETHER THIS RUN'S CARGO VERDICT IS SENSITIVE TO THE BYTES IT WAS HANDED.
///
/// Only `Discriminated` is a pass. Every other arm says the baseline green above it carries no
/// information, which is a phase failure rather than a note -- the whole point of the arm is
/// that a decoration must not be able to report coverage.
#[derive(Debug, Clone)]
pub enum MutationVerdict {
    /// No fault was injected. Carries why: a baseline that never went green has nothing to
    /// discriminate against, and a tree with no writable module has nowhere to put the fault.
    NotAttempted { reason: String },
    /// The fault went in and cargo still compiled the tree. THE INSTRUMENT IS NOT MEASURING
    /// WHAT IT CLAIMS TO.
    NotDiscriminating { detail: String },
    /// The fault produced a red, and the restore did not return the tree to the state it
    /// started in -- either the bytes differ, or the restored tree does not compile. The red is
    /// then unattributable: it may be residue rather than the fault.
    RestoreFailed { detail: String },
    /// Red under the fault, green again after a byte-exact restore.
    Discriminated {
        subject: MutationSubject,
        red_line: String,
    },
}

pub fn mutation_verdict_discriminated(verdict: &MutationVerdict) -> bool {
    matches!(verdict, MutationVerdict::Discriminated { .. })
}

pub fn mutation_verdict_summary(verdict: &MutationVerdict) -> String {
    match verdict {
        MutationVerdict::NotAttempted { reason } => format!("NotAttempted reason={reason}"),
        MutationVerdict::NotDiscriminating { detail } => {
            format!("NotDiscriminating detail={detail}")
        }
        MutationVerdict::RestoreFailed { detail } => format!("RestoreFailed detail={detail}"),
        MutationVerdict::Discriminated { subject, red_line } => format!(
            "Discriminated subject={} module={} red={red_line}",
            mutation_subject_name(subject),
            mutation_subject_rust_module(subject)
        ),
    }
}

/// One entry's whole story, and every verdict is reached THROUGH the stage that produced it.
///
/// `EmissionRefused` and `CrateNotWritten` have no cargo verdict to carry, so "cargo found
/// nothing wrong" and "cargo was never reached" cannot share a spelling -- the
/// execution-provenance-loss failure DESIGN names, which is exactly what a `passed: bool`
/// beside an optional cause would have reintroduced.
#[derive(Debug, Clone)]
pub enum EmitCompileOutcome {
    /// The emission transaction did not complete. The emitted tree does not exist, so nothing
    /// downstream ran.
    EmissionRefused {
        entry: String,
        stage: String,
        cause: String,
    },
    /// Emission completed and the crate could not be laid out on disk.
    CrateNotWritten { entry: String, cause: String },
    /// The entry was never reached, because an earlier entry's restore failed and a failed
    /// restore is TERMINAL for the run. Carries the entry that ended it, so "not reached" can
    /// never be read as "reached and clean".
    NotExecuted { entry: String, cause: String },
    /// The crate exists and both arms ran.
    Measured {
        entry: String,
        crate_dir: String,
        emitted_files: usize,
        baseline: CargoVerdict,
        mutation: MutationVerdict,
    },
}

/// A pass is a completed emission, a green baseline AND an executed discrimination. The third
/// conjunct is not decoration: without it the first two are satisfied by an instrument that
/// cannot fail.
pub fn emit_compile_outcome_passed(outcome: &EmitCompileOutcome) -> bool {
    match outcome {
        EmitCompileOutcome::Measured {
            baseline, mutation, ..
        } => cargo_verdict_compiled(baseline) && mutation_verdict_discriminated(mutation),
        _ => false,
    }
}

pub fn emit_compile_outcome_summary(outcome: &EmitCompileOutcome) -> String {
    match outcome {
        EmitCompileOutcome::EmissionRefused {
            entry,
            stage,
            cause,
        } => {
            format!("{entry} EmissionRefused stage={stage} cause={cause}")
        }
        EmitCompileOutcome::CrateNotWritten { entry, cause } => {
            format!("{entry} CrateNotWritten cause={cause}")
        }
        EmitCompileOutcome::NotExecuted { entry, cause } => {
            format!("{entry} NotExecuted cause={cause}")
        }
        EmitCompileOutcome::Measured {
            entry,
            crate_dir,
            emitted_files,
            baseline,
            mutation,
        } => format!(
            "{entry} Measured files={emitted_files} crate={crate_dir} baseline=[{}] mutation=[{}]",
            cargo_verdict_summary(baseline),
            mutation_verdict_summary(mutation)
        ),
    }
}

/// The manifest for the probe crate, rendered from the modeled cargo authorities rather than
/// authored as markup.
///
/// The package header comes from `extdeps.rust.version` `render_cargo_package_header_prefix`,
/// the dependency rows from `v1.compiler.stage0_crates`
/// `stage0_foundation_runtime_dependencies` -- the seed's own runtime dependency set, which is
/// what emitted code links against -- and each row is rendered by that module's
/// `render_stage0_crate_dep`. No `[lib]` section is written because `src/lib.rs` is cargo's own
/// default library path, so naming it would be a second spelling of a fact cargo already owns.
///
/// A hand-authored TOML string was available and is deliberately not used: the corpus already
/// carries one (`tools.self_host_curated_seed_linked_harness`
/// `cssl_v1_compiled_probe_lib_cargo_toml`), it is marked scaffold debt in its own module for
/// being concat-authored markup, and adding a required gate as a consumer of it would have
/// pinned that debt open on the merge path.
fn probe_manifest(workspace: &Path, entry: &str) -> String {
    let mut deps: Vec<CargoDependency> = stage0_foundation_runtime_dependencies()
        .iter()
        .map(|dep| (**dep).clone())
        .collect();
    // The emitted closure links against the seed crate for the runtime surface it does not
    // emit for itself (`v1_rt` and friends). A path dependency, absolute, because the probe
    // crate is written outside the repository and a relative path would not resolve from it.
    deps.push(CargoDependency {
        name: "v1-compiler".to_string(),
        source: std::rc::Rc::new(CargoDepSource::LocalPathDep {
            path: workspace.join("src/v1/stage0").display().to_string(),
        }),
    });
    let rendered: String = deps
        .into_iter()
        .map(|dep| render_stage0_crate_dep(std::rc::Rc::new(dep)))
        .collect::<Vec<_>>()
        .join("");
    // THE FEATURE SECTION IS NOT OPTIONAL, AND CI IS WHERE ITS ABSENCE BITES.
    //
    // The emitted `v1_rt.rs` gates on `#[cfg(feature = "text_lookup_work_counter")]`. A crate
    // that references a feature it does not declare earns the `unexpected_cfgs` lint, which is a
    // WARNING locally and a hard ERROR under CI's `RUSTFLAGS=-D warnings` — so the probe crate
    // compiled clean on a workstation and failed `status=101` on every entry in CI, with the
    // baseline arm reporting a red that had nothing to do with the emitted closure.
    //
    // Rendered from the same modeled authority the partition crates use
    // (`stage0_partition_row_features` / `render_stage0_crate_features_section`) rather than
    // authored here: the corpus already carries two hand-concatenated `[features]` blocks for
    // exactly this reason, each with a note explaining the failure, and adding a third string
    // would be the second representation those notes are evidence against.
    let features = render_stage0_crate_features_section(stage0_partition_row_features(
        std::rc::Rc::new(GeneratedPartitionCrateRow {
            package_name: probe_package_name(entry),
            crate_dir: String::new(),
            // The foundation kind is the one that carries the feature the emitted `v1_rt.rs`
            // gates on, and the emitted closure always contains `v1_rt`.
            kind: GeneratedPartitionCrateKind::GeneratedFoundationCrate,
            // `im::Vector`, not `std::Vec`: the generated row's list fields are the corpus
            // `List<T>`, which that module aliases to `im::Vector`.
            modules: std::rc::Rc::new(im::Vector::new()),
            reexport_packages: std::rc::Rc::new(im::Vector::new()),
            carries_non_empty_wrappers: false,
        }),
    ));
    format!(
        "{}\nedition = \"2021\"\n{features}\n[dependencies]\n{rendered}",
        render_cargo_package_header_prefix(probe_package_name(entry))
    )
}

/// Where one entry's probe crate is written. Outside the repository deliberately: a crate under
/// the workspace root is inferred into the workspace by cargo and would have to declare its own
/// `[workspace]` to escape, which is a manifest fact invented to work around its own location.
fn probe_root() -> PathBuf {
    std::env::temp_dir().join("gunbc-emit-compile")
}

fn probe_crate_dir(entry: &str) -> PathBuf {
    let slug: String = entry
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    probe_root().join(slug)
}

/// Write the emitted Rust files plus a manifest, and return the crate directory.
fn write_probe_crate(run: &CompileRun, entry: &str) -> Result<(PathBuf, usize), String> {
    let emission = run
        .emissions
        .iter()
        .find(|emission| emission.target_name == "rust")
        .ok_or_else(|| "the emission carries no rust target".to_string())?;
    let dir = probe_crate_dir(entry);
    // A STALE TREE IS NOT A SUBJECT. The previous run's bytes under the same slug would let a
    // module deleted from the closure keep compiling, so the directory is removed rather than
    // written over.
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src"))
        .map_err(|e| format!("creating {}: {e}", dir.display()))?;
    let mut written = 0usize;
    for file in emission.result.files.iter() {
        let path = dir.join(&*file.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("creating {}: {e}", parent.display()))?;
        }
        std::fs::write(&path, &*file.content)
            .map_err(|e| format!("writing {}: {e}", path.display()))?;
        written += 1;
    }
    if !dir.join("src/lib.rs").is_file() {
        return Err(format!(
            "the emission wrote no src/lib.rs into {} — there is no crate root to compile",
            dir.display()
        ));
    }
    std::fs::write(
        dir.join("Cargo.toml"),
        probe_manifest(&process_workspace_root(), entry),
    )
    .map_err(|e| format!("writing the manifest into {}: {e}", dir.display()))?;
    Ok((dir, written))
}

/// Run cargo over the probe crate.
///
/// `build --release` INTO THE WORKSPACE TARGET DIRECTORY, and both halves are cost decisions
/// with one reason. The lane's own first step is `cargo build --release -p v1-compiler --bins`,
/// so the seed crate this probe depends on is already compiled there under exactly that
/// profile; a `check`, or a private target directory, would share no fingerprint with it and
/// would rebuild the whole dependency graph inside a required phase. Sharing it means the
/// baseline arm compiles the emitted crate and nothing else, and the two further arms are
/// incremental on top of that.
///
/// The phases inside one required run are sequential in one process, so nothing else is holding
/// cargo's lock on that directory while this runs.
fn run_cargo(crate_dir: &Path, workspace: &Path) -> CargoVerdict {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut command = std::process::Command::new(&cargo);
    command
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(crate_dir.join("Cargo.toml"))
        .env("CARGO_TARGET_DIR", workspace.join("target"))
        .current_dir(crate_dir);
    match command.output() {
        Err(e) => CargoVerdict::DidNotComplete {
            detail: format!("spawning {cargo} failed: {e}"),
        },
        Ok(output) => match output.status.code() {
            None => CargoVerdict::DidNotComplete {
                detail: format!("{cargo} terminated by signal without an exit status"),
            },
            Some(status) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let tail: Vec<&str> = stderr.lines().rev().take(20).collect();
                let probe_line = stderr
                    .lines()
                    .find(|line| line.contains(MUTATION_PROBE_SYMBOL))
                    .map(|line| line.trim().to_string());
                CargoVerdict::Completed {
                    status,
                    stderr_tail: tail.into_iter().rev().collect::<Vec<_>>().join("\n"),
                    probe_line,
                }
            }
        },
    }
}

/// The rust module basenames the emitted `lib.rs` declares, in its own order.
fn closure_modules(lib_rs: &Path) -> Vec<String> {
    let Ok(content) = std::fs::read_to_string(lib_rs) else {
        return Vec::new();
    };
    content
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("pub mod ")
                .and_then(|rest| rest.strip_suffix(';'))
                .map(|m| m.trim().to_string())
        })
        .collect()
}

/// Pick the file the fault goes into: a closure member other than the entry where one exists,
/// the entry itself otherwise.
fn mutation_subject(crate_dir: &Path, entry_module: &str) -> Option<MutationSubject> {
    let modules = closure_modules(&crate_dir.join("src/lib.rs"));
    if let Some(member) = modules
        .iter()
        .find(|m| m.as_str() != entry_module && crate_dir.join(format!("src/{m}.rs")).is_file())
    {
        return Some(MutationSubject::ClosureMember {
            rust_module: member.clone(),
        });
    }
    if crate_dir.join(format!("src/{entry_module}.rs")).is_file() {
        return Some(MutationSubject::EntryModule {
            rust_module: entry_module.to_string(),
        });
    }
    None
}

/// THE DISCRIMINATING RED, ESTABLISHED BY MUTATION AND RESTORED BEFORE THE PHASE REPORTS.
///
/// One fault, in one file, failing alone -- the baseline immediately above it is the control,
/// and the restore immediately after it is the second control. A round in which several things
/// change at once would establish that cargo responds to damage, not that this instrument reads
/// this closure.
fn establish_discriminating_red(
    crate_dir: &Path,
    workspace: &Path,
    entry_module: &str,
) -> MutationVerdict {
    let Some(subject) = mutation_subject(crate_dir, entry_module) else {
        return MutationVerdict::NotAttempted {
            reason: format!(
                "no writable emitted module under {} to carry the fault",
                crate_dir.display()
            ),
        };
    };
    let path = crate_dir.join(format!("src/{}.rs", mutation_subject_rust_module(&subject)));
    let original = match std::fs::read_to_string(&path) {
        Ok(bytes) => bytes,
        Err(e) => {
            return MutationVerdict::NotAttempted {
                reason: format!("reading {}: {e}", path.display()),
            }
        }
    };
    let mutated = format!("{original}{MUTATION_ITEM}");
    if let Err(e) = std::fs::write(&path, &mutated) {
        return MutationVerdict::NotAttempted {
            reason: format!("writing the fault into {}: {e}", path.display()),
        };
    }

    let red = run_cargo(crate_dir, workspace);

    // THE RESTORE RUNS WHATEVER THE FAULTED ARM ANSWERED. Leaving a faulted tree behind would
    // make the next run's baseline red for a reason that has nothing to do with the corpus.
    let restore_write = std::fs::write(&path, &original);
    let restored_bytes = std::fs::read_to_string(&path).unwrap_or_default();

    // THE FAULTED ARM MUST HAVE COMPLETED, AND ITS RED MUST BE ATTRIBUTABLE TO THE FAULT.
    //
    // A NONZERO EXIT IS NOT EVIDENCE ON ITS OWN, which is the hole this block closes. Cargo can
    // be killed, fail to spawn, run out of disk, or die for a reason having nothing to do with
    // the injected item — and `!cargo_verdict_compiled(&red)` is true in every one of those
    // cases. Accepting them would let the phase report `Discriminated` while establishing
    // nothing about sensitivity to the mutation, and then green a merge gate on it: a fabricated
    // red, which is the fabricated-plausible-output failure aimed at the phase's own evidence.
    //
    // THIS IS NOT HYPOTHETICAL. Verifying the blunted-mutation arm, a concurrent run produced
    // exactly this shape — a `Discriminated` verdict whose red line quoted a `#[cfg]` WARNING
    // over a cargo run that had actually said `Finished`. The probe-root lock closes the cause;
    // this closes the arm that accepted the result, and the two are different defects.
    //
    // So the arm demands three things of the faulted run, in order of what they rule out:
    //   1. `Completed` — cargo ran to a verdict, so `NotAttempted`/`DidNotComplete` fail rather
    //      than passing as a red;
    //   2. a nonzero status — it refused;
    //   3. a diagnostic naming THE INJECTED SYMBOL — it refused for OUR reason. Requiring the
    //      symbol rather than merely the code is what distinguishes the injected fault from an
    //      unrelated `E0308` that was already in the emitted tree; the code alone would accept a
    //      pre-existing type error as the phase's own evidence.
    match &red {
        CargoVerdict::Completed { status: 0, .. } => {
            return MutationVerdict::NotDiscriminating {
                detail: format!(
                    "cargo compiled {} with a deliberate type error appended to src/{}.rs — \
                     the verdict is not a function of the emitted bytes, so the green baseline \
                     beside it carries no information",
                    crate_dir.display(),
                    mutation_subject_rust_module(&subject)
                ),
            };
        }
        CargoVerdict::NotAttempted { reason } => {
            return MutationVerdict::NotDiscriminating {
                detail: format!(
                    "the faulted arm never ran cargo ({reason}) — a run that did not happen is \
                     not a red, and treating its absence as one would fabricate the evidence \
                     this phase exists to establish"
                ),
            };
        }
        CargoVerdict::DidNotComplete { detail } => {
            return MutationVerdict::NotDiscriminating {
                detail: format!(
                    "the faulted arm did not reach a cargo verdict ({detail}) — a killed or \
                     unspawnable cargo is not evidence that the injected fault was refused"
                ),
            };
        }
        CargoVerdict::Completed { .. } => {}
    }
    let Some(attributed) = cargo_verdict_probe_line(&red) else {
        return MutationVerdict::NotDiscriminating {
            detail: format!(
                "the faulted arm refused, but no diagnostic names {MUTATION_PROBE_SYMBOL} — the \
                 red is not attributable to the injected fault, so it establishes nothing about \
                 sensitivity to the emitted bytes"
            ),
        };
    };
    let attributed = attributed.to_string();

    if let Err(e) = restore_write {
        return MutationVerdict::RestoreFailed {
            detail: format!("restoring {}: {e}", path.display()),
        };
    }
    if restored_bytes != original {
        return MutationVerdict::RestoreFailed {
            detail: format!(
                "{} did not return to its emitted bytes after the fault was removed",
                path.display()
            ),
        };
    }
    let restored = run_cargo(crate_dir, workspace);
    if !cargo_verdict_compiled(&restored) {
        return MutationVerdict::RestoreFailed {
            detail: format!(
                "the restored tree does not compile ({}) — the red above it cannot be \
                 attributed to the injected fault",
                cargo_verdict_summary(&restored)
            ),
        };
    }

    // The reported line is the diagnostic that NAMES THE FAULT, which is the same line the
    // attribution check above accepted -- so the receipt a reader sees is the evidence the arm
    // actually decided on, rather than a separately-chosen line that could disagree with it.
    MutationVerdict::Discriminated {
        subject,
        red_line: attributed,
    }
}

/// The rust module basename an entry `.dag` file emits under, from its own `module` line.
fn entry_rust_module(entry: &str, workspace: &Path) -> Result<String, String> {
    let path = workspace.join(entry);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("reading the entry {}: {e}", path.display()))?;
    content
        .lines()
        .find(|line| line.starts_with("module "))
        .map(|line| line.trim_start_matches("module ").trim().replace('.', "_"))
        .ok_or_else(|| format!("the entry {entry} declares no module line"))
}

/// One entry, end to end.
pub fn run_emit_compile_entry(source_roots: &[String], entry: &str) -> EmitCompileOutcome {
    // PROGRESS IS REPORTED AS THE STAGE IS ENTERED, NOT WHEN THE ENTRY FINISHES.
    //
    // This is not decoration. Every stage below is a long host effect -- a whole-index emission,
    // then up to three cargo invocations -- and a phase that reports only on completion renders a
    // HANG and a KILL identically to a reader, and both identically to a slow run. That is the
    // execution-provenance loss DESIGN names, arriving in the instrument's own output: measured
    // the hard way, when two verification runs died inside this function and produced no line at
    // all, so nothing in the log distinguished "still emitting" from "killed".
    eprintln!("emit-compile: {entry} emitting");
    let workspace = process_workspace_root();
    let run = compile_entry_emission(
        source_roots,
        entry,
        true,
        crate::v1_compiler_artifact::RenderTarget::Rust,
    );
    match &run.disposition {
        CompileDisposition::Refused { phase, cause } => {
            return EmitCompileOutcome::EmissionRefused {
                entry: entry.to_string(),
                stage: phase.clone(),
                cause: cause.clone(),
            }
        }
        CompileDisposition::NotExecuted {
            earlier_phase,
            cause,
        } => {
            return EmitCompileOutcome::EmissionRefused {
                entry: entry.to_string(),
                stage: earlier_phase.clone(),
                cause: cause.clone(),
            }
        }
        CompileDisposition::Completed { .. } => {}
    }

    let (crate_dir, emitted_files) = match write_probe_crate(&run, entry) {
        Ok(pair) => pair,
        Err(cause) => {
            return EmitCompileOutcome::CrateNotWritten {
                entry: entry.to_string(),
                cause,
            }
        }
    };
    let entry_module = match entry_rust_module(entry, &workspace) {
        Ok(module) => module,
        Err(cause) => {
            return EmitCompileOutcome::CrateNotWritten {
                entry: entry.to_string(),
                cause,
            }
        }
    };

    eprintln!(
        "emit-compile: {entry} emitted {emitted_files} file(s) into {} — cargo baseline",
        crate_dir.display()
    );
    let baseline = run_cargo(&crate_dir, &workspace);
    eprintln!(
        "emit-compile: {entry} baseline {} — mutation",
        cargo_verdict_summary(&baseline)
    );
    // THE DISCRIMINATION IS NOT ATTEMPTED OVER A RED BASELINE, and it says so rather than
    // reporting a red it cannot attribute: a tree that already fails would go red under the
    // fault for a reason the fault did not cause, which is a green control wearing a red one's
    // clothes.
    let mutation = if cargo_verdict_compiled(&baseline) {
        establish_discriminating_red(&crate_dir, &workspace, &entry_module)
    } else {
        MutationVerdict::NotAttempted {
            reason: format!(
                "the baseline did not compile ({}) — a fault injected into a failing tree \
                 discriminates nothing",
                cargo_verdict_summary(&baseline)
            ),
        }
    };

    EmitCompileOutcome::Measured {
        entry: entry.to_string(),
        crate_dir: crate_dir.display().to_string(),
        emitted_files,
        baseline,
        mutation,
    }
}

/// Every configured entry, each run whatever the previous one did -- the stopped-line audit
/// shape the required run already uses: report everything, green nothing.
///
/// AN EMPTY ROSTER REFUSES. Zero entries compiled is not zero breaks; it is the phase failing
/// to reach any subject, and reporting it as clean is the empty-observation narrow.
/// THE NUMERATOR, IN THE SAME UNIT AS THE DENOMINATOR: modules the cover's closures REACHED.
///
/// WHY NOT THE ENTRY COUNT. `covered_entries=8 of 3900 authored modules` is not a fraction — the
/// numerator is entries and the denominator is modules — and it reads as a coverage ratio
/// precisely because it is formatted like one. It understates by a wide and unknown margin, since
/// eight closures reach far more than eight modules, and understating is not the safe direction:
/// a number that looks that bad invites growing the entry roster, which is exactly the move the
/// retirement trigger forbids (a trigger satisfied at forty-one entries leaves the corpus
/// unmeasured). An unpaired count tells no lie; a mismatched fraction does.
///
/// This is the union of the emitted module sets, read from the crates the phase already wrote, so
/// it costs a readdir per entry and no extra compilation. It moves for the right reason: up when
/// the cover reaches new code, unchanged when an unrelated emitter repair lands.
///
/// WHAT IT DOES NOT DISTINGUISH, said here rather than left for a reader to assume: reached AS AN
/// ENTRY and reached ONLY AS A DEPENDENCY are both counted. The second is real coverage of a
/// weaker kind — a dependency module is compiled, but no run ever emits from its own closure, so
/// an emit-stage diagnostic reachable only from ITS entry is still invisible. Splitting the two
/// numerators is strictly better and is not done here.
pub fn emit_compile_modules_reached(outcomes: &[EmitCompileOutcome]) -> usize {
    // `src/lib.rs` IS NOT DEDUPLICABLE BY NAME, AND UNIONING IT WOULD UNDER-COUNT.
    // The emission writes each entry's own root module as `lib.rs` — the compiler refuses the
    // crate outright without one — so every entry contributes a DIFFERENT root under the SAME
    // file name. Unioning names would collapse N distinct roots into one, which is a numerator
    // that shrinks as the cover grows. The roots are therefore counted per measured entry and
    // the dependency modules unioned by name, which is what "reached" means.
    let mut reached: Vec<String> = Vec::new();
    let mut roots = 0usize;
    for outcome in outcomes {
        if let EmitCompileOutcome::Measured { crate_dir, .. } = outcome {
            let src = std::path::Path::new(crate_dir).join("src");
            if let Ok(entries) = std::fs::read_dir(&src) {
                roots += 1;
                for entry in entries.flatten() {
                    if let Some(name) = entry.file_name().to_str() {
                        if name == "lib.rs" {
                            continue;
                        }
                        reached.push(name.to_string());
                    }
                }
            }
        }
    }
    reached.sort();
    reached.dedup();
    reached.len() + roots
}

/// THE SELECTION, WITH ITS REMAINDER CARRIED AT IDENTITY GRAIN.
///
/// WHY THIS REPLACES THE COVERAGE FRACTION RATHER THAN JOINING IT. A fraction — however
/// unit-consistent — is the wrong instrument for an uncovered remainder, because §4b(3) asks a
/// rung drop for a BOUNDED POPULATION and a percentage is not one: it says how much is missing
/// and never WHICH, so nothing downstream can join it, refuse on it, or watch it shrink. The
/// identities are the population. A percentage may stand as context and may never be the gap's
/// identity or its dissolution trigger.
///
/// The universe is authored `.dag` modules under the invoked source roots; the selection is the
/// declared roster; the remainder is the set difference, RETAINED — written to a file beside the
/// run and digested, so the phase's own output names where the unselected identities are rather
/// than summarising them away. Counts and digests are printed; the identities are persisted.
pub struct EmitCompileSelection {
    pub universe: Vec<String>,
    pub selected: Vec<String>,
    pub not_selected: Vec<String>,
}

fn digest_of_identities(identities: &[String]) -> String {
    let mut material = String::new();
    for identity in identities {
        material.push_str(identity);
        material.push('\0');
    }
    super::fnv1a64_digest_of_material(&material)
}

pub fn emit_compile_selection(source_roots: &[String]) -> EmitCompileSelection {
    let mut universe: Vec<String> = Vec::new();
    for root in source_roots {
        let mut files = Vec::new();
        super::collect_dag_files_tolerant(std::path::Path::new(root), &mut files);
        for file in files {
            universe.push(file.to_string_lossy().to_string());
        }
    }
    universe.sort();
    universe.dedup();

    let mut selected = required_emit_compile_entries();
    selected.sort();
    selected.dedup();

    // THE SELECTION IS NOT ASSUMED TO BE A SUBSET, IT IS INTERSECTED. A roster row naming a path
    // the walk does not find is a defect worth seeing rather than a silent membership; it shows
    // up here as a selected identity absent from the universe, and the remainder stays exact.
    let not_selected: Vec<String> = universe
        .iter()
        .filter(|module| !selected.contains(module))
        .cloned()
        .collect();

    EmitCompileSelection {
        universe,
        selected,
        not_selected,
    }
}

pub fn emit_compile_selection_universe_digest(selection: &EmitCompileSelection) -> String {
    digest_of_identities(&selection.universe)
}

pub fn emit_compile_selection_selected_digest(selection: &EmitCompileSelection) -> String {
    digest_of_identities(&selection.selected)
}

pub fn emit_compile_selection_not_selected_digest(selection: &EmitCompileSelection) -> String {
    digest_of_identities(&selection.not_selected)
}

/// Persist the unselected identities beside the run. RETAINED means retained: the phase writes
/// the list rather than reporting its size, so the remainder is a population a later operation can
/// read, and not a number a later reader must trust.
pub fn retain_not_selected_identities(
    selection: &EmitCompileSelection,
    dir: &str,
) -> Result<String, String> {
    let path = std::path::Path::new(dir).join("emit-compile-not-selected.txt");
    let mut body = String::new();
    for identity in &selection.not_selected {
        body.push_str(identity);
        body.push('\n');
    }
    std::fs::create_dir_all(dir)
        .map_err(|e| format!("could not create {dir} to retain the remainder: {e}"))?;
    std::fs::write(&path, body)
        .map_err(|e| format!("could not retain the unselected identities at {path:?}: {e}"))?;
    Ok(path.to_string_lossy().to_string())
}

/// THE ONE REPORT BOTH SURFACES PRINT.
///
/// The required phase and the standalone `--required-emit-compile` mode are two callers of one
/// producer, and a report authored twice is one fact with two authorities — free to disagree about
/// exactly the numbers a reader compares across the two surfaces. So the selection, the remainder
/// and the context line are rendered here, once, and each caller prints what it is given.
///
/// Retention is attempted here and its failure is RETURNED rather than swallowed, because the
/// remainder is the declared population of what this phase does not observe: a run reporting a
/// remainder it could not persist has published a count with nothing behind it.
pub fn emit_compile_report(
    outcomes: &[EmitCompileOutcome],
    source_roots: &[String],
    prefix: &str,
) -> (Vec<String>, Option<String>) {
    let selection = emit_compile_selection(source_roots);
    let mut lines = Vec::new();
    lines.push(format!(
        "{prefix} selection: universe={} {} selected={} {} not_selected={} {}",
        selection.universe.len(),
        emit_compile_selection_universe_digest(&selection),
        selection.selected.len(),
        emit_compile_selection_selected_digest(&selection),
        selection.not_selected.len(),
        emit_compile_selection_not_selected_digest(&selection),
    ));
    let retained_dir = std::env::temp_dir()
        .join("gunbc-emit-compile")
        .to_string_lossy()
        .to_string();
    let retention_error = match retain_not_selected_identities(&selection, &retained_dir) {
        Ok(path) => {
            lines.push(format!(
                "{prefix} remainder retained at {path} (unselected identities, one per line)"
            ));
            None
        }
        Err(err) => {
            lines.push(format!("{prefix} FAILED to retain remainder: {err}"));
            Some(err)
        }
    };
    // Context only, after the identities and never in place of them.
    lines.push(format!(
        "{prefix} context: {} declared entries reach {} modules (dependencies included; they are \
         compiled but never emitted from)",
        outcomes.len(),
        emit_compile_modules_reached(outcomes)
    ));
    (lines, retention_error)
}

/// TWO CONCURRENT RUNS IN ONE WORKSPACE CORRUPT EACH OTHER'S EVIDENCE, SO THE SECOND REFUSES.
///
/// The arms deliberately share one probe root and one cargo target directory — that is what makes
/// the baseline warm and the restore comparable. It also means two runs interleave: one run's
/// faulted tree is the other run's baseline, and one run's restore erases the other's red before
/// it is read. Neither process observes anything wrong; both report confidently.
///
/// MEASURED, NOT ANTICIPATED. Verifying the blunted-mutation arm, a stale background invocation
/// overlapped a foreground one. The phase reported `Discriminated` with a red line quoting a
/// `#[cfg]` WARNING, over a cargo run whose own tail said `Finished` — a green compile reported as
/// a discriminating red. The arm that exists to catch a non-discriminating verdict was itself
/// given a fabricated one. A clean re-run answered `NotDiscriminating` correctly.
///
/// The refusal is a lock file created exclusively, NOT a wait and NOT a private directory per run.
/// Waiting would serialize into the same shared state with the same ambiguity about whose
/// artifacts are whose; a private directory would buy isolation by throwing away the warm target
/// dir the phase is built around. Refusing is the fail-closed arm: the line stops, the cause is
/// typed and located, and the operator sees that two runs were attempted rather than receiving a
/// verdict computed across both.
fn acquire_probe_root_lock(root: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(root)
        .map_err(|e| format!("could not create the probe root {}: {e}", root.display()))?;
    let lock = root.join("emit-compile.lock");
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock)
    {
        Ok(_) => Ok(lock),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => Err(format!(
            "another emitted-closure compile run holds {} — two runs sharing one probe root \
             interleave their faulted and restored trees, so neither verdict is attributable. \
             Remove the lock only after establishing no other run is live.",
            lock.display()
        )),
        Err(e) => Err(format!("could not take {}: {e}", lock.display())),
    }
}

pub fn run_required_emit_compile(
    source_roots: &[String],
) -> Result<Vec<EmitCompileOutcome>, String> {
    let entries = required_emit_compile_entries();
    if entries.is_empty() {
        return Err(
            "gunbc.ci_layer_roots required_emit_compile_entries is empty — the phase has no subject"
                .to_string(),
        );
    }
    // See `acquire_probe_root_lock`: a second concurrent run refuses rather than interleaving.
    let lock = acquire_probe_root_lock(&probe_root())?;
    // A FAILED RESTORE ENDS THE RUN, and it is not merely reported per entry.
    //
    // WHY IT IS TERMINAL RATHER THAN A FINDING SIBLINGS CONTINUE PAST, which is the shape every
    // other refusal here takes: the arms share ONE cargo target directory, so after a restore
    // fails the tree that directory holds is unknown -- it may carry artifacts of a faulted
    // crate. Every later entry's baseline is then unattributable, and reporting those baselines
    // would be the execution-provenance loss this module exists to refuse, one level out.
    //
    // AND IT MUST NOT BE PAPERABLE OVER BY A RE-RUN. A re-run re-emits from scratch, so a
    // transient restore failure simply vanishes and the phase greens -- at which point the
    // byte-exact restore has stopped being evidence and has become a flaky step people re-run.
    // Ending the run means the head that produced it has NO green from this phase at all, rather
    // than a green whose restore arm was never established.
    let mut outcomes = Vec::new();
    for entry in &entries {
        let outcome = run_emit_compile_entry(source_roots, entry);
        let terminal = matches!(
            &outcome,
            EmitCompileOutcome::Measured {
                mutation: MutationVerdict::RestoreFailed { .. },
                ..
            }
        );
        outcomes.push(outcome);
        if terminal {
            let ended_at = entry.clone();
            for remaining in entries.iter().skip(outcomes.len()) {
                outcomes.push(EmitCompileOutcome::NotExecuted {
                    entry: remaining.clone(),
                    cause: format!(
                        "the run ended at {ended_at}: a failed restore is terminal, because the \
                         shared cargo target directory is in an unknown state and no later \
                         baseline taken through it would be attributable"
                    ),
                });
            }
            break;
        }
    }
    // The lock is released here, at the one exit this function has below the acquisition: every
    // branch of the loop pushes an outcome and falls through. A run killed before this point
    // leaves the lock behind deliberately -- a probe root whose last writer died is not a state a
    // later run should silently build on, and the stale lock is what says so.
    let _ = std::fs::remove_file(&lock);
    Ok(outcomes)
}

/// THESE ARE LOCAL-ONLY EVIDENCE AND ARE LABELLED AS SUCH. The Rust suite was removed from CI
/// on 2026-07-11 (DESIGN, Building & checks), so nothing here executes on the merge path and
/// none of it may be cited as coverage. THE EXECUTED EVIDENCE FOR THIS PHASE IS THE PHASE
/// ITSELF: `establish_discriminating_red` runs on every required run, and a mutation that fails
/// to go red stops the line. What these add is the discrimination the in-run arm cannot perform
/// on itself -- that a non-`Discriminated` mutation is a FAILURE rather than a note, and that
/// the fault prefers a closure member over the entry.
#[cfg(test)]
mod tests {
    use super::*;

    /// The manifest is DERIVED, so this asserts the derivation reached the modeled rows rather
    /// than asserting a golden string: a package header from the version authority, the seed's
    /// runtime dependency set, and the path dependency the emitted closure links against.
    #[test]
    fn manifest_carries_the_modeled_dependency_rows() {
        let manifest = probe_manifest(Path::new("/repo"), "dag/std/logic.dag");
        assert!(manifest.starts_with("[package]\nname = \"gunbc-emitted-closure\""));
        assert!(manifest.contains("edition = \"2021\""));
        for name in ["im", "serde", "serde_json", "stacker"] {
            assert!(manifest.contains(name), "missing dependency row {name}");
        }
        assert!(manifest.contains("/repo/src/v1/stage0"));
        // `src/lib.rs` is cargo's own default, so restating it would be a second spelling.
        assert!(!manifest.contains("[lib]"));
    }

    /// A closure member is preferred over the entry, because it is the stronger subject.
    #[test]
    fn mutation_prefers_a_closure_member_over_the_entry() {
        let dir = std::env::temp_dir().join(format!("emit_compile_subject_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).expect("src");
        std::fs::write(
            dir.join("src/lib.rs"),
            "pub mod std_logic;\npub mod v2_std_node;\n",
        )
        .expect("lib");
        std::fs::write(dir.join("src/std_logic.rs"), "// member\n").expect("member");
        std::fs::write(dir.join("src/v2_std_node.rs"), "// entry\n").expect("entry");
        let subject = mutation_subject(&dir, "v2_std_node").expect("a subject");
        assert!(matches!(subject, MutationSubject::ClosureMember { .. }));
        assert_eq!(mutation_subject_rust_module(&subject), "std_logic");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A closure whose only member is the entry falls back to the entry AND SAYS SO, so the
    /// weaker measurement is legible as the weaker one.
    #[test]
    fn mutation_falls_back_to_the_entry_and_names_it() {
        let dir = std::env::temp_dir().join(format!("emit_compile_solo_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).expect("src");
        std::fs::write(dir.join("src/lib.rs"), "pub mod v2_std_node;\n").expect("lib");
        std::fs::write(dir.join("src/v2_std_node.rs"), "// entry\n").expect("entry");
        let subject = mutation_subject(&dir, "v2_std_node").expect("a subject");
        assert!(matches!(subject, MutationSubject::EntryModule { .. }));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// AN ENTRY THE RUN NEVER REACHED IS NOT A PASS. The terminal-restore arm exists precisely
    /// so that "not reached" has a spelling of its own; if it could pass, ending the run early
    /// would silently green every entry after the failure.
    #[test]
    fn an_unreached_entry_is_not_a_pass() {
        assert!(!emit_compile_outcome_passed(
            &EmitCompileOutcome::NotExecuted {
                entry: "e.dag".to_string(),
                cause: "the run ended earlier".to_string(),
            }
        ));
    }

    /// EVERY NON-`Discriminated` MUTATION ARM FAILS THE PHASE. Stated as a test because the
    /// tempting weakening -- treating the mutation as advisory beside a green baseline -- is
    /// exactly what would turn this phase into the decoration it exists not to be.
    #[test]
    fn a_green_baseline_does_not_pass_without_the_discrimination() {
        let green = CargoVerdict::Completed {
            status: 0,
            stderr_tail: String::new(),
            probe_line: None,
        };
        for mutation in [
            MutationVerdict::NotAttempted {
                reason: "r".to_string(),
            },
            MutationVerdict::NotDiscriminating {
                detail: "d".to_string(),
            },
            MutationVerdict::RestoreFailed {
                detail: "d".to_string(),
            },
        ] {
            let outcome = EmitCompileOutcome::Measured {
                entry: "e.dag".to_string(),
                crate_dir: "/tmp/x".to_string(),
                emitted_files: 1,
                baseline: green.clone(),
                mutation,
            };
            assert!(
                !emit_compile_outcome_passed(&outcome),
                "{}",
                emit_compile_outcome_summary(&outcome)
            );
        }
        let discriminated = EmitCompileOutcome::Measured {
            entry: "e.dag".to_string(),
            crate_dir: "/tmp/x".to_string(),
            emitted_files: 1,
            baseline: green,
            mutation: MutationVerdict::Discriminated {
                subject: MutationSubject::ClosureMember {
                    rust_module: "std_logic".to_string(),
                },
                red_line: "error[E0308]".to_string(),
            },
        };
        assert!(emit_compile_outcome_passed(&discriminated));
    }
}
