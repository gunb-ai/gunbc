//! Resource definitions and input patterns.
//!
//! This module defines how resources declare their inputs and outputs.
//! The key insight is that hash scope is **derived from declared inputs**,
//! not configured — similar to Bazel's action graph.

use super::super::ResourceId;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A pattern describing an input to a resource.
///
/// Inputs determine the freshness key — if any input changes, the resource
/// becomes stale. This follows the "deduce over configure" principle.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InputPattern {
    /// Glob pattern for source files (e.g., "src/**/*.rs").
    ///
    /// Files matching the pattern are hashed. The hash includes:
    /// - File contents (not metadata/timestamps)
    /// - Sorted paths for deterministic ordering
    Glob(String),

    /// Another resource's freshness key (transitive dependency).
    ///
    /// When computing this resource's hash, include the dependency's
    /// manifest key. This propagates staleness through the dependency chain.
    Resource(ResourceId),

    /// Environment variable value (e.g., toolchain version).
    ///
    /// The value of the environment variable is included in the hash.
    /// If the variable is unset, an empty string is used.
    Env(String),

    /// A single specific file path.
    ///
    /// Like Glob but for exactly one file. Useful for config files
    /// or other single-file inputs.
    File(PathBuf),

    /// Output of a command (e.g., `rustc --version`).
    ///
    /// Runs the command at hash time and hashes its stdout.
    /// Fails hard if the command cannot be executed.
    CommandOutput { command: String, args: Vec<String> },
}

impl InputPattern {
    /// Create a glob pattern input.
    pub fn glob(pattern: impl Into<String>) -> Self {
        Self::Glob(pattern.into())
    }

    /// Create a resource dependency input.
    pub fn resource(id: ResourceId) -> Self {
        Self::Resource(id)
    }

    /// Create an environment variable input.
    pub fn env(var: impl Into<String>) -> Self {
        Self::Env(var.into())
    }

    /// Create a single file input.
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self::File(path.into())
    }

    /// Create a command output input.
    ///
    /// The command is run at hash time and its stdout is hashed.
    pub fn command_output(command: impl Into<String>, args: &[&str]) -> Self {
        Self::CommandOutput {
            command: command.into(),
            args: args.iter().map(|s| (*s).to_string()).collect(),
        }
    }
}

/// Scope of a resource's outputs.
///
/// Resources can be as granular as needed — a single file, a pattern,
/// or a named logical resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceScope {
    /// A single output file.
    File(PathBuf),

    /// Multiple files matching a glob pattern.
    Pattern(String),

    /// A named logical resource (abstract, no specific files).
    Named(String),
}

impl ResourceScope {
    /// Create a single file scope.
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self::File(path.into())
    }

    /// Create a pattern scope.
    pub fn pattern(pat: impl Into<String>) -> Self {
        Self::Pattern(pat.into())
    }

    /// Create a named scope.
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }
}

/// Reference to a DAG that can create a resource.
///
/// The provider is invoked when a resource is missing or stale
/// and the execution mode is `Ensure`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DagRef {
    /// Name/identifier of the DAG.
    pub name: String,
}

impl DagRef {
    /// Create a new DAG reference.
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

/// Definition of a managed resource.
///
/// This declares:
/// - What the resource is (id)
/// - What its inputs are (determines freshness key)
/// - What it produces (outputs)
/// - How to create it (provider DAG)
///
/// The hash scope is derived from `inputs` — no configuration needed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceDef {
    /// Unique identifier for this resource.
    pub id: ResourceId,

    /// Input patterns — these determine the freshness key.
    ///
    /// The hash is computed from all matching files/values.
    pub inputs: Vec<InputPattern>,

    /// Output scope — what this resource produces.
    ///
    /// Used for cleanup and for downstream resources to reference.
    pub outputs: Vec<ResourceScope>,

    /// The DAG that creates this resource (if creatable).
    ///
    /// If None, the resource must be created externally (e.g., tools).
    pub provider: Option<DagRef>,
}

impl ResourceDef {
    /// Create a new resource definition.
    pub fn new(id: ResourceId) -> Self {
        Self {
            id,
            inputs: Vec::new(),
            outputs: Vec::new(),
            provider: None,
        }
    }

    /// Add an input pattern.
    pub fn with_input(mut self, input: InputPattern) -> Self {
        self.inputs.push(input);
        self
    }

    /// Add multiple input patterns.
    pub fn with_inputs(mut self, inputs: impl IntoIterator<Item = InputPattern>) -> Self {
        self.inputs.extend(inputs);
        self
    }

    /// Add an output scope.
    pub fn with_output(mut self, output: ResourceScope) -> Self {
        self.outputs.push(output);
        self
    }

    /// Add multiple output scopes.
    pub fn with_outputs(mut self, outputs: impl IntoIterator<Item = ResourceScope>) -> Self {
        self.outputs.extend(outputs);
        self
    }

    /// Set the provider DAG.
    pub fn with_provider(mut self, dag: DagRef) -> Self {
        self.provider = Some(dag);
        self
    }

    /// Check if this resource has a provider.
    pub fn has_provider(&self) -> bool {
        self.provider.is_some()
    }

    /// Get all resource dependencies (InputPattern::Resource entries).
    pub fn resource_dependencies(&self) -> impl Iterator<Item = &ResourceId> {
        self.inputs.iter().filter_map(|input| {
            if let InputPattern::Resource(id) = input {
                Some(id)
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_pattern_glob() {
        let pat = InputPattern::glob("src/**/*.rs");
        assert!(matches!(pat, InputPattern::Glob(s) if s == "src/**/*.rs"));
    }

    #[test]
    fn test_input_pattern_resource() {
        let id = ResourceId::new("build:codegen");
        let pat = InputPattern::resource(id.clone());
        assert!(matches!(pat, InputPattern::Resource(ref r) if r == &id));
    }

    #[test]
    fn test_input_pattern_env() {
        let pat = InputPattern::env("RUSTC_VERSION");
        assert!(matches!(pat, InputPattern::Env(s) if s == "RUSTC_VERSION"));
    }

    #[test]
    fn test_input_pattern_command_output() {
        let pat = InputPattern::command_output("rustc", &["--version"]);
        match pat {
            InputPattern::CommandOutput { command, args } => {
                assert_eq!(command, "rustc");
                assert_eq!(args, vec!["--version"]);
            }
            _ => panic!("expected CommandOutput"),
        }
    }

    #[test]
    fn test_resource_scope_file() {
        let scope = ResourceScope::file("target/output.txt");
        assert!(matches!(scope, ResourceScope::File(_)));
    }

    #[test]
    fn test_resource_def_builder() {
        let def = ResourceDef::new(ResourceId::new("build:test"))
            .with_input(InputPattern::glob("src/**/*.rs"))
            .with_input(InputPattern::env("CARGO_PKG_VERSION"))
            .with_output(ResourceScope::pattern("target/codegen/**"))
            .with_provider(DagRef::new("codegen"));

        assert_eq!(def.id.0, "build:test");
        assert_eq!(def.inputs.len(), 2);
        assert_eq!(def.outputs.len(), 1);
        assert!(def.has_provider());
    }

    #[test]
    fn test_resource_dependencies() {
        let dep1 = ResourceId::new("build:a");
        let dep2 = ResourceId::new("build:b");

        let def = ResourceDef::new(ResourceId::new("build:c"))
            .with_input(InputPattern::glob("*.rs"))
            .with_input(InputPattern::resource(dep1.clone()))
            .with_input(InputPattern::env("FOO"))
            .with_input(InputPattern::resource(dep2.clone()));

        let deps: Vec<_> = def.resource_dependencies().collect();
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&&dep1));
        assert!(deps.contains(&&dep2));
    }
}
