use crate::common::{cached_compile_outcome, CachedCompileOutcome};
use v3_compiler::diagnostics::{apply_correction_and_reparse, Correction, Diagnostic};

fn compile_semantic_fixture(source: &str, file: &str) -> v3_compiler::Dag {
    match cached_compile_outcome(source, file) {
        CachedCompileOutcome::Semantic(dag) => dag,
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
        matches!(
            diagnostic.correction(),
            v3_compiler::diagnostics::Correction::LiveCorrection { .. }
        ),
        "fixture should carry a live correction"
    );
    let fix = diagnostic.correction();
    let repaired = apply_correction_and_reparse(source, file, fix).unwrap_or_else(|error| {
        panic!("correction should apply and reparse for {file}: {fix:?}\nerror: {error:?}")
    });
    match cached_compile_outcome(&repaired, file) {
        CachedCompileOutcome::Clean(_) => {}
        CachedCompileOutcome::Semantic(_) if !require_clean_compile => {}
        CachedCompileOutcome::Semantic(dag) => panic!(
            "applied correction should compile cleanly for {file}: {fix:?}\ndiagnostics: {:?}\nrepaired source:\n{repaired}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
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
fn ambiguous_missing_field_correction_is_deferred() {
    let source = "\
type Pair { left: Int, right: Int }
fn read(x: Pair) -> Int = x.bad
";
    let file = "lane3_db1_ambiguous_missing_field.v3";
    let dag = compile_semantic_fixture(source, file);
    let diagnostic = find_diagnostic(&dag, |diagnostic| {
        matches!(
            diagnostic,
            Diagnostic::ResolveError { name, .. } if name.contains("field `bad` does not exist")
        )
    });
    assert!(
        matches!(
            diagnostic.correction(),
            Correction::DeferredCorrection { reason, .. }
                if reason.contains("left") && reason.contains("right")
        ),
        "ambiguous missing-field repair should defer instead of choosing an arbitrary field: {:?}",
        diagnostic.correction()
    );
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
fn non_exhaustive_match_correction_covers_all_missing_arms() {
    let source = "\
type ABC = A | B | C
fn read(x: ABC) -> Int = match x { A => 1 }
";
    let file = "lane3_db1_non_exhaustive_multiple.v3";
    let dag = compile_semantic_fixture(source, file);
    let diagnostic = find_diagnostic(&dag, |diagnostic| {
        matches!(
            diagnostic,
            Diagnostic::ResolveError { name, .. }
                if name.contains("non-exhaustive match")
                    && name.contains("`B, C`")
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
    assert_fixes_apply_and_recompile(source, file, diagnostic, true);
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
