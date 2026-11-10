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
    Rc<crate::v1_compiler_resolve::ModuleGraph>,
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
    (sources, intern, si, graph.modules.clone(), graph)
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

// The same production route, returning the PASS so the arm can read the typed state the demanded
// scope BUILT -- items, item_registry, func_env locals -- not only the diagnostics it emitted.
// This is the instrument that separates 'the realization filter dropped the service item' from
// 'the item survived and a later projection never consulted it'.
fn route_pass(
    graph: &Rc<crate::v1_compiler_resolve::ModuleGraph>,
    intern: &Rc<crate::v1_std_core::InternTable>,
    si: &Rc<im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
    requested: &[&str],
) -> Rc<infer::TypecheckPass> {
    let scope = if requested.is_empty() {
        Rc::new(infer::DeclarationScope::EveryDeclaration)
    } else {
        Rc::new(infer::DeclarationScope::ReachableFrom {
            requested: Rc::new(requested.iter().map(|s| s.to_string()).collect()),
        })
    };
    infer::typecheck_scoped(graph.clone(), si.clone(), intern.clone(), scope)
}

// Print the typed state a scoped pass holds for the two modules the service-op witness spans:
// the fixture that DECLARES test.RestReplay and the claim module whose entry SPELLS the call.
// For each: every item's authored name, kind, transport presence and child count (the exact
// predicates merge_scope_from_imports gates on, src/v1/04_infer.dag @545-585), the item_registry
// keys, and the func_env local/parent view keys (func_env_views_for_import @19901: an is_all
// import brings the parent's WHOLE view; a named list filters -- if the service root binding is
// demanded-absent while closure-present, this is where it shows).
fn grain_probe(
    tag: &str,
    pass: &Rc<infer::TypecheckPass>,
    si: &Rc<im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) {
    fn keys_of(
        fe: &Rc<crate::v1_compiler_infer_sigs::ResolvedFuncEnv>,
        depth: usize,
        out: &mut Vec<String>,
    ) {
        let mut ks: Vec<String> = fe.local.keys().cloned().collect();
        ks.sort();
        out.push(format!("d{depth}:{}[{}]", fe.name, ks.join(",")));
        if depth < 2 {
            for parent in fe.parents.iter() {
                keys_of(parent, depth + 1, out);
            }
        }
    }
    for tm in pass.graph.modules.iter() {
        let name = tm.module.name.clone();
        if !(name.contains("rest_exchange_replay_probe")
            || name.contains("github_pull_request_decode_witness"))
        {
            continue;
        }
        let items: Vec<String> = tm
            .items
            .iter()
            .map(|it| {
                format!(
                    "{}|{:?}|t{}|c{}",
                    authored_name_at(si.clone(), it.clone()),
                    it.module_item_kind,
                    if it.transport.is_some() { "Y" } else { "n" },
                    it.children.len()
                )
            })
            .collect();
        let mut reg: Vec<String> = tm.item_registry.keys().cloned().collect();
        reg.sort();
        let mut fe_keys = Vec::new();
        keys_of(&tm.func_env, 0, &mut fe_keys);
        eprintln!(
            "GRAIN_PROBE tag={tag} module={name} items={items:?} registry={reg:?} funcenv={fe_keys:?}",
            tag = tag,
            name = name,
            items = items,
            reg = reg,
            fe_keys = fe_keys
        );
    }
    let svc_reg: Vec<String> = pass
        .graph
        .item_registry
        .keys()
        .filter(|k| k.contains("RestReplay") || k.contains("PullRequestPage"))
        .cloned()
        .collect();
    eprintln!(
        "GRAIN_PROBE tag={tag} graph_registry={svc_reg:?}",
        tag = tag,
        svc_reg = svc_reg
    );
}

// TIER-1 REPLAY. check_service_method_call_node (the tier that establishes a service operation's
// method surface, src/v1/04_service.dag @581) is GATED on the receiver node's SHAPE:
// connective == NoConnective AND children.is_empty(), and keyed on authored_name_at(receiver).
// The grain probes show the census registry reaches both scopes complete; the remaining suspect
// is the receiver NODE the demanded checker saw -- a differently-shaped typed node (children
// non-empty, a connective, or an authored name that is not `test.RestReplay`) silently skips
// tier 1 and falls to method_existence_decision, which is exactly the observed refusal text.
// This walks the TYPED entry body of each pass, captures every node touching the call
// (rest_replay / test.RestReplay / RestReplay / PullRequestPage), prints its shape and its
// stored inference, and REPLAYS the exported tier-1 function against the census registry.
fn tier1_replay(
    tag: &str,
    pass: &Rc<infer::TypecheckPass>,
    graph: &Rc<crate::v1_compiler_resolve::ModuleGraph>,
    intern: &Rc<crate::v1_std_core::InternTable>,
    si: &Rc<im::HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) {
    fn walk(
        n: &Rc<crate::v1_std_core::Node>,
        depth: usize,
        hits: &mut Vec<Rc<crate::v1_std_core::Node>>,
    ) {
        if depth > 12 || hits.len() > 200 {
            return;
        }
        if n.name.contains("RestReplay")
            || n.name.contains("rest_replay")
            || n.name == "PullRequestPage"
        {
            hits.push(n.clone());
        }
        for c in n.children.iter() {
            walk(c, depth + 1, hits);
        }
        if let Some(b) = &n.body {
            walk(b, depth + 1, hits);
        }
        if let Some(t) = &n.type_annotation {
            walk(t, depth + 1, hits);
        }
    }
    // The SAME authorities the production typecheck_scoped builds @21963/@21984.
    let census = infer::build_symbol_index_census(graph.modules.clone(), si.clone());
    let env = infer::type_env_over_symbol_index(
        infer::compiler_kernel_type_env(si.clone(), intern.clone()),
        census.clone(),
    );
    let reg = infer::service_census_registry(env);
    let mut reg_keys: Vec<String> = reg.keys().cloned().collect();
    reg_keys.sort();
    let mut items_walked = 0usize;
    for tm in pass.graph.modules.iter() {
        if !tm
            .module
            .name
            .contains("github_pull_request_decode_witness")
        {
            continue;
        }
        for item in tm.items.iter() {
            // authored_name_at returns the BARE name for claim-module fns (the GRAIN_PROBE line
            // at 58e3f601: "decoded_pull_request|ModuleItemFunction|tn|c0") -- compare the LAST
            // SEGMENT; a dotted composite silently skips every item.
            if qualified_last_segment(authored_name_at(si.clone(), item.clone()))
                != "decoded_pull_request"
            {
                continue;
            }
            items_walked += 1;
            let mut hits = Vec::new();
            walk(item, 0, &mut hits);
            for h in hits {
                let inf = h
                    .inferred
                    .as_deref()
                    .map(|i| match i {
                        crate::v1_std_core::InferredNode::Resolved { node, .. } => {
                            format!("{}#{}c", node.name, node.children.len())
                        }
                        other => format!("{other:?}"),
                    })
                    .unwrap_or_else(|| "-".to_string());
                let replay = infer::check_service_method_call_node(
                    h.clone(),
                    "PullRequestPage".to_string(),
                    reg.clone(),
                    si.clone(),
                );
                eprintln!(
                    "TIER1_REPLAY tag={tag} node={node} conn={conn:?} children={children} auth={auth} inferred={inf} replay={replay}",
                    tag = tag,
                    node = h.name,
                    conn = h.connective,
                    children = h.children.len(),
                    auth = authored_name_at(si.clone(), h.clone()),
                    replay = if replay.is_some() { "PRESENT" } else { "ABSENT" },
                );
            }
        }
    }
    eprintln!(
        "TIER1_REPLAY tag={tag} items_walked={items_walked} census_registry_keys={reg_keys:?}",
        tag = tag,
        items_walked = items_walked,
        reg_keys = reg_keys
    );
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
            assert!(
                closure_surface.is_empty(),
                "the full-closure grain must refuse no service method surface: closure={closure_surface:?}"
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
            let (_sources, intern, si, modules, graph) =
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
            assert!(
                closure_surface.is_empty(),
                "the full-closure grain must refuse no service method surface: closure={closure_surface:?}"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

// ---------------------------------------------------------------------------------------------
// THE WITNESS-ENTRY ARM: the caebcfaa case, verbatim. The two arms above run on the harness fixture
// `service_op_grain.dag`; a fixture shows the shape a failure has but cannot answer a real-route
// observation with a different entry. BuildBuddy caebcfaa-a82c-4e44-af06-f455d4d27635 refused
// `test.RestReplay.PullRequestPage()` at dag/test/claim/github_pull_request_decode_witness.dag:96 --
// braceless whole-module import of the probe module (:57), the call inside the `fn(_scope) { ... }`
// subject of `evaluate_in_witness_frame` (:90), reached through the ENTRY `decoded_pull_request`
// -- and resolved at 5e315131's parent e1dd6b50. This arm drives THAT entry, in THAT file, through
// the same production equivalence: the demanded grain must refuse no method surface the full closure
// does not. The file's own header separates the two refusals it documents: the frame's missing
// RUNTIME realization on the run route exists at every commit and is not this witness; what regressed
// at 5e315131 is the TYPECHECK-stage method surface, which is what `typecheck_scoped` reports.

#[test]
fn service_op_surface_in_the_pull_request_decode_witness_entry_is_realized() {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            let (_sources, intern, si, modules, graph) =
                graph_from("dag/test/claim/github_pull_request_decode_witness.dag");
            let demand = grain_demand(&modules, &["decoded_pull_request"], &si);
            let realization = match &*demand {
                DeclarationDemand::OnlyDeclarations { realization, .. } => realization.clone(),
                DeclarationDemand::AllDeclarations => panic!("scoped demand expected"),
            };
            let service_realized = realization.realized.contains_key("test.RestReplay")
                || realization.realized.contains_key("RestReplay");
            let demanded = route_messages(&graph, &intern, &si, &["decoded_pull_request"]);
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
            // SIMULATED FIX MEASUREMENT: does SEEDING THE SERVICE (the reader's key) close the gap?
            // Both key forms are tried; the one that zeroes the demanded refusals is the key form the
            // registry's service_method_surface seed must emit. An unknown seed name is inert in the
            // realization closure, so a wrong form costs only a pass, not a false reading.
            let with_bare = route_messages(&graph, &intern, &si, &["decoded_pull_request", "RestReplay"]);
            let with_dotted =
                route_messages(&graph, &intern, &si, &["decoded_pull_request", "test.RestReplay"]);
            let count_surface = |ms: &Vec<String>| {
                ms.iter()
                    .filter(|m| {
                        m.contains("RestReplay")
                            && (m.contains("method surface") || m.contains("no declared frontier row"))
                    })
                    .count()
            };
            let realized_bare = realization.realized.contains_key("RestReplay");
            let realized_dotted = realization.realized.contains_key("test.RestReplay");
            // CENSUS SHAPE at the witness graph: the census is computed identically in both scopes
            // (@21984, from graph.modules) -- so if it carries the service under the dotted key WITH
            // the operation children, the demanded-scope refusal is the projection never consulting
            // it; if the entry or its children are absent, the census is the broken link.
            let census = infer::build_symbol_index_census(graph.modules.clone(), si.clone());
            let census_keys: Vec<String> = census
                .services
                .keys()
                .filter(|k| k.contains("RestReplay"))
                .cloned()
                .collect();
            let census_ops: Vec<String> = census
                .services
                .get("test.RestReplay")
                .map(|e| {
                    e.item
                        .children
                        .iter()
                        .map(|c| authored_name_at(si.clone(), c.clone()))
                        .collect()
                })
                .unwrap_or_default();
            eprintln!(
                "SERVICE_OP_GRAIN_WITNESS entry=decoded_pull_request service_realized={} realized_bare={} realized_dotted={} demanded_refusals={} closure_refusals={} seed_bare_refusals={} seed_dotted_refusals={} census_keys={:?} census_ops={:?} surface={:?}",
                service_realized,
                realized_bare,
                realized_dotted,
                surface.len(),
                closure_surface.len(),
                count_surface(&with_bare),
                count_surface(&with_dotted),
                census_keys,
                census_ops,
                surface
            );
            // TYPED-STATE DIFF. The probes print, per scope, the fixture's typed items with the
            // exact predicates merge_scope_from_imports gates on, the item_registry keys, and the
            // func_env view keys. demanded-vs-closure divergence localizes the drop:
            // - service item missing/demanded (or t-/c- predicates false) -> realization filter;
            // - item present but registry/funcenv lacks the root or the service key -> the
            //   projection link quick-newt's seed-simulation pointed at (msg_9e3db779).
            let demanded_pass = route_pass(&graph, &intern, &si, &["decoded_pull_request"]);
            let closure_pass = route_pass(&graph, &intern, &si, &[]);
            grain_probe("demanded", &demanded_pass, &si);
            grain_probe("closure", &closure_pass, &si);
            tier1_replay("demanded", &demanded_pass, &graph, &intern, &si);
            tier1_replay("closure", &closure_pass, &graph, &intern, &si);
            assert!(
                service_realized,
                "the service whose operation decoded_pull_request spells inside its witness-frame subject must be realized"
            );
            assert!(
                surface.iter().all(|m| closure_surface.contains(m)),
                "the demanded grain must refuse no method surface the full closure does not: demanded={surface:?} closure={closure_surface:?}"
            );
            assert!(
                closure_surface.is_empty(),
                "the full-closure grain must refuse no service method surface: closure={closure_surface:?}"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

// MAIN-WET ARM (production victim, mgr msg 2026-11-10: quick-newt BB fcc42fff at 2b02d146).
// `gunbc run --entry dag/gunbc/instruments/generated_artifact_gate.dag --function main_wet_one
//  --arg path=dag/gunbc/stage0/stage0_crate_partition_generated.dag` fails on the demanded run
// grain with NoSuchVariable { name: "github.Pulls" } since #16156 -- a real production instrument
// (it blocks #16251's stage0 partition projection). Mirrors main.rs's run path exactly:
// resolve -> make_eval_context (Wet, non-claim) -> bind_run_args_for_entry ->
// run_in_context_with_args. RED at main (BB fcc42fff), must be GREEN at head.
#[test]
fn main_wet_one_evaluates_service_surface_on_the_run_grain() {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            let root = std::env::current_dir()
                .unwrap()
                .ancestors()
                .find(|d| d.join("dag").exists() && d.join("Cargo.toml").exists())
                .unwrap()
                .to_path_buf();
            std::env::set_current_dir(&root).unwrap();
            let source_roots = vec!["dag".to_string(), "src/v2".to_string()];
            let requested = vec!["main_wet_one".to_string()];
            let (graph, source_indices) =
                match crate::cli_run::resolve_entry_graph_demand_acquired_scoped(
                    &source_roots,
                    "dag/gunbc/instruments/generated_artifact_gate.dag",
                    &requested,
                ) {
                    Ok(pair) => pair,
                    Err(cause) => {
                        panic!("main_wet_one resolve failed: {cause}");
                    }
                };
            let mode = crate::cli_run::execution_mode_for(None, false, false);
            let ctx =
                crate::cli_run::make_eval_context(graph.as_ref(), source_indices.clone(), mode);
            let run_args: Vec<(String, String)> = vec![(
                "path".to_string(),
                "dag/gunbc/stage0/stage0_crate_partition_generated.dag".to_string(),
            )];
            let bound = match crate::cli_run::bind_run_args_for_entry(
                &ctx,
                "main_wet_one",
                &run_args,
            ) {
                Ok(b) => b,
                Err(message) => {
                    panic!("main_wet_one argv admission refused: {message}");
                }
            };
            let result = crate::v1_interpreter::run_in_context_with_args(
                &ctx,
                "main_wet_one",
                &bound,
                true,
            );
            eprintln!(
                "MAIN_WET ok={} err={:?}",
                result.is_ok(),
                result.as_ref().err().map(|e| format!("{e:?}"))
            );
            assert!(
                result.is_ok(),
                "main_wet_one must evaluate on the demanded run grain (production instrument, blocks #16251)"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
// #16156 witness (silent-otter-754, msg_9de1e557; adapted to this file's helper shapes) — a
// realized fn reaches a service op (here inside a lambda subject); the demanded grain gunbc run
// uses must agree with the whole-closure grain on the service op method surface. RED on main
// (closure refuses nothing, demanded refuses Observe); GREEN once the demanded grain binds
// reachable service op surfaces at the scope/ctx grain.
#[test]
fn demanded_grain_realizes_service_op_surface_reached_via_lambda_subject() {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            let (_sources, intern, si, _modules, graph) =
                graph_from("dag/test/claim/rest_exchange_replay.dag");
            let entry = "rest_replay_missing_fixture_refuses_without_network_fallback";
            let realization = grain_demand(&graph.modules, &[entry], &si);
            assert!(
                matches!(
                    realization.as_ref(),
                    crate::v1_compiler_infer::DeclarationDemand::OnlyDeclarations { realization, .. }
                        if realization.realized.get("test.RestReplay").copied().unwrap_or(false)
                ),
                "reachable service must be realized under the demanded grain"
            );
            let closure_messages = grain_typecheck(
                &graph.modules,
                &intern,
                &si,
                Rc::new(crate::v1_compiler_infer::DeclarationDemand::AllDeclarations),
            );
            let demanded_messages = grain_typecheck(&graph.modules, &intern, &si, realization);
            let closure_refuses = sdo_observe_refusal_present(&closure_messages);
            let demanded_refuses = sdo_observe_refusal_present(&demanded_messages);
            assert_eq!(
                demanded_refuses, closure_refuses,
                "demanded grain must agree with closure grain on the RestReplay.Observe method surface"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}

fn sdo_observe_refusal_present(messages: &[String]) -> bool {
    messages
        .iter()
        .any(|m| m.contains("RestReplay") && m.contains("Observe"))
}
