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
    /// Executable name (e.g., "cargo", "git", "secretctl").
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

/// Argument matching mode for advanced shell cassette rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellArgMatchMode {
    Exact,
    Glob,
    Any,
}

/// A richer shell cassette rule for hermetic shell testing.
#[derive(Debug, Clone)]
pub struct ShellCassetteRule {
    pub command: String,
    pub args: Vec<String>,
    pub args_mode: ShellArgMatchMode,
    pub cwd: Option<String>,
    pub required_env: HashMap<String, String>,
    pub stdin: Option<String>,
    /// Ordered responses: first call returns index 0, second returns index 1, ...
    /// Exhaustion is an error.
    pub responses: Vec<ShellResponse>,
}

#[derive(Debug, Clone)]
struct RegisteredShellCassette {
    rule: ShellCassetteRule,
    next_response: usize,
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

/// HTTP path matching strategy for advanced stubs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpPathMatchMode {
    Prefix,
    Exact,
    /// Parameterized template path (e.g. `/repos/{owner}/{repo}/issues`).
    Template,
}

/// One HTTP response entry in an ordered stub sequence.
#[derive(Debug, Clone)]
pub struct HttpStubResponse {
    pub status: u16,
    pub response_body: String,
    pub response_headers: HashMap<String, String>,
}

/// A richer HTTP stub rule for method+path+header matching with ordered responses.
#[derive(Debug, Clone)]
pub struct HttpStubRule {
    pub method: Option<HttpMethod>,
    pub path_pattern: String,
    pub path_mode: HttpPathMatchMode,
    pub required_headers: Vec<(String, String)>,
    pub responses: Vec<HttpStubResponse>,
}

#[derive(Debug, Clone)]
struct RegisteredHttpStub {
    rule: HttpStubRule,
    next_response: usize,
}

#[derive(Debug, Clone)]
struct MatchedHttpStubResponse {
    status: u16,
    response_body: String,
    response_headers: HashMap<String, String>,
}

/// Extract the path component from a URL string (without query or fragment).
fn extract_url_path(url: &str) -> &str {
    // Strip "https://host" or "http://host" prefix.
    let after_scheme = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .unwrap_or(url);
    // Find the first '/' after the host.
    let path = after_scheme
        .find('/')
        .map(|i| &after_scheme[i..])
        .unwrap_or("/");
    // Strip query string and fragment.
    let path = path.split('?').next().unwrap_or(path);
    path.split('#').next().unwrap_or(path)
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
    shell_cassettes: Mutex<Vec<RegisteredShellCassette>>,
    http_stubs: Mutex<Vec<RegisteredHttpStub>>,
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
        let args_mode = if cassette.args.is_empty() {
            ShellArgMatchMode::Any
        } else {
            ShellArgMatchMode::Exact
        };
        self.add_shell_cassette_rule(ShellCassetteRule {
            command: cassette.command,
            args: cassette.args,
            args_mode,
            cwd: None,
            required_env: HashMap::new(),
            stdin: None,
            responses: vec![ShellResponse {
                exit_code: cassette.exit_code,
                stdout: cassette.stdout,
                stderr: cassette.stderr,
            }],
        });
    }

    /// Register an advanced shell cassette rule.
    pub fn add_shell_cassette_rule(&self, rule: ShellCassetteRule) {
        assert!(
            !rule.responses.is_empty(),
            "shell cassette rule must have at least one response"
        );
        self.shell_cassettes
            .lock()
            .expect("shell cassettes lock poisoned")
            .push(RegisteredShellCassette {
                rule,
                next_response: 0,
            });
    }

    /// Register an HTTP stub (pre-recorded request → response).
    pub fn add_http_stub(&self, stub: HttpStub) {
        let path_mode = if stub.exact_path {
            HttpPathMatchMode::Exact
        } else {
            HttpPathMatchMode::Prefix
        };
        self.add_http_stub_rule(HttpStubRule {
            method: stub.method,
            path_pattern: stub.path_pattern,
            path_mode,
            required_headers: Vec::new(),
            responses: vec![HttpStubResponse {
                status: stub.status,
                response_body: stub.response_body,
                response_headers: stub.response_headers,
            }],
        });
    }

    /// Register an advanced HTTP stub rule.
    pub fn add_http_stub_rule(&self, rule: HttpStubRule) {
        assert!(
            !rule.responses.is_empty(),
            "http stub rule must have at least one response"
        );
        self.http_stubs
            .lock()
            .expect("http stubs lock poisoned")
            .push(RegisteredHttpStub {
                rule,
                next_response: 0,
            });
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
        if let Some(response) = self.match_shell_cassette(request)? {
            return Ok(response);
        }
        // Fall through to built-in handlers.
        match request.command.as_str() {
            "find" => self.execute_find(request),
            "printenv" => self.execute_printenv(request),
            "sh" => self.execute_sh(request),
            other => Err(TransportError::new(format!(
                "virtual backend: unsupported shell command '{}' args={:?} cwd={:?}. {}",
                other,
                request.args,
                request.cwd,
                self.shell_stub_diagnostics()
            ))),
        }
    }

    /// Match a shell request against registered cassettes.
    fn match_shell_cassette(
        &self,
        request: &ShellRequest,
    ) -> Result<Option<ShellResponse>, TransportError> {
        let mut cassettes = self
            .shell_cassettes
            .lock()
            .expect("shell cassettes lock poisoned");
        for cassette in cassettes.iter_mut() {
            if cassette.rule.command != request.command {
                continue;
            }
            if !shell_cassette_rule_matches(&cassette.rule, request) {
                continue;
            }
            if cassette.next_response >= cassette.rule.responses.len() {
                return Err(TransportError::new(format!(
                    "virtual backend: shell cassette sequence exhausted for command '{}' (matched rule args={:?}, mode={:?})",
                    cassette.rule.command, cassette.rule.args, cassette.rule.args_mode
                )));
            }
            let response = cassette.rule.responses[cassette.next_response].clone();
            cassette.next_response += 1;
            return Ok(Some(response));
        }
        Ok(None)
    }

    /// Execute a REST request against the HTTP stub registry (RT11).
    fn execute_rest(&self, request: &RestRequest) -> Result<RestResponse, TransportError> {
        let matched = self.match_http_stub(&request.method, &request.url, &request.headers)?;
        let Some(stub) = matched else {
            return Err(TransportError::new(format!(
                "virtual backend: no HTTP stub matches {} {} headers={:?}. {}",
                request.method,
                request.url,
                request.headers,
                self.http_stub_diagnostics()
            )));
        };
        let body: serde_json::Value = if stub.response_body.is_empty() {
            serde_json::json!({})
        } else {
            serde_json::from_str(&stub.response_body).map_err(|e| {
                TransportError::new(format!(
                    "virtual backend: invalid JSON in stub response body: {e}"
                ))
            })?
        };
        Ok(RestResponse {
            status: stub.status,
            headers: stub.response_headers,
            body,
        })
    }

    /// Execute an HTTP request against the HTTP stub registry (RT11).
    fn execute_http(&self, request: &HttpRequest) -> Result<HttpResponse, TransportError> {
        let matched = self.match_http_stub(&request.method, &request.url, &request.headers)?;
        let Some(stub) = matched else {
            return Err(TransportError::new(format!(
                "virtual backend: no HTTP stub matches {} {} headers={:?}. {}",
                request.method,
                request.url,
                request.headers,
                self.http_stub_diagnostics()
            )));
        };
        Ok(HttpResponse {
            status: stub.status,
            headers: stub.response_headers,
            body: stub.response_body,
        })
    }

    fn match_http_stub(
        &self,
        method: &HttpMethod,
        url: &str,
        headers: &HashMap<String, String>,
    ) -> Result<Option<MatchedHttpStubResponse>, TransportError> {
        let path = extract_url_path(url);
        let mut stubs = self.http_stubs.lock().expect("http stubs lock poisoned");
        for stub in stubs.iter_mut() {
            if !http_stub_rule_matches(&stub.rule, method, path, headers) {
                continue;
            }
            if stub.next_response >= stub.rule.responses.len() {
                return Err(TransportError::new(format!(
                    "virtual backend: HTTP stub response sequence exhausted for {} {} (path_mode={:?})",
                    method, stub.rule.path_pattern, stub.rule.path_mode
                )));
            }
            let response = stub.rule.responses[stub.next_response].clone();
            stub.next_response += 1;
            return Ok(Some(MatchedHttpStubResponse {
                status: response.status,
                response_body: response.response_body,
                response_headers: response.response_headers,
            }));
        }
        Ok(None)
    }

    fn http_stub_diagnostics(&self) -> String {
        let stubs = self.http_stubs.lock().expect("http stubs lock poisoned");
        if stubs.is_empty() {
            return "registered stubs: <none>".to_string();
        }
        let entries = stubs
            .iter()
            .enumerate()
            .map(|(idx, stub)| {
                format!(
                    "#{idx} method={:?} path={} mode={:?} headers={:?} remaining={}",
                    stub.rule.method,
                    stub.rule.path_pattern,
                    stub.rule.path_mode,
                    stub.rule.required_headers,
                    stub.rule.responses.len().saturating_sub(stub.next_response)
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        format!("registered stubs: [{entries}]")
    }

    fn shell_stub_diagnostics(&self) -> String {
        let cassettes = self
            .shell_cassettes
            .lock()
            .expect("shell cassettes lock poisoned");
        if cassettes.is_empty() {
            return "registered shell cassettes: <none>".to_string();
        }
        let entries = cassettes
            .iter()
            .enumerate()
            .map(|(idx, cassette)| {
                format!(
                    "#{idx} command={} args={:?} mode={:?} cwd={:?} env_keys={:?} remaining={}",
                    cassette.rule.command,
                    cassette.rule.args,
                    cassette.rule.args_mode,
                    cassette.rule.cwd,
                    cassette
                        .rule
                        .required_env
                        .keys()
                        .cloned()
                        .collect::<Vec<_>>(),
                    cassette
                        .rule
                        .responses
                        .len()
                        .saturating_sub(cassette.next_response)
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        format!("registered shell cassettes: [{entries}]")
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

fn shell_cassette_rule_matches(rule: &ShellCassetteRule, request: &ShellRequest) -> bool {
    if rule.command != request.command {
        return false;
    }
    if let Some(expected_cwd) = &rule.cwd {
        if request.cwd.as_deref() != Some(expected_cwd.as_str()) {
            return false;
        }
    }
    if let Some(expected_stdin) = &rule.stdin {
        if request.stdin.as_deref() != Some(expected_stdin.as_str()) {
            return false;
        }
    }
    if !rule.required_env.iter().all(|(key, expected)| {
        request
            .env
            .get(key)
            .is_some_and(|actual| actual == expected)
    }) {
        return false;
    }
    shell_args_match(rule, &request.args)
}

fn shell_args_match(rule: &ShellCassetteRule, request_args: &[String]) -> bool {
    match rule.args_mode {
        ShellArgMatchMode::Any => true,
        ShellArgMatchMode::Exact => rule.args == request_args,
        ShellArgMatchMode::Glob => {
            if rule.args.len() != request_args.len() {
                return false;
            }
            rule.args.iter().zip(request_args).all(|(pattern, actual)| {
                match glob::Pattern::new(pattern) {
                    Ok(glob) => glob.matches(actual),
                    Err(_) => false,
                }
            })
        }
    }
}

fn http_stub_rule_matches(
    rule: &HttpStubRule,
    method: &HttpMethod,
    path: &str,
    headers: &HashMap<String, String>,
) -> bool {
    if let Some(expected) = rule.method {
        if expected != *method {
            return false;
        }
    }
    if !path_matches(path, &rule.path_pattern, rule.path_mode) {
        return false;
    }
    headers_match(headers, &rule.required_headers)
}

fn headers_match(headers: &HashMap<String, String>, required: &[(String, String)]) -> bool {
    required.iter().all(|(key, expected)| {
        headers
            .iter()
            .find(|(actual_key, _)| actual_key.eq_ignore_ascii_case(key))
            .is_some_and(|(_, actual_value)| actual_value == expected)
    })
}

fn path_matches(path: &str, pattern: &str, mode: HttpPathMatchMode) -> bool {
    match mode {
        HttpPathMatchMode::Prefix => path.starts_with(pattern),
        HttpPathMatchMode::Exact => path == pattern,
        HttpPathMatchMode::Template => path_template_matches(path, pattern),
    }
}

fn path_template_matches(path: &str, template: &str) -> bool {
    let path_segments: Vec<&str> = path.trim_matches('/').split('/').collect();
    let template_segments: Vec<&str> = template.trim_matches('/').split('/').collect();
    if path_segments.len() != template_segments.len() {
        return false;
    }
    path_segments
        .iter()
        .zip(template_segments)
        .all(|(actual, expected)| {
            if expected.starts_with('{') && expected.ends_with('}') && expected.len() > 2 {
                !actual.is_empty()
            } else {
                actual == &expected
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_url_path_strips_query_and_fragment() {
        assert_eq!(
            extract_url_path("https://api.example.com/v1/items"),
            "/v1/items"
        );
        assert_eq!(
            extract_url_path("https://api.example.com/v1/items?key=val"),
            "/v1/items"
        );
        assert_eq!(
            extract_url_path("https://api.example.com/v1/items#section"),
            "/v1/items"
        );
        assert_eq!(
            extract_url_path("https://api.example.com/v1/items?key=val&b=2#frag"),
            "/v1/items"
        );
        assert_eq!(extract_url_path("https://api.example.com/"), "/");
        assert_eq!(extract_url_path("https://api.example.com"), "/");
    }

    #[test]
    fn execute_rest_errors_on_invalid_stub_json() {
        let backend = VirtualTransportBackend::new();
        backend.add_http_stub(HttpStub {
            method: Some(HttpMethod::Get),
            path_pattern: "/test".to_string(),
            exact_path: true,
            status: 200,
            response_body: "not valid json{".to_string(),
            response_headers: Default::default(),
        });
        let request = RestRequest {
            url: "https://api.example.com/test".to_string(),
            method: HttpMethod::Get,
            headers: Default::default(),
            query: Default::default(),
            body: None,
            auth: None,
            timeout_ms: None,
            requires_auth: false,
        };
        let result = backend.execute_rest(&request);
        assert!(
            result.is_err(),
            "invalid JSON in stub should produce an error"
        );
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("invalid JSON"),
            "error should mention invalid JSON, got: {err_msg}"
        );
    }

    #[test]
    fn execute_rest_empty_body_returns_empty_json_object() {
        let backend = VirtualTransportBackend::new();
        backend.add_http_stub(HttpStub {
            method: Some(HttpMethod::Get),
            path_pattern: "/test".to_string(),
            exact_path: true,
            status: 200,
            response_body: String::new(),
            response_headers: Default::default(),
        });
        let request = RestRequest {
            url: "https://api.example.com/test".to_string(),
            method: HttpMethod::Get,
            headers: Default::default(),
            query: Default::default(),
            body: None,
            auth: None,
            timeout_ms: None,
            requires_auth: false,
        };
        let result = backend
            .execute_rest(&request)
            .expect("empty body should succeed");
        assert_eq!(result.body, serde_json::json!({}));
    }

    #[test]
    fn template_path_matching_supports_parameter_segments() {
        assert!(path_template_matches(
            "/repos/octo/repo/issues",
            "/repos/{owner}/{repo}/issues"
        ));
        assert!(!path_template_matches(
            "/repos/octo/repo/pulls",
            "/repos/{owner}/{repo}/issues"
        ));
    }

    #[test]
    fn shell_glob_arg_matching_uses_glob_patterns() {
        let rule = ShellCassetteRule {
            command: "cargo".to_string(),
            args: vec!["test".to_string(), "--package".to_string(), "*".to_string()],
            args_mode: ShellArgMatchMode::Glob,
            cwd: None,
            required_env: HashMap::new(),
            stdin: None,
            responses: vec![ShellResponse::ok("ok")],
        };
        let request = ShellRequest::new("cargo")
            .arg("test")
            .arg("--package")
            .arg("gunbc-lib-transport");
        assert!(shell_cassette_rule_matches(&rule, &request));
    }

    #[test]
    fn header_matching_is_case_insensitive_on_key() {
        let mut headers = HashMap::new();
        headers.insert("X-GitHub-Api-Version".to_string(), "2022-11-28".to_string());
        assert!(headers_match(
            &headers,
            &[("x-github-api-version".to_string(), "2022-11-28".to_string())]
        ));
    }
}
