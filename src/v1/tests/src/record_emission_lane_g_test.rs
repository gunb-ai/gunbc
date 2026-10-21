// Lane G (v1-interface-cull-audit@63a464edf01 P1.5 steps 1-2) emission receipts.
//
// The mgr's landing-evidence ask (2026-10-21): one single-field and one multi-field anonymous
// record literal run through the CHANGED emission route -- `emit_typed_record_lit`'s two
// candidate-set arms in `src/v1/05_emit_rust.dag`, executed via the stage0 mirror these tests
// link -- showing emitted output and refusals unchanged versus the merge base. Each case
// compiles a provider module (record types + consuming function) plus a consumer module that
// imports ONLY the function and passes an anonymous literal in argument position, spelling no
// record type name -- the mechanism pinned by test.claim.anonymous_record_head_use_line_witness
// (the emitter decides the head). The full emitted text is printed under --nocapture so the
// receipt is a text diff of these dumps against the same tests executed at the merge base,
// not a transcription.

use std::rc::Rc;

use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, SourceFile};

fn compile_pair(
    provider: &str,
    consumer: &str,
) -> Rc<v1_compiler::v1_compiler_compile::PipelineResult> {
    compile_sources(
        Rc::new(im::vector![
            Rc::new(SourceFile {
                path: "test/fixture/laneg_provider.dag".to_string(),
                content: provider.to_string(),
            }),
            Rc::new(SourceFile {
                path: "test/fixture/laneg_consumer.dag".to_string(),
                content: consumer.to_string(),
            }),
        ]),
        RenderTarget::Rust,
    )
}

fn emitted_text(result: &v1_compiler::v1_compiler_compile::PipelineResult) -> String {
    result
        .files
        .iter()
        .map(|f| format!("--- {}\n{}", f.path, f.content))
        .collect::<Vec<_>>()
        .join("\n")
}

fn dump(tag: &str, text: &str) {
    println!("===LANEG-EMIT {tag}===\n{text}\n===LANEG-END===");
}

fn dump_with_diagnostics(
    tag: &str,
    result: &v1_compiler::v1_compiler_compile::PipelineResult,
) -> String {
    let text = emitted_text(result);
    let diags = result
        .diagnostics
        .iter()
        .map(|d| {
            format!(
                "[diagnostic] {}",
                v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone())
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let combined = format!("{text}\n{diags}");
    dump(tag, &combined);
    combined
}

const REFUSAL: &str =
    "compile_error!(\"ambiguous anonymous record literal shape; add a nominal type\")";

fn provider(prefix: &str, decls: &str, fns: &str) -> String {
    format!("module test.fixture.{prefix}_provider\n\n{decls}\n{fns}\n")
}

fn consumer(prefix: &str, imports: &[&str], body: &str) -> String {
    let import_line = format!(
        "import test.fixture.{prefix}_provider {{\n{}\n}}\n\n",
        imports
            .iter()
            .map(|i| format!("  {i},"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    format!("module test.fixture.{prefix}_consumer\n\n{import_line}{body}\n")
}

/// Single-field anonymous literal, zero field-name candidates in scope: the changed arm's
/// zero-candidate pass-through renders the bare value (`takes_solo(7)` at the merge base).
#[test]
fn lane_g_single_field_literal_passes_through_bare_value() {
    let provider = provider(
        "laneg_single_res",
        "type LanegSolo { v: Int }",
        "fn takes_solo(s: LanegSolo) -> Int {\n  1\n}",
    );
    let c = consumer(
        "laneg_single_res",
        &["takes_solo"],
        "fn probe() -> Int {\n  takes_solo(s: { v: 7 })\n}",
    );
    let result = compile_pair(&provider, &c);
    let text = dump_with_diagnostics("single_resolution", &result);
    assert!(
        !text.contains(REFUSAL),
        "single-field literal must not hit the refusal: {text}"
    );
    assert!(
        text.contains("takes_solo(7)"),
        "single-field pass-through must render the bare value, matching the merge base: {text}"
    );
}

/// Single-field, two same-shape candidates: the changed arm's refusal, verbatim.
#[test]
fn lane_g_single_field_ambiguous_literal_refuses() {
    let provider = provider(
        "laneg_single_amb",
        "type LanegDupS1 { w: Int }\n\ntype LanegDupS2 { w: Int }",
        "fn takes_ds1(a: LanegDupS1) -> Int {\n  1\n}",
    );
    let c = consumer(
        "laneg_single_amb",
        &["takes_ds1"],
        "fn probe() -> Int {\n  takes_ds1(a: { w: 7 })\n}",
    );
    let result = compile_pair(&provider, &c);
    let text = dump_with_diagnostics("single_ambiguous", &result);
    assert!(
        text.contains(REFUSAL),
        "ambiguous single-field shape must emit the refusal from the changed arm: {text}"
    );
}

/// Multi-field, unique shape: the changed arm renders a named struct literal.
#[test]
fn lane_g_multi_field_literal_emits_named_struct() {
    let provider = provider(
        "laneg_multi_res",
        "type LanegPoint { x: Int, y: Int }",
        "fn takes_point(p: LanegPoint) -> Int {\n  1\n}",
    );
    let c = consumer(
        "laneg_multi_res",
        &["takes_point"],
        "fn probe() -> Int {\n  takes_point(p: { x: 1, y: 2 })\n}",
    );
    let result = compile_pair(&provider, &c);
    let text = dump_with_diagnostics("multi_resolution", &result);
    assert!(
        !text.contains(REFUSAL),
        "unique multi-field shape must not hit the changed arm's refusal: {text}"
    );
    assert!(
        text.contains("LanegPoint {"),
        "multi-field literal must render as the resolved struct literal: {text}"
    );
}

/// Multi-field, two same-shape candidates: the refusal surfaces as the located
/// AmbiguousAnonymousRecordLiteral diagnostic naming both structs (the base emits no
/// compile_error! line for this route; both refusals are asserted from the base run).
#[test]
fn lane_g_multi_field_ambiguous_literal_refuses() {
    let provider = provider(
        "laneg_multi_amb",
        "type LanegDupA { left: Bool, right: Int }\n\ntype LanegDupB { left: Bool, right: Int }",
        "fn takes_da(a: LanegDupA) -> Int {\n  1\n}",
    );
    let c = consumer(
        "laneg_multi_amb",
        &["takes_da"],
        "fn probe() -> Int {\n  takes_da(a: { left: 1, right: 2 })\n}",
    );
    let result = compile_pair(&provider, &c);
    let text = dump_with_diagnostics("multi_ambiguous", &result);
    assert!(
        text.contains("ambiguous anonymous record literal shape matches 2 structs: LanegDupA, LanegDupB"),
        "ambiguous multi-field shape must refuse with the located diagnostic naming both candidates, matching the merge base: {text}"
    );
    assert!(
        !text.contains("LanegDupA {"),
        "the ambiguous multi-field shape must not silently pick a candidate: {text}"
    );
}
