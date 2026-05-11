//! §1.8 gate **`method_template_projection_emit_shim_retirement_coherence`** (ledger row **#97**).
//!
//! Q-V2-Retirement-Boundary-Matrix consumer: **terminal state** after the legacy compiler tree and Gap-4
//! emit-shim retirement (Gunbc **#1982**). While the tree existed, Gap-4 required
//! `pb_method_template_projection_dag_emit` + the `emit_method_template_projection` bin; after PB deletes
//! the tree, those surfaces must be **absent** (`Cargo.toml` `[[bin]]`, `lib.rs` export, sources, census —
//! Grounding G6).
//!
//! **Pass condition.** Let `legacy_stage0` be `workspace_root/src` + `v2` + `stage0` (path built without a
//! single `src/v2…` literal for mechanical greps). If that directory exists, the shim triplet must exist;
//! otherwise none of the shim artifacts may remain.
//!
//! **`autobins = false`** (`src/v3/compiler/Cargo.toml`): when the shim was load-bearing, the explicit
//! **`[[bin]]`** table mattered, not merely `src/bin/*.rs` on disk.

use std::path::PathBuf;

/// True iff `Cargo.toml` contains a `[[bin]]` table naming `emit_method_template_projection` with the
/// canonical `path = "src/bin/emit_method_template_projection.rs"` (PB #1560 Gap-4 / regen consumer wiring).
fn cargo_toml_declares_emit_method_template_projection_bin(cargo_toml: &str) -> bool {
    let lines: Vec<&str> = cargo_toml.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() == "[[bin]]" {
            i += 1;
            let block_start = i;
            while i < lines.len() && !lines[i].trim().starts_with("[[") {
                i += 1;
            }
            let mut has_name = false;
            let mut has_path = false;
            for line in &lines[block_start..i] {
                let t = line.trim();
                if let Some(rest) = t.strip_prefix("name") {
                    let rest = rest.trim_start();
                    if let Some(rest) = rest.strip_prefix('=') {
                        let v = rest.trim().trim_matches('"');
                        if v == "emit_method_template_projection" {
                            has_name = true;
                        }
                    }
                }
                if let Some(rest) = t.strip_prefix("path") {
                    let rest = rest.trim_start();
                    if let Some(rest) = rest.strip_prefix('=') {
                        let v = rest.trim().trim_matches('"');
                        if v == "src/bin/emit_method_template_projection.rs" {
                            has_path = true;
                        }
                    }
                }
            }
            if has_name && has_path {
                return true;
            }
            continue;
        }
        i += 1;
    }
    false
}

#[test]
fn method_template_projection_emit_shim_retirement_coherence() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.join("..").join("..").join("..");
    assert!(
        workspace_root.join("Cargo.toml").is_file(),
        "expected workspace root at {}",
        workspace_root.display()
    );

    let legacy_stage0 = workspace_root.join("src").join("v2").join("stage0");
    let shim_lib =
        workspace_root.join("src/v3/compiler/src/pb_method_template_projection_dag_emit.rs");
    let shim_bin =
        workspace_root.join("src/v3/compiler/src/bin/emit_method_template_projection.rs");

    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    let cargo_toml = std::fs::read_to_string(&cargo_toml_path)
        .unwrap_or_else(|e| panic!("§1.8 gate #97: read {}: {e}", cargo_toml_path.display()));
    let bin_manifest = cargo_toml_declares_emit_method_template_projection_bin(&cargo_toml);

    let legacy_tree_present = legacy_stage0.is_dir();
    let shim_lib_present = shim_lib.is_file();
    let shim_bin_present = shim_bin.is_file();

    assert!(
        !legacy_tree_present || (shim_lib_present && shim_bin_present && bin_manifest),
        "§1.8 gate #97: with legacy stage0 at {}, Gap-4 emit shim must remain \
         (`pb_method_template_projection_dag_emit` + `emit_method_template_projection` bin + \
         `[[bin]]` in {}); saw shim_lib={shim_lib_present} shim_rs={shim_bin_present} \
         cargo_bin_table={bin_manifest}",
        legacy_stage0.display(),
        cargo_toml_path.display(),
    );

    assert!(
        legacy_tree_present || (!shim_lib_present && !shim_bin_present && !bin_manifest),
        "§1.8 gate #97: legacy tree removed but Gap-4 emit shim still present — retire \
         `pb_method_template_projection_dag_emit.rs`, `emit_method_template_projection` bin target in \
         `Cargo.toml`, `emit_method_template_projection.rs`, \
         `lib.rs` export, and census entries per Gunbc #1982 (Grounding G6)"
    );
}
