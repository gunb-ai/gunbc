//! SG-2 **parser staging** ratchet (PR #589 option 1 — **not** SG-2b closure): `runtime_mirrors.dag`
//! is load-bearing for the **Surface carrier** schema (shared with `parse_surface_generated.rs`);
//! `parse_generated.rs` (types from that `.dag` + **temporary semantic** algorithm from
//! `parse_parser_body.txt`) must stay in sync via `regen_parse`. The snapshot test compares an
//! in-process render (no tracked-file rewrite). SG-2b remains the follow-on lane that deletes the
//! body fragment per the dissolution trigger in `parse_parser_body.txt`.

use v3_compiler::compile_runtime_mirrors_authority_dag;
use v3_compiler::render_parse_generated_rs;

const RUNTIME_MIRRORS_DAG: &str = include_str!("../../runtime_mirrors.dag");
const CHECKED_IN_GENERATED: &str = include_str!("../../src/parse_generated.rs");
const PARSER_BODY: &str = include_str!("../../parse_parser_body.txt");

#[test]
fn runtime_mirrors_dag_compiles_cleanly_for_regen_parse() {
    compile_runtime_mirrors_authority_dag(
        RUNTIME_MIRRORS_DAG,
        "src/v3/compiler/runtime_mirrors.dag",
    )
    .unwrap_or_else(|e| {
        panic!("runtime_mirrors.dag should compile for regen_parse authority: {e:?}")
    });
}

#[test]
fn parse_generated_module_matches_checked_in_snapshot() {
    let regen = render_parse_generated_rs(
        RUNTIME_MIRRORS_DAG,
        "src/v3/compiler/runtime_mirrors.dag",
        PARSER_BODY,
    )
    .unwrap_or_else(|e| panic!("render parse_generated.rs in-process: {e}"));
    assert_eq!(
        CHECKED_IN_GENERATED.trim(),
        regen.trim(),
        "checked-in parse_generated.rs is stale; run `cargo run -p v3-compiler --bin regen_parse`"
    );
}
