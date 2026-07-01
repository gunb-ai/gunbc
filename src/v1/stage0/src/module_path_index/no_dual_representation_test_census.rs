// The no-dual-representation-test audit (DESIGN.md §5 construction-not-validation, recursive on the
// test corpus). A witness that mirrors the authority it claims to exercise is specification-without-
// execution — fluent, type-checking, grep-passing, and uninformative when wrong (DESIGN §5).
//
// WALL THE DECIDABLE SUBSET ONLY, HONESTLY:
//   (1) same-provenance equality — `ident == ident` (including `X == X`): always true by reflexivity;
//       it cannot discriminate a defect in the subject under test.
//   (2) witness RHS bare-literal duplication — `data_name == <literal>` where the same module (or an
//       explicitly-imported authority module) already declares `data data_name = <literal>`: the test
//       re-states the initializer instead of exercising a derived fact.
//
// Host-fed today (text scan over `*_test.dag` files); DISSOLUTION: a pure `.dag` Node-tree reader over
// resolved `Equals`/`Data` nodes when compile-graph access lands (gunbc#5364). Additive corpus-gate
// builtin seam — sibling to non_fold_residue_* / inert_carrier_*.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

use crate::cli_run::{corpus_dag_files, is_test_dag, strip_line_comment};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViolationKind {
    SameSymbolEquality,
    DataLiteralMirror,
}

impl ViolationKind {
    fn tag(self) -> &'static str {
        match self {
            Self::SameSymbolEquality => "same_symbol",
            Self::DataLiteralMirror => "literal_mirror",
        }
    }
}

fn strip_comments(content: &str) -> String {
    content
        .lines()
        .map(strip_line_comment)
        .collect::<Vec<_>>()
        .join("\n")
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn is_ident(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !s.chars().next().unwrap().is_ascii_digit()
}

fn normalize_literal(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn matching_brace(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut j = open;
    while j < bytes.len() {
        match bytes[j] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
        j += 1;
    }
    None
}

fn matching_paren(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut j = open;
    while j < bytes.len() {
        match bytes[j] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
        j += 1;
    }
    None
}

struct FnSig {
    name: String,
    body: String,
}

fn parse_test_fns(src: &str) -> Vec<FnSig> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    for (start, _) in src.match_indices("fn ") {
        let prefix_start = start.saturating_sub(6);
        let is_test = src[prefix_start..start].ends_with("test ");
        let is_witness = src[start..].starts_with("fn witness_");
        if !is_test && !is_witness {
            continue;
        }
        if start > 0 && is_ident_byte(bytes[start - 1]) && !is_test {
            continue;
        }
        let after = start + 3;
        let name: String = src[after..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            continue;
        }
        let paren_open = match src[after..].find('(') {
            Some(p) => after + p,
            None => continue,
        };
        let paren_close = match matching_paren(bytes, paren_open) {
            Some(p) => p,
            None => continue,
        };
        let brace_open = match src[paren_close..].find('{') {
            Some(b) => paren_close + b,
            None => continue,
        };
        let brace_close = match matching_brace(bytes, brace_open) {
            Some(b) => b,
            None => continue,
        };
        out.push(FnSig {
            name,
            body: src[brace_open + 1..brace_close].to_string(),
        });
    }
    out
}

fn bare_ident_before(bytes: &[u8], eq_pos: usize) -> Option<String> {
    let mut end = eq_pos;
    while end > 0 && (bytes[end - 1] == b' ' || bytes[end - 1] == b'\t' || bytes[end - 1] == b'\n') {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && is_ident_byte(bytes[start - 1]) {
        start -= 1;
    }
    if start == end {
        return None;
    }
    let ident = std::str::from_utf8(&bytes[start..end]).ok()?;
    if is_ident(ident) {
        Some(ident.to_string())
    } else {
        None
    }
}

fn bare_ident_after(bytes: &[u8], eq_pos: usize) -> Option<String> {
    let mut start = eq_pos;
    while start < bytes.len()
        && (bytes[start] == b' ' || bytes[start] == b'\t' || bytes[start] == b'\n')
    {
        start += 1;
    }
    let mut end = start;
    while end < bytes.len() && is_ident_byte(bytes[end]) {
        end += 1;
    }
    if start == end {
        return None;
    }
    let ident = std::str::from_utf8(&bytes[start..end]).ok()?;
    if is_ident(ident) {
        Some(ident.to_string())
    } else {
        None
    }
}

fn expr_after_eq(bytes: &[u8], eq_pos: usize) -> Option<String> {
    let mut start = eq_pos;
    while start < bytes.len()
        && (bytes[start] == b' ' || bytes[start] == b'\t' || bytes[start] == b'\n')
    {
        start += 1;
    }
    if start >= bytes.len() {
        return None;
    }
    let mut depth = 0i32;
    let mut end = start;
    while end < bytes.len() {
        match bytes[end] {
            b'(' | b'{' | b'[' => depth += 1,
            b')' | b'}' | b']' => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            b'&' if depth == 0 && end + 1 < bytes.len() && bytes[end + 1] == b'&' => break,
            b'|' if depth == 0 && end + 1 < bytes.len() && bytes[end + 1] == b'|' => break,
            b',' if depth == 0 => break,
            b'\n' if depth == 0 => break,
            _ => {}
        }
        end += 1;
    }
    let s = std::str::from_utf8(&bytes[start..end]).trim();
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

fn same_symbol_equalities(body: &str) -> usize {
    let bytes = body.as_bytes();
    let mut depth = 0i32;
    let mut count = 0usize;
    let mut i = 0usize;
    while i + 1 < bytes.len() {
        match bytes[i] {
            b'(' | b'{' | b'[' => depth += 1,
            b')' | b'}' | b']' => depth -= 1,
            b'=' if depth == 0 && i + 1 < bytes.len() && bytes[i + 1] == b'=' => {
                if let (Some(lhs), Some(rhs)) = (
                    bare_ident_before(bytes, i),
                    bare_ident_after(bytes, i + 2),
                ) {
                    if lhs == rhs {
                        count += 1;
                    }
                }
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }
    count
}

fn module_name_from_content(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("module ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '_')
                .collect();
            if !name.is_empty() {
                return Some(name);
            }
        }
    }
    None
}

fn build_module_index(files: &[(String, String)]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for (rel, content) in files {
        if let Some(m) = module_name_from_content(content) {
            out.insert(m, rel.clone());
        }
    }
    out
}

fn parse_imported_modules(content: &str) -> Vec<String> {
    let src = strip_comments(content);
    let mut out = Vec::new();
    for (start, _) in src.match_indices("import ") {
        let after = start + "import ".len();
        let name: String = src[after..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '_')
            .collect();
        if !name.is_empty() {
            out.push(name);
        }
    }
    out
}

fn parse_data_initializers(content: &str) -> BTreeMap<String, String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut out = BTreeMap::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        let Some(rest) = trimmed.strip_prefix("data ") else {
            i += 1;
            continue;
        };
        let name: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if name.is_empty() {
            i += 1;
            continue;
        }
        let mut block = String::new();
        block.push_str(&strip_line_comment(lines[i]));
        let eq_rel = block.find('=');
        if eq_rel.is_none() {
            i += 1;
            continue;
        }
        let mut depth = brace_delta_line(lines[i]);
        i += 1;
        while i < lines.len() && (depth > 0 || !is_top_level_decl_start(lines[i].trim_start())) {
            block.push('\n');
            block.push_str(&strip_line_comment(lines[i]));
            depth += brace_delta_line(lines[i]);
            i += 1;
        }
        let Some(eq_pos) = block.find('=') else {
            continue;
        };
        let literal = normalize_literal(block[eq_pos + 1..].trim());
        if !literal.is_empty() {
            out.insert(name, literal);
        }
    }
    out
}

fn brace_delta_line(line: &str) -> i32 {
    let mut delta = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for b in line.as_bytes() {
        if in_string {
            if escaped {
                escaped = false;
            } else if *b == b'\\' {
                escaped = true;
            } else if *b == b'"' {
                in_string = false;
            }
            continue;
        }
        match b {
            b'"' => in_string = true,
            b'{' => delta += 1,
            b'}' => delta -= 1,
            _ => {}
        }
    }
    delta
}

fn is_top_level_decl_start(trimmed: &str) -> bool {
    trimmed.starts_with("data ")
        || trimmed.starts_with("type ")
        || trimmed.starts_with("fn ")
        || trimmed.starts_with("test fn ")
        || trimmed.starts_with("service ")
        || trimmed.starts_with("import ")
        || trimmed.starts_with("module ")
}

fn literal_mirror_equalities(body: &str, data_literals: &BTreeMap<String, String>) -> usize {
    let bytes = body.as_bytes();
    let mut depth = 0i32;
    let mut count = 0usize;
    let mut i = 0usize;
    while i + 1 < bytes.len() {
        match bytes[i] {
            b'(' | b'{' | b'[' => depth += 1,
            b')' | b'}' | b']' => depth -= 1,
            b'=' if depth == 0 && i + 1 < bytes.len() && bytes[i + 1] == b'=' => {
                let lhs_ident = bare_ident_before(bytes, i);
                let rhs_expr = expr_after_eq(bytes, i + 2);
                if let (Some(name), Some(rhs_lit)) = (lhs_ident, rhs_expr) {
                    if let Some(init) = data_literals.get(&name) {
                        if normalize_literal(&rhs_lit) == *init {
                            count += 1;
                        }
                    }
                }
                i += 1;
            }
            _ => {}
        }
        i += 1;
    }
    count
}

fn authority_data_literals(
    test_content: &str,
    module_index: &BTreeMap<String, String>,
    file_literals: &BTreeMap<String, String>,
    files_by_path: &BTreeMap<String, &str>,
) -> BTreeMap<String, String> {
    let mut out = file_literals.clone();
    for imported in parse_imported_modules(test_content) {
        let Some(path) = module_index.get(&imported) else {
            continue;
        };
        let Some(content) = files_by_path.get(path.as_str()) else {
            continue;
        };
        for (k, v) in parse_data_initializers(content) {
            out.entry(k).or_insert(v);
        }
    }
    out
}

fn dual_representation_sites(files: &[(String, String)]) -> Vec<String> {
    let module_index = build_module_index(files);
    let files_by_path: BTreeMap<String, &str> = files
        .iter()
        .map(|(p, c)| (p.clone(), c.as_str()))
        .collect();
    let mut out = BTreeSet::new();
    for (rel, content) in files {
        if !is_test_dag(rel) {
            continue;
        }
        let file_literals = parse_data_initializers(content);
        let data_literals = authority_data_literals(
            content,
            &module_index,
            &file_literals,
            &files_by_path,
        );
        let src = strip_comments(content);
        for sig in parse_test_fns(&src) {
            let same = same_symbol_equalities(&sig.body);
            for _ in 0..same {
                out.insert(format!("{}::{}::{}", rel, sig.name, ViolationKind::SameSymbolEquality.tag()));
            }
            let mirror = literal_mirror_equalities(&sig.body, &data_literals);
            for _ in 0..mirror {
                out.insert(format!("{}::{}::{}", rel, sig.name, ViolationKind::DataLiteralMirror.tag()));
            }
        }
    }
    out.into_iter().collect()
}

// Named exception roster: sites acknowledged as tautological but retained until migrated.
// Empty at launch — the live census seeds any pre-existing debt here if needed.
const NO_DUAL_REPRESENTATION_TEST_ROSTER: &[&str] = &[];

struct NoDualReport {
    sites: Vec<String>,
    test_file_count: usize,
}

fn build_report() -> &'static NoDualReport {
    static REPORT: OnceLock<NoDualReport> = OnceLock::new();
    REPORT.get_or_init(|| {
        let files = corpus_dag_files();
        let test_file_count = files.iter().filter(|(p, _)| is_test_dag(p)).count();
        NoDualReport {
            sites: dual_representation_sites(&files),
            test_file_count,
        }
    })
}

pub fn no_dual_representation_test_count() -> i64 {
    build_report().sites.len() as i64
}

pub fn no_dual_representation_test_unrostered_count() -> i64 {
    let roster: BTreeSet<&str> = NO_DUAL_REPRESENTATION_TEST_ROSTER.iter().copied().collect();
    build_report()
        .sites
        .iter()
        .filter(|s| !roster.contains(s.as_str()))
        .count() as i64
}

pub fn no_dual_representation_test_stale_roster_count() -> i64 {
    let sites: BTreeSet<&String> = build_report().sites.iter().collect();
    NO_DUAL_REPRESENTATION_TEST_ROSTER
        .iter()
        .filter(|s| !sites.contains(&s.to_string()))
        .count() as i64
}

pub fn no_dual_representation_test_file_count() -> i64 {
    build_report().test_file_count as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn red_control_same_symbol_equality_is_flagged() {
        let sites = dual_representation_sites(&[(
            "t_test.dag",
            "module t\ntest fn taut() -> Bool {\n  let m = one()\n  return m == m\n}\n",
        )]);
        assert!(
            sites.iter().any(|s| s.contains("same_symbol")),
            "expected same_symbol violation; got {sites:?}"
        );
    }

    #[test]
    fn green_control_distinct_symbol_equality_is_not_flagged() {
        let sites = dual_representation_sites(&[(
            "t_test.dag",
            "module t\ntest fn ok() -> Bool {\n  return a == b\n}\n",
        )]);
        assert!(
            sites.is_empty(),
            "distinct identifiers must not be flagged; got {sites:?}"
        );
    }

    #[test]
    fn red_control_literal_mirror_is_flagged() {
        let sites = dual_representation_sites(&[(
            "t_test.dag",
            "module t\ndata ttl: Int = 3600\ntest fn witness_ttl() -> Bool {\n  return ttl == 3600\n}\n",
        )]);
        assert!(
            sites.iter().any(|s| s.contains("literal_mirror")),
            "expected literal_mirror violation; got {sites:?}"
        );
    }

    #[test]
    fn green_control_imported_data_reference_without_literal_mirror() {
        let sites = dual_representation_sites(&[
            (
                "auth.dag",
                "module auth\ndata ttl: Int = 3600\nfn ttl_seconds() -> Int { ttl }\n",
            ),
            (
                "auth_test.dag",
                "module auth_test\nimport auth { ttl_seconds }\ntest fn witness_ttl() -> Bool {\n  return ttl_seconds() == 3600\n}\n",
            ),
        ]);
        assert!(
            !sites.iter().any(|s| s.contains("literal_mirror")),
            "computed subject must not be flagged as literal mirror; got {sites:?}"
        );
    }

    #[test]
    fn red_control_imported_literal_mirror_is_flagged() {
        let sites = dual_representation_sites(&[
            (
                "auth.dag",
                "module auth\ndata ttl: Int = 3600\n",
            ),
            (
                "auth_test.dag",
                "module auth_test\nimport auth { ttl }\ntest fn witness_ttl() -> Bool {\n  return ttl == 3600\n}\n",
            ),
        ]);
        assert!(
            sites.iter().any(|s| s.contains("literal_mirror")),
            "imported data mirror must be flagged; got {sites:?}"
        );
    }
}
