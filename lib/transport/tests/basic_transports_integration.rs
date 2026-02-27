use gunbc_ir::resource::ResourceIo;
use gunbc_ir::transport::{
    FileRequest, HttpRequest, RestRequest, ShellRequest, TcpRequest, TransportRequest,
    TransportResponse,
};
use gunbc_lib_transport::backend::execute_transport_with_backend;
use gunbc_lib_transport::test_backend::VirtualTransportBackend;
use gunbc_lib_transport::{TransportBackendGuard, TransportIo};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::UNIX_EPOCH;

#[test]
fn transport_io_roundtrip_with_virtual_backend() {
    let backend = Arc::new(VirtualTransportBackend::new());
    backend.create_dir_all("repo/src");
    let _guard = TransportBackendGuard::install(backend);
    let io = TransportIo::new();
    let file_path = Path::new("repo/src/main.rs");

    io.write_file(file_path, b"fn main() {}\n")
        .expect("write file via transport");
    assert!(io.file_exists(file_path).expect("exists check"));
    assert!(!io
        .file_exists(Path::new("repo/src/missing.rs"))
        .expect("missing exists check"));

    let content = io.read_file(file_path).expect("read file via transport");
    assert_eq!(content, b"fn main() {}\n");

    let globbed = io.glob_paths("repo/**/*.rs").expect("glob via transport");
    assert_eq!(globbed, vec![PathBuf::from("repo/src/main.rs")]);

    let mtime = io.file_mtime(file_path).expect("metadata via transport");
    assert!(mtime > UNIX_EPOCH);

    let find_args = vec![
        "repo".to_string(),
        "-maxdepth".to_string(),
        "2".to_string(),
        "-mindepth".to_string(),
        "1".to_string(),
        "-type".to_string(),
        "d".to_string(),
    ];
    let find_stdout = io
        .command_output("find", &find_args)
        .expect("find command via transport");
    let find_stdout = String::from_utf8(find_stdout).expect("find output utf8");
    assert!(find_stdout.contains("repo/src\n"));

    let unsupported = io
        .command_output("echo", &["hello".to_string()])
        .expect_err("virtual backend should reject unsupported shell commands");
    assert!(
        unsupported
            .to_string()
            .contains("unsupported shell command"),
        "unexpected error: {}",
        unsupported
    );
}

#[test]
fn virtual_backend_dispatch_rejects_network_transports() {
    let backend = Arc::new(VirtualTransportBackend::new());
    backend.create_dir_all("workspace");
    let _guard = TransportBackendGuard::install(backend);

    let file_response =
        execute_transport_with_backend(&TransportRequest::File(FileRequest::exists("workspace")))
            .expect("file transport should execute");
    assert!(
        matches!(file_response, TransportResponse::File(ref resp) if resp.success),
        "unexpected file response: {:?}",
        file_response
    );

    let shell_response = execute_transport_with_backend(&TransportRequest::Shell(
        ShellRequest::new("find")
            .arg("workspace")
            .arg("-maxdepth")
            .arg("1")
            .arg("-mindepth")
            .arg("1")
            .arg("-type")
            .arg("d"),
    ))
    .expect("shell transport should execute");
    assert!(
        matches!(shell_response, TransportResponse::Shell(ref resp) if resp.success()),
        "unexpected shell response: {:?}",
        shell_response
    );

    // Without stubs registered, REST/HTTP/TCP still error (no matching stub).
    let rest_err = execute_transport_with_backend(&TransportRequest::Rest(RestRequest::get(
        "https://example.invalid",
    )))
    .expect_err("virtual backend should reject unmatched REST");
    assert!(rest_err.to_string().contains("no HTTP stub matches"));

    let http_err = execute_transport_with_backend(&TransportRequest::Http(HttpRequest::get(
        "https://example.invalid",
    )))
    .expect_err("virtual backend should reject unmatched HTTP");
    assert!(http_err.to_string().contains("no HTTP stub matches"));

    let tcp_err =
        execute_transport_with_backend(&TransportRequest::Tcp(TcpRequest::new("localhost", 7)))
            .expect_err("virtual backend should reject unmatched TCP");
    assert!(tcp_err.to_string().contains("no TCP loopback on port"));
}

// ── Shell cassette tests (RT10) ──────────────────────────────────────

#[test]
fn shell_cassette_matches_exact_command_args() {
    use gunbc_lib_transport::test_backend::ShellCassette;

    let backend = Arc::new(VirtualTransportBackend::new());
    backend.add_shell_cassette(ShellCassette {
        command: "cargo".to_string(),
        args: vec!["build".to_string(), "--release".to_string()],
        stdout: "Compiling...\n".to_string(),
        stderr: String::new(),
        exit_code: 0,
    });
    let _guard = TransportBackendGuard::install(backend);

    let resp = execute_transport_with_backend(&TransportRequest::Shell(
        ShellRequest::new("cargo").arg("build").arg("--release"),
    ))
    .expect("cassette should match");
    match resp {
        TransportResponse::Shell(shell) => {
            assert_eq!(shell.exit_code, 0);
            assert_eq!(shell.stdout, "Compiling...\n");
        }
        other => panic!("expected Shell response, got {other:?}"),
    }
}

#[test]
fn shell_cassette_wildcard_args_matches_any() {
    use gunbc_lib_transport::test_backend::ShellCassette;

    let backend = Arc::new(VirtualTransportBackend::new());
    backend.add_shell_cassette(ShellCassette {
        command: "git".to_string(),
        args: vec![], // wildcard: matches any args
        stdout: "abc123\n".to_string(),
        stderr: String::new(),
        exit_code: 0,
    });
    let _guard = TransportBackendGuard::install(backend);

    let resp = execute_transport_with_backend(&TransportRequest::Shell(
        ShellRequest::new("git").arg("rev-parse").arg("HEAD"),
    ))
    .expect("wildcard cassette should match");
    match resp {
        TransportResponse::Shell(shell) => {
            assert_eq!(shell.stdout, "abc123\n");
        }
        other => panic!("expected Shell response, got {other:?}"),
    }
}

#[test]
fn shell_cassette_nonzero_exit_code() {
    use gunbc_lib_transport::test_backend::ShellCassette;

    let backend = Arc::new(VirtualTransportBackend::new());
    backend.add_shell_cassette(ShellCassette {
        command: "cargo".to_string(),
        args: vec!["test".to_string()],
        stdout: String::new(),
        stderr: "test failed\n".to_string(),
        exit_code: 101,
    });
    let _guard = TransportBackendGuard::install(backend);

    let resp = execute_transport_with_backend(&TransportRequest::Shell(
        ShellRequest::new("cargo").arg("test"),
    ))
    .expect("cassette should match even with non-zero exit");
    match resp {
        TransportResponse::Shell(shell) => {
            assert_eq!(shell.exit_code, 101);
            assert_eq!(shell.stderr, "test failed\n");
        }
        other => panic!("expected Shell response, got {other:?}"),
    }
}

// ── HTTP stub tests (RT11) ───────────────────────────────────────────

#[test]
fn http_stub_matches_rest_request() {
    use gunbc_lib_transport::test_backend::HttpStub;
    use std::collections::HashMap;

    let backend = Arc::new(VirtualTransportBackend::new());
    backend.add_http_stub(HttpStub {
        method: Some(gunbc_ir::transport::http::HttpMethod::Post),
        path_pattern: "/gists".to_string(),
        exact_path: false,
        status: 201,
        response_body: r#"{"id":"abc","html_url":"https://gist.github.com/abc"}"#.to_string(),
        response_headers: HashMap::new(),
    });
    let _guard = TransportBackendGuard::install(backend);

    let resp = execute_transport_with_backend(&TransportRequest::Rest(
        RestRequest::post("https://api.github.com/gists"),
    ))
    .expect("HTTP stub should match REST request");
    match resp {
        TransportResponse::Rest(rest) => {
            assert_eq!(rest.status, 201);
            assert_eq!(rest.body["id"], "abc");
        }
        other => panic!("expected Rest response, got {other:?}"),
    }
}

#[test]
fn http_stub_matches_raw_http_request() {
    use gunbc_lib_transport::test_backend::HttpStub;
    use std::collections::HashMap;

    let backend = Arc::new(VirtualTransportBackend::new());
    backend.add_http_stub(HttpStub {
        method: None, // any method
        path_pattern: "/health".to_string(),
        exact_path: true,
        status: 200,
        response_body: "OK".to_string(),
        response_headers: HashMap::new(),
    });
    let _guard = TransportBackendGuard::install(backend);

    let resp = execute_transport_with_backend(&TransportRequest::Http(HttpRequest::get(
        "https://example.com/health",
    )))
    .expect("HTTP stub should match raw HTTP request");
    match resp {
        TransportResponse::Http(http) => {
            assert_eq!(http.status, 200);
            assert_eq!(http.body, "OK");
        }
        other => panic!("expected Http response, got {other:?}"),
    }
}

// ── TCP loopback tests (RT12) ────────────────────────────────────────

#[test]
fn tcp_loopback_returns_canned_data() {
    use gunbc_lib_transport::test_backend::TcpLoopback;

    let backend = Arc::new(VirtualTransportBackend::new());
    backend.add_tcp_loopback(TcpLoopback {
        port: 8080,
        response_data: "PONG\n".to_string(),
    });
    let _guard = TransportBackendGuard::install(backend);

    let resp = execute_transport_with_backend(&TransportRequest::Tcp(
        TcpRequest::new("localhost", 8080).data("PING\n"),
    ))
    .expect("TCP loopback should respond");
    match resp {
        TransportResponse::Tcp(tcp) => {
            assert!(tcp.connected);
            assert_eq!(tcp.data, Some("PONG\n".to_string()));
            assert_eq!(tcp.bytes_sent, 5);
            assert_eq!(tcp.bytes_received, 5);
        }
        other => panic!("expected Tcp response, got {other:?}"),
    }
}
