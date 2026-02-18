//! CLI generation from DAG entrypoints.
//!
//! Generates a `main.rs` file that:
//! - Has CLI flags for each entrypoint port
//! - Has --dry-run flag (automatic from boundary detection)
//! - Executes the DAG with mode selection
//! - Formats output based on execution log
//!
//! Uses the language module for Rust type mappings and naming conventions.
//!
//! # IR Strategy
//!
//! The generated source file uses proper `Item::Use(Import)` for imports
//! and `Item::Fn(FnDef)` for function definitions. Complex function bodies
//! use `Expr::RawCode` as an escape hatch where full IR decomposition would
//! be fragile (e.g., arg-parsing while loops, format strings with escaped
//! braces). This gives structural decomposition at the file level while
//! allowing incremental IR deepening later.

use crate::testgen::render_rust::plain_rust_renderer;
use gunbc_cli::ParamType;
use gunbc_ir::code_ir::{Expr, FnDef, Import, Item, SourceFile, Stmt};
use gunbc_ir::language::{rust_type as lang_rust_type, NamingCase};
use gunbc_ir::render_ir::CodeRenderer;
use gunbc_ir::Cardinality;
use std::fmt::Write;

/// Metadata about a tool for CLI generation.
#[derive(Debug, Clone)]
pub struct ToolMeta {
    /// Crate name (e.g., "gunbc-gist")
    pub crate_name: String,
    /// Tool name for display (e.g., "gist")
    pub tool_name: String,
    /// Short description
    pub description: String,
    /// The graph builder function name (e.g., "build_gist_graph").
    pub graph_builder_call: String,
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
    /// Rust expression that returns a MockSpec for dry-run boundary mocking.
    /// When set, the generated CLI calls this instead of using inline boundary values.
    /// Example: "gunbc_gist::graph_mock::gist_snapshot_mock_spec()"
    pub mock_spec_call: Option<String>,
}

/// An entrypoint that becomes a CLI flag.
#[derive(Debug, Clone)]
pub struct CliEntrypoint {
    /// The port name (becomes --port-name flag)
    pub port_name: String,
    /// The type (Str, Int, Bool).
    pub type_id: ParamType,
    /// Cardinality of this entrypoint's port.
    ///
    /// Used to determine CLI behavior: `allows_many()` → repeatable flag,
    /// `allows_empty()` → optional argument.
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
    /// set it explicitly for collection entrypoints.
    pub fn new(port_name: impl Into<String>, type_id: ParamType) -> Self {
        let port = port_name.into();
        let help = format!("Value for {} port", port);
        Self {
            port_name: port,
            type_id,
            cardinality: Cardinality::ONE,
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
            let element = self.type_id.as_str();
            lang_rust_type(&format!("List<{}>", element))
        } else {
            lang_rust_type(self.type_id.as_str())
        }
    }

    /// Get the Value constructor for this type.
    ///
    /// Collection types are derived from cardinality, not type_id string matching.
    pub fn value_constructor(&self) -> &str {
        if self.cardinality.allows_many() {
            "Value::str_list"
        } else {
            match self.type_id {
                ParamType::Str => "Value::Str",
                ParamType::Int => "Value::Int",
                ParamType::Bool => "Value::Bool",
            }
        }
    }

    /// Whether this entrypoint accepts multiple values (repeatable CLI flag).
    ///
    /// Derived from cardinality, not type_id string matching.
    pub fn is_repeatable(&self) -> bool {
        self.cardinality.allows_many()
    }

    /// Convert to a `gunbc_cli::CliParam` for in-process parsing.
    pub fn to_cli_param(&self) -> gunbc_cli::CliParam {
        let mut p = gunbc_cli::CliParam::new(&self.port_name, self.type_id)
            .with_cardinality(self.cardinality);
        if let Some(c) = self.short_flag {
            p = p.short(c);
        }
        if let Some(ref d) = self.default_value {
            p = p.default(d);
        }
        p
    }

    /// Parse entrypoints from a JSON string (as stored in `ToolRegistration.entrypoints_json`).
    ///
    /// JSON format: `[{"port_name":"...","type_id":"...","cardinality":"ONE","short":"r","default":".","help":"...","make_var":"REPO"}]`
    /// All fields except `port_name` and `type_id` are optional.
    pub fn from_json(json: &str) -> Vec<Self> {
        if json.is_empty() {
            return Vec::new();
        }
        let entries: Vec<serde_json::Value> = serde_json::from_str(json)
            .unwrap_or_else(|e| panic!("invalid entrypoints JSON: {}: {}", e, json));
        entries
            .iter()
            .map(|entry| {
                let port_name = entry["port_name"]
                    .as_str()
                    .expect("port_name required")
                    .to_string();
                let type_id_raw = entry["type_id"].as_str().expect("type_id required");
                let type_id = ParamType::try_from(type_id_raw).unwrap_or_else(|e| {
                    panic!(
                        "entrypoint '{}' has invalid type_id '{}': {}",
                        port_name, type_id_raw, e
                    )
                });
                let cardinality = match entry.get("cardinality").and_then(|v| v.as_str()) {
                    Some("ZERO_OR_MORE") => Cardinality::ZERO_OR_MORE,
                    Some("ONE_OR_MORE") => Cardinality::ONE_OR_MORE,
                    Some("ZERO_OR_ONE") => Cardinality::ZERO_OR_ONE,
                    _ => Cardinality::ONE,
                };
                let short_flag = entry
                    .get("short")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.chars().next());
                let default_value = entry
                    .get("default")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let help = entry
                    .get("help")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let make_var = entry
                    .get("make_var")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                Self {
                    port_name,
                    type_id,
                    cardinality,
                    short_flag,
                    default_value,
                    help,
                    make_var,
                }
            })
            .collect()
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Generate a complete main.rs for a tool.
pub fn generate_cli(tool: &ToolMeta, entrypoints: &[CliEntrypoint]) -> String {
    generate_cli_with_import(tool, entrypoints, None)
}

/// Generate a complete main.rs for a tool with optional custom import.
pub fn generate_cli_with_import(
    tool: &ToolMeta,
    entrypoints: &[CliEntrypoint],
    custom_import: Option<&str>,
) -> String {
    let file = if tool.enable_step_mode {
        build_step_mode_source_file(tool, entrypoints, custom_import)
    } else {
        build_cli_source_file(tool, entrypoints, custom_import)
    };
    plain_rust_renderer().render_source_file(&file)
}

// ============================================================================
// Import builder
// ============================================================================

/// Build the import items for the generated CLI.
fn build_cli_imports(
    tool: &ToolMeta,
    custom_import: Option<&str>,
    step_mode: bool,
    has_entrypoints: bool,
) -> Vec<Item> {
    let crate_module = NamingCase::SnakeCase.apply(&tool.crate_name);

    // gunbc_exec imports
    let mut exec_items = vec![
        "compose_with_freshness".to_string(),
        "execute_and_display".to_string(),
        "BoundaryMocks".to_string(),
        "ExecutionMode".to_string(),
        "Preamble".to_string(),
        "print_preamble_auto".to_string(),
    ];
    if step_mode {
        exec_items.push("execute_single_node".to_string());
        exec_items.push("print_value".to_string());
        exec_items.push("run_freshness_step".to_string());
    }

    let mut items = vec![
        Item::Use(Import {
            path: vec!["gunbc_exec".to_string()],
            items: exec_items,
        }),
        Item::Use(Import {
            path: vec!["gunbc_ir".to_string()],
            items: {
                let mut ir_items = vec!["detect_entrypoints".to_string()];
                if has_entrypoints {
                    ir_items.push("Value".to_string());
                }
                ir_items
            },
        }),
    ];

    // Freshness policy import (runtime freshness check)
    items.push(Item::Use(Import {
        path: vec!["gunbc_lib_transport".to_string()],
        items: vec!["check_and_plan_freshness".to_string()],
    }));

    // Tool-specific import
    let tool_import = match custom_import {
        Some(line) if !line.is_empty() => line.to_string(),
        _ => format!("use {}::build_{}_graph;", crate_module, tool.tool_name),
    };
    items.push(Item::Raw(tool_import));

    // std imports (HashMap only needed in step mode for env_dict/inputs)
    if step_mode {
        items.push(Item::Use(Import {
            path: vec!["std".to_string(), "collections".to_string()],
            items: vec!["HashMap".to_string()],
        }));
        items.push(Item::Use(Import {
            path: vec!["std".to_string(), "fmt".to_string()],
            items: vec!["Write".to_string()],
        }));
        items.push(Item::Use(Import {
            path: vec!["gunbc_ir".to_string(), "resource".to_string()],
            items: vec!["ResourceIo".to_string()],
        }));
        items.push(Item::Use(Import {
            path: vec!["gunbc_lib_transport".to_string()],
            items: vec!["TransportIo".to_string()],
        }));
    }
    items.push(Item::Use(Import {
        path: vec!["std".to_string(), "env".to_string()],
        items: vec![],
    }));
    items.push(Item::Use(Import {
        path: vec!["std".to_string(), "process".to_string()],
        items: vec![],
    }));

    items
}

// ============================================================================
// Shared helpers (return String fragments for RawCode)
// ============================================================================

/// Generate the graph builder call expression.
fn generate_graph_builder_call(tool: &ToolMeta) -> String {
    let f = &tool.graph_builder_call;
    let args = &tool.graph_builder_args;
    if tool.returns_result {
        let call = if args.is_empty() {
            format!("{}()", f)
        } else {
            format!("{}({})", f, args)
        };
        format!(
            "match {} {{\n    Ok(d) => d,\n    Err(e) => {{\n        eprintln!(\"Error building graph: {{}}\", e);\n        process::exit(1);\n    }}\n}}",
            call
        )
    } else if args.is_empty() {
        format!("{}()", f)
    } else {
        format!("{}({})", f, args)
    }
}

/// Generate arg-parsing code using `gunbc_cli::parse()`.
///
/// Emits a schema definition + parse call, then extracts local variables from
/// `ParseResult.values` so that `graph_builder_args` (which references locals
/// by name) still compiles.
fn generate_arg_parsing(entrypoints: &[CliEntrypoint]) -> String {
    let mut code = String::new();

    // Build schema
    code.push_str("let schema = vec![\n");
    for ep in entrypoints {
        let type_expr = match ep.type_id {
            ParamType::Str => "gunbc_cli::ParamType::Str",
            ParamType::Int => "gunbc_cli::ParamType::Int",
            ParamType::Bool => "gunbc_cli::ParamType::Bool",
        };
        write!(
            code,
            "    gunbc_cli::CliParam::new(\"{}\", {})",
            ep.port_name, type_expr
        )
        .unwrap();
        if ep.is_repeatable() {
            code.push_str(".with_cardinality(gunbc_ir::Cardinality::ZERO_OR_MORE)");
        }
        if let Some(c) = ep.short_flag {
            write!(code, ".short('{}')", c).unwrap();
        }
        if let Some(ref d) = ep.default_value {
            write!(code, ".default(\"{}\")", d).unwrap();
        }
        code.push_str(",\n");
    }
    code.push_str("];\n\n");

    // Parse
    code.push_str("let parsed = gunbc_cli::parse(&args, &schema).unwrap_or_else(|e| {\n");
    code.push_str("    eprintln!(\"{}\", e);\n");
    code.push_str("    process::exit(1);\n");
    code.push_str("});\n\n");

    // Handle help
    code.push_str("if parsed.help {\n    print_help();\n    return;\n}\n\n");

    // Extract dry_run and cli_inputs (take ownership of values map)
    code.push_str("let dry_run = parsed.dry_run;\n");
    code.push_str("let cli_inputs = parsed.values;\n");

    // Extract local variables from cli_inputs for graph_builder_args compatibility
    for ep in entrypoints {
        if ep.is_repeatable() {
            write!(code,
                "let {}: Vec<String> = match cli_inputs.get(\"{}\") {{\n    Some(Value::List(items)) => items.iter().filter_map(|v| match v {{ Value::Str(s) => Some(s.clone()), _ => None }}).collect(),\n    _ => vec![],\n}};\n",
                ep.var_name(), ep.port_name
            ).unwrap();
        } else {
            match ep.type_id {
                ParamType::Bool => writeln!(
                    code,
                    "let {} = matches!(cli_inputs.get(\"{}\"), Some(Value::Bool(true)));",
                    ep.var_name(),
                    ep.port_name
                )
                .unwrap(),
                ParamType::Int => {
                    let default = match ep.default_value.as_deref() {
                        Some(d) => {
                            gunbc_cli::parse_int_flag(&ep.port_name, d).unwrap_or_else(|_| {
                                panic!(
                                    "entrypoint '{}' has invalid default int value: {:?}",
                                    ep.port_name, d
                                )
                            })
                        }
                        None => 0,
                    };
                    writeln!(code,
                        "let {} = match cli_inputs.get(\"{}\") {{ Some(Value::Int(i)) => *i, _ => {} }};",
                        ep.var_name(), ep.port_name, default
                    ).unwrap();
                }
                ParamType::Str => {
                    if ep.default_value.is_some() {
                        let default = ep.default_value.as_deref().unwrap_or("");
                        writeln!(code,
                            "let {} = match cli_inputs.get(\"{}\") {{ Some(Value::Str(s)) => s.clone(), _ => \"{}\".to_string() }};",
                            ep.var_name(), ep.port_name, default
                        ).unwrap();
                    } else {
                        writeln!(code,
                            "let {}: Option<String> = cli_inputs.get(\"{}\").and_then(|v| match v {{ Value::Str(s) => Some(s.clone()), _ => None }});",
                            ep.var_name(), ep.port_name
                        ).unwrap();
                    }
                }
            }
        }
    }

    code
}

/// Generate the mock_spec dry-run setup expression.
fn generate_mock_setup(mock_spec_call: &Option<String>) -> String {
    let call = mock_spec_call
        .as_deref()
        .expect("all tools must have mock_spec_call set");
    format!(
        "let _spec = {};\nExecutionMode::DryRun(_spec.to_dry_run_mocks())",
        call
    )
}

/// Generate a `Vec<String>` expression that collects input args for the preamble body.
fn generate_preamble_body_lines(entrypoints: &[CliEntrypoint]) -> String {
    let mut items = Vec::new();
    for ep in entrypoints {
        if ep.is_repeatable() {
            items.push(format!(
                "format!(\"{}: {{:?}}\", {})",
                ep.port_name,
                ep.var_name()
            ));
        } else {
            match ep.type_id {
                ParamType::Bool => {
                    items.push(format!(
                        "format!(\"{}: {{}}\", {})",
                        ep.port_name,
                        ep.var_name()
                    ));
                }
                _ => {
                    if ep.default_value.is_some() {
                        items.push(format!(
                            "format!(\"{}: {{}}\", {})",
                            ep.port_name,
                            ep.var_name()
                        ));
                    } else {
                        items.push(format!(
                            "format!(\"{}: {{}}\", {}.as_deref().unwrap_or(\"<default>\"))",
                            ep.port_name,
                            ep.var_name()
                        ));
                    }
                }
            }
        }
    }
    if items.is_empty() {
        "Vec::new()".to_string()
    } else {
        format!("vec![{}]", items.join(", "))
    }
}

/// Generate the input_mocks block using `cli_inputs` from parse result.
///
/// Since `generate_arg_parsing()` already stores `parsed.values` as `cli_inputs`,
/// we simply wire those into boundary mocks via entrypoint detection.
fn generate_input_mocks(_entrypoints: &[CliEntrypoint]) -> String {
    let mut code = String::new();

    code.push_str("let entrypoints = detect_entrypoints(&dag);\n");
    code.push_str("let mut input_mocks = BoundaryMocks::new();\n");
    code.push_str("for (node_id, port_name, _) in entrypoints.entrypoint_ports {\n");
    code.push_str("    if let Some(value) = cli_inputs.get(&port_name.0) {\n");
    code.push_str(
        "        input_mocks.set_input(node_id.0.clone(), port_name.0.clone(), value.clone());\n",
    );
    code.push_str("    }\n");
    code.push_str("}\n");

    code
}

/// Generate help option lines.
fn generate_help_options(entrypoints: &[CliEntrypoint]) -> String {
    let mut code = String::new();
    for ep in entrypoints {
        let flag = ep.flag_name();
        let short = ep
            .short_flag
            .map(|c| format!("-{}, ", c))
            .unwrap_or_else(|| "    ".to_string());
        let type_hint = if ep.is_repeatable() {
            " <VAL>..."
        } else {
            match ep.type_id {
                ParamType::Bool => "",
                ParamType::Int => " <NUM>",
                ParamType::Str => " <VAL>",
            }
        };
        writeln!(
            code,
            "println!(\"    {}--{}{:width$}  {}\");",
            short,
            flag,
            type_hint,
            ep.help,
            width = 20 - flag.len()
        )
        .unwrap();
    }
    code
}

/// Generate the dry-run mode block.
fn generate_dry_run_block(tool: &ToolMeta) -> String {
    let mock_setup = generate_mock_setup(&tool.mock_spec_call);
    format!(
        "let mode = if dry_run {{\n    {}\n}} else {{\n    ExecutionMode::Real\n}};",
        mock_setup.replace('\n', "\n    ")
    )
}

/// Generate the success_port argument expression.
fn generate_success_port_arg(tool: &ToolMeta) -> String {
    match &tool.success_port {
        Some(port) => format!("Some(\"{}\")", port),
        None => "None".to_string(),
    }
}

// ============================================================================
// Standard Mode
// ============================================================================

/// Build a `SourceFile` IR for a standard CLI main.rs.
fn build_cli_source_file(
    tool: &ToolMeta,
    entrypoints: &[CliEntrypoint],
    custom_import: Option<&str>,
) -> SourceFile {
    let imports = build_cli_imports(tool, custom_import, false, !entrypoints.is_empty());

    let main_fn = build_main_fn(tool, entrypoints);
    let help_fn = build_help_fn(tool, entrypoints);

    let mut items = imports;
    items.push(Item::Fn(main_fn));
    items.push(Item::Fn(help_fn));

    SourceFile {
        doc: vec![
            format!("Generated CLI for {}.", tool.tool_name),
            String::new(),
            "This file is generated by gunbc-codegen. Do not edit manually.".to_string(),
            "Regenerate with: make codegen".to_string(),
        ],
        items,
    }
}

/// Build the `main()` function for standard mode.
fn build_main_fn(tool: &ToolMeta, entrypoints: &[CliEntrypoint]) -> FnDef {
    let arg_parsing = generate_arg_parsing(entrypoints);
    let graph_builder_call = generate_graph_builder_call(tool);
    let input_mocks = generate_input_mocks(entrypoints);
    let dry_run_block = generate_dry_run_block(tool);
    let body_lines_expr = generate_preamble_body_lines(entrypoints);
    let success_port_arg = generate_success_port_arg(tool);

    let body_code = format!(
        "let args: Vec<String> = env::args().collect();\n\
         \n\
         // Parse arguments\n\
         {arg_parsing}\n\
         // Build the graph and compose with freshness checks\n\
         let dag = {graph_builder_call};\n\
         let steps = check_and_plan_freshness();\n\
         let dag = compose_with_freshness(dag, steps);\n\
         \n\
         {input_mocks}\n\
         // Set up execution mode\n\
         {dry_run_block}\n\
         \n\
         // Build preamble with args inside the box\n\
         let mut body_lines = {body_lines_expr};\n\
         body_lines.push(format!(\"mode: {{}}\", if dry_run {{ \"dry-run\" }} else {{ \"real\" }}));\n\
         let preamble = Preamble::with_body(\"{tool_name}\", \"{tool_description}\", body_lines);\n\
         let animated = print_preamble_auto(&preamble);\n\
         \n\
         // Execute DAG with unified display\n\
         execute_and_display(&dag, mode, animated, {success_port_arg}, Some(&input_mocks));",
        arg_parsing = arg_parsing,
        graph_builder_call = graph_builder_call,
        input_mocks = input_mocks,
        dry_run_block = dry_run_block,
        body_lines_expr = body_lines_expr,
        tool_name = tool.tool_name,
        tool_description = tool.description.replace('"', "\\\""),
        success_port_arg = success_port_arg,
    );

    FnDef {
        name: "main".to_string(),
        is_pub: false,
        params: vec![],
        return_type: None,
        body: vec![Stmt::TailExpr(Expr::RawCode(body_code))],
        doc: vec![],
        attributes: vec![],
    }
}

/// Build the `print_help()` function.
fn build_help_fn(tool: &ToolMeta, entrypoints: &[CliEntrypoint]) -> FnDef {
    let help_options = generate_help_options(entrypoints);

    let body_code = format!(
        "println!(\"{tool_name} - {description}\");\n\
         println!();\n\
         println!(\"USAGE:\");\n\
         println!(\"    {tool_name} [OPTIONS]\");\n\
         println!();\n\
         println!(\"OPTIONS:\");\n\
         {help_options}\
         println!(\"    -n, --dry-run        Don't perform actual I/O\");\n\
         println!(\"    -h, --help           Print this help\");\n\
         println!();\n\
         println!(\"Progress display is automatic based on terminal capabilities.\");",
        tool_name = tool.tool_name,
        description = tool.description,
        help_options = help_options,
    );

    FnDef {
        name: "print_help".to_string(),
        is_pub: false,
        params: vec![],
        return_type: None,
        body: vec![Stmt::TailExpr(Expr::RawCode(body_code))],
        doc: vec![],
        attributes: vec![],
    }
}

// ============================================================================
// Step Mode
// ============================================================================

/// Build a `SourceFile` IR for a step-mode CLI main.rs.
fn build_step_mode_source_file(
    tool: &ToolMeta,
    entrypoints: &[CliEntrypoint],
    custom_import: Option<&str>,
) -> SourceFile {
    let imports = build_cli_imports(tool, custom_import, true, !entrypoints.is_empty());

    let main_fn = build_step_main_fn();
    let run_full_fn = build_run_full_dag_fn(tool, entrypoints);
    let step_fn = build_run_single_step_fn(tool);
    let list_fn = build_list_dag_steps_fn(tool);
    let load_fn = build_load_step_inputs_fn();
    let emit_fn = build_emit_step_outputs_fn();
    let help_fn = build_step_help_fn(tool);

    let mut items = imports;
    items.push(Item::Fn(main_fn));
    items.push(Item::Fn(run_full_fn));
    items.push(Item::Fn(step_fn));
    items.push(Item::Fn(list_fn));
    items.push(Item::Fn(load_fn));
    items.push(Item::Fn(emit_fn));
    items.push(Item::Fn(help_fn));

    SourceFile {
        doc: vec![
            format!(
                "Generated CLI for {} with step mode support.",
                tool.tool_name
            ),
            String::new(),
            "This file is generated by gunbc-codegen. Do not edit manually.".to_string(),
            "Regenerate with: make codegen".to_string(),
            String::new(),
            "Subcommands:".to_string(),
            "- run (default): Execute the full DAG".to_string(),
            "- step <node>: Execute a single node".to_string(),
            "- list-steps: List all available steps".to_string(),
        ],
        items,
    }
}

/// Build the `main()` function for step mode.
fn build_step_main_fn() -> FnDef {
    let body_code = "\
let args: Vec<String> = env::args().collect();\n\
\n\
let parsed = match gunbc_cli::parse_step_mode(&args) {\n\
    Ok(parsed) => parsed,\n\
    Err(e) => {\n\
        eprintln!(\"error: {}\", e);\n\
        print_help();\n\
        process::exit(1);\n\
    }\n\
};\n\
\n\
match parsed.subcommand {\n\
    gunbc_cli::StepModeSubcommand::Run => run_full_dag(&parsed.args),\n\
    gunbc_cli::StepModeSubcommand::Step => run_single_step(&parsed.args),\n\
    gunbc_cli::StepModeSubcommand::ListSteps => list_dag_steps(),\n\
    gunbc_cli::StepModeSubcommand::Help => print_help(),\n\
}\n\
"
    .to_string();

    FnDef {
        name: "main".to_string(),
        is_pub: false,
        params: vec![],
        return_type: None,
        body: vec![Stmt::TailExpr(Expr::RawCode(body_code))],
        doc: vec![],
        attributes: vec![],
    }
}

/// Build the `run_full_dag()` function for step mode.
fn build_run_full_dag_fn(tool: &ToolMeta, entrypoints: &[CliEntrypoint]) -> FnDef {
    let arg_parsing = generate_arg_parsing(entrypoints);
    let graph_builder_call = generate_graph_builder_call(tool);
    let input_mocks = generate_input_mocks(entrypoints);
    let dry_run_block = generate_dry_run_block(tool);
    let body_lines_expr = generate_preamble_body_lines(entrypoints);
    let success_port_arg = generate_success_port_arg(tool);

    let body_code = format!(
        "let mut args: Vec<String> = Vec::new();\n\
         args.push(\"run\".to_string());\n\
         args.extend_from_slice(raw_args);\n\
         \n\
         {arg_parsing}\n\
         // Build the graph and compose with freshness checks\n\
         let dag = {graph_builder_call};\n\
         let steps = check_and_plan_freshness();\n\
         let dag = compose_with_freshness(dag, steps);\n\
         \n\
         {input_mocks}\n\
         // Set up execution mode\n\
         {dry_run_block}\n\
         \n\
         // Build preamble with args inside the box\n\
         let mut body_lines = {body_lines_expr};\n\
         body_lines.push(format!(\"mode: {{}}\", if dry_run {{ \"dry-run\" }} else {{ \"real\" }}));\n\
         let preamble = Preamble::with_body(\"{tool_name}\", \"{tool_description}\", body_lines);\n\
         let animated = print_preamble_auto(&preamble);\n\
         \n\
         // Execute DAG with unified display\n\
         execute_and_display(&dag, mode, animated, {success_port_arg}, Some(&input_mocks));",
        arg_parsing = arg_parsing,
        graph_builder_call = graph_builder_call,
        input_mocks = input_mocks,
        dry_run_block = dry_run_block,
        body_lines_expr = body_lines_expr,
        tool_name = tool.tool_name,
        tool_description = tool.description.replace('"', "\\\""),
        success_port_arg = success_port_arg,
    );

    FnDef {
        name: "run_full_dag".to_string(),
        is_pub: false,
        params: vec![("raw_args".to_string(), "&[String]".to_string())],
        return_type: None,
        body: vec![Stmt::TailExpr(Expr::RawCode(body_code))],
        doc: vec![],
        attributes: vec![],
    }
}

/// Build the `run_single_step()` function for step mode.
fn build_run_single_step_fn(tool: &ToolMeta) -> FnDef {
    let graph_builder_call = generate_graph_builder_call(tool);
    let dry_run_block = generate_dry_run_block(tool);
    let success_port_or_empty = tool.success_port.as_deref().unwrap_or("");

    let body_code = format!(
        "let step_name = match args.first() {{\n\
             Some(name) => name.clone(),\n\
             None => {{\n\
                 eprintln!(\"Error: step name required\");\n\
                 eprintln!(\"Usage: {tool_name} step <node_name>\");\n\
                 eprintln!(\"Run '{tool_name} list-steps' to see available steps\");\n\
                 process::exit(1);\n\
             }}\n\
         }};\n\
         \n\
         let mut dry_run = false;\n\
         for arg in args.iter().skip(1) {{\n\
             match arg.as_str() {{\n\
                 \"-n\" | \"--dry-run\" => dry_run = true,\n\
                 _ => {{}}\n\
             }}\n\
         }}\n\
         \n\
         // Freshness check (auto-fix if stale)\n\
         if let Some(steps) = check_and_plan_freshness() {{\n\
             for step in &steps {{\n\
                 if let Err(e) = run_freshness_step(step) {{\n\
                     eprintln!(\"{{}}\", e);\n\
                     process::exit(1);\n\
                 }}\n\
             }}\n\
         }}\n\
         \n\
         // Build the graph\n\
         let dag = {graph_builder_call};\n\
         \n\
         // Capture environment once at the boundary\n\
         let env_dict: HashMap<String, String> = env::vars().collect();\n\
         \n\
         // Load inputs from environment (CI step outputs from previous steps)\n\
         let inputs = load_step_inputs_from_env(&step_name, &env_dict);\n\
         \n\
         // Set up execution mode\n\
         {dry_run_block}\n\
         \n\
         println!(\"[{tool_name}:step:{{}}]\", step_name);\n\
         \n\
         // Execute single node\n\
         match execute_single_node(&dag, &step_name, inputs, mode) {{\n\
             Ok(outputs) => {{\n\
                 // Print outputs\n\
                 for (port, value) in &outputs {{\n\
                     print_value(port, value);\n\
                 }}\n\
                 \n\
                 // Emit outputs for next steps (CI provider format)\n\
                 emit_step_outputs(&step_name, &outputs, &env_dict);\n\
                 \n\
                 // Check for failure\n\
                 if let Some(Value::Bool(false)) = outputs.get(\"{success_port_or_empty}\") {{\n\
                     process::exit(1);\n\
                 }}\n\
             }}\n\
             Err(e) => {{\n\
                 eprintln!(\"Error executing step '{{}}': {{}}\", step_name, e);\n\
                 process::exit(1);\n\
             }}\n\
         }}",
        tool_name = tool.tool_name,
        graph_builder_call = graph_builder_call,
        dry_run_block = dry_run_block,
        success_port_or_empty = success_port_or_empty,
    );

    FnDef {
        name: "run_single_step".to_string(),
        is_pub: false,
        params: vec![("args".to_string(), "&[String]".to_string())],
        return_type: None,
        body: vec![Stmt::TailExpr(Expr::RawCode(body_code))],
        doc: vec![],
        attributes: vec![],
    }
}

/// Build the `list_dag_steps()` function for step mode.
fn build_list_dag_steps_fn(tool: &ToolMeta) -> FnDef {
    let graph_builder_call = generate_graph_builder_call(tool);

    let body_code = format!(
        "let dag = {graph_builder_call};\n\
         \n\
         println!(\"Available steps for {tool_name}:\");\n\
         println!();\n\
         \n\
         // Get nodes in topological order\n\
         for node in &dag.nodes {{\n\
             let inputs: Vec<_> = node.inputs.iter().map(|p| p.name.0.as_str()).collect();\n\
             let outputs: Vec<_> = node.outputs.iter().map(|p| p.name.0.as_str()).collect();\n\
             \n\
             println!(\"  {{}}\", node.id.0);\n\
             if !inputs.is_empty() {{\n\
                 println!(\"    inputs:  {{}}\", inputs.join(\", \"));\n\
             }}\n\
             if !outputs.is_empty() {{\n\
                 println!(\"    outputs: {{}}\", outputs.join(\", \"));\n\
             }}\n\
         }}",
        graph_builder_call = graph_builder_call,
        tool_name = tool.tool_name,
    );

    FnDef {
        name: "list_dag_steps".to_string(),
        is_pub: false,
        params: vec![],
        return_type: None,
        body: vec![Stmt::TailExpr(Expr::RawCode(body_code))],
        doc: vec![],
        attributes: vec![],
    }
}

/// Build the `load_step_inputs_from_env()` function.
fn build_load_step_inputs_fn() -> FnDef {
    let body_code = "\
let mut inputs = HashMap::new();\n\
\n\
// Look for environment variables matching our convention\n\
for (key, value) in env_dict {\n\
    if key.starts_with(\"STEP_\") && key != format!(\"STEP_{}_\", step_name.to_uppercase()) {\n\
        // Parse: STEP_NODENAME_PORTNAME\n\
        let parts: Vec<&str> = key.splitn(3, '_').collect();\n\
        if parts.len() >= 3 {\n\
            let port_name = parts[2].to_lowercase();\n\
            // Try to parse as appropriate type\n\
            if let Ok(b) = value.parse::<bool>() {\n\
                inputs.insert(port_name, Value::Bool(b));\n\
            } else if let Ok(i) = value.parse::<i64>() {\n\
                inputs.insert(port_name, Value::Int(i));\n\
            } else {\n\
                inputs.insert(port_name, Value::Str(value.clone()));\n\
            }\n\
        }\n\
    }\n\
}\n\
\n\
inputs"
        .to_string();

    FnDef {
        name: "load_step_inputs_from_env".to_string(),
        is_pub: false,
        params: vec![
            ("step_name".to_string(), "&str".to_string()),
            (
                "env_dict".to_string(),
                "&HashMap<String, String>".to_string(),
            ),
        ],
        return_type: Some("HashMap<String, Value>".to_string()),
        body: vec![Stmt::TailExpr(Expr::RawCode(body_code))],
        doc: vec![
            "Load inputs from environment variables set by previous CI steps.".to_string(),
            String::new(),
            "Convention: STEP_<NODE>_<PORT> = value".to_string(),
            String::new(),
            "Accepts an env dictionary captured at the boundary instead of reading".to_string(),
            "env vars directly, making this function pure and testable.".to_string(),
        ],
        attributes: vec![],
    }
}

/// Build the `emit_step_outputs()` function.
fn build_emit_step_outputs_fn() -> FnDef {
    let body_code = "\
// Check if we're in GitHub Actions\n\
if let Some(output_file) = env_dict.get(\"GITHUB_OUTPUT\") {\n\
    // GitHub Actions format: write to $GITHUB_OUTPUT file\n\
    let io = TransportIo::new();\n\
    let mut payload = String::new();\n\
    for (port, value) in outputs {\n\
        let str_value = match value {\n\
            Value::Str(s) => s.clone(),\n\
            Value::Int(i) => i.to_string(),\n\
            Value::Bool(b) => b.to_string(),\n\
            _ => continue,\n\
        };\n\
        if !payload.is_empty() {\n\
            payload.push('\\n');\n\
        }\n\
        write!(payload, \"STEP_{}_{}={}\",\n\
            step_name.to_uppercase(), port.to_uppercase(), str_value).unwrap();\n\
    }\n\
    if !payload.is_empty() {\n\
        let mut combined = String::new();\n\
        if let Ok(existing) = io.read_file(std::path::Path::new(output_file)) {\n\
            if let Ok(existing_str) = String::from_utf8(existing) {\n\
                combined = existing_str;\n\
            }\n\
        }\n\
        if !combined.is_empty() && !combined.ends_with('\\n') {\n\
            combined.push('\\n');\n\
        }\n\
        combined.push_str(&payload);\n\
        combined.push('\\n');\n\
        let _ = io.write_file(std::path::Path::new(output_file), combined.as_bytes());\n\
    }\n\
} else if env_dict.contains_key(\"GITLAB_CI\") {\n\
    // GitLab CI format: export to dotenv artifact\n\
    for (port, value) in outputs {\n\
        let str_value = match value {\n\
            Value::Str(s) => s.clone(),\n\
            Value::Int(i) => i.to_string(),\n\
            Value::Bool(b) => b.to_string(),\n\
            _ => continue,\n\
        };\n\
        println!(\"STEP_{}_{}={}\",\n\
            step_name.to_uppercase(), port.to_uppercase(), str_value);\n\
    }\n\
}\n\
// Plain mode: just print (already done above)"
        .to_string();

    FnDef {
        name: "emit_step_outputs".to_string(),
        is_pub: false,
        params: vec![
            ("step_name".to_string(), "&str".to_string()),
            ("outputs".to_string(), "&HashMap<String, Value>".to_string()),
            (
                "env_dict".to_string(),
                "&HashMap<String, String>".to_string(),
            ),
        ],
        return_type: None,
        body: vec![Stmt::TailExpr(Expr::RawCode(body_code))],
        doc: vec![
            "Emit outputs in CI provider format for next steps.".to_string(),
            String::new(),
            "Accepts an env dictionary captured at the boundary instead of reading".to_string(),
            "env vars directly, making this function pure and testable.".to_string(),
        ],
        attributes: vec![],
    }
}

/// Build the `print_help()` function for step mode.
fn build_step_help_fn(tool: &ToolMeta) -> FnDef {
    let body_code = format!(
        "println!(\"{tool_name} - {description}\");\n\
         println!();\n\
         println!(\"USAGE:\");\n\
         println!(\"    {tool_name} [SUBCOMMAND] [OPTIONS]\");\n\
         println!();\n\
         println!(\"SUBCOMMANDS:\");\n\
         println!(\"    run          Execute the full DAG (default)\");\n\
         println!(\"    step <node>  Execute a single DAG node\");\n\
         println!(\"    list-steps   List all available steps\");\n\
         println!(\"    help         Print this help\");\n\
         println!();\n\
         println!(\"OPTIONS:\");\n\
         println!(\"    -n, --dry-run    Don't perform actual I/O\");\n\
         println!(\"    -h, --help       Print this help\");\n\
         println!();\n\
         println!(\"Progress display is automatic based on terminal capabilities.\");",
        tool_name = tool.tool_name,
        description = tool.description,
    );

    FnDef {
        name: "print_help".to_string(),
        is_pub: false,
        params: vec![],
        return_type: None,
        body: vec![Stmt::TailExpr(Expr::RawCode(body_code))],
        doc: vec![],
        attributes: vec![],
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_entrypoint_flag_name() {
        let ep = CliEntrypoint::new("repo_path", ParamType::Str);
        assert_eq!(ep.flag_name(), "repo-path");
    }

    #[test]
    fn test_generate_simple_cli() {
        let tool = ToolMeta {
            crate_name: "gunbc-gist".to_string(),
            tool_name: "gist".to_string(),
            description: "Create gist from files".to_string(),
            graph_builder_call: "build_gist_graph".to_string(),
            graph_builder_args: "extensions.clone(), public".to_string(),
            returns_result: false,
            success_port: None,
            enable_step_mode: false,
            mock_spec_call: Some("gunbc_gist::graph_mock::gist_snapshot_mock_spec()".to_string()),
        };

        let entrypoints = vec![CliEntrypoint::new("repo_path", ParamType::Str)
            .short('r')
            .help("Repository path")];

        let code = generate_cli(&tool, &entrypoints);
        assert!(code.contains("--repo-path"));
        assert!(code.contains("--dry-run"));
        assert!(code.contains("build_gist_graph"));
        assert!(code.contains("execute_and_display"));
        assert!(code.contains("print_preamble_auto"));
        assert!(code.contains("Preamble::with_body"));
    }

    #[test]
    fn test_generate_cli_uses_ir_imports() {
        let tool = ToolMeta {
            crate_name: "gunbc-gist".to_string(),
            tool_name: "gist".to_string(),
            description: "Test".to_string(),
            graph_builder_call: "build_gist_graph".to_string(),
            graph_builder_args: String::new(),
            returns_result: false,
            success_port: None,
            enable_step_mode: false,
            mock_spec_call: Some("mock_spec()".to_string()),
        };
        let entrypoints = vec![];

        let code = generate_cli(&tool, &entrypoints);
        // Verify IR-based imports are rendered (not embedded in Raw string)
        assert!(code.contains("use gunbc_exec::{"));
        assert!(code.contains("use gunbc_ir::"));
        assert!(code.contains("use std::env;"));
        assert!(code.contains("use std::process;"));
        // Verify functions are rendered as proper fn definitions
        assert!(code.contains("fn main()"));
        assert!(code.contains("fn print_help()"));
    }

    #[test]
    fn test_generate_step_mode_cli() {
        let tool = ToolMeta {
            crate_name: "gunbc-ci".to_string(),
            tool_name: "ci".to_string(),
            description: "CI pipeline".to_string(),
            graph_builder_call: "build_ci_graph".to_string(),
            graph_builder_args: String::new(),
            returns_result: true,
            success_port: Some("overall_success".to_string()),
            enable_step_mode: true,
            mock_spec_call: Some("ci_mock_spec()".to_string()),
        };
        let entrypoints = vec![];

        let code = generate_cli(&tool, &entrypoints);
        assert!(code.contains("fn main()"));
        assert!(code.contains("fn run_full_dag("));
        assert!(code.contains("fn run_single_step("));
        assert!(code.contains("fn list_dag_steps()"));
        assert!(code.contains("fn load_step_inputs_from_env("));
        assert!(code.contains("fn emit_step_outputs("));
        assert!(code.contains("fn print_help()"));
        assert!(code.contains("gunbc_cli::parse_step_mode"));
        assert!(code.contains("gunbc_cli::StepModeSubcommand::Run"));
        assert!(code.contains("execute_single_node"));
        assert!(code.contains("print_value"));
    }

    #[test]
    fn test_generate_cli_with_result_builder() {
        let tool = ToolMeta {
            crate_name: "gunbc-ci".to_string(),
            tool_name: "ci".to_string(),
            description: "Test".to_string(),
            graph_builder_call: "build_ci_graph".to_string(),
            graph_builder_args: String::new(),
            returns_result: true,
            success_port: None,
            enable_step_mode: false,
            mock_spec_call: Some("mock()".to_string()),
        };
        let entrypoints = vec![];

        let code = generate_cli(&tool, &entrypoints);
        assert!(code.contains("match build_ci_graph()"));
        assert!(code.contains("Ok(d) => d"));
        assert!(code.contains("process::exit(1)"));
    }

    #[test]
    fn test_source_file_structure() {
        let tool = ToolMeta {
            crate_name: "gunbc-gist".to_string(),
            tool_name: "gist".to_string(),
            description: "Test".to_string(),
            graph_builder_call: "build_gist_graph".to_string(),
            graph_builder_args: String::new(),
            returns_result: false,
            success_port: None,
            enable_step_mode: false,
            mock_spec_call: Some("mock()".to_string()),
        };
        let entrypoints = vec![];

        let file = build_cli_source_file(&tool, &entrypoints, None);
        // Should have doc comments
        assert!(!file.doc.is_empty());
        // Should have imports + 2 functions (main, print_help)
        let fn_count = file
            .items
            .iter()
            .filter(|i| matches!(i, Item::Fn(_)))
            .count();
        assert_eq!(fn_count, 2, "standard mode should have 2 functions");
        let import_count = file
            .items
            .iter()
            .filter(|i| matches!(i, Item::Use(_)))
            .count();
        assert!(import_count >= 4, "should have at least 4 import items");
    }

    #[test]
    fn test_step_mode_source_file_structure() {
        let tool = ToolMeta {
            crate_name: "gunbc-ci".to_string(),
            tool_name: "ci".to_string(),
            description: "CI".to_string(),
            graph_builder_call: "build_ci_graph".to_string(),
            graph_builder_args: String::new(),
            returns_result: true,
            success_port: Some("overall_success".to_string()),
            enable_step_mode: true,
            mock_spec_call: Some("mock()".to_string()),
        };
        let entrypoints = vec![];

        let file = build_step_mode_source_file(&tool, &entrypoints, None);
        let fn_count = file
            .items
            .iter()
            .filter(|i| matches!(i, Item::Fn(_)))
            .count();
        assert_eq!(fn_count, 7, "step mode should have 7 functions");
    }
}
