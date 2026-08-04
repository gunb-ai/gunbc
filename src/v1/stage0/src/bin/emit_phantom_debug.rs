fn check(name: &str, source: &str, path: &str, includes: &[&str], excludes: &[&str]) {
    let ok = v1_compiler::cli_run::compile_dag_rust_emit_check(
        source,
        path,
        &includes.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        &excludes.iter().map(|s| s.to_string()).collect::<Vec<_>>(),
    );
    println!("{name}: {ok}");
}

fn main() {
    check(
        "phantom_rate",
        "module fn_clone_bound_witness.phantom_rate\n\ntype PerMinute\n\ntype MoneyRate<P> {\n  amount: Int\n  currency: String\n}\n\nfn money_rate_micros<P>(q: MoneyRate<P>) -> Nat {\n  q.amount\n}\n",
        "src/fn_clone_bound_witness_phantom_rate.rs",
        &["fn money_rate_micros<P>"],
        &["fn money_rate_micros<P: Clone>"],
    );
    check(
        "phantom_measure",
        "module fn_clone_bound_witness.phantom_measure\n\ntype Measure<Q, S, M> {\n  count: M\n}\n\nfn measure_count<Q, S, M>(m: Measure<Q, S, M>) -> M {\n  m.count\n}\n",
        "src/fn_clone_bound_witness_phantom_measure.rs",
        &["fn measure_count<Q, S, M>"],
        &[
            "fn measure_count<Q: Clone, S, M>",
            "fn measure_count<Q, S: Clone, M>",
            "fn measure_count<Q: Clone, S: Clone, M>",
        ],
    );
    check(
        "wf_negative",
        "module fn_clone_bound_witness.wf_negative\n\ntype Holder<N> {\n  tag: Int\n}\n\nfn describe_holder<N>(holder: Holder<N>) -> String {\n  \"holder\"\n}\n",
        "src/fn_clone_bound_witness_wf_negative.rs",
        &["fn describe_holder<N>"],
        &["fn describe_holder<N: Clone>"],
    );
}
