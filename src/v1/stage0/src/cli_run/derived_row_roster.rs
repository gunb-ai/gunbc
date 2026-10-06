//! Derived membership for the one-row-per-file ledgers: `gunbc.recurring_failure_mode.roster`
//! and `gunbc.rung_drop.roster` (`DERIVED_ROW_ROSTERS`).
//!
//! Row files under each row directory are the authority. A hand-appended roster list is a second
//! authoring of the same membership -- the rung-drop list was one until it was cut over to this
//! fold, and a drop absent from it was absent from `docs/design-rung-drops.md`.
//!
//! MEMBERSHIP IS SELECTED BY DECLARED TYPE, NOT BY DIRECTORY ALONE. A sibling file is a member
//! exactly when it declares the row under its own stem with the ledger's row type
//! (`data <stem>: <RowType> =`), so a helper module that lives beside the rows
//! (`gunbc.rung_drop.shared_capability`, `gunbc.rung_drop.standing`) is not rostered. That
//! selector is a line-shape read and NOT the authority on what was declared: the parse join in
//! `rostered_row_join` reads the compiled declaration population, and a `RowType` row the selector
//! missed refuses there as `DeclaredNotRostered`. The two disagree only loudly.
//!
//! THIS IS NOT CALLED FROM EVERY DIRECTORY WALK. It is invoked from the collect sites
//! (`collect_dag_files_result`, `collect_dag_files_tolerant`, `main.rs` `collect_dag_files`,
//! `compiler_tests` `collect_dag_recursive`) and from `run_dag_parse_sweep`, which is the
//! required-CI parse phase. That last site is load-bearing: modules import each roster
//! (`gunbc.design_ledgers`, `gunbc.ledger_row_coherence`, and for rung drops
//! `gunbc.rung_drop.standing` and its readers), and the parse join reads that module from the
//! sweep index. A checkout with no committed roster must derive it before the
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

/// One ledger whose membership is the set of row files in one directory.
///
/// `row_dir_rel` is `row_module` with `/` for `.`, and `roster_module` is `row_module` plus the
/// `ROSTER_BASENAME` stem; both compositions are asserted by
/// `the_roster_module_is_the_row_module_plus_the_roster_stem` and
/// `row_dir_is_the_modeled_module_home_not_a_bare_folder_name` rather than constructed, because
/// `concat!` takes literal tokens and these are consts.
///
/// ONE SPELLING, BECAUSE `rostered_row_join` LOOKS THE ROSTER MODULE UP BY NAME.
/// `ENROLLED_ROW_TYPES` carries each roster's module identity from this table and
/// `run_rostered_row_join` resolves it with `index_get(index, enrolled.roster_module)` against the
/// module header `render_roster` WRITES. A divergence fails closed as the typed, located
/// `RosterModuleAbsent` finding.
///
/// THOSE ASSERTIONS ARE NOT A WALL, AND THE RUNG IS 1 -- MITIGATION. They are `#[cfg(test)]` under
/// `repo_self_test_command`, which no CI step runs (`gunbc.rung_drop`
/// `rust_unit_tests_off_the_merge_path`). Next-rung trigger: the parts composable in a const
/// context, which makes a second spelling unconstructible rather than assert-checked.
pub struct DerivedRowRoster {
    pub row_module: &'static str,
    pub row_dir_rel: &'static str,
    pub row_type: &'static str,
    pub roster_module: &'static str,
    pub roster_declaration: &'static str,
}

pub const RECURRING_FAILURE_MODE: DerivedRowRoster = DerivedRowRoster {
    row_module: "gunbc.recurring_failure_mode",
    row_dir_rel: "gunbc/recurring_failure_mode",
    row_type: "RecurringFailureMode",
    roster_module: "gunbc.recurring_failure_mode.roster",
    roster_declaration: "recurring_failure_mode_roster",
};

pub const RUNG_DROP: DerivedRowRoster = DerivedRowRoster {
    row_module: "gunbc.rung_drop",
    row_dir_rel: "gunbc/rung_drop",
    row_type: "RungDrop",
    roster_module: "gunbc.rung_drop.roster",
    roster_declaration: "rung_drop_roster",
};

pub const DERIVED_ROW_ROSTERS: [&DerivedRowRoster; 2] = [&RECURRING_FAILURE_MODE, &RUNG_DROP];

/// The failure-mode ledger's parts, read by `rostered_row_join`'s file-to-roster join, which is
/// specific to that ledger: its directory holds rows only, so every file must be a member. The
/// rung-drop directory also holds helper modules, so for it the type-selected membership and the
/// parse join's `DeclaredNotRostered` are the whole check.
pub const ROW_MODULE: &str = RECURRING_FAILURE_MODE.row_module;
pub const ROSTER_MODULE: &str = RECURRING_FAILURE_MODE.roster_module;
pub const ROW_DIR_REL: &str = RECURRING_FAILURE_MODE.row_dir_rel;

/// Which derived roster, if any, `rel` IS -- under any sweep root.
///
/// THE ROSTER HAS NO BASE SIDE IN A DIFF, WHICH IS THE WHOLE REASON THIS PREDICATE EXISTS.
/// The file is gitignored and written on the read path, so it never appears in
/// `git diff --name-status` — and a baseline reconstructed by dropping only the paths the diff
/// TOUCHED therefore inherits the HEAD's roster bytes as the BASE's. That inheritance made a
/// row append report a binding the base already spelled (`base {} -> head {row}` on a key
/// present both sides) and, worse, made the module's own source read as UNCHANGED, so the
/// authorship discriminator answered false and an ordinary append classified
/// `NewPoolCoincidenceResolution` — a pool coincidence, caused elsewhere, for a name the roster
/// itself imports. Every future append would have blocked identically.
fn derived_roster_at(rel: &str) -> Option<(&'static DerivedRowRoster, &str)> {
    DERIVED_ROW_ROSTERS.iter().find_map(|roster| {
        let suffix = format!("/{}/{ROSTER_BASENAME}", roster.row_dir_rel);
        rel.strip_suffix(&suffix).map(|root| (*roster, root))
    })
}

pub fn is_derived_roster_path(rel: &str) -> bool {
    derived_roster_at(rel).is_some()
}

/// The sweep-root prefix of a derived roster path: `dag/gunbc/recurring_failure_mode/roster.dag`
/// -> `dag`. `None` when `rel` is not a roster path.
pub fn roster_root_prefix(rel: &str) -> Option<&str> {
    derived_roster_at(rel).map(|(_, root)| root)
}

/// The roster that `roster_rel` would derive from an ARBITRARY path listing rather than from the
/// filesystem — the base tree's answer, rendered by the SAME function the writer uses, so there is
/// one authority for the roster's bytes and not a second reconstruction beside it.
///
/// `paths` is a repo-relative listing (`git ls-tree -r --name-only <base>`) and `read` returns a
/// listed file's content at that same revision, because membership is selected by declared type
/// and a stem alone does not say whether its file declares a row. Returns `Ok(None)` when
/// `roster_rel` is not a derived roster or the listing carries NO row files there: that is the
/// roster module being genuinely absent at the base, which is a different state from a present
/// empty list and must not be fabricated as one. A failed read refuses rather than shortening the
/// list.
pub fn roster_from_path_listing<'a>(
    roster_rel: &str,
    paths: impl IntoIterator<Item = &'a str>,
    read: impl Fn(&str) -> Result<String, String>,
) -> Result<Option<String>, String> {
    let Some((roster, root)) = derived_roster_at(roster_rel) else {
        return Ok(None);
    };
    let dir = format!("{root}/{}/", roster.row_dir_rel);
    let mut names: Vec<String> = Vec::new();
    for rel in paths {
        let Some(rest) = rel.strip_prefix(&dir) else {
            continue;
        };
        if rest.contains('/') {
            continue;
        }
        let Some(stem) = rest.strip_suffix(".dag") else {
            continue;
        };
        if stem == "roster" {
            continue;
        }
        if declares_row(&read(rel)?, stem, roster.row_type) {
            names.push(stem.to_string());
        }
    }
    if names.is_empty() {
        return Ok(None);
    }
    names.sort();
    Ok(Some(render_roster(roster, &names)))
}

/// Does `source` declare its own stem as a row of `row_type`: `data <stem>: <row_type> =`.
fn declares_row(source: &str, stem: &str, row_type: &str) -> bool {
    let head = format!("data {stem}: {row_type} =");
    source.lines().any(|line| line.starts_with(&head))
}

fn row_dir_roster(dir: &Path) -> Option<&'static DerivedRowRoster> {
    DERIVED_ROW_ROSTERS
        .iter()
        .copied()
        .find(|roster| dir.ends_with(Path::new(roster.row_dir_rel)))
}

pub fn ensure_if_row_dir(dir: &Path) -> io::Result<()> {
    match row_dir_roster(dir) {
        Some(roster) => ensure_derived_roster(roster, dir),
        None => Ok(()),
    }
}

pub fn ensure_if_row_dir_or_panic(dir: &Path) {
    if let Err(e) = ensure_if_row_dir(dir) {
        panic!("failed to derive row roster in {:?}: {}", dir, e);
    }
}

pub fn ensure_derived_roster(roster: &DerivedRowRoster, dir: &Path) -> io::Result<()> {
    let names = row_stems(roster, dir)?;
    let body = render_roster(roster, &names);
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

fn row_stems(roster: &DerivedRowRoster, dir: &Path) -> io::Result<Vec<String>> {
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
                        "{} row file {:?} has a non-utf8 stem; refusing to derive a shortened roster",
                        roster.row_module, path
                    ),
                ));
            }
        };
        if stem == "roster" {
            continue;
        }
        if declares_row(&fs::read_to_string(&path)?, stem, roster.row_type) {
            names.push(stem.to_string());
        }
    }
    names.sort();
    Ok(names)
}

pub fn render_roster(roster: &DerivedRowRoster, names: &[String]) -> String {
    let DerivedRowRoster {
        row_module,
        row_type,
        roster_module,
        roster_declaration,
        ..
    } = roster;
    let mut out = format!(
        "module {roster_module}\n\
         \n\
         // DERIVED from sibling files declaring a {row_type} row under their own stem. Do not\n\
         // hand-edit. Membership is the directory filtered by declared type; order is the sorted\n\
         // filename stem, which is the declaration name. An append is a new file in this\n\
         // directory, never an edit here.\n\
         \n\
         import std.types {{ List }}\n\
         import {row_module} {{ {row_type} }}\n"
    );
    for name in names {
        out.push_str(&format!("import {row_module}.{name} {{ {name} }}\n"));
    }
    out.push_str(&format!(
        "\ndata {roster_declaration}: List<{row_type}> = [\n"
    ));
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
    use super::{DERIVED_ROW_ROSTERS, RECURRING_FAILURE_MODE, RUNG_DROP};

    /// A base-tree reader over an in-memory listing; an unlisted path is a read fault.
    fn reader<'a>(files: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Result<String, String> + 'a {
        move |rel| {
            files
                .iter()
                .find(|(p, _)| *p == rel)
                .map(|(_, c)| c.to_string())
                .ok_or_else(|| format!("no such file {rel}"))
        }
    }

    #[test]
    fn the_roster_module_is_the_row_module_plus_the_roster_stem() {
        // THE COMPOSITION THE CONST CANNOT EXPRESS, asserted per ledger, so renaming either part
        // reds here rather than silently handing the row join a name it will fail to resolve.
        for roster in DERIVED_ROW_ROSTERS {
            assert_eq!(
                roster.roster_module,
                format!(
                    "{}.{}",
                    roster.row_module,
                    super::ROSTER_BASENAME.trim_end_matches(".dag")
                ),
                "the derived roster's module is its row module plus the roster file's stem"
            );
        }
    }

    #[test]
    fn absent_is_not_an_empty_list_render_of_no_names_is_a_present_empty_literal() {
        let body = super::render_roster(&RECURRING_FAILURE_MODE, &[]);
        assert!(
            // THE LITERAL IS THE POINT: an oracle built from the table would assert
            // `render_roster` against the same const it renders from -- `measure() == measure()`
            // (DESIGN section 5). These spellings are INDEPENDENT referents.
            body.contains("module gunbc.recurring_failure_mode.roster"),
            "absence of members is a present module with an empty list, not a missing module"
        );
        assert!(body.contains("= [\n]\n"));
        let body = super::render_roster(&RUNG_DROP, &[]);
        assert!(body.contains("module gunbc.rung_drop.roster"));
        assert!(body.contains("data rung_drop_roster: List<RungDrop> = [\n]\n"));
    }

    #[test]
    fn a_base_tree_with_no_row_files_has_no_roster_module_rather_than_an_empty_one() {
        // The two absences are different states with different remedies: a base that never
        // carried the row directory has NO roster module, while a present `= []` asserts the
        // directory exists and is empty. Fabricating the second from the first would give every
        // name in the head roster a base side to be compared against.
        assert_eq!(
            super::roster_from_path_listing(
                "dag/gunbc/recurring_failure_mode/roster.dag",
                ["dag/gunbc/other/thing.dag"],
                reader(&[]),
            ),
            Ok(None)
        );
    }

    #[test]
    fn the_base_side_is_the_base_listing_not_the_head_directory() {
        let files = [
            (
                "dag/gunbc/recurring_failure_mode/beta.dag",
                "module x\n\ndata beta: RecurringFailureMode = RecurringFailureMode {\n}\n",
            ),
            (
                "dag/gunbc/recurring_failure_mode/alpha.dag",
                "module x\n\ndata alpha: RecurringFailureMode = RecurringFailureMode {\n}\n",
            ),
        ];
        let body = super::roster_from_path_listing(
            "dag/gunbc/recurring_failure_mode/roster.dag",
            [
                "dag/gunbc/recurring_failure_mode/beta.dag",
                "dag/gunbc/recurring_failure_mode/alpha.dag",
                "dag/gunbc/recurring_failure_mode/roster.dag",
                "dag/gunbc/recurring_failure_mode/nested/deep.dag",
                "dag/gunbc/recurring_failure_mode/notes.md",
            ],
            reader(&files),
        )
        .expect("every listed row file is readable")
        .expect("two row files at the base are a present roster");
        assert!(body.contains("import gunbc.recurring_failure_mode.alpha { alpha }"));
        assert!(body.contains("= [\n  alpha,\n  beta,\n]\n"));
        // The roster never rosters itself, and membership is the directory, not its subtrees.
        assert!(!body.contains("roster,"));
        assert!(!body.contains("deep"));
    }

    #[test]
    fn membership_is_selected_by_declared_row_type_not_by_directory() {
        // THE CONTROL PAIR the rung-drop cutover exists for: a new drop file with no roster edit
        // is a member, and a helper module in the same directory is not.
        let files = [
            (
                "dag/gunbc/rung_drop/new_drop.dag",
                "module gunbc.rung_drop.new_drop\n\ndata new_drop: RungDrop = RungDrop {\n}\n",
            ),
            (
                "dag/gunbc/rung_drop/shared_capability.dag",
                "module gunbc.rung_drop.shared_capability\n\ndata shared_capabilities: List<SharedCapability> = []\n",
            ),
        ];
        let body = super::roster_from_path_listing(
            "dag/gunbc/rung_drop/roster.dag",
            files.iter().map(|(p, _)| *p),
            reader(&files),
        )
        .expect("readable")
        .expect("one row file is a present roster");
        assert!(body.contains("import gunbc.rung_drop.new_drop { new_drop }"));
        assert!(body.contains("data rung_drop_roster: List<RungDrop> = [\n  new_drop,\n]\n"));
        assert!(!body.contains("shared_capability"));
    }

    #[test]
    fn an_unreadable_base_row_refuses_rather_than_shortening_the_roster() {
        assert!(super::roster_from_path_listing(
            "dag/gunbc/rung_drop/roster.dag",
            ["dag/gunbc/rung_drop/a.dag"],
            reader(&[]),
        )
        .is_err());
    }

    #[test]
    fn a_derived_roster_path_is_recognised_under_any_sweep_root() {
        for (roster, row) in [
            (
                "dag/gunbc/recurring_failure_mode/roster.dag",
                "dag/gunbc/recurring_failure_mode/some_row.dag",
            ),
            (
                "dag/gunbc/rung_drop/roster.dag",
                "dag/gunbc/rung_drop/some_row.dag",
            ),
        ] {
            assert!(super::is_derived_roster_path(roster));
            assert_eq!(super::roster_root_prefix(roster), Some("dag"));
            assert!(!super::is_derived_roster_path(row));
            assert_eq!(super::roster_root_prefix(row), None);
        }
    }

    #[test]
    fn row_dir_is_the_modeled_module_home_not_a_bare_folder_name() {
        use std::path::Path;
        assert!(super::row_dir_roster(Path::new("dag/gunbc/recurring_failure_mode")).is_some());
        assert!(super::row_dir_roster(Path::new("gunbc/rung_drop")).is_some());
        assert!(super::row_dir_roster(Path::new("recurring_failure_mode")).is_none());
        assert!(super::row_dir_roster(Path::new("rung_drop")).is_none());
        for roster in DERIVED_ROW_ROSTERS {
            assert_eq!(
                roster.row_dir_rel,
                roster.row_module.replace('.', "/"),
                "directory match is the module path, not a second spelling"
            );
        }
    }
}
