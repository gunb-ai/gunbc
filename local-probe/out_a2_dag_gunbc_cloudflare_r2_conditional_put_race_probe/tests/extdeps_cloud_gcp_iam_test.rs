// Generated tests -- do not edit.
// Source module: extdeps.cloud.gcp.iam

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
async fn test_gcp___i_a_m_generate_access_token() {
        // Signature: Gcp__IAM.GenerateAccessToken(access_token: String, target_sa: String, scope: Vec<String>, lifetime: String) -> (String, String)
        let service = Gcp__IAM::new(crate::dry_run::DryRunMode(true));
        let result = service.generate_access_token(String::new(), String::new(), compile_error!("no test default for type: List"), String::new()).await;
        assert!(result.is_ok(), "Gcp__IAM.GenerateAccessToken should succeed in dry-run mode: {:?}", result.err());
}
}
