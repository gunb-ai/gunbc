use std::collections::HashMap;
use std::sync::OnceLock;

pub fn workspace_root() -> std::path::PathBuf {
    crate::cli_run::workspace_root()
}

// The witness layer roots have a SINGLE authority — `gunbc.ci_layer_roots.witness_layer_roots` in the
// .dag substrate (DESIGN §3). The CI floor, the layering-imports / resolved-imports / compile-clean
// gates all derive their `--source-root` flags from it. The Rust censuses that walk "the witness
// corpus" (non_fold_residue, inert_carrier, external_authority) MUST derive their roots from the same
// authority, not a hardcoded `["dsl", "src/v2"]` copy: a frozen copy is a second representation that a
// sibling PR extending the authority silently invalidates, leaving the census blind to the new root and
// reding main on merge. So the roots are read live from the substrate — anchor-completeness by
// construction (DESIGN §5), the bad state (census roots ≠ authority roots) is unwritable, not flagged
// after the fact. (DISSOLUTION-aligned with the corpus walk: a pure .dag Node-tree read of the
// authority, the same direction gunbc#5364 takes the residue scan.)
const CI_LAYER_ROOTS_AUTHORITY_REL: &str = "dsl/gunbc/ci_layer_roots.dag";
const WITNESS_LAYER_ROOTS_DATA_NAME: &str = "witness_layer_roots";

/// Project the `witness_layer_roots` `List<String>` literal out of the ci_layer_roots authority's
/// SOURCE TEXT via the real front-end (`tokenize` + `parse`) — no second hand-rolled scanner. Pure
/// (text in, roots out) so a synthetic authority carrying non-default roots can drive it: a reader
/// that ignored its input and returned a hardcoded `["dsl", "src/v2"]` (the reverted shape) fails that
/// control — the by-construction discrimination (DESIGN §5). Fail-closed: a parse error, a missing data
/// def, a non-string-list body, or an empty list is a loud panic, never a silent fallback that would
/// re-open the drift.
pub(crate) fn witness_layer_roots_from_source(content: &str) -> Vec<String> {
    use crate::v1_std_core::{ExprData, LiteralValue};

    let filename = CI_LAYER_ROOTS_AUTHORITY_REL.to_string();
    let tokens = crate::v1_compiler_tokenize::tokenize(content.to_string(), filename.clone());
    let source_index =
        crate::v1_std_core::build_newline_index(filename.clone(), content.to_string());
    let mut source_indices = HashMap::new();
    source_indices.insert(filename.clone(), source_index);
    let result = crate::v1_compiler_parse::parse(tokens, std::rc::Rc::new(source_indices));
    if let Some(err) = result.error.as_ref() {
        panic!(
            "ci_layer_roots authority: parse error in {CI_LAYER_ROOTS_AUTHORITY_REL}: {}",
            crate::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
        );
    }
    let module = result.module.as_ref().unwrap_or_else(|| {
        panic!("ci_layer_roots authority: {CI_LAYER_ROOTS_AUTHORITY_REL} parsed to no module")
    });
    for item in module.children.iter() {
        if item.name != WITNESS_LAYER_ROOTS_DATA_NAME
            || !crate::v1_compiler_emit_core_support::is_data_def_item(item.clone())
        {
            continue;
        }
        let body = item.body.as_ref().unwrap_or_else(|| {
            panic!(
                "ci_layer_roots authority: `data {WITNESS_LAYER_ROOTS_DATA_NAME}` in \
                 {CI_LAYER_ROOTS_AUTHORITY_REL} has no value body"
            )
        });
        if !matches!(body.expr_data.as_ref(), ExprData::ExprListLit) {
            panic!(
                "ci_layer_roots authority: `data {WITNESS_LAYER_ROOTS_DATA_NAME}` in \
                 {CI_LAYER_ROOTS_AUTHORITY_REL} is not a `List<String>` literal"
            );
        }
        let mut roots = Vec::new();
        for el in body.children.iter() {
            match el.expr_data.as_ref() {
                ExprData::ExprLiteral { value } => match value.as_ref() {
                    LiteralValue::LitStr { value } => roots.push(value.clone()),
                    _ => panic!(
                        "ci_layer_roots authority: an element of `{WITNESS_LAYER_ROOTS_DATA_NAME}` in \
                         {CI_LAYER_ROOTS_AUTHORITY_REL} is not a string literal"
                    ),
                },
                _ => panic!(
                    "ci_layer_roots authority: an element of `{WITNESS_LAYER_ROOTS_DATA_NAME}` in \
                     {CI_LAYER_ROOTS_AUTHORITY_REL} is not a literal"
                ),
            }
        }
        if roots.is_empty() {
            panic!(
                "ci_layer_roots authority: `{WITNESS_LAYER_ROOTS_DATA_NAME}` in \
                 {CI_LAYER_ROOTS_AUTHORITY_REL} is empty (fail-closed: an empty witness corpus would \
                 vacuously pass every census wall)"
            );
        }
        return roots;
    }
    panic!(
        "ci_layer_roots authority: no `data {WITNESS_LAYER_ROOTS_DATA_NAME}` def in \
         {CI_LAYER_ROOTS_AUTHORITY_REL}"
    )
}

/// The witness layer roots, read live from the single .dag authority and memoized (the authority is
/// fixed for a process's lifetime). Repo-relative roots (e.g. `dsl`, `src/v2`); consumers join them
/// onto `workspace_root()`. Every Rust census derives its corpus from HERE — see the module note.
pub(crate) fn witness_layer_roots() -> Vec<String> {
    static ROOTS: OnceLock<Vec<String>> = OnceLock::new();
    ROOTS
        .get_or_init(|| {
            let path = workspace_root().join(CI_LAYER_ROOTS_AUTHORITY_REL);
            let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
                panic!(
                    "ci_layer_roots authority: failed to read {}: {e}",
                    path.display()
                )
            });
            witness_layer_roots_from_source(&content)
        })
        .clone()
}

/// By-execution wall for the authority-derivation construction (DESIGN §5, red-on-revert): the
/// layer-roots reader must FOLLOW the authority, not a hardcoded copy. Driven by a synthetic authority
/// carrying three non-default roots — a reverted reader that returned `["dsl", "src/v2"]` regardless of
/// input fails the control. The live arm additionally proves the real authority parses to a non-empty
/// root set (the fail-closed oracle: an empty corpus would vacuously pass the census walls).
pub fn census_corpus_roots_follow_layer_authority() -> bool {
    let synthetic = "module gunbc.ci_layer_roots\n\n\
         data witness_layer_roots: List<String> = [\"alpha_layer_root\", \"beta_layer_root\", \"gamma_layer_root\"]\n";
    let follows = witness_layer_roots_from_source(synthetic)
        == ["alpha_layer_root", "beta_layer_root", "gamma_layer_root"];
    let live_nonempty = !witness_layer_roots().is_empty();
    follows && live_nonempty
}

pub(crate) fn default_source_roots() -> Vec<String> {
    let ws = workspace_root();
    witness_layer_roots()
        .iter()
        .map(|r| ws.join(r).to_string_lossy().into_owned())
        .collect()
}

pub fn build_module_path_index() -> HashMap<String, String> {
    crate::cli_run::build_module_path_index(&default_source_roots())
}

pub fn source_path_for_module_path(module_path: String) -> String {
    let index = build_module_path_index();
    index
        .get(&module_path)
        .cloned()
        .unwrap_or_else(|| panic!("module_path_index: unknown module path '{module_path}'"))
}

pub fn qualified_name_value_to_module_path(value: &crate::v1_interpreter::Value) -> String {
    crate::v1_interpreter::qualified_name_value_to_module_path(value)
}

pub fn qualified_name_value_from_dotted_string(
    ctx: &crate::v1_interpreter::InterpContext,
    dotted: &str,
) -> crate::v1_interpreter::Value {
    use crate::v1_interpreter::{sorted_fields, Value};
    use std::rc::Rc;

    let qn_variant = |variant: &str, fields: Vec<_>| Value::Variant {
        type_name: ctx.sym("QualifiedName"),
        variant_name: ctx.sym(variant),
        fields: Rc::new(fields),
    };
    if dotted.is_empty() {
        return qn_variant("QnEmpty", vec![]);
    }
    let mut qn = qn_variant("QnEmpty", vec![]);
    for seg in dotted.split('.').rev() {
        qn = qn_variant(
            "QnCons",
            sorted_fields(vec![
                (ctx.sym("head"), Value::Str(seg.to_string())),
                (ctx.sym("tail"), qn),
            ]),
        );
    }
    qn
}

pub(crate) fn repo_rel(path: &std::path::Path) -> String {
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

pub(crate) fn collect_dag_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
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

pub(crate) fn corpus_dag_files() -> Vec<(String, String)> {
    let mut paths = Vec::new();
    for root in witness_layer_roots() {
        collect_dag_files(&workspace_root().join(&root), &mut paths);
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
            break;
        } else {
            out.push(b);
        }
        i += 1;
    }
    String::from_utf8(out).expect("strip_line_comment output is valid UTF-8")
}

pub(crate) fn brace_delta(line: &str) -> i32 {
    let c = strip_line_comment(line);
    c.matches('{').count() as i32 - c.matches('}').count() as i32
}

#[cfg(test)]
mod corpus_lex_tests {
    use super::*;

    #[test]
    fn strip_blanks_string_interior_and_drops_comment() {
        let got = strip_line_comment("data u = \"https://x // y\" // real comment");
        assert!(got.starts_with("data u = \""));
        assert!(
            !got.contains("real comment"),
            "trailing // comment dropped: {got:?}"
        );
        assert!(!got.contains("https"), "string interior blanked: {got:?}");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_build_resolves_by_module_path_not_directory_nickname() {
        let path = source_path_for_module_path("extdeps.cargo_build".to_string());
        assert_eq!(path, "dsl/extdeps/rust/cargo_build.dag");
    }

    #[test]
    fn git_module_resolves() {
        let path = source_path_for_module_path("extdeps.git".to_string());
        assert_eq!(path, "dsl/extdeps/git/git.dag");
    }

    #[test]
    fn co_root_overlay_last_root_wins_on_duplicate_module_path() {
        let path = source_path_for_module_path("extdeps.shell".to_string());
        assert_eq!(path, "src/v2/extdeps/shell.dag");
    }

    #[test]
    fn reader_follows_synthetic_authority_with_nondefault_roots() {
        // RED-ON-REVERT discrimination: a synthetic authority carrying three NON-default roots — the
        // reader must return exactly them. A reverted, re-hardcoded reader (returning `[dsl, src/v2]`
        // regardless of input) fails this. This is the construction's discriminating witness.
        let synthetic = "module gunbc.ci_layer_roots\n\n\
             data witness_layer_roots: List<String> = [\"r_one\", \"r_two\", \"r_three\"]\n";
        assert_eq!(
            witness_layer_roots_from_source(synthetic),
            vec![
                "r_one".to_string(),
                "r_two".to_string(),
                "r_three".to_string()
            ],
            "the layer-roots reader must FOLLOW the authority, not a hardcoded copy"
        );
    }

    #[test]
    fn reader_projects_live_authority_value() {
        // GREEN wire check: the real authority file projects to its current value, and the derived
        // source roots / corpus self-check agree.
        assert_eq!(
            witness_layer_roots(),
            vec!["dsl".to_string(), "src/v2".to_string()],
            "live authority value drifted from the expected [dsl, src/v2]"
        );
        assert!(
            census_corpus_roots_follow_layer_authority(),
            "census corpus roots must derive from the layer-roots authority"
        );
    }

    #[test]
    fn default_source_roots_derive_from_authority() {
        // default_source_roots is the authority joined onto workspace_root — not a parallel literal.
        let ws = workspace_root();
        assert_eq!(
            default_source_roots(),
            vec![
                ws.join("dsl").to_string_lossy().into_owned(),
                ws.join("src/v2").to_string_lossy().into_owned(),
            ]
        );
    }
}
