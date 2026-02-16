use std::fs;
use std::path::{Path, PathBuf};

use daglang_syntax::ast::{Item, TypeBody};

fn dsl_file(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../dsl")
        .join(path)
}

fn parse_dsl(path: &str) -> daglang_syntax::ast::SourceFile {
    let file = dsl_file(path);
    let source = fs::read_to_string(&file).expect("failed to read representative .dag file");
    daglang_syntax::parser::parse(&source)
        .unwrap_or_else(|errors| panic!("failed to parse {}: {errors:?}", file.display()))
}

#[test]
fn makegen_contains_fn_and_func_items() {
    let source = parse_dsl("tools/makegen.dag");
    assert_eq!(source.items.len(), 2, "makegen should contain 2 top-level items");
    assert_eq!(
        source.module_path.as_ref().map(|module| module.node.segments.clone()),
        Some(vec!["tools".into(), "makegen".into()])
    );
    assert!(source
        .items
        .iter()
        .any(|item| matches!(item.node, Item::FnDef(_))));
    assert!(source
        .items
        .iter()
        .any(|item| matches!(item.node, Item::FuncDef(_))));

    let render_fn = source
        .items
        .iter()
        .find_map(|item| match &item.node {
            Item::FnDef(def) if def.name == "render_makefile" => Some(def),
            _ => None,
        })
        .expect("render_makefile fn should exist");
    assert!(
        !render_fn.body.stmts.is_empty(),
        "render_makefile body should retain parsed statements"
    );

    let makegen_func = source
        .items
        .iter()
        .find_map(|item| match &item.node {
            Item::FuncDef(def) if def.name == "makegen" => Some(def),
            _ => None,
        })
        .expect("makegen func should exist");
    assert!(
        !makegen_func.body.stmts.is_empty(),
        "makegen func body should retain parsed statements"
    );
}

#[test]
fn types_file_contains_record_sum_and_alias_definitions() {
    let source = parse_dsl("std/types.dag");
    assert_eq!(source.items.len(), 37, "std/types.dag item count changed unexpectedly");
    assert_eq!(
        source.module_path.as_ref().map(|module| module.node.segments.clone()),
        Some(vec!["std".into(), "types".into()])
    );

    let mut saw_record = false;
    let mut saw_sum = false;
    let mut saw_alias = false;
    for item in &source.items {
        if let Item::TypeDef(typedef) = &item.node {
            match typedef.body {
                TypeBody::Record(_) => saw_record = true,
                TypeBody::Sum(_) => saw_sum = true,
                TypeBody::Alias(_) => saw_alias = true,
            }
        }
    }

    assert!(saw_record, "expected at least one record type definition");
    assert!(saw_sum, "expected at least one sum type definition");
    assert!(saw_alias, "expected at least one alias type definition");
}

#[test]
fn patterns_file_contains_pattern_defs() {
    let source = parse_dsl("std/patterns.dag");
    assert_eq!(
        source.items.len(),
        11,
        "std/patterns.dag item count changed unexpectedly"
    );
    assert_eq!(
        source.module_path.as_ref().map(|module| module.node.segments.clone()),
        Some(vec!["std".into(), "patterns".into()])
    );
    assert!(source
        .items
        .iter()
        .any(|item| matches!(item.node, Item::PatternDef(_))));
}

#[test]
fn shell_service_file_contains_service_defs() {
    let source = parse_dsl("services/shell.dag");
    assert_eq!(
        source.items.len(),
        6,
        "services/shell.dag item count changed unexpectedly"
    );
    assert_eq!(
        source.module_path.as_ref().map(|module| module.node.segments.clone()),
        Some(vec!["services".into(), "shell".into()])
    );
    assert!(source
        .items
        .iter()
        .any(|item| matches!(item.node, Item::ServiceDef(_))));
}

#[test]
fn resources_file_contains_resource_defs() {
    let source = parse_dsl("std/resources.dag");
    assert_eq!(
        source.items.len(),
        4,
        "std/resources.dag item count changed unexpectedly"
    );
    assert_eq!(
        source.module_path.as_ref().map(|module| module.node.segments.clone()),
        Some(vec!["std".into(), "resources".into()])
    );
    assert!(source
        .items
        .iter()
        .any(|item| matches!(item.node, Item::ResourceDef(_))));
}
