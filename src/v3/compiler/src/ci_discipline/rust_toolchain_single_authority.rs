//! P2 single-authority: pinned rustc channel only in `rust-toolchain.toml`; guard rustup.dag + workflows.

use std::fs;
use std::path::Path;

use super::repo_root;

const RUSTUP_DAG: &str = "dsl/extdeps/rustup.dag";
const TOOLCHAIN_TOML: &str = "rust-toolchain.toml";
const WORKFLOWS_DIR: &str = ".github/workflows";
const SETUP_ACTION: &str = "actions-rust-lang/setup-rust-toolchain";

pub fn check_rust_toolchain_single_authority() -> Result<(), String> {
    let root = repo_root()?;
    let rustup_dag = root.join(RUSTUP_DAG);
    let toolchain_toml = root.join(TOOLCHAIN_TOML);
    let workflows_dir = root.join(WORKFLOWS_DIR);

    for path in [&rustup_dag, &toolchain_toml] {
        if !path.is_file() {
            return Err(format!("::error::missing {}", path.display()));
        }
    }
    if !workflows_dir.is_dir() {
        return Err(format!("::error::missing {}", workflows_dir.display()));
    }

    let toml_text = fs::read_to_string(&toolchain_toml)
        .map_err(|e| format!("read rust-toolchain.toml: {e}"))?;
    let channel = parse_toolchain_channel(&toml_text)?;
    let quoted = format!("\"{channel}\"");
    let rustup_text = fs::read_to_string(&rustup_dag)
        .map_err(|e| format!("read {RUSTUP_DAG}: {e}"))?;

    if rustup_text.contains(&quoted) {
        return Err(format!(
            "::error::dsl/extdeps/rustup.dag contains the pinned channel literal {quoted} — duplicate authority (keep the channel only in rust-toolchain.toml)."
        ));
    }

    if channel.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        if rustup_text.contains(&channel) {
            return Err(format!(
                "::error::dsl/extdeps/rustup.dag contains bare channel token '{channel}' — duplicate authority (keep the channel only in rust-toolchain.toml)."
            ));
        }
    }

    if rustup_text.contains("data ci_pinned_toolchain") {
        return Err(
            "::error::dsl/extdeps/rustup.dag declares ci_pinned_toolchain — retired duplicate authority symbol. Use rust-toolchain.toml only.".into(),
        );
    }

    scan_workflows_for_toolchain_input(&workflows_dir, &root)?;

    eprintln!(
        "Rust toolchain single-authority check OK (channel={channel}; rustup.dag + workflow guard)."
    );
    Ok(())
}

fn parse_toolchain_channel(toml: &str) -> Result<String, String> {
    for line in toml.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("channel") {
            if let Some(eq_pos) = rest.find('=') {
                let value = rest[eq_pos + 1..].trim();
                if let Some(inner) = value.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                    if !inner.is_empty() {
                        return Ok(inner.to_string());
                    }
                }
            }
        }
    }
    Err("::error::could not parse [toolchain].channel from rust-toolchain.toml".into())
}

fn scan_workflows_for_toolchain_input(workflows_dir: &Path, repo_root: &Path) -> Result<(), String> {
    let mut files = Vec::new();
    for entry in fs::read_dir(workflows_dir).map_err(|e| format!("read workflows: {e}"))? {
        let entry = entry.map_err(|e| format!("read workflow entry: {e}"))?;
        let path = entry.path();
        if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "yml" || ext == "yaml" {
                files.push(path);
            }
        }
    }
    files.sort();
    if files.is_empty() {
        return Err(format!(
            "::error::no *.yml or *.yaml under {}",
            workflows_dir.display()
        ));
    }

    for wf in files {
        let text = fs::read_to_string(&wf).map_err(|e| format!("read {}: {e}", wf.display()))?;
        let lines: Vec<&str> = text.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if !trimmed.starts_with("toolchain:") {
                continue;
            }
            let tc_indent = line.len() - trimmed.len();
            let mut step_start = None;
            for j in (0..=i).rev() {
                let l = lines[j];
                if let Some(pos) = l.find('-') {
                    let before = &l[..pos];
                    if before.trim().is_empty() && l.trim_start().starts_with('-') {
                        let dash_indent = before.len();
                        if dash_indent < tc_indent {
                            step_start = Some(j);
                            break;
                        }
                    }
                }
            }
            let Some(start) = step_start else {
                continue;
            };
            let mut end = lines.len();
            let base_indent = lines[start]
                .find('-')
                .map(|p| lines[start][..p].len())
                .unwrap_or(0);
            for k in (start + 1)..lines.len() {
                let l = lines[k];
                if let Some(pos) = l.find('-') {
                    let before = &l[..pos];
                    if before.trim().is_empty() && l.trim_start().starts_with('-') {
                        if before.len() == base_indent {
                            end = k;
                            break;
                        }
                    }
                }
            }
            let block = lines[start..end].join("\n");
            if block.contains(SETUP_ACTION) {
                let rel = wf.strip_prefix(repo_root).unwrap_or(&wf);
                return Err(format!(
                    "::error::file={},line={}::explicit `toolchain:` input on {SETUP_ACTION} — rust-toolchain.toml would be ignored. Remove it from that step's `with:`.",
                    rel.display(),
                    i + 1
                ));
            }
        }
    }
    Ok(())
}
