use std::fs;
use std::path::{Path, PathBuf};

use daglang_syntax::ast::Item;

// Test infrastructure: filesystem access for test fixtures
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
fn collect_dag_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("dag") {
            out.push(path);
        }
    }
    Ok(())
}

// Test infrastructure: filesystem access for test fixtures
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[test]
fn corpus_covers_all_top_level_item_variants() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
    let mut dag_files = Vec::new();
    collect_dag_files(&dsl_root, &mut dag_files).expect("failed to discover .dag files");
    dag_files.sort();

    let mut saw_type = false;
    let mut saw_fn = false;
    let mut saw_func = false;
    let mut saw_pattern = false;
    let mut saw_service = false;
    let mut saw_resource = false;
    let mut saw_interface = false;
    let mut saw_pipeline = false;

    for path in dag_files {
        let source = fs::read_to_string(&path).expect("failed to read .dag source");
        let ast = daglang_syntax::parser::parse(&source).unwrap_or_else(|errors| {
            panic!("failed to parse {} with errors {errors:?}", path.display())
        });
        for item in ast.items {
            match item.node {
                Item::TypeDef(_) => saw_type = true,
                Item::FnDef(_) => saw_fn = true,
                Item::FuncDef(_) => saw_func = true,
                Item::PatternDef(_) => saw_pattern = true,
                Item::ServiceDef(_) => saw_service = true,
                Item::ResourceDef(_) => saw_resource = true,
                Item::InterfaceDef(_) => saw_interface = true,
                Item::PipelineDef(_) => saw_pipeline = true,
                Item::TestDef(_) | Item::FixtureDef(_) => {}
                Item::ProjectDef(_)
                | Item::FeatureDef(_)
                | Item::TaskDef(_)
                | Item::DesignDef(_)
                | Item::ComponentDef(_)
                | Item::EnvironmentDef(_) => {}
            }
        }
    }

    assert!(saw_type, "expected at least one type definition in corpus");
    assert!(saw_fn, "expected at least one fn definition in corpus");
    assert!(saw_func, "expected at least one func definition in corpus");
    assert!(
        saw_pattern,
        "expected at least one pattern definition in corpus"
    );
    assert!(
        saw_service,
        "expected at least one service definition in corpus"
    );
    assert!(
        saw_resource,
        "expected at least one resource definition in corpus"
    );
    assert!(
        saw_interface,
        "expected at least one interface definition in corpus"
    );
    assert!(
        saw_pipeline,
        "expected at least one pipeline definition in corpus"
    );
}
