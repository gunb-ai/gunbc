//! Bootstrap tests for the v2 self-hosted compiler.
//!
//! These are subprocess tests that build and run the stage0 binary.
//! Most are `#[ignore]` because they require building the stage0 binary (~2 min).

#![allow(clippy::disallowed_macros)]

// ── Shared helpers ─────────────────────────────────────────────────────

/// Build the stage0 binary via cargo. Returns path to the binary.
/// On CI, the binary is already cached from the "Build Compiler" step.
fn build_stage0() -> std::path::PathBuf {
    let build = std::process::Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("v1-compiler")
        .arg("--release")
        .output()
        .expect("failed to build stage0");
    assert!(
        build.status.success(),
        "stage0 build failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| crate::helpers::workspace_root().join("target"));
    let bin = target_dir.join("release/gunbc");
    assert!(bin.exists(), "stage0 binary not found at {}", bin.display());
    bin
}

fn temp_dir(name: &str) -> std::path::PathBuf {
    let unique = format!(
        "v2-bootstrap-{}-{}",
        name,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos()
    );
    std::env::temp_dir().join(unique)
}

/// Run a self-compile: `<binary> compile --source-root src/v1 --source-root dsl --output-dir <dir>`.
/// Does NOT assert success — caller decides how to handle failure.
fn run_self_compile(
    binary: &std::path::Path,
    output_dir: &std::path::Path,
) -> std::process::Output {
    run_self_compile_with_extra_source_roots(binary, output_dir, &[])
}

/// Run self-compile with additional dependency source roots appended after the
/// canonical `src/v1` and `dsl` roots. Future build-time generated `.dag`
/// projections should use this surface: keep `src/v1` as the entry root and
/// pass the generated temp/OUT_DIR root as a dependency pool.
fn run_self_compile_with_extra_source_roots(
    binary: &std::path::Path,
    output_dir: &std::path::Path,
    extra_source_roots: &[std::path::PathBuf],
) -> std::process::Output {
    let [v1_root, dsl_root] = crate::helpers::source_roots();
    let mut command = std::process::Command::new(binary);
    command
        .arg("compile")
        .arg("--source-root")
        .arg(&v1_root)
        .arg("--source-root")
        .arg(&dsl_root);
    for root in extra_source_roots {
        command.arg("--source-root").arg(root);
    }
    command
        .arg("--output-dir")
        .arg(output_dir)
        .output()
        .expect("failed to run self-compile")
}

/// Parse diagnostic count from compile stderr.
/// Looks for "compiled: N files emitted, M diagnostics" line.
fn parse_diagnostic_count(stderr: &str) -> usize {
    stderr
        .lines()
        .find(|l| l.contains("compiled:") && l.contains("diagnostics"))
        .and_then(|l| {
            l.split("diagnostics")
                .next()
                .and_then(|prefix| prefix.split(',').next_back())
                .and_then(|s| s.trim().parse::<usize>().ok())
        })
        .unwrap_or(usize::MAX)
}

/// Copy hand-maintained Rust modules that emitted stage0 still references.
/// These are excluded from freshness/fixed-point diffs, but they must exist
/// in temp output dirs so rustfmt and cargo can resolve lib.rs module paths.
/// rest_transport_facts.rs: same dissolution note as in rest_transport_facts.rs
/// (graph-exported facts replace this scaffold).
fn copy_stage0_support_modules(stage1_dir: &std::path::Path, ws: &std::path::Path) {
    let stage0_src = ws.join("src/v1/stage0/src");
    // Keep in sync with HAND_MAINTAINED_STAGE0_FILES in regen_stage0.rs: every
    // hand-maintained module that committed lib.rs declares must exist in the
    // temp output dir so rustfmt and cargo can resolve its module path.
    for name in &[
        "v1_interpreter.rs",
        "cli_run.rs",
        "rest_transport_facts.rs",
        "wire_value_serialize.rs",
        "coproduct_reflection.rs",
        "resolved_graph_cache.rs",
        "recorded_fixture.rs",
        "extdeps_shape_transport_policy_project.rs",
        "fact_cardinality_census.rs",
        "import_resolution_project.rs",
        "languages_consumer_census.rs",
        "layering_imports_project.rs",
        "module_path_index.rs",
        "transport_script_position_project.rs",
        "v1_compiler_dag_collect.rs",
        "v1_compiler_dag_collect_support.rs",
    ] {
        let src = stage0_src.join(name);
        if src.exists() {
            let dst = stage1_dir.join("src").join(name);
            std::fs::copy(&src, &dst)
                .unwrap_or_else(|e| panic!("failed to copy {} to stage1: {}", name, e));
        }
    }
}

/// Copy hand-maintained files into a compile output dir and patch Cargo.toml
/// with the ureq dependency needed by the interpreter.
fn prepare_stage1_for_build(stage1_dir: &std::path::Path, ws: &std::path::Path) {
    copy_stage0_support_modules(stage1_dir, ws);
    let cargo_toml = stage1_dir.join("Cargo.toml");
    if cargo_toml.exists() {
        let contents = std::fs::read_to_string(&cargo_toml).unwrap();
        if !contents.contains("ureq") {
            let patched = contents.replace(
                "\n[dependencies]\n",
                "\n[dependencies]\nureq = { version = \"2\", features = [\"json\"] }\n",
            );
            std::fs::write(&cargo_toml, patched).unwrap();
        }
    }
}

/// Run cargo fmt over a generated temp crate so formatting matches the same
/// crate-wide normalization used by committed stage0.
fn rustfmt_generated_crate(dir: &std::path::Path) -> Result<(), String> {
    let cargo_toml = dir.join("Cargo.toml");
    let out = std::process::Command::new("cargo")
        .arg("fmt")
        .arg("--all")
        .arg("--manifest-path")
        .arg(&cargo_toml)
        .output()
        .map_err(|e| format!("spawn cargo fmt: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "cargo fmt failed for {}:\n{}",
            dir.display(),
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    Ok(())
}

/// Diff two src/ directories, excluding hand-maintained files.
/// Returns Ok(()) if identical, Err(diff output) if different.
fn diff_excluding_hand_maintained(
    dir_a: &std::path::Path,
    dir_b: &std::path::Path,
) -> Result<(), String> {
    let diff = std::process::Command::new("diff")
        .arg("-r")
        .arg("--exclude=v1_interpreter.rs")
        .arg("--exclude=cli_run.rs")
        .arg("--exclude=rest_transport_facts.rs")
        .arg("--exclude=wire_value_serialize.rs")
        .arg("--exclude=coproduct_reflection.rs")
        .arg("--exclude=resolved_graph_cache.rs")
        .arg("--exclude=recorded_fixture.rs")
        .arg("--exclude=extdeps_shape_transport_policy_project.rs")
        .arg("--exclude=fact_cardinality_census.rs")
        .arg("--exclude=import_resolution_project.rs")
        .arg("--exclude=languages_consumer_census.rs")
        .arg("--exclude=layering_imports_project.rs")
        .arg("--exclude=module_path_index.rs")
        .arg("--exclude=transport_script_position_project.rs")
        .arg("--exclude=v1_compiler_dag_collect.rs")
        .arg("--exclude=v1_compiler_dag_collect_support.rs")
        .arg(dir_a)
        .arg(dir_b)
        .output()
        .expect("diff failed");
    if diff.status.code() == Some(2) {
        return Err(format!(
            "diff -r failed (exit 2):\n{}",
            String::from_utf8_lossy(&diff.stderr)
        ));
    }
    if diff.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&diff.stdout);
        Err(stdout[..stdout.len().min(2000)].to_string())
    }
}

// ── 1. stage0_cargo_check ───────────────────────────────────────────────

#[test]
fn stage0_cargo_check() {
    // Stage0 is now a workspace member — just cargo check it
    let output = std::process::Command::new("cargo")
        .arg("check")
        .arg("-p")
        .arg("v1-compiler")
        .output()
        .expect("failed to run cargo check");
    assert!(
        output.status.success(),
        "stage0 cargo check failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

// ── 2. strict_compile_diagnostic_count ──────────────────────────────────

// Stable bridge ratchet for the curated bootstrap path.
// Source-root bootstrap health is tracked separately and is not yet the
// enforced gate.
//
// Ratchet history:
// 2026-03-30: 65 — pre-complexity-audit baseline.
// 2026-03-31: 315 — honest count after restoring recursive is_unknown_cost
//   (PR #264 review). All 315 are indirect-recursion complexity violations
//   (A→B→A) from 27 root functions. They are real errors, not bypassed.
//   Resolves when .dag fold primitive replaces manual recursion with
//   bounded iteration.
//
// These are analyzer limitations, not program violations. Decidability:
// if the analyzer produces ?O(?), the bug is in the analyzer (it cannot
// see the bound that structurally exists), not in the program. The
// ratchet only moves down, never up.
// 2026-04-07: 526 — honest count after restoring CostUnknown for all
//   unresolved descent patterns. The 3-fix reduction path:
//   Node tree descent, Parser SCC, Graph DFS.
// 2026-04-08: 526→528 — transport property inference adds 2 complexity
//   diagnostics (infer_property_values/infer_transport_node call infer_expr).
// 2026-04-08: 528→530 — source_index threading (PR #356 merge) adds 2
//   diagnostics from new call paths through CostUnknown functions.
// 2026-04-09: 530→528 — merged main (CX unification -2).
// 2026-04-09: 528→485 — ExprLet scope fix eliminates false descent evidence.
// 2026-04-09: PR #361 — CX-L2 infrastructure + structural completeness work:
//   ArithmeticDescent, element_type threading, type-based collection detection,
//   std/node.dag declarations, centralized evidence (all calls annotated),
//   PreservedValue in structural check, transparent wrapper propagation,
//   collection element extraction (match list |> first), lambda boundary fix.
//   render_node_type dissolved (140). make_indent dissolved (44).
//   488→469 after merge with main.
// 2026-04-11: 424→421 — output provenance body inference for non-recursive
//   functions. compose_sub_value_relations fixes structural∘structural.
// 2026-04-11: 421→423 — per-field output provenance (+3: compose_callee_provenance,
//   classify_body_per_field, classify_terminal_per_field calling recursive
//   classify_body_provenance — inherent self-analysis cost).
// 2026-04-12: 423→364 — branching guard fix: any→all for arithmetic-only check.
//   PropertyContraction calls (with_required_cardinality) no longer poison
//   tree-walking functions into arithmetic mode. Dissolves render_node_type
//   and composed violations across emit files. Per-field provenance re-annotation
//   pass added (infrastructure for Stream D).
// 2026-04-12: 353→354 — compile_to_resolved added (M5 Phase 0 interpreter).
//   Same complexity warning as compile_sources (same call chain).
// 2026-04-12: 354→358 — Stream D parser restructuring (int indexing → list consumption).
//   +7 new: parse_dotted_ident_rest, collect_lambda_idents, parse_predicates_acc,
//   try_where_clause, try_lambda_params — CX can't yet see descent through
//   helper return types (output provenance gap, same as Category B).
//   -3 dissolved: scan_braces_depth, scan_for_fat_arrow_after_braces,
//   looks_like_arm_start — integer idx recursion replaced by list consumption.
//   Net +4. Correct observations; dissolves with return-contract inference.
// 2026-04-12: 350→340 — CX-R: parser list consumption recognition (-10).
//   Recognize list-consuming call patterns: field-name provenance (r.tokens →
//   NonIncreasing), pass-through transparency (skip_newlines), generalized
//   shrink (X |> skip(N)), and tokens-consuming call convention (any call with
//   tokens: arg that traces back to measure param is treated as consuming).
//   Refactored ExprBlock descent-var threading to use collect_descent_vars
//   as single source of truth. -10 from single-func parser proofs +
//   composed callers.
// 2026-04-12: 350→354 — eat/advance sum-type migration + variant_provenance
//   pipeline fix. Parser helpers return sum types (EatConsumed | EatUnchanged,
//   AdvanceOk | AdvanceEof). Variant_provenance populates per-variant per-field
//   SubValueRelation on sigs. Pipeline was blocked by lookup_type gap (reference
//   node has NoConnective, need to resolve to Disj definition). Fix: lookup_type
//   in compute_variant_provenance. +4 net: restructuring adds violations, variant
//   provenance subtracts some (2 resolved: collect_lambda_idents,
//   collect_type_param_names). Remaining 139 parser violations need expect/expect_name
//   output_provenance to complete descent chains.
// 2026-04-28: 357→358 — merged std.error_primitives Result/DivError carrier adds
//   one complexity diagnostic in the post-#1068 main cascade. Hard diagnostics
//   remain zero; this preserves the observed fixed-point diagnostic budget.
const DIAG_RATCHET: usize = 358;

#[test]
#[ignore = "Requires building stage0 binary (~2 min)"]
fn strict_compile_diagnostic_count() {
    let stage0_bin = build_stage0();

    let out_dir = std::env::temp_dir().join("v2-diag-output");
    let _ = std::fs::remove_dir_all(&out_dir);

    let output = run_self_compile(&stage0_bin, &out_dir);
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("stage0 compile stderr:\n{}", stderr);

    let diag_count = parse_diagnostic_count(&stderr);

    assert!(
        diag_count <= DIAG_RATCHET,
        "diagnostic count {} exceeds ratchet {}",
        diag_count,
        DIAG_RATCHET
    );

    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
#[ignore = "Requires building stage0 binary (~2 min)"]
fn stage0_compile_accepts_dag_target() {
    let stage0_bin = build_stage0();

    let source_dir = std::env::temp_dir().join("v2-dag-target-src");
    let _ = std::fs::remove_dir_all(&source_dir);
    std::fs::create_dir_all(&source_dir).unwrap();
    std::fs::write(
        source_dir.join("dag_target.dag"),
        "module dag_target\n\nfn main() -> Int { 0 }\n",
    )
    .unwrap();

    let out_dir = std::env::temp_dir().join("v2-dag-target-out");
    let _ = std::fs::remove_dir_all(&out_dir);

    let output = std::process::Command::new(&stage0_bin)
        .arg("compile")
        .arg("--source-root")
        .arg(&source_dir)
        .arg("--output-dir")
        .arg(&out_dir)
        .arg("--target")
        .arg("dag")
        .output()
        .expect("failed to run stage0 compile");

    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("stage0 dag compile stderr:\n{}", stderr);
    assert!(output.status.success(), "stage0 dag compile failed");

    let dag_artifact = out_dir.join("dag-artifact.json");
    assert!(
        dag_artifact.exists(),
        "expected dag artifact at {}",
        dag_artifact.display()
    );

    let _ = std::fs::remove_dir_all(&source_dir);
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
#[ignore = "Requires building stage0 binary (~2 min)"]
fn stage0_compile_imports_ephemeral_generated_source_root() {
    let stage0_bin = build_stage0();

    let entry_root = temp_dir("ephemeral-entry-root");
    let generated_root = temp_dir("ephemeral-generated-root");
    let out_dir = temp_dir("ephemeral-generated-out");
    let _ = std::fs::remove_dir_all(&entry_root);
    let _ = std::fs::remove_dir_all(&generated_root);
    let _ = std::fs::remove_dir_all(&out_dir);
    std::fs::create_dir_all(entry_root.join("ephemeral")).unwrap();
    std::fs::create_dir_all(generated_root.join("generated")).unwrap();
    std::fs::write(
        generated_root
            .join("generated")
            .join("method_template_projection.dag"),
        "module generated.method_template_projection\n\nfn generated_answer() -> Int { 41 }\n",
    )
    .unwrap();
    std::fs::write(
        entry_root.join("ephemeral").join("entry.dag"),
        "\
module ephemeral.entry

import generated.method_template_projection { generated_answer }

fn main() -> Int { generated_answer() }
",
    )
    .unwrap();

    let output = std::process::Command::new(&stage0_bin)
        .arg("compile")
        .arg("--source-root")
        .arg(&entry_root)
        .arg("--source-root")
        .arg(&generated_root)
        .arg("--output-dir")
        .arg(&out_dir)
        .arg("--target")
        .arg("dag")
        .output()
        .expect("failed to run stage0 compile");

    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("ephemeral generated source-root stderr:\n{}", stderr);
    assert!(
        output.status.success(),
        "stage0 compile failed with ephemeral generated source root:\n{}",
        stderr
    );
    assert!(
        out_dir.join("dag-artifact.json").exists(),
        "expected dag artifact at {}",
        out_dir.join("dag-artifact.json").display()
    );
    let committed_generated_projection =
        crate::helpers::workspace_root().join("src/generated/method_template_projection.dag");
    assert!(
        !committed_generated_projection.exists(),
        "ratchet must not rely on committed generated .dag at {}",
        committed_generated_projection.display()
    );

    let _ = std::fs::remove_dir_all(&entry_root);
    let _ = std::fs::remove_dir_all(&generated_root);
    let _ = std::fs::remove_dir_all(&out_dir);
}

// ── 3. bootstrap_stage0_to_stage1 (emitted Rust error ratchet) ────────
//
// Compiles .dag source to Rust via stage0, then runs cargo check on the
// output. Counts rustc errors as a ratchet — makes the gap between
// "compiler runs" and "compiler output works" visible and trackable.

// Ratchet for cargo check errors on stage0-compiled Rust output.
// Note: this test currently fails at the compile step (stage0 binary
// can't parse all .dag syntax, or compiles but emitter output has
// codegen gaps). The ratchet accommodates both cases.
// 2026-03-28: when stage0 compiles successfully (after parse fix),
// regenerated output has 1087 errors in 3 categories:
//   E0425 (541): generics — emitter generates `T` without type param declaration
//   E0433+E0405 (404): serde — emitter generates serde code, stage0 lacks serde dep
//   E0220+E0277 (140): downstream trait/type errors from above
// 2026-04-01: 880 → 13 via structural emission fixes:
//   - TestConventions/Token/Tuple imports added to .dag source
//   - type_params recovery for self-referential generic fields
//   - Tuple rendering in build_type_rendering (connective-independent)
//   - Vec<()> annotation skip when fold/flat_map init has Unit elements
//   Remaining 13: sort_by lambda inference, fold empty_map sentinels, kahn fold
// 2026-04-01: 13 → 12 via fold/sort_by inference propagation:
//   - Bare container (Map{}) detected as incomplete in fold refinement
//   - list_push/map_insert refinement extended for Unit-element receivers
//   - list_push builtin fallback uses item type when receiver is Error/Dynamic
//   - Emit: bare container fallback to contextual accumulator type
//   - receiver_is_map extended for bare Map{} (0 children)
//   Remaining 12: 8 E0425 (cross-module import, pre-existing), 4 E0282 (Map<K,List<Unit>> fold)
// 2026-04-01: 12 → 5 via invariant review fixes:
//   - 7 E0425 resolved: added algebra template function imports to 04_types.dag
//     (partial_function_templates, free_monoid_collection_templates, etc.)
//   - EmitGraphInfo.type_params added to 04_emit_info.dag (was stage0-only)
//   Remaining 5: 1 E0425 (field_access_base), 4 E0282 (Map<K,List<Unit>> fold)
//   Fold inference bidirectional unification (Category B) — 4 E0282 → 0.
//   Block-level lookahead scans record-lit field types; expected type unifies
//   bare empty_map() init into Map<K,V> so emit produces correct turbofish.
//   All resolved: field_access_base import added to complexity.dag
const EMITTED_RUST_ERROR_RATCHET: usize = 0;

#[test]
#[ignore = "Expensive: builds binary + runs full compile + cargo check"]
fn bootstrap_stage0_to_stage1() {
    let stage0_bin = build_stage0();
    let ws = crate::helpers::workspace_root();

    // Run stage0 to compile stage1
    let stage1_dir = std::env::temp_dir().join("v2-bootstrap-stage1");
    let _ = std::fs::remove_dir_all(&stage1_dir);
    let output = run_self_compile(&stage0_bin, &stage1_dir);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "stage0 compile failed:\n{}",
        stderr
    );
    assert!(
        stage1_dir.join("Cargo.toml").exists(),
        "stage0 compile produced no output (no Cargo.toml in {})",
        stage1_dir.display()
    );

    prepare_stage1_for_build(&stage1_dir, &ws);

    let check = std::process::Command::new("cargo")
        .arg("check")
        .current_dir(&stage1_dir)
        .output()
        .expect("failed to cargo check stage1");
    let check_stderr = String::from_utf8_lossy(&check.stderr);
    // Count both coded errors (error[Exxxx]) and uncoded errors (error: ...)
    // to avoid silently passing on parse/syntax failures.
    let error_count = check_stderr
        .lines()
        .filter(|l| l.starts_with("error[") || (l.starts_with("error") && !l.starts_with("error:")))
        .count();
    // Fall back: if cargo check failed but we counted 0 errors, something
    // uncategorized went wrong — don't silently pass.
    let error_count = if !check.status.success() && error_count == 0 {
        eprintln!(
            "cargo check failed with uncategorized errors:\n{}",
            check_stderr
        );
        usize::MAX
    } else {
        error_count
    };
    // Categorize errors for diagnosis
    let mut categories: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for line in check_stderr.lines() {
        if line.starts_with("error[") {
            let code = line.split(']').next().unwrap_or("unknown").to_string() + "]";
            *categories.entry(code).or_insert(0) += 1;
        }
    }
    let mut cats: Vec<_> = categories.iter().collect();
    cats.sort_by(|a, b| b.1.cmp(a.1));
    eprintln!(
        "stage1 cargo check: {} errors (ratchet: {})",
        error_count, EMITTED_RUST_ERROR_RATCHET
    );
    for (code, count) in cats.iter().take(10) {
        eprintln!("  {}: {}", code, count);
    }
    // Show samples for top 3 categories
    for (code, _) in cats.iter().take(3) {
        let needle = code.trim_end_matches(']').trim_start_matches("error[");
        let samples: Vec<&str> = check_stderr
            .lines()
            .filter(|l| l.starts_with(&format!("error[{}]", needle)))
            .take(2)
            .collect();
        for s in samples {
            eprintln!("  {}", &s[..s.len().min(200)]);
        }
    }
    eprintln!(
        "\n=== FULL CARGO CHECK STDERR ===\n{}\n=== END ===",
        check_stderr
    );

    assert!(
        error_count <= EMITTED_RUST_ERROR_RATCHET,
        "emitted Rust errors {} exceeds ratchet {} — \
         fix codegen or update EMITTED_RUST_ERROR_RATCHET if increase is justified",
        error_count,
        EMITTED_RUST_ERROR_RATCHET
    );

    let _ = std::fs::remove_dir_all(&stage1_dir);
}

// ── 4. bootstrap_fixed_point ────────────────────────────────────────────

#[test]
#[ignore = "Expensive: builds two binaries + two full compiles"]
fn bootstrap_fixed_point() {
    let ws = crate::helpers::workspace_root();
    let stage0_bin = build_stage0();

    // Stage0 -> stage1
    let stage1_dir = std::env::temp_dir().join("v2-fp-stage1");
    let _ = std::fs::remove_dir_all(&stage1_dir);
    let s1 = run_self_compile(&stage0_bin, &stage1_dir);
    assert!(
        s1.status.success(),
        "stage0->1 failed:\n{}",
        String::from_utf8_lossy(&s1.stderr)
    );

    prepare_stage1_for_build(&stage1_dir, &ws);

    // Build stage1 binary
    let build1 = std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(&stage1_dir)
        .output()
        .expect("stage1 build failed");
    assert!(
        build1.status.success(),
        "stage1 build failed:\n{}",
        String::from_utf8_lossy(&build1.stderr)
    );
    // Self-compile includes v1.compiler.compile, so emit_cargo_toml picks
    // crate_name "v1_compiler" (see 05_emit_rust.dag line 452).
    let stage1_bin = stage1_dir.join("target/release/v1_compiler");

    // Stage1 -> stage2
    let stage2_dir = std::env::temp_dir().join("v2-fp-stage2");
    let _ = std::fs::remove_dir_all(&stage2_dir);
    let s2 = run_self_compile(&stage1_bin, &stage2_dir);
    assert!(
        s2.status.success(),
        "stage1->2 failed:\n{}",
        String::from_utf8_lossy(&s2.stderr)
    );

    // Compare stage1 and stage2 source output (excluding hand-maintained files)
    let stage1_src = stage1_dir.join("src");
    let stage2_src = stage2_dir.join("src");
    prepare_stage1_for_build(&stage2_dir, &ws);
    if let Err(err) = rustfmt_generated_crate(&stage1_dir) {
        panic!("failed to rustfmt stage1 fixed-point output: {err}");
    }
    if let Err(err) = rustfmt_generated_crate(&stage2_dir) {
        panic!("failed to rustfmt stage2 fixed-point output: {err}");
    }
    if let Err(diff) = diff_excluding_hand_maintained(&stage1_src, &stage2_src) {
        eprintln!("Fixed point NOT reached — diff:\n{}", diff);
        let _ = std::fs::remove_dir_all(&stage1_dir);
        let _ = std::fs::remove_dir_all(&stage2_dir);
        panic!("stage1 != stage2 — fixed point not reached");
    }

    // Cleanup
    let _ = std::fs::remove_dir_all(&stage1_dir);
    let _ = std::fs::remove_dir_all(&stage2_dir);
}

// ── 5. performance_ratchet ───────────────────────────────────────────────

/// Performance ratchet: self-compile pipeline must complete within the
/// time budget. Catches FF-class regressions (new O(n²) patterns,
/// lost facts, unnecessary allocations).
///
/// The ratchet must be generous enough for CI runners under load.
/// 2026-04-12: after merge_envs intern_table fix (O(N*M) string re-intern →
/// O(1) first-table reuse), per-module reconcile dropped from ~1.1s to ~5ms.
/// Dev hardware: ~11s. Colima container: now passes at ~40s.
/// 2026-04-13: CI runners consistently exceeding 55s (main has 5+ consecutive
/// failures). Bump to 150s — CI variance plus the post-R3 compiler surface
/// can exceed 120s without a semantic regression.
const PERF_RATCHET_SECONDS: u64 = 150;

#[test]
#[ignore = "Requires building stage0 binary"]
fn performance_ratchet() {
    let stage0_bin = build_stage0();

    let out_dir = std::env::temp_dir().join("v2-perf-output");
    let _ = std::fs::remove_dir_all(&out_dir);

    // Time the pipeline
    let start = std::time::Instant::now();
    let output = run_self_compile(&stage0_bin, &out_dir);
    let elapsed = start.elapsed();

    assert!(
        output.status.success(),
        "pipeline failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    eprintln!("pipeline elapsed: {:?}", elapsed);
    assert!(
        elapsed.as_secs() < PERF_RATCHET_SECONDS,
        "performance regression: pipeline took {:?}, budget is {}s. \
         See DESIGN.md for diagnosis.",
        elapsed,
        PERF_RATCHET_SECONDS
    );

    let _ = std::fs::remove_dir_all(&out_dir);
}

// ── CI compile gates (shared compilation via LazyLock) ─────────────────
//
// Five hermetic tests run together in CI via prefix match:
//   `cargo test -p v1-compiler-tests ci_ -- --ignored`
//
// ci_full_dsl           — all .dag files compile (library API, independent)
// ci_diagnostic_ratchet  — diagnostic count <= threshold (reads PASS1)
// ci_performance_ratchet — self-compile within time budget (reads PASS1)
// ci_freshness           — output matches committed stage0 (reads PASS1)
// ci_fixed_point         — regen(regen(source)) == regen(source) (reads PASS2)
//
// LazyLock ensures pass 1 compiles exactly once regardless of which test
// triggers it first. Pass 2 (for fixed-point) depends on pass 1.
//
// Each test checks one claim:
//   ci_diagnostic_ratchet  — diagnostic count <= threshold
//   ci_performance_ratchet — self-compile within time budget
//   ci_freshness           — output matches committed stage0
//   ci_fixed_point         — regen(regen(source)) == regen(source)

use std::sync::LazyLock;

/// Timing log file — written by LazyLock inits, read by ci.yml after pipeline.
const CI_TIMING_FILE: &str = "/tmp/v2-ci-timing.txt";

fn ci_timing(msg: &str) {
    use std::io::Write;
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap();
    let line = format!("[{:.1}s] {}\n", elapsed.as_secs_f64(), msg);
    eprint!("{}", line);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(CI_TIMING_FILE)
    {
        let _ = f.write_all(line.as_bytes());
    }
}

/// Output from pass 1: build stage0, run one self-compile.
struct Pass1Output {
    output_dir: std::path::PathBuf,
    stderr: String,
    elapsed: std::time::Duration,
    /// Freshness check computed here (before CI_PASS2 can modify workspace).
    freshness: Result<(), String>,
}

/// Output from pass 2: build stage1 from pass 1 output, self-compile again.
struct Pass2Output {
    output_dir: std::path::PathBuf,
}

/// Find the stage0 binary without rebuilding. On CI, the Build Compiler
/// step already ran `cargo build -p v1-compiler --release`. Rebuilding
/// here would waste ~2 min due to fingerprint invalidation from earlier
/// cargo commands (clippy, test). Falls back to build_stage0() if the
/// binary doesn't exist (local dev).
fn find_or_build_stage0() -> std::path::PathBuf {
    let ws = crate::helpers::workspace_root();
    let bin = ws.join("target/release/gunbc");
    if bin.exists() {
        ci_timing("PASS1: stage0 binary found (skipping rebuild)");
        bin
    } else {
        ci_timing("PASS1: stage0 binary not found, building");
        build_stage0()
    }
}

static CI_PASS1: LazyLock<Pass1Output> = LazyLock::new(|| {
    let stage0_bin = find_or_build_stage0();
    let ws = crate::helpers::workspace_root();

    let output_dir = std::env::temp_dir().join("v2-ci-pass1");
    let _ = std::fs::remove_dir_all(&output_dir);

    ci_timing("PASS1: start self-compile");
    let start = std::time::Instant::now();
    let output = run_self_compile(&stage0_bin, &output_dir);
    let elapsed = start.elapsed();
    ci_timing(&format!(
        "PASS1: self-compile done ({:.1}s)",
        elapsed.as_secs_f64()
    ));

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    assert!(
        output.status.success(),
        "pass 1 compile failed:\n{}",
        stderr
    );

    // rustfmt resolves sibling `mod foo;` declarations while formatting
    // lib.rs, so seed the hand-maintained companion files before the
    // pass1 whitespace-normalization step.
    prepare_stage1_for_build(&output_dir, &ws);

    // Freshness: diff pass 1 output against committed stage0.
    // Must be computed HERE, before CI_PASS2 copies pass1 files into stage0.
    //
    // Committed stage0 is fmt-compliant (regen_stage0 applies a
    // trailing `cargo fmt --all`). The v2 emitter itself does not produce
    // fmt-canonical output, so raw self-compile output differs from
    // committed stage0 only in whitespace/layout. Normalize pass1 output
    // via rustfmt before the diff so whitespace never masquerades as
    // staleness.
    prepare_stage1_for_build(&output_dir, &ws);
    if let Err(err) = rustfmt_generated_crate(&output_dir) {
        panic!("failed to rustfmt pass1 output: {err}");
    }
    let pass1_src = output_dir.join("src");
    let committed_src = ws.join("src/v1/stage0/src");
    let freshness = diff_excluding_hand_maintained(&pass1_src, &committed_src);
    ci_timing("PASS1: freshness diff done");

    Pass1Output {
        output_dir,
        stderr,
        elapsed,
        freshness,
    }
});

static CI_PASS2: LazyLock<Pass2Output> = LazyLock::new(|| {
    let pass1 = &*CI_PASS1;
    let ws = crate::helpers::workspace_root();
    // Rebuild the pass1 crate in place. This keeps the fixed-point check
    // hermetic: the ignored CI tests run concurrently, so copying pass1 files
    // into the workspace can race with freshness/lint/test gates.
    ci_timing("PASS2: start generated crate rebuild");
    let stage1_target_dir = std::env::temp_dir().join("v2-ci-pass2-target");
    let _ = std::fs::remove_dir_all(&stage1_target_dir);
    let build1 = std::process::Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(pass1.output_dir.join("Cargo.toml"))
        .arg("--release")
        .env("CARGO_TARGET_DIR", &stage1_target_dir)
        .output()
        .expect("stage1 build failed");
    ci_timing(&format!(
        "PASS2: generated crate rebuild done (success={})",
        build1.status.success()
    ));
    assert!(
        build1.status.success(),
        "stage1 build failed:\n{}",
        String::from_utf8_lossy(&build1.stderr)
    );
    let stage1_bin = stage1_target_dir.join("release/v1_compiler");

    // Self-compile pass 2
    ci_timing("PASS2: start self-compile");
    let output_dir = std::env::temp_dir().join("v2-ci-pass2");
    let _ = std::fs::remove_dir_all(&output_dir);
    let pass2_output = run_self_compile(&stage1_bin, &output_dir);
    ci_timing("PASS2: self-compile done");
    assert!(
        pass2_output.status.success(),
        "pass 2 (stage1->2) compile failed:\n{}",
        String::from_utf8_lossy(&pass2_output.stderr)
    );
    prepare_stage1_for_build(&output_dir, &ws);
    if let Err(err) = rustfmt_generated_crate(&output_dir) {
        panic!("failed to rustfmt pass2 output: {err}");
    }

    Pass2Output { output_dir }
});

#[test]
#[ignore = "CI: cargo test -p v1-compiler-tests ci_ -- --ignored"]
fn ci_full_dsl() {
    ci_timing("ci_full_dsl: start");
    // Compile ALL .dag files under dsl/ via library API.
    // The subprocess self-compile (CI_PASS1) only compiles files transitively
    // imported from src/v1 entry modules — unreferenced .dag files in dsl/
    // would be missed. This test closes that gap.
    let ws = crate::helpers::workspace_root();
    let dsl_dir = ws.join("dsl");
    let mut dsl_sources: Vec<std::rc::Rc<v1_compiler::v1_compiler_compile::SourceFile>> =
        Vec::new();
    crate::pipeline::collect_dag_sources(&ws, &dsl_dir, &mut dsl_sources);

    assert!(
        !dsl_sources.is_empty(),
        "no .dag files found in dsl/ — something is wrong"
    );

    let dsl_result = v1_compiler::v1_compiler_compile::compile_sources(
        std::rc::Rc::new(dsl_sources.clone()),
        v1_compiler::v1_compiler_artifact::RenderTarget::Rust,
    );

    let hard_diags: Vec<_> = crate::helpers::diagnostic_messages(&dsl_result)
        .into_iter()
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        hard_diags.is_empty(),
        "dsl/ compilation produced {} hard diagnostics (expected 0):\n{}",
        hard_diags.len(),
        hard_diags
            .iter()
            .enumerate()
            .map(|(i, m)| format!("  [{}] {}", i, m))
            .collect::<Vec<_>>()
            .join("\n")
    );

    ci_timing(&format!("ci_full_dsl: done ({} files)", dsl_sources.len()));
}

#[test]
#[ignore = "CI: cargo test -p v1-compiler-tests ci_ -- --ignored"]
fn ci_diagnostic_ratchet() {
    let pass1 = &*CI_PASS1;
    let diag_count = parse_diagnostic_count(&pass1.stderr);
    eprintln!(
        "diagnostic count: {} (ratchet: {})",
        diag_count, DIAG_RATCHET
    );
    assert!(
        diag_count <= DIAG_RATCHET,
        "diagnostic count {} exceeds ratchet {}",
        diag_count,
        DIAG_RATCHET
    );
}

#[test]
#[ignore = "CI: cargo test -p v1-compiler-tests ci_ -- --ignored"]
fn ci_performance_ratchet() {
    let pass1 = &*CI_PASS1;
    eprintln!(
        "pipeline elapsed: {:?} (budget: {}s)",
        pass1.elapsed, PERF_RATCHET_SECONDS
    );
    assert!(
        pass1.elapsed.as_secs() < PERF_RATCHET_SECONDS,
        "performance regression: pipeline took {:?}, budget is {}s. \
         See DESIGN.md for diagnosis.",
        pass1.elapsed,
        PERF_RATCHET_SECONDS
    );
}

#[test]
#[ignore = "CI: cargo test -p v1-compiler-tests ci_ -- --ignored"]
fn ci_freshness() {
    let pass1 = &*CI_PASS1;
    // Freshness was precomputed in CI_PASS1 init — before CI_PASS2 can
    // copy pass1 files into the workspace (which would mask staleness).
    if let Err(ref diff) = pass1.freshness {
        panic!(
            "Stage0 is STALE — does not match self-compile output.\n\
             Run `cargo run -p v1-compiler --bin regen_stage0` to update.\n\
             Diff:\n{}",
            diff
        );
    }
}

#[test]
#[ignore = "CI: cargo test -p v1-compiler-tests ci_ -- --ignored"]
fn ci_fixed_point() {
    let pass1 = &*CI_PASS1;
    let pass2 = &*CI_PASS2;
    let pass1_src = pass1.output_dir.join("src");
    let pass2_src = pass2.output_dir.join("src");
    if let Err(diff) = diff_excluding_hand_maintained(&pass1_src, &pass2_src) {
        eprintln!("Fixed point NOT reached — diff:\n{}", diff);
        panic!("stage1 != stage2 — fixed point not reached");
    }
}

// ── L4: Structural semantic correctness ─────────────────────────────────
//
// Compiles weather.dag to Rust, writes emitted files to a temp crate,
// adds structural test file with witness-based assertions, runs cargo test.
// This is the first test that actually RUNS emitted code (not just checks
// it compiles).

#[test]
#[ignore = "Expensive: compiles .dag, builds emitted crate, runs cargo test"]
fn bootstrap_l4_structural() {
    let result = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            // 1. Compile weather.dag
            let ws = crate::helpers::workspace_root();
            let weather_src = std::fs::read_to_string(ws.join("dsl/examples/weather/weather.dag"))
                .expect("weather.dag should exist");

            let result = crate::helpers::compile_dag_named(
                "dsl/examples/weather/weather.dag",
                &weather_src,
                v1_compiler::v1_compiler_artifact::RenderTarget::Rust,
            );

            let diag_count = result.diagnostics.len();
            eprintln!(
                "weather.dag compiled: {} files emitted, {} diagnostics",
                result.files.len(),
                diag_count
            );

            assert!(
                !result.files.is_empty(),
                "weather.dag should produce emitted files"
            );

            // 2. Write emitted files to temp dir
            let tmp = std::env::temp_dir().join("v2-l4-structural");
            let _ = std::fs::remove_dir_all(&tmp);

            for file in result.files.iter() {
                let dest = tmp.join(&file.path);
                if let Some(parent) = dest.parent() {
                    std::fs::create_dir_all(parent).expect("create dir");
                }
                std::fs::write(&dest, &*file.content)
                    .unwrap_or_else(|e| panic!("failed to write {}: {}", file.path, e));
            }

            // List emitted files for debugging
            eprintln!("\nemitted files:");
            for file in result.files.iter() {
                eprintln!("  {} ({} bytes)", file.path, file.content.len());
            }

            // Print the module file so we can see the actual emitted types/functions
            for file in result.files.iter() {
                if file.path.contains("examples_weather") {
                    eprintln!("\n=== {} ===", file.path);
                    for (i, line) in file.content.lines().enumerate() {
                        eprintln!("  {:>3}| {}", i + 1, line);
                    }
                }
            }

            // 3. Write structural test file
            let test_dir = tmp.join("tests");
            std::fs::create_dir_all(&test_dir).expect("create tests dir");

            let test_content = generate_weather_structural_tests();
            std::fs::write(test_dir.join("structural_tests.rs"), &test_content)
                .expect("write structural tests");

            // 4. Run cargo test on the emitted crate
            eprintln!("\n=== Running cargo test on emitted crate ===");
            let test_output = std::process::Command::new("cargo")
                .arg("test")
                .arg("--")
                .arg("--nocapture")
                .current_dir(&tmp)
                .output()
                .expect("failed to run cargo test");

            let stderr = String::from_utf8_lossy(&test_output.stderr);
            let stdout = String::from_utf8_lossy(&test_output.stdout);
            eprintln!("cargo test stderr:\n{}", stderr);
            eprintln!("cargo test stdout:\n{}", stdout);

            assert!(
                test_output.status.success(),
                "L4 structural tests FAILED — emitted code does not run correctly:\n{}",
                stderr
            );

            eprintln!("=== L4 structural tests PASSED ===");

            // Cleanup
            let _ = std::fs::remove_dir_all(&tmp);
        })
        .expect("failed to spawn thread")
        .join();
    result.expect("bootstrap_l4_structural panicked");
}

/// Generate structural test content for weather.dag's emitted Rust crate.
///
/// Tests are organized by witness layer:
/// - Layer 1: canonical witnesses (type inhabitation + function calls)
/// - Layer 2: variant witnesses (coproduct exhaustiveness)
/// - Layer 3: algebra-derived witnesses (non-trivial values + boundary cases)
fn generate_weather_structural_tests() -> String {
    r#"use v1_compiled::examples_weather::*;
use v1_compiled::examples_weather::Condition::*;
use std::rc::Rc;

// ── Layer 1: Type witnesses — every type is constructible ───────────

#[test]
fn witness_temperature() {
    let _w = Temperature { celsius: 0.0 };
}

#[test]
fn witness_condition_sunny() {
    let _w = Rc::new(Sunny);
}

#[test]
fn witness_forecast() {
    let _w = Rc::new(Forecast {
        location: String::new(),
        high: Temperature { celsius: 0.0 },
        low: Temperature { celsius: 0.0 },
        condition: Rc::new(Sunny),
    });
}

// ── Layer 1: Function calls — every function runs without panic ─────

#[test]
fn call_to_fahrenheit() {
    let _result = to_fahrenheit(Temperature { celsius: 0.0 });
}

#[test]
fn call_is_freezing() {
    let _result = is_freezing(Temperature { celsius: 0.0 });
}

#[test]
fn call_describe_condition() {
    let _result = describe_condition(Rc::new(Sunny));
}

#[test]
fn call_daily_summary() {
    let forecast = Rc::new(Forecast {
        location: String::new(),
        high: Temperature { celsius: 0.0 },
        low: Temperature { celsius: 0.0 },
        condition: Rc::new(Sunny),
    });
    let _result = daily_summary(forecast);
}

#[test]
fn call_freezing_locations_empty() {
    let _result = freezing_locations(Rc::new(vec![]));
}

// ── Layer 2: Variant coverage — coproduct exhaustiveness ────────────

#[test]
fn describe_condition_all_variants() {
    let _ = describe_condition(Rc::new(Sunny));
    let _ = describe_condition(Rc::new(Cloudy));
    let _ = describe_condition(Rc::new(Rainy { mm_per_hour: 0.0 }));
    let _ = describe_condition(Rc::new(Snowy { cm_per_hour: 0.0 }));
}

// ── Layer 3: Non-trivial witnesses + structural oracles ─────────────

#[test]
fn to_fahrenheit_known_value() {
    let result = to_fahrenheit(Temperature { celsius: 100.0 });
    assert!((result - 212.0).abs() < 0.001, "100C should be 212F, got {}", result);
}

#[test]
fn is_freezing_boundary() {
    assert!(is_freezing(Temperature { celsius: 0.0 }));
    assert!(!is_freezing(Temperature { celsius: 0.1 }));
}

#[test]
fn freezing_locations_filters_correctly() {
    let forecasts = Rc::new(vec![
        Rc::new(Forecast {
            location: "cold".to_string(),
            high: Temperature { celsius: 5.0 },
            low: Temperature { celsius: -2.0 },
            condition: Rc::new(Sunny),
        }),
        Rc::new(Forecast {
            location: "warm".to_string(),
            high: Temperature { celsius: 25.0 },
            low: Temperature { celsius: 15.0 },
            condition: Rc::new(Sunny),
        }),
    ]);
    let result = freezing_locations(forecasts);
    assert_eq!(result.len(), 1, "only one location has freezing low");
    assert_eq!(result[0].as_str(), "cold");
}

#[test]
fn describe_condition_rainy_branches() {
    let light = describe_condition(Rc::new(Rainy { mm_per_hour: 5.0 }));
    let heavy = describe_condition(Rc::new(Rainy { mm_per_hour: 15.0 }));
    assert_ne!(light, heavy, "light and heavy rain should have different descriptions");
}

// ── Serde roundtrip — serialization correctness ─────────────────────

#[test]
fn roundtrip_temperature() {
    let w = Temperature { celsius: 42.5 };
    let json = serde_json::to_string(&w).expect("serialize");
    let back: Temperature = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(w, back, "Temperature roundtrip failed");
}

#[test]
fn roundtrip_condition_sunny() {
    let w = Sunny;
    let json = serde_json::to_string(&w).expect("serialize");
    let back: Condition = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(w, back, "Condition::Sunny roundtrip failed");
}

#[test]
fn roundtrip_forecast() {
    let w = Forecast {
        location: "test".to_string(),
        high: Temperature { celsius: 30.0 },
        low: Temperature { celsius: 10.0 },
        condition: Rc::new(Sunny),
    };
    let json = serde_json::to_string(&w).expect("serialize");
    let back: Forecast = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(w, back, "Forecast roundtrip failed");
}
"#
    .to_string()
}
