//! Module-qualified type/constructor authority (§3 single-authority).
//!
//! Two modules may define the same type name (e.g. v1 `std.algebra`'s
//! `FreeMonoid` record and v2 `v2.std.algebra`'s `FreeMonoid = Empty | Cons`
//! sum). A name must resolve to exactly the definition the importing module's
//! import graph selects — never to a foreign same-named type pulled in by an
//! implicit base injection or by topological merge order.
//!
//! These witnesses pin the resolver fix that removed the `std.types`
//! base-injection bootstrap bridge (the second, implicit authority). They go
//! RED if that leak — or order-decided last-writer-wins resolution — regresses.

use crate::helpers::{compile_multi, diagnostic_messages};
use v1_compiler::v1_std_core::{diagnostic_to_message, CompilerDiagnostic};

/// Strong discriminator: a module that never imports `std.types` (nor the
/// module that actually declares a type) must NOT inherit that type. Pre-fix,
/// the resolver injected `std.types`' entire transitive env as an implicit base
/// into every non-importing module, so `Shape` resolved for free — a §3
/// violation. Post-fix the reference is a hard `UnresolvedType`.
#[test]
fn std_types_freebie_no_longer_leaks_types_into_non_importers() {
    // `std.types` declares `Shape`; `leak_probe` never imports `std.types`.
    // Pre-fix the resolver injected std.types' whole env as an implicit base,
    // so `Shape` resolved for free (no diagnostic). Post-fix it is unresolved.
    let std_types = "module std.types\ntype Shape { width: Int }\n";
    let leak_probe = "module leak_probe\ntype Holder { item: Shape }\n";

    let result = compile_multi(&[("std_types.dag", std_types), ("leak_probe.dag", leak_probe)]);

    let unresolved: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| matches!(&*d.diagnostic, CompilerDiagnostic::UnresolvedType { .. }))
        .collect();
    assert!(
        !unresolved.is_empty(),
        "leak_probe must NOT inherit `Shape` from the std.types freebie \
         (the implicit base-injection is gone); got diagnostics {:?}",
        diagnostic_messages(&result)
    );
    let msg = diagnostic_to_message(unresolved[0].diagnostic.clone());
    assert!(
        msg.contains("Shape"),
        "the unresolved type should name `Shape`: {msg}"
    );
}

/// The brief's two-module witness: modules `a` and `b` each define
/// `type T = X | Y`; each module's own `X` must bind to its own `T`. Per-module
/// envs already isolate them — this guards against a regression that would
/// flatten same-named definitions into one bare-interned authority.
#[test]
fn same_named_sum_type_in_two_modules_resolves_to_own_authority() {
    let a = "module collide_a\ntype T = X | Y\nfn a_make() -> T { X }\n";
    let b = "module collide_b\ntype T = X | Y\nfn b_make() -> T { X }\n";

    let result = compile_multi(&[("collide_a.dag", a), ("collide_b.dag", b)]);

    let hard: Vec<String> = diagnostic_messages(&result)
        .into_iter()
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        hard.is_empty(),
        "two modules defining the same sum type must not collide: {hard:?}"
    );
}

/// §5 truthful-diagnostic guard: modules whose ONLY problem is an unresolved
/// import must report `UnresolvedImport`, never a fabricated `CircularDependency`.
///
/// Pre-fix, `topological_sort` counted EVERY import toward a module's in-degree
/// (including imports of modules absent from the resolve set), while the
/// adjacency only carried in-set edges. So a module blocked solely on a missing
/// import never reached in-degree 0, never drained, and the leftover set was
/// reported as a circular dependency — a §5 fabricated wrong answer masking the
/// real error. `a` and `b` here import only *missing* modules and never each
/// other; the diagnosis must be unresolved-import, with no false cycle.
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

/// The discriminating control for the test above: a *genuine* mutual import
/// cycle (a→b→a, both in-set) must STILL be detected as `CircularDependency`.
/// This goes RED if the in-degree fix over-corrected and dropped real cycles.
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

/// §3 cross-authority guard: a record `T` and a sum `T` coexist in the compile
/// set (the shape of the v1-record vs v2-sum `FreeMonoid` collision). A consumer
/// that imports the SUM must resolve its variant to the sum — never to the
/// foreign record of the same name.
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

    // If the foreign record `T` won, `X` would be reported as a variant not
    // found in the (record) `T`.
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
