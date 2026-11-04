// THE CENSUS RUNS ON THE COMPILED GRAPH'S DIAGNOSTICS AND REQUESTS NO RENDER SUBJECT
// (mgr ruling msg_546ed998: "remove the census's demand for a render subject entirely; it runs
// on the compiled graph's diagnostics and requests none"). This file holds the three things that
// ruling needs committed as executed evidence:
//
//  1. THE FIVE PARSE-ERROR FIXTURES REPORT THEIR LOCATED PARSE DIAGNOSTICS. Each fixture below is
//     the same String the fleet witness carries (byte-identical `data` rows from
//     dag/test/claim/{arity_zero_import_refusal_witness, duplicate_record_field_refusal_witness,
//     generic_effectful_declaration_wall_witness}.dag). Before the fix each one refused the census
//     with "render selection refused: none of the N compiled module(s) is declared in fixture
//     source 'test.dag'" -- a refusal whose subject was a render, not a diagnostic -- and the
//     fold `census_blocking_count_for_class` turned that refusal into none, so the witnesses
//     judged -1 instead of the located parse errors they exist to judge. Each RED below fails at
//     the pre-fix head and passes at this one.
//  2. THE PARSE-CLEAN CONTROLS STAY GREEN: the same fixtures with the defect removed must report
//     an Observed census with NO ParseError row. If a fix made everything report a parse error,
//     the REDs would look right and the controls would catch it.
//  3. RENDEREVERYMODULE IS NEVER REACHED BY A CENSUS. The counters at
//     `v1_compiler::cli_run::every_module_render_calls` /
//     `non_empty_selected_render_calls` count every render REQUEST passing through the host seam
//     `emit_resolved_for_target_counting`, by requested shape (whole-graph vs a non-empty
//     selected-basenames list). The widening the census route must never take is a whole-graph
//     render -- it re-buys the cost the empty selection exists to not-pay. Each census call below
//     runs between counter snapshots and requires the delta to be zero. The positive control in
//     `a_whole_graph_render_really_moves_the_counter` proves neither counter is stuck at zero: it
//     drives all three shapes through the same seam -- the census's own empty-selection shape
//     (moves neither counter), a whole-graph request (moves the whole-graph counter by exactly
//     one), and a non-empty selected-basenames request (moves the selected counter by exactly
//     one) -- so a counter wired to nothing cannot pass this file either.
//
// RUNG, stated honestly (same rule this branch's measurement test learned; review 84038): this is
// a cargo integration test in the v1 seed, and NO CI STEP RUNS IT -- the required path runs no v1
// unit tests (gunbc.rung_drop v1_required_lanes_withdrawn). It is executed by whoever runs
// `cargo test -p v1-compiler --test census_no_render_subject_control -- --exact <name>`, which is
// how its REDs were established: each RED fails at the pre-fix head and only when run.
use std::rc::Rc;
use v1_compiler::cli_run::{
    compile_dag_diagnostic_census, emit_resolved_for_target_counting, every_module_render_calls,
    non_empty_selected_render_calls, CompileDiagnosticCensus,
};
use v1_compiler::v1_compiler_compile::{
    compile_to_resolved, RenderTarget, RustModuleRenderSelection, SourceFile,
};

// The five RED fixtures, verbatim from the fleet witness `data` rows.
const RED_ARITY_ZERO_MEMBER_LIST: &str =
    "module probe_arity_zero_import\nimport std.types {\n}\nfn answers() -> Int { 7 }\n";
const RED_DUPLICATE_RECORD_FIELD: &str = "module probe_duplicate_record_field\ntype DupPair {\n  a: Int\n  b: Int\n}\nfn answers() -> Int {\n  let p = DupPair { a: 1, a: 2, b: 3 }\n  p.a\n}\n";
const RED_GEW_GENERIC_EFFECTFUL: &str = "module gew_wall_generic\n\ntype GewNet {\n  id: Int\n}\n\nfn gew_generic_effectful<T>(x: T) -> T\n  uses net: GewNet\n{\n  x\n}\n";
const RED_GEW_LEGACY_FUNC: &str = "module gew_wall_legacy\n\nfunc gew_legacy() -> Int {\n  1\n}\n";
// The removed-func wall's second refusal fixture is the generic-effectful one under the wall's
// own name; the duplicate list covers the five distinct RED rows the mgr listed: arity, dup x2
// (both refusal witnesses share this source), generic-effectful x2 (the parse refusal and the
// removed-func spelling refusal use the wall's fixtures).
//
// The parse-clean controls, verbatim from the same witness files' green rows.
const GREEN_ONE_MEMBER: &str =
    "module probe_arity_one_import\nimport std.types { Int }\nfn answers() -> Int { 7 }\n";
const GREEN_EACH_FIELD_ONCE: &str = "module probe_single_record_field\ntype DupPair {\n  a: Int\n  b: Int\n}\nfn answers() -> Int {\n  let p = DupPair { a: 1, b: 3 }\n  p.a\n}\n";
const GREEN_GEW_PLAIN_EFFECTFUL: &str = "module gew_wall_effectful\n\ntype GewNet2 {\n  id: Int\n}\n\nfn gew_plain_effectful(x: Int) -> Int\n  uses net: GewNet2\n{\n  net.id\n}\n";

struct CensusObservation {
    rows: Vec<(String, String, bool, i64)>,
    render_calls_during: u64,
    non_empty_selected_during: u64,
}

fn observe_census(source: &str) -> CensusObservation {
    let before = every_module_render_calls();
    let selected_before = non_empty_selected_render_calls();
    let census = compile_dag_diagnostic_census(source);
    let after = every_module_render_calls();
    match census {
        CompileDiagnosticCensus::Observed(rows) => CensusObservation {
            rows: rows
                .into_iter()
                .map(|r| (r.diagnostic_class, r.subject_name, r.blocking, r.count))
                .collect(),
            render_calls_during: after - before,
            non_empty_selected_during: non_empty_selected_render_calls() - selected_before,
        },
        CompileDiagnosticCensus::NotRunnable(cause) => panic!(
            "the census refused instead of reporting the compiled graph's diagnostics: {cause}"
        ),
    }
}

fn parse_error_rows(obs: &CensusObservation) -> Vec<(String, bool, i64)> {
    obs.rows
        .iter()
        .filter(|(class, _, _, _)| class == "ParseError")
        .map(|(_, subject, blocking, count)| (subject.clone(), *blocking, *count))
        .collect()
}

/// THE FIVE REDS: each parse-error fixture reports at least one located ParseError row and the
/// census never reaches a whole-graph render. At the pre-fix head each of these panics in
/// `observe_census` with the render-selection refusal; that panic is the RED.
#[test]
fn the_arity_zero_member_list_fixture_reports_its_parse_diagnostics() {
    let obs = observe_census(RED_ARITY_ZERO_MEMBER_LIST);
    assert_eq!(
        obs.render_calls_during, 0,
        "a census call reached a RenderEveryModule render"
    );
    assert_eq!(
        obs.non_empty_selected_during, 0,
        "a census call requested a render with a non-empty selected-basenames list"
    );
    assert!(
        !parse_error_rows(&obs).is_empty(),
        "the arity-zero import member list reported no ParseError row; rows were {:?}",
        obs.rows
    );
}

#[test]
fn the_duplicate_record_field_fixture_reports_its_parse_diagnostics() {
    let obs = observe_census(RED_DUPLICATE_RECORD_FIELD);
    assert_eq!(
        obs.render_calls_during, 0,
        "a census call reached a RenderEveryModule render"
    );
    assert_eq!(
        obs.non_empty_selected_during, 0,
        "a census call requested a render with a non-empty selected-basenames list"
    );
    assert!(
        !parse_error_rows(&obs).is_empty(),
        "the duplicate record field reported no ParseError row; rows were {:?}",
        obs.rows
    );
}

#[test]
fn the_parse_error_rows_name_the_field_and_both_positions() {
    // The duplicate-field witness pair (a_record_literal... and the_refusal_names_the_field...)
    // is carried by ONE source; the second refusal witness checks the located positions, so this
    // RED asserts the ParseError row carries the SAME located subject the fleet witness matches
    // exactly: `duplicate record field_refusal_witness.dag`'s the_refusal_names_the_field_and_both_positions
    // requires the subject "duplicate field 'a' in record literal: first at 7:21, again at 7:27"
    // -- the field name as a whole token inside quotes, and both line:col positions of the two
    // `a:` assignments in RED_DUPLICATE_RECORD_FIELD. Matching the whole subject string (review
    // 85075 finding 2) makes this discriminating: a refusal that lost the field name, either
    // position, or the whole-record context reads different here. A `contains("a")` read as
    // almost anything and is what this replaces.
    let obs = observe_census(RED_DUPLICATE_RECORD_FIELD);
    assert_eq!(
        obs.render_calls_during, 0,
        "a census call reached a RenderEveryModule render"
    );
    assert_eq!(
        obs.non_empty_selected_during, 0,
        "a census call requested a render with a non-empty selected-basenames list"
    );
    let rows = parse_error_rows(&obs);
    const LOCATED_SUBJECT: &str =
        "duplicate field 'a' in record literal: first at 7:21, again at 7:27";
    let names_field_and_both_positions = rows
        .iter()
        .any(|(subject, blocking, _)| *blocking && subject == LOCATED_SUBJECT);
    assert!(
        names_field_and_both_positions,
        "the duplicate-field refusal did not carry the located subject {LOCATED_SUBJECT:?}; \
         ParseError rows were {rows:?}; all rows {:?}",
        obs.rows
    );
}

#[test]
fn the_generic_effectful_declaration_fixture_reports_its_parse_diagnostics() {
    let obs = observe_census(RED_GEW_GENERIC_EFFECTFUL);
    assert_eq!(
        obs.render_calls_during, 0,
        "a census call reached a RenderEveryModule render"
    );
    assert_eq!(
        obs.non_empty_selected_during, 0,
        "a census call requested a render with a non-empty selected-basenames list"
    );
    assert!(
        !parse_error_rows(&obs).is_empty(),
        "the generic effectful declaration reported no ParseError row; rows were {:?}",
        obs.rows
    );
}

#[test]
fn the_removed_func_spelling_fixture_reports_its_parse_diagnostics() {
    let obs = observe_census(RED_GEW_LEGACY_FUNC);
    assert_eq!(
        obs.render_calls_during, 0,
        "a census call reached a RenderEveryModule render"
    );
    assert_eq!(
        obs.non_empty_selected_during, 0,
        "a census call requested a render with a non-empty selected-basenames list"
    );
    assert!(
        !parse_error_rows(&obs).is_empty(),
        "the removed `func` spelling reported no ParseError row; rows were {:?}",
        obs.rows
    );
}

/// THE CONTROLS: parse-clean fixtures report an Observed census carrying NO ParseError row.
#[test]
fn the_parse_clean_controls_report_no_parse_error() {
    for (name, source) in [
        ("green_one_member", GREEN_ONE_MEMBER),
        ("green_each_field_once", GREEN_EACH_FIELD_ONCE),
        ("green_gew_plain_effectful", GREEN_GEW_PLAIN_EFFECTFUL),
    ] {
        let obs = observe_census(source);
        assert_eq!(
            obs.render_calls_during, 0,
            "a census call on {name} reached a RenderEveryModule render"
        );
        assert_eq!(
            obs.non_empty_selected_during, 0,
            "a census call on {name} requested a render with a non-empty selected-basenames list"
        );
        assert!(
            parse_error_rows(&obs).is_empty(),
            "the parse-clean control {name} reported ParseError rows: {:?}; all rows {:?}",
            parse_error_rows(&obs),
            obs.rows
        );
    }
}

/// THE CONTROL OF THE CONTROL: the counters are wired to real render requests. Through the same
/// seam the census route takes (`emit_resolved_for_target_counting`), a whole-graph request moves
/// the whole-graph counter by exactly one, an empty-selection request moves neither, and a
/// non-empty selected-basenames request moves the selected counter by exactly one. Together these
/// three legs pin each counter to the shape it names, so the zero-delta assertions above cannot
/// be passing on a counter that never increments.
#[test]
fn a_whole_graph_render_really_moves_the_counter() {
    let sources = Rc::new(im::vector![Rc::new(SourceFile {
        path: "census_counter_positive_control.dag".to_string(),
        content: "module census_counter_positive_control\ndata answer: Int = 7\n".to_string(),
    })]);
    let resolved = compile_to_resolved(sources.clone());

    // Leg 1: the census route's own shape -- a `RenderSelectedMirrors` with no basenames -- moves
    // NEITHER counter. This is the request `compile_fixture_rendering_nothing` makes, so the
    // census-side zeros above are the same shape proven here, not an absence of evidence. No file
    // assertion here on purpose: an empty selection still emits the artifact scaffold (the
    // selection gates which MODULES render, not whether the artifact exists -- see
    // `selected_artifact_plan`), so "the census emitted no module files" is a claim about the
    // census rows' content, which the per-census tests already pin, not about these counters.
    // Legs 2 and 3 carry the not-stuck-at-zero proof by moving their counters exactly one.
    let before = every_module_render_calls();
    let selected_before = non_empty_selected_render_calls();
    let _empty_selected = emit_resolved_for_target_counting(
        resolved.clone(),
        RenderTarget::Rust,
        Rc::new(RustModuleRenderSelection::RenderSelectedMirrors {
            basenames: Rc::new(im::Vector::new()),
        }),
    );
    assert_eq!(
        every_module_render_calls() - before,
        0,
        "an empty-selection render moved the whole-graph counter, so the counters do not name \
         the shapes they count"
    );
    assert_eq!(
        non_empty_selected_render_calls() - selected_before,
        0,
        "an empty-basenames render moved the non-empty-selected counter"
    );

    // Leg 2: a whole-graph request through the same seam moves the whole-graph counter by exactly
    // one and the selected counter not at all.
    let before = every_module_render_calls();
    let selected_before = non_empty_selected_render_calls();
    let full = emit_resolved_for_target_counting(
        resolved.clone(),
        RenderTarget::Rust,
        Rc::new(RustModuleRenderSelection::RenderEveryModule),
    );
    assert_eq!(
        non_empty_selected_render_calls() - selected_before,
        0,
        "a whole-graph render moved the selected-render counter, so the two counters are not \
         counting the shapes they name"
    );
    assert_eq!(
        every_module_render_calls() - before,
        1,
        "a whole-graph RenderEveryModule render did not move the counter by exactly one"
    );
    assert!(
        !full.files.is_empty(),
        "the whole-graph render emitted no files, so the counter's zero-delta assertions above \
         would not distinguish a render from a refusal"
    );

    // Leg 3: a non-empty selected-basenames request moves the selected counter by exactly one and
    // the whole-graph counter not at all. Without this leg the selected counter would only ever
    // be asserted at zero and could not be shown wired at all.
    let names: im::Vector<String> = im::vector!["census_counter_positive_control".to_string()];
    let before = every_module_render_calls();
    let selected_before = non_empty_selected_render_calls();
    let _ = emit_resolved_for_target_counting(
        resolved,
        RenderTarget::Rust,
        Rc::new(RustModuleRenderSelection::RenderSelectedMirrors {
            basenames: Rc::new(names),
        }),
    );
    assert_eq!(
        every_module_render_calls() - before,
        0,
        "a selected render moved the whole-graph counter"
    );
    assert_eq!(
        non_empty_selected_render_calls() - selected_before,
        1,
        "a non-empty selected-basenames render did not move the selected counter by exactly one"
    );
}
