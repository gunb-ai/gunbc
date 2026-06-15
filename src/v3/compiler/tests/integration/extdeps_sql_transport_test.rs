//! **Layer:** integration
//! HTTP/SQL/audit extdep authority checks.

use crate::common::cached_compile_to_dag;
use std::collections::HashSet;
use v3_compiler::dag::{Dag, DeclarationId, TypeConnective};

const HTTP_SERVER_DAG: &str = include_str!("../../../../../dsl/extdeps/http/server.dag");
const REST_TRANSPORT_DAG: &str = include_str!("../../../../../dsl/extdeps/transports/rest.dag");
const SQL_MIGRATION_DAG: &str = include_str!("../../../../../dsl/extdeps/sql/migration.dag");
const SQL_TRANSPORT_DAG: &str = include_str!("../../../../../dsl/extdeps/transports/sql.dag");
const AUDIT_CLOUDEVENTS_DAG: &str =
    include_str!("../../../../../dsl/extdeps/audit/cloudevents.dag");

fn compile_extdep(source: &str, path: &str) -> Dag {
    cached_compile_to_dag(source, path)
}

fn conj_field_labels(dag: &Dag, name: &str) -> Vec<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing"));
    match &decl.connective {
        TypeConnective::Conj { children } => children.iter().map(|f| f.label.clone()).collect(),
        other => panic!("`{name}` is not a Conj: {other:?}"),
    }
}

fn conj_field_ty(dag: &Dag, owner: &str, label: &str) -> DeclarationId {
    let decl = dag
        .declaration_by_name(owner)
        .unwrap_or_else(|| panic!("`{owner}` missing"));
    let children = match &decl.connective {
        TypeConnective::Conj { children } => children,
        other => panic!("`{owner}` is not a Conj: {other:?}"),
    };
    children
        .iter()
        .find(|f| f.label == label)
        .unwrap_or_else(|| panic!("`{owner}.{label}` missing"))
        .ty
}

fn decl_id_by_name(dag: &Dag, name: &str) -> DeclarationId {
    dag.declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing"))
        .id
}

fn disj_variant_labels(dag: &Dag, name: &str) -> HashSet<String> {
    let decl = dag
        .declaration_by_name(name)
        .unwrap_or_else(|| panic!("`{name}` missing"));
    match &decl.connective {
        TypeConnective::Disj { variants } => variants.iter().map(|f| f.label.clone()).collect(),
        other => panic!("`{name}` is not a Disj: {other:?}"),
    }
}

#[test]
fn http_server_extdep_dag_compiles_cleanly() {
    cached_compile_to_dag(HTTP_SERVER_DAG, "dsl/extdeps/http/server.dag");
}

#[test]
fn http_server_target_fields_are_authoritative_substrate_edges() {
    let dag = compile_extdep(HTTP_SERVER_DAG, "dsl/extdeps/http/server.dag");

    assert_eq!(
        conj_field_ty(&dag, "HttpStatusResponse", "status"),
        decl_id_by_name(&dag, "HttpStatus"),
        "HTTP response status must use the shared HttpStatus authority, not raw Int"
    );
    assert_eq!(
        conj_field_ty(&dag, "HttpServerRoute", "path_template"),
        decl_id_by_name(&dag, "PathTemplate"),
        "HTTP route paths must use std.http_path.PathTemplate, not a parallel string"
    );

    let route_fields: HashSet<String> = conj_field_labels(&dag, "HttpServerRoute")
        .into_iter()
        .collect();
    for field in [
        "operation_name",
        "method",
        "path_template",
        "request_body",
        "parameters",
        "responses",
    ] {
        assert!(
            route_fields.contains(field),
            "HttpServerRoute must carry field-sensitive emission target input `{field}`"
        );
    }
    assert!(
        !route_fields.contains("handler"),
        "HttpServerRoute must stay framework-agnostic — Node handler binding belongs on NodeHttpServerRouteBinding"
    );

    assert_eq!(
        conj_field_ty(&dag, "NodeHttpServerRouteBinding", "route"),
        decl_id_by_name(&dag, "HttpServerRoute"),
        "Node route bindings must reuse the shared HttpServerRoute authority"
    );
    assert_eq!(
        conj_field_ty(&dag, "NodeHttpServerRouteBinding", "listener"),
        decl_id_by_name(&dag, "NodeHttpRequestListener"),
        "Node route bindings must use NodeHttpRequestListener, not a parallel handler authority"
    );

    let chain_fields: HashSet<String> = conj_field_labels(&dag, "NodeHttpCreateServerEffectChain")
        .into_iter()
        .collect();
    for field in ["serve", "listen"] {
        assert!(
            chain_fields.contains(field),
            "Node HTTP server effect chain must structurally encode `{field}` before emission"
        );
    }
    assert_eq!(
        conj_field_ty(&dag, "NodeHttpCreateServerEffectChain", "serve"),
        decl_id_by_name(&dag, "NodeHttpCreateServerServe"),
        "Node serve step must use NodeHttpCreateServerServe"
    );
    assert_eq!(
        conj_field_ty(&dag, "NodeHttpCreateServerEffectChain", "listen"),
        decl_id_by_name(&dag, "HttpServerListenConfig"),
        "Node listen step must use HttpServerListenConfig"
    );
    assert_eq!(
        conj_field_ty(&dag, "NodeHttpCreateServerEmissionTarget", "effects"),
        decl_id_by_name(&dag, "NodeHttpCreateServerEffectChain"),
        "Node HTTP server emission must project through the fixed serve→listen chain"
    );
}

#[test]
fn rest_transport_dag_compiles_cleanly() {
    cached_compile_to_dag(REST_TRANSPORT_DAG, "dsl/extdeps/transports/rest.dag");
}

#[test]
fn sql_migration_extdep_dag_compiles_cleanly() {
    cached_compile_to_dag(SQL_MIGRATION_DAG, "dsl/extdeps/sql/migration.dag");
}

#[test]
fn sql_migration_target_fields_bound_raw_sql_scaffold() {
    let dag = compile_extdep(SQL_MIGRATION_DAG, "dsl/extdeps/sql/migration.dag");

    let variants = disj_variant_labels(&dag, "SqlMigrationOperationKind");
    assert!(
        variants.contains("RawSqlStep"),
        "RawSqlStep is the explicitly bounded dialect-specific scaffold variant"
    );

    let step_fields: HashSet<String> = conj_field_labels(&dag, "SqlMigrationStep")
        .into_iter()
        .collect();
    for field in ["name", "operation", "dialect", "statement", "reversible"] {
        assert!(
            step_fields.contains(field),
            "SqlMigrationStep must carry field-sensitive migration target input `{field}`"
        );
    }
}

#[test]
fn sql_transport_dag_compiles_cleanly() {
    cached_compile_to_dag(SQL_TRANSPORT_DAG, "dsl/extdeps/transports/sql.dag");
}

#[test]
fn audit_cloudevents_extdep_dag_compiles_cleanly() {
    cached_compile_to_dag(AUDIT_CLOUDEVENTS_DAG, "dsl/extdeps/audit/cloudevents.dag");
}

#[test]
fn audit_cloudevents_target_fields_preserve_core_names() {
    let dag = compile_extdep(AUDIT_CLOUDEVENTS_DAG, "dsl/extdeps/audit/cloudevents.dag");

    for (field, ty) in [
        ("id", "CloudEventId"),
        ("source", "CloudEventSource"),
        ("specversion", "CloudEventSpecVersion"),
        ("type", "CloudEventType"),
        ("subject", "CloudEventSubject"),
        ("time", "Timestamp"),
    ] {
        assert_eq!(
            conj_field_ty(&dag, "AuditEventRecord", field),
            decl_id_by_name(&dag, ty),
            "AuditEventRecord.{field} must use the shared CloudEvents/std carrier `{ty}`"
        );
    }
    assert_eq!(
        conj_field_ty(&dag, "AuditEventField", "key"),
        decl_id_by_name(&dag, "CloudEventExtensionName"),
        "CloudEvents extension keys must use the branded extension-name carrier"
    );
    assert_eq!(
        conj_field_ty(&dag, "AuditEventField", "value"),
        decl_id_by_name(&dag, "CloudEventExtensionValue"),
        "CloudEvents extension values must use the branded extension-value carrier"
    );

    let event_fields: HashSet<String> = conj_field_labels(&dag, "AuditEventRecord")
        .into_iter()
        .collect();
    for field in [
        "id",
        "source",
        "specversion",
        "type",
        "subject",
        "actor",
        "outcome",
        "time",
        "fields",
    ] {
        assert!(
            event_fields.contains(field),
            "AuditEventRecord must preserve CloudEvents/audit field `{field}`"
        );
    }
    assert!(
        !event_fields.contains("event_type") && !event_fields.contains("occurred_at"),
        "AuditEventRecord must not regress to ctrl-local aliases for CloudEvents core fields"
    );
}
