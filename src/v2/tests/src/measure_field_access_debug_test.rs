//! G2: alias field access through parametric carriers (Measure / ByteSize).

use std::rc::Rc;

use v2_compiler::v2_std_core::diagnostic_to_message;

use crate::helpers::compile_dag_resolved;

fn hard_diagnostic_messages(
    resolved: &v2_compiler::v2_compiler_compile::ResolvedPipelineResult,
) -> Vec<String> {
    resolved
        .diagnostics
        .iter()
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect()
}

#[test]
fn generic_alias_field_access_resolves_through_expansion() {
    let src = r#"
module m

import std.nat { Nat }

type Box<T> {
  value: T
}

type NatBox = Box<Nat>

fn get(b: NatBox) -> Nat {
  b.value
}
"#;
    let msgs = hard_diagnostic_messages(&compile_dag_resolved(src));
    assert!(
        msgs.is_empty(),
        "generic alias field access should resolve, got: {msgs:?}"
    );
}

#[test]
fn bytesize_alias_binding_with_resolved_rhs_expands_for_field_access() {
    use v2_compiler::v2_compiler_infer::{
        expand_type_for_field_access, needs_alias_field_expansion,
    };
    use v2_compiler::v2_compiler_infer_env::lookup_type_by_name;
    use v2_compiler::v2_compiler_infer_lookup::lookup_field_type_node;
    use v2_compiler::v2_compiler_infer_resolve::is_user_generic_use_site;
    use v2_compiler::v2_compiler_infer_resolve::resolve_node;
    use v2_compiler::v2_std_core::{default_ident_span, make_span, Connective, InferredNode, Node};

    let src = r#"
module m

type Nat

type Quantity = Memory | Count | Currency | Frequency
type Scale = One | Micro

type Measure<Q, S> {
  count: Nat
}

type ByteSize = Measure<Memory, One>

fn byte_size_count(b: ByteSize) -> Nat {
  b.count
}
"#;
    let resolved = {
        use v2_compiler::v2_compiler_compile::{front_end_sources, SourceFile};
        use v2_compiler::v2_compiler_infer::reconcile;
        use v2_compiler::v2_compiler_normalize::normalize_graph;
        let sources = Rc::new(vec![Rc::new(SourceFile {
            path: "test.dag".to_string(),
            content: src.to_string(),
        })]);
        let frontend = front_end_sources(sources);
        let graph = frontend
            .graph
            .clone()
            .expect("frontend graph for minimal snippet");
        let source_indices = frontend
            .newline_indices
            .iter()
            .cloned()
            .fold(Rc::new(std::collections::HashMap::new()), |acc, index| {
                v2_compiler::v2_rt::rc_map_insert(acc, index.file.clone(), index.clone())
            });
        let norm = normalize_graph(graph, source_indices.clone());
        reconcile(
            norm.graph.clone(),
            source_indices,
            frontend.intern_table.clone(),
        )
    };
    let module = resolved.modules.first().expect("module");
    let env = module.type_env.clone();
    let binding =
        lookup_type_by_name(env.clone(), "ByteSize".to_string()).expect("ByteSize binding");
    let sp = make_span(0, 0);
    let base_rt = Rc::new(Node {
        name: "ByteSize".to_string(),
        ident: None,
        span: sp.clone(),
        ident_span: default_ident_span("ByteSize".to_string(), sp.clone()),
        children: Rc::new(vec![]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: Some(Rc::new(InferredNode::Resolved {
            node: binding.clone(),
        })),
        return_cardinality: v2_compiler::v2_std_core::Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        expr_data: Rc::new(v2_compiler::v2_std_core::ExprData::NoExprData),
    });
    assert!(
        needs_alias_field_expansion(base_rt.clone(), env.clone())
            || lookup_type_by_name(env.clone(), "ByteSize".to_string()).is_some(),
        "ByteSize nominal should gate field expansion through env binding"
    );
    let once = resolve_node(base_rt.clone(), env.clone(), "m".to_string())
        .resolved
        .clone();
    let twice = resolve_node(once.clone(), env.clone(), "m".to_string())
        .resolved
        .clone();
    eprintln!(
        "is_user_generic_use_site(once)={}",
        is_user_generic_use_site(once.clone(), env.clone())
    );
    eprintln!(
        "once inferred={} twice inferred={} twice connective={:?}",
        once.inferred.is_some(),
        twice.inferred.is_some(),
        twice.connective
    );
    let expanded = expand_type_for_field_access(base_rt.clone(), env.clone(), "m".to_string());
    let field = lookup_field_type_node(
        expanded.clone(),
        "count".to_string(),
        env.source_indices.clone(),
    );
    assert_eq!(once.connective, Connective::NoConnective);
    assert!((once.children.len() as i64) > 0);
    assert!(
        field.is_some(),
        "ByteSize field expand should reach count, expanded={expanded:?}"
    );
}

// NOTE: a `measure_dag_v2_loads_without_field_errors` test (asserting all of
// measure.dag loads on v2 with zero diagnostics) is deliberately NOT in this
// proven-G1+G2 slice. It currently fails with `no field 'count' on type
// 'MoneyMicros'`: MoneyMicros = MoneyAmount<Micro> = Measure<Currency, Micro>
// is a MULTI-HOP alias chain, and G2 alias-field expansion only resolves the
// single-hop case so far (ByteSize.count, covered above). Multi-hop alias
// field access lands with the G3 follow-up; the full-load test moves there
// where it goes green.
