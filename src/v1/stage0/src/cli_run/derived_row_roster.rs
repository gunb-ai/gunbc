//! Derived membership for `gunbc.recurring_failure_mode.roster`.
//!
//! Row files under `dag/gunbc/recurring_failure_mode/` are the authority. A hand-appended
//! roster list is a second authoring of the same membership, so two lanes appending conflict
//! on the list and a forgotten list entry drops a row from every projection that folds it.
//! This writer folds the directory: every sibling `*.dag` except `roster.dag` is one member,
//! identity-sorted, so an append is a new file and membership cannot omit a file that exists.
//!
//! Called from every `.dag` directory walk that can reach that folder, before the listing,
//! so a clone with no committed roster still compiles. The file is gitignored.

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
        _ => fs::write(path, body),
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
            None => continue,
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
