//! **Layer:** integration
//! SQL transport extdep authority checks.

use v3_compiler::compile_to_dag;

const SQL_TRANSPORT_DAG: &str =
    include_str!("../../../../../dsl/extdeps/transports/sql.dag");

#[test]
fn sql_transport_dag_compiles_cleanly() {
    compile_to_dag(SQL_TRANSPORT_DAG, "dsl/extdeps/transports/sql.dag")
        .unwrap_or_else(|e| panic!("sql transport extdep should compile: {e:?}"));
}

