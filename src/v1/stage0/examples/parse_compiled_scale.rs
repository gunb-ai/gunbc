//! adhoc-c328b166-bca forensics probe (not wired into any build/CI gate):
//! times COMPILED (native) tokenize+parse of fixed .dag files via the v1
//! seed's own generated parser, to calibrate the interpretation-tax ratio
//! against the same files run through v1_interpreter.rs.
use std::time::Instant;
use im_rc::HashMap;
use v1_compiler::cli_run::workspace_root;
use v1_compiler::v1_std_core::build_newline_index;

fn parse_one(path: &str) {
    let full = workspace_root().join(path);
    let source = std::fs::read_to_string(&full).unwrap_or_else(|e| panic!("read {}: {}", path, e));
    let t0 = Instant::now();
    let tokens = v1_compiler::v1_compiler_tokenize::tokenize(source.clone(), path.to_string());
    let t_tok = t0.elapsed();
    let source_index = build_newline_index(path.to_string(), source.clone());
    let mut source_indices = HashMap::new();
    source_indices.insert(path.to_string(), source_index);
    let t1 = Instant::now();
    let result = v1_compiler::v1_compiler_parse::parse(tokens, std::rc::Rc::new(source_indices));
    let t_parse = t1.elapsed();
    let ok = result.error.is_none() && result.module.is_some();
    println!(
        "{path}\twords={}\ttokenize_us={}\tparse_us={}\tok={}",
        source.split_whitespace().count(),
        t_tok.as_micros(),
        t_parse.as_micros(),
        ok
    );
}

fn main() {
    for path in [
        "src/v2/std/witness.dag",
        "src/v2/std/execution_mode.dag",
        "src/v2/std/datetime.dag",
        "src/v2/std/node.dag",
        "src/v2/std/grammar.dag",
    ] {
        parse_one(path);
    }
}
