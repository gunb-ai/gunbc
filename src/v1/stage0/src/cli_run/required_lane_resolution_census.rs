//! THE REQUIRED-LANE RESOLUTION CENSUS'S THREE COMPILER QUERIES.
//!
//! The question the census asks is DESIGN section 3's standing warning read as a measurement:
//! outside the required gate the substrate still refuses and nothing asks it to, so a
//! product-layer dependent can stop resolving, stay broken, and let every required lane report
//! SUCCESS. On 2026-09-20 that was the live state -- `gunbc.fleet.site_pxe_edge_converge` did not
//! resolve on main while all four required lanes were green (gunbc#11607) -- and the question
//! "which modules can do that" had no instrument. This file answers the three population
//! queries the `.dag` census (`gunbc.required_lane_resolution_census`) folds over; the census,
//! not this file, decides which lanes exist and what each one resolves, and it reads that roster
//! from `gunbc.witness.compiler_gate_workflow` so a job the workflow emits cannot go unclassified.
//!
//! "RESOLVE" MEANS ONE THING HERE: the module's source is in the source set a lane's compile
//! transaction hands to `compile_to_resolved` under the Strict gate. For the floor that is the
//! prepared subject (`assemble_prepared_subject_closure` over the nominal seeds); for a
//! `gunbc run --entry` step it is the entry's loader closure (`load_sources_for_entry_with_pool`,
//! the same call `resolve_entry_graph` makes). A module reached only by name in a `run:` line
//! is NOT resolved by that step unless the entry's closure reaches it, and that is the whole
//! finding.
//!
//! NOMINAL, NOT DIFF-SCOPED, AND DELIBERATELY SO. The floor also seeds its subject from the
//! run's diff -- changed witnesses, touched entries, arm-set consumers -- so a module a PR
//! touches is resolved on THAT PR's run. The failure this census exists to expose is the other
//! case: a module nobody touched, broken by a rename in a module it depends on, on a PR whose
//! diff never seeds it. The population that can break that way is exactly the complement of
//! the NOMINAL subject, which is why the census reads the floor's seeds through
//! `required_floor_nominal_subject_seeds` -- the one producer the floor itself consumes -- and
//! adds no diff seeds.
//!
//! EACH QUERY REFUSES RATHER THAN NARROWS. An index that cannot be built, a seed roster that
//! cannot be decoded, an entry whose closure cannot be loaded: each is a typed refusal the
//! `.dag` census must decide on. None returns an empty population as if it had observed one.

use super::*;

/// One population of module identities, or the reason it could not be established.
pub enum ModuleIdentityPopulation {
    Observed { modules: Vec<String> },
    Refused { cause: String },
}

/// The denominator: every module identity the source-root ingestion admits. This is the SAME
/// producer `claim_executor --required-ci` prints as `admitted-module-identities`, so the census
/// and the required run cannot disagree about what "every module under dag/ and src/v2/" means.
/// A `.dag` file with no `module` header is not an identity and is outside this universe.
pub fn source_root_ingest_module_identities(source_roots: &[String]) -> ModuleIdentityPopulation {
    match source_root_ingest_module_identities_for_ci(source_roots) {
        Ok(modules) => ModuleIdentityPopulation::Observed { modules },
        Err(cause) => ModuleIdentityPopulation::Refused { cause },
    }
}

/// The floor's NOMINAL prepared subject: the gate prefix closure plus the runtime-authority,
/// authored-module and wet-schedule seeds, assembled by `assemble_prepared_subject_closure`
/// exactly as `run_required_floor` assembles it before adding a run's diff seeds. Read-and-fold
/// only -- no strict preparation is paid here, because membership in the subject is the
/// question, not whether the subject then resolves.
pub fn required_floor_nominal_subject_module_identities(
    source_roots: &[String],
) -> ModuleIdentityPopulation {
    let index = process_shared_index(source_roots);
    let seeds = match required_floor_nominal_subject_seeds(source_roots, &index) {
        Ok(seeds) => seeds,
        Err(cause) => {
            return ModuleIdentityPopulation::Refused {
                cause: format!("nominal subject seeds: {cause}"),
            }
        }
    };
    let module_seeds = required_floor_nominal_closure_module_seeds(
        &seeds.required_gate_authored_modules,
        &seeds.local_repo_wet_schedule_rows,
    );
    let subject = match assemble_prepared_subject_closure(
        source_roots,
        &floor_prepared_subject_exclusions(),
        Some((&index, &seeds.required_gate_prefixes, &module_seeds)),
    ) {
        Ok(subject) => subject,
        Err(cause) => {
            return ModuleIdentityPopulation::Refused {
                cause: format!("nominal subject assembly: {cause}"),
            }
        }
    };
    let mut modules: Vec<String> = subject
        .inventory
        .iter()
        .map(|view| view.module_path.clone())
        .collect();
    modules.sort();
    modules.dedup();
    ModuleIdentityPopulation::Observed { modules }
}

/// THE TYPED ADMISSION ANSWER FOR ONE IDENTITY. The census's old receipt reported "never
/// admitted" only as ABSENCE from both the admitted and the unreached lines -- an inferred
/// exclusion a reader had to derive themselves (DESIGN section 5: every path fails or reports
/// with a typed, located diagnostic, not silence). This arm makes the state itself the carrier:
/// a queried identity the ingestion does not know PRIVES its state and its evidence, it does not
/// go quiet. `IndexRefused` exists so an ingestion failure is never misread as a never-admitted
/// verdict -- an unobserved identity is not an absent one.
/// THE OUTCOME OF THE DELETION ATTRIBUTION, three-way so an answer and an ignorance are never
/// the same printed value (DESIGN section 5: every degradation typed, located and countable).
/// A flat `Option<String>` conflated "the history has no such commit" (an ANSWER, and the reason
/// a never-existed identity still has a verdict) with "git could not be consulted" (IGNORANCE,
/// which must not be read as an answer -- that is the absence-as-evidence problem this carrier
/// exists to remove).
pub enum DeletionAttribution {
    /// A commit was found whose diff removed the module header, verified to have had it in its
    /// parent. Derived, never hand-written.
    Attributed { commit: String },
    /// The probes ran and answered: the reachable history contains no commit that DELETED a
    /// file declaring this header -- the identity never existed here. The verdict stands
    /// without an attribution. `probed_commits` says how much was consulted: the reachable
    /// commit count when no same-stem file was ever deleted, or the number of same-stem
    /// candidates examined (and rejected for declaring a different module) otherwise.
    NoDeletionInHistory { probed_commits: usize },
    /// A probe could not be consulted, so attribution is UNKNOWN, not empty. The verdict stands
    /// only for the tree at the run's SHA; how the identity disappeared is unobserved.
    ProbeRefused { cause: String },
}

pub enum ModuleAdmission {
    /// The ingestion admits the identity: some file under the source roots declares it.
    Admitted { module: String },
    /// No file in the tree at the run's SHA declares the identity. `run_sha` is the git answer
    /// for HEAD -- if git could not produce it the query refuses instead (`EvidenceRefused`);
    /// the instrument never prints a made-up SHA as evidence.
    NeverAdmitted {
        module: String,
        run_sha: String,
        attribution: DeletionAttribution,
    },
    /// The identity is absent AND the run's own SHA could not be observed: the verdict would
    /// claim evidence that was never observed, so the query refuses instead. Distinct from
    /// `IndexRefused` (the ingestion itself failed) so the two failures stay countable apart.
    EvidenceRefused { module: String, cause: String },
    /// The ingestion itself refused, so the identity's state was never observed. The census
    /// must decide on this arm; reading it as never-admitted would be absence-as-evidence
    /// smuggled back in through the error path.
    IndexRefused { module: String, cause: String },
}

/// THE OUTCOME OF ONE GIT PROBE, typed so an ABSENCE and a FAILURE are never the same value.
/// `Absent` means git ran the query and returned no match -- that is an answer. `ProbeFailed`
/// means git could not be asked (no repo, no binary, a broken pipe) -- that is ignorance, and
/// reading it as an absence would smuggle absence-as-evidence back in through the error path
/// (DESIGN section 5: every degradation is typed, located and countable; no plausible output is
/// fabricated).
pub enum GitProbe {
    /// git ran and produced this (possibly empty after trimming) stdout.
    Ran(String),
    /// git could not be run or refused: the reason it could not answer.
    ProbeFailed(String),
}

fn git_probe(args: &[&str]) -> GitProbe {
    match std::process::Command::new("git").args(args).output() {
        Err(why) => {
            GitProbe::ProbeFailed(format!("git {} could not be run: {why}", args.join(" ")))
        }
        Ok(out) => {
            if out.status.success() {
                GitProbe::Ran(String::from_utf8_lossy(&out.stdout).trim().to_string())
            } else {
                GitProbe::ProbeFailed(format!(
                    "git {} exited {}: {}",
                    args.join(" "),
                    out.status,
                    String::from_utf8_lossy(&out.stderr).trim()
                ))
            }
        }
    }
}

// THE DELETION ATTRIBUTION SEARCH, bounded by the file-name convention rather than by a
// whole-tree content pickaxe. THE OLD PROBE was `git log -1 -S "module <id>"` over the whole
// tree: measured at 60-72s LOCAL for a missing identity (PD7: over 600s on the remote runner,
// where it TIMED OUT), because a `-S` miss walks every reachable diff. That cost makes the
// never-existent arm unusable on any scheduled lane. This query is path-bounded instead:
// `git log -m --full-history --diff-filter=D -- '*/<leaf>.dag'` costs ~2s for a hit AND for a
// miss (measured on a 3.6k-commit checkout). BOUND, STATED: the search space is files named
// <leaf>.dag, where <leaf> is the identity's last dot-segment -- the repo's naming convention
// (2993/2993 .dag files under dag/ and src/v2 match it), NOT a compiler-enforced invariant.
// A hit is NOT taken on the name alone: the deleted file's PARENT BLOB must declare exactly
// `git show <commit>^:<path>` must declare exactly `module <identity>` (the parent-blob check
// below), so a same-stem file belonging to another module cannot be attributed here. A module
// living at a file named after a different leaf evades BOTH arms of this search and prints
// NoDeletionInHistory -- the bound of this instrument, not an invented fact; the content pickaxe
// (`git log -S`) is the exhaustive fallback and its measured cost is recorded above so nobody
// re-adds it blind.
fn deletion_attribution(identity: &str) -> Result<DeletionAttribution, String> {
    let leaf = identity.rsplit('.').next().unwrap_or(identity);
    if leaf.is_empty() {
        return Err(format!(
            "identity '{identity}' has no file-name leaf to search history by"
        ));
    }
    let pathspec = format!("*/{leaf}.dag");
    // -m --full-history: without these two, path simplification HIDES deletions that sit on
    // merge commits -- the same silence `git log --follow` is warned about in this repo's
    // drift probes. A hidden deletion would print as "never existed": absence-as-evidence.
    let probe = git_probe(&[
        "log",
        "-m",
        "--full-history",
        "--diff-filter=D",
        "-n",
        "8",
        "--format=%H",
        "--name-only",
        "--",
        &pathspec,
    ]);
    let out = match probe {
        GitProbe::ProbeFailed(cause) => {
            return Err(format!(
                "the deletion-history query (`git log --diff-filter=D -- {pathspec}`) could not be consulted: {cause}"
            ))
        }
        GitProbe::Ran(out) => out,
    };
    // The output alternates a commit line with its deleted path(s), blank-separated.
    let mut candidates: Vec<(String, String)> = Vec::new();
    {
        let mut sha: Option<String> = None;
        for line in out.lines().map(str::trim).filter(|l| !l.is_empty()) {
            if line.bytes().all(|b| b.is_ascii_hexdigit()) && line.len() >= 7 {
                sha = Some(line.to_string());
            } else if let Some(s) = sha.clone() {
                candidates.push((s, line.to_string()));
            }
        }
    }
    if candidates.is_empty() {
        // No reachable commit deleted ANY file named <leaf>.dag. Confirm the history was
        // actually consultable before answering "never existed here": a probe over zero
        // reachable commits is ignorance, not an answer.
        return match git_probe(&["rev-list", "--count", "HEAD"]) {
            GitProbe::Ran(count) if count == "0" => {
                Err("the reachable history has zero commits to search for a deletion".to_string())
            }
            GitProbe::Ran(count) => Ok(DeletionAttribution::NoDeletionInHistory {
                probed_commits: count.parse().unwrap_or(0),
            }),
            GitProbe::ProbeFailed(cause) => Err(format!(
                "the reachable-commit count could not be consulted: {cause}"
            )),
        };
    }
    // Exact confirmation against each candidate's PARENT BLOB, not the name: the file git
    // says was deleted must have declared this very header in the commit before the deletion.
    // With `-m` a deletion can be reported against EITHER parent of a merge, so parent 1 is
    // tried first and parent 2 only when parent 1 cannot be read (the file never existed
    // there). Neither reading is ever silently skipped: an unreadable pair refuses.
    let want = format!("module {identity}");
    let mut rejected = 0usize;
    for (sha, path) in &candidates {
        let p1 = git_probe(&["show", &format!("{sha}^:{path}")]);
        let blob = match p1 {
            GitProbe::Ran(blob) => GitProbe::Ran(blob),
            GitProbe::ProbeFailed(_) => git_probe(&["show", &format!("{sha}^2:{path}")]),
        };
        match blob {
            GitProbe::Ran(blob)
                if blob.lines().any(|l| l.trim_start().trim_end() == want) =>
            {
                return Ok(DeletionAttribution::Attributed { commit: sha.clone() });
            }
            GitProbe::Ran(_) => rejected += 1,
            GitProbe::ProbeFailed(cause) => {
                return Err(format!(
                    "neither parent blob of `{sha}` (`{sha}^:{path}` nor `{sha}^2:{path}`) could be read to verify the header: {cause}"
                ))
            }
        }
    }
    // Candidates existed but every deleted same-stem file declared a DIFFERENT module: this
    // identity's own file was never deleted in reachable history.
    Ok(DeletionAttribution::NoDeletionInHistory {
        probed_commits: rejected,
    })
}

/// Query ONE identity against the ingestion, deriving the never-admitted evidence at run time.
pub fn module_admission(source_roots: &[String], identity: &str) -> ModuleAdmission {
    let identities = match source_root_ingest_module_identities_for_ci(source_roots) {
        Ok(identities) => identities,
        Err(cause) => {
            return ModuleAdmission::IndexRefused {
                module: identity.to_string(),
                cause,
            }
        }
    };
    if identities.iter().any(|module| module == identity) {
        return ModuleAdmission::Admitted {
            module: identity.to_string(),
        };
    }
    // The identity is in no file's `module` header. Establish WHERE it went, by derivation --
    // and refuse rather than fabricate: if git cannot answer even "what SHA is this run", the
    // verdict would name evidence it never observed (DESIGN 5: no plausible output is
    // fabricated), so the query exits on EvidenceRefused instead of printing "unknown".
    let run_sha = match git_probe(&["rev-parse", "HEAD"]) {
        GitProbe::Ran(sha) if !sha.is_empty() => sha,
        GitProbe::Ran(_) => {
            return ModuleAdmission::EvidenceRefused {
                module: identity.to_string(),
                cause: "git rev-parse HEAD succeeded but printed nothing".to_string(),
            }
        }
        GitProbe::ProbeFailed(cause) => {
            return ModuleAdmission::EvidenceRefused {
                module: identity.to_string(),
                cause,
            }
        }
    };
    let attribution = match deletion_attribution(identity) {
        Ok(a) => a,
        Err(cause) => DeletionAttribution::ProbeRefused { cause },
    };
    ModuleAdmission::NeverAdmitted {
        module: identity.to_string(),
        run_sha,
        attribution,
    }
}

/// The loader closure of one `--entry` file: what `gunbc run --entry <path>` resolves before it
/// evaluates anything. `entry_path` is workspace-relative, as the `run:` line spells it.
pub fn entry_closure_module_identities(
    source_roots: &[String],
    entry_path: &str,
) -> ModuleIdentityPopulation {
    let index = match entry_resolve::try_index_for_run_or_owned_pool(source_roots) {
        Ok(index) => index,
        Err(cause) => {
            return ModuleIdentityPopulation::Refused {
                cause: format!("entry closure of {entry_path}: source-discovery: {cause}"),
            }
        }
    };
    let sources = match load_sources_for_entry_with_pool(&index, entry_path) {
        Ok(sources) => sources,
        Err(cause) => {
            return ModuleIdentityPopulation::Refused {
                cause: format!("entry closure of {entry_path}: {cause}"),
            }
        }
    };
    let mut modules: Vec<String> = sources
        .iter()
        .filter_map(|source| extract_module_path(&source.content))
        .collect();
    modules.sort();
    modules.dedup();
    ModuleIdentityPopulation::Observed { modules }
}

#[cfg(test)]
mod tests {
    /// THE DELETING COMMIT IS DERIVED, NOT WRITTEN. The instrument attributes a deletion by
    /// running git over the module header; a hard-coded 40-hex SHA anywhere in this file's own
    /// text would be a hand-written observation posing as a derived one (DESIGN 4b: a written-
    /// down value is never evidence). This test scans THIS SOURCE FILE for any 40-hex run.
    #[test]
    fn the_instrument_carries_no_hand_written_commit_sha() {
        let text = std::fs::read_to_string("src/cli_run/required_lane_resolution_census.rs")
            .expect("instrument source readable from crate root");
        let is_hex = |c: char| c.is_ascii_hexdigit();
        let mut run = 0usize;
        for c in text.chars() {
            run = if is_hex(c) { run + 1 } else { 0 };
            assert!(
                run < 40,
                "a 40-hex literal appears in the instrument source"
            );
        }
    }
}
