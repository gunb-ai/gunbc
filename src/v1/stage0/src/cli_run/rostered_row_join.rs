//! THE DECLARED-VERSUS-ROSTERED IDENTITY JOIN, executed as a merge-blocking check.
//!
//! WHAT IT DECIDES. A carrier whose rows live one per module and whose roster re-enumerates them
//! by hand authors one membership fact twice, so a row that lands DECLARED and UNROSTERED is
//! absent from every projection derived from that roster and nothing refuses it
//! (`gunbc.guarantee_stall.roster_re_enumerates_its_own_rows_stall`). This join reads the declared
//! population through the `data_decl_type_facts` producer, reads each roster's membership out of
//! the parse the required-CI parse phase has already performed, and refuses with the LOCATED
//! IDENTITY of every declared row its roster omits.
//!
//! IT IS AN IDENTITY JOIN AND NEVER A COUNT. `declared == rostered` is satisfied by a
//! simultaneous add and drop, which is precisely the merge shape these contended carriers
//! produce. Every declared row is looked up in the roster by its own declaration name, the number
//! CHECKED is asserted equal to the number DECLARED, and an empty declared population is itself a
//! refusal — a subject silently narrowed to nothing is the state in which this whole check reads
//! as coverage while ranging over nothing.
//!
//! EXISTENCE BEFORE MEANING. The source inventory is established from disk BEFORE any parse
//! decides anything, and every `.dag` file under the pool roots must be accounted for by the
//! declared-population walk. A file the walk did not account for is refused BY SOURCE IDENTITY —
//! path plus a content digest — because before a parse yields a declaration identity, the source
//! is the only subject there is. "It failed to parse" is not an exclusion policy: the underlying
//! walk already refuses, loudly, on an unparseable source, and this arm covers the remaining way a
//! source can contribute nothing (an unclassified module binding), which was silent until here.
//!
//! WHAT IT DOES NOT DO. It does not remove the double authoring — after this check an author still
//! writes the row and then writes the roster entry — so the invalid state stays writable and the
//! class is MECHANICALLY PREVENTABLE, not structural. The ceiling is a roster FOLDED over the
//! declared population, which needs declaration-value binding the substrate does not have.
//!
//! THE UNACCOUNTED-SOURCE ARM'S RED IS NOT EXERCISED TODAY, and that is stated rather than left
//! for a reader to assume from a green. Measured on the live corpus, every `.dag` under the pool
//! roots carries a module header and no two declare the same module path, so the arm's population
//! is empty and it has never fired. Its RED IS authorable — a headerless source, or a second file
//! declaring an existing module path — but planting one is not free: the parse sweep in this same
//! phase adjudicates the same file, so a fixture there would be answering a different check's
//! question. The arm is enrolled because the accounting obligation is the constraint (a source
//! contributing nothing must be refused, not dropped), and its evidence is a measurement of the
//! population, not an executed refusal.
//!
//! WHY THIS FILE SITS UNDER `cli_run/` AND NOT AT THE STAGE0 CRATE ROOT, recorded because the
//! first placement was refused by CI and the refusal was RIGHT. `committed_generated_basenames`
//! walks the stage0 crate's TOP LEVEL and treats every `.rs` there as a committed generated
//! mirror unless an authored roster exempts it, so a hand-written module at the root is compared
//! against an emitted population that does not contain it and refuses as
//! `CommittedMirrorNoLongerEmitted`. The two remedies were: add a row to
//! `HAND_MAINTAINED_STAGE0_FILES`, or live in a hand-maintained DIRECTORY. The directory is the
//! one that adds no second enumeration of a membership fact — which is precisely the class this
//! module exists to make loud, so paying for it here would have been the defect wearing a fix's
//! clothes.
//!
//! `type_name` IS A SPELLING. `DataDeclTypeFact` carries the authored head name of the declared
//! type annotation, so membership here is a string comparison over discovery evidence, not typed
//! membership. It is exact only because each enrolled spelling is, today, declared in one module.

use std::collections::{BTreeMap, BTreeSet};

use crate::cli_run::declaration_index::{
    index_get, module_is_fixture_carrier, DeclarationIndex, ModuleDeclarationRecord,
};

/// The `.dag` authority this host roster realizes.
pub const JOIN_AUTHORITY_MODULE: &str = "gunbc.rostered_row_join";
pub const JOIN_AUTHORITY_TYPE: &str = "RosteredRowType";
pub const JOIN_AUTHORITY_ROSTER: &str = "rostered_row_joins";

/// The pool roots the declared population is read from — the authored `.dag` roots the parse
/// sweep already covers, minus `src/v1`, whose modules collide on last segments and are not part
/// of any rostered carrier.
pub const JOIN_POOL_ROOTS: [&str; 2] = ["dag", "src/v2"];

/// One enrolled row type: the declared-type spelling and the declaration that rosters it.
pub struct EnrolledRowType {
    /// The `RosteredRowType` variant this row realizes, in the authority's own spelling.
    pub variant: &'static str,
    pub type_name: &'static str,
    pub roster_module: &'static str,
    pub roster_declaration: &'static str,
}

/// Every rostered row type, joined against `RosteredRowType`'s variants in both directions on
/// every required run — the same shape as `claim_executor`'s phase-roster join, for the same
/// reason: nothing else holds these two enumerations together.
pub const ENROLLED_ROW_TYPES: [EnrolledRowType; 5] = [
    EnrolledRowType {
        variant: "GuaranteeStallRows",
        type_name: "GuaranteeStall",
        roster_module: "gunbc.guarantee_stall.roster",
        roster_declaration: "all_guarantee_stalls",
    },
    EnrolledRowType {
        variant: "RecurringFailureModeRows",
        type_name: "RecurringFailureMode",
        roster_module: "gunbc.recurring_failure_mode.roster",
        roster_declaration: "recurring_failure_mode_roster",
    },
    EnrolledRowType {
        variant: "RungDropRows",
        type_name: "RungDrop",
        roster_module: "gunbc.rung_drop.roster",
        roster_declaration: "rung_drop_roster",
    },
    EnrolledRowType {
        variant: "CollisionRowControl",
        type_name: "CollisionControlRow",
        roster_module: "test.fixture.rostered_row_join.collision_control_roster",
        roster_declaration: "collision_control_roster",
    },
    EnrolledRowType {
        variant: "OmittedRowControl",
        type_name: "OmittedControlRow",
        roster_module: "test.fixture.rostered_row_join.omitted_control",
        roster_declaration: "omitted_control_roster",
    },
];

/// One typed refusal. One variant per QUESTION, never per site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinFindingKind {
    /// The `.dag` authority and this host roster disagree about which row types are enrolled.
    Vocabulary,
    /// The module a roster is declared in is absent from the index.
    RosterModuleAbsent,
    /// The roster module exists and declares no such roster.
    RosterDeclarationAbsent,
    /// A declared row of an enrolled type that its roster does not name.
    DeclaredNotRostered,
    /// A roster member spelling that resolves to more than one module.
    RosterMemberAmbiguous,
    /// A roster member spelling imported from a module the index does not carry.
    RosterMemberUnresolved,
    /// An enrolled type whose declared population is EMPTY — the join ranged over nothing.
    DeclaredPopulationEmpty,
    /// The join checked fewer rows than it declared, so its own subject narrowed mid-run.
    CheckedNarrowerThanDeclared,
    /// A source under the pool roots that the declared-population walk did not account for.
    SourceUnaccounted,
    /// A fixture-homed enrolled type that produced no finding: the control did not fire, so the
    /// join is green by construction and establishes nothing.
    ControlDidNotFire,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinFinding {
    pub kind: JoinFindingKind,
    pub row_type: String,
    pub subject: String,
    pub detail: String,
}

impl JoinFinding {
    pub fn rendered(&self) -> String {
        format!(
            "{:?} [{}] {} — {}",
            self.kind, self.row_type, self.subject, self.detail
        )
    }
}

/// The denominators a green must name: nothing here reads as coverage without them.
///
/// `rostered` IS A SUPERSET AND IS NEVER COMPARED TO ANYTHING. It counts the DECLARATION
/// IDENTITIES the roster declaration resolves to, which for a list literal includes its own
/// declared type as well as its members, so it is a denominator for the reader and not a term in
/// any verdict. Membership is
/// decided one declaration at a time by lookup; `declared == rostered` would be exactly the count
/// equality this join exists to refuse, and it is not computed anywhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinCounts {
    pub row_type: String,
    /// Every row in the declared population carrying this type spelling, before eligibility.
    pub population_of_type: usize,
    /// Rows of this spelling excluded because their carrier class differs from the roster's.
    pub excluded_other_carrier: usize,
    pub declared: usize,
    pub rostered: usize,
    pub checked: usize,
    pub fixture_home: bool,
}

#[derive(Debug, Clone, Default)]
pub struct JoinReport {
    /// Refusals from PRODUCTION rosters and from the join's own integrity — these stop the line.
    pub findings: Vec<JoinFinding>,
    /// Refusals from FIXTURE rosters — the enrolled controls, which are expected to fire.
    pub control_findings: Vec<JoinFinding>,
    pub counts: Vec<JoinCounts>,
    pub sources_accounted: usize,
}

/// A source's content identity, for a subject that has no declaration identity yet.
fn content_digest(bytes: &[u8]) -> String {
    // FNV-1a 64, the digest the seed already uses for content identity elsewhere. It names the
    // bytes; it is not a security claim.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

/// Which DECLARATIONS a roster declaration names — as (module path, declaration name) pairs,
/// resolved, never as bare spellings.
///
/// WHY THE MODULE COMPONENT IS LOAD-BEARING AND A BARE NAME IS NOT A MEMBERSHIP KEY. The first
/// version of this join tested `rostered.contains(&row.decl_name)` against the set of spellings
/// occurring inside the roster declaration. Both sides carried a module path and both discarded it
/// before the decision, so two eligible declarations sharing a bare name in different modules were
/// one key: `alpha.row` rostered and `beta.row` not, and BOTH read as rostered. Every accounting
/// equality still held — population 2, declared 2, excluded 0, checked 2 — because the check
/// examined both declarations and gave one of them the wrong answer, which no denominator
/// reconciliation can see. It was latent rather than live (measured: the three enrolled row types
/// carried no bare-name collision at the time), and current-corpus uniqueness is an accident, not an
/// enforced precondition, so it could not be cited as the guard. Found in review of gunbc#10496.
///
/// HOW A MEMBER RESOLVES. A roster is a list of bare references, and the roster module's own import
/// claims say where each name comes from — the same surface the resolver reads. A spelling declared
/// by the roster module itself resolves to that module; otherwise every `import M { .. S .. }`
/// naming it contributes `M`. This needs REFERENCE-TARGET resolution only; it does not need the
/// declaration VALUE binding the ceiling awaits, which is why the repair was affordable here.
///
/// AMBIGUITY REFUSES RATHER THAN WIDENING. If a spelling resolves to more than one module, the
/// join reports it and admits NEITHER identity: taking the union would be the absorbing fallback
/// (§5) — "cannot tell which declaration this names, so accept both" — in the one position where
/// the answer decides whether a row is rostered.
fn rostered_identities(
    index: &DeclarationIndex,
    record: &ModuleDeclarationRecord,
    roster_declaration: &str,
    row_type: &str,
) -> (BTreeSet<(String, String)>, Vec<JoinFinding>) {
    let mut identities = BTreeSet::new();
    let mut findings = Vec::new();
    let spellings: BTreeSet<&str> = record
        .referenced
        .iter()
        .chain(record.authored_type_references.iter())
        .filter(|(in_declaration, _)| in_declaration == roster_declaration)
        .map(|(_, spelling)| spelling.as_str())
        .collect();
    for spelling in spellings {
        let mut homes: BTreeSet<String> = BTreeSet::new();
        if record.declared.contains(spelling) {
            homes.insert(record.module_path.clone());
        }
        for claim in record.imports.iter() {
            if !claim.members.iter().any(|(name, _)| name == spelling) {
                continue;
            }
            match crate::cli_run::declaration_index::resolve_cited_module(index, &claim.target) {
                Some(target) => {
                    homes.insert(target.module_path.clone());
                }
                // An import of a module the index does not carry names no declaration this join
                // can key on. It is not admitted as a member and it is not silent.
                None => findings.push(JoinFinding {
                    kind: JoinFindingKind::RosterMemberUnresolved,
                    row_type: row_type.to_string(),
                    subject: format!(
                        "{}.{roster_declaration} names `{spelling}`",
                        record.module_path
                    ),
                    detail: format!(
                        "imported from `{}`, which is absent from the index, so this member \
                         resolves to no declaration and is not admitted as one",
                        claim.target
                    ),
                }),
            }
        }
        if homes.len() > 1 {
            findings.push(JoinFinding {
                kind: JoinFindingKind::RosterMemberAmbiguous,
                row_type: row_type.to_string(),
                subject: format!(
                    "{}.{roster_declaration} names `{spelling}`",
                    record.module_path
                ),
                detail: format!(
                    "which resolves to {:?} — the join admits none of them, because accepting \
                     either would decide membership by guess",
                    homes
                ),
            });
            continue;
        }
        for home in homes {
            identities.insert((home, spelling.to_string()));
        }
    }
    (identities, findings)
}

/// Join the host roster against `RosteredRowType`, in both directions.
fn vocabulary_findings(index: &DeclarationIndex) -> Vec<JoinFinding> {
    let finding = |detail: String| JoinFinding {
        kind: JoinFindingKind::Vocabulary,
        row_type: JOIN_AUTHORITY_TYPE.to_string(),
        subject: JOIN_AUTHORITY_MODULE.to_string(),
        detail,
    };
    let Some(record) = index_get(index, JOIN_AUTHORITY_MODULE) else {
        return vec![finding(format!(
            "the join authority `{JOIN_AUTHORITY_MODULE}` is absent from the index, so nothing \
             holds this host roster to the row types the corpus declares"
        ))];
    };
    let mut out = Vec::new();
    if !record.declared.contains(JOIN_AUTHORITY_ROSTER) {
        out.push(finding(format!(
            "`{JOIN_AUTHORITY_MODULE}` declares no `{JOIN_AUTHORITY_ROSTER}`"
        )));
    }
    let here: BTreeSet<String> = ENROLLED_ROW_TYPES
        .iter()
        .map(|e| e.variant.to_string())
        .collect();
    let authored: BTreeSet<String> = record
        .decl_fields
        .get(JOIN_AUTHORITY_TYPE)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|name| record.variants.contains(name))
        .collect();
    if authored.is_empty() {
        out.push(finding(format!(
            "`{JOIN_AUTHORITY_MODULE}` declares no variants of `{JOIN_AUTHORITY_TYPE}`, so the \
             vocabulary join has nothing to compare and would pass on any host roster"
        )));
        return out;
    }
    for missing in authored.difference(&here) {
        out.push(finding(format!(
            "`{JOIN_AUTHORITY_TYPE}` declares `{missing}` and this host roster does not enrol it \
             — a rostered row type nothing joins"
        )));
    }
    for extra in here.difference(&authored) {
        out.push(finding(format!(
            "this host roster enrols `{extra}` and `{JOIN_AUTHORITY_TYPE}` does not declare it — \
             a join running with no authority"
        )));
    }
    out
}

/// Every `.dag` source under the pool roots that the declared-population walk did not account
/// for, named by source identity because it has no declaration identity to be named by.
fn unaccounted_sources(accounted: &[(String, String)]) -> Vec<JoinFinding> {
    let ws = crate::cli_run::workspace_root();
    let accounted_paths: BTreeSet<&str> = accounted.iter().map(|(_, rel)| rel.as_str()).collect();
    let mut out = Vec::new();
    for root in JOIN_POOL_ROOTS {
        let mut files = Vec::new();
        crate::cli_run::collect_dag_files_tolerant(&ws.join(root), &mut files);
        files.sort();
        for file in files {
            let rel = file
                .strip_prefix(&ws)
                .unwrap_or(&file)
                .to_string_lossy()
                .replace('\\', "/");
            if accounted_paths.contains(rel.as_str()) {
                continue;
            }
            let digest = match std::fs::read(&file) {
                Ok(bytes) => content_digest(&bytes),
                Err(e) => format!("unreadable: {e}"),
            };
            out.push(JoinFinding {
                kind: JoinFindingKind::SourceUnaccounted,
                row_type: "(no row type — the source has no accounted module)".to_string(),
                subject: format!("{rel} ({digest})"),
                detail:
                    "this source is under a declared pool root and the declared-population walk \
                     accounted for no module from it, so every join below ranges over a corpus \
                     that does not contain it. A source contributing nothing is refused by \
                     source identity rather than dropped"
                        .to_string(),
            });
        }
    }
    out
}

/// Run the join. `Err` is reserved for the declared population being unobtainable at all — a
/// state in which no verdict exists, as distinct from a verdict of "unrostered rows found".
pub fn run_rostered_row_join(index: &DeclarationIndex) -> Result<JoinReport, String> {
    let roots: Vec<String> = JOIN_POOL_ROOTS.iter().map(|r| r.to_string()).collect();
    let population = crate::coproduct_reflection::data_decl_type_rows(&roots)?;

    let mut report = JoinReport {
        sources_accounted: population.accounted_modules.len(),
        ..JoinReport::default()
    };
    report.findings.extend(vocabulary_findings(index));
    report
        .findings
        .extend(unaccounted_sources(&population.accounted_modules));

    // The declared population, bucketed by declared-type spelling ONCE, so each enrolled type is
    // answered from the same walk rather than from a fresh one per type.
    let mut by_type: BTreeMap<&str, Vec<&crate::coproduct_reflection::DataDeclTypeRow>> =
        BTreeMap::new();
    for row in population.rows.iter() {
        by_type.entry(row.type_name.as_str()).or_default().push(row);
    }

    for enrolled in ENROLLED_ROW_TYPES.iter() {
        let mut findings: Vec<JoinFinding> = Vec::new();
        let roster_record = index_get(index, enrolled.roster_module);
        let fixture_home = match roster_record {
            Some(record) => module_is_fixture_carrier(&record.module_path, &record.rel_path),
            // Undecidable without the module, and the absence is itself refused below; treat it
            // as production so a missing control roster cannot be quietly excused.
            None => false,
        };
        // ELIGIBILITY IS THE ROSTER'S OWN CARRIER CLASS, never a flag on the row: a production
        // roster ranges over production declarations, a fixture roster over fixture ones.
        let of_type: &[&crate::coproduct_reflection::DataDeclTypeRow] = by_type
            .get(enrolled.type_name)
            .map(|rows| rows.as_slice())
            .unwrap_or(&[]);
        let (declared, excluded): (
            Vec<&crate::coproduct_reflection::DataDeclTypeRow>,
            Vec<&crate::coproduct_reflection::DataDeclTypeRow>,
        ) = of_type.iter().copied().partition(|row| {
            module_is_fixture_carrier(&row.module_path, &row.rel_path) == fixture_home
        });

        let rostered: BTreeSet<(String, String)> = match roster_record {
            None => {
                findings.push(JoinFinding {
                    kind: JoinFindingKind::RosterModuleAbsent,
                    row_type: enrolled.variant.to_string(),
                    subject: enrolled.roster_module.to_string(),
                    detail: format!(
                        "the roster module for `{}` is absent from the index, so no membership \
                         can be read and every declared row of that type is unanswered",
                        enrolled.type_name
                    ),
                });
                BTreeSet::new()
            }
            Some(record) if !record.declared.contains(enrolled.roster_declaration) => {
                findings.push(JoinFinding {
                    kind: JoinFindingKind::RosterDeclarationAbsent,
                    row_type: enrolled.variant.to_string(),
                    subject: format!("{}.{}", enrolled.roster_module, enrolled.roster_declaration),
                    detail: "the roster module declares no such roster, so this join would \
                             otherwise report every declared row as unrostered or none at all"
                        .to_string(),
                });
                BTreeSet::new()
            }
            Some(record) => {
                let (identities, member_findings) = rostered_identities(
                    index,
                    record,
                    enrolled.roster_declaration,
                    enrolled.variant,
                );
                findings.extend(member_findings);
                identities
            }
        };

        let mut checked = 0usize;
        for row in declared.iter() {
            checked += 1;
            // THE KEY IS THE DECLARATION IDENTITY, module path included. A bare name is not an
            // identity: see `rostered_identities`.
            if rostered.contains(&(row.module_path.clone(), row.decl_name.clone())) {
                continue;
            }
            findings.push(JoinFinding {
                kind: JoinFindingKind::DeclaredNotRostered,
                row_type: enrolled.variant.to_string(),
                subject: format!("{}.{} ({})", row.module_path, row.decl_name, row.rel_path),
                detail: format!(
                    "declared as `{}` and not named by `{}.{}`, so it is absent from every \
                     projection derived from that roster",
                    enrolled.type_name, enrolled.roster_module, enrolled.roster_declaration
                ),
            });
        }
        // THE SUBJECT IS ACCOUNTED FOR IN BOTH DIRECTIONS. Every row carrying this type spelling
        // is either checked against the roster or excluded by carrier class, and the two must sum
        // to the population that carried the spelling: a row that reached neither bucket is a
        // subject narrowed mid-run, which is the state a count-based join reports as clean.
        if checked != declared.len() || checked + excluded.len() != of_type.len() {
            findings.push(JoinFinding {
                kind: JoinFindingKind::CheckedNarrowerThanDeclared,
                row_type: enrolled.variant.to_string(),
                subject: enrolled.type_name.to_string(),
                detail: format!(
                    "checked {checked} and excluded {} of {} declaration(s) carrying this type \
                     spelling ({} eligible)",
                    excluded.len(),
                    of_type.len(),
                    declared.len()
                ),
            });
        }
        if declared.is_empty() {
            findings.push(JoinFinding {
                kind: JoinFindingKind::DeclaredPopulationEmpty,
                row_type: enrolled.variant.to_string(),
                subject: enrolled.type_name.to_string(),
                detail: "no declaration of this type was discovered in the declared population, \
                         so this row type's join ranged over nothing and its green means only \
                         that the subject vanished"
                    .to_string(),
            });
        }
        report.counts.push(JoinCounts {
            row_type: enrolled.variant.to_string(),
            population_of_type: of_type.len(),
            excluded_other_carrier: excluded.len(),
            declared: declared.len(),
            rostered: rostered.len(),
            checked,
            fixture_home,
        });

        // THE CONTROL MUST FIRE. A fixture-homed enrolled type exists to produce a refusal on
        // every run; producing none is the green-by-construction state (§4b), and it is reported
        // as a failure of the JOIN, not of the fixture.
        if fixture_home {
            // A control fires by producing the refusal it was authored to produce, and nothing
            // else counts: an EMPTY declared population or an absent roster would otherwise be
            // read as the control having fired, which is the same silence one indirection out.
            let (fired, other): (Vec<JoinFinding>, Vec<JoinFinding>) = findings
                .into_iter()
                .partition(|f| f.kind == JoinFindingKind::DeclaredNotRostered);
            if fired.is_empty() {
                report.findings.push(JoinFinding {
                    kind: JoinFindingKind::ControlDidNotFire,
                    row_type: enrolled.variant.to_string(),
                    subject: format!("{}.{}", enrolled.roster_module, enrolled.roster_declaration),
                    detail: "the enrolled control reported no unrostered declaration, so this \
                             run establishes only that the join is silent — the discriminating \
                             red is gone"
                        .to_string(),
                });
            }
            report.control_findings.extend(fired);
            report.findings.extend(other);
        } else {
            report.findings.extend(findings);
        }
    }
    Ok(report)
}
