#![allow(clippy::disallowed_methods)]

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

#[test]
fn github_actions_cache_excludes_target_tree() {
    let path = workspace_root().join(".github/workflows/ci.yml");
    let yaml = fs::read_to_string(&path).expect("read generated GitHub Actions workflow");

    assert!(
        yaml.contains("~/.cargo/registry/cache/"),
        "expected cargo registry cache path in {}",
        path.display()
    );
    assert!(
        !yaml.contains("\n            target/\n"),
        "GitHub Actions cache must not restore target/; found forbidden entry in {}",
        path.display()
    );
    assert!(
        yaml.contains("cargo run -p gunbc-app --bin gunbc-codegen -- codegen"),
        "GitHub Actions workflow must regenerate real generated binaries before running gunbc-ci in {}",
        path.display()
    );
}

#[test]
fn gitlab_cache_excludes_target_tree() {
    let path = workspace_root().join(".gitlab-ci.yml");
    let yaml = fs::read_to_string(&path).expect("read generated GitLab CI pipeline");

    assert!(
        yaml.contains("\n    - .cargo/\n"),
        "expected local cargo cache path in {}",
        path.display()
    );
    assert!(
        !yaml.contains("\n    - target/\n"),
        "GitLab cache must not restore target/; found forbidden entry in {}",
        path.display()
    );
}

#[test]
fn cigen_is_typed_assembly_not_inline_yaml_builder() {
    let path = workspace_root().join("dsl/tools/cigen.dag");
    let source = fs::read_to_string(&path).expect("read tools/cigen.dag");

    assert!(
        source.contains("render_github_workflow_yaml"),
        "tools/cigen.dag should call the GitHub leaf serializer in {}",
        path.display()
    );
    assert!(
        source.contains("render_gitlab_pipeline_yaml"),
        "tools/cigen.dag should call the GitLab leaf serializer in {}",
        path.display()
    );
    assert!(
        !source.contains("fn render_github_workflow("),
        "tools/cigen.dag should not own GitHub YAML layout in {}",
        path.display()
    );
    assert!(
        !source.contains("fn render_gitlab_pipeline("),
        "tools/cigen.dag should not own GitLab YAML layout in {}",
        path.display()
    );
    assert!(
        !source.contains("tool_command: String"),
        "CiDiscovery should not regress to raw tool_command strings in {}",
        path.display()
    );
    assert!(
        !source.contains("bootstrap_script: String"),
        "CiDiscovery should not regress to raw bootstrap_script blobs in {}",
        path.display()
    );
}
