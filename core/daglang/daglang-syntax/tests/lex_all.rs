use daglang_syntax::lexer::Lexer;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

mod common;

use common::{collect_dag_files, expected_dsl_files_sorted, to_relative_unix_path};

// Test infrastructure: filesystem access for test fixtures
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[test]
fn lex_all_golden_dag_files_without_diagnostics() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
    let mut dag_files = Vec::new();
    collect_dag_files(&dsl_root, &mut dag_files).expect("failed to discover .dag files");
    dag_files.sort();
    let discovered_files: Vec<String> = dag_files
        .iter()
        .map(|path| to_relative_unix_path(&dsl_root, path))
        .collect();
    let expected_files: Vec<String> = expected_dsl_files_sorted()
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(
        discovered_files, expected_files,
        "golden corpus inventory changed unexpectedly"
    );

    assert_eq!(
        dag_files.len(),
        73,
        "expected 73 golden .dag files, found {}",
        dag_files.len()
    );

    let mut failures = Vec::new();

    for path in dag_files {
        let source = fs::read_to_string(&path).expect("failed to read .dag source");
        let (_tokens, diagnostics) = Lexer::tokenize_with_diagnostics(&source);
        if !diagnostics.is_empty() {
            let file = path.to_string_lossy().to_string();
            let rendered: Vec<String> = diagnostics
                .iter()
                .map(|diagnostic| diagnostic.clone().with_file(file.clone()).render())
                .collect();
            failures.push((path, rendered));
        }
    }

    if !failures.is_empty() {
        let mut message = String::from("failed to lex golden .dag files:\n");
        for (path, diagnostics) in failures {
            let _ = writeln!(&mut message, "- {}", path.display());
            for diagnostic in diagnostics {
                let _ = writeln!(&mut message, "    {diagnostic}");
            }
        }
        panic!("{message}");
    }
}
