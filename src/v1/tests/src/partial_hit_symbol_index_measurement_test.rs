//! Execution receipt: assembly_symbol_index on all-hits vs exactly-one-miss (same entry, same index).

use std::fs;

use v1_compiler::cli_run::{
    build_multi_entry_index, clear_resolved_graph_memo_for_test,
    evict_typed_module_cache_key_for_test, reset_resolve_stage_attribution_for_test,
    resolve_entry_with_index, resolve_stage_for_entry_file,
    typed_module_cache_content_keys_for_test,
};

fn write_fixture(dir: &std::path::Path) -> (Vec<String>, String) {
    let common = "module test.common\n\
        type Box { v: Int }\n\
        fn boxed(n: Int) -> Box { Box { v: n } }\n\
        fn unbox(b: Box) -> Int { b.v }\n";
    let shared2 = "module test.shared2\nfn val() -> Int { 20 }\n";
    let extra = "module test.extra\nfn pad() -> Int { 7 }\n";
    let entry_b = "module test.b\n\
        import test.common { boxed, unbox }\n\
        import test.shared2 { val }\n\
        import test.extra { pad }\n\
        fn witness_b_true() -> Bool { (unbox(boxed(val())) + pad()) == 27 }\n";

    for (name, src) in [
        ("common.dag", common),
        ("shared2.dag", shared2),
        ("extra.dag", extra),
        ("entry_b.dag", entry_b),
    ] {
        fs::write(dir.join(name), src).unwrap_or_else(|e| panic!("write {name}: {e}"));
    }
    let roots = vec![dir.to_string_lossy().to_string()];
    let entry = dir.join("entry_b.dag").to_string_lossy().to_string();
    (roots, entry)
}

fn symbol_index_ms(entry_file: &str) -> f64 {
    let ns = resolve_stage_for_entry_file(entry_file).assembly_symbol_index;
    ns as f64 / 1_000_000.0
}

fn symbol_index_merge_ms(entry_file: &str) -> f64 {
    let st = resolve_stage_for_entry_file(entry_file);
    (st.assembly_symbol_index + st.assembly_symbol_index_merge) as f64 / 1_000_000.0
}

#[test]
fn partial_hit_symbol_index_pathology_receipt() {
    let dir = crate::helpers::workspace_root()
        .join("target")
        .join(format!("gunbc-partial-hit-sym-idx-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");

    let (roots, entry) = write_fixture(&dir);
    let index = build_multi_entry_index(&roots);

    // Cold baseline (all modules miss typed cache).
    reset_resolve_stage_attribution_for_test();
    resolve_entry_with_index(&index, &entry).expect("cold resolve");
    let cold_symbol_index_ms = symbol_index_ms(&entry);
    let cold_symbol_index_merge_ms = symbol_index_merge_ms(&entry);

    // All-hits: same index, typed cache warm, resolved-graph memo cleared.
    clear_resolved_graph_memo_for_test(&index);
    reset_resolve_stage_attribution_for_test();
    resolve_entry_with_index(&index, &entry).expect("all-hits resolve");
    let all_hits_symbol_index_ms = symbol_index_ms(&entry);
    let all_hits_symbol_index_merge_ms = symbol_index_merge_ms(&entry);

    // Exactly-one-miss: evict one typed key, same index, same entry.
    let keys = typed_module_cache_content_keys_for_test(&index);
    let evict_key = keys
        .iter()
        .next()
        .cloned()
        .expect("typed cache must be populated after cold resolve");
    assert!(
        evict_typed_module_cache_key_for_test(&index, &evict_key),
        "evict must remove one cache entry"
    );
    clear_resolved_graph_memo_for_test(&index);
    reset_resolve_stage_attribution_for_test();
    resolve_entry_with_index(&index, &entry).expect("one-miss resolve");
    let one_miss_symbol_index_ms = symbol_index_ms(&entry);
    let one_miss_symbol_index_merge_ms = symbol_index_merge_ms(&entry);

    eprintln!(
        "PARTIAL_HIT_SYMBOL_INDEX_RECEIPT cold_symbol_index_ms={} \
         all_hits_symbol_index_ms={} \
         one_miss_symbol_index_ms={} \
         cold_symbol_index_merge_ms={} \
         all_hits_symbol_index_merge_ms={} \
         one_miss_symbol_index_merge_ms={} \
         evicted_key={} typed_keys={}",
        cold_symbol_index_ms,
        all_hits_symbol_index_ms,
        one_miss_symbol_index_ms,
        cold_symbol_index_merge_ms,
        all_hits_symbol_index_merge_ms,
        one_miss_symbol_index_merge_ms,
        evict_key,
        keys.len()
    );

    assert!(
        all_hits_symbol_index_ms < 0.001,
        "all-hits path must skip closure symbol_index construction (got {all_hits_symbol_index_ms}ms)"
    );
    assert!(
        cold_symbol_index_ms > 0.0,
        "cold path must pay symbol_index (got {cold_symbol_index_ms}ms)"
    );
    // Absorbing-fallback shape: one miss costs substantially the same as all-miss, not proportional to 1/N.
    let ratio = if cold_symbol_index_ms > 0.0 {
        one_miss_symbol_index_ms / cold_symbol_index_ms
    } else {
        0.0
    };
    eprintln!(
        "PARTIAL_HIT_SYMBOL_INDEX_RATIO one_miss_over_cold={:.3}",
        ratio
    );
    assert!(
        ratio > 0.8,
        "one miss should cost ~full closure symbol_index (ratio {}, cold={}ms one_miss={}ms)",
        ratio,
        cold_symbol_index_ms,
        one_miss_symbol_index_ms
    );

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn partial_hit_symbol_index_pathology_receipt_ci_spec() {
    let roots = vec!["dag".to_string(), "src/v2".to_string()];
    let entry = crate::helpers::workspace_root()
        .join("dag/gunbc/ci_spec.dag")
        .to_string_lossy()
        .into_owned();
    let index = build_multi_entry_index(&roots);

    reset_resolve_stage_attribution_for_test();
    resolve_entry_with_index(&index, &entry).expect("cold ci_spec resolve");
    let cold_ms = symbol_index_ms(&entry);
    let cold_merge_ms = symbol_index_merge_ms(&entry);

    clear_resolved_graph_memo_for_test(&index);
    reset_resolve_stage_attribution_for_test();
    resolve_entry_with_index(&index, &entry).expect("all-hits ci_spec");
    let all_hits_ms = symbol_index_ms(&entry);
    let all_hits_merge_ms = symbol_index_merge_ms(&entry);

    let keys = typed_module_cache_content_keys_for_test(&index);
    let evict_key = keys.iter().next().cloned().expect("typed keys after cold");
    evict_typed_module_cache_key_for_test(&index, &evict_key);
    clear_resolved_graph_memo_for_test(&index);
    reset_resolve_stage_attribution_for_test();
    resolve_entry_with_index(&index, &entry).expect("one-miss ci_spec");
    let one_miss_ms = symbol_index_ms(&entry);
    let one_miss_merge_ms = symbol_index_merge_ms(&entry);

    let ratio = if cold_ms > 0.0 {
        one_miss_ms / cold_ms
    } else {
        0.0
    };
    eprintln!(
        "PARTIAL_HIT_CI_SPEC cold_ms={} all_hits_ms={} one_miss_ms={} \
         cold_merge_ms={} all_hits_merge_ms={} one_miss_merge_ms={} \
         ratio={:.3} typed_keys={}",
        cold_ms,
        all_hits_ms,
        one_miss_ms,
        cold_merge_ms,
        all_hits_merge_ms,
        one_miss_merge_ms,
        ratio,
        keys.len()
    );

    assert!(all_hits_ms < 0.001, "all-hits must skip symbol_index");
    assert!(
        cold_ms > 1.0,
        "ci_spec cold symbol_index should be measurable"
    );
    assert!(
        ratio > 0.8,
        "one miss ~ full closure index (ratio {:.3})",
        ratio
    );
}
