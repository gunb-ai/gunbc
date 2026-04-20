//! SG-3f-prep: `lower_generated.rs` is the pass-through output of `regen_lower` from
//! canonical `lower.rs` (not imported by `lib.rs`); the snapshot must stay in sync.
//!
//! Hermetic: `regen_lower` is invoked with `--out` under `temp_dir()` so the workspace
//! tree is not rewritten during the test (TESTING.md).

const CHECKED_IN_GENERATED: &str = include_str!("../../src/lower_generated.rs");

#[test]
fn lower_generated_module_matches_regen_snapshot() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_path = std::env::temp_dir().join(format!(
        "v3_regen_lower_{}_lower_generated.rs",
        std::process::id()
    ));
    let fresh =
        std::process::Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
            .current_dir(&manifest_dir)
            .args([
                "run",
                "-q",
                "-p",
                "v3-compiler",
                "--bin",
                "regen_lower",
                "--",
                "--out",
            ])
            .arg(&out_path)
            .output()
            .expect("spawn regen_lower");
    assert!(
        fresh.status.success(),
        "regen_lower failed: {}",
        String::from_utf8_lossy(&fresh.stderr)
    );
    let regen = std::fs::read_to_string(&out_path).expect("read regenerated lower_generated.rs");
    let _ = std::fs::remove_file(&out_path);
    assert_eq!(
        CHECKED_IN_GENERATED.trim(),
        regen.trim(),
        "checked-in lower_generated.rs is stale; run `cargo run -p v3-compiler --bin regen_lower`"
    );
}
