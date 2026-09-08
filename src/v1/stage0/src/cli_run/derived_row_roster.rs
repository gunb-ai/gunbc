//! Derived membership for `gunbc.recurring_failure_mode.roster`.
//!
//! Row files under `dag/gunbc/recurring_failure_mode/` are the authority. A hand-appended
//! roster list is a second authoring of the same membership.
//!
//! THIS IS NOT CALLED FROM EVERY DIRECTORY WALK. It is invoked from the collect sites
//! (`collect_dag_files_result`, `collect_dag_files_tolerant`, `main.rs` `collect_dag_files`,
//! `compiler_tests` `collect_dag_recursive`) and from `run_dag_parse_sweep`, which is the
//! required-CI parse phase. That last site is load-bearing: three modules import
//! `gunbc.recurring_failure_mode.roster` (`gunbc.design_ledgers`, `gunbc.ledger_row_coherence`,
//! `test.claim.generated_artifact_merge_driver_real_execution_witness`), and the parse join reads
//! that module from the sweep index. A checkout with no committed roster must derive it before the
//! sweep lists `.dag` files, or the join reports `RosterModuleAbsent` on the intended steady
//! state. The collect sites are convenience for other compilers; they are not the merge-path
//! writer.
//! An empty list is a different state — a present `= []` — and only arises if the directory
//! contains no sibling row files. A `read_dir` or dirent error refuses the collect; a `.dag`
//! file whose stem is not utf-8 refuses rather than being dropped from the list. If a listed
//! row file is compiled as `RecurringFailureMode` and the derived list omits it, the parse
//! join reports `DeclaredNotRostered`. Neither arm is a silent zero.
//!
//! The compiler writes this gitignored file as a realization of a capability the substrate
//! lacks (directory enumeration sufficient to bind each sibling as a typed value). That
//! write is a read-path side effect; a failed write refuses the collect rather than proceeding
//! with a missing module. Dissolution: declaration-value binding over the declared population,
//! same grounding as `gunbc.guarantee_stall.roster_re_enumerates_its_own_rows_stall`.

use std::fs;
use std::io;
use std::path::Path;

pub const ROSTER_BASENAME: &str = "roster.dag";
/// Module path of the row files; `ROW_DIR_REL` is this spelling with `/` for `.`.
pub const ROW_MODULE: &str = "gunbc.recurring_failure_mode";
pub const ROW_DIR_REL: &str = "gunbc/recurring_failure_mode";

/// This module's own source path, as the repository spells it.
pub const ROSTER_REL_PATH: &str = "dag/gunbc/recurring_failure_mode/roster.dag";

/// The Rust module that CARRIES this derivation, named as a repository path.
///
/// A REVISION EITHER HAS THE DERIVATION OR IT DOES NOT, and that is a checkable fact rather than an
/// assumption. A base predating gunbc#10822 tracked `roster.dag` as ordinary source and had no
/// deriver; a base older still had neither. Anything reconstructing this module at another revision
/// must ask which of those it is looking at, because deriving a roster for a revision that never
/// derived one invents a module that did not exist there.
pub const DERIVATION_CARRIER_REL_PATH: &str = "src/v1/stage0/src/cli_run/derived_row_roster.rs";

/// The row stem a path contributes, or `None` when the path is not a row.
///
/// ONE PREDICATE, TWO CALLERS. The filesystem writer walks a directory and the base-index
/// reconstruction walks a git listing, but "which files are rows" is one fact and a second copy of
/// it would drift in the direction that silently shortens a roster.
pub fn row_stem_of_basename(basename: &str) -> Option<&str> {
    let stem = basename.strip_suffix(".dag")?;
    if stem == "roster" || stem.is_empty() {
        return None;
    }
    Some(stem)
}

/// The roster source this repository would derive from a set of repository-relative paths.
///
/// FOR A REVISION THAT IS NOT THE WORKING TREE. The writer reads a directory because it is writing
/// into one; a caller holding a `git ls-tree` listing of another revision has the same membership
/// fact in a different representation, and rendering it through the SAME renderer is what makes the
/// two answers one answer. Returning source rather than parsed facts is the point: every consumer
/// then derives its own facts from source, as it does for a tracked file.
pub fn roster_source_from_repo_paths(paths: &std::collections::BTreeSet<String>) -> String {
    let dir_prefix = format!("dag/{ROW_DIR_REL}/");
    let mut names: Vec<String> = paths
        .iter()
        .filter_map(|p| p.strip_prefix(&dir_prefix))
        .filter(|rest| !rest.contains('/'))
        .filter_map(row_stem_of_basename)
        .map(|s| s.to_string())
        .collect();
    names.sort();
    render_roster(&names)
}

pub fn is_recurring_failure_mode_row_dir(dir: &Path) -> bool {
    dir.ends_with(Path::new(ROW_DIR_REL))
}

pub fn ensure_if_row_dir(dir: &Path) -> io::Result<()> {
    if is_recurring_failure_mode_row_dir(dir) {
        ensure_derived_recurring_failure_mode_roster(dir)
    } else {
        Ok(())
    }
}

pub fn ensure_if_row_dir_or_panic(dir: &Path) {
    if let Err(e) = ensure_if_row_dir(dir) {
        panic!(
            "failed to derive recurring_failure_mode roster in {:?}: {}",
            dir, e
        );
    }
}

pub fn ensure_derived_recurring_failure_mode_roster(dir: &Path) -> io::Result<()> {
    let names = row_stems(dir)?;
    let body = render_roster(&names);
    let path = dir.join(ROSTER_BASENAME);
    match fs::read(&path) {
        Ok(existing) if existing == body.as_bytes() => Ok(()),
        _ => write_atomically(&path, body.as_bytes()),
    }
}

fn write_atomically(path: &Path, body: &[u8]) -> io::Result<()> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = path.with_file_name(format!(
        "roster.dag.{}.{}.deriving",
        std::process::id(),
        nanos
    ));
    fs::write(&tmp, body)?;
    match fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = fs::remove_file(&tmp);
            Err(e)
        }
    }
}

fn row_stems(dir: &Path) -> io::Result<Vec<String>> {
    let mut names = Vec::new();
    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("dag") {
            continue;
        }
        let basename = match path.file_name().and_then(|n| n.to_str()) {
            Some(s) => s,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "recurring_failure_mode row file {:?} has a non-utf8 name; refusing to derive a shortened roster",
                        path
                    ),
                ));
            }
        };
        let Some(stem) = row_stem_of_basename(basename) else {
            continue;
        };
        names.push(stem.to_string());
    }
    names.sort();
    Ok(names)
}

fn render_roster(names: &[String]) -> String {
    let mut out = String::from(
        "module gunbc.recurring_failure_mode.roster\n\
         \n\
         // DERIVED from sibling RecurringFailureMode row files. Do not hand-edit.\n\
         // Membership is the directory; order is the sorted filename stem, which is the\n\
         // declaration name. An append is a new file in this directory, never an edit here.\n\
         \n\
         import std.types { List }\n\
         import gunbc.recurring_failure_mode { RecurringFailureMode }\n",
    );
    for name in names {
        out.push_str("import gunbc.recurring_failure_mode.");
        out.push_str(name);
        out.push_str(" { ");
        out.push_str(name);
        out.push_str(" }\n");
    }
    out.push_str("\ndata recurring_failure_mode_roster: List<RecurringFailureMode> = [\n");
    for name in names {
        out.push_str("  ");
        out.push_str(name);
        out.push_str(",\n");
    }
    out.push_str("]\n");
    out
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    fn paths(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| s.to_string()).collect()
    }

    /// THE ACUTE CASE gunbc#10814 HIT: one revision has a row the other does not.
    ///
    /// Rendering from a path listing rather than a directory is what lets the namespace wall see
    /// the roster as it stood at ANOTHER revision. If this ever answered with both rows, that wall
    /// would compare a head roster against itself and report an authored addition as a coincidence
    /// resolving from a pool -- the defect this exists to prevent.
    #[test]
    fn a_row_absent_from_the_listing_is_absent_from_the_rendered_roster() {
        let base = super::roster_source_from_repo_paths(&paths(&[
            "dag/gunbc/recurring_failure_mode/alpha.dag",
        ]));
        assert!(base.contains("alpha"));
        assert!(
            !base.contains("beta"),
            "a row with no file in the listing must not be rendered: {base}"
        );

        let head = super::roster_source_from_repo_paths(&paths(&[
            "dag/gunbc/recurring_failure_mode/alpha.dag",
            "dag/gunbc/recurring_failure_mode/beta.dag",
        ]));
        assert!(head.contains("beta"));
    }

    /// THE RENDERER'S OWN MEMBERSHIP RULE, ASKED THROUGH THE PATH CHANNEL. The directory walk
    /// already excludes `roster.dag` and non-`.dag` files; a second answer here that disagreed
    /// would put a row in one representation and not the other.
    #[test]
    fn only_row_files_directly_in_the_row_directory_are_rows() {
        let out = super::roster_source_from_repo_paths(&paths(&[
            "dag/gunbc/recurring_failure_mode/alpha.dag",
            "dag/gunbc/recurring_failure_mode/roster.dag",
            "dag/gunbc/recurring_failure_mode/notes.md",
            "dag/gunbc/recurring_failure_mode/nested/deep.dag",
            "dag/gunbc/other/elsewhere.dag",
        ]));
        assert!(out.contains("alpha"));
        for absent in ["roster,", "notes", "deep", "elsewhere"] {
            assert!(
                !out.contains(absent),
                "{absent} is not a row of this directory: {out}"
            );
        }
    }

    #[test]
    fn absent_is_not_an_empty_list_render_of_no_names_is_a_present_empty_literal() {
        let body = super::render_roster(&[]);
        assert!(
            body.contains("module gunbc.recurring_failure_mode.roster"),
            "absence of members is a present module with an empty list, not a missing module"
        );
        assert!(body.contains("= [\n]\n"));
    }

    #[test]
    fn row_dir_is_the_modeled_module_home_not_a_bare_folder_name() {
        use std::path::Path;
        assert!(super::is_recurring_failure_mode_row_dir(Path::new(
            "dag/gunbc/recurring_failure_mode"
        )));
        assert!(super::is_recurring_failure_mode_row_dir(Path::new(
            "gunbc/recurring_failure_mode"
        )));
        assert!(!super::is_recurring_failure_mode_row_dir(Path::new(
            "recurring_failure_mode"
        )));
        assert_eq!(
            super::ROW_DIR_REL,
            super::ROW_MODULE.replace('.', "/"),
            "directory match is the module path, not a second spelling"
        );
    }
}
