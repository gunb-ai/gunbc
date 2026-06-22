use crate::v1_rt::{self, Hash};

pub trait AuditedRealization {
    fn content_key(&self) -> Hash;

    fn realize_cold(&self) -> Vec<u8>;
}

pub struct HiddenInputProbe<'a> {
    pub axis: &'a str,
    pub perturb: Box<dyn FnMut() + 'a>,
    pub restore: Box<dyn FnMut() + 'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachePurityViolation {
    pub content_key: Hash,
    pub unkeyed_axis: String,
    pub warm_digest: Hash,
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

pub fn audit_warm_equals_cold(
    realization: &impl AuditedRealization,
    probes: &mut [HiddenInputProbe<'_>],
) -> Result<(), CachePurityViolation> {
    let baseline_key = realization.content_key();
    let warm_digest = v1_rt::bytes_identity_hash(&realization.realize_cold());

    for probe in probes.iter_mut() {
        (probe.perturb)();
        let perturbed_key = realization.content_key();
        let perturbed_bytes = realization.realize_cold();
        (probe.restore)();

        if perturbed_key != baseline_key {
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
