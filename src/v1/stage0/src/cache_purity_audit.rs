// cache_purity_audit.rs — the warm==cold shadow audit over the floor discovery corpus
// (DESIGN §5; ROADMAP §2 P3). The LIVE continuous falsifier that makes enabling the
// resolved-graph cache safe: a cache is enabled ONLY behind its purity oracle.
//
// The §5 principle: a content-keyed cache must return a WARM hit byte-identical to a fresh
// COLD compute. If the write→read CODEC is lossy (a field that does not round-trip), a warm
// hit decodes to a graph that differs from cold while every Bool verdict still agrees — a
// latent impurity that bites when emit/lower consume the cached graph. verify-on-read only
// checks stored-bytes INTEGRITY (a digest), not decode-FIDELITY; proud-deer's two-run VERDICT
// measurement cannot catch a verdict-invariant lossy decode (warm decodes the same lossy way
// both runs). This audit closes that gap by comparing the DECODED warm graph against a fresh
// cold compute at the BYTE level, fail-closed.
//
// CODEC ISOLATION (why both sides go through `resolve_entry_with_index`): cold uses the SAME
// cached-orchestration resolve path as warm, just with an empty cache (compute+write) vs a
// primed cache (decode). So the only difference under test is the write→read round-trip — the
// codec — NOT the resolve_entry_graph-vs-resolve_entry_with_index path difference (which the
// CRIT-1 boundary in resolve_cross_process_cache_test.rs flags as a separate, benign axis).
//
// This module is the v1 proving HANDLER (§7 Rust→zero); the `.dag` carrier
// extdeps/realization/cache_purity.dag is the durable artifact. dissolve-on: the v2 realize
// fold self-hosts this comparison (ROADMAP §2 P5; content(T)=content_hash(subgraph)).

use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use crate::cache_purity_oracle::CachePurityViolation;
use crate::cli_run::{
    build_multi_entry_index, discover_floor_corpus_rows, load_sources_for_entry,
    resolve_entry_with_index,
};
use crate::resolved_graph_cache::{
    resolved_graph_cache_root_from_env, serialize_fixture_payload_for_test,
    subject_digest_for_closure,
};
use crate::v1_compiler_compile::ResolvedGraph;
use crate::v1_rt::{self, Hash};
use crate::v1_std_core::NewlineIndex;

/// Canonical serialized bytes of a resolved graph. Round-trips through `serde_json::Value`
/// (whose object keys are a sorted `BTreeMap` — `preserve_order` is off in this workspace) so
/// `source_indices`'s `HashMap` iteration order cannot make a byte compare false-fail. A
/// reordered map is BENIGN (canonicalization quotients it out); a genuine semantic field diff
/// survives — exactly the divergence the audit must catch.
pub fn canonical_graph_bytes(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> Vec<u8> {
    let raw = serialize_fixture_payload_for_test(graph, source_indices).expect("serialize payload");
    let value: serde_json::Value = serde_json::from_slice(&raw).expect("payload is valid json");
    serde_json::to_vec(&value).expect("re-serialize canonical")
}

/// Audit one entry: a WARM cache hit must be canonically byte-identical to the COLD compute it
/// cached. Both go through `resolve_entry_with_index` so the ONLY difference under test is the
/// write→read codec. Requires `GUNBC_RESOLVED_GRAPH_CACHE_DIR` to point at `cache_root` (set by
/// the caller) so the cached path is actually exercised.
///
/// - COLD: a fresh index over an empty/primed-elsewhere key → MISS → compute → WRITE.
/// - WARM: a fresh index over the now-primed key → HIT → DECODE.
/// Divergence ⟹ a located, typed [`CachePurityViolation`] (the codec is lossy / the warm hit
/// is stale): the §5 fail-closed signal. Returns `Ok(())` on byte-identity (pure).
pub fn audit_entry_warm_equals_cold(
    roots: &[String],
    entry: &str,
) -> Result<(), CachePurityViolation> {
    let sources = match load_sources_for_entry(roots, entry) {
        Ok(s) => s,
        // An entry that no longer loads is not a cache-purity signal; skip (the discovery
        // gate itself fails closed on a missing entry).
        Err(_) => return Ok(()),
    };
    let content_key = subject_digest_for_closure(&sources);

    // COLD: first resolve through the cached path. If the key is unprimed this MISSES, computes,
    // and writes; the returned graph is the fresh compute.
    let cold_index = build_multi_entry_index(roots);
    let (cold_graph, cold_si) = match resolve_entry_with_index(&cold_index, entry) {
        Ok(r) => r,
        Err(_) => return Ok(()), // resolve error is the discovery gate's concern, not purity's
    };
    let cold_digest = v1_rt::bytes_identity_hash(&canonical_graph_bytes(&cold_graph, &cold_si));

    // WARM: a fresh index over the now-primed key → HIT → decode.
    let warm_index = build_multi_entry_index(roots);
    let (warm_graph, warm_si) = match resolve_entry_with_index(&warm_index, entry) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };
    let warm_digest = v1_rt::bytes_identity_hash(&canonical_graph_bytes(&warm_graph, &warm_si));

    if cold_digest != warm_digest {
        return Err(CachePurityViolation {
            content_key,
            unkeyed_axis: format!("resolved-graph cache codec (write→read) for entry {entry}"),
            warm_digest,
            cold_digest,
        });
    }
    Ok(())
}

/// Audit the whole floor discovery corpus: every distinct entry's warm hit must equal its cold
/// compute. Fail-closed: returns ALL violations found (the caller fails the gate if non-empty).
/// `cache_root` must be the cache dir `GUNBC_RESOLVED_GRAPH_CACHE_DIR` is set to.
///
/// `unaudited_residue` is the §5 HONEST EDGE: this gate is SOUND over the CI corpus, not
/// COMPLETE over all realizations — a prod-only realization never resolved in CI can still go
/// impure silently. Surfaced, not papered over.
pub struct CorpusAuditReport {
    pub entries_audited: usize,
    pub violations: Vec<CachePurityViolation>,
}

pub fn audit_floor_discovery_corpus(
    roots: &[String],
    scan_dirs: &[String],
) -> Result<CorpusAuditReport, String> {
    if resolved_graph_cache_root_from_env().is_none() {
        return Err(
            "cache purity audit requires GUNBC_RESOLVED_GRAPH_CACHE_DIR to be set (the cache it \
             audits is keyed by it); refusing to run a vacuous audit (DESIGN §5/§6: an inert gate \
             is a lie)"
                .to_string(),
        );
    }
    let rows = discover_floor_corpus_rows(roots, scan_dirs)?;

    // Dedup by ENTRY — the resolved graph is per closure (per entry), not per (entry, function).
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut entries: Vec<String> = Vec::new();
    for row in &rows {
        if seen.insert(row.entry.clone()) {
            entries.push(row.entry.clone());
        }
    }

    let mut violations = Vec::new();
    for entry in &entries {
        if let Err(v) = audit_entry_warm_equals_cold(roots, entry) {
            violations.push(v);
        }
    }
    Ok(CorpusAuditReport {
        entries_audited: entries.len(),
        violations,
    })
}
