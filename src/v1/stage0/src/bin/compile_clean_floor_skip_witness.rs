#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;

use v1_compiler::cli_run::{documentation_only_floor_skip_label_for_ci, workspace_root};

fn main() -> ExitCode {
    std::env::set_current_dir(workspace_root()).expect("chdir to workspace root");
    match documentation_only_floor_skip_label_for_ci() {
        Ok(disposition) => {
            println!("{}", disposition.witness_label());
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
        documentation_only_floor_skip_label_for_ci, DocumentationOnlyFloorFullFloorCause,
        DocumentationOnlyFloorSkipDisposition, DOCUMENTATION_ONLY_FLOOR_SKIP_LABEL,
        RUN_FULL_FLOOR_DIFF_OBSERVATION_FAILED_LABEL, RUN_FULL_FLOOR_EMPTY_DIFF_LABEL,
        RUN_FULL_FLOOR_NON_DOCS_CHANGE_LABEL, RUN_FULL_FLOOR_SCOPING_INACTIVE_LABEL,
    };

    #[test]
    fn labels_are_distinct() {
        assert_ne!(
            DOCUMENTATION_ONLY_FLOOR_SKIP_LABEL,
            RUN_FULL_FLOOR_EMPTY_DIFF_LABEL
        );
        let disposition = documentation_only_floor_skip_label_for_ci().expect("witness label");
        let label = disposition.witness_label();
        assert!(
            label == DOCUMENTATION_ONLY_FLOOR_SKIP_LABEL
                || label == RUN_FULL_FLOOR_SCOPING_INACTIVE_LABEL
                || label == RUN_FULL_FLOOR_DIFF_OBSERVATION_FAILED_LABEL
                || label == RUN_FULL_FLOOR_EMPTY_DIFF_LABEL
                || label == RUN_FULL_FLOOR_NON_DOCS_CHANGE_LABEL,
            "unexpected label: {label}"
        );
        assert!(
            matches!(
                disposition,
                DocumentationOnlyFloorSkipDisposition::DocumentationOnlySkip
                    | DocumentationOnlyFloorSkipDisposition::RunFullFloor { .. }
            ),
            "unexpected disposition: {disposition:?}"
        );
        if let DocumentationOnlyFloorSkipDisposition::RunFullFloor { cause } = disposition {
            assert!(matches!(
                cause,
                DocumentationOnlyFloorFullFloorCause::ScopingInactive
                    | DocumentationOnlyFloorFullFloorCause::DiffObservationFailed { .. }
                    | DocumentationOnlyFloorFullFloorCause::EmptyDiff
                    | DocumentationOnlyFloorFullFloorCause::NonDocsChange
            ));
        }
    }
}
