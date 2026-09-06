// THE HOST REALIZATION OF `gunbc test <label>`, AND IT IS A HAND MIRROR OF A `.dag` AUTHORITY.
//
// The model is `gunbc.target_invocation` (generic route, termination vocabulary),
// `gunbc.instrument_targets` (live target and binding rows, the differential's classifier and
// rendering) and `extdeps.bazel.label` (the label grammar mirrored here). None is in the v1
// seed's emitted closure — `src/gunbc_cli_dispatch_surface.rs` is the only `gunbc.*` mirror the
// emitter produces — so this file is hand-written beside the carrier, as `required_regen_host.rs`
// mirrors `v2.workflow.required_regen`. The seam is therefore MITIGATABLE, not structurally
// guaranteed: the two can drift until the seam is emitted rather than authored. The obligation
// is enrolled in `gunbc.target_invocation_seed_growth`.
//
// WHAT IS AND IS NOT GENERIC HERE. One route: argv operand -> admit label -> build the registry
// -> exact lookup -> invoke the bound producer -> render its native standing. No per-instrument
// arm on that route, and none may be added; a second instrument is a row in `instrument_registry`
// plus one `Producer` arm in `run_producer` — the peripheral realization dispatch DESIGN section 3
// keeps out of the interface. Deliberately NOT here: any consultation of `//:required` aggregate
// policy or the Blaze status export — both refuse instrument producers by design.

use crate::cli_run;
use crate::v1_interpreter;

/// The label subset `extdeps.bazel.label` admits, and its refusal vocabulary, mirrored.
///
/// Every family that authority excludes is excluded here with the SAME named cause, because the
/// remedies differ: a pattern means "name one target", a relative label "make it absolute". Main
/// repository only, so no field can name another repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelRefusal {
    RepositoryQualifiedLabel(String),
    MissingRepositoryRootPrefix(String),
    MultipleColonSeparators(String),
    EmptyTargetName(String),
    EmptyPackageSegment(String),
    DotSegment(String),
    TargetPattern(String),
    TargetNameContainsSlash(String),
}

/// Rendering is a FREE FUNCTION, not an inherent method: `std.decl_ref` `DeclarationRef` names a
/// declaration or a named field but has no spelling for an impl method, so an inherent method
/// would be uncitable in the seed-growth roster that must enumerate every item this file adds.
fn label_refusal_rendered(cause: &LabelRefusal) -> String {
    {
        match cause {
            LabelRefusal::RepositoryQualifiedLabel(t) => {
                format!("repository-qualified labels are outside the admitted subset: {t}")
            }
            LabelRefusal::MissingRepositoryRootPrefix(t) => {
                format!("label is not absolute (expected a leading `//`): {t}")
            }
            LabelRefusal::MultipleColonSeparators(t) => {
                format!("label carries more than one `:` separator: {t}")
            }
            LabelRefusal::EmptyTargetName(t) => format!("label names no target: {t}"),
            LabelRefusal::EmptyPackageSegment(t) => {
                format!("label carries an empty package segment: {t}")
            }
            LabelRefusal::DotSegment(s) => format!("label carries a dot package segment: {s}"),
            LabelRefusal::TargetPattern(p) => format!(
                "`{p}` is a target PATTERN and denotes a set; `gunbc test` names exactly one target"
            ),
            LabelRefusal::TargetNameContainsSlash(n) => {
                format!("target name contains `/`, which this subset does not admit: {n}")
            }
        }
    }
}

/// A label in the main repository: a package (the root package is its own state, the empty
/// segment list) and a target name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub package_segments: Vec<String>,
    pub target: String,
}

/// The structural inverse of `parse_label`, always the explicit colon form. This is the registry
/// key, DERIVED on every call rather than stored, so a target cannot acquire a second identity.
pub fn render_label(l: &Label) -> String {
    format!("//{}:{}", l.package_segments.join("/"), l.target)
}

fn is_pattern_target_name(s: &str) -> bool {
    matches!(s, "all" | "*" | "..." | "all-targets")
}

fn parse_target_name(text: &str) -> Result<String, LabelRefusal> {
    if text.is_empty() {
        Err(LabelRefusal::EmptyTargetName(text.to_string()))
    } else if is_pattern_target_name(text) {
        Err(LabelRefusal::TargetPattern(text.to_string()))
    } else if text.contains('/') {
        Err(LabelRefusal::TargetNameContainsSlash(text.to_string()))
    } else {
        Ok(text.to_string())
    }
}

fn parse_package_segments(package_text: &str) -> Result<Vec<String>, LabelRefusal> {
    let raw: Vec<String> = package_text.split('/').map(|s| s.to_string()).collect();
    // Leading `/`, trailing `/` and embedded `//` all yield an empty segment: one arm, three
    // malformed shapes.
    if raw.iter().any(|s| s.is_empty()) {
        return Err(LabelRefusal::EmptyPackageSegment(package_text.to_string()));
    }
    if let Some(dotted) = raw.iter().find(|s| s.as_str() == "." || s.as_str() == "..") {
        return Err(LabelRefusal::DotSegment(dotted.clone()));
    }
    if let Some(patterned) = raw.iter().find(|s| s.as_str() == "...") {
        return Err(LabelRefusal::TargetPattern(patterned.clone()));
    }
    Ok(raw)
}

/// `//my/app/lib` IS `//my/app/lib:lib`: the shorthand is folded at parse time.
pub fn parse_label(text: &str) -> Result<Label, LabelRefusal> {
    if text.starts_with('@') {
        return Err(LabelRefusal::RepositoryQualifiedLabel(text.to_string()));
    }
    let Some(body) = text.strip_prefix("//") else {
        return Err(LabelRefusal::MissingRepositoryRootPrefix(text.to_string()));
    };
    let parts: Vec<&str> = body.split(':').collect();
    if parts.len() > 2 {
        return Err(LabelRefusal::MultipleColonSeparators(text.to_string()));
    }
    let package_text = parts.first().copied().unwrap_or("");
    if package_text.is_empty() {
        let target_text = if parts.len() == 2 { parts[1] } else { "" };
        return Ok(Label {
            package_segments: Vec::new(),
            target: parse_target_name(target_text)?,
        });
    }
    let segments = parse_package_segments(package_text)?;
    let target_text = if parts.len() == 2 {
        parts[1].to_string()
    } else {
        segments.last().cloned().unwrap_or_default()
    };
    Ok(Label {
        package_segments: segments,
        target: parse_target_name(&target_text)?,
    })
}

/// `gunbc.target_binding` `TargetProducer`, narrowed to the members this seam realizes today.
/// Adding one is a row in `instrument_registry` and an arm here; it is not a new route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetProducer {
    HeadsReadingDifferential,
    BehavioralReceiptPlan,
    BehavioralReceiptCensus,
    BehavioralReceiptSelftest,
}

/// `gunbc.instrument_targets` `instrument_targets` / `instrument_bindings`, as the pairs the
/// index is built from. All four modeled instruments are rows on the same generic seam.
/// `gunbc.instrument_targets` `heads_reading_differential_source_roots`. The subject is the
/// instrument's own fact, not a CLI option, so an invocation cannot quietly measure another corpus.
fn heads_reading_differential_source_roots() -> Vec<String> {
    vec!["dag".to_string(), "src/v2".to_string()]
}

fn instrument_registry() -> Vec<(Label, TargetProducer)> {
    vec![
        (
            Label {
                package_segments: vec!["gunbc".to_string(), "instruments".to_string()],
                target: "heads-reading-differential".to_string(),
            },
            TargetProducer::HeadsReadingDifferential,
        ),
        (
            instrument_label("behavioral-receipt-plan"),
            TargetProducer::BehavioralReceiptPlan,
        ),
        (
            instrument_label("behavioral-receipt-census"),
            TargetProducer::BehavioralReceiptCensus,
        ),
        (
            instrument_label("behavioral-receipt-selftest"),
            TargetProducer::BehavioralReceiptSelftest,
        ),
    ]
}

fn instrument_label(target: &str) -> Label {
    Label {
        package_segments: vec!["gunbc".to_string(), "instruments".to_string()],
        target: target.to_string(),
    }
}

/// Extract the file-path set from all three `FloorDiffEdits` buckets:
/// `touched_entry_files`, `overlapping_data_items`, and `edited_test_fns`.
/// This is the single authority for what files a diff touched — used by
/// `test_affected_select` and tested by `data_row_only_diff_selects...`.
fn touched_paths_from_diff_edits(edits: &cli_run::FloorDiffEdits) -> Vec<String> {
    let mut set: std::collections::HashSet<String> =
        edits.touched_entry_files.iter().cloned().collect();
    for (file, _) in &edits.overlapping_data_items {
        set.insert(file.clone());
    }
    for (file, _) in &edits.edited_test_fns {
        set.insert(file.clone());
    }
    set.into_iter().collect()
}

/// `gunbc test --affected-select`: run every discovery-roster witness whose entry is
/// touched by the working-tree diff (directly or transitively through the module-graph
/// import closure), not every witness in the corpus.
///
/// EXECUTION PATH: discover the full roster, compute the affected set via the existing
/// `entry_file_touched_via_import_closure` authority (the §3 consolidated survivor),
/// run only affected rows through the existing witness-runner (`run_discovery_rows`),
/// and return a single aggregate outcome. This is NOT an instrument target — it uses
/// the same witness execution infrastructure as the required floor.
///
/// Receipt format:
///   SELECTED //test:a
///   SELECTED //test:c
///   RUN //test:a -> PASS
///   RUN //test:c -> PASS
///   N selected, M executed, K passed
///
/// Refusal: if the affected-set computation refuses for any entry (EntryOutsideModuleGraphFacts
/// or ReferenceEdgesUnaccounted), the batch is refused — never widened to run-all.
pub fn test_affected_select() -> InvocationOutcome {
    let source_roots = cli_run::default_source_roots();
    // The source roots exist for the purpose of building the module-graph index.
    // This matches the same roots the floor runner and the `heads_reading_differential`
    // instrument use.
    let missing: Vec<&String> = source_roots
        .iter()
        .filter(|r| !std::path::Path::new(r.as_str()).exists())
        .collect();
    if !missing.is_empty() {
        let named: Vec<String> = missing.into_iter().cloned().collect();
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!(
                "affected-select: REFUSED (source root absent): {}",
                named.join(", ")
            ),
        };
    }

    // Step 1: build the module-graph index (facts, adjacency, declared paths).
    let index = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cli_run::build_multi_entry_index(&source_roots)
    })) {
        Ok(idx) => idx,
        Err(_) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: "affected-select: REFUSED — module-graph index build panicked".to_string(),
            };
        }
    };
    let facts = index.module_graph_facts();

    // Step 2: observe the working-tree diff.
    let diff_text = match std::process::Command::new("git")
        .args(["diff", "HEAD"])
        .output()
    {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).to_string()
        }
        Ok(_) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: "affected-select: REFUSED — git diff HEAD exited with a non-zero status"
                    .to_string(),
            };
        }
        Err(e) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!("affected-select: REFUSED — git diff HEAD failed: {e}"),
            };
        }
    };

    // An empty diff means nothing changed — no tests selected.
    if diff_text.trim().is_empty() {
        return InvocationOutcome {
            termination: Termination::ObservationHeld,
            message: "affected-select: no working-tree diff — 0 selected, 0 executed, 0 passed"
                .to_string(),
        };
    }

    // Step 3: parse the diff into FloorDiffEdits (touched entry files, edited functions, etc.).
    let edits = match cli_run::floor_diff_edits_from_diff_text(&index, &diff_text) {
        Ok(e) => e,
        Err(err) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!("affected-select: REFUSED — diff parse: {err}"),
            };
        }
    };

    // Step 4: compute touched paths from the diff edits (single authority:
    // touched_paths_from_diff_edits — both production and test call this).
    let touched_paths = touched_paths_from_diff_edits(&edits);
    if touched_paths.is_empty() {
        return InvocationOutcome {
            termination: Termination::ObservationHeld,
            message: "affected-select: diff touches no .dag entry files — 0 selected, 0 executed, 0 passed"
                .to_string(),
        };
    }

    // Step 5: discover the full witness roster (all discoverable entries, not CI-filtered).
    let exclude_substrings = cli_run::witness_exclusion_substrings();
    let all_rows = match cli_run::discover_floor_witness_roster(
        &source_roots,
        &source_roots,
        &exclude_substrings,
        &[], // no discovery scope dirs — full tree
    ) {
        Ok(rows) => rows,
        Err(err) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!(
                    "affected-select: REFUSED — witness roster discovery failed: {err}"
                ),
            };
        }
    };

    // Step 6: filter to affected rows only.
    let mut selected_rows = Vec::new();
    let mut messages: Vec<String> = Vec::new();
    let mut any_refused = false;
    let mut refusal_message = String::new();

    for row in &all_rows {
        match cli_run::entry_file_touched_via_import_closure(
            &row.entry,
            facts,
            &facts.declared_paths,
            &touched_paths,
        ) {
            Ok(true) => {
                selected_rows.push(row.clone());
                messages.push(format!("SELECTED {}", row.label));
            }
            Ok(false) => { /* unaffected — skip */ }
            Err(refusal) => {
                // Refusal on any entry fails the whole batch (§5 fail-closed).
                any_refused = true;
                refusal_message = format!(
                    "affected-select: REFUSED — entry '{}' (label '{}'): {refusal}",
                    row.entry, row.label,
                );
                break;
            }
        }
    }

    if any_refused {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: refusal_message,
        };
    }

    let selected_count = selected_rows.len();

    // Step 7: prime witness execution legs — required before run_discovery_rows.
    // Without this, the first selected row panics ("entry was not primed").
    // Also arm the entry retention schedule for the filtered set.
    cli_run::prime_witness_execution_legs(
        &index,
        selected_rows.iter().map(|row| row.entry.as_str()),
    );
    cli_run::index_arm_schedule_retention(&index, &selected_rows);

    // Step 8: run the affected rows through the existing witness runner.
    // Wrap in catch_unwind for §5 fail-closed: a panic inside the runner must
    // never escape as an abort — it must land as Termination::Refused.
    let summary = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        cli_run::run_discovery_rows(
            &selected_rows,
            &index,
            v1_interpreter::ExecutionMode::Hermetic,
            None,
            cli_run::WitnessBudgetPolicy {
                cpu_eval_budget_ms: None,
                wet_receipt_wall_budget_ms: None,
            },
            cli_run::ShardStyle::single_shard(),
        )
    })) {
        Ok(Ok(s)) => s,
        Ok(Err(err)) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!("affected-select: witness runner refused: {err}"),
            };
        }
        Err(panic_payload) => {
            let msg = panic_payload
                .downcast_ref::<String>()
                .map(|s| s.clone())
                .or_else(|| panic_payload.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "unknown panic".to_string());
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!("affected-select: witness runner panicked — {msg}"),
            };
        }
    };

    let passed = summary.passed;
    let skipped = summary.skipped;
    // Fail-closed accounting: if the runner's summary reports more passed+skipped than
    // selected rows, the accounting is inconsistent — refuse with a typed diagnostic
    // rather than computing a panicking unsigned underflow or a misleading negative.
    let failed = if skipped > selected_rows.len() || passed + skipped > selected_rows.len() {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!(
                "affected-select: REFUSED — witness runner accounting mismatch: \
                 selected={selected_count} passed={passed} skipped={skipped} violates passed+skipped ≤ selected"
            ),
        };
    } else {
        selected_rows.len() - passed - skipped
    };

    // Step 8: build the receipt message.
    let mut message = messages.join("\n");
    for failure in &summary.failures {
        message.push_str(&format!("\nFAIL {failure}"));
    }
    for divergence in &summary.divergences {
        message.push_str(&format!("\nDIVERGENCE {divergence}"));
    }
    message.push_str(&format!(
        "\n{selected_count} selected, {} executed ({} passed, {failed} failed, {skipped} skipped)",
        selected_count.saturating_sub(skipped),
        passed,
    ));

    let termination = if failed > 0 || !summary.failures.is_empty() {
        Termination::ObservationDidNotHold
    } else {
        Termination::ObservationHeld
    };

    InvocationOutcome {
        termination,
        message,
    }
}

/// `gunbc.target_invocation` `TargetInvocationRefusal`, MINUS ONE ARM, deliberately.
///
/// The model separates a known target with no bound producer from an unknown target because
/// `gunbc.target_binding` keeps two lists. Here the registry is a list of PAIRS, so a producer-less
/// target is unwritable (DESIGN section 4b, structural impossibility) and an unconstructible arm
/// would be decoration read as coverage. If the host ever takes the two populations separately,
/// the arm returns with the state that makes it reachable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvocationRefusal {
    OperandNotALabel {
        operand: String,
        cause: LabelRefusal,
    },
    TargetIsUnknown {
        target: String,
    },
}

fn invocation_refusal_rendered(refusal: &InvocationRefusal) -> String {
    {
        match refusal {
            InvocationRefusal::OperandNotALabel { operand, cause } => format!(
                "gunbc test: operand is not an absolute label: {operand}\n  cause: {}",
                label_refusal_rendered(cause)
            ),
            InvocationRefusal::TargetIsUnknown { target } => {
                format!("gunbc test: no such target: {target}")
            }
        }
    }
}

/// `gunbc.target_invocation` `InvocationTermination`. `SubjectUnreached` is NOT
/// `ObservationDidNotHold`: a nonexistent root observed nothing, and that is not a defect finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Termination {
    ObservationHeld,
    ObservationDidNotHold,
    SubjectUnreached,
    Refused,
}

/// Three statuses, not two: 1 is an observation that did not hold, 2 is no observation — conflating
/// them is the absorbing answer DESIGN section 5 forbids. Wildcard-free: a fifth termination must
/// fail to compile rather than inherit a `_` status.
pub fn invocation_exit_status(t: Termination) -> i32 {
    match t {
        Termination::ObservationHeld => 0,
        Termination::ObservationDidNotHold => 1,
        Termination::SubjectUnreached => 2,
        Termination::Refused => 2,
    }
}

/// AN OUTCOME, DELIBERATELY NOT A RECEIPT, AND THE NAME IS THE WHOLE OF WHAT IS CLAIMED.
///
/// The producer executes against an UNPINNED LIVE WORKING TREE: the corpus may change mid-walk,
/// so the reported population is bound to no source state a later run could reconstruct. A
/// receipt would assert that binding.
///
/// NO HASH FIELD CLOSES THIS AND ONE MUST NOT BE ADDED. Hashing before, after, or both proves
/// nothing — the tree can go A -> B -> A while the producer reads a MIXED population and both
/// hashes agree. The bytes producing the manifest must BE the bytes the producer consumes, which
/// observing a mutable namespace and executing afterwards cannot arrange. A digest would look
/// like evidence and carry none.
///
/// NOT CLAIMED: target-result caching, cross-run comparison, remote execution, replay, or "this
/// standing was about source X". Each needs source binding first.
pub struct InvocationOutcome {
    pub termination: Termination,
    pub message: String,
}

/// The differential's own standing, rendered in its own vocabulary.
///
/// It must not say PASSED or FAILED: those are the `//:required` aggregate's words. `narrowed` is
/// reported but excluded from the verdict — declared, bounded scope narrowing, counted rather
/// than absorbed — so only `divergent` and `regressed` decide whether the reading held.
fn run_heads_reading_differential(source_roots: &[String]) -> InvocationOutcome {
    let missing: Vec<&String> = source_roots
        .iter()
        .filter(|r| !std::path::Path::new(r.as_str()).exists())
        .collect();
    if !missing.is_empty() {
        let named: Vec<String> = missing.into_iter().cloned().collect();
        return InvocationOutcome {
            termination: Termination::SubjectUnreached,
            message: format!(
                "heads-reading-differential: subject unreached (source root absent): {}",
                named.join(", ")
            ),
        };
    }
    let d = cli_run::heads_reading_differential(source_roots);
    if d.modules_compared == 0 {
        return InvocationOutcome {
            termination: Termination::SubjectUnreached,
            message: format!(
                "heads-reading-differential: subject unreached (source population index refused): {}",
                source_roots.join(", ")
            ),
        };
    }
    let mut message = format!(
        "heads-reading-differential: compared={} divergent={} narrowed={} regressed={} both_refused={}",
        d.modules_compared,
        d.divergent.len(),
        d.narrowed.len(),
        d.regressed.len(),
        d.both_refused.len(),
    );
    for path in d.divergent.iter() {
        message.push_str(&format!("\nheads-reading-differential: DIVERGENT {path}"));
    }
    for path in d.regressed.iter() {
        message.push_str(&format!("\nheads-reading-differential: REGRESSED {path}"));
    }
    // THE PARSE-WALL FIGURES ARE CARRIED OVER FROM THE DELETED `--heads-reading-differential`
    // MODE, AND THEY ARE HOST OUTPUT RATHER THAN PART OF THE MODELED OBSERVATION.
    //
    // `HeadsReadingDifferentialObservation` carries four populations and no timing, so these two
    // numbers have no home in the standing. Printed rather than dropped because a replacement that
    // silently loses a capability is not a replacement; printed BELOW the populations and outside
    // the verdict, per the split `gunbc.build_target` states: cost has its own authority. Next rung
    // is a cost carrier on the observation; until then this is a named unmodeled host line.
    //
    // It measures the PARSE only — tokenize, newline indexing and per-file setup sit outside both
    // timers — so it is not the whole `pool_parse` saving and must never be quoted as one.
    message.push_str(&format!(
        "\nheads-reading-differential: full_reading_parse_ms={} heads_reading_parse_ms={}",
        d.full_reading_nanos / 1_000_000,
        d.heads_reading_nanos / 1_000_000,
    ));
    InvocationOutcome {
        termination: if d.holds() {
            Termination::ObservationHeld
        } else {
            Termination::ObservationDidNotHold
        },
        message,
    }
}

/// THE REALIZATION DISPATCH, AND IT IS THE ONLY PLACE A PRODUCER IS NAMED. Selecting a realization
/// is itself realization (DESIGN section 3): periphery, never the route above or the CLI surface.
fn run_producer(producer: TargetProducer) -> InvocationOutcome {
    match producer {
        TargetProducer::HeadsReadingDifferential => {
            run_heads_reading_differential(&heads_reading_differential_source_roots())
        }
        TargetProducer::BehavioralReceiptPlan => behavioral_outcome(
            cli_run::behavioral_receipt_host::run_plan(&behavioral_receipt_source_roots()),
        ),
        TargetProducer::BehavioralReceiptCensus => behavioral_outcome(
            cli_run::behavioral_receipt_host::run_census(&behavioral_receipt_source_roots()),
        ),
        TargetProducer::BehavioralReceiptSelftest => behavioral_outcome(
            cli_run::behavioral_receipt_host::run_selftest(&behavioral_receipt_source_roots()),
        ),
    }
}

fn behavioral_receipt_source_roots() -> Vec<String> {
    vec!["dag".to_string(), "src/v2".to_string()]
}

fn behavioral_outcome(
    outcome: cli_run::behavioral_receipt_host::BehavioralHostOutcome,
) -> InvocationOutcome {
    use cli_run::behavioral_receipt_host::BehavioralHostTermination;
    InvocationOutcome {
        termination: match outcome.termination {
            BehavioralHostTermination::ObservationHeld => Termination::ObservationHeld,
            BehavioralHostTermination::ObservationDidNotHold => Termination::ObservationDidNotHold,
            BehavioralHostTermination::SubjectUnreached => Termination::SubjectUnreached,
            BehavioralHostTermination::Refused => Termination::Refused,
        },
        message: outcome.message,
    }
}

/// THE ONE SEAM: argv operand -> label -> registry -> exact binding -> producer -> native standing.
///
/// Lookup is `label_eq` once per row, and EXACT — no prefix, suffix or "did you mean": a near miss
/// silently running a different target is worse than a refusal naming the one asked for.
pub fn test_verb(operand: &str) -> InvocationOutcome {
    let label = match parse_label(operand) {
        Ok(l) => l,
        Err(cause) => {
            let refusal = InvocationRefusal::OperandNotALabel {
                operand: operand.to_string(),
                cause,
            };
            return InvocationOutcome {
                termination: Termination::Refused,
                message: invocation_refusal_rendered(&refusal),
            };
        }
    };
    let key = render_label(&label);
    match instrument_registry()
        .into_iter()
        .find(|(candidate, _)| render_label(candidate) == key)
    {
        None => {
            let refusal = InvocationRefusal::TargetIsUnknown { target: key };
            InvocationOutcome {
                termination: Termination::Refused,
                message: invocation_refusal_rendered(&refusal),
            }
        }
        Some((_, producer)) => run_producer(producer),
    }
}

/// Discriminating RED controls for the affected-set selection authority.
/// These tests validate `entry_file_touched_via_import_closure` against a
/// CONTROLLED dependency fixture whose expected identities come from authored
/// structure — independent of the selector. A mutation that flips the predicate
/// (e.g., mapping "refused" to "unaffected") must go RED.
///
/// Fixture structure (compile_clean_scope, /src/v2/test/fixture/compile_clean_scope/):
///   scope_shared   — edgeless, imported by scope_importer
///   scope_importer — imports scope_shared
///   scope_isolated — edgeless, no import edge to scope_shared
#[cfg(test)]
mod affected_select_discriminating_red_controls {
    use super::*;
    use crate::cli_run::{
        build_multi_entry_index, entry_file_touched_via_import_closure,
        floor_diff_edits_from_diff_text, workspace_root,
    };
    use std::path::PathBuf;

    fn ws() -> PathBuf {
        workspace_root()
    }

    fn abs(ws: &PathBuf, rel: &str) -> String {
        ws.join(rel).to_string_lossy().into_owned()
    }

    fn setup_roots(ws: &PathBuf) -> Vec<String> {
        vec![
            ws.join("dag").to_string_lossy().into_owned(),
            ws.join("src/v2").to_string_lossy().into_owned(),
        ]
    }

    fn diff_at(file: &str, line: i64) -> String {
        format!("diff --git a/{file} b/{file}\n--- a/{file}\n+++ b/{file}\n@@ -{line},1 +{line},1 @@\n-old\n+new\n")
    }

    const FIXTURE_DIR: &str = "src/v2/test/fixture/compile_clean_scope";
    const SCOPE_IMPORTER: &str = "src/v2/test/fixture/compile_clean_scope/scope_importer.dag";
    const SCOPE_SHARED: &str = "src/v2/test/fixture/compile_clean_scope/scope_shared.dag";
    const SCOPE_ISOLATED: &str = "src/v2/test/fixture/compile_clean_scope/scope_isolated.dag";

    /// 1. Entry file touched → selected.
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    #[test]
    fn entry_file_touched_is_selected() {
        let ws = ws();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let facts = index.module_graph_facts();
        let declared = facts.declared_paths.clone();
        let touched = vec![SCOPE_IMPORTER.to_string()];
        assert!(
            entry_file_touched_via_import_closure(
                &abs(&ws, SCOPE_IMPORTER),
                facts,
                &declared,
                &touched,
            )
            .expect("entry_file_touched_via_import_closure"),
            "touching the entry itself must select it"
        );
    }

    /// 2. Direct dependency touched → selected (scope_importer imports scope_shared).
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    #[test]
    fn direct_dependency_touched_is_selected() {
        let ws = ws();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let facts = index.module_graph_facts();
        let declared = facts.declared_paths.clone();
        let touched = vec![SCOPE_SHARED.to_string()];
        assert!(
            entry_file_touched_via_import_closure(
                &abs(&ws, SCOPE_IMPORTER),
                facts,
                &declared,
                &touched,
            )
            .expect("entry_file_touched_via_import_closure"),
            "touching scope_shared must select scope_importer through the import edge"
        );
    }

    /// 3. Transitive dependency touched → selected.
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    #[test]
    fn transitive_dependency_touched_is_selected() {
        let ws = ws();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let facts = index.module_graph_facts();
        let declared = facts.declared_paths.clone();
        let touched = vec![SCOPE_SHARED.to_string()];
        assert!(
            entry_file_touched_via_import_closure(
                &abs(&ws, SCOPE_SHARED),
                facts,
                &declared,
                &touched,
            )
            .expect("entry_file_touched_via_import_closure"),
            "touching scope_shared must select scope_shared itself (self-selection)"
        );
    }

    /// 4. Disconnected entry → excluded (scope_isolated has no import edge to scope_shared).
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    #[test]
    fn disconnected_entry_is_excluded() {
        let ws = ws();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let facts = index.module_graph_facts();
        let declared = facts.declared_paths.clone();
        let touched = vec![SCOPE_SHARED.to_string()];
        assert!(
            !entry_file_touched_via_import_closure(
                &abs(&ws, SCOPE_ISOLATED),
                facts,
                &declared,
                &touched,
            )
            .expect("entry_file_touched_via_import_closure"),
            "touching scope_shared must NOT select scope_isolated (no import edge)"
        );
    }

    /// 5. Edgeless entry, untouched → excluded.
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    #[test]
    fn edgeless_untouched_entry_is_excluded() {
        let ws = ws();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let facts = index.module_graph_facts();
        let declared = facts.declared_paths.clone();
        let touched = vec!["src/v2/lens/affected_set.dag".to_string()];
        assert!(
            !entry_file_touched_via_import_closure(
                &abs(&ws, SCOPE_ISOLATED),
                facts,
                &declared,
                &touched,
            )
            .expect("entry_file_touched_via_import_closure"),
            "touching an unrelated file must NOT select edgeless scope_isolated"
        );
    }

    /// 6. Edgeless entry, itself touched → selected.
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    #[test]
    fn edgeless_entry_self_touched_is_selected() {
        let ws = ws();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let facts = index.module_graph_facts();
        let declared = facts.declared_paths.clone();
        let touched = vec![SCOPE_ISOLATED.to_string()];
        assert!(
            entry_file_touched_via_import_closure(
                &abs(&ws, SCOPE_ISOLATED),
                facts,
                &declared,
                &touched,
            )
            .expect("entry_file_touched_via_import_closure"),
            "touching scope_isolated's own file must select it even with no imports"
        );
    }

    /// 7. Empty touched set → selects none.
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    #[test]
    fn empty_touched_set_selects_none() {
        let ws = ws();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let facts = index.module_graph_facts();
        let declared = facts.declared_paths.clone();
        let touched: Vec<String> = vec![];
        assert!(
            !entry_file_touched_via_import_closure(
                &abs(&ws, SCOPE_IMPORTER),
                facts,
                &declared,
                &touched,
            )
            .expect("entry_file_touched_via_import_closure"),
            "empty touched set must select nothing"
        );
    }

    /// 8. End-to-end: a diff touching a fixture file and verifying the affected-set
    /// produces non-empty touched paths and selects the dependent.
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    #[test]
    fn diff_touching_shared_selects_importer() {
        let ws = ws();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let facts = index.module_graph_facts();
        let declared = facts.declared_paths.clone();
        let diff = diff_at(SCOPE_SHARED, 9);
        let edits = floor_diff_edits_from_diff_text(&index, &diff).expect("diff parse");
        let touched: Vec<String> = edits.touched_entry_files.iter().cloned().collect();
        assert!(
            !touched.is_empty(),
            "diff touching scope_shared must produce touched entry files"
        );
        assert!(
            entry_file_touched_via_import_closure(
                &abs(&ws, SCOPE_IMPORTER),
                facts,
                &declared,
                &touched,
            )
            .expect("entry_file_touched_via_import_closure"),
            "diff touching scope_shared must select scope_importer (direct dependency)"
        );
    }

    /// 9. Discriminating RED: a data-row-only diff (touching the value of a `data`
    /// declaration) must produce a nonempty touched-path set through the union of
    /// ALL THREE diff-edit buckets. Before the fix, `test_affected_select` consumed
    /// only `touched_entry_files` — a data-row edit routed to `overlapping_data_items`,
    /// the touched set came back empty, and the command printed "0 selected" for a
    /// real change inside the import closure of live witnesses.
    ///
    /// Uses `scope_importer.dag` line 8 (`data scope_importer_marker: String = scope_shared_marker`),
    /// whose body is a name reference (not a string literal), so `item_kind` classifies it as
    /// `DataItem` -> the edit routes to `overlapping_data_items`. A string-literal-only data decl
    /// (e.g. `scope_shared_marker`) is NOT classified as `DataItem` because the parsed node has
    /// `body = None` for string literal values -- it falls through to `OtherItem` -> `touched_entry_files`.
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    #[test]
    fn data_row_only_diff_selects_through_overlapping_data_items() {
        let ws = ws();
        std::env::set_current_dir(&ws).expect("chdir workspace");
        let roots = setup_roots(&ws);
        let index = build_multi_entry_index(&roots);
        let facts = index.module_graph_facts();
        let declared = facts.declared_paths.clone();
        // Touch a data-declaration line at scope_importer.dag line 8
        // (data scope_importer_marker: String = scope_shared_marker).
        // This uses a NAME REFERENCE as the body, so the parsed node has body != None,
        // and item_kind classifies it as DataItem -> overlapping_data_items.
        let diff = diff_at(SCOPE_IMPORTER, 8);
        let edits = floor_diff_edits_from_diff_text(&index, &diff).expect("diff parse");
        // Verify the edit lands in overlapping_data_items specifically, NOT touched_entry_files.
        // This is the discriminating RED: if the item_kind classifier changes or the
        // three-bucket union drops overlapping_data_items, this assertion goes red.
        assert!(
            !edits.overlapping_data_items.is_empty(),
            "data-row diff (non-literal body) must populate overlapping_data_items: touched_entry_files={}, overlapping_data_items={}, edited_test_fns={}",
            edits.touched_entry_files.len(),
            edits.overlapping_data_items.len(),
            edits.edited_test_fns.len(),
        );
        // Consume the SINGLE AUTHORITY: touched_paths_from_diff_edits -- the same
        // function test_affected_select calls. If the union logic regresses (e.g.
        // dropping overlapping_data_items), this test goes RED.
        let touched = touched_paths_from_diff_edits(&edits);
        assert!(
            !touched.is_empty(),
            "data-row-only diff must produce nonempty touched paths through three-bucket union"
        );
        assert!(
            entry_file_touched_via_import_closure(
                &abs(&ws, SCOPE_IMPORTER),
                facts,
                &declared,
                &touched,
            )
            .expect("entry_file_touched_via_import_closure"),
            "data-row-only diff in scope_importer must select scope_importer (self-selection)"
        );
    }
}
