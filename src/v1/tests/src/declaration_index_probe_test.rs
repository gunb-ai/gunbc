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

    // THE PARSE GATE. cargo does not read .dag, so a cut branch's cargo runs
    // are green on .dag edits by not looking at them. Building this index
    // parses every .dag file under the roots, which makes it a whole-corpus
    // parse check as a side effect -- but only if the result is ASSERTED.
    // Printing the count and passing anyway is how a malformed literal rides
    // four green pushes.
    assert!(
        unparsed.is_empty(),
        "{} source file(s) failed to parse: {:?}",
        unparsed.len(),
        unparsed.iter().take(10).collect::<Vec<_>>()
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
    let (closure, pulls) =
        v1_compiler::source_closure::closure_for_entry_attributed("test.dag", ENTRY, &index);
    let closure_elapsed = started.elapsed();

    let paths: Vec<&str> = closure.iter().map(|s| s.path.as_str()).collect();
    eprintln!(
        "[closure] {} of {} modules in {:?} (index {:?})",
        closure.len(),
        index.module_count(),
        closure_elapsed,
        index_elapsed
    );
    eprintln!("[closure] top pulling names: {pulls:?}");
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

/// Quantifies the cause the width probe could not distinguish by itself.
///
/// A bare reference to a name declared by N modules pulls all N, so homonyms
/// multiply closure width by construction rather than by defect. This reports
/// how much of the flat namespace is homonymous.
#[test]
fn homonym_census_over_the_flat_namespace() {
    let (index, _unparsed, _elapsed) = build();
    let (multi, total, worst) = index.homonym_stats();
    eprintln!(
        "[homonyms] {multi} of {total} names declared by >1 module ({:.1}%)",
        (multi as f64 / total as f64) * 100.0
    );
    eprintln!("[homonyms] worst: {worst:?}");
    assert!(total > 1000, "census must run over a real namespace");
}

/// Parse-checks the `.dag` files that the corpus roots do NOT cover.
///
/// Asked from the other side, per the census rule: the main gate reports 3711
/// modules, but the repository holds 3786 `.dag` files, so "0 unparsed" was
/// only ever a statement about the population that gate was pointed at. This
/// enumerates the remainder and checks it, rather than trusting that the roots
/// are the whole tree.
///
/// `src/v1/**` is deliberately excluded: its `.dag` authority is deleted on the
/// v1 cut, so those files are moot at integration rather than unchecked. What
/// this covers is the residue that survives -- notably `fixtures/`, which this
/// lane stripped imports from and which no other gate parses.
#[test]
fn dag_files_outside_the_corpus_roots_also_parse() {
    let ws = workspace_root();
    let roots = vec![ws.join("fixtures"), ws.join("test")];
    let present: Vec<PathBuf> = roots.iter().filter(|r| r.exists()).cloned().collect();
    assert!(
        !present.is_empty(),
        "no auxiliary roots found; this check would pass vacuously"
    );

    let (index, unparsed) = v1_compiler::source_closure::build_declaration_index(&present, &ws);
    eprintln!(
        "[aux-roots] {} modules, {} unparsed, roots {:?}",
        index.module_count(),
        unparsed.len(),
        present
    );
    assert!(
        index.module_count() > 0,
        "auxiliary roots yielded no modules; the check would prove nothing"
    );
    assert!(
        unparsed.is_empty(),
        "{} auxiliary source file(s) failed to parse: {:?}",
        unparsed.len(),
        unparsed
    );
}

/// Discriminating control for the instrument itself: what does the walk DO when
/// a file is malformed?
///
/// Routed obligation: the whole-source-root parse walk in cli_run refuses by
/// PANIC rather than by a typed located diagnostic. It is fail-closed in effect
/// but the wrong shape for DESIGN §5, and since this module performs the same
/// walk it is the likeliest terminal home for that obligation. I asserted
/// earlier that a panicking parser would make the index panic too; that was an
/// assertion, so this measures it instead.
///
/// Plants a malformed module in a temp root and records the observed behavior.
/// If this test PASSES, the index refuses by returning the file in `unparsed`,
/// which is the typed-and-located shape the obligation wants. If it panics
/// instead, the panic IS the finding and the obligation is inherited.
#[test]
fn malformed_module_is_returned_as_unparsed_not_panicked() {
    let dir = std::env::temp_dir().join(format!("gunbc_malformed_probe_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp root");

    // A well-formed control, so a vacuous empty root cannot pass this test.
    std::fs::write(
        dir.join("good.dag"),
        "module probe.good\ntype Ok { v: Int }\n",
    )
    .expect("write good");
    // The planted defect: a list literal whose closing bracket is missing, which
    // is the exact shape a variant deletion left behind on a sibling branch.
    std::fs::write(
        dir.join("bad.dag"),
        "module probe.bad\nfn rows() -> List<Int> { [1, 2,\n",
    )
    .expect("write bad");

    let (index, unparsed) =
        v1_compiler::source_closure::build_declaration_index(&[dir.clone()], &dir);
    eprintln!(
        "[malformed-probe] {} modules indexed, unparsed: {:?}",
        index.module_count(),
        unparsed
    );

    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(
        index.module_count(),
        1,
        "the well-formed control must still index, or this proves nothing"
    );
    assert_eq!(
        unparsed.len(),
        1,
        "the malformed module must be REPORTED, not silently skipped: {unparsed:?}"
    );
}
