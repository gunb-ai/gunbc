// Seed realization for v2.compiler.tokenize (Wave 2 Gate-A flip lane).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.tokenize
// is emitted-only and the behavioral harness is modeled (st_scaffold_dissolution_trigger).

/// Mirrors `data tokenize_module_authority_note` in src/v2/compiler/01_tokenize.dag.
pub fn tokenize_module_authority_note() -> String {
    "Wave 2 Gate-A flip lane (01_tokenize self-emit): data-driven lexing via ModeledLexRules/LexRuleSet fold — no hand-rolled tokenizer. Curated seed-linked behavioral receipt compares emitted vs seed oracle constant; full token-stream equivalence oracle deferred until emit Rc/Optional surface greens (#6775).".to_string()
}
