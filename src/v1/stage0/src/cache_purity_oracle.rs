// cache_purity_oracle.rs — warm==cold cache-purity detective (DESIGN §5; ROADMAP §2 P1).
//
// The §5 principle (caching-via-realization): a cache that changes a verdict is not a cache
// bug, it is a PURITY bug the cache EXPOSED. A content-keyed realization promises
//   output = f(declared key inputs).
// If an input is READ during realization but is NOT in the content-key, then a WARM hit
// (addressed by the now-stale key) serves a result a fresh COLD recompute would no longer
// produce: warm != cold. So the cache IS the purity falsifier — and this oracle makes that
// falsifier EXECUTABLE: hold the content-key fixed, perturb a candidate hidden input, and a
// pure realization MUST stay byte-identical. A divergence is a LOCATED, TYPED, LOUD
// fail-closed violation naming the read-but-unkeyed axis — never a silent warning.
//
// Authority row: extdeps/realization/cache_purity.dag (`CachePurityVerdict` / the warm==cold
// invariant). This Rust module is the v1 proving HANDLER (§7 Rust→zero); the `.dag` carrier is
// the durable artifact v2 inherits. The realizer it audits is the lone correct content-keyed
// realization — `resolved_graph_cache.rs` (`subject_digest_for_closure`).
//
// SCAFFOLD — §7 Rust-shrinkage receipt (the hand-Rust gate): this module is NET-NEW capability
// (no prior oracle existed, so there is no scaffold to delete here), bound to a named dissolution
// trigger rather than left open-ended. dissolve-on: when the v2 `fold_node` evaluator self-hosts
// the realize fold and `content(T) = content_hash(subgraph)` lands (DESIGN §7; ROADMAP §2 P5), the
// axis set and the audit become one derived `.dag` fact — delete this Rust handler then. Until
// that fixed point, the `.dag` carrier above is the authority and this is one handler of it.

use crate::v1_rt::{self, Hash};

/// A content-keyed realization under purity audit. The cache addresses results by
/// `content_key()`; `realize_cold()` materializes the result FRESH (a forced cache miss) as
/// canonical bytes. Purity ⟺ `realize_cold()` is a pure function of exactly the inputs
/// `content_key()` folds: anything it reads that the key omits is an impure (un-keyed) axis.
pub trait AuditedRealization {
    /// The CLOSED content-key the cache addresses this realization by (the realizer's declared
    /// `inputs_considered`, e.g. `subject_digest_for_closure`).
    fn content_key(&self) -> Hash;

    /// Materialize the result FRESH (forced miss), as canonical bytes. MUST be deterministic in
    /// (and only in) the inputs `content_key` folds — that invariant is exactly what the oracle
    /// falsifies. "Canonical" = stable byte encoding (e.g. sorted map keys) so a benign encoding
    /// reordering is not mistaken for an impurity.
    fn realize_cold(&self) -> Vec<u8>;
}

/// A named hidden-input PROBE: an ambient mutation the oracle applies to try to make
/// `realize_cold` diverge while `content_key` stays fixed. `axis` names the candidate input so
/// a violation is LOCATED. A probe whose perturbation MOVES the key is a DECLARED (keyed) axis,
/// not a hidden one — the oracle skips it, so a legitimately-keyed input never reads as impure.
pub struct HiddenInputProbe<'a> {
    /// Human name of the candidate hidden input (the §5 locus on a violation).
    pub axis: &'a str,
    /// Perturb the ambient state this axis represents (set an env var, touch a file, …).
    pub perturb: Box<dyn FnMut() + 'a>,
    /// Undo `perturb`, restoring the baseline ambient state.
    pub restore: Box<dyn FnMut() + 'a>,
}

/// A LOCATED, TYPED, LOUD fail-closed purity violation: under an UNCHANGED content-key, the
/// fresh realization diverged — an input was read but is NOT in the key (`unkeyed_axis`). A warm
/// cache hit under `content_key` would serve `warm_digest` while the correct fresh answer is now
/// `cold_digest`: the cache would silently serve a stale (wrong) result. Mirrors the `.dag`
/// carrier `extdeps.realization.cache_purity.CachePurityViolation`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachePurityViolation {
    /// The content-key that stayed FIXED across the perturbation (what the cache addresses by).
    pub content_key: Hash,
    /// The hidden input that moved the output — the impure, un-keyed axis. The §5 locus.
    pub unkeyed_axis: String,
    /// Digest of what a WARM hit under `content_key` serves (the cached baseline).
    pub warm_digest: Hash,
    /// Digest of what a fresh COLD recompute now yields after the hidden axis moved.
    pub cold_digest: Hash,
}

impl std::fmt::Display for CachePurityViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CACHE PURITY VIOLATION: under fixed content-key {key}, the realization output \
             diverged when hidden input axis `{axis}` changed — a WARM hit serves {warm} but a \
             fresh COLD recompute now yields {cold}. The axis `{axis}` is READ during realization \
             yet is NOT in the content-key, so a cache hit silently serves a stale result. \
             Fail-closed: either KEY on `{axis}` or stop reading it.",
            key = self.content_key,
            axis = self.unkeyed_axis,
            warm = self.warm_digest,
            cold = self.cold_digest,
        )
    }
}

impl std::error::Error for CachePurityViolation {}

/// The warm==cold oracle. For a realization and a set of candidate hidden-input probes:
///   1. Take the baseline (cold) output and its content-key — this is what a WARM hit would
///      serve under that key.
///   2. For each probe: perturb the ambient state, take a FRESH (cold) output + key, restore.
///      - if the key MOVED, the probe perturbs a DECLARED/keyed axis — a keyed change is a cache
///        MISS, never a stale hit, so it cannot be an impurity: SKIP (no false positive).
///      - if the key stayed FIXED but the output MOVED, the probe's axis is read-but-unkeyed: a
///        warm hit would serve the baseline while a fresh cold recompute now differs → a LOUD,
///        located, typed [`CachePurityViolation`].
///
/// Returns `Ok(())` iff every key-preserving probe leaves the output byte-identical (PURE).
/// Fail-closed: the FIRST divergence is reported at its axis, never swallowed or downgraded.
pub fn audit_warm_equals_cold(
    realization: &impl AuditedRealization,
    probes: &mut [HiddenInputProbe<'_>],
) -> Result<(), CachePurityViolation> {
    let baseline_key = realization.content_key();
    // The baseline is what a WARM hit under `baseline_key` serves (the first value written).
    let warm_digest = v1_rt::bytes_identity_hash(&realization.realize_cold());

    for probe in probes.iter_mut() {
        (probe.perturb)();
        let perturbed_key = realization.content_key();
        let perturbed_bytes = realization.realize_cold();
        (probe.restore)();

        if perturbed_key != baseline_key {
            // The probe moved the content-key — it is a DECLARED (keyed) axis. A keyed change is
            // a cache MISS, not a stale hit, so it cannot be a purity violation. Skip.
            continue;
        }

        let cold_digest = v1_rt::bytes_identity_hash(&perturbed_bytes);
        if cold_digest != warm_digest {
            return Err(CachePurityViolation {
                content_key: baseline_key,
                unkeyed_axis: probe.axis.to_string(),
                warm_digest,
                cold_digest,
            });
        }
    }
    Ok(())
}
