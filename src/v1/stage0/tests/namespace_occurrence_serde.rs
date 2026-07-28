// Seed-growth mark + dissolution trigger live on the carrier:
// test.claim.namespace_occurrence_transport_test.namespace_occurrence_serde_seed_test_dissolution
// (dag/test/claim/namespace_occurrence_transport_test.dag). v1-test class: this file
// deletes with src/v1 once its behavior is carried per the v1-test-migration coverage bar.
use im::HashMap;
use std::rc::Rc;

use v1_compiler::std_occurrence_identity::{
    occurrence_id_allocator_initial, OccurrenceIdAllocator,
};
use v1_compiler::v1_compiler_parse::{parse_with_table_in_occurrence_scope, ParseWithTableResult};
use v1_compiler::v1_compiler_tokenize::tokenize;
use v1_compiler::v1_std_core::{build_newline_index, empty_intern_table, NewlineIndex};

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

    let encoded = serde_json::to_vec(&parsed).expect("serialize occurrence parse result");
    let decoded: Rc<ParseWithTableResult> =
        serde_json::from_slice(&encoded).expect("deserialize occurrence parse result");
    assert_eq!(decoded, parsed);

    let parsed_with_perturbed_span_file = parse_source(
        source,
        "different_span_file.dag",
        occurrence_id_allocator_initial(),
    );
    let original_ids: Vec<i64> = parsed
        .occurrence_transport
        .index
        .entries
        .iter()
        .map(|entry| entry.projection.occurrence.value)
        .collect();
    let perturbed_ids: Vec<i64> = parsed_with_perturbed_span_file
        .occurrence_transport
        .index
        .entries
        .iter()
        .map(|entry| entry.projection.occurrence.value)
        .collect();
    assert_eq!(
        original_ids, perturbed_ids,
        "filename/span evidence must not participate in occurrence identity"
    );
}
