#![allow(clippy::disallowed_macros)]

//! Affected-set compile-contract admission witness (host fast path).
//!
//! Prints `compile_contract_not_affected_skip` when the merge-base diff touches no input
//! of the v1-compiler-tests compile-contract gate (`src/v1/tests/**`, `src/v1/stage0/**`,
//! Cargo/toolchain build config), else `run_compile_contract`.
//!
//! Fail-closed by RUNNING on uncertainty, never by skipping: diff-observation failure,
//! empty diff, and departed closure paths all answer `run_compile_contract` so unknown
//! state never narrows coverage. Deliberately UNLIKE `selection_control_skip_witness`,
//! which refuses on observation failure — here the gate must still compile when the
//! affected set is unknown.
//!
//! The whole decision lives in `cli_run::compile_contract_admission_label_for_ci`; this bin
//! is just the transport.

use std::process::ExitCode;

use v1_compiler::cli_run::{compile_contract_admission_label_for_ci, workspace_root};

fn main() -> ExitCode {
    std::env::set_current_dir(workspace_root()).expect("chdir to workspace root");
    println!("{}", compile_contract_admission_label_for_ci());
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use v1_compiler::cli_run::{
        compile_contract_admission_label_for_ci, COMPILE_CONTRACT_NOT_AFFECTED_SKIP_LABEL,
        RUN_COMPILE_CONTRACT_LABEL,
    };

    #[test]
    fn labels_are_distinct() {
        assert_ne!(
            COMPILE_CONTRACT_NOT_AFFECTED_SKIP_LABEL,
            RUN_COMPILE_CONTRACT_LABEL
        );
        let label = compile_contract_admission_label_for_ci();
        assert!(
            label == COMPILE_CONTRACT_NOT_AFFECTED_SKIP_LABEL
                || label == RUN_COMPILE_CONTRACT_LABEL,
            "unexpected label: {label}"
        );
    }
}
