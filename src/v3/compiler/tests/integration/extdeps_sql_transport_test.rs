//! **Layer:** integration
//! HTTP/SQL transport extdep authority checks.

use v3_compiler::compile_to_dag;

const REST_TRANSPORT_DAG: &str = include_str!("../../../../../dsl/extdeps/transports/rest.dag");
const SQL_TRANSPORT_DAG: &str = include_str!("../../../../../dsl/extdeps/transports/sql.dag");

#[test]
fn rest_transport_dag_compiles_cleanly() {
    compile_to_dag(REST_TRANSPORT_DAG, "dsl/extdeps/transports/rest.dag")
        .unwrap_or_else(|e| panic!("rest transport extdep should compile: {e:?}"));
}

#[test]
fn sql_transport_dag_compiles_cleanly() {
    compile_to_dag(SQL_TRANSPORT_DAG, "dsl/extdeps/transports/sql.dag")
        .unwrap_or_else(|e| panic!("sql transport extdep should compile: {e:?}"));
}
