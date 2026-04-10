//! Bootstrap tests for the v2 self-hosted compiler.
//!
//! These are subprocess tests that build and run the stage0 binary.
//! Most are `#[ignore]` because they require building the stage0 binary (~2 min).

#![allow(clippy::disallowed_macros)]

// ── Helper: return the real workspace source roots ──
//
// The compiler reads .dag files read-only via --source-root, so tests
// point directly at the workspace tree instead of copying files into
// a temp dir. This eliminates the FF-9 curated-file-list workaround
// and ensures bootstrap tests always see the same sources as the real
// compiler.

fn source_roots() -> (std::path::PathBuf, std::path::PathBuf) {
    let ws = crate::helpers::workspace_root();
    (ws.join("src/v2"), ws.join("dsl"))
}

// ── 1. stage0_cargo_check ───────────────────────────────────────────────

#[test]
fn stage0_cargo_check() {
    // Stage0 is now a workspace member — just cargo check it
    let output = std::process::Command::new("cargo")
        .arg("check")
        .arg("-p")
        .arg("v2-compiler")
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
//   bounded iteration (I1/I2 in ROADMAP Exploratory Directions).
//
// These are analyzer limitations, not program violations. INVARIANTS.md
// §Decidability: "If the analyzer produces ?O(?), the bug is in the
// analyzer (it cannot see the bound that structurally exists), not in
// the program." The ratchet only moves down, never up.
// 2026-04-07: 526 — honest count after restoring CostUnknown for all
//   unresolved descent patterns. See docs/cx-violation-triage.md for
//   the 3-fix reduction path (Node tree descent, Parser SCC, Graph DFS).
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
const DIAG_RATCHET: usize = 472;

#[test]
#[ignore] // Requires building stage0 binary (~2 min)
fn strict_compile_diagnostic_count() {
    // Build stage0 binary
    let build = std::process::Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("v2-compiler")
        .arg("--release")
        .output()
        .expect("failed to build stage0");
    assert!(
        build.status.success(),
        "stage0 build failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );

    // Find the binary
    let ws = crate::helpers::workspace_root();
    let stage0_bin = ws.join("target/release/v2-compiler");
    assert!(
        stage0_bin.exists(),
        "stage0 binary not found at {}",
        stage0_bin.display()
    );

    let (v2_root, dsl_root) = source_roots();

    let out_dir = std::env::temp_dir().join("v2-diag-output");
    let _ = std::fs::remove_dir_all(&out_dir);

    let output = std::process::Command::new(&stage0_bin)
        .arg("compile")
        .arg("--source-root")
        .arg(&v2_root)
        .arg("--source-root")
        .arg(&dsl_root)
        .arg("--output-dir")
        .arg(&out_dir)
        .output()
        .expect("failed to run stage0 compile");

    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("stage0 compile stderr:\n{}", stderr);

    // Parse diagnostic count from stderr: "compiled: N files emitted, M diagnostics"
    let diag_count = stderr
        .lines()
        .find(|l| l.contains("compiled:") && l.contains("diagnostics"))
        .and_then(|l| {
            l.split("diagnostics")
                .next()
                .and_then(|prefix| prefix.split(',').next_back())
                .and_then(|s| s.trim().parse::<usize>().ok())
        })
        .unwrap_or(usize::MAX);

    assert!(
        diag_count <= DIAG_RATCHET,
        "diagnostic count {} exceeds ratchet {}",
        diag_count, DIAG_RATCHET
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
#[ignore] // Requires building stage0 binary (~2 min)
fn stage0_compile_accepts_dag_target() {
    let build = std::process::Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("v2-compiler")
        .arg("--release")
        .output()
        .expect("failed to build stage0");
    assert!(
        build.status.success(),
        "stage0 build failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );

    let ws = crate::helpers::workspace_root();
    let stage0_bin = ws.join("target/release/v2-compiler");
    assert!(
        stage0_bin.exists(),
        "stage0 binary not found at {}",
        stage0_bin.display()
    );

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
#[ignore] // Expensive: builds binary + runs full compile + cargo check
fn bootstrap_stage0_to_stage1() {
    // Build stage0 binary from committed seed
    let build = std::process::Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("v2-compiler")
        .arg("--release")
        .env("CARGO_BUILD_JOBS", "2")
        .output()
        .expect("failed to build stage0");
    assert!(
        build.status.success(),
        "stage0 build failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );

    let ws = crate::helpers::workspace_root();
    let stage0_bin = ws.join("target/release/v2-compiler");
    let (v2_root, dsl_root) = source_roots();

    // Run stage0 to compile stage1
    let stage1_dir = std::env::temp_dir().join("v2-bootstrap-stage1");
    let _ = std::fs::remove_dir_all(&stage1_dir);
    let output = std::process::Command::new(&stage0_bin)
        .arg("compile")
        .arg("--source-root")
        .arg(&v2_root)
        .arg("--source-root")
        .arg(&dsl_root)
        .arg("--output-dir")
        .arg(&stage1_dir)
        .output()
        .expect("failed to run stage0 compile");
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

    let check = std::process::Command::new("cargo")
        .arg("check")
        .current_dir(&stage1_dir)
        .output()
        .expect("failed to cargo check stage1");
    let check_stderr = String::from_utf8_lossy(&check.stderr);
    // Count both coded errors (error[Exxxx]) and uncoded errors (error: ...)
    // to avoid silently passing on parse/syntax failures.
    let error_count = check_stderr.lines()
        .filter(|l| l.starts_with("error[") || (l.starts_with("error") && !l.starts_with("error:")))
        .count();
    // Fall back: if cargo check failed but we counted 0 errors, something
    // uncategorized went wrong — don't silently pass.
    let error_count = if !check.status.success() && error_count == 0 {
        eprintln!("cargo check failed with uncategorized errors:\n{}", check_stderr);
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
    eprintln!("stage1 cargo check: {} errors (ratchet: {})", error_count, EMITTED_RUST_ERROR_RATCHET);
    for (code, count) in cats.iter().take(10) {
        eprintln!("  {}: {}", code, count);
    }
    // Show samples for top 3 categories
    for (code, _) in cats.iter().take(3) {
        let needle = code.trim_end_matches(']').trim_start_matches("error[");
        let samples: Vec<&str> = check_stderr.lines()
            .filter(|l| l.starts_with(&format!("error[{}]", needle)))
            .take(2)
            .collect();
        for s in samples {
            eprintln!("  {}", &s[..s.len().min(200)]);
        }
    }
    eprintln!("\n=== FULL CARGO CHECK STDERR ===\n{}\n=== END ===", check_stderr);

    assert!(
        error_count <= EMITTED_RUST_ERROR_RATCHET,
        "emitted Rust errors {} exceeds ratchet {} — \
         fix codegen or update EMITTED_RUST_ERROR_RATCHET if increase is justified",
        error_count, EMITTED_RUST_ERROR_RATCHET
    );

    let _ = std::fs::remove_dir_all(&stage1_dir);
}

// ── 4. bootstrap_fixed_point ────────────────────────────────────────────

#[test]
#[ignore] // Expensive: builds two binaries + two full compiles
fn bootstrap_fixed_point() {
    let ws = crate::helpers::workspace_root();

    // Build stage0
    let build = std::process::Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("v2-compiler")
        .arg("--release")
        .env("CARGO_BUILD_JOBS", "2")
        .output()
        .expect("failed to build stage0");
    assert!(
        build.status.success(),
        "stage0 build failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let stage0_bin = ws.join("target/release/v2-compiler");
    let (v2_root, dsl_root) = source_roots();

    // Stage0 -> stage1
    let stage1_dir = std::env::temp_dir().join("v2-fp-stage1");
    let _ = std::fs::remove_dir_all(&stage1_dir);
    let s1 = std::process::Command::new(&stage0_bin)
        .arg("compile")
        .arg("--source-root")
        .arg(&v2_root)
        .arg("--source-root")
        .arg(&dsl_root)
        .arg("--output-dir")
        .arg(&stage1_dir)
        .output()
        .expect("stage0 compile failed");
    assert!(
        s1.status.success(),
        "stage0->1 failed:\n{}",
        String::from_utf8_lossy(&s1.stderr)
    );

    // Build stage1 binary
    let build1 = std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .env("CARGO_BUILD_JOBS", "2")
        .current_dir(&stage1_dir)
        .output()
        .expect("stage1 build failed");
    assert!(
        build1.status.success(),
        "stage1 build failed:\n{}",
        String::from_utf8_lossy(&build1.stderr)
    );
    // Self-compile includes v2.compiler.compile, so emit_cargo_toml picks
    // crate_name "v2_compiler" (see 05_emit_rust.dag line 452).
    let stage1_bin = stage1_dir.join("target/release/v2_compiler");

    // Stage1 -> stage2
    let stage2_dir = std::env::temp_dir().join("v2-fp-stage2");
    let _ = std::fs::remove_dir_all(&stage2_dir);
    let s2 = std::process::Command::new(&stage1_bin)
        .arg("compile")
        .arg("--source-root")
        .arg(&v2_root)
        .arg("--source-root")
        .arg(&dsl_root)
        .arg("--output-dir")
        .arg(&stage2_dir)
        .output()
        .expect("stage1 compile failed");
    assert!(
        s2.status.success(),
        "stage1->2 failed:\n{}",
        String::from_utf8_lossy(&s2.stderr)
    );

    // Compare stage1 and stage2 source output
    let stage1_src = stage1_dir.join("src");
    let stage2_src = stage2_dir.join("src");

    let diff = std::process::Command::new("diff")
        .arg("-r")
        .arg(&stage1_src)
        .arg(&stage2_src)
        .output()
        .expect("diff failed");

    // diff exits 0 (identical), 1 (different), 2 (error).
    // Check for errors first, then differences.
    assert!(
        diff.status.code() != Some(2),
        "diff -r failed (exit 2):\n{}",
        String::from_utf8_lossy(&diff.stderr)
    );
    if !diff.stdout.is_empty() {
        eprintln!(
            "Fixed point NOT reached — diff:\n{}",
            String::from_utf8_lossy(&diff.stdout)
        );
    }
    assert!(
        diff.status.success(),
        "stage1 != stage2 — fixed point not reached"
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&stage1_dir);
    let _ = std::fs::remove_dir_all(&stage2_dir);
}

// ── 5. gist_full_pipeline ──────────────────────────────────────────────

#[test]
#[ignore] // Requires building stage0 binary (~2 min)
fn gist_full_pipeline() {
    let gist_files = [
        "dsl/std/types.dag",
        "dsl/std/errors.dag",
        "dsl/std/resources.dag",
        "dsl/extdeps/cloud/cloud.dag",
        "dsl/extdeps/cloud/gcp/gcp.dag",
        "dsl/extdeps/github/github.dag",
        "dsl/extdeps/github/auth.dag",
        "dsl/extdeps/github/gists.dag",
        "dsl/extdeps/git.dag",
        "dsl/gunbc/auth/credentials.dag",
        "dsl/gunbc/tools/gist.dag",
    ];

    // Build stage0
    let build = std::process::Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("v2-compiler")
        .arg("--release")
        .env("CARGO_BUILD_JOBS", "2")
        .output()
        .expect("failed to build stage0");
    assert!(
        build.status.success(),
        "stage0 build failed:\n{}",
        String::from_utf8_lossy(&build.stderr)
    );

    let ws = crate::helpers::workspace_root();
    let stage0_bin = ws.join("target/release/v2-compiler");

    let out_dir = std::env::temp_dir().join("v2-gist-pipeline-out");
    let _ = std::fs::create_dir_all(&out_dir);

    let source_dir = std::env::temp_dir().join("v2-gist-pipeline-src");
    let _ = std::fs::create_dir_all(&source_dir);
    for rel_path in &gist_files {
        let src = ws.join(rel_path);
        let dst = source_dir.join(rel_path);
        let _ = std::fs::create_dir_all(dst.parent().unwrap());
        std::fs::copy(&src, &dst)
            .unwrap_or_else(|e| panic!("failed to copy {}: {}", src.display(), e));
    }

    let output = std::process::Command::new(&stage0_bin)
        .arg("compile")
        .arg("--source-root")
        .arg(&source_dir)
        .arg("--output-dir")
        .arg(&out_dir)
        .output()
        .expect("stage0 compile should run");

    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("gist pipeline stderr:\n{}", stderr);
    assert!(output.status.success(), "stage0 compile failed");

    // Verify Cargo.toml exists in output
    let cargo_toml = out_dir.join("Cargo.toml");
    assert!(
        cargo_toml.exists(),
        "no Cargo.toml in emitted gist output"
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&source_dir);
    let _ = std::fs::remove_dir_all(&out_dir);
}

// ── 6. performance_ratchet ───────────────────────────────────────────────

/// Performance ratchet: self-compile pipeline must complete within the
/// time budget. Catches FF-class regressions (new O(n²) patterns,
/// lost facts, unnecessary allocations).
///
/// The ratchet is generous (30s) to avoid flaky failures from system load.
/// The actual pipeline time is ~6.5s in release mode. If this test fails,
/// a structural performance regression has been introduced.
const PERF_RATCHET_SECONDS: u64 = 30;

#[test]
#[ignore] // Requires building stage0 binary
fn performance_ratchet() {
    let ws = crate::helpers::workspace_root();

    // Build stage0
    let build = std::process::Command::new("cargo")
        .arg("build")
        .arg("-p")
        .arg("v2-compiler")
        .arg("--release")
        .output()
        .expect("failed to build stage0");
    assert!(build.status.success(), "stage0 build failed");

    let stage0_bin = ws.join("target/release/v2-compiler");
    let (v2_root, dsl_root) = source_roots();

    let out_dir = std::env::temp_dir().join("v2-perf-output");
    let _ = std::fs::remove_dir_all(&out_dir);

    // Time the pipeline
    let start = std::time::Instant::now();
    let output = std::process::Command::new(&stage0_bin)
        .arg("compile")
        .arg("--source-root")
        .arg(&v2_root)
        .arg("--source-root")
        .arg(&dsl_root)
        .arg("--output-dir")
        .arg(&out_dir)
        .output()
        .expect("failed to run stage0 compile");
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
         See INVARIANTS.md 'Facts Flow Forward' for diagnosis.",
        elapsed,
        PERF_RATCHET_SECONDS
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&out_dir);
}

// ── L4: Structural semantic correctness ─────────────────────────────────
//
// Compiles weather.dag to Rust, writes emitted files to a temp crate,
// adds structural test file with witness-based assertions, runs cargo test.
// This is the first test that actually RUNS emitted code (not just checks
// it compiles).

#[test]
#[ignore] // Expensive: compiles .dag, builds emitted crate, runs cargo test
fn bootstrap_l4_structural() {
    let result = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            // 1. Compile weather.dag
            let ws = crate::helpers::workspace_root();
            let weather_src = std::fs::read_to_string(
                ws.join("dsl/examples/weather/weather.dag"),
            )
            .expect("weather.dag should exist");

            let result = crate::helpers::compile_dag_named(
                "dsl/examples/weather/weather.dag",
                &weather_src,
                v2_compiler::v2_compiler_artifact::RenderTarget::Rust,
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
    r#"use v2_compiled::examples_weather::*;
use v2_compiled::examples_weather::Condition::*;
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
