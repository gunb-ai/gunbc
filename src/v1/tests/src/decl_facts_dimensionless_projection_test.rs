//! Source-order invariance for decl_facts initializer projection marshalling.
//!
//! Merge-blocking projection behavior lives in `dag/test/claim/decl_facts_initializer_projection_witness_test.dag`.
//! This module retains only the seam those witnesses cannot reach: `compile_to_resolved` source-file
//! order must not change projection kind or constructor parent identity for the same specimen QN.
//!
//! That seam is SOURCE ORDER, not source discovery. The module used to obtain its source vector by
//! running the transitive closure over the whole of `src/v2` and `dag` -- twice -- and then reversing
//! the result. Whole-corpus closure assembly is not this test's subject; it belongs to the
//! compile-clean gate. Carrying it here made a source-order check depend on a ~1,100-module resolve,
//! and that resolve is what exhausted a stack segment and produced a 12-minute SIGSEGV.
//!
//! The fixture below is instead the smallest pair of modules that can carry the property: a
//! cross-module constructor (whose parent type is declared in the OTHER module, so a source-order
//! bug can misattribute it) and a local constructor (the control that stays put). Two modules is the
//! requirement plus one -- one module could not exhibit cross-module attribution at all.

use std::rc::Rc;

use im::HashMap;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_interpreter::{ExecutionMode, InterpContext, Value};

const CARRIER_MODULE: &str = "test.decl_facts_order.carrier";
const SPECIMEN_MODULE: &str = "test.decl_facts_order.specimens";

const CARRIER_SRC: &str = r#"
module test.decl_facts_order.carrier

type Disposition
  = Scaffold { dissolves_to: String }
  | Terminal { reason: String }
"#;

const SPECIMEN_SRC: &str = r#"
module test.decl_facts_order.specimens

type LocalCarrierA
  = TaggedA { tag: String }
  | OtherA

data cross_module_scaffold: test.decl_facts_order.carrier.Disposition = test.decl_facts_order.carrier.Scaffold { dissolves_to: "single-authority" }

data local_a_scaffold: LocalCarrierA = TaggedA { tag: "carrier-a" }
"#;

const CROSS_MODULE_SPECIMEN_QN: &str = "test.decl_facts_order.specimens.cross_module_scaffold";
const LOCAL_SPECIMEN_QN: &str = "test.decl_facts_order.specimens.local_a_scaffold";

fn ctx_from_sources(sources: Vec<Rc<SourceFile>>) -> InterpContext {
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    let graph = resolved
        .graph
        .as_ref()
        .unwrap_or_else(|| panic!("fixture should resolve to a graph"));
    InterpContext::new(graph, Rc::new(HashMap::new()), ExecutionMode::Hermetic)
}

fn source(path: &str, content: &str) -> Rc<SourceFile> {
    Rc::new(SourceFile {
        path: format!("{path}.dag"),
        content: content.to_string(),
    })
}

/// The identical source vector, in the two orders. Nothing else differs.
fn specimens_ctx_with_source_order(reversed: bool) -> InterpContext {
    let mut sources = vec![
        source(CARRIER_MODULE, CARRIER_SRC),
        source(SPECIMEN_MODULE, SPECIMEN_SRC),
    ];
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
                                Value::Str(s) => Some(s.to_string()),
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
                                    Some(Value::Str(s)) => s.as_ref(),
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
fn marshal_identity_is_invariant_under_reversed_source_order() {
    use v1_compiler::data_initializer_identity::marshal_data_initializer_projection;
    let forward = specimens_ctx_with_source_order(false);
    let reversed = specimens_ctx_with_source_order(true);

    // EXACT expected parents, asserted in BOTH orders -- not merely forward == reversed.
    // Equality alone is satisfied by two absent results, so a marshal that silently stopped
    // resolving constructor parents would report None == None and pass. Naming the expected
    // value is what makes an absent or misattributed parent fail.
    for (qn, expected_parent) in [
        (
            CROSS_MODULE_SPECIMEN_QN,
            "test.decl_facts_order.carrier.Disposition",
        ),
        (
            LOCAL_SPECIMEN_QN,
            "test.decl_facts_order.specimens.LocalCarrierA",
        ),
    ] {
        let forward_projection = marshal_data_initializer_projection(&forward, qn)
            .unwrap_or_else(|e| panic!("forward marshal {qn}: {e}"));
        let reversed_projection = marshal_data_initializer_projection(&reversed, qn)
            .unwrap_or_else(|e| panic!("reversed marshal {qn}: {e}"));

        let forward_parent = constructor_parent_qualified_name(&forward, &forward_projection);
        let reversed_parent = constructor_parent_qualified_name(&reversed, &reversed_projection);
        assert_eq!(
            forward_parent.as_deref(),
            Some(expected_parent),
            "forward order should attribute {qn} to {expected_parent}"
        );
        assert_eq!(
            reversed_parent.as_deref(),
            Some(expected_parent),
            "reversed order should attribute {qn} to {expected_parent}"
        );

        let forward_kind = projection_kind_lexeme(&forward, &forward_projection);
        assert!(
            forward_kind.is_some(),
            "projection kind should be present for {qn}, else the invariance below is vacuous"
        );
        assert_eq!(
            forward_kind,
            projection_kind_lexeme(&reversed, &reversed_projection),
            "projection kind must not depend on source file order for {qn}"
        );
    }
}
