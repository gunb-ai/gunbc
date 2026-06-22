use self::ArtifactKind::*;
use self::BoundaryKind::*;
use self::DagInferredRecord::*;
use self::PartitionRule::*;
use self::RenderTarget::*;
pub use crate::std_types::SourceSpan;
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
pub use crate::v1_std_core::TextFile;
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum RenderTarget {
    Rust,
    Python,
    Go,
    Dag,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum ArtifactKind {
    ServiceBinary,
    Library,
    Frontend,
    GeneratedSupport,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Artifact {
    pub name: String,
    pub kind: ArtifactKind,
    pub target: RenderTarget,
    pub entry_modules: Rc<Vec<String>>,
    pub dependencies: Rc<Vec<String>>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum BoundaryKind {
    DirectCall,
    HttpJson,
    MessageQueue,
    Ffi,
    FileProtocol,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Boundary {
    pub from_artifact: String,
    pub to_artifact: String,
    pub kind: BoundaryKind,
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ArtifactPlan {
    pub artifacts: Rc<Vec<Rc<Artifact>>>,
    pub boundaries: Rc<Vec<Rc<Boundary>>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum PartitionRule {
    Explicit { artifacts: Rc<Vec<Rc<Artifact>>> },
}
impl PartitionRule {
    pub fn artifacts(&self) -> Rc<Vec<Rc<Artifact>>> {
        match self {
            PartitionRule::Explicit {
                artifacts: __val, ..
            } => __val.clone(),
        }
    }
}

pub fn plan_artifacts(rule: Rc<PartitionRule>) -> Rc<ArtifactPlan> {
    match (*rule).clone() {
        PartitionRule::Explicit {
            artifacts: arts, ..
        } => Rc::new(ArtifactPlan {
            artifacts: arts.clone(),
            boundaries: Rc::new(vec![]),
        }),
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ArtifactOutput {
    pub artifact: Rc<Artifact>,
    pub files: Rc<Vec<Rc<TextFile>>>,
}

pub fn default_artifact_plan(
    root_modules: Rc<Vec<String>>,
    target: RenderTarget,
) -> Rc<ArtifactPlan> {
    plan_artifacts(Rc::new(PartitionRule::Explicit {
        artifacts: Rc::new(vec![Rc::new(Artifact {
            name: "default".to_string(),
            kind: ArtifactKind::ServiceBinary,
            target: target,
            entry_modules: root_modules,
            dependencies: Rc::new(vec![]),
        })]),
    }))
}

pub type DagNodeId = String;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum DagInferredRecord {
    ResolvedRef {
        node: Box<DagNodeId>,
    },
    TypeVariableRef {
        id: String,
    },
    CompilerErrorRecord {
        message: String,
        span: Rc<SourceSpan>,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DagModuleRef {
    pub module: Box<DagNodeId>,
    pub items: Rc<Vec<DagNodeId>>,
    pub item_registry_keys: Rc<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DagDiagnosticRecord {
    pub severity: String,
    pub message: String,
    pub span: Rc<SourceSpan>,
    pub module_name: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DagArtifact {
    pub version: String,
    pub nodes: Rc<HashMap<DagNodeId, String>>,
    pub modules: Rc<Vec<String>>,
    pub item_registry_keys: Rc<Vec<String>>,
    pub diagnostics: Rc<Vec<String>>,
    pub files: Rc<Vec<String>>,
}
