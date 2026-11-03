// TEMPORARY PROBE (not a committed witness): measures what the equality admission wall does
// for ALIAS operands of == (srdk26 a1-a5 class). Prints the census rows for each source so the
// branch can be located before editing. Delete before the PR lands.
use v1_compiler::cli_run::{compile_dag_diagnostic_census, CompileDiagnosticCensus};

const BASE_FN_RECORD: &str = "module eq_probe_fn_record\ntype Ops {\n  step: fn(Int) -> Int\n}\nfn same(a: Ops, b: Ops) -> Bool { a == b }\n";
const A4_ALIAS_FN_RECORD: &str = "module eq_probe_alias_fn_record\ntype Ops {\n  step: fn(Int) -> Int\n}\ntype OpsAlias = Ops\nfn same(a: OpsAlias, b: OpsAlias) -> Bool { a == b }\n";
const A3_LOCAL_GENERIC_FN_CELL: &str = "module eq_probe_alias_local_generic\ntype Cell<T> {\n  value: T\n}\ntype CellFn = Cell<fn(Int) -> Int>\nfn same(a: CellFn, b: CellFn) -> Bool { a == b }\n";
const A5_HOLDER_OF_ALIAS: &str = "module eq_probe_alias_holder\ntype Ops {\n  step: fn(Int) -> Int\n}\ntype OpsAlias = Ops\ntype Holder {\n  ops: OpsAlias\n}\nfn same(a: Holder, b: Holder) -> Bool { a == b }\n";
const A1_IMPORTED_GENERIC_FN_CELL: &str = "module eq_probe_alias_imported_generic\nimport std.types { List }\ntype ListFn = List<fn(Int) -> Int>\nfn same(a: ListFn, b: ListFn) -> Bool { a == b }\n";
const A2_PLAIN_ALIAS_GREEN: &str = "module eq_probe_alias_plain\ntype Cell {\n  value: Int\n}\ntype CellAlias = Cell\nfn same(a: CellAlias, b: CellAlias) -> Bool { a == b }\n";
const A2PRIME_LOCAL_GENERIC_PLAIN_CELL: &str = "module eq_probe_alias_local_generic_plain\ntype Cell<T> {\n  value: T\n}\ntype CellInt = Cell<Int>\nfn same(a: CellInt, b: CellInt) -> Bool { a == b }\n";

fn report(label: &str, source: &str) {
    match compile_dag_diagnostic_census(source) {
        CompileDiagnosticCensus::Observed(rows) => {
            println!("PROBE {label}: {} row(s)", rows.len());
            for r in &rows {
                println!(
                    "  class={} subject={} blocking={} count={}",
                    r.diagnostic_class, r.subject_name, r.blocking, r.count
                );
            }
            if rows.is_empty() {
                println!("  (ADMITTED CLEAN — no diagnostics)");
            }
        }
        CompileDiagnosticCensus::NotRunnable(cause) => {
            println!("PROBE {label}: NOT RUNNABLE: {cause}");
        }
    }
}

#[test]
fn probe_alias_equality_base_behaviour() {
    // SAFETY: single-threaded test entry; the flag only gates diagnostic eprintlns.
    unsafe { std::env::set_var("EQ_ALIAS_PROBE_DIAG", "1") };
    report("BASE_FN_RECORD", BASE_FN_RECORD);
    report("A4_ALIAS_FN_RECORD", A4_ALIAS_FN_RECORD);
    report("A3_LOCAL_GENERIC_FN_CELL", A3_LOCAL_GENERIC_FN_CELL);
    report("A5_HOLDER_OF_ALIAS", A5_HOLDER_OF_ALIAS);
    report("A1_IMPORTED_GENERIC_FN_CELL", A1_IMPORTED_GENERIC_FN_CELL);
    report("A2_PLAIN_ALIAS_GREEN", A2_PLAIN_ALIAS_GREEN);
    report(
        "A2PRIME_LOCAL_GENERIC_PLAIN_CELL",
        A2PRIME_LOCAL_GENERIC_PLAIN_CELL,
    );
}
