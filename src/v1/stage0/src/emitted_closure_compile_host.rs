//! Host realization for the required `emit-compile` phase: emit one entry's closure,
//! write it as a crate, and run cargo over it.
//!
//! WHY THIS MODULE EXISTS. DESIGN's Building-&-checks section carries a declared rung drop headed
//! "A BLOCKING EMIT-STAGE DIAGNOSTIC CAN SIT ON MAIN INDEFINITELY WITH NO REQUIRED PHASE THAT
//! FAILS", whose restoration trigger names a required phase that emits over a closure and
//! compiles it. The `v2-emission` phase emits and stops — its header says it does not catch "a
//! rustc error in the emitted tree (nothing here compiles the emission)". This module is that
//! missing conjunct, deliberately the SAME PRODUCER (`compile_entry_emission`), so a green there
//! and here cannot be two facts about two emissions.
//!
//! THE SUBJECT IS A CLOSURE, NOT A FILE. DESIGN's row measures its specimen as reachable from an
//! entry whose closure INCLUDES the offending call site, unreachable from the file HOLDING it,
//! and the holding module compiling clean in isolation. So the subject is the emitted closure of
//! a declared entry, compiled whole.
//!
//! THE DISCRIMINATION IS EXECUTED, NOT ASSERTED. A cargo phase that cannot go red is worse than
//! absent (DESIGN 4b): it gets cited as coverage. So every run establishes its own red BY
//! MUTATION: baseline green, then ONE injected fault in ONE emitted file which must fail ALONE,
//! then a byte-exact restore which must return green. A mutated tree that still compiles FAILS
//! THE PHASE — the cargo verdict is insensitive to its bytes and the green above carries no
//! information. The restore is evidence too: without it a red could be emission residue. A
//! FAILED RESTORE IS TERMINAL FOR THE RUN and not recoverable by re-running — see
//! `run_required_emit_compile`, which stops there and reports every later entry as `NotExecuted`
//! rather than measuring through a target directory of unknown state.
//!
//! TWO ROUTES REACH RUSTC FROM HERE, AND THE SECOND IS WHY THE HEADER ABOVE IS NOT THE WHOLE
//! FILE. The required phase's subject is a rostered `.dag` ENTRY FILE, so nothing a fixture can
//! author is a subject for it. `fixture_closure_rustc_verdict` is the second route: the same
//! in-memory `.dag` source shape a witness already hands `compile_dag_rust_emit_check`, emitted
//! through the same `compile_sources`, written by the same crate writer, handed to the same cargo
//! invocation. It exists because a substring oracle over emitted TEXT cannot see a meaning-level
//! emitter defect, and rustc can. The two routes share `write_probe_crate_files`, `probe_manifest`
//! and `run_cargo` deliberately: a separately authored fixture harness would make a green here and
//! a green there two facts about two crates.
//!
//! THE FIXTURE ROUTE IS `#[cfg(test)]` AND HAS EXACTLY ONE CONSUMER: the generated
//! `fixture_closure_rustc_discrimination` (authored in `v1.compiler.compiler_tests_rust`), which
//! is `#[ignore]`d and therefore available on demand rather than executing on push. It reaches no
//! CLI flag and no required phase, and it is not on the emitted seed's public surface.
//!
//! WHAT THIS PHASE IS NOT. No baseline, no diagnostic count, no ratchet. Cargo's exit status is
//! the whole verdict; warnings are not errors here. Pinning a diagnostic population measured on
//! the current tree would be the tree-copied oracle DESIGN 5 rejects; an identity-grain debt
//! contract over the emitted population is a separate construction with a separate argument.

use std::path::{Path, PathBuf};

use super::{
    ci_layer_roots_authority_content, compile_entry_emission, process_workspace_root,
    string_list_data_from_ci_layer_roots_source, CompileDisposition, CompileRun,
};
use crate::extdeps_cargo::{CargoDepSource, CargoDependency};
use crate::extdeps_cargo_version::render_cargo_package_header_prefix;
use crate::gunbc_stage0_crate_partition_generated::GeneratedPartitionCrateKind;
use crate::v1_compiler_stage0_crates::{
    render_stage0_crate_dep, render_stage0_crate_features_section, stage0_features_for_crate_kind,
    stage0_foundation_runtime_dependencies,
};

const REQUIRED_EMIT_COMPILE_ENTRIES_DATA_NAME: &str = "required_emit_compile_entries";
const PROBE_ROOT_DIR_NAME: &str = "gunbc-emit-compile";

/// The crate name the emitted closure is compiled under — DERIVED PER ENTRY, not shared.
///
/// WHY IT IS NOT ONE NAME FOR EVERY ENTRY, WHICH IS WHAT IT WAS. Entries share one
/// `CARGO_TARGET_DIR` deliberately — rebuilding `im`, `serde` and the seed crate per entry would
/// multiply cost by roster size. But a shared target dir plus one package name and version leaves
/// only cargo's use of the manifest path separating two entries' fingerprints: a tool
/// implementation detail, load-bearing for a merge gate, stated nowhere.
///
/// THE FAILURE IT WOULD PRODUCE IS FAIL-OPEN, WHICH IS WHY IT IS WORTH A NAME RATHER THAN A
/// COMMENT SAYING CARGO HANDLES IT. A build judged fresh against another entry's artifacts
/// replays its cached diagnostics, and a replayed clean compile is byte-identical to a real one:
/// `Completed status=0` for an entry never compiled, and the gate greens over it.
///
/// Deriving the name per entry makes each probe crate its own package, so fingerprints cannot
/// alias; dependencies are separate packages and stay warm.
fn probe_package_name(entry: &str) -> String {
    let slug: String = entry
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    format!("gunbc-emitted-closure-{slug}")
}

/// THE INJECTED FAULT. A type error, not a syntax error: a syntax error is caught by anything
/// that merely parses, so cannot discriminate a cargo verdict from a cheaper reader. `E0308`
/// requires rustc to have type-checked the module — the reach being claimed. The name cannot
/// collide with emitted output, and the item is `pub` so no dead-code lint elides it.
/// The symbol the injected item declares. The faulted arm's diagnostics must NAME it — that is
/// what attributes the red to this phase's fault rather than anything else wrong in the tree.
const MUTATION_PROBE_SYMBOL: &str = "EMIT_COMPILE_MUTATION_PROBE";

const MUTATION_ITEM: &str =
    "\npub const EMIT_COMPILE_MUTATION_PROBE: u8 = \"the phase's own discriminating red\";\n";

/// The `.dag` entry paths whose emitted closure a required phase compiles, read live from
/// `gunbc.ci_layer_roots` `required_emit_compile_entries`.
///
/// A `List<String>` as the emission roster is: the axis is WHICH ENTRIES, so a second entry is a
/// row, never a second host reader.
pub fn required_emit_compile_entries() -> Vec<String> {
    string_list_data_from_ci_layer_roots_source(
        ci_layer_roots_authority_content(),
        REQUIRED_EMIT_COMPILE_ENTRIES_DATA_NAME,
    )
}

/// What cargo did, at the grain the phase can act on.
///
/// The three arms `PartitionCompileOutcome` already separates, for the reason recorded there: a
/// process killed without an exit status reports an empty diagnostic population, which reads as
/// a clean build unless the disposition says otherwise.
#[derive(Debug, Clone)]
pub enum CargoVerdict {
    /// The toolchain was never invoked.
    NotAttempted { reason: String },
    /// Launched, and reached no exit status of its own -- killed, or the spawn failed.
    DidNotComplete { detail: String },
    /// Ran to completion and reported its own exit status.
    ///
    /// `probe_line` is the first ERROR-diagnostic line naming the CALLER'S ATTRIBUTION SYMBOL, scanned
    /// from the WHOLE stderr, not `stderr_tail`: the tail is the last 20 lines kept for a human
    /// reader, and a genuine diagnostic for the attributed item can sit above it when other
    /// diagnostics follow. Attributing from the tail would fail a run whose fault WAS refused
    /// because the receipt was short.
    ///
    /// THE SYMBOL IS THE CALLER'S BECAUSE THE TWO ROUTES ATTRIBUTE DIFFERENT THINGS AND MUST NOT
    /// SHARE ONE ANSWER: the required phase attributes its own injected fault
    /// (`MUTATION_PROBE_SYMBOL`), while the fixture route attributes a red to the FIXTURE'S OWN
    /// emitted module. A red naming neither is a red about something else in the closure, and
    /// both callers need to be able to say so.
    /// `probe_diagnostic` is the rustc DIAGNOSTIC HEADER governing `probe_line` -- the
    /// `error[E0308]: mismatched types` line the attributed location sits under. A location line
    /// alone says only WHERE a fault was reported and never WHAT rustc refused, so a caller
    /// holding only `probe_line` passes on ANY refusal reported in the attributed file. The
    /// header is what lets a caller name the error class it expects.
    Completed {
        status: i32,
        stderr_tail: String,
        probe_line: Option<String>,
        probe_diagnostic: Option<String>,
    },
}

/// Only a completed, zero-status run compiled; every other arm, including never launched, is a
/// refusal.
///
/// FREE FUNCTIONS RATHER THAN `impl` METHODS, throughout this module: an `impl` method has no
/// `DeclarationRef` spelling (`std.decl_ref` offers `WholeDeclaration` or `NamedField`), so every
/// method would grow the uncitable-item class `gunbc.seed_growth_admission` reports in
/// `seed_growth_uncitable_item_keys`. `v1_compiler.declaration_index` took the same route.
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

/// The diagnostic line naming the caller's attribution symbol, if the run produced one.
pub fn cargo_verdict_probe_line(verdict: &CargoVerdict) -> Option<&str> {
    match verdict {
        CargoVerdict::Completed { probe_line, .. } => probe_line.as_deref(),
        _ => None,
    }
}

/// The rustc diagnostic header governing the attributed line -- WHAT was refused, beside
/// `cargo_verdict_probe_line`'s WHERE.
pub fn cargo_verdict_probe_diagnostic(verdict: &CargoVerdict) -> Option<&str> {
    match verdict {
        CargoVerdict::Completed {
            probe_diagnostic, ..
        } => probe_diagnostic.as_deref(),
        _ => None,
    }
}

pub fn cargo_verdict_stderr_tail(verdict: &CargoVerdict) -> &str {
    match verdict {
        CargoVerdict::Completed { stderr_tail, .. } => stderr_tail.as_str(),
        _ => "",
    }
}

/// WHICH FILE THE FAULT WENT INTO -- AND IT IS THE ENTRY'S OWN EMITTED MODULE, BY CONSTRUCTION.
///
/// THIS TYPE USED TO OFFER A CHOICE, AND THE CHOICE WAS THE DEFECT. The selector took the first
/// declared non-entry module — a SHARED-CORE member in every closure that has one. Measured over
/// the roster at landing, 8 of 8 entries mutated a shared member: 7 x `std_error_primitives`, 1 x
/// `v1_rt`, the emitted runtime in every closure. Every arm was honestly `Discriminated`; the
/// verdict established `cargo refused when the shared core was broken`, never `THIS entry's own
/// closure was compiled` — total at the level examined, blind one level down.
///
/// WHY THE ENTRY'S OWN MODULE IS THE ONLY SUBJECT WORTH THE FAULT. It is the one member NOTHING
/// ELSE REFERENCES (a dependency does not import the root), so it is exactly the file a faulty
/// emission could DROP while the crate still compiled. A drop that breaks a reference is caught
/// by the baseline; the uncaught case is a REFERENCE-CLOSED drop, and the entry's own leaf is the
/// canonical one. A shared member is reached whether or not the entry's bytes are in the tree.
///
/// SO THE CARRIER HOLDS ONE THING AND HAS NO SPELLING FOR THE OTHER. `mutation_subject` is the
/// only constructor and derives the module from the ENTRY, so `a shared member carried the fault`
/// is unwritable, not merely unselected — construction over validation, no fallback arm
/// (DESIGN 5). Absence is a typed refusal naming the entry (`MutationSubjectRefusal`).
///
/// THE PRIVACY BOUNDARY IS NAMED, BECAUSE RUST'S IS MODULE-SCOPED AND NOT FUNCTION-SCOPED. A
/// private field beside its constructor walls OTHER modules and is convention within this one —
/// the sole-constructor finding this repository records: such a wall governs WHO constructs and
/// says nothing inside the declaring module. So carrier and constructor live in the submodule
/// below and everything else is OUTSIDE it: `MutationSubject { rust_module: ... }` elsewhere in
/// this file does not compile. Structurally impossible rather than review diligence, in one `mod`
/// block.
pub use entry_own_subject::{mutation_subject, subject_rust_module, MutationSubject};

mod entry_own_subject {
    use super::MutationSubjectRefusal;
    use std::path::Path;

    #[derive(Debug, Clone)]
    pub struct MutationSubject {
        /// PRIVATE, made meaningful by the wrapping module: the only route to a value from
        /// anywhere in the file is `mutation_subject`, which derives the name from the ENTRY.
        rust_module: String,
    }

    /// A FREE FUNCTION AND NOT AN `impl` METHOD, for a receipt rather than taste. `std.decl_ref`
    /// offers `WholeDeclaration` or `NamedField`, neither naming an impl method, so a method is
    /// UNCITABLE in this file's seed-growth roster and the census silently short.
    /// `gunbc.emitted_closure_compile_seed_growth` states this file adds ZERO uncitable items; an
    /// `impl` would falsify that while the census still reported clean. Inside the privacy
    /// boundary it reads the private field as the constructor does.
    pub fn subject_rust_module(subject: &MutationSubject) -> &str {
        subject.rust_module.as_str()
    }

    /// THE FAULT GOES INTO THE ENTRY'S OWN EMITTED MODULE, OR NOWHERE.
    ///
    /// No member-preferring arm and no fallback: the module name is DERIVED from the entry, so
    /// only presence is decided, and absence is a typed refusal naming the entry. Substitution is
    /// what this function used to do, making every roster verdict a statement about the shared
    /// core (see `MutationSubject`).
    pub fn mutation_subject(
        crate_dir: &Path,
        entry_module: &str,
    ) -> Result<MutationSubject, MutationSubjectRefusal> {
        let lib_rs = crate_dir.join("src/lib.rs");
        let declared = match super::closure_modules(&lib_rs) {
            Ok(modules) => modules,
            Err(detail) => {
                return Err(MutationSubjectRefusal::ClosureManifestUnreadable {
                    lib_rs: lib_rs.display().to_string(),
                    detail,
                })
            }
        };
        // DECLARED AND WRITTEN ARE TWO FACTS AND BOTH ARE REQUIRED. A `pub mod` with no file does
        // not compile; a file no `pub mod` reaches is not in the closure. Checking one would admit
        // a subject cargo did not compile, and the fault would prove nothing.
        if !declared.iter().any(|m| m == entry_module) {
            return Err(MutationSubjectRefusal::EntryModuleNotDeclared {
                entry_module: entry_module.to_string(),
                declared,
            });
        }
        let path = crate_dir.join(format!("src/{entry_module}.rs"));
        if !path.is_file() {
            return Err(MutationSubjectRefusal::EntryModuleFileMissing {
                entry_module: entry_module.to_string(),
                path: path.display().to_string(),
            });
        }
        Ok(MutationSubject {
            rust_module: entry_module.to_string(),
        })
    }
}

pub fn mutation_subject_rust_module(subject: &MutationSubject) -> &str {
    entry_own_subject::subject_rust_module(subject)
}

/// The SUBJECT KIND, printed rather than inferred. One kind, still logged, so a receipt reader
/// need not know this file — and a second kind would find every receipt already carrying the
/// field.
pub fn mutation_subject_name(_subject: &MutationSubject) -> &'static str {
    "EntryOwnModule"
}

/// WHY THE ENTRY'S OWN MODULE WAS NOT AVAILABLE TO CARRY THE FAULT.
///
/// Three causes with three owners, kept apart: an unreadable manifest is a probe-crate defect, a
/// module the manifest never declares is an EMISSION defect (the closure lost its own root), a
/// declared module with no file is a WRITE defect. Collapsing them is the state-space conflation
/// DESIGN names.
///
/// MEASURED AS REACHABLE-BUT-EMPTY, WHICH IS A HEALTHY QUIET GUARD AND NOT A DECORATION: all 8
/// rostered entries emit their own module as its own `.rs`, so no arm fires today. The mechanism
/// exists — a dropped or renamed root is what this phase is for — and a fixture authors every arm
/// directly, so its RED is authorable (DESIGN 4b).
#[derive(Debug, Clone)]
pub enum MutationSubjectRefusal {
    ClosureManifestUnreadable {
        lib_rs: String,
        detail: String,
    },
    EntryModuleNotDeclared {
        entry_module: String,
        declared: Vec<String>,
    },
    EntryModuleFileMissing {
        entry_module: String,
        path: String,
    },
}

pub fn mutation_subject_refusal_summary(refusal: &MutationSubjectRefusal) -> String {
    match refusal {
        MutationSubjectRefusal::ClosureManifestUnreadable { lib_rs, detail } => format!(
            "EntryModuleAbsent/ClosureManifestUnreadable lib_rs={lib_rs} detail={detail} — the \
             emitted crate's own module list could not be read, so the entry's own module can \
             neither be found nor ruled out; nothing else may carry the fault in its place"
        ),
        MutationSubjectRefusal::EntryModuleNotDeclared {
            entry_module,
            declared,
        } => format!(
            "EntryModuleAbsent/EntryModuleNotDeclared entry_module={entry_module} \
             declared=[{}] — the emitted closure does not declare the entry's OWN module. That \
             is the reference-closed drop this phase exists to catch: the crate compiles because \
             every remaining member is a dependency of the missing root. Mutating a member \
             instead would report Discriminated over exactly the tree that is broken",
            declared.join(",")
        ),
        MutationSubjectRefusal::EntryModuleFileMissing { entry_module, path } => format!(
            "EntryModuleAbsent/EntryModuleFileMissing entry_module={entry_module} path={path} — \
             the closure declares the entry's own module and no file was written for it"
        ),
    }
}

/// WHETHER THIS RUN'S CARGO VERDICT IS SENSITIVE TO THE BYTES IT WAS HANDED.
///
/// Only `Discriminated` is a pass. Every other arm says the baseline green carries no
/// information — a phase failure, not a note: a decoration must not report coverage.
#[derive(Debug, Clone)]
pub enum MutationVerdict {
    /// No fault was injected. Carries why: a red baseline has nothing to discriminate against; a
    /// tree with no writable module has nowhere to put the fault.
    NotAttempted { reason: String },
    /// The fault went in and cargo still compiled the tree. THE INSTRUMENT IS NOT MEASURING
    /// WHAT IT CLAIMS TO.
    NotDiscriminating { detail: String },
    /// THE ENTRY'S OWN EMITTED MODULE WAS NOT THERE TO CARRY THE FAULT, and nothing stood in for
    /// it. Its own arm, not a `NotAttempted` reason string: a POSITIVE FINDING ABOUT THE EMISSION
    /// with its own owner and repair — the case a silent fallback to a shared member would have
    /// reported as `Discriminated`.
    SubjectRefused { refusal: MutationSubjectRefusal },
    /// The fault produced a red and the restore did not return the starting state -- bytes
    /// differ, or the restored tree does not compile. The red is unattributable: possibly residue.
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
        MutationVerdict::SubjectRefused { refusal } => {
            format!(
                "SubjectRefused {}",
                mutation_subject_refusal_summary(refusal)
            )
        }
        MutationVerdict::Discriminated { subject, red_line } => format!(
            "Discriminated subject={} module={} red={red_line}",
            mutation_subject_name(subject),
            mutation_subject_rust_module(subject)
        ),
    }
}

/// One entry's whole story, and every verdict is reached THROUGH the stage that produced it.
///
/// `EmissionRefused` and `CrateNotWritten` carry no cargo verdict, so "cargo found nothing
/// wrong" and "cargo was never reached" cannot share a spelling -- the execution-provenance loss
/// DESIGN names, which a `passed: bool` beside an optional cause would reintroduce.
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
    /// Never reached: an earlier entry's restore failed, which is TERMINAL for the run. Carries
    /// the entry that ended it, so "not reached" is never read as "reached and clean".
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

/// A pass is a completed emission, a green baseline AND an executed discrimination. Without the
/// third, the first two are satisfied by an instrument that cannot fail.
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
/// Package header from `extdeps.rust.version` `render_cargo_package_header_prefix`; dependency
/// rows from `v1.compiler.stage0_crates` `stage0_foundation_runtime_dependencies` (the seed's
/// runtime dependency set, which emitted code links against), each rendered by that module's
/// `render_stage0_crate_dep`. No `[lib]` section: `src/lib.rs` is cargo's own default, so naming
/// it would be a second spelling.
///
/// The corpus's hand-authored TOML string (`tools.self_host_curated_seed_linked_harness`
/// `cssl_v1_compiled_probe_lib_cargo_toml`) is deliberately not used: it is marked scaffold debt
/// in its own module as concat-authored markup, and a required gate consuming it would pin that
/// debt open on the merge path.
fn probe_manifest(workspace: &Path, entry: &str) -> String {
    let mut deps: Vec<CargoDependency> = stage0_foundation_runtime_dependencies()
        .iter()
        .map(|dep| (**dep).clone())
        .collect();
    // The emitted closure links against the seed crate for the runtime surface it does not emit
    // (`v1_rt` and friends). An absolute path dependency: the probe crate is written outside the
    // repository.
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
    // The emitted `v1_rt.rs` gates on `#[cfg(feature = "text_lookup_work_counter")]`. Referencing
    // an undeclared feature earns `unexpected_cfgs` — a WARNING locally, a hard ERROR under CI's
    // `RUSTFLAGS=-D warnings` — so the probe crate compiled clean on a workstation and failed
    // `status=101` on every entry in CI, the baseline reporting a red unrelated to the closure.
    //
    // Rendered from the partition crates' own authority, `stage0_features_for_crate_kind` (which
    // the partition rows reach through `stage0_partition_row_features`), not authored here: the
    // corpus already carries two hand-concatenated `[features]` blocks for this reason, each with
    // a note, and a third string would be the second representation those notes argue against.
    // THE KIND IS THE WHOLE SUBJECT, so the kind is what is passed. An earlier revision handed a
    // fabricated `GeneratedPartitionCrateRow` -- blank crate_dir, empty module lists -- to a
    // function reading only `row.kind`. That row ASSERTED this probe is a generated partition
    // crate; it is a per-entry crate outside the repository sharing only the foundation kind's
    // feature set, because the emitted `v1_rt.rs` gates on it.
    let features = render_stage0_crate_features_section(stage0_features_for_crate_kind(
        GeneratedPartitionCrateKind::GeneratedFoundationCrate,
    ));
    format!(
        "{}\nedition = \"2021\"\n{features}\n[dependencies]\n{rendered}",
        render_cargo_package_header_prefix(probe_package_name(entry))
    )
}

/// Where one entry's probe crate is written. Outside the repository: a crate under the workspace
/// root is inferred into the workspace and would need its own `[workspace]` to escape — a manifest
/// fact invented to work around its location.
///
/// RUNNER-SCOPED, NOT HOST-SHARED, AND THIS WAS MEASURED THE HARD WAY. A fixed path in the host's
/// `/tmp` is shared by every tenant of a SELF-HOSTED runner and persists across runs, slots and
/// jobs. On the first required run the directory existed owned by another uid, so creating the
/// lock returned `EACCES` and the phase refused — permanently, on every PR landing on that
/// runner, the only closing move being someone deleting a directory over SSH. A required gate
/// whose sole remedy is manual host intervention has no reachable green: the shape DESIGN records
/// for a gate that launders rather than gates.
///
/// `RUNNER_TEMP` is created and torn down per job and owned by the process needing it, so two
/// tenants never name one path. Its ABSENCE IS A REFUSAL, not permission to write the host-shared
/// system temp and hope this runner makes it safe. The shared target dir is untouched: one run's
/// entries still share `workspace/target`, with per-entry package names separating fingerprints.
///
/// This refusal governs `required_ci_probe_root_from_runner_temp` and its environment-reading
/// wrapper `required_ci_emit_compile_probe_root`; `local_emit_compile_probe_root` deliberately
/// remains the standalone mode's explicit system-temp selection.
fn required_ci_probe_root_from_runner_temp(
    runner_temp: Option<&std::ffi::OsStr>,
) -> Result<PathBuf, String> {
    let base = runner_temp
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            "RUNNER_TEMP is unset or empty; the required emit-compile phase refuses rather than \
             falling back to a host-shared temp directory"
                .to_string()
        })?;
    Ok(PathBuf::from(base).join(PROBE_ROOT_DIR_NAME))
}

pub fn required_ci_emit_compile_probe_root() -> Result<PathBuf, String> {
    required_ci_probe_root_from_runner_temp(std::env::var_os("RUNNER_TEMP").as_deref())
}

pub fn local_emit_compile_probe_root() -> PathBuf {
    std::env::temp_dir().join(PROBE_ROOT_DIR_NAME)
}

fn probe_crate_dir(probe_root: &Path, entry: &str) -> PathBuf {
    let slug: String = entry
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    probe_root.join(slug)
}

/// Write the emitted Rust files plus a manifest, and return the crate directory.
fn write_probe_crate(
    run: &CompileRun,
    probe_root: &Path,
    entry: &str,
) -> Result<(PathBuf, usize), String> {
    let emission = run
        .emissions
        .iter()
        .find(|emission| emission.target_name == "rust")
        .ok_or_else(|| "the emission carries no rust target".to_string())?;
    write_probe_crate_files(&emission.result.files, probe_root, entry)
}

/// The crate writer both routes share: emitted files in, a written crate directory out.
///
/// ONE WRITER, TWO CALLERS, AND THE SHARING IS THE POINT (DESIGN §2). The required phase reaches
/// it through a `CompileRun`'s rust emission; the fixture route reaches it with the files a
/// virtual source's `compile_sources` produced. A second writer beside this one would let the two
/// routes disagree about what "the emitted crate" is -- manifest, stale-tree removal, or the
/// `src/lib.rs` requirement -- so a green on one would stop being evidence about the other.
fn write_probe_crate_files(
    files: &im::Vector<std::rc::Rc<crate::v1_std_core::TextFile>>,
    probe_root: &Path,
    entry: &str,
) -> Result<(PathBuf, usize), String> {
    let dir = probe_crate_dir(probe_root, entry);
    // A STALE TREE IS NOT A SUBJECT. A previous run's bytes under the same slug would let a module
    // deleted from the closure keep compiling, so the directory is removed, not written over.
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src"))
        .map_err(|e| format!("creating {}: {e}", dir.display()))?;
    let mut written = 0usize;
    for file in files.iter() {
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
//// THE ATTRIBUTED LOCATION AND THE DIAGNOSTIC IT SITS UNDER, READ FROM ONE PASS OVER STDERR.
///
/// WHY BOTH, AND WHY THE SECOND IS NOT A NICETY. A caller holding only the location line knows a
/// fault was reported IN a file and nothing about WHAT was reported: a syntax error, an
/// unresolved name, a wrong arity and the type mismatch a caller is actually adjudicating are one
/// answer to it. That is the difference between "rustc refused something here" and "rustc refused
/// THIS CLASS here", and only the second can carry a claim about an emitter behaviour.
///
/// THE GOVERNING HEADER IS THE MOST RECENT ONE ABOVE THE LOCATION, which is rustc's own layout:
/// `error[E0308]: mismatched types` followed by its ` --> file:line:col`. Scanning forward and
/// keeping the last header seen therefore attributes the location to its OWN diagnostic and not
/// to a neighbouring one -- the property the unit test beside this function pins, because both
/// directions typecheck and only one is right.
///
/// ONLY AN `error` DIAGNOSTIC ATTRIBUTES, AND A `warning` CLEARS THE HEADER RATHER THAN
/// STANDING AS ONE. Both callers are asking WHY A RUN WAS REFUSED, and a warning naming the file
/// is not a refusal: attributing a red to a warning's location would name a line that did not
/// stop the build, and letting an intervening warning keep an earlier error's header would
/// attribute a warning's span to an error it does not belong to. A file named only by warnings is
/// therefore unattributed -- `None`, never the nearest error elsewhere in the run, because an
/// answer invented from a diagnostic that does not govern the line is a fabricated attribution,
/// which is worse for a caller than admitting that stderr did not say.
///
/// PURE, AND SEPARATE FROM `run_cargo`, so the parse this route's verdict depends on is testable
/// without spawning a toolchain: the scan executes on every push through the unit test, while the
/// cargo route around it does not.
fn attributed_diagnostic(
    stderr: &str,
    attribution_symbol: &str,
) -> (Option<String>, Option<String>) {
    let mut header: Option<&str> = None;
    for line in stderr.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("error") {
            header = Some(trimmed);
        } else if trimmed.starts_with("warning") {
            header = None;
        }
        if let Some(error_header) = header {
            if line.contains(attribution_symbol) {
                return (Some(trimmed.to_string()), Some(error_header.to_string()));
            }
        }
    }
    (None, None)
}

/// `build --release` INTO THE WORKSPACE TARGET DIRECTORY, both halves one cost decision: the
/// lane's first step is `cargo build --release -p v1-compiler --bins`, so the seed crate is
/// already compiled there under that profile; a `check` or a private target dir would share no
/// fingerprint and rebuild the whole dependency graph inside a required phase. The baseline arm
/// thus compiles only the emitted crate, and the two further arms are incremental.
///
/// Phases within one required run are sequential in one process, so nothing else holds cargo's
/// lock on that directory.
fn run_cargo(crate_dir: &Path, workspace: &Path, attribution_symbol: &str) -> CargoVerdict {
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
                let (probe_line, probe_diagnostic) =
                    attributed_diagnostic(&stderr, attribution_symbol);
                CargoVerdict::Completed {
                    status,
                    stderr_tail: tail.into_iter().rev().collect::<Vec<_>>().join("\n"),
                    probe_line,
                    probe_diagnostic,
                }
            }
        },
    }
}

/// The rust module basenames the emitted `lib.rs` declares, in its own order.
///
/// AN UNREADABLE MANIFEST IS RETURNED, NOT RENDERED AS AN EMPTY CLOSURE. An empty vector reads as
/// `this crate declares no modules`, so a caller would answer `entry module not declared` for a
/// crate it never read -- execution-provenance loss, misattributing a filesystem defect to the
/// emission.
pub(crate) fn closure_modules(lib_rs: &Path) -> Result<Vec<String>, String> {
    let content = std::fs::read_to_string(lib_rs).map_err(|e| e.to_string())?;
    Ok(content
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("pub mod ")
                .and_then(|rest| rest.strip_suffix(';'))
                .map(|m| m.trim().to_string())
        })
        .collect())
}

/// THE DISCRIMINATING RED, ESTABLISHED BY MUTATION AND RESTORED BEFORE THE PHASE REPORTS.
///
/// One fault, in one file, failing alone -- the baseline before is the control, the restore after
/// the second control. Several things changing at once would show cargo responds to damage, not
/// that this instrument reads this closure.
fn establish_discriminating_red(
    crate_dir: &Path,
    workspace: &Path,
    entry_module: &str,
) -> MutationVerdict {
    // NO FALLBACK ARM. A closure missing its own entry module is the finding -- substituting
    // another member would yield `Discriminated` over precisely the broken tree.
    let subject = match mutation_subject(crate_dir, entry_module) {
        Ok(subject) => subject,
        Err(refusal) => return MutationVerdict::SubjectRefused { refusal },
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

    let red = run_cargo(crate_dir, workspace, MUTATION_PROBE_SYMBOL);

    // THE RESTORE RUNS WHATEVER THE FAULTED ARM ANSWERED, or the next run's baseline goes red for
    // a reason unrelated to the corpus.
    let restore_write = std::fs::write(&path, &original);
    let restored_bytes = std::fs::read_to_string(&path).unwrap_or_default();

    // RESTORATION IS ADJUDICATED BEFORE ANY FAULT VERDICT, AND THE ORDER IS THE WHOLE POINT.
    //
    // Every fault-verdict arm below returns `NotDiscriminating`, failing this entry while SIBLINGS
    // CONTINUE; `RestoreFailed` alone ends the run. The two conditions are independent and
    // co-occur, so deciding the fault first would report the non-terminal verdict and swallow the
    // terminal one — the next entry's baseline then runs against a target directory of
    // unestablished state, the unattributable measurement the terminal rule prevents. Whichever
    // is checked first wins, and only one is safe to lose.
    //
    // The BYTE half is adjudicated here: a filesystem fact, in hand and free. Whether the RESTORED
    // TREE COMPILES stays below the fault verdicts: a third cargo invocation, and a question about
    // attributing THIS entry's red, moot once the arm has refused to claim one.
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

    // THE FAULTED ARM MUST HAVE COMPLETED, AND ITS RED MUST BE ATTRIBUTABLE TO THE FAULT.
    //
    // A NONZERO EXIT IS NOT EVIDENCE ON ITS OWN. Cargo can be killed, fail to spawn, run out of
    // disk, or die for reasons unrelated to the injected item — `!cargo_verdict_compiled(&red)`
    // is true in every case. Accepting them reports `Discriminated` while establishing nothing
    // about sensitivity, and greens a merge gate on a fabricated red.
    //
    // THIS IS NOT HYPOTHETICAL. Verifying the blunted-mutation arm, a concurrent run produced a
    // `Discriminated` verdict whose red line quoted a `#[cfg]` WARNING over a cargo run that said
    // `Finished`. The probe-root lock closes the cause; this closes the arm that accepted the
    // result — different defects.
    //
    // So the arm demands three things of the faulted run, in order of what they rule out:
    //   1. `Completed` — cargo reached a verdict, so `NotAttempted`/`DidNotComplete` fail rather
    //      than passing as a red;
    //   2. a nonzero status — it refused;
    //   3. a diagnostic naming THE INJECTED SYMBOL — it refused for OUR reason. The symbol rather
    //      than the code distinguishes the injected fault from an unrelated `E0308` already in the
    //      emitted tree.
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

    let restored = run_cargo(crate_dir, workspace, MUTATION_PROBE_SYMBOL);
    if !cargo_verdict_compiled(&restored) {
        return MutationVerdict::RestoreFailed {
            detail: format!(
                "the restored tree does not compile ({}) — the red above it cannot be \
                 attributed to the injected fault",
                cargo_verdict_summary(&restored)
            ),
        };
    }

    // The reported line is the diagnostic NAMING THE FAULT, the same line the attribution check
    // accepted -- the receipt is the evidence the arm decided on, not a separately chosen line.
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
pub fn run_emit_compile_entry(
    source_roots: &[String],
    probe_root: &Path,
    entry: &str,
) -> EmitCompileOutcome {
    // PROGRESS IS REPORTED AS THE STAGE IS ENTERED, NOT WHEN THE ENTRY FINISHES.
    //
    // Every stage below is a long host effect -- a whole-index emission, then up to three cargo
    // invocations -- and reporting only on completion renders a HANG, a KILL and a slow run
    // identically: execution-provenance loss in the instrument's own output. Measured when two
    // verification runs died inside this function with no line at all, so nothing distinguished
    // "still emitting" from "killed".
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

    let (crate_dir, emitted_files) = match write_probe_crate(&run, probe_root, entry) {
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
    let baseline = run_cargo(&crate_dir, &workspace, MUTATION_PROBE_SYMBOL);
    eprintln!(
        "emit-compile: {entry} baseline {} — mutation",
        cargo_verdict_summary(&baseline)
    );
    // THE DISCRIMINATION IS NOT ATTEMPTED OVER A RED BASELINE, and says so: an already-failing
    // tree goes red under the fault for a reason the fault did not cause -- a green control
    // wearing a red one's clothes.
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

/// WHAT A FIXTURE CAN ASK RUSTC, AND WHY NOTHING COULD ASK IT BEFORE.
///
/// `compile_dag_rust_emit_check` compiles an in-memory `.dag` source and lets a fixture assert on
/// the emitted TEXT — substring includes and excludes. That is a SPELLING oracle: it answers
/// "does this byte sequence occur", never "does the emitted program MEAN what the source said".
/// The required `emit-compile` phase does reach rustc, but only over the eight `.dag` ENTRY files
/// its roster names, so a fixture cannot pose a subject to it at all.
///
/// So the two capabilities sat on opposite sides of a gap: fixture-authorable subjects with no
/// compiler behind them, and a real compiler with no fixture-authorable subjects. This function
/// closes it — the SAME in-memory source shape `compile_dag_rust_emit_check` takes, emitted
/// through the same `compile_sources`, written by the same crate writer the required phase uses,
/// and handed to the same cargo invocation.
///
/// WHY THAT MATTERS FOR SAFETY RATHER THAN CONVENIENCE. A meaning-level emitter defect —— a host
/// representation emitted where a structural carrier was declared, a dropped type argument, a
/// reference emitted past its resolved binding —— is invisible to a substring oracle whenever the
/// wrong bytes happen to contain the right substring, and it is exactly what rustc's type checker
/// refuses. `gunbc.rung_drop` `text_boundary_identity_wall` already CLAIMS that rung ("on the
/// emitted-Rust self-host path the class is mechanically preventable — ... renders as a Rust type
/// mismatch and the mandatory emitted-crate cargo check refuses it") for a fixture the required
/// phase's roster cannot reach. DESIGN §4b(1) says a rung is established by evidence executing on
/// the acceptance path; this is the route that lets such a claim execute over its own fixture
/// instead of being asserted about one.
///
/// GUNBC'S OWN REFUSAL IS A DISTINCT ARM AND NEVER A RUSTC VERDICT. If the fixture does not reach
/// emission compile-clean, the answer is `SourceRefused` carrying the count and the first hard
/// diagnostic — not a red. Collapsing the two would let a fixture that gunbc rejected be reported
/// as "rustc refused the emitted bytes", which is a claim about bytes that were never emitted.
#[derive(Debug, Clone)]
#[cfg(test)]
pub(crate) enum FixtureClosureOutcome {
    /// The v1 compiler refused the fixture itself: emission never happened, so rustc has no
    /// subject. `first` names one hard diagnostic so the caller can say WHICH refusal.
    SourceRefused {
        hard_diagnostics: usize,
        first: String,
    },
    /// Emission completed and the crate could not be written — a filesystem or closure-shape
    /// fact, again not a rustc verdict.
    CrateNotWritten { cause: String },
    /// The emitted closure was written and cargo answered over it.
    Measured {
        crate_dir: String,
        emitted_files: usize,
        cargo: CargoVerdict,
    },
}

/// True only when cargo COMPLETED with status zero over the written closure. Every other arm —
/// a gunbc refusal, an unwritten crate, a killed toolchain — answers false, because none of them
/// is evidence the emitted program type-checks.
#[cfg(test)]
pub(crate) fn fixture_closure_compiled(outcome: &FixtureClosureOutcome) -> bool {
    match outcome {
        FixtureClosureOutcome::Measured { cargo, .. } => cargo_verdict_compiled(cargo),
        _ => false,
    }
}

/// DID THIS ARM REACH A RUSTC VERDICT AT ALL -- which is a different question from whether it
/// compiled, and the one a caller must ask before reading a red as evidence.
///
/// TRUE only for a cargo run that COMPLETED and reported its own exit status. A red (`status`
/// non-zero) is reached: rustc ran and refused, which is half of what a discriminator is for.
/// Everything else answered nothing about the emitted bytes and must not be counted as if it had:
///
/// - `SourceRefused` -- gunbc refused the fixture, so nothing was emitted and rustc had no subject;
/// - `CrateNotWritten` -- the closure never reached disk;
/// - `NotAttempted` -- the toolchain was never invoked;
/// - `DidNotComplete` -- cargo was killed or failed to spawn, reporting no status of its own.
///
/// THIS EXISTS BECAUSE A CALL SITE GOT IT WRONG (review 58120). A caller counted only
/// `CrateNotWritten` as unanswered and therefore treated a fixture gunbc REJECTED, and a cargo run
/// that never FINISHED, as successfully measured -- reporting success without reaching rustc,
/// which is the fail-open §5 forbids and which contradicted this module's own outcome separation.
/// That caller was a CLI mode since withdrawn, so the specimen no longer exists in the tree; the
/// predicate stays because the defect was a caller enumerating a SUBSET of the arms that cannot
/// answer, and the next caller would enumerate a different subset. A predicate beside the carrier
/// is what makes that unavailable, rather than a `matches!` repeated at each call site.
#[cfg(test)]
pub(crate) fn fixture_closure_reached_rustc(outcome: &FixtureClosureOutcome) -> bool {
    match outcome {
        FixtureClosureOutcome::Measured { cargo, .. } => {
            matches!(cargo, CargoVerdict::Completed { .. })
        }
        FixtureClosureOutcome::SourceRefused { .. }
        | FixtureClosureOutcome::CrateNotWritten { .. } => false,
    }
}

/// The diagnostic line attributing a red to the fixture's OWN emitted module, if there is one.
///
/// A CALLER MUST BE ABLE TO SEPARATE "RUSTC REFUSED THIS FIXTURE" FROM "THE CLOSURE WAS ALREADY
/// RED". The subject is one module inside a closure of hundreds; a bare non-zero status says the
/// crate did not build and says nothing about which member broke. So the red direction is only
/// claimable when a diagnostic line names the fixture's emitted file, which is what this returns.
#[cfg(test)]
pub(crate) fn fixture_closure_attributed_line(outcome: &FixtureClosureOutcome) -> Option<&str> {
    match outcome {
        FixtureClosureOutcome::Measured { cargo, .. } => cargo_verdict_probe_line(cargo),
        _ => None,
    }
}

/// The rustc diagnostic HEADER governing the attributed line -- WHAT rustc refused in the
/// fixture's own emitted module, as against `fixture_closure_attributed_line`'s WHERE.
#[cfg(test)]
pub(crate) fn fixture_closure_attributed_diagnostic(
    outcome: &FixtureClosureOutcome,
) -> Option<&str> {
    match outcome {
        FixtureClosureOutcome::Measured { cargo, .. } => cargo_verdict_probe_diagnostic(cargo),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn fixture_closure_summary(outcome: &FixtureClosureOutcome) -> String {
    match outcome {
        FixtureClosureOutcome::SourceRefused {
            hard_diagnostics,
            first,
        } => format!("SourceRefused hard={hard_diagnostics} first={first}"),
        FixtureClosureOutcome::CrateNotWritten { cause } => {
            format!("CrateNotWritten cause={cause}")
        }
        FixtureClosureOutcome::Measured {
            crate_dir,
            emitted_files,
            cargo,
        } => format!(
            "Measured files={emitted_files} dir={crate_dir} cargo={}",
            cargo_verdict_summary(cargo)
        ),
    }
}

/// The emitted Rust module basename for a `.dag` module path — `a.b.c` emits `src/a_b_c.rs`.
///
/// It is the ATTRIBUTION SYMBOL for the fixture route, and it is derived from the fixture's own
/// module line rather than passed in beside it: a caller-supplied symbol could name a module the
/// fixture does not declare, and every red would then be reported as unattributed while the
/// fixture's real diagnostics sat in the same stderr.
#[cfg(test)]
fn fixture_rust_module(source: &str) -> Result<String, String> {
    source
        .lines()
        .find(|line| line.starts_with("module "))
        .map(|line| line.trim_start_matches("module ").trim().replace('.', "_"))
        .ok_or_else(|| "the fixture source declares no module line".to_string())
}

/// Emit one in-memory `.dag` fixture's closure and hand it to rustc.
///
/// The fixture's imports are resolved transitively against the live tree by
/// `resolve_virtual_source_with_imports` — the same resolver `compile_dag_rust_emit_check` uses,
/// so a fixture that compiles under one reaches the same closure under the other.
#[cfg(test)]
pub(crate) fn fixture_closure_rustc_verdict(
    source: &str,
    probe_root: &Path,
) -> FixtureClosureOutcome {
    let rust_module = match fixture_rust_module(source) {
        Ok(module) => module,
        Err(cause) => return FixtureClosureOutcome::CrateNotWritten { cause },
    };
    eprintln!("fixture-closure: {rust_module} emitting");
    let module_index = crate::cli_run::build_module_path_index_from_witness_roots();
    let sources =
        crate::cli_run::resolve_virtual_source_with_imports("fixture.dag", source, &module_index);
    let result = crate::v1_compiler_compile::compile_sources(
        std::rc::Rc::new(sources.into()),
        crate::v1_compiler_artifact::RenderTarget::Rust,
    );
    let hard: Vec<String> = result
        .diagnostics
        .iter()
        .filter(|d| crate::cli_run::compile_clean_diagnostic_is_hard(d))
        .map(|d| format!("{d:?}"))
        .collect();
    if !hard.is_empty() {
        return FixtureClosureOutcome::SourceRefused {
            hard_diagnostics: hard.len(),
            first: hard[0].chars().take(400).collect(),
        };
    }
    // The crate slug is the fixture's own emitted module, so two fixtures never share a package
    // name in the shared target directory — the fingerprint-aliasing fail-open `probe_package_name`
    // records for the entry route applies identically here.
    let (crate_dir, emitted_files) =
        match write_probe_crate_files(&result.files, probe_root, &rust_module) {
            Ok(pair) => pair,
            Err(cause) => return FixtureClosureOutcome::CrateNotWritten { cause },
        };
    eprintln!(
        "fixture-closure: {rust_module} emitted {emitted_files} file(s) into {} — cargo",
        crate_dir.display()
    );
    let cargo = run_cargo(
        &crate_dir,
        &process_workspace_root(),
        &format!("{rust_module}.rs"),
    );
    eprintln!(
        "fixture-closure: {rust_module} {}",
        cargo_verdict_summary(&cargo)
    );
    FixtureClosureOutcome::Measured {
        crate_dir: crate_dir.display().to_string(),
        emitted_files,
        cargo,
    }
}

/// THE FIXTURE ROUTE'S OWN DISCRIMINATING PAIR, EXECUTED RATHER THAN DESCRIBED.
///
/// A route that reaches rustc proves nothing on its own: a green says only that SOME crate
/// compiled, and DESIGN §4b calls a check that cannot go red worse than absent because it is
/// cited as coverage. So the route ships with the two arms that make its answer information —
/// run together, in one report, because either alone is satisfiable by an instrument that ignores
/// its input.
///
/// THE POSITIVE CONTROL is a self-contained fixture whose emitted closure must compile. It fails
/// if the crate writer, the manifest or the emitter regresses, and it is what stops the red arm
/// below from passing for an unrelated reason.
///
/// THE RED IS MEANING-LEVEL, WHICH IS THE WHOLE POINT OF REACHING RUSTC. Its fixture is the one
/// `gunbc.rung_drop` `text_boundary_identity_wall` names: a NON-LITERAL kernel-String value at an
/// exact `v2.std.text.String` boundary, which the corpus-wide compatibility relation admits by
/// leaf-name spelling and which therefore emits a host representation where the structural
/// carrier was declared. A substring oracle over the emitted text cannot see it — the emitted
/// bytes contain every substring a spelling assertion would look for. rustc's type checker does,
/// and that row's temporary rung ("on the emitted-Rust self-host path the class is mechanically
/// preventable — ... renders as a Rust type mismatch and the mandatory emitted-crate cargo check
/// refuses it") is a claim about exactly this, made about a fixture the required phase's entry
/// roster cannot reach. This arm is where that claim executes.
///
/// ATTRIBUTION IS PART OF THE RED AND NOT A NICETY. The red fixture's closure is hundreds of
/// modules; a bare non-zero status could come from any of them. So the arm passes only when a
/// diagnostic line NAMES the fixture's own emitted module — otherwise the report says
/// `unattributed` and the pair fails, rather than crediting a red the fixture did not cause.
#[derive(Debug, Clone)]
#[cfg(test)]
pub(crate) struct FixtureDiscrimination {
    pub(crate) green: FixtureClosureOutcome,
    pub(crate) red: FixtureClosureOutcome,
}

/// THE TWO ARMS ARE FIXTURE FILES, NOT STRING CONSTANTS IN THIS FILE, and the difference is
/// §3 rather than taste. The same arms are files any caller can hand this route by path; carrying
/// a second copy of their source here would be one fixture with two authorities
/// that drift the first time either is edited, and the arm this pair adjudicates would stop being
/// the arm anyone else runs.
///
/// The positive control has no imports, so its closure is its own emitted module plus the runtime.
#[cfg(test)]
const FIXTURE_GREEN_PATH: &str = "fixtures/fixture_closure_rustc/green_probe.dag";

/// The meaning-level red: `concat` answers the kernel string, the boundary declares the
/// structural `v2.std.text.String`, and no homomorphism row stands between them.
#[cfg(test)]
const FIXTURE_RED_PATH: &str = "fixtures/fixture_closure_rustc/text_nonliteral_probe.dag";

/// THE ERROR CLASS THE RED ARM CLAIMS, NAMED IN RUSTC'S OWN VOCABULARY: `E0308`, the mismatched
/// types code of the rustc error index. It is cited rather than coined because the arm's whole
/// claim is about what THAT compiler does, and `gunbc.rung_drop` `text_boundary_identity_wall`
/// states the class in the same words — a host representation emitted where the structural
/// carrier was declared "renders as a Rust type mismatch".
///
/// WHAT IT DISCRIMINATES AND WHAT IT DOES NOT, so nobody reads it as more: it separates a TYPE
/// MISMATCH in the fixture's emitted module from every other way that module could be refused. It
/// does not pin WHICH two types were mismatched — that pair sits in rustc's `expected`/`found`
/// notes, whose wording is not part of the error index and would bind this arm to a rustc
/// release. Every arm's diagnostic lines are reported in full beside the verdict, so a reader who
/// needs the pair reads it there.
#[cfg(test)]
const FIXTURE_RED_EXPECTED_RUSTC_CODE: &str = "E0308";

/// AN UNREADABLE ARM IS `CrateNotWritten`, NAMING THE PATH -- never an empty source quietly
/// compiled. An empty `.dag` text emits a crate that builds, so substituting one would turn a
/// missing fixture into a GREEN control and a red arm that stopped discriminating.
#[cfg(test)]
fn fixture_arm_verdict(rel_path: &str, probe_root: &Path) -> FixtureClosureOutcome {
    let path = process_workspace_root().join(rel_path);
    match std::fs::read_to_string(&path) {
        Ok(source) => fixture_closure_rustc_verdict(&source, probe_root),
        Err(e) => FixtureClosureOutcome::CrateNotWritten {
            cause: format!("reading the fixture arm {}: {e}", path.display()),
        },
    }
}

#[cfg(test)]
pub(crate) fn run_fixture_closure_discrimination(probe_root: &Path) -> FixtureDiscrimination {
    FixtureDiscrimination {
        green: fixture_arm_verdict(FIXTURE_GREEN_PATH, probe_root),
        red: fixture_arm_verdict(FIXTURE_RED_PATH, probe_root),
    }
}

/// The pair passes only when BOTH directions hold: the control compiled, and the meaning-level
/// fixture was refused BY RUSTC, in its own emitted module, WITH THE ERROR CLASS THE ARM CLAIMS.
///
/// A `SourceRefused` red does NOT pass. gunbc refusing the fixture is a different and better
/// outcome — it means the wall climbed — but it is not this pair's subject, and reporting it as
/// a pass here would let the route's evidence green while nothing reached rustc at all.
///
/// THE ERROR CODE IS PART OF THE RED, AND WITHOUT IT THE ARM CLAIMED MORE THAN IT CHECKED. An
/// earlier revision required only that SOME diagnostic line name the fixture's emitted module, so
/// any refusal reported in that file passed: a fixture edited into a syntax error, an emitter
/// regression producing an unresolved path, an arity fault — every one of them reds the arm in
/// the right file and none of them is the class this pair adjudicates. The arm would have gone on
/// reporting PASSED while the text-boundary subject it names had stopped being the subject.
#[cfg(test)]
pub(crate) fn fixture_discrimination_passed(pair: &FixtureDiscrimination) -> bool {
    fixture_closure_compiled(&pair.green)
        && fixture_closure_reached_rustc(&pair.red)
        && !fixture_closure_compiled(&pair.red)
        && fixture_closure_attributed_line(&pair.red).is_some()
        && fixture_closure_attributed_diagnostic(&pair.red)
            .is_some_and(|diagnostic| diagnostic.contains(FIXTURE_RED_EXPECTED_RUSTC_CODE))
}

/// EACH ARM REPORTS ITS OWN DIAGNOSTIC, NOT A VERDICT ABOUT ITSELF.
///
/// A pass/fail per arm is not enough for the thing this route exists to carry. A two-arm emitter
/// discriminator separates its arms BY WHAT RUSTC SAID -- one arm `E0308 expected Quantity found
/// Time`, the other `E0573 expected type found variant` -- and a route reporting only that both
/// arms were red cannot tell those two apart, so it cannot discriminate the emitter behaviours
/// that produced them. So every arm's diagnostic lines are reported, red or green, and the
/// summary sits beside them rather than replacing them.
#[cfg(test)]
pub(crate) fn fixture_discrimination_report(pair: &FixtureDiscrimination) -> Vec<String> {
    let mut lines = vec![format!(
        "fixture-closure: control {}",
        fixture_closure_summary(&pair.green)
    )];
    lines.extend(
        fixture_arm_diagnostic_lines("control", &pair.green)
            .into_iter()
            .map(|line| format!("fixture-closure: {line}")),
    );
    lines.push(format!(
        "fixture-closure: red     {}",
        fixture_closure_summary(&pair.red)
    ));
    lines.extend(
        fixture_arm_diagnostic_lines("red", &pair.red)
            .into_iter()
            .map(|line| format!("fixture-closure: {line}")),
    );
    lines.push(format!(
        "fixture-closure: red attribution {}",
        fixture_closure_attributed_line(&pair.red).unwrap_or("unattributed")
    ));
    lines.push(format!(
        "fixture-closure: red diagnostic {} (expected {})",
        fixture_closure_attributed_diagnostic(&pair.red).unwrap_or("none"),
        FIXTURE_RED_EXPECTED_RUSTC_CODE
    ));
    lines.push(format!(
        "fixture-closure: pair {}",
        if fixture_discrimination_passed(pair) {
            "PASSED"
        } else {
            "FAILED"
        }
    ));
    lines
}

/// One arm's diagnostics, whatever arm the outcome took.
///
/// A gunbc refusal reports the diagnostic gunbc produced; a cargo run reports what cargo said.
/// Neither is silently rendered as the other, because the reader's next move differs: one names
/// a fixture the compiler rejected, the other names emitted bytes rustc rejected.
#[cfg(test)]
pub(crate) fn fixture_arm_diagnostic_lines(
    label: &str,
    outcome: &FixtureClosureOutcome,
) -> Vec<String> {
    match outcome {
        FixtureClosureOutcome::SourceRefused { first, .. } => {
            vec![format!("{label}| gunbc {first}")]
        }
        FixtureClosureOutcome::CrateNotWritten { cause } => {
            vec![format!("{label}| not-written {cause}")]
        }
        FixtureClosureOutcome::Measured { cargo, .. } => cargo_verdict_stderr_tail(cargo)
            .lines()
            .map(|line| format!("{label}| cargo {line}"))
            .collect(),
    }
}

/// Every configured entry, each run whatever the previous one did -- the stopped-line audit
/// shape: report everything, green nothing.
///
/// AN EMPTY ROSTER REFUSES. Zero entries compiled is the phase reaching no subject, not zero
/// breaks; reporting it clean is the empty-observation narrow.
/// THE NUMERATOR, IN THE SAME UNIT AS THE DENOMINATOR: modules the cover's closures REACHED.
///
/// WHY NOT THE ENTRY COUNT. `covered_entries=8 of 3900 authored modules` is not a fraction —
/// entries over modules — yet reads as a coverage ratio because it is formatted like one. It
/// understates by a wide, unknown margin (eight closures reach far more than eight modules), and
/// understating is not safe: a number that bad invites growing the entry roster, the move the
/// retirement trigger forbids (a trigger satisfied at forty-one entries leaves the corpus
/// unmeasured). An unpaired count tells no lie; a mismatched fraction does.
///
/// The union of the emitted module sets, read from crates already written: a readdir per entry,
/// no extra compilation. It moves up when the cover reaches new code, and not when an unrelated
/// emitter repair lands.
///
/// WHAT IT DOES NOT DISTINGUISH: reached AS AN ENTRY and reached ONLY AS A DEPENDENCY are both
/// counted. The second is weaker coverage — a dependency module is compiled, but no run emits
/// from its own closure, so an emit-stage diagnostic reachable only from ITS entry is invisible.
/// Splitting the two numerators is strictly better and not done here.
pub fn emit_compile_modules_reached(outcomes: &[EmitCompileOutcome]) -> usize {
    // `src/lib.rs` IS NOT DEDUPLICABLE BY NAME, AND UNIONING IT WOULD UNDER-COUNT. Each entry's
    // root module is written as `lib.rs` (the compiler refuses a crate without one), so every
    // entry contributes a DIFFERENT root under the SAME name; unioning would collapse N roots into
    // one, a numerator shrinking as the cover grows. Roots are counted per measured entry and
    // dependency modules unioned by name.
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
/// WHY THIS REPLACES THE COVERAGE FRACTION RATHER THAN JOINING IT. §4b(3) asks a rung drop for a
/// BOUNDED POPULATION, and a percentage is not one: it says how much is missing, never WHICH, so
/// nothing downstream can join it, refuse on it, or watch it shrink. The identities are the
/// population; a percentage may stand as context, never as the gap's identity or dissolution
/// trigger.
///
/// Universe: authored `.dag` modules under the invoked source roots. Selection: the declared
/// roster. Remainder: the set difference, RETAINED — written to a file beside the run and
/// digested, so the output names where the unselected identities are. Counts and digests are
/// printed; identities are persisted.
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
    // the walk does not find shows up as a selected identity absent from the universe, and the
    // remainder stays exact.
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

/// Persist the unselected identities beside the run. RETAINED means retained: the list, not its
/// size, so the remainder is a population a later operation can read.
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
/// producer; a report authored twice is one fact with two authorities, free to disagree on the
/// numbers a reader compares. So selection, remainder and context line are rendered here once.
///
/// Retention failure is RETURNED, not swallowed: the remainder is the declared population of what
/// this phase does not observe, and a count it could not persist has nothing behind it.
pub fn emit_compile_report(
    outcomes: &[EmitCompileOutcome],
    source_roots: &[String],
    probe_root: &Path,
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
    // THROUGH `probe_root()`, NEVER BY RESPELLING IT. This line composed the directory name a
    // second time, so when the root moved to `RUNNER_TEMP` one spelling was repaired and the
    // other not: crate dirs went under the per-job temp while the retention file went to the
    // host-shared `/tmp` and hit the same `EACCES` the reroot escaped. Two homes for one fact is
    // the §3 violation; the tell was two different paths in adjacent log lines. The
    // caller-selected `probe_root` is the authority; nothing below the mode boundary chooses
    // another root.
    let retained_dir = probe_root.to_string_lossy().to_string();
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

/// A ROOT WHOSE LAST WRITER DIED IS NOT A ROOT TO SILENTLY BUILD ON, SO A SECOND HOLDER REFUSES.
///
/// WHAT THIS LOCK IS FOR HAS NARROWED; the argument below was written for the wider case. Under a
/// per-job `RUNNER_TEMP` root two CONCURRENT runs cannot collide — no path they both name — so
/// the lock no longer prevents interleaving in CI. It still catches a previous attempt in THIS
/// job that died mid-flight, or two invocations given the same runner temp: both leave the tree's
/// state unestablished, which is what is worth refusing on.
///
/// The arms share one probe root and one cargo target directory — what makes the baseline warm
/// and the restore comparable. So two runs interleave: one's faulted tree is the other's
/// baseline, one's restore erases the other's red before it is read. Both report confidently.
///
/// MEASURED, NOT ANTICIPATED. Verifying the blunted-mutation arm, a stale background invocation
/// overlapped a foreground one: `Discriminated` with a red line quoting a `#[cfg]` WARNING over a
/// cargo run whose tail said `Finished` — a green compile reported as a discriminating red, handed
/// to the very arm that exists to catch one. A clean re-run answered `NotDiscriminating`.
///
/// The refusal is a lock file created exclusively, NOT a wait and NOT a private directory per run.
/// Waiting serializes into the same shared state with the same ambiguity; a private directory
/// throws away the warm target dir. Refusing is the fail-closed arm: line stops, cause typed and
/// located, operator sees two runs were attempted rather than a verdict computed across both.
fn acquire_probe_root_lock(root: &Path) -> Result<PathBuf, String> {
    std::fs::create_dir_all(root).map_err(|e| {
        format!(
            "could not create the caller-selected probe root {} ({e})",
            root.display()
        )
    })?;
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
        // A LIVE PEER AND AN UNWRITABLE ROOT ARE OPPOSITE REMEDIES, so opposite refusals.
        // `AlreadyExists` says investigate a concurrent run; `PermissionDenied` says the ROOT is
        // wrong and no peer exists. Collapsing them is the state-space conflation DESIGN names,
        // and cost a triage cycle when the catch-all string sent a reader hunting a concurrent
        // run on a runner that had none.
        Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => Err(format!(
            "the probe root {} is not writable by this process ({e}) — this is NOT a concurrent \
             run; the caller-selected root itself is wrong.",
            root.display()
        )),
        Err(e) => Err(format!("could not take {}: {e}", lock.display())),
    }
}

pub fn run_required_emit_compile(
    source_roots: &[String],
    probe_root: &Path,
) -> Result<Vec<EmitCompileOutcome>, String> {
    let entries = required_emit_compile_entries();
    if entries.is_empty() {
        return Err(
            "gunbc.ci_layer_roots required_emit_compile_entries is empty — the phase has no subject"
                .to_string(),
        );
    }
    // See `acquire_probe_root_lock`: a second concurrent run refuses rather than interleaving.
    let lock = acquire_probe_root_lock(probe_root)?;
    // A FAILED RESTORE ENDS THE RUN, not merely the entry.
    //
    // WHY IT IS TERMINAL RATHER THAN A FINDING SIBLINGS CONTINUE PAST, as every other refusal
    // here is: the arms share ONE cargo target directory, so after a failed restore its tree is
    // unknown -- possibly artifacts of a faulted crate. Every later baseline is unattributable,
    // and reporting them is execution-provenance loss one level out.
    //
    // AND IT MUST NOT BE PAPERABLE OVER BY A RE-RUN. A re-run re-emits from scratch, so a
    // transient restore failure vanishes and the phase greens -- the byte-exact restore becomes a
    // flaky step people re-run. Ending the run means the head has NO green from this phase,
    // rather than a green whose restore arm was never established.
    let mut outcomes = Vec::new();
    for entry in &entries {
        let outcome = run_emit_compile_entry(source_roots, probe_root, entry);
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
    // The lock is released at the one exit below the acquisition: every loop branch pushes an
    // outcome and falls through. A run killed before this point leaves the lock deliberately --
    // a probe root whose last writer died is not a state to silently build on.
    let _ = std::fs::remove_file(&lock);
    Ok(outcomes)
}

/// THESE ARE LOCAL-ONLY EVIDENCE AND ARE LABELLED AS SUCH. The Rust suite was removed from CI on
/// 2026-07-11 (DESIGN, Building & checks), so nothing here runs on the merge path or may be cited
/// as coverage. THE EXECUTED EVIDENCE FOR THIS PHASE IS THE PHASE ITSELF:
/// `establish_discriminating_red` runs on every required run, and a mutation that fails to go red
/// stops the line. These add what the in-run arm cannot check on itself: a non-`Discriminated`
/// mutation is a FAILURE, not a note, and the fault targets the ENTRY'S OWN emitted module, its
/// absence refused rather than substituted for.
#[cfg(test)]
mod tests {
    use super::*;

    /// The manifest is DERIVED, so this asserts the derivation reached the modeled rows, not a
    /// golden string: version-authority package header, the seed's runtime dependency set, and
    /// the path dependency the emitted closure links against.
    #[test]
    fn manifest_carries_the_modeled_dependency_rows() {
        let manifest = probe_manifest(Path::new("/repo"), "dag/std/logic.dag");
        // The package name is DERIVED PER ENTRY, so two entries cannot alias each other's cargo
        // fingerprints in the shared target directory.
        assert!(
            manifest.starts_with("[package]\nname = \"gunbc-emitted-closure-dag-std-logic-dag\"")
        );
        assert!(manifest.contains("edition = \"2021\""));
        // THE FEATURE SECTION IS A REGRESSION GUARD: the emitted `v1_rt.rs` gates on this feature,
        // and an undeclared feature compiles clean locally but fails under the required lane's
        // `RUSTFLAGS=-D warnings` -- invisible to every default-flag run, which is how it reached
        // CI once.
        assert!(manifest.contains("[features]"));
        assert!(manifest.contains("text_lookup_work_counter = []"));
        for name in ["im", "serde", "serde_json", "stacker"] {
            assert!(manifest.contains(name), "missing dependency row {name}");
        }
        assert!(manifest.contains("/repo/src/v1/stage0"));
        // `src/lib.rs` is cargo's own default, so restating it would be a second spelling.
        assert!(!manifest.contains("[lib]"));
    }

    /// One emitted crate on disk, authored by the caller, so each test states the shape it means.
    fn probe_tree(tag: &str, lib_rs: &str, files: &[&str]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "emit_compile_{tag}_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).expect("src");
        std::fs::write(dir.join("src/lib.rs"), lib_rs).expect("lib");
        for file in files {
            std::fs::write(dir.join(format!("src/{file}.rs")), "// emitted\n").expect("member");
        }
        dir
    }

    /// THE SUBJECT IS THE ENTRY'S OWN MODULE EVEN WHEN A SHARED MEMBER IS DECLARED FIRST.
    ///
    /// The measured shape of 7 of the 8 rostered entries: shared-core member first, entry
    /// second-to-last, emitted runtime last. The old selector mutated `std_error_primitives` here
    /// -- a verdict about the shared core wearing this entry's name.
    #[test]
    fn the_subject_is_the_entry_own_module_past_a_leading_shared_member() {
        let dir = probe_tree(
            "shared_first",
            "pub mod std_error_primitives;\npub mod v2_std_node;\npub mod v1_rt;\n",
            &["std_error_primitives", "v2_std_node", "v1_rt"],
        );
        let subject = mutation_subject(&dir, "v2_std_node").expect("the entry's own module");
        assert_eq!(mutation_subject_rust_module(&subject), "v2_std_node");
        assert_eq!(mutation_subject_name(&subject), "EntryOwnModule");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// AND WHEN IT IS DECLARED FIRST, the other measured shape (2 of 8: `std_abi`, `std_logic`).
    /// Its own case because ORDER IS NOT THE MECHANISM: the tempting repair -- mutate the LAST
    /// module -- picks `v1_rt`, the emitted runtime, in all 8, strictly more shared.
    #[test]
    fn the_subject_is_the_entry_own_module_when_it_is_declared_first() {
        let dir = probe_tree(
            "entry_first",
            "pub mod std_logic;\npub mod v1_rt;\n",
            &["std_logic", "v1_rt"],
        );
        let subject = mutation_subject(&dir, "std_logic").expect("the entry's own module");
        assert_eq!(mutation_subject_rust_module(&subject), "std_logic");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// THE ABSENT ENTRY MODULE REFUSES AND NAMES THE ENTRY -- IT DOES NOT FALL BACK.
    ///
    /// The whole point of the change, and the case a fallback would report as `Discriminated`:
    /// the closure lost its own root and still compiles, since every remaining member is a
    /// dependency of the missing root and nothing references it back.
    #[test]
    fn an_entry_module_missing_from_the_closure_refuses_rather_than_substituting() {
        let dir = probe_tree(
            "entry_dropped",
            "pub mod std_error_primitives;\npub mod v1_rt;\n",
            &["std_error_primitives", "v1_rt"],
        );
        let refusal = mutation_subject(&dir, "v2_std_node").expect_err("no substitution");
        match &refusal {
            MutationSubjectRefusal::EntryModuleNotDeclared {
                entry_module,
                declared,
            } => {
                assert_eq!(entry_module, "v2_std_node");
                assert_eq!(declared, &["std_error_primitives", "v1_rt"]);
            }
            other => panic!("wrong refusal: {other:?}"),
        }
        let summary = mutation_subject_refusal_summary(&refusal);
        assert!(summary.contains("v2_std_node"), "{summary}");
        // The members that WERE available are named: what a fallback would have chosen, and that
        // nothing chose it.
        assert!(summary.contains("std_error_primitives"), "{summary}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// DECLARED AND WRITTEN ARE TWO FACTS. A `pub mod` line with no file behind it is a write
    /// defect, not an emission one, and it gets its own refusal for that reason.
    #[test]
    fn an_entry_module_declared_without_a_file_refuses_as_a_write_defect() {
        let dir = probe_tree(
            "entry_unwritten",
            "pub mod v2_std_node;\npub mod v1_rt;\n",
            &["v1_rt"],
        );
        let refusal = mutation_subject(&dir, "v2_std_node").expect_err("no substitution");
        assert!(matches!(
            refusal,
            MutationSubjectRefusal::EntryModuleFileMissing { .. }
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// AN UNREADABLE MANIFEST IS NOT AN EMPTY CLOSURE. Rendering it as one reports the EMISSION
    /// arm -- `the entry's own module is not declared` -- for a crate nobody read, sending the
    /// reader to the emitter over a filesystem fault.
    #[test]
    fn an_unreadable_closure_manifest_refuses_on_its_own_cause() {
        let dir = std::env::temp_dir().join(format!(
            "emit_compile_no_manifest_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).expect("src");
        let refusal = mutation_subject(&dir, "v2_std_node").expect_err("no manifest, no subject");
        assert!(matches!(
            refusal,
            MutationSubjectRefusal::ClosureManifestUnreadable { .. }
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// AN ENTRY THE RUN NEVER REACHED IS NOT A PASS. The terminal-restore arm gives "not reached"
    /// its own spelling; if it could pass, ending early would silently green every later entry.
    #[test]
    fn an_unreached_entry_is_not_a_pass() {
        assert!(!emit_compile_outcome_passed(
            &EmitCompileOutcome::NotExecuted {
                entry: "e.dag".to_string(),
                cause: "the run ended earlier".to_string(),
            }
        ));
    }

    /// EVERY NON-`Discriminated` MUTATION ARM FAILS THE PHASE. A test because the tempting
    /// weakening -- mutation advisory beside a green baseline -- would make this phase the
    /// decoration it exists not to be.
    /// THE PROBE ROOT HAS EXACTLY ONE SPELLING.
    ///
    /// It had two. When the root moved to `RUNNER_TEMP`, `probe_root()` was repaired and a
    /// hand-composed copy in the report was not: crate dirs under the per-job temp, retention
    /// file in the host-shared `/tmp`, hitting the very `EACCES` the reroot escaped. Not a missed
    /// callsite -- a fact with two homes.
    ///
    /// So this pins the SPELLING, not the behaviour: a behavioural test cannot catch a second
    /// composition, because both spellings are correct until the authority moves.
    #[test]
    fn the_probe_root_name_is_composed_in_exactly_one_place() {
        let src = include_str!("emitted_closure_compile_host.rs");
        // The needle is ASSEMBLED, not written: a literal would appear in this file and the test
        // would count itself as a second spelling. Caught by the test failing on its own text.
        let dir_name = format!("{}{}", "gunbc-emit-", "compile");
        let needle = format!("{dir_name:?}");
        let compositions = src.matches(needle.as_str()).count();
        assert_eq!(
            compositions, 1,
            "the probe root directory name must have one authority; a second spelling silently \
             keeps the old name when the authority moves"
        );
    }

    /// A REQUIRED PATH DOES NOT PROBE THE HOST-SHARED FALLBACK.
    ///
    /// The safety property is that `RUNNER_TEMP` identifies a per-job directory. Falling back to
    /// system temp and checking writability makes safety an accident of the runner's filesystem
    /// state. The missing and empty arms refuse before a path exists to inspect.
    #[test]
    fn the_probe_root_does_not_exist_without_runner_temp() {
        for absent in [None, Some(std::ffi::OsStr::new(""))] {
            let refusal = required_ci_probe_root_from_runner_temp(absent)
                .expect_err("an absent runner temp must not produce a fallback path");
            assert!(refusal.contains("RUNNER_TEMP is unset or empty"));
            assert!(refusal.contains("refuses rather than falling back"));
        }

        assert_eq!(
            required_ci_probe_root_from_runner_temp(Some(std::ffi::OsStr::new("/runner/job")))
                .expect("a declared runner temp owns the probe root"),
            PathBuf::from("/runner/job/gunbc-emit-compile")
        );
    }

    /// A FAILED RESTORE MUST WIN OVER EVERY NON-TERMINAL FAULT VERDICT.
    ///
    /// The two conditions are independent and co-occur: the fault arm can be green, incomplete
    /// or unattributed while the restore fails. Only `RestoreFailed` ends the run; every
    /// `NotDiscriminating` arm lets siblings continue against an unestablished tree. This pins
    /// the ORDER, because both orders typecheck and only one is safe -- how it was wrong before.
    #[test]
    fn a_failed_restore_is_not_masked_by_a_non_terminal_fault_verdict() {
        // Source order is the guarantee: both restore adjudications must precede the first
        // fault-verdict return, or a masked terminal failure is reachable again.
        let src = include_str!("emitted_closure_compile_host.rs");
        let body = &src[src
            .find("fn establish_discriminating_red")
            .expect("the function this test pins must exist")..];
        let first_restore = body
            .find("MutationVerdict::RestoreFailed")
            .expect("the restore adjudication must be present");
        let first_fault = body
            .find("MutationVerdict::NotDiscriminating")
            .expect("the fault verdicts must be present");
        assert!(
            first_restore < first_fault,
            "a RestoreFailed adjudication must precede every NotDiscriminating return: a \
             terminal failure returned after a non-terminal one is swallowed, and the run \
             continues against an unrestored tree"
        );
    }

    /// THE ATTRIBUTED LOCATION MUST CARRY ITS OWN DIAGNOSTIC, NOT A NEIGHBOUR'S.
    ///
    /// This is the parse the fixture route's red arm rests on, and it runs on every push while
    /// the cargo route around it does not. The stderr below is the shape rustc emits for a
    /// multi-module failure: an unrelated diagnostic in another module of the closure, then the
    /// one in the attributed file. Reading the header from the WRONG direction — nearest below,
    /// or first in the run — returns `E0433` for a location governed by `E0308`, and the pair
    /// would then adjudicate an error class the fixture did not produce.
    #[test]
    fn the_attributed_diagnostic_is_the_header_governing_the_attributed_line() {
        let stderr = "\
error[E0433]: failed to resolve: use of undeclared crate or module `nope`
  --> src/some_other_module.rs:4:5
   |
4  |     nope::thing()
   |     ^^^^ use of undeclared crate or module
error[E0308]: mismatched types
  --> src/fixture_probe.rs:3:52
   |
3  | pub fn p() -> String { concat() }
   |               ------   ^^^^^^^^ expected `String`, found `&str`
error: aborting due to 2 previous errors
";
        let (line, diagnostic) = attributed_diagnostic(stderr, "fixture_probe.rs");
        assert_eq!(
            line.as_deref(),
            Some("--> src/fixture_probe.rs:3:52"),
            "the attributed line is the location naming the symbol"
        );
        assert_eq!(
            diagnostic.as_deref(),
            Some("error[E0308]: mismatched types"),
            "the governing header is the last one ABOVE the attributed line, not the first in \
             the run and not the nearest below it"
        );
    }

    /// A SYMBOL STDERR NEVER NAMES IS `None` IN BOTH FIELDS -- never the run's first diagnostic
    /// standing in for an attribution that was not made. Answering a header for an unattributed
    /// run is the fabricated attribution the fixture pair exists to refuse.
    #[test]
    fn an_unnamed_symbol_attributes_nothing_rather_than_the_first_diagnostic() {
        let stderr = "error[E0308]: mismatched types\n  --> src/elsewhere.rs:1:1\n";
        let (line, diagnostic) = attributed_diagnostic(stderr, "fixture_probe.rs");
        assert_eq!(line, None);
        assert_eq!(diagnostic, None);
    }

    /// A WARNING NAMING THE FILE IS NOT AN ATTRIBUTION OF A REFUSAL.
    ///
    /// Both callers of this scan ask why a run was REFUSED. An emitted module routinely collects
    /// warnings — an unused import, a dead item — and they are printed above the error that
    /// actually stopped the build. Attributing from one would report a line that refused nothing,
    /// and would then carry a diagnostic header that is not an error at all.
    #[test]
    fn a_warning_naming_the_file_does_not_attribute_the_refusal() {
        let stderr = "\
warning: unused import: `std::rc::Rc`
  --> src/fixture_probe.rs:1:5
error[E0433]: failed to resolve: use of undeclared crate or module `nope`
  --> src/some_other_module.rs:4:5
error: could not compile `probe` (lib) due to 1 previous error
";
        let (line, diagnostic) = attributed_diagnostic(stderr, "fixture_probe.rs");
        assert_eq!(
            line, None,
            "a file named only by a warning is unattributed: the warning refused nothing"
        );
        assert_eq!(
            diagnostic, None,
            "and no error header elsewhere in the run stands in for one"
        );
    }

    #[test]
    fn a_green_baseline_does_not_pass_without_the_discrimination() {
        let green = CargoVerdict::Completed {
            status: 0,
            stderr_tail: String::new(),
            probe_line: None,
            probe_diagnostic: None,
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
            // THE REFUSAL IS A PHASE FAILURE, not a note beside a green baseline: a closure that
            // lost its own entry module is the emission defect this phase exists to catch.
            MutationVerdict::SubjectRefused {
                refusal: MutationSubjectRefusal::EntryModuleNotDeclared {
                    entry_module: "v2_std_node".to_string(),
                    declared: vec!["v1_rt".to_string()],
                },
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
        // THE SUBJECT IS OBTAINED THE ONLY WAY IT CAN BE: through `mutation_subject`, over a real
        // tree. An earlier revision wrote `MutationSubject { rust_module: .. }` directly; the
        // privacy boundary turned that into a COMPILE ERROR (`E0451: field rust_module is
        // private`) -- executed evidence the wall is structural, since module-scoped privacy
        // beside the constructor would have left the literal compiling.
        let dir = probe_tree("passing_subject", "pub mod std_logic;\n", &["std_logic"]);
        let subject = mutation_subject(&dir, "std_logic").expect("the entry's own module");
        let discriminated = EmitCompileOutcome::Measured {
            entry: "e.dag".to_string(),
            crate_dir: "/tmp/x".to_string(),
            emitted_files: 1,
            baseline: green,
            mutation: MutationVerdict::Discriminated {
                subject,
                red_line: "error[E0308]".to_string(),
            },
        };
        assert!(emit_compile_outcome_passed(&discriminated));
        assert!(emit_compile_outcome_summary(&discriminated).contains("subject=EntryOwnModule"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
