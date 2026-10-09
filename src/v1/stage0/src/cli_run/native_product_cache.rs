//! NATIVE PRODUCT REUSE (CI programme row R3, `docs/plans/ci-end-to-end-accounting.md`).
//!
//! `prepare_emitted_compiler_for_entry` pays reconcile + emit + cargo build + the discriminating
//! red to turn one closure into one executable, and it pays them again on every run even when no
//! input changed. This module keys the PRODUCT -- the emitted executable plus the observation of
//! the build that made it -- by the effective inputs of that production, so a run whose inputs are
//! identical reuses it and spends its time on the instrument's controls instead.
//!
//! THE KEY IS A SEED-LOCAL PRODUCT KEY OVER THE SAME SIX CAUSAL AXES as `v2.compiler.self_host.generation`
//! `PreMaterializationIdentity` (producer compiler, source closure, target model, toolchain, build
//! configuration, required lens contract), in that declared order. It is NOT `pre_materialization_digest`
//! and NOT `std.materialization_provider` `request_key`: those are structural Fnv folds (the digest
//! combines the last two axes as a pair, and `request_key` wraps it in an evaluation bucket with demand
//! kind and declared inputs, a 16-hex `ContentHash`). This one left-folds six SHA-256 digests under a
//! fixed tag into a 64-hex key. STATED REASON FOR THE DIVERGENCE (DESIGN 3b, diverges with a reason):
//! a collision here serves a wrong compiler rather than missing, so the store address must be
//! cryptographic, and the provider does not yet serve the native artifact so there is no `request_key`
//! to inhabit. Axis identity and order are the shared fact; the fold and the wrapper are this seed's.
//! The relation keyed is `ArtifactRequest::NativeCompilerArtifactRequest`; this is its first consumer, and
//! the key is dissolved into `request_key`'s cryptographic realization when the native ancestry lane
//! makes `provider_serve` the only producer (`native_product_cache_seed_growth`), which is what
//! discharges the divergence.
//!
//! EVERY AXIS ERRS TOWARD A MISS. A miss costs one build; a stale hit serves a compiler that does
//! not correspond to the tree. So the axes are over-approximations of relevance, never under:
//! the producer is the running seed executable itself (`/proc/self/exe` bytes) with the
//! build-script provenance string masked. `src/v1/stage0/build.rs` has exactly one
//! `cargo:rustc-env`: `GUNBC_BUILD_IDENTITY` (git HEAD, or `GUNBC_MATERIALIZED_TREE_IDENTITY`
//! when that input is set — not a second embed). That string is baked into the image, so a
//! docs-only commit would otherwise re-key every native product (the same defect
//! `closure_identity::transform_content_digest` documents and still carries). The axis replaces
//! every occurrence of `env!("GUNBC_BUILD_IDENTITY")` with a same-length placeholder, then
//! hashes. Empty identity, identity shorter than a git SHA (40 bytes), or zero occurrences is a
//! counted MISS (`producer_identity_unmasked`). Seed builds remap checkout, isolated cargo home
//! and rustup home so host paths do not remain. An unreadable executable is a counted MISS
//! (`producer_image_unreadable`), never a lane refusal and never a whole-tree fallback. The
//! closure and the census-only declaration universe are hashed at content grain (a span-insensitive
//! heads hash would narrow the latter and is a declared next step, not assumed here).
//!
//! WRITES ARE ATOMIC AND READS ARE VERIFIED. An entry is committed only after the discriminating
//! red held on the artifact, by writing a private sibling directory and renaming it into place; a
//! read re-hashes the executable against the manifest and the manifest against the key, and any
//! disagreement REFUSES the entry (typed, counted, left in place) and falls back to a rebuild -- it never
//! serves the entry and never widens. With `GUNBC_NATIVE_CACHE_ROOT` unset there is no store and no
//! host-global fallback: the preparation builds exactly as before.

use sha2::Digest;
use std::path::{Path, PathBuf};

const STORE_DIR: &str = "native-products";
const MANIFEST: &str = "manifest.json";
const EXECUTABLE: &str = "executable";
const RECEIPT: &str = "receipt.jsonl";
/// Marker written inside an entry this run committed from its own verified build. `save` packs only
/// marked entries, so a restored or refused entry is never republished. Named in
/// `gunbc.native_product_shared_transfer` (`native_product_committed_marker_name`).
pub(super) const COMMITTED_MARKER: &str = "committed-this-run";
/// Counted MISS cause when the running seed image cannot be read for `producer_compiler`.
pub(super) const PRODUCER_UNOBSERVED_CAUSE: &str = "producer_image_unreadable";
/// Counted MISS cause when `GUNBC_BUILD_IDENTITY` cannot be masked out of the seed image.
pub(super) const PRODUCER_IDENTITY_UNMASKED_CAUSE: &str = "producer_identity_unmasked";
/// Git SHA length; shorter provenance would collide inside ordinary binary bytes.
const MIN_EMBEDDED_PROVENANCE_LEN: usize = 40;
const PROVENANCE_PLACEHOLDER_BYTE: u8 = b'*';

/// The axes in declared order, each already a SHA-256 hex digest of its own preimage.
#[derive(Debug, Clone)]
pub(super) struct ProductKey {
    pub digest: String,
    pub axes: Vec<(&'static str, String)>,
}

fn hex(bytes: &[u8]) -> String {
    format!("{:x}", sha2::Sha256::digest(bytes))
}

/// Why a product key was not derived. A missing producer record is a counted MISS (build cold);
/// any other unreadable input is a lane refusal — it is not a miss, because we cannot say what
/// the key would have been.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum DeriveKeyError {
    /// Producer image unreadable or provenance unmaskable. Named causes
    /// `producer_image_unreadable` / `producer_identity_unmasked`. Never a lane refusal.
    ProducerInputsUnobserved {
        cause: String,
    },
    Unreadable {
        cause: String,
    },
}

fn miss(cause: impl Into<String>) -> DeriveKeyError {
    DeriveKeyError::ProducerInputsUnobserved {
        cause: cause.into(),
    }
}

/// SHA-256 of the process image that is deriving the key. Prefer `/proc/self/exe` so a
/// deleted-or-replaced path still hashes the bytes that are running.
fn running_seed_image() -> Result<PathBuf, DeriveKeyError> {
    let proc = PathBuf::from("/proc/self/exe");
    if proc.exists() {
        return Ok(proc);
    }
    std::env::current_exe().map_err(|e| miss(format!("current_exe: {e}")))
}

fn mask_embedded_provenance(image: &[u8], identity: &str) -> Result<Vec<u8>, DeriveKeyError> {
    if identity.is_empty() || identity.len() < MIN_EMBEDDED_PROVENANCE_LEN {
        return Err(miss(PRODUCER_IDENTITY_UNMASKED_CAUSE));
    }
    let needle = identity.as_bytes();
    let mut out = image.to_vec();
    let mut hits = 0usize;
    let mut i = 0usize;
    while i + needle.len() <= out.len() {
        if &out[i..i + needle.len()] == needle {
            for byte in &mut out[i..i + needle.len()] {
                *byte = PROVENANCE_PLACEHOLDER_BYTE;
            }
            hits += 1;
            i += needle.len();
        } else {
            i += 1;
        }
    }
    if hits == 0 {
        return Err(miss(PRODUCER_IDENTITY_UNMASKED_CAUSE));
    }
    Ok(out)
}

/// Producer axis: the running seed with build identity masked. Unreadable image or
/// unmaskable provenance is `ProducerInputsUnobserved`.
fn producer_axis() -> Result<String, DeriveKeyError> {
    let path = running_seed_image()?;
    let bytes = std::fs::read(&path).map_err(|_| miss(PRODUCER_UNOBSERVED_CAUSE))?;
    if bytes.is_empty() {
        return Err(miss(PRODUCER_UNOBSERVED_CAUSE));
    }
    let masked = mask_embedded_provenance(&bytes, env!("GUNBC_BUILD_IDENTITY"))?;
    Ok(format!("{:x}", sha2::Sha256::digest(&masked)))
}

fn sources_axis(sources: &[std::rc::Rc<crate::v1_compiler_compile::SourceFile>]) -> String {
    let mut sorted: Vec<_> = sources.iter().collect();
    sorted.sort_by(|a, b| a.path.cmp(&b.path));
    let mut h = sha2::Sha256::new();
    for s in sorted {
        h.update((s.path.len() as u64).to_le_bytes());
        h.update(s.path.as_bytes());
        h.update((s.content.len() as u64).to_le_bytes());
        h.update(s.content.as_bytes());
    }
    format!("{:x}", h.finalize())
}

/// The measured inputs of one key, before folding.
pub(super) struct AxisInputs {
    pub producer_compiler: String,
    pub source_closure: String,
    pub toolchain: String,
    pub build_configuration: String,
    pub entry: String,
}

/// THE ONE FOLD, in `PreMaterializationIdentity` declared order. `derive_key` measures and hands
/// the inputs here, so a control over this function is a control over the key the run uses.
pub(super) fn assemble_key(i: AxisInputs) -> ProductKey {
    let axes: Vec<(&'static str, String)> = vec![
        ("producer_compiler", hex(i.producer_compiler.as_bytes())),
        ("source_closure", hex(i.source_closure.as_bytes())),
        (
            "target_model",
            hex(format!("rust;{}-{}", std::env::consts::ARCH, std::env::consts::OS).as_bytes()),
        ),
        ("toolchain", hex(i.toolchain.as_bytes())),
        ("build_configuration", hex(i.build_configuration.as_bytes())),
        (
            "required_lens_contract",
            hex(format!("entry={}", i.entry).as_bytes()),
        ),
    ];
    let mut digest = hex(b"gunbc.native_product.pre_materialization_identity");
    for (_, axis) in &axes {
        digest = hex(format!("{digest}\u{0}{axis}").as_bytes());
    }
    ProductKey { digest, axes }
}

/// Derive the key for `entry` over `source_roots`. Loads the SAME shared index and closure the
/// emission will use (process-shared, so the emission reuses them rather than re-reading).
pub(super) fn derive_key(
    source_roots: &[String],
    entry: &str,
    workspace: &Path,
) -> Result<ProductKey, DeriveKeyError> {
    let unreadable = |c: String| DeriveKeyError::Unreadable { cause: c };
    let index = super::try_process_shared_index_for_pool(source_roots, true).map_err(|c| {
        unreadable(format!(
            "NativeProductKeyUnreadable — source discovery: {c}"
        ))
    })?;
    let closure = super::load_sources_for_entry_with_pool(&index, entry)
        .map_err(|c| unreadable(format!("NativeProductKeyUnreadable — closure load: {c}")))?;
    let universe =
        super::compile_clean::compile_clean_census_only_sources_for_compiled(&index, &closure);
    let source_closure = hex(format!(
        "closure={};declaration_universe={}",
        sources_axis(&closure),
        sources_axis(&universe)
    )
    .as_bytes());
    let toolchain = super::emitted_closure_compile_host::probe_toolchain_identity_for_key()
        .map_err(unreadable)?;
    let build_configuration =
        hex(super::emitted_closure_compile_host::probe_build_configuration_for_key().as_bytes());
    Ok(assemble_key(AxisInputs {
        producer_compiler: producer_axis()?,
        source_closure,
        toolchain,
        build_configuration,
        entry: entry.to_string(),
    }))
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct Manifest {
    pub key: String,
    pub binary_sha256: String,
    pub closure_identity: String,
    pub cargo_argv: Vec<String>,
    pub rustflags: String,
    pub compiler_path: String,
    pub rustc_identity: String,
    pub exit_status: i64,
    pub warning_count: i64,
    pub warning_headers: Vec<String>,
    /// Committed only after the discriminating red held on this artifact; recorded so a reader of
    /// the entry can see which evidence admitted it.
    pub discriminating_red_held: bool,
}

pub(super) enum Lookup {
    Hit {
        executable: PathBuf,
        manifest: Box<Manifest>,
    },
    Miss,
    /// The entry existed and failed verification; it is left in place (a read never mutates the store) and the caller rebuilds.
    Refused {
        cause: String,
    },
}

/// The store root, only when the host declared one. No fallback.
pub(super) fn store_root() -> Option<PathBuf> {
    std::env::var("GUNBC_NATIVE_CACHE_ROOT")
        .ok()
        .filter(|r| !r.trim().is_empty())
        .map(|r| PathBuf::from(r).join(STORE_DIR))
}

fn entry_dir(root: &Path, key: &ProductKey) -> PathBuf {
    root.join(&key.digest)
}

fn sha256_path(path: &Path) -> Result<String, String> {
    std::fs::read(path)
        .map(|b| hex(&b))
        .map_err(|e| format!("reading {}: {e}", path.display()))
}

pub(super) fn lookup(root: &Path, key: &ProductKey) -> Lookup {
    let dir = entry_dir(root, key);
    if !dir.is_dir() {
        return Lookup::Miss;
    }
    let verdict = (|| -> Result<(PathBuf, Manifest), String> {
        let text = std::fs::read_to_string(dir.join(MANIFEST))
            .map_err(|e| format!("manifest unreadable: {e}"))?;
        let manifest: Manifest =
            serde_json::from_str(&text).map_err(|e| format!("manifest malformed: {e}"))?;
        if manifest.key != key.digest {
            return Err(format!(
                "manifest names key {} but the entry is stored under {}",
                manifest.key, key.digest
            ));
        }
        if !manifest.discriminating_red_held {
            return Err("entry was committed without the discriminating red".to_string());
        }
        let executable = dir.join(EXECUTABLE);
        let actual = sha256_path(&executable)?;
        if actual != manifest.binary_sha256 {
            return Err(format!(
                "executable hashes to {actual}, manifest records {}",
                manifest.binary_sha256
            ));
        }
        Ok((executable, manifest))
    })();
    match verdict {
        Ok((executable, manifest)) => Lookup::Hit {
            executable,
            manifest: Box::new(manifest),
        },
        // A read never mutates the store: a refused entry is reported and rebuilt around, and
        // only a protected writer (`writer_standing`) may remove or replace shared entries.
        Err(cause) => Lookup::Refused { cause },
    }
}

const SHARED_RESTORE_ENTRY: &str = "dag/gunbc/native_product_shared_transfer.dag";
const SHARED_RESTORE_FUNCTION: &str = "native_product_restore_wet";

/// What a restore attempt established. Every non-`Restored` arm is a counted MISS with the cause that
/// made it one; the caller records it once (`record_outcome`) and builds cold.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RestoreOutcome {
    Restored,
    Unavailable { cause: &'static str, detail: String },
}

impl RestoreOutcome {
    fn unavailable(cause: &'static str, detail: impl Into<String>) -> Self {
        RestoreOutcome::Unavailable {
            cause,
            detail: detail.into(),
        }
    }
}

/// SHARED-STORE RESTORE on an L1 miss. The key is derived here and nowhere else, so the restore is
/// asked for exactly this key (`GUNBC_NATIVE_PRODUCT_KEY`); the `.dag` entry only moves bytes into
/// the job-private store, and the caller's `lookup` then re-verifies them like any local entry. Every
/// failure is returned as a cause-bearing outcome and the caller builds cold; nothing here fails a lane.
/// The store directory is created BEFORE the spawn: the entry installs with `mv -T <scratch>/<key>
/// <store>/<key>`, which is ENOENT when `<store>` does not exist, and a fresh job-private root holds
/// only the directory the workflow made.
pub(super) fn restore_from_shared_store(
    source_roots: &[String],
    store: &Path,
    key: &ProductKey,
    workspace: &Path,
) -> RestoreOutcome {
    // Only the cost gate lives here: no credential means no point loading the closure for the entry,
    // whose `native_product_run_standing` and `select_access_token_source` own the event and
    // credential decisions (a fork, an unattributed run and a missing token each refuse there).
    if std::env::var("WIF_ACCESS_TOKEN")
        .map(|t| t.trim().is_empty())
        .unwrap_or(true)
    {
        return RestoreOutcome::unavailable(
            "auth",
            "no workload identity in this run (fork pull request, failed auth step or federation not provisioned)",
        );
    }
    if let Err(e) = std::fs::create_dir_all(store) {
        return RestoreOutcome::unavailable(
            "store_dir",
            format!("creating {}: {e}", store.display()),
        );
    }
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(e) => return RestoreOutcome::unavailable("current_exe", e.to_string()),
    };
    let mut cmd = std::process::Command::new(exe);
    cmd.current_dir(workspace).arg("run");
    for root in source_roots {
        cmd.arg("--source-root").arg(root);
    }
    cmd.arg("--entry")
        .arg(SHARED_RESTORE_ENTRY)
        .arg("--function")
        .arg(SHARED_RESTORE_FUNCTION)
        .env("GUNBC_NATIVE_PRODUCT_KEY", &key.digest)
        .stdout(std::process::Stdio::null());
    match cmd.output() {
        Ok(out) if out.status.success() => RestoreOutcome::Restored,
        Ok(out) => {
            let err = String::from_utf8_lossy(&out.stderr);
            let reason = err
                .lines()
                .rev()
                .find(|l| !l.starts_with("[pre-entry]") && !l.trim().is_empty())
                .unwrap_or("no reason reported");
            RestoreOutcome::unavailable("transfer", reason)
        }
        Err(e) => RestoreOutcome::unavailable("spawn", e.to_string()),
    }
}

/// The structured counter: ONE JSON line per entry preparation, appended to `receipt.jsonl` in the
/// job-private root's parent-of-store (the root itself), carrying the final outcome and, for a miss,
/// the cause. It is the record a later reader counts; the eprintln beside it is only the live log.
pub(super) fn record_outcome(
    store: &Path,
    entry: &str,
    key: &ProductKey,
    outcome: &str,
    cause: Option<&str>,
) {
    let line = serde_json::json!({
        "entry": entry,
        "key": key.digest,
        "outcome": outcome,
        "cause": cause,
    })
    .to_string();
    eprintln!("v2-native-route: native product RECEIPT {line}");
    write_receipt_line(store, &line);
}

/// A MISS with no key: the producer inputs were unobserved, so minting a digest would be a
/// fabricated address. The receipt still counts the miss under a named cause.
pub(super) fn record_unkeyed_outcome(store: &Path, entry: &str, outcome: &str, cause: &str) {
    let line = serde_json::json!({
        "entry": entry,
        "key": serde_json::Value::Null,
        "outcome": outcome,
        "cause": cause,
    })
    .to_string();
    eprintln!("v2-native-route: native product RECEIPT {line}");
    write_receipt_line(store, &line);
}

fn write_receipt_line(store: &Path, line: &str) {
    let Some(root) = store.parent() else { return };
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(root.join(RECEIPT))
    {
        let _ = writeln!(f, "{line}");
    }
}

/// Publication standing: only the merge queue's composed-revision run may publish. Pull-request
/// code can set any variable inside its own job, so this names the protected event and does not
/// claim to stop hostile PR code on a shared filesystem; that boundary is the credential
/// asymmetry of the shared R2 store (`gunbc.native_product_shared_store`).
pub(super) fn writer_standing() -> bool {
    event_may_publish(std::env::var("GITHUB_EVENT_NAME").ok().as_deref())
}

fn event_may_publish(event: Option<&str>) -> bool {
    event == Some("merge_group")
}

/// Publication is admitted only for a protected writer whose red+restore COMPLETED.
pub(super) fn publication_admitted(red_completed: bool, event: Option<&str>) -> bool {
    red_completed && event_may_publish(event)
}

/// Commit atomically: write a private sibling, rename into place. An existing entry is left as it
/// is ONLY when it verifies under the key (a peer won the race: both are the product of one key);
/// an existing entry that fails verification is a known-refused destination, so it is removed and
/// replaced by this run's verified build, never treated as committed. The committed marker is written
/// after the rename, so `save` publishes only what this run built and verified.
pub(super) fn commit(
    root: &Path,
    key: &ProductKey,
    executable: &Path,
    manifest: &Manifest,
) -> Result<(), String> {
    std::fs::create_dir_all(root).map_err(|e| format!("creating {}: {e}", root.display()))?;
    let dest = entry_dir(root, key);
    if dest.is_dir() {
        match lookup(root, key) {
            Lookup::Hit { .. } => return Ok(()),
            _ => std::fs::remove_dir_all(&dest)
                .map_err(|e| format!("removing refused entry {}: {e}", dest.display()))?,
        }
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let staging = root.join(format!(".staging-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&staging).map_err(|e| format!("creating staging: {e}"))?;
    let result = (|| -> Result<(), String> {
        std::fs::copy(executable, staging.join(EXECUTABLE))
            .map_err(|e| format!("copying executable: {e}"))?;
        let text = serde_json::to_string_pretty(manifest)
            .map_err(|e| format!("serializing manifest: {e}"))?;
        std::fs::write(staging.join(MANIFEST), text)
            .map_err(|e| format!("writing manifest: {e}"))?;
        std::fs::write(staging.join(COMMITTED_MARKER), b"")
            .map_err(|e| format!("writing marker: {e}"))?;
        match std::fs::rename(&staging, &dest) {
            Ok(()) => Ok(()),
            // A peer committed between the check and the rename: its entry must verify to stand.
            Err(_) if matches!(lookup(root, key), Lookup::Hit { .. }) => Ok(()),
            Err(e) => Err(format!("renaming into place: {e}")),
        }
    })();
    let _ = std::fs::remove_dir_all(&staging);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(digest: &str) -> ProductKey {
        ProductKey {
            digest: digest.to_string(),
            axes: vec![],
        }
    }

    fn manifest_for(key: &ProductKey, bytes: &[u8]) -> Manifest {
        Manifest {
            key: key.digest.clone(),
            binary_sha256: hex(bytes),
            closure_identity: "c".into(),
            cargo_argv: vec![],
            rustflags: "-D warnings".into(),
            compiler_path: "rustc".into(),
            rustc_identity: "rustc 1".into(),
            exit_status: 0,
            warning_count: 0,
            warning_headers: vec![],
            discriminating_red_held: true,
        }
    }

    fn scratch(name: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("gunbc-npc-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    // A committed product is a hit under the same key and a miss under any other key: a changed
    // axis changes the digest, so the entry is not found.
    #[test]
    fn committed_product_hits_under_its_key_and_misses_under_a_changed_one() {
        let root = scratch("hit");
        let bin = root.join("bin");
        std::fs::write(&bin, b"emitted-compiler").unwrap();
        let k = key("aaaa");
        commit(
            &root.join("store"),
            &k,
            &bin,
            &manifest_for(&k, b"emitted-compiler"),
        )
        .unwrap();
        assert!(matches!(
            lookup(&root.join("store"), &k),
            Lookup::Hit { .. }
        ));
        assert!(matches!(
            lookup(&root.join("store"), &key("aaab")),
            Lookup::Miss
        ));
    }

    // A corrupted cached executable is refused and left in place, so the next lookup refuses it again.
    #[test]
    fn corrupted_executable_is_refused_and_left_for_the_writer() {
        let root = scratch("corrupt");
        let store = root.join("store");
        let bin = root.join("bin");
        std::fs::write(&bin, b"emitted-compiler").unwrap();
        let k = key("bbbb");
        commit(&store, &k, &bin, &manifest_for(&k, b"emitted-compiler")).unwrap();
        std::fs::write(store.join("bbbb").join(EXECUTABLE), b"tampered").unwrap();
        assert!(matches!(lookup(&store, &k), Lookup::Refused { .. }));
        // A read never deletes: the entry is still there and still refused.
        assert!(matches!(lookup(&store, &k), Lookup::Refused { .. }));
    }

    // Deleting the publish rule makes this red: a skipped red or a pull-request run never publishes.
    #[test]
    fn only_a_completed_red_on_the_merge_queue_publishes() {
        assert!(publication_admitted(true, Some("merge_group")));
        assert!(!publication_admitted(false, Some("merge_group")));
        assert!(!publication_admitted(true, Some("pull_request")));
        assert!(!publication_admitted(true, Some("workflow_dispatch")));
        assert!(!publication_admitted(true, None));
    }

    // An entry whose manifest does not record the red is refused rather than served.
    #[test]
    fn entry_without_the_red_is_refused() {
        let root = scratch("nored");
        let store = root.join("store");
        let bin = root.join("bin");
        std::fs::write(&bin, b"x").unwrap();
        let k = key("cccc");
        let mut m = manifest_for(&k, b"x");
        m.discriminating_red_held = false;
        commit(&store, &k, &bin, &m).unwrap();
        assert!(matches!(lookup(&store, &k), Lookup::Refused { .. }));
    }

    // Each measured input, changed alone, changes the key `derive_key` returns: it folds through
    // `assemble_key`, so dropping an axis there turns this red.
    #[test]
    fn every_axis_reaches_the_key() {
        let base = || AxisInputs {
            producer_compiler: "p".into(),
            source_closure: "s".into(),
            toolchain: "t".into(),
            build_configuration: "b".into(),
            entry: "e".into(),
        };
        let k0 = assemble_key(base()).digest;
        let variants: Vec<AxisInputs> = vec![
            AxisInputs {
                producer_compiler: "X".into(),
                ..base()
            },
            AxisInputs {
                source_closure: "X".into(),
                ..base()
            },
            AxisInputs {
                toolchain: "X".into(),
                ..base()
            },
            AxisInputs {
                build_configuration: "X".into(),
                ..base()
            },
            AxisInputs {
                entry: "X".into(),
                ..base()
            },
        ];
        for v in variants {
            assert_ne!(k0, assemble_key(v).digest);
        }
        assert_eq!(assemble_key(base()).axes.len(), 6);
    }

    // Defect: a restored entry that fails verification stayed in place and the cold commit treated the
    // existing directory as success, dropping the fresh build. Commit must replace a refused entry.
    #[test]
    fn a_refused_entry_is_replaced_by_the_verified_build_and_marked() {
        let root = scratch("replace");
        let store = root.join("store");
        let bin = root.join("bin");
        std::fs::write(&bin, b"fresh-build").unwrap();
        let k = key("dddd");
        std::fs::create_dir_all(store.join("dddd")).unwrap();
        std::fs::write(store.join("dddd").join(EXECUTABLE), b"planted").unwrap();
        std::fs::write(store.join("dddd").join(MANIFEST), b"{}").unwrap();
        assert!(matches!(lookup(&store, &k), Lookup::Refused { .. }));
        commit(&store, &k, &bin, &manifest_for(&k, b"fresh-build")).unwrap();
        assert!(matches!(lookup(&store, &k), Lookup::Hit { .. }));
        assert!(store.join("dddd").join(COMMITTED_MARKER).exists());
    }

    // A verified existing entry (a peer won, or a verified restore) is left as it is and is NOT
    // marked, so save does not republish what this run did not build.
    #[test]
    fn a_verified_existing_entry_is_kept_and_not_marked() {
        let root = scratch("keep");
        let store = root.join("store");
        let bin = root.join("bin");
        std::fs::write(&bin, b"peer").unwrap();
        let k = key("eeee");
        let mut m = manifest_for(&k, b"peer");
        m.discriminating_red_held = true;
        std::fs::create_dir_all(store.join("eeee")).unwrap();
        std::fs::write(store.join("eeee").join(EXECUTABLE), b"peer").unwrap();
        std::fs::write(
            store.join("eeee").join(MANIFEST),
            serde_json::to_string(&m).unwrap(),
        )
        .unwrap();
        commit(&store, &k, &bin, &m).unwrap();
        assert!(matches!(lookup(&store, &k), Lookup::Hit { .. }));
        assert!(!store.join("eeee").join(COMMITTED_MARKER).exists());
    }

    // Defect: the workflow makes only the job-private root; the restore installs into
    // `<root>/native-products/<key>`, ENOENT when that directory is missing. The restore prepares the
    // store directory first, and a no-credential attempt returns a cause-bearing outcome.
    #[test]
    fn restore_into_a_fresh_root_has_a_store_directory_to_install_into() {
        let root = scratch("fresh");
        let store = root.join(STORE_DIR);
        assert!(!store.exists());
        std::env::set_var("WIF_ACCESS_TOKEN", "x");
        let k = key("ffff");
        // No spawnable closure here: the attempt fails at transfer, after the directory exists.
        let out = restore_from_shared_store(&[], &store, &k, &root);
        std::env::remove_var("WIF_ACCESS_TOKEN");
        assert!(store.is_dir(), "outcome {out:?}");
        assert!(matches!(out, RestoreOutcome::Unavailable { .. }));
        // The install the .dag entry performs, on that store: mv -T of an extracted key directory.
        let extracted = root.join("scratch").join("ffff");
        std::fs::create_dir_all(&extracted).unwrap();
        let status = std::process::Command::new("mv")
            .arg("-T")
            .arg(&extracted)
            .arg(store.join("ffff"))
            .status()
            .unwrap();
        assert!(status.success());
        // Control: the same mv into a root whose store directory was never made is the ENOENT.
        let bare = scratch("fresh-bare");
        std::fs::create_dir_all(root.join("scratch2")).unwrap();
        let status = std::process::Command::new("mv")
            .arg("-T")
            .arg(root.join("scratch2"))
            .arg(bare.join(STORE_DIR).join("ffff"))
            .status()
            .unwrap();
        assert!(!status.success());
    }

    #[test]
    fn a_missing_credential_is_a_cause_bearing_miss_and_the_receipt_counts_it_once() {
        let root = scratch("receipt");
        let store = root.join(STORE_DIR);
        std::env::remove_var("WIF_ACCESS_TOKEN");
        let k = key("1111");
        let out = restore_from_shared_store(&[], &store, &k, &root);
        let RestoreOutcome::Unavailable { cause, .. } = out else {
            panic!("expected a miss")
        };
        assert_eq!(cause, "auth");
        record_outcome(&store, "e", &k, "miss", Some(cause));
        let text = std::fs::read_to_string(root.join(RECEIPT)).unwrap();
        assert_eq!(text.lines().count(), 1);
        assert!(text.contains("\"cause\":\"auth\"") && text.contains("\"outcome\":\"miss\""));
    }

    #[test]
    fn mask_replaces_every_identity_occurrence_with_a_same_length_placeholder() {
        let id = "a".repeat(40);
        let image = [b"pre", id.as_bytes(), b"mid", id.as_bytes(), b"post"].concat();
        let masked = mask_embedded_provenance(&image, &id).unwrap();
        assert_eq!(masked.len(), image.len());
        assert!(!masked.windows(id.len()).any(|w| w == id.as_bytes()));
        assert_eq!(
            masked
                .windows(40)
                .filter(|w| w.iter().all(|b| *b == b'*'))
                .count(),
            2
        );
        assert_ne!(
            format!("{:x}", sha2::Sha256::digest(&masked)),
            format!("{:x}", sha2::Sha256::digest(&image))
        );
    }

    #[test]
    fn empty_short_or_absent_identity_is_a_counted_miss() {
        let image = b"no-identity-here";
        assert_eq!(
            mask_embedded_provenance(image, "").unwrap_err(),
            miss(PRODUCER_IDENTITY_UNMASKED_CAUSE)
        );
        assert_eq!(
            mask_embedded_provenance(image, &"a".repeat(39)).unwrap_err(),
            miss(PRODUCER_IDENTITY_UNMASKED_CAUSE)
        );
        assert_eq!(
            mask_embedded_provenance(image, &"b".repeat(40)).unwrap_err(),
            miss(PRODUCER_IDENTITY_UNMASKED_CAUSE)
        );
    }

    #[test]
    fn producer_axis_is_the_sha256_of_the_masked_running_executable() {
        let path = running_seed_image().unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let identity = env!("GUNBC_BUILD_IDENTITY");
        let masked = mask_embedded_provenance(&bytes, identity).unwrap();
        assert_ne!(
            format!("{:x}", sha2::Sha256::digest(&bytes)),
            format!("{:x}", sha2::Sha256::digest(&masked))
        );
        assert_eq!(
            producer_axis().unwrap(),
            format!("{:x}", sha2::Sha256::digest(&masked))
        );
    }

    #[test]
    fn a_changed_seed_image_moves_the_producer_digest() {
        let a = format!("{:x}", sha2::Sha256::digest(b"seed-image-a"));
        let b = format!("{:x}", sha2::Sha256::digest(b"seed-image-b"));
        assert_ne!(a, b);
        let k0 = assemble_key(AxisInputs {
            producer_compiler: a.clone(),
            source_closure: "c".into(),
            toolchain: "t".into(),
            build_configuration: "b".into(),
            entry: "e".into(),
        });
        let k1 = assemble_key(AxisInputs {
            producer_compiler: b,
            source_closure: "c".into(),
            toolchain: "t".into(),
            build_configuration: "b".into(),
            entry: "e".into(),
        });
        assert_ne!(k0.digest, k1.digest);
        assert_eq!(
            k0.digest,
            assemble_key(AxisInputs {
                producer_compiler: a,
                source_closure: "c".into(),
                toolchain: "t".into(),
                build_configuration: "b".into(),
                entry: "e".into(),
            })
            .digest
        );
    }

    #[test]
    fn an_unkeyed_miss_is_counted_with_the_named_cause_and_no_digest() {
        let root = scratch("unkeyed");
        let store = root.join(STORE_DIR);
        std::fs::create_dir_all(&store).unwrap();
        record_unkeyed_outcome(&store, "e", "miss", PRODUCER_UNOBSERVED_CAUSE);
        let text = std::fs::read_to_string(root.join(RECEIPT)).unwrap();
        assert!(text.contains("\"key\":null"));
        assert!(text.contains("\"cause\":\"producer_image_unreadable\""));
        assert!(text.contains("\"outcome\":\"miss\""));
    }
}
