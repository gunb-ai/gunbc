use super::*;
use daglang_resolve::{ModuleGraph, ResolvedModule};
use daglang_syntax::parser;
use daglang_typecheck::typecheck_module_graph;
use gunbc_dag::deps_tool::build_deps_graph;
use gunbc_dag::{build_bootstrap_graph, build_codegen_graph, build_pragma_graph};
use gunbc_ir::node::NodeBody;
use gunbc_ir::{Edge, Port};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

fn typed_project_from_sources(sources: &[(&str, &str)]) -> TypedProject {
    let modules = sources
        .iter()
        .map(|(path, source)| {
            let ast = parser::parse(source).expect("source should parse");
            let module_path = ast
                .module_path
                .as_ref()
                .map(|module| module.node.clone())
                .expect("module declaration is required");
            ResolvedModule {
                path: PathBuf::from(path),
                ast,
                module_path,
                dependencies: Vec::new(),
            }
        })
        .collect();
    typecheck_module_graph(ModuleGraph { modules }).expect("typecheck should succeed")
}

fn callable_stmts_from_source(source: &str) -> Vec<Stmt> {
    let ast = parser::parse(source).expect("source should parse");
    let item = ast.items.first().expect("source should contain one item");
    match &item.node {
        Item::FnDef(def) => def.body.stmts.clone(),
        Item::FuncDef(def) => def.body.stmts.clone(),
        Item::PatternDef(def) => def.body.stmts.clone(),
        other => panic!("expected callable item, got {other:?}"),
    }
}

// Test infrastructure: filesystem access for real DSL corpus fixtures.
#[allow(clippy::disallowed_methods)]
fn typed_project_for_module_with_dependency_closure(module_name: &str) -> TypedProject {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
    let graph =
        ModuleGraph::discover(&[dsl_root]).expect("module graph discovery should succeed");
    let target_index = graph
        .modules
        .iter()
        .position(|module| module.module_path.as_dotted() == module_name)
        .expect("target module should exist in graph");

    let mut required_indices = HashSet::<usize>::new();
    let mut queue = VecDeque::from([target_index]);
    while let Some(module_index) = queue.pop_front() {
        if !required_indices.insert(module_index) {
            continue;
        }
        let Some(module) = graph.modules.get(module_index) else {
            continue;
        };
        for dependency in &module.dependencies {
            queue.push_back(*dependency);
        }
    }

    let mut ordered_indices = required_indices.iter().copied().collect::<Vec<_>>();
    ordered_indices.sort_unstable();
    let index_map = ordered_indices
        .iter()
        .enumerate()
        .map(|(new_index, old_index)| (*old_index, new_index))
        .collect::<HashMap<_, _>>();

    let mut modules = Vec::new();
    for (old_index, mut module) in graph.modules.into_iter().enumerate() {
        if !required_indices.contains(&old_index) {
            continue;
        }
        module.dependencies = module
            .dependencies
            .into_iter()
            .filter_map(|dependency| index_map.get(&dependency).copied())
            .collect::<Vec<_>>();
        modules.push(module);
    }
    typecheck_module_graph(ModuleGraph { modules }).expect("typecheck should succeed")
}

fn lower_target_module(typed: &TypedProject, module_name: &str) -> Dag<LoweredOp> {
    let mut scope = HashSet::new();
    scope.insert(module_name.to_string());
    lower_typed_project_for_modules(typed, &scope).expect("lowering should succeed")
}

fn lower_target_module_with_dependency_scope(
    typed: &TypedProject,
    module_name: &str,
) -> Dag<LoweredOp> {
    let module_lookup = typed
        .modules
        .iter()
        .enumerate()
        .map(|(index, module)| (module.module_path.as_dotted(), index))
        .collect::<HashMap<_, _>>();
    let target_index = typed
        .modules
        .iter()
        .position(|module| module.module_path.as_dotted() == module_name)
        .expect("target module should exist in typed project");
    let mut scope = HashSet::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::from([target_index]);
    while let Some(module_index) = queue.pop_front() {
        if !visited.insert(module_index) {
            continue;
        }
        let Some(module) = typed.modules.get(module_index) else {
            continue;
        };
        scope.insert(module.module_path.as_dotted());
        for import in &module.imports {
            let import_name = import.as_dotted();
            if let Some(import_index) = module_lookup.get(&import_name) {
                queue.push_back(*import_index);
            }
        }
    }
    lower_typed_project_for_modules(typed, &scope).expect("lowering should succeed")
}

#[test]
fn collect_collection_ops_detects_pipe_map_filter_join_chain() {
    let stmts = callable_stmts_from_source(
        r#"
module sample.collections
fn run(values: List<String>) -> { out: String } {
  rendered = values
|> map(v => v)
|> filter(v => v != "")
|> join(",")
  return { out: rendered }
}
"#,
    );
    let mut sites = Vec::new();
    collect_collection_ops_from_stmts(&stmts, &mut sites);
    let kinds = sites.iter().map(|site| site.kind).collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            CollectionOpKind::Join,
            CollectionOpKind::Filter,
            CollectionOpKind::Map,
        ]
    );
}

#[test]
fn collect_collection_ops_detects_pipe_flat_map_chain() {
    let stmts = callable_stmts_from_source(
        r#"
module sample.collections
fn run(values: List<String>) -> { out: String } {
  rendered = values
|> flat_map(v => [v])
|> join(",")
  return { out: rendered }
}
"#,
    );
    let mut sites = Vec::new();
    collect_collection_ops_from_stmts(&stmts, &mut sites);
    let kinds = sites.iter().map(|site| site.kind).collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![CollectionOpKind::Join, CollectionOpKind::FlatMap]
    );
}

#[test]
fn collect_collection_ops_ignores_non_pipe_collection_calls() {
    let stmts = callable_stmts_from_source(
        r#"
module sample.collections
fn run(values: List<String>) -> { out: String } {
  rendered = map(values, v => v)
  return { out: rendered }
}
"#,
    );
    let mut sites = Vec::new();
    collect_collection_ops_from_stmts(&stmts, &mut sites);
    assert!(sites.is_empty());
}

#[test]
fn collect_collection_ops_detects_extended_collection_intrinsics() {
    let stmts = callable_stmts_from_source(
        r#"
module sample.collections
fn run(values: List<String>) -> { out: Int } {
  evaluated = values
|> sort()
|> dedup()
|> contains("needle")
|> len()
  return { out: evaluated }
}
"#,
    );
    let mut sites = Vec::new();
    collect_collection_ops_from_stmts(&stmts, &mut sites);
    let kinds = sites.iter().map(|site| site.kind).collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            CollectionOpKind::Len,
            CollectionOpKind::Contains,
            CollectionOpKind::Dedup,
            CollectionOpKind::Sort,
        ]
    );
}

#[test]
fn derive_collection_node_specs_orders_pipeline_left_to_right() {
    let stmts = callable_stmts_from_source(
        r#"
module sample.collections
fn run(values: List<String>) -> { out: String } {
  rendered = values
|> map(v => v)
|> filter(v => v != "")
|> join(",")
  return { out: rendered }
}
"#,
    );
    let specs = derive_collection_node_specs("sample.collections::run", &stmts);
    assert_eq!(
        specs,
        vec![
            CollectionNodeSpec {
                node_id: "sample.collections::run::MapNode_0".to_string(),
                kind: CollectionOpKind::Map,
            },
            CollectionNodeSpec {
                node_id: "sample.collections::run::FilterNode_1".to_string(),
                kind: CollectionOpKind::Filter,
            },
            CollectionNodeSpec {
                node_id: "sample.collections::run::JoinNode_2".to_string(),
                kind: CollectionOpKind::Join,
            },
        ]
    );
}

#[test]
fn build_collection_lowering_plan_chains_nodes_and_wires_target_dependency() {
    let specs = vec![
        CollectionNodeSpec {
            node_id: "sample.collections::run::MapNode_0".to_string(),
            kind: CollectionOpKind::Map,
        },
        CollectionNodeSpec {
            node_id: "sample.collections::run::FilterNode_1".to_string(),
            kind: CollectionOpKind::Filter,
        },
        CollectionNodeSpec {
            node_id: "sample.collections::run::JoinNode_2".to_string(),
            kind: CollectionOpKind::Join,
        },
    ];
    let plan =
        build_collection_lowering_plan("sample.collections", "sample.collections::run", &specs);
    assert_eq!(plan.nodes.len(), 3);
    assert_eq!(plan.edges.len(), 3);
    let kinds = plan
        .nodes
        .iter()
        .map(|node| match &node.body {
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Collection { kind, .. }) => *kind,
            _ => panic!("expected collection lowered op"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            CollectionOpKind::Map,
            CollectionOpKind::Filter,
            CollectionOpKind::Join,
        ]
    );
    assert_eq!(
        plan.edges,
        vec![
            (
                "sample.collections::run::MapNode_0".to_string(),
                "items".to_string(),
                "sample.collections::run::FilterNode_1".to_string(),
                "items".to_string(),
            ),
            (
                "sample.collections::run::FilterNode_1".to_string(),
                "items".to_string(),
                "sample.collections::run::JoinNode_2".to_string(),
                "items".to_string(),
            ),
            (
                "sample.collections::run::JoinNode_2".to_string(),
                "items".to_string(),
                "sample.collections::run".to_string(),
                PortName::DEPS.to_string(),
            ),
        ]
    );
}

#[test]
fn lower_typed_project_emits_collection_nodes_for_pipe_chain() {
    let typed = typed_project_from_sources(&[(
        "sample.collections",
        r#"
module sample.collections

fn run(values: List<String>) -> String {
  rendered = values
|> map(v => v)
|> filter(v => v != "")
|> join(",")
  return rendered
}
"#,
    )]);
    let dag =
        lower_typed_project_with_collection_nodes(&typed).expect("lowering should succeed");
    let node_ids = dag
        .nodes
        .iter()
        .map(|node| node.id.0.clone())
        .collect::<HashSet<_>>();
    assert!(node_ids.contains("sample.collections::run::MapNode_0"));
    assert!(node_ids.contains("sample.collections::run::FilterNode_1"));
    assert!(node_ids.contains("sample.collections::run::JoinNode_2"));
    let mut collection_kinds = dag
        .nodes
        .iter()
        .filter_map(|node| match &node.body {
            gunbc_ir::node::NodeBody::Opaque(LoweredOp::Collection { kind, .. }) => Some(*kind),
            _ => None,
        })
        .collect::<Vec<_>>();
    collection_kinds.sort_by_key(|kind| match kind {
        CollectionOpKind::Map => 0,
        CollectionOpKind::Filter => 1,
        CollectionOpKind::Fold => 2,
        CollectionOpKind::Join => 3,
        CollectionOpKind::FlatMap => 4,
        CollectionOpKind::Sort => 5,
        CollectionOpKind::Dedup => 6,
        CollectionOpKind::Any => 7,
        CollectionOpKind::All => 8,
        CollectionOpKind::Len => 9,
        CollectionOpKind::Contains => 10,
        CollectionOpKind::Split => 11,
        CollectionOpKind::Zip => 12,
    });
    assert_eq!(
        collection_kinds,
        vec![
            CollectionOpKind::Map,
            CollectionOpKind::Filter,
            CollectionOpKind::Join,
        ]
    );

    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "sample.collections::run::MapNode_0"
            && edge.from_port.0 == "items"
            && edge.to_node.0 == "sample.collections::run::FilterNode_1"
            && edge.to_port.0 == "items"
    }));
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "sample.collections::run::FilterNode_1"
            && edge.from_port.0 == "items"
            && edge.to_node.0 == "sample.collections::run::JoinNode_2"
            && edge.to_port.0 == "items"
    }));
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "sample.collections::run::JoinNode_2"
            && edge.from_port.0 == "items"
            && edge.to_node.0 == "sample.collections::run"
            && edge.to_port.0 == PortName::DEPS
    }));
}

#[test]
fn lower_typed_project_emits_control_flow_nodes_for_if_for_match() {
    let typed = typed_project_from_sources(&[(
        "sample.control",
        r#"
module sample.control

fn run(values: List<Int>, gate: Bool, mode: String) -> Int {
  let iterated = for value in values {
if gate { value } else { 0 }
  }
  let chosen = match mode {
"hot" => 1
_ => 0
  }
  let final = if gate { chosen } else { 0 }
  final
}
"#,
    )]);
    let fn_body = typed
        .modules
        .first()
        .and_then(|module| module.ast.items.first())
        .and_then(|item| match &item.node {
            Item::FnDef(def) => Some(&def.body),
            _ => None,
        })
        .expect("sample source should contain a fn body");
    assert!(
        !fn_body.lossy,
        "expected non-lossy parsed function body for control-flow fixture"
    );
    assert!(
        !fn_body.stmts.is_empty(),
        "expected control-flow fixture to retain parsed statements"
    );
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let node_ids = dag
        .nodes
        .iter()
        .map(|node| node.id.0.as_str())
        .collect::<HashSet<_>>();
    assert!(
        node_ids.contains("sample.control::run::cf_for_0"),
        "node ids: {node_ids:?}"
    );
    assert!(
        node_ids.contains("sample.control::run::cf_if_0"),
        "node ids: {node_ids:?}"
    );
    assert!(
        node_ids.contains("sample.control::run::cf_if_1"),
        "node ids: {node_ids:?}"
    );
    assert!(
        node_ids.contains("sample.control::run::cf_match_0"),
        "node ids: {node_ids:?}"
    );
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "sample.control::run::cf_for_0"
            && edge.to_node.0 == "sample.control::run"
            && edge.to_port.0 == PortName::DEPS
    }));
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "sample.control::run::cf_match_0"
            && edge.to_node.0 == "sample.control::run"
            && edge.to_port.0 == PortName::DEPS
    }));
}

#[test]
fn expr_to_template_string_preserves_identifier_interpolation() {
    let expr = Expr::StringInterp(vec![
        daglang_syntax::ast::StringPart::Literal("prefix-".to_string()),
        daglang_syntax::ast::StringPart::Expr(Expr::Ident("left".to_string())),
        daglang_syntax::ast::StringPart::Literal("-".to_string()),
        daglang_syntax::ast::StringPart::Expr(Expr::Ident("right".to_string())),
    ]);
    assert_eq!(
        expr_to_template_string(&expr),
        Some("prefix-{left}-{right}".to_string())
    );
}

#[test]
fn deps_parity_report_is_deterministic() {
    let typed = typed_project_for_module_with_dependency_closure("tools.deps");
    let dag = lower_target_module_with_dependency_scope(&typed, "tools.deps");
    let reference = build_deps_graph().expect("deps builder graph should be available");

    let report_a = compare_ir(&dag, &reference);
    let report_b = compare_ir(&dag, &reference);
    assert_eq!(report_a, report_b);
    assert!(report_a.candidate_nodes > 0);
    assert!(report_a.reference_nodes > 0);
}

#[test]
fn bootstrap_parity_report_is_deterministic() {
    let typed = typed_project_for_module_with_dependency_closure("tools.bootstrap");
    let dag = lower_target_module_with_dependency_scope(&typed, "tools.bootstrap");
    let reference =
        build_bootstrap_graph().expect("bootstrap builder graph should be available");

    let report_a = compare_ir(&dag, &reference);
    let report_b = compare_ir(&dag, &reference);
    assert_eq!(report_a, report_b);
    assert!(report_a.candidate_nodes > 0);
    assert!(report_a.reference_nodes > 0);
}

#[test]
fn codegen_parity_report_is_deterministic() {
    let typed = typed_project_for_module_with_dependency_closure("tools.codegen");
    let dag = lower_target_module_with_dependency_scope(&typed, "tools.codegen");
    let reference = build_codegen_graph().expect("codegen builder graph should be available");

    let report_a = compare_ir(&dag, &reference);
    let report_b = compare_ir(&dag, &reference);
    assert_eq!(report_a, report_b);
    assert!(report_a.candidate_nodes > 0);
    assert!(report_a.reference_nodes > 0);
}

#[test]
fn pragma_parity_report_is_deterministic() {
    let typed = typed_project_for_module_with_dependency_closure("tools.pragma");
    let dag = lower_target_module_with_dependency_scope(&typed, "tools.pragma");
    let reference = build_pragma_graph().expect("pragma builder graph should be available");

    let report_a = compare_ir(&dag, &reference);
    let report_b = compare_ir(&dag, &reference);
    assert_eq!(report_a, report_b);
    assert!(report_a.candidate_nodes > 0);
    assert!(report_a.reference_nodes > 0);
}

#[test]
fn interface_backed_service_lowers_transport_triplet_nodes() {
    let typed = typed_project_from_sources(&[(
        "dsl/services/storage.dag",
        r#"module sample.services
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let node_ids = dag
        .nodes
        .iter()
        .map(|node| node.id.0.as_str())
        .collect::<Vec<_>>();
    let suffix = "sample_services_FsStorage_read";
    let prepare = format!("prepare_transport_{suffix}");
    let execute = format!("execute_transport_{suffix}");
    let parse = format!("parse_transport_{suffix}");

    assert!(node_ids.contains(&prepare.as_str()));
    assert!(node_ids.contains(&execute.as_str()));
    assert!(node_ids.contains(&parse.as_str()));
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == prepare.as_str()
            && edge.from_port.0 == "request"
            && edge.to_node.0 == execute.as_str()
            && edge.to_port.0 == "request"
    }));
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == execute.as_str()
            && edge.from_port.0 == "response"
            && edge.to_node.0 == parse.as_str()
            && edge.to_port.0 == "response"
    }));
}

#[test]
fn concrete_service_without_interface_lowers_transport_triplet_nodes() {
    let typed = typed_project_from_sources(&[(
        "dsl/services/shell.dag",
        r#"module sample.services
service shell.Tools {
  operation Echo(message: String) -> { output: String } {
transport shell { argv: ["echo", "{message}"] }
  }
}
func run() -> { output: String } {
  result = shell.Tools.Echo(message: "hello")
  return { output: result.output }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let node_ids = dag
        .nodes
        .iter()
        .map(|node| node.id.0.as_str())
        .collect::<Vec<_>>();
    let suffix = "sample_services_shell_Tools_Echo";
    let prepare = format!("prepare_transport_{suffix}");
    let execute = format!("execute_transport_{suffix}");
    let parse = format!("parse_transport_{suffix}");

    assert!(node_ids.contains(&prepare.as_str()));
    assert!(node_ids.contains(&execute.as_str()));
    assert!(node_ids.contains(&parse.as_str()));
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == parse.as_str()
            && edge.to_node.0 == "sample.services::run"
            && edge.to_port.0 == PortName::DEPS
    }));
}

#[test]
fn service_transport_metadata_preserves_operation_annotations() {
    let typed = typed_project_from_sources(&[(
        "dsl/services/metadata.dag",
        r#"module sample.services
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
service RemoteStorage implements Storage {
  operation read(path: String) -> { body: String } idempotent {
transport rest { method: GET, path: "/read" }
  }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let execute_node = dag
        .nodes
        .iter()
        .find(|node| node.id.0 == "execute_transport_sample_services_RemoteStorage_read")
        .expect("execute transport node should exist");
    let metadata = match &execute_node.body {
        gunbc_ir::node::NodeBody::Opaque(op) => op
            .service_call_metadata()
            .expect("service metadata should be preserved"),
        gunbc_ir::node::NodeBody::SubDag(_) => {
            panic!("expected opaque lowered node for execute transport")
        }
    };
    assert_eq!(metadata.service, "RemoteStorage");
    assert_eq!(metadata.operation, "read");
    assert_eq!(metadata.transport, ServiceTransportClass::RestNetwork);
    assert!(metadata.idempotent);
    assert!(metadata.readonly, "REST GET should auto-derive readonly");
    // Note: permissions not yet expressible in typed syntax (parser gap).
    assert!(metadata.permissions.is_empty());
}

#[test]
fn service_transport_metadata_uses_service_level_annotations_as_fallback() {
    let typed = typed_project_from_sources(&[(
        "dsl/services/service_annotations.dag",
        r#"module sample.services
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String } readonly {
transport shell { argv: ["cat", "{path}"] }
  }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let prepare_node = dag
        .nodes
        .iter()
        .find(|node| node.id.0 == "prepare_transport_sample_services_FsStorage_read")
        .expect("prepare transport node should exist");
    let (transport_class, readonly) = match &prepare_node.body {
        gunbc_ir::node::NodeBody::Opaque(op) => (
            classify_service_transport(op).expect("service transport class should be present"),
            op.service_call_metadata()
                .expect("service metadata should be present")
                .readonly,
        ),
        gunbc_ir::node::NodeBody::SubDag(_) => {
            panic!("expected opaque lowered node for prepare transport")
        }
    };
    assert_eq!(transport_class, ServiceTransportClass::ShellLocal);
    assert!(
        readonly,
        "service-level readonly annotation should be preserved"
    );
}

fn shell_output_parsing_for_node(dag: &Dag<LoweredOp>, node_id: &str) -> ShellOutputParsing {
    let node = dag
        .nodes
        .iter()
        .find(|node| node.id.0 == node_id)
        .expect("transport node should exist");
    let metadata = match &node.body {
        gunbc_ir::node::NodeBody::Opaque(op) => op
            .service_call_metadata()
            .expect("service metadata should be present"),
        gunbc_ir::node::NodeBody::SubDag(_) => {
            panic!("expected opaque lowered node for transport metadata")
        }
    };
    let spec = metadata
        .spec
        .as_ref()
        .expect("service metadata should include operation spec");
    match spec {
        ServiceOperationSpec::Shell(spec) => spec.output_parsing,
        other => panic!("expected shell operation spec, got {other:?}"),
    }
}

#[test]
fn shell_explicit_parse_mode_overrides_inferred() {
    // After annotation removal, parse mode is inferred from output types.
    // List<String> output infers SplitLines.
    let typed = typed_project_from_sources(&[(
        "dsl/services/shell_parse_override.dag",
        r#"module sample.services
service shell.Tools {
  operation Echo(value: String) -> { lines: List<String> } {
transport shell { argv: ["echo", "{value}"] }
  }
}
func run() -> { lines: List<String> } {
  result = shell.Tools.Echo(value: "hello")
  return { lines: result.lines }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let parsing = shell_output_parsing_for_node(
        &dag,
        "execute_transport_sample_services_shell_Tools_Echo",
    );
    assert_eq!(parsing, ShellOutputParsing::SplitLines);
}

#[test]
fn shell_explicit_parse_mode_prefers_operation_over_service() {
    // After annotation removal, parse mode is inferred from output types.
    // Single Bool output infers ExitCodeBool.
    let typed = typed_project_from_sources(&[(
        "dsl/services/shell_parse_precedence.dag",
        r#"module sample.services
service shell.Tools {
  operation Echo(value: String, suffix: String) -> { ok: Bool } {
transport shell { argv: ["echo", "{value}:{suffix}"] }
  }
}
func run() -> { ok: Bool } {
  result = shell.Tools.Echo(value: "a", suffix: "b")
  return { ok: result.ok }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let parsing = shell_output_parsing_for_node(
        &dag,
        "execute_transport_sample_services_shell_Tools_Echo",
    );
    assert_eq!(parsing, ShellOutputParsing::ExitCodeBool);
}

#[test]
fn shell_argv_annotation_supports_string_interpolation_templates() {
    let typed = typed_project_from_sources(&[(
        "dsl/services/shell_argv_templates.dag",
        r#"module sample.services
service shell.Tools {
  operation Echo(value: String, suffix: String) -> { out: String } {
transport shell { argv: ["echo", "{value}:{suffix}", "{value}"] }
  }
}
func run() -> { out: String } {
  result = shell.Tools.Echo(value: "alpha", suffix: "beta")
  return { out: result.out }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let node = dag
        .nodes
        .iter()
        .find(|node| node.id.0 == "prepare_transport_sample_services_shell_Tools_Echo")
        .expect("prepare node should exist");
    let metadata = match &node.body {
        gunbc_ir::node::NodeBody::Opaque(op) => op
            .service_call_metadata()
            .expect("service metadata should be present"),
        gunbc_ir::node::NodeBody::SubDag(_) => {
            panic!("expected opaque lowered node for prepare transport")
        }
    };
    let spec = metadata
        .spec
        .as_ref()
        .expect("service metadata should include operation spec");
    let shell = match spec {
        ServiceOperationSpec::Shell(shell) => shell,
        other => panic!("expected shell operation spec, got {other:?}"),
    };
    assert_eq!(
        shell.argv_template[0],
        ArgvSegment::Literal("echo".to_string())
    );
    assert_eq!(
        shell.argv_template[1],
        ArgvSegment::Literal("{value}:{suffix}".to_string())
    );
    assert_eq!(
        shell.argv_template[2],
        ArgvSegment::InputRef("value".to_string())
    );
}

#[test]
fn service_calls_link_parse_triplet_output_into_caller_dependencies() {
    let typed = typed_project_from_sources(&[(
        "dsl/services/storage_calls.dag",
        r#"module sample.services
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path: path)
  return { body: response.body }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let parse_node = "parse_transport_sample_services_FsStorage_read";
    let prepare_node = "prepare_transport_sample_services_FsStorage_read";
    let param_source = "param_source_sample_services_run_path";
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == parse_node
            && edge.to_node.0 == "sample.services::run"
            && edge.to_port.0 == PortName::DEPS
    }));
    assert!(dag.nodes.iter().any(|node| node.id.0 == param_source));
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == param_source
            && edge.from_port.0 == "path"
            && edge.to_node.0 == prepare_node
            && edge.to_port.0 == "path"
    }));
}

#[test]
fn resource_bound_capability_calls_do_not_raise_unresolved_service_call_errors() {
    let typed = typed_project_from_sources(&[(
        "dsl/resources/resource_capability_calls.dag",
        r#"module sample.resources
resource Filesystem {
  capability read {
input { path: String }
output { body: String }
  }
}
func run(path: String) -> { body: String } uses fs: Filesystem {
  let response = fs.read(path: path)
  return { body: response.body }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    assert!(dag
        .nodes
        .iter()
        .any(|node| node.id.0 == "sample.resources::run"));
}

#[test]
fn unresolved_service_call_reports_lower_error() {
    let typed = typed_project_from_sources(&[(
        "dsl/services/unresolved_call.dag",
        r#"module sample.services
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = MissingStorage.read(path: path)
  return { body: response.body }
}"#,
    )]);
    let error = lower_typed_project(&typed).expect_err("lowering should fail");
    assert!(matches!(
        error,
        LowerError::UnresolvedServiceCall { caller, service_call }
            if caller == "sample.services::run" && service_call == "MissingStorage.read"
    ));
}

#[test]
fn positional_service_call_args_wire_by_operation_input_order() {
    let typed = typed_project_from_sources(&[(
        "dsl/services/storage_calls_positional.dag",
        r#"module sample.services
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run(path: String) -> { body: String } {
  let response = FsStorage.read(path)
  return { body: response.body }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "param_source_sample_services_run_path"
            && edge.from_port.0 == "path"
            && edge.to_node.0 == "prepare_transport_sample_services_FsStorage_read"
            && edge.to_port.0 == "path"
    }));
}

#[test]
fn literal_service_call_args_wire_to_prepare_inputs() {
    let typed = typed_project_from_sources(&[(
        "dsl/services/storage_calls_literal.dag",
        r#"module sample.services
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func run() -> { body: String } {
  let response = FsStorage.read(path: "crates")
  return { body: response.body }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let literal_node = dag
        .nodes
        .iter()
        .find(|node| {
            matches!(
                &node.body,
                gunbc_ir::node::NodeBody::Opaque(LoweredOp::Primitive {
                    kind: PrimitiveOpKind::CallLiteralSource {
                        literal: PrimitiveLiteral::String(value)
                    },
                    ..
                }) if value == "crates"
            )
        })
        .expect("literal source node should be present");
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node == literal_node.id
            && edge.from_port.0 == "path"
            && edge.to_node.0 == "prepare_transport_sample_services_FsStorage_read"
            && edge.to_port.0 == "path"
    }));
}

#[test]
fn field_access_service_call_args_wire_to_prepare_inputs() {
    let typed = typed_project_from_sources(&[(
        "dsl/services/storage_calls_field_access.dag",
        r#"module sample.services
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
service FsStorage implements Storage {
  operation read(path: String) -> { body: String }
}
func make_path() -> { path: String } {
  return { path: "crates" }
}
func run() -> { body: String } {
  let req = make_path()
  let response = FsStorage.read(path: req.path)
  return { body: response.body }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "sample.services::make_path"
            && edge.from_port.0 == "path"
            && edge.to_node.0 == "prepare_transport_sample_services_FsStorage_read"
            && edge.to_port.0 == "path"
    }));
}

#[test]
/// IS-3: Compilation without --profile now succeeds with stub interfaces.
fn interface_bound_service_call_requires_active_profile() {
    let typed = typed_project_from_sources(&[(
        "dsl/profiles/interface_binding.dag",
        r#"module sample.profile
interface IssueProvider {
  capability get {
input {}
output { ok: Bool }
  }
}
service impl.Provider : IssueProvider {
  operation get {
input {}
output { ok: Bool }
transport rest { method: GET, path: "/ok" }
  }
}
profile unit_test {
  bind IssueProvider -> impl.Provider
}
func run() -> { ok: Bool } uses issues: IssueProvider {
  result = issues.get()
  return { ok: result.ok }
}"#,
    )]);

    // IS-3: No-profile compilation succeeds with stub transport.
    let dag = lower_typed_project(&typed)
        .expect("lowering without profile should succeed with stub interfaces (IS-3)");
    assert!(
        !dag.nodes.is_empty(),
        "lowered DAG should contain nodes with stub transport"
    );
}

#[test]
fn interface_bound_service_call_resolves_via_active_profile_binding() {
    let typed = typed_project_from_sources(&[(
        "dsl/profiles/interface_binding.dag",
        r#"module sample.profile
interface IssueProvider {
  capability get {
input {}
output { ok: Bool }
  }
}
service impl.Provider : IssueProvider {
  operation get {
input {}
output { ok: Bool }
transport rest { method: GET, path: "/ok" }
  }
}
profile unit_test {
  bind IssueProvider -> impl.Provider
}
func run() -> { ok: Bool } uses issues: IssueProvider {
  result = issues.get()
  return { ok: result.ok }
}"#,
    )]);

    let dag = lower_typed_project_with_profile(&typed, Some("unit_test"))
        .expect("lowering should succeed with active profile");
    assert!(
        dag.nodes.iter().any(|node| {
            node.id
                .0
                .starts_with("parse_transport_sample_profile_impl_Provider_get")
        }),
        "profile binding should add transport triplet nodes for bound implementation"
    );
    assert!(
        dag.edges.iter().any(|edge| {
            edge.from_node
                .0
                .starts_with("parse_transport_sample_profile_impl_Provider_get")
                && edge.to_node.0 == "sample.profile::run"
                && edge.to_port.0 == PortName::DEPS
        }),
        "bound parse node should feed caller dependencies"
    );
}

#[test]
fn active_profile_env_config_requires_present_environment_variable() {
    let typed = typed_project_from_sources(&[(
        "dsl/profiles/interface_env_binding.dag",
        r#"module sample.profile
interface IssueProvider {
  capability get {
input {}
output { ok: Bool }
  }
}
service impl.Provider : IssueProvider {
  operation get {
input {}
output { ok: Bool }
transport rest { method: GET, path: "/ok" }
  }
}
profile local {
  bind IssueProvider -> impl.Provider {
credential: env("DAGLANG_TEST_PROFILE_ENV_MISSING_9F8C")
  }
}
func run() -> { ok: Bool } uses issues: IssueProvider {
  result = issues.get()
  return { ok: result.ok }
}"#,
    )]);

    let error = lower_typed_project_with_profile(&typed, Some("local"))
        .expect_err("lowering should fail when env profile config is unset");
    assert!(matches!(
        error,
        LowerError::MissingProfileConfigEnv {
            profile,
            interface_type,
            key,
            env_var,
        } if profile == "sample.profile.local"
            && interface_type == "sample.profile.IssueProvider"
            && key == "credential"
            && env_var == "DAGLANG_TEST_PROFILE_ENV_MISSING_9F8C"
    ));
}

#[test]
fn active_profile_secret_config_is_accepted_for_active_profile() {
    let typed = typed_project_from_sources(&[(
        "dsl/profiles/interface_secret_binding.dag",
        r#"module sample.profile
interface IssueProvider {
  capability get {
input { id: String }
output { ok: Bool }
  }
}
service impl.Provider : IssueProvider {
  config { endpoint: "https://api.github.com", auth: BearerToken }
  operation get {
input { id: String }
output { ok: Bool }
transport rest { method: GET, path: "/repos/\{config.owner\}/\{config.repo\}/issues/\{id\}" }
  }
}
profile unit_test {
  bind IssueProvider -> impl.Provider {
owner: "gunb-ai"
repo: "gunbc"
credential: secret("github-token")
  }
}
func run(id: String) -> { ok: Bool } uses issues: IssueProvider {
  result = issues.get(id: id)
  return { ok: result.ok }
}"#,
    )]);

    let dag = lower_typed_project_with_profile(&typed, Some("unit_test"))
        .expect("lowering should succeed with secret profile config binding");
    let prepare_node_id = "prepare_transport_sample_profile_impl_Provider_get";
    let execute_node_id = "execute_transport_sample_profile_impl_Provider_get";
    assert!(dag
        .edges
        .iter()
        .any(|edge| edge.to_node.0 == prepare_node_id && edge.to_port.0 == "config.owner"));
    assert!(dag
        .edges
        .iter()
        .any(|edge| edge.to_node.0 == prepare_node_id && edge.to_port.0 == "config.repo"));
    assert!(
        dag.edges.iter().any(|edge| {
            edge.to_node.0 == execute_node_id && edge.to_port.0 == PortName::RESOURCE_CREDENTIAL
        }),
        "credential should be wired to res:credential on execute node, not config.credential on prepare node"
    );
}

#[test]
fn auth_annotated_service_adds_res_credential_to_execute_node() {
    let typed = typed_project_from_sources(&[(
        "dsl/services/auth_test.dag",
        r#"module sample.auth
service sample.Api {
  config { endpoint: "https://api.example.com", auth: BearerToken }
  operation Get {
input { id: String }
output { ok: Bool }
transport rest { method: GET, path: "/v1/items" }
  }
}
func caller(id: String) -> { ok: Bool }
  provides auth: AuthContext
{
  result = sample.Api.Get(id: id)
  return { ok: result.ok }
}"#,
    )]);

    let dag = lower_typed_project(&typed).expect("lowering should succeed");

    let execute_node_id = "execute_transport_sample_auth_sample_Api_Get";

    // Execute node should have res:credential input port
    let execute_node = dag
        .nodes
        .iter()
        .find(|n| n.id.0 == execute_node_id)
        .expect("execute node should exist");
    assert!(
        execute_node
            .inputs
            .iter()
            .any(|port| port.name.0 == PortName::RESOURCE_CREDENTIAL),
        "execute node should have res:credential input port"
    );

    // No config.credential on prepare node
    let prepare_node_id = "prepare_transport_sample_auth_sample_Api_Get";
    let prepare_node = dag
        .nodes
        .iter()
        .find(|n| n.id.0 == prepare_node_id)
        .expect("prepare node should exist");
    assert!(
        !prepare_node
            .inputs
            .iter()
            .any(|port| port.name.0 == "config.credential"),
        "prepare node should NOT have config.credential input (deprecated)"
    );

    // Auth scheme should be stored on the spec metadata
    if let NodeBody::Opaque(LoweredOp::Callable {
        service_metadata: Some(ref metadata),
        ..
    }) = execute_node.body
    {
        if let Some(ServiceOperationSpec::Rest(spec)) = &metadata.spec {
            assert_eq!(
                spec.auth_scheme.as_deref(),
                Some("BearerToken"),
                "auth_scheme should be stored on RestOperationSpec"
            );
        }
    }
}

#[test]
fn credential_chain_output_wires_to_execute_node_res_credential() {
    let typed = typed_project_from_sources(&[(
        "dsl/services/cred_threading.dag",
        r#"module sample.cred

resource AuthContext {
  kind: Capability
  mode: Read
}

service sample.Api {
  config { endpoint: "https://api.example.com", auth: BearerToken }
  operation Get {
input { id: String }
output { ok: Bool }
transport rest { method: GET, path: "/v1/items" }
  }
}
pattern cred_provider() -> { token: String }
  provides auth: AuthContext
{
  return { token: "test-credential" }
}
func caller(id: String) -> { ok: Bool }
  provides auth: AuthContext
{
  cred = cred_provider()
  result = sample.Api.Get(id: id)
  return { ok: result.ok }
}"#,
    )]);

    let dag = lower_typed_project(&typed).expect("lowering should succeed");

    let execute_node_id = "execute_transport_sample_cred_sample_Api_Get";
    assert!(
        dag.edges.iter().any(|edge| {
            edge.to_node.0 == execute_node_id && edge.to_port.0 == PortName::RESOURCE_CREDENTIAL
        }),
        "credential provider output should be wired to execute node res:credential"
    );
}

#[test]
fn auth_input_config_wires_arg_to_execute_res_credential() {
    let typed = typed_project_from_sources(&[(
        "dsl/services/auth_input_test.dag",
        r#"module sample.auth_input
service sample.Api {
  config { endpoint: "https://api.example.com", auth: BearerToken, auth_input: auth_token }
  operation Create {
input { name: String, auth_token: Secret }
output { id: String }
transport rest { method: POST, path: "/v1/things" }
  }
}
func caller(name: String, token: Secret) -> { id: String } {
  result = sample.Api.Create(name: name, auth_token: token)
  return { id: result.id }
}"#,
    )]);

    let dag = lower_typed_project(&typed).expect("lowering should succeed");

    let execute_node_id = "execute_transport_sample_auth_input_sample_Api_Create";
    let prepare_node_id = "prepare_transport_sample_auth_input_sample_Api_Create";

    // auth_token should be wired to res:credential on execute node
    assert!(
        dag.edges.iter().any(|edge| {
            edge.to_node.0 == execute_node_id && edge.to_port.0 == PortName::RESOURCE_CREDENTIAL
        }),
        "auth_input arg should be wired to execute node res:credential"
    );

    // auth_token should NOT be wired to prepare node
    assert!(
        !dag.edges.iter().any(|edge| {
            edge.to_node.0 == prepare_node_id && edge.to_port.0 == "auth_token"
        }),
        "auth_input arg should NOT be wired to prepare node"
    );

    // prepare node should not have auth_token as an input port
    let prepare_node = dag
        .nodes
        .iter()
        .find(|n| n.id.0 == prepare_node_id)
        .expect("prepare node should exist");
    assert!(
        !prepare_node
            .inputs
            .iter()
            .any(|port| port.name.0 == "auth_token"),
        "prepare node should NOT have auth_token input port (excluded by auth_input)"
    );
}

#[test]
fn auth_input_positional_arg_wires_to_execute_res_credential() {
    // When a service call uses positional arguments (no `name:` label),
    // the auth_input argument must still be wired to res:credential on
    // the execute node, and must NOT be wired to the prepare node.
    let typed = typed_project_from_sources(&[(
        "dsl/services/auth_input_positional.dag",
        r#"module sample.auth_positional
service sample.Api {
  config { endpoint: "https://api.example.com", auth: BearerToken, auth_input: auth_token }
  operation Create {
input { name: String, auth_token: Secret }
output { id: String }
transport rest { method: POST, path: "/v1/things" }
  }
}
func caller(name: String, token: Secret) -> { id: String } {
  result = sample.Api.Create(name, token)
  return { id: result.id }
}"#,
    )]);

    let dag = lower_typed_project(&typed).expect("lowering should succeed");

    let execute_node_id = "execute_transport_sample_auth_positional_sample_Api_Create";
    let prepare_node_id = "prepare_transport_sample_auth_positional_sample_Api_Create";

    // Positional auth_token should be wired to res:credential on execute node
    assert!(
        dag.edges.iter().any(|edge| {
            edge.to_node.0 == execute_node_id && edge.to_port.0 == PortName::RESOURCE_CREDENTIAL
        }),
        "positional auth_input arg should be wired to execute node res:credential; edges to execute: {:?}",
        dag.edges.iter().filter(|e| e.to_node.0 == execute_node_id).collect::<Vec<_>>()
    );

    // Positional auth_token should NOT be wired to prepare node
    assert!(
        !dag.edges.iter().any(|edge| {
            edge.to_node.0 == prepare_node_id && edge.to_port.0 == "auth_token"
        }),
        "positional auth_input arg should NOT be wired to prepare node"
    );

    // Non-auth positional arg (name at index 0) should still be wired to prepare
    assert!(
        dag.edges.iter().any(|edge| {
            edge.to_node.0 == prepare_node_id && edge.to_port.0 == "name"
        }),
        "non-auth positional arg 'name' should be wired to prepare node"
    );
}

#[test]
fn auth_input_with_field_access_wires_to_res_credential() {
    let typed = typed_project_from_sources(&[(
        "dsl/services/auth_input_field_access.dag",
        r#"module sample.auth_field
service sample.Api {
  config { endpoint: "https://api.example.com", auth: BearerToken, auth_input: auth_token }
  operation Get {
input { id: String, auth_token: Secret }
output { ok: Bool }
transport rest { method: GET, path: "/v1/items" }
  }
}
func fetch_secret() -> { token: String } {
  return { token: "test-credential" }
}
func caller(id: String) -> { ok: Bool } {
  secret = fetch_secret()
  result = sample.Api.Get(id: id, auth_token: secret.token)
  return { ok: result.ok }
}"#,
    )]);

    let dag = lower_typed_project(&typed).expect("lowering should succeed");

    let execute_node_id = "execute_transport_sample_auth_field_sample_Api_Get";
    assert!(
        dag.edges.iter().any(|edge| {
            edge.to_node.0 == execute_node_id && edge.to_port.0 == PortName::RESOURCE_CREDENTIAL
        }),
        "field-access auth_input should wire to execute node res:credential"
    );
}

#[test]
fn auth_input_literal_arg_wires_to_execute_res_credential() {
    // When auth_input is supplied as a string literal (not a variable), the
    // literal source node's output port must NOT use the `res:` prefix
    // (reserved for inputs). The literal flows through a safe port name
    // into res:credential on the execute node.
    let typed = typed_project_from_sources(&[(
        "dsl/services/auth_input_literal.dag",
        r#"module sample.auth_literal
service sample.Api {
  config { endpoint: "https://api.example.com", auth: BearerToken, auth_input: auth_token }
  operation Create {
input { name: String, auth_token: Secret }
output { id: String }
transport rest { method: POST, path: "/v1/things" }
  }
}
func caller(name: String) -> { id: String } {
  result = sample.Api.Create(name: name, auth_token: "hardcoded-token")
  return { id: result.id }
}"#,
    )]);

    let dag = lower_typed_project(&typed).expect("literal auth_input should lower without error");

    let execute_node_id = "execute_transport_sample_auth_literal_sample_Api_Create";

    // Literal auth_token should be wired to res:credential on execute node
    assert!(
        dag.edges.iter().any(|edge| {
            edge.to_node.0 == execute_node_id && edge.to_port.0 == PortName::RESOURCE_CREDENTIAL
        }),
        "literal auth_input arg should be wired to execute node res:credential"
    );

    // The literal source node should NOT have a res: output port
    let literal_nodes: Vec<_> = dag
        .nodes
        .iter()
        .filter(|n| n.id.0.starts_with("literal_source_"))
        .collect();
    for node in &literal_nodes {
        for port in &node.outputs {
            assert!(
                !port.name.is_resource(),
                "literal source node '{}' has res: output port '{}' — res: prefix is reserved for inputs",
                node.id.0, port.name.0
            );
        }
    }
}

#[test]
/// IS-3: Pipeline compilation without --profile now succeeds with stub interfaces.
fn pipeline_stage_bound_service_call_requires_active_profile() {
    let typed = typed_project_from_sources(&[(
        "dsl/profiles/pipeline_interface_binding.dag",
        r#"module sample.profile
interface IssueProvider {
  capability get {
input {}
output { ok: Bool }
  }
}
service impl.Provider : IssueProvider {
  operation get {
input {}
output { ok: Bool }
transport rest { method: GET, path: "/ok" }
  }
}
profile unit_test {
  bind IssueProvider -> impl.Provider
}
pipeline sdlc uses issues: IssueProvider {
  stage fetch {
issue = issues.get()
  }
}"#,
    )]);

    // IS-3: No-profile compilation succeeds with stub transport.
    let dag = lower_typed_project(&typed)
        .expect("pipeline lowering without profile should succeed with stubs (IS-3)");
    assert!(
        !dag.nodes.is_empty(),
        "lowered DAG should contain pipeline nodes with stub transport"
    );
}

#[test]
fn pipeline_stage_bound_service_call_resolves_via_active_profile_binding() {
    let typed = typed_project_from_sources(&[(
        "dsl/profiles/pipeline_interface_binding.dag",
        r#"module sample.profile
interface IssueProvider {
  capability get {
input {}
output { ok: Bool }
  }
}
service impl.Provider : IssueProvider {
  operation get {
input {}
output { ok: Bool }
transport rest { method: GET, path: "/ok" }
  }
}
profile unit_test {
  bind IssueProvider -> impl.Provider
}
pipeline sdlc uses issues: IssueProvider {
  stage fetch {
issue = issues.get()
  }
}"#,
    )]);

    let dag = lower_typed_project_with_profile(&typed, Some("unit_test"))
        .expect("lowering should succeed with active profile");
    assert!(
        dag.nodes
            .iter()
            .any(|node| node.id.0 == "sample.profile::sdlc"),
        "pipeline node should be lowered"
    );
    assert!(
        dag.nodes.iter().any(|node| {
            node.id
                .0
                .starts_with("parse_transport_sample_profile_impl_Provider_get")
        }),
        "profile binding should include transport triplet nodes for bound implementation"
    );
}

#[test]
fn resource_acquire_release_lower_to_lifecycle_nodes() {
    let typed = typed_project_from_sources(&[(
        "dsl/resources/fs.dag",
        r#"module sample.resources
resource TempFile {
  acquire {
let path = "/tmp/file"
  }
  release {
let done = true
  }
}
func run() -> { ok: Bool } {
  return { ok: true }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let acquire = "acquire_resource_sample_resources_TempFile";
    let release = "release_resource_sample_resources_TempFile";
    assert!(dag.nodes.iter().any(|node| node.id.0 == acquire));
    assert!(dag.nodes.iter().any(|node| node.id.0 == release));
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == acquire
            && edge.from_port.0 == "resource_handle"
            && edge.to_node.0 == release
            && edge.to_port.0 == "resource_handle"
    }));
}

#[test]
fn core_interface_bindings_resolve_with_active_profile() {
    let typed = typed_project_from_sources(&[(
        "dsl/profiles/core_interface_bindings.dag",
        r#"module sample.bindings
interface IssueProvider {
  capability ping {
input {}
output { ok: Bool }
  }
}
interface ClaimStore {
  capability ping {
input {}
output { ok: Bool }
  }
}
interface OutcomeLedger {
  capability ping {
input {}
output { ok: Bool }
  }
}
interface AgentProvider {
  capability ping {
input {}
output { ok: Bool }
  }
}
interface SignalStore {
  capability ping {
input {}
output { ok: Bool }
  }
}
interface ArtifactStore {
  capability ping {
input {}
output { ok: Bool }
  }
}
service impl.Issues : IssueProvider {
  operation ping {
input {}
output { ok: Bool }
transport rest { method: GET, path: "/issues" }
  }
}
service impl.Claims : ClaimStore {
  operation ping {
input {}
output { ok: Bool }
transport rest { method: GET, path: "/claims" }
  }
}
service impl.Outcomes : OutcomeLedger {
  operation ping {
input {}
output { ok: Bool }
transport rest { method: GET, path: "/outcomes" }
  }
}
service impl.Agents : AgentProvider {
  operation ping {
input {}
output { ok: Bool }
transport rest { method: GET, path: "/agents" }
  }
}
service impl.Signals : SignalStore {
  operation ping {
input {}
output { ok: Bool }
transport rest { method: GET, path: "/signals" }
  }
}
service impl.Artifacts : ArtifactStore {
  operation ping {
input {}
output { ok: Bool }
transport rest { method: GET, path: "/artifacts" }
  }
}
profile unit_test {
  bind IssueProvider -> impl.Issues
  bind ClaimStore -> impl.Claims
  bind OutcomeLedger -> impl.Outcomes
  bind AgentProvider -> impl.Agents
  bind SignalStore -> impl.Signals
  bind ArtifactStore -> impl.Artifacts
}
func run() -> {
  issue_ok: Bool,
  claim_ok: Bool,
  outcome_ok: Bool,
  agent_ok: Bool,
  signal_ok: Bool,
  artifact_ok: Bool
}
  uses issues: IssueProvider
  uses claims: ClaimStore
  uses outcomes: OutcomeLedger
  uses agents: AgentProvider
  uses signals: SignalStore
  uses artifacts: ArtifactStore
{
  issue = issues.ping()
  claim = claims.ping()
  outcome = outcomes.ping()
  agent = agents.ping()
  signal = signals.ping()
  artifact = artifacts.ping()
  return {
issue_ok: issue.ok,
claim_ok: claim.ok,
outcome_ok: outcome.ok,
agent_ok: agent.ok,
signal_ok: signal.ok,
artifact_ok: artifact.ok
  }
}"#,
    )]);

    // IS-3: No-profile compilation succeeds with stub transport.
    let stub_dag = lower_typed_project(&typed)
        .expect("lowering without profile should succeed with stubs (IS-3)");
    assert!(
        !stub_dag.nodes.is_empty(),
        "lowered DAG should contain nodes with stub transport"
    );

    let dag = lower_typed_project_with_profile(&typed, Some("unit_test"))
        .expect("lowering should resolve all core interface bindings via profile");
    let expected_parse_prefixes = [
        "parse_transport_sample_bindings_impl_Issues_ping",
        "parse_transport_sample_bindings_impl_Claims_ping",
        "parse_transport_sample_bindings_impl_Outcomes_ping",
        "parse_transport_sample_bindings_impl_Agents_ping",
        "parse_transport_sample_bindings_impl_Signals_ping",
        "parse_transport_sample_bindings_impl_Artifacts_ping",
    ];
    for prefix in expected_parse_prefixes {
        assert!(
            dag.nodes.iter().any(|node| node.id.0.starts_with(prefix)),
            "missing transport parse node for bound implementation `{prefix}`"
        );
    }
}

#[test]
fn uses_clause_wires_acquire_and_release_lifecycle_edges() {
    let typed = typed_project_from_sources(&[(
        "dsl/resources/uses_wiring.dag",
        r#"module sample.resources
resource TempFile {
  acquire { let path = "/tmp/file" }
  release { let done = true }
}
func run() -> { ok: Bool } uses fs: TempFile {
  return { ok: true }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let acquire = "acquire_resource_sample_resources_TempFile";
    let release = "release_resource_sample_resources_TempFile";
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == acquire
            && edge.from_port.0 == "resource_handle"
            && edge.to_node.0 == "sample.resources::run"
            && edge.to_port.0 == PortName::DEPS
    }));
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "sample.resources::run"
            && edge.from_port.0 == "ok"
            && edge.to_node.0 == release
            && edge.to_port.0 == "resource_handle"
    }));
}

#[test]
fn unresolved_uses_clause_reports_lower_error() {
    let typed = typed_project_from_sources(&[(
        "dsl/resources/unresolved_uses.dag",
        r#"module sample.resources
func run() -> { ok: Bool } uses fs: MissingResource {
  return { ok: true }
}"#,
    )]);
    let error = lower_typed_project(&typed).expect_err("lowering should fail");
    assert!(matches!(
        error,
        LowerError::UnresolvedUsedResource {
            caller,
            binding,
            resource_type,
        } if caller == "sample.resources::run"
            && binding == "fs"
            && resource_type == "MissingResource"
    ));
}

#[test]
fn uses_clause_with_runtime_config_suffix_resolves_resource_type() {
    let typed = typed_project_from_sources(&[(
        "dsl/resources/configured_uses_wiring.dag",
        r#"module sample.resources
resource TempFile {
  acquire { let path = "/tmp/file" }
  release { let done = true }
}
func run() -> { ok: Bool } uses fs: TempFile(mode: ReadWrite) {
  return { ok: true }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "acquire_resource_sample_resources_TempFile"
            && edge.from_port.0 == "resource_handle"
            && edge.to_node.0 == "sample.resources::run"
            && edge.to_port.0 == PortName::DEPS
    }));
}

#[test]
fn uses_interface_with_provider_hint_resolves_matching_resource_lifecycle() {
    let typed = typed_project_from_sources(&[(
        "dsl/infra/providers.dag",
        r#"module infra.providers
interface ObjectStorage {
  capability read {
input { path: String }
output { body: String }
  }
}
resource GcsBucket implements ObjectStorage {
  provider: Gcp
  acquire { let ready = true }
  capability read {
input { path: String }
output { body: String }
  }
}
resource S3Bucket implements ObjectStorage {
  provider: Aws
  acquire { let ready = true }
  capability read {
input { path: String }
output { body: String }
  }
}
func run() -> { ok: Bool } uses store: ObjectStorage(cloud: GcpConfig) {
  return { ok: true }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "acquire_resource_infra_providers_GcsBucket"
            && edge.from_port.0 == "resource_handle"
            && edge.to_node.0 == "infra.providers::run"
            && edge.to_port.0 == PortName::DEPS
    }));
    assert!(
        !dag.edges.iter().any(|edge| {
            edge.from_node.0 == "acquire_resource_infra_providers_S3Bucket"
                && edge.to_node.0 == "infra.providers::run"
                && edge.to_port.0 == PortName::DEPS
        }),
        "provider hint should avoid wiring non-matching provider resources"
    );
}

#[test]
fn uses_interface_without_provider_hint_fails_when_multiple_providers_exist() {
    let typed = typed_project_from_sources(&[(
        "dsl/infra/providers_ambiguous.dag",
        r#"module infra.providers
interface ObjectStorage {
  capability read {
input { path: String }
output { body: String }
  }
}
resource GcsBucket implements ObjectStorage {
  provider: Gcp
  acquire { let ready = true }
  capability read {
input { path: String }
output { body: String }
  }
}
resource S3Bucket implements ObjectStorage {
  provider: Aws
  acquire { let ready = true }
  capability read {
input { path: String }
output { body: String }
  }
}
func run() -> { ok: Bool } uses store: ObjectStorage {
  return { ok: true }
}"#,
    )]);
    let error = lower_typed_project(&typed).expect_err("lowering should fail");
    assert!(matches!(
        error,
        LowerError::AmbiguousUsedResource {
            caller,
            binding,
            resource_type,
        } if caller == "infra.providers::run"
            && binding == "store"
            && resource_type == "ObjectStorage"
    ));
}

#[test]
fn uses_interface_with_aws_and_azure_provider_hints_wire_matching_resources() {
    let typed = typed_project_from_sources(&[(
        "dsl/infra/providers_multi_cloud.dag",
        r#"module infra.providers
interface ObjectStorage {
  capability read {
input { path: String }
output { body: String }
  }
}
resource GcsBucket implements ObjectStorage {
  provider: Gcp
  acquire { let ready = true }
  capability read {
input { path: String }
output { body: String }
  }
}
resource S3Bucket implements ObjectStorage {
  provider: Aws
  acquire { let ready = true }
  capability read {
input { path: String }
output { body: String }
  }
}
resource BlobContainer implements ObjectStorage {
  provider: Azure
  acquire { let ready = true }
  capability read {
input { path: String }
output { body: String }
  }
}
func run_aws() -> { ok: Bool } uses store: ObjectStorage(cloud: AwsConfig) {
  return { ok: true }
}
func run_azure() -> { ok: Bool } uses store: ObjectStorage(cloud: AzureConfig) {
  return { ok: true }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");

    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "acquire_resource_infra_providers_S3Bucket"
            && edge.from_port.0 == "resource_handle"
            && edge.to_node.0 == "infra.providers::run_aws"
            && edge.to_port.0 == PortName::DEPS
    }));
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "acquire_resource_infra_providers_BlobContainer"
            && edge.from_port.0 == "resource_handle"
            && edge.to_node.0 == "infra.providers::run_azure"
            && edge.to_port.0 == PortName::DEPS
    }));
    assert!(
        !dag.edges.iter().any(|edge| {
            edge.from_node.0 == "acquire_resource_infra_providers_GcsBucket"
                && (edge.to_node.0 == "infra.providers::run_aws"
                    || edge.to_node.0 == "infra.providers::run_azure")
                && edge.to_port.0 == PortName::DEPS
        }),
        "aws/azure provider hints should not wire unrelated gcp resources"
    );
}

#[test]
fn store_artifact_portability_wires_gcp_aws_and_azure_resources() {
    let typed = typed_project_from_sources(&[(
        "dsl/examples/portability.dag",
        r#"module examples.portability
interface ObjectStorage {
  capability write {
input { key: String, content: String }
output { ok: Bool }
  }
}
resource GcsBucket implements ObjectStorage {
  provider: Gcp
  acquire { let ready = true }
  capability write {
input { key: String, content: String }
output { ok: Bool }
  }
}
resource S3Bucket implements ObjectStorage {
  provider: Aws
  acquire { let ready = true }
  capability write {
input { key: String, content: String }
output { ok: Bool }
  }
}
resource BlobContainer implements ObjectStorage {
  provider: Azure
  acquire { let ready = true }
  capability write {
input { key: String, content: String }
output { ok: Bool }
  }
}
func store_artifact_gcp(key: String, content: String) -> { ok: Bool } uses store: ObjectStorage(cloud: GcpConfig) {
  result = store.write(key: key, content: content)
  return { ok: result.ok }
}
func store_artifact_aws(key: String, content: String) -> { ok: Bool } uses store: ObjectStorage(cloud: AwsConfig) {
  result = store.write(key: key, content: content)
  return { ok: result.ok }
}
func store_artifact_azure(key: String, content: String) -> { ok: Bool } uses store: ObjectStorage(cloud: AzureConfig) {
  result = store.write(key: key, content: content)
  return { ok: result.ok }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");

    for (resource, target) in [
        ("GcsBucket", "examples.portability::store_artifact_gcp"),
        ("S3Bucket", "examples.portability::store_artifact_aws"),
        (
            "BlobContainer",
            "examples.portability::store_artifact_azure",
        ),
    ] {
        let acquire = format!("acquire_resource_examples_portability_{resource}");
        assert!(
            dag.edges.iter().any(|edge| {
                edge.from_node.0 == acquire
                    && edge.from_port.0 == "resource_handle"
                    && edge.to_node.0 == target
                    && edge.to_port.0 == PortName::DEPS
            }),
            "expected provider-specific resource wiring for {target}"
        );
    }
}

#[test]
fn cross_provider_auth_calls_resolve_all_credential_chains() {
    let typed = typed_project_from_sources(&[
        (
            "dsl/cloud/gcp/credential.dag",
            r#"module cloud.gcp.credential
func acquire_gcp_secret() -> { token: String } {
  return { token: "gcp" }
}"#,
        ),
        (
            "dsl/cloud/aws/credential.dag",
            r#"module cloud.aws.credential
func acquire_aws_secret() -> { token: String } {
  return { token: "aws" }
}"#,
        ),
        (
            "dsl/cloud/azure/credential.dag",
            r#"module cloud.azure.credential
func acquire_azure_secret() -> { token: String } {
  return { token: "azure" }
}"#,
        ),
        (
            "dsl/examples/auth.dag",
            r#"module examples.auth
import cloud.gcp.credential { acquire_gcp_secret }
import cloud.aws.credential { acquire_aws_secret }
import cloud.azure.credential { acquire_azure_secret }

func cross_provider_auth() -> { ok: Bool } {
  gcp = acquire_gcp_secret()
  aws = acquire_aws_secret()
  azure = acquire_azure_secret()
  return { ok: true }
}"#,
        ),
    ]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let caller = "examples.auth::cross_provider_auth";
    for callee in [
        "cloud.gcp.credential::acquire_gcp_secret",
        "cloud.aws.credential::acquire_aws_secret",
        "cloud.azure.credential::acquire_azure_secret",
    ] {
        assert!(
            dag.edges
                .iter()
                .any(|edge| edge.from_node.0 == callee && edge.to_node.0 == caller),
            "expected dependency edge from {callee} into cross-provider auth caller"
        );
    }
}

#[test]
fn aws_resource_module_emits_object_storage_contract_verification_nodes() {
    let typed = typed_project_for_module_with_dependency_closure("infra.aws.resources");
    let object_storage_contracts = typed
        .modules
        .iter()
        .find(|module| module.module_path.as_dotted() == "infra.core")
        .and_then(|module| {
            module.ast.items.iter().find_map(|item| match &item.node {
                Item::InterfaceDef(interface) if interface.name == "ObjectStorage" => {
                    Some(interface.contracts.len())
                }
                _ => None,
            })
        })
        .expect("infra.core.ObjectStorage interface should exist");
    assert!(
        object_storage_contracts > 0,
        "infra.core.ObjectStorage should carry typed contract declarations"
    );
    let dag = lower_target_module(&typed, "infra.aws.resources");

    let verify_node = "verify_contract_infra_aws_resources_S3Bucket_ObjectStorage_0";
    assert!(
        dag.nodes.iter().any(|node| node.id.0 == verify_node),
        "aws resources should emit object-storage contract verification nodes"
    );
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "acquire_resource_infra_aws_resources_S3Bucket"
            && edge.from_port.0 == "resource_handle"
            && edge.to_node.0 == verify_node
            && edge.to_port.0 == PortName::DEPS
    }));
}

#[test]
fn azure_resource_module_emits_object_storage_contract_verification_nodes() {
    let typed = typed_project_for_module_with_dependency_closure("infra.azure.resources");
    let dag = lower_target_module(&typed, "infra.azure.resources");

    let verify_node = "verify_contract_infra_azure_resources_BlobContainer_ObjectStorage_0";
    assert!(
        dag.nodes.iter().any(|node| node.id.0 == verify_node),
        "azure resources should emit object-storage contract verification nodes"
    );
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "acquire_resource_infra_azure_resources_BlobContainer"
            && edge.from_port.0 == "resource_handle"
            && edge.to_node.0 == verify_node
            && edge.to_port.0 == PortName::DEPS
    }));
}

#[test]
fn interface_contract_annotations_lower_to_verification_nodes_for_implementors() {
    let typed = typed_project_from_sources(&[(
        "dsl/infra/contracts.dag",
        r#"module infra.contracts
interface ObjectStorage {
  capability read {
input { path: String }
output { body: String }
  }
  contract read_after_write_returns_value
  contract read_missing_returns_empty
}
resource GcsBucket implements ObjectStorage {
  acquire { let ready = true }
  capability read {
input { path: String }
output { body: String }
  }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let verify_0 = "verify_contract_infra_contracts_GcsBucket_ObjectStorage_0";
    let verify_1 = "verify_contract_infra_contracts_GcsBucket_ObjectStorage_1";
    assert!(dag.nodes.iter().any(|node| node.id.0 == verify_0));
    assert!(dag.nodes.iter().any(|node| node.id.0 == verify_1));
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "acquire_resource_infra_contracts_GcsBucket"
            && edge.from_port.0 == "resource_handle"
            && edge.to_node.0 == verify_0
            && edge.to_port.0 == PortName::DEPS
    }));
}

#[test]
fn interface_contract_annotations_cover_all_provider_implementors() {
    let typed = typed_project_from_sources(&[(
        "dsl/infra/contracts_multi_cloud.dag",
        r#"module infra.contracts
interface ObjectStorage {
  capability read {
input { path: String }
output { body: String }
  }
  contract read_returns_value
}
resource GcsBucket implements ObjectStorage {
  acquire { let ready = true }
  capability read {
input { path: String }
output { body: String }
  }
}
resource S3Bucket implements ObjectStorage {
  acquire { let ready = true }
  capability read {
input { path: String }
output { body: String }
  }
}
resource BlobContainer implements ObjectStorage {
  acquire { let ready = true }
  capability read {
input { path: String }
output { body: String }
  }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");

    for (resource, verify_id) in [
        (
            "GcsBucket",
            "verify_contract_infra_contracts_GcsBucket_ObjectStorage_0",
        ),
        (
            "S3Bucket",
            "verify_contract_infra_contracts_S3Bucket_ObjectStorage_0",
        ),
        (
            "BlobContainer",
            "verify_contract_infra_contracts_BlobContainer_ObjectStorage_0",
        ),
    ] {
        let acquire_id = format!("acquire_resource_infra_contracts_{resource}");
        assert!(
            dag.nodes.iter().any(|node| node.id.0 == verify_id),
            "expected verification node for {resource}"
        );
        assert!(
            dag.edges.iter().any(|edge| {
                edge.from_node.0 == acquire_id
                    && edge.from_port.0 == "resource_handle"
                    && edge.to_node.0 == verify_id
                    && edge.to_port.0 == PortName::DEPS
            }),
            "expected acquire->verify edge for {resource}"
        );
    }
}

#[test]
fn provides_clause_adds_provider_node_and_edge() {
    let typed = typed_project_from_sources(&[(
        "dsl/resources/provides_wiring.dag",
        r#"module sample.resources
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
func run() -> { ok: Bool } provides out: Storage {
  return { ok: true }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let provider = "provide_resource_sample_resources_run_out";
    assert!(dag.nodes.iter().any(|node| node.id.0 == provider));
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "sample.resources::run"
            && edge.from_port.0 == "ok"
            && edge.to_node.0 == provider
            && edge.to_port.0 == "trigger"
    }));
}

#[test]
fn unresolved_provides_clause_reports_lower_error() {
    let typed = typed_project_from_sources(&[(
        "dsl/resources/unresolved_provides.dag",
        r#"module sample.resources
func run() -> { ok: Bool } provides out: MissingResource {
  return { ok: true }
}"#,
    )]);
    let error = lower_typed_project(&typed).expect_err("lowering should fail");
    assert!(matches!(
        error,
        LowerError::UnresolvedProvidedResource {
            caller,
            binding,
            resource_type,
        } if caller == "sample.resources::run"
            && binding == "out"
            && resource_type == "MissingResource"
    ));
}

#[test]
fn ambiguous_provides_clause_reports_lower_error() {
    let typed = typed_project_from_sources(&[(
        "dsl/resources/ambiguous_provides.dag",
        r#"module sample.resources
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
resource LocalStore implements Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
resource BackupStore implements Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
func run() -> { ok: Bool } provides out: Storage {
  return { ok: true }
}"#,
    )]);
    let error = lower_typed_project(&typed).expect_err("lowering should fail");
    assert!(matches!(
        error,
        LowerError::AmbiguousProvidedResource {
            caller,
            binding,
            resource_type,
        } if caller == "sample.resources::run"
            && binding == "out"
            && resource_type == "Storage"
    ));
}

#[test]
fn provides_clause_with_runtime_config_suffix_resolves_resource_type() {
    let typed = typed_project_from_sources(&[(
        "dsl/resources/configured_provides_wiring.dag",
        r#"module sample.resources
resource TempFile {
  release {
let done = true
  }
}
func run() -> { ok: Bool } provides file: TempFile(mode: ReadWrite) {
  return { ok: true }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "provide_resource_sample_resources_run_file"
            && edge.from_port.0 == "file"
            && edge.to_node.0 == "release_resource_sample_resources_TempFile"
            && edge.to_port.0 == "resource_handle"
    }));
}

#[test]
fn provides_resource_with_release_wires_provider_output_to_lifecycle_release() {
    let typed = typed_project_from_sources(&[(
        "dsl/resources/provides_release_wiring.dag",
        r#"module sample.resources
resource TempFile {
  release {
let done = true
  }
}
func run() -> { ok: Bool } provides file: TempFile {
  return { ok: true }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    assert!(dag.edges.iter().any(|edge| {
        edge.from_node.0 == "provide_resource_sample_resources_run_file"
            && edge.from_port.0 == "file"
            && edge.to_node.0 == "release_resource_sample_resources_TempFile"
            && edge.to_port.0 == "resource_handle"
    }));
}

#[test]
fn known_interface_uses_without_lifecycle_are_tolerated() {
    let typed = typed_project_from_sources(&[(
        "dsl/resources/interface_uses.dag",
        r#"module sample.resources
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
func run() -> { ok: Bool } uses store: Storage {
  return { ok: true }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    assert!(dag
        .nodes
        .iter()
        .any(|node| node.id.0 == "sample.resources::run"));
    assert!(
        !dag.edges.iter().any(|edge| edge.to_node.0 == "sample.resources::run"
            && edge.to_port.0 == PortName::DEPS),
        "interface-only uses should not fabricate lifecycle dependency edges"
    );
}

#[test]
fn known_interface_provides_without_lifecycle_are_tolerated() {
    let typed = typed_project_from_sources(&[(
        "dsl/resources/interface_provides.dag",
        r#"module sample.resources
interface Storage {
  capability read {
input { path: String }
output { body: String }
  }
}
func run() -> { ok: Bool } provides out: Storage {
  return { ok: true }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    assert!(dag
        .nodes
        .iter()
        .any(|node| node.id.0 == "provide_resource_sample_resources_run_out"));
    assert!(
        !dag.edges.iter().any(|edge| {
            edge.from_node.0 == "provide_resource_sample_resources_run_out"
                && edge.to_port.0 == "resource_handle"
        }),
        "interface-only provides should not fabricate lifecycle release edges"
    );
}

#[test]
fn known_std_resource_provides_without_lifecycle_are_tolerated() {
    let typed = typed_project_from_sources(&[(
        "dsl/resources/std_resource_provides.dag",
        r#"module sample.resources
func run() -> { ok: Bool } provides auth: AuthContext {
  return { ok: true }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    assert!(dag
        .nodes
        .iter()
        .any(|node| node.id.0 == "provide_resource_sample_resources_run_auth"));
    assert!(
        !dag.edges.iter().any(|edge| {
            edge.from_node.0 == "provide_resource_sample_resources_run_auth"
                && edge.to_port.0 == "resource_handle"
        }),
        "std resource provides should not fabricate lifecycle release edges when lifecycle module is absent"
    );
}

#[test]
fn lower_errors_when_no_callable_items_exist() {
    let typed = typed_project_from_sources(&[(
        "dsl/types_only.dag",
        "module sample.types\ntype Name = String",
    )]);
    let error = lower_typed_project(&typed).expect_err("should fail without callable items");
    assert!(matches!(error, LowerError::NoLowerableItems));
}

#[test]
fn classify_obligation_uses_structural_lowered_metadata() {
    let op = LoweredOp::Callable {
        module: "sample.services".to_string(),
        kind: CallableKind::Pattern,
        name: "prepare_transport_sample".to_string(),
        obligation: ObligationCategory::ServiceTransportPrepare,
        service_metadata: None,
        is_interactive: false,
        resource_target: None,
        fn_body: None,
    };
    assert_eq!(
        classify_obligation(&op),
        ObligationCategory::ServiceTransportPrepare
    );

    let pipeline = LoweredOp::Pipeline {
        module: "pipelines.ci".to_string(),
        name: "ci".to_string(),
        stages: 4,
        stage_names: vec![
            "cloud_env".to_string(),
            "codegen_stage".to_string(),
            "deps_check".to_string(),
            "generate".to_string(),
        ],
    };
    assert_eq!(classify_obligation(&pipeline), ObligationCategory::None);
}

#[test]
fn interactive_func_annotation_sets_structural_callable_metadata() {
    // After annotation removal, is_interactive is always false (no typed equivalent).
    // This test verifies the callable node is correctly lowered.
    let typed = typed_project_from_sources(&[(
        "dsl/interactive.dag",
        r#"module sample.ui
func prompt() -> { ok: Bool } {
  return { ok: true }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let op = dag
        .nodes
        .iter()
        .find(|node| node.id.0 == "sample.ui::prompt")
        .and_then(|node| match &node.body {
            gunbc_ir::node::NodeBody::Opaque(op) => Some(op),
            gunbc_ir::node::NodeBody::SubDag(_) => None,
        })
        .expect("callable node should exist");
    assert!(matches!(
        op,
        LoweredOp::Callable {
            is_interactive: false,
            ..
        }
    ));
}

#[test]
fn canonical_kind_for_obligation_maps_categories() {
    assert_eq!(
        canonical_kind_for_obligation(ObligationCategory::None),
        None
    );
    assert_eq!(
        canonical_kind_for_obligation(ObligationCategory::ServiceTransportExecute),
        Some("transport")
    );

    for obligation in [
        ObligationCategory::ServiceTransportPrepare,
        ObligationCategory::ServiceTransportParse,
        ObligationCategory::ServiceParamSource,
        ObligationCategory::ResourceProvide,
        ObligationCategory::ResourceAcquire,
        ObligationCategory::ResourceRelease,
        ObligationCategory::InterfaceContractVerification,
    ] {
        assert_eq!(
            canonical_kind_for_obligation(obligation),
            Some("pattern-expanded")
        );
    }
}

#[test]
fn topology_with_obligation_kinds_populates_canonical_kind_metadata() {
    let mut dag: Dag<LoweredOp> = Dag::new();
    dag.add_node(Node::opaque(
        "execute_transport_sample",
        vec![],
        vec![],
        LoweredOp::Callable {
            module: "sample.services".to_string(),
            kind: CallableKind::Pattern,
            name: "execute_transport_sample".to_string(),
            obligation: ObligationCategory::ServiceTransportExecute,
            service_metadata: None,
            is_interactive: false,
            resource_target: None,
            fn_body: None,
        },
    ));

    let topo = topology_with_obligation_kinds(&dag);
    assert_eq!(topo.nodes.len(), 1);
    assert_eq!(topo.nodes[0].canonical_kind.as_deref(), Some("transport"));
}

// Test infrastructure: filesystem access for test fixtures
#[allow(clippy::disallowed_methods)]
#[test]
fn makegen_parity_report_is_deterministic() {
    let lowered = load_makegen_lowered();
    let reference = reference_makegen_shape();

    let report_a = compare_topology(&lowered, &reference);
    let report_b = compare_topology(&lowered, &reference);
    assert_eq!(report_a, report_b, "parity reports must be deterministic");
    assert!(
        !report_a.is_exact_match(),
        "phase-1 scaffold should still report makegen parity deltas"
    );
    assert!(
        report_a.added_nodes + report_a.removed_nodes + report_a.changed_nodes > 0,
        "parity report should continue surfacing remaining topology differences"
    );
}

#[test]
fn canonical_ir_json_is_deterministic_for_makegen() {
    let lowered = load_makegen_lowered();
    let first = canonical_ir_json(&lowered).expect("first canonical json run should serialize");
    let second =
        canonical_ir_json(&lowered).expect("second canonical json run should serialize");
    assert_eq!(first, second, "canonical IR json must be byte-stable");
}

#[test]
fn parity_report_includes_changed_node_details() {
    let mut candidate = Dag::new();
    candidate.add_node(Node::opaque(
        "render",
        vec![Port::scalar("registry", "ToolRegistry")],
        vec![Port::scalar("content", "String")],
        LoweredOp::Callable {
            module: "tools.makegen".to_string(),
            kind: CallableKind::Fn,
            name: "render".to_string(),
            obligation: ObligationCategory::None,
            service_metadata: None,
            is_interactive: false,
            resource_target: None,
            fn_body: None,
        },
    ));
    let mut reference = Dag::new();
    reference.add_node(Node::opaque(
        "render",
        vec![Port::scalar("registry", "ToolRegistry")],
        vec![Port::scalar("makefile", "String")],
        (),
    ));

    let report = compare_ir(&candidate, &reference);
    assert_eq!(report.changed_nodes, 1);
    assert_eq!(report.changed_node_details.len(), 1);
    assert_eq!(report.changed_node_details[0].node_id, "render");
    assert!(report.changed_node_details[0]
        .differences
        .iter()
        .any(|difference| difference.contains("output ports differ")));
}

#[test]
fn parity_report_lists_added_and_removed_items_in_sorted_order() {
    let mut candidate = Dag::new();
    candidate.add_node(Node::opaque(
        "b",
        vec![Port::scalar("in", "String")],
        vec![Port::scalar("out", "String")],
        LoweredOp::Callable {
            module: "sample".to_string(),
            kind: CallableKind::Fn,
            name: "b".to_string(),
            obligation: ObligationCategory::None,
            service_metadata: None,
            is_interactive: false,
            resource_target: None,
            fn_body: None,
        },
    ));
    candidate.add_node(Node::opaque(
        "a",
        vec![Port::scalar("in", "String")],
        vec![Port::scalar("out", "String")],
        LoweredOp::Callable {
            module: "sample".to_string(),
            kind: CallableKind::Fn,
            name: "a".to_string(),
            obligation: ObligationCategory::None,
            service_metadata: None,
            is_interactive: false,
            resource_target: None,
            fn_body: None,
        },
    ));
    candidate.add_edge(Edge::new("a", "out", "b", "in"));

    let mut reference = Dag::new();
    reference.add_node(Node::opaque(
        "c",
        vec![Port::scalar("in", "String")],
        vec![Port::scalar("out", "String")],
        (),
    ));
    reference.add_node(Node::opaque(
        "d",
        vec![Port::scalar("in", "String")],
        vec![Port::scalar("out", "String")],
        (),
    ));
    reference.add_edge(Edge::new("c", "out", "d", "in"));

    let report = compare_ir(&candidate, &reference);
    assert_eq!(
        report.added_node_ids,
        vec!["a".to_string(), "b".to_string()]
    );
    assert_eq!(
        report.removed_node_ids,
        vec!["c".to_string(), "d".to_string()]
    );
    assert_eq!(
        report.added_edge_ids,
        vec!["a.out->b.in".to_string()],
        "added edges should be deterministic and sorted"
    );
    assert_eq!(
        report.removed_edge_ids,
        vec!["c.out->d.in".to_string()],
        "removed edges should be deterministic and sorted"
    );
}

#[allow(clippy::disallowed_methods)]
fn load_makegen_lowered() -> Dag<LoweredOp> {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl/tools/makegen.dag");
    let source = fs::read_to_string(file).expect("should read makegen source");
    let typed = typed_project_from_sources(&[("dsl/tools/makegen.dag", &source)]);
    lower_typed_project(&typed).expect("lowering should succeed")
}

fn reference_makegen_shape() -> Dag<()> {
    let mut dag = Dag::new();
    dag.add_node(Node::opaque(
        "fs_env",
        vec![],
        vec![Port::scalar("FilesystemHandle", "FilesystemHandle")],
        (),
    ));
    dag.add_node(Node::opaque(
        "load_registry",
        vec![],
        vec![Port::scalar("registry", "ToolRegistry")],
        (),
    ));
    dag.add_node(Node::opaque(
        "render_makefile",
        vec![Port::scalar("registry", "ToolRegistry")],
        vec![Port::scalar("makefile_content", "String")],
        (),
    ));
    dag.add_node(Node::opaque(
        "prepare_read_makegen",
        vec![
            Port::scalar("path", "String"),
            Port::scalar("res:file:Makefile", "FilesystemHandle"),
        ],
        vec![Port::scalar("request", "TransportRequest")],
        (),
    ));
    dag.add_node(Node::opaque(
        "execute_read_makegen",
        vec![Port::scalar("request", "TransportRequest")],
        vec![Port::scalar("response", "TransportResponse")],
        (),
    ));
    dag.add_node(Node::opaque(
        "compare_makegen_content",
        vec![
            Port::scalar("expected_content", "String"),
            Port::scalar("response", "TransportResponse"),
        ],
        vec![Port::scalar("fresh", "Bool"), Port::scalar("skip", "Bool")],
        (),
    ));
    dag.add_node(Node::opaque(
        "prepare_write_makegen",
        vec![
            Port::scalar("content", "String"),
            Port::scalar("path", "String"),
        ],
        vec![Port::scalar("request", "TransportRequest")],
        (),
    ));
    dag.add_node(Node::opaque(
        "execute_makegen_transport",
        vec![
            Port::scalar("request", "TransportRequest"),
            Port::scalar("skip", "Bool"),
        ],
        vec![Port::scalar("response", "TransportResponse")],
        (),
    ));
    dag.add_node(Node::opaque(
        "param_source_tools_makegen_makegen_path",
        vec![Port::scalar("path", "String")],
        vec![Port::scalar("path", "String")],
        (),
    ));

    dag.add_edge(Edge::new(
        "load_registry",
        "registry",
        "render_makefile",
        "registry",
    ));
    dag.add_edge(Edge::new(
        "render_makefile",
        "makefile_content",
        "compare_makegen_content",
        "expected_content",
    ));
    dag.add_edge(Edge::new(
        "render_makefile",
        "makefile_content",
        "prepare_write_makegen",
        "content",
    ));
    dag.add_edge(Edge::new(
        "prepare_read_makegen",
        "request",
        "execute_read_makegen",
        "request",
    ));
    dag.add_edge(Edge::new(
        "execute_read_makegen",
        "response",
        "compare_makegen_content",
        "response",
    ));
    dag.add_edge(Edge::new(
        "prepare_write_makegen",
        "request",
        "execute_makegen_transport",
        "request",
    ));
    dag.add_edge(Edge::new(
        "compare_makegen_content",
        "skip",
        "execute_makegen_transport",
        "skip",
    ));
    dag.add_edge(Edge::new(
        "param_source_tools_makegen_makegen_path",
        "path",
        "prepare_read_makegen",
        "path",
    ));
    dag.add_edge(Edge::new(
        "param_source_tools_makegen_makegen_path",
        "path",
        "prepare_write_makegen",
        "path",
    ));
    dag
}

// ── Cross-callable transport dedup ─────────────────────────────────

#[test]
fn cross_callable_service_dedup_clones_transport_triplet() {
    // Two func items in the same module that both call the same service
    // operation must each get their own transport triplet (original + _c1).
    let typed = typed_project_from_sources(&[(
        "dsl/services/dedup_cross.dag",
        r#"module sample.dedup
service Echo {
  operation Ping(message: String) -> { reply: String } {
transport shell { argv: ["echo", "{message}"] }
  }
}
func alpha(msg: String) -> { reply: String } {
  result = Echo.Ping(message: msg)
  return { reply: result.reply }
}
func beta(msg: String) -> { reply: String } {
  result = Echo.Ping(message: msg)
  return { reply: result.reply }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let prepare_nodes: Vec<_> = dag
        .nodes
        .iter()
        .filter(|n| n.id.0.starts_with("prepare_transport_") && n.id.0.contains("Echo_Ping"))
        .collect();
    assert!(
        prepare_nodes.len() >= 2,
        "expected at least 2 prepare nodes for Echo.Ping (original + clone), found {}: {:?}",
        prepare_nodes.len(),
        prepare_nodes.iter().map(|n| &n.id.0).collect::<Vec<_>>()
    );
    // Verify no two edges target the same scalar (node, port) from
    // different sources — the original duplicate-edge bug.
    let mut edge_targets: std::collections::HashMap<(String, String), Vec<String>> =
        std::collections::HashMap::new();
    for edge in &dag.edges {
        edge_targets
            .entry((edge.to_node.0.clone(), edge.to_port.0.clone()))
            .or_default()
            .push(edge.from_node.0.clone());
    }
    for ((node, port), sources) in &edge_targets {
        if port == PortName::DEPS || port == "request" || port == "response" {
            continue; // these are allowed to have multiple sources
        }
        assert!(
            sources.len() <= 1,
            "duplicate scalar edge to {node}:{port} from {sources:?}",
        );
    }
}

#[test]
fn same_callable_dual_service_call_still_clones() {
    // Regression: a single func calling the same service twice must still
    // produce a cloned triplet for the second invocation.
    let typed = typed_project_from_sources(&[(
        "dsl/services/dedup_same.dag",
        r#"module sample.dedup_same
service Echo {
  operation Ping(message: String) -> { reply: String } {
transport shell { argv: ["echo", "{message}"] }
  }
}
func dual(msg: String) -> { reply: String } {
  first = Echo.Ping(message: msg)
  second = Echo.Ping(message: first.reply)
  return { reply: second.reply }
}"#,
    )]);
    let dag = lower_typed_project(&typed).expect("lowering should succeed");
    let prepare_nodes: Vec<_> = dag
        .nodes
        .iter()
        .filter(|n| n.id.0.starts_with("prepare_transport_") && n.id.0.contains("Echo_Ping"))
        .collect();
    assert!(
        prepare_nodes.len() >= 2,
        "expected at least 2 prepare nodes (original + clone), found {}: {:?}",
        prepare_nodes.len(),
        prepare_nodes.iter().map(|n| &n.id.0).collect::<Vec<_>>()
    );
}

// ── Pattern expansion tests ──────────────────────────────────────

/// Lower a synthetic test module injected into the real DSL corpus.
/// The synthetic_modules are (path, source) pairs that get added to
/// the module graph alongside the real DSL standard library.
#[allow(clippy::disallowed_methods)]
fn lower_typed_project_for_module_with_dependency_closure_and_entry(
    target_module: &str,
    synthetic_modules: &[(&str, &str)],
) -> Dag<LoweredOp> {
    let dsl_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
    let mut graph =
        ModuleGraph::discover(&[dsl_root]).expect("module graph discovery should succeed");

    // Inject synthetic modules.
    for (path, source) in synthetic_modules {
        let ast = parser::parse(source).expect("synthetic module should parse");
        let module_path = ast
            .module_path
            .as_ref()
            .map(|module| module.node.clone())
            .expect("module declaration is required");
        let module = ResolvedModule {
            path: PathBuf::from(path),
            ast,
            module_path,
            dependencies: Vec::new(),
        };
        graph.modules.push(module);
    }

    // Resolve import dependencies for the synthetic modules.
    let module_path_to_index: HashMap<String, usize> = graph
        .modules
        .iter()
        .enumerate()
        .map(|(index, module)| (module.module_path.as_dotted(), index))
        .collect();
    // For each module, resolve its import paths to dependency indices.
    for module_index in 0..graph.modules.len() {
        let imports: Vec<String> = {
            let module = &graph.modules[module_index];
            module
                .ast
                .imports
                .iter()
                .map(|import| import.node.path.segments.join("."))
                .collect()
        };
        let deps: Vec<usize> = imports
            .iter()
            .filter_map(|import_path| module_path_to_index.get(import_path).copied())
            .collect();
        if !deps.is_empty() {
            let module = &mut graph.modules[module_index];
            for dep in deps {
                if !module.dependencies.contains(&dep) {
                    module.dependencies.push(dep);
                }
            }
        }
    }

    let typed = typecheck_module_graph(graph).expect("typecheck should succeed");

    // Build the scope: target module + all its transitive dependencies.
    let module_lookup: HashMap<String, usize> = typed
        .modules
        .iter()
        .enumerate()
        .map(|(index, module)| (module.module_path.as_dotted(), index))
        .collect();
    let target_index = typed
        .modules
        .iter()
        .position(|module| module.module_path.as_dotted() == target_module)
        .unwrap_or_else(|| panic!("target module {target_module} should exist"));
    let mut scope = HashSet::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::from([target_index]);
    while let Some(module_index) = queue.pop_front() {
        if !visited.insert(module_index) {
            continue;
        }
        let Some(module) = typed.modules.get(module_index) else {
            continue;
        };
        scope.insert(module.module_path.as_dotted());
        for import in &module.imports {
            let import_name = import.as_dotted();
            if let Some(import_index) = module_lookup.get(&import_name) {
                queue.push_back(*import_index);
            }
        }
    }
    lower_typed_project_for_modules_with_entry(&typed, &scope, None, Some(target_module))
        .expect("lowering should succeed")
}

#[test]
fn expand_non_generic_pattern_creates_transport_triplet_and_compare_node() {
    // Uses the real DSL corpus: std.resources defines Filesystem with
    // capabilities, and std.patterns defines `file_content_matches` pattern.
    // The test func calls the pattern and we verify the expanded nodes
    // appear in the lowered DAG.
    //
    // Note: Filesystem.read is a resource capability, not a service operation.
    // Resource capabilities are lowered as callable nodes (not transport triplets).
    // The pattern expansion creates a compare node for eq() and wires
    // the param source for the expected value.
    let dag = lower_typed_project_for_module_with_dependency_closure_and_entry(
        "test.pattern_caller",
        &[(
            "dsl/test/pattern_caller.dag",
            r#"module test.pattern_caller
import std.resources { Filesystem }
import std.patterns { file_content_matches }

func check_file(path: String, expected: String) -> { matches: Bool }
  uses fs: Filesystem(mode: Read)
{
  result = file_content_matches(path: path, expected: expected)
  return { matches: result.matches }
}
"#,
        )],
    );

    let node_ids: Vec<&str> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();

    // The pattern callable node should be present.
    let has_pattern = node_ids
        .iter()
        .any(|id| id.contains("file_content_matches"));

    // A CompareEquality node for the eq() call.
    let has_compare = node_ids.iter().any(|id| id.contains("compare_"));

    // The caller func node.
    let has_caller = node_ids
        .iter()
        .any(|id| id.contains("test.pattern_caller::check_file"));

    // The param source for expected.
    let has_param_expected = node_ids
        .iter()
        .any(|id| id.contains("param_source_") && id.contains("expected"));

    // Resource lifecycle nodes for Filesystem.
    let has_fs_acquire = node_ids
        .iter()
        .any(|id| id.contains("acquire_resource_") && id.contains("Filesystem"));

    assert!(
        has_pattern,
        "expected file_content_matches pattern node; found: {:?}",
        node_ids
    );
    assert!(
        has_compare,
        "expected a compare_ node for eq() call; found: {:?}",
        node_ids
    );
    assert!(
        has_caller,
        "expected test.pattern_caller::check_file node; found: {:?}",
        node_ids
    );
    assert!(
        has_param_expected,
        "expected param_source node for expected; found: {:?}",
        node_ids
    );
    assert!(
        has_fs_acquire,
        "expected acquire_resource node for Filesystem; found: {:?}",
        node_ids
    );
}

#[test]
fn expand_non_generic_pattern_wires_internal_edges_correctly() {
    // Verifies that the pattern expansion creates correct structural nodes
    // and dependency edges. Resource capabilities (Filesystem.read) are lowered
    // as callable nodes, not transport triplets.
    let dag = lower_typed_project_for_module_with_dependency_closure_and_entry(
        "test.pattern_wiring",
        &[(
            "dsl/test/pattern_wiring.dag",
            r#"module test.pattern_wiring
import std.resources { Filesystem }
import std.patterns { file_content_matches }

func verify(path: String, expected: String) -> { ok: Bool }
  uses fs: Filesystem(mode: Read)
{
  result = file_content_matches(path: path, expected: expected)
  return { ok: result.matches }
}
"#,
        )],
    );

    let node_ids: Vec<&str> = dag.nodes.iter().map(|n| n.id.0.as_str()).collect();
    let edges: Vec<(&str, &str, &str, &str)> = dag
        .edges
        .iter()
        .map(|e| {
            (
                e.from_node.0.as_str(),
                e.from_port.0.as_str(),
                e.to_node.0.as_str(),
                e.to_port.0.as_str(),
            )
        })
        .collect();

    // The compare node should exist (for eq() call).
    let compare = node_ids
        .iter()
        .find(|id| id.contains("compare_"))
        .expect("should have compare node");

    // The pattern callable node should exist.
    let has_pattern = node_ids
        .iter()
        .any(|id| id.contains("file_content_matches"));
    assert!(
        has_pattern,
        "expected file_content_matches pattern node; found: {:?}",
        node_ids
    );

    // The caller func should exist.
    let has_caller = node_ids
        .iter()
        .any(|id| id.contains("test.pattern_wiring::verify"));
    assert!(
        has_caller,
        "expected test.pattern_wiring::verify node; found: {:?}",
        node_ids
    );

    // Verify resource lifecycle: Filesystem acquire node exists and
    // wires to the caller via __deps.
    let has_fs_acquire = node_ids
        .iter()
        .any(|id| id.contains("acquire_resource_") && id.contains("Filesystem"));
    assert!(
        has_fs_acquire,
        "expected acquire_resource node for Filesystem; found: {:?}",
        node_ids
    );

    // The compare node should exist in the edge graph.
    let compare_has_edges = edges.iter().any(|(_, _, to, _)| to == compare);
    assert!(
        compare_has_edges,
        "expected edges wiring to compare node; edges: {:?}",
        edges
    );
}

#[test]
fn missing_transport_annotation_returns_error() {
    // A partially-specified service: one operation has transport,
    // another does not. This catches the bug where a developer adds
    // a new operation but forgets the transport block.
    // Fully-abstract services (no transport on any operation) are
    // exempt because they get transport via profile bindings.
    let typed = typed_project_from_sources(&[(
        "dsl/services/no_transport.dag",
        r#"module sample.no_transport
service sample.NoTransport {
  config { endpoint: "https://api.example.com" }
  operation Fetch {
input { id: String }
output { name: String }
  }
  operation List {
input { page: Int }
output { names: List<String> }
transport rest { method: GET, path: "/items" }
  }
}
func caller(id: String) -> { name: String } {
  result = sample.NoTransport.Fetch(id: id)
  return { name: result.name }
}"#,
    )]);

    let err = lower_typed_project(&typed).expect_err(
        "lowering a partially-specified service should fail (RT4)",
    );
    let msg = err.to_string();
    assert!(
        msg.contains("NoTransport") && msg.contains("Fetch") && msg.contains("no transport"),
        "error should mention service, operation, and missing transport; got: {msg}",
    );
}
