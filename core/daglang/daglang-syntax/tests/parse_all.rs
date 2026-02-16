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
fn parse_all_golden_dag_files() {
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
