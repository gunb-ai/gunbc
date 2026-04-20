//! SG-2 **parser staging** ratchet (PR #589 option 1 — **not** SG-2b closure): `runtime_mirrors.dag`
//! is load-bearing for the **Surface carrier** schema (shared with `parse_surface_generated.rs`);
//! `parse_generated.rs` (types from that `.dag` + **temporary semantic** algorithm from
//! `parse_parser_body.txt`) must stay in sync via `regen_parse`. SG-2b remains the follow-on lane
//! that deletes the body fragment per the dissolution trigger in `parse_parser_body.txt`.

use v3_compiler::compile_runtime_mirrors_authority_dag;

const RUNTIME_MIRRORS_DAG: &str = include_str!("../../runtime_mirrors.dag");
const CHECKED_IN_GENERATED: &str = include_str!("../../src/parse_generated.rs");

#[test]
fn runtime_mirrors_dag_compiles_cleanly_for_regen_parse() {
    compile_runtime_mirrors_authority_dag(RUNTIME_MIRRORS_DAG, "src/v3/compiler/runtime_mirrors.dag")
        .unwrap_or_else(
        |e| panic!("runtime_mirrors.dag should compile for regen_parse authority: {e:?}"),
    );
}

#[test]
fn parse_generated_module_matches_checked_in_snapshot() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_path = manifest_dir.join("src").join("parse_generated.rs");
    let fresh =
        std::process::Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
            .current_dir(&manifest_dir)
            .args(["run", "-q", "-p", "v3-compiler", "--bin", "regen_parse"])
            .output()
            .expect("spawn regen_parse");
    assert!(
        fresh.status.success(),
        "regen_parse failed: {}",
        String::from_utf8_lossy(&fresh.stderr)
    );
    let regen = std::fs::read_to_string(&out_path).expect("read regenerated parse_generated.rs");
    assert_eq!(
        CHECKED_IN_GENERATED.trim(),
        regen.trim(),
        "checked-in parse_generated.rs is stale; run `cargo run -p v3-compiler --bin regen_parse`"
    );
}
