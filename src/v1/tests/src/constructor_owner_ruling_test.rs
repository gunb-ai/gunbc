//! Constructor-owner ruling (§1c, operator 2026-07-04): resolution follows the
//! binding edge; the constructor namespace is flat (locals + direct imports);
//! ambiguity and unbound constructors are typed errors, never a scan.
//!
//! Red controls: the collision wall (`VariantCollision`) and the unbound
//! constructor literal (`UnresolvedType`) must go red on the bad inputs and
//! stay silent on the legal ones — including the re-export case, where one
//! declaration reached via two import paths is one owner, not a collision.

use crate::helpers::{compile_dag, compile_multi, diagnostic_messages};
use v1_compiler::v1_std_core::CompilerDiagnostic;

#[test]
fn intra_file_arm_collision_is_a_typed_error() {
    let source = "module test.collide\n\
        type Alpha\n\
        \x20 = SharedVariant { x: Int }\n\
        \x20 | AlphaOnly { y: Int }\n\
        type Beta\n\
        \x20 = SharedVariant { x: Int }\n\
        \x20 | BetaOnly { z: Int }\n\
        fn mk() -> Alpha { SharedVariant { x: 1 } }\n";
    let result = compile_dag(source);
    let has_collision = result.diagnostics.iter().any(|diag| {
        matches!(
            &*diag.diagnostic,
            CompilerDiagnostic::VariantCollision { variant, enum1, enum2, .. }
                if variant == "SharedVariant"
                    && ((enum1 == "Alpha" && enum2 == "Beta")
                        || (enum1 == "Beta" && enum2 == "Alpha"))
        )
    });
    assert!(
        has_collision,
        "two local coproducts sharing an arm name must raise VariantCollision, got:\n{}",
        diagnostic_messages(&result).join("\n")
    );
}

#[test]
fn cross_import_arm_collision_is_a_typed_error() {
    let lib_a = "module test.liba\ntype Left = Shared { x: Int } | LeftOnly { y: Int }\n";
    let lib_b = "module test.libb\ntype Right = Shared { x: Int } | RightOnly { y: Int }\n";
    let entry = "module test.entry\n\
        import test.liba { Left }\n\
        import test.libb { Right }\n\
        fn mk() -> Left { Shared { x: 1 } }\n";
    let result = compile_multi(&[
        ("liba.dag", lib_a),
        ("libb.dag", lib_b),
        ("entry.dag", entry),
    ]);
    let has_collision = result.diagnostics.iter().any(|diag| {
        matches!(
            &*diag.diagnostic,
            CompilerDiagnostic::VariantCollision { variant, .. } if variant == "Shared"
        )
    });
    assert!(
        has_collision,
        "one arm name from two imported owners must raise VariantCollision, got:\n{}",
        diagnostic_messages(&result).join("\n")
    );
}

#[test]
fn unbound_constructor_literal_is_unresolved_type() {
    let source = "module test.unbound\n\
        fn mk() -> Int { let g = Ghost { x: 1 } 2 }\n";
    let result = compile_dag(source);
    let has_unresolved = result.diagnostics.iter().any(|diag| {
        matches!(
            &*diag.diagnostic,
            CompilerDiagnostic::UnresolvedType { name, .. } if name == "Ghost"
        )
    });
    assert!(
        has_unresolved,
        "a constructor literal bound nowhere in scope must raise UnresolvedType, got:\n{}",
        diagnostic_messages(&result).join("\n")
    );
}

#[test]
fn imported_enum_arms_construct_via_binding_edge() {
    let lib = "module test.colorlib\ntype Color = Red { shade: Int } | Blue { shade: Int }\n";
    let entry = "module test.usecolor\n\
        import test.colorlib { Color }\n\
        fn mk() -> Color { Red { shade: 3 } }\n";
    let result = compile_multi(&[("colorlib.dag", lib), ("usecolor.dag", entry)]);
    assert!(
        result.diagnostics.is_empty(),
        "importing the enum by name must bind its arms (owner on the binding edge), got:\n{}",
        diagnostic_messages(&result).join("\n")
    );
}

#[test]
fn specifically_imported_arm_constructs_without_enum_name_visible() {
    let lib = "module test.shapelib\ntype Shape = Circle { r: Int } | Square { s: Int }\n";
    let entry = "module test.useshape\n\
        import test.shapelib { Circle, Shape }\n\
        fn mk() -> Shape { Circle { r: 2 } }\n";
    let result = compile_multi(&[("shapelib.dag", lib), ("useshape.dag", entry)]);
    assert!(
        result.diagnostics.is_empty(),
        "importing an arm name directly must bind it to its owner node, got:\n{}",
        diagnostic_messages(&result).join("\n")
    );
}

#[test]
fn glob_import_binds_all_arms() {
    let lib = "module test.toollib\ntype Tool = Hammer { w: Int } | Saw { t: Int }\n";
    // The brace-less import form is the glob (is_all) import.
    let entry = "module test.usetool\n\
        import test.toollib\n\
        fn mk() -> Tool { Hammer { w: 5 } }\n";
    let result = compile_multi(&[("toollib.dag", lib), ("usetool.dag", entry)]);
    assert!(
        result.diagnostics.is_empty(),
        "a glob import must bind every coproduct arm of the module, got:\n{}",
        diagnostic_messages(&result).join("\n")
    );
}

#[test]
fn same_declaration_via_two_import_paths_is_not_a_collision() {
    // test.reexport re-exposes test.baselib's enum; the entry sees the SAME
    // declaration node through both paths — one owner, not a collision.
    let base = "module test.baselib\ntype Fruit = Apple { n: Int } | Pear { n: Int }\n";
    let reexport = "module test.reexport\n\
        import test.baselib { Fruit }\n\
        fn pick() -> Fruit { Apple { n: 1 } }\n";
    let entry = "module test.usefruit\n\
        import test.baselib { Fruit }\n\
        import test.reexport { pick }\n\
        fn mk() -> Fruit { Pear { n: 2 } }\n";
    let result = compile_multi(&[
        ("baselib.dag", base),
        ("reexport.dag", reexport),
        ("usefruit.dag", entry),
    ]);
    assert!(
        result.diagnostics.is_empty(),
        "the same declaration via two import paths is one owner, got:\n{}",
        diagnostic_messages(&result).join("\n")
    );
}
