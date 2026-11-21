// Generated tests -- do not edit.
// Source module: extdeps.systemd.systemd_run

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
async fn test_systemd___systemd_run_run_transient() {
        // Signature: Systemd__SystemdRun.RunTransient(command_argv: Vec<String>) -> (bool, String, String)
        let service = Systemd__SystemdRun::new(crate::dry_run::DryRunMode(true));
        let result = service.run_transient(compile_error!("no test default for type: List")).await;
        assert!(result.is_ok(), "Systemd__SystemdRun.RunTransient should succeed in dry-run mode: {:?}", result.err());
}
}
