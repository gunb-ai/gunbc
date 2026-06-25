use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const AUTHORITY_REL: &str = "dsl/std/languages.dag";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguagesDeclConsumerRecord {
    pub decl_name: String,
    pub external_consumer_paths: Vec<String>,
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("workspace root")
        .to_path_buf()
}

fn extract_data_decl_names(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("data ")?;
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if name.is_empty() {
                None
            } else {
                Some(name)
            }
        })
        .collect()
}

fn read_source(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        panic!(
            "languages_consumer_census: failed to read {}: {e}",
            path.display()
        )
    })
}

fn rel_path_from_workspace(ws: &Path, path: &Path) -> String {
    path.strip_prefix(ws)
        .unwrap_or_else(|_| {
            panic!(
                "languages_consumer_census: path {} is not under workspace {}",
                path.display(),
                ws.display()
            )
        })
        .to_string_lossy()
        .replace('\\', "/")
}

fn strip_comments_and_string_literals(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'/') {
            while chars.next().is_some_and(|ch| ch != '\n') {}
            out.push('\n');
            continue;
        }
        if c == '"' {
            while let Some(ch) = chars.next() {
                if ch == '\\' {
                    chars.next();
                    continue;
                }
                if ch == '"' {
                    break;
                }
            }
            out.push(' ');
            continue;
        }
        if c == '`' {
            while chars.next().is_some_and(|ch| ch != '`') {}
            out.push(' ');
            continue;
        }
        out.push(c);
    }
    out
}

fn is_census_infrastructure_path(rel: &str) -> bool {
    rel == "src/v1/stage0/src/languages_consumer_census.rs"
        || rel.starts_with("src/v2/test/claim/languages_consumer_census/")
        || rel == "src/v2/lens/languages_consumer_census.dag"
}

fn tokenize_identifiers(content: &str) -> HashSet<String> {
    let stripped = strip_comments_and_string_literals(content);
    stripped
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect()
}

fn walk_tree_dir(top_root: &Path, dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| {
        panic!(
            "languages_consumer_census: failed to read dir {}: {e}",
            dir.display()
        )
    });
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_tree_dir(top_root, &path, out);
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext == "dag" || ext == "rs" {
            out.push(path);
        }
    }
}

fn walk_tree(top_root: &Path, out: &mut Vec<PathBuf>) {
    if !top_root.is_dir() {
        panic!(
            "languages_consumer_census: tree root {} does not exist",
            top_root.display()
        );
    }
    walk_tree_dir(top_root, top_root, out);
}

fn decl_records_inner() -> Vec<LanguagesDeclConsumerRecord> {
    let ws = workspace_root();
    let authority = ws.join(AUTHORITY_REL);
    let authority_content = read_source(&authority);
    let decl_names = extract_data_decl_names(&authority_content);
    let decl_name_set: HashSet<String> = decl_names.iter().cloned().collect();

    let mut files = Vec::new();
    walk_tree(&ws.join("dsl"), &mut files);
    walk_tree(&ws.join("src"), &mut files);

    let mut by_decl: HashMap<String, HashSet<String>> = decl_names
        .iter()
        .map(|name| (name.clone(), HashSet::new()))
        .collect();

    for path in files {
        let rel = rel_path_from_workspace(&ws, &path);
        if rel == AUTHORITY_REL || is_census_infrastructure_path(&rel) {
            continue;
        }
        let tokens = tokenize_identifiers(&read_source(&path));
        for decl_name in tokens.intersection(&decl_name_set) {
            by_decl
                .get_mut(decl_name)
                .expect("decl map key")
                .insert(rel.clone());
        }
    }

    let mut records = Vec::new();
    for decl_name in decl_names {
        let mut paths: Vec<String> = by_decl
            .remove(&decl_name)
            .expect("decl map key")
            .into_iter()
            .collect();
        paths.sort();
        records.push(LanguagesDeclConsumerRecord {
            decl_name,
            external_consumer_paths: paths,
        });
    }
    records
}

fn decl_records_cached() -> &'static [LanguagesDeclConsumerRecord] {
    static RECORDS: OnceLock<Vec<LanguagesDeclConsumerRecord>> = OnceLock::new();
    RECORDS.get_or_init(decl_records_inner)
}

fn record_for_decl(decl_name: &str) -> Option<&'static LanguagesDeclConsumerRecord> {
    decl_records_cached()
        .iter()
        .find(|record| record.decl_name == decl_name)
}

fn is_format_row(decl_name: &str) -> bool {
    decl_name.ends_with("_format")
}

pub fn languages_decl_records() -> Vec<LanguagesDeclConsumerRecord> {
    decl_records_cached().to_vec()
}

pub fn languages_consumer_census_data_decl_count() -> i64 {
    decl_records_cached().len() as i64
}

pub fn languages_consumer_census_per_language_row_count() -> i64 {
    decl_records_cached()
        .iter()
        .filter(|record| !is_format_row(&record.decl_name))
        .count() as i64
}

pub fn languages_consumer_census_format_row_count() -> i64 {
    decl_records_cached()
        .iter()
        .filter(|record| is_format_row(&record.decl_name))
        .count() as i64
}

pub fn languages_consumer_census_external_consumer_count(decl_name: String) -> i64 {
    record_for_decl(&decl_name)
        .map(|record| record.external_consumer_paths.len() as i64)
        .unwrap_or(-1)
}

pub fn languages_consumer_census_is_composition_only(decl_name: String) -> bool {
    record_for_decl(&decl_name)
        .map(|record| record.external_consumer_paths.is_empty())
        .unwrap_or(false)
}

pub fn languages_consumer_census_has_external_consumer(decl_name: String) -> bool {
    record_for_decl(&decl_name)
        .map(|record| !record.external_consumer_paths.is_empty())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_decl_baseline_is_seventy_one() {
        assert_eq!(languages_consumer_census_data_decl_count(), 71);
    }

    #[test]
    fn per_language_row_baseline_is_sixty_four() {
        assert_eq!(languages_consumer_census_per_language_row_count(), 64);
    }

    #[test]
    fn format_row_baseline_is_seven() {
        assert_eq!(languages_consumer_census_format_row_count(), 7);
    }

    #[test]
    fn rust_statements_is_composition_only() {
        assert!(languages_consumer_census_is_composition_only(
            "rust_statements".to_string()
        ));
    }

    #[test]
    fn rust_spec_has_external_consumers() {
        assert!(languages_consumer_census_has_external_consumer(
            "rust_spec".to_string()
        ));
    }

    #[test]
    fn rust_language_reaches_task_manager_fixture() {
        let record = record_for_decl("rust_language").expect("rust_language row");
        assert!(record
            .external_consumer_paths
            .iter()
            .any(|path| path.contains("task_manager_demo.dag")));
    }
}
