// Seed-growth mark + dissolution trigger live on the carrier:
// test.claim.namespace_occurrence_transport_test.namespace_occurrence_serde_seed_test_dissolution
// (dag/test/claim/namespace_occurrence_transport_test.dag). v1-test class: this file
// deletes with src/v1 once its behavior is carried per the v1-test-migration coverage bar.
use im::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

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

fn repeated_module(functions: usize) -> String {
    let mut source = String::from("module occurrence.scale\n\n");
    for index in 0..functions {
        source.push_str(&format!("fn f_{index}(same: Int) -> Int {{ same }}\n"));
    }
    source
}

fn max_rss_kib() -> i64 {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
    let rc = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) };
    assert_eq!(rc, 0, "getrusage must be available for the size receipt");
    unsafe { usage.assume_init().ru_maxrss }
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

#[test]
fn occurrence_transport_serialization_and_parse_cost_scale_linearly() {
    let small_source = repeated_module(32);
    let small_start = Instant::now();
    let small = parse_source(
        &small_source,
        "occurrence_scale_small.dag",
        occurrence_id_allocator_initial(),
    );
    let small_elapsed = small_start.elapsed();
    let small_bytes = serde_json::to_vec(&small.occurrence_transport)
        .expect("serialize small occurrence transport")
        .len();
    let small_rss = max_rss_kib();

    let large_source = repeated_module(64);
    let large_start = Instant::now();
    let large = parse_source(
        &large_source,
        "occurrence_scale_large.dag",
        occurrence_id_allocator_initial(),
    );
    let large_elapsed = large_start.elapsed();
    let large_bytes = serde_json::to_vec(&large.occurrence_transport)
        .expect("serialize large occurrence transport")
        .len();
    let large_rss = max_rss_kib();

    assert!(small.result.error.is_none(), "{:?}", small.result.error);
    assert!(large.result.error.is_none(), "{:?}", large.result.error);
    assert!(large_bytes > small_bytes);
    assert!(
        large_bytes <= small_bytes * 3,
        "doubling source declarations must remain linear: small={small_bytes}, large={large_bytes}"
    );
    assert!(
        large_elapsed <= small_elapsed.saturating_mul(8) + Duration::from_secs(2),
        "doubling source declarations exceeded a generous linear-time wall: small={small_elapsed:?}, large={large_elapsed:?}"
    );
    assert!(
        large_rss <= small_rss.saturating_mul(2).max(small_rss + 64 * 1024),
        "doubling source declarations exceeded the RSS wall: small={small_rss}KiB, large={large_rss}KiB"
    );
}
