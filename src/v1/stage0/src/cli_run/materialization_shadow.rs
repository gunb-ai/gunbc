//! Shadow one compiler question at the real provider-round boundary. Inspect the previous record
//! before the ordinary compile; hash the actual NewlineIndex returned by that compile afterwards.
//! Records contain only digests and cannot supply a compiler result. The producer and its result
//! are untouched, including when the experiment is unreadable or observations disagree.

use crate::v1_compiler_compile::SourceFile;
use crate::v1_std_core::NewlineIndex;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::Instant;

const QUESTION: &str = "v1.std.core::build_newline_index";
const SCHEMA: &[u8] = b"newline-index-shadow-v1";
type Fingerprint = [u8; 32];

enum Prior {
    Absent,
    Observed(Fingerprint),
    Corrupt,
    Unreadable(io::Error),
}

struct Probe {
    key: Fingerprint,
    prior: Prior,
    lookup_ns: u64,
}

pub(super) struct Round {
    directory: PathBuf,
    probes: Vec<Probe>,
}

fn issue(reason: impl std::fmt::Display) {
    let _ = writeln!(
        io::stderr().lock(),
        "[materialization-shadow-error] {reason}"
    );
}

fn compiler_image() -> Result<&'static Fingerprint, &'static str> {
    static IMAGE: OnceLock<Result<Fingerprint, String>> = OnceLock::new();
    IMAGE
        .get_or_init(|| {
            let result = || -> io::Result<Fingerprint> {
                #[cfg(target_os = "linux")]
                let mut file = File::open("/proc/self/exe")?;
                #[cfg(not(target_os = "linux"))]
                let mut file = File::open(std::env::current_exe()?)?;
                let mut hash = Sha256::new();
                io::copy(&mut file, &mut hash)?;
                Ok(hash.finalize().into())
            };
            result().map_err(|error| error.to_string())
        })
        .as_ref()
        .map_err(String::as_str)
}

fn field(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_le_bytes());
    hash.update(bytes);
}

fn input_key(image: &Fingerprint, source: &SourceFile) -> Fingerprint {
    let mut hash = Sha256::new();
    field(&mut hash, SCHEMA);
    field(&mut hash, QUESTION.as_bytes());
    field(&mut hash, image);
    field(&mut hash, source.path.as_bytes());
    field(&mut hash, source.content.as_bytes());
    hash.finalize().into()
}

fn answer(index: &NewlineIndex) -> (Fingerprint, u64) {
    // Stream complete, length-framed fields in declaration order. Offsets and code points are
    // the producer's signed 64-bit scalars; no serialization of the closure or derived positions.
    let mut hash = Sha256::new();
    field(&mut hash, index.file.as_bytes());
    let mut bytes = 8 + index.file.len() as u64;
    for list in [&index.offsets, &index.char_codes] {
        hash.update((list.len() as u64).to_le_bytes());
        bytes += 8 + 8 * list.len() as u64;
        for value in list.iter() {
            hash.update(value.to_le_bytes());
        }
    }
    (hash.finalize().into(), bytes)
}

fn record(key: &Fingerprint, value: &Fingerprint) -> [u8; 64] {
    let mut hash = Sha256::new();
    hash.update(SCHEMA);
    hash.update(key);
    hash.update(value);
    let mut bytes = [0; 64];
    bytes[..32].copy_from_slice(value);
    bytes[32..].copy_from_slice(&hash.finalize());
    bytes
}

fn read_prior(path: &Path, key: &Fingerprint) -> Prior {
    let mut open = fs::OpenOptions::new();
    open.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        // A damaged experiment entry can be a FIFO or symlink. It must not block the compiler.
        open.custom_flags(libc::O_NONBLOCK | libc::O_NOFOLLOW);
    }
    let file = match open.open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Prior::Absent,
        Err(error) => return Prior::Unreadable(error),
    };
    match file.metadata() {
        Ok(metadata) if metadata.is_file() && metadata.len() == 64 => (),
        Ok(_) => return Prior::Corrupt,
        Err(error) => return Prior::Unreadable(error),
    }
    let mut bytes = Vec::new();
    if let Err(error) = file.take(65).read_to_end(&mut bytes) {
        return Prior::Unreadable(error);
    }
    let Ok(slice) = <&[u8; 64]>::try_from(bytes.as_slice()) else {
        return Prior::Corrupt;
    };
    let mut value = [0; 32];
    value.copy_from_slice(&slice[..32]);
    if &record(key, &value) == slice {
        Prior::Observed(value)
    } else {
        Prior::Corrupt
    }
}

pub(super) fn begin(sources: &[Rc<SourceFile>]) -> Option<Round> {
    let root = PathBuf::from(std::env::var_os("GUNBC_MATERIALIZATION_SHADOW_DIR")?);
    if !root.is_absolute() {
        issue("GUNBC_MATERIALIZATION_SHADOW_DIR must be absolute");
        return None;
    }
    let directory = root.join("newline-index-v1");
    if let Err(error) = fs::create_dir_all(&directory) {
        issue(error);
        return None;
    }
    let image = match compiler_image() {
        Ok(image) => image,
        Err(error) => {
            issue(error);
            return None;
        }
    };
    let probes = sources
        .iter()
        .map(|source| {
            let start = Instant::now();
            let key = input_key(image, source);
            let prior = read_prior(&directory.join(hex::encode(key)), &key);
            Probe {
                key,
                prior,
                lookup_ns: start.elapsed().as_nanos() as u64,
            }
        })
        .collect();
    Some(Round { directory, probes })
}

pub(super) fn observe(
    round: Round,
    sources: &[Rc<SourceFile>],
    indices: &im::Vector<Rc<NewlineIndex>>,
) {
    if sources.len() != indices.len() {
        issue(format!(
            "source/index count mismatch: {} / {}",
            sources.len(),
            indices.len()
        ));
        return;
    }
    for ((source, index), probe) in sources.iter().zip(indices).zip(round.probes) {
        if source.path != index.file {
            issue(format!(
                "source/index path mismatch: {:?} / {:?}",
                source.path, index.file
            ));
            continue;
        }
        let (fresh, bytes) = answer(index);
        let path = round.directory.join(hex::encode(probe.key));
        let (lookup, comparison) = compare_and_record(&path, &probe.key, probe.prior, &fresh);
        let event = serde_json::json!({
            "question": QUESTION, "logical_source": source.path,
            "input_digest": hex::encode(probe.key), "output_digest": hex::encode(fresh),
            "lookup": lookup, "comparison": comparison, "bytes": bytes,
            "producer_ns": null, "lookup_ns": probe.lookup_ns,
        });
        let _ = writeln!(io::stderr().lock(), "[materialization-shadow] {event}");
    }
}

fn compare_and_record(
    path: &Path,
    key: &Fingerprint,
    prior: Prior,
    fresh: &Fingerprint,
) -> (&'static str, &'static str) {
    match prior {
        Prior::Absent => {
            // Atomic create-only. A concurrent winner is read and checked against the
            // actual fresh result. Never overwrite a discrepancy or a damaged record.
            let publish = path
                .to_str()
                .ok_or_else(|| io::Error::other("shadow path is not UTF-8"))
                .and_then(|p| {
                    crate::v1_rt::gunbc_file_write_create_new(p, &record(key, fresh), None)
                });
            match publish {
                Ok(()) => ("miss", "absent"),
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => match read_prior(path, key) {
                    Prior::Observed(value) if value == *fresh => ("miss", "absent"),
                    Prior::Observed(_) => ("raced", "different"),
                    Prior::Corrupt => ("corrupt", "unavailable"),
                    Prior::Unreadable(e) => {
                        issue(e);
                        ("unreadable", "unavailable")
                    }
                    Prior::Absent => ("unreadable", "unavailable"),
                },
                Err(error) => {
                    issue(error);
                    ("unwritable", "unavailable")
                }
            }
        }
        Prior::Observed(value) if value == *fresh => ("hit", "equal"),
        Prior::Observed(_) => ("hit", "different"),
        Prior::Corrupt => ("corrupt", "unavailable"),
        Prior::Unreadable(error) => {
            issue(error);
            ("unreadable", "unavailable")
        }
    }
}

#[test]
fn a_valid_prior_disagreement_and_an_integrity_failure_are_distinct() {
    let dir = std::env::temp_dir().join(format!(
        "gunbc-shadow-control-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&dir).unwrap();
    let key = [41; 32];
    let path = dir.join(hex::encode(key));
    let fresh_index = crate::v1_std_core::build_newline_index("sample.dag".into(), "a\n".into());
    let (fresh, _) = answer(&fresh_index);
    let prior_index = crate::v1_std_core::build_newline_index("sample.dag".into(), "b\n".into());
    let (different, _) = answer(&prior_index);
    assert_ne!(fresh, different);
    let before = record(&key, &different);
    fs::write(&path, before).unwrap();
    assert_eq!(
        compare_and_record(&path, &key, read_prior(&path, &key), &fresh),
        ("hit", "different")
    );
    assert_eq!(
        fs::read(&path).unwrap(),
        before,
        "a disagreement preserves its evidence"
    );
    let mut damaged = before;
    damaged[32] ^= 1;
    fs::write(&path, damaged).unwrap();
    assert_eq!(
        compare_and_record(&path, &key, read_prior(&path, &key), &fresh),
        ("corrupt", "unavailable")
    );
    assert_eq!(fs::read(&path).unwrap(), damaged);
    fs::remove_dir_all(dir).unwrap();
}
