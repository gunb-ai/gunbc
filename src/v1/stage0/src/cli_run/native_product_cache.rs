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
//! the producer is rustc's own per-crate dep-info for the seed (`target/<profile>/deps/<crate>-<hash>.d`),
//! not cargo's combined beside-binary `.d` (that one carries build-script directory watches and
//! `.git` identity files). File contents are hashed, keyed by workspace-relative path, including
//! OUT_DIR-generated files even when they sit outside the workspace; registry crates are Cargo.lock
//! plus the existing toolchain axis. A directory or unreadable entry is a counted MISS
//! (`producer_dep_info`), never a lane refusal and never a whole-tree fallback. The closure and the
//! census-only declaration universe are hashed at content grain (a span-insensitive heads hash
//! would narrow the latter and is a declared next step, not assumed here).
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
    /// rustc's per-crate dep-info was missing, listed a directory, or named an unreadable file.
    /// Named cause `producer_dep_info`. Never a lane refusal.
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

/// Hash compiled-input contents, keyed by identity (workspace-relative, or the rustc-spelled
/// path for an OUT_DIR file outside the workspace). A directory or unreadable path is a counted
/// MISS, never a skipped member and never a whole-tree fallback.
fn hash_compiled_inputs(entries: &[(String, PathBuf)]) -> Result<String, DeriveKeyError> {
    if entries.is_empty() {
        return Err(miss(
            "NativeProductProducerEmpty — rustc dep-info named no compiled file",
        ));
    }
    let mut sorted = entries.to_vec();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    let mut h = sha2::Sha256::new();
    let mut prev: Option<String> = None;
    for (key, path) in &sorted {
        if prev.as_deref() == Some(key.as_str()) {
            continue;
        }
        prev = Some(key.clone());
        if path.is_dir() {
            return Err(miss(format!(
                "rustc dep-info names directory {key}, which has no file contents to hash"
            )));
        }
        let bytes = std::fs::read(path).map_err(|e| miss(format!("{key}: {e}")))?;
        h.update((key.len() as u64).to_le_bytes());
        h.update(key.as_bytes());
        h.update((bytes.len() as u64).to_le_bytes());
        h.update(&bytes);
    }
    Ok(format!("{:x}", h.finalize()))
}

/// `.../build/<pkg>-<hash>/out/...` — cargo's generated-sources directory, which may sit
/// outside the workspace when `CARGO_TARGET_DIR` does.
fn is_cargo_out_dir_file(path: &Path) -> bool {
    let mut parts = path.iter().map(|s| s.to_string_lossy());
    while let Some(part) = parts.next() {
        if part == "build" {
            let _pkg = parts.next();
            return parts.next().as_deref() == Some("out");
        }
    }
    false
}

fn producer_entry_key(workspace: &Path, prerequisite: &str) -> (String, PathBuf) {
    let raw = PathBuf::from(prerequisite);
    let path = if raw.is_absolute() {
        raw
    } else {
        workspace.join(prerequisite)
    };
    if let Ok(rel) = path.strip_prefix(workspace) {
        return (rel.to_string_lossy().replace('\\', "/"), path);
    }
    (prerequisite.replace('\\', "/"), path)
}

fn keep_compiled_prerequisite(workspace: &Path, prerequisite: &str) -> bool {
    let (_, path) = producer_entry_key(workspace, prerequisite);
    if let Ok(rel) = path.strip_prefix(workspace) {
        let rel = rel.to_string_lossy().replace('\\', "/");
        if rel.starts_with(".git/") || rel == ".git" {
            return false;
        }
        return true;
    }
    is_cargo_out_dir_file(&path)
}

/// rustc's per-crate dep-info in `deps/`: `<crate>-<metadata>.d` beside the rlib/rmeta/bin object.
fn select_rustc_dep_info(deps_dir: &Path, rustc_crate: &str) -> Result<PathBuf, DeriveKeyError> {
    let prefix = format!("{rustc_crate}-");
    let mut candidates = Vec::new();
    let entries = std::fs::read_dir(deps_dir).map_err(|e| {
        miss(format!(
            "rustc deps directory {} unreadable: {e}",
            deps_dir.display()
        ))
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| miss(format!("listing {}: {e}", deps_dir.display())))?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        let Some(rest) = name
            .strip_prefix(&prefix)
            .and_then(|s| s.strip_suffix(".d"))
        else {
            continue;
        };
        if rest.is_empty() || !rest.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
            continue;
        }
        let rlib = deps_dir.join(format!("lib{rustc_crate}-{rest}.rlib"));
        let rmeta = deps_dir.join(format!("lib{rustc_crate}-{rest}.rmeta"));
        let obj = deps_dir.join(format!("{rustc_crate}-{rest}"));
        if rlib.exists() || rmeta.exists() || obj.exists() {
            let modified = entry.metadata().and_then(|m| m.modified()).ok();
            candidates.push((modified, entry.path()));
        }
    }
    if candidates.is_empty() {
        return Err(miss(format!(
            "no rustc dep-info for crate {rustc_crate} in {}",
            deps_dir.display()
        )));
    }
    candidates.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(candidates.pop().unwrap().1)
}

fn cargo_package_name(manifest_text: &str) -> Option<String> {
    let mut in_package = false;
    for line in manifest_text.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            in_package = t == "[package]";
            continue;
        }
        if in_package {
            if let Some(rest) = t.strip_prefix("name") {
                let rest = rest.trim().strip_prefix('=')?.trim();
                let name = rest.trim_matches('"').trim_matches('\'').to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
    }
    None
}

fn rustc_crate_name(package: &str) -> String {
    package.replace('-', "_")
}

const LINKED_PARTITION_CRATES: &str = "src/v1/stage0/linked_partition_crates.generated.txt";

fn linked_partition_rustc_crates(workspace: &Path) -> Result<Vec<String>, DeriveKeyError> {
    let listing = workspace.join(LINKED_PARTITION_CRATES);
    let text = std::fs::read_to_string(&listing)
        .map_err(|e| miss(format!("{}: {e}", listing.display())))?;
    let mut crates = Vec::new();
    for line in text.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let manifest = workspace.join(line).join("Cargo.toml");
        let body = std::fs::read_to_string(&manifest)
            .map_err(|e| miss(format!("{}: {e}", manifest.display())))?;
        let package = cargo_package_name(&body).ok_or_else(|| {
            miss(format!(
                "{} has no [package] name, so its rustc crate is unknown",
                manifest.display()
            ))
        })?;
        crates.push(rustc_crate_name(&package));
    }
    if crates.is_empty() {
        return Err(miss(
            "linked-partition-crates projection named no crate; the seed's infer crate would be missing from the producer key",
        ));
    }
    Ok(crates)
}

fn entries_from_rustc_dep_info(
    workspace: &Path,
    text: &str,
) -> Result<Vec<(String, PathBuf)>, DeriveKeyError> {
    let mut out = Vec::new();
    for prerequisite in super::checker_dependency::parse_dep_info_prerequisites(text) {
        if !keep_compiled_prerequisite(workspace, &prerequisite) {
            continue;
        }
        let (key, path) = producer_entry_key(workspace, &prerequisite);
        if path.is_dir() {
            return Err(miss(format!(
                "rustc dep-info names directory {key}, which has no file contents to hash"
            )));
        }
        out.push((key, path));
    }
    Ok(out)
}

/// rustc crate units whose sources the running seed binary was compiled from: this package's
/// lib, the running bin, and every linked partition crate. rustc's per-crate `.d` omits
/// dependency crates, so the infer crate is not in `gunbc-<hash>.d` alone.
fn seed_rustc_units(workspace: &Path) -> Result<Vec<String>, DeriveKeyError> {
    let mut units = vec![rustc_crate_name(env!("CARGO_PKG_NAME"))];
    let exe = std::env::current_exe().map_err(|e| miss(format!("current_exe: {e}")))?;
    if let Some(stem) = exe.file_stem().and_then(|s| s.to_str()) {
        let bin = rustc_crate_name(stem);
        if !units.iter().any(|u| u == &bin) {
            units.push(bin);
        }
    }
    for crate_name in linked_partition_rustc_crates(workspace)? {
        if !units.iter().any(|u| u == &crate_name) {
            units.push(crate_name);
        }
    }
    Ok(units)
}

fn observe_rustc_compiled_inputs(
    workspace: &Path,
) -> Result<Vec<(String, PathBuf)>, DeriveKeyError> {
    let exe = std::env::current_exe().map_err(|e| miss(format!("current_exe: {e}")))?;
    let Some(profile_dir) = exe.parent() else {
        return Err(miss("running seed has no parent directory"));
    };
    let deps_dir = profile_dir.join("deps");
    if !deps_dir.is_dir() {
        return Err(miss(format!(
            "rustc deps directory {} is absent; producer key needs the seed cargo built in this workspace",
            deps_dir.display()
        )));
    }
    let mut entries = Vec::new();
    for unit in seed_rustc_units(workspace)? {
        let record = select_rustc_dep_info(&deps_dir, &unit)?;
        let text = std::fs::read_to_string(&record)
            .map_err(|e| miss(format!("{}: {e}", record.display())))?;
        entries.extend(entries_from_rustc_dep_info(workspace, &text)?);
    }
    let lock = workspace.join("Cargo.lock");
    if !lock.is_file() {
        return Err(miss(
            "Cargo.lock is unreadable; registry crates have no producer coverage",
        ));
    }
    entries.push(("Cargo.lock".into(), lock));
    Ok(entries)
}

/// Producer axis: rustc's compiled-file set for the seed crate graph. Not cargo's combined
/// beside-binary `.d`. An unobserved set is `ProducerInputsUnobserved`, never a walk of `src/v1`.
fn producer_axis(workspace: &Path) -> Result<String, DeriveKeyError> {
    hash_compiled_inputs(&observe_rustc_compiled_inputs(workspace)?)
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
        producer_compiler: producer_axis(workspace)?,
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

    fn compiled(workspace: &Path, rel: &str) -> (String, PathBuf) {
        (rel.to_string(), workspace.join(rel))
    }

    // Control: a commit that touches only .git or unrelated docs leaves the producer digest.
    // A compiled .rs change moves it.
    #[test]
    fn a_seed_file_outside_the_compiled_set_leaves_the_producer_key_unchanged() {
        let root = scratch("producer-set");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::create_dir_all(root.join("docs")).unwrap();
        std::fs::create_dir_all(root.join(".git")).unwrap();
        std::fs::write(root.join("src/compiled.rs"), b"seed").unwrap();
        std::fs::write(root.join("docs/note.md"), b"docs").unwrap();
        std::fs::write(root.join(".git/HEAD"), b"ref: refs/heads/main\n").unwrap();
        let set = vec![compiled(&root, "src/compiled.rs")];
        let h0 = hash_compiled_inputs(&set).unwrap();
        std::fs::write(root.join("docs/note.md"), b"docs-changed").unwrap();
        std::fs::write(root.join(".git/HEAD"), b"ref: refs/heads/other\n").unwrap();
        assert_eq!(h0, hash_compiled_inputs(&set).unwrap());
        std::fs::write(root.join("src/compiled.rs"), b"seed-changed").unwrap();
        assert_ne!(h0, hash_compiled_inputs(&set).unwrap());
    }

    #[test]
    fn a_changed_out_dir_file_changes_the_producer_key() {
        let root = scratch("producer-out");
        let out_root = scratch("producer-target");
        let gen = out_root
            .join("release")
            .join("build")
            .join("v1-compiler-deadbeef")
            .join("out")
            .join("generated.rs");
        std::fs::create_dir_all(gen.parent().unwrap()).unwrap();
        std::fs::write(&gen, b"gen-1").unwrap();
        std::fs::write(root.join("src.rs"), b"src").unwrap();
        let set = vec![
            compiled(&root, "src.rs"),
            (gen.to_string_lossy().into_owned(), gen.clone()),
        ];
        let h0 = hash_compiled_inputs(&set).unwrap();
        std::fs::write(&gen, b"gen-2").unwrap();
        assert_ne!(h0, hash_compiled_inputs(&set).unwrap());
        assert!(keep_compiled_prerequisite(&root, &gen.to_string_lossy()));
    }

    #[test]
    fn a_directory_or_unreadable_compiled_entry_is_a_counted_miss() {
        let root = scratch("producer-dir");
        std::fs::create_dir_all(root.join("src")).unwrap();
        let dir_err = hash_compiled_inputs(&[compiled(&root, "src")])
            .expect_err("a directory must not mint a producer digest");
        assert!(
            matches!(dir_err, DeriveKeyError::ProducerInputsUnobserved { .. }),
            "{dir_err:?}"
        );
        let missing = hash_compiled_inputs(&[("src/gone.rs".into(), root.join("src/gone.rs"))])
            .expect_err("an unreadable file must not mint a producer digest");
        assert!(
            matches!(missing, DeriveKeyError::ProducerInputsUnobserved { .. }),
            "{missing:?}"
        );
    }

    #[test]
    fn rustc_dep_info_without_cargo_toml_is_the_compiled_set() {
        let root = scratch("rustc-d");
        std::fs::create_dir_all(root.join("src")).unwrap();
        std::fs::write(root.join("src/lib.rs"), b"lib").unwrap();
        std::fs::write(root.join("src/v1_compiler_infer.rs"), b"infer").unwrap();
        let bin_d = format!(
            "{}: {}\n",
            root.join("deps/gunbc-abcd.d").display(),
            root.join("src/lib.rs").display()
        );
        let infer_d = format!(
            "{}: {}\n",
            root.join("deps/v1_stage0_v1_infer-abcd.d").display(),
            root.join("src/v1_compiler_infer.rs").display()
        );
        let bin = entries_from_rustc_dep_info(&root, &bin_d).unwrap();
        let infer = entries_from_rustc_dep_info(&root, &infer_d).unwrap();
        assert!(bin.iter().any(|(k, _)| k == "src/lib.rs"));
        assert!(infer.iter().any(|(k, _)| k == "src/v1_compiler_infer.rs"));
        assert!(!keep_compiled_prerequisite(
            &root,
            &root.join(".git/HEAD").to_string_lossy()
        ));
    }

    #[test]
    fn an_empty_compiled_set_is_a_miss_not_a_key() {
        let err = hash_compiled_inputs(&[])
            .expect_err("an empty compiled set must not mint a producer digest");
        assert!(
            matches!(err, DeriveKeyError::ProducerInputsUnobserved { .. }),
            "{err:?}"
        );
    }

    // Inhabitance: the unit-test harness is not target/<profile>/gunbc, so rustc's seed
    // crate-graph dep-info is not beside it. producer_axis must miss, not walk src/v1.
    #[test]
    fn linked_partition_crates_include_the_infer_crate() {
        let crates =
            linked_partition_rustc_crates(&super::super::process_workspace_root()).unwrap();
        assert!(
            crates.iter().any(|c| c == "v1_stage0_v1_infer"),
            "partition rustc units {crates:?} must include the infer crate; gunbc-<hash>.d alone omits it"
        );
    }

    #[test]
    fn this_test_binary_does_not_mint_a_producer_key() {
        let err = producer_axis(&super::super::process_workspace_root())
            .expect_err("a test binary has no rustc seed-graph dep-info beside it");
        assert!(
            matches!(err, DeriveKeyError::ProducerInputsUnobserved { .. }),
            "{err:?}"
        );
    }

    #[test]
    fn an_unkeyed_miss_is_counted_with_the_named_cause_and_no_digest() {
        let root = scratch("unkeyed");
        let store = root.join(STORE_DIR);
        std::fs::create_dir_all(&store).unwrap();
        record_unkeyed_outcome(&store, "e", "miss", "producer_dep_info");
        let text = std::fs::read_to_string(root.join(RECEIPT)).unwrap();
        assert!(text.contains("\"key\":null"));
        assert!(text.contains("\"cause\":\"producer_dep_info\""));
        assert!(text.contains("\"outcome\":\"miss\""));
    }
}
