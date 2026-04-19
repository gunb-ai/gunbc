//! SG-1: `tokenize.dag` is load-bearing authority; `tokenize_generated.rs` must stay in sync.

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{FieldValue, TypeConnective, ValueBody};

const TOKENIZE_DAG: &str = include_str!("../../tokenize.dag");
const CHECKED_IN_GENERATED: &str = include_str!("../../src/tokenize_generated.rs");

#[test]
fn tokenize_dag_compiles_cleanly() {
    compile_to_dag(TOKENIZE_DAG, "src/v3/compiler/tokenize.dag")
        .unwrap_or_else(|e| panic!("tokenize.dag should compile: {e:?}"));
}

#[test]
fn tokenize_generated_module_matches_checked_in_snapshot() {
    let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_path = manifest_dir.join("src").join("tokenize_generated.rs");
    let fresh =
        std::process::Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
            .current_dir(&manifest_dir)
            .args(["run", "-q", "-p", "v3-compiler", "--bin", "regen_tokenize"])
            .output()
            .expect("spawn regen_tokenize");
    assert!(
        fresh.status.success(),
        "regen_tokenize failed: {}",
        String::from_utf8_lossy(&fresh.stderr)
    );
    let regen = std::fs::read_to_string(&out_path).expect("read regenerated tokenize_generated.rs");
    assert_eq!(
        CHECKED_IN_GENERATED.trim(),
        regen.trim(),
        "checked-in tokenize_generated.rs is stale; run `cargo run -p v3-compiler --bin regen_tokenize`"
    );
}

#[test]
fn tokenize_registry_rows_use_structural_token_kind_and_derive_punct_width_from_pattern() {
    let dag = compile_to_dag(TOKENIZE_DAG, "src/v3/compiler/tokenize.dag")
        .unwrap_or_else(|e| panic!("tokenize.dag should compile: {e:?}"));

    let token_kind_decl = dag
        .declaration_by_name("TokenKind")
        .expect("TokenKind declaration");
    let TypeConnective::Disj { variants } = &token_kind_decl.connective else {
        panic!("TokenKind should lower to a Disj");
    };

    for decl in dag.declarations() {
        let Some(name) = &decl.name else {
            continue;
        };
        if !name.starts_with("keyword_") && !name.starts_with("punct_") {
            continue;
        }
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            panic!("token row `{name}` should carry a structural body");
        };

        assert!(
            fields.iter().all(|(label, _)| label != "kind_name" && label != "width"),
            "token row `{name}` should not carry string `kind_name` or redundant `width` fields"
        );

        let kind_field = fields
            .iter()
            .find(|(label, _)| label == "kind")
            .unwrap_or_else(|| panic!("token row `{name}` should carry a `kind` field"));
        let FieldValue::Variant {
            constructor,
            payload,
        } = &kind_field.1
        else {
            panic!("token row `{name}` should store `kind` as a structural TokenKind variant");
        };
        assert!(
            payload.is_empty(),
            "token row `{name}` should store only nullary TokenKind variants"
        );
        assert!(
            variants.iter().any(|variant| variant.ty == *constructor),
            "token row `{name}` kind constructor should be a TokenKind variant"
        );
    }
}
