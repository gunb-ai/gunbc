use std::rc::Rc;
use v1_compiled::v2_compiler_materialization_carriers as emitted;
use v1_compiler::v2_compiler_materialization_carriers as seed;

fn mat_eq(e: &emitted::Materialization, s: &seed::Materialization) -> bool {
    use emitted::Materialization as E;
    use seed::Materialization as S;
    matches!(
        (e, s),
        (E::Memoize, S::Memoize) | (E::Recompute, S::Recompute) | (E::Share, S::Share)
    )
}

fn rc_str_eq(e: &Rc<String>, s: &str) -> bool {
    e.as_str() == s
}

fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let mut all_pass = true;

    let memo_cases = [
        (emitted::Materialization::Memoize, seed::Materialization::Memoize, true),
        (emitted::Materialization::Recompute, seed::Materialization::Recompute, false),
        (emitted::Materialization::Share, seed::Materialization::Share, false),
    ];
    for (i, (em, sm, expected)) in memo_cases.iter().enumerate() {
        let em_use = if inject_fault && i == 0 {
            emitted::Materialization::Recompute
        } else {
            em.clone()
        };
        let ev = emitted::materialization_allows_memo_store(em_use.clone());
        let sv = seed::materialization_allows_memo_store(sm.clone());
        let ok = ev == sv && ev == *expected;
        println!("memo_store({em_use:?}) emitted={ev} seed={sv} expected={expected} eq={ok}");
        all_pass &= ok;
    }

    let ladder_ok = emitted::v2_compiler_materialization_ladder_holds()
        == seed::v2_compiler_materialization_ladder_holds();
    println!(
        "ladder_holds emitted={} seed={} eq={ladder_ok}",
        emitted::v2_compiler_materialization_ladder_holds(),
        seed::v2_compiler_materialization_ladder_holds()
    );
    all_pass &= ladder_ok;

    let refusals_ok = emitted::v2_compiler_catalog_projection_refusals()
        == seed::v2_compiler_catalog_projection_refusals();
    println!(
        "catalog_refusals emitted={} seed={} eq={refusals_ok}",
        emitted::v2_compiler_catalog_projection_refusals(),
        seed::v2_compiler_catalog_projection_refusals()
    );
    all_pass &= refusals_ok;

    let parse_id_ok = rc_str_eq(
        &emitted::parse_table_memo_provider_id(),
        &seed::parse_table_memo_provider_id(),
    );
    println!(
        "parse_table_provider_id emitted={:?} seed={} eq={parse_id_ok}",
        emitted::parse_table_memo_provider_id(),
        seed::parse_table_memo_provider_id()
    );
    all_pass &= parse_id_ok;

    let stage_id_ok = rc_str_eq(
        &emitted::compile_stage_memo_provider_id(),
        &seed::compile_stage_memo_provider_id(),
    );
    println!(
        "compile_stage_provider_id emitted={:?} seed={} eq={stage_id_ok}",
        emitted::compile_stage_memo_provider_id(),
        seed::compile_stage_memo_provider_id()
    );
    all_pass &= stage_id_ok;

    let parse_mat_ok = mat_eq(
        &emitted::parse_table_memo_materialization(),
        &seed::parse_table_memo_materialization(),
    );
    println!(
        "parse_table_materialization emitted={:?} seed={:?} eq={parse_mat_ok}",
        emitted::parse_table_memo_materialization(),
        seed::parse_table_memo_materialization()
    );
    all_pass &= parse_mat_ok;

    let carrier_ok = emitted::parse_table_carrier_grounded_on_catalog()
        == seed::parse_table_carrier_grounded_on_catalog();
    println!(
        "carrier_grounded emitted={} seed={} eq={carrier_ok}",
        emitted::parse_table_carrier_grounded_on_catalog(),
        seed::parse_table_carrier_grounded_on_catalog()
    );
    all_pass &= carrier_ok;

    let narrow_ok = emitted::parse_table_memo_plural_scope_too_narrow_count()
        == seed::parse_table_memo_plural_scope_too_narrow_count();
    println!(
        "plural_scope_too_narrow emitted={} seed={} eq={narrow_ok}",
        emitted::parse_table_memo_plural_scope_too_narrow_count(),
        seed::parse_table_memo_plural_scope_too_narrow_count()
    );
    all_pass &= narrow_ok;

    let plural_holds_ok = emitted::parse_table_memo_plural_holds_with_provider()
        == seed::parse_table_memo_plural_holds_with_provider();
    println!(
        "plural_holds_with_provider emitted={} seed={} eq={plural_holds_ok}",
        emitted::parse_table_memo_plural_holds_with_provider(),
        seed::parse_table_memo_plural_holds_with_provider()
    );
    all_pass &= plural_holds_ok;

    if all_pass {
        println!("SELF_HOST_MATERIALIZATION_CARRIERS_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_MATERIALIZATION_CARRIERS_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
