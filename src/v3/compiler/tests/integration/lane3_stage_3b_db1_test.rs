use v3_compiler::diagnostics::{apply_correction_and_reparse, Diagnostic};
use v3_compiler::{compile_to_dag, CompileError};

fn compile_semantic_fixture(source: &str, file: &str) -> v3_compiler::Dag {
    match compile_to_dag(source, file) {
        Err(CompileError::Semantic(dag)) => dag,
        other => panic!("expected semantic failure for {file}, got {other:?}"),
    }
}

fn find_diagnostic(dag: &v3_compiler::Dag, predicate: impl Fn(&Diagnostic) -> bool) -> &Diagnostic {
    dag.diagnostics()
        .iter()
        .find_map(|(_, diagnostic)| predicate(diagnostic).then_some(diagnostic))
        .unwrap_or_else(|| {
            panic!(
                "expected matching diagnostic, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            )
        })
}

fn assert_fixes_apply_and_recompile(
    source: &str,
    file: &str,
    diagnostic: &Diagnostic,
    require_clean_compile: bool,
) {
    assert!(
        !diagnostic.fixes().is_empty(),
        "fixture should carry corrections"
    );
    for fix in diagnostic.fixes() {
        let repaired = apply_correction_and_reparse(source, file, fix).unwrap_or_else(|error| {
            panic!("correction should apply and reparse for {file}: {fix:?}\nerror: {error:?}")
        });
        match compile_to_dag(&repaired, file) {
            Ok(_) => {}
            Err(CompileError::Semantic(_)) if !require_clean_compile => {}
            Err(CompileError::Semantic(dag)) => panic!(
                "applied correction should compile cleanly for {file}: {fix:?}\ndiagnostics: {:?}\nrepaired source:\n{repaired}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            ),
            Err(CompileError::Tokenize(error)) => panic!(
                "applied correction should not tokenize-fail for {file}: {fix:?}\nerror: {error:?}\nrepaired source:\n{repaired}"
            ),
            Err(CompileError::Parse(error)) => panic!(
                "applied correction should not parse-fail for {file}: {fix:?}\nerror: {error:?}\nrepaired source:\n{repaired}"
            ),
        }
    }
}

#[test]
fn missing_field_corrections_apply_to_the_exact_segment_and_compile() {
    let source = "\
type Inner { leaf: Int }
type Outer { ok: Inner }
fn read(x: Outer) -> Int = x.bad.leaf
";
    let file = "lane3_db1_missing_field.v3";
    let dag = compile_semantic_fixture(source, file);
    let diagnostic = find_diagnostic(&dag, |diagnostic| {
        matches!(
            diagnostic,
            Diagnostic::ResolveError { name, .. } if name.contains("field `bad` does not exist")
        )
    });
    assert_fixes_apply_and_recompile(source, file, diagnostic, true);
}

#[test]
fn non_exhaustive_match_corrections_apply_and_compile_when_one_arm_is_missing() {
    let source = "\
type AB = A | B
fn read(x: AB) -> Int = match x { A => 1 }
";
    let file = "lane3_db1_non_exhaustive.v3";
    let dag = compile_semantic_fixture(source, file);
    let diagnostic = find_diagnostic(&dag, |diagnostic| {
        matches!(
            diagnostic,
            Diagnostic::ResolveError { name, .. } if name.contains("non-exhaustive match")
        )
    });
    assert_fixes_apply_and_recompile(source, file, diagnostic, true);
}

#[test]
fn empty_match_seed_corrections_apply_without_parse_breakage() {
    let source = "\
type AB = A | B
fn read(x: AB) -> Int = match x {}
";
    let file = "lane3_db1_empty_match.v3";
    let dag = compile_semantic_fixture(source, file);
    let diagnostic = find_diagnostic(&dag, |diagnostic| {
        matches!(
            diagnostic,
            Diagnostic::ResolveError { name, .. } if name.contains("non-exhaustive match")
        )
    });
    assert_fixes_apply_and_recompile(source, file, diagnostic, false);
}

#[test]
fn type_mismatch_corrections_apply_and_compile() {
    let source = "let x: Bool = 1\n";
    let file = "lane3_db1_type_mismatch.v3";
    let dag = compile_semantic_fixture(source, file);
    let diagnostic = find_diagnostic(&dag, |diagnostic| {
        matches!(diagnostic, Diagnostic::TypeMismatch { .. })
    });
    assert_fixes_apply_and_recompile(source, file, diagnostic, true);
}

#[test]
fn unresolved_call_corrections_apply_and_compile() {
    let source = "fn bad() -> Int = nope(1)\n";
    let file = "lane3_db1_unresolved_call.v3";
    let dag = compile_semantic_fixture(source, file);
    let diagnostic = find_diagnostic(
        &dag,
        |diagnostic| matches!(diagnostic, Diagnostic::ResolveError { name, .. } if name == "nope"),
    );
    assert_fixes_apply_and_recompile(source, file, diagnostic, true);
}

#[test]
fn termination_corrections_apply_and_compile() {
    let source = "fn diverge(x: Int) -> Int = diverge(x)\n";
    let file = "lane3_db1_termination.v3";
    let dag = compile_semantic_fixture(source, file);
    let diagnostic = find_diagnostic(&dag, |diagnostic| {
        matches!(
            diagnostic,
            Diagnostic::ResolveError { name, .. }
                if name.contains("cannot prove recursion in `diverge` terminates")
        )
    });
    assert_fixes_apply_and_recompile(source, file, diagnostic, true);
}
