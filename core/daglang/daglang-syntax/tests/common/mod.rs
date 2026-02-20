use std::fs;
use std::path::{Path, PathBuf};

// Test infrastructure: filesystem access for test fixtures
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
pub fn collect_dag_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
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

pub fn expected_dsl_files_sorted() -> Vec<&'static str> {
    vec![
        "cloud/aws/credential.dag",
        "cloud/aws/credential_test.dag",
        "cloud/azure/credential.dag",
        "cloud/azure/credential_test.dag",
        "cloud/gcp/credential.dag",
        "cloud/gcp/credential_test.dag",
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
        "pipelines/ci_test.dag",
        "services/cargo.dag",
        "services/gcp/iam.dag",
        "services/gcp/secret_manager.dag",
        "services/gcp/sts.dag",
        "services/git.dag",
        "services/github/gist.dag",
        "services/llm/anthropic.dag",
        "services/llm/openai.dag",
        "services/llm/openai_test.dag",
        "services/shell.dag",
        "shared/dag_util.dag",
        "shared/gist_modes.dag",
        "std/patterns.dag",
        "std/resources.dag",
        "std/types.dag",
        "tools/bootstrap.dag",
        "tools/bootstrap_test.dag",
        "tools/build.dag",
        "tools/clippy.dag",
        "tools/clippy_test.dag",
        "tools/codegen.dag",
        "tools/dag_viz.dag",
        "tools/dag_viz_test.dag",
        "tools/deps.dag",
        "tools/deps_test.dag",
        "tools/docgen.dag",
        "tools/gist.dag",
        "tools/gist_test.dag",
        "tools/makegen.dag",
        "tools/makegen_test.dag",
        "tools/pragma.dag",
        "tools/pragma_test.dag",
        "tools/review_test.dag",
        "tools/testgen.dag",
        "tools/testgen_test.dag",
    ]
}

pub fn to_relative_unix_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("discovered path should be under dsl root")
        .components()
        .map(|segment| segment.as_os_str().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join("/")
}
