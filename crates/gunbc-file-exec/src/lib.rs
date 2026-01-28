use std::collections::HashMap;
use std::path::Path;

use gunbc_exec::{ExecError, Executable, Value};
use gunbc_ir::transport::file::FileOp;
use sha2::{Digest, Sha256};

const STATUS_CREATED: &str = "Created";
const STATUS_UPDATED: &str = "Updated";
const STATUS_UNCHANGED: &str = "Unchanged";
const STATUS_DRY_RUN: &str = "DryRun";

impl Executable for FileOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            FileOp::CheckExisting => {
                let file_path = require_str(&inputs, "file_path")?;
                let input_hash = require_str(&inputs, "input_hash")?;
                let force = bool_or(&inputs, "force", false);

                let path = Path::new(&file_path);
                let existing_hash = if path.exists() {
                    std::fs::read_to_string(path).ok().and_then(|content| extract_hash(&content))
                } else {
                    None
                };

                let needs_write = force || existing_hash.as_ref() != Some(&input_hash);

                let mut out = HashMap::new();
                out.insert("file_path".into(), Value::Str(file_path));
                out.insert("input_hash".into(), Value::Str(input_hash));
                out.insert("needs_write".into(), Value::Bool(needs_write));
                out.insert("file_existed".into(), Value::Bool(path.exists()));
                Ok(out)
            }
            FileOp::ResolveUpsert => {
                let needs_write = bool_or(&inputs, "needs_write", false);
                let write_status = inputs.get("write_status");

                let status = if !needs_write {
                    STATUS_UNCHANGED.to_string()
                } else {
                    match write_status {
                        Some(Value::Str(s)) => s.clone(),
                        Some(Value::Skipped) => {
                            return Err(ExecError(
                                "resolve expected write_status but write node was skipped".into(),
                            ))
                        }
                        _ => {
                            return Err(ExecError(
                                "resolve missing write_status input".into(),
                            ))
                        }
                    }
                };

                let mut out = HashMap::new();
                out.insert("status".into(), Value::Str(status));
                Ok(out)
            }
            FileOp::WriteFile => {
                let needs_write = bool_or(&inputs, "needs_write", false);
                let content = optional_str(&inputs, "content");
                let file_path = require_str(&inputs, "file_path")?;
                let file_existed = bool_or(&inputs, "file_existed", false);

                let mut out = HashMap::new();

                if !needs_write {
                    out.insert("write_status".into(), Value::Str(STATUS_UNCHANGED.to_string()));
                    return Ok(out);
                }

                match content {
                    Some(c) => {
                        std::fs::write(&file_path, &c)
                            .map_err(|e| ExecError(format!("Failed to write file: {}", e)))?;
                        let status = if file_existed {
                            STATUS_UPDATED
                        } else {
                            STATUS_CREATED
                        };
                        out.insert("write_status".into(), Value::Str(status.to_string()));
                    }
                    None => {
                        out.insert("write_status".into(), Value::Str(STATUS_UNCHANGED.to_string()));
                    }
                }

                Ok(out)
            }
            FileOp::PrintStdout => {
                let needs_write = bool_or(&inputs, "needs_write", false);
                let content = optional_str(&inputs, "content");

                let mut out = HashMap::new();

                if !needs_write {
                    out.insert("write_status".into(), Value::Str(STATUS_UNCHANGED.to_string()));
                    return Ok(out);
                }

                match content {
                    Some(c) => {
                        println!("{}", c);
                        out.insert("write_status".into(), Value::Str(STATUS_DRY_RUN.to_string()));
                    }
                    None => {
                        out.insert("write_status".into(), Value::Str(STATUS_UNCHANGED.to_string()));
                    }
                }

                Ok(out)
            }
        }
    }
}

pub fn compute_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)[..16].to_string()
}

pub fn extract_hash(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with("# Hash: ") {
            return Some(line[8..].trim().to_string());
        }
    }
    None
}

fn require_str(inputs: &HashMap<String, Value>, key: &str) -> Result<String, ExecError> {
    match inputs.get(key) {
        Some(Value::Str(s)) => Ok(s.clone()),
        _ => Err(ExecError(format!("missing or invalid '{key}' input"))),
    }
}

fn optional_str(inputs: &HashMap<String, Value>, key: &str) -> Option<String> {
    inputs
        .get(key)
        .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
}

fn bool_or(inputs: &HashMap<String, Value>, key: &str, default: bool) -> bool {
    inputs
        .get(key)
        .and_then(|v| if let Value::Bool(b) = v { Some(*b) } else { None })
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_hash_is_deterministic() {
        let h1 = compute_hash("test input");
        let h2 = compute_hash("test input");
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 16);
    }

    #[test]
    fn extract_hash_finds_hash() {
        let content = "# Generated\n# Hash: abc123\n";
        assert_eq!(extract_hash(content), Some("abc123".into()));
    }

    #[test]
    fn resolve_outputs_unchanged_when_no_write_needed() {
        let mut inputs = HashMap::new();
        inputs.insert("needs_write".into(), Value::Bool(false));
        let out = FileOp::ResolveUpsert.execute(inputs).unwrap();
        assert!(matches!(out.get("status"), Some(Value::Str(s)) if s == "Unchanged"));
    }
}
