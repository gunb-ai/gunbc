//! P5-SPIKE witness: does the manual compiler-stage IR fixture seam
//! (`front_end_sources` → `normalize_graph` → `reconcile` with `frontend.intern_table`)
//! agree with the production `compile_to_resolved_with_options` path on complexity output?
//!
//! The seam under test is `compile_dag_with_complexity` in pipeline.rs — it stitches stages
//! manually instead of calling `compile_sources_with_options(analyze_complexity: true)`.

use std::collections::HashMap;
use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{
    build_recursion_context, compile_sources_with_options, extract_func_entries,
    front_end_sources, CompilePipelineOptions,
};
use v1_compiler::v1_compiler_complexity::build_complexity_report;
use v1_compiler::v1_compiler_infer::reconcile;
use v1_compiler::v1_compiler_normalize::normalize_graph;
use v1_compiler::v1_std_core::NewlineIndex;
use v1_compiler::RenderTarget;

use crate::helpers::resolve_imports_transitively;

fn fixture_seam_complexity(source: &str) -> Rc<v1_compiler::v1_compiler_complexity::ComplexityReport> {
    let sources = resolve_imports_transitively("test.dag", source);
    let frontend = front_end_sources(Rc::new(sources));
    let graph = frontend.graph.clone().expect("frontend must produce a graph");
    let norm = normalize_graph(graph, Rc::new(HashMap::new()));
    let typed = reconcile(
        norm.graph.clone(),
        Rc::new(HashMap::new()),
        frontend.intern_table.clone(),
    );
    let func_entries = extract_func_entries(typed.clone());
    let recursion_ctx = build_recursion_context(typed);
    build_complexity_report(func_entries, recursion_ctx, Rc::new(HashMap::new()))
}

fn production_complexity(source: &str) -> Rc<v1_compiler::v1_compiler_complexity::ComplexityReport> {
    let sources = resolve_imports_transitively("test.dag", source);
    let result = compile_sources_with_options(
        Rc::new(sources),
        RenderTarget::Rust,
        CompilePipelineOptions {
            analyze_complexity: true,
        },
    );
    result.complexity.clone()
}

/// Production path with real source_indices but same intern_table handoff as fixture seam.
fn fixture_seam_with_real_source_indices(
    source: &str,
) -> Rc<v1_compiler::v1_compiler_complexity::ComplexityReport> {
    let sources = resolve_imports_transitively("test.dag", source);
    let frontend = front_end_sources(Rc::new(sources));
    let graph = frontend.graph.clone().expect("frontend must produce a graph");
    let source_indices = frontend.newline_indices.clone().iter().cloned().fold(
        Rc::new(HashMap::<String, Rc<NewlineIndex>>::new()),
        |acc, si| {
            let mut m = (*acc).clone();
            m.insert(si.file.clone(), si.clone());
            Rc::new(m)
        },
    );
    let norm = normalize_graph(graph, source_indices.clone());
    let typed = reconcile(
        norm.graph.clone(),
        source_indices.clone(),
        frontend.intern_table.clone(),
    );
    let func_entries = extract_func_entries(typed.clone());
    let recursion_ctx = build_recursion_context(typed);
    build_complexity_report(func_entries, recursion_ctx, source_indices)
}

fn report_key(report: &v1_compiler::v1_compiler_complexity::ComplexityReport) -> String {
    let mut classes: Vec<(String, String)> = report
        .function_classes
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    classes.sort_by(|a, b| a.0.cmp(&b.0));
    let mut violations: Vec<String> = report
        .violations
        .iter()
        .map(|v| format!("{}:{:?}", v.func_name, v.reason))
        .collect();
    violations.sort();
    format!("classes={classes:?};violations={violations:?}")
}

#[test]
fn ir_fixture_seam_matches_production_on_small_programs() {
    let sources = [
        r#"module linear
fn sum(xs: List<Int>) -> Int {
  xs |> fold(init: 0, f: (acc, x) => acc + x)
}
"#,
        r#"module nested
fn outer(n: Int) -> Int {
  if n <= 0 { 0 } else { inner(n: n - 1) + 1 }
}
fn inner(n: Int) -> Int { n }
"#,
    ];
    for source in sources {
        let fixture = fixture_seam_complexity(source);
        let production = production_complexity(source);
        assert_eq!(
            report_key(&fixture),
            report_key(&production),
            "fixture seam diverged from production pipeline"
        );
    }
}

#[test]
#[ignore = "spike: full 02_parse.dag is heavy; run explicitly"]
fn ir_fixture_seam_matches_production_on_parse_stage() {
    let ws = crate::helpers::workspace_root();
    let content = std::fs::read_to_string(ws.join("src/v1/02_parse.dag")).unwrap();
    let sources = crate::helpers::resolve_imports_transitively("src/v1/02_parse.dag", &content);
    let source = sources
        .first()
        .map(|s| s.content.as_str())
        .expect("parse stage sources");
    let fixture = fixture_seam_complexity(source);
    let production = production_complexity(source);
    assert_eq!(report_key(&fixture), report_key(&production));
}

#[test]
fn ir_fixture_seam_empty_vs_real_source_indices_on_typed_graph() {
    let source = r#"module si_probe
type Box { v: Int }
fn f(x: Box) -> Int { x.v }
"#;
    let sources = resolve_imports_transitively("test.dag", source);
    let frontend = front_end_sources(Rc::new(sources));
    let graph = frontend.graph.clone().expect("graph");
    let empty = Rc::new(HashMap::<String, Rc<NewlineIndex>>::new());
    let real = frontend.newline_indices.clone().iter().cloned().fold(
        Rc::new(HashMap::<String, Rc<NewlineIndex>>::new()),
        |acc, si| {
            let mut m = (*acc).clone();
            m.insert(si.file.clone(), si.clone());
            Rc::new(m)
        },
    );
    let norm_empty = normalize_graph(graph.clone(), empty.clone());
    let norm_real = normalize_graph(graph, real.clone());
    let typed_empty = reconcile(norm_empty.graph.clone(), empty, frontend.intern_table.clone());
    let typed_real = reconcile(norm_real.graph.clone(), real, frontend.intern_table.clone());
    assert_eq!(
        typed_empty.modules.len(),
        typed_real.modules.len(),
        "module count should not depend on source_indices seam"
    );
    let fixture_empty = fixture_seam_complexity(source);
    let fixture_real = fixture_seam_with_real_source_indices(source);
    assert_eq!(
        report_key(&fixture_empty),
        report_key(&fixture_real),
        "complexity should not depend on empty vs real source_indices for this probe"
    );
}
