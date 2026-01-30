//! CLI generation from DAG entrypoints.
//!
//! Generates a `main.rs` file that:
//! - Has CLI flags for each entrypoint port
//! - Has --dry-run flag (automatic from boundary detection)
//! - Executes the DAG with mode selection
//! - Formats output based on execution log

/// Metadata about a tool for CLI generation.
#[derive(Debug, Clone)]
pub struct ToolMeta {
    /// Crate name (e.g., "gunbc-gist")
    pub crate_name: String,
    /// Tool name for display (e.g., "gist")
    pub tool_name: String,
    /// Short description
    pub description: String,
    /// The function to call to build the graph (e.g., "build_gist_graph")
    pub graph_builder: String,
    /// Arguments to pass to graph builder (e.g., "extensions.clone(), public")
    pub graph_builder_args: String,
    /// Whether the graph builder returns Result<Dag, BuilderError>
    pub returns_result: bool,
}

/// An entrypoint that becomes a CLI flag.
#[derive(Debug, Clone)]
pub struct CliEntrypoint {
    /// The port name (becomes --port-name flag)
    pub port_name: String,
    /// The type (String, Int, Bool, StrList, etc.)
    pub type_id: String,
    /// Short flag (e.g., "-r" for repo_path)
    pub short_flag: Option<char>,
    /// Default value if not provided
    pub default_value: Option<String>,
    /// Help text
    pub help: String,
}

impl CliEntrypoint {
    /// Create a new CLI entrypoint.
    pub fn new(port_name: impl Into<String>, type_id: impl Into<String>) -> Self {
        let port = port_name.into();
        let help = format!("Value for {} port", port);
        Self {
            port_name: port,
            type_id: type_id.into(),
            short_flag: None,
            default_value: None,
            help,
        }
    }

    /// Set short flag.
    pub fn short(mut self, c: char) -> Self {
        self.short_flag = Some(c);
        self
    }

    /// Set default value.
    pub fn default(mut self, val: impl Into<String>) -> Self {
        self.default_value = Some(val.into());
        self
    }

    /// Set help text.
    pub fn help(mut self, text: impl Into<String>) -> Self {
        self.help = text.into();
        self
    }

    /// Convert port name to CLI flag name (snake_case to kebab-case).
    pub fn flag_name(&self) -> String {
        self.port_name.replace('_', "-")
    }

    /// Convert port name to Rust variable name.
    pub fn var_name(&self) -> String {
        self.port_name.clone()
    }

    /// Get the Rust type for this entrypoint.
    pub fn rust_type(&self) -> &str {
        match self.type_id.as_str() {
            "String" => "String",
            "Int" => "i64",
            "Bool" => "bool",
            "StrList" => "Vec<String>",
            _ => "String", // Default to String
        }
    }

    /// Get the Value constructor for this type.
    pub fn value_constructor(&self) -> &str {
        match self.type_id.as_str() {
            "String" => "Value::Str",
            "Int" => "Value::Int",
            "Bool" => "Value::Bool",
            "StrList" => "Value::StrList",
            _ => "Value::Str",
        }
    }
}

/// A boundary node that gets mocked in dry-run.
#[derive(Debug, Clone)]
pub struct CliBoundary {
    /// Node ID
    pub node_id: String,
    /// Output ports to mock
    pub mock_outputs: Vec<(String, String)>, // (port_name, mock_value_expr)
}

/// Generate a complete main.rs for a tool.
pub fn generate_cli(
    tool: &ToolMeta,
    entrypoints: &[CliEntrypoint],
    boundaries: &[CliBoundary],
) -> String {
    generate_cli_with_import(tool, entrypoints, boundaries, None)
}

/// Generate a complete main.rs for a tool with optional custom import.
pub fn generate_cli_with_import(
    tool: &ToolMeta,
    entrypoints: &[CliEntrypoint],
    boundaries: &[CliBoundary],
    custom_import: Option<&str>,
) -> String {
    let crate_module = tool.crate_name.replace('-', "_");
    let arg_parsing = generate_arg_parsing(entrypoints);
    let mock_setup = generate_mock_setup(boundaries);
    let print_inputs = generate_print_inputs(entrypoints);
    let final_output = generate_final_output(boundaries);
    let help_options = generate_help_options(entrypoints);
    
    let import_line = custom_import.unwrap_or("").to_string();
    let default_import = format!("use {}::build_{}_graph;", crate_module, tool.tool_name);
    let actual_import = if import_line.is_empty() { default_import } else { import_line };

    // Generate the graph builder call - handle Result-returning builders
    let graph_builder_call = if tool.returns_result {
        if tool.graph_builder_args.is_empty() {
            format!(
                r#"match {}() {{
        Ok(d) => d,
        Err(e) => {{
            eprintln!("Error building graph: {{}}", e);
            process::exit(1);
        }}
    }}"#,
                tool.graph_builder
            )
        } else {
            format!(
                r#"match {}({}) {{
        Ok(d) => d,
        Err(e) => {{
            eprintln!("Error building graph: {{}}", e);
            process::exit(1);
        }}
    }}"#,
                tool.graph_builder, tool.graph_builder_args
            )
        }
    } else if tool.graph_builder_args.is_empty() {
        format!("{}()", tool.graph_builder)
    } else {
        format!("{}({})", tool.graph_builder, tool.graph_builder_args)
    };

    format!(
        r#"//! Generated CLI for {tool_name}.
//!
//! This file is generated by gunbc-codegen. Do not edit manually.
//! Regenerate with: make codegen

use gunbc_exec::{{execute_with_mode, BoundaryMocks, ExecutionMode}};
use gunbc_ir::Value;
{import_line}
use std::env;
use std::process;

fn main() {{
    let args: Vec<String> = env::args().collect();
    
    // Parse arguments
{arg_parsing}
    
    // Build the graph
    let dag = {graph_builder_call};
    
    // Set up execution mode
    let mode = if dry_run {{
        let mut mocks = BoundaryMocks::new();
{mock_setup}
        ExecutionMode::DryRun(mocks)
    }} else {{
        ExecutionMode::Real
    }};
    
    // Print header
    println!("{tool_name}");
{print_inputs}
    println!("  mode: {{}}", if dry_run {{ "dry-run" }} else {{ "real" }});
    println!();
    
    // Execute
    match execute_with_mode(&dag, mode) {{
        Ok(log) => {{
            for entry in &log.entries {{
                let marker = if entry.was_intercepted {{ " [DRY-RUN]" }} else {{ "" }};
                println!("[{{}}]{{}}", entry.node_id, marker);
                
                for (port, value) in &entry.outputs {{
                    print_value(port, value);
                }}
            }}
{final_output}
        }}
        Err(e) => {{
            eprintln!("Error: {{}}", e);
            process::exit(1);
        }}
    }}
}}

fn print_value(port: &str, value: &Value) {{
    match value {{
        Value::Str(s) if s.len() < 80 => println!("  {{}}: {{}}", port, s),
        Value::Str(s) => println!("  {{}}: {{}}...", port, &s[..60.min(s.len())]),
        Value::Int(i) => println!("  {{}}: {{}}", port, i),
        Value::Bool(b) => println!("  {{}}: {{}}", port, b),
        Value::StrList(list) => println!("  {{}}: [{{}} items]", port, list.len()),
        Value::MapStrStr(map) => println!("  {{}}: {{{{{{}} entries}}}}", port, map.len()),
        Value::Json(_) => println!("  {{}}: <JSON>", port),
        _ => {{}}
    }}
}}

fn print_help() {{
    println!("{tool_name} - {description}");
    println!();
    println!("USAGE:");
    println!("    {tool_name} [OPTIONS]");
    println!();
    println!("OPTIONS:");
{help_options}
    println!("    -n, --dry-run        Don't perform actual I/O");
    println!("    -h, --help           Print this help");
}}
"#,
        tool_name = tool.tool_name,
        import_line = actual_import,
        arg_parsing = arg_parsing,
        graph_builder_call = graph_builder_call,
        mock_setup = mock_setup,
        print_inputs = print_inputs,
        final_output = final_output,
        description = tool.description,
        help_options = help_options,
    )
}

fn generate_arg_parsing(entrypoints: &[CliEntrypoint]) -> String {
    let mut code = String::new();
    
    // Declare variables with defaults
    for ep in entrypoints {
        let default = ep.default_value.as_deref().unwrap_or_default();
        match ep.type_id.as_str() {
            "String" => {
                if default.is_empty() {
                    code.push_str(&format!("    let mut {}: Option<String> = None;\n", ep.var_name()));
                } else {
                    code.push_str(&format!("    let mut {} = \"{}\".to_string();\n", ep.var_name(), default));
                }
            }
            "Bool" => {
                let default_bool = default == "true";
                code.push_str(&format!("    let mut {} = {};\n", ep.var_name(), default_bool));
            }
            "Int" => {
                let default_int = default.parse::<i64>().unwrap_or(0);
                code.push_str(&format!("    let mut {} = {}i64;\n", ep.var_name(), default_int));
            }
            "StrList" => {
                code.push_str(&format!("    let mut {}: Vec<String> = vec![];\n", ep.var_name()));
            }
            _ => {
                code.push_str(&format!("    let mut {} = \"{}\".to_string();\n", ep.var_name(), default));
            }
        }
    }
    code.push_str("    let mut dry_run = false;\n");
    code.push('\n');
    
    // Parse loop
    code.push_str("    let mut i = 1;\n");
    code.push_str("    while i < args.len() {\n");
    code.push_str("        match args[i].as_str() {\n");
    
    for ep in entrypoints {
        let flag = ep.flag_name();
        let short = ep.short_flag.map(|c| format!("\"-{}\" | ", c)).unwrap_or_default();
        
        match ep.type_id.as_str() {
            "Bool" => {
                code.push_str(&format!(
                    "            {}\"--{}\" => {} = true,\n",
                    short, flag, ep.var_name()
                ));
            }
            "StrList" => {
                code.push_str(&format!(
                    "            {}\"--{}\" => {{\n",
                    short, flag
                ));
                code.push_str("                i += 1;\n");
                code.push_str(&format!(
                    "                if i < args.len() {{ {}.push(args[i].clone()); }}\n",
                    ep.var_name()
                ));
                code.push_str("            }\n");
            }
            _ => {
                code.push_str(&format!(
                    "            {}\"--{}\" => {{\n",
                    short, flag
                ));
                code.push_str("                i += 1;\n");
                if ep.default_value.is_some() || ep.type_id != "String" {
                    code.push_str(&format!(
                        "                if i < args.len() {{ {} = args[i].clone(){}; }}\n",
                        ep.var_name(),
                        if ep.type_id == "Int" { ".parse().unwrap_or(0)" } else { "" }
                    ));
                } else {
                    code.push_str(&format!(
                        "                if i < args.len() {{ {} = Some(args[i].clone()); }}\n",
                        ep.var_name()
                    ));
                }
                code.push_str("            }\n");
            }
        }
    }
    
    code.push_str("            \"-n\" | \"--dry-run\" => dry_run = true,\n");
    code.push_str("            \"-h\" | \"--help\" => { print_help(); return; }\n");
    code.push_str("            _ => {}\n");
    code.push_str("        }\n");
    code.push_str("        i += 1;\n");
    code.push_str("    }\n");
    
    code
}

fn generate_mock_setup(boundaries: &[CliBoundary]) -> String {
    let mut code = String::new();
    for boundary in boundaries {
        for (port, value_expr) in &boundary.mock_outputs {
            code.push_str(&format!(
                "        mocks.set_value(\"{}\", \"{}\", {});\n",
                boundary.node_id, port, value_expr
            ));
        }
    }
    code
}

fn generate_print_inputs(entrypoints: &[CliEntrypoint]) -> String {
    let mut code = String::new();
    for ep in entrypoints {
        match ep.type_id.as_str() {
            "Bool" => {
                code.push_str(&format!(
                    "    println!(\"  {}: {{}}\", {});\n",
                    ep.port_name, ep.var_name()
                ));
            }
            "StrList" => {
                code.push_str(&format!(
                    "    println!(\"  {}: {{:?}}\", {});\n",
                    ep.port_name, ep.var_name()
                ));
            }
            _ => {
                if ep.default_value.is_some() {
                    code.push_str(&format!(
                        "    println!(\"  {}: {{}}\", {});\n",
                        ep.port_name, ep.var_name()
                    ));
                } else {
                    code.push_str(&format!(
                        "    println!(\"  {}: {{}}\", {}.as_deref().unwrap_or(\"<default>\"));\n",
                        ep.port_name, ep.var_name()
                    ));
                }
            }
        }
    }
    code
}

fn generate_final_output(_boundaries: &[CliBoundary]) -> String {
    // Generic final output - can be customized per tool
    String::new()
}

fn generate_help_options(entrypoints: &[CliEntrypoint]) -> String {
    let mut code = String::new();
    for ep in entrypoints {
        let flag = ep.flag_name();
        let short = ep.short_flag.map(|c| format!("-{}, ", c)).unwrap_or_else(|| "    ".to_string());
        let type_hint = match ep.type_id.as_str() {
            "Bool" => "",
            "Int" => " <NUM>",
            "StrList" => " <VAL>...",
            _ => " <VAL>",
        };
        code.push_str(&format!(
            "    println!(\"    {}--{}{:width$}  {}\");\n",
            short, flag, type_hint, ep.help,
            width = 20 - flag.len()
        ));
    }
    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_entrypoint_flag_name() {
        let ep = CliEntrypoint::new("repo_path", "String");
        assert_eq!(ep.flag_name(), "repo-path");
    }

    #[test]
    fn test_generate_simple_cli() {
        let tool = ToolMeta {
            crate_name: "gunbc-gist".to_string(),
            tool_name: "gist".to_string(),
            description: "Create gist from files".to_string(),
            graph_builder: "build_gist_graph".to_string(),
            graph_builder_args: "extensions.clone(), public".to_string(),
            returns_result: false,
        };

        let entrypoints = vec![
            CliEntrypoint::new("repo_path", "String")
                .short('r')
                .default(".")
                .help("Repository path"),
        ];

        let boundaries = vec![CliBoundary {
            node_id: "execute_transport".to_string(),
            mock_outputs: vec![
                ("url".to_string(), "Value::Str(\"<DRY-RUN>\".to_string())".to_string()),
            ],
        }];

        let code = generate_cli(&tool, &entrypoints, &boundaries);
        assert!(code.contains("--repo-path"));
        assert!(code.contains("--dry-run"));
        assert!(code.contains("build_gist_graph"));
    }
}
