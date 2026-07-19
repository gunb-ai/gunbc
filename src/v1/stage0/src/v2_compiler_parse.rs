// Seed realization for v2.compiler.parse (Gate-A flip wave, neat-swift-795).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.parse
// is emitted-only and the behavioral harness is modeled (sp_scaffold_dissolution_trigger).

/// Mirrors `data prepared_grammar_carrier_note` in src/v2/compiler/02_parse.dag.
pub fn prepared_grammar_carrier_note() -> String {
    "PreparedGrammar is the once-per-grammar carrier: well-formedness, the five grammar_validate_for_parse checks, and the FIRST/nullable analysis (GrammarFirstAnalysis) are pure functions of the grammar, so prepare_grammar runs them ONCE and parse_module_prepared reuses the result per module. Previously parse_module re-ran the whole 5-check validation AND compute_grammar_first_analysis was computed twice (once in the ambiguity check, once in parse_table_for_production) on EVERY parse. Hoisting prepare_grammar above the assemble fold makes a door call over a K-module closure validate once, not K times — correctness-by-construction (the prepare sits above fold_list), no timing gate. The cache-interface fields on ParseTableRealization (grammar_digest/provider_id/materialization) stay for the cross-call content-addressed memo, which is gated on wiring the currently Miss-stubbed cache backend.".to_string()
}
