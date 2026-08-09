//! Source-order invariance for decl_facts initializer projection marshalling.
//!
//! Merge-blocking projection behavior lives in `dag/test/claim/decl_facts_initializer_projection_witness_test.dag`.
//! This module retains only the seam those witnesses cannot reach: `compile_to_resolved` source-file
//! order must not change projection kind or constructor parent identity for the same specimen QN.

use std::rc::Rc;

use im::HashMap;

use v1_compiler::data_initializer_identity::marshal_data_initializer_projection;
use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_interpreter::{ExecutionMode, InterpContext, Value};

use crate::helpers::{
    resolve_imports_transitively_with_source_roots, v2_layer_roots, workspace_root,
};

const SPECIMENS_REL: &str = "dag/test/fixture/decl_facts_reflection/specimens.dag";
const DISPOSITION_SCAFFOLD_QN: &str =
    "test.fixture.decl_facts_reflection.specimens.disposition_scaffold";
const LOCAL_A_SCAFFOLD_QN: &str = "test.fixture.decl_facts_reflection.specimens.local_a_scaffold";
const AMBIGUOUS_ARM_SPECIMEN_QN: &str =
    "test.fixture.decl_facts_reflection.ambiguous_specimen.ambiguous_arm_specimen";
const AMBIGUOUS_SHARED_A_SHARED_BARE_ARM_QN: &str =
    "test.fixture.decl_facts_reflection.ambiguous_shared_a.SharedBareArm";
const WITNESS_SUPPORT_REL: &str = "dag/test/claim/decl_facts_reflection_witness_support.dag";

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
    InterpContext::new(graph, Rc::new(HashMap::new()), ExecutionMode::Hermetic)
}

fn specimens_ctx_with_source_order(reversed: bool) -> InterpContext {
    let mut sources: Vec<Rc<SourceFile>> = resolve_imports_transitively_with_source_roots(
        SPECIMENS_REL,
        &read_fixture(SPECIMENS_REL),
        &v2_layer_roots(),
    );
    if reversed {
        sources.reverse();
    }
    ctx_from_sources(sources)
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

fn constructor_parent_qualified_name(ctx: &InterpContext, projection: &Value) -> Option<String> {
    let ctor = edge_target_named(ctx, projection, "constructor_identity")?;
    let parent = edge_target_named(ctx, &ctor, "parent_type")?;
    declaration_identity_qualified_name(ctx, &parent)
}

#[test]
fn eval_decl_facts_explicit_import_resolves_unique_variant_projection() {
    use v1_compiler::coproduct_reflection::eval_decl_facts;

    let sources = resolve_imports_transitively_with_source_roots(
        WITNESS_SUPPORT_REL,
        &read_fixture(WITNESS_SUPPORT_REL),
        &v2_layer_roots(),
    );
    let ctx = ctx_from_sources(sources);
    let roots = vec![workspace_root()
        .join("dag/test/fixture/decl_facts_reflection")
        .to_string_lossy()
        .into_owned()];
    let facts = eval_decl_facts(&ctx, &roots).expect("decl_facts must marshal");
    match facts {
        Value::List(rows) => {
            let node = rows
                .iter()
                .find_map(|row| {
                    let fields = match row {
                        Value::Record { fields, .. } => fields,
                        _ => return None,
                    };
                    let qn = ctx.field(fields, "qualified_name").and_then(|v| match v {
                        Value::Str(s) => Some(s.clone()),
                        _ => None,
                    })?;
                    if qn != AMBIGUOUS_ARM_SPECIMEN_QN {
                        return None;
                    }
                    ctx.field(fields, "node")
                })
                .expect("ambiguous specimen row");
            let projection = marshal_data_initializer_projection(&ctx, AMBIGUOUS_ARM_SPECIMEN_QN)
                .expect("direct marshal");
            let value_identity = edge_target_named(&ctx, &projection, "value_identity")
                .expect("value_identity edge");
            assert_eq!(
                projection_kind_lexeme(&ctx, &value_identity),
                Some("ResolvedVariantValueProjection".to_string()),
                "direct marshal value_identity kind"
            );
            let fact_value_identity =
                edge_target_named(&ctx, node, "value_identity").expect("decl_facts value_identity");
            assert_eq!(
                projection_kind_lexeme(&ctx, &fact_value_identity),
                Some("ResolvedVariantValueProjection".to_string()),
                "decl_facts value_identity must resolve to explicit import"
            );
            assert_eq!(
                constructor_parent_qualified_name(&ctx, &projection),
                Some(AMBIGUOUS_SHARED_A_SHARED_BARE_ARM_QN.to_string()),
                "constructor parent must be module A SharedBareArm"
            );
            assert_eq!(
                projection_kind_lexeme(&ctx, node),
                projection_kind_lexeme(&ctx, &projection),
                "decl_facts node must match direct marshal top-level kind"
            );
        }
        _ => panic!("expected list"),
    }
}

#[test]
fn explicit_import_resolves_unique_variant_projection() {
    let sources = resolve_imports_transitively_with_source_roots(
        WITNESS_SUPPORT_REL,
        &read_fixture(WITNESS_SUPPORT_REL),
        &v2_layer_roots(),
    );
    let ctx = ctx_from_sources(sources);
    let projection = marshal_data_initializer_projection(&ctx, AMBIGUOUS_ARM_SPECIMEN_QN)
        .expect("ambiguous specimen must marshal without error");
    assert_eq!(
        projection_kind_lexeme(&ctx, &projection),
        Some("DataInitializerNullaryProjection".to_string()),
        "top-level projection kind"
    );
    let value_identity = edge_target_named(&ctx, &projection, "value_identity")
        .expect("nullary projection must carry value_identity");
    assert_eq!(
        projection_kind_lexeme(&ctx, &value_identity),
        Some("ResolvedVariantValueProjection".to_string()),
        "explicit SharedBareArm import from ambiguous_shared_a must resolve uniquely"
    );
    assert_eq!(
        constructor_parent_qualified_name(&ctx, &projection),
        Some(AMBIGUOUS_SHARED_A_SHARED_BARE_ARM_QN.to_string()),
        "constructor parent must be the explicitly imported SharedBareArm from module A"
    );
}

#[test]
fn marshal_identity_is_invariant_under_reversed_source_order() {
    let forward = specimens_ctx_with_source_order(false);
    let reversed = specimens_ctx_with_source_order(true);
    for qn in [DISPOSITION_SCAFFOLD_QN, LOCAL_A_SCAFFOLD_QN] {
        let forward_projection = marshal_data_initializer_projection(&forward, qn)
            .unwrap_or_else(|e| panic!("forward marshal {qn}: {e}"));
        let reversed_projection = marshal_data_initializer_projection(&reversed, qn)
            .unwrap_or_else(|e| panic!("reversed marshal {qn}: {e}"));
        assert_eq!(
            projection_kind_lexeme(&forward, &forward_projection),
            projection_kind_lexeme(&reversed, &reversed_projection),
            "projection kind must not depend on source file order for {qn}"
        );
        assert_eq!(
            constructor_parent_qualified_name(&forward, &forward_projection),
            constructor_parent_qualified_name(&reversed, &reversed_projection),
            "constructor parent identity must not depend on source file order for {qn}"
        );
    }
}
