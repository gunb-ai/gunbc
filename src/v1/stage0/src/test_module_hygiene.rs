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
//!   `plain_data_claim_rows_count_as_enrollment_surface`
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

/// Explicit cross-module fixture-library entries (repo-relative, `/`-normalized).
/// These keep the historical `*_test.dag` suffix while exporting plain helpers for
/// cross-module import — local reachability cannot see importers. Bounded roster,
/// never a directory-wide bypass (review 43367 / DESIGN §5). Dissolve-on: rename
/// each off `_test.dag`.
const CROSS_MODULE_FIXTURE_LIBRARY_ENTRIES: &[&str] = &[
    "src/v2/extdeps/languages/rust_test.dag",
    "dag/examples/interp_test/interp_test.dag",
    // long/ wrappers execute these holds; module-local orphan walk cannot see importers
    // (review 43557 — honest fixture-library disposition, not if-false pins).
    // Dissolve-on: rename each off `*_test.dag`.
    "src/v2/lens/no_dual_representation_test.dag",
    "src/v2/lens/affected_set/edit_locus_resolver_test.dag",
];

fn is_cross_module_fixture_library_path(path: &str) -> bool {
    let p = path.replace('\\', "/");
    CROSS_MODULE_FIXTURE_LIBRARY_ENTRIES
        .iter()
        .any(|auth| p == *auth || p.ends_with(&format!("/{auth}")))
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
    /// Enrollment surface present: `test fn` / `test data` *or* plain `data` claim rows.
    /// Plain `data` is a legacy/manual enrollment grain (e.g. CompilesClaim receipts) —
    /// treating "no test fn" alone as total demotion falsely orphans helpers reached
    /// only from those rows (regen red after orphan-gate wire).
    has_enrollment_surface: bool,
    si: Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
}

fn analyze_test_module(path: &Path, content: &str) -> Option<ModuleFns> {
    let parsed = parse_dag_file(path)?;
    let test_names: HashSet<String> = scan_test_decl_names(content).into_iter().collect();
    let mut fns = BTreeMap::new();
    let mut data_bodies = Vec::new();
    let mut data_item_count = 0usize;
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
                data_item_count += 1;
                if let Some(body) = item.body.clone() {
                    data_bodies.push(body);
                }
            }
            _ => {}
        }
    }
    let has_enrollment_surface = !test_names.is_empty() || data_item_count > 0;
    Some(ModuleFns {
        fns,
        data_bodies,
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

    // Cross-module fixture libraries export plain helpers for importers local
    // reachability cannot see — exempt the whole orphan check, not only the
    // zero-enrollment demotion arm (review 43367 allowlist intent).
    if is_cross_module_fixture_library_path(entry) {
        return Vec::new();
    }

    // Zero enrollment surface + plain helpers is the demotion failure mode
    // (all witnesses accidentally plain) — refuse (DESIGN §5; reviews 43330 / 43367).
    // Enrollment surface = `test fn` / `test data` OR plain `data` claim rows.
    if !module.has_enrollment_surface {
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

    // Roots: test fn bodies + data/const initializers (fixture / claim-row surface).
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
         are unreachable from every test fn and data/const initializer in their module — silent \
         de-enrollment. Promote to `test fn` or delete:",
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
        // review 43367: /extdeps/languages/ is not a bypass — only the explicit roster.
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
    fn allowlisted_fixture_library_path_exempts_zero_enrolled_plains() {
        let dir = tmp_dir();
        // Path must end with an authority entry from CROSS_MODULE_FIXTURE_LIBRARY_ENTRIES.
        let file = dir
            .join("src")
            .join("v2")
            .join("extdeps")
            .join("languages")
            .join("rust_test.dag");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(
            &file,
            r#"module rust_test_fixture_probe

fn export_for_importers() -> Bool {
  true
}
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        check_orphan_helpers_in_entries(&[entry])
            .expect("allowlisted rust_test.dag path is a bounded fixture library");
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
    fn plain_data_claim_rows_count_as_enrollment_surface() {
        // Manual/CompilesClaim modules enroll via plain `data`, not `test fn`.
        let dir = tmp_dir();
        let file = dir.join("plain_data_enroll_test.dag");
        std::fs::write(
            &file,
            r#"module plain_data_enroll

fn helper_holds() -> Bool { true }

fn dark_plain() -> Bool { false }

data receipt: Bool = helper_holds()
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        let err = check_orphan_helpers_in_entries(&[entry])
            .expect_err("plain data enrollment must still orphan unreachable plains");
        assert!(
            err.contains("dark_plain"),
            "must name unreachable plain: {err}"
        );
        assert!(
            !err.contains("helper_holds"),
            "reachable-from-plain-data must not red: {err}"
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
