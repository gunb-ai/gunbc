use std::collections::HashSet;
use std::fs;
use std::path::Path;

mod common;

use common::{collect_dag_files, expected_dsl_files_sorted, to_relative_unix_path};

fn expected_module_from_relative_path(relative_path: &str) -> String {
    relative_path.trim_end_matches(".dag").replace('/', ".")
}

// Test infrastructure: filesystem access for test fixtures
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[test]
fn corpus_module_declarations_match_file_paths() {
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

    let mut failures = Vec::new();
    let mut seen_modules = HashSet::new();

    for file in &dag_files {
        let source = fs::read_to_string(file).expect("failed to read .dag source");
        let parsed = daglang_syntax::parser::parse(&source)
            .unwrap_or_else(|errors| panic!("failed to parse {}: {errors:?}", file.display()));
        let relative = to_relative_unix_path(&dsl_root, file);
        let expected_module = expected_module_from_relative_path(&relative);
        let actual_module = parsed
            .module_path
            .as_ref()
            .map(|module| module.node.segments.join("."));

        match actual_module {
            Some(actual) if actual == expected_module => {
                if !seen_modules.insert(actual.clone()) {
                    failures.push(format!(
                        "{relative}: duplicate parsed module path '{actual}'"
                    ));
                }
            }
            Some(actual) => failures.push(format!(
                "{relative}: expected module '{expected_module}', found '{actual}'"
            )),
            None => failures.push(format!(
                "{relative}: missing module declaration, expected '{expected_module}'"
            )),
        }
    }

    assert!(
        failures.is_empty(),
        "module declaration mismatches:\n{}",
        failures.join("\n")
    );
    assert_eq!(
        seen_modules.len(),
        68,
        "expected 68 unique module declarations in corpus"
    );
}
