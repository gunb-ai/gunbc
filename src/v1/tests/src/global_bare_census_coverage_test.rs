//! Root-cause probe for resolve failures on the namespace import strip.
//!
//! `global_bare` binds a bare cross-module reference iff the name is globally
//! unique. These probes isolate which declaration shapes the census covers.

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};
use v1_compiler::v1_std_core::is_interpreter_blocking_diagnostic;

fn src(path: &str, content: &str) -> Rc<SourceFile> {
    Rc::new(SourceFile {
        path: path.to_string(),
        content: content.to_string(),
    })
}

fn hard_diags(definer: &str, user: &str) -> Vec<String> {
    let sources = vec![src("dag/probe_def.dag", definer), src("dag/probe_use.dag", user)];
    let result = compile_sources(
        Rc::new(sources.into()),
        v1_compiler::v1_compiler_artifact::RenderTarget::Rust,
    );
    result
        .diagnostics
        .iter()
        .filter(|d| is_interpreter_blocking_diagnostic(d.diagnostic.clone()))
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

const DEFINER: &str = r#"module probe.def

type ProbeCurrency =
    ProbeEur
  | ProbeUsd

fn probe_minor_unit(c: ProbeCurrency) -> Int {
  match c {
    ProbeEur => 2
    ProbeUsd => 2
  }
}
"#;

#[test]
fn probe_census_indexes_variant_and_fn() {
    use v1_compiler::v1_compiler_compile::front_end_sources;
    use v1_compiler::v1_compiler_infer::build_global_bare_census;
    use v1_compiler::v1_compiler_infer_env::GlobalBareLookupState;

    let sources = Rc::new(
        vec![src("dag/probe_def.dag", DEFINER)]
            .into_iter()
            .collect::<im_rc::Vector<_>>(),
    );
    let frontend = front_end_sources(sources);
    let graph = frontend.graph.as_ref().expect("graph");
    let source_indices = frontend
        .newline_indices
        .iter()
        .cloned()
        .fold(
            im_rc::HashMap::new(),
            |acc, si| acc.update(si.file.clone(), si),
        );
    let source_indices_rc = Rc::new(source_indices);
    let census = build_global_bare_census(graph.modules.clone(), source_indices_rc.clone());
    assert!(
        matches!(
            census.get("ProbeEur").map(|s| &**s),
            Some(GlobalBareLookupState::GlobalBareUniqueBinding { .. })
        ),
        "census must index module-unique Disj variant ProbeEur: keys={:?}",
        census.keys().collect::<Vec<_>>()
    );
    assert!(
        matches!(
            census.get("probe_minor_unit").map(|s| &**s),
            Some(GlobalBareLookupState::GlobalBareUniqueBinding { .. })
        ),
        "census must index fn-with-body probe_minor_unit"
    );
}

#[test]
fn probe_bare_type_reference_resolves() {
    let user = r#"module probe.use

fn takes_currency(c: ProbeCurrency) -> Int {
  1
}
"#;
    let d = hard_diags(DEFINER, user);
    assert!(d.is_empty(), "bare TYPE ref should resolve via global_bare: {d:?}");
}

#[test]
fn probe_bare_function_reference_resolves() {
    let user = r#"module probe.use

fn call_it(c: ProbeCurrency) -> Int {
  probe_minor_unit(c: c)
}
"#;
    let d = hard_diags(DEFINER, user);
    assert!(d.is_empty(), "bare FN ref should resolve via global_bare: {d:?}");
}

#[test]
fn probe_bare_variant_reference_resolves() {
    let user = r#"module probe.use

fn pick() -> ProbeCurrency {
  ProbeEur
}
"#;
    let d = hard_diags(DEFINER, user);
    assert!(d.is_empty(), "bare VARIANT ref should resolve via global_bare: {d:?}");
}

#[test]
fn probe_bare_data_with_value_reference_resolves() {
    let definer = r#"module probe.def

data probe_exit_code: Int = 1
"#;
    let user = r#"module probe.use

fn code() -> Int {
  probe_exit_code
}
"#;
    let d = hard_diags(definer, user);
    assert!(
        d.is_empty(),
        "bare DATA-with-value ref should resolve via global_bare: {d:?}"
    );
}

#[test]
fn probe_bare_data_ambiguity_stays_red() {
    let definer_a = r#"module probe.def_a

data probe_exit_code: Int = 1
"#;
    let definer_b = r#"module probe.def_b

data probe_exit_code: Int = 2
"#;
    let user = r#"module probe.use

fn code() -> Int {
  probe_exit_code
}
"#;
    let sources = vec![
        src("dag/probe_def_a.dag", definer_a),
        src("dag/probe_def_b.dag", definer_b),
        src("dag/probe_use.dag", user),
    ];
    let result = compile_sources(
        Rc::new(sources.into()),
        v1_compiler::v1_compiler_artifact::RenderTarget::Rust,
    );
    let d: Vec<String> = result
        .diagnostics
        .iter()
        .filter(|diag| is_interpreter_blocking_diagnostic(diag.diagnostic.clone()))
        .map(|diag| v1_compiler::v1_std_core::diagnostic_to_message(diag.diagnostic.clone()))
        .collect();
    assert!(
        d.iter().any(|m| m.contains("undefined variable 'probe_exit_code'")),
        "ambiguous global_bare data must stay unresolved: {d:?}"
    );
}

/// GAP 4 — a ZERO-PARAMETER fn with a body. Every census branch admitting a
/// params-bearing item requires `params |> count > 0`, so a nullary fn matches none
/// of them. The earlier fn probe took a parameter, which is why it went green while
/// the corpus kept reding — the probe and the corpus disagreed on shape, not scale.
///
/// Live instances (both declared exactly once, both ABSENT from the corpus census):
///   dag/std/numerical_contract.dag:15  fn integer_exact_contract() -> NumericalContract
///   dag/gunbc/gunbhub_serve.dag:139    fn gunbhub_hostile_page()   -> MarkupNode
#[test]
fn probe_bare_nullary_function_reference_resolves() {
    let sources = vec![
        src(
            "dag/probe_nullary_def.dag",
            r#"module probe.nullarydef

fn probe_nullary_thing() -> Int {
  7
}
"#,
        ),
        src(
            "dag/probe_nullary_use.dag",
            r#"module probe.nullaryuse

fn call_nullary() -> Int {
  probe_nullary_thing()
}
"#,
        ),
    ];
    let result = compile_sources(
        Rc::new(sources.into()),
        v1_compiler::v1_compiler_artifact::RenderTarget::Rust,
    );
    let d: Vec<String> = result
        .diagnostics
        .iter()
        .filter(|x| is_interpreter_blocking_diagnostic(x.diagnostic.clone()))
        .map(|x| v1_compiler::v1_std_core::diagnostic_to_message(x.diagnostic.clone()))
        .collect();
    assert!(
        d.is_empty(),
        "bare ref to globally-unique NULLARY fn must resolve via global_bare \
         (census branches admitting fn-with-body all require params > 0); got {d:?}"
    );
}
