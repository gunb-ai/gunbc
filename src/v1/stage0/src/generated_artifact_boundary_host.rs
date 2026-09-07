//! Host realization for `gunbc.generated_artifact_emit` — the committed generated-artifact
//! population, produced from its authority and adjudicated against the tree.
//!
//! WHY THIS MODULE EXISTS — the same shape as `partition_crate_boundary_host`, one population
//! over. The producer was already complete: `generated_artifact_body_for_path` is a pure
//! projection over the three authorities `main_wet` folds — the committed-artifact roster,
//! `artifact_path`, the single `artifact_generate` dispatch — answering what the tree ought to
//! hold at any repo-relative path. Nothing asked it about the WHOLE roster: the one caller
//! (`claim_executor`'s behavioural-receipt census) asks only about `src/v1/stage0/src/<mirror>`
//! paths, so the 15 committed non-mirror artifacts — `DESIGN.md`, `ROADMAP.md`, the workflow
//! YAML, `.gitignore`, `.gitattributes`, the githooks, the plans — were computed and compared by
//! nothing.
//!
//! WHAT THAT COST, measured. `DESIGN.md` is a generated projection of `gunbc.design_document`;
//! of the twelve most recent commits touching it, six changed the artifact with no authority
//! change. gunbc#9392 found the result by hand while doing something else and recorded it in
//! the artifact: 2198 characters in `DESIGN.md` that no authority held — the
//! parallel-representation debt DESIGN §2/§3 names, in the document that names it. It could
//! accumulate because the drift gates went out with the 2026-08-15 floor cut and this family sat
//! on that cut's re-add queue.
//!
//! PRODUCTION PRECEDES ADJUDICATION, carried by the type, not discipline — the construction
//! `v2.workflow.required_regen` adopted after the regen cut shipped a gate whose only closing
//! move did not exist. Every verdict is reached THROUGH a produced population; "refused, having
//! compared nothing" has no spelling except the two arms carrying no files: the authority
//! declined to answer, or one artifact's own generator refused.
//!
//! THIS IS NOT A SECOND PRODUCER. It resolves `gunbc.generated_artifact_emit` and asks it; it
//! mints no roster, forks no dispatch, names no individual artifact. Adding an artifact to
//! `generated_artifact_registry` enrols it here with no edit to this file.

use std::fs;

use super::workspace_root;
use crate::v1_interpreter::{InterpContext, Value};

/// The entry whose closure carries both the roster and the per-path projection.
const AUTHORITY_ENTRY: &str = "dag/gunbc/generated_artifact_emit.dag";

/// The route an author takes to close a drift red. Named once, so the refusal below cannot name
/// a command that does not exist — the laundering trap DESIGN records twice, most recently the
/// partition-crate headers pointing at the deleted `regen_stage0`.
pub const GENERATED_ARTIFACT_PRODUCING_COMMAND: &str =
    "gunbc run --source-root dag --source-root src/v2 \
     --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet";

/// Regenerates only the two `docs/*.md` ledger projections and returns `ProcessExit`.
/// Distinct from `GENERATED_ARTIFACT_PRODUCING_COMMAND`: that entry resolves the whole
/// registry emit graph and does not complete on the runners that SIGKILL `main_wet`.
/// The operator-facing recipe is `tools.docs_projection_agreement` `docs_projection_regen_command`
/// and is already inside the gate's `ProcessExit` reason; this host does not mint a second copy.
const DOCS_PROJECTION_GATE_ENTRY: &str = "dag/gunbc/instruments/docs_projection_gate.dag";

/// Outcome of the docs-ledger agreement entry. Kept separate from
/// `GeneratedArtifactBoundaryOutcome` because that carrier resolves the whole registry;
/// this one must be able to refuse a stale `docs/design-failure-modes.md` even when that
/// resolve never returns.
pub enum DocsProjectionAgreement {
    Clean,
    Refused { cause: String },
}

pub fn run_docs_projection_agreement(source_roots: &[String]) -> DocsProjectionAgreement {
    let (graph, indices) = match crate::cli_run::resolve_entry_graph_shared(
        source_roots,
        DOCS_PROJECTION_GATE_ENTRY,
    ) {
        Ok(v) => v,
        Err(e) => {
            return DocsProjectionAgreement::Refused {
                cause: format!("resolve {DOCS_PROJECTION_GATE_ENTRY}: {e}"),
            }
        }
    };
    let ctx = crate::cli_run::make_eval_context(
        &graph,
        indices,
        crate::v1_interpreter::ExecutionMode::Hermetic,
    );
    let out = match crate::v1_interpreter::run_in_context_with_args(&ctx, "main", &[], false) {
        Ok(v) => v,
        Err(e) => {
            return DocsProjectionAgreement::Refused {
                cause: format!("docs projection gate: {e:?}"),
            }
        }
    };
    match super::classify_exit(&out, &ctx) {
        super::ExitClass::Success => DocsProjectionAgreement::Clean,
        super::ExitClass::Failure { reason, .. } => DocsProjectionAgreement::Refused {
            cause: reason.unwrap_or_else(|| "docs projection drift".to_string()),
        },
        super::ExitClass::NotProcessExit { type_name } => DocsProjectionAgreement::Refused {
            cause: format!("docs projection gate returned {type_name}, not ProcessExit"),
        },
    }
}

/// What the generated-artifact population says about one repo-relative path.
///
/// Three states because the honest answers are three. `NotGenerated` is a POSITIVE answer —
/// the path is not in the generated-artifact population — and routes a caller to the
/// mirror-emit population. Folded into `Refused` it would say generation FAILED for an ordinary
/// mirror: false and differently actionable.
pub enum GeneratedArtifactPathBody {
    Produced(String),
    Refused(String),
    NotGenerated,
}

/// The generated-artifact authority's evaluation context, resolved AT MOST ONCE per run and
/// shared by every caller that needs it.
///
/// One cell, not one resolve per asking site: the behavioural-receipt census asks for a module
/// yielding no call, its differential loop for every selected module, the required phase below
/// once per rostered artifact. Two resolves of one closure are two producers of one context
/// paying the corpus-sized cost twice.
pub fn generated_artifact_ctx<'a>(
    source_roots: &[String],
    cell: &'a mut Option<InterpContext>,
) -> Result<&'a InterpContext, String> {
    if cell.is_none() {
        let (graph, indices) =
            crate::cli_run::resolve_entry_graph_shared(source_roots, AUTHORITY_ENTRY)
                .map_err(|e| format!("resolve {AUTHORITY_ENTRY}: {e}"))?;
        *cell = Some(crate::cli_run::make_eval_context(
            &graph,
            indices,
            // HERMETIC, not Wet. The projection is pure (folds a roster, returns a String), so
            // a host effect reached during it means a generator is doing something this gate
            // must not perform on its behalf; Hermetic refuses there instead of carrying it out.
            crate::v1_interpreter::ExecutionMode::Hermetic,
        ));
    }
    Ok(cell.as_ref().expect("the context was just installed"))
}

/// Ask the already-resolved generated-artifact authority for the body it generates at a path.
///
/// COST SHAPE is why this takes a CONTEXT rather than `source_roots`: resolving the closure in a
/// per-path loop makes the unit of computation the corpus while the unit of fact is one path —
/// DESIGN §6's cost-shape defect, fixed regardless of realized n. The caller resolves once; each
/// path is one interpreter call against that context.
pub fn generated_artifact_body_for_path(
    ctx: &InterpContext,
    repo_rel_path: &str,
) -> Result<GeneratedArtifactPathBody, String> {
    let out = crate::v1_interpreter::run_in_context_with_args(
        ctx,
        "generated_artifact_body_for_path",
        &[(
            Some("path".to_string()),
            Value::Str(repo_rel_path.to_string().into()),
        )],
        false,
    )
    .map_err(|e| format!("generated_artifact_body_for_path({repo_rel_path}): {e:?}"))?;
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = &out
    else {
        // No default arm: an unrecognised shape is ignorance; guessing NotGenerated would route
        // a real generated artifact to the mirror emit and refuse it there for the wrong reason.
        return Err(format!(
            "generated_artifact_body_for_path({repo_rel_path}) returned a non-variant value"
        ));
    };
    if ctx.sym_eq(*variant_name, "GeneratedArtifactPathNotGenerated") {
        return Ok(GeneratedArtifactPathBody::NotGenerated);
    }
    if ctx.sym_eq(*variant_name, "GeneratedArtifactPathBodyProduced") {
        return match ctx.field(fields, "content") {
            Some(Value::Str(c)) => Ok(GeneratedArtifactPathBody::Produced(c.to_string())),
            _ => Err(format!(
                "GeneratedArtifactPathBodyProduced for {repo_rel_path} carried no String content"
            )),
        };
    }
    if ctx.sym_eq(*variant_name, "GeneratedArtifactPathBodyRefused") {
        return match ctx.field(fields, "reason") {
            Some(Value::Str(r)) => Ok(GeneratedArtifactPathBody::Refused(r.to_string())),
            _ => Err(format!(
                "GeneratedArtifactPathBodyRefused for {repo_rel_path} carried no String reason"
            )),
        };
    }
    Err(format!(
        "generated_artifact_body_for_path({repo_rel_path}) returned an unknown variant"
    ))
}

/// The committed roster, asked of the authority rather than restated here.
///
/// `committed_generated_artifact_paths` is `map(artifact_path)` over
/// `committed_generated_artifacts`, so this roster and the one `main_wet` writes are one fold
/// over one registry. A path list here would be the second roster DESIGN §3 forbids, stale on
/// the first artifact added.
fn committed_generated_artifact_paths(ctx: &InterpContext) -> Result<Vec<String>, String> {
    let out = crate::v1_interpreter::run_in_context_with_args(
        ctx,
        "committed_generated_artifact_paths",
        &[],
        false,
    )
    .map_err(|e| format!("committed_generated_artifact_paths: {e:?}"))?;
    let Value::List(items) = &out else {
        return Err("committed_generated_artifact_paths returned a non-list value".to_string());
    };
    let mut paths = Vec::with_capacity(items.len());
    for item in items.iter() {
        match item {
            Value::Str(s) => paths.push(s.to_string()),
            _ => {
                return Err(
                    "committed_generated_artifact_paths returned a non-String element".to_string(),
                )
            }
        }
    }
    Ok(paths)
}

/// One generated artifact, beside what the tree currently holds at its path.
///
/// `committed` is `None` for a path with no file — a DIFFERENT state from differing bytes: an
/// absent artifact is the shape a newly rostered one takes on its introducing commit, and
/// collapsing the two would report it as ordinary drift.
#[derive(Debug, Clone)]
pub struct AdjudicatedArtifact {
    pub path: String,
    pub generated: String,
    pub committed: Option<String>,
}

/// FREE FUNCTIONS RATHER THAN INHERENT METHODS, THROUGHOUT THIS FILE, not as style:
/// `std.decl_ref` offers `WholeDeclaration` or `NamedField`, neither naming an `impl` method, so
/// every method here would be an item this file's seed-growth obligation
/// (`gunbc.generated_artifact_boundary_seed_growth`) could not cite -- the uncitable-item class
/// `gunbc.seed_growth_admission` counts. A roster that cannot name half of what it owes is the
/// completeness gap `gunbc.seed_growth` discloses; no reason to widen it for four methods.
pub fn artifact_disposition(a: &AdjudicatedArtifact) -> ArtifactDisposition {
    match &a.committed {
        None => ArtifactDisposition::Absent,
        Some(bytes) if *bytes == a.generated => ArtifactDisposition::Matches,
        Some(_) => ArtifactDisposition::Drifted,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArtifactDisposition {
    /// The committed bytes are exactly what the authority generates.
    Matches,
    /// A file exists and its bytes are not what the authority generates — a hand edit, or an
    /// authority change whose projection was never installed.
    Drifted,
    /// No file at the path. An artifact rostered but never written.
    Absent,
}

pub fn artifact_disposition_name(d: ArtifactDisposition) -> &'static str {
    match d {
        ArtifactDisposition::Matches => "matches",
        ArtifactDisposition::Drifted => "drifted",
        ArtifactDisposition::Absent => "absent",
    }
}

/// A rostered artifact this run could reach no verdict on, with the cause that stopped it.
///
/// Kept SEPARATE from the adjudicated population, not a third disposition: a drift is work an
/// author does now, an unanswerable artifact is a defect in the authority or this bridge. A run
/// reaching no verdict on half the roster yet reporting "0 drifted" is the execution-provenance
/// loss DESIGN names — an unreached observation reading as a pass — so the two counts print side
/// by side and either stops the line.
#[derive(Debug, Clone)]
pub struct UnadjudicatedArtifact {
    pub path: String,
    pub cause: String,
}

/// THE OUTCOME IS REACHED THROUGH THE PRODUCED POPULATION, NEVER BESIDE IT.
///
/// `CarrierRefused` is the one arm without a population, reachable only when the authority could
/// not be resolved or its roster not read — before any artifact was asked about. Every other
/// verdict names the population it was computed against, so no consumer reads a verdict without
/// holding what produced it.
#[derive(Debug)]
pub enum GeneratedArtifactBoundaryOutcome {
    /// The authority refused to answer at all. Nothing was compared, and the cause says why.
    CarrierRefused { cause: String },
    /// The roster was read and every member was asked. `unadjudicated` is the members that
    /// reached no verdict, each with its cause.
    Adjudicated {
        artifacts: Vec<AdjudicatedArtifact>,
        unadjudicated: Vec<UnadjudicatedArtifact>,
    },
}

/// Every rostered artifact whose committed bytes are not what its authority generates.
pub fn boundary_divergent(o: &GeneratedArtifactBoundaryOutcome) -> Vec<&AdjudicatedArtifact> {
    match o {
        GeneratedArtifactBoundaryOutcome::CarrierRefused { .. } => Vec::new(),
        GeneratedArtifactBoundaryOutcome::Adjudicated { artifacts, .. } => artifacts
            .iter()
            .filter(|a| artifact_disposition(a) != ArtifactDisposition::Matches)
            .collect(),
    }
}

/// Clean means: the carrier answered, THE POPULATION WAS NON-EMPTY, every rostered member
/// reached a verdict, every verdict was `Matches`. A refusal anywhere in that chain is not clean,
/// so "nothing drifted" cannot cover for "nothing was asked".
///
/// THE EMPTINESS CONJUNCT IS NOT DEFENSIVE PADDING; omitting it was a real defect in this file's
/// first cut (caught in review of gunbc#9415). Zero adjudicated and zero unadjudicated satisfies
/// "every verdict matched" VACUOUSLY, so asking about nothing rendered like asking about seventy
/// paths and finding all correct — the empty-observation narrow DESIGN names, bottom-as-answer
/// conflated with bottom-as-ignorance, strictly worse than the widen §5 forbids: a widen is
/// merely expensive, a narrow is silently uncovered.
///
/// Checked HERE as well as at the producer below, not redundantly: the producer refuses an
/// empty roster where the roster is READ (every live run); this conjunct covers any outcome
/// VALUE, including one a caller or fixture built by hand — the only boundary at which an empty
/// population is still expressible, so the discriminating red is authored against the value.
pub fn boundary_is_clean(o: &GeneratedArtifactBoundaryOutcome) -> bool {
    match o {
        GeneratedArtifactBoundaryOutcome::CarrierRefused { .. } => false,
        GeneratedArtifactBoundaryOutcome::Adjudicated {
            artifacts,
            unadjudicated,
        } => {
            !(artifacts.is_empty() && unadjudicated.is_empty())
                && unadjudicated.is_empty()
                && artifacts
                    .iter()
                    .all(|a| artifact_disposition(a) == ArtifactDisposition::Matches)
        }
    }
}

/// Ask the authority about every committed generated artifact and adjudicate each against the
/// tree.
///
/// READ-ONLY BY CONSTRUCTION: no write path, no flag that opens one. Installing a regenerated
/// artifact is `main_wet`'s job; a gate that could write its own subject has a green that proves
/// nothing.
pub fn run_generated_artifact_boundary(
    source_roots: &[String],
) -> GeneratedArtifactBoundaryOutcome {
    let mut cell: Option<InterpContext> = None;
    let ctx = match generated_artifact_ctx(source_roots, &mut cell) {
        Ok(ctx) => ctx,
        Err(cause) => return GeneratedArtifactBoundaryOutcome::CarrierRefused { cause },
    };
    let paths = match committed_generated_artifact_paths(ctx) {
        Ok(paths) => paths,
        Err(cause) => return GeneratedArtifactBoundaryOutcome::CarrierRefused { cause },
    };
    // AN EMPTY ROSTER IS IGNORANCE, NOT A FACT ABOUT THE TREE. `committed_generated_artifacts`
    // filters a module-scope literal registry, so empty means the projection stopped seeing the
    // registry or this bridge asked the wrong question -- defects in the asking, not evidence
    // the repository commits no generated artifacts. Admitting it would report a clean
    // adjudication over a population never had.
    if paths.is_empty() {
        return GeneratedArtifactBoundaryOutcome::CarrierRefused {
            cause: "committed_generated_artifact_paths returned an EMPTY roster. The registry it                     filters is a module-scope literal, so empty is not a fact about the tree --                     it means the projection or this bridge lost sight of the population.                     Adjudicating nothing and reporting it clean is the empty-observation narrow"
                .to_string(),
        };
    }

    let workspace = workspace_root();
    let mut artifacts = Vec::new();
    let mut unadjudicated = Vec::new();
    for path in paths {
        let generated = match generated_artifact_body_for_path(ctx, &path) {
            Ok(GeneratedArtifactPathBody::Produced(content)) => content,
            Ok(GeneratedArtifactPathBody::Refused(reason)) => {
                unadjudicated.push(UnadjudicatedArtifact {
                    path,
                    cause: format!("its generator refused: {reason}"),
                });
                continue;
            }
            // Roster and projection read the SAME registry, so a rostered path the projection
            // calls ungenerated is a defect in the authority, not a fact about the tree; it
            // refuses rather than being skipped as "nothing to compare".
            Ok(GeneratedArtifactPathBody::NotGenerated) => {
                unadjudicated.push(UnadjudicatedArtifact {
                    path,
                    cause: "the committed roster names this path but the per-path projection \
                            reports it as not generated — the two disagree about one registry"
                        .to_string(),
                });
                continue;
            }
            Err(cause) => {
                unadjudicated.push(UnadjudicatedArtifact { path, cause });
                continue;
            }
        };
        // A read error is NOT `Absent`. "No file here" and "a file is here and I could not read
        // it" have different remedies, and only the first is the newly-rostered shape.
        let full = workspace.join(&path);
        let committed = match fs::read_to_string(&full) {
            Ok(bytes) => Some(bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => {
                unadjudicated.push(UnadjudicatedArtifact {
                    path,
                    cause: format!("the committed bytes could not be read: {e}"),
                });
                continue;
            }
        };
        artifacts.push(AdjudicatedArtifact {
            path,
            generated,
            committed,
        });
    }

    GeneratedArtifactBoundaryOutcome::Adjudicated {
        artifacts,
        unadjudicated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matched(path: &str, bytes: &str) -> AdjudicatedArtifact {
        AdjudicatedArtifact {
            path: path.to_string(),
            generated: bytes.to_string(),
            committed: Some(bytes.to_string()),
        }
    }

    fn adjudicated(path: &str, generated: &str, committed: Option<&str>) -> AdjudicatedArtifact {
        AdjudicatedArtifact {
            path: path.to_string(),
            generated: generated.to_string(),
            committed: committed.map(|c| c.to_string()),
        }
    }

    #[test]
    fn disposition_separates_absent_from_drifted() {
        assert_eq!(
            artifact_disposition(&matched("DESIGN.md", "a")),
            ArtifactDisposition::Matches
        );
        assert_eq!(
            artifact_disposition(&adjudicated("DESIGN.md", "a", Some("b"))),
            ArtifactDisposition::Drifted
        );
        assert_eq!(
            artifact_disposition(&adjudicated("DESIGN.md", "a", None)),
            ArtifactDisposition::Absent
        );
    }

    /// THE DISCRIMINATING RED this gate exists for: a run reaching no verdict on a rostered
    /// member must not read as clean because every member it DID reach matched.
    /// Green-with-an-unadjudicated-row is the execution-provenance loss the module header names.
    #[test]
    fn an_unadjudicated_member_is_not_clean_even_when_every_verdict_matched() {
        let all_matched = GeneratedArtifactBoundaryOutcome::Adjudicated {
            artifacts: vec![matched("DESIGN.md", "a"), matched("ROADMAP.md", "b")],
            unadjudicated: Vec::new(),
        };
        assert!(boundary_is_clean(&all_matched));
        assert!(boundary_divergent(&all_matched).is_empty());

        let one_unanswered = GeneratedArtifactBoundaryOutcome::Adjudicated {
            artifacts: vec![matched("DESIGN.md", "a"), matched("ROADMAP.md", "b")],
            unadjudicated: vec![UnadjudicatedArtifact {
                path: ".gitignore".to_string(),
                cause: "its generator refused: population refused".to_string(),
            }],
        };
        assert!(!boundary_is_clean(&one_unanswered));
        // And it is NOT reported as drift: the two counts stay separable, so the ledger says
        // which of the two happened.
        assert!(boundary_divergent(&one_unanswered).is_empty());
    }

    /// THE SECOND DISCRIMINATING RED, the one review found missing: zero adjudicated and zero
    /// unadjudicated satisfies "every verdict matched" vacuously; before the emptiness conjunct
    /// this returned TRUE, so asking about nothing reported like asking about everything.
    #[test]
    fn an_empty_population_is_not_clean_even_though_no_verdict_disagreed() {
        let asked_nothing = GeneratedArtifactBoundaryOutcome::Adjudicated {
            artifacts: Vec::new(),
            unadjudicated: Vec::new(),
        };
        assert!(!boundary_is_clean(&asked_nothing));
        // And it is not drift either: the ledger must not name a path, because there is none.
        assert!(boundary_divergent(&asked_nothing).is_empty());

        // The positive control that keeps the conjunct honest: one matching member IS clean, so
        // the check above is emptiness and not a blanket refusal.
        let asked_one = GeneratedArtifactBoundaryOutcome::Adjudicated {
            artifacts: vec![matched("DESIGN.md", "a")],
            unadjudicated: Vec::new(),
        };
        assert!(boundary_is_clean(&asked_one));
    }

    #[test]
    fn a_carrier_refusal_is_not_clean_and_names_no_population() {
        let refused = GeneratedArtifactBoundaryOutcome::CarrierRefused {
            cause: "resolve failed".to_string(),
        };
        assert!(!boundary_is_clean(&refused));
        assert!(boundary_divergent(&refused).is_empty());
    }

    #[test]
    fn drift_is_reported_and_stops_the_line() {
        let drifted = GeneratedArtifactBoundaryOutcome::Adjudicated {
            artifacts: vec![
                matched("ROADMAP.md", "b"),
                adjudicated("DESIGN.md", "authority", Some("hand edit")),
            ],
            unadjudicated: Vec::new(),
        };
        assert!(!boundary_is_clean(&drifted));
        let divergent = boundary_divergent(&drifted);
        assert_eq!(divergent.len(), 1);
        assert_eq!(divergent[0].path, "DESIGN.md");
        assert_eq!(
            artifact_disposition(divergent[0]),
            ArtifactDisposition::Drifted
        );
    }
}
