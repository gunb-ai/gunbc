// Build script: enumerate every `*.dag` file in the v3-only staged
// directories and generate Rust arrays of `(path, content)` pairs
// the bootstrap loader consumes.
//
// Generated arrays:
//   - `STAGED_FILES`          for `src/v3/std/*.dag`
//   - `V3_SPECS`              for `src/v3/spec/*.dag`
//   - `COMPILER_FILES`        for `src/v3/compiler/*.dag` (except `tokenize.dag`; see below)
//
// Adding a new staged std/spec/compiler file becomes a pure file-system change
// — no Rust edits to `bootstrap.rs`, no fixture-array maintenance, no skip-list drift.
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

/// Escape `body` so it can replace a `*_SPLICE_V1` placeholder inside a `.dag` double-quoted
/// `TestClaim.source` literal (R1 gate fixture hygiene; see `emit_r1_gates_fixture`).
fn escape_dag_double_quoted_body(body: &str) -> String {
    let mut out = String::with_capacity(body.len() + 8);
    for ch in body.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

/// Emit `tests/fixtures/r1_gates.dag` from `r1_gates.template.dag` + canonical lens and `.v3`
/// program text.
///
/// **Hygiene:** `r1_gates.dag` is tracked in git but produced here so the spliced lens stays
/// byte-identical to `src/v3/lenses/named_function_count.dag`, and Lane E `TestClaim.source`
/// strings are spliced from canonical `.v3` fixtures (single authority; INVARIANTS P2).
/// Hand-edit **`r1_gates.template.dag`** only — direct edits to `r1_gates.dag` are overwritten
/// on the next `cargo build`. We skip `fs::write` when the generated bytes are unchanged to
/// avoid spurious working-tree noise.
fn emit_r1_gates_fixture(manifest_path: &Path, v3_dir: &Path) {
    /// Must appear exactly once in `r1_gates.template.dag` (inside the `user_authored_lens_compiles` `source:` line).
    const LENS_SPLICE_SENTINEL: &str = "R1_NAMED_FUNCTION_COUNT_LENS_SPLICE_V1";
    /// May appear multiple times (each `TestClaim.source` that embeds merge-sort gets the same bytes).
    const MERGE_SORT_PAIR_V3_SPLICE: &str = "R1_MERGE_SORT_PAIR_V3_SPLICE_V1";
    const LANE_E_DIFF_WITNESS_V3_SPLICE: &str = "R1_LANE_E_DIFFERENTIAL_WITNESS_V3_SPLICE_V1";

    let fixtures_dir = manifest_path.join("tests/fixtures");
    let template_path = fixtures_dir.join("r1_gates.template.dag");
    let lens_path = v3_dir.join("lenses/named_function_count.dag");
    let merge_sort_pair_v3 = fixtures_dir.join("r1_merge_sort_pair.v3");
    let lane_e_diff_witness_v3 = fixtures_dir.join("r1_lane_e_differential_witness.v3");
    let out_path = fixtures_dir.join("r1_gates.dag");

    println!("cargo:rerun-if-changed={}", template_path.display());
    for p0_script in [
        manifest_path.join("../../../scripts/r1_p0_no_fabrication_sentinel.sh"),
        manifest_path.join("../../../scripts/r1_p0_rest_ops_aligned.py"),
    ] {
        println!("cargo:rerun-if-changed={}", p0_script.display());
    }
    println!("cargo:rerun-if-changed={}", lens_path.display());
    println!("cargo:rerun-if-changed={}", merge_sort_pair_v3.display());
    println!(
        "cargo:rerun-if-changed={}",
        lane_e_diff_witness_v3.display()
    );

    let template = fs::read_to_string(&template_path).unwrap_or_else(|e| {
        panic!(
            "failed to read R1 gate template {}: {}",
            template_path.display(),
            e
        )
    });
    let lens = fs::read_to_string(&lens_path).unwrap_or_else(|e| {
        panic!(
            "failed to read canonical lens {}: {}",
            lens_path.display(),
            e
        )
    });
    let merge_sort_pair_src = fs::read_to_string(&merge_sort_pair_v3).unwrap_or_else(|e| {
        panic!(
            "failed to read R1 merge-sort pair fixture {}: {}",
            merge_sort_pair_v3.display(),
            e
        )
    });
    let lane_e_diff_src = fs::read_to_string(&lane_e_diff_witness_v3).unwrap_or_else(|e| {
        panic!(
            "failed to read R1 Lane E differential witness fixture {}: {}",
            lane_e_diff_witness_v3.display(),
            e
        )
    });

    let lens_count = template.matches(LENS_SPLICE_SENTINEL).count();
    if lens_count != 1 {
        panic!(
            "{} must contain exactly one `{LENS_SPLICE_SENTINEL}` for named_function_count lens (found {lens_count})",
            template_path.display()
        );
    }
    let merge_count = template.matches(MERGE_SORT_PAIR_V3_SPLICE).count();
    if merge_count < 1 {
        panic!(
            "{} must contain at least one `{MERGE_SORT_PAIR_V3_SPLICE}` (spliced into every Lane E claim that embeds merge-sort; found {merge_count})",
            template_path.display()
        );
    }
    let diff_count = template.matches(LANE_E_DIFF_WITNESS_V3_SPLICE).count();
    if diff_count != 1 {
        panic!(
            "{} must contain exactly one `{LANE_E_DIFF_WITNESS_V3_SPLICE}` for r1_lane_e_differential_witness.v3 (found {diff_count})",
            template_path.display()
        );
    }

    let mut generated =
        template.replace(LENS_SPLICE_SENTINEL, &escape_dag_double_quoted_body(&lens));
    generated = generated.replace(
        MERGE_SORT_PAIR_V3_SPLICE,
        &escape_dag_double_quoted_body(&merge_sort_pair_src),
    );
    generated = generated.replace(
        LANE_E_DIFF_WITNESS_V3_SPLICE,
        &escape_dag_double_quoted_body(&lane_e_diff_src),
    );
    let needs_write = match fs::read_to_string(&out_path) {
        Ok(existing) => existing != generated,
        Err(_) => true,
    };
    if needs_write {
        fs::write(&out_path, generated).unwrap_or_else(|e| {
            panic!(
                "failed to write generated R1 gate fixture {}: {}",
                out_path.display(),
                e
            )
        });
    }
}

/// Emit `dsl/std/render_repeat_string_bootstrap.dag` from the excerpt between
/// `GUNBC_BOOTSTRAP_EMIT_BEGIN` / `GUNBC_BOOTSTRAP_EMIT_END` in `dsl/std/render.dag`.
///
/// **Hygiene:** `repeat_string` exists in the bootstrap bundle only via this excerpt — no
/// hand-maintained duplicate of `render.dag` (INVARIANTS P2). Edit the marked region in
/// `render.dag` only; this output is overwritten on `cargo build -p v3-compiler`.
fn emit_render_repeat_string_bootstrap(repo_root: &Path) {
    const BEGIN: &str = "// GUNBC_BOOTSTRAP_EMIT_BEGIN\n";
    const END: &str = "// GUNBC_BOOTSTRAP_EMIT_END";
    let render_path = repo_root.join("dsl/std/render.dag");
    let out_path = repo_root.join("dsl/std/render_repeat_string_bootstrap.dag");
    println!("cargo:rerun-if-changed={}", render_path.display());
    let render_src = fs::read_to_string(&render_path).unwrap_or_else(|e| {
        panic!(
            "emit_render_repeat_string_bootstrap: read {}: {e}",
            render_path.display()
        )
    });
    let start = render_src.find(BEGIN).unwrap_or_else(|| {
        panic!(
            "{}: missing `{}` marker line",
            render_path.display(),
            BEGIN.trim_end()
        )
    });
    let inner_start = start + BEGIN.len();
    let end_rel = render_src[inner_start..].find(END).unwrap_or_else(|| {
        panic!(
            "{}: missing `{END}` after begin marker",
            render_path.display()
        )
    });
    let excerpt = render_src[inner_start..inner_start + end_rel].trim_end();
    let generated = format!(
        "// AUTO-GENERATED by src/v3/compiler/build.rs. Do not hand-edit.\n\
         //\n\
         // Source: excerpt between GUNBC_BOOTSTRAP_EMIT_BEGIN/END in dsl/std/render.dag.\n\
         // Single substrate slice for repeat_string in the bootstrap bundle (INVARIANTS P2).\n\
         // Regenerated when render.dag changes. Dissolution: full render.dag in bootstrap.\n\
         \n\
         module std.render_repeat_string_bootstrap\n\
         \n\
         {excerpt}\n"
    );
    match fs::read_to_string(&out_path) {
        Ok(existing) if existing == generated => {}
        _ => fs::write(&out_path, generated).unwrap_or_else(|e| {
            panic!(
                "emit_render_repeat_string_bootstrap: write {}: {e}",
                out_path.display()
            )
        }),
    }
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
    emit_r1_gates_fixture(&manifest_path, v3_dir);
    let src_dir = v3_dir.parent().expect("v3 dir has parent");
    let repo_root = src_dir.parent().expect("src dir has parent");
    emit_render_repeat_string_bootstrap(repo_root);
    let std_dir = v3_dir.join("std");
    let spec_dir = v3_dir.join("spec");
    let compiler_dir = manifest_path.clone();
    let extdeps_dir = repo_root.join("dsl").join("extdeps");
    let gunbc_dir = repo_root.join("dsl").join("gunbc");
    let dsl_std_dir = repo_root.join("dsl").join("std");

    // Tell Cargo to re-run the script if any staged std/spec/compiler file
    // changes. Without this, adding a new file wouldn't trigger a
    // rebuild and bootstrap would silently miss it.
    println!("cargo:rerun-if-changed={}", spec_dir.display());
    println!("cargo:rerun-if-changed={}", compiler_dir.display());
    println!("cargo:rerun-if-changed={}", extdeps_dir.display());
    println!("cargo:rerun-if-changed={}", gunbc_dir.display());
    println!("cargo:rerun-if-changed={}", dsl_std_dir.display());

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
    let staged_entries = collect_dag_entries(
        &std_dir,
        &[
            "list.dag",
            "substrate_minimal.dag",
            "effects.dag",
            "substrate.dag",
            // `emit_model.dag` / per-target `*_method_template_contracts.dag` rows
            // reference `MethodRef` from `methods.dag`. Alphabetical staging would
            // process `go_method_template_contracts.dag` before `methods.dag` (g < m),
            // leaving `MethodRef` as an identifier stub during structural lowering of
            // `dag_method: { decl: … }` — `walk_to_conj_decl` then fails closed. Rust and
            // Python fixtures happen to load after `methods.dag` and were unaffected.
            "methods.dag",
        ],
    );
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
        "src/v3/compiler/src/dag_lookup_generated.rs",
        "src/v3/compiler/src/dag_cost_generated.rs",
        "src/v3/compiler/src/dag_scalar_generated.rs",
        "src/v3/compiler/src/dag_value_body_generated.rs",
        "src/v3/compiler/src/diagnostics_generated.rs",
        "src/v3/compiler/src/infer_helpers_generated.rs",
        "src/v3/compiler/src/complexity_lens_generated.rs",
        "src/v3/compiler/src/lens_cost_symbolic_generated.rs",
        "src/v3/compiler/src/lens_cost_target_realization_generated.rs",
        "src/v3/compiler/src/lens_effect_enumeration_generated.rs",
        "src/v3/compiler/src/lens_provenance_generated.rs",
        "src/v3/compiler/src/lens_structural_resolution_generated.rs",
        "src/v3/compiler/src/lens_unused_parameters_generated.rs",
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
}
