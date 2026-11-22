// Generated tests -- do not edit.
// Source module: extdeps.linux.procfs

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
async fn test_linux___procfs_read_stat() {
        // Signature: Linux__Procfs.ReadStat() -> (String, bool)
        let service = Linux__Procfs::new(crate::dry_run::DryRunMode(true));
        let result = service.read_stat().await;
        assert!(result.is_ok(), "Linux__Procfs.ReadStat should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_linux___procfs_read_uptime() {
        // Signature: Linux__Procfs.ReadUptime() -> (String, bool)
        let service = Linux__Procfs::new(crate::dry_run::DryRunMode(true));
        let result = service.read_uptime().await;
        assert!(result.is_ok(), "Linux__Procfs.ReadUptime should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_linux___procfs_read_meminfo() {
        // Signature: Linux__Procfs.ReadMeminfo() -> (String, bool)
        let service = Linux__Procfs::new(crate::dry_run::DryRunMode(true));
        let result = service.read_meminfo().await;
        assert!(result.is_ok(), "Linux__Procfs.ReadMeminfo should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_linux___procfs_read_boot_id() {
        // Signature: Linux__Procfs.ReadBootId(path: String) -> (String, bool)
        let service = Linux__Procfs::new(crate::dry_run::DryRunMode(true));
        let result = service.read_boot_id(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Linux__Procfs.ReadBootId should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_linux___procfs_read_pid_stat() {
        // Signature: Linux__Procfs.ReadPidStat(pid: String) -> (String, bool)
        let service = Linux__Procfs::new(crate::dry_run::DryRunMode(true));
        let result = service.read_pid_stat(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Linux__Procfs.ReadPidStat should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_linux___procfs_read_net_tcp() {
        // Signature: Linux__Procfs.ReadNetTcp() -> (String, bool)
        let service = Linux__Procfs::new(crate::dry_run::DryRunMode(true));
        let result = service.read_net_tcp().await;
        assert!(result.is_ok(), "Linux__Procfs.ReadNetTcp should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_linux___procfs_read_net_tcp6() {
        // Signature: Linux__Procfs.ReadNetTcp6() -> (String, bool)
        let service = Linux__Procfs::new(crate::dry_run::DryRunMode(true));
        let result = service.read_net_tcp6().await;
        assert!(result.is_ok(), "Linux__Procfs.ReadNetTcp6 should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_linux___procfs_read_pid_cgroup() {
        // Signature: Linux__Procfs.ReadPidCgroup(pid: String) -> (String, bool)
        let service = Linux__Procfs::new(crate::dry_run::DryRunMode(true));
        let result = service.read_pid_cgroup(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Linux__Procfs.ReadPidCgroup should succeed in dry-run mode: {:?}", result.err());
}
}
