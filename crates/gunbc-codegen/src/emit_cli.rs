//! CLI scaffolding generator.
//!
//! Generates main.rs content for CLI entrypoints including:
//! - Clap struct generation
//! - Argument parsing
//! - DAG construction
//! - Execution boilerplate

use crate::emit_entrypoint::{
    extract_layer_name, find_all_boundaries_recursive, CliArgSpec, EntrypointInfo,
};
use gunbc_ir::Dag;

/// Configuration for CLI code generation.
#[derive(Debug, Clone)]
pub struct CliCodegenConfig {
    /// The name of the binary (used in clap about text).
    pub binary_name: String,
    /// Description for the CLI tool.
    pub description: String,
    /// The crate that provides the core DAG builder.
    pub core_crate: String,
    /// The function to call to build the DAG (fully qualified).
    pub dag_builder: String,
    /// The op type (fully qualified).
    pub op_type: String,
    /// Whether to include SVG visualization flags.
    pub include_viz_flags: bool,
}

impl Default for CliCodegenConfig {
    fn default() -> Self {
        Self {
            binary_name: "tool".into(),
            description: "A CLI tool".into(),
            core_crate: "core".into(),
            dag_builder: "core::build_dag".into(),
            op_type: "core::Op".into(),
            include_viz_flags: true,
        }
    }
}

/// Map a TypeId to a Rust type for clap.
fn type_id_to_rust_type(type_id: &str) -> &'static str {
    match type_id {
        "String" => "String",
        "Bool" => "bool",
        "Int" | "I64" => "i64",
        "Uint" | "U64" => "u64",
        "Float" | "F64" => "f64",
        "StrList" => "Vec<String>",
        _ => "String", // Default to String
    }
}

/// Map a TypeId to a clap default value representation.
fn type_id_default_value(type_id: &str, value: &str) -> String {
    match type_id {
        "Bool" => value.to_string(),
        _ => format!("\"{}\"", value),
    }
}

/// Generate the clap Cli struct from entrypoint info.
pub fn emit_cli_struct(info: &EntrypointInfo, config: &CliCodegenConfig) -> String {
    let mut out = String::new();

    out.push_str("use clap::Parser;\n\n");

    out.push_str(&format!(
        "#[derive(Parser, Debug)]\n#[command(name = \"{}\", about = \"{}\")]\n",
        config.binary_name, config.description
    ));
    out.push_str("struct Cli {\n");

    for arg in &info.cli_args {
        let rust_type = type_id_to_rust_type(&arg.type_id);
        let is_bool = rust_type == "bool";

        // Help text
        if let Some(help) = &arg.help {
            out.push_str(&format!("    /// {}\n", help));
        }

        // Clap attributes
        let mut attrs = Vec::new();

        if is_bool {
            attrs.push("long".to_string());
        } else if arg.required {
            // Required args are positional by default, or use --name for optional
            if let Some(default) = &arg.default {
                attrs.push(format!("default_value = {}", type_id_default_value(&arg.type_id, default)));
            }
        } else {
            attrs.push("long".to_string());
            if let Some(default) = &arg.default {
                attrs.push(format!("default_value = {}", type_id_default_value(&arg.type_id, default)));
            }
        }

        if !attrs.is_empty() {
            out.push_str(&format!("    #[arg({})]\n", attrs.join(", ")));
        }

        // Field
        let field_name = arg.name.replace('-', "_");
        // Bool flags are always `bool` (clap treats --flag as setting true)
        // Other types are Option if not required and no default
        if is_bool || arg.required || arg.default.is_some() {
            out.push_str(&format!("    {}: {},\n", field_name, rust_type));
        } else {
            out.push_str(&format!("    {}: Option<{}>,\n", field_name, rust_type));
        }
        out.push('\n');
    }

    // Add visualization flags if configured
    if config.include_viz_flags {
        out.push_str("    /// Emit node-level SVG graph to stdout and exit\n");
        out.push_str("    #[arg(long)]\n");
        out.push_str("    svg: bool,\n\n");

        out.push_str("    /// Emit tool-level SVG graph to stdout and exit\n");
        out.push_str("    #[arg(long)]\n");
        out.push_str("    svg_tools: bool,\n\n");

        out.push_str("    /// Include guard expressions on input ports in SVG output\n");
        out.push_str("    #[arg(long)]\n");
        out.push_str("    svg_show_guards: bool,\n");
    }

    out.push_str("}\n");

    out
}

/// Generate the main function for a CLI entrypoint.
pub fn emit_main_function(info: &EntrypointInfo, config: &CliCodegenConfig) -> String {
    let mut out = String::new();

    out.push_str("fn main() {\n");
    out.push_str("    let cli = Cli::parse();\n");

    // Build DAG call - pass CLI args
    out.push_str(&format!("    let dag = {}(\n", config.dag_builder));
    for arg in &info.cli_args {
        let field_name = arg.name.replace('-', "_");
        // Handle different types appropriately
        if arg.type_id == "String" {
            out.push_str(&format!("        &cli.{},\n", field_name));
        } else {
            out.push_str(&format!("        cli.{},\n", field_name));
        }
    }
    out.push_str("    );\n\n");

    // SVG visualization handling if enabled
    if config.include_viz_flags {
        out.push_str("    if cli.svg_tools {\n");
        out.push_str("        println!(\"{}\", gunbc_ir::viz::tools_to_svg(&dag));\n");
        out.push_str("        return;\n");
        out.push_str("    }\n");
        out.push_str("    if cli.svg {\n");
        out.push_str("        println!(\"{}\", gunbc_ir::viz::dag_to_svg(&dag, cli.svg_show_guards));\n");
        out.push_str("        return;\n");
        out.push_str("    }\n\n");
    }

    // Print DAG info
    out.push_str("    eprintln!(\"DAG constructed ({} nodes, {} edges)\", dag.nodes.len(), dag.edges.len());\n\n");

    // Execute the DAG
    out.push_str("    match gunbc_exec::execute(&dag) {\n");
    out.push_str("        Ok(log) => {\n");
    out.push_str("            eprintln!(\"\\nExecution log:\");\n");
    out.push_str("            eprint!(\"{log}\");\n");
    out.push_str("        }\n");
    out.push_str("        Err(e) => {\n");
    out.push_str("            eprintln!(\"Execution failed: {e}\");\n");
    out.push_str("            std::process::exit(1);\n");
    out.push_str("        }\n");
    out.push_str("    }\n");

    out.push_str("}\n");

    out
}

/// Generate a complete main.rs file for a CLI entrypoint.
pub fn emit_cli_main(info: &EntrypointInfo, config: &CliCodegenConfig) -> String {
    let mut out = String::new();

    // Module documentation
    out.push_str(&format!(
        "//! CLI entrypoint for {}.\n//!\n//! GENERATED - do not edit manually.\n\n",
        config.binary_name
    ));

    // Imports
    out.push_str(&emit_cli_struct(info, config));
    out.push('\n');
    out.push_str(&emit_main_function(info, config));

    out
}

/// Derive execution mode flags from boundary declarations in a DAG.
///
/// This function scans all boundaries (including nested SubDAGs) and generates
/// CLI flag specs for mocking at each transport layer:
///
/// - `--dry-run` - Mock all external boundaries (if any boundaries exist)
/// - `--mock-{layer}` - Mock at a specific transport layer (e.g., --mock-gist, --mock-http)
///
/// # Example
///
/// For a DAG with `External::GitHub::Gist` and `External::HTTP::Request` boundaries:
///
/// ```ignore
/// $ tool --help
/// Options:
///   --dry-run     Mock all external boundaries
///   --mock-github Mock at GitHub layer (fake gist URL, no network)
///   --mock-http   Mock at HTTP layer (fake response, real parsing)
/// ```
pub fn derive_execution_mode_flags<T>(dag: &Dag<T>) -> Vec<CliArgSpec> {
    let boundaries = find_all_boundaries_recursive(dag);
    let mut flags = Vec::new();

    if boundaries.is_empty() {
        return flags;
    }

    // Add --dry-run as alias for "mock everything"
    flags.push(CliArgSpec {
        name: "dry_run".into(),
        type_id: "Bool".into(),
        required: false,
        default: None,
        help: Some("Mock all external boundaries (no network/filesystem calls)".into()),
    });

    // Add per-layer mock flags derived from transport stack
    let mut seen_layers = std::collections::HashSet::new();
    for boundary in &boundaries {
        if let Some(layer) = extract_layer_name(&boundary.external_type) {
            if seen_layers.insert(layer.clone()) {
                flags.push(CliArgSpec {
                    name: format!("mock_{}", layer),
                    type_id: "Bool".into(),
                    required: false,
                    default: None,
                    help: Some(format!(
                        "Mock at {} layer (swap {} SubDAG for mock)",
                        layer, layer
                    )),
                });
            }
        }
    }

    flags
}

/// Generate a complete main.rs with explicit arguments (not from EntrypointInfo).
///
/// This is useful when the CLI interface is defined manually rather than
/// derived from DAG source nodes.
pub fn emit_cli_main_explicit(args: &[CliArgSpec], config: &CliCodegenConfig) -> String {
    let info = EntrypointInfo {
        kind: crate::emit_entrypoint::EntrypointKind::Cli,
        source_nodes: vec![],
        sink_nodes: vec![],
        cli_args: args.to_vec(),
    };
    emit_cli_main(&info, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> CliCodegenConfig {
        CliCodegenConfig {
            binary_name: "gistgen".into(),
            description: "Generate a GitHub Gist from repository files".into(),
            core_crate: "gunbc_gistgen".into(),
            dag_builder: "gunbc_gistgen::build_gistgen_dag".into(),
            op_type: "gunbc_gistgen::GistgenOp".into(),
            include_viz_flags: true,
        }
    }

    fn sample_args() -> Vec<CliArgSpec> {
        vec![
            CliArgSpec {
                name: "path".into(),
                type_id: "String".into(),
                required: false,
                default: Some(".".into()),
                help: Some("Path to the repository root".into()),
            },
            CliArgSpec {
                name: "glob".into(),
                type_id: "String".into(),
                required: false,
                default: Some("**/*".into()),
                help: Some("Glob pattern for file selection".into()),
            },
            CliArgSpec {
                name: "dry_run".into(),
                type_id: "Bool".into(),
                required: false,
                default: None,
                help: Some("Print what would be uploaded without actually creating a gist".into()),
            },
        ]
    }

    #[test]
    fn test_emit_cli_struct() {
        let info = EntrypointInfo {
            kind: crate::emit_entrypoint::EntrypointKind::Cli,
            source_nodes: vec![],
            sink_nodes: vec![],
            cli_args: sample_args(),
        };
        let config = sample_config();
        let code = emit_cli_struct(&info, &config);

        assert!(code.contains("struct Cli"));
        assert!(code.contains("path: String"));
        assert!(code.contains("glob: String"));
        assert!(code.contains("dry_run: bool"));
        assert!(code.contains("#[command(name = \"gistgen\""));
    }

    #[test]
    fn test_emit_main_function() {
        let info = EntrypointInfo {
            kind: crate::emit_entrypoint::EntrypointKind::Cli,
            source_nodes: vec![],
            sink_nodes: vec![],
            cli_args: sample_args(),
        };
        let config = sample_config();
        let code = emit_main_function(&info, &config);

        assert!(code.contains("fn main()"));
        assert!(code.contains("Cli::parse()"));
        assert!(code.contains("gunbc_gistgen::build_gistgen_dag"));
        assert!(code.contains("gunbc_exec::execute"));
    }

    #[test]
    fn test_emit_cli_main_explicit() {
        let args = sample_args();
        let config = sample_config();
        let code = emit_cli_main_explicit(&args, &config);

        assert!(code.contains("//! CLI entrypoint for gistgen"));
        assert!(code.contains("struct Cli"));
        assert!(code.contains("fn main()"));
    }

    #[test]
    fn test_type_id_mapping() {
        assert_eq!(type_id_to_rust_type("String"), "String");
        assert_eq!(type_id_to_rust_type("Bool"), "bool");
        assert_eq!(type_id_to_rust_type("Int"), "i64");
        assert_eq!(type_id_to_rust_type("StrList"), "Vec<String>");
        assert_eq!(type_id_to_rust_type("Unknown"), "String");
    }

    #[test]
    fn test_derive_execution_mode_flags_empty() {
        use gunbc_ir::{Dag, DagMetadata};

        #[derive(Debug, Clone)]
        struct DummyOp;

        let dag: Dag<DummyOp> = Dag {
            nodes: vec![],
            edges: vec![],
            metadata: DagMetadata::default(),
        };

        let flags = derive_execution_mode_flags(&dag);
        assert!(flags.is_empty(), "no boundaries → no flags");
    }

    #[test]
    fn test_derive_execution_mode_flags_with_boundaries() {
        use gunbc_ir::{port, BoundaryDeclaration, Dag, DagMetadata, Node, NodeBody, NodeId, PortName, TypeId};

        #[derive(Debug, Clone)]
        struct DummyOp;

        let dag: Dag<DummyOp> = Dag {
            nodes: vec![
                Node {
                    id: NodeId("upload".into()),
                    inputs: vec![],
                    outputs: vec![port("url", "String")],
                    body: NodeBody::Opaque(DummyOp),
                },
            ],
            edges: vec![],
            metadata: DagMetadata {
                boundary_declarations: vec![
                    BoundaryDeclaration {
                        node: NodeId("upload".into()),
                        port: PortName("url".into()),
                        external_type: TypeId("External::GitHub::Gist".into()),
                    },
                ],
                ..Default::default()
            },
        };

        let flags = derive_execution_mode_flags(&dag);
        assert_eq!(flags.len(), 2);

        // First flag should be --dry-run
        assert_eq!(flags[0].name, "dry_run");
        assert_eq!(flags[0].type_id, "Bool");

        // Second flag should be --mock-github
        assert_eq!(flags[1].name, "mock_github");
        assert_eq!(flags[1].type_id, "Bool");
    }

    #[test]
    fn test_derive_execution_mode_flags_multiple_layers() {
        use gunbc_ir::{port, BoundaryDeclaration, Dag, DagMetadata, Node, NodeBody, NodeId, PortName, TypeId};

        #[derive(Debug, Clone)]
        struct DummyOp;

        let dag: Dag<DummyOp> = Dag {
            nodes: vec![
                Node {
                    id: NodeId("op1".into()),
                    inputs: vec![],
                    outputs: vec![port("out", "String")],
                    body: NodeBody::Opaque(DummyOp),
                },
            ],
            edges: vec![],
            metadata: DagMetadata {
                boundary_declarations: vec![
                    BoundaryDeclaration {
                        node: NodeId("op1".into()),
                        port: PortName("out".into()),
                        external_type: TypeId("External::GitHub::Gist".into()),
                    },
                    BoundaryDeclaration {
                        node: NodeId("op1".into()),
                        port: PortName("conn".into()),
                        external_type: TypeId("External::HTTP::Request".into()),
                    },
                    BoundaryDeclaration {
                        node: NodeId("op1".into()),
                        port: PortName("rest".into()),
                        external_type: TypeId("External::REST::Request".into()),
                    },
                ],
                ..Default::default()
            },
        };

        let flags = derive_execution_mode_flags(&dag);
        // Should have: dry_run, mock_github, mock_http, mock_rest
        assert_eq!(flags.len(), 4);
        assert_eq!(flags[0].name, "dry_run");

        let flag_names: Vec<&str> = flags.iter().map(|f| f.name.as_str()).collect();
        assert!(flag_names.contains(&"mock_github"));
        assert!(flag_names.contains(&"mock_http"));
        assert!(flag_names.contains(&"mock_rest"));
    }
}
