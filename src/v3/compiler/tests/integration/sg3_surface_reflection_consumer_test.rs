//! **Layer:** integration
//!
//! SG-3f-d consumption proof: a real `.dag` consumer reads the reflected
//! `SurfaceModule` / `SurfaceItem` schema, emits Rust against the
//! `parse_surface` realizations in `rust.dag`, rustc-links, and runs on
//! parser output. This is a sub-lane receipt, not closure of full SG-3f:
//! it closes the gap between "surface types are declared" and
//! "surface types are consumable substrate authority."

use std::path::PathBuf;
use std::sync::OnceLock;

use v3_compiler::compile_to_dag;
use v3_compiler::emit_rust::emit_rust_module;

use crate::common::{HarnessLinkMode, RustcHarness};

const SURFACE_CONSUMER_SOURCE: &str = r#"
module tests.sg3_surface_reflection

import v3.compiler.runtime_mirrors { SurfaceModule, SurfaceItem }
import std.list { List, length }

fn item_kind_score(item: SurfaceItem) -> Int =
  match item {
    Let(_) => 0
    Fn(_) => 0
    FnExternalBody(_) => 0
    Data(_) => 0
    Module(_) => 1
    Import(_) => 0
    TypeAtom(_) => 0
    TypeRecord(_) => 0
    TypeSum(_) => 0
    TypeAlias(_) => 0
  }

fn module_item_count(m: SurfaceModule) -> Int =
  length(m.items)
"#;

static HARNESS: OnceLock<RustcHarness> = OnceLock::new();

fn harness() -> &'static RustcHarness {
    HARNESS.get_or_init(|| RustcHarness::new("sg3_surface_reflection_consumer"))
}

fn consumer_fixture_path() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("integration")
        .join("sg3_surface_reflection_consumer_fixture.dag")
        .display()
        .to_string()
}

fn emitted_surface_consumer_module() -> String {
    let dag = compile_to_dag(SURFACE_CONSUMER_SOURCE, &consumer_fixture_path())
        .expect("compile reflected-surface consumer");
    assert!(
        dag.diagnostics().is_empty(),
        "reflected-surface consumer should compile cleanly, got {:?}",
        dag.diagnostics()
    );
    emit_rust_module(&dag).expect("emit Rust for reflected-surface consumer")
}

fn build_surface_consumer_harness(module_source: &str) -> PathBuf {
    let wrapped = format!(
        r#"
#[allow(warnings, clippy::all)]
mod emitted {{
    use v3_compiler::parse_surface;
    use v3_compiler::parse_surface::SurfaceItem;
    {module_source}
}}

fn main() {{
    let source = "module demo.core\nimport foo.bar {{ Baz }}\nlet x: Int = 1\nfn id<T>(x: T) -> T = x\nlet y = x";
    let tokens = v3_compiler::tokenize_for_test(source, "sg3_surface_reflection_consumer.v3")
        .expect("tokenize fixture");
    let parsed = v3_compiler::parse_for_test(&tokens, "sg3_surface_reflection_consumer.v3")
        .expect("parse fixture");
    let mirrored = v3_compiler::parse_surface::SurfaceModule::from(&parsed);
    let item_count = emitted::module_item_count(&mirrored);
    assert_eq!(item_count, 5, "emitted surface consumer should traverse SurfaceModule.items through emitted code");
}}
"#
    );
    harness().compile(
        &wrapped,
        "sg3_surface_consumer",
        HarnessLinkMode::WithV3Compiler,
    )
}

#[test]
fn reflected_surface_consumer_dag_compiles_cleanly() {
    let dag = compile_to_dag(SURFACE_CONSUMER_SOURCE, &consumer_fixture_path())
        .expect("compile reflected-surface consumer");
    assert!(
        dag.diagnostics().is_empty(),
        "reflected-surface consumer should compile without diagnostics, got {:?}",
        dag.diagnostics()
    );
}

#[test]
fn reflected_surface_consumer_emits_links_and_runs() {
    let module = emitted_surface_consumer_module();
    let bin = build_surface_consumer_harness(&module);
    let run = std::process::Command::new(&bin)
        .output()
        .expect("run reflected-surface consumer harness");
    assert!(
        run.status.success(),
        "surface consumer harness failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
}
