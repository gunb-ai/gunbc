//! Generic seed-shim assembly for curated self-host behavioral receipts.
//! Authority: tools.self_host_curated_seed_linked_harness (5-arm design).
//! dissolve-on: v2 std self-emits + gunbc emits seed-linked extern imports.

use sha2::Digest;
use std::fs;
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

/// Membership in the gunbc-emitted closure: module listed in emitted `src/lib.rs`
/// and backed by an emitted `src/{module}.rs`. The assembly loop only visits
/// such modules; `seed lib.rs` `pub mod` presence is an external-oracle fact and
/// must not trigger seed-replace for closure members (seed stubs may lack
/// gunbc-emitted type surface — e.g. `ResolvedTree` in `v2_compiler_resolve`).
fn is_emitted_closure_member(emitted_lib_rs: &Path, module: &str, dest: &Path) -> bool {
    dest.is_file()
        && parse_closure_mods(emitted_lib_rs)
            .map(|mods| mods.iter().any(|m| m == module))
            .unwrap_or(false)
}

fn is_compiler_family_module(module: &str) -> bool {
    module.starts_with("v2_compiler_")
        || module.starts_with("extdeps_")
        || module.starts_with("v1_compiler_")
}

/// The emit-artifact sanitize scaffold is DISSOLVED (2026-07-23, dissolution
/// trigger in `cssl_emit_artifact_sanitize_scaffold_debt` fired): its last rule
/// (dedupe symbols across `pub use` lines) moved to the emitter's construction
/// seam (`strip_repeated_use_symbols` in v1.compiler.emit_rust), and the 21-module
/// curated sweep census (docs/probes/curated_cargo_frontier_probe_sweep.tsv,
/// raw_dup_pub_use column, measured on the RAW emit before assembly) shows zero
/// firings. Emit-retained artifacts are now byte-untouched by assembly.
///
/// Assembly arms: entry untouched · compiler seed re-export · bootstrap_inline ·
/// whole-closure emit-retain default (typed Refused only when closure mod lacks
/// emitted .rs).
pub fn assemble_seed_linked_closure(
    out_dir: &Path,
    entry_dag: &Path,
    repo_root: &Path,
    _std_bridge_dir: &Path,
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

        if is_compiler_family_module(&module)
            && !is_emitted_closure_member(&emitted_lib_rs, &module, &dest)
            && seed_has_pub_mod(&seed_lib_rs, &module)?
        {
            write_compiler_seed_reexport(&dest, &module)?;
            continue;
        }

        // Whole-closure default (cssl_closure_assembly_note): any module in the
        // gunbc-emitted closure manifest with a sibling .rs is emit-retained,
        // byte-untouched — v2_std_*, std_*, v1_rt, v2_extdeps_*, v2_lens_*,
        // gunbc_* product modules, test_* witnesses, tools_*, etc. Refusal
        // relocates to the cargo verdict, not assemble-time prefix whitelisting.
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
    fn closure_compiler_mod_emit_retained_when_seed_also_has_pub_mod() {
        let root = temp_fixture_root();
        let out = root.join("out");
        let src = out.join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(
            out.join("src/lib.rs"),
            "pub mod v2_compiler_resolve;\npub mod v2_compiler_infer;\n",
        )
        .expect("lib");
        fs::write(
            src.join("v2_compiler_resolve.rs"),
            "pub struct ResolvedTree;\n",
        )
        .expect("resolve");
        fs::write(src.join("v2_compiler_infer.rs"), "// entry\n").expect("infer");
        let seed_src = root.join("seed/src");
        fs::create_dir_all(&seed_src).expect("seed dir");
        fs::write(
            seed_src.join("lib.rs"),
            "pub mod v2_compiler_resolve;\npub mod v2_compiler_infer;\n",
        )
        .expect("seed lib");
        fs::write(seed_src.join("v2_compiler_resolve.rs"), "// seed stub\n").expect("seed");
        let repo = root.join("repo");
        fs::create_dir_all(repo.join("src/v1/stage0/src")).expect("seed path");
        fs::copy(
            seed_src.join("lib.rs"),
            repo.join("src/v1/stage0/src/lib.rs"),
        )
        .expect("copy");
        let dag = repo.join("src/v2/compiler/04_infer.dag");
        fs::create_dir_all(dag.parent().unwrap()).expect("dag dir");
        fs::write(&dag, "module v2.compiler.infer\n").expect("dag");
        let bridge = repo.join("dag/tools/self_host_std_bridge_shims");
        fs::create_dir_all(&bridge).expect("bridge");
        assemble_seed_linked_closure(&out, &dag, &repo, &bridge).expect("assemble");
        let kept = fs::read_to_string(src.join("v2_compiler_resolve.rs")).expect("read");
        assert!(
            kept.contains("pub struct ResolvedTree"),
            "closure member must stay emit-retained, not seed-shimmed"
        );
        assert!(!kept.contains("pub use v1_compiler::"));
    }

    #[test]
    fn deliberate_red_control_broken_closure_dep_surfaces_at_cargo() {
        let root = temp_fixture_root();
        let out = root.join("out");
        let src = out.join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(
            out.join("src/lib.rs"),
            "pub mod v2_compiler_tokenize;\npub mod v2_compiler_resolve;\n",
        )
        .expect("lib");
        fs::write(src.join("v2_compiler_tokenize.rs"), "// entry\n").expect("entry");
        fs::write(
            src.join("v2_compiler_resolve.rs"),
            "pub fn broken_syntax(] {}\n",
        )
        .expect("broken dep");
        let seed_src = root.join("seed/src");
        fs::create_dir_all(&seed_src).expect("seed dir");
        fs::write(
            seed_src.join("lib.rs"),
            "pub mod v2_compiler_resolve;\npub mod v2_compiler_tokenize;\n",
        )
        .expect("seed lib");
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
        assemble_seed_linked_closure(&out, &dag, &repo, &bridge).expect("assemble");
        let kept = fs::read_to_string(src.join("v2_compiler_resolve.rs")).expect("read");
        assert!(
            kept.contains("broken_syntax"),
            "emit-retain must not mask broken dep bytes"
        );
        fs::write(
            out.join("Cargo.toml"),
            "[package]\nname = \"red_control\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
        )
        .expect("cargo");
        let status = std::process::Command::new("cargo")
            .args(["build", "--lib"])
            .current_dir(&out)
            .env("RUSTC_WRAPPER", "")
            .status()
            .expect("cargo");
        assert!(
            !status.success(),
            "deliberate-red: broken closure dep must refuse cargo"
        );
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
    fn gunbc_product_closure_dep_emit_retained() {
        let root = temp_fixture_root();
        let out = write_minimal_emit_tree(
            &root,
            "v2_compiler_name_resolve",
            &["gunbc_plans_md_helpers", "v2_compiler_name_resolve"],
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
        let dag = repo.join("src/v2/compiler/03_name_resolve.dag");
        fs::create_dir_all(dag.parent().unwrap()).expect("dag dir");
        fs::write(&dag, "module v2.compiler.name_resolve\n").expect("dag");
        let bridge = repo.join("dag/tools/self_host_std_bridge_shims");
        fs::create_dir_all(&bridge).expect("bridge");
        assemble_seed_linked_closure(&out, &dag, &repo, &bridge).expect("assemble");
        let kept = fs::read_to_string(out.join("src/gunbc_plans_md_helpers.rs")).expect("read");
        assert!(kept.contains("emitted gunbc_plans_md_helpers"));
    }

    #[test]
    fn test_witness_closure_dep_emit_retained() {
        let root = temp_fixture_root();
        let out = write_minimal_emit_tree(
            &root,
            "v2_compiler_emit",
            &[
                "test_claim_materialization_ladder_witness",
                "v2_compiler_emit",
            ],
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
        let dag = repo.join("src/v2/compiler/05_emit.dag");
        fs::create_dir_all(dag.parent().unwrap()).expect("dag dir");
        fs::write(&dag, "module v2.compiler.emit\n").expect("dag");
        let bridge = repo.join("dag/tools/self_host_std_bridge_shims");
        fs::create_dir_all(&bridge).expect("bridge");
        assemble_seed_linked_closure(&out, &dag, &repo, &bridge).expect("assemble");
        let kept = fs::read_to_string(out.join("src/test_claim_materialization_ladder_witness.rs"))
            .expect("read");
        assert!(kept.contains("emitted test_claim_materialization_ladder_witness"));
    }

    #[test]
    fn closure_mod_missing_emitted_rs_is_typed_refused() {
        let root = temp_fixture_root();
        let out = root.join("out");
        let src = out.join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(
            out.join("src/lib.rs"),
            "pub mod not_a_routable_mod;\npub mod v2_compiler_tokenize;\n",
        )
        .expect("lib");
        fs::write(src.join("v2_compiler_tokenize.rs"), "// entry\n").expect("entry");
        let seed_src = root.join("seed/src");
        fs::create_dir_all(&seed_src).expect("seed dir");
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
                assert!(reason.contains("missing emitted .rs"));
            }
            other => panic!("expected RefusedDep, got {other:?}"),
        }
    }

    /// RED control for 03_normalize wet-receipt shim refresh: dropping `pub mod
    /// v2_compiler_namespace_graft` from the narrow hand lib must refuse cargo.
    #[test]
    fn normalize_stale_narrow_lib_without_namespace_graft_refuses_cargo() {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root = manifest
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .expect("repo root")
            .to_path_buf();
        let gunbc = root.join("target/release/gunbc");
        let assemble_bin = root.join("target/release/cssl_assemble");
        let shim_dir = root.join("dag/tools/self_host_03_normalize_shims");
        if !gunbc.is_file() || !assemble_bin.is_file() || !shim_dir.is_dir() {
            panic!(
                "release bins or shim dir missing (gunbc={}, assemble={}, shims={})",
                gunbc.is_file(),
                assemble_bin.is_file(),
                shim_dir.is_dir()
            );
        }
        let out =
            std::env::temp_dir().join(format!("cssl_normalize_stale_red_{}", std::process::id()));
        let _ = fs::remove_dir_all(&out);
        let compile_status = std::process::Command::new(&gunbc)
            .args([
                "compile",
                "--source-root",
                "dag",
                "--source-root",
                "src/v2",
                "--entry",
                "src/v2/compiler/03_normalize.dag",
                "--output-dir",
                &out.to_string_lossy(),
                "--target",
                "rust",
                "--dependency-pool-index",
                "primary-precedence",
            ])
            .current_dir(&root)
            .output()
            .expect("gunbc compile spawn");
        if !compile_status.status.success() {
            panic!(
                "RED-control setup refused: gunbc compile failed (exit={:?}):\n{}\n{}",
                compile_status.status.code(),
                String::from_utf8_lossy(&compile_status.stdout),
                String::from_utf8_lossy(&compile_status.stderr)
            );
        }
        let assemble_status = std::process::Command::new(&assemble_bin)
            .args([
                "--out-dir",
                &out.to_string_lossy(),
                "--entry-dag",
                "src/v2/compiler/03_normalize.dag",
                "--root",
                &root.to_string_lossy(),
                "--std-bridge-dir",
                "dag/tools/self_host_std_bridge_shims",
            ])
            .current_dir(&root)
            .output()
            .expect("cssl_assemble spawn");
        if !assemble_status.status.success() {
            panic!(
                "RED-control setup refused: cssl_assemble failed (exit={:?}):\n{}\n{}",
                assemble_status.status.code(),
                String::from_utf8_lossy(&assemble_status.stdout),
                String::from_utf8_lossy(&assemble_status.stderr)
            );
        }
        for entry in fs::read_dir(&shim_dir).expect("shim dir") {
            let entry = entry.expect("entry");
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name == "lib.rs" || name == "witness_main.rs" {
                continue;
            }
            fs::copy(entry.path(), out.join("src").join(name.as_ref())).expect("copy shim");
        }
        let stale_lib = "// stale RED control — namespace_graft pub mod deliberately dropped\n\
            #![allow(clippy::all, dead_code, unused_imports)]\n\
            pub use v1_compiler::NonEmptyVec;\n\
            pub use v1_compiler::NonEmptyBTreeSet;\n\
            pub use v1_compiler::v1_rt;\n\
            pub mod std_algebra;\npub mod std_types;\npub mod v2_std_integer;\n\
            pub mod v2_std_algebra;\npub mod v2_std_collection;\npub mod v2_std_grammar;\n\
            pub mod v2_std_diagnostic;\npub mod v2_std_node;\npub mod v2_std_compilers_sugar;\n\
            pub mod v2_compiler_body_lowering_fold;\npub mod v2_compiler_normalized_tree;\n\
            pub mod v2_extdeps_languages_dag;\npub mod v2_compiler_normalize;\n";
        fs::write(out.join("src/lib.rs"), stale_lib).expect("stale lib");
        fs::write(
            out.join("Cargo.toml"),
            format!(
                "[package]\nname = \"stale_red\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n\
                 [lib]\npath = \"src/lib.rs\"\n\n[dependencies]\n\
                 im = {{ version = \"15.1\", features = [\"serde\"] }}\n\
                 v1-compiler = {{ path = \"{}\" }}\n",
                root.join("src/v1/stage0").display()
            ),
        )
        .expect("cargo");
        let status = std::process::Command::new("cargo")
            .args(["build", "--lib"])
            .current_dir(&out)
            .env("RUSTC_WRAPPER", "")
            .env("CTRL_BUILD_WRAP_CARGO", "0")
            .status()
            .expect("cargo");
        assert!(
            !status.success(),
            "stale narrow lib without namespace_graft must refuse cargo"
        );
        let _ = fs::remove_dir_all(&out);
    }
}
