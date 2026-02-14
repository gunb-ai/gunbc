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

    let rest_err = execute_transport_with_backend(&TransportRequest::Rest(RestRequest::get(
        "https://example.invalid",
    )))
    .expect_err("virtual backend should reject REST");
    assert!(rest_err.to_string().contains("REST transport unsupported"));

    let http_err = execute_transport_with_backend(&TransportRequest::Http(HttpRequest::get(
        "https://example.invalid",
    )))
    .expect_err("virtual backend should reject HTTP");
    assert!(http_err.to_string().contains("HTTP transport unsupported"));

    let tcp_err =
        execute_transport_with_backend(&TransportRequest::Tcp(TcpRequest::new("localhost", 7)))
            .expect_err("virtual backend should reject TCP");
    assert!(tcp_err.to_string().contains("TCP transport unsupported"));
}
