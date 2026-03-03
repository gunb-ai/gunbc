//! Make target modeling and invariants for makegen renderers.
//!
//! This module defines the structural model that both Makefile and Justfile
//! projections must satisfy. In particular, target names live in one global
//! namespace; collisions are rejected before any file content is rendered.

use std::collections::BTreeMap;
use std::path::Path;

use daglang_driver::compile_data_from_sources;
use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::makegen::registry::ToolRegistry;

const BUILD_TARGETS_SOURCE: &str = include_str!("../../../dsl/config/build_targets.dag");
const EXTDEPS_MAKE_SOURCE: &str = include_str!("../../../dsl/extdeps/make.dag");

/// Core workflow declaration loaded from `config/build_targets.dag`.
#[derive(Debug, Clone, Deserialize)]
pub struct CoreWorkflowData {
    pub name: String,
    pub description: String,
    pub deps: Vec<String>,
    pub body: Vec<String>,
    pub comment: Option<String>,
}

/// Resource need declaration loaded from `config/build_targets.dag`.
#[derive(Debug, Clone, Deserialize)]
pub struct ResourceNeedData {
    pub resource: String,
    pub mode: String,
}

/// Meta target declaration loaded from `config/build_targets.dag`.
#[derive(Debug, Clone, Deserialize)]
pub struct MetaTargetData {
    pub name: String,
    pub description: String,
    pub has_check: bool,
    pub has_fix: bool,
    pub command: String,
    pub check_command: Option<String>,
    pub fix_command: Option<String>,
    pub command_prefix: Option<String>,
    pub resource_needs: Vec<ResourceNeedData>,
    pub fix_prerequisites: Vec<String>,
}

/// Resource target mapping loaded from `config/build_targets.dag`.
#[derive(Debug, Clone, Deserialize)]
pub struct ResourceTargetEntryData {
    pub resource: String,
    pub ensure_target: String,
    pub verify_target: String,
}

/// Parsed, typed build-target declarations.
#[derive(Debug, Clone)]
pub struct BuildTargetsData {
    pub core_workflows: Vec<CoreWorkflowData>,
    pub meta_targets: Vec<MetaTargetData>,
    pub resource_targets: Vec<ResourceTargetEntryData>,
}

/// Source category for a concrete make target name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetOrigin {
    Help,
    Core,
    Meta,
    MetaFix,
    MetaCheck,
    Tool,
    ToolDry,
    ToolExtra,
}

/// Origin metadata for a single target name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetSource {
    pub origin: TargetOrigin,
    pub owner: String,
}

/// Typed model-level errors for make target invariants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MakegenModelError {
    BuildTargetsCompile { details: String },
    MissingData { key: &'static str },
    DeserializeData { key: &'static str, details: String },
    DuplicateTargetName {
        name: String,
        first: TargetSource,
        second: TargetSource,
    },
}

impl std::fmt::Display for MakegenModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MakegenModelError::BuildTargetsCompile { details } => {
                write!(f, "failed to compile build target model: {details}")
            }
            MakegenModelError::MissingData { key } => {
                write!(f, "build target model missing data declaration `{key}`")
            }
            MakegenModelError::DeserializeData { key, details } => {
                write!(f, "failed to deserialize `{key}` from build target model: {details}")
            }
            MakegenModelError::DuplicateTargetName {
                name,
                first,
                second,
            } => write!(
                f,
                "duplicate make target `{name}` (first: {:?} `{}`, second: {:?} `{}`)",
                first.origin, first.owner, second.origin, second.owner
            ),
        }
    }
}

impl std::error::Error for MakegenModelError {}

/// Load typed build target declarations from DSL sources.
pub fn load_build_targets_data() -> Result<BuildTargetsData, MakegenModelError> {
    let output = compile_data_from_sources(&[
        (
            Path::new("<embedded>/extdeps/make.dag"),
            EXTDEPS_MAKE_SOURCE,
        ),
        (
            Path::new("<embedded>/config/build_targets.dag"),
            BUILD_TARGETS_SOURCE,
        ),
    ])
    .map_err(|details| MakegenModelError::BuildTargetsCompile {
        details: details.to_string(),
    })?;

    let core_workflows =
        deserialize_data_vec::<CoreWorkflowData>(&output.data_values, "core_workflows")?;
    let meta_targets = deserialize_data_vec::<MetaTargetData>(&output.data_values, "meta_targets")?;
    let resource_targets =
        deserialize_data_vec::<ResourceTargetEntryData>(&output.data_values, "resource_targets")?;

    Ok(BuildTargetsData {
        core_workflows,
        meta_targets,
        resource_targets,
    })
}

/// Build a uniqueness-enforced index of all make target names.
pub fn index_unique_target_names(
    registry: &ToolRegistry,
    build_targets: &BuildTargetsData,
) -> Result<BTreeMap<String, TargetSource>, MakegenModelError> {
    let mut index = BTreeMap::new();

    insert_target(
        &mut index,
        "help".to_string(),
        TargetSource {
            origin: TargetOrigin::Help,
            owner: "help".to_string(),
        },
    )?;

    for workflow in &build_targets.core_workflows {
        insert_target(
            &mut index,
            workflow.name.clone(),
            TargetSource {
                origin: TargetOrigin::Core,
                owner: workflow.name.clone(),
            },
        )?;
    }

    for meta in &build_targets.meta_targets {
        insert_target(
            &mut index,
            meta.name.clone(),
            TargetSource {
                origin: TargetOrigin::Meta,
                owner: meta.name.clone(),
            },
        )?;
        if meta.has_fix {
            insert_target(
                &mut index,
                format!("{}-fix", meta.name),
                TargetSource {
                    origin: TargetOrigin::MetaFix,
                    owner: meta.name.clone(),
                },
            )?;
        }
        if meta.has_check {
            insert_target(
                &mut index,
                format!("{}-check", meta.name),
                TargetSource {
                    origin: TargetOrigin::MetaCheck,
                    owner: meta.name.clone(),
                },
            )?;
        }
    }

    for tool in &registry.tools {
        insert_target(
            &mut index,
            tool.short_name.clone(),
            TargetSource {
                origin: TargetOrigin::Tool,
                owner: tool.short_name.clone(),
            },
        )?;
        insert_target(
            &mut index,
            format!("{}-dry", tool.short_name),
            TargetSource {
                origin: TargetOrigin::ToolDry,
                owner: tool.short_name.clone(),
            },
        )?;
        for extra in &tool.extra_targets {
            insert_target(
                &mut index,
                format!("{}-{}", tool.short_name, extra.suffix),
                TargetSource {
                    origin: TargetOrigin::ToolExtra,
                    owner: format!("{}::{}", tool.short_name, extra.suffix),
                },
            )?;
        }
    }

    Ok(index)
}

/// Validate the target namespace using the current DSL build target model.
pub fn validate_target_namespace(registry: &ToolRegistry) -> Result<(), MakegenModelError> {
    let build_targets = load_build_targets_data()?;
    validate_target_namespace_with_data(registry, &build_targets)
}

/// Validate the target namespace with preloaded build target data.
pub fn validate_target_namespace_with_data(
    registry: &ToolRegistry,
    build_targets: &BuildTargetsData,
) -> Result<(), MakegenModelError> {
    let _ = index_unique_target_names(registry, build_targets)?;
    Ok(())
}

fn deserialize_data_vec<T: DeserializeOwned>(
    data_values: &std::collections::HashMap<String, serde_json::Value>,
    key: &'static str,
) -> Result<Vec<T>, MakegenModelError> {
    let value = data_values
        .get(key)
        .cloned()
        .ok_or(MakegenModelError::MissingData { key })?;
    serde_json::from_value::<Vec<T>>(value).map_err(|err| MakegenModelError::DeserializeData {
        key,
        details: err.to_string(),
    })
}

fn insert_target(
    index: &mut BTreeMap<String, TargetSource>,
    name: String,
    source: TargetSource,
) -> Result<(), MakegenModelError> {
    if let Some(first) = index.get(&name) {
        return Err(MakegenModelError::DuplicateTargetName {
            name,
            first: first.clone(),
            second: source,
        });
    }
    index.insert(name, source);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::makegen::registry::{ToolInfo, ToolRegistry};
    use gunbc_ir::cargo::CargoInvocation;

    #[test]
    fn detects_duplicate_names_across_core_and_tool_targets() {
        let mut registry = ToolRegistry::new();
        registry.register(ToolInfo {
            invocation: CargoInvocation::composed("codegen", "dag"),
            short_name: "codegen".to_string(),
            description: "Codegen".to_string(),
            entrypoints: Vec::new(),
            extra_targets: Vec::new(),
            has_declarative_dag: false,
            needs_generated_cli: true,
            live_secrets: Vec::new(),
        });
        let build_targets = BuildTargetsData {
            core_workflows: vec![CoreWorkflowData {
                name: "codegen".to_string(),
                description: "core codegen".to_string(),
                deps: Vec::new(),
                body: Vec::new(),
                comment: None,
            }],
            meta_targets: Vec::new(),
            resource_targets: Vec::new(),
        };

        let err = index_unique_target_names(&registry, &build_targets).expect_err("must collide");
        match err {
            MakegenModelError::DuplicateTargetName { name, first, second } => {
                assert_eq!(name, "codegen");
                assert_eq!(first.origin, TargetOrigin::Core);
                assert_eq!(second.origin, TargetOrigin::Tool);
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn accepts_disjoint_target_names() {
        let mut registry = ToolRegistry::new();
        registry.register(ToolInfo {
            invocation: CargoInvocation::composed("gist", "dag"),
            short_name: "gist".to_string(),
            description: "Gist".to_string(),
            entrypoints: Vec::new(),
            extra_targets: Vec::new(),
            has_declarative_dag: false,
            needs_generated_cli: true,
            live_secrets: Vec::new(),
        });
        let build_targets = BuildTargetsData {
            core_workflows: vec![CoreWorkflowData {
                name: "codegen".to_string(),
                description: "core codegen".to_string(),
                deps: Vec::new(),
                body: Vec::new(),
                comment: None,
            }],
            meta_targets: vec![MetaTargetData {
                name: "fmt".to_string(),
                description: "format".to_string(),
                has_check: true,
                has_fix: false,
                command: "@cargo fmt".to_string(),
                check_command: Some("@cargo fmt --check".to_string()),
                fix_command: None,
                command_prefix: None,
                resource_needs: Vec::new(),
                fix_prerequisites: Vec::new(),
            }],
            resource_targets: Vec::new(),
        };

        let names = index_unique_target_names(&registry, &build_targets).expect("must be unique");
        assert!(names.contains_key("help"));
        assert!(names.contains_key("codegen"));
        assert!(names.contains_key("fmt"));
        assert!(names.contains_key("fmt-check"));
        assert!(names.contains_key("gist"));
        assert!(names.contains_key("gist-dry"));
    }
}
