//! SG-3b: `lowering_rust.authority` is load-bearing; `lower_generated.rs` must stay in sync.

const CHECKED_IN_GENERATED: &str = include_str!("../../src/lower_generated.rs");

#[test]
fn lower_generated_module_matches_regen_snapshot() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_path = manifest_dir.join("src").join("lower_generated.rs");
    let fresh =
        std::process::Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
            .current_dir(&manifest_dir)
            .args(["run", "-q", "-p", "v3-compiler", "--bin", "regen_lower"])
            .output()
            .expect("spawn regen_lower");
    assert!(
        fresh.status.success(),
        "regen_lower failed: {}",
        String::from_utf8_lossy(&fresh.stderr)
    );
    let regen = std::fs::read_to_string(&out_path).expect("read regenerated lower_generated.rs");
    assert_eq!(
        CHECKED_IN_GENERATED.trim(),
        regen.trim(),
        "checked-in lower_generated.rs is stale; run `cargo run -p v3-compiler --bin regen_lower`"
    );
}
