#![allow(clippy::disallowed_macros)]

use im::HashMap;
use im::{vector as vec, Vector as Vec};
use std::process::ExitCode;
use std::rc::Rc;
use v1_compiler::v1_rt::VecCompat;

use v1_compiler::cli_run::workspace_root;
use v1_compiler::std_induction::SubValueRelation;
use v1_compiler::std_types::container_param_name;
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{
    compile_sources, compile_to_resolved, PipelineResult, ResolvedPipelineResult, SourceFile,
};
use v1_compiler::v1_compiler_infer::InferScope;
use v1_compiler::v1_compiler_infer_access;
use v1_compiler::v1_compiler_infer_env::lookup_type_by_name;
use v1_compiler::v1_compiler_infer_env::{TypeBinding, TypeEnv};
use v1_compiler::v1_compiler_infer_lookup;
use v1_compiler::v1_compiler_infer_patterns::{self, NodeLookupStatus};
use v1_compiler::v1_compiler_infer_resolve::resolve_node;
use v1_compiler::v1_compiler_infer_sigs::ResolvedFuncEnv;
use v1_compiler::v1_compiler_infer_types::{
    bare_map_node, is_fully_resolved, make_optional_type, node_is_keyed_collection,
    node_type_compatible, nominal_type_ref, resolved_type, type_resolution_verdict,
    TypeResolutionVerdict,
};
use v1_compiler::v1_compiler_parse;
use v1_compiler::v1_compiler_trait_derive_emit::{v1_type_expr_keyed_map_verdict, KeyedMapVerdict};
use v1_compiler::v1_std_core::NewlineIndex;
use v1_compiler::v1_std_core::{
    authored_container_spelling_verdict, build_newline_index, default_ident_span,
    leaf_node_with_span, make_arm_node, no_span, Cardinality, CompilerDiagnostic, Connective,
    ContainerSpellingVerdict, ExprData, InferredNode, MatchPattern, Node, SourceSpan,
};

fn fail(msg: impl std::fmt::Display) -> ExitCode {
    eprintln!("infer_semantics_witness: {msg}");
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

fn scan_dag_files(dir: &std::path::Path, index: &mut HashMap<String, std::path::PathBuf>) {
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

fn build_module_index() -> HashMap<String, std::path::PathBuf> {
    let mut index = HashMap::new();
    for root in source_roots() {
        if root.exists() {
            scan_dag_files(&root, &mut index);
        }
    }
    index
}

fn extract_imports(source: &str) -> Vec<String> {
    let tokens = v1_compiler::v1_compiler_tokenize::tokenize(
        source.to_string(),
        "test.dag".to_string(),
        v1_compiler::extdeps_languages_dag_syntax::dag_parse_environment(),
    );
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
    module_index: &HashMap<String, std::path::PathBuf>,
) -> Vec<Rc<SourceFile>> {
    let ws = workspace_root();
    let mut seen: HashMap<String, Rc<SourceFile>> = HashMap::new();
    let mut queue = vec![(entry_path.to_string(), entry_content.to_string())];

    while let Some((_path, content)) = queue.pop_back() {
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

fn compile_dag(source: &str) -> Rc<PipelineResult> {
    let module_index = build_module_index();
    let sources = resolve_imports_transitively("test.dag", source, &module_index);
    compile_sources(Rc::new(sources), RenderTarget::Rust)
}

fn compile_dag_resolved(source: &str) -> Rc<ResolvedPipelineResult> {
    let module_index = build_module_index();
    let sources = resolve_imports_transitively("test.dag", source, &module_index);
    compile_to_resolved(Rc::new(sources))
}

fn assert_no_diagnostics(result: &PipelineResult) {
    if !result.diagnostics.is_empty() {
        let messages: Vec<String> = result
            .diagnostics
            .iter()
            .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
            .collect();
        panic!("expected no diagnostics, got: {messages:?}");
    }
}

fn diagnostic_messages(result: &PipelineResult) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

fn empty_source_indices() -> Rc<HashMap<String, Rc<NewlineIndex>>> {
    Rc::new(im::HashMap::new())
}

fn leaf_node(name: String) -> Rc<Node> {
    leaf_node_with_span(
        Rc::new(v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
        name,
        no_span(),
    )
}

fn container_node(kind_name: String, element: Rc<Node>) -> Rc<Node> {
    let param_name = match container_param_name(kind_name.clone(), 0) {
        Some(n) => n,
        None => kind_name.clone(),
    };
    let sp = no_span();
    Rc::new(Node {
        occurrence_identity: Rc::new(
            v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
        ),
        name: kind_name.clone(),
        ident: None,
        span: sp.clone(),
        ident_span: default_ident_span(kind_name, sp.clone()),
        children: Rc::new(vec![Rc::new(Node {
            occurrence_identity: Rc::new(
                v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
            ),
            name: param_name.clone(),
            ident: None,
            span: sp.clone(),
            ident_span: default_ident_span(param_name, sp.clone()),
            children: Rc::new(vec![]),
            connective: Connective::NoConnective,
            params: Rc::new(vec![]),
            inferred: Some(Rc::new(InferredNode::Resolved { node: element })),
            return_cardinality: Cardinality::Required,
            uses: Rc::new(vec![]),
            body: None,
            transport: None,
            properties: Rc::new(vec![]),
            type_annotation: None,
            is_self_recursive: false,
            has_non_tail_self_call: false,
            match_pattern: None,
            module_item_kind: v1_compiler::v1_std_core::ParsedModuleItemKind::NotAModuleItem,
            expr_data: Rc::new(ExprData::NoExprData),
        })]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        module_item_kind: v1_compiler::v1_std_core::ParsedModuleItemKind::NotAModuleItem,
        expr_data: Rc::new(ExprData::NoExprData),
    })
}

fn map_node(key: Rc<Node>, value: Rc<Node>) -> Rc<Node> {
    let key_name = container_param_name("Map".to_string(), 0)
        .expect("kernel Map should resolve K from PartialFunction profile");
    let val_name = container_param_name("Map".to_string(), 1)
        .expect("kernel Map should resolve V from PartialFunction profile");
    let sp = no_span();
    Rc::new(Node {
        occurrence_identity: Rc::new(v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
        name: "Map".to_string(),
        ident: None,
        span: sp.clone(),
        ident_span: Some(sp.clone()),
        children: Rc::new(vec![
            Rc::new(Node {
                occurrence_identity: Rc::new(v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
                name: key_name,
                ident: None,
                span: sp.clone(),
                ident_span: Some(sp.clone()),
                children: Rc::new(vec![]),
                connective: Connective::NoConnective,
                params: Rc::new(vec![]),
                inferred: Some(Rc::new(InferredNode::Resolved { node: key })),
                return_cardinality: Cardinality::Required,
                uses: Rc::new(vec![]),
                body: None,
                transport: None,
                properties: Rc::new(vec![]),
                type_annotation: None,
                is_self_recursive: false,
                has_non_tail_self_call: false,
                match_pattern: None,
                module_item_kind: v1_compiler::v1_std_core::ParsedModuleItemKind::NotAModuleItem, expr_data: Rc::new(ExprData::NoExprData),
            }),
            Rc::new(Node {
                occurrence_identity: Rc::new(v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
                name: val_name,
                ident: None,
                span: sp.clone(),
                ident_span: Some(sp.clone()),
                children: Rc::new(vec![]),
                connective: Connective::NoConnective,
                params: Rc::new(vec![]),
                inferred: Some(Rc::new(InferredNode::Resolved { node: value })),
                return_cardinality: Cardinality::Required,
                uses: Rc::new(vec![]),
                body: None,
                transport: None,
                properties: Rc::new(vec![]),
                type_annotation: None,
                is_self_recursive: false,
                has_non_tail_self_call: false,
                match_pattern: None,
                module_item_kind: v1_compiler::v1_std_core::ParsedModuleItemKind::NotAModuleItem, expr_data: Rc::new(ExprData::NoExprData),
            }),
        ]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        module_item_kind: v1_compiler::v1_std_core::ParsedModuleItemKind::NotAModuleItem, expr_data: Rc::new(ExprData::NoExprData),
    })
}

fn zero_span() -> Rc<SourceSpan> {
    Rc::new(SourceSpan {
        file: String::new(),
        start: 0,
        end: 0,
    })
}

fn unit_expr() -> Rc<Node> {
    leaf_node("Unit".to_string())
}

fn empty_type_env() -> Rc<TypeEnv> {
    Rc::new(TypeEnv {
        module_path: "".to_string(),
        authored_import_names: Rc::new(im::HashMap::new()),
        bindings: Rc::new(im::HashMap::new()),
        str_bindings: Rc::new(im::HashMap::new()),
        unit_variant_index: Rc::new(im::HashMap::new()),
        unit_variant_index_observed: false,
        ancestry_str_bindings: Rc::new(im::HashMap::new()),
        parents: Rc::new(vec![]),
        recursive_types: Rc::new(vec![]),
        recursive_type_set: Rc::new(im::HashMap::new()),
        inductive_fields: Rc::new(im::HashMap::new()),
        source_indices: Rc::new(im::HashMap::new()),
        intern_table: v1_compiler::v1_std_core::empty_intern_table(),
        source_visible_names: Rc::new(im::HashMap::new()),
        symbol_index: v1_compiler::v1_compiler_infer_env::empty_symbol_index(),
    })
}

fn empty_infer_scope() -> Rc<InferScope> {
    Rc::new(InferScope {
        type_env: empty_type_env(),
        func_env: Rc::new(ResolvedFuncEnv {
            name: "test".to_string(),
            local: Rc::new(im::HashMap::new()),
            parents: Rc::new(vec![]),
        }),
        locals: Rc::new(im::HashMap::new()),
        body_locals: Rc::new(im::HashMap::new()),
        match_bound_names: Rc::new(im::HashMap::new()),
        module_name: "test".to_string(),
        service_registry: Rc::new(im::HashMap::new()),
        item_registry: Rc::new(im::HashMap::new()),
        lambda_param_provenance: Rc::new(im::HashMap::new()),
        caller_decl_name: "".to_string(),
    })
}

fn sum_node(name: &str, variants: Vec<Rc<Node>>, cardinality: Cardinality) -> Rc<Node> {
    let sp = no_span();
    Rc::new(Node {
        occurrence_identity: Rc::new(
            v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
        ),
        name: name.to_string(),
        ident: None,
        span: sp.clone(),
        ident_span: default_ident_span(name.to_string(), sp),
        children: Rc::new(variants),
        connective: Connective::Disj,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: cardinality,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        module_item_kind: v1_compiler::v1_std_core::ParsedModuleItemKind::NotAModuleItem,
        expr_data: Rc::new(ExprData::NoExprData),
    })
}

fn variant_arm(name: &str) -> Rc<Node> {
    make_arm_node(
        Rc::new(v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
        Rc::new(MatchPattern::VariantPattern {
            name: name.to_string(),
            parent_enum: None,
            field_bindings: Rc::new(vec![]),
        }),
        None,
        unit_expr(),
        zero_span(),
    )
}

fn assert_compiler_error(inferred: &Option<Rc<InferredNode>>, message_fragment: &str) {
    match inferred.as_ref().expect("expected inferred").as_ref() {
        InferredNode::CompilerError { message, .. } => {
            assert!(
                message.contains(message_fragment),
                "expected compiler error containing '{message_fragment}', got '{message}'"
            );
        }
        other => panic!("expected CompilerError return type, got {:?}", other),
    }
}

fn m1_brand_twins_over_refined_base_remain_distinct_in_infer_representation() {
    let source = r#"
module m1.brand_twins

type Refined<T> {
  base: T
}
type UserId = Refined<String>
type AccountId = Refined<String>
"#;

    let result = compile_dag_resolved(source);
    assert!(
        result.diagnostics.is_empty(),
        "brand-twin infer probe should compile without diagnostics, got: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
            .collect::<Vec<_>>()
    );
    let graph = result.graph.as_ref().expect("resolved graph");
    let module = graph
        .modules
        .iter()
        .find(|m| {
            v1_compiler::v1_std_core::authored_name_at(
                result.source_indices.clone(),
                m.module.clone(),
            ) == "m1.brand_twins"
        })
        .expect("m1.brand_twins module");

    let user_id = lookup_type_by_name(module.type_env.clone(), "UserId".to_string())
        .expect("UserId type binding");
    let same_user_id = lookup_type_by_name(module.type_env.clone(), "UserId".to_string())
        .expect("second UserId type binding");
    let account_id = lookup_type_by_name(module.type_env.clone(), "AccountId".to_string())
        .expect("AccountId type binding");
    let refined_string = lookup_type_by_name(module.type_env.clone(), "Refined".to_string())
        .expect("Refined type binding");
    let string = lookup_type_by_name(module.type_env.clone(), "String".to_string())
        .expect("String type binding");

    assert_eq!(
        user_id, same_user_id,
        "M1 positive control: two references to the same brand declaration must compare equal"
    );
    assert_ne!(
        user_id, account_id,
        "M1 brand twins must keep distinct declaration identities; collapse here means A3 needs brand-aware rework"
    );
    assert_ne!(
        user_id, refined_string,
        "UserId must not collapse to the shared Refined carrier"
    );
    assert_ne!(
        account_id, refined_string,
        "AccountId must not collapse to the shared Refined carrier"
    );
    assert_ne!(
        user_id, string,
        "UserId must not collapse to the shared base String"
    );
    assert_ne!(
        account_id, string,
        "AccountId must not collapse to the shared base String"
    );
}

fn pd3_brand_twins_incompatible_at_node_type_compatible() {
    let source = r#"
module pd3.brand_relation

type Refined<T> {
  base: T
}
type UserId = Refined<String>
type AccountId = Refined<String>
"#;

    let result = compile_dag_resolved(source);
    assert!(
        result.diagnostics.is_empty(),
        "PD-3 relation probe should resolve cleanly, got: {:?}",
        result
            .diagnostics
            .iter()
            .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
            .collect::<Vec<_>>()
    );
    let graph = result.graph.as_ref().expect("resolved graph");
    let module = graph
        .modules
        .iter()
        .find(|m| {
            v1_compiler::v1_std_core::authored_name_at(
                result.source_indices.clone(),
                m.module.clone(),
            ) == "pd3.brand_relation"
        })
        .expect("pd3.brand_relation module");

    let user_id =
        lookup_type_by_name(module.type_env.clone(), "UserId".to_string()).expect("UserId binding");
    let account_id = lookup_type_by_name(module.type_env.clone(), "AccountId".to_string())
        .expect("AccountId binding");

    assert!(
        !node_type_compatible(user_id.clone(), account_id, result.source_indices.clone()),
        "PD-3: node_type_compatible must reject brand-twin UserId-for-AccountId"
    );
    assert!(
        node_type_compatible(user_id.clone(), user_id, result.source_indices.clone()),
        "PD-3: node_type_compatible must accept same-brand UserId-for-UserId"
    );
}

fn pd3_direct_call_rejects_brand_twin_mismatch() {
    let source = r#"
module pd3.brand_call_reject

type Refined<T> {
  base: T
}
type UserId = Refined<String>
type AccountId = Refined<String>

fn take_account(id: AccountId) -> String {
  ""
}

fn caller(uid: UserId) -> String {
  take_account(uid)
}
"#;

    let result = compile_dag(source);
    let has_type_mismatch = result
        .diagnostics
        .iter()
        .any(|diag| matches!(&*diag.diagnostic, CompilerDiagnostic::TypeMismatch { .. }));
    assert!(
        has_type_mismatch,
        "PD-3: direct call must reject UserId-for-AccountId, got: {:?}",
        diagnostic_messages(&result)
    );
}

fn pd3_direct_call_accepts_same_brand() {
    let source = r#"
module pd3.brand_call_accept

type Refined<T> {
  base: T
}
type UserId = Refined<String>

fn take_user(id: UserId) -> String {
  ""
}

fn caller(uid: UserId) -> String {
  take_user(uid)
}
"#;

    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

fn pd3_direct_call_accepts_list_for_freemonoid_alias() {
    let source = r#"
module pd3.alias_call_accept

fn take_fm(xs: FreeMonoid<Int>) -> Int {
  0
}

fn caller(xs: List<Int>) -> Int {
  take_fm(xs)
}
"#;

    let result = compile_dag(source);
    assert_no_diagnostics(&result);
}

fn list_int_index_returns_optional_element_type() {
    let result = v1_compiler_infer_access::check_index_access_node(
        container_node("List".to_string(), leaf_node("Int".to_string())),
        leaf_node("Int".to_string()),
        zero_span(),
        "test".to_string(),
        empty_source_indices(),
    );

    assert_eq!(
        result.diagnostics.len(),
        0,
        "List<Int> indexed by Int should succeed"
    );
    assert!(
        result.inferred.is_some(),
        "List<Int> index should produce a type"
    );
}

fn malformed_map_index_returns_compiler_error_type() {
    let result = v1_compiler_infer_access::check_index_access_node(
        bare_map_node().expect("Map kernel container profile"),
        leaf_node("String".to_string()),
        zero_span(),
        "test".to_string(),
        empty_source_indices(),
    );

    assert_eq!(result.diagnostics.len(), 1);
    assert_compiler_error(&result.inferred, "key type does not match");
}

// Negative control for slice admission. `ordered_element_collections()`
// (std_types.rs) holds `List` alone, so a `Map` base is neither String nor an
// ordered element collection and must still refuse. Re-pointed from `List<Int>`
// at #7196, which deliberately admitted list slicing (`base_is_list` in
// `check_slice_access_node`) and left this control pinning the withdrawn
// refusal — the witness went stale, the compiler did not.
fn invalid_slice_returns_compiler_error_type() {
    let result = v1_compiler_infer_access::check_slice_access_node(
        map_node(
            leaf_node("String".to_string()),
            leaf_node("Int".to_string()),
        ),
        leaf_node("Int".to_string()),
        leaf_node("Int".to_string()),
        zero_span(),
        "test".to_string(),
        empty_source_indices(),
    );

    assert_eq!(result.diagnostics.len(), 1);
    assert_compiler_error(&result.inferred, "slice is only supported");
}

// Positive control #7196 admitted but never witnessed: a list slice is legal and
// preserves the base type (`slice_result_type = normed_base` on the list arm),
// where a String slice yields String. Discriminating against the pre-#7196
// behaviour, which produced one diagnostic and a String result here.
fn valid_list_slice_preserves_list_type() {
    let result = v1_compiler_infer_access::check_slice_access_node(
        container_node("List".to_string(), leaf_node("Int".to_string())),
        leaf_node("Int".to_string()),
        leaf_node("Int".to_string()),
        zero_span(),
        "test".to_string(),
        empty_source_indices(),
    );

    assert!(result.diagnostics.is_empty());
    match result
        .inferred
        .as_ref()
        .expect("expected return type")
        .as_ref()
    {
        InferredNode::Resolved { node, .. } => {
            assert_eq!(node.name, "List");
        }
        other => panic!("expected resolved list return type, got {:?}", other),
    }
}

fn valid_map_index_preserves_optional_value_type() {
    let result = v1_compiler_infer_access::check_index_access_node(
        map_node(
            leaf_node("String".to_string()),
            leaf_node("Int".to_string()),
        ),
        leaf_node("String".to_string()),
        zero_span(),
        "test".to_string(),
        empty_source_indices(),
    );

    assert!(result.diagnostics.is_empty());
    match result
        .inferred
        .as_ref()
        .expect("expected return type")
        .as_ref()
    {
        InferredNode::Resolved { node, .. } => {
            assert_eq!(node.name, "Optional");
            assert_eq!(node.return_cardinality, Cardinality::Required);
            assert_eq!(node.children.len(), 1);
            assert_eq!(node.children[0].name, "Int");
        }
        other => panic!("expected resolved return type, got {:?}", other),
    }
}

fn pattern_lookup_blocks_on_infer_error_without_cascade_diagnostic() {
    let subject = v1_compiler_infer_patterns::pattern_subject_from_inferred(Some(Rc::new(
        InferredNode::CompilerError {
            message: "upstream failure".to_string(),
            span: zero_span(),
        },
    )));
    let lookup = v1_compiler_infer_patterns::lookup_variant_in_type(
        subject,
        "Some".to_string(),
        "test".to_string(),
        empty_type_env(),
        0,
    );

    assert!(matches!(
        lookup.status.as_ref(),
        NodeLookupStatus::LookupFailed
    ));
    assert!(
        lookup.diagnostics.is_empty(),
        "upstream infer failure should not add cascade diagnostics"
    );
}

fn pattern_lookup_reports_error_scrutinee_structurally() {
    use v1_compiler::v1_std_core::error_type;
    let subject = v1_compiler_infer_patterns::pattern_subject_from_node(error_type());
    let lookup = v1_compiler_infer_patterns::lookup_variant_in_type(
        subject,
        "Some".to_string(),
        "test".to_string(),
        empty_type_env(),
        0,
    );

    assert!(matches!(
        lookup.status.as_ref(),
        NodeLookupStatus::LookupFailed
    ));
}

fn optional_pattern_lookup_rejects_some_variant() {
    let subject = v1_compiler_infer_patterns::pattern_subject_from_node(make_optional_type(
        leaf_node("String".to_string()),
    ));
    let lookup = v1_compiler_infer_patterns::lookup_variant_in_type(
        subject,
        "Some".to_string(),
        "test".to_string(),
        empty_type_env(),
        0,
    );

    assert!(matches!(
        lookup.status.as_ref(),
        NodeLookupStatus::LookupFailed
    ));
    assert_eq!(lookup.diagnostics.len(), 1);
}

fn optional_pattern_lookup_resolves_present_variant() {
    let subject = v1_compiler_infer_patterns::pattern_subject_from_node(make_optional_type(
        leaf_node("String".to_string()),
    ));
    let lookup = v1_compiler_infer_patterns::lookup_variant_in_type(
        subject,
        "Present".to_string(),
        "test".to_string(),
        empty_type_env(),
        0,
    );

    match lookup.status.as_ref() {
        NodeLookupStatus::LookupResolved { node, .. } => {
            assert_eq!(node.name, "Present");
            assert_eq!(node.children.len(), 1);
            assert_eq!(node.children[0].name, "value");
        }
        status => panic!("expected Present lookup to resolve, got {:?}", status),
    }
}

fn optional_pattern_lookup_prefers_optional_present_over_inner_present_variant() {
    let sp = no_span();
    let inner_present = Rc::new(Node {
        occurrence_identity: Rc::new(
            v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
        ),
        name: "Present".to_string(),
        ident: None,
        span: sp.clone(),
        ident_span: default_ident_span("Present".to_string(), sp.clone()),
        children: Rc::new(vec![Rc::new(Node {
            occurrence_identity: Rc::new(
                v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
            ),
            name: "inner".to_string(),
            ident: None,
            span: sp.clone(),
            ident_span: default_ident_span("inner".to_string(), sp.clone()),
            children: Rc::new(vec![]),
            connective: Connective::NoConnective,
            params: Rc::new(vec![]),
            inferred: Some(Rc::new(InferredNode::Resolved {
                node: leaf_node("Int".to_string()),
            })),
            return_cardinality: Cardinality::Required,
            uses: Rc::new(vec![]),
            body: None,
            transport: None,
            properties: Rc::new(vec![]),
            type_annotation: None,
            is_self_recursive: false,
            has_non_tail_self_call: false,
            match_pattern: None,
            module_item_kind: v1_compiler::v1_std_core::ParsedModuleItemKind::NotAModuleItem,
            expr_data: Rc::new(ExprData::NoExprData),
        })]),
        connective: Connective::Conj,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        module_item_kind: v1_compiler::v1_std_core::ParsedModuleItemKind::NotAModuleItem,
        expr_data: Rc::new(ExprData::NoExprData),
    });
    let optional_inner_sum = make_optional_type(Rc::new(Node {
        occurrence_identity: Rc::new(
            v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
        ),
        name: "Inner".to_string(),
        ident: None,
        span: sp.clone(),
        ident_span: default_ident_span("Inner".to_string(), sp.clone()),
        children: Rc::new(vec![inner_present]),
        connective: Connective::Disj,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        module_item_kind: v1_compiler::v1_std_core::ParsedModuleItemKind::NotAModuleItem,
        expr_data: Rc::new(ExprData::NoExprData),
    }));
    let subject = v1_compiler_infer_patterns::pattern_subject_from_node(optional_inner_sum);
    let lookup = v1_compiler_infer_patterns::lookup_variant_in_type(
        subject,
        "Present".to_string(),
        "test".to_string(),
        empty_type_env(),
        1,
    );

    match lookup.status.as_ref() {
        NodeLookupStatus::LookupResolved { node, .. } => {
            assert_eq!(node.name, "Present");
            assert_eq!(node.children[0].name, "value");
        }
        status => panic!(
            "expected Optional Present lookup to resolve as Present, got {:?}",
            status
        ),
    }
}

fn optional_present_absent_patterns_keep_canonical_names() {
    let scope = empty_infer_scope();
    let subject = v1_compiler_infer_patterns::pattern_subject_from_node(make_optional_type(
        leaf_node("String".to_string()),
    ));

    let present = v1_compiler::v1_compiler_infer::annotate_pattern_parent_enums(
        Rc::new(MatchPattern::VariantPattern {
            name: "Present".to_string(),
            parent_enum: None,
            field_bindings: Rc::new(vec![]),
        }),
        subject.clone(),
        scope.clone(),
    );
    let absent = v1_compiler::v1_compiler_infer::annotate_pattern_parent_enums(
        Rc::new(MatchPattern::VariantPattern {
            name: "Absent".to_string(),
            parent_enum: None,
            field_bindings: Rc::new(vec![]),
        }),
        subject,
        scope,
    );

    assert!(matches!(
        present.as_ref(),
        MatchPattern::VariantPattern { name, parent_enum: Some(parent), .. }
          if name == "Present" && parent == "Optional"
    ));
    assert!(matches!(
        absent.as_ref(),
        MatchPattern::VariantPattern { name, parent_enum: Some(parent), .. }
          if name == "Absent" && parent == "Optional"
    ));
}

fn applied_generic_type_node(type_name: &str, type_arg: Rc<Node>) -> Rc<Node> {
    Rc::new(Node {
        occurrence_identity: Rc::new(
            v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
        ),
        name: type_name.to_string(),
        ident: None,
        span: no_span(),
        ident_span: default_ident_span(type_name.to_string(), no_span()),
        children: Rc::new(vec![type_arg]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        module_item_kind: v1_compiler::v1_std_core::ParsedModuleItemKind::NotAModuleItem,
        expr_data: Rc::new(ExprData::NoExprData),
    })
}

fn optional_applied_generic_lookup_resolves_present_absent_without_disj_children() {
    let applied_optional = make_optional_type(leaf_node("Bool".to_string()));
    let subject = v1_compiler_infer_patterns::pattern_subject_from_node(applied_optional);
    let present_lookup = v1_compiler_infer_patterns::lookup_variant_in_type(
        subject.clone(),
        "Present".to_string(),
        "test".to_string(),
        empty_type_env(),
        1,
    );
    match present_lookup.status.as_ref() {
        NodeLookupStatus::LookupResolved { node, .. } => assert_eq!(node.name, "Present"),
        status => panic!(
            "expected applied Optional<Bool> Present lookup to resolve, got {:?}",
            status
        ),
    }
    let absent_lookup = v1_compiler_infer_patterns::lookup_variant_in_type(
        subject,
        "Absent".to_string(),
        "test".to_string(),
        empty_type_env(),
        0,
    );
    assert!(
        matches!(
            absent_lookup.status.as_ref(),
            NodeLookupStatus::LookupResolved { .. }
        ),
        "expected applied Optional<Bool> Absent lookup to resolve, got {:?}",
        absent_lookup.status
    );
}

fn optional_applied_generic_lookup_rejects_wrong_variant_name() {
    let subject = v1_compiler_infer_patterns::pattern_subject_from_node(make_optional_type(
        leaf_node("Bool".to_string()),
    ));
    let lookup = v1_compiler_infer_patterns::lookup_variant_in_type(
        subject,
        "Some".to_string(),
        "test".to_string(),
        empty_type_env(),
        0,
    );
    assert!(matches!(
        lookup.status.as_ref(),
        NodeLookupStatus::LookupFailed
    ));
    assert_eq!(
        lookup.diagnostics.len(),
        1,
        "wrong variant on applied Optional must fail closed with VariantNotFound"
    );
}

fn non_optional_applied_generic_missing_variant_still_fails() {
    let subject = v1_compiler_infer_patterns::pattern_subject_from_node(applied_generic_type_node(
        "Outcome",
        leaf_node("Bool".to_string()),
    ));
    let lookup = v1_compiler_infer_patterns::lookup_variant_in_type(
        subject,
        "Present".to_string(),
        "test".to_string(),
        empty_type_env(),
        0,
    );
    assert!(matches!(
        lookup.status.as_ref(),
        NodeLookupStatus::LookupFailed
    ));
    assert_eq!(
        lookup.diagnostics.len(),
        1,
        "non-Optional applied type must not get optional_coproduct synthesis"
    );
}

fn real_optional_coproduct_preserves_present_absent_pattern_names() {
    let scope = empty_infer_scope();
    let optional_sum = sum_node(
        "Optional",
        vec![
            leaf_node("Absent".to_string()),
            leaf_node("Present".to_string()),
        ],
        Cardinality::Required,
    );
    let subject = v1_compiler_infer_patterns::pattern_subject_from_node(optional_sum);

    let present = v1_compiler::v1_compiler_infer::annotate_pattern_parent_enums(
        Rc::new(MatchPattern::VariantPattern {
            name: "Present".to_string(),
            parent_enum: None,
            field_bindings: Rc::new(vec![]),
        }),
        subject,
        scope,
    );

    assert!(matches!(
        present.as_ref(),
        MatchPattern::VariantPattern { name, parent_enum: Some(parent), .. }
          if name == "Present" && parent == "Optional"
    ));
}

fn optional_match_exhaustiveness_reports_missing_absent() {
    let diags = v1_compiler_infer_patterns::check_match_exhaustiveness(
        make_optional_type(leaf_node("String".to_string())),
        Rc::new(vec![variant_arm("Present")]),
        Rc::new(TypeEnv {
            module_path: "".to_string(),
            authored_import_names: Rc::new(im::HashMap::new()),
            bindings: Rc::new(im::HashMap::new()),
            str_bindings: Rc::new(im::HashMap::new()),
            unit_variant_index: Rc::new(im::HashMap::new()),
            unit_variant_index_observed: false,
            ancestry_str_bindings: Rc::new(im::HashMap::new()),
            parents: Rc::new(vec![]),
            recursive_types: Rc::new(vec![]),
            recursive_type_set: Rc::new(im::HashMap::new()),
            inductive_fields: Rc::new(im::HashMap::new()),
            source_indices: Rc::new(im::HashMap::new()),
            intern_table: v1_compiler::v1_std_core::empty_intern_table(),
            source_visible_names: Rc::new(im::HashMap::new()),
            symbol_index: v1_compiler::v1_compiler_infer_env::empty_symbol_index(),
        }),
        zero_span(),
        "test".to_string(),
    );

    assert_eq!(diags.len(), 1);
    let diag0_msg = v1_compiler::v1_std_core::diagnostic_to_message(diags[0].diagnostic.clone());
    assert!(diag0_msg.contains("non-exhaustive"));
    assert!(diag0_msg.contains("Absent"));
}

fn optional_match_exhaustiveness_rejects_some_and_none() {
    let diags = v1_compiler_infer_patterns::check_match_exhaustiveness(
        make_optional_type(leaf_node("String".to_string())),
        Rc::new(vec![variant_arm("Some"), variant_arm("None")]),
        Rc::new(TypeEnv {
            module_path: "".to_string(),
            authored_import_names: Rc::new(im::HashMap::new()),
            bindings: Rc::new(im::HashMap::new()),
            str_bindings: Rc::new(im::HashMap::new()),
            unit_variant_index: Rc::new(im::HashMap::new()),
            unit_variant_index_observed: false,
            ancestry_str_bindings: Rc::new(im::HashMap::new()),
            parents: Rc::new(vec![]),
            recursive_types: Rc::new(vec![]),
            recursive_type_set: Rc::new(im::HashMap::new()),
            inductive_fields: Rc::new(im::HashMap::new()),
            source_indices: Rc::new(im::HashMap::new()),
            intern_table: v1_compiler::v1_std_core::empty_intern_table(),
            source_visible_names: Rc::new(im::HashMap::new()),
            symbol_index: v1_compiler::v1_compiler_infer_env::empty_symbol_index(),
        }),
        zero_span(),
        "test".to_string(),
    );

    assert_eq!(diags.len(), 1);
    let diag0_msg = v1_compiler::v1_std_core::diagnostic_to_message(diags[0].diagnostic.clone());
    assert!(diag0_msg.contains("Present"));
    assert!(diag0_msg.contains("Absent"));
}

fn optional_match_exhaustiveness_accepts_present_and_absent() {
    let diags = v1_compiler_infer_patterns::check_match_exhaustiveness(
        make_optional_type(leaf_node("String".to_string())),
        Rc::new(vec![variant_arm("Present"), variant_arm("Absent")]),
        Rc::new(TypeEnv {
            module_path: "".to_string(),
            authored_import_names: Rc::new(im::HashMap::new()),
            bindings: Rc::new(im::HashMap::new()),
            str_bindings: Rc::new(im::HashMap::new()),
            unit_variant_index: Rc::new(im::HashMap::new()),
            unit_variant_index_observed: false,
            ancestry_str_bindings: Rc::new(im::HashMap::new()),
            parents: Rc::new(vec![]),
            recursive_types: Rc::new(vec![]),
            recursive_type_set: Rc::new(im::HashMap::new()),
            inductive_fields: Rc::new(im::HashMap::new()),
            source_indices: Rc::new(im::HashMap::new()),
            intern_table: v1_compiler::v1_std_core::empty_intern_table(),
            source_visible_names: Rc::new(im::HashMap::new()),
            symbol_index: v1_compiler::v1_compiler_infer_env::empty_symbol_index(),
        }),
        zero_span(),
        "test".to_string(),
    );

    assert!(
        diags.is_empty(),
        "Present/Absent arms should exhaust Optional matches, got {:?}",
        diags
    );
}

fn qualified_coproduct_variant_leaf(qualified_name: &str) -> Rc<Node> {
    leaf_node(qualified_name.to_string())
}

fn sample_qualified_coproduct() -> Rc<Node> {
    sum_node(
        "ProbeRefusal",
        vec![
            qualified_coproduct_variant_leaf("fixture.probe.Alpha"),
            qualified_coproduct_variant_leaf("fixture.probe.Beta"),
        ],
        Cardinality::Required,
    )
}

fn coproduct_exhaustiveness_qualified_declared_names_accept_bare_arms() {
    let diags = v1_compiler_infer_patterns::check_match_exhaustiveness(
        sample_qualified_coproduct(),
        Rc::new(vec![variant_arm("Alpha"), variant_arm("Beta")]),
        empty_type_env(),
        zero_span(),
        "test".to_string(),
    );

    assert!(
        diags.is_empty(),
        "bare pattern arms should exhaust qualified declared variants, got {:?}",
        diags
    );
}

fn coproduct_exhaustiveness_missing_arm_names_only_gap() {
    let diags = v1_compiler_infer_patterns::check_match_exhaustiveness(
        sample_qualified_coproduct(),
        Rc::new(vec![variant_arm("Alpha")]),
        empty_type_env(),
        zero_span(),
        "test".to_string(),
    );

    assert_eq!(diags.len(), 1);
    match diags[0].diagnostic.as_ref() {
        CompilerDiagnostic::NonExhaustiveMatch { missing, .. } => {
            assert_eq!(missing.len(), 1, "missing roster must name only the gap");
            assert_eq!(
                missing[0], "Beta",
                "missing must name the bare uncovered variant"
            );
        }
        other => panic!("expected NonExhaustiveMatch, got {:?}", other),
    }
}

fn coproduct_runtime_arm_names_match_on_last_segment_not_type_name_compatible() {
    use v1_compiler::v1_std_core::{qualified_last_segment, type_name_compatible};

    let admission = "product.compute_board.admission.DuplicateComponentIdentity".to_string();
    let witness =
        "test.claim.compute_board_admission_witness.DuplicateComponentIdentity".to_string();

    assert!(
        !type_name_compatible(admission.clone(), witness.clone()),
        "both-qualified different containment paths must not pass type_name_compatible"
    );
    assert_eq!(
        qualified_last_segment(admission.clone()),
        qualified_last_segment(witness.clone()),
        "runtime coproduct arms still share the bare variant identity"
    );
}

fn resolve_node_uses_node_name_for_lookup() {
    let node_ref = Rc::new(Node {
        occurrence_identity: Rc::new(
            v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
        ),
        name: "User".to_string(),
        ident: None,
        span: zero_span(),
        ident_span: Some(Rc::new(v1_compiler::v1_std_core::SourceSpan {
            file: "".to_string(),
            start: 0,
            end: 0,
        })),
        children: Rc::new(vec![]),
        connective: v1_compiler::v1_std_core::Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        module_item_kind: v1_compiler::v1_std_core::ParsedModuleItemKind::NotAModuleItem,
        expr_data: Rc::new(ExprData::NoExprData),
    });
    let user_intern = v1_compiler::v1_std_core::intern(
        v1_compiler::v1_std_core::empty_intern_table(),
        "User".to_string(),
    );
    let user_binding = Rc::new(TypeBinding {
        name: "User".to_string(),
        resolved: leaf_node("User".to_string()),
        provenance: Rc::new(SubValueRelation::SubValueUnknown),
    });
    let env = Rc::new(TypeEnv {
        module_path: "".to_string(),
        authored_import_names: Rc::new(im::HashMap::new()),
        bindings: Rc::new(im::HashMap::from_iter([(
            user_intern.id,
            user_binding.clone(),
        )])),
        str_bindings: Rc::new(im::HashMap::from_iter([("User".to_string(), user_binding)])),
        unit_variant_index: Rc::new(im::HashMap::new()),
        unit_variant_index_observed: false,
        ancestry_str_bindings: Rc::new(im::HashMap::new()),
        parents: Rc::new(vec![]),
        recursive_types: Rc::new(vec![]),
        recursive_type_set: Rc::new(im::HashMap::new()),
        inductive_fields: Rc::new(im::HashMap::new()),
        source_indices: Rc::new(im::HashMap::new()),
        intern_table: user_intern.table.clone(),
        source_visible_names: Rc::new(im::HashMap::new()),
        symbol_index: v1_compiler::v1_compiler_infer_env::empty_symbol_index(),
    });

    let result = resolve_node(node_ref, env, "test".to_string());

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    assert_eq!(result.resolved.name, "User");
}

fn structural_method_lookup_resolves_all_list_collection_methods() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let expected_methods = [
        "map",
        "filter",
        "flat_map",
        "fold",
        "any",
        "all",
        "count",
        "first",
        "last",
        "skip",
        "take",
        "sort_by",
        "append",
        "contains",
        "enumerate",
        "reverse",
        "join",
        "concat",
    ];
    for method_name in &expected_methods {
        assert!(
            v1_compiler_infer_lookup::lookup_structural_method(
                list_int.clone(),
                method_name.to_string(),
                empty_source_indices(),
            )
            .resolution
            .is_some(),
            "lookup_structural_method should resolve '{}' on List<Int>",
            method_name
        );
    }
}

fn structural_method_any_on_list_returns_bool() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let result = v1_compiler_infer_lookup::lookup_structural_method(
        list_int.clone(),
        "any".to_string(),
        empty_source_indices(),
    )
    .resolution
    .as_ref()
    .expect("any must resolve on List<Int>")
    .clone();
    assert_eq!(
        result.result_type.name, "Bool",
        "any on List<Int> should return Bool"
    );
}

fn structural_method_all_on_list_returns_bool() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let result = v1_compiler_infer_lookup::lookup_structural_method(
        list_int.clone(),
        "all".to_string(),
        empty_source_indices(),
    )
    .resolution
    .as_ref()
    .expect("all must resolve on List<Int>")
    .clone();
    assert_eq!(
        result.result_type.name, "Bool",
        "all on List<Int> should return Bool"
    );
}

fn structural_method_sort_by_on_list_returns_self() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let result = v1_compiler_infer_lookup::lookup_structural_method(
        list_int.clone(),
        "sort_by".to_string(),
        empty_source_indices(),
    )
    .resolution
    .as_ref()
    .expect("sort_by must resolve on List<Int>")
    .clone();
    assert_eq!(
        result.result_type.name, "List",
        "sort_by on List<Int> should return List (ReceiverSelf)"
    );
}

fn structural_method_first_on_list_returns_optional_element() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let result = v1_compiler_infer_lookup::lookup_structural_method(
        list_int.clone(),
        "first".to_string(),
        empty_source_indices(),
    )
    .resolution
    .as_ref()
    .expect("first must resolve on List<Int>")
    .clone();
    assert_eq!(result.result_type.name, "Optional");
    assert_eq!(result.result_type.return_cardinality, Cardinality::Required);
    assert_eq!(result.result_type.children.len(), 1);
    assert_eq!(result.result_type.children[0].name, "Int");
}

fn structural_method_count_on_list_returns_int() {
    let list_string = container_node("List".to_string(), leaf_node("String".to_string()));
    let result = v1_compiler_infer_lookup::lookup_structural_method(
        list_string,
        "count".to_string(),
        empty_source_indices(),
    )
    .resolution
    .as_ref()
    .expect("count must resolve on List<String>")
    .clone();
    assert_eq!(result.result_type.name, "Int", "count should return Int");
}

fn structural_method_lookup_resolves_all_int_ring_methods() {
    let int_node = leaf_node("Int".to_string());
    let expected_methods = ["add", "zero", "negate", "mul", "one", "compare"];
    for method_name in &expected_methods {
        assert!(
            v1_compiler_infer_lookup::lookup_structural_method(
                int_node.clone(),
                method_name.to_string(),
                empty_source_indices(),
            )
            .resolution
            .is_some(),
            "lookup_structural_method should resolve '{}' on Int",
            method_name
        );
    }
}

fn structural_method_compare_on_int_returns_ordering() {
    let int_node = leaf_node("Int".to_string());
    let result = v1_compiler_infer_lookup::lookup_structural_method(
        int_node,
        "compare".to_string(),
        empty_source_indices(),
    )
    .resolution
    .as_ref()
    .expect("compare must resolve on Int")
    .clone();
    assert_eq!(
        result.result_type.name, "Ordering",
        "compare on Int should return Ordering"
    );
}

fn structural_method_lookup_resolves_all_map_partial_function_methods() {
    let m = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    let expected_methods = [
        "get",
        "map_get",
        "lookup",
        "map_insert",
        "map_merge",
        "map_has",
        "map_keys",
        "map_values",
        "contains",
        "length",
    ];
    for method_name in &expected_methods {
        assert!(
            v1_compiler_infer_lookup::lookup_structural_method(
                m.clone(),
                method_name.to_string(),
                empty_source_indices(),
            )
            .resolution
            .is_some(),
            "lookup_structural_method should resolve '{}' on Map<String,Int>",
            method_name
        );
    }
}

fn structural_method_get_on_map_returns_optional_value() {
    let m = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    let result = v1_compiler_infer_lookup::lookup_structural_method(
        m.clone(),
        "get".to_string(),
        empty_source_indices(),
    )
    .resolution
    .as_ref()
    .expect("get must resolve on Map<String,Int>")
    .clone();
    assert_eq!(result.result_type.name, "Optional");
    assert_eq!(result.result_type.return_cardinality, Cardinality::Required);
    assert_eq!(result.result_type.children.len(), 1);
    assert_eq!(result.result_type.children[0].name, "Int");
}

fn structural_method_keys_on_map_returns_list_of_key_type() {
    let m = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    let result = v1_compiler_infer_lookup::lookup_structural_method(
        m,
        "map_keys".to_string(),
        empty_source_indices(),
    )
    .resolution
    .as_ref()
    .expect("map_keys must resolve on Map<String,Int>")
    .clone();
    assert_eq!(
        result.result_type.name, "List",
        "map_keys should return List"
    );
    assert_eq!(
        result.result_type.children.len(),
        1,
        "map_keys result should have one child"
    );
    let elem_child = &result.result_type.children[0];
    let elem_type = resolved_type(elem_child.clone());
    assert_eq!(
        elem_type.name, "String",
        "map_keys on Map<String,Int> should return List<String>"
    );
}

fn structural_method_lookup_returns_none_for_unknown_type() {
    let custom = leaf_node("MyType".to_string());
    assert!(
        v1_compiler_infer_lookup::lookup_structural_method(
            custom,
            "add".to_string(),
            empty_source_indices()
        )
        .resolution
        .is_none(),
        "custom types without algebra should not have structural methods"
    );
}

fn keyed_collection_parts_extracts_key_and_value() {
    let m = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    let parts = v1_compiler_infer_access::keyed_collection_parts(m, empty_source_indices());
    let parts = parts.expect("Map<String,Int> should decompose to keyed parts");
    assert_eq!(parts.key_type.name, "String");
    assert_eq!(parts.value_type.name, "Int");
}

fn keyed_collection_parts_returns_none_for_element_collection() {
    let list = container_node("List".to_string(), leaf_node("Int".to_string()));
    let parts = v1_compiler_infer_access::keyed_collection_parts(list, empty_source_indices());
    assert!(
        parts.is_none(),
        "List<Int> is not a keyed collection, should return None"
    );
}

fn keyed_collection_parts_returns_type_variables_for_bare_map() {
    let bare = bare_map_node().expect("Map kernel container profile");
    let parts = v1_compiler_infer_access::keyed_collection_parts(bare, empty_source_indices());
    assert!(
        parts.is_some(),
        "bare Map has K/V children (TypeVariable inferred)"
    );
}

fn node_is_keyed_collection_true_for_map() {
    let m = map_node(
        leaf_node("String".to_string()),
        leaf_node("Bool".to_string()),
    );
    assert!(node_is_keyed_collection(m, empty_source_indices()));
}

fn node_is_keyed_collection_false_for_list() {
    let list = container_node("List".to_string(), leaf_node("Int".to_string()));
    assert!(!node_is_keyed_collection(list, empty_source_indices()));
}

fn node_is_keyed_collection_false_for_leaf() {
    let leaf = leaf_node("String".to_string());
    assert!(!node_is_keyed_collection(leaf, empty_source_indices()));
}

fn is_fully_resolved_rejects_under_parameterized_container() {
    let bare_list = leaf_node("List".to_string());
    assert!(!is_fully_resolved(bare_list, empty_source_indices()));
}

fn is_fully_resolved_accepts_parameterized_container() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    assert!(is_fully_resolved(list_int, empty_source_indices()));
}

fn is_fully_resolved_ignores_unknown_type_names() {
    let widget = leaf_node("Widget".to_string());
    assert!(is_fully_resolved(widget, empty_source_indices()));
}

// ---------------------------------------------------------------------------
// Container-lookup refusal: an unrecognized container spelling is a THIRD state.
//
// `container_expected_arity` returning None used to mean both "not a container"
// and "a container spelling with no row", and `is_fully_resolved` read the second
// as the first — answering "not under-parameterized" about a name whose arity it
// did not know. These fixtures pin the discriminator that separates them, in both
// directions: an AUTHORED qualified container spelling refuses, and the six
// service operations whose last segment is a container name do not.
// ---------------------------------------------------------------------------

const PROBE_FILE: &str = "probe.dag";

/// A source index a name can actually be read back out of. `type_node_name_is_authored`
/// keys on exactly this: a span whose file is present here is authored source, and a
/// synthesized `<kernel:…>` span never is.
fn authored_source_indices(text: &str) -> Rc<HashMap<String, Rc<NewlineIndex>>> {
    let idx = build_newline_index(PROBE_FILE.to_string(), text.to_string());
    Rc::new(im::HashMap::unit(PROBE_FILE.to_string(), idx))
}

/// A leaf type node whose ident span slices `name` out of `text` — i.e. a name a
/// human wrote, as opposed to one the compiler minted.
fn authored_leaf_node(name: &str, text: &str) -> Rc<Node> {
    let start = text
        .find(name)
        .expect("probe text must contain the authored name");
    let start = text[..start].chars().count() as i64;
    let span = Rc::new(SourceSpan {
        file: PROBE_FILE.to_string(),
        start,
        end: start + name.chars().count() as i64,
    });
    leaf_node_with_span(
        Rc::new(v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
        name.to_string(),
        span,
    )
}

fn container_spelling_verdict_declared_for_kernel_container() {
    let text = "data x: List";
    let node = authored_leaf_node("List", text);
    match &*authored_container_spelling_verdict(node, authored_source_indices(text)) {
        ContainerSpellingVerdict::ContainerSpellingDeclared { arity } => assert_eq!(*arity, 1),
        other => panic!("expected ContainerSpellingDeclared for List, got {other:?}"),
    }
}

fn container_spelling_verdict_not_a_container_for_user_type() {
    let text = "data x: Widget";
    let node = authored_leaf_node("Widget", text);
    match &*authored_container_spelling_verdict(node, authored_source_indices(text)) {
        ContainerSpellingVerdict::NotAContainerSpelling => {}
        other => panic!("expected NotAContainerSpelling for Widget, got {other:?}"),
    }
}

fn container_spelling_verdict_unknown_for_qualified_container() {
    let text = "data x: std.types.List";
    let node = authored_leaf_node("std.types.List", text);
    match &*authored_container_spelling_verdict(node, authored_source_indices(text)) {
        ContainerSpellingVerdict::ContainerSpellingUnknown { container_leaf } => {
            assert_eq!(container_leaf, "List")
        }
        other => panic!("expected ContainerSpellingUnknown for std.types.List, got {other:?}"),
    }
}

/// The defect this lane exists for: before the refusal, an authored qualified
/// container spelling reported FULLY RESOLVED, because the missing arity row took
/// the same arm as "String is not a container". It is now refused — and refused
/// distinctly from under-parameterization, which has a different remedy.
fn type_resolution_verdict_refuses_unrecognized_container_spelling() {
    let text = "data x: std.types.List";
    let node = authored_leaf_node("std.types.List", text);
    match &*type_resolution_verdict(node.clone(), authored_source_indices(text)) {
        TypeResolutionVerdict::ContainerSpellingUnresolvable { name } => {
            assert_eq!(name, "std.types.List")
        }
        other => panic!("expected ContainerSpellingUnresolvable, got {other:?}"),
    }
    assert!(
        !is_fully_resolved(node, authored_source_indices(text)),
        "a container spelling whose arity is unknown is not established as resolved"
    );
}

/// The positive control the refusal must not swallow: a declared container that IS
/// parameterized still resolves, so the fixture above measures the refusal and not
/// a walk that stopped answering.
fn type_resolution_verdict_accepts_authored_parameterized_container() {
    let text = "data x: List";
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    match &*type_resolution_verdict(list_int, authored_source_indices(text)) {
        TypeResolutionVerdict::FullyResolved => {}
        other => panic!("expected FullyResolved for List<Int>, got {other:?}"),
    }
}

/// Under-parameterization keeps its own arm: `List` with no argument is not the
/// same finding as `std.types.List`, and collapsing the two would have made the
/// refusal indistinguishable from the diagnostic that already existed.
fn type_resolution_verdict_reports_under_resolved_separately_from_refusal() {
    let text = "data x: List";
    let bare_list = authored_leaf_node("List", text);
    match &*type_resolution_verdict(bare_list, authored_source_indices(text)) {
        TypeResolutionVerdict::UnderResolved => {}
        other => panic!("expected UnderResolved for bare List, got {other:?}"),
    }
}

/// THE FIVE KNOWN FALSE-REFUSALS. Each is a service operation whose last segment is
/// a container name; `check_service_field_access_node` renders one as a LEAF TYPE
/// NODE via `nominal_type_ref`, so each does reach this walk. None is a container,
/// so refusing any of them would be wrong, and the fix a refusal asks for —
/// declare an arity row — would be the wrong action.
///
/// They are excluded by AUTHORSHIP, not by an exception list: a synthesized node
/// carries a `<kernel:…>` span that is in no source index. A sixth service
/// operation named after a container therefore needs no edit here.
///
/// (`os.Hostname.Set` left this list with the CLI-invocation migration: the op's
/// hand-authored transport row was deleted and the hostnamectl set-hostname
/// vertical realizes through hostnamectl.Process.Run, whose last segment is not a
/// container name.)
fn service_operations_named_after_containers_are_not_refused() {
    let known_false_refusals = [
        "cron.Tab.List",
        "Filesystem.List",
        "github.Pulls.List",
        "tmux.Session.List",
        "req.Header.Set",
    ];
    for op in known_false_refusals {
        let node = nominal_type_ref(op.to_string());
        match &*authored_container_spelling_verdict(node.clone(), empty_source_indices()) {
            ContainerSpellingVerdict::NotAContainerSpelling => {}
            other => panic!("service operation {op} must not be classified a container: {other:?}"),
        }
        assert!(
            is_fully_resolved(node, empty_source_indices()),
            "service operation {op} must not be refused as an unknown container spelling"
        );
    }
}

/// The same discriminator read from the other side: the identical spelling, when a
/// human authored it in a type position, IS refused. Without this pair the fixture
/// above would pass for a walk that had simply stopped refusing anything.
fn authored_spelling_refuses_where_synthesized_one_does_not() {
    let text = "data x: cron.Tab.List";
    let authored = authored_leaf_node("cron.Tab.List", text);
    match &*authored_container_spelling_verdict(authored, authored_source_indices(text)) {
        ContainerSpellingVerdict::ContainerSpellingUnknown { container_leaf } => {
            assert_eq!(container_leaf, "List")
        }
        other => panic!("an AUTHORED cron.Tab.List in type position must refuse, got {other:?}"),
    }
}

fn keyed_map_verdict_undecidable_for_unrecognized_container_spelling() {
    let text = "data x: std.types.Map";
    let node = authored_leaf_node("std.types.Map", text);
    match &*v1_type_expr_keyed_map_verdict(node, authored_source_indices(text)) {
        KeyedMapVerdict::KeyedMapUndecidable { name } => assert_eq!(name, "std.types.Map"),
        other => panic!("expected KeyedMapUndecidable for std.types.Map, got {other:?}"),
    }
}

fn keyed_map_verdict_still_decides_the_kernel_map() {
    let text = "data x: Map";
    let m = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    match &*v1_type_expr_keyed_map_verdict(m, authored_source_indices(text)) {
        KeyedMapVerdict::KeyedMap => {}
        other => panic!("expected KeyedMap for kernel Map<String,Int>, got {other:?}"),
    }
}

fn keyed_map_verdict_not_keyed_for_element_collection() {
    let text = "data x: List";
    let list = container_node("List".to_string(), leaf_node("Int".to_string()));
    match &*v1_type_expr_keyed_map_verdict(list, authored_source_indices(text)) {
        KeyedMapVerdict::NotKeyedMap => {}
        other => panic!("expected NotKeyedMap for List<Int>, got {other:?}"),
    }
}

fn map_index_with_correct_key_type_succeeds() {
    let map_type = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    let result = v1_compiler_infer_access::check_index_access_node(
        map_type,
        leaf_node("String".to_string()),
        zero_span(),
        "test".to_string(),
        empty_source_indices(),
    );
    assert!(
        result.diagnostics.is_empty(),
        "Map<String,Int>[String] should succeed, got: {:?}",
        result.diagnostics.len()
    );
    match result.inferred.as_ref().map(|i| i.as_ref()) {
        Some(InferredNode::Resolved { node }) => {
            assert_eq!(node.name, "Optional");
            assert_eq!(node.return_cardinality, Cardinality::Required);
            assert_eq!(node.children.len(), 1);
            assert_eq!(node.children[0].name, "Int");
        }
        other => panic!("expected Resolved(Int?), got {:?}", other),
    }
}

fn map_index_with_wrong_key_type_reports_error() {
    let map_type = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    let result = v1_compiler_infer_access::check_index_access_node(
        map_type,
        leaf_node("Int".to_string()),
        zero_span(),
        "test".to_string(),
        empty_source_indices(),
    );
    assert_eq!(
        result.diagnostics.len(),
        1,
        "Map<String,Int>[Int] should report key type mismatch"
    );
}

fn node_inferred_to_outputs_returns_empty_when_child_has_error() {
    let syn_span = Some(Rc::new(v1_compiler::v1_std_core::SourceSpan {
        file: "".to_string(),
        start: 0,
        end: 0,
    }));
    let typed_child = Rc::new(Node {
        occurrence_identity: Rc::new(
            v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
        ),
        name: "x".to_string(),
        ident_span: syn_span.clone(),
        inferred: Some(Rc::new(InferredNode::Resolved {
            node: leaf_node("Int".to_string()),
        })),
        connective: Connective::NoConnective,
        ..(*leaf_node("".to_string())).clone()
    });
    let error_child = Rc::new(Node {
        occurrence_identity: Rc::new(
            v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
        ),
        name: "y".to_string(),
        ident_span: syn_span.clone(),
        inferred: Some(Rc::new(InferredNode::CompilerError {
            message: "upstream failure".to_string(),
            span: zero_span(),
        })),
        connective: Connective::NoConnective,
        ..(*leaf_node("".to_string())).clone()
    });
    let conj_node = Rc::new(Node {
        occurrence_identity: Rc::new(
            v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
        ),
        name: "Result".to_string(),
        ident_span: syn_span.clone(),
        connective: Connective::Conj,
        children: Rc::new(vec![typed_child, error_child]),
        ..(*leaf_node("".to_string())).clone()
    });

    let outputs = v1_compiler_parse::node_inferred_to_outputs(conj_node, empty_source_indices());
    assert!(
        outputs.is_empty(),
        "fail-closed gate: Conj with error child must produce 0 outputs, got {}",
        outputs.len()
    );
}

fn list_and_freemonoid_compatible_same_element() {
    let list_sym = container_node("List".to_string(), leaf_node("Symbol".to_string()));
    let fm_sym = container_node("FreeMonoid".to_string(), leaf_node("Symbol".to_string()));
    assert!(
        node_type_compatible(list_sym, fm_sym, empty_source_indices()),
        "List<Symbol> and FreeMonoid<Symbol> are declared aliases — must be compatible at type-comparison"
    );
}

fn list_and_freemonoid_incompatible_different_element() {
    let list_int = container_node("List".to_string(), leaf_node("Int".to_string()));
    let fm_string = container_node("FreeMonoid".to_string(), leaf_node("String".to_string()));
    assert!(
        !node_type_compatible(list_int, fm_string, empty_source_indices()),
        "List<Int> vs FreeMonoid<String> differ in element type — must stay incompatible"
    );
}

fn list_freemonoid_compat_is_symmetric() {
    let fm_sym = container_node("FreeMonoid".to_string(), leaf_node("Symbol".to_string()));
    let list_sym = container_node("List".to_string(), leaf_node("Symbol".to_string()));
    assert!(
        node_type_compatible(fm_sym, list_sym, empty_source_indices()),
        "alias compatibility must hold in both argument orders"
    );
}

fn resolve_applied_generic_struct_expands_to_conj_for_field_lookup() {
    use v1_compiler::v1_compiler_infer_lookup::{
        lookup_field_type_node, resolve_scrutinee_type_node,
    };
    use v1_compiler::v1_compiler_infer_resolve::is_user_generic_use_site;
    use v1_compiler::v1_std_core::{empty_intern_table, intern};

    let t_param = leaf_node("T".to_string());
    let value_field = Rc::new(Node {
        occurrence_identity: Rc::new(
            v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
        ),
        name: "value".to_string(),
        ident: None,
        span: no_span(),
        ident_span: default_ident_span("value".to_string(), no_span()),
        children: Rc::new(vec![]),
        connective: Connective::NoConnective,
        params: Rc::new(vec![]),
        inferred: Some(Rc::new(InferredNode::Resolved {
            node: t_param.clone(),
        })),
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        module_item_kind: v1_compiler::v1_std_core::ParsedModuleItemKind::NotAModuleItem,
        expr_data: Rc::new(ExprData::NoExprData),
    });
    let box_decl = Rc::new(Node {
        occurrence_identity: Rc::new(
            v1_compiler::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
        ),
        name: "Box".to_string(),
        ident: None,
        span: no_span(),
        ident_span: default_ident_span("Box".to_string(), no_span()),
        children: Rc::new(vec![value_field]),
        connective: Connective::Conj,
        params: Rc::new(vec![t_param]),
        inferred: None,
        return_cardinality: Cardinality::Required,
        uses: Rc::new(vec![]),
        body: None,
        transport: None,
        properties: Rc::new(vec![]),
        type_annotation: None,
        is_self_recursive: false,
        has_non_tail_self_call: false,
        match_pattern: None,
        module_item_kind: v1_compiler::v1_std_core::ParsedModuleItemKind::NotAModuleItem,
        expr_data: Rc::new(ExprData::NoExprData),
    });
    let box_intern = intern(empty_intern_table(), "Box".to_string());
    let box_binding = Rc::new(TypeBinding {
        name: "Box".to_string(),
        resolved: box_decl.clone(),
        provenance: Rc::new(SubValueRelation::SubValueUnknown),
    });
    let env = Rc::new(TypeEnv {
        module_path: "".to_string(),
        authored_import_names: Rc::new(im::HashMap::new()),
        bindings: Rc::new(im::HashMap::from_iter([(
            box_intern.id,
            box_binding.clone(),
        )])),
        str_bindings: Rc::new(im::HashMap::from_iter([("Box".to_string(), box_binding)])),
        unit_variant_index: Rc::new(im::HashMap::new()),
        unit_variant_index_observed: false,
        ancestry_str_bindings: Rc::new(im::HashMap::new()),
        parents: Rc::new(vec![]),
        recursive_types: Rc::new(vec![]),
        recursive_type_set: Rc::new(im::HashMap::new()),
        inductive_fields: Rc::new(im::HashMap::new()),
        source_indices: empty_source_indices(),
        intern_table: box_intern.table.clone(),
        source_visible_names: Rc::new(im::HashMap::new()),
        symbol_index: v1_compiler::v1_compiler_infer_env::empty_symbol_index(),
    });

    let box_nat = container_node("Box".to_string(), leaf_node("Nat".to_string()));
    assert!(
        is_user_generic_use_site(box_nat.clone(), env.clone()),
        "Box<Nat> should be a generic use site"
    );

    let expanded = resolve_node(box_nat, env.clone(), "test".to_string())
        .resolved
        .clone();
    assert!(
        expanded.inferred.is_some(),
        "expanded applied generic should carry inferred structural target, got {expanded:?}"
    );
    let resolved = resolve_scrutinee_type_node(env.clone(), expanded);
    assert_eq!(
        resolved.connective,
        Connective::Conj,
        "scrutinee expansion should reach Conj, got {resolved:?}"
    );
    let field = lookup_field_type_node(resolved, "value".to_string(), empty_source_indices());
    assert!(
        field.is_some(),
        "field lookup should find value on expanded struct"
    );
}

// THE SCOPE-MAP DISAGREEMENT IS AUTHORABLE HERE EVEN THOUGH IT IS UNWRITABLE IN THE CORPUS.
//
// `locals` and `body_locals` are populated in lockstep by build_params_scope and
// extend_scope_with_params, so no accepted .dag program can put a name in one and not the other.
// That made the earlier claim "no discriminating RED is authorable" look true -- but DESIGN §4b
// says the question is what a FIXTURE can hand the compiler, not what the accepted corpus
// contains, and this harness constructs an InferScope with the two maps as INDEPENDENT fields.
// So the state the sixth constructor exists for is authorable exactly here, one level below the
// source boundary, and the wall is testable rather than decorative.
fn call_target_scope_map_disagreement_is_classified_as_missing_binding() {
    let mut body_locals = im::HashMap::new();
    body_locals.insert("phantom_callee".to_string(), true);
    // locals deliberately left EMPTY -- this is the contradiction under test.
    let scope = Rc::new(InferScope {
        body_locals: Rc::new(body_locals),
        ..(*empty_infer_scope()).clone()
    });

    let outcome = v1_compiler::v1_compiler_infer::call_target_for_direct_call(
        scope,
        "phantom_callee".to_string(),
    );

    // The discriminating assertion: this state must NOT be classified as an ordinary
    // non-declared callee. Before the partition it was indistinguishable from one.
    match &*outcome {
        v1_compiler::v1_compiler_infer_sigs::CallTargetOutcome::LocallyBoundBindingMissing {
            name,
        } => assert_eq!(name, "phantom_callee"),
        other => panic!(
            "a name in body_locals with no binding in locals must classify as \
             LocallyBoundBindingMissing, not {other:?}"
        ),
    }
}

// POSITIVE CONTROL. The same seam, with the two maps AGREEING, must classify as an ordinary
// locally-bound callee -- otherwise the check above would pass by refusing everything.
fn call_target_agreeing_scope_maps_are_locally_bound() {
    let mut body_locals = im::HashMap::new();
    body_locals.insert("real_callee".to_string(), true);
    let mut locals = im::HashMap::new();
    locals.insert(
        "real_callee".to_string(),
        Rc::new(v1_compiler::v1_compiler_infer_env::TypeBinding {
            name: "real_callee".to_string(),
            resolved: leaf_node("Int".to_string()),
            provenance: Rc::new(v1_compiler::std_induction::SubValueRelation::PreservedValue),
        }),
    );
    let scope = Rc::new(InferScope {
        body_locals: Rc::new(body_locals),
        locals: Rc::new(locals),
        ..(*empty_infer_scope()).clone()
    });

    let outcome = v1_compiler::v1_compiler_infer::call_target_for_direct_call(
        scope,
        "real_callee".to_string(),
    );

    match &*outcome {
        v1_compiler::v1_compiler_infer_sigs::CallTargetOutcome::LocallyBoundCallee {
            name, ..
        } => assert_eq!(name, "real_callee"),
        other => panic!("agreeing scope maps must classify as LocallyBoundCallee, not {other:?}"),
    }
}

// Consumer: the Optional replacement migration's resolved-producer boundary.
// Feed authored syntax through compile_to_resolved; a hand-built application
// would not discriminate the old CardOptional result from the root cut.
fn authored_optional_field_resolves_to_instantiated_owner_application() {
    let result =
        compile_dag_resolved("module optional_root.authored\ntype Sample { value: Int? }\n");
    let errors: std::vec::Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| v1_compiler::v1_std_core::is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect();
    assert!(
        errors.is_empty(),
        "authored Optional resolution refused: {errors:?}"
    );
    let graph = result.graph.as_ref().expect("resolved graph");
    let module = graph
        .modules
        .iter()
        .find(|m| {
            v1_compiler::v1_std_core::authored_name_at(
                result.source_indices.clone(),
                m.module.clone(),
            ) == "optional_root.authored"
        })
        .expect("authored module");
    let sample = lookup_type_by_name(module.type_env.clone(), "Sample".to_string())
        .expect("Sample declaration");
    let field = sample
        .children
        .iter()
        .find(|f| f.name == "value")
        .expect("value field");
    let ty = resolved_type(field.clone());
    assert_eq!(
        ty.return_cardinality,
        Cardinality::Required,
        "authored T? must not escape as resolved CardOptional"
    );
    assert_eq!(ty.connective, Connective::NoConnective);
    assert_eq!(
        ty.children.len(),
        1,
        "Optional application has one ordered argument"
    );
    assert_eq!(ty.children[0].name, "Int");
    let owner = match ty.inferred.as_deref() {
        Some(InferredNode::Resolved { node }) => node,
        other => panic!("application needs instantiated owner payload: {other:?}"),
    };
    assert_eq!(owner.connective, Connective::Disj);
    assert_eq!(owner.name, "Optional");
    let present = owner
        .children
        .iter()
        .find(|v| v.name == "Present")
        .expect("Present arm");
    let value = present
        .children
        .iter()
        .find(|f| f.name == "value")
        .expect("Present.value");
    assert_eq!(
        resolved_type(value.clone()).name,
        "Int",
        "payload substitutes the argument"
    );
}

// Consumer: lookup_field_type_node's declared, bounded idempotent lookup.
fn optional_lookup_keeps_one_optional_layer_for_optional_field() {
    let result = compile_dag_resolved(
        "module optional_root.flatten\ntype Inner { count: Int? }\ntype Outer { inner: Inner? }\n",
    );
    assert!(
        result
            .diagnostics
            .iter()
            .all(|d| !v1_compiler::v1_std_core::is_error_diagnostic(d.diagnostic.clone())),
        "lookup source must resolve"
    );
    let module = result
        .graph
        .as_ref()
        .expect("resolved graph")
        .modules
        .iter()
        .find(|m| {
            v1_compiler::v1_std_core::authored_name_at(
                result.source_indices.clone(),
                m.module.clone(),
            ) == "optional_root.flatten"
        })
        .expect("lookup module");
    let outer = lookup_type_by_name(module.type_env.clone(), "Outer".to_string()).expect("Outer");
    let inner = resolved_type(
        outer
            .children
            .iter()
            .find(|f| f.name == "inner")
            .expect("inner field")
            .clone(),
    );
    let found = v1_compiler_infer_lookup::lookup_field_type_node(
        inner,
        "count".to_string(),
        result.source_indices.clone(),
    )
    .expect("count lookup");
    assert_eq!(found.return_cardinality, Cardinality::Required);
    assert_eq!(found.name, "Optional");
    assert_eq!(found.children.len(), 1);
    assert_eq!(
        found.children[0].name, "Int",
        "lookup deliberately flattens only its already-Optional result"
    );
}

// Consumer: resolve_node_bounded's ordinary generic-application arity refusal.
fn authored_optional_wrong_arity_refuses() {
    let result = compile_dag_resolved(
        "module optional_root.arity\ntype Sample { value: Optional<Int, String> }\n",
    );
    assert!(
        result.diagnostics.iter().any(|d| matches!(
            d.diagnostic.as_ref(),
            CompilerDiagnostic::ArityMismatch {
                expected: 1,
                got: 2,
                ..
            }
        )),
        "Optional must refuse two arguments at its single-parameter owner: {:?}",
        result.diagnostics
    );
}

// Consumer: apply_optional_owner's malformed-owner refusal, independent of
// the authored wrong-arity source control. A non-coproduct cannot supply arms.
fn optional_application_rejects_non_coproduct_owner() {
    let mut owner = v1_compiler::v1_compiler_infer_types::kernel_optional_type_node();
    Rc::make_mut(&mut owner).connective = Connective::Conj;
    let result = v1_compiler::v1_compiler_infer_types::apply_optional_owner(
        owner,
        leaf_node("Int".to_string()),
        empty_source_indices(),
        "test".to_string(),
        zero_span(),
    );
    assert!(result.diagnostics.iter().any(|d| matches!(d.diagnostic.as_ref(),
        CompilerDiagnostic::InternalError { message, .. } if message == "Optional owner is not a coproduct"
    )), "malformed owner must refuse at the producer");
}

// Consumer: node_type_compatible must preserve the element-mismatch refusal
// when inference stops producing CardOptional. Shared owner spelling is insufficient.
fn optional_application_rejects_mismatched_element() {
    assert!(
        !node_type_compatible(
            make_optional_type(leaf_node("Int".to_string())),
            make_optional_type(leaf_node("String".to_string())),
            empty_source_indices(),
        ),
        "Optional<Int> and Optional<String> must remain incompatible"
    );
}

// Consumers: both comparison relations' shared positional application rule.
// These pairs exercise the real Optional producer and main's collection slots.
fn application_comparison_checks_all_arguments_and_arity() {
    let optional_int = make_optional_type(leaf_node("Int".to_string()));
    let optional_string = make_optional_type(leaf_node("String".to_string()));
    let map_int = map_node(
        leaf_node("String".to_string()),
        leaf_node("Int".to_string()),
    );
    let map_string = map_node(
        leaf_node("String".to_string()),
        leaf_node("String".to_string()),
    );
    let mut wrong_arity = optional_int.clone();
    Rc::make_mut(&mut Rc::make_mut(&mut wrong_arity).children)
        .push_back(leaf_node("String".to_string()));
    let pairs = [
        (
            "Optional identical",
            optional_int.clone(),
            optional_int.clone(),
            true,
        ),
        (
            "Optional element mismatch",
            optional_int.clone(),
            optional_string.clone(),
            false,
        ),
        (
            "Optional arity mismatch",
            optional_int.clone(),
            wrong_arity,
            false,
        ),
        ("Map identical", map_int.clone(), map_int.clone(), true),
        ("Map value mismatch", map_int, map_string, false),
        (
            "List element mismatch",
            container_node("List".to_string(), leaf_node("Int".to_string())),
            container_node("List".to_string(), leaf_node("String".to_string())),
            false,
        ),
        (
            "nested Optional mismatch",
            make_optional_type(optional_int),
            make_optional_type(optional_string),
            false,
        ),
    ];
    for (name, left, right, expected) in pairs {
        assert_eq!(
            node_type_compatible(left.clone(), right.clone(), empty_source_indices()),
            expected,
            "compatible: {name}"
        );
        assert_eq!(
            v1_compiler::v1_compiler_infer_types::node_type_equals(
                left,
                right,
                empty_source_indices()
            ),
            expected,
            "equals: {name}"
        );
    }
}

// Consumer: resolve_node_bounded's finite recursive-field path must still send
// authored optional syntax through the same ingestion boundary.
fn recursive_generic_optional_field_uses_canonical_application() {
    let result = compile_dag_resolved(
        "module optional_root.recursive\ntype Chain<T> = End | Link { next: Chain<T>? }\n",
    );
    assert!(
        result
            .diagnostics
            .iter()
            .all(|d| !v1_compiler::v1_std_core::is_error_diagnostic(d.diagnostic.clone())),
        "recursive declaration must resolve: {:?}",
        result.diagnostics
    );
    let module = result
        .graph
        .as_ref()
        .expect("resolved graph")
        .modules
        .iter()
        .find(|m| {
            v1_compiler::v1_std_core::authored_name_at(
                result.source_indices.clone(),
                m.module.clone(),
            ) == "optional_root.recursive"
        })
        .expect("recursive module");
    let chain = lookup_type_by_name(module.type_env.clone(), "Chain".to_string()).expect("Chain");
    let link = chain
        .children
        .iter()
        .find(|v| v.name == "Link")
        .expect("Link");
    let next = link
        .children
        .iter()
        .find(|f| f.name == "next")
        .expect("next field");
    let ty = resolved_type(next.clone());
    assert_eq!(ty.return_cardinality, Cardinality::Required);
    assert_eq!(ty.name, "Optional");
    assert_eq!(ty.children.len(), 1);
    assert_eq!(ty.children[0].name, "Chain");
}

// Consumer: the ownership fold must retain the constructor branch's whole-value Read.
// Both authored field orders conservatively clone until per-path last-use evidence exists.
fn branch_read_prevents_record_move_before_projection() {
    for fields in [
        "child: Present { value: item }, text: item.text",
        "text: item.text, child: Present { value: item }",
    ] {
        let source = format!(
            "module ownership_branch_probe\n\
             type Item {{ child: Item? text: String other: String }}\n\
             fn build(item: Item, stop: Bool, choose: Bool) -> Item {{\n\
               if stop {{ return item }}\n\
               if choose {{ Item {{ {fields}, other: \"\" }} }}\n\
               else {{ Item {{ child: item.child, text: item.text, other: item.other }} }}\n\
             }}"
        );
        let result = compile_dag(&source);
        assert_no_diagnostics(&result);
        let proof = result
            .ownership
            .iter()
            .find(|proof| proof.func_name == "build")
            .unwrap();
        let params = ["item".to_string(), "stop".to_string(), "choose".to_string()]
            .into_iter()
            .collect::<im::OrdSet<_>>();
        let movable =
            v1_compiler::v1_compiler_ownership::build_movable_set(proof.clone(), Rc::new(params));
        assert!(
            !movable.contains("item"),
            "branch Read disappeared for {fields}"
        );
    }
}

fn main() -> ExitCode {
    let tests: &[(&str, fn())] = &[
        (
            "branch_read_prevents_record_move_before_projection",
            branch_read_prevents_record_move_before_projection,
        ),
        (
            "recursive_generic_optional_field_uses_canonical_application",
            recursive_generic_optional_field_uses_canonical_application,
        ),
        (
            "application_comparison_checks_all_arguments_and_arity",
            application_comparison_checks_all_arguments_and_arity,
        ),
        (
            "optional_application_rejects_mismatched_element",
            optional_application_rejects_mismatched_element,
        ),
        (
            "optional_application_rejects_non_coproduct_owner",
            optional_application_rejects_non_coproduct_owner,
        ),
        (
            "optional_lookup_keeps_one_optional_layer_for_optional_field",
            optional_lookup_keeps_one_optional_layer_for_optional_field,
        ),
        (
            "authored_optional_wrong_arity_refuses",
            authored_optional_wrong_arity_refuses,
        ),
        (
            "authored_optional_field_resolves_to_instantiated_owner_application",
            authored_optional_field_resolves_to_instantiated_owner_application,
        ),
        (
            "call_target_scope_map_disagreement_is_classified_as_missing_binding",
            call_target_scope_map_disagreement_is_classified_as_missing_binding,
        ),
        (
            "call_target_agreeing_scope_maps_are_locally_bound",
            call_target_agreeing_scope_maps_are_locally_bound,
        ),
        (
            "m1_brand_twins_over_refined_base_remain_distinct_in_infer_representation",
            m1_brand_twins_over_refined_base_remain_distinct_in_infer_representation,
        ),
        (
            "pd3_brand_twins_incompatible_at_node_type_compatible",
            pd3_brand_twins_incompatible_at_node_type_compatible,
        ),
        (
            "pd3_direct_call_rejects_brand_twin_mismatch",
            pd3_direct_call_rejects_brand_twin_mismatch,
        ),
        (
            "pd3_direct_call_accepts_same_brand",
            pd3_direct_call_accepts_same_brand,
        ),
        (
            "pd3_direct_call_accepts_list_for_freemonoid_alias",
            pd3_direct_call_accepts_list_for_freemonoid_alias,
        ),
        (
            "list_int_index_returns_optional_element_type",
            list_int_index_returns_optional_element_type,
        ),
        (
            "malformed_map_index_returns_compiler_error_type",
            malformed_map_index_returns_compiler_error_type,
        ),
        (
            "invalid_slice_returns_compiler_error_type",
            invalid_slice_returns_compiler_error_type,
        ),
        (
            "valid_list_slice_preserves_list_type",
            valid_list_slice_preserves_list_type,
        ),
        (
            "valid_map_index_preserves_optional_value_type",
            valid_map_index_preserves_optional_value_type,
        ),
        (
            "pattern_lookup_blocks_on_infer_error_without_cascade_diagnostic",
            pattern_lookup_blocks_on_infer_error_without_cascade_diagnostic,
        ),
        (
            "pattern_lookup_reports_error_scrutinee_structurally",
            pattern_lookup_reports_error_scrutinee_structurally,
        ),
        (
            "optional_pattern_lookup_rejects_some_variant",
            optional_pattern_lookup_rejects_some_variant,
        ),
        (
            "optional_pattern_lookup_resolves_present_variant",
            optional_pattern_lookup_resolves_present_variant,
        ),
        (
            "optional_pattern_lookup_prefers_optional_present_over_inner_present_variant",
            optional_pattern_lookup_prefers_optional_present_over_inner_present_variant,
        ),
        (
            "optional_present_absent_patterns_keep_canonical_names",
            optional_present_absent_patterns_keep_canonical_names,
        ),
        (
            "optional_applied_generic_lookup_resolves_present_absent_without_disj_children",
            optional_applied_generic_lookup_resolves_present_absent_without_disj_children,
        ),
        (
            "optional_applied_generic_lookup_rejects_wrong_variant_name",
            optional_applied_generic_lookup_rejects_wrong_variant_name,
        ),
        (
            "non_optional_applied_generic_missing_variant_still_fails",
            non_optional_applied_generic_missing_variant_still_fails,
        ),
        (
            "real_optional_coproduct_preserves_present_absent_pattern_names",
            real_optional_coproduct_preserves_present_absent_pattern_names,
        ),
        (
            "optional_match_exhaustiveness_reports_missing_absent",
            optional_match_exhaustiveness_reports_missing_absent,
        ),
        (
            "optional_match_exhaustiveness_rejects_some_and_none",
            optional_match_exhaustiveness_rejects_some_and_none,
        ),
        (
            "optional_match_exhaustiveness_accepts_present_and_absent",
            optional_match_exhaustiveness_accepts_present_and_absent,
        ),
        (
            "coproduct_exhaustiveness_qualified_declared_names_accept_bare_arms",
            coproduct_exhaustiveness_qualified_declared_names_accept_bare_arms,
        ),
        (
            "coproduct_exhaustiveness_missing_arm_names_only_gap",
            coproduct_exhaustiveness_missing_arm_names_only_gap,
        ),
        (
            "coproduct_runtime_arm_names_match_on_last_segment_not_type_name_compatible",
            coproduct_runtime_arm_names_match_on_last_segment_not_type_name_compatible,
        ),
        (
            "resolve_node_uses_node_name_for_lookup",
            resolve_node_uses_node_name_for_lookup,
        ),
        (
            "structural_method_lookup_resolves_all_list_collection_methods",
            structural_method_lookup_resolves_all_list_collection_methods,
        ),
        (
            "structural_method_any_on_list_returns_bool",
            structural_method_any_on_list_returns_bool,
        ),
        (
            "structural_method_all_on_list_returns_bool",
            structural_method_all_on_list_returns_bool,
        ),
        (
            "structural_method_sort_by_on_list_returns_self",
            structural_method_sort_by_on_list_returns_self,
        ),
        (
            "structural_method_first_on_list_returns_optional_element",
            structural_method_first_on_list_returns_optional_element,
        ),
        (
            "structural_method_count_on_list_returns_int",
            structural_method_count_on_list_returns_int,
        ),
        (
            "structural_method_lookup_resolves_all_int_ring_methods",
            structural_method_lookup_resolves_all_int_ring_methods,
        ),
        (
            "structural_method_compare_on_int_returns_ordering",
            structural_method_compare_on_int_returns_ordering,
        ),
        (
            "structural_method_lookup_resolves_all_map_partial_function_methods",
            structural_method_lookup_resolves_all_map_partial_function_methods,
        ),
        (
            "structural_method_get_on_map_returns_optional_value",
            structural_method_get_on_map_returns_optional_value,
        ),
        (
            "structural_method_keys_on_map_returns_list_of_key_type",
            structural_method_keys_on_map_returns_list_of_key_type,
        ),
        (
            "structural_method_lookup_returns_none_for_unknown_type",
            structural_method_lookup_returns_none_for_unknown_type,
        ),
        (
            "keyed_collection_parts_extracts_key_and_value",
            keyed_collection_parts_extracts_key_and_value,
        ),
        (
            "keyed_collection_parts_returns_none_for_element_collection",
            keyed_collection_parts_returns_none_for_element_collection,
        ),
        (
            "keyed_collection_parts_returns_type_variables_for_bare_map",
            keyed_collection_parts_returns_type_variables_for_bare_map,
        ),
        (
            "node_is_keyed_collection_true_for_map",
            node_is_keyed_collection_true_for_map,
        ),
        (
            "node_is_keyed_collection_false_for_list",
            node_is_keyed_collection_false_for_list,
        ),
        (
            "node_is_keyed_collection_false_for_leaf",
            node_is_keyed_collection_false_for_leaf,
        ),
        (
            "is_fully_resolved_rejects_under_parameterized_container",
            is_fully_resolved_rejects_under_parameterized_container,
        ),
        (
            "is_fully_resolved_accepts_parameterized_container",
            is_fully_resolved_accepts_parameterized_container,
        ),
        (
            "is_fully_resolved_ignores_unknown_type_names",
            is_fully_resolved_ignores_unknown_type_names,
        ),
        (
            "container_spelling_verdict_declared_for_kernel_container",
            container_spelling_verdict_declared_for_kernel_container,
        ),
        (
            "container_spelling_verdict_not_a_container_for_user_type",
            container_spelling_verdict_not_a_container_for_user_type,
        ),
        (
            "container_spelling_verdict_unknown_for_qualified_container",
            container_spelling_verdict_unknown_for_qualified_container,
        ),
        (
            "type_resolution_verdict_refuses_unrecognized_container_spelling",
            type_resolution_verdict_refuses_unrecognized_container_spelling,
        ),
        (
            "type_resolution_verdict_accepts_authored_parameterized_container",
            type_resolution_verdict_accepts_authored_parameterized_container,
        ),
        (
            "type_resolution_verdict_reports_under_resolved_separately_from_refusal",
            type_resolution_verdict_reports_under_resolved_separately_from_refusal,
        ),
        (
            "service_operations_named_after_containers_are_not_refused",
            service_operations_named_after_containers_are_not_refused,
        ),
        (
            "authored_spelling_refuses_where_synthesized_one_does_not",
            authored_spelling_refuses_where_synthesized_one_does_not,
        ),
        (
            "keyed_map_verdict_undecidable_for_unrecognized_container_spelling",
            keyed_map_verdict_undecidable_for_unrecognized_container_spelling,
        ),
        (
            "keyed_map_verdict_still_decides_the_kernel_map",
            keyed_map_verdict_still_decides_the_kernel_map,
        ),
        (
            "keyed_map_verdict_not_keyed_for_element_collection",
            keyed_map_verdict_not_keyed_for_element_collection,
        ),
        (
            "map_index_with_correct_key_type_succeeds",
            map_index_with_correct_key_type_succeeds,
        ),
        (
            "map_index_with_wrong_key_type_reports_error",
            map_index_with_wrong_key_type_reports_error,
        ),
        (
            "node_inferred_to_outputs_returns_empty_when_child_has_error",
            node_inferred_to_outputs_returns_empty_when_child_has_error,
        ),
        (
            "list_and_freemonoid_compatible_same_element",
            list_and_freemonoid_compatible_same_element,
        ),
        (
            "list_and_freemonoid_incompatible_different_element",
            list_and_freemonoid_incompatible_different_element,
        ),
        (
            "list_freemonoid_compat_is_symmetric",
            list_freemonoid_compat_is_symmetric,
        ),
        (
            "resolve_applied_generic_struct_expands_to_conj_for_field_lookup",
            resolve_applied_generic_struct_expands_to_conj_for_field_lookup,
        ),
    ];

    for (name, test) in tests {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test));
        if result.is_err() {
            return fail(format!("{name} panicked"));
        }
    }

    ExitCode::SUCCESS
}
