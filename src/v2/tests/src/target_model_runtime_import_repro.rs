//! Repro for generic-instantiation SCALE failure (adhoc-708ea66d-bb3).
//! Regression fixture: target_model.dag + v4.std.runtime import must compile clean.

use std::rc::Rc;

use v2_compiler::v2_compiler_compile::{compile_to_resolved, SourceFile};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const TARGET_MODEL: &str = "src/v4/std/compilers/target_model.dag";
const RUNTIME_IMPORT: &str =
    "import v4.std.runtime { EffectRequestKind, ReadResource, WriteResource }\n";

fn v4_source_roots() -> Vec<std::path::PathBuf> {
    vec![workspace_root().join("src/v4")]
}

fn compile_target_model_with_optional_runtime_import(with_runtime: bool) -> (usize, Vec<String>) {
    let mut content = std::fs::read_to_string(workspace_root().join(TARGET_MODEL))
        .unwrap_or_else(|e| panic!("read {TARGET_MODEL}: {e}"));
    if with_runtime {
        if let Some(pos) = content.find('\n') {
            content.insert_str(pos + 1, RUNTIME_IMPORT);
        }
    }
    let sources = resolve_imports_transitively_with_source_roots(
        TARGET_MODEL,
        &content,
        &v4_source_roots(),
    );
    let result = compile_to_resolved(Rc::new(sources));
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    (msgs.len(), msgs)
}

#[test]
fn target_model_baseline_compiles() {
    let (count, msgs) = compile_target_model_with_optional_runtime_import(false);
    assert!(
        count == 0,
        "baseline target_model should compile clean, got {count} diagnostics: {msgs:?}"
    );
}

#[test]
fn target_model_with_runtime_import_compiles() {
    let (count, msgs) = compile_target_model_with_optional_runtime_import(true);
    assert!(
        count == 0,
        "target_model + v4.std.runtime import should compile clean, got {count} diagnostics: {:?}",
        msgs.iter().take(15).collect::<Vec<_>>()
    );
}
