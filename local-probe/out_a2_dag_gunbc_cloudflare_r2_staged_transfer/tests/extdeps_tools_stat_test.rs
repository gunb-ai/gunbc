// Generated tests -- do not edit.
// Source module: extdeps.tools.stat

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
async fn test_stat___file_file_type() {
        // Signature: Stat__File.FileType(path: String) -> (String, bool, String)
        let service = Stat__File::new(crate::dry_run::DryRunMode(true));
        let result = service.file_type(String::new()).await;
        assert!(result.is_ok(), "Stat__File.FileType should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_stat___file_filesystem_type() {
        // Signature: Stat__File.FilesystemType(path: String) -> (String, bool)
        let service = Stat__File::new(crate::dry_run::DryRunMode(true));
        let result = service.filesystem_type(String::new()).await;
        assert!(result.is_ok(), "Stat__File.FilesystemType should succeed in dry-run mode: {:?}", result.err());
}
}
