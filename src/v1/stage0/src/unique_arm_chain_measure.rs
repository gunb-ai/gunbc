// BRANCH-LOCAL MEASUREMENT ARM -- NOT FOR MERGE.
//
// Subject: bare references that bind through global_bare_lookup's UNIQUE arm and
// would FAIL the ancestor-chain filter the AMBIGUOUS arm already applies.
//
// Why this arm and not the existing instruments: global_bare_lookup consults
// containment only on the ambiguous arm --
//
//     Some(GlobalBareUniqueBinding { binding, .. }) => Some(binding.clone())
//     Some(GlobalBareAmbiguousBinding { candidates, .. }) => ... chain filter ...
//
// so when exactly one declarer is resident in the assembled pool no visibility
// check runs at all. Every count this lane has produced is therefore blind to
// this class: diagnostic histograms see only sites that also happened to red,
// the (file,name) census sees only names with 2+ declarers corpus-wide, and the
// whole SilentPickTelemetry family guards on candidate_count < 2 so it has no
// member here. Failing the chain filter is a property of EVERY bare reference,
// red or green, so this count is the complete population at its grain.
//
// Reads only. Records nothing into any binding decision -- the measurement is
// taken BESIDE the production answer and the production answer is unchanged, so
// arming this cannot alter what the compile accepts.
//
// Dedup is in-process on (env_module_path, name, candidate_module_path) and each
// distinct key is appended once, so the output is bounded by distinct sites
// rather than by evaluation count.

use std::collections::HashSet;
use std::io::Write;
use std::sync::Mutex;
use std::sync::OnceLock;

struct MeasureState {
    seen_divergent: HashSet<(String, String, String)>,
    seen_total: HashSet<(String, String, String)>,
    sink: Option<std::fs::File>,
}

fn state() -> Option<&'static Mutex<MeasureState>> {
    static STATE: OnceLock<Option<Mutex<MeasureState>>> = OnceLock::new();
    STATE
        .get_or_init(|| {
            let path = std::env::var("GUNBC_UNIQUE_ARM_CHAIN_MEASURE").ok()?;
            let sink = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .ok();
            Some(Mutex::new(MeasureState {
                seen_divergent: HashSet::new(),
                seen_total: HashSet::new(),
                sink,
            }))
        })
        .as_ref()
}

/// True iff `candidate_module_path` is a leading-segment prefix of (or equal to)
/// `env_module_path` -- the exact predicate global_bare_chain_candidates applies
/// on the ambiguous arm, restated here rather than called so that arming this
/// cannot perturb the production filter's own inputs.
fn on_chain(env_module_path: &str, candidate_module_path: &str) -> bool {
    let env_segs: Vec<&str> = env_module_path
        .split('.')
        .filter(|s| !s.is_empty())
        .collect();
    let cand_segs: Vec<&str> = candidate_module_path
        .split('.')
        .filter(|s| !s.is_empty())
        .collect();
    if cand_segs.len() > env_segs.len() {
        return false;
    }
    cand_segs.iter().zip(env_segs.iter()).all(|(a, b)| a == b)
}

/// Called from global_bare_lookup's UNIQUE arm, after the production answer is
/// already determined. Never changes it.
pub fn observe_unique_arm(env_module_path: &str, name: &str, candidate_module_path: &str) {
    let Some(state) = state() else { return };
    let Ok(mut st) = state.lock() else { return };
    let key = (
        env_module_path.to_string(),
        name.to_string(),
        candidate_module_path.to_string(),
    );
    let novel_total = st.seen_total.insert(key.clone());
    let diverges = !on_chain(env_module_path, candidate_module_path);
    if novel_total {
        if let Some(sink) = st.sink.as_mut() {
            let _ = writeln!(
                sink,
                "{}\t{}\t{}\t{}",
                if diverges { "OFFCHAIN" } else { "ONCHAIN" },
                env_module_path,
                name,
                candidate_module_path
            );
            let _ = sink.flush();
        }
    }
    if diverges {
        st.seen_divergent.insert(key);
    }
}

#[cfg(test)]
mod tests {
    use super::on_chain;

    // The chain predicate is prefix-on-SEGMENTS, not prefix-on-characters: a
    // shared textual prefix that breaks mid-segment is off-chain, and a
    // candidate deeper than the referencing module can never be an ancestor of
    // it. Both are the ways a naive restatement of this filter goes wrong.
    #[test]
    fn on_chain_is_segment_prefix_not_text_prefix() {
        assert!(on_chain("std.content_hash", "std"));
        assert!(on_chain("std.content_hash", "std.content_hash"));
        assert!(!on_chain("std.content_hash", "std.content"));
        assert!(!on_chain("std.content_hash", "stdx"));
        assert!(!on_chain("std", "std.content_hash"));
        assert!(!on_chain("v2.std.node", "std.node"));
    }
}
