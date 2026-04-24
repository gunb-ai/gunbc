// Build script: enumerate every `*.dag` file in the v3-only staged
// directories and generate Rust arrays of `(path, content)` pairs
// the bootstrap loader consumes.
//
// Generated arrays:
//   - `STAGED_FILES`          for `src/v3/std/*.dag`
//   - `V3_SPECS`              for `src/v3/spec/*.dag`
//   - `COMPILER_FILES`        for `src/v3/compiler/*.dag` (except `tokenize.dag`; see below)
//   - `LENS_BOOTSTRAP_FILES`  for `src/v3/lenses/bootstrap/*.dag` (bootstrap-only user lenses;
//     regen lenses stay under `src/v3/lenses/*.dag` and are not auto-bundled)
//
// Adding a new staged std/spec/compiler file (or a `*.dag` under
// `src/v3/lenses/bootstrap/`) becomes a pure file-system change — no Rust
// edits to `bootstrap.rs`, no fixture-array maintenance, no skip-list drift.
//
// **Why a build script.** The pre-unwind shape used hardcoded
// `const RUST_DAG: &str = include_str!("../../spec/rust.dag");`
// constants in `bootstrap.rs`. PR #445 review flagged this as a
// duplicate-authority bug: the on-disk spec files and the Rust
// fixture array are two parallel representations of the same set,
// and adding a new target requires editing both. This script
// removes the parallel representation by deriving the arrays from
// the staged directories.
//
// **Load order.** The v3 lowerer doesn't strictly enforce import
// resolution at M1(3), but later spec files that reference
// declarations from earlier ones (e.g. rust.dag importing v3_l1's
// `Bind` marker) must follow them in load order. The script
// hardcodes one rule: `v3_l1.dag` (the substrate marker file)
// loads first; everything else follows in alphabetical order. The
// rule is the only special case because v3_l1 is the file every
// per-target spec depends on. If a future spec file introduces a
// dependency that needs sorting, this script grows a topological
// pass; until then, the simple rule is sufficient.
//
// **Output**: writes three Rust files:
//
//   pub static STAGED_FILES: &[(&str, &str)] = &[
//       ("src/v3/std/list.dag", include_str!("...")),
//   ];
//
//   pub static V3_SPECS: &[(&str, &str)] = &[
//       ("src/v3/spec/v3_l1.dag", include_str!("...")),
//       ("src/v3/spec/rust.dag",  include_str!("...")),
//   ];
//
//   pub static COMPILER_FILES: &[(&str, &str)] = &[
//       ("src/v3/compiler/pipeline.dag", include_str!("...")),
//   ];
//   `tokenize.dag` is intentionally omitted: tokenizer authority for `regen_tokenize`
//   must not be folded into the bootstrap Dag (duplicate declarations when
//   `compile_to_dag` parses that authority standalone on top of `Dag::new()`'s
//   bootstrapped clone). `src/v3/std/parse_surface.dag` stays in the bundle so the
//   production substrate matches the staged std surface; `regen_parse` and staging
//   tests compile it via `compile_parse_surface_std_authority_dag` in
//   `src/v3/compiler/src/lib.rs`, which boots from a clone that omits that staged
//   fixture to avoid the duplicate-name path.
//
// `bootstrap.rs` uses `include!(concat!(env!("OUT_DIR"), ...))` to
// pull the arrays in. The `include_str!` calls inside the generated
// files resolve at compile time against absolute paths, so the
// runtime is still hermetic — no filesystem access at `Dag::new()`
// time.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_dag_entries(dir: &Path, prioritized: &[&str]) -> Vec<PathBuf> {
    collect_dag_entries_impl(dir, prioritized, false)
}

fn collect_dag_entries_recursive(dir: &Path, prioritized: &[&str]) -> Vec<PathBuf> {
    collect_dag_entries_impl(dir, prioritized, true)
}

fn collect_dag_entries_impl(dir: &Path, prioritized: &[&str], recursive: bool) -> Vec<PathBuf> {
    let mut entries = Vec::new();
    let mut dirs = vec![dir.to_path_buf()];

    while let Some(current_dir) = dirs.pop() {
        let mut child_dirs = Vec::new();
        for entry in fs::read_dir(&current_dir)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", current_dir.display(), e))
        {
            let entry = entry.unwrap_or_else(|e| {
                panic!(
                    "failed to read dir entry in {}: {}",
                    current_dir.display(),
                    e
                )
            });
            let path = entry.path();
            if path.is_dir() {
                if recursive {
                    child_dirs.push(path);
                }
                continue;
            }
            if path.extension().and_then(|s| s.to_str()) == Some("dag") {
                entries.push(path);
            }
        }
        child_dirs.sort();
        for child_dir in child_dirs.into_iter().rev() {
            dirs.push(child_dir);
        }
    }

    entries.sort_by(|a, b| {
        let priority_key = |path: &Path| -> (usize, String, String) {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let rank = prioritized
                .iter()
                .position(|candidate| *candidate == name)
                .unwrap_or(prioritized.len());
            (
                rank,
                name.to_string(),
                path.strip_prefix(dir).unwrap_or(path).display().to_string(),
            )
        };
        priority_key(a).cmp(&priority_key(b))
    });

    entries
}

fn generate_static(
    static_name: &str,
    dir_label: &str,
    display_prefix: &str,
    base_dir: &Path,
    entries: &[PathBuf],
) -> String {
    let mut generated = format!(
        "// AUTO-GENERATED by build.rs. Do not edit.\n\
         //\n\
         // Files enumerated from {dir_label}/*.dag. Adding a new\n\
         // staged file is a pure file-system change.\n\
         pub static {static_name}: &[(&str, &str)] = &[\n"
    );
    for path in entries {
        let relative_path = path
            .strip_prefix(base_dir)
            .unwrap_or_else(|_| panic!("{} is not under {}", path.display(), base_dir.display()));
        let display_path = format!("{display_prefix}/{}", relative_path.display());
        let abs_path = path
            .canonicalize()
            .unwrap_or_else(|e| panic!("failed to canonicalize {}: {}", path.display(), e));
        generated.push_str(&format!(
            "    (\"{}\", include_str!(\"{}\")),\n",
            display_path,
            abs_path.display()
        ));
    }
    generated.push_str("];\n");
    generated
}

fn main() {
    // Resolve the staged directories relative to CARGO_MANIFEST_DIR
    // (which points at src/v3/compiler/). std/ and spec/ live one
    // level up; compiler-staged files live in the manifest dir itself.
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by Cargo");
    let manifest_path = PathBuf::from(&manifest_dir);
    let v3_dir = manifest_path.parent().expect("compiler dir has parent");
    let src_dir = v3_dir.parent().expect("v3 dir has parent");
    let repo_root = src_dir.parent().expect("src dir has parent");
    let std_dir = v3_dir.join("std");
    let spec_dir = v3_dir.join("spec");
    let compiler_dir = manifest_path.clone();
    let extdeps_dir = repo_root.join("dsl").join("extdeps");
    let gunbc_dir = repo_root.join("dsl").join("gunbc");

    // Tell Cargo to re-run the script if any staged std/spec/compiler file
    // changes. Without this, adding a new file wouldn't trigger a
    // rebuild and bootstrap would silently miss it.
    println!("cargo:rerun-if-changed={}", std_dir.display());
    println!("cargo:rerun-if-changed={}", spec_dir.display());
    println!("cargo:rerun-if-changed={}", compiler_dir.display());
    println!("cargo:rerun-if-changed={}", extdeps_dir.display());
    println!("cargo:rerun-if-changed={}", gunbc_dir.display());

    let lens_bootstrap_dir = v3_dir.join("lenses").join("bootstrap");
    println!("cargo:rerun-if-changed={}", lens_bootstrap_dir.display());

    // Structural-recursion termination analysis walks a recursing
    // argument back to its declared Disj connective (see
    // `structural_binding_info_for_variant` in `lower.rs`). The walk
    // only succeeds after the declaring file has been phase-2 lowered,
    // so `std/list.dag` (declares `List<element> = Empty | Cons {...}`)
    // and `std/substrate.dag` (declares `Behavior = Value | Transform
    // | Branch | Loop | Bind`) must land before any sibling std file
    // that recursively descends over those variants. Without this
    // priority list, alphabetical order puts `algebra.dag` and
    // `dimensions.dag` ahead of `list.dag`/`substrate.dag`, and their
    // recursive helpers fail termination against placeholder
    // connectives.
    // `substrate_minimal` + `effects` before full `substrate` so `substrate.dag`
    // can import `WorkflowEffect` for reflected `lane2_workflow` without a
    // module cycle (`effects` still needs `PortId` / list primitives first).
    let staged_entries = {
        let staged_entries = collect_dag_entries(
            &std_dir,
            &[
                "list.dag",
                "substrate_minimal.dag",
                "effects.dag",
                "substrate.dag",
            ],
        );
        // `r1_gates.dag` depends on `std.verification` (`TestClaim` / `TestPredicate`
        // variants). Lexicographic staged order would load it before
        // `verification.dag`. Move it to the tail of the staged bundle here so
        // bootstrap load order stays a single `build.rs` authority (no parallel
        // reorder in `bootstrap.rs`).
        const R1_GATES_STAGED: &str = "r1_gates.dag";
        let (mut staged_middle, r1_tail): (Vec<PathBuf>, Vec<PathBuf>) =
            staged_entries.into_iter().partition(|p| {
                p.file_name()
                    .and_then(|s| s.to_str())
                    .map(|n| n != R1_GATES_STAGED)
                    .unwrap_or(true)
            });
        staged_middle.extend(r1_tail);
        staged_middle
    };
    let spec_entries = collect_dag_entries(&spec_dir, &["v3_l1.dag"]);
    let mut compiler_entries = collect_dag_entries(&compiler_dir, &["pipeline.dag"]);
    // `tokenize.dag` is SG-1 tokenizer authority consumed by `regen_tokenize`; it is
    // stripped from the runtime bootstrap bundle — see COMPILER_FILES header.
    // `parse_tables.dag` is SG-2c-1 grammar-tables authority consumed by
    // `regen_parse_tables`; same exclusion rationale — `compile_to_dag` parses it
    // standalone at regen time, so bundling it into the bootstrap Dag would create
    // duplicate-declaration diagnostics.
    compiler_entries.retain(|p| {
        p.file_name()
            .and_then(|s| s.to_str())
            .map(|n| n != "tokenize.dag" && n != "parse_tables.dag")
            .unwrap_or(true)
    });
    let extdeps_entries = collect_dag_entries_recursive(&extdeps_dir, &[]);
    let gunbc_entries = collect_dag_entries_recursive(&gunbc_dir, &[]);
    let staged_generated = generate_static(
        "STAGED_FILES",
        "src/v3/std",
        "src/v3/std",
        &std_dir,
        &staged_entries,
    );
    let specs_generated = generate_static(
        "V3_SPECS",
        "src/v3/spec",
        "src/v3/spec",
        &spec_dir,
        &spec_entries,
    );
    let compiler_generated = generate_static(
        "COMPILER_FILES",
        "src/v3/compiler",
        "src/v3/compiler",
        &compiler_dir,
        &compiler_entries,
    );
    let extdeps_generated = generate_static(
        "EXTDEPS_FILES",
        "dsl/extdeps",
        "dsl/extdeps",
        &extdeps_dir,
        &extdeps_entries,
    );
    let gunbc_generated = generate_static(
        "GUNBC_FILES",
        "dsl/gunbc",
        "dsl/gunbc",
        &gunbc_dir,
        &gunbc_entries,
    );
    let lens_bootstrap_entries = collect_dag_entries(&lens_bootstrap_dir, &[]);
    let lens_bootstrap_generated = generate_static(
        "LENS_BOOTSTRAP_FILES",
        "src/v3/lenses/bootstrap",
        "src/v3/lenses/bootstrap",
        &lens_bootstrap_dir,
        &lens_bootstrap_entries,
    );

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR must be set by Cargo");
    let out_dir = Path::new(&out_dir);
    let staged_out = out_dir.join("v3_staged_files.rs");
    fs::write(&staged_out, staged_generated)
        .unwrap_or_else(|e| panic!("failed to write {}: {}", staged_out.display(), e));
    let specs_out = out_dir.join("v3_specs.rs");
    fs::write(&specs_out, specs_generated)
        .unwrap_or_else(|e| panic!("failed to write {}: {}", specs_out.display(), e));
    let compiler_out = out_dir.join("v3_compiler_files.rs");
    fs::write(&compiler_out, compiler_generated)
        .unwrap_or_else(|e| panic!("failed to write {}: {}", compiler_out.display(), e));
    // SG-0 producer-owned generated manifest. The census at
    // `tests/integration/sg0_census_test.rs` reads this list as the
    // sole authority for "which .rs files under src/v3/compiler are
    // generated." Every codegen driver or checked-in generator output
    // must be represented here so generated Rust never drifts back into
    // the hand-authored census.
    //
    // The manifest lives only in OUT_DIR — it is ephemeral,
    // reconstructed every build from this list. A hand-edited
    // manifest file is clobbered on the next `cargo build`; the
    // spoof surface collapses to this `REGEN_OUTPUTS` literal,
    // reviewed alongside any real producer change.
    //
    // Paths are workspace-relative and sorted for determinism.
    const REGEN_OUTPUTS: &[&str] = &[
        "src/v3/compiler/src/bootstrap_generated.rs",
        "src/v3/compiler/src/bootstrap_generated_without_parse_surface.rs",
        "src/v3/compiler/src/bootstrap_std_generated.rs",
        // SG-5 substrate / runtime-mirror projections, generated from
        // `src/v3/std/substrate.dag` + `src/v3/std/parse_surface.dag`
        // and consumed by hand-authored Rust via `include!(...)`.
        // Registering them here keeps SG-0's census honest: they carry
        // `// AUTO-GENERATED` headers AND are listed as producer-owned
        // outputs, so the content header alone never masquerades as
        // generated (SG-0's `sg0_generated_partition_is_producer_owned`
        // invariant).
        "src/v3/compiler/src/dag_branch_generated.rs",
        "src/v3/compiler/src/dag_cluster_generated.rs",
        "src/v3/compiler/src/dag_cost_generated.rs",
        "src/v3/compiler/src/dag_scalar_generated.rs",
        "src/v3/compiler/src/diagnostics_generated.rs",
        "src/v3/compiler/src/infer_helpers_generated.rs",
        "src/v3/compiler/src/lens_cost_generated.rs",
        "src/v3/compiler/src/lens_cost_symbolic_generated.rs",
        "src/v3/compiler/src/lens_provenance_generated.rs",
        "src/v3/compiler/src/lens_structural_resolution_generated.rs",
        "src/v3/compiler/src/lens_unused_parameters_generated.rs",
        "src/v3/compiler/src/lower_generated.rs",
        "src/v3/compiler/src/lower_helpers_generated.rs",
        "src/v3/compiler/src/operators_generated.rs",
        "src/v3/compiler/src/parse_generated.rs",
        "src/v3/compiler/src/parse_surface_generated.rs",
        "src/v3/compiler/src/parse_tables_generated.rs",
        "src/v3/compiler/src/serialize_generated.rs",
        "src/v3/compiler/src/tokenize_generated.rs",
        "src/v3/compiler/src/types_generated.rs",
        "src/v3/compiler/src/variant_payload_generated.rs",
    ];
    let mut manifest = String::from(
        "// AUTO-GENERATED by build.rs. Do not edit.\n\
         //\n\
         // Paths (workspace-relative) of every .rs file under\n\
         // src/v3/compiler that is produced by a codegen authority.\n\
         pub static GENERATED_FILES: &[&str] = &[\n",
    );
    for path in REGEN_OUTPUTS {
        manifest.push_str(&format!("    \"{path}\",\n"));
    }
    manifest.push_str("];\n");
    let manifest_out = out_dir.join("v3_generated_files.rs");
    fs::write(&manifest_out, manifest)
        .unwrap_or_else(|e| panic!("failed to write {}: {}", manifest_out.display(), e));
    let extdeps_out = out_dir.join("v3_extdeps_files.rs");
    fs::write(&extdeps_out, extdeps_generated)
        .unwrap_or_else(|e| panic!("failed to write {}: {}", extdeps_out.display(), e));
    let gunbc_out = out_dir.join("v3_gunbc_files.rs");
    fs::write(&gunbc_out, gunbc_generated)
        .unwrap_or_else(|e| panic!("failed to write {}: {}", gunbc_out.display(), e));
    let lens_bootstrap_out = out_dir.join("v3_lens_bootstrap_files.rs");
    fs::write(&lens_bootstrap_out, lens_bootstrap_generated)
        .unwrap_or_else(|e| panic!("failed to write {}: {}", lens_bootstrap_out.display(), e));
}
