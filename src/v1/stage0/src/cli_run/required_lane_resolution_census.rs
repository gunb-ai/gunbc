//! THE REQUIRED-LANE RESOLUTION CENSUS'S THREE COMPILER QUERIES.
//!
//! The question the census asks is DESIGN section 3's standing warning read as a measurement:
//! outside the required gate the substrate still refuses and nothing asks it to, so a
//! product-layer dependent can stop resolving, stay broken, and let every required lane report
//! SUCCESS. On 2026-09-20 that was the live state -- `gunbc.site_pxe_edge_converge` did not
//! resolve on main while all four required lanes were green (gunbc#11607) -- and the question
//! "which modules can do that" had no instrument. This file answers the three population
//! queries the `.dag` census (`gunbc.required_lane_resolution_census`) folds over; the census,
//! not this file, decides which lanes exist and what each one resolves, and it reads that roster
//! from `gunbc.compiler_gate_workflow` so a job the workflow emits cannot go unclassified.
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

/// The loader closure of one `--entry` file: what `gunbc run --entry <path>` resolves before it
/// evaluates anything. `entry_path` is workspace-relative, as the `run:` line spells it.
pub fn entry_closure_module_identities(
    source_roots: &[String],
    entry_path: &str,
) -> ModuleIdentityPopulation {
    let index = process_shared_index(source_roots);
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
