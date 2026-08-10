//! Interim v1_interpreter seed host-effect: bounded concurrent shell stream drain.
//!
//! Scaffold authority: `std.shell_stream_capture` (`seed_host_shell_stream_bounded_drain_*`).
//! Not a modeled policy surface — constants and refusal semantics live here until
//! emit-on-demand native serve and the shared `CapturedProcessStream` carrier land.

use std::io::Read;
use std::process::{Child, ExitStatus};

const FNV1A64_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A64_PRIME: u64 = 0x100000001b3;

const READ_CHUNK_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StreamCapturePolicy {
    Discarded,
    /// Stdout default: retain only when the full stream fits; overflow sets `truncated`.
    CompleteWithin {
        max_bytes: usize,
    },
    /// Stderr default: total byte count + digest + bounded tail; truncation explicit.
    DigestAndBoundedTail {
        max_tail_bytes: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamCaptureObservation {
    pub total_bytes: u64,
    pub retained: Vec<u8>,
    pub truncated: bool,
    pub digest_hex: Option<String>,
}

impl StreamCaptureObservation {
    pub fn retained_utf8_lossy_trimmed(&self) -> String {
        String::from_utf8_lossy(&self.retained)
            .trim_end()
            .to_string()
    }

    pub fn retained_bytes(&self) -> usize {
        self.retained.len()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellCaptureResult {
    pub exit_status: ExitStatus,
    pub stdout: StreamCaptureObservation,
    pub stderr: StreamCaptureObservation,
}

pub const DEFAULT_SHELL_STDOUT_MAX_BYTES: usize = 8 * 1024 * 1024;
pub const DEFAULT_SHELL_STDERR_TAIL_BYTES: usize = 16 * 1024;

pub fn default_shell_stdout_capture_policy() -> StreamCapturePolicy {
    StreamCapturePolicy::CompleteWithin {
        max_bytes: DEFAULT_SHELL_STDOUT_MAX_BYTES,
    }
}

pub fn default_shell_stderr_capture_policy() -> StreamCapturePolicy {
    StreamCapturePolicy::DigestAndBoundedTail {
        max_tail_bytes: DEFAULT_SHELL_STDERR_TAIL_BYTES,
    }
}

fn fnv1a64_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for b in bytes {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(FNV1A64_PRIME);
    }
    hash
}

struct RingTail {
    capacity: usize,
    buf: Vec<u8>,
    len: usize,
    start: usize,
}

impl RingTail {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            buf: Vec::with_capacity(capacity),
            len: 0,
            start: 0,
        }
    }

    fn push_byte(&mut self, b: u8) {
        if self.capacity == 0 {
            return;
        }
        if self.len < self.capacity {
            if self.buf.len() < self.capacity {
                self.buf.push(b);
            } else {
                self.buf[self.start] = b;
                self.start = (self.start + 1) % self.capacity;
            }
            self.len += 1;
            return;
        }
        self.buf[self.start] = b;
        self.start = (self.start + 1) % self.capacity;
    }

    fn push_slice(&mut self, chunk: &[u8]) {
        for &b in chunk {
            self.push_byte(b);
        }
    }

    fn into_vec(self) -> Vec<u8> {
        if self.len < self.capacity {
            return self.buf;
        }
        let mut out = Vec::with_capacity(self.capacity);
        for i in 0..self.capacity {
            out.push(self.buf[(self.start + i) % self.capacity]);
        }
        out
    }
}

pub fn drain_stream<R: Read>(
    mut reader: R,
    policy: StreamCapturePolicy,
) -> std::io::Result<StreamCaptureObservation> {
    let mut buf = [0u8; READ_CHUNK_BYTES];
    let mut total_bytes: u64 = 0;
    let mut retained = Vec::new();
    let mut ring = match policy {
        StreamCapturePolicy::DigestAndBoundedTail { max_tail_bytes } => {
            Some(RingTail::new(max_tail_bytes))
        }
        _ => None,
    };
    let mut digest = FNV1A64_OFFSET;
    let track_digest = matches!(policy, StreamCapturePolicy::DigestAndBoundedTail { .. });

    let max_complete = match policy {
        StreamCapturePolicy::CompleteWithin { max_bytes } => max_bytes,
        _ => usize::MAX,
    };

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        total_bytes += n as u64;
        let chunk = &buf[..n];
        if track_digest {
            digest = fnv1a64_update(digest, chunk);
        }

        match policy {
            StreamCapturePolicy::Discarded => {}
            StreamCapturePolicy::CompleteWithin { .. } => {
                if total_bytes <= max_complete as u64 {
                    retained.extend_from_slice(chunk);
                }
            }
            StreamCapturePolicy::DigestAndBoundedTail { .. } => {
                if let Some(ring) = ring.as_mut() {
                    ring.push_slice(chunk);
                }
            }
        }
    }

    let retained = match policy {
        StreamCapturePolicy::Discarded => Vec::new(),
        StreamCapturePolicy::CompleteWithin { max_bytes } => {
            if total_bytes > max_bytes as u64 {
                Vec::new()
            } else {
                retained
            }
        }
        StreamCapturePolicy::DigestAndBoundedTail { .. } => ring
            .expect("ring buffer initialized for tail policy")
            .into_vec(),
    };

    let retained_len = retained.len() as u64;
    let truncated = match policy {
        StreamCapturePolicy::CompleteWithin { max_bytes } => total_bytes > max_bytes as u64,
        _ => total_bytes > retained_len,
    };

    Ok(StreamCaptureObservation {
        total_bytes,
        retained,
        truncated,
        digest_hex: if track_digest {
            Some(format!("{:016x}", digest))
        } else {
            None
        },
    })
}

pub fn capture_child_output(
    mut child: Child,
    stdout_policy: StreamCapturePolicy,
    stderr_policy: StreamCapturePolicy,
) -> std::io::Result<ShellCaptureResult> {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let stdout_handle =
        stdout.map(|out| std::thread::spawn(move || drain_stream(out, stdout_policy)));
    let stderr_handle =
        stderr.map(|err| std::thread::spawn(move || drain_stream(err, stderr_policy)));

    let exit_status = child.wait()?;

    let stdout = match stdout_handle {
        Some(handle) => handle.join().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "stdout drain thread panicked")
        })??,
        None => StreamCaptureObservation {
            total_bytes: 0,
            retained: Vec::new(),
            truncated: false,
            digest_hex: None,
        },
    };
    let stderr = match stderr_handle {
        Some(handle) => handle.join().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "stderr drain thread panicked")
        })??,
        None => StreamCaptureObservation {
            total_bytes: 0,
            retained: Vec::new(),
            truncated: false,
            digest_hex: None,
        },
    };

    Ok(ShellCaptureResult {
        exit_status,
        stdout,
        stderr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn complete_within_retains_only_when_fits() {
        let data = vec![b'a'; 64];
        let obs = drain_stream(
            Cursor::new(data.clone()),
            StreamCapturePolicy::CompleteWithin { max_bytes: 128 },
        )
        .expect("drain");
        assert_eq!(obs.total_bytes, 64);
        assert_eq!(obs.retained, data);
        assert!(!obs.truncated);
    }

    #[test]
    fn complete_within_overflow_does_not_retain_prefix() {
        let data = vec![b'a'; 1024];
        let obs = drain_stream(
            Cursor::new(data),
            StreamCapturePolicy::CompleteWithin { max_bytes: 64 },
        )
        .expect("drain");
        assert_eq!(obs.total_bytes, 1024);
        assert!(obs.retained.is_empty());
        assert!(obs.truncated);
    }

    #[test]
    fn tail_policy_retains_suffix_only() {
        let data: Vec<u8> = (0..256).map(|i| (i % 251) as u8).collect();
        let obs = drain_stream(
            Cursor::new(data.clone()),
            StreamCapturePolicy::DigestAndBoundedTail { max_tail_bytes: 16 },
        )
        .expect("drain");
        assert_eq!(obs.total_bytes, 256);
        assert_eq!(obs.retained.len(), 16);
        assert_eq!(&obs.retained, &data[240..]);
        assert!(obs.truncated);
        assert!(obs.digest_hex.is_some());
    }

    #[test]
    fn discarded_counts_without_retaining() {
        let data = vec![0u8; 4096];
        let obs = drain_stream(Cursor::new(data), StreamCapturePolicy::Discarded).expect("drain");
        assert_eq!(obs.total_bytes, 4096);
        assert!(obs.retained.is_empty());
        assert!(obs.truncated);
    }
}
