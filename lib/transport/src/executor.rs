//! Transport request executors.

use gunbc_ir::transport::{
    FileOp, FileRequest, FileResponse, HttpRequest, HttpResponse, RestRequest, RestResponse,
    ShellRequest, ShellResponse, TcpRequest, TcpResponse, TransportRequest, TransportResponse,
};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Transport execution error.
#[derive(Debug)]
pub struct TransportError {
    pub message: String,
}

impl TransportError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TransportError {}

/// Execute a transport request.
pub fn execute_transport(request: &TransportRequest) -> Result<TransportResponse, TransportError> {
    match request {
        TransportRequest::Rest(r) => execute_rest(r).map(TransportResponse::Rest),
        TransportRequest::Http(r) => execute_http(r).map(TransportResponse::Http),
        TransportRequest::File(r) => execute_file(r).map(TransportResponse::File),
        TransportRequest::Tcp(r) => execute_tcp(r).map(TransportResponse::Tcp),
        TransportRequest::Shell(r) => execute_shell(r).map(TransportResponse::Shell),
    }
}

/// Execute a REST request.
///
/// Uses the HTTP executor and parses JSON responses.
fn execute_rest(request: &RestRequest) -> Result<RestResponse, TransportError> {
    // Apply credential (if any) to headers before conversion.
    let mut request = request.clone();
    if let Some(cred) = request.auth.take() {
        cred.apply(&mut request);
    }

    // For now, convert to HTTP and use that
    let url = append_query(&request.url, &request.query);
    let mut http_req = HttpRequest::post(url);
    http_req.method = request.method;

    // Add headers
    for (k, v) in &request.headers {
        http_req.headers.insert(k.clone(), v.clone());
    }

    // Add JSON body
    if let Some(ref body) = request.body {
        http_req.body = Some(
            serde_json::to_string(body)
                .map_err(|e| TransportError::new(format!("failed to serialize body: {}", e)))?,
        );
        http_req
            .headers
            .insert("Content-Type".to_string(), "application/json".to_string());
    }

    http_req.timeout_ms = request.timeout_ms;

    let http_resp = execute_http(&http_req)?;

    // Parse JSON response
    let body: serde_json::Value = serde_json::from_str(&http_resp.body)
        .unwrap_or_else(|_| serde_json::json!({ "raw": http_resp.body }));

    Ok(RestResponse {
        status: http_resp.status,
        headers: http_resp.headers,
        body,
    })
}

/// Execute a raw HTTP request.
///
/// Uses ureq for synchronous HTTP with TLS support.
fn execute_http(request: &HttpRequest) -> Result<HttpResponse, TransportError> {
    let mut req = ureq::request(request.method.as_str(), &request.url);

    if let Some(timeout) = request.timeout_ms {
        req = req.timeout(Duration::from_millis(timeout));
    }

    for (key, value) in &request.headers {
        req = req.set(key, value);
    }

    let response = match request.body.as_ref() {
        Some(body) => match req.send_string(body) {
            Ok(resp) => resp,
            Err(ureq::Error::Status(_, resp)) => resp,
            Err(e) => return Err(TransportError::new(format!("http request failed: {}", e))),
        },
        None => match req.call() {
            Ok(resp) => resp,
            Err(ureq::Error::Status(_, resp)) => resp,
            Err(e) => return Err(TransportError::new(format!("http request failed: {}", e))),
        },
    };

    let status = response.status() as u16;
    let mut headers = HashMap::new();
    for name in response.headers_names() {
        if let Some(value) = response.header(&name) {
            headers.insert(name.to_string(), value.to_string());
        }
    }
    let body = response.into_string().unwrap_or_default();

    Ok(HttpResponse {
        status,
        headers,
        body,
    })
}

fn append_query(url: &str, query: &HashMap<String, String>) -> String {
    if query.is_empty() {
        return url.to_string();
    }
    let mut out = String::from(url);
    out.push(if url.contains('?') { '&' } else { '?' });

    let mut first = true;
    for (key, value) in query {
        if !first {
            out.push('&');
        }
        first = false;
        out.push_str(&url_encode(key));
        out.push('=');
        out.push_str(&url_encode(value));
    }
    out
}

fn url_encode(input: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(bytes.len());
    for &b in bytes {
        if is_unreserved_url_byte(b) {
            out.push(b as char);
        } else {
            out.push('%');
            out.push(HEX[(b >> 4) as usize] as char);
            out.push(HEX[(b & 0x0f) as usize] as char);
        }
    }
    out
}

fn is_unreserved_url_byte(b: u8) -> bool {
    matches!(
        b,
        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~'
    )
}

/// Execute a file operation.
fn execute_file(request: &FileRequest) -> Result<FileResponse, TransportError> {
    match request.operation {
        FileOp::Read => match fs::read_to_string(&request.path) {
            Ok(content) => Ok(FileResponse::read_ok(&request.path, content)),
            Err(e) => Ok(FileResponse::error(
                &request.path,
                FileOp::Read,
                e.to_string(),
            )),
        },
        FileOp::Write => {
            if request.create_parents {
                if let Some(parent) = std::path::Path::new(&request.path).parent() {
                    fs::create_dir_all(parent).ok();
                }
            }

            let content = request.content.as_deref().unwrap_or("");
            match fs::write(&request.path, content) {
                Ok(()) => Ok(FileResponse::written(&request.path)),
                Err(e) => Ok(FileResponse::error(
                    &request.path,
                    FileOp::Write,
                    e.to_string(),
                )),
            }
        }
        FileOp::Append => {
            use std::fs::OpenOptions;

            if request.create_parents {
                if let Some(parent) = std::path::Path::new(&request.path).parent() {
                    fs::create_dir_all(parent).ok();
                }
            }

            let content = request.content.as_deref().unwrap_or("");
            match OpenOptions::new()
                .append(true)
                .create(true)
                .open(&request.path)
            {
                Ok(mut file) => match file.write_all(content.as_bytes()) {
                    Ok(()) => Ok(FileResponse {
                        path: request.path.clone(),
                        operation: FileOp::Append,
                        success: true,
                        content: None,
                        exists: None,
                        error: None,
                    }),
                    Err(e) => Ok(FileResponse::error(
                        &request.path,
                        FileOp::Append,
                        e.to_string(),
                    )),
                },
                Err(e) => Ok(FileResponse::error(
                    &request.path,
                    FileOp::Append,
                    e.to_string(),
                )),
            }
        }
        FileOp::Delete => match fs::remove_file(&request.path) {
            Ok(()) => Ok(FileResponse {
                path: request.path.clone(),
                operation: FileOp::Delete,
                success: true,
                content: None,
                exists: None,
                error: None,
            }),
            Err(e) => Ok(FileResponse::error(
                &request.path,
                FileOp::Delete,
                e.to_string(),
            )),
        },
        FileOp::Exists => {
            let exists = std::path::Path::new(&request.path).exists();
            Ok(FileResponse::exists_result(&request.path, exists))
        }
        FileOp::CreateDir => match fs::create_dir_all(&request.path) {
            Ok(()) => Ok(FileResponse {
                path: request.path.clone(),
                operation: FileOp::CreateDir,
                success: true,
                content: None,
                exists: None,
                error: None,
            }),
            Err(e) => Ok(FileResponse::error(
                &request.path,
                FileOp::CreateDir,
                e.to_string(),
            )),
        },
        FileOp::Glob => {
            let entries = match glob::glob(&request.path) {
                Ok(e) => e,
                Err(e) => {
                    return Ok(FileResponse::error(
                        &request.path,
                        FileOp::Glob,
                        e.to_string(),
                    ))
                }
            };

            let mut paths: Vec<String> = Vec::new();
            for entry in entries {
                match entry {
                    Ok(path) => {
                        if path.is_file() {
                            paths.push(path.to_string_lossy().to_string());
                        }
                    }
                    Err(e) => {
                        return Ok(FileResponse::error(
                            &request.path,
                            FileOp::Glob,
                            e.to_string(),
                        ));
                    }
                }
            }

            paths.sort();
            Ok(FileResponse::glob_result(&request.path, paths))
        }
        FileOp::Metadata => match fs::metadata(&request.path) {
            Ok(meta) => match meta.modified() {
                Ok(mtime) => {
                    use std::time::{Duration, UNIX_EPOCH};
                    let millis = mtime
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or(Duration::from_secs(0))
                        .as_millis() as i64;
                    Ok(FileResponse::metadata_result(&request.path, millis))
                }
                Err(e) => Ok(FileResponse::error(
                    &request.path,
                    FileOp::Metadata,
                    e.to_string(),
                )),
            },
            Err(e) => Ok(FileResponse::error(
                &request.path,
                FileOp::Metadata,
                e.to_string(),
            )),
        },
    }
}

/// Execute a TCP request.
fn execute_tcp(request: &TcpRequest) -> Result<TcpResponse, TransportError> {
    let addr = format!("{}:{}", request.host, request.port);

    let mut stream = TcpStream::connect(&addr)
        .map_err(|e| TransportError::new(format!("connection failed: {}", e)))?;

    if let Some(timeout) = request.connect_timeout_ms {
        stream
            .set_read_timeout(Some(Duration::from_millis(timeout)))
            .ok();
    }
    if let Some(timeout) = request.read_timeout_ms {
        stream
            .set_write_timeout(Some(Duration::from_millis(timeout)))
            .ok();
    }

    let mut bytes_sent = 0;
    if let Some(ref data) = request.data {
        stream
            .write_all(data.as_bytes())
            .map_err(|e| TransportError::new(format!("write failed: {}", e)))?;
        bytes_sent = data.len();
    }

    let mut response = String::new();
    stream.read_to_string(&mut response).ok(); // May timeout, that's ok
    let bytes_received = response.len();

    Ok(TcpResponse::ok(
        if response.is_empty() {
            None
        } else {
            Some(response)
        },
        bytes_sent,
        bytes_received,
    ))
}

/// Execute a shell command.
///
/// This is the I/O boundary for shell requests (TransportRequest::Shell).
/// CLI tool execution uses the transport-layer helpers in `cli.rs`.
#[allow(clippy::disallowed_methods)]
fn execute_shell(request: &ShellRequest) -> Result<ShellResponse, TransportError> {
    let mut cmd = Command::new(&request.command);
    cmd.args(&request.args);

    if let Some(ref cwd) = request.cwd {
        cmd.current_dir(cwd);
    }

    for (key, value) in &request.env {
        cmd.env(key, value);
    }

    // Handle stdin
    if request.stdin.is_some() {
        cmd.stdin(Stdio::piped());
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| TransportError::new(format!("failed to spawn: {}", e)))?;

    // Write stdin if provided
    if let Some(ref stdin_data) = request.stdin {
        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(stdin_data.as_bytes()).ok();
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|e| TransportError::new(format!("failed to wait: {}", e)))?;

    Ok(ShellResponse {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use gunbc_test::{guard_test, FermiCost, TestClass};
    use std::env::temp_dir;
    use std::path::{Path, PathBuf};

    /// Generate a unique temp file path for testing.
    fn temp_path(name: &str) -> PathBuf {
        let mut path = temp_dir();
        path.push(format!(
            "gunbc_transport_test_{}_{}",
            std::process::id(),
            name
        ));
        path
    }

    fn guard_fs(name: &str) -> bool {
        guard_test(name, TestClass::Integration, FermiCost::S, &["fs"], &[])
    }

    fn guard_shell(name: &str) -> bool {
        guard_test(name, TestClass::Integration, FermiCost::M, &["shell"], &[])
    }

    fn guard_git(name: &str) -> bool {
        guard_test(
            name,
            TestClass::Integration,
            FermiCost::M,
            &["git", "shell"],
            &[],
        )
    }

    // ========================================================================
    // FileOp::Read tests
    // ========================================================================

    #[test]
    fn test_file_read_success() {
        if !guard_fs(stringify!(test_file_read_success)) {
            return;
        }

        let request = FileRequest::read("Cargo.toml");
        let response = execute_file(&request).unwrap();

        assert!(response.success, "read should succeed");
        assert_eq!(response.operation, FileOp::Read);
        assert!(response.content.is_some(), "content should be present");
        assert!(
            response.content.unwrap().contains("[package]"),
            "should read Cargo.toml"
        );
        assert!(response.error.is_none());
    }

    #[test]
    fn test_file_read_not_found() {
        if !guard_fs(stringify!(test_file_read_not_found)) {
            return;
        }

        let request = FileRequest::read("nonexistent_file_xyz_12345.txt");
        let response = execute_file(&request).unwrap();

        assert!(!response.success, "read of missing file should fail");
        assert_eq!(response.operation, FileOp::Read);
        assert!(response.error.is_some(), "should have error message");
        assert!(response.content.is_none());
    }

    // ========================================================================
    // FileOp::Write tests
    // ========================================================================

    #[test]
    fn test_file_write_success() {
        if !guard_fs(stringify!(test_file_write_success)) {
            return;
        }

        let path = temp_path("write_test.txt");
        let content = "hello, integration test!";

        let request = FileRequest::write(path.to_str().unwrap(), content);
        let response = execute_file(&request).unwrap();

        assert!(response.success, "write should succeed");
        assert_eq!(response.operation, FileOp::Write);
        assert!(response.error.is_none());

        // Verify content was written
        let read_back = fs::read_to_string(&path).unwrap();
        assert_eq!(read_back, content);

        // Cleanup
        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_file_write_overwrites() {
        if !guard_fs(stringify!(test_file_write_overwrites)) {
            return;
        }

        let path = temp_path("write_overwrite.txt");

        // Write initial content
        fs::write(&path, "initial").unwrap();

        // Overwrite
        let request = FileRequest::write(path.to_str().unwrap(), "overwritten");
        let response = execute_file(&request).unwrap();

        assert!(response.success);
        assert_eq!(fs::read_to_string(&path).unwrap(), "overwritten");

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_file_write_creates_parents() {
        if !guard_fs(stringify!(test_file_write_creates_parents)) {
            return;
        }

        let mut path = temp_path("nested");
        path.push("subdir");
        path.push("file.txt");

        let mut request = FileRequest::write(path.to_str().unwrap(), "nested content");
        request.create_parents = true;

        let response = execute_file(&request).unwrap();

        assert!(response.success, "write with create_parents should succeed");
        assert!(path.exists(), "nested file should exist");
        assert_eq!(fs::read_to_string(&path).unwrap(), "nested content");

        // Cleanup - remove the whole nested directory
        let mut parent = path.clone();
        parent.pop();
        parent.pop();
        fs::remove_dir_all(&parent).ok();
    }

    // ========================================================================
    // FileOp::Append tests
    // ========================================================================

    #[test]
    fn test_file_append_to_existing() {
        if !guard_fs(stringify!(test_file_append_to_existing)) {
            return;
        }

        let path = temp_path("append_test.txt");

        // Create initial file
        fs::write(&path, "line1\n").unwrap();

        // Append
        let request = FileRequest::append(path.to_str().unwrap(), "line2\n");
        let response = execute_file(&request).unwrap();

        assert!(response.success, "append should succeed");
        assert_eq!(response.operation, FileOp::Append);
        assert_eq!(fs::read_to_string(&path).unwrap(), "line1\nline2\n");

        fs::remove_file(&path).ok();
    }

    #[test]
    fn test_file_append_creates_file() {
        if !guard_fs(stringify!(test_file_append_creates_file)) {
            return;
        }

        let path = temp_path("append_new.txt");

        // Ensure file doesn't exist
        fs::remove_file(&path).ok();

        let request = FileRequest::append(path.to_str().unwrap(), "new content");
        let response = execute_file(&request).unwrap();

        assert!(response.success, "append to new file should succeed");
        assert_eq!(fs::read_to_string(&path).unwrap(), "new content");

        fs::remove_file(&path).ok();
    }

    // ========================================================================
    // FileOp::Delete tests
    // ========================================================================

    #[test]
    fn test_file_delete_success() {
        if !guard_fs(stringify!(test_file_delete_success)) {
            return;
        }

        let path = temp_path("delete_test.txt");

        // Create file to delete
        fs::write(&path, "to be deleted").unwrap();
        assert!(path.exists());

        let request = FileRequest::delete(path.to_str().unwrap());
        let response = execute_file(&request).unwrap();

        assert!(response.success, "delete should succeed");
        assert_eq!(response.operation, FileOp::Delete);
        assert!(!path.exists(), "file should be deleted");
    }

    #[test]
    fn test_file_delete_not_found() {
        if !guard_fs(stringify!(test_file_delete_not_found)) {
            return;
        }

        let path = temp_path("delete_nonexistent.txt");
        fs::remove_file(&path).ok(); // Ensure doesn't exist

        let request = FileRequest::delete(path.to_str().unwrap());
        let response = execute_file(&request).unwrap();

        assert!(!response.success, "delete of missing file should fail");
        assert!(response.error.is_some());
    }

    // ========================================================================
    // FileOp::Exists tests
    // ========================================================================

    #[test]
    fn test_file_exists_true() {
        if !guard_fs(stringify!(test_file_exists_true)) {
            return;
        }

        let request = FileRequest::exists("Cargo.toml");
        let response = execute_file(&request).unwrap();

        assert!(response.success);
        assert_eq!(response.operation, FileOp::Exists);
        assert_eq!(response.exists, Some(true));
    }

    #[test]
    fn test_file_exists_false() {
        if !guard_fs(stringify!(test_file_exists_false)) {
            return;
        }

        let request = FileRequest::exists("nonexistent_file_12345.txt");
        let response = execute_file(&request).unwrap();

        assert!(response.success, "exists check always succeeds");
        assert_eq!(response.exists, Some(false));
    }

    #[test]
    fn test_file_exists_directory() {
        if !guard_fs(stringify!(test_file_exists_directory)) {
            return;
        }

        let request = FileRequest::exists("src");
        let response = execute_file(&request).unwrap();

        assert!(response.success);
        assert_eq!(response.exists, Some(true), "directories should exist too");
    }

    // ========================================================================
    // FileOp::CreateDir tests
    // ========================================================================

    #[test]
    fn test_file_create_dir_success() {
        if !guard_fs(stringify!(test_file_create_dir_success)) {
            return;
        }

        let path = temp_path("create_dir_test");
        fs::remove_dir_all(&path).ok(); // Ensure doesn't exist

        let request = FileRequest::create_dir(path.to_str().unwrap());
        let response = execute_file(&request).unwrap();

        assert!(response.success, "create_dir should succeed");
        assert_eq!(response.operation, FileOp::CreateDir);
        assert!(path.is_dir(), "directory should exist");

        fs::remove_dir(&path).ok();
    }

    #[test]
    fn test_file_create_dir_nested() {
        if !guard_fs(stringify!(test_file_create_dir_nested)) {
            return;
        }

        let mut path = temp_path("nested_dir");
        path.push("level1");
        path.push("level2");
        fs::remove_dir_all(temp_path("nested_dir")).ok();

        let request = FileRequest::create_dir(path.to_str().unwrap());
        let response = execute_file(&request).unwrap();

        assert!(response.success, "create_dir_all should create nested dirs");
        assert!(path.is_dir());

        fs::remove_dir_all(temp_path("nested_dir")).ok();
    }

    #[test]
    fn test_file_create_dir_already_exists() {
        if !guard_fs(stringify!(test_file_create_dir_already_exists)) {
            return;
        }

        let path = temp_path("existing_dir");
        fs::create_dir_all(&path).ok();

        let request = FileRequest::create_dir(path.to_str().unwrap());
        let response = execute_file(&request).unwrap();

        assert!(response.success, "create_dir on existing should succeed");

        fs::remove_dir(&path).ok();
    }

    // ========================================================================
    // Shell executor tests
    // ========================================================================

    #[test]
    fn test_shell_echo_basic() {
        if !guard_shell(stringify!(test_shell_echo_basic)) {
            return;
        }

        let request = ShellRequest::new("echo").arg("hello");
        let response = execute_shell(&request).unwrap();

        assert_eq!(response.exit_code, 0);
        assert!(response.stdout.contains("hello"));
        assert!(response.stderr.is_empty());
    }

    #[test]
    fn test_shell_multiple_args() {
        if !guard_shell(stringify!(test_shell_multiple_args)) {
            return;
        }

        let request = ShellRequest::new("echo").arg("one").arg("two").arg("three");
        let response = execute_shell(&request).unwrap();

        assert_eq!(response.exit_code, 0);
        assert!(response.stdout.contains("one two three"));
    }

    #[test]
    fn test_shell_with_stdin() {
        if !guard_shell(stringify!(test_shell_with_stdin)) {
            return;
        }

        let request = ShellRequest::new("cat").stdin("test input");
        let response = execute_shell(&request).unwrap();

        assert_eq!(response.exit_code, 0);
        assert_eq!(response.stdout.trim(), "test input");
    }

    #[test]
    fn test_shell_stderr_capture() {
        if !guard_shell(stringify!(test_shell_stderr_capture)) {
            return;
        }

        // Use sh -c to write to stderr
        let request = ShellRequest::new("sh")
            .arg("-c")
            .arg("echo error message >&2");
        let response = execute_shell(&request).unwrap();

        assert_eq!(response.exit_code, 0);
        assert!(
            response.stderr.contains("error message"),
            "stderr should be captured"
        );
    }

    #[test]
    fn test_shell_nonzero_exit() {
        if !guard_shell(stringify!(test_shell_nonzero_exit)) {
            return;
        }

        let request = ShellRequest::new("sh").arg("-c").arg("exit 42");
        let response = execute_shell(&request).unwrap();

        assert_eq!(response.exit_code, 42, "should capture non-zero exit code");
    }

    #[test]
    fn test_shell_false_command() {
        if !guard_shell(stringify!(test_shell_false_command)) {
            return;
        }

        let request = ShellRequest::new("false");
        let response = execute_shell(&request).unwrap();

        assert_ne!(response.exit_code, 0, "false command should fail");
    }

    #[test]
    fn test_shell_env_vars() {
        if !guard_shell(stringify!(test_shell_env_vars)) {
            return;
        }

        let request = ShellRequest::new("sh")
            .arg("-c")
            .arg("echo $TEST_VAR")
            .env("TEST_VAR", "custom_value");
        let response = execute_shell(&request).unwrap();

        assert_eq!(response.exit_code, 0);
        assert!(
            response.stdout.contains("custom_value"),
            "env var should be set"
        );
    }

    #[test]
    fn test_shell_working_directory() {
        if !guard_shell(stringify!(test_shell_working_directory)) {
            return;
        }

        let cwd = temp_dir();
        let request = ShellRequest::new("pwd").cwd(cwd.to_str().unwrap());
        let response = execute_shell(&request).unwrap();

        assert_eq!(response.exit_code, 0);
        // The output should contain the temp dir path (may have symlink resolution)
        // Just check it ran successfully
        assert!(!response.stdout.is_empty());
    }

    #[test]
    fn test_shell_command_not_found() {
        if !guard_shell(stringify!(test_shell_command_not_found)) {
            return;
        }

        let request = ShellRequest::new("nonexistent_command_xyz_12345");
        let result = execute_shell(&request);

        assert!(result.is_err(), "nonexistent command should error");
    }

    // ========================================================================
    // Top-level execute_transport dispatch tests
    // ========================================================================

    #[test]
    fn test_execute_transport_file_dispatch() {
        if !guard_fs(stringify!(test_execute_transport_file_dispatch)) {
            return;
        }

        let request = TransportRequest::File(FileRequest::exists("Cargo.toml"));
        let response = execute_transport(&request).unwrap();

        match response {
            TransportResponse::File(f) => {
                assert!(f.success);
                assert_eq!(f.exists, Some(true));
            }
            _ => panic!("expected File response"),
        }
    }

    #[test]
    fn test_execute_transport_shell_dispatch() {
        if !guard_shell(stringify!(test_execute_transport_shell_dispatch)) {
            return;
        }

        let request = TransportRequest::Shell(ShellRequest::new("echo").arg("test"));
        let response = execute_transport(&request).unwrap();

        match response {
            TransportResponse::Shell(s) => {
                assert_eq!(s.exit_code, 0);
                assert!(s.stdout.contains("test"));
            }
            _ => panic!("expected Shell response"),
        }
    }

    // ========================================================================
    // Git transport integration tests
    //
    // These tests create a real temp git repo and verify GitRequest commands
    // produce parseable output via the shell executor.
    // ========================================================================

    mod git_integration {
        use super::*;
        use gunbc_ir::transport::git::{
            parse_current_branch, parse_diff_chunks, parse_diff_name_only, parse_ls_files,
            GitRequest,
        };

        /// Helper: create a temp git repo for testing.
        ///
        /// Returns the path to the repo. Caller is responsible for cleanup.
        fn create_temp_git_repo(name: &str) -> PathBuf {
            let path = temp_path(&format!("git_{}", name));
            fs::create_dir_all(&path).expect("create dir");

            // git init
            let init = execute_shell(
                &ShellRequest::new("git")
                    .arg("init")
                    .cwd(path.to_str().unwrap()),
            )
            .expect("git init");
            assert_eq!(init.exit_code, 0, "git init failed: {}", init.stderr);

            // Configure user for commits (needed in CI)
            let _ = execute_shell(
                &ShellRequest::new("git")
                    .arg("config")
                    .arg("user.email")
                    .arg("test@example.com")
                    .cwd(path.to_str().unwrap()),
            );
            let _ = execute_shell(
                &ShellRequest::new("git")
                    .arg("config")
                    .arg("user.name")
                    .arg("Test User")
                    .cwd(path.to_str().unwrap()),
            );
            // Disable commit signing (some environments have it enabled globally)
            let _ = execute_shell(
                &ShellRequest::new("git")
                    .arg("config")
                    .arg("commit.gpgsign")
                    .arg("false")
                    .cwd(path.to_str().unwrap()),
            );

            path
        }

        /// Helper: add a file and commit it.
        fn add_and_commit(repo: &Path, filename: &str, content: &str, message: &str) {
            let filepath = repo.join(filename);
            if let Some(parent) = filepath.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(&filepath, content).expect("write file");

            let add = execute_shell(
                &ShellRequest::new("git")
                    .arg("add")
                    .arg(filename)
                    .cwd(repo.to_str().unwrap()),
            )
            .expect("git add");
            assert_eq!(add.exit_code, 0, "git add failed: {}", add.stderr);

            let commit = execute_shell(
                &ShellRequest::new("git")
                    .arg("commit")
                    .arg("-m")
                    .arg(message)
                    .cwd(repo.to_str().unwrap()),
            )
            .expect("git commit");
            assert_eq!(commit.exit_code, 0, "git commit failed: {}", commit.stderr);
        }

        /// Helper: cleanup temp git repo.
        fn cleanup_repo(path: &PathBuf) {
            fs::remove_dir_all(path).ok();
        }

        #[test]
        fn test_git_ls_files_integration() {
            if !guard_git(stringify!(test_git_ls_files_integration)) {
                return;
            }

            let repo = create_temp_git_repo("ls_files");

            // Add some files
            add_and_commit(&repo, "README.md", "# Test", "Initial commit");
            add_and_commit(&repo, "src/main.rs", "fn main() {}", "Add main.rs");

            // Build and execute GitRequest
            let request = GitRequest::ls_files()
                .cwd(repo.to_str().unwrap())
                .to_shell_request();

            let response = match request {
                TransportRequest::Shell(shell) => execute_shell(&shell).unwrap(),
                _ => panic!("expected Shell request"),
            };

            assert_eq!(response.exit_code, 0);

            // Parse and verify
            let files = parse_ls_files(&response.stdout);
            assert!(files.contains(&"README.md".to_string()));
            assert!(files.contains(&"src/main.rs".to_string()));

            cleanup_repo(&repo);
        }

        #[test]
        fn test_git_current_branch_integration() {
            if !guard_git(stringify!(test_git_current_branch_integration)) {
                return;
            }

            let repo = create_temp_git_repo("current_branch");
            add_and_commit(&repo, "file.txt", "content", "Initial");

            // Execute current_branch request
            let request = GitRequest::current_branch()
                .cwd(repo.to_str().unwrap())
                .to_shell_request();

            let response = match request {
                TransportRequest::Shell(shell) => execute_shell(&shell).unwrap(),
                _ => panic!("expected Shell request"),
            };

            assert_eq!(response.exit_code, 0);

            // Parse - should be "main" or "master" depending on git default
            let branch = parse_current_branch(&response.stdout);
            assert!(
                branch == "main" || branch == "master",
                "unexpected branch: {}",
                branch
            );

            cleanup_repo(&repo);
        }

        #[test]
        fn test_git_diff_integration() {
            if !guard_git(stringify!(test_git_diff_integration)) {
                return;
            }

            let repo = create_temp_git_repo("diff");

            // Initial commit on main
            add_and_commit(&repo, "file.txt", "line 1\n", "Initial");

            // Create a branch and make changes
            let branch = execute_shell(
                &ShellRequest::new("git")
                    .arg("checkout")
                    .arg("-b")
                    .arg("feature")
                    .cwd(repo.to_str().unwrap()),
            )
            .expect("checkout");
            assert_eq!(branch.exit_code, 0);

            add_and_commit(&repo, "file.txt", "line 1\nline 2\n", "Add line");
            add_and_commit(&repo, "new_file.rs", "fn hello() {}", "Add new file");

            // Diff against main
            // Note: We use "main" or "master" depending on what was created.
            // Let's detect which one exists.
            let detect_main = execute_shell(
                &ShellRequest::new("git")
                    .arg("rev-parse")
                    .arg("--verify")
                    .arg("main")
                    .cwd(repo.to_str().unwrap()),
            )
            .expect("detect main");

            let base_branch = if detect_main.exit_code == 0 {
                "main"
            } else {
                "master"
            };

            let request = GitRequest::diff(base_branch)
                .cwd(repo.to_str().unwrap())
                .to_shell_request();

            let response = match request {
                TransportRequest::Shell(shell) => execute_shell(&shell).unwrap(),
                _ => panic!("expected Shell request"),
            };

            assert_eq!(response.exit_code, 0);

            // Parse diff chunks
            let chunks = parse_diff_chunks(&response.stdout);
            // Should have diffs for both files
            assert!(
                chunks.contains_key("file.txt") || chunks.contains_key("new_file.rs"),
                "expected diff output, got: {:?}",
                chunks.keys().collect::<Vec<_>>()
            );

            cleanup_repo(&repo);
        }

        #[test]
        fn test_git_diff_name_only_integration() {
            if !guard_git(stringify!(test_git_diff_name_only_integration)) {
                return;
            }

            let repo = create_temp_git_repo("diff_name_only");

            // Initial commit
            add_and_commit(&repo, "unchanged.txt", "stays same", "Initial");

            // Branch and make changes
            let _ = execute_shell(
                &ShellRequest::new("git")
                    .arg("checkout")
                    .arg("-b")
                    .arg("changes")
                    .cwd(repo.to_str().unwrap()),
            );

            add_and_commit(&repo, "changed.txt", "new file", "Add changed file");

            // Detect base branch
            let detect = execute_shell(
                &ShellRequest::new("git")
                    .arg("rev-parse")
                    .arg("--verify")
                    .arg("main")
                    .cwd(repo.to_str().unwrap()),
            )
            .unwrap();
            let base = if detect.exit_code == 0 {
                "main"
            } else {
                "master"
            };

            let request = GitRequest::diff_name_only(base)
                .cwd(repo.to_str().unwrap())
                .to_shell_request();

            let response = match request {
                TransportRequest::Shell(shell) => execute_shell(&shell).unwrap(),
                _ => panic!("expected Shell request"),
            };

            assert_eq!(response.exit_code, 0);

            // Parse file names
            let files = parse_diff_name_only(&response.stdout);
            assert!(
                files.contains(&"changed.txt".to_string()),
                "expected changed.txt in {:?}",
                files
            );
            // unchanged.txt should NOT be in the list
            assert!(
                !files.contains(&"unchanged.txt".to_string()),
                "unchanged.txt should not be in diff"
            );

            cleanup_repo(&repo);
        }

        #[test]
        fn test_git_ls_files_with_pathspec() {
            if !guard_git(stringify!(test_git_ls_files_with_pathspec)) {
                return;
            }

            let repo = create_temp_git_repo("pathspec");

            add_and_commit(&repo, "src/main.rs", "fn main() {}", "Add main");
            add_and_commit(&repo, "src/lib.rs", "pub fn lib() {}", "Add lib");
            add_and_commit(&repo, "README.md", "# Readme", "Add readme");

            // ls-files with pathspec filtering
            let request = GitRequest::ls_files()
                .pathspecs(vec!["*.rs"])
                .cwd(repo.to_str().unwrap())
                .to_shell_request();

            let response = match request {
                TransportRequest::Shell(shell) => execute_shell(&shell).unwrap(),
                _ => panic!("expected Shell request"),
            };

            assert_eq!(response.exit_code, 0);

            let files = parse_ls_files(&response.stdout);
            // Should only include .rs files
            assert!(files.iter().all(|f| f.ends_with(".rs")));
            assert!(!files.contains(&"README.md".to_string()));

            cleanup_repo(&repo);
        }

        #[test]
        fn test_git_merge_base_integration() {
            if !guard_git(stringify!(test_git_merge_base_integration)) {
                return;
            }

            let repo = create_temp_git_repo("merge_base");

            // Create initial commit
            add_and_commit(&repo, "base.txt", "base", "Base commit");

            // Store main/master commit hash
            let main_hash = execute_shell(
                &ShellRequest::new("git")
                    .arg("rev-parse")
                    .arg("HEAD")
                    .cwd(repo.to_str().unwrap()),
            )
            .unwrap();
            let base_hash = main_hash.stdout.trim().to_string();

            // Create branch
            let _ = execute_shell(
                &ShellRequest::new("git")
                    .arg("checkout")
                    .arg("-b")
                    .arg("feature")
                    .cwd(repo.to_str().unwrap()),
            );

            add_and_commit(&repo, "feature.txt", "feature", "Feature commit");

            // Detect base branch name
            let detect = execute_shell(
                &ShellRequest::new("git")
                    .arg("rev-parse")
                    .arg("--verify")
                    .arg("main")
                    .cwd(repo.to_str().unwrap()),
            )
            .unwrap();
            let base_branch = if detect.exit_code == 0 {
                "main"
            } else {
                "master"
            };

            // Get merge-base
            let request = GitRequest::merge_base(base_branch)
                .cwd(repo.to_str().unwrap())
                .to_shell_request();

            let response = match request {
                TransportRequest::Shell(shell) => execute_shell(&shell).unwrap(),
                _ => panic!("expected Shell request"),
            };

            assert_eq!(response.exit_code, 0);
            // The merge-base should be the original main commit
            assert!(
                response.stdout.trim().starts_with(&base_hash[..8]),
                "expected merge-base to be {}, got {}",
                base_hash,
                response.stdout.trim()
            );

            cleanup_repo(&repo);
        }
    }
}
