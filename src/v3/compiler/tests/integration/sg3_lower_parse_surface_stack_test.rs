//! **Layer:** integration
//!
//! SG-3f parse/parse_surface convergence guard: lowering still reads `parse::Surface*`,
//! but `parse_surface` now names the same Rust carrier family. This pins that fixtures
//! exercised through the full compile boundary remain consumable through the public
//! `parse_surface` namespace with no bridging clone path in between.

use v3_compiler::operators::{ArithmeticOp, OperatorKind};
use v3_compiler::parse_surface;
use v3_compiler::{
    compile_to_dag, default_fixed_point_source, parse_for_test,
    surface_top_level_let_names_for_test, tokenize_for_test,
};

fn top_level_let_names_mirror(m: &parse_surface::SurfaceModule) -> Vec<String> {
    m.items
        .iter()
        .filter_map(|item| match item {
            parse_surface::SurfaceItem::Let { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect()
}

fn type_named(ty: &Option<parse_surface::SurfaceType>, want: &str) -> bool {
    matches!(
        ty,
        Some(parse_surface::SurfaceType::Named { name, .. }) if name == want
    )
}

fn literal_int(expr: &parse_surface::SurfaceExpr) -> Option<i64> {
    match expr {
        parse_surface::SurfaceExpr::Literal {
            value: parse_surface::SurfaceLiteral::Int(n),
            ..
        } => i64::try_from(*n).ok(),
        _ => None,
    }
}

/// Shape checks for `default_fixed_point_source()` on the **mirrored** surface.
fn assert_default_fixed_point_mirror_shape(m: &parse_surface::SurfaceModule) {
    assert_eq!(m.items.len(), 2);

    match &m.items[0] {
        parse_surface::SurfaceItem::Let {
            name,
            type_ann,
            expr,
        } => {
            assert_eq!(name, "x");
            assert!(type_named(type_ann, "Int"));
            match expr {
                parse_surface::SurfaceExpr::Operator { op, args, .. } => {
                    assert_eq!(*op, OperatorKind::Arithmetic(ArithmeticOp::Add));
                    assert_eq!(args.len(), 2);
                    assert_eq!(literal_int(&args[0]), Some(1));
                    assert_eq!(literal_int(&args[1]), Some(2));
                }
                other => panic!("expected `1 + 2` operator, got {other:?}"),
            }
        }
        other => panic!("expected first let, got {other:?}"),
    }

    match &m.items[1] {
        parse_surface::SurfaceItem::Let {
            name,
            type_ann,
            expr,
        } => {
            assert_eq!(name, "y");
            assert!(type_named(type_ann, "Int"));
            match expr {
                parse_surface::SurfaceExpr::Operator { op, args, .. } => {
                    assert_eq!(*op, OperatorKind::Arithmetic(ArithmeticOp::Add));
                    assert_eq!(args.len(), 2);
                    assert!(matches!(
                        &args[0],
                        parse_surface::SurfaceExpr::Var { name, .. } if name == "x"
                    ));
                    assert_eq!(literal_int(&args[1]), Some(3));
                }
                other => panic!("expected `x + 3` operator, got {other:?}"),
            }
        }
        other => panic!("expected second let, got {other:?}"),
    }
}

#[test]
fn lower_pipeline_fixture_aligns_with_parse_surface_mirror() {
    let file = "sg3_lower_parse_surface_stack.v3";
    let source = default_fixed_point_source();
    let tokens = tokenize_for_test(source, file).expect("tokenize");
    let parsed = parse_for_test(&tokens, file).expect("parse");
    let mirrored: &parse_surface::SurfaceModule = &parsed;

    assert_eq!(
        mirrored.items.len(),
        parsed.items.len(),
        "parse_surface mirror must preserve top-level item count"
    );

    let hand_lets = surface_top_level_let_names_for_test(&parsed);
    let mirror_lets = top_level_let_names_mirror(mirrored);
    assert_eq!(hand_lets, mirror_lets);
    assert_eq!(hand_lets, vec!["x".to_string(), "y".to_string()]);

    assert_default_fixed_point_mirror_shape(mirrored);

    compile_to_dag(source, file).expect("lower + infer should succeed on the same surface");
}
