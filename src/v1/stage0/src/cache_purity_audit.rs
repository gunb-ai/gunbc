// cache_purity_audit.rs — the warm==cold shadow audit over the floor discovery corpus
// (DESIGN §5; ROADMAP §2 P3). The LIVE continuous falsifier that makes enabling the
// resolved-graph cache safe: a cache is enabled ONLY behind its purity oracle.
//
// The §5 principle: a content-keyed cache must return a WARM hit equal to a fresh COLD compute.
// If the write→read CODEC is lossy (a field that does not round-trip), a warm hit decodes to a
// graph that differs from cold while every Bool verdict still agrees — a latent impurity that
// bites when emit/lower consume the cached graph. verify-on-read only checks stored-bytes
// INTEGRITY (a digest), not decode-FIDELITY; proud-deer's two-run VERDICT measurement cannot
// catch a verdict-invariant lossy decode (warm decodes the same lossy way both runs). This audit
// closes that gap by comparing the DECODED warm graph against a fresh cold compute, fail-closed.
//
// CANONICAL EQUALITY = STRUCTURAL `==` (not a serialized-byte compare). `ResolvedGraph` and its
// `source_indices` derive `PartialEq`, so `==` compares every field and, crucially, compares the
// only nondeterministic members — the `item_registry` and `source_indices` HashMaps — order-
// INDEPENDENTLY. For a derived `PartialEq` (no hand-written `eq` to under-cover a field) this IS
// canonical-byte equality: equal ⟺ canonical bytes equal. It is preferred over a
// `serde_json::Value` byte round-trip because the latter overflows serde's 128-level parse
// recursion limit on a compiler-sized AST (and a giant `Value` risks a Drop stack-overflow); `==`
// walks the existing structures with no allocation and is depth-safe.
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
use std::rc::Rc;

use crate::cache_purity_oracle::CachePurityViolation;
use crate::cli_run::{
    build_multi_entry_index, discover_floor_corpus_rows, load_sources_for_entry,
    resolve_entry_with_index, MultiEntryIndex,
};
use crate::resolved_graph_cache::{
    resolved_graph_cache_root_from_env, serialize_fixture_payload_for_test,
    subject_digest_for_closure,
};
use crate::v1_compiler_compile::ResolvedGraph;
use crate::v1_rt::{self, Hash};
use crate::v1_std_core::NewlineIndex;

/// A located divergence fingerprint for the violation report — computed ONLY when the two sides
/// already differ structurally (so the order-nondeterminism of the raw HashMap serialization is
/// irrelevant: the contents provably differ, so the fingerprints differ). Not a canonical cache
/// key — purely a `warm`-vs-`cold` content fingerprint for the §5 located error.
fn divergence_fingerprint(
    graph: &ResolvedGraph,
    source_indices: &HashMap<String, Rc<NewlineIndex>>,
) -> Hash {
    v1_rt::bytes_identity_hash(
        &serialize_fixture_payload_for_test(graph, source_indices).unwrap_or_default(),
    )
}

/// Audit one entry: a WARM cache hit must equal the COLD compute it cached (canonical structural
/// `==`, see the module header). Requires `GUNBC_RESOLVED_GRAPH_CACHE_DIR` set to an EMPTY dir
/// (caller's responsibility) so the entry's key starts UNPRIMED and COLD genuinely computes.
///
/// One `index` is shared across all entries and across this entry's cold+warm calls: it is SOUND
/// because `resolve_entry_with_parse_cache` consults the DISK cache (`cross_process_lookup`)
/// FIRST, before any in-process index cache — so the index's `parse_cache` only speeds the cold
/// COMPUTE and never bypasses the warm DISK DECODE (the codec under test). Per-entry index
/// rebuilds (the naive form) are ~1200× redundant whole-tree parse-index builds.
///
/// - COLD: the unprimed key → DISK MISS → compute → WRITE.
/// - WARM: the now-primed key → DISK HIT → DECODE.
/// Divergence ⟹ a located, typed [`CachePurityViolation`] (the codec is lossy / the warm hit
/// is stale): the §5 fail-closed signal. Returns `Ok(())` on equality (pure).
pub fn audit_entry_warm_equals_cold(
    index: &MultiEntryIndex,
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

    // COLD: resolve through the cached path. With the key unprimed this MISSES on disk, computes,
    // and writes; the returned graph is the fresh in-memory compute (not a disk read-back).
    let (cold_graph, cold_si) = match resolve_entry_with_index(index, entry) {
        Ok(r) => r,
        Err(_) => return Ok(()), // resolve error is the discovery gate's concern, not purity's
    };

    // WARM: the now-primed key → DISK HIT → decode (cross_process_lookup short-circuits before any
    // in-process cache, so this exercises the write→read codec even on the shared index).
    let (warm_graph, warm_si) = match resolve_entry_with_index(index, entry) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };

    // Canonical structural equality (depth-safe; HashMaps compared order-independently).
    if cold_graph != warm_graph || cold_si != warm_si {
        return Err(CachePurityViolation {
            content_key,
            unkeyed_axis: format!("resolved-graph cache codec (write→read) for entry {entry}"),
            warm_digest: divergence_fingerprint(&warm_graph, &warm_si),
            cold_digest: divergence_fingerprint(&cold_graph, &cold_si),
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

    // One whole-tree parse index, reused across every entry's cold+warm resolve (sound: the disk
    // cache is consulted before any in-process index cache — see audit_entry_warm_equals_cold).
    let index = build_multi_entry_index(roots);

    let mut violations = Vec::new();
    for entry in &entries {
        if let Err(v) = audit_entry_warm_equals_cold(&index, roots, entry) {
            violations.push(v);
        }
    }
    Ok(CorpusAuditReport {
        entries_audited: entries.len(),
        violations,
    })
}
