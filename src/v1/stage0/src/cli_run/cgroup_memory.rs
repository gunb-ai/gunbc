//! Generic cgroup v2 memory readers: the memory interface files, swap and their refusals. The
//! consumers are the live `gunbc test` memory-envelope seam.

use std::path::Path;

/// One cgroup's memory interface, read as text so an unparseable body stays distinguishable from
/// an absent file. Authority for the file names: `extdeps.linux.cgroup_v2_memory`
/// `cgroup_memory_interface_file_name`.
#[derive(Debug, Clone)]
pub struct CgroupMemoryRead {
    pub dir: String,
    pub peak: u64,
    pub limit_max: String,
    pub limit_high: String,
    pub high_events: u64,
    pub max_events: u64,
    pub oom_kills: u64,
}

/// WHY A READ CANNOT SILENTLY RETURN ZERO. Each arm names a distinct thing that went wrong, and
/// none of them may render as "it fit".
#[derive(Debug, Clone)]
pub enum QualificationRefusal {
    CgroupFileUnreadable { path: String, detail: String },
    CgroupValueUnparseable { file: String, body: String },
}

impl QualificationRefusal {
    pub fn render(&self) -> String {
        match self {
            QualificationRefusal::CgroupFileUnreadable { path, detail } => {
                format!("CgroupFileUnreadable — {path}: {detail}")
            }
            QualificationRefusal::CgroupValueUnparseable { file, body } => {
                format!("CgroupValueUnparseable — {file} held {body:?}")
            }
        }
    }
}

fn read_file(dir: &Path, name: &str) -> Result<String, QualificationRefusal> {
    let path = dir.join(name);
    std::fs::read_to_string(&path)
        .map(|s| s.trim().to_string())
        .map_err(|e| QualificationRefusal::CgroupFileUnreadable {
            path: path.to_string_lossy().to_string(),
            detail: e.to_string(),
        })
}

fn read_count(dir: &Path, name: &str) -> Result<u64, QualificationRefusal> {
    let body = read_file(dir, name)?;
    body.parse::<u64>()
        .map_err(|_| QualificationRefusal::CgroupValueUnparseable {
            file: name.to_string(),
            body,
        })
}

/// `memory.events` is `key value` lines. A MISSING KEY IS NOT A ZERO: the file documents a fixed
/// key set, so a key that is not there means this kernel did not report it, and reporting that as
/// "no events occurred" is the substitution this whole instrument exists to refuse.
fn events_field(body: &str, field: &str) -> Result<u64, QualificationRefusal> {
    body.lines()
        .find_map(|l| {
            let mut it = l.split_whitespace();
            match (it.next(), it.next()) {
                (Some(k), Some(v)) if k == field => Some(v.parse::<u64>().ok()),
                _ => None,
            }
        })
        .flatten()
        .ok_or_else(|| QualificationRefusal::CgroupValueUnparseable {
            file: "memory.events".to_string(),
            body: format!("no parseable {field} key in {body:?}"),
        })
}

pub fn read_cgroup_memory(dir: &Path) -> Result<CgroupMemoryRead, QualificationRefusal> {
    let events_body = read_file(dir, "memory.events")?;
    Ok(CgroupMemoryRead {
        dir: dir.to_string_lossy().to_string(),
        peak: read_count(dir, "memory.peak")?,
        limit_max: read_file(dir, "memory.max")?,
        limit_high: read_file(dir, "memory.high")?,
        high_events: events_field(&events_body, "high")?,
        max_events: events_field(&events_body, "max")?,
        oom_kills: events_field(&events_body, "oom_kill")?,
    })
}

/// The swap half of a cgroup's envelope (extdeps.linux.cgroup_v2_memory MemorySwapMax,
/// MemorySwapCurrent, MemorySwapPeak): `memory.swap.max` as written (`max` when unlimited -- a third
/// state, not a large number), `memory.swap.current` (resident swap, an instantaneous gauge) and
/// `memory.swap.peak` (the lifetime maximum, ABSENT on kernels that predate it -- reported as absent,
/// never as zero, because a reader that cannot see the peak must say so rather than report no rise).
/// An unreadable max or gauge is a refusal like every other file here.
pub struct CgroupSwapRead {
    pub swap_max: String,
    pub swap_current: u64,
    pub swap_peak: Option<u64>,
}

pub fn read_cgroup_swap(dir: &Path) -> Result<CgroupSwapRead, QualificationRefusal> {
    let swap_peak = if dir.join("memory.swap.peak").is_file() {
        Some(read_count(dir, "memory.swap.peak")?)
    } else {
        None
    };
    Ok(CgroupSwapRead {
        swap_max: read_file(dir, "memory.swap.max")?,
        swap_current: read_count(dir, "memory.swap.current")?,
        swap_peak,
    })
}
