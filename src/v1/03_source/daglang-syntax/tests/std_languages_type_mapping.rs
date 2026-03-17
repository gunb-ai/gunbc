#![allow(clippy::disallowed_methods)]

use daglang_syntax::ast::{Expr, Item, TypeBody};
use daglang_syntax::parser::parse;
use gunbc_ir::BuiltinType;

fn load_std_languages() -> daglang_syntax::ast::SourceFile {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../dsl/std/languages.dag"
    ))
    .expect("failed to read dsl/std/languages.dag");
    parse(&source).unwrap_or_else(|errors| panic!("failed to parse std.languages: {errors:#?}"))
}

fn find_type_def<'a>(
    ast: &'a daglang_syntax::ast::SourceFile,
    name: &str,
) -> &'a daglang_syntax::ast::TypeDef {
    ast.items
        .iter()
        .find_map(|item| match &item.node {
            Item::TypeDef(type_def) if type_def.name == name => Some(type_def),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type definition `{name}`"))
}

fn find_data_def<'a>(
    ast: &'a daglang_syntax::ast::SourceFile,
    name: &str,
) -> &'a daglang_syntax::ast::DataDef {
    ast.items
        .iter()
        .find_map(|item| match &item.node {
            Item::DataDef(data_def) if data_def.name == name => Some(data_def),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing data definition `{name}`"))
}

fn record_field<'a>(expr: &'a Expr, field_name: &str) -> &'a Expr {
    let Expr::Record(_, fields) = expr else {
        panic!("expected record expression");
    };
    fields
        .iter()
        .find_map(|(name, value)| (name == field_name).then_some(value))
        .unwrap_or_else(|| panic!("missing record field `{field_name}`"))
}

fn primitive_mapping_names(data_def: &daglang_syntax::ast::DataDef) -> Vec<String> {
    let types_expr = record_field(&data_def.value, "types");
    let mappings_expr = record_field(types_expr, "primitive_mappings");
    let Expr::List(entries) = mappings_expr else {
        panic!("expected primitive_mappings list");
    };

    entries
        .iter()
        .map(|entry| {
            let builtin_expr = record_field(entry, "builtin_type");
            let Expr::Literal(daglang_syntax::ast::Literal::String(value)) = builtin_expr else {
                panic!("builtin_type must be a string literal");
            };
            value.clone()
        })
        .collect()
}

#[test]
fn std_languages_type_mapping_uses_keyed_primitive_list() {
    let ast = load_std_languages();
    let type_mapping = find_type_def(&ast, "TypeMapping");
    let TypeBody::Record(fields) = &type_mapping.body else {
        panic!("TypeMapping must stay a record");
    };

    let field_names: std::collections::BTreeSet<_> =
        fields.iter().map(|field| field.name.as_str()).collect();

    assert!(field_names.contains("primitive_mappings"));
    assert!(field_names.contains("list_template"));
    assert!(field_names.contains("optional_template"));
    assert!(field_names.contains("map_template"));

    for legacy_field in ["string", "int", "float", "bool", "bytes", "json"] {
        assert!(
            !field_names.contains(legacy_field),
            "legacy primitive field `{legacy_field}` reintroduced a second authority"
        );
    }
}

#[test]
fn std_languages_data_covers_builtin_target_primitives() {
    let ast = load_std_languages();
    let expected: std::collections::BTreeSet<_> = BuiltinType::all()
        .iter()
        .filter(|builtin| builtin.supports_target_language_primitive_mapping())
        .map(|builtin| builtin.name.to_string())
        .collect();

    for language_data in [
        "rust_language",
        "go_language",
        "python_language",
        "typescript_language",
    ] {
        let actual: std::collections::BTreeSet<_> =
            primitive_mapping_names(find_data_def(&ast, language_data))
                .into_iter()
                .collect();
        assert_eq!(
            actual, expected,
            "{language_data} primitive mappings drifted from builtin type authority"
        );
    }
}
