//! **Layer:** integration
//!
//! SG-3b stack: lowering still reads `parse::Surface*`, while `parse_surface` is the
//! reflected mirror (`From<&parse::…>`). This pins that fixtures exercised through
//! the full compile boundary also round-trip structurally through the mirror so the
//! eventual spec-driven lowering handoff cannot silently fork surface facts.

use v3_compiler::parse_surface;
use v3_compiler::{
    compile_to_dag, default_fixed_point_source, parse_for_test,
    surface_top_level_let_names_for_test, tokenize_for_test,
};

fn top_level_let_names_mirror(m: &parse_surface::SurfaceModule) -> Vec<String> {
    m.items
        .iter()
        .filter_map(|item| match item {
            parse_surface::SurfaceItem::Let { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn lower_pipeline_fixture_aligns_with_parse_surface_mirror() {
    let file = "sg3_lower_parse_surface_stack.v3";
    let source = default_fixed_point_source();
    let tokens = tokenize_for_test(source, file).expect("tokenize");
    let parsed = parse_for_test(&tokens, file).expect("parse");
    let mirrored = parse_surface::SurfaceModule::from(&parsed);

    assert_eq!(
        mirrored.items.len(),
        parsed.items.len(),
        "parse_surface mirror must preserve top-level item count"
    );

    let hand_lets = surface_top_level_let_names_for_test(&parsed);
    let mirror_lets = top_level_let_names_mirror(&mirrored);
    assert_eq!(hand_lets, mirror_lets);
    assert_eq!(hand_lets, vec!["x".to_string(), "y".to_string()]);

    compile_to_dag(source, file).expect("lower + infer should succeed on the same surface");
}
