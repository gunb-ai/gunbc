use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::process::Command;

use gunbc_exec::{ExecError, Executable, Value};

/// The operation type for gistgen nodes.
#[derive(Debug, Clone)]
pub enum GistgenOp {
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
    UploadGist {
        dry_run: bool,
    },
}

impl Executable for GistgenOp {
    fn execute(&self, inputs: HashMap<String, Value>) -> Result<HashMap<String, Value>, ExecError> {
        match self {
            GistgenOp::Context { repo_path, glob_pattern } => {
                let mut out = HashMap::new();
                let abs_path = std::fs::canonicalize(repo_path)
                    .unwrap_or_else(|_| std::path::PathBuf::from(repo_path));
                out.insert("repo".into(), Value::Str(abs_path.to_string_lossy().into_owned()));
                out.insert("selection_spec".into(), Value::Str(glob_pattern.clone()));
                Ok(out)
            }

            GistgenOp::AuthCheck => {
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

            GistgenOp::AuthCreate => {
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

            GistgenOp::AuthResolve => {
                let token = inputs.get("check_token")
                    .filter(|v| !matches!(v, Value::Skipped))
                    .or_else(|| inputs.get("create_token").filter(|v| !matches!(v, Value::Skipped)))
                    .cloned()
                    .unwrap_or(Value::Unit);
                let mut out = HashMap::new();
                out.insert("token".into(), token);
                Ok(out)
            }

            GistgenOp::EnumerateFiles => {
                let repo = inputs.get("repo")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| ".".into());

                let mut files = Vec::new();
                let walker = ignore::WalkBuilder::new(&repo)
                    .hidden(false)
                    .git_ignore(true)
                    .git_global(true)
                    .git_exclude(true)
                    .build();

                for entry in walker {
                    match entry {
                        Ok(e) if e.file_type().map_or(false, |ft| ft.is_file()) => {
                            files.push(e.path().to_string_lossy().into_owned());
                        }
                        _ => {}
                    }
                }
                files.sort();
                let mut out = HashMap::new();
                out.insert("files".into(), Value::StrList(files));
                Ok(out)
            }

            GistgenOp::FilterFiles => {
                let file_list = inputs.get("files")
                    .and_then(|v| if let Value::StrList(v) = v { Some(v.clone()) } else { None })
                    .unwrap_or_default();
                let spec = inputs.get("selection_spec")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_else(|| "**/*".into());

                let glob = globset::Glob::new(&spec)
                    .map_err(|e| ExecError(format!("Invalid glob pattern '{spec}': {e}")))?
                    .compile_matcher();

                let filtered: Vec<String> = file_list.into_iter()
                    .filter(|line| {
                        let p = Path::new(line);
                        glob.is_match(p) || glob.is_match(p.file_name().unwrap_or_default())
                    })
                    .collect();

                let mut out = HashMap::new();
                out.insert("files".into(), Value::StrList(filtered));
                Ok(out)
            }

            GistgenOp::ReadFiles => {
                let file_list = inputs.get("files")
                    .and_then(|v| if let Value::StrList(v) = v { Some(v.clone()) } else { None })
                    .unwrap_or_default();

                let mut contents = BTreeMap::new();
                for path in &file_list {
                    if path.is_empty() {
                        continue;
                    }
                    match std::fs::read_to_string(path) {
                        Ok(content) => {
                            contents.insert(path.clone(), content);
                        }
                        Err(e) => {
                            contents.insert(path.clone(), format!("[error reading file: {e}]"));
                        }
                    }
                }
                let mut out = HashMap::new();
                out.insert("contents".into(), Value::MapStrStr(contents));
                Ok(out)
            }

            GistgenOp::ComposeSnapshot => {
                let contents = inputs.get("contents")
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

            GistgenOp::UploadGist { dry_run } => {
                let snapshot = inputs.get("snapshot")
                    .and_then(|v| if let Value::Str(s) = v { Some(s.clone()) } else { None })
                    .unwrap_or_default();
                let token_display = match inputs.get("token") {
                    Some(Value::Secret(_)) => "<REDACTED>".to_string(),
                    Some(v) => format!("{v}"),
                    None => "none".into(),
                };

                if *dry_run {
                    eprintln!("[DRY RUN] Would upload gist:");
                    eprintln!("  Token: {token_display}");
                    eprintln!("  Snapshot length: {} bytes", snapshot.len());
                    if !snapshot.is_empty() {
                        eprintln!("  Preview: {}...", &snapshot[..snapshot.len().min(200)]);
                    }
                    let mut out = HashMap::new();
                    out.insert("gist_url".into(), Value::Str("https://gist.github.com/dry-run/preview".into()));
                    return Ok(out);
                }

                // Real upload via `gh gist create -`
                eprintln!("Uploading gist ({} bytes)...", snapshot.len());
                let mut child = Command::new("gh")
                    .args(["gist", "create", "-"])
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()
                    .map_err(|e| ExecError(format!("Failed to spawn gh: {e}")))?;

                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    stdin.write_all(snapshot.as_bytes())
                        .map_err(|e| ExecError(format!("Failed to write to gh stdin: {e}")))?;
                }

                let output = child.wait_with_output()
                    .map_err(|e| ExecError(format!("Failed to wait for gh: {e}")))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(ExecError(format!("gh gist create failed: {stderr}")));
                }

                let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
                eprintln!("Gist created: {url}");
                let mut out = HashMap::new();
                out.insert("gist_url".into(), Value::Str(url));
                Ok(out)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enumerate_finds_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "fn main() {}").unwrap();
        std::fs::write(dir.path().join("b.txt"), "hello").unwrap();

        let mut inputs = HashMap::new();
        inputs.insert("repo".into(), Value::Str(dir.path().to_string_lossy().into_owned()));

        let op = GistgenOp::EnumerateFiles;
        let out = op.execute(inputs).unwrap();
        if let Value::StrList(files) = &out["files"] {
            let joined = files.join("\n");
            assert!(joined.contains("a.rs"));
            assert!(joined.contains("b.txt"));
        } else {
            panic!("expected StrList");
        }
    }

    #[test]
    fn filter_respects_glob() {
        let mut inputs = HashMap::new();
        inputs.insert("files".into(), Value::StrList(vec![
            "src/main.rs".into(), "src/lib.rs".into(), "README.md".into(),
        ]));
        inputs.insert("selection_spec".into(), Value::Str("**/*.rs".into()));

        let op = GistgenOp::FilterFiles;
        let out = op.execute(inputs).unwrap();
        if let Value::StrList(files) = &out["files"] {
            assert_eq!(files.len(), 2);
            assert!(files.iter().any(|f| f.contains("main.rs")));
            assert!(files.iter().any(|f| f.contains("lib.rs")));
        } else {
            panic!("expected StrList");
        }
    }

    #[test]
    fn read_files_returns_map() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        std::fs::write(&file_path, "hello world").unwrap();

        let mut inputs = HashMap::new();
        inputs.insert("files".into(), Value::StrList(vec![
            file_path.to_string_lossy().into_owned(),
        ]));

        let op = GistgenOp::ReadFiles;
        let out = op.execute(inputs).unwrap();
        if let Value::MapStrStr(map) = &out["contents"] {
            assert_eq!(map.len(), 1);
            assert_eq!(map.values().next().unwrap(), "hello world");
        } else {
            panic!("expected MapStrStr");
        }
    }

    #[test]
    fn auth_check_reads_env() {
        std::env::set_var("GITHUB_TOKEN", "test_token_abc");
        let op = GistgenOp::AuthCheck;
        let out = op.execute(HashMap::new()).unwrap();
        assert!(matches!(out.get("token"), Some(Value::Secret(_))));
        assert!(matches!(out.get("needs_create"), Some(Value::Bool(false))));
        std::env::remove_var("GITHUB_TOKEN");
    }

    #[test]
    fn dry_run_does_not_call_gh() {
        let mut inputs = HashMap::new();
        inputs.insert("snapshot".into(), Value::Str("test snapshot".into()));
        inputs.insert("token".into(), Value::Secret(gunbc_ir::Secret("tok".into())));

        let op = GistgenOp::UploadGist { dry_run: true };
        let out = op.execute(inputs).unwrap();
        if let Value::Str(url) = &out["gist_url"] {
            assert!(url.contains("dry-run"));
        } else {
            panic!("expected Str");
        }
    }

    #[test]
    fn auth_resolve_picks_non_skipped() {
        let mut inputs = HashMap::new();
        inputs.insert("check_token".into(), Value::Skipped);
        inputs.insert("create_token".into(), Value::Secret(gunbc_ir::Secret("real_tok".into())));

        let op = GistgenOp::AuthResolve;
        let out = op.execute(inputs).unwrap();
        assert!(matches!(out.get("token"), Some(Value::Secret(_))));
    }
}
