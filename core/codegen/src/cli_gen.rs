//! CLI generation from DAG entrypoints.
//!
//! Generates a `main.rs` file that:
//! - Has CLI flags for each entrypoint port
//! - Has --dry-run flag (automatic from boundary detection)
//! - Executes the DAG with mode selection
//! - Formats output based on execution log
//!
//! Uses the language module for Rust type mappings and naming conventions.

use gunbc_ir::language::{rust_type as lang_rust_type, NamingCase};
use gunbc_ir::Cardinality;

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
    /// Output port to check for success (e.g., "overall_success" for CI).
    /// If this port is false, the CLI exits with code 1.
    pub success_port: Option<String>,
    /// Enable step mode - generates `step <node>` subcommand for CI providers.
    /// This allows executing individual DAG nodes for better CI visibility.
    pub enable_step_mode: bool,
}

/// An entrypoint that becomes a CLI flag.
#[derive(Debug, Clone)]
pub struct CliEntrypoint {
    /// The port name (becomes --port-name flag)
    pub port_name: String,
    /// The type (String, Int, Bool, List, etc.)
    pub type_id: String,
    /// Cardinality of this entrypoint's port.
    ///
    /// Used to determine CLI behavior: `allows_many()` → repeatable flag,
    /// `allows_empty()` → optional argument. This replaces string matching
    /// on `type_id == "List"` with proper cardinality queries.
    pub cardinality: Cardinality,
    /// Short flag (e.g., "-r" for repo_path)
    pub short_flag: Option<char>,
    /// Default value if not provided
    pub default_value: Option<String>,
    /// Help text
    pub help: String,
    /// Make variable name (e.g., "REPO" for repo_path).
    /// When set, this entrypoint is exposed as a Make variable in the generated
    /// Makefile. Entrypoints without make_var are CLI-only (not in Makefile).
    pub make_var: Option<String>,
}

impl CliEntrypoint {
    /// Create a new CLI entrypoint.
    ///
    /// Cardinality defaults to `ONE` (scalar). Use `with_cardinality()` to
    /// set it explicitly, or use `list()` / `set()` constructors for
    /// collection entrypoints.
    pub fn new(port_name: impl Into<String>, type_id: impl Into<String>) -> Self {
        let port = port_name.into();
        let type_id_str = type_id.into();
        let help = format!("Value for {} port", port);
        // Infer cardinality from type_id for backward compatibility.
        // New code should use with_cardinality() or the typed constructors.
        let cardinality = match type_id_str.as_str() {
            "List" | "Set" => Cardinality::ZERO_OR_MORE,
            _ => Cardinality::ONE,
        };
        Self {
            port_name: port,
            type_id: type_id_str,
            cardinality,
            short_flag: None,
            default_value: None,
            help,
            make_var: None,
        }
    }

    /// Set the cardinality explicitly.
    pub fn with_cardinality(mut self, cardinality: Cardinality) -> Self {
        self.cardinality = cardinality;
        self
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

    /// Set Make variable name for Makefile generation.
    ///
    /// When set, this entrypoint is exposed as a Make variable in the generated
    /// Makefile (e.g., `make gist REPO=.`). Entrypoints without a make_var
    /// are CLI-only and won't appear as Makefile variables.
    pub fn make_var(mut self, var: impl Into<String>) -> Self {
        self.make_var = Some(var.into());
        self
    }

    /// Convert port name to CLI flag name (snake_case to kebab-case).
    ///
    /// Uses the language module's `NamingCase::KebabCase` for consistent conversion.
    pub fn flag_name(&self) -> String {
        NamingCase::KebabCase.apply(&self.port_name)
    }

    /// Convert port name to Rust variable name.
    pub fn var_name(&self) -> String {
        self.port_name.clone()
    }

    /// Get the Rust type for this entrypoint.
    ///
    /// Uses the language module's type system mapping for standard types.
    /// Collection types are derived from cardinality, not type_id string matching.
    pub fn rust_type(&self) -> String {
        if self.cardinality.allows_many() {
            // Collection entrypoint — element type is type_id
            let element = match self.type_id.as_str() {
                "List" | "Set" => "String", // Legacy: "List"/"Set" as type_id means list-of-strings
                other => other,
            };
            lang_rust_type(&format!("List<{}>", element))
        } else {
            lang_rust_type(&self.type_id)
        }
    }

    /// Get the Value constructor for this type.
    ///
    /// Collection types are derived from cardinality, not type_id string matching.
    pub fn value_constructor(&self) -> &str {
        if self.cardinality.allows_many() {
            "Value::str_list"
        } else {
            match self.type_id.as_str() {
                "String" => "Value::Str",
                "Int" => "Value::Int",
                "Bool" => "Value::Bool",
                _ => "Value::Str",
            }
        }
    }

    /// Whether this entrypoint accepts multiple values (repeatable CLI flag).
    ///
    /// Derived from cardinality, not type_id string matching.
    pub fn is_repeatable(&self) -> bool {
        self.cardinality.allows_many()
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
    // If step mode is enabled, use the step-mode template
    if tool.enable_step_mode {
        return generate_cli_with_step_mode(tool, entrypoints, boundaries, custom_import);
    }
    
    // Convert crate name (kebab-case) to module name (snake_case)
    let crate_module = NamingCase::SnakeCase.apply(&tool.crate_name);
    let arg_parsing = generate_arg_parsing(entrypoints);
    let mock_setup = generate_mock_setup(boundaries);
    let print_inputs = generate_print_inputs(entrypoints);
    let final_output = generate_final_output(boundaries);
    let help_options = generate_help_options(entrypoints);
    let success_check = generate_success_check(&tool.success_port);
    
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
{success_check}
        }}
        Err(e) => {{
            eprintln!("Error: {{}}", e);
            process::exit(1);
        }}
    }}
}}

fn print_value(port: &str, value: &Value) {{
    match value {{
        Value::Str(s) => {{
            // For stderr/stdout, print full output (important for debugging CI failures)
            if port.ends_with("stderr") || port.ends_with("stdout") {{
                if !s.is_empty() {{
                    println!("  {{}}: {{}}", port, s);
                }}
            }} else if s.len() < 80 {{
                println!("  {{}}: {{}}", port, s);
            }} else {{
                println!("  {{}}: {{}}...", port, &s[..60.min(s.len())]);
            }}
        }}
        Value::Int(i) => println!("  {{}}: {{}}", port, i),
        Value::Bool(b) => println!("  {{}}: {{}}", port, b),
        Value::List(list) => println!("  {{}}: [{{}} items]", port, list.len()),
        Value::Map(map) => println!("  {{}}: {{{{{{}} entries}}}}", port, map.len()),
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
        success_check = success_check,
        description = tool.description,
        help_options = help_options,
    )
}

fn generate_arg_parsing(entrypoints: &[CliEntrypoint]) -> String {
    let mut code = String::new();

    // Declare variables with defaults
    for ep in entrypoints {
        let default = ep.default_value.as_deref().unwrap_or_default();
        if ep.is_repeatable() {
            // Collection entrypoints: Vec<String>
            code.push_str(&format!("    let mut {}: Vec<String> = vec![];\n", ep.var_name()));
        } else {
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
                _ => {
                    code.push_str(&format!("    let mut {} = \"{}\".to_string();\n", ep.var_name(), default));
                }
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

        if ep.is_repeatable() {
            // Repeatable flags: --flag val --flag val2
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
        } else {
            match ep.type_id.as_str() {
                "Bool" => {
                    code.push_str(&format!(
                        "            {}\"--{}\" => {} = true,\n",
                        short, flag, ep.var_name()
                    ));
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
        if ep.is_repeatable() {
            code.push_str(&format!(
                "    println!(\"  {}: {{:?}}\", {});\n",
                ep.port_name, ep.var_name()
            ));
        } else {
            match ep.type_id.as_str() {
                "Bool" => {
                    code.push_str(&format!(
                        "    println!(\"  {}: {{}}\", {});\n",
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
    }
    code
}

fn generate_final_output(_boundaries: &[CliBoundary]) -> String {
    // Generic final output - can be customized per tool
    String::new()
}

/// Generate code to check a success port and exit with code 1 if false.
fn generate_success_check(success_port: &Option<String>) -> String {
    match success_port {
        Some(port) => format!(
            r#"
            // Check success port and exit with appropriate code
            for entry in &log.entries {{
                if let Some(Value::Bool(false)) = entry.outputs.get("{}") {{
                    process::exit(1);
                }}
            }}"#,
            port
        ),
        None => String::new(),
    }
}

fn generate_help_options(entrypoints: &[CliEntrypoint]) -> String {
    let mut code = String::new();
    for ep in entrypoints {
        let flag = ep.flag_name();
        let short = ep.short_flag.map(|c| format!("-{}, ", c)).unwrap_or_else(|| "    ".to_string());
        let type_hint = if ep.is_repeatable() {
            " <VAL>..."
        } else {
            match ep.type_id.as_str() {
                "Bool" => "",
                "Int" => " <NUM>",
                _ => " <VAL>",
            }
        };
        code.push_str(&format!(
            "    println!(\"    {}--{}{:width$}  {}\");\n",
            short, flag, type_hint, ep.help,
            width = 20 - flag.len()
        ));
    }
    code
}

// ============================================================================
// Step Mode CLI Generation
// ============================================================================

/// Generate a CLI with step mode support for CI tools.
///
/// This generates a CLI that supports:
/// - `run` (or no subcommand): Execute the full DAG
/// - `step <node>`: Execute a single node (for CI step visibility)
/// - `list-steps`: List all available steps in topological order
fn generate_cli_with_step_mode(
    tool: &ToolMeta,
    _entrypoints: &[CliEntrypoint],
    boundaries: &[CliBoundary],
    custom_import: Option<&str>,
) -> String {
    // Convert crate name (kebab-case) to module name (snake_case)
    let crate_module = NamingCase::SnakeCase.apply(&tool.crate_name);
    let mock_setup = generate_mock_setup(boundaries);
    let success_check = generate_success_check(&tool.success_port);
    
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
        r#"//! Generated CLI for {tool_name} with step mode support.
//!
//! This file is generated by gunbc-codegen. Do not edit manually.
//! Regenerate with: make codegen
//!
//! Subcommands:
//! - run (default): Execute the full DAG
//! - step <node>: Execute a single node
//! - list-steps: List all available steps

use gunbc_exec::{{execute_with_mode, execute_single_node, BoundaryMocks, ExecutionMode}};
use gunbc_ir::Value;
{import_line}
use std::env;
use std::process;
use std::collections::HashMap;

fn main() {{
    let args: Vec<String> = env::args().collect();
    
    // Parse subcommand
    let subcommand = args.get(1).map(|s| s.as_str());
    
    match subcommand {{
        Some("run") => run_full_dag(&args[2..]),
        Some("step") => run_single_step(&args[2..]),
        Some("list-steps") => list_dag_steps(),
        Some("-h") | Some("--help") | Some("help") => print_help(),
        Some("-n") | Some("--dry-run") => run_full_dag(&args[1..]),  // backwards compat
        Some(arg) if arg.starts_with('-') => run_full_dag(&args[1..]),  // flags go to run
        None => run_full_dag(&[]),
        _ => run_full_dag(&args[1..]),  // unknown subcommand, try as args
    }}
}}

fn run_full_dag(args: &[String]) {{
    let mut dry_run = false;
    
    for arg in args {{
        match arg.as_str() {{
            "-n" | "--dry-run" => dry_run = true,
            "-h" | "--help" => {{ print_help(); return; }}
            _ => {{}}
        }}
    }}
    
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
{success_check}
        }}
        Err(e) => {{
            eprintln!("Error: {{}}", e);
            process::exit(1);
        }}
    }}
}}

fn run_single_step(args: &[String]) {{
    let step_name = match args.first() {{
        Some(name) => name.clone(),
        None => {{
            eprintln!("Error: step name required");
            eprintln!("Usage: {tool_name} step <node_name>");
            eprintln!("Run '{tool_name} list-steps' to see available steps");
            process::exit(1);
        }}
    }};
    
    let mut dry_run = false;
    for arg in args.iter().skip(1) {{
        match arg.as_str() {{
            "-n" | "--dry-run" => dry_run = true,
            _ => {{}}
        }}
    }}
    
    // Build the graph
    let dag = {graph_builder_call};
    
    // DI: capture env vars once at the boundary, pass dict to helpers.
    // Phase 2 will acquire env dict through DAG input ports.
    let env_dict: HashMap<String, String> = env::vars().collect();

    // Load inputs from environment (CI step outputs from previous steps)
    let inputs = load_step_inputs_from_env(&step_name, &env_dict);
    
    // Set up execution mode
    let mode = if dry_run {{
        let mut mocks = BoundaryMocks::new();
{mock_setup}
        ExecutionMode::DryRun(mocks)
    }} else {{
        ExecutionMode::Real
    }};
    
    println!("[{tool_name}:step:{{}}]", step_name);
    
    // Execute single node
    match execute_single_node(&dag, &step_name, inputs, mode) {{
        Ok(outputs) => {{
            // Print outputs
            for (port, value) in &outputs {{
                print_value(port, value);
            }}
            
            // Emit outputs for next steps (CI provider format)
            emit_step_outputs(&step_name, &outputs, &env_dict);
            
            // Check for failure
            if let Some(Value::Bool(false)) = outputs.get("{success_port_or_empty}") {{
                process::exit(1);
            }}
        }}
        Err(e) => {{
            eprintln!("Error executing step '{{}}': {{}}", step_name, e);
            process::exit(1);
        }}
    }}
}}

fn list_dag_steps() {{
    let dag = {graph_builder_call};
    
    println!("Available steps for {tool_name}:");
    println!();
    
    // Get nodes in topological order
    for node in &dag.nodes {{
        let inputs: Vec<_> = node.inputs.iter().map(|p| p.name.0.as_str()).collect();
        let outputs: Vec<_> = node.outputs.iter().map(|p| p.name.0.as_str()).collect();
        
        println!("  {{}}", node.id.0);
        if !inputs.is_empty() {{
            println!("    inputs:  {{}}", inputs.join(", "));
        }}
        if !outputs.is_empty() {{
            println!("    outputs: {{}}", outputs.join(", "));
        }}
    }}
}}

/// Load inputs from environment variables set by previous CI steps.
///
/// Convention: STEP_<NODE>_<PORT> = value
fn load_step_inputs_from_env(step_name: &str, env_dict: &HashMap<String, String>) -> HashMap<String, Value> {{
    let mut inputs = HashMap::new();

    // Look for environment variables matching our convention
    for (key, value) in env_dict {{
        if key.starts_with("STEP_") && key != format!("STEP_{{}}_", step_name.to_uppercase()) {{
            // Parse: STEP_NODENAME_PORTNAME
            let parts: Vec<&str> = key.splitn(3, '_').collect();
            if parts.len() >= 3 {{
                let port_name = parts[2].to_lowercase();
                // Try to parse as appropriate type
                if let Ok(b) = value.parse::<bool>() {{
                    inputs.insert(port_name, Value::Bool(b));
                }} else if let Ok(i) = value.parse::<i64>() {{
                    inputs.insert(port_name, Value::Int(i));
                }} else {{
                    inputs.insert(port_name, Value::Str(value.clone()));
                }}
            }}
        }}
    }}

    inputs
}}

/// Emit outputs in CI provider format for next steps.
fn emit_step_outputs(step_name: &str, outputs: &HashMap<String, Value>, env_dict: &HashMap<String, String>) {{
    // Check if we're in GitHub Actions
    if let Some(output_file) = env_dict.get("GITHUB_OUTPUT") {{
        // GitHub Actions format: write to $GITHUB_OUTPUT file
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&output_file) 
        {{
            use std::io::Write;
            for (port, value) in outputs {{
                let str_value = match value {{
                    Value::Str(s) => s.clone(),
                    Value::Int(i) => i.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => continue,
                }};
                let _ = writeln!(file, "STEP_{{}}_{{}}={{}}", 
                    step_name.to_uppercase(), port.to_uppercase(), str_value);
            }}
        }}
    }} else if env_dict.contains_key("GITLAB_CI") {{
        // GitLab CI format: export to dotenv artifact
        for (port, value) in outputs {{
            let str_value = match value {{
                Value::Str(s) => s.clone(),
                Value::Int(i) => i.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => continue,
            }};
            println!("STEP_{{}}_{{}}={{}}", 
                step_name.to_uppercase(), port.to_uppercase(), str_value);
        }}
    }}
    // Plain mode: just print (already done above)
}}

fn print_value(port: &str, value: &Value) {{
    match value {{
        Value::Str(s) => {{
            if port.ends_with("stderr") || port.ends_with("stdout") {{
                if !s.is_empty() {{
                    println!("  {{}}: {{}}", port, s);
                }}
            }} else if s.len() < 80 {{
                println!("  {{}}: {{}}", port, s);
            }} else {{
                println!("  {{}}: {{}}...", port, &s[..60.min(s.len())]);
            }}
        }}
        Value::Int(i) => println!("  {{}}: {{}}", port, i),
        Value::Bool(b) => println!("  {{}}: {{}}", port, b),
        Value::List(list) => println!("  {{}}: [{{}} items]", port, list.len()),
        Value::Map(map) => println!("  {{}}: {{{{{{}} entries}}}}", port, map.len()),
        Value::Json(_) => println!("  {{}}: <JSON>", port),
        _ => {{}}
    }}
}}

fn print_help() {{
    println!("{tool_name} - {description}");
    println!();
    println!("USAGE:");
    println!("    {tool_name} [SUBCOMMAND] [OPTIONS]");
    println!();
    println!("SUBCOMMANDS:");
    println!("    run          Execute the full DAG (default)");
    println!("    step <node>  Execute a single DAG node");
    println!("    list-steps   List all available steps");
    println!("    help         Print this help");
    println!();
    println!("OPTIONS:");
    println!("    -n, --dry-run    Don't perform actual I/O");
    println!("    -h, --help       Print this help");
}}
"#,
        tool_name = tool.tool_name,
        import_line = actual_import,
        graph_builder_call = graph_builder_call,
        mock_setup = mock_setup,
        success_check = success_check,
        description = tool.description,
        success_port_or_empty = tool.success_port.as_deref().unwrap_or(""),
    )
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
            success_port: None,
            enable_step_mode: false,
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
