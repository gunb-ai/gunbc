#![allow(clippy::disallowed_macros)]

//! Regen self-host fixed-point affected-set skip witness (host fast path).
//!
//! Prints `regen_not_affected_skip` when the merge-base diff touches no regen input
//! (the `[src/v1, dag]` compile closure the two regen gates share), else `run_regen`.
//! Fail-closed: any diff-observation or closure-computation failure prints `run_regen`.
//! Mirrors `compile_clean_floor_skip_witness` — the whole decision lives in the lib
//! (`cli_run::regen_floor_skip_label_for_ci`); this bin is just the transport.

use std::process::ExitCode;

use v1_compiler::cli_run::{regen_floor_skip_label_for_ci, workspace_root};

fn main() -> ExitCode {
    std::env::set_current_dir(workspace_root()).expect("chdir to workspace root");
    println!("{}", regen_floor_skip_label_for_ci());
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use v1_compiler::cli_run::{
        regen_floor_skip_label_for_ci, REGEN_NOT_AFFECTED_SKIP_LABEL, RUN_REGEN_LABEL,
    };

    #[test]
    fn labels_are_distinct() {
        assert_ne!(REGEN_NOT_AFFECTED_SKIP_LABEL, RUN_REGEN_LABEL);
        let label = regen_floor_skip_label_for_ci();
        assert!(
            label == REGEN_NOT_AFFECTED_SKIP_LABEL || label == RUN_REGEN_LABEL,
            "unexpected label: {label}"
        );
    }
}
