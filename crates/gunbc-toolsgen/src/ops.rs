use std::collections::HashMap;
use std::path::Path;

use gunbc_exec::{ExecError, Executable, Value};
use sha2::{Digest, Sha256};

use crate::types::{ToolsgenConfig, UpsertStatus};

/// Static body for the cargo wrapper C source.
const CARGO_WRAPPER_BODY: &str = r#"#include <errno.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#ifdef _WIN32
#include <process.h>

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: cargo_wrapper <args>\n");
        return 2;
    }

    char **args = (char **)calloc((size_t)argc + 1, sizeof(char *));
    if (!args) {
        fprintf(stderr, "cargo_wrapper: allocation failed\n");
        return 2;
    }

    args[0] = "cargo";
    for (int i = 1; i < argc; i++) {
        args[i] = argv[i];
    }
    args[argc] = NULL;

    int rc = _spawnvp(_P_WAIT, "cargo", (const char *const *)args);
    if (rc == -1) {
        fprintf(stderr, "cargo_wrapper: failed to launch cargo (%d)\n", errno);
        return 127;
    }

    return rc;
}
#else
#include <unistd.h>

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "usage: cargo_wrapper <args>\n");
        return 2;
    }

    char **args = (char **)calloc((size_t)argc + 1, sizeof(char *));
    if (!args) {
        fprintf(stderr, "cargo_wrapper: allocation failed\n");
        return 2;
    }

    args[0] = "cargo";
    for (int i = 1; i < argc; i++) {
        args[i] = argv[i];
    }
    args[argc] = NULL;

    execvp("cargo", args);
    fprintf(stderr, "cargo_wrapper: failed to exec cargo (%s)\n", strerror(errno));
    return 127;
}
#endif
"#;

/// The operation type for toolsgen nodes.
#[derive(Debug, Clone)]
pub enum ToolsgenOp {
    /// Initialize context from config
    Context { config: ToolsgenConfig },
    /// Check existing file state
    Check,
    /// Compose the cargo wrapper source
    ComposeCargoWrapper,
    /// Resolve upsert state from check + generated content
    Resolve,
    /// Write content to file
    WriteFile,
    /// Print content to stdout (dry-run)
    PrintStdout,
}

impl Executable for ToolsgenOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            ToolsgenOp::Context { config } => {
                let abs_path = std::fs::canonicalize(&config.workspace_path)
                    .unwrap_or_else(|_| std::path::PathBuf::from(&config.workspace_path));
                let mut out = HashMap::new();
                out.insert(
                    "workspace_path".into(),
                    Value::Str(abs_path.to_string_lossy().into_owned()),
                );
                out.insert("output_path".into(), Value::Str(config.output_path.clone()));
                out.insert("force".into(), Value::Bool(config.force));
                Ok(out)
            }

            ToolsgenOp::Check => {
                let workspace_path = inputs
                    .get("workspace_path")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| ".".into());
                let output_path = inputs
                    .get("output_path")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| "tools/cargo_wrapper.c".into());
                let force = inputs
                    .get("force")
                    .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);

                let input_hash = compute_hash(CARGO_WRAPPER_BODY);

                let file_path = Path::new(&workspace_path).join(&output_path);
                let existing_hash = if file_path.exists() {
                    std::fs::read_to_string(&file_path)
                        .ok()
                        .and_then(|content| extract_hash(&content))
                } else {
                    None
                };

                let mut out = HashMap::new();
                out.insert("input_hash".into(), Value::Str(input_hash.clone()));
                out.insert(
                    "file_path".into(),
                    Value::Str(file_path.to_string_lossy().into_owned()),
                );

                let needs_write = force || existing_hash.as_ref() != Some(&input_hash);
                out.insert("needs_write".into(), Value::Bool(needs_write));
                out.insert("file_existed".into(), Value::Bool(file_path.exists()));

                Ok(out)
            }

            ToolsgenOp::ComposeCargoWrapper => {
                let hash = inputs
                    .get("input_hash")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| "unknown".into());

                let content = cargo_wrapper_content(&hash);

                let mut out = HashMap::new();
                out.insert("content".into(), Value::Str(content));
                Ok(out)
            }

            ToolsgenOp::Resolve => {
                let needs_write = inputs
                    .get("needs_write")
                    .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);
                let write_status = inputs.get("write_status");

                let status = if !needs_write {
                    UpsertStatus::Unchanged.to_string()
                } else {
                    match write_status {
                        Some(Value::Str(s)) => s.clone(),
                        Some(Value::Skipped) => {
                            return Err(ExecError(
                                "resolve expected write_status but write node was skipped".into(),
                            ))
                        }
                        _ => {
                            return Err(ExecError("resolve missing write_status input".into()))
                        }
                    }
                };

                let mut out = HashMap::new();
                out.insert("status".into(), Value::Str(status));
                Ok(out)
            }

            ToolsgenOp::WriteFile => {
                let content = inputs
                    .get("content")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None });
                let needs_write = inputs
                    .get("needs_write")
                    .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);
                let file_path = inputs
                    .get("file_path")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| "tools/cargo_wrapper.c".into());
                let file_existed = inputs
                    .get("file_existed")
                    .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);

                let mut out = HashMap::new();

                if !needs_write {
                    out.insert(
                        "write_status".into(),
                        Value::Str(UpsertStatus::Unchanged.to_string()),
                    );
                    return Ok(out);
                }

                match content {
                    Some(c) => {
                        std::fs::write(&file_path, &c).map_err(|e| {
                            ExecError(format!("Failed to write cargo wrapper: {}", e))
                        })?;
                        let status = if file_existed {
                            UpsertStatus::Updated
                        } else {
                            UpsertStatus::Created
                        };
                        out.insert("write_status".into(), Value::Str(status.to_string()));
                    }
                    None => {
                        out.insert(
                            "write_status".into(),
                            Value::Str(UpsertStatus::Unchanged.to_string()),
                        );
                    }
                }

                Ok(out)
            }

            ToolsgenOp::PrintStdout => {
                let content = inputs
                    .get("content")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None });
                let needs_write = inputs
                    .get("needs_write")
                    .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
                    .unwrap_or(false);

                let mut out = HashMap::new();

                if !needs_write {
                    out.insert(
                        "write_status".into(),
                        Value::Str(UpsertStatus::Unchanged.to_string()),
                    );
                    return Ok(out);
                }

                match content {
                    Some(c) => {
                        println!("{}", c);
                        out.insert(
                            "write_status".into(),
                            Value::Str(UpsertStatus::DryRun.to_string()),
                        );
                    }
                    None => {
                        out.insert(
                            "write_status".into(),
                            Value::Str(UpsertStatus::Unchanged.to_string()),
                        );
                    }
                }

                Ok(out)
            }
        }
    }
}

pub fn cargo_wrapper_content(hash: &str) -> String {
    format!("// Generated by gunbc-toolsgen\n// Hash: {hash}\n\n{CARGO_WRAPPER_BODY}")
}

pub fn compute_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..16].to_string()
}

pub fn extract_hash(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("// Hash: ") {
            return Some(line[9..].trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_wrapper_includes_hash() {
        let content = cargo_wrapper_content("abc123");
        assert!(content.contains("// Hash: abc123"));
        assert!(content.contains("cargo_wrapper"));
    }

    #[test]
    fn extract_hash_finds_hash() {
        let content = "// Generated by gunbc-toolsgen\n// Hash: abc123\n";
        assert_eq!(extract_hash(content), Some("abc123".into()));
    }

    #[test]
    fn compute_hash_is_deterministic() {
        let h1 = compute_hash(CARGO_WRAPPER_BODY);
        let h2 = compute_hash(CARGO_WRAPPER_BODY);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 16);
    }
}
