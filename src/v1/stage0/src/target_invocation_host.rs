// THE HOST REALIZATION OF `gunbc test <target-pattern>`, AND IT IS A HAND MIRROR OF A `.dag`
// AUTHORITY.
//
// The model is `gunbc.target_invocation` (operand admission, generic exact-label route,
// termination vocabulary), `gunbc.instrument_targets` (live target and binding rows, the
// differential's classifier and rendering), `extdeps.bazel.label` (the label grammar mirrored
// here) and `extdeps.bazel.target_pattern` (the pattern grammar mirrored here). The SET forms
// (`//pkg:all`, `//pkg:*`, `//pkg/...`) this file admits are NOT executed here: they delegate to
// the `.dag` authority `gunbc.compute.test_run` `test_verb_pattern_cli` through the interpreter,
// so witness selection and execution stay modeled and this file stays a mirror. None of the
// modeled modules is in the v1 seed's emitted closure — `src/gunbc_cli_dispatch_surface.rs` is
// the only `gunbc.*` mirror the emitter produces — so this file is hand-written beside the
// carrier, as `required_regen_host.rs` mirrors `v2.workflow.required_regen`. The seam is
// therefore MITIGATABLE, not structurally guaranteed: the two can drift until the seam is
// emitted rather than authored. The obligation is enrolled in
// `gunbc.target_invocation_seed_growth`.
//
// WHAT IS AND IS NOT GENERIC HERE. One route: argv operand -> admit pattern -> a single target
// builds the registry, exact lookup, invoke the bound producer, render its native standing; a
// set form runs the witness machinery in `.dag`. No per-instrument arm on that route, and none
// may be added; a second instrument is a row in `instrument_registry` plus one `Producer` arm in
// `run_producer` — the peripheral realization dispatch DESIGN section 3 keeps out of the
// interface. Deliberately NOT here: any consultation of `//:required` aggregate policy or the
// Blaze status export — both refuse instrument producers by design — and any re-implementation
// of witness selection, which is `gunbc.compute.test_selection`'s to own.

use crate::cli_run;

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

/// `extdeps.bazel.target_pattern` `TargetPattern`, mirrored: a pattern denotes a SET, a label one
/// target. The single-target arm delegates to the label grammar above so a target name is spelled
/// and refused in exactly one place on this side of the seam as well.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetPattern {
    SingleTarget(Label),
    PackageTargets(Vec<String>),
    SubtreeTargets(Vec<String>),
}

/// `extdeps.bazel.target_pattern` `TargetPatternRefusal`, mirrored arm for arm: the remedies
/// differ (make it absolute; fix the package the label grammar refuses; fix the single target the
/// label grammar refuses; `:all-targets` has no meaning over a corpus of test modules), so the
/// causes are not collapsible into one malformed-operand bit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetPatternRefusal {
    PatternNotAbsolute(String),
    PatternPackageRefused(LabelRefusal),
    PatternLabelRefused(LabelRefusal),
    PatternAllTargetsUnsupported(String),
}

/// `target_pattern_refusal_text`, mirrored.
fn target_pattern_refusal_text(cause: &TargetPatternRefusal) -> String {
    match cause {
        TargetPatternRefusal::PatternNotAbsolute(t) => {
            format!("target pattern is not absolute (must start with //): {t}")
        }
        TargetPatternRefusal::PatternPackageRefused(_) => {
            "target pattern names a package the label grammar refuses".to_string()
        }
        TargetPatternRefusal::PatternLabelRefused(_) => {
            "target pattern names a single target the label grammar refuses".to_string()
        }
        TargetPatternRefusal::PatternAllTargetsUnsupported(t) => {
            format!(":all-targets has no meaning over a corpus of test modules: {t}")
        }
    }
}

fn parse_pattern_package(package_text: &str) -> Result<Vec<String>, TargetPatternRefusal> {
    if package_text.is_empty() {
        Ok(Vec::new())
    } else {
        parse_package_segments(package_text).map_err(TargetPatternRefusal::PatternPackageRefused)
    }
}

/// `parse_target_pattern`, mirrored. The branch order is the authority's: absolute prefix, the
/// `:all-targets` exclusion, the subtree suffixes, the package-wide suffixes, and only then the
/// single-target delegation — a different order would admit or refuse different texts.
pub fn parse_target_pattern(text: &str) -> Result<TargetPattern, TargetPatternRefusal> {
    if !text.starts_with("//") {
        return Err(TargetPatternRefusal::PatternNotAbsolute(text.to_string()));
    }
    if text.ends_with(":all-targets") {
        return Err(TargetPatternRefusal::PatternAllTargetsUnsupported(
            text.to_string(),
        ));
    }
    let body = &text[2..];
    let subtree_body = if body.ends_with("/...:all") {
        Some(&body[..body.len() - ":all".len()])
    } else if body.ends_with("/...:*") {
        Some(&body[..body.len() - ":*".len()])
    } else if body.ends_with("/...") || body == "..." {
        Some(body)
    } else if body == "...:all" || body == "...:*" {
        Some("...")
    } else {
        None
    };
    if let Some(stripped) = subtree_body {
        let package_text = if stripped == "..." {
            ""
        } else {
            &stripped[..stripped.len() - "/...".len()]
        };
        return parse_pattern_package(package_text).map(TargetPattern::SubtreeTargets);
    }
    if let Some(stripped) = body
        .strip_suffix(":all")
        .or_else(|| body.strip_suffix(":*"))
    {
        return parse_pattern_package(stripped).map(TargetPattern::PackageTargets);
    }
    parse_label(text)
        .map(TargetPattern::SingleTarget)
        .map_err(TargetPatternRefusal::PatternLabelRefused)
}

/// `gunbc.target_binding` `TargetProducer`, narrowed to the members this seam realizes today.
/// Adding one is a row in `instrument_registry` and an arm here; it is not a new route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetProducer {
    SelfHost,
    V2NativeCli,
    HeadsReadingDifferential,
    BehavioralReceiptPlan,
    BehavioralReceiptCensus,
    BehavioralReceiptSelftest,
    CompileCleanDiagnosticCensus,
    EvaluationStoreAddressExactHead,
    PrimitiveEgressCensus,
    PrimitiveEgressCensusV2,
    PrimitiveEgressCensusDag,
    PrimitiveEgressCensusSeed,
}

/// `gunbc.instrument_targets` `instrument_targets` / `instrument_bindings`, as the pairs the
/// index is built from. All four modeled instruments are rows on the same generic seam.
/// `gunbc.instrument_targets` `heads_reading_differential_source_roots`. The subject is the
/// instrument's own fact, not a CLI option, so an invocation cannot quietly measure another corpus.
fn heads_reading_differential_source_roots() -> Vec<String> {
    vec!["dag".to_string(), "src/v2".to_string()]
}

/// `gunbc.instrument_targets` `self_host_source_roots`. Which corpus the seed emits v2 FROM is
/// this instrument's own fact on the same rule its siblings follow, so an invocation cannot quietly
/// build a different closure while reporting this target's standing.
fn self_host_source_roots() -> Vec<String> {
    vec!["dag".to_string(), "src/v2".to_string()]
}

/// `gunbc.instrument_targets` `v2_native_cli_source_roots`. Which corpus the v2-native CLI's closure
/// is emitted FROM is this instrument's own fact, on the same rule its siblings follow.
fn v2_native_cli_source_roots() -> Vec<String> {
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
        (
            instrument_label("compile-clean-diagnostic-census"),
            TargetProducer::CompileCleanDiagnosticCensus,
        ),
        (instrument_label("self-host"), TargetProducer::SelfHost),
        (
            instrument_label("v2-native-cli"),
            TargetProducer::V2NativeCli,
        ),
        (
            instrument_label("evaluation-store-address-exact-head"),
            TargetProducer::EvaluationStoreAddressExactHead,
        ),
        (
            instrument_label("primitive-egress-census"),
            TargetProducer::PrimitiveEgressCensus,
        ),
        (
            instrument_label("primitive-egress-census-v2"),
            TargetProducer::PrimitiveEgressCensusV2,
        ),
        (
            instrument_label("primitive-egress-census-dag"),
            TargetProducer::PrimitiveEgressCensusDag,
        ),
        (
            instrument_label("primitive-egress-census-seed"),
            TargetProducer::PrimitiveEgressCensusSeed,
        ),
    ]
}

fn instrument_label(target: &str) -> Label {
    Label {
        package_segments: vec!["gunbc".to_string(), "instruments".to_string()],
        target: target.to_string(),
    }
}

/// `gunbc.target_invocation` refusal vocabulary AT THIS SEAM, narrowed twice, deliberately.
///
/// The model separates a known target with no bound producer from an unknown target because
/// `gunbc.target_binding` keeps two lists. Here the registry is a list of PAIRS, so a producer-less
/// target is unwritable (DESIGN section 4b, structural impossibility) and an unconstructible arm
/// would be decoration read as coverage. If the host ever takes the two populations separately,
/// the arm returns with the state that makes it reachable.
///
/// `OperandNotALabel` is likewise ABSENT since the operand became a target PATTERN: every label
/// refusal now arrives inside `parse_target_pattern`'s `PatternLabelRefused`, rendered by
/// `target_pattern_refusal_text`, so the arm has no constructor left. The modeled
/// `TargetInvocationRefusal` keeps it — `route_target_invocation` remains the exact-label route
/// the witness-selection half delegates around.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvocationRefusal {
    TargetIsUnknown { target: String },
}

/// THE REFUSAL NAMES WHAT WOULD HAVE WORKED, AND IT IS DERIVED RATHER THAN WRITTEN DOWN.
///
/// A reader who has forgotten the label is exactly the reader holding this refusal, so the roster
/// belongs here and not only in a document. It is read from `instrument_registry` -- the same rows
/// the lookup just failed against -- so a listing cannot disagree with what is invocable: adding an
/// instrument updates this text by construction, and a prose page listing them would be the second
/// representation DESIGN sections 2 and 3 price, drifting the first time someone adds a row and
/// does not think to edit prose.
fn rostered_targets_rendered() -> String {
    let mut lines = vec!["  available targets:".to_string()];
    for (label, _) in instrument_registry() {
        lines.push(format!("    {}", render_label(&label)));
    }
    lines.join("\n")
}

fn invocation_refusal_rendered(refusal: &InvocationRefusal) -> String {
    {
        match refusal {
            InvocationRefusal::TargetIsUnknown { target } => {
                format!(
                    "gunbc test: no such target: {target}\n{}",
                    rostered_targets_rendered()
                )
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
        TargetProducer::CompileCleanDiagnosticCensus => run_compile_clean_diagnostic_census(),
        TargetProducer::SelfHost => run_self_host(&self_host_source_roots()),
        TargetProducer::V2NativeCli => run_v2_native_cli(&v2_native_cli_source_roots()),
        TargetProducer::EvaluationStoreAddressExactHead => {
            run_evaluation_store_address_exact_head()
        }
        TargetProducer::PrimitiveEgressCensus => {
            run_primitive_egress_census("primitive-egress-census", "primitive_egress_census_exit")
        }
        TargetProducer::PrimitiveEgressCensusV2 => run_primitive_egress_census(
            "primitive-egress-census-v2",
            "primitive_egress_census_v2_exit",
        ),
        TargetProducer::PrimitiveEgressCensusDag => run_primitive_egress_census(
            "primitive-egress-census-dag",
            "primitive_egress_census_dag_exit",
        ),
        TargetProducer::PrimitiveEgressCensusSeed => run_primitive_egress_census(
            "primitive-egress-census-seed",
            "primitive_egress_census_seed_exit",
        ),
    }
}

/// THE PRIMITIVE EGRESS CENSUS PRODUCERS (gunbc#11642): evaluate one of the
/// `gunbc.primitive_egress.census_live` `primitive_egress_census_*_exit` entries, each answering a
/// `CliWireResponse` -- the receipt bytes (one JSON object per line) plus the exit the census
/// standing carries. The bytes are printed on EVERY termination, because the receipt is the
/// product and a did-not-hold census is exactly when its identity lists are wanted.
///
/// THE THREE TERMINATIONS map off the wire exit read through the one classifier in `cli_run`:
/// success is the census holding; `ExitFailure { code: 1 }` is a located census finding (an
/// identity with zero or two dispositions) and is `ObservationDidNotHold`; `ExitFailure { code: 2 }`
/// is the population not being established (a refused entry, no identities) and is `Refused`;
/// a resolve or eval failure is `SubjectUnreached`.
/// One runner for the four census labels: the scope is the `.dag` entry function the label
/// names (`gunbc.primitive_egress.census_live` `census_wire(scope:)` decides what it walks), so a
/// bounded projection is a label of its own and not a flag on the full one.
fn run_primitive_egress_census(label: &'static str, function: &'static str) -> InvocationOutcome {
    const ENTRY: &str = "dag/gunbc/primitive_egress/census_live.dag";
    let label_name = label;
    let function_name = function;
    if let Err(e) = std::env::set_current_dir(cli_run::workspace_root()) {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!("{label_name}: refused: could not anchor at the workspace root: {e}"),
        };
    }
    let roots = cli_run::default_source_roots();
    let (graph, source_indices) = match cli_run::resolve_entry_graph(&roots, ENTRY) {
        Ok(resolved) => resolved,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label_name}: resolve failed for {ENTRY}: {cause}"),
            };
        }
    };
    let blocking = crate::v1_compiler_compile::interpreter_blocking_diagnostic_messages(
        graph.diagnostics.clone(),
    );
    if !blocking.is_empty() {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!(
                "{label_name}: {ENTRY} has blocking diagnostics: {}",
                blocking.iter().cloned().collect::<Vec<_>>().join("; ")
            ),
        };
    }
    let ctx = cli_run::make_eval_context(
        graph.as_ref(),
        source_indices,
        crate::v1_interpreter::ExecutionMode::Wet,
    );
    let value =
        match crate::v1_interpreter::run_in_context_with_args(&ctx, function_name, &[], true) {
            Ok(value) => value,
            Err(cause) => {
                return InvocationOutcome {
                    termination: Termination::SubjectUnreached,
                    message: format!("{label_name}: eval failed: {cause}"),
                }
            }
        };
    match cli_run::classify_cli_wire(&value, &ctx) {
        cli_run::CliWireClass::Printable { bytes, exit } => {
            let termination = match &exit {
                cli_run::ExitClass::Success => Termination::ObservationHeld,
                cli_run::ExitClass::Failure { code: 1, .. } => Termination::ObservationDidNotHold,
                cli_run::ExitClass::Failure { .. } => Termination::Refused,
                cli_run::ExitClass::NotProcessExit { .. } => Termination::Refused,
            };
            let reason = match exit {
                cli_run::ExitClass::Success => format!("{label_name}: held"),
                cli_run::ExitClass::Failure { reason, .. } => {
                    reason.unwrap_or_else(|| format!("{label_name}: failed"))
                }
                cli_run::ExitClass::NotProcessExit { type_name } => {
                    format!("{label_name}: wire exit is `{type_name}`, not a ProcessExit")
                }
            };
            InvocationOutcome {
                termination,
                message: format!("{bytes}{reason}"),
            }
        }
        cli_run::CliWireClass::Unprintable { cause } => InvocationOutcome {
            termination: Termination::Refused,
            message: format!("{label_name}: renderer refused: {cause}"),
        },
        cli_run::CliWireClass::NotCliWire { type_name } => InvocationOutcome {
            termination: Termination::Refused,
            message: format!(
                "{label_name}: {function_name} returned `{type_name}`, not a CliWireResponse"
            ),
        },
        cli_run::CliWireClass::MalformedCliWire { detail } => InvocationOutcome {
            termination: Termination::Refused,
            message: format!("{label_name}: malformed CliWireResponse: {detail}"),
        },
    }
}

fn run_evaluation_store_address_exact_head() -> InvocationOutcome {
    const ENTRY: &str = "dag/gunbc/evaluation_store_address_census.dag";
    const FUNCTION: &str = "evaluation_store_address_census_joins_exact_head_declaration_graph";
    if let Err(e) = std::env::set_current_dir(cli_run::workspace_root()) {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!(
                "evaluation-store-address-exact-head: refused: could not anchor at the workspace root: {e}"
            ),
        };
    }
    let roots = cli_run::default_source_roots();
    let (graph, source_indices) = match cli_run::resolve_entry_graph(&roots, ENTRY) {
        Ok(resolved) => resolved,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!(
                    "evaluation-store-address-exact-head: resolve failed for {ENTRY}: {cause}"
                ),
            };
        }
    };
    let blocking = crate::v1_compiler_compile::interpreter_blocking_diagnostic_messages(
        graph.diagnostics.clone(),
    );
    if !blocking.is_empty() {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!(
                "evaluation-store-address-exact-head: {ENTRY} has blocking diagnostics: {}",
                blocking.iter().cloned().collect::<Vec<_>>().join("; ")
            ),
        };
    }
    let ctx = cli_run::make_eval_context(
        graph.as_ref(),
        source_indices,
        crate::v1_interpreter::ExecutionMode::Wet,
    );
    match crate::v1_interpreter::run_in_context_with_args(&ctx, FUNCTION, &[], true) {
        Ok(value) => exact_head_standing_outcome(&ctx, FUNCTION, &value),
        Err(cause) => InvocationOutcome {
            termination: Termination::SubjectUnreached,
            message: format!("evaluation-store-address-exact-head: eval failed: {cause}"),
        },
    }
}

fn exact_head_standing_outcome(
    ctx: &crate::v1_interpreter::InterpContext,
    function: &str,
    value: &crate::v1_interpreter::Value,
) -> InvocationOutcome {
    match value {
        crate::v1_interpreter::Value::Variant { variant_name, .. }
            if ctx.sym_eq(*variant_name, "ExactHeadHeld") =>
        {
            InvocationOutcome {
                termination: Termination::ObservationHeld,
                message: format!("evaluation-store-address-exact-head: held ({function})"),
            }
        }
        crate::v1_interpreter::Value::Variant { variant_name, .. }
            if ctx.sym_eq(*variant_name, "ExactHeadDidNotHold") =>
        {
            InvocationOutcome {
                termination: Termination::ObservationDidNotHold,
                message: format!(
                    "evaluation-store-address-exact-head: {}",
                    ctx.format_value(value)
                ),
            }
        }
        crate::v1_interpreter::Value::Variant { variant_name, .. }
            if ctx.sym_eq(*variant_name, "ExactHeadRefused") =>
        {
            InvocationOutcome {
                termination: Termination::Refused,
                message: format!(
                    "evaluation-store-address-exact-head: {}",
                    ctx.format_value(value)
                ),
            }
        }
        other => InvocationOutcome {
            termination: Termination::Refused,
            message: format!(
                "evaluation-store-address-exact-head: {function} returned a non-standing value: {}",
                ctx.format_value(other)
            ),
        },
    }
}

/// THE SELF-HOST PRODUCER: the generation this repository can perform, asked as one question.
///
/// It calls the SAME producer `--v2-native-route` calls, so the two cannot disagree about whether
/// the seed can build v2 -- only one of them decides it. That route then spends roughly nine
/// further minutes executing the v2.test.* universe through the emitted binary, which is a
/// different claim (what the emitted compiler ANSWERS), and bundling it here would price the
/// self-host question at the cost of a question nobody asked.
///
/// THE THREE TERMINATIONS ARE NOT TWO. A refusal from `prepare_emitted_compiler` is the subject
/// failing to be reached -- the emit refused, the crate would not write, cargo could not run --
/// which is `SubjectUnreached` rather than an observation that the seed cannot build v2. Only a
/// completed build whose own counters are non-zero is `ObservationDidNotHold`. Collapsing those is
/// the absorbing answer DESIGN section 5 forbids, and here it would report a broken bench as a
/// broken compiler.
fn run_self_host(source_roots: &[String]) -> InvocationOutcome {
    match cli_run::run_self_host(source_roots) {
        Ok(held) => InvocationOutcome {
            termination: if held.exit_status == 0 && held.warning_count == 0 {
                Termination::ObservationHeld
            } else {
                Termination::ObservationDidNotHold
            },
            message: format!(
                "self-host v1->v2: closure={} binary={} seed={} exit_status={} warning_count={}",
                held.closure_identity,
                held.binary_identity,
                held.seed_identity,
                held.exit_status,
                held.warning_count,
            ),
        },
        Err(cause) => InvocationOutcome {
            termination: Termination::SubjectUnreached,
            message: cause,
        },
    }
}

/// THE V2-NATIVE CLI PRODUCER: does the v2-exclusive front door compile.
///
/// The three terminations are the same partition its sibling makes and for the same reason. A
/// refusal from the preparation is the subject never having been reached -- the emit refused, the
/// crate would not write, cargo could not run -- and only a completed build with non-zero counters
/// is an observation that did not hold. Collapsing them would report a broken bench as a broken
/// door.
///
/// IT SHARES `prepare_emitted_compiler_for_entry` WITH THE SELF-HOST STEP, parameterised by the
/// entry, so the two instruments cannot disagree about what "emitted and built clean" means. What
/// they do not share is the closure: this one compiles `v2.cli.compile_cli`, which declares
/// `NativeCliDriver` and reaches no part of `v2.compiler.compile`.
fn run_v2_native_cli(source_roots: &[String]) -> InvocationOutcome {
    match cli_run::run_v2_native_cli(source_roots) {
        Ok(held) => InvocationOutcome {
            termination: if held.exit_status == 0 && held.warning_count == 0 {
                Termination::ObservationHeld
            } else {
                Termination::ObservationDidNotHold
            },
            message: format!(
                "v2-native-cli: closure={} binary={} seed={} exit_status={} warning_count={}",
                held.closure_identity,
                held.binary_identity,
                held.seed_identity,
                held.exit_status,
                held.warning_count,
            ),
        },
        Err(cause) => InvocationOutcome {
            termination: Termination::SubjectUnreached,
            message: cause,
        },
    }
}

/// THE DIAGNOSTIC CENSUS PRODUCER. The subject is the whole-tree compile-clean closure, which the
/// census itself derives from `witness_layer_roots` — like the differential's roots, which corpus
/// is measured is the instrument's own fact, not an invocation option.
///
/// The modeled observation (`gunbc.target_binding` `CompileCleanDiagnosticCensusObservation`) is
/// the per-class raw summary with its identity block, and the rendering below mirrors
/// `gunbc.instrument_targets` `compile_clean_diagnostic_census_observation_rendered` line for
/// line. Two host sections ride below it, outside the model, exactly as the differential's
/// parse-wall figures do: the UnlistedImportUse binding-source partition and the per-row
/// worklist, because those are the repair instrument's detail for the one class whose promotion
/// is staged, not per-class census vocabulary. Their next rung is a modeled carrier; until then
/// they are named unmodeled host output, printed rather than dropped.
///
/// THE TERMINATION IS THE MODELED CLASS-SPECIFIC WALL: the RAW UnlistedImportUse count over the
/// unfiltered population is zero (`gunbc.instrument_targets`
/// `compile_clean_diagnostic_census_holds`). Not the all-classes-empty completion target — while
/// the other advisory classes burn down on their own tracks, this instrument holds exactly when
/// UnlistedImportUse reaches zero, and a severity change cannot hide a recurrence because no
/// severity filter stands between emission and the count.
fn run_compile_clean_diagnostic_census() -> InvocationOutcome {
    if let Err(e) = std::env::set_current_dir(cli_run::workspace_root()) {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!(
                "compile-clean-diagnostic-census: refused: could not anchor at the workspace root: {e}"
            ),
        };
    }
    let census = match cli_run::compile_clean_diagnostic_census() {
        Ok(c) => c,
        Err(refusal) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!(
                    "compile-clean-diagnostic-census: refused: {}",
                    cli_run::diagnostic_census_refusal_rendered(&refusal)
                ),
            };
        }
    };
    let unlisted_import_use_raw: usize = census
        .classes
        .iter()
        .filter(|e| e.class_name == "UnlistedImportUse")
        .map(|e| e.diagnostics)
        .sum();
    let mut message = format!(
        "compile-clean-diagnostic-census: closure_modules={} raw_diagnostics={} classes={} unlisted_import_use_raw={}",
        census.closure_modules,
        census.raw_diagnostics,
        census.classes.len(),
        unlisted_import_use_raw,
    );
    message.push_str(&format!(
        "\nidentity source_vector={} compiler={} resolver_policy={} diagnostic_class_schema={} closure={}",
        census.identity.source_vector_digest,
        census.identity.compiler_executable_digest,
        census.identity.resolver_policy_digest,
        census.identity.diagnostic_class_schema_digest,
        census.identity.closure_digest,
    ));
    for entry in &census.classes {
        message.push_str(&format!(
            "\n{} gate={} severity={} diagnostics={} distinct_modules={} distinct_positions={}",
            entry.class_name,
            cli_run::census_gate_tag(&entry.gate),
            cli_run::census_severity_tag(&entry.severity),
            entry.diagnostics,
            entry.distinct_modules,
            entry.distinct_positions,
        ));
    }
    if !census.unlisted_import_rows.is_empty() {
        let mut by_source: std::collections::BTreeMap<&'static str, usize> =
            std::collections::BTreeMap::new();
        for row in &census.unlisted_import_rows {
            *by_source.entry(row.binding_source.as_str()).or_default() += 1;
        }
        message.push_str("\n--- UNLISTED_IMPORT_USE BINDING_SOURCE ---");
        for (source, count) in &by_source {
            message.push_str(&format!("\nSOURCE\t{source}\t{count}"));
        }
        message.push_str(
            "\n--- UNLISTED_IMPORT_USE TSV ---\nfile\tposition\treferenced_name\treferencing_module\tdefiner_module\tbinding_source",
        );
        for row in &census.unlisted_import_rows {
            message.push_str(&format!(
                "\n{}\t{}\t{}\t{}\t{}\t{}",
                row.file,
                row.position,
                row.referenced_name,
                row.referencing_module,
                row.definer_module.as_deref().unwrap_or(""),
                row.binding_source.as_str(),
            ));
        }
    }
    InvocationOutcome {
        termination: if unlisted_import_use_raw == 0 {
            Termination::ObservationHeld
        } else {
            Termination::ObservationDidNotHold
        },
        message,
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

/// THE ONE SEAM: argv operand -> pattern admission -> route. A SINGLE target builds the registry
/// and looks up EXACTLY — no prefix, suffix or "did you mean": a near miss silently running a
/// different target is worse than a refusal naming the one asked for. A SET form delegates to the
/// `.dag` authority `gunbc.compute.test_run` — selection and execution are modeled there, and a
/// host re-implementation would be the second authority this file exists to avoid being.
pub fn test_verb(operand: &str) -> InvocationOutcome {
    let label = match parse_target_pattern(operand) {
        Ok(TargetPattern::SingleTarget(label)) => label,
        Ok(TargetPattern::PackageTargets(_)) | Ok(TargetPattern::SubtreeTargets(_)) => {
            return run_witness_pattern(operand);
        }
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!(
                    "gunbc test: operand is not an admitted target pattern: {operand}\n  cause: {}",
                    target_pattern_refusal_text(&cause)
                ),
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

/// The host the verb runs on, as the short name `dashboard_instance_for_host` is keyed on.
/// `/proc/sys/kernel/hostname` first (no libc buffer sizing), then the POSIX call; an unreadable
/// name is returned as such and the instance lookup refuses it, rather than fabricating a host.
fn short_hostname() -> String {
    if let Ok(name) = std::fs::read_to_string("/proc/sys/kernel/hostname") {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return trimmed.split('.').next().unwrap_or(trimmed).to_string();
        }
    }
    let mut buf = [0u8; 256];
    // SAFETY: `buf` is a live, fully-owned buffer; `gethostname` writes at most `buf.len()`
    // bytes into it and nothing else.
    let rc = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
    if rc == 0 {
        let end = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());
        let name = String::from_utf8_lossy(&buf[..end]).trim().to_string();
        if !name.is_empty() {
            return name.split('.').next().unwrap_or(&name).to_string();
        }
    }
    "hostname-unreadable".to_string()
}

/// THE WITNESS-SET ROUTE: hand the operand to `gunbc.compute.test_run` `test_verb_pattern_cli`
/// and report its ProcessExit in this seam's termination vocabulary. The instance is resolved
/// there by HOST — the verb runs wherever the caller is standing, and the worktree under test is
/// this checkout; the snapshot, selection, per-module claims run and receipt are all the modeled
/// machinery's. The receipt JSON is the message, exactly as the worker flow writes it.
///
/// THE THREE TERMINATIONS map off the modeled exit: success is every selected witness green (or
/// wholly declined), `ExitFailure { code: 1 }` is a run that took readings and found a
/// non-green module — `ObservationDidNotHold` — and any other failure code is a refusal before
/// or around the run (unadmitted pattern shape the mirror already excluded, unknown host,
/// snapshot or receipt failure). A resolve or eval failure of the entry itself is
/// `SubjectUnreached`: the machinery never observed anything.
fn run_witness_pattern(operand: &str) -> InvocationOutcome {
    const ENTRY: &str = "dag/gunbc/compute/test_run.dag";
    const FUNCTION: &str = "gunbc.compute.test_run.test_verb_pattern_cli";
    if let Err(e) = std::env::set_current_dir(cli_run::workspace_root()) {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!("gunbc test: refused: could not anchor at the workspace root: {e}"),
        };
    }
    let roots = cli_run::default_source_roots();
    let (graph, source_indices) = match cli_run::resolve_entry_graph(&roots, ENTRY) {
        Ok(resolved) => resolved,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("gunbc test: resolve failed for {ENTRY}: {cause}"),
            };
        }
    };
    let blocking = crate::v1_compiler_compile::interpreter_blocking_diagnostic_messages(
        graph.diagnostics.clone(),
    );
    if !blocking.is_empty() {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!(
                "gunbc test: {ENTRY} has blocking diagnostics: {}",
                blocking.iter().cloned().collect::<Vec<_>>().join("; ")
            ),
        };
    }
    let worktree = cli_run::workspace_root().to_string_lossy().into_owned();
    let receipt_path = format!("{worktree}/.gunbc-test-receipt.json");
    let arguments = vec![
        (
            Some("host".to_string()),
            crate::v1_interpreter::str_value(short_hostname()),
        ),
        (
            Some("worktree".to_string()),
            crate::v1_interpreter::str_value(worktree),
        ),
        (
            Some("pattern".to_string()),
            crate::v1_interpreter::str_value(operand),
        ),
        (
            Some("receipt_path".to_string()),
            crate::v1_interpreter::str_value(receipt_path.clone()),
        ),
    ];
    let ctx = cli_run::make_eval_context(
        graph.as_ref(),
        source_indices,
        crate::v1_interpreter::ExecutionMode::Wet,
    );
    let value =
        match crate::v1_interpreter::run_in_context_with_args(&ctx, FUNCTION, &arguments, true) {
            Ok(value) => value,
            Err(cause) => {
                return InvocationOutcome {
                    termination: Termination::SubjectUnreached,
                    message: format!("gunbc test: eval failed: {cause}"),
                }
            }
        };
    let exit = cli_run::classify_exit(&value, &ctx);
    let receipt = std::fs::read_to_string(&receipt_path).unwrap_or_default();
    match exit {
        cli_run::ExitClass::Success => InvocationOutcome {
            termination: Termination::ObservationHeld,
            message: receipt,
        },
        cli_run::ExitClass::Failure { code: 1, reason } => InvocationOutcome {
            termination: Termination::ObservationDidNotHold,
            message: if receipt.is_empty() {
                reason.unwrap_or_else(|| "gunbc test: run did not hold".to_string())
            } else {
                receipt
            },
        },
        cli_run::ExitClass::Failure { code: _, reason } => InvocationOutcome {
            termination: Termination::Refused,
            message: reason.unwrap_or_else(|| "gunbc test: refused".to_string()),
        },
        cli_run::ExitClass::NotProcessExit { type_name } => InvocationOutcome {
            termination: Termination::Refused,
            message: format!("gunbc test: {FUNCTION} returned `{type_name}`, not a ProcessExit"),
        },
    }
}
