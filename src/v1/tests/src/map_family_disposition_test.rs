// PROBE ONLY -- never ships (branch probe/family-map-fixture, off the Acc head).
//
// Decides, by EXECUTED fixtures, which family the dominant Map-vs-Primitive refusals belong to:
// (i) a true disagreement in the source that the Acc correctly surfaces, or (ii) an Acc/producer
// defect where the compared "type" is a fabricated placeholder. Reading the code cannot decide
// this; parent ruled "Decide each with a minimal two-arm fixture, not by reading the code."
//
// The shape under test, from CI run 39146111607 (34 of 76 self-host refusals, all at the
// declaration site dag/std/algebra.dag fn map_insert<K,V>):
//   expected 'type variable A bound as Map(Primitive(K),Primitive(V)) (first argument)',
//   got        'Map(Primitive(Symbol),Product(...)) (later argument)'
// K and V are map_insert's OWN formal parameters, rendered as if they were concrete types.
//
// Why a v1 Rust test and not a `test fn` claim row: this is a measurement, not a lane assertion
// (the sibling map_insert_record_shaped_refusal_test.rs states in its own header that no required
// lane runs those assertions today). It calls the same host builtin the corpus census uses.
//
// ARM is printed for every fixture: Observed(vec![]) means nothing blocked, NotRunnable means the
// instrument could not measure, PANIC means it crashed. Those are three different facts and are
// never collapsed. Nothing here asserts before every measurement has been printed.
use v1_compiler::cli_run::{compile_dag_diagnostic_census, CompileDiagnosticCensus};

/// A fixture is measured by (label, source). `unmeasured` is the sentinel for "the instrument
/// refused", which is NOT the same as "0 blocking diagnostics".
fn measure(label: &str, source: &str) -> (i64, Vec<String>, String) {
    let arm = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        compile_dag_diagnostic_census(source)
    }));
    match arm {
        Err(e) => {
            let msg = e
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "<non-string panic>".to_string());
            println!("FAM {label} ARM=PANIC msg={msg}");
            (-1, vec![format!("PANIC {msg}")], "PANIC".to_string())
        }
        Ok(CompileDiagnosticCensus::NotRunnable(cause)) => {
            println!("FAM {label} ARM=NotRunnable cause={cause}");
            (
                -1,
                vec![format!("NotRunnable {cause}")],
                "NotRunnable".to_string(),
            )
        }
        Ok(CompileDiagnosticCensus::Observed(rows)) => {
            let blocking: Vec<_> = rows.iter().filter(|r| r.blocking).collect();
            let total: i64 = blocking.iter().map(|r| r.count).sum();
            let identities: Vec<String> = blocking
                .iter()
                .map(|r| format!("{}|{}", r.diagnostic_class, r.subject_name))
                .collect();
            for r in &blocking {
                println!(
                    "FAM {label} ROW class={} subject={} blocking={} count={}",
                    r.diagnostic_class, r.subject_name, r.blocking, r.count
                );
            }
            let arm_name = "Observed".to_string();
            println!(
                "FAM {label} ARM=Observed rows={} blocking_rows={} blocking_total={total}",
                rows.len(),
                blocking.len()
            );
            (total, identities, arm_name)
        }
    }
}

const HDR: &str = r#"
import std.types { Map, Int, String, Bool }
import std.algebra { empty_map, map_insert }
"#;

fn src(body: &str) -> String {
    format!("module probe.map_family{HDR}\n{body}\n")
}

#[test]
fn map_family_disposition_by_executed_fixture() {
    // F5 CONTROL, checked first because if it refuses, nothing else here is interpretable:
    // a map whose type is known from its parameter, no generic call producing a Map.
    let (ctl, _, ctl_arm) = measure(
        "F5_control_typed_map_insert",
        &src(r#"fn f5(m: Map<String, Int>) -> Map<String, Int> {
  map_insert(m: m, key: "k", value: 1)
}"#),
    );

    // F1 THE SHAPE UNDER TEST: the Map argument comes from an unannotated generic call to
    // empty_map(), i.e. through the call-type producer, with no annotation to instantiate against.
    let (f1, f1_ids, f1_arm) = measure(
        "F1_unannotated_empty_map_insert",
        &src(r#"fn f1() -> Map<String, Int> {
  map_insert(m: empty_map(), key: "k", value: 1)
}"#),
    );

    // F2 SAME PROGRAM, the same call bound through an annotation. If F1 refuses and F2 accepts, the
    // refusal is not about the program's types at all -- it is about how the unannotated call's
    // result type was produced. That is a producer defect whose home is upstream of the Acc.
    let (f2, f2_ids, f2_arm) = measure(
        "F2_annotated_empty_map_insert",
        &src(r#"fn f2() -> Map<String, Int> {
  let seed: Map<String, Int> = empty_map()
  map_insert(m: seed, key: "k", value: 1)
}"#),
    );

    // F3 A TRUE disagreement that must survive any fix: two concrete Maps with different arguments
    // flowing into one type variable. The Acc is right to refuse this.
    let (f3, f3_ids, f3_arm) = measure(
        "F3_two_distinct_concrete_maps_must_refuse",
        &src(r#"fn same_shape<T>(x: T, y: T) -> Bool { true }
fn f3(a: Map<String, Int>, b: Map<Int, String>) -> Bool {
  same_shape(x: a, y: b)
}"#),
    );

    // F4 THE CONTROL FOR F3: identical concrete Maps must NOT refuse. If this refuses, the Acc is
    // simply too strict and the whole family is (ii), regardless of what F1 says.
    let (f4, _f4_ids, f4_arm) = measure(
        "F4_two_identical_concrete_maps_must_accept",
        &src(r#"fn same_shape<T>(x: T, y: T) -> Bool { true }
fn f4(a: Map<String, Int>, b: Map<String, Int>) -> Bool {
  same_shape(x: a, y: b)
}"#),
    );

    // MEASUREMENT FIRST, VERDICT LAST -- every number above is on the record before any assert.
    println!("FAM SUMMARY F5(accept)={ctl}/{ctl_arm} F1={f1}/{f1_arm} F2(accept)={f2}/{f2_arm} F3(refuse)={f3}/{f3_arm} F4(accept)={f4}/{f4_arm}");
    println!("FAM F1_IDENTITIES {:?}", f1_ids);
    println!("FAM F2_IDENTITIES {:?}", f2_ids);
    println!("FAM F3_IDENTITIES {:?}", f3_ids);

    let mut verdict: Vec<String> = Vec::new();
    if ctl_arm != "Observed"
        || f1_arm != "Observed"
        || f2_arm != "Observed"
        || f3_arm != "Observed"
        || f4_arm != "Observed"
    {
        verdict.push(format!(
            "UNMEASURABLE(F5={ctl_arm},F1={f1_arm},F2={f2_arm},F3={f3_arm},F4={f4_arm}) -- no family verdict is possible"
        ));
    } else {
        if ctl != 0 {
            verdict.push(format!("CONTROL BROKEN: a parameter-typed map_insert refuses ({ctl}); every other reading here is void"));
        }
        if f1 > 0 && f2 == 0 {
            verdict.push("FAMILY=(ii) PRODUCER DEFECT: unannotated empty_map() refuses while the annotated form accepts. The compared Map(Primitive(K),Primitive(V)) is the callee's own formal parameters, not a type the program ever has. Home is upstream of the Acc.".to_string());
        } else if f1 == 0 && f2 == 0 {
            verdict.push("FAMILY=(not reproduced in isolation): neither arm refuses, so the 34 corpus sites need a larger context than this fixture (fold accumulator, or a second Map producer).".to_string());
        } else if f1 > 0 && f2 > 0 {
            verdict.push("FAMILY=(i) OR TOO-STRICT: the annotation does not help, so the refusal does not depend on the call-type producer.".to_string());
        }
        if f3 == 0 {
            verdict.push("F3 FAILS TO REFUSE: a genuine two-concrete-Map disagreement is admitted, so the Acc is too LENIENT on this shape -- the family verdict above cannot be (i) for it.".to_string());
        }
        if f4 > 0 {
            verdict.push("F4 REFUSES: identical concrete Maps are rejected; the Acc is too STRICT everywhere, which subsumes the family verdict.".to_string());
        }
    }
    for v in &verdict {
        println!("FAM VERDICT {v}");
    }
    // No panic unless the instrument could not measure at all: an unmeasurable run must not look
    // like a passing one.
    assert!(
        !(ctl_arm != "Observed"
            || f1_arm != "Observed"
            || f2_arm != "Observed"
            || f3_arm != "Observed"
            || f4_arm != "Observed"),
        "instrument could not measure; FAM VERDICT lines above name which arms refused to measure"
    );
}
