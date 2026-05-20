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

use std::collections::BTreeSet;

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{
    SurfaceField, SurfaceItem, SurfaceType, SurfaceVariant, TypeAngleArg,
};
use v3_compiler::tokenize_for_test;

const FILE_SYSTEM_DAG: &str = include_str!("../../../../v4/extdeps/file_system.dag");
const FILE_SYSTEM_PATH: &str = "src/v4/extdeps/file_system.dag";

fn file_system_surface_or_panic() -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(FILE_SYSTEM_DAG, FILE_SYSTEM_PATH)
        .unwrap_or_else(|e| panic!("{FILE_SYSTEM_PATH}: tokenize: {e:?}"));
    parse_for_test(&tokens, FILE_SYSTEM_PATH)
        .unwrap_or_else(|e| panic!("{FILE_SYSTEM_PATH}: parse: {e:?}"))
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

fn type_sum_variants<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    name: &str,
) -> &'a [SurfaceVariant] {
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

fn variant_name_set(variants: &[SurfaceVariant]) -> BTreeSet<&str> {
    variants
        .iter()
        .map(|variant| variant.name.as_str())
        .collect()
}

fn import_paths(module: &v3_compiler::parse_surface::SurfaceModule) -> Vec<Vec<&str>> {
    module
        .items
        .iter()
        .filter_map(|item| match item {
            SurfaceItem::Import { path, .. } => {
                Some(path.iter().map(String::as_str).collect::<Vec<_>>())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn v4_extdeps_file_system_dag_tokenizes_and_parses() {
    let _module = file_system_surface_or_panic();
}

#[test]
fn v4_extdeps_file_system_dag_practice11_companion_has_no_node_import_or_binding() {
    let module = file_system_surface_or_panic();
    assert!(
        !import_paths(&module)
            .iter()
            .any(|path| path.as_slice() == ["v4", "std", "node"]),
        "{FILE_SYSTEM_PATH}: must not import v4.std.node (Practice 11 companion)"
    );
    for forbidden in ["NodeFileBinding", "Node", "NodeRef"] {
        assert!(
            !surface_declares_type(&module, forbidden),
            "{FILE_SYSTEM_PATH}: must not declare compiler-domain type `{forbidden}`"
        );
    }
}

#[test]
fn v4_extdeps_file_system_dag_file_path_is_posix_grounded_record() {
    let module = file_system_surface_or_panic();
    let fields = type_record_fields(&module, "FilePath");
    assert_eq!(
        record_field_type_map(fields),
        BTreeSet::from([("absolute", "AbsolutePath".to_string())]),
        "FilePath must ground path identity in AbsolutePath (POSIX fact-bundle), not an empty carrier"
    );
}

#[test]
fn v4_extdeps_file_system_dag_wave2_c2_modeled_effects_and_witness_shape() {
    let module = file_system_surface_or_panic();
    let witness_arms = variant_name_set(type_sum_variants(&module, "FileEffectWitness"));
    assert_eq!(
        witness_arms,
        BTreeSet::from(["ReadWitness", "WriteWitness"]),
        "FileEffectWitness must distinguish read vs write receipt arms"
    );

    let read_witness_fields = record_field_type_map(type_record_fields(&module, "FileReadWitness"));
    assert_eq!(
        read_witness_fields,
        BTreeSet::from([
            ("request", "FileRead".to_string()),
            ("content", "FileContent".to_string()),
        ]),
        "FileReadWitness must embed the read request (Practice 11 — no duplicated resource/path fields)"
    );

    let write_witness_fields =
        record_field_type_map(type_record_fields(&module, "FileWriteWitness"));
    assert_eq!(
        write_witness_fields,
        BTreeSet::from([
            ("request", "FileWrite".to_string()),
            ("resource", "FileResource".to_string()),
        ]),
        "FileWriteWitness must thread post-write FileResource (facts-forward write shape)"
    );

    let write_result_alias = module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeAlias {
                name: item_name,
                target,
                ..
            } if item_name == "FileWriteResult" => Some(surface_type_name(target)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type alias FileWriteResult"));
    assert_eq!(
        write_result_alias, "Outcome<FileResource>",
        "file_write must return modified resource per THESIS unenumerated-effects write shape"
    );

    let modeled = record_field_type_map(type_record_fields(&module, "ModeledFileEffects"));
    assert_eq!(
        modeled,
        BTreeSet::from([
            ("file_read", "fn".to_string()),
            ("file_write", "fn".to_string()),
        ]),
        "ModeledFileEffects must be the canonical public read/write effect surface"
    );
}

#[test]
fn v4_extdeps_file_system_dag_legacy_consumer_exports_remain() {
    let module = file_system_surface_or_panic();
    for legacy in [
        "PosixByteString",
        "AbsolutePath",
        "Filesystem",
        "FileBody",
        "FileKindResolutionPolicy",
        "FileSystemOperations",
        "ReadFileRequest",
        "WriteFileRequest",
        "ListDirRequest",
        "FileKindRequest",
    ] {
        assert!(
            surface_declares_type(&module, legacy),
            "{FILE_SYSTEM_PATH}: legacy consumer export `{legacy}` must remain until P5-bridge dissolution"
        );
    }
}
