//! Host transports for CI discipline checks modeled in `src/v4/workflow/ci.dag`.
//! Replaces `scripts/check-*.sh` invocations from `.github/workflows/*.yml`.
//! Each binary mirrors one legacy shell script until T-38 / DisciplinePolicyCommand lands.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// P0-C fabrication sentinel (built with `concat!` so repo grep stays clean).
pub const FABRICATION_SENTINEL: &str = concat!("__BUG", "_NO_PROFILE_");

/// Fail if `FABRICATION_SENTINEL` appears in any tracked `*.rs` / `*.dag` outside `docs/`.
pub fn check_fabrication_sentinels(repo_root: &Path) -> Result<(), Vec<String>> {
    let mut violations = Vec::new();
    for path in git_ls_files(repo_root, &["*.rs", "*.dag"])? {
        if path.starts_with("docs/") {
            continue;
        }
        let abs = repo_root.join(&path);
        let contents = match std::fs::read_to_string(&abs) {
            Ok(c) => c,
            Err(e) => {
                violations.push(format!("{path}: read failed: {e}"));
                continue;
            }
        };
        if contents.contains(FABRICATION_SENTINEL) {
            violations.push(format!("error: {FABRICATION_SENTINEL} found in {path}"));
        }
    }
    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

fn git_ls_files(repo_root: &Path, pathspecs: &[&str]) -> io::Result<Vec<String>> {
    let mut cmd = Command::new("git");
    cmd.current_dir(repo_root).arg("ls-files");
    for spec in pathspecs {
        cmd.arg(spec);
    }
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!(
            "git ls-files exited {}",
            output.status
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

/// Repository root: `CARGO_MANIFEST_DIR/../..` for `tools/ci_host_checks`.
pub fn repo_root_from_manifest_dir(manifest_dir: &Path) -> PathBuf {
    manifest_dir.join("../..").canonicalize().unwrap_or_else(|_| {
        manifest_dir.join("../..")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fabrication_sentinel_scan_clean_on_repo() {
        let root = repo_root_from_manifest_dir(Path::new(env!("CARGO_MANIFEST_DIR")));
        check_fabrication_sentinels(&root).expect("tracked rs/dag sources must stay sentinel-free");
    }
}
