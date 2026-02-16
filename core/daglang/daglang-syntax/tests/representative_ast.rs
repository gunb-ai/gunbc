use std::fs;
use std::path::{Path, PathBuf};

use daglang_syntax::ast::{Item, SourceFile, TypeBody};

fn dsl_file(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../dsl")
        .join(path)
}

// Test infrastructure: filesystem access for test fixtures
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
fn parse_dsl(path: &str) -> daglang_syntax::ast::SourceFile {
    let file = dsl_file(path);
    let source = fs::read_to_string(&file).expect("failed to read representative .dag file");
    daglang_syntax::parser::parse(&source)
        .unwrap_or_else(|errors| panic!("failed to parse {}: {errors:?}", file.display()))
}

fn item_signatures(source: &SourceFile) -> Vec<String> {
    source
        .items
        .iter()
        .map(|item| match &item.node {
            Item::TypeDef(def) => format!("type {}", def.name),
            Item::FnDef(def) => format!("fn {}", def.name),
            Item::FuncDef(def) => format!("func {}", def.name),
            Item::PatternDef(def) => format!("pattern {}", def.name),
            Item::ServiceDef(def) => format!("service {}", def.name),
            Item::ResourceDef(def) => format!("resource {}", def.name),
            Item::InterfaceDef(def) => format!("interface {}", def.name),
            Item::PipelineDef(def) => format!("pipeline {}", def.name),
        })
        .collect()
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
    assert_eq!(
        item_signatures(&source),
        vec!["fn render_makefile", "func makegen"]
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
    assert_eq!(
        item_signatures(&source),
        vec![
            "type CommitSha",
            "type RetryCount",
            "type HttpStatus",
            "type Email",
            "type Port",
            "type GistId",
            "type SecretValue",
            "type Url",
            "type FilePath",
            "type SemVer",
            "type NonEmptyStr",
            "type GitRef",
            "type ProjectId",
            "type ServiceAccountEmail",
            "type CloudRuntime",
            "type Platform",
            "type ContentEncoding",
            "type TextFilePath",
            "type BinaryFilePath",
            "type FileClassification",
            "type MimeType",
            "type AuthScheme",
            "type AccessToken",
            "type CloudSecretConfig",
            "type Credential",
            "type TestResult",
            "type Summary",
            "type StageResult",
            "type ToolEntry",
            "type ToolRegistry",
            "type DagTopology",
            "type TopologyNode",
            "type TopologyEdge",
            "type DagDiff",
            "type CodegenTarget",
            "type PragmaDirective",
            "type DocSource",
        ]
    );
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
    assert_eq!(
        item_signatures(&source),
        vec![
            "pattern file_content_matches",
            "pattern classify_files",
            "pattern read_text_files",
            "pattern acquire_subject_token",
            "pattern optional_impersonation",
            "pattern ensure",
            "pattern upsert",
            "pattern content_upsert",
            "pattern credential_chain",
            "pattern transaction",
            "pattern retry",
        ]
    );
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
    assert_eq!(
        item_signatures(&source),
        vec![
            "service gcloud.Auth",
            "service oauth2.Google",
            "service shell.Find",
            "service shell.Codegen",
            "service rustup.Component",
            "service shell.Which",
        ]
    );
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
    assert_eq!(
        item_signatures(&source),
        vec![
            "resource Filesystem",
            "resource Network",
            "resource Clock",
            "resource AuthContext",
        ]
    );
}
