// PROBE ONLY (probe/* refs; never ships). Answers stern-bat-401's executed-run question for PR-2
// prereq (b): does substituting in `transparent_alias_rhs` move the diagnostic produced at
// `variant_reference_inferred_node`?
//
// WHY THIS SHAPE, after five failed builds. My earlier attempts approximated the judgment with
// `compile_to_resolved` (which never invokes the census builtin that yields MethodNotFound) and with
// `gunbc run --entry <claim> --function <testfn>` (which returns rc=2 for every row, accepts included,
// because `test fn` rows are not evaluated that way). Both exited with a code and NO verdict, and I
// nearly read the absence as a result. This probe instead calls `compile_dag_diagnostic_census` -- the
// exact host builtin `gunbc.compile_census_probe`'s `total_blocking_count_for` /
// `blocking_subject_count_for` wrap, hence the exact function CI's claim rows judge with.
//
// The programs are READ FROM the authority CI runs (dag/test/claim/algebra_receiver_alias_witness.dag),
// and the inlined copies below are used only as a DRIFT RING: if the file's fixture no longer equals the
// copy, the test fails, so the probe can never quietly judge different programs than CI does. Reading, not
// copying, is what keeps this honest; the ring is what keeps the read honest.
//
// The census call is wrapped in catch_unwind and its ARM is printed: Observed(vec![]) (nothing blocking)
// and NotRunnable (could not measure) are different facts, and conflating them is the exact failure this
// exercise keeps running into.
use v1_compiler::cli_run::{compile_dag_diagnostic_census, CompileDiagnosticCensus};

const AUTHORITY: &str = "dag/test/claim/algebra_receiver_alias_witness.dag";

/// (test fn CI runs, the `data` name it passes, the copy used only as a drift ring)
const FIXTURES: [(&str, &str, &str); 4] = [
    (
        "nominal_return_is_an_acceptance_fixture",
        "nominal_return_source",
        "module alias_nominal_return\nimport std.types { Int, List }\nimport std.algebra { Empty, FreeMonoid }\nfn asks() -> FreeMonoid<Int> { [1, 2] |> fold(init: Empty, f: fn(acc, value) { acc |> list_push(value) }) }\n",
    ),
    (
        "alias_return_is_a_refusal_fixture",
        "alias_return_source",
        "module alias_return\nimport std.types { Int, List }\nimport std.algebra { Empty, FreeMonoid }\nfn asks() -> List<Int> { [1, 2] |> fold(init: Empty, f: fn(acc, value) { acc |> list_push(value) }) }\n",
    ),
    (
        "alias_field_is_a_refusal_fixture",
        "alias_field_source",
        "module alias_field\nimport std.types { Int, List }\nimport std.algebra { Empty, FreeMonoid }\ntype Box { values: List<Int> }\nfn asks() -> List<Int> { let b = Box { values: [1, 2] |> fold(init: Empty, f: fn(acc, value) { acc |> list_push(value) }) }; b.values }\n",
    ),
    (
        "alias_initializer_is_an_acceptance_fixture",
        "alias_initializer_source",
        "module alias_initializer\nimport std.types { Int, List }\nimport std.algebra { Empty, FreeMonoid }\nfn asks() -> List<Int> { let initial: FreeMonoid<Int> = Empty; [1, 2] |> fold(init: initial, f: fn(acc, value) { acc |> list_push(value) }) }\n",
    ),
];

/// Pull `data NAME: String = "..."` out of the authority and un-escape it. None = could not read it,
/// which the caller must treat as NO DATA rather than as a clean result.
fn authority_fixture(text: &str, name: &str) -> Option<String> {
    let needle = format!("data {name}: String = \"");
    let line = text.lines().find(|l| l.starts_with(&needle))?;
    let body = &line[needle.len()..];
    let end = body.rfind('"')?;
    Some(
        body[..end]
            .to_string()
            .replace("\\n", "\n")
            .replace("\\\"", "\""),
    )
}

fn guard_state() -> (bool, bool, bool) {
    // (env_gated, guard_live_for_this_run, observed)
    let ws = crate::helpers::workspace_root();
    let gated = match std::fs::read_to_string(ws.join("src/v1/stage0/src/v1_compiler_infer_env.rs"))
    {
        Ok(t) => t.contains("GUNBC_ALIAS_GUARD_OFF"),
        Err(e) => {
            println!("SEAM GUARD unreadable {e}");
            false
        }
    };
    // This binary is env-gated, so the run's guard state is set by the environment, not by a literal.
    let live = gated && std::env::var("GUNBC_ALIAS_GUARD_OFF").is_err();
    (gated, live, gated)
}

/// Call the builtin, never letting a panic take the whole run down silently.
fn census(source: &str) -> Result<CompileDiagnosticCensus, String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        compile_dag_diagnostic_census(source)
    }))
    .map_err(|p| {
        let m = p
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
            .unwrap_or_else(|| "<non-string panic payload>".to_string());
        m
    })
}

fn report(label: &str, source: &str) -> (i64, Vec<String>) {
    match census(source) {
        Err(panic_msg) => {
            println!("SEAM {label} ARM=PANIC msg={panic_msg}");
            (-1, vec![])
        }
        Ok(CompileDiagnosticCensus::NotRunnable(cause)) => {
            println!("SEAM {label} ARM=NotRunnable cause={cause}");
            (-1, vec![])
        }
        Ok(CompileDiagnosticCensus::Observed(rows)) => {
            let blocking: Vec<&_> = rows.iter().filter(|r| r.blocking).collect();
            let total: i64 = blocking.iter().map(|r| r.count).sum();
            println!(
                "SEAM {label} ARM=Observed rows={} blocking_rows={} blocking_total={total}",
                rows.len(),
                blocking.len()
            );
            let mut ids = Vec::new();
            for r in rows.iter() {
                let line = format!(
                    "class={} subject={} blocking={} count={}",
                    r.diagnostic_class, r.subject_name, r.blocking, r.count
                );
                println!("SEAM {label} ROW {line}");
                if r.blocking {
                    ids.push(line);
                }
            }
            (total, ids)
        }
    }
}

#[test]
fn alias_seam_a_b() {
    let (gated, live, observed) = guard_state();
    let neutered = gated && !live;
    println!(
        "SEAM GUARD env_gated={gated} guard_live={live} neutered={neutered} observed={observed}"
    );
    assert!(gated, "SEAM FATAL the guard is not env-gated on this tree: the A/B cannot run and is VOID, not a pass");
    let ws = crate::helpers::workspace_root();
    let path = ws.join(AUTHORITY);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("SEAM FATAL cannot read authority {AUTHORITY}: {e}"));

    let mut accepts: Vec<(String, i64)> = Vec::new();
    let mut refuses: Vec<(String, i64)> = Vec::new();
    let mut refusal_rows: Vec<String> = Vec::new();

    for (test_fn, data_name, ring_copy) in FIXTURES {
        let src = authority_fixture(&text, data_name).unwrap_or_else(|| {
            panic!("SEAM FATAL {data_name} is not a data string in {AUTHORITY}")
        });
        assert_eq!(
            src, ring_copy,
            "DRIFT: the probe's copy of {data_name} differs from the authority CI judges; \
             re-read the authority instead of trusting this file"
        );
        println!(
            "SEAM {test_fn} fixture_from={AUTHORITY} bytes={}",
            src.len()
        );
        let (total, ids) = report(test_fn, &src);
        if test_fn.contains("refusal") {
            refuses.push((test_fn.to_string(), total));
            refusal_rows = ids;
        } else {
            accepts.push((test_fn.to_string(), total));
        }
    }

    // MEASUREMENT FIRST, VERDICT LAST. An earlier revision asserted the control ring inline, so the
    // panic fired before SEAM SUMMARY could print and the run produced no measurement at all -- a
    // destroyed result is not a red result, and a red result is not a measurement. Every line that
    // carries information is printed unconditionally; the only panic is the final one, after the data
    // is on the record.
    //
    // -1 is the UNMEASURED sentinel (PANIC or NotRunnable arm: the instrument refused), never "0
    // blocking diagnostics". Treating -1 as "failed to refuse" would score an instrument refusal as a
    // program fact -- the exact conflation this lane keeps being burned by -- so unmeasurable fixtures
    // are reported as a named residual and excluded from the ring.
    println!("SEAM SUMMARY accepts={accepts:?} refuses={refuses:?}");
    for r in &refusal_rows {
        println!("SEAM REFUSAL_ROW {r}");
    }

    let mut unmeasured: Vec<String> = Vec::new();
    for (label, total) in accepts.iter().chain(refuses.iter()) {
        if *total < 0 {
            unmeasured.push(label.clone());
        }
    }
    if !unmeasured.is_empty() {
        println!(
            "SEAM RESIDUAL unmeasured_by_the_census_host={} (instrument refusal, NOT a clean compile)",
            unmeasured.join(",")
        );
    }

    if live && !neutered {
        let mut failures: Vec<String> = Vec::new();
        for (label, total) in &accepts {
            if *total >= 0 && *total != 0 {
                failures.push(format!("control: {label} must be clean, got {total}"));
            }
        }
        for (label, total) in &refuses {
            // Only a MEASURED refusal can falsify the ring; an unmeasured fixture is a residual above.
            if *total == 0 {
                failures.push(format!(
                    "control: {label} measured 0 blocking diagnostics, so the harness measures \
                     nothing here and the run is VOID for that fixture"
                ));
            }
        }
        if failures.is_empty() {
            println!("SEAM CONTROL_RING PASS accepts clean, refusals block");
        } else {
            for f in &failures {
                println!("SEAM CONTROL_RING FAIL {f}");
            }
            panic!(
                "SEAM CONTROL_RING {} failure(s); SEAM SUMMARY above is the measurement",
                failures.len()
            );
        }
    } else {
        println!("SEAM CONTROL_RING SKIPPED mutant tree: refusal totals are the measurement, not an invariant");
    }
}
