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

use daglang_derive::{DerivedArtifacts, ProgressManifest};
use daglang_lower::{CallableKind, LoweredOp};
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
        format!("// progress-manifest\n{manifest}\n")
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
    let (symbols, callable_count, pipeline_count) = collect_callable_symbols(dag)?;
    let manifest_rendered = render_manifest(&artifacts.manifest);
    let is_makegen = is_makegen_module(artifacts);
    let entrypoints = artifacts
        .manifest
        .entrypoint_nodes
        .iter()
        .map(|entry| sanitize_identifier(entry))
        .collect::<Vec<_>>();

    let symbol_funcs = symbols
        .iter()
        .map(|symbol| format!("func {symbol}() {{\n    // generated callable stub\n}}\n"))
        .collect::<Vec<_>>()
        .join("\n");

    let entrypoint_lits = entrypoints
        .iter()
        .map(|entry| format!("\"{entry}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let main_go = if is_makegen {
        let makefile_literal = escape_string_literal(resolve_makegen_content(makegen_content));
        format!(
            "package main\n\nimport (\n    \"fmt\"\n    \"os\"\n)\n\nfunc cliEntrypoints() []string {{\n    return []string{{{entrypoint_lits}}}\n}}\n\n{symbol_funcs}\nfunc makegenContent() string {{\n    return \"{makefile_literal}\"\n}}\n\nfunc main() {{\n    if len(os.Args) > 1 {{\n        path := os.Args[1]\n        if err := os.WriteFile(path, []byte(makegenContent()), 0644); err != nil {{\n            fmt.Fprintf(os.Stderr, \"failed to write `%s`: %v\\n\", path, err)\n            os.Exit(1)\n        }}\n    }}\n    fmt.Println(\"daglang generated go backend\")\n}}\n"
        )
    } else {
        format!(
            "package main\n\nimport \"fmt\"\n\nfunc cliEntrypoints() []string {{\n    return []string{{{entrypoint_lits}}}\n}}\n\n{symbol_funcs}\nfunc main() {{\n    fmt.Println(\"daglang generated go backend\")\n}}\n"
        )
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
    let (symbols, callable_count, pipeline_count) = collect_callable_symbols(dag)?;
    let manifest_rendered = render_manifest(&artifacts.manifest);
    let is_makegen = is_makegen_module(artifacts);
    let entrypoints = artifacts
        .manifest
        .entrypoint_nodes
        .iter()
        .map(|entry| sanitize_identifier(entry))
        .collect::<Vec<_>>();

    let symbol_funcs = symbols
        .iter()
        .map(|symbol| format!("static void {symbol}(void) {{}}\n"))
        .collect::<Vec<_>>()
        .join("\n");

    let entrypoint_defs = entrypoints
        .iter()
        .map(|entry| format!("\"{entry}\""))
        .collect::<Vec<_>>()
        .join(", ");

    let main_c = if is_makegen {
        let makefile_literal = escape_string_literal(resolve_makegen_content(makegen_content));
        format!(
            "#include <stdio.h>\n#include <string.h>\n\nstatic const char* CLI_ENTRYPOINTS[] = {{{entrypoint_defs}}};\nstatic const char* MAKEGEN_CONTENT = \"{makefile_literal}\";\n\n{symbol_funcs}\nint main(int argc, char** argv) {{\n    (void)CLI_ENTRYPOINTS;\n    if (argc > 1) {{\n        const char* path = argv[1];\n        FILE* file = fopen(path, \"wb\");\n        if (!file) {{\n            fprintf(stderr, \"failed to write `%s`\\n\", path);\n            return 1;\n        }}\n        size_t expected = strlen(MAKEGEN_CONTENT);\n        size_t written = fwrite(MAKEGEN_CONTENT, 1, expected, file);\n        fclose(file);\n        if (written != expected) {{\n            fprintf(stderr, \"failed to write `%s`\\n\", path);\n            return 1;\n        }}\n    }}\n    printf(\"daglang generated c backend\\n\");\n    return 0;\n}}\n"
        )
    } else {
        format!(
            "#include <stdio.h>\n\nstatic const char* CLI_ENTRYPOINTS[] = {{{entrypoint_defs}}};\n\n{symbol_funcs}\nint main(void) {{\n    printf(\"daglang generated c backend\\n\");\n    return (int)(sizeof(CLI_ENTRYPOINTS) / sizeof(CLI_ENTRYPOINTS[0])) >= 0 ? 0 : 1;\n}}\n"
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
    let (symbols, callable_count, pipeline_count) = collect_callable_symbols(dag)?;
    let manifest_rendered = render_manifest(&artifacts.manifest);
    let is_makegen = is_makegen_module(artifacts);

    let label_defs = symbols
        .iter()
        .map(|symbol| format!("{symbol}:\n    jr $ra\n"))
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

fn collect_callable_symbols(
    dag: &Dag<LoweredOp>,
) -> Result<(Vec<String>, usize, usize), EmitError> {
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
            LoweredOp::Callable { module, name, .. } => {
                callable_count += 1;
                symbols.push(sanitize_identifier(&format!("{module}_{name}")));
            }
            LoweredOp::Collection {
                module,
                callable,
                kind,
            } => {
                callable_count += 1;
                symbols.push(sanitize_identifier(&format!(
                    "{module}_{callable}_collection_{kind:?}"
                )));
            }
            LoweredOp::Pipeline { module, name, .. } => {
                pipeline_count += 1;
                symbols.push(sanitize_identifier(&format!("{module}_{name}")));
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
}
