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
// inventory belongs to Self-Generation-0 and is intentionally out of scope here.
//
// **Bounded-debt trigger: out-of-band registry copies.** `regen.dag`
// is now the primary authority for each lens's `(name, lens_file,
// generated_file)` triple, but two classes of downstream consumer
// still reach for the same paths through hardcoded bytes instead of
// resolving from the registry:
//
//   1. Every `m2_lens_*_migration_test.rs` under
//      `src/v3/compiler/tests/integration/` declares a local
//      `lens_path()` (e.g. `../lenses/complexity.dag`) and a
//      `checked_in_generated_module()` backed by `include_str!`.
//      The triple-ratchet below catches any divergence loudly, but
//      the path still lives in two places.
//
//   2. `src/v3/compiler/src/lib.rs` (inline lens host modules, e.g.
//      `lens_unused_parameters`, `lens_cost`)
//      embeds each `lens_<name>_generated.rs` via `include_str!`. That
//      call is compile-time and so is not a natural fit for a
//      runtime registry walk, but it still duplicates the
//      `generated_file` field structurally.
//
// Dissolution trigger (scoped as an SG-6 follow-up PR, not this
// one): add a shared `tests/common` helper that walks `regen.dag`
// and returns the absolute lens-source path for a given registry
// name, and re-point every migration test at that helper. Once the
// source-side path is sourced exclusively from the registry, the
// `lens_file` column in the triple ratchet collapses into a
// dependency on the same helper (i.e., SG-6's triple-ratchet ends
// up pinning only `name` plus `generated_file`, because `lens_file`
// is re-derived at read time instead of mirrored in the test).
//
// Until that follow-up lands, the triple ratchet below IS the
// bridge that keeps the duplication from drifting silently — it
// fails loudly the moment `regen.dag`, the migration-test
// `lens_path()` helpers, or the `include_str!` targets diverge.

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use v3_compiler::compile_to_dag;
use v3_compiler::dag::{Dag, Declaration, FieldValue, LiteralBits, ValueBody};
use v3_compiler::emit_rust::emit_rust_module;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn bin_dir() -> PathBuf {
    manifest_dir().join("src").join("bin")
}

fn workspace_root() -> PathBuf {
    manifest_dir().join("..").join("..").join("..")
}

fn rustfmt_stdout(source: &str, context: &str) -> String {
    let mut child = Command::new("rustfmt")
        .arg("--emit")
        .arg("stdout")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| panic!("spawn rustfmt for {context}: {err}"));
    child
        .stdin
        .as_mut()
        .expect("rustfmt stdin")
        .write_all(source.as_bytes())
        .unwrap_or_else(|err| panic!("write source to rustfmt for {context}: {err}"));
    let output = child
        .wait_with_output()
        .unwrap_or_else(|err| panic!("wait for rustfmt for {context}: {err}"));
    assert!(output.status.success(), "rustfmt failed on {context}");
    String::from_utf8(output.stdout).expect("rustfmt output should be utf-8")
}

/// Enumerate every entry under `src/v3/compiler/src/bin/` that Cargo's
/// auto-discovery would promote to a bin target. Cargo picks up two
/// shapes out of that directory:
///
///   1. `src/bin/<name>.rs`         — flat single-file bin, yields `<name>.rs`
///   2. `src/bin/<name>/main.rs`    — directory-form bin,    yields `<name>/`
///
/// SG-6 pins both: a flat `.rs` file outside the expected set grows
/// the hand-authored bin census, and — more subtly — a new directory
/// under `src/bin/` with its own `main.rs` is silently a new bin even
/// though it does not create a top-level `.rs` file. Without
/// detecting the directory form the ratchet leaks and a new driver
/// can land as `src/bin/foo/main.rs` without tripping the census.
fn bin_basenames() -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for entry in std::fs::read_dir(bin_dir())
        .expect("read src/v3/compiler/src/bin")
        .filter_map(|entry| entry.ok())
    {
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        let file_type = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        if file_type.is_file() && path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.insert(file_name.to_string());
        } else if file_type.is_dir() && path.join("main.rs").is_file() {
            // Cargo names the resulting bin after the directory itself
            // (not `main.rs`), so store the directory label with a
            // trailing `/` to keep the directory form distinct from a
            // flat `<name>.rs` bin in the expected set and in error
            // output.
            out.insert(format!("{file_name}/"));
        }
    }
    out
}

#[test]
fn self_gen6_bin_census_is_locked_to_expected_regen_shims() {
    // SG-1 receipts `regen_tokenize.rs` here. The tokenizer cutover uses
    // `src/v3/compiler/tokenize.dag` as lexical authority and
    // `regen_tokenize.rs` as the host driver that projects it into
    // `src/v3/compiler/src/tokenize_generated.rs`. The driver does not
    // fit the lens-registry shape that `regen_lens` enumerates (its
    // input is tokenizer spec rows — keyword / punctuation / escape
    // tables — not `LensRegistryEntry`-tagged lens declarations), so
    // for SG-1 it lands as a parallel shim rather than a registry entry.
    //
    // Dissolution trigger (SG-2+): unify `regen_tokenize` with the
    // registry-driven pattern — either extend `regen.dag` to carry a
    // tokenizer-registry shape that `regen_lens` can dispatch on, or
    // introduce a generalized "producer registry" that both lenses and
    // the tokenizer share. Either path retires `regen_tokenize.rs` and
    // shrinks the parallel shim set.
    let expected: BTreeSet<String> = [
        // F.14 / T-PB-B: `ExecuteCommand` logical child for
        // `tests/dag/boundary_emit_gates.template.dag`. Calls
        // `v3_compiler::boundary_emit_gates::check_*`; exits 0/1 (PR #792 host boundary).
        // **P5 dissolution:** delete when ROADMAP.md **PB-Runtime-External-Toolchain-TestClaims**
        // / **T-PB-B** `pb_rust_tests_outside_residual_zero` retires remaining `tests/boundary/*.rs`
        // host shims and this bin is no longer referenced by any `.dag` `ExecuteCommand` claim.
        "boundary_emit_gates.rs",
        // R1C-E (T-Emit `.dag` `TestClaim` wrappers): irreducible host-shim
        // for the `ExecuteCommand` logical child that the `.dag` claim
        // invokes. Calls into `v3_compiler::r1c_e_gates::check_*` (single
        // source of truth) and exits 0/1 — the bounded host-spawn boundary
        // PR #792 / `TestPredicate::ExecuteCommand` is built around. Cannot
        // be expressed via the `regen.dag` registry shape (the registry's
        // job is `Dag → emitted file`; this bin's job is `process exit
        // code` for a `.dag` predicate). Dissolution trigger: when R1
        // closes and the `.dag` runner can express in-process compilation
        // checks without a host child, the wrappers + bin retire together
        // (R1 Closure dispatch on issue #973). Documented in ROADMAP T-Emit
        // / R1C-E lane row.
        "r1c_e_emit_gates.rs",
        "regen_bootstrap.rs",
        "regen_parse.rs",
        // SG-2c-1 grammar-tables prototype: `regen_parse_tables` projects
        // `src/v3/compiler/parse_tables.dag` into
        // `src/v3/compiler/src/parse_tables_generated.rs`. Same host-shim
        // posture as `regen_parse` / `regen_tokenize`: this shim
        // dissolves into a unified producer registry when SG-2c proper
        // lands (blocked on recursive list-body emission — see
        // `parse_tables.dag` header).
        "regen_parse_tables.rs",
        "regen_tokenize.rs",
        "regen_v3.rs",
        "self_host_fixed_point.rs",
    ]
    .into_iter()
    .map(String::from)
    .collect();

    let actual = bin_basenames();

    assert_eq!(
        actual, expected,
        "SG-6 hand-authored bin census changed. The census is \
         `boundary_emit_gates` (F.14 / T-PB-B class-5 boundary `.dag` `ExecuteCommand` child; \
         dissolves with PB-Runtime-External-Toolchain-TestClaims / zero-floor boundary migration), \
         `r1c_e_emit_gates` (R1C-E T-Emit `.dag` wrapper logical child; \
         issue #973), `regen_parse` (reads `src/v3/std/parse_surface.dag` for Surface \
         carriers), `regen_tokenize` (reads `src/v3/compiler/tokenize.dag`), \
         `regen_v3`, and `self_host_fixed_point`. The `regen_lens` binary is a \
         thin `[[bin]]` outside `src/bin/` (R3 gate #7); its driver lives in \
         `regen_lens_driver.rs`. Adding a new bin re-introduces a \
         per-lens (or per-target) Rust driver — the SG-6 lane requires that \
         new regen / harness targets be added via a `.dag` registry instead. \
         Both `src/bin/<name>.rs` (flat-file bins; basename reported) and \
         `src/bin/<name>/main.rs` (directory-form bins; reported as `<name>/`) \
         are counted, because Cargo's auto-discovery promotes both shapes. \
         If you believe the new bin is genuinely irreducible host-shim work, \
         update this ratchet in the same PR and document the reason in the \
         ROADMAP."
    );
}

struct RegistryRow {
    binding: String,
    name: String,
    lens_file: String,
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
                lens_file: string_field(fields, "lens_file", &binding),
                generated_file: string_field(fields, "generated_file", &binding),
            }
        })
        .collect()
}

fn load_registry() -> (Dag, Vec<RegistryRow>) {
    let dag = Dag::new();
    assert!(
        dag.diagnostics().is_empty(),
        "bootstrap should load `src/v3/compiler/regen.dag` cleanly, got {:?}",
        dag.diagnostics().iter().collect::<Vec<_>>()
    );
    let rows = read_registry_rows(&dag);
    (dag, rows)
}

fn registry_row<'a>(rows: &'a [RegistryRow], name: &str) -> &'a RegistryRow {
    rows.iter()
        .find(|row| row.name == name)
        .unwrap_or_else(|| panic!("registry row for `{name}`"))
}

fn generated_header(lens_file: &str) -> String {
    format!(
        "// AUTO-GENERATED from `{lens_file}` via\n\
         // `emit_rust_module`. Regenerate instead of hand-editing.\n\n"
    )
}

fn emit_registry_module(row: &RegistryRow) -> String {
    let lens_path = workspace_root().join(&row.lens_file);
    let lens_source = std::fs::read_to_string(&lens_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", lens_path.display()));
    let dag = compile_to_dag(&lens_source, lens_path.to_string_lossy().as_ref())
        .unwrap_or_else(|diag| panic!("compiled {}: {diag:?}", lens_path.display()));
    assert!(
        dag.diagnostics().is_empty(),
        "{} should compile cleanly, got {:?}",
        row.lens_file,
        dag.diagnostics()
    );
    let raw = emit_rust_module(&dag)
        .unwrap_or_else(|err| panic!("emit compiled module for {}: {err:?}", row.name));
    rustfmt_stdout(
        &format!("{}{raw}", generated_header(&row.lens_file)),
        &format!("generated module `{}`", row.name),
    )
}

fn checked_in_generated_module(row: &RegistryRow) -> String {
    let out_path = workspace_root().join(&row.generated_file);
    std::fs::read_to_string(&out_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", out_path.display()))
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

// Full-triple ratchet over `src/v3/compiler/regen.dag`. `name`
// alone pins `--lens` selector identity, but leaving `lens_file`
// out-of-band would let the source path for a lens drift
// independently of the per-lens migration tests' hard-coded
// `lens_path()` helpers (e.g. renaming `complexity.dag` or moving
// a lens across directories). Asserting all three fields keeps
// every registry-visible path under one structural ratchet, so
// any drift forces a paired edit across `regen.dag`, this test,
// and the referring migration test.
#[test]
fn self_gen6_regen_dag_registry_triples_are_pinned() {
    let (_dag, mut rows) = load_registry();
    rows.sort_by(|a, b| a.name.cmp(&b.name));

    let expected: Vec<(&str, &str, &str)> = vec![
        (
            "cost",
            "src/v3/lenses/complexity.dag",
            "src/v3/compiler/src/complexity_lens_generated.rs",
        ),
        (
            "cost_symbolic",
            "src/v3/lenses/cost.dag",
            "src/v3/compiler/src/cost_symbolic_lens_generated.rs",
        ),
        (
            "cost_target_realization",
            "src/v3/lenses/cost_target_realization.dag",
            "src/v3/compiler/src/lens_cost_target_realization_generated.rs",
        ),
        (
            "effect_enumeration",
            "src/v3/lenses/effect_enumeration.dag",
            "src/v3/compiler/src/lens_effect_enumeration_generated.rs",
        ),
        (
            "infer_helpers",
            "src/v3/lenses/infer_helpers.dag",
            "src/v3/compiler/src/infer_helpers_generated.rs",
        ),
        (
            "lower_helpers",
            "src/v3/lenses/lower_helpers.dag",
            "src/v3/compiler/src/lower_helpers_generated.rs",
        ),
        (
            "parallelism",
            "src/v3/lenses/parallelism.dag",
            "src/v3/compiler/src/lens_parallelism_generated.rs",
        ),
        (
            "provenance",
            "src/v3/lenses/provenance.dag",
            "src/v3/compiler/src/lens_provenance_generated.rs",
        ),
        (
            "structural_resolution",
            "src/v3/lenses/structural_resolution.dag",
            "src/v3/compiler/src/lens_structural_resolution_generated.rs",
        ),
        (
            "unused_parameters",
            "src/v3/lenses/unused_parameters.dag",
            "src/v3/compiler/src/lens_unused_parameters_generated.rs",
        ),
        (
            "variant_payload",
            "src/v3/lenses/variant_payload.dag",
            "src/v3/compiler/src/variant_payload_generated.rs",
        ),
    ];

    let actual: Vec<(&str, &str, &str)> = rows
        .iter()
        .map(|row| {
            (
                row.name.as_str(),
                row.lens_file.as_str(),
                row.generated_file.as_str(),
            )
        })
        .collect();

    assert_eq!(
        actual, expected,
        "lens registry triple drift. Every `(name, lens_file, \
         generated_file)` tuple is pinned so changing the source path \
         or output path of a lens has to land in the same PR as the \
         registry edit and the matching migration-test update. If a \
         lens is being added, renamed, relocated, or retired, update \
         `src/v3/compiler/regen.dag`, this snapshot, the corresponding \
         `m2_lens_*_migration_test.rs` hard-coded `lens_path()`, and \
         any `include_str!` in `src/v3/compiler/src/lib.rs` in the \
         same commit."
    );
}

// `--lens <name>` is the selection key in `regen_lens`'s CLI surface.
// If two registry entries carry the same `name`, the driver cannot
// distinguish them and the first-match-wins iteration order becomes
// a hidden contract. The driver itself fails closed on this case in
// `read_registry`; the test below pins the invariant at the registry
// source so the structural guarantee is visible at the authority.
#[test]
fn self_gen6_lens_registry_names_are_unique() {
    let (_dag, rows) = load_registry();
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
fn self_gen6_lens_registry_generated_files_are_unique() {
    let (_dag, rows) = load_registry();
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
fn self_gen6_lens_registry_names_resolve_to_singleton_entry() {
    let (_dag, rows) = load_registry();
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

// Band-C cementing: `cementing_dispatch.dag` is exercised through
// `t_pb_b_1_dag_runner_test::cementing_dispatch_suite_passes_through_runner`. SG-6 owns
// integration harness glue checks; this ratchet lives here so CI stays honest when the
// runner wiring drifts.
#[test]
fn self_gen6_cementing_dispatch_dag_is_wired_in_runner_receipt() {
    let path = manifest_dir()
        .join("tests")
        .join("integration")
        .join("t_pb_b_1_dag_runner_test.rs");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    assert!(
        text.contains("cementing_dispatch.dag"),
        "{} must reference `cementing_dispatch.dag` so the Band-C dispatch suite stays wired through the PB-B-1 runner receipt.",
        path.display(),
    );
    assert!(
        text.contains("cementing_dispatch_suite"),
        "{} must reference suite name `cementing_dispatch_suite` alongside the `.dag` include.",
        path.display(),
    );
}

#[test]
fn self_gen6_infer_helpers_generated_module_matches_checked_in_snapshot() {
    let (_dag, rows) = load_registry();
    let row = registry_row(&rows, "infer_helpers");
    let fresh = emit_registry_module(row);
    assert_eq!(
        fresh.trim(),
        checked_in_generated_module(row).trim(),
        "checked-in generated module is stale; regenerate {} from {} via `cargo run -p v3-compiler --bin regen_lens -- --lens {}`",
        row.generated_file,
        row.lens_file,
        row.name,
    );
}

#[test]
#[ignore]
fn self_gen6_emit_infer_helpers_snapshot() {
    let (_dag, rows) = load_registry();
    let row = registry_row(&rows, "infer_helpers");
    let fresh = emit_registry_module(row);
    let out_path = workspace_root().join(&row.generated_file);
    std::fs::write(&out_path, fresh)
        .unwrap_or_else(|err| panic!("write {}: {err}", out_path.display()));
    println!("wrote {}", out_path.display());
}

#[test]
fn self_gen6_lower_helpers_generated_module_matches_checked_in_snapshot() {
    let (_dag, rows) = load_registry();
    let row = registry_row(&rows, "lower_helpers");
    let fresh = emit_registry_module(row);
    assert_eq!(
        fresh.trim(),
        checked_in_generated_module(row).trim(),
        "checked-in generated module is stale; regenerate {} from {} via `cargo run -p v3-compiler --bin regen_lens -- --lens {}`",
        row.generated_file,
        row.lens_file,
        row.name,
    );
}

#[test]
#[ignore]
fn self_gen6_emit_lower_helpers_snapshot() {
    let (_dag, rows) = load_registry();
    let row = registry_row(&rows, "lower_helpers");
    let fresh = emit_registry_module(row);
    let out_path = workspace_root().join(&row.generated_file);
    std::fs::write(&out_path, fresh)
        .unwrap_or_else(|err| panic!("write {}: {err}", out_path.display()));
    println!("wrote {}", out_path.display());
}

// Director/Codex follow-up on #560: prove the real CLI path works, not
// just the structural registry ratchets. This smoke test runs the
// built `regen_lens` binary against a single concrete registry entry
// and asserts three things:
//   1. `--lens <name>` exits successfully,
//   2. stdout reports the expected generated target path, and
//   3. the checked-in generated file is unchanged after the run.
//
// If the file bytes change, restore the original snapshot before
// failing so a local red test does not leave the worktree dirty.
budgeted_test! {
    15_000,
    self_gen6_regen_lens_cli_smoke_regenerates_named_entry_without_drift,
    {
        let (_dag, rows) = load_registry();
        let row = registry_row(&rows, "cost");

        let out_path = workspace_root().join(&row.generated_file);
        let before = std::fs::read(&out_path).expect("read checked-in generated file");

        let output = Command::new(env!("CARGO_BIN_EXE_regen_lens"))
            .current_dir(manifest_dir())
            .arg("--lens")
            .arg(&row.name)
            .output()
            .expect("run regen_lens binary");

        assert!(
            output.status.success(),
            "regen_lens --lens {} failed:\nstdout:\n{}\nstderr:\n{}",
            row.name,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );

        let stdout = String::from_utf8(output.stdout).expect("regen_lens stdout should be utf-8");
        assert_eq!(
            stdout.trim(),
            format!("wrote {}", out_path.display()),
            "`regen_lens --lens {}` should report the single generated target it rewrote",
            row.name,
        );

        let after = std::fs::read(&out_path).expect("read regenerated file");
        if after != before {
            std::fs::write(&out_path, &before)
                .expect("restore checked-in generated file after smoke drift");
        }
        assert_eq!(
            after, before,
            "`regen_lens --lens {}` changed `{}`. The smoke test expects the CLI path to be clean against the checked-in snapshot; if this fails, regenerate in the same PR that updates the snapshot.",
            row.name,
            row.generated_file,
        );
    }
}
