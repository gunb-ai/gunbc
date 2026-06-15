// enforce_host_validate.rs — Shared marshal→lens host transport for Gate-1 probes and
// production `gunbc validate`. Not generated — survives stage0 regeneration.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::cli_run::{self, make_eval_context};
use crate::coproduct_reflection::marshal_conj_type_item;
use crate::v2_compiler_compile::{compile_to_resolved, SourceFile};
use crate::v2_compiler_infer_items::{ItemKind, ResolvedGraph};
use crate::v2_interpreter::{run_in_context_with_args, InterpContext, InterpResult, Value};
use crate::v2_std_core::Node;

pub const DEFAULT_HARNESS_ENTRY: &str =
    "src/v4/test/claim/manual/enforce_host_lens_bridge_harness.dag";

const LENS_PROBE_TIMEOUT: Duration = Duration::from_secs(90);

const PROBE_REJECT_FN: &str = "probe_lens_rejects_unit_modeling_from_marshaled_root";
const PROBE_ACCEPT_FN: &str = "probe_lens_accepts_from_marshaled_root";

/// Lens verdict after marshaling a subject `MemorySpec` and running required lens gates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MarshalLensVerdict {
    /// `run_required_lens_gates` returned `Accepted`.
    Accepted,
    /// `run_required_lens_gates` returned `Rejected` (unit-modeling or other).
    Rejected,
}

/// Full validate outcome including compile/transport failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidateOutcome {
    Pass(MarshalLensVerdict),
    Fail {
        reason: String,
    },
}

impl ValidateOutcome {
    /// Production exit code: Accepted → 0, Rejected → 1, transport failure → 2.
    pub fn exit_code(&self) -> i32 {
        match self {
            ValidateOutcome::Pass(MarshalLensVerdict::Accepted) => 0,
            ValidateOutcome::Pass(MarshalLensVerdict::Rejected) => 1,
            ValidateOutcome::Fail { .. } => 2,
        }
    }
}

fn default_source_roots(workspace: &Path) -> Vec<String> {
    vec![
        workspace.join("src/v4").to_string_lossy().to_string(),
        workspace.join("dsl").to_string_lossy().to_string(),
    ]
}

fn resolve_workspace_path(workspace: &Path, relative: &str) -> PathBuf {
    let path = Path::new(relative);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace.join(path)
    }
}

fn compile_probe_bundle(
    workspace: &Path,
    source_roots: &[String],
    harness_entry: &str,
    subject_fixture: &str,
) -> Result<Rc<crate::v2_compiler_compile::ResolvedPipelineResult>, String> {
    let harness_path = resolve_workspace_path(workspace, harness_entry);
    let subject_path = resolve_workspace_path(workspace, subject_fixture);
    let harness_entry = harness_path.to_string_lossy().to_string();
    let subject_entry = subject_path.to_string_lossy().to_string();
    let harness_sources = cli_run::load_sources_for_entry(source_roots, &harness_entry)
        .map_err(|e| format!("load harness {harness_entry}: {e}"))?;
    let subject_sources = cli_run::load_sources_for_entry(source_roots, &subject_entry)
        .map_err(|e| format!("load subject {subject_entry}: {e}"))?;
    let mut by_path: HashMap<String, Rc<SourceFile>> = HashMap::new();
    for source in harness_sources.iter().chain(subject_sources.iter()) {
        by_path.insert(source.path.clone(), source.clone());
    }
    Ok(compile_to_resolved(Rc::new(by_path.into_values().collect())))
}

fn assert_resolved_ok(
    resolved: &Rc<crate::v2_compiler_compile::ResolvedPipelineResult>,
) -> Result<(), String> {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| crate::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    if msgs.is_empty() && resolved.graph.is_some() {
        Ok(())
    } else {
        Err(format!(
            "expected resolved probe graph, diagnostics: {msgs:?}"
        ))
    }
}

fn probe_eval_context(
    resolved: &Rc<crate::v2_compiler_compile::ResolvedPipelineResult>,
) -> InterpContext {
    let graph = resolved.graph.as_ref().expect("probe graph");
    make_eval_context(graph, resolved.source_indices.clone())
}

fn find_type_item<'a>(graph: &'a ResolvedGraph, type_name: &str) -> Result<&'a Rc<Node>, String> {
    let info = graph
        .item_registry
        .values()
        .find(|info| info.kind == ItemKind::TypeItem && info.name == type_name)
        .ok_or_else(|| format!("{type_name} not in item_registry"))?;
    graph
        .modules
        .iter()
        .flat_map(|m| m.items.iter())
        .find(|item| {
            graph
                .item_registry
                .get(&item.name)
                .is_some_and(|i| i.kind == ItemKind::TypeItem && i.name == info.name)
        })
        .ok_or_else(|| format!("{type_name} type item node missing"))
}

fn memory_spec_root_value(
    ctx: &InterpContext,
    resolved: &Rc<crate::v2_compiler_compile::ResolvedPipelineResult>,
) -> Result<Value, String> {
    let graph = resolved
        .graph
        .as_ref()
        .ok_or_else(|| "resolved probe graph missing".to_string())?;
    let item = find_type_item(graph, "MemorySpec")?;
    marshal_conj_type_item(ctx, item).map_err(|e| format!("marshal MemorySpec: {e}"))
}

fn run_probe_fn_timed(
    ctx: &InterpContext,
    fn_name: &str,
    root: Value,
) -> Result<bool, String> {
    let start = Instant::now();
    let args = [(Some("root".to_string()), root)];
    let result: InterpResult<Value> = run_in_context_with_args(ctx, fn_name, &args, false);
    let elapsed = start.elapsed();
    if elapsed > LENS_PROBE_TIMEOUT {
        return Err(format!(
            "HANG: {fn_name} exceeded {:?} (elapsed {:?})",
            LENS_PROBE_TIMEOUT, elapsed
        ));
    }
    match result {
        Ok(Value::Bool(v)) => Ok(v),
        Ok(other) => Err(format!("probe {fn_name}: expected Bool, got {other:?}")),
        Err(e) => Err(format!("probe {fn_name}: {e}")),
    }
}

/// Marshal `subject_fixture` and classify lens verdict via the bridge harness probes.
pub fn validate_marshal_lens(
    workspace: &Path,
    source_roots: &[String],
    harness_entry: &str,
    subject_fixture: &str,
) -> ValidateOutcome {
    let resolved = match compile_probe_bundle(workspace, source_roots, harness_entry, subject_fixture)
    {
        Ok(r) => r,
        Err(e) => {
            return ValidateOutcome::Fail {
                reason: e,
            };
        }
    };
    if let Err(e) = assert_resolved_ok(&resolved) {
        return ValidateOutcome::Fail { reason: e };
    }
    let ctx = probe_eval_context(&resolved);
    let root = match memory_spec_root_value(&ctx, &resolved) {
        Ok(v) => v,
        Err(e) => {
            return ValidateOutcome::Fail { reason: e };
        }
    };

    let reject_holds = match run_probe_fn_timed(&ctx, PROBE_REJECT_FN, root.clone()) {
        Ok(v) => v,
        Err(e) => {
            return ValidateOutcome::Fail { reason: e };
        }
    };
    let accept_holds = match run_probe_fn_timed(&ctx, PROBE_ACCEPT_FN, root) {
        Ok(v) => v,
        Err(e) => {
            return ValidateOutcome::Fail { reason: e };
        }
    };

    if accept_holds && !reject_holds {
        return ValidateOutcome::Pass(MarshalLensVerdict::Accepted);
    }
    if reject_holds && !accept_holds {
        return ValidateOutcome::Pass(MarshalLensVerdict::Rejected);
    }
    ValidateOutcome::Fail {
        reason: format!(
            "ambiguous marshal lens probes (reject={reject_holds}, accept={accept_holds})"
        ),
    }
}

/// Entry point for `gunbc validate`. Exits the process with production codes.
pub fn handle_validate(
    source_roots: Vec<String>,
    harness_entry: String,
    subject_fixture: String,
) {
    let workspace = std::env::current_dir().unwrap_or_else(|e| {
        eprintln!("error: cannot determine working directory: {e}");
        std::process::exit(2);
    });
    let roots = if source_roots.is_empty() {
        default_source_roots(&workspace)
    } else {
        source_roots
    };
    if !resolve_workspace_path(&workspace, &subject_fixture).is_file() {
        eprintln!(
            "error: subject fixture does not exist: {}",
            resolve_workspace_path(&workspace, &subject_fixture).display()
        );
        std::process::exit(2);
    }
    if !resolve_workspace_path(&workspace, &harness_entry).is_file() {
        eprintln!(
            "error: harness entry does not exist: {}",
            resolve_workspace_path(&workspace, &harness_entry).display()
        );
        std::process::exit(2);
    }

    match validate_marshal_lens(&workspace, &roots, &harness_entry, &subject_fixture) {
        ValidateOutcome::Pass(MarshalLensVerdict::Accepted) => {}
        ValidateOutcome::Pass(MarshalLensVerdict::Rejected) => {
            std::process::exit(1);
        }
        ValidateOutcome::Fail { reason } => {
            eprintln!("error: {reason}");
            std::process::exit(2);
        }
    }
}
