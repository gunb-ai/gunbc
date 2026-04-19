use v3_compiler::runtime_mirrors_codegen::render_runtime_mirrors;

#[test]
fn generated_runtime_mirrors_match_checked_in_snapshots() {
    let generated = render_runtime_mirrors().expect("render runtime mirrors");

    assert_eq!(
        generated.types.trim(),
        include_str!("../../src/types_generated.rs").trim(),
        "checked-in types_generated.rs is stale; run `cargo run --bin regen_runtime_mirrors`"
    );
    assert_eq!(
        generated.diagnostics.trim(),
        include_str!("../../src/diagnostics_generated.rs").trim(),
        "checked-in diagnostics_generated.rs is stale; run `cargo run --bin regen_runtime_mirrors`"
    );
    assert_eq!(
        generated.serialize.trim(),
        include_str!("../../src/serialize_generated.rs").trim(),
        "checked-in serialize_generated.rs is stale; run `cargo run --bin regen_runtime_mirrors`"
    );
    assert_eq!(
        generated.dag_cost.trim(),
        include_str!("../../src/dag_cost_generated.rs").trim(),
        "checked-in dag_cost_generated.rs is stale; run `cargo run --bin regen_runtime_mirrors`"
    );
}
