//! Shadow one compiler question at the real provider-round boundary. Inspect the previous record
//! before the ordinary compile; hash the actual NewlineIndex returned by that compile afterwards.
//! The store contains only digests. An explicitly requested observation must complete and agree
//! with the fresh result before the selected entry can run.

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
    compiler_image_sha256: String,
    probes: Vec<Probe>,
}

fn refusal(reason: impl std::fmt::Display) -> String {
    format!("[materialization-shadow-error] {QUESTION}: {reason}")
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

pub(super) fn begin(sources: &[Rc<SourceFile>]) -> Result<Option<Round>, String> {
    let Some(root) = std::env::var_os("GUNBC_MATERIALIZATION_SHADOW_DIR") else {
        return Ok(None);
    };
    let root = PathBuf::from(root);
    if !root.is_absolute() {
        return Err(refusal("GUNBC_MATERIALIZATION_SHADOW_DIR must be absolute"));
    }
    let directory = root.join("newline-index-v1");
    fs::create_dir_all(&directory).map_err(|error| {
        refusal(format!(
            "cannot initialize {}: {error}",
            directory.display()
        ))
    })?;
    let image = compiler_image()
        .map_err(|error| refusal(format!("cannot identify the running compiler: {error}")))?;
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
    Ok(Some(Round {
        directory,
        compiler_image_sha256: hex::encode(image),
        probes,
    }))
}

pub(super) fn observe(
    round: Round,
    sources: &[Rc<SourceFile>],
    indices: &im::Vector<Rc<NewlineIndex>>,
    mut events: impl Write,
) -> Result<(), String> {
    if sources.is_empty() {
        return Err(refusal("empty observation: the round has no sources"));
    }
    if sources.len() != indices.len() || sources.len() != round.probes.len() {
        return Err(refusal(format!(
            "incomplete observation: {} sources, {} indices, {} probes",
            sources.len(),
            indices.len(),
            round.probes.len()
        )));
    }
    let mut first_refusal = None;
    for ((source, index), probe) in sources.iter().zip(indices).zip(round.probes) {
        if source.path != index.file {
            return Err(refusal(format!(
                "source/index path mismatch: {:?} / {:?}",
                source.path, index.file
            )));
        }
        let (fresh, bytes) = answer(index);
        let path = round.directory.join(hex::encode(probe.key));
        let (lookup, comparison, reason) =
            match compare_and_record(&path, &probe.key, probe.prior, &fresh) {
                Observation::Published => ("miss", "absent", None),
                Observation::Equal => ("hit", "equal", None),
                Observation::Different(lookup) => (
                    lookup,
                    "different",
                    Some("stored answer differs from fresh".to_owned()),
                ),
                Observation::Corrupt => (
                    "corrupt",
                    "unavailable",
                    Some("invalid stored record".to_owned()),
                ),
                Observation::Io(lookup, error) => (lookup, "unavailable", Some(error.to_string())),
            };
        let event = serde_json::json!({
            "question": QUESTION, "logical_source": source.path,
            "compiler_image_sha256": round.compiler_image_sha256,
            "input_digest": hex::encode(probe.key), "output_digest": hex::encode(fresh),
            "lookup": lookup, "comparison": comparison, "bytes": bytes,
            "producer_ns": null, "lookup_ns": probe.lookup_ns,
        });
        writeln!(events, "[materialization-shadow] {event}").map_err(|error| {
            refusal(format!(
                "cannot write observation for {}: {error}",
                source.path
            ))
        })?;
        if let Some(reason) = reason {
            first_refusal.get_or_insert_with(|| {
                refusal(format!("{}: {lookup}/{comparison}: {reason}", source.path))
            });
        }
    }
    first_refusal.map_or(Ok(()), Err)
}

enum Observation {
    Published,
    Equal,
    Different(&'static str),
    Corrupt,
    Io(&'static str, io::Error),
}

fn compare_and_record(
    path: &Path,
    key: &Fingerprint,
    prior: Prior,
    fresh: &Fingerprint,
) -> Observation {
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
                Ok(()) => Observation::Published,
                Err(e) if e.kind() == io::ErrorKind::AlreadyExists => match read_prior(path, key) {
                    Prior::Observed(value) if value == *fresh => Observation::Published,
                    Prior::Observed(_) => Observation::Different("raced"),
                    Prior::Corrupt => Observation::Corrupt,
                    Prior::Unreadable(e) => Observation::Io("unreadable", e),
                    Prior::Absent => Observation::Io(
                        "unreadable",
                        io::Error::other("concurrent record disappeared"),
                    ),
                },
                Err(error) => Observation::Io("unwritable", error),
            }
        }
        Prior::Observed(value) if value == *fresh => Observation::Equal,
        Prior::Observed(_) => Observation::Different("hit"),
        Prior::Corrupt => Observation::Corrupt,
        Prior::Unreadable(error) => Observation::Io("unreadable", error),
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
    assert!(matches!(
        compare_and_record(&path, &key, read_prior(&path, &key), &fresh),
        Observation::Different("hit")
    ));
    assert!(matches!(
        compare_and_record(&path, &key, Prior::Absent, &fresh),
        Observation::Different("raced")
    ));
    assert_eq!(
        fs::read(&path).unwrap(),
        before,
        "a disagreement preserves its evidence"
    );
    let mut damaged = before;
    damaged[32] ^= 1;
    fs::write(&path, damaged).unwrap();
    assert!(matches!(
        compare_and_record(&path, &key, read_prior(&path, &key), &fresh),
        Observation::Corrupt
    ));
    assert_eq!(fs::read(&path).unwrap(), damaged);
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn missing_or_undeliverable_observations_refuse_the_round() {
    let sources = [Rc::new(SourceFile {
        path: "sample.dag".into(),
        content: "a".into(),
    })];
    let index = crate::v1_std_core::build_newline_index("sample.dag".into(), "a".into());
    let round = |probes| Round {
        directory: PathBuf::new(),
        compiler_image_sha256: hex::encode([0u8; 32]),
        probes,
    };
    let probe = || Probe {
        key: [0; 32],
        prior: Prior::Observed(answer(&index).0),
        lookup_ns: 0,
    };
    let indices = im::vector![index.clone()];
    let empty = observe(round(vec![]), &[], &im::Vector::new(), io::sink()).unwrap_err();
    assert!(
        empty.contains("empty observation: the round has no sources"),
        "{empty}"
    );
    let missing_probe = observe(round(vec![]), &sources, &indices, io::sink()).unwrap_err();
    assert!(
        missing_probe.contains("1 sources, 1 indices, 0 probes"),
        "{missing_probe}"
    );
    let missing_index = observe(
        round(vec![probe()]),
        &sources,
        &im::Vector::new(),
        io::sink(),
    )
    .unwrap_err();
    assert!(
        missing_index.contains("1 sources, 0 indices, 1 probes"),
        "{missing_index}"
    );

    let wrong = crate::v1_std_core::build_newline_index("other.dag".into(), "a".into());
    let wrong_source = observe(
        round(vec![probe()]),
        &sources,
        &im::vector![wrong],
        io::sink(),
    )
    .unwrap_err();
    assert!(
        wrong_source.contains("source/index path mismatch"),
        "{wrong_source}"
    );

    let mut full = [];
    let undelivered = observe(round(vec![probe()]), &sources, &indices, &mut full[..]).unwrap_err();
    assert!(
        undelivered.contains("cannot write observation for sample.dag"),
        "{undelivered}"
    );
}
