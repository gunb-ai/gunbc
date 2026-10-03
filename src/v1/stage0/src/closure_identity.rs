// Seed-retained keying (§7 seed-retained HAND-RUST). These derivations name WHAT a resolve or a
// witness is about -- a closure's content, the compiler that judges it, and a witness function
// over that closure -- and are consumed by the in-process resolved-graph memo, the typed-module
// key (std.interface_summary typed_module_key), the floor's per-row work key and the emitted
// compiler's identity. They are keying, not caching: they moved here unchanged from the deleted
// cross-process resolved-graph disk tier, whose store never served a hit after its JSON
// representation was disarmed. The seed strings below are preserved byte-for-byte so every key a
// receipt already carries stays the same key.
//
// Compiler identity is still the RUNNING BINARY's content hash. That is a known keying defect, not
// a property of this module: the binary embeds the checkout's commit, so a corpus-only commit
// re-keys every module. Its replacement is compiler identity derived from what the compiler IS
// (source closure + toolchain, the v2.compiler.self_host.generation axes) through the sealed
// per-axis observations; until that lands this function is the one authority every key consumes.

use std::fs;
use std::rc::Rc;
use std::sync::OnceLock;

use crate::v1_compiler_compile::SourceFile;
use crate::v1_rt::{self, Hash};

fn extract_module_path(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("module ") {
            return Some(rest.trim().to_string());
        }
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            break;
        }
    }
    None
}

pub fn closure_content_digest(sources: &[Rc<SourceFile>]) -> Hash {
    let mut pairs: Vec<(String, &str)> = sources
        .iter()
        .map(|s| {
            let module = extract_module_path(&s.content).unwrap_or_else(|| s.path.clone());
            (module, s.content.as_str())
        })
        .collect();
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    let mut acc = v1_rt::atom_identity_hash("resolved-graph-closure-v1".to_string());
    for (module, content) in pairs {
        acc = v1_rt::hash_combine(acc, v1_rt::atom_identity_hash(module));
        acc = v1_rt::hash_combine(acc, v1_rt::atom_identity_hash(content.to_string()));
    }
    acc
}

/// Content hash of the running compiler binary — the compiler-identity key term.
/// One authority for every key that must invalidate across a seed rebuild: the
/// resolved-graph subject digest (below) and the typed-module content key
/// (`std.interface_summary.typed_module_key`) both consume this digest rather than
/// re-deriving the executable read.
pub fn transform_content_digest() -> Hash {
    static DIGEST: OnceLock<Hash> = OnceLock::new();
    DIGEST
        .get_or_init(|| {
            // THE RUNNING IMAGE, NOT THE PATH IT WAS STARTED FROM. On Linux `current_exe()` is
            // the start path, which reads `<path> (deleted)` once a cargo build re-links it.
            // Measured 2026-08-30 (BuildBuddy, regen round cost): the in-round seed build
            // re-linked a byte-identical binary and this read panicked on the stale path.
            // `/proc/self/exe` resolves to the mapped image even after the directory entry is
            // replaced; elsewhere the start path is the only spelling available.
            let exe = if cfg!(target_os = "linux") {
                std::path::PathBuf::from("/proc/self/exe")
            } else {
                std::env::current_exe().unwrap_or_else(|e| {
                    panic!(
                        "closure identity: cannot locate compiler executable to content-address \
                         the transform: {e}"
                    )
                })
            };
            let bytes = fs::read(&exe).unwrap_or_else(|e| {
                panic!(
                    "closure identity: cannot read compiler executable {:?} to content-address \
                     the transform: {}",
                    exe, e
                )
            });
            v1_rt::bytes_identity_hash(&bytes)
        })
        .clone()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyInputAxis {
    ClosureSubject,
    TransformContent,
}

impl KeyInputAxis {
    pub const ALL: &'static [KeyInputAxis] =
        &[KeyInputAxis::ClosureSubject, KeyInputAxis::TransformContent];
}

#[derive(Debug, Clone)]
pub struct KeyInputMaterials {
    closure_subject: Hash,
    transform_content: Hash,
}

impl KeyInputMaterials {
    pub fn new(closure_subject: Hash, transform_content: Hash) -> Self {
        Self {
            closure_subject,
            transform_content,
        }
    }

    fn materialize(&self, axis: KeyInputAxis) -> Hash {
        match axis {
            KeyInputAxis::ClosureSubject => self.closure_subject.clone(),
            KeyInputAxis::TransformContent => self.transform_content.clone(),
        }
    }
}

pub fn derive_subject_digest(materials: &KeyInputMaterials) -> Hash {
    KeyInputAxis::ALL.iter().fold(
        v1_rt::atom_identity_hash("resolved-graph-subject-v1".to_string()),
        |acc, axis| v1_rt::hash_combine(acc, materials.materialize(*axis)),
    )
}

pub fn subject_digest_for_closure(sources: &[Rc<SourceFile>]) -> Hash {
    let materials =
        KeyInputMaterials::new(closure_content_digest(sources), transform_content_digest());
    derive_subject_digest(&materials)
}

pub fn witness_work_subject_key(closure_subject_digest: &str, function: &str) -> Hash {
    v1_rt::hash_combine(
        v1_rt::atom_identity_hash(closure_subject_digest.to_string()),
        v1_rt::atom_identity_hash(function.to_string()),
    )
}
