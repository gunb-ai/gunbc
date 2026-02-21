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
//! tools/makegen.dag
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

#[cfg(test)]
mod backend_harness;

use daglang_derive::{DerivedArtifacts, ProgressManifest};
use daglang_lower::{CallableKind, LoweredOp, ServiceOperationSpec};
use gunbc_ir::Dag;
use std::fmt::Write as _;

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
pub fn emit_rust_bundle(
    dag: &Dag<LoweredOp>,
    artifacts: &DerivedArtifacts,
) -> Result<EmissionBundle, EmitError> {
    let backend = RustBackend;
    let mut emitted_functions = Vec::new();
    let mut callable_count = 0usize;
    let mut pipeline_count = 0usize;

    for node in &dag.nodes {
        let Some(op) = node.body.as_opaque() else {
            return Err(EmitError::InvalidLoweredNode(format!(
                "subdag node `{}` is not supported in phase-1 emit",
                node.id.0
            )));
        };

        match op {
            LoweredOp::Callable {
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
        }
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
    dag: &Dag<LoweredOp>,
    artifacts: &DerivedArtifacts,
    makegen_content: Option<&str>,
) -> Result<EmissionBundle, EmitError> {
    let (symbols, callable_count, pipeline_count) = collect_symbols_with_metadata(dag)?;
    let manifest_rendered = render_manifest(&artifacts.manifest);
    let is_makegen = is_makegen_module(artifacts);
    let entrypoints = artifacts
        .manifest
        .entrypoint_nodes
        .iter()
        .map(|entry| sanitize_identifier(entry))
        .collect::<Vec<_>>();

    let has_service_transport = symbols.iter().any(|s| s.spec.is_some());

    let symbol_funcs = symbols
        .iter()
        .map(|sym| {
            if let Some(ref spec) = sym.spec {
                service_emit::emit_go_service_func(&sym.name, &sym.raw_name, spec)
            } else {
                format!(
                    "func {name}() {{\n    // generated callable stub\n}}\n",
                    name = sym.name
                )
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let entrypoint_lits = entrypoints
        .iter()
        .map(|entry| format!("\"{entry}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let imports = if is_makegen {
        "import (\n    \"fmt\"\n    \"os\"\n)\n".to_string()
    } else if has_service_transport {
        "import (\n    \"fmt\"\n    \"net/http\"\n    \"bytes\"\n    \"encoding/json\"\n    \"os/exec\"\n    \"strings\"\n)\n".to_string()
    } else {
        "import \"fmt\"\n".to_string()
    };

    let main_go = if is_makegen {
        let makefile_literal = escape_string_literal(resolve_makegen_content(makegen_content));
        format!(
            "package main\n\n{imports}\nfunc cliEntrypoints() []string {{\n    return []string{{{entrypoint_lits}}}\n}}\n\n{symbol_funcs}\nfunc makegenContent() string {{\n    return \"{makefile_literal}\"\n}}\n\nfunc main() {{\n    if len(os.Args) > 1 {{\n        path := os.Args[1]\n        if err := os.WriteFile(path, []byte(makegenContent()), 0644); err != nil {{\n            fmt.Fprintf(os.Stderr, \"failed to write `%s`: %v\\n\", path, err)\n            os.Exit(1)\n        }}\n    }}\n    fmt.Println(\"daglang generated go backend\")\n}}\n"
        )
    } else {
        format!(
            "package main\n\n{imports}\nfunc cliEntrypoints() []string {{\n    return []string{{{entrypoint_lits}}}\n}}\n\n{symbol_funcs}\nfunc main() {{\n    fmt.Println(\"daglang generated go backend\")\n}}\n"
        )
    };

    // Suppress unused import warnings for service transport code.
    let main_go = if has_service_transport && !is_makegen {
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
    dag: &Dag<LoweredOp>,
    artifacts: &DerivedArtifacts,
    makegen_content: Option<&str>,
) -> Result<EmissionBundle, EmitError> {
    let (symbols, callable_count, pipeline_count) = collect_symbols_with_metadata(dag)?;
    let manifest_rendered = render_manifest(&artifacts.manifest);
    let is_makegen = is_makegen_module(artifacts);
    let has_service_transport = symbols.iter().any(|s| s.spec.is_some());
    let entrypoints = artifacts
        .manifest
        .entrypoint_nodes
        .iter()
        .map(|entry| sanitize_identifier(entry))
        .collect::<Vec<_>>();

    let symbol_funcs = symbols
        .iter()
        .map(|sym| {
            if let Some(ref spec) = sym.spec {
                service_emit::emit_c_service_func(&sym.name, &sym.raw_name, spec)
            } else {
                format!("static void {name}(void) {{}}\n", name = sym.name)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let entrypoint_defs = entrypoints
        .iter()
        .map(|entry| format!("\"{entry}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let includes = if is_makegen {
        "#include <stdio.h>\n#include <string.h>\n".to_string()
    } else if has_service_transport {
        "#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n#include <curl/curl.h>\n"
            .to_string()
    } else {
        "#include <stdio.h>\n".to_string()
    };

    let main_c = if is_makegen {
        let makefile_literal = escape_string_literal(resolve_makegen_content(makegen_content));
        format!(
            "{includes}\nstatic const char* CLI_ENTRYPOINTS[] = {{{entrypoint_defs}}};\nstatic const char* MAKEGEN_CONTENT = \"{makefile_literal}\";\n\n{symbol_funcs}\nint main(int argc, char** argv) {{\n    (void)CLI_ENTRYPOINTS;\n    if (argc > 1) {{\n        const char* path = argv[1];\n        FILE* file = fopen(path, \"wb\");\n        if (!file) {{\n            fprintf(stderr, \"failed to write `%s`\\n\", path);\n            return 1;\n        }}\n        size_t expected = strlen(MAKEGEN_CONTENT);\n        size_t written = fwrite(MAKEGEN_CONTENT, 1, expected, file);\n        fclose(file);\n        if (written != expected) {{\n            fprintf(stderr, \"failed to write `%s`\\n\", path);\n            return 1;\n        }}\n    }}\n    printf(\"daglang generated c backend\\n\");\n    return 0;\n}}\n"
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
    dag: &Dag<LoweredOp>,
    artifacts: &DerivedArtifacts,
    makegen_content: Option<&str>,
) -> Result<EmissionBundle, EmitError> {
    let (symbols, callable_count, pipeline_count) = collect_symbols_with_metadata(dag)?;
    let manifest_rendered = render_manifest(&artifacts.manifest);
    let is_makegen = is_makegen_module(artifacts);

    let label_defs = symbols
        .iter()
        .map(|sym| {
            if let Some(ref spec) = sym.spec {
                service_emit::emit_mips_service_func(&sym.name, &sym.raw_name, spec)
            } else {
                format!("{name}:\n    jr $ra\n", name = sym.name)
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    let main_s = if is_makegen {
        let content = resolve_makegen_content(makegen_content);
        let makefile_bytes = content
            .as_bytes()
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            ".text\n.globl main\n\n{label_defs}\nmain:\n    li $a0, 1\n    la $a1, makegen_content\n    li $a2, {}\n    li $v0, 4004\n    syscall\n    li $a0, 0\n    li $v0, 4001\n    syscall\n\n.data\nmakegen_content:\n    .byte {makefile_bytes}\n",
            content.len()
        )
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
    /// The raw lowered-op name (e.g., "service_transport::prepare::github.Gist::Create").
    raw_name: String,
}

fn collect_symbols_with_metadata(
    dag: &Dag<LoweredOp>,
) -> Result<(Vec<CollectedSymbol>, usize, usize), EmitError> {
    let mut symbols = Vec::new();
    let mut callable_count = 0usize;
    let mut pipeline_count = 0usize;

    for node in &dag.nodes {
        let Some(op) = node.body.as_opaque() else {
            return Err(EmitError::InvalidLoweredNode(format!(
                "subdag node `{}` is not supported in backend emit",
                node.id.0
            )));
        };

        match op {
            LoweredOp::Callable {
                module,
                name,
                service_metadata,
                ..
            } => {
                callable_count += 1;
                let spec = service_metadata.as_ref().and_then(|m| m.spec.clone());
                symbols.push(CollectedSymbol {
                    name: sanitize_identifier(&format!("{module}_{name}")),
                    spec,
                    raw_name: name.clone(),
                });
            }
            LoweredOp::Primitive { module, name, .. } => {
                callable_count += 1;
                symbols.push(CollectedSymbol {
                    name: sanitize_identifier(&format!("{module}_{name}")),
                    spec: None,
                    raw_name: name.clone(),
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
                    raw_name: String::new(),
                });
            }
            LoweredOp::Pipeline { module, name, .. } => {
                pipeline_count += 1;
                symbols.push(CollectedSymbol {
                    name: sanitize_identifier(&format!("{module}_{name}")),
                    spec: None,
                    raw_name: String::new(),
                });
            }
        }
    }

    Ok((symbols, callable_count, pipeline_count))
}

fn is_makegen_module(artifacts: &DerivedArtifacts) -> bool {
    artifacts
        .tool_metadata
        .modules
        .iter()
        .any(|module| module.module == "tools.makegen")
}

fn resolve_makegen_content(override_content: Option<&str>) -> &str {
    override_content.unwrap_or(MAKEGEN_STUB_CONTENT)
}

const MAKEGEN_STUB_CONTENT: &str =
    "# Generated by daglang\n.PHONY: makegen\n\nmakegen:\n\tcargo run -p gunbc-dag --bin gunbc-makegen\n";

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
            gunbc_ir::node::NodeBody::SubDag(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_derive::derive_artifacts;
    use daglang_lower::ObligationCategory;
    use gunbc_ir::{Edge, Node, Port};

    fn sample_dag() -> Dag<LoweredOp> {
        let mut dag = Dag::new();
        dag.add_node(Node::opaque(
            "tools.makegen::render_makefile",
            vec![Port::scalar("registry", "ToolRegistry")],
            vec![Port::scalar("return", "String")],
            LoweredOp::Callable {
                module: "tools.makegen".to_string(),
                kind: CallableKind::Fn,
                name: "render_makefile".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
            },
        ));
        dag.add_node(Node::opaque(
            "tools.makegen::makegen",
            vec![Port::scalar("registry", "ToolRegistry")],
            vec![Port::scalar("written", "Bool")],
            LoweredOp::Callable {
                module: "tools.makegen".to_string(),
                kind: CallableKind::Func,
                name: "makegen".to_string(),
                obligation: ObligationCategory::None,
                service_metadata: None,
                is_interactive: false,
                resource_target: None,
            },
        ));
        dag.add_edge(Edge::new(
            "tools.makegen::render_makefile",
            "return",
            "tools.makegen::makegen",
            "registry",
        ));
        dag
    }

    #[test]
    fn emit_rust_bundle_generates_main_and_manifest_files() {
        let dag = sample_dag();
        let artifacts = derive_artifacts(&dag).expect("derive should succeed");
        let bundle = emit_rust_bundle(&dag, &artifacts).expect("emit should succeed");

        assert_eq!(bundle.backend, "rust");
        assert_eq!(bundle.files.len(), 3);
        assert!(bundle
            .files
            .iter()
            .any(|file| file.path.ends_with("main.rs")
                && file.content.contains("tools_makegen_makegen")
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
        let bundle = emit_go_bundle(&dag, &artifacts, None).expect("emit should succeed");

        assert_eq!(bundle.backend, "go");
        assert_eq!(bundle.files.len(), 3);
        assert!(bundle
            .files
            .iter()
            .any(|file| file.path.ends_with("main.go")
                && file.content.contains("package main")
                && file.content.contains("os.WriteFile")
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
        let bundle = emit_c_bundle(&dag, &artifacts, None).expect("emit should succeed");

        assert_eq!(bundle.backend, "c");
        assert_eq!(bundle.files.len(), 3);
        assert!(bundle.files.iter().any(|file| file.path.ends_with("main.c")
            && file.content.contains("int main(int argc, char** argv)")
            && file.content.contains("MAKEGEN_CONTENT")
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
        let bundle = emit_mips_bundle(&dag, &artifacts, None).expect("emit should succeed");

        assert_eq!(bundle.backend, "mips");
        assert_eq!(bundle.files.len(), 3);
        assert!(bundle.files.iter().any(|file| file.path.ends_with("main.s")
            && file.content.contains(".globl main")
            && file.content.contains("makegen_content")
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

    // ======================================================================
    // SC7: New service smoke tests across all languages
    // ======================================================================

    /// Build a sample DAG with REST and Shell service transport nodes.
    fn service_dag() -> Dag<LoweredOp> {
        use daglang_lower::{
            FieldSpec, OutputFieldSpec, RestOperationSpec, ServiceCallMetadata,
            ServiceOperationSpec, ServiceTransportClass, ShellOperationSpec, ShellOutputParsing,
        };

        let rest_spec = ServiceOperationSpec::Rest(RestOperationSpec {
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
                },
                OutputFieldSpec {
                    name: "model".to_string(),
                    type_id: "String".to_string(),
                    json_path: "model".to_string(),
                    is_secret: false,
                    is_raw_body: false,
                },
            ],
            body_template: None,
            headers: vec![("anthropic-version".to_string(), "2023-06-01".to_string())],
        });

        let shell_spec = ServiceOperationSpec::Shell(ShellOperationSpec {
            argv_template: vec![
                daglang_lower::ArgvSegment::Literal("cargo".to_string()),
                daglang_lower::ArgvSegment::Literal("build".to_string()),
                daglang_lower::ArgvSegment::Literal("--all-targets".to_string()),
            ],
            input_fields: vec![],
            output_fields: vec![],
            output_parsing: ShellOutputParsing::SuccessStdoutStderr,
        });

        let mut dag = Dag::new();

        // REST prepare node.
        dag.add_node(Node::opaque(
            "svc::rest_prepare",
            vec![Port::scalar("model", "String")],
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Callable {
                module: "services.llm.anthropic".to_string(),
                kind: CallableKind::Func,
                name: "service_transport::prepare::llm.Anthropic::Messages".to_string(),
                obligation: ObligationCategory::ServiceTransportPrepare,
                service_metadata: Some(Box::new(ServiceCallMetadata {
                    service: "llm.Anthropic".to_string(),
                    operation: "Messages".to_string(),
                    transport: ServiceTransportClass::RestNetwork,
                    idempotent: false,
                    readonly: false,
                    permissions: vec!["messages".to_string()],
                    spec: Some(rest_spec.clone()),
                })),
                is_interactive: false,
                resource_target: None,
            },
        ));

        // REST execute node.
        dag.add_node(Node::opaque(
            "svc::rest_execute",
            vec![Port::scalar("request", "TransportRequest")],
            vec![Port::scalar("response", "TransportResponse")],
            LoweredOp::Callable {
                module: "services.llm.anthropic".to_string(),
                kind: CallableKind::Func,
                name: "service_transport::execute::llm.Anthropic::Messages".to_string(),
                obligation: ObligationCategory::ServiceTransportExecute,
                service_metadata: Some(Box::new(ServiceCallMetadata {
                    service: "llm.Anthropic".to_string(),
                    operation: "Messages".to_string(),
                    transport: ServiceTransportClass::RestNetwork,
                    idempotent: false,
                    readonly: false,
                    permissions: vec!["messages".to_string()],
                    spec: Some(rest_spec.clone()),
                })),
                is_interactive: false,
                resource_target: None,
            },
        ));

        // REST parse node.
        dag.add_node(Node::opaque(
            "svc::rest_parse",
            vec![Port::scalar("response", "TransportResponse")],
            vec![Port::scalar("content", "String")],
            LoweredOp::Callable {
                module: "services.llm.anthropic".to_string(),
                kind: CallableKind::Func,
                name: "service_transport::parse::llm.Anthropic::Messages".to_string(),
                obligation: ObligationCategory::ServiceTransportParse,
                service_metadata: Some(Box::new(ServiceCallMetadata {
                    service: "llm.Anthropic".to_string(),
                    operation: "Messages".to_string(),
                    transport: ServiceTransportClass::RestNetwork,
                    idempotent: false,
                    readonly: false,
                    permissions: vec!["messages".to_string()],
                    spec: Some(rest_spec),
                })),
                is_interactive: false,
                resource_target: None,
            },
        ));

        // Shell prepare node.
        dag.add_node(Node::opaque(
            "svc::shell_prepare",
            vec![],
            vec![Port::scalar("request", "TransportRequest")],
            LoweredOp::Callable {
                module: "services.cargo".to_string(),
                kind: CallableKind::Func,
                name: "service_transport::prepare::cargo.Cargo::Build".to_string(),
                obligation: ObligationCategory::ServiceTransportPrepare,
                service_metadata: Some(Box::new(ServiceCallMetadata {
                    service: "cargo.Cargo".to_string(),
                    operation: "Build".to_string(),
                    transport: ServiceTransportClass::ShellLocal,
                    idempotent: false,
                    readonly: false,
                    permissions: vec![],
                    spec: Some(shell_spec.clone()),
                })),
                is_interactive: false,
                resource_target: None,
            },
        ));

        // Shell parse node.
        dag.add_node(Node::opaque(
            "svc::shell_parse",
            vec![Port::scalar("response", "TransportResponse")],
            vec![Port::scalar("success", "Bool")],
            LoweredOp::Callable {
                module: "services.cargo".to_string(),
                kind: CallableKind::Func,
                name: "service_transport::parse::cargo.Cargo::Build".to_string(),
                obligation: ObligationCategory::ServiceTransportParse,
                service_metadata: Some(Box::new(ServiceCallMetadata {
                    service: "cargo.Cargo".to_string(),
                    operation: "Build".to_string(),
                    transport: ServiceTransportClass::ShellLocal,
                    idempotent: false,
                    readonly: false,
                    permissions: vec![],
                    spec: Some(shell_spec),
                })),
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
        let bundle = emit_go_bundle(&dag, &artifacts, None).expect("emit should succeed");

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
        let bundle = emit_go_bundle(&dag, &artifacts, None).expect("emit should succeed");

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
        let bundle = emit_c_bundle(&dag, &artifacts, None).expect("emit should succeed");

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
        let bundle = emit_mips_bundle(&dag, &artifacts, None).expect("emit should succeed");

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

        let go_bundle = emit_go_bundle(&dag, &artifacts, None).expect("go emit");
        let c_bundle = emit_c_bundle(&dag, &artifacts, None).expect("c emit");
        let mips_bundle = emit_mips_bundle(&dag, &artifacts, None).expect("mips emit");

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
}
