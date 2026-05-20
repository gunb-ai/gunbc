//! **Layer:** integration
//!
//! Wave-2-C2 / Practice 11 companion: `src/v4/extdeps/file_system.dag` is a pure
//! external-resource + modeled-effect surface — **no** `import v4.std.node`, **no**
//! `NodeFileBinding`, and no compiler-domain carriers in the resource model.
//!
//! Full `compile_to_dag` on this module alone does not resolve `import v4.std.diagnostic`
//! peers under today's M1(2.7) single-file path (same posture as `v4_bin_main_dag_smoke_test`
//! / `v4_lens_testgen_dag_smoke_test`). This harness uses **tokenize + parse** as the
//! hermetic surface gate; v2-compile over `src/v4` is the integration authority.
//!
//! **PR receipt (P5 Mechanism (b)):** this harness + matching `EXPECTED_HAND_AUTHORED_TEST`
//! line in `sg0_census_test.rs` + INVARIANTS table row land in the same PR.

const FILE_SYSTEM_DAG: &str = include_str!("../../../../v4/extdeps/file_system.dag");
const FILE_SYSTEM_PATH: &str = "src/v4/extdeps/file_system.dag";

fn file_system_surface_or_panic() -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = v3_compiler::tokenize_for_test(FILE_SYSTEM_DAG, FILE_SYSTEM_PATH)
        .unwrap_or_else(|e| panic!("{FILE_SYSTEM_PATH}: tokenize: {e:?}"));
    v3_compiler::parse_for_test(&tokens, FILE_SYSTEM_PATH)
        .unwrap_or_else(|e| panic!("{FILE_SYSTEM_PATH}: parse: {e:?}"))
}

fn surface_declares_type(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    use v3_compiler::parse_surface::SurfaceItem;
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

#[test]
fn v4_extdeps_file_system_dag_tokenizes_and_parses() {
    let _module = file_system_surface_or_panic();
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
    assert!(
        !FILE_SYSTEM_DAG.contains("file_read_not_wired"),
        "{FILE_SYSTEM_PATH}: fail-closed stubs must not use Symbol data without std.node import"
    );
}

#[test]
fn v4_extdeps_file_system_dag_file_path_is_posix_grounded_not_empty() {
    let _module = file_system_surface_or_panic();
    assert!(
        FILE_SYSTEM_DAG.contains("type FilePath") && FILE_SYSTEM_DAG.contains("absolute: AbsolutePath"),
        "{FILE_SYSTEM_PATH}: FilePath must be a POSIX-grounded fact-bundle (absolute: AbsolutePath), not an empty carrier"
    );
    assert!(
        FILE_SYSTEM_DAG.contains("pubs.opengroup.org"),
        "{FILE_SYSTEM_PATH}: FilePath/FileResource path facts must cite POSIX anchor (extdeps fidelity)"
    );
}

#[test]
fn v4_extdeps_file_system_dag_wave2_c2_and_legacy_consumers_coexist() {
    let module = file_system_surface_or_panic();
    for decl in [
        "FileResource",
        "FilePath",
        "FileContent",
        "FileRead",
        "FileWrite",
        "FileReadWitness",
        "FileWriteWitness",
        "FileEffectWitness",
        "ModeledFileEffects",
        "PosixByteString",
        "AbsolutePath",
        "Filesystem",
        "FileBody",
        "FileKindResolutionPolicy",
        "FileSystemOperations",
    ] {
        assert!(
            surface_declares_type(&module, decl),
            "{FILE_SYSTEM_PATH}: expected declaration `{decl}`"
        );
    }
    assert!(
        FILE_SYSTEM_DAG.contains("file_read:") && FILE_SYSTEM_DAG.contains("file_write:"),
        "{FILE_SYSTEM_PATH}: ModeledFileEffects must expose file_read/file_write as canonical modeled-effect surface"
    );
    assert!(
        FILE_SYSTEM_DAG.contains("coproduct dissolution — Wave-2-C2"),
        "{FILE_SYSTEM_PATH}: FileEffectWitness must carry Practice-4 disposition"
    );
}
