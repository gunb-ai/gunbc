//! Generic `decl_facts` data-init skeleton reflection prerequisite (step 1 / #7759 split).
//!
//! PARTIAL capability: outer record-constructor SPELLING (OuterRecordConstructorLexeme) only.
//! Exact parent-variant identity and nullary variant VALUE identity remain open.
//!
//! Developer convenience only — CI runs `cargo check` on this crate, not `cargo test`
//! (nextest retired 2026-07-11). Merge-blocking evidence lives in
//! `dag/test/claim/decl_facts_reflection_witness_test.dag`.

use std::collections::BTreeSet;
use std::rc::Rc;

use im::HashMap;
use v1_compiler::coproduct_reflection::eval_decl_facts;
use v1_compiler::v1_compiler_infer_emit_info::empty_emit_graph_info;
use v1_compiler::v1_compiler_infer_items::ResolvedGraph;
use v1_compiler::v1_interpreter::{ExecutionMode, InterpContext, Value};

use crate::helpers::workspace_root;

const FIXTURE_POOL: &str = "dag/test/fixture/decl_facts_reflection";

const QN_SCAFFOLD: &str = "test.fixture.decl_facts_reflection.specimens.planted_scaffold_specimen";
const QN_TERMINAL: &str = "test.fixture.decl_facts_reflection.specimens.planted_terminal_specimen";
const QN_NULLARY: &str =
    "test.fixture.decl_facts_reflection.specimens.planted_nullary_disposition_specimen";
const QN_NAMED_FIELD_REF: &str =
    "test.fixture.decl_facts_reflection.specimens.planted_named_field_ref_specimen";
const QN_PLAIN_INT: &str =
    "test.fixture.decl_facts_reflection.specimens.planted_plain_int_specimen";

fn wet_ctx() -> InterpContext {
    let graph = ResolvedGraph {
        modules: Rc::new(im::vector![]),
        item_registry: Rc::new(HashMap::new()),
        diagnostics: Rc::new(im::vector![]),
        emit_graph_info: empty_emit_graph_info(),
    };
    InterpContext::new(&graph, Rc::new(HashMap::new()), ExecutionMode::Wet)
}

fn fixture_roots() -> Vec<String> {
    vec![workspace_root()
        .join(FIXTURE_POOL)
        .to_string_lossy()
        .into_owned()]
}

fn collect_atom_identities(val: &Value, ctx: &InterpContext, out: &mut BTreeSet<String>) {
    match val {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            if ctx.sym_eq(*variant_name, "Atom") {
                if let Some((_, Value::Str(identity))) =
                    fields.iter().find(|(k, _)| ctx.sym_eq(*k, "identity"))
                {
                    out.insert(identity.to_string());
                }
            }
            for (_, v) in fields.iter() {
                collect_atom_identities(v, ctx, out);
            }
        }
        Value::Record { fields, .. } => {
            for (_, v) in fields.iter() {
                collect_atom_identities(v, ctx, out);
            }
        }
        Value::List(items) => {
            for v in items.iter() {
                collect_atom_identities(v, ctx, out);
            }
        }
        _ => {}
    }
}

fn count_atom_identity(val: &Value, ctx: &InterpContext, identity: &str) -> usize {
    let mut count = 0usize;
    fn walk(val: &Value, ctx: &InterpContext, identity: &str, count: &mut usize) {
        match val {
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                if ctx.sym_eq(*variant_name, "Atom")
                    && fields.iter().any(|(k, v)| {
                        ctx.sym_eq(*k, "identity")
                            && matches!(v, Value::Str(s) if s.as_str() == identity)
                    })
                {
                    *count += 1;
                }
                for (_, v) in fields.iter() {
                    walk(v, ctx, identity, count);
                }
            }
            Value::Record { fields, .. } => {
                for (_, v) in fields.iter() {
                    walk(v, ctx, identity, count);
                }
            }
            Value::List(items) => {
                for v in items.iter() {
                    walk(v, ctx, identity, count);
                }
            }
            _ => {}
        }
    }
    walk(val, ctx, identity, &mut count);
    count
}

fn value_contains_atom_identity(val: &Value, ctx: &InterpContext, identity: &str) -> bool {
    count_atom_identity(val, ctx, identity) > 0
}

fn decl_fact_node_skeleton(
    ctx: &InterpContext,
    rows: &Value,
    qualified_name: &str,
) -> Option<Value> {
    let items = match rows {
        Value::List(items) => items,
        other => panic!("expected decl_facts List, got {other:?}"),
    };
    for row in items.iter() {
        let fields = match row {
            Value::Record { fields, .. } => fields.as_ref(),
            other => panic!("expected DeclFact Record, got {other:?}"),
        };
        let qn = match ctx.field(fields, "qualified_name") {
            Some(Value::Str(s)) => s.as_str(),
            other => panic!("expected qualified_name Str, got {other:?}"),
        };
        if qn != qualified_name {
            continue;
        }
        return ctx.field(fields, "node").cloned();
    }
    None
}

fn fixture_decl_facts(ctx: &InterpContext) -> Value {
    eval_decl_facts(ctx, &fixture_roots()).expect("eval_decl_facts")
}

#[test]
fn outer_record_constructor_lexeme_scaffold_specimen() {
    let ctx = wet_ctx();
    let skeleton = decl_fact_node_skeleton(&ctx, &fixture_decl_facts(&ctx), QN_SCAFFOLD)
        .expect("missing scaffold specimen");
    assert!(
        value_contains_atom_identity(&skeleton, &ctx, "Scaffold"),
        "Scaffold outer record-constructor lexeme must appear in skeleton"
    );
    assert_eq!(
        count_atom_identity(&skeleton, &ctx, "Scaffold"),
        1,
        "Scaffold constructor lexeme must appear exactly once (no duplicate emission)"
    );
}

#[test]
fn outer_record_constructor_lexeme_terminal_specimen() {
    let ctx = wet_ctx();
    let skeleton = decl_fact_node_skeleton(&ctx, &fixture_decl_facts(&ctx), QN_TERMINAL)
        .expect("missing terminal specimen");
    assert!(
        value_contains_atom_identity(&skeleton, &ctx, "Terminal"),
        "Terminal outer record-constructor lexeme must appear in skeleton"
    );
}

#[test]
fn nullary_variant_value_absent_without_infer_stamping_nested() {
    let ctx = wet_ctx();
    let skeleton = decl_fact_node_skeleton(&ctx, &fixture_decl_facts(&ctx), QN_SCAFFOLD)
        .expect("missing scaffold specimen");
    assert!(
        !value_contains_atom_identity(&skeleton, &ctx, "SingleAuthority"),
        "parse-only marshal must not invent nullary variant value atoms"
    );
}

#[test]
fn nullary_variant_value_absent_without_infer_stamping_top_level() {
    let ctx = wet_ctx();
    let skeleton = decl_fact_node_skeleton(&ctx, &fixture_decl_facts(&ctx), QN_NULLARY)
        .expect("missing nullary disposition specimen");
    assert!(
        !value_contains_atom_identity(&skeleton, &ctx, "SingleAuthority"),
        "parse-only marshal must not invent nullary variant value atoms"
    );
}

#[test]
fn declaration_ref_fields_present_in_scaffold_bind() {
    let ctx = wet_ctx();
    let skeleton = decl_fact_node_skeleton(&ctx, &fixture_decl_facts(&ctx), QN_SCAFFOLD)
        .expect("missing scaffold specimen");
    assert!(
        value_contains_atom_identity(&skeleton, &ctx, "DeclarationRef"),
        "DeclarationRef constructor must appear in bind conj"
    );
    assert!(
        value_contains_atom_identity(
            &skeleton,
            &ctx,
            "test.fixture.decl_facts_reflection.specimens"
        ),
        "module_path literal must appear in bind conj"
    );
    assert!(
        value_contains_atom_identity(&skeleton, &ctx, "planted_scaffold_specimen"),
        "decl_name literal must appear in bind conj"
    );
}

#[test]
fn named_field_declaration_ref_fields_present() {
    let ctx = wet_ctx();
    let skeleton = decl_fact_node_skeleton(&ctx, &fixture_decl_facts(&ctx), QN_NAMED_FIELD_REF)
        .expect("missing named-field ref specimen");
    assert!(
        value_contains_atom_identity(&skeleton, &ctx, "NamedField"),
        "NamedField constructor must appear"
    );
    assert!(
        value_contains_atom_identity(&skeleton, &ctx, "dissolves_to"),
        "field_name literal must appear"
    );
}

#[test]
fn plain_int_specimen_has_no_coproduct_variant_atoms() {
    let ctx = wet_ctx();
    let skeleton = decl_fact_node_skeleton(&ctx, &fixture_decl_facts(&ctx), QN_PLAIN_INT)
        .expect("missing plain int specimen");
    let mut atoms = BTreeSet::new();
    collect_atom_identities(&skeleton, &ctx, &mut atoms);
    assert!(
        !atoms.contains("Scaffold")
            && !atoms.contains("Terminal")
            && !atoms.contains("SingleAuthority")
            && !atoms.contains("DeclarationRef"),
        "plain Int literal must not gain coproduct variant atoms, got {atoms:?}"
    );
}

#[test]
fn fn_body_does_not_gain_variant_atoms_param_only() {
    let ctx = wet_ctx();
    let rows = fixture_decl_facts(&ctx);
    let skeleton = decl_fact_node_skeleton(
        &ctx,
        &rows,
        "test.fixture.decl_facts_reflection.specimens.uses_param_only",
    )
    .expect("missing uses_param_only");
    assert!(
        !value_contains_atom_identity(&skeleton, &ctx, "Scaffold")
            && !value_contains_atom_identity(&skeleton, &ctx, "SingleAuthority"),
        "fn body must not gain variant atoms from param-only reference"
    );
}

#[test]
fn fn_body_does_not_gain_variant_atoms_local_bindings() {
    let ctx = wet_ctx();
    let rows = fixture_decl_facts(&ctx);
    let skeleton = decl_fact_node_skeleton(
        &ctx,
        &rows,
        "test.fixture.decl_facts_reflection.specimens.uses_local",
    )
    .expect("missing uses_local");
    assert!(
        !value_contains_atom_identity(&skeleton, &ctx, "Scaffold")
            && !value_contains_atom_identity(&skeleton, &ctx, "SingleAuthority"),
        "fn body must not gain variant atoms from local bindings"
    );
}
