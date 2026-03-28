//! Bootstrap tests for the v2 self-hosted compiler.
//!
//! These are subprocess tests that build and run the stage0 binary.
//! Most are `#[ignore]` because they require building the stage0 binary (~2 min).

#![allow(clippy::disallowed_macros)]

// ── Helper: copy .dag sources into a temp directory ─────────────────────

fn prepare_sources(sources_dir: &std::path::Path) {
    let ws = crate::helpers::workspace_root();

    // Copy v2 compiler .dag files
    for entry in std::fs::read_dir(ws.join("src/v2")).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map(|e| e == "dag").unwrap_or(false) {
            std::fs::copy(&path, sources_dir.join(entry.file_name())).unwrap();
        }
    }

    // Copy language extdeps
    let lang_dirs = ["rust", "python", "go"];
    for lang in &lang_dirs {
        let src = ws.join(format!("dsl/extdeps/languages/{}/emit.dag", lang));
        if src.exists() {
            let dst_dir = sources_dir.join(format!("dsl/extdeps/languages/{}", lang));
            std::fs::create_dir_all(&dst_dir).unwrap();
            std::fs::copy(&src, dst_dir.join("emit.dag")).unwrap();
        }
    }

    // Copy std modules that the v2 parser can handle.
    // logic.dag, bit.dag, integer.dag, float.dag use v1-era where syntax
    // (field-level where, width(), unsigned, etc.) that the v2 parser doesn't
    // support yet. These will be loadable once integer/float types compose
    // from algebra.dag generic types instead of using where labels (Part B).
    let dst_dir = sources_dir.join("dsl/std");
    std::fs::create_dir_all(&dst_dir).unwrap();
    let std_files = [
        "constructors",
        "types", "algebra", "containers",
        "logic", "bit", "integer", "float", "string_type",
        "encoding",
    ];
    for name in &std_files {
        let src = ws.join(format!("dsl/std/{}.dag", name));
        if src.exists() {
            std::fs::copy(&src, dst_dir.join(format!("{}.dag", name))).unwrap();
        }
    }
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

const DIAG_RATCHET: usize = 0;

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

    // Prepare source directory with all .dag files
    let sources_dir = std::env::temp_dir().join("v2-diag-sources");
    let _ = std::fs::remove_dir_all(&sources_dir);
    std::fs::create_dir_all(&sources_dir).unwrap();
    prepare_sources(&sources_dir);

    let out_dir = std::env::temp_dir().join("v2-diag-output");
    let _ = std::fs::remove_dir_all(&out_dir);

    let output = std::process::Command::new(&stage0_bin)
        .arg("compile")
        .arg("--source-dir")
        .arg(&sources_dir)
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
    let _ = std::fs::remove_dir_all(&sources_dir);
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
        .arg("--source-dir")
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
const EMITTED_RUST_ERROR_RATCHET: usize = 1200;

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

    // Prepare source directory
    let sources_dir = std::env::temp_dir().join("v2-bootstrap-sources");
    let _ = std::fs::remove_dir_all(&sources_dir);
    std::fs::create_dir_all(&sources_dir).unwrap();
    prepare_sources(&sources_dir);

    // Run stage0 to compile stage1
    let stage1_dir = std::env::temp_dir().join("v2-bootstrap-stage1");
    let _ = std::fs::remove_dir_all(&stage1_dir);
    let output = std::process::Command::new(&stage0_bin)
        .arg("compile")
        .arg("--source-dir")
        .arg(&sources_dir)
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

    // Cargo check stage1 — count errors as ratchet.
    // Missing output is a hard failure, not a silent pass.
    assert!(
        stage1_dir.join("Cargo.toml").exists(),
        "stage0 produced no Cargo.toml — bootstrap is broken"
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
    eprintln!("stage1 cargo check: {} errors (ratchet: {})", error_count, EMITTED_RUST_ERROR_RATCHET);

    assert!(
        error_count <= EMITTED_RUST_ERROR_RATCHET,
        "emitted Rust errors {} exceeds ratchet {} — \
         fix codegen or update EMITTED_RUST_ERROR_RATCHET if increase is justified",
        error_count, EMITTED_RUST_ERROR_RATCHET
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&sources_dir);
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

    // Prepare sources
    let sources_dir = std::env::temp_dir().join("v2-fp-sources");
    let _ = std::fs::remove_dir_all(&sources_dir);
    std::fs::create_dir_all(&sources_dir).unwrap();
    prepare_sources(&sources_dir);

    // Stage0 -> stage1
    let stage1_dir = std::env::temp_dir().join("v2-fp-stage1");
    let _ = std::fs::remove_dir_all(&stage1_dir);
    let s1 = std::process::Command::new(&stage0_bin)
        .arg("compile")
        .arg("--source-dir")
        .arg(&sources_dir)
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
    // Stage1 emits crate name "v2_compiled" (see 05_emit_rust.dag emit_cargo_toml),
    // so the binary is v2_compiled, not v2-compiler.
    let stage1_bin = stage1_dir.join("target/release/v2_compiled");

    // Stage1 -> stage2
    let stage2_dir = std::env::temp_dir().join("v2-fp-stage2");
    let _ = std::fs::remove_dir_all(&stage2_dir);
    let s2 = std::process::Command::new(&stage1_bin)
        .arg("compile")
        .arg("--source-dir")
        .arg(&sources_dir)
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
    let _ = std::fs::remove_dir_all(&sources_dir);
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
        .arg("--source-dir")
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

    // Prepare sources
    let sources_dir = std::env::temp_dir().join("v2-perf-sources");
    let _ = std::fs::remove_dir_all(&sources_dir);
    std::fs::create_dir_all(&sources_dir).unwrap();
    prepare_sources(&sources_dir);

    let out_dir = std::env::temp_dir().join("v2-perf-output");
    let _ = std::fs::remove_dir_all(&out_dir);

    // Time the pipeline
    let start = std::time::Instant::now();
    let output = std::process::Command::new(&stage0_bin)
        .arg("compile")
        .arg("--source-dir")
        .arg(&sources_dir)
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
    let _ = std::fs::remove_dir_all(&sources_dir);
    let _ = std::fs::remove_dir_all(&out_dir);
}
