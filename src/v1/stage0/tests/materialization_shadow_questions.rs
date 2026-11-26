// Query outputs on complete fresh evaluations. The comparisons exercise what
// changed, independently of storage, cache policy or timing.
use std::collections::BTreeMap;
use std::rc::Rc;
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{
    compile_to_resolved, emit_resolved_for_target, ResolvedPipelineResult, SourceFile,
};
use v1_compiler::v1_compiler_infer_items::{ModuleTypecheckProgress, TypedModule};
use v1_compiler::v1_std_core::{is_error_diagnostic, CompilerDiagnostic};

fn evaluate(helper_type: &str, expression: &str, caller_type: &str) -> Rc<ResolvedPipelineResult> {
    let helper = Rc::new(SourceFile {
        path: "helper.dag".to_owned(),
        content: format!("module probe.helper\nfn answer() -> {helper_type} {{ {expression} }}\n"),
    });
    let caller = Rc::new(SourceFile {
        path: "caller.dag".to_owned(),
        content: format!("module probe.caller\nimport probe.helper {{ answer }}\nfn invoke() -> {caller_type} {{ answer() }}\n"),
    });
    compile_to_resolved(Rc::new(im::vector![helper, caller]))
}

fn checked(resolution: &ResolvedPipelineResult, name: &str) -> Rc<TypedModule> {
    let module = resolution
        .graph
        .as_ref()
        .expect("resolved graph")
        .modules
        .iter()
        .find(|m| m.interface.summary.module_path == name)
        .expect("named checked module");
    assert_eq!(module.progress, ModuleTypecheckProgress::ItemsChecked);
    assert_eq!(module.items.len(), 1, "one checked function in {name}");
    module.clone()
}

fn assert_clean(result: &ResolvedPipelineResult) {
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

fn emitted(result: Rc<ResolvedPipelineResult>) -> BTreeMap<String, String> {
    let built = emit_resolved_for_target(result, RenderTarget::Rust);
    let errors: Vec<_> = built
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
    built
        .files
        .iter()
        .map(|f| (f.path.clone(), f.content.clone()))
        .collect()
}

fn source_of<'a>(files: &'a BTreeMap<String, String>, declaration: &str) -> &'a str {
    let selected: Vec<_> = files
        .iter()
        .filter(|(path, body)| path.ends_with(".rs") && body.contains(declaration))
        .collect();
    assert_eq!(
        selected.len(),
        1,
        "expected a single Rust file declaring {declaration}; file roster: {:?}",
        files.keys().collect::<Vec<_>>()
    );
    selected[0].1
}

#[test]
fn imported_signature_checked_body_and_emitted_module_have_distinct_answers() {
    let baseline = evaluate("Int", "1", "Int");
    let repeat = evaluate("Int", "1", "Int");
    let body = evaluate("Int", "2", "Int");
    let incompatible = evaluate("String", "\"x\"", "Int");
    let repaired = evaluate("String", "\"x\"", "String");
    for clean in [&baseline, &repeat, &body, &repaired] {
        assert_clean(clean);
    }
    let helper = |result: &ResolvedPipelineResult| checked(result, "probe.helper");
    let caller = |result: &ResolvedPipelineResult| checked(result, "probe.caller");
    let original_helper = helper(&baseline);
    let original_caller = caller(&baseline);
    let reference_files = emitted(baseline.clone());
    let repeat_files = emitted(repeat.clone());
    let changed_files = emitted(body.clone());
    let repaired_files = emitted(repaired.clone());
    let exact_caller_error = incompatible.diagnostics.iter().any(|e| matches!(
        &*e.diagnostic,
        CompilerDiagnostic::TypeMismatch { expected, got, span }
        if expected == "Primitive(Int)" && got == "Primitive(String)" && span.file == "caller.dag" && is_error_diagnostic(e.diagnostic.clone())
    ));
    let predictions = [
        (
            "repeat helper interface agrees",
            original_helper.interface.summary == helper(&repeat).interface.summary,
        ),
        (
            "repeat helper checked items agree",
            original_helper.items == helper(&repeat).items,
        ),
        (
            "repeat caller checked items agree",
            original_caller.items == caller(&repeat).items,
        ),
        (
            "repeat helper output agrees",
            source_of(&reference_files, "pub fn answer(")
                == source_of(&repeat_files, "pub fn answer("),
        ),
        (
            "repeat caller output agrees",
            source_of(&reference_files, "pub fn invoke(")
                == source_of(&repeat_files, "pub fn invoke("),
        ),
        (
            "body edit preserves helper interface",
            original_helper.interface.summary == helper(&body).interface.summary,
        ),
        (
            "body edit changes helper checked items",
            original_helper.items != helper(&body).items,
        ),
        (
            "body edit preserves caller checked items",
            original_caller.items == caller(&body).items,
        ),
        (
            "body edit changes helper Rust bytes",
            source_of(&reference_files, "pub fn answer(")
                != source_of(&changed_files, "pub fn answer("),
        ),
        (
            "body edit preserves caller Rust bytes",
            source_of(&reference_files, "pub fn invoke(")
                == source_of(&changed_files, "pub fn invoke("),
        ),
        (
            "signature edit changes helper interface",
            original_helper.interface.summary != helper(&incompatible).interface.summary,
        ),
        (
            "unchanged caller reports Int/String type mismatch at caller.dag",
            exact_caller_error,
        ),
        (
            "repair changes caller Rust bytes",
            source_of(&reference_files, "pub fn invoke(")
                != source_of(&repaired_files, "pub fn invoke("),
        ),
    ];
    for (prediction, observed) in predictions {
        eprintln!("{observed}\t{prediction}");
    }
    assert!(
        predictions.iter().all(|(_, matched)| *matched),
        "one or more output predictions were falsified"
    );
}
