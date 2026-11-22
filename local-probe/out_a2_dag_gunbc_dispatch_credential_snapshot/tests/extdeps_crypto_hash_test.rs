// Generated tests -- do not edit.
// Source module: extdeps.crypto.hash

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
async fn test_crypto___sha256_sum_file() {
        // Signature: Crypto__Sha256Sum.File(path: String) -> (String, bool, String)
        let service = Crypto__Sha256Sum::new(crate::dry_run::DryRunMode(true));
        let result = service.file(String::new()).await;
        assert!(result.is_ok(), "Crypto__Sha256Sum.File should succeed in dry-run mode: {:?}", result.err());
}
}
