//! **Layer:** integration
//!
//! Zero-residual **receipt + ratchet** for the PB Tier-2 lower-helper
//! post-processing bridge tracked as **`bridge_patch_lower_helpers_residual_retired`**
//! / **`bridge_exact_string_patching_residual_retired`** (lower-helper slice only;
//! see `docs/r3-structure.md` (T-Bridge-Retirement) and
//! `docs/briefs/r2-pure-bootstrap-manager.md`).
//!
//! ## Audit (PB v3-compiler crate, post–PR #1014)
//!
//! - `rg 'patch_lower_helpers' src/v3/compiler --glob '*.rs'` → **no matches**
//!   (the named `patch_lower_helpers_generated_type_alias_refinement` helper and
//!   `regen_lens` / SG-6 string-patch special cases were deleted in #1014).
//! - `build.rs` at the crate root → **no** contiguous `patch_lower` + `_helpers`
//!   substring.
//! - `src/lib.rs` is under `src/` and covered by the same walk as other compiler
//!   Rust sources.
//!
//! ## Ratchet scope (narrow)
//!
//! Fails CI only if the **contiguous** symbol `patch_lower` + `_helpers` reappears
//! in any `.rs` under this crate's `src/` or `tests/` trees, or in `build.rs`.
//! Filesystem read failures fail the test (no silent skip — incomplete scans must
//! not report a clean bill of health).
//! This is specific to the **retired lower-helper generated-Rust exact-string patch
//! class** from #1014 — it does **not** ban unrelated `String::replace` /
//! template splicing (e.g. `build.rs` r1_gates lens splice, emitter `%Q` escapes).

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

const SELF_BASENAME: &str = "bridge_lower_helpers_patch_zero_residual_test.rs";

/// Retired bridge symbols use this contiguous substring (`patch_lower` +
/// `_helpers`, e.g. `patch_lower_helpers_generated`, `patch_lower_helpers_*`).
/// Defined with `concat!` so this source file does not embed the forbidden spelling.
const FORBIDDEN: &str = concat!("patch_lower", "_helpers");

fn visit_rs(root: &Path, offenders: &mut Vec<String>, scan_errors: &mut Vec<String>) {
    let read_dir = match fs::read_dir(root) {
        Ok(rd) => rd,
        Err(e) => {
            scan_errors.push(format!("{}: read_dir: {e}", root.display()));
            return;
        }
    };
    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                scan_errors.push(format!("{}: directory entry: {e}", root.display()));
                continue;
            }
        };
        let path = entry.path();
        if path.is_dir() {
            visit_rs(&path, offenders, scan_errors);
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
        let text = match fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                scan_errors.push(format!("{}: read_to_string: {e}", path.display()));
                continue;
            }
        };
        if text.contains(FORBIDDEN) {
            offenders.push(path.display().to_string());
        }
    }
}

#[test]
fn lower_helpers_patch_bridge_exact_string_residual_stays_zero() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut offenders = Vec::new();
    let mut scan_errors = Vec::new();

    visit_rs(&manifest_dir.join("src"), &mut offenders, &mut scan_errors);
    visit_rs(
        &manifest_dir.join("tests"),
        &mut offenders,
        &mut scan_errors,
    );

    let build_rs = manifest_dir.join("build.rs");
    match fs::read_to_string(&build_rs) {
        Ok(text) => {
            if text.contains(FORBIDDEN) {
                offenders.push(build_rs.display().to_string());
            }
        }
        Err(e) => {
            scan_errors.push(format!("{}: read_to_string: {e}", build_rs.display()));
        }
    }

    assert!(
        scan_errors.is_empty(),
        "bridge_patch_lower_helpers_residual_retired: source scan must be complete (scan_errors: {scan_errors:?})"
    );
    assert!(
        offenders.is_empty(),
        "bridge_patch_lower_helpers_residual_retired: `{FORBIDDEN}` must not reappear in v3-compiler Rust after PR #1014 (offenders: {offenders:?})"
    );
}
