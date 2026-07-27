//! Test-module umbrella dissolution + orphan-helper hygiene (proud-wren-892).
//!
//! U1 — umbrella roster via the real front-end (`parse_dag_file`).
//! U2 — plain fns in `*_test.dag` unreachable from test-fn / data-init roots refuse.
//! U3 — empty `function` on an explicit entry means file-grain: enumerate test decls
//!      through the same `scan_test_decl_names` path discovery uses.
//!
//! SCAFFOLD (§7 HAND-RUST — `test_module_hygiene_orphan_gate`):
//! Seed-retained host for U1–U3. Naming-walk live via `check_orphan_helpers_or_err`
//! (proud-wren-892 sweep B). Not a census shrink: this file is already on
//! `HAND_MAINTAINED_STAGE0_FILES` until the hygiene fold is expressed in `.dag`
//! and the module deletes.
//! DELETE WHEN dissolved: this module, the `cli_run` naming-walk call site, and the
//! unit RED plants below — host then reads a modeled enroll-or-refuse lens.
//! Lane: ROADMAP §1 "**drain the HAND_MAINTAINED queue**"
//! (`gunbc.roadmap_authority` ticket `5-dissolve-patches`; plan anchor
//! `dag/gunbc/v1_deletion_plan.dag` `^hand_queue_drain`).
//! Discriminating receipt (HAND-RUST GATE — unit RED plants, not a marker-count rg):
//!   `zero_enrolled_plain_helpers_refuse_demotion`
//!   `allowlisted_fixture_library_path_exempts_zero_enrolled_plains`
//!   `directory_prefix_alone_does_not_exempt_zero_enrolled_plains`
//!   `orphan_collect_refuses_unparsable_test_dag`
//!   `test_data_only_module_still_orphans_unreachable_plains`
//!   `plain_data_alone_does_not_enroll_demoted_holds`
//!   `unrelated_test_plus_dead_plain_data_does_not_enroll`
//!   `plain_data_reached_from_test_data_still_covers_helpers`
//! Authority row: `dag/gunbc/test_module_hygiene_scaffold.dag`.

/// INTERIM hand-Rust scaffold marker (`test_module_hygiene_orphan_gate` / §7).
/// Discriminating receipt: unit RED plants named in the module-level SCAFFOLD header
/// (and `gunbc.test_module_hygiene_scaffold`); not a marker-count rg.
pub const TEST_MODULE_HYGIENE_ORPHAN_GATE_SCAFFOLD_MARKER: &str = "test_module_hygiene_orphan_gate";

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

fn collect_dag_files(root: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries = std::fs::read_dir(root).map_err(|e| {
        format!(
            "orphan/umbrella hygiene: cannot read_dir {}: {e}",
            root.display()
        )
    })?;
    for ent in entries {
        let ent = ent.map_err(|e| {
            format!(
                "orphan/umbrella hygiene: read_dir entry under {}: {e}",
                root.display()
            )
        })?;
        let path = ent.path();
        if path.is_dir() {
            collect_dag_files(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("dag") {
            out.push(path);
        }
    }
    Ok(())
}

fn is_test_dag_path(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .map(|n| n.ends_with("_test.dag"))
        .unwrap_or(false)
}

/// Cross-module plain exports (repo-relative path → declaration names).
/// Local orphan walk cannot see importers; these names seed reachability as if
/// called (review 43604 — declaration-grain, never a whole-file bypass / DESIGN §5).
/// A plain in an allowlisted file that is NOT listed (and not reached from tests/
/// data/exports) still orphans. Dissolve-on: rename each file off `*_test.dag`,
/// or import-graph reachability supersedes the roster.
const CROSS_MODULE_EXPORTED_PLAINS: &[(&str, &[&str])] = &[
    // long/ wrappers execute these holds (review 43557).
    (
        "src/v2/lens/no_dual_representation_test.dag",
        &[
            "no_dual_representation_test_clean_holds",
            "no_dual_representation_test_coverage_honesty_holds",
        ],
    ),
    (
        "src/v2/lens/affected_set/edit_locus_resolver_test.dag",
        &[
            "edit_locus_source_provenance_affected_set_wire_holds",
            "edit_locus_source_provenance_producer_rejects_malformed_source_holds",
            "edit_locus_source_provenance_producer_holds",
            "edit_locus_source_provenance_parse_root_span_holds",
        ],
    ),
];

fn cross_module_exported_plains_for(path: &str) -> &'static [&'static str] {
    let p = path.replace('\\', "/");
    for (auth, exports) in CROSS_MODULE_EXPORTED_PLAINS {
        if p == *auth || p.ends_with(&format!("/{auth}")) {
            return exports;
        }
    }
    &[]
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
    /// data/const: name -> (is_test_data, body).
    /// Only `test data` are enrollment roots; plain `data` bodies are walked only
    /// when that data name is reachable from an executable root (reviews 43610 / 43617).
    data: BTreeMap<String, (bool, Option<Rc<Node>>)>,
    /// Executable enrollment: `test fn` / `test data` only (not plain `data`).
    has_enrollment_surface: bool,
    si: Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
}

fn analyze_test_module(path: &Path, content: &str) -> Option<ModuleFns> {
    let parsed = parse_dag_file(path)?;
    let test_names: HashSet<String> = scan_test_decl_names(content).into_iter().collect();
    let mut fns = BTreeMap::new();
    let mut data = BTreeMap::new();
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
                let is_test_data = test_names.contains(&name);
                data.insert(name, (is_test_data, item.body.clone()));
            }
            _ => {}
        }
    }
    let has_enrollment_surface = !test_names.is_empty();
    Some(ModuleFns {
        fns,
        data,
        has_enrollment_surface,
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
    let plain: HashSet<String> = module
        .fns
        .iter()
        .filter(|(_, (is_test, _, _))| !*is_test)
        .map(|(n, _)| n.clone())
        .collect();
    if plain.is_empty() {
        return Vec::new();
    }

    let cross_exports: HashSet<String> = cross_module_exported_plains_for(entry)
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    // Zero executable enrollment + plain helpers is the demotion failure mode
    // (all witnesses accidentally plain) — refuse (DESIGN §5; reviews 43330 / 43367 / 43610).
    // Enrollment = `test fn` / `test data` only. Plain `data` is fixture/claim shape,
    // not execution (specification-without-execution). Cross-module export names alone
    // are NOT enrollment: they only seed reachability below (review 43604).
    if !module.has_enrollment_surface && cross_exports.is_empty() {
        let mut orphans: Vec<OrphanHelper> = plain
            .into_iter()
            .map(|name| OrphanHelper {
                entry: entry.to_string(),
                name,
            })
            .collect();
        orphans.sort_by(|a, b| a.name.cmp(&b.name));
        return orphans;
    }

    let mut reachable: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut data_seen: HashSet<String> = HashSet::new();
    let mut data_queue: VecDeque<String> = VecDeque::new();

    // Plain `data` is never a root — only entered when its name is reached from an
    // executable path (test fn / test data / already-reached plain) — review 43617.
    let enqueue_refs = |body: &Rc<Node>,
                        si: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
                        plain: &HashSet<String>,
                        data_keys: &HashSet<String>,
                        queue: &mut VecDeque<String>,
                        data_queue: &mut VecDeque<String>,
                        data_seen: &mut HashSet<String>| {
        let mut refs = HashSet::new();
        walk_refs(body, si, &mut refs);
        for c in refs {
            if plain.contains(&c) {
                queue.push_back(c.clone());
            }
            if data_keys.contains(&c) && data_seen.insert(c.clone()) {
                data_queue.push_back(c);
            }
        }
    };
    let data_keys: HashSet<String> = module.data.keys().cloned().collect();

    // Roots: test fn bodies + test data bodies + export roster seeds.
    for (name, (is_test, _, body)) in &module.fns {
        if *is_test {
            reachable.insert(name.clone());
            if let Some(body) = body {
                enqueue_refs(
                    body,
                    &module.si,
                    &plain,
                    &data_keys,
                    &mut queue,
                    &mut data_queue,
                    &mut data_seen,
                );
            }
        }
    }
    for (name, (is_test_data, body)) in &module.data {
        if *is_test_data {
            data_seen.insert(name.clone());
            if let Some(body) = body {
                enqueue_refs(
                    body,
                    &module.si,
                    &plain,
                    &data_keys,
                    &mut queue,
                    &mut data_queue,
                    &mut data_seen,
                );
            }
        }
    }
    for name in &cross_exports {
        if plain.contains(name) {
            queue.push_back(name.clone());
        }
    }

    while !queue.is_empty() || !data_queue.is_empty() {
        while let Some(n) = queue.pop_front() {
            if !reachable.insert(n.clone()) {
                continue;
            }
            if let Some((_, _, Some(body))) = module.fns.get(&n) {
                let body = body.clone();
                enqueue_refs(
                    &body,
                    &module.si,
                    &plain,
                    &data_keys,
                    &mut queue,
                    &mut data_queue,
                    &mut data_seen,
                );
            }
        }
        while let Some(n) = data_queue.pop_front() {
            if let Some((_, Some(body))) = module.data.get(&n) {
                let body = body.clone();
                enqueue_refs(
                    &body,
                    &module.si,
                    &plain,
                    &data_keys,
                    &mut queue,
                    &mut data_queue,
                    &mut data_seen,
                );
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
        collect_dag_files(&ws.join(root), &mut files)?;
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
    if !parse_failures.is_empty() {
        return Err(format!(
            "umbrella roster: {} read/parse failure(s) — refuse (DESIGN §5), never fabricate a partial roster:\n  {}",
            parse_failures.len(),
            parse_failures.join("\n  ")
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
        collect_dag_files(&abs, &mut files)?;
    }
    files.sort();
    let mut out = Vec::new();
    let mut failures = Vec::new();
    for path in files {
        let rel = repo_relative(&path, &ws);
        if !is_test_dag_path(&rel) {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                failures.push(format!("read {rel}: {e}"));
                continue;
            }
        };
        let Some(module) = analyze_test_module(&path, &content) else {
            failures.push(format!("parse failed: {rel}"));
            continue;
        };
        out.extend(orphans_in_module(&rel, &module));
    }
    if !failures.is_empty() {
        // review 43367: an enforcement gate must refuse on unknown input, never
        // silently skip unreadable/unparsable `*_test.dag` and return Ok.
        return Err(format!(
            "test-module orphan-helper hygiene: {} read/parse failure(s) — refuse (DESIGN §5), \
             never fabricate a clean result:\n  {}",
            failures.len(),
            failures.join("\n  ")
        ));
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
         are unreachable from every `test fn` / `test data` (fixture data only when those exist) — silent \
         de-enrollment. Promote to `test fn`/`test data` or delete:",
        orphans.len()
    ));
    // Cap the printed list so the diagnostic stays usable; full count is in the header.
    for o in orphans.iter().take(200) {
        lines.push(format!("  {}::{}", o.entry, o.name));
    }
    if orphans.len() > 200 {
        lines.push(format!("  … and {} more", orphans.len() - 200));
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
    fn zero_enrolled_plain_helpers_refuse_demotion() {
        // review 43330: absence of `test fn` must not silently accept demoted witnesses.
        let dir = tmp_dir();
        let file = dir.join("demoted_witness_test.dag");
        std::fs::write(
            &file,
            r#"module demoted_witness

fn formerly_test_holds() -> Bool {
  true
}
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        let err = check_orphan_helpers_in_entries(&[entry])
            .expect_err("zero enrolled decls + plain helper must refuse");
        assert!(
            err.contains("formerly_test_holds"),
            "must name the demoted plain fn: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn directory_prefix_alone_does_not_exempt_zero_enrolled_plains() {
        // review 43367 / 43604: /extdeps/languages/ is not a bypass.
        let dir = tmp_dir().join("extdeps").join("languages");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("not_on_allowlist_test.dag");
        std::fs::write(
            &file,
            r#"module not_on_allowlist

fn export_for_importers() -> Bool {
  true
}
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        let err = check_orphan_helpers_in_entries(&[entry])
            .expect_err("non-allowlisted path under extdeps/languages must refuse");
        assert!(
            err.contains("export_for_importers"),
            "must name the demoted plain fn: {err}"
        );
        let _ = std::fs::remove_dir_all(dir.parent().unwrap().parent().unwrap());
    }

    #[test]
    fn cross_module_export_roster_seeds_named_plain_not_whole_file() {
        // review 43604: declaration-grain export seed, never whole-file clean.
        let dir = tmp_dir();
        let file = dir
            .join("src")
            .join("v2")
            .join("lens")
            .join("no_dual_representation_test.dag");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(
            &file,
            r#"module no_dual_representation_test

fn no_dual_representation_test_clean_holds() -> Bool {
  true
}

fn accidental_demotion_holds() -> Bool {
  false
}
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        let err = check_orphan_helpers_in_entries(&[entry])
            .expect_err("non-exported plain beside an export roster must refuse");
        assert!(
            err.contains("accidental_demotion_holds"),
            "must name the non-exported orphan: {err}"
        );
        assert!(
            !err.contains("no_dual_representation_test_clean_holds"),
            "exported plain must not orphan: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cross_module_export_roster_alone_does_not_green_empty_file() {
        // Export roster without the named decls present still refuses unknowns.
        let dir = tmp_dir();
        let file = dir
            .join("src")
            .join("v2")
            .join("lens")
            .join("no_dual_representation_test.dag");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(
            &file,
            r#"module no_dual_representation_test

fn unrelated_dark_holds() -> Bool {
  true
}
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        let err = check_orphan_helpers_in_entries(&[entry])
            .expect_err("path on export roster must not whole-file green");
        assert!(
            err.contains("unrelated_dark_holds"),
            "must name the orphan: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn orphan_collect_refuses_unparsable_test_dag() {
        // review 43367: parse failure must not fabricate Ok([]).
        let dir = tmp_dir();
        let file = dir.join("garbage_unparsable_test.dag");
        std::fs::write(&file, "this is not valid dag {{{{").unwrap();
        let root = dir.to_string_lossy().into_owned();
        let err = collect_orphan_helpers(&[root]).expect_err("unparsable *_test.dag must refuse");
        assert!(
            err.contains("parse failed") || err.contains("garbage_unparsable_test.dag"),
            "must locate the parse failure: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_data_only_module_still_orphans_unreachable_plains() {
        let dir = tmp_dir();
        let file = dir.join("test_data_only_test.dag");
        std::fs::write(
            &file,
            r#"module test_data_only

fn used_by_test_data() -> Bool { true }

fn dark_plain() -> Bool { false }

test data enrolled_row: Bool = used_by_test_data()
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        let err = check_orphan_helpers_in_entries(&[entry])
            .expect_err("test data enrollment must still orphan unreachable plains");
        assert!(
            err.contains("dark_plain"),
            "must name unreachable plain: {err}"
        );
        assert!(
            !err.contains("used_by_test_data"),
            "reachable-from-test-data must not red: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn plain_data_alone_does_not_enroll_demoted_holds() {
        // review 43610: `data fixture = formerly_test_holds()` must not certify a
        // zero-test module clean (specification-without-execution / DESIGN §5).
        let dir = tmp_dir();
        let file = dir.join("plain_data_demotion_test.dag");
        std::fs::write(
            &file,
            r#"module plain_data_demotion

fn formerly_test_holds() -> Bool { true }

data fixture: Bool = formerly_test_holds()
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        let err = check_orphan_helpers_in_entries(&[entry])
            .expect_err("plain data alone must not enroll demoted holds");
        assert!(
            err.contains("formerly_test_holds"),
            "must name the demoted plain fn: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unrelated_test_plus_dead_plain_data_does_not_enroll() {
        // review 43617: an unrelated `test fn` must not make dead plain `data`
        // into a reachability root for demoted holds.
        let dir = tmp_dir();
        let file = dir.join("dead_plain_data_test.dag");
        std::fs::write(
            &file,
            r#"module dead_plain_data

test fn unrelated_holds() -> Bool { true }

fn demoted_holds() -> Bool { true }

data fixture: Bool = demoted_holds()
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        let err = check_orphan_helpers_in_entries(&[entry])
            .expect_err("dead plain data beside an unrelated test must not enroll");
        assert!(
            err.contains("demoted_holds"),
            "must name the demoted plain fn: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn plain_data_reached_from_test_data_still_covers_helpers() {
        // Chain: test data → plain data → helper must still green the helper.
        let dir = tmp_dir();
        let file = dir.join("plain_data_chain_test.dag");
        std::fs::write(
            &file,
            r#"module plain_data_chain

fn helper_holds() -> Bool { true }

fn dark_plain() -> Bool { false }

data claim_row: Bool = helper_holds()

test data enrolled_run: Bool = claim_row
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        let err = check_orphan_helpers_in_entries(&[entry])
            .expect_err("unreachable plain must still red");
        assert!(
            err.contains("dark_plain"),
            "must name unreachable plain: {err}"
        );
        assert!(
            !err.contains("helper_holds"),
            "helper reached via test-data→plain-data chain must not red: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn orphan_gate_scaffold_marker_is_declared() {
        assert_eq!(
            TEST_MODULE_HYGIENE_ORPHAN_GATE_SCAFFOLD_MARKER,
            "test_module_hygiene_orphan_gate"
        );
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
