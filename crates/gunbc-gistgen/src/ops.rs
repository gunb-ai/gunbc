use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::process::Command;

use gunbc_exec::{ExecError, Executable, Value};
use gunbc_ir::transport::gist::GistOp;
use serde_json::json;

/// Core operation type for gistgen nodes.
#[derive(Debug, Clone)]
pub enum GistgenCoreOp {
    Context {
        repo_path: String,
        glob_pattern: String,
    },
    AuthCheck,
    AuthCreate,
    AuthResolve,
    EnumerateFiles,
    FilterFiles,
    ReadFiles,
    ComposeSnapshot,
    /// Convert a snapshot string into a single-file gist map.
    WrapSingleGistFile,
    /// Convert a map of file contents into a gist files map.
    ComposeGistFiles,
    /// Build a Create Gist request JSON string.
    BuildGistCreateRequest,
}

/// Union op type used by gistgen DAGs.
#[derive(Debug, Clone)]
pub enum GistgenOp {
    Core(GistgenCoreOp),
    Gist(GistOp),
}

impl Executable for GistgenCoreOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GistgenCoreOp::Context { repo_path, glob_pattern } => {
                let mut out = HashMap::new();
                let abs_path = std::fs::canonicalize(repo_path)
                    .unwrap_or_else(|_| std::path::PathBuf::from(repo_path));
                out.insert("repo".into(), Value::Str(abs_path.to_string_lossy().into_owned()));
                out.insert("selection_spec".into(), Value::Str(glob_pattern.clone()));
                Ok(out)
            }

            GistgenCoreOp::AuthCheck => {
                let mut out = HashMap::new();
                // Try GITHUB_TOKEN env var first
                if let Ok(token) = std::env::var("GITHUB_TOKEN") {
                    if !token.is_empty() {
                        out.insert("token".into(), Value::Secret(gunbc_ir::Secret(token)));
                        out.insert("needs_create".into(), Value::Bool(false));
                        return Ok(out);
                    }
                }
                // Fall back to `gh auth token`
                match Command::new("gh").args(["auth", "token"]).output() {
                    Ok(output) if output.status.success() => {
                        let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if !token.is_empty() {
                            out.insert("token".into(), Value::Secret(gunbc_ir::Secret(token)));
                            out.insert("needs_create".into(), Value::Bool(false));
                            return Ok(out);
                        }
                    }
                    _ => {}
                }
                // No token found
                out.insert("token".into(), Value::Skipped);
                out.insert("needs_create".into(), Value::Bool(true));
                Ok(out)
            }

            GistgenCoreOp::AuthCreate => {
                eprintln!("No GitHub token found. Please authenticate with `gh auth login`.");
                let login_status = Command::new("gh")
                    .args(["auth", "login"])
                    .status()
                    .map_err(|e| ExecError(format!("Failed to run gh auth login: {e}")))?;

                if !login_status.success() {
                    return Err(ExecError("gh auth login failed".into()));
                }

                let output = Command::new("gh")
                    .args(["auth", "token"])
                    .output()
                    .map_err(|e| ExecError(format!("Failed to run gh auth token: {e}")))?;

                if !output.status.success() {
                    return Err(ExecError("gh auth token failed after login".into()));
                }

                let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let mut out = HashMap::new();
                out.insert("token".into(), Value::Secret(gunbc_ir::Secret(token)));
                Ok(out)
            }

            GistgenCoreOp::AuthResolve => {
                let token = inputs
                    .get("check_token")
                    .filter(|v| !matches!(v, Value::Skipped))
                    .or_else(|| inputs.get("create_token").filter(|v| !matches!(v, Value::Skipped)))
                    .cloned()
                    .unwrap_or(Value::Unit);
                let mut out = HashMap::new();
                out.insert("token".into(), token);
                Ok(out)
            }

            GistgenCoreOp::EnumerateFiles => {
                let repo = inputs
                    .get("repo")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| ".".into());

                let repo_path = Path::new(&repo);
                let mut files = match git_ls_files(&repo) {
                    Ok(files) => files,
                    Err(_) => {
                        let mut fallback = Vec::new();
                        let walker = ignore::WalkBuilder::new(&repo)
                            .hidden(false)
                            .git_ignore(true)
                            .git_global(true)
                            .git_exclude(true)
                            .build();

                        for entry in walker {
                            match entry {
                                Ok(e)
                                    if e.file_type().map_or(false, |ft| ft.is_file())
                                        && !is_git_dir(e.path()) =>
                                {
                                    fallback.push(normalize_repo_relative(repo_path, e.path()));
                                }
                                _ => {}
                            }
                        }
                        fallback
                    }
                };
                files.sort();
                let mut out = HashMap::new();
                out.insert("files".into(), Value::StrList(files));
                Ok(out)
            }

            GistgenCoreOp::FilterFiles => {
                let file_list = inputs
                    .get("files")
                    .and_then(|v| if let Value::StrList(v) = v { Some(v.clone()) } else { None })
                    .unwrap_or_default();
                let spec = inputs
                    .get("selection_spec")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| "**/*".into());

                let glob = globset::Glob::new(&spec)
                    .map_err(|e| ExecError(format!("Invalid glob pattern '{spec}': {e}")))?
                    .compile_matcher();

                let filtered: Vec<String> = file_list
                    .into_iter()
                    .filter(|line| {
                        let p = Path::new(line);
                        glob.is_match(p) || glob.is_match(p.file_name().unwrap_or_default())
                    })
                    .collect();

                let mut out = HashMap::new();
                out.insert("files".into(), Value::StrList(filtered));
                Ok(out)
            }

            GistgenCoreOp::ReadFiles => {
                let file_list = inputs
                    .get("files")
                    .and_then(|v| if let Value::StrList(v) = v { Some(v.clone()) } else { None })
                    .unwrap_or_default();
                let repo = inputs
                    .get("repo")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| ".".into());
                let repo_path = Path::new(&repo);

                let mut contents = BTreeMap::new();
                for path in &file_list {
                    if path.is_empty() {
                        continue;
                    }
                    let file_path = Path::new(path);
                    let (read_path, key) = if file_path.is_absolute() {
                        let key = file_path
                            .strip_prefix(repo_path)
                            .unwrap_or(file_path)
                            .to_string_lossy()
                            .into_owned();
                        (file_path.to_path_buf(), key)
                    } else {
                        (repo_path.join(file_path), path.clone())
                    };

                    match std::fs::read_to_string(&read_path) {
                        Ok(content) => {
                            contents.insert(key, content);
                        }
                        Err(e) => {
                            contents.insert(key, format!("[error reading file: {e}]"));
                        }
                    }
                }
                let mut out = HashMap::new();
                out.insert("contents".into(), Value::MapStrStr(contents));
                Ok(out)
            }

            GistgenCoreOp::ComposeSnapshot => {
                let contents = inputs
                    .get("contents")
                    .and_then(|v| if let Value::MapStrStr(m) = v { Some(m.clone()) } else { None })
                    .unwrap_or_default();

                let mut snapshot = String::from("# Gist Snapshot\n\n");
                for (path, content) in &contents {
                    snapshot.push_str(&format!("--- {path} ---\n{content}\n\n"));
                }
                let mut out = HashMap::new();
                out.insert("snapshot".into(), Value::Str(snapshot));
                Ok(out)
            }

            GistgenCoreOp::WrapSingleGistFile => {
                let snapshot = inputs
                    .get("snapshot")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_default();

                let mut files = BTreeMap::new();
                files.insert("snapshot.md".into(), snapshot);
                let mut out = HashMap::new();
                out.insert("files".into(), Value::MapStrStr(files));
                Ok(out)
            }

            GistgenCoreOp::ComposeGistFiles => {
                let contents = inputs
                    .get("contents")
                    .and_then(|v| if let Value::MapStrStr(m) = v { Some(m.clone()) } else { None })
                    .unwrap_or_default();

                let mut files = BTreeMap::new();
                for (path, content) in contents {
                    let name = Path::new(&path)
                        .file_name()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or(path);
                    files.insert(name, content);
                }

                let mut out = HashMap::new();
                out.insert("files".into(), Value::MapStrStr(files));
                Ok(out)
            }

            GistgenCoreOp::BuildGistCreateRequest => {
                let files = inputs
                    .get("files")
                    .and_then(|v| if let Value::MapStrStr(m) = v { Some(m.clone()) } else { None })
                    .unwrap_or_default();

                let files_json: serde_json::Map<String, serde_json::Value> = files
                    .into_iter()
                    .map(|(name, content)| (name, json!({ "content": content })))
                    .collect();

                let request = json!({
                    "description": "gistgen snapshot",
                    "public": false,
                    "files": files_json,
                });

                let request_json = serde_json::to_string(&request)
                    .map_err(|e| ExecError(format!("failed to serialize gist request: {e}")))?;

                let mut out = HashMap::new();
                out.insert("request".into(), Value::Str(request_json));
                Ok(out)
            }
        }
    }
}

impl Executable for GistgenOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GistgenOp::Core(op) => op.execute(inputs),
            GistgenOp::Gist(op) => execute_gist_op(op, inputs),
        }
    }
}

fn execute_gist_op(op: &GistOp, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    match op {
        GistOp::FormatCreateRequest => {
            let request_json = get_str(&inputs, "request")?;
            let value: serde_json::Value = serde_json::from_str(&request_json)
                .map_err(|e| ExecError(format!("invalid gist request JSON: {e}")))?;
            validate_gist_request(&value)?;

            let mut out = HashMap::new();
            out.insert("request_json".into(), Value::Str(request_json));
            Ok(out)
        }

        GistOp::CallMock => {
            let request_json = get_str(&inputs, "request_json")?;
            let value: serde_json::Value = serde_json::from_str(&request_json)
                .map_err(|e| ExecError(format!("invalid gist request JSON: {e}")))?;
            let (files, description) = extract_files_and_description(&value)?;

            let id = format!("{:x}", fnv1a_hash(&request_json));
            let html_url = format!("https://gist.github.com/mock/{id}");
            let api_url = format!("https://api.github.com/gists/{id}");

            let files_json: serde_json::Map<String, serde_json::Value> = files
                .iter()
                .map(|(name, content)| {
                    let raw_url = format!("https://gist.github.com/mock/{id}/raw/{name}");
                    let meta = json!({
                        "filename": name,
                        "type": "text/plain",
                        "language": serde_json::Value::Null,
                        "raw_url": raw_url,
                        "size": content.len(),
                        "truncated": false,
                        "content": content,
                        "encoding": "utf-8",
                    });
                    (name.clone(), meta)
                })
                .collect();

            let response = json!({
                "id": id,
                "html_url": html_url,
                "url": api_url,
                "files": files_json,
                "public": false,
                "description": description,
                "truncated": false,
            });

            let response_json = serde_json::to_string(&response)
                .map_err(|e| ExecError(format!("failed to serialize mock gist response: {e}")))?;

            let mut out = HashMap::new();
            out.insert("response_json".into(), Value::Str(response_json));
            Ok(out)
        }

        GistOp::CallReal => {
            let request_json = get_str(&inputs, "request_json")?;
            let value: serde_json::Value = serde_json::from_str(&request_json)
                .map_err(|e| ExecError(format!("invalid gist request JSON: {e}")))?;
            let (files, _description) = extract_files_and_description(&value)?;

            let token = match inputs.get("token") {
                Some(Value::Secret(s)) => s.as_inner().clone(),
                _ => return Err(ExecError("missing or invalid token for gist upload".into())),
            };

            let dir = tempfile::tempdir()
                .map_err(|e| ExecError(format!("failed to create temp dir: {e}")))?;
            let mut file_paths = Vec::new();
            for (name, content) in &files {
                if name.contains('/') || name.contains('\\') {
                    return Err(ExecError(format!("invalid gist filename '{name}'")));
                }
                let path = dir.path().join(name);
                std::fs::write(&path, content)
                    .map_err(|e| ExecError(format!("failed to write temp file '{name}': {e}")))?;
                file_paths.push(path);
            }

            let mut cmd = Command::new("gh");
            cmd.arg("gist").arg("create");
            for path in &file_paths {
                cmd.arg(path);
            }
            cmd.env("GH_TOKEN", &token);
            let output = cmd
                .output()
                .map_err(|e| ExecError(format!("Failed to run gh gist create: {e}")))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ExecError(format!("gh gist create failed: {stderr}")));
            }

            let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let response = json!({
                "id": "",
                "html_url": url,
                "url": "",
                "files": {},
                "public": false,
                "description": serde_json::Value::Null,
                "truncated": false,
            });
            let response_json = serde_json::to_string(&response)
                .map_err(|e| ExecError(format!("failed to serialize gist response: {e}")))?;

            let mut out = HashMap::new();
            out.insert("response_json".into(), Value::Str(response_json));
            Ok(out)
        }

        GistOp::ParseCreateResponse => {
            let response_json = get_str(&inputs, "response_json")?;
            serde_json::from_str::<serde_json::Value>(&response_json)
                .map_err(|e| ExecError(format!("invalid gist response JSON: {e}")))?;

            let mut out = HashMap::new();
            out.insert("response".into(), Value::Str(response_json));
            Ok(out)
        }

        GistOp::ExtractGistUrl => {
            let response_json = get_str(&inputs, "response")?;
            let value: serde_json::Value = serde_json::from_str(&response_json)
                .map_err(|e| ExecError(format!("invalid gist response JSON: {e}")))?;
            let url = value
                .get("html_url")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ExecError("missing html_url in gist response".into()))?;

            let mut out = HashMap::new();
            out.insert("response".into(), Value::Str(response_json));
            out.insert("gist_url".into(), Value::Str(url.to_string()));
            Ok(out)
        }
    }
}

fn get_str(inputs: &HashMap<String, Value>, key: &str) -> Result<String, ExecError> {
    inputs
        .get(key)
        .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
        .ok_or_else(|| ExecError(format!("missing or invalid '{key}'")))
}

fn validate_gist_request(value: &serde_json::Value) -> Result<(), ExecError> {
    let files = value.get("files").and_then(|v| v.as_object())
        .ok_or_else(|| ExecError("gist request missing 'files' map".into()))?;
    if files.is_empty() {
        return Err(ExecError("gist request 'files' map is empty".into()));
    }
    for (name, file) in files {
        if is_reserved_gistfile(name) {
            return Err(ExecError(format!("gist filename '{name}' is reserved")));
        }
        let content = file.get("content").and_then(|v| v.as_str());
        if content.is_none() {
            return Err(ExecError(format!("gist file '{name}' missing content")));
        }
    }
    Ok(())
}

fn extract_files_and_description(
    value: &serde_json::Value,
) -> Result<(Vec<(String, String)>, serde_json::Value), ExecError> {
    let files_value = value.get("files").and_then(|v| v.as_object())
        .ok_or_else(|| ExecError("gist request missing 'files' map".into()))?;
    if files_value.is_empty() {
        return Err(ExecError("gist request 'files' map is empty".into()));
    }

    let mut files = Vec::new();
    for (name, file) in files_value {
        if is_reserved_gistfile(name) {
            return Err(ExecError(format!("gist filename '{name}' is reserved")));
        }
        let content = file
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExecError(format!("gist file '{name}' missing content")))?;
        files.push((name.clone(), content.to_string()));
    }

    let description = value.get("description")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    Ok((files, description))
}

fn is_reserved_gistfile(name: &str) -> bool {
    if let Some(rest) = name.strip_prefix("gistfile") {
        return !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit());
    }
    false
}

fn fnv1a_hash(input: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in input.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn git_ls_files(repo: &str) -> Result<Vec<String>, ExecError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["ls-files", "-co", "--exclude-standard", "-z"])
        .output()
        .map_err(|e| ExecError(format!("git ls-files failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ExecError(format!("git ls-files failed: {stderr}")));
    }

    let mut files = Vec::new();
    for entry in output.stdout.split(|b| *b == 0) {
        if entry.is_empty() {
            continue;
        }
        files.push(String::from_utf8_lossy(entry).to_string());
    }
    Ok(files)
}

fn is_git_dir(path: &Path) -> bool {
    path.components()
        .any(|c| c.as_os_str() == std::ffi::OsStr::new(".git"))
}

fn normalize_repo_relative(repo_path: &Path, path: &Path) -> String {
    path.strip_prefix(repo_path)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}
