// Seed realization for v2.compiler.program_assembly (Gate-A flip lane).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.program_assembly
// is emitted-only and the behavioral harness is modeled (spa_scaffold_dissolution_trigger).

/// Mirrors `data program_assembly_prepare_once_note` in src/v2/compiler/program_assembly.dag.
pub fn program_assembly_prepare_once_note() -> String {
    "prepare_grammar is hoisted ABOVE fold_list here so the grammar is well-formed-checked, validated (5 checks), and FIRST/nullable-analyzed ONCE per assembly — then parse_module_prepared reuses the PreparedGrammar for every module in the ingest. A door call over a K-module import closure therefore validates once, not K times (the recompute-per-module cost this dissolves). Empty-ingest is guarded so it never pays the one-time prepare for zero modules. Fail-closed: an invalid grammar fails the whole assembly (ProgramAssemblyFoldFailed) before any module is parsed, carrying the validation diagnostic — never a per-module silent skip.".to_string()
}
