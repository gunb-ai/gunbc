#![allow(clippy::disallowed_macros)]

use im_rc::HashMap;
use std::process::ExitCode;
use std::rc::Rc;

use v1_compiler::cli_run::workspace_root;
use v1_compiler::v1_compiler_artifact::RenderTarget;
use v1_compiler::v1_compiler_compile::{compile_sources, PipelineResult, SourceFile};

type WitnessCase = (&'static str, fn());

fn fail(msg: impl std::fmt::Display) -> ExitCode {
    eprintln!("bootstrap_witness: {msg}");
    ExitCode::from(1)
}

fn source_roots() -> [std::path::PathBuf; 2] {
    let ws = workspace_root();
    [ws.join("src/v1"), ws.join("dag")]
}

fn cargo_infra_failure_transient(stderr: &str) -> bool {
    stderr.contains("couldn't create a temp dir")
        || stderr.contains("Resource temporarily unavailable")
        || stderr.contains("failed to spawn")
        || stderr.contains("sccache: encountered fatal error")
}

fn run_cargo_with_infra_retry<F>(build: F) -> std::process::Output
where
    F: Fn() -> std::process::Command,
{
    let first = build().output().expect("failed to spawn cargo");
    if first.status.success()
        || !cargo_infra_failure_transient(&String::from_utf8_lossy(&first.stderr))
    {
        return first;
    }

    let mut retry = build();
    retry.env("CARGO_BUILD_JOBS", "1");
    let second = retry.output().expect("failed to spawn cargo retry");
    if second.status.success()
        || !cargo_infra_failure_transient(&String::from_utf8_lossy(&second.stderr))
    {
        return second;
    }

    let mut cold = build();
    cold.env_remove("RUSTC_WRAPPER");
    cold.env("CARGO_BUILD_JOBS", "1");
    cold.output().expect("failed to spawn cargo cold retry")
}

fn tokenize_for_parse(source: &str) -> Rc<Vec<Rc<v1_compiler::v1_std_core::Token>>> {
    v1_compiler::v1_compiler_tokenize::tokenize(source.to_string(), "test.dag".to_string())
}

fn parse_source(source: &str) -> Rc<v1_compiler::v1_compiler_parse::ParseResult> {
    let tokens = tokenize_for_parse(source);
    let source_index =
        v1_compiler::v1_std_core::build_newline_index("test.dag".to_string(), source.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert("test.dag".to_string(), source_index);
    v1_compiler::v1_compiler_parse::parse(tokens, Rc::new(source_indices))
}

fn build_module_index_for_roots(
    roots: &[std::path::PathBuf],
) -> HashMap<String, std::path::PathBuf> {
    let mut index = HashMap::new();
    for root in roots {
        if root.exists() {
            scan_dag_files(root, &mut index);
        }
    }
    index
}

fn build_module_index() -> HashMap<String, std::path::PathBuf> {
    build_module_index_for_roots(&source_roots())
}

fn scan_dag_files(dir: &std::path::Path, index: &mut HashMap<String, std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dag_files(&path, index);
        } else if path.extension().map(|e| e == "dag").unwrap_or(false) {
            if let Some(module_path) = extract_module_declaration(&path) {
                index.insert(module_path, path);
            }
        }
    }
}

fn extract_module_declaration(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        return trimmed
            .strip_prefix("module ")
            .and_then(|rest| rest.split_whitespace().next())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
    }
    None
}

fn extract_imports(source: &str) -> Vec<String> {
    let result = parse_source(source);
    match &result.module {
        Some(module) => v1_compiler::v1_std_core::module_imports(module.clone())
            .iter()
            .map(|imp| imp.name.clone())
            .collect(),
        None => vec![],
    }
}

fn resolve_imports_transitively_with_index(
    entry_path: &str,
    entry_content: &str,
    module_index: &HashMap<String, std::path::PathBuf>,
) -> Vec<Rc<SourceFile>> {
    let ws = workspace_root();
    let mut seen: HashMap<String, Rc<SourceFile>> = HashMap::new();
    let mut queue: Vec<(String, String)> = Vec::new();

    queue.push((entry_path.to_string(), entry_content.to_string()));

    while let Some((_path, content)) = queue.pop() {
        let imports = extract_imports(&content);
        for module_path in imports {
            if seen.contains_key(&module_path) {
                continue;
            }
            if let Some(file_path) = module_index.get(&module_path) {
                if let Ok(file_content) = std::fs::read_to_string(file_path) {
                    let rel_path = file_path
                        .strip_prefix(&ws)
                        .unwrap_or(file_path)
                        .to_string_lossy()
                        .to_string();
                    let source = Rc::new(SourceFile {
                        path: rel_path.clone(),
                        content: file_content.clone(),
                    });
                    seen.insert(module_path.clone(), source);
                    queue.push((rel_path, file_content));
                }
            }
        }
    }

    let mut sources: Vec<Rc<SourceFile>> = seen.into_iter().map(|(_, v)| v).collect();
    sources.push(Rc::new(SourceFile {
        path: entry_path.to_string(),
        content: entry_content.to_string(),
    }));
    sources
}

fn resolve_imports_transitively(entry_path: &str, entry_content: &str) -> Vec<Rc<SourceFile>> {
    resolve_imports_transitively_with_index(entry_path, entry_content, &build_module_index())
}

fn compile_dag_named(filename: &str, source: &str, target: RenderTarget) -> Rc<PipelineResult> {
    let sources = resolve_imports_transitively(filename, source);
    compile_sources(Rc::new(sources), target)
}

fn diagnostic_messages(result: &PipelineResult) -> Vec<String> {
    result
        .diagnostics
        .iter()
        .map(|d| v1_compiler::v1_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

fn collect_dag_sources(
    root: &std::path::Path,
    dir: &std::path::Path,
    sources: &mut Vec<Rc<SourceFile>>,
) {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", dir.display(), e))
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_dag_sources(root, &path, sources);
        } else if path.extension().map(|e| e == "dag").unwrap_or(false) {
            let content = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            sources.push(Rc::new(SourceFile { path: rel, content }));
        }
    }
}

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
        .unwrap_or_else(|| workspace_root().join("target"));
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
    let [v1_root, dag_root] = source_roots();
    let mut command = std::process::Command::new(binary);
    command
        .arg("compile")
        .arg("--source-root")
        .arg(&v1_root)
        .arg("--source-root")
        .arg(&dag_root);
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
        "main.rs",
        "coproduct_reflection.rs",
        "resolved_graph_cache.rs",
        "recorded_fixture.rs",
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
        .arg("--exclude=main.rs")
        .arg("--exclude=coproduct_reflection.rs")
        .arg("--exclude=resolved_graph_cache.rs")
        .arg("--exclude=recorded_fixture.rs")
        .arg("--exclude=module_path_index")
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

fn stage0_cargo_check() {
    let output = run_cargo_with_infra_retry(|| {
        let mut cmd = std::process::Command::new("cargo");
        cmd.arg("check")
            .arg("-p")
            .arg("v1-compiler")
            .current_dir(workspace_root());
        cmd
    });
    assert!(
        output.status.success(),
        "stage0 cargo check failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Fast in-process floor keystone — no nested `cargo` subprocess (nested check
/// during claim_executor floor blew CI budget; stage0_cargo_check lives in expensive).
fn floor_smoke() {
    let ws = workspace_root();
    assert!(
        ws.join("src/v1/stage0/Cargo.toml").is_file(),
        "stage0 Cargo.toml missing under {}",
        ws.display()
    );
    assert!(
        ws.join("dag").is_dir(),
        "dag/ missing under {}",
        ws.display()
    );
    let [v1_root, dag_root] = source_roots();
    assert!(
        v1_root.is_dir(),
        "v1 source root missing: {}",
        v1_root.display()
    );
    assert!(
        dag_root.is_dir(),
        "dag source root missing: {}",
        dag_root.display()
    );
}

const DIAG_RATCHET: usize = 358;

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
        workspace_root().join("src/generated/method_template_projection.dag");
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

fn bootstrap_stage0_to_stage1() {
    let stage0_bin = find_or_build_stage0();
    let ws = workspace_root();

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

    // The ratchet is a tunable ceiling that currently sits at its minimum (0), which
    // makes `<=` degenerate; keep the ratchet comparison rather than hardcoding `== 0`
    // so raising the ceiling stays a one-constant edit.
    #[allow(clippy::absurd_extreme_comparisons)]
    {
        assert!(
            error_count <= EMITTED_RUST_ERROR_RATCHET,
            "emitted Rust errors {} exceeds ratchet {} — \
             fix codegen or update EMITTED_RUST_ERROR_RATCHET if increase is justified",
            error_count,
            EMITTED_RUST_ERROR_RATCHET
        );
    }

    let _ = std::fs::remove_dir_all(&stage1_dir);
}

fn bootstrap_fixed_point() {
    let ws = workspace_root();
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
        .unwrap_or_else(|| workspace_root().join("target"));
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
    let ws = workspace_root();

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
    let ws = workspace_root();
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

fn ci_full_dag() {
    ci_timing("ci_full_dag: start");
    let ws = workspace_root();
    let dag_dir = ws.join("dag");
    let mut dag_sources: Vec<Rc<SourceFile>> = Vec::new();
    collect_dag_sources(&ws, &dag_dir, &mut dag_sources);

    assert!(
        !dag_sources.is_empty(),
        "no .dag files found in dag/ — something is wrong"
    );

    let dag_result = v1_compiler::v1_compiler_compile::compile_sources(
        std::rc::Rc::new(dag_sources.clone()),
        v1_compiler::v1_compiler_artifact::RenderTarget::Rust,
    );

    let hard_diags: Vec<_> = diagnostic_messages(&dag_result)
        .into_iter()
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        hard_diags.is_empty(),
        "dag/ compilation produced {} hard diagnostics (expected 0):\n{}",
        hard_diags.len(),
        hard_diags
            .iter()
            .enumerate()
            .map(|(i, m)| format!("  [{}] {}", i, m))
            .collect::<Vec<_>>()
            .join("\n")
    );

    ci_timing(&format!("ci_full_dag: done ({} files)", dag_sources.len()));
}

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

fn bootstrap_l4_structural() {
    let result = std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let ws = workspace_root();
            let weather_src = std::fs::read_to_string(ws.join("dag/examples/weather/weather.dag"))
                .expect("weather.dag should exist");

            let result = compile_dag_named(
                "dag/examples/weather/weather.dag",
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

fn floor_suite() -> Vec<WitnessCase> {
    vec![("floor_smoke", floor_smoke)]
}

fn expensive_suite() -> Vec<WitnessCase> {
    vec![
        ("stage0_cargo_check", stage0_cargo_check),
        (
            "strict_compile_diagnostic_count",
            strict_compile_diagnostic_count,
        ),
        (
            "stage0_compile_accepts_dag_target",
            stage0_compile_accepts_dag_target,
        ),
        (
            "stage0_compile_imports_ephemeral_generated_source_root",
            stage0_compile_imports_ephemeral_generated_source_root,
        ),
        ("bootstrap_stage0_to_stage1", bootstrap_stage0_to_stage1),
        ("bootstrap_fixed_point", bootstrap_fixed_point),
        ("performance_ratchet", performance_ratchet),
    ]
}

fn ci_suite() -> Vec<WitnessCase> {
    vec![
        ("ci_full_dag", ci_full_dag),
        ("ci_diagnostic_ratchet", ci_diagnostic_ratchet),
        ("ci_performance_ratchet", ci_performance_ratchet),
        ("ci_freshness", ci_freshness),
        ("ci_fixed_point", ci_fixed_point),
    ]
}

fn all_suite() -> Vec<WitnessCase> {
    let mut tests = floor_suite();
    tests.extend(expensive_suite());
    tests.extend(ci_suite());
    tests.push(("bootstrap_l4_structural", bootstrap_l4_structural));
    tests
}

fn run_suite(tests: &[WitnessCase]) -> ExitCode {
    for (name, test) in tests {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test));
        if result.is_err() {
            return fail(format!("{name} panicked"));
        }
    }
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    let suite = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "--suite".to_string());
    let suite_name = if suite == "--suite" {
        std::env::args()
            .nth(2)
            .unwrap_or_else(|| "floor".to_string())
    } else {
        suite
    };

    let tests: Vec<WitnessCase> = match suite_name.as_str() {
        "floor" => floor_suite(),
        "expensive" => expensive_suite(),
        "ci" => ci_suite(),
        "l4" => vec![("bootstrap_l4_structural", bootstrap_l4_structural)],
        "all" => all_suite(),
        other => {
            return fail(format!(
                "unknown suite {other:?}; expected floor|expensive|ci|l4|all"
            ))
        }
    };

    run_suite(&tests)
}
