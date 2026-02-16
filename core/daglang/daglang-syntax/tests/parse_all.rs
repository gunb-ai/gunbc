use std::fs;
use std::path::Path;

mod common;

use common::{collect_dag_files, expected_dsl_files_sorted, to_relative_unix_path};

#[test]
fn parse_all_golden_dag_files() {
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
        42,
        "expected 42 golden .dag files, found {}",
        dag_files.len()
    );

    let mut failures = Vec::new();

    for path in &dag_files {
        let source = fs::read_to_string(path).expect("failed to read .dag source");
        if let Err(errors) = daglang_syntax::parser::parse(&source) {
            let rendered: Vec<String> = errors
                .iter()
                .map(|err| err.format_with_source(path, &source))
                .collect();
            failures.push((path.display().to_string(), rendered));
        }
    }

    if !failures.is_empty() {
        let mut message = String::from("failed to parse golden .dag files:\n");
        for (file, errors) in failures {
            message.push_str(&format!("- {file}\n"));
            for error in errors {
                message.push_str(&format!("    {error}\n"));
            }
        }
        panic!("{message}");
    }
}
