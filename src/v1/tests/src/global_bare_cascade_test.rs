//! Is the ~9.4k residue CASCADE rather than census-coverage?
//!
//! Three census gaps (variant, fn-with-body, data-with-value) moved the #6640 batch-1
//! count 11023 -> 9365 (~15%). Shapes that fail in the corpus (record type, fn) resolve
//! fine in a 2-module probe. So the residue is not coverage.
//!
//! Hypothesis: `build_global_bare_census` folds over `graph.modules`. If a module does
//! not survive to that fold, its declarations are never censused, so EVERY bare
//! reference to its names reds — one root failure manufacturing many diagnostics. If
//! true, the count is a noise measure, not a worklist, and the real quantity is the
//! number of ROOT failures.

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};
use v1_compiler::v1_std_core::is_interpreter_blocking_diagnostic;

fn src(path: &str, content: &str) -> Rc<SourceFile> {
    Rc::new(SourceFile { path: path.to_string(), content: content.to_string() })
}

fn hard(sources: Vec<Rc<SourceFile>>) -> Vec<String> {
    let result = compile_sources(
        Rc::new(sources.into()),
        v1_compiler::v1_compiler_artifact::RenderTarget::Rust,
    );
    result
        .diagnostics
        .iter()
        .filter(|x| is_interpreter_blocking_diagnostic(x.diagnostic.clone()))
        .map(|x| v1_compiler::v1_std_core::diagnostic_to_message(x.diagnostic.clone()))
        .collect()
}

const CLEAN_DEF: &str = r#"module probe.def

type ProbeThing {
  field_a: Int
}
"#;

const USE_IT: &str = r#"module probe.use

fn take(t: ProbeThing) -> Int {
  1
}
"#;

/// BASELINE: definer + user, nothing broken. Must be clean.
#[test]
fn baseline_clean_two_modules() {
    let d = hard(vec![src("dag/probe_def.dag", CLEAN_DEF), src("dag/probe_use.dag", USE_IT)]);
    assert!(d.is_empty(), "baseline must be clean; got {d:?}");
}

/// CASCADE PROBE A — an UNRELATED broken module elsewhere in the program.
/// `ProbeThing` is still declared once and cleanly. If `probe.use` now fails to see it,
/// a failure anywhere poisons the census for everyone.
#[test]
fn unrelated_broken_module_does_not_break_others() {
    let broken = r#"module probe.broken

fn oops() -> Int {
  totally_undeclared_name
}
"#;
    let d = hard(vec![
        src("dag/probe_def.dag", CLEAN_DEF),
        src("dag/probe_broken.dag", broken),
        src("dag/probe_use.dag", USE_IT),
    ]);
    let poisoned: Vec<&String> = d.iter().filter(|m| m.contains("ProbeThing")).collect();
    assert!(
        poisoned.is_empty(),
        "an unrelated broken module must NOT stop probe.use from resolving ProbeThing \
         (cascade). ProbeThing diagnostics: {poisoned:?} | all: {d:?}"
    );
}

/// CASCADE PROBE B — the DEFINER itself has an unrelated error, but still declares
/// `ProbeThing` correctly. Does its own valid declaration still reach the census?
#[test]
fn broken_definer_still_censuses_its_valid_declarations() {
    let broken_def = r#"module probe.def

type ProbeThing {
  field_a: Int
}

fn oops() -> Int {
  totally_undeclared_name
}
"#;
    let d = hard(vec![src("dag/probe_def.dag", broken_def), src("dag/probe_use.dag", USE_IT)]);
    let poisoned: Vec<&String> = d.iter().filter(|m| m.contains("ProbeThing")).collect();
    assert!(
        poisoned.is_empty(),
        "a definer with an UNRELATED error must still census its valid ProbeThing \
         declaration (cascade). ProbeThing diagnostics: {poisoned:?} | all: {d:?}"
    );
}
