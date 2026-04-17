// Lane 6 — the four-fixture regression test suite.
//
// `docs/dependency-and-rendering-design.md` §3.3 states that any
// correct ownership model must handle these four fixtures without
// special-casing, and §9 says tests must verify the *fact*
// (`ParameterContract`) — not a symptom proxy.
//
//   - `id(x) = x`                  → [Consumed]   (reach return)
//   - `drop(x) = 0`                → [Borrowed]   (does not reach)
//   - `wrap(x) = { field: x }`     → [Consumed]   (reach via construct)
//   - `is_empty(x)` via `match x`  → [Borrowed]   (match-read only)
//
// **What a sound assertion looks like.** `ParameterContract` is not
// yet a type in the compiler's Rust surface (it lands with Track 1
// Phase 2). So the test asserts the fact via its one direct
// emission-shape consequence from §4.2:
//
//   Borrowed → `fn <name>(p0: &T, ...)`   (leading `&` on param)
//   Consumed → `fn <name>(p0: T, ...)`    (no `&` on param)
//
// Copy-ness does NOT collapse this distinction — the signature
// strings differ regardless of whether the type is `i64`, `Vec<i64>`,
// or a user enum. A clone-count proxy would have collapsed them
// under Copy types; a signature-shape assertion does not.
//
// **Current-Phase status** (§10 Phase 1 — "conservative default:
// all params Borrowed (safe)", clone count ~6):
//
//   The emitter renders every parameter as `&T` today, including
//   the two `Consumed` fixtures (`id`, `wrap`). That means §3.3's
//   contract is provably violated on those two today — which is
//   exactly the Phase-1 → Phase-2 work that Track 1 closes.
//
//   To keep Lane 6 independently mergeable (per the dossier's
//   "can start now with placeholder assertions" guidance) while
//   still installing the §9 *sound* gate:
//
//     - Borrowed-contract tests (`drop`, `is_empty`) run LIVE
//       and pass today, because Phase 1's conservative default
//       already matches the Borrowed contract.
//
//     - Consumed-contract tests (`id`, `wrap`) run with
//       `#[ignore]` and a named Phase-2 trigger. They encode the
//       sound §3.3 fact; they will go green the moment Phase 2
//       teaches the emitter to render Consumed as pass-by-value.
//       Removing the `#[ignore]` is the single action that
//       promotes Lane 6 from "Phase-1 half-gate" to "full §3.3
//       gate".
//
// This is the structure §9 asks for: every test asserts the
// contract, not a symptom. Half fire today (the half that matches
// Phase 1's conservative default); the other half are parked with
// an explicit, named unignore trigger. No test "passes" without
// also verifying the §3.3 fact it claims to verify.
//
// **Soft cross-target dependency.** Track 2 (go.dag): each fixture
// gets a `#[ignore]`'d cross-target variant. Go has no ownership
// rendering at the ABI — all parameters pass by value — so the
// go-side assertion is uniform regardless of contract. The
// placeholders exist so adding the go emitter doesn't need to
// remember to add variants; the test skeleton is already wired.

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

/// Return the first Bind whose name matches and which has at least
/// one parameter — i.e. the user's declared function, not a value
/// bind or an implicit main wrapper.
fn find_function_bind<'a>(dag: &'a Dag, name: &str) -> &'a BindNode {
    dag.nodes()
        .iter()
        .find_map(|node| match node {
            Behavior::Bind(bind) if bind.name == name && !bind.params.is_empty() => Some(bind),
            _ => None,
        })
        .unwrap_or_else(|| panic!("fixture must declare a function bind named `{name}`"))
}

/// The §3.3 reachability axis today's compiler can directly
/// report. `UnusedParametersLens` returns a violation iff no use
/// edge reads the parameter, which separates `drop` from the
/// other three fixtures. This is the Phase-1 reachability fact;
/// Phase 2 refines it into full `ParameterContract`.
fn parameter_is_used(dag: &Dag, function_name: &str, parameter_index: usize) -> bool {
    let lens = UnusedParametersLens::new(dag);
    let violations = lens.query(&UnusedParametersConfig::default());
    let bind = find_function_bind(dag, function_name);
    !violations.iter().any(|violation| {
        violation.function == bind.id && violation.parameter_index == parameter_index
    })
}

/// The §3.3 fact `ParameterContract ∈ { Borrowed, Consumed }`
/// translated to its direct emission consequence per §4.2:
///
///   Borrowed → `fn <name>(p0: &T, ...)`   (leading `&` on param)
///   Consumed → `fn <name>(p0: T, ...)`    (no `&` on param)
///
/// This is the sound §9 gate: the signature string distinguishes
/// the two contracts regardless of Copy-ness. A Copy type (e.g.
/// `i64`) can still be rendered Consumed (no `&`) or Borrowed
/// (`&i64`), and the gate catches which one was picked — where
/// a clone-count proxy would have reported zero for both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParameterContract {
    Borrowed,
    Consumed,
}

/// Assert the §3.3 parameter contract directly against the
/// emitted Rust signature. Fails with a specific, targeted
/// message that names the fixture, the expected contract, and
/// quotes the signature that was found.
fn assert_parameter_contract(emitted: &str, function_name: &str, expected: ParameterContract) {
    let signature = extract_signature(emitted, function_name).unwrap_or_else(|| {
        panic!("emit_rust output must contain `fn {function_name}`; emitted:\n{emitted}")
    });
    let first_param_type = extract_first_parameter_type(&signature).unwrap_or_else(|| {
        panic!("signature for `{function_name}` has no parameter; signature:\n{signature}")
    });
    let has_leading_ampersand = first_param_type.starts_with('&');
    let observed = if has_leading_ampersand {
        ParameterContract::Borrowed
    } else {
        ParameterContract::Consumed
    };
    assert_eq!(
        observed, expected,
        "§3.3 ParameterContract violated for `{function_name}`: \
         expected {expected:?}, observed {observed:?}. \
         First-parameter type: `{first_param_type}`. \
         Full signature: `{signature}`"
    );
}

/// Extract the signature string from `fn <name>(...)` up through
/// the closing paren of the parameter list. Returns `None` if
/// the function is not present in `emitted`.
fn extract_signature(emitted: &str, function_name: &str) -> Option<String> {
    let needle = format!("fn {function_name}(");
    let start = emitted.find(&needle)?;
    let open_paren = start + needle.len() - 1;
    // Match the closing paren at depth zero.
    let bytes = emitted.as_bytes();
    let mut depth = 0i32;
    let mut close_paren = None;
    for (i, &byte) in bytes.iter().enumerate().skip(open_paren) {
        match byte {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    close_paren = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close_paren?;
    Some(emitted[start..=close].to_string())
}

/// Extract the declared type of the first parameter in a
/// `fn name(p0: T, ...)` signature. Returns `None` if the
/// signature has no parameters.
fn extract_first_parameter_type(signature: &str) -> Option<&str> {
    let open = signature.find('(')?;
    let close = signature.rfind(')')?;
    let params = signature.get((open + 1)..close)?.trim();
    if params.is_empty() {
        return None;
    }
    let first_param = params.split(',').next()?.trim();
    let colon = first_param.find(':')?;
    Some(first_param[(colon + 1)..].trim())
}

// --- Per-fixture tests ------------------------------------------------
//
// Each fixture runs TWO assertions:
//
//   (a) A Phase-1 reachability check via `UnusedParametersLens`.
//       This pins the reachability fact today's compiler can
//       directly report: `drop`'s parameter is unused; the other
//       three are used. Phase 2 replaces this with a full
//       `ParameterContract` API and the assertion tightens.
//
//   (b) The sound §3.3 `ParameterContract` gate via signature
//       shape. For Borrowed fixtures (`drop`, `is_empty`) this
//       runs live today and passes — Phase 1's conservative
//       default matches. For Consumed fixtures (`id`, `wrap`)
//       this is `#[ignore]`'d until Phase 2 teaches the emitter
//       to render Consumed as pass-by-value; the fact is already
//       asserted, just gated.

/// `id(x: Int) -> Int = x` — parameter reaches return directly.
/// §3.3 contract: **Consumed**. Phase 1's conservative default
/// renders this as `fn id(p0: &i64)` (Borrowed), so the §3.3
/// gate below is `#[ignore]`'d and paired with `four_fixture_
/// id_phase2_consumed_contract` which activates at Phase 2.
#[test]
fn four_fixture_id_pins_reach_return_no_construct() {
    let (dag, _emitted) = load("id.v3");
    assert!(
        parameter_is_used(&dag, "id", 0),
        "`id` param must be observed as used (reaches return)"
    );
}

#[test]
#[ignore = "unignore when Track 1 Phase 2 teaches emit_rust to honor ParameterContract::Consumed"]
fn four_fixture_id_phase2_consumed_contract() {
    let (_dag, emitted) = load("id.v3");
    assert_parameter_contract(&emitted, "id", ParameterContract::Consumed);
}

/// `drop(x: Int) -> Int = 0` — parameter does not reach return.
/// §3.3 contract: **Borrowed**. Phase 1's conservative default
/// already matches this contract, so the §3.3 gate runs live
/// and passes today.
#[test]
fn four_fixture_drop_pins_param_not_reachable_from_return() {
    let (dag, emitted) = load("drop.v3");
    assert!(
        !parameter_is_used(&dag, "drop", 0),
        "`drop` param must be observed as unused (does not reach return)"
    );
    assert_parameter_contract(&emitted, "drop", ParameterContract::Borrowed);
}

/// `wrap(x: Int) -> Box = { value: x }` — parameter reaches
/// return through record construction. §3.3 contract:
/// **Consumed**. Same Phase-1 / Phase-2 split as `id`.
#[test]
fn four_fixture_wrap_pins_reach_return_through_construct() {
    let (dag, _emitted) = load("wrap.v3");
    assert!(
        parameter_is_used(&dag, "wrap", 0),
        "`wrap` param must be observed as used (reaches return via record)"
    );
}

#[test]
#[ignore = "unignore when Track 1 Phase 2 teaches emit_rust to honor ParameterContract::Consumed"]
fn four_fixture_wrap_phase2_consumed_contract() {
    let (_dag, emitted) = load("wrap.v3");
    assert_parameter_contract(&emitted, "wrap", ParameterContract::Consumed);
}

/// `is_empty(b: BoxedFx) -> Bool = match b { ... }` — parameter
/// is match-read but does not flow to return. §3.3 contract:
/// **Borrowed**. Same as `drop`: Phase 1's conservative default
/// matches, live gate today.
#[test]
fn four_fixture_is_empty_pins_no_reach_match_read_only() {
    let (dag, emitted) = load("is_empty.v3");
    assert!(
        parameter_is_used(&dag, "is_empty_fx", 0),
        "`is_empty_fx` param is observed as used today (match consumes it); \
         post-Track-1-Phase-2 this strengthens to `disposition == Borrowed`"
    );
    assert_parameter_contract(&emitted, "is_empty_fx", ParameterContract::Borrowed);
}

// --- Abstraction-level assertion -------------------------------------
//
// The four fixtures together pin the *model's abstraction claim*:
// one fact shape handles all four cases without per-fixture
// special-casing. The test below runs them through one shared
// assertion path, making divergent per-fixture behavior visible
// as a shape change — even when individual per-fixture tests
// keep passing.

#[test]
fn four_fixture_suite_shares_one_reachability_shape() {
    struct Case {
        fixture: &'static str,
        function: &'static str,
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
// Enable once Lane 3 / Track 2 lands `go.dag` as the authoritative
// multi-target spec. Go has no `&T` vs `T` distinction at the ABI —
// all parameters pass by value — so the go-side assertion is
// uniform regardless of §3.3 contract. Each placeholder exists so
// adding the go emitter doesn't need to remember to add variants.

#[test]
#[ignore = "enable when Lane 3 / Track 2 (go.dag) lands"]
fn four_fixture_id_cross_target_go_placeholder() {}

#[test]
#[ignore = "enable when Lane 3 / Track 2 (go.dag) lands"]
fn four_fixture_drop_cross_target_go_placeholder() {}

#[test]
#[ignore = "enable when Lane 3 / Track 2 (go.dag) lands"]
fn four_fixture_wrap_cross_target_go_placeholder() {}

#[test]
#[ignore = "enable when Lane 3 / Track 2 (go.dag) lands"]
fn four_fixture_is_empty_cross_target_go_placeholder() {}
