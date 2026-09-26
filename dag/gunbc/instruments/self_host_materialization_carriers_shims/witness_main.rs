use v1_compiled::std_realization::Materialization;
use v1_compiled::v2_compiler_materialization_carriers as emitted;

// NO SEED ORACLE, ON PURPOSE. This driver used to compare the prose constant
// v2_compiler_materialization_note against v1_compiler::v2_compiler_materialization_carriers, a
// module that no longer existed, so the receipt could not build. Its expected verdicts are read off
// the authority instead: src/v2/compiler/materialization_carriers.dag
// materialization_allows_memo_store is the governed memo door -- a Memoize verdict permits the
// store, and Recompute and Share refuse it.

// --inject-fault asserts ONLY the planted wrong acceptance (the #12275 shape): a Recompute verdict
// permits the memo store. A correct module reds it; a door that stores regardless greens it, which
// the harness rejects.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let recompute_stores = emitted::materialization_allows_memo_store(Materialization::Recompute);
    let all_pass = if inject_fault {
        println!("materialization injected: Recompute stores={recompute_stores}");
        recompute_stores
    } else {
        let memoize_stores = emitted::materialization_allows_memo_store(Materialization::Memoize);
        let share_stores = emitted::materialization_allows_memo_store(Materialization::Share);
        println!("materialization Memoize stores={memoize_stores}");
        println!("materialization Recompute refuses={}", !recompute_stores);
        println!("materialization Share refuses={}", !share_stores);
        memoize_stores && !recompute_stores && !share_stores
    };

    if all_pass {
        println!("SELF_HOST_MATERIALIZATION_CARRIERS_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_MATERIALIZATION_CARRIERS_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
