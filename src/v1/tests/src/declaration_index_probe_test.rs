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

/// Cross-lane control: a name mentioned inside a STRING LITERAL must not pull
/// the module declaring it.
///
/// The trap is real for grep-based joins — .dag modules carry long §4c
/// rationale strings that name exactly what a cut is searching for, so six
/// apparent hits on a sibling lane were prose discussing the interpreter rather
/// than references to it. This walk reads parsed `Node` structure rather than
/// text, so it should be immune by construction; that is an assertion until it
/// is executed, and assertions have not fared well in this lane.
///
/// Plants a module whose only mention of a declared name is inside a string.
#[test]
fn a_name_inside_a_string_literal_is_not_a_reference() {
    let dir = std::env::temp_dir().join(format!("gunbc_strlit_probe_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp root");

    std::fs::write(
        dir.join("provider.dag"),
        "module probe.provider\ntype PulledIfReferenced { v: Int }\n",
    )
    .expect("write provider");
    // The mention is prose inside a rationale string, exactly the §4c shape.
    std::fs::write(
        dir.join("prose.dag"),
        "module probe.prose\ndata note: String = \"discusses PulledIfReferenced at length\"\n",
    )
    .expect("write prose");
    // Positive control: the same name in a real reference position MUST pull.
    std::fs::write(
        dir.join("real.dag"),
        "module probe.real\nfn use_it(x: PulledIfReferenced) -> Int { x.v }\n",
    )
    .expect("write real");

    let (index, unparsed) =
        v1_compiler::source_closure::build_declaration_index(&[dir.clone()], &dir);
    assert!(unparsed.is_empty(), "fixtures must parse: {unparsed:?}");

    let prose = std::fs::read_to_string(dir.join("prose.dag")).unwrap();
    let real = std::fs::read_to_string(dir.join("real.dag")).unwrap();
    let from_prose = v1_compiler::source_closure::closure_for_entry("prose.dag", &prose, &index);
    let from_real = v1_compiler::source_closure::closure_for_entry("real.dag", &real, &index);

    let pulled = |c: &Vec<std::rc::Rc<v1_compiler::v1_compiler_compile::SourceFile>>| {
        c.iter().any(|s| s.path.ends_with("provider.dag"))
    };
    eprintln!(
        "[string-literal] prose pulls provider: {} | real pulls provider: {}",
        pulled(&from_prose),
        pulled(&from_real)
    );

    let _ = std::fs::remove_dir_all(&dir);

    // The positive control comes first: if a genuine reference does not pull,
    // the negative result below would be meaningless.
    assert!(
        pulled(&from_real),
        "a genuine reference must pull the declaring module, or this test proves nothing"
    );
    assert!(
        !pulled(&from_prose),
        "a name inside a string literal pulled its declaring module: the walk is \
         reading text, not structure"
    );
}

/// BLAST RADIUS for the type-argument qualification hazard (tidy-pike-117 /
/// sleek-moth-351, 2026-08-15).
///
/// The reported defect: of 116 corpus sites matching `Empty`/`Cons`, the only
/// failing one has a scrutinee whose declared type carries a DOTTED QUALIFIED
/// type ARGUMENT (`List<v2.lens.complexity_accumulator_copy.Finding>`), and
/// the diagnostic lands at the type-annotation column rather than the arm.
/// The hypothesis is therefore that a qualified name fails to resolve in
/// type-argument position, and that it was CREATED by an earlier qualification
/// pass rather than revealed by one.
///
/// The question this lane can answer and the others cannot: how many
/// type-argument positions exist, and how many are dotted today. That is the
/// denominator qualification would move names INTO.
///
/// What this measures precisely: a name occurring in type position whose
/// PARENT is also in type position -- i.e. an argument of a type constructor,
/// not the constructor itself. Counted per occurrence, not per distinct name,
/// because the hazard is per site.
#[test]
fn type_argument_position_census_over_the_corpus() {
    use std::collections::BTreeMap;
    use v1_compiler::v1_std_core::Node;

    fn walk(
        node: &std::rc::Rc<Node>,
        in_type_position: bool,
        parent_is_type: bool,
        bare: &mut usize,
        dotted: &mut usize,
        dotted_names: &mut BTreeMap<String, usize>,
    ) {
        let here_is_type_arg = in_type_position && parent_is_type;
        if here_is_type_arg && !node.name.is_empty() {
            if node.name.contains('.') {
                *dotted += 1;
                *dotted_names.entry(node.name.clone()).or_insert(0) += 1;
            } else {
                *bare += 1;
            }
        }
        if let Some(annotation) = node.type_annotation.as_ref() {
            walk(annotation, true, false, bare, dotted, dotted_names);
        }
        for child in node.children.iter() {
            walk(
                child,
                in_type_position,
                in_type_position,
                bare,
                dotted,
                dotted_names,
            );
        }
        for param in node.params.iter() {
            for declared_type in param.children.iter() {
                walk(declared_type, true, false, bare, dotted, dotted_names);
            }
            if let Some(annotation) = param.type_annotation.as_ref() {
                walk(annotation, true, false, bare, dotted, dotted_names);
            }
        }
        if let Some(body) = node.body.as_ref() {
            walk(body, false, false, bare, dotted, dotted_names);
        }
    }

    let roots = corpus_roots();
    let files = v1_compiler::source_closure::dag_files_under(&roots);
    let (mut bare, mut dotted) = (0usize, 0usize);
    let mut dotted_names: BTreeMap<String, usize> = BTreeMap::new();
    let mut parsed = 0usize;

    for path in files.iter() {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let module =
            match v1_compiler::source_closure::parse_file(&path.to_string_lossy(), &content) {
                Some(m) => m,
                None => continue,
            };
        parsed += 1;
        walk(
            &module,
            false,
            false,
            &mut bare,
            &mut dotted,
            &mut dotted_names,
        );
    }

    eprintln!(
        "[type-arg census] {} files parsed | {} type-argument occurrences: {} bare, {} dotted",
        parsed,
        bare + dotted,
        bare,
        dotted
    );
    let mut ranked: Vec<_> = dotted_names.iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (name, count) in ranked.iter().take(25) {
        eprintln!("[type-arg dotted] {:>4}  {}", count, name);
    }

    // POSITIVE CONTROL. A census that could only ever report zero is not a
    // census. If the walk finds no type-argument position at all in a corpus
    // this size, the walk is broken, not the corpus.
    assert!(
        bare + dotted > 0,
        "walk found no type-argument positions in {} files -- the walk is wrong, not the corpus",
        parsed
    );

    // THE SECOND CONTROL, and it is the one the first does not cover. The
    // dotted count is read off `name.contains('.')`, which silently assumes the
    // parser keeps a qualified name as ONE node rather than splitting it into a
    // projection chain. If it splits, `dotted` is 0 for a representation reason
    // and I would report "the corpus has no dotted type arguments" -- a false
    // absence, and the same shape as the parameter-type walk that returned an
    // empty set for an ordinary function. So plant one and demand the walk sees
    // it. A bare-only assertion above would pass either way.
    let planted = v1_compiler::source_closure::parse_file(
        "planted.dag",
        "module probe.planted\n\nfn f(xs: List<lens.finding_provider.Finding>) -> Int { 0 }\n",
    )
    .expect("planted control must parse");
    let (mut pbare, mut pdotted) = (0usize, 0usize);
    let mut pnames: BTreeMap<String, usize> = BTreeMap::new();
    walk(
        &planted,
        false,
        false,
        &mut pbare,
        &mut pdotted,
        &mut pnames,
    );
    eprintln!(
        "[type-arg control] planted dotted argument -> bare {} dotted {} names {:?}",
        pbare, pdotted, pnames
    );
    assert!(
        pdotted > 0,
        "the walk cannot see a planted dotted type argument, so the corpus dotted \
         count of {} is a representation artifact and not a measurement (saw bare={} names={:?})",
        dotted,
        pbare,
        pnames
    );
}

/// sleek-moth-351's experiment, with the controls that decide the axis
/// (fifth cell from tidy-pike-117, 2026-08-15).
///
/// The reported defect: of 116 corpus sites matching `Empty`/`Cons`, the only
/// failing one has a scrutinee whose declared type carries a DOTTED QUALIFIED
/// type ARGUMENT, and the diagnostic lands at the type-annotation column rather
/// than the arm. So the hypothesis is that a qualified name fails to resolve in
/// type-argument position -- and it was CREATED by an earlier qualification
/// pass, which is the same transformation this lane runs at scale.
///
/// "Dotted AND type-argument AND alias AND match" is a CONJUNCTION, and the
/// single corpus positive cannot separate its terms. Five cells can:
///
///   A  bare arg, alias, match          Sack<Finding>            expect clean
///   B  dotted arg, alias, match        Sack<prov.Finding>       the subject
///   C  dotted, NOT an argument         x: prov.Finding          isolates "dotted"
///   D  dotted arg, alias, NO match     Sack<prov.Finding>       isolates "match"
///   E  dotted arg, DIRECT type, match  Bag<prov.Finding>        isolates "alias"
///
/// Only-B-fails means the conjunction is real. C failing too means the axis is
/// merely "dotted". D failing means the arms were never load-bearing and the
/// type-annotation column was telling the truth. B failing while E passes means
/// the ALIAS is load-bearing and this rejoins sleek-moth's question rather than
/// being mine.
///
/// The shapes are declared IN the fixture rather than borrowed from `std`. A
/// fixture that reached for `List`/`FreeMonoid` would be testing whether the
/// std tree is in this harness's pool at the same time as it tested
/// qualification, and a failure could not be attributed to either.
#[test]
fn dotted_type_argument_experiment_with_axis_controls() {
    use crate::helpers::diagnostic_messages;
    use std::rc::Rc;
    use v1_compiler::v1_compiler_artifact::RenderTarget;
    use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};

    // Compiled from the two fixture files ALONE, not through compile_multi.
    // That helper resolves each file transitively against the whole corpus
    // index, which would put ~3700 modules in the pool -- so a cell could
    // resolve a name from somewhere incidental, and the experiment would be
    // measuring the corpus as much as the fixture. It also refuses outright on
    // src/v1/stage0/tests/fixtures/fact_cardinality_split_brace.dag, a
    // DELIBERATELY malformed parser specimen that my parse gate cannot tell
    // apart from real breakage. Both problems disappear by giving the compiler
    // exactly the two modules under test, which is what a controlled fixture
    // means.
    let compile = |cell: &str| -> Vec<String> {
        let sources: Vec<Rc<SourceFile>> = vec![
            Rc::new(SourceFile {
                path: PROVIDER.0.to_string(),
                content: PROVIDER.1.to_string(),
            }),
            Rc::new(SourceFile {
                path: REMOTE_ALIAS.0.to_string(),
                content: REMOTE_ALIAS.1.to_string(),
            }),
            Rc::new(SourceFile {
                path: "cell.dag".to_string(),
                content: cell.to_string(),
            }),
        ];
        let result = compile_sources(Rc::new(sources.into()), RenderTarget::Rust);
        diagnostic_messages(&result)
    };

    // Syntax measured off the corpus, not assumed: coproducts and records are
    // both `type`, and `data` is a VALUE declaration -- the first version of
    // this fixture wrote `data Bag<t> = ... | ...` and the parser answered
    // "expected Colon, found Eq", which cascaded into unresolved-type
    // diagnostics in every cell including the control. That is precisely what
    // the control is for: cell A was dirty, so no cell's verdict counted.
    const PROVIDER: (&str, &str) = (
        "provider.dag",
        "module probe.provider\n\
         \n\
         type Finding { detail: String }\n\
         \n\
         type Bag<t>\n\
           = BagEmpty\n\
           | BagCons { head: t, tail: Bag<t> }\n\
         \n\
         type Sack<t> = Bag<t>\n",
    );

    // A SECOND alias, declared in a module SEPARATE from its target coproduct.
    // This is the factor tidy-pike-117 relayed from sleek-moth-351 as
    // "alias chain across trees": the corpus specimen's List aliases a
    // FreeMonoid that lives in ANOTHER module in ANOTHER tree, whereas cells
    // A-E alias a coproduct declared beside them.
    //
    // The two halves of that description are SEPARABLE and only one is
    // reproducible here: "another module" is structural and synthesizable;
    // "another tree" is a property of the corpus source roots, which this
    // harness deliberately does not use (it compiles the fixture modules
    // alone). So cell F tests the module-separation half ONLY, and a clean
    // result narrows the corpus factor to tree-separation rather than
    // clearing the hypothesis.
    const REMOTE_ALIAS: (&str, &str) = (
        "remote_alias.dag",
        "module probe.remote\n\
         \n\
         type RemoteSack<t> = Bag<t>\n",
    );

    let cells: Vec<(&str, &str)> = vec![
        (
            "A bare arg, alias, match",
            "module probe.cell\n\nfn f(xs: Sack<Finding>) -> Int {\n  match xs {\n    BagEmpty => 0,\n    BagCons { head: h, tail: t } => 1,\n  }\n}\n",
        ),
        (
            "B dotted arg, alias, match",
            "module probe.cell\n\nfn f(xs: Sack<probe.provider.Finding>) -> Int {\n  match xs {\n    BagEmpty => 0,\n    BagCons { head: h, tail: t } => 1,\n  }\n}\n",
        ),
        (
            "C dotted, not an argument",
            "module probe.cell\n\nfn f(x: probe.provider.Finding) -> Int { 0 }\n",
        ),
        (
            "D dotted arg, alias, no match",
            "module probe.cell\n\nfn f(xs: Sack<probe.provider.Finding>) -> Int { 0 }\n",
        ),
        (
            "F dotted arg, alias in ANOTHER module, match",
            "module probe.cell\n\nfn f(xs: RemoteSack<probe.provider.Finding>) -> Int {\n  match xs {\n    BagEmpty => 0,\n    BagCons { head: h, tail: t } => 1,\n  }\n}\n",
        ),
        (
            "G bare arg, alias in ANOTHER module, match",
            "module probe.cell\n\nfn f(xs: RemoteSack<Finding>) -> Int {\n  match xs {\n    BagEmpty => 0,\n    BagCons { head: h, tail: t } => 1,\n  }\n}\n",
        ),
        (
            "E dotted arg, DIRECT type, match",
            "module probe.cell\n\nfn f(xs: Bag<probe.provider.Finding>) -> Int {\n  match xs {\n    BagEmpty => 0,\n    BagCons { head: h, tail: t } => 1,\n  }\n}\n",
        ),
    ];

    let mut verdicts: Vec<(&str, Vec<String>)> = Vec::new();
    for (label, source) in cells.iter() {
        let diags = compile(source);
        eprintln!(
            "[cell] {:<34} {}",
            label,
            if diags.is_empty() {
                "clean".to_string()
            } else {
                format!("{} diagnostic(s): {:?}", diags.len(), diags)
            }
        );
        verdicts.push((label, diags));
    }

    let clean = |i: usize| verdicts[i].1.is_empty();
    eprintln!(
        "[axis] {}   (clean=true)",
        verdicts
            .iter()
            .enumerate()
            .map(|(i, (label, _))| format!(
                "{}={}",
                label.split_whitespace().next().unwrap_or("?"),
                clean(i)
            ))
            .collect::<Vec<_>>()
            .join(" ")
    );

    // THE POSITIVE CONTROL, and it is the one that matters. Cell A contains no
    // dotted name anywhere and exercises every other feature the subject uses:
    // the alias, the type argument, the match arms, the cross-module reference.
    // If A is dirty the fixture proves nothing, and every other cell's failure
    // would be an artifact of this harness rather than evidence about
    // qualification -- the way a negative-only test reports success on a walk
    // that collected nothing at all.
    assert!(
        clean(0),
        "cell A (no dotted name anywhere) must compile clean or this fixture \
         cannot attribute any other cell's failure: {:?}",
        verdicts[0].1
    );
}

/// The 18 siblings (tidy-pike-117's next question, 2026-08-15).
///
/// 19 of the corpus's 49 dotted type-argument occurrences sit in ONE module,
/// `v2.lens.complexity_accumulator_copy`, and exactly one of them is known to
/// fail. If the axis were dottedness-in-type-argument-position, its siblings
/// should fail too. If 18 of 19 compile clean, the axis is something narrower
/// living inside that one declaration, and "dotted type argument" is the wrong
/// name for the class.
///
/// This compiles the real module through its reference-derived closure rather
/// than re-staging its shape in a fixture, because a re-staged shape is my
/// reading of the site and the question is precisely whether my reading is
/// complete.
#[test]
fn dotted_argument_siblings_in_the_cluster_module() {
    use crate::helpers::diagnostic_messages;
    use v1_compiler::v1_compiler_artifact::RenderTarget;
    use v1_compiler::v1_compiler_compile::compile_sources;

    let ws = workspace_root();
    let path = ws.join("src/v2/lens/complexity_accumulator_copy.dag");
    let content = std::fs::read_to_string(&path).expect("cluster module must exist");

    let (index, unparsed, _elapsed) = build();
    assert!(unparsed.is_empty(), "index refuses: {unparsed:?}");

    let sources =
        v1_compiler::source_closure::closure_for_entry(&path.to_string_lossy(), &content, &index);
    let closure_size = sources.len();
    eprintln!("[cluster] closure of {closure_size} modules");

    let result = compile_sources(std::rc::Rc::new(sources.into()), RenderTarget::Rust);
    let diags = diagnostic_messages(&result);
    eprintln!("[cluster] {} diagnostic(s)", diags.len());
    for d in diags.iter().take(40) {
        eprintln!("[cluster diag] {d}");
    }

    // No assertion on the COUNT. This is a measurement of a module nobody has
    // claimed is clean, and pinning a number here would turn whatever it
    // reports today into a contract. What it must not do is report zero for
    // the reason the census nearly did -- an empty closure would compile
    // nothing and print "0 diagnostics", which reads exactly like success.
    assert!(
        closure_size > 1,
        "closure holds {closure_size} module(s), so this compiled nothing and its \
         diagnostic count says nothing"
    );
}

/// The measured expected population, between the 49 floor and the 8978 ceiling.
///
/// tidy-pike-117 proposed applying DESIGN's "98% of names are globally unique"
/// census to the 8978 bare type-argument positions, and correctly refused to
/// treat the result as an answer: that census runs over ALL names, and type
/// names may have a different uniqueness profile from functions and values.
/// This measures the profile for the names actually occurring in bare
/// type-argument position, which is the population in question.
///
/// BOUND, stated because the number invites over-reading: the index answers
/// how many modules DECLARE a name globally. A name with two declarers may
/// still be unique on the referencing module's containment chain and never need
/// qualification. So the homonym figure is an UPPER bound on the must-qualify
/// population, not an estimate of it.
#[test]
fn uniqueness_profile_of_bare_type_argument_names() {
    use std::collections::BTreeMap;
    use v1_compiler::v1_std_core::Node;

    fn walk(
        node: &std::rc::Rc<Node>,
        in_type_position: bool,
        parent_is_type: bool,
        bare: &mut BTreeMap<String, usize>,
    ) {
        if in_type_position && parent_is_type && !node.name.is_empty() && !node.name.contains('.') {
            *bare.entry(node.name.clone()).or_insert(0) += 1;
        }
        if let Some(a) = node.type_annotation.as_ref() {
            walk(a, true, false, bare);
        }
        for c in node.children.iter() {
            walk(c, in_type_position, in_type_position, bare);
        }
        for p in node.params.iter() {
            for t in p.children.iter() {
                walk(t, true, false, bare);
            }
            if let Some(a) = p.type_annotation.as_ref() {
                walk(a, true, false, bare);
            }
        }
        if let Some(b) = node.body.as_ref() {
            walk(b, false, false, bare);
        }
    }

    let (index, unparsed, _e) = build();
    assert!(unparsed.is_empty(), "index refuses: {unparsed:?}");

    let roots = corpus_roots();
    let mut bare: BTreeMap<String, usize> = BTreeMap::new();
    for path in v1_compiler::source_closure::dag_files_under(&roots) {
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Some(module) =
            v1_compiler::source_closure::parse_file(&path.to_string_lossy(), &content)
        else {
            continue;
        };
        walk(&module, false, false, &mut bare);
    }

    let (mut occ_unique, mut occ_homonym, mut occ_undeclared) = (0usize, 0usize, 0usize);
    let (mut n_unique, mut n_homonym, mut n_undeclared) = (0usize, 0usize, 0usize);
    let mut worst: Vec<(usize, usize, String)> = Vec::new();
    for (name, count) in bare.iter() {
        match index.modules_declaring(name) {
            None => {
                occ_undeclared += count;
                n_undeclared += 1;
            }
            Some(d) if d.len() == 1 => {
                occ_unique += count;
                n_unique += 1;
            }
            Some(d) => {
                occ_homonym += count;
                n_homonym += 1;
                worst.push((d.len(), *count, name.clone()));
            }
        }
    }
    worst.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));

    let occ_total = occ_unique + occ_homonym + occ_undeclared;
    eprintln!(
        "[uniqueness] {} distinct names over {} bare type-argument occurrences",
        bare.len(),
        occ_total
    );
    eprintln!(
        "[uniqueness] occurrences: {} unique ({:.1}%), {} homonym ({:.1}%), {} undeclared ({:.1}%)",
        occ_unique,
        occ_unique as f64 / occ_total as f64 * 100.0,
        occ_homonym,
        occ_homonym as f64 / occ_total as f64 * 100.0,
        occ_undeclared,
        occ_undeclared as f64 / occ_total as f64 * 100.0
    );
    eprintln!(
        "[uniqueness] distinct names: {n_unique} unique, {n_homonym} homonym, {n_undeclared} undeclared"
    );
    // EVERY homonym, with its declaring modules -- not a top-N. The population
    // is 37 distinct names, which is small enough to READ, and reading it is
    // the only way to tell a genuine two-concept collision from one concept
    // declared in both std trees. A count cannot make that distinction, and the
    // count is dominated by three names, so the headline rests entirely on how
    // those three classify (tidy-pike-117, 2026-08-15).
    for (declarers, occurrences, name) in worst.iter() {
        let mods: Vec<&String> = index
            .modules_declaring(name)
            .map(|d| d.iter().collect())
            .unwrap_or_default();
        eprintln!(
            "[homonym] {declarers:>2} declarers, {occurrences:>5} sites  {name:<28} {mods:?}"
        );
    }

    // EVERY undeclared name. If these are type VARIABLES (the T in List<T>)
    // they are a different kind sitting in the same census and can never need
    // qualification, so they do not belong in the denominator. If they are not,
    // then N type names are referenced with no declarer anywhere in the corpus,
    // which is a larger finding than the homonym count and must not stay buried
    // inside a percentage.
    let mut undeclared: Vec<(&String, &usize)> = bare
        .iter()
        .filter(|(n, _)| index.modules_declaring(n).is_none())
        .collect();
    undeclared.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (name, count) in undeclared.iter() {
        eprintln!("[undeclared] {count:>5} sites  {name}");
    }

    // CONTROL. `Int` is declared and abundant in type-argument position; if the
    // walk classified nothing at all, every percentage above would be a
    // division over an empty population reported as a clean result.
    assert!(
        occ_total > 1000,
        "classified only {occ_total} occurrences, so this profile is not over the \
         population the census measured"
    );
}

/// Sizes the CI regression, at the exact entry that caused it.
///
/// The retained-kernel gate times out at 15 minutes having completed zero of
/// 27 tests. Reproduced single-threaded, the FIRST test --
/// `decl_facts_dimensionless_projection_test::marshal_identity_is_invariant_under_reversed_source_order`
/// -- had not finished after 40 minutes. It is not a probe test and it does not
/// mention the closure: it calls `resolve_imports_transitively_with_source_roots`
/// on one fixture, twice, and builds an interpreter context over whatever comes
/// back.
///
/// Under imports that was the fixture's declared dependencies. Under
/// reference-derived assembly it is whatever the reference walk closes over,
/// and this reports that number so the regression is a measured size rather
/// than an inference from a timeout. A timeout alone cannot distinguish a large
/// closure from an unrelated hang.
#[test]
fn the_specimen_entry_that_stalls_ci_has_this_closure_size() {
    let ws = workspace_root();
    let rel = "dag/test/fixture/decl_facts_reflection/specimens.dag";
    let path = ws.join(rel);
    let content = std::fs::read_to_string(&path).expect("specimen fixture must exist");

    let (index, unparsed, _e) = build();
    assert!(unparsed.is_empty(), "index refuses: {unparsed:?}");

    let (closure, pulls) =
        v1_compiler::source_closure::closure_for_entry_attributed(rel, &content, &index);
    eprintln!(
        "[ci-stall] specimens.dag closes over {} of {} modules",
        closure.len(),
        index.module_count()
    );
    eprintln!("[ci-stall] top pulling names: {pulls:?}");

    // Deliberately no upper bound. Asserting a ceiling here would make this a
    // second copy of the width probe's requirement, and that probe already
    // carries it and is already red. This test's job is to report the number
    // that turns "CI times out" into "CI compiles N modules per test", so the
    // only thing it must refuse is a closure so small it could not explain the
    // stall -- which would mean the stall is something else and I have
    // attributed it wrongly.
    assert!(
        closure.len() > 100,
        "specimens.dag closes over only {} modules, which does NOT explain a \
         40-minute stall -- the CI failure is something other than closure width \
         and this attribution is wrong",
        closure.len()
    );
}

/// Prices the #8262 typechecker factor on a CHEAP subject (tidy-pike-117's
/// route, 2026-08-15).
///
/// #8262 added refusals at argument and field sites: +569 lines in
/// `v1_compiler_infer.rs`, +343 in `04_infer.dag`. Its cost is per-module and
/// per-site, applying to every subject the typechecker touches -- so it is NOT
/// specific to the 40-minute entry, and isolating it does not require running
/// that entry twice. A per-module multiplicative factor is measurable on any
/// subject, and the cheapest one is the right one.
///
/// This half of the pair is the timing on a small fixed entry at THIS head. The
/// other half is the same entry compiled by A TREE PREDATING aa1a3a8cd2 --
/// stated as the property rather than as a position, because that is the only
/// thing the measurement depends on. `64ebefa7416` satisfies it today (measured:
/// one commit before the wall, and `git merge-base --is-ancestor aa1a3a8cd2
/// 64ebefa7416` is false), and the SHA is carried here as a convenience beside
/// the requirement, never as the requirement itself.
///
/// An earlier revision of this comment called that SHA "the fork base". That
/// label was FALSE for this lane -- this branch's merge-base with main is
/// 0ed10d7decb, since main was merged in to clear a real conflict -- while the
/// SHA remained valid, because the property the ratio needs was never
/// fork-position at all. A position copied from someone else's history decays
/// the moment your own history moves, silently, with nobody touching it: the
/// positional-citation class applied to a commit.
///
/// Run by checking out such a tree, not from inside this test, because a test
/// cannot compile itself against a different compiler.
///
/// WHAT THIS DOES NOT ESTABLISH, and the reason it asserts nothing about the
/// number: one timing is not a factor. It is one operand of a ratio whose other
/// operand lives in a different tree, and quoting it alone would be a
/// measurement with one side missing.
#[test]
fn typecheck_cost_on_a_small_fixed_entry() {
    use crate::helpers::diagnostic_messages;
    use v1_compiler::v1_compiler_artifact::RenderTarget;
    use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};

    // Deliberately self-contained: no corpus, no closure, no index. The subject
    // must hold the CLOSURE factor fixed so the only thing a cross-tree ratio
    // can be attributed to is the typechecker. An entry that pulled modules
    // would vary both factors at once and measure neither.
    const ENTRY: &str = "module probe.cheap_subject\n\
        \n\
        type Pair { left: Int, right: Int }\n\
        \n\
        fn mk(a: Int, b: Int) -> Pair { Pair { left: a, right: b } }\n\
        \n\
        fn sum(p: Pair) -> Int { p.left + p.right }\n\
        \n\
        fn go() -> Int { sum(p: mk(a: 1, b: 2)) }\n";

    let started = std::time::Instant::now();
    let mut runs = 0;
    let mut diags: Vec<String> = Vec::new();
    while runs < 50 {
        let sources: Vec<std::rc::Rc<SourceFile>> = vec![std::rc::Rc::new(SourceFile {
            path: "cheap.dag".to_string(),
            content: ENTRY.to_string(),
        })];
        let result = compile_sources(std::rc::Rc::new(sources.into()), RenderTarget::Rust);
        diags = diagnostic_messages(&result);
        runs += 1;
    }
    let elapsed = started.elapsed();

    eprintln!(
        "[8262-cost] {} compiles of the cheap subject in {:?} ({:?} each) | head {}",
        runs,
        elapsed,
        elapsed / runs,
        option_env!("GITHUB_SHA").unwrap_or("local")
    );
    eprintln!("[8262-cost] diagnostics on final run: {diags:?}");

    // THE CONTROL. A subject that fails to compile is not exercising the
    // typechecker's argument- and field-site paths -- which are exactly what
    // #8262 touches -- so a fast time would mean "refused early", not "cheap".
    // Timing a broken fixture across two trees would compare two error paths
    // and report the ratio as a typechecker cost.
    assert!(
        diags.is_empty(),
        "the cheap subject must compile clean or its timing measures an error \
         path rather than the argument/field checking #8262 changed: {diags:?}"
    );
}
