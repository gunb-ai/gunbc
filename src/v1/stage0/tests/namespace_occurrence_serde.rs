// Seed-growth mark + dissolution trigger live on the carrier:
// test.claim.namespace_occurrence_transport_test.namespace_occurrence_serde_seed_test_dissolution
// (dag/test/claim/namespace_occurrence_transport_test.dag). v1-test class: this file
// deletes with src/v1 once its behavior is carried per the v1-test-migration coverage bar.
use im::HashMap;
use std::collections::HashSet;
use std::rc::Rc;

use v1_compiler::std_occurrence_identity::{
    occurrence_id_allocator_initial, NodeOccurrenceIdentity, OccurrenceCategory,
    OccurrenceContainmentPath, OccurrenceIdAllocator, OccurrenceTransport,
};
use v1_compiler::v1_compiler_parse::{parse_with_table_in_occurrence_scope, ParseWithTableResult};
use v1_compiler::v1_compiler_tokenize::tokenize;
use v1_compiler::v1_std_core::{
    build_newline_index, empty_intern_table, make_expr_node, no_span, with_required_cardinality,
    ExprData, InferredNode, MatchPattern, NewlineIndex, Node,
};

fn parse_source(
    source: &str,
    file: &str,
    allocator: OccurrenceIdAllocator,
) -> Rc<ParseWithTableResult> {
    let mut source_indices: HashMap<String, Rc<NewlineIndex>> = HashMap::new();
    source_indices.insert(
        file.to_string(),
        build_newline_index(file.to_string(), source.to_string()),
    );
    parse_with_table_in_occurrence_scope(
        tokenize(source.to_string(), file.to_string()),
        Rc::new(source_indices),
        empty_intern_table(),
        allocator,
    )
}

fn containment_ids(path: &OccurrenceContainmentPath) -> (Vec<i64>, i64) {
    (
        path.ancestors
            .iter()
            .map(|ancestor| ancestor.value)
            .collect(),
        path.terminal.value,
    )
}

type OccurrenceIdentityView = (
    Vec<(i64, (Vec<i64>, i64))>,
    Vec<(i64, OccurrenceCategory, (Vec<i64>, i64))>,
    Vec<(i64, OccurrenceCategory, (Vec<i64>, i64))>,
);

fn occurrence_identity_view(transport: &OccurrenceTransport) -> OccurrenceIdentityView {
    (
        transport
            .index
            .entries
            .iter()
            .map(|entry| {
                (
                    entry.projection.occurrence.value,
                    containment_ids(&entry.containment),
                )
            })
            .collect(),
        transport
            .declarations
            .iter()
            .map(|declaration| {
                (
                    declaration.occurrence.value,
                    declaration.category,
                    containment_ids(&declaration.containment),
                )
            })
            .collect(),
        transport
            .references
            .iter()
            .map(|reference| {
                (
                    reference.occurrence.value,
                    reference.category,
                    containment_ids(&reference.containment),
                )
            })
            .collect(),
    )
}

fn declaration_spans(transport: &OccurrenceTransport) -> Vec<(i64, i64)> {
    transport
        .declarations
        .iter()
        .map(|declaration| {
            (
                declaration.diagnostic_span.start,
                declaration.diagnostic_span.end,
            )
        })
        .collect()
}

fn node_occurrence_ids(node: &Node, ids: &mut Vec<i64>, synthetic_count: &mut usize) {
    match node.occurrence_identity.as_ref() {
        NodeOccurrenceIdentity::OccurrenceMinted { id } => ids.push(id.value),
        NodeOccurrenceIdentity::OccurrenceSynthetic => *synthetic_count += 1,
        NodeOccurrenceIdentity::OccurrenceProjected { .. } => {
            panic!("authored parser returned a projected occurrence")
        }
    }
    for child in node.children.iter() {
        node_occurrence_ids(child, ids, synthetic_count);
    }
    for param in node.params.iter() {
        node_occurrence_ids(param, ids, synthetic_count);
    }
    for used in node.uses.iter() {
        node_occurrence_ids(used, ids, synthetic_count);
    }
    for property in node.properties.iter() {
        node_occurrence_ids(property, ids, synthetic_count);
    }
    for optional in [
        node.body.as_ref(),
        node.transport.as_ref(),
        node.type_annotation.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        node_occurrence_ids(optional, ids, synthetic_count);
    }
    if let Some(InferredNode::Resolved { node: inferred }) = node.inferred.as_deref() {
        node_occurrence_ids(inferred, ids, synthetic_count);
    }
    match node.match_pattern.as_deref() {
        Some(MatchPattern::Bind { declaration }) => {
            node_occurrence_ids(declaration, ids, synthetic_count);
        }
        Some(MatchPattern::VariantPattern { field_bindings, .. }) => {
            for binding in field_bindings.iter() {
                node_occurrence_ids(binding, ids, synthetic_count);
            }
        }
        Some(MatchPattern::LitPattern { .. } | MatchPattern::Wildcard) | None => {}
    }
}

fn node_occurrence_descriptions(node: &Node, wanted: i64, found: &mut Vec<String>) {
    if matches!(node.occurrence_identity.as_ref(), NodeOccurrenceIdentity::OccurrenceMinted { id } if id.value == wanted)
    {
        found.push(format!(
            "name={:?} span={}:{}-{}",
            node.name, node.span.file, node.span.start, node.span.end
        ));
    }
    for child in node
        .children
        .iter()
        .chain(node.params.iter())
        .chain(node.uses.iter())
        .chain(node.properties.iter())
    {
        node_occurrence_descriptions(child, wanted, found);
    }
    for optional in [
        node.body.as_ref(),
        node.transport.as_ref(),
        node.type_annotation.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        node_occurrence_descriptions(optional, wanted, found);
    }
    if let Some(InferredNode::Resolved { node: inferred }) = node.inferred.as_deref() {
        node_occurrence_descriptions(inferred, wanted, found);
    }
    match node.match_pattern.as_deref() {
        Some(MatchPattern::Bind { declaration }) => {
            node_occurrence_descriptions(declaration, wanted, found)
        }
        Some(MatchPattern::VariantPattern { field_bindings, .. }) => {
            for binding in field_bindings.iter() {
                node_occurrence_descriptions(binding, wanted, found);
            }
        }
        _ => {}
    }
}

#[test]
fn oci_digest_parser_does_not_reuse_authored_identity() {
    let parsed = parse_source(
        include_str!("../../../../dag/extdeps/container/oci/digest.dag"),
        "dag/extdeps/container/oci/digest.dag",
        occurrence_id_allocator_initial(),
    );
    assert!(parsed.result.error.is_none(), "{:?}", parsed.result.error);
    let mut seen = HashSet::new();
    if let Some(duplicate) = parsed
        .occurrence_transport
        .index
        .entries
        .iter()
        .map(|entry| entry.projection.occurrence.value)
        .find(|id| !seen.insert(*id))
    {
        let mut descriptions = Vec::new();
        node_occurrence_descriptions(
            parsed
                .result
                .module
                .as_ref()
                .expect("successful parse module"),
            duplicate,
            &mut descriptions,
        );
        panic!("duplicate authored identity {duplicate}: {descriptions:?}");
    }
}

#[test]
fn authored_identity_allocator_crosses_real_module_boundary() {
    let materialization = parse_source(
        include_str!("../../../../dag/extdeps/cache/materialization.dag"),
        "dag/extdeps/cache/materialization.dag",
        occurrence_id_allocator_initial(),
    );
    assert!(
        materialization.result.error.is_none(),
        "{:?}",
        materialization.result.error
    );
    let digest = parse_source(
        include_str!("../../../../dag/extdeps/container/oci/digest.dag"),
        "dag/extdeps/container/oci/digest.dag",
        materialization.occurrence_allocator.clone(),
    );
    assert!(digest.result.error.is_none(), "{:?}", digest.result.error);
    let left: HashSet<i64> = materialization
        .occurrence_transport
        .index
        .entries
        .iter()
        .map(|entry| entry.projection.occurrence.value)
        .collect();
    let overlap: Vec<i64> = digest
        .occurrence_transport
        .index
        .entries
        .iter()
        .map(|entry| entry.projection.occurrence.value)
        .filter(|id| left.contains(id))
        .collect();
    assert!(
        overlap.is_empty(),
        "cross-module identity overlap: {overlap:?}"
    );
}

#[test]
fn expected_red_roster_items_remain_annotation_subjects() {
    let parsed = parse_source(
        include_str!("../../expected_red_roster_join.dag"),
        "src/v1/expected_red_roster_join.dag",
        occurrence_id_allocator_initial(),
    );
    assert!(parsed.result.error.is_none(), "{:?}", parsed.result.error);
    let module = parsed
        .result
        .module
        .as_ref()
        .expect("successful parse module");
    let synthetic_items: Vec<String> = module
        .children
        .iter()
        .filter(|item| {
            matches!(
                item.occurrence_identity.as_ref(),
                NodeOccurrenceIdentity::OccurrenceSynthetic
            )
        })
        .map(|item| item.name.clone())
        .collect();
    let depth_one = parsed
        .occurrence_transport
        .index
        .entries
        .iter()
        .filter(|entry| entry.containment.ancestors.len() == 1)
        .count();
    assert_eq!(
        depth_one,
        module.children.len() + module.params.len(),
        "every module item/import must remain an annotation subject; synthetic items={synthetic_items:?}"
    );
}

#[test]
fn occurrence_transport_round_trips_with_node_identity_and_paths() {
    let source = "module occurrence.serde\n\nfn left(x: Int) -> Int { x }\nfn right(x: Int) -> Int { left(x) }\n";
    let parsed = parse_source(
        source,
        "occurrence_serde.dag",
        occurrence_id_allocator_initial(),
    );
    assert!(parsed.result.error.is_none(), "{:?}", parsed.result.error);
    assert!(!parsed.occurrence_transport.index.entries.is_empty());
    assert!(!parsed.occurrence_transport.declarations.is_empty());
    assert!(!parsed.occurrence_transport.references.is_empty());
    let module = parsed
        .result
        .module
        .as_ref()
        .expect("successful parse module");
    let mut node_ids = Vec::new();
    let mut synthetic_count = 0;
    node_occurrence_ids(module, &mut node_ids, &mut synthetic_count);
    let mut sidecar_ids: Vec<i64> = parsed
        .occurrence_transport
        .index
        .entries
        .iter()
        .map(|entry| entry.projection.occurrence.value)
        .collect();
    node_ids.sort_unstable();
    sidecar_ids.sort_unstable();
    let mut unique_node_ids = node_ids.clone();
    unique_node_ids.dedup();
    let mut unique_sidecar_ids = sidecar_ids.clone();
    unique_sidecar_ids.dedup();
    assert_eq!(
        sidecar_ids, unique_sidecar_ids,
        "the parser sidecar must index each authored identity exactly once"
    );
    assert_eq!(
        unique_node_ids, unique_sidecar_ids,
        "authored node and sidecar identity sets must agree despite shared node reachability"
    );
    let mut equal_parameter_ids: Vec<i64> = module
        .children
        .iter()
        .flat_map(|declaration| declaration.params.iter())
        .filter(|parameter| parameter.name == "x")
        .map(|parameter| match parameter.occurrence_identity.as_ref() {
            NodeOccurrenceIdentity::OccurrenceMinted { id } => id.value,
            _ => panic!("authored parameter did not carry a minted identity"),
        })
        .collect();
    equal_parameter_ids.sort_unstable();
    equal_parameter_ids.dedup();
    assert_eq!(
        equal_parameter_ids.len(),
        2,
        "structurally equal authored parameters must have distinct identities"
    );
    assert_eq!(
        synthetic_count, 0,
        "successful authored parse must not default nodes to synthetic"
    );

    let rebuilt = with_required_cardinality(module.clone());
    assert_eq!(
        rebuilt.occurrence_identity, module.occurrence_identity,
        "rebuild must preserve identity exactly"
    );

    let synthesized = make_expr_node(
        Rc::new(NodeOccurrenceIdentity::OccurrenceSynthetic),
        Rc::new(ExprData::NoExprData),
        Rc::new(vec![].into()),
        None,
        no_span(),
    );
    assert!(matches!(
        synthesized.occurrence_identity.as_ref(),
        NodeOccurrenceIdentity::OccurrenceSynthetic
    ));

    let encoded = serde_json::to_vec(&parsed).expect("serialize occurrence parse result");
    let decoded: Rc<ParseWithTableResult> =
        serde_json::from_slice(&encoded).expect("deserialize occurrence parse result");
    assert_eq!(decoded, parsed);

    let parsed_with_perturbed_span_file = parse_source(
        source,
        "different_span_file.dag",
        occurrence_id_allocator_initial(),
    );
    assert!(
        parsed_with_perturbed_span_file.result.error.is_none(),
        "{:?}",
        parsed_with_perturbed_span_file.result.error
    );
    assert_eq!(
        occurrence_identity_view(&parsed.occurrence_transport),
        occurrence_identity_view(&parsed_with_perturbed_span_file.occurrence_transport),
        "filename evidence must not participate in occurrence identity or containment"
    );

    let layout_perturbed_source = "module occurrence.serde\n\nfn   left( x: Int ) -> Int {   x }\nfn   right( x: Int ) -> Int {   left( x ) }\n";
    let parsed_with_perturbed_layout = parse_source(
        layout_perturbed_source,
        "layout_perturbed_file.dag",
        occurrence_id_allocator_initial(),
    );
    assert!(
        parsed_with_perturbed_layout.result.error.is_none(),
        "{:?}",
        parsed_with_perturbed_layout.result.error
    );

    let original_declaration_spans = declaration_spans(&parsed.occurrence_transport);
    let layout_declaration_spans =
        declaration_spans(&parsed_with_perturbed_layout.occurrence_transport);
    assert_eq!(
        original_declaration_spans.len(),
        layout_declaration_spans.len(),
        "layout twin must mint the same declaration population"
    );
    assert!(
        original_declaration_spans
            .iter()
            .zip(layout_declaration_spans.iter())
            .any(|(original, perturbed)| original != perturbed),
        "layout perturbation control: no declaration span moved, so the independence assertion below would be vacuous"
    );
    assert_eq!(
        occurrence_identity_view(&parsed.occurrence_transport),
        occurrence_identity_view(&parsed_with_perturbed_layout.occurrence_transport),
        "layout/span evidence must not participate in occurrence identity or containment"
    );
}
