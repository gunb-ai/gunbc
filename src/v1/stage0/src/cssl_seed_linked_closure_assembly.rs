//! Generic seed-shim assembly for curated self-host behavioral receipts.
//! Authority: tools.self_host_curated_seed_linked_harness (5-arm design).
//! dissolve-on: v2 std self-emits + gunbc emits seed-linked extern imports.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

const BOOTSTRAP_INLINE_MODS: &[&str] = &["NonEmptyVec", "NonEmptyBTreeSet"];

#[derive(Debug)]
pub enum AssemblyError {
    MissingEntryFile { path: PathBuf },
    EntryMutated { before: String, after: String },
    MissingEmittedLibRs { path: PathBuf },
    MissingSeedLibRs { path: PathBuf },
    RefusedDep { module: String, reason: String },
    Io(std::io::Error),
}

impl std::fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEntryFile { path } => write!(f, "missing entry module file {path:?}"),
            Self::EntryMutated { before, after } => {
                write!(f, "entry module mutated during assembly ({before} != {after})")
            }
            Self::MissingEmittedLibRs { path } => write!(f, "missing emitted lib.rs {path:?}"),
            Self::MissingSeedLibRs { path } => write!(f, "missing seed lib.rs {path:?}"),
            Self::RefusedDep { module, reason } => {
                write!(f, "refused dep assembly for {module}: {reason}")
            }
            Self::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl std::error::Error for AssemblyError {}

impl From<std::io::Error> for AssemblyError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub fn dag_entry_rust_module(dag_path: &Path) -> Result<String, AssemblyError> {
    let content = fs::read_to_string(dag_path)?;
    let line = content
        .lines()
        .find(|l| l.starts_with("module "))
        .ok_or_else(|| AssemblyError::RefusedDep {
            module: dag_path.display().to_string(),
            reason: "no module line".to_string(),
        })?;
    let qualified = line.trim_start_matches("module ").trim();
    Ok(qualified.replace('.', "_"))
}

fn sha256_hex(path: &Path) -> Result<String, AssemblyError> {
    use std::io::Read;
    let mut file = fs::File::open(path)?;
    let mut hasher = sha2::Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        use sha2::Digest;
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn seed_has_pub_mod(seed_lib_rs: &Path, module: &str) -> Result<bool, AssemblyError> {
    let content = fs::read_to_string(seed_lib_rs)?;
    let needle = format!("pub mod {module};");
    Ok(content.lines().any(|l| l.trim() == needle))
}

fn parse_closure_mods(lib_rs: &Path) -> Result<Vec<String>, AssemblyError> {
    let content = fs::read_to_string(lib_rs)?;
    Ok(content
        .lines()
        .filter_map(|l| {
            let t = l.trim();
            t.strip_prefix("pub mod ")
                .and_then(|rest| rest.strip_suffix(';'))
                .map(|m| m.trim().to_string())
        })
        .collect())
}

fn write_compiler_seed_reexport(dest: &Path, seed_mod: &str) -> Result<(), AssemblyError> {
    let body = format!(
        "// seed-linked dep shim — auto-derived compiler re-export\n\
         #![allow(clippy::all, dead_code, unused_imports)]\n\
         pub use v1_compiler::{seed_mod}::*;\n"
    );
    fs::write(dest, body)?;
    Ok(())
}

fn copy_std_bridge(dest: &Path, bridge_src: &Path) -> Result<(), AssemblyError> {
    fs::copy(bridge_src, dest)?;
    Ok(())
}

/// Five-arm assembly: entry untouched · compiler seed re-export · shared std-bridge · bootstrap_inline · Refused.
pub fn assemble_seed_linked_closure(
    out_dir: &Path,
    entry_dag: &Path,
    repo_root: &Path,
    std_bridge_dir: &Path,
) -> Result<(), AssemblyError> {
    let seed_lib_rs = repo_root.join("src/v1/stage0/src/lib.rs");
    let src_dir = out_dir.join("src");
    let emitted_lib_rs = src_dir.join("lib.rs");
    let entry_mod = dag_entry_rust_module(entry_dag)?;
    let entry_file = src_dir.join(format!("{entry_mod}.rs"));

    if !entry_file.is_file() {
        return Err(AssemblyError::MissingEntryFile {
            path: entry_file,
        });
    }
    if !emitted_lib_rs.is_file() {
        return Err(AssemblyError::MissingEmittedLibRs {
            path: emitted_lib_rs,
        });
    }
    if !seed_lib_rs.is_file() {
        return Err(AssemblyError::MissingSeedLibRs {
            path: seed_lib_rs,
        });
    }

    let entry_hash_before = sha256_hex(&entry_file)?;
    let closure_mods = parse_closure_mods(&emitted_lib_rs)?;

    for module in closure_mods {
        if module == entry_mod {
            continue;
        }
        if BOOTSTRAP_INLINE_MODS.contains(&module.as_str()) {
            continue;
        }

        let dest = src_dir.join(format!("{module}.rs"));
        if !dest.is_file() {
            return Err(AssemblyError::RefusedDep {
                module: module.clone(),
                reason: "closure mod missing emitted .rs file".to_string(),
            });
        }

        if module.starts_with("v2_compiler_")
            || module.starts_with("extdeps_")
            || module.starts_with("v1_compiler_")
        {
            if seed_has_pub_mod(&seed_lib_rs, &module)? {
                write_compiler_seed_reexport(&dest, &module)?;
            } else {
                return Err(AssemblyError::RefusedDep {
                    module,
                    reason: "compiler dep not present in v1-compiler seed lib.rs".to_string(),
                });
            }
            continue;
        }

        if module.starts_with("v2_std_") {
            let bridge_src = std_bridge_dir.join(format!("{module}.rs"));
            if bridge_src.is_file() {
                copy_std_bridge(&dest, &bridge_src)?;
            } else {
                return Err(AssemblyError::RefusedDep {
                    module,
                    reason: "no shared std-bridge shim for referenced v2_std module".to_string(),
                });
            }
            continue;
        }

        if module == "v1_rt" {
            continue;
        }

        return Err(AssemblyError::RefusedDep {
            module,
            reason: "unroutable closure dependency (not entry/compiler/std-bridge/bootstrap)".to_string(),
        });
    }

    let entry_hash_after = sha256_hex(&entry_file)?;
    if entry_hash_before != entry_hash_after {
        return Err(AssemblyError::EntryMutated {
            before: entry_hash_before,
            after: entry_hash_after,
        });
    }

    Ok(())
}
