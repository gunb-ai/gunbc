//! Projection authority controls for #7855 (loyal-ant-382):
//! - dimensionless: imported RECORD type resolves via type env → plain record arm (not coproduct)
//! - unimported globally-unique type: constructor resolution refused (not error, not absent)

use std::rc::Rc;

use v1_compiler::data_initializer_identity::marshal_data_initializer_projection;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_interpreter::{ExecutionMode, InterpContext, Value};

use crate::helpers::{
    resolve_imports_transitively_with_source_roots, v2_layer_roots, workspace_root,
};

const DIMENSIONLESS_REL: &str = "dag/extdeps/units/dimensionless.dag";
const ANCHOR_QN: &str = "extdeps.units.dimensionless.extdeps_external_authority_anchor";

const GLOBALLY_UNIQUE_CARRIER_REL: &str =
    "dag/test/fixture/decl_facts_reflection/globally_unique_carrier.dag";
const UNIMPORTED_USER_REL: &str =
    "dag/test/fixture/decl_facts_reflection/unimported_globally_unique_user.dag";
const UNIMPORTED_SPECIMEN_QN: &str =
    "test.fixture.decl_facts_reflection.unimported_globally_unique_user.unimported_globally_unique_specimen";

fn read_fixture(rel: &str) -> String {
    std::fs::read_to_string(workspace_root().join(rel))
        .unwrap_or_else(|e| panic!("read {rel}: {e}"))
}

fn ctx_from_sources(sources: Vec<Rc<SourceFile>>) -> InterpContext {
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    let graph = resolved
        .graph
        .as_ref()
        .expect("fixture closure resolves to a graph");
    InterpContext::new(graph, Rc::new(im::HashMap::new()), ExecutionMode::Hermetic)
}

fn dimensionless_ctx() -> InterpContext {
    let sources: Vec<Rc<SourceFile>> = resolve_imports_transitively_with_source_roots(
        DIMENSIONLESS_REL,
        &read_fixture(DIMENSIONLESS_REL),
        &v2_layer_roots(),
    );
    ctx_from_sources(sources)
}

fn unimported_globally_unique_ctx() -> InterpContext {
    ctx_from_sources(vec![
        Rc::new(SourceFile {
            path: GLOBALLY_UNIQUE_CARRIER_REL.to_string(),
            content: read_fixture(GLOBALLY_UNIQUE_CARRIER_REL),
        }),
        Rc::new(SourceFile {
            path: UNIMPORTED_USER_REL.to_string(),
            content: read_fixture(UNIMPORTED_USER_REL),
        }),
    ])
}

fn projection_kind_lexeme(ctx: &InterpContext, projection: &Value) -> Option<String> {
    match projection {
        Value::Record { fields, .. } => {
            let kind_key = ctx.sym("kind");
            let connective_key = ctx.sym("connective");
            let identity_key = ctx.sym("identity");
            let kind = fields
                .iter()
                .find(|(k, _)| *k == kind_key)
                .map(|(_, v)| v)?;
            match kind {
                Value::Variant {
                    fields: kind_fields,
                    ..
                } => {
                    let connective = kind_fields
                        .iter()
                        .find(|(k, _)| *k == connective_key)
                        .map(|(_, v)| v)?;
                    match connective {
                        Value::Variant {
                            fields: conn_fields,
                            ..
                        } => conn_fields
                            .iter()
                            .find(|(k, _)| *k == identity_key)
                            .and_then(|(_, v)| match v {
                                Value::Str(s) => Some(s.clone()),
                                _ => None,
                            }),
                        _ => None,
                    }
                }
                _ => None,
            }
        }
        _ => None,
    }
}

fn edge_target_named(ctx: &InterpContext, projection: &Value, label: &str) -> Option<Value> {
    match projection {
        Value::Record { fields, .. } => {
            let children = match ctx.field(fields, "children") {
                Some(Value::List(items)) => items,
                _ => return None,
            };
            for edge in children.iter() {
                match edge {
                    Value::Record {
                        fields: edge_fields,
                        ..
                    } => {
                        let edge_label = match ctx.field(edge_fields, "label") {
                            Some(Value::Variant {
                                variant_name,
                                fields: label_fields,
                                ..
                            }) => {
                                if *variant_name != ctx.sym("Named") {
                                    continue;
                                }
                                match ctx.field(label_fields, "name") {
                                    Some(Value::Str(s)) => s.as_str(),
                                    _ => continue,
                                }
                            }
                            _ => continue,
                        };
                        if edge_label != label {
                            continue;
                        }
                        return ctx.field(edge_fields, "target").cloned().map(|v| v.clone());
                    }
                    _ => {}
                }
            }
            None
        }
        _ => None,
    }
}

fn declaration_identity_qualified_name(ctx: &InterpContext, projection: &Value) -> Option<String> {
    let qn_projection = edge_target_named(ctx, projection, "qualified_name")?;
    projection_kind_lexeme(ctx, &qn_projection)
}

const SPECIMENS_REL: &str = "dag/test/fixture/decl_facts_reflection/specimens.dag";
const DISPOSITION_SCAFFOLD_QN: &str =
    "test.fixture.decl_facts_reflection.specimens.disposition_scaffold";

fn specimens_ctx() -> InterpContext {
    let sources: Vec<Rc<SourceFile>> = resolve_imports_transitively_with_source_roots(
        SPECIMENS_REL,
        &read_fixture(SPECIMENS_REL),
        &v2_layer_roots(),
    );
    ctx_from_sources(sources)
}

#[test]
#[ignore = "whole-tree resolve (~minutes); run with: cargo test witness_layer_decl_facts_disposition_scaffold -- --ignored --nocapture"]
fn witness_layer_decl_facts_disposition_scaffold_fact_is_coproduct_record() {
    use v1_compiler::cli_run::whole_tree_resolved_ctx;
    use v1_compiler::coproduct_reflection::eval_decl_facts;

    let roots = vec!["dag".to_string(), "src/v2".to_string()];
    let whole = whole_tree_resolved_ctx(&roots, &[], ExecutionMode::Hermetic)
        .expect("witness layer whole-tree resolve");
    let facts = eval_decl_facts(&whole.ctx, &roots).expect("decl_facts must complete");
    let qn_key = whole.ctx.sym("qualified_name");
    let node_key = whole.ctx.sym("node");
    let mut found = false;
    match &facts {
        Value::List(rows) => {
            for row in rows.iter() {
                let fields = match row {
                    Value::Record { fields, .. } => fields,
                    _ => continue,
                };
                let qn = match fields.iter().find(|(k, _)| *k == qn_key) {
                    Some((_, Value::Str(s))) => s.as_str(),
                    _ => continue,
                };
                if qn != DISPOSITION_SCAFFOLD_QN {
                    continue;
                }
                found = true;
                let node = fields
                    .iter()
                    .find(|(k, _)| *k == node_key)
                    .map(|(_, v)| v)
                    .expect("DeclFact.node");
                assert_eq!(
                    projection_kind_lexeme(&whole.ctx, node),
                    Some("DataInitializerRecordProjection".to_string()),
                    "witness-layer decl_facts must marshal disposition_scaffold as coproduct record"
                );
            }
        }
        _ => panic!("expected list"),
    }
    assert!(
        found,
        "disposition_scaffold must appear in witness-layer decl_facts"
    );
}

#[test]
fn disposition_scaffold_marshals_coproduct_record_projection() {
    let ctx = specimens_ctx();
    let projection = marshal_data_initializer_projection(&ctx, DISPOSITION_SCAFFOLD_QN)
        .expect("disposition scaffold must marshal without error");
    assert_eq!(
        projection_kind_lexeme(&ctx, &projection),
        Some("DataInitializerRecordProjection".to_string()),
        "imported coproduct initializer must use coproduct record projection"
    );
}

#[test]
fn dimensionless_imported_external_authority_marshals_defining_module_parent() {
    let ctx = dimensionless_ctx();
    let projection = marshal_data_initializer_projection(&ctx, ANCHOR_QN)
        .expect("marshal must not throw on resolvable imported-type initializer");
    assert_eq!(
        projection_kind_lexeme(&ctx, &projection),
        Some("DataInitializerPlainRecordProjection".to_string()),
        "expected plain record projection for imported ExternalAuthority initializer"
    );
    assert_ne!(
        projection_kind_lexeme(&ctx, &projection),
        Some("DataInitializerRecordProjection".to_string()),
        "imported record type must not be classified into the coproduct record arm"
    );
    assert_eq!(
        declaration_identity_qualified_name(
            &ctx,
            &edge_target_named(&ctx, &projection, "parent_type")
                .expect("plain record projection must carry parent_type edge")
        ),
        Some("extdeps.external_authority.ExternalAuthority".to_string()),
        "parent type identity must name the defining module, not the importing module"
    );
}

#[test]
fn unimported_globally_unique_type_marshals_constructor_refused_not_error() {
    let ctx = unimported_globally_unique_ctx();
    let projection = marshal_data_initializer_projection(&ctx, UNIMPORTED_SPECIMEN_QN)
        .expect("unimported type in initializer must refuse with projection, never throw");
    assert_eq!(
        projection_kind_lexeme(&ctx, &projection),
        Some("DataInitializerConstructorResolutionRefusedProjection".to_string()),
        "type not visible in importing module type env must refuse constructor resolution"
    );
}

/// Whole-tree census for loyal-ant-382 blast-radius reporting. CI exercises the same
/// `decl_facts(witness_layer_roots)` path via `decl_facts_reflection_witness_test.dag`.
#[test]
#[ignore = "whole-tree resolve (~minutes); run with: cargo test witness_layer_decl_facts_projection_census -- --ignored --nocapture"]
fn witness_layer_decl_facts_projection_census() {
    use v1_compiler::cli_run::whole_tree_resolved_ctx;
    use v1_compiler::coproduct_reflection::eval_decl_facts;

    let roots = vec!["dag".to_string(), "src/v2".to_string()];
    let whole = whole_tree_resolved_ctx(&roots, &[], ExecutionMode::Hermetic)
        .expect("witness layer whole-tree resolve");
    let facts = eval_decl_facts(&whole.ctx, &roots).expect("decl_facts must marshal without error");

    let mut plain_record = 0usize;
    let mut coproduct_record = 0usize;
    let mut constructor_refused = 0usize;
    let mut typechecked_absent = 0usize;
    let mut other = 0usize;

    match &facts {
        Value::List(rows) => {
            for row in rows.iter() {
                let node = whole
                    .ctx
                    .field(
                        match row {
                            Value::Record { fields, .. } => fields,
                            _ => panic!("expected DeclFact record"),
                        },
                        "node",
                    )
                    .expect("DeclFact.node");
                match projection_kind_lexeme(&whole.ctx, node) {
                    Some(k) if k == "DataInitializerPlainRecordProjection" => plain_record += 1,
                    Some(k) if k == "DataInitializerRecordProjection" => coproduct_record += 1,
                    Some(k) if k == "DataInitializerConstructorResolutionRefusedProjection" => {
                        constructor_refused += 1
                    }
                    Some(k) if k == "DataInitializerTypecheckedSubjectAbsentProjection" => {
                        typechecked_absent += 1
                    }
                    _ => other += 1,
                }
            }
        }
        other => panic!("expected decl_facts list, got {other:?}"),
    }

    eprintln!(
        "witness_layer decl_facts projection census: plain_record={plain_record} coproduct_record={coproduct_record} constructor_refused={constructor_refused} typechecked_absent={typechecked_absent} other={other}"
    );
    assert!(
        plain_record > 0,
        "imported record initializers must marshal as plain record projections"
    );
    assert!(
        constructor_refused > 0,
        "unimported types must route to constructor resolution refused"
    );
}
