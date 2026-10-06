//! Distinct seed operations that look up and commit TypecheckModuleRequest through
//! `gunbc.typecheck_module_store` (C2). Not arguments on `typecheck_module`.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::v1_compiler_infer::TypecheckModuleResult;
use crate::v1_interpreter::{self, list_value, str_value, ExecutionMode, Value};
use im::Vector;

use std::path::Path;

use super::{make_eval_context, resolve_entry_graph_shared, witness_layer_roots};

const STORE_ENTRY: &str = "dag/gunbc/typecheck_module_store.dag";
const DURABLE_ROOT: &str = "/var/lib/gunbc/materialization-store";

thread_local! {
    static PREPARING: Cell<bool> = const { Cell::new(false) };
    static SESSION: RefCell<Option<v1_interpreter::InterpContext>> = const { RefCell::new(None) };
    static OBSERVED_LARGEST_ENTRY: Cell<u64> = const { Cell::new(0) };
}

pub(crate) enum DurableTypecheckGet {
    Hit(Rc<TypecheckModuleResult>),
    Miss,
    Unavailable,
}

fn durable_volume_present() -> bool {
    Path::new(DURABLE_ROOT).is_dir()
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
        return Ok(DurableTypecheckGet::Miss);
    }
    if !durable_volume_present() {
        return Ok(DurableTypecheckGet::Unavailable);
    }
    let imports = list_value(
        import_interface_hexes
            .iter()
            .cloned()
            .map(str_value)
            .collect::<Vector<_>>(),
    );
    let args = [
        (
            Some("source_digest_hex".to_string()),
            str_value(source_digest_hex),
        ),
        (Some("import_interface_hexes".to_string()), imports),
        (
            Some("compiler_digest_hex".to_string()),
            str_value(compiler_digest_hex),
        ),
    ];
    with_session(|ctx| {
        let result = v1_interpreter::run_in_context_with_args(
            ctx,
            "seed_lookup_typecheck_hex",
            &args,
            false,
        )
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
            return Ok(DurableTypecheckGet::Miss);
        }
        if ctx.sym_eq(*variant_name, "SeedTypecheckUnavailable") {
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
            let computed: TypecheckModuleResult = serde_json::from_str(&interface_text)
                .map_err(|e| format!("typecheck-store hit is not a TypecheckModuleResult: {e}"))?;
            return Ok(DurableTypecheckGet::Hit(Rc::new(computed)));
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
    result: &TypecheckModuleResult,
) -> Result<(), String> {
    if PREPARING.with(|c| c.get()) {
        return Ok(());
    }
    if !durable_volume_present() {
        return Ok(());
    }
    let interface_text = serde_json::to_string(result)
        .map_err(|e| format!("typecheck-store serialize TypecheckModuleResult: {e}"))?;
    let diagnostics_text = serde_json::to_string(&result.diagnostics)
        .map_err(|e| format!("typecheck-store serialize diagnostics: {e}"))?;
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
    let args = [
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
    ];
    with_session(|ctx| {
        let result = v1_interpreter::run_in_context_with_args(
            ctx,
            "seed_commit_typecheck_hex",
            &args,
            false,
        )
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
        if ctx.sym_eq(*variant_name, "SeedTypecheckCommitted")
            || ctx.sym_eq(*variant_name, "SeedTypecheckCommitMissedOpen")
        {
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
