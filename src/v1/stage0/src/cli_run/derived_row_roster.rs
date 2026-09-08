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
        let stem = match path.file_stem().and_then(|n| n.to_str()) {
            Some(s) => s,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "recurring_failure_mode row file {:?} has a non-utf8 stem; refusing to derive a shortened roster",
                        path
                    ),
                ));
            }
        };
        if stem == "roster" {
            continue;
        }
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
