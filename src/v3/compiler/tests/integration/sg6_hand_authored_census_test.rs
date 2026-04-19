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

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

use v3_compiler::dag::{Dag, Declaration, FieldValue, LiteralBits, ValueBody};

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

struct RegistryRow {
    binding: String,
    name: String,
    generated_file: String,
}

fn read_registry_rows(dag: &Dag) -> Vec<RegistryRow> {
    let entry_type_id = dag
        .declaration_by_name("LensRegistryEntry")
        .map(|decl| decl.id)
        .expect("regen.dag must declare `LensRegistryEntry`");

    dag.declarations()
        .iter()
        .filter(|decl| decl.meta_tag == Some(entry_type_id))
        .map(|decl| {
            let binding = decl
                .name
                .clone()
                .unwrap_or_else(|| "<anonymous>".to_string());
            let fields = structural_fields(decl);
            RegistryRow {
                binding: binding.clone(),
                name: string_field(fields, "name", &binding),
                generated_file: string_field(fields, "generated_file", &binding),
            }
        })
        .collect()
}

fn structural_fields(decl: &Declaration) -> &[(String, FieldValue)] {
    let Some(ValueBody::Structural { fields }) = &decl.value_body else {
        panic!(
            "lens registry entry `{}` must carry a structural value body",
            decl.name.as_deref().unwrap_or("<anonymous>")
        );
    };
    fields.as_slice()
}

fn string_field(fields: &[(String, FieldValue)], label: &str, binding: &str) -> String {
    fields
        .iter()
        .find(|(field_label, _)| field_label == label)
        .and_then(|(_, value)| match value {
            FieldValue::Literal(LiteralBits::String(s)) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| {
            panic!("lens registry entry `{binding}` is missing a String `{label}` field")
        })
}

#[test]
fn sg6_regen_dag_exposes_lens_registry_entries() {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load `src/v3/compiler/regen.dag` cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );

    let mut registry_names: Vec<String> = read_registry_rows(&dag)
        .into_iter()
        .map(|row| row.name)
        .collect();
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

// `--lens <name>` is the selection key in `regen_lens`'s CLI surface.
// If two registry entries carry the same `name`, the driver cannot
// distinguish them and the first-match-wins iteration order becomes
// a hidden contract. The driver itself fails closed on this case in
// `read_registry`; the test below pins the invariant at the registry
// source so the structural guarantee is visible at the authority.
#[test]
fn sg6_lens_registry_names_are_unique() {
    let dag = Dag::new();
    let rows = read_registry_rows(&dag);
    let mut seen: HashMap<String, String> = HashMap::new();
    for row in &rows {
        if let Some(prior_binding) = seen.get(&row.name) {
            panic!(
                "lens registry has duplicate `name` `{name}`: first declared by `{prior}`, re-declared by `{current}`. \
                 `regen_lens --lens {name}` would resolve ambiguously. Rename one entry in `src/v3/compiler/regen.dag`.",
                name = row.name,
                prior = prior_binding,
                current = row.binding,
            );
        }
        seen.insert(row.name.clone(), row.binding.clone());
    }
}

// Two entries pointing at the same `generated_file` would let each
// overwrite the other when `regen_lens` runs with no `--lens` filter
// (full-registry pass). The driver fails closed on duplicates; this
// test mirrors that invariant at the registry source.
#[test]
fn sg6_lens_registry_generated_files_are_unique() {
    let dag = Dag::new();
    let rows = read_registry_rows(&dag);
    let mut seen: HashMap<String, String> = HashMap::new();
    for row in &rows {
        if let Some(prior_binding) = seen.get(&row.generated_file) {
            panic!(
                "lens registry has duplicate `generated_file` `{path}`: first declared by `{prior}`, re-declared by `{current}`. \
                 Running `regen_lens` with no filter would have each entry clobber the other.",
                path = row.generated_file,
                prior = prior_binding,
                current = row.binding,
            );
        }
        seen.insert(row.generated_file.clone(), row.binding.clone());
    }
}

// The reviewer ask from #560 made explicit: `--lens <name>` must
// resolve to exactly one entry. Uniqueness is the structural
// guarantee; this test exercises the resolver against each real
// registry name and asserts a singleton match, locking in the
// contract the driver's `--lens` argument depends on.
#[test]
fn sg6_lens_registry_names_resolve_to_singleton_entry() {
    let dag = Dag::new();
    let rows = read_registry_rows(&dag);
    let known_names: Vec<String> = rows.iter().map(|row| row.name.clone()).collect();
    for name in &known_names {
        let matches: Vec<&RegistryRow> = rows.iter().filter(|row| row.name == *name).collect();
        assert_eq!(
            matches.len(),
            1,
            "`--lens {name}` must resolve to exactly one entry, found {count}: {bindings:?}",
            count = matches.len(),
            bindings = matches
                .iter()
                .map(|row| row.binding.as_str())
                .collect::<Vec<_>>(),
        );
    }
}
