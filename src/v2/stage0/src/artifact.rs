use crate::v2_core::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum RenderTarget {
    #[default]
    Rust,
    Python,
    Go,
    Dag,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum ArtifactKind {
    #[default]
    ServiceBinary,
    Library,
    Frontend,
    GeneratedSupport,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Artifact {
    pub name: String,
    pub kind: ArtifactKind,
    pub target: RenderTarget,
    pub entry_modules: Rc<Vec<String>>,
    pub dependencies: Rc<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash, Default)]
pub enum BoundaryKind {
    #[default]
    DirectCall,
    HttpJson,
    MessageQueue,
    Ffi,
    FileProtocol,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Boundary {
    pub from_artifact: String,
    pub to_artifact: String,
    pub kind: BoundaryKind,
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArtifactPlan {
    pub artifacts: Rc<Vec<Rc<Artifact>>>,
    pub boundaries: Rc<Vec<Rc<Boundary>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartitionRule {
    Explicit { artifacts: Rc<Vec<Rc<Artifact>>> },
}

impl Default for PartitionRule {
    fn default() -> Self {
        PartitionRule::Explicit { artifacts: Default::default() }
    }
}

impl PartitionRule {
    pub fn artifacts(&self) -> Rc<Vec<Rc<Artifact>>> {
        match self {
            PartitionRule::Explicit { artifacts, .. } => artifacts.clone()
        }
    }
}

pub fn plan_artifacts(rule: Rc<PartitionRule>) -> Rc<ArtifactPlan> {
    match rule.as_ref() {
    PartitionRule::Explicit { artifacts: arts, .. } => {
        Rc::new(ArtifactPlan { artifacts: arts.clone(), boundaries: Rc::new(Vec::new()) })
    }
}
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArtifactOutput {
    pub artifact: Rc<Artifact>,
    pub files: Rc<Vec<Rc<TextFile>>>,
}

pub fn default_artifact_plan(root_modules: Rc<Vec<String>>, target: RenderTarget) -> Rc<ArtifactPlan> {
    plan_artifacts(Rc::new(PartitionRule::Explicit { artifacts: Rc::new(vec!(Rc::new(Artifact { name: "default".to_string(), kind: ArtifactKind::ServiceBinary, target, entry_modules: root_modules, dependencies: Rc::new(Vec::new()) }))) }))
}

