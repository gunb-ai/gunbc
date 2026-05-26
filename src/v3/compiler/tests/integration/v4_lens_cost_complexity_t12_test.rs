//! **Layer:** integration
//!
//! T-12 receipt for `src/v4/lens/cost.dag` + `src/v4/lens/complexity.dag`.
//! The full v4 source-root compile checks lowering, but this ratchet pins the
//! review contract that matters for the lens substrate: cost owns the symbolic
//! carrier and complexity consumes it as an asymptotic projection, with no
//! second cost authority in `complexity.dag`.

use v3_compiler::parse_for_test;
use v3_compiler::parse_surface::{SurfaceItem, SurfaceVariant};
use v3_compiler::tokenize_for_test;

const COST_DAG: &str = include_str!("../../../../v4/lens/cost.dag");
const COST_PATH: &str = "src/v4/lens/cost.dag";
const COMPLEXITY_DAG: &str = include_str!("../../../../v4/lens/complexity.dag");
const COMPLEXITY_PATH: &str = "src/v4/lens/complexity.dag";

fn parse_module(source: &str, path: &str) -> v3_compiler::parse_surface::SurfaceModule {
    let tokens =
        tokenize_for_test(source, path).unwrap_or_else(|e| panic!("{path}: tokenize: {e:?}"));
    parse_for_test(&tokens, path).unwrap_or_else(|e| panic!("{path}: parse: {e:?}"))
}

fn module_path_is(module: &v3_compiler::parse_surface::SurfaceModule, path: &[&str]) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Module {
            path: item_path, ..
        } => item_path
            .iter()
            .map(String::as_str)
            .eq(path.iter().copied()),
        _ => false,
    })
}

fn import_includes_name(
    module: &v3_compiler::parse_surface::SurfaceModule,
    path: &[&str],
    name: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Import {
            path: item_path,
            names,
            ..
        } => {
            item_path
                .iter()
                .map(String::as_str)
                .eq(path.iter().copied())
                && names.iter().any(|n| n == name)
        }
        _ => false,
    })
}

fn surface_declares_fn(module: &v3_compiler::parse_surface::SurfaceModule, name: &str) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::Fn {
            name: item_name, ..
        }
        | SurfaceItem::FnExternalBody {
            name: item_name, ..
        } => item_name == name,
        _ => false,
    })
}

fn surface_declares_type(
    module: &v3_compiler::parse_surface::SurfaceModule,
    type_name: &str,
) -> bool {
    module.items.iter().any(|item| match item {
        SurfaceItem::TypeAlias { name, .. }
        | SurfaceItem::TypeRecord { name, .. }
        | SurfaceItem::TypeSum { name, .. } => name == type_name,
        _ => false,
    })
}

fn type_sum_variants<'a>(
    module: &'a v3_compiler::parse_surface::SurfaceModule,
    type_name: &str,
) -> Vec<&'a SurfaceVariant> {
    module
        .items
        .iter()
        .find_map(|item| match item {
            SurfaceItem::TypeSum { name, variants, .. } if name == type_name => {
                Some(variants.iter().collect())
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("missing type sum `{type_name}`"))
}

fn has_variant(
    module: &v3_compiler::parse_surface::SurfaceModule,
    type_name: &str,
    variant: &str,
) -> bool {
    type_sum_variants(module, type_name)
        .iter()
        .any(|v| v.name == variant)
}

#[test]
fn v4_t12_cost_owns_symbolic_cost_and_asymptotic_class() {
    let cost = parse_module(COST_DAG, COST_PATH);
    assert!(module_path_is(&cost, &["v4", "lens", "cost"]));
    assert!(surface_declares_fn(&cost, "cost_lens"));
    assert!(surface_declares_fn(&cost, "asymptotic_class_of_cost"));
    assert!(surface_declares_type(&cost, "SymbolicCost"));
    assert!(surface_declares_type(&cost, "AsymptoticClass"));
    assert!(surface_declares_type(&cost, "SizeVariable"));

    for variant in [
        "ConstantCost",
        "LinearCost",
        "LogCost",
        "PolynomialCost",
        "PolyLogCost",
        "ExponentialCost",
        "FactorialCost",
        "SumCost",
        "ProductCost",
        "UnknownCost",
    ] {
        assert!(
            has_variant(&cost, "SymbolicCost", variant),
            "T-12 SymbolicCost carrier missing variant `{variant}`"
        );
    }

    for variant in [
        "ClassConstant",
        "ClassLog",
        "ClassLinear",
        "ClassLinearithmic",
        "ClassPolynomial",
        "ClassPolyLog",
        "ClassExponential",
        "ClassFactorial",
        "ClassUnknown",
    ] {
        assert!(
            has_variant(&cost, "AsymptoticClass", variant),
            "T-12 AsymptoticClass carrier missing variant `{variant}`"
        );
    }
}

#[test]
fn v4_t12_complexity_consumes_cost_projection_without_redeclaring_cost() {
    let complexity = parse_module(COMPLEXITY_DAG, COMPLEXITY_PATH);
    assert!(module_path_is(&complexity, &["v4", "lens", "complexity"]));
    assert!(surface_declares_fn(&complexity, "complexity_lens"));
    assert!(surface_declares_fn(&complexity, "asymptotic_projection"));
    assert!(surface_declares_type(&complexity, "ComplexityBound"));

    for imported in [
        "SymbolicCost",
        "SizeVariable",
        "AsymptoticClass",
        "asymptotic_class_of_cost",
        "asymptotic_class_dominates",
        "cost_lens",
    ] {
        assert!(
            import_includes_name(&complexity, &["v4", "lens", "cost"], imported),
            "complexity.dag must import `{imported}` from v4.lens.cost"
        );
    }

    for forbidden in ["SymbolicCost", "AsymptoticClass", "SizeVariable"] {
        assert!(
            !surface_declares_type(&complexity, forbidden),
            "complexity.dag must not redeclare `{forbidden}`; cost.dag is the T-12 authority"
        );
    }

    for variant in ["ConstantComplexity", "SizedComplexity", "UnknownComplexity"] {
        assert!(
            has_variant(&complexity, "ComplexityBound", variant),
            "T-12 ComplexityBound carrier missing variant `{variant}`"
        );
    }
}
