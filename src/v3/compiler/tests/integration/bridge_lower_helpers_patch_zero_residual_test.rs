//! **Layer:** integration
//!
//! Zero-residual **receipt + ratchet** for the PB Tier-2 lower-helper
//! post-processing bridge: ledger row **`bridge_exact_string_patching_residual_retired`**
//! is **`Retired`** in `src/v3/std/bridge_ledger.dag` for the **lower-helper slice**
//! only (PB brief
//! `docs/briefs/r2-pure-bootstrap-manager.md` names the sibling Tier-2 row as
//! `bridge_patch_lower` + `_helpers_residual_retired` split across lines here
//! so this source stays free of the contiguous forbidden token under scan).
//! Remaining Row-4 semantic exact-string patching outside this slice is tracked
//! as the open ledger row `bridge_exact_string_semantic_patching_residual`.
//! See `docs/r3-structure.md` (T-Bridge-Retirement).
//!
//! ## Audit (PB v3-compiler crate, post–PR #1014)
//!
//! - Hand-audit / `rg` for the same **contiguous** token this test forbids (built
//!   via `concat!` in code) should find **no occurrences** in any crate `.rs` or
//!   `build.rs` after #1014’s deletions — this test enforces that predicate on
//!   disk, **including this file** (no self-skip).
//! - `build.rs` at the crate root → **no** contiguous `patch_lower` + `_helpers`
//!   substring.
//! - `src/lib.rs` is under `src/` and covered by the same walk as other compiler
//!   Rust sources.
//!
//! ## Ratchet scope (narrow)
//!
//! Fails CI only if the **contiguous** symbol `patch_lower` + `_helpers` reappears
//! in any `.rs` under this crate's `src/` or `tests/` trees, or in `build.rs`
//! (including **this** file — no self-skip).
//! Filesystem read failures fail the test (no silent skip — incomplete scans must
//! not report a clean bill of health).
//! This is specific to the **retired lower-helper generated-Rust exact-string patch
//! class** from #1014 — it does **not** ban unrelated `String::replace` /
//! template splicing (e.g. `build.rs` r1_gates lens splice, emitter `%Q` escapes).

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

/// Retired bridge symbols use this contiguous substring (`patch_lower` +
/// `_helpers`, e.g. deleted `*_generated` / `*_refinement` helpers from #1014).
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
        "lower_helper_patch_bridge_zero_residual: source scan must be complete (scan_errors: {scan_errors:?})"
    );
    assert!(
        offenders.is_empty(),
        "lower_helper_patch_bridge_zero_residual: `{FORBIDDEN}` must not reappear in v3-compiler Rust after PR #1014 (offenders: {offenders:?})"
    );
}
