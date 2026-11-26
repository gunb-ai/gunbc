// Query outputs on complete fresh evaluations. The comparisons exercise what
// changed, independently of storage, cache policy or timing.
use std::collections::BTreeMap;
use std::rc::Rc;
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{
    compile_to_resolved, emit_resolved_for_target, front_end_sources, ResolvedPipelineResult,
    SourceFile,
};
use v1_compiler::v1_compiler_infer_env::SymbolIndex;
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

fn census(literal: &str, banner_in_helper: bool, banner_in_other: bool) -> Rc<SymbolIndex> {
    use v1_compiler::v1_compiler_infer::build_symbol_index_census;
    use v1_compiler::v1_compiler_normalize::normalize_graph;
    let banner = "type Banner { rank: Int }\n";
    let mut modules = im::vector![Rc::new(SourceFile {
        path: "helper.dag".into(),
        content: format!("module census.helper\nfn answer() -> Int {{ {literal} }}\ntype Existing {{ rank: Int }}\n{}",
                  if banner_in_helper {banner} else {""}),
    })];
    if banner_in_other {
        modules.push_back(Rc::new(SourceFile {
            path: "other.dag".into(),
            content: format!("module census.other\n{banner}"),
        }));
    }
    let front = front_end_sources(Rc::new(modules));
    assert!(
        front
            .diagnostics
            .iter()
            .all(|d| !is_error_diagnostic(d.diagnostic.clone())),
        "{:?}",
        front.diagnostics
    );
    let indices: Rc<im::HashMap<_, _>> = Rc::new(
        front
            .newline_indices
            .iter()
            .map(|s| (s.file.clone(), s.clone()))
            .collect(),
    );
    let normalized = normalize_graph(
        front.graph.clone().expect("resolved frontend module graph"),
        indices.clone(),
    );
    assert!(
        normalized.diagnostics.is_empty(),
        "{:?}",
        normalized.diagnostics
    );
    build_symbol_index_census(normalized.graph.modules.clone(), indices)
}

#[test]
fn symbol_census_distinguishes_body_edits_source_locations_and_competing_declarations() {
    use std::collections::BTreeSet;
    use v1_compiler::v1_compiler_infer_env::GlobalBareLookupState;
    let baseline = census("1", false, false);
    let repeat = census("1", false, false);
    let body = census("2", false, false);
    let wider = census("12345", false, false);
    let appended = census("1", true, false);
    let competing = census("1", true, true);
    let existing = "census.helper.Existing";
    // The widened literal shifts the span of the following declaration. Checking the
    // actual location prevents an unrelated change elsewhere in the index masking this case.
    let original = &baseline.entries[existing].span;
    let shifted = &wider.entries[existing].span;
    assert_eq!(original.file, shifted.file);
    assert_eq!(
        (shifted.start, shifted.end),
        (original.start + 4, original.end + 4)
    );
    let first = "census.helper.Banner";
    let second = "census.other.Banner";
    let unique_owner = match appended.global_bare.get("Banner").map(Rc::as_ref) {
        Some(GlobalBareLookupState::GlobalBareUniqueBinding { module_path, .. }) => {
            Some(module_path.as_str())
        }
        _ => None,
    };
    let competing_owners: BTreeSet<_> = match competing.global_bare.get("Banner").map(Rc::as_ref) {
        Some(GlobalBareLookupState::GlobalBareAmbiguousBinding { candidates }) => candidates
            .iter()
            .map(|owner| owner.module_path.as_str())
            .collect(),
        _ => BTreeSet::new(),
    };
    let checks = [
        (
            "existing qualified type observed",
            baseline.entries.contains_key(existing),
        ),
        (
            "repeat complete 8-field SymbolIndex equals baseline",
            *baseline == *repeat,
        ),
        (
            "same-width edit complete SymbolIndex equals baseline",
            *baseline == *body,
        ),
        (
            "wider body keeps qualified name roster",
            baseline
                .entries
                .keys()
                .collect::<std::collections::BTreeSet<_>>()
                == wider.entries.keys().collect(),
        ),
        (
            "wider body changes complete source-located SymbolIndex",
            *wider != *baseline,
        ),
        (
            "append actual type changes complete SymbolIndex",
            *appended != *baseline,
        ),
        (
            "appended qualified type previously absent",
            !baseline.entries.contains_key(first) && appended.entries.contains_key(first),
        ),
        (
            "appended type preserves existing qualified node",
            baseline.entries.get(existing) == appended.entries.get(existing),
        ),
        (
            "appended type introduces unique bare owner",
            !baseline.global_bare.contains_key("Banner") && unique_owner == Some("census.helper"),
        ),
        (
            "both competing types retain qualified declarations",
            competing.entries.contains_key(first) && competing.entries.contains_key(second),
        ),
        (
            "bare ownership becomes ambiguous",
            competing_owners == BTreeSet::from(["census.helper", "census.other"]),
        ),
    ];
    for (name, matched) in checks {
        eprintln!("{matched}\t{name}");
    }
    assert!(
        checks.iter().all(|(_, matched)| *matched),
        "a predeclared census output prediction failed"
    );
}
