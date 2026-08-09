//! Bounded concurrent shell stream capture for the v1 interpreter seed.
//!
//! Authority: `dag/std/shell_stream_capture.dag`. Drains stdout/stderr while the
//! child runs; no stream may grow an unbounded owned `String` in the runtime.

use std::io::Read;
use std::process::{Child, ExitStatus};

const FNV1A64_OFFSET: u64 = 0xcbf29ce484222325;
const FNV1A64_PRIME: u64 = 0x100000001b3;

const READ_CHUNK_BYTES: usize = 64 * 1024;

/// Seed realization of `std.shell_stream_capture.StreamCapturePolicy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamCapturePolicy {
    Discarded,
    Bounded { max_retained_bytes: usize },
    DigestAndBoundedTail { max_tail_bytes: usize },
}

/// Seed realization of `std.shell_stream_capture.CapturedStreamObservation`.
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

pub const DEFAULT_SHELL_STDOUT_MAX_RETAINED_BYTES: usize = 8 * 1024 * 1024;
pub const DEFAULT_SHELL_STDERR_TAIL_BYTES: usize = 16 * 1024;

pub fn default_shell_stdout_capture_policy() -> StreamCapturePolicy {
    StreamCapturePolicy::Bounded {
        max_retained_bytes: DEFAULT_SHELL_STDOUT_MAX_RETAINED_BYTES,
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

/// Drain one captured pipe according to `policy` while the child is still running.
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

    let max_bounded = match policy {
        StreamCapturePolicy::Bounded { max_retained_bytes } => max_retained_bytes,
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
            StreamCapturePolicy::Bounded { .. } => {
                if retained.len() < max_bounded {
                    let take = max_bounded.saturating_sub(retained.len()).min(n);
                    retained.extend_from_slice(&chunk[..take]);
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
        StreamCapturePolicy::Bounded { .. } => retained,
        StreamCapturePolicy::DigestAndBoundedTail { .. } => ring
            .expect("ring buffer initialized for tail policy")
            .into_vec(),
    };

    let retained_len = retained.len() as u64;
    let truncated = total_bytes > retained_len;

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

/// Wait for `child` while concurrently draining stdout/stderr with bounded policies.
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
    fn bounded_retains_prefix_only() {
        let data = vec![b'a'; 1024];
        let obs = drain_stream(
            Cursor::new(data),
            StreamCapturePolicy::Bounded {
                max_retained_bytes: 64,
            },
        )
        .expect("drain");
        assert_eq!(obs.total_bytes, 1024);
        assert_eq!(obs.retained.len(), 64);
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
