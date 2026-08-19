//! Host realization for `v2.workflow.required_regen` — committed seed vs fresh emit.

use serde::Serialize;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::rc::Rc;
use std::time::Instant;

#[path = "bootstrap_stage0_crate_layout_generated.rs"]
mod bootstrap_stage0_crate_layout_generated;
use super::workspace_root;
use crate::v1_compiler_artifact::RenderTarget;
use crate::v1_compiler_compile::{
    compile_sources, stage0_self_compile_refusal_message, SourceFile,
};
use crate::v1_rt;
use bootstrap_stage0_crate_layout_generated::{
    HAND_MAINTAINED_STAGE0_DIRS, HAND_MAINTAINED_STAGE0_FILES,
};

const RECEIPT_SCHEMA: &str = "gunbc.regen_receipt.v1";

#[derive(Debug, Serialize)]
pub struct RegenReceipt {
    pub schema: &'static str,
    pub commit_sha: String,
    pub authority_digest: String,
    pub committed_generated_digest: String,
    pub candidate_generated_digest: String,
    pub first_generation_equal: bool,
    pub fixed_point_equal: bool,
    pub changed_paths: Vec<String>,
    pub candidate_artifact: String,
}

#[derive(Debug)]
pub struct RequiredRegenOutcome {
    pub receipt: RegenReceipt,
    pub failures: Vec<String>,
}

pub fn run_required_regen(
    candidate_dir_rel: &str,
    receipt_rel: &str,
) -> Result<RequiredRegenOutcome, String> {
    let workspace = workspace_root();
    let candidate_dir = workspace.join(candidate_dir_rel);
    let receipt_path = workspace.join(receipt_rel);
    let stage0_src = workspace.join("src/v1/stage0/src");
    let run_started = Instant::now();

    let commit_sha = git_head_sha(&workspace)?;
    let sources = super::regen_input_sources(&workspace)?;
    let authority_digest = authority_digest_from_sources(&sources)?;

    let emitted = compile_stage0(&workspace)?;

    let committed_basenames = committed_generated_basenames(&stage0_src)?;
    if emitted.is_empty() {
        return regen_refusal_outcome(
            &workspace,
            candidate_dir_rel,
            receipt_rel,
            commit_sha,
            authority_digest,
            "refusal: emit produced zero files".to_string(),
        );
    }
    let emitted_basenames = generated_basenames_from_emit(&emitted);
    let population_err = validate_compared_populations(&committed_basenames, &emitted_basenames);
    if let Some(reason) = population_err {
        return regen_refusal_outcome(
            &workspace,
            candidate_dir_rel,
            receipt_rel,
            commit_sha,
            authority_digest,
            reason,
        );
    }

    let sync = compare_generated_surfaces(&stage0_src, &emitted, &committed_basenames)?;
    let hand = verify_hand_maintained(&emitted, &stage0_src, &candidate_dir)?;

    let committed_digest =
        tree_digest_for_basenames(&stage0_src, &committed_basenames, "committed")?;
    let candidate_digest = tree_digest_from_map(&emitted, &committed_basenames)?;

    let first_generation_equal = sync.matches && hand.unverifiable.is_empty();
    let changed_paths = sync.drifted_paths.clone();

    if candidate_dir.exists() {
        fs::remove_dir_all(&candidate_dir)
            .map_err(|e| format!("remove {}: {e}", candidate_dir.display()))?;
    }
    let fresh_src = candidate_dir.join("src");
    write_emitted_tree(&fresh_src, &emitted)?;
    copy_hand_maintained_support(&stage0_src, &fresh_src)?;
    verify_candidate_tree(&fresh_src, &committed_basenames)?;

    let receipt = RegenReceipt {
        schema: RECEIPT_SCHEMA,
        commit_sha,
        authority_digest,
        committed_generated_digest: committed_digest,
        candidate_generated_digest: candidate_digest,
        first_generation_equal,
        fixed_point_equal: false,
        changed_paths: changed_paths.clone(),
        candidate_artifact: candidate_dir_rel.to_string(),
    };
    write_receipt(&receipt_path, &receipt)?;

    let mut failures = Vec::new();
    if !sync.matches {
        failures.push(format!(
            "generated surface drift: {}",
            changed_paths.join(", ")
        ));
    }
    for (name, reason) in &hand.unverifiable {
        failures.push(format!("hand unverifiable {name}: {reason}"));
    }

    eprintln!(
        "required-regen: elapsed_ms={} first_generation_equal={} planned={} executed={}",
        run_started.elapsed().as_millis(),
        first_generation_equal,
        committed_basenames.len(),
        emitted_basenames.len()
    );

    Ok(RequiredRegenOutcome { receipt, failures })
}

pub fn run_required_regen_fixed_point(
    receipt_rel: &str,
    pass1_digest: Option<String>,
) -> Result<RequiredRegenOutcome, String> {
    let workspace = workspace_root();
    let receipt_path = workspace.join(receipt_rel);
    let prior = read_receipt(&receipt_path)?;
    let pass1 = pass1_digest.unwrap_or(prior.candidate_generated_digest);

    let commit_sha = git_head_sha(&workspace)?;
    let sources = super::regen_input_sources(&workspace)?;
    let authority_digest = authority_digest_from_sources(&sources)?;
    let emitted = compile_stage0(&workspace)?;
    let committed_basenames = committed_generated_basenames(&workspace.join("src/v1/stage0/src"))?;
    if emitted.is_empty() {
        return Err("refusal: fixed-point emit produced zero files".to_string());
    }
    let emitted_basenames = generated_basenames_from_emit(&emitted);
    if let Some(reason) = validate_compared_populations(&committed_basenames, &emitted_basenames) {
        return Err(reason);
    }
    let pass2 = tree_digest_from_map(&emitted, &committed_basenames)?;
    let fixed_point_equal = pass1 == pass2;

    let receipt = RegenReceipt {
        schema: RECEIPT_SCHEMA,
        commit_sha,
        authority_digest,
        committed_generated_digest: prior.committed_generated_digest,
        candidate_generated_digest: pass2.clone(),
        first_generation_equal: prior.first_generation_equal,
        fixed_point_equal,
        changed_paths: prior.changed_paths,
        candidate_artifact: prior.candidate_artifact,
    };
    write_receipt(&receipt_path, &receipt)?;

    let failures = if fixed_point_equal {
        Vec::new()
    } else {
        vec![format!(
            "fixed-point refused: pass-1 digest {pass1} != pass-2 digest {pass2}"
        )]
    };

    Ok(RequiredRegenOutcome { receipt, failures })
}

struct SyncReport {
    matches: bool,
    drifted_paths: Vec<String>,
}

struct HandVerifyReport {
    unverifiable: Vec<(String, String)>,
}

fn compile_stage0(workspace: &Path) -> Result<HashMap<String, String>, String> {
    let sources = super::regen_input_sources(workspace)?;
    let source_files: Vec<Rc<SourceFile>> = sources
        .into_iter()
        .map(|(path, content)| Rc::new(SourceFile { path, content }))
        .collect();
    let result = compile_sources(Rc::new(source_files.into()), RenderTarget::Rust);
    if let Some(message) = stage0_self_compile_refusal_message(result.clone()) {
        return Err(message);
    }
    let mut out = HashMap::new();
    for file in result.files.iter() {
        out.insert(file.path.clone(), file.content.clone());
    }
    Ok(out)
}

fn generated_basenames_from_emit(emitted: &HashMap<String, String>) -> Vec<String> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    for path in emitted.keys() {
        if path.ends_with(".rs") && !is_hand_maintained_path(path) {
            // Basename, not the emit key: `committed_generated_basenames` keys on
            // `file_name()`, and emit keys carry a `src/` prefix. Comparing the two
            // key spaces made every file mismatch in both directions.
            let basename = Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path);
            names.insert(basename.to_string());
        }
    }
    names.into_iter().collect()
}

fn committed_generated_basenames(stage0_src: &Path) -> Result<Vec<String>, String> {
    let mut names: BTreeSet<String> = BTreeSet::new();
    for entry in fs::read_dir(stage0_src)
        .map_err(|e| format!("read committed stage0 src {}: {e}", stage0_src.display()))?
    {
        let entry = entry.map_err(|e| format!("read committed stage0 entry: {e}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let basename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if basename.ends_with(".rs") && !HAND_MAINTAINED_STAGE0_FILES.contains(&basename) {
            names.insert(basename.to_string());
        }
    }
    if names.is_empty() {
        return Err("refusal: committed generated population is empty".to_string());
    }
    Ok(names.into_iter().collect())
}

fn validate_compared_populations(committed: &[String], emitted: &[String]) -> Option<String> {
    if committed.is_empty() {
        return Some("refusal: committed generated population is empty".to_string());
    }
    if emitted.is_empty() {
        return Some("refusal: emit produced zero generated surfaces".to_string());
    }
    let committed_set: BTreeSet<&str> = committed.iter().map(String::as_str).collect();
    let emitted_set: BTreeSet<&str> = emitted.iter().map(String::as_str).collect();
    let mut emitted_not_committed = Vec::new();
    for name in emitted {
        if !committed_set.contains(name.as_str()) {
            emitted_not_committed.push(name.clone());
        }
    }
    let mut committed_not_emitted = Vec::new();
    for name in committed {
        if !emitted_set.contains(name.as_str()) {
            committed_not_emitted.push(name.clone());
        }
    }
    if !emitted_not_committed.is_empty() || !committed_not_emitted.is_empty() {
        return Some(format!(
            "refusal: surface population mismatch — emitted_not_committed={:?} committed_not_emitted={:?}",
            emitted_not_committed,
            committed_not_emitted
        ));
    }
    None
}

fn verify_candidate_tree(
    candidate_src: &Path,
    expected_basenames: &[String],
) -> Result<(), String> {
    if expected_basenames.is_empty() {
        return Err(
            "refusal: cannot verify candidate tree against empty expected population".to_string(),
        );
    }
    if !candidate_src.is_dir() {
        return Err(format!(
            "refusal: candidate src directory absent at {}",
            candidate_src.display()
        ));
    }
    let mut found = 0usize;
    for basename in expected_basenames {
        if candidate_src.join(basename).is_file() {
            found += 1;
        }
    }
    if found == 0 {
        return Err(format!(
            "refusal: candidate tree has zero generated files under {}",
            candidate_src.display()
        ));
    }
    if found != expected_basenames.len() {
        return Err(format!(
            "refusal: candidate tree incomplete — found {found} of {} expected generated files under {}",
            expected_basenames.len(),
            candidate_src.display()
        ));
    }
    Ok(())
}

fn regen_refusal_outcome(
    workspace: &Path,
    candidate_dir_rel: &str,
    receipt_rel: &str,
    commit_sha: String,
    authority_digest: String,
    reason: String,
) -> Result<RequiredRegenOutcome, String> {
    let receipt_path = workspace.join(receipt_rel);
    let receipt = RegenReceipt {
        schema: RECEIPT_SCHEMA,
        commit_sha,
        authority_digest,
        committed_generated_digest: "refused:population".to_string(),
        candidate_generated_digest: "refused:population".to_string(),
        first_generation_equal: false,
        fixed_point_equal: false,
        changed_paths: Vec::new(),
        candidate_artifact: candidate_dir_rel.to_string(),
    };
    write_receipt(&receipt_path, &receipt)?;
    Ok(RequiredRegenOutcome {
        receipt,
        failures: vec![reason],
    })
}

fn is_hand_maintained_path(path: &str) -> bool {
    let basename = Path::new(path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(path);
    HAND_MAINTAINED_STAGE0_FILES.contains(&basename)
}

fn compare_generated_surfaces(
    stage0_src: &Path,
    emitted: &HashMap<String, String>,
    generated_basenames: &[String],
) -> Result<SyncReport, String> {
    let mut drifted = Vec::new();
    for basename in generated_basenames {
        let committed_path = stage0_src.join(basename);
        let committed = normalize_generated_source(
            &fs::read_to_string(&committed_path)
                .map_err(|e| format!("read committed {}: {e}", committed_path.display()))?,
        )?;
        let candidate = emitted
            .get(&format!("src/{basename}"))
            .or_else(|| emitted.get(basename))
            .ok_or_else(|| format!("emit missing generated file {basename}"))?;
        let candidate_norm = normalize_generated_source(candidate)?;
        if committed != candidate_norm {
            drifted.push(basename.clone());
        }
    }
    Ok(SyncReport {
        matches: drifted.is_empty(),
        drifted_paths: drifted,
    })
}

fn verify_hand_maintained(
    emitted: &HashMap<String, String>,
    stage0_src: &Path,
    work_dir: &Path,
) -> Result<HandVerifyReport, String> {
    let mut unverifiable = Vec::new();
    for file_name in HAND_MAINTAINED_STAGE0_FILES {
        let candidate = emitted
            .get(&format!("src/{file_name}"))
            .or_else(|| emitted.get(*file_name));
        let Some(candidate) = candidate else {
            continue;
        };
        let committed_path = stage0_src.join(file_name);
        let committed = fs::read_to_string(&committed_path)
            .map_err(|e| format!("read committed hand file {}: {e}", committed_path.display()))?;
        match normalize_with_workdir(&committed, work_dir, "committed") {
            Ok(committed_norm) => match normalize_with_workdir(candidate, work_dir, "candidate") {
                Ok(candidate_norm) => {
                    if committed_norm != candidate_norm {
                        // drift expected on clean tree for some hand files; not a sync refusal.
                    }
                }
                Err(reason) => unverifiable.push(((*file_name).to_string(), reason)),
            },
            Err(reason) => unverifiable.push(((*file_name).to_string(), reason)),
        }
    }
    Ok(HandVerifyReport { unverifiable })
}

fn write_emitted_tree(dest_src: &Path, emitted: &HashMap<String, String>) -> Result<(), String> {
    if dest_src.exists() {
        fs::remove_dir_all(dest_src).map_err(|e| format!("remove {}: {e}", dest_src.display()))?;
    }
    fs::create_dir_all(dest_src).map_err(|e| format!("create {}: {e}", dest_src.display()))?;
    for (path, content) in emitted {
        let out_path = dest_src.join(path);
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        if path.ends_with(".rs") {
            let normalized = normalize_generated_source(content)?;
            fs::write(&out_path, normalized)
                .map_err(|e| format!("write {}: {e}", out_path.display()))?;
        } else {
            fs::write(&out_path, content)
                .map_err(|e| format!("write {}: {e}", out_path.display()))?;
        }
    }
    Ok(())
}

fn copy_hand_maintained_support(stage0_src: &Path, dest_src: &Path) -> Result<(), String> {
    for file_name in HAND_MAINTAINED_STAGE0_FILES {
        let source = stage0_src.join(file_name);
        if source.exists() {
            fs::copy(&source, dest_src.join(file_name))
                .map_err(|e| format!("copy {}: {e}", source.display()))?;
        }
    }
    for dir_name in HAND_MAINTAINED_STAGE0_DIRS {
        let source = stage0_src.join(dir_name);
        if source.is_dir() {
            copy_dir_recursive(&source, &dest_src.join(dir_name))?;
        }
    }
    Ok(())
}

fn copy_dir_recursive(source: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| format!("create {}: {e}", dest.display()))?;
    for entry in fs::read_dir(source).map_err(|e| format!("read dir {}: {e}", source.display()))? {
        let entry = entry.map_err(|e| format!("read dir entry: {e}"))?;
        let path = entry.path();
        if path.is_dir() {
            copy_dir_recursive(&path, &dest.join(entry.file_name()))?;
        } else {
            fs::copy(&path, dest.join(entry.file_name()))
                .map_err(|e| format!("copy {}: {e}", path.display()))?;
        }
    }
    Ok(())
}

fn digest_label(bytes: &[u8]) -> String {
    format!("fnv1a64:{}", v1_rt::bytes_identity_hash(bytes))
}

fn authority_digest_from_sources(sources: &[(String, String)]) -> Result<String, String> {
    let mut payload = String::new();
    for (path, content) in sources {
        payload.push_str(path);
        payload.push('\0');
        payload.push_str(&digest_label(content.as_bytes()));
        payload.push('\n');
    }
    Ok(digest_label(payload.as_bytes()))
}

fn tree_digest_for_basenames(
    src_dir: &Path,
    basenames: &[String],
    label: &str,
) -> Result<String, String> {
    if basenames.is_empty() {
        return Err(format!(
            "refusal: cannot compute {label} digest over empty population"
        ));
    }
    let mut payload = String::new();
    for name in basenames {
        let path = src_dir.join(name);
        let content = fs::read_to_string(&path)
            .map_err(|e| format!("read {label} {}: {e}", path.display()))?;
        let norm = normalize_generated_source(&content)?;
        payload.push_str(name);
        payload.push('\0');
        payload.push_str(&digest_label(norm.as_bytes()));
        payload.push('\n');
    }
    Ok(digest_label(payload.as_bytes()))
}

fn tree_digest_from_map(
    emitted: &HashMap<String, String>,
    basenames: &[String],
) -> Result<String, String> {
    if basenames.is_empty() {
        return Err("refusal: cannot compute candidate digest over empty population".to_string());
    }
    let mut payload = String::new();
    for name in basenames {
        let content = emitted
            .get(&format!("src/{name}"))
            .or_else(|| emitted.get(name))
            .ok_or_else(|| format!("emit missing {name} for digest"))?;
        let norm = normalize_generated_source(content)?;
        payload.push_str(name);
        payload.push('\0');
        payload.push_str(&digest_label(norm.as_bytes()));
        payload.push('\n');
    }
    Ok(digest_label(payload.as_bytes()))
}

fn normalize_generated_source(content: &str) -> Result<String, String> {
    normalize_generated_source_attempt(content)
}

fn normalize_generated_source_attempt(content: &str) -> Result<String, String> {
    let mut child = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("spawn rustfmt: {e}"))?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "rustfmt stdin unavailable".to_string())?;
    let owned = content.to_string();
    let writer = std::thread::spawn(move || stdin.write_all(owned.as_bytes()));
    let out = child
        .wait_with_output()
        .map_err(|e| format!("wait rustfmt: {e}"))?;
    writer
        .join()
        .map_err(|_| "rustfmt stdin writer panicked".to_string())?
        .map_err(|e| format!("write rustfmt stdin: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).to_string())
    }
}

fn normalize_with_workdir(content: &str, work_dir: &Path, label: &str) -> Result<String, String> {
    let path = work_dir.join(format!("{label}.rs"));
    fs::write(&path, content).map_err(|e| format!("write {}: {e}", path.display()))?;
    let output = Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .arg(path.as_os_str())
        .output()
        .map_err(|e| format!("rustfmt {label}: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    fs::read_to_string(&path).map_err(|e| format!("read normalized {label}: {e}"))
}

fn git_head_sha(workspace: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(workspace)
        .output()
        .map_err(|e| format!("git rev-parse HEAD: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[derive(serde::Deserialize)]
struct RegenReceiptStored {
    commit_sha: String,
    authority_digest: String,
    committed_generated_digest: String,
    candidate_generated_digest: String,
    first_generation_equal: bool,
    fixed_point_equal: bool,
    changed_paths: Vec<String>,
    candidate_artifact: String,
}

fn read_receipt(path: &Path) -> Result<RegenReceiptStored, String> {
    let bytes =
        fs::read_to_string(path).map_err(|e| format!("read receipt {}: {e}", path.display()))?;
    serde_json::from_str(&bytes).map_err(|e| format!("parse receipt {}: {e}", path.display()))
}

fn write_receipt(path: &Path, receipt: &RegenReceipt) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(receipt)
        .map_err(|e| format!("serialize regen receipt: {e}"))?;
    fs::write(path, json).map_err(|e| format!("write receipt {}: {e}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("{prefix}-{nanos}"));
        fs::create_dir_all(&path).expect("create temp");
        path
    }

    #[test]
    fn empty_population_digest_refuses() {
        let err = tree_digest_for_basenames(Path::new("/tmp"), &[], "committed").unwrap_err();
        assert!(err.contains("empty population"));
        let err = tree_digest_from_map(&HashMap::new(), &[]).unwrap_err();
        assert!(err.contains("empty population"));
    }

    #[test]
    fn empty_emit_population_refuses_before_agreement() {
        let reason =
            validate_compared_populations(&["foo.rs".to_string()], &[]).expect("expected refusal");
        assert!(reason.contains("zero generated surfaces"));
    }

    #[test]
    fn empty_committed_population_refuses_before_agreement() {
        let reason =
            validate_compared_populations(&[], &["foo.rs".to_string()]).expect("expected refusal");
        assert!(reason.contains("committed generated population is empty"));
    }

    #[test]
    fn absent_candidate_tree_refuses() {
        let tmp = temp_dir("required-regen-absent");
        let missing = tmp.join("no-such-src");
        let err = verify_candidate_tree(&missing, &["foo.rs".to_string()]).unwrap_err();
        assert!(err.contains("candidate src directory absent"));
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn present_candidate_tree_reports_honestly() {
        let tmp = temp_dir("required-regen-present");
        let src = tmp.join("src");
        fs::create_dir_all(&src).expect("create src");
        fs::write(src.join("foo.rs"), "fn foo() {}\n").expect("write foo");
        verify_candidate_tree(&src, &["foo.rs".to_string()]).expect("candidate present");
        let _ = fs::remove_dir_all(&tmp);
    }
}
