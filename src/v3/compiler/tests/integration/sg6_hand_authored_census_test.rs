// SG-6 hand-authored-Rust census for the driver + harness surfaces
// SG-6 owns:
//
//   - `src/v3/compiler/src/bin/` — regen drivers and the self-host
//     CI binary. The 4 per-lens regen bins collapsed to a single
//     `regen_lens` shim that reads `src/v3/compiler/regen.dag`.
//   - `src/v3/compiler/regen.dag` — lens registry. Every entry in
//     the registry is tagged with `LensRegistryEntry`, so the
//     `regen_lens` driver enumerates them structurally rather than
//     hard-coding per-lens paths.
//
// The tests below pin the post-cutover census. Any new
// hand-authored driver or mutation that silently grows the bin set
// fails this test before it can become a hidden authority — the
// SG-6 rule "every PR reduces the hand-authored Rust census;
// ratchet only down" can't be upheld without a machine check.
//
// Scope is deliberately SG-6-local. A full `src/v3/compiler/src`
// inventory belongs to SG-0 and is intentionally out of scope here.

use std::collections::BTreeSet;
use std::path::PathBuf;

use v3_compiler::dag::{Dag, FieldValue, LiteralBits, ValueBody};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn bin_dir() -> PathBuf {
    manifest_dir().join("src").join("bin")
}

fn bin_basenames() -> BTreeSet<String> {
    std::fs::read_dir(bin_dir())
        .expect("read src/v3/compiler/src/bin")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("rs"))
        .filter_map(|path| {
            path.file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .collect()
}

#[test]
fn sg6_bin_census_is_locked_to_three_shims() {
    let expected: BTreeSet<String> = ["regen_lens.rs", "regen_v3.rs", "self_host_fixed_point.rs"]
        .into_iter()
        .map(String::from)
        .collect();

    let actual = bin_basenames();

    assert_eq!(
        actual, expected,
        "SG-6 hand-authored bin census changed. The cutover collapsed the 4 \
         per-lens `regen_lens_*` bins into a single `regen_lens` driver that \
         reads `src/v3/compiler/regen.dag`. Adding a new bin re-introduces a \
         per-lens (or per-target) Rust driver — the SG-6 lane requires that \
         new regen / harness targets be added via the `.dag` registry instead. \
         If you believe the new bin is genuinely irreducible host-shim work, \
         update this ratchet in the same PR and document the reason in the \
         ROADMAP."
    );
}

#[test]
fn sg6_regen_dag_exposes_lens_registry_entries() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load `src/v3/compiler/regen.dag` cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    let entry_type_id = dag
        .declaration_by_name("LensRegistryEntry")
        .map(|decl| decl.id)
        .expect("regen.dag must declare `LensRegistryEntry`");

    let mut registry_names: Vec<String> = Vec::new();
    for decl in dag.declarations() {
        if decl.meta_tag != Some(entry_type_id) {
            continue;
        }
        let Some(ValueBody::Structural { fields }) = &decl.value_body else {
            panic!(
                "lens registry entry `{}` must carry a structural value body",
                decl.name.as_deref().unwrap_or("<anonymous>")
            );
        };
        let name = fields
            .iter()
            .find(|(label, _)| label == "name")
            .and_then(|(_, value)| match value {
                FieldValue::Literal(LiteralBits::String(s)) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| {
                panic!(
                    "lens registry entry `{}` is missing a String `name` field",
                    decl.name.as_deref().unwrap_or("<anonymous>")
                )
            });
        registry_names.push(name);
    }
    registry_names.sort();

    let expected = vec![
        "cost".to_string(),
        "cost_symbolic".to_string(),
        "provenance".to_string(),
        "structural_resolution".to_string(),
        "unused_parameters".to_string(),
    ];
    assert_eq!(
        registry_names, expected,
        "lens registry drift. `regen_lens` relies on these names to resolve \
         `--lens <name>`; the snapshot migration tests and the ROADMAP \
         reference them as well. If a lens is being added or retired, update \
         both `src/v3/compiler/regen.dag` and this test in the same PR."
    );
}
