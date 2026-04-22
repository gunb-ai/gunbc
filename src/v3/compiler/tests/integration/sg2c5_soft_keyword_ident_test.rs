//! SG-2c-5 — parser-owned soft-keyword name table extraction.

use v3_compiler::parse_surface::{SurfaceItem, VariantPayload};

#[test]
fn parser_accepts_kw_type_as_generated_soft_keyword_name_alias() {
    let source = "type Example = Plain | type { type: Int }\n";
    let tokens = v3_compiler::tokenize_for_test(source, "sg2c5_soft_keyword_ident.v3")
        .expect("tokenize fixture");
    let parsed = v3_compiler::parse_for_test(&tokens, "sg2c5_soft_keyword_ident.v3")
        .expect("parse fixture");

    match &parsed.items[0] {
        SurfaceItem::TypeSum { variants, .. } => {
            assert_eq!(variants.len(), 2, "expected two variants");
            assert_eq!(variants[1].name, "type");
            match &variants[1].payload {
                VariantPayload::Record(fields) => {
                    assert_eq!(fields.len(), 1, "expected single record field");
                    assert_eq!(fields[0].name, "type");
                }
                other => panic!("expected record payload, got {other:?}"),
            }
        }
        other => panic!("expected first item to parse as TypeSum, got {other:?}"),
    }
}
