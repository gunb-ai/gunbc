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
fn probe_bare_function_reference_resolves() {
    let user = r#"module probe.use

fn call_it(c: ProbeCurrency) -> Int {
  probe_minor_unit(c: c)
}
"#;
    let d = hard_diags(DEFINER, user);
    assert!(d.is_empty(), "bare FN ref should resolve via global_bare: {d:?}");
}
