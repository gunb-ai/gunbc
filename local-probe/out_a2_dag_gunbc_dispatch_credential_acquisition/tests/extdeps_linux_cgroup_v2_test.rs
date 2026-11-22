// Generated tests -- do not edit.
// Source module: extdeps.linux.cgroup_v2

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
async fn test_linux___cgroup_v2_path_is_directory() {
        // Signature: Linux__CgroupV2.PathIsDirectory(path: String) -> (String, bool)
        let service = Linux__CgroupV2::new(crate::dry_run::DryRunMode(true));
        let result = service.path_is_directory(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Linux__CgroupV2.PathIsDirectory should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_linux___cgroup_v2_read_events() {
        // Signature: Linux__CgroupV2.ReadEvents(events_path: String) -> (String, bool)
        let service = Linux__CgroupV2::new(crate::dry_run::DryRunMode(true));
        let result = service.read_events(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Linux__CgroupV2.ReadEvents should succeed in dry-run mode: {:?}", result.err());
}
}
