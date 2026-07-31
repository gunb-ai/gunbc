use crate::helpers::{compile_multi, diagnostic_messages};
use v1_compiler::v1_std_core::{diagnostic_to_message, CompilerDiagnostic};

#[test]
fn std_types_freebie_no_longer_leaks_types_into_non_importers() {
    let std_types = "module std.types\ntype Shape { width: Int }\n";
    let leak_probe = "module leak_probe\ntype Holder { item: Shape }\n";

    let result = compile_multi(&[("std_types.dag", std_types), ("leak_probe.dag", leak_probe)]);

    // Namespace-only resolution (wave-1) supersedes this test's original premise: `Shape`
    // is corpus-globally-unique, so it resolves from anywhere via the census walk — by
    // design, not via the old std.types freebie base-injection (still gone). The interim
    // surface is the UnlistedImportUse ADVISORY naming the unlisted use; there must be no
    // hard UnresolvedType, and no silent resolution either (the advisory is the receipt
    // that resolution came from the census, not an import).
    let unresolved: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| matches!(&*d.diagnostic, CompilerDiagnostic::UnresolvedType { .. }))
        .collect();
    assert!(
        unresolved.is_empty(),
        "globally-unique `Shape` must resolve via the census walk (namespace-only \
         resolution); got hard diagnostics {:?}",
        diagnostic_messages(&result)
    );
    assert!(
        diagnostic_messages(&result)
            .iter()
            .any(|m| m.starts_with("unlisted import use") && m.contains("Shape")),
        "census-resolved unlisted use must carry the UnlistedImportUse advisory naming \
         `Shape`; got {:?}",
        diagnostic_messages(&result)
    );
}

#[test]
fn same_named_sum_type_in_two_modules_resolves_to_own_authority() {
    let a = "module collide_a\ntype T = X | Y\nfn a_make() -> T { X }\n";
    let b = "module collide_b\ntype T = X | Y\nfn b_make() -> T { X }\n";

    let result = compile_multi(&[("collide_a.dag", a), ("collide_b.dag", b)]);

    let hard: Vec<String> = diagnostic_messages(&result)
        .into_iter()
        .filter(|m| !m.starts_with("complexity: ") && !m.starts_with("unlisted import use "))
        .collect();
    assert!(
        hard.is_empty(),
        "two modules defining the same sum type must not collide: {hard:?}"
    );
}

#[test]
fn unresolved_imports_do_not_masquerade_as_circular_dependency() {
    let a = "module false_cycle_a\nimport totally.missing.one\n";
    let b = "module false_cycle_b\nimport totally.missing.two\n";

    let result = compile_multi(&[("false_cycle_a.dag", a), ("false_cycle_b.dag", b)]);

    let circular: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            matches!(
                &*d.diagnostic,
                CompilerDiagnostic::CircularDependency { .. }
            )
        })
        .collect();
    assert!(
        circular.is_empty(),
        "modules blocked only on missing imports must NOT be reported as a \
         circular dependency: {:?}",
        diagnostic_messages(&result)
    );
    let unresolved: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| matches!(&*d.diagnostic, CompilerDiagnostic::UnresolvedImport { .. }))
        .collect();
    assert_eq!(
        unresolved.len(),
        2,
        "both missing imports must surface as UnresolvedImport: {:?}",
        diagnostic_messages(&result)
    );
}

#[test]
fn genuine_import_cycle_still_detected() {
    let a = "module real_cycle_a\nimport real_cycle_b\n";
    let b = "module real_cycle_b\nimport real_cycle_a\n";

    let result = compile_multi(&[("real_cycle_a.dag", a), ("real_cycle_b.dag", b)]);

    let circular: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| {
            matches!(
                &*d.diagnostic,
                CompilerDiagnostic::CircularDependency { .. }
            )
        })
        .collect();
    assert!(
        !circular.is_empty(),
        "a genuine mutual-import cycle must still be detected: {:?}",
        diagnostic_messages(&result)
    );
}

#[test]
fn consumer_resolves_imported_sum_not_foreign_record_of_same_name() {
    let record = "module rec_auth\ntype T { width: Int }\n";
    let sum = "module sum_auth\ntype T = X | Y\n";
    let consumer = "module picks_sum\nimport sum_auth { T, X, Y }\nfn make() -> T { X }\n";

    let result = compile_multi(&[
        ("rec_auth.dag", record),
        ("sum_auth.dag", sum),
        ("picks_sum.dag", consumer),
    ]);

    let variant_errors: Vec<String> = diagnostic_messages(&result)
        .into_iter()
        .filter(|m| m.contains("variant") && m.contains("'X'"))
        .collect();
    assert!(
        variant_errors.is_empty(),
        "`X` must resolve to sum_auth::T, not rec_auth::T: {:?}",
        diagnostic_messages(&result)
    );
}
