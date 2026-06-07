//! **T-15 receipt:** `src/v4/bin/main.dag` **tokenizes and parses** cleanly (surface syntax +
//! import shape). Full `compile_to_dag` on this file alone is not hermetic today (`import
//! v4.std.node` is not resolved without the staged multi-module load path). **Grounded-indicator
//! authoring is HOLD** pending hub-relayed canonical spec (neat-hawk-87 cascade 2026-05-19).
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching `EXPECTED_HAND_AUTHORED_TEST`
//! line in `sg0_census_test.rs` + INVARIANTS table row land in the same PR.
//!
//! **Dissolution:** remove when `main.dag` is exercised only by `.dag` `TestClaim` rows /
//! generated harness without this per-file Rust probe (or when a single-file `compile_to_dag`
//! path links `v4.std.node` without substrate collision).
//!
//! **Wave-A W1 classification (B-interim, stays as-is):** every receipt here is a
//! declaration-shape / source-presence check over a trampoline file with no executable
//! behavior (data-id presence, the `include!` spelling, digest-stub ids). There is no
//! foldable-now-A chunk. These are **READ-axis-reflection consumers** — they migrate to a
//! `.dag` reflection witness over the parsed module only when the ctrl#1476 READ-axis
//! reflection substrate lands (TRIGGER). No standalone migration PR / no new guard file.

const MAIN_DAG: &str = include_str!("../../../../v4/bin/main.dag");
const MAIN_DAG_PATH: &str = "src/v4/bin/main.dag";

#[test]
fn v4_bin_main_dag_tokenizes_and_parses_with_trampoline_source_anchor() {
    let tokens = v3_compiler::tokenize_for_test(MAIN_DAG, MAIN_DAG_PATH)
        .unwrap_or_else(|e| panic!("{MAIN_DAG_PATH}: tokenize: {e:?}"));
    let _module = v3_compiler::parse_for_test(&tokens, MAIN_DAG_PATH)
        .unwrap_or_else(|e| panic!("{MAIN_DAG_PATH}: parse: {e:?}"));

    assert!(
        MAIN_DAG.contains("main_rs_trampoline_authority"),
        "{MAIN_DAG_PATH}: expected trampoline data id"
    );
    assert!(
        MAIN_DAG.contains(r#"include!(\"v4_main_generated.rs\");"#),
        "{MAIN_DAG_PATH}: expected trampoline include! spelling in String literal"
    );
    for name in [
        "stub_stage1_emitted_rust_digest_placeholder",
        "stub_stage2_emitted_rust_digest_placeholder",
    ] {
        assert!(
            MAIN_DAG.contains(name),
            "{MAIN_DAG_PATH}: expected digest stub data id {name}"
        );
    }
}
