//! SG-1: `tokenize.dag` is load-bearing authority; `tokenize_generated.rs` must stay in sync.

use v3_compiler::compile_to_dag;

const TOKENIZE_DAG: &str = include_str!("../../tokenize.dag");
const CHECKED_IN_GENERATED: &str = include_str!("../../src/tokenize_generated.rs");

#[test]
fn tokenize_dag_compiles_cleanly() {
    compile_to_dag(TOKENIZE_DAG, "src/v3/compiler/tokenize.dag")
        .unwrap_or_else(|e| panic!("tokenize.dag should compile: {e:?}"));
}

#[test]
fn tokenize_generated_module_matches_checked_in_snapshot() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_path = manifest_dir.join("src").join("tokenize_generated.rs");
    let fresh = std::process::Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
        .current_dir(&manifest_dir)
        .args(["run", "-q", "-p", "v3-compiler", "--bin", "regen_tokenize"])
        .output()
        .expect("spawn regen_tokenize");
    assert!(
        fresh.status.success(),
        "regen_tokenize failed: {}",
        String::from_utf8_lossy(&fresh.stderr)
    );
    let regen = std::fs::read_to_string(&out_path).expect("read regenerated tokenize_generated.rs");
    assert_eq!(
        CHECKED_IN_GENERATED.trim(),
        regen.trim(),
        "checked-in tokenize_generated.rs is stale; run `cargo run -p v3-compiler --bin regen_tokenize`"
    );
}
