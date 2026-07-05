#![allow(clippy::disallowed_macros)]

//! Host-physics floor witness for essential v1 compiler diagnostic behavior
//! (curated from `src/v1/tests/src/diagnostics.rs`; not 1:1 test-count parity).
//! Exercises compile-time diagnostic variants, grounded messages, and span
//! mapping via `compile_multi` until that harness is witness-layer importable.

use im_rc::HashMap;
use std::env;
use std::process::ExitCode;
use std::rc::Rc;

use v1_compiler::cli_run::workspace_root;
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, PipelineResult, SourceFile};
use v1_compiler::v1_std_core::{
    build_newline_index, byte_to_line_col, diagnostic_to_message, diagnostic_to_span,
    CompilerDiagnostic, ErrorNode,
};

type ModuleIndex = HashMap<String, std::path::PathBuf>;
type WitnessCase = (&'static str, fn(&ModuleIndex));

const SUITE_IMPORT_RESOLUTION: &str = "import_resolution";
const SUITE_REEXPORT_SURFACE: &str = "reexport_surface";
const SUITE_TYPE_AND_ARITY: &str = "type_and_arity";
const SUITE_EMPTY_LIST_CONTEXT: &str = "empty_list_context";
const SUITE_CONSTRUCTOR_OWNER: &str = "constructor_owner";

fn fail(msg: impl std::fmt::Display) -> ExitCode {
    eprintln!("diagnostics_witness: {msg}");
    ExitCode::from(1)
}

fn source_roots() -> [std::path::PathBuf; 2] {
    let ws = workspace_root();
    [ws.join("src/v1"), ws.join("dag")]
}

fn extract_module_declaration(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        return trimmed
            .strip_prefix("module ")
            .and_then(|rest| rest.split_whitespace().next())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
    }
    None
}

fn scan_dag_files(dir: &std::path::Path, index: &mut ModuleIndex) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dag_files(&path, index);
        } else if path.extension().map(|e| e == "dag").unwrap_or(false) {
            if let Some(module_path) = extract_module_declaration(&path) {
                index.insert(module_path, path);
            }
        }
    }
}

fn build_module_index() -> ModuleIndex {
    let mut index = HashMap::new();
    for root in source_roots() {
        if root.exists() {
            scan_dag_files(&root, &mut index);
        }
    }
    index
}

fn extract_imports(source: &str) -> Vec<String> {
    let tokens =
        v1_compiler::v1_compiler_tokenize::tokenize(source.to_string(), "test.dag".to_string());
    let source_index =
        v1_compiler::v1_std_core::build_newline_index("test.dag".to_string(), source.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert("test.dag".to_string(), source_index);
    let result = v1_compiler::v1_compiler_parse::parse(tokens, Rc::new(source_indices));
    match &result.module {
        Some(module) => v1_compiler::v1_std_core::module_imports(module.clone())
            .iter()
            .map(|imp| imp.name.clone())
            .collect(),
        None => vec![],
    }
}

fn resolve_imports_transitively(
    entry_path: &str,
    entry_content: &str,
    module_index: &ModuleIndex,
) -> Vec<Rc<SourceFile>> {
    let ws = workspace_root();
    let mut seen: HashMap<String, Rc<SourceFile>> = HashMap::new();
    let mut queue = vec![(entry_path.to_string(), entry_content.to_string())];

    while let Some((_path, content)) = queue.pop() {
        for module_path in extract_imports(&content) {
            if seen.contains_key(&module_path) {
                continue;
            }
            if let Some(file_path) = module_index.get(&module_path) {
                if let Ok(file_content) = std::fs::read_to_string(file_path) {
                    let rel_path = file_path
                        .strip_prefix(&ws)
                        .unwrap_or(file_path)
                        .to_string_lossy()
                        .to_string();
                    seen.insert(
                        module_path.clone(),
                        Rc::new(SourceFile {
                            path: rel_path.clone(),
                            content: file_content.clone(),
                        }),
                    );
                    queue.push((rel_path, file_content));
                }
            }
        }
    }

    let mut sources: Vec<Rc<SourceFile>> = seen.into_iter().map(|(_, v)| v).collect();
    sources.push(Rc::new(SourceFile {
        path: entry_path.to_string(),
        content: entry_content.to_string(),
    }));
    sources
}

fn compile_multi(module_index: &ModuleIndex, files: &[(&str, &str)]) -> Rc<PipelineResult> {
    let mut all_sources: HashMap<String, Rc<SourceFile>> = HashMap::new();
    for (path, content) in files {
        let resolved = resolve_imports_transitively(path, content, module_index);
        for src in resolved {
            all_sources.entry(src.path.clone()).or_insert(src);
        }
    }
    let sources: Vec<Rc<SourceFile>> = all_sources.into_iter().map(|(_, v)| v).collect();
    compile_sources(Rc::new(sources), RenderTarget::Rust)
}

fn diag_line_col(diag: &ErrorNode, source: &str, file: &str) -> (i64, i64) {
    let span = diagnostic_to_span(diag.diagnostic.clone());
    let idx = build_newline_index(file.to_string(), source.to_string());
    let lc = byte_to_line_col(idx, span.start);
    (lc.line, lc.col)
}

fn diagnostic_messages(result: &PipelineResult) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

fn has_arity_mismatch(result: &PipelineResult) -> bool {
    result
        .diagnostics
        .iter()
        .any(|d| matches!(&*d.diagnostic, CompilerDiagnostic::ArityMismatch { .. }))
}

fn first_arity_mismatch_message(result: &PipelineResult) -> Option<String> {
    result.diagnostics.iter().find_map(|d| {
        if matches!(&*d.diagnostic, CompilerDiagnostic::ArityMismatch { .. }) {
            Some(diagnostic_to_message(d.diagnostic.clone()))
        } else {
            None
        }
    })
}

/// Golden: MissingExport names export/module/importer and span points at the missing name.
/// Also covers UnresolvedImport variant + grounded message.
fn import_resolution_variants_and_span(module_index: &ModuleIndex) {
    let provider = "module provider\ntype User { name: String }\n";
    let missing_export = "module consumer\nimport provider { NonExistent }\n";
    let missing_result = compile_multi(
        module_index,
        &[("provider.dag", provider), ("consumer.dag", missing_export)],
    );
    assert_eq!(missing_result.diagnostics.len(), 1);
    let missing = &missing_result.diagnostics[0];
    assert!(
        matches!(
            &*missing.diagnostic,
            CompilerDiagnostic::MissingExport { .. }
        ),
        "expected MissingExport, got: {:?}",
        missing.diagnostic
    );
    let missing_msg = diagnostic_to_message(missing.diagnostic.clone());
    assert!(
        missing_msg.contains("NonExistent"),
        "missing export: {missing_msg}"
    );
    assert!(
        missing_msg.contains("provider"),
        "target module: {missing_msg}"
    );
    assert!(
        missing_msg.contains("consumer"),
        "importing module: {missing_msg}"
    );
    let (line, col) = diag_line_col(missing, missing_export, "consumer.dag");
    assert_eq!(line, 2, "span should be on import line");
    assert_eq!(
        col, 19,
        "span should point at missing name, not import keyword"
    );

    let unresolved = "module consumer\nimport nonexistent { Thing }\n";
    let unresolved_result = compile_multi(module_index, &[("consumer.dag", unresolved)]);
    assert!(
        !unresolved_result.diagnostics.is_empty(),
        "expected UnresolvedImport diagnostic"
    );
    let unresolved_diag = &unresolved_result.diagnostics[0];
    assert!(
        matches!(
            &*unresolved_diag.diagnostic,
            CompilerDiagnostic::UnresolvedImport { .. }
        ),
        "expected UnresolvedImport, got: {:?}",
        unresolved_diag.diagnostic
    );
    let unresolved_msg = diagnostic_to_message(unresolved_diag.diagnostic.clone());
    assert!(unresolved_msg.contains("nonexistent"), "{unresolved_msg}");
    assert!(unresolved_msg.contains("consumer"), "{unresolved_msg}");
}

/// Golden: variant not re-exported through a type-only import surface.
fn reexport_surface_missing_variant(module_index: &ModuleIndex) {
    let files = &[
        ("def.dag", "module self_gen8_def\ntype E = A | B\n"),
        (
            "proxy.dag",
            "module self_gen8_proxy\nimport self_gen8_def { E }\n",
        ),
        (
            "use_mod.dag",
            "module self_gen8_use\nimport self_gen8_proxy { B }\n",
        ),
    ];
    let result = compile_multi(module_index, files);
    assert_eq!(result.diagnostics.len(), 1);
    let d = &result.diagnostics[0];
    assert!(
        matches!(&*d.diagnostic, CompilerDiagnostic::MissingExport { .. }),
        "expected MissingExport for variant not in proxy export surface, got: {:?}",
        d.diagnostic
    );
    let msg = diagnostic_to_message(d.diagnostic.clone());
    assert!(msg.contains("B"), "missing variant export: {msg}");
    assert!(msg.contains("self_gen8_proxy"), "proxy module: {msg}");
}

/// Golden: UnresolvedType names the unknown type; ArityMismatch fires on bare containers
/// but not on parameterized std containers or user-defined types.
fn type_and_arity_discrimination(module_index: &ModuleIndex) {
    let unresolved_source = "module types\ntype Wrapper { inner: Bogus }\n";
    let unresolved_result = compile_multi(module_index, &[("types.dag", unresolved_source)]);
    let unresolved: Vec<_> = unresolved_result
        .diagnostics
        .iter()
        .filter(|d| matches!(&*d.diagnostic, CompilerDiagnostic::UnresolvedType { .. }))
        .collect();
    assert!(
        !unresolved.is_empty(),
        "expected UnresolvedType, got: {:?}",
        diagnostic_messages(&unresolved_result)
    );
    let unresolved_msg = diagnostic_to_message(unresolved[0].diagnostic.clone());
    assert!(unresolved_msg.contains("Bogus"), "{unresolved_msg}");

    let bare = "module bare\nimport std.types { List }\ntype Foo { items: List }\n";
    let bare_result = compile_multi(module_index, &[("bare.dag", bare)]);
    assert!(
        has_arity_mismatch(&bare_result),
        "bare List should trigger ArityMismatch, got: {:?}",
        diagnostic_messages(&bare_result)
    );
    let bare_msg = first_arity_mismatch_message(&bare_result).expect("arity message");
    assert!(bare_msg.contains("List"), "ArityMismatch should name List");

    let parameterized = "module param\nimport std.types { List }\ntype Foo { items: List<Int> }\n";
    let parameterized_result = compile_multi(module_index, &[("param.dag", parameterized)]);
    assert!(
        !has_arity_mismatch(&parameterized_result),
        "parameterized List<Int> should not trigger ArityMismatch, got: {:?}",
        diagnostic_messages(&parameterized_result)
    );

    let user_defined = "module custom\ntype Widget { label: String }\ntype Bag { item: Widget }\n";
    let user_defined_result = compile_multi(module_index, &[("custom.dag", user_defined)]);
    assert!(
        !has_arity_mismatch(&user_defined_result),
        "user-defined type should not trigger ArityMismatch, got: {:?}",
        diagnostic_messages(&user_defined_result)
    );
}

/// Golden: empty list literal fails without collection context; succeeds with List<T> context.
fn empty_list_literal_context(module_index: &ModuleIndex) {
    let wrong = "module elist\nfn make_stuff() -> String {\n  []\n}\n";
    let wrong_result = compile_multi(module_index, &[("elist.dag", wrong)]);
    let wrong_diags: Vec<_> = wrong_result
        .diagnostics
        .iter()
        .filter(|d| {
            d.module_name == "elist"
                && match &*d.diagnostic {
                    CompilerDiagnostic::InternalError { message, .. } => {
                        message.contains("empty list literal")
                    }
                    _ => false,
                }
        })
        .collect();
    assert!(
        !wrong_diags.is_empty(),
        "empty list with non-collection expected type should diagnose, got: {:?}",
        diagnostic_messages(&wrong_result)
    );

    let ok =
        "module elist_ok\nimport std.types { List }\nfn make_list() -> List<String> {\n  []\n}\n";
    let ok_result = compile_multi(module_index, &[("elist_ok.dag", ok)]);
    let ok_diags: Vec<_> = ok_result
        .diagnostics
        .iter()
        .filter(|d| {
            d.module_name == "elist_ok"
                && match &*d.diagnostic {
                    CompilerDiagnostic::InternalError { message, .. } => {
                        message.contains("empty list literal")
                    }
                    _ => false,
                }
        })
        .collect();
    assert!(
        ok_diags.is_empty(),
        "empty list with List<String> context should not diagnose, got: {:?}",
        diagnostic_messages(&ok_result)
    );
}

/// Constructor-owner ruling (§1c) walls. RED control: the retired CI canary
/// fixture (two local coproducts sharing SharedVariant, disambiguated by the
/// deleted expected-type pick) must now raise VariantCollision. RED control:
/// an unbound constructor literal must raise UnresolvedType. GREEN control:
/// a sole-owner arm constructs clean.
fn constructor_owner_ruling_walls(module_index: &ModuleIndex) {
    let collided = "module test.claim.variant_owner_expected_type\n\
        import std.logic { Bool }\n\
        type AEarlyOwner = SharedVariant | AOnlyVariant\n\
        type ZLaterOwner = SharedVariant | ZOnlyVariant\n\
        fn make_z_variant() -> ZLaterOwner { SharedVariant }\n";
    let collided_result = compile_multi(module_index, &[("variant_owner.dag", collided)]);
    assert!(
        collided_result.diagnostics.iter().any(|d| matches!(
            &*d.diagnostic,
            CompilerDiagnostic::VariantCollision { variant, .. } if variant == "SharedVariant"
        )),
        "retired canary fixture must raise VariantCollision, got: {:?}",
        diagnostic_messages(&collided_result)
    );

    let unbound = "module test.claim.unbound_ctor\n\
        fn mk() -> Int { let g = GhostArm { x: 1 } 2 }\n";
    let unbound_result = compile_multi(module_index, &[("unbound_ctor.dag", unbound)]);
    assert!(
        unbound_result.diagnostics.iter().any(|d| matches!(
            &*d.diagnostic,
            CompilerDiagnostic::UnresolvedType { name, .. } if name == "GhostArm"
        )),
        "unbound constructor literal must raise UnresolvedType, got: {:?}",
        diagnostic_messages(&unbound_result)
    );

    let sole = "module test.claim.sole_owner\n\
        type OnlyOwner = SoleArm | OtherArm\n\
        fn mk() -> OnlyOwner { SoleArm }\n";
    let sole_result = compile_multi(module_index, &[("sole_owner.dag", sole)]);
    let sole_hard: Vec<String> = diagnostic_messages(&sole_result)
        .into_iter()
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        sole_hard.is_empty(),
        "sole-owner construction must stay clean, got: {sole_hard:?}"
    );
}

fn suite_cases(suite: &str) -> Result<Vec<WitnessCase>, String> {
    match suite {
        SUITE_IMPORT_RESOLUTION => Ok(vec![(
            "import_resolution_variants_and_span",
            import_resolution_variants_and_span,
        )]),
        SUITE_REEXPORT_SURFACE => Ok(vec![(
            "reexport_surface_missing_variant",
            reexport_surface_missing_variant,
        )]),
        SUITE_TYPE_AND_ARITY => Ok(vec![(
            "type_and_arity_discrimination",
            type_and_arity_discrimination,
        )]),
        SUITE_EMPTY_LIST_CONTEXT => Ok(vec![(
            "empty_list_literal_context",
            empty_list_literal_context,
        )]),
        SUITE_CONSTRUCTOR_OWNER => Ok(vec![(
            "constructor_owner_ruling_walls",
            constructor_owner_ruling_walls,
        )]),
        _ => Err(format!(
            "unknown suite '{suite}'; expected one of: {SUITE_IMPORT_RESOLUTION}, \
             {SUITE_REEXPORT_SURFACE}, {SUITE_TYPE_AND_ARITY}, {SUITE_EMPTY_LIST_CONTEXT}, \
             {SUITE_CONSTRUCTOR_OWNER}"
        )),
    }
}

fn run_suite(module_index: &ModuleIndex, suite: &str) -> Result<(), String> {
    for (name, test) in suite_cases(suite)? {
        let index = module_index.clone();
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| test(&index))).is_err() {
            return Err(format!("{name} panicked"));
        }
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let suite = args.first().map(String::as_str).unwrap_or("");
    if suite.is_empty() {
        return fail("usage: diagnostics_witness <suite>");
    }

    let module_index = build_module_index();
    match run_suite(&module_index, suite) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => fail(msg),
    }
}
