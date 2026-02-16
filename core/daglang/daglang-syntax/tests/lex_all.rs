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

fn expected_dsl_files_sorted() -> Vec<&'static str> {
    vec![
        "cloud/aws/credential.dag",
        "cloud/azure/credential.dag",
        "cloud/gcp/credential.dag",
        "examples/abstract_services.dag",
        "examples/deployment.dag",
        "examples/integration_tests.dag",
        "examples/rich_types.dag",
        "infra/aws/config.dag",
        "infra/aws/resources.dag",
        "infra/aws/services.dag",
        "infra/azure/config.dag",
        "infra/azure/resources.dag",
        "infra/azure/services.dag",
        "infra/core.dag",
        "infra/gcp/config.dag",
        "infra/gcp/resources.dag",
        "infra/gcp/services.dag",
        "infra/spec.dag",
        "pipelines/ci.dag",
        "services/cargo.dag",
        "services/gcp/iam.dag",
        "services/gcp/secret_manager.dag",
        "services/gcp/sts.dag",
        "services/git.dag",
        "services/github/gist.dag",
        "services/shell.dag",
        "shared/dag_util.dag",
        "shared/gist_modes.dag",
        "std/patterns.dag",
        "std/resources.dag",
        "std/types.dag",
        "tools/bootstrap.dag",
        "tools/build.dag",
        "tools/clippy.dag",
        "tools/codegen.dag",
        "tools/dag_viz.dag",
        "tools/deps.dag",
        "tools/docgen.dag",
        "tools/gist.dag",
        "tools/makegen.dag",
        "tools/pragma.dag",
        "tools/testgen.dag",
    ]
}

fn to_relative_unix_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("discovered path should be under dsl root")
        .components()
        .map(|segment| segment.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}

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
