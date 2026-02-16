use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use daglang_resolve::{ModuleGraph, ResolvedModule};
use daglang_syntax::ast::SourceFile;
use daglang_syntax::diagnostic::{Diagnostic, DiagnosticKind};
use daglang_syntax::parser;
use gunbc_ir::{Dag, Edge, Node, Port};
use gunbc_ir::types::Cardinality;

const NODE_DISCOVER: &str = "discover_files";
const NODE_PARSE: &str = "parse_all";
const NODE_BUILD: &str = "build_module_graph";
const NODE_REPORT: &str = "report_modules";

const PORT_CONTEXT: &str = "context";
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
    pub diagnostics: Vec<Diagnostic>,
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
    Diagnostics(Vec<Diagnostic>),
    Report(String),
}

pub fn build_pipeline_dag() -> Dag<CompilerOp> {
    let mut dag = Dag::new();

    dag.add_node(Node::opaque(
        NODE_DISCOVER,
        vec![Port::with_cardinality(
            PORT_CONTEXT,
            "PipelineContext",
            Cardinality::ONE,
        )],
        vec![
            Port::with_cardinality(PORT_FILES, "Vec<FileSource>", Cardinality::ONE),
            Port::with_cardinality(PORT_DIAGNOSTICS, "Vec<Diagnostic>", Cardinality::ONE),
        ],
        CompilerOp::DiscoverFiles,
    ));
    dag.add_node(Node::opaque(
        NODE_PARSE,
        vec![
            Port::with_cardinality(PORT_FILES, "Vec<FileSource>", Cardinality::ONE),
            Port::with_cardinality(PORT_DIAGNOSTICS, "Vec<Diagnostic>", Cardinality::ONE),
        ],
        vec![
            Port::with_cardinality(PORT_PARSED_MODULES, "Vec<ParsedModule>", Cardinality::ONE),
            Port::with_cardinality(PORT_DIAGNOSTICS, "Vec<Diagnostic>", Cardinality::ONE),
        ],
        CompilerOp::ParseAll,
    ));
    dag.add_node(Node::opaque(
        NODE_BUILD,
        vec![
            Port::with_cardinality(PORT_PARSED_MODULES, "Vec<ParsedModule>", Cardinality::ONE),
            Port::with_cardinality(PORT_DIAGNOSTICS, "Vec<Diagnostic>", Cardinality::ONE),
        ],
        vec![
            Port::with_cardinality(PORT_MODULE_GRAPH, "ModuleGraph", Cardinality::ONE),
            Port::with_cardinality(PORT_DIAGNOSTICS, "Vec<Diagnostic>", Cardinality::ONE),
        ],
        CompilerOp::BuildModuleGraph,
    ));
    dag.add_node(Node::opaque(
        NODE_REPORT,
        vec![
            Port::with_cardinality(PORT_MODULE_GRAPH, "ModuleGraph", Cardinality::ONE),
            Port::with_cardinality(PORT_DIAGNOSTICS, "Vec<Diagnostic>", Cardinality::ONE),
        ],
        vec![
            Port::with_cardinality(PORT_REPORT, "String", Cardinality::ONE),
            Port::with_cardinality(PORT_DIAGNOSTICS, "Vec<Diagnostic>", Cardinality::ONE),
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
    validate_pipeline_semantics(&dag)?;
    let topo = topological_order(&dag)?;
    let mut values: HashMap<String, PipeValue> = HashMap::new();
    let mut parsed_count = 0usize;

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
        if node_id == NODE_PARSE {
            if let Some(PipeValue::ParsedModules(parsed)) = outputs.get(PORT_PARSED_MODULES) {
                parsed_count = parsed.len();
            }
        }
        for (port, value) in outputs {
            values.insert(edge_key(&node_id, &port), value);
        }

        match (&stop, node_id.as_str()) {
            (PipelineStop::Parse, NODE_PARSE) | (PipelineStop::Report, NODE_REPORT) => break,
            _ => {}
        }
    }

    let diagnostics = take_diagnostics(&mut values)?;
    let parsed_count_from_values = match values.remove(&edge_key(NODE_PARSE, PORT_PARSED_MODULES)) {
        Some(PipeValue::ParsedModules(parsed)) => parsed.len(),
        _ => 0,
    };
    let parsed_count = parsed_count.max(parsed_count_from_values);
    let module_graph = match values.remove(&edge_key(NODE_BUILD, PORT_MODULE_GRAPH)) {
        Some(PipeValue::ModuleGraph(graph)) => Some(graph),
        _ => None,
    };
    let report = match values.remove(&edge_key(NODE_REPORT, PORT_REPORT)) {
        Some(PipeValue::Report(report)) => Some(report),
        _ => None,
    };

    Ok(PipelineResult {
        diagnostics: normalize_diagnostics(diagnostics),
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
            let canonical_roots = canonicalize_roots(&context.roots);

            for file in files {
                match parser::parse_with_file_diagnostics(&file.path, &file.source) {
                    Ok(ast) => {
                        let module_path = ast
                            .module_path
                            .as_ref()
                            .map(|module| module.node.segments.clone())
                            .unwrap_or_else(|| {
                                path_to_module_path(&file.path, &context.roots, &canonical_roots)
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
                    Err(file_diagnostics) => {
                        diagnostics.extend(file_diagnostics);
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
                    PipeValue::Diagnostics(normalize_diagnostics(diagnostics)),
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
                    PipeValue::Diagnostics(normalize_diagnostics(diagnostics)),
                ),
            ]))
        }
        CompilerOp::ReportModules => {
            let graph = take_module_graph(&mut inputs)?;
            let diagnostics = normalize_diagnostics(take_diagnostics_from_inputs(&mut inputs));
            let mut report = String::new();
            report.push_str("Discovered modules:\n");
            report.push_str(&graph.display_tree());
            if !diagnostics.is_empty() {
                report.push_str("\nDiagnostics:\n");
                for diagnostic in &diagnostics {
                    report.push_str(&format!("  {}\n", diagnostic.render()));
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
    let mut visited_dirs = HashSet::new();
    for root in &context.roots {
        if !root.exists() {
            return Err(format!("input root does not exist: {}", root.display()));
        }
        if !root.is_dir() {
            return Err(format!("input root is not a directory: {}", root.display()));
        }
        collect_dag_files(root, &mut dag_files, &mut visited_dirs)
            .map_err(|error| format!("failed to collect .dag files in {}: {error}", root.display()))?;
    }
    let mut canonical_dag_files = Vec::with_capacity(dag_files.len());
    for path in dag_files {
        let canonical = fs::canonicalize(&path)
            .map_err(|error| format!("failed to canonicalize {}: {error}", path.display()))?;
        canonical_dag_files.push(canonical);
    }
    dag_files = canonical_dag_files;
    dag_files.sort();
    dag_files.dedup();

    let mut out = Vec::new();
    for path in dag_files {
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        out.push(FileSource { path, source });
    }
    Ok(out)
}

fn collect_dag_files(
    dir: &Path,
    out: &mut Vec<PathBuf>,
    visited_dirs: &mut HashSet<PathBuf>,
) -> std::io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let canonical_dir = fs::canonicalize(dir)?;
    if !visited_dirs.insert(canonical_dir) {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files(&path, out, visited_dirs)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("dag") {
            out.push(path);
        }
    }
    Ok(())
}

fn relative_path_to_module_path(relative: &Path) -> Vec<String> {
    relative
        .with_extension("")
        .components()
        .filter_map(|segment| segment.as_os_str().to_str().map(String::from))
        .collect()
}

fn choose_preferred_relative(
    current: Option<PathBuf>,
    candidate: &Path,
) -> Option<PathBuf> {
    let candidate_buf = candidate.to_path_buf();
    match current {
        None => Some(candidate_buf),
        Some(existing) => {
            let existing_depth = existing.components().count();
            let candidate_depth = candidate_buf.components().count();
            if candidate_depth < existing_depth {
                return Some(candidate_buf);
            }
            if candidate_depth > existing_depth {
                return Some(existing);
            }

            let existing_key = existing.as_os_str().to_string_lossy();
            let candidate_key = candidate_buf.as_os_str().to_string_lossy();
            if candidate_key < existing_key {
                Some(candidate_buf)
            } else {
                Some(existing)
            }
        }
    }
}

fn canonicalize_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    roots
        .iter()
        .filter_map(|root| fs::canonicalize(root).ok())
        .collect()
}

fn path_to_module_path(path: &Path, roots: &[PathBuf], canonical_roots: &[PathBuf]) -> Vec<String> {
    let canonical_path = fs::canonicalize(path).ok();
    let mut best_relative: Option<PathBuf> = None;
    for root in roots {
        if let Ok(relative) = path.strip_prefix(root) {
            best_relative = choose_preferred_relative(best_relative, relative);
        }
    }
    for canonical_root in canonical_roots {
        if let Ok(relative) = path.strip_prefix(canonical_root) {
            best_relative = choose_preferred_relative(best_relative, relative);
        }
        if let Some(canonical_path) = &canonical_path {
            if let Ok(relative) = canonical_path.strip_prefix(canonical_root) {
                best_relative = choose_preferred_relative(best_relative, relative);
            }
        }
    }
    if let Some(relative) = best_relative {
        return relative_path_to_module_path(&relative);
    }
    vec![
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("unknown")
            .to_string(),
    ]
}

fn build_module_graph(
    parsed_modules: Vec<ParsedModule>,
    diagnostics: &mut Vec<Diagnostic>,
) -> ModuleGraph {
    let mut unique_modules: Vec<ParsedModule> = Vec::new();
    let mut module_index: HashMap<Vec<String>, usize> = HashMap::new();
    let mut seen_duplicates = HashSet::new();

    for module in parsed_modules {
        match module_index.get(&module.module_path).copied() {
            Some(_) => {
                if seen_duplicates.insert(module.module_path.join(".")) {
                    diagnostics.push(
                        Diagnostic::new(
                            DiagnosticKind::Resolve,
                            format!("duplicate module path {}", module.module_path.join(".")),
                        )
                        .with_file(&module.path),
                    );
                }
            }
            None => {
                let idx = unique_modules.len();
                module_index.insert(module.module_path.clone(), idx);
                unique_modules.push(module);
            }
        }
    }

    let mut modules = Vec::with_capacity(unique_modules.len());
    for module in unique_modules {
        let mut dependencies = Vec::new();
        for import in &module.imports {
            if let Some(dep_idx) = module_index.get(import).copied() {
                dependencies.push(dep_idx);
            } else {
                diagnostics.push(
                    Diagnostic::new(
                        DiagnosticKind::Resolve,
                        format!(
                            "unresolved import: {} -> {}",
                            module.module_path.join("."),
                            import.join(".")
                        ),
                    )
                    .with_file(&module.path),
                );
            }
        }
        modules.push(ResolvedModule {
            path: module.path,
            ast: module.ast,
            module_path: module.module_path,
            dependencies,
        });
    }

    let cycle_modules = topo_sort_modules(&mut modules);
    if !cycle_modules.is_empty() {
        diagnostics.push(Diagnostic::new(
            DiagnosticKind::Resolve,
            format!(
                "cyclic dependencies detected among modules: {}",
                cycle_modules
                    .into_iter()
                    .map(|module| module.join("."))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ));
    }
    ModuleGraph { modules }
}

fn topo_sort_modules(modules: &mut Vec<ResolvedModule>) -> Vec<Vec<String>> {
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

    let mut cycle_modules = Vec::new();
    if order.len() != n {
        let mut seen = vec![false; n];
        for idx in &order {
            seen[*idx] = true;
        }
        for idx in 0..n {
            if !seen[idx] {
                cycle_modules.push(modules[idx].module_path.clone());
                order.push(idx);
            }
        }
    }

    let mut old_to_new = vec![0usize; n];
    for (new_idx, old_idx) in order.iter().enumerate() {
        old_to_new[*old_idx] = new_idx;
    }

    let mut drained: Vec<Option<ResolvedModule>> = modules.drain(..).map(Some).collect();
    for old_idx in order {
        let mut module = drained[old_idx].take().expect("module should exist");
        for dep in &mut module.dependencies {
            if *dep < n {
                *dep = old_to_new[*dep];
            }
        }
        modules.push(module);
    }
    cycle_modules
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

fn validate_pipeline_semantics(dag: &Dag<CompilerOp>) -> Result<(), String> {
    let mut seen_node_ids: HashSet<String> = HashSet::new();
    for node in &dag.nodes {
        if !seen_node_ids.insert(node.id.0.clone()) {
            return Err(format!("duplicate node id is not allowed: {}", node.id.0));
        }

        let mut seen_input_ports: HashSet<String> = HashSet::new();
        for input in &node.inputs {
            if !seen_input_ports.insert(input.name.0.clone()) {
                return Err(format!(
                    "duplicate input port is not allowed on node {}: {}",
                    node.id.0, input.name.0
                ));
            }
        }

        let mut seen_output_ports: HashSet<String> = HashSet::new();
        for output in &node.outputs {
            if !seen_output_ports.insert(output.name.0.clone()) {
                return Err(format!(
                    "duplicate output port is not allowed on node {}: {}",
                    node.id.0, output.name.0
                ));
            }
        }
    }

    let node_by_id: HashMap<String, &Node<CompilerOp>> =
        dag.nodes.iter().map(|node| (node.id.0.clone(), node)).collect();
    if !node_by_id.contains_key(NODE_DISCOVER) {
        return Err(format!(
            "missing required entrypoint node {}",
            NODE_DISCOVER
        ));
    }
    let mut occupied_inputs: HashSet<(String, String)> = HashSet::new();
    let mut incoming_by_node: HashMap<String, usize> = HashMap::new();
    for edge in &dag.edges {
        let from_node = node_by_id
            .get(&edge.from_node.0)
            .ok_or_else(|| format!("edge source node does not exist: {}", edge.from_node.0))?;
        if !from_node
            .outputs
            .iter()
            .any(|port| port.name.0 == edge.from_port.0)
        {
            return Err(format!(
                "edge source port does not exist: {}.{}",
                edge.from_node.0, edge.from_port.0
            ));
        }

        let to_node = node_by_id
            .get(&edge.to_node.0)
            .ok_or_else(|| format!("edge target node does not exist: {}", edge.to_node.0))?;
        if !to_node.inputs.iter().any(|port| port.name.0 == edge.to_port.0) {
            return Err(format!(
                "edge target port does not exist: {}.{}",
                edge.to_node.0, edge.to_port.0
            ));
        }

        *incoming_by_node.entry(edge.to_node.0.clone()).or_insert(0) += 1;
        let key = (edge.to_node.0.clone(), edge.to_port.0.clone());
        if !occupied_inputs.insert(key.clone()) {
            return Err(format!(
                "implicit fan-in is not allowed for input port {}.{}",
                key.0, key.1
            ));
        }
    }

    for node in &dag.nodes {
        let incoming = incoming_by_node.get(&node.id.0).copied().unwrap_or(0);
        if node.id.0 == NODE_DISCOVER && incoming > 0 {
            return Err(format!(
                "entrypoint node {} must not have inbound edges",
                NODE_DISCOVER
            ));
        }
        if incoming == 0 {
            if node.id.0 != NODE_DISCOVER && !node.inputs.is_empty() {
                return Err(format!(
                    "non-entrypoint node {} has unconnected inputs; only {} may define entrypoint inputs",
                    node.id.0, NODE_DISCOVER
                ));
            }
            continue;
        }
        for input in &node.inputs {
            let key = (node.id.0.clone(), input.name.0.clone());
            if !occupied_inputs.contains(&key) {
                return Err(format!(
                    "unconnected input port {}.{} is not allowed on non-entrypoint nodes",
                    key.0, key.1
                ));
            }
        }
    }

    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &dag.edges {
        adjacency
            .entry(edge.from_node.0.clone())
            .or_default()
            .push(edge.to_node.0.clone());
    }

    let mut visited: HashSet<String> = HashSet::new();
    let mut stack = vec![NODE_DISCOVER.to_string()];
    while let Some(current) = stack.pop() {
        if !visited.insert(current.clone()) {
            continue;
        }
        if let Some(neighbors) = adjacency.get(&current) {
            for neighbor in neighbors {
                stack.push(neighbor.clone());
            }
        }
    }

    for node in &dag.nodes {
        if !visited.contains(&node.id.0) {
            return Err(format!(
                "pipeline node {} is unreachable from entrypoint {}",
                node.id.0, NODE_DISCOVER
            ));
        }
    }
    Ok(())
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

fn take_diagnostics_from_inputs(inputs: &mut HashMap<String, PipeValue>) -> Vec<Diagnostic> {
    match inputs.remove(PORT_DIAGNOSTICS) {
        Some(PipeValue::Diagnostics(diagnostics)) => diagnostics,
        _ => Vec::new(),
    }
}

fn take_diagnostics(values: &mut HashMap<String, PipeValue>) -> Result<Vec<Diagnostic>, String> {
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

fn normalize_diagnostics(mut diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
    diagnostics.sort_by_key(|diag| {
        (
            diagnostic_kind_rank(&diag.kind),
            diag.file
                .as_ref()
                .map(|file| file.display().to_string())
                .unwrap_or_default(),
            diag.line.unwrap_or_default(),
            diag.column.unwrap_or_default(),
            diag.message.clone(),
        )
    });
    diagnostics.dedup_by(|a, b| {
        a.kind == b.kind
            && a.file == b.file
            && a.line == b.line
            && a.column == b.column
            && a.message == b.message
    });
    diagnostics
}

fn diagnostic_kind_rank(kind: &DiagnosticKind) -> u8 {
    match kind {
        DiagnosticKind::Lex => 0,
        DiagnosticKind::Parse => 1,
        DiagnosticKind::Resolve => 2,
        DiagnosticKind::Pipeline => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
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

    fn expected_real_corpus_module_order() -> Vec<&'static str> {
        vec![
            "std.types",
            "std.resources",
            "services.shell",
            "tools.codegen",
            "services.github.gist",
            "services.git",
            "services.gcp.sts",
            "services.gcp.secret_manager",
            "services.gcp.iam",
            "std.patterns",
            "tools.testgen",
            "tools.docgen",
            "shared.dag_util",
            "tools.pragma",
            "tools.makegen",
            "tools.deps",
            "tools.bootstrap",
            "services.cargo",
            "tools.clippy",
            "tools.build",
            "pipelines.ci",
            "infra.gcp.services",
            "infra.core",
            "infra.spec",
            "infra.gcp.resources",
            "infra.gcp.config",
            "infra.azure.services",
            "infra.azure.resources",
            "infra.azure.config",
            "infra.aws.services",
            "infra.aws.resources",
            "infra.aws.config",
            "examples.rich_types",
            "examples.integration_tests",
            "examples.abstract_services",
            "cloud.gcp.credential",
            "cloud.azure.credential",
            "cloud.aws.credential",
            "examples.deployment",
            "shared.gist_modes",
            "tools.dag_viz",
            "tools.gist",
        ]
    }

    fn reported_modules_in_order(report: &str) -> Vec<String> {
        report
            .lines()
            .skip_while(|line| !line.starts_with("Discovered modules:"))
            .skip(1)
            .take_while(|line| !line.is_empty())
            .filter_map(|line| line.strip_prefix("  "))
            .filter_map(|line| line.split_once("  (").map(|(module, _)| module.to_string()))
            .collect()
    }

    fn run_parse_and_build_ops(context: &PipelineContext) -> (ModuleGraph, Vec<Diagnostic>) {
        let files = discover_files(context).expect("discovery should succeed");
        let parse_outputs = execute_op(
            &gunbc_ir::node::NodeBody::Opaque(CompilerOp::ParseAll),
            context,
            HashMap::from([
                (PORT_FILES.to_string(), PipeValue::Files(files)),
                (
                    PORT_DIAGNOSTICS.to_string(),
                    PipeValue::Diagnostics(Vec::new()),
                ),
            ]),
        )
        .expect("parse op should succeed");

        let mut parse_outputs = parse_outputs;
        let parsed_modules = match parse_outputs.remove(PORT_PARSED_MODULES) {
            Some(PipeValue::ParsedModules(parsed_modules)) => parsed_modules,
            other => panic!("expected parsed modules output, got {other:?}"),
        };
        let parse_diagnostics = match parse_outputs.remove(PORT_DIAGNOSTICS) {
            Some(PipeValue::Diagnostics(diagnostics)) => diagnostics,
            other => panic!("expected parse diagnostics output, got {other:?}"),
        };

        let build_outputs = execute_op(
            &gunbc_ir::node::NodeBody::Opaque(CompilerOp::BuildModuleGraph),
            context,
            HashMap::from([
                (
                    PORT_PARSED_MODULES.to_string(),
                    PipeValue::ParsedModules(parsed_modules),
                ),
                (
                    PORT_DIAGNOSTICS.to_string(),
                    PipeValue::Diagnostics(parse_diagnostics),
                ),
            ]),
        )
        .expect("build op should succeed");

        let mut build_outputs = build_outputs;
        let graph = match build_outputs.remove(PORT_MODULE_GRAPH) {
            Some(PipeValue::ModuleGraph(graph)) => graph,
            other => panic!("expected module graph output, got {other:?}"),
        };
        let diagnostics = match build_outputs.remove(PORT_DIAGNOSTICS) {
            Some(PipeValue::Diagnostics(diagnostics)) => diagnostics,
            other => panic!("expected build diagnostics output, got {other:?}"),
        };

        (graph, diagnostics)
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

        let discover = dag
            .nodes
            .iter()
            .find(|node| node.id.0 == NODE_DISCOVER)
            .expect("discover node should exist");
        assert_eq!(discover.inputs.len(), 1);
        assert_eq!(discover.inputs[0].name.0, PORT_CONTEXT);
        assert_eq!(discover.inputs[0].cardinality, Cardinality::ONE);

        for node in &dag.nodes {
            for port in node.inputs.iter().chain(node.outputs.iter()) {
                assert_eq!(
                    port.cardinality,
                    Cardinality::ONE,
                    "phase-0 pipeline ports should be scalar by default"
                );
            }
        }

        let expected_edges = HashSet::from([
            (NODE_DISCOVER, PORT_FILES, NODE_PARSE, PORT_FILES),
            (NODE_DISCOVER, PORT_DIAGNOSTICS, NODE_PARSE, PORT_DIAGNOSTICS),
            (
                NODE_PARSE,
                PORT_PARSED_MODULES,
                NODE_BUILD,
                PORT_PARSED_MODULES,
            ),
            (NODE_PARSE, PORT_DIAGNOSTICS, NODE_BUILD, PORT_DIAGNOSTICS),
            (
                NODE_BUILD,
                PORT_MODULE_GRAPH,
                NODE_REPORT,
                PORT_MODULE_GRAPH,
            ),
            (NODE_BUILD, PORT_DIAGNOSTICS, NODE_REPORT, PORT_DIAGNOSTICS),
        ]);
        let actual_edges: HashSet<(&str, &str, &str, &str)> = dag
            .edges
            .iter()
            .map(|edge| {
                (
                    edge.from_node.0.as_str(),
                    edge.from_port.0.as_str(),
                    edge.to_node.0.as_str(),
                    edge.to_port.0.as_str(),
                )
            })
            .collect();
        assert_eq!(actual_edges, expected_edges);
    }

    #[test]
    fn pipeline_topological_order_is_deterministic() {
        let dag = build_pipeline_dag();
        let order_a = topological_order(&dag).expect("topological order should succeed");
        let order_b = topological_order(&dag).expect("topological order should succeed");
        assert_eq!(order_a, order_b, "topological order should be stable");
        assert_eq!(
            order_a,
            vec![
                NODE_DISCOVER.to_string(),
                NODE_PARSE.to_string(),
                NODE_BUILD.to_string(),
                NODE_REPORT.to_string()
            ]
        );
    }

    #[test]
    fn topo_sort_modules_remaps_dependency_indices_after_reorder() {
        let ast_a = parser::parse("module a\nfn ok() -> Unit {}").expect("parse should succeed");
        let ast_b = parser::parse("module b\nfn ok() -> Unit {}").expect("parse should succeed");
        let ast_c = parser::parse("module c\nfn ok() -> Unit {}").expect("parse should succeed");

        let mut modules = vec![
            ResolvedModule {
                path: PathBuf::from("a.dag"),
                ast: ast_a,
                module_path: vec!["a".into()],
                dependencies: vec![2],
            },
            ResolvedModule {
                path: PathBuf::from("b.dag"),
                ast: ast_b,
                module_path: vec!["b".into()],
                dependencies: vec![],
            },
            ResolvedModule {
                path: PathBuf::from("c.dag"),
                ast: ast_c,
                module_path: vec!["c".into()],
                dependencies: vec![],
            },
        ];

        let cycle_modules = topo_sort_modules(&mut modules);
        assert!(cycle_modules.is_empty(), "expected acyclic ordering");
        assert_eq!(modules[0].module_path, vec!["c".to_string()]);
        assert_eq!(modules[1].module_path, vec!["b".to_string()]);
        assert_eq!(modules[2].module_path, vec!["a".to_string()]);
        assert_eq!(
            modules[2].dependencies,
            vec![0],
            "dependency index should be remapped to new module positions"
        );
    }

    #[test]
    fn build_module_graph_remaps_dependency_indices_after_topological_sort() {
        let parsed_modules = vec![
            ParsedModule {
                path: PathBuf::from("a.dag"),
                module_path: vec!["a".into()],
                imports: vec![vec!["c".into()]],
                ast: parser::parse("module a\nimport c\nfn ok() -> Unit {}")
                    .expect("parse should succeed"),
            },
            ParsedModule {
                path: PathBuf::from("b.dag"),
                module_path: vec!["b".into()],
                imports: vec![],
                ast: parser::parse("module b\nfn ok() -> Unit {}").expect("parse should succeed"),
            },
            ParsedModule {
                path: PathBuf::from("c.dag"),
                module_path: vec!["c".into()],
                imports: vec![],
                ast: parser::parse("module c\nfn ok() -> Unit {}").expect("parse should succeed"),
            },
        ];
        let mut diagnostics = Vec::new();
        let graph = build_module_graph(parsed_modules, &mut diagnostics);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
        assert_eq!(graph.modules.len(), 3);
        assert_eq!(graph.modules[0].module_path, vec!["c".to_string()]);
        assert_eq!(graph.modules[1].module_path, vec!["b".to_string()]);
        assert_eq!(graph.modules[2].module_path, vec!["a".to_string()]);
        assert_eq!(
            graph.modules[2].dependencies,
            vec![0],
            "graph dependency index should point at reordered dependency"
        );
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
            PipeValue::Diagnostics(vec![Diagnostic::new(
                DiagnosticKind::Pipeline,
                "not files",
            )]),
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
    fn build_op_rejects_wrong_pipe_value_variant() {
        let context = PipelineContext {
            roots: vec![],
            target_file: None,
        };
        let mut inputs = HashMap::new();
        inputs.insert(
            PORT_PARSED_MODULES.to_string(),
            PipeValue::Diagnostics(vec![Diagnostic::new(
                DiagnosticKind::Pipeline,
                "not parsed modules",
            )]),
        );
        inputs.insert(
            PORT_DIAGNOSTICS.to_string(),
            PipeValue::Diagnostics(Vec::new()),
        );

        let err = execute_op(
            &gunbc_ir::node::NodeBody::Opaque(CompilerOp::BuildModuleGraph),
            &context,
            inputs,
        )
        .expect_err("expected build op to reject wrong input variant");
        assert!(err.contains("expected parsed modules input"));
    }

    #[test]
    fn report_op_rejects_wrong_pipe_value_variant() {
        let context = PipelineContext {
            roots: vec![],
            target_file: None,
        };
        let mut inputs = HashMap::new();
        inputs.insert(
            PORT_MODULE_GRAPH.to_string(),
            PipeValue::Diagnostics(vec![Diagnostic::new(
                DiagnosticKind::Pipeline,
                "not module graph",
            )]),
        );
        inputs.insert(
            PORT_DIAGNOSTICS.to_string(),
            PipeValue::Diagnostics(Vec::new()),
        );

        let err = execute_op(
            &gunbc_ir::node::NodeBody::Opaque(CompilerOp::ReportModules),
            &context,
            inputs,
        )
        .expect_err("expected report op to reject wrong input variant");
        assert!(err.contains("expected module graph input"));
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
                .any(|diag| diag.render().contains("broken.dag:2:")),
            "diagnostics should contain file:line:col locations"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn parse_pipeline_parses_full_real_dsl_corpus_without_diagnostics() {
        let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
        let result = run_pipeline(
            &PipelineContext {
                roots: vec![dsl_root],
                target_file: None,
            },
            PipelineStop::Parse,
        )
        .expect("pipeline should execute");

        assert_eq!(result.parsed_count, 42);
        assert!(
            result.diagnostics.is_empty(),
            "real corpus parse stop should not emit parse diagnostics: {:?}",
            result.diagnostics
        );
        assert!(result.report.is_none());
        assert!(result.module_graph.is_none());
    }

    #[test]
    fn report_pipeline_emits_expected_real_corpus_resolve_diagnostics() {
        let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
        let result = run_pipeline(
            &PipelineContext {
                roots: vec![dsl_root],
                target_file: None,
            },
            PipelineStop::Report,
        )
        .expect("pipeline should execute");

        assert_eq!(
            result.diagnostics.len(),
            2,
            "real corpus report should emit expected resolve diagnostics"
        );
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diag| diag.kind == DiagnosticKind::Resolve),
            "real corpus report diagnostics should be resolve-kind only"
        );
        let rendered: Vec<String> = result.diagnostics.iter().map(|diag| diag.render()).collect();
        assert!(
            rendered.iter().any(|line| line.contains(
                "cyclic dependencies detected among modules: examples.deployment, shared.gist_modes, tools.dag_viz, tools.gist"
            )),
            "expected cycle diagnostic in report diagnostics: {rendered:?}"
        );
        assert!(
            rendered
                .iter()
                .any(|line| line.contains("unresolved import: examples.integration_tests -> infra.gcp")),
            "expected unresolved-import diagnostic in report diagnostics: {rendered:?}"
        );
        let report = result.report.as_ref().expect("report should be available");
        assert!(report.contains("Discovered modules:"));
        assert!(report.contains("tools.makegen"));
    }

    #[test]
    fn report_pipeline_real_corpus_retains_parsed_count() {
        let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
        let result = run_pipeline(
            &PipelineContext {
                roots: vec![dsl_root],
                target_file: None,
            },
            PipelineStop::Report,
        )
        .expect("pipeline should execute");
        assert_eq!(
            result.parsed_count, 42,
            "report stop should retain parse-stage file count for real corpus"
        );
    }

    #[test]
    fn report_pipeline_real_corpus_module_order_matches_expected_snapshot() {
        let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
        let result = run_pipeline(
            &PipelineContext {
                roots: vec![dsl_root],
                target_file: None,
            },
            PipelineStop::Report,
        )
        .expect("pipeline should execute");
        let report = result.report.as_ref().expect("report should be available");
        let actual = reported_modules_in_order(report);
        let expected: Vec<String> = expected_real_corpus_module_order()
            .into_iter()
            .map(String::from)
            .collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn build_pipeline_real_corpus_dependency_indices_match_declared_imports() {
        let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
        let context = PipelineContext {
            roots: vec![dsl_root],
            target_file: None,
        };
        let (graph, diagnostics) = run_parse_and_build_ops(&context);
        assert_eq!(
            diagnostics.len(),
            2,
            "real corpus build should emit known resolve diagnostics"
        );
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.kind == DiagnosticKind::Resolve),
            "build diagnostics should be resolve-kind only"
        );

        let module_index: HashMap<Vec<String>, usize> = graph
            .modules
            .iter()
            .enumerate()
            .map(|(idx, module)| (module.module_path.clone(), idx))
            .collect();

        for module in &graph.modules {
            let declared_imports: HashSet<Vec<String>> = module
                .ast
                .imports
                .iter()
                .map(|import| import.node.path.segments.clone())
                .collect();
            let resolved_dependencies: HashSet<Vec<String>> = module
                .dependencies
                .iter()
                .map(|dep_idx| graph.modules[*dep_idx].module_path.clone())
                .collect();

            for dep_path in &resolved_dependencies {
                assert!(
                    declared_imports.contains(dep_path),
                    "resolved dependency {} was not declared by module {}",
                    dep_path.join("."),
                    module.module_path.join(".")
                );
            }
            for import in &declared_imports {
                if module_index.contains_key(import) {
                    assert!(
                        resolved_dependencies.contains(import),
                        "declared import {} should resolve as dependency for module {}",
                        import.join("."),
                        module.module_path.join(".")
                    );
                }
            }
        }
    }

    #[test]
    fn build_pipeline_real_corpus_dependency_counts_match_resolve_discovery() {
        let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
        let context = PipelineContext {
            roots: vec![dsl_root.clone()],
            target_file: None,
        };
        let (pipeline_graph, _) = run_parse_and_build_ops(&context);
        let resolve_graph =
            daglang_resolve::ModuleGraph::discover(&[dsl_root]).expect("resolve discovery should succeed");

        let pipeline_counts: BTreeMap<String, usize> = pipeline_graph
            .modules
            .iter()
            .map(|module| (module.module_path.join("."), module.dependencies.len()))
            .collect();
        let resolve_counts: BTreeMap<String, usize> = resolve_graph
            .modules
            .iter()
            .map(|module| (module.module_path.join("."), module.dependencies.len()))
            .collect();

        assert_eq!(pipeline_counts, resolve_counts);
    }

    #[test]
    fn build_pipeline_real_corpus_module_graph_matches_resolve_discovery() {
        let dsl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../dsl");
        let context = PipelineContext {
            roots: vec![dsl_root.clone()],
            target_file: None,
        };
        let (pipeline_graph, _) = run_parse_and_build_ops(&context);
        let resolve_graph =
            daglang_resolve::ModuleGraph::discover(&[dsl_root]).expect("resolve discovery should succeed");

        let pipeline_order: Vec<String> = pipeline_graph
            .modules
            .iter()
            .map(|module| module.module_path.join("."))
            .collect();
        let resolve_order: Vec<String> = resolve_graph
            .modules
            .iter()
            .map(|module| module.module_path.join("."))
            .collect();
        assert_eq!(pipeline_order, resolve_order);

        let pipeline_deps: BTreeMap<String, Vec<String>> = pipeline_graph
            .modules
            .iter()
            .map(|module| {
                let mut deps: Vec<String> = module
                    .dependencies
                    .iter()
                    .map(|dep_idx| pipeline_graph.modules[*dep_idx].module_path.join("."))
                    .collect();
                deps.sort();
                (module.module_path.join("."), deps)
            })
            .collect();
        let resolve_deps: BTreeMap<String, Vec<String>> = resolve_graph
            .modules
            .iter()
            .map(|module| {
                let mut deps: Vec<String> = module
                    .dependencies
                    .iter()
                    .map(|dep_idx| resolve_graph.modules[*dep_idx].module_path.join("."))
                    .collect();
                deps.sort();
                (module.module_path.join("."), deps)
            })
            .collect();
        assert_eq!(pipeline_deps, resolve_deps);
    }

    #[test]
    fn parse_stop_does_not_emit_module_graph_or_report_values() {
        let root = unique_temp_dir("parse_stop_outputs");
        fs::create_dir_all(&root).expect("failed to create temp root");
        fs::write(root.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
            .expect("failed to write source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Parse).expect("pipeline should execute");
        assert_eq!(result.parsed_count, 1);
        assert!(result.module_graph.is_none());
        assert!(result.report.is_none());
        assert!(result.diagnostics.is_empty());

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn report_stop_emits_module_graph_and_report_values() {
        let root = unique_temp_dir("report_stop_outputs");
        fs::create_dir_all(&root).expect("failed to create temp root");
        fs::write(root.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
            .expect("failed to write source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Report).expect("pipeline should execute");
        assert_eq!(
            result.parsed_count, 1,
            "report stop should retain parsed count from parse stage"
        );
        assert!(
            result.module_graph.is_none(),
            "report stop currently consumes module graph into report stage"
        );
        let report = result.report.as_ref().expect("report stop should include report text");
        assert!(report.contains("Discovered modules:"));
        assert!(report.contains("sample.main"));
        assert!(result.diagnostics.is_empty());

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn parse_pipeline_aggregates_diagnostics_across_multiple_invalid_files() {
        let root = unique_temp_dir("multi_invalid");
        fs::create_dir_all(&root).expect("failed to create temp root");
        fs::write(root.join("z_broken.dag"), "module broken.z\nfn")
            .expect("failed to write invalid source A");
        fs::write(root.join("a_broken.dag"), "module broken.a\nimport")
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
                .any(|diag| diag.render().contains("z_broken.dag")),
            "expected diagnostics to include z_broken.dag"
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.render().contains("a_broken.dag")),
            "expected diagnostics to include a_broken.dag"
        );
        assert!(
            result.diagnostics[0].render().contains("a_broken.dag"),
            "diagnostics should be deterministically sorted by path"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn parse_pipeline_sorts_lex_before_parse_diagnostics() {
        let root = unique_temp_dir("parse_kind_order");
        fs::create_dir_all(&root).expect("failed to create temp root");
        fs::write(root.join("a_parse.dag"), "module broken.parse\nfn")
            .expect("failed to write parse-error source");
        fs::write(root.join("z_lex.dag"), "module broken.lex\n$\n")
            .expect("failed to write lex-error source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Parse).expect("pipeline should execute");

        assert!(
            result.diagnostics.len() >= 2,
            "expected lexical and parse diagnostics"
        );
        assert_eq!(
            result.diagnostics[0].kind,
            DiagnosticKind::Lex,
            "lex diagnostics should sort before parse diagnostics"
        );
        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.kind == DiagnosticKind::Parse),
            "expected at least one parse diagnostic"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn parse_pipeline_is_independent_of_root_argument_order() {
        let root = unique_temp_dir("parse_root_order");
        let a = root.join("a");
        let b = root.join("b");
        fs::create_dir_all(&a).expect("failed to create root a");
        fs::create_dir_all(&b).expect("failed to create root b");
        fs::write(a.join("a_parse.dag"), "module broken.a\nfn")
            .expect("failed to write parse-broken source");
        fs::write(b.join("b_lex.dag"), "module broken.b\n$\n")
            .expect("failed to write lex-broken source");

        let first = run_pipeline(
            &PipelineContext {
                roots: vec![a.clone(), b.clone()],
                target_file: None,
            },
            PipelineStop::Parse,
        )
        .expect("first run should complete");
        let second = run_pipeline(
            &PipelineContext {
                roots: vec![b, a],
                target_file: None,
            },
            PipelineStop::Parse,
        )
        .expect("second run should complete");

        assert_eq!(first.parsed_count, second.parsed_count);
        assert_eq!(first.diagnostics, second.diagnostics);
        assert!(
            first.diagnostics.len() >= 2,
            "expected diagnostics from both parse and lexical failures"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn parse_pipeline_does_not_emit_resolve_diagnostics_for_unresolved_imports() {
        let root = unique_temp_dir("parse_only_unresolved_import");
        fs::create_dir_all(&root).expect("failed to create temp root");
        fs::write(
            root.join("main.dag"),
            "module sample.main\nimport missing.dep\nfn ok() -> Unit {}",
        )
        .expect("failed to write source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Parse).expect("pipeline should execute");

        assert_eq!(result.parsed_count, 1, "expected parsed module count");
        assert!(
            result.diagnostics.is_empty(),
            "parse pipeline should not emit resolve diagnostics"
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

    #[test]
    fn target_file_mode_reports_missing_file_error() {
        let root = unique_temp_dir("missing_target");
        fs::create_dir_all(&root).expect("failed to create temp root");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: Some(root.join("missing.dag")),
        };
        let err = run_pipeline(&context, PipelineStop::Parse)
            .expect_err("missing target file should fail pipeline execution");
        assert!(err.contains("failed to read"));
        assert!(err.contains("missing.dag"));

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn target_file_mode_accepts_symlinked_file() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("target_file_symlink");
        fs::create_dir_all(&root).expect("failed to create temp root");
        let real = root.join("real.dag");
        let link = root.join("link.dag");
        fs::write(&real, "module sample.real\nfn ok() -> Unit {}")
            .expect("failed to write real source");
        symlink(&real, &link).expect("failed to create file symlink");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: Some(link),
        };
        let result = run_pipeline(&context, PipelineStop::Parse).expect("pipeline should execute");
        assert_eq!(result.parsed_count, 1);
        assert!(result.diagnostics.is_empty());

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn target_file_mode_reports_dangling_symlink_read_error() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("target_file_dangling_symlink");
        fs::create_dir_all(&root).expect("failed to create temp root");
        let dangling_target = root.join("missing.dag");
        let dangling_link = root.join("broken.dag");
        symlink(&dangling_target, &dangling_link).expect("failed to create dangling file symlink");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: Some(dangling_link),
        };
        let err = run_pipeline(&context, PipelineStop::Parse)
            .expect_err("dangling symlink target file should fail");
        assert!(err.contains("failed to read"));
        assert!(err.contains("broken.dag"));

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn report_pipeline_target_file_symlink_derives_module_path_against_real_root() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("target_file_symlink_real_root_fallback");
        let real_root = root.join("real");
        let link_root = root.join("link");
        let nested = real_root.join("nested");
        fs::create_dir_all(&nested).expect("failed to create nested real root");
        fs::write(nested.join("no_module.dag"), "fn ok() -> Unit {}")
            .expect("failed to write source");
        symlink(&real_root, &link_root).expect("failed to create root symlink");

        let context = PipelineContext {
            roots: vec![real_root],
            target_file: Some(link_root.join("nested/no_module.dag")),
        };
        let result =
            run_pipeline(&context, PipelineStop::Report).expect("pipeline should execute");
        let report = result.report.expect("report output should be present");
        assert!(
            report.contains("nested.no_module"),
            "target-file symlink fallback should derive nested module path via real root: {report}"
        );
        assert!(result.diagnostics.is_empty());

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn report_pipeline_target_file_module_fallback_is_independent_of_symlink_alias_root_order() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("target_file_symlink_alias_root_order");
        let real_root = root.join("real");
        let link_root = root.join("link");
        let nested = real_root.join("nested");
        fs::create_dir_all(&nested).expect("failed to create nested real root");
        fs::write(nested.join("no_module.dag"), "fn ok() -> Unit {}")
            .expect("failed to write source");
        symlink(&real_root, &link_root).expect("failed to create root symlink");
        let target = link_root.join("nested/no_module.dag");

        let first = run_pipeline(
            &PipelineContext {
                roots: vec![real_root.clone(), link_root.clone()],
                target_file: Some(target.clone()),
            },
            PipelineStop::Report,
        )
        .expect("first pipeline run should execute");
        let second = run_pipeline(
            &PipelineContext {
                roots: vec![link_root, real_root],
                target_file: Some(target),
            },
            PipelineStop::Report,
        )
        .expect("second pipeline run should execute");

        assert_eq!(first.diagnostics, second.diagnostics);
        assert_eq!(first.report, second.report);

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn report_pipeline_fallback_module_path_is_independent_of_overlapping_root_order() {
        let root = unique_temp_dir("report_overlapping_root_order_no_module");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).expect("failed to create nested root");
        fs::write(nested.join("no_module.dag"), "fn ok() -> Unit {}")
            .expect("failed to write source");

        let first = run_pipeline(
            &PipelineContext {
                roots: vec![root.clone(), nested.clone()],
                target_file: None,
            },
            PipelineStop::Report,
        )
        .expect("first pipeline run should execute");
        let second = run_pipeline(
            &PipelineContext {
                roots: vec![nested, root.clone()],
                target_file: None,
            },
            PipelineStop::Report,
        )
        .expect("second pipeline run should execute");

        assert_eq!(first.diagnostics, second.diagnostics);
        assert_eq!(first.report, second.report);
        let report = first.report.expect("report should be present");
        assert!(
            report.contains("no_module"),
            "expected fallback module-path output: {report}"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn target_file_mode_ignores_directory_roots() {
        let valid_root = unique_temp_dir("target_file_ignores_roots");
        fs::create_dir_all(&valid_root).expect("failed to create valid temp root");
        let target = valid_root.join("good.dag");
        fs::write(&target, "module sample.good\nfn ok() -> Unit {}")
            .expect("failed to write valid source");

        let missing_root = unique_temp_dir("target_file_missing_root");
        let context = PipelineContext {
            roots: vec![missing_root],
            target_file: Some(target),
        };
        let result = run_pipeline(&context, PipelineStop::Parse)
            .expect("target-file mode should not require directory roots");
        assert_eq!(result.parsed_count, 1);
        assert!(result.diagnostics.is_empty());

        fs::remove_dir_all(valid_root).expect("failed to cleanup temp root");
    }

    #[test]
    fn parse_pipeline_with_empty_roots_succeeds_with_zero_files() {
        let context = PipelineContext {
            roots: vec![],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Parse)
            .expect("empty roots should be valid for parse pipeline");
        assert_eq!(result.parsed_count, 0);
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn parse_pipeline_deduplicates_files_from_overlapping_roots() {
        let root = unique_temp_dir("overlapping_roots");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).expect("failed to create nested root");
        fs::write(
            nested.join("main.dag"),
            "module sample.main\nfn ok() -> Unit {}",
        )
        .expect("failed to write source");

        let context = PipelineContext {
            roots: vec![root.clone(), nested],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Parse).expect("pipeline should execute");
        assert_eq!(
            result.parsed_count, 1,
            "overlapping roots should not duplicate parsed files"
        );
        assert!(result.diagnostics.is_empty());

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn parse_pipeline_deduplicates_files_from_duplicate_roots() {
        let root = unique_temp_dir("duplicate_roots");
        fs::create_dir_all(&root).expect("failed to create temp root");
        fs::write(root.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
            .expect("failed to write source");

        let context = PipelineContext {
            roots: vec![root.clone(), root.clone()],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Parse).expect("pipeline should execute");
        assert_eq!(
            result.parsed_count, 1,
            "duplicate roots should not duplicate parsed files"
        );
        assert!(result.diagnostics.is_empty());

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn parse_pipeline_deduplicates_diagnostics_from_duplicate_roots() {
        let root = unique_temp_dir("duplicate_roots_diags");
        fs::create_dir_all(&root).expect("failed to create temp root");
        fs::write(root.join("broken.dag"), "module sample.broken\nfn")
            .expect("failed to write invalid source");

        let context = PipelineContext {
            roots: vec![root.clone(), root.clone()],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Parse).expect("pipeline should execute");
        let broken_hits = result
            .diagnostics
            .iter()
            .filter(|diag| diag.render().contains("broken.dag"))
            .count();
        assert_eq!(
            broken_hits, 1,
            "duplicate roots should not duplicate parse diagnostics for the same file"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn parse_pipeline_deduplicates_files_from_symlink_and_real_roots() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("symlink_and_real_roots");
        let real = root.join("real");
        let link = root.join("link");
        fs::create_dir_all(&real).expect("failed to create real root");
        fs::write(real.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
            .expect("failed to write source");
        symlink(&real, &link).expect("failed to create root symlink");

        let context = PipelineContext {
            roots: vec![real.clone(), link],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Parse).expect("pipeline should execute");
        assert_eq!(
            result.parsed_count, 1,
            "real+symlink roots should not duplicate parsed files"
        );
        assert!(result.diagnostics.is_empty());

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn parse_pipeline_handles_directory_symlink_cycle_without_recursing_forever() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("dir_symlink_cycle");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).expect("failed to create nested root");
        fs::write(nested.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
            .expect("failed to write source");
        symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Parse)
            .expect("pipeline should handle directory cycle symlink");
        assert_eq!(result.parsed_count, 1);
        assert!(result.diagnostics.is_empty());

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn parse_pipeline_is_independent_of_symlink_alias_root_order() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("symlink_alias_root_order");
        let real = root.join("real");
        let link = root.join("link");
        fs::create_dir_all(&real).expect("failed to create real root");
        fs::write(real.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
            .expect("failed to write source");
        symlink(&real, &link).expect("failed to create root symlink");

        let first = run_pipeline(
            &PipelineContext {
                roots: vec![real.clone(), link.clone()],
                target_file: None,
            },
            PipelineStop::Parse,
        )
        .expect("first run should complete");
        let second = run_pipeline(
            &PipelineContext {
                roots: vec![link, real],
                target_file: None,
            },
            PipelineStop::Parse,
        )
        .expect("second run should complete");

        assert_eq!(first.parsed_count, second.parsed_count);
        assert_eq!(first.diagnostics, second.diagnostics);

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn parse_pipeline_error_output_is_independent_of_symlink_alias_root_order() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("symlink_alias_root_order_parse_errors");
        let real = root.join("real");
        let link = root.join("link");
        fs::create_dir_all(&real).expect("failed to create real root");
        fs::write(real.join("broken.dag"), "module sample.broken\nfn")
            .expect("failed to write invalid source");
        symlink(&real, &link).expect("failed to create root symlink");

        let first = run_pipeline(
            &PipelineContext {
                roots: vec![real.clone(), link.clone()],
                target_file: None,
            },
            PipelineStop::Parse,
        )
        .expect("first run should complete");
        let second = run_pipeline(
            &PipelineContext {
                roots: vec![link, real],
                target_file: None,
            },
            PipelineStop::Parse,
        )
        .expect("second run should complete");

        assert_eq!(first.parsed_count, second.parsed_count);
        assert_eq!(first.diagnostics, second.diagnostics);
        assert!(
            !first.diagnostics.is_empty(),
            "expected parse diagnostics for malformed source"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn parse_pipeline_deduplicates_diagnostics_in_directory_symlink_cycle() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("dir_symlink_cycle_diags");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).expect("failed to create nested root");
        fs::write(nested.join("broken.dag"), "module sample.broken\nfn")
            .expect("failed to write invalid source");
        symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Parse)
            .expect("pipeline should handle directory cycle symlink");
        let broken_hits = result
            .diagnostics
            .iter()
            .filter(|diag| diag.render().contains("broken.dag"))
            .count();
        assert_eq!(
            broken_hits, 1,
            "directory symlink cycle should not duplicate diagnostics for same file"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn parse_pipeline_output_is_deterministic_in_directory_symlink_cycle_with_errors() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("dir_symlink_cycle_deterministic_errors");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).expect("failed to create nested root");
        fs::write(nested.join("broken.dag"), "module sample.broken\nfn")
            .expect("failed to write invalid source");
        symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

        let first = run_pipeline(
            &PipelineContext {
                roots: vec![root.clone()],
                target_file: None,
            },
            PipelineStop::Parse,
        )
        .expect("first pipeline run should complete");
        let second = run_pipeline(
            &PipelineContext {
                roots: vec![root.clone()],
                target_file: None,
            },
            PipelineStop::Parse,
        )
        .expect("second pipeline run should complete");

        assert_eq!(first.parsed_count, second.parsed_count);
        assert_eq!(first.diagnostics, second.diagnostics);

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn parse_pipeline_deduplicates_diagnostics_from_symlink_and_real_roots() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("symlink_and_real_roots_diags");
        let real = root.join("real");
        let link = root.join("link");
        fs::create_dir_all(&real).expect("failed to create real root");
        fs::write(real.join("broken.dag"), "module sample.broken\nfn")
            .expect("failed to write invalid source");
        symlink(&real, &link).expect("failed to create root symlink");

        let context = PipelineContext {
            roots: vec![real.clone(), link],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Parse).expect("pipeline should execute");
        let broken_hits = result
            .diagnostics
            .iter()
            .filter(|diag| diag.render().contains("broken.dag"))
            .count();
        assert_eq!(
            broken_hits, 1,
            "real+symlink roots should not duplicate parse diagnostics for the same file"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn parse_pipeline_reports_canonicalize_error_for_dangling_dag_symlink() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("dangling_dag_symlink");
        fs::create_dir_all(&root).expect("failed to create temp root");
        let dangling_target = root.join("missing.dag");
        let dangling_link = root.join("broken.dag");
        symlink(&dangling_target, &dangling_link).expect("failed to create dangling symlink");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let err = run_pipeline(&context, PipelineStop::Parse)
            .expect_err("dangling dag symlink should fail canonicalization");
        assert!(err.contains("failed to canonicalize"));
        assert!(err.contains("broken.dag"));

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn report_pipeline_derives_module_path_for_symlink_root_without_module_decl() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("symlink_root_module_path_fallback");
        let real = root.join("real");
        let link = root.join("link");
        let nested = real.join("nested");
        fs::create_dir_all(&nested).expect("failed to create nested root");
        fs::write(nested.join("no_module.dag"), "fn ok() -> Unit {}")
            .expect("failed to write source");
        symlink(&real, &link).expect("failed to create root symlink");

        let context = PipelineContext {
            roots: vec![link],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Report).expect("pipeline should execute");

        assert!(result.diagnostics.is_empty());
        let report = result.report.expect("report output should be present");
        assert!(
            report.contains("nested.no_module"),
            "module-path fallback should be derived relative to symlink root: {report}"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn report_pipeline_deduplicates_parse_diagnostics_in_directory_symlink_cycle() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("report_symlink_cycle_parse_dedup");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).expect("failed to create nested root");
        fs::write(nested.join("broken.dag"), "module sample.broken\nfn")
            .expect("failed to write invalid source");
        symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

        let result = run_pipeline(
            &PipelineContext {
                roots: vec![root.clone()],
                target_file: None,
            },
            PipelineStop::Report,
        )
        .expect("report pipeline should execute");
        let broken_hits = result
            .diagnostics
            .iter()
            .filter(|diag| diag.render().contains("broken.dag"))
            .count();
        assert_eq!(
            broken_hits, 1,
            "report pipeline should not duplicate parse diagnostics in symlink cycle"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn report_pipeline_output_is_deterministic_in_directory_symlink_cycle_with_errors() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("report_symlink_cycle_deterministic_errors");
        let nested = root.join("nested");
        fs::create_dir_all(&nested).expect("failed to create nested root");
        fs::write(nested.join("broken.dag"), "module sample.broken\nfn")
            .expect("failed to write invalid source");
        symlink(&root, nested.join("loop")).expect("failed to create directory cycle symlink");

        let first = run_pipeline(
            &PipelineContext {
                roots: vec![root.clone()],
                target_file: None,
            },
            PipelineStop::Report,
        )
        .expect("first report pipeline run should execute");
        let second = run_pipeline(
            &PipelineContext {
                roots: vec![root.clone()],
                target_file: None,
            },
            PipelineStop::Report,
        )
        .expect("second report pipeline run should execute");

        assert_eq!(first.diagnostics, second.diagnostics);
        assert_eq!(first.report, second.report);

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn report_pipeline_with_empty_roots_emits_empty_graph_report() {
        let context = PipelineContext {
            roots: vec![],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Report)
            .expect("empty roots should be valid for report pipeline");
        let report = result.report.expect("report should be generated");
        assert!(report.contains("Discovered modules:"));
        assert!(!report.contains("Diagnostics:"));
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn directory_mode_reports_missing_root_error() {
        let root = unique_temp_dir("missing_root");
        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let err = run_pipeline(&context, PipelineStop::Parse)
            .expect_err("missing directory root should fail pipeline execution");
        assert!(err.contains("input root does not exist"));
    }

    #[test]
    fn directory_mode_fails_when_any_root_is_missing() {
        let valid_root = unique_temp_dir("mixed_valid_missing_root_valid");
        fs::create_dir_all(&valid_root).expect("failed to create valid root");
        fs::write(
            valid_root.join("main.dag"),
            "module sample.main\nfn ok() -> Unit {}",
        )
        .expect("failed to write source");
        let missing_root = unique_temp_dir("mixed_valid_missing_root_missing");

        let context = PipelineContext {
            roots: vec![valid_root.clone(), missing_root.clone()],
            target_file: None,
        };
        let err = run_pipeline(&context, PipelineStop::Parse)
            .expect_err("pipeline should fail when any root is missing");
        assert!(err.contains("input root does not exist"));
        assert!(err.contains(&missing_root.display().to_string()));

        fs::remove_dir_all(valid_root).expect("failed to cleanup valid root");
    }

    #[test]
    fn directory_mode_rejects_non_directory_root() {
        let root_file = unique_temp_dir("non_directory_root");
        fs::write(&root_file, "not a directory").expect("failed to create root file");

        let context = PipelineContext {
            roots: vec![root_file.clone()],
            target_file: None,
        };
        let err = run_pipeline(&context, PipelineStop::Parse)
            .expect_err("non-directory root should fail pipeline execution");
        assert!(err.contains("input root is not a directory"));

        fs::remove_file(root_file).expect("failed to cleanup root file");
    }

    #[test]
    fn directory_mode_fails_when_any_root_is_non_directory() {
        let valid_root = unique_temp_dir("mixed_valid_nondir_root_valid");
        fs::create_dir_all(&valid_root).expect("failed to create valid root");
        fs::write(
            valid_root.join("main.dag"),
            "module sample.main\nfn ok() -> Unit {}",
        )
        .expect("failed to write source");

        let non_directory_root = unique_temp_dir("mixed_valid_nondir_root_file");
        fs::write(&non_directory_root, "not a directory")
            .expect("failed to create non-directory root file");

        let context = PipelineContext {
            roots: vec![valid_root.clone(), non_directory_root.clone()],
            target_file: None,
        };
        let err = run_pipeline(&context, PipelineStop::Parse)
            .expect_err("pipeline should fail when any root is non-directory");
        assert!(err.contains("input root is not a directory"));
        assert!(err.contains(&non_directory_root.display().to_string()));

        fs::remove_dir_all(valid_root).expect("failed to cleanup valid root");
        fs::remove_file(non_directory_root).expect("failed to cleanup non-directory root file");
    }

    #[test]
    fn report_pipeline_emits_unresolved_import_diagnostic() {
        let root = unique_temp_dir("unresolved_import");
        fs::create_dir_all(&root).expect("failed to create temp root");
        fs::write(
            root.join("main.dag"),
            "module sample.main\nimport missing.dep\nfn ok() -> Unit {}",
        )
        .expect("failed to write source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Report).expect("pipeline should execute");

        let unresolved = result
            .diagnostics
            .iter()
            .find(|diag| diag.render().contains("unresolved import"))
            .expect("expected unresolved import diagnostic in report pipeline");
        assert_eq!(unresolved.kind, DiagnosticKind::Resolve);

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn report_pipeline_emits_duplicate_module_diagnostic() {
        let root = unique_temp_dir("duplicate_module_diag");
        fs::create_dir_all(root.join("a")).expect("failed to create temp root A");
        fs::create_dir_all(root.join("b")).expect("failed to create temp root B");
        fs::write(
            root.join("a/one.dag"),
            "module dup.mod\nfn one() -> Unit {}",
        )
        .expect("failed to write source one");
        fs::write(
            root.join("b/two.dag"),
            "module dup.mod\nfn two() -> Unit {}",
        )
        .expect("failed to write source two");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Report).expect("pipeline should execute");

        let duplicate = result
            .diagnostics
            .iter()
            .find(|diag| diag.render().contains("duplicate module path"))
            .expect("expected duplicate module diagnostic in report pipeline");
        assert_eq!(duplicate.kind, DiagnosticKind::Resolve);

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn report_pipeline_emits_cycle_diagnostic() {
        let root = unique_temp_dir("cycle_diag");
        fs::create_dir_all(&root).expect("failed to create temp root");
        fs::write(
            root.join("a.dag"),
            "module cycle.a\nimport cycle.b\nfn a() -> Unit {}",
        )
        .expect("failed to write source a");
        fs::write(
            root.join("b.dag"),
            "module cycle.b\nimport cycle.a\nfn b() -> Unit {}",
        )
        .expect("failed to write source b");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Report).expect("pipeline should execute");

        let cycle = result
            .diagnostics
            .iter()
            .find(|diag| diag.render().contains("cyclic dependencies detected"))
            .expect("expected cycle diagnostic in report pipeline");
        assert_eq!(cycle.kind, DiagnosticKind::Resolve);

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn report_pipeline_emits_unresolved_import_diagnostic_rendered() {
        let root = unique_temp_dir("unresolved_import_render");
        fs::create_dir_all(&root).expect("failed to create temp root");
        fs::write(
            root.join("main.dag"),
            "module sample.main\nimport missing.dep\nfn ok() -> Unit {}",
        )
        .expect("failed to write source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Report).expect("pipeline should execute");

        assert!(
            result
                .diagnostics
                .iter()
                .any(|diag| diag.render().contains("unresolved import")),
            "expected unresolved import diagnostic in report pipeline"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn report_pipeline_deduplicates_duplicate_unresolved_import_diagnostics() {
        let root = unique_temp_dir("dedupe_unresolved");
        fs::create_dir_all(&root).expect("failed to create temp root");
        fs::write(
            root.join("main.dag"),
            "module sample.main\nimport missing.dep\nimport missing.dep\nfn ok() -> Unit {}",
        )
        .expect("failed to write source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Report).expect("pipeline should execute");

        let unresolved_count = result
            .diagnostics
            .iter()
            .filter(|diag| diag.render().contains("unresolved import"))
            .count();
        assert_eq!(
            unresolved_count, 1,
            "duplicate unresolved imports should collapse into one diagnostic"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn report_pipeline_sorts_lex_before_resolve_diagnostics() {
        let root = unique_temp_dir("diag_sorting");
        fs::create_dir_all(&root).expect("failed to create temp root");
        fs::write(
            root.join("z_good.dag"),
            "module sample.good\nimport missing.dep\nfn ok() -> Unit {}",
        )
        .expect("failed to write good source");
        fs::write(root.join("a_bad.dag"), "module sample.bad\n$\n")
            .expect("failed to write bad source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let result = run_pipeline(&context, PipelineStop::Report).expect("pipeline should execute");

        assert!(
            result.diagnostics.len() >= 2,
            "expected at least two diagnostics from lex + resolve phases"
        );
        assert_eq!(
            result.diagnostics[0].kind,
            DiagnosticKind::Lex,
            "lex diagnostics should be sorted before resolve diagnostics"
        );
        assert!(
            result.diagnostics
                .iter()
                .any(|diag| diag.kind == DiagnosticKind::Resolve),
            "expected at least one resolve diagnostic"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn report_pipeline_output_is_deterministic_for_same_input_with_diagnostics() {
        let root = unique_temp_dir("report_deterministic_with_diags");
        fs::create_dir_all(&root).expect("failed to create temp root");
        fs::write(
            root.join("z_resolve.dag"),
            "module sample.resolve\nimport missing.dep\nfn ok() -> Unit {}",
        )
        .expect("failed to write resolve source");
        fs::write(root.join("a_lex.dag"), "module sample.lex\n$\n")
            .expect("failed to write lex source");

        let context = PipelineContext {
            roots: vec![root.clone()],
            target_file: None,
        };
        let first = run_pipeline(&context, PipelineStop::Report).expect("first run should succeed");
        let second =
            run_pipeline(&context, PipelineStop::Report).expect("second run should succeed");

        assert_eq!(first.diagnostics, second.diagnostics);
        assert_eq!(first.report, second.report);

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn report_pipeline_is_independent_of_root_argument_order() {
        let root = unique_temp_dir("report_root_order");
        let a = root.join("a");
        let b = root.join("b");
        fs::create_dir_all(&a).expect("failed to create root a");
        fs::create_dir_all(&b).expect("failed to create root b");
        fs::write(a.join("a.dag"), "module sample.a\nfn ok() -> Unit {}")
            .expect("failed to write source a");
        fs::write(b.join("b.dag"), "module sample.b\nfn ok() -> Unit {}")
            .expect("failed to write source b");

        let first = run_pipeline(
            &PipelineContext {
                roots: vec![a.clone(), b.clone()],
                target_file: None,
            },
            PipelineStop::Report,
        )
        .expect("first run should succeed");
        let second = run_pipeline(
            &PipelineContext {
                roots: vec![b, a],
                target_file: None,
            },
            PipelineStop::Report,
        )
        .expect("second run should succeed");

        assert_eq!(first.diagnostics, second.diagnostics);
        assert_eq!(first.report, second.report);

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn report_pipeline_is_independent_of_symlink_alias_root_order() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("report_symlink_alias_root_order");
        let real = root.join("real");
        let link = root.join("link");
        fs::create_dir_all(&real).expect("failed to create real root");
        fs::write(real.join("main.dag"), "module sample.main\nfn ok() -> Unit {}")
            .expect("failed to write source");
        symlink(&real, &link).expect("failed to create root symlink");

        let first = run_pipeline(
            &PipelineContext {
                roots: vec![real.clone(), link.clone()],
                target_file: None,
            },
            PipelineStop::Report,
        )
        .expect("first run should succeed");
        let second = run_pipeline(
            &PipelineContext {
                roots: vec![link, real],
                target_file: None,
            },
            PipelineStop::Report,
        )
        .expect("second run should succeed");

        assert_eq!(first.diagnostics, second.diagnostics);
        assert_eq!(first.report, second.report);

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[cfg(unix)]
    #[test]
    fn report_pipeline_error_output_is_independent_of_symlink_alias_root_order() {
        use std::os::unix::fs::symlink;

        let root = unique_temp_dir("report_symlink_alias_root_order_errors");
        let real = root.join("real");
        let link = root.join("link");
        fs::create_dir_all(&real).expect("failed to create real root");
        fs::write(real.join("broken.dag"), "module sample.broken\nfn")
            .expect("failed to write invalid source");
        symlink(&real, &link).expect("failed to create root symlink");

        let first = run_pipeline(
            &PipelineContext {
                roots: vec![real.clone(), link.clone()],
                target_file: None,
            },
            PipelineStop::Report,
        )
        .expect("first run should complete");
        let second = run_pipeline(
            &PipelineContext {
                roots: vec![link, real],
                target_file: None,
            },
            PipelineStop::Report,
        )
        .expect("second run should complete");

        assert_eq!(first.diagnostics, second.diagnostics);
        assert_eq!(first.report, second.report);
        assert!(
            !first.diagnostics.is_empty(),
            "expected report diagnostics for malformed source"
        );

        fs::remove_dir_all(root).expect("failed to cleanup temp root");
    }

    #[test]
    fn pipeline_rejects_implicit_fanin_on_single_input_port() {
        let mut dag = build_pipeline_dag();
        dag.add_edge(Edge::new(
            NODE_DISCOVER,
            PORT_FILES,
            NODE_PARSE,
            PORT_FILES,
        ));

        let err = validate_pipeline_semantics(&dag)
            .expect_err("duplicate edge to same input port should fail");
        assert!(err.contains("implicit fan-in is not allowed"));
    }

    #[test]
    fn pipeline_rejects_unconnected_inputs_on_non_entrypoints() {
        let mut dag = build_pipeline_dag();
        let parse_node = dag
            .nodes
            .iter_mut()
            .find(|node| node.id.0 == NODE_PARSE)
            .expect("parse node should exist");
        parse_node
            .inputs
            .push(Port::with_cardinality("extra", "Debug", Cardinality::ONE));

        let err = validate_pipeline_semantics(&dag)
            .expect_err("non-entrypoint with unconnected input should fail");
        assert!(err.contains("unconnected input port"));
    }

    #[test]
    fn pipeline_rejects_edge_with_unknown_source_port() {
        let mut dag = build_pipeline_dag();
        dag.add_edge(Edge::new(
            NODE_DISCOVER,
            "missing_output",
            NODE_PARSE,
            PORT_FILES,
        ));

        let err = validate_pipeline_semantics(&dag)
            .expect_err("edge from unknown source port should fail");
        assert!(err.contains("edge source port does not exist"));
    }

    #[test]
    fn pipeline_rejects_edge_with_unknown_target_port() {
        let mut dag = build_pipeline_dag();
        dag.add_edge(Edge::new(
            NODE_DISCOVER,
            PORT_FILES,
            NODE_PARSE,
            "missing_input",
        ));

        let err = validate_pipeline_semantics(&dag)
            .expect_err("edge to unknown target port should fail");
        assert!(err.contains("edge target port does not exist"));
    }

    #[test]
    fn pipeline_rejects_edge_with_unknown_source_node() {
        let mut dag = build_pipeline_dag();
        dag.add_edge(Edge::new(
            "missing-node",
            PORT_FILES,
            NODE_PARSE,
            PORT_FILES,
        ));

        let err = validate_pipeline_semantics(&dag)
            .expect_err("edge from unknown source node should fail");
        assert!(err.contains("edge source node does not exist"));
    }

    #[test]
    fn pipeline_rejects_edge_with_unknown_target_node() {
        let mut dag = build_pipeline_dag();
        dag.add_edge(Edge::new(
            NODE_DISCOVER,
            PORT_FILES,
            "missing-node",
            PORT_FILES,
        ));

        let err = validate_pipeline_semantics(&dag)
            .expect_err("edge to unknown target node should fail");
        assert!(err.contains("edge target node does not exist"));
    }

    #[test]
    fn pipeline_allows_unconnected_entrypoint_inputs() {
        let dag = build_pipeline_dag();
        validate_pipeline_semantics(&dag)
            .expect("discover node should allow unconnected entrypoint input ports");
    }

    #[test]
    fn pipeline_rejects_disconnected_non_entrypoint_with_inputs() {
        let mut dag = build_pipeline_dag();
        dag.edges.retain(|edge| edge.to_node.0 != NODE_PARSE);

        let err = validate_pipeline_semantics(&dag)
            .expect_err("non-entrypoint without inbound edges should fail");
        assert!(err.contains("non-entrypoint node"));
    }

    #[test]
    fn pipeline_rejects_duplicate_node_ids() {
        let mut dag = build_pipeline_dag();
        dag.add_node(Node::opaque(
            NODE_DISCOVER,
            vec![],
            vec![],
            CompilerOp::DiscoverFiles,
        ));

        let err = validate_pipeline_semantics(&dag)
            .expect_err("duplicate node ids should fail validation");
        assert!(err.contains("duplicate node id"));
    }

    #[test]
    fn pipeline_rejects_duplicate_input_port_names() {
        let mut dag = build_pipeline_dag();
        let parse_node = dag
            .nodes
            .iter_mut()
            .find(|node| node.id.0 == NODE_PARSE)
            .expect("parse node should exist");
        parse_node
            .inputs
            .push(Port::with_cardinality(PORT_FILES, "Vec<FileSource>", Cardinality::ONE));

        let err = validate_pipeline_semantics(&dag)
            .expect_err("duplicate input ports should fail validation");
        assert!(err.contains("duplicate input port"));
    }

    #[test]
    fn pipeline_rejects_duplicate_output_port_names() {
        let mut dag = build_pipeline_dag();
        let parse_node = dag
            .nodes
            .iter_mut()
            .find(|node| node.id.0 == NODE_PARSE)
            .expect("parse node should exist");
        parse_node.outputs.push(Port::with_cardinality(
            PORT_PARSED_MODULES,
            "Vec<ParsedModule>",
            Cardinality::ONE,
        ));

        let err = validate_pipeline_semantics(&dag)
            .expect_err("duplicate output ports should fail validation");
        assert!(err.contains("duplicate output port"));
    }

    #[test]
    fn pipeline_topological_order_rejects_cycles() {
        let mut dag = build_pipeline_dag();
        dag.add_edge(Edge::new(
            NODE_REPORT,
            PORT_DIAGNOSTICS,
            NODE_DISCOVER,
            PORT_CONTEXT,
        ));

        let err = topological_order(&dag).expect_err("cyclic pipeline DAG should fail ordering");
        assert!(err.contains("contains a cycle"));
    }

    #[test]
    fn pipeline_rejects_unreachable_node() {
        let mut dag = build_pipeline_dag();
        dag.add_node(Node::opaque(
            "orphan_node",
            vec![],
            vec![Port::with_cardinality("out", "Unit", Cardinality::ONE)],
            CompilerOp::ReportModules,
        ));

        let err = validate_pipeline_semantics(&dag)
            .expect_err("unreachable node should fail pipeline semantics");
        assert!(err.contains("is unreachable from entrypoint"));
    }

    #[test]
    fn pipeline_rejects_inbound_edges_to_entrypoint_node() {
        let mut dag = build_pipeline_dag();
        dag.add_edge(Edge::new(
            NODE_REPORT,
            PORT_DIAGNOSTICS,
            NODE_DISCOVER,
            PORT_CONTEXT,
        ));

        let err = validate_pipeline_semantics(&dag)
            .expect_err("entrypoint node should not accept inbound edges");
        assert!(err.contains("must not have inbound edges"));
    }

    #[test]
    fn pipeline_requires_discover_entrypoint_node() {
        let mut dag = build_pipeline_dag();
        dag.nodes.retain(|node| node.id.0 != NODE_DISCOVER);

        let err = validate_pipeline_semantics(&dag)
            .expect_err("missing discover entrypoint should fail validation");
        assert!(err.contains("missing required entrypoint node"));
    }
}
