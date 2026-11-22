// Generated tests -- do not edit.
// Source module: extdeps.git.inspect

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
async fn test_git___inspect_toplevel() {
        // Signature: Git__Inspect.Toplevel() -> (String)
        let service = Git__Inspect::new(crate::dry_run::DryRunMode(true));
        let result = service.toplevel().await;
        assert!(result.is_ok(), "Git__Inspect.Toplevel should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_git___inspect_working_tree_status() {
        // Signature: Git__Inspect.WorkingTreeStatus(repository_path: String) -> (String, bool, String)
        let service = Git__Inspect::new(crate::dry_run::DryRunMode(true));
        let result = service.working_tree_status(String::new()).await;
        assert!(result.is_ok(), "Git__Inspect.WorkingTreeStatus should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_git___inspect_head_commit_in() {
        // Signature: Git__Inspect.HeadCommitIn(repository_path: String) -> (String, bool, String)
        let service = Git__Inspect::new(crate::dry_run::DryRunMode(true));
        let result = service.head_commit_in(String::new()).await;
        assert!(result.is_ok(), "Git__Inspect.HeadCommitIn should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_git___inspect_read_tree_into_index() {
        // Signature: Git__Inspect.ReadTreeIntoIndex(repository_path: String, revision_hex: String, index_path: String) -> (bool, String)
        let service = Git__Inspect::new(crate::dry_run::DryRunMode(true));
        let result = service.read_tree_into_index(String::new(), String::new(), String::new()).await;
        assert!(result.is_ok(), "Git__Inspect.ReadTreeIntoIndex should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_git___inspect_status_against_index() {
        // Signature: Git__Inspect.StatusAgainstIndex(repository_path: String, index_path: String) -> (String, bool, String)
        let service = Git__Inspect::new(crate::dry_run::DryRunMode(true));
        let result = service.status_against_index(String::new(), String::new()).await;
        assert!(result.is_ok(), "Git__Inspect.StatusAgainstIndex should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_git___inspect_ignored_against_index() {
        // Signature: Git__Inspect.IgnoredAgainstIndex(repository_path: String, index_path: String) -> (String, bool, String)
        let service = Git__Inspect::new(crate::dry_run::DryRunMode(true));
        let result = service.ignored_against_index(String::new(), String::new()).await;
        assert!(result.is_ok(), "Git__Inspect.IgnoredAgainstIndex should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_git___inspect_ignored_files() {
        // Signature: Git__Inspect.IgnoredFiles() -> (String, bool, String)
        let service = Git__Inspect::new(crate::dry_run::DryRunMode(true));
        let result = service.ignored_files().await;
        assert!(result.is_ok(), "Git__Inspect.IgnoredFiles should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_git___inspect_index_flags_in() {
        // Signature: Git__Inspect.IndexFlagsIn(repository_path: String, path: String) -> (String, bool, String)
        let service = Git__Inspect::new(crate::dry_run::DryRunMode(true));
        let result = service.index_flags_in(String::new(), String::new()).await;
        assert!(result.is_ok(), "Git__Inspect.IndexFlagsIn should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_git___inspect_resolve_ref_commit() {
        // Signature: Git__Inspect.ResolveRefCommit(ref: String) -> (String, bool, String)
        let service = Git__Inspect::new(crate::dry_run::DryRunMode(true));
        let result = service.resolve_ref_commit(compile_error!("no test default for type: GitRef")).await;
        assert!(result.is_ok(), "Git__Inspect.ResolveRefCommit should succeed in dry-run mode: {:?}", result.err());
}
}
