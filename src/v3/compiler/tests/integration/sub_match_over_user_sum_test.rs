//! Day-1 T-Sub gate: source-local user-defined sums must survive the
//! parse -> lower -> infer -> Rust emit -> rustc pipeline as first-class surface.

use std::path::PathBuf;
use std::sync::OnceLock;

use v3_compiler::compile_to_dag;
use v3_compiler::emit_rust::emit_rust_module;

use crate::common::{HarnessLinkMode, RustcHarness};

const SUB_MATCH_OVER_USER_SUM_SOURCE: &str = r#"
module tests.sub_match_over_user_sum

type Choice = Number(Int) | Missing

fn score(choice: Choice) -> Int =
  match choice {
    Number(value) => value
    Missing => 0
  }
"#;

static HARNESS: OnceLock<RustcHarness> = OnceLock::new();

fn harness() -> &'static RustcHarness {
    HARNESS.get_or_init(|| RustcHarness::new("sub_match_over_user_sum"))
}

fn fixture_path() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("integration")
        .join("sub_match_over_user_sum_fixture.dag")
        .display()
        .to_string()
}

fn emitted_module() -> String {
    let dag = compile_to_dag(SUB_MATCH_OVER_USER_SUM_SOURCE, &fixture_path())
        .expect("compile sub_match_over_user_sum fixture");
    assert!(
        dag.diagnostics().is_empty(),
        "sub_match_over_user_sum fixture should compile cleanly, got {:?}",
        dag.diagnostics()
    );
    emit_rust_module(&dag).expect("emit Rust for sub_match_over_user_sum fixture")
}

fn build_harness(module_source: &str) -> PathBuf {
    let wrapped = format!(
        r#"
#[allow(warnings, clippy::all)]
mod emitted {{
    {module_source}
}}

fn main() {{
    let present = emitted::Choice::Number {{ _0: 7 }};
    let missing = emitted::Choice::Missing;
    assert_eq!(emitted::score(&present), 7);
    assert_eq!(emitted::score(&missing), 0);
}}
"#
    );
    harness().compile(
        &wrapped,
        "sub_match_over_user_sum",
        HarnessLinkMode::Standalone,
    )
}

#[test]
fn sub_match_over_user_sum_compiles_and_emits() {
    let dag = compile_to_dag(SUB_MATCH_OVER_USER_SUM_SOURCE, &fixture_path())
        .expect("compile sub_match_over_user_sum fixture");
    assert!(
        dag.diagnostics().is_empty(),
        "sub_match_over_user_sum fixture should compile without diagnostics, got {:?}",
        dag.diagnostics()
    );

    let module = emit_rust_module(&dag).expect("emit Rust for sub_match_over_user_sum fixture");
    assert!(
        module.contains("pub enum Choice"),
        "emitted module should define the user sum as a Rust enum:\n{module}"
    );
    assert!(
        module.contains("match"),
        "emitted module should lower the source match directly:\n{module}"
    );
}

#[test]
fn sub_match_over_user_sum_links_and_runs() {
    let module = emitted_module();
    let bin = build_harness(&module);
    let run = std::process::Command::new(&bin)
        .output()
        .expect("run sub_match_over_user_sum harness");
    assert!(
        run.status.success(),
        "sub_match_over_user_sum harness failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
}
