// PROBE ONLY -- never ships (branch probe/family-map-fixture, off the Acc head 97e47f8e562).
//
// Decides, by EXECUTED fixtures, which family the dominant Map-vs-Primitive refusals belong to:
// (i) a true disagreement in the source that the Acc correctly surfaces, or (ii) an Acc/producer
// defect where the compared "type" is a fabricated placeholder. Parent's ruling: "Decide each with
// a minimal two-arm fixture, not by reading the code."
//
// The shape under test, from CI run 39146111607 (34 of 76 self-host refusals at the declaration
// site dag/std/algebra.dag fn map_insert<K,V>):
//   expected 'type variable A bound as Map(Primitive(K),Primitive(V)) (first argument)',
//   got        'Map(Primitive(String),Product(...)) (later argument)'
// K and V are map_insert's OWN formal parameters, rendered as if concrete.
//
// SEVEN SEPARATE #[test]s, not one: an earlier single-test version lost two dispatches because a
// build-script heal ate the wall clock and NOTHING had been printed -- an unreadable result, not a
// red one. Per-test fns let the runner apply its own timeout to each fixture, so one slow shape
// cannot swallow the rest. ARM is printed for every fixture and -1 means UNMEASURED (PANIC or
// NotRunnable), never "0 blocking". Nothing asserts before its measurement has been printed.
use v1_compiler::cli_run::{compile_dag_diagnostic_census, CompileDiagnosticCensus};

fn measure(label: &str, source: &str) {
    // BEGIN before the call: if a shape hangs, the log names the fixture that ate the clock.
    println!("FAM {label} BEGIN");
    let t0 = std::time::Instant::now();
    let arm = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        compile_dag_diagnostic_census(source)
    }));
    let secs = t0.elapsed().as_secs();
    match arm {
        Err(e) => {
            let msg = e
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "<non-string panic>".to_string());
            println!("FAM {label} ARM=PANIC secs={secs} msg={msg}");
        }
        Ok(CompileDiagnosticCensus::NotRunnable(cause)) => {
            println!("FAM {label} ARM=NotRunnable secs={secs} cause={cause}");
        }
        Ok(CompileDiagnosticCensus::Observed(rows)) => {
            let blocking: Vec<_> = rows.iter().filter(|r| r.blocking).collect();
            let total: i64 = blocking.iter().map(|r| r.count).sum();
            for r in &blocking {
                println!(
                    "FAM {label} ROW class={} subject={} blocking={} count={}",
                    r.diagnostic_class, r.subject_name, r.blocking, r.count
                );
            }
            println!(
                "FAM {label} ARM=Observed secs={secs} rows={} blocking_rows={} blocking_total={total}",
                rows.len(),
                blocking.len()
            );
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

// F5 CONTROL, read first: if a parameter-typed map_insert refuses, nothing else here is interpretable.
#[test]
fn fam_f5_control_typed_map_insert() {
    measure(
        "F5_control_typed_map_insert",
        &src(r#"fn f5(m: Map<String, Int>) -> Map<String, Int> {
  map_insert(m: m, key: "k", value: 1)
}"#),
    );
}

// F1 THE SHAPE UNDER TEST: the Map argument comes from an unannotated generic call to empty_map().
#[test]
fn fam_f1_unannotated_empty_map() {
    measure(
        "F1_unannotated_empty_map_insert",
        &src(r#"fn f1() -> Map<String, Int> {
  map_insert(m: empty_map(), key: "k", value: 1)
}"#),
    );
}

// F2 the same program with the call bound through an annotation. F1 refuses + F2 accepts means the
// refusal is about how the unannotated call's result type was PRODUCED, not about the program.
#[test]
fn fam_f2_annotated_empty_map() {
    measure(
        "F2_annotated_empty_map_insert",
        &src(r#"fn f2() -> Map<String, Int> {
  let seed: Map<String, Int> = empty_map()
  map_insert(m: seed, key: "k", value: 1)
}"#),
    );
}

// F3 a TRUE disagreement that must survive any fix: one type variable, two distinct concrete Maps.
#[test]
fn fam_f3_two_distinct_concrete_maps() {
    measure(
        "F3_two_distinct_concrete_maps_must_refuse",
        &src(r#"fn same_shape<T>(x: T, y: T) -> Bool { true }
fn f3(a: Map<String, Int>, b: Map<Int, String>) -> Bool {
  same_shape(x: a, y: b)
}"#),
    );
}

// F4 control for F3: identical concrete Maps must NOT refuse. If it refuses, the Acc is simply too
// strict and that subsumes the whole family verdict.
#[test]
fn fam_f4_two_identical_concrete_maps() {
    measure(
        "F4_two_identical_concrete_maps_must_accept",
        &src(r#"fn same_shape<T>(x: T, y: T) -> Bool { true }
fn f4(a: Map<String, Int>, b: Map<String, Int>) -> Bool {
  same_shape(x: a, y: b)
}"#),
    );
}

// F6 WARM-WOLF'S EXACT SHAPE, copied verbatim from
// dag/test/claim/constructor_tag_route_probe_red.dag (identity
// same_arm_one_t_two_coproducts_must_refuse), which Charter 4 enrols as ExpectedRed on the claim
// that it ACCEPTS and answers false on main. F3 is a Map proxy and cannot answer whether PR-2
// flips their rows; only this shape can.
#[test]
fn fam_f6_one_t_two_coproducts_warm_wolf_shape() {
    measure(
        "F6_one_t_two_distinct_coproducts_warm_wolf_shape",
        &src(r#"type Pa = P1 { a: Int } | P2
type Pb = Q1 { a: Int } | Q2
type ConstructorTag<T> = ConstructorTag { name: String }
fn tag_of<T>(v: T) -> ConstructorTag<T> {
  ConstructorTag { name: discriminant(v: v) }
}
fn same_arm<T>(a: T, b: T) -> Bool {
  tag_of(v: a) == tag_of(v: b)
}
fn f6() -> Bool {
  same_arm(a: P1 { a: 1 }, b: Q1 { a: 1 })
}"#),
    );
}

// F7 control for F6: the SAME coproduct twice must not refuse; without it an F6 refusal could be
// over-strictness on any constructor comparison rather than a real tag distinction.
#[test]
fn fam_f7_one_t_same_coproduct() {
    measure(
        "F7_one_t_same_coproduct_must_accept",
        &src(r#"type Pa = P1 { a: Int } | P2
type ConstructorTag<T> = ConstructorTag { name: String }
fn tag_of<T>(v: T) -> ConstructorTag<T> {
  ConstructorTag { name: discriminant(v: v) }
}
fn same_arm<T>(a: T, b: T) -> Bool {
  tag_of(v: a) == tag_of(v: b)
}
fn f7(x: Pa, y: Pa) -> Bool {
  same_arm(a: x, b: y)
}"#),
    );
}
