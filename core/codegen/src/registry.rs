//! Tool registry for CLI generation.
//!
//! Defines metadata for all tools that need CLI generation.

use crate::cli_gen::{CliBoundary, CliEntrypoint, ToolMeta};

/// A tool that needs CLI generation.
pub struct ToolDef {
    pub meta: ToolMeta,
    pub entrypoints: Vec<CliEntrypoint>,
    pub boundaries: Vec<CliBoundary>,
    /// Custom import line (if different from default pattern)
    pub custom_import: Option<String>,
    /// Output artifacts produced by this tool (for clean/rollback)
    pub outputs: Vec<String>,
}

impl ToolDef {
    pub fn new(
        crate_name: &str,
        tool_name: &str,
        description: &str,
        graph_builder: &str,
        graph_builder_args: &str,
    ) -> Self {
        Self {
            meta: ToolMeta {
                crate_name: crate_name.to_string(),
                tool_name: tool_name.to_string(),
                description: description.to_string(),
                graph_builder: graph_builder.to_string(),
                graph_builder_args: graph_builder_args.to_string(),
            },
            entrypoints: vec![],
            boundaries: vec![],
            custom_import: None,
            outputs: vec![],
        }
    }

    /// Set a custom import line.
    pub fn import(mut self, import_line: &str) -> Self {
        self.custom_import = Some(import_line.to_string());
        self
    }

    /// Add an output artifact (file or directory produced by this tool).
    pub fn output(mut self, path: &str) -> Self {
        self.outputs.push(path.to_string());
        self
    }

    pub fn entrypoint(mut self, ep: CliEntrypoint) -> Self {
        self.entrypoints.push(ep);
        self
    }

    pub fn boundary(mut self, node_id: &str, mock_outputs: Vec<(&str, &str)>) -> Self {
        self.boundaries.push(CliBoundary {
            node_id: node_id.to_string(),
            mock_outputs: mock_outputs
                .into_iter()
                .map(|(p, v)| (p.to_string(), v.to_string()))
                .collect(),
        });
        self
    }
}

/// Get all tool definitions for CLI generation.
pub fn all_tools() -> Vec<ToolDef> {
    vec![
        // gunbc-gist
        ToolDef::new(
            "gunbc-gist",
            "gist",
            "Create a GitHub gist from code files",
            "build_gist_graph",
            "extensions.clone(), public",
        )
        .entrypoint(
            CliEntrypoint::new("repo_path", "String")
                .short('r')
                .default(".")
                .help("Repository path to scan"),
        )
        .entrypoint(
            CliEntrypoint::new("extensions", "StrList")
                .short('e')
                .help("File extensions to include (can be repeated)"),
        )
        .entrypoint(
            CliEntrypoint::new("public", "Bool")
                .short('p')
                .help("Make gist public"),
        )
        .boundary(
            "execute_transport",
            vec![
                ("url", "Value::Str(\"<DRY-RUN: gist URL>\".to_string())"),
                ("response", "Value::Response(gunbc_ir::transport::TransportResponse::Shell(gunbc_ir::transport::ShellResponse { exit_code: 0, stdout: String::new(), stderr: String::new() }))"),
            ],
        ), // gist creates a remote gist, no local output

        // gunbc-buck2
        ToolDef::new(
            "gunbc-buck2",
            "buck2",
            "Generate BUCK file from Cargo.toml",
            "build_buck2_graph",
            "",
        )
        .entrypoint(
            CliEntrypoint::new("cargo_toml_path", "String")
                .short('i')
                .default("Cargo.toml")
                .help("Path to Cargo.toml"),
        )
        .entrypoint(
            CliEntrypoint::new("output_path", "String")
                .short('o')
                .default("BUCK")
                .help("Output BUCK file path"),
        )
        .output("BUCK")  // default output
        .boundary(
            "execute_transport",
            vec![
                ("written_path", "Value::Str(\"<DRY-RUN>\".to_string())"),
                ("content", "Value::Str(\"<DRY-RUN>\".to_string())"),
                ("response", "Value::Response(gunbc_ir::transport::TransportResponse::File(gunbc_ir::transport::FileResponse::written(\"BUCK\")))"),
            ],
        ),

        // gunbc-viz
        ToolDef::new(
            "gunbc-viz",
            "viz",
            "Generate DAG visualization data",
            "build_viz_graph",
            "",
        )
        .entrypoint(
            CliEntrypoint::new("output_path", "String")
                .short('o')
                .default("viz-data.json")
                .help("Output JSON file path"),
        )
        .output("viz-data.json")  // default output
        .boundary(
            "execute_transport",
            vec![
                ("written_path", "Value::Str(\"<DRY-RUN>\".to_string())"),
                ("response", "Value::Response(gunbc_ir::transport::TransportResponse::File(gunbc_ir::transport::FileResponse::written(\"viz-data.json\")))"),
            ],
        ),

        // gunbc-makegen
        ToolDef::new(
            "gunbc-makegen",
            "makegen",
            "Generate Makefile from tool registry",
            "build_makegen_graph",
            "",
        )
        .import("use gunbc_makegen::build_makegen_graph;")
        .entrypoint(
            CliEntrypoint::new("output_path", "String")
                .short('o')
                .default("Makefile")
                .help("Output Makefile path"),
        )
        .boundary(
            "write_makefile",
            vec![
                ("written_path", "Value::Str(\"<DRY-RUN>\".to_string())"),
                ("content", "Value::Str(\"<DRY-RUN>\".to_string())"),
                ("changed", "Value::Bool(true)"),
            ],
        ),

        // gunbc-deps
        ToolDef::new(
            "gunbc-deps",
            "deps",
            "Install tool dependencies",
            "build_deps_graph",
            "",
        )
        .import("use gunbc_deps::build_deps_graph;")
        .entrypoint(
            CliEntrypoint::new("manifest_path", "String")
                .short('m')
                .default("deps.toml")
                .help("Path to deps.toml manifest"),
        )
        .boundary(
            "install",
            vec![
                ("installed", "Value::Bool(true)"),
            ],
        ),

        // gunbc-ci
        ToolDef::new(
            "gunbc-ci",
            "ci",
            "Run CI pipeline",
            "build_ci_graph",
            "",
        )
        .import("use gunbc_ci::build_ci_graph;")
        .boundary(
            "run_tests",
            vec![
                ("passed", "Value::Bool(true)"),
            ],
        ),

        // gunbc-bootstrap
        ToolDef::new(
            "gunbc-bootstrap",
            "bootstrap",
            "Generate Makefile and .gitignore",
            "build_bootstrap_graph",
            "",
        )
        .import("use gunbc_bootstrap::build_bootstrap_graph;")
        .boundary(
            "write_makefile",
            vec![
                ("written_path", "Value::Str(\"<DRY-RUN>\".to_string())"),
            ],
        )
        .boundary(
            "write_gitignore",
            vec![
                ("written_path", "Value::Str(\"<DRY-RUN>\".to_string())"),
            ],
        ),
    ]
}

/// Core build system artifacts (not tool-specific).
pub fn core_outputs() -> Vec<&'static str> {
    vec![
        "buck-out/",      // codegen output directory
        "target/",        // cargo build output
        "bin",            // symlink to target/release
    ]
}

/// Get all cleanable artifacts from tools and core.
pub fn all_cleanable_outputs() -> Vec<String> {
    let mut outputs: Vec<String> = core_outputs()
        .into_iter()
        .map(|s| s.to_string())
        .collect();
    
    for tool in all_tools() {
        outputs.extend(tool.outputs);
    }
    
    // Deduplicate
    outputs.sort();
    outputs.dedup();
    outputs
}
