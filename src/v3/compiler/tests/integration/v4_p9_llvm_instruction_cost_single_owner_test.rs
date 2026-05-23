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
//! **ROADMAP:** T-PB-B / `pb_rust_tests_outside_residual_zero`; dissolves when M2
//! reflection or generated harness executes the `.dag` claim over the full corpus.

use std::fs;
use std::path::{Path, PathBuf};

const CANONICAL_OWNER: &str = "src/v4/lens/cost.dag";
const FN_DEF_NEEDLE: &str = "fn llvm_instruction_cost";

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(3)
        .expect("workspace root is three ancestors above src/v3/compiler/")
        .to_path_buf()
}

fn dag_line_without_trailing_line_comment(line: &str) -> &str {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") || trimmed.is_empty() {
        return "";
    }
    let bytes = line.as_bytes();
    let mut i = 0usize;
    while i + 1 < bytes.len() {
        if bytes[i] == b'/' && bytes[i + 1] == b'/' {
            if i >= 1 && bytes[i - 1] == b':' {
                i += 2;
                continue;
            }
            return line[..i].trim_end();
        }
        i += 1;
    }
    line.trim_end()
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
        let source =
            fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        for (line_no, line) in source.lines().enumerate() {
            let code = dag_line_without_trailing_line_comment(line);
            if code.contains(FN_DEF_NEEDLE) {
                defs.push(format!(
                    "{}:{}: {}",
                    rel_path(&root, &path),
                    line_no + 1,
                    line.trim_end()
                ));
            }
        }
    }

    assert_eq!(
        defs.len(),
        1,
        "P9 single-owner: `{FN_DEF_NEEDLE}` must appear exactly once under src/v4/ \
         (canonical owner `{CANONICAL_OWNER}`); whole-tree parse does not substitute. \
         Found {} definition(s):\n{}",
        defs.len(),
        defs.join("\n")
    );
    assert!(
        defs[0].starts_with(CANONICAL_OWNER),
        "P9 single-owner: `{FN_DEF_NEEDLE}` must live in `{CANONICAL_OWNER}`; got:\n{}",
        defs[0]
    );
}

#[test]
fn v4_p9_llvm_instruction_cost_absent_from_llvm_ir_dag() {
    let root = workspace_root();
    let llvm_ir = root.join("src/v4/extdeps/languages/llvm_ir.dag");
    let source = fs::read_to_string(&llvm_ir)
        .unwrap_or_else(|e| panic!("read {}: {e}", llvm_ir.display()));
    assert!(
        !source.contains(FN_DEF_NEEDLE),
        "P9: `{FN_DEF_NEEDLE}` must not be re-authored in llvm_ir.dag (cost authority is v4.lens.cost only)"
    );
}
