//! Generic seed-shim assembly for curated self-host behavioral receipts.
//! Authority: tools.self_host_curated_seed_linked_harness (5-arm design).
//! dissolve-on: v2 std self-emits + gunbc emits seed-linked extern imports.

use sha2::Digest;
use std::fs;
use std::path::{Path, PathBuf};

const BOOTSTRAP_INLINE_MODS: &[&str] = &["NonEmptyVec", "NonEmptyBTreeSet"];

/// Emitted peripheral helpers that are not seed-routed but are known gunbc closure artifacts.
const PERIPHERAL_EMIT_RETAIN_MODS: &[&str] = &["dry_run"];

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
                write!(
                    f,
                    "entry module mutated during assembly ({before} != {after})"
                )
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

/// SCAFFOLD — see `cssl_emit_artifact_sanitize_scaffold_debt` in
/// `dag/tools/self_host_curated_seed_linked_harness.dag`. String-scrub on emitted
/// `v2_std_integer.rs` / `v2_std_witness.rs` only; dissolve-on #6775 emitter defects.
fn sanitize_emitter_artifact_in_place(path: &Path) -> Result<(), AssemblyError> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    if name != "v2_std_integer.rs" && name != "v2_std_witness.rs" {
        return Ok(());
    }
    let content = fs::read_to_string(path)?;
    let sanitized = if name == "v2_std_integer.rs" {
        sanitize_integer_inhabitant_dupes(&content)
    } else {
        sanitize_witness_import_conflict(&content)
    };
    if sanitized != content {
        eprintln!(
            "CSSL_ASSEMBLE: sanitize scaffold applied to {}",
            path.display()
        );
        fs::write(path, sanitized)?;
    }
    Ok(())
}

fn sanitize_integer_inhabitant_dupes(content: &str) -> String {
    let mut out = Vec::new();
    let mut skip = false;
    for line in content.lines() {
        if line.trim() == "pub struct Int128;" {
            skip = true;
            continue;
        }
        if skip {
            if line.trim() == "pub struct UInt8;" {
                skip = false;
            }
            continue;
        }
        out.push(line);
    }
    out.join("\n")
}

fn sanitize_witness_import_conflict(content: &str) -> String {
    if !content.lines().any(|l| l.trim() == "pub enum Witness") {
        return content.to_string();
    }
    content
        .lines()
        .filter(|l| {
            let t = l.trim();
            t != "use crate::v1_rt::Witness;" && !t.starts_with("use crate::v1_rt::Witness::")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Assembly arms: entry untouched · compiler seed re-export · shared std-bridge · dag/std
/// emit-retain · lens/peripheral emit-retain · bootstrap_inline · typed Refused.
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
        return Err(AssemblyError::MissingEntryFile { path: entry_file });
    }
    if !emitted_lib_rs.is_file() {
        return Err(AssemblyError::MissingEmittedLibRs {
            path: emitted_lib_rs,
        });
    }
    if !seed_lib_rs.is_file() {
        return Err(AssemblyError::MissingSeedLibRs { path: seed_lib_rs });
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
                sanitize_emitter_artifact_in_place(&dest)?;
            }
            continue;
        }

        if module.starts_with("v2_std_") {
            let bridge_src = std_bridge_dir.join(format!("{module}.rs"));
            if bridge_src.is_file() {
                copy_std_bridge(&dest, &bridge_src)?;
            } else {
                sanitize_emitter_artifact_in_place(&dest)?;
            }
            continue;
        }

        if module.starts_with("std_") {
            sanitize_emitter_artifact_in_place(&dest)?;
            continue;
        }

        if module == "v1_rt" || module.starts_with("v2_extdeps_") {
            sanitize_emitter_artifact_in_place(&dest)?;
            continue;
        }

        if module.starts_with("v2_lens_") || PERIPHERAL_EMIT_RETAIN_MODS.contains(&module.as_str())
        {
            sanitize_emitter_artifact_in_place(&dest)?;
            continue;
        }

        return Err(AssemblyError::RefusedDep {
            module,
            reason: "unroutable closure dependency (not entry/compiler/std-bridge/bootstrap/lens/peripheral)"
                .to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn write_minimal_emit_tree(
        root: &Path,
        entry_mod: &str,
        closure_mods: &[&str],
    ) -> Result<PathBuf, AssemblyError> {
        let out = root.join("out");
        let src = out.join("src");
        fs::create_dir_all(&src)?;
        let lib = closure_mods
            .iter()
            .map(|m| format!("pub mod {m};"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(
            out.join("src/lib.rs"),
            format!("{lib}\npub mod {entry_mod};\n"),
        )?;
        fs::write(src.join(format!("{entry_mod}.rs")), "// entry\n")?;
        for m in closure_mods {
            if *m == entry_mod {
                continue;
            }
            fs::write(src.join(format!("{m}.rs")), format!("// emitted {m}\n"))?;
        }
        let seed_src = root.join("seed/src");
        fs::create_dir_all(&seed_src)?;
        fs::write(
            seed_src.join("lib.rs"),
            "pub mod v2_compiler_tokenize;\npub mod std_algebra;\n",
        )?;
        fs::write(seed_src.join("v2_compiler_tokenize.rs"), "pub fn f() {}\n")?;
        fs::write(seed_src.join("std_algebra.rs"), "pub fn g() {}\n")?;
        Ok(out)
    }

    fn temp_fixture_root() -> PathBuf {
        let base = std::env::temp_dir().join(format!("cssl_assembly_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).expect("temp root");
        base
    }

    #[test]
    fn std_prefix_deps_are_emit_retained_not_refused() {
        let root = temp_fixture_root();
        let out = write_minimal_emit_tree(
            &root,
            "v2_compiler_tokenize",
            &[
                "std_error_primitives",
                "std_algebra",
                "v2_compiler_tokenize",
            ],
        )
        .expect("tree");
        let seed_lib = root.join("seed/src/lib.rs");
        let repo = root.join("repo");
        fs::create_dir_all(repo.join("src/v1/stage0/src")).expect("seed path");
        fs::copy(&seed_lib, repo.join("src/v1/stage0/src/lib.rs")).expect("copy seed");
        let dag = repo.join("src/v2/compiler/01_tokenize.dag");
        fs::create_dir_all(dag.parent().unwrap()).expect("dag dir");
        fs::write(&dag, "module v2.compiler.tokenize\n").expect("dag");
        let bridge = repo.join("dag/tools/self_host_std_bridge_shims");
        fs::create_dir_all(&bridge).expect("bridge");
        assemble_seed_linked_closure(&out, &dag, &repo, &bridge).expect("assemble");
        let kept = fs::read_to_string(out.join("src/std_error_primitives.rs")).expect("read");
        assert!(kept.contains("emitted std_error_primitives"));
    }

    #[test]
    fn extdeps_without_seed_stays_emitted() {
        let root = temp_fixture_root();
        let out = write_minimal_emit_tree(
            &root,
            "v2_compiler_self_host",
            &["extdeps_communication_medium", "v2_compiler_self_host"],
        )
        .expect("tree");
        let seed_src = root.join("seed/src");
        fs::write(seed_src.join("lib.rs"), "pub mod v2_compiler_tokenize;\n").expect("seed");
        let repo = root.join("repo");
        fs::create_dir_all(repo.join("src/v1/stage0/src")).expect("seed path");
        fs::copy(
            seed_src.join("lib.rs"),
            repo.join("src/v1/stage0/src/lib.rs"),
        )
        .expect("copy");
        let dag = repo.join("src/v2/compiler/self_host.dag");
        fs::create_dir_all(dag.parent().unwrap()).expect("dag dir");
        fs::write(&dag, "module v2.compiler.self_host\n").expect("dag");
        let bridge = repo.join("dag/tools/self_host_std_bridge_shims");
        fs::create_dir_all(&bridge).expect("bridge");
        assemble_seed_linked_closure(&out, &dag, &repo, &bridge).expect("assemble");
        let kept =
            fs::read_to_string(out.join("src/extdeps_communication_medium.rs")).expect("read");
        assert!(kept.contains("emitted extdeps_communication_medium"));
    }

    #[test]
    fn dry_run_peripheral_emit_retain_holds() {
        let root = temp_fixture_root();
        let out = write_minimal_emit_tree(
            &root,
            "v2_compiler_self_host",
            &["dry_run", "v2_compiler_self_host"],
        )
        .expect("tree");
        let seed_src = root.join("seed/src");
        fs::write(seed_src.join("lib.rs"), "pub mod v2_compiler_tokenize;\n").expect("seed");
        let repo = root.join("repo");
        fs::create_dir_all(repo.join("src/v1/stage0/src")).expect("seed path");
        fs::copy(
            seed_src.join("lib.rs"),
            repo.join("src/v1/stage0/src/lib.rs"),
        )
        .expect("copy");
        let dag = repo.join("src/v2/compiler/self_host.dag");
        fs::create_dir_all(dag.parent().unwrap()).expect("dag dir");
        fs::write(&dag, "module v2.compiler.self_host\n").expect("dag");
        let bridge = repo.join("dag/tools/self_host_std_bridge_shims");
        fs::create_dir_all(&bridge).expect("bridge");
        assemble_seed_linked_closure(&out, &dag, &repo, &bridge).expect("assemble");
        let kept = fs::read_to_string(out.join("src/dry_run.rs")).expect("read");
        assert!(kept.contains("emitted dry_run"));
    }

    #[test]
    fn unknown_closure_dep_is_typed_refused() {
        let root = temp_fixture_root();
        let out = write_minimal_emit_tree(
            &root,
            "v2_compiler_tokenize",
            &["not_a_routable_mod", "v2_compiler_tokenize"],
        )
        .expect("tree");
        let seed_src = root.join("seed/src");
        fs::write(seed_src.join("lib.rs"), "pub mod v2_compiler_tokenize;\n").expect("seed");
        let repo = root.join("repo");
        fs::create_dir_all(repo.join("src/v1/stage0/src")).expect("seed path");
        fs::copy(
            seed_src.join("lib.rs"),
            repo.join("src/v1/stage0/src/lib.rs"),
        )
        .expect("copy");
        let dag = repo.join("src/v2/compiler/01_tokenize.dag");
        fs::create_dir_all(dag.parent().unwrap()).expect("dag dir");
        fs::write(&dag, "module v2.compiler.tokenize\n").expect("dag");
        let bridge = repo.join("dag/tools/self_host_std_bridge_shims");
        fs::create_dir_all(&bridge).expect("bridge");
        let err = assemble_seed_linked_closure(&out, &dag, &repo, &bridge).unwrap_err();
        match err {
            AssemblyError::RefusedDep { module, reason } => {
                assert_eq!(module, "not_a_routable_mod");
                assert!(reason.contains("unroutable"));
            }
            other => panic!("expected RefusedDep, got {other:?}"),
        }
    }
}
