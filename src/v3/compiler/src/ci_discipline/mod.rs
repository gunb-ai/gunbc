//! CI discipline gates — Rust implementations replacing `scripts/check-*.sh` invoked
//! from GitHub Actions (workflow shell elimination). Each `check_*` returns `Ok(())`
//! or `Err(detail)` for the `gunbc-ci discipline` host shim.

mod banked_dissolutions;
mod fabrication_sentinels;
mod rust_toolchain_single_authority;

pub use banked_dissolutions::check_banked_dissolutions;
pub use fabrication_sentinels::check_fabrication_sentinels;
pub use rust_toolchain_single_authority::check_rust_toolchain_single_authority;

use std::path::{Path, PathBuf};
use std::process::Command;

/// Repository root: `GITHUB_WORKSPACE` when set, else `git rev-parse --show-toplevel`.
pub fn repo_root() -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var("GITHUB_WORKSPACE") {
        let path = PathBuf::from(p);
        if path.is_dir() {
            return Ok(path);
        }
    }
    let out = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| format!("git rev-parse: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned());
    }
    let root = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if root.is_empty() {
        return Err("git rev-parse returned empty path".into());
    }
    Ok(PathBuf::from(root))
}

/// `git ls-files` with pathspec arguments; paths are relative to `repo_root`.
pub fn git_ls_files(repo_root: &Path, pathspecs: &[&str]) -> Result<Vec<PathBuf>, String> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root).arg("ls-files");
    for spec in pathspecs {
        cmd.arg(spec);
    }
    let out = cmd
        .output()
        .map_err(|e| format!("git ls-files: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).into_owned());
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(PathBuf::from)
        .collect())
}
