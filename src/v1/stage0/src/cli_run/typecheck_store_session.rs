//! Distinct seed operations that look up and commit TypecheckModuleRequest through
//! `gunbc.typecheck_module_store` (C2). Not arguments on `typecheck_module`.

use std::cell::{Cell, RefCell};

use crate::v1_compiler_infer::{TypecheckModuleOwn, TypecheckModuleResult};
use crate::v1_interpreter::{self, list_value, str_value, ExecutionMode, Value};
use im::Vector;

use super::{make_eval_context, resolve_entry_with_index, MultiEntryIndex};

const STORE_ENTRY: &str = "dag/gunbc/typecheck_module_store.dag";
const STORE_MODULE: &str = "gunbc.typecheck_module_store";

thread_local! {
    static PREPARING: Cell<bool> = const { Cell::new(false) };
    static SESSION: RefCell<Option<(v1_interpreter::InterpContext, Value)>> = const { RefCell::new(None) };
    static OBSERVED_LARGEST_ENTRY: Cell<u64> = const { Cell::new(0) };
    /// Test-only: route the session through the witness door over a scratch root instead of the
    /// granted durable volume. Same .dag bodies, different opening (`*_witness_hex`).
    #[cfg(test)]
    static WITNESS_ROOT: RefCell<Option<String>> = const { RefCell::new(None) };
}

#[cfg(test)]
pub(crate) fn set_witness_root(root: Option<String>) {
    WITNESS_ROOT.with(|w| *w.borrow_mut() = root);
}

/// The opening door: the production grant, or (tests) the witness scratch root. Both are .dag
/// functions returning a `LocalStoreOpening`; the session calls one exactly once.
fn open_door() -> (&'static str, Vec<(Option<String>, Value)>) {
    #[cfg(test)]
    if let Some(root) = WITNESS_ROOT.with(|w| w.borrow().clone()) {
        return (
            "open_typecheck_store_witness_door",
            vec![(Some("root_path".to_string()), str_value(root))],
        );
    }
    ("open_typecheck_store_door", Vec::new())
}

/// What the store handed back for one module: the module's OWN tail (the entry never holds the
/// import-derived closure; a hit re-runs the head from the imports, `typecheck_module_restore`).
pub(crate) enum DurableTypecheckGet {
    Hit(TypecheckModuleOwn),
    Miss,
    /// The .dag door refused to open the store (grant or catalog refusal); counted, never a Miss.
    Unavailable,
}

/// Every outcome of the durable door is one of these and each is counted (DESIGN 5: a degradation is
/// typed, located and countable, never a silent Ok/Miss). Reported by `store_receipt_line`.
#[derive(Clone, Copy, Default)]
struct OutcomeCounts {
    hit: u64,
    miss: u64,
    unavailable: u64,
    commit_committed: u64,
    commit_missed_open: u64,
    /// Commits whose stored size could not be read back: excluded from the largest-entry reading.
    commit_size_unread: u64,
    /// Calls made while the invoking workflow configured no store: no open, no write attempted.
    not_configured_lookup: u64,
    not_configured_commit: u64,
    reentry_lookup: u64,
    reentry_commit: u64,
    unreachable_lookup: u64,
    unreachable_commit: u64,
    /// Times the store was opened (grant check, marker converge, catalog): one per session.
    store_openings: u64,
    /// Openings the .dag door refused; every lookup in that session is then a counted Unavailable.
    open_refused: u64,
}

thread_local! {
    static OUTCOMES: Cell<OutcomeCounts> = const { Cell::new(OutcomeCounts {
        hit: 0, miss: 0, unavailable: 0, commit_committed: 0, commit_missed_open: 0, commit_size_unread: 0, not_configured_lookup: 0, not_configured_commit: 0,
        reentry_lookup: 0, reentry_commit: 0, unreachable_lookup: 0, unreachable_commit: 0,
        store_openings: 0, open_refused: 0,
    }) };
}

fn count(f: impl FnOnce(&mut OutcomeCounts)) {
    OUTCOMES.with(|c| {
        let mut v = c.get();
        f(&mut v);
        c.set(v);
    });
}

/// The run-receipt projection of the counted outcomes and the largest committed entry.
/// `Some` when the store was exercised or any outcome other than hit/miss/committed occurred.
pub(crate) fn store_receipt_line() -> Option<String> {
    let c = OUTCOMES.with(|c| c.get());
    let degraded = c.unavailable
        + c.open_refused
        + c.commit_missed_open
        + c.commit_size_unread
        + c.not_configured_lookup
        + c.not_configured_commit
        + c.reentry_lookup
        + c.reentry_commit
        + c.unreachable_lookup
        + c.unreachable_commit;
    let touched = c.hit + c.miss + c.commit_committed + degraded;
    if touched == 0 {
        return None;
    }
    Some(format!(
        "[typecheck-store] hit={} miss={} committed={} unavailable={} commit_missed_open={} commit_size_unread={} not_configured_lookup={} not_configured_commit={} reentry_lookup={} reentry_commit={} unreachable_lookup={} unreachable_commit={} store_openings={} open_refused={} largest_entry_bytes={}",
        c.hit,
        c.miss,
        c.commit_committed,
        c.unavailable,
        c.commit_missed_open,
        c.commit_size_unread,
        c.not_configured_lookup,
        c.not_configured_commit,
        c.reentry_lookup,
        c.reentry_commit,
        c.unreachable_lookup,
        c.unreachable_commit,
        c.store_openings,
        c.open_refused,
        observed_largest_entry_bytes()
    ))
}

/// Serialize the module's OWN tail: signatures and typed items in the interface payload, own
/// diagnostics in the diagnostics payload. No `TypeEnv`, no parents.
pub(crate) fn own_payload_texts(own: &TypecheckModuleOwn) -> Result<(String, String), String> {
    let interface_text = serde_json::to_string(&(&own.items, &own.func_local, &own.item_registry))
        .map_err(|e| format!("typecheck-store serialize own interface: {e}"))?;
    let diagnostics_text = serde_json::to_string(&own.diagnostics)
        .map_err(|e| format!("typecheck-store serialize own diagnostics: {e}"))?;
    Ok((interface_text, diagnostics_text))
}

fn own_from_payload_texts(
    interface_text: &str,
    diagnostics_text: &str,
) -> Result<TypecheckModuleOwn, String> {
    let (items, func_local, item_registry) = serde_json::from_str(interface_text)
        .map_err(|e| format!("typecheck-store hit interface is not a module tail: {e}"))?;
    let diagnostics = serde_json::from_str(diagnostics_text)
        .map_err(|e| format!("typecheck-store hit diagnostics are not a diagnostic list: {e}"))?;
    Ok(TypecheckModuleOwn {
        items,
        func_local,
        item_registry,
        diagnostics,
    })
}

/// The own tail of an already-typechecked module (what a commit stores).
pub(crate) fn own_of_result(result: &TypecheckModuleResult) -> TypecheckModuleOwn {
    TypecheckModuleOwn {
        items: result.typed.items.clone(),
        func_local: result.typed.func_env.local.clone(),
        item_registry: result.typed.item_registry.clone(),
        diagnostics: result.diagnostics.clone(),
    }
}

/// The invoking workflow's explicit choice to use the granted durable store is a declared .dag
/// fact (`gunbc.typecheck_store_selector`: the variable's name, its one admitted value, absence =
/// not configured); Rust only reads the environment variable that declaration names and asks the
/// declaration's `typecheck_store_selection` what it means. Decided once per thread.
const SELECTOR_ENTRY: &str = "dag/gunbc/typecheck_store_selector.dag";
const SELECTOR_MODULE: &str = "gunbc.typecheck_store_selector";

thread_local! {
    static SELECTED: Cell<Option<bool>> = const { Cell::new(None) };
}

/// `Ok(Some(true))` configured, `Ok(Some(false))` not configured, `Ok(None)` the declaring module
/// is not in the caller's pool (counted unreachable by the caller), `Err` a refused value.
fn ctx_optional_str(
    ctx: &v1_interpreter::InterpContext,
    v: Option<String>,
) -> Result<Value, String> {
    Ok(match v {
        Some(v) => Value::Variant {
            type_name: ctx.sym("Optional"),
            variant_name: ctx.sym("Present"),
            fields: std::rc::Rc::new(vec![(ctx.sym("value"), str_value(v))]),
        },
        None => Value::Variant {
            type_name: ctx.sym("Optional"),
            variant_name: ctx.sym("Absent"),
            fields: std::rc::Rc::new(vec![]),
        },
    })
}

fn store_selected(index: &MultiEntryIndex) -> Result<Option<bool>, String> {
    #[cfg(test)]
    if WITNESS_ROOT.with(|w| w.borrow().is_some()) {
        return Ok(Some(true));
    }
    if let Some(v) = SELECTED.with(|c| c.get()) {
        return Ok(Some(v));
    }
    if !index.holds_module(SELECTOR_MODULE) {
        return Ok(None);
    }
    let entry = super::workspace_root()
        .join(SELECTOR_ENTRY)
        .to_string_lossy()
        .into_owned();
    // Resolving the selector's own closure typechecks modules, which reach this store's lookup:
    // the re-entry guard turns that into a counted re-entry instead of a recursion.
    PREPARING.with(|c| c.set(true));
    let resolved = resolve_entry_with_index(index, &entry);
    PREPARING.with(|c| c.set(false));
    let (graph, indices) = resolved?;
    let ctx = make_eval_context(&graph, indices, ExecutionMode::Hermetic);
    let name_value = v1_interpreter::run_in_context_with_args(
        &ctx,
        "typecheck_store_selector_env_name_of",
        &[],
        false,
    )
    .map_err(|e| format!("typecheck_store_selector_env_name_of: {e}"))?;
    let name = match &name_value {
        Value::Str(s) => s.to_string(),
        other => return Err(format!("selector name not a String: {other:?}")),
    };
    let value = match std::env::var(&name) {
        Ok(v) => ctx_optional_str(&ctx, Some(v))?,
        Err(_) => ctx_optional_str(&ctx, None)?,
    };
    let decision = v1_interpreter::run_in_context_with_args(
        &ctx,
        "typecheck_store_selection",
        &[(Some("value".to_string()), value)],
        false,
    )
    .map_err(|e| format!("typecheck_store_selection: {e}"))?;
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = &decision
    else {
        return Err(format!("selection not a variant: {decision:?}"));
    };
    let configured = if ctx.sym_eq(*variant_name, "TypecheckStoreConfigured") {
        true
    } else if ctx.sym_eq(*variant_name, "TypecheckStoreNotConfigured") {
        false
    } else if ctx.sym_eq(*variant_name, "TypecheckStoreSelectorRefused") {
        let v = field_str(&ctx, fields, "value")?;
        return Err(format!("{name} has an unadmitted value ({v})"));
    } else {
        return Err(format!("unknown selection {}", ctx.resolve(*variant_name)));
    };
    SELECTED.with(|c| c.set(Some(configured)));
    Ok(Some(configured))
}

/// `Ok(None)` when the store's own closure is not reachable from the compiler's roots (the entry
/// file is absent): the caller counts it as a typed Unavailable, never a Miss and never a panic.
fn with_session<T>(
    index: &MultiEntryIndex,
    f: impl FnOnce(&v1_interpreter::InterpContext, &Value) -> Result<T, String>,
) -> Result<Option<T>, String> {
    if PREPARING.with(|c| c.get()) {
        return Err("typecheck-store session prepare re-entered".to_string());
    }
    SESSION.with(|slot| {
        if slot.borrow().is_none() {
            // The store closure is resolved through the CALLER's own index (the compiler's
            // roots; one index per module-name set), never a second pool and never a cwd path.
            if !index.holds_module(STORE_MODULE) {
                return Ok(None);
            }
            // Anchored at the checkout root: the loader reads the entry path as given.
            let entry = super::workspace_root()
                .join(STORE_ENTRY)
                .to_string_lossy()
                .into_owned();
            PREPARING.with(|c| c.set(true));
            let prepared = (|| {
                let (graph, indices) = resolve_entry_with_index(index, &entry)?;
                Ok(make_eval_context(&graph, indices, ExecutionMode::Wet))
            })();
            PREPARING.with(|c| c.set(false));
            match prepared {
                Ok(ctx) => {
                    // The opening is one fact per session: opened here, passed to every call.
                    let (function, args) = open_door();
                    let opening =
                        v1_interpreter::run_in_context_with_args(&ctx, function, &args, false)
                            .map_err(|e| format!("{function}: {e}"))?;
                    count(|c| c.store_openings += 1);
                    if let Value::Variant { variant_name, .. } = &opening {
                        if ctx.sym_eq(*variant_name, "LocalStoreUnavailable") {
                            count(|c| c.open_refused += 1);
                        }
                    }
                    *slot.borrow_mut() = Some((ctx, opening));
                }
                Err(e) => return Err(e),
            }
        }
        let borrow = slot.borrow();
        let (ctx, opening) = borrow.as_ref().expect("session installed");
        f(ctx, opening).map(Some)
    })
}

fn field_str(
    ctx: &v1_interpreter::InterpContext,
    fields: &[(v1_interpreter::Symbol, Value)],
    name: &str,
) -> Result<String, String> {
    match ctx.field(fields, name) {
        Some(Value::Str(s)) => Ok(s.to_string()),
        other => Err(format!("{name} not a String: {other:?}")),
    }
}

/// Distinct from `index_get_typed`: durable TypecheckModuleRequest lookup.
pub(crate) fn durable_typecheck_lookup(
    index: &MultiEntryIndex,
    source_digest_hex: &str,
    import_interface_hexes: &[String],
    compiler_digest_hex: &str,
) -> Result<DurableTypecheckGet, String> {
    if PREPARING.with(|c| c.get()) {
        count(|c| c.reentry_lookup += 1);
        return Ok(DurableTypecheckGet::Miss);
    }
    match store_selected(index)? {
        Some(true) => {}
        Some(false) => {
            count(|c| c.not_configured_lookup += 1);
            return Ok(DurableTypecheckGet::Miss);
        }
        None => {
            count(|c| c.unreachable_lookup += 1);
            return Ok(DurableTypecheckGet::Unavailable);
        }
    }
    let imports = list_value(
        import_interface_hexes
            .iter()
            .cloned()
            .map(str_value)
            .collect::<Vector<_>>(),
    );
    let function = "seed_lookup_typecheck_opened_hex";
    let mut args: Vec<(Option<String>, Value)> = Vec::new();
    args.extend([
        (
            Some("source_digest_hex".to_string()),
            str_value(source_digest_hex),
        ),
        (Some("import_interface_hexes".to_string()), imports),
        (
            Some("compiler_digest_hex".to_string()),
            str_value(compiler_digest_hex),
        ),
    ]);
    let got = with_session(index, |ctx, opening| {
        let mut args = args;
        args.insert(0, (Some("opening".to_string()), opening.clone()));
        let result = v1_interpreter::run_in_context_with_args(ctx, function, &args, false)
            .map_err(|e| format!("seed_lookup_typecheck_opened_hex: {e}"))?;
        let Value::Variant {
            variant_name,
            fields,
            ..
        } = &result
        else {
            return Err(format!(
                "seed_lookup_typecheck_opened_hex not a variant: {result:?}"
            ));
        };
        if ctx.sym_eq(*variant_name, "SeedTypecheckMiss") {
            count(|c| c.miss += 1);
            return Ok(DurableTypecheckGet::Miss);
        }
        if ctx.sym_eq(*variant_name, "SeedTypecheckUnavailable") {
            count(|c| c.unavailable += 1);
            return Ok(DurableTypecheckGet::Unavailable);
        }
        if ctx.sym_eq(*variant_name, "SeedTypecheckDigestRefused") {
            let which = field_str(ctx, fields, "which")?;
            return Err(format!("typecheck-store request digest refused ({which})"));
        }
        if ctx.sym_eq(*variant_name, "SeedTypecheckIntegrityRefused") {
            let tag = field_str(ctx, fields, "tag")?;
            return Err(format!(
                "typecheck-store integrity refusal ({tag}): stale or tampered entry"
            ));
        }
        if ctx.sym_eq(*variant_name, "SeedTypecheckHit") {
            let interface_text = field_str(ctx, fields, "interface_text")?;
            let diagnostics_text = field_str(ctx, fields, "diagnostics_text")?;
            let own = own_from_payload_texts(&interface_text, &diagnostics_text)?;
            count(|c| c.hit += 1);
            return Ok(DurableTypecheckGet::Hit(own));
        }
        Err(format!(
            "seed_lookup_typecheck_opened_hex unknown variant {}",
            ctx.resolve(*variant_name)
        ))
    })?;
    Ok(match got {
        Some(g) => g,
        None => {
            count(|c| c.unreachable_lookup += 1);
            DurableTypecheckGet::Unavailable
        }
    })
}

/// Distinct from `index_insert_typed`: durable TypecheckModuleRequest commit.
pub(crate) fn durable_typecheck_commit(
    index: &MultiEntryIndex,
    source_digest_hex: &str,
    import_interface_hexes: &[String],
    compiler_digest_hex: &str,
    own: &TypecheckModuleOwn,
) -> Result<(), String> {
    if PREPARING.with(|c| c.get()) {
        count(|c| c.reentry_commit += 1);
        return Ok(());
    }
    match store_selected(index)? {
        Some(true) => {}
        Some(false) => {
            count(|c| c.not_configured_commit += 1);
            return Ok(());
        }
        None => {
            count(|c| c.unreachable_commit += 1);
            return Ok(());
        }
    }
    let (interface_text, diagnostics_text) = own_payload_texts(own)?;
    let imports = list_value(
        import_interface_hexes
            .iter()
            .cloned()
            .map(str_value)
            .collect::<Vector<_>>(),
    );
    let function = "seed_commit_typecheck_opened_hex";
    let mut args: Vec<(Option<String>, Value)> = Vec::new();
    args.extend([
        (
            Some("source_digest_hex".to_string()),
            str_value(source_digest_hex),
        ),
        (Some("import_interface_hexes".to_string()), imports),
        (
            Some("compiler_digest_hex".to_string()),
            str_value(compiler_digest_hex),
        ),
        (
            Some("interface_text".to_string()),
            str_value(interface_text),
        ),
        (
            Some("diagnostics_text".to_string()),
            str_value(diagnostics_text),
        ),
    ]);
    let got = with_session(index, |ctx, opening| {
        let mut args = args;
        args.insert(0, (Some("opening".to_string()), opening.clone()));
        let result = v1_interpreter::run_in_context_with_args(ctx, function, &args, false)
            .map_err(|e| format!("seed_commit_typecheck_opened_hex: {e}"))?;
        let Value::Variant {
            variant_name,
            fields,
            ..
        } = &result
        else {
            return Err(format!(
                "seed_commit_typecheck_opened_hex not a variant: {result:?}"
            ));
        };
        if ctx.sym_eq(*variant_name, "SeedTypecheckCommitted") {
            count(|c| c.commit_committed += 1);
            // The stored object's size as the store sizes it (local_store_object_bytes), read
            // back from the door: the grain the budget frontier names.
            let stored = match ctx.field(fields, "stored") {
                Some(Value::Record { fields: m, .. }) => match ctx.field(m, "count") {
                    Some(Value::Int(n)) if *n >= 0 => *n as u64,
                    other => return Err(format!("stored size not a count: {other:?}")),
                },
                other => return Err(format!("stored not a ByteSize: {other:?}")),
            };
            record_stored_entry_bytes(stored);
            return Ok(());
        }
        if ctx.sym_eq(*variant_name, "SeedTypecheckCommittedSizeUnread") {
            // Committed, size unmeasured: counted, and never a reading (Absent is not zero).
            count(|c| {
                c.commit_committed += 1;
                c.commit_size_unread += 1;
            });
            return Ok(());
        }
        if ctx.sym_eq(*variant_name, "SeedTypecheckCommitMissedOpen") {
            count(|c| c.commit_missed_open += 1);
            return Ok(());
        }
        if ctx.sym_eq(*variant_name, "SeedTypecheckCommitRefused") {
            return Err("typecheck-store commit refused".to_string());
        }
        if ctx.sym_eq(*variant_name, "SeedTypecheckCommitDigestRefused") {
            let which = field_str(ctx, fields, "which")?;
            return Err(format!("typecheck-store commit digest refused ({which})"));
        }
        Err(format!(
            "seed_commit_typecheck_opened_hex unknown variant {}",
            ctx.resolve(*variant_name)
        ))
    })?;
    if got.is_none() {
        count(|c| c.unreachable_commit += 1);
    }
    Ok(())
}

fn record_stored_entry_bytes(stored: u64) {
    OBSERVED_LARGEST_ENTRY.with(|c| {
        if stored > c.get() {
            c.set(stored);
        }
    });
}

pub(crate) fn observed_largest_entry_bytes() -> u64 {
    OBSERVED_LARGEST_ENTRY.with(|c| c.get())
}

#[cfg(test)]
pub(crate) fn outcome_snapshot() -> (u64, u64, u64) {
    let c = OUTCOMES.with(|c| c.get());
    (c.hit, c.miss, c.commit_committed)
}

#[cfg(test)]
pub(crate) fn outcome_reset() {
    OUTCOMES.with(|c| c.set(OutcomeCounts::default()));
}

/// Executing controls over REAL modules (DESIGN 3 pairing obligation): the production
/// `reconcile_with_typed_cache` commits each module's OWN tail through the .dag door, and a fresh
/// index (empty in-process cache) restores every module from the store via
/// `typecheck_module_restore` to a result byte-equal to the computed one.
#[cfg(test)]
mod real_module_round_trip {
    use super::*;
    use crate::cli_run::{build_multi_entry_index, resolve_entry_with_index, workspace_root};

    fn entry_and_roots() -> (String, Vec<String>) {
        // The resolver reads repo-relative paths from the process cwd.
        std::env::set_current_dir(workspace_root()).expect("enter workspace root");
        (
            "dag/std/optional.dag".to_string(),
            vec!["dag".to_string(), "src/v2".to_string()],
        )
    }

    struct CountingWriter(u64);
    impl std::io::Write for CountingWriter {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0 += buf.len() as u64;
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// The typed results by content key. Compared with the derived structural `PartialEq`
    /// (map-order-insensitive): serialized text is not canonical because HashMap iteration order
    /// differs between two builds of the same map.
    fn typed_results(
        index: &crate::cli_run::MultiEntryIndex,
    ) -> Vec<(String, std::rc::Rc<TypecheckModuleResult>)> {
        let mut v = index.typed_module_cache_for_tests();
        v.sort_by(|a, b| a.0.cmp(&b.0));
        v
    }

    #[ignore = "live-corpus: resolves and typechecks the live dag+src/v2 closure (minutes); the receipts lane runs these with --ignored, the required unit run does not"]
    #[test]
    fn restore_equals_compute_and_entry_excludes_parent_envs() {
        let (entry, roots) = entry_and_roots();
        round_trip("live", entry, roots);
    }

    /// The same integration over a SMALL REAL fixture pool (two modules, one importing the other),
    /// in the ordinary unit run: cold compile through `reconcile_with_typed_cache` -> durable
    /// miss -> real own tail committed; then a FRESH index -> durable hit -> `typecheck_module_restore`
    /// -> restored own tail equals the computed one -> the stored entry excludes parent envs.
    /// Removing the lookup/restore integration from `reconcile_with_typed_cache` turns this red
    /// (no hit), which no store-boundary claim does.
    #[test]
    fn the_reconcile_integration_misses_commits_then_hits_and_restores_a_fixture_pool() {
        std::env::set_current_dir(workspace_root()).expect("enter workspace root");
        let dir = format!("target/gunbc_fx_rt_{}", std::process::id());
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(format!("{dir}/fx")).unwrap();
        std::fs::write(
            format!("{dir}/fx/base.dag"),
            "module fx.base\n\ndata base_value: Int = 7\n\nfn base_fn(x: Int) -> Int { x + base_value }\n",
        )
        .unwrap();
        std::fs::write(
            format!("{dir}/fx/top.dag"),
            "module fx.top\n\nimport fx.base { base_fn }\n\nfn top_fn() -> Int { base_fn(x: 1) }\n",
        )
        .unwrap();
        round_trip(
            "fixture",
            format!("{dir}/fx/top.dag"),
            vec![dir.clone(), "dag".to_string(), "src/v2".to_string()],
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn round_trip(tag: &str, entry: String, roots: Vec<String>) {
        let scratch =
            std::path::PathBuf::from(format!("/tmp/gunbc_tcstore_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&scratch);
        std::fs::create_dir_all(&scratch).unwrap();
        set_witness_root(Some(scratch.to_string_lossy().into_owned()));
        outcome_reset();

        let cold_index = build_multi_entry_index(&roots);
        resolve_entry_with_index(&cold_index, &entry).expect("cold resolve");
        let (hit0, miss0, committed0) = outcome_snapshot();
        assert_eq!(hit0, 0, "a cold store cannot hit");
        assert!(
            miss0 > 0 && committed0 >= miss0,
            "every miss commits ({miss0} miss, {committed0} committed): {:?}",
            store_receipt_line()
        );
        // The opening is one fact per session: every lookup and commit of the cold run (hundreds
        // of modules) shared one opening.
        let opened = OUTCOMES.with(|c| c.get());
        assert_eq!(
            opened.store_openings, 1,
            "N module lookups must initialize the store exactly once"
        );
        let cold = typed_results(&cold_index);
        assert!(!cold.is_empty());

        let warm_index = build_multi_entry_index(&roots);
        resolve_entry_with_index(&warm_index, &entry).expect("warm resolve");
        let (hit1, miss1, committed1) = outcome_snapshot();
        assert_eq!(
            committed1, committed0,
            "commit exactly once per key: a warm run (restores and in-process hits) commits nothing"
        );
        assert_eq!(
            miss1, miss0,
            "warm run must not miss: every module restores"
        );
        assert!(
            hit1 >= miss0,
            "every module hit ({hit1} hit, {miss0} cold misses)"
        );
        assert_eq!(
            OUTCOMES.with(|c| c.get()).store_openings,
            1,
            "the warm run reuses the session's opening"
        );
        let warm = typed_results(&warm_index);
        // The cold run also typechecked the store's own closure inside the session prepare
        // (counted reentry), so it holds MORE modules than the warm run needs; every module the
        // warm run restored must equal what the cold run computed.
        assert!(!warm.is_empty());
        for (k, b) in warm.iter() {
            let (_, a) = cold
                .iter()
                .find(|(ka, _)| ka == k)
                .unwrap_or_else(|| panic!("module key {k} missing from the cold index"));
            // What the store persists and a restore grafts is the OWN tail. The import-derived head
            // is recomputed from the warm index's graph, which can hold fewer modules than the
            // cold index's (the cold run also typechecked the store closure in the session
            // prepare), exactly as the in-process content-keyed cache already shares one module
            // result across entry graphs; so the head is not compared across the two graphs.
            let (oa, ob) = (own_of_result(a), own_of_result(b));
            assert!(
                oa.items == ob.items
                    && oa.func_local == ob.func_local
                    && oa.item_registry == ob.item_registry
                    && oa.diagnostics == ob.diagnostics,
                "restored own tail != computed own tail for module key {k}"
            );
            assert!(a.typed.module == b.typed.module && a.typed.progress == b.typed.progress);
        }

        // size control: the entry is the module's own tail; the closure snapshot (parent envs)
        // is what made the deleted store multi-GB.
        // Measured over the modules the store committed (the ones the warm run restored): the
        // cold index also holds the store closure's ~two hundred modules from the session
        // prepare, and serializing each of those in full is the test's own cost, not the
        // product's.
        let largest_full = cold
            .iter()
            .filter(|(k, _)| warm.iter().any(|(kw, _)| kw == k))
            .map(|(_, r)| {
                let mut w = CountingWriter(0);
                serde_json::to_writer(&mut w, &**r).expect("serialize result");
                w.0
            })
            .max()
            .unwrap();
        let largest_entry = observed_largest_entry_bytes();
        assert!(largest_entry > 0);
        assert!(
            largest_entry * 2 < largest_full,
            "entry {largest_entry} bytes is not materially smaller than the full result {largest_full}: parent envs leaked in"
        );
        set_witness_root(None);
        let _ = std::fs::remove_dir_all(&scratch);
    }
}

/// Controls for the session's reach: the lookup resolves the store closure through the CALLER's
/// own index, so it neither depends on the cwd nor demands a second pool, and a pool that does not
/// hold the store module is a typed, counted Unavailable.
#[cfg(test)]
mod session_reach {
    use super::*;
    use crate::cli_run::{build_multi_entry_index, workspace_root};

    fn fixture_index() -> (std::path::PathBuf, MultiEntryIndex) {
        let root = workspace_root()
            .join("target")
            .join(format!("gunbc_tcstore_reach_{}", std::process::id()));
        std::fs::create_dir_all(root.join("fx")).unwrap();
        std::fs::write(
            root.join("fx/one.dag"),
            "module fx.one\nfn one() -> Int { 1 }\n",
        )
        .unwrap();
        let idx = build_multi_entry_index(&[root.to_string_lossy().into_owned()]);
        (root, idx)
    }

    #[test]
    fn a_pool_without_the_store_module_is_a_counted_unavailable_not_a_miss() {
        let (root, idx) = fixture_index();
        let hex = "0".repeat(64);
        set_witness_root(Some("/tmp/gunbc_unused".to_string()));
        let before = OUTCOMES.with(|c| c.get());
        let got = durable_typecheck_lookup(&idx, &hex, &[], &hex);
        let own = TypecheckModuleOwn {
            items: Default::default(),
            func_local: Default::default(),
            item_registry: Default::default(),
            diagnostics: Default::default(),
        };
        let committed = durable_typecheck_commit(&idx, &hex, &[], &hex, &own);
        let after = OUTCOMES.with(|c| c.get());
        set_witness_root(None);
        let _ = std::fs::remove_dir_all(root);
        assert!(matches!(got, Ok(DurableTypecheckGet::Unavailable)));
        assert!(committed.is_ok());
        assert_eq!(after.unreachable_lookup, before.unreachable_lookup + 1);
        assert_eq!(after.unreachable_commit, before.unreachable_commit + 1);
        assert_eq!(after.miss, before.miss);
    }

    /// A pool that lacks the selector declaration cannot decide: counted unreachable, no open.
    #[test]
    fn a_pool_without_the_selector_module_is_counted_unreachable_and_never_opens() {
        let (root, idx) = fixture_index();
        let hex = "0".repeat(64);
        set_witness_root(None);
        let before = OUTCOMES.with(|c| c.get());
        let got = durable_typecheck_lookup(&idx, &hex, &[], &hex);
        let after = OUTCOMES.with(|c| c.get());
        let _ = std::fs::remove_dir_all(root);
        assert!(matches!(got, Ok(DurableTypecheckGet::Unavailable)));
        assert_eq!(after.unreachable_lookup, before.unreachable_lookup + 1);
        assert_eq!(after.store_openings, before.store_openings);
    }

    /// The compile runs on a large-stack thread in production; the default test thread's 2 MiB cannot
    /// hold the selector evaluation over a live pool, so the live tests run their body on one too.
    fn on_big_stack(body: impl FnOnce() + Send + 'static) {
        std::thread::Builder::new()
            .stack_size(256 * 1024 * 1024)
            .spawn(body)
            .expect("spawn")
            .join()
            .unwrap_or_else(|e| std::panic::resume_unwind(e));
    }

    /// The declared selector resolves through the caller's real pool: with the variable unset the
    /// invocation is NOT CONFIGURED (counted, nothing opened); an admitted value would configure it.
    #[ignore = "live-corpus: resolves the live dag+src/v2 pool (minutes); the receipts lane runs these with --ignored"]
    #[test]
    fn the_declared_selector_reads_absence_as_not_configured_through_the_callers_pool() {
        on_big_stack(|| {
            let root = workspace_root();
            let idx = build_multi_entry_index(&[
                root.join("dag").to_string_lossy().into_owned(),
                root.join("src/v2").to_string_lossy().into_owned(),
            ]);
            set_witness_root(None);
            if std::env::var("GUNBC_TYPECHECK_STORE").is_ok() {
                return;
            }
            let hex = "0".repeat(64);
            let before = OUTCOMES.with(|c| c.get());
            let got = durable_typecheck_lookup(&idx, &hex, &[], &hex);
            let after = OUTCOMES.with(|c| c.get());
            assert!(
                matches!(got, Ok(DurableTypecheckGet::Miss)),
                "{:?}",
                got.err()
            );
            assert_eq!(
                after.not_configured_lookup,
                before.not_configured_lookup + 1
            );
            assert_eq!(after.store_openings, before.store_openings);
        });
    }

    /// The caller's real pool holds the store: the lookup resolves through that same index from a
    /// foreign cwd, so no second pool is built and nothing is resolved cwd-relative.
    #[ignore = "live-corpus: resolves and typechecks the live dag+src/v2 closure (minutes); the receipts lane runs these with --ignored, the required unit run does not"]
    #[test]
    fn the_callers_pool_serves_the_session_from_a_foreign_cwd() {
        on_big_stack(|| {
            let root = workspace_root();
            let idx = build_multi_entry_index(&[
                root.join("dag").to_string_lossy().into_owned(),
                root.join("src/v2").to_string_lossy().into_owned(),
            ]);
            let saved = std::env::current_dir().expect("cwd");
            std::env::set_current_dir(std::env::temp_dir()).expect("leave workspace");
            let hex = "0".repeat(64);
            let got = durable_typecheck_lookup(&idx, &hex, &[], &hex);
            std::env::set_current_dir(saved).expect("restore cwd");
            assert!(
                got.is_ok()
                    || got
                        .as_ref()
                        .err()
                        .is_some_and(|e| e.contains("digest refused")),
                "{:?}",
                got.err()
            );
        });
    }
}

#[cfg(test)]
mod size_unread_tests {
    use super::*;

    #[test]
    fn a_commit_whose_size_read_fails_does_not_change_the_largest_entry_reading() {
        OBSERVED_LARGEST_ENTRY.with(|c| c.set(0));
        record_stored_entry_bytes(10);
        let before = observed_largest_entry_bytes();
        count(|c| {
            c.commit_committed += 1;
            c.commit_size_unread += 1;
        });
        assert_eq!(observed_largest_entry_bytes(), before);
        assert_eq!(OUTCOMES.with(|c| c.get().commit_size_unread), 1);
    }
}
