//! Generic entrypoint abstraction.
//!
//! Separates structural facts about entrypoints (port name, type, cardinality,
//! default) from exposure-specific policy (CLI flags, REST bindings, Lambda
//! event paths). Each exposure type implements [`ExposureCodegen`] to derive
//! its own per-parameter config from the shared [`EntrypointParam`].

use gunbc_cli::ParamType;
use gunbc_ir::code_ir::SourceFile;
use gunbc_ir::Cardinality;
use std::collections::HashSet;

use crate::cli_gen::ToolMeta;

/// A parameter on an inferred entrypoint. Structural fact, not exposure policy.
///
/// This is the stable seam between inference (which discovers entrypoints) and
/// exposure (which maps them to CLI/REST/Lambda interfaces).
#[derive(Debug, Clone)]
pub struct EntrypointParam {
    /// Port name (e.g., "base_ref").
    pub port_name: String,
    /// Parameter type.
    pub type_id: ParamType,
    /// Cardinality (ONE, ZERO_OR_ONE, ZERO_OR_MORE).
    pub cardinality: Cardinality,
    /// Default value from DSL source.
    pub default: Option<String>,
}

/// A discovered entrypoint with its generic parameters.
///
/// This is what inference produces — no exposure-specific info yet.
#[derive(Debug, Clone)]
pub struct EntrypointDef {
    /// Tool metadata (name, description, graph builder call, etc.).
    pub meta: ToolMeta,
    /// Generic parameters, derived from the DSL func signature.
    pub params: Vec<EntrypointParam>,
    /// Output port names.
    pub outputs: Vec<String>,
    /// Cargo invocation for running this entrypoint.
    pub invocation: Option<gunbc_ir::cargo::CargoInvocation>,
}

/// How an entrypoint is exposed to the outside world.
///
/// Each exposure type (CLI, REST, Lambda) implements this trait to:
/// 1. Derive exposure-specific config per parameter (short flags, query bindings, etc.)
/// 2. Generate the source file for that exposure
pub trait ExposureCodegen {
    /// Per-parameter exposure configuration produced by this exposure type.
    type ParamConfig;

    /// Derive exposure-specific config for a parameter using conventions.
    ///
    /// This is where CLI derives short flags, REST derives query/body binding,
    /// Lambda derives JSON field names, etc.
    fn derive_param_config(
        &self,
        param: &EntrypointParam,
        context: &ExposureContext,
    ) -> Self::ParamConfig;

    /// Generate the source file for this entrypoint under this exposure type.
    fn generate(
        &self,
        entrypoint: &EntrypointDef,
        params: &[(EntrypointParam, Self::ParamConfig)],
    ) -> SourceFile;
}

/// Shared context for exposure derivation.
///
/// Carries state that prevents collisions (e.g., short-flag dedup for CLI)
/// and provides scoping info (module name for namespacing).
pub struct ExposureContext {
    /// Previously used short flags (for CLI collision avoidance).
    pub used_short_flags: HashSet<char>,
    /// Module name for namespacing.
    pub module_name: String,
}

impl ExposureContext {
    /// Create a new context for a module.
    pub fn new(module_name: impl Into<String>) -> Self {
        Self {
            used_short_flags: HashSet::new(),
            module_name: module_name.into(),
        }
    }
}
