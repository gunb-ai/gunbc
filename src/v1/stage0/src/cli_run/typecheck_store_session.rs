//! Distinct seed operations that look up and commit TypecheckModuleRequest through
//! `gunbc.typecheck_module_store` (C2). Not arguments on `typecheck_module`.

use std::cell::{Cell, RefCell};

use crate::v1_compiler_infer::{TypecheckModuleOwn, TypecheckModuleResult};
use crate::v1_interpreter::{self, list_value, str_value, ExecutionMode, Value};
use im::Vector;

use super::{make_eval_context, resolve_entry_graph_shared, witness_layer_roots};

const STORE_ENTRY: &str = "dag/gunbc/typecheck_module_store.dag";

thread_local! {
    static PREPARING: Cell<bool> = const { Cell::new(false) };
    static SESSION: RefCell<Option<v1_interpreter::InterpContext>> = const { RefCell::new(None) };
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

/// (function, leading root arg) the session calls: production door, or the witness door in tests.
fn door(
    production: &'static str,
    witness: &'static str,
) -> (&'static str, Vec<(Option<String>, Value)>) {
    #[cfg(test)]
    if let Some(root) = WITNESS_ROOT.with(|w| w.borrow().clone()) {
        return (
            witness,
            vec![(Some("root_path".to_string()), str_value(root))],
        );
    }
    let _ = witness;
    (production, Vec::new())
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
    reentry_lookup: u64,
    reentry_commit: u64,
}

thread_local! {
    static OUTCOMES: Cell<OutcomeCounts> = const { Cell::new(OutcomeCounts {
        hit: 0, miss: 0, unavailable: 0, commit_committed: 0, commit_missed_open: 0,
        reentry_lookup: 0, reentry_commit: 0,
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
    let degraded = c.unavailable + c.commit_missed_open + c.reentry_lookup + c.reentry_commit;
    let touched = c.hit + c.miss + c.commit_committed + degraded;
    if touched == 0 {
        return None;
    }
    Some(format!(
        "[typecheck-store] hit={} miss={} committed={} unavailable={} commit_missed_open={} reentry_lookup={} reentry_commit={} largest_entry_bytes={}",
        c.hit,
        c.miss,
        c.commit_committed,
        c.unavailable,
        c.commit_missed_open,
        c.reentry_lookup,
        c.reentry_commit,
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

fn with_session<T>(
    f: impl FnOnce(&v1_interpreter::InterpContext) -> Result<T, String>,
) -> Result<T, String> {
    if PREPARING.with(|c| c.get()) {
        return Err("typecheck-store session prepare re-entered".to_string());
    }
    SESSION.with(|slot| {
        if slot.borrow().is_none() {
            PREPARING.with(|c| c.set(true));
            let prepared = (|| {
                let roots = witness_layer_roots();
                let (graph, indices) = resolve_entry_graph_shared(&roots, STORE_ENTRY)?;
                Ok(make_eval_context(&graph, indices, ExecutionMode::Wet))
            })();
            PREPARING.with(|c| c.set(false));
            match prepared {
                Ok(ctx) => *slot.borrow_mut() = Some(ctx),
                Err(e) => return Err(e),
            }
        }
        let borrow = slot.borrow();
        let ctx = borrow.as_ref().expect("session installed");
        f(ctx)
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
    source_digest_hex: &str,
    import_interface_hexes: &[String],
    compiler_digest_hex: &str,
) -> Result<DurableTypecheckGet, String> {
    if PREPARING.with(|c| c.get()) {
        count(|c| c.reentry_lookup += 1);
        return Ok(DurableTypecheckGet::Miss);
    }
    let imports = list_value(
        import_interface_hexes
            .iter()
            .cloned()
            .map(str_value)
            .collect::<Vector<_>>(),
    );
    let (function, mut args) = door(
        "seed_lookup_typecheck_hex",
        "seed_lookup_typecheck_witness_hex",
    );
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
    with_session(|ctx| {
        let result = v1_interpreter::run_in_context_with_args(ctx, function, &args, false)
            .map_err(|e| format!("seed_lookup_typecheck_hex: {e}"))?;
        let Value::Variant {
            variant_name,
            fields,
            ..
        } = &result
        else {
            return Err(format!(
                "seed_lookup_typecheck_hex not a variant: {result:?}"
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
            "seed_lookup_typecheck_hex unknown variant {}",
            ctx.resolve(*variant_name)
        ))
    })
}

/// Distinct from `index_insert_typed`: durable TypecheckModuleRequest commit.
pub(crate) fn durable_typecheck_commit(
    source_digest_hex: &str,
    import_interface_hexes: &[String],
    compiler_digest_hex: &str,
    own: &TypecheckModuleOwn,
) -> Result<(), String> {
    if PREPARING.with(|c| c.get()) {
        count(|c| c.reentry_commit += 1);
        return Ok(());
    }
    let (interface_text, diagnostics_text) = own_payload_texts(own)?;
    let bytes = (interface_text.len() + diagnostics_text.len()) as u64;
    OBSERVED_LARGEST_ENTRY.with(|c| {
        if bytes > c.get() {
            c.set(bytes);
        }
    });
    let imports = list_value(
        import_interface_hexes
            .iter()
            .cloned()
            .map(str_value)
            .collect::<Vector<_>>(),
    );
    let (function, mut args) = door(
        "seed_commit_typecheck_hex",
        "seed_commit_typecheck_witness_hex",
    );
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
    with_session(|ctx| {
        let result = v1_interpreter::run_in_context_with_args(ctx, function, &args, false)
            .map_err(|e| format!("seed_commit_typecheck_hex: {e}"))?;
        let Value::Variant {
            variant_name,
            fields,
            ..
        } = &result
        else {
            return Err(format!(
                "seed_commit_typecheck_hex not a variant: {result:?}"
            ));
        };
        if ctx.sym_eq(*variant_name, "SeedTypecheckCommitted") {
            count(|c| c.commit_committed += 1);
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
            "seed_commit_typecheck_hex unknown variant {}",
            ctx.resolve(*variant_name)
        ))
    })
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

    #[test]
    fn restore_equals_compute_and_entry_excludes_parent_envs() {
        let scratch =
            std::path::PathBuf::from(format!("/tmp/gunbc_tcstore_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&scratch);
        std::fs::create_dir_all(&scratch).unwrap();
        set_witness_root(Some(scratch.to_string_lossy().into_owned()));
        outcome_reset();
        let (entry, roots) = entry_and_roots();

        let cold_index = build_multi_entry_index(&roots);
        resolve_entry_with_index(&cold_index, &entry).expect("cold resolve");
        let (hit0, miss0, committed0) = outcome_snapshot();
        assert_eq!(hit0, 0, "a cold store cannot hit");
        assert!(
            miss0 > 0 && committed0 >= miss0,
            "every miss commits ({miss0} miss, {committed0} committed): {:?}",
            store_receipt_line()
        );
        let cold = typed_results(&cold_index);
        assert!(!cold.is_empty());

        let warm_index = build_multi_entry_index(&roots);
        resolve_entry_with_index(&warm_index, &entry).expect("warm resolve");
        let (hit1, miss1, _) = outcome_snapshot();
        assert_eq!(
            miss1, miss0,
            "warm run must not miss: every module restores"
        );
        assert!(
            hit1 >= miss0,
            "every module hit ({hit1} hit, {miss0} cold misses)"
        );
        let warm = typed_results(&warm_index);
        assert_eq!(warm.len(), cold.len());
        for ((k, a), (kb, b)) in cold.iter().zip(warm.iter()) {
            assert_eq!(k, kb);
            assert!(
                a == b,
                "restore(head, stored own tail) != computed result for module key {k}"
            );
        }

        // size control: the entry is the module's own tail; the closure snapshot (parent envs)
        // is what made the deleted store multi-GB.
        let largest_full = cold
            .iter()
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
