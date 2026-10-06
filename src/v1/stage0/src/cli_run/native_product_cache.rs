//! NATIVE PRODUCT REUSE (CI programme row R3, `docs/plans/ci-end-to-end-accounting.md`).
//!
//! `prepare_emitted_compiler_for_entry` pays reconcile + emit + cargo build + the discriminating
//! red to turn one closure into one executable, and it pays them again on every run even when no
//! input changed. This module keys the PRODUCT -- the emitted executable plus the observation of
//! the build that made it -- by the effective inputs of that production, so a run whose inputs are
//! identical reuses it and spends its time on the instrument's controls instead.
//!
//! THE KEY IS THE SIX CAUSAL AXES of `v2.compiler.self_host.generation` `PreMaterializationIdentity`
//! in their declared order (producer compiler, source closure, target model, toolchain, build
//! configuration, required lens contract), folded as `pre_materialization_digest` folds them. The
//! fold hash is SHA-256 (`std.content_hash` `HashFamily` cryptographic arm), not the 64-bit
//! structural family, because a collision here serves a wrong compiler rather than missing.
//! The keyed relation is `std.materialization_provider` `ArtifactRequest::NativeCompilerArtifactRequest`;
//! this is its first consumer (the seed-side realization of `request_key`, v1 maintenance standing:
//! it serves the v2 self-host program and is dissolved when the native ancestry lane makes the
//! provider the only producer -- `native_product_cache_seed_growth`).
//!
//! EVERY AXIS ERRS TOWARD A MISS. A miss costs one build; a stale hit serves a compiler that does
//! not correspond to the tree. So the axes are over-approximations of relevance, never under:
//! the producer is the seed's own source tree, the closure and the census-only declaration universe
//! are hashed at content grain (a span-insensitive heads hash would narrow the latter and is a
//! declared next step, not assumed here).
//!
//! WRITES ARE ATOMIC AND READS ARE VERIFIED. An entry is committed only after the discriminating
//! red held on the artifact, by writing a private sibling directory and renaming it into place; a
//! read re-hashes the executable against the manifest and the manifest against the key, and any
//! disagreement REFUSES the entry (typed, counted, removed) and falls back to a rebuild -- it never
//! serves the entry and never widens. With `GUNBC_NATIVE_CACHE_ROOT` unset there is no store and no
//! host-global fallback: the preparation builds exactly as before.

use sha2::Digest;
use std::path::{Path, PathBuf};

const STORE_DIR: &str = "native-products";
const MANIFEST: &str = "manifest.json";
const EXECUTABLE: &str = "executable";

/// The axes in declared order, each already a SHA-256 hex digest of its own preimage.
#[derive(Debug, Clone)]
pub(super) struct ProductKey {
    pub digest: String,
    pub axes: Vec<(&'static str, String)>,
}

fn hex(bytes: &[u8]) -> String {
    format!("{:x}", sha2::Sha256::digest(bytes))
}

fn walk_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir).map_err(|e| {
        format!(
            "NativeProductKeyUnreadable — listing {}: {e}",
            dir.display()
        )
    })?;
    for entry in entries {
        let path = entry
            .map_err(|e| format!("NativeProductKeyUnreadable — {}: {e}", dir.display()))?
            .path();
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned());
        if path.is_dir() {
            if name.as_deref() == Some("target") {
                continue;
            }
            walk_files(&path, out)?;
        } else {
            out.push(path);
        }
    }
    Ok(())
}

/// Producer axis: the seed's own source. Its binary is not stable across runner slots (build paths
/// differ), but what it is built FROM is, and the seed's behaviour is a function of that source.
fn producer_axis(workspace: &Path) -> Result<String, String> {
    let mut files = Vec::new();
    walk_files(&workspace.join("src").join("v1"), &mut files)?;
    for name in ["Cargo.toml", "Cargo.lock"] {
        let path = workspace.join(name);
        if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    let mut h = sha2::Sha256::new();
    for path in &files {
        let rel = path.strip_prefix(workspace).unwrap_or(path);
        let bytes = std::fs::read(path)
            .map_err(|e| format!("NativeProductKeyUnreadable — {}: {e}", path.display()))?;
        h.update((rel.to_string_lossy().len() as u64).to_le_bytes());
        h.update(rel.to_string_lossy().as_bytes());
        h.update((bytes.len() as u64).to_le_bytes());
        h.update(&bytes);
    }
    Ok(format!("{:x}", h.finalize()))
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

/// Derive the key for `entry` over `source_roots`. Loads the SAME shared index and closure the
/// emission will use (process-shared, so the emission reuses them rather than re-reading).
pub(super) fn derive_key(
    source_roots: &[String],
    entry: &str,
    workspace: &Path,
) -> Result<ProductKey, String> {
    let index = super::try_process_shared_index_for_pool(source_roots, true)
        .map_err(|c| format!("NativeProductKeyUnreadable — source discovery: {c}"))?;
    let closure = super::load_sources_for_entry_with_pool(&index, entry)
        .map_err(|c| format!("NativeProductKeyUnreadable — closure load: {c}"))?;
    let universe =
        super::compile_clean::compile_clean_census_only_sources_for_compiled(&index, &closure);
    let source_closure = hex(format!(
        "closure={};declaration_universe={}",
        sources_axis(&closure),
        sources_axis(&universe)
    )
    .as_bytes());
    let toolchain = super::emitted_closure_compile_host::probe_toolchain_identity_for_key()?;
    let build_configuration = hex(format!(
        "cargo build --release --offline --message-format json --color never;RUSTFLAGS={}",
        super::emitted_closure_compile_host::WARNING_DENIAL_RUSTFLAGS
    )
    .as_bytes());
    let axes: Vec<(&'static str, String)> = vec![
        ("producer_compiler", producer_axis(workspace)?),
        ("source_closure", source_closure),
        (
            "target_model",
            hex(format!("rust;{}-{}", std::env::consts::ARCH, std::env::consts::OS).as_bytes()),
        ),
        ("toolchain", hex(toolchain.as_bytes())),
        ("build_configuration", build_configuration),
        (
            "required_lens_contract",
            hex(format!("entry={entry}").as_bytes()),
        ),
    ];
    let mut digest = hex(b"gunbc.native_product.pre_materialization_identity");
    for (_, axis) in &axes {
        digest = hex(format!("{digest}\u{0}{axis}").as_bytes());
    }
    Ok(ProductKey { digest, axes })
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
        manifest: Manifest,
    },
    Miss,
    /// The entry existed and failed verification; it has been removed and the caller rebuilds.
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
            manifest,
        },
        Err(cause) => {
            let _ = std::fs::remove_dir_all(&dir);
            Lookup::Refused { cause }
        }
    }
}

/// Commit atomically: write a private sibling, rename into place. WriteOnce -- an entry already
/// present (a peer won the race) is left as it is, since both are the product of one key.
pub(super) fn commit(
    root: &Path,
    key: &ProductKey,
    executable: &Path,
    manifest: &Manifest,
) -> Result<(), String> {
    std::fs::create_dir_all(root).map_err(|e| format!("creating {}: {e}", root.display()))?;
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
        match std::fs::rename(&staging, entry_dir(root, key)) {
            Ok(()) => Ok(()),
            Err(_) if entry_dir(root, key).is_dir() => Ok(()),
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

    // A corrupted cached executable is refused, removed, and the next lookup is a plain miss.
    #[test]
    fn corrupted_executable_is_refused_and_removed() {
        let root = scratch("corrupt");
        let store = root.join("store");
        let bin = root.join("bin");
        std::fs::write(&bin, b"emitted-compiler").unwrap();
        let k = key("bbbb");
        commit(&store, &k, &bin, &manifest_for(&k, b"emitted-compiler")).unwrap();
        std::fs::write(store.join("bbbb").join(EXECUTABLE), b"tampered").unwrap();
        assert!(matches!(lookup(&store, &k), Lookup::Refused { .. }));
        assert!(matches!(lookup(&store, &k), Lookup::Miss));
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

    // Each axis, changed alone, changes the folded digest (no axis is dropped from the fold).
    #[test]
    fn every_axis_reaches_the_digest() {
        let fold = |axes: &[&str]| {
            let mut d = hex(b"gunbc.native_product.pre_materialization_identity");
            for a in axes {
                d = hex(format!("{d}\u{0}{a}").as_bytes());
            }
            d
        };
        let base = ["p", "s", "t", "c", "b", "l"];
        for i in 0..base.len() {
            let mut changed = base;
            changed[i] = "X";
            assert_ne!(fold(&base), fold(&changed), "axis {i} is not in the fold");
        }
    }
}
