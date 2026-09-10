//! THE DECODE SEAM THE BASE ENVIRONMENT LOADER STANDS ON, EXERCISED AGAINST AN INDEPENDENT ORACLE.
//!
//! `cli_run::base_parse_environment` reads a revision's parse environment out of source so that
//! revision can be parsed under its own grammar. The half most able to fail silently is the decode:
//! an evaluated data item is an interpreter `Value`, and the tokenizer needs a typed
//! `ParseEnvironment`.
//!
//! The oracle here is `dag_parse_environment()` -- the environment COMPILED INTO this binary from
//! the same declarations the loader evaluates from source. Running the loader over the live tree
//! must reproduce it exactly. That is an independent referent rather than a restatement: the two
//! agree only if evaluation, wire encoding and deserialization all preserved the environment.
//! Asserting that the value merely decoded, or spot-checking one keyword, would pass over a decode
//! that silently dropped every operator row.
//!
//! The test calls the loader's own public entry point rather than re-implementing its body, so a
//! defect in the loader reddens this rather than being masked by a second copy of the logic.

use v1_compiler::cli_run::namespace_wave_admission::evaluate_environment_in;
use v1_compiler::cli_run::workspace_root;
use v1_compiler::extdeps_languages_dag_syntax::dag_parse_environment;

/// The environment the loader reads from source equals the one compiled into this binary.
///
/// Whole-value equality rather than a field probe: every item form, every operator row with its
/// binding powers, every keyword literal and both keyword rosters have to survive, because the
/// loader hands the WHOLE environment to the tokenizer and parser.
#[test]
fn the_loader_reproduces_the_compiled_in_environment() {
    let root = workspace_root();
    let loaded = match evaluate_environment_in(&root, "HEAD") {
        Ok(env) => env,
        Err(e) => panic!("loader refused on the live tree: {e}"),
    };
    let compiled = dag_parse_environment();
    assert_eq!(
        *loaded, *compiled,
        "environment read from source diverged from the compiled-in one"
    );
}

/// The oracle is not vacuous.
///
/// Without this, a load that produced an empty environment would pass the equality test on a day
/// the oracle was also empty. It pins the control itself -- the half a positive control usually
/// leaves unstated.
#[test]
fn the_compiled_in_environment_is_not_empty() {
    let compiled = dag_parse_environment();
    assert!(
        !compiled.syntax_spec.item_forms.is_empty(),
        "oracle has no item forms, so equality against it would prove nothing"
    );
    assert!(
        !compiled.syntax_spec.operators.is_empty(),
        "oracle has no operators, so equality against it would prove nothing"
    );
    assert!(
        !compiled.non_name_keywords.is_empty(),
        "oracle has no non-name keywords, so equality against it would prove nothing"
    );
}
