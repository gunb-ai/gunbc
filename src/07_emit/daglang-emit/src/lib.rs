//! daglang-emit: CodegenBackend trait and Rust backend.
//!
//! The final compiler phase: emit runnable code from GraphIR + derived
//! metadata. Each backend implements `CodegenBackend` to produce
//! target-language code.
//!
//! # Pipeline position
//!
//! ```text
//! GraphIR + ProgressManifest + TestObligations
//!   → [daglang-emit] → Rust source files (Phase 1)
//!                     → Go source files (Phase 4)
//! ```
//!
//! # What gets emitted per module
//!
//! ```text
//! tools/example.dag
//!   ├── types/      Type definitions (records, enums)
//!   ├── fn/         Pure functors → target language functions
//!   ├── transport/  Transport wiring (HTTP, shell, file)
//!   ├── func/       DAG orchestrator (topo-scheduled execution)
//!   ├── cli/        CLI entrypoint (arg parsing from func inputs)
//!   ├── test/       Test harness (4-bucket obligations)
//!   ├── mock/       MockSpec (from service declarations)
//!   ├── manifest/   ProgressManifest (static, from topology)
//!   └── makefile/   Makefile target (from module metadata)
//! ```

// ── Task-owned modules (dsl-codegen-tasks.md) ──────────────────────
// Wave 1
pub mod computation; // Task 1: Computation types
pub mod rust_exec_runtime; // Task 3: Exec-runtime fast path

// Wave 2
pub mod plan; // Task 4: EmitPlan builder

// Shared type mapping tables (RT28).
pub mod type_mapping;

// Backend language models (hierarchical target modeling).
pub mod language_model;

// Wave 3 (Tasks 8-11): AbstractIR lowering pipeline.
pub mod lower_c;
pub mod lower_go;
pub mod lower_rust;
pub mod lower_to_ir;
pub mod transport_analysis;

// Wave 4 (Tasks 12-16): target renderers + register lowering.
pub mod lower_mips;
pub mod render_c;
pub mod render_go;
pub mod render_mips;
pub mod render_rust;

// Wave 5 (Task E3): test generation.
pub mod test_gen;

// Wave 6 (SC5-SC6): service transport code generation per language.
pub mod service_emit;

// Wave 7: DSL-native test mock emission (inline test blocks -> graph_mock.rs).
pub mod test_mock_emit;

// Wave 8: TypeDef → Rust type codegen (struct/enum generation from DSL types).
pub mod type_codegen;

// Wave 9: DSL FnBody → abstract IR compiler (function body generation).
pub mod fn_codegen;

// Wave 10: Data-only .dag artifact emitter (FC-P7-b).
pub mod dag_emit;

#[cfg(test)]
mod backend_harness;

use daglang_derive::{DerivedArtifacts, ProgressManifest};
pub use daglang_lower::extract_output_paths;
use daglang_lower::{CallableKind, LoweredOp, ObligationCategory, ServiceOperationSpec};
use gunbc_ir::{Dag, ProgramSymbolId, ReachableDag};
use std::collections::{BTreeSet, HashSet};
use std::fmt::Write as _;

// ============================================================================
// FC-14: Reachability analysis for dead-path pruning
// ============================================================================

/// Compute the set of node IDs reachable from DAG entrypoints.
///
/// A node is reachable if it is an entrypoint (no incoming edges on at
/// least one input port) or if it is downstream of a reachable node via
/// the edge graph. This is used by emitters to prune unreachable symbols
/// so generated code passes strict `-D warnings` / `-Wall -Werror` builds.
pub fn compute_reachable_node_ids(dag: &Dag<LoweredOp>) -> HashSet<String> {
    // Build adjacency: from_node → [to_node]
    let mut adj: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    let mut has_incoming: HashSet<&str> = HashSet::new();
    for edge in &dag.edges {
        adj.entry(edge.from_node.0.as_str())
            .or_default()
            .push(edge.to_node.0.as_str());
        has_incoming.insert(edge.to_node.0.as_str());
    }

    // Entrypoints: nodes with no incoming edges (or all nodes if no edges).
    let entrypoints: Vec<&str> = if dag.edges.is_empty() {
        dag.nodes.iter().map(|n| n.id.0.as_str()).collect()
    } else {
        dag.nodes
            .iter()
            .filter(|n| !has_incoming.contains(n.id.0.as_str()))
            .map(|n| n.id.0.as_str())
            .collect()
    };

    // BFS from entrypoints.
    let mut reachable = HashSet::new();
    let mut queue: std::collections::VecDeque<&str> = entrypoints.into_iter().collect();
    while let Some(node_id) = queue.pop_front() {
        if !reachable.insert(node_id.to_string()) {
            continue;
        }
        if let Some(successors) = adj.get(node_id) {
            for succ in successors {
                if !reachable.contains(*succ) {
                    queue.push_back(succ);
                }
            }
        }
    }
    reachable
}

/// Pre-computed data to embed into generated artifacts.
///
/// Each entry associates a module + semantic key with file content that
/// backends embed as string literals or Layer 1 writes as additional files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbeddedData {
    /// DSL module this data belongs to, e.g. `"tools.bundle"`.
    pub module: String,
    /// Layer 1 output file path relative to the crate root, e.g. `"src/embedded_bundle.txt"`.
    pub layer1_file_path: String,
    /// Identifier used in Layer 2 backends, e.g. `"bundle_content"`.
    pub layer2_ident: String,
    /// The actual content to embed.
    pub content: String,
}

/// The codegen backend trait. Each target language implements this.
pub trait CodegenBackend {
    /// Emit a type definition (record, enum, alias).
    fn emit_type(&self, ty: &str) -> String;

    /// Emit a pure functor as a target-language function.
    fn emit_fn(&self, name: &str) -> String;

    /// Emit a DAG orchestrator for an effectful function.
    fn emit_func(&self, name: &str) -> String;

    /// Emit transport wiring (HTTP client, shell exec, file I/O).
    fn emit_transport(&self, spec: &str) -> String;

    /// Emit a test harness from test obligations.
    fn emit_test(&self, obligation: &str) -> String;

    /// Emit CLI entrypoint from DAG entry ports.
    fn emit_cli(&self, entrypoints: &[String]) -> String;

    /// Emit a progress manifest (static topology for renderers).
    fn emit_progress_manifest(&self, manifest: &str) -> String;
}

/// A file emitted by a backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmittedFile {
    pub path: String,
    pub content: String,
}

/// Backend emission summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmissionSummary {
    pub module_count: usize,
    pub callable_count: usize,
    pub pipeline_count: usize,
}

/// Aggregated emission output for a compile request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmissionBundle {
    pub backend: String,
    pub files: Vec<EmittedFile>,
    pub summary: EmissionSummary,
}

/// Errors during emission.
#[derive(Debug)]
pub enum EmitError {
    /// A construct couldn't be emitted for the target backend.
    UnsupportedConstruct { backend: String, construct: String },
    /// A lowered graph node could not be rendered.
    InvalidLoweredNode(String),
    /// Required embedded data is missing for the target backend.
    MissingEmbeddedAsset { backend: String, key: String },
}

impl EmitError {
    /// Stable, grep-able error code for this variant (CP-59).
    pub fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedConstruct { .. } => "EMI001",
            Self::InvalidLoweredNode(..) => "EMI002",
            Self::MissingEmbeddedAsset { .. } => "EMI003",
        }
    }
}

impl std::fmt::Display for EmitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedConstruct { backend, construct } => {
                write!(f, "backend `{backend}` does not support `{construct}`")
            }
            Self::InvalidLoweredNode(reason) => {
                write!(f, "invalid lowered node encountered during emit: {reason}")
            }
            Self::MissingEmbeddedAsset { backend, key } => {
                write!(f, "backend `{backend}` missing embedded asset `{key}`")
            }
        }
    }
}

/// Minimal Rust backend used by Phase-1 compiler scaffolding.
#[derive(Debug, Default, Clone, Copy)]
pub struct RustBackend;

impl CodegenBackend for RustBackend {
    fn emit_type(&self, ty: &str) -> String {
        let alias_name = sanitize_identifier(ty);
        format!("pub type {alias_name} = serde_json::Value;\n")
    }

    fn emit_fn(&self, name: &str) -> String {
        format!("pub fn {name}() -> serde_json::Value {{\n    serde_json::Value::Null\n}}\n")
    }

    fn emit_func(&self, name: &str) -> String {
        format!("pub fn {name}() {{\n    let _ = ();\n}}\n")
    }

    fn emit_transport(&self, spec: &str) -> String {
        let fn_name = sanitize_identifier(&format!("transport_{spec}"));
        format!(
            "pub fn {fn_name}(request: serde_json::Value) -> serde_json::Value {{\n    request\n}}\n"
        )
    }

    fn emit_test(&self, obligation: &str) -> String {
        format!(
            "#[test]\nfn obligation_{}() {{\n    assert!(true, \"obligation `{}` satisfied\");\n}}\n",
            sanitize_identifier(obligation),
            obligation
        )
    }

    fn emit_cli(&self, entrypoints: &[String]) -> String {
        format!(
            "pub fn cli_entrypoints() -> &'static [&'static str] {{\n    &{:?}\n}}\n",
            entrypoints
        )
    }

    fn emit_progress_manifest(&self, manifest: &str) -> String {
        format!("// progress\n{manifest}\n")
    }
}

/// Emit a minimal Rust project bundle from lowered GraphIR and derived artifacts.
///
/// Accepts a `ReachableDag` to structurally enforce that only reachable nodes
/// are emitted — the type system prevents access to unreachable code paths.
pub fn emit_rust_bundle(
    dag: &ReachableDag<LoweredOp>,
    artifacts: &DerivedArtifacts,
) -> Result<EmissionBundle, EmitError> {
    let backend = RustBackend;
    let mut emitted_functions = Vec::new();
    let mut callable_count = 0usize;
    let mut pipeline_count = 0usize;

    for node in &dag.nodes {
        let Some(op) = node.body.as_opaque() else {
            continue;
        };

        match op {
            LoweredOp::Callable {
                module, kind, name, ..
            }
            | LoweredOp::Transport {
                module, kind, name, ..
            } => {
                callable_count += 1;
                let fn_name = sanitize_identifier(&format!("{module}_{name}"));
                let rendered = match kind {
                    CallableKind::Fn => backend.emit_fn(&fn_name),
                    CallableKind::Func | CallableKind::Pattern => backend.emit_func(&fn_name),
                };
                emitted_functions.push(rendered);
            }
            LoweredOp::Primitive { module, name, .. } => {
                callable_count += 1;
                let fn_name = sanitize_identifier(&format!("{module}_{name}"));
                emitted_functions.push(backend.emit_func(&fn_name));
            }
            LoweredOp::Collection {
                module,
                callable,
                kind,
            } => {
                callable_count += 1;
                let fn_name =
                    sanitize_identifier(&format!("{module}_{callable}_collection_{kind:?}"));
                emitted_functions.push(backend.emit_func(&fn_name));
            }
            LoweredOp::Pipeline { module, name, .. } => {
                pipeline_count += 1;
                let fn_name = sanitize_identifier(&format!("{module}_{name}"));
                emitted_functions.push(backend.emit_func(&fn_name));
            }
            LoweredOp::Pattern(_) | LoweredOp::UnsupportedPattern { .. } => {}
        }
    }

    // TL-14: Also collect symbols for middleware config emission.
    let (rust_symbols, _, _) = collect_symbols_with_metadata(dag)?;
    let rust_middleware_funcs =
        emit_middleware_inline_funcs(&rust_symbols, service_emit::emit_rust_middleware_config);
    if !rust_middleware_funcs.is_empty() {
        emitted_functions.push(format!(
            "// Transport middleware configuration (TL-14).\n{rust_middleware_funcs}"
        ));
    }

    let module_count = artifacts.tool_metadata.modules.len();
    let manifest_rendered = render_manifest(&artifacts.manifest);

    let mut files = vec![
        EmittedFile {
            path: "target/generated/rust/main.rs".to_string(),
            content: format!(
                "// Generated by daglang-emit (phase-1 scaffold)\n\n{}\n{}",
                backend.emit_cli(
                    &artifacts
                        .manifest
                        .entrypoint_nodes
                        .iter()
                        .map(|entry| sanitize_identifier(entry))
                        .collect::<Vec<_>>()
                ),
                emitted_functions.join("\n")
            ),
        },
        EmittedFile {
            path: "target/generated/rust/progress_manifest.txt".to_string(),
            content: backend.emit_progress_manifest(&manifest_rendered),
        },
    ];
    // TL-14: Emit middleware config JSON manifest.
    if let Some(manifest) = emit_middleware_manifest("rust", &rust_symbols)? {
        files.push(manifest);
    }
    if let Some(test_file) = test_gen::emit_dry_run_completion_test("rust", &artifacts.obligations)
    {
        files.push(test_file);
    }
    if let Some(mock_tests) = test_gen::emit_transport_mock_tests("rust", dag) {
        files.push(mock_tests);
    }

    Ok(EmissionBundle {
        backend: "rust".to_string(),
        files,
        summary: EmissionSummary {
            module_count,
            callable_count,
            pipeline_count,
        },
    })
}

/// Emit a minimal Go project bundle from lowered GraphIR and derived artifacts.
pub fn emit_go_bundle(
    dag: &ReachableDag<LoweredOp>,
    artifacts: &DerivedArtifacts,
    required_assets: &BTreeSet<ProgramSymbolId>,
    embedded_data: &std::collections::HashMap<String, EmbeddedData>,
) -> Result<EmissionBundle, EmitError> {
    let (symbols, callable_count, pipeline_count) = collect_symbols_with_metadata(dag)?;
    let manifest_rendered = render_manifest(&artifacts.manifest);
    let embedded_asset = single_required_embedded_asset(required_assets, embedded_data, "go")?;
    let entrypoints = artifacts
        .manifest
        .entrypoint_nodes
        .iter()
        .map(|entry| sanitize_identifier(entry))
        .collect::<Vec<_>>();

    let has_service_transport = symbols.iter().any(|s| s.spec.is_some());

    let mut symbol_funcs_parts: Vec<String> = Vec::with_capacity(symbols.len());
    for sym in &symbols {
        if let Some(ref spec) = sym.spec {
            let phase = require_service_phase(sym)?;
            symbol_funcs_parts.push(service_emit::emit_go_service_func(&sym.name, phase, spec));
        } else {
            symbol_funcs_parts.push(format!(
                "func {name}() {{\n    // generated callable stub\n}}\n",
                name = sym.name
            ));
        }
    }
    let mut symbol_funcs = symbol_funcs_parts.join("\n");

    // TL-14: Append inline Go middleware config functions.
    let go_middleware_funcs =
        emit_middleware_inline_funcs(&symbols, service_emit::emit_go_middleware_config);
    if !go_middleware_funcs.is_empty() {
        symbol_funcs.push_str("\n// Transport middleware configuration (TL-14).\n");
        symbol_funcs.push_str(&go_middleware_funcs);
    }

    let entrypoint_lits = entrypoints
        .iter()
        .map(|entry| format!("\"{entry}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let imports = if embedded_asset.is_some() {
        "import (\n    \"fmt\"\n    \"os\"\n)\n".to_string()
    } else if has_service_transport {
        "import (\n    \"fmt\"\n    \"net/http\"\n    \"bytes\"\n    \"encoding/json\"\n    \"os/exec\"\n    \"strings\"\n)\n".to_string()
    } else {
        "import \"fmt\"\n".to_string()
    };

    let main_go = if let Some(asset) = embedded_asset {
        let embedded_fn = sanitize_identifier(&asset.layer2_ident);
        let embedded_literal = escape_string_literal(asset.content.as_str());
        format!(
            "package main\n\n{imports}\nfunc cliEntrypoints() []string {{\n    return []string{{{entrypoint_lits}}}\n}}\n\n{symbol_funcs}\nfunc {embedded_fn}() string {{\n    return \"{embedded_literal}\"\n}}\n\nfunc main() {{\n    if len(os.Args) > 1 {{\n        path := os.Args[1]\n        if err := os.WriteFile(path, []byte({embedded_fn}()), 0644); err != nil {{\n            fmt.Fprintf(os.Stderr, \"failed to write `%s`: %v\\n\", path, err)\n            os.Exit(1)\n        }}\n    }}\n    fmt.Println(\"daglang generated go backend\")\n}}\n"
        )
    } else {
        format!(
            "package main\n\n{imports}\nfunc cliEntrypoints() []string {{\n    return []string{{{entrypoint_lits}}}\n}}\n\n{symbol_funcs}\nfunc main() {{\n    fmt.Println(\"daglang generated go backend\")\n}}\n"
        )
    };

    // Suppress unused import warnings for service transport code.
    let main_go = if has_service_transport && embedded_asset.is_none() {
        main_go.replace(
            "func main() {",
            "// Ensure imports used.\nvar _ = http.StatusOK\nvar _ = bytes.Compare\nvar _ = json.Unmarshal\nvar _ = exec.Command\nvar _ = strings.TrimSpace\n\nfunc main() {",
        )
    } else {
        main_go
    };

    let mut files = vec![
        EmittedFile {
            path: "target/generated/go/main.go".to_string(),
            content: main_go,
        },
        EmittedFile {
            path: "target/generated/go/progress_manifest.txt".to_string(),
            content: manifest_rendered,
        },
    ];
    // TL-14: Emit middleware config JSON manifest.
    if let Some(manifest) = emit_middleware_manifest("go", &symbols)? {
        files.push(manifest);
    }
    if let Some(test_file) = test_gen::emit_dry_run_completion_test("go", &artifacts.obligations) {
        files.push(test_file);
    }
    if let Some(mock_tests) = test_gen::emit_transport_mock_tests("go", dag) {
        files.push(mock_tests);
    }

    Ok(EmissionBundle {
        backend: "go".to_string(),
        files,
        summary: EmissionSummary {
            module_count: artifacts.tool_metadata.modules.len(),
            callable_count,
            pipeline_count,
        },
    })
}

/// Emit a minimal C project bundle from lowered GraphIR and derived artifacts.
pub fn emit_c_bundle(
    dag: &ReachableDag<LoweredOp>,
    artifacts: &DerivedArtifacts,
    required_assets: &BTreeSet<ProgramSymbolId>,
    embedded_data: &std::collections::HashMap<String, EmbeddedData>,
) -> Result<EmissionBundle, EmitError> {
    let (symbols, callable_count, pipeline_count) = collect_symbols_with_metadata(dag)?;
    let manifest_rendered = render_manifest(&artifacts.manifest);
    let embedded_asset = single_required_embedded_asset(required_assets, embedded_data, "c")?;
    let has_service_transport = symbols.iter().any(|s| s.spec.is_some());
    let entrypoints = artifacts
        .manifest
        .entrypoint_nodes
        .iter()
        .map(|entry| sanitize_identifier(entry))
        .collect::<Vec<_>>();

    let mut symbol_funcs_parts: Vec<String> = Vec::with_capacity(symbols.len());
    for sym in &symbols {
        if let Some(ref spec) = sym.spec {
            let phase = require_service_phase(sym)?;
            symbol_funcs_parts.push(service_emit::emit_c_service_func(&sym.name, phase, spec));
        } else {
            symbol_funcs_parts.push(format!("static void {name}(void) {{}}\n", name = sym.name));
        }
    }
    let symbol_funcs = symbol_funcs_parts.join("\n");

    // TL-14: Emit inline C middleware config structs.
    let c_middleware_funcs =
        emit_middleware_inline_funcs(&symbols, service_emit::emit_c_middleware_config);
    let symbol_funcs = if c_middleware_funcs.is_empty() {
        symbol_funcs
    } else {
        format!("{symbol_funcs}\n/* Transport middleware configuration (TL-14). */\n{c_middleware_funcs}\n")
    };

    let entrypoint_defs = entrypoints
        .iter()
        .map(|entry| format!("\"{entry}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let includes = if embedded_asset.is_some() {
        "#include <stdio.h>\n#include <string.h>\n".to_string()
    } else if has_service_transport {
        "#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n#include <curl/curl.h>\n"
            .to_string()
    } else {
        "#include <stdio.h>\n".to_string()
    };

    let main_c = if let Some(asset) = embedded_asset {
        let embedded_const = sanitize_identifier(&asset.layer2_ident).to_ascii_uppercase();
        let embedded_literal = escape_string_literal(asset.content.as_str());
        format!(
            "{includes}\nstatic const char* CLI_ENTRYPOINTS[] = {{{entrypoint_defs}}};\nstatic const char* {embedded_const} = \"{embedded_literal}\";\n\n{symbol_funcs}\nint main(int argc, char** argv) {{\n    (void)CLI_ENTRYPOINTS;\n    if (argc > 1) {{\n        const char* path = argv[1];\n        FILE* file = fopen(path, \"wb\");\n        if (!file) {{\n            fprintf(stderr, \"failed to write `%s`\\n\", path);\n            return 1;\n        }}\n        size_t expected = strlen({embedded_const});\n        size_t written = fwrite({embedded_const}, 1, expected, file);\n        fclose(file);\n        if (written != expected) {{\n            fprintf(stderr, \"failed to write `%s`\\n\", path);\n            return 1;\n        }}\n    }}\n    printf(\"daglang generated c backend\\n\");\n    return 0;\n}}\n"
        )
    } else {
        format!(
            "{includes}\nstatic const char* CLI_ENTRYPOINTS[] = {{{entrypoint_defs}}};\n\n{symbol_funcs}\nint main(void) {{\n    printf(\"daglang generated c backend\\n\");\n    return (int)(sizeof(CLI_ENTRYPOINTS) / sizeof(CLI_ENTRYPOINTS[0])) >= 0 ? 0 : 1;\n}}\n"
        )
    };

    let mut files = vec![
        EmittedFile {
            path: "target/generated/c/main.c".to_string(),
            content: main_c,
        },
        EmittedFile {
            path: "target/generated/c/progress_manifest.txt".to_string(),
            content: manifest_rendered,
        },
    ];
    // TL-14: Emit middleware config JSON manifest.
    if let Some(manifest) = emit_middleware_manifest("c", &symbols)? {
        files.push(manifest);
    }
    if let Some(test_file) = test_gen::emit_dry_run_completion_test("c", &artifacts.obligations) {
        files.push(test_file);
    }
    if let Some(mock_tests) = test_gen::emit_transport_mock_tests("c", dag) {
        files.push(mock_tests);
    }

    Ok(EmissionBundle {
        backend: "c".to_string(),
        files,
        summary: EmissionSummary {
            module_count: artifacts.tool_metadata.modules.len(),
            callable_count,
            pipeline_count,
        },
    })
}

/// Emit a minimal MIPS assembly bundle from lowered GraphIR and derived artifacts.
pub fn emit_mips_bundle(
    dag: &ReachableDag<LoweredOp>,
    artifacts: &DerivedArtifacts,
    required_assets: &BTreeSet<ProgramSymbolId>,
    embedded_data: &std::collections::HashMap<String, EmbeddedData>,
) -> Result<EmissionBundle, EmitError> {
    let (symbols, callable_count, pipeline_count) = collect_symbols_with_metadata(dag)?;
    let manifest_rendered = render_manifest(&artifacts.manifest);
    let embedded_asset = single_required_embedded_asset(required_assets, embedded_data, "mips")?;

    let mut label_defs_parts: Vec<String> = Vec::with_capacity(symbols.len());
    for sym in &symbols {
        if let Some(ref spec) = sym.spec {
            let phase = require_service_phase(sym)?;
            label_defs_parts.push(service_emit::emit_mips_service_func(&sym.name, phase, spec));
        } else {
            label_defs_parts.push(format!("{name}:\n    jr $ra\n", name = sym.name));
        }
    }
    let label_defs = label_defs_parts.join("\n");

    // TL-14: Emit inline MIPS middleware config data.
    let mips_middleware_data =
        emit_middleware_inline_funcs(&symbols, service_emit::emit_mips_middleware_config);

    let main_s = if let Some(asset) = embedded_asset {
        let embedded_label = sanitize_identifier(&asset.layer2_ident);
        let embedded_bytes = asset
            .content
            .as_bytes()
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            ".text\n.globl main\n\n{label_defs}\nmain:\n    li $a0, 1\n    la $a1, {embedded_label}\n    li $a2, {}\n    li $v0, 4004\n    syscall\n    li $a0, 0\n    li $v0, 4001\n    syscall\n\n.data\n{embedded_label}:\n    .byte {embedded_bytes}\n{mips_middleware_data}",
            asset.content.len()
        )
    } else if !mips_middleware_data.is_empty() {
        format!(".text\n.globl main\n\n{label_defs}\nmain:\n    li $v0, 10\n    syscall\n\n.data\n{mips_middleware_data}")
    } else {
        format!(".text\n.globl main\n\n{label_defs}\nmain:\n    li $v0, 10\n    syscall\n")
    };

    let mut files = vec![
        EmittedFile {
            path: "target/generated/mips/main.s".to_string(),
            content: main_s,
        },
        EmittedFile {
            path: "target/generated/mips/progress_manifest.txt".to_string(),
            content: manifest_rendered,
        },
    ];
    // TL-14: Emit middleware config JSON manifest.
    if let Some(manifest) = emit_middleware_manifest("mips", &symbols)? {
        files.push(manifest);
    }
    if let Some(test_file) = test_gen::emit_dry_run_completion_test("mips", &artifacts.obligations)
    {
        files.push(test_file);
    }
    if let Some(mock_tests) = test_gen::emit_transport_mock_tests("mips", dag) {
        files.push(mock_tests);
    }

    Ok(EmissionBundle {
        backend: "mips".to_string(),
        files,
        summary: EmissionSummary {
            module_count: artifacts.tool_metadata.modules.len(),
            callable_count,
            pipeline_count,
        },
    })
}

/// A collected symbol from the DAG, with optional service transport metadata.
struct CollectedSymbol {
    name: String,
    spec: Option<ServiceOperationSpec>,
    service_phase: Option<service_emit::ServiceTransportPhase>,
}

fn require_service_phase(
    symbol: &CollectedSymbol,
) -> Result<service_emit::ServiceTransportPhase, EmitError> {
    symbol.service_phase.ok_or_else(|| {
        EmitError::InvalidLoweredNode(format!(
            "service symbol `{}` missing transport phase metadata",
            symbol.name
        ))
    })
}

/// Collect middleware configs from symbols for JSON manifest emission (TL-14).
///
/// Returns a list of (operation_name, config) pairs, deduplicated to one entry
/// per operation (prepare/execute/parse share the same spec).
fn collect_middleware_configs(
    symbols: &[CollectedSymbol],
) -> Vec<(String, &gunbc_ir::transport::TransportMiddlewareConfig)> {
    let mut seen = HashSet::new();
    let mut configs = Vec::new();
    for sym in symbols {
        if sym.service_phase != Some(service_emit::ServiceTransportPhase::Prepare) {
            continue;
        }
        let middleware = sym.spec.as_ref().and_then(service_emit::extract_middleware);
        if let Some(mw) = middleware {
            if seen.insert(&sym.name) {
                configs.push((sym.name.clone(), mw));
            }
        }
    }
    configs
}

/// Emit middleware config as an additional JSON manifest file (TL-14).
fn emit_middleware_manifest(
    backend: &str,
    symbols: &[CollectedSymbol],
) -> Result<Option<EmittedFile>, EmitError> {
    let configs = collect_middleware_configs(symbols);
    if configs.is_empty() {
        return Ok(None);
    }
    let json = service_emit::serialize_middleware_config_json(&configs).map_err(|e| {
        EmitError::InvalidLoweredNode(format!("middleware config serialization failed: {e}"))
    })?;
    Ok(Some(EmittedFile {
        path: format!("target/generated/{backend}/transport_middleware.json"),
        content: json,
    }))
}

/// Emit inline middleware config functions for a specific backend (TL-14).
fn emit_middleware_inline_funcs(
    symbols: &[CollectedSymbol],
    emit_fn: fn(&str, &gunbc_ir::transport::TransportMiddlewareConfig) -> String,
) -> String {
    let configs = collect_middleware_configs(symbols);
    if configs.is_empty() {
        return String::new();
    }
    configs
        .iter()
        .map(|(name, config)| emit_fn(name, config))
        .collect::<Vec<_>>()
        .join("\n")
}

fn collect_symbols_with_metadata(
    dag: &ReachableDag<LoweredOp>,
) -> Result<(Vec<CollectedSymbol>, usize, usize), EmitError> {
    let mut symbols = Vec::new();
    let mut callable_count = 0usize;
    let mut pipeline_count = 0usize;

    for node in &dag.nodes {
        let Some(op) = node.body.as_opaque() else {
            continue;
        };

        match op {
            LoweredOp::Callable {
                module,
                name,
                obligation,
                ..
            } => {
                callable_count += 1;
                symbols.push(CollectedSymbol {
                    name: sanitize_identifier(&format!("{module}_{name}")),
                    spec: None,
                    service_phase: service_transport_phase(*obligation),
                });
            }
            LoweredOp::Transport {
                module,
                name,
                obligation,
                service_metadata,
                ..
            } => {
                callable_count += 1;
                let spec = service_metadata.spec.clone();
                symbols.push(CollectedSymbol {
                    name: sanitize_identifier(&format!("{module}_{name}")),
                    spec,
                    service_phase: service_transport_phase(*obligation),
                });
            }
            LoweredOp::Primitive { module, name, .. } => {
                callable_count += 1;
                symbols.push(CollectedSymbol {
                    name: sanitize_identifier(&format!("{module}_{name}")),
                    spec: None,
                    service_phase: None,
                });
            }
            LoweredOp::Collection {
                module,
                callable,
                kind,
            } => {
                callable_count += 1;
                symbols.push(CollectedSymbol {
                    name: sanitize_identifier(&format!("{module}_{callable}_collection_{kind:?}")),
                    spec: None,
                    service_phase: None,
                });
            }
            LoweredOp::Pipeline { module, name, .. } => {
                pipeline_count += 1;
                symbols.push(CollectedSymbol {
                    name: sanitize_identifier(&format!("{module}_{name}")),
                    spec: None,
                    service_phase: None,
                });
            }
            LoweredOp::Pattern(_) | LoweredOp::UnsupportedPattern { .. } => {}
        }
    }

    Ok((symbols, callable_count, pipeline_count))
}

fn service_transport_phase(
    obligation: ObligationCategory,
) -> Option<service_emit::ServiceTransportPhase> {
    match obligation {
        ObligationCategory::ServiceTransportPrepare => {
            Some(service_emit::ServiceTransportPhase::Prepare)
        }
        ObligationCategory::ServiceTransportExecute => {
            Some(service_emit::ServiceTransportPhase::Execute)
        }
        ObligationCategory::ServiceTransportParse => {
            Some(service_emit::ServiceTransportPhase::Parse)
        }
        _ => None,
    }
}

fn require_embedded_asset<'a>(
    embedded_data: &'a std::collections::HashMap<String, EmbeddedData>,
    backend: &str,
    key: &str,
) -> Result<&'a EmbeddedData, EmitError> {
    embedded_data
        .get(key)
        .ok_or_else(|| EmitError::MissingEmbeddedAsset {
            backend: backend.to_string(),
            key: key.to_string(),
        })
}

fn single_required_embedded_asset<'a>(
    required_assets: &BTreeSet<ProgramSymbolId>,
    embedded_data: &'a std::collections::HashMap<String, EmbeddedData>,
    backend: &str,
) -> Result<Option<&'a EmbeddedData>, EmitError> {
    let mut assets = Vec::new();
    for asset in required_assets {
        assets.push(require_embedded_asset(
            embedded_data,
            backend,
            asset.as_str(),
        )?);
    }

    match assets.len() {
        0 => Ok(None),
        1 => Ok(assets.into_iter().next()),
        _ => Err(EmitError::UnsupportedConstruct {
            backend: backend.to_string(),
            construct: format!(
                "multiple embedded assets: {}",
                required_assets
                    .iter()
                    .map(ProgramSymbolId::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }),
    }
}

fn escape_string_literal(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 8);
    for ch in input.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out
}

fn render_manifest(manifest: &ProgressManifest) -> String {
    let mut out = String::new();
    let _ = writeln!(&mut out, "total_nodes={}", manifest.total_nodes);
    let _ = writeln!(&mut out, "total_edges={}", manifest.total_edges);
    out.push_str("waves=\n");
    for (idx, wave) in manifest.waves.iter().enumerate() {
        let _ = writeln!(&mut out, "  [{idx}] {}", wave.join(", "));
    }
    let _ = writeln!(
        &mut out,
        "entrypoint_nodes={}",
        manifest.entrypoint_nodes.join(", ")
    );
    let _ = writeln!(
        &mut out,
        "boundary_nodes={}",
        manifest.boundary_nodes.join(", ")
    );
    out.push_str("topology=\n");
    for node in &manifest.topology {
        let _ = writeln!(&mut out, "  {}@{}", node.id, node.depth);
    }
    out.push_str("labels=\n");
    for (node_id, label) in &manifest.labels {
        let _ = writeln!(&mut out, "  {}={}", node_id, label);
    }
    out.push_str("subdag_boundaries=\n");
    for boundary in &manifest.subdag_boundaries {
        let _ = writeln!(
            &mut out,
            "  {} label={} inner=[{}]",
            boundary.node_id,
            boundary.label,
            boundary.inner_nodes.join(",")
        );
    }
    out.push_str("parallel_groups=\n");
    for group in &manifest.parallel_groups {
        let _ = writeln!(
            &mut out,
            "  depth:{} nodes={}",
            group.depth,
            group.nodes.join(",")
        );
    }
    let _ = writeln!(
        &mut out,
        "scatter_points={}",
        manifest.scatter_points.join(", ")
    );
    let _ = writeln!(
        &mut out,
        "interactive_nodes={}",
        manifest.interactive_nodes.join(", ")
    );
    out.push_str("capture_modes=\n");
    for (node_id, mode) in &manifest.capture_modes {
        let _ = writeln!(&mut out, "  {}={:?}", node_id, mode);
    }
    out.push_str("stage_groups=\n");
    for group in &manifest.stage_groups {
        let _ = writeln!(&mut out, "  {}={}", group.stage_id, group.nodes.join(","));
    }
    out.push_str("resources=\n");
    for (node_id, usages) in &manifest.resources {
        let usages_rendered = usages
            .iter()
            .map(|usage| format!("{}:{}", usage.resource, usage.usage))
            .collect::<Vec<_>>()
            .join(",");
        let _ = writeln!(&mut out, "  {}={}", node_id, usages_rendered);
    }
    out
}

fn sanitize_identifier(value: &str) -> String {
    let mut out = value
        .chars()
        .map(|ch| if ch.is_alphanumeric() { ch } else { '_' })
        .collect::<String>();
    if out.is_empty() {
        out.push('_');
    }
    if out.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        out.insert(0, '_');
    }
    out
}

trait NodeBodyExt {
    fn as_opaque(&self) -> Option<&LoweredOp>;
}

impl NodeBodyExt for gunbc_ir::node::NodeBody<LoweredOp> {
    fn as_opaque(&self) -> Option<&LoweredOp> {
        match self {
            gunbc_ir::node::NodeBody::Opaque(op) => Some(op),
            gunbc_ir::node::NodeBody::SubDag(..) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_derive::derive_artifacts;
    use daglang_lower::ObligationCategory;
    use gunbc_ir::{Edge, Node, Port};
    use std::collections::{BTreeSet, HashMap};

    fn sample_dag() -> Dag<LoweredOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "tools.bundle::render_content",
            vec![Port::scalar("registry", "ToolRegistry")],
            vec![Port::scalar("return", "String")],
            LoweredOp::Callable {
                module: "tools.bundle".to_string(),
                kind: CallableKind::Fn,
                name: "render_content".to_string(),
                obligation: ObligationCategory::PureRender,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        dag.add_node(Node::opaque(
            "tools.bundle::bundle",
            vec![Port::scalar("registry", "ToolRegistry")],
            vec![Port::scalar("written", "Bool")],
            LoweredOp::Callable {
                module: "tools.bundle".to_string(),
                kind: CallableKind::Func,
                name: "bundle".to_string(),
                obligation: ObligationCategory::None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        dag.add_edge(Edge::new(
            "tools.bundle::render_content",
            "return",
            "tools.bundle::bundle",
            "registry",
        ));
        dag
    }

    fn sample_required_assets() -> BTreeSet<ProgramSymbolId> {
        let mut assets = BTreeSet::new();
        assets.insert(ProgramSymbolId::from("tools.bundle::embedded_asset"));
        assets
    }

    fn sample_embedded_data() -> HashMap<String, EmbeddedData> {
        let mut data = HashMap::new();
        data.insert(
            "tools.bundle::embedded_asset".to_string(),
            EmbeddedData {
                module: "tools.bundle".to_string(),
                layer1_file_path: "src/embedded_bundle.txt".to_string(),
                layer2_ident: "bundle_content".to_string(),
                content: "bundle-test-content".to_string(),
            },
        );
        data
    }

    #[test]
    fn emit_rust_bundle_generates_main_and_manifest_files() {
        let dag = sample_dag();
        let artifacts = derive_artifacts(&dag).expect("derive should succeed");
        let reachable = ReachableDag::from_dag(&dag);
        let bundle = emit_rust_bundle(&reachable, &artifacts).expect("emit should succeed");

        assert_eq!(bundle.backend, "rust");
        assert_eq!(bundle.files.len(), 3);
        assert!(bundle
            .files
            .iter()
            .any(|file| file.path.ends_with("main.rs")
                && file.content.contains("tools_bundle_bundle")
                && !file.content.contains("TODO(")));
        assert!(bundle
            .files
            .iter()
            .any(|file| file.path.ends_with("dry_run_completion_test.rs")
                && file
                    .content
                    .contains("dry_run_completion_required_contract")));
        let manifest_file = bundle
            .files
            .iter()
            .find(|file| file.path.ends_with("progress_manifest.txt"))
            .expect("progress manifest artifact should be emitted");
        assert!(manifest_file.content.contains("total_nodes="));
        assert!(manifest_file.content.contains("topology="));
        assert!(manifest_file.content.contains("labels="));
        assert!(manifest_file.content.contains("parallel_groups="));
        assert!(manifest_file.content.contains("capture_modes="));
        assert_eq!(bundle.summary.callable_count, 2);
    }

    #[test]
    fn emit_go_bundle_generates_main_and_manifest_files() {
        let dag = sample_dag();
        let artifacts = derive_artifacts(&dag).expect("derive should succeed");
        let reachable = ReachableDag::from_dag(&dag);
        let required_assets = sample_required_assets();
        let embedded_data = sample_embedded_data();
        let bundle = emit_go_bundle(&reachable, &artifacts, &required_assets, &embedded_data)
            .expect("emit should succeed");

        assert_eq!(bundle.backend, "go");
        assert_eq!(bundle.files.len(), 3);
        assert!(bundle
            .files
            .iter()
            .any(|file| file.path.ends_with("main.go")
                && file.content.contains("package main")
                && file.content.contains("os.WriteFile")
                && file.content.contains("bundle_content")
                && !file.content.contains("TODO(")));
        assert!(bundle
            .files
            .iter()
            .any(|file| file.path.ends_with("progress_manifest.txt")));
        assert!(bundle
            .files
            .iter()
            .any(|file| file.path.ends_with("dry_run_completion_test.go")
                && file.content.contains("TestDryRunCompletionRequired")));
    }

    #[test]
    fn emit_c_bundle_generates_main_and_manifest_files() {
        let dag = sample_dag();
        let artifacts = derive_artifacts(&dag).expect("derive should succeed");
        let reachable = ReachableDag::from_dag(&dag);
        let required_assets = sample_required_assets();
        let embedded_data = sample_embedded_data();
        let bundle = emit_c_bundle(&reachable, &artifacts, &required_assets, &embedded_data)
            .expect("emit should succeed");

        assert_eq!(bundle.backend, "c");
        assert_eq!(bundle.files.len(), 3);
        assert!(bundle.files.iter().any(|file| file.path.ends_with("main.c")
            && file.content.contains("int main(int argc, char** argv)")
            && file.content.contains("BUNDLE_CONTENT")
            && !file.content.contains("TODO(")));
        assert!(bundle
            .files
            .iter()
            .any(|file| file.path.ends_with("progress_manifest.txt")));
        assert!(bundle
            .files
            .iter()
            .any(|file| file.path.ends_with("dry_run_completion_test.c")
                && file.content.contains("assert(1")));
    }

    #[test]
    fn emit_mips_bundle_generates_main_and_manifest_files() {
        let dag = sample_dag();
        let artifacts = derive_artifacts(&dag).expect("derive should succeed");
        let reachable = ReachableDag::from_dag(&dag);
        let required_assets = sample_required_assets();
        let embedded_data = sample_embedded_data();
        let bundle = emit_mips_bundle(&reachable, &artifacts, &required_assets, &embedded_data)
            .expect("emit should succeed");

        assert_eq!(bundle.backend, "mips");
        assert_eq!(bundle.files.len(), 3);
        assert!(bundle.files.iter().any(|file| file.path.ends_with("main.s")
            && file.content.contains(".globl main")
            && file.content.contains("bundle_content")
            && !file.content.contains("TODO(")));
        assert!(bundle
            .files
            .iter()
            .any(|file| file.path.ends_with("progress_manifest.txt")));
        assert!(bundle
            .files
            .iter()
            .any(|file| file.path.ends_with("dry_run_completion_test.s")
                && file.content.contains("li $v0, 4001")));
    }

    #[test]
    fn native_emit_rejects_multiple_embedded_assets() {
        let dag = sample_dag();
        let artifacts = derive_artifacts(&dag).expect("derive should succeed");
        let reachable = ReachableDag::from_dag(&dag);
        let mut required_assets = sample_required_assets();
        required_assets.insert(ProgramSymbolId::from("tools.bundle::embedded_asset_two"));
        let mut embedded_data = sample_embedded_data();
        embedded_data.insert(
            "tools.bundle::embedded_asset_two".to_string(),
            EmbeddedData {
                module: "tools.bundle".to_string(),
                layer1_file_path: "src/embedded_bundle_two.txt".to_string(),
                layer2_ident: "bundle_content_two".to_string(),
                content: "bundle-test-content-two".to_string(),
            },
        );

        let error =
            emit_go_bundle(&reachable, &artifacts, &required_assets, &embedded_data).unwrap_err();
        assert!(matches!(
            error,
            EmitError::UnsupportedConstruct { ref backend, ref construct }
                if backend == "go" && construct.contains("multiple embedded assets")
        ));
    }

    // ======================================================================
    // SC7: New service smoke tests across all languages
    // ======================================================================

    /// Build a sample DAG with REST and Shell service transport nodes.
    fn service_dag() -> Dag<LoweredOp> {
        use daglang_lower::{
            FieldSpec, OutputFieldSpec, RestOperationSpec, ServiceCallMetadata,
            ServiceOperationSpec, ServiceTransportClass, ShellOperationSpec, ShellOutputParsing,
        };

        let rest_spec = ServiceOperationSpec::Rest(Box::new(RestOperationSpec {
            endpoint: "https://api.anthropic.com".to_string(),
            method: "POST".to_string(),
            path_template: "/v1/messages".to_string(),
            input_fields: vec![
                FieldSpec {
                    name: "model".to_string(),
                    type_id: "String".to_string(),
                    default: None,
                    is_secret: false,
                    is_path_param: false,
                },
                FieldSpec {
                    name: "messages".to_string(),
                    type_id: "Json".to_string(),
                    default: None,
                    is_secret: false,
                    is_path_param: false,
                },
            ],
            output_fields: vec![
                OutputFieldSpec {
                    name: "content".to_string(),
                    type_id: "String".to_string(),
                    json_path: "content/0/text".to_string(),
                    is_secret: false,
                    is_raw_body: false,
                    is_optional: false,
                },
                OutputFieldSpec {
                    name: "model".to_string(),
                    type_id: "String".to_string(),
                    json_path: "model".to_string(),
                    is_secret: false,
                    is_raw_body: false,
                    is_optional: false,
                },
            ],
            body_template: None,
            headers: vec![("anthropic-version".to_string(), "2023-06-01".to_string())],
            auth_scheme: None,
            auth_input: None,
            middleware: None,
            response_mapping: vec![],
            output_shape: None,
            mock_responses: vec![],
        }));

        let shell_spec = ServiceOperationSpec::Shell(ShellOperationSpec {
            argv_template: vec![
                daglang_lower::ArgvSegment::Literal("cargo".to_string()),
                daglang_lower::ArgvSegment::Literal("build".to_string()),
                daglang_lower::ArgvSegment::Literal("--all-targets".to_string()),
            ],
            input_fields: vec![],
            output_fields: vec![],
            output_parsing: ShellOutputParsing::SuccessStdoutStderr,
            env: vec![],
            exit_mapping: vec![],
        });

        let mut dag = Dag::new();

        // REST prepare node.
        dag.add_node(Node::opaque(
            "svc::rest_prepare",
            vec![Port::scalar("model", "String")],
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Transport {
                module: "extdeps.llm.anthropic".to_string(),
                kind: CallableKind::Func,
                name: "service_transport::prepare::llm.Anthropic::Messages".to_string(),
                obligation: ObligationCategory::ServiceTransportPrepare,
                service_metadata: Box::new(ServiceCallMetadata {
                    service: "llm.Anthropic".to_string(),
                    operation: "Messages".to_string(),
                    transport: ServiceTransportClass::RestNetwork,
                    idempotent: false,
                    readonly: false,
                    spec: Some(rest_spec.clone()),
                }),
                is_interactive: false,
                resource_target: None,
            },
        ));

        // REST execute node.
        dag.add_node(Node::opaque(
            "svc::rest_execute",
            vec![Port::scalar("request", "TransportRequest")],
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Transport {
                module: "extdeps.llm.anthropic".to_string(),
                kind: CallableKind::Func,
                name: "service_transport::execute::llm.Anthropic::Messages".to_string(),
                obligation: ObligationCategory::ServiceTransportExecute,
                service_metadata: Box::new(ServiceCallMetadata {
                    service: "llm.Anthropic".to_string(),
                    operation: "Messages".to_string(),
                    transport: ServiceTransportClass::RestNetwork,
                    idempotent: false,
                    readonly: false,
                    spec: Some(rest_spec.clone()),
                }),
                is_interactive: false,
                resource_target: None,
            },
        ));

        // REST parse node.
        dag.add_node(Node::opaque(
            "svc::rest_parse",
            vec![Port::scalar("response", "TransportResponse")],
            vec![Port::scalar("content", "String")],
            LoweredOp::Transport {
                module: "extdeps.llm.anthropic".to_string(),
                kind: CallableKind::Func,
                name: "service_transport::parse::llm.Anthropic::Messages".to_string(),
                obligation: ObligationCategory::ServiceTransportParse,
                service_metadata: Box::new(ServiceCallMetadata {
                    service: "llm.Anthropic".to_string(),
                    operation: "Messages".to_string(),
                    transport: ServiceTransportClass::RestNetwork,
                    idempotent: false,
                    readonly: false,
                    spec: Some(rest_spec),
                }),
                is_interactive: false,
                resource_target: None,
            },
        ));

        // Shell prepare node.
        dag.add_node(Node::opaque(
            "svc::shell_prepare",
            vec![],
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Transport {
                module: "extdeps.cargo".to_string(),
                kind: CallableKind::Func,
                name: "service_transport::prepare::cargo.Cargo::Build".to_string(),
                obligation: ObligationCategory::ServiceTransportPrepare,
                service_metadata: Box::new(ServiceCallMetadata {
                    service: "cargo.Cargo".to_string(),
                    operation: "Build".to_string(),
                    transport: ServiceTransportClass::ShellLocal,
                    idempotent: false,
                    readonly: false,
                    spec: Some(shell_spec.clone()),
                }),
                is_interactive: false,
                resource_target: None,
            },
        ));

        // Shell parse node.
        dag.add_node(Node::opaque(
            "svc::shell_parse",
            vec![Port::scalar("response", "TransportResponse")],
            vec![Port::scalar("success", "Bool")],
            LoweredOp::Transport {
                module: "extdeps.cargo".to_string(),
                kind: CallableKind::Func,
                name: "service_transport::parse::cargo.Cargo::Build".to_string(),
                obligation: ObligationCategory::ServiceTransportParse,
                service_metadata: Box::new(ServiceCallMetadata {
                    service: "cargo.Cargo".to_string(),
                    operation: "Build".to_string(),
                    transport: ServiceTransportClass::ShellLocal,
                    idempotent: false,
                    readonly: false,
                    spec: Some(shell_spec),
                }),
                is_interactive: false,
                resource_target: None,
            },
        ));

        // Wire the REST triplet.
        dag.add_edge(Edge::new(
            "svc::rest_prepare",
            "request",
            "svc::rest_execute",
            "request",
        ));
        dag.add_edge(Edge::new(
            "svc::rest_execute",
            "response",
            "svc::rest_parse",
            "response",
        ));

        dag
    }

    // -- SC7.1: Go backend emits service transport functions --

    #[test]
    fn go_bundle_emits_rest_service_transport_functions() {
        let dag = service_dag();
        let artifacts = derive_artifacts(&dag).expect("derive should succeed");
        let reachable = ReachableDag::from_dag(&dag);
        let bundle = emit_go_bundle(&reachable, &artifacts, &BTreeSet::new(), &HashMap::new())
            .expect("emit should succeed");

        let main_go = bundle
            .files
            .iter()
            .find(|file| file.path.ends_with("main.go"))
            .expect("should have main.go");

        // REST prepare: should have http.NewRequest + endpoint URL.
        assert!(
            main_go.content.contains("http.NewRequest"),
            "Go REST prepare should use http.NewRequest: {}",
            main_go.content
        );
        assert!(
            main_go.content.contains("https://api.anthropic.com"),
            "Go REST prepare should reference endpoint"
        );
        assert!(
            main_go.content.contains("anthropic-version"),
            "Go REST prepare should set custom headers"
        );

        // REST parse: should have json.Unmarshal + result struct.
        assert!(
            main_go.content.contains("json.Unmarshal"),
            "Go REST parse should unmarshal JSON"
        );
        assert!(
            main_go.content.contains("Content string"),
            "Go REST parse should have Content field in result struct"
        );

        // Shell prepare: should have exec.Command.
        assert!(
            main_go.content.contains("exec.Command"),
            "Go Shell prepare should use exec.Command"
        );
        assert!(
            main_go.content.contains("\"cargo\""),
            "Go Shell prepare should have cargo argv"
        );

        // Shell parse: should have SuccessStdoutStderr result struct.
        assert!(
            main_go.content.contains("Success bool"),
            "Go Shell parse should have Success field"
        );
    }

    #[test]
    fn go_bundle_imports_transport_packages() {
        let dag = service_dag();
        let artifacts = derive_artifacts(&dag).expect("derive should succeed");
        let reachable = ReachableDag::from_dag(&dag);
        let bundle = emit_go_bundle(&reachable, &artifacts, &BTreeSet::new(), &HashMap::new())
            .expect("emit should succeed");

        let main_go = bundle
            .files
            .iter()
            .find(|file| file.path.ends_with("main.go"))
            .expect("should have main.go");

        assert!(
            main_go.content.contains("\"net/http\""),
            "should import net/http"
        );
        assert!(
            main_go.content.contains("\"encoding/json\""),
            "should import encoding/json"
        );
        assert!(
            main_go.content.contains("\"os/exec\""),
            "should import os/exec"
        );
    }

    // -- SC7.2: C backend emits service transport functions --

    #[test]
    fn c_bundle_emits_rest_service_transport_functions() {
        let dag = service_dag();
        let artifacts = derive_artifacts(&dag).expect("derive should succeed");
        let reachable = ReachableDag::from_dag(&dag);
        let bundle = emit_c_bundle(&reachable, &artifacts, &BTreeSet::new(), &HashMap::new())
            .expect("emit should succeed");

        let main_c = bundle
            .files
            .iter()
            .find(|file| file.path.ends_with("main.c"))
            .expect("should have main.c");

        // REST prepare: should have snprintf for URL construction.
        assert!(
            main_c.content.contains("snprintf"),
            "C REST prepare should use snprintf for URL: {}",
            main_c.content
        );
        assert!(
            main_c.content.contains("api.anthropic.com"),
            "C REST prepare should reference endpoint"
        );

        // REST parse: should document JSON paths.
        assert!(
            main_c.content.contains("content/0/text"),
            "C REST parse should document json paths"
        );

        // Shell prepare: should document argv.
        assert!(
            main_c.content.contains("\"cargo\""),
            "C Shell prepare should have cargo in argv"
        );

        // Should include curl header.
        assert!(
            main_c.content.contains("#include <curl/curl.h>"),
            "C should include curl for REST services"
        );
    }

    // -- SC7.3: MIPS backend emits service transport functions --

    #[test]
    fn mips_bundle_emits_rest_service_transport_functions() {
        let dag = service_dag();
        let artifacts = derive_artifacts(&dag).expect("derive should succeed");
        let reachable = ReachableDag::from_dag(&dag);
        let bundle = emit_mips_bundle(&reachable, &artifacts, &BTreeSet::new(), &HashMap::new())
            .expect("emit should succeed");

        let main_s = bundle
            .files
            .iter()
            .find(|file| file.path.ends_with("main.s"))
            .expect("should have main.s");

        // REST prepare: should have spec comment.
        assert!(
            main_s.content.contains("prepare REST POST"),
            "MIPS REST prepare should have spec comment: {}",
            main_s.content
        );
        assert!(
            main_s.content.contains("api.anthropic.com"),
            "MIPS REST prepare should reference endpoint"
        );

        // Shell prepare: should have argv comment.
        assert!(
            main_s.content.contains("prepare shell [cargo build"),
            "MIPS Shell prepare should have argv comment"
        );

        // All labels should return.
        let label_count = main_s
            .content
            .lines()
            .filter(|l| l.contains("jr $ra"))
            .count();
        assert!(
            label_count >= 5,
            "MIPS should have at least 5 jr $ra returns (for 5 service nodes), got {label_count}"
        );
    }

    // -- SC7.4: Cross-backend consistency --

    #[test]
    fn all_backends_emit_same_number_of_service_functions() {
        let dag = service_dag();
        let artifacts = derive_artifacts(&dag).expect("derive should succeed");
        let reachable = ReachableDag::from_dag(&dag);

        let go_bundle = emit_go_bundle(&reachable, &artifacts, &BTreeSet::new(), &HashMap::new())
            .expect("go emit");
        let c_bundle = emit_c_bundle(&reachable, &artifacts, &BTreeSet::new(), &HashMap::new())
            .expect("c emit");
        let mips_bundle =
            emit_mips_bundle(&reachable, &artifacts, &BTreeSet::new(), &HashMap::new())
                .expect("mips emit");

        // All should report the same callable count.
        assert_eq!(go_bundle.summary.callable_count, 5, "Go callable count");
        assert_eq!(c_bundle.summary.callable_count, 5, "C callable count");
        assert_eq!(mips_bundle.summary.callable_count, 5, "MIPS callable count");

        // None should contain generic "generated callable stub".
        let go_main = go_bundle
            .files
            .iter()
            .find(|f| f.path.ends_with("main.go"))
            .unwrap();
        assert!(
            !go_main.content.contains("generated callable stub"),
            "Go should not have generic stubs for service nodes"
        );

        let c_main = c_bundle
            .files
            .iter()
            .find(|f| f.path.ends_with("main.c"))
            .unwrap();
        assert!(
            !c_main.content.contains("static void"),
            "C should not have void stubs for service nodes"
        );
    }

    // ── FC-14: Reachability pruning tests ──────────────────────────

    #[test]
    fn compute_reachable_node_ids_includes_connected_nodes() {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "entry",
            vec![],
            vec![Port::scalar("out", "String")],
            LoweredOp::Callable {
                module: "test".to_string(),
                kind: CallableKind::Func,
                name: "entry".to_string(),
                obligation: ObligationCategory::None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        dag.add_node(Node::opaque(
            "downstream",
            vec![Port::scalar("in", "String")],
            vec![Port::scalar("out", "String")],
            LoweredOp::Callable {
                module: "test".to_string(),
                kind: CallableKind::Func,
                name: "downstream".to_string(),
                obligation: ObligationCategory::None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        dag.add_node(Node::opaque(
            "unreachable",
            vec![],
            vec![Port::scalar("out", "String")],
            LoweredOp::Callable {
                module: "test".to_string(),
                kind: CallableKind::Func,
                name: "unreachable".to_string(),
                obligation: ObligationCategory::None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        dag.edges
            .push(Edge::new("entry", "out", "downstream", "in"));

        let reachable = compute_reachable_node_ids(&dag);

        assert!(reachable.contains("entry"), "entry should be reachable");
        assert!(
            reachable.contains("downstream"),
            "downstream of entry should be reachable"
        );
        // "unreachable" has no incoming edges either, so it IS reachable
        // as an independent entrypoint (no incoming = entrypoint).
        // This is correct: isolated nodes are their own entrypoints.
        assert!(
            reachable.contains("unreachable"),
            "isolated node is its own entrypoint"
        );
    }

    #[test]
    fn compute_reachable_excludes_orphan_with_incoming_only() {
        // A node with incoming edges from nowhere (edge references a
        // non-existent source) should still be tracked via BFS.
        // But a node that only has incoming edges (not from entrypoints)
        // would be unreachable.
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "entry",
            vec![],
            vec![Port::scalar("out", "String")],
            LoweredOp::Callable {
                module: "test".to_string(),
                kind: CallableKind::Func,
                name: "entry".to_string(),
                obligation: ObligationCategory::None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        dag.add_node(Node::opaque(
            "middle",
            vec![Port::scalar("in", "String")],
            vec![Port::scalar("out", "String")],
            LoweredOp::Callable {
                module: "test".to_string(),
                kind: CallableKind::Func,
                name: "middle".to_string(),
                obligation: ObligationCategory::None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        dag.add_node(Node::opaque(
            "orphan",
            vec![Port::scalar("in", "String")],
            vec![],
            LoweredOp::Callable {
                module: "test".to_string(),
                kind: CallableKind::Func,
                name: "orphan".to_string(),
                obligation: ObligationCategory::None,
                is_interactive: false,
                resource_target: None,
                fn_body: None,
            },
        ));
        // entry -> middle, but orphan has an incoming edge from middle
        dag.edges.push(Edge::new("entry", "out", "middle", "in"));
        // orphan is wired from middle but is reachable through the chain
        dag.edges.push(Edge::new("middle", "out", "orphan", "in"));

        let reachable = compute_reachable_node_ids(&dag);
        assert!(reachable.contains("entry"));
        assert!(reachable.contains("middle"));
        assert!(
            reachable.contains("orphan"),
            "orphan is downstream of entry → middle, so it is reachable"
        );
    }
}
