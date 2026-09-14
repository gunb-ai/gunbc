//! THE JOINT-CLAIM JOIN — checkpoint 1 of `gunbc.plans.joint_incompatibility_claim_join`.
//!
//! WHAT IT ANSWERS. Two changes that are each green on their own head, textually disjoint and
//! merge-clean can be jointly incompatible: one SHRINKS a module's export surface and the other
//! ADDS an import claim on the entry that left. Each change's CI compiles its head against `main`
//! at run start, never against its sibling, so the union is first compiled by whoever lands after
//! both. This instrument computes, per change, the two set differences the join needs — surface
//! entries removed, import claims added (and retired) — and intersects them across a set of
//! changes. It is an INSTRUMENT: it reports, it exits by what it found, and nothing gates on it.
//!
//! THE CUT IS THE SURFACE, NOT THE EDIT. The removed side is `import_surface_has`'s own union —
//! `declared ∪ variants ∪ reexported` — taken base-minus-head, because of the three 2026-09-10
//! instances one was a deletion, one a rename and one a MOVE to another module (the declaration
//! survived; its old home stopped exporting it). Keying on the kind of edit would have caught one
//! of three; a dropped re-export is the fourth kind the same surface covers, and has its control.
//!
//! THE EXCLUSION, AND WHY IT IS NOT A KNOB. A claim is joined only if it is STILL LIVE when the
//! surface entry leaves: not retired by the removing change's own diff, and — in retrospective
//! mode, where the subjects are a history rather than a set of open siblings — not retired by any
//! change that landed between the two. Both are one statement: the join asks about the MERGED
//! tree, and a retired claim is not in that tree. Without it the join over fourteen days of `main`
//! is a superset that is never wrong and never useful, DESIGN §5's absorbing fallback read from
//! the other side; `raw_candidates` is still reported beside `findings` so the width the exclusion
//! removes is a number, not a memory. An open-PR-set join needs only the first half, because a
//! merged change is not an open sibling.
//!
//! WHAT IT REUSES RATHER THAN RE-DERIVES. Records come from `declaration_index::record_from_module`
//! through `namespace_wave_admission::base_records`, the same parse the wave wall reconstructs a
//! baseline from; touched paths come from `diff_sides` over `git diff --name-status -z -M`, which
//! is the LISTING that establishes which side a path exists on, so a `git show` failure here is a
//! refusal and never evidence of absence.
//!
//! RETROSPECTIVE MODE. `joint_claim_join <since>..<tip> [--window-seconds N]` walks the
//! first-parent commits of the range, takes each commit against its first parent as one subject,
//! and joins every ordered pair whose commit times are within the window. Its ground truth is the
//! `ImportMemberAbsent` reds the declarations rider reported on `main` in the same range.
#![allow(clippy::disallowed_macros)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::ExitCode;

use v1_compiler::cli_run::declaration_index::ModuleDeclarationRecord;
use v1_compiler::cli_run::namespace_wave_admission::{
    base_records, diff_sides, git_stdout, in_sweep_scope,
};

/// Which of the three export-surface sets a removed name left. Kept so a dropped re-export reads
/// as what it is rather than as a phantom deletion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SurfaceKind {
    Declared,
    Variant,
    Reexported,
}

pub fn surface_kind_label(kind: SurfaceKind) -> &'static str {
    match kind {
        SurfaceKind::Declared => "declared",
        SurfaceKind::Variant => "variant",
        SurfaceKind::Reexported => "reexported",
    }
}

/// One `import target { member }` claim authored in `module`, located at its member.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImportClaimSite {
    pub module: String,
    pub target: String,
    pub member: String,
    pub rel_path: String,
    pub offset: i64,
}

/// The identity a claim keeps across edits: `(module, target, member)`. Offsets shift under any
/// edit above the line, so they are not part of it.
pub type ClaimKey = (String, String, String);

/// A free function rather than a method so it is citable as a `DeclarationRef` (no `impl` block).
pub fn claim_key(c: &ImportClaimSite) -> ClaimKey {
    (c.module.clone(), c.target.clone(), c.member.clone())
}

/// One change's delta: what its head no longer exports, and which claims it added or retired.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClaimDelta {
    pub removed_surface: BTreeMap<(String, String), SurfaceKind>,
    pub added_claims: Vec<ImportClaimSite>,
    pub retired_claims: Vec<ImportClaimSite>,
}

fn surface_of(record: &ModuleDeclarationRecord) -> BTreeMap<String, SurfaceKind> {
    let mut out = BTreeMap::new();
    for n in &record.reexported {
        out.insert(n.clone(), SurfaceKind::Reexported);
    }
    for n in &record.variants {
        out.insert(n.clone(), SurfaceKind::Variant);
    }
    for n in &record.declared {
        out.insert(n.clone(), SurfaceKind::Declared);
    }
    out
}

fn claim_sites(records: &[ModuleDeclarationRecord]) -> Vec<ImportClaimSite> {
    let mut out = Vec::new();
    for record in records {
        for claim in &record.imports {
            for (member, location) in &claim.members {
                out.push(ImportClaimSite {
                    module: record.module_path.clone(),
                    target: claim.target.clone(),
                    member: member.clone(),
                    rel_path: location.file.clone(),
                    offset: location.offset,
                });
            }
        }
    }
    out
}

/// The delta of one change, from the records of ONLY the files its diff touched, on both sides.
/// An untouched file has the same record on both sides and contributes nothing to a difference,
/// so passing it is harmless and omitting it is the whole economy.
pub fn claim_delta(
    base: &[ModuleDeclarationRecord],
    head: &[ModuleDeclarationRecord],
) -> ClaimDelta {
    let head_by_module: BTreeMap<&str, &ModuleDeclarationRecord> =
        head.iter().map(|r| (r.module_path.as_str(), r)).collect();
    let mut removed_surface = BTreeMap::new();
    for record in base {
        let head_surface = head_by_module
            .get(record.module_path.as_str())
            .map(|r| surface_of(r))
            .unwrap_or_default();
        for (name, kind) in surface_of(record) {
            if !head_surface.contains_key(&name) {
                removed_surface.insert((record.module_path.clone(), name), kind);
            }
        }
    }
    let base_claims = claim_sites(base);
    let head_claims = claim_sites(head);
    let base_keys: BTreeSet<_> = base_claims.iter().map(claim_key).collect();
    let head_keys: BTreeSet<_> = head_claims.iter().map(claim_key).collect();
    ClaimDelta {
        removed_surface,
        added_claims: head_claims
            .into_iter()
            .filter(|c| !base_keys.contains(&claim_key(c)))
            .collect(),
        retired_claims: base_claims
            .into_iter()
            .filter(|c| !head_keys.contains(&claim_key(c)))
            .collect(),
    }
}

/// One change under the join, named by whatever identity the caller has — a commit sha, a PR.
#[derive(Debug, Clone)]
pub struct JointClaimSubject {
    pub id: String,
    pub delta: ClaimDelta,
}

/// The typed, located finding: the entry that left, which surface it left, who removed it, and
/// the site that claims it. One finding per (removed entry × claiming site), never per pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JointClaimFinding {
    pub removed_module: String,
    pub removed_name: String,
    pub removed_from: SurfaceKind,
    pub removing_subject: String,
    pub claiming_subject: String,
    pub claim: ImportClaimSite,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JointClaimJoin {
    /// Hits before the retirement exclusion — the candidate list, reported so the width the
    /// exclusion removed stays visible.
    pub raw_candidates: usize,
    pub findings: Vec<JointClaimFinding>,
}

/// Join every ordered pair `(remover, claimer)` the predicate admits. `claim_retired_between`
/// answers whether some OTHER subject, landing between the pair, retired the claim key: true in
/// retrospective mode when a third commit fixed the importer before the entry left; always false
/// over a set of open siblings, where nothing has landed between anything.
pub fn joint_claim_join(
    subjects: &[JointClaimSubject],
    admits_pair: &dyn Fn(&JointClaimSubject, &JointClaimSubject) -> bool,
    claim_retired_between: &dyn Fn(&JointClaimSubject, &JointClaimSubject, &ClaimKey) -> bool,
) -> JointClaimJoin {
    let mut out = JointClaimJoin::default();
    for remover in subjects {
        if remover.delta.removed_surface.is_empty() {
            continue;
        }
        let retired: BTreeSet<_> = remover.delta.retired_claims.iter().map(claim_key).collect();
        for claimer in subjects {
            if claimer.id == remover.id || !admits_pair(remover, claimer) {
                continue;
            }
            for claim in &claimer.delta.added_claims {
                let entry = (claim.target.clone(), claim.member.clone());
                let Some(kind) = remover.delta.removed_surface.get(&entry) else {
                    continue;
                };
                out.raw_candidates += 1;
                let key = claim_key(claim);
                if retired.contains(&key) || claim_retired_between(remover, claimer, &key) {
                    continue;
                }
                out.findings.push(JointClaimFinding {
                    removed_module: entry.0,
                    removed_name: entry.1,
                    removed_from: *kind,
                    removing_subject: remover.id.clone(),
                    claiming_subject: claimer.id.clone(),
                    claim: claim.clone(),
                });
            }
        }
    }
    out
}

pub fn render_finding(f: &JointClaimFinding) -> String {
    format!(
        "joint_claim_join: FINDING `{}` no longer exports `{}` ({}) at {}; claimed at {} by `{}` \
         importing `{}` from `{}` ({}:{})",
        f.removed_module,
        f.removed_name,
        surface_kind_label(f.removed_from),
        f.removing_subject,
        f.claiming_subject,
        f.claim.module,
        f.claim.member,
        f.claim.target,
        f.claim.rel_path,
        f.claim.offset
    )
}

/// One commit as a subject: its diff against its first parent, both sides parsed from git.
struct CommitSubject {
    sha: String,
    time: i64,
    subject_line: String,
}

fn commit_delta(workspace: &Path, sha: &str) -> Result<ClaimDelta, String> {
    let parent = format!("{sha}^");
    let name_status = git_stdout(
        workspace,
        &["diff", "--name-status", "-z", "-M", &parent, sha],
    )?;
    let (head_touched, base_side) = diff_sides(&name_status);
    let mut base = Vec::new();
    for rel in base_side.iter().filter(|p| in_sweep_scope(p)) {
        let content = git_stdout(workspace, &["show", &format!("{parent}:{rel}")])
            .map_err(|e| format!("{sha}: cannot read {rel} at its parent ({e})"))?;
        base.extend(base_records(rel, &content)?);
    }
    let mut head = Vec::new();
    for rel in head_touched.iter().filter(|p| in_sweep_scope(p)) {
        let content = git_stdout(workspace, &["show", &format!("{sha}:{rel}")])
            .map_err(|e| format!("{sha}: cannot read {rel} at the commit ({e})"))?;
        head.extend(base_records(rel, &content)?);
    }
    Ok(claim_delta(&base, &head))
}

fn usage() -> ExitCode {
    eprintln!(
        "usage: joint_claim_join <since>..<tip> [--window-seconds N]   (default window 259200)"
    );
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut range = None;
    let mut window_seconds: i64 = 3 * 24 * 3600;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--window-seconds" => {
                let Some(v) = args.get(i + 1).and_then(|v| v.parse::<i64>().ok()) else {
                    return usage();
                };
                window_seconds = v;
                i += 2;
            }
            other if other.contains("..") => {
                range = Some(other.to_string());
                i += 1;
            }
            _ => return usage(),
        }
    }
    let Some(range) = range else { return usage() };
    let workspace = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("joint_claim_join: current_dir: {e}");
            return ExitCode::from(1);
        }
    };
    let listing = match git_stdout(
        &workspace,
        &[
            "log",
            "--first-parent",
            "--reverse",
            "--format=%H %ct %s",
            &range,
        ],
    ) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("joint_claim_join: the range does not resolve ({e})");
            return ExitCode::from(1);
        }
    };
    let mut commits = Vec::new();
    for line in listing.lines() {
        let mut parts = line.splitn(3, ' ');
        let (Some(sha), Some(time)) = (parts.next(), parts.next()) else {
            continue;
        };
        let Ok(time) = time.parse::<i64>() else {
            continue;
        };
        commits.push(CommitSubject {
            sha: sha.to_string(),
            time,
            subject_line: parts.next().unwrap_or("").to_string(),
        });
    }
    let mut subjects = Vec::new();
    let mut times: BTreeMap<String, i64> = BTreeMap::new();
    let mut lines: BTreeMap<String, String> = BTreeMap::new();
    for c in &commits {
        match commit_delta(&workspace, &c.sha) {
            Ok(delta) => {
                times.insert(c.sha.clone(), c.time);
                lines.insert(c.sha.clone(), c.subject_line.clone());
                subjects.push(JointClaimSubject {
                    id: c.sha.clone(),
                    delta,
                });
            }
            Err(e) => {
                // A subject whose delta is unobservable makes the join unobservable: refuse, never
                // continue over a quieter population.
                eprintln!("joint_claim_join: NOT EVALUATED — {e}");
                return ExitCode::from(1);
            }
        }
    }
    let with_removals = subjects
        .iter()
        .filter(|s| !s.delta.removed_surface.is_empty())
        .count();
    let with_added = subjects
        .iter()
        .filter(|s| !s.delta.added_claims.is_empty())
        .count();
    let join = joint_claim_join(
        &subjects,
        &|a, b| (times[&a.id] - times[&b.id]).abs() <= window_seconds,
        &|a, b, key| {
            let (lo, hi) = (
                times[&a.id].min(times[&b.id]),
                times[&a.id].max(times[&b.id]),
            );
            subjects.iter().any(|s| {
                let t = times[&s.id];
                t > lo && t < hi && s.delta.retired_claims.iter().any(|c| claim_key(c) == *key)
            })
        },
    );
    for f in &join.findings {
        eprintln!(
            "{}\n    remover: {}\n    claimer: {}",
            render_finding(f),
            lines[&f.removing_subject],
            lines[&f.claiming_subject]
        );
    }
    eprintln!(
        "joint_claim_join: range={range} subjects={} with_removals={with_removals} \
         with_added_claims={with_added} window_seconds={window_seconds} raw_candidates={} findings={}",
        subjects.len(),
        join.raw_candidates,
        join.findings.len()
    );
    if join.findings.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(3)
    }
}

// THE FIXTURE BOUNDARY IS WHERE THE RED IS AUTHORABLE (DESIGN §4b). Each arm below hands the real
// parser real source and asks the real delta and join; nothing is hand-built into a record.
#[cfg(test)]
mod tests {
    use super::*;

    fn records(rel: &str, src: &str) -> Vec<ModuleDeclarationRecord> {
        base_records(rel, src).unwrap_or_else(|e| panic!("fixture must parse: {e}"))
    }

    fn subject(id: &str, base: &[(&str, &str)], head: &[(&str, &str)]) -> JointClaimSubject {
        let base: Vec<_> = base.iter().flat_map(|(r, s)| records(r, s)).collect();
        let head: Vec<_> = head.iter().flat_map(|(r, s)| records(r, s)).collect();
        JointClaimSubject {
            id: id.to_string(),
            delta: claim_delta(&base, &head),
        }
    }

    const CARRIER_BOTH: &str =
        "module fx.carrier\n\ntype Read\n  = DurableOriginRead\n  | DisposableCacheRead\n";
    const CARRIER_ONE: &str = "module fx.carrier\n\ntype Read\n  = DurableOriginRead\n";
    const WITNESS_CLAIMS: &str =
        "module fx.witness\n\nimport fx.carrier { DisposableCacheRead }\n\ndata x: Int = 1\n";
    const WITNESS_SILENT: &str = "module fx.witness\n\ndata x: Int = 1\n";

    fn all(_: &JointClaimSubject, _: &JointClaimSubject) -> bool {
        true
    }

    fn never_between(
        _: &JointClaimSubject,
        _: &JointClaimSubject,
        _: &(String, String, String),
    ) -> bool {
        false
    }

    /// POSITIVE CONTROL, the #10933 × #10934 shape: a variant deleted on one side, imported on the
    /// other, in different files. One finding, naming the variant surface and the claiming site.
    #[test]
    fn variant_deleted_by_one_change_and_imported_by_another_is_one_located_finding() {
        let remover = subject(
            "remover",
            &[("dag/fx/carrier.dag", CARRIER_BOTH)],
            &[("dag/fx/carrier.dag", CARRIER_ONE)],
        );
        let claimer = subject("claimer", &[], &[("dag/fx/witness.dag", WITNESS_CLAIMS)]);
        let join = joint_claim_join(&[remover, claimer], &all, &never_between);
        assert_eq!(join.raw_candidates, 1);
        assert_eq!(join.findings.len(), 1, "{join:?}");
        let f = &join.findings[0];
        assert_eq!(
            (f.removed_module.as_str(), f.removed_name.as_str()),
            ("fx.carrier", "DisposableCacheRead")
        );
        assert_eq!(f.removed_from, SurfaceKind::Variant);
        assert_eq!(f.removing_subject, "remover");
        assert_eq!(f.claiming_subject, "claimer");
        assert_eq!(f.claim.module, "fx.witness");
        assert_eq!(f.claim.rel_path, "dag/fx/witness.dag");
        assert!(
            f.claim.offset > 0,
            "the claim is located at its member, not at the file start"
        );
    }

    /// THE DISCRIMINATING RED FOR THE EXCLUSION. Same removal, but the remover's own diff also
    /// retires the claim — the merged tree carries no dangling import. The candidate is counted
    /// and the finding is not emitted; a join without the exclusion would report it.
    #[test]
    fn a_remover_that_retires_the_claim_itself_is_a_candidate_and_not_a_finding() {
        let remover = subject(
            "remover",
            &[
                ("dag/fx/carrier.dag", CARRIER_BOTH),
                ("dag/fx/witness.dag", WITNESS_CLAIMS),
            ],
            &[
                ("dag/fx/carrier.dag", CARRIER_ONE),
                ("dag/fx/witness.dag", WITNESS_SILENT),
            ],
        );
        let claimer = subject(
            "claimer",
            &[("dag/fx/witness.dag", WITNESS_SILENT)],
            &[("dag/fx/witness.dag", WITNESS_CLAIMS)],
        );
        let join = joint_claim_join(&[remover, claimer], &all, &never_between);
        assert_eq!(join.raw_candidates, 1, "the superset still sees it");
        assert!(join.findings.is_empty(), "{join:?}");
    }

    /// THE RETROSPECTIVE'S SECOND EXCLUSION. The two 09-05/09-08 artefacts the first run of this
    /// instrument reported were both a THIRD commit retiring the claim between the pair. Handed a
    /// predicate that says so, the candidate is counted and the finding is not emitted.
    #[test]
    fn a_claim_retired_by_a_third_change_between_the_pair_is_a_candidate_and_not_a_finding() {
        let remover = subject(
            "remover",
            &[("dag/fx/carrier.dag", CARRIER_BOTH)],
            &[("dag/fx/carrier.dag", CARRIER_ONE)],
        );
        let claimer = subject("claimer", &[], &[("dag/fx/witness.dag", WITNESS_CLAIMS)]);
        let join = joint_claim_join(&[remover, claimer], &all, &|_, _, key| {
            key.2 == "DisposableCacheRead"
        });
        assert_eq!(join.raw_candidates, 1);
        assert!(join.findings.is_empty(), "{join:?}");
    }

    /// A dropped re-export: NO declaration is deleted anywhere; the remover drops its own import,
    /// so the name leaves its `reexported` surface, and a sibling imports it from there. No 09-10
    /// instance had this shape (instance 2 was a MOVE, which is `declared` leaving); the surface
    /// covers it and this is its control.
    #[test]
    fn a_dropped_reexport_is_a_finding_that_names_the_reexported_surface() {
        let actuator_before =
            "module fx.actuator\n\nimport fx.rest { commit_ambiguous }\n\ndata y: Int = 2\n";
        let actuator_after = "module fx.actuator\n\ndata y: Int = 2\n";
        let mint = "module fx.mint\n\nimport fx.actuator { commit_ambiguous }\n\ndata z: Int = 3\n";
        let remover = subject(
            "remover",
            &[("dag/fx/actuator.dag", actuator_before)],
            &[("dag/fx/actuator.dag", actuator_after)],
        );
        let claimer = subject("claimer", &[], &[("dag/fx/mint.dag", mint)]);
        let join = joint_claim_join(&[remover, claimer], &all, &never_between);
        assert_eq!(join.findings.len(), 1, "{join:?}");
        assert_eq!(join.findings[0].removed_from, SurfaceKind::Reexported);
        assert_eq!(join.findings[0].removed_name, "commit_ambiguous");
    }

    /// The #10865 shape: a rename is a removal on its old-name half, and that half is what fires.
    /// A MOVE (#10923: the fn re-homed to another module) is the same delta on the old home.
    #[test]
    fn a_rename_fires_on_the_old_name_and_not_on_the_new_one() {
        let before = "module fx.density\n\nfn grain_admits_single_cabinet() -> Bool { true }\n";
        let after = "module fx.density\n\nfn grain_offers_single_cabinet() -> Bool { true }\n";
        let census = "module fx.census\n\nimport fx.density { grain_admits_single_cabinet }\n\ndata w: Int = 4\n";
        let remover = subject(
            "remover",
            &[("dag/fx/density.dag", before)],
            &[("dag/fx/density.dag", after)],
        );
        let claimer = subject("claimer", &[], &[("dag/fx/census.dag", census)]);
        let join = joint_claim_join(&[remover, claimer], &all, &never_between);
        assert_eq!(join.findings.len(), 1, "{join:?}");
        assert_eq!(join.findings[0].removed_name, "grain_admits_single_cabinet");
        assert_eq!(join.findings[0].removed_from, SurfaceKind::Declared);
    }

    /// The pair predicate is the caller's: a pair it refuses is neither a candidate nor a finding.
    #[test]
    fn a_pair_outside_the_admitted_window_is_not_joined() {
        let remover = subject(
            "remover",
            &[("dag/fx/carrier.dag", CARRIER_BOTH)],
            &[("dag/fx/carrier.dag", CARRIER_ONE)],
        );
        let claimer = subject("claimer", &[], &[("dag/fx/witness.dag", WITNESS_CLAIMS)]);
        let join = joint_claim_join(&[remover, claimer], &|_, _| false, &never_between);
        assert_eq!(join.raw_candidates, 0);
        assert!(join.findings.is_empty());
    }

    /// An import that was already on the claimer's base is not an ADDED claim: the remover's own
    /// CI would have seen it on `main`, so the join must not report it a second time.
    #[test]
    fn a_claim_already_present_at_the_claimer_base_is_not_added() {
        let remover = subject(
            "remover",
            &[("dag/fx/carrier.dag", CARRIER_BOTH)],
            &[("dag/fx/carrier.dag", CARRIER_ONE)],
        );
        let claimer = subject(
            "claimer",
            &[("dag/fx/witness.dag", WITNESS_CLAIMS)],
            &[("dag/fx/witness.dag", WITNESS_CLAIMS)],
        );
        let join = joint_claim_join(&[remover, claimer], &all, &never_between);
        assert_eq!(join.raw_candidates, 0);
        assert!(join.findings.is_empty());
    }
}
