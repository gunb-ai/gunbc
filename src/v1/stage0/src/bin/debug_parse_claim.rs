#![allow(clippy::disallowed_macros)]

use std::process::ExitCode;
use std::rc::Rc;

use v1_compiler::v1_compiler_parse::parse;
use v1_compiler::v1_compiler_tokenize::tokenize;
use v1_compiler::v1_std_core::{
    build_newline_index, byte_to_line_col, diagnostic_to_message, diagnostic_to_span,
};

fn main() -> ExitCode {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "dag/test/claim/pattern_binder_declaration_span_witness_test.dag".to_string());
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("debug_parse_claim: read {path}: {err}");
            return ExitCode::from(1);
        }
    };
    let tokens = tokenize(content.clone(), path.clone());
    let source_index = build_newline_index(path.clone(), content.clone());
    let mut indices = im::HashMap::new();
    indices.insert(path.clone(), source_index);
    let result = parse(tokens, Rc::new(indices));
    match &result.error {
        Some(err) => {
            let span = diagnostic_to_span(err.diagnostic.clone());
            let lc = byte_to_line_col(build_newline_index(path.clone(), content.clone()), span.start);
            eprintln!(
                "{}:{}:{}: {}",
                span.file,
                lc.line,
                lc.col,
                diagnostic_to_message(err.diagnostic.clone())
            );
            ExitCode::from(1)
        }
        None => {
            println!("parse clean");
            ExitCode::SUCCESS
        }
    }
}
