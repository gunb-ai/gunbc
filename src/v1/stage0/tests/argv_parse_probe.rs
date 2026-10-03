#![allow(clippy::disallowed_macros)]

//! THROWAWAY parse probe for the typed-argv census lens (deleted before commit).
//! Prints the exact v1 parse error for each argv-word-construction file.

use im::HashMap;
use std::rc::Rc;
use v1_compiler::cli_run::workspace_root;
use v1_compiler::v1_compiler_parse::ParseResult;
use v1_compiler::v1_std_core::{diagnostic_to_message, diagnostic_to_span};

fn parse_path(relative_path: &str) -> Result<(), String> {
    let path = workspace_root().join(relative_path);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
    let tokens = v1_compiler::v1_compiler_tokenize::tokenize(
        source.clone(),
        relative_path.to_string(),
        v1_compiler::extdeps_languages_dag_syntax::dag_parse_environment(),
    );
    let source_index =
        v1_compiler::v1_std_core::build_newline_index(relative_path.to_string(), source.clone());
    let mut source_indices = HashMap::new();
    source_indices.insert(relative_path.to_string(), source_index);
    let result: Rc<ParseResult> =
        v1_compiler::v1_compiler_parse::parse(tokens, Rc::new(source_indices));
    match result.error.clone() {
        Some(err) => {
            let s = diagnostic_to_span(err.diagnostic.clone());
            let line = source[..(s.start.max(0) as usize).min(source.len())]
                .chars()
                .filter(|c| *c == '\n')
                .count()
                + 1;
            Err(format!(
                "{}:{} parse error: {}",
                relative_path,
                line,
                diagnostic_to_message(err.diagnostic.clone())
            ))
        }
        None => Ok(()),
    }
}

#[test]
fn probe_parses_argv_word_construction_files() {
    let files = [
        "src/v2/lens/argv_word_construction.dag",
        "dag/gunbc/instruments/argv_word_construction_census.dag",
        "src/v2/test/claim/argv_word_construction/census_test.dag",
        "test/fixture_roots/argv_word_construction/host_words.dag",
    ];
    let mut failures = Vec::new();
    for f in files {
        match parse_path(f) {
            Ok(()) => println!("PARSE-OK {}", f),
            Err(e) => {
                println!("PARSE-FAIL {}", e);
                failures.push(e);
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} files failed",
        failures.len(),
        files.len()
    );
}
