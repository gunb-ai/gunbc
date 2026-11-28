// THE HOST REALIZATION OF `gunbc test <target-pattern>`, AND IT IS A HAND MIRROR OF A `.dag`
// AUTHORITY.
//
// The model is `gunbc.target_invocation` (operand admission, generic exact-label route,
// termination vocabulary), `gunbc.instrument_targets` (live target and binding rows, the
// differential's classifier and rendering), `extdeps.bazel.label` (the label grammar mirrored
// here) and `extdeps.bazel.target_pattern` (the pattern grammar mirrored here). The SET forms
// (`//pkg:all`, `//pkg:*`, `//pkg/...`) execute only inside a route's universe: the native test
// route (`//v2/...`) or the claim route (`//test/claim/...`, the discovered witness claims,
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
// `run_producer` — the peripheral realization dispatch gunbc.design_argument::single_authority keeps out of the
// interface. Deliberately NOT here: any consultation of `//:required` aggregate policy or the
// Blaze status export — both refuse instrument producers by design — and any re-implementation
// of witness selection, which is `gunbc.compute.test_selection`'s to own.

use crate::cli_run;
use crate::instrument_registry_generated::{
    bare_reference_channel_source_roots, emitted_crate_workspace_source_roots,
    heads_reading_differential_source_roots, self_host_source_roots, v2_native_cli_source_roots,
};

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

/// `gunbc.target_binding` `TargetProducer`, narrowed to the members this seam realizes today.
/// Adding one is a row in `instrument_registry` and an arm here; it is not a new route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetProducer {
    SelfHost,
    V2NativeCli,
    V2NativeFrontier,
    EmittedCrateWorkspace,
    HeadsReadingDifferential,
    TypedGraphExclusiveBytes,
    TypedGraphExclusiveBytesFloorSubject,
    BareReferenceChannelOutcome,
    SelfHostBehavioralEquivalence,
    SelfHostFrontier,
    CardinalityRealization,
    RegenRoundCost,
    /// `NativeClaimProgramProducer { entry, deadline }`: the entry is carried so a second program
    /// of the same shape is a registry row, never another variant. The Millisecond deadline is
    /// read from `gunbc.instrument_targets` `native_claim_child_deadline_count` at spawn.
    NativeClaimProgram {
        entry: &'static str,
    },
    /// `NativeServeProgramProducer { entry }`: generic over its entry on the same rule.
    NativeServeProgram {
        entry: &'static str,
    },
    /// Generated from a CliWireProgramProducer declaration: wet interpreter eval of a CliWireResponse
    /// fold. Generic over entry and function so a second such instrument is a registry row.
    CliWireProgram {
        entry: &'static str,
        function: &'static str,
    },
}
/// Modules this producer evaluates by name at spawn. Parallel to
/// `REQUIRED_FLOOR_RUNTIME_AUTHORITY_MODULES`: listing here is what makes that by-name
/// evaluation legitimate rather than an undeclared reach. Not a floor-subject seed — the floor
/// list must not grow this module's closure.
const NATIVE_CLAIM_RUNTIME_AUTHORITY_MODULES: [&str; 1] = ["gunbc.instrument_targets"];
/// THE INSTRUMENT TABLE THE DISPATCH READS, RENDERED FROM THE MODEL'S BINDING ROWS.
///
/// This function used to author its own rows: each spelled an instrument's subject -- the entry path
/// the host evaluates, the wire function it calls, the census verb it runs -- beside the label, while
/// `gunbc.instrument_targets` `instrument_bindings()` declared the same subject once. Two owners for
/// one subject, with the dispatch reading the copy no authority owns, is gunbc.design_argument::single_authority's fork: a row here
/// could go stale and nothing consulted the model, which is the class PR 15298 is the specimen of.
/// The rows are now rendered from the roster by `gunbc.instruments.instrument_registry_emit` into
/// `instrument_registry_generated.rs`, a seed-build artifact
/// (`gunbc.seed_build_artifact_emit` `seed_build_artifacts`). What that closes, and what it does not,
/// is stated at the rung the ledger records --
/// `gunbc.recurring_failure_mode.instrument_subject_bound_twice_the_registry_reads_the_unbound_copy`,
/// `Mitigatable` with ceiling `StructurallyGuaranteed`. Closes: the dispatch reads ONE copy, so the
/// hand-written second copy is gone, and a committed table naming a subject this checkout lacks is
/// refused at dispatch, with a located cause at the instrument contract's exit 2. Does not close: a
/// table merely BEHIND its roster still compiles and still dispatches the old subject, and NOTHING on
/// the required path compares the two. The required context is the single `v2-bootstrap` job of
/// `.github/workflows/witnesses.yml`: it builds the compiler, compiles the v1 unit-test harness with
/// `cargo test --release -p v1-compiler --lib --no-run` -- a step whose own name says no assertion runs
/// there, so `registry_matches_rows` is compiled and never executed -- and emits and builds the
/// self-host and v2-native-cli instruments. The comparison claim this change files,
/// `test.claim.instrument_registry_artifact_rendering_witness`, is selected by no floor either; it runs
/// under bounded, operator-authorized runs. That non-execution is not a new gap: it is a member of
/// `gunbc.rung_drop.v1_required_lanes_withdrawn`, and that row's restoration trigger is the trigger
/// here. heal is not a reader of staleness either: it judges whether a generated file belongs to the
/// population, refuses no merge, and is not a required context. Staleness here is mitigated, not
/// refused.
fn instrument_registry() -> Vec<(Label, TargetProducer)> {
    crate::instrument_registry_generated::instrument_registry_rows()
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
/// target is unwritable (gunbc.design_argument::guarantee_ladder, structural impossibility) and an unconstructible arm
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
    TargetIsUnknown {
        target: String,
    },
    /// A generated row names a subject -- the entry path the host would evaluate -- that this
    /// checkout does not carry. The table and the corpus disagree; running the row would report on a
    /// subject that is gone, so the dispatch refuses instead (gunbc.design_argument::fail_closed: a refusal, not a plausible
    /// line). `Termination::Refused` is the instrument contract's exit 2.
    InstrumentSubjectAbsent {
        label: String,
        field: String,
        path: String,
    },
}

/// THE REFUSAL NAMES WHAT WOULD HAVE WORKED, AND IT IS DERIVED RATHER THAN WRITTEN DOWN.
///
/// A reader who has forgotten the label is exactly the reader holding this refusal, so the roster
/// belongs here and not only in a document. It is read from `instrument_registry` -- the same rows
/// the lookup just failed against -- so a listing cannot disagree with what is invocable: adding an
/// instrument updates this text by construction, and a prose page listing them would be the second
/// representation gunbc.design_argument::minimize_redundancy, gunbc.design_argument::single_authority price, drifting the first time someone adds a row and
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
            InvocationRefusal::InstrumentSubjectAbsent { label, field, path } => {
                format!(
                    "gunbc test: instrument {label} names {field} \"{path}\" which is absent from this checkout; the instrument table is stale against the corpus -- regenerate src/v1/stage0/src/instrument_registry_generated.rs from gunbc.instrument_targets instrument_bindings() rather than run a subject that is gone"
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
/// them is the absorbing answer gunbc.design_argument::fail_closed forbids. Wildcard-free: a fifth termination must
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

/// IS THE SUBJECT IN THIS CHECKOUT? Resolved against the host's ONE checkout root:
/// `cli_run::workspace_root` -- the nearest ancestor of the process working directory that carries a
/// `.git`, refused loudly when the process runs outside any checkout. A dispatch runs from the
/// repository root, which is where a subject is named from; a `cargo test` process starts in the
/// package directory, and the host's root derivation answers that too. The root is derived once and
/// never guessed by searching, so a subject cannot be satisfied by a file in an unrelated ancestor or
/// in an outer checkout: gunbc.design_argument::fail_closed refuses an unavailable precise answer instead of widening it.
fn instrument_subject_is_present(path: &str) -> bool {
    cli_run::workspace_root().join(path).exists()
}

/// THE SUBJECT A GENERATED ROW NAMES MUST BE IN THE CHECKOUT BEFORE IT RUNS.
///
/// The table is rendered from `gunbc.instrument_targets`, so a row whose entry path the checkout does
/// not carry can only mean the committed table is stale against the corpus -- the shape of the defect
/// this change removes, where the host read its own copy and a gone subject kept being reported on as
/// though it were present. The dispatch therefore refuses, located and typed, at the instrument
/// contract's exit 2 (`Termination::Refused`), rather than run what is no longer there.
fn instrument_subject_absence_refusal(
    producer: &TargetProducer,
    label: &str,
) -> Option<InvocationRefusal> {
    let (field, path) = match producer {
        TargetProducer::NativeClaimProgram { entry, .. } => ("entry", entry),
        TargetProducer::NativeServeProgram { entry, .. } => ("entry", entry),
        TargetProducer::CliWireProgram { entry, .. } => ("entry", entry),
        _ => return None,
    };
    if instrument_subject_is_present(path) {
        None
    } else {
        Some(InvocationRefusal::InstrumentSubjectAbsent {
            label: label.to_string(),
            field: field.to_string(),
            path: path.to_string(),
        })
    }
}

/// THE REALIZATION DISPATCH, AND IT IS THE ONLY PLACE A PRODUCER IS NAMED. Selecting a realization
/// is itself realization (gunbc.design_argument::single_authority): periphery, never the route above or the CLI surface.
///
/// Its first act is the subject gate: a generated row whose subject the checkout does not carry
/// refuses, located, before any producer is selected (see `instrument_subject_absence_refusal`).
fn run_producer(producer: TargetProducer, label: &str) -> InvocationOutcome {
    if let Some(refusal) = instrument_subject_absence_refusal(&producer, label) {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: invocation_refusal_rendered(&refusal),
        };
    }
    match producer {
        TargetProducer::HeadsReadingDifferential => {
            run_heads_reading_differential(&heads_reading_differential_source_roots())
        }
        TargetProducer::SelfHost => run_self_host(&self_host_source_roots()),
        TargetProducer::V2NativeCli => run_v2_native_cli(&v2_native_cli_source_roots()),
        TargetProducer::V2NativeFrontier => run_v2_native_frontier(&self_host_source_roots()),
        TargetProducer::NativeClaimProgram { entry } => run_native_claim_program(entry),
        TargetProducer::NativeServeProgram { entry } => run_native_serve_program(entry),
        TargetProducer::CliWireProgram { entry, function } => run_cli_wire_program(entry, function),
        TargetProducer::EmittedCrateWorkspace => {
            run_emitted_crate_workspace(&emitted_crate_workspace_source_roots())
        }
        TargetProducer::TypedGraphExclusiveBytes => run_typed_graph_exclusive_bytes(),
        TargetProducer::TypedGraphExclusiveBytesFloorSubject => {
            run_typed_graph_exclusive_bytes_floor_subject()
        }
        TargetProducer::BareReferenceChannelOutcome => run_bare_reference_channel_outcome(),
        TargetProducer::SelfHostFrontier => run_self_host_frontier(
            &self_host_source_roots(),
            &crate::gunbc_stage0_stage0_cargo_manifest::stage0_package_root(),
        ),
        TargetProducer::CardinalityRealization => run_cardinality_realization(),
        TargetProducer::RegenRoundCost => run_regen_round_cost_instrument(),
        TargetProducer::SelfHostBehavioralEquivalence => run_cli_wire_census(
            "self-host-behavioral-equivalence",
            "dag/gunbc/instruments/self_host_behavioral_equivalence_take.dag",
            "take_self_host_behavioral_equivalence_receipt",
        ),
    }
}

/// Generic CliWireProgramProducer realization: entry and function are the binding's, never a
/// string-match inside NativeClaimProgram.
fn run_cli_wire_program(entry: &'static str, function: &'static str) -> InvocationOutcome {
    run_cli_wire_census(function, entry, function)
}

/// ONE RUNNER FOR EVERY INSTRUMENT WHOSE `.dag` ENTRY ANSWERS A `CliWireResponse`: resolve the
/// entry, evaluate the named function, print the wire bytes on every termination, and map the
/// wire exit to a Termination through the one classifier in `cli_run`. A new instrument of that
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
    let (_, ctx) = match cli_run::load_entry_for_functions(
        &roots,
        entry,
        &[function],
        crate::v1_interpreter::ExecutionMode::Wet,
    ) {
        Ok(loaded) => loaded,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label_name}: resolve failed for {entry}: {cause}"),
            };
        }
    };
    let value = match cli_run::run_named_function(&ctx, function, &[]) {
        Ok(value) => value,
        Err(refusal) => {
            let cause = match refusal {
                cli_run::NamedFunctionRefusal::FunctionAbsent => {
                    format!("function `{function}` is absent")
                }
                cli_run::NamedFunctionRefusal::ArgumentRefused(cause) => cause,
                cli_run::NamedFunctionRefusal::EvaluationFailed(error) => error.to_string(),
            };
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label_name}: eval failed: {cause}"),
            };
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

/// THE SELF-HOST PRODUCER: the generation this repository can perform, asked as one question.
///
/// It calls the SAME producer `--v2-native-route` calls, so the two cannot disagree about whether
/// the seed can build v2 -- only one of them decides it. That route then spends roughly nine
/// further minutes executing the marked `v2` declarations through the emitted binary, which is a
/// different claim (what the emitted compiler ANSWERS), and bundling it here would price the
/// self-host question at the cost of a question nobody asked.
///
/// THE THREE TERMINATIONS ARE NOT TWO. A refusal from `prepare_emitted_compiler` is the subject
/// failing to be reached -- the emit refused, the crate would not write, cargo could not run --
/// which is `SubjectUnreached` rather than an observation that the seed cannot build v2. Only a
/// completed build whose own counters are non-zero is `ObservationDidNotHold`. Collapsing those is
/// the absorbing answer gunbc.design_argument::fail_closed forbids, and here it would report a broken bench as a
/// broken compiler.
fn run_self_host(source_roots: &[String]) -> InvocationOutcome {
    match cli_run::run_self_host(source_roots) {
        Ok(held) => {
            // THE TERMINATION IS DERIVED FROM THE CAUSE, so a did-not-hold cannot exit 1 without
            // printing why: the cause IS the discriminator (gunbc.design_argument::fail_closed).
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
/// `v2.workflow.native_frontier_roster` records. The verdict is `v2.workflow.native_frontier_ratchet`'s, decided
/// inside the emitted binary; this arm only maps its word to a termination, and that map is closed:
/// an unknown word is a harness defect, never a pass.
///
/// `held` and `advanced` are the observation holding: every planned identity reached a terminal
/// verdict and every honest failure is rostered debt. An advance also prints a proposed smaller
/// roster, which a reviewed pull request may carry (the required native-route lane publishes it). An owned
/// correctness flip (`grew-by-owned-correctness-flip`) holds for the same reason: every added
/// identity is owed debt under a declared, owned cause, and it too prints a proposed roster.
/// (v1 PURPOSE admission, `gunbc.v1.v1_maintenance_standing`: this arm only maps a v2 frontier
/// verdict word to its termination; the verdict itself is decided in `.dag`, so no seed growth.)
/// `lost` and `unminted` are the observation
/// not holding. `unminted` is a complete run with nothing to hold it to, and an empty roster read as
/// no debt would be a vacuous pass. `not-a-measurement` means the receipt failed an integrity
/// clause or the pattern was narrower than the universe, so the subject was not reached.
fn run_v2_native_frontier(source_roots: &[String]) -> InvocationOutcome {
    match cli_run::run_v2_native_frontier(source_roots) {
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
                             v2.workflow.native_frontier_ratchet native_frontier_verdict_word and this match must agree"
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
            // printing why: the cause IS the discriminator (gunbc.design_argument::fail_closed).
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
fn native_claim_child_deadline_from_binding(
    entry: &str,
    label_name: &str,
) -> Result<std::time::Duration, InvocationOutcome> {
    const READER: &str = "dag/gunbc/instrument_targets.dag";
    let authority = NATIVE_CLAIM_RUNTIME_AUTHORITY_MODULES[0];
    let ungrounded = |detail: String| -> InvocationOutcome {
        InvocationOutcome {
            termination: Termination::Refused,
            message: format!(
                "{label_name}: NATIVE-CLAIM REFUSAL cause=NativeClaimDeadlineUngrounded \
                 module={authority} entry={entry} — {detail}"
            ),
        }
    };
    const DEADLINE: &str = "native_claim_child_deadline_count";
    let roots = cli_run::default_source_roots();
    let (_, ctx) = match cli_run::load_entry_for_functions(
        &roots,
        READER,
        &[DEADLINE],
        crate::v1_interpreter::ExecutionMode::Wet,
    ) {
        Ok(resolved) => resolved,
        Err(cause) => {
            return Err(ungrounded(format!("resolve failed for {READER}: {cause}")));
        }
    };
    let value = match crate::v1_interpreter::run_in_context_with_args(
        &ctx,
        DEADLINE,
        &[(
            Some("entry".to_string()),
            crate::v1_interpreter::str_value(entry),
        )],
        true,
    ) {
        Ok(value) => value,
        Err(cause) => {
            return Err(ungrounded(format!(
                "native_claim_child_deadline_count failed: {cause}"
            )));
        }
    };
    let crate::v1_interpreter::Value::Variant {
        variant_name,
        fields,
        ..
    } = &value
    else {
        return Err(ungrounded(
            "native_claim_child_deadline_count is not an Optional".to_string(),
        ));
    };
    if ctx.sym_eq(*variant_name, "Absent") {
        return Err(ungrounded(
            "no NativeClaimProgramProducer binding for this entry".to_string(),
        ));
    }
    if !ctx.sym_eq(*variant_name, "Present") {
        return Err(ungrounded(
            "native_claim_child_deadline_count is not Present or Absent".to_string(),
        ));
    }
    match ctx.field(fields, "value") {
        Some(crate::v1_interpreter::Value::Int(n)) if *n > 0 => {
            Ok(std::time::Duration::from_millis(*n as u64))
        }
        Some(crate::v1_interpreter::Value::Int(n)) => Err(ungrounded(format!(
            "native_claim_child_deadline_count returned {n}ms — a child deadline must be strictly positive"
        ))),
        other => Err(ungrounded(format!(
            "native_claim_child_deadline_count expected a positive Int, got {other:?}"
        ))),
    }
}

/// THE CARDINALITY REALIZATION INSTRUMENT (`gunbc.cardinality_realization_seed_growth`). The plan,
/// the join to the coverage rows and the exit are `v2.test.claim.cardinality_realization_coverage`
/// (`instrument_plan`, `cardinality_realization_standing`); this host only runs what the plan names
/// and reports each outcome as a typed `ObservationKind` record: `ObservedRealized` (the program printed `tag=<n>`), `ObservedRefused`
/// (the seed emit or the native entry module refused, with its cause), `ObservedRed` (the emitted program
/// failed to build or run), `ObservedUnreached` (no observation could be taken). Emission is never read as
/// execution: a native module that emitted is `ObservedEmitted`.
fn run_cardinality_realization() -> InvocationOutcome {
    use crate::v1_interpreter::{list_value, str_value, Value};
    let label_name = "cardinality-realization";
    const READER: &str = "src/v2/test/claim/cardinality_realization_coverage.dag";
    let unreached = |message: String| InvocationOutcome {
        termination: Termination::SubjectUnreached,
        message: format!("{label_name}: {message}"),
    };
    if let Err(e) = std::env::set_current_dir(cli_run::workspace_root()) {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!("{label_name}: refused: could not anchor at the workspace root: {e}"),
        };
    }
    const PLAN: &str = "instrument_plan";
    const STANDING: &str = "cardinality_realization_standing";
    let roots = v2_native_cli_source_roots();
    let (_, ctx) = match cli_run::load_entry_for_functions(
        &roots,
        READER,
        &[PLAN, STANDING],
        crate::v1_interpreter::ExecutionMode::Wet,
    ) {
        Ok(resolved) => resolved,
        Err(cause) => return unreached(format!("resolve failed for {READER}: {cause}")),
    };
    let plan_value = match crate::v1_interpreter::run_in_context_with_args(&ctx, PLAN, &[], true) {
        Ok(value) => value,
        Err(cause) => return unreached(format!("instrument_plan failed: {cause}")),
    };
    let Value::List(plan) = &plan_value else {
        return unreached(format!(
            "instrument_plan returned {plan_value:?}, not a list"
        ));
    };
    // One planned pair: the route (read from the typed variant), and the target and specimen
    // references, kept as the plan's own values so the outcome echoes them back unchanged.
    struct Planned {
        seed_entry: Option<String>,
        native_module: Option<String>,
        target: Value,
        specimen: Value,
    }
    let field = |fields: &[(crate::v1_interpreter::Symbol, Value)], name: &str| {
        let want = ctx.sym(name);
        fields
            .iter()
            .find(|(k, _)| *k == want)
            .map(|(_, v)| v.clone())
    };
    let mut planned: Vec<Planned> = Vec::new();
    for item in plan.iter() {
        let Value::Record { fields, .. } = item else {
            return unreached(format!(
                "instrument_plan row {item:?} is not a PlannedObservation"
            ));
        };
        let (Some(route), Some(target), Some(specimen)) = (
            field(fields, "route"),
            field(fields, "target"),
            field(fields, "specimen"),
        ) else {
            return unreached(format!(
                "instrument_plan row {item:?} lacks route, target or specimen"
            ));
        };
        let Value::Variant {
            variant_name,
            fields: route_fields,
            ..
        } = &route
        else {
            return unreached(format!(
                "instrument_plan route {route:?} is not a PlannedRoute"
            ));
        };
        let text = |name: &str| {
            let found = field(route_fields, name);
            match &found {
                Some(Value::Str(s)) => Some(s.to_string()),
                _ => None,
            }
        };
        let (seed_entry, native_module) = if *variant_name == ctx.sym("SeedRoute") {
            (text("entry"), None)
        } else if *variant_name == ctx.sym("NativeRoute") {
            (None, text("module"))
        } else {
            return unreached(format!(
                "instrument_plan route {route:?} is neither SeedRoute nor NativeRoute"
            ));
        };
        if seed_entry.is_none() && native_module.is_none() {
            return unreached(format!(
                "instrument_plan route {route:?} names no entry or module"
            ));
        }
        planned.push(Planned {
            seed_entry,
            native_module,
            target,
            specimen,
        });
    }
    let deadline = std::time::Duration::from_secs(1800);
    // (planned index, ObservationKind variant name, its one field name, its text)
    let mut outcomes: Vec<(usize, &'static str, &'static str, String)> = Vec::new();
    for (i, p) in planned.iter().enumerate() {
        let Some(entry) = &p.seed_entry else { continue };
        let outcome = match cli_run::run_native_claim_program(&roots, entry, deadline) {
            Ok(cli_run::NativeClaimProgramAttempt::Completed(run)) => {
                let tag = run
                    .stdout
                    .lines()
                    .find_map(|l| l.strip_prefix("tag="))
                    .map(str::to_string);
                match (run.status, tag) {
                    (Some(0), Some(tag)) => ("ObservedRealized", "tag", tag),
                    (status, _) => (
                        "ObservedRed",
                        "detail",
                        format!("exit {status:?}: {}", run.stderr.trim_end()),
                    ),
                }
            }
            Ok(cli_run::NativeClaimProgramAttempt::DeadlineExceeded { .. }) => (
                "ObservedUnreached",
                "detail",
                format!("{entry} exceeded its deadline"),
            ),
            Err(refusal) => match refusal.cause {
                cli_run::PreparationCause::EmissionRefused => {
                    ("ObservedRefused", "cause", "EmissionRefused".to_string())
                }
                cli_run::PreparationCause::EmittedCompilerBuildFailed => {
                    ("ObservedRed", "detail", refusal.message)
                }
                cli_run::PreparationCause::Other => {
                    ("ObservedUnreached", "detail", refusal.message)
                }
            },
        };
        outcomes.push((i, outcome.0, outcome.1, outcome.2));
    }
    let mut modules: Vec<String> = planned
        .iter()
        .filter_map(|p| p.native_module.clone())
        .collect();
    modules.sort();
    modules.dedup();
    let emitted = if modules.is_empty() {
        Ok(Vec::new())
    } else {
        cli_run::run_v2_native_cli_emit(&roots, &modules)
    };
    for (i, p) in planned.iter().enumerate() {
        let Some(entry) = &p.native_module else {
            continue;
        };
        let outcome = match &emitted {
            Err(cause) => ("ObservedUnreached", "detail", cause.clone()),
            Ok(runs) => match runs.iter().find(|r| &r.0 == entry) {
                None => (
                    "ObservedUnreached",
                    "detail",
                    format!("no native emit run for {entry}"),
                ),
                Some((_, status, stdout, stderr)) => {
                    let own = serde_json::from_str::<serde_json::Value>(stdout)
                        .ok()
                        .and_then(|doc| {
                            doc.get("modules")?.as_array()?.iter().find_map(|m| {
                                (m.get("module")?.as_str()? == entry.as_str()).then(|| {
                                    m.get("refused")
                                        .and_then(|r| r.as_str())
                                        .map(str::to_string)
                                })
                            })
                        });
                    match (own, status) {
                        (Some(Some(cause)), _) => ("ObservedRefused", "cause", cause),
                        (Some(None), Some(0)) => ("ObservedEmitted", "", String::new()),
                        _ => (
                            "ObservedUnreached",
                            "detail",
                            format!(
                                "native emit of {entry}: status {status:?} {}",
                                stderr.lines().next().unwrap_or("")
                            ),
                        ),
                    }
                }
            },
        };
        outcomes.push((i, outcome.0, outcome.1, outcome.2));
    }
    let rendered: Vec<String> = outcomes
        .iter()
        .map(|(i, variant, _, text)| {
            let p = &planned[*i];
            let entry = p.seed_entry.as_ref().or(p.native_module.as_ref());
            format!(
                "{} | {variant} | {}",
                entry.map(String::as_str).unwrap_or(""),
                text.lines().next().unwrap_or("")
            )
        })
        .collect();
    let observations = list_value(
        outcomes
            .iter()
            .map(|(i, variant, field_name, text)| {
                let p = &planned[*i];
                let outcome_fields = if field_name.is_empty() {
                    Vec::new()
                } else {
                    vec![(ctx.sym(field_name), str_value(text))]
                };
                Value::Record {
                    type_name: ctx.sym("ObservationRecord"),
                    fields: std::rc::Rc::new(vec![
                        (ctx.sym("target"), p.target.clone()),
                        (ctx.sym("specimen"), p.specimen.clone()),
                        (
                            ctx.sym("outcome"),
                            Value::Variant {
                                type_name: ctx.sym("ObservationKind"),
                                variant_name: ctx.sym(variant),
                                fields: std::rc::Rc::new(outcome_fields),
                            },
                        ),
                    ]),
                }
            })
            .collect::<Vec<_>>(),
    );
    let standing = match crate::v1_interpreter::run_in_context_with_args(
        &ctx,
        STANDING,
        &[(Some("observations".to_string()), observations)],
        true,
    ) {
        Ok(value) => cli_run::classify_exit(&value, &ctx),
        Err(cause) => return unreached(format!("the standing failed: {cause}")),
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
            format!("the standing returned `{type_name}`, not a ProcessExit"),
        ),
    };
    InvocationOutcome {
        termination,
        message: format!("{}\n{label_name}: {verdict}", rendered.join("\n")),
    }
}

fn run_native_claim_program(entry: &'static str) -> InvocationOutcome {
    let label_name = "native-claim";
    if let Err(e) = std::env::set_current_dir(cli_run::workspace_root()) {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!("{label_name}: refused: could not anchor at the workspace root: {e}"),
        };
    }
    let deadline = match native_claim_child_deadline_from_binding(entry, label_name) {
        Ok(deadline) => deadline,
        Err(outcome) => return outcome,
    };
    let run = match cli_run::run_native_claim_program(
        &v2_native_cli_source_roots(),
        entry,
        deadline,
    ) {
        Ok(cli_run::NativeClaimProgramAttempt::Completed(run)) => run,
        Ok(cli_run::NativeClaimProgramAttempt::DeadlineExceeded {
            entry,
            budget,
            elapsed,
            closure_identity,
            binary_identity,
            seed_identity,
            warning_count,
        }) => {
            return InvocationOutcome {
                termination: Termination::ObservationDidNotHold,
                message: format!(
                    "{label_name}: deadline exceeded entry={entry} budget_ms={} elapsed_ms={} closure={closure_identity} binary={binary_identity} seed={seed_identity} warning_count={warning_count}",
                    budget.as_millis(),
                    elapsed.as_millis()
                ),
            };
        }
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: cause.message,
            }
        }
    };
    const READER: &str = "dag/gunbc/native_claim_program.dag";
    const STANDING: &str = "native_claim_program_standing";
    let roots = cli_run::default_source_roots();
    let (_, ctx) = match cli_run::load_entry_for_functions(
        &roots,
        READER,
        &[STANDING],
        crate::v1_interpreter::ExecutionMode::Wet,
    ) {
        Ok(resolved) => resolved,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label_name}: resolve failed for {READER}: {cause}"),
            };
        }
    };
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
    let standing =
        match crate::v1_interpreter::run_in_context_with_args(&ctx, STANDING, &args, true) {
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
            "{}{label_name}: entry={entry} closure={} binary={} seed={} warning_count={} status={status} child_wall_ms={} deadline_ms={} -- {verdict}\n{}",
            run.stdout,
            run.closure_identity,
            run.binary_identity,
            run.seed_identity,
            run.warning_count,
            run.child_wall.as_millis(),
            run.deadline.as_millis(),
            run.stderr.trim_end(),
        ),
    }
}

/// THE NATIVE SERVE PROGRAM BINDING READ: the reader module, the phases the row declares and the
/// reader's own judgment of them, all `.dag` facts the host only carries. The row is
/// `gunbc.target_binding` `NativeServeProgramProducer { entry, probe }` read through
/// `gunbc.instrument_targets` (`native_serve_probe_reader_for_entry` and the per-phase answers); the
/// join of declared to judged phases is `gunbc.native_serve_probe_plan` `native_serve_phase_refusals`,
/// exposed by the reader as `native_serve_probe_phases_refused`. A non-empty join refuses before
/// anything is built or spawned.
struct NativeServeBindingRead {
    reader: String,
    phases: cli_run::NativeServePhases,
    declared: Vec<String>,
}

/// The unix-socket phase's directory, removed when the run ends by any path, including an early
/// refusal between its creation and the run.
struct NativeServeSocketDir(std::path::PathBuf);

impl Drop for NativeServeSocketDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn read_native_serve_binding(entry: &str) -> Result<NativeServeBindingRead, String> {
    const BINDINGS: &str = "dag/gunbc/instrument_targets.dag";
    const READER: &str = "native_serve_probe_reader_for_entry";
    const DECLARED: &str = "native_serve_probe_declared_names_for_entry";
    const PEER: &str = "native_serve_probe_runs_peer";
    const REFUSED_LAUNCH: &str = "native_serve_probe_runs_refused_launch";
    const DEADLINE: &str = "native_serve_probe_runs_deadline_exit";
    const UNIX_SOCKET: &str = "native_serve_probe_runs_unix_socket";
    let roots = cli_run::default_source_roots();
    let (_, ctx) = cli_run::load_entry_for_functions(
        &roots,
        BINDINGS,
        &[
            READER,
            DECLARED,
            PEER,
            REFUSED_LAUNCH,
            DEADLINE,
            UNIX_SOCKET,
        ],
        crate::v1_interpreter::ExecutionMode::Wet,
    )
    .map_err(|cause| format!("resolve failed for {BINDINGS}: {cause}"))?;
    let ask = |function: &str| -> Result<crate::v1_interpreter::Value, String> {
        crate::v1_interpreter::run_in_context_with_args(
            &ctx,
            function,
            &[(
                Some("entry".to_string()),
                crate::v1_interpreter::str_value(entry),
            )],
            true,
        )
        .map_err(|cause| format!("{BINDINGS} {function} failed: {cause}"))
    };
    let flag = |function: &str| -> Result<bool, String> {
        match ask(function)? {
            crate::v1_interpreter::Value::Bool(b) => Ok(b),
            other => Err(format!("{function} is not a Bool: {other:?}")),
        }
    };
    let reader_value = ask(READER)?;
    let reader = match &reader_value {
        crate::v1_interpreter::Value::Variant {
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*variant_name, "Present") => match ctx.field(fields, "value") {
            Some(crate::v1_interpreter::Value::Str(s)) => s.to_string(),
            other => return Err(format!("the probe reader is not a String: {other:?}")),
        },
        _ => {
            return Err(format!(
                "no NativeServeProgramProducer binding names the entry {entry}"
            ))
        }
    };
    let declared_value = ask(DECLARED)?;
    let declared = match &declared_value {
        crate::v1_interpreter::Value::List(items) => items
            .iter()
            .map(|item| match item {
                crate::v1_interpreter::Value::Str(s) => Ok(s.to_string()),
                other => Err(format!("a declared phase is not a String: {other:?}")),
            })
            .collect::<Result<Vec<String>, String>>()?,
        other => return Err(format!("the declared phases are not a List: {other:?}")),
    };
    Ok(NativeServeBindingRead {
        reader,
        phases: cli_run::NativeServePhases {
            peer_instance: flag(PEER)?,
            refused_launch: flag(REFUSED_LAUNCH)?,
            deadline_exit: flag(DEADLINE)?,
            unix_socket: flag(UNIX_SOCKET)?,
        },
        declared,
    })
}

/// THE NATIVE SERVE PROGRAM PRODUCER: emit, build and start a `NativeIngressDriver` entry, exchange
/// the requests the binding row's probe reader names over real TCP, then let that reader decide what
/// the run established. The host decides nothing about a case, and branches on no binding: the
/// reader module and the phases come from the row (`read_native_serve_binding`), the requests and
/// every launch's argv are read from the reader module, the bytes go back to its
/// `native_serve_probe_standing` unjudged, and its `ProcessExit` maps through the one classifier
/// exactly as the claim producer's does.
fn run_native_serve_program(entry: &'static str) -> InvocationOutcome {
    let label_name = "native-serve";
    if let Err(e) = std::env::set_current_dir(cli_run::workspace_root()) {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: format!("{label_name}: refused: could not anchor at the workspace root: {e}"),
        };
    }
    let binding = match read_native_serve_binding(entry) {
        Ok(binding) => binding,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!("{label_name}: the binding row is unreadable: {cause}"),
            }
        }
    };
    #[allow(non_snake_case)]
    let READER: &str = binding.reader.as_str();
    const PHASES: &str = "native_serve_probe_phases_refused";
    const REQUESTS: &str = "native_serve_probe_requests";
    const STANDING: &str = "native_serve_probe_standing";
    const UNIX_LAUNCH: &str = "native_serve_probe_unix_launch_argv";
    const LAUNCH: &str = "native_serve_probe_launch_argv";
    const REFUSED_LAUNCHES: &str = "native_serve_probe_refused_launches";
    const DEADLINE_LAUNCHES: &str = "native_serve_probe_deadline_launches";
    const DEADLINE_REQUESTS: &str = "native_serve_probe_deadline_requests";
    let launch = if binding.phases.unix_socket {
        UNIX_LAUNCH
    } else {
        LAUNCH
    };
    let mut requested = vec![PHASES, REQUESTS, STANDING, launch];
    if binding.phases.refused_launch {
        requested.push(REFUSED_LAUNCHES);
    }
    if binding.phases.deadline_exit {
        requested.extend([DEADLINE_LAUNCHES, DEADLINE_REQUESTS]);
    }
    let roots = cli_run::default_source_roots();
    let (_, ctx) = match cli_run::load_entry_for_functions(
        &roots,
        READER,
        &requested,
        crate::v1_interpreter::ExecutionMode::Wet,
    ) {
        Ok(resolved) => resolved,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label_name}: resolve failed for {READER}: {cause}"),
            };
        }
    };
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
    // A DECLARED PHASE THE READER CANNOT JUDGE (or a judged phase nobody declared) STOPS THE LINE
    // before anything is built: the join is the reader's native_serve_probe_phases_refused.
    let declared_value = crate::v1_interpreter::Value::List(std::rc::Rc::new(
        binding
            .declared
            .iter()
            .map(|name| crate::v1_interpreter::str_value(name))
            .collect::<Vec<_>>()
            .into(),
    ));
    let refusals = crate::v1_interpreter::run_in_context_with_args(
        &ctx,
        PHASES,
        &[(Some("declared".to_string()), declared_value)],
        true,
    );
    match &refusals {
        Ok(crate::v1_interpreter::Value::List(items)) if items.is_empty() => {}
        Ok(crate::v1_interpreter::Value::List(items)) => {
            let causes: Vec<String> = items.iter().filter_map(|item| text(item)).collect();
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!(
                    "{label_name}: NATIVE-SERVE REFUSAL cause=PhaseUnsatisfied entry={entry} reader={READER} — {}",
                    causes.join("; ")
                ),
            };
        }
        Ok(other) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!(
                "{label_name}: {READER} native_serve_probe_phases_refused is not a List: {other:?}"
            ),
            }
        }
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!(
                    "{label_name}: {READER} native_serve_probe_phases_refused failed: {cause}"
                ),
            }
        }
    }
    // The requests may depend on the peer's bound port, which only exists once the peer is up (and
    // is -1 for a row declaring no peer), so they are read from the reader module with that port as
    // its argument -- still every byte there.
    let requests_for_peer_port = |peer_port: i64| -> Result<Vec<String>, String> {
        let value = crate::v1_interpreter::run_in_context_with_args(
            &ctx,
            REQUESTS,
            &[(
                Some("peer_port".to_string()),
                crate::v1_interpreter::Value::Int(peer_port),
            )],
            true,
        )
        .map_err(|cause| {
            format!("{label_name}: {READER} native_serve_probe_requests failed: {cause}")
        })?;
        match &value {
            crate::v1_interpreter::Value::List(items) => items
                .iter()
                .map(|item| text(item).ok_or("a request is not a String".to_string()))
                .collect::<Result<Vec<String>, String>>(),
            _ => Err("native_serve_probe_requests is not a List".to_string()),
        }
    };
    let deadline_requests_for_silent_port = |silent_port: i64| -> Result<Vec<String>, String> {
        let value = crate::v1_interpreter::run_in_context_with_args(
            &ctx,
            DEADLINE_REQUESTS,
            &[(
                Some("silent_port".to_string()),
                crate::v1_interpreter::Value::Int(silent_port),
            )],
            true,
        )
        .map_err(|cause| {
            format!("{label_name}: {READER} native_serve_probe_deadline_requests failed: {cause}")
        })?;
        match &value {
            crate::v1_interpreter::Value::List(items) => items
                .iter()
                .map(|item| text(item).ok_or("a deadline request is not a String".to_string()))
                .collect::<Result<Vec<String>, String>>(),
            _ => Err("native_serve_probe_deadline_requests is not a List".to_string()),
        }
    };
    let words = |value: &crate::v1_interpreter::Value| -> Option<Vec<String>> {
        match value {
            crate::v1_interpreter::Value::List(items) => {
                items.iter().map(|item| text(item)).collect()
            }
            _ => None,
        }
    };
    let argvs = |value: &crate::v1_interpreter::Value| -> Option<Vec<Vec<String>>> {
        match value {
            crate::v1_interpreter::Value::List(items) => {
                items.iter().map(|item| words(item)).collect()
            }
            _ => None,
        }
    };
    // UNDER THE UNIX-SOCKET PHASE the host chooses the socket's path, in a fresh directory only this
    // process may enter, under /tmp because a socket path is bounded by sun_path (108 bytes) and a
    // deep temp root would not fit. The reader renders the argv over that path.
    let unix_socket_dir = if binding.phases.unix_socket {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::path::PathBuf::from(format!(
            "/tmp/gunbc-native-serve-unix-{}-{nanos}",
            std::process::id()
        ));
        let created = {
            use std::os::unix::fs::DirBuilderExt;
            std::fs::DirBuilder::new().mode(0o700).create(&dir)
        };
        if let Err(cause) = created {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!(
                    "{label_name}: NATIVE-SERVE REFUSAL cause=SocketDirectoryUncreated path={} — {cause}",
                    dir.display()
                ),
            };
        }
        Some(NativeServeSocketDir(dir))
    } else {
        None
    };
    let unix_socket_path = unix_socket_dir
        .as_ref()
        .map(|dir| dir.0.join("serve.sock").display().to_string());
    // Every launch's argv is the reader's (gunbc.native_serve_probe_plan native_serve_probe_launch); a
    // phase the row does not declare owes no function.
    let unix_launch = |path: &str| -> Result<Vec<String>, String> {
        let value = crate::v1_interpreter::run_in_context_with_args(
            &ctx,
            UNIX_LAUNCH,
            &[(
                Some("socket_path".to_string()),
                crate::v1_interpreter::str_value(path),
            )],
            true,
        )
        .map_err(|cause| {
            format!("{label_name}: {READER} native_serve_probe_unix_launch_argv failed: {cause}")
        })?;
        words(&value).ok_or_else(|| "the unix launch argv is not a List of String".to_string())
    };
    // The contended paths, one per way another realization can be found at a path.
    let unix_contended_paths: Vec<(cli_run::NativeServeContention, String)> = match &unix_socket_dir
    {
        Some(dir) => vec![
            (
                cli_run::NativeServeContention::LiveListener,
                dir.0.join("live.sock").display().to_string(),
            ),
            (
                cli_run::NativeServeContention::HeldLock,
                dir.0.join("locked.sock").display().to_string(),
            ),
        ],
        None => Vec::new(),
    };
    let plan = (|| -> Result<cli_run::NativeServeLaunches, String> {
        let subject = match &unix_socket_path {
            Some(path) => unix_launch(path)?,
            None => words(&read(LAUNCH)?).ok_or("the launch argv is not a List of String")?,
        };
        let unix_contended = unix_contended_paths
            .iter()
            .map(|(contention, path)| Ok((*contention, path.clone(), unix_launch(path)?)))
            .collect::<Result<Vec<_>, String>>()?;
        let refused = if binding.phases.refused_launch {
            argvs(&read(REFUSED_LAUNCHES)?).ok_or("the refused launches are not a List of argv")?
        } else {
            Vec::new()
        };
        let deadline = if binding.phases.deadline_exit {
            argvs(&read(DEADLINE_LAUNCHES)?)
                .ok_or("the deadline launches are not a List of argv")?
        } else {
            Vec::new()
        };
        Ok(cli_run::NativeServeLaunches {
            subject,
            unix_socket_path: unix_socket_path.clone(),
            unix_contended,
            refused,
            deadline,
        })
    })();
    let launches = match plan {
        Ok(plan) => plan,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label_name}: the probe plan is unreadable: {cause}"),
            }
        }
    };
    let run = cli_run::run_native_serve_program(
        &v2_native_cli_source_roots(),
        entry,
        &launches,
        binding.phases,
        &requests_for_peer_port,
        &deadline_requests_for_silent_port,
    );
    let run = match run {
        Ok(run) => run,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: cause,
            }
        }
    };
    let statuses = |items: &[Option<i32>]| -> crate::v1_interpreter::Value {
        crate::v1_interpreter::Value::List(std::rc::Rc::new(
            items
                .iter()
                .map(|s| crate::v1_interpreter::Value::Int(i64::from(s.unwrap_or(-1))))
                .collect::<Vec<_>>()
                .into(),
        ))
    };
    let texts = |items: &[String]| -> crate::v1_interpreter::Value {
        crate::v1_interpreter::Value::List(std::rc::Rc::new(
            items
                .iter()
                .map(|r| crate::v1_interpreter::Value::Str(r.clone().into()))
                .collect::<Vec<_>>()
                .into(),
        ))
    };
    let refused_status = i64::from(
        run.refused_statuses
            .first()
            .copied()
            .flatten()
            .unwrap_or(-1),
    );
    let args = [
        (
            Some("announcement".to_string()),
            crate::v1_interpreter::Value::Str(run.announcement.clone().into()),
        ),
        (
            Some("peer_announcement".to_string()),
            crate::v1_interpreter::Value::Str(run.peer_announcement.clone().into()),
        ),
        (
            Some("peer_port".to_string()),
            crate::v1_interpreter::Value::Int(run.peer_port),
        ),
        (Some("responses".to_string()), texts(&run.responses)),
        (
            Some("refused_statuses".to_string()),
            statuses(&run.refused_statuses),
        ),
        (
            Some("refused_stderrs".to_string()),
            texts(&run.refused_stderrs),
        ),
        (
            Some("deadline_announcements".to_string()),
            texts(&run.deadline_announcements),
        ),
        (
            Some("deadline_responses".to_string()),
            texts(&run.deadline_responses),
        ),
        (
            Some("deadline_exit_statuses".to_string()),
            statuses(&run.deadline_exit_statuses),
        ),
        (
            Some("unix_peer_account".to_string()),
            crate::v1_interpreter::Value::Str(run.unix_peer_account.clone().into()),
        ),
        (
            Some("unix_contended_statuses".to_string()),
            statuses(&run.unix_contended_statuses),
        ),
        (
            Some("unix_contended_stderrs".to_string()),
            texts(&run.unix_contended_stderrs),
        ),
        (
            Some("unix_contended_kept".to_string()),
            crate::v1_interpreter::Value::List(std::rc::Rc::new(
                run.unix_contended_kept
                    .iter()
                    .map(|k| crate::v1_interpreter::Value::Bool(*k))
                    .collect::<Vec<_>>()
                    .into(),
            )),
        ),
    ];
    let standing =
        match crate::v1_interpreter::run_in_context_with_args(&ctx, STANDING, &args, true) {
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

/// `gunbc.native_route_universe` `native_route_default_pattern`, READ from the file rendered from
/// it: the native test route's whole universe. An operand CONTAINED in it belongs to that route.
///
/// THIS SEED HAS NO SPELLING OF THAT UNIVERSE. `NATIVE_ROUTE_DEFAULT_PATTERN` is a constant of
/// `native_route_seed_boundary_generated`, which `gunbc.native_route_seed_boundary_emit` renders from
/// the one declaration and the generated-artifact drift wall holds to it: moving the boundary in
/// `.dag` moves this routing at the next regeneration, and a hand edit of the constant is a drift
/// finding that regeneration undoes.
/// The declaration cannot be read here directly, because this decision is taken before any emitted
/// binary exists. The text goes through this file's own pattern parser, and a text that does not
/// parse is a refusal the caller reports, never a panic and never a fallback to some other universe.
///
/// The universe the EMITTED binary adjudicates by default is the same declaration read inside the
/// compiler (`v2.compiler.source_root_eval`, `adjudicate` without `--pattern`); the seed passes it a pattern
/// only when the operator wrote one, so nothing here is handed to it as text.
fn native_route_default_pattern() -> Result<TargetPattern, TargetPatternRefusal> {
    parse_target_pattern(crate::native_route_seed_boundary_generated::NATIVE_ROUTE_DEFAULT_PATTERN)
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
/// `v2.workflow.native_route_member_termination` `native_route_member_termination`, mirrored in the runner, so this
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
    match cli_run::run_required_v2_native(&self_host_source_roots(), Some(&rendered)) {
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
    // `v2.workflow.native_route_member_termination` `native_route_termination_under_qualification`, mirrored: the
    // lane's qualification GATES a positive answer and nothing else. Several of its clauses are
    // what establish the verdict surface is trustworthy at all -- the false and true controls, and
    // MalformedSpecimenAccepted, whose own note says that when they fail every verdict in the
    // census is worthless -- so a pass read off rows with no established provenance is the
    // fabricated plausible output gunbc.design_argument::fail_closed forbids. The clamp is ONE-DIRECTIONAL: it can
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

fn claim_route_empty_selection_note(patterns: &[TargetPattern]) -> String {
    let subtrees: Vec<String> = patterns
        .iter()
        .filter_map(|pattern| match pattern {
            TargetPattern::SingleTarget(label) => {
                Some(format!("`//{}:all`", label.package_segments.join("/")))
            }
            TargetPattern::PackageTargets(_) | TargetPattern::SubtreeTargets(_) => None,
        })
        .collect();
    if subtrees.is_empty() {
        " — the pattern selected no discovered claim".to_string()
    } else {
        format!(
            " — the pattern selected no discovered claim; an exact label names one claim, and every \
             claim of its module is {}",
            subtrees.join(", ")
        )
    }
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

/// The claim-runner protocol's own "the subject was not reached" exit code. A claim that returns a
/// typed `ProcessExit::ExitFailure` carries a protocol verdict, not an OS status: code 1 (and any
/// other code) is the runner's definite refusal -- an observation that did not hold -- while code 2
/// is the runner saying it never reached the subject. That is the same 2 the invocation reserves for
/// `SubjectUnreached` (`invocation_exit_status`), and it is read HERE, at the typed claim-result
/// boundary, because a host command's exit code is not a claim-runner protocol result: a
/// `NotProcessExit` value, whatever integer it holds, is a non-Bool observation (`NotBool`), never a
/// protocol exit, and it stays did-not-hold above.
const CLAIM_RUNNER_UNREACHED_EXIT_CODE: i32 = 2;

/// One member's outcome as the route's verdict. An evaluated claim is read by the claim-arm rule
/// below. A known selected claim that cannot execute is ALWAYS the absence of an observation -- an
/// unobserved outcome carrying its cause (plan `claim_execution_consolidation` section 2: "A known
/// selected claim that cannot execute gets an unobserved outcome with its cause") -- a resolve
/// finding is not evidence that the claim evaluated and did not hold. Wildcard-free on both layers
/// so a new arm must be classified here.
fn claim_route_verdict(outcome: &cli_run::ClaimMemberOutcome) -> ClaimRouteVerdict {
    use cli_run::{ClaimMemberOutcome as M, EntryResolveRefusal as R};
    match outcome {
        M::Evaluated(o) => evaluated_claim_verdict(o),
        // Every resolve refusal is the absence of an observation, not a failing claim. The refusal's
        // kind (ClosureDiagnostics / EntryMissing / ResolveInfrastructure) stays on the line as the
        // typed `outcome=` and its text as `detail`; only the VERDICT is uniformly unobserved.
        M::Unresolved(R::ClosureDiagnostics { .. })
        | M::Unresolved(R::EntryMissing { .. })
        | M::Unresolved(R::ResolveInfrastructure { .. }) => ClaimRouteVerdict::ClaimUnobserved,
    }
}

/// The claim-arm reading: `Fail`, `NotBool` and a definite `ExitFailure` are observations that did
/// not hold; every other arm -- an error, a refusal, an unwind, a budget interrupt, and an
/// `ExitFailure` the runner raised as "subject unreached" (code 2) -- is the absence of an
/// observation and is never reported as a failing claim.
fn evaluated_claim_verdict(outcome: &cli_run::ClaimOutcome) -> ClaimRouteVerdict {
    use cli_run::ClaimOutcome as O;
    match outcome {
        O::Pass => ClaimRouteVerdict::ClaimHeld,
        // A typed ProcessExit refusal: code 2 is the claim-runner's own "unreached" protocol
        // result and belongs to the unobserved family; any other code is a definite refusal.
        O::ExitFailure { code, .. } => {
            if *code == CLAIM_RUNNER_UNREACHED_EXIT_CODE {
                ClaimRouteVerdict::ClaimUnobserved
            } else {
                ClaimRouteVerdict::ClaimDidNotHold
            }
        }
        O::Fail | O::NotBool { .. } => ClaimRouteVerdict::ClaimDidNotHold,
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

/// The ONE FOLD: printed lines and the exit status are BOTH derived here, from the single
/// collection of per-claim outcomes the route established once. Nothing else computes the route's
/// verdict.
///
/// The fold keys on the claim IDENTITY (module path and function), so the contract "exactly one
/// outcome per identity" is enforced at the only place that turns outcomes into a result: a second
/// outcome for identity A is dropped and the run cannot certify SUCCESS -- though a definite failure
/// still keeps precedence over the duplicate in the exit status -- so a duplicated outcome for A can
/// never stand in for a missing B. Every kept identity's verdict is folded by the EXISTING rule
/// `claim_route_termination` -- the policy is not redesigned here.
struct ClaimRouteFold {
    lines: Vec<String>,
    verdicts: Vec<ClaimRouteVerdict>,
    duplicate: Option<String>,
}

impl ClaimRouteFold {
    fn count(&self, verdict: ClaimRouteVerdict) -> usize {
        self.verdicts.iter().filter(|x| **x == verdict).count()
    }

    /// THE ONE EXIT STATUS. The existing `claim_route_termination` rule decides: a definite failure
    /// dominates (exit 1), otherwise an unobserved member or an empty selection is the subject
    /// unreached (exit 2), otherwise success (exit 0). A duplicated identity is a broken
    /// one-outcome-per-claim contract, so it blocks SUCCESS only: it upgrades a would-be success to
    /// the subject unreached, but it never overrides a definite failure (which still exits 1) and
    /// never softens an unobserved/empty result (which already exits 2).
    fn termination(&self) -> Termination {
        let base = claim_route_termination(&self.verdicts);
        if self.duplicate.is_some() && base == Termination::ObservationHeld {
            Termination::SubjectUnreached
        } else {
            base
        }
    }
}

fn claim_route_fold(members: &[cli_run::ClaimRouteMember]) -> ClaimRouteFold {
    let mut lines = Vec::with_capacity(members.len());
    let mut verdicts = Vec::with_capacity(members.len());
    let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();
    let mut duplicate = None;
    for m in members {
        let label = render_label(&Label {
            package_segments: m.module_path.split('.').map(str::to_string).collect(),
            target: m.function.clone(),
        });
        if !seen.insert((m.module_path.clone(), m.function.clone())) {
            // This identity already carries an outcome. Keep the first printed line; refuse to
            // let the duplicate fold, so it cannot satisfy the contract in another claim's place.
            if duplicate.is_none() {
                duplicate = Some(label);
            }
            continue;
        }
        let verdict = claim_route_verdict(&m.outcome);
        let word = match verdict {
            ClaimRouteVerdict::ClaimHeld => "HELD",
            ClaimRouteVerdict::ClaimDidNotHold => "DID-NOT-HOLD",
            ClaimRouteVerdict::ClaimUnobserved => "UNOBSERVED",
        };
        // THE ONE LINE, per the ruled schema:
        //   <label> <VERDICT> outcome=<kind> [cause=<token>] [code=<n>] [at=<file:line:col>] [detail="<quoted>"] eval_steps=<n> cpu_ms=<n>
        // `outcome` is the CLOSED kind name (never Rust Debug, which is not a schema); `cause` and
        // `code` are the only machine keys beyond the verdict; every free-text fact goes to `detail`,
        // the LAST field, double-quoted and single-line, and parsers never key on it. `at=` is emitted
        // only where a file:line:col is known -- on this revision no claim outcome carries one (the
        // raise path records a byte offset, not a line/col; a panic carries no location), so it stays
        // absent rather than fabricated. Populating it once a source location is carried on the
        // outcome is the deferred trigger, not this item.
        use cli_run::{ClaimMemberOutcome as M, ClaimOutcome as O, EntryResolveRefusal as R};
        let (kind, cause, code, detail): (&str, Option<String>, Option<i32>, Option<String>) =
            match &m.outcome {
                M::Evaluated(o) => {
                    let cause = match o {
                        O::RuntimeError { cause, .. } => Some(cause.token().to_string()),
                        // The cause token is the TYPED payload kind decided at the catch site, so a
                        // non-string panic renders `non_string_payload`, never `string_payload`.
                        O::Panicked { payload_kind, .. } => Some(payload_kind.token().to_string()),
                        _ => None,
                    };
                    let code = match o {
                        O::ExitFailure { code, .. } => Some(*code),
                        _ => None,
                    };
                    let detail = match o {
                        O::RuntimeError { message, .. } => Some(message.clone()),
                        O::Panicked { payload, .. } => Some(payload.clone()),
                        O::ExitFailure { reason, .. } => reason.clone(),
                        O::NotBool { got } => Some(got.clone()),
                        O::NotAttempted { halted_by } => Some(halted_by.clone()),
                        O::HostToolUnresolved { name, .. } => Some(name.clone()),
                        O::HostEffectRefused { operation, .. } => Some(operation.clone()),
                        _ => None,
                    };
                    (cli_run::claim_outcome_kind(o), cause, code, detail)
                }
                M::Unresolved(r) => {
                    let kind = match r {
                        R::EntryMissing { .. } => "EntryMissing",
                        R::ClosureDiagnostics { .. } => "ClosureDiagnostics",
                        R::ResolveInfrastructure { .. } => "ResolveInfrastructure",
                    };
                    (kind, None, None, Some(r.message().to_string()))
                }
            };
        let mut line = format!("{label} {word} outcome={kind}");
        if let Some(cause) = &cause {
            line.push_str(&format!(" cause={cause}"));
        }
        if let Some(code) = code {
            line.push_str(&format!(" code={code}"));
        }
        if let Some(detail) = &detail {
            line.push_str(&format!(" detail={}", quote_detail(detail)));
        }
        line.push_str(&format!(" eval_steps={} cpu_ms={}", m.eval_steps, m.cpu_ms));
        lines.push(line);
        verdicts.push(verdict);
    }
    ClaimRouteFold {
        lines,
        verdicts,
        duplicate,
    }
}

/// The `detail` field: one double-quoted, escaped, single-line string, so the human text (panic
/// payload verbatim, error message, exit reason, refusal message) is on the line for a reader but is
/// always the LAST free-text field a parser ignores. Backslashes and quotes are escaped and any line
/// break is folded to a space so the record stays one line.
fn quote_detail(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' | '\r' | '\t' => out.push(' '),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// THE CLAIM ROUTE: select the discovered claims the batch's patterns contain, evaluate each, print one
/// line per claim, fold the verdicts. THE BATCH IS ONE PROCESS AND ONE SELECTION: the selected modules
/// are read once (`cli_run::run_claim_route`'s selection), the claims are enumerated from them, and
/// each is matched against EVERY pattern -- so selection is driven by the discovered corpus rather
/// than by the operand list, and a claim two operands both denote is enrolled once, not twice. No
/// operand widens another: a pattern selects only what it contains, and no selected module changes
/// what another module's entry resolves against.
///
/// The subject is narrowed to the modules the patterns can reach BEFORE discovery, so a batch of exact
/// labels reads those modules rather than the corpus; the corpus roots arrive from the caller's
/// `admitted_source_roots` (the witness-layer roots when no option was given) and are the floor's own
/// authority for them.
fn claim_selection_scopes(patterns: &[TargetPattern]) -> Vec<cli_run::ClaimSelectionScope> {
    patterns
        .iter()
        .map(|pattern| match pattern {
            TargetPattern::SingleTarget(label) => cli_run::ClaimSelectionScope {
                module_segments: label.package_segments.clone(),
                beneath: false,
            },
            TargetPattern::PackageTargets(package) => cli_run::ClaimSelectionScope {
                module_segments: package.clone(),
                beneath: false,
            },
            TargetPattern::SubtreeTargets(package) => cli_run::ClaimSelectionScope {
                module_segments: package.clone(),
                beneath: true,
            },
        })
        .collect()
}

fn run_claim_selection(
    patterns: &[TargetPattern],
    roots: &[String],
    execution_mode: crate::v1_interpreter::ExecutionMode,
    fixture_store: Option<std::rc::Rc<crate::recorded_fixture::RecordedFixtureStore>>,
) -> InvocationOutcome {
    let rendered = patterns
        .iter()
        .map(render_target_pattern)
        .collect::<Vec<String>>()
        .join(", ");
    let module_selected = |module_path: &str| {
        let segments: Vec<String> = module_path.split('.').map(str::to_string).collect();
        patterns.iter().any(|pattern| match pattern {
            TargetPattern::SingleTarget(label) => label.package_segments == segments,
            TargetPattern::PackageTargets(_) | TargetPattern::SubtreeTargets(_) => {
                target_pattern_within(&TargetPattern::PackageTargets(segments.clone()), pattern)
            }
        })
    };
    let claim_selected = |module_path: &str, function: &str| {
        let label = Label {
            package_segments: module_path.split('.').map(str::to_string).collect(),
            target: function.to_string(),
        };
        patterns.iter().any(|pattern| {
            target_pattern_within(&TargetPattern::SingleTarget(label.clone()), pattern)
        })
    };
    let scopes = claim_selection_scopes(patterns);
    let members = match cli_run::run_claim_route(
        roots,
        &scopes,
        &module_selected,
        &claim_selected,
        execution_mode,
        fixture_store,
    ) {
        Ok(members) => members,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("claim route: {rendered} — {cause}"),
            }
        }
    };
    // Both the printed lines and the exit status come from the ONE fold over this collection.
    let fold = claim_route_fold(&members);
    for line in &fold.lines {
        println!("{line}");
    }
    // The expectation frontier, AFTER the located lines and ON STDERR. The two channels are the
    // boundary: stdout is the machine-readable line schema a decoder keys on, and this receipt is a
    // diagnostic about the subject, not an outcome — putting it on stdout would make a parser decide
    // whether an unrelated line is a claim. It is the same call the legacy batch made, through the one
    // renderer in `v1_interpreter`, so the consumer at
    // `dag/test/claim/expectation_frontier_witness.dag` reads the identical line from a `gunbc test`
    // child and the claim route is not a route that loses an observation on the way to becoming the
    // only one. Empty tally prints nothing: the negative half of that pair is the point.
    if let Some(line) = crate::v1_interpreter::expectation_frontier_receipt_line() {
        eprintln!("{line}");
    }
    let held = fold.count(ClaimRouteVerdict::ClaimHeld);
    let did_not_hold = fold.count(ClaimRouteVerdict::ClaimDidNotHold);
    let unobserved = fold.count(ClaimRouteVerdict::ClaimUnobserved);
    let selected = fold.verdicts.len();
    if let Some(label) = fold.duplicate.as_deref() {
        return InvocationOutcome {
            termination: fold.termination(),
            message: format!(
                "claim route: {rendered} — {label} carries more than one outcome; the route owes \
                 exactly one outcome per claim identity ({selected} claim(s): {held} held, \
                 {did_not_hold} did not hold, {unobserved} unobserved)",
            ),
        };
    }
    let note = if fold.verdicts.is_empty() {
        claim_route_empty_selection_note(patterns)
    } else {
        String::new()
    };
    InvocationOutcome {
        termination: fold.termination(),
        message: format!(
            "claim route: {rendered} — {selected} claim(s): {held} held, {did_not_hold} did not \
             hold, {unobserved} unobserved{note}",
        ),
    }
}

/// `gunbc.target_invocation` `BinaryInput`, mirrored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryInput {
    NotNewer(String),
    Newer(String),
    Missing(String),
    DiffersFromThisCheckout(String),
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

/// `gunbc.target_invocation` `binary_freshness_step`, mirrored arm for arm.
fn binary_freshness_step(acc: BinaryFreshness, input: &BinaryInput) -> BinaryFreshness {
    match acc {
        BinaryFreshness::Undecided { .. } => acc,
        BinaryFreshness::Stale { .. } => acc,
        BinaryFreshness::Fresh { binary, inputs } => match input {
            BinaryInput::DiffersFromThisCheckout(p) => BinaryFreshness::Stale {
                binary,
                input: p.clone(),
                why: "differs from the file the binary was built from in another checkout"
                    .to_string(),
            },
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
/// lists; the checkout is that manifest's `src/v1/stage0` ancestor. When that root is not ours,
/// each input under it is also compared byte for byte with ours at the same relative path. Registry
/// and sysroot inputs are outside every checkout and are judged by mtime like any other input.
///
/// Git paths are skipped, with a reason: build.rs watches `HEAD`, `index`, `packed-refs` and the
/// branch ref only to re-stamp `GUNBC_BUILD_IDENTITY`, the version string. `packed-refs` moves on
/// any fetch in any worktree, so judging them would refuse a binary whose code did not change --
/// the spurious refusal this check exists not to make. The cost is that a fresh binary may print
/// an older commit as its identity; its bytes are still those that commit built.
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
    let built_elsewhere = std::path::Path::new(&build_root) != workspace;
    let root_prefix = format!("{build_root}/");
    let mut out = Vec::new();
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
        let classified = match (&classified, input.strip_prefix(&root_prefix)) {
            (BinaryInput::NotNewer(_), Some(relative)) if built_elsewhere => {
                input_against_this_checkout(input, &workspace.join(relative))?.unwrap_or(classified)
            }
            _ => classified,
        };
        out.push(classified);
    }
    Ok(out)
}

fn input_against_this_checkout(
    built_from: &str,
    here: &std::path::Path,
) -> Result<Option<BinaryInput>, String> {
    let built = input_content(std::path::Path::new(built_from))?
        .ok_or_else(|| format!("dep-info input {built_from} vanished while it was read"))?;
    let differing = match (&built, input_content(here)?) {
        (built, Some(content)) if content == *built => return Ok(None),
        (InputContent::Tree(theirs), Some(InputContent::Tree(ours))) => {
            first_differing_path(theirs, &ours)
                .map(|relative| here.join(relative))
                .unwrap_or_else(|| here.to_path_buf())
        }
        _ => here.to_path_buf(),
    };
    Ok(Some(BinaryInput::DiffersFromThisCheckout(
        differing.display().to_string(),
    )))
}

fn first_differing_path(
    theirs: &[(std::path::PathBuf, Vec<u8>)],
    ours: &[(std::path::PathBuf, Vec<u8>)],
) -> Option<std::path::PathBuf> {
    use std::cmp::Ordering;
    let (mut i, mut j) = (0, 0);
    loop {
        match (theirs.get(i), ours.get(j)) {
            (None, None) => return None,
            (Some((path, _)), None) | (None, Some((path, _))) => return Some(path.clone()),
            (Some((a, a_bytes)), Some((b, b_bytes))) => match a.cmp(b) {
                Ordering::Less => return Some(a.clone()),
                Ordering::Greater => return Some(b.clone()),
                Ordering::Equal if a_bytes != b_bytes => return Some(a.clone()),
                Ordering::Equal => {
                    i += 1;
                    j += 1;
                }
            },
        }
    }
}

#[derive(PartialEq, Eq)]
enum InputContent {
    File(Vec<u8>),
    Tree(Vec<(std::path::PathBuf, Vec<u8>)>),
}

fn input_content(path: &std::path::Path) -> Result<Option<InputContent>, String> {
    let unreadable = |e: std::io::Error| format!("cannot read {}: {e}", path.display());
    match std::fs::metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(unreadable(e)),
        Ok(meta) if meta.is_dir() => {
            let mut files = Vec::new();
            tree_files(path, std::path::Path::new(""), &mut files)?;
            files.sort();
            let mut tree = Vec::with_capacity(files.len());
            for relative in files {
                let full = path.join(&relative);
                let bytes = std::fs::read(&full)
                    .map_err(|e| format!("cannot read {}: {e}", full.display()))?;
                tree.push((relative, bytes));
            }
            Ok(Some(InputContent::Tree(tree)))
        }
        Ok(_) => std::fs::read(path)
            .map(|bytes| Some(InputContent::File(bytes)))
            .map_err(unreadable),
    }
}

fn tree_files(
    root: &std::path::Path,
    relative: &std::path::Path,
    out: &mut Vec<std::path::PathBuf>,
) -> Result<(), String> {
    let dir = root.join(relative);
    let entries =
        std::fs::read_dir(&dir).map_err(|e| format!("cannot list {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("cannot list {}: {e}", dir.display()))?;
        let child = relative.join(entry.file_name());
        let kind = entry
            .file_type()
            .map_err(|e| format!("cannot stat {}: {e}", root.join(&child).display()))?;
        if kind.is_dir() {
            tree_files(root, &child, out)?;
        } else {
            out.push(child);
        }
    }
    Ok(())
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
/// WHAT A `gunbc test` INVOCATION ASKED FOR, as the CLI collected it. One struct rather than six
/// parameters, because the admission below reads all of it together: the batch route decides whether the
/// mode and the corpus may be spelled at all, so the operands and the options are one decision, not two.
///
/// `gunbc.target_invocation` mirrors: the operand list is admitted by `admit_test_operands`, the options
/// by `admit_test_execution_mode` and `admit_test_source_roots`, each keyed on `test_selection_batch_route`.
pub struct TestInvocationOptions {
    /// Absolute labels and target patterns, as typed. At least one; an empty batch refuses.
    pub operands: Vec<String>,
    pub hermetic: bool,
    pub wet: bool,
    pub record: bool,
    pub fixture_store: Option<String>,
    /// Corpus the batch's labels are looked up in, replacing the default witness-layer roots.
    pub source_roots: Vec<String>,
}

impl TestInvocationOptions {
    /// The options a hand-off or a witness uses: one operand, default corpus, default mode.
    pub fn single_operand(operand: &str) -> Self {
        Self {
            operands: vec![operand.to_string()],
            hermetic: false,
            wet: false,
            record: false,
            fixture_store: None,
            source_roots: Vec::new(),
        }
    }

    /// The first mode flag spelled, in the order `--hermetic --wet --record`: `None` means the mode is
    /// unspelled, which is a GRANT of Hermetic rather than a failure.
    fn spelled_mode_flag(&self) -> Option<&'static str> {
        [
            (self.hermetic, "--hermetic"),
            (self.wet, "--wet"),
            (self.record, "--record"),
        ]
        .into_iter()
        .find(|(set, _)| *set)
        .map(|(_, name)| name)
    }

    /// THE OPTION SURFACE, ADMITTED ONCE FOR THE WHOLE BATCH: `gunbc.target_invocation`
    /// `admit_test_execution_mode`, mirrored. Three refusals, each naming what the caller typed:
    /// two mode flags (Record is a mode of its own, so `--record --wet` is a conflict rather than a
    /// combination -- naming BOTH is what lets a reader correct it); `--record` with nowhere to write;
    /// and a fixture store with no mode to belong to. With nothing spelled this returns `Ok(None)` and
    /// the caller's `execution_mode_for` supplies Hermetic, which is what every existing
    /// `gunbc test <label>` depends on -- there is deliberately no refusal arm for an unspelled mode.
    fn admitted_execution_mode(
        &self,
    ) -> Result<Option<crate::v1_interpreter::ExecutionMode>, String> {
        let spelled: Vec<&'static str> = [
            (self.hermetic, "--hermetic"),
            (self.wet, "--wet"),
            (self.record, "--record"),
        ]
        .into_iter()
        .filter(|(set, _)| *set)
        .map(|(_, name)| name)
        .collect();
        if spelled.len() > 1 {
            return Err(format!(
                "two mode flags are not one batch's mode: {} and {}. Record is a mode of its own (effects run and fixtures are written), so --record --wet is not a combination",
                spelled[0], spelled[1]
            ));
        }
        let store = self.fixture_store.as_deref().unwrap_or("");
        if self.record && store.trim().is_empty() {
            return Err(
                "--record writes fixtures and was given nowhere to write them: pass --fixture-store <dir>. The fixture store is the mode's own argument, never a source root".to_string(),
            );
        }
        if !store.trim().is_empty() && !self.record && !self.hermetic {
            return Err(
                "--fixture-store names the mode it belongs to and no mode was spelled: use --record (effects run, fixtures written) or --hermetic (replay from the store, no effects)"
                    .to_string(),
            );
        }
        Ok(if self.wet {
            Some(crate::v1_interpreter::ExecutionMode::Wet)
        } else if self.record {
            Some(crate::v1_interpreter::ExecutionMode::Record)
        } else if self.hermetic {
            Some(crate::v1_interpreter::ExecutionMode::Hermetic)
        } else {
            None
        })
    }

    fn fixture_store_path(&self) -> Option<std::path::PathBuf> {
        self.fixture_store
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(std::path::PathBuf::from)
    }

    /// `gunbc.target_invocation` `admit_test_source_roots`, mirrored. THE REPLACE-NOT-APPEND RULE IS IN
    /// THE SHAPE: this returns the corpus the route will read, and the route reads only what it returns,
    /// so `default_roots` is a parameter here rather than something the route adds later -- an option
    /// given leaves the defaults nowhere to reappear. The control that makes that readable gives one
    /// root and asserts no default is among the granted roots.
    fn admitted_source_roots(&self, default_roots: &[String]) -> Result<Vec<String>, String> {
        if self.source_roots.is_empty() {
            return Ok(default_roots.to_vec());
        }
        if self.source_roots.iter().any(|r| r.trim().is_empty()) {
            return Err(
                "--source-root was given without a value; a root is a directory the module index walks"
                    .to_string(),
            );
        }
        // `gunbc.target_invocation` `test_source_root_not_walkable`: a root the walk cannot read refuses
        // here rather than reaching the route, because an unreadable root selects no claims and the batch
        // would report "selected no discovered claim" -- a true sentence about the wrong subject.
        for root in &self.source_roots {
            if let Some(reason) = test_source_root_not_walkable(root) {
                return Err(format!("--source-root {root} cannot be read: {reason}"));
            }
        }
        Ok(self.source_roots.clone())
    }

    /// Whether ANY option that belongs to the claim route was spelled. Checked on a native, registry or
    /// mixed batch, where there is no claim evaluation for the option to reach.
    fn claim_route_option_spelled(&self) -> Option<&'static str> {
        match self.spelled_mode_flag() {
            Some(flag) => Some(flag),
            None if self.fixture_store.is_some() => Some("--fixture-store"),
            None if !self.source_roots.is_empty() => Some("--source-root"),
            None => None,
        }
    }
}

/// WHY ONE INVOCATION RUNS ONE ROUTE. The counts are in the message because the remedy is to pass
/// separate invocations, and a reader needs to know what was mixed with what.
fn mixed_batch_refusal(native: usize, claim: usize, exact: usize) -> InvocationOutcome {
    InvocationOutcome {
        termination: Termination::Refused,
        message: format!(
            "gunbc test: one invocation runs one route's batch; this mixed {native} native, {claim} claim and {exact} registry operand(s). Pass them as separate invocations -- an invocation's result is derived by one rule, and a mixed batch would have two",
        ),
    }
}

/// THE ONE REFUSAL SHAPE FOR AN OPTION THE BATCH'S ROUTE CANNOT RECEIVE. Names the option, the route
/// it was spelled on, and the reason -- the reason differs per option, so it is passed in rather than
/// written twice.
fn option_on_other_route_refusal(option: &str, route: &str, why: &str) -> InvocationOutcome {
    InvocationOutcome {
        termination: Termination::Refused,
        message: format!("gunbc test: {option} is a claim-route option; {route}. {why}"),
    }
}

/// `gunbc.target_invocation` `test_source_root_not_walkable`, mirrored: `Some(reason)` when the walk could
/// not read this root. Absence is reported distinctly from emptiness, because the two have different
/// remedies -- a missing root is a typo or a deleted directory, an empty one is a corpus with no claims in
/// it, which is a legitimate (if suspicious) request.
fn test_source_root_not_walkable(root: &str) -> Option<String> {
    let path = std::path::Path::new(root);
    if !path.exists() {
        return Some("no such path".to_string());
    }
    if !path.is_dir() {
        return Some("it is not a directory".to_string());
    }
    match std::fs::read_dir(path) {
        Ok(_) => None,
        Err(error) => Some(error.to_string()),
    }
}

/// WHY THE NATIVE AND REGISTRY ROUTES STILL ANSWER ONE OPERAND. Both enumerate a population the caller
/// cannot see (the native compiler's build graph; one registry binding), so a second operand is not a
/// second claim added to a batch -- it is a second run whose result the route's fold has no rule for.
/// The plan's batch is the CLAIM batch; widening these two is a change to those routes, not to this one.
fn single_operand_route_refusal(route: &str, count: usize) -> InvocationOutcome {
    InvocationOutcome {
        termination: Termination::Refused,
        message: format!(
            "gunbc test: the {route} route runs one operand per invocation and this named {count}; the batch form is the claim route's (claim-entrypoints-cutover, gunb-ai/gunbc#16872)",
        ),
    }
}

pub fn test_verb_checked(options: &TestInvocationOptions) -> InvocationOutcome {
    cli_run::user_work_scope::reexec_into_user_work_slice_if_owed();
    // THE SECOND ACT, BEFORE ANY PRODUCER: bind this process to a kernel memory limit when the
    // invocation asked for one (`GUNBC_BIND_MEMORY_CGROUP_BYTES`, gunbc.host.memory_cgroup_binding --
    // the same bind claim_executor takes; claim_batch took it too until the cutover deleted it) at gunbc.emitted_subject_build_gate
    // native_step_memory_trip, the slot's MemoryHigh less a declared headroom. The required
    // `witnesses` job passes that input only where a job owns a bounded cgroup
    // (gunbc.runner.runner_slot_desired gunbc_runner_slot_job_cgroup_ownership decides; today a fleet job
    // is the deprivileged job user under a root unit and the governor's leaf lives at the cgroup
    // root, and the climb is gunbc.guarantee_stall native_step_trip_awaits_fleet_job_cgroup_stall).
    // A refused bind stops the line: the run asked to be bounded and is not.
    let bind = crate::memory_governor::apply_memory_cgroup_bind();
    eprintln!("{}", crate::memory_governor::cgroup_bind_note(&bind));
    if let Some(diagnostic) = crate::memory_governor::cgroup_bind_refusal_diagnostic(&bind) {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: diagnostic,
        };
    }
    // THE THIRD ACT: what bounds this run, decided and READ before the producer. The required job
    // passes `GUNBC_MEMORY_ENVELOPE_REQUIRED=slot` on every subject step
    // (gunbc.emitted_subject_build_gate native_step_memory_envelope_env_name): a cgroup memory.max
    // must already bind the step -- the fleet slot's -- and a run none binds is refused, because a
    // required step that cannot say what bounded it is not the measurement the gate names
    // (operator ruling 2026-10-09: a fixed hard maximum, swap refused, a peak-and-swap receipt,
    // no larger-runner retry).
    let envelope = match memory_envelope_requirement(&bind) {
        Ok(envelope) => envelope,
        Err(refusal) => {
            eprintln!("{refusal}");
            return InvocationOutcome {
                termination: Termination::Refused,
                message: refusal,
            };
        }
    };
    let outcome = test_verb_after(options, &assess_binary_freshness(&observe_binary_inputs()));
    // THE PEAK-AND-SWAP RECEIPT, after the producer and whatever it spawned have run: read from
    // the cgroup that bounds the step (every child process was charged to it), against the
    // reading taken before the producer, so a cgroup that outlives the step -- the slot's --
    // reports what THIS step added. A violated envelope -- an OOM kill, swap charged, or a swap
    // counter that cannot be read -- is a refusal even over a producer that held, because "no
    // thrashing, ever" is part of the verdict.
    match memory_envelope_receipt(&envelope) {
        None => outcome,
        Some(Ok(receipt)) => {
            eprintln!("{receipt}");
            outcome
        }
        Some(Err(violation)) => {
            eprintln!("{violation}");
            InvocationOutcome {
                termination: Termination::Refused,
                message: format!("{}\n{violation}", outcome.message),
            }
        }
    }
}

/// Authority: `gunbc.emitted_subject_build_gate` `native_step_memory_envelope_env_name` and
/// `native_step_memory_envelope_slot_value` (the latter the projection of `gunbc.host.memory_envelope`
/// `EnvelopeSlotRequired`), pinned by `test.claim.compiler_gate_emit_build_lane_witness`
/// `the_seed_envelope_requirement_inputs_are_the_dag_rows`.
const MEMORY_ENVELOPE_REQUIRED_ENV: &str = "GUNBC_MEMORY_ENVELOPE_REQUIRED";
const MEMORY_ENVELOPE_SLOT_VALUE: &str = "slot";

/// Authority: `gunbc.host.memory_envelope` `MemoryEnvelopeRequirement`. Absent and unreadable are
/// different answers, exactly as for the bind request.
#[derive(Debug, Clone, PartialEq, Eq)]
enum MemoryEnvelopeRequirement {
    NotRequested,
    SlotRequired,
    Unreadable { raw: String },
}

/// Authority: `gunbc.host.memory_envelope` `MemoryEnvelopeGrain` / `memory_envelope_grain_label`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MemoryEnvelopeGrain {
    Leaf,
    Existing,
    Slot,
}

impl MemoryEnvelopeGrain {
    fn label(self) -> &'static str {
        match self {
            MemoryEnvelopeGrain::Leaf => "leaf",
            MemoryEnvelopeGrain::Existing => "existing",
            MemoryEnvelopeGrain::Slot => "slot",
        }
    }
}

/// Authority: `gunbc.host.memory_envelope` `MemoryEnvelopeRefusalCause`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum MemoryEnvelopeRefusalCause {
    GrainUnknown { raw: String },
    Absent { grain: MemoryEnvelopeGrain },
}

/// Authority: `gunbc.host.memory_envelope` `MemoryEnvelopeDecision`.
#[derive(Debug, Clone, PartialEq, Eq)]
enum MemoryEnvelopeDecision {
    NotRequired,
    Bounded { grain: MemoryEnvelopeGrain },
    Refused { cause: MemoryEnvelopeRefusalCause },
}

/// Mirror of `gunbc.host.memory_envelope` `memory_envelope_requirement_from_text`.
fn memory_envelope_requirement_from_text(raw: Option<&str>) -> MemoryEnvelopeRequirement {
    match raw {
        None => MemoryEnvelopeRequirement::NotRequested,
        Some(v) if v == MEMORY_ENVELOPE_SLOT_VALUE => MemoryEnvelopeRequirement::SlotRequired,
        Some(v) => MemoryEnvelopeRequirement::Unreadable { raw: v.to_string() },
    }
}

/// THE PURE DECISION, mirroring `gunbc.host.memory_envelope` `memory_envelope_decision` arm for arm:
/// every input is a parameter so the discriminating cases -- above all a slot requirement over a
/// process no numeric memory.max binds, which must refuse -- are reachable from a test on any
/// machine. `numeric_max_binds` is the governor's `binding_cap_cgroup_dir` walk, read once by the
/// shell below; a body of `max` is no limit.
fn memory_envelope_decision(
    bind: &crate::memory_governor::CgroupBindDecision,
    requirement: &MemoryEnvelopeRequirement,
    numeric_max_binds: bool,
) -> MemoryEnvelopeDecision {
    use crate::memory_governor::CgroupBindDecision as Bind;
    let bounded_or_absent = |grain: MemoryEnvelopeGrain| {
        if numeric_max_binds {
            MemoryEnvelopeDecision::Bounded { grain }
        } else {
            MemoryEnvelopeDecision::Refused {
                cause: MemoryEnvelopeRefusalCause::Absent { grain },
            }
        }
    };
    match requirement {
        MemoryEnvelopeRequirement::Unreadable { raw } => MemoryEnvelopeDecision::Refused {
            cause: MemoryEnvelopeRefusalCause::GrainUnknown { raw: raw.clone() },
        },
        MemoryEnvelopeRequirement::NotRequested => match bind {
            Bind::Applicable { .. } => bounded_or_absent(MemoryEnvelopeGrain::Leaf),
            Bind::UnnecessaryLimitAlreadyBinds { .. } => {
                bounded_or_absent(MemoryEnvelopeGrain::Existing)
            }
            Bind::NotRequested => MemoryEnvelopeDecision::NotRequired,
            Bind::Refused { .. } => MemoryEnvelopeDecision::Refused {
                cause: MemoryEnvelopeRefusalCause::Absent {
                    grain: MemoryEnvelopeGrain::Leaf,
                },
            },
        },
        MemoryEnvelopeRequirement::SlotRequired => match bind {
            Bind::Applicable { .. } => bounded_or_absent(MemoryEnvelopeGrain::Leaf),
            Bind::UnnecessaryLimitAlreadyBinds { .. } => {
                bounded_or_absent(MemoryEnvelopeGrain::Existing)
            }
            Bind::NotRequested => bounded_or_absent(MemoryEnvelopeGrain::Slot),
            Bind::Refused { .. } => MemoryEnvelopeDecision::Refused {
                cause: MemoryEnvelopeRefusalCause::Absent {
                    grain: MemoryEnvelopeGrain::Leaf,
                },
            },
        },
    }
}

/// THE PURE VERDICT, mirroring `gunbc.host.memory_envelope` `memory_envelope_verdict`: violated on any
/// OOM kill the step added, any swap resident at exit, an unreadable swap gauge, a swap peak that
/// rose during the step, or a swap peak that could not be observed (`None`: no memory.swap.peak on
/// this kernel) under a cgroup whose memory.swap.max is not zero -- unobservable is not "did not
/// rise", and only a zero swap limit makes the missing counter moot.
fn memory_envelope_violated(
    oom_kills_delta: u64,
    swap_current: Option<u64>,
    swap_peak_rose: Option<bool>,
    swap_max_zero: bool,
) -> bool {
    let swap_rose_or_unobservable = match swap_peak_rose {
        Some(rose) => rose,
        None => !swap_max_zero,
    };
    oom_kills_delta > 0 || swap_current.map(|v| v > 0).unwrap_or(true) || swap_rose_or_unobservable
}

/// What bounds this run's memory, decided before any producer, with the readings taken at the
/// start so the receipt can report what this step added to a cgroup that outlives it.
enum MemoryEnvelope {
    NotRequired,
    Bounded {
        grain: MemoryEnvelopeGrain,
        dir: std::path::PathBuf,
        start: cli_run::cgroup_memory::CgroupMemoryRead,
        start_swap: cli_run::cgroup_memory::CgroupSwapRead,
    },
}

/// The effectful shell: read the requirement and the one host fact the decision needs, decide
/// through the mirror, and take the start readings of the cgroup the decision named.
fn memory_envelope_requirement(
    bind: &crate::memory_governor::CgroupBindDecision,
) -> Result<MemoryEnvelope, String> {
    use crate::memory_governor::CgroupBindDecision as Bind;
    let raw = std::env::var(MEMORY_ENVELOPE_REQUIRED_ENV).ok();
    let requirement = memory_envelope_requirement_from_text(raw.as_deref());
    // THE ONE HOST FACT: which cgroup carries the numeric memory.max that binds this process. The
    // governor's walk is the authority (gunbc.design_argument::single_authority) and reads `max` as unset; the peak locator
    // the first version used answers where counters may be read, never whether anything bounds them.
    let binding_dir = crate::memory_governor::binding_cap_cgroup_dir();
    let required_by = match (&requirement, bind) {
        (MemoryEnvelopeRequirement::SlotRequired, Bind::NotRequested) => {
            format!("{MEMORY_ENVELOPE_REQUIRED_ENV}={MEMORY_ENVELOPE_SLOT_VALUE}")
        }
        (MemoryEnvelopeRequirement::SlotRequired, _) => format!(
            "{MEMORY_ENVELOPE_REQUIRED_ENV}={MEMORY_ENVELOPE_SLOT_VALUE} and the bind {}",
            crate::memory_governor::BIND_MEMORY_CGROUP_ENV
        ),
        (_, _) => format!(
            "the bind {} ({MEMORY_ENVELOPE_REQUIRED_ENV} unset)",
            crate::memory_governor::BIND_MEMORY_CGROUP_ENV
        ),
    };
    match memory_envelope_decision(bind, &requirement, binding_dir.is_some()) {
        MemoryEnvelopeDecision::NotRequired => Ok(MemoryEnvelope::NotRequired),
        MemoryEnvelopeDecision::Refused {
            cause: MemoryEnvelopeRefusalCause::GrainUnknown { raw },
        } => Err(format!(
            "gunbc test: MEMORY ENVELOPE REFUSED cause=MemoryEnvelopeGrainUnknown — \
             {MEMORY_ENVELOPE_REQUIRED_ENV}={raw:?} names no envelope grain this binary knows \
             (it knows {MEMORY_ENVELOPE_SLOT_VALUE:?}); a run that asked to be bounded with a \
             value the binary cannot read proceeds nowhere"
        )),
        MemoryEnvelopeDecision::Refused {
            cause: MemoryEnvelopeRefusalCause::Absent { grain },
        } => Err(format!(
            "gunbc test: MEMORY ENVELOPE REFUSED cause=MemoryEnvelopeAbsent — {required_by} \
             requires a cgroup memory.max to bind this process (grain {}), and no cgroup on its \
             path to the root carries a numeric memory.max: a body of `max` is no limit \
             (extdeps.linux.cgroup_v2_memory CgroupMemoryUnlimited), and an unconverged slot is \
             refused rather than receipted as a bound",
            grain.label()
        )),
        MemoryEnvelopeDecision::Bounded { grain } => {
            let Some(dir) = binding_dir else {
                return Err(
                    "gunbc test: MEMORY ENVELOPE REFUSED cause=MemoryEnvelopeAbsent — the decision \
                     bounded this run but the governor's walk found no binding cgroup; refusing \
                     rather than reading an unnamed one"
                        .to_string(),
                );
            };
            let start = cli_run::cgroup_memory::read_cgroup_memory(&dir).map_err(|refusal| {
                format!(
                    "gunbc test: MEMORY RECEIPT REFUSED cause=MemoryReceiptUnreadable — {}",
                    refusal.render()
                )
            })?;
            let start_swap = cli_run::cgroup_memory::read_cgroup_swap(&dir).map_err(|refusal| {
                format!(
                    "gunbc test: MEMORY RECEIPT REFUSED cause=MemoryReceiptUnreadable — {}",
                    refusal.render()
                )
            })?;
            Ok(MemoryEnvelope::Bounded {
                grain,
                dir,
                start,
                start_swap,
            })
        }
    }
}

/// `None` when nothing bounds the run by request (an operator-local run measures nothing);
/// otherwise the receipt line, or the violation that refuses the run. The counters are the
/// cgroup's own, read through the floor memory supervisor's readers so a missing key is never a
/// zero; the event counters are reported as the DIFFERENCE from the start reading, because a slot
/// cgroup outlives this step; and the peak is labelled by whether it ROSE during the step, because
/// memory.peak is a lifetime maximum and the bind leaf is reused across runs.
fn memory_envelope_receipt(envelope: &MemoryEnvelope) -> Option<Result<String, String>> {
    let MemoryEnvelope::Bounded {
        grain,
        dir,
        start,
        start_swap,
    } = envelope
    else {
        return None;
    };
    let unreadable = |refusal: cli_run::cgroup_memory::QualificationRefusal| {
        format!(
            "gunbc test: MEMORY RECEIPT REFUSED cause=MemoryReceiptUnreadable — {}",
            refusal.render()
        )
    };
    let end = match cli_run::cgroup_memory::read_cgroup_memory(dir) {
        Ok(read) => read,
        Err(refusal) => return Some(Err(unreadable(refusal))),
    };
    let end_swap = match cli_run::cgroup_memory::read_cgroup_swap(dir) {
        Ok(read) => read,
        Err(refusal) => return Some(Err(unreadable(refusal))),
    };
    let oom_kills_delta = end.oom_kills.saturating_sub(start.oom_kills);
    let max_events_delta = end.max_events.saturating_sub(start.max_events);
    let high_events_delta = end.high_events.saturating_sub(start.high_events);
    let (swap_peak_rose, swap_peak_text) = match (start_swap.swap_peak, end_swap.swap_peak) {
        (Some(before), Some(after)) => (
            Some(after > before),
            after.saturating_sub(before).to_string(),
        ),
        _ => (
            None,
            "unobservable (no memory.swap.peak on this kernel)".to_string(),
        ),
    };
    let swap_max_zero = end_swap.swap_max.trim() == "0";
    let peak_scope = if end.peak > start.peak {
        format!("rose from {} during this step", start.peak)
    } else {
        format!(
            "prior history of the cgroup: memory.peak stood at {} before the producer and did not rise",
            start.peak
        )
    };
    let line = format!(
        "gunbc test: MEMORY RECEIPT grain={} cgroup={} limit_max={} limit_high={} swap_max={} \
         peak_bytes={} (peak: {peak_scope}) high_events_delta={high_events_delta} \
         max_events_delta={max_events_delta} oom_kills_delta={oom_kills_delta} \
         swap_current_bytes={} swap_peak_delta={swap_peak_text}",
        grain.label(),
        end.dir,
        end.limit_max,
        end.limit_high,
        end_swap.swap_max,
        end.peak,
        end_swap.swap_current
    );
    if memory_envelope_violated(
        oom_kills_delta,
        Some(end_swap.swap_current),
        swap_peak_rose,
        swap_max_zero,
    ) {
        Some(Err(format!(
            "gunbc test: MEMORY ENVELOPE VIOLATED cause=MemoryEnvelopeViolated — an OOM kill this \
             step added, swap resident at exit, a swap peak that rose during the step, or a swap peak \
             this kernel cannot show under a cgroup that permits swap, inside the bounding cgroup; the \
             run is refused even if its producer held. {line}"
        )))
    } else {
        Some(Ok(line))
    }
}

#[cfg(test)]
mod memory_envelope_tests {
    use super::*;
    use crate::memory_governor::{
        CgroupBindDecision as Bind, CgroupBindRefusalCause, HostBudgetSource,
    };

    fn existing() -> Bind {
        Bind::UnnecessaryLimitAlreadyBinds {
            source: HostBudgetSource::CgroupMemoryMax {
                cgroup_dir: "/sys/fs/cgroup/system.slice/actions-runner@srv4-15.service"
                    .to_string(),
            },
        }
    }

    #[test]
    fn requirement_reads_absent_slot_and_anything_else_as_three_answers() {
        assert_eq!(
            memory_envelope_requirement_from_text(None),
            MemoryEnvelopeRequirement::NotRequested
        );
        assert_eq!(
            memory_envelope_requirement_from_text(Some("slot")),
            MemoryEnvelopeRequirement::SlotRequired
        );
        assert_eq!(
            memory_envelope_requirement_from_text(Some("leaf")),
            MemoryEnvelopeRequirement::Unreadable {
                raw: "leaf".to_string()
            }
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn RED_a_slot_requirement_with_no_numeric_max_refuses_as_absent() {
        let d = memory_envelope_decision(
            &Bind::NotRequested,
            &MemoryEnvelopeRequirement::SlotRequired,
            false,
        );
        assert_eq!(
            d,
            MemoryEnvelopeDecision::Refused {
                cause: MemoryEnvelopeRefusalCause::Absent {
                    grain: MemoryEnvelopeGrain::Slot
                }
            }
        );
    }

    #[test]
    fn a_slot_requirement_under_a_numeric_max_is_bounded_at_the_slot() {
        assert_eq!(
            memory_envelope_decision(
                &Bind::NotRequested,
                &MemoryEnvelopeRequirement::SlotRequired,
                true
            ),
            MemoryEnvelopeDecision::Bounded {
                grain: MemoryEnvelopeGrain::Slot
            }
        );
    }

    #[test]
    fn no_requirement_and_no_bind_is_not_required_whatever_the_host() {
        for binds in [true, false] {
            assert_eq!(
                memory_envelope_decision(
                    &Bind::NotRequested,
                    &MemoryEnvelopeRequirement::NotRequested,
                    binds
                ),
                MemoryEnvelopeDecision::NotRequired
            );
        }
    }

    #[test]
    fn binds_are_bounded_at_their_own_grains() {
        assert_eq!(
            memory_envelope_decision(
                &Bind::Applicable { bound: 1 },
                &MemoryEnvelopeRequirement::NotRequested,
                true
            ),
            MemoryEnvelopeDecision::Bounded {
                grain: MemoryEnvelopeGrain::Leaf
            }
        );
        assert_eq!(
            memory_envelope_decision(&existing(), &MemoryEnvelopeRequirement::SlotRequired, true),
            MemoryEnvelopeDecision::Bounded {
                grain: MemoryEnvelopeGrain::Existing
            }
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn RED_a_refused_bind_and_an_unknown_value_refuse() {
        let refused = Bind::Refused {
            cause: CgroupBindRefusalCause::CgroupTreeNotWritable,
        };
        assert!(matches!(
            memory_envelope_decision(&refused, &MemoryEnvelopeRequirement::SlotRequired, true),
            MemoryEnvelopeDecision::Refused {
                cause: MemoryEnvelopeRefusalCause::Absent {
                    grain: MemoryEnvelopeGrain::Leaf
                }
            }
        ));
        assert!(matches!(
            memory_envelope_decision(
                &Bind::Applicable { bound: 1 },
                &MemoryEnvelopeRequirement::Unreadable {
                    raw: "leaf".to_string()
                },
                true
            ),
            MemoryEnvelopeDecision::Refused {
                cause: MemoryEnvelopeRefusalCause::GrainUnknown { .. }
            }
        ));
    }

    #[test]
    fn the_verdict_holds_only_on_a_clean_pair_of_readings() {
        assert!(!memory_envelope_violated(0, Some(0), Some(false), false));
        assert!(!memory_envelope_violated(0, Some(0), None, true));
        assert!(memory_envelope_violated(1, Some(0), Some(false), false));
        assert!(memory_envelope_violated(0, Some(1), Some(false), false));
        assert!(memory_envelope_violated(0, None, Some(false), false));
        assert!(memory_envelope_violated(0, Some(0), Some(true), false));
    }

    #[test]
    #[allow(non_snake_case)]
    fn RED_an_unobservable_swap_peak_refuses_unless_the_swap_limit_is_zero() {
        assert!(memory_envelope_violated(0, Some(0), None, false));
        assert!(!memory_envelope_violated(0, Some(0), None, true));
    }

    #[test]
    #[allow(non_snake_case)]
    fn RED_an_applied_or_existing_bind_over_no_numeric_max_refuses_at_its_grain() {
        assert!(matches!(
            memory_envelope_decision(
                &Bind::Applicable { bound: 1 },
                &MemoryEnvelopeRequirement::NotRequested,
                false
            ),
            MemoryEnvelopeDecision::Refused {
                cause: MemoryEnvelopeRefusalCause::Absent {
                    grain: MemoryEnvelopeGrain::Leaf
                }
            }
        ));
        assert!(matches!(
            memory_envelope_decision(&existing(), &MemoryEnvelopeRequirement::NotRequested, false),
            MemoryEnvelopeDecision::Refused {
                cause: MemoryEnvelopeRefusalCause::Absent {
                    grain: MemoryEnvelopeGrain::Existing
                }
            }
        ));
        assert!(matches!(
            memory_envelope_decision(&existing(), &MemoryEnvelopeRequirement::NotRequested, true),
            MemoryEnvelopeDecision::Bounded {
                grain: MemoryEnvelopeGrain::Existing
            }
        ));
    }
}

fn test_verb_after(
    options: &TestInvocationOptions,
    freshness: &BinaryFreshness,
) -> InvocationOutcome {
    match stale_binary_refusal(freshness) {
        Some(refused) => refused,
        None => test_verb(options),
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

/// WHAT A FOLDED BATCH CONTAINS, by route population. The three populations are the three arms of
/// `gunbc.target_invocation` `TestOperandSelection` (`exact` = `TestOperandExactTarget`, the registry's
/// to answer; `native` and `claim` the two universes' patterns), and `selected` counts the operands that
/// were admitted -- kept because a refusal that says "named nothing" must not fire on a batch whose
/// operands were all duplicates of one another.
pub struct TestOperandClasses {
    pub exact: Vec<Label>,
    pub native: Vec<TargetPattern>,
    pub claim: Vec<TargetPattern>,
    pub selected: usize,
}

impl TestOperandClasses {
    fn selected_is_empty(&self) -> bool {
        self.selected == 0
    }
    fn is_claim_selection(&self) -> bool {
        !self.claim.is_empty() && self.native.is_empty() && self.exact.is_empty()
    }
}

/// THE FOLD, WHICH IS THE ADMISSION: each operand is classified by the SAME authority a single operand is
/// classified by today (`gunbc.target_invocation` `admit_test_operand`, mirrored by the two universe tests
/// below), duplicates are dropped on the operand text so AN OVERLAP SELECTS ONCE, and the first refusal
/// ends the batch -- an operand whose pattern cannot be read is never run as if it had been.
///
/// It is a function of its operands and the two universes and nothing else, so the controls can ask it what
/// a batch denotes without selecting a producer, and the answer they check is the same one `test_verb`
/// routes on.
fn classify_test_operands(
    operands: &[String],
    native_universe: &TargetPattern,
    claim_universe: &TargetPattern,
) -> Result<TestOperandClasses, InvocationOutcome> {
    let mut exact: Vec<Label> = Vec::new();
    let mut native: Vec<TargetPattern> = Vec::new();
    let mut claim: Vec<TargetPattern> = Vec::new();
    let mut seen: Vec<String> = Vec::new();
    for operand in operands {
        if seen.iter().any(|previous| previous == operand) {
            continue;
        }
        seen.push(operand.clone());
        let pattern = match parse_target_pattern(operand) {
            Ok(pattern) => pattern,
            Err(cause) => {
                return Err(InvocationOutcome {
                    termination: Termination::Refused,
                    message: format!(
                    "gunbc test: operand is not an admitted target pattern: {operand}\n  cause: {}",
                    target_pattern_refusal_text(&cause)
                ),
                })
            }
        };
        if target_pattern_within(&pattern, native_universe) {
            native.push(pattern);
        } else if target_pattern_within(&pattern, claim_universe) {
            claim.push(pattern);
        } else {
            match pattern {
                // An exact label outside both universes is the registry's to answer (an instrument
                // subject), and an unknown one is refused as TargetIsUnknown when the registry is read.
                TargetPattern::SingleTarget(label) => exact.push(label),
                TargetPattern::PackageTargets(_) | TargetPattern::SubtreeTargets(_) => {
                    return Err(InvocationOutcome {
                        termination: Termination::Refused,
                        message: test_operand_set_form_refusal_rendered(operand),
                    })
                }
            }
        }
    }
    Ok(TestOperandClasses {
        exact,
        native,
        claim,
        selected: seen.len(),
    })
}

/// `gunbc.target_invocation` `admit_resolvable_exact_targets`, mirrored: an exact label the registry
/// does not bind, whose module resolves in the roots, moves to the claim route as a `SingleTarget`.
/// The registry is asked first, so a bound instrument subject keeps its producer, and an unbound label
/// whose module does not resolve stays exact and is refused as `TargetIsUnknown`. Both questions are
/// parameters, so the controls ask the step without a registry or a corpus.
fn admit_resolvable_exact_targets(
    classes: TestOperandClasses,
    registry_binds: &dyn Fn(&Label) -> bool,
    module_resolves: &dyn Fn(&Label) -> bool,
) -> TestOperandClasses {
    let (exact, moved): (Vec<Label>, Vec<Label>) = classes
        .exact
        .into_iter()
        .partition(|label| registry_binds(label) || !module_resolves(label));
    let mut claim = classes.claim;
    claim.extend(moved.into_iter().map(TargetPattern::SingleTarget));
    TestOperandClasses {
        exact,
        native: classes.native,
        claim,
        selected: classes.selected,
    }
}

/// Whether the instrument registry binds this exact label (`gunbc.instrument_targets`).
fn instrument_registry_binds(label: &Label) -> bool {
    let key = render_label(label);
    instrument_registry()
        .iter()
        .any(|(candidate, _)| render_label(candidate) == key)
}

/// THE OPTION REFUSAL, KEPT OUT OF THE DISPATCH SO IT CAN BE ASKED WITHOUT RUNNING ANYTHING: an option
/// belonging to the claim route, spelled on a batch that is not one, is refused with the reason that names
/// the route it landed on.
fn claim_route_option_refusal(
    classes: &TestOperandClasses,
    options: &TestInvocationOptions,
) -> Option<InvocationOutcome> {
    if classes.is_claim_selection() {
        return None;
    }
    let option = options.claim_route_option_spelled()?;
    let route = if !classes.native.is_empty() {
        "the native route compiles and builds under the native compiler's own authority"
    } else if !classes.exact.is_empty() {
        "an instrument subject is one registry binding, run by its own producer"
    } else {
        "no claim was named, so there is no claim evaluation for the option to reach"
    };
    Some(option_on_other_route_refusal(
        option,
        route,
        "the execution mode and the corpus are options on claim evaluation, which is the claim route",
    ))
}

/// WHY A MIXED BATCH IS REFUSED RATHER THAN SPLIT: one invocation's result is derived by one rule, and a
/// batch whose claims and native patterns would be answered by two executors has no single such rule.
fn mixed_batch_refusal_if_mixed(classes: &TestOperandClasses) -> Option<InvocationOutcome> {
    if classes.claim.is_empty() || (classes.native.is_empty() && classes.exact.is_empty()) {
        return None;
    }
    Some(mixed_batch_refusal(
        classes.native.len(),
        classes.claim.len(),
        classes.exact.len(),
    ))
}

pub fn test_verb(options: &TestInvocationOptions) -> InvocationOutcome {
    // THE NATIVE ROUTE'S BOUNDARY IS READ FIRST, AND ITS OWN FAILURE IS REPORTED AS ITSELF: the
    // boundary is a generated artifact of the seed build, and a batch cannot be routed without it.
    let native_universe = match native_route_default_pattern() {
        Ok(universe) => universe,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::Refused,
                message: format!(
                    "gunbc test: the native route's boundary, {:?}, is not an admitted target pattern; \
                     src/v1/stage0/src/native_route_seed_boundary_generated.rs is rendered from \
                     gunbc.native_route_universe native_route_default_pattern by \
                     gunbc.native_route_seed_boundary_emit, so regenerate it rather than route without a boundary\n  cause: {}",
                    crate::native_route_seed_boundary_generated::NATIVE_ROUTE_DEFAULT_PATTERN,
                    target_pattern_refusal_text(&cause)
                ),
            };
        }
    };
    let claim_universe = claim_route_universe();
    let classes = match classify_test_operands(&options.operands, &native_universe, &claim_universe)
    {
        Ok(classes) => classes,
        Err(refused) => return refused,
    };
    // AN EXACT CLAIM OUTSIDE `//test/claim/...` (a pinned oracle under dag/test/manual, a parse control
    // under src/v1/tests/claim) is admitted by its module resolving in the roots the batch would read.
    // The corpus is indexed only when some exact label is unbound, so an instrument run pays nothing,
    // and the index it fills is the process-shared one the claim route reads next.
    let classes = if classes
        .exact
        .iter()
        .any(|label| !instrument_registry_binds(label))
    {
        let roots = if options.source_roots.is_empty() {
            cli_run::witness_layer_roots()
        } else {
            options.source_roots.clone()
        };
        // A CORPUS THAT CANNOT BE INDEXED LEAVES THE QUESTION UNANSWERED, AND THAT IS WHAT IS REPORTED:
        // whether an unbound label names a claim is unknown, so the run is the subject unreached with
        // the index's own cause -- never `TargetIsUnknown`, which would name a real label a typo
        // (gunbc.design_argument::fail_closed: every degradation typed and located).
        let modules = match cli_run::claim_route_module_paths(&roots) {
            Ok(modules) => modules,
            Err(cause) => {
                let unbound: Vec<String> = classes
                    .exact
                    .iter()
                    .filter(|label| !instrument_registry_binds(label))
                    .map(render_label)
                    .collect();
                return InvocationOutcome {
                    termination: Termination::SubjectUnreached,
                    message: format!(
                        "gunbc test: {} is bound to no instrument, and whether it names a claim \
                         could not be decided because the claim corpus ({}) did not index — no \
                         observation\n  cause: {cause}",
                        unbound.join(", "),
                        roots.join(", ")
                    ),
                };
            }
        };
        admit_resolvable_exact_targets(classes, &instrument_registry_binds, &|label: &Label| {
            modules.contains(&label.package_segments.join("."))
        })
    } else {
        classes
    };

    if classes.selected_is_empty() {
        return InvocationOutcome {
            termination: Termination::Refused,
            message: "gunbc test: an invocation names at least one target; nothing was supplied"
                .to_string(),
        };
    }

    // THE ROUTE, DECIDED FROM WHAT THE BATCH CONTAINS. An option that belongs to the claim route is
    // refused on every other route, because there is no claim evaluation there for it to reach -- and it
    // is refused BEFORE any producer is selected, so an option cannot be silently dropped on a route
    // that ignores it and read as honoured.
    if let Some(refused) = claim_route_option_refusal(&classes, options) {
        return refused;
    }
    if let Some(refused) = mixed_batch_refusal_if_mixed(&classes) {
        return refused;
    }
    let (exact, native, claim) = (classes.exact, classes.native, classes.claim);
    if !claim.is_empty() {
        let execution_mode = match options.admitted_execution_mode() {
            Ok(explicit) => cli_run::execution_mode_for(
                explicit, /*is_claim*/ true, /*dry_run*/ false,
            ),
            Err(reason) => {
                return InvocationOutcome {
                    termination: Termination::Refused,
                    message: format!("gunbc test: {reason}"),
                }
            }
        };
        let roots = match options.admitted_source_roots(&cli_run::witness_layer_roots()) {
            Ok(roots) => roots,
            Err(reason) => {
                return InvocationOutcome {
                    termination: Termination::Refused,
                    message: format!("gunbc test: {reason}"),
                }
            }
        };
        return run_claim_selection(
            &claim,
            &roots,
            execution_mode,
            cli_run::claim_fixture_store_rc(&options.fixture_store_path()),
        );
    }
    if !native.is_empty() {
        if native.len() != 1 {
            return single_operand_route_refusal("native test", native.len());
        }
        return run_native_test_route(&native[0]);
    }
    if exact.len() != 1 {
        return single_operand_route_refusal("instrument registry", exact.len());
    }
    let label = &exact[0];
    let key = render_label(label);
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
        Some((bound, producer)) => run_producer(producer, &bound.target),
    }
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
    // THE HOST TRANSCRIBES, THE .dag FOLD DECIDES (gunbc.floor.typed_graph_exclusive_bytes). Every
    // reading crosses as primitives; a count past i64 refuses rather than clamping.
    const ENTRY: &str = "dag/gunbc/floor/typed_graph_exclusive_bytes.dag";
    const REPORT: &str = "typed_graph_exclusive_report";
    let roots = vec!["dag".to_string(), "src/v2".to_string()];
    let (_, ctx) = match cli_run::load_entry_for_functions(
        &roots,
        ENTRY,
        &[REPORT],
        crate::v1_interpreter::ExecutionMode::Wet,
    ) {
        Ok(resolved) => resolved,
        Err(cause) => {
            return InvocationOutcome {
                termination: Termination::SubjectUnreached,
                message: format!("{label}: resolve failed for {ENTRY}: {cause}"),
            }
        }
    };
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
    let report = match crate::v1_interpreter::run_in_context_with_args(&ctx, REPORT, &args, true) {
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
        "{label} graph_total={} sum_of_exclusives={} shared_or_unlisted={residual} (gunbc.floor.typed_graph_exclusive_bytes; graph_total from the first run, every class from its own fresh resolve)",
        bytes(ctx.field(fields, "graph_total")),
        bytes(ctx.field(fields, "sum_of_exclusives")),
    ));
    InvocationOutcome {
        termination: Termination::ObservationHeld,
        message: lines.join("\n"),
    }
}

/// ONE SCRATCH CLAIM CORPUS, written where the module index will find it and named so two controls that
/// build different corpora cannot read each other's. `name` is the directory suffix; `files` are
/// `(file, source)` pairs placed under `<dir>/test/claim`, which is the claim universe's location under a
/// root -- the same shape the route's own discovery expects.
#[cfg(test)]
fn scratch_claim_root(name: &str, files: &[(&str, &str)]) -> String {
    let dir = cli_run::workspace_root()
        .join("target")
        .join(format!("gunbc_claim_route_contract_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    let claim_dir = dir.join("test").join("claim");
    std::fs::create_dir_all(&claim_dir).expect("scratch claim dir");
    for (file, body) in files {
        std::fs::write(claim_dir.join(file), body).expect("scratch claim source");
    }
    dir.to_string_lossy().into_owned()
}

#[cfg(test)]
mod claim_route_result_contract_tests {
    // THE CONTRACT: one located outcome per selected claim, with printed lines AND the exit status
    // both derived from that single collection -- and the seed adapter `claim_route_verdict` reading
    // the typed ProcessExit code 2 as the subject unreached. These run the REAL seed path: the real
    // `claim_route_verdict` adapter and the real `claim_route_fold` (not the `.dag` model's pure
    // fold), and, for the outcomes a `.dag` claim can actually return, the real `cli_run::run_claim_route`
    // selection-and-execution. Executed locally / on a fleet host, never by required CI
    // (gunbc.design_ruling::no_ci_claim_execution_until_native_frontier_clears).

    use super::*;
    use crate::v1_interpreter::RaisePath;
    use cli_run::{
        ClaimMemberOutcome as M, ClaimOutcome as O, ClaimRouteMember, EntryResolveRefusal as R,
        PanicPayloadKind as K, WitnessRuntimeCause,
    };

    /// An EVALUATED member as the route would hand it to the fold. The receipts are irrelevant to
    /// the verdict (only the outcome is read) and so are pinned to zero here.
    fn member(module_path: &str, function: &str, outcome: O) -> ClaimRouteMember {
        ClaimRouteMember {
            module_path: module_path.to_string(),
            function: function.to_string(),
            outcome: M::Evaluated(outcome),
            eval_steps: 0,
            cpu_ms: 0,
        }
    }

    /// An UNRESOLVED member: the entry carried this identity but its refusal says what kind of
    /// failure it is. The fold reads it by the resolve authority's rule, not as a claim outcome.
    fn unresolved_member(module_path: &str, function: &str, refusal: R) -> ClaimRouteMember {
        ClaimRouteMember {
            module_path: module_path.to_string(),
            function: function.to_string(),
            outcome: M::Unresolved(refusal),
            eval_steps: 0,
            cpu_ms: 0,
        }
    }

    fn runtime_error() -> O {
        O::RuntimeError {
            cause: WitnessRuntimeCause::DivisionByZero,
            message: "division by zero".into(),
            raised_in: RaisePath {
                frames: Vec::new(),
                elided: 0,
            },
        }
    }

    fn budget_interrupt() -> O {
        O::BudgetInterrupted {
            elapsed_at_least_ms: 1000,
            budget_ms: 1000,
            kind: cli_run::BudgetKind::Cpu,
        }
    }

    /// (printed lines, exit status, fold) for one collection -- the only place the route turns
    /// outcomes into a result.
    fn route_fold(members: &[ClaimRouteMember]) -> (Vec<String>, i32, ClaimRouteFold) {
        let fold = claim_route_fold(members);
        let status = invocation_exit_status(fold.termination());
        (fold.lines.clone(), status, fold)
    }

    // ---- the line schema (gunbc ruling, manager msg_1c4665df) -----------------------------
    // <label> <VERDICT> outcome=<kind> [cause=<token>] [code=<n>] [at=<file:line:col>] [detail="<quoted>"] eval_steps=<n> cpu_ms=<n>
    // Parsers key ONLY on label, verdict, outcome, cause, code, at -- never on detail.

    /// A runtime-error line carries the CLOSED kind and the typed cause token, NOT Rust `Debug`
    /// output (Debug is not a schema -- it re-renders whenever a field is added). The frozen policy
    /// reads a runtime error as NO OBSERVATION (exit 2), never as a definite failure: a raise is the
    /// subject not producing an answer, not the subject answering false.
    #[test]
    fn a_runtime_error_line_carries_the_closed_kind_and_cause_not_debug() {
        let (lines, status, _) = route_fold(&[member("test/claim/x", "boom", runtime_error())]);
        assert_eq!(
            status, 2,
            "a runtime error is no observation, not a failure"
        );
        let line = lines
            .iter()
            .find(|l| l.starts_with("//test/claim/x:boom"))
            .expect("one located line for the identity");
        // Fields parsers key on, verbatim.
        assert!(line.contains("UNOBSERVED"), "the verdict word: {line}");
        assert!(line.contains("outcome=RuntimeError"), "closed kind: {line}");
        assert!(
            line.contains("cause=division-by-zero"),
            "the closed cause token: {line}"
        );
        // The human text is in detail, and it is the last free-text field before the counters.
        assert!(
            line.contains("detail=\"division by zero\""),
            "quoted detail: {line}"
        );
        assert!(
            line.trim_end().ends_with("eval_steps=0 cpu_ms=0"),
            "counters last: {line}"
        );
        // Debug output would open the arm with a brace -- the kind name must not.
        assert!(
            !line.contains("outcome=RuntimeError {"),
            "no Debug rendering: {line}"
        );
    }

    /// The `detail` field is one double-quoted, escaped, SINGLE-LINE string and always the last
    /// free-text field: a panic payload carrying a quote, a backslash and a newline stays on one
    /// line and never leaks a raw newline that would break a parser's per-line read.
    #[test]
    fn a_detail_is_one_escaped_single_line_field_last() {
        let payload = "boom\nsay \"hi\" \\ done";
        let panicked = O::Panicked {
            payload: payload.to_string(),
            payload_kind: K::StringPayload,
        };
        let (lines, status, _) = route_fold(&[member("test/claim/x", "pan", panicked)]);
        assert_eq!(status, 2, "a panic is no observation");
        let line = lines
            .iter()
            .find(|l| l.starts_with("//test/claim/x:pan"))
            .expect("one located line for the identity");
        assert!(line.contains("UNOBSERVED"), "a panic is unobserved: {line}");
        assert!(line.contains("outcome=Panicked"), "closed kind: {line}");
        // The payload kind is StringPayload here, so the typed cause is the string arm.
        assert!(
            line.contains("cause=string_payload"),
            "closed cause: {line}"
        );
        // The whole record is ONE physical line even though the payload carried a newline.
        assert!(!line.contains('\n'), "single line: {line:?}");
        let after_detail = line.split("detail=").nth(1).expect("a detail field");
        assert!(after_detail.starts_with('"'), "detail opens quoted");
        assert!(
            after_detail.ends_with(" eval_steps=0 cpu_ms=0"),
            "detail precedes the trailing counters: {after_detail}"
        );
        // The payload's inner quote is escaped, so the quoted field does not terminate early.
        assert!(
            after_detail.contains("\\\"hi\\\""),
            "the inner quote is escaped: {after_detail}"
        );
        assert!(
            !after_detail.contains("boom\n"),
            "no raw newline from the payload survives"
        );
        // The words survived the escaping round trip.
        assert!(after_detail.contains("boom") && after_detail.contains("done"));
    }

    /// A NON-string panic payload (the kind a `panic_any(42u32)` produces -- two real floor claims do
    /// this) must carry `cause=non_string_payload`, never the fabricated `string_payload`. The kind is
    /// decided at the catch site (`panic_payload_text`), and the fold renders the typed kind.
    #[test]
    fn a_non_string_panic_line_carries_non_string_payload_cause() {
        // `panic_payload_text` substitutes the placeholder text for a non-string payload; the kind is
        // what distinguishes it, so a wrong kind is a wrong machine key on a real crash.
        let placeholder =
            "panic payload is not a string; see the panic hook's stderr line for the \
                           message and location";
        let (lines, status, _) = route_fold(&[member(
            "test/claim/x",
            "nonstring",
            O::Panicked {
                payload: placeholder.to_string(),
                payload_kind: K::NonStringPayload,
            },
        )]);
        assert_eq!(status, 2, "a panic is no observation");
        let line = lines
            .iter()
            .find(|l| l.starts_with("//test/claim/x:nonstring"))
            .expect("one located line for the identity");
        assert!(line.contains("outcome=Panicked"), "closed kind: {line}");
        assert!(
            line.contains("cause=non_string_payload"),
            "the non-string kind, never string_payload: {line}"
        );
        assert!(
            !line.contains("cause=string_payload"),
            "must not mislabel a non-string payload: {line}"
        );
    }

    // ---- held ---------------------------------------------------------------------------
    #[test]
    fn a_held_claim_is_held_and_the_route_exits_zero() {
        let (lines, status, fold) = route_fold(&[member("test.claim.zz", "a_holds", O::Pass)]);
        assert_eq!(status, 0);
        assert_eq!(lines.len(), 1);
        assert!(
            lines[0].contains("a_holds") && lines[0].contains("HELD"),
            "{lines:?}"
        );
        assert_eq!(fold.count(ClaimRouteVerdict::ClaimHeld), 1);
    }

    // ---- false --------------------------------------------------------------------------
    #[test]
    fn a_false_claim_did_not_hold_and_the_route_exits_one() {
        let (lines, status, _) = route_fold(&[member("test.claim.zz", "b_fails", O::Fail)]);
        assert_eq!(status, 1);
        assert!(lines[0].contains("b_fails") && lines[0].contains("DID-NOT-HOLD"));
    }

    // ---- non-Bool -----------------------------------------------------------------------
    #[test]
    fn a_non_bool_claim_is_an_observation_that_did_not_hold() {
        let (_lines, status, fold) = route_fold(&[member(
            "test.claim.zz",
            "c_nonbool",
            O::NotBool { got: "5".into() },
        )]);
        assert_eq!(
            status, 1,
            "a non-Bool observation is did-not-hold, never unobserved"
        );
        assert_eq!(fold.count(ClaimRouteVerdict::ClaimDidNotHold), 1);
    }

    // ---- runtime error ------------------------------------------------------------------
    #[test]
    fn a_runtime_error_is_unobserved_and_the_route_exits_two() {
        let (_lines, status, fold) =
            route_fold(&[member("test.claim.zz", "d_error", runtime_error())]);
        assert_eq!(status, 2);
        assert_eq!(fold.count(ClaimRouteVerdict::ClaimUnobserved), 1);
    }

    // ---- unwind -------------------------------------------------------------------------
    #[test]
    fn an_unwind_is_unobserved_not_a_failing_claim() {
        let (_lines, status, fold) = route_fold(&[member(
            "test.claim.zz",
            "e_unwind",
            O::Panicked {
                payload: "boom".into(),
                payload_kind: K::StringPayload,
            },
        )]);
        assert_eq!(status, 2);
        assert_eq!(fold.count(ClaimRouteVerdict::ClaimUnobserved), 1);
    }

    // ---- budget interrupt ---------------------------------------------------------------
    #[test]
    fn a_budget_interrupt_is_unobserved_not_a_failing_claim() {
        let (_lines, status, fold) =
            route_fold(&[member("test.claim.zz", "f_budget", budget_interrupt())]);
        assert_eq!(status, 2);
        assert_eq!(fold.count(ClaimRouteVerdict::ClaimUnobserved), 1);
    }

    // ---- ExitFailure code 1 (the definite refusal) --------------------------------------
    #[test]
    fn a_definite_process_exit_refusal_is_did_not_hold() {
        let (_lines, status, fold) = route_fold(&[member(
            "test.claim.zz",
            "g_exit1",
            O::ExitFailure {
                code: 1,
                reason: Some("refused".into()),
            },
        )]);
        assert_eq!(
            status, 1,
            "a code-1 protocol refusal is an observation that did not hold"
        );
        assert_eq!(fold.count(ClaimRouteVerdict::ClaimDidNotHold), 1);
    }

    // ---- ExitFailure code 2 (THE REPAIR) ------------------------------------------------
    #[test]
    fn a_process_exit_code_two_is_unobserved_not_a_failing_claim() {
        let (lines, status, fold) = route_fold(&[member(
            "test.claim.zz",
            "h_exit2",
            O::ExitFailure {
                code: 2,
                reason: Some("unreached".into()),
            },
        )]);
        assert_eq!(
            status, 2,
            "code 2 is the claim-runner's own 'subject unreached' protocol result"
        );
        assert_eq!(fold.count(ClaimRouteVerdict::ClaimUnobserved), 1);
        assert!(
            lines[0].contains("UNOBSERVED"),
            "must not be reported as a failing claim: {lines:?}"
        );
    }

    // ---- false together with unobserved: exits 1 and reports both -----------------------
    #[test]
    fn a_false_claim_beside_an_unobserved_exit_reports_both_and_exits_one() {
        let (lines, status, fold) = route_fold(&[
            member("test.claim.zz", "i_fails", O::Fail),
            member(
                "test.claim.zz",
                "j_unreached",
                O::ExitFailure {
                    code: 2,
                    reason: None,
                },
            ),
        ]);
        assert_eq!(status, 1, "a definite failure dominates");
        assert_eq!(fold.count(ClaimRouteVerdict::ClaimDidNotHold), 1);
        assert_eq!(fold.count(ClaimRouteVerdict::ClaimUnobserved), 1);
        // BOTH are reported, on their own located lines.
        assert!(lines
            .iter()
            .any(|l| l.contains("i_fails") && l.contains("DID-NOT-HOLD")));
        assert!(lines
            .iter()
            .any(|l| l.contains("j_unreached") && l.contains("UNOBSERVED")));
    }

    // ---- empty selection exits 2 --------------------------------------------------------
    #[test]
    fn an_empty_selection_is_the_subject_unreached() {
        let (lines, status, fold) = route_fold(&[]);
        assert_eq!(status, 2);
        assert!(lines.is_empty());
        assert!(fold.verdicts.is_empty());
    }

    // ---- a resolve failure is NOT a finding: it is the absence of an observation -------------
    //
    // The route calls the TYPED resolve, so each refusal carries its own kind, and the fold maps
    // EVERY unresolved entry to UNOBSERVED (plan section 2: a known selected claim that cannot
    // execute gets an unobserved outcome with its cause). A resolve finding is not evidence the claim
    // evaluated and did not hold. `NotAttempted` is NOT reused here -- it means "a sibling's unwind
    // stopped the line."
    #[test]
    fn a_closure_diagnostic_is_unobserved_and_held_siblings_still_run() {
        // The brief's A/B/C case: A holds, B cannot resolve, C holds. A and C run; B is located as
        // an unobserved outcome carrying its cause; the aggregate is subject unreached (exit 2), not
        // success and not a finding.
        let (lines, status, fold) = route_fold(&[
            member("test.claim.zz", "a_holds", O::Pass),
            unresolved_member(
                "test.claim.zz",
                "b_broken",
                R::ClosureDiagnostics {
                    message: "name 'phantom_fn' not found in test.claim.zz_broken".into(),
                },
            ),
            member("test.claim.zz", "c_holds", O::Pass),
        ]);
        assert_eq!(
            status, 2,
            "one unobserved among held siblings, no definite failure -> subject unreached (exit 2)"
        );
        assert_eq!(
            fold.count(ClaimRouteVerdict::ClaimHeld),
            2,
            "A and C still run"
        );
        assert_eq!(
            fold.count(ClaimRouteVerdict::ClaimUnobserved),
            1,
            "B is an absence of an observation, not a finding"
        );
        assert_eq!(fold.count(ClaimRouteVerdict::ClaimDidNotHold), 0);
        assert!(
            lines.iter().any(|l| l.contains("b_broken")
                && l.contains("UNOBSERVED")
                && l.contains("outcome=ClosureDiagnostics")
                && l.contains("phantom_fn")),
            "B is located as unobserved with its typed arm keyed and text in detail: {lines:?}"
        );
        // A and C report their own held lines -- one resolve failure does not abort the others.
        assert!(lines
            .iter()
            .any(|l| l.contains("a_holds") && l.contains("HELD")));
        assert!(lines
            .iter()
            .any(|l| l.contains("c_holds") && l.contains("HELD")));
    }

    #[test]
    fn a_missing_entry_is_no_observation() {
        let (lines, status, fold) = route_fold(&[unresolved_member(
            "test.claim.zz",
            "b_missing",
            R::EntryMissing {
                message: "entry test/claim/zz_missing.dag is absent".into(),
            },
        )]);
        assert_eq!(status, 2);
        assert_eq!(fold.count(ClaimRouteVerdict::ClaimUnobserved), 1);
        assert!(
            lines[0].contains("UNOBSERVED")
                && lines[0].contains("outcome=EntryMissing")
                && lines[0].contains("detail=\"entry "),
            "the located arm is keyed and the refusal text is in detail: {lines:?}"
        );
    }

    #[test]
    fn a_resolve_infrastructure_refusal_is_no_observation() {
        let (_lines, status, fold) = route_fold(&[unresolved_member(
            "test.claim.zz",
            "b_infra",
            R::ResolveInfrastructure {
                message: "HostBudgetUnreadable: no modeled host memory source".into(),
            },
        )]);
        assert_eq!(
            status, 2,
            "an unresolved infrastructure refusal is no observation, never a finding"
        );
        assert_eq!(fold.count(ClaimRouteVerdict::ClaimUnobserved), 1);
    }

    // ---- a duplicated outcome for A cannot satisfy B ------------------------------------
    #[test]
    fn a_duplicated_outcome_for_one_identity_cannot_satisfy_a_missing_one() {
        // Two outcomes for A, none for B: the contract is one outcome per identity, so the route
        // fails closed rather than reporting success on a doubled A.
        let (lines, status, fold) = route_fold(&[
            member("test.claim.zz", "a", O::Pass),
            member("test.claim.zz", "a", O::Pass),
        ]);
        assert_ne!(status, 0, "a doubled A must not certify success");
        assert_eq!(status, 2);
        assert!(fold.duplicate.as_deref().is_some());
        assert_eq!(
            fold.verdicts.len(),
            1,
            "the duplicate is dropped, not counted"
        );
        assert_eq!(lines.len(), 1, "only the first line for A survives");

        // The discriminating control: the SAME two held outcomes across two DISTINCT identities DO
        // satisfy the contract -- so the failure above is about the identity join, not the count.
        let (_l2, status2, fold2) = route_fold(&[
            member("test.claim.zz", "a", O::Pass),
            member("test.claim.zz", "b", O::Pass),
        ]);
        assert_eq!(status2, 0);
        assert!(fold2.duplicate.is_none());
    }

    // ---- a duplicate blocks success but NEVER overrides a definite failure -----------------
    #[test]
    fn a_duplicate_does_not_override_a_definite_failure() {
        // A Pass, A Pass (the second dropped as a duplicate), B Fail. The rule gives exit 1 (a
        // definite failure dominates), NOT the subject unreached a lone duplicate would give.
        let (lines, status, fold) = route_fold(&[
            member("test.claim.zz", "a", O::Pass),
            member("test.claim.zz", "a", O::Pass),
            member("test.claim.zz", "b", O::Fail),
        ]);
        assert_eq!(
            status, 1,
            "a definite failure still gives 1 despite the duplicate"
        );
        assert!(
            fold.duplicate.as_deref().is_some(),
            "the duplicate is still recorded (it blocks success)"
        );
        assert_eq!(fold.count(ClaimRouteVerdict::ClaimDidNotHold), 1);
        assert!(
            lines
                .iter()
                .any(|l| l.contains("b") && l.contains("DID-NOT-HOLD")),
            "B's definite failure is reported on its own line: {lines:?}"
        );
    }

    // ---- the real selection-and-execution path, end to end ------------------------------
    //
    // These drive the WHOLE seed claim route on scratch sources -- the floor's discovery authority,
    // the real per-claim evaluation, then the one fold -- which is "the real gunbc test path" as
    // opposed to the pure adapter/fold controls above. Discovery resolves the floor's own authority
    // module from the real witness roots and the scratch claims from the scratch root; entry
    // resolution reads workspace-relative paths against the process cwd (the `gunbc` binary stands
    // at the workspace root, a cargo test at the crate), so each run pins it to the workspace root
    // -- the value the seed's own `run_claim_route` sets it to anyway.
    //
    // They are `#[ignore]`d because they index the whole `.dag` corpus. Run them with `--release`
    // (a debug interpreter is far too slow to finish the corpus index walk; the release walk is on
    // the order of minutes), one at a time, so each command and its output stand alone as evidence.
    // Never by required CI (gunbc.design_ruling::no_ci_claim_execution_until_native_frontier_clears):
    //     cargo test -p v1-compiler --release --lib claim_route_result_contract_tests::<name> -- --ignored --test-threads=1
    //
    // ONE scratch corpus carries every scenario; a single route run selects them all and returns the
    // ONE collection every assertion below is drawn from -- so the contract is checked over the very
    // collection the route produced, not a hand-assembled one.
    #[test]
    #[ignore = "indexes the whole .dag corpus; run with --release (see the module comment)"]
    fn the_real_route_gives_each_selected_identity_one_located_outcome() {
        let members = real_route(&[
            // A and C: resolvable entries, both hold.
            ("zz_ac.dag", "module test.claim.zz_ac\n\ntest fn zz_a() -> Bool {\n  true\n}\n\ntest fn zz_c() -> Bool {\n  true\n}\n"),
            // B: its entry imports a module the subject does not hold, so the entry cannot resolve.
            ("zz_broken.dag", "module test.claim.zz_broken\nimport test.claim.zz_nonexistent_module_xyz { phantom_fn }\n\ntest fn zz_b() -> Bool {\n  phantom_fn()\n}\n"),
            // The code-2 repair, end to end: a held claim and an unobserved ProcessExit code 2.
            ("zz_exits.dag", "module test.claim.zz_exits\nimport std.process { ProcessExit, ExitSuccess, ExitFailure }\n\ntest fn held() -> Bool {\n  true\n}\n\ntest fn unreached() -> ProcessExit {\n  ExitFailure { code: 2, reason: \"the subject was not reached\" }\n}\n"),
            // The value-returning outcomes the real runner produces.
            ("zz_truths.dag", "module test.claim.zz_truths\nimport std.process { ProcessExit, ExitSuccess, ExitFailure }\n\ntest fn passes() -> Bool {\n  true\n}\n\ntest fn fails() -> Bool {\n  false\n}\n\ntest fn nonbool() -> Int {\n  5\n}\n\ntest fn refused() -> ProcessExit {\n  ExitFailure { code: 1, reason: \"a definite refusal\" }\n}\n"),
        ]);

        let outcome_of = |name: &str| -> M {
            members
                .iter()
                .find(|m| m.function == name)
                .unwrap_or_else(|| panic!("the selected identity {name} must appear once"))
                .outcome
                .clone()
        };

        // Each selected identity is present EXACTLY once and carries the outcome its runner observed.
        for name in ["zz_a", "zz_c", "held", "passes"] {
            assert!(
                matches!(outcome_of(name), M::Evaluated(O::Pass)),
                "{name} must hold"
            );
            assert_eq!(
                members.iter().filter(|m| m.function == name).count(),
                1,
                "{name} appears exactly once"
            );
        }
        assert!(matches!(outcome_of("fails"), M::Evaluated(O::Fail)));
        assert!(matches!(
            outcome_of("nonbool"),
            M::Evaluated(O::NotBool { .. })
        ));
        assert!(matches!(
            outcome_of("refused"),
            M::Evaluated(O::ExitFailure { code: 1, .. })
        ));
        // THE REPAIR, end to end: the runner hands back the typed ProcessExit; the fold reads code 2.
        assert!(
            matches!(
                outcome_of("unreached"),
                M::Evaluated(O::ExitFailure { code: 2, .. })
            ),
            "the code-2 claim must arrive as a typed ProcessExit, not a Bool"
        );
        // B's entry cannot resolve: a phantom import is a CLOSURE DIAGNOSTIC carried as the typed
        // refusal -- an ABSENCE of an observation (unobserved) with its cause, NOT a finding that the
        // claim evaluated and did not hold (plan section 2) -- on exactly one located line.
        assert!(
            matches!(
                outcome_of("zz_b"),
                M::Unresolved(R::ClosureDiagnostics { .. })
            ),
            "B's phantom import must be a located closure refusal: {:?}",
            outcome_of("zz_b")
        );

        // Both the printed lines and the exit status come from THIS one collection.
        let (lines, status, fold) = route_fold(&members);
        assert_eq!(members.len(), 9, "one outcome per selected identity");
        assert_eq!(fold.verdicts.len(), 9);
        assert_eq!(fold.count(ClaimRouteVerdict::ClaimHeld), 4); // zz_a, zz_c, held, passes
        assert_eq!(
            fold.count(ClaimRouteVerdict::ClaimDidNotHold),
            3,
            "the definite observations: fails, nonbool, refused(1)"
        );
        assert_eq!(
            fold.count(ClaimRouteVerdict::ClaimUnobserved),
            2,
            "the code-2 exit AND B's unresolved closure -- both absences of an observation"
        );
        // A definite failure is present, so the route exits 1 -- and A and C still ran, B is located.
        assert_eq!(status, 1);
        assert_eq!(lines.len(), 9);
        assert!(
            lines
                .iter()
                .any(|l| l.contains("unreached") && l.contains("UNOBSERVED")),
            "code 2 must be reported unobserved, never as a failing claim: {lines:?}"
        );
        assert!(
            lines.iter().any(|l| l.contains("zz_b")
                && l.contains("UNOBSERVED")
                && l.contains("outcome=ClosureDiagnostics")),
            "B is located on its own unobserved line with its typed cause: {lines:?}"
        );
        assert!(lines
            .iter()
            .any(|l| l.contains("zz_a") && l.contains("HELD")));
    }

    // ---- an empty selection through the real route is the subject unreached ---------------
    #[test]
    #[ignore = "indexes the whole .dag corpus; run with --release (see the module comment)"]
    fn an_empty_selection_through_the_real_route_is_the_subject_unreached() {
        let members = real_route_named(
            "zz_ac.dag",
            "module test.claim.zz_ac\n\ntest fn zz_a() -> Bool {\n  true\n}\n",
            "test.claim.zz_matches_nothing",
        );
        assert!(members.is_empty(), "selecting nothing yields no outcome");
        let (lines, status, fold) = route_fold(&members);
        assert_eq!(status, 2);
        assert!(lines.is_empty());
        assert!(fold.verdicts.is_empty());
    }

    // ---- scratch claim root + the real route --------------------------------------------
    fn scratch_root(name: &str, files: &[(&str, &str)]) -> String {
        let dir = cli_run::workspace_root()
            .join("target")
            .join(format!("gunbc_claim_route_contract_{name}"));
        let _ = std::fs::remove_dir_all(&dir);
        let claim_dir = dir.join("test").join("claim");
        std::fs::create_dir_all(&claim_dir).expect("scratch claim dir");
        for (file, body) in files {
            std::fs::write(claim_dir.join(file), body).expect("scratch claim source");
        }
        dir.to_string_lossy().into_owned()
    }

    /// Run the REAL claim route over one scratch corpus, selecting ONLY the scratch modules (the
    /// `zz_` prefix is unique to these fixtures -- no real `test.claim.zz_*` module exists), so the
    /// route evaluates the scratch claims and not the floor. This is the host route's own discipline
    /// (the operand narrows the module before evaluation); a `_|_ true` selector would select every
    /// corpus claim and run the floor, which is out of scope here.
    fn real_route(files: &[(&str, &str)]) -> Vec<ClaimRouteMember> {
        real_route_run(files, &|module| module.starts_with("test.claim.zz_"))
    }

    /// As `real_route`, but selecting only `selected_module` (so a corpus can drive an empty
    /// selection by naming a module it does not contain).
    fn real_route_named(file: &str, body: &str, selected_module: &str) -> Vec<ClaimRouteMember> {
        real_route_run(&[(file, body)], &|module| module == selected_module)
    }

    fn real_route_run(
        files: &[(&str, &str)],
        module_selected: &dyn Fn(&str) -> bool,
    ) -> Vec<ClaimRouteMember> {
        std::env::set_current_dir(cli_run::workspace_root()).expect("enter workspace");
        let scratch = scratch_root("shared", files);
        // The floor discovery authority and every real import a scratch fixture's closure can reach
        // are under the floor's own witness roots, so the index walk uses them; the scratch dir is
        // prepended so the scratch modules themselves are in the index.
        let mut roots = vec![scratch];
        roots.extend(cli_run::witness_layer_roots());
        // The witness exercises SELECTION, not a mode: it asks the question an unspelled `gunbc test`
        // asks, which `execution_mode_for` answers Hermetic, and passes no fixture store.
        cli_run::run_claim_route(
            &roots,
            &[],
            module_selected,
            &|_, _| true,
            crate::v1_interpreter::ExecutionMode::Hermetic,
            None,
        )
        .expect("the scratch corpus must resolve to a collection, not a route refusal")
    }

    fn route_over(
        roots: &[String],
        scopes: &[cli_run::ClaimSelectionScope],
        module_selected: &dyn Fn(&str) -> bool,
    ) -> Result<Vec<ClaimRouteMember>, String> {
        std::env::set_current_dir(cli_run::workspace_root()).expect("enter workspace");
        cli_run::run_claim_route(
            roots,
            scopes,
            module_selected,
            &|_, _| true,
            crate::v1_interpreter::ExecutionMode::Hermetic,
            None,
        )
    }

    fn module_scope(module: &str, beneath: bool) -> cli_run::ClaimSelectionScope {
        cli_run::ClaimSelectionScope {
            module_segments: module.split('.').map(str::to_string).collect(),
            beneath,
        }
    }

    fn outcomes_of(members: &[ClaimRouteMember], module: &str) -> Vec<(String, M)> {
        let mut out: Vec<(String, M)> = members
            .iter()
            .filter(|m| m.module_path == module)
            .map(|m| (m.function.clone(), m.outcome.clone()))
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    fn labels_of(members: &[ClaimRouteMember]) -> Vec<String> {
        let mut labels: Vec<String> = members
            .iter()
            .map(|m| format!("{}:{}", m.module_path, m.function))
            .collect();
        labels.sort();
        labels
    }

    struct GovernedClaimPackage(std::path::PathBuf);

    impl GovernedClaimPackage {
        fn new(package: &str, files: &[(&str, &str)]) -> Self {
            let dir = cli_run::workspace_root()
                .join("dag/test/claim")
                .join(package);
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("governed fixture package");
            for (file, body) in files {
                std::fs::write(dir.join(file), body).expect("governed fixture source");
            }
            GovernedClaimPackage(dir)
        }
    }

    impl Drop for GovernedClaimPackage {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    #[ignore = "drives the real claim route over the live corpus; run with --release --ignored"]
    fn a_generated_provider_is_acquired_whatever_else_the_batch_selects() {
        let importer = "test.claim.zz_coercion_importer";
        let scratch = scratch_root(
            "provider_batch",
            &[(
                "zz_coercion_importer.dag",
                "module test.claim.zz_coercion_importer\n\nimport std.coercion { dag_can_cast }\n\ntest fn zz_imports_the_route() -> Bool {\n  dag_can_cast(source_type: \"Int\", target_type: \"Int\") || true\n}\n",
            )],
        );
        let mut roots = vec![scratch];
        roots.extend(cli_run::witness_layer_roots());
        let crossing = "test.claim.host_text_crossing_entry_witness";
        let isolated = route_over(&roots, &[module_scope(crossing, false)], &|m| m == crossing)
            .expect("the isolated crossing witness is a collection, not a route refusal");
        let batched = route_over(
            &roots,
            &[module_scope(crossing, false), module_scope(importer, false)],
            &|m| m == crossing || m == importer,
        )
        .expect("the batched crossing witness is a collection, not a route refusal");
        let held = vec![
            (
                "a_nonempty_host_text_crosses_to_a_nonempty_sequence".to_string(),
                M::Evaluated(O::Pass),
            ),
            (
                "an_empty_host_text_crosses_to_an_empty_sequence".to_string(),
                M::Evaluated(O::Pass),
            ),
        ];
        assert_eq!(outcomes_of(&isolated, crossing), held, "isolated");
        assert_eq!(outcomes_of(&batched, crossing), held, "batched");
    }

    #[test]
    #[ignore = "drives the real claim route over the live corpus; run with --release --ignored"]
    fn a_bare_name_binds_alike_whatever_else_the_batch_selects() {
        let parent = "test.claim.zz_bare_parent";
        let child = "test.claim.zz_bare_parent.child";
        let scratch = scratch_root(
            "bare_batch",
            &[
                (
                    "zz_bare_parent.dag",
                    "module test.claim.zz_bare_parent\n\nfn zz_bare_unique_helper() -> Bool {\n  true\n}\n\ntest fn zz_parent_calls_its_helper() -> Bool {\n  zz_bare_unique_helper()\n}\n",
                ),
                (
                    "zz_bare_child.dag",
                    "module test.claim.zz_bare_parent.child\n\ntest fn zz_child_calls_the_parent_helper() -> Bool {\n  zz_bare_unique_helper()\n}\n",
                ),
            ],
        );
        let mut roots = vec![scratch];
        roots.extend(cli_run::witness_layer_roots());
        let isolated = route_over(&roots, &[module_scope(child, false)], &|m| m == child)
            .expect("the isolated child is a collection, not a route refusal");
        let batched = route_over(
            &roots,
            &[module_scope(child, false), module_scope(parent, false)],
            &|m| m == child || m == parent,
        )
        .expect("the batched child is a collection, not a route refusal");
        assert_eq!(outcomes_of(&isolated, child).len(), 1);
        assert_eq!(
            outcomes_of(&isolated, child),
            outcomes_of(&batched, child),
            "the child's bare name must bind alike whether or not its parent is selected"
        );
        assert_eq!(
            outcomes_of(&batched, parent),
            vec![(
                "zz_parent_calls_its_helper".to_string(),
                M::Evaluated(O::Pass)
            )],
            "the helper binds for the module that declares it"
        );
    }

    #[test]
    #[ignore = "drives the real claim route over the live corpus; run with --release --ignored"]
    fn every_spelling_of_a_governed_root_selects_the_same_subtree() {
        let package = format!("zz_claim_route_spelling_{}", std::process::id());
        let _fixture = GovernedClaimPackage::new(
            &package,
            &[
                (
                    "a.dag",
                    &format!(
                        "module test.claim.{package}.a\n\ntest fn zz_a() -> Bool {{\n  true\n}}\n"
                    ),
                ),
                (
                    "b.dag",
                    &format!(
                        "module test.claim.{package}.b\n\ntest fn zz_b() -> Bool {{\n  true\n}}\n"
                    ),
                ),
            ],
        );
        let subtree = format!("test.claim.{package}");
        let prefix = format!("{subtree}.");
        let absolute = cli_run::workspace_root()
            .join("dag")
            .to_string_lossy()
            .into_owned();
        let labels_under = |dag: &str| {
            let roots = vec![dag.to_string(), "src/v2".to_string()];
            labels_of(
                &route_over(&roots, &[module_scope(&subtree, true)], &|m| {
                    m.starts_with(&prefix)
                })
                .unwrap_or_else(|cause| panic!("--source-root {dag}: {cause}")),
            )
        };
        let expected = vec![format!("{subtree}.a:zz_a"), format!("{subtree}.b:zz_b")];
        assert_eq!(labels_under("dag"), expected, "dag");
        assert_eq!(labels_under("./dag"), expected, "./dag");
        assert_eq!(labels_under(&absolute), expected, "{absolute}");
    }

    #[test]
    #[ignore = "drives the real claim route over the live corpus; run with --release --ignored"]
    fn a_module_two_roots_declare_refuses_under_every_selector_and_one_file_is_one_member() {
        let package = format!("zz_claim_route_dup_{}", std::process::id());
        let child = format!("test.claim.{package}.child");
        let body = format!("module {child}\n\ntest fn zz_child() -> Bool {{\n  true\n}}\n");
        let _fixture = GovernedClaimPackage::new(&package, &[("child.dag", &body)]);
        let governed_path = format!("dag/test/claim/{package}/child.dag");
        let custom = scratch_root(
            &format!("dup_{}", std::process::id()),
            &[("child.dag", &body)],
        );
        let custom_path = format!(
            "target/gunbc_claim_route_contract_dup_{}/test/claim/child.dag",
            std::process::id()
        );
        let subtree = format!("test.claim.{package}");
        let prefix = format!("{subtree}.");
        let selected = |m: &str| m.starts_with(&prefix);
        let both_roots = vec!["dag".to_string(), "src/v2".to_string(), custom.clone()];
        for (spelling, scope) in [
            ("subtree", module_scope(&subtree, true)),
            (":all", module_scope(&child, false)),
        ] {
            let refused = route_over(&both_roots, &[scope], &selected);
            let cause = refused
                .as_ref()
                .err()
                .unwrap_or_else(|| panic!("{spelling} must refuse the collision: {refused:?}"));
            assert!(
                cause.contains(&format!("module {child} is declared in both"))
                    && cause.contains(&governed_path)
                    && cause.contains(&custom_path),
                "{spelling} must name both files: {cause}"
            );
        }
        let _ = std::fs::remove_dir_all(&custom);
        let governed_only = vec!["dag".to_string(), "src/v2".to_string()];
        let members = route_over(
            &governed_only,
            &[module_scope(&subtree, true), module_scope(&child, false)],
            &selected,
        )
        .expect("overlapping selectors over one file are a collection, not a refusal");
        assert_eq!(labels_of(&members), vec![format!("{child}:zz_child")]);
    }
}

#[cfg(test)]
mod native_route_termination_tests {
    use super::*;

    /// THE CONTROL FOR THE NATIVE ROUTE OF THE GENERATOR SWEEP. The instrument's only producer is
    /// `TargetProducer::NativeClaimProgram` -- emit-by-seed + cargo-build + exec, the same authority
    /// instrument labels travel. If the row is removed, retargeted at any interpreter route
    /// (SelfHost/Interpreted/RetainedHost) or downgraded to a wrapper, this test goes red: the
    /// sweep's native route is the claim, and a fallback in the registry would silently run the
    /// interpreter instead of the emitted program while still printing rows.
    #[test]
    fn roster_import_sweep_native_is_bound_to_the_native_claim_program_producer() {
        let label = instrument_label("roster-import-sweep-native");
        let row = instrument_registry()
            .into_iter()
            .find(|(l, _)| l == &label)
            .unwrap_or_else(|| panic!("roster-import-sweep-native lost its registry row"));
        assert!(
            matches!(
                row.1,
                TargetProducer::NativeClaimProgram {
                    entry: "dag/gunbc/instruments/roster_import_sweep_native.dag"
                }
            ),
            "roster-import-sweep-native must run through NativeClaimProgramProducer, found a fallback: {:?}",
            row.1
        );
    }

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

    /// `v2.workflow.native_route_member_termination` `native_route_termination_under_qualification`, mirrored here and
    /// given its own red. The `.dag` witness pins the fold; this pins that THIS host applies it.
    ///
    /// A REFUSED QUALIFICATION CANNOT YIELD A PASS (review 70107). Several qualification clauses
    /// are what establish the verdict surface is trustworthy at all, so a pass read off rows with
    /// no established provenance is fabricated plausible output.
    #[test]
    fn a_refused_qualification_clamps_a_pass_to_unreached() {
        let out = native_member_outcome(
            "//v2/test/claim/parse:all",
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
            "//v2/test/claim/parse:all",
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
            "//v2/test/claim/parse:all",
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
        match assess_binary_freshness(&obs(vec![
            BinaryInput::NotNewer("a".into()),
            BinaryInput::DiffersFromThisCheckout("/w/here/b".into()),
            BinaryInput::Newer("c".into()),
        ])) {
            BinaryFreshness::Stale { input, why, .. } => {
                assert_eq!(input, "/w/here/b");
                assert!(why.contains("another checkout"), "{why}");
            }
            other => panic!("{other:?}"),
        }
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
        assert!(elsewhere.contains(&BinaryInput::DiffersFromThisCheckout(
            dir.join("other/old file.rs").display().to_string()
        )));
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

    #[test]
    fn a_binary_built_in_another_checkout_is_judged_by_content() {
        let dir = std::env::temp_dir().join(format!("freshness-content-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let checkout = |name: &str, lib: &str| {
            let root = dir.join(name);
            std::fs::create_dir_all(root.join("src/v1/stage0/src")).unwrap();
            std::fs::write(root.join("src/v1/stage0/Cargo.toml"), "[package]").unwrap();
            std::fs::write(root.join("src/v1/stage0/src/lib.rs"), lib).unwrap();
            root
        };
        let built = checkout("built", "fn a() {}");
        let registry = dir.join("registry/dep.rs");
        std::fs::create_dir_all(registry.parent().unwrap()).unwrap();
        std::fs::write(&registry, "dep").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(20));
        let built_at = std::time::SystemTime::now();
        let same = checkout("same", "fn a() {}");
        let edited = checkout("edited", "fn a() { edited() }");
        let inputs = vec![
            built.join("src/v1/stage0/Cargo.toml").display().to_string(),
            built.join("src/v1/stage0/src").display().to_string(),
            built.join("src/v1/stage0/src/lib.rs").display().to_string(),
            registry.display().to_string(),
        ];
        let verdict = |workspace: &std::path::Path| {
            assess_binary_freshness(&BinaryInputObservation::Observed {
                binary: "bin".into(),
                inputs: classify_binary_inputs(&inputs, workspace, built_at).unwrap(),
            })
        };
        assert!(matches!(
            verdict(&same),
            BinaryFreshness::Fresh { inputs: 4, .. }
        ));
        match verdict(&edited) {
            BinaryFreshness::Stale { input, .. } => {
                assert_eq!(
                    input,
                    edited.join("src/v1/stage0/src/lib.rs").display().to_string()
                )
            }
            other => panic!("{other:?}"),
        }
        let added = checkout("added", "fn a() {}");
        std::fs::write(added.join("src/v1/stage0/src/extra.rs"), "").unwrap();
        match verdict(&added) {
            BinaryFreshness::Stale { input, .. } => {
                assert_eq!(
                    input,
                    added.join("src/v1/stage0/src/extra.rs").display().to_string()
                )
            }
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            verdict(&dir.join("absent")),
            BinaryFreshness::Stale { .. }
        ));
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(built.join("src/v1/stage0/src/lib.rs"), "fn a() {}").unwrap();
        match verdict(&same) {
            BinaryFreshness::Stale { input, why, .. } => {
                assert_eq!(
                    input,
                    built.join("src/v1/stage0/src/lib.rs").display().to_string()
                );
                assert!(why.contains("changed after"), "{why}");
            }
            other => panic!("{other:?}"),
        }
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

fn run_self_host_frontier(source_roots: &[String], stage0_crate_dir: &str) -> InvocationOutcome {
    match cli_run::run_self_host_frontier(source_roots, stage0_crate_dir) {
        Ok(code) => InvocationOutcome {
            termination: match code {
                0 => Termination::ObservationHeld,
                1 => Termination::ObservationDidNotHold,
                _ => Termination::SubjectUnreached,
            },
            message: format!(
                "self-host-frontier: exit {code}; the [self-host-frontier] lines above are the derived frontier"
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
    // NOT a heal round: an instrument prices a round on the checkout it runs on, so it asks for
    // no base mirror and its receipt renders no base-mirror line. Which authority a round seeds
    // from is decided by the CALLER's context, never by whether something failed.
    let round = match cli_run::run_regen_round_cost(
        "target/stage0-regen-candidate",
        "target/stage0-regen-receipt.json",
        &roots,
        false,
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

#[cfg(test)]
mod registry_rows_identity_tests {
    use super::*;
    use crate::v1_interpreter::Value;
    use std::collections::BTreeMap;

    /// One row as the identity join reads it: producer variant name, entry and function.
    type RowReading = (String, Option<String>, Option<String>);

    fn text(
        ctx: &crate::v1_interpreter::InterpContext,
        fields: &[(crate::v1_interpreter::Symbol, Value)],
        name: &str,
    ) -> Option<String> {
        match ctx.field(fields, name) {
            Some(Value::Str(s)) => Some(s.to_string()),
            _ => None,
        }
    }

    fn row_label(ctx: &crate::v1_interpreter::InterpContext, target: &Value) -> String {
        let Value::Record { fields, .. } = target else {
            panic!("binding target is not a Label")
        };
        let Some(Value::Record { fields: name, .. }) = ctx.field(fields, "target") else {
            panic!("Label.target missing")
        };
        let target_name = text(ctx, name, "name").expect("TargetName.name");
        let package = match ctx.field(fields, "package") {
            Some(Value::Variant {
                variant_name,
                fields,
                ..
            }) if ctx.sym_eq(*variant_name, "PackagePath") => {
                let Some(Value::List(segments)) = ctx.field(fields, "segments") else {
                    panic!("PackagePath.segments missing")
                };
                segments
                    .iter()
                    .map(|s| match s {
                        Value::Str(s) => s.to_string(),
                        other => panic!("segment {other:?}"),
                    })
                    .collect::<Vec<_>>()
                    .join("/")
            }
            _ => String::new(),
        };
        format!("//{package}:{target_name}")
    }

    /// The identity and mutation controls share one observation of the declared rows.
    fn read_rows() -> &'static BTreeMap<String, RowReading> {
        static ROWS: std::sync::OnceLock<BTreeMap<String, RowReading>> = std::sync::OnceLock::new();
        ROWS.get_or_init(evaluate_rows)
    }

    fn evaluate_rows() -> BTreeMap<String, RowReading> {
        let reader = cli_run::workspace_root().join("dag/gunbc/instrument_targets.dag");
        let roots = cli_run::default_source_roots();
        let (_, ctx) = cli_run::load_entry_for_functions(
            &roots,
            reader.to_str().expect("utf8 path"),
            &["instrument_bindings".to_string()],
            crate::v1_interpreter::ExecutionMode::Wet,
        )
        .expect("resolve instrument_bindings");
        let value =
            crate::v1_interpreter::run_in_context_with_args(&ctx, "instrument_bindings", &[], true)
                .expect("instrument_bindings");
        let Value::List(bindings) = &value else {
            panic!("instrument_bindings is not a List")
        };
        let mut rows = BTreeMap::new();
        for binding in bindings.iter() {
            let Value::Record { fields, .. } = binding else {
                panic!("binding is not a record")
            };
            let label = row_label(&ctx, ctx.field(fields, "target").expect("binding.target"));
            let Some(Value::Variant {
                variant_name,
                fields: producer,
                ..
            }) = ctx.field(fields, "producer")
            else {
                panic!("{label}: producer is not a variant")
            };
            let (entry, function) = if ctx.sym_eq(*variant_name, "CliWireProgramProducer") {
                let Some(Value::Record { fields: target, .. }) = ctx.field(producer, "target")
                else {
                    panic!("{label}: CliWire target missing")
                };
                let Some(Value::Record {
                    fields: declaration,
                    ..
                }) = ctx.field(target, "declaration")
                else {
                    panic!("{label}: target declaration missing")
                };
                let module = text(&ctx, declaration, "module_path").expect("declaration module");
                let name = text(&ctx, declaration, "decl_name").expect("declaration name");
                let path = cli_run::source_layout_lookup::module_storage_relative(&module)
                    .expect("repository entry path");
                (Some(path), Some(name))
            } else {
                (text(&ctx, producer, "entry"), None)
            };
            let reading = (ctx.resolve(*variant_name), entry, function);
            assert!(
                rows.insert(label.clone(), reading).is_none(),
                "duplicate row {label}"
            );
        }
        rows
    }

    fn registry_reading(producer: &TargetProducer) -> RowReading {
        let debug = format!("{producer:?}");
        let kind = debug
            .split(|c: char| c == ' ' || c == '{')
            .next()
            .unwrap()
            .to_string();
        let (entry, function) = match producer {
            TargetProducer::NativeClaimProgram { entry }
            | TargetProducer::NativeServeProgram { entry } => (Some(entry.to_string()), None),
            TargetProducer::CliWireProgram { entry, function } => {
                (Some(entry.to_string()), Some(function.to_string()))
            }
            _ => (None, None),
        };
        (format!("{kind}Producer"), entry, function)
    }

    /// Every disagreement, each naming its label; empty when the two populations agree by identity.
    fn mismatches(
        registry: &[(Label, TargetProducer)],
        rows: &BTreeMap<String, RowReading>,
    ) -> Vec<String> {
        let mut out = Vec::new();
        let mut seen = std::collections::BTreeSet::new();
        for (label, producer) in registry {
            let name = render_label(label);
            seen.insert(name.clone());
            match rows.get(&name) {
                None => out.push(format!("{name}: in registry, no row")),
                Some(row) if *row != registry_reading(producer) => out.push(format!(
                    "{name}: registry {:?} != row {row:?}",
                    registry_reading(producer)
                )),
                Some(_) => {}
            }
        }
        out.extend(
            rows.keys()
                .filter(|n| !seen.contains(*n))
                .map(|n| format!("{n}: row, not in registry")),
        );
        out
    }

    /// POSITIVE CONTROL: today's registry and rows agree on all labels.
    #[test]
    fn registry_matches_rows() {
        let rows = read_rows();
        let registry = instrument_registry();
        assert_eq!(registry.len(), rows.len(), "label count differs");
        let found = mismatches(&registry, rows);
        assert!(
            found.is_empty(),
            "registry/rows drift:\n{}",
            found.join("\n")
        );
    }

    /// RED: a shared label whose entry (or function, or kind) differs is the silent hazard; the
    /// check must fail and name that label.
    #[test]
    fn a_mutated_registry_entry_fails_by_name() {
        let rows = read_rows();
        let mut registry = instrument_registry();
        let at = registry
            .iter()
            .position(|(_, p)| matches!(p, TargetProducer::CliWireProgram { .. }))
            .expect("a CliWireProgram arm");
        let victim = render_label(&registry[at].0);
        registry[at].1 = TargetProducer::CliWireProgram {
            entry: "dag/gunbc/instruments/mutated.dag",
            function: "mutated",
        };
        let found = mismatches(&registry, rows);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].starts_with(&victim), "{found:?}");
        let at = registry
            .iter()
            .position(|(_, p)| matches!(p, TargetProducer::SelfHost))
            .expect("SelfHost arm");
        registry[at].1 = TargetProducer::V2NativeCli;
        assert_eq!(
            mismatches(&registry, rows).len(),
            2,
            "a kind change must also be named"
        );
    }
}

#[cfg(test)]
mod instrument_registry_generated_falsifier {
    use super::*;

    // Completeness of this table is not asserted here by counting rows. The identity join already lives
    // in this file: `registry_rows_identity_tests::registry_matches_rows` evaluates
    // gunbc.instrument_targets instrument_bindings through the interpreter and joins it against
    // instrument_registry() by label, kind, entry and function in BOTH directions, so a row dropped from
    // the roster, a row added to the roster without one, or a subject rewritten on either side is named
    // by label. A count copied from today's tree would be a change detector, which gunbc.design_argument::fail_closed refuses:
    // "Completeness is an identity join, not a count equality."

    /// CONTROL: every subject the committed table names is present in this checkout. This is the leg
    /// that a stale table fails -- it is the shape of PR 15298, where a row named a subject that was
    /// no longer the one the model declared.
    #[test]
    fn every_declared_subject_is_present() {
        for (label, producer) in instrument_registry() {
            if let Some(refusal) = instrument_subject_absence_refusal(&producer, &label.target) {
                panic!(
                    "the committed instrument table names a subject the checkout lacks: {}",
                    invocation_refusal_rendered(&refusal)
                );
            }
        }
    }

    /// RED, AND THE FAST ONE: the gate itself refuses a subject the checkout does not carry. This
    /// leg calls the gate directly so the refusal is observed without a dispatch that, absent the
    /// gate, walks into the interpreter on a path that is gone -- the fail-open behaviour this gate
    /// exists to stop, and the reason the dispatch leg below is the slower one.
    #[test]
    fn the_gate_refuses_an_absent_subject_by_name() {
        let absent = "dag/gunbc/instruments/absent_subject_fixture.dag";
        assert!(!std::path::Path::new(absent).exists());
        let refusal = instrument_subject_absence_refusal(
            &TargetProducer::NativeClaimProgram { entry: absent },
            "native-claim-program",
        )
        .expect("an absent subject must be refused, not dispatched");
        let rendered = invocation_refusal_rendered(&refusal);
        assert!(
            rendered.contains("instrument native-claim-program")
                && rendered.contains("names entry ")
                && rendered.contains(absent)
                && rendered.contains("absent from this checkout"),
            "the refusal must name the instrument, the field and the path: {rendered}"
        );
        assert!(
            rendered.contains("instrument_registry_generated.rs"),
            "the refusal must name the file to regenerate: {rendered}"
        );
    }

    /// RED: a generated row whose subject the checkout lacks refuses, typed and located, at exit 2,
    /// and never reaches the subject. Without the check the host would evaluate a path that is gone
    /// and report on it.
    #[test]
    fn a_row_naming_an_absent_subject_refuses_at_the_instrument_exit_two() {
        let absent = "dag/gunbc/instruments/absent_subject_fixture.dag";
        assert!(
            !std::path::Path::new(absent).exists(),
            "the fixture path must stay absent for this test to mean anything"
        );
        let cases = vec![
            (
                TargetProducer::NativeClaimProgram { entry: absent },
                "native-claim-program",
            ),
            (
                TargetProducer::NativeServeProgram { entry: absent },
                "native-serve-startup-refusal",
            ),
            (
                TargetProducer::CliWireProgram {
                    entry: absent,
                    function: "a_function",
                },
                "roster-import-sweep-controls",
            ),
        ];
        for (producer, label) in cases {
            let outcome = run_producer(producer, label);
            assert_eq!(
                outcome.termination,
                Termination::Refused,
                "an absent subject must refuse, not run: {label}"
            );
            assert!(
                outcome
                    .message
                    .contains("which is absent from this checkout")
                    && outcome.message.contains(absent)
                    && outcome.message.contains(&format!("instrument {label}")),
                "the refusal must name the instrument, the field and the path: {}",
                outcome.message
            );
            assert_eq!(
                invocation_exit_status(outcome.termination),
                2,
                "the instrument contract refuses at exit 2"
            );
        }
    }
}

#[cfg(test)]
mod test_batch_admission_controls {
    use super::*;

    /// One request, as the CLI would hand it: nothing spelled but the operands.
    fn request(operands: &[&str]) -> TestInvocationOptions {
        TestInvocationOptions {
            operands: operands.iter().map(|o| o.to_string()).collect(),
            hermetic: false,
            wet: false,
            record: false,
            fixture_store: None,
            source_roots: Vec::new(),
        }
    }

    fn universes() -> (TargetPattern, TargetPattern) {
        (
            native_route_default_pattern().expect("the seed build must hand over its boundary"),
            claim_route_universe(),
        )
    }

    const CANARY: &str =
        "//test/manual/roadmap_launch_canary_oracle:roadmap_launch_canary_oracle_completed";

    fn classified(operands: &[&str]) -> TestOperandClasses {
        let (native, claim) = universes();
        let operands: Vec<String> = operands.iter().map(|o| o.to_string()).collect();
        match classify_test_operands(&operands, &native, &claim) {
            Ok(classes) => classes,
            Err(refused) => panic!("the fold refused: {}", refused.message),
        }
    }

    /// `test.claim.target_invocation_exact_claim_admission_witness` `an_unbound_exact_label_whose_module_resolves_is_a_claim`,
    /// on the mirror: only the canary's module resolves, so the typo beside it stays exact.
    #[test]
    fn an_unbound_exact_label_whose_module_resolves_is_a_claim() {
        let routed = admit_resolvable_exact_targets(
            classified(&[CANARY, "//test/manual/roadmap_launch_canary_oracle_typo:x"]),
            &|_| false,
            &|label: &Label| render_label(label) == CANARY,
        );
        assert_eq!(routed.exact.len(), 1);
        assert_eq!(
            render_label(&routed.exact[0]),
            "//test/manual/roadmap_launch_canary_oracle_typo:x"
        );
        assert_eq!(routed.claim.len(), 1);
        assert!(
            !routed.is_claim_selection(),
            "a batch holding an unknown exact stays mixed"
        );
    }

    /// Review 98733: a corpus that does not index leaves "is this label a claim?" unanswered, so the
    /// run is the subject unreached carrying the index's cause -- never `TargetIsUnknown`.
    #[test]
    fn an_unindexable_corpus_is_the_subject_unreached_not_an_unknown_target() {
        let root = scratch_claim_root(
            "unindexable_exact_label",
            &[("headerless.dag", "fn not_a_module() -> Bool { true }\n")],
        );
        let mut options = request(&[CANARY]);
        options.source_roots = vec![root];
        let out = test_verb(&options);
        assert_eq!(
            out.termination,
            Termination::SubjectUnreached,
            "{}",
            out.message
        );
        assert!(out.message.contains("did not index"), "{}", out.message);
        assert!(!out.message.contains("unknown"), "{}", out.message);
    }

    #[test]
    fn an_unbound_exact_label_whose_module_does_not_resolve_stays_unknown() {
        let routed = admit_resolvable_exact_targets(classified(&[CANARY]), &|_| false, &|_| false);
        assert_eq!((routed.exact.len(), routed.claim.len()), (1, 0));
    }

    #[test]
    fn a_registry_bound_label_keeps_its_producer_even_when_a_module_resolves() {
        let routed = admit_resolvable_exact_targets(
            classified(&["//gunbc/instruments:self-host"]),
            &instrument_registry_binds,
            &|_| true,
        );
        assert_eq!((routed.exact.len(), routed.claim.len()), (1, 0));
    }

    /// The fold, as `test_verb` uses it. `InvocationOutcome` carries no `Debug`, so a refusal is reported
    /// through the message the caller would have read rather than through a derived dump.
    fn admitted(operands: &[&str]) -> TestOperandClasses {
        let (native_universe, claim_universe) = universes();
        classify_test_operands(
            &operands
                .iter()
                .map(|o| o.to_string())
                .collect::<Vec<String>>(),
            &native_universe,
            &claim_universe,
        )
        .unwrap_or_else(|refused| panic!("the batch was refused: {}", refused.message))
    }

    fn refused(operands: &[&str]) -> InvocationOutcome {
        let (native_universe, claim_universe) = universes();
        match classify_test_operands(
            &operands
                .iter()
                .map(|o| o.to_string())
                .collect::<Vec<String>>(),
            &native_universe,
            &claim_universe,
        ) {
            Ok(classes) => panic!(
                "the batch was admitted; it had to be refused (selected {} operand(s))",
                classes.selected
            ),
            Err(refused) => refused,
        }
    }

    /// THE DEFAULT IS A GRANT, NOT A REFUSAL. `gunbc test //test/claim/x:y` is today's contributor and
    /// remote-build test command; an admission that demanded a spelled mode would break every one of them,
    /// so an unspelled mode must reach `execution_mode_for` and come back Hermetic -- the same answer the
    /// claim path gets today.
    #[test]
    fn an_unspelled_mode_grants_hermetic() {
        let request = request(&["//test/claim/github_pull_request_decode_witness:the_recorded_pull_request_body_decodes_at_all"]);
        assert_eq!(request.spelled_mode_flag(), None);
        let granted = request
            .admitted_execution_mode()
            .expect("no mode spelled is not a refusal");
        assert_eq!(granted, None);
        assert_eq!(
            cli_run::execution_mode_for(granted, true, false),
            crate::v1_interpreter::ExecutionMode::Hermetic
        );
    }

    /// THE THREE ARMS OF `std.execution_mode` `ExecutionMode`, each reached by the option that names it.
    /// Record is an arm in its own right -- effects run AND fixtures are written -- which is why it is a
    /// mode flag and not a modifier on another mode.
    #[test]
    fn each_mode_flag_grants_the_mode_it_names() {
        let hermetic = TestInvocationOptions {
            hermetic: true,
            ..request(&["//test/claim/x:y"])
        };
        assert_eq!(
            cli_run::execution_mode_for(
                hermetic
                    .admitted_execution_mode()
                    .expect("--hermetic is admissible"),
                true,
                false
            ),
            crate::v1_interpreter::ExecutionMode::Hermetic
        );

        let wet = TestInvocationOptions {
            wet: true,
            ..request(&["//test/claim/x:y"])
        };
        assert_eq!(
            cli_run::execution_mode_for(
                wet.admitted_execution_mode().expect("--wet is admissible"),
                true,
                false
            ),
            crate::v1_interpreter::ExecutionMode::Wet
        );

        let record = TestInvocationOptions {
            record: true,
            fixture_store: Some("/tmp/keen-fox-230-fixture-store".to_string()),
            ..request(&["//test/claim/x:y"])
        };
        assert_eq!(
            cli_run::execution_mode_for(
                record
                    .admitted_execution_mode()
                    .expect("--record with a store is admissible"),
                true,
                false
            ),
            crate::v1_interpreter::ExecutionMode::Record
        );
    }

    /// `--record --wet` IS A CONFLICT, NOT A COMBINATION, and the refusal names both flags as typed: a
    /// reader who typed two needs to know which two. Under the alternative -- last flag wins -- the same
    /// argv would silently run a different mode from the one the caller asked for.
    #[test]
    fn two_mode_flags_refuse_naming_both() {
        let request = TestInvocationOptions {
            record: true,
            wet: true,
            fixture_store: Some("/tmp/keen-fox-230-fixture-store".to_string()),
            ..request(&["//test/claim/x:y"])
        };
        let reason = request
            .admitted_execution_mode()
            .expect_err("two mode flags are one batch's mode");
        assert!(
            reason.contains("--wet") && reason.contains("--record"),
            "{reason}"
        );
    }

    /// `--record` WITH NOWHERE TO WRITE REFUSES. A recorded batch whose fixtures go nowhere is a batch that
    /// reports success while discarding the evidence it exists to produce.
    #[test]
    fn record_without_a_fixture_store_refuses() {
        let request = TestInvocationOptions {
            record: true,
            ..request(&["//test/claim/x:y"])
        };
        let reason = request
            .admitted_execution_mode()
            .expect_err("--record must name a store");
        assert!(reason.contains("--fixture-store"), "{reason}");
    }

    /// A FIXTURE STORE NAMES THE MODE IT BELONGS TO: a bare `--fixture-store` does not default a mode, it
    /// is refused. Defaulting one would let a caller point at a store and get effects (or none) they never
    /// asked for.
    #[test]
    fn a_fixture_store_without_a_mode_refuses() {
        let request = TestInvocationOptions {
            fixture_store: Some("/tmp/keen-fox-230-fixture-store".to_string()),
            ..request(&["//test/claim/x:y"])
        };
        let reason = request
            .admitted_execution_mode()
            .expect_err("a store must name the mode it belongs to");
        assert!(
            reason.contains("--record") && reason.contains("--hermetic"),
            "{reason}"
        );
    }

    /// `--hermetic --fixture-store` IS GRANTED: replay from a store with no effects is a coherent request,
    /// and refusing it would leave recorded fixtures with no reader.
    #[test]
    fn hermetic_with_a_fixture_store_grants() {
        let request = TestInvocationOptions {
            hermetic: true,
            fixture_store: Some("/tmp/keen-fox-230-fixture-store".to_string()),
            ..request(&["//test/claim/x:y"])
        };
        let granted = request
            .admitted_execution_mode()
            .expect("--hermetic with a store is one mode plus its argument");
        assert_eq!(
            cli_run::execution_mode_for(granted, true, false),
            crate::v1_interpreter::ExecutionMode::Hermetic
        );
        assert!(cli_run::claim_fixture_store_rc(&request.fixture_store_path()).is_some());
    }

    /// THE REPLACE-NOT-APPEND RULE, READABLE IN THE GRANTED VALUE. `--source-root` REPLACES the default
    /// pair: what is granted here is what the route will read, and a granted value still carrying a default
    /// would mean the option was additive -- which is the case where a caller's extra root silently coexists
    /// with a corpus it did not ask for. The control the ruling asked for: give one root, assert the
    /// defaults are absent from what the route receives.
    #[test]
    fn source_roots_replace_the_defaults_rather_than_joining_them() {
        let defaults = cli_run::witness_layer_roots();
        // A root the walk could read: walkability is refused, so the control has to hand over a real
        // directory -- and only ONE, which is the point of the assertion below it.
        let given_root = std::env::temp_dir().join("keen-fox-230-given-root");
        std::fs::create_dir_all(&given_root).expect("a readable root for the control");
        let given = TestInvocationOptions {
            source_roots: vec![given_root.to_string_lossy().into_owned()],
            ..request(&["//test/claim/x:y"])
        };
        let granted = given
            .admitted_source_roots(&defaults)
            .expect("one given root is admissible");
        assert_eq!(granted, vec![given_root.to_string_lossy().into_owned()]);
        for default in &defaults {
            assert!(
                !granted.contains(default),
                "a default root survived an option that replaces the corpus: {default}"
            );
        }
        // And with no option the defaults are what the route reads: setting the corpus is not what the option is for.
        assert_eq!(
            request(&["//test/claim/x:y"])
                .admitted_source_roots(&defaults)
                .expect("no option is admissible"),
            defaults
        );
    }

    /// AN UNREADABLE ROOT REFUSES AT ADMISSION. A root that is absent, or is a file, or cannot be listed
    /// selects no claims, and the batch would then report the true sentence "the pattern selected no
    /// discovered claim" about a subject it never read -- which is how a typo becomes a passing run.
    #[test]
    fn an_unreadable_source_root_refuses_with_its_reason() {
        let missing = TestInvocationOptions {
            source_roots: vec!["/tmp/keen-fox-230-no-such-root".to_string()],
            ..request(&["//test/claim/x:y"])
        };
        let reason = missing
            .admitted_source_roots(&cli_run::witness_layer_roots())
            .expect_err("an absent root is not an empty corpus");
        assert!(
            reason.contains("--source-root") && reason.contains("no such path"),
            "{reason}"
        );

        let file = std::env::temp_dir().join("keen-fox-230-not-a-root");
        std::fs::write(&file, "not a directory").expect("a file to name as a root");
        let not_a_dir = TestInvocationOptions {
            source_roots: vec![file.to_string_lossy().into_owned()],
            ..request(&["//test/claim/x:y"])
        };
        let reason = not_a_dir
            .admitted_source_roots(&cli_run::witness_layer_roots())
            .expect_err("a file is not a root");
        assert!(reason.contains("not a directory"), "{reason}");
    }

    /// A ROOT WITH NO VALUE IS NOT AN EMPTY CORPUS: it refuses, because a walk of "" is a walk of nothing,
    /// and a batch that selected no claims from it would read like an empty selection.
    #[test]
    fn a_source_root_without_a_value_refuses() {
        let request = TestInvocationOptions {
            source_roots: vec!["  ".to_string()],
            ..request(&["//test/claim/x:y"])
        };
        let reason = request
            .admitted_source_roots(&cli_run::witness_layer_roots())
            .expect_err("a root needs a value");
        assert!(reason.contains("--source-root"), "{reason}");
    }

    /// AN OPTION ON A ROUTE THAT CANNOT RECEIVE IT REFUSES, PER ROUTE. The native route builds under the
    /// native compiler's authority and the registry runs one bound producer: neither evaluates a claim, so
    /// there is no evaluation for a mode or a corpus to apply to. Dropping the option silently would let a
    /// caller read a hermetic run as a wet one.
    #[test]
    fn a_claim_route_option_on_another_route_refuses() {
        let native_operand = render_target_pattern(&universes().0);
        let cases: Vec<(&str, &str)> = vec![
            ("native", native_operand.as_str()),
            ("registry", "//gunbc/instruments/registry-absent-subject:x"),
        ];
        for (route, operand) in cases {
            let classes = admitted(&[operand]);
            let request = TestInvocationOptions {
                wet: true,
                ..request(&[operand])
            };
            let refused = claim_route_option_refusal(&classes, &request).unwrap_or_else(|| {
                panic!("{route} batch must refuse --wet; it cannot be honoured there")
            });
            assert_eq!(refused.termination, Termination::Refused);
            assert!(refused.message.contains("--wet"), "{}", refused.message);
            assert!(
                refused.message.contains("claim-route option"),
                "{}",
                refused.message
            );
        }
    }

    /// ONE INVOCATION RUNS ONE ROUTE, and the refusal counts what was mixed so the remedy is visible: a
    /// mixed batch would have two result rules, and an invocation whose exit status came from both would not
    /// mean anything.
    #[test]
    fn a_mixed_batch_refuses_naming_the_counts() {
        let native_operand = render_target_pattern(&universes().0);
        let classes = admitted(&[native_operand.as_str(), "//test/claim/x:y"]);
        assert_eq!(classes.native.len(), 1);
        assert_eq!(classes.claim.len(), 1);
        let refused = mixed_batch_refusal_if_mixed(&classes).expect("a mixed batch refuses");
        assert_eq!(refused.termination, Termination::Refused);
        assert!(
            refused.message.contains("1 native") && refused.message.contains("1 claim"),
            "{}",
            refused.message
        );
    }

    /// AN OVERLAP SELECTS ONCE, at the fold: the same operand text twice is one selection, so a caller that
    /// builds an argv by concatenation cannot run a claim twice and read two outcomes for one identity. Two
    /// DISTINCT exact labels plus one of them repeated select two -- the dedup is on operand text, so the
    /// second selection is another exact label (a package-wide `:all` is no longer an admissible claim
    /// operand to test with; see `a_package_wide_all_refuses_on_the_claim_route`).
    #[test]
    fn a_repeated_operand_selects_once() {
        let classes = admitted(&[
            "//test/claim/x:y",
            "//test/claim/x:y",
            "//test/claim/two:fn",
        ]);
        assert_eq!(
            classes.selected, 2,
            "the duplicate operand must not select twice"
        );
        assert_eq!(classes.claim.len(), 2);
    }

    #[test]
    fn a_package_wide_all_selects_the_package_on_the_claim_route() {
        for operand in ["//test/claim/x:all", "//test/claim/x:*"] {
            let all = admitted(&[operand]);
            assert_eq!(all.claim.len(), 1, "{operand}");
            assert_eq!(
                all.claim[0],
                TargetPattern::PackageTargets(vec![
                    "test".to_string(),
                    "claim".to_string(),
                    "x".to_string()
                ]),
                "{operand}"
            );
        }
        let subtree = admitted(&["//test/claim/x/..."]);
        assert_eq!(subtree.claim.len(), 1);
    }

    #[test]
    fn an_empty_single_label_selection_names_the_module_form() {
        let bare = parse_target_pattern("//test/claim/a/b").expect("a bare label parses");
        let note = claim_route_empty_selection_note(&[bare]);
        assert!(note.contains("selected no discovered claim"), "{note}");
        assert!(note.contains("`//test/claim/a/b:all`"), "{note}");
        let subtree = parse_target_pattern("//test/claim/a/b/...").expect("a subtree parses");
        let plain = claim_route_empty_selection_note(&[subtree]);
        assert_eq!(plain, " — the pattern selected no discovered claim");
    }

    /// A SET FORM OUTSIDE EVERY UNIVERSE REFUSES RATHER THAN WIDENING: `//not/a/universe/...` names no
    /// population the route can enumerate, and running it as a subtree of the whole corpus is exactly the
    /// widening the plan refuses.
    #[test]
    fn a_set_form_outside_every_universe_refuses() {
        let refused = refused(&["//not/a/universe/..."]);
        assert_eq!(refused.termination, Termination::Refused);
    }

    /// `:all-targets` IS NOT THIS GRAMMAR'S ALL, and the refusal carries the pattern authority's own cause
    /// rather than a batch-shaped paraphrase of it: the remedy for `:all-targets` is `:all`, not the remedy
    /// for a relative label.
    #[test]
    fn all_targets_refuses_with_the_pattern_authoritys_cause() {
        let refused = refused(&["//test/claim/x:all-targets"]);
        assert!(
            refused.message.contains("all") && refused.message.contains("//test/claim/x:all"),
            "{}",
            refused.message
        );
    }

    /// THE EMPTY BATCH: an invocation must name something. `selected == 0` is what the refusal keys on, so
    /// a batch whose operands were all duplicates of one another still selects its one claim.
    #[test]
    fn an_empty_batch_selects_nothing_and_a_duplicated_batch_does_not() {
        assert!(admitted(&[]).selected_is_empty());
        assert!(!admitted(&["//test/claim/x:y", "//test/claim/x:y"]).selected_is_empty());
    }

    /// THE BATCH AT THE ROUTE, ON A SCRATCH CORPUS: three claims under one subtree plus one exact label
    /// INSIDE that subtree, named together in one batch, produce one result per claim with the FULL LABEL as
    /// its key, and no claim twice. Selection is driven by the discovered corpus -- claims are enumerated
    /// from the walk and matched against every pattern -- so an overlap cannot enroll a claim twice. The
    /// label key is dated provenance from before the cutover: the deleted `claim_batch` printed the bare
    /// function name on its verdict line, so two entries declaring one function name printed
    /// indistinguishable lines there; the route keys with the full label.
    ///
    /// The subtree operand is scoped to `test/claim/zz_batch_scratch/...`, a package only this scratch
    /// corpus has, so the batch stays three claims rather than every claim in the corpus: a control that
    /// asked for the whole universe would be a floor run wearing a test name.
    #[test]
    fn one_process_runs_the_named_claims_once_each_and_keys_them_by_label() {
        std::env::set_current_dir(cli_run::workspace_root()).expect("enter workspace");
        let scratch = scratch_claim_root(
            "batch",
            &[
                (
                    "zz_scratch_a.dag",
                    "module test.claim.zz_batch_scratch.a\n\ntest fn zz_a1() -> Bool {\n  true\n}\n\ntest fn zz_a2() -> Bool {\n  true\n}\n",
                ),
                (
                    "zz_scratch_b.dag",
                    "module test.claim.zz_batch_scratch.b\n\ntest fn zz_b1() -> Bool {\n  true\n}\n",
                ),
            ],
        );
        let mut roots = vec![scratch];
        roots.extend(cli_run::witness_layer_roots());
        let (native_universe, claim_universe) = universes();
        let operands = vec![
            "//test/claim/zz_batch_scratch/a:zz_a1".to_string(),
            "//test/claim/zz_batch_scratch/...".to_string(),
        ];
        let classes = match classify_test_operands(&operands, &native_universe, &claim_universe) {
            Ok(classes) => classes,
            Err(refused) => panic!(
                "an exact label and a subtree over it are one batch: {}",
                refused.message
            ),
        };
        assert_eq!(classes.claim.len(), 2, "both operands are claim patterns");
        let patterns = classes.claim.clone();
        // THE BATCH'S OWN SELECTION, not a copy: run_claim_selection builds these two closures from the
        // patterns, so the control drives the route the way the verb does.
        let module_selected = |module_path: &str| {
            let segments: Vec<String> = module_path.split('.').map(str::to_string).collect();
            patterns.iter().any(|pattern| match pattern {
                TargetPattern::SingleTarget(label) => label.package_segments == segments,
                TargetPattern::PackageTargets(_) | TargetPattern::SubtreeTargets(_) => {
                    target_pattern_within(&TargetPattern::PackageTargets(segments.clone()), pattern)
                }
            })
        };
        let claim_selected = |module_path: &str, function: &str| {
            let label = Label {
                package_segments: module_path.split('.').map(str::to_string).collect(),
                target: function.to_string(),
            };
            patterns.iter().any(|pattern| {
                target_pattern_within(&TargetPattern::SingleTarget(label.clone()), pattern)
            })
        };
        let members = match cli_run::run_claim_route(
            &roots,
            &claim_selection_scopes(&patterns),
            &module_selected,
            &claim_selected,
            crate::v1_interpreter::ExecutionMode::Hermetic,
            None,
        ) {
            Ok(members) => members,
            Err(cause) => panic!("the scratch corpus must resolve, not refuse: {cause}"),
        };
        let fold = claim_route_fold(&members);
        assert!(
            fold.duplicate.is_none(),
            "one identity carried more than one outcome"
        );
        let mut labels: Vec<String> = members
            .iter()
            .map(|member| {
                render_label(&Label {
                    package_segments: member.module_path.split('.').map(str::to_string).collect(),
                    target: member.function.clone(),
                })
            })
            .collect();
        labels.sort();
        assert_eq!(
            labels,
            vec![
                "//test/claim/zz_batch_scratch/a:zz_a1".to_string(),
                "//test/claim/zz_batch_scratch/a:zz_a2".to_string(),
                "//test/claim/zz_batch_scratch/b:zz_b1".to_string(),
            ],
            "the batch must run exactly the claims its operands denote, each once, keyed by full label"
        );
        assert!(
            fold.lines
                .iter()
                .any(|line| line.starts_with("//test/claim/zz_batch_scratch/a:zz_a1 HELD")),
            "the line is keyed on the full label: {:?}",
            fold.lines
        );
    }
}
