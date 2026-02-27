//! Test-only transport backends.
//!
//! These are intended for integration tests that need deterministic, hermetic
//! I/O without touching the real filesystem or shell.

use crate::backend::TransportBackend;
use crate::executor::TransportError;
use gunbc_ir::transport::http::HttpMethod;
use gunbc_ir::transport::{
    FileOp, FileRequest, FileResponse, HttpRequest, HttpResponse, RestRequest, RestResponse,
    ShellRequest, ShellResponse, TcpRequest, TcpResponse, TransportRequest, TransportResponse,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct FileEntry {
    content: String,
    modified_millis: i64,
}

#[derive(Debug, Default)]
struct VirtualFilesystem {
    files: BTreeMap<String, FileEntry>,
    dirs: BTreeSet<String>,
}

impl VirtualFilesystem {
    fn normalize_path(path: &str) -> String {
        let mut out = path.replace('\\', "/");
        while out.contains("//") {
            out = out.replace("//", "/");
        }
        if out.starts_with("./") {
            out = out.trim_start_matches("./").to_string();
        }
        if out.ends_with('/') && out.len() > 1 {
            while out.ends_with('/') {
                out.pop();
            }
        }
        out
    }

    fn now_millis() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }

    fn ensure_dir(&mut self, path: &str) {
        let norm = Self::normalize_path(path);
        if norm.is_empty() || norm == "." {
            return;
        }
        let mut current = String::new();
        for part in norm.split('/') {
            if current.is_empty() {
                current.push_str(part);
            } else {
                current.push('/');
                current.push_str(part);
            }
            self.dirs.insert(current.clone());
        }
    }

    fn create_dir_all(&mut self, path: &str) {
        self.ensure_dir(path);
    }

    fn write_file(&mut self, path: &str, content: &str, create_parents: bool) -> FileResponse {
        let norm = Self::normalize_path(path);
        if create_parents {
            if let Some(parent) = norm.rsplit_once('/') {
                self.ensure_dir(parent.0);
            }
        }
        self.files.insert(
            norm.clone(),
            FileEntry {
                content: content.to_string(),
                modified_millis: Self::now_millis(),
            },
        );
        FileResponse::written(norm)
    }

    fn append_file(&mut self, path: &str, content: &str, create_parents: bool) -> FileResponse {
        let norm = Self::normalize_path(path);
        if create_parents {
            if let Some(parent) = norm.rsplit_once('/') {
                self.ensure_dir(parent.0);
            }
        }
        let entry = self.files.entry(norm.clone()).or_insert_with(|| FileEntry {
            content: String::new(),
            modified_millis: Self::now_millis(),
        });
        entry.content.push_str(content);
        entry.modified_millis = Self::now_millis();
        FileResponse {
            path: norm,
            operation: FileOp::Append,
            success: true,
            content: None,
            bytes: None,
            exists: None,
            error: None,
        }
    }

    fn read_file(&self, path: &str) -> FileResponse {
        let norm = Self::normalize_path(path);
        match self.files.get(&norm) {
            Some(entry) => FileResponse::read_ok(norm, entry.content.clone()),
            None => FileResponse::error(norm, FileOp::Read, "file not found"),
        }
    }

    fn delete_file(&mut self, path: &str) -> FileResponse {
        let norm = Self::normalize_path(path);
        if self.files.remove(&norm).is_some() {
            FileResponse {
                path: norm,
                operation: FileOp::Delete,
                success: true,
                content: None,
                bytes: None,
                exists: None,
                error: None,
            }
        } else {
            FileResponse::error(norm, FileOp::Delete, "file not found")
        }
    }

    fn exists(&self, path: &str) -> FileResponse {
        let norm = Self::normalize_path(path);
        let exists = self.files.contains_key(&norm) || self.dirs.contains(&norm);
        FileResponse::exists_result(norm, exists)
    }

    fn metadata(&self, path: &str) -> FileResponse {
        let norm = Self::normalize_path(path);
        match self.files.get(&norm) {
            Some(entry) => FileResponse::metadata_result(norm, entry.modified_millis),
            None => FileResponse::error(norm, FileOp::Metadata, "file not found"),
        }
    }

    fn glob_files(&self, pattern: &str) -> FileResponse {
        let pat = match glob::Pattern::new(pattern) {
            Ok(p) => p,
            Err(e) => return FileResponse::error(pattern.to_string(), FileOp::Glob, e.to_string()),
        };

        let mut matches: Vec<String> = self
            .files
            .keys()
            .filter(|path| pat.matches_path(Path::new(path)))
            .cloned()
            .collect();
        matches.sort();
        FileResponse::glob_result(pattern.to_string(), matches)
    }

    fn list_dirs(&self, base: &str, maxdepth: usize, mindepth: usize) -> Vec<String> {
        let base_norm = Self::normalize_path(base);
        let base_prefix = if base_norm.is_empty() {
            String::new()
        } else {
            format!("{}/", base_norm)
        };

        let mut out = Vec::new();
        for dir in &self.dirs {
            let rel = if base_norm.is_empty() {
                dir.as_str()
            } else if dir == &base_norm {
                continue;
            } else if dir.starts_with(&base_prefix) {
                &dir[base_prefix.len()..]
            } else {
                continue;
            };

            if rel.is_empty() {
                continue;
            }

            let depth = rel.split('/').count();
            if depth >= mindepth && depth <= maxdepth {
                out.push(dir.clone());
            }
        }

        out.sort();
        out
    }
}

// ── Shell cassette registry (RT10) ────────────────────────────────────

/// A pre-recorded shell command → response mapping.
/// The virtual backend matches incoming `ShellRequest`s against cassettes
/// by `(command, args)` tuple. First match wins; unmatched commands fall
/// through to the built-in handlers (find, printenv, sh) or error.
#[derive(Debug, Clone)]
pub struct ShellCassette {
    /// Executable name (e.g., "cargo", "git", "gcloud").
    pub command: String,
    /// Expected argument list. Matching modes:
    /// - Non-empty: exact match on `request.args`
    /// - Empty: matches any args for this command
    pub args: Vec<String>,
    /// Stdout to return.
    pub stdout: String,
    /// Stderr to return.
    pub stderr: String,
    /// Exit code to return.
    pub exit_code: i32,
}

// ── HTTP/REST stub registry (RT11) ────────────────────────────────────

/// A pre-recorded HTTP request → response mapping.
/// The virtual backend matches incoming `RestRequest`s and `HttpRequest`s
/// against stubs by `(method, url_path)`. First match wins.
#[derive(Debug, Clone)]
pub struct HttpStub {
    /// HTTP method to match (None = match any method).
    pub method: Option<HttpMethod>,
    /// URL path prefix to match (e.g., "/gists").
    /// Empty string matches all paths.
    pub path_pattern: String,
    /// When true, path must match exactly (not prefix).
    pub exact_path: bool,
    /// HTTP status code to return.
    pub status: u16,
    /// Response body (JSON string for REST, raw for HTTP).
    pub response_body: String,
    /// Response headers.
    pub response_headers: HashMap<String, String>,
}

impl HttpStub {
    /// Match this stub against a (method, url) pair.
    fn matches(&self, method: &HttpMethod, url: &str) -> bool {
        if let Some(ref m) = self.method {
            if m != method {
                return false;
            }
        }
        // Extract path from URL (strip scheme + host).
        let path = extract_url_path(url);
        if self.exact_path {
            path == self.path_pattern
        } else {
            path.starts_with(&self.path_pattern)
        }
    }
}

/// Extract the path component from a URL string.
fn extract_url_path(url: &str) -> &str {
    // Strip "https://host" or "http://host" prefix.
    let after_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    // Find the first '/' after the host.
    after_scheme
        .find('/')
        .map(|i| &after_scheme[i..])
        .unwrap_or("/")
}

// ── TCP loopback registry (RT12) ──────────────────────────────────────

/// A TCP loopback configuration for connection-level testing.
/// Matches by port number and returns canned data.
#[derive(Debug, Clone)]
pub struct TcpLoopback {
    /// Port number to match.
    pub port: u16,
    /// Data to return on connection.
    pub response_data: String,
}

/// Virtual transport backend for deterministic, hermetic integration tests.
#[derive(Debug, Default)]
pub struct VirtualTransportBackend {
    fs: Mutex<VirtualFilesystem>,
    shell_cassettes: Mutex<Vec<ShellCassette>>,
    http_stubs: Mutex<Vec<HttpStub>>,
    tcp_loopbacks: Mutex<Vec<TcpLoopback>>,
}

impl VirtualTransportBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_dir_all(&self, path: &str) {
        let mut fs = self.fs.lock().expect("virtual fs lock poisoned");
        fs.create_dir_all(path);
    }

    pub fn read_file(&self, path: &str) -> Option<String> {
        let fs = self.fs.lock().expect("virtual fs lock poisoned");
        let norm = VirtualFilesystem::normalize_path(path);
        fs.files.get(&norm).map(|entry| entry.content.clone())
    }

    /// Register a shell cassette (pre-recorded command → response).
    pub fn add_shell_cassette(&self, cassette: ShellCassette) {
        self.shell_cassettes
            .lock()
            .expect("shell cassettes lock poisoned")
            .push(cassette);
    }

    /// Register an HTTP stub (pre-recorded request → response).
    pub fn add_http_stub(&self, stub: HttpStub) {
        self.http_stubs
            .lock()
            .expect("http stubs lock poisoned")
            .push(stub);
    }

    /// Register a TCP loopback (port → canned response).
    pub fn add_tcp_loopback(&self, loopback: TcpLoopback) {
        self.tcp_loopbacks
            .lock()
            .expect("tcp loopbacks lock poisoned")
            .push(loopback);
    }

    fn execute_file(&self, request: &FileRequest) -> FileResponse {
        let mut fs = self.fs.lock().expect("virtual fs lock poisoned");
        match request.operation {
            FileOp::Read => fs.read_file(&request.path),
            FileOp::ReadBytes => {
                let resp = fs.read_file(&request.path);
                if resp.success {
                    FileResponse::read_bytes_ok(
                        &request.path,
                        resp.content.unwrap_or_default().into_bytes(),
                    )
                } else {
                    FileResponse::error(
                        &request.path,
                        FileOp::ReadBytes,
                        resp.error.unwrap_or_default(),
                    )
                }
            }
            FileOp::Write => fs.write_file(
                &request.path,
                request.content.as_deref().unwrap_or(""),
                request.create_parents,
            ),
            FileOp::Append => fs.append_file(
                &request.path,
                request.content.as_deref().unwrap_or(""),
                request.create_parents,
            ),
            FileOp::Delete => fs.delete_file(&request.path),
            FileOp::Exists => fs.exists(&request.path),
            FileOp::CreateDir => {
                fs.create_dir_all(&request.path);
                FileResponse {
                    path: VirtualFilesystem::normalize_path(&request.path),
                    operation: FileOp::CreateDir,
                    success: true,
                    content: None,
                    bytes: None,
                    exists: None,
                    error: None,
                }
            }
            FileOp::Glob => fs.glob_files(&request.path),
            FileOp::Metadata => fs.metadata(&request.path),
        }
    }

    fn execute_shell(&self, request: &ShellRequest) -> Result<ShellResponse, TransportError> {
        // Check cassette registry first (RT10).
        if let Some(response) = self.match_shell_cassette(request) {
            return Ok(response);
        }
        // Fall through to built-in handlers.
        match request.command.as_str() {
            "find" => self.execute_find(request),
            "printenv" => self.execute_printenv(request),
            "sh" => self.execute_sh(request),
            other => Err(TransportError::new(format!(
                "virtual backend: unsupported shell command '{}'",
                other
            ))),
        }
    }

    /// Match a shell request against registered cassettes.
    fn match_shell_cassette(&self, request: &ShellRequest) -> Option<ShellResponse> {
        let cassettes = self
            .shell_cassettes
            .lock()
            .expect("shell cassettes lock poisoned");
        for cassette in cassettes.iter() {
            if cassette.command != request.command {
                continue;
            }
            // Empty args = match any args for this command.
            if !cassette.args.is_empty() && cassette.args != request.args {
                continue;
            }
            return Some(ShellResponse {
                exit_code: cassette.exit_code,
                stdout: cassette.stdout.clone(),
                stderr: cassette.stderr.clone(),
            });
        }
        None
    }

    /// Execute a REST request against the HTTP stub registry (RT11).
    fn execute_rest(&self, request: &RestRequest) -> Result<RestResponse, TransportError> {
        let stubs = self
            .http_stubs
            .lock()
            .expect("http stubs lock poisoned");
        for stub in stubs.iter() {
            if stub.matches(&request.method, &request.url) {
                let body: serde_json::Value =
                    serde_json::from_str(&stub.response_body).unwrap_or_else(|_| {
                        serde_json::Value::String(stub.response_body.clone())
                    });
                return Ok(RestResponse {
                    status: stub.status,
                    headers: stub.response_headers.clone(),
                    body,
                });
            }
        }
        Err(TransportError::new(format!(
            "virtual backend: no HTTP stub matches {} {}",
            request.method, request.url
        )))
    }

    /// Execute an HTTP request against the HTTP stub registry (RT11).
    fn execute_http(&self, request: &HttpRequest) -> Result<HttpResponse, TransportError> {
        let stubs = self
            .http_stubs
            .lock()
            .expect("http stubs lock poisoned");
        for stub in stubs.iter() {
            if stub.matches(&request.method, &request.url) {
                return Ok(HttpResponse {
                    status: stub.status,
                    headers: stub.response_headers.clone(),
                    body: stub.response_body.clone(),
                });
            }
        }
        Err(TransportError::new(format!(
            "virtual backend: no HTTP stub matches {} {}",
            request.method, request.url
        )))
    }

    /// Execute a TCP request against the loopback registry (RT12).
    fn execute_tcp(&self, request: &TcpRequest) -> Result<TcpResponse, TransportError> {
        let loopbacks = self
            .tcp_loopbacks
            .lock()
            .expect("tcp loopbacks lock poisoned");
        for loopback in loopbacks.iter() {
            if loopback.port == request.port {
                let bytes_sent = request.data.as_ref().map_or(0, |d| d.len());
                return Ok(TcpResponse::ok(
                    Some(loopback.response_data.clone()),
                    bytes_sent,
                    loopback.response_data.len(),
                ));
            }
        }
        Err(TransportError::new(format!(
            "virtual backend: no TCP loopback on port {}",
            request.port
        )))
    }

    fn execute_find(&self, request: &ShellRequest) -> Result<ShellResponse, TransportError> {
        if request.args.is_empty() {
            return Err(TransportError::new("virtual find: missing path"));
        }

        let base = request.args[0].as_str();
        let mut maxdepth: Option<usize> = None;
        let mut mindepth: Option<usize> = None;
        let mut type_filter: Option<&str> = None;

        let mut i = 1;
        while i < request.args.len() {
            match request.args[i].as_str() {
                "-maxdepth" => {
                    i += 1;
                    maxdepth = request.args.get(i).and_then(|v| v.parse().ok());
                }
                "-mindepth" => {
                    i += 1;
                    mindepth = request.args.get(i).and_then(|v| v.parse().ok());
                }
                "-type" => {
                    i += 1;
                    type_filter = request.args.get(i).map(|v| v.as_str());
                }
                _ => {}
            }
            i += 1;
        }

        if type_filter != Some("d") {
            return Err(TransportError::new(
                "virtual find: only -type d is supported",
            ));
        }

        let maxdepth = maxdepth.unwrap_or(1);
        let mindepth = mindepth.unwrap_or(1);

        let fs = self.fs.lock().expect("virtual fs lock poisoned");
        let dirs = fs.list_dirs(base, maxdepth, mindepth);
        let mut stdout = dirs.join("\n");
        if !stdout.is_empty() {
            stdout.push('\n');
        }

        Ok(ShellResponse::ok(stdout))
    }

    fn execute_printenv(&self, request: &ShellRequest) -> Result<ShellResponse, TransportError> {
        // printenv in the virtual backend always returns empty (env var not set).
        // The bootstrap graph calls services.shell::GetEnv which compiles to
        // `printenv <name>`. In tests we return exit code 1 (var not set).
        let name = request.args.first().map(|s| s.as_str()).unwrap_or("");
        // Check the request's env map first (for test-injected values).
        if let Some(value) = request.env.get(name) {
            return Ok(ShellResponse::ok(value));
        }
        Ok(ShellResponse::failed(1, ""))
    }

    fn execute_sh(&self, request: &ShellRequest) -> Result<ShellResponse, TransportError> {
        if request.args.len() >= 2 && request.args[0] == "-c" {
            let script = request.args[1].as_str();
            if let Some(paths) = parse_test_f_chain(script) {
                let fs = self.fs.lock().expect("virtual fs lock poisoned");
                let ok = paths
                    .iter()
                    .all(|p| fs.files.contains_key(&VirtualFilesystem::normalize_path(p)));
                return if ok {
                    Ok(ShellResponse::ok(""))
                } else {
                    Ok(ShellResponse::failed(1, "missing"))
                };
            }
        }
        Err(TransportError::new("virtual sh: unsupported script"))
    }
}

impl TransportBackend for VirtualTransportBackend {
    fn execute(&self, request: &TransportRequest) -> Result<TransportResponse, TransportError> {
        match request {
            TransportRequest::File(req) => Ok(TransportResponse::File(self.execute_file(req))),
            TransportRequest::Shell(req) => self.execute_shell(req).map(TransportResponse::Shell),
            TransportRequest::Local(req) => Ok(TransportResponse::Local(
                gunbc_ir::transport::LocalResponse {
                    outputs: req.inputs.clone(),
                },
            )),
            TransportRequest::Rest(req) => self.execute_rest(req).map(TransportResponse::Rest),
            TransportRequest::Http(req) => self.execute_http(req).map(TransportResponse::Http),
            TransportRequest::Tcp(req) => self.execute_tcp(req).map(TransportResponse::Tcp),
        }
    }
}

fn parse_test_f_chain(script: &str) -> Option<Vec<String>> {
    let mut paths = Vec::new();
    for part in script.split("&&") {
        let part = part.trim();
        let rest = part.strip_prefix("test -f ")?;
        let path = unquote(rest.trim());
        paths.push(path.to_string());
    }
    Some(paths)
}

fn unquote(value: &str) -> &str {
    let trimmed = value.trim();
    if trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2 {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    }
}
