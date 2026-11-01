#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;
use v1_compiler::cli_run::PhaseProfile;

// THE REQUIRED-CI MEASUREMENT RECEIPT AND ITS ADMISSION RELATION NOW LIVE IN THE LIBRARY,
// `v1_compiler::cli_run::required_ci_measurement`, which carries the hand-Rust deferral note
// this block used to. They moved for one reason: the relation that decides a required lane's
// exit status was wrong on at least two landed runs and nothing executed it. A library module
// has a documented execution route (`cargo test --release -p v1-compiler --lib`); a module
// beside this binary has none. What stays here is what only a process can do -- print the
// located lines and set the status.

fn require_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    match args.get(idx) {
        Some(v) => Ok(v.clone()),
        None => {
            eprintln!("claim_executor: {} requires a value", flag);
            Err(ExitCode::from(2))
        }
    }
}

/// Path-valued arguments resolve against the PROCESS CWD at the CLI boundary, refusing
/// on a nonexistent path — never falling back to the compile-time-baked workspace root
/// (`v1_compiler::cli_run::resolve_cli_path_arg`; DESIGN §5 fail-open closed there).
fn require_path_value(args: &[String], idx: usize, flag: &str) -> Result<String, ExitCode> {
    let given = require_value(args, idx, flag)?;
    match v1_compiler::cli_run::resolve_cli_path_arg("claim_executor", flag, &given) {
        Ok(resolved) => Ok(resolved),
        Err(msg) => {
            eprintln!("{msg}");
            Err(ExitCode::from(2))
        }
    }
}

/// Verify each declared release artifact exists, is executable, and is non-empty — failing CLOSED
/// (exit 1) with the GitHub-Actions `::error::` annotation on the first violation. This is the
/// in-binary home of what was previously inline `[ -x ]` / `[ -s ]` shell in the generated ci.yml
/// (DESIGN §5 fail-open guard: an sccache-served truncated/empty cached artifact after a
/// `successful` build). The artifact paths are authored by the .dag spec and passed positionally.
fn verify_build_artifacts(paths: &[String]) -> Result<ExitCode, ExitCode> {
    use std::os::unix::fs::PermissionsExt;
    if paths.is_empty() {
        eprintln!("claim_executor: --verify-build-artifacts requires at least one artifact path");
        return Err(ExitCode::from(2));
    }
    for path in paths {
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path.as_str());
        match std::fs::metadata(path) {
            Ok(meta) => {
                let executable = meta.permissions().mode() & 0o111 != 0;
                if !meta.is_file() || !executable {
                    eprintln!(
                        "::error::build verification: declared artifact '{name}' absent or not \
                         executable after a 'successful' build (sccache/cache corruption — DESIGN \
                         §5 fail-open); failing closed: {path}"
                    );
                    return Err(ExitCode::from(1));
                }
                if meta.len() == 0 {
                    eprintln!(
                        "::error::build verification: declared artifact '{name}' is zero-byte after \
                         a 'successful' build (sccache served a truncated/empty cached artifact — \
                         DESIGN §5 fail-open); failing closed: {path}"
                    );
                    return Err(ExitCode::from(1));
                }
            }
            Err(_) => {
                eprintln!(
                    "::error::build verification: declared artifact '{name}' absent or not \
                     executable after a 'successful' build (sccache/cache corruption — DESIGN §5 \
                     fail-open); failing closed: {path}"
                );
                return Err(ExitCode::from(1));
            }
        }
    }
    eprintln!(
        "claim_executor: build-artifact verification passed ({} declared release binar{} present \
         + non-empty)",
        paths.len(),
        if paths.len() == 1 { "y" } else { "ies" }
    );
    Ok(ExitCode::SUCCESS)
}

fn run() -> Result<ExitCode, ExitCode> {
    // BEFORE ANY WORK, AND BEFORE ARGUMENTS, because a limit acquired after the first large
    // allocation bounds nothing that already happened. On a machine that already binds this
    // process — every self-hosted CI runner — this is a no-op that says so; on an executor with
    // no limit at all it creates one, so the whole-corpus adjudication can run somewhere the
    // kernel is enforcing a ceiling instead of thrashing against a machine.
    // A refused bind STOPS THE LINE: the run asked to be bounded, it is not bounded, and
    // proceeding would be exactly the escape hatch the request existed to close.
    // Authority: `gunbc.host.memory_cgroup_binding`.
    let bind = v1_compiler::memory_governor::apply_memory_cgroup_bind();
    eprintln!("{}", v1_compiler::memory_governor::cgroup_bind_note(&bind));
    if v1_compiler::memory_governor::cgroup_bind_refusal_diagnostic(&bind).is_some() {
        return Err(ExitCode::FAILURE);
    }

    let args: Vec<String> = std::env::args().collect();
    let mut source_roots: Vec<String> = Vec::new();
    let mut verify_artifacts: Vec<String> = Vec::new();
    let mut verify_artifacts_mode = false;
    let mut write_checker_inputs: Option<String> = None;
    let mut required_v2_emission_mode = false;
    let mut required_emit_compile_mode = false;
    let mut v2_native_route_mode = false;
    let mut required_regen_mode = false;
    let mut emit_partition_crates_mode = false;
    let mut emit_partition_crates_write = false;
    let mut regen_round_cost_mode = false;
    let mut base_mirror_mode = false;
    let mut regen_affected_set_mode = false;
    let mut required_admitted_set_mode = false;
    let mut regen_affected_scope = false;
    let mut regen_candidate_dir = "target/stage0-regen-candidate".to_string();
    let mut regen_receipt_path = "target/stage0-regen-receipt.json".to_string();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--verify-build-artifacts" => {
                // All remaining positional args are declared release-artifact paths to verify.
                verify_artifacts_mode = true;
                i += 1;
                while i < args.len() {
                    verify_artifacts.push(args[i].clone());
                    i += 1;
                }
                break;
            }
            // THE PRODUCER OF A RESTORED PAIR'S CHECKER-INPUT RECORD, run by the job that built the
            // pair (see `write_checker_inputs_record`). Takes no roots.
            "--write-checker-inputs" => {
                i += 1;
                // An OUTPUT path: it does not exist yet, so it is not resolved-and-required.
                write_checker_inputs = Some(require_value(&args, i, "--write-checker-inputs")?);
            }
            "--source-root" => {
                i += 1;
                source_roots.push(require_path_value(&args, i, "--source-root")?);
            }
            "--required-v2-emission" => {
                required_v2_emission_mode = true;
            }
            // THE PHASE AS ITS OWN ENTRY POINT, for the reason `--required-v2-emission` is one:
            // running this alone is a real local action, and it runs the SAME producer the
            // required phase runs, so a green here and a green there cannot be two facts.
            "--required-emit-compile" => {
                required_emit_compile_mode = true;
            }
            // THE NATIVE ROUTE AS AN OPERATOR-INVOKED INSTRUMENT, NOT A REQUIRED PHASE. It ran
            // as the required `v2-native` lane from 2026-09-09 until the 2026-09-11 operator
            // ruling deleted that job (~4h wall on every push against a 180-minute envelope;
            // gunbc.rung_drop v2_native_route_off_the_merge_path). The harness and the
            // admission authority are unchanged — this flag is the one route to them, so a
            // receipt minted here and one minted by the deleted lane cannot be two facts.
            "--v2-native-route" => {
                v2_native_route_mode = true;
            }
            "--required-regen" => {
                required_regen_mode = true;
            }
            // THE SANCTIONED PRODUCER for the derived partition's boundary files. Named by the
            // phase's own refusal, because a stop whose only remedy does not exist is what
            // produced this class in the first place.
            "--emit-partition-crates" => {
                emit_partition_crates_mode = true;
            }
            "--write" => {
                emit_partition_crates_write = true;
            }
            // ONE PRICED REGEN ROUND: seed build, the same `--required-regen` emit, install of
            // what drifted, rebuild from the installed seed, diff — every phase on two clocks,
            // rendered by `gunbc.regen_round_cost`. It installs into src/v1/stage0/src, which
            // is what a round IS; the receipt names the tree and every installed path.
            "--regen-round-cost" => {
                regen_round_cost_mode = true;
            }
            // THE HEAL ROUND'S BOOTSTRAP AUTHORITY (DESIGN §5, RFM heal_round_one_bootstraps_
            // from_the_mirror_the_change_must_repair / PR #14963): round 1's seed compiles from
            // the BASE COMMIT's committed stage0 mirror, never from the mirror the PR's own .dag
            // changes exist to repair. The flag carries NO value — the commit arrives as the
            // HEAL_BASE_MIRROR_COMMIT environment value, named by `gunbc.heal_workflow`
            // `heal_round_base_mirror_env` (env, not argv, because it is pull-request-controlled
            // text; same shape as GUNBC_AFFECTED_SCOPE). It is a 40-hex COMMIT, not a ref:
            // HEAL_BASE_REF belongs to the candidate step (gunbc.heal_candidate) and carries a
            // branch NAME for a different purpose, so the round must not read it. Without the
            // flag the round is the probe/CI shape:
            // the working tree's own committed mirror, byte for byte what it was. There is no
            // "try the tree, fall back on failure" path — the flag is set unconditionally by
            // the heal round step and its absence is a different CONTEXT, not a second attempt.
            "--base-mirror" => {
                base_mirror_mode = true;
            }
            // THE AFFECTED SET OF THE FLOOR'S DIFF RANGE: which committed mirrors can change
            // for the .dag modules this edit touched, as `gunbc.regen_affected_set` bounds it.
            // Reports; it installs nothing. An edited path the tree cannot name refuses.
            "--regen-affected-set" => {
                regen_affected_set_mode = true;
            }
            // CONSUME the bound rather than report it: the round adjudicates, writes and digests
            // only the mirrors `--regen-affected-set` names for this edit, and REFUSES when that
            // bound cannot locate an edited path. Opt-in and narrowing: without it the round is
            // whole-population, which is what the required CI phase runs and what establishes
            // the fixed point a scoped round starts from (v2.workflow.required_regen
            // RegenEmissionScope).
            "--regen-affected-scope" => {
                regen_affected_scope = true;
            }
            "--regen-candidate-dir" => {
                i += 1;
                regen_candidate_dir = require_value(&args, i, "--regen-candidate-dir")?;
            }
            "--regen-receipt" => {
                i += 1;
                regen_receipt_path = require_value(&args, i, "--regen-receipt")?;
            }
            "--required-ci" => {
                required_admitted_set_mode = true;
            }
            // Accepted so every command form this repository cites parses. The lane selector
            // distinguished the two withdrawn measurement lanes (build vs witnesses) that this
            // mode does not reproduce: the denominator here is the ingestion's admitted set,
            // which no lane choice changes. Stored and ignored; an unknown value is a
            // non-parsing argument (rc 2), which is what rc 2 is for.
            "--required-lane" => {
                i += 1;
                let lane = require_value(&args, i, "--required-lane")?;
                if lane != "build" && lane != "witnesses" {
                    eprintln!("claim_executor: unknown --required-lane value: {}", lane);
                    return Err(ExitCode::from(2));
                }
            }
            other => {
                eprintln!("claim_executor: unknown argument: {}", other);
                return Err(ExitCode::from(2));
            }
        }
        i += 1;
    }

    // Build-artifact verification (no plan run): the floor's bootstrap `cargo build` is followed by
    // this check so a `successful` build that nonetheless produced a missing/zero-byte binary
    // (sccache serving a truncated/empty cached artifact — DESIGN §5 fail-open) fails CLOSED before
    // the floor runs. The LOGIC lives here (the floor binary) instead of inline shell in ci.yml; the
    // declared artifact paths are still authored by the .dag spec and passed as positional args.
    // Short-circuits before the plan-arg requirements so it needs no `--plan-entry`.
    if verify_artifacts_mode {
        return verify_build_artifacts(&verify_artifacts);
    }

    if let Some(path) = write_checker_inputs {
        return match v1_compiler::cli_run::write_checker_inputs_record(std::path::Path::new(&path))
        {
            Ok(n) => {
                eprintln!("claim_executor: wrote {n} checker inputs to {path}");
                Ok(ExitCode::SUCCESS)
            }
            Err(e) => {
                eprintln!("::error::write-checker-inputs refused: {e}");
                Err(ExitCode::from(1))
            }
        };
    }

    // ORDERED AHEAD OF THE SOURCE-ROOT REQUIREMENT, for the reason `--verify-build-artifacts`
    // is: this mode takes NO roots. It renders from the emitted
    // authority carrier and reads only the files it is adjudicating, so the generic guard below
    // would refuse the only invocation it has -- a mode that could never have run once.
    // MEASURED, NOT REASONED -- the first remote run of this entry point exited 2 with
    // `provide at least one --source-root` before the branch was moved here.
    //
    // THE PRODUCER, AS ITS OWN ENTRY POINT. `--write` installs; without it the run reports and
    // changes nothing, so the read-only form is safe to run anywhere and the `written` list is
    // empty BY CONSTRUCTION rather than by a caller remembering not to ask.
    if emit_partition_crates_mode {
        let outcome =
            v1_compiler::cli_run::run_partition_crate_boundary(emit_partition_crates_write);
        match &outcome {
            v1_compiler::cli_run::PartitionCrateBoundaryOutcome::CarrierRefused { cause } => {
                eprintln!("emit-partition-crates: CarrierRefused cause={cause}");
                return Err(ExitCode::from(1));
            }
            v1_compiler::cli_run::PartitionCrateBoundaryOutcome::Rendered { files, written } => {
                for file in files {
                    eprintln!(
                        "emit-partition-crates: {} {}",
                        file.disposition().name(),
                        file.path
                    );
                }
                eprintln!(
                    "emit-partition-crates: rendered={} written={} mode={}",
                    files.len(),
                    written.len(),
                    if emit_partition_crates_write {
                        "write"
                    } else {
                        "read-only"
                    }
                );
                // A READ-ONLY RUN THAT FOUND DRIFT EXITS NONZERO; the write form is what closes
                // it. Exiting zero here would make the producer's own entry point disagree with
                // the phase that names it as the remedy.
                let unresolved = outcome.divergent().len();
                // THE TYPED REPORT, on stdout: callers read the drifted population from here and
                // never from the diagnostic lines above.
                println!(
                    "{}",
                    serde_json::json!({
                        "schema": "gunbc.partition_boundary_report.v1",
                        "divergent": outcome
                            .divergent()
                            .iter()
                            .map(|f| f.path.clone())
                            .collect::<Vec<_>>(),
                    })
                );
                return if emit_partition_crates_write || unresolved == 0 {
                    Ok(ExitCode::SUCCESS)
                } else {
                    Err(ExitCode::from(1))
                };
            }
        }
    }

    if source_roots.is_empty() {
        eprintln!("claim_executor: provide at least one --source-root");
        return Err(ExitCode::from(2));
    }

    // `--required-ci`: print the module identities the source-root ingestion admits
    // (`source_root_ingest_module_identities_for_ci`). Read-only. Refusal of the ingestion is rc 1
    // with the cause, never rc 2 -- rc 2 is reserved for arguments this binary does not parse.
    if required_admitted_set_mode {
        return match v1_compiler::cli_run::source_root_ingest_module_identities_for_ci(
            &source_roots,
        ) {
            Ok(identities) => {
                for identity in &identities {
                    println!("admitted-module-identities: {identity}");
                }
                eprintln!(
                    "claim_executor --required-ci: admitted {} module identities",
                    identities.len()
                );
                Ok(ExitCode::SUCCESS)
            }
            Err(cause) => {
                eprintln!("claim_executor --required-ci: ingestion refused: {cause}");
                Err(ExitCode::FAILURE)
            }
        };
    }

    let _phase_profile = PhaseProfile::install_from_env();

    if required_emit_compile_mode {
        let roots = if source_roots.is_empty() {
            v1_compiler::cli_run::witness_layer_roots()
        } else {
            source_roots.clone()
        };
        let probe_root = match v1_compiler::cli_run::local_emit_compile_probe_root() {
            Ok(root) => root,
            Err(e) => {
                eprintln!("required-emit-compile: probe root not created: {e}");
                return Err(ExitCode::from(1));
            }
        };
        match v1_compiler::cli_run::run_required_emit_compile(&roots, &probe_root) {
            Ok(outcomes) => {
                let mut not_passed = 0usize;
                for outcome in &outcomes {
                    eprintln!(
                        "required-emit-compile: {}",
                        v1_compiler::cli_run::emit_compile_outcome_summary(outcome)
                    );
                    if !v1_compiler::cli_run::emit_compile_outcome_passed(outcome) {
                        not_passed += 1;
                        if let v1_compiler::cli_run::EmitCompileOutcome::Measured {
                            baseline, ..
                        } = outcome
                        {
                            for line in
                                v1_compiler::cli_run::cargo_verdict_stderr_tail(baseline).lines()
                            {
                                eprintln!("required-emit-compile: cargo| {line}");
                            }
                        }
                    }
                }
                let (report, retention_error) = v1_compiler::cli_run::emit_compile_report(
                    &outcomes,
                    &roots,
                    &probe_root,
                    "required-emit-compile:",
                );
                for line in report {
                    eprintln!("{line}");
                }
                // The report printed this root and the retention file inside it, so it is kept
                // for the reader rather than removed with the value.
                let _ = probe_root.retain();
                return if not_passed == 0 && retention_error.is_none() {
                    Ok(ExitCode::SUCCESS)
                } else {
                    Err(ExitCode::from(1))
                };
            }
            Err(e) => {
                eprintln!("required-emit-compile: roster refused: {e}");
                return Err(ExitCode::from(1));
            }
        }
    }

    if v2_native_route_mode {
        let roots = if source_roots.is_empty() {
            v1_compiler::cli_run::witness_layer_roots()
        } else {
            source_roots.clone()
        };
        eprintln!(
            "v2-native-route: emitted-native compiler executes the derived native-lane universe (operator-invoked; not a required lane)"
        );
        // THE LANE ADJUDICATES ITS WHOLE UNIVERSE, so it passes no pattern: the emitted binary
        // adjudicates by `gunbc.native_route_universe` `native_route_default_pattern`, read inside
        // the compiler where it is declared, and this caller holds no spelling of it. A narrower
        // pattern reaches the same runner through `gunbc test <operand>`.
        return match v1_compiler::cli_run::run_required_v2_native(&roots, None) {
            v1_compiler::cli_run::NativeRouteOutcome::LaneQualificationHeld { .. } => {
                Ok(ExitCode::SUCCESS)
            }
            v1_compiler::cli_run::NativeRouteOutcome::LaneQualificationRefused {
                summary, ..
            } => {
                eprintln!("v2-native-route: refused: LaneQualificationRefused — {summary}");
                Err(ExitCode::from(1))
            }
            v1_compiler::cli_run::NativeRouteOutcome::Unreached { cause } => {
                eprintln!("v2-native-route: refused: {cause}");
                Err(ExitCode::from(1))
            }
        };
    }

    if required_v2_emission_mode {
        let roots = if source_roots.is_empty() {
            v1_compiler::cli_run::witness_layer_roots()
        } else {
            source_roots.clone()
        };
        return match v1_compiler::cli_run::run_required_v2_emission(&roots) {
            Ok(runs) => {
                let mut not_completed = 0usize;
                for run in &runs {
                    // ONE SELF-DESCRIBING LINE. The disposition is ON the line, so a run
                    // that never reached the compiler cannot be read as a clean emission
                    // of nothing by anything that reads one line at a time.
                    eprintln!("{}", run.measurement_line("required-v2-emission"));
                    match &run.disposition {
                        v1_compiler::cli_run::CompileDisposition::Completed { .. } => {}
                        v1_compiler::cli_run::CompileDisposition::Refused { phase, cause } => {
                            not_completed += 1;
                            eprintln!(
                                "required-v2-emission: EmissionRefused {} phase={phase} cause={cause}",
                                run.subject.receipt()
                            );
                        }
                        v1_compiler::cli_run::CompileDisposition::NotExecuted {
                            earlier_phase,
                            cause,
                        } => {
                            not_completed += 1;
                            eprintln!(
                                "required-v2-emission: EmissionNotExecuted {} earlier_phase={earlier_phase} cause={cause}",
                                run.subject.receipt()
                            );
                        }
                    }
                }
                eprintln!(
                    "required-v2-emission: entries={} not_completed={}",
                    runs.len(),
                    not_completed
                );
                if not_completed == 0 {
                    Ok(ExitCode::SUCCESS)
                } else {
                    Err(ExitCode::from(1))
                }
            }
            // The roster itself being unreadable is reported as itself: no entry ran, so
            // there is no per-entry disposition to render.
            Err(e) => {
                eprintln!(
                    "required-v2-emission: EmissionNotExecuted earlier_phase=roster cause={e}"
                );
                Err(ExitCode::from(1))
            }
        };
    }

    // THE CITED-SYMBOL CENSUS IS GONE FROM HERE, AND ITS SUBJECT MOVED RATHER THAN LAPSING.
    //
    // `--required-cited-symbol` ran `v2.lens.cited_symbol_resolution` over the corpus-wide
    // `decl_facts` + `module_declaration_facts` walks, answering each authored reference by a
    // LINEAR SCAN of a flat list of every declaration in the repository. DESIGN §3's rung-drop
    // row names exactly that shape as the thing to stop doing: the wall belongs "checked at
    // ingestion, on the module whose source carries the citation, from that module's own text,
    // rather than reconstructed corpus-wide by a second job".
    //
    // It is now the `parse` phase above, over the per-module declaration index the sweep builds
    // from the parse it was already performing. The mode is deleted rather than left standing as
    // a second route to one question (§3, single authority), and the replacement is STRICTLY
    // WIDER, which is the test §3's replacement doctrine sets: it enrolls every authored
    // `DeclarationRef` in the corpus rather than the five carriers the lens's population named,
    // and it indexes test modules, which `decl_facts` deliberately did not — so the
    // outside-index disposition those exclusions forced is not needed at all.

    // THE HEADS READING'S OWN INSTRUMENT MOVED TO `gunbc test`, AND IT MOVED ATOMICALLY.
    //
    // What stood here was a `--heads-reading-differential` mode: the same producer, reached by a
    // flag on this binary. It is DELETED in the change that makes the instrument addressable as
    // `//gunbc/instruments:heads-reading-differential`, rather than left standing beside it. Two
    // routes to one producer is the fork DESIGN section 3 forbids, and the interval in which both
    // work is exactly when a caller learns the old one and keeps it.
    //
    // The reason it was never enrolled in the required run is unchanged and is now recorded on the
    // target instead: reading the corpus TWICE is precisely the cost the heads reading exists to
    // remove, so paying it on every push would spend more than the repair saves. The instrument is
    // outside `//:required` by construction there -- `gunbc.discovery_census` derives that
    // aggregate from discovered witness sites, and no fold feeds the instrument population into it.

    if regen_affected_set_mode {
        return match v1_compiler::cli_run::run_regen_affected_set(&source_roots) {
            Ok(outcome) => {
                eprint!("{}", outcome.rendered);
                if outcome.arm == "EditedSetUnlocatable" {
                    Err(ExitCode::from(1))
                } else {
                    Ok(ExitCode::SUCCESS)
                }
            }
            Err(e) => {
                eprintln!("regen-affected-set: refused: {e}");
                Err(ExitCode::from(1))
            }
        };
    }

    if regen_round_cost_mode {
        return match v1_compiler::cli_run::run_regen_round_cost(
            &regen_candidate_dir,
            &regen_receipt_path,
            &source_roots,
            regen_affected_scope,
            base_mirror_mode,
        ) {
            Ok(outcome) => {
                eprint!("{}", outcome.rendered);
                eprintln!(
                    "regen-round-cost: receipt={}",
                    outcome.receipt_path.display()
                );
                for failure in &outcome.round_failures {
                    eprintln!("regen-round-cost: FAIL {failure}");
                }
                if outcome.round_failures.is_empty() {
                    Ok(ExitCode::SUCCESS)
                } else {
                    Err(ExitCode::from(1))
                }
            }
            Err(e) => {
                eprintln!("regen-round-cost: refused: {e}");
                Err(ExitCode::from(1))
            }
        };
    }

    if required_regen_mode {
        let regen = if regen_affected_scope {
            v1_compiler::cli_run::run_required_regen_scoped(
                &regen_candidate_dir,
                &regen_receipt_path,
                &source_roots,
            )
        } else {
            v1_compiler::cli_run::run_required_regen(&regen_candidate_dir, &regen_receipt_path)
        };
        return match regen {
            Ok(outcome) => {
                // Both values here ARE measured by this pass, so they print unqualified. Read
                // through accessors rather than by matching the variant: the
                // `required_regen_host` module is private to `cli_run`, so the type is usable here
                // but not nameable. The accessors return Option because the sibling variant does
                // not measure these fields; a `None` on this path would mean the first pass built
                // the wrong variant, so it prints `unmeasured` rather than defaulting to a
                // plausible-looking value.
                let fge = outcome
                    .receipt
                    .first_generation_equal()
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "unmeasured".to_string());
                let candidate = outcome
                    .receipt
                    .candidate_artifact()
                    .unwrap_or("unmeasured")
                    .to_string();
                eprintln!("required-regen: first_generation_equal={fge} candidate={candidate}");
                for failure in &outcome.failures {
                    eprintln!("required-regen: FAIL {failure}");
                }
                if outcome.failures.is_empty() {
                    Ok(ExitCode::SUCCESS)
                } else {
                    Err(ExitCode::from(1))
                }
            }
            Err(e) => {
                eprintln!("required-regen: refused: {e}");
                Err(ExitCode::from(1))
            }
        };
    }

    // THE PLAN/WALK SURFACE IS DELETED (2026-08-25). Every mode above returns before this
    // point, and what stood here -- the --plan-entry walk, the batch executor, the
    // coordinator/worker protocol, the perturb re-walk and their terminal reporting -- was
    // reachable ONLY through --plan-entry. Nothing has supplied it since the 2026-08-15 floor
    // cut deleted the composed floor it drove, and `src/v2/workflow/ci_floor_plan.dag`, which
    // --plan-function named, does not exist.
    //
    // This is a REACHABILITY deletion, not an occupancy one, and the distinction is the one
    // DESIGN names: the arms are not quiet guards that happen to be empty today, their
    // governing MECHANISM was removed, so no input any caller can author reaches them.
    //
    // The refusal is typed and names the live modes rather than exiting silently: an argv this
    // binary no longer understands must stop the line, never fall through to a default.
    eprintln!(
        "claim_executor: CLAIM-EXECUTOR REFUSAL cause=NoRunnableMode — live modes are \
         --verify-build-artifacts and the required-* producers. The plan/walk surface (--plan-entry, \
         --plan-function, --floor-worker-role, --scoped-batch-id) was deleted 2026-08-25 with \
         the machinery it drove."
    );
    Err(ExitCode::from(2))
}

fn main() -> ExitCode {
    // NO SECOND ARGV READER. What stood here read std::env::args() directly and dispatched the
    // floor coordinator BEFORE run() parsed anything, so deleting --plan-function from run()'s
    // parser left the coordinator fully reachable -- and reachable through a path that removed
    // receipt files and spawned workers before any refusal could fire. Review 55875 caught it,
    // and the state it caught was worse than the one before this branch: --plan-function answered
    // "unknown argument" for every value EXCEPT the one that ran the whole coordinator.
    //
    // The lesson is the cut's, not the coordinator's: a flag is not deleted when one of two
    // parsers stops reading it. run() is now the only thing that reads argv.
    // The materialization demand receipt is mandatory on the floor: enable the
    // interpreter's recompute-trace ledger unless the environment already set
    // it. An explicit =0 zeroes the receipt, and the derived ci.yml gate fails
    // closed on keyed_calls=0 — disabling is loud, never silent.
    if std::env::var_os("GUNBC_RECOMPUTE_TRACE").is_none() {
        std::env::set_var("GUNBC_RECOMPUTE_TRACE", "1");
    }
    let code = match run() {
        Ok(code) => code,
        Err(code) => code,
    };
    // THE CORPUS CACHES ARE RELEASED WITH THE PROCESS, NOT FREED ONE NODE AT A TIME. See
    // `release_process_caches_at_exit` for what it forgets, why that is safe, and the residue it
    // leaves measured rather than assumed.
    v1_compiler::cli_run::release_process_caches_at_exit();
    code
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo_root_from_manifest() -> PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .expect("repo root from CARGO_MANIFEST_DIR")
    }

    fn dag_source_from_repo(rel: &str) -> String {
        let path = repo_root_from_manifest().join(rel);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    fn dag_record_string_field(source: &str, data_name: &str, field: &str) -> String {
        let marker = format!("data {data_name}:");
        let start = source
            .find(&marker)
            .unwrap_or_else(|| panic!("data row {data_name} not found in authority source"));
        let slice = &source[start..];
        let field_marker = format!("{field}: \"");
        let fstart = slice
            .find(&field_marker)
            .unwrap_or_else(|| panic!("field {field} not found on data row {data_name}"))
            + field_marker.len();
        let rest = &slice[fstart..];
        let end = rest
            .find('"')
            .expect("unterminated string field in authority source");
        rest[..end].to_string()
    }

    // Test-only mirror of gunbc.ci.ci_materialization obligation subject rows. Production
    // parses transported obligations from WalkPlan.finalization; this module is the
    // checkable drift receipt (review 48261 / 48570).
    const TEST_COMPILE_ANCHOR_OBLIGATION_ENTRY: &str =
        "dag/gunbc/instruments/floor_effect_gate_witness.dag";
    const TEST_COMPILE_ANCHOR_OBLIGATION_FUNCTION: &str = "dag_compile_clean_gate_passes";
    const TEST_NATIVE_BUNDLE_OBLIGATION_ENTRY: &str =
        "src/v2/test/claim/execution/native_selected_witness_bundle_production.dag";
    const TEST_NATIVE_BUNDLE_OBLIGATION_FUNCTION: &str = "native_selected_logic_production_spec";

    /// Test-only obligation-subject literals must track gunbc.ci.ci_materialization authority rows.
    #[test]
    fn floor_resolve_obligation_seed_constants_match_dag_authority() {
        let ci_materialization = dag_source_from_repo("dag/gunbc/ci/ci_materialization.dag");
        assert_eq!(
            dag_record_string_field(
                &ci_materialization,
                "compile_anchor_obligation_subject",
                "entry"
            ),
            TEST_COMPILE_ANCHOR_OBLIGATION_ENTRY,
        );
        assert_eq!(
            dag_record_string_field(
                &ci_materialization,
                "compile_anchor_obligation_subject",
                "function"
            ),
            TEST_COMPILE_ANCHOR_OBLIGATION_FUNCTION,
        );
        assert_eq!(
            dag_record_string_field(
                &ci_materialization,
                "native_bundle_obligation_subject",
                "entry"
            ),
            TEST_NATIVE_BUNDLE_OBLIGATION_ENTRY,
        );
        assert_eq!(
            dag_record_string_field(
                &ci_materialization,
                "native_bundle_obligation_subject",
                "function"
            ),
            TEST_NATIVE_BUNDLE_OBLIGATION_FUNCTION,
        );
    }

    // --- floor-finalization DISPOSITION visibility (the fix this PR ships) -----------
    //
    // Three previously-silent cases, each with its own discriminating control so the
    // three cannot collapse back into one indistinguishable "nothing printed" bucket:
    //   A — finalization absent by construction (scoped floor worker)
    //   B — finalization absent incidentally (walk never reached the call)
    //   C — finalization HELD on an otherwise-failed floor (verdict must not vanish)

    // ENROLLMENT MAY ONLY HOLD A CLAIM THAT WAS DECIDED. Found on gunbc#8642 by the side
    // thread: the first version of `from_outcome` matched `_ if enrolled => KnownRed` above
    // every failing arm, so an enrolled row that was INTERRUPTED at a budget, or whose host
    // tool could not be resolved, printed KNOWN-RED — owned semantic debt, for two states that
    // carry no semantic verdict at all.
    //
    // RED: restoring the blanket `_ if enrolled` arm turns the two interruption cases into
    // KnownRed and fails this test. The `Fail` case is the positive control — without it, a
    // `from_outcome` that never returned KnownRed at all would also pass.
    #[test]
    fn enrollment_holds_only_semantic_failures_not_undecided_claims() {
        use v1_compiler::cli_run::{BudgetKind, CiWitnessVerdict, ClaimOutcome};

        let interrupted = ClaimOutcome::BudgetInterrupted {
            elapsed_at_least_ms: 5_000,
            budget_ms: 5_000,
            kind: BudgetKind::Cpu,
        };
        let missing_tool = ClaimOutcome::HostToolUnresolved {
            name: "git".to_string(),
            probed: vec!["/usr/bin/git".to_string()],
        };

        assert_eq!(
            CiWitnessVerdict::from_outcome(&ClaimOutcome::Fail, true),
            CiWitnessVerdict::KnownRed,
            "positive control: an enrolled SEMANTIC failure is the one thing enrollment holds"
        );
        assert_eq!(
            CiWitnessVerdict::from_outcome(&interrupted, true),
            CiWitnessVerdict::BudgetRefused,
            "an interruption is a lower bound on cost, not a verdict enrollment can hold"
        );
        assert_eq!(
            CiWitnessVerdict::from_outcome(&missing_tool, true),
            CiWitnessVerdict::HostToolUnresolved,
            "an unresolved host tool is an infra gap, not owned semantic debt"
        );
        assert_eq!(
            CiWitnessVerdict::from_outcome(&interrupted, false),
            CiWitnessVerdict::from_outcome(&interrupted, true),
            "enrollment must not change the token for an undecided claim, either direction"
        );
        assert_eq!(
            CiWitnessVerdict::from_outcome(&ClaimOutcome::Pass, true),
            CiWitnessVerdict::Passed,
            "an enrolled row that passes still prints PASSED; the roster staleness is the \
             ledger's to report, not a token the console invents"
        );
    }

    // --- build-artifact verification teeth (DESIGN §5 fail-open guard) ---

    fn write_exec(dir: &std::path::Path, name: &str, bytes: &[u8]) -> String {
        use std::os::unix::fs::PermissionsExt;
        let p = dir.join(name);
        std::fs::write(&p, bytes).expect("write artifact");
        let mut perms = std::fs::metadata(&p).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&p, perms).expect("chmod");
        p.to_string_lossy().into_owned()
    }

    #[test]
    fn verify_build_artifacts_accepts_nonempty_executables() {
        let dir = std::env::temp_dir().join(format!("cev-ok-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = write_exec(&dir, "claim_executor", b"\x7fELF-not-really-but-nonempty");
        let b = write_exec(&dir, "gunbc", b"binary");
        let r = verify_build_artifacts(&[a, b]);
        std::fs::remove_dir_all(&dir).ok();
        assert!(
            matches!(r, Ok(c) if c == ExitCode::SUCCESS),
            "real bins pass"
        );
    }

    #[test]
    fn verify_build_artifacts_reds_on_zero_byte() {
        let dir = std::env::temp_dir().join(format!("cev-zero-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = write_exec(&dir, "claim_executor", b"ok");
        let b = write_exec(&dir, "gunbc", b""); // sccache served a truncated/empty cached artifact
        let r = verify_build_artifacts(&[a, b]);
        std::fs::remove_dir_all(&dir).ok();
        assert!(
            matches!(r, Err(c) if c == ExitCode::from(1)),
            "zero-byte artifact fails closed"
        );
    }

    #[test]
    fn verify_build_artifacts_reds_on_missing() {
        let missing = std::env::temp_dir()
            .join(format!("cev-missing-{}/gunbc", std::process::id()))
            .to_string_lossy()
            .into_owned();
        let r = verify_build_artifacts(&[missing]);
        assert!(
            matches!(r, Err(c) if c == ExitCode::from(1)),
            "absent artifact fails closed"
        );
    }

    #[test]
    fn verify_build_artifacts_reds_on_empty_arglist() {
        let r = verify_build_artifacts(&[]);
        assert!(
            matches!(r, Err(c) if c == ExitCode::from(2)),
            "no declared artifacts is a usage error, fail closed"
        );
    }

    // ---- walk-attempt identity ----
    //
    // The refusals below are the point of the feature, so each is asserted as a REFUSAL,
    // not merely as "not equal to the good value". The positive controls exist so the
    // negatives are discriminating: a `compose_walk_attempt_id` that refused everything
    // would pass every Err assertion and fail these.
}

// ---------------------------------------------------------------------------
// BEHAVIORAL RECEIPT — selection and corpus-domain derivation
// ---------------------------------------------------------------------------
//
// WHAT THIS ANSWERS THAT NOTHING ELSE DOES. `--required-regen` proves the committed mirrors
// equal what the authority emits. It never COMPILES the emitted candidate, let alone runs it: the regen host spawns exactly
// rustfmt, rustfmt and git. So the whole promotion story rests on bytes — and DESIGN §7 says in
// as many words that a byte-identical fixed point is NOT the goal, because matching bytes forces
