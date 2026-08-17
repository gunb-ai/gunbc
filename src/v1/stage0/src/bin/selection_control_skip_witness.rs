#![allow(clippy::disallowed_macros)]

//! Affected-set selection-control skip witness (host fast path).
//!
//! Prints `selection_control_not_affected_skip` when the merge-base diff touches no input
//! of the `floor_skip_discovery_witness` control suite (`src/v1/**`, the suite's declared
//! `.dag` entries plus their import closure through `[src/v2, dag]`, and the Cargo/toolchain
//! build config), else `run_selection_control`.
//!
//! Fail-closed by REFUSING, not by widening. A diff-observation or closure-computation failure
//! means the affected set is UNKNOWN, which is a different state from "everything is affected"
//! and has a different remedy — so this bin prints a typed, located, countable diagnostic to
//! stderr and exits NON-ZERO, stopping the line. It never prints a label in that case, so the
//! CI step's string comparison cannot mistake ignorance for an answer, and `bash -e` aborts the
//! step RED rather than silently running the suite and passing.
//!
//! That is DESIGN §5's absorbing-fallback rule ("a failure arm must refuse, never widen") and
//! matches the ruling already recorded in `floor_diff_baseline_law`: a diff-observation failure
//! HALTS with a typed AFFECTED-SET REFUSAL (operator ruling 2026-07-05). Deliberately UNLIKE the
//! `regen_floor_skip_witness` precedent, which suppresses stderr and defaults via a shell
//! fallback — that shape is the thing this one refuses to copy.
//!
//! The whole decision lives in `cli_run::selection_control_skip_label_for_ci`; this bin is just
//! the transport.

use std::process::ExitCode;

use v1_compiler::cli_run::{selection_control_skip_label_for_ci, workspace_root};

fn main() -> ExitCode {
    std::env::set_current_dir(workspace_root()).expect("chdir to workspace root");
    match selection_control_skip_label_for_ci() {
        Ok(label) => {
            println!("{label}");
            ExitCode::SUCCESS
        }
        Err(refusal) => {
            // stderr + non-zero, and NO label on stdout: the step must stop, not proceed.
            eprintln!("{}", refusal.diagnostic());
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use v1_compiler::cli_run::{
        selection_control_input_sources, selection_control_skip_label_for_ci, workspace_root,
        RUN_SELECTION_CONTROL_LABEL, SELECTION_CONTROL_DECLARED_ENTRIES,
        SELECTION_CONTROL_NOT_AFFECTED_SKIP_LABEL,
    };

    #[test]
    fn labels_are_distinct() {
        assert_ne!(
            SELECTION_CONTROL_NOT_AFFECTED_SKIP_LABEL,
            RUN_SELECTION_CONTROL_LABEL
        );
        let label = selection_control_skip_label_for_ci()
            .expect("a clean tree observes its diff and answers with a label");
        assert!(
            label == SELECTION_CONTROL_NOT_AFFECTED_SKIP_LABEL
                || label == RUN_SELECTION_CONTROL_LABEL,
            "unexpected label: {label}"
        );
    }

    /// Every declared entry must exist on disk. A stale path would silently shrink the
    /// closure (the read error is fail-closed at label time, but the roster would be lying),
    /// so this pins the authority against the tree.
    #[test]
    fn declared_entries_exist() {
        let ws = workspace_root();
        for rel in SELECTION_CONTROL_DECLARED_ENTRIES {
            assert!(
                ws.join(rel).is_file(),
                "declared selection-control entry missing from tree: {rel}"
            );
        }
    }

    /// The closure must contain every declared entry, and must be a strict SUPERSET of them
    /// — an import closure that returned only the entries themselves would mean the walk
    /// found no imports, which for these entries is impossible and would silently narrow the
    /// skip decision.
    #[test]
    fn closure_covers_entries_and_reaches_imports() {
        let ws = workspace_root();
        let closure = selection_control_input_sources(&ws).expect("closure must compute");
        for rel in SELECTION_CONTROL_DECLARED_ENTRIES {
            assert!(
                closure.iter().any(|p| p == rel),
                "closure omits declared entry {rel}"
            );
        }
        assert!(
            closure.len() > SELECTION_CONTROL_DECLARED_ENTRIES.len(),
            "closure ({}) did not reach past the {} declared entries — import walk is dead",
            closure.len(),
            SELECTION_CONTROL_DECLARED_ENTRIES.len()
        );
    }
}
