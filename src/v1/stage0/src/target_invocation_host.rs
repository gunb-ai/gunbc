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

/// `gunbc.target_binding` `TargetProducer`, narrowed to the members this seam realizes today.
/// Adding one is a row in `instrument_registry` and an arm here; it is not a new route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetProducer {
    SelfHost,
    V2NativeCli,
    V2NativeFrontier,
    EmittedCrateWorkspace,
    HeadsReadingDifferential,
    BehavioralReceiptPlan,
    BehavioralReceiptCensus,
    BehavioralReceiptSelftest,
    CompileCleanDiagnosticCensus,
    EvaluationStoreAddressExactHead,
    TypedGraphExclusiveBytes,
    TypedGraphExclusiveBytesFloorSubject,
    PrimitiveEgressCensus,
    PrimitiveEgressCensusV2,
    PrimitiveEgressCensusDag,
    PrimitiveEgressCensusSeed,
    RequiredLaneResolutionCensus,
    BareReferenceChannelOutcome,
    SelfHostBehavioralEquivalence,
    /// `NativeCensusVerbProducer { verb }`: a whole-corpus census is a verb of the emitted compiler,
    /// so a further census is a registry row naming its verb, never another variant.
    NativeCensusVerb {
        verb: &'static str,
    },
    SelfHostFrontier,
    GenericIdentityCensus,
    InterpolationHoleCensus,
    SharedTypeLeafCollisionCensus,
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
    /// `CliWireProgramProducer { entry, function }`: wet interpreter eval of a CliWireResponse
    /// fold. Generic over entry and function so a second such instrument is a registry row.
    CliWireProgram {
        entry: &'static str,
        function: &'static str,
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

/// Modules this producer evaluates by name at spawn. Parallel to
/// `REQUIRED_FLOOR_RUNTIME_AUTHORITY_MODULES`: listing here is what makes that by-name
/// evaluation legitimate rather than an undeclared reach. Not a floor-subject seed — the floor
/// list must not grow this module's closure.
const NATIVE_CLAIM_RUNTIME_AUTHORITY_MODULES: [&str; 1] = ["gunbc.instrument_targets"];

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
            instrument_label("rfm-rung-census"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/rfm_rung_census.dag",
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
            instrument_label("fleet-convergence-verdict-route"),
            TargetProducer::CliWireProgram {
                entry: "dag/gunbc/instruments/fleet_convergence_verdict_route.dag",
                function: "fleet_convergence_verdict_route_wire",
            },
        ),
        (
            instrument_label("native-allocation-observation"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/native_allocation_observation.dag",
            },
        ),
        (
            instrument_label("native-xl2-rehearsal"),
            TargetProducer::NativeClaimProgram {
                entry: "src/v2/workflow/namespace_xl2_rehearsal_native.dag",
            },
        ),
        (
            instrument_label("brand-mark-assets"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/brand_mark_assets_control.dag",
            },
        ),
        (
            instrument_label("router-proxy-falsifier"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/router_proxy_falsifier.dag",
            },
        ),
        (
            instrument_label("serving-benchmark-run"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/serving_benchmark_run.dag",
            },
        ),
        (
            instrument_label("value-position-block-nests"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/value_position_block_nests.dag",
            },
        ),
        (
            instrument_label("roster-import-sweep-native"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/roster_import_sweep_native.dag",
            },
        ),
        (
            instrument_label("native-serve"),
            TargetProducer::NativeServeProgram {
                entry: "dag/gunbc/instruments/native_serve_fixture.dag",
            },
        ),
        (
            instrument_label("native-serve-device-crypto"),
            TargetProducer::NativeServeProgram {
                entry: "dag/gunbc/auth/approval_device_crypto_serve.dag",
            },
        ),
        (
            instrument_label("native-serve-startup-refusal"),
            TargetProducer::NativeServeProgram {
                entry: "dag/gunbc/instruments/native_serve_startup_refusal_fixture.dag",
            },
        ),
        (
            instrument_label("native-serve-startup-slow"),
            TargetProducer::NativeServeProgram {
                entry: "dag/gunbc/instruments/native_serve_startup_slow_fixture.dag",
            },
        ),
        (
            instrument_label("native-serve-startup-overrun"),
            TargetProducer::NativeServeProgram {
                entry: "dag/gunbc/instruments/native_serve_startup_overrun_fixture.dag",
            },
        ),
        (
            instrument_label("evaluation-store-address-exact-head"),
            TargetProducer::EvaluationStoreAddressExactHead,
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
            TargetProducer::NativeCensusVerb {
                verb: "demand-census",
            },
        ),
        (
            instrument_label("decl-ref-census"),
            TargetProducer::NativeCensusVerb {
                verb: "decl-ref-census",
            },
        ),
        (
            instrument_label("effect-pair-census"),
            TargetProducer::NativeCensusVerb {
                verb: "effect-pair-census",
            },
        ),
        (
            instrument_label("self-host-frontier"),
            TargetProducer::SelfHostFrontier,
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
            instrument_label("shared-type-leaf-collision-census"),
            TargetProducer::SharedTypeLeafCollisionCensus,
        ),
        (
            instrument_label("cardinality-realization"),
            TargetProducer::CardinalityRealization,
        ),
        (
            instrument_label("regen-round-cost"),
            TargetProducer::RegenRoundCost,
        ),
        (
            instrument_label("roster-import-sweep-controls"),
            TargetProducer::CliWireProgram {
                entry: "src/v2/workflow/roster_import_sweep.dag",
                function: "roster_import_sweep_controls_wire",
            },
        ),
        (
            instrument_label("credential-acquisition-systemd-rows"),
            TargetProducer::CliWireProgram {
                entry: "dag/test/claim/dispatch_credential_acquisition_witness.dag",
                function: "credential_acquisition_systemd_rows_wire",
            },
        ),
        (
            instrument_label("append-chain-ladder"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/append_chain_ladder_native.dag",
            },
        ),
        (
            instrument_label("last-use-move"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/last_use_move_native.dag",
            },
        ),
        (
            instrument_label("derived-equality-routes"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/derived_equality_routes_native.dag",
            },
        ),
        (
            instrument_label("cons-tail-recursion"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/cons_tail_recursion_native.dag",
            },
        ),
        (
            instrument_label("member-home-cost"),
            TargetProducer::NativeClaimProgram {
                entry: "dag/gunbc/instruments/member_home_cost_native.dag",
            },
        ),
        (
            instrument_label("claim-coverage-census"),
            TargetProducer::CliWireProgram {
                entry: "dag/gunbc/claim_coverage_census_live.dag",
                function: "claim_coverage_census_exit",
            },
        ),
        // Second home for a binding `gunbc.instrument_targets` `known_red_derivation_binding` already
        // owns: this table is hand-maintained -- `heal` never regenerates it -- so a rostered label with
        // no row here exits 2 'no such target' whatever the .dag registry says. Adding the row is the
        // only way to make the instrument's label reachable, and it follows the table's existing shape
        // (the `claim-coverage-census` row is built the same way).
        // RECEIPT / DEFERRAL for the duplicated mapping: generating this table from
        // `gunbc.instrument_targets` is the repair, and it is a seed lane rather than this instrument's
        // lane because it touches every instrument's binding at once. The hand-maintained table is
        // already under a seed census, so the duplication is tracked rather than invisible: the
        // `DeclarationRef { decl_name: "instrument_registry" }` row inside
        // `gunbc.target_invocation_seed_growth` `target_invocation_seed_growth_justification`.
        (
            instrument_label("known-red-derivation"),
            TargetProducer::CliWireProgram {
                entry: "dag/gunbc/instruments/known_red_derivation.dag",
                function: "known_red_derivation_exit",
            },
        ),
        (
            instrument_label("roster-reach-inhabitance"),
            TargetProducer::CliWireProgram {
                entry: "dag/gunbc/instruments/roster_reach_inhabitance.dag",
                function: "roster_reach_inhabitance_exit",
            },
        ),
        (
            instrument_label("witness-deferral-freeze-population"),
            TargetProducer::CliWireProgram {
                entry: "dag/gunbc/instruments/witness_deferral_freeze_population.dag",
                function: "witness_deferral_freeze_population_exit",
            },
        ),
        (
            instrument_label("compute-run-unit-cold"),
            TargetProducer::CliWireProgram {
                entry: "dag/gunbc/instruments/compute_run_unit_cold.dag",
                function: "compute_run_unit_cold_exit",
            },
        ),
        (
            instrument_label("compute-wrong-capture"),
            TargetProducer::CliWireProgram {
                entry: "dag/gunbc/instruments/compute_wrong_capture.dag",
                function: "compute_wrong_capture_exit",
            },
        ),
        (
            // C5: the dependency-only arm of the C1 instrument. Same entry file, its own entry
            // function -- one body of logic (wrong_capture_run) behind both labels.
            instrument_label("compute-dependency-wrong-capture"),
            TargetProducer::CliWireProgram {
                entry: "dag/gunbc/instruments/compute_wrong_capture.dag",
                function: "compute_dependency_wrong_capture_exit",
            },
        ),
        (
            instrument_label("compute-run-unit-sccache-probe"),
            TargetProducer::CliWireProgram {
                entry: "dag/gunbc/instruments/compute_run_unit_cold.dag",
                function: "compute_run_unit_sccache_probe_exit",
            },
        ),
        (
            instrument_label("compute-run-unit-cold-same-path"),
            TargetProducer::CliWireProgram {
                entry: "dag/gunbc/instruments/compute_run_unit_cold.dag",
                function: "compute_run_unit_cold_same_path_exit",
            },
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
fn run_producer(producer: TargetProducer, label: &str) -> InvocationOutcome {
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
        TargetProducer::NativeClaimProgram { entry } => run_native_claim_program(entry),
        TargetProducer::NativeServeProgram { entry } => run_native_serve_program(entry),
        TargetProducer::CliWireProgram { entry, function } => run_cli_wire_program(entry, function),
        TargetProducer::EmittedCrateWorkspace => {
            run_emitted_crate_workspace(&emitted_crate_workspace_source_roots())
        }
        TargetProducer::EvaluationStoreAddressExactHead => {
            run_evaluation_store_address_exact_head()
        }
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
        TargetProducer::NativeCensusVerb { verb } => {
            run_census_verb(verb, label, &self_host_source_roots())
        }
        TargetProducer::SelfHostFrontier => run_self_host_frontier(
            &self_host_source_roots(),
            &crate::gunbc_stage0_stage0_cargo_manifest::stage0_package_root(),
        ),
        TargetProducer::GenericIdentityCensus => {
            run_generic_identity_census(&self_host_source_roots())
        }
        TargetProducer::InterpolationHoleCensus => {
            run_interpolation_hole_census(&self_host_source_roots())
        }
        TargetProducer::SharedTypeLeafCollisionCensus => {
            run_shared_type_leaf_collision_census(&self_host_source_roots())
        }
        TargetProducer::CardinalityRealization => run_cardinality_realization(),
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

/// Generic CliWireProgramProducer realization: entry and function are the binding's, never a
/// string-match inside NativeClaimProgram.
fn run_cli_wire_program(entry: &'static str, function: &'static str) -> InvocationOutcome {
    run_cli_wire_census(function, entry, function)
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
    // `source_indices` came back from the resolve alongside the graph and was never used; the
    // reader-facing refusal lines now go through the located renderer with it.
    let blocking =
        crate::v1_compiler_compile::interpreter_blocking_diagnostic_messages_with_source_indices(
            graph.diagnostics.clone(),
            source_indices.clone(),
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
    // `source_indices` came back from the resolve alongside the graph and was never used; the
    // reader-facing refusal lines now go through the located renderer with it.
    let blocking =
        crate::v1_compiler_compile::interpreter_blocking_diagnostic_messages_with_source_indices(
            graph.diagnostics.clone(),
            source_indices.clone(),
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
/// further minutes executing the marked `v2` declarations through the emitted binary, which is a
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
    let roots = cli_run::default_source_roots();
    let (graph, source_indices) = match cli_run::resolve_entry_graph(&roots, READER) {
        Ok(resolved) => resolved,
        Err(cause) => {
            return Err(ungrounded(format!("resolve failed for {READER}: {cause}")));
        }
    };
    let ctx = cli_run::make_eval_context(
        graph.as_ref(),
        source_indices,
        crate::v1_interpreter::ExecutionMode::Wet,
    );
    let value = match crate::v1_interpreter::run_in_context_with_args(
        &ctx,
        "native_claim_child_deadline_count",
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
    let roots = v2_native_cli_source_roots();
    let (graph, source_indices) = match cli_run::resolve_entry_graph(&roots, READER) {
        Ok(resolved) => resolved,
        Err(cause) => return unreached(format!("resolve failed for {READER}: {cause}")),
    };
    let ctx = cli_run::make_eval_context(
        graph.as_ref(),
        source_indices,
        crate::v1_interpreter::ExecutionMode::Wet,
    );
    let plan_value =
        match crate::v1_interpreter::run_in_context_with_args(&ctx, "instrument_plan", &[], true) {
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
        "cardinality_realization_standing",
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

fn read_native_serve_binding(entry: &str) -> Result<NativeServeBindingRead, String> {
    const BINDINGS: &str = "dag/gunbc/instrument_targets.dag";
    let roots = cli_run::default_source_roots();
    let (graph, source_indices) = cli_run::resolve_entry_graph(&roots, BINDINGS)
        .map_err(|cause| format!("resolve failed for {BINDINGS}: {cause}"))?;
    let ctx = cli_run::make_eval_context(
        graph.as_ref(),
        source_indices,
        crate::v1_interpreter::ExecutionMode::Wet,
    );
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
    let reader_value = ask("native_serve_probe_reader_for_entry")?;
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
    let declared_value = ask("native_serve_probe_declared_names_for_entry")?;
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
            peer_instance: flag("native_serve_probe_runs_peer")?,
            refused_launch: flag("native_serve_probe_runs_refused_launch")?,
            stuck_ceiling_exit: flag("native_serve_probe_runs_stuck_ceiling_exit")?,
        },
        declared,
    })
}

/// THE NATIVE SERVE PROGRAM PRODUCER: emit, build and start a `NativeServeDriver` entry, exchange
/// the requests the binding row's probe reader names over real TCP, then let that reader decide what
/// the run established. The host decides nothing about a case, and branches on no binding: the
/// reader module and the phases come from the row (`read_native_serve_binding`), the requests and
/// both launch revisions are read from the reader module, the bytes go back to its
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
        "native_serve_probe_phases_refused",
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
            "native_serve_probe_requests",
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
    let plan = (|| -> Result<(String, String, String), String> {
        let release = text(&read("native_serve_probe_release_revision")?)
            .ok_or("the release revision is not a String")?;
        // The refused revision is read only for a row that declares the refused launch; a reader
        // with no such phase owes no such function.
        let refused = if binding.phases.refused_launch {
            text(&read("native_serve_probe_refused_revision")?)
                .ok_or("the refused revision is not a String")?
        } else {
            String::new()
        };
        let deadline = text(&read("native_serve_probe_request_deadline_ms")?)
            .ok_or("the request deadline is not a String")?;
        Ok((release, refused, deadline))
    })();
    let (release, refused, deadline) = match plan {
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
        binding.phases,
        &requests_for_peer_port,
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
            Some("peer_announcement".to_string()),
            crate::v1_interpreter::Value::Str(run.peer_announcement.clone().into()),
        ),
        (
            Some("peer_port".to_string()),
            crate::v1_interpreter::Value::Int(run.peer_port),
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
    message.push_str(&format!(
        "\n--- ROUTED_EQUALITY_RESIDUAL TSV (gunbc.rung_drop derived_equality_residual_field_equality) rows={} ---\nclass\tfile\tposition\tdeclaration\tsubject_type",
        census.routed_equality_residual_rows.len()
    ));
    for row in &census.routed_equality_residual_rows {
        message.push_str(&format!(
            "\n{}\t{}\t{}\t{}\t{}",
            row.class_name, row.file, row.position, row.declaration, row.subject_type,
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
    cli_run::user_work_scope::reexec_into_user_work_slice_if_owed();
    // THE SECOND ACT, BEFORE ANY PRODUCER: bind this process to a kernel memory limit when the
    // invocation asked for one (`GUNBC_BIND_MEMORY_CGROUP_BYTES`, gunbc.host.memory_cgroup_binding --
    // the same bind claim_executor and claim_batch take) at gunbc.emitted_subject_build_gate
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
    let outcome = test_verb_after(operand, &assess_binary_freshness(&observe_binary_inputs()));
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
    // governor's walk is the authority (DESIGN section 3) and reads `max` as unset; the peak locator
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
    if target_pattern_within(&pattern, &native_universe) {
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
        "{label} graph_total={} sum_of_exclusives={} shared_or_unlisted={residual} (gunbc.floor.typed_graph_exclusive_bytes; graph_total from the first run, every class from its own fresh resolve)",
        bytes(ctx.field(fields, "graph_total")),
        bytes(ctx.field(fields, "sum_of_exclusives")),
    ));
    InvocationOutcome {
        termination: Termination::ObservationHeld,
        message: lines.join("\n"),
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
    let mut paths = Vec::new();
    for root in source_roots {
        if let Err(detail) =
            cli_run::collect_dag_files_result(std::path::Path::new(root), &mut paths)
        {
            return unreached(format!("root {root}: {detail}"));
        }
    }
    let mut corpus: Vec<Rc<SourceFile>> = Vec::new();
    for source in &paths {
        let path = source.path().to_string_lossy().to_string();
        match source.read() {
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

/// `gunbc.instrument_targets` `shared_type_leaf_collision_census_label`.
///
/// TRANSPORT: load each measured closure, `compile_to_resolved`, emit, and ask cargo
/// once per subject. Sharing rows and fixture standing are
/// `v1.tests.claim.shared_type_leaf_collision_census`. Independent sharing is
/// `v1.compiler.emit_rust` `maybe_mark_shared_type` per declaration. Standing is the
/// three fixture cargo controls; the live rows are a report (sharing at declaration
/// grain, rustc at subject grain). Completed cargo is a subject-grain finding;
/// NotAttempted / DidNotComplete / DependencyFetchFailed are SubjectUnreached.
fn shared_type_census_subject_build_from_cargo_verdict(
    verdict: &crate::cli_run::emitted_closure_compile_host::CargoVerdict,
) -> Result<
    std::rc::Rc<crate::v1_tests_claim_shared_type_leaf_collision_census::SubjectBuild>,
    String,
> {
    use crate::cli_run::emitted_closure_compile_host::{cargo_verdict_summary, CargoVerdict};
    use crate::v1_tests_claim_shared_type_leaf_collision_census as census;
    match verdict {
        CargoVerdict::Completed { status, .. } if *status == 0 => Ok(census::build_accepted()),
        CargoVerdict::Completed { .. } => Ok(census::build_refused(cargo_verdict_summary(verdict))),
        CargoVerdict::NotAttempted { .. }
        | CargoVerdict::DidNotComplete { .. }
        | CargoVerdict::DependencyFetchFailed { .. } => Err(cargo_verdict_summary(verdict)),
    }
}

fn shared_type_census_termination(
    standing_held: bool,
    mapping_controls_hold: bool,
    live_unobserved: bool,
) -> Termination {
    if !standing_held || !mapping_controls_hold {
        Termination::ObservationDidNotHold
    } else if live_unobserved {
        Termination::SubjectUnreached
    } else {
        Termination::ObservationHeld
    }
}

fn run_shared_type_leaf_collision_census(source_roots: &[String]) -> InvocationOutcome {
    use crate::v1_compiler_compile::{compile_to_resolved, SourceFile};
    use crate::v1_compiler_emit_rust::emit_rust;
    use crate::v1_tests_claim_shared_type_leaf_collision_census as census;
    use std::rc::Rc;

    const PREFIX: &str = "shared-type-leaf-collision-census";
    const FIXTURE_DIR: &str = "fixtures/shared_type_leaf_collision";
    let unreached = |detail: String| InvocationOutcome {
        termination: Termination::SubjectUnreached,
        message: format!("{PREFIX}: subject unreached: {detail}"),
    };
    if let Err(e) = std::env::set_current_dir(cli_run::workspace_root()) {
        return unreached(format!("could not anchor at the workspace root: {e}"));
    }

    fn load_dir(dir: &str) -> Result<Vec<Rc<SourceFile>>, String> {
        let mut out = Vec::new();
        let entries = std::fs::read_dir(dir).map_err(|e| format!("{dir}: {e}"))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("{dir}: {e}"))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("dag") {
                continue;
            }
            let content =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            out.push(Rc::new(SourceFile {
                path: path.to_string_lossy().to_string(),
                content,
            }));
        }
        if out.is_empty() {
            return Err(format!("{dir}: no .dag files"));
        }
        out.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(out)
    }

    fn subject_build_of_cargo(
        entry: &str,
        graph: Rc<crate::v1_compiler_infer_items::ResolvedGraph>,
    ) -> Result<Rc<census::SubjectBuild>, String> {
        let emit = emit_rust(graph);
        match super::emitted_closure_compile_host::cargo_verdict_of_emit(entry, &emit) {
            Err(cause) => Err(cause),
            Ok(verdict) => shared_type_census_subject_build_from_cargo_verdict(&verdict),
        }
    }

    fn print_report(kind: &str, subject: &str, report: Rc<census::SubjectCollisionReport>) {
        println!(
            "{PREFIX}: {}",
            census::subject_summary_line(kind.to_string(), subject.to_string(), report.clone())
        );
        for row in report.rows.iter() {
            println!(
                "{PREFIX}: {}",
                census::collision_report_line(subject.to_string(), row.clone())
            );
        }
    }

    fn classify_sources(
        label: &str,
        sources: Vec<Rc<SourceFile>>,
    ) -> Result<Rc<census::SubjectCollisionReport>, String> {
        let compiled = compile_to_resolved(Rc::new(sources.into()));
        let graph = compiled
            .graph
            .clone()
            .ok_or_else(|| format!("{label}: compile_to_resolved produced no graph"))?;
        let rows = census::collisions_of_graph(graph.clone());
        let build = subject_build_of_cargo(label, graph)?;
        Ok(census::subject_report(rows, build))
    }

    let fixture_sources = match load_dir(FIXTURE_DIR) {
        Ok(all) => all,
        Err(e) => return unreached(e),
    };
    let select = |names: &[&str]| {
        fixture_sources
            .iter()
            .filter(|s| {
                let name = std::path::Path::new(&s.path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                names.contains(&name)
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    let refuse_sources = select(&["copy_word.dag", "shared_word.dag", "red_entry.dag"]);
    let silent_sources = select(&["silent_zcopy.dag", "silent_share.dag", "silent_entry.dag"]);
    let green_sources = select(&["green_a.dag", "green_b.dag", "green_entry.dag"]);
    if refuse_sources.len() != 3 {
        return unreached("refuse fixture files missing".to_string());
    }
    if silent_sources.len() != 3 {
        return unreached("silent fixture files missing".to_string());
    }
    if green_sources.len() != 3 {
        return unreached("green fixture files missing".to_string());
    }

    let refuse_report = match classify_sources("stlc.red_entry", refuse_sources) {
        Ok(report) => report,
        Err(e) => return unreached(e),
    };
    let silent_report = match classify_sources("stlc.silent_entry", silent_sources) {
        Ok(report) => report,
        Err(e) => return unreached(e),
    };
    let green_report = match classify_sources("stlc.green_entry", green_sources) {
        Ok(report) => report,
        Err(e) => return unreached(e),
    };
    print_report("fixture", "stlc.red_entry", refuse_report.clone());
    print_report("fixture", "stlc.silent_entry", silent_report.clone());
    print_report("fixture", "stlc.green_entry", green_report.clone());

    let standing = census::fixture_standing(
        refuse_report.clone(),
        silent_report.clone(),
        green_report.clone(),
    );
    for line in standing.lines() {
        println!("{PREFIX}: {line}");
    }
    let held = census::fixture_controls_hold(
        refuse_report.clone(),
        silent_report.clone(),
        green_report.clone(),
    );

    use super::emitted_closure_compile_host::CargoVerdict;
    let unobserved_mapped = [
        CargoVerdict::NotAttempted {
            reason: "bounded-control-unobserved".to_string(),
        },
        CargoVerdict::DidNotComplete {
            detail: "bounded-control-unobserved".to_string(),
        },
        CargoVerdict::DependencyFetchFailed {
            status: None,
            stderr_tail: "bounded-control-unobserved".to_string(),
        },
    ]
    .iter()
    .all(|v| shared_type_census_subject_build_from_cargo_verdict(v).is_err());
    let unobserved_live_control = unobserved_mapped && !census::observation_held(held, true);
    let completed_nonzero_control = census::build_is_refused_finding(refuse_report.build.clone())
        && census::build_is_reportable_finding(refuse_report.build.clone())
        && census::observation_held(held, false);
    println!(
        "{PREFIX}: {}",
        census::control_line(
            "unobserved_live_with_standing_unreaches".to_string(),
            unobserved_live_control,
        )
    );
    println!(
        "{PREFIX}: {}",
        census::control_line(
            "completed_nonzero_refusal_does_not_unreach".to_string(),
            completed_nonzero_control,
        )
    );
    let mapping_controls_hold = unobserved_live_control && completed_nonzero_control;

    let mut subjects: Vec<(String, String)> = Vec::new();
    for (label, producer) in instrument_registry() {
        match producer {
            TargetProducer::SelfHost => subjects.push((
                label.target.clone(),
                cli_run::native_compile_entry().to_string(),
            )),
            TargetProducer::V2NativeCli => subjects.push((
                label.target.clone(),
                cli_run::v2_native_cli_entry().to_string(),
            )),
            TargetProducer::NativeClaimProgram { entry } => {
                subjects.push((label.target.clone(), entry.to_string()));
            }
            _ => {}
        }
    }

    let mut reports_ok = true;
    for (name, entry) in subjects {
        match cli_run::load_sources_for_entry_with_pool_index(source_roots, &entry, false) {
            Ok(sources) => match classify_sources(&name, sources) {
                Ok(report) => print_report("closure", &name, report),
                Err(e) => {
                    reports_ok = false;
                    println!("{PREFIX}: subject kind=closure name={name} unreached {e}");
                }
            },
            Err(e) => {
                reports_ok = false;
                println!("{PREFIX}: subject kind=closure name={name} unreached {e}");
            }
        }
    }

    let live_unobserved = !reports_ok;
    InvocationOutcome {
        termination: shared_type_census_termination(
            held,
            mapping_controls_hold,
            live_unobserved,
        ),
        message: format!(
            "{PREFIX}: {} (controls over {FIXTURE_DIR}; report over required subjects and NativeClaimProgramProducer entries)",
            if census::observation_held(held && mapping_controls_hold, live_unobserved) {
                "STANDING held"
            } else if live_unobserved && held && mapping_controls_hold {
                "subject unreached"
            } else {
                "STANDING unmet"
            }
        ),
    }
}

/// THE DEPENDENCY-DEMAND CENSUS PRODUCER (D13 step b2; `gunbc.instrument_targets`
/// `dependency_demand_census_label`). The seed emits and builds the compiler closure exactly as the
/// self-host step does, and spawns it in `demand-census` mode over the same corpus; every line and
/// the exit are decided by `v2.compiler.source_root_eval` `native_demand_census_output`, so the host adds no
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
    const REPORT_ENTRY: &str = "src/v2/compiler/compile.dag";
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

/// A census verb of the emitted compiler, spawned by `cli_run::run_v2_census_verb`. Every line and
/// the exit are decided by the emitted verb; the host adds no verdict.
fn run_census_verb(verb: &str, label: &str, source_roots: &[String]) -> InvocationOutcome {
    match cli_run::run_v2_census_verb(verb, source_roots) {
        Ok(code) => InvocationOutcome {
            termination: match code {
                0 => Termination::ObservationHeld,
                1 => Termination::ObservationDidNotHold,
                _ => Termination::SubjectUnreached,
            },
            message: format!("{label}: exit {code}; the [{verb}] lines above are the census"),
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
mod shared_type_census_cargo_mapping_tests {
    use super::*;
    use crate::cli_run::emitted_closure_compile_host::CargoVerdict;
    use crate::v1_tests_claim_shared_type_leaf_collision_census as census;

    fn completed(status: i32) -> CargoVerdict {
        CargoVerdict::Completed {
            status,
            stderr_tail: "error[E0204]: bounded-control".to_string(),
            probe_line: None,
            probe_diagnostic: None,
            warning_count: 0,
            warning_headers: Vec::new(),
            cargo_network_retries: 0,
            first_error: None,
        }
    }

    #[test]
    fn unobserved_cargo_outcomes_are_unreached_not_refused_findings() {
        let not_attempted = CargoVerdict::NotAttempted {
            reason: "live-subject-not-attempted".to_string(),
        };
        let did_not = CargoVerdict::DidNotComplete {
            detail: "live-subject-did-not-complete".to_string(),
        };
        let fetch = CargoVerdict::DependencyFetchFailed {
            status: Some(1),
            stderr_tail: "live-subject-fetch-failed".to_string(),
        };
        let na = shared_type_census_subject_build_from_cargo_verdict(&not_attempted);
        let dc = shared_type_census_subject_build_from_cargo_verdict(&did_not);
        let ff = shared_type_census_subject_build_from_cargo_verdict(&fetch);
        assert!(na.as_ref().unwrap_err().contains("NotAttempted"));
        assert!(na
            .as_ref()
            .unwrap_err()
            .contains("live-subject-not-attempted"));
        assert!(dc.as_ref().unwrap_err().contains("DidNotComplete"));
        assert!(dc
            .as_ref()
            .unwrap_err()
            .contains("live-subject-did-not-complete"));
        assert!(ff.as_ref().unwrap_err().contains("DependencyFetchFailed"));
        assert!(ff
            .as_ref()
            .unwrap_err()
            .contains("live-subject-fetch-failed"));
        assert_eq!(
            shared_type_census_termination(true, true, true),
            Termination::SubjectUnreached
        );
        assert!(!census::observation_held(true, true));
    }

    #[test]
    fn completed_nonzero_is_a_reportable_refusal_and_does_not_unreach() {
        let build = shared_type_census_subject_build_from_cargo_verdict(&completed(101)).unwrap();
        assert!(census::build_is_refused_finding(build.clone()));
        assert!(census::build_is_reportable_finding(build));
        assert!(census::observation_held(true, false));
        assert_eq!(
            shared_type_census_termination(true, true, false),
            Termination::ObservationHeld
        );
    }

    #[test]
    fn completed_zero_is_accepted() {
        let build = shared_type_census_subject_build_from_cargo_verdict(&completed(0)).unwrap();
        assert!(!census::build_is_refused_finding(build.clone()));
        assert!(census::build_is_reportable_finding(build));
    }
}

/// THE REGISTRY-VERSUS-ROWS IDENTITY CHECK (`gunbc.target_invocation_seed_growth`). `instrument_registry`
/// is hand-written beside the `gunbc.instrument_targets` `instrument_bindings` rows it mirrors, and
/// nothing compared them: a shared label whose producer kind, entry or function differs dispatched
/// the registry's value while the rows said another. This evaluates the rows through the interpreter
/// and joins by label identity. It is a frozen-X residue that dissolves with the v1 CLI, and it
/// runs by hand (`cargo test registry_matches_rows`), not on the required path.
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

    /// The rows are read ONCE per process: each read drives the wet interpreter over the whole
    /// corpus, and both tests join against the same reading.
    fn read_rows() -> &'static BTreeMap<String, RowReading> {
        static ROWS: std::sync::OnceLock<BTreeMap<String, RowReading>> = std::sync::OnceLock::new();
        ROWS.get_or_init(evaluate_rows)
    }

    fn evaluate_rows() -> BTreeMap<String, RowReading> {
        let reader = cli_run::workspace_root().join("dag/gunbc/instrument_targets.dag");
        let roots = cli_run::default_source_roots();
        let (graph, source_indices) =
            cli_run::resolve_entry_graph(&roots, reader.to_str().expect("utf8 path"))
                .expect("resolve rows");
        let ctx = cli_run::make_eval_context(
            graph.as_ref(),
            source_indices,
            crate::v1_interpreter::ExecutionMode::Wet,
        );
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
            let reading = (
                ctx.resolve(*variant_name),
                text(&ctx, producer, "entry"),
                text(&ctx, producer, "function"),
            );
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
