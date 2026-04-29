//! **Layer:** integration
//!
//! Zero-residual receipt for **`bridge_patch_lower_helpers_residual_retired`**
//! (T-Bridge-Retirement distributed bridge #5; see `docs/r3-structure.md` and
//! `docs/briefs/r2-pure-bootstrap-manager.md`). PR #1014 retired the
//! `patch_lower` + `_helpers` generated-Rust exact-string patch class; this
//! test fails if that contiguous symbol reappears in v3-compiler Rust sources,
//! `build.rs`, or integration/boundary/unit tests under this crate.

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

const SELF_BASENAME: &str = "patch_lower_helpers_bridge_zero_residual_test.rs";

/// The retired bridge spells this contiguous substring; keep it out of the
/// tree except via `concat!(...)` in this file's own needle definition.
const FORBIDDEN: &str = concat!("patch_lower", "_helpers");

fn visit_rs(root: &Path, offenders: &mut Vec<String>) {
    let Ok(read_dir) = fs::read_dir(root) else {
        return;
    };
    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_rs(&path, offenders);
            continue;
        }
        if path.extension() != Some(OsStr::new("rs")) {
            continue;
        }
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n == SELF_BASENAME)
        {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if text.contains(FORBIDDEN) {
            offenders.push(path.display().to_string());
        }
    }
}

#[test]
fn patch_lower_helpers_exact_string_patch_bridge_stays_zero_residual() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut offenders = Vec::new();

    visit_rs(&manifest_dir.join("src"), &mut offenders);
    visit_rs(&manifest_dir.join("tests"), &mut offenders);

    let build_rs = manifest_dir.join("build.rs");
    if let Ok(text) = fs::read_to_string(&build_rs) {
        if text.contains(FORBIDDEN) {
            offenders.push(build_rs.display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "bridge_patch_lower_helpers_residual_retired: contiguous `{FORBIDDEN}` must not appear in v3-compiler crate Rust after PR #1014 (offenders: {offenders:?})"
    );
}
