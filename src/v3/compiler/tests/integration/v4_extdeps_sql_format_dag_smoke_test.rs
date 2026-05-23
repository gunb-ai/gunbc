//! **Layer:** integration
//!
//! T-4.6 SQL format slice receipt: `src/v4/extdeps/formats/sql.dag` ports the
//! existing v3 SQL migration and transport authorities into the v4 checked
//! Shape-B format home required by T-16.

use std::collections::BTreeSet;

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{
    SurfaceField, SurfaceItem, SurfaceModule, SurfaceType, SurfaceVariant, VariantPayload,
};
use v3_compiler::tokenize_for_test;

const SQL_FORMAT_DAG: &str = include_str!("../../../../v4/extdeps/formats/sql.dag");
const SQL_FORMAT_PATH: &str = "src/v4/extdeps/formats/sql.dag";

fn parse_module(source: &str, file: &str) -> SurfaceModule {
    let tokens = tokenize_for_test(source, file)
        .unwrap_or_else(|diag| panic!("{file}: tokenization failed: {diag:?}"));
    parse_for_test(&tokens, file).unwrap_or_else(|diag| panic!("{file}: parse failed: {diag:?}"))
}

fn type_record<'a>(module: &'a SurfaceModule, name: &str) -> &'a [SurfaceField] {
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

fn type_sum<'a>(module: &'a SurfaceModule, name: &str) -> &'a [SurfaceVariant] {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeSum {
                name: item_name,
                variants,
                ..
            } if item_name == name => Some(variants.as_slice()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type sum {name}"))
}

fn field_type_name(field: &SurfaceField) -> &str {
    match &field.ty {
        SurfaceType::Named { name, .. } => name,
        other => panic!("field `{}` expected named type, got {other:?}", field.name),
    }
}

fn variant_payload_field_names(variant: &SurfaceVariant) -> BTreeSet<&str> {
    match &variant.payload {
        VariantPayload::Record(fields) => fields.iter().map(|field| field.name.as_str()).collect(),
        VariantPayload::Positional(fields) if fields.is_empty() => BTreeSet::new(),
        other => panic!(
            "variant `{}` expected record payload, got {other:?}",
            variant.name
        ),
    }
}

#[test]
fn v4_sql_format_dag_tokenizes_and_parses() {
    let _module = parse_module(SQL_FORMAT_DAG, SQL_FORMAT_PATH);
}

#[test]
fn v4_sql_format_ports_v3_migration_step_artifact_boundary() {
    let module = parse_module(SQL_FORMAT_DAG, SQL_FORMAT_PATH);
    let step_fields = type_record(&module, "SqlMigrationStep");
    let field_names = step_fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        field_names,
        BTreeSet::from(["dialect", "name", "operation", "reversible", "statement"]),
        "SqlMigrationStep must carry the v3 migration authority fields plus the structured v4 operation"
    );

    let statement = step_fields
        .iter()
        .find(|field| field.name == "statement")
        .expect("SqlMigrationStep.statement should be present");
    assert_eq!(
        field_type_name(statement),
        "String",
        "SqlMigrationStep.statement remains the checked emitted artifact boundary"
    );
}

#[test]
fn v4_sql_format_retains_bounded_raw_sql_escape_hatch() {
    let module = parse_module(SQL_FORMAT_DAG, SQL_FORMAT_PATH);
    let schema_ops = type_sum(&module, "SqlSchemaOperation");
    let raw_step = schema_ops
        .iter()
        .find(|variant| variant.name == "SqlRawStep")
        .expect("SqlSchemaOperation must retain the bounded RawSqlStep port");

    assert_eq!(
        variant_payload_field_names(raw_step),
        BTreeSet::from(["statement"]),
        "SqlRawStep should carry only the raw statement text until a structural operation kind is identified"
    );
}

#[test]
fn v4_sql_format_ports_v3_transport_statement_axes() {
    let module = parse_module(SQL_FORMAT_DAG, SQL_FORMAT_PATH);

    let statement_kind_variants = type_sum(&module, "SqlStatementKind")
        .iter()
        .map(|variant| variant.name.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        statement_kind_variants,
        BTreeSet::from([
            "SqlDdl",
            "SqlDelete",
            "SqlInsert",
            "SqlSelect",
            "SqlTransactionControl",
            "SqlUpdate",
        ])
    );

    let parameter_style_variants = type_sum(&module, "SqlParameterStyle")
        .iter()
        .map(|variant| variant.name.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        parameter_style_variants,
        BTreeSet::from([
            "SqlParamNamedAt",
            "SqlParamNamedColon",
            "SqlParamNamedDollar",
            "SqlParamNumberedDollar",
            "SqlParamQuestionMark",
        ])
    );
}
