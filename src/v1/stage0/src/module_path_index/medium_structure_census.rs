use im_rc::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::cli_run::collect_dag_files_tolerant;
use crate::v1_compiler_parse::parse;
use crate::v1_compiler_tokenize::tokenize;
use crate::v1_std_core::{
    arg_value, build_newline_index, expr_call_func_at, field_init_node_name_at,
    field_init_node_value, record_lit_type_name_at, ExprData, LiteralValue, NewlineIndex, Node,
};

type SourceIndices = Rc<HashMap<String, Rc<NewlineIndex>>>;

fn rel_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Parse-only module items for one `.dag` file (no resolve). Shared substrate for
/// `decl_facts(roots)` (#5966) and emit-only corpus audits.
pub struct ParsedDagFile {
    pub items: Rc<im_rc::Vector<Rc<Node>>>,
    pub source_indices: SourceIndices,
}

pub fn parse_dag_file(path: &Path) -> Option<ParsedDagFile> {
    let content = std::fs::read_to_string(path).ok()?;
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("?");
    let tokens = tokenize(content.clone(), filename.to_string());
    let source_index = build_newline_index(filename.to_string(), content);
    let mut indices = HashMap::new();
    indices.insert(filename.to_string(), source_index);
    let source_indices: SourceIndices = Rc::new(indices);
    let result = parse(tokens, source_indices.clone());
    if result.error.is_some() {
        return None;
    }
    let module = result.module.as_ref()?;
    Some(ParsedDagFile {
        items: module.children.clone(),
        source_indices,
    })
}

pub fn parse_file(path: &Path) -> Option<(Rc<im_rc::Vector<Rc<Node>>>, SourceIndices)> {
    parse_dag_file(path).map(|parsed| (parsed.items, parsed.source_indices))
}

fn item_value_bodies(items: &Rc<im_rc::Vector<Rc<Node>>>) -> Vec<Rc<Node>> {
    items.iter().filter_map(|item| item.body.clone()).collect()
}

fn dag_files_sorted(root: &str) -> Vec<PathBuf> {
    let root_path = Path::new(root);
    if !root_path.is_dir() {
        return Vec::new();
    }
    let mut files: Vec<PathBuf> = Vec::new();
    collect_dag_files_tolerant(root_path, &mut files);
    files.sort();
    files
}

// ---- no-smuggled-programs wall (HALF B): grammar-ignorant field-expression parts projection ----
//
// The Hole producer for `v2.lens.medium_structure_containment` grammar classifier. GRAMMAR- AND
// LANGUAGE-IGNORANT (ruling R1b): this walks record CONSTRUCTIONS (`ExprRecordLit`) whose
// `Constructor.field` matches a CALLER-SUPPLIED target ("RawLine.text", "Heredoc.body"), and
// projects each such field's VALUE expression into a neutral parts list `[Const(text)|Hole]`. It
// assigns NO meaning — the `.dag` lens owns the language decision (RawLine.text -> bash classify;
// Heredoc.body -> NoLanguageDecision) and the composition recognizer. Projection: a string literal
// -> Const(text); `concat(..)` args flattened; string interpolation decomposed (text segment ->
// Const, interpolated expr -> Hole); any other dynamic sub-expression -> Hole (a .dag COMPOSE-TIME
// value spliced into the source before the target language parses it — never assume-Const).
//
// This REPLACES the deleted marker mechanism (medium_structure_leak_facts + text.contains) in one
// motion with the .dag `medium_syntax_markers` deletion — single authority is the grammar, not a
// marker set.
//
// SCAFFOLD (§7 hand-Rust shrink-to-zero): rides the `marshal_string_literal_atom` literal-
// preservation idiom (coproduct_reflection.rs) and the #5364 field-expression projection corridor;
// dissolves when the projection is a modeled substrate fold rather than a hand-Rust walk here.
//
// §3 note (review, 2026-07-15): `RawLiteralPartsFact.{constructor,field}` are two String tags used as
// an IN-BAND discriminator that `classify_raw_literal_fact` matches against ("RawLine","text") else
// NoLanguageDecision — an implicit sum across the seed<->.dag seam. It stays String-shaped only while
// this walk is hand-Rust and the target set is the 2-element `medium_bash_sink_targets`. DISSOLVES-ON
// the same trigger as this scaffold: when the projection moves in-substrate the caller-supplied target
// vector becomes a typed sink COPRODUCT (a variant per smuggling-capable field), and the (constructor,
// field) string pair is replaced by that coproduct's discriminant — no in-band string sum remains.

pub struct LiteralPartRaw {
    pub is_hole: bool,
    pub text: String,
}

pub struct RawLiteralPartsFact {
    pub path: String,
    pub constructor: String,
    pub field: String,
    pub parts: Vec<LiteralPartRaw>,
}

// Project a field-value expression into neutral [Const(text)|Hole] parts. Fail-closed: any
// sub-expression that is not a preserved literal segment becomes a Hole (a compose-time splice),
// never a silent Const.
fn project_field_expr_parts(node: &Rc<Node>, si: &SourceIndices, out: &mut Vec<LiteralPartRaw>) {
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitStr { value: s, .. } => out.push(LiteralPartRaw {
                is_hole: false,
                text: s.clone(),
            }),
            _ => out.push(LiteralPartRaw {
                is_hole: true,
                text: String::new(),
            }),
        },
        ExprData::ExprCall { .. } => {
            let callee = expr_call_func_at(node.clone(), si.clone());
            if callee == "concat" {
                for child in node.children.iter() {
                    project_field_expr_parts(&arg_value(child.clone()), si, out);
                }
            } else {
                out.push(LiteralPartRaw {
                    is_hole: true,
                    text: String::new(),
                });
            }
        }
        ExprData::ExprStringInterp => {
            for child in node.children.iter() {
                project_field_expr_parts(child, si, out);
            }
        }
        _ => out.push(LiteralPartRaw {
            is_hole: true,
            text: String::new(),
        }),
    }
}

// An empty literal (RawLine{text:""}, a fold sentinel/no-op line) has no program to classify, so
// it is not emitted as a fact — "must not classify" (parent review 2026-07-14). Match-arm PATTERNS
// (`RawLine{text:t} =>`) are `MatchPattern::VariantPattern`, not `ExprData::ExprRecordLit`, so this
// walk excludes them by construction — only genuine record CONSTRUCTIONS reach here.
fn parts_are_empty(parts: &[LiteralPartRaw]) -> bool {
    parts.iter().all(|p| !p.is_hole && p.text.is_empty())
}

fn walk_record_lit_parts(
    node: &Rc<Node>,
    targets: &[(String, String)],
    path: &str,
    si: &SourceIndices,
    out: &mut Vec<RawLiteralPartsFact>,
) {
    if let ExprData::ExprRecordLit { .. } = node.expr_data.as_ref() {
        if let Some(type_name) = record_lit_type_name_at(node.clone(), si.clone()) {
            for (target_ctor, target_field) in targets {
                if target_ctor == &type_name {
                    for f in node.children.iter() {
                        let fname = field_init_node_name_at(f.clone(), si.clone());
                        if &fname == target_field {
                            let value = field_init_node_value(f.clone());
                            let mut parts: Vec<LiteralPartRaw> = Vec::new();
                            project_field_expr_parts(&value, si, &mut parts);
                            if !parts_are_empty(&parts) {
                                out.push(RawLiteralPartsFact {
                                    path: path.to_string(),
                                    constructor: type_name.clone(),
                                    field: fname.clone(),
                                    parts,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    for child in node.children.iter() {
        walk_record_lit_parts(child, targets, path, si, out);
    }
    if let Some(body) = node.body.as_ref() {
        walk_record_lit_parts(body, targets, path, si, out);
    }
}

fn parse_targets(targets: &[String]) -> Vec<(String, String)> {
    targets
        .iter()
        .filter_map(|t| {
            let mut it = t.splitn(2, '.');
            let ctor = it.next()?.to_string();
            let field = it.next()?.to_string();
            if ctor.is_empty() || field.is_empty() {
                None
            } else {
                Some((ctor, field))
            }
        })
        .collect()
}

// A cheap grammar-ignorant pre-gate: skip a file whose text contains none of the target
// constructor names. This is a string the CALLER supplied (a name, not a grammar marker), so the
// seed still assigns no language meaning; it only avoids parsing files with no candidate site.
fn any_target_ctor_present(content: &str, targets: &[(String, String)]) -> bool {
    targets
        .iter()
        .any(|(ctor, _)| !ctor.is_empty() && content.contains(ctor.as_str()))
}

pub fn medium_structure_literal_parts_facts(
    roots: &[String],
    targets: &[String],
) -> Vec<RawLiteralPartsFact> {
    let parsed = parse_targets(targets);
    let mut out: Vec<RawLiteralPartsFact> = Vec::new();
    for root in roots {
        for file in dag_files_sorted(root) {
            let Ok(content) = std::fs::read_to_string(&file) else {
                continue;
            };
            if !any_target_ctor_present(&content, &parsed) {
                continue;
            }
            if let Some((items, si)) = parse_file(&file) {
                let rel = rel_path(&file);
                for body in item_value_bodies(&items) {
                    walk_record_lit_parts(&body, &parsed, &rel, &si, &mut out);
                }
            }
        }
    }
    out.sort_by(|a, b| {
        a.path
            .cmp(&b.path)
            .then(a.constructor.cmp(&b.constructor))
            .then(a.field.cmp(&b.field))
    });
    out
}
