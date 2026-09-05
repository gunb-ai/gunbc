use super::*;

fn fixture() -> (ResolvedGraph, Rc<HashMap<String, Rc<NewlineIndex>>>) {
    let mut first = String::from("module fixture.cast_first\ntype Shared = String\n");
    for i in 0..512 {
        first.push_str(&format!("data unused_{i}: Int = {i}\n"));
    }
    let files = [
        ("cast_first.dag", first),
        (
            "cast_second.dag",
            "module fixture.cast_second\ntype Shared = Int\n".to_string(),
        ),
    ]
    .into_iter()
    .map(|(path, content)| {
        Rc::new(crate::v1_compiler_compile::SourceFile {
            path: path.to_string(),
            content,
        })
    })
    .collect();
    let result = crate::v1_compiler_compile::compile_to_resolved(Rc::new(files));
    let graph = result.graph.as_ref().expect("fixture resolves");
    ((**graph).clone(), result.source_indices.clone())
}

// The counted work is the host-side declaration walk, which eval_steps cannot see.
// Multiple fresh frames have the same prepared scope, exactly as on the required floor.
// Reverting to a frame-owned index authors the red: each first lookup walks every item.
#[test]
fn fresh_cast_frames_do_not_rescan_the_prepared_scope() {
    PROFILE_FLAG.with(|flag| flag.set(Some(true)));
    let (graph, sources) = fixture();
    let indexes = InterpContext::build_scope_indexes(&graph, sources);
    let mut samples = Vec::new();
    for _ in 0..4 {
        let ctx =
            InterpContext::over_scope_indexes(indexes.clone(), ExecutionMode::Hermetic, None, None);
        let before = cast_lookup_counters().2;
        let started = thread_cpu_nanos();
        assert!(lookup_type_item_across_modules(&ctx, "Shared").is_some());
        let cold_cpu = thread_cpu_nanos().saturating_sub(started);
        let cold_visits = cast_lookup_counters().2 - before;
        let started = thread_cpu_nanos();
        assert!(lookup_type_item_across_modules(&ctx, "Shared").is_some());
        let warm_cpu = thread_cpu_nanos().saturating_sub(started);
        let warm_visits = cast_lookup_counters().2 - before - cold_visits;
        samples.push((cold_visits, warm_visits, cold_cpu, warm_cpu));
    }
    eprintln!("cast-scope-index (cold visits, warm visits, cold cpu ns, warm cpu ns): {samples:?}");
    assert!(samples
        .iter()
        .all(|(cold, warm, _, _)| *cold == 0 && *warm == 0));
}

// Both modules author Shared with different kernels. Alias lookup has always used
// graph order, first declaration wins, independently of function lookup precedence.
#[test]
fn cast_lookup_preserves_graph_order_even_with_reversed_function_precedence() {
    let (graph, sources) = fixture();
    let first = graph
        .modules
        .iter()
        .flat_map(|m| m.items.iter())
        .find(|item| authored_name_at(sources.clone(), (*item).clone()) == "Shared")
        .expect("fixture has colliding declarations")
        .clone();
    let order: Vec<String> = graph
        .modules
        .iter()
        .rev()
        .map(|module| module.func_env.name.clone())
        .collect();
    for module_order in [None, Some(order.as_slice())] {
        let indexes = InterpContext::build_scope_indexes_with_module_order(
            &graph,
            sources.clone(),
            module_order,
            None,
        );
        let ctx = InterpContext::over_scope_indexes(indexes, ExecutionMode::Hermetic, None, None);
        let selected = lookup_type_item_across_modules(&ctx, "Shared").expect("Shared resolves");
        assert!(Rc::ptr_eq(&selected, &first));
        assert!(lookup_type_item_across_modules(&ctx, "Absent").is_none());
        assert!(lookup_type_item_across_modules(&ctx, "fixture.cast_first.Shared").is_none());
    }
}
