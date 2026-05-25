//! **Layer:** integration
//!
//! P9 single-owner receipt: `llvm_instruction_cost` must be defined exactly once in
//! `src/v4/lens/cost.dag`. Replaces the dissolved `v4_lens_cost_dag_smoke_test.rs`
//! ratchet — whole-tree parse does **not** substitute (see `docs/v4-close-interrogation.md`
//! §P5(b) / `feature:p9-llvm-instruction-cost-single-owner`).
//!
//! Complements the structural `.dag` TestClaim in
//! `src/v4/test/claim/lens_cost/p9_llvm_instruction_cost_registry_owner.dag`
//! (registry exclusivity) with a corpus scan for shadow `fn llvm_instruction_cost`
//! definitions outside the canonical owner module.
//!
//! **TESTING.md:** M1(2.7) tokenize/parse surface scan (`SurfaceItem::Fn` /
//! `FnExternalBody`); per-file isolation matches peer v4 smoke harness posture.
//!
//! **ROADMAP:** T-PB-B / `pb_rust_tests_outside_residual_zero`; dissolves when M2
//! reflection or generated harness executes the `.dag` claim over the full corpus.

use std::fs;
use std::path::{Path, PathBuf};

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::SurfaceItem;
use v3_compiler::tokenize_for_test;

const CANONICAL_OWNER: &str = "src/v4/lens/cost.dag";
const TARGET_FN_NAME: &str = "llvm_instruction_cost";

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(3)
        .expect("workspace root is three ancestors above src/v3/compiler/")
        .to_path_buf()
}

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

fn surface_declares_fn(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Fn {
            name: item_name, ..
        }
        | SurfaceItem::FnExternalBody {
            name: item_name, ..
        } => item_name == name,
        _ => false,
    })
}

fn collect_dag_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
    for ent in entries {
        let ent = ent.unwrap_or_else(|e| panic!("read_dir entry {}: {e}", dir.display()));
        let p = ent.path();
        if p.is_dir() {
            collect_dag_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "dag") {
            out.push(p);
        }
    }
}

fn rel_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[test]
fn v4_p9_llvm_instruction_cost_defined_only_in_lens_cost_dag() {
    let root = workspace_root();
    let scan_root = root.join("src/v4");
    assert!(
        scan_root.is_dir(),
        "expected v4 substrate tree {} — scope drift?",
        scan_root.display()
    );

    let mut paths = Vec::new();
    collect_dag_files(&scan_root, &mut paths);
    paths.sort();
    paths.dedup();

    let mut defs = Vec::new();
    for path in paths {
        let rel = rel_path(&root, &path);
        let source =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        let module = parse_module(&source, &rel);
        if surface_declares_fn(&module, TARGET_FN_NAME) {
            defs.push(rel);
        }
    }

    assert_eq!(
        defs.len(),
        1,
        "P9 single-owner: `fn {TARGET_FN_NAME}` must be declared exactly once under src/v4/ \
         (canonical owner `{CANONICAL_OWNER}`); whole-tree parse does not substitute. \
         Found {} module(s):\n{}",
        defs.len(),
        defs.join("\n")
    );
    assert_eq!(
        defs[0], CANONICAL_OWNER,
        "P9 single-owner: `fn {TARGET_FN_NAME}` must live in `{CANONICAL_OWNER}`; got: {}",
        defs[0]
    );
}

#[test]
fn v4_p9_llvm_instruction_cost_absent_from_llvm_ir_dag() {
    let root = workspace_root();
    let llvm_ir = root.join("src/v4/extdeps/languages/llvm_ir.dag");
    let rel = rel_path(&root, &llvm_ir);
    let source =
        fs::read_to_string(&llvm_ir).unwrap_or_else(|e| panic!("read {}: {e}", llvm_ir.display()));
    let module = parse_module(&source, &rel);
    assert!(
        !surface_declares_fn(&module, TARGET_FN_NAME),
        "P9: `fn {TARGET_FN_NAME}` must not be declared in llvm_ir.dag (cost authority is v4.lens.cost only)"
    );
}
