use im::HashMap;
use im::Vector;
use std::fs;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use v1_compiler::cli_run::resolve_entry_graph;
use v1_compiler::cli_run::{
    load_sources_for_entry, resolve_closure_request_key_from_digests,
    resolved_graph_parts_semantic_digest,
};
use v1_compiler::resolved_graph_cache::{
    build_valid_artifact_bytes, closure_content_digest, encode_resolved_graph_parts,
    transform_content_digest, write, CacheWriteOutcome,
};

// Serialize the cap env var across tests in this binary.
static CAP_ENV_MUTEX: Mutex<()> = Mutex::new(());

fn temp_dir(label: &str) -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = crate::helpers::workspace_root()
        .join("target")
        .join(format!(
            "gunbc-rg-cap-{label}-{}-{}",
            std::process::id(),
            nanos
        ));
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn write_fixture(dir: &std::path::Path) -> (Vec<String>, String) {
    let common = "module test.common\n\
        type Box { v: Int }\n\
        fn boxed(n: Int) -> Box { Box { v: n } }\n\
        fn unbox(b: Box) -> Int { b.v }\n";
    let entry_a = "module test.a\n\
        import test.common { boxed, unbox }\n\
        fn witness_a_true() -> Bool { (unbox(boxed(10)) + 0) == 10 }\n";
    for (name, src) in [("common.dag", common), ("entry_a.dag", entry_a)] {
        fs::write(dir.join(name), src).unwrap_or_else(|e| panic!("write {name}: {e}"));
    }
    let roots = vec![dir.to_string_lossy().into_owned()];
    let a = dir.join("entry_a.dag").to_string_lossy().into_owned();
    (roots, a)
}

fn total_bin_bytes(root: &std::path::Path) -> u64 {
    let mut total = 0u64;
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = fs::read_dir(&d) else {
            continue;
        };
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().and_then(|s| s.to_str()) == Some("bin") {
                if let Ok(m) = fs::metadata(&p) {
                    total += m.len();
                }
            }
        }
    }
    total
}

/// Classifies the leak and fixes it in one discriminating witness:
/// writing many distinct content-addressed artifacts into a cache whose modeled
/// eviction is `SizeBounded { cap_bytes }` must keep the on-disk footprint under
/// the cap. Before the bound is enforced, the directory grows monotonically and
/// blows past the cap (RED) — the runner-filling bug. After enforcement it stays
/// bounded (GREEN).
#[test]
fn resolved_graph_cache_footprint_stays_under_modeled_cap() {
    let _lock = CAP_ENV_MUTEX.lock().expect("cap env mutex");
    let dir = temp_dir("bound");
    let (roots, a) = write_fixture(&dir);
    let cache_dir = dir.join("cache");
    fs::create_dir_all(&cache_dir).expect("cache dir");

    // A real resolved graph to persist under many synthetic content addresses.
    let (graph, si) = resolve_entry_graph(&roots, &a).expect("resolve fixture");
    let sources = load_sources_for_entry(&roots, &a).expect("sources");
    let closure_digest = closure_content_digest(&sources);
    let compiler_digest = transform_content_digest();
    let encoded = encode_resolved_graph_parts(
        &graph,
        &si.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<HashMap<_, _>>(),
        &Vector::new(),
    )
    .expect("encode");
    let request_key = resolve_closure_request_key_from_digests(&closure_digest, &compiler_digest)
        .expect("request key");
    let semantic = resolved_graph_parts_semantic_digest(
        &encoded.graph_digest,
        encoded.graph_bytes.len() as u64,
        &encoded.indices_digest,
        encoded.indices_bytes.len() as u64,
        &encoded.union_digest,
        encoded.union_bytes.len() as u64,
    )
    .expect("semantic digest");
    let one_artifact = build_valid_artifact_bytes(
        "0000000000000000",
        &graph,
        si.as_ref(),
        &Vector::new(),
        &request_key,
        &semantic,
    )
    .expect("artifact");
    let artifact_len = one_artifact.len() as u64;

    // Cap the cache at room for ~3 artifacts; then write 16 distinct ones.
    let cap = artifact_len * 3 + 64;
    let n = 16u64;
    v1_compiler::resolved_graph_cache::set_resolved_graph_cache_cap_bytes_for_test(Some(cap));

    for i in 0..n {
        let digest = format!("{i:016x}");
        match write(
            &cache_dir,
            &digest,
            &graph,
            si.as_ref(),
            &Vector::new(),
            &request_key,
            &semantic,
        ) {
            Ok(CacheWriteOutcome::Written) | Ok(CacheWriteOutcome::AlreadyExists) => {}
            other => panic!("write {digest} failed: {other:?}"),
        }
    }

    v1_compiler::resolved_graph_cache::set_resolved_graph_cache_cap_bytes_for_test(None);

    let footprint = total_bin_bytes(&cache_dir);
    assert!(
        footprint <= cap,
        "resolved-graph cache exceeded its modeled SizeBounded cap: \
         {footprint} bytes on disk after {n} writes of {artifact_len}-byte artifacts, cap = {cap}. \
         The cache is not enforcing eviction (unbounded growth — fills CI runners)."
    );

    let _ = fs::remove_dir_all(&dir);
}

/// Single-authority guard: the cap the realizer enforces must equal the modeled
/// `SizeBounded` cap declared in the substrate. Reading the `.dag` here means an
/// edit to the modeled cap that isn't mirrored into the Rust seed goes RED,
/// rather than the two drifting silently (the §3 fork this whole fix closes).
#[test]
fn cap_matches_modeled_authority() {
    let _lock = CAP_ENV_MUTEX.lock().expect("cap env mutex");
    v1_compiler::resolved_graph_cache::set_resolved_graph_cache_cap_bytes_for_test(None);
    let ws = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| {
            p.join("dag/extdeps/realization/resolved_graph.dag")
                .exists()
        })
        .expect("locate workspace root containing the resolved_graph .dag authority");
    let dag = fs::read_to_string(ws.join("dag/extdeps/realization/resolved_graph.dag"))
        .expect("read resolved_graph.dag");

    // data resolved_graph_cache_cap_bytes: ByteSize = byte_size(count: 10737418240)
    let line = dag
        .lines()
        .find(|l| l.contains("resolved_graph_cache_cap_bytes") && l.contains("byte_size"))
        .expect("modeled cap declaration present");
    let count = line
        .split("count:")
        .nth(1)
        .and_then(|rest| rest.trim().trim_end_matches(')').trim().parse::<u64>().ok())
        .expect("parse modeled cap count");

    assert_eq!(
        count,
        v1_compiler::resolved_graph_cache::resolved_graph_cache_cap_bytes(),
        "Rust-enforced resolved-graph cache cap drifted from the modeled \
         SizeBounded cap in extdeps.realization.resolved_graph"
    );
}
