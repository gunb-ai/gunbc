//! Host transcription of the floor's heartbeat beat lines into primitives for
//! `gunbc.floor.floor_demand`; the reading is the fold's, not this module's.

/// ONE HEARTBEAT BEAT AS PRIMITIVES, the host half of `gunbc.floor.floor_demand` `FloorBeatReading`.
/// Transcription only: what a token is, what the held set is and where a phase begins are the
/// fold's decisions, not this reader's.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FloorBeatLine {
    pub beat: usize,
    pub opens_phase: bool,
    pub seam_before: String,
    pub seam_after: String,
    pub stall_per_min: Option<u64>,
    pub swap_bytes: Option<u64>,
    pub charge: u64,
    /// `memory_stat=[...]` in the order the heartbeat prints it, which is the order of
    /// `FLOOR_BEAT_STAT_KEYS`.
    pub stat: [u64; 11],
}

pub const FLOOR_BEAT_STAT_KEYS: [&str; 11] = [
    "anon",
    "file",
    "shmem",
    "unevictable",
    "slab_unreclaimable",
    "slab_reclaimable",
    "kernel_stack",
    "pagetables",
    "percpu",
    "sock",
    "file_dirty",
];

fn beat_field<'a>(line: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!(" {name}=");
    let at = line.find(&needle)? + needle.len();
    let rest = &line[at..];
    Some(rest.split(' ').next().unwrap_or(rest))
}

/// Transcribes the heartbeat's `[floor-cgroup] when=` lines into beats, pairing each
/// `stat_level=` line with the `swap_current=` line the heartbeat prints for the same `when=`
/// just before it. The other `when=` lines (the envelope's `path=` and `level=` walk) carry no
/// beat and are passed over by shape.
///
/// A COUNTER THE LINE PRINTED AS `na` REFUSES THE TRANSCRIPTION, with the line named. The stat
/// counters have no unread arm in the typed beat, and a zero would read as memory the run did not
/// hold -- the fabricated small demand `beat_held_set` exists to refuse. Stall and swap DO have
/// unread arms, so the literal `na` there -- and ONLY that spelling -- is carried as `None`. A stall
/// or swap value that is missing, garbled or negative refuses like a stat counter: reading it as
/// "unread" would turn a malformed line into the healthy-looking absence the fold must not invent.
/// A stall or swap clause: `na` is unread; a non-negative integer is read; anything else refuses.
fn unread_or_count(raw: Option<&str>, what: &str, line: &str) -> Result<Option<u64>, String> {
    match raw {
        None => Err(format!("floor beat line unreadable (no {what}=): {line}")),
        Some("na") => Ok(None),
        Some(v) => v
            .parse::<u64>()
            .map(Some)
            .map_err(|_| format!("floor beat line unreadable ({what}={v}): {line}")),
    }
}

pub fn transcribe_floor_beats(lines: &[String]) -> Result<Vec<FloorBeatLine>, String> {
    let mut swap_by_when: std::collections::HashMap<String, Option<u64>> =
        std::collections::HashMap::new();
    let mut beats = Vec::new();
    for line in lines {
        let Some(when) = beat_field(line, "when") else {
            continue;
        };
        if let Some(swap) = beat_field(line, "swap_current") {
            swap_by_when.insert(
                when.to_string(),
                unread_or_count(Some(swap), "swap_current", line)?,
            );
            continue;
        }
        if beat_field(line, "stat_level").is_none() {
            continue;
        }
        let refuse = |what: &str| format!("floor beat line unreadable ({what}): {line}");
        let seam = beat_field(line, "seam").ok_or_else(|| refuse("no seam="))?;
        let (seam_before, seam_after) = match seam.strip_prefix("transition:") {
            Some(moved) => {
                let (a, b) = moved
                    .split_once('>')
                    .ok_or_else(|| refuse("transition without '>'"))?;
                (a.to_string(), b.to_string())
            }
            None => (seam.to_string(), seam.to_string()),
        };
        let charge = beat_field(line, "current")
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or_else(|| refuse("current="))?;
        let body = line
            .split_once("memory_stat=[")
            .and_then(|(_, rest)| rest.split_once(']'))
            .map(|(body, _)| body)
            .ok_or_else(|| refuse("memory_stat="))?;
        let parts: Vec<&str> = body.split(',').collect();
        let mut stat = [0u64; 11];
        for (i, key) in FLOOR_BEAT_STAT_KEYS.iter().enumerate() {
            let (k, v) = (parts.get(2 * i), parts.get(2 * i + 1));
            if k != Some(key) {
                return Err(refuse(&format!("memory_stat key {i} is not {key}")));
            }
            stat[i] = v
                .and_then(|v| v.parse::<u64>().ok())
                .ok_or_else(|| refuse(&format!("memory_stat {key}")))?;
        }
        beats.push(FloorBeatLine {
            beat: beats.len(),
            opens_phase: when.starts_with("seam-"),
            seam_before,
            seam_after,
            stall_per_min: unread_or_count(
                beat_field(line, "stall_per_min"),
                "stall_per_min",
                line,
            )?,
            swap_bytes: swap_by_when.remove(when).flatten(),
            charge,
            stat,
        });
    }
    Ok(beats)
}

#[cfg(test)]
mod transcription_tests {
    use super::*;

    fn stat_line(when: &str, seam: &str, anon: &str) -> String {
        format!(
            "[floor-cgroup] when={when} stat_level=/sys/fs/cgroup/x seam={seam} stall_per_min=na \
             current=100 memory_stat=[anon,{anon},file,2,shmem,3,unevictable,4,\
             slab_unreclaimable,5,slab_reclaimable,6,kernel_stack,7,pagetables,8,percpu,9,sock,10,\
             file_dirty,11]"
        )
    }

    #[test]
    fn a_seam_beat_opens_a_phase_and_carries_its_paired_swap() {
        let lines = vec![
            "[floor-cgroup] when=seam-parse path=/sys/fs/cgroup/x".to_string(),
            "[floor-cgroup] when=seam-parse seam=parse unix_ms=1 swap_current=4096 events_local=[] pressure=[] procs=[]".to_string(),
            stat_line("seam-parse", "parse", "1"),
            "[floor-cgroup] when=beat-3 seam=parse unix_ms=2 swap_current=na events_local=[] pressure=[] procs=[]".to_string(),
            stat_line("beat-3", "transition:parse>declarations", "12"),
        ];
        let beats = transcribe_floor_beats(&lines).expect("transcribes");
        assert_eq!(
            beats.len(),
            2,
            "the path= envelope line is not a beat: {beats:?}"
        );
        assert!(beats[0].opens_phase && !beats[1].opens_phase);
        assert_eq!(beats[0].swap_bytes, Some(4096));
        assert_eq!(beats[1].swap_bytes, None, "`na` swap is unread, not zero");
        assert_eq!(
            beats[0].stall_per_min, None,
            "`na` stall is unread, not zero"
        );
        assert_eq!(
            (beats[1].seam_before.as_str(), beats[1].seam_after.as_str()),
            ("parse", "declarations")
        );
        assert_eq!(beats[1].stat[0], 12);
        assert_eq!(beats[0].stat[10], 11);
    }

    #[test]
    fn a_garbled_stall_or_swap_refuses_rather_than_reading_as_unread() {
        let garbled_stall =
            vec![stat_line("seam-parse", "parse", "1")
                .replace("stall_per_min=na", "stall_per_min=-3")];
        let refused =
            transcribe_floor_beats(&garbled_stall).expect_err("a negative stall must refuse");
        assert!(refused.contains("stall_per_min=-3"), "{refused}");
        let garbled_swap = vec![
            "[floor-cgroup] when=seam-parse seam=parse unix_ms=1 swap_current=4k events_local=[] pressure=[] procs=[]".to_string(),
            stat_line("seam-parse", "parse", "1"),
        ];
        let refused =
            transcribe_floor_beats(&garbled_swap).expect_err("a garbled swap must refuse");
        assert!(refused.contains("swap_current=4k"), "{refused}");
    }

    #[test]
    fn an_unread_stat_counter_refuses_rather_than_reading_zero() {
        let lines = vec![stat_line("seam-parse", "parse", "na")];
        let refused = transcribe_floor_beats(&lines).expect_err("an `na` anon must refuse");
        assert!(refused.contains("memory_stat anon"), "{refused}");
    }
}
