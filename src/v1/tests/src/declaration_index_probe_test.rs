//! Measures the declaration index against real corpus specimens.
//!
//! Subject: the 151-site `undefined variable Empty` class the floor cut's
//! whole-corpus strict fold refused on. `Empty` is the nullary constructor of
//! `FreeMonoid`, declared once in `dag/std/algebra.dag`, referenced bare across
//! many files with no import bringing it into scope. If containment-based
//! resolution is to handle the corpus, the index must map that bare name to its
//! single declaring module.
//!
//! These are MEASUREMENTS, not the closure. They isolate the index — the half
//! that maps a name to the modules declaring it — from the fixpoint that walks
//! it, because the fixpoint currently over-approximates and its cost would
//! otherwise mask what the index does or does not know.

use std::path::PathBuf;

use crate::helpers::workspace_root;

fn corpus_roots() -> Vec<PathBuf> {
    let ws = workspace_root();
    vec![ws.join("dag"), ws.join("src/v2")]
}

fn build() -> (
    v1_compiler::source_closure::DeclarationIndex,
    Vec<String>,
    std::time::Duration,
) {
    let ws = workspace_root();
    let roots = corpus_roots();
    let started = std::time::Instant::now();
    let (index, unparsed) = v1_compiler::source_closure::build_declaration_index(&roots, &ws);
    (index, unparsed, started.elapsed())
}

#[test]
fn declaration_index_binds_bare_empty_to_its_single_declaring_module() {
    let (index, unparsed, elapsed) = build();

    eprintln!(
        "[index] {} modules, {} distinct names, {} unparsed, built in {:?}",
        index.module_count(),
        index.name_count(),
        unparsed.len(),
        elapsed
    );

    // The index must be built over a whole corpus, not a fragment: a small
    // module count would make every assertion below vacuously easy.
    assert!(
        index.module_count() > 3000,
        "index covers only {} modules; the corpus is ~3700, so this is not a \
         whole-corpus measurement",
        index.module_count()
    );

    let declaring = index.modules_declaring("Empty").unwrap_or_else(|| {
        panic!(
            "bare `Empty` resolves to NO declaring module. It is the nullary \
             constructor of FreeMonoid in dag/std/algebra.dag, so the index is \
             not indexing coproduct variant arms."
        )
    });

    let found: Vec<&String> = declaring.iter().collect();
    eprintln!("[Empty] declaring modules: {found:?}");

    assert!(
        declaring.contains("std.algebra"),
        "bare `Empty` does not bind to std.algebra; got {found:?}"
    );
}

#[test]
fn declaration_index_reports_the_nat_compare_homonym_rather_than_picking_one() {
    let (index, _unparsed, _elapsed) = build();

    let declaring = index
        .modules_declaring("nat_compare")
        .expect("nat_compare must be declared somewhere in the corpus");
    let found: Vec<&String> = declaring.iter().collect();
    eprintln!("[nat_compare] declaring modules: {found:?}");

    // The floor's strict fold reported nat_compare as ambiguous across the two
    // std trees. The index must SURFACE that as multiple candidates, never
    // collapse it to one -- picking a winner here would hide a real ambiguity
    // from the resolver, which is the layer entitled to refuse.
    assert!(
        declaring.len() >= 2,
        "expected nat_compare to be declared by both std trees, so the index \
         surfaces the ambiguity instead of picking a winner; got {found:?}"
    );
}

/// The number that decides whether closure assembly is usable at all.
///
/// A five-line entry referencing one std type must produce a SMALL closure. If
/// it closes over most of the corpus, every consumer pays a whole-corpus
/// typecheck to compile five lines, which is what the over-approximation cost
/// before free-name narrowing.
#[test]
fn small_entry_produces_a_small_closure() {
    const ENTRY: &str = r#"
module test.closure_width_probe

fn one_half() -> FieldOfFractions<Int> { FieldOfFractions { num: 1, denom: 2 } }
"#;
    let (index, _unparsed, index_elapsed) = build();
    let started = std::time::Instant::now();
    let closure = v1_compiler::source_closure::closure_for_entry("test.dag", ENTRY, &index);
    let closure_elapsed = started.elapsed();

    let paths: Vec<&str> = closure.iter().map(|s| s.path.as_str()).collect();
    eprintln!(
        "[closure] {} of {} modules in {:?} (index {:?})",
        closure.len(),
        index.module_count(),
        closure_elapsed,
        index_elapsed
    );
    if closure.len() <= 40 {
        eprintln!("[closure] members: {paths:?}");
    }

    assert!(
        closure.iter().any(|s| s.path.ends_with("std/algebra.dag")),
        "closure must contain the module declaring FieldOfFractions; got {} members",
        closure.len()
    );

    // The bar is deliberately far below the corpus rather than at some tuned
    // value: this asserts the closure is a CLOSURE, not that it is optimal.
    assert!(
        closure.len() < index.module_count() / 4,
        "closure over-approximates: {} of {} modules for a five-line entry",
        closure.len(),
        index.module_count()
    );
}
