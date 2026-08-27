// THE HOST REALIZATION OF `gunbc test <label>`, AND IT IS A HAND MIRROR OF A `.dag` AUTHORITY.
//
// The model is `gunbc.target_invocation` (the generic route and the termination vocabulary),
// `gunbc.instrument_targets` (the live target and binding rows, and the differential's own
// classifier and rendering) and `extdeps.bazel.label` (the label grammar this admission mirrors).
// None of those three modules is in the v1 seed's emitted closure -- `src/gunbc_cli_dispatch_surface.rs`
// is the only `gunbc.*` mirror the emitter produces -- so this file is written by hand beside the
// carrier rather than derived from it, exactly as `required_regen_host.rs` mirrors
// `v2.workflow.required_regen` by hand. That makes this seam MITIGATABLE and not structurally
// guaranteed: the two can drift, and the thing that would stop them is the seam being emitted
// rather than authored. The obligation is enrolled in `gunbc.target_invocation_seed_growth`.
//
// WHAT IS AND IS NOT GENERIC HERE. One route runs: argv operand -> admit label -> build the
// registry -> exact lookup -> invoke the bound producer -> render that producer's native standing.
// There is no arm per instrument on that route and none may be added; a second instrument is a row
// in `instrument_registry` plus one `Producer` arm in `run_producer`, which is the peripheral
// realization dispatch DESIGN section 3 keeps out of the interface. What is deliberately NOT here
// is any consultation of `//:required` aggregate policy or of the Blaze status export: both refuse
// instrument producers by design, and reaching for either would answer a question this verb is not
// asking.

use crate::cli_run;

/// The label subset `extdeps.bazel.label` admits, and its refusal vocabulary, mirrored.
///
/// Every family that authority excludes is excluded here with the SAME named cause, because the
/// causes have different remedies: a pattern means "name one target", a relative label means "make
/// it absolute", and collapsing them into one malformed-operand message sends both to the same
/// place. The subset is the main repository only, so there is no field in which a different
/// repository could be written.
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

/// Rendering is a FREE FUNCTION rather than an inherent method, and the placement is not style.
/// `std.decl_ref` `DeclarationRef` can name a whole declaration or a named field and has no
/// spelling for a method on an impl block, so an inherent method is uncitable in the seed-growth
/// roster that must enumerate every item this file adds. A file whose obligations cannot be
/// written down is a file whose obligations do not get discharged.
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

/// The structural inverse of `parse_label`, always writing the explicit colon form. This is the
/// registry key, and it is DERIVED on every call rather than stored beside the label, so a target
/// cannot acquire a second identity that drifts from its first.
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
    // A leading `/`, a trailing `/` and an embedded `//` all produce the same observable -- an
    // empty segment -- so one arm covers three malformed shapes by construction.
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

/// `//my/app/lib` IS `//my/app/lib:lib`: the shorthand is folded at parse time so no consumer
/// downstream ever has to ask which spelling it was handed.
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
}

/// `gunbc.instrument_targets` `instrument_targets` / `instrument_bindings`, as the pairs the
/// index is built from. One entry, deliberately: the seam is generic, so the remaining three
/// instruments are rows rather than a second lane's worth of dispatch.
/// `gunbc.instrument_targets` `heads_reading_differential_source_roots`. The subject is the
/// instrument's own fact and not a CLI option, so an invocation cannot quietly measure a different
/// corpus while reporting this target's standing.
fn heads_reading_differential_source_roots() -> Vec<String> {
    vec!["dag".to_string(), "src/v2".to_string()]
}

fn instrument_registry() -> Vec<(Label, TargetProducer)> {
    vec![(
        Label {
            package_segments: vec!["gunbc".to_string(), "instruments".to_string()],
            target: "heads-reading-differential".to_string(),
        },
        TargetProducer::HeadsReadingDifferential,
    )]
}

/// `gunbc.target_invocation` `TargetInvocationRefusal`, MINUS ONE ARM, and the omission is
/// deliberate rather than an oversight.
///
/// The model keeps a known target with no bound producer apart from an unknown target, because in
/// `gunbc.target_binding` the two populations are separate lists and a target can be registered
/// with nothing realizing it. In THIS realization the registry is a list of PAIRS, so a registered
/// target without a producer has no representation at all -- the state is unwritable rather than
/// checked (DESIGN section 4b, structural impossibility), and carrying an arm nothing can
/// construct would be a decoration that reads as coverage. If the host ever takes its two
/// populations separately, the arm comes back with the state that makes it reachable.
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
/// `ObservationDidNotHold`: an instrument handed a root that does not exist observed nothing, and
/// reporting that as a defect would announce a finding where no reading was taken.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Termination {
    ObservationHeld,
    ObservationDidNotHold,
    SubjectUnreached,
    Refused,
}

/// Three statuses rather than two: 1 is an observation that did not hold, 2 is the absence of an
/// observation, and a caller that cannot tell them apart has been handed the absorbing answer
/// DESIGN section 5 forbids. Wildcard-free: a fifth termination must fail to compile here rather
/// than inherit whichever status a `_` happened to name.
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
/// The producer executes against an UNPINNED LIVE WORKING TREE. Nothing binds the bytes it read to
/// the answer it returned: the corpus may change while the walk is in progress, so the population
/// reported here is not identified with any source state a later run could reconstruct. Calling
/// this a receipt would assert exactly that binding, so it is not called one.
///
/// NO HASH FIELD CLOSES THIS AND ONE MUST NOT BE ADDED. Hashing the tree before the run, after it,
/// or both proves nothing -- the tree can go A -> B -> A while the producer reads a MIXED
/// population, and both hashes agree. The requirement is that the bytes producing the manifest ARE
/// the bytes the producer consumes, which cannot be arranged by observing one mutable namespace and
/// executing against it afterwards. A digest field here would look like evidence and carry none,
/// which is worse than the honest gap.
///
/// SO WHAT IS NOT CLAIMED, named rather than left for a reader to assume safe: this outcome does
/// not support target-result caching, cross-run comparison, remote execution, replay, or the
/// statement "this standing was about source X". Each of those needs source binding first.
pub struct InvocationOutcome {
    pub termination: Termination,
    pub message: String,
}

/// The differential's own standing, rendered in its own vocabulary.
///
/// It does not say PASSED or FAILED and it must not: those are the `//:required` aggregate's
/// words, and this verb is not asking the aggregate's question. `narrowed` is reported and
/// deliberately excluded from the verdict -- it is the declared, bounded scope narrowing, counted
/// rather than absorbed -- so only `divergent` and `regressed` decide whether the reading held.
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
    // `HeadsReadingDifferentialObservation` carries the four populations and no timing, so these
    // two numbers have no home in the standing. They are printed rather than dropped because the
    // route they came from is being deleted in this same change and a replacement that silently
    // loses a capability is not a replacement. They are printed BELOW the populations and take no
    // part in the verdict, which is the split `gunbc.build_target` already states: cost has its own
    // authority and folding it into a correctness answer gives the cost axis a second home. Their
    // next rung is a cost carrier on the observation; until one exists, this is an unmodeled host
    // line and is named as such rather than left to look like modeled output.
    //
    // It measures the PARSE only -- tokenize, newline indexing and per-file setup sit outside both
    // timers -- so it is not the whole `pool_parse` saving and must never be quoted as one.
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
/// is itself realization (DESIGN section 3), so it sits here, at the periphery, and never in the
/// route above or in the CLI surface.
fn run_producer(producer: TargetProducer) -> InvocationOutcome {
    match producer {
        TargetProducer::HeadsReadingDifferential => {
            run_heads_reading_differential(&heads_reading_differential_source_roots())
        }
    }
}

/// THE ONE SEAM: argv operand -> label -> registry -> exact binding -> producer -> native standing.
///
/// The lookup is by the label's own rendering, which is `label_eq` evaluated once per row rather
/// than a first-match scan over a hand-written comparison, and it is EXACT -- there is no prefix
/// match, no suffix match and no "did you mean", because a near miss silently running a different
/// target is worse than a refusal naming the one that was asked for.
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
