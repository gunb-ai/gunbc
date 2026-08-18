//! Non-mutating stage0 regen CI receipt: authority digest, first-generation sync,
//! fixed-point stability (emit → rebuild → re-emit), and a typed JSON receipt.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use v1_compiler::cli_run::{lens_string_list_data, regen_input_sources, workspace_root};

const STAGE0_EMIT_PLAN_PROJECTION_REL: &str = "dag/gunbc/stage0_emit_plan_generated.dag";
const GENERATED_STAGE0_FILES_DATA_NAME: &str = "generated_stage0_files";
const RECEIPT_SCHEMA: &str = "gunbc.regen_receipt.v1";

#[derive(serde::Serialize)]
struct RegenReceipt {
    schema: &'static str,
    commit_sha: String,
    authority_digest: String,
    committed_generated_digest: String,
    candidate_generated_digest: String,
    first_generation_equal: bool,
    fixed_point_equal: bool,
    changed_paths: Vec<String>,
    candidate_artifact: String,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            let _ = writeln!(std::io::stderr(), "{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let workspace = workspace_root();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut candidate_rel = "target/stage0-regen-candidate".to_string();
    let mut receipt_rel = "target/stage0-regen-receipt.json".to_string();
    let mut regen_bin: Option<PathBuf> = None;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--candidate-dir" => {
                candidate_rel = args
                    .get(index + 1)
                    .ok_or("--candidate-dir requires <path>")?
                    .clone();
                index += 2;
            }
            "--receipt" => {
                receipt_rel = args
                    .get(index + 1)
                    .ok_or("--receipt requires <path>")?
                    .clone();
                index += 2;
            }
            "--regen-bin" => {
                regen_bin = Some(PathBuf::from(
                    args.get(index + 1).ok_or("--regen-bin requires <path>")?,
                ));
                index += 2;
            }
            unexpected => {
                return Err(format!(
                    "stage0_regen_receipt: unexpected argument {unexpected:?}\n\
                     Usage: stage0_regen_receipt [--candidate-dir <rel>] [--receipt <rel>] [--regen-bin <path>]"
                ));
            }
        }
    }

    let candidate_dir = workspace.join(&candidate_rel);
    let receipt_path = workspace.join(&receipt_rel);
    let committed_src = workspace.join("src/v1/stage0/src");
    let regen_bin = regen_bin.unwrap_or_else(default_regen_bin);
    if !regen_bin.is_file() {
        return Err(format!(
            "regen_stage0 binary missing at {} — build it before running the receipt",
            regen_bin.display()
        ));
    }

    let commit_sha = git_head_sha(&workspace)?;
    let authority_digest = authority_digest_hex(&workspace)?;
    let generated_files = generated_stage0_files()?;

    let manifest_path = candidate_dir.join("roster_manifest.txt");
    let pass1 = run_regen_emit_fresh(&regen_bin, &workspace, &candidate_dir, &manifest_path, true)?;
    let candidate_src = candidate_dir.join("src");
    let changed_paths = if candidate_src.is_dir() {
        changed_generated_paths(&committed_src, &candidate_src, &generated_files)?
    } else {
        Vec::new()
    };
    let first_generation_equal = pass1.success && changed_paths.is_empty();
    let committed_generated_digest = generated_tree_digest(&committed_src, &generated_files)?;
    let candidate_generated_digest = if candidate_src.is_dir() {
        generated_tree_digest(&candidate_src, &generated_files)?
    } else {
        String::from("fnv1a64:0000000000000000")
    };

    let mut fixed_point_equal = false;
    let fixed_point_error = if first_generation_equal {
        let pass2_dir = workspace.join("target/stage0-regen-fixed-point");
        let _ = fs::remove_dir_all(&pass2_dir);
        match build_regen_from_staged_candidate(&workspace, &candidate_src, &generated_files) {
            Ok(rebuilt_regen) => {
                let pass2 = run_regen_emit_fresh(
                    &rebuilt_regen,
                    &workspace,
                    &pass2_dir,
                    &pass2_dir.join("roster_manifest.txt"),
                    false,
                )?;
                let pass2_src = pass2_dir.join("src");
                fixed_point_equal = pass2.success
                    && generated_tree_digest(&candidate_src, &generated_files)?
                        == generated_tree_digest(&pass2_src, &generated_files)?;
                if fixed_point_equal {
                    None
                } else {
                    Some("re-emit after staged rebuild did not reproduce candidate".to_string())
                }
            }
            Err(message) => Some(message),
        }
    } else {
        None
    };

    let receipt = RegenReceipt {
        schema: RECEIPT_SCHEMA,
        commit_sha,
        authority_digest,
        committed_generated_digest,
        candidate_generated_digest,
        first_generation_equal,
        fixed_point_equal,
        changed_paths: changed_paths.clone(),
        candidate_artifact: candidate_rel,
    };
    write_receipt(&receipt_path, &receipt)?;

    println!(
        "stage0_regen_receipt: first_generation_equal={first_generation_equal} fixed_point_equal={fixed_point_equal} changed_paths={}",
        changed_paths.len()
    );
    if !first_generation_equal {
        let detail = if !pass1.success && !pass1.detail.is_empty() {
            format!("; regen_stage0: {}", pass1.detail)
        } else {
            String::new()
        };
        return Err(format!(
            "stage0 regen drift: committed seed differs from fresh self-compile ({} path(s)): {}{detail}",
            changed_paths.len(),
            changed_paths.join(", ")
        ));
    }
    if let Some(message) = fixed_point_error {
        return Err(format!("stage0 regen fixed-point check failed: {message}"));
    }
    Ok(())
}

struct RegenEmitOutcome {
    success: bool,
    detail: String,
}

fn default_regen_bin() -> PathBuf {
    let target_dir = std::env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("target"));
    target_dir.join("release/regen_stage0")
}

fn generated_stage0_files() -> Result<Vec<String>, String> {
    Ok(lens_string_list_data(
        STAGE0_EMIT_PLAN_PROJECTION_REL,
        GENERATED_STAGE0_FILES_DATA_NAME,
        false,
    ))
}

fn git_head_sha(workspace: &Path) -> Result<String, String> {
    if let Ok(sha) = std::env::var("GITHUB_SHA") {
        if !sha.is_empty() {
            return Ok(sha);
        }
    }
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("git rev-parse HEAD: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "git rev-parse HEAD failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn combine_hash(hash: u64, bytes: &[u8]) -> u64 {
    let mut next = hash;
    next ^= fnv1a64(bytes);
    next = next.wrapping_mul(0x100000001b3);
    next
}

fn authority_digest_hex(workspace: &Path) -> Result<String, String> {
    let sources = regen_input_sources(workspace)?;
    let mut hash = 0xcbf29ce484222325u64;
    for (rel, content) in sources {
        hash = combine_hash(hash, rel.as_bytes());
        hash = combine_hash(hash, content.as_bytes());
    }
    Ok(format!("fnv1a64:{hash:016x}"))
}

fn generated_tree_digest(stage0_src: &Path, file_names: &[String]) -> Result<String, String> {
    let mut hash = 0xcbf29ce484222325u64;
    for file_name in file_names {
        let path = stage0_src.join(file_name);
        let bytes =
            fs::read(&path).map_err(|e| format!("read generated file {}: {e}", path.display()))?;
        hash = combine_hash(hash, file_name.as_bytes());
        hash = combine_hash(hash, &bytes);
    }
    Ok(format!("fnv1a64:{hash:016x}"))
}

fn changed_generated_paths(
    committed_src: &Path,
    candidate_src: &Path,
    file_names: &[String],
) -> Result<Vec<String>, String> {
    let mut changed = Vec::new();
    for file_name in file_names {
        let committed = committed_src.join(file_name);
        let candidate = candidate_src.join(file_name);
        let committed_text = fs::read_to_string(&committed)
            .map_err(|e| format!("read committed generated file {}: {e}", committed.display()))?;
        let candidate_text = fs::read_to_string(&candidate)
            .map_err(|e| format!("read candidate generated file {}: {e}", candidate.display()))?;
        if committed_text != candidate_text {
            changed.push(file_name.clone());
        }
    }
    Ok(changed)
}

fn run_regen_emit_fresh(
    regen_bin: &Path,
    workspace: &Path,
    fresh_dir: &Path,
    manifest_path: &Path,
    verify: bool,
) -> Result<RegenEmitOutcome, String> {
    let _ = fs::remove_dir_all(fresh_dir);
    let mut command = Command::new(regen_bin);
    command
        .current_dir(workspace)
        .arg("--emit-fresh")
        .arg(fresh_dir)
        .arg("--write-manifest")
        .arg(manifest_path);
    if verify {
        command.arg("--verify");
    }
    let output = command
        .output()
        .map_err(|e| format!("spawn {}: {e}", regen_bin.display()))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.is_empty() {
        print!("{stdout}");
    }
    if !stderr.is_empty() {
        eprint!("{stderr}");
    }
    if !output.status.success() {
        let detail = if stderr.is_empty() {
            stdout.trim().to_string()
        } else {
            stderr.trim().to_string()
        };
        return Ok(RegenEmitOutcome {
            success: false,
            detail,
        });
    }
    Ok(RegenEmitOutcome {
        success: true,
        detail: String::new(),
    })
}

fn copy_dir_recursive(source: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| format!("create {}: {e}", dest.display()))?;
    for entry in fs::read_dir(source).map_err(|e| format!("read {}: {e}", source.display()))? {
        let entry = entry.map_err(|e| format!("read dir entry: {e}"))?;
        let src_path = entry.path();
        let dst_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path).map_err(|e| {
                format!("copy {} -> {}: {e}", src_path.display(), dst_path.display())
            })?;
        }
    }
    Ok(())
}

fn patch_staging_cargo_manifest(staging_root: &Path) -> Result<(), String> {
    let cargo_toml = staging_root.join("Cargo.toml");
    let mut contents = fs::read_to_string(&cargo_toml)
        .map_err(|e| format!("read {}: {e}", cargo_toml.display()))?;
    contents = contents.replace("toml = { workspace = true }", "toml = \"0.8\"");
    if !contents.contains("[workspace]") {
        contents.push_str("\n[workspace]\n");
    }
    fs::write(&cargo_toml, contents).map_err(|e| format!("write staging Cargo.toml: {e}"))?;
    Ok(())
}

fn build_regen_from_staged_candidate(
    workspace: &Path,
    candidate_src: &Path,
    generated_files: &[String],
) -> Result<PathBuf, String> {
    let stage0_root = workspace.join("src/v1/stage0");
    let staging_root = workspace.join("target/stage0-regen-staging");
    let _ = fs::remove_dir_all(&staging_root);
    copy_dir_recursive(&stage0_root, &staging_root)?;
    patch_staging_cargo_manifest(&staging_root)?;
    let staging_src = staging_root.join("src");
    for file_name in generated_files {
        let from = candidate_src.join(file_name);
        let to = staging_src.join(file_name);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        fs::copy(&from, &to).map_err(|e| {
            format!(
                "stage generated file {} -> {}: {e}",
                from.display(),
                to.display()
            )
        })?;
    }
    let target_dir = workspace.join("target/stage0-regen-receipt-build");
    let output = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(staging_root.join("Cargo.toml"))
        .arg("--release")
        .arg("--bin")
        .arg("regen_stage0")
        .env("CARGO_TARGET_DIR", &target_dir)
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("spawn cargo build for staged regen_stage0: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "staged regen_stage0 rebuild failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let rebuilt = target_dir.join("release/regen_stage0");
    if !rebuilt.is_file() {
        return Err(format!(
            "staged regen_stage0 rebuild produced no binary at {}",
            rebuilt.display()
        ));
    }
    Ok(rebuilt)
}

fn write_receipt(path: &Path, receipt: &RegenReceipt) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    let body = serde_json::to_string_pretty(receipt)
        .map_err(|e| format!("serialize regen receipt: {e}"))?;
    fs::write(path, format!("{body}\n")).map_err(|e| format!("write {}: {e}", path.display()))
}
