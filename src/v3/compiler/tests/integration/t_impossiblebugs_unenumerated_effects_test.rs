//! **Layer:** integration
//!
//! Source-level ratchets for the T-ImpossibleBugs unenumerated-effects
//! lens landing. Behavioral execution is owned by the generated-lens
//! migration test once `regen_lens` can run in this environment.

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("expected src/v3/compiler -> workspace root")
        .to_path_buf()
}

fn read_workspace_file(rel: &str) -> String {
    let path = workspace_root().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

#[test]
fn effect_enumeration_lens_anchors_on_signature_shape_not_operation_effect() {
    let lens = read_workspace_file("src/v3/lenses/effect_enumeration.dag");

    assert!(
        lens.contains("callable_signature_effect"),
        "effect_enumeration lens must classify callable transforms from signature shape"
    );
    assert!(
        lens.contains("output_in_inputs"),
        "effect_enumeration lens must recognize returned-modified-resource shape"
    );
    assert!(
        !lens.contains("OperationEffect"),
        "effect_enumeration lens must not re-anchor on the retired OperationEffect taxonomy"
    );
}

#[test]
fn audit_receipt_proves_path_ii_existence_case() {
    let resources = read_workspace_file("dsl/std/resources.dag");
    let primitives = read_workspace_file("dsl/std/primitives.dag");
    let shell = read_workspace_file("dsl/extdeps/shell.dag");
    let github_auth = read_workspace_file("dsl/extdeps/github/auth.dag");

    assert!(
        resources.contains("capability write") && resources.contains("output { written: Bool }"),
        "Filesystem.write remains a non-resource-threaded write-shaped primitive"
    );
    assert!(
        primitives.contains("filesystem_read_contract"),
        "filesystem_read_contract remains an I/O primitive contract outside returned-resource shape"
    );
    assert!(
        shell.contains("operation Run") && shell.contains("transport shell"),
        "shell.Exec.Run remains transport-effectful without resource-threaded signature shape"
    );
    assert!(
        github_auth.contains("uses net: Network"),
        "github_token still derives resource use from ambient `uses`, not its return shape"
    );
}
