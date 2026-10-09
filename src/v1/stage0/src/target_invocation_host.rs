// THE HOST REALIZATION OF `gunbc test <target-pattern>`, AND IT IS A HAND MIRROR OF A `.dag`
// AUTHORITY.
//
// The model is `gunbc.target_invocation` (operand admission, generic exact-label route,
// termination vocabulary), `gunbc.instrument_targets` (live target and binding rows, the
// differential's classifier and rendering), `extdeps.bazel.label` (the label grammar mirrored
// here) and `extdeps.bazel.target_pattern` (the pattern grammar mirrored here). The SET forms
// (`//pkg:all`, `//pkg:*`, `//pkg/...`) execute only inside a route's universe: the native test
// route (`//v2/test/...`) or the claim route (`//test/claim/...`, the discovered witness claims,
// evaluated by the floor's own discovery authority and claim evaluation). A set form outside both
// is refused with status 2 (`test_operand_set_form_refusal_rendered`, mirroring
// `gunbc.target_invocation`) and is never widened into another route. None of the
// modeled modules is in the v1 seed's emitted closure — `src/gunbc_cli_dispatch_surface.rs` is
// the only `gunbc.*` mirror the emitter produces — so this file is hand-written beside the
// carrier, as `required_regen_host.rs` mirrors `v2.workflow.required_regen`. The seam is
// therefore MITIGATABLE, not structurally guaranteed: the two can drift until the seam is
// emitted rather than authored. The obligation is enrolled in
// `gunbc.target_invocation_seed_growth`.
//
// WHAT IS AND IS NOT GENERIC HERE. One route: argv operand -> admit pattern -> the operand's
// CONTAINMENT in a route's universe decides the executor. Inside the native one, the native test
// route adjudicates that pattern through the emitted compiler; inside the claim one, the claim
// route evaluates the discovered claims the pattern selects. Outside both, a single target builds
// the registry, exact lookup, invoke the bound producer, render its native standing; a set form
// has no executor and is refused with status 2. No per-instrument arm on that route, and none
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
/// Mirrors `extdeps.bazel.label` `label_refusal_text`: each refusal renders its located cause.
fn label_refusal_text(cause: &LabelRefusal) -> String {
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
        LabelRefusal::TargetPattern(p) => {
            format!("`{p}` is a target PATTERN where a single target was expected")
        }
        LabelRefusal::TargetNameContainsSlash(n) => {
            format!("target name contains `/`, which this subset does not admit: {n}")
        }
    }
}

fn target_pattern_refusal_text(cause: &TargetPatternRefusal) -> String {
    match cause {
        TargetPatternRefusal::PatternNotAbsolute(t) => {
            format!("target pattern is not absolute (must start with //): {t}")
        }
        TargetPatternRefusal::PatternPackageRefused(c) => format!(
            "target pattern names a package the label grammar refuses: {}",
            label_refusal_text(c)
        ),
        TargetPatternRefusal::PatternLabelRefused(c) => format!(
            "target pattern names a single target the label grammar refuses: {}",
            label_refusal_text(c)
        ),
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

/// `gunbc.target_binding` `V2NativeCensusReading`: which census verb and reader a native-census row takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum V2NativeCensusReading {
    ResolveRefusal,
    TypeDeclarationUse,
}

/// `gunbc.target_binding` `TargetProducer`, narrowed to the members this seam realizes today.
/// Adding one is a row in `instrument_registry` and an arm here; it is not a new route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetProducer {
    SelfHost,
    V2NativeCli,
    V2NativeFrontier,
    V2NativeCensus {
        reading: V2NativeCensusReading,
    },
    EmittedCrateWorkspace,
    HeadsReadingDifferential,
    BehavioralReceiptPlan,
    BehavioralReceiptCensus,
    BehavioralReceiptSelftest,
    CompileCleanDiagnosticCensus,
    EvaluationStoreAddressExactHead,
    FloorMemoryQualification,
    TypedGraphExclusiveBytes,
    TypedGraphExclusiveBytesFloorSubject,
    PrimitiveEgressCensus,
    PrimitiveEgressCensusV2,
    PrimitiveEgressCensusDag,
    PrimitiveEgressCensusSeed,
    RequiredLaneResolutionCensus,
    BareReferenceChannelOutcome,
    SelfHostBehavioralEquivalence,
    DependencyDemandCensus,
    GenericIdentityCensus,
    InterpolationHoleCensus,
    RegenRoundCost,
    /// `NativeClaimProgramProducer { entry }`: the entry is carried, so a second program of the same
    /// shape is a registry row naming its entry, never another variant.
    NativeClaimProgram {
        entry: &'static str,
    },
    /// `NativeServeProgramProducer { entry }`: generic over its entry on the same rule.
    NativeServeProgram {
        entry: &'static str,
    },
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

/// `gunbc.instrument_targets` `emitted_crate_workspace_label`: the closure is emitted from the same
/// corpus the self-host step emits from, so the two instruments measure one closure two ways.
fn emitted_crate_workspace_source_roots() -> Vec<String> {
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
            instrument_label("v2-native-frontier"),
            TargetProducer::V2NativeFrontier,
        ),
        (
            instrument_label("v2-native-census"),
            TargetProducer::V2NativeCensus {
                reading: V2NativeCensusReading::ResolveRefusal,
            },
        ),
        (
            instrument_label("type-declaration-use-census"),
            TargetProducer::V2NativeCensus {
                reading: V2NativeCensusReading::TypeDeclarationUse,
            },
        ),
        (
            instrument_label("emitted-crate-workspace"),
            TargetProducer::EmittedCrateWorkspace,
        ),
        (
            instrument_label("native-crypto-vectors"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/native_crypto_vectors.dag",
            },
        ),
        (
            instrument_label("native-app-attest"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/native_app_attest.dag",
            },
        ),
        (
            instrument_label("dag-emit-real-grammar-round-trips"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/dag_emit_real_grammar_round_trips.dag",
            },
        ),
        (
            instrument_label("native-materialization-store-closure"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/native_materialization_store_closure.dag",
            },
        ),
        (
            instrument_label("native-emission-controls"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/native_emission_controls.dag",
            },
        ),
        (
            instrument_label("native-serve"),
            TargetProducer::NativeServeProgram {
                entry: "dag/gunbc/instruments/native_serve_fixture.dag",
            },
        ),
        (
            instrument_label("evaluation-store-address-exact-head"),
            TargetProducer::EvaluationStoreAddressExactHead,
        ),
        (
            instrument_label("floor-memory-qualification"),
            TargetProducer::FloorMemoryQualification,
        ),
        (
            instrument_label("typed-graph-exclusive-bytes"),
            TargetProducer::TypedGraphExclusiveBytes,
        ),
        (
            instrument_label("typed-graph-exclusive-bytes-floor-subject"),
            TargetProducer::TypedGraphExclusiveBytesFloorSubject,
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
        (
            instrument_label("required-lane-resolution-census"),
            TargetProducer::RequiredLaneResolutionCensus,
        ),
        (
            instrument_label("bare-reference-channel-outcome"),
            TargetProducer::BareReferenceChannelOutcome,
        ),
        (
            instrument_label("self-host-behavioral-equivalence"),
            TargetProducer::SelfHostBehavioralEquivalence,
        ),
        (
            instrument_label("dependency-demand-census"),
            TargetProducer::DependencyDemandCensus,
        ),
        (
            instrument_label("generic-identity-census"),
            TargetProducer::GenericIdentityCensus,
        ),
        (
            instrument_label("interpolation-hole-census"),
            TargetProducer::InterpolationHoleCensus,
        ),
        (
            instrument_label("regen-round-cost"),
            TargetProducer::RegenRoundCost,
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
        "heads-reading-differential: compared={} divergent={} occurrence_identity_only={} narrowed={} regressed={} both_refused={}",
        d.modules_compared,
        d.divergent.len(),
        d.occurrence_identity_only.len(),
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
    // DECLARATION-NAME AGREEMENT, printed as host output beside the parse figures rather than
    // folded into the verdict: `HeadsReadingDifferentialObservation` carries four populations and
    // this is a fifth, so it has no home in the modeled standing yet. It is the population a pool
    // name census consumes, which the whole-node `divergent` row cannot isolate. FOLD-IN TRIGGER:
    // the first consumer that decides on this population (a gate, or step 1's reference-edge name
    // index claiming its exactness) lands it as a field of `HeadsReadingDifferentialObservation`
    // with `holds()` requiring it empty; until then it is a reading, not a verdict.
    message.push_str(&format!(
        "\nheads-reading-differential: declaration_names_divergent={}",
        d.declaration_names_divergent.len()
    ));
    for row in d.declaration_names_divergent.iter() {
        message.push_str(&format!("\nheads-reading-differential: NAMES {row}"));
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

// ---------------------------------------------------------------------------------------------
// THE BARE-REFERENCE CHANNEL'S OUTCOME OVER THE FOUR HERMETIC ENTRIES
// ---------------------------------------------------------------------------------------------

/// `gunbc.instrument_targets` `bare_reference_channel_source_roots`. The subject is the
/// instrument's own fact on the rule every sibling here follows, and it is a FIXTURE root rather
/// than the live corpus: the reading must not move when `dag` does.
fn bare_reference_channel_source_roots() -> Vec<String> {
    vec!["fixtures/bare_reference_channel".to_string()]
}

/// `gunbc.target_binding` `BareChannelEligibility`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BareChannelEligibility {
    Runs,
    DisabledByImportLine,
}

/// `gunbc.instrument_targets` `bare_reference_channel_expectations`, mirrored. Each row states the
/// OUTCOME the entry's one isolated condition produces, so a gate that moves flips exactly the row
/// that isolates it.
struct BareChannelExpectation {
    module_path: &'static str,
    isolated_condition: &'static str,
    eligibility: BareChannelEligibility,
    pulled_modules: &'static [&'static str],
}

fn bare_reference_channel_expectations() -> Vec<BareChannelExpectation> {
    vec![
        BareChannelExpectation {
            module_path: "probe.brc.bare_record_consumer",
            isolated_condition: "no imports; bare reference to a RECORD type -- pullable holds",
            eligibility: BareChannelEligibility::Runs,
            pulled_modules: &["probe.brc.record_home"],
        },
        BareChannelExpectation {
            module_path: "probe.brc.bare_alias_consumer",
            isolated_condition:
                "no imports; bare reference to a nullary type alias -- every arm of pullable declines",
            eligibility: BareChannelEligibility::Runs,
            pulled_modules: &[],
        },
        BareChannelExpectation {
            module_path: "probe.brc.imported_record_consumer",
            isolated_condition:
                "the row-one reference plus ONE unrelated import line -- gate one turns the channel off",
            eligibility: BareChannelEligibility::DisabledByImportLine,
            pulled_modules: &[],
        },
        BareChannelExpectation {
            module_path: "probe.brc.transitive_alias_consumer",
            isolated_condition:
                "no imports; a bare CALL whose callee imports the alias home -- the alias arrives as a passenger",
            eligibility: BareChannelEligibility::Runs,
            pulled_modules: &["probe.brc.alias_home", "probe.brc.passenger_home"],
        },
    ]
}

fn bare_channel_eligibility_rendered(e: BareChannelEligibility) -> &'static str {
    match e {
        BareChannelEligibility::Runs => "RUNS",
        BareChannelEligibility::DisabledByImportLine => "DISABLED",
    }
}

/// `gunbc.instrument_targets` `bare_reference_channel_holds`, mirrored: set equality at identity
/// grain, never a count, and eligibility compared as its own field so an ineligible channel and an
/// eligible one that pulled nothing can never satisfy each other's row.
fn run_bare_reference_channel_outcome() -> InvocationOutcome {
    let source_roots = bare_reference_channel_source_roots();
    let missing: Vec<String> = source_roots
        .iter()
        .filter(|r| !std::path::Path::new(r.as_str()).exists())
        .cloned()
        .collect();
    if !missing.is_empty() {
        return InvocationOutcome {
            termination: Termination::SubjectUnreached,
            message: format!(
                "bare-reference-channel-outcome: subject unreached (fixture source root absent): {}",
                missing.join(", ")
            ),
        };
    }
    let expectations = bare_reference_channel_expectations();
    let entries: Vec<String> = expectations
        .iter()
        .map(|e| e.module_path.to_string())
        .collect();
    let readings = match cli_run::bare_reference_channel_readings(&source_roots, &entries) {
        Ok(readings) => readings,
        Err(detail) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!(
                    "bare-reference-channel-outcome: subject unreached (module index refused): {detail}"
                ),
            };
        }
    };
    let mut lines: Vec<String> = Vec::new();
    let mut unmet: Vec<String> = Vec::new();
    for expected in expectations.iter() {
        let matched: Vec<&cli_run::BareReferenceChannelEntryReading> = readings
            .iter()
            .filter(|r| r.module_path == expected.module_path)
            .collect();
        let [observed] = matched.as_slice() else {
            unmet.push(format!(
                "bare-reference-channel-outcome: UNMET {} ({}): expected exactly one reading, got {}",
                expected.module_path,
                expected.isolated_condition,
                matched.len()
            ));
            continue;
        };
        let observed_eligibility = if observed.bare_channel_eligible {
            BareChannelEligibility::Runs
        } else {
            BareChannelEligibility::DisabledByImportLine
        };
        let expected_pulled: Vec<String> = {
            let mut v: Vec<String> = expected
                .pulled_modules
                .iter()
                .map(|m| (*m).to_string())
                .collect();
            v.sort();
            v
        };
        lines.push(format!(
            "bare-reference-channel-outcome: {} channel={} pulled=[{}]",
            observed.module_path,
            bare_channel_eligibility_rendered(observed_eligibility),
            observed.pulled_modules.join(", ")
        ));
        if observed_eligibility != expected.eligibility
            || observed.pulled_modules != expected_pulled
        {
            unmet.push(format!(
                "bare-reference-channel-outcome: UNMET {} ({}): expected channel={} pulled=[{}], observed channel={} pulled=[{}]",
                expected.module_path,
                expected.isolated_condition,
                bare_channel_eligibility_rendered(expected.eligibility),
                expected_pulled.join(", "),
                bare_channel_eligibility_rendered(observed_eligibility),
                observed.pulled_modules.join(", "),
            ));
        }
    }
    let mut message = format!(
        "bare-reference-channel-outcome: entries={} unmet={}",
        // THE READINGS, NOT THE EXPECTATIONS, because that is what the `.dag` authority renders
        // (`gunbc.instrument_targets` `bare_reference_channel_standing_rendered` counts the
        // readings). The two cannot differ today -- one reading is demanded per expectation -- so
        // this is not a defect being fixed but a drift point being closed, of exactly the kind
        // `gunbc.bare_reference_channel_outcome_seed_growth` enrolls this mirror for.
        readings.len(),
        unmet.len()
    );
    for line in lines.iter().chain(unmet.iter()) {
        message.push('\n');
        message.push_str(line);
    }
    InvocationOutcome {
        termination: if unmet.is_empty() {
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
        TargetProducer::V2NativeFrontier => run_v2_native_frontier(&self_host_source_roots()),
        TargetProducer::V2NativeCensus {
            reading: V2NativeCensusReading::ResolveRefusal,
        } => run_v2_native_census(&self_host_source_roots()),
        TargetProducer::V2NativeCensus {
            reading: V2NativeCensusReading::TypeDeclarationUse,
        } => run_type_declaration_use_census(&self_host_source_roots()),
        TargetProducer::NativeClaimProgram { entry } => run_native_claim_program(entry),
        TargetProducer::NativeServeProgram { entry } => run_native_serve_program(entry),
        TargetProducer::EmittedCrateWorkspace => {
            run_emitted_crate_workspace(&emitted_crate_workspace_source_roots())
        }
        TargetProducer::EvaluationStoreAddressExactHead => {
            run_evaluation_store_address_exact_head()
        }
        TargetProducer::FloorMemoryQualification => run_floor_memory_qualification(),
        TargetProducer::TypedGraphExclusiveBytes => run_typed_graph_exclusive_bytes(),
        TargetProducer::TypedGraphExclusiveBytesFloorSubject => {
            run_typed_graph_exclusive_bytes_floor_subject()
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
        TargetProducer::BareReferenceChannelOutcome => run_bare_reference_channel_outcome(),
        TargetProducer::DependencyDemandCensus => {
            run_dependency_demand_census(&self_host_source_roots())
        }
        TargetProducer::GenericIdentityCensus => {
            run_generic_identity_census(&self_host_source_roots())
        }
        TargetProducer::InterpolationHoleCensus => {
            run_interpolation_hole_census(&self_host_source_roots())
        }
        TargetProducer::RegenRoundCost => run_regen_round_cost_instrument(),
        TargetProducer::SelfHostBehavioralEquivalence => run_cli_wire_census(
            "self-host-behavioral-equivalence",
            "dag/gunbc/instruments/self_host_behavioral_equivalence_take.dag",
            "take_self_host_behavioral_equivalence_receipt",
        ),
        TargetProducer::RequiredLaneResolutionCensus => run_cli_wire_census(
            "required-lane-resolution-census",
            "dag/gunbc/required_lane_resolution_census_live.dag",
            "required_lane_resolution_census_exit",
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
    run_cli_wire_census(
        label,
        "dag/gunbc/primitive_egress/census_live.dag",
        function,
    )
}

/// ONE RUNNER FOR EVERY INSTRUMENT WHOSE `.dag` ENTRY ANSWERS A `CliWireResponse`: resolve the
/// entry, evaluate the named function, print the wire bytes on every termination, and map the
/// wire exit to a Termination through the one classifier in `cli_run`. The primitive egress
/// census labels and the required-lane resolution census share it; a new instrument of that
/// shape is a label row plus one arm naming its entry and function, never a second runner.
fn run_cli_wire_census(
    label: &'static str,
    entry: &'static str,
    function: &'static str,
) -> InvocationOutcome {
    let label_name = label;
    let function_name = function;
    if let Err(e) = std::env::set_current_dir(cli_run::workspace_root()) {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!("{label_name}: refused: could not anchor at the workspace root: {e}"),
        };
    }
    let roots = cli_run::default_source_roots();
    let (graph, source_indices) = match cli_run::resolve_entry_graph(&roots, entry) {
        Ok(resolved) => resolved,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label_name}: resolve failed for {entry}: {cause}"),
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
                "{label_name}: {entry} has blocking diagnostics: {}",
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
        Ok(held) => {
            // THE TERMINATION IS DERIVED FROM THE CAUSE, so a did-not-hold cannot exit 1 without
            // printing why: the cause IS the discriminator (DESIGN §5).
            let not_clean = cli_run::emitted_build_not_clean_cause(
                "SELF-HOST",
                held.exit_status,
                held.warning_count,
                &held.warning_headers,
            );
            let termination = if not_clean.is_none() {
                Termination::ObservationHeld
            } else {
                Termination::ObservationDidNotHold
            };
            let counters = format!(
                "self-host v1->v2: closure={} binary={} seed={} exit_status={} warning_count={} \
                 door_refusal_reason=\"{}\"",
                held.closure_identity,
                held.binary_identity,
                held.seed_identity,
                held.exit_status,
                held.warning_count,
                held.door_refusal_reason,
            );
            InvocationOutcome {
                termination,
                message: match not_clean {
                    Some(cause) => format!("{cause}\n{counters}"),
                    None => counters,
                },
            }
        }
        Err(cause) => InvocationOutcome {
            termination: Termination::SubjectUnreached,
            message: cause,
        },
    }
}

/// THE NATIVE FRONTIER PRODUCER: did one complete native run keep the debt
/// `gunbc.native_frontier_roster` records. The verdict is `gunbc.native_frontier_ratchet`'s, decided
/// inside the emitted binary; this arm only maps its word to a termination, and that map is closed:
/// an unknown word is a harness defect, never a pass.
///
/// `held` and `advanced` are the observation holding: every planned identity reached a terminal
/// verdict and every honest failure is rostered debt. An advance also prints a proposed smaller
/// roster, which a reviewed pull request may carry (the required native-route lane publishes it). An owned
/// correctness flip (`grew-by-owned-correctness-flip`) holds for the same reason: every added
/// identity is owed debt under a declared, owned cause, and it too prints a proposed roster.
/// (v1 PURPOSE admission, `gunbc.v1_maintenance_standing`: this arm only maps a v2 frontier
/// verdict word to its termination; the verdict itself is decided in `.dag`, so no seed growth.)
/// `lost` and `unminted` are the observation
/// not holding. `unminted` is a complete run with nothing to hold it to, and an empty roster read as
/// no debt would be a vacuous pass. `not-a-measurement` means the receipt failed an integrity
/// clause or the pattern was narrower than the universe, so the subject was not reached.
fn run_v2_native_frontier(source_roots: &[String]) -> InvocationOutcome {
    let pattern = native_route_default_pattern_text();
    match cli_run::run_v2_native_frontier(source_roots, &pattern) {
        Ok(run) => {
            let termination = match run.frontier.as_str() {
                "held" | "advanced" | "grew-by-owned-correctness-flip" => Termination::ObservationHeld,
                "lost" | "unminted" => Termination::ObservationDidNotHold,
                "not-a-measurement" => Termination::SubjectUnreached,
                other => {
                    return InvocationOutcome {
                        termination: Termination::SubjectUnreached,
                        message: format!(
                            "v2-native-frontier: the emitted binary reported an unknown frontier word {other:?}; \
                             gunbc.native_frontier_ratchet native_frontier_verdict_word and this match must agree"
                        ),
                    }
                }
            };
            InvocationOutcome {
                termination,
                message: format!(
                    "v2-native-frontier: frontier={} (lane qualification: {}); findings and any proposed \
                     roster are the [native-frontier] and [native-frontier-roster] lines above",
                    run.frontier, run.admission_summary
                ),
            }
        }
        Err(cause) => InvocationOutcome {
            termination: Termination::SubjectUnreached,
            message: cause,
        },
    }
}

/// THE V2-NATIVE CENSUS PRODUCER. A report: a completed census is held, a run that did not complete
/// is the subject unreached, and there is deliberately no did-not-hold arm because no red of the
/// census is authorable in a real run (review 74324; `gunbc.instrument_targets`
/// `v2_native_census_label`).
fn run_v2_native_census(source_roots: &[String]) -> InvocationOutcome {
    match cli_run::run_v2_native_census(source_roots) {
        Ok(run) => InvocationOutcome {
            termination: Termination::ObservationHeld,
            message: format!(
                "v2-native-census: modules={} file_refusals={} advised_files={} residual_rows={} \
                 cause_groups={}; the rows grouped by fatal reason are the cause_group lines above",
                run.modules,
                run.file_refusals,
                run.advised_files,
                run.residual_rows,
                run.cause_groups
            ),
        },
        Err(cause) => InvocationOutcome {
            termination: Termination::SubjectUnreached,
            message: cause,
        },
    }
}

/// `gunbc.instruments.type_declaration_use_census_reading` `TypeCensusFixtureFile` rows, read off the
/// reader's own list: a value of any other shape refuses rather than writing a partial root.
fn fixture_file_rows(
    ctx: &crate::v1_interpreter::InterpContext,
    files: &crate::v1_interpreter::Value,
) -> Result<Vec<(String, String)>, String> {
    use crate::v1_interpreter::Value;
    let Value::List(rows) = files else {
        return Err("type-declaration-use-census: the fixture rows are not a list".to_string());
    };
    rows.iter()
        .map(|row| {
            let Value::Record { fields, .. } = row else {
                return Err(
                    "type-declaration-use-census: a fixture row is not a record".to_string()
                );
            };
            match (ctx.field(fields, "path"), ctx.field(fields, "content")) {
                (Some(Value::Str(path)), Some(Value::Str(content))) => {
                    Ok((path.to_string(), content.to_string()))
                }
                _ => Err(
                    "type-declaration-use-census: a fixture row lacks a string path or content"
                        .to_string(),
                ),
            }
        })
        .collect()
}

/// THE TYPE-DECLARATION-USE CENSUS PRODUCER (the nominal-type plan's M0). The host writes the two
/// control roots from the reader's own rows (`type_census_fixture_files`,
/// `type_census_unindexed_files`), runs the emitted compiler's `census-infer` over them and over the
/// whole tree, and hands all three to `type_declaration_use_census_standing`, which decides the
/// standing and filters the affected modules by the required gate. The host decides nothing.
fn run_type_declaration_use_census(source_roots: &[String]) -> InvocationOutcome {
    let label_name = "type-declaration-use-census";
    if let Err(e) = std::env::set_current_dir(cli_run::workspace_root()) {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!("{label_name}: refused: could not anchor at the workspace root: {e}"),
        };
    }
    const READER: &str = "dag/gunbc/instruments/type_declaration_use_census_reading.dag";
    let roots = cli_run::default_source_roots();
    let (graph, source_indices) = match cli_run::resolve_entry_graph(&roots, READER) {
        Ok(resolved) => resolved,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label_name}: resolve failed for {READER}: {cause}"),
            };
        }
    };
    let ctx = cli_run::make_eval_context(
        graph.as_ref(),
        source_indices,
        crate::v1_interpreter::ExecutionMode::Wet,
    );
    let scratch = std::path::Path::new("target").join("type-declaration-use-census");
    let mut write_root = |name: &str, function: &str| -> Result<String, String> {
        let root = scratch.join(name);
        let _ = std::fs::remove_dir_all(&root);
        let files = crate::v1_interpreter::run_in_context_with_args(&ctx, function, &[], true)
            .map_err(|cause| format!("{label_name}: {function} failed: {cause}"))?;
        for (path, content) in fixture_file_rows(&ctx, &files)? {
            let target = root.join(&path);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("{label_name}: {e}"))?;
            }
            std::fs::write(&target, content).map_err(|e| format!("{label_name}: {e}"))?;
        }
        Ok(root.to_string_lossy().into_owned())
    };
    let roots_written = write_root("fixture", "type_census_fixture_files")
        .and_then(|f| write_root("unindexed", "type_census_unindexed_files").map(|u| (f, u)));
    let (fixture_root, unindexed_root) = match roots_written {
        Ok(pair) => pair,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: cause,
            }
        }
    };
    let runs = match cli_run::run_type_declaration_use_census_runs(
        source_roots,
        &fixture_root,
        &unindexed_root,
    ) {
        Ok(runs) => runs,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: cause,
            }
        }
    };
    let status = |s: Option<i32>| crate::v1_interpreter::Value::Int(i64::from(s.unwrap_or(-1)));
    let args = [
        (
            Some("fixture_stdout".to_string()),
            crate::v1_interpreter::Value::Str(runs.fixture.stdout.clone().into()),
        ),
        (
            Some("fixture_status".to_string()),
            status(runs.fixture.status),
        ),
        (
            Some("unindexed_status".to_string()),
            status(runs.unindexed.status),
        ),
        (
            Some("tree_stdout".to_string()),
            crate::v1_interpreter::Value::Str(runs.tree.stdout.clone().into()),
        ),
        (Some("tree_status".to_string()), status(runs.tree.status)),
    ];
    let value = match crate::v1_interpreter::run_in_context_with_args(
        &ctx,
        "type_declaration_use_census_standing",
        &args,
        true,
    ) {
        Ok(value) => value,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label_name}: the reader failed: {cause}"),
            }
        }
    };
    match cli_run::classify_cli_wire(&value, &ctx) {
        cli_run::CliWireClass::Printable { bytes, exit } => {
            let (termination, verdict) = match exit {
                cli_run::ExitClass::Success => (Termination::ObservationHeld, "held".to_string()),
                cli_run::ExitClass::Failure { code: 1, reason } => (
                    Termination::ObservationDidNotHold,
                    reason.unwrap_or_else(|| "not held".to_string()),
                ),
                cli_run::ExitClass::Failure { reason, .. } => (
                    Termination::SubjectUnreached,
                    reason.unwrap_or_else(|| "no observation".to_string()),
                ),
                cli_run::ExitClass::NotProcessExit { type_name } => (
                    Termination::Refused,
                    format!("the reader returned `{type_name}`, not a ProcessExit"),
                ),
            };
            InvocationOutcome { termination, message: format!("{bytes}{label_name}: {verdict}") }
        }
        cli_run::CliWireClass::Unprintable { cause } => InvocationOutcome {
            termination: Termination::Refused,
            message: format!("{label_name}: renderer refused: {cause}"),
        },
        cli_run::CliWireClass::NotCliWire { type_name } => InvocationOutcome {
            termination: Termination::Refused,
            message: format!(
                "{label_name}: type_declaration_use_census_standing returned `{type_name}`, not a CliWireResponse"
            ),
        },
        cli_run::CliWireClass::MalformedCliWire { detail } => InvocationOutcome {
            termination: Termination::Refused,
            message: format!("{label_name}: malformed CliWireResponse: {detail}"),
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
/// entry, so the two instruments cannot disagree about what "emitted and built clean" means — and
/// since that preparation now establishes its own discriminating red by mutation, neither subject
/// can report a green cargo verdict that is not a function of the emitted bytes. What they do not
/// share is the closure: this one compiles `v2.cli.compile_cli`, which declares `NativeCliDriver`
/// and reaches no part of `v2.compiler.compile`.
///
/// WHAT IT ADDS THAT ITS SIBLING DOES NOT: the built artifact's ENTRYPOINT IS EXECUTED. The
/// self-host step's binary is the SourceRootEvalDriver, whose entrypoint the operator-invoked
/// `--required-v2-native` route already spawns in both its modes; this one's was spawned by
/// nothing until `walk_cli_door` ran it, so the instrument admitted a door that compiled and was
/// never opened.
fn run_v2_native_cli(source_roots: &[String]) -> InvocationOutcome {
    match cli_run::run_v2_native_cli(source_roots) {
        Ok(held) => {
            // THE TERMINATION IS DERIVED FROM THE CAUSE, so a did-not-hold cannot exit 1 without
            // printing why: the cause IS the discriminator (DESIGN §5).
            let not_clean = cli_run::emitted_build_not_clean_cause(
                "V2-NATIVE-CLI",
                held.exit_status,
                held.warning_count,
                &held.warning_headers,
            );
            let termination = if not_clean.is_none() {
                Termination::ObservationHeld
            } else {
                Termination::ObservationDidNotHold
            };
            let counters = format!(
                "v2-native-cli: closure={} binary={} seed={} exit_status={} warning_count={} \
                 door_exit_status={} door_emitted_bytes={} door_refusal_exit_status={} \
                 generation_one_executable={}",
                held.closure_identity,
                held.binary_identity,
                held.seed_identity,
                held.exit_status,
                held.warning_count,
                held.door_exit_status,
                held.door_emitted_bytes,
                held.door_refusal_exit_status,
                held.generation_one_executable,
            );
            InvocationOutcome {
                termination,
                message: match not_clean {
                    Some(cause) => format!("{cause}\n{counters}"),
                    None => counters,
                },
            }
        }
        Err(cause) => InvocationOutcome {
            termination: Termination::SubjectUnreached,
            message: cause,
        },
    }
}

/// THE EMITTED-WORKSPACE PRODUCER (Pkg7e). SubjectUnreached when the emission, the plan, the render
/// or either cargo run could not be reached or the red could not be attributed; the observation
/// holds only when the derived partition built clean AND the dropped-dependency red refused with an
/// error naming the dropped target module. Every refusal carries its typed cause from the runner.
fn run_emitted_crate_workspace(source_roots: &[String]) -> InvocationOutcome {
    match cli_run::run_emitted_crate_workspace(source_roots) {
        Ok(held) => InvocationOutcome {
            termination: if held.green_exit_status == 0 && held.green_warning_count == 0 {
                Termination::ObservationHeld
            } else {
                Termination::ObservationDidNotHold
            },
            message: format!(
                "emitted-crate-workspace: head={} closure_modules={} crates={}                  facade_is_whole_closure={} rustc={} green_exit_status={} green_warning_count={}                  red=drop {} from {} ({} -> {}) red_refused=\"{}\"",
                held.head,
                held.closure_modules,
                held.crate_count,
                held.facade_is_whole_closure,
                held.rustc_identity,
                held.green_exit_status,
                held.green_warning_count,
                held.red_dropped_package,
                held.red_package,
                held.red_from_module,
                held.red_to_module,
                held.red_diagnostic,
            ),
        },
        Err(cause) => InvocationOutcome {
            termination: Termination::SubjectUnreached,
            message: cause,
        },
    }
}

/// THE NATIVE CLAIM PROGRAM PRODUCER: emit, build and run a `NativeClaimDriver` entry, then let the
/// `.dag` reader decide what the run established. The host decides nothing about the cases -- it
/// hands the program's stdout and status to `gunbc.native_claim_program`
/// `native_claim_program_standing` and maps that fold's `ProcessExit` through the one classifier
/// (`cli_run::classify_exit`): success is held, code 1 is an observation that did not hold, and code
/// 2 -- a roster/row join that breaks, a status the rows do not support, a signal -- is no
/// observation. The source roots are the instrument's own fact, as for every sibling here.
fn run_native_claim_program(entry: &'static str) -> InvocationOutcome {
    let label_name = "native-claim";
    if let Err(e) = std::env::set_current_dir(cli_run::workspace_root()) {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!("{label_name}: refused: could not anchor at the workspace root: {e}"),
        };
    }
    let run = match cli_run::run_native_claim_program(&v2_native_cli_source_roots(), entry) {
        Ok(run) => run,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: cause,
            }
        }
    };
    const READER: &str = "dag/gunbc/native_claim_program.dag";
    let roots = cli_run::default_source_roots();
    let (graph, source_indices) = match cli_run::resolve_entry_graph(&roots, READER) {
        Ok(resolved) => resolved,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label_name}: resolve failed for {READER}: {cause}"),
            };
        }
    };
    let ctx = cli_run::make_eval_context(
        graph.as_ref(),
        source_indices,
        crate::v1_interpreter::ExecutionMode::Wet,
    );
    let status = i64::from(run.status.unwrap_or(-1));
    let args = [
        (
            Some("stdout".to_string()),
            crate::v1_interpreter::Value::Str(run.stdout.clone().into()),
        ),
        (
            Some("status".to_string()),
            crate::v1_interpreter::Value::Int(status),
        ),
    ];
    let standing = match crate::v1_interpreter::run_in_context_with_args(
        &ctx,
        "native_claim_program_standing",
        &args,
        true,
    ) {
        Ok(value) => cli_run::classify_exit(&value, &ctx),
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label_name}: the report reader failed: {cause}"),
            }
        }
    };
    let (termination, verdict) = match standing {
        cli_run::ExitClass::Success => (Termination::ObservationHeld, "held".to_string()),
        cli_run::ExitClass::Failure { code: 1, reason } => (
            Termination::ObservationDidNotHold,
            reason.unwrap_or_else(|| "not held".to_string()),
        ),
        cli_run::ExitClass::Failure { reason, .. } => (
            Termination::SubjectUnreached,
            reason.unwrap_or_else(|| "no observation".to_string()),
        ),
        cli_run::ExitClass::NotProcessExit { type_name } => (
            Termination::Refused,
            format!("the report reader returned `{type_name}`, not a ProcessExit"),
        ),
    };
    InvocationOutcome {
        termination,
        message: format!(
            "{}{label_name}: entry={entry} closure={} binary={} seed={} warning_count={} status={status} -- {verdict}\n{}",
            run.stdout,
            run.closure_identity,
            run.binary_identity,
            run.seed_identity,
            run.warning_count,
            run.stderr.trim_end(),
        ),
    }
}

/// THE NATIVE SERVE PROGRAM PRODUCER: emit, build and start a `NativeServeDriver` entry, exchange
/// the requests `gunbc.native_serve_probe` names over real TCP, then let that module decide what the
/// run established. The host decides nothing about a case: the requests and both launch revisions
/// are read from the reader module, the bytes go back to `native_serve_probe_standing` unjudged, and
/// its `ProcessExit` maps through the one classifier exactly as the claim producer's does.
fn run_native_serve_program(entry: &'static str) -> InvocationOutcome {
    let label_name = "native-serve";
    if let Err(e) = std::env::set_current_dir(cli_run::workspace_root()) {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!("{label_name}: refused: could not anchor at the workspace root: {e}"),
        };
    }
    const READER: &str = "dag/gunbc/native_serve_probe.dag";
    let roots = cli_run::default_source_roots();
    let (graph, source_indices) = match cli_run::resolve_entry_graph(&roots, READER) {
        Ok(resolved) => resolved,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label_name}: resolve failed for {READER}: {cause}"),
            };
        }
    };
    let ctx = cli_run::make_eval_context(
        graph.as_ref(),
        source_indices,
        crate::v1_interpreter::ExecutionMode::Wet,
    );
    let read = |function: &str| -> Result<crate::v1_interpreter::Value, String> {
        crate::v1_interpreter::run_in_context_with_args(&ctx, function, &[], true)
            .map_err(|cause| format!("{label_name}: {READER} {function} failed: {cause}"))
    };
    let text = |value: &crate::v1_interpreter::Value| -> Option<String> {
        match value {
            crate::v1_interpreter::Value::Str(s) => Some(s.to_string()),
            _ => None,
        }
    };
    let plan = (|| -> Result<(Vec<String>, String, String, String), String> {
        let requests = match &read("native_serve_probe_requests")? {
            crate::v1_interpreter::Value::List(items) => items
                .iter()
                .map(|item| text(item).ok_or("a request is not a String".to_string()))
                .collect::<Result<Vec<String>, String>>()?,
            _ => return Err("native_serve_probe_requests is not a List".to_string()),
        };
        let release = text(&read("native_serve_probe_release_revision")?)
            .ok_or("the release revision is not a String")?;
        let refused = text(&read("native_serve_probe_refused_revision")?)
            .ok_or("the refused revision is not a String")?;
        let deadline = text(&read("native_serve_probe_request_deadline_ms")?)
            .ok_or("the request deadline is not a String")?;
        Ok((requests, release, refused, deadline))
    })();
    let (requests, release, refused, deadline) = match plan {
        Ok(plan) => plan,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label_name}: the probe plan is unreadable: {cause}"),
            }
        }
    };
    let run = match cli_run::run_native_serve_program(
        &v2_native_cli_source_roots(),
        entry,
        &release,
        &refused,
        &deadline,
        &requests,
    ) {
        Ok(run) => run,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: cause,
            }
        }
    };
    let refused_status = i64::from(run.refused_status.unwrap_or(-1));
    let args = [
        (
            Some("announcement".to_string()),
            crate::v1_interpreter::Value::Str(run.announcement.clone().into()),
        ),
        (
            Some("responses".to_string()),
            crate::v1_interpreter::Value::List(std::rc::Rc::new(
                run.responses
                    .iter()
                    .map(|r| crate::v1_interpreter::Value::Str(r.clone().into()))
                    .collect::<Vec<_>>()
                    .into(),
            )),
        ),
        (
            Some("refused_status".to_string()),
            crate::v1_interpreter::Value::Int(refused_status),
        ),
        (
            Some("refused_stderr".to_string()),
            crate::v1_interpreter::Value::Str(run.refused_stderr.clone().into()),
        ),
        (
            Some("served_exit_status".to_string()),
            crate::v1_interpreter::Value::Int(i64::from(run.served_exit_status.unwrap_or(-1))),
        ),
    ];
    let standing = match crate::v1_interpreter::run_in_context_with_args(
        &ctx,
        "native_serve_probe_standing",
        &args,
        true,
    ) {
        Ok(value) => cli_run::classify_exit(&value, &ctx),
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label_name}: the probe reader failed: {cause}"),
            }
        }
    };
    let (termination, verdict) = match standing {
        cli_run::ExitClass::Success => (Termination::ObservationHeld, "held".to_string()),
        cli_run::ExitClass::Failure { code: 1, reason } => (
            Termination::ObservationDidNotHold,
            reason.unwrap_or_else(|| "not held".to_string()),
        ),
        cli_run::ExitClass::Failure { reason, .. } => (
            Termination::SubjectUnreached,
            reason.unwrap_or_else(|| "no observation".to_string()),
        ),
        cli_run::ExitClass::NotProcessExit { type_name } => (
            Termination::Refused,
            format!("the probe reader returned `{type_name}`, not a ProcessExit"),
        ),
    };
    InvocationOutcome {
        termination,
        message: format!(
            "{label_name}: entry={entry} closure={} binary={} seed={} warning_count={} requests={} refused_status={refused_status} -- {verdict}\n{}\n{}",
            run.closure_identity,
            run.binary_identity,
            run.seed_identity,
            run.warning_count,
            run.responses.len(),
            run.announcement,
            run.stderr.trim_end(),
        ),
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

/// `extdeps.bazel.target_pattern` `target_pattern_package`, mirrored.
fn target_pattern_package(pattern: &TargetPattern) -> &[String] {
    match pattern {
        TargetPattern::SingleTarget(label) => &label.package_segments,
        TargetPattern::PackageTargets(package) => package,
        TargetPattern::SubtreeTargets(package) => package,
    }
}

/// `extdeps.bazel.target_pattern` `package_within`, mirrored: `a` is `b` or lies under it.
fn package_within(a: &[String], b: &[String]) -> bool {
    b.len() <= a.len() && b.iter().zip(a.iter()).all(|(outer, inner)| outer == inner)
}

/// `extdeps.bazel.target_pattern` `target_pattern_within`, mirrored arm for arm, including the
/// subtree-inside-package arm that answers `false` rather than claiming a containment this side
/// cannot establish either.
fn target_pattern_within(inner: &TargetPattern, outer: &TargetPattern) -> bool {
    match outer {
        TargetPattern::SubtreeTargets(package) => {
            package_within(target_pattern_package(inner), package)
        }
        TargetPattern::PackageTargets(package) => match inner {
            TargetPattern::SingleTarget(label) => &label.package_segments == package,
            TargetPattern::PackageTargets(inner_package) => inner_package == package,
            TargetPattern::SubtreeTargets(_) => false,
        },
        TargetPattern::SingleTarget(label) => match inner {
            TargetPattern::SingleTarget(inner_label) => inner_label == label,
            TargetPattern::PackageTargets(_) | TargetPattern::SubtreeTargets(_) => false,
        },
    }
}

/// `extdeps.bazel.target_pattern` `render_target_pattern`, mirrored. This is what reaches the
/// emitted binary's `adjudicate` operand, so the pattern the operator wrote and the pattern the
/// native universe is selected by are one value rendered once, never two spellings.
fn render_target_pattern(pattern: &TargetPattern) -> String {
    match pattern {
        TargetPattern::SingleTarget(label) => render_label(label),
        TargetPattern::PackageTargets(package) => format!("//{}:all", package.join("/")),
        TargetPattern::SubtreeTargets(package) => {
            if package.is_empty() {
                "//...".to_string()
            } else {
                format!("//{}/...", package.join("/"))
            }
        }
    }
}

/// `gunbc.witness_v2_native_route` `native_route_default_pattern`, mirrored: the native test
/// route's whole universe, `//v2/test/...`. An operand CONTAINED in it belongs to that route.
///
/// THIS IS THE ONLY SEED-SIDE SPELLING OF THAT UNIVERSE, and it is `pub` for exactly that reason.
/// The lane's runner needs the same fact as TEXT (it is an argv word) and this verb needs it as a
/// PATTERN, which is two renderings of one value, not two values. A bare `"//v2/test/..."` literal
/// beside this one would be the second independently editable spelling DESIGN section 3 forbids,
/// and the drift would be load-bearing rather than cosmetic: narrow one and not the other and the
/// verb routes an operand native that the lane's default excludes, or the reverse.
pub fn native_route_default_pattern() -> TargetPattern {
    TargetPattern::SubtreeTargets(vec!["v2".to_string(), "test".to_string()])
}

/// The universe above as the argv word the emitted binary's `adjudicate` verb takes. Derived from
/// the one pattern rather than spelled again.
pub fn native_route_default_pattern_text() -> String {
    render_target_pattern(&native_route_default_pattern())
}

/// THE NATIVE TEST ROUTE, ENTERED WITH THE OPERAND'S OWN PATTERN.
///
/// The seed emits the compiler closure, cargo builds it, and the EMITTED BINARY adjudicates the
/// selected population. Nothing on this path consults the interpreter, and nothing substitutes a
/// cached or seed-side answer.
///
/// THE TERMINATION IS THE OPERAND'S OWN, FOLDED FROM THE PER-IDENTITY ROWS. Every arm of this
/// function used to answer `SubjectUnreached`, because the only bit the route returned was the
/// LANE'S whole-route qualification and consuming that as the operand's verdict conflates two
/// scopes in both directions -- a typo reported as a failing test, and worse, a held qualification
/// reported as the operator's tests PASSING. That class is rostered as
/// `gunbc.recurring_failure_mode` `route_scoped_qualification_read_as_one_members_verdict`.
///
/// The route now folds the population it actually emitted through
/// `gunbc.instrument_targets` `native_route_member_termination`, mirrored in the runner, so this
/// caller receives a MEMBER-SCOPED standing and reads it directly. The lane's qualification is
/// still carried and still reported, because a route-integrity refusal is something an operator
/// needs to see -- but it no longer decides this verb's exit status.
///
/// THE LANE'S QUALIFICATION GATES A PASS, AND THE SUMMARY RIDES ON EVERY ARM. An earlier shape of
/// this function said "only the member one decides the status", which over-corrected the original
/// conflation: ruling out "qualification IS the verdict" never licensed "qualification may be
/// IGNORED", and a refused route beside an all-passing population answered exit 0 with the refusal
/// demoted to a sentence (review 70107). A refused qualification now clamps a `held` to
/// `SubjectUnreached` while letting a definite member failure through unchanged.
fn run_native_test_route(pattern: &TargetPattern) -> InvocationOutcome {
    let rendered = render_target_pattern(pattern);
    match cli_run::run_required_v2_native(&self_host_source_roots(), &rendered) {
        cli_run::NativeRouteOutcome::LaneQualificationHeld { summary, members } => {
            native_member_outcome(
                &rendered,
                members,
                true,
                "the lane's qualification held",
                &summary,
            )
        }
        cli_run::NativeRouteOutcome::LaneQualificationRefused { summary, members } => {
            native_member_outcome(
                &rendered,
                members,
                false,
                "the lane's qualification REFUSED",
                &summary,
            )
        }
        cli_run::NativeRouteOutcome::Unreached { cause } => InvocationOutcome {
            termination: Termination::SubjectUnreached,
            message: format!("native test route: {rendered} — {cause}"),
        },
    }
}

/// THE MEMBER STANDING BECOMES THIS VERB'S TERMINATION, ONE ARM EACH AND NO WILDCARD.
///
/// `NativeMemberTermination` has three arms because `InvocationRefused` is unreachable from a
/// population fold; a fourth arm here would be a constructor nothing can build. A new arm upstream
/// must fail to compile here rather than inherit whichever status a `_` happened to name.
fn native_member_outcome(
    rendered: &str,
    members: cli_run::NativeMemberTermination,
    lane_qualified: bool,
    lane_note: &str,
    summary: &str,
) -> InvocationOutcome {
    let member_termination = match members {
        cli_run::NativeMemberTermination::ObservationHeld => Termination::ObservationHeld,
        cli_run::NativeMemberTermination::ObservationDidNotHold => {
            Termination::ObservationDidNotHold
        }
        cli_run::NativeMemberTermination::SubjectUnreached => Termination::SubjectUnreached,
    };
    // `gunbc.instrument_targets` `native_route_termination_under_qualification`, mirrored: the
    // lane's qualification GATES a positive answer and nothing else. Several of its clauses are
    // what establish the verdict surface is trustworthy at all -- the false and true controls, and
    // MalformedSpecimenAccepted, whose own note says that when they fail every verdict in the
    // census is worthless -- so a pass read off rows with no established provenance is the
    // fabricated plausible output DESIGN section 5 forbids. The clamp is ONE-DIRECTIONAL: it can
    // only weaken an answer, never strengthen one, and it never manufactures a failure no member
    // reported.
    let termination = if lane_qualified {
        member_termination
    } else {
        match member_termination {
            Termination::ObservationHeld => Termination::SubjectUnreached,
            other => other,
        }
    };
    let members_note = match members {
        cli_run::NativeMemberTermination::ObservationHeld => "every selected test passed",
        cli_run::NativeMemberTermination::ObservationDidNotHold => {
            "a selected test evaluated and did not hold"
        }
        cli_run::NativeMemberTermination::SubjectUnreached => {
            "no verdict was obtained for part of the selection (an empty selection, or a test that \
             could not be reached)"
        }
    };
    InvocationOutcome {
        termination,
        message: format!(
            "native test route: {rendered} adjudicated — {members_note}; {lane_note} — {summary}"
        ),
    }
}

/// THE ONE SEAM: argv operand -> pattern admission -> route. Mirrors `gunbc.target_invocation`
/// `admit_test_operand`, including the containment that decides WHICH executor answers.
///
/// An operand inside the native route's universe -- single target or set form alike -- enters the
/// native implementation carrying that pattern. Outside it, a SINGLE target builds the registry
/// and looks up EXACTLY — no prefix, suffix or "did you mean": a near miss silently running a
/// different target is worse than a refusal naming the one asked for. A SET form outside it has no
/// executor and is refused with status 2, no observation; handing it to the interpreter would let
/// an interpreted run stand in for a native one. Mirrors
/// `test_operand_set_form_refusal_rendered`.
fn test_operand_set_form_refusal_rendered(operand: &str) -> String {
    format!(
        "gunbc test: {operand} denotes a SET of targets outside every route's universe (the native test route and the claim route); no executor enumerates that population, and a set form is never widened into another route"
    )
}

/// `gunbc.discovery_census` `claim_route_universe`, mirrored: `//test/claim/...`, the subtree
/// `site_label` maps every `test.claim.*` claim into. An operand contained in it (and not in the
/// native universe, which is asked first) belongs to the claim route.
pub fn claim_route_universe() -> TargetPattern {
    TargetPattern::SubtreeTargets(vec!["test".to_string(), "claim".to_string()])
}

/// `gunbc.target_invocation` `ClaimRouteVerdict`, mirrored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClaimRouteVerdict {
    ClaimHeld,
    ClaimDidNotHold,
    ClaimUnobserved,
}

/// One claim's outcome as the route's verdict. `Fail`, `NotBool` and `ExitFailure` are
/// observations that did not hold; every other arm is the absence of an observation -- an error,
/// a refusal, an unwind, a budget interrupt -- and is never reported as a failing claim.
/// Wildcard-free so a new outcome arm must be classified here.
fn claim_route_verdict(outcome: &cli_run::ClaimOutcome) -> ClaimRouteVerdict {
    use cli_run::ClaimOutcome as O;
    match outcome {
        O::Pass => ClaimRouteVerdict::ClaimHeld,
        O::Fail | O::NotBool { .. } | O::ExitFailure { .. } => ClaimRouteVerdict::ClaimDidNotHold,
        O::RuntimeError { .. }
        | O::BudgetInterrupted { .. }
        | O::CompletedOverBudget { .. }
        | O::HostToolUnresolved { .. }
        | O::HostEffectRefused { .. }
        | O::Panicked { .. }
        | O::NotAttempted { .. } => ClaimRouteVerdict::ClaimUnobserved,
    }
}

/// `gunbc.target_invocation` `claim_route_termination`, mirrored: a definite failure dominates, an
/// unobserved member or an empty selection is `SubjectUnreached`.
fn claim_route_termination(verdicts: &[ClaimRouteVerdict]) -> Termination {
    if verdicts.contains(&ClaimRouteVerdict::ClaimDidNotHold) {
        Termination::ObservationDidNotHold
    } else if verdicts.contains(&ClaimRouteVerdict::ClaimUnobserved) || verdicts.is_empty() {
        Termination::SubjectUnreached
    } else {
        Termination::ObservationHeld
    }
}

/// THE CLAIM ROUTE: select the discovered claims the operand's pattern contains, evaluate each,
/// print one line per claim, fold the verdicts. The subject is narrowed to the modules the pattern
/// can reach BEFORE discovery, so `//test/claim/m:x` reads one module, not the corpus; the corpus
/// roots are `cli_run::witness_layer_roots`, the floor's own authority for them.
fn run_claim_route(pattern: &TargetPattern) -> InvocationOutcome {
    let rendered = render_target_pattern(pattern);
    let module_selected = |module_path: &str| {
        let segments: Vec<String> = module_path.split('.').map(str::to_string).collect();
        match pattern {
            TargetPattern::SingleTarget(label) => label.package_segments == segments,
            TargetPattern::PackageTargets(_) | TargetPattern::SubtreeTargets(_) => {
                target_pattern_within(&TargetPattern::PackageTargets(segments), pattern)
            }
        }
    };
    let claim_selected = |module_path: &str, function: &str| {
        let label = Label {
            package_segments: module_path.split('.').map(str::to_string).collect(),
            target: function.to_string(),
        };
        target_pattern_within(&TargetPattern::SingleTarget(label), pattern)
    };
    let members = match cli_run::run_claim_route(
        &cli_run::witness_layer_roots(),
        &module_selected,
        &claim_selected,
    ) {
        Ok(members) => members,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("claim route: {rendered} — {cause}"),
            }
        }
    };
    let mut verdicts = Vec::with_capacity(members.len());
    for m in &members {
        let verdict = claim_route_verdict(&m.outcome);
        let word = match verdict {
            ClaimRouteVerdict::ClaimHeld => "HELD",
            ClaimRouteVerdict::ClaimDidNotHold => "DID-NOT-HOLD",
            ClaimRouteVerdict::ClaimUnobserved => "UNOBSERVED",
        };
        let label = render_label(&Label {
            package_segments: m.module_path.split('.').map(str::to_string).collect(),
            target: m.function.clone(),
        });
        let detail = match verdict {
            ClaimRouteVerdict::ClaimHeld => String::new(),
            _ => format!(" outcome={:?}", m.outcome),
        };
        println!(
            "{label} {word} eval_steps={} cpu_ms={}{detail}",
            m.eval_steps, m.cpu_ms
        );
        verdicts.push(verdict);
    }
    let count = |v: ClaimRouteVerdict| verdicts.iter().filter(|x| **x == v).count();
    let held = count(ClaimRouteVerdict::ClaimHeld);
    let did_not_hold = count(ClaimRouteVerdict::ClaimDidNotHold);
    let unobserved = count(ClaimRouteVerdict::ClaimUnobserved);
    let termination = claim_route_termination(&verdicts);
    let note = if verdicts.is_empty() {
        " — the pattern selected no discovered claim"
    } else {
        ""
    };
    InvocationOutcome {
        termination,
        message: format!(
            "claim route: {rendered} — {} claim(s): {held} held, {did_not_hold} did not hold, \
             {unobserved} unobserved{note}",
            verdicts.len()
        ),
    }
}

/// `gunbc.target_invocation` `BinaryInput`, mirrored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryInput {
    NotNewer(String),
    Newer(String),
    Missing(String),
    OutsideWorktree(String),
}

/// `gunbc.target_invocation` `BinaryInputObservation`, mirrored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryInputObservation {
    Observed {
        binary: String,
        inputs: Vec<BinaryInput>,
    },
    DepInfoUnreadable {
        binary: String,
        reason: String,
    },
}

/// `gunbc.target_invocation` `BinaryFreshness`, mirrored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryFreshness {
    Fresh {
        binary: String,
        inputs: usize,
    },
    Stale {
        binary: String,
        input: String,
        why: String,
    },
    Undecided {
        binary: String,
        reason: String,
    },
}

fn binary_built_elsewhere(binary: &str, path: &str) -> BinaryFreshness {
    BinaryFreshness::Undecided {
        binary: binary.to_string(),
        reason: format!(
            "its dep-info names an input outside this worktree ({path}); it was built from another checkout, so nothing here says whether it answers for this one"
        ),
    }
}

/// `gunbc.target_invocation` `binary_freshness_step`, mirrored arm for arm.
fn binary_freshness_step(acc: BinaryFreshness, input: &BinaryInput) -> BinaryFreshness {
    match acc {
        BinaryFreshness::Undecided { .. } => acc,
        BinaryFreshness::Stale { ref binary, .. } => match input {
            BinaryInput::OutsideWorktree(p) => binary_built_elsewhere(binary, p),
            _ => acc,
        },
        BinaryFreshness::Fresh { binary, inputs } => match input {
            BinaryInput::OutsideWorktree(p) => binary_built_elsewhere(&binary, p),
            BinaryInput::Newer(p) => BinaryFreshness::Stale {
                binary,
                input: p.clone(),
                why: "changed after the binary was built".to_string(),
            },
            BinaryInput::Missing(p) => BinaryFreshness::Stale {
                binary,
                input: p.clone(),
                why: "no longer exists".to_string(),
            },
            BinaryInput::NotNewer(_) => BinaryFreshness::Fresh {
                binary,
                inputs: inputs + 1,
            },
        },
    }
}

/// `gunbc.target_invocation` `assess_binary_freshness`, mirrored.
pub fn assess_binary_freshness(observed: &BinaryInputObservation) -> BinaryFreshness {
    match observed {
        BinaryInputObservation::DepInfoUnreadable { binary, reason } => {
            BinaryFreshness::Undecided {
                binary: binary.clone(),
                reason: reason.clone(),
            }
        }
        BinaryInputObservation::Observed { binary, inputs } => inputs.iter().fold(
            BinaryFreshness::Fresh {
                binary: binary.clone(),
                inputs: 0,
            },
            binary_freshness_step,
        ),
    }
}

/// `gunbc.target_invocation` `binary_freshness_rendered`, mirrored.
pub fn binary_freshness_rendered(f: &BinaryFreshness) -> String {
    match f {
        BinaryFreshness::Fresh { binary, .. } => {
            format!("gunbc test: binary fresh against its dep-info: {binary}")
        }
        BinaryFreshness::Stale { binary, input, why } => format!(
            "gunbc test: REFUSED cause=StaleBinary — {binary}: input {input} {why}; rebuild before measuring"
        ),
        BinaryFreshness::Undecided { binary, reason } => {
            format!("gunbc test: REFUSED cause=BinaryFreshnessUndecided — {binary}: {reason}")
        }
    }
}

/// The inputs of a cargo dep-info file: the first rule's prerequisites, `\ `-escaped spaces
/// unescaped. Cargo's uplifted `<bin>.d` is one rule, `<bin>: <input> <input> ...`.
pub fn dep_info_inputs(dep_info: &str) -> Result<Vec<String>, String> {
    let rule = dep_info
        .lines()
        .find(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .ok_or("dep-info carries no rule")?;
    let (_, prerequisites) = rule
        .split_once(": ")
        .ok_or("dep-info rule has no `: ` separator")?;
    let mut inputs = Vec::new();
    let mut current = String::new();
    let mut chars = prerequisites.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' if chars.peek() == Some(&' ') => {
                current.push(' ');
                chars.next();
            }
            ' ' => {
                if !current.is_empty() {
                    inputs.push(std::mem::take(&mut current));
                }
            }
            other => current.push(other),
        }
    }
    if !current.is_empty() {
        inputs.push(current);
    }
    if inputs.is_empty() {
        return Err("dep-info lists no inputs".to_string());
    }
    Ok(inputs)
}

/// THE CRATE THE BINARY WAS BUILT FROM is the directory of the first `Cargo.toml` its dep-info
/// lists; the checkout is that manifest's `src/v1/stage0` ancestor. Comparing that ONE root
/// against ours is the "built elsewhere" test -- not a list of excluded prefixes. Registry and
/// sysroot inputs are outside every checkout and are judged by mtime like any other input.
///
/// Git paths are skipped: they are not source inputs of the crate, and judging them would refuse
/// a binary whose code did not change after a fetch in any worktree.
pub fn classify_binary_inputs(
    inputs: &[String],
    workspace: &std::path::Path,
    built_at: std::time::SystemTime,
) -> Result<Vec<BinaryInput>, String> {
    // NO BUILD ROOT IS A REFUSAL, NOT A SKIPPED CHECK (review 74282). If no input names the
    // crate manifest, the checkout the binary was built from is unknown, and judging the
    // remaining inputs by mtime alone would answer `Fresh` for a binary another worktree built.
    let build_root = inputs
        .iter()
        .find(|p| p.ends_with("/src/v1/stage0/Cargo.toml"))
        .map(|p| p.trim_end_matches("/src/v1/stage0/Cargo.toml").to_string())
        .ok_or_else(|| {
            "its dep-info names no src/v1/stage0/Cargo.toml, so the checkout it was built from is unknown"
                .to_string()
        })?;
    let mut out = Vec::new();
    if std::path::Path::new(&build_root) != workspace {
        out.push(BinaryInput::OutsideWorktree(build_root.clone()));
    }
    for input in inputs {
        if input.contains("/.git/") {
            continue;
        }
        // Only NotFound is `Missing`; any other stat failure (permission, I/O) is a reading we
        // could not take, which refuses with its own cause rather than a wrong label (review 74474).
        let classified = match std::fs::metadata(input).and_then(|m| m.modified()) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                BinaryInput::Missing(input.clone())
            }
            Err(e) => return Err(format!("cannot stat dep-info input {input}: {e}")),
            Ok(modified) if modified > built_at => BinaryInput::Newer(input.clone()),
            Ok(_) => BinaryInput::NotNewer(input.clone()),
        };
        out.push(classified);
    }
    Ok(out)
}

/// THE ANCHOR IS WHEN COMPILATION STARTED, NOT WHEN THE BINARY WAS LINKED. An input edited after
/// rustc read it but before the link finished is older than the binary and newer than the
/// compile, so comparing against the binary's mtime would answer `Fresh` where cargo rebuilds
/// (calm-boar-904's objection on #13007). Cargo's own comparator is the unit's fingerprint
/// dep-info, `.fingerprint/<pkg>-<hash>/dep-bin-<name>`, whose mtime it rewinds to build start.
///
/// THE UNIT IS DERIVED, NOT SEARCHED FOR BY NAME. The uplifted executable is a hard link to
/// `deps/<name>-<hash>`; that file is found by inode, its `<hash>` names exactly one fingerprint
/// directory, and `dep-bin-<name>` in it is the anchor. Many fingerprint directories carry
/// `dep-bin-gunbc` (one per past build configuration), so picking by name or by newest mtime
/// would be a guess. Every break in the chain -- no hard-link twin, two twins, zero or several
/// fingerprint directories -- is an error, which refuses.
pub fn build_start_anchor(exe: &std::path::Path) -> Result<std::time::SystemTime, String> {
    use std::os::unix::fs::MetadataExt;
    let profile_dir = exe
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", exe.display()))?;
    let name = exe
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| format!("{} has no file name", exe.display()))?;
    let inode = std::fs::metadata(exe)
        .map_err(|e| format!("cannot stat {}: {e}", exe.display()))?
        .ino();
    let deps = profile_dir.join("deps");
    let prefix = format!("{name}-");
    let twins: Vec<String> = std::fs::read_dir(&deps)
        .map_err(|e| format!("cannot list {}: {e}", deps.display()))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.metadata().map(|m| m.ino() == inode).unwrap_or(false))
        .filter_map(|entry| entry.file_name().to_str().map(str::to_string))
        .filter(|file| file.starts_with(&prefix) && !file.ends_with(".d"))
        .collect();
    let hash = match twins.as_slice() {
        [one] => one[prefix.len()..].to_string(),
        _ => {
            return Err(format!(
                "the binary has {} hard-link twins under {}, so the compile unit that built it is unknown",
                twins.len(),
                deps.display()
            ))
        }
    };
    let fingerprint = profile_dir.join(".fingerprint");
    let suffix = format!("-{hash}");
    let units: Vec<std::path::PathBuf> = std::fs::read_dir(&fingerprint)
        .map_err(|e| format!("cannot list {}: {e}", fingerprint.display()))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.ends_with(&suffix))
                .unwrap_or(false)
        })
        .collect();
    let unit = match units.as_slice() {
        [one] => one.join(format!("dep-bin-{name}")),
        _ => {
            return Err(format!(
                "{} fingerprint units end in {suffix}, so the build-start anchor is unknown",
                units.len()
            ))
        }
    };
    std::fs::metadata(&unit)
        .and_then(|m| m.modified())
        .map_err(|e| format!("cannot read the build-start anchor {}: {e}", unit.display()))
}

/// The effectful half: read the binary's dep-info, anchor at build start, and stat its inputs.
/// Every failure is `DepInfoUnreadable`, which refuses -- never a pass because the question could
/// not be asked.
pub fn observe_binary_inputs_of(
    exe: &std::path::Path,
    workspace: &std::path::Path,
) -> BinaryInputObservation {
    let binary = exe.display().to_string();
    let dep_info_path = exe.with_extension("d");
    let observed = (|| {
        let build_start = build_start_anchor(exe)?;
        let text = std::fs::read_to_string(&dep_info_path)
            .map_err(|e| format!("cannot read dep-info {}: {e}", dep_info_path.display()))?;
        let inputs = dep_info_inputs(&text)?;
        classify_binary_inputs(&inputs, workspace, build_start)
    })();
    match observed {
        Ok(inputs) => BinaryInputObservation::Observed { binary, inputs },
        Err(reason) => BinaryInputObservation::DepInfoUnreadable { binary, reason },
    }
}

fn observe_binary_inputs() -> BinaryInputObservation {
    match std::env::current_exe() {
        Ok(exe) => observe_binary_inputs_of(&exe, &cli_run::process_workspace_root()),
        Err(e) => BinaryInputObservation::DepInfoUnreadable {
            binary: "<current_exe unreadable>".to_string(),
            reason: e.to_string(),
        },
    }
}

/// THE INVOCATION'S FIRST ACT: refuse a binary that no longer answers for its own inputs
/// (`gunbc.target_invocation` `assess_binary_freshness`), then route. Kept beside `test_verb`
/// rather than inside it so the routing tests stay a function of the operand alone.
pub fn test_verb_checked(operand: &str) -> InvocationOutcome {
    test_verb_after(operand, &assess_binary_freshness(&observe_binary_inputs()))
}

fn test_verb_after(operand: &str, freshness: &BinaryFreshness) -> InvocationOutcome {
    match stale_binary_refusal(freshness) {
        Some(refused) => refused,
        None => test_verb(operand),
    }
}

/// THE FRESHNESS DECISION, ALONE: `Some` is the refusal a stale or undecided binary earns before any
/// producer runs, `None` admits the run (after logging the fresh inputs). Split from
/// `test_verb_after` so its witness exercises the decision without reaching `test_verb`'s producers,
/// several of which set the process cwd (`process_cwd_mutation_reachability_gate`).
fn stale_binary_refusal(freshness: &BinaryFreshness) -> Option<InvocationOutcome> {
    let line = binary_freshness_rendered(freshness);
    match freshness {
        BinaryFreshness::Fresh { inputs, .. } => {
            eprintln!("{line} inputs={inputs}");
            None
        }
        BinaryFreshness::Stale { .. } | BinaryFreshness::Undecided { .. } => {
            Some(InvocationOutcome {
                termination: Termination::Refused,
                message: line,
            })
        }
    }
}

pub fn test_verb(operand: &str) -> InvocationOutcome {
    let pattern = match parse_target_pattern(operand) {
        Ok(pattern) => pattern,
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
    if target_pattern_within(&pattern, &native_route_default_pattern()) {
        return run_native_test_route(&pattern);
    }
    if target_pattern_within(&pattern, &claim_route_universe()) {
        return run_claim_route(&pattern);
    }
    let label = match pattern {
        TargetPattern::SingleTarget(label) => label,
        TargetPattern::PackageTargets(_) | TargetPattern::SubtreeTargets(_) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: test_operand_set_form_refusal_rendered(operand),
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

/// THE SUBJECT IS OWNED HERE, IN RUST, AND THAT IS A MEASURED DECISION RATHER THAN A SHORTCUT.
///
/// Every sibling instrument keeps its subject in `gunbc.instrument_targets` and mirrors it here,
/// and this one was built that way first: two rows, `floor_memory_qualification_source_roots` and
/// `floor_memory_qualification_lane`, read through the interpreter before spawning the child so
/// the model would be the single authority for what gets measured.
///
/// IT CORRUPTS THE MEASUREMENT, AND THE COST WAS MEASURED RATHER THAN FEARED. Reading those rows
/// means resolving a corpus graph, and this supervisor shares its cgroup with the child BY
/// DESIGN — that sharing is what lets `memory.peak` survive the child's death. So the resolve
/// lands in the very counter the instrument reports. Three runs of the same failing floor, same
/// tree, same binary, differing only in whether the subject was read from the model:
///
///   Rust-owned subject   peak 15746146304  (14.66 GiB)
///   Rust-owned subject   peak 15704227840  (14.63 GiB)   <- 0.2% apart, reproducible
///   read from the model  peak 22293544960  (20.76 GiB)   <- +6.1 GiB, 42% inflation
///
/// An instrument may not consult the authority from inside the cgroup it is measuring; the act of
/// reading perturbs the reading. Netting the supervisor's footprint back out is not available
/// either — that would replace a measured number with an adjusted one, which is the whole habit
/// `gunbc.floor_memory_demand` exists to refuse.
///
/// SO THE TWO `.dag` ROWS ARE DELETED RATHER THAN LEFT UNCONSUMED. A modeled subject nothing reads
/// is the DESIGN section 3c dangling declaration, and keeping it while Rust silently owned the
/// real fact would be worse than owning it openly: two authorities, one of them decorative. The
/// honest state is one authority, here, saying plainly that it is here and why.
///
/// WHAT WOULD RESTORE THE MODELED SUBJECT: any route that reads the rows OUTSIDE the measured
/// cgroup — a supervisor that resolves the subject before entering the scope, or a scope created
/// after the read rather than around it. Both need the cgroup lifecycle to be modeled, which is
/// the same capability `gunbc.target_invocation_seed_growth` names as this subset's trigger.
fn floor_memory_qualification_source_roots() -> Vec<String> {
    vec!["dag".to_string(), "src/v2".to_string()]
}

fn floor_memory_qualification_lane() -> String {
    "witnesses".to_string()
}

/// THE SUPERVISED MEMORY QUALIFICATION OF A REQUIRED-FLOOR RUN.
///
/// The three terminations are load-bearing and map onto `gunbc.floor_memory_demand`'s three arms:
/// `DemandObserved` is ObservationHeld (0), `DemandBounded` is ObservationDidNotHold (1) — the
/// peak is a LOWER BOUND, so the run produced a reading that may not size anything downward — and
/// every refusal is Refused (2), no observation at all. An instrument that rendered "could not
/// read" the same as "it fits" would reproduce one layer out the conflation it exists to remove.
///
/// THE PRECONDITION IS CHECKED BEFORE THE WORKLOAD, NOT AFTER. Resolving the measurement cgroup
/// first means a 35-minute run is never spent producing a figure that cannot be qualified — and,
/// more importantly, an invocation with no enforceable limit REFUSES instead of reporting a
/// number. An unbounded run and a bounded one are indistinguishable in everything the workload
/// itself emits, which is the class
/// `gunbc.recurring_failure_mode.suppressed_precondition_failure_runs_the_workload_unconstrained`.
/// THE MEASURED RUN BY PHASE: the child's seam beats, transcribed by the supervisor and folded by
/// `gunbc.floor_demand` `floor_phase_attribution`, one line per phase in the order the run entered
/// them. `entry` is the held set the phase inherited, `peak` the largest over its own beats, and
/// `left` what it left resident -- the next phase's entry, read from that row, so this renderer
/// does no arithmetic of its own. A phase whose held set could not be represented prints the arm.
///
/// This is a reading beside the verdict, never a verdict: a run with no beats (a lane that owns no
/// floor phase) or a transcription refusal says so here and leaves the qualification unchanged.
fn floor_phase_attribution_rendered(
    ctx: &crate::v1_interpreter::InterpContext,
    beat_lines: &[String],
) -> String {
    use crate::v1_interpreter::Value;
    use cli_run::floor_memory_supervisor as sup;
    let beats = match sup::transcribe_floor_beats(beat_lines) {
        Ok(beats) if beats.is_empty() => {
            return "floor-phase-attribution: no seam beats were printed by the measured child"
                .to_string();
        }
        Ok(beats) => beats,
        Err(refusal) => return format!("floor-phase-attribution: refused: {refusal}"),
    };
    // A COUNTER PAST i64 REFUSES rather than clamping: a clamp would hand the fold a value nobody
    // read. None of these can reach it on a real host, which is why a refusal costs nothing.
    if let Some(b) = beats.iter().find(|b| {
        std::iter::once(b.charge)
            .chain(b.stall_per_min)
            .chain(b.swap_bytes)
            .chain(b.stat.iter().copied())
            .any(|v| i64::try_from(v).is_err())
    }) {
        return format!(
            "floor-phase-attribution: refused: beat {} carries a counter past the i64 the fold reads",
            b.beat
        );
    }
    let int = |v: u64| Value::Int(v as i64);
    // AN UNREAD COUNTER CROSSES AS `Absent`, the fold's own optional, never as a zero beside a flag.
    let optional = |v: Option<u64>| -> Value {
        match v {
            Some(n) => Value::Variant {
                type_name: ctx.sym("Optional"),
                variant_name: ctx.sym("Present"),
                fields: std::rc::Rc::new(vec![(ctx.sym("value"), int(n))]),
            },
            None => Value::Variant {
                type_name: ctx.sym("Optional"),
                variant_name: ctx.sym("Absent"),
                fields: std::rc::Rc::new(vec![]),
            },
        }
    };
    let readings: Vec<Value> = beats
        .iter()
        .map(|b| {
            let mut fields = vec![
                (ctx.sym("beat"), Value::Int(b.beat as i64)),
                (ctx.sym("opens_phase"), Value::Bool(b.opens_phase)),
                (
                    ctx.sym("seam_before"),
                    crate::v1_interpreter::str_value(&b.seam_before),
                ),
                (
                    ctx.sym("seam_after"),
                    crate::v1_interpreter::str_value(&b.seam_after),
                ),
                (ctx.sym("stall_per_min"), optional(b.stall_per_min)),
                (ctx.sym("swap_bytes"), optional(b.swap_bytes)),
                (ctx.sym("charge"), int(b.charge)),
            ];
            for (key, value) in sup::FLOOR_BEAT_STAT_KEYS.iter().zip(b.stat.iter()) {
                fields.push((ctx.sym(key), int(*value)));
            }
            Value::Record {
                type_name: ctx.sym("FloorBeatReading"),
                fields: std::rc::Rc::new(fields),
            }
        })
        .collect();
    let args = vec![(
        Some("readings".to_string()),
        crate::v1_interpreter::list_value(readings),
    )];
    let attributed = match crate::v1_interpreter::run_in_context_with_args(
        ctx,
        "floor_phase_attribution_from_readings",
        &args,
        true,
    ) {
        Ok(v) => v,
        Err(cause) => return format!("floor-phase-attribution: not reached: {cause}"),
    };
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = &attributed
    else {
        return format!(
            "floor-phase-attribution: unrecognised result {}",
            ctx.format_value(&attributed)
        );
    };
    if !ctx.sym_eq(*variant_name, "FloorPhasesAttributed") {
        return format!("floor-phase-attribution: {}", ctx.format_value(&attributed));
    }
    let Some(Value::List(rows)) = ctx.field(fields, "phases") else {
        return format!(
            "floor-phase-attribution: unrecognised result {}",
            ctx.format_value(&attributed)
        );
    };
    let held = |v: Option<&Value>| -> String {
        match v {
            Some(Value::Variant {
                variant_name,
                fields,
                ..
            }) if ctx.sym_eq(*variant_name, "HeldSetPeakAt") => match ctx.field(fields, "bytes") {
                Some(Value::Record { fields, .. }) => match ctx.field(fields, "count") {
                    Some(Value::Int(n)) => n.to_string(),
                    other => format!("{other:?}"),
                },
                other => format!("{other:?}"),
            },
            Some(other) => ctx.format_value(other),
            None => "<absent>".to_string(),
        }
    };
    let rows: Vec<&Value> = rows.iter().collect();
    let mut out = vec![format!(
        "floor-phase-attribution: phases={} beats={} (held set per gunbc.floor_demand beat_held_set, bytes)",
        rows.len(),
        beats.len()
    )];
    for (i, row) in rows.iter().enumerate() {
        let Value::Record { fields, .. } = row else {
            out.push(format!("  unrecognised row {}", ctx.format_value(row)));
            continue;
        };
        let left = match rows.get(i + 1) {
            Some(Value::Record { fields: next, .. }) => held(ctx.field(next, "entry")),
            _ => "end-of-run".to_string(),
        };
        out.push(format!(
            "  phase={} opened_at_beat={} entry={} peak={} left={}",
            ctx.field(fields, "seam")
                .map(|v| ctx.format_value(v))
                .unwrap_or_default(),
            ctx.field(fields, "opened_at")
                .map(|v| ctx.format_value(v))
                .unwrap_or_default(),
            held(ctx.field(fields, "entry")),
            held(ctx.field(fields, "peak")),
            left,
        ));
    }
    out.join("\n")
}

/// THE TYPED GRAPH'S BYTES BY CLASS, READ LEAVE-ONE-OUT. The floor's sequential byte attribution
/// (`cli_run::typed_graph_byte_attribution`) credits a class dropped late with every node it shared
/// with classes dropped before it, so its figures are an order-dependent upper bound -- read as a
/// saving, its `emit_graph_info` share predicted a peak cut that measured -0.07 GB (gunbc#12832).
/// What a removal SAVES is the bytes freed by dropping that class alone while every other class is
/// still held, and that needs a fresh graph per class: a dropped class cannot be restored and a
/// deep clone would double the heap under measurement.
///
/// THE SUBJECT IS THE WHOLE-TREE STRICT CLOSURE over `dag` and `src/v2`, the floor's own
/// exclusions applied: at roughly five thousand modules it is the size at which the floor's
/// superlinear growth is the question, and it needs no diff to reproduce. The graph is taken
/// whether or not the typecheck refuses, since a refused graph is the same allocation a refusing
/// floor holds. Six classes, the ones carrying the bytes; the rest are inside the shared residual.
fn run_typed_graph_exclusive_bytes() -> InvocationOutcome {
    let roots = vec!["dag".to_string(), "src/v2".to_string()];
    let excludes = cli_run::floor_prepared_subject_exclusions();
    typed_graph_exclusive_bytes_over("typed-graph-exclusive-bytes", || {
        let picked = cli_run::whole_tree_strict_sources(&roots, &excludes)?;
        let result = crate::v1_compiler_compile::compile_to_resolved(std::rc::Rc::new(
            picked.sources.into(),
        ));
        std::rc::Rc::try_unwrap(result)
            .ok()
            .and_then(|r| r.graph)
            .ok_or_else(|| {
                "the strict resolve produced no graph, or its result has another owner".to_string()
            })
    })
}

/// THE SAME READING OVER THE FLOOR'S OWN PREPARED SUBJECT AT THIS CHECKOUT: the nominal closure
/// (gate prefixes, gate-authored modules, the local-repo wet schedule) the required floor prepares
/// when no diff adds seeds, through the floor's own `prepare_repository_from_corpus`. A second
/// SIZE of the same observation, so `type_env`'s exclusive bytes are read at two closure sizes and
/// the scaling the floor's peak shows is measured on the class rather than inferred from the phase.
fn run_typed_graph_exclusive_bytes_floor_subject() -> InvocationOutcome {
    let roots = vec!["dag".to_string(), "src/v2".to_string()];
    typed_graph_exclusive_bytes_over("typed-graph-exclusive-bytes-floor-subject", || {
        let corpus = cli_run::read_source_corpus_once(&roots);
        let gate_entry_index = cli_run::build_multi_entry_index(&roots);
        let seeds =
            cli_run::required_floor_nominal_subject_seeds_from_corpus(&corpus, &gate_entry_index)?;
        let module_seeds = cli_run::required_floor_nominal_closure_module_seeds(
            &seeds.required_gate_authored_modules,
            &seeds.local_repo_wet_schedule_rows,
        );
        // THE FLOOR'S OWN SUBJECT, COMPILED AS ITS STRICT PREPARE COMPILES IT -- before the prepared
        // repository drops the typecheck caches -- because the floor's peak is inside that compile,
        // where the caches the import union rides on are still alive.
        let subject = cli_run::assemble_prepared_subject_from_corpus(
            &corpus,
            &cli_run::floor_prepared_subject_exclusions(),
            Some((
                &gate_entry_index,
                &seeds.required_gate_prefixes,
                &module_seeds,
            )),
        )?;
        drop(gate_entry_index);
        let result = crate::v1_compiler_compile::compile_to_resolved(std::rc::Rc::new(
            subject.sources.into(),
        ));
        std::rc::Rc::try_unwrap(result)
            .ok()
            .and_then(|r| r.graph)
            .ok_or_else(|| {
                "the strict resolve produced no graph, or its result has another owner".to_string()
            })
    })
}

/// The leave-one-out loop, over whichever subject `subject` resolves -- once per class, from a
/// fresh graph each time, since a dropped class cannot be restored.
fn typed_graph_exclusive_bytes_over(
    label: &str,
    subject: impl Fn() -> Result<std::rc::Rc<crate::v1_compiler_compile::ResolvedGraph>, String>,
) -> InvocationOutcome {
    use cli_run::TypedModuleClass as C;
    if let Err(e) = std::env::set_current_dir(cli_run::workspace_root()) {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!("{label}: refused: could not anchor at the workspace root: {e}"),
        };
    }
    // Every single class, then the union's carriers JOINTLY: the per-module import union is
    // materialized both as TypeEnv.ancestry_str_bindings and as the TypeEnvCache the module's
    // interface hands its importers, and those share spine, so neither alone reads what removing
    // the union would save.
    let sets: Vec<Vec<C>> = vec![
        vec![C::TypeEnv],
        vec![C::TypeEnvCache],
        vec![C::Interface],
        vec![C::Items],
        vec![C::ModuleNodes],
        vec![C::FuncEnv],
        vec![C::OccurrenceTransport],
        vec![C::TypeEnv, C::TypeEnvCache, C::Interface],
    ];
    let mut raw = Vec::new();
    for set in &sets {
        let class_name = set.iter().map(|c| c.name()).collect::<Vec<_>>().join("+");
        let graph = match subject() {
            Ok(g) => g,
            Err(e) => {
                return InvocationOutcome {
                    termination: Termination::SubjectUnreached,
                    message: format!("{label}: subject unreached: {e}"),
                }
            }
        };
        match cli_run::typed_module_class_exclusive_bytes(graph, set) {
            Ok(r) => raw.push((class_name, r)),
            Err(cause) => {
                return InvocationOutcome {
                    termination: Termination::Refused,
                    message: format!("{label}: class {class_name} unattributable: {cause}"),
                }
            }
        }
    }
    // THE HOST TRANSCRIBES, THE .dag FOLD DECIDES (gunbc.typed_graph_exclusive_bytes). Every
    // reading crosses as primitives; a count past i64 refuses rather than clamping.
    const ENTRY: &str = "dag/gunbc/floor/typed_graph_exclusive_bytes.dag";
    let roots = vec!["dag".to_string(), "src/v2".to_string()];
    let (graph, source_indices) = match cli_run::resolve_entry_graph(&roots, ENTRY) {
        Ok(resolved) => resolved,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label}: resolve failed for {ENTRY}: {cause}"),
            }
        }
    };
    let ctx = cli_run::make_eval_context(
        graph.as_ref(),
        source_indices,
        crate::v1_interpreter::ExecutionMode::Wet,
    );
    use crate::v1_interpreter::Value;
    let int = |v: u64| i64::try_from(v).map(Value::Int);
    let mut readings = Vec::new();
    for (class, r) in &raw {
        let counts = [
            r.members as u64,
            r.modules as u64,
            r.in_use_all,
            r.in_use_after_class,
            r.in_use_end,
            r.ancestry_entries,
            r.own_entries,
        ];
        let Ok(vals) = counts
            .iter()
            .map(|v| int(*v))
            .collect::<Result<Vec<_>, _>>()
        else {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!(
                    "{label}: class {} carries a count past the i64 the fold reads",
                    class
                ),
            };
        };
        let names = [
            "members",
            "modules",
            "in_use_all",
            "in_use_after_class",
            "in_use_end",
            "ancestry_entries",
            "own_entries",
        ];
        let mut fields = vec![(ctx.sym("class"), crate::v1_interpreter::str_value(class))];
        for (n, v) in names.iter().zip(vals) {
            fields.push((ctx.sym(n), v));
        }
        readings.push(Value::Record {
            type_name: ctx.sym("TypedGraphClassReading"),
            fields: std::rc::Rc::new(fields),
        });
    }
    let args = vec![(
        Some("readings".to_string()),
        crate::v1_interpreter::list_value(readings),
    )];
    let report = match crate::v1_interpreter::run_in_context_with_args(
        &ctx,
        "typed_graph_exclusive_report",
        &args,
        true,
    ) {
        Ok(v) => v,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label}: the report could not be reached: {cause}"),
            }
        }
    };
    let bytes = |v: Option<&Value>| -> String {
        match v {
            Some(Value::Record { fields, .. }) => match ctx.field(fields, "count") {
                Some(Value::Int(n)) => n.to_string(),
                other => format!("{other:?}"),
            },
            Some(other) => ctx.format_value(other),
            None => "<absent>".to_string(),
        }
    };
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = &report
    else {
        return InvocationOutcome {
            termination: Termination::SubjectUnreached,
            message: format!("{label}: unrecognised report {}", ctx.format_value(&report)),
        };
    };
    if !ctx.sym_eq(*variant_name, "ExclusiveBytesReported") {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!("{label}: {}", ctx.format_value(&report)),
        };
    }
    let mut lines = Vec::new();
    if let Some(Value::List(cs)) = ctx.field(fields, "classes") {
        for c in cs.iter() {
            if let Value::Record { fields: cf, .. } = c {
                let f = |n: &str| {
                    ctx.field(cf, n)
                        .map(|v| ctx.format_value(v))
                        .unwrap_or_default()
                };
                lines.push(format!(
                    "{label} class={} exclusive={} graph_total={} modules={} ancestry_entries={} own_entries={}",
                    f("class"), bytes(ctx.field(cf, "exclusive")), bytes(ctx.field(cf, "graph_total")),
                    f("modules"), f("ancestry_entries"), f("own_entries")
                ));
            }
        }
    }
    let residual = match ctx.field(fields, "shared_or_unlisted") {
        Some(Value::Variant {
            variant_name,
            fields: rf,
            ..
        }) if ctx.sym_eq(*variant_name, "MeasureDifference") => bytes(ctx.field(rf, "value")),
        Some(other) => ctx.format_value(other),
        None => "<absent>".to_string(),
    };
    lines.push(format!(
        "{label} graph_total={} sum_of_exclusives={} shared_or_unlisted={residual} (gunbc.typed_graph_exclusive_bytes; graph_total from the first run, every class from its own fresh resolve)",
        bytes(ctx.field(fields, "graph_total")),
        bytes(ctx.field(fields, "sum_of_exclusives")),
    ));
    InvocationOutcome {
        termination: Termination::ObservationHeld,
        message: lines.join("\n"),
    }
}

fn run_floor_memory_qualification() -> InvocationOutcome {
    use cli_run::floor_memory_supervisor as sup;

    if let Err(e) = std::env::set_current_dir(cli_run::workspace_root()) {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!(
                "floor-memory-qualification: refused: could not anchor at the workspace root: {e}"
            ),
        };
    }

    let measured_roots = floor_memory_qualification_source_roots();
    let measured_lane = floor_memory_qualification_lane();

    let cgroup = match sup::resolve_measurement_cgroup() {
        Ok(dir) => dir,
        Err(refusal) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!("floor-memory-qualification: refused: {}", refusal.render()),
            };
        }
    };

    // BOTH GUARDS RUN BEFORE THE WORKLOAD, not after: a 35-minute run that turns out to be
    // unattributable is a wasted run, and worse, a tempting one to publish anyway.
    match sup::children_of_cgroup(&cgroup) {
        Ok(children) if !children.is_empty() => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!(
                    "floor-memory-qualification: refused: {}",
                    sup::QualificationRefusal::MeasurementCgroupHasChildren {
                        dir: cgroup.to_string_lossy().to_string(),
                        children,
                    }
                    .render()
                ),
            };
        }
        Ok(_) => {}
        Err(refusal) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!("floor-memory-qualification: refused: {}", refusal.render()),
            };
        }
    }

    match sup::strangers_in_cgroup(&cgroup) {
        Ok(strangers) if !strangers.is_empty() => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!(
                    "floor-memory-qualification: refused: {}",
                    sup::QualificationRefusal::MeasurementCgroupShared {
                        dir: cgroup.to_string_lossy().to_string(),
                        strangers,
                    }
                    .render()
                ),
            };
        }
        Ok(_) => {}
        Err(refusal) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!("floor-memory-qualification: refused: {}", refusal.render()),
            };
        }
    }

    let baseline_peak = match sup::reset_or_baseline_peak(&cgroup) {
        Ok(b) => b,
        Err(refusal) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!("floor-memory-qualification: refused: {}", refusal.render()),
            };
        }
    };

    let exe = match std::env::current_exe() {
        Ok(p) => p.with_file_name("claim_executor"),
        Err(e) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!(
                    "floor-memory-qualification: refused: could not locate the measured binary \
                     beside this one: {e}"
                ),
            };
        }
    };

    let mut args = vec![
        "--required-ci".to_string(),
        "--required-lane".to_string(),
        measured_lane.clone(),
    ];
    for root in measured_roots.iter().cloned() {
        args.push("--source-root".to_string());
        args.push(root);
    }

    let (termination, beat_lines) =
        match sup::run_child_in_own_cgroup(&exe.to_string_lossy(), &args) {
            Ok(t) => t,
            Err(refusal) => {
                return InvocationOutcome {
                    termination: Termination::Refused,
                    message: format!("floor-memory-qualification: refused: {}", refusal.render()),
                };
            }
        };

    let read = match sup::read_cgroup_memory(&cgroup) {
        Ok(r) => r,
        Err(refusal) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!("floor-memory-qualification: refused: {}", refusal.render()),
            };
        }
    };

    // THE PEAK MUST HAVE RISEN, or it is not this run's. Where the kernel refused to reset the
    // counter, a post-run peak equal to the pre-run one says only that nothing here exceeded
    // history — it says nothing about what this run demanded, and publishing it would attribute
    // another workload's high-water mark to the floor.
    if read.peak <= baseline_peak {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!(
                "floor-memory-qualification: refused: {}",
                sup::QualificationRefusal::PeakDominatedByPriorHistory {
                    dir: read.dir.clone(),
                    before: baseline_peak,
                    after: read.peak,
                }
                .render()
            ),
        };
    }

    // THE JUDGMENT IS THE `.dag` MODULE'S, NOT THIS FUNCTION'S. Reproducing the arms here in
    // Rust would give one decision two authorities (DESIGN section 3); the host reads the
    // counters and the substrate decides what they mean — including how to parse a `max` body,
    // which `extdeps.linux.cgroup_v2_memory` already owns.
    const ENTRY: &str = "dag/gunbc/floor_memory_demand.dag";
    const FUNCTION: &str = "qualify_floor_memory_from_readings";
    let (graph, source_indices) = match cli_run::resolve_entry_graph(&measured_roots, ENTRY) {
        Ok(resolved) => resolved,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("floor-memory-qualification: resolve failed for {ENTRY}: {cause}"),
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
                "floor-memory-qualification: {ENTRY} has blocking diagnostics: {}",
                blocking.iter().cloned().collect::<Vec<_>>().join("; ")
            ),
        };
    }
    let ctx = cli_run::make_eval_context(
        graph.as_ref(),
        source_indices,
        crate::v1_interpreter::ExecutionMode::Wet,
    );
    let (signalled, code) = match &termination {
        sup::SupervisedTermination::Exited { code } => (false, *code as i64),
        sup::SupervisedTermination::Signalled { signal } => (true, *signal as i64),
    };
    let args: Vec<(Option<String>, crate::v1_interpreter::Value)> = vec![
        (
            Some("peak_bytes".to_string()),
            crate::v1_interpreter::Value::Int(read.peak as i64),
        ),
        (
            Some("limit_max_body".to_string()),
            crate::v1_interpreter::Value::Str(read.limit_max.as_str().into()),
        ),
        (
            Some("limit_high_body".to_string()),
            crate::v1_interpreter::Value::Str(read.limit_high.as_str().into()),
        ),
        (
            Some("high_events".to_string()),
            crate::v1_interpreter::Value::Int(read.high_events as i64),
        ),
        (
            Some("max_events".to_string()),
            crate::v1_interpreter::Value::Int(read.max_events as i64),
        ),
        (
            Some("oom_kills".to_string()),
            crate::v1_interpreter::Value::Int(read.oom_kills as i64),
        ),
        (
            Some("terminated_by_signal".to_string()),
            crate::v1_interpreter::Value::Bool(signalled),
        ),
        (
            Some("exit_code".to_string()),
            crate::v1_interpreter::Value::Int(code),
        ),
    ];
    let verdict = match crate::v1_interpreter::run_in_context_with_args(&ctx, FUNCTION, &args, true)
    {
        Ok(v) => v,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!(
                    "floor-memory-qualification: the judgment could not be reached: {cause}"
                ),
            };
        }
    };

    let phases = floor_phase_attribution_rendered(&ctx, &beat_lines);

    // The supervisor shares the cgroup with the child, so its own few MiB are inside this peak.
    // Stated rather than netted out: subtracting an estimate would replace a measured number with
    // an adjusted one.
    let detail = format!(
        "floor-memory-qualification: cgroup={} peak={} memory.max={} memory.high={} \
         events=[high {} / max {} / oom_kill {}] termination={} lane={} \
         (the supervisor shares this cgroup with the measured child, so its own footprint — a few \
         MiB — is included in the peak rather than subtracted)\n{phases}",
        read.dir,
        read.peak,
        read.limit_max,
        read.limit_high,
        read.high_events,
        read.max_events,
        read.oom_kills,
        match &termination {
            sup::SupervisedTermination::Exited { code } => format!("exited {code}"),
            sup::SupervisedTermination::Signalled { signal } => format!("signalled {signal}"),
        },
        measured_lane,
    );

    // WILDCARD-FREE ON PURPOSE: a fourth arm added to `FloorMemoryQualification` must land here
    // rather than inherit whichever termination a `_` happened to name.
    match &verdict {
        crate::v1_interpreter::Value::Variant { variant_name, .. }
            if ctx.sym_eq(*variant_name, "DemandObserved") =>
        {
            InvocationOutcome {
                termination: Termination::ObservationHeld,
                message: format!(
                    "{detail}\nDemandObserved — nothing held this run, so the peak is a DEMAND."
                ),
            }
        }
        crate::v1_interpreter::Value::Variant { variant_name, .. }
            if ctx.sym_eq(*variant_name, "DemandBounded") =>
        {
            InvocationOutcome {
                termination: Termination::ObservationDidNotHold,
                message: format!(
                    "{detail}\nDemandBounded — the peak is a LOWER BOUND, not a demand: the run \
                     was killed, throttled, or pinned to a limit. It may not be used to size \
                     anything downward."
                ),
            }
        }
        crate::v1_interpreter::Value::Variant { variant_name, .. }
            if ctx.sym_eq(*variant_name, "DemandUnreadable") =>
        {
            InvocationOutcome {
                termination: Termination::Refused,
                message: format!(
                    "{detail}\nDemandUnreadable — the reading could not be qualified."
                ),
            }
        }
        other => InvocationOutcome {
            termination: Termination::SubjectUnreached,
            message: format!(
                "floor-memory-qualification: {FUNCTION} returned a value this seam does not \
                 recognise as a FloorMemoryQualification: {other:?}"
            ),
        },
    }
}

#[cfg(test)]
mod native_route_termination_tests {
    use super::*;

    /// `gunbc.target_invocation` `claim_route_termination`, mirrored: the same five rows
    /// `test.claim.target_invocation_witness` `the_claim_route_termination_maps_to_the_three_exit_statuses`
    /// pins on the model, so the host mirror cannot drift from it silently.
    #[test]
    fn claim_route_termination_matches_the_model() {
        use ClaimRouteVerdict::*;
        let status = |v: &[ClaimRouteVerdict]| invocation_exit_status(claim_route_termination(v));
        assert_eq!(status(&[ClaimHeld, ClaimHeld]), 0);
        assert_eq!(status(&[ClaimHeld, ClaimDidNotHold]), 1);
        assert_eq!(status(&[ClaimUnobserved, ClaimDidNotHold]), 1);
        assert_eq!(status(&[ClaimHeld, ClaimUnobserved]), 2);
        assert_eq!(status(&[]), 2);
    }

    /// The claim universe is asked AFTER the native one and refuses what lies outside it, mirroring
    /// the model's admission rows.
    #[test]
    fn claim_universe_containment_matches_the_model() {
        let within = |operand: &str| {
            target_pattern_within(
                &parse_target_pattern(operand).expect("pattern"),
                &claim_route_universe(),
            )
        };
        assert!(within("//test/claim/target_invocation_witness:w"));
        assert!(within("//test/claim/target_invocation_witness:all"));
        assert!(within("//test/claim/..."));
        assert!(!within("//test/..."));
        assert!(!within("//dag/test/claim/compute:all"));
        assert!(!within("//gunbc/instruments:self-host"));
    }

    /// `gunbc.instrument_targets` `native_route_termination_under_qualification`, mirrored here and
    /// given its own red. The `.dag` witness pins the fold; this pins that THIS host applies it.
    ///
    /// A REFUSED QUALIFICATION CANNOT YIELD A PASS (review 70107). Several qualification clauses
    /// are what establish the verdict surface is trustworthy at all, so a pass read off rows with
    /// no established provenance is fabricated plausible output.
    #[test]
    fn a_refused_qualification_clamps_a_pass_to_unreached() {
        let out = native_member_outcome(
            "//v2/test/parse:all",
            cli_run::NativeMemberTermination::ObservationHeld,
            false,
            "refused",
            "s",
        );
        assert_eq!(out.termination, Termination::SubjectUnreached);
    }

    /// AND A QUALIFIED ROUTE PASSES THE MEMBER STANDING THROUGH, which is what makes the clamp a
    /// gate rather than a constant: without this, always answering SubjectUnreached would pass the
    /// arm above.
    #[test]
    fn a_qualified_route_passes_a_pass_through() {
        let out = native_member_outcome(
            "//v2/test/parse:all",
            cli_run::NativeMemberTermination::ObservationHeld,
            true,
            "held",
            "s",
        );
        assert_eq!(out.termination, Termination::ObservationHeld);
    }

    /// A DEFINITE FAILURE SURVIVES A REFUSED QUALIFICATION. Demoting it would bury a located defect
    /// behind an infrastructure problem, and the gate is one-directional by construction.
    #[test]
    fn a_refused_qualification_does_not_demote_a_failure() {
        let out = native_member_outcome(
            "//v2/test/parse:all",
            cli_run::NativeMemberTermination::ObservationDidNotHold,
            false,
            "refused",
            "s",
        );
        assert_eq!(out.termination, Termination::ObservationDidNotHold);
    }
}

#[cfg(test)]
mod binary_freshness_tests {
    use super::*;

    /// THE OLD BEHAVIOUR RAN THE INSTRUMENT ON EVERY ARM BELOW; the stale and undecided arms are
    /// the refusals it lacked. Mirrors `test.claim.target_invocation_witness`'s freshness tests.
    #[test]
    fn freshness_precedence_matches_the_model() {
        let obs = |inputs: Vec<BinaryInput>| BinaryInputObservation::Observed {
            binary: "bin".into(),
            inputs,
        };
        let fresh = assess_binary_freshness(&obs(vec![
            BinaryInput::NotNewer("a".into()),
            BinaryInput::NotNewer("b".into()),
        ]));
        assert!(matches!(fresh, BinaryFreshness::Fresh { inputs: 2, .. }));
        match assess_binary_freshness(&obs(vec![
            BinaryInput::NotNewer("a".into()),
            BinaryInput::Newer("b".into()),
            BinaryInput::Missing("c".into()),
        ])) {
            BinaryFreshness::Stale { input, .. } => assert_eq!(input, "b"),
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            assess_binary_freshness(&obs(vec![
                BinaryInput::Newer("b".into()),
                BinaryInput::OutsideWorktree("/w/other".into()),
            ])),
            BinaryFreshness::Undecided { .. }
        ));
        assert!(matches!(
            assess_binary_freshness(&BinaryInputObservation::DepInfoUnreadable {
                binary: "bin".into(),
                reason: "none".into()
            }),
            BinaryFreshness::Undecided { .. }
        ));
    }

    /// The observation over REAL files: a file touched after the build instant is named; a
    /// git watch path is skipped; another checkout's manifest is outside; escaped spaces parse.
    #[test]
    fn dep_info_classification_over_real_files() {
        let dir = std::env::temp_dir().join(format!("freshness-{}", std::process::id()));
        let ws = dir.join("ws");
        std::fs::create_dir_all(ws.join("src/v1/stage0")).unwrap();
        let old = ws.join("old file.rs");
        std::fs::write(&old, "x").unwrap();
        let built_at = std::time::SystemTime::now();
        std::thread::sleep(std::time::Duration::from_millis(20));
        let new = ws.join("new.rs");
        std::fs::write(&new, "y").unwrap();
        let manifest = ws.join("src/v1/stage0/Cargo.toml");
        std::fs::write(&manifest, "").unwrap();
        let text = format!(
            "/t/gunbc: {} {} {} {} /nowhere/.git/index\n",
            manifest.display(),
            old.display().to_string().replace(' ', "\\ "),
            new.display(),
            ws.join("gone.rs").display()
        );
        let inputs = dep_info_inputs(&text).unwrap();
        assert_eq!(inputs.len(), 5);
        let classified = classify_binary_inputs(&inputs, &ws, built_at).unwrap();
        assert_eq!(
            classified.len(),
            4,
            "the .git path is skipped: {classified:?}"
        );
        assert!(classified.contains(&BinaryInput::NotNewer(old.display().to_string())));
        assert!(classified.contains(&BinaryInput::Newer(new.display().to_string())));
        assert!(classified.contains(&BinaryInput::Missing(
            ws.join("gone.rs").display().to_string()
        )));
        let elsewhere = classify_binary_inputs(&inputs, &dir.join("other"), built_at).unwrap();
        assert_eq!(
            elsewhere[0],
            BinaryInput::OutsideWorktree(ws.display().to_string())
        );
        // NO MANIFEST, NO VERDICT: dep-info without the crate's Cargo.toml refuses rather than
        // judging the other inputs by mtime alone (review 74282's silent-Fresh case).
        let no_manifest = dep_info_inputs(&format!(
            "/t/gunbc: {}\n",
            old.display().to_string().replace(' ', "\\ ")
        ))
        .unwrap();
        assert!(classify_binary_inputs(&no_manifest, &dir.join("other"), built_at).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// THE MID-BUILD EDIT (calm-boar-904): build_start < input_mtime < binary_mtime must be
    /// StaleBinary. Against the binary's mtime it read Fresh. Laid out as cargo lays it out: an
    /// uplifted binary hard-linked to deps/<name>-<hash>, and .fingerprint/<pkg>-<hash>/dep-bin-<name>
    /// stamped at build start. A decoy unit with the same file name and a later stamp must not be
    /// chosen, because only the hash names the unit.
    #[test]
    fn an_input_edited_during_the_build_is_stale() {
        use std::time::{Duration, SystemTime};
        let dir = std::env::temp_dir().join(format!("anchor-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let ws = dir.join("ws");
        let release = dir.join("target/release");
        std::fs::create_dir_all(ws.join("src/v1/stage0")).unwrap();
        std::fs::create_dir_all(release.join("deps")).unwrap();
        std::fs::create_dir_all(release.join(".fingerprint/pkg-abc123")).unwrap();
        std::fs::create_dir_all(release.join(".fingerprint/pkg-decoy99")).unwrap();
        let t0 = SystemTime::now() - Duration::from_secs(100);
        let stamp = |path: &std::path::Path, at: SystemTime| {
            std::fs::File::options()
                .write(true)
                .open(path)
                .unwrap()
                .set_modified(at)
                .unwrap();
        };
        let start = release.join(".fingerprint/pkg-abc123/dep-bin-gunbc");
        std::fs::write(&start, "").unwrap();
        stamp(&start, t0);
        let decoy = release.join(".fingerprint/pkg-decoy99/dep-bin-gunbc");
        std::fs::write(&decoy, "").unwrap();
        stamp(&decoy, t0 + Duration::from_secs(80));
        let manifest = ws.join("src/v1/stage0/Cargo.toml");
        std::fs::write(&manifest, "").unwrap();
        stamp(&manifest, t0 - Duration::from_secs(10));
        let edited = ws.join("edited.rs");
        std::fs::write(&edited, "").unwrap();
        stamp(&edited, t0 + Duration::from_secs(30));
        let linked = release.join("deps/gunbc-abc123");
        std::fs::write(&linked, "bin").unwrap();
        stamp(&linked, t0 + Duration::from_secs(60));
        let exe = release.join("gunbc");
        std::fs::hard_link(&linked, &exe).unwrap();
        std::fs::write(
            release.join("gunbc.d"),
            format!(
                "{}: {} {}\n",
                exe.display(),
                manifest.display(),
                edited.display()
            ),
        )
        .unwrap();
        assert_eq!(build_start_anchor(&exe).unwrap(), t0);
        match assess_binary_freshness(&observe_binary_inputs_of(&exe, &ws)) {
            BinaryFreshness::Stale { input, .. } => assert_eq!(input, edited.display().to_string()),
            other => panic!("a mid-build edit must be stale: {other:?}"),
        }
        // THE OLD ANCHOR, FOR CONTRAST: against the binary's own mtime the same tree reads fresh.
        let at_link = std::fs::metadata(&exe).unwrap().modified().unwrap();
        let inputs =
            dep_info_inputs(&std::fs::read_to_string(release.join("gunbc.d")).unwrap()).unwrap();
        assert!(matches!(
            assess_binary_freshness(&BinaryInputObservation::Observed {
                binary: "b".into(),
                inputs: classify_binary_inputs(&inputs, &ws, at_link).unwrap()
            }),
            BinaryFreshness::Fresh { .. }
        ));
        // A copied binary has no hard-link twin: undecided, not fresh.
        let copied = release.join("copied");
        std::fs::copy(&linked, &copied).unwrap();
        assert!(build_start_anchor(&copied).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// THE ROUTE: a stale verdict refuses with status 2 and never reaches the producer.
    #[test]
    fn a_stale_binary_is_refused_before_any_instrument_runs() {
        let stale = BinaryFreshness::Stale {
            binary: "bin".into(),
            input: "a.rs".into(),
            why: "changed after the binary was built".into(),
        };
        let outcome = stale_binary_refusal(&stale).expect("a stale binary is refused");
        assert_eq!(outcome.termination, Termination::Refused);
        assert!(
            outcome.message.contains("cause=StaleBinary"),
            "{}",
            outcome.message
        );
        assert!(outcome.message.contains("a.rs"));
    }
}

/// `gunbc.instrument_targets` `interpolation_hole_census_label`. TRANSPORT ONLY: every `.dag` file
/// under the source roots is read and handed to `v1.tests.claim.interpolation_hole_census` as
/// `SourceFile` data; that module decides which files are subjects (those whose v1 lexing yields an
/// interpolating template), the host loads each subject's closure, and the union is handed back for
/// one compile. The standing, every row, every count and every completeness mismatch are that
/// module's; this function holds only when the fixture standing AND the receipt's corpus standing (its first line) both hold. An unreadable root,
/// file, fixture or closure is `SubjectUnreached`, never a standing.
fn run_interpolation_hole_census(source_roots: &[String]) -> InvocationOutcome {
    use crate::v1_compiler_compile::SourceFile;
    use crate::v1_tests_claim_interpolation_hole_census as census;
    use std::rc::Rc;
    const FIXTURE: &str = "fixtures/interpolation_hole_census/a.dag";
    let unreached = |detail: String| InvocationOutcome {
        termination: Termination::SubjectUnreached,
        message: format!("interpolation-hole-census: subject unreached: {detail}"),
    };
    let fixture = match std::fs::read_to_string(FIXTURE) {
        Ok(content) => vec![Rc::new(SourceFile {
            path: FIXTURE.to_string(),
            content,
        })],
        Err(err) => return unreached(format!("fixture {FIXTURE}: {err}")),
    };
    let standing = census::interpolation_hole_fixture_standing(Rc::new(fixture.into()));
    if standing.starts_with("REFUSED") {
        return unreached(format!("fixture did not compile: {standing}"));
    }
    for line in standing.lines() {
        println!("interpolation-hole-census: {line}");
    }
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    for root in source_roots {
        if let Err(detail) =
            cli_run::collect_dag_files_result(std::path::Path::new(root), &mut paths)
        {
            return unreached(format!("root {root}: {detail}"));
        }
    }
    let mut corpus: Vec<Rc<SourceFile>> = Vec::new();
    for path in &paths {
        let path = path.to_string_lossy().to_string();
        match std::fs::read_to_string(&path) {
            Ok(content) => corpus.push(Rc::new(SourceFile { path, content })),
            Err(err) => return unreached(format!("corpus file {path}: {err}")),
        }
    }
    let corpus_files = corpus.len();
    let subjects: Vec<String> = census::interpolation_hole_census_subjects(Rc::new(corpus.into()))
        .iter()
        .map(|p| p.to_string())
        .collect();
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut closure: Vec<Rc<SourceFile>> = Vec::new();
    for subject in &subjects {
        let sources =
            match cli_run::load_sources_for_entry_with_pool_index(source_roots, subject, false) {
                Ok(sources) => sources,
                Err(detail) => return unreached(format!("closure of {subject}: {detail}")),
            };
        for source in sources {
            if seen.insert(source.path.clone()) {
                closure.push(source);
            }
        }
    }
    let compiled = closure.len();
    let receipt = census::interpolation_hole_census_from_sources(
        Rc::new(closure.into()),
        Rc::new(subjects.clone().into()),
    );
    if receipt.starts_with("REFUSED") {
        return unreached(format!("subject closure did not compile: {receipt}"));
    }
    for line in receipt.lines() {
        println!("interpolation-hole-census: report {line}");
    }
    let held = standing.lines().next() == Some("STANDING held")
        && receipt.lines().next() == Some("CORPUS complete");
    InvocationOutcome {
        termination: if held {
            Termination::ObservationHeld
        } else {
            Termination::ObservationDidNotHold
        },
        message: format!(
            "interpolation-hole-census: {} {} (controls over {FIXTURE}; report over {} subjects of {corpus_files} corpus files, {compiled} sources compiled)",
            standing.lines().next().unwrap_or("STANDING absent"),
            receipt.lines().next().unwrap_or("CORPUS absent"),
            subjects.len()
        ),
    }
}

/// THE DEPENDENCY-DEMAND CENSUS PRODUCER (D13 step b2; `gunbc.instrument_targets`
/// `dependency_demand_census_label`). The seed emits and builds the compiler closure exactly as the
/// self-host step does, and spawns it in `demand-census` mode over the same corpus; every line and
/// the exit are decided by `v2.compiler.compile` `native_demand_census_output`, so the host adds no
/// verdict: exit 0 is the census holding (its rows account for every uses fn), 1 is a census finding,
/// and 2 or a refused preparation is no observation.
/// `gunbc.instrument_targets` `generic_identity_census_label`. TRANSPORT ONLY: the fixture and the
/// `v2.compiler.compile` closure are read from disk and handed to
/// `v1.tests.claim.generic_identity_census` as `SourceFile` data. The standing line, each control,
/// the report's counts and the unobserved populations are that module's; this function maps the
/// first line of the returned standing to a termination and decides nothing else. A fixture or
/// closure that cannot be read is `SubjectUnreached`, never a standing.
fn run_generic_identity_census(source_roots: &[String]) -> InvocationOutcome {
    use crate::v1_compiler_compile::SourceFile;
    use crate::v1_tests_claim_generic_identity_census as census;
    use std::rc::Rc;
    const FIXTURE: &str = "fixtures/generic_identity_census/a.dag";
    const REPORT_ENTRY: &str = "src/v2/compiler/00_compile.dag";
    let unreached = |detail: String| InvocationOutcome {
        termination: Termination::SubjectUnreached,
        message: format!("generic-identity-census: subject unreached: {detail}"),
    };
    let fixture = match std::fs::read_to_string(FIXTURE) {
        Ok(content) => vec![Rc::new(SourceFile {
            path: FIXTURE.to_string(),
            content,
        })],
        Err(err) => return unreached(format!("fixture {FIXTURE}: {err}")),
    };
    let standing = census::generic_identity_fixture_standing(Rc::new(fixture.into()));
    if standing.starts_with("REFUSED") {
        return unreached(format!("fixture did not compile: {standing}"));
    }
    for line in standing.lines() {
        println!("generic-identity-census: {line}");
    }
    let closure =
        match cli_run::load_sources_for_entry_with_pool_index(source_roots, REPORT_ENTRY, false) {
            Ok(sources) => sources,
            Err(detail) => return unreached(format!("closure of {REPORT_ENTRY}: {detail}")),
        };
    let subjects = closure.len();
    let report = census::generic_identity_census_summary_from_sources(Rc::new(closure.into()));
    if report.starts_with("REFUSED") {
        return unreached(format!(
            "closure of {REPORT_ENTRY} did not compile: {report}"
        ));
    }
    for line in report.lines() {
        println!("generic-identity-census: report {line}");
    }
    let held = standing.lines().next() == Some("STANDING held");
    InvocationOutcome {
        termination: if held {
            Termination::ObservationHeld
        } else {
            Termination::ObservationDidNotHold
        },
        message: format!(
            "generic-identity-census: {} (controls over {FIXTURE}; report over the closure of {REPORT_ENTRY}, {subjects} sources)",
            standing.lines().next().unwrap_or("STANDING absent")
        ),
    }
}

fn run_dependency_demand_census(source_roots: &[String]) -> InvocationOutcome {
    match cli_run::run_v2_demand_census(source_roots) {
        Ok(code) => InvocationOutcome {
            termination: match code {
                0 => Termination::ObservationHeld,
                1 => Termination::ObservationDidNotHold,
                _ => Termination::SubjectUnreached,
            },
            message: format!(
                "dependency-demand-census: exit {code}; the [demand-census] lines above are the census"
            ),
        },
        Err(cause) => InvocationOutcome {
            termination: Termination::SubjectUnreached,
            message: cause,
        },
    }
}

/// `gunbc test //gunbc/instruments:regen-round-cost`: one whole-population regen round, priced
/// by phase on two clocks (`gunbc.regen_round_cost`), then the one-mirror discriminator over the
/// same corpus. On-demand only: it is reached through instrument-dispatch and no required job.
///
/// The probe's mirror is a fixed fact of the instrument rather than an option, so two runs
/// measure the same selection. `std_measure.rs` is an ordinary leaf mirror of the population.
const REGEN_ROUND_COST_PROBE_MIRROR: &str = "std_measure.rs";

fn run_regen_round_cost_instrument() -> InvocationOutcome {
    let unreached = |detail: String| InvocationOutcome {
        termination: Termination::SubjectUnreached,
        message: format!("regen-round-cost: subject unreached: {detail}"),
    };
    let roots = self_host_source_roots();
    let round = match cli_run::run_regen_round_cost(
        "target/stage0-regen-candidate",
        "target/stage0-regen-receipt.json",
        &roots,
        false,
    ) {
        Ok(round) => round,
        Err(e) => return unreached(format!("round: {e}")),
    };
    print!("{}", round.rendered);
    let rows = match cli_run::run_regen_one_mirror_emit_probe(REGEN_ROUND_COST_PROBE_MIRROR) {
        Ok(rows) => rows,
        Err(e) => return unreached(format!("one-mirror probe: {e}")),
    };
    for row in &rows {
        println!(
            "regen-round-cost: one-mirror-probe mirror={} phase={} wall_ms={} cpu_ms={}",
            REGEN_ROUND_COST_PROBE_MIRROR,
            row.label,
            row.wall_ms,
            row.cpu_ms
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unreadable".to_string())
        );
    }
    if round.round_failures.is_empty() {
        InvocationOutcome {
            termination: Termination::ObservationHeld,
            message: format!(
                "regen-round-cost: round clean; receipt={}",
                round.receipt_path.display()
            ),
        }
    } else {
        InvocationOutcome {
            termination: Termination::ObservationDidNotHold,
            message: format!(
                "regen-round-cost: round not clean: {}",
                round.round_failures.join("; ")
            ),
        }
    }
}
