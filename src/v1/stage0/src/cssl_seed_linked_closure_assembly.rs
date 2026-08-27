//! Generic seed-shim assembly for curated self-host behavioral receipts.
//! Authority: tools.self_host_curated_seed_linked_harness (5-arm design).
//! dissolve-on: v2 std self-emits + gunbc emits seed-linked extern imports.

use sha2::Digest;
use std::fs;
use std::path::{Path, PathBuf};

use v1_compiler::gunbc_stage0_emitted_population_manifest::{
    emitted_population_manifest_basename, emitted_population_manifest_line_prefix,
    emitted_population_manifest_line_separator,
};

#[derive(Debug)]
pub enum AssemblyError {
    MissingEntryFile {
        path: PathBuf,
    },
    EntryMutated {
        before: String,
        after: String,
    },
    MissingPopulationManifest {
        path: PathBuf,
    },
    EmptyPopulationManifest {
        path: PathBuf,
    },
    RefusedDeclaredMember {
        declared_path: String,
        reason: String,
    },
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
            Self::MissingPopulationManifest { path } => {
                write!(f, "missing emitted-population manifest {path:?}")
            }
            Self::EmptyPopulationManifest { path } => write!(
                f,
                "{path:?} carried no declared paths -- it is not the emitted-population manifest"
            ),
            Self::RefusedDeclaredMember {
                declared_path,
                reason,
            } => {
                write!(
                    f,
                    "refused declared emitted member {declared_path}: {reason}"
                )
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
        .ok_or_else(|| AssemblyError::RefusedDeclaredMember {
            declared_path: dag_path.display().to_string(),
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

/// Recover the emitted population from the emitter's own declaration of it.
///
/// THIS REPLACES A RUST TEXT-PARSE, AND THAT IS THE POINT RATHER THAN A REFACTOR. Assembly
/// used to recover closure membership by scraping `pub mod <name>;` lines out of the emitted
/// `lib.rs` -- a hand-rolled reader of Rust surface syntax standing in for a fact the emitter
/// already knows and already writes down. `v1.compiler.emit_rust` `emit_emitted_population_manifest`
/// renders one line per produced path into `src/emitted_population.rs`, in the same `out_dir`,
/// under a grammar both ends resolve from one authority
/// (`gunbc.stage0_emitted_population_manifest`): `line_prefix ++ path`, joined by the separator.
/// Reading it is a split and a strip. No Rust grammar is involved on either side, which is
/// exactly the property that made the artifact a file of `//` lines in the first place.
///
/// A line that does not carry the prefix is DROPPED rather than guessed at, matching the modeled
/// reader in `gunbc.stage0_rust_host_observation` `emitted_population_paths_from_manifest`: one
/// grammar read in both directions (DESIGN section 4), never a second parser with its own idea
/// of the shape.
fn declared_emitted_paths(manifest: &Path) -> Result<Vec<String>, AssemblyError> {
    let content = fs::read_to_string(manifest)?;
    let prefix = emitted_population_manifest_line_prefix();
    let separator = emitted_population_manifest_line_separator();
    Ok(content
        .split(separator.as_str())
        .filter_map(|line| line.strip_prefix(prefix.as_str()))
        .map(|path| path.to_string())
        .collect())
}

/// The emit-artifact sanitize scaffold is DISSOLVED (2026-07-23): its last rule
/// (dedupe symbols across `pub use` lines) moved to the emitter's construction
/// seam (`strip_repeated_use_symbols` in v1.compiler.emit_rust), and the 21-module
/// curated sweep census (a probe TSV deleted 2026-08-16 (operator: delete anything not actively derived),
/// raw_dup_pub_use column, measured on the RAW emit before assembly) shows zero
/// firings. Emit-retained artifacts are now byte-untouched by assembly.
///
/// Assembly arms: entry untouched · whole-closure emit-retain default (typed refusal only
/// when a member the emitter DECLARED it produced is not on disk).
///
/// THE LAST RUST TEXT-PARSE ON THIS PATH IS GONE, AND WITH IT THE SECOND AUTHORITY OVER
/// EMITTED-POPULATION MEMBERSHIP. The prior revision derived the population from `lib.rs`'s
/// `pub mod` lines and then asserted a sibling `.rs` existed for each. That check compared one
/// emit artifact against another emit artifact -- `lib.rs`'s mod list is rendered FROM the same
/// module file list the writer then writes out, so on the real path it could only ever agree
/// with itself, and its only reachable red came from a hand-authored `lib.rs`. The manifest
/// makes the question answerable at the grain that matters: the emitter DECLARES the paths it
/// produced, and assembly refuses if any declared path is not materialized in `out_dir`. That is
/// strictly wider coverage (`Cargo.toml`, `lib.rs`, `main.rs`, every module, every test file and
/// the manifest itself are all declared members) obtained by deleting a parser rather than by
/// adding a check.
///
/// `BOOTSTRAP_INLINE_MODS` GOES WITH IT. It named `NonEmptyVec` and `NonEmptyBTreeSet`, whose
/// only reason to exist was that a hand-authored shim `lib.rs` can carry `pub mod` lines for
/// wrappers the emitter inlines as bare `struct`s rather than emitting as files. The manifest
/// never declares them -- they are not files -- so the skip has no subject: the state it
/// existed to tolerate is not representable in the population assembly now reads, which is
/// DESIGN 4b's top rung rather than the one below it.
///
/// THE COMPILER-SEED-RE-EXPORT ARM WAS DELETED EARLIER (gunbc#8690), AND WITH IT THE ONLY READ
/// OF THE COMMITTED SEED `lib.rs`. That arm replaced a closure member's emitted bytes with
/// `pub use v1_compiler::{mod}::*` when the seed's `lib.rs` text carried a matching `pub mod`
/// line -- a second authority over seed mod-tree membership, decided by text-parsing a file
/// whose modeled authority is `v2.compiler.self_host.stage0_crate_layout`. The `repo_root`
/// PARAMETER went with it, so assembly has no input from which the seed `lib.rs` location is
/// derivable at all. `closure_compiler_mod_stays_emit_retained` remains enrolled as the
/// regression control for that climb (DESIGN 4b(4)): a compiler-family closure member keeps its
/// emitted bytes and never becomes a `pub use v1_compiler::` re-export.
pub fn assemble_seed_linked_closure(
    out_dir: &Path,
    entry_dag: &Path,
    _std_bridge_dir: &Path,
) -> Result<(), AssemblyError> {
    let src_dir = out_dir.join("src");
    let manifest_path = src_dir.join(emitted_population_manifest_basename());
    let entry_mod = dag_entry_rust_module(entry_dag)?;
    let entry_file = src_dir.join(format!("{entry_mod}.rs"));

    if !entry_file.is_file() {
        return Err(AssemblyError::MissingEntryFile { path: entry_file });
    }
    if !manifest_path.is_file() {
        return Err(AssemblyError::MissingPopulationManifest {
            path: manifest_path,
        });
    }

    let entry_hash_before = sha256_hex(&entry_file)?;
    let declared = declared_emitted_paths(&manifest_path)?;
    if declared.is_empty() {
        return Err(AssemblyError::EmptyPopulationManifest {
            path: manifest_path,
        });
    }

    for declared_path in declared {
        // Whole-closure default (cssl_closure_assembly_note): every member the emitter
        // declared it produced is emit-retained, byte-untouched -- v2_std_*, std_*, v1_rt,
        // v2_extdeps_*, v2_lens_*, gunbc_* product modules, test_* witnesses, tools_*, etc.
        // Assembly's only obligation is that the declaration and the tree agree; refusal for
        // anything the bytes then say relocates to the cargo verdict, not to assemble-time
        // prefix whitelisting.
        if !out_dir.join(&declared_path).is_file() {
            return Err(AssemblyError::RefusedDeclaredMember {
                declared_path,
                reason: "declared emitted path missing from out_dir".to_string(),
            });
        }
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

    /// Write the emitter's own declaration of what it produced, under the shared grammar.
    ///
    /// Fixtures build this the way `emit_emitted_population_manifest` does -- one prefixed line
    /// per produced path, the manifest naming itself, sorted -- rather than hand-spelling `// `
    /// and `\n`, so a change to the grammar authority moves both ends at once instead of
    /// leaving the fixtures agreeing with a spelling nothing else uses.
    fn write_population_manifest(out: &Path, paths: &[String]) -> Result<(), AssemblyError> {
        let prefix = emitted_population_manifest_line_prefix();
        let separator = emitted_population_manifest_line_separator();
        let mut declared: std::vec::Vec<String> = paths.to_vec();
        declared.push(format!("src/{}", emitted_population_manifest_basename()));
        declared.sort();
        let body = declared
            .iter()
            .map(|path| format!("{prefix}{path}"))
            .collect::<std::vec::Vec<_>>()
            .join(separator.as_str());
        fs::write(
            out.join("src").join(emitted_population_manifest_basename()),
            format!("{body}{separator}"),
        )?;
        Ok(())
    }

    fn write_minimal_emit_tree(
        root: &Path,
        entry_mod: &str,
        closure_mods: &[&str],
    ) -> Result<PathBuf, AssemblyError> {
        let out = root.join("out");
        let src = out.join("src");
        fs::create_dir_all(&src)?;
        let mut declared = std::vec![format!("src/lib.rs"), format!("src/{entry_mod}.rs")];
        let lib = closure_mods
            .iter()
            .map(|m| format!("pub mod {m};"))
            .collect::<std::vec::Vec<_>>()
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
            declared.push(format!("src/{m}.rs"));
        }
        write_population_manifest(&out, &declared)?;
        Ok(out)
    }

    /// Per-CASE root, not per-process. Every case builds its tree at `<root>/out`, and the
    /// prior helper keyed the root on the pid alone, so the whole module shared one directory
    /// while cargo ran the cases CONCURRENTLY -- each `remove_dir_all` racing every sibling's
    /// fixture writes. It passed by timing rather than by construction; keying on the case name
    /// makes the collision unrepresentable instead of unlikely.
    fn temp_fixture_root(case: &str) -> PathBuf {
        let base =
            std::env::temp_dir().join(format!("cssl_assembly_test_{}_{case}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).expect("temp root");
        base
    }

    fn entry_dag_and_bridge(root: &Path, stem: &str, module: &str) -> (PathBuf, PathBuf) {
        let repo = root.join("repo");
        let dag = repo.join(format!("src/v2/compiler/{stem}.dag"));
        fs::create_dir_all(dag.parent().unwrap()).expect("dag dir");
        fs::write(&dag, format!("module {module}\n")).expect("dag");
        let bridge = repo.join("dag/tools/self_host_std_bridge_shims");
        fs::create_dir_all(&bridge).expect("bridge");
        (dag, bridge)
    }

    #[test]
    fn std_prefix_deps_are_emit_retained_not_refused() {
        let root = temp_fixture_root("std_prefix_deps_are_emit_retained_not_refused");
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
        let repo = root.join("repo");
        let dag = repo.join("src/v2/compiler/01_tokenize.dag");
        fs::create_dir_all(dag.parent().unwrap()).expect("dag dir");
        fs::write(&dag, "module v2.compiler.tokenize\n").expect("dag");
        let bridge = repo.join("dag/tools/self_host_std_bridge_shims");
        fs::create_dir_all(&bridge).expect("bridge");
        assemble_seed_linked_closure(&out, &dag, &bridge).expect("assemble");
        let kept = fs::read_to_string(out.join("src/std_error_primitives.rs")).expect("read");
        assert!(kept.contains("emitted std_error_primitives"));
    }

    #[test]
    // REGRESSION CONTROL for the deleted compiler-seed-re-export arm. Its former subject --
    // a seed `lib.rs` carrying the same `pub mod` line -- is no longer constructible: assembly
    // takes no repo root, so no input names the seed tree. What remains checkable, and what
    // the arm actually broke, is that a compiler-family closure member keeps its EMITTED bytes
    // and never becomes a `pub use v1_compiler::` re-export. `ResolvedTree` is the discriminating
    // payload: it is the type surface a seed stub would have lacked.
    fn closure_compiler_mod_stays_emit_retained() {
        let root = temp_fixture_root("closure_compiler_mod_stays_emit_retained");
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
        write_population_manifest(
            &out,
            &[
                "src/lib.rs".to_string(),
                "src/v2_compiler_resolve.rs".to_string(),
                "src/v2_compiler_infer.rs".to_string(),
            ],
        )
        .expect("manifest");
        let repo = root.join("repo");
        let dag = repo.join("src/v2/compiler/04_infer.dag");
        fs::create_dir_all(dag.parent().unwrap()).expect("dag dir");
        fs::write(&dag, "module v2.compiler.infer\n").expect("dag");
        let bridge = repo.join("dag/tools/self_host_std_bridge_shims");
        fs::create_dir_all(&bridge).expect("bridge");
        assemble_seed_linked_closure(&out, &dag, &bridge).expect("assemble");
        let kept = fs::read_to_string(src.join("v2_compiler_resolve.rs")).expect("read");
        assert!(
            kept.contains("pub struct ResolvedTree"),
            "closure member must stay emit-retained, not seed-shimmed"
        );
        assert!(!kept.contains("pub use v1_compiler::"));
    }

    #[test]
    fn deliberate_red_control_broken_closure_dep_surfaces_at_cargo() {
        let root = temp_fixture_root("deliberate_red_control_broken_closure_dep_surfaces_at_cargo");
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
        write_population_manifest(
            &out,
            &[
                "src/lib.rs".to_string(),
                "src/v2_compiler_tokenize.rs".to_string(),
                "src/v2_compiler_resolve.rs".to_string(),
            ],
        )
        .expect("manifest");
        let repo = root.join("repo");
        let dag = repo.join("src/v2/compiler/01_tokenize.dag");
        fs::create_dir_all(dag.parent().unwrap()).expect("dag dir");
        fs::write(&dag, "module v2.compiler.tokenize\n").expect("dag");
        let bridge = repo.join("dag/tools/self_host_std_bridge_shims");
        fs::create_dir_all(&bridge).expect("bridge");
        assemble_seed_linked_closure(&out, &dag, &bridge).expect("assemble");
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
        let root = temp_fixture_root("extdeps_without_seed_stays_emitted");
        let out = write_minimal_emit_tree(
            &root,
            "v2_compiler_self_host",
            &["extdeps_communication_medium", "v2_compiler_self_host"],
        )
        .expect("tree");
        let repo = root.join("repo");
        let dag = repo.join("src/v2/compiler/self_host.dag");
        fs::create_dir_all(dag.parent().unwrap()).expect("dag dir");
        fs::write(&dag, "module v2.compiler.self_host\n").expect("dag");
        let bridge = repo.join("dag/tools/self_host_std_bridge_shims");
        fs::create_dir_all(&bridge).expect("bridge");
        assemble_seed_linked_closure(&out, &dag, &bridge).expect("assemble");
        let kept =
            fs::read_to_string(out.join("src/extdeps_communication_medium.rs")).expect("read");
        assert!(kept.contains("emitted extdeps_communication_medium"));
    }

    #[test]
    fn dry_run_peripheral_emit_retain_holds() {
        let root = temp_fixture_root("dry_run_peripheral_emit_retain_holds");
        let out = write_minimal_emit_tree(
            &root,
            "v2_compiler_self_host",
            &["dry_run", "v2_compiler_self_host"],
        )
        .expect("tree");
        let repo = root.join("repo");
        let dag = repo.join("src/v2/compiler/self_host.dag");
        fs::create_dir_all(dag.parent().unwrap()).expect("dag dir");
        fs::write(&dag, "module v2.compiler.self_host\n").expect("dag");
        let bridge = repo.join("dag/tools/self_host_std_bridge_shims");
        fs::create_dir_all(&bridge).expect("bridge");
        assemble_seed_linked_closure(&out, &dag, &bridge).expect("assemble");
        let kept = fs::read_to_string(out.join("src/dry_run.rs")).expect("read");
        assert!(kept.contains("emitted dry_run"));
    }

    #[test]
    fn gunbc_product_closure_dep_emit_retained() {
        let root = temp_fixture_root("gunbc_product_closure_dep_emit_retained");
        let out = write_minimal_emit_tree(
            &root,
            "v2_compiler_name_resolve",
            &["gunbc_plans_md_helpers", "v2_compiler_name_resolve"],
        )
        .expect("tree");
        let repo = root.join("repo");
        let dag = repo.join("src/v2/compiler/03_name_resolve.dag");
        fs::create_dir_all(dag.parent().unwrap()).expect("dag dir");
        fs::write(&dag, "module v2.compiler.name_resolve\n").expect("dag");
        let bridge = repo.join("dag/tools/self_host_std_bridge_shims");
        fs::create_dir_all(&bridge).expect("bridge");
        assemble_seed_linked_closure(&out, &dag, &bridge).expect("assemble");
        let kept = fs::read_to_string(out.join("src/gunbc_plans_md_helpers.rs")).expect("read");
        assert!(kept.contains("emitted gunbc_plans_md_helpers"));
    }

    #[test]
    fn test_witness_closure_dep_emit_retained() {
        let root = temp_fixture_root("test_witness_closure_dep_emit_retained");
        let out = write_minimal_emit_tree(
            &root,
            "v2_compiler_emit",
            &[
                "test_claim_materialization_ladder_witness",
                "v2_compiler_emit",
            ],
        )
        .expect("tree");
        let repo = root.join("repo");
        let dag = repo.join("src/v2/compiler/05_emit.dag");
        fs::create_dir_all(dag.parent().unwrap()).expect("dag dir");
        fs::write(&dag, "module v2.compiler.emit\n").expect("dag");
        let bridge = repo.join("dag/tools/self_host_std_bridge_shims");
        fs::create_dir_all(&bridge).expect("bridge");
        assemble_seed_linked_closure(&out, &dag, &bridge).expect("assemble");
        let kept = fs::read_to_string(out.join("src/test_claim_materialization_ladder_witness.rs"))
            .expect("read");
        assert!(kept.contains("emitted test_claim_materialization_ladder_witness"));
    }

    /// RED, AND DISCRIMINATING FOR THE SWAP ITSELF. The manifest declares
    /// `src/not_a_routable_mod.rs` and no such file exists, so assembly refuses -- while
    /// `lib.rs` NEVER NAMES IT. The retired `pub mod` scrape read its population out of
    /// `lib.rs`, so it saw nothing to check here and this fixture went green under it. The
    /// refusal is therefore evidence that the manifest is the authority now, not merely that
    /// some refusal survived the change.
    #[test]
    fn declared_member_missing_from_out_dir_is_typed_refused() {
        let root = temp_fixture_root("declared_member_missing_from_out_dir_is_typed_refused");
        let out = root.join("out");
        let src = out.join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(out.join("src/lib.rs"), "pub mod v2_compiler_tokenize;\n").expect("lib");
        fs::write(src.join("v2_compiler_tokenize.rs"), "// entry\n").expect("entry");
        write_population_manifest(
            &out,
            &[
                "src/lib.rs".to_string(),
                "src/v2_compiler_tokenize.rs".to_string(),
                "src/not_a_routable_mod.rs".to_string(),
            ],
        )
        .expect("manifest");
        let (dag, bridge) = entry_dag_and_bridge(&root, "01_tokenize", "v2.compiler.tokenize");
        let err = assemble_seed_linked_closure(&out, &dag, &bridge).unwrap_err();
        match err {
            AssemblyError::RefusedDeclaredMember {
                declared_path,
                reason,
            } => {
                assert_eq!(declared_path, "src/not_a_routable_mod.rs");
                assert!(reason.contains("missing from out_dir"));
            }
            other => panic!("expected RefusedDeclaredMember, got {other:?}"),
        }
    }

    /// The population has one authority, so its ABSENCE is a refusal and never a fall back to
    /// reading `lib.rs`. An `out_dir` with a perfectly good `lib.rs` and no manifest is exactly
    /// the state the retired parse would have accepted.
    #[test]
    fn missing_population_manifest_is_typed_refused() {
        let root = temp_fixture_root("missing_population_manifest_is_typed_refused");
        let out = root.join("out");
        let src = out.join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(out.join("src/lib.rs"), "pub mod v2_compiler_tokenize;\n").expect("lib");
        fs::write(src.join("v2_compiler_tokenize.rs"), "// entry\n").expect("entry");
        let (dag, bridge) = entry_dag_and_bridge(&root, "01_tokenize", "v2.compiler.tokenize");
        match assemble_seed_linked_closure(&out, &dag, &bridge).unwrap_err() {
            AssemblyError::MissingPopulationManifest { path } => {
                assert!(path.ends_with(emitted_population_manifest_basename()));
            }
            other => panic!("expected MissingPopulationManifest, got {other:?}"),
        }
    }

    /// A file at the manifest's path carrying no line under the manifest grammar is IGNORANCE,
    /// not an empty population: answering "nothing was declared, so nothing is missing" would be
    /// the absorbing fallback read in the narrowing direction (DESIGN section 5).
    #[test]
    fn population_manifest_without_declared_lines_is_typed_refused() {
        let root = temp_fixture_root("population_manifest_without_declared_lines_is_typed_refused");
        let out = root.join("out");
        let src = out.join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(src.join("v2_compiler_tokenize.rs"), "// entry\n").expect("entry");
        fs::write(
            src.join(emitted_population_manifest_basename()),
            "not a declared line\n",
        )
        .expect("manifest");
        let (dag, bridge) = entry_dag_and_bridge(&root, "01_tokenize", "v2.compiler.tokenize");
        match assemble_seed_linked_closure(&out, &dag, &bridge).unwrap_err() {
            AssemblyError::EmptyPopulationManifest { path } => {
                assert!(path.ends_with(emitted_population_manifest_basename()));
            }
            other => panic!("expected EmptyPopulationManifest, got {other:?}"),
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
        // Mirror the roster row's shim writes: the shared std surface comes from the
        // std-bridge (one authority, not one copy per transport), the rest from this
        // transport's own shim dir. Copying only shim_dir would leave the emitted std
        // stubs in place and the control would then refuse for the wrong reason —
        // non-discriminating, since the assertion below is about the dropped
        // `pub mod v2_compiler_namespace_graft`, not about a broken std surface.
        let std_bridge_dir = root.join("dag/tools/self_host_std_bridge_shims");
        for dir in [&std_bridge_dir, &shim_dir] {
            for entry in fs::read_dir(dir).expect("shim dir") {
                let entry = entry.expect("entry");
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name == "lib.rs" || name == "witness_main.rs" {
                    continue;
                }
                fs::copy(entry.path(), out.join("src").join(name.as_ref())).expect("copy shim");
            }
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
