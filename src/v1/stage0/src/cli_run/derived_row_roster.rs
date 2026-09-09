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
/// Module path of the DERIVED roster itself: `ROW_MODULE` plus the `ROSTER_BASENAME` stem.
///
/// ONE SPELLING, BECAUSE `rostered_row_join` LOOKS THIS MODULE UP BY NAME. `ENROLLED_ROW_TYPES`
/// carries the roster's module identity and `run_rostered_row_join` resolves it with
/// `index_get(index, enrolled.roster_module)`, against the module header `render_roster` WRITES.
/// Those two are the speller pair that must agree (DESIGN section 3).
///
/// A DIVERGENCE FAILS CLOSED: the lookup misses and raises the typed, located
/// `RosterModuleAbsent` finding, and the run goes red naming the module. The cost of a second
/// spelling is a confusing refusal about a module that appears to exist, not a silent one.
///
/// NOT THE NAMESPACE WAVE'S DISCOVERY, BUT NOW ONE OF ITS CLASSIFIERS.
/// `run_required_wave_admission` still finds this file by PATH -- `is_derived_roster_path` and
/// `roster_root_prefix`, both composed from `ROW_DIR_REL` and `ROSTER_BASENAME` -- and no
/// discovery path reads this constant. An earlier version of this comment said the const was
/// consumed by "the namespace wave's base reconstruction"; that was false and the correction
/// stays, because the true consumer is narrower and later: `derived_generator_input_binding`
/// reads it to decide whether the module being classified IS this generator, and reads
/// `ROW_MODULE` to decide whether what it binds to is one of this generator's declared inputs.
///
/// THAT CONSUMER IS WHY THIS IS A CONSTANT AND NOT A STRIP OF `ROSTER_BASENAME`. Recovering the
/// name from the filename means stripping `.dag` and choosing what to do when the suffix is
/// absent, and every such choice is a guess about a name this module owns. The guess would sit
/// on an AUTO-ADMISSION path, where a fabricated fallback keeps comparing against a plausible
/// name instead of refusing -- the fabricated-plausible-output DESIGN section 5 forbids.
/// Declaring it once leaves no default to guess.
///
/// It cannot be `concat!` of its parts because `ROW_MODULE` is a `const` and not a literal token,
/// so the composition is ASSERTED by `the_roster_module_is_the_row_module_plus_the_roster_stem`
/// rather than constructed.
///
/// THAT ASSERTION IS NOT A WALL, AND THE RUNG IS 1 -- MITIGATION. The test is `#[cfg(test)]`
/// under `repo_self_test_command`, which no CI step runs: the 2026-09-04 runner-capacity ruling
/// deleted the `rust-unit-tests` job and its loss stands as `gunbc.rung_drop`
/// `rust_unit_tests_off_the_merge_path`. The required clippy lane COMPILES this test and executes
/// it never -- DESIGN "Building & checks": "the test targets are compiled by the clippy step and
/// run by nobody" -- and a rename of either part still typechecks, so nothing on the acceptance
/// path goes red for it.
///
/// Next-rung trigger: the parts composable in a const context, which makes a second spelling
/// unconstructible rather than assert-checked -- rung 4, not 2, because it removes the
/// constructor instead of adding an executing check. Restoring an executing unit-test lane
/// reaches only rung 2 and is the weaker of the two.
pub const ROSTER_MODULE: &str = "gunbc.recurring_failure_mode.roster";
pub const ROW_DIR_REL: &str = "gunbc/recurring_failure_mode";

/// Is `rel` the DERIVED roster itself, under any sweep root?
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
pub fn is_derived_roster_path(rel: &str) -> bool {
    rel.ends_with(&format!("/{ROW_DIR_REL}/{ROSTER_BASENAME}"))
}

/// The sweep-root prefix of a derived roster path: `dag/gunbc/recurring_failure_mode/roster.dag`
/// -> `dag`. `None` when `rel` is not a roster path.
pub fn roster_root_prefix(rel: &str) -> Option<&str> {
    let suffix = format!("/{ROW_DIR_REL}/{ROSTER_BASENAME}");
    rel.strip_suffix(&suffix)
}

/// The roster this row directory would derive from an ARBITRARY path listing rather than from
/// the filesystem — the base tree's answer, rendered by the SAME function the writer uses, so
/// there is one authority for the roster's bytes and not a second reconstruction beside it.
///
/// `paths` is a repo-relative listing (`git ls-tree -r --name-only <base>`); `root` is the sweep
/// root the roster lives under. Returns `None` when the base tree carries NO row files there:
/// that is the roster module being genuinely absent at the base, which is a different state from
/// a present empty list and must not be fabricated as one.
pub fn roster_from_path_listing<'a>(
    paths: impl IntoIterator<Item = &'a str>,
    root: &str,
) -> Option<String> {
    let dir = format!("{root}/{ROW_DIR_REL}/");
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
        names.push(stem.to_string());
    }
    if names.is_empty() {
        return None;
    }
    names.sort();
    Some(render_roster(&names))
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

pub fn render_roster(names: &[String]) -> String {
    let mut out = format!(
        "module {ROSTER_MODULE}\n\
         \n\
         // DERIVED from sibling RecurringFailureMode row files. Do not hand-edit.\n\
         // Membership is the directory; order is the sorted filename stem, which is the\n\
         // declaration name. An append is a new file in this directory, never an edit here.\n\
         \n\
         import std.types {{ List }}\n\
         import {ROW_MODULE} {{ RecurringFailureMode }}\n"
    );
    for name in names {
        out.push_str("import ");
        out.push_str(ROW_MODULE);
        out.push_str(".");
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
    fn the_roster_module_is_the_row_module_plus_the_roster_stem() {
        // THE COMPOSITION THE CONST CANNOT EXPRESS. `ROSTER_MODULE` is a literal because
        // `concat!` takes literal tokens and `ROW_MODULE` is a const; this asserts the relation
        // the literal stands for, so renaming either part reds here rather than silently
        // handing the row join a name it will fail to resolve.
        assert_eq!(
            super::ROSTER_MODULE,
            format!(
                "{}.{}",
                super::ROW_MODULE,
                super::ROSTER_BASENAME.trim_end_matches(".dag")
            ),
            "the derived roster's module is its row module plus the roster file's stem"
        );
    }

    #[test]
    fn absent_is_not_an_empty_list_render_of_no_names_is_a_present_empty_literal() {
        let body = super::render_roster(&[]);
        assert!(
            // THE LITERAL IS THE POINT: an oracle built from `ROSTER_MODULE` would assert
            // `render_roster` against the same const `render_roster` renders from, so it could
            // not fail on a wrong module name -- `measure() == measure()` (DESIGN section 5).
            // This spelling is an INDEPENDENT referent and must stay hand-written.
            body.contains("module gunbc.recurring_failure_mode.roster"),
            "absence of members is a present module with an empty list, not a missing module"
        );
        assert!(body.contains("= [\n]\n"));
    }

    #[test]
    fn a_base_tree_with_no_row_files_has_no_roster_module_rather_than_an_empty_one() {
        // The two absences are different states with different remedies: a base that never
        // carried the row directory has NO roster module, while a present `= []` asserts the
        // directory exists and is empty. Fabricating the second from the first would give every
        // name in the head roster a base side to be compared against.
        assert_eq!(
            super::roster_from_path_listing(["dag/gunbc/other/thing.dag"], "dag"),
            None
        );
    }

    #[test]
    fn the_base_side_is_the_base_listing_not_the_head_directory() {
        let body = super::roster_from_path_listing(
            [
                "dag/gunbc/recurring_failure_mode/beta.dag",
                "dag/gunbc/recurring_failure_mode/alpha.dag",
                "dag/gunbc/recurring_failure_mode/roster.dag",
                "dag/gunbc/recurring_failure_mode/nested/deep.dag",
                "dag/gunbc/recurring_failure_mode/notes.md",
            ],
            "dag",
        )
        .expect("two row files at the base are a present roster");
        assert!(body.contains("import gunbc.recurring_failure_mode.alpha { alpha }"));
        assert!(body.contains("= [\n  alpha,\n  beta,\n]\n"));
        // The roster never rosters itself, and membership is the directory, not its subtrees.
        assert!(!body.contains("roster,"));
        assert!(!body.contains("deep"));
    }

    #[test]
    fn a_derived_roster_path_is_recognised_under_any_sweep_root() {
        assert!(super::is_derived_roster_path(
            "dag/gunbc/recurring_failure_mode/roster.dag"
        ));
        assert_eq!(
            super::roster_root_prefix("dag/gunbc/recurring_failure_mode/roster.dag"),
            Some("dag")
        );
        assert!(!super::is_derived_roster_path(
            "dag/gunbc/recurring_failure_mode/some_row.dag"
        ));
        assert_eq!(
            super::roster_root_prefix("dag/gunbc/recurring_failure_mode/some_row.dag"),
            None
        );
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
