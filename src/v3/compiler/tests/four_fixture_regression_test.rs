// Lane 6 — the four-fixture regression test suite.
//
// `docs/dependency-and-rendering-design.md` §3.3 states that any
// correct ownership model must handle these four fixtures without
// special-casing:
//
//   - `id(x) = x`                  (reach return, no construct)
//   - `drop(x) = 0`                (param not reachable from return)
//   - `wrap(x) = { field: x }`     (reach return through construct)
//   - `is_empty(x)` via `match x`  (no reach from return, read-only)
//
// Together they form the minimum-sufficient test set converged on
// across the reviewer rounds: one fact shape must handle all four
// reachability patterns. Any future regression in the ownership
// model shows up as a per-fixture failure here within one CI run.
//
// **What this test pins today.** Two properties per fixture:
//
//   1. The fixture compiles with zero diagnostics — the model's
//      expressiveness holds.
//   2. `UnusedParametersLens` classifies the parameter as
//      used-or-unused matching the fixture's §3.3 intent. This is
//      the *reachability fact available today*. It splits `drop`
//      from the other three — which is what today's model sees.
//   3. The emit_rust clone count matches a pinned ratchet (today
//      each of the four emits zero clones; see per-test notes).
//
// **What this test tightens to post-Track-1-Phase-2.** Today's
// lens only splits "param used at all" from "param unused." The
// §3.3 design distinguishes four cases on *two* axes:
//
//         | reaches return | doesn't reach return |
//   ------|----------------|----------------------|
//   used  | id, wrap       | is_empty             |
//   unused|     —          | drop                 |
//
// When Track 1 Phase 2 lands with `ParameterDisposition ∈
// { Consumed, Borrowed }`, swap the `parameter_is_used` assertions
// for `disposition == Consumed | Borrowed` per the §3.3 table. The
// test *interface* — four named tests, per-fixture
// assertions — stays the same; only the assertions tighten.
//
// **Soft dependencies.**
//
//   - Track 1 / ownership Phase 2: the upgrade above.
//   - Track 2 / go.dag: adds a cross-target variant for each
//     fixture (currently `#[ignore]`d; flips on when go.dag is the
//     authoritative multi-target spec).

use std::path::PathBuf;

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Behavior, BindNode};
use v3_compiler::emit_rust::emit_rust;
use v3_compiler::lens_unused_parameters::{UnusedParametersConfig, UnusedParametersLens};
use v3_compiler::Dag;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("four_fixture_pressure")
}

fn load(file: &str) -> (Dag, String) {
    let path = fixtures_dir().join(file);
    let source = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read fixture {}: {e}", path.display()));
    let dag = compile_to_dag(&source, path.to_string_lossy().as_ref())
        .unwrap_or_else(|e| panic!("fixture {} must compile cleanly, got: {e:?}", file));
    assert!(
        dag.diagnostics().is_empty(),
        "fixture {file} produced diagnostics: {:?}",
        dag.diagnostics()
    );
    let emitted = emit_rust(&dag).expect("emit_rust");
    (dag, emitted)
}

fn clone_call_count(source: &str) -> usize {
    source.match_indices(".clone(").count()
}

/// Return the first Bind whose name matches and which has at least
/// one parameter — i.e. the user's declared function, not a value
/// bind or an implicit main wrapper. Panics with a useful message
/// if none is found so the test failure points at the right thing.
fn find_function_bind<'a>(dag: &'a Dag, name: &str) -> &'a BindNode {
    dag.nodes()
        .iter()
        .find_map(|node| match node {
            Behavior::Bind(bind) if bind.name == name && !bind.params.is_empty() => Some(bind),
            _ => None,
        })
        .unwrap_or_else(|| panic!("fixture must declare a function bind named `{name}`"))
}

/// The §3.3 abstraction axis we can observe TODAY, before Track 1
/// Phase 2 ships `ParameterDisposition`. The lens reports a
/// parameter as "unused" iff no use-edge reads it. For the four
/// §3.3 fixtures this is sufficient to separate `drop` (the only
/// "param unused") from the other three.
fn parameter_is_used(dag: &Dag, function_name: &str, parameter_index: usize) -> bool {
    let lens = UnusedParametersLens::new(dag);
    let violations = lens.query(&UnusedParametersConfig::default());
    let bind = find_function_bind(dag, function_name);
    !violations.iter().any(|violation| {
        violation.function == bind.id && violation.parameter_index == parameter_index
    })
}

// --- Per-fixture tests ------------------------------------------------

/// `id(x: Int) -> Int = x` — parameter reaches return directly.
///
/// §3.3 disposition: **Consumed**. Once Track 1 Phase 2 lands,
/// tighten the `parameter_is_used` assertion to
/// `disposition == Consumed`.
#[test]
fn four_fixture_id_pins_reach_return_no_construct() {
    let (dag, emitted) = load("id.v3");
    assert!(
        parameter_is_used(&dag, "id", 0),
        "`id` param must be observed as used (reaches return)"
    );
    // Today: i64 is Copy, param is passed by reference, no clone.
    // Ratchet at the current observed count so any regression
    // that introduces a clone for this reachability pattern fires.
    const ID_CLONE_COUNT: usize = 0;
    assert_eq!(
        clone_call_count(&emitted),
        ID_CLONE_COUNT,
        "emit_rust clone count for `id` changed — review whether \
         the ownership model regressed. Emitted source:\n{emitted}"
    );
}

/// `drop(x: Int) -> Int = 0` — parameter does not reach return.
///
/// §3.3 disposition: **Borrowed**. Today the
/// `UnusedParametersLens` is strong enough to name this case
/// directly: the parameter is *unused*. Track 1 Phase 2 will
/// distinguish "unused entirely" (this) from "read but not
/// forwarded" (`is_empty`). Both are Borrowed.
#[test]
fn four_fixture_drop_pins_param_not_reachable_from_return() {
    let (dag, emitted) = load("drop.v3");
    assert!(
        !parameter_is_used(&dag, "drop", 0),
        "`drop` param must be observed as unused (does not reach return)"
    );
    const DROP_CLONE_COUNT: usize = 0;
    assert_eq!(
        clone_call_count(&emitted),
        DROP_CLONE_COUNT,
        "emit_rust clone count for `drop` changed — review whether \
         the ownership model regressed. Emitted source:\n{emitted}"
    );
}

/// `wrap(x: Int) -> Box = { value: x }` — parameter reaches
/// return through record construction.
///
/// §3.3 disposition: **Consumed**. This is the "embed into Conj"
/// variant of Consumed; distinct from `id`'s direct pass-through
/// because the parameter flows through a construction node before
/// reaching the result port.
#[test]
fn four_fixture_wrap_pins_reach_return_through_construct() {
    let (dag, emitted) = load("wrap.v3");
    assert!(
        parameter_is_used(&dag, "wrap", 0),
        "`wrap` param must be observed as used (reaches return via record)"
    );
    const WRAP_CLONE_COUNT: usize = 0;
    assert_eq!(
        clone_call_count(&emitted),
        WRAP_CLONE_COUNT,
        "emit_rust clone count for `wrap` changed — review whether \
         the ownership model regressed. Emitted source:\n{emitted}"
    );
}

/// `is_empty(b: BoxedInt) -> Bool = match b { ... }` — parameter
/// is inspected (read-only) but does not flow into the return.
///
/// §3.3 disposition: **Borrowed**. Today the lens reports this
/// parameter as *used* (the match consumes it), but the return
/// value is independent of the payload. Track 1 Phase 2 is what
/// separates this from `id`/`wrap` (used AND reaches) vs
/// `is_empty` (used AND does NOT reach). Until then, this test
/// anchors the "compiles + shape holds + clone count stable"
/// contract and the upgrade swaps the `parameter_is_used`
/// assertion for a Borrowed disposition check.
#[test]
fn four_fixture_is_empty_pins_no_reach_match_read_only() {
    let (dag, emitted) = load("is_empty.v3");
    assert!(
        parameter_is_used(&dag, "is_empty_fx", 0),
        "`is_empty_fx` param is observed as used today (match consumes it); \
         post-Track-1-Phase-2 this strengthens to `disposition == Borrowed`"
    );
    const IS_EMPTY_CLONE_COUNT: usize = 0;
    assert_eq!(
        clone_call_count(&emitted),
        IS_EMPTY_CLONE_COUNT,
        "emit_rust clone count for `is_empty_fx` changed — review whether \
         the ownership model regressed. Emitted source:\n{emitted}"
    );
}

// --- Abstraction-level assertion -------------------------------------
//
// The four fixtures together pin the *model's abstraction claim*:
// one fact shape handles all four cases without per-fixture
// special-casing. The test below captures that claim directly —
// it runs the same machinery against all four fixtures and asserts
// each one's *observed* reachability fact matches the §3.3 table.
//
// If someone adds a `match fixture_name { ... }` somewhere in the
// pipeline to special-case one of these four, the per-fixture
// tests above will keep passing but the model's abstraction will
// have broken. The right diagnostic for that regression is usually
// a code-review catch, not a test; this test's value is that it
// runs the four fixtures through one shared assertion path, making
// divergent per-fixture behavior visible as a shape change.

#[test]
fn four_fixture_suite_shares_one_reachability_shape() {
    struct Case {
        fixture: &'static str,
        function: &'static str,
        // §3.3 intent. Today's lens can only observe `used` vs
        // `unused`; Track 1 Phase 2 upgrades this to a full
        // disposition and this field flips to ParameterDisposition.
        param_reaches_anywhere: bool,
    }

    let cases = [
        Case {
            fixture: "id.v3",
            function: "id",
            param_reaches_anywhere: true,
        },
        Case {
            fixture: "drop.v3",
            function: "drop",
            param_reaches_anywhere: false,
        },
        Case {
            fixture: "wrap.v3",
            function: "wrap",
            param_reaches_anywhere: true,
        },
        Case {
            fixture: "is_empty.v3",
            function: "is_empty_fx",
            param_reaches_anywhere: true,
        },
    ];

    for case in cases {
        let (dag, _) = load(case.fixture);
        let used = parameter_is_used(&dag, case.function, 0);
        assert_eq!(
            used, case.param_reaches_anywhere,
            "§3.3 reachability regressed for `{}`: fixture predicts used={}, lens reports used={}",
            case.function, case.param_reaches_anywhere, used
        );
    }
}

// --- Cross-target placeholders (Track 2 soft dependency) --------------
//
// Once Lane 3 / Track 2 lands `go.dag` as the authoritative
// multi-target spec, these `#[ignore]`d tests get their `#[ignore]`
// removed and the assertion body filled in with the go emitter's
// clone-equivalent metric (Go has no clones — the analogue is "any
// insertion of an ownership-rendering decision" where the §3.3
// model says the parameter is Borrowed).
//
// The tests exist today, gated, so that adding go.dag doesn't need
// to remember to add the variant — the test skeleton is already
// here.

#[test]
#[ignore = "enable when Lane 3 / Track 2 (go.dag) lands"]
fn four_fixture_id_cross_target_go_placeholder() {
    // Parity claim: §3.3's `id` fixture emits to Go with zero
    // ownership-rendering decisions (Go has no borrow/consume
    // distinction at the ABI; passing by value is uniform).
}

#[test]
#[ignore = "enable when Lane 3 / Track 2 (go.dag) lands"]
fn four_fixture_drop_cross_target_go_placeholder() {}

#[test]
#[ignore = "enable when Lane 3 / Track 2 (go.dag) lands"]
fn four_fixture_wrap_cross_target_go_placeholder() {}

#[test]
#[ignore = "enable when Lane 3 / Track 2 (go.dag) lands"]
fn four_fixture_is_empty_cross_target_go_placeholder() {}
