//! A typecheck that infers bodies only for the DEMANDED declarations must give every demanded
//! declaration exactly the typed item and function signature a full typecheck gives it, and must
//! refuse nothing the full typecheck does not. Run on a real closure from the repository: the full
//! chain and the demanded chain each thread their own module index, as `cli_run` does.
use crate::v1_compiler_infer::{self as infer, DeclarationDemand};
use crate::v1_std_core::{authored_name_at, build_newline_index, empty_intern_table, module_items};
use std::collections::HashSet;
use std::rc::Rc;

fn closure_of(entry: &str) -> Vec<Rc<crate::v1_compiler_compile::SourceFile>> {
    let root = std::env::current_dir()
        .unwrap()
        .ancestors()
        .find(|d| d.join("dag").exists() && d.join("Cargo.toml").exists())
        .unwrap()
        .to_path_buf();
    std::env::set_current_dir(&root).unwrap();
    let seed = Rc::new(crate::v1_compiler_compile::SourceFile {
        path: entry.to_string(),
        content: std::fs::read_to_string(root.join(entry)).unwrap(),
    });
    let mut v =
        super::entry_resolve::resolve_seeded_compile_closure(vec![seed], &["dag".to_string()])
            .unwrap();
    v.sort_by(|a, b| a.path.cmp(&b.path));
    v
}

#[test]
fn demanded_set_types_every_demanded_declaration_as_a_full_typecheck_does() {
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(body)
        .unwrap()
        .join()
        .unwrap();
}

fn body() {
    let sources = closure_of("dag/std/pareto.dag");
    let mut intern = empty_intern_table();
    let mut parsed = Vec::new();
    let mut si = crate::v1_rt::rc_empty_map::<String, Rc<crate::v1_std_core::NewlineIndex>>();
    for s in &sources {
        let idx = build_newline_index(s.path.clone(), s.content.clone());
        si = crate::v1_rt::rc_map_insert(si, s.path.clone(), idx.clone());
        let toks = crate::v1_compiler_tokenize::tokenize(
            s.content.clone(),
            s.path.clone(),
            crate::extdeps_languages_dag_syntax::dag_parse_environment(),
        );
        let p = crate::v1_compiler_parse::parse_with_table(
            toks,
            crate::v1_rt::rc_map_insert(
                crate::v1_rt::rc_empty_map::<String, Rc<crate::v1_std_core::NewlineIndex>>(),
                idx.file.clone(),
                idx.clone(),
            ),
            intern.clone(),
        );
        intern = p.intern_table.clone();
        parsed.push(
            p.result
                .module
                .clone()
                .unwrap_or_else(|| panic!("parse {}", s.path)),
        );
    }
    let graph = crate::v1_compiler_resolve::resolve_modules(Rc::new(parsed.into()), si.clone());
    let entry_module = graph.modules.last().unwrap().clone();
    let seeds: Vec<String> = module_items(entry_module.module.clone())
        .iter()
        .filter(|i| i.body.is_some() && !i.params.is_empty())
        .map(|i| authored_name_at(si.clone(), i.clone()))
        .take(1)
        .collect();
    assert!(!seeds.is_empty(), "entry module has no function to demand");
    let names =
        infer::demanded_declaration_names(graph.modules.clone(), Rc::new(seeds.into()), si.clone());
    let demand = Rc::new(DeclarationDemand::OnlyDeclarations {
        names: names.clone(),
        // The equivalence this test pins is #16069's BODY demand at the whole-declaration grain: an
        // inactive realization realizes everything, so nothing here is filtered by the newer cut.
        realization: infer::inactive_realization(),
    });

    let mut full = (
        crate::v1_rt::rc_empty_map(),
        crate::v1_rt::rc_empty_map::<String, Rc<infer::VariantExportSurface>>(),
        crate::v1_rt::rc_empty_map::<String, bool>(),
    );
    let mut part = full.clone();
    let (mut skipped, mut compared) = (0usize, 0usize);
    for m in graph.modules.iter() {
        let f = infer::typecheck_module(
            m.clone(),
            full.0.clone(),
            full.1.clone(),
            si.clone(),
            intern.clone(),
            crate::v1_compiler_infer_env::empty_symbol_index(),
            crate::v1_rt::rc_empty_map(),
            full.2.clone(),
        );
        let p = infer::typecheck_module_demanded(
            m.clone(),
            part.0.clone(),
            part.1.clone(),
            si.clone(),
            intern.clone(),
            crate::v1_compiler_infer_env::empty_symbol_index(),
            crate::v1_rt::rc_empty_map(),
            part.2.clone(),
            demand.clone(),
            None,
        );
        assert_eq!(f.typed.items.len(), p.typed.items.len());
        for (fi, pi) in f.typed.items.iter().zip(p.typed.items.iter()) {
            if infer::declaration_demanded(demand.clone(), fi.clone(), si.clone()) {
                assert_eq!(
                    fi,
                    pi,
                    "demanded declaration {} typed differently",
                    authored_name_at(si.clone(), fi.clone())
                );
                compared += 1;
            } else {
                assert!(pi.body.is_none());
                skipped += 1;
            }
        }
        let fe: HashSet<String> = f.diagnostics.iter().map(|d| format!("{:?}", d)).collect();
        for d in p.diagnostics.iter() {
            assert!(
                fe.contains(&format!("{:?}", d)),
                "demanded typecheck raised a diagnostic the full typecheck did not: {:?}",
                d
            );
        }
        for (st, r) in [(&mut full, &f), (&mut part, &p)] {
            let t = r.typed.clone();
            let path = authored_name_at(si.clone(), t.module.clone());
            let vs = infer::build_variant_export_surface(t.clone(), st.1.clone(), si.clone());
            st.1 = crate::v1_rt::rc_map_insert(st.1.clone(), path.clone(), vs);
            st.2 = infer::equality_table_with_module(st.2.clone(), t.clone(), si.clone());
            st.0 = crate::v1_rt::rc_map_insert(st.0.clone(), path, t);
        }
    }
    assert!(
        skipped > 0,
        "vacuous: nothing was skipped ({compared} compared)"
    );
    assert!(compared > 0);
    eprintln!("demanded typecheck: {compared} items identical, {skipped} bodies skipped");
}

// ---------------------------------------------------------------------------------------------
// DECLARATION-GRAIN REALIZATION. #16069 dropped the bodies a run cannot reach; this drops the
// signatures, type rows and module context of the declarations no demanded declaration names. The
// three tests below are the discriminators that tell that apart from a run that simply stopped
// looking: an unreached declaration that WOULD refuse must be invisible, the same declaration
// reached must refuse exactly as the whole-closure typecheck refuses it, and a read that crosses
// out of the realized set must refuse by name rather than answer 'no such declaration'.

fn grain_fixture_graph() -> (
    Vec<Rc<crate::v1_compiler_compile::SourceFile>>,
    Rc<crate::v1_std_core::InternTable>,
    Rc<im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
    Rc<im::Vector<Rc<crate::v1_compiler_resolve::ResolvedModule>>>,
) {
    graph_from("dag/test/fixture/demanded_scope/declaration_grain.dag")
}

// The same graph, from any entry: the realization witnesses below also run against real corpus
// files, because a fixture can only show the shape a failure has, not that the failure is gone.
fn graph_from(
    entry: &str,
) -> (
    Vec<Rc<crate::v1_compiler_compile::SourceFile>>,
    Rc<crate::v1_std_core::InternTable>,
    Rc<im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
    Rc<im::Vector<Rc<crate::v1_compiler_resolve::ResolvedModule>>>,
) {
    let sources = closure_of(entry);
    let mut intern = empty_intern_table();
    let mut parsed = Vec::new();
    let mut si = crate::v1_rt::rc_empty_map::<String, Rc<crate::v1_std_core::NewlineIndex>>();
    for s in &sources {
        let idx = build_newline_index(s.path.clone(), s.content.clone());
        si = crate::v1_rt::rc_map_insert(si, s.path.clone(), idx.clone());
        let toks = crate::v1_compiler_tokenize::tokenize(
            s.content.clone(),
            s.path.clone(),
            crate::extdeps_languages_dag_syntax::dag_parse_environment(),
        );
        let p = crate::v1_compiler_parse::parse_with_table(
            toks,
            crate::v1_rt::rc_map_insert(
                crate::v1_rt::rc_empty_map::<String, Rc<crate::v1_std_core::NewlineIndex>>(),
                idx.file.clone(),
                idx.clone(),
            ),
            intern.clone(),
        );
        intern = p.intern_table.clone();
        parsed.push(
            p.result
                .module
                .clone()
                .unwrap_or_else(|| panic!("parse {}", s.path)),
        );
    }
    let graph = crate::v1_compiler_resolve::resolve_modules(Rc::new(parsed.into()), si.clone());
    (sources, intern, si, graph.modules.clone())
}

fn grain_demand(
    modules: &Rc<im::Vector<Rc<crate::v1_compiler_resolve::ResolvedModule>>>,
    seeds: &[&str],
    si: &Rc<im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Rc<DeclarationDemand> {
    let seeds: Rc<im::Vector<String>> = Rc::new(
        seeds
            .iter()
            .map(|s| s.to_string())
            .collect::<im::Vector<String>>(),
    );
    let census = infer::build_symbol_index_census(modules.clone(), si.clone());
    let realization = infer::realization_of(modules.clone(), seeds.clone(), census, si.clone());
    Rc::new(DeclarationDemand::OnlyDeclarations {
        names: realization.realized.clone(),
        realization: realization.clone(),
    })
}

fn grain_typecheck(
    modules: &Rc<im::Vector<Rc<crate::v1_compiler_resolve::ResolvedModule>>>,
    intern: &Rc<crate::v1_std_core::InternTable>,
    si: &Rc<im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
    demand: Rc<DeclarationDemand>,
) -> Vec<String> {
    let mut parents = crate::v1_rt::rc_empty_map::<String, Rc<infer::TypedModule>>();
    let mut surfaces = crate::v1_rt::rc_empty_map::<String, Rc<infer::VariantExportSurface>>();
    let mut equality = crate::v1_rt::rc_empty_map::<String, bool>();
    let mut messages = Vec::new();
    for m in modules.iter() {
        let r = infer::typecheck_module_demanded(
            m.clone(),
            parents.clone(),
            surfaces.clone(),
            si.clone(),
            intern.clone(),
            crate::v1_compiler_infer_env::empty_symbol_index(),
            crate::v1_rt::rc_empty_map(),
            equality.clone(),
            demand.clone(),
            None,
        );
        for d in r.diagnostics.iter() {
            messages.push(crate::v1_std_core::diagnostic_to_message(
                d.diagnostic.clone(),
            ));
        }
        let t = r.typed.clone();
        let path = authored_name_at(si.clone(), t.module.clone());
        let vs = infer::build_variant_export_surface(t.clone(), surfaces.clone(), si.clone());
        surfaces = crate::v1_rt::rc_map_insert(surfaces.clone(), path.clone(), vs);
        equality = infer::equality_table_with_module(equality.clone(), t.clone(), si.clone());
        parents = crate::v1_rt::rc_map_insert(parents.clone(), path, t);
    }
    messages
}

fn fixture_module_messages(messages: &[String], needle: &str) -> Vec<String> {
    messages
        .iter()
        .filter(|m| m.contains(needle))
        .cloned()
        .collect()
}

/// The closure names what the entry names and nothing else: `reached_entry` pulls in its own type,
/// and neither of the two declarations no demanded declaration names is realized -- including the
/// broken one whose refusal is the whole point of leaving it out.
#[test]
fn declaration_grain_realizes_only_what_a_demanded_declaration_names() {
    let (_sources, _intern, si, modules) = grain_fixture_graph();
    let demand = grain_demand(&modules, &["reached_entry"], &si);
    let realization = match &*demand {
        DeclarationDemand::OnlyDeclarations { realization, .. } => realization.clone(),
        DeclarationDemand::AllDeclarations => panic!("scoped demand expected"),
    };
    assert!(
        realization.realized.contains_key("reached_entry"),
        "the seed is realized"
    );
    assert!(
        realization.realized.contains_key("GrainPoint"),
        "a type the entry names is realized"
    );
    assert!(
        !realization.realized.contains_key("UnreachedBrokenShape"),
        "a type nothing names must not be realized: {:?}",
        realization.realized.keys().collect::<Vec<_>>()
    );
    assert!(
        !realization.realized.contains_key("broken_entry"),
        "an unreached entry is not realized"
    );
    assert!(
        realization.declared.contains_key("UnreachedBrokenShape"),
        "the unrealized type is still DECLARED -- that is what makes crossing out a refusal, not an absence"
    );
}

/// An unreached declaration that would refuse is invisible, and the whole-closure typecheck proves
/// it really would have: the same module, the same declaration, two very different runs.
#[test]
fn declaration_grain_leaves_an_unreached_broken_declaration_unseen() {
    let (_sources, intern, si, modules) = grain_fixture_graph();
    let full = grain_typecheck(
        &modules,
        &intern,
        &si,
        Rc::new(DeclarationDemand::AllDeclarations),
    );
    let broken_full = fixture_module_messages(&full, "TypeNobodyDeclares");
    assert!(
        !broken_full.is_empty(),
        "the whole-closure typecheck must refuse the broken type at all: {full:?}"
    );
    let grain = grain_typecheck(
        &modules,
        &intern,
        &si,
        grain_demand(&modules, &["reached_entry"], &si),
    );
    assert!(
        fixture_module_messages(&grain, "TypeNobodyDeclares").is_empty(),
        "a declaration no demanded declaration names must not be realized, yet was: {grain:?}"
    );
    assert!(
        fixture_module_messages(&grain, "DeclarationRealizationGap").is_empty(),
        "realizing less must not manufacture a gap in a module that reads nothing it did not realize: {grain:?}"
    );
}

/// The reachable error still refuses: demand the entry that NAMES the broken type and the same
/// refusal arrives. A cut that also dropped errors would pass the test above and fail this one.
#[test]
fn declaration_grain_still_refuses_a_reachable_error() {
    let (_sources, intern, si, modules) = grain_fixture_graph();
    let grain = grain_typecheck(
        &modules,
        &intern,
        &si,
        grain_demand(&modules, &["broken_entry"], &si),
    );
    assert!(
        !fixture_module_messages(&grain, "TypeNobodyDeclares").is_empty(),
        "demanding the entry that names the broken type must refuse it: {grain:?}"
    );
}

/// A read outside the realized set refuses BY NAME. The audit is given the edges of a declaration
/// that names one realized type and one declaration the closure declares but this run never realized:
/// the second must come back as a located `DeclarationRealizationGap` naming the reading check, not as
/// silence and not as 'no such declaration' -- absence and unreality are different facts.
#[test]
fn a_read_outside_the_realized_set_refuses_typed() {
    let (_sources, _intern, si, modules) = grain_fixture_graph();
    let seeds: Rc<im::Vector<String>> = Rc::new(im::vector!["reached_entry".to_string()]);
    let census = infer::build_symbol_index_census(modules.clone(), si.clone());
    let realization = infer::realization_of(modules.clone(), seeds, census, si.clone());
    let edges: Rc<im::Vector<Rc<infer::RealizationEdge>>> = Rc::new(im::vector![
        Rc::new(infer::RealizationEdge {
            reader: "inhabitance_row".to_string(),
            name: "GrainPoint".to_string(),
            span: Rc::new(crate::v1_std_core::SourceSpan {
                file: "test.dag".to_string(),
                start: 0,
                end: 1
            }),
        }),
        Rc::new(infer::RealizationEdge {
            reader: "global_bare_lookup".to_string(),
            name: "UnreachedBrokenShape".to_string(),
            span: Rc::new(crate::v1_std_core::SourceSpan {
                file: "test.dag".to_string(),
                start: 0,
                end: 1
            }),
        }),
        Rc::new(infer::RealizationEdge {
            reader: "global_bare_lookup".to_string(),
            name: "NameNoModuleDeclares".to_string(),
            span: Rc::new(crate::v1_std_core::SourceSpan {
                file: "test.dag".to_string(),
                start: 0,
                end: 1
            }),
        }),
    ]);
    let diags = infer::realization_audit(
        realization,
        edges,
        "test.fixture.demanded_scope.declaration_grain".to_string(),
    );
    let messages: Vec<String> = diags
        .iter()
        .map(|d| crate::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect();
    assert_eq!(
        messages.len(),
        1,
        "exactly the declared-but-unrealized read refuses; realized reads and never-declared names stay silent: {messages:?}"
    );
    assert!(
        messages[0].contains("UnreachedBrokenShape") && messages[0].contains("global_bare_lookup"),
        "the refusal names the declaration and the reading check that asked: {}",
        messages[0]
    );
}

/// The row a READ reaches, not a name spells. `grain_alias_entry` names only the alias; the record it is
/// declared to be lives in the item's `inferred` node, outside the extent `spelled_names_into` walks. The
/// type reader reads that right-hand side anyway, so `realization_row_edges` declares the edge and the
/// closure realizes the row. With a seed that never names the alias the same row stays unrealized, which
/// is what makes the alias -- and not a name some file listed -- the thing that reached it.
#[test]
fn a_row_reached_only_through_an_alias_body_counts_reachable() {
    let (_sources, intern, si, modules) = grain_fixture_graph();
    let through_alias = grain_demand(&modules, &["grain_alias_entry"], &si);
    let realization = match &*through_alias {
        DeclarationDemand::OnlyDeclarations { realization, .. } => realization.clone(),
        DeclarationDemand::AllDeclarations => panic!("scoped demand expected"),
    };
    assert!(
        realization.realized.contains_key("GrainAliasRow"),
        "the alias the entry spells is realized"
    );
    assert!(
        realization.realized.contains_key("GrainHiddenRow"),
        "the alias body is a row the type reader reads, so it must be realized: {:?}",
        realization.realized.keys().collect::<Vec<_>>()
    );
    let elsewhere = grain_demand(&modules, &["reached_entry"], &si);
    let plain = match &*elsewhere {
        DeclarationDemand::OnlyDeclarations { realization, .. } => realization.clone(),
        DeclarationDemand::AllDeclarations => panic!("scoped demand expected"),
    };
    assert!(
        !plain.realized.contains_key("GrainHiddenRow"),
        "nothing reaches that row without the alias"
    );
    assert!(
        !plain.realized.contains_key("GrainAliasRow"),
        "and nothing reaches the alias either"
    );
    let messages = grain_typecheck(&modules, &intern, &si, through_alias);
    assert!(
        fixture_module_messages(&messages, "DeclarationRealizationGap").is_empty(),
        "reading the alias must not manufacture a gap for the row it expands to: {messages:?}"
    );
}

/// THE SILENT MISS, WITNESSED AT ITS REAL SIZE. `fetch_secret_ref_credential`'s return row
/// `SecretCredentialFetch` names `SmResolvedVersionIdentityRefusal` in one of its arms, two modules
/// away, and nothing else in the run's demand set spells either name. A cut that skipped them did not
/// refuse -- the run answered `unresolved type`, which is the failure this mechanism exists to make
/// impossible. Pinned against the corpus files themselves, because a fixture can show the shape of a
/// failure but not its absence.
#[test]
fn a_row_named_only_through_a_demanded_signature_is_realized() {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            let (_sources, _intern, si, modules) =
                graph_from("dag/gunbc/auth/secret_ref_credential.dag");
            let demand = grain_demand(&modules, &["fetch_secret_ref_credential"], &si);
            let realization = match &*demand {
                DeclarationDemand::OnlyDeclarations { realization, .. } => realization.clone(),
                DeclarationDemand::AllDeclarations => panic!("scoped demand expected"),
            };
            assert!(
                realization.realized.contains_key("SecretCredentialFetch"),
                "the return row of a demanded signature is realized"
            );
            assert!(
                realization
                    .realized
                    .contains_key("SmResolvedVersionIdentityRefusal"),
                "the row an arm of that return row names, two modules away, is realized"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

// ---------------------------------------------------------------------------------------------
// HOST CENSUS SEEDING. The run host reads wire-contract rows it does not spell: the interpreter's
// coproduct index enumerates every `CoproductWireContract` row by SHAPE and its module-encoding
// resolver reads each module's `wire_contract` default by that host key. A declaration-grain cut
// that realizes only what demanded declarations name hides those rows from the host scan, and the
// REST decode then answers `no CoproductWireContract row names this coproduct` for a row that
// exists -- a SILENT MISS of the kind gunbc.design_argument::fail_closed forbids. The census
// (@ref(v1.compiler.emit_core_support::declaration_is_host_census_row), seeded into the realization by
// @ref(v1.compiler.infer::census_realized_names), which follows the whole alias chain like the host's
// resolve_module_encoding) is the declared edge from those reads to the rows; these tests pin that
// the census is real in both directions: census rows are realized though nothing demands them,
// and an ordinary undemanded data declaration stays unrealized (the census is not a blanket rule).

#[test]
fn grain_seeds_the_host_wire_census_so_the_decode_index_resolves_actor_spelling() {
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(host_census_body)
        .unwrap()
        .join()
        .unwrap();
}

fn host_census_body() {
    let (_sources, intern, si, modules) = graph_from("dag/extdeps/github/workflow_runs.dag");
    let seed_fn = "github_workflow_run_id_is_positive";
    let census = infer::build_symbol_index_census(modules.clone(), si.clone());
    let realization = infer::realization_of(
        modules.clone(),
        Rc::new(im::vector![seed_fn.to_string()]),
        census,
        si.clone(),
    );
    // Positive control: the grant tool's GetRun decode reads `run.actor.type` at
    // (extdeps.github.github, UserType). No demanded declaration names this row; the census must.
    assert!(
        realization.realized.contains_key("user_type_wire_contract"),
        "the CoproductWireContract row for UserType must be seeded by the host census"
    );
    assert!(
        realization
            .realized
            .contains_key("workflow_run_status_wire_contract"),
        "the ListWorkflowRuns decode rows must be seeded by the host census too"
    );
    assert!(
        realization.realized.contains_key("wire_contract"),
        "the module `wire_contract` default is read by the host (resolve_module_encoding)"
    );
    // Negative control: an ordinary data declaration in the same closure that no demanded
    // declaration names and no host census reads stays UNREALIZED. If this ever realizes, the
    // census became a blanket rule and the cut stopped being a cut.
    assert!(
        !realization
            .realized
            .contains_key("extdeps_external_authority_anchor"),
        "host-census seeding must not re-widen realization to all data declarations"
    );
    // The index the REST decode consults, built exactly the way the interpreter builds it.
    let demand = Rc::new(DeclarationDemand::OnlyDeclarations {
        names: realization.realized.clone(),
        realization: realization.clone(),
    });
    let mut typed_map = crate::v1_rt::rc_empty_map::<String, Rc<infer::TypedModule>>();
    let mut surfaces = crate::v1_rt::rc_empty_map::<String, Rc<infer::VariantExportSurface>>();
    let mut eq = crate::v1_rt::rc_empty_map::<String, bool>();
    for m in modules.iter() {
        let r = infer::typecheck_module_demanded(
            m.clone(),
            typed_map.clone(),
            surfaces.clone(),
            si.clone(),
            intern.clone(),
            crate::v1_compiler_infer_env::empty_symbol_index(),
            crate::v1_rt::rc_empty_map(),
            eq.clone(),
            demand.clone(),
            None,
        );
        let path = authored_name_at(si.clone(), r.typed.module.clone());
        let vs = infer::build_variant_export_surface(r.typed.clone(), surfaces.clone(), si.clone());
        surfaces = crate::v1_rt::rc_map_insert(surfaces, path.clone(), vs);
        eq = infer::equality_table_with_module(eq.clone(), r.typed.clone(), si.clone());
        typed_map = crate::v1_rt::rc_map_insert(typed_map, path, r.typed.clone());
    }
    let typed: Vec<Rc<infer::TypedModule>> = typed_map.values().cloned().collect();
    let graph = crate::v1_compiler_infer_items::ResolvedGraph {
        modules: Rc::new(typed.into()),
        item_registry: Rc::new(im::HashMap::new()),
        item_leaf_owner_modules: Rc::new(im::HashMap::new()),
        diagnostics: Rc::new(im::vector![]),
    };
    let ctx = crate::v1_interpreter::InterpContext::new(
        &graph,
        si.clone(),
        crate::v1_interpreter::ExecutionMode::Hermetic,
    );
    let index = crate::v1_interpreter::build_coproduct_wire_contract_index(&ctx);
    assert!(
        index
            .encodings
            .contains_key(&("extdeps.github.github".to_string(), "UserType".to_string())),
        "the decode reads the UserType spelling; an index that cannot see the row is a silent miss"
    );
    assert!(
        index.encodings.contains_key(&(
            "extdeps.github.workflow_runs".to_string(),
            "WorkflowRunStatus".to_string()
        )) || index.encodings.contains_key(&(
            "extdeps.github.github".to_string(),
            "WorkflowRunStatus".to_string()
        )),
        "the list decode reads the WorkflowRunStatus spelling; its row must be indexed too"
    );
}

// THE ALIAS CHAIN, END TO END. v1_interpreter::resolve_module_encoding follows `data x = other`
// aliases recursively (fuel 32). A census that seeded only wire_contract's first alias hop would
// leave a later link unrealized, and the host would then refuse a row that EXISTS by blaming the
// next link ("aliases ..., which resolves nowhere") -- the silent miss this census exists to
// prevent, dressed as a located error. The fixture pairs the chain with an undemanded, uncensused
// data declaration that must stay unrealized, so the seed stays a census and not a blanket rule.
#[test]
fn grain_seeds_every_link_of_a_wire_contract_alias_chain() {
    let (_sources, _intern, si, modules) =
        graph_from("dag/test/fixture/demanded_scope/wire_contract_alias_chain.dag");
    let census = infer::build_symbol_index_census(modules.clone(), si.clone());
    let realization = infer::realization_of(
        modules.clone(),
        Rc::new(im::vector!["demanded_entry".to_string()]),
        census,
        si.clone(),
    );
    for link in [
        "wire_contract",
        "alias_left",
        "alias_right",
        "grain_row_wire_contract",
    ] {
        assert!(
            realization.realized.contains_key(link),
            "{link} must be seeded by the host census (alias chain and row), got {:?}",
            realization.realized.keys().collect::<Vec<_>>()
        );
    }
    assert!(
        !realization
            .realized
            .contains_key("unreached_sidecar_note"),
        "an undemanded, uncensused data declaration must stay unrealized: the census is a census, not the old blanket rule"
    );
}
