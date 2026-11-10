//! A typecheck that infers bodies only for the DEMANDED declarations must give every demanded
//! declaration exactly the typed item and function signature a full typecheck gives it, and must
//! refuse nothing the full typecheck does not. Run on a real closure from the repository: the full
//! chain and the demanded chain each thread their own module index, as `cli_run` does.
use crate::v1_compiler_infer::{self as infer, DeclarationDemand};
use crate::v1_std_core::{
    authored_name_at, build_newline_index, empty_intern_table, module_items, qualified_last_segment,
};
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
    let (sources, intern, si, modules, _graph) =
        graph_from("dag/test/fixture/demanded_scope/declaration_grain.dag");
    (sources, intern, si, modules)
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
    (sources, intern, si, graph.modules.clone(), Rc::new(graph))
}

// PRODUCTION-ROUTE EQUIVALENCE. `gunbc run` types an entry through v1.compiler.infer
// `typecheck_scoped` (src/v1/04_infer.dag @21872) with `DeclarationScope::ReachableFrom { requested }`;
// the demanded realization AND the service census both come from the graph the pass is handed
// (@21898, build_symbol_index_census over graph.modules -- the same reason a hand-assembled
// empty-symbol-index pass could see a registry the production pass does not). Handing BOTH scopes the
// SAME graph isolates the scope itself: any diagnostic the demanded pass adds that the
// EveryDeclaration pass does not is a read the realization wave dropped, not a graph difference.
fn route_messages(
    graph: &Rc<crate::v1_compiler_resolve::ModuleGraph>,
    intern: &Rc<crate::v1_std_core::InternTable>,
    si: &Rc<im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
    requested: &[&str],
) -> Vec<String> {
    let scope = if requested.is_empty() {
        Rc::new(infer::DeclarationScope::EveryDeclaration)
    } else {
        Rc::new(infer::DeclarationScope::ReachableFrom {
            requested: Rc::new(requested.iter().map(|s| s.to_string()).collect()),
        })
    };
    let pass = infer::typecheck_scoped(graph.clone(), si.clone(), intern.clone(), scope);
    pass.graph
        .diagnostics
        .iter()
        .map(|d| crate::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
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
            let (_sources, _intern, si, modules, _graph) =
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
// SERVICE-OPERATION METHOD SURFACE ON THE RUN GRAIN (#16156 second regression). A service reached
// by a BRACELESS whole-module import, whose operation the demanded entry spells only inside a
// `fn(_scope) { ... }` lambda subject, must still have its service row realized -- the reader needs
// it to answer the operation. A cut that dropped it answers `receiver type Primitive(<service>)
// establishes no method surface ... no declared frontier row admits it`, which is the refusal this
// witness fails on at main and must not raise at head. The grain is run against the corpus file
// itself, per the seed-growth precedent above (`a_row_named_only_through_a_demanded_signature_is_realized`),
// because a fixture can show the shape of a failure but not its absence.

#[test]
fn service_op_surface_spelled_only_in_a_lambda_subject_is_realized() {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            let (_sources, intern, si, modules, graph) =
                graph_from("dag/test/claim/rest_exchange_replay.dag");
            let demand = grain_demand(&modules, &["rest_status_and_body_are_persistable"], &si);
            let realization = match &*demand {
                DeclarationDemand::OnlyDeclarations { realization, .. } => realization.clone(),
                DeclarationDemand::AllDeclarations => panic!("scoped demand expected"),
            };
            let service_realized = realization.realized.contains_key("test.RestReplay");
            // CENSUS-MEMBERSHIP DISCRIMINATOR. The corpus census (build_symbol_index_census over
            // graph.modules, @04_infer:18815) is what feeds scope.service_registry via
            // service_census_registry (@19891). A `gunbc run` uses --source-root dag, so dag/test IS
            // scanned and the probe's declaring module IS in graph.modules -- so if the census CONTAINS
            // the service yet the demanded run still refuses the op, the drop is NOT census-membership
            // (directory exclusion) but the demand-driven assembly of the registry: the op entries stop
            // reaching scope.service_registry once #16156 narrowed realization. That is the fix target.
            let census = infer::build_symbol_index_census(modules.clone(), si.clone());
            let census_has_service = census.services.contains_key("test.RestReplay");
            // DECISIVE LEVER INSTRUMENT. `declaration_demanded` (@04_infer:17120) gates a non-function
            // item on `map_has(realization.realized, authored_name_at(item))`, and
            // `realized_declaration_items` (@04_infer:21234) filters the module CONTEXT on it. If the
            // service item's authored key is not in `realized` while the qualified spelling is, the item
            // is filtered out of the probe module's context, `merge_scope_from_imports` (@04_infer:545)
            // silently skips the parent entry it no longer lists, `scope.service_registry` loses the
            // OpEntry list, and the call falls to the Primitive method-pipe (@04_infer:8542). This
            // instrument prints the exact item-level verdict, so the fix targets the measured link.
            let svc_probe = modules
                .iter()
                .flat_map(|m| module_items(m.module.clone()).iter().cloned().collect::<Vec<_>>())
                .find(|it| qualified_last_segment(it.name.clone()) == "RestReplay")
                .map(|it| {
                    (
                        it.name.clone(),
                        authored_name_at(si.clone(), it.clone()),
                        infer::declaration_demanded(demand.clone(), it.clone(), si.clone()),
                        realization.realized.contains_key("RestReplay"),
                    )
                });
            let svc_found = svc_probe.is_some();
            let (svc_name, svc_authored, svc_demanded, realized_bare) =
                svc_probe.unwrap_or((String::new(), String::new(), false, false));
            // DEMANDED-VERSUS-CLOSURE EQUIVALENCE, via the PRODUCTION ENTRY POINTS (route_messages):
            // both scopes see the SAME graph, so any demanded-only refusal is the realization wave, not
            // a graph difference. The demanded chain must refuse no method surface the EveryDeclaration
            // chain does not; the service is realized, so the closure resolves the Observe op and yields
            // no surface refusal — GREEN below is the equivalence restored.
            let demanded = route_messages(&graph, &intern, &si, &["rest_status_and_body_are_persistable"]);
            let closure = route_messages(&graph, &intern, &si, &[]);
            let surface: Vec<String> = demanded
                .iter()
                .filter(|m| {
                    m.contains("RestReplay")
                        && (m.contains("method surface") || m.contains("no declared frontier row"))
                })
                .cloned()
                .collect();
            let closure_surface: Vec<String> = closure
                .iter()
                .filter(|m| {
                    m.contains("RestReplay")
                        && (m.contains("method surface") || m.contains("no declared frontier row"))
                })
                .cloned()
                .collect();
            eprintln!(
                "SERVICE_OP_GRAIN entry=rest_status_and_body_are_persistable service_realized={} census_has_service={} svc_found={} svc_name={} svc_authored={} svc_demanded={} realized_bare={} demanded_refusals={} closure_refusals={} surface={:?}",
                service_realized,
                census_has_service,
                svc_found,
                svc_name,
                svc_authored,
                svc_demanded,
                realized_bare,
                surface.len(),
                closure_surface.len(),
                surface
            );
            assert!(
                service_realized,
                "the service whose operation the demanded entry spells inside its lambda subject must be realized"
            );
            assert!(
                surface.iter().all(|m| closure_surface.contains(m)),
                "the demanded grain must refuse no method surface the full closure does not: demanded={surface:?} closure={closure_surface:?}"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

// THE BODY ARM OF THE PAIR: the SAME operation of the SAME service, reached through the SAME braceless
// whole-module import, spelled DIRECTLY in an entry body. It is the control that decides whether the
// lambda-subject position is the discriminator. Measured: BOTH this arm and the lambda arm refuse at main
// (quick-newt BB 20375b92 / a96b05f2) and both resolve at e1dd6b50, so position is NOT the axis -- the
// arm is retained as a permanent guard that a body-spelled service op also passes the demanded-vs-closure
// equivalence, so a future fix cannot fix only the lambda extent and miss the body extent.
#[test]
fn service_op_surface_spelled_in_an_entry_body_is_realized() {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            let (_sources, intern, si, _modules, graph) =
                graph_from("dag/test/fixture/demanded_scope/service_op_grain.dag");
            let demand = grain_demand(&modules, &["service_op_spelled_in_entry_body"], &si);
            let realization = match &*demand {
                DeclarationDemand::OnlyDeclarations { realization, .. } => realization.clone(),
                DeclarationDemand::AllDeclarations => panic!("scoped demand expected"),
            };
            let service_realized = realization.realized.contains_key("test.RestReplay");
            // Demanded-vs-closure through the PRODUCTION ENTRY POINTS (route_messages): same graph,
            // two scopes; a demanded-only refusal is the realization wave.
            let demanded = route_messages(&graph, &intern, &si, &["service_op_spelled_in_entry_body"]);
            let closure = route_messages(&graph, &intern, &si, &[]);
            let surface: Vec<String> = demanded
                .iter()
                .filter(|m| {
                    m.contains("RestReplay")
                        && (m.contains("method surface") || m.contains("no declared frontier row"))
                })
                .cloned()
                .collect();
            let closure_surface: Vec<String> = closure
                .iter()
                .filter(|m| {
                    m.contains("RestReplay")
                        && (m.contains("method surface") || m.contains("no declared frontier row"))
                })
                .cloned()
                .collect();
            eprintln!(
                "SERVICE_OP_GRAIN_BODY entry=service_op_spelled_in_entry_body service_realized={} demanded_refusals={} closure_refusals={} surface={:?}",
                service_realized,
                surface.len(),
                closure_surface.len(),
                surface
            );
            assert!(
                service_realized,
                "the service whose operation the demanded entry spells directly in its body must be realized"
            );
            assert!(
                surface.iter().all(|m| closure_surface.contains(m)),
                "the demanded grain must refuse no method surface the full closure does not: demanded={surface:?} closure={closure_surface:?}"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

// ---------------------------------------------------------------------------------------------
// WIRE-CONTRACT ROW ON THE RUN GRAIN (#16241). A `CoproductWireContract` data row is a lookup key the
// interpreter holds in one module while the coproduct it designates lives in another; the demanded entry
// reaches the coproduct (through `github.Pulls.Get` -> `PullRequestState`) but spells no lexeme for the
// contract row itself, so before #16241 the declaration-grain cut never realized it and the run answered
// 'no such coproduct' at the first body that decoded it. This is the DEMANDED-VERSUS-CLOSURE equivalence at
// the reader-key row: the full closure (`EveryDeclaration`) realizes every DECLARED row, so a row that is
// declared but not realized by the demanded grain is precisely a divergence the closure does not have.
// #16241's `realization_wire_contract_read_names` seeds every such row by its own shape (a data row named
// `wire_contract`, or a `CoproductWireContract` row initialising both `coproduct` and `encoding`) into the
// demanded set, restoring the equivalence. Same authority as the precedent
// `a_row_named_only_through_a_demanded_signature_is_realized`: a row only reachable through a reader's key,
// not a source lexeme, grown into the closure. RED at origin/main (the reader is absent: declared, not
// realized), GREEN at head once #16241's reader is integrated -- which is what makes it the discriminating
// witness for that fix.

#[test]
fn wire_contract_row_reached_by_the_publisher_route_is_realized() {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            let (_sources, _intern, si, modules, _graph) = graph_from("dag/gunbc/heal_publication.dag");
            let demand = grain_demand(&modules, &["heal_publication_publish_for"], &si);
            let realization = match &*demand {
                DeclarationDemand::OnlyDeclarations { realization, .. } => realization.clone(),
                DeclarationDemand::AllDeclarations => panic!("scoped demand expected"),
            };
            let row = "extdeps.github.pulls_contracts.pull_request_state_wire_contract";
            // Realization sets may key either by the full dotted path or the item's authored
            // (last-segment) name depending on which inventory fed them; probe BOTH forms so the
            // datum is a reading, not a key-format assumption.
            let row_bare = qualified_last_segment(row.to_string());
            let row_realized = realization.realized.contains_key(row)
                || realization.realized.contains_key(row_bare.as_str());
            // A declared row is realized by the full closure (EveryDeclaration realizes everything); so
            // `row_declared && !row_realized` is exactly a demanded-versus-closure divergence, not an
            // 'undeclared name' error. row_declared=true is the closure's side of the pair being GREEN.
            let row_declared =
                realization.declared.contains_key(row) || realization.declared.contains_key(row_bare.as_str());
            eprintln!(
                "WIRE_CONTRACT_GRAIN entry=heal_publication_publish_for row={row} row_realized={row_realized} row_declared={row_declared} realized_qualified={} realized_bare={} bare={}",
                realization.realized.contains_key(row),
                realization.realized.contains_key(row_bare.as_str()),
                row_bare
            );
            assert!(
                row_declared,
                "the contract row is declared in the graph the publisher route reaches (a realization gap vs the closure, not an absence)"
            );
            assert!(
                row_realized,
                "the demanded grain must realize the CoproductWireContract row the closure realizes and the interpreter decodes at publish (reader seed, #16241): {row}"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
