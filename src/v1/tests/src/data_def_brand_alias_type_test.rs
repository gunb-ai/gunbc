//! E0425: a `data NAME: BrandAlias = lit` declaration emitted its accessor with the brand
//! alias PEELED to its base type. `data min_cargo_version: CargoToolVersionFloor = ">= 1.56"`
//! (where `type CargoToolVersionFloor = SemVerConstraint where brand(..)`) emitted
//! `pub fn min_cargo_version() -> SemVerConstraint` — but the emitting module imports
//! `CargoToolVersionFloor` (the declared type), NOT `SemVerConstraint` → `cannot find type
//! SemVerConstraint in this scope`.
//!
//! Root cause: `emit_data_def` rendered the accessor's return type from the VALUE's inferred
//! type (which peels the brand to its base) whenever the annotation leaf has no type args —
//! discarding the declared brand name. Faithful fix: for a named non-container leaf annotation,
//! emit the DECLARED type (preserves §3 brand identity AND it is the name the module imports).
//!
//! Single-module witness: both the brand alias and its base are emitted locally, so this pins
//! the brand-PRESERVATION (declared name, not peeled base). The cross-module import-resolution
//! half is confirmed by the `--emit-fresh` full build (E0425 → 0).

use crate::helpers::compile_dag_named;
use v1_compiler::v1_compiler_artifact::RenderTarget;

const FIXTURE: &str = concat!(
    "module branddata.fixture\n\n",
    "type SemConstraint = String\n\n",
    "type ToolFloor = SemConstraint where brand(\"ToolFloor\")\n\n",
    "data min_floor: ToolFloor = \">= 1.56\"\n"
);

fn emit_host() -> String {
    compile_dag_named("src/v1/data_def_brand_fixture.dag", FIXTURE, RenderTarget::Rust)
        .files
        .iter()
        .map(|f| f.content.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The emitted accessor signature line `pub fn min_floor() -> ...`.
fn accessor_sig(emitted: &str) -> String {
    let needle = "fn min_floor()";
    let start = emitted
        .find(needle)
        .unwrap_or_else(|| panic!("min_floor accessor not emitted:\n{emitted}"));
    let rest = &emitted[start..];
    let end = rest.find('\n').unwrap_or(rest.len());
    rest[..end].to_string()
}

#[test]
fn data_def_brand_alias_accessor_preserves_declared_type() {
    let emitted = emit_host();
    let sig = accessor_sig(&emitted);
    // The accessor return type must be the DECLARED brand alias `ToolFloor`, not the peeled
    // base `SemConstraint` (which, cross-module, would be an unimported `SemVerConstraint`).
    assert!(
        sig.contains("ToolFloor"),
        "data accessor must return the declared brand alias `ToolFloor`, got:\n{sig}"
    );
    assert!(
        !sig.contains("SemConstraint"),
        "data accessor peeled the brand alias to its base type, got:\n{sig}"
    );
}
