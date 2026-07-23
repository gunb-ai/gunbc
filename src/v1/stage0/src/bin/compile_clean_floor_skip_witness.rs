#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;

use v1_compiler::cli_run::{documentation_only_floor_skip_label_for_ci, workspace_root};

fn main() -> ExitCode {
    std::env::set_current_dir(workspace_root()).expect("chdir to workspace root");
    match documentation_only_floor_skip_label_for_ci() {
        Ok(label) => {
            println!("{label}");
            ExitCode::SUCCESS
        }
        Err(reason) => {
            eprintln!("documentation-only floor skip witness: refused ({reason})");
            ExitCode::from(1)
        }
    }
}

#[cfg(test)]
mod tests {
    use v1_compiler::cli_run::{
        documentation_only_floor_skip_label_for_ci, DOCUMENTATION_ONLY_FLOOR_SKIP_LABEL,
        RUN_FULL_FLOOR_LABEL,
    };

    #[test]
    fn labels_are_distinct() {
        assert_ne!(DOCUMENTATION_ONLY_FLOOR_SKIP_LABEL, RUN_FULL_FLOOR_LABEL);
        let label = documentation_only_floor_skip_label_for_ci().expect("witness label");
        assert!(
            label == DOCUMENTATION_ONLY_FLOOR_SKIP_LABEL || label == RUN_FULL_FLOOR_LABEL,
            "unexpected label: {label}"
        );
    }
}
