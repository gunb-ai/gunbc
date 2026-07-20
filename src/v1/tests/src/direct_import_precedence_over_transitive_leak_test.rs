//! Discriminating receipt for direct-import export precedence
//! (overlay_direct_import_exports, 04_infer.dag: direct_import_export_precedence_note).
//!
//! The class (the 2026-07-16 #6663 x #6686 main-red): the ancestry union folds each
//! direct import's WHOLE flattened cache, so a later import's transitively-leaked
//! homonym could overlay an earlier import's OWN export that this module's import
//! statement explicitly selects. `import v2.std.algebra { Monoid }` lost `Monoid` to
//! dag-root `std.algebra` riding `v2.std.node`'s ancestry, and the field-presence
//! wall then demanded the wrong shape's fields at 28 sites.
//!
//! Green arms: the direct-selected declaration wins regardless of import order
//! (selective and is_all). Red control: the presence wall still refuses missing
//! fields of the TRUE shape — an env fix that neutered the wall would pass green
//! arms silently and fail here. Ledger arm: the union conflict row is still
//! recorded (the fix corrects which binding serves lookups; it does not hide forks).
//!
//! Module paths are load-bearing: realleaf sits under the consumer's OWN parent
//! (forkc.app) and leakleaf under a sibling parent (forkc.other), so nearest-ancestor
//! containment resolves `Widget` UNIQUELY and the census gate
//! (presence_check_census_gate_note) does not stand the wall down. Flatten them to a
//! common parent and the red control silently stops testing the overlay: the name goes
//! census-ambiguous, the gate suppresses the diagnostic by design, and the arm fails
//! for a reason that has nothing to do with import precedence.

use std::rc::Rc;

use crate::helpers::{compile_multi, diagnostic_messages};
use v1_compiler::v1_compiler_compile::{front_end_sources, normalize_graph, SourceFile};
use v1_compiler::v1_compiler_infer::{
    build_variant_export_surface, is_error_diagnostic, typecheck_module, TypecheckModuleResult,
    VariantExportSurface,
};
use v1_compiler::v1_compiler_infer_items::TypedModule;
use v1_compiler::v1_rt;
use v1_compiler::v1_std_core::{authored_name_at, InternTable, NewlineIndex};

// The shape the consumer's import statement selects: Widget { semigroup }.
const REAL_LEAF: &str = "module forkc.app.realleaf\n\
    type Widget<T> { semigroup: T }\n";

// A same-named homonym with a DIFFERENT shape: Widget { op }.
const LEAK_LEAF: &str = "module forkc.other.leakleaf\n\
    type Widget<T> { op: T }\n";

// The carrier: imports the homonym, so its exported flattened cache LEAKS
// leakleaf's Widget into every downstream union (the v2.std.node role).
const LEAK_MID: &str = "module forkc.other.leakmid\n\
    import forkc.other.leakleaf { Widget }\n\
    fn mk_leak() -> Widget<Int> { Widget { op: 1 } }\n";

fn error_messages(result: &v1_compiler::v1_compiler_compile::PipelineResult) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| format!("{:?}", d.diagnostic))
        .collect()
}

// GREEN arm (selective): the consumer imports realleaf's Widget BY NAME first and
// leakmid AFTER — pre-fix, overlay-wins handed `Widget` to the leak and the wall
// demanded `op`; the direct-selected export must win instead.
#[test]
fn direct_selected_export_beats_later_transitive_leak() {
    let entry = "module forkc.app.consumer\n\
        import forkc.app.realleaf { Widget }\n\
        import forkc.other.leakmid { mk_leak }\n\
        fn mk_real() -> Widget<Int> { Widget { semigroup: 3 } }\n";
    let result = compile_multi(&[
        ("realleaf.dag", REAL_LEAF),
        ("leakleaf.dag", LEAK_LEAF),
        ("leakmid.dag", LEAK_MID),
        ("consumer.dag", entry),
    ]);
    let errors = error_messages(&result);
    assert!(
        errors.is_empty(),
        "a literal of the direct-selected Widget shape must typecheck against the \
         SELECTED declaration (semigroup), not a later import's transitive leak (op). \
         Errors:\n{}\nAll diagnostics:\n{}",
        errors.join("\n"),
        diagnostic_messages(&result).join("\n")
    );
}

// GREEN arm (is_all): same precedence when the direct import is unselective.
#[test]
fn direct_is_all_export_beats_later_transitive_leak() {
    let entry = "module forkc.app.consumer\n\
        import forkc.app.realleaf\n\
        import forkc.other.leakmid { mk_leak }\n\
        fn mk_real() -> Widget<Int> { Widget { semigroup: 3 } }\n";
    let result = compile_multi(&[
        ("realleaf.dag", REAL_LEAF),
        ("leakleaf.dag", LEAK_LEAF),
        ("leakmid.dag", LEAK_MID),
        ("consumer.dag", entry),
    ]);
    let errors = error_messages(&result);
    assert!(
        errors.is_empty(),
        "an is_all direct import's own export must also win over a later import's \
         transitive leak. Errors:\n{}\nAll diagnostics:\n{}",
        errors.join("\n"),
        diagnostic_messages(&result).join("\n")
    );
}

// RED control: with the direct import winning, the field-presence wall must still
// refuse a literal missing the TRUE shape's field — and name `semigroup`, not `op`.
// This is what makes the green arms discriminating: an env change that blinded the
// wall (or left the leak winning) cannot pass both.
#[test]
fn presence_wall_still_refuses_true_shape_missing_field() {
    let entry = "module forkc.app.consumer\n\
        import forkc.app.realleaf { Widget }\n\
        import forkc.other.leakmid { mk_leak }\n\
        fn mk_missing() -> Widget<Int> { Widget { } }\n";
    let result = compile_multi(&[
        ("realleaf.dag", REAL_LEAF),
        ("leakleaf.dag", LEAK_LEAF),
        ("leakmid.dag", LEAK_MID),
        ("consumer.dag", entry),
    ]);
    let errors = error_messages(&result);
    assert!(
        errors.iter().any(|e| e.contains("semigroup")),
        "an empty Widget literal must red on the SELECTED shape's missing field \
         'semigroup' (wall live, correct winner). Errors:\n{}\nAll diagnostics:\n{}",
        errors.join("\n"),
        diagnostic_messages(&result).join("\n")
    );
    assert!(
        !errors.iter().any(|e| e.contains("'op'")),
        "the leak's shape must not serve the wall: an error naming 'op' means the \
         transitive homonym still wins. Errors:\n{}",
        errors.join("\n")
    );
}

// Kernel-layer arm: the overlay must NOT let a direct import shadow a KERNEL name
// (kernel_type_set + containers + Optional/Present/Absent) — the kernel scope layer
// stays positionally above imports (2026-07-11 ruling; builtins type against kernel
// identities, and the v2 substrate models of these concepts are the known
// dual-representation interim resolved corpus-wide by kernel-wins). A consumer that
// imports a String homonym still types string literals as kernel String.
#[test]
fn kernel_names_are_not_overridden_by_direct_imports() {
    let str_leaf = "module forkc.strleaf\n\
        type String { chars: Int }\n";
    let entry = "module forkc.app.consumer\n\
        import forkc.strleaf { String }\n\
        fn lit() -> String { \"kernel\" }\n";
    let result = compile_multi(&[("strleaf.dag", str_leaf), ("consumer.dag", entry)]);
    let errors = error_messages(&result);
    assert!(
        errors.is_empty(),
        "a string literal must still type as KERNEL String even when a String \
         homonym is directly imported (kernel positional layer above imports). \
         Errors:\n{}\nAll diagnostics:\n{}",
        errors.join("\n"),
        diagnostic_messages(&result).join("\n")
    );
}

// Ledger arm: the precedence overlay corrects which binding serves lookups; the
// union's conflict LEDGER (binding_forks channel, 2026-07-11 ruling) still records
// the Widget fork — the fix must not zero the fork's observability.
#[test]
fn fork_ledger_still_records_the_leak_conflict() {
    let entry = "module forkc.app.consumer\n\
        import forkc.app.realleaf { Widget }\n\
        import forkc.other.leakmid { mk_leak }\n\
        fn mk_real() -> Widget<Int> { Widget { semigroup: 3 } }\n";
    let files: &[(&str, &str)] = &[
        ("realleaf.dag", REAL_LEAF),
        ("leakleaf.dag", LEAK_LEAF),
        ("leakmid.dag", LEAK_MID),
        ("consumer.dag", entry),
    ];
    let results = typecheck_fixture_incremental(files);
    let forks: Vec<String> = results
        .iter()
        .flat_map(|tc| tc.binding_forks.iter().map(|c| format!("{c:?}")))
        .collect();
    assert!(
        forks.iter().any(|c| c.contains("Widget")),
        "the realleaf-vs-leakleaf Widget fork must still be ledgered as a typed \
         binding_forks row, got rows:\n{}",
        forks.join("\n")
    );
}

// Same per-module typecheck plumbing as transitive_interface_binding_test: the
// ledger rides the typed out-of-band channel, never diagnostics.
fn typecheck_fixture_incremental(files: &[(&str, &str)]) -> Vec<Rc<TypecheckModuleResult>> {
    let sources: im_rc::Vector<Rc<SourceFile>> = files
        .iter()
        .map(|(path, content)| {
            Rc::new(SourceFile {
                path: path.to_string(),
                content: content.to_string(),
            })
        })
        .collect();
    let frontend = front_end_sources(Rc::new(sources));
    let graph = frontend.graph.clone().expect("resolved module graph");
    let source_indices = frontend.newline_indices.iter().cloned().fold(
        v1_rt::rc_empty_map::<String, Rc<NewlineIndex>>(),
        |acc, si| v1_rt::rc_map_insert(acc, si.file.clone(), si),
    );
    let norm = normalize_graph(graph, source_indices.clone());
    let intern_table: Rc<InternTable> = frontend.intern_table.clone();

    let mut module_index: Rc<im_rc::HashMap<String, Rc<TypedModule>>> = v1_rt::rc_empty_map();
    let mut variant_surfaces: Rc<im_rc::HashMap<String, Rc<VariantExportSurface>>> =
        v1_rt::rc_empty_map();
    let mut results = Vec::new();
    for resolved in norm.graph.modules.iter() {
        let tc = typecheck_module(
            resolved.clone(),
            module_index.clone(),
            variant_surfaces.clone(),
            source_indices.clone(),
            intern_table.clone(),
            v1_compiler::v1_compiler_infer_env::empty_symbol_index(),
        );
        let typed = tc.typed.clone();
        let path = authored_name_at(source_indices.clone(), typed.module.clone());
        variant_surfaces = v1_rt::rc_map_insert(
            variant_surfaces.clone(),
            path.clone(),
            build_variant_export_surface(
                typed.clone(),
                variant_surfaces.clone(),
                source_indices.clone(),
            ),
        );
        module_index = v1_rt::rc_map_insert(module_index, path, typed);
        results.push(tc);
    }
    results
}
