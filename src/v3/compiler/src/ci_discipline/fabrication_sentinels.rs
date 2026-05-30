//! Fail if `__BUG_NO_PROFILE_` is reintroduced into tracked `*.rs` / `*.dag` sources.

use std::fs;
use std::path::Path;

use super::{git_ls_files, repo_root};

const NEEDLE: &str = "__BUG_NO_PROFILE_";

pub fn check_fabrication_sentinels() -> Result<(), String> {
    let root = repo_root()?;
    let files = git_ls_files(&root, &["*.rs", "*.dag"])?;
    let mut violations = 0usize;
    for rel in files {
        if rel.starts_with("docs/") {
            continue;
        }
        let path = root.join(&rel);
        let contents = fs::read_to_string(&path)
            .map_err(|e| format!("read {}: {e}", path.display()))?;
        if contents.contains(NEEDLE) {
            eprintln!("error: {NEEDLE} found in {}", rel.display());
            violations += 1;
        }
    }
    if violations > 0 {
        return Err(format!(
            "check-fabrication-sentinels: failed ({violations} file(s))"
        ));
    }
    eprintln!("check-fabrication-sentinels: ok");
    Ok(())
}
