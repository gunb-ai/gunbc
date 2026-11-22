// Generated tests -- do not edit.
// Source module: extdeps.posix.signal

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
async fn test_posix___signal_terminate_process() {
        // Signature: Posix__Signal.TerminateProcess(pid: String) -> (bool, String)
        let service = Posix__Signal::new(crate::dry_run::DryRunMode(true));
        let result = service.terminate_process(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Posix__Signal.TerminateProcess should succeed in dry-run mode: {:?}", result.err());
}
}
