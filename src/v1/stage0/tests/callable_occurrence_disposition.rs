// THE DISPOSITION MUST BE MINTED WHERE THE CONTAINMENT DECISION IS MADE.
//
// `CallableOccurrenceAccounting` declares four arms -- Admitted, NoProviderDeclared,
// FilteredByContainment, IdentityUnavailable. Before the repair this type was the COLLECTOR's
// return type, and `callable_occurrence_row` stamped every identified occurrence
// `CallableOccurrenceNoProviderDeclared` while taking only (n, source_indices): no owner index was
// in scope, so it could not have checked whether a provider exists. Two of the four arms had no
// constructor anywhere in the repository, and the arm that was produced asserted a fact that was
// never computed.
//
// The repair is construction, not validation: the collector now returns `CallableOccurrenceRow`,
// which has no disposition field at all, so the invalid state is unwritable rather than checked.
// The accounting is minted in `reference_derived_supply`, in the same fold that decides whether a
// row enters the parent-env supply.
//
// THE REPAIR IS ESTABLISHED BY THE CHANGE IN WHAT THIS PROBE READS, NOT BY IT GREENING.
// Against the collector it read `rows=3 admitted=0 no_provider_declared=3` -- one arm, uniformly,
// with no owner index consulted. Against the supply it reads
// `rows=3 admitted=0 no_provider_declared=2 filtered_by_containment=1`, with
// `rows_detail=["FilteredByContainment(pf)", "NoProviderDeclared(probe)", "NoProviderDeclared(x)"]`
// and `pf_owners=["probe.provider"]`. The disposition now DISCRIMINATES, and it discriminates using
// the owner index the production declaration layer built.
//
// IT IS STILL RED, AND THE RED HAS MOVED TO A DIFFERENT DEFECT. `probe.caller` calls
// `probe.provider.pf`; the declaration layer indexes `pf` under owner `probe.provider`; but the
// occurrence the collector hands the supply carries the authored name `pf` -- the LEAF -- with the
// qualifying segments arriving as separate occurrences (`probe` is its own row). So
// `reference_containment_qualifier` returns the empty string, the supply takes the BARE arm, and
// the bare arm correctly refuses an owner that is neither the referencing module nor one of its
// containment ancestors. The qualified arm of `containment_admits_owner` is never reached for a
// qualified cross-module call through this collector.
//
// That is a defect in the SUPPLY'S INPUT, not in the disposition, and it is not repaired here: the
// fix is a decision about occurrence grain, not a widening this lane may make on its own. The probe
// stays enrolled and stays red until that decision lands -- which is the point of it. Making the
// collector reassemble the qualified spelling, or making the bare arm admit non-ancestor owners,
// would both make this green by admitting MORE, which is the line-stop signal.
//
// WHICH EDGE EACH PIECE OF EVIDENCE CROSSES:
//   * `callable_occurrence_rows_are_produced_at_all` -- the COLLECTOR edge (parsed items ->
//     occurrence rows). Positive control: without it, an empty population would satisfy every claim
//     below for a reason unrelated to the defect. GREEN.
//   * `owner_index_declares_the_probe_callee` -- the PRODUCER edge (declaration layer -> owner
//     index). It establishes that the input to the join was built by the production path and not by
//     this test, so the red below is a property of the join rather than of a hand-authored fixture.
//     GREEN.
//   * `some_callable_occurrence_is_accounted_admitted` -- the SUPPLY edge (occurrence rows x owner
//     index -> accounting). RED, and its message carries the per-row detail that names the grain
//     mismatch.

use std::rc::Rc;

const CALLER_SOURCE: &str =
    "module probe.caller\n\nfn af(x: Int) -> Int {\n  probe.provider.pf(x: x)\n}\n";
const PROVIDER_SOURCE: &str = "module probe.provider\n\nfn pf(x: Int) -> Int {\n  x\n}\n";

struct Fixture {
    caller_items: Rc<im::Vector<Rc<v1_compiler::v1_std_core::Node>>>,
    owner_index: Rc<v1_compiler::v1_compiler_infer_sigs::CallableOwnerIndex>,
    source_indices: Rc<im::HashMap<String, Rc<v1_compiler::v1_std_core::NewlineIndex>>>,
}

fn fixture() -> Fixture {
    let sources = [
        ("probe_caller.dag", CALLER_SOURCE),
        ("probe_provider.dag", PROVIDER_SOURCE),
    ];
    let mut modules: Vec<Rc<v1_compiler::v1_std_core::Node>> = Vec::new();
    let mut indices = im::HashMap::new();
    let mut intern_table = v1_compiler::v1_std_core::empty_intern_table();
    for (key, content) in sources {
        let tokens =
            v1_compiler::v1_compiler_tokenize::tokenize(content.to_string(), key.to_string());
        let index =
            v1_compiler::v1_std_core::build_newline_index(key.to_string(), content.to_string());
        indices.insert(key.to_string(), index);
        let parsed = v1_compiler::v1_compiler_parse::parse_with_table(
            tokens,
            Rc::new(indices.clone()),
            intern_table.clone(),
        );
        intern_table = parsed.intern_table.clone();
        modules.push(
            parsed
                .result
                .module
                .as_ref()
                .expect("fixture must parse")
                .clone(),
        );
    }
    let source_indices = Rc::new(indices);
    let graph = v1_compiler::v1_compiler_resolve::resolve_modules(
        Rc::new(modules.into_iter().collect()),
        source_indices.clone(),
    );
    // The owner index comes from the PRODUCER -- the same phase-1 declaration layer the host runs --
    // so this probe cannot establish the join by constructing the join's own input.
    let intern_table = v1_compiler::v1_compiler_infer::seed_kernel_intern_table(intern_table);
    let mut resolved_by_name = im::HashMap::new();
    for rm in graph.modules.iter() {
        let name =
            v1_compiler::v1_std_core::authored_name_at(source_indices.clone(), rm.module.clone());
        resolved_by_name.insert(name, rm.clone());
    }
    let resolved_by_name = Rc::new(resolved_by_name);
    let symbol_index = v1_compiler::v1_compiler_infer::build_symbol_index_census(
        graph.modules.clone(),
        source_indices.clone(),
    );
    let declarations = v1_compiler::v1_compiler_infer::build_declaration_layer(
        graph.modules.clone(),
        resolved_by_name,
        source_indices.clone(),
        intern_table,
        symbol_index,
    );
    let caller = graph
        .modules
        .iter()
        .find(|rm| {
            v1_compiler::v1_std_core::authored_name_at(source_indices.clone(), rm.module.clone())
                == "probe.caller"
        })
        .expect("caller module must resolve")
        .clone();
    Fixture {
        caller_items: v1_compiler::v1_std_core::module_items(caller.module.clone()),
        owner_index: declarations.callable_owner_index.clone(),
        source_indices,
    }
}

#[test]
fn callable_occurrence_rows_are_produced_at_all() {
    let f = fixture();
    let rows = v1_compiler::v1_compiler_infer_sigs::callable_occurrence_rows(
        f.caller_items.clone(),
        f.source_indices.clone(),
    );
    assert!(
        !rows.is_empty(),
        "POSITIVE CONTROL (collector edge): the fixture must produce callable occurrence rows, \
         otherwise the accounting probe is satisfied by an empty population and measures nothing"
    );
}

#[test]
fn owner_index_declares_the_probe_callee() {
    let f = fixture();
    assert!(
        f.owner_index.by_name.get("pf").is_some(),
        "POSITIVE CONTROL (producer edge): the declaration layer must have indexed `pf` as a \
         declared callable. Without it the supply has nothing to admit and the probe below would \
         be red for a reason that is not the disposition defect. keys={:?}",
        f.owner_index.by_name.keys().collect::<Vec<_>>()
    );
}

#[test]
fn some_callable_occurrence_is_accounted_admitted() {
    use v1_compiler::v1_compiler_infer_sigs::CallableOccurrenceAccounting as A;
    let f = fixture();
    let rows = v1_compiler::v1_compiler_infer_sigs::reference_supply_accounting(
        f.caller_items.clone(),
        f.owner_index.clone(),
        "probe.caller".to_string(),
        f.source_indices.clone(),
    );
    let admitted = rows
        .iter()
        .filter(|r| matches!(***r, A::CallableOccurrenceAdmitted { .. }))
        .count();
    let no_provider = rows
        .iter()
        .filter(|r| matches!(***r, A::CallableOccurrenceNoProviderDeclared { .. }))
        .count();
    let filtered = rows
        .iter()
        .filter(|r| matches!(***r, A::CallableOccurrenceFilteredByContainment { .. }))
        .count();
    let detail: Vec<String> = rows
        .iter()
        .map(|r| match &**r {
            A::CallableOccurrenceAdmitted {
                authored_name,
                owner_module_paths,
                ..
            } => {
                format!("Admitted({authored_name} <- {owner_module_paths:?})")
            }
            A::CallableOccurrenceNoProviderDeclared { authored_name, .. } => {
                format!("NoProviderDeclared({authored_name})")
            }
            A::CallableOccurrenceFilteredByContainment { authored_name, .. } => {
                format!("FilteredByContainment({authored_name})")
            }
            A::CallableOccurrenceIdentityUnavailable { authored_name, .. } => {
                format!("IdentityUnavailable({authored_name})")
            }
        })
        .collect();
    let owners: Vec<String> = f
        .owner_index
        .by_name
        .get("pf")
        .map(|rows| rows.iter().map(|r| r.owner_module_path.clone()).collect())
        .unwrap_or_default();
    assert!(
        admitted > 0,
        "SUPPLY EDGE: `probe.caller` calls `probe.provider.pf` and the declaration layer indexes \
         `pf` under owner `probe.provider`, so the accounting must carry an Admitted row. It does \
         not, because the occurrence reaches the supply under its LEAF name: the qualifier is \
         empty, the bare containment arm is taken, and it refuses an owner that is not on the \
         referencing module's ancestor chain. See this file's header -- the grain mismatch is the \
         subject, and greening this by admitting more is the line-stop signal. \
         rows={} admitted={} no_provider_declared={} filtered_by_containment={} rows_detail={:?} pf_owners={:?}",
        rows.len(),
        admitted,
        no_provider,
        filtered,
        detail,
        owners
    );
}
