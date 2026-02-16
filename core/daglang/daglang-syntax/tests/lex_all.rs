use daglang_syntax::lexer::Lexer;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_dag_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("dag") {
            out.push(path);
        }
    }

    Ok(())
}

#[test]
fn lex_all_golden_dag_files_without_diagnostics() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
    let mut dag_files = Vec::new();
    collect_dag_files(&dsl_root, &mut dag_files).expect("failed to discover .dag files");
    dag_files.sort();

    assert_eq!(
        dag_files.len(),
        42,
        "expected 42 golden .dag files, found {}",
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
            message.push_str(&format!("- {}\n", path.display()));
            for diagnostic in diagnostics {
                message.push_str(&format!("    {diagnostic}\n"));
            }
        }
        panic!("{message}");
    }
}
