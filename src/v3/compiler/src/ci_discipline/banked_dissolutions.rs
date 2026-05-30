//! Banked-dissolutions ratchet — scan lane/phase docs for shapes rejected in the master plan.

use std::fs;
use std::path::Path;

use super::repo_root;

const MASTER_PLAN: &str = "docs/post-l15-phase-plan.md";

pub fn check_banked_dissolutions() -> Result<(), String> {
    let root = repo_root()?;
    let master = root.join(MASTER_PLAN);
    if !master.is_file() {
        return Err(format!("banked-dissolutions: missing master plan {MASTER_PLAN}"));
    }
    let plan_text = fs::read_to_string(&master)
        .map_err(|e| format!("read {MASTER_PLAN}: {e}"))?;
    let forbidden = parse_forbidden_array(&plan_text)?;
    if forbidden.is_empty() {
        return Err(format!(
            "banked-dissolutions: could not extract FORBIDDEN array from {MASTER_PLAN}"
        ));
    }

    let files = collect_lane_phase_docs(&root)?;
    if files.is_empty() {
        eprintln!("banked-dissolutions: no lane/phase docs to scan");
        return Ok(());
    }

    let mut violation_patterns = 0usize;
    for pat in &forbidden {
        let mut hits = Vec::new();
        for file in &files {
            let text = fs::read_to_string(file).map_err(|e| format!("read {}: {e}", file.display()))?;
            for (i, line) in text.lines().enumerate() {
                if line.contains(pat.as_str()) {
                    hits.push(format!("{}:{}:{}", file.display(), i + 1, line));
                }
            }
        }
        if !hits.is_empty() {
            if violation_patterns == 0 {
                eprintln!("banked-dissolutions ratchet: forbidden shapes found in lane/phase docs.");
                eprintln!("Authority: {MASTER_PLAN} § Banked dissolutions.");
                eprintln!();
            }
            eprintln!("--- forbidden: {pat} ---");
            for h in &hits {
                eprintln!("{h}");
            }
            eprintln!();
            violation_patterns += 1;
        }
    }

    if violation_patterns > 0 {
        return Err(format!(
            "banked-dissolutions ratchet: {violation_patterns} forbidden shape(s) found."
        ));
    }
    eprintln!(
        "banked-dissolutions ratchet: clean ({} docs scanned, {} forbidden shapes from {MASTER_PLAN})",
        files.len(),
        forbidden.len()
    );
    Ok(())
}

pub(crate) fn parse_forbidden_array(text: &str) -> Result<Vec<String>, String> {
    let mut in_block = false;
    let mut entries = Vec::new();
    for line in text.lines() {
        if line.starts_with("FORBIDDEN=(") {
            in_block = true;
            continue;
        }
        if in_block {
            if line.starts_with(')') {
                break;
            }
            let mut rest = line;
            while let Some(start) = rest.find('"') {
                let after = &rest[start + 1..];
                let Some(end) = after.find('"') else {
                    break;
                };
                let inner = &after[..end];
                if !inner.is_empty() {
                    entries.push(inner.to_string());
                }
                rest = &after[end + 1..];
            }
        }
    }
    Ok(entries)
}

fn collect_lane_phase_docs(root: &Path) -> Result<Vec<std::path::PathBuf>, String> {
    let docs = root.join("docs");
    if !docs.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    for entry in fs::read_dir(&docs).map_err(|e| format!("read docs/: {e}"))? {
        let entry = entry.map_err(|e| format!("read docs entry: {e}"))?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let is_lane = name.starts_with("lane") && name.ends_with(".md");
        let is_phase = name.starts_with("phase") && name.ends_with(".md");
        if !is_lane && !is_phase {
            continue;
        }
        if name == "post-l15-phase-plan.md" {
            continue;
        }
        if name.starts_with("design-") {
            continue;
        }
        files.push(path);
    }
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod parse_tests {
    use super::parse_forbidden_array;

    #[test]
    fn reads_quoted_entries_per_line() {
        let sample = r#"
FORBIDDEN=(
  "port_by_id" "node_by_id"
  "MutualLoop"
)
"#;
        let got = parse_forbidden_array(sample).unwrap();
        assert_eq!(got, vec!["port_by_id", "node_by_id", "MutualLoop"]);
    }
}
