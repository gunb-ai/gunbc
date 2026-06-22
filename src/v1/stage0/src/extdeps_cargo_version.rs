pub use crate::extdeps_version_semver::{SemVerConstraint, SemVerIdentity};
use crate::v1_rt;
use crate::v1_rt::Witness;
use crate::v1_rt::Witness::{Holds, Violates};
use crate::NonEmptyBTreeSet;
use crate::NonEmptyVec;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::rc::Rc;

pub type CargoPackageVersion = SemVerIdentity;

pub type CargoVersionRequirement = SemVerConstraint;

pub type CargoToolVersionFloor = SemVerConstraint;

pub fn default_stage0_package_version() -> Rc<FreeMonoid<Nat>> {
    thread_local! {
        static CACHED: Rc<FreeMonoid<Nat>> = {
            "0.1.0".to_string()
        };
    }
    CACHED.with(|c: &Rc<FreeMonoid<Nat>>| c.clone())
}

pub fn render_cargo_package_version_field(version: CargoPackageVersion) -> Rc<FreeMonoid<Nat>> {
    v1_rt::concat(
        v1_rt::concat("version = \"".to_string(), version),
        "\"".to_string(),
    )
}

pub fn render_cargo_version_requirement_toml(req: CargoVersionRequirement) -> Rc<FreeMonoid<Nat>> {
    v1_rt::concat(v1_rt::concat("\"".to_string(), req), "\"".to_string())
}

pub fn render_cargo_package_header_prefix(name: Rc<FreeMonoid<Nat>>) -> Rc<FreeMonoid<Nat>> {
    v1_rt::concat(
        v1_rt::concat(
            v1_rt::concat("[package]\nname = \"".to_string(), name),
            "\"\n".to_string(),
        ),
        render_cargo_package_version_field(default_stage0_package_version()),
    )
}
