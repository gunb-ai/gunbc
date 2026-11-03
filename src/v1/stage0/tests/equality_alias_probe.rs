// TEMPORARY PROBE (not a committed witness): measures what the equality admission wall does
// for ALIAS operands of == (srdk26 a1-a5 class), and prints the census rows plus walk DIAG
// lines (env-gated in the compiler) so the branch can be located before editing.
//
// One #[test] per source, run with --test-threads=1: if the row hits its wall, the sources
// already run have flushed their observations (every line goes to stderr, unbuffered).
// Delete before the PR lands.
use v1_compiler::cli_run::{compile_dag_diagnostic_census, CompileDiagnosticCensus};

const BASE_FN_RECORD: &str = "module eq_probe_fn_record\ntype Ops {\n  step: fn(Int) -> Int\n}\nfn same(a: Ops, b: Ops) -> Bool { a == b }\n";
const A4_ALIAS_FN_RECORD: &str = "module eq_probe_alias_fn_record\ntype Ops {\n  step: fn(Int) -> Int\n}\ntype OpsAlias = Ops\nfn same(a: OpsAlias, b: OpsAlias) -> Bool { a == b }\n";
const A3_LOCAL_GENERIC_FN_CELL: &str = "module eq_probe_alias_local_generic\ntype Cell<T> {\n  value: T\n}\ntype CellFn = Cell<fn(Int) -> Int>\nfn same(a: CellFn, b: CellFn) -> Bool { a == b }\n";
const A5_HOLDER_OF_ALIAS: &str = "module eq_probe_alias_holder\ntype Ops {\n  step: fn(Int) -> Int\n}\ntype OpsAlias = Ops\ntype Holder {\n  ops: OpsAlias\n}\nfn same(a: Holder, b: Holder) -> Bool { a == b }\n";
const A1_IMPORTED_GENERIC_FN_CELL: &str = "module eq_probe_alias_imported_generic\nimport std.types { List }\ntype ListFn = List<fn(Int) -> Int>\nfn same(a: ListFn, b: ListFn) -> Bool { a == b }\n";
const A2_PLAIN_ALIAS_GREEN: &str = "module eq_probe_alias_plain\ntype Cell {\n  value: Int\n}\ntype CellAlias = Cell\nfn same(a: CellAlias, b: CellAlias) -> Bool { a == b }\n";
const A2PRIME_LOCAL_GENERIC_PLAIN_CELL: &str = "module eq_probe_alias_local_generic_plain\ntype Cell<T> {\n  value: T\n}\ntype CellInt = Cell<Int>\nfn same(a: CellInt, b: CellInt) -> Bool { a == b }\n";

fn report(label: &str, source: &str) {
    // SAFETY: the row runs tests with --test-threads=1; the flag only gates diagnostic eprintlns.
    unsafe { std::env::set_var("EQ_ALIAS_PROBE_DIAG", "1") };
    // stderr is unbuffered, so a wall-timeout still leaves every completed line on disk.
    eprintln!("PROBE-BEGIN {label}");
    match compile_dag_diagnostic_census(source) {
        CompileDiagnosticCensus::Observed(rows) => {
            eprintln!("PROBE {label}: {} row(s)", rows.len());
            for r in &rows {
                eprintln!(
                    "PROBE {label}: class={} subject={} blocking={} count={}",
                    r.diagnostic_class, r.subject_name, r.blocking, r.count
                );
            }
            if rows.is_empty() {
                eprintln!("PROBE {label}: ADMITTED CLEAN (no diagnostics)");
            }
        }
        CompileDiagnosticCensus::NotRunnable(cause) => {
            eprintln!("PROBE {label}: NOT RUNNABLE: {cause}");
        }
    }
    eprintln!("PROBE-END {label}");
}

// Numeric prefixes fix the order under --test-threads=1: the simplest reproducer and the
// control come first, so even a truncated row yields the branch + the control.
#[test]
fn t0_a4_alias_fn_record() {
    report("A4_ALIAS_FN_RECORD", A4_ALIAS_FN_RECORD);
}
#[test]
fn t1_a2_plain_alias_green() {
    report("A2_PLAIN_ALIAS_GREEN", A2_PLAIN_ALIAS_GREEN);
}
#[test]
fn t2_base_fn_record() {
    report("BASE_FN_RECORD", BASE_FN_RECORD);
}
#[test]
fn t3_a3_local_generic_fn_cell() {
    report("A3_LOCAL_GENERIC_FN_CELL", A3_LOCAL_GENERIC_FN_CELL);
}
#[test]
fn t4_a5_holder_of_alias() {
    report("A5_HOLDER_OF_ALIAS", A5_HOLDER_OF_ALIAS);
}
#[test]
fn t5_a1_imported_generic_fn_cell() {
    report("A1_IMPORTED_GENERIC_FN_CELL", A1_IMPORTED_GENERIC_FN_CELL);
}
#[test]
fn t6_a2prime_local_generic_plain_cell() {
    report(
        "A2PRIME_LOCAL_GENERIC_PLAIN_CELL",
        A2PRIME_LOCAL_GENERIC_PLAIN_CELL,
    );
}
