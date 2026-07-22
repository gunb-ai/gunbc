//! Generic seed-shim assembly for curated self-host behavioral receipts.
//! Authority: tools.self_host_curated_seed_linked_harness (5-arm design).
//! dissolve-on: v2 std self-emits + gunbc emits seed-linked extern imports.

use sha2::Digest;
use std::collections::HashSet;
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

/// SCAFFOLD — see `cssl_emit_artifact_sanitize_scaffold_debt` in
/// `dag/tools/self_host_curated_seed_linked_harness.dag`. One remaining structural
/// pass on every emit-retained artifact: dedupe symbols across sequential
/// `pub use path::{...}` lines (emitter re-exports the same name via transitive
/// paths — e.g. `Optional` via both `v2_std_grammar` and `v2_std_collection`).
///
/// The former `use crate::v1_rt::NAME` strip (redundant import when the file
/// locally defines `pub enum/struct/type NAME`) was CENSUSED DEAD 2026-07-22 across
/// a representative sweep of the deepest emit-retained closures (02_parse,
/// 06_translate, 00_compile, 05_eval, materialization_carriers, 05_emit,
/// emit_module, emit_produced, emit_semantic_decl, emit_host, fold_lowering,
/// 03_name_resolve — zero real firings, confirmed by content diff, not raw
/// `!=` which false-positives on this fn's own trailing-newline rejoin) —
/// #6981 (`emit_imports`: field-struct import synth asks "provides" not
/// "physically defines") and #7033 (fold-closure generic-T, Witness prelude,
/// value-qualified refs) dissolved it at the source; deleted here, not improved.
fn sanitize_emitter_artifact_in_place(path: &Path) -> Result<(), AssemblyError> {
    let content = fs::read_to_string(path)?;
    let sanitized = sanitize_emit_artifact_content(&content);
    if sanitized != content {
        eprintln!(
            "CSSL_ASSEMBLE: sanitize scaffold applied to {}",
            path.display()
        );
        fs::write(path, sanitized)?;
    }
    Ok(())
}

fn dedupe_pub_use_symbols(content: &str) -> String {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for line in content.lines() {
        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub use ") {
            if let Some(brace_start) = rest.find('{') {
                if let Some(brace_end) = rest.rfind('}') {
                    if brace_end > brace_start {
                        let path_prefix = &rest[..brace_start];
                        let symbols_str = &rest[brace_start + 1..brace_end];
                        let suffix = &rest[brace_end + 1..];
                        let symbols: Vec<&str> = symbols_str
                            .split(',')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .collect();
                        let mut kept = Vec::new();
                        for sym in symbols {
                            if seen.insert(sym.to_string()) {
                                kept.push(sym);
                            }
                        }
                        if kept.is_empty() {
                            continue;
                        }
                        out.push(format!(
                            "{indent}pub use {}{{{}}}{suffix}",
                            path_prefix,
                            kept.join(", "),
                        ));
                        continue;
                    }
                }
            }
        }
        out.push(line.to_string());
    }
    out.join("\n")
}

fn sanitize_emit_artifact_content(content: &str) -> String {
    dedupe_pub_use_symbols(content)
}

/// Assembly arms: entry untouched · compiler seed re-export · shared std-bridge · dag/std
/// emit-retain · lens/peripheral emit-retain · bootstrap_inline · typed Refused.
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

        if is_compiler_family_module(&module) {
            if is_emitted_closure_member(&emitted_lib_rs, &module, &dest) {
                // Structural: closure manifest membership → emit-retain (gunbc surface).
                sanitize_emitter_artifact_in_place(&dest)?;
            } else if seed_has_pub_mod(&seed_lib_rs, &module)? {
                write_compiler_seed_reexport(&dest, &module)?;
            } else {
                sanitize_emitter_artifact_in_place(&dest)?;
            }
            continue;
        }

        if module.starts_with("v2_std_") {
            // Emit-retain: gunbc-emitted v2 std is closure authority. Hand bridges
            // (dag/tools/self_host_std_bridge_shims) are minimal ABI stubs that break
            // dependents (e.g. v2_std_logic re-exports from v2_std_algebra).
            sanitize_emitter_artifact_in_place(&dest)?;
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

    /// RED control for the v1_rt-import-strip DISSOLUTION (2026-07-22 census):
    /// fixture is the real, current `gunbc compile --target rust` output for
    /// `v2.std.witness` (src/v2/std/witness.dag, entry `src/v2/compiler/02_parse.dag`
    /// closure) — a locally-defined `pub enum Witness<C>` with NO redundant
    /// `use crate::v1_rt::Witness` import line. Before #6981/#7033 this fixture would
    /// have carried that redundant import and the (now-deleted) strip rule would have
    /// fired on it; today `sanitize_emit_artifact_content` is a byte-identical no-op,
    /// proving the emitter — not the scaffold — produces the correct form natively.
    #[test]
    fn witness_emit_no_longer_carries_redundant_v1_rt_import_construction_receipt() {
        let real_emitted_witness_src = concat!(
            "// Generated by v1 compiler -- do not edit.\n",
            "// Source module: v2.std.witness\n",
            "\n",
            "use im::{vector as vec, HashMap, OrdSet as BTreeSet, Vector as Vec};\n",
            "use crate::v1_rt::{VecCompat, VecJoin};\n",
            "use std::rc::Rc;\n",
            "use crate::v1_rt;\n",
            "use crate::NonEmptyVec;\n",
            "use crate::NonEmptyBTreeSet;\n",
            "pub use crate::v2_std_diagnostic::{Diagnostic};\n",
            "pub use crate::v2_std_node::{Node, Symbol};\n",
            "use self::Witness::*;\n",
            "\n",
            "#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]\n",
            "#[serde(tag = \"_variant\")]\n",
            "pub enum Witness<C> {\n",
            "    Holds {\n",
            "        value: C,\n",
            "    },\n",
            "    Violates {\n",
            "        diagnostic: Rc<Diagnostic>,\n",
            "    },\n",
            "}\n",
        );
        // Precondition the fixture actually exercises the dissolved rule's trigger shape:
        // a locally-defined generic `pub enum Witness<C>` alongside `use crate::v1_rt::`
        // lines for OTHER names (VecCompat/VecJoin/the bare module) — never `Witness` itself.
        assert!(real_emitted_witness_src.contains("pub enum Witness<C>"));
        assert!(!real_emitted_witness_src.contains("use crate::v1_rt::Witness"));
        let kept = sanitize_emit_artifact_content(real_emitted_witness_src);
        // trim_end: the surviving dedupe pass always rejoins lines (drops a trailing
        // newline even as a no-op) — semantically a no-op, not a second live rule.
        assert_eq!(
            kept.trim_end(),
            real_emitted_witness_src.trim_end(),
            "sanitize must be a no-op on current emitter output — the redundant \
             v1_rt::Witness import that the deleted rule targeted is already absent"
        );
    }

    #[test]
    fn pub_use_symbol_dedup_clears_transitive_reexport_collision() {
        let src = concat!(
            "pub use crate::v2_std_grammar::{grammar_formal_terminal, Optional};\n",
            "pub use crate::v2_std_collection::{List, Map, Set, Optional};\n"
        );
        let kept = sanitize_emit_artifact_content(src);
        assert_eq!(kept.matches("Optional").count(), 1);
        assert!(!kept.contains("Set, Optional"));
    }

    /// LIVE rule receipt via the in-place path (2026-07-22 census): captured from real
    /// `gunbc compile --target rust` output for `v2.extdeps.languages.rust` in the
    /// `src/v2/compiler/emit_produced.dag` closure — `Optional` is re-exported once via
    /// `v2_std_grammar` and again via `v2_std_collection` (transitive re-export collision,
    /// still live on main; emitter gap named in `cssl_emit_artifact_sanitize_scaffold_debt`).
    #[test]
    fn pub_use_dedup_file_sanitize_via_in_place_path() {
        let root = temp_fixture_root();
        let out = root.join("out");
        let src = out.join("src");
        fs::create_dir_all(&src).expect("src");
        let real_collision_src = concat!(
            "pub use crate::v2_std_grammar::{grammar_formal_terminal, Optional};\n",
            "pub use crate::v2_std_collection::{List, Map, Set, Optional};\n",
        );
        let path = src.join("v2_extdeps_languages_rust.rs");
        fs::write(&path, real_collision_src).expect("write collision fixture");
        sanitize_emitter_artifact_in_place(&path).expect("sanitize");
        let kept = fs::read_to_string(&path).expect("read");
        assert_eq!(kept.matches("Optional").count(), 1);
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
