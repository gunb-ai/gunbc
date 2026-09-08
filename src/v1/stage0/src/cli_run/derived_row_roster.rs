//! Derived membership for `gunbc.recurring_failure_mode.roster`.
//!
//! Row files under `dag/gunbc/recurring_failure_mode/` are the authority. A hand-appended
//! roster list is a second authoring of the same membership.
//!
//! THIS IS NOT CALLED FROM EVERY DIRECTORY WALK. It is invoked from four collect sites
//! (`collect_dag_files_result`, `collect_dag_files_tolerant`, `main.rs` `collect_dag_files`,
//! `compiler_tests` `collect_dag_recursive`). Those writes are convenience so a clone compiles
//! without a committed roster. They are not load-bearing for the absent case: three modules
//! import `gunbc.recurring_failure_mode.roster` (`gunbc.design_ledgers`, `gunbc.ledger_row_coherence`,
//! `test.claim.generated_artifact_merge_driver_real_execution_witness`), so a reader that never
//! writes still hits a missing module and refuses at resolve.
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
pub const ROW_DIR_NAME: &str = "recurring_failure_mode";
pub const PARENT_DIR_NAME: &str = "gunbc";

pub fn is_recurring_failure_mode_row_dir(dir: &Path) -> bool {
    dir.file_name().and_then(|n| n.to_str()) == Some(ROW_DIR_NAME)
        && dir
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            == Some(PARENT_DIR_NAME)
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
    let tmp = path.with_extension("dag.deriving");
    fs::write(&tmp, body)?;
    fs::rename(&tmp, path)
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
}
