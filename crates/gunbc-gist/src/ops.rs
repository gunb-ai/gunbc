//! Gist operations.

use gunbc_exec::{ExecError, Executable};
use gunbc_ir::transport::{gist::GistRequest, ShellResponse, TransportResponse};
#[cfg(test)]
use gunbc_ir::transport::TransportRequest;
use gunbc_ir::Value;
use gunbc_transport::execute_transport;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Operations for the gist tool.
#[derive(Debug, Clone)]
pub enum GistOp {
    /// List files in a directory using git ls-files
    ListFiles,
    /// Filter files by extension
    FilterFiles { extensions: Vec<String> },
    /// Read file contents
    ReadFiles,
    /// Render files as markdown
    RenderMarkdown,
    /// Prepare a gist request (PURE - no I/O)
    PrepareGistRequest { public: bool },
    /// Execute a transport request (BOUNDARY - world write)
    ExecuteTransport,
}

impl Executable for GistOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GistOp::ListFiles => execute_list_files(inputs),
            GistOp::FilterFiles { extensions } => execute_filter_files(inputs, extensions),
            GistOp::ReadFiles => execute_read_files(inputs),
            GistOp::RenderMarkdown => execute_render_markdown(inputs),
            GistOp::PrepareGistRequest { public } => execute_prepare_gist_request(inputs, *public),
            GistOp::ExecuteTransport => execute_transport_op(inputs),
        }
    }
}

/// List files in a directory using git ls-files.
fn execute_list_files(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let repo_path = match inputs.get("repo_path") {
        Some(Value::Str(s)) => s.clone(),
        _ => ".".to_string(),
    };

    // Use git ls-files to respect .gitignore
    let output = Command::new("git")
        .current_dir(&repo_path)
        .args(["ls-files", "--cached", "--others", "--exclude-standard"])
        .output()
        .map_err(|e| ExecError::new(format!("failed to run git ls-files: {}", e)))?;

    if !output.status.success() {
        // Fallback to simple directory listing
        let files = list_files_recursive(Path::new(&repo_path))
            .map_err(|e| ExecError::new(format!("failed to list files: {}", e)))?;

        let mut out = HashMap::new();
        out.insert("files".to_string(), Value::StrList(files));
        return Ok(out);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files: Vec<String> = stdout
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();

    let mut out = HashMap::new();
    out.insert("files".to_string(), Value::StrList(files));
    Ok(out)
}

/// Simple recursive file listing fallback.
fn list_files_recursive(dir: &Path) -> Result<Vec<String>, std::io::Error> {
    let mut files = Vec::new();

    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip hidden directories
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }

            if path.is_dir() {
                files.extend(list_files_recursive(&path)?);
            } else if let Some(p) = path.to_str() {
                files.push(p.to_string());
            }
        }
    }

    Ok(files)
}

/// Filter files by extension.
fn execute_filter_files(
    inputs: HashMap<String, Value>,
    extensions: &[String],
) -> Result<HashMap<String, Value>, ExecError> {
    let files = match inputs.get("files") {
        Some(Value::StrList(list)) => list.clone(),
        _ => return Err(ExecError::new("missing or invalid 'files' input")),
    };

    let filtered: Vec<String> = if extensions.is_empty() {
        files
    } else {
        files
            .into_iter()
            .filter(|f| extensions.iter().any(|ext| f.ends_with(ext)))
            .collect()
    };

    let mut out = HashMap::new();
    out.insert("files".to_string(), Value::StrList(filtered));
    Ok(out)
}

/// Read file contents.
fn execute_read_files(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let files = match inputs.get("files") {
        Some(Value::StrList(list)) => list.clone(),
        _ => return Err(ExecError::new("missing or invalid 'files' input")),
    };

    let repo_path = match inputs.get("repo_path") {
        Some(Value::Str(s)) => s.clone(),
        _ => ".".to_string(),
    };

    let mut contents: BTreeMap<String, String> = BTreeMap::new();

    for file in &files {
        let path = Path::new(&repo_path).join(file);
        match fs::read_to_string(&path) {
            Ok(content) => {
                contents.insert(file.clone(), content);
            }
            Err(_) => {
                // Skip files that can't be read (binary, permissions, etc.)
            }
        }
    }

    let mut out = HashMap::new();
    out.insert("contents".to_string(), Value::MapStrStr(contents));
    Ok(out)
}

/// Render files as markdown.
fn execute_render_markdown(
    inputs: HashMap<String, Value>,
) -> Result<HashMap<String, Value>, ExecError> {
    let contents = match inputs.get("contents") {
        Some(Value::MapStrStr(map)) => map.clone(),
        _ => return Err(ExecError::new("missing or invalid 'contents' input")),
    };

    let mut markdown = String::new();
    markdown.push_str("# Code Snapshot\n\n");

    for (filename, content) in &contents {
        // Detect language from extension
        let lang = detect_language(filename);

        markdown.push_str(&format!("## `{}`\n\n", filename));
        markdown.push_str(&format!("```{}\n", lang));
        markdown.push_str(content);
        if !content.ends_with('\n') {
            markdown.push('\n');
        }
        markdown.push_str("```\n\n");
    }

    let mut out = HashMap::new();
    out.insert("markdown".to_string(), Value::Str(markdown));
    Ok(out)
}

/// Detect language from file extension.
fn detect_language(filename: &str) -> &'static str {
    if filename.ends_with(".rs") {
        "rust"
    } else if filename.ends_with(".py") {
        "python"
    } else if filename.ends_with(".js") {
        "javascript"
    } else if filename.ends_with(".ts") {
        "typescript"
    } else if filename.ends_with(".go") {
        "go"
    } else if filename.ends_with(".md") {
        "markdown"
    } else if filename.ends_with(".toml") {
        "toml"
    } else if filename.ends_with(".json") {
        "json"
    } else if filename.ends_with(".yaml") || filename.ends_with(".yml") {
        "yaml"
    } else if filename.ends_with(".sh") {
        "bash"
    } else {
        ""
    }
}

/// Prepare a gist request (PURE - just builds the request, no I/O).
fn execute_prepare_gist_request(
    inputs: HashMap<String, Value>,
    public: bool,
) -> Result<HashMap<String, Value>, ExecError> {
    let markdown = match inputs.get("markdown") {
        Some(Value::Str(s)) => s.clone(),
        _ => return Err(ExecError::new("missing or invalid 'markdown' input")),
    };

    // Build the gist request using the transport layer types
    let gist_request = GistRequest::new()
        .file("snapshot.md", markdown)
        .public(public)
        .description("Code snapshot created by gunbc-gist");

    // Convert to shell request (using gh CLI)
    let transport_request = gist_request.to_shell_request();

    let mut out = HashMap::new();
    out.insert("request".to_string(), Value::Request(transport_request));
    Ok(out)
}

/// Execute a transport request (BOUNDARY - world write).
fn execute_transport_op(inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
    let request = match inputs.get("request") {
        Some(Value::Request(r)) => r.clone(),
        _ => return Err(ExecError::new("missing or invalid 'request' input")),
    };

    let response = execute_transport(&request)
        .map_err(|e| ExecError::new(format!("transport error: {}", e)))?;

    // Extract URL from response
    let url = match &response {
        TransportResponse::Shell(ShellResponse { stdout, .. }) => {
            gunbc_ir::transport::gist::parse_gist_url_from_shell(stdout)
                .unwrap_or_else(|| stdout.trim().to_string())
        }
        TransportResponse::Rest(r) => {
            gunbc_ir::transport::gist::parse_gist_url_from_rest(&r.body)
                .unwrap_or_else(|| "unknown".to_string())
        }
        _ => "unknown".to_string(),
    };

    let mut out = HashMap::new();
    out.insert("response".to_string(), Value::Response(response));
    out.insert("url".to_string(), Value::Str(url));
    Ok(out)
}

// ============================================================================
// Mockable trait implementation
// ============================================================================

use gunbc_ir::CardinalityCase;
use gunbc_test::{CardinalityTestInput, ErrorTestCase, Mockable};

impl Mockable for GistOp {
    fn mock_outputs(&self) -> HashMap<String, Value> {
        match self {
            GistOp::ListFiles => {
                let mut out = HashMap::new();
                out.insert(
                    "files".to_string(),
                    Value::StrList(vec![
                        "src/lib.rs".to_string(),
                        "src/main.rs".to_string(),
                        "Cargo.toml".to_string(),
                    ]),
                );
                out
            }
            GistOp::FilterFiles { .. } => {
                let mut out = HashMap::new();
                out.insert(
                    "files".to_string(),
                    Value::StrList(vec!["src/lib.rs".to_string(), "src/main.rs".to_string()]),
                );
                out
            }
            GistOp::ReadFiles => {
                let mut contents = BTreeMap::new();
                contents.insert("src/lib.rs".to_string(), "// lib code\n".to_string());
                contents.insert("src/main.rs".to_string(), "fn main() {}\n".to_string());
                let mut out = HashMap::new();
                out.insert("contents".to_string(), Value::MapStrStr(contents));
                out
            }
            GistOp::RenderMarkdown => {
                let mut out = HashMap::new();
                out.insert(
                    "markdown".to_string(),
                    Value::Str("# Code Snapshot\n\n```rust\nfn main() {}\n```\n".to_string()),
                );
                out
            }
            GistOp::PrepareGistRequest { public } => {
                let request = GistRequest::new()
                    .file("snapshot.md", "# Mock Gist")
                    .public(*public)
                    .description("Mock gist request")
                    .to_shell_request();
                let mut out = HashMap::new();
                out.insert("request".to_string(), Value::Request(request));
                out
            }
            GistOp::ExecuteTransport => {
                let mut out = HashMap::new();
                out.insert(
                    "response".to_string(),
                    Value::Response(TransportResponse::Shell(ShellResponse {
                        exit_code: 0,
                        stdout: "https://gist.github.com/mock123".to_string(),
                        stderr: String::new(),
                    })),
                );
                out.insert(
                    "url".to_string(),
                    Value::Str("https://gist.github.com/mock123".to_string()),
                );
                out
            }
        }
    }

    fn cardinality_inputs(&self) -> Vec<CardinalityTestInput> {
        match self {
            GistOp::ListFiles => vec![
                // repo_path is optional, defaults to "."
            ],
            GistOp::FilterFiles { .. } => vec![
                CardinalityTestInput::succeeds(
                    "files",
                    CardinalityCase::Empty,
                    Value::StrList(vec![]),
                ),
                CardinalityTestInput::succeeds(
                    "files",
                    CardinalityCase::One,
                    Value::StrList(vec!["single.rs".to_string()]),
                ),
                CardinalityTestInput::succeeds(
                    "files",
                    CardinalityCase::Many,
                    Value::StrList(vec![
                        "a.rs".to_string(),
                        "b.rs".to_string(),
                        "c.rs".to_string(),
                    ]),
                ),
            ],
            GistOp::ReadFiles => vec![
                CardinalityTestInput::succeeds(
                    "files",
                    CardinalityCase::Empty,
                    Value::StrList(vec![]),
                ),
                CardinalityTestInput::succeeds(
                    "files",
                    CardinalityCase::One,
                    Value::StrList(vec!["test.rs".to_string()]),
                ),
            ],
            GistOp::RenderMarkdown => vec![
                CardinalityTestInput::succeeds(
                    "contents",
                    CardinalityCase::Empty,
                    Value::MapStrStr(BTreeMap::new()),
                ),
                CardinalityTestInput::succeeds(
                    "contents",
                    CardinalityCase::One,
                    Value::MapStrStr({
                        let mut m = BTreeMap::new();
                        m.insert("test.rs".to_string(), "fn main() {}".to_string());
                        m
                    }),
                ),
            ],
            _ => vec![],
        }
    }

    fn error_cases(&self) -> Vec<ErrorTestCase> {
        match self {
            GistOp::FilterFiles { .. } => vec![
                ErrorTestCase::new(
                    "missing_files_input",
                    HashMap::new(),
                    "missing or invalid 'files' input",
                ),
                ErrorTestCase::new(
                    "wrong_type_files_input",
                    {
                        let mut m = HashMap::new();
                        m.insert("files".to_string(), Value::Str("not a list".to_string()));
                        m
                    },
                    "missing or invalid 'files' input",
                ),
            ],
            GistOp::ReadFiles => vec![ErrorTestCase::new(
                "missing_files_input",
                HashMap::new(),
                "missing or invalid 'files' input",
            )],
            GistOp::RenderMarkdown => vec![ErrorTestCase::new(
                "missing_contents_input",
                HashMap::new(),
                "missing or invalid 'contents' input",
            )],
            GistOp::PrepareGistRequest { .. } => vec![ErrorTestCase::new(
                "missing_markdown_input",
                HashMap::new(),
                "missing or invalid 'markdown' input",
            )],
            GistOp::ExecuteTransport => vec![ErrorTestCase::new(
                "missing_request_input",
                HashMap::new(),
                "missing or invalid 'request' input",
            )],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_files_by_extension() {
        let mut inputs = HashMap::new();
        inputs.insert(
            "files".to_string(),
            Value::StrList(vec![
                "foo.rs".to_string(),
                "bar.py".to_string(),
                "baz.rs".to_string(),
                "README.md".to_string(),
            ]),
        );

        let result = execute_filter_files(inputs, &["rs".to_string()]).unwrap();

        match result.get("files") {
            Some(Value::StrList(files)) => {
                assert_eq!(files.len(), 2);
                assert!(files.contains(&"foo.rs".to_string()));
                assert!(files.contains(&"baz.rs".to_string()));
            }
            _ => panic!("expected file list"),
        }
    }

    #[test]
    fn test_render_markdown() {
        let mut contents = BTreeMap::new();
        contents.insert("test.rs".to_string(), "fn main() {}".to_string());

        let mut inputs = HashMap::new();
        inputs.insert("contents".to_string(), Value::MapStrStr(contents));

        let result = execute_render_markdown(inputs).unwrap();

        match result.get("markdown") {
            Some(Value::Str(md)) => {
                assert!(md.contains("# Code Snapshot"));
                assert!(md.contains("## `test.rs`"));
                assert!(md.contains("```rust"));
                assert!(md.contains("fn main() {}"));
            }
            _ => panic!("expected markdown"),
        }
    }

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language("foo.rs"), "rust");
        assert_eq!(detect_language("bar.py"), "python");
        assert_eq!(detect_language("unknown.xyz"), "");
    }

    #[test]
    fn test_prepare_gist_request() {
        let mut inputs = HashMap::new();
        inputs.insert("markdown".to_string(), Value::Str("# Test".to_string()));

        let result = execute_prepare_gist_request(inputs, false).unwrap();

        match result.get("request") {
            Some(Value::Request(TransportRequest::Shell(req))) => {
                assert_eq!(req.command, "gh");
                assert!(req.args.contains(&"gist".to_string()));
            }
            _ => panic!("expected shell request"),
        }
    }
}
