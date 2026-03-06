use gunbc_app::makegen::{
    model::{load_build_targets_data, reserved_target_names},
    shared::render_justfile,
    tools::{discover_makegen_tools, filter_reserved_tools, DiscoveredToolData},
};
use gunbc_app::render_makefile;

fn non_colliding_tools() -> Vec<DiscoveredToolData> {
    let tools = discover_makegen_tools().expect("tool discovery should succeed");
    let build_targets = load_build_targets_data().expect("build target model should load");
    filter_reserved_tools(&tools, &reserved_target_names(&build_targets))
}

#[test]
fn render_makefile_matches_golden_fixture() {
    let tools = non_colliding_tools();
    let rendered = render_makefile(&tools).expect("render makefile");
    let expected = include_str!("fixtures/makefile_golden.txt");

    assert_eq!(rendered, expected);
}

#[test]
fn render_justfile_uses_leaf_serializer_contract() {
    let tools = non_colliding_tools();
    let rendered = render_justfile(&tools).expect("render justfile");
    let expected = include_str!("fixtures/justfile_golden.txt");

    assert_eq!(rendered, expected);
    assert!(
        !rendered.contains('\t'),
        "Justfile output should use space indentation, not Makefile tabs"
    );
}
