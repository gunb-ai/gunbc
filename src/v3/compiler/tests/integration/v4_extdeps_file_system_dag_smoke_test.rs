//! **Layer:** integration
//!
//! Wave-2-C2 / Practice 11 companion: `src/v4/extdeps/file_system.dag` is a pure
//! external-resource + modeled-effect surface — **no** `import v4.std.node`, **no**
//! `NodeFileBinding`, and no compiler-domain carriers in the resource model.
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching `EXPECTED_HAND_AUTHORED_TEST`
//! line in `sg0_census_test.rs` + INVARIANTS table row land in the same PR.

use v3_compiler::compile_to_dag;
use v3_compiler::CompileError;

const FILE_SYSTEM_DAG: &str = include_str!("../../../../v4/extdeps/file_system.dag");
const FILE_SYSTEM_PATH: &str = "src/v4/extdeps/file_system.dag";

fn file_system_extdeps_dag_or_panic() -> v3_compiler::Dag {
    match compile_to_dag(FILE_SYSTEM_DAG, FILE_SYSTEM_PATH) {
        Ok(dag) => {
            assert!(
                dag.diagnostics().is_empty(),
                "{FILE_SYSTEM_PATH}: expected empty diagnostics, got {:?}",
                dag.diagnostics().iter().collect::<Vec<_>>()
            );
            dag
        }
        Err(CompileError::Semantic(dag)) => panic!(
            "{FILE_SYSTEM_PATH}: semantic errors: {:?}",
            dag.diagnostics().iter().collect::<Vec<_>>()
        ),
        Err(other) => panic!("{FILE_SYSTEM_PATH}: {other:?}"),
    }
}

#[test]
fn v4_extdeps_file_system_dag_compiles() {
    let _dag = file_system_extdeps_dag_or_panic();
}

#[test]
fn v4_extdeps_file_system_dag_practice11_companion_source_has_no_node_import_or_binding() {
    assert!(
        !FILE_SYSTEM_DAG.contains("import v4.std.node"),
        "{FILE_SYSTEM_PATH}: must not import v4.std.node (Practice 11 companion)"
    );
    for forbidden in ["NodeFileBinding", "NodeRef", " Node ", " Node\n", " Node,"] {
        assert!(
            !FILE_SYSTEM_DAG.contains(forbidden),
            "{FILE_SYSTEM_PATH}: forbidden compiler-domain token `{forbidden:?}` in source"
        );
    }
}

#[test]
fn v4_extdeps_file_system_dag_practice11_companion_no_node_file_binding_declaration() {
    let dag = file_system_extdeps_dag_or_panic();
    assert!(
        dag.declaration_by_name("NodeFileBinding").is_none(),
        "{FILE_SYSTEM_PATH}: NodeFileBinding must not be declared in the external-resource model"
    );
    for name in ["Node", "NodeRef"] {
        assert!(
            dag.declaration_by_name(name).is_none(),
            "{FILE_SYSTEM_PATH}: compiler-domain type `{name}` must not be declared here"
        );
    }
}

#[test]
fn v4_extdeps_file_system_dag_wave2_c2_effect_surface_is_canonical() {
    let dag = file_system_extdeps_dag_or_panic();
    for carrier in [
        "FileResource",
        "FilePath",
        "FileContent",
        "FileRead",
        "FileWrite",
        "FileReadWitness",
        "FileWriteWitness",
        "FileEffectWitness",
    ] {
        assert!(
            dag.declaration_by_name(carrier).is_some(),
            "{FILE_SYSTEM_PATH}: expected Wave-2-C2 carrier `{carrier}`"
        );
    }
    assert!(
        dag.declaration_by_name("FileSystemOperations").is_none(),
        "{FILE_SYSTEM_PATH}: FileSystemOperations must not compete with file_read/file_write"
    );
    assert!(
        dag.declaration_by_name("file_read").is_some()
            && dag.declaration_by_name("file_write").is_some(),
        "{FILE_SYSTEM_PATH}: modeled-effect fns file_read and file_write must exist"
    );
}
