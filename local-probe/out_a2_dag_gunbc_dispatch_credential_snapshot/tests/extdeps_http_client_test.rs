// Generated tests -- do not edit.
// Source module: extdeps.http.client

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
async fn test_http___client_get_localhost_bounded() {
        // Signature: Http__Client.GetLocalhostBounded(url: String) -> (String, bool)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.get_localhost_bounded(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Http__Client.GetLocalhostBounded should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_http___client_get_bounded() {
        // Signature: Http__Client.GetBounded(url: String) -> (String, bool)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.get_bounded(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Http__Client.GetBounded should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_http___client_get_bounded_reporting_peer() {
        // Signature: Http__Client.GetBoundedReportingPeer(url: String) -> (String, bool)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.get_bounded_reporting_peer(compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Http__Client.GetBoundedReportingPeer should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_http___client_get_within() {
        // Signature: Http__Client.GetWithin(url: String, max_seconds: String) -> (String, bool)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.get_within(compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Http__Client.GetWithin should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_http___client_get_within_with_status() {
        // Signature: Http__Client.GetWithinWithStatus(url: String, connect_seconds: String, max_seconds: String) -> (String, bool)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.get_within_with_status(compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Http__Client.GetWithinWithStatus should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_http___client_get_within_unix_socket() {
        // Signature: Http__Client.GetWithinUnixSocket(socket: String, url: String, max_seconds: String) -> (String, bool)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.get_within_unix_socket(compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Http__Client.GetWithinUnixSocket should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_http___client_status_within() {
        // Signature: Http__Client.StatusWithin(url: String, max_seconds: String) -> (String, bool, i64)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.status_within(compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Http__Client.StatusWithin should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_http___client_get_query_stdin_within() {
        // Signature: Http__Client.GetQueryStdinWithin(url: String, query: String, connect_seconds: String, max_seconds: String) -> (bool, String)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.get_query_stdin_within(compile_error!("no test default for type: NonEmptyStr"), String::new(), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Http__Client.GetQueryStdinWithin should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_http___client_post_stdin_within() {
        // Signature: Http__Client.PostStdinWithin(url: String, request_body: String, connect_seconds: String, max_seconds: String) -> (i64, bool, String, String)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.post_stdin_within(compile_error!("no test default for type: NonEmptyStr"), String::new(), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Http__Client.PostStdinWithin should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_http___client_post_stdin_within_retrying_refused() {
        // Signature: Http__Client.PostStdinWithinRetryingRefused(url: String, request_body: String, connect_seconds: String, max_seconds: String, startup_seconds: String) -> (i64, bool, String, String)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.post_stdin_within_retrying_refused(compile_error!("no test default for type: NonEmptyStr"), String::new(), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Http__Client.PostStdinWithinRetryingRefused should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_http___client_post_stdin_within_unix_socket() {
        // Signature: Http__Client.PostStdinWithinUnixSocket(socket: String, url: String, request_body: String, connect_seconds: String, max_seconds: String) -> (i64, bool, String, String)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.post_stdin_within_unix_socket(compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), String::new(), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Http__Client.PostStdinWithinUnixSocket should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_http___client_post_json_from_file_with_header_file() {
        // Signature: Http__Client.PostJsonFromFileWithHeaderFile(url: String, request_body_file: String, header_file: String, response_headers_file: String, max_seconds: String) -> (String, bool)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.post_json_from_file_with_header_file(compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Http__Client.PostJsonFromFileWithHeaderFile should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_http___client_post_json_from_file_with_header_file_pinned() {
        // Signature: Http__Client.PostJsonFromFileWithHeaderFilePinned(url: String, request_body_file: String, header_file: String, response_headers_file: String, ca_file: String, pinned_pubkey_file: String, max_seconds: String) -> (String, bool, i64)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.post_json_from_file_with_header_file_pinned(compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Http__Client.PostJsonFromFileWithHeaderFilePinned should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_http___client_get_bounded_with_header_file_pinned() {
        // Signature: Http__Client.GetBoundedWithHeaderFilePinned(url: String, header_file: String, response_headers_file: String, ca_file: String, pinned_pubkey_file: String, max_seconds: String) -> (String, bool, i64)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.get_bounded_with_header_file_pinned(compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Http__Client.GetBoundedWithHeaderFilePinned should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_http___client_post_form_from_file() {
        // Signature: Http__Client.PostFormFromFile(url: String, request_body_file: String, max_seconds: String) -> (String, bool)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.post_form_from_file(compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Http__Client.PostFormFromFile should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_http___client_get_follow_redirects_bounded_with_header_file() {
        // Signature: Http__Client.GetFollowRedirectsBoundedWithHeaderFile(url: String, max_seconds: String, accept: String, header_file: String) -> (String, bool)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.get_follow_redirects_bounded_with_header_file(compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: NonEmptyStr")).await;
        assert!(result.is_ok(), "Http__Client.GetFollowRedirectsBoundedWithHeaderFile should succeed in dry-run mode: {:?}", result.err());
}

#[tokio::test]
async fn test_http___client_probe() {
        // Signature: Http__Client.Probe(url: String, connect_time: Second, max_time: Second) -> (String, bool, i64)
        let service = Http__Client::new(crate::dry_run::DryRunMode(true));
        let result = service.probe(compile_error!("no test default for type: NonEmptyStr"), compile_error!("no test default for type: Second"), compile_error!("no test default for type: Second")).await;
        assert!(result.is_ok(), "Http__Client.Probe should succeed in dry-run mode: {:?}", result.err());
}
}
