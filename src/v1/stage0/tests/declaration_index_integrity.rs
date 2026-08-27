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
    citation_debt_findings_against, citation_debt_findings_named, cited_symbol_findings_against,
    corpus_findings, index_findings, index_get, index_population, planted_control_findings_against,
    DeclarationIndex, DeclarationIntegrityKind, ModuleDeclarationRecord,
};
use v1_compiler::cli_run::{
    compile_dag_multi_module_fixture, run_dag_parse_sweep, MultiModuleCompileFixtureOutcome,
};

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

// ── PLANT WELL-FORMEDNESS: the precondition every planted red owes before its verdict ──
//
// WHY THESE EXIST, and it is a finding rather than tidiness. Two fixture defects in this file
// produced EXACTLY the observation a broken guard produces:
//
//   wrong module header      -> import target absent -> claim SKIPPED  -> no finding
//   single-variant coproduct -> parse question       -> not admitted   -> no finding
//
// A MALFORMED PLANT AND A BROKEN GUARD ARE INDISTINGUISHABLE AT THE ASSERTION. So "no finding"
// is a three-way ambiguity — fixture malformed, plant never reached, guard broken — and the
// repair pressure points at the production predicate, which is how a correct guard nearly got
// "fixed" to satisfy a bad fixture. That is the design doc's not-applicable-versus-malformed
// conflation arrived at from the fixture side: SKIPPED and ADMITTED are different states and
// the assertion could not see the difference.
//
// The remedy is a positive control ON THE PLANT, asserted BEFORE the guard's verdict, so the
// guard's answer is the only remaining variable.

/// The plant is in the index at the identity the fixture intended — i.e. it parsed AND
/// declared the module path the rest of the test names.
fn plant<'a>(index: &'a DeclarationIndex, module_path: &str) -> &'a ModuleDeclarationRecord {
    index_get(index, module_path).unwrap_or_else(|| {
        panic!(
            "PLANT MALFORMED: no module `{module_path}` in the index. The fixture did not \
             parse, or its `module` header names something else — this is not a verdict \
             about the guard."
        )
    })
}

/// The plant declares the name the fixture is about, so a later "resolves" is a real answer.
fn plant_declares(index: &DeclarationIndex, module_path: &str, name: &str) {
    let record = plant(index, module_path);
    assert!(
        record.declared.contains(name) || record.variants.contains(name),
        "PLANT MALFORMED: `{module_path}` was authored to declare `{name}` and the index does \
         not see it (declared: {:?}, variants: {:?}) — not a verdict about the guard",
        record.declared,
        record.variants
    );
}

/// The import claim REACHED the admit/refuse decision rather than being skipped because its
/// target is absent from the index. This is the one that hid a fixture bug behind a zero.
fn plant_import_target_resolves(index: &DeclarationIndex, importer: &str, target: &str) {
    let record = plant(index, importer);
    assert!(
        record.imports.iter().any(|c| c.target == target),
        "PLANT MALFORMED: `{importer}` was authored to import from `{target}` and no such \
         claim is indexed — not a verdict about the guard"
    );
    assert!(
        index_get(index, target).is_some(),
        "PLANT NEVER REACHED: `{importer}` imports `{target}`, which is absent from the \
         index, so the claim is SKIPPED rather than judged. A guard that never ran and a \
         guard that found nothing are different states."
    );
}

/// The citation is indexed at the identity the fixture intended.
fn plant_cites(index: &DeclarationIndex, citer: &str, cited_module: &str, cited_decl: &str) {
    let record = plant(index, citer);
    assert!(
        record
            .cited
            .iter()
            .any(|c| c.module_path == cited_module && c.decl_name == cited_decl),
        "PLANT MALFORMED: `{citer}` was authored to cite `{cited_module}` `{cited_decl}` and \
         the extractor did not index it (cited: {:?}) — not a verdict about the guard",
        record.cited
    );
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
    // PRECONDITION: the claim reached the decision rather than being skipped as target-absent.
    plant_import_target_resolves(&sweep.index, "probe.claimant", "probe.authority");
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
        "module probe.declares\n\ntype Unit = OnlyInhabitant | OtherInhabitant\n",
    );
    // Declares no `Unit` — the `std.types` `Int` case.
    author(
        &dir,
        "silent.dag",
        "module probe.silent\n\ndata real_declaration: Bool = true\n",
    );
    author(
        &dir,
        "claimant.dag",
        "module probe.claimant\n\nimport probe.declares { Unit }\n\
         import probe.silent { Unit }\n\ndata claimant_present: Bool = true\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    // PRECONDITION: both plants are well-formed and both claims REACHED the decision. Without
    // this, a wrong module header makes the claim target-absent, the counter reads 0, and the
    // failure is indistinguishable from a broken predicate — which is what happened.
    plant_declares(&sweep.index, "probe.declares", "Unit");
    plant_import_target_resolves(&sweep.index, "probe.claimant", "probe.declares");
    plant_import_target_resolves(&sweep.index, "probe.claimant", "probe.silent");
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

// ARM 2's THIRD RED, AND IT HAD NO FIXTURE UNTIL 2026-08-26. `CitedFieldAbsent` was the one
// refusal arm of the cited-symbol wall with no controlled fixture anywhere: its only evidence
// was a PLANTED_CONTROL_CITATIONS row whose citation was authored inside
// `v2.lens.cited_symbol_resolution`. Deleting that lens deleted the citation, the row stopped
// reproducing, and the arm would have been left with nothing executing against it — the
// §4b(4) failure of deleting evidence along with the machinery it outlived. Measured, not
// assumed: before this test the string `CitedFieldAbsent` did not occur in this file at all.
//
// A controlled fixture is also the STRONGER replacement, not merely an equal one. The planted
// row asserted that one hand-authored citation in the live corpus still refuses; this authors
// both the input and the expected population, which is the oracle DESIGN §5 actually asks for.
//
// THE PAIR IS THE POINT: `absent_field` must refuse and `present_field` must NOT, in one
// fixture. A wall that refused every `NamedField` citation would satisfy the red alone.
#[test]
fn citation_to_an_absent_field_is_refused_and_a_present_field_is_not() {
    let dir = scratch_root("citation_field");
    author(
        &dir,
        "authority.dag",
        "module probe.authority\n\ndata real_declaration: Bool = true\n\n\
         type Carrier {\n  present_field: Bool\n}\n",
    );
    author(
        &dir,
        "citer.dag",
        "module probe.citer\n\nimport std.decl_ref { DeclarationRef, NamedField }\n\n\
         data absent_field_citation: DeclarationRef = DeclarationRef {\n  \
         module_path: \"probe.authority\",\n  decl_name: \"Carrier\",\n  \
         field: NamedField { field_name: \"no_such_field\" }\n}\n\n\
         data present_field_citation: DeclarationRef = DeclarationRef {\n  \
         module_path: \"probe.authority\",\n  decl_name: \"Carrier\",\n  \
         field: NamedField { field_name: \"present_field\" }\n}\n",
    );
    let found = findings_over(&dir);
    assert_eq!(
        found.len(),
        1,
        "the absent field refuses; the present one does not, got {found:?}"
    );
    assert_eq!(found[0].0, DeclarationIntegrityKind::CitedFieldAbsent);
    assert!(found[0].1.contains("no_such_field"), "{}", found[0].1);
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

// DESIGN's third emit-stage escape mode is a JOIN, not a specimen count: a module is in
// the retained population when an ordinary module cites it as an authority and no import
// edge reaches it. Both directions are planted so a collector that merely counts citations,
// or one that mistakes deliberately-false fixture text for authority demand, cannot pass.
#[test]
fn cited_authority_without_an_import_edge_is_named_until_an_edge_reaches_it() {
    let dir = scratch_root("cited_authority_reachability");
    author(&dir, "authority.dag", AUTHORITY);
    author(
        &dir,
        "citer.dag",
        "module probe.citer\n\nimport std.decl_ref { DeclarationRef, WholeDeclaration }\n\n\
         data citation: DeclarationRef = DeclarationRef {\n  \
         module_path: \"probe.authority\",\n  decl_name: \"real_declaration\",\n  \
         field: WholeDeclaration\n}\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    assert_eq!(
        index_population(&sweep.index).cited_authorities_without_import_edges,
        vec!["probe.authority"],
        "the retained population carries its identity, not only one counted specimen"
    );

    author(
        &dir,
        "consumer.dag",
        "module probe.consumer\n\nimport probe.authority { real_declaration }\n\n\
         data consumes_authority: Bool = real_declaration\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    assert_eq!(
        index_population(&sweep.index).cited_authorities_without_import_edges,
        Vec::<String>::new(),
        "one real import edge reaches and therefore removes the authority from the remainder"
    );
}

#[test]
fn retained_authorities_partition_by_a_call_to_the_cited_declaration() {
    let dir = scratch_root("cited_authority_call_partition");
    author(
        &dir,
        "invoked.dag",
        "module probe.invoked\n\nfn probe_call() -> Bool { true }\n",
    );
    author(
        &dir,
        "data_only.dag",
        "module probe.data_only\n\ndata authority: Bool = true\n",
    );
    author(
        &dir,
        "citer.dag",
        "module probe.citer\n\nimport std.decl_ref { DeclarationRef, WholeDeclaration }\n\n\
         data called_ref: DeclarationRef = DeclarationRef { module_path: \"probe.invoked\", decl_name: \"probe_call\", field: WholeDeclaration }\n\n\
         data data_ref: DeclarationRef = DeclarationRef { module_path: \"probe.data_only\", decl_name: \"authority\", field: WholeDeclaration }\n\n\
         fn invokes() -> Bool { probe_call() }\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    let population = index_population(&sweep.index);
    assert_eq!(
        population.cited_and_called_without_import_edges,
        vec!["probe.invoked"]
    );
    assert_eq!(
        population.cited_not_called_without_import_edges,
        vec!["probe.data_only"]
    );
}

#[test]
fn fixture_citation_does_not_claim_authority_reachability() {
    let dir = scratch_root("fixture_citation_reachability");
    author(&dir, "authority.dag", AUTHORITY);
    author(
        &dir,
        "citer_test.dag",
        "module probe.citer_test\n\nimport std.decl_ref { DeclarationRef, WholeDeclaration }\n\n\
         data citation: DeclarationRef = DeclarationRef {\n  \
         module_path: \"probe.authority\",\n  decl_name: \"real_declaration\",\n  \
         field: WholeDeclaration\n}\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    assert_eq!(
        index_population(&sweep.index).cited_authorities_without_import_edges,
        Vec::<String>::new(),
        "fixture text is evidence input, not a claim that its target is a live authority"
    );
}

// The live credentials defect is intentionally NOT this control: another change may repair it,
// which must not make the evidence permanently green. These two caller-authored modules keep the
// compiler's refusing and accepting directions under the fixture's control.
#[test]
fn orphan_entry_compile_has_a_controlled_red_and_clean_control() {
    let bad = compile_dag_multi_module_fixture(
        &["probe_bad.dag".to_string()],
        &["module probe.bad\n\nfn broken() -> Bool { missing_value }\n".to_string()],
        "probe_bad.dag",
    );
    assert!(
        matches!(bad, MultiModuleCompileFixtureOutcome::CompileRefused { .. }),
        "the controlled orphan with an undefined value must refuse, got {bad:?}"
    );

    let clean = compile_dag_multi_module_fixture(
        &["probe_clean.dag".to_string()],
        &["module probe.clean\n\ndata okay: Bool = true\n".to_string()],
        "probe_clean.dag",
    );
    assert!(
        matches!(
            clean,
            MultiModuleCompileFixtureOutcome::CompileCompleted { .. }
        ),
        "the controlled clean orphan must compile, got {clean:?}"
    );
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
// this file receive 42 findings it did not plant — the production roster joined against a
// tree of `probe.*` modules, where all 42 rows are trivially absent. That falsified the
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
    // PRECONDITION: the citation was extracted at all. A citation the extractor missed is
    // indistinguishable from one the roster suppressed.
    plant_cites(
        &sweep.index,
        "probe.citer",
        "probe.authority",
        "no_such_declaration",
    );
    let roster = [(
        "probe.citer",
        "probe_citation",
        "probe.authority",
        "no_such_declaration",
        "",
    )];
    assert_eq!(
        citation_debt_findings_against(&sweep.index, &roster),
        Vec::new(),
        "the citation still refuses, so the row is live and must not be reported as spent"
    );
    // And while the row stands, the citation itself is suppressed rather than double-counted.
    assert_eq!(
        cited_symbol_findings_against(&sweep.index, &roster),
        Vec::new(),
        "an enrolled debt row suppresses its own citation's finding"
    );
    // The SAME tree with an EMPTY roster must refuse, or the suppression above is not
    // suppression — it is the wall failing to judge the citation at all.
    assert_eq!(
        cited_symbol_findings_against(&sweep.index, &[]).len(),
        1,
        "unenrolled, the citation must refuse; otherwise the enrolled case proves nothing"
    );
}

#[test]
fn a_debt_row_whose_citation_stopped_refusing_is_spent_and_refuses() {
    let dir = scratch_root("debt_spent");
    author(&dir, "authority.dag", AUTHORITY);
    // Nothing cites the roster's target, so the row is spent.
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    let roster = [(
        "probe.citer",
        "probe_citation",
        "probe.authority",
        "no_such_declaration",
        "",
    )];
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

// THE CLASS, NOT THE INSTANCE.
//
// `review 55817` found ONE arm reading a module-scope roster from inside itself. Asking the
// same question of the other four found a second, and a worse one: `cited_symbol_findings`
// SUPPRESSED citations against the same constant. A spurious refusal is loud; a spurious
// SUPPRESSION is a citation the wall quietly declines to judge, so that arm could have
// stopped enrolling an entire class with every fixture in this file still green.
//
// Both rosters are parameters now, and the default is EMPTY rather than the production
// constant, so `index_findings` judges every citation in whatever tree it is handed. This
// test is the control on that default: a fixture citation that the production roster happens
// to name must still be judged by `index_findings`, because the production roster is not a
// fact about a fixture tree. It reds if either default is ever pointed back at the constant.
#[test]
fn the_production_roster_does_not_reach_an_arbitrary_tree() {
    let dir = scratch_root("roster_default");
    // `std.bytes` `builtin_function_registry` is a real row of PRE_EXISTING_CITATION_DEBT.
    author(
        &dir,
        "citer.dag",
        "module probe.citer\n\nimport std.decl_ref { DeclarationRef, WholeDeclaration }\n\n\
         data probe_citation: DeclarationRef = DeclarationRef {\n\
         \u{20}\u{20}module_path: \"probe.absent\",\n\
         \u{20}\u{20}decl_name: \"builtin_function_registry\",\n\
         \u{20}\u{20}field: WholeDeclaration,\n}\n",
    );
    author(
        &dir,
        "absent.dag",
        "module probe.absent\n\ndata other: Bool = true\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    let found = index_findings(&sweep.index);
    assert_eq!(
        found.len(),
        1,
        "index_findings must judge this citation on its own tree, got {found:?}"
    );
    assert_eq!(
        found[0].kind,
        DeclarationIntegrityKind::CitedDeclarationAbsent
    );
}

// THE SEAM THE REPAIR ITSELF CREATED.
//
// Both rosters now enter from exactly one place, `corpus_findings`. That is the right shape,
// and it makes the WIRING a fact that needs its own evidence: unreachable-from-`index_findings`
// is proven by construction, but reachable-from-`corpus_findings` is otherwise just an
// assertion about a call site. Without this test the suppression arm would be perfectly
// evidenced at the fixture boundary and silently disableable at the only seam that matters —
// pass `&[]` there instead of the production roster and nothing fails.
//
// The discriminating pair is authorable because a fixture may reproduce a production SITE.
// `std.encoding` citing `std.bytes` `builtin_function_registry` is a real row of
// `PRE_EXISTING_CITATION_DEBT`, so a fixture that declares `std.bytes` WITHOUT that
// declaration and cites it FROM A MODULE CALLING ITSELF `std.encoding` must be refused by
// `index_findings` (empty roster: judge everything) and suppressed by `corpus_findings`
// (production roster: enrolled). The two answers over ONE tree are what prove the wiring
// rather than the fold.
//
// THE CITING MODULE IS PART OF THE FIXTURE NOW, and that is not incidental: a row exempts a
// SITE, so naming the target alone no longer reaches the production roster. The companion
// test below plants the same target from a different citer and requires it to REFUSE.
#[test]
fn corpus_findings_is_wired_to_the_production_suppression_roster() {
    let dir = scratch_root("seam_suppression");
    author(
        &dir,
        "bytes.dag",
        "module std.bytes\n\ndata something_else: Bool = true\n",
    );
    author(
        &dir,
        "citer.dag",
        "module std.encoding\n\nimport std.decl_ref { DeclarationRef, WholeDeclaration }\n\n\
         data utf8_decode_bytes_host_realization_marker: DeclarationRef = DeclarationRef {\n\
         \u{20}\u{20}module_path: \"std.bytes\",\n\
         \u{20}\u{20}decl_name: \"builtin_function_registry\",\n\
         \u{20}\u{20}field: WholeDeclaration,\n}\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    // PRECONDITION: the citation is indexed at the exact identity the production roster names,
    // and the cited module really is present and really does NOT declare it. Otherwise a
    // "suppressed" reading below could just be a citation nobody extracted.
    plant_cites(
        &sweep.index,
        "std.encoding",
        "std.bytes",
        "builtin_function_registry",
    );
    assert!(
        !plant(&sweep.index, "std.bytes")
            .declared
            .contains("builtin_function_registry"),
        "PLANT MALFORMED: std.bytes must NOT declare the cited name, or there is nothing to \
         refuse and nothing to suppress"
    );

    let unenrolled: Vec<_> = index_findings(&sweep.index)
        .into_iter()
        .filter(|f| f.kind == DeclarationIntegrityKind::CitedDeclarationAbsent)
        .collect();
    assert_eq!(
        unenrolled.len(),
        1,
        "with no roster the citation must be judged and refused, got {unenrolled:?}"
    );

    let enrolled: Vec<_> = corpus_findings(&sweep.index)
        .into_iter()
        .filter(|f| f.kind == DeclarationIntegrityKind::CitedDeclarationAbsent)
        .collect();
    assert_eq!(
        enrolled,
        Vec::new(),
        "corpus_findings must pass the PRODUCTION roster to the citation arm, which enrolls \
         this exact identity; passing an empty roster here would leave the suppression arm \
         perfectly tested at the fixture boundary and silently disabled at the one seam that \
         matters"
    );
}

// THE PLANTED-CONTROL ARM READS THE SAME TRIGGER THE DEBT ARM DOES, IN THE OPPOSITE DIRECTION,
// and that inversion is the only reason it is a second carrier rather than a flag on the first.
// Both directions are planted, because a one-sided assertion here cannot tell a working arm
// from one that never fires.
#[test]
fn a_planted_control_that_still_refuses_is_healthy() {
    let dir = scratch_root("control_healthy");
    author(&dir, "authority.dag", AUTHORITY);
    author(
        &dir,
        "citer.dag",
        "module probe.citer\n\nimport std.decl_ref { DeclarationRef, WholeDeclaration }\n\n\
         data probe_citation: DeclarationRef = DeclarationRef {\n\
         \u{20}\u{20}module_path: \"probe.authority\",\n\
         \u{20}\u{20}decl_name: \"deliberately_absent_RED\",\n\
         \u{20}\u{20}field: WholeDeclaration,\n}\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    plant_cites(
        &sweep.index,
        "probe.citer",
        "probe.authority",
        "deliberately_absent_RED",
    );
    let roster = [(
        "probe.citer",
        "probe_citation",
        "probe.authority",
        "deliberately_absent_RED",
        "",
    )];
    assert_eq!(
        planted_control_findings_against(&sweep.index, &roster),
        Vec::new(),
        "a control that still refuses is doing its job and must not be reported"
    );
}

#[test]
fn a_planted_control_that_resolves_has_lost_its_power_and_refuses() {
    let dir = scratch_root("control_lost");
    // The control's target now EXISTS, so the citation resolves and the control is spent.
    author(
        &dir,
        "authority.dag",
        "module probe.authority\n\ndata deliberately_absent_RED: Bool = true\n",
    );
    author(
        &dir,
        "citer.dag",
        "module probe.citer\n\nimport std.decl_ref { DeclarationRef, WholeDeclaration }\n\n\
         data probe_citation: DeclarationRef = DeclarationRef {\n\
         \u{20}\u{20}module_path: \"probe.authority\",\n\
         \u{20}\u{20}decl_name: \"deliberately_absent_RED\",\n\
         \u{20}\u{20}field: WholeDeclaration,\n}\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    // PRECONDITION: the target really is declared now, or "resolves" is a fixture accident.
    plant_declares(&sweep.index, "probe.authority", "deliberately_absent_RED");
    plant_cites(
        &sweep.index,
        "probe.citer",
        "probe.authority",
        "deliberately_absent_RED",
    );
    let roster = [(
        "probe.citer",
        "probe_citation",
        "probe.authority",
        "deliberately_absent_RED",
        "",
    )];
    let lost = planted_control_findings_against(&sweep.index, &roster);
    assert_eq!(lost.len(), 1, "the spent control must refuse, got {lost:?}");
    assert_eq!(
        lost[0].kind,
        DeclarationIntegrityKind::PlantedControlNoLongerRefuses
    );
}

// THE DESYNCHRONIZATION IS ASSERTED HERE RATHER THAN OBSERVED IN A REPORT.
//
// The first corpus run reported two CONTRADICTORY findings about ONE citation:
// `extdeps.tcgplayer.store` `UpdateSkuPrice` was called unenrolled debt by the suppression arm
// AND its roster row was called spent by the staleness arm. Cause: the citation carries
// `field: NamedField { "price" }` and its roster row carried an empty field, so the two arms
// were matching on different identities. Both arms were locally correct. Both were wrong.
//
// NEITHER ARM CAN DETECT THAT ALONE — each is right about its own half — so the only observable
// is the two answers being present together and disagreeing. That was caught by a human reading
// two lines of a report, which is not a mechanism. This test makes it a wall: plant the exact
// identity mismatch and require BOTH findings, then repair the row and require NEITHER.
#[test]
fn a_roster_row_on_the_wrong_identity_desynchronizes_both_arms() {
    let dir = scratch_root("desync");
    author(
        &dir,
        "authority.dag",
        "module probe.authority\n\ndata real_declaration: Bool = true\n",
    );
    author(
        &dir,
        "citer.dag",
        "module probe.citer\n\nimport std.decl_ref { DeclarationRef, NamedField }\n\n\
         data probe_citation: DeclarationRef = DeclarationRef {\n\
         \u{20}\u{20}module_path: \"probe.authority\",\n\
         \u{20}\u{20}decl_name: \"absent_declaration\",\n\
         \u{20}\u{20}field: NamedField { field_name: \"price\" },\n}\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    plant_cites(
        &sweep.index,
        "probe.citer",
        "probe.authority",
        "absent_declaration",
    );

    // THE MISMATCH: the row names the same declaration with NO field, the citation has one.
    let mismatched = [(
        "probe.citer",
        "probe_citation",
        "probe.authority",
        "absent_declaration",
        "",
    )];
    let unsuppressed = cited_symbol_findings_against(&sweep.index, &mismatched);
    let spent = citation_debt_findings_against(&sweep.index, &mismatched);
    assert_eq!(
        unsuppressed.len(),
        1,
        "arm 1 must still refuse the citation, because the row does not name its identity: \
         {unsuppressed:?}"
    );
    assert_eq!(
        spent.len(),
        1,
        "arm 2 must call the row spent, because no live citation carries the row's identity: \
         {spent:?}"
    );
    assert_eq!(
        spent[0].kind,
        DeclarationIntegrityKind::CitationDebtRowStale,
        "the two findings together are the desynchronization signal: one says this citation is \
         unenrolled, the other says its enrollment is obsolete, about the same citation in the \
         same run"
    );

    // THE REPAIR: the row on the citation's real identity silences both arms at once.
    let matched = [(
        "probe.citer",
        "probe_citation",
        "probe.authority",
        "absent_declaration",
        "price",
    )];
    assert_eq!(
        cited_symbol_findings_against(&sweep.index, &matched),
        Vec::new(),
        "enrolled at the right identity, the citation is suppressed"
    );
    assert_eq!(
        citation_debt_findings_against(&sweep.index, &matched),
        Vec::new(),
        "and the row is live, not spent — both arms agree only when the identity matches"
    );
}

// ARM 8 — a fixture carrier's citations are JUDGED, and the exemption is per citation.
//
// The removed form skipped every citation in a module `module_is_fixture_carrier` answered
// true for. These three tests are the discriminating evidence that carrier identity no longer
// suppresses anything by itself: a refusing citation inside a witness module is an ordinary
// finding, an enumerated exemption suppresses exactly its own identity, and an exemption whose
// citation resolves refuses as spent. Without the third, the roster could rot into a list of
// things that used to be false — the same inverse arm the debt roster carries.

#[test]
fn a_refusing_citation_inside_a_fixture_carrier_is_judged() {
    let dir = scratch_root("fixture_citation_judged");
    author(&dir, "authority.dag", AUTHORITY);
    author(
        &dir,
        "witness_test.dag",
        "module probe.witness\n\nimport std.decl_ref { DeclarationRef, WholeDeclaration }\n\n\
         data cite: DeclarationRef = DeclarationRef {\n\u{20}\u{20}module_path: \
         \"probe.authority\",\n\u{20}\u{20}decl_name: \"no_such_declaration\",\n\u{20}\u{20}\
         field: WholeDeclaration,\n}\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    // PRECONDITION 1: the module really is classified as a fixture carrier, or this test
    // proves nothing about the class it is named for.
    let record = plant(&sweep.index, "probe.witness");
    assert!(
        record.is_fixture_carrier,
        "PLANT MALFORMED: `probe.witness` must be a fixture carrier for this arm to bite"
    );
    // PRECONDITION 2: the citation was extracted.
    plant_cites(
        &sweep.index,
        "probe.witness",
        "probe.authority",
        "no_such_declaration",
    );
    // THE VERDICT: unexempted, a fixture carrier's refusing citation is an ordinary finding.
    let findings = cited_symbol_findings_against(&sweep.index, &[]);
    assert_eq!(
        findings.len(),
        1,
        "a refusing citation in a fixture carrier must be judged, got {findings:?}"
    );
    assert_eq!(
        findings[0].kind,
        DeclarationIntegrityKind::CitedDeclarationAbsent
    );
}

#[test]
fn a_fixture_citation_exemption_suppresses_only_its_own_identity() {
    let dir = scratch_root("fixture_citation_exempt");
    author(&dir, "authority.dag", AUTHORITY);
    author(
        &dir,
        "witness_test.dag",
        "module probe.witness\n\nimport std.decl_ref { DeclarationRef, WholeDeclaration }\n\n\
         data planted: DeclarationRef = DeclarationRef {\n\u{20}\u{20}module_path: \
         \"probe.authority\",\n\u{20}\u{20}decl_name: \"deliberately_absent\",\n\u{20}\u{20}\
         field: WholeDeclaration,\n}\n\ndata stale: DeclarationRef = DeclarationRef {\n\
         \u{20}\u{20}module_path: \"probe.authority\",\n\u{20}\u{20}decl_name: \
         \"accidentally_absent\",\n\u{20}\u{20}field: WholeDeclaration,\n}\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    plant_cites(
        &sweep.index,
        "probe.witness",
        "probe.authority",
        "deliberately_absent",
    );
    plant_cites(
        &sweep.index,
        "probe.witness",
        "probe.authority",
        "accidentally_absent",
    );
    // Both refuse with no roster — the denominator this test measures against.
    assert_eq!(
        cited_symbol_findings_against(&sweep.index, &[]).len(),
        2,
        "both planted citations must refuse unenrolled"
    );
    // Exempting ONE leaves the OTHER refusing. This is the whole content of the repair: the
    // exemption is keyed on the citation, so it cannot cover a sibling in the same module.
    let roster = [(
        "probe.witness",
        "planted",
        "probe.authority",
        "deliberately_absent",
        "",
    )];
    let findings = cited_symbol_findings_against(&sweep.index, &roster);
    assert_eq!(
        findings.len(),
        1,
        "exempting one citation must not shield its sibling, got {findings:?}"
    );
    assert!(
        findings[0].message.contains("accidentally_absent"),
        "the surviving finding must be the UNEXEMPTED one, got {:?}",
        findings[0].message
    );
}

#[test]
fn a_fixture_exemption_whose_citation_resolves_is_spent_and_refuses() {
    let dir = scratch_root("fixture_exempt_spent");
    author(&dir, "authority.dag", AUTHORITY);
    author(
        &dir,
        "witness_test.dag",
        "module probe.witness\n\nimport std.decl_ref { DeclarationRef, WholeDeclaration }\n\n\
         data cite: DeclarationRef = DeclarationRef {\n\u{20}\u{20}module_path: \
         \"probe.authority\",\n\u{20}\u{20}decl_name: \"real_declaration\",\n\u{20}\u{20}\
         field: WholeDeclaration,\n}\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    // PRECONDITION: the citation RESOLVES — `real_declaration` is declared by AUTHORITY. A
    // roster row over a resolving citation is exactly a spent row.
    plant_cites(
        &sweep.index,
        "probe.witness",
        "probe.authority",
        "real_declaration",
    );
    assert_eq!(
        cited_symbol_findings_against(&sweep.index, &[]),
        Vec::new(),
        "PLANT MALFORMED: `real_declaration` must resolve for this to be a SPENT row"
    );
    let roster = [(
        "probe.witness",
        "cite",
        "probe.authority",
        "real_declaration",
        "",
    )];
    let spent = citation_debt_findings_named(&sweep.index, &roster, "FIXTURE_CARRIER_EXEMPTIONS");
    assert_eq!(
        spent.len(),
        1,
        "an exemption over a resolving citation must refuse as spent, got {spent:?}"
    );
    assert_eq!(
        spent[0].kind,
        DeclarationIntegrityKind::CitationDebtRowStale
    );
    assert!(
        spent[0].message.contains("FIXTURE_CARRIER_EXEMPTIONS"),
        "the diagnostic must name the roster holding the row, got {:?}",
        spent[0].message
    );
}

// THE CLASS THIS CHANGE CLOSES, AND THE ONLY TEST THAT DISTINGUISHES SITE GRAIN FROM TARGET
// GRAIN.
//
// Every roster row used to name a TARGET — `(module, decl, field)` — so one row suppressed
// every citation of that target, corpus-wide and for as long as the row stood. Measured on the
// live tree, the 70 rows covered 75 refusing sites and six targets were already cited from more
// than one module (`gunbc.host_effect` `host_effect_apply` from three, `std.bytes`
// `builtin_function_registry` from three). The consequence is the one this test plants: a patch
// could author a BRAND NEW dangling citation to any enrolled target, from any module, and the
// wall would silently decline to judge it — citation rot admitted by the mechanism built to
// refuse it, and decidable from the patch alone.
//
// The fixture reproduces one production row exactly (`std.encoding` -> `std.bytes`
// `builtin_function_registry`) and plants a SECOND citation of the same target from
// `probe.newcomer`. Under the production roster the enrolled site is suppressed and the new one
// must refuse. Revert the roster to target grain and this test goes green with zero findings,
// which is precisely the state it exists to forbid.
#[test]
fn a_new_citation_of_an_enrolled_target_from_another_module_still_refuses() {
    let dir = scratch_root("site_grain_newcomer");
    author(
        &dir,
        "bytes.dag",
        "module std.bytes\n\ndata something_else: Bool = true\n",
    );
    author(
        &dir,
        "enrolled.dag",
        "module std.encoding\n\nimport std.decl_ref { DeclarationRef, WholeDeclaration }\n\n\
         data utf8_decode_bytes_host_realization_marker: DeclarationRef = DeclarationRef {\n\
         \u{20}\u{20}module_path: \"std.bytes\",\n\
         \u{20}\u{20}decl_name: \"builtin_function_registry\",\n\
         \u{20}\u{20}field: WholeDeclaration,\n}\n",
    );
    author(
        &dir,
        "newcomer.dag",
        "module probe.newcomer\n\nimport std.decl_ref { DeclarationRef, WholeDeclaration }\n\n\
         data fresh_citation: DeclarationRef = DeclarationRef {\n\
         \u{20}\u{20}module_path: \"std.bytes\",\n\
         \u{20}\u{20}decl_name: \"builtin_function_registry\",\n\
         \u{20}\u{20}field: WholeDeclaration,\n}\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    // PRECONDITIONS: both citations are indexed, and the target really is absent — otherwise
    // "one finding" below could be a citation nobody extracted rather than a wall that fired.
    plant_cites(
        &sweep.index,
        "std.encoding",
        "std.bytes",
        "builtin_function_registry",
    );
    plant_cites(
        &sweep.index,
        "probe.newcomer",
        "std.bytes",
        "builtin_function_registry",
    );
    assert_eq!(
        cited_symbol_findings_against(&sweep.index, &[]).len(),
        2,
        "unenrolled, both citations must refuse; otherwise the enrolled case proves nothing"
    );

    let found: Vec<_> = corpus_findings(&sweep.index)
        .into_iter()
        .filter(|f| f.kind == DeclarationIntegrityKind::CitedDeclarationAbsent)
        .collect();
    assert_eq!(
        found.len(),
        1,
        "the enrolled SITE is suppressed and the new one is not, got {found:?}"
    );
    assert!(
        found[0].message.contains("probe.newcomer"),
        "the surviving finding must be the UNENROLLED citer, got {:?}",
        found[0].message
    );
}

// THE SAME DISTINCTION AT THE ARM BOUNDARY, so the site grain is not an accident of one
// production row. A roster row exempting `probe.citer` may not shield an identical citation
// authored in `probe.other`.
#[test]
fn a_roster_row_exempts_its_own_citer_and_no_other() {
    let dir = scratch_root("site_grain_arm");
    author(&dir, "authority.dag", AUTHORITY);
    for (basename, module) in [("citer.dag", "probe.citer"), ("other.dag", "probe.other")] {
        let source = format!(
            "module {module}\n\nimport std.decl_ref {{ DeclarationRef, WholeDeclaration }}\n\ndata cite: DeclarationRef = DeclarationRef {{\n  module_path: \"probe.authority\",\n  decl_name: \"no_such_declaration\",\n  field: WholeDeclaration,\n}}\n"
        );
        author(&dir, basename, &source);
    }
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    plant_cites(
        &sweep.index,
        "probe.citer",
        "probe.authority",
        "no_such_declaration",
    );
    plant_cites(
        &sweep.index,
        "probe.other",
        "probe.authority",
        "no_such_declaration",
    );
    let roster = [(
        "probe.citer",
        "cite",
        "probe.authority",
        "no_such_declaration",
        "",
    )];
    let findings = cited_symbol_findings_against(&sweep.index, &roster);
    assert_eq!(
        findings.len(),
        1,
        "the row exempts one site, not the target, got {findings:?}"
    );
    assert!(
        findings[0].message.contains("probe.other"),
        "the surviving finding must be the unenrolled citer, got {:?}",
        findings[0].message
    );
    // And the row is LIVE, not spent — its own citation still refuses.
    assert_eq!(
        citation_debt_findings_against(&sweep.index, &roster),
        Vec::new(),
        "the staleness arm must key on the same site the suppression arm does"
    );
}

// THE RESIDUE THIS BRANCH FIRST DISCLOSED AND THEN CLOSED (review 56227).
//
// The citing MODULE alone was not a site: two citations of one target inside one module shared
// a row, so a new dangling citation authored BESIDE an enrolled one stayed suppressed — the
// same fail-open the module grain closed, one level in, and the reviewer was right that a
// disclosed residue is not a closed one when the identity that closes it is available.
//
// It was available. `record_from_module` already iterates top-level items to build `declared`,
// so the enclosing declaration's name costs one string at extraction and is a NAME rather than
// an offset — reachable from the containment tree, and therefore not the positional citation
// DESIGN §3 forbids.
//
// The fixture is the reviewer's scenario exactly: one module, one enrolled citation, and a
// second dangling citation of the SAME target in a DIFFERENT declaration. Under the module
// grain this test reports zero findings, which is the state it exists to forbid.
#[test]
fn a_second_citation_of_an_enrolled_target_in_another_declaration_still_refuses() {
    let dir = scratch_root("decl_grain_sibling");
    author(&dir, "authority.dag", AUTHORITY);
    author(
        &dir,
        "citer.dag",
        "module probe.citer\n\nimport std.decl_ref { DeclarationRef, WholeDeclaration }\n\n\
         data enrolled: DeclarationRef = DeclarationRef {\n\
         \u{20}\u{20}module_path: \"probe.authority\",\n\
         \u{20}\u{20}decl_name: \"no_such_declaration\",\n\
         \u{20}\u{20}field: WholeDeclaration,\n}\n\n\
         data authored_later: DeclarationRef = DeclarationRef {\n\
         \u{20}\u{20}module_path: \"probe.authority\",\n\
         \u{20}\u{20}decl_name: \"no_such_declaration\",\n\
         \u{20}\u{20}field: WholeDeclaration,\n}\n",
    );
    let sweep = run_dag_parse_sweep(&dir, &["probe_root"]).expect("fixture must parse");
    // PRECONDITION: both citations are indexed, and each is attributed to its OWN declaration.
    // Without this, one finding below could be one citation the extractor missed.
    let record = plant(&sweep.index, "probe.citer");
    let homes: Vec<&str> = record
        .cited
        .iter()
        .map(|c| c.in_declaration.as_str())
        .collect();
    assert_eq!(
        homes,
        vec!["enrolled", "authored_later"],
        "PLANT MALFORMED: each citation must be attributed to the declaration that carries it"
    );
    assert_eq!(
        cited_symbol_findings_against(&sweep.index, &[]).len(),
        2,
        "unenrolled, both citations must refuse"
    );

    let roster = [(
        "probe.citer",
        "enrolled",
        "probe.authority",
        "no_such_declaration",
        "",
    )];
    let findings = cited_symbol_findings_against(&sweep.index, &roster);
    assert_eq!(
        findings.len(),
        1,
        "the sibling citation must still refuse — a row exempts one declaration's citation, not \
         the module's, got {findings:?}"
    );
    assert_eq!(
        findings[0].kind,
        DeclarationIntegrityKind::CitedDeclarationAbsent
    );
    // And the row is LIVE, not spent, so the staleness arm reads the same declaration-grained
    // identity the suppression arm does.
    assert_eq!(
        citation_debt_findings_against(&sweep.index, &roster),
        Vec::new(),
        "both inverse arms must key on the declaration too, or they desynchronize"
    );
}
