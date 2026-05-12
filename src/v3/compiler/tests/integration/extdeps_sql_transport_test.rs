//! **Layer:** integration
//! HTTP/SQL/audit extdep authority checks.

use v3_compiler::compile_to_dag;

const HTTP_SERVER_DAG: &str = include_str!("../../../../../dsl/extdeps/http/server.dag");
const REST_TRANSPORT_DAG: &str = include_str!("../../../../../dsl/extdeps/transports/rest.dag");
const SQL_MIGRATION_DAG: &str = include_str!("../../../../../dsl/extdeps/sql/migration.dag");
const SQL_TRANSPORT_DAG: &str = include_str!("../../../../../dsl/extdeps/transports/sql.dag");
const AUDIT_EVENT_DAG: &str = include_str!("../../../../../dsl/extdeps/audit/event.dag");

#[test]
fn http_server_extdep_dag_compiles_cleanly() {
    compile_to_dag(HTTP_SERVER_DAG, "dsl/extdeps/http/server.dag")
        .unwrap_or_else(|e| panic!("http server extdep should compile: {e:?}"));
}

#[test]
fn rest_transport_dag_compiles_cleanly() {
    compile_to_dag(REST_TRANSPORT_DAG, "dsl/extdeps/transports/rest.dag")
        .unwrap_or_else(|e| panic!("rest transport extdep should compile: {e:?}"));
}

#[test]
fn sql_migration_extdep_dag_compiles_cleanly() {
    compile_to_dag(SQL_MIGRATION_DAG, "dsl/extdeps/sql/migration.dag")
        .unwrap_or_else(|e| panic!("sql migration extdep should compile: {e:?}"));
}

#[test]
fn sql_transport_dag_compiles_cleanly() {
    compile_to_dag(SQL_TRANSPORT_DAG, "dsl/extdeps/transports/sql.dag")
        .unwrap_or_else(|e| panic!("sql transport extdep should compile: {e:?}"));
}

#[test]
fn audit_event_extdep_dag_compiles_cleanly() {
    compile_to_dag(AUDIT_EVENT_DAG, "dsl/extdeps/audit/event.dag")
        .unwrap_or_else(|e| panic!("audit event extdep should compile: {e:?}"));
}
