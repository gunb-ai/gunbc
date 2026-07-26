//! Test-module umbrella dissolution + orphan-helper hygiene (proud-wren-892).
//!
//! U1 — umbrella roster via the real front-end (`parse_dag_file`).
//! U2 — plain fns in `*_test.dag` unreachable from test-fn / data-init roots refuse.
//! U3 — empty `function` on an explicit entry means file-grain: enumerate test decls
//!      through the same `scan_test_decl_names` path discovery uses.

use im::HashMap;
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::module_path_index::parsed_dag_file::parse_dag_file;
use crate::std_syntax::BinOp;
use crate::v1_compiler_infer_items::{item_kind, ItemKind};
use crate::v1_std_core::{
    authored_name_at, binop_left, binop_right, block_stmts, expr_call_func_at, expr_var_name_at,
    ExprData, Node,
};

/// Empty `ScheduleWitnessEntry.function` / empty `CommitWitnessClaim.check_fns` means
/// file-grain: the executor enumerates the entry's test fns (U3).
pub fn is_file_grain_function(function: &str) -> bool {
    function.is_empty()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UmbrellaRecord {
    pub entry: String,
    pub umbrella_fn: String,
    pub conjuncts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrphanHelper {
    pub entry: String,
    pub name: String,
}

fn collect_dag_files(root: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(root) {
        Ok(e) => e,
        Err(_) => return,
    };
    for ent in entries.flatten() {
        let path = ent.path();
        if path.is_dir() {
            collect_dag_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("dag") {
            out.push(path);
        }
    }
}

fn is_test_dag_path(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .map(|n| n.ends_with("_test.dag"))
        .unwrap_or(false)
}

fn scan_test_decl_names(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        let rest = trimmed
            .strip_prefix("test fn ")
            .or_else(|| trimmed.strip_prefix("test data "));
        if let Some(rest) = rest {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.push(name);
            }
        }
    }
    out
}

/// Public: same enumerator discovery uses for test-marked decls in an entry file.
pub fn enumerate_entry_test_fns(entry_path: &str) -> Result<Vec<String>, String> {
    let content = std::fs::read_to_string(entry_path)
        .map_err(|e| format!("file-grain enumerate: read {entry_path}: {e}"))?;
    let names = scan_test_decl_names(&content);
    if names.is_empty() {
        return Err(format!(
            "file-grain enumerate: entry {entry_path} has no test fn / test data declarations"
        ));
    }
    Ok(names)
}

/// Expand explicit (entry, function) pairs: empty function → all test decls in the entry.
pub fn expand_explicit_entries(
    explicit_entries: &[(String, String)],
) -> Result<Vec<(String, String)>, String> {
    let mut out = Vec::new();
    let mut seen: HashSet<(String, String)> = HashSet::new();
    for (entry, function) in explicit_entries {
        if is_file_grain_function(function) {
            for name in enumerate_entry_test_fns(entry)? {
                let key = (entry.clone(), name.clone());
                if seen.insert(key.clone()) {
                    out.push(key);
                }
            }
        } else {
            let key = (entry.clone(), function.clone());
            if seen.insert(key.clone()) {
                out.push(key);
            }
        }
    }
    Ok(out)
}

/// Collect local names reached from an expression: call callees AND bare vars
/// (fn-as-value bindings like `call_primitive: bind_eval_call_primitive`).
fn walk_refs(
    node: &Rc<Node>,
    si: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
    out: &mut HashSet<String>,
) {
    match node.expr_data.as_ref() {
        ExprData::ExprCall { .. } => {
            let name = expr_call_func_at(node.clone(), si.clone());
            if !name.is_empty() {
                out.insert(name);
            }
        }
        ExprData::ExprVar { .. } => {
            let name = expr_var_name_at(node.clone(), si.clone());
            if !name.is_empty() {
                out.insert(name);
            }
        }
        _ => {}
    }
    for child in node.children.iter() {
        walk_refs(child, si, out);
    }
}

fn peel_block(body: &Rc<Node>) -> Rc<Node> {
    match body.expr_data.as_ref() {
        ExprData::ExprBlock => {
            let stmts = block_stmts(body.clone());
            if stmts.len() == 1 {
                peel_block(&stmts[0])
            } else if let Some(last) = stmts.last() {
                peel_block(last)
            } else {
                body.clone()
            }
        }
        _ => body.clone(),
    }
}

/// If `expr` is a pure conjunction of nullary calls to names in `nullary_locals`,
/// return those call names in left-to-right order. Otherwise None.
fn umbrella_conjuncts(
    expr: &Rc<Node>,
    si: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
    nullary_locals: &HashSet<String>,
) -> Option<Vec<String>> {
    match expr.expr_data.as_ref() {
        ExprData::ExprBinOp { op: BinOp::And, .. } => {
            let left = umbrella_conjuncts(&binop_left(expr.clone()), si, nullary_locals)?;
            let right = umbrella_conjuncts(&binop_right(expr.clone()), si, nullary_locals)?;
            let mut out = left;
            out.extend(right);
            Some(out)
        }
        ExprData::ExprCall { .. } => {
            let name = expr_call_func_at(expr.clone(), si.clone());
            let arg_count = expr
                .children
                .iter()
                .filter(|a| !a.children.is_empty())
                .count();
            if arg_count == 0 && nullary_locals.contains(&name) {
                Some(vec![name])
            } else {
                None
            }
        }
        ExprData::ExprBlock => umbrella_conjuncts(&peel_block(expr), si, nullary_locals),
        _ => None,
    }
}

struct ModuleFns {
    /// name -> (is_test, param_count, body)
    fns: BTreeMap<String, (bool, usize, Option<Rc<Node>>)>,
    /// data/const initializer bodies (reachability roots for fixture constructors)
    data_bodies: Vec<Rc<Node>>,
    si: Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
}

fn analyze_test_module(path: &Path, content: &str) -> Option<ModuleFns> {
    let parsed = parse_dag_file(path)?;
    let test_names: HashSet<String> = scan_test_decl_names(content).into_iter().collect();
    let mut fns = BTreeMap::new();
    let mut data_bodies = Vec::new();
    for item in parsed.items.iter() {
        let name = authored_name_at(parsed.source_indices.clone(), item.clone());
        if name.is_empty() {
            continue;
        }
        let kind = item_kind(item.clone());
        match kind {
            ItemKind::FnItem | ItemKind::FuncItem => {
                let param_count = item.params.len();
                let is_test = test_names.contains(&name);
                fns.insert(name, (is_test, param_count, item.body.clone()));
            }
            ItemKind::DataItem => {
                if let Some(body) = item.body.clone() {
                    data_bodies.push(body);
                }
            }
            _ => {}
        }
    }
    Some(ModuleFns {
        fns,
        data_bodies,
        si: parsed.source_indices,
    })
}

fn umbrellas_in_module(entry: &str, module: &ModuleFns) -> Vec<UmbrellaRecord> {
    // Nullary locals include both plain helpers and test fns — a residual umbrella
    // that &&-chains already-promoted test fns must still be detected and deleted.
    let nullary_locals: HashSet<String> = module
        .fns
        .iter()
        .filter(|(_, (_is_test, params, _))| *params == 0)
        .map(|(n, _)| n.clone())
        .collect();
    let mut out = Vec::new();
    for (name, (is_test, _params, body)) in &module.fns {
        if !is_test {
            continue;
        }
        let Some(body) = body else { continue };
        let peeled = peel_block(body);
        let Some(conjuncts) = umbrella_conjuncts(&peeled, &module.si, &nullary_locals) else {
            continue;
        };
        // A conjunction requires ≥2 conjuncts; census used ≥3 as the upper-bound filter.
        if conjuncts.len() < 2 {
            continue;
        }
        out.push(UmbrellaRecord {
            entry: entry.to_string(),
            umbrella_fn: name.clone(),
            conjuncts,
        });
    }
    out
}

fn orphans_in_module(entry: &str, module: &ModuleFns) -> Vec<OrphanHelper> {
    // U2 enroll-or-refuse is for test modules: plain helpers beside live `test fn`s.
    // A `*_test.dag` with zero `test fn`s is a fixture/example library (e.g.
    // `extdeps/languages/rust_test.dag` — cross-module consumers, module-local
    // reachability cannot see them). Skip rather than false-red every export.
    // Dissolve-on: rename fixture libraries off the `_test.dag` suffix, or lift
    // reachability to the import graph.
    let has_test_fn = module.fns.values().any(|(is_test, _, _)| *is_test);
    if !has_test_fn {
        return Vec::new();
    }
    let plain: HashSet<String> = module
        .fns
        .iter()
        .filter(|(_, (is_test, _, _))| !*is_test)
        .map(|(n, _)| n.clone())
        .collect();
    if plain.is_empty() {
        return Vec::new();
    }
    let mut reachable: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();

    // Roots: test fn bodies + data/const initializers (fixture surface).
    for (name, (is_test, _, body)) in &module.fns {
        if *is_test {
            reachable.insert(name.clone());
            if let Some(body) = body {
                let mut refs = HashSet::new();
                walk_refs(body, &module.si, &mut refs);
                for c in refs {
                    if plain.contains(&c) {
                        queue.push_back(c);
                    }
                }
            }
        }
    }
    // data + test data initializers (fixture / claim-row surface)
    for body in &module.data_bodies {
        let mut refs = HashSet::new();
        walk_refs(body, &module.si, &mut refs);
        for c in refs {
            if plain.contains(&c) {
                queue.push_back(c);
            }
        }
    }

    while let Some(n) = queue.pop_front() {
        if !reachable.insert(n.clone()) {
            continue;
        }
        if let Some((_, _, Some(body))) = module.fns.get(&n) {
            let mut refs = HashSet::new();
            walk_refs(body, &module.si, &mut refs);
            for c in refs {
                if plain.contains(&c) && !reachable.contains(&c) {
                    queue.push_back(c);
                }
            }
        }
    }

    let mut orphans = Vec::new();
    for name in &plain {
        if !reachable.contains(name) {
            orphans.push(OrphanHelper {
                entry: entry.to_string(),
                name: name.clone(),
            });
        }
    }
    orphans.sort_by(|a, b| a.name.cmp(&b.name));
    orphans
}

fn repo_relative(path: &Path, ws: &Path) -> String {
    path.strip_prefix(ws)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

/// U1 — parse every `*_test.dag` under `roots` and emit the umbrella roster.
pub fn collect_umbrella_roster(roots: &[String]) -> Result<Vec<UmbrellaRecord>, String> {
    let ws = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
    let mut files = Vec::new();
    for root in roots {
        collect_dag_files(&ws.join(root), &mut files);
    }
    files.sort();
    let mut out = Vec::new();
    let mut parse_failures = Vec::new();
    for path in files {
        let rel = repo_relative(&path, &ws);
        if !is_test_dag_path(&rel) {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                parse_failures.push(format!("read {rel}: {e}"));
                continue;
            }
        };
        let Some(module) = analyze_test_module(&path, &content) else {
            parse_failures.push(format!("parse failed: {rel}"));
            continue;
        };
        out.extend(umbrellas_in_module(&rel, &module));
    }
    if !parse_failures.is_empty() && out.is_empty() {
        return Err(format!(
            "umbrella roster: no modules parsed ({})",
            parse_failures.len()
        ));
    }
    out.sort_by(|a, b| (&a.entry, &a.umbrella_fn).cmp(&(&b.entry, &b.umbrella_fn)));
    Ok(out)
}

/// U2 — orphan plain fns across `*_test.dag` under roots.
pub fn collect_orphan_helpers(roots: &[String]) -> Result<Vec<OrphanHelper>, String> {
    // Relative roots resolve against cwd (floor discovery: workspace root). Absolute
    // roots are used as-is — callers must not mutate process cwd to pass a workspace
    // (cargo tests are multi-threaded; set_current_dir races).
    let ws = std::env::current_dir().map_err(|e| format!("cwd: {e}"))?;
    let mut files = Vec::new();
    for root in roots {
        let root_path = Path::new(root);
        let abs = if root_path.is_absolute() {
            root_path.to_path_buf()
        } else {
            ws.join(root_path)
        };
        collect_dag_files(&abs, &mut files);
    }
    files.sort();
    let mut out = Vec::new();
    for path in files {
        let rel = repo_relative(&path, &ws);
        if !is_test_dag_path(&rel) {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let Some(module) = analyze_test_module(&path, &content) else {
            continue;
        };
        out.extend(orphans_in_module(&rel, &module));
    }
    out.sort_by(|a, b| (&a.entry, &a.name).cmp(&(&b.entry, &b.name)));
    Ok(out)
}

/// Fail-closed orphan check for the naming walk. `roots` are source roots.
pub fn check_orphan_helpers_or_err(roots: &[String]) -> Result<(), String> {
    let orphans = collect_orphan_helpers(roots)?;
    if orphans.is_empty() {
        return Ok(());
    }
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!(
        "test-module orphan-helper hygiene (DESIGN §5/§6 enroll-or-refuse): {} plain fn(s) in `*_test.dag` \
         are unreachable from every test fn and data/const initializer in their module — silent \
         de-enrollment. Promote to `test fn` or delete:",
        orphans.len()
    ));
    // Cap the printed list so the diagnostic stays usable; full count is in the header.
    for o in orphans.iter().take(40) {
        lines.push(format!("  {}::{}", o.entry, o.name));
    }
    if orphans.len() > 40 {
        lines.push(format!("  … and {} more", orphans.len() - 40));
    }
    Err(lines.join("\n"))
}

/// Orphan check scoped to an explicit set of absolute/relative entry paths (RED fixtures).
pub fn check_orphan_helpers_in_entries(entry_paths: &[String]) -> Result<(), String> {
    let mut orphans = Vec::new();
    for entry in entry_paths {
        let path = Path::new(entry);
        if !is_test_dag_path(entry) {
            continue;
        }
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("orphan check read {entry}: {e}"))?;
        let Some(module) = analyze_test_module(path, &content) else {
            return Err(format!("orphan check: parse failed for {entry}"));
        };
        orphans.extend(orphans_in_module(entry, &module));
    }
    if orphans.is_empty() {
        return Ok(());
    }
    let mut lines = vec![format!(
        "test-module orphan-helper hygiene: {} unreachable plain fn(s):",
        orphans.len()
    )];
    for o in &orphans {
        lines.push(format!("  {}::{}", o.entry, o.name));
    }
    Err(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn tmp_dir() -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("gunbc_umbrella_hygiene_{n}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn orphan_plant_reds_then_removing_greens() {
        let dir = tmp_dir();
        let file = dir.join("orphan_plant_test.dag");
        std::fs::write(
            &file,
            r#"module orphan_plant_fixture

test fn live_holds() -> Bool {
  true
}

fn dark_unreachable_check() -> Bool {
  false
}
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        let err = check_orphan_helpers_in_entries(&[entry.clone()])
            .expect_err("planted dark helper must red");
        assert!(
            err.contains("dark_unreachable_check"),
            "must name the orphan fn: {err}"
        );
        std::fs::write(
            &file,
            r#"module orphan_plant_fixture

test fn live_holds() -> Bool {
  true
}
"#,
        )
        .unwrap();
        check_orphan_helpers_in_entries(&[entry]).expect("removing plant must green");
        let _ = std::fs::remove_dir_all(&dir);
    }


    #[test]
    fn file_grain_enumerate_yields_two_test_fns() {
        let dir = tmp_dir();
        let file = dir.join("two_fn_file_grain_test.dag");
        std::fs::write(
            &file,
            r#"module two_fn_file_grain

test fn alpha_holds() -> Bool { true }

test fn beta_holds() -> Bool { true }
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        let expanded = expand_explicit_entries(&[(entry.clone(), String::new())]).expect("expand");
        assert_eq!(expanded.len(), 2, "file-grain row must yield two verdicts");
        let names: BTreeSet<_> = expanded.into_iter().map(|(_, f)| f).collect();
        assert_eq!(
            names,
            ["alpha_holds".to_string(), "beta_holds".to_string()]
                .into_iter()
                .collect()
        );
        let missing = expand_explicit_entries(&[(entry, "does_not_exist_fn".into())]);
        // Non-empty function is not validated here — discovery refuses missing fns at eval.
        assert!(missing.is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn umbrella_detection_on_conjunction() {
        let dir = tmp_dir();
        let file = dir.join("umbrella_shape_test.dag");
        std::fs::write(
            &file,
            r#"module umbrella_shape

fn a_holds() -> Bool { true }
fn b_holds() -> Bool { true }
fn c_holds() -> Bool { true }

test fn umbrella_holds() -> Bool {
  a_holds() && b_holds() && c_holds()
}
"#,
        )
        .unwrap();
        let content = std::fs::read_to_string(&file).unwrap();
        let module = analyze_test_module(&file, &content).expect("parse");
        let umbrellas = umbrellas_in_module("umbrella_shape_test.dag", &module);
        assert_eq!(umbrellas.len(), 1);
        assert_eq!(umbrellas[0].umbrella_fn, "umbrella_holds");
        assert_eq!(
            umbrellas[0].conjuncts,
            vec!["a_holds", "b_holds", "c_holds"]
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
