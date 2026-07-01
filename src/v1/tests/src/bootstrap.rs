#![allow(clippy::disallowed_macros)]

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

fn run_self_compile(
    binary: &std::path::Path,
    output_dir: &std::path::Path,
) -> std::process::Output {
    run_self_compile_with_extra_source_roots(binary, output_dir, &[])
}

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

fn copy_dir_recursive_bootstrap(source: &std::path::Path, dest: &std::path::Path) {
    std::fs::create_dir_all(dest)
        .unwrap_or_else(|e| panic!("failed to create {}: {}", dest.display(), e));
    for entry in std::fs::read_dir(source)
        .unwrap_or_else(|e| panic!("failed to read dir {}: {}", source.display(), e))
    {
        let entry = entry.unwrap_or_else(|e| panic!("failed to read dir entry: {}", e));
        let src_path = entry.path();
        let dst_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive_bootstrap(&src_path, &dst_path);
        } else {
            std::fs::copy(&src_path, &dst_path).unwrap_or_else(|e| {
                panic!(
                    "failed to copy {} -> {}: {}",
                    src_path.display(),
                    dst_path.display(),
                    e
                )
            });
        }
    }
}

fn copy_stage0_support_modules(stage1_dir: &std::path::Path, ws: &std::path::Path) {
    let stage0_src = ws.join("src/v1/stage0/src");
    for name in &[
        "v1_interpreter.rs",
        "cli_run.rs",
        "rest_transport_facts.rs",
        "wire_value_serialize.rs",
        "coproduct_reflection.rs",
        "resolved_graph_cache.rs",
        "recorded_fixture.rs",
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
    let mpi_src = stage0_src.join("module_path_index");
    if mpi_src.is_dir() {
        copy_dir_recursive_bootstrap(&mpi_src, &stage1_dir.join("src/module_path_index"));
    }
}

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
        .arg("--exclude=module_path_index")
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

#[test]
fn stage0_cargo_check() {
    // Nested `cargo check` inherits RUSTC_WRAPPER=sccache from CI. Under the
    // full nextest parallel load, sccache can fail with exit 254 without a
    // rustc diagnostic — mirror the CI release-build retry discipline.
    let output = std::process::Command::new("cargo")
        .env_remove("RUSTC_WRAPPER")
        .env("CARGO_BUILD_JOBS", "1")
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

const DIAG_RATCHET: usize = 358;

#[test]
#[ignore = "Requires building stage0 binary (~2 min)"]
fn strict_compile_diagnostic_count() {
    let stage0_bin = find_or_build_stage0();

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
    let stage0_bin = find_or_build_stage0();

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
    let stage0_bin = find_or_build_stage0();

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

const EMITTED_RUST_ERROR_RATCHET: usize = 0;

#[test]
#[ignore = "Expensive: builds binary + runs full compile + cargo check"]
fn bootstrap_stage0_to_stage1() {
    let stage0_bin = find_or_build_stage0();
    let ws = crate::helpers::workspace_root();

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
    let error_count = check_stderr
        .lines()
        .filter(|l| l.starts_with("error[") || (l.starts_with("error") && !l.starts_with("error:")))
        .count();
    let error_count = if !check.status.success() && error_count == 0 {
        eprintln!(
            "cargo check failed with uncategorized errors:\n{}",
            check_stderr
        );
        usize::MAX
    } else {
        error_count
    };
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

#[test]
#[ignore = "Expensive: builds two binaries + two full compiles"]
fn bootstrap_fixed_point() {
    let ws = crate::helpers::workspace_root();
    let stage0_bin = find_or_build_stage0();

    let stage1_dir = std::env::temp_dir().join("v2-fp-stage1");
    let _ = std::fs::remove_dir_all(&stage1_dir);
    let s1 = run_self_compile(&stage0_bin, &stage1_dir);
    assert!(
        s1.status.success(),
        "stage0->1 failed:\n{}",
        String::from_utf8_lossy(&s1.stderr)
    );

    prepare_stage1_for_build(&stage1_dir, &ws);

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
    let stage1_bin = stage1_dir.join("target/release/v1_compiler");

    let stage2_dir = std::env::temp_dir().join("v2-fp-stage2");
    let _ = std::fs::remove_dir_all(&stage2_dir);
    let s2 = run_self_compile(&stage1_bin, &stage2_dir);
    assert!(
        s2.status.success(),
        "stage1->2 failed:\n{}",
        String::from_utf8_lossy(&s2.stderr)
    );

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

    let _ = std::fs::remove_dir_all(&stage1_dir);
    let _ = std::fs::remove_dir_all(&stage2_dir);
}

const PERF_RATCHET_SECONDS: u64 = 150;

#[test]
#[ignore = "Requires building stage0 binary"]
fn performance_ratchet() {
    let stage0_bin = find_or_build_stage0();

    let out_dir = std::env::temp_dir().join("v2-perf-output");
    let _ = std::fs::remove_dir_all(&out_dir);

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

use std::sync::LazyLock;

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

struct Pass1Output {
    output_dir: std::path::PathBuf,
    stderr: String,
    elapsed: std::time::Duration,
    freshness: Result<(), String>,
}

struct Pass2Output {
    output_dir: std::path::PathBuf,
}

fn prebuilt_stage0_path() -> std::path::PathBuf {
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| crate::helpers::workspace_root().join("target"));
    target_dir.join("release/gunbc")
}

fn find_or_build_stage0() -> std::path::PathBuf {
    let bin = prebuilt_stage0_path();
    if bin.exists() {
        ci_timing("stage0 binary found (skipping rebuild)");
        bin
    } else {
        ci_timing("stage0 binary not found, building");
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

    prepare_stage1_for_build(&output_dir, &ws);

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

#[test]
#[ignore = "Expensive: compiles .dag, builds emitted crate, runs cargo test"]
fn bootstrap_l4_structural() {
    let result = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
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

            eprintln!("\nemitted files:");
            for file in result.files.iter() {
                eprintln!("  {} ({} bytes)", file.path, file.content.len());
            }

            for file in result.files.iter() {
                if file.path.contains("examples_weather") {
                    eprintln!("\n=== {} ===", file.path);
                    for (i, line) in file.content.lines().enumerate() {
                        eprintln!("  {:>3}| {}", i + 1, line);
                    }
                }
            }

            let test_dir = tmp.join("tests");
            std::fs::create_dir_all(&test_dir).expect("create tests dir");

            let test_content = generate_weather_structural_tests();
            std::fs::write(test_dir.join("structural_tests.rs"), &test_content)
                .expect("write structural tests");

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

            let _ = std::fs::remove_dir_all(&tmp);
        })
        .expect("failed to spawn thread")
        .join();
    result.expect("bootstrap_l4_structural panicked");
}

fn generate_weather_structural_tests() -> String {
    r#"use v1_compiled::examples_weather::*;
use v1_compiled::examples_weather::Condition::*;
use std::rc::Rc;


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


#[test]
fn describe_condition_all_variants() {
    let _ = describe_condition(Rc::new(Sunny));
    let _ = describe_condition(Rc::new(Cloudy));
    let _ = describe_condition(Rc::new(Rainy { mm_per_hour: 0.0 }));
    let _ = describe_condition(Rc::new(Snowy { cm_per_hour: 0.0 }));
}


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
