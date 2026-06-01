//! **Layer:** integration
//!
//! E0308 W1-A2: `v4.std.text` owns the host-string/text-carrier boundary fact.
//! Literal and value-form migrations can consume this carrier later without
//! minting a second raw-string authority.

use std::collections::BTreeSet;

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceField, SurfaceItem, SurfaceType, TypeAngleArg};
use v3_compiler::tokenize_for_test;

const TEXT_DAG: &str = include_str!("../../../../v4/std/text.dag");
const TEXT_PATH: &str = "src/v4/std/text.dag";

fn text_surface_or_panic() -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(TEXT_DAG, TEXT_PATH)
        .unwrap_or_else(|e| panic!("{TEXT_PATH}: tokenize: {e:?}"));
    parse_for_test(&tokens, TEXT_PATH).unwrap_or_else(|e| panic!("{TEXT_PATH}: parse: {e:?}"))
}

fn surface_declares_type(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| {
        matches!(
            item,
            SurfaceItem::TypeSum { name: decl_name, .. }
                | SurfaceItem::TypeRecord { name: decl_name, .. }
                | SurfaceItem::TypeAlias { name: decl_name, .. }
                | SurfaceItem::TypeAtom { name: decl_name, .. }
                if decl_name == name
        )
    })
}

fn type_record_fields<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> &'a [SurfaceField] {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeRecord {
                name: item_name,
                fields,
                ..
            } if item_name == name => Some(fields.as_slice()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type record {name}"))
}

fn type_alias_target<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> &'a SurfaceType {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeAlias {
                name: item_name,
                target,
                ..
            } if item_name == name => Some(target),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type alias {name}"))
}

fn surface_type_name(ty: &SurfaceType) -> String {
    match ty {
        SurfaceType::Named { name, .. } => name.clone(),
        SurfaceType::Parameterized { name, args, .. } => {
            let rendered = args
                .iter()
                .map(|arg| match arg {
                    TypeAngleArg::TypeExpr { ty } => surface_type_name(ty),
                    TypeAngleArg::WidthNatLiteral { decimal, .. } => decimal.clone(),
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}<{rendered}>")
        }
        SurfaceType::Optional { inner, .. } => format!("?{}", surface_type_name(inner)),
        SurfaceType::Arrow { .. } => "fn".to_string(),
    }
}

fn record_field_type_map(fields: &[SurfaceField]) -> BTreeSet<(&str, String)> {
    fields
        .iter()
        .map(|field| (field.name.as_str(), surface_type_name(&field.ty)))
        .collect()
}

#[test]
fn v4_std_text_dag_tokenizes_and_parses() {
    let _module = text_surface_or_panic();
}

#[test]
fn v4_std_text_dag_string_remains_unicode_free_monoid_carrier() {
    let module = text_surface_or_panic();
    assert_eq!(
        surface_type_name(type_alias_target(&module, "String")),
        "FreeMonoid<Char>",
        "String must remain the Unicode text carrier, not a host/file byte carrier"
    );
}

#[test]
fn v4_std_text_dag_owns_host_string_text_boundary_fact() {
    let module = text_surface_or_panic();
    let fields = type_record_fields(&module, "HostStringText");
    assert_eq!(
        record_field_type_map(fields),
        BTreeSet::from([("text", "String".to_string())]),
        "HostStringText must be the single text-side boundary fact for already-decoded host strings"
    );

    for forbidden in ["ByteString", "FileBody", "FileContent", "TargetSource"] {
        assert!(
            !surface_declares_type(&module, forbidden),
            "{TEXT_PATH}: text module must not redeclare byte/file/target carriers as text"
        );
    }
}
