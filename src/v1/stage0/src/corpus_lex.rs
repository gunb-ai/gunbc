// Shared corpus walk + lexical normalization for the host-fed census lenses (DESIGN §2/§3: one
// authority for "what is code text", not a copy per census module). The inert-carrier and
// non-fold-residue censuses both read the same `.dag` corpus and both need the same string-literal-
// aware comment stripping; forking either would be the redundancy §2/§3 reject (and would widen the
// bug surface — a future fix to e.g. raw-string handling would have to be made in two places).
//
// Host-fed today; DISSOLUTION: folds into a pure `.dag` Node-tree reader (the compile graph already
// carries each module's source + node structure) when gunbc#5364 (.dag compile-graph / BindsTo
// access) lands — at which point the token scan here is replaced by structural Node reads.

use std::path::{Path, PathBuf};

pub(crate) fn workspace_root() -> PathBuf {
    crate::module_path_index::workspace_root()
}

pub(crate) fn repo_rel(path: &Path) -> String {
    let ws = workspace_root();
    let s = path.to_string_lossy().replace('\\', "/");
    let prefix = format!("{}/", ws.to_string_lossy().replace('\\', "/"));
    s.strip_prefix(&prefix)
        .map(|p| p.to_string())
        .unwrap_or(s)
        .trim_start_matches("./")
        .to_string()
}

pub(crate) fn is_test_dag(path: &str) -> bool {
    path.ends_with("_test.dag")
}

pub(crate) fn collect_dag_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_dag_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("dag") {
            out.push(path);
        }
    }
}

/// The live `.dag` corpus: every `*.dag` under `dsl/` + `src/v2/`, keyed by repo-relative path,
/// sorted and deduped. The two trees match `gunbc.ci_layer_roots` (`witness_layer_roots`).
pub(crate) fn corpus_dag_files() -> Vec<(String, String)> {
    let mut paths = Vec::new();
    for root in ["dsl", "src/v2"] {
        collect_dag_files(&workspace_root().join(root), &mut paths);
    }
    let mut out = Vec::new();
    for p in paths {
        let rel = repo_rel(&p);
        if let Ok(content) = std::fs::read_to_string(&p) {
            out.push((rel, content));
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Lexically normalize one line: drop the trailing `//` comment AND blank the interior of every
/// `"..."` string literal (each interior byte → a space, delimiters kept). String-literal awareness
/// is the single authority for "what is code text": a `//` inside a URL string (`"https://..."`) is
/// not a comment start, and `_ =>`/`{`/`}`/`|`/an identifier inside a string literal is text, not a
/// wildcard arm / brace / coproduct separator / consumer — so none of it is read as code by the
/// downstream scanners. Byte length up to the comment is preserved (interior chars blanked 1:1,
/// including multi-byte continuation bytes) so byte-offset brace matching stays stable.
pub(crate) fn strip_line_comment(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            if escaped {
                out.push(b' ');
                escaped = false;
            } else if b == b'\\' {
                out.push(b' ');
                escaped = true;
            } else if b == b'"' {
                out.push(b'"');
                in_string = false;
            } else {
                out.push(b' ');
            }
        } else if b == b'"' {
            in_string = true;
            out.push(b'"');
        } else if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            break; // real (out-of-string) line comment — drop the rest of the line.
        } else {
            out.push(b);
        }
        i += 1;
    }
    // Only ASCII bytes (space, `"`) or original out-of-string bytes are pushed; every string-interior
    // byte (incl. multi-byte continuation bytes) is replaced with a space, so the result is valid UTF-8.
    String::from_utf8(out).expect("strip_line_comment output is valid UTF-8")
}

/// Net brace depth of one line, comments/strings stripped first (so a `{`/`}` inside a string or
/// comment does not perturb the count).
pub(crate) fn brace_delta(line: &str) -> i32 {
    let c = strip_line_comment(line);
    c.matches('{').count() as i32 - c.matches('}').count() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_blanks_string_interior_and_drops_comment() {
        // `//` inside a string survives as blanked interior; the trailing real comment is dropped.
        let got = strip_line_comment("data u = \"https://x // y\" // real comment");
        assert!(got.starts_with("data u = \""));
        assert!(
            !got.contains("real comment"),
            "trailing // comment dropped: {got:?}"
        );
        assert!(!got.contains("https"), "string interior blanked: {got:?}");
        // byte length preserved up to the dropped comment (interior chars → spaces 1:1).
        assert!(got.len() <= "data u = \"https://x // y\" // real comment".len());
    }

    #[test]
    fn brace_delta_ignores_braces_in_strings() {
        assert_eq!(brace_delta("fn f() {"), 1);
        assert_eq!(brace_delta("let s = \"{ { {\""), 0);
        assert_eq!(brace_delta("} // }"), -1);
    }

    #[test]
    fn is_test_dag_matches_suffix() {
        assert!(is_test_dag("src/v2/lens/x_test.dag"));
        assert!(!is_test_dag("src/v2/lens/x.dag"));
    }
}
