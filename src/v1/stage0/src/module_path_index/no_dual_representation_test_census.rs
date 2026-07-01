// The no-dual-representation-test audit (DESIGN.md §5 construction-not-validation, recursive on the
// test corpus). A witness that mirrors the authority it claims to exercise is specification-without-
// execution — fluent, type-checking, and uninformative when wrong (DESIGN §5).
//
// WALL THE DECIDABLE SUBSET ONLY, HONESTLY (Rice: full "test adds no info" is undecidable):
//   (1) same-provenance equality — Eq whose two sides are the same bare identifier (X == X).
//   (2) RHS bare-literal duplicating a single-authority data initializer — `name == <lit>` where
//       `data name = <lit>` is in scope; independent oracles like `add(2,2) == 4` stay green.
//   (3) RHS structurally alpha-equivalent to a tested fn body — witness compares a call to F with an
//       expression alpha-equivalent to inlining F's body (the #6070 plan_w == floor_w shape).
//
// Host-fed today (parse-only Node-tree walk over `*_test.dag` witnesses); DISSOLUTION: pure `.dag`
// reader when compile-graph access lands (gunbc#5364). Additive corpus-gate builtin seam.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

use crate::cli_run::{corpus_dag_files, is_test_dag};
use crate::module_path_index::medium_structure_census::parse_dag_file;
use crate::v1_compiler_infer_items::{item_kind, ItemKind};
use crate::v1_std_core::{
    authored_name_at, binop_left, binop_right, expr_call_func_at, expr_var_name_at, let_body,
    let_binding_name_at, let_value, param_node_name_at, BinOp, ExprData, LiteralValue, Node,
    NewlineIndex,
};

type SourceIndices = Rc<HashMap<String, Rc<NewlineIndex>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViolationKind {
    SameSymbolEquality,
    DataLiteralMirror,
    BodyAlphaEquivalent,
}

impl ViolationKind {
    fn tag(self) -> &'static str {
        match self {
            Self::SameSymbolEquality => "same_symbol",
            Self::DataLiteralMirror => "literal_mirror",
            Self::BodyAlphaEquivalent => "body_mirror",
        }
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("workspace root")
        .to_path_buf()
}

fn resolve_rel_path(rel: &str) -> PathBuf {
    let candidate = Path::new(rel);
    if candidate.is_file() {
        return candidate.to_path_buf();
    }
    workspace_root().join(rel)
}

fn scan_test_decl_lines(content: &str) -> Vec<(String, i64)> {
    let mut out = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        let rest = trimmed
            .strip_prefix("test fn ")
            .or_else(|| trimmed.strip_prefix("test data "));
        if let Some(rest) = rest {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.push((name, (i + 1) as i64));
            }
        }
    }
    out
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

fn item_fn_name(item: &Rc<Node>, si: &SourceIndices) -> String {
    authored_name_at(si.clone(), item.clone())
}

fn is_fn_item(item: &Rc<Node>) -> bool {
    matches!(
        item_kind(item.clone()),
        ItemKind::FuncItem | ItemKind::FnItem
    )
}

fn is_data_item(item: &Rc<Node>) -> bool {
    item_kind(item.clone()) == ItemKind::DataItem
}

fn literal_fingerprint(node: &Rc<Node>) -> Option<String> {
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitStr { value: s } => Some(format!("str:{s}")),
            LiteralValue::LitInt { value: n } => Some(format!("int:{n}")),
            LiteralValue::LitFloat { value: s } => Some(format!("float:{s}")),
            LiteralValue::LitBool { value: b } => Some(format!("bool:{b}")),
            LiteralValue::LitNull => Some("null".to_string()),
            LiteralValue::LitSymbol { value: s } => Some(format!("sym:{s}")),
        },
        _ => None,
    }
}

fn node_fingerprint(node: &Rc<Node>, si: &SourceIndices) -> String {
    match node.expr_data.as_ref() {
        ExprData::ExprVar { .. } => format!("var:{}", expr_var_name_at(node.clone(), si.clone())),
        ExprData::ExprLiteral { .. } => {
            literal_fingerprint(node).unwrap_or_else(|| "lit:?".to_string())
        }
        ExprData::ExprCall { .. } => {
            let func = expr_call_func_at(node.clone(), si.clone());
            let args: Vec<String> = node
                .children
                .iter()
                .skip(1)
                .map(|a| node_fingerprint(a, si))
                .collect();
            format!("call:{func}({})", args.join(","))
        }
        ExprData::ExprFieldAccess { .. } => {
            let base = node
                .children
                .first()
                .map(|b| node_fingerprint(b, si))
                .unwrap_or_default();
            let field = node
                .children
                .get(1)
                .map(|f| authored_name_at(si.clone(), f.clone()))
                .unwrap_or_default();
            format!("field:{base}.{field}")
        }
        ExprData::ExprBinOp { op, .. } => {
            let lhs = node_fingerprint(&binop_left(node.clone()), si);
            let rhs = node_fingerprint(&binop_right(node.clone()), si);
            format!("binop:{op:?}({lhs},{rhs})")
        }
        ExprData::ExprUnaryOp { .. } => {
            let operand = node_fingerprint(&node.children[0], si);
            format!("unary({operand})")
        }
        ExprData::ExprLet { .. } => {
            let name = let_binding_name_at(node.clone(), si.clone());
            let val = node_fingerprint(&let_value(node.clone()), si);
            let body = let_body(node.clone())
                .map(|b| node_fingerprint(&b, si))
                .unwrap_or_default();
            format!("let:{name}={val};{body}")
        }
        ExprData::ExprBlock { .. } => {
            let parts: Vec<String> = node
                .children
                .iter()
                .map(|c| node_fingerprint(c, si))
                .collect();
            format!("block[{}]", parts.join(";"))
        }
        _ => format!("other:{:?}", node.expr_data),
    }
}

fn nodes_alpha_equal(a: &Rc<Node>, b: &Rc<Node>, si: &SourceIndices) -> bool {
    node_fingerprint(a, si) == node_fingerprint(b, si)
}

fn substitute_params(
    body: &Rc<Node>,
    params: &[String],
    args: &[Rc<Node>],
    si: &SourceIndices,
) -> Rc<Node> {
    let mut map: BTreeMap<String, Rc<Node>> = BTreeMap::new();
    for (p, a) in params.iter().zip(args.iter()) {
        map.insert(p.clone(), a.clone());
    }
    substitute_vars(body, &map, si)
}

fn substitute_vars(
    node: &Rc<Node>,
    map: &BTreeMap<String, Rc<Node>>,
    si: &SourceIndices,
) -> Rc<Node> {
    if let ExprData::ExprVar { .. } = node.expr_data.as_ref() {
        let name = expr_var_name_at(node.clone(), si.clone());
        if let Some(repl) = map.get(&name) {
            return repl.clone();
        }
    }
    let new_children: Rc<Vec<Rc<Node>>> = Rc::new(
        node.children
            .iter()
            .map(|c| substitute_vars(c, map, si))
            .collect(),
    );
    Rc::new(Node {
        children: new_children,
        ..(*node).clone()
    })
}

fn collect_eq_sites(node: &Rc<Node>, si: &SourceIndices, out: &mut Vec<(Rc<Node>, Rc<Node>)>) {
    if let ExprData::ExprBinOp { op, .. } = node.expr_data.as_ref() {
        if matches!(op, BinOp::Eq) {
            out.push((binop_left(node.clone()), binop_right(node.clone())));
        }
    }
    for child in node.children.iter() {
        collect_eq_sites(child, si, out);
    }
}

fn is_bare_var(node: &Rc<Node>, si: &SourceIndices) -> Option<String> {
    match node.expr_data.as_ref() {
        ExprData::ExprVar { .. } => Some(expr_var_name_at(node.clone(), si.clone())),
        _ => None,
    }
}

fn is_independent_call(node: &Rc<Node>) -> bool {
    matches!(node.expr_data.as_ref(), ExprData::ExprCall { .. })
}

fn fn_param_names(item: &Rc<Node>, si: &SourceIndices) -> Vec<String> {
    item.params
        .iter()
        .map(|p| param_node_name_at(p.clone(), si.clone()))
        .filter(|n| !n.is_empty())
        .collect()
}

fn call_args(node: &Rc<Node>) -> Vec<Rc<Node>> {
    node.children.iter().skip(1).cloned().collect()
}

struct FnRecord {
    body: Rc<Node>,
    params: Vec<String>,
}

struct ModuleRecords {
    fns: BTreeMap<String, FnRecord>,
    data_inits: BTreeMap<String, Rc<Node>>,
}

struct CorpusIndex {
    module_to_path: BTreeMap<String, String>,
    by_path: BTreeMap<String, ModuleRecords>,
}

fn build_corpus_index(files: &[(String, String)]) -> CorpusIndex {
    let mut module_to_path = BTreeMap::new();
    let mut by_path = BTreeMap::new();
    for (rel, content) in files {
        let path = resolve_rel_path(rel);
        let Some(parsed) = parse_dag_file(&path) else {
            continue;
        };
        let si = parsed.source_indices.clone();
        let module = module_name_from_content(content).unwrap_or_else(|| rel.clone());
        module_to_path.insert(module.clone(), rel.clone());
        let mut fns = BTreeMap::new();
        let mut data_inits = BTreeMap::new();
        for item in parsed.items.iter() {
            if is_fn_item(item) {
                let name = item_fn_name(item, &si);
                if let Some(body) = item.body.clone() {
                    fns.insert(
                        name,
                        FnRecord {
                            body,
                            params: fn_param_names(item, &si),
                        },
                    );
                }
            } else if is_data_item(item) {
                let name = item_fn_name(item, &si);
                if let Some(body) = item.body.clone() {
                    data_inits.insert(name, body);
                }
            }
        }
        by_path.insert(rel.clone(), ModuleRecords { fns, data_inits });
    }
    CorpusIndex {
        module_to_path,
        by_path,
    }
}

fn imported_modules(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("import ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '_')
                .collect();
            if !name.is_empty() {
                out.push(name);
            }
        }
    }
    out
}

fn scoped_data_inits(
    rel: &str,
    content: &str,
    index: &CorpusIndex,
) -> BTreeMap<String, Rc<Node>> {
    let mut out = BTreeMap::new();
    if let Some(rec) = index.by_path.get(rel) {
        out.extend(rec.data_inits.clone());
    }
    for module in imported_modules(content) {
        let Some(path) = index.module_to_path.get(&module) else {
            continue;
        };
        if let Some(rec) = index.by_path.get(path) {
            for (k, v) in &rec.data_inits {
                out.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
    }
    out
}

fn scoped_fns(rel: &str, content: &str, index: &CorpusIndex) -> BTreeMap<String, FnRecord> {
    let mut out = BTreeMap::new();
    if let Some(rec) = index.by_path.get(rel) {
        out.extend(
            rec.fns
                .iter()
                .map(|(k, v)| (k.clone(), FnRecord { body: v.body.clone(), params: v.params.clone() })),
        );
    }
    for module in imported_modules(content) {
        let Some(path) = index.module_to_path.get(&module) else {
            continue;
        };
        if let Some(rec) = index.by_path.get(path) {
            for (k, v) in &rec.fns {
                out.entry(k.clone()).or_insert_with(|| FnRecord {
                    body: v.body.clone(),
                    params: v.params.clone(),
                });
            }
        }
    }
    out
}

fn witness_fn_names(content: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for (name, _) in scan_test_decl_lines(content) {
        names.insert(name);
    }
    for line in content.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("fn witness_") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                names.insert(format!("witness_{name}"));
            }
        }
    }
    names
}

fn classify_eq_violations(
    rel: &str,
    fn_name: &str,
    body: &Rc<Node>,
    si: &SourceIndices,
    data_inits: &BTreeMap<String, Rc<Node>>,
    fns: &BTreeMap<String, FnRecord>,
) -> Vec<String> {
    let mut violations = Vec::new();
    let mut eqs = Vec::new();
    collect_eq_sites(body, si, &mut eqs);
    for (lhs, rhs) in eqs {
        if let (Some(a), Some(b)) = (is_bare_var(&lhs, si), is_bare_var(&rhs, si)) {
            if a == b {
                violations.push(format!(
                    "{}::{}::{}",
                    rel,
                    fn_name,
                    ViolationKind::SameSymbolEquality.tag()
                ));
                continue;
            }
        }
        if let (Some(name), Some(lit_fp)) = (is_bare_var(&lhs, si), literal_fingerprint(&rhs)) {
            if let Some(init) = data_inits.get(&name) {
                if literal_fingerprint(init).as_deref() == Some(lit_fp.as_str()) {
                    violations.push(format!(
                        "{}::{}::{}",
                        rel,
                        fn_name,
                        ViolationKind::DataLiteralMirror.tag()
                    ));
                    continue;
                }
            }
        }
        if is_independent_call(&lhs) {
            continue;
        }
        if let ExprData::ExprCall { .. } = lhs.expr_data.as_ref() {
            let callee = expr_call_func_at(lhs.clone(), si.clone());
            let args = call_args(&lhs);
            if let Some(rec) = fns.get(&callee) {
                if rec.params.len() == args.len() {
                    let inlined =
                        substitute_params(&rec.body, &rec.params, &args, si);
                    if nodes_alpha_equal(&inlined, &rhs, si) {
                        violations.push(format!(
                            "{}::{}::{}",
                            rel,
                            fn_name,
                            ViolationKind::BodyAlphaEquivalent.tag()
                        ));
                    }
                }
            }
        }
        if let ExprData::ExprCall { .. } = rhs.expr_data.as_ref() {
            let callee = expr_call_func_at(rhs.clone(), si.clone());
            let args = call_args(&rhs);
            if let Some(rec) = fns.get(&callee) {
                if rec.params.len() == args.len() {
                    let inlined =
                        substitute_params(&rec.body, &rec.params, &args, si);
                    if nodes_alpha_equal(&inlined, &lhs, si) {
                        violations.push(format!(
                            "{}::{}::{}",
                            rel,
                            fn_name,
                            ViolationKind::BodyAlphaEquivalent.tag()
                        ));
                    }
                }
            }
        }
    }
    violations
}

fn dual_representation_sites(files: &[(String, String)]) -> Vec<String> {
    let index = build_corpus_index(files);
    let mut out = BTreeSet::new();
    for (rel, content) in files {
        if !is_test_dag(rel) {
            continue;
        }
        let path = resolve_rel_path(rel);
        let Some(parsed) = parse_dag_file(&path) else {
            continue;
        };
        let si = parsed.source_indices.clone();
        let witness_names = witness_fn_names(content);
        let data_inits = scoped_data_inits(rel, content, &index);
        let fns = scoped_fns(rel, content, &index);
        for item in parsed.items.iter() {
            if !is_fn_item(item) {
                continue;
            }
            let name = item_fn_name(item, &si);
            if !witness_names.contains(&name) {
                continue;
            }
            let Some(body) = item.body.clone() else {
                continue;
            };
            for site in classify_eq_violations(rel, &name, &body, &si, &data_inits, &fns) {
                out.insert(site);
            }
        }
    }
    out.into_iter().collect()
}

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
    use std::fs;

    fn write_fixture(dir: &Path, name: &str, content: &str) -> String {
        let path = dir.join(name);
        fs::write(&path, content).expect("write fixture");
        path.to_string_lossy().replace('\\', "/")
    }

    fn sites_in_fixtures(dir: &Path, specs: &[(&str, &str)]) -> Vec<String> {
        let mut files = Vec::new();
        for (name, content) in specs {
            let rel = write_fixture(dir, name, content);
            files.push((rel, content.to_string()));
        }
        dual_representation_sites(&files)
    }

    #[test]
    fn red_control_same_symbol_equality_is_flagged() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sites = sites_in_fixtures(
            dir.path(),
            &[(
                "t_test.dag",
                "module t\nfn witness_taut() -> Bool {\n  let m = one()\n  return m == m\n}\n",
            )],
        );
        assert!(
            sites.iter().any(|s| s.contains("same_symbol")),
            "expected same_symbol violation; got {sites:?}"
        );
    }

    #[test]
    fn green_control_distinct_symbol_equality_is_not_flagged() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sites = sites_in_fixtures(
            dir.path(),
            &[(
                "t_test.dag",
                "module t\ntest fn ok() -> Bool {\n  return a == b\n}\n",
            )],
        );
        assert!(
            sites.is_empty(),
            "distinct identifiers must not be flagged; got {sites:?}"
        );
    }

    #[test]
    fn green_control_independent_oracle_literal_is_not_flagged() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sites = sites_in_fixtures(
            dir.path(),
            &[(
                "t_test.dag",
                "module t\nfn add(a: Int, b: Int) -> Int { a + b }\ntest fn oracle() -> Bool {\n  return add(a: 2, b: 2) == 4\n}\n",
            )],
        );
        assert!(
            sites.is_empty(),
            "independent oracle literal must stay green; got {sites:?}"
        );
    }

    #[test]
    fn red_control_literal_mirror_is_flagged() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sites = sites_in_fixtures(
            dir.path(),
            &[(
                "t_test.dag",
                "module t\ndata ttl: Int = 3600\nfn witness_ttl() -> Bool {\n  return ttl == 3600\n}\n",
            )],
        );
        assert!(
            sites.iter().any(|s| s.contains("literal_mirror")),
            "expected literal_mirror violation; got {sites:?}"
        );
    }

    #[test]
    fn red_control_body_mirror_is_flagged() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sites = sites_in_fixtures(
            dir.path(),
            &[(
                "t_test.dag",
                "module t\nfn wrapped(x: Int) -> Int { inner(y: x) }\nfn inner(y: Int) -> Int { y }\nfn witness_wrap() -> Bool {\n  return wrapped(x: 1) == inner(y: 1)\n}\n",
            )],
        );
        assert!(
            sites.iter().any(|s| s.contains("body_mirror")),
            "expected body_mirror violation; got {sites:?}"
        );
    }
}
