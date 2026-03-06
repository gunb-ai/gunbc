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
