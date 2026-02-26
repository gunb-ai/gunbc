//! Tool registry for CLI generation.
//!
//! Tool metadata is derived from DSL structural entrypoint inference via
//! `discover_tool_defs_from_dsl()` in `gunbc-dag/src/dsl_registry.rs`.

use crate::cli_gen::{CliEntrypoint, ToolMeta};
use gunbc_ir::cargo;
use gunbc_test::{FermiCost, TestClass};
use std::borrow::Cow;

// ============================================================================
// Tool Definition
// ============================================================================

/// A tool that needs CLI generation.
pub struct ToolDef {
    pub meta: ToolMeta,
    pub entrypoints: Vec<CliEntrypoint>,
    /// Custom import line (if different from default pattern)
    pub custom_import: Option<String>,
    /// Output artifacts produced by this tool (for clean/rollback)
    pub outputs: Vec<String>,
    /// Cargo invocation for running this tool.
    /// When set, the tool gets a Makefile target automatically.
    /// When None, the tool has no runnable binary (e.g., library-only or not wired up yet).
    pub invocation: Option<cargo::CargoInvocation>,
}

/// Configuration for test generation.
#[derive(Debug, Clone)]
pub struct TestgenTargetDef {
    /// Short identifier (e.g., "bootstrap", "llm-openai")
    pub name: Cow<'static, str>,
    /// Output path for generated tests (relative to workspace)
    pub output_path: Cow<'static, str>,
    /// Module name for the generated test module
    pub module_name: Cow<'static, str>,
    /// MockSpec function path (e.g., "crate::graph_mock::my_mock_spec")
    pub mock_spec_path: Cow<'static, str>,
    /// DAG builder call expression (e.g., "crate::build_graph().unwrap()")
    pub dag_builder_call: Cow<'static, str>,
    /// Signature function path (e.g., "crate::makegen_signature()")
    pub signature_path: Option<Cow<'static, str>>,
    /// Enable boundary tests
    pub boundary_tests: bool,
    /// Enable chain tests
    pub chain_tests: bool,
    /// Enable flow tests
    pub flow_tests: bool,
    /// Enable live flow tests (Real execution)
    pub live_flow_tests: bool,
    /// Max window size for windowed tests (None = no limit)
    pub window_max_nodes: Option<usize>,
    /// Test class override (unit/hermetic/integration)
    pub test_class: Option<TestClass>,
    /// Fermi cost override
    pub fermi_cost: Option<FermiCost>,
    /// External requirements override
    pub requires: Option<Vec<String>>,
    /// Required secrets override (env vars)
    pub secrets: Option<Vec<String>>,
    /// Live test class override
    pub live_test_class: Option<TestClass>,
    /// Live test cost override
    pub live_fermi_cost: Option<FermiCost>,
    /// Live test external requirements override
    pub live_requires: Option<Vec<String>>,
    /// Live test required env vars (hard requirements)
    pub live_required: Option<Vec<String>>,
    /// Live test required any-of env var groups
    pub live_required_any_of: Option<Vec<Vec<String>>>,
    /// Tool name for CLI contract test generation. When set, entrypoints
    /// are looked up from DSL-driven `discover_tool_defs_from_dsl()` and a CLI contract test is emitted
    /// alongside the DAG tests.
    pub tool_name: Option<Cow<'static, str>>,
    /// Per-profile live test configurations (PT-3).
    /// Each entry generates a `test_live_flow_{module}_{profile}()` function
    /// gated by the profile's env requirements.
    pub live_profile_tests: Vec<LiveProfileTestConfig>,
}

/// Configuration for a per-profile live test (PT-3).
///
/// Each config generates one test function that compiles the module with
/// the specified profile and executes with `ExecutionMode::Real`.
#[derive(Debug, Clone)]
pub struct LiveProfileTestConfig {
    /// Profile name (e.g., "unit_test", "local").
    pub profile_name: String,
    /// Test class for this profile test (Hermetic, Integration, etc.).
    pub test_class: TestClass,
    /// Estimated cost for test prioritization.
    pub fermi_cost: FermiCost,
    /// Environment variables that MUST be set for this test to run.
    pub required_env: Vec<String>,
    /// Groups of env vars where at least one must be set.
    pub required_any_of: Vec<Vec<String>>,
    /// DAG builder call expression for this profile.
    pub dag_builder_call: String,
}

impl TestgenTargetDef {
    /// Create a new testgen target definition.
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        output_path: impl Into<Cow<'static, str>>,
        module_name: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            name: name.into(),
            output_path: output_path.into(),
            module_name: module_name.into(),
            mock_spec_path: Cow::Borrowed(""),
            dag_builder_call: Cow::Borrowed(""),
            signature_path: None,
            boundary_tests: true,
            chain_tests: true,
            flow_tests: false,
            live_flow_tests: false,
            window_max_nodes: None, // Deprecated: use probe_observer_tests instead
            test_class: None,
            fermi_cost: None,
            requires: None,
            secrets: None,
            live_test_class: None,
            live_fermi_cost: None,
            live_requires: None,
            live_required: None,
            live_required_any_of: None,
            tool_name: None,
            live_profile_tests: Vec::new(),
        }
    }

    /// Set the MockSpec function path.
    pub fn mock_spec(mut self, path: impl Into<Cow<'static, str>>) -> Self {
        self.mock_spec_path = path.into();
        self
    }

    /// Set the DAG builder call expression.
    pub fn dag_builder(mut self, call: impl Into<Cow<'static, str>>) -> Self {
        self.dag_builder_call = call.into();
        self
    }

    /// Set the signature function path.
    pub fn signature(mut self, path: impl Into<Cow<'static, str>>) -> Self {
        self.signature_path = Some(path.into());
        self
    }

    /// Enable flow tests (and disable boundary/chain tests).
    pub fn flow_tests(mut self) -> Self {
        self.boundary_tests = false;
        self.chain_tests = false;
        self.flow_tests = true;
        self
    }

    /// Set the max window size for windowed tests.
    pub fn window_max_nodes(mut self, max: usize) -> Self {
        self.window_max_nodes = Some(max);
        self
    }

    /// Disable boundary tests.
    pub fn no_boundary_tests(mut self) -> Self {
        self.boundary_tests = false;
        self
    }
}

impl ToolDef {
    pub fn new(
        crate_name: impl Into<Cow<'static, str>>,
        tool_name: impl Into<Cow<'static, str>>,
        description: impl Into<Cow<'static, str>>,
        graph_builder_call: impl Into<Cow<'static, str>>,
        graph_builder_args: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            meta: ToolMeta {
                crate_name: crate_name.into(),
                tool_name: tool_name.into(),
                description: description.into(),
                graph_builder_call: graph_builder_call.into(),
                graph_builder_args: graph_builder_args.into(),
                returns_result: false,
                success_port: None,
                enable_step_mode: false,
                mock_spec_call: None,
            },
            entrypoints: vec![],
            custom_import: None,
            outputs: vec![],
            invocation: None,
        }
    }

    /// Mark that this tool's graph builder returns Result<Dag, BuilderError>.
    pub fn returns_result(mut self) -> Self {
        self.meta.returns_result = true;
        self
    }

    /// Set the output port to check for success.
    /// If this port is false, the CLI exits with code 1.
    pub fn check_success(mut self, port_name: impl Into<Cow<'static, str>>) -> Self {
        self.meta.success_port = Some(port_name.into());
        self
    }

    /// Enable step mode for this tool.
    pub fn enable_step_mode(mut self) -> Self {
        self.meta.enable_step_mode = true;
        self
    }

    /// Set the cargo invocation for running this tool.
    pub fn invocation(mut self, inv: cargo::CargoInvocation) -> Self {
        self.invocation = Some(inv);
        self
    }

    /// Set a custom import line.
    pub fn import(mut self, import_line: impl Into<String>) -> Self {
        self.custom_import = Some(import_line.into());
        self
    }

    /// Add an output artifact (file or directory produced by this tool).
    pub fn output(mut self, path: impl Into<String>) -> Self {
        self.outputs.push(path.into());
        self
    }

    pub fn entrypoint(mut self, ep: CliEntrypoint) -> Self {
        self.entrypoints.push(ep);
        self
    }

    /// Set entrypoints from a JSON string.
    pub fn entrypoints_json(mut self, json: &str) -> Self {
        self.entrypoints = CliEntrypoint::from_json(json);
        self
    }

    /// Set the mock_spec_call expression for dry-run boundary mocking.
    pub fn mock_spec_call(mut self, call: impl Into<Cow<'static, str>>) -> Self {
        self.meta.mock_spec_call = Some(call.into());
        self
    }
}

/// Core build system artifacts (not tool-specific).
pub fn core_outputs() -> Vec<&'static str> {
    vec![
        "target/", // cargo build output
        "bin",     // symlink to target/release
    ]
}
