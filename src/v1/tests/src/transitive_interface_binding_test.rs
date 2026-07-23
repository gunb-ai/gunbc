//! Discriminating control for the interface-grain resolver increment (S2a move 2
//! increment B, resolver-graph-major-design.md §7): a dependent must see the
//! STRUCTURE of a type it never imports when that type reaches it through a direct
//! import's exported signature (A imports B, B imports C, B's exported fn returns a
//! C-declared record; A projects a field of that record). This is exactly the
//! regression class of #6304 (ancestry cache-sharing dropped transitive bindings,
//! fixed in #6310): a synthesized/flattened parent env that drops the transitive
//! type identity makes the GREEN case fail — and the RED control below proves the
//! checker genuinely reads the transitive structure rather than waving projections
//! through, so a fail-open flatten cannot pass both arms.

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

const LIB_C: &str = "module fixture.c\n\
    type CPayload { amount: Int }\n";

const LIB_B: &str = "module fixture.b\n\
    import fixture.c { CPayload }\n\
    fn make_payload() -> CPayload { CPayload { amount: 7 } }\n";

fn error_messages(result: &v1_compiler::v1_compiler_compile::PipelineResult) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| format!("{:?}", d.diagnostic))
        .collect()
}

// GREEN arm: the projection through the transitive chain typechecks today.
// This is the arm interface-grain parents must keep green — it goes red the
// moment the flatten drops C's type structure from what flows through B.
#[test]
fn transitive_type_structure_reaches_dependent_through_direct_import_signature() {
    let entry = "module fixture.a\n\
        import fixture.b { make_payload }\n\
        fn read_amount() -> Int { make_payload().amount }\n";
    let result = compile_multi(&[("c.dag", LIB_C), ("b.dag", LIB_B), ("a.dag", entry)]);
    let errors = error_messages(&result);
    assert!(
        errors.is_empty(),
        "A's projection of a C-declared field through B's exported signature must \
         typecheck (transitive type identity flows through the direct import's \
         interface). Errors:\n{}\nAll diagnostics:\n{}",
        errors.join("\n"),
        diagnostic_messages(&result).join("\n")
    );
}

// Binding-fork ledger controls (S2a move 2 increment B, lane ruling REVISED 2026-07-11:
// NOVELTY, not TREE, is the refusal axis). A PRE-EXISTING fork — one already resolved on
// main by import-order overlay-wins — is LEDGERED, not refused, regardless of tree: refusing
// it retroactively would regress working resolution. TREE only labels the dissolution
// partition (same-tree homonym vs cross-tree v1-seed-vs-v2 debt). Both arms below assert the
// SAME ledger-not-refuse behavior; they differ only in the partition the ledger records.

const FORK_DAG_LEAF: &str = "module forka.dagleaf\n\
    type Payload { x: Int }\n";
const FORK_DAG_MID: &str = "module forka.dagmid\n\
    import forka.dagleaf { Payload }\n\
    fn mk_dag() -> Payload { Payload { x: 1 } }\n";
const FORK_V2_LEAF: &str = "module forka.v2leaf\n\
    type Payload { y: Int }\n";
const FORK_V2_MID: &str = "module forka.v2mid\n\
    import forka.v2leaf { Payload }\n\
    fn mk_v2() -> Payload { Payload { y: 2 } }\n";

#[test]
fn same_tree_cross_parent_conflict_is_ledgered_not_refused() {
    // Both chains under plain paths → same tree. Under the revised ruling this is a
    // PRE-EXISTING same-tree fork: LEDGERED (no refusal), each chain keeps its own meaning
    // (.x through the dag-mid chain, .y through the v2-mid chain — overlay-wins winner never
    // leaks into either), and the ledger records the fork with BOTH sites in one tree.
    let entry = "module forka.consumer\n\
        import forka.dagmid { mk_dag }\n\
        import forka.v2mid { mk_v2 }\n\
        fn read_dag_side() -> Int { mk_dag().x }\n\
        fn read_v2_side() -> Int { mk_v2().y }\n";
    let files: &[(&str, &str)] = &[
        ("dagleaf.dag", FORK_DAG_LEAF),
        ("dagmid.dag", FORK_DAG_MID),
        ("v2leaf.dag", FORK_V2_LEAF),
        ("v2mid.dag", FORK_V2_MID),
        ("consumer.dag", entry),
    ];
    let result = compile_multi(files);
    let errors = error_messages(&result);
    let msgs = diagnostic_messages(&result).join("\n");
    assert!(
        errors.is_empty(),
        "a same-tree PRE-EXISTING fork is LEDGERED, never refused (novelty-not-tree ruling); \
         refusing it would regress main's working overlay-wins resolution. Errors:\n{}\nAll:\n{msgs}",
        errors.join("\n")
    );
    assert!(
        !msgs.contains("cross-parent") && !msgs.contains("binding fork"),
        "the ledger must NOT ride the diagnostics channel, got:\n{msgs}"
    );
    // Ledger observability: the fork is recorded on the typed channel with both decl
    // sites in the SAME tree (the same-tree partition).
    let results = typecheck_fixture_incremental(files);
    let forks: Vec<String> = results
        .iter()
        .flat_map(|tc| tc.binding_forks.iter().map(|c| format!("{c:?}")))
        .collect();
    assert!(
        forks.iter().any(|c| c.contains("Payload")),
        "the same-tree fork must be ledgered as a typed binding_forks row naming 'Payload', \
         got rows:\n{}",
        forks.join("\n")
    );
    assert!(
        forks
            .iter()
            .all(|c| !(c.contains("src/v2/") || c.contains("src/v1/"))),
        "same-tree ledger rows must locate both decl sites in ONE tree (no src/v2 or src/v1 \
         path here — plain paths classify as the 'dag' partition sibling 'other'), got rows:\n{}",
        forks.join("\n")
    );
}

// Direct-typecheck plumbing for the ledger-channel assertion: the ledger rides the
// typed out-of-band channel (TypecheckModuleResult.binding_forks), never diagnostics
// (consumers rightly read diagnostics as compile cleanliness), so observing it means
// reading the per-module result the floor's receipt line aggregates — the same shape
// variant_export_surface_witness_test uses.
fn typecheck_fixture_incremental(files: &[(&str, &str)]) -> Vec<Rc<TypecheckModuleResult>> {
    let sources: im::Vector<Rc<SourceFile>> = files
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

    let mut module_index: Rc<im::HashMap<String, Rc<TypedModule>>> = v1_rt::rc_empty_map();
    let mut variant_surfaces: Rc<im::HashMap<String, Rc<VariantExportSurface>>> =
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

#[test]
fn cross_tree_conflict_is_ledgered_and_keeps_import_order_winner() {
    // One chain homed under dag/, the other under src/v2/ → the known two-tree
    // fork debt: no refusal, a ledger row instead, and — the behavior-preservation
    // invariant — each chain keeps its OWN meaning: the dag-side projection (.x)
    // and the v2-side projection (.y) BOTH typecheck through their respective
    // imports, so the cache-union winner never leaks into either chain's inference.
    let entry = "module forka.consumer\n\
        import forka.dagmid { mk_dag }\n\
        import forka.v2mid { mk_v2 }\n\
        fn read_dag_side() -> Int { mk_dag().x }\n\
        fn read_v2_side() -> Int { mk_v2().y }\n";
    let files: &[(&str, &str)] = &[
        ("dag/forka/dagleaf.dag", FORK_DAG_LEAF),
        ("dag/forka/dagmid.dag", FORK_DAG_MID),
        ("src/v2/forka/v2leaf.dag", FORK_V2_LEAF),
        ("src/v2/forka/v2mid.dag", FORK_V2_MID),
        ("consumer.dag", entry),
    ];
    let result = compile_multi(files);
    let errors = error_messages(&result);
    let msgs = diagnostic_messages(&result).join("\n");
    assert!(
        errors.is_empty(),
        "a cross-tree fork is LEDGERED, never refused (declared interim; dissolve-on \
         std consolidation). Errors:\n{}\nAll:\n{msgs}",
        errors.join("\n")
    );
    assert!(
        !msgs.contains("cross-tree"),
        "the ledger must NOT ride the diagnostics channel (out-of-band typed field \
         only — diagnostics are a compile-cleanliness signal), got:\n{msgs}"
    );

    // Ledger observability on the typed channel: the consumer's per-module result
    // carries the counted, located rows the floor receipt aggregates.
    let results = typecheck_fixture_incremental(files);
    let forks: Vec<String> = results
        .iter()
        .flat_map(|tc| tc.binding_forks.iter().map(|c| format!("{c:?}")))
        .collect();
    assert!(
        forks.iter().any(|c| c.contains("Payload")),
        "the cross-tree fork must be ledgered as a typed binding_forks row naming \
         'Payload', got rows:\n{}",
        forks.join("\n")
    );
    assert!(
        forks
            .iter()
            .all(|c| c.contains("dag/") && c.contains("src/v2/")),
        "every ledger row must locate BOTH decl sites (one per tree), got rows:\n{}",
        forks.join("\n")
    );
}

// RED control: a bogus field on the same transitive record MUST produce an error.
// This is what makes the green arm discriminating — a parent-env flatten that is
// blind to CPayload's structure would wave both projections through, and this arm
// catches that fail-open before the green arm's silence can be misread as success.
#[test]
fn bogus_field_on_transitive_record_is_a_typed_error() {
    let entry = "module fixture.a\n\
        import fixture.b { make_payload }\n\
        fn read_bogus() -> Int { make_payload().no_such_field }\n";
    let result = compile_multi(&[("c.dag", LIB_C), ("b.dag", LIB_B), ("a.dag", entry)]);
    let errors = error_messages(&result);
    assert!(
        !errors.is_empty(),
        "projecting a nonexistent field of the transitive record must be a typed \
         error — silence here means inference is blind to the transitive structure. \
         All diagnostics:\n{}",
        diagnostic_messages(&result).join("\n")
    );
}
