use crate::module_path_index::parsed_dag_file::parse_dag_file;
use crate::v1_compiler_infer_items::{item_kind, ItemKind};
use crate::v1_interpreter::{self, sorted_fields, ExecutionMode, InterpContext, Value};
use crate::v1_std_core::{authored_name_at, expr_call_func_at, expr_var_name_at, ExprData, Node};
use im::HashMap;
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc as Rc;

const TEST_MODULE_HYGIENE_ENTRY: &str = "dag/gunbc/test_module_hygiene.dag";

fn hygiene_entry_file() -> String {
    let path = Path::new(TEST_MODULE_HYGIENE_ENTRY);
    if path.is_file() {
        return path.to_string_lossy().into_owned();
    }
    let ws_path = super::workspace_root().join(path);
    if ws_path.is_file() {
        return ws_path.to_string_lossy().into_owned();
    }
    TEST_MODULE_HYGIENE_ENTRY.to_string()
}

#[derive(Debug, Clone)]
pub(crate) struct DeclSurface {
    pub name: String,
    pub refs: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct ModuleSurface {
    pub entry: String,
    pub test_fns: Vec<DeclSurface>,
    pub plain_fns: Vec<DeclSurface>,
    pub test_data: Vec<DeclSurface>,
    pub plain_data: Vec<DeclSurface>,
}

pub(crate) fn is_file_grain_function(function: &str) -> bool {
    function.is_empty()
}

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

fn refs_from_body(
    body: &Rc<Node>,
    si: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Vec<String> {
    let mut set = HashSet::new();
    walk_refs(body, si, &mut set);
    let mut out: Vec<String> = set.into_iter().collect();
    out.sort();
    out
}

fn test_decl_names_from_content(content: &str) -> Result<Vec<String>, String> {
    let roots = super::default_source_roots();
    let ctx = resolve_hygiene_ctx(&roots)?;
    let args = [(Some("content".to_string()), Value::Str(content.to_string()))];
    let result =
        v1_interpreter::run_in_context_with_args(&ctx, "enumerate_entry_test_names", &args, false)
            .map_err(|e| format!("enumerate_entry_test_names: {e}"))?;
    let Value::List(items) = result else {
        return Err(format!(
            "enumerate_entry_test_names returned `{}`, expected List",
            ctx.format_value(&result)
        ));
    };
    items
        .iter()
        .map(|item| match item {
            Value::Str(s) => Ok(s.clone()),
            other => Err(format!(
                "enumerate_entry_test_names element `{}` is not String",
                ctx.format_value(other)
            )),
        })
        .collect()
}

fn analyze_to_surface(path: &Path, content: &str, entry: &str) -> Option<ModuleSurface> {
    let parsed = parse_dag_file(path)?;
    let test_names: HashSet<String> = test_decl_names_from_content(content)
        .ok()?
        .into_iter()
        .collect();
    let si = parsed.source_indices.clone();
    let mut test_fns = Vec::new();
    let mut plain_fns = Vec::new();
    let mut test_data = Vec::new();
    let mut plain_data = Vec::new();
    for item in parsed.items.iter() {
        let name = authored_name_at(si.clone(), item.clone());
        if name.is_empty() {
            continue;
        }
        let kind = item_kind(item.clone());
        match kind {
            ItemKind::FnItem | ItemKind::FuncItem => {
                let is_test = test_names.contains(&name);
                let refs = item
                    .body
                    .as_ref()
                    .map(|b| refs_from_body(b, &si))
                    .unwrap_or_default();
                let decl = DeclSurface { name, refs };
                if is_test {
                    test_fns.push(decl);
                } else {
                    plain_fns.push(decl);
                }
            }
            ItemKind::DataItem => {
                let is_test_data = test_names.contains(&name);
                let refs = item
                    .body
                    .as_ref()
                    .map(|b| refs_from_body(b, &si))
                    .unwrap_or_default();
                let decl = DeclSurface { name, refs };
                if is_test_data {
                    test_data.push(decl);
                } else {
                    plain_data.push(decl);
                }
            }
            _ => {}
        }
    }
    Some(ModuleSurface {
        entry: entry.to_string(),
        test_fns,
        plain_fns,
        test_data,
        plain_data,
    })
}

fn is_test_dag_path(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .map(|n| n.ends_with("_test.dag"))
        .unwrap_or(false)
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

fn repo_relative(path: &Path, ws: &Path) -> String {
    path.strip_prefix(ws)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}

fn decl_surface_to_value(decl: &DeclSurface, ctx: &InterpContext) -> Value {
    let refs: Vec<Value> = decl.refs.iter().map(|r| Value::Str(r.clone())).collect();
    Value::Record {
        type_name: ctx.sym("DeclSurface"),
        fields: Rc::new(sorted_fields(vec![
            (ctx.sym("name"), Value::Str(decl.name.clone())),
            (ctx.sym("refs"), v1_interpreter::list_value(refs)),
        ])),
    }
}

fn module_surface_to_value(surface: &ModuleSurface, ctx: &InterpContext) -> Value {
    let test_fns: Vec<Value> = surface
        .test_fns
        .iter()
        .map(|d| decl_surface_to_value(d, ctx))
        .collect();
    let plain_fns: Vec<Value> = surface
        .plain_fns
        .iter()
        .map(|d| decl_surface_to_value(d, ctx))
        .collect();
    let test_data: Vec<Value> = surface
        .test_data
        .iter()
        .map(|d| decl_surface_to_value(d, ctx))
        .collect();
    let plain_data: Vec<Value> = surface
        .plain_data
        .iter()
        .map(|d| decl_surface_to_value(d, ctx))
        .collect();
    Value::Record {
        type_name: ctx.sym("ModuleSurface"),
        fields: Rc::new(sorted_fields(vec![
            (ctx.sym("entry"), Value::Str(surface.entry.clone())),
            (ctx.sym("test_fns"), v1_interpreter::list_value(test_fns)),
            (ctx.sym("plain_fns"), v1_interpreter::list_value(plain_fns)),
            (ctx.sym("test_data"), v1_interpreter::list_value(test_data)),
            (
                ctx.sym("plain_data"),
                v1_interpreter::list_value(plain_data),
            ),
        ])),
    }
}

fn resolve_hygiene_ctx(source_roots: &[String]) -> Result<InterpContext, String> {
    let entry = hygiene_entry_file();
    let (graph, indices) = super::resolve_entry_graph_shared(source_roots, &entry)
        .map_err(|e| format!("test_module_hygiene resolve: {e}"))?;
    Ok(super::make_eval_context(
        &graph,
        indices,
        ExecutionMode::Hermetic,
    ))
}

fn collect_module_surfaces(roots: &[String]) -> Result<Vec<ModuleSurface>, String> {
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
    let mut surfaces = Vec::new();
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
        let Some(surface) = analyze_to_surface(&path, &content, &rel) else {
            failures.push(format!("parse failed: {rel}"));
            continue;
        };
        surfaces.push(surface);
    }
    if !failures.is_empty() {
        return Err(format!(
            "test-module orphan-helper hygiene: {} read/parse failure(s) — refuse (DESIGN §5), \
             never fabricate a clean result:\n  {}",
            failures.len(),
            failures.join("\n  ")
        ));
    }
    Ok(surfaces)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureReceiptCompanionLookup {
    NotDeclared,
    Declared(String),
    AuthorityRefused { cause: String },
}

pub(crate) fn failure_receipt_companion_from_authority(
    function: &str,
) -> FailureReceiptCompanionLookup {
    let roots = super::default_source_roots();
    let ctx = match resolve_hygiene_ctx(&roots) {
        Ok(ctx) => ctx,
        Err(detail) => {
            return FailureReceiptCompanionLookup::AuthorityRefused { cause: detail };
        }
    };
    let args = [(
        Some("function".to_string()),
        v1_interpreter::Value::Str(function.to_string()),
    )];
    let result = match v1_interpreter::run_in_context_with_args(
        &ctx,
        "failure_receipt_companion",
        &args,
        false,
    ) {
        Ok(value) => value,
        Err(detail) => {
            return FailureReceiptCompanionLookup::AuthorityRefused {
                cause: detail.to_string(),
            };
        }
    };
    match &result {
        v1_interpreter::Value::Variant {
            type_name,
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*type_name, "Optional") && ctx.sym_eq(*variant_name, "Present") => {
            match ctx.field(fields, "value") {
                Some(v1_interpreter::Value::Str(companion)) => {
                    FailureReceiptCompanionLookup::Declared(companion.clone())
                }
                other => FailureReceiptCompanionLookup::AuthorityRefused {
                    cause: format!(
                        "Present value must be String, got {}",
                        other
                            .map(|v| ctx.format_value(v))
                            .unwrap_or_else(|| "<missing>".to_string())
                    ),
                },
            }
        }
        v1_interpreter::Value::Variant {
            type_name,
            variant_name,
            ..
        } if ctx.sym_eq(*type_name, "Optional") && ctx.sym_eq(*variant_name, "Absent") => {
            FailureReceiptCompanionLookup::NotDeclared
        }
        other => FailureReceiptCompanionLookup::AuthorityRefused {
            cause: format!(
                "returned {}, expected Optional<String>",
                ctx.format_value(other)
            ),
        },
    }
}

pub(crate) fn check_orphan_helpers_or_err(source_roots: &[String]) -> Result<(), String> {
    let surfaces = collect_module_surfaces(source_roots)?;
    let ctx = resolve_hygiene_ctx(source_roots)?;
    let module_values: Vec<Value> = surfaces
        .iter()
        .map(|s| module_surface_to_value(s, &ctx))
        .collect();
    let args = [(
        Some("modules".to_string()),
        v1_interpreter::list_value(module_values),
    )];
    let result = v1_interpreter::run_in_context_with_args(
        &ctx,
        "check_orphan_surfaces_or_refuse",
        &args,
        false,
    )
    .map_err(|e| format!("check_orphan_surfaces_or_refuse: {e}"))?;
    match &result {
        Value::Variant {
            type_name,
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*type_name, "Optional") && ctx.sym_eq(*variant_name, "Present") => {
            match ctx.field(&fields, "value") {
                Some(Value::Str(reason)) => Err(reason.clone()),
                _ => {
                    Err("check_orphan_surfaces_or_refuse Present missing reason string".to_string())
                }
            }
        }
        Value::Variant {
            type_name,
            variant_name,
            ..
        } if ctx.sym_eq(*type_name, "Optional") && ctx.sym_eq(*variant_name, "Absent") => Ok(()),
        other => Err(format!(
            "check_orphan_surfaces_or_refuse returned `{}`, expected Optional<String>",
            ctx.format_value(other)
        )),
    }
}

pub(crate) fn enumerate_entry_test_fns(entry_path: &str) -> Result<Vec<String>, String> {
    let content = std::fs::read_to_string(entry_path)
        .map_err(|e| format!("file-grain enumerate: read {entry_path}: {e}"))?;
    let names = test_decl_names_from_content(&content)?;
    if names.is_empty() {
        return Err(format!(
            "file-grain enumerate: entry {entry_path} has no test fn / test data declarations"
        ));
    }
    Ok(names)
}

pub(crate) fn expand_explicit_entries(
    explicit_entries: &[(String, String)],
) -> Result<Vec<(String, String)>, String> {
    let roots = super::default_source_roots();
    let ctx = resolve_hygiene_ctx(&roots)?;
    let mut inputs = Vec::new();
    for (entry, function) in explicit_entries {
        let content = if is_file_grain_function(function) {
            std::fs::read_to_string(entry)
                .map_err(|e| format!("file-grain expand read {entry}: {e}"))?
        } else {
            String::new()
        };
        inputs.push(Value::Record {
            type_name: ctx.sym("ExplicitExpandInput"),
            fields: Rc::new(sorted_fields(vec![
                (ctx.sym("entry"), Value::Str(entry.clone())),
                (ctx.sym("function"), Value::Str(function.clone())),
                (ctx.sym("content"), Value::Str(content)),
            ])),
        });
    }
    let args = [(
        Some("inputs".to_string()),
        v1_interpreter::list_value(inputs),
    )];
    let result = v1_interpreter::run_in_context_with_args(
        &ctx,
        "expand_explicit_pairs_or_refuse",
        &args,
        false,
    )
    .map_err(|e| format!("expand_explicit_pairs_or_refuse: {e}"))?;
    match &result {
        Value::Variant {
            type_name,
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*type_name, "ExpandExplicitPairsOutcome")
            && ctx.sym_eq(*variant_name, "Refused") =>
        {
            match ctx.field(fields, "reason") {
                Some(Value::Str(reason)) => Err(reason.clone()),
                _ => Err("ExpandExplicitPairsOutcome.Refused missing reason".to_string()),
            }
        }
        Value::Variant {
            type_name,
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*type_name, "ExpandExplicitPairsOutcome")
            && ctx.sym_eq(*variant_name, "Expanded") =>
        {
            let Value::List(items) = ctx
                .field(fields, "pairs")
                .ok_or_else(|| "ExpandExplicitPairsOutcome.Expanded missing pairs".to_string())?
            else {
                return Err("ExpandExplicitPairsOutcome.Expanded pairs is not List".to_string());
            };
            let mut out = Vec::new();
            for item in items.iter() {
                let Value::Record { fields, .. } = item else {
                    return Err(
                        "expand_explicit_pairs_or_refuse element is not ExplicitEntryPair"
                            .to_string(),
                    );
                };
                let entry = match ctx.field(fields, "entry") {
                    Some(Value::Str(s)) => s.clone(),
                    _ => return Err("ExplicitEntryPair missing entry".to_string()),
                };
                let function = match ctx.field(fields, "function") {
                    Some(Value::Str(s)) => s.clone(),
                    _ => return Err("ExplicitEntryPair missing function".to_string()),
                };
                out.push((entry, function));
            }
            Ok(out)
        }
        other => Err(format!(
            "expand_explicit_pairs_or_refuse returned `{}`, expected ExpandExplicitPairsOutcome",
            ctx.format_value(other)
        )),
    }
}

pub(crate) fn module_surface_for_test(
    path: &Path,
    content: &str,
    entry: &str,
) -> Option<ModuleSurface> {
    analyze_to_surface(path, content, entry)
}

pub(crate) fn invoke_orphan_plain_names(
    ctx: &InterpContext,
    surface: &ModuleSurface,
) -> Result<Vec<String>, String> {
    let args = [(
        Some("module".to_string()),
        module_surface_to_value(surface, ctx),
    )];
    let result =
        v1_interpreter::run_in_context_with_args(ctx, "orphan_plain_names_or_refuse", &args, false)
            .map_err(|e| format!("orphan_plain_names_or_refuse: {e}"))?;
    match &result {
        Value::Variant {
            type_name,
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*type_name, "OrphanPlainNamesOutcome")
            && ctx.sym_eq(*variant_name, "ReachBudgetRefused") =>
        {
            match ctx.field(fields, "reason") {
                Some(Value::Str(reason)) => Err(reason.clone()),
                _ => Err("OrphanPlainNamesOutcome.ReachBudgetRefused missing reason".to_string()),
            }
        }
        Value::Variant {
            type_name,
            variant_name,
            fields,
            ..
        } if ctx.sym_eq(*type_name, "OrphanPlainNamesOutcome")
            && ctx.sym_eq(*variant_name, "Orphans") =>
        {
            let Value::List(items) = ctx
                .field(fields, "names")
                .ok_or_else(|| "OrphanPlainNamesOutcome.Orphans missing names".to_string())?
            else {
                return Err("OrphanPlainNamesOutcome.Orphans names is not List".to_string());
            };
            items
                .iter()
                .map(|item| match item {
                    Value::Str(s) => Ok(s.clone()),
                    other => Err(format!(
                        "orphan_plain_names_or_refuse element `{}` is not String",
                        ctx.format_value(other)
                    )),
                })
                .collect()
        }
        other => Err(format!(
            "orphan_plain_names_or_refuse returned `{}`, expected OrphanPlainNamesOutcome",
            ctx.format_value(other)
        )),
    }
}

#[cfg(test)]
mod test_module_hygiene_bridge_equivalence_tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    fn tmp_dir() -> PathBuf {
        let n = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("gunbc_tmh_bridge_{n}_{id}"))
    }

    fn check_entries_or_err(entry_paths: &[String]) -> Result<(), String> {
        let mut surfaces = Vec::new();
        for entry in entry_paths {
            if !is_test_dag_path(entry) {
                continue;
            }
            let path = Path::new(entry);
            let content = std::fs::read_to_string(path)
                .map_err(|e| format!("orphan check read {entry}: {e}"))?;
            let Some(surface) = analyze_to_surface(path, &content, entry) else {
                return Err(format!("orphan check: parse failed for {entry}"));
            };
            surfaces.push(surface);
        }
        let roots = super::super::default_source_roots();
        let ctx = resolve_hygiene_ctx(&roots)?;
        let module_values: Vec<Value> = surfaces
            .iter()
            .map(|s| module_surface_to_value(s, &ctx))
            .collect();
        let args = [(
            Some("modules".to_string()),
            v1_interpreter::list_value(module_values),
        )];
        let result = v1_interpreter::run_in_context_with_args(
            &ctx,
            "check_orphan_surfaces_or_refuse",
            &args,
            false,
        )
        .map_err(|e| format!("check_orphan_surfaces_or_refuse: {e}"))?;
        match &result {
            Value::Variant {
                type_name,
                variant_name,
                fields,
                ..
            } if ctx.sym_eq(*type_name, "Optional") && ctx.sym_eq(*variant_name, "Present") => {
                match ctx.field(&fields, "value") {
                    Some(Value::Str(reason)) => Err(reason.clone()),
                    _ => Err("Present missing reason".to_string()),
                }
            }
            Value::Variant {
                type_name,
                variant_name,
                ..
            } if ctx.sym_eq(*type_name, "Optional") && ctx.sym_eq(*variant_name, "Absent") => {
                Ok(())
            }
            other => Err(ctx.format_value(other)),
        }
    }

    #[test]
    fn orphan_plant_reds_then_removing_greens() {
        let dir = tmp_dir();
        std::fs::create_dir_all(&dir).unwrap();
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
        let err = check_entries_or_err(&[entry.clone()]).expect_err("planted dark helper must red");
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
        check_entries_or_err(&[entry]).expect("removing plant must green");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn zero_enrolled_plain_helpers_refuse_demotion() {
        let dir = tmp_dir();
        std::fs::create_dir_all(&dir).unwrap();
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
        let err = check_entries_or_err(&[entry]).expect_err("zero enrolled must refuse");
        assert!(
            err.contains("formerly_test_holds"),
            "must name plain: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn orphan_collect_refuses_unparsable_test_dag() {
        let dir = tmp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("garbage_unparsable_test.dag");
        std::fs::write(&file, "this is not valid dag {{{{").unwrap();
        let root = dir.to_string_lossy().into_owned();
        let err =
            check_orphan_helpers_or_err(&[root]).expect_err("unparsable *_test.dag must refuse");
        assert!(
            err.contains("parse failed") || err.contains("garbage_unparsable_test.dag"),
            "must locate the parse failure: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_grain_expand_enumerates_test_decls() {
        let dir = tmp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("expand_grain_test.dag");
        std::fs::write(
            &file,
            r#"module expand_grain

test fn alpha_holds() -> Bool { true }
test data beta_fixture: Bool = true
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        let expanded = expand_explicit_entries(&[(entry, String::new())]).expect("expand");
        let names: BTreeSet<_> = expanded.into_iter().map(|(_, f)| f).collect();
        assert_eq!(
            names,
            BTreeSet::from(["alpha_holds".to_string(), "beta_fixture".to_string()])
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_grain_expand_refuses_empty_test_decls() {
        let dir = tmp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("empty_grain_test.dag");
        std::fs::write(
            &file,
            r#"module empty_grain

fn plain_only() -> Bool {
  true
}
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        let err = expand_explicit_entries(&[(entry.clone(), String::new())])
            .expect_err("file-grain with no test decls must refuse");
        assert!(
            err.contains("has no test fn / test data declarations"),
            "must refuse empty enumerate: {err}"
        );
        assert!(err.contains(&entry), "must name entry: {err}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn failure_receipt_companion_reachable_from_holds_witness() {
        let dir = tmp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("failure_receipt_companion_test.dag");
        std::fs::write(
            &file,
            r#"module failure_receipt_companion_fixture

test fn witness_holds() -> Bool {
  true
}

func witness_failure_receipt() -> String {
  "detail"
}
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        check_entries_or_err(&[entry]).expect("companion must be reachable from holds witness");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn failure_receipt_without_matching_witness_still_refuses() {
        let dir = tmp_dir();
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("failure_receipt_orphan_test.dag");
        std::fs::write(
            &file,
            r#"module failure_receipt_orphan_fixture

test fn witness_holds() -> Bool {
  true
}

func orphan_failure_receipt() -> String {
  "detail"
}
"#,
        )
        .unwrap();
        let entry = file.to_string_lossy().into_owned();
        let err = check_entries_or_err(&[entry])
            .expect_err("companion without matching witness must refuse");
        assert!(
            err.contains("orphan_failure_receipt"),
            "must name the orphan companion: {err}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
