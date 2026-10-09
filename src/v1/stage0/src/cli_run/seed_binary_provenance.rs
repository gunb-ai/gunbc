//! Runtime realization of `gunbc.seed_binary_provenance`.
//!
//! Wire labels and field order are the `.dag` data rows / `seed_binary_provenance_wire`.
//! This parser is pinned to that writer by `test.claim.seed_binary_provenance_witness`
//! (round-trip plus pack-script label inhabitance) and by the unit tests below using the
//! same three-line format.

use sha2::Digest;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const BINARY_SHA256_LABEL: &str = "binary_sha256";
const SOURCE_COMMIT_LABEL: &str = "source_commit";
const TREE_IDENTITY_LABEL: &str = "tree_identity";
const SIDECAR_SUFFIX: &str = ".provenance";
pub const UNVERIFIED_VERSION_TEXT: &str = "provenance-unverified";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeedBinaryProvenance {
    pub binary_sha256: String,
    pub source_commit: String,
    pub tree_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvenanceParse {
    Parsed(SeedBinaryProvenance),
    Malformed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProvenanceStanding {
    Verified(SeedBinaryProvenance),
    Unverified,
}

pub fn sidecar_path_for_exe(exe: &Path) -> PathBuf {
    let mut name = exe
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(SIDECAR_SUFFIX);
    exe.with_file_name(name)
}

pub fn wire_text(record: &SeedBinaryProvenance) -> String {
    format!(
        "{BINARY_SHA256_LABEL} {}\n{SOURCE_COMMIT_LABEL} {}\n{TREE_IDENTITY_LABEL} {}\n",
        record.binary_sha256, record.source_commit, record.tree_identity
    )
}

fn labeled<'a>(line: &'a str, label: &str) -> Option<&'a str> {
    line.strip_prefix(label)
        .and_then(|rest| rest.strip_prefix(' '))
}

fn is_lower_hex(s: &str, len: usize) -> bool {
    s.len() == len
        && s.bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

fn tree_identity_holds(value: &str) -> bool {
    let mut parts = value.split(':');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some("tree"), Some("sha1"), Some(hex), None) => is_lower_hex(hex, 40),
        (Some("tree"), Some("sha256"), Some(hex), None) => is_lower_hex(hex, 64),
        _ => false,
    }
}

pub fn parse_wire(text: &str) -> ProvenanceParse {
    let lines: Vec<&str> = text.split('\n').collect();
    if lines.len() != 4 || lines[3] != "" {
        return ProvenanceParse::Malformed;
    }
    let Some(sha) = labeled(lines[0], BINARY_SHA256_LABEL).filter(|h| is_lower_hex(h, 64)) else {
        return ProvenanceParse::Malformed;
    };
    let Some(commit) = labeled(lines[1], SOURCE_COMMIT_LABEL).filter(|h| is_lower_hex(h, 40))
    else {
        return ProvenanceParse::Malformed;
    };
    let Some(tree) = labeled(lines[2], TREE_IDENTITY_LABEL).filter(|t| tree_identity_holds(t))
    else {
        return ProvenanceParse::Malformed;
    };
    ProvenanceParse::Parsed(SeedBinaryProvenance {
        binary_sha256: sha.to_string(),
        source_commit: commit.to_string(),
        tree_identity: tree.to_string(),
    })
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", sha2::Sha256::digest(bytes))
}

pub fn verify_record(record: &SeedBinaryProvenance, running_sha256: &str) -> ProvenanceStanding {
    if record.binary_sha256 == running_sha256 {
        ProvenanceStanding::Verified(record.clone())
    } else {
        ProvenanceStanding::Unverified
    }
}

fn running_image() -> Option<PathBuf> {
    let proc = PathBuf::from("/proc/self/exe");
    if proc.exists() {
        Some(proc)
    } else {
        std::env::current_exe().ok()
    }
}

pub fn standing_for_running_image() -> ProvenanceStanding {
    let Some(exe) = running_image() else {
        return ProvenanceStanding::Unverified;
    };
    let Ok(bytes) = std::fs::read(&exe) else {
        return ProvenanceStanding::Unverified;
    };
    if bytes.is_empty() {
        return ProvenanceStanding::Unverified;
    }
    let digest = sha256_hex(&bytes);
    let path = sidecar_path_for_exe(&exe);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return ProvenanceStanding::Unverified;
    };
    match parse_wire(&text) {
        ProvenanceParse::Malformed => ProvenanceStanding::Unverified,
        ProvenanceParse::Parsed(record) => verify_record(&record, &digest),
    }
}

pub fn clap_version_text() -> &'static str {
    static VERSION: OnceLock<String> = OnceLock::new();
    VERSION
        .get_or_init(|| match standing_for_running_image() {
            ProvenanceStanding::Verified(record) => record.source_commit,
            ProvenanceStanding::Unverified => UNVERIFIED_VERSION_TEXT.to_string(),
        })
        .as_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> SeedBinaryProvenance {
        SeedBinaryProvenance {
            binary_sha256: "a".repeat(64),
            source_commit: "c".repeat(40),
            tree_identity: format!("tree:sha1:{}", "b".repeat(40)),
        }
    }

    #[test]
    fn wire_round_trips() {
        let record = fixture();
        match parse_wire(&wire_text(&record)) {
            ProvenanceParse::Parsed(back) => assert_eq!(back, record),
            ProvenanceParse::Malformed => panic!("fixture wire must parse"),
        }
    }

    #[test]
    fn missing_sidecar_is_unverified() {
        assert_eq!(standing_for_running_image(), ProvenanceStanding::Unverified);
    }

    #[test]
    fn mismatched_digest_is_unverified() {
        let record = fixture();
        assert_eq!(
            verify_record(&record, &"d".repeat(64)),
            ProvenanceStanding::Unverified
        );
    }

    #[test]
    fn matching_digest_is_verified() {
        let record = fixture();
        assert_eq!(
            verify_record(&record, &record.binary_sha256),
            ProvenanceStanding::Verified(record)
        );
    }

    #[test]
    fn reordered_labels_are_malformed() {
        let record = fixture();
        let swapped = wire_text(&record).replacen(BINARY_SHA256_LABEL, SOURCE_COMMIT_LABEL, 1);
        match parse_wire(&swapped) {
            ProvenanceParse::Malformed => {}
            ProvenanceParse::Parsed(_) => panic!("reordered labels must refuse"),
        }
    }
}
