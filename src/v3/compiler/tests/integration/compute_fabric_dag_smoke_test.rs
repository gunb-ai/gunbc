//! **Layer:** integration
//!
//! P-CF-TYPE receipt: `dsl/std/compute_fabric.dag` tokenizes and parses cleanly
//! (Worksheet A §2 parser gate).

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceItem, SurfaceType};
use v3_compiler::tokenize_for_test;

const COMPUTE_FABRIC_DAG: &str = include_str!("../../../../../dsl/std/compute_fabric.dag");
const COMPUTE_FABRIC_PATH: &str = "dsl/std/compute_fabric.dag";

fn compute_fabric_surface_or_panic() -> v3_compiler::parse_surface::SurfaceModule {
    let tokens = tokenize_for_test(COMPUTE_FABRIC_DAG, COMPUTE_FABRIC_PATH)
        .unwrap_or_else(|e| panic!("{COMPUTE_FABRIC_PATH}: tokenize: {e:?}"));
    parse_for_test(&tokens, COMPUTE_FABRIC_PATH)
        .unwrap_or_else(|e| panic!("{COMPUTE_FABRIC_PATH}: parse: {e:?}"))
}

fn surface_declares_type(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| {
        matches!(
            item,
            SurfaceItem::TypeSum { name: decl_name, .. }
                | SurfaceItem::TypeRecord { name: decl_name, .. }
                | SurfaceItem::TypeAlias { name: decl_name, .. }
                if decl_name == name
        )
    })
}

fn surface_declares_data(
    module: &v3_compiler::parse_surface::SurfaceModule,
    name: &str,
    ty_name: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Data {
            name: decl_name,
            ty,
            ..
        } => decl_name == name && surface_type_name(ty) == ty_name,
        _ => false,
    })
}

fn surface_type_name(ty: &SurfaceType) -> String {
    match ty {
        SurfaceType::Named { name, .. } => name.clone(),
        SurfaceType::Parameterized { name, args, .. } => {
            let rendered = args
                .iter()
                .map(|arg| match arg {
                    v3_compiler::parse_surface::TypeAngleArg::TypeExpr { ty } => {
                        surface_type_name(ty)
                    }
                    v3_compiler::parse_surface::TypeAngleArg::WidthNatLiteral { decimal, .. } => {
                        decimal.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}<{rendered}>")
        }
        SurfaceType::Optional { inner, .. } => format!("{}?", surface_type_name(inner)),
        SurfaceType::Arrow { .. } => "fn".to_string(),
    }
}

#[test]
fn compute_fabric_dag_parses() {
    let _module = compute_fabric_surface_or_panic();
}

#[test]
fn compute_fabric_declares_worksheet_a_core_types() {
    let module = compute_fabric_surface_or_panic();
    for name in [
        "WorkDemand",
        "ComputeSupplyFacts",
        "ComputeArtifactLocality",
        "ExecutionReceipt",
        "PerformanceReceipt",
        "MeasurementConfidence",
    ] {
        assert!(
            surface_declares_type(&module, name),
            "missing type {name}"
        );
    }
}

#[test]
fn compute_fabric_falsification_supply_rows() {
    let module = compute_fabric_surface_or_panic();
    for (data_name, ty_name) in [
        ("supply_srv1", "ComputeSupplyFacts"),
        ("supply_srv2", "ComputeSupplyFacts"),
        ("supply_wsl", "ComputeSupplyFacts"),
        ("supply_gcloud_container", "ComputeSupplyFacts"),
    ] {
        assert!(
            surface_declares_data(&module, data_name, ty_name),
            "missing data {data_name}: {ty_name}"
        );
    }
}
