// STEP 0 of the census cost-shape scoping. It began as measurement only (the numbers went to
// the manager before any behavior change was proposed, mgr msg_f3513042) and became the
// strongest committed assertion the overlay fix has: the checks below are the ones a regression
// trips -- WHEN THEY ARE EXECUTED.
//
// EXECUTED WHERE, exactly, because "regression gate" was too strong a word for this file to
// carry (review 84038). This is a cargo integration test in the v1 seed, and NO CI STEP RUNS IT:
// `cargo test --release -p v1-compiler --lib` and the seed's integration tests left the merge
// path with gunbc.rung_drop rust_unit_tests_off_the_merge_path (declared 2026-09-04, standing
// Superseded; its loss holder gunbc.rung_drop v1_required_lanes_withdrawn is what states
// today's rung, and DESIGN's "Building & checks" says it plainly -- the required path runs no
// v1 unit tests). So the rung this file holds is LOCAL MEASUREMENT: what a developer, or an
// instrument run that chooses to execute it, sees. It is NOT a merge-blocking gate, and a green
// required job is not evidence that the zero-clone, zero-refusal or digest-equality assertions
// hold -- they hold because someone ran this file.
//
// WHAT WOULD ENROL IT, named rather than implied, per §6 ("a lens nothing gates on is itself a
// lie"): the restoration trigger of gunbc.rung_drop v1_required_lanes_withdrawn -- a binary
// built from an emission of v2 judging this population on the required path with a discriminating
// red. The alternative the review offered (a row of gunbc.emitted_subject_build_gate) was
// declined deliberately: that gate is the SINGLE required lane, and putting a v1 cargo test on it
// would reverse the 2026-10-09 v1-withdrawal ruling rather than report to it. Reversing an
// operator ruling is not this PR's authority to spend; stating the rung is.
//
// WHAT IT MEASURES, at the seams where a `compile_dag_diagnostic_census` call can pay for the
// whole corpus:
//  * the module-path index a call receives (INDEX_CLONES, INDEX_ENTRIES = entries materialized
//    per clone) -- GATED at zero per warm call; the resolve borrows the memo entry for entry, so
//    an owned corpus index materializing is the defect coming back;
//  * the scratch route's source-file map clone (VEC_CLONES, VEC_LEN);
//  * every real disk read the resolve path performs (READS, DISK_BYTES) — this is what settled
//    closure-scale vs corpus-scale with numbers.
// Cross-checked per call against the PROCESS-level counters /proc/self/io rchar + read_bytes and
// getrusage CPU, so the counter claim and the kernel's books cannot disagree.
//
// SHAPE: one 4-line fixture that imports std.algebra; N=1 calls and N=4 identical calls in one
// process, then a collision-named fixture, then one ordinary call again. Nothing caches the
// census anymore (the memo is deleted; see its entry point's deletion story), so each call does
// its own work — which is what makes the per-call counters the right instrument.
use std::time::Instant;
use v1_compiler::cli_run::{
    compile_dag_diagnostic_census, compile_dag_diagnostic_census_cost_counts, pre_entry_phase,
    reset_compile_dag_diagnostic_census_cost_counts, resolve_stage_totals,
};

/// module header / blank / one corpus import / one data. 4 lines, per the brief.
const FIXTURE: &str =
    "module c4.census_cost_fixture\n\nimport std.algebra { FreeMonoid }\ndata measured: Int = 1\n";

/// CONDITION 1 PROBE (mgr msg_ef915e6b): a fixture whose own module path COLLIDES with a real
/// pool module (`std.algebra`) while it imports that same module. If any shared pool cache ever
/// depended on the fixture — absorbing its state, or a pool answer leaking differently through
/// the scratch — the digest of the ordinary fixture measured BEFORE and AFTER this one runs in
/// the same process would differ. The census either observes or refuses; both are compared
/// before/after the overlay as digests, so even NotRunnable is a judged outcome.
const FIXTURE_COLLIDE: &str =
    "module std.algebra\n\nimport std.algebra { FreeMonoid }\ndata collision_marker: Int = 1\n";

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

fn sample(calls: usize, fixture: &'static str, tag: &str) -> Vec<String> {
    let _ = reset_compile_dag_diagnostic_census_cost_counts();
    let mut out = Vec::new();
    let mut prev = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
    for i in 1..=calls {
        let rchar0 = proc_io("rchar:");
        let blk0 = proc_io("read_bytes:");
        let cpu0 = cpu_micros();
        // Drain the phase receipt before the call so what prints below is THIS call's rows,
        // and snapshot the cumulative stage account so the call's delta is readable as the
        // difference between consecutive STAGE lines (the account is monotonic per process).
        let _ = pre_entry_phase::take_rows_with_attribution();
        let t0 = Instant::now();
        let census = compile_dag_diagnostic_census(fixture);
        let wall_us = t0.elapsed().as_micros() as u64;
        let cpu_us = cpu_micros().saturating_sub(cpu0);
        let phase_rows = pre_entry_phase::take_rows_with_attribution();
        let stage = resolve_stage_totals();
        let (ic, ie, vc, vl, r, b, refusals) = compile_dag_diagnostic_census_cost_counts();
        let (pic, pie, pvc, pvl, pr, pb, pref) = prev;
        prev = (ic, ie, vc, vl, r, b, refusals);
        let (ic, ie, vc, vl, r, b, refusals) = (
            ic - pic,
            ie - pie,
            vc - pvc,
            vl - pvl,
            r - pr,
            b - pb,
            refusals - pref,
        );
        out.push(format!(
            "{tag} CALL {i} WALL_US={wall_us} CPU_US={cpu_us} INDEX_CLONES={ic} INDEX_ENTRIES={ie} \
             VEC_CLONES={vc} VEC_LEN={vl} READS={r} DISK_BYTES={b} ID_REFUSALS={refusals} PROC_RCHAR={} PROC_BLK_BYTES={} \
             STAGE{{load={}us parse={}us resolve={}us normalize={}us typecheck={}us \
             reconcile_assembly={}us schedule={}us probe={}us graph={}us sym={}us}} \
             CENSUS={}",
            proc_io("rchar:").saturating_sub(rchar0),
            proc_io("read_bytes:").saturating_sub(blk0),
            stage.load / 1000,
            stage.parse / 1000,
            stage.resolve / 1000,
            stage.normalize / 1000,
            stage.typecheck_compute / 1000,
            stage.reconcile_assembly / 1000,
            stage.assembly_schedule / 1000,
            stage.assembly_probe / 1000,
            stage.assembly_graph / 1000,
            stage.assembly_symbol_index / 1000,
            match &census {
                v1_compiler::cli_run::CompileDiagnosticCensus::Observed(rows) => {
                    // The stability comparisons judge this digest: every row's Debug
                    // rendering, hashed. The count alone could survive a per-row change.
                    // (Identity across the whole change is the PR's cross-head receipt
                    // comparison -- the same test printed at pre-overlay and overlay heads --
                    // not an assertion in this file; no pre-change code runs in this process.)
                    use std::hash::{Hash, Hasher};
                    let mut h = std::collections::hash_map::DefaultHasher::new();
                    rows.len().hash(&mut h);
                    for row in rows {
                        format!("{row:?}").hash(&mut h);
                    }
                    format!("Observed({} rows DIGEST={:016x})", rows.len(), h.finish())
                }
                v1_compiler::cli_run::CompileDiagnosticCensus::NotRunnable(cause) =>
                    format!("NotRunnable(len {} DIGEST={:016x})", cause.len(), {
                        use std::hash::{Hash, Hasher};
                        let mut h = std::collections::hash_map::DefaultHasher::new();
                        cause.hash(&mut h);
                        h.finish()
                    }),
            }
        ));
        for row in phase_rows {
            out.push(format!("  PHASE {i}: {row}"));
        }
    }
    out
}

/// The digest token of a printed CALL line — everything a stability comparison must match on,
/// stripped of the cost fields that legitimately differ between runs.
fn digest_of(line: &str) -> &str {
    line.split("CENSUS=").nth(1).unwrap_or("<none>")
}

#[test]
fn step0_census_cost_one_and_four_calls() {
    // N=1 then N=4 of the ordinary fixture: the before/after cost series.
    let n1 = sample(1, FIXTURE, "STD");
    for line in &n1 {
        println!("N=1 {line}");
    }
    let _ = reset_compile_dag_diagnostic_census_cost_counts();
    let n4 = sample(4, FIXTURE, "STD");
    for line in &n4 {
        println!("N=4 {line}");
    }
    // THE COMMITTED CONSUMER (mgr msg_89cafe3f: the counters land only if a committed consumer
    // reads them; review 84038: "committed" is not the same as "executed by CI" -- see the
    // execution-locus block at the top of this file). Bracketed at the reset above, these
    // totals hold the four WARM calls only.
    // Zero corpus-index clones is what the overlay + the module-path borrow promised: a call
    // may not materialize the corpus module-path index out of its memo, because nothing asks
    // it for an owned copy anymore (the resolve borrows the memo entry for entry). REVERT
    // EITHER HALF AND THIS GOES RED: the borrow back to `build_module_path_index_from_witness_roots`
    // lands one 9003-entry clone per call (total 4), and the whole-pool/per-root census
    // delegations back to per-scratch rebuilds put the ~10 s back into WALL_US/CPU_US, which
    // print on every line above. VEC_CLONES and READS stay PRINTED, not gated: they are the
    // observation of what the overlay deliberately still does per call (the scratch's own
    // source-file map, Rc-bump scale; the fixture's 3-file import closure off disk).
    let warm = compile_dag_diagnostic_census_cost_counts();
    assert_eq!(
        warm.0, 0,
        "warm census calls materialized the corpus module-path index {} times: the borrow or \
         the pool-cache overlay regressed (per-call value must be 0)",
        warm.0
    );
    // Instrument sanity against a dead-counter false green: the disk counter must still see
    // the fixture's import closure on those same calls. If it reads 0 the counters stopped
    // seeing the work and the zero above would be decoration, not cost.
    assert!(
        warm.4 >= 1,
        "disk-read counter must still see the import closure on the warm series, got {}",
        warm.4
    );
    // THE READ-THROUGH IS ACTUALLY BEING TAKEN. A wrongly-refused scratch surfaces as
    // NotRunnable -- but sample() renders even a refusal as a digest, so the collision-probe
    // equality BELOW could survive an always-refusing census (pre and post refusing alike is
    // still "equal"). This counter is therefore not decoration behind that equality: it is the
    // assertion that makes a refusal loud at all, and it attributes a regression to the guard
    // rather than to the census content. (The drifted-facts RED that drives this counter nonzero
    // lives in cli_run's own #[cfg(test)] unit tests, a separate binary, so it cannot contaminate
    // this process's count.)
    assert_eq!(
        warm.6, 0,
        "warm census calls were refused the underlay read-through {} times: a false refusal \
         silently puts the ~10 s back, so this must stay 0 for the fix to be live",
        warm.6
    );
    // CONDITION 1 (mgr msg_ef915e6b): run a fixture whose module path collides with a pool
    // module, then re-measure the ordinary fixture. If any shared pool cache is fixture-
    // dependent (or absorbs fixture state), the ordinary fixture's digest shifts after the
    // collision fixture has run. Same-process, so the caches persist across the swap — exactly
    // the contamination this probe is for. The pre-overlay run of THIS test is the reference:
    // if it already shows a shift, the shared pool cache could not be shared as-is.
    let col = sample(2, FIXTURE_COLLIDE, "COLLIDE");
    for line in &col {
        println!("COLLIDE {line}");
    }
    let post = sample(1, FIXTURE, "STD-POST");
    for line in &post {
        println!("N=1-POST {line}");
    }
    // The only judgment here: the ordinary fixture's digest is stable across the collision
    // probe. This is the condition-1 arbiter, printed whether or not it holds.
    let pre_digest = digest_of(n1.first().expect("N=1 line"));
    let post_digest = digest_of(post.first().expect("N=1-POST line"));
    println!(
        "STABILITY pre={pre_digest} post={post_digest} stable={}",
        pre_digest == post_digest
    );
    assert_eq!(
        pre_digest, post_digest,
        "the ordinary fixture's census digest changed after a module-name-colliding fixture ran \
         in the same process: a shared pool cache depends on (or absorbs) the fixture"
    );
    // Instrument sanity, the only other judgments in this file: the counters must actually see
    // the work the accusation names, on the LAST series too (one warm call after the collision
    // probe). INDEX_CLONES is asserted ZERO above, on the warm series -- that is the regression
    // gate, not sanity; the two counters here prove the counter mechanism is alive, so the zero
    // above cannot be a dead counter. (sample(1, STD-POST) resets at its start, so these hold
    // that one call.)
    let (_ic, _ie, vc, _vl, r, _b, _refusals) = compile_dag_diagnostic_census_cost_counts();
    assert!(
        vc >= 1,
        "scratch-map clone counter must see the calls in the post-collision series, got {vc}"
    );
    assert!(
        r >= 1,
        "disk-read counter must see the import closure on the post-collision call, got {r}"
    );
    println!("STEP0-DONE");
}
