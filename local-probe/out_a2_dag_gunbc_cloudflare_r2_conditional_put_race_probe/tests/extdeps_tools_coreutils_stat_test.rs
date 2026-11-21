// Generated tests -- do not edit.
// Source module: extdeps.tools.coreutils_stat

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
async fn test_coreutils___stat_path_size() {
        // Signature: Coreutils__Stat.PathSize(path: String) -> (String, bool)
        let service = Coreutils__Stat::new(crate::dry_run::DryRunMode(true));
        let result = service.path_size(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Coreutils__Stat.PathSize should succeed in dry-run mode: {:?}", result.err());
}
}
