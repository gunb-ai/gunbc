//! THE EMITTED CLOSURE, BUILT AS A MULTI-CRATE CARGO WORKSPACE (Pkg7e), with its discriminating red.
//!
//! `//gunbc/instruments:emitted-crate-workspace`. The seed emits `v2.compiler.compile`'s closure
//! once; `v2.workflow.emitted_crate_workspace` `emitted_workspace_plan` partitions it into crate
//! rows from the derived module DAG, the emission's own returned edges and the transitive unit
//! dependencies; `v1.compiler.stage0_crates` `partition_crate_boundary_emit_outcome_over` renders
//! those rows with the one partition renderer; this host writes the tree and runs cargo.
//!
//! WHAT THIS FILE DECIDES: nothing about the partition. Every row, dependency and the dropped edge
//! come from the `.dag` plan; the manifests and lib.rs files come from the stage0 renderer. What is
//! here is the effect boundary -- the emission, the file writes, the two cargo runs -- plus the
//! transport that carries the emission's edges into the plan and the plan's rows back out.
//!
//! THE RED IS A DROPPED DERIVED DEPENDENCY THE GLOB FACADE CANNOT RE-EXPORT BY ANOTHER ROAD. The
//! plan picks a unit dep edge in the transitive reduction that the emitter also wrote as a use-line;
//! the red rows remove that package from its source crate's manifest and facade. A green there would
//! mean the facade re-exports the target through some other dependency, which the reduction
//! excludes, or that cargo never read the tree. Both runs are in one invocation.

use std::path::Path;
use std::rc::Rc;

use crate::gunbc_stage0_crate_partition_generated::{
    GeneratedPartitionCrateKind, GeneratedPartitionCrateRow,
};
use crate::v1_interpreter::{self, ExecutionMode, Value};

const WORKSPACE_ENTRY: &str = "src/v2/compiler/00_compile.dag";
const PLAN_MODULE: &str = "src/v2/workflow/emitted_crate_workspace.dag";

pub struct EmittedCrateWorkspaceHeld {
    pub head: String,
    pub closure_modules: i64,
    pub crate_count: i64,
    pub facade_is_whole_closure: bool,
    pub rustc_identity: String,
    pub green_exit_status: i64,
    pub green_warning_count: i64,
    pub red_package: String,
    pub red_dropped_package: String,
    pub red_from_module: String,
    pub red_to_module: String,
    pub red_diagnostic: String,
}

struct PlanRows {
    rows: Vec<GeneratedPartitionCrateRow>,
    red_rows: Vec<GeneratedPartitionCrateRow>,
    red_package: String,
    red_dropped_package: String,
    red_from_module: String,
    red_to_module: String,
    module_count: i64,
    crate_count: i64,
    facade_is_whole_closure: bool,
}

fn refusal(cause: &str, detail: impl std::fmt::Display) -> String {
    format!("EMITTED-WORKSPACE REFUSAL cause={cause} — {detail}")
}

fn field<'a>(
    ctx: &v1_interpreter::InterpContext,
    fields: &'a [(v1_interpreter::Symbol, Value)],
    name: &str,
) -> Result<&'a Value, String> {
    fields
        .iter()
        .find(|(sym, _)| ctx.sym_eq(*sym, name))
        .map(|(_, v)| v)
        .ok_or_else(|| refusal("PlanShapeUnexpected", format!("no field `{name}`")))
}

fn as_str(v: &Value, what: &str) -> Result<String, String> {
    match v {
        Value::Str(s) => Ok(s.to_string()),
        other => Err(refusal(
            "PlanShapeUnexpected",
            format!("{what} is {} not a String", other.type_label_public()),
        )),
    }
}

fn as_int(v: &Value, what: &str) -> Result<i64, String> {
    match v {
        Value::Int(i) => Ok(*i),
        other => Err(refusal(
            "PlanShapeUnexpected",
            format!("{what} is {} not an Int", other.type_label_public()),
        )),
    }
}

fn as_bool(v: &Value, what: &str) -> Result<bool, String> {
    match v {
        Value::Bool(b) => Ok(*b),
        other => Err(refusal(
            "PlanShapeUnexpected",
            format!("{what} is {} not a Bool", other.type_label_public()),
        )),
    }
}

fn as_str_list(v: &Value, what: &str) -> Result<Vec<String>, String> {
    match v {
        Value::List(items) => items.iter().map(|i| as_str(i, what)).collect(),
        other => Err(refusal(
            "PlanShapeUnexpected",
            format!("{what} is {} not a List", other.type_label_public()),
        )),
    }
}

fn str_list(xs: &[String]) -> Value {
    Value::List(Rc::new(
        xs.iter()
            .map(|s| v1_interpreter::str_value(s))
            .collect::<Vec<Value>>()
            .into(),
    ))
}

fn row_from_value(
    ctx: &v1_interpreter::InterpContext,
    v: &Value,
) -> Result<GeneratedPartitionCrateRow, String> {
    let Value::Record { fields, .. } = v else {
        return Err(refusal(
            "PlanShapeUnexpected",
            "a crate row is not a record",
        ));
    };
    let kind = match field(ctx, fields, "kind")? {
        Value::Variant { variant_name, .. }
            if ctx.sym_eq(*variant_name, "GeneratedFoundationCrate") =>
        {
            GeneratedPartitionCrateKind::GeneratedFoundationCrate
        }
        Value::Variant { variant_name, .. }
            if ctx.sym_eq(*variant_name, "GeneratedLayeredCoreCrate") =>
        {
            GeneratedPartitionCrateKind::GeneratedLayeredCoreCrate
        }
        Value::Variant { variant_name, .. }
            if ctx.sym_eq(*variant_name, "GeneratedFacadeCrate") =>
        {
            GeneratedPartitionCrateKind::GeneratedFacadeCrate
        }
        other => {
            return Err(refusal(
                "PlanShapeUnexpected",
                format!(
                "a crate row's kind is {} — the plan derives foundation, layered and facade crates only",
                other.type_label_public()
            ),
            ))
        }
    };
    Ok(GeneratedPartitionCrateRow {
        package_name: as_str(field(ctx, fields, "package_name")?, "package_name")?,
        crate_dir: as_str(field(ctx, fields, "crate_dir")?, "crate_dir")?,
        kind,
        modules: Rc::new(
            as_str_list(field(ctx, fields, "modules")?, "modules")?
                .into_iter()
                .collect(),
        ),
        reexport_packages: Rc::new(
            as_str_list(
                field(ctx, fields, "reexport_packages")?,
                "reexport_packages",
            )?
            .into_iter()
            .collect(),
        ),
        carries_non_empty_wrappers: as_bool(
            field(ctx, fields, "carries_non_empty_wrappers")?,
            "carries_non_empty_wrappers",
        )?,
    })
}

fn rows_from_value(
    ctx: &v1_interpreter::InterpContext,
    v: &Value,
    what: &str,
) -> Result<Vec<GeneratedPartitionCrateRow>, String> {
    match v {
        Value::List(items) => items.iter().map(|i| row_from_value(ctx, i)).collect(),
        other => Err(refusal(
            "PlanShapeUnexpected",
            format!("{what} is {} not a List", other.type_label_public()),
        )),
    }
}

/// Evaluate `emitted_workspace_plan` over the emission's basenames and edges. The plan derives the
/// module DAG from the live dependency facts itself, so the corpus it reads is the source roots.
fn evaluate_plan(
    source_roots: &[String],
    emitted_basenames: &[String],
    edge_froms: &[String],
    edge_tos: &[String],
    edge_provenances: &[String],
) -> Result<PlanRows, String> {
    let index = super::process_shared_index(source_roots);
    let (graph, indices) = super::resolve_entry_with_index(&index, PLAN_MODULE)
        .map_err(|e| refusal("PlanModuleUnresolved", format!("{PLAN_MODULE}: {e}")))?;
    let ctx = super::make_eval_context(&graph, indices, ExecutionMode::Wet);
    let args = vec![
        (
            Some("emitted_basenames".to_string()),
            str_list(emitted_basenames),
        ),
        (Some("edge_froms".to_string()), str_list(edge_froms)),
        (Some("edge_tos".to_string()), str_list(edge_tos)),
        (
            Some("edge_provenances".to_string()),
            str_list(edge_provenances),
        ),
    ];
    let outcome = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(&ctx, "emitted_workspace_plan", &args, false)
    })
    .map_err(|e| refusal("PlanNotEvaluated", e))?;
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = &outcome
    else {
        return Err(refusal(
            "PlanShapeUnexpected",
            "the plan outcome is not a variant",
        ));
    };
    if !ctx.sym_eq(*variant_name, "EmittedWorkspacePlanned") {
        return Err(refusal("PlanRefused", format!("{outcome:?}")));
    }
    let Value::Record { fields: plan, .. } = field(&ctx, fields, "plan")? else {
        return Err(refusal("PlanShapeUnexpected", "plan is not a record"));
    };
    // The RED is the instrument's discriminator over the layout, read through its own `.dag` fold:
    // a layout with no cross-crate edge refuses here, not in the plan.
    let plan_value = field(&ctx, fields, "plan")?.clone();
    let red_outcome = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(
            &ctx,
            "emitted_workspace_red_over",
            &[(Some("plan".to_string()), plan_value)],
            false,
        )
    })
    .map_err(|e| refusal("RedNotEvaluated", e))?;
    let Value::Variant {
        variant_name: red_variant,
        fields: red_fields,
        ..
    } = &red_outcome
    else {
        return Err(refusal(
            "PlanShapeUnexpected",
            "the red outcome is not a variant",
        ));
    };
    if !ctx.sym_eq(*red_variant, "EmittedWorkspaceRedPlanned") {
        return Err(refusal("RedRefused", format!("{red_outcome:?}")));
    }
    let Value::Record { fields: red, .. } = field(&ctx, red_fields, "red")? else {
        return Err(refusal("PlanShapeUnexpected", "red is not a record"));
    };
    Ok(PlanRows {
        rows: rows_from_value(&ctx, field(&ctx, plan, "rows")?, "rows")?,
        red_rows: rows_from_value(&ctx, field(&ctx, red_fields, "red_rows")?, "red_rows")?,
        red_package: as_str(field(&ctx, red, "package")?, "red.package")?,
        red_dropped_package: as_str(field(&ctx, red, "dropped_package")?, "red.dropped_package")?,
        red_from_module: as_str(field(&ctx, red, "from_module")?, "red.from_module")?,
        red_to_module: as_str(field(&ctx, red, "to_module")?, "red.to_module")?,
        module_count: as_int(field(&ctx, plan, "module_count")?, "module_count")?,
        crate_count: as_int(field(&ctx, plan, "crate_count")?, "crate_count")?,
        facade_is_whole_closure: as_bool(
            field(&ctx, plan, "facade_is_whole_closure")?,
            "facade_is_whole_closure",
        )?,
    })
}

/// Render rows through the one partition renderer and write them, with the workspace manifest that
/// lists exactly those crates. The emitted module files sit where the renderer's includes expect
/// them (`src/v1/stage0/src`).
fn write_workspace(
    root: &Path,
    rows: &[GeneratedPartitionCrateRow],
    registry_dependencies: &Rc<im::Vector<Rc<crate::extdeps_cargo::CargoDependency>>>,
) -> Result<(), String> {
    let rows_value = Rc::new(rows.iter().cloned().map(Rc::new).collect());
    let files = match &*crate::v1_compiler_stage0_crates::partition_crate_boundary_emit_outcome_over(
        rows_value,
        registry_dependencies.clone(),
    ) {
        crate::v1_compiler_stage0_crates::Stage0CrateBoundaryEmitOutcome::Stage0CrateBoundaryEmitOk { files } => files.clone(),
        crate::v1_compiler_stage0_crates::Stage0CrateBoundaryEmitOutcome::Stage0CrateBoundaryEmitRefused { cause } => {
            return Err(refusal(
                "RenderRefused",
                crate::v1_compiler_stage0_crates::stage0_crate_boundary_emit_refusal_message(cause.clone()),
            ))
        }
    };
    for file in files.iter() {
        let path = root.join(&*file.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                refusal("WorkspaceNotWritten", format!("{}: {e}", parent.display()))
            })?;
        }
        std::fs::write(&path, &*file.content)
            .map_err(|e| refusal("WorkspaceNotWritten", format!("{}: {e}", path.display())))?;
    }
    // The workspace section lists every crate beneath the root. A package AT the root (the top
    // package of an emitted workspace, crate_dir ".") is the workspace's root package, so the
    // section is appended to its manifest rather than written over it.
    let members = rows
        .iter()
        .filter(|r| r.crate_dir != ".")
        .map(|r| format!("    \"{}\",", r.crate_dir))
        .collect::<Vec<_>>()
        .join("\n");
    let section = format!("\n[workspace]\nresolver = \"2\"\nmembers = [\n{members}\n]\n");
    let root_manifest = root.join("Cargo.toml");
    let head = if rows.iter().any(|r| r.crate_dir == ".") {
        std::fs::read_to_string(&root_manifest)
            .map_err(|e| refusal("WorkspaceNotWritten", format!("root package manifest: {e}")))?
    } else {
        String::new()
    };
    std::fs::write(&root_manifest, format!("{head}{section}"))
        .map_err(|e| refusal("WorkspaceNotWritten", format!("root manifest: {e}")))
}

/// The receipt names the revision it measured; an unreadable head refuses rather than printing a
/// receipt that cannot be joined to a tree.
fn git_head(workspace: &Path) -> Result<String, String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(workspace)
        .output()
        .map_err(|e| refusal("HeadUnreadable", e))?;
    if !out.status.success() {
        return Err(refusal(
            "HeadUnreadable",
            String::from_utf8_lossy(&out.stderr),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub fn run_emitted_crate_workspace(
    source_roots: &[String],
) -> Result<EmittedCrateWorkspaceHeld, String> {
    let workspace = super::process_workspace_root();
    let head = git_head(&workspace)?;
    let probe_root =
        super::lane_emit_compile_probe_root().map_err(|c| refusal("ProbeRootNotCreated", c))?;
    eprintln!(
        "emitted-crate-workspace: head={head} probe root {}",
        probe_root.display()
    );

    eprintln!("emitted-crate-workspace: emitting {WORKSPACE_ENTRY} (seed, in-process)");
    let run = super::compile_entry_emission(
        source_roots,
        WORKSPACE_ENTRY,
        true,
        crate::v1_compiler_artifact::RenderTarget::Rust,
    );
    match &run.disposition {
        super::CompileDisposition::Completed { .. } => {}
        super::CompileDisposition::Refused { phase, cause } => {
            return Err(refusal("EmissionRefused", format!("stage={phase} {cause}")))
        }
        super::CompileDisposition::NotExecuted {
            earlier_phase,
            cause,
        } => {
            return Err(refusal(
                "EmissionRefused",
                format!("stage={earlier_phase} {cause}"),
            ))
        }
    }
    let emission = run
        .emissions
        .iter()
        .find(|e| e.target_name == "rust")
        .ok_or_else(|| refusal("EmissionRefused", "no rust target"))?;

    let root = probe_root.target_dir().join("emitted_crate_workspace");
    let _ = std::fs::remove_dir_all(&root);
    let written = write_emitted_modules(&root, &emission.result.files)?;
    let emitted_basenames = written.basenames;
    let (froms, tos, provenances) = edge_columns(&emission.result.emitted_edges);
    let registry_dependencies = emission_registry_dependencies(&emission.result.rust_crates)?;
    eprintln!(
        "emitted-crate-workspace: emitted {} module files and {} edges; evaluating the partition plan",
        emitted_basenames.len(),
        froms.len()
    );
    drop(run);
    let _ = super::trim_retained_heap();

    let plan = evaluate_plan(source_roots, &emitted_basenames, &froms, &tos, &provenances)?;
    eprintln!(
        "emitted-crate-workspace: plan modules={} crates={} facade_is_whole_closure={} red={} drops {} ({} -> {})",
        plan.module_count, plan.crate_count, plan.facade_is_whole_closure, plan.red_package, plan.red_dropped_package,
        plan.red_from_module, plan.red_to_module
    );

    let target_dir = probe_root
        .target_dir()
        .join("emitted_crate_workspace_target");
    let invocation =
        super::emitted_closure_compile_host::probe_cargo_invocation(&root, &target_dir)
            .map_err(|c| refusal("CargoNotResolved", c))?;

    // POSITIVE CONTROL: the derived partition builds.
    write_workspace(&root, &plan.rows, &registry_dependencies)?;
    let green =
        super::emitted_closure_compile_host::run_cargo(&root, &target_dir, &plan.red_to_module);
    let (green_exit_status, green_warning_count) = match &green {
        super::emitted_closure_compile_host::CargoVerdict::Completed {
            status,
            warning_count,
            ..
        } => (i64::from(*status), *warning_count as i64),
        other => {
            return Err(refusal(
                "GreenBuildNotCompleted",
                super::emitted_closure_compile_host::cargo_verdict_summary(other),
            ))
        }
    };
    eprintln!(
        "emitted-crate-workspace: GREEN {} rustc={}",
        super::emitted_closure_compile_host::cargo_verdict_summary(&green),
        invocation.rustc_identity
    );

    if green_exit_status != 0 {
        // The positive control did not build, so there is no clean baseline for a red to be read
        // against. That is the observation not holding, reported with cargo's own tail; the red is
        // not attempted rather than reported as refused.
        return Ok(EmittedCrateWorkspaceHeld {
            head,
            closure_modules: plan.module_count,
            crate_count: plan.crate_count,
            facade_is_whole_closure: plan.facade_is_whole_closure,
            rustc_identity: invocation.rustc_identity,
            green_exit_status,
            green_warning_count,
            red_package: plan.red_package,
            red_dropped_package: plan.red_dropped_package,
            red_from_module: plan.red_from_module,
            red_to_module: plan.red_to_module,
            red_diagnostic: format!(
                "not attempted: the positive control did not build — {}",
                super::emitted_closure_compile_host::cargo_verdict_summary(&green)
            ),
        });
    }

    // RED: the same tree with one derived dependency dropped from one crate.
    write_workspace(&root, &plan.red_rows, &registry_dependencies)?;
    let red =
        super::emitted_closure_compile_host::run_cargo(&root, &target_dir, &plan.red_to_module);
    let red_diagnostic = match &red {
        super::emitted_closure_compile_host::CargoVerdict::Completed { status, .. } if *status != 0 => {
            match (
                super::emitted_closure_compile_host::cargo_verdict_probe_diagnostic(&red),
                super::emitted_closure_compile_host::cargo_verdict_probe_line(&red),
            ) {
                (Some(diag), Some(line)) => format!("{diag} @ {line}"),
                _ => {
                    return Err(refusal(
                        "RedNotAttributed",
                        format!(
                            "the red build failed but no error names {} — {}",
                            plan.red_to_module,
                            super::emitted_closure_compile_host::cargo_verdict_summary(&red)
                        ),
                    ))
                }
            }
        }
        super::emitted_closure_compile_host::CargoVerdict::Completed { .. } => {
            return Err(refusal(
                "RedBuiltGreen",
                format!(
                    "dropping {} from {} still built — the facade re-exports it by another road, or cargo never read the tree",
                    plan.red_dropped_package, plan.red_package
                ),
            ))
        }
        other => return Err(refusal("RedBuildNotCompleted", super::emitted_closure_compile_host::cargo_verdict_summary(other))),
    };
    eprintln!("emitted-crate-workspace: RED refused as required — {red_diagnostic}");
    // Leave the tree as the green plan rendered it, so the directory describes the positive control.
    write_workspace(&root, &plan.rows, &registry_dependencies)?;

    Ok(EmittedCrateWorkspaceHeld {
        head,
        closure_modules: plan.module_count,
        crate_count: plan.crate_count,
        facade_is_whole_closure: plan.facade_is_whole_closure,
        rustc_identity: invocation.rustc_identity,
        green_exit_status,
        green_warning_count,
        red_package: plan.red_package,
        red_dropped_package: plan.red_dropped_package,
        red_from_module: plan.red_from_module,
        red_to_module: plan.red_to_module,
        red_diagnostic,
    })
}

/// Where the partition renderer's crates include emitted module files from: every crate's lib.rs
/// reaches `../../stage0/src/<module>.rs` from `src/v1/<package>` (`v1.compiler.stage0_crates`
/// `stage0_crate_mod_include`), so the files sit at this path under the workspace root.
const EMITTED_MODULE_DIR: &str = "src/v1/stage0/src";

struct EmittedModules {
    module_dir: std::path::PathBuf,
    basenames: Vec<String>,
    /// The emitted entry file, written by the caller where the plan places the binary.
    main_rs: Option<String>,
}

/// Write the emission's module files where the renderer's includes expect them. The single-package
/// assembly's crate root (`src/lib.rs`) and population manifest belong to a layout the partition
/// replaces, so they are not written; `src/main.rs` is returned for the binary target.
fn write_emitted_modules(
    root: &Path,
    files: &Rc<im::Vector<Rc<crate::v1_std_core::TextFile>>>,
) -> Result<EmittedModules, String> {
    let module_dir = root.join(EMITTED_MODULE_DIR);
    std::fs::create_dir_all(&module_dir).map_err(|e| refusal("WorkspaceNotWritten", e))?;
    let mut basenames = Vec::new();
    let mut main_rs = None;
    for file in files.iter() {
        let Some(basename) = file
            .path
            .strip_prefix("src/")
            .and_then(|p| p.strip_suffix(".rs"))
        else {
            continue;
        };
        if basename == "main" {
            main_rs = Some(file.content.to_string());
            continue;
        }
        if basename.contains('/') || matches!(basename, "lib" | "emitted_population") {
            continue;
        }
        std::fs::write(module_dir.join(format!("{basename}.rs")), &*file.content)
            .map_err(|e| refusal("WorkspaceNotWritten", format!("{basename}: {e}")))?;
        basenames.push(basename.to_string());
    }
    Ok(EmittedModules {
        module_dir,
        basenames,
        main_rs,
    })
}

/// The emission's edges as the three parallel columns the plan reads.
fn edge_columns(
    edges: &Rc<im::Vector<Rc<crate::gunbc_rust_emitted_edge::EmittedEdge>>>,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let (mut froms, mut tos, mut provenances) = (Vec::new(), Vec::new(), Vec::new());
    for edge in edges.iter() {
        froms.push(edge.from.clone());
        tos.push(match &*edge.to {
            crate::gunbc_rust_emitted_edge::EmittedEdgeTarget::EmittedModuleTarget { module } => {
                module.clone()
            }
            crate::gunbc_rust_emitted_edge::EmittedEdgeTarget::EmittedCrateRootTarget => {
                String::new()
            }
        });
        provenances.push(
            crate::gunbc_stage0_emitted_edge_admission::stage0_emitted_edge_provenance_name(
                edge.provenance.clone(),
            ),
        );
    }
    (froms, tos, provenances)
}

/// The one Rust crate an emission names. Zero or several is an emission this writer cannot
/// realize, refused rather than guessed.
fn emitted_rust_crate(
    rust_crates: &Rc<im::Vector<Rc<crate::gunbc_rust_emitted_crate::EmittedRustCrate>>>,
) -> Result<Rc<crate::gunbc_rust_emitted_crate::EmittedRustCrate>, String> {
    match (rust_crates.len(), rust_crates.front()) {
        (1, Some(one)) => Ok(one.clone()),
        (count, _) => Err(refusal(
            "EmittedCrateNotUnique",
            format!("the emission names {count} Rust crates, not one"),
        )),
    }
}

fn emission_registry_dependencies(
    rust_crates: &Rc<im::Vector<Rc<crate::gunbc_rust_emitted_crate::EmittedRustCrate>>>,
) -> Result<Rc<im::Vector<Rc<crate::extdeps_cargo::CargoDependency>>>, String> {
    let rust_crate = emitted_rust_crate(rust_crates)?;
    Ok(crate::v1_compiler_stage0_crates::emitted_crate_dependencies(rust_crate.demand.clone()))
}

/// AN EMITTED CLOSURE, WRITTEN AS THE CARGO WORKSPACE ITS PARTITION PLAN DERIVES. The one writer
/// every emitted-Rust build route uses: the layout, the top package the emitted main paths
/// through and the binary's place are `v2.workflow.emitted_crate_workspace`
/// `emitted_workspace_build`'s; the manifests and lib.rs files are
/// `v1.compiler.stage0_crates` `partition_crate_boundary_emit_outcome_over`'s; the registry
/// dependencies are the emission's own demand. What is here is the file writes and the transport.
pub struct EmittedWorkspace {
    /// The workspace root: the directory whose Cargo.toml `run_cargo` builds.
    pub root: std::path::PathBuf,
    /// Where the emitted module files are, by basename.
    pub module_dir: std::path::PathBuf,
    /// The modules the top package re-exports -- the closure the binary links.
    pub modules: Vec<String>,
    pub crate_count: usize,
    pub written: usize,
}

/// The roots the plan module resolves against. The plan is the compiler's own `.dag`, so it is
/// read from the repository whichever closure is being built -- a fixture's virtual sources
/// included.
const PLAN_SOURCE_ROOTS: [&str; 2] = ["dag", "src/v2"];

pub fn write_emitted_workspace(
    files: &Rc<im::Vector<Rc<crate::v1_std_core::TextFile>>>,
    edges: &Rc<im::Vector<Rc<crate::gunbc_rust_emitted_edge::EmittedEdge>>>,
    rust_crates: &Rc<im::Vector<Rc<crate::gunbc_rust_emitted_crate::EmittedRustCrate>>>,
    root: &Path,
    bin_name: &str,
) -> Result<EmittedWorkspace, String> {
    let rust_crate = emitted_rust_crate(rust_crates)?;
    let registry_dependencies = emission_registry_dependencies(rust_crates)?;
    let modules = write_emitted_modules(root, files)?;
    let main_rs = modules.main_rs.clone().ok_or_else(|| {
        refusal(
            "EntryFileAbsent",
            "the emission wrote no src/main.rs for the binary",
        )
    })?;
    let (froms, tos, provenances) = edge_columns(edges);
    let plan_roots: Vec<String> = PLAN_SOURCE_ROOTS.iter().map(|r| r.to_string()).collect();
    let build = evaluate_build(
        &plan_roots,
        &modules.basenames,
        &froms,
        &tos,
        &provenances,
        &rust_crate.crate_name,
        bin_name,
    )?;
    write_workspace(root, &build.rows, &registry_dependencies)?;
    let bin_path = root.join(&build.bin_source_path);
    if let Some(parent) = bin_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| refusal("WorkspaceNotWritten", format!("{}: {e}", parent.display())))?;
    }
    std::fs::write(&bin_path, main_rs).map_err(|e| {
        refusal(
            "WorkspaceNotWritten",
            format!("{}: {e}", bin_path.display()),
        )
    })?;
    Ok(EmittedWorkspace {
        root: root.to_path_buf(),
        module_dir: modules.module_dir,
        written: modules.basenames.len() + 1,
        modules: build.modules,
        crate_count: build.rows.len(),
    })
}

struct BuildRows {
    rows: Vec<GeneratedPartitionCrateRow>,
    bin_source_path: String,
    modules: Vec<String>,
}

/// Evaluate `emitted_workspace_build` over one emission. The plan reads only its arguments, so the
/// corpus it resolves against is the plan module's own closure.
///
/// THE PLAN'S INDEX IS ITS OWN, BUILT FOR THIS EVALUATION AND DROPPED WITH IT. The emission it
/// lays out was compiled over the caller's roots, which hold the process-shared index; the plan
/// module lives in the repository's roots, a different pool. A reader of another pool owns its
/// index (`build_multi_entry_index`) rather than a second slot in the shared memo, so the two never
/// share residency.
fn evaluate_build(
    source_roots: &[String],
    emitted_basenames: &[String],
    edge_froms: &[String],
    edge_tos: &[String],
    edge_provenances: &[String],
    crate_name: &str,
    bin_name: &str,
) -> Result<BuildRows, String> {
    let index = super::build_multi_entry_index(source_roots);
    let (graph, indices) = super::resolve_entry_with_index(&index, PLAN_MODULE)
        .map_err(|e| refusal("PlanModuleUnresolved", format!("{PLAN_MODULE}: {e}")))?;
    let ctx = super::make_eval_context(&graph, indices, ExecutionMode::Wet);
    let args = vec![
        (
            Some("emitted_basenames".to_string()),
            str_list(emitted_basenames),
        ),
        (Some("edge_froms".to_string()), str_list(edge_froms)),
        (Some("edge_tos".to_string()), str_list(edge_tos)),
        (
            Some("edge_provenances".to_string()),
            str_list(edge_provenances),
        ),
        (
            Some("crate_name".to_string()),
            v1_interpreter::str_value(crate_name),
        ),
        (
            Some("bin_name".to_string()),
            v1_interpreter::str_value(bin_name),
        ),
    ];
    let outcome = v1_interpreter::with_active_context(&ctx, || {
        v1_interpreter::run_in_context_with_args(&ctx, "emitted_workspace_build", &args, false)
    })
    .map_err(|e| refusal("PlanNotEvaluated", e))?;
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = &outcome
    else {
        return Err(refusal(
            "PlanShapeUnexpected",
            "the build outcome is not a variant",
        ));
    };
    if !ctx.sym_eq(*variant_name, "EmittedWorkspaceBuildPlanned") {
        return Err(refusal("PlanRefused", format!("{outcome:?}")));
    }
    let Value::Record { fields: build, .. } = field(&ctx, fields, "build")? else {
        return Err(refusal("PlanShapeUnexpected", "build is not a record"));
    };
    let rows = rows_from_value(&ctx, field(&ctx, build, "rows")?, "rows")?;
    let top_package = as_str(field(&ctx, build, "top_package")?, "top_package")?;
    let modules = rows
        .iter()
        .find(|r| r.package_name == top_package)
        .map(|r| r.modules.iter().cloned().collect())
        .ok_or_else(|| {
            refusal(
                "PlanShapeUnexpected",
                format!("no row names the top package {top_package}"),
            )
        })?;
    Ok(BuildRows {
        rows,
        bin_source_path: as_str(field(&ctx, build, "bin_source_path")?, "bin_source_path")?,
        modules,
    })
}
