use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn occurrence_identity_debt_receipts_execute() {
    let root = workspace_root();
    let source_roots = vec![
        root.join("dag").to_string_lossy().into_owned(),
        root.join("src/v1").to_string_lossy().into_owned(),
    ];
    let entry = root
        .join("dag/test/manual/occurrence_identity_debt_receipt_test.dag")
        .to_string_lossy()
        .into_owned();
    let claims = [
        "w_rebuilt_reference_identity_preservation_executes_holds",
        "w_structurally_equal_distinct_occurrences_execute_holds",
        "w_pattern_declaration_reachability_executes_holds",
        "w_same_spelling_parser_declaration_isolation_executes_holds",
    ]
    .into_iter()
    .map(|function| (entry.clone(), function.to_string()))
    .collect::<Vec<_>>();

    assert!(
        v1_compiler::cli_run::run_claims_in_process(
            &source_roots,
            &claims,
            v1_compiler::v1_interpreter::ExecutionMode::Hermetic,
        ),
        "all four occurrence-identity debt receipts must remain enrolled and green",
    );
}
