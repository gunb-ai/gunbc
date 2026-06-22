use self::CargoDepSource::*;
use self::CargoTarget::*;
use self::TestHarness::*;
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CargoPackage {
    pub name: String,
    pub version: String,
    pub edition: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum CargoDepSource {
    RegistryDep {
        version: String,
        features: Rc<Vec<String>>,
    },
    LocalPathDep {
        path: String,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CargoDependency {
    pub name: String,
    pub source: Rc<CargoDepSource>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum CargoTarget {
    Lib,
    Bin { name: String },
    CargoTest { name: String },
    Example { name: String },
    Bench { name: String },
}
impl CargoTarget {
    pub fn name(&self) -> String {
        match self {
            CargoTarget::Lib => panic!("no name on unit variant"),
            CargoTarget::Bin { name: __val, .. } => __val.clone(),
            CargoTarget::CargoTest { name: __val, .. } => __val.clone(),
            CargoTarget::Example { name: __val, .. } => __val.clone(),
            CargoTarget::Bench { name: __val, .. } => __val.clone(),
        }
    }
}

pub type CargoProfile = String;

pub fn canonical_profiles() -> Rc<Vec<String>> {
    thread_local! {
        static CACHED: Rc<Vec<String>> = {
            Rc::new(vec!["dev".to_string(), "release".to_string(), "test".to_string(), "bench".to_string()])
        };
    }
    CACHED.with(|c: &Rc<Vec<String>>| c.clone())
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CargoFeature {
    pub name: String,
    pub dependencies: Rc<Vec<String>>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(tag = "_variant")]
pub enum TestHarness {
    Harness,
    NoHarness,
}

pub fn default_edition() -> String {
    thread_local! {
        static CACHED: String = {
            "2021".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}

pub fn default_profile() -> String {
    thread_local! {
        static CACHED: String = {
            "dev".to_string()
        };
    }
    CACHED.with(|c: &String| c.clone())
}
