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
pub enum ModuleAdmission {
    /// The ingestion admits the identity: some file under the source roots declares it.
    Admitted { module: String },
    /// No file in the tree at the run's SHA declares the identity. `deleting_commit` is DERIVED
    /// (a pickaxe over the module header: the newest commit that changed the header's occurrence
    /// count, verified to have found it in its parent), never hand-written; `None` means either
    /// the identity never existed in the reachable history or the history is too shallow to
    /// prove which -- the state still holds, the attribution just stays silent.
    NeverAdmitted {
        module: String,
        run_sha: String,
        deleting_commit: Option<String>,
    },
    /// The ingestion itself refused, so the identity's state was never observed. The census
    /// must decide on this arm; reading it as never-admitted would be absence-as-evidence
    /// smuggled back in through the error path.
    IndexRefused { module: String, cause: String },
}

fn git_output(args: &[&str]) -> Option<String> {
    let out = std::process::Command::new("git").args(args).output().ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        None
    }
}

fn regex_escape_identity(module: &str) -> String {
    let mut out = String::new();
    for c in module.chars() {
        if ".^$*+?()[]{}|\\".contains(c) {
            out.push('\\');
        }
        out.push(c);
    }
    out
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
    // The identity is in no file's `module` header. Establish WHERE it went, by derivation:
    // the newest commit whose diff changed the occurrence count of the exact header line, and
    // whether its parent still had it (only a commit that FOUND it in the parent deleted it --
    // without this check, a commit that merely ADDED the header elsewhere would be accused).
    let run_sha = git_output(&["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".to_string());
    let header = format!("module {identity}\n");
    let deleting_commit =
        git_output(&["log", "-1", "--format=%H", "-S", &header, "--"]).and_then(|sha| {
            let pattern = format!("^module {}\\s*$", regex_escape_identity(identity));
            let parent = format!("{sha}^");
            match git_output(&["grep", "-l", "-E", &pattern, &parent, "--"]) {
                Some(_) => Some(sha),
                None => None,
            }
        });
    ModuleAdmission::NeverAdmitted {
        module: identity.to_string(),
        run_sha,
        deleting_commit,
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

    /// The escape class is real: an identity with regex metacharacters cannot crash or spoof
    /// the `git grep` pattern used for the parent-contains check.
    #[test]
    fn identity_escaping_neutralises_metacharacters() {
        // A regex metacharacter inside an identity must not reach `git grep -E` raw: `a.b*c`
        // must match ONLY itself, so every metacharacter is escaped -- dots included.
        let escaped = super::regex_escape_identity("a.b*c");
        assert_eq!(escaped, "a\\.b\\*c");
        // An identity with no metacharacters passes through untouched.
        assert_eq!(super::regex_escape_identity("plainname"), "plainname");
    }
}
