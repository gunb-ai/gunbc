// Ratchet test: counts total `Annotation` nodes across the parsed AST of all
// `.dag` files. The baseline decreases as files are migrated from annotations
// to typed syntax (where clauses, transport blocks, behavioral keywords, etc.).
// When the baseline reaches 0, annotations can be deleted from the compiler.

use std::path::Path;

#[allow(dead_code)]
mod common;

use common::collect_dag_files;
use daglang_syntax::ast::*;
use daglang_syntax::parser;

/// Current annotation baseline. Decrease this as .dag files are migrated.
const ANNOTATION_BASELINE: usize = 0;

fn count_annotations_in_type_expr(ty: &TypeExpr) -> usize {
    match ty {
        TypeExpr::Annotated(inner, anns) => anns.len() + count_annotations_in_type_expr(inner),
        TypeExpr::Optional(inner) => count_annotations_in_type_expr(inner),
        TypeExpr::Refined(inner, _) => count_annotations_in_type_expr(inner),
        TypeExpr::Generic(_, args) => args.iter().map(count_annotations_in_type_expr).sum(),
        TypeExpr::Record(fields) => count_annotations_in_fields(fields),
        TypeExpr::Named(_) => 0,
    }
}

fn count_annotations_in_fields(fields: &[Field]) -> usize {
    fields
        .iter()
        .map(|f| f.annotations.len() + count_annotations_in_type_expr(&f.ty))
        .sum()
}

fn count_annotations_in_item(item: &Item) -> usize {
    match item {
        Item::ServiceDef(s) => {
            s.annotations.len()
                + s.operations
                    .iter()
                    .map(|op| {
                        op.annotations.len()
                            + count_annotations_in_fields(&op.inputs)
                            + count_annotations_in_fields(&op.outputs)
                    })
                    .sum::<usize>()
        }
        Item::FuncDef(f) => {
            f.annotations.len()
                + count_annotations_in_fields(&f.outputs)
        }
        Item::InterfaceDef(i) => {
            i.contracts.len()
                + i.capabilities
                    .iter()
                    .map(|c| {
                        c.annotations.len()
                            + count_annotations_in_fields(&c.inputs)
                            + count_annotations_in_fields(&c.outputs)
                    })
                    .sum::<usize>()
        }
        Item::TestDef(t) => t.annotations.len(),
        Item::ResourceDef(r) => {
            r.capabilities
                .iter()
                .map(|c| {
                    c.annotations.len()
                        + count_annotations_in_fields(&c.inputs)
                        + count_annotations_in_fields(&c.outputs)
                })
                .sum::<usize>()
                + count_annotations_in_fields(&r.config)
        }
        Item::TypeDef(t) => match &t.body {
            TypeBody::Alias(ty) => count_annotations_in_type_expr(ty),
            TypeBody::Record(fields) => count_annotations_in_fields(fields),
            TypeBody::Sum(variants) => variants
                .iter()
                .map(|v| count_annotations_in_fields(&v.fields))
                .sum(),
        },
        Item::ExternFuncDecl(e) => {
            e.annotations.len()
                + count_annotations_in_fields(&e.inputs)
                + count_annotations_in_fields(&e.outputs)
        }
        Item::ExternAssetDecl(e) => {
            e.annotations.len() + count_annotations_in_type_expr(&e.ty)
        }
        Item::FnDef(_)
        | Item::PatternDef(_)
        | Item::PipelineDef(_)
        | Item::ProfileDef(_)
        | Item::FixtureDef(_)
        | Item::ProjectDef(_)
        | Item::FeatureDef(_)
        | Item::TaskDef(_)
        | Item::DesignDef(_)
        | Item::ComponentDef(_)
        | Item::EnvironmentDef(_)
        | Item::ParamDecl(_)
        | Item::DataDef(_) => 0,
    }
}

// Test infrastructure: filesystem access for test fixtures
#[allow(clippy::disallowed_methods, clippy::disallowed_types)]
#[test]
fn annotation_ratchet() {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
    let mut dag_files = Vec::new();
    collect_dag_files(&dsl_root, &mut dag_files).expect("failed to discover .dag files");
    dag_files.sort();

    let mut total = 0usize;
    for file in &dag_files {
        let source =
            std::fs::read_to_string(file).expect("failed to read .dag source");
        let parsed = parser::parse(&source)
            .unwrap_or_else(|errors| panic!("failed to parse {}: {errors:?}", file.display()));

        let file_count: usize = parsed.items.iter().map(|s| count_annotations_in_item(&s.node)).sum();
        total += file_count;
    }

    assert!(
        total <= ANNOTATION_BASELINE,
        "Annotation count {total} exceeds baseline {ANNOTATION_BASELINE}. \
         Migrate to typed syntax (where clauses, transport blocks, behavioral keywords) \
         instead of adding annotations."
    );

    // Print the count for visibility during migration
    eprintln!("annotation_ratchet: {total} annotations (baseline: {ANNOTATION_BASELINE})");
}
