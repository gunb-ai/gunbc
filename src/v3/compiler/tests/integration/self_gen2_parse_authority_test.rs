//! SG-2 **parser staging** ratchet (PR #589 option 1 — **not** SG-2b closure): `parse_surface.dag`
//! is load-bearing for the **Surface carrier** schema (shared with `parse_surface_generated.rs`);
//! `parse_generated.rs` (types from that `.dag` + **temporary semantic** algorithm from
//! `parse_parser_body.txt`) must stay in sync via `regen_parse`. The snapshot test compares an
//! in-process render (no tracked-file rewrite). SG-2b remains the follow-on lane that deletes the
//! body fragment per the dissolution trigger in `parse_parser_body.txt`.

use v3_compiler::compile_parse_surface_std_authority_dag;
use v3_compiler::render_parse_generated_rs;

const PARSE_SURFACE_DAG: &str = include_str!("../../../std/parse_surface.dag");
const CHECKED_IN_GENERATED: &str = include_str!("../../src/parse_generated.rs");
const PARSER_BODY: &str = include_str!("../../parse_parser_body.txt");

#[test]
fn parse_surface_dag_compiles_cleanly_for_regen_parse() {
    compile_parse_surface_std_authority_dag(PARSE_SURFACE_DAG, "src/v3/std/parse_surface.dag")
        .unwrap_or_else(|e| {
            panic!("parse_surface.dag should compile for regen_parse authority: {e:?}")
        });
}

#[test]
fn parse_generated_module_matches_checked_in_snapshot() {
    let regen = render_parse_generated_rs(
        PARSE_SURFACE_DAG,
        "src/v3/std/parse_surface.dag",
        PARSER_BODY,
    )
    .unwrap_or_else(|e| panic!("render parse_generated.rs in-process: {e}"));
    assert_eq!(
        CHECKED_IN_GENERATED.trim(),
        regen.trim(),
        "checked-in parse_generated.rs is stale; run `cargo run -p v3-compiler --bin regen_parse`"
    );
}
