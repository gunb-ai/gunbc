// THE FIXTURE-BOUNDARY RED FOR THE INGESTION-TIME DECLARATION INDEX.
//
// DESIGN §4b requires this question to be asked BEFORE the check is written: is the
// forbidden state authorable anywhere the check can run? And it is explicit that the
// CORPUS boundary does not decide it — the FIXTURE boundary does. That distinction is the
// whole reason this file exists. Measured on the live tree, all three arms are green:
// every top-level lens carries `construction_justification`, every import member resolves,
// every in-namespace citation resolves. A check that is green over the corpus and whose
// red cannot be authored anywhere would be a decoration — permanently green by
// construction, carrying no information, and worse than absent because it would be cited
// as coverage.
//
// It is not one, and this file is the receipt. `run_dag_parse_sweep` takes a directory of
// AUTHORED SOURCE, so a fixture hands the index whatever module text it likes and reads
// back a typed, located refusal. Every arm below plants its own violation and requires the
// exact refusal, beside a positive control that must stay clean.
//
// THE ONE PREMISE THAT IS MEASURED RATHER THAN INFERRED. Arm 1 is only worth anything if
// v1's own `MissingExport` does not already cover it. It does — inside a COMPILE CLOSURE.
// `orphan_import_claim_is_refused_here_and_nowhere_else` holds both directions: the same
// false claim in a module the resolver sees (v1 refuses; the index agrees) and in a module
// no closure reaches (v1 is structurally silent, because it is never asked; the index
// refuses alone). Without both arms the exhibited cost behind this change would be an
// inference about the compiler rather than an observation of it.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use v1_compiler::cli_run::declaration_index::{
    citation_debt_findings_against, corpus_findings, index_findings, index_population,
    DeclarationIntegrityKind,
};
use v1_compiler::cli_run::run_dag_parse_sweep;

/// A scratch tree of our own, cleaned on the way IN so a run is deterministic whatever the
/// previous one left behind.
fn scratch_root(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gunbc_decl_index_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("probe_root")).expect("scratch tree");
    dir
}

/// Author one `.dag` file into the fixture root.
fn author(dir: &Path, basename: &str, source: &str) {
    std::fs::write(dir.join("probe_root").join(basename), source).expect("fixture source");
}

/// The index's findings over an authored fixture tree. Panics on a parse refusal, because
/// a fixture that does not parse is testing the parser rather than the index.
fn findings_over(dir: &Path) -> Vec<(DeclarationIntegrityKind, String)> {
    match run_dag_parse_sweep(dir, &["probe_root"]) {
        Ok(sweep) => index_findings(&sweep.index)
            .into_iter()
            .map(|f| (f.kind, f.message))
            .collect(),
        Err(errors) => panic!("fixture must parse; sweep refused: {errors:?}"),
    }
}

/// The authority module every fixture below imports from and cites into.
const AUTHORITY: &str = "module probe.authority\n\ndata real_declaration: Bool = true\n";

// ARM 1 — import-member claim integrity, with its located refusal.
#[test]
fn import_member_absent_is_refused_and_located() {
    let dir = scratch_root("import_member");
    author(&dir, "authority.dag", AUTHORITY);
    author(
        &dir,
        "claimant.dag",
        "module probe.claimant\n\nimport probe.authority { no_such_declaration }\n\n\
         data claimant_present: Bool = true\n",
    );
    let found = findings_over(&dir);
    assert_eq!(
        found.len(),
        1,
        "exactly the planted claim must refuse, got {found:?}"
    );
    assert_eq!(found[0].0, DeclarationIntegrityKind::ImportMemberAbsent);
    assert!(
        found[0].1.contains("no_such_declaration") && found[0].1.contains("probe.authority"),
        "the refusal must name the member and the module it claimed it from: {}",
        found[0].1
    );
}

// ARM 1's positive control: the SAME shape with a real member must stay clean, so the arm
// is discriminating rather than allergic to imports.
#[test]
fn import_member_present_stays_clean() {
    let dir = scratch_root("import_member_control");
    author(&dir, "authority.dag", AUTHORITY);
    author(
        &dir,
        "claimant.dag",
        "module probe.claimant\n\nimport probe.authority { real_declaration }\n\n\
         data claimant_present: Bool = true\n",
    );
    assert_eq!(findings_over(&dir), Vec::new());
}

// ARM 1's DECLARED HOLE, HELD OPEN ON PURPOSE AND HELD COUNTABLE.
//
// `v1.03_resolve` `get_exported_names` appends every `kernel_type_set` key to every
// module's export surface, so `import m { Int }` is admitted whatever `m` declares. The
// index must admit it too — refusing source the seed compiler accepts is not this change's
// to do, and editing `get_exported_names` is `NewLanguageBehavior`, which the v1 freeze
// refuses. What the index MAY do is refuse to hide it.
//
// So this pair asserts the hole's SHAPE rather than its absence: the planted claim produces
// NO finding (that is the seed's rule, faithfully mirrored) and DOES produce a count of
// exactly one. The count is the whole point. A bare `continue` would satisfy the first
// assertion and zero the second, which is the absorbing-fallback shape DESIGN §5 names —
// the deficit's frequency zeroed by construction, so it can never rank for fixing. This
// test goes RED if anyone re-silences it, which is a red no corpus measurement could
// produce, since the corpus is exactly where the class is invisible.
#[test]
fn kernel_named_import_member_is_admitted_but_counted() {
    let dir = scratch_root("kernel_named");
    author(&dir, "authority.dag", AUTHORITY);
    author(
        &dir,
        "claimant.dag",
        "module probe.claimant\n\nimport probe.authority { Int }\n\n\
         data claimant_present: Bool = true\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    assert_eq!(
        index_findings(&sweep.index)
            .into_iter()
            .map(|f| (f.kind, f.message))
            .collect::<Vec<_>>(),
        Vec::new(),
        "the seed compiler admits a kernel-named member from any module; the index must not \
         refuse source the compiler accepts"
    );
    assert_eq!(
        index_population(&sweep.index).import_members_kernel_named,
        1,
        "admitted is not the same as unobserved: the escape must be counted, or its \
         frequency is zero by construction and the class never ranks for repair"
    );
}

// THE POSITIVE CONTROL FOR THE COUNT ITSELF. A real member must not inflate it, or the
// counter would be measuring imports rather than the escape.
#[test]
fn a_declared_member_does_not_count_as_kernel_named() {
    let dir = scratch_root("kernel_named_control");
    author(&dir, "authority.dag", AUTHORITY);
    author(
        &dir,
        "claimant.dag",
        "module probe.claimant\n\nimport probe.authority { real_declaration }\n\n\
         data claimant_present: Bool = true\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    assert_eq!(
        index_population(&sweep.index).import_members_kernel_named,
        0
    );
}

// KERNEL-NAMED AND DECLARED ARE INDEPENDENT AXES, AND THE COUNTER MUST SPLIT THEM.
//
// This is the arm that decides whether the counter measures anything. `kernel_type_set`
// has eight keys, and over `std.types` — the module 'import std.types { Int }' names —
// FIVE of them are genuinely declared there and THREE are not: `Bool`, `Secret`, `Json`,
// `Unit` and `Bytes` have real declarations, while `Int`, `String` and `Float` exist only
// as string keys of that map. So `import std.types { Bool }` is a TRUE claim and
// `import std.types { Int }` is a FALSE one, and a counter that renders them identically
// is reporting "this name appears in kernel_type_set", which needs no census.
//
// The predicate splits them by ORDER — it asks `!import_surface_has` FIRST and only then
// consults the kernel set — but an ordering is an implementation detail until something
// asserts the behaviour it produces. This test asserts it: one module declaring a
// kernel-NAMED type for real, one not, both imported under the same name, and the counter
// must answer 1 rather than 2. Reversing the two conjuncts leaves every other test in this
// file green and turns this one red.
#[test]
fn kernel_named_counter_splits_declared_from_undeclared() {
    let dir = scratch_root("kernel_named_split");
    // Declares `Unit` for real — the `std.types` `Bool` case.
    author(
        &dir,
        "declares.dag",
        "module probe.declares\n\ntype Unit = OnlyInhabitant\n",
    );
    // Declares no `Unit` — the `std.types` `Int` case.
    author(&dir, "silent.dag", AUTHORITY);
    author(
        &dir,
        "claimant.dag",
        "module probe.claimant\n\nimport probe.declares { Unit }\n\
         import probe.silent { Unit }\n\ndata claimant_present: Bool = true\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    assert_eq!(
        index_findings(&sweep.index)
            .into_iter()
            .map(|f| (f.kind, f.message))
            .collect::<Vec<_>>(),
        Vec::new(),
        "both claims are admitted by the seed compiler; neither may be a finding"
    );
    assert_eq!(
        index_population(&sweep.index).import_members_kernel_named,
        1,
        "the TRUE claim over a module that really declares `Unit` must not be counted as \
         an escape; only the FALSE one is"
    );
}

// THE DISCRIMINATING PAIR BEHIND THE EXHIBITED COST.
//
// Both modules make the identical false claim. The difference is whether any compile
// closure reaches them — and that difference is what v1's `MissingExport` is a function
// of, because `resolve_modules` only ever sees the modules it is handed. The index is a
// function of what is AUTHORED, so it answers for both.
#[test]
fn orphan_import_claim_is_refused_here_and_nowhere_else() {
    let dir = scratch_root("orphan");
    author(&dir, "authority.dag", AUTHORITY);
    author(
        &dir,
        "in_closure.dag",
        "module probe.in_closure\n\nimport probe.authority { no_such_declaration }\n\n\
         data in_closure_present: Bool = true\n",
    );
    author(
        &dir,
        "orphan.dag",
        "module probe.orphan\n\nimport probe.authority { no_such_declaration }\n\n\
         data orphan_present: Bool = true\n",
    );

    // (a) The index answers for BOTH, because both are authored.
    let found = findings_over(&dir);
    assert_eq!(found.len(), 2, "both claims must refuse, got {found:?}");
    assert!(found
        .iter()
        .all(|(kind, _)| *kind == DeclarationIntegrityKind::ImportMemberAbsent));
    assert!(
        found.iter().any(|(_, m)| m.contains("probe.orphan")),
        "the orphan's claim must be among them: {found:?}"
    );

    // (b) The compiler's own resolve answers ONLY for the closure it is handed. This is
    // not a defect in `MissingExport` — it is what a closure-scoped check IS, and it is
    // why an unreached module's claims are covered by nothing else in the repository.
    let closure_diagnostics = missing_export_names(&dir, &["authority.dag", "in_closure.dag"]);
    assert_eq!(
        closure_diagnostics,
        vec!["no_such_declaration".to_string()],
        "inside a closure the compiler already refuses; the index agrees with it"
    );
    let without_orphan = missing_export_names(&dir, &["authority.dag"]);
    assert!(
        without_orphan.is_empty(),
        "a closure that does not contain the orphan says nothing about it: {without_orphan:?}"
    );
}

/// Resolve exactly the named fixture files as one pool and return the names
/// `MissingExport` fired on. The pool IS the closure — which is the point of the control.
fn missing_export_names(dir: &Path, basenames: &[&str]) -> Vec<String> {
    let mut modules: Vec<Rc<v1_compiler::v1_std_core::Node>> = Vec::new();
    let mut indices = im::HashMap::new();
    for basename in basenames {
        let path = dir.join("probe_root").join(basename);
        let content = std::fs::read_to_string(&path).expect("fixture source");
        let key = basename.to_string();
        let tokens = v1_compiler::v1_compiler_tokenize::tokenize(content.clone(), key.clone());
        let index = v1_compiler::v1_std_core::build_newline_index(key.clone(), content);
        indices.insert(key, index);
        let parsed = v1_compiler::v1_compiler_parse::parse(tokens, Rc::new(indices.clone()));
        modules.push(parsed.module.as_ref().expect("fixture must parse").clone());
    }
    let graph = v1_compiler::v1_compiler_resolve::resolve_modules(
        Rc::new(modules.into_iter().collect()),
        Rc::new(indices),
    );
    graph
        .diagnostics
        .iter()
        .filter_map(|d| match &*d.diagnostic {
            v1_compiler::v1_std_core::CompilerDiagnostic::MissingExport { name, .. } => {
                Some(name.clone())
            }
            _ => None,
        })
        .collect()
}

// ARM 2 — the cited-symbol wall. A citation naming a declaration its module does not
// carry is §3's stale-citation class, and it refuses here.
#[test]
fn stale_citation_is_refused() {
    let dir = scratch_root("citation");
    author(&dir, "authority.dag", AUTHORITY);
    author(
        &dir,
        "citer.dag",
        "module probe.citer\n\nimport std.decl_ref { DeclarationRef, WholeDeclaration }\n\n\
         data citation: DeclarationRef = DeclarationRef {\n  \
         module_path: \"probe.authority\",\n  decl_name: \"deleted_declaration\",\n  \
         field: WholeDeclaration\n}\n",
    );
    let found = findings_over(&dir);
    assert_eq!(found.len(), 1, "exactly the stale citation, got {found:?}");
    assert_eq!(found[0].0, DeclarationIntegrityKind::CitedDeclarationAbsent);
    assert!(found[0].1.contains("deleted_declaration"), "{}", found[0].1);
}

// ARM 2's second red: a citation whose MODULE is gone. It refuses because some swept
// module declares the `probe` namespace root — the decidable line that separates a deleted
// `.dag` module from a citation naming hand-Rust, which no `.dag` namespace covers.
#[test]
fn citation_to_a_deleted_module_is_refused_and_a_foreign_namespace_is_not() {
    let dir = scratch_root("citation_module");
    author(&dir, "authority.dag", AUTHORITY);
    author(
        &dir,
        "citer.dag",
        "module probe.citer\n\nimport std.decl_ref { DeclarationRef, WholeDeclaration }\n\n\
         data deleted_module_citation: DeclarationRef = DeclarationRef {\n  \
         module_path: \"probe.deleted\",\n  decl_name: \"anything\",\n  \
         field: WholeDeclaration\n}\n\n\
         data foreign_namespace_citation: DeclarationRef = DeclarationRef {\n  \
         module_path: \"v1_compiler.cli_run\",\n  decl_name: \"run_dag_parse_sweep\",\n  \
         field: WholeDeclaration\n}\n",
    );
    let found = findings_over(&dir);
    assert_eq!(
        found.len(),
        1,
        "the deleted `probe.` module refuses; the hand-Rust citation does not, got {found:?}"
    );
    assert_eq!(found[0].0, DeclarationIntegrityKind::CitedModuleAbsent);
    assert!(found[0].1.contains("probe.deleted"), "{}", found[0].1);

    // AND THE ONE THAT IS NOT REFUSED IS COUNTED, never silently dropped: a green that
    // cannot say what it did not cover is the instrument failure DESIGN §5 names.
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    let population = index_population(&sweep.index);
    assert_eq!(population.citations, 2);
    assert_eq!(population.citations_outside_index, 1);
}

// ARM 3 — module authorship. The live corpus is 72 of 72 green, so this is the arm whose
// red MUST be authorable somewhere or the check is a decoration. It is authorable here.
#[test]
fn lens_without_construction_justification_is_refused() {
    let dir = scratch_root("authorship");
    author(
        &dir,
        "lens.dag",
        "module v2.lens.probe_fixture_lens\n\ndata some_other_declaration: Bool = true\n",
    );
    let found = findings_over(&dir);
    assert_eq!(found.len(), 1, "the unjustified lens, got {found:?}");
    assert_eq!(found[0].0, DeclarationIntegrityKind::LensAuthorshipAbsent);
    assert!(
        found[0].1.contains("v2.lens.probe_fixture_lens"),
        "{}",
        found[0].1
    );
}

// ARM 3's positive control, and it is the one that keeps the arm honest: the SAME module
// carrying the declaration must be clean, so the refusal is a function of the authorship
// fact and not of the module's name.
#[test]
fn lens_with_construction_justification_stays_clean() {
    let dir = scratch_root("authorship_control");
    author(
        &dir,
        "lens.dag",
        "module v2.lens.probe_fixture_lens\n\ndata construction_justification: Bool = true\n",
    );
    assert_eq!(findings_over(&dir), Vec::new());
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    assert_eq!(
        index_population(&sweep.index).lens_modules,
        1,
        "the green must name a nonzero denominator, or it covered nothing"
    );
}

// THE DEBT CONTRACT'S OWN DISCRIMINATING PAIR, AND WHY IT LIVES BEHIND A ROSTER PARAMETER.
//
// `review 55817` found that folding the debt arm into `index_findings` made every fixture in
// this file receive 38 findings it did not plant — the production roster joined against a
// tree of `probe.*` modules, where all 38 rows are trivially absent. That falsified the
// §4b evidence in place: the planted-red and positive-control assertions could not pass.
//
// The repair splits the arm out into `corpus_findings`, whose denominator is this repository,
// and gives the join an explicit roster so the contract's OWN red becomes authorable here for
// the first time. Both directions are planted below, because a one-sided assertion would not
// distinguish a working contract from an arm that never fires.
#[test]
fn a_debt_row_whose_citation_still_refuses_is_live() {
    let dir = scratch_root("debt_live");
    author(&dir, "authority.dag", AUTHORITY);
    author(
        &dir,
        "citer.dag",
        "module probe.citer\n\nimport std.decl_ref { DeclarationRef, WholeDeclaration }\n\n\
         data probe_citation: DeclarationRef = DeclarationRef {\n\
         \u{20}\u{20}module_path: \"probe.authority\",\n\
         \u{20}\u{20}decl_name: \"no_such_declaration\",\n\
         \u{20}\u{20}field: WholeDeclaration,\n}\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    let roster = [("probe.authority", "no_such_declaration", "")];
    assert_eq!(
        citation_debt_findings_against(&sweep.index, &roster),
        Vec::new(),
        "the citation still refuses, so the row is live and must not be reported as spent"
    );
    // And while the row stands, the citation itself is suppressed rather than double-counted.
    assert_eq!(
        index_findings(&sweep.index),
        Vec::new(),
        "an enrolled debt row suppresses its own citation's finding"
    );
}

#[test]
fn a_debt_row_whose_citation_stopped_refusing_is_spent_and_refuses() {
    let dir = scratch_root("debt_spent");
    author(&dir, "authority.dag", AUTHORITY);
    // Nothing cites the roster's target, so the row is spent.
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    let roster = [("probe.authority", "no_such_declaration", "")];
    let spent = citation_debt_findings_against(&sweep.index, &roster);
    assert_eq!(spent.len(), 1, "the spent row must refuse, got {spent:?}");
    assert_eq!(
        spent[0].kind,
        DeclarationIntegrityKind::CitationDebtRowStale
    );
    assert!(
        spent[0].message.contains("no_such_declaration"),
        "the refusal must name the spent row: {}",
        spent[0].message
    );
}

// THE REGRESSION CONTROL FOR THE SPLIT ITSELF. The arm must be absent from `index_findings`
// and present in `corpus_findings`, or the review's failure returns silently.
#[test]
fn the_debt_arm_is_corpus_scoped_not_index_scoped() {
    let dir = scratch_root("debt_scope");
    author(&dir, "authority.dag", AUTHORITY);
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    assert_eq!(
        index_findings(&sweep.index),
        Vec::new(),
        "the production debt roster must not reach a fixture tree through index_findings"
    );
    assert!(
        !corpus_findings(&sweep.index).is_empty(),
        "corpus_findings carries the debt arm, so the production roster is spent over a \
         fixture tree — which is exactly why the arm may not live in index_findings"
    );
}
