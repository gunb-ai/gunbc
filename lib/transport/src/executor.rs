//! Transport request executors.

use gunbc_ir::transport::{
    FileOp, FileRequest, FileResponse, HttpRequest, HttpResponse, RestRequest,
    RestResponse, ShellRequest, ShellResponse, TcpRequest, TcpResponse, TransportRequest,
    TransportResponse,
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
/// Note: This is a simplified implementation. In production, you'd use
/// a proper HTTP client like reqwest.
fn execute_rest(request: &RestRequest) -> Result<RestResponse, TransportError> {
    // For now, convert to HTTP and use that
    let mut http_req = HttpRequest::post(&request.url);
    http_req.method = request.method;

    // Add headers
    for (k, v) in &request.headers {
        http_req.headers.insert(k.clone(), v.clone());
    }

    // Add JSON body
    if let Some(ref body) = request.body {
        http_req.body = Some(serde_json::to_string(body).map_err(|e| {
            TransportError::new(format!("failed to serialize body: {}", e))
        })?);
        http_req
            .headers
            .insert("Content-Type".to_string(), "application/json".to_string());
    }

    // Handle auth — EnvVar/EnvVarHeader should be resolved before reaching
    // the executor (see TransportOps::Execute in ops.rs).
    if let Some(ref auth) = request.auth {
        match auth {
            gunbc_ir::transport::AuthMethod::Bearer(token) => {
                http_req
                    .headers
                    .insert("Authorization".to_string(), format!("Bearer {}", token));
            }
            gunbc_ir::transport::AuthMethod::Basic { username, password } => {
                let creds = base64_encode(&format!("{}:{}", username, password));
                http_req
                    .headers
                    .insert("Authorization".to_string(), format!("Basic {}", creds));
            }
            gunbc_ir::transport::AuthMethod::ApiKey { header, key } => {
                http_req.headers.insert(header.clone(), key.clone());
            }
            gunbc_ir::transport::AuthMethod::EnvVar(_)
            | gunbc_ir::transport::AuthMethod::EnvVarHeader { .. } => {
                debug_assert!(
                    false,
                    "EnvVar/EnvVarHeader auth should be resolved before reaching the executor"
                );
                // Graceful fallback: skip auth if not resolved
            }
            gunbc_ir::transport::AuthMethod::None => {}
        }
    }

    http_req.timeout_ms = request.timeout_ms;

    let http_resp = execute_http(&http_req)?;

    // Parse JSON response
    let body: serde_json::Value = serde_json::from_str(&http_resp.body).unwrap_or_else(|_| {
        serde_json::json!({ "raw": http_resp.body })
    });

    Ok(RestResponse {
        status: http_resp.status,
        headers: http_resp.headers,
        body,
    })
}

/// Simple base64 encoding for basic auth.
fn base64_encode(input: &str) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = input.as_bytes();
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);
        
        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }
        
        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }
    }

    result
}

/// Execute a raw HTTP request.
///
/// This is a simplified implementation using std::net.
/// In production, use reqwest or similar.
fn execute_http(request: &HttpRequest) -> Result<HttpResponse, TransportError> {
    // Parse URL to extract host and path
    let url = &request.url;
    
    // For now, return a mock response for non-local URLs
    // A real implementation would use reqwest or similar
    if !url.starts_with("http://localhost") && !url.starts_with("http://127.0.0.1") {
        return Err(TransportError::new(
            "HTTP transport not fully implemented - use Shell transport with curl for now"
        ));
    }

    // Simple localhost handling
    let parts: Vec<&str> = url.trim_start_matches("http://").splitn(2, '/').collect();
    let host_port = parts[0];
    let path = format!("/{}", parts.get(1).unwrap_or(&""));

    let mut stream = TcpStream::connect(host_port)
        .map_err(|e| TransportError::new(format!("connection failed: {}", e)))?;

    if let Some(timeout) = request.timeout_ms {
        stream.set_read_timeout(Some(Duration::from_millis(timeout))).ok();
        stream.set_write_timeout(Some(Duration::from_millis(timeout))).ok();
    }

    // Build request
    let mut req_str = format!("{} {} HTTP/1.1\r\n", request.method, path);
    req_str.push_str(&format!("Host: {}\r\n", host_port));
    
    for (key, value) in &request.headers {
        req_str.push_str(&format!("{}: {}\r\n", key, value));
    }

    if let Some(ref body) = request.body {
        req_str.push_str(&format!("Content-Length: {}\r\n", body.len()));
    }

    req_str.push_str("\r\n");

    if let Some(ref body) = request.body {
        req_str.push_str(body);
    }

    stream.write_all(req_str.as_bytes())
        .map_err(|e| TransportError::new(format!("write failed: {}", e)))?;

    // Read response
    let mut response = String::new();
    stream.read_to_string(&mut response)
        .map_err(|e| TransportError::new(format!("read failed: {}", e)))?;

    // Parse response (very basic)
    let mut lines = response.lines();
    let status_line = lines.next().unwrap_or("HTTP/1.1 500 Error");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(500);

    let mut headers = HashMap::new();
    let mut body = String::new();
    let mut in_body = false;

    for line in lines {
        if in_body {
            body.push_str(line);
            body.push('\n');
        } else if line.is_empty() {
            in_body = true;
        } else if let Some((key, value)) = line.split_once(": ") {
            headers.insert(key.to_string(), value.to_string());
        }
    }

    Ok(HttpResponse {
        status,
        headers,
        body,
    })
}

/// Execute a file operation.
fn execute_file(request: &FileRequest) -> Result<FileResponse, TransportError> {
    match request.operation {
        FileOp::Read => {
            match fs::read_to_string(&request.path) {
                Ok(content) => Ok(FileResponse::read_ok(&request.path, content)),
                Err(e) => Ok(FileResponse::error(&request.path, FileOp::Read, e.to_string())),
            }
        }
        FileOp::Write => {
            if request.create_parents {
                if let Some(parent) = std::path::Path::new(&request.path).parent() {
                    fs::create_dir_all(parent).ok();
                }
            }
            
            let content = request.content.as_deref().unwrap_or("");
            match fs::write(&request.path, content) {
                Ok(()) => Ok(FileResponse::written(&request.path)),
                Err(e) => Ok(FileResponse::error(&request.path, FileOp::Write, e.to_string())),
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
                Ok(mut file) => {
                    match file.write_all(content.as_bytes()) {
                        Ok(()) => Ok(FileResponse {
                            path: request.path.clone(),
                            operation: FileOp::Append,
                            success: true,
                            content: None,
                            exists: None,
                            error: None,
                        }),
                        Err(e) => Ok(FileResponse::error(&request.path, FileOp::Append, e.to_string())),
                    }
                }
                Err(e) => Ok(FileResponse::error(&request.path, FileOp::Append, e.to_string())),
            }
        }
        FileOp::Delete => {
            match fs::remove_file(&request.path) {
                Ok(()) => Ok(FileResponse {
                    path: request.path.clone(),
                    operation: FileOp::Delete,
                    success: true,
                    content: None,
                    exists: None,
                    error: None,
                }),
                Err(e) => Ok(FileResponse::error(&request.path, FileOp::Delete, e.to_string())),
            }
        }
        FileOp::Exists => {
            let exists = std::path::Path::new(&request.path).exists();
            Ok(FileResponse::exists_result(&request.path, exists))
        }
        FileOp::CreateDir => {
            match fs::create_dir_all(&request.path) {
                Ok(()) => Ok(FileResponse {
                    path: request.path.clone(),
                    operation: FileOp::CreateDir,
                    success: true,
                    content: None,
                    exists: None,
                    error: None,
                }),
                Err(e) => Ok(FileResponse::error(&request.path, FileOp::CreateDir, e.to_string())),
            }
        }
    }
}

/// Execute a TCP request.
fn execute_tcp(request: &TcpRequest) -> Result<TcpResponse, TransportError> {
    let addr = format!("{}:{}", request.host, request.port);
    
    let mut stream = TcpStream::connect(&addr)
        .map_err(|e| TransportError::new(format!("connection failed: {}", e)))?;

    if let Some(timeout) = request.connect_timeout_ms {
        stream.set_read_timeout(Some(Duration::from_millis(timeout))).ok();
    }
    if let Some(timeout) = request.read_timeout_ms {
        stream.set_write_timeout(Some(Duration::from_millis(timeout))).ok();
    }

    let mut bytes_sent = 0;
    if let Some(ref data) = request.data {
        stream.write_all(data.as_bytes())
            .map_err(|e| TransportError::new(format!("write failed: {}", e)))?;
        bytes_sent = data.len();
    }

    let mut response = String::new();
    stream.read_to_string(&mut response).ok(); // May timeout, that's ok
    let bytes_received = response.len();

    Ok(TcpResponse::ok(
        if response.is_empty() { None } else { Some(response) },
        bytes_sent,
        bytes_received,
    ))
}

/// Execute a shell command.
///
/// This is the I/O boundary - the official place where Command::new is used.
/// All shell execution flows through this function.
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

    let mut child = cmd.spawn()
        .map_err(|e| TransportError::new(format!("failed to spawn: {}", e)))?;

    // Write stdin if provided
    if let Some(ref stdin_data) = request.stdin {
        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(stdin_data.as_bytes()).ok();
        }
    }

    let output = child.wait_with_output()
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

    #[test]
    fn test_file_read() {
        let request = FileRequest::read("Cargo.toml");
        let response = execute_file(&request).unwrap();
        
        assert!(response.success);
        assert!(response.content.is_some());
        assert!(response.content.unwrap().contains("[package]"));
    }

    #[test]
    fn test_file_exists() {
        let request = FileRequest::exists("Cargo.toml");
        let response = execute_file(&request).unwrap();
        
        assert!(response.success);
        assert_eq!(response.exists, Some(true));
    }

    #[test]
    fn test_file_not_exists() {
        let request = FileRequest::exists("nonexistent_file_12345.txt");
        let response = execute_file(&request).unwrap();
        
        assert!(response.success);
        assert_eq!(response.exists, Some(false));
    }

    #[test]
    fn test_shell_echo() {
        let request = ShellRequest::new("echo").arg("hello");
        let response = execute_shell(&request).unwrap();
        
        assert_eq!(response.exit_code, 0);
        assert!(response.stdout.contains("hello"));
    }

    #[test]
    fn test_shell_with_stdin() {
        let request = ShellRequest::new("cat").stdin("test input");
        let response = execute_shell(&request).unwrap();
        
        assert_eq!(response.exit_code, 0);
        assert_eq!(response.stdout.trim(), "test input");
    }

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode("hello"), "aGVsbG8=");
        assert_eq!(base64_encode("user:pass"), "dXNlcjpwYXNz");
    }
}
