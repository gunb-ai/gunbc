use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use daglang_resolve::{ModuleGraph, ResolvedModule};
use daglang_syntax::ast::SourceFile;
use daglang_syntax::parser;
use gunbc_ir::{Dag, Edge, Node, Port};

const NODE_DISCOVER: &str = "discover_files";
const NODE_PARSE: &str = "parse_all";
const NODE_BUILD: &str = "build_module_graph";
const NODE_REPORT: &str = "report_modules";

const PORT_FILES: &str = "files";
const PORT_PARSED_MODULES: &str = "parsed_modules";
const PORT_MODULE_GRAPH: &str = "module_graph";
const PORT_DIAGNOSTICS: &str = "diagnostics";
const PORT_REPORT: &str = "report";

#[derive(Debug, Clone)]
pub enum CompilerOp {
    DiscoverFiles,
    ParseAll,
    BuildModuleGraph,
    ReportModules,
}

#[derive(Debug)]
pub struct PipelineContext {
    pub roots: Vec<PathBuf>,
    pub target_file: Option<PathBuf>,
}

#[derive(Debug)]
pub enum PipelineStop {
    Parse,
    Report,
}

#[derive(Debug)]
pub struct PipelineResult {
    pub diagnostics: Vec<String>,
    pub parsed_count: usize,
    pub module_graph: Option<ModuleGraph>,
    pub report: Option<String>,
}

#[derive(Debug)]
pub struct FileSource {
    pub path: PathBuf,
    pub source: String,
}

#[derive(Debug)]
pub struct ParsedModule {
    pub path: PathBuf,
    pub module_path: Vec<String>,
    pub imports: Vec<Vec<String>>,
    pub ast: SourceFile,
}

#[derive(Debug)]
enum PipeValue {
    Files(Vec<FileSource>),
    ParsedModules(Vec<ParsedModule>),
    ModuleGraph(ModuleGraph),
    Diagnostics(Vec<String>),
    Report(String),
}

pub fn build_pipeline_dag() -> Dag<CompilerOp> {
    let mut dag = Dag::new();

    dag.add_node(Node::opaque(
        NODE_DISCOVER,
        vec![],
        vec![
            Port::new(PORT_FILES, "Vec<FileSource>"),
            Port::new(PORT_DIAGNOSTICS, "Vec<Diagnostic>"),
        ],
        CompilerOp::DiscoverFiles,
    ));
    dag.add_node(Node::opaque(
        NODE_PARSE,
        vec![
            Port::new(PORT_FILES, "Vec<FileSource>"),
            Port::new(PORT_DIAGNOSTICS, "Vec<Diagnostic>"),
        ],
        vec![
            Port::new(PORT_PARSED_MODULES, "Vec<ParsedModule>"),
            Port::new(PORT_DIAGNOSTICS, "Vec<Diagnostic>"),
        ],
        CompilerOp::ParseAll,
    ));
    dag.add_node(Node::opaque(
        NODE_BUILD,
        vec![
            Port::new(PORT_PARSED_MODULES, "Vec<ParsedModule>"),
            Port::new(PORT_DIAGNOSTICS, "Vec<Diagnostic>"),
        ],
        vec![
            Port::new(PORT_MODULE_GRAPH, "ModuleGraph"),
            Port::new(PORT_DIAGNOSTICS, "Vec<Diagnostic>"),
        ],
        CompilerOp::BuildModuleGraph,
    ));
    dag.add_node(Node::opaque(
        NODE_REPORT,
        vec![
            Port::new(PORT_MODULE_GRAPH, "ModuleGraph"),
            Port::new(PORT_DIAGNOSTICS, "Vec<Diagnostic>"),
        ],
        vec![
            Port::new(PORT_REPORT, "String"),
            Port::new(PORT_DIAGNOSTICS, "Vec<Diagnostic>"),
        ],
        CompilerOp::ReportModules,
    ));

    dag.add_edge(Edge::new(NODE_DISCOVER, PORT_FILES, NODE_PARSE, PORT_FILES));
    dag.add_edge(Edge::new(
        NODE_DISCOVER,
        PORT_DIAGNOSTICS,
        NODE_PARSE,
        PORT_DIAGNOSTICS,
    ));
    dag.add_edge(Edge::new(
        NODE_PARSE,
        PORT_PARSED_MODULES,
        NODE_BUILD,
        PORT_PARSED_MODULES,
    ));
    dag.add_edge(Edge::new(
        NODE_PARSE,
        PORT_DIAGNOSTICS,
        NODE_BUILD,
        PORT_DIAGNOSTICS,
    ));
    dag.add_edge(Edge::new(
        NODE_BUILD,
        PORT_MODULE_GRAPH,
        NODE_REPORT,
        PORT_MODULE_GRAPH,
    ));
    dag.add_edge(Edge::new(
        NODE_BUILD,
        PORT_DIAGNOSTICS,
        NODE_REPORT,
        PORT_DIAGNOSTICS,
    ));

    dag
}

pub fn run_pipeline(context: &PipelineContext, stop: PipelineStop) -> Result<PipelineResult, String> {
    let dag = build_pipeline_dag();
    let topo = topological_order(&dag)?;
    let mut values: HashMap<String, PipeValue> = HashMap::new();

    for node_id in topo {
        let node = dag
            .get_node(&node_id.as_str().into())
            .ok_or_else(|| format!("missing node in pipeline DAG: {node_id}"))?;
        let mut inputs = HashMap::new();

        for edge in dag.edges.iter().filter(|edge| edge.to_node.0 == node_id) {
            let key = edge_key(&edge.from_node.0, &edge.from_port.0);
            let value = values
                .remove(&key)
                .ok_or_else(|| format!("missing pipeline value for edge {key}"))?;
            inputs.insert(edge.to_port.0.clone(), value);
        }

        let outputs = execute_op(&node.body, context, inputs)?;
        for (port, value) in outputs {
            values.insert(edge_key(&node_id, &port), value);
        }

        match (&stop, node_id.as_str()) {
            (PipelineStop::Parse, NODE_PARSE) | (PipelineStop::Report, NODE_REPORT) => break,
            _ => {}
        }
    }

    let diagnostics = take_diagnostics(&mut values)?;
    let parsed_count = match values.remove(&edge_key(NODE_PARSE, PORT_PARSED_MODULES)) {
        Some(PipeValue::ParsedModules(parsed)) => parsed.len(),
        _ => 0,
    };
    let module_graph = match values.remove(&edge_key(NODE_BUILD, PORT_MODULE_GRAPH)) {
        Some(PipeValue::ModuleGraph(graph)) => Some(graph),
        _ => None,
    };
    let report = match values.remove(&edge_key(NODE_REPORT, PORT_REPORT)) {
        Some(PipeValue::Report(report)) => Some(report),
        _ => None,
    };

    Ok(PipelineResult {
        diagnostics,
        parsed_count,
        module_graph,
        report,
    })
}

fn execute_op(
    body: &gunbc_ir::node::NodeBody<CompilerOp>,
    context: &PipelineContext,
    mut inputs: HashMap<String, PipeValue>,
) -> Result<HashMap<String, PipeValue>, String> {
    let op = match body {
        gunbc_ir::node::NodeBody::Opaque(op) => op,
        gunbc_ir::node::NodeBody::SubDag(_) => return Err("compiler pipeline uses opaque ops only".into()),
    };

    match op {
        CompilerOp::DiscoverFiles => {
            let files = discover_files(context)?;
            Ok(HashMap::from([
                (PORT_FILES.to_string(), PipeValue::Files(files)),
                (PORT_DIAGNOSTICS.to_string(), PipeValue::Diagnostics(Vec::new())),
            ]))
        }
        CompilerOp::ParseAll => {
            let files = take_files(&mut inputs)?;
            let mut diagnostics = take_diagnostics_from_inputs(&mut inputs);
            let mut parsed = Vec::new();

            for file in files {
                match parser::parse(&file.source) {
                    Ok(ast) => {
                        let module_path = ast
                            .module_path
                            .as_ref()
                            .map(|module| module.node.segments.clone())
                            .unwrap_or_else(|| {
                                path_to_module_path(&file.path, &context.roots)
                            });
                        let imports = ast
                            .imports
                            .iter()
                            .map(|import| import.node.path.segments.clone())
                            .collect();
                        parsed.push(ParsedModule {
                            path: file.path,
                            module_path,
                            imports,
                            ast,
                        });
                    }
                    Err(errors) => {
                        diagnostics.extend(
                            errors
                                .iter()
                                .map(|error| error.format_with_source(&file.path, &file.source)),
                        );
                    }
                }
            }

            Ok(HashMap::from([
                (
                    PORT_PARSED_MODULES.to_string(),
                    PipeValue::ParsedModules(parsed),
                ),
                (
                    PORT_DIAGNOSTICS.to_string(),
                    PipeValue::Diagnostics(diagnostics),
                ),
            ]))
        }
        CompilerOp::BuildModuleGraph => {
            let parsed_modules = take_parsed_modules(&mut inputs)?;
            let mut diagnostics = take_diagnostics_from_inputs(&mut inputs);
            let graph = build_module_graph(parsed_modules, &mut diagnostics);

            Ok(HashMap::from([
                (PORT_MODULE_GRAPH.to_string(), PipeValue::ModuleGraph(graph)),
                (
                    PORT_DIAGNOSTICS.to_string(),
                    PipeValue::Diagnostics(diagnostics),
                ),
            ]))
        }
        CompilerOp::ReportModules => {
            let graph = take_module_graph(&mut inputs)?;
            let diagnostics = take_diagnostics_from_inputs(&mut inputs);
            let mut report = String::new();
            report.push_str("Discovered modules:\n");
            report.push_str(&graph.display_tree());
            if !diagnostics.is_empty() {
                report.push_str("\nDiagnostics:\n");
                for diagnostic in &diagnostics {
                    report.push_str(&format!("  {diagnostic}\n"));
                }
            }

            Ok(HashMap::from([
                (PORT_REPORT.to_string(), PipeValue::Report(report)),
                (
                    PORT_DIAGNOSTICS.to_string(),
                    PipeValue::Diagnostics(diagnostics),
                ),
            ]))
        }
    }
}

fn discover_files(context: &PipelineContext) -> Result<Vec<FileSource>, String> {
    if let Some(target_file) = &context.target_file {
        let source = fs::read_to_string(target_file)
            .map_err(|error| format!("failed to read {}: {error}", target_file.display()))?;
        return Ok(vec![FileSource {
            path: target_file.clone(),
            source,
        }]);
    }

    let mut dag_files = Vec::new();
    for root in &context.roots {
        collect_dag_files(root, &mut dag_files)
            .map_err(|error| format!("failed to collect .dag files in {}: {error}", root.display()))?;
    }
    dag_files.sort();

    let mut out = Vec::new();
    for path in dag_files {
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        out.push(FileSource { path, source });
    }
    Ok(out)
}

fn collect_dag_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("dag") {
            out.push(path);
        }
    }
    Ok(())
}

fn path_to_module_path(path: &Path, roots: &[PathBuf]) -> Vec<String> {
    for root in roots {
        if let Ok(relative) = path.strip_prefix(root) {
            return relative
                .with_extension("")
                .components()
                .filter_map(|segment| segment.as_os_str().to_str().map(String::from))
                .collect();
        }
    }
    vec![
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("unknown")
            .to_string(),
    ]
}

fn build_module_graph(parsed_modules: Vec<ParsedModule>, diagnostics: &mut Vec<String>) -> ModuleGraph {
    let mut modules = Vec::new();
    let mut module_index = HashMap::new();
    let mut seen_duplicates = HashSet::new();

    for (idx, module) in parsed_modules.iter().enumerate() {
        match module_index.insert(module.module_path.clone(), idx) {
            Some(_) => {
                if seen_duplicates.insert(module.module_path.join(".")) {
                    diagnostics.push(format!(
                        "{}: duplicate module path {}",
                        module.path.display(),
                        module.module_path.join(".")
                    ));
                }
            }
            None => {}
        }
    }

    for (idx, module) in parsed_modules.into_iter().enumerate() {
        if module_index
            .get(&module.module_path)
            .is_some_and(|winner| *winner != idx)
        {
            continue;
        }
        let dependencies = module
            .imports
            .iter()
            .filter_map(|import| module_index.get(import).copied())
            .collect::<Vec<_>>();
        modules.push(ResolvedModule {
            path: module.path,
            ast: module.ast,
            module_path: module.module_path,
            dependencies,
        });
    }

    topo_sort_modules(&mut modules);
    ModuleGraph { modules }
}

fn topo_sort_modules(modules: &mut Vec<ResolvedModule>) {
    let n = modules.len();
    let mut in_degree = vec![0usize; n];
    let mut adjacency = vec![Vec::new(); n];

    for (idx, module) in modules.iter().enumerate() {
        for dependency in &module.dependencies {
            if *dependency < n {
                adjacency[*dependency].push(idx);
                in_degree[idx] += 1;
            }
        }
    }

    let mut queue: Vec<usize> = (0..n).filter(|idx| in_degree[*idx] == 0).collect();
    queue.sort_unstable();
    let mut order = Vec::with_capacity(n);

    while let Some(current) = queue.pop() {
        order.push(current);
        for next in &adjacency[current] {
            in_degree[*next] -= 1;
            if in_degree[*next] == 0 {
                queue.push(*next);
            }
        }
        queue.sort_unstable();
    }

    if order.len() != n {
        let mut seen = vec![false; n];
        for idx in &order {
            seen[*idx] = true;
        }
        for idx in 0..n {
            if !seen[idx] {
                order.push(idx);
            }
        }
    }

    let mut drained: Vec<Option<ResolvedModule>> = modules.drain(..).map(Some).collect();
    for idx in order {
        modules.push(drained[idx].take().expect("module should exist"));
    }
}

fn topological_order(dag: &Dag<CompilerOp>) -> Result<Vec<String>, String> {
    let mut node_ids: Vec<String> = dag.nodes.iter().map(|node| node.id.0.clone()).collect();
    node_ids.sort();
    let mut in_degree: HashMap<String, usize> =
        node_ids.iter().map(|node| (node.clone(), 0usize)).collect();
    let mut adjacency: HashMap<String, Vec<String>> =
        node_ids.iter().map(|node| (node.clone(), Vec::new())).collect();

    for edge in &dag.edges {
        *in_degree
            .get_mut(&edge.to_node.0)
            .ok_or_else(|| format!("edge targets missing node {}", edge.to_node.0))? += 1;
        adjacency
            .get_mut(&edge.from_node.0)
            .ok_or_else(|| format!("edge source missing node {}", edge.from_node.0))?
            .push(edge.to_node.0.clone());
    }

    let mut ready: Vec<String> = in_degree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(node, _)| node.clone())
        .collect();
    ready.sort();
    let mut order = Vec::new();

    while let Some(node) = ready.pop() {
        order.push(node.clone());
        if let Some(neighbors) = adjacency.get(&node) {
            for neighbor in neighbors {
                let degree = in_degree
                    .get_mut(neighbor)
                    .ok_or_else(|| format!("missing in-degree entry for {neighbor}"))?;
                *degree -= 1;
                if *degree == 0 {
                    ready.push(neighbor.clone());
                }
            }
        }
        ready.sort();
    }

    if order.len() != dag.nodes.len() {
        return Err("compiler pipeline DAG contains a cycle".into());
    }
    Ok(order)
}

fn take_files(inputs: &mut HashMap<String, PipeValue>) -> Result<Vec<FileSource>, String> {
    match inputs.remove(PORT_FILES) {
        Some(PipeValue::Files(files)) => Ok(files),
        Some(other) => Err(format!("expected files input, found {other:?}")),
        None => Err("missing files input".into()),
    }
}

fn take_parsed_modules(inputs: &mut HashMap<String, PipeValue>) -> Result<Vec<ParsedModule>, String> {
    match inputs.remove(PORT_PARSED_MODULES) {
        Some(PipeValue::ParsedModules(parsed)) => Ok(parsed),
        Some(other) => Err(format!("expected parsed modules input, found {other:?}")),
        None => Err("missing parsed modules input".into()),
    }
}

fn take_module_graph(inputs: &mut HashMap<String, PipeValue>) -> Result<ModuleGraph, String> {
    match inputs.remove(PORT_MODULE_GRAPH) {
        Some(PipeValue::ModuleGraph(graph)) => Ok(graph),
        Some(other) => Err(format!("expected module graph input, found {other:?}")),
        None => Err("missing module graph input".into()),
    }
}

fn take_diagnostics_from_inputs(inputs: &mut HashMap<String, PipeValue>) -> Vec<String> {
    match inputs.remove(PORT_DIAGNOSTICS) {
        Some(PipeValue::Diagnostics(diagnostics)) => diagnostics,
        _ => Vec::new(),
    }
}

fn take_diagnostics(values: &mut HashMap<String, PipeValue>) -> Result<Vec<String>, String> {
    if let Some(PipeValue::Diagnostics(diagnostics)) =
        values.remove(&edge_key(NODE_REPORT, PORT_DIAGNOSTICS))
    {
        return Ok(diagnostics);
    }
    if let Some(PipeValue::Diagnostics(diagnostics)) =
        values.remove(&edge_key(NODE_BUILD, PORT_DIAGNOSTICS))
    {
        return Ok(diagnostics);
    }
    if let Some(PipeValue::Diagnostics(diagnostics)) =
        values.remove(&edge_key(NODE_PARSE, PORT_DIAGNOSTICS))
    {
        return Ok(diagnostics);
    }
    if let Some(PipeValue::Diagnostics(diagnostics)) =
        values.remove(&edge_key(NODE_DISCOVER, PORT_DIAGNOSTICS))
    {
        return Ok(diagnostics);
    }
    Err("missing diagnostics output in pipeline".into())
}

fn edge_key(node: &str, port: &str) -> String {
    format!("{node}.{port}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "daglang_cli_pipeline_{name}_{}_{}",
            std::process::id(),
            nanos
        ))
    }

    #[test]
    fn compiler_pipeline_has_expected_dag_shape() {
        let dag = build_pipeline_dag();
        assert_eq!(dag.nodes.len(), 4);
        assert_eq!(dag.edges.len(), 6);

        let ids: Vec<String> = dag.nodes.iter().map(|node| node.id.0.clone()).collect();
        assert!(ids.iter().any(|id| id == NODE_DISCOVER));
        assert!(ids.iter().any(|id| id == NODE_PARSE));
        assert!(ids.iter().any(|id| id == NODE_BUILD));
        assert!(ids.iter().any(|id| id == NODE_REPORT));
    }

    #[test]
    fn parse_op_rejects_wrong_pipe_value_variant() {
        let context = PipelineContext {
            roots: vec![],
            target_file: None,
        };
        let mut inputs = HashMap::new();
        inputs.insert(
            PORT_FILES.to_string(),
            PipeValue::Diagnostics(vec!["not files".into()]),
        );
        inputs.insert(
            PORT_DIAGNOSTICS.to_string(),
            PipeValue::Diagnostics(Vec::new()),
        );

        let err = execute_op(
            &gunbc_ir::node::NodeBody::Opaque(CompilerOp::ParseAll),
            &context,
            inputs,
        )
        .expect_err("expected parse op to reject wrong input variant");
        assert!(err.contains("expected files input"));
    }

    #[test]
    fn parse_pipeline_collects_diagnostics_for_invalid_source() {
        let root = unique_temp_dir("invalid");
        fs::create_dir_all(&root).expect("failed to create temp root");
        fs::write(root.join("broken.dag"), "module broken\nfn x( -> Unit {")
            .expect("failed to write invalid source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Parse).expect("pipeline should execute");
        assert!(
            !result.diagnostics.is_empty(),
            "invalid source should produce diagnostics"
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.contains("broken.dag:2:")),
            "diagnostics should contain file:line:col locations"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn parse_pipeline_aggregates_diagnostics_across_multiple_invalid_files() {
        let root = unique_temp_dir("multi_invalid");
        fs::create_dir_all(&root).expect("failed to create temp root");
        fs::write(root.join("broken_a.dag"), "module broken.a\nfn")
            .expect("failed to write invalid source A");
        fs::write(root.join("broken_b.dag"), "module broken.b\nimport")
            .expect("failed to write invalid source B");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Parse).expect("pipeline should execute");

        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.contains("broken_a.dag")),
            "expected diagnostics to include broken_a.dag"
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.contains("broken_b.dag")),
            "expected diagnostics to include broken_b.dag"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn target_file_mode_limits_pipeline_input_to_single_file() {
        let root = unique_temp_dir("target_file_mode");
        fs::create_dir_all(&root).expect("failed to create temp root");
        fs::write(root.join("good.dag"), "module sample.good\nfn ok() -> Unit {}")
            .expect("failed to write valid source");
        fs::write(root.join("broken.dag"), "module sample.broken\nfn")
            .expect("failed to write invalid source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: Some(root.join("good.dag")),
        };
        let result = run_pipeline(&context, PipelineStop::Parse).expect("pipeline should execute");

        assert_eq!(result.parsed_count, 1, "target file mode should parse one file");
        assert!(
            result.diagnostics.is_empty(),
            "target file mode should ignore sibling invalid files"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }
}
