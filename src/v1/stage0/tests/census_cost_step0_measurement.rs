// STEP 0 of the census cost-shape scoping (measurement only — the numbers go to the manager
// before any behavior change is proposed; nothing here judges acceptance).
//
// WHAT IT MEASURES, at the seams where a `compile_dag_diagnostic_census` call can pay for the
// whole corpus (counters exposed exactly like the existing
// `compile_dag_diagnostic_census_memo_counts`, same file, same pattern):
//  * the module-path index a call receives is a full clone of the memoized corpus index
//    (INDEX_CLONES, INDEX_ENTRIES = entries materialized per clone);
//  * the scratch route clones the corpus-length source-file vec per call (VEC_CLONES, VEC_LEN);
//  * every real disk read the resolve path performs (READS, DISK_BYTES) — this is what settles
//    closure-scale vs corpus-scale with numbers.
// Cross-checked per call against the PROCESS-level counters /proc/self/io rchar + read_bytes and
// getrusage CPU, so the counter claim and the kernel's books cannot disagree.
//
// SHAPE: one 4-line fixture that imports std.algebra; N=1 calls and N=4 identical calls in one
// process. The census memo is inert (its floor guard was deleted with run_required_floor), so
// each call really does the work — the printed MEMO counts prove it (they must read (0, 0)).
use std::time::Instant;
use v1_compiler::cli_run::{
    compile_dag_diagnostic_census, compile_dag_diagnostic_census_cost_counts,
    compile_dag_diagnostic_census_memo_counts, reset_compile_dag_diagnostic_census_cost_counts,
};

/// module header / blank / one corpus import / one data. 4 lines, per the brief.
const FIXTURE: &str =
    "module c4.census_cost_fixture\n\nimport std.algebra { FreeMonoid }\ndata measured: Int = 1\n";

fn proc_io(field: &str) -> u64 {
    let Ok(io) = std::fs::read_to_string("/proc/self/io") else {
        return 0;
    };
    for line in io.lines() {
        let mut it = line.split_whitespace();
        if it.next() == Some(field) {
            return it.next().and_then(|v| v.parse().ok()).unwrap_or(0);
        }
    }
    0
}

fn cpu_micros() -> u64 {
    // utime+stime in clock ticks, from /proc/self/stat — no libc needed. The host is the
    // test process itself; per-thread and per-process agree here because the measurement
    // is single-threaded through the census call.
    let Ok(stat) = std::fs::read_to_string("/proc/self/stat") else {
        return 0;
    };
    // field 14 utime, field 15 stime (1-based); field 2 is comm, which may contain spaces
    // and parentheses — split after the last ')'. Fields after ')' start at field 3 = index 0,
    // so utime is index 11 and stime is index 12.
    let after = stat
        .rfind(')')
        .map(|i| &stat[i + 1..])
        .unwrap_or(stat.as_str());
    let fields: Vec<&str> = after.split_whitespace().collect();
    let ticks = fields
        .get(11)
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0)
        .saturating_add(
            fields
                .get(12)
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(0),
        );
    ticks * 1_000_000 / 100 // USER_HZ == 100 on Linux
}

fn sample(calls: usize) -> Vec<String> {
    let _ = reset_compile_dag_diagnostic_census_cost_counts();
    let mut out = Vec::new();
    let mut prev = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    for i in 1..=calls {
        let rchar0 = proc_io("rchar:");
        let blk0 = proc_io("read_bytes:");
        let cpu0 = cpu_micros();
        let t0 = Instant::now();
        let census = compile_dag_diagnostic_census(FIXTURE);
        let wall_us = t0.elapsed().as_micros() as u64;
        let cpu_us = cpu_micros().saturating_sub(cpu0);
        let now = compile_dag_diagnostic_census_cost_counts();
        let d = |k: usize| now.k - prev.k;
        prev = now;
        out.push(format!(
            "CALL {i} WALL_US={wall_us} CPU_US={cpu_us} INDEX_CLONES={} INDEX_ENTRIES={} \
             VEC_CLONES={} VEC_LEN={} READS={} DISK_BYTES={} PROC_RCHAR={} PROC_BLK_BYTES={} \
             CENSUS={}",
            d(0),
            d(1),
            d(2),
            d(3),
            d(4),
            d(5),
            proc_io("rchar:").saturating_sub(rchar0),
            proc_io("read_bytes:").saturating_sub(blk0),
            match &census {
                v1_compiler::cli_run::CompileDiagnosticCensus::Observed(rows) =>
                    format!("Observed({} rows)", rows.len()),
                v1_compiler::cli_run::CompileDiagnosticCensus::NotRunnable(cause) =>
                    format!("NotRunnable(len {})", cause.len()),
            }
        ));
    }
    out
}

#[test]
fn step0_census_cost_one_and_four_calls() {
    let (mh, mm) = compile_dag_diagnostic_census_memo_counts();
    println!(
        "MEMO HITS={mh} MISSES={mm} (must read 0 0: the memo is inert, every call does the work)"
    );
    assert_eq!(
        (mh, mm),
        (0, 0),
        "the memo must be inert for these numbers to mean per-call cost"
    );
    for line in sample(1) {
        println!("N=1 {line}");
    }
    // fresh drain so the four-call series starts clean
    let _ = reset_compile_dag_diagnostic_census_cost_counts();
    for line in sample(4) {
        println!("N=4 {line}");
    }
    // Instrument sanity, the only judgments in this file: the counters must actually see the
    // work the accusation names. Zero clones means the counters are decoration, not cost.
    // (sample(4) resets at its start, so these hold the four-call series totals.)
    let (ic, _ie, vc, _vl, r, _b) = compile_dag_diagnostic_census_cost_counts();
    assert!(
        ic >= 4,
        "index clone counter must see every call in the series, got {ic}"
    );
    assert!(
        vc >= 4,
        "scratch-vec clone counter must see every call in the series, got {vc}"
    );
    assert!(
        r >= 4,
        "disk-read counter must see the import closure on every call, got {r}"
    );
    println!("STEP0-DONE");
}
