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

use daglang_derive::{DerivedArtifacts, ProgressManifest};
use daglang_lower::{CallableKind, LoweredOp};
use gunbc_ir::Dag;

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
        format!("// TODO(type): emit `{ty}`")
    }

    fn emit_fn(&self, name: &str) -> String {
        format!(
            "pub fn {name}() {{\n    // TODO(fn): compile pure functor body\n}}\n"
        )
    }

    fn emit_func(&self, name: &str) -> String {
        format!(
            "pub fn {name}() {{\n    // TODO(func): execute lowered DAG orchestration\n}}\n"
        )
    }

    fn emit_transport(&self, spec: &str) -> String {
        format!("// TODO(transport): emit transport wiring for `{spec}`")
    }

    fn emit_test(&self, obligation: &str) -> String {
        format!(
            "#[test]\nfn obligation_{}() {{\n    // TODO(test): satisfy obligation\n    assert!(true);\n}}\n",
            sanitize_identifier(obligation)
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
            LoweredOp::Callable { module, kind, name } => {
                callable_count += 1;
                let fn_name = sanitize_identifier(&format!("{module}_{name}"));
                let rendered = match kind {
                    CallableKind::Fn => backend.emit_fn(&fn_name),
                    CallableKind::Func | CallableKind::Pattern => backend.emit_func(&fn_name),
                };
                emitted_functions.push(rendered);
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

    let files = vec![
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

fn render_manifest(manifest: &ProgressManifest) -> String {
    let mut out = String::new();
    out.push_str(&format!("total_nodes={}\n", manifest.total_nodes));
    out.push_str(&format!("total_edges={}\n", manifest.total_edges));
    out.push_str("waves=\n");
    for (idx, wave) in manifest.waves.iter().enumerate() {
        out.push_str(&format!("  [{idx}] {}\n", wave.join(", ")));
    }
    out.push_str(&format!(
        "entrypoint_nodes={}\n",
        manifest.entrypoint_nodes.join(", ")
    ));
    out.push_str(&format!(
        "boundary_nodes={}\n",
        manifest.boundary_nodes.join(", ")
    ));
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
    if out
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_digit())
    {
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
        assert_eq!(bundle.files.len(), 2);
        assert!(bundle
            .files
            .iter()
            .any(|file| file.path.ends_with("main.rs") && file.content.contains("tools_makegen_makegen")));
        assert!(bundle
            .files
            .iter()
            .any(|file| file.path.ends_with("progress_manifest.txt") && file.content.contains("total_nodes=")));
        assert_eq!(bundle.summary.callable_count, 2);
    }
}
