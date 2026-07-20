use std::sync::Arc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const TARGET_MODEL: &str = "src/v2/std/compilers/target_model.dag";
const CLAIM_ENTRY: &str = "src/v2/test/claim/manual/rust_add_emit_translate_test.dag";

#[derive(Clone, Copy, PartialEq, Eq)]
enum TargetModelPatch {
    None,
    RuntimeImport,
    MachineByteImport,
    ModelCoreImport,
}

fn patch_target_model(content: &str, patch: TargetModelPatch) -> String {
    let anchor = "module v2.std.compilers.target_model\n";
    let pos = content
        .find(anchor)
        .unwrap_or_else(|| panic!("missing module anchor in {TARGET_MODEL}"));
    let insert = match patch {
        TargetModelPatch::None => return content.to_string(),
        TargetModelPatch::RuntimeImport => {
            "import v2.std.runtime { EffectRequestKind, ReadResource, WriteResource }\n"
        }
        TargetModelPatch::MachineByteImport => "import v2.std.machine { Byte }\n",
        TargetModelPatch::ModelCoreImport => "import v2.std.model_core { ModelCore }\n",
    };
    let mut out = content.to_string();
    out.insert_str(pos + anchor.len(), insert);
    out
}

fn v2_source_roots() -> Vec<std::path::PathBuf> {
    crate::helpers::v2_layer_roots()
}

fn compile_claim_with_target_model_patch(patch: TargetModelPatch) -> (usize, usize, Vec<String>) {
    let entry_content = std::fs::read_to_string(workspace_root().join(CLAIM_ENTRY))
        .unwrap_or_else(|e| panic!("read {CLAIM_ENTRY}: {e}"));
    let mut pairs: Vec<(String, String)> = resolve_imports_transitively_with_source_roots(
        CLAIM_ENTRY,
        &entry_content,
        &v2_source_roots(),
    )
    .iter()
    .map(|s| (s.path.clone(), s.content.clone()))
    .collect();

    if patch != TargetModelPatch::None {
        for (path, content) in &mut pairs {
            if path == TARGET_MODEL {
                *content = patch_target_model(content, patch);
            }
        }
    }

    let sources: Vec<Arc<SourceFile>> = pairs
        .iter()
        .map(|(path, content)| {
            Arc::new(SourceFile {
                path: path.clone(),
                content: content.clone(),
            })
        })
        .collect();
    let source_count = sources.len();
    let result = compile_to_resolved(Arc::new(sources.into()));
    let msgs: Vec<String> = result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
        .collect();
    (source_count, msgs.len(), msgs)
}

fn assert_claim_patch_compiles(patch: TargetModelPatch, label: &str) {
    let (sources, count, msgs) = compile_claim_with_target_model_patch(patch);
    assert!(
        count == 0,
        "{label}: expected clean compile over {sources} sources, got {count} diagnostics: {:?}",
        msgs.iter().take(20).collect::<Vec<_>>()
    );
}

#[test]
fn claim_baseline_compiles() {
    assert_claim_patch_compiles(TargetModelPatch::None, "baseline claim");
}

#[test]
fn claim_with_target_model_runtime_import_compiles() {
    assert_claim_patch_compiles(
        TargetModelPatch::RuntimeImport,
        "claim + target_model runtime import",
    );
}

#[test]
fn claim_with_target_model_machine_import_compiles() {
    assert_claim_patch_compiles(
        TargetModelPatch::MachineByteImport,
        "claim + target_model machine Byte import",
    );
}

#[test]
fn claim_with_target_model_model_core_import_compiles() {
    assert_claim_patch_compiles(
        TargetModelPatch::ModelCoreImport,
        "claim + target_model model_core import",
    );
}
