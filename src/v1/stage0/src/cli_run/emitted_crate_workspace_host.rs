//! THE EMITTED CLOSURE, BUILT AS A MULTI-CRATE CARGO WORKSPACE (Pkg7e), with its discriminating red.
//!
//! Two callers share one realization. `write_emitted_workspace` is the writer every emitted-Rust
//! route uses: `v1.compiler.emitted_workspace` `emitted_workspace_realization` -- compiled into the
//! seed, not interpreted -- turns an emission into the complete file population of its workspace
//! (layout from `gunbc.emitted_crate_workspace` `emitted_workspace_build`, manifests and crate
//! roots from `v1.compiler.stage0_crates`, every emitted file placed or refused), and this file
//! writes those files. `//gunbc/instruments:emitted-crate-workspace` emits `v2.compiler.compile`,
//! writes it through the same realization, builds it, then lays the RED over it
//! (`emitted_workspace_red_realization`) and builds again.
//!
//! WHAT THIS FILE DECIDES: nothing about the workspace. Every path and byte comes from the `.dag`
//! realization; what is here is the effect boundary -- the emission, the file writes, the cargo runs.
//!
//! THE RED IS A DROPPED DERIVED DEPENDENCY THE GLOB FACADE CANNOT RE-EXPORT BY ANOTHER ROAD. The
//! plan picks a unit dep edge in the transitive reduction that the emitter also wrote as a use-line;
//! the red rows remove that package from its source crate's manifest and facade. A green there would
//! mean the facade re-exports the target through some other dependency, which the reduction
//! excludes, or that cargo never read the tree. Both runs are in one invocation.

use std::path::Path;
use std::rc::Rc;

use crate::v1_compiler_emitted_workspace::{
    emitted_workspace_realization, emitted_workspace_realization_refusal_message,
    emitted_workspace_red_realization, EmittedWorkspaceRealization, EmittedWorkspaceRealized,
    EmittedWorkspaceRedRealization,
};

const WORKSPACE_ENTRY: &str = "src/v2/compiler/00_compile.dag";

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

fn refusal(cause: &str, detail: impl std::fmt::Display) -> String {
    format!("EMITTED-WORKSPACE REFUSAL cause={cause} — {detail}")
}

/// Write realized files under `root`, each at its workspace-relative path.
fn write_files(
    root: &Path,
    files: &im::Vector<Rc<crate::v1_std_core::TextFile>>,
) -> Result<(), String> {
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
    Ok(())
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
    let bin_name = emission
        .result
        .rust_crates
        .front()
        .map(|c| c.crate_name.to_string())
        .ok_or_else(|| refusal("EmissionRefused", "the emission names no Rust crate"))?;

    let root = probe_root.target_dir().join("emitted_crate_workspace");
    let _ = std::fs::remove_dir_all(&root);
    let realized = realize(
        &emission.result.files,
        &emission.result.emitted_edges,
        &emission.result.rust_crates,
        &bin_name,
    )?;
    drop(run);
    let _ = super::trim_retained_heap();
    let layout = &realized.build.layout;
    let red_realized = match &*emitted_workspace_red_realization(realized.clone()) {
        EmittedWorkspaceRedRealization::EmittedWorkspaceRedRealized { red, files } => {
            (red.clone(), files.clone())
        }
        EmittedWorkspaceRedRealization::EmittedWorkspaceRedRealizationRefused { message } => {
            return Err(refusal("RedRefused", message))
        }
    };
    let (red, red_files) = red_realized;
    eprintln!(
        "emitted-crate-workspace: plan modules={} crates={} facade_is_whole_closure={} red={} drops {} ({} -> {})",
        layout.module_count, layout.crate_count, layout.facade_is_whole_closure, red.package, red.dropped_package,
        red.from_module, red.to_module
    );

    let target_dir = probe_root
        .target_dir()
        .join("emitted_crate_workspace_target");
    // POSITIVE CONTROL: the production layout builds. The files land first: the compiler
    // identity is asked from the crate's own directory, which must exist to spawn in.
    write_files(&root, &realized.files)?;
    let invocation =
        super::emitted_closure_compile_host::probe_cargo_invocation(&root, &target_dir)
            .map_err(|c| refusal("CargoNotResolved", c))?;
    let green = super::emitted_closure_compile_host::run_cargo(&root, &target_dir, &red.to_module);
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

    let held = |red_diagnostic: String| EmittedCrateWorkspaceHeld {
        head: head.clone(),
        closure_modules: layout.module_count,
        crate_count: layout.crate_count,
        facade_is_whole_closure: layout.facade_is_whole_closure,
        rustc_identity: invocation.rustc_identity.clone(),
        green_exit_status,
        green_warning_count,
        red_package: red.package.to_string(),
        red_dropped_package: red.dropped_package.to_string(),
        red_from_module: red.from_module.to_string(),
        red_to_module: red.to_module.to_string(),
        red_diagnostic,
    };

    if green_exit_status != 0 {
        // The positive control did not build, so there is no clean baseline for a red to be read
        // against. That is the observation not holding, reported with cargo's own tail; the red is
        // not attempted rather than reported as refused.
        return Ok(held(format!(
            "not attempted: the positive control did not build — {}",
            super::emitted_closure_compile_host::cargo_verdict_summary(&green)
        )));
    }

    // RED: the same tree with one derived dependency dropped from one crate.
    write_files(&root, &red_files)?;
    let red_build =
        super::emitted_closure_compile_host::run_cargo(&root, &target_dir, &red.to_module);
    let red_diagnostic = match &red_build {
        super::emitted_closure_compile_host::CargoVerdict::Completed { status, .. } if *status != 0 => {
            match (
                super::emitted_closure_compile_host::cargo_verdict_probe_diagnostic(&red_build),
                super::emitted_closure_compile_host::cargo_verdict_probe_line(&red_build),
            ) {
                (Some(diag), Some(line)) => format!("{diag} @ {line}"),
                _ => {
                    return Err(refusal(
                        "RedNotAttributed",
                        format!(
                            "the red build failed but no error names {} — {}",
                            red.to_module,
                            super::emitted_closure_compile_host::cargo_verdict_summary(&red_build)
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
                    red.dropped_package, red.package
                ),
            ))
        }
        other => return Err(refusal("RedBuildNotCompleted", super::emitted_closure_compile_host::cargo_verdict_summary(other))),
    };
    eprintln!("emitted-crate-workspace: RED refused as required — {red_diagnostic}");
    // Leave the tree as the green realization wrote it, so the directory describes the positive control.
    write_files(&root, &realized.files)?;
    Ok(held(red_diagnostic))
}

/// AN EMITTED CLOSURE, WRITTEN AS THE CARGO WORKSPACE ITS REALIZATION DERIVES. The one writer every
/// emitted-Rust build route uses. Every path and byte is `v1.compiler.emitted_workspace`
/// `emitted_workspace_realization`'s; what is here is the file writes.
pub struct EmittedWorkspace {
    /// The workspace root: the directory whose Cargo.toml `run_cargo` builds.
    pub root: std::path::PathBuf,
    /// Where the emitted module files are, by basename.
    pub module_dir: std::path::PathBuf,
    /// The modules the top package re-exports -- the closure the binary links.
    pub modules: Vec<String>,
    pub crate_count: usize,
    /// Every file the workspace holds, by path relative to `root`, in the realization's order --
    /// the one population a completeness check or an identity over the workspace reads.
    pub files: Vec<String>,
}

fn realize(
    files: &Rc<im::Vector<Rc<crate::v1_std_core::TextFile>>>,
    edges: &Rc<im::Vector<Rc<crate::gunbc_rust_emitted_edge::EmittedEdge>>>,
    rust_crates: &Rc<im::Vector<Rc<crate::gunbc_rust_emitted_crate::EmittedRustCrate>>>,
    bin_name: &str,
) -> Result<Rc<EmittedWorkspaceRealized>, String> {
    match &*emitted_workspace_realization(
        files.clone(),
        edges.clone(),
        rust_crates.clone(),
        bin_name.to_string(),
    ) {
        EmittedWorkspaceRealization::EmittedWorkspaceRealizedOk { workspace } => {
            Ok(workspace.clone())
        }
        EmittedWorkspaceRealization::EmittedWorkspaceRealizationRefused { cause } => Err(refusal(
            "WorkspaceNotRealized",
            emitted_workspace_realization_refusal_message(cause.clone()),
        )),
    }
}

pub fn write_emitted_workspace(
    files: &Rc<im::Vector<Rc<crate::v1_std_core::TextFile>>>,
    edges: &Rc<im::Vector<Rc<crate::gunbc_rust_emitted_edge::EmittedEdge>>>,
    rust_crates: &Rc<im::Vector<Rc<crate::gunbc_rust_emitted_crate::EmittedRustCrate>>>,
    root: &Path,
    bin_name: &str,
) -> Result<EmittedWorkspace, String> {
    let realized = realize(files, edges, rust_crates, bin_name)?;
    write_files(root, &realized.files)?;
    let top_package = realized.build.top_package.to_string();
    let modules = realized
        .build
        .rows
        .iter()
        .find(|r| *r.package_name == *top_package)
        .map(|r| r.modules.iter().map(|m| m.to_string()).collect())
        .ok_or_else(|| refusal("TopPackageAbsent", format!("no row names {top_package}")))?;
    Ok(EmittedWorkspace {
        root: root.to_path_buf(),
        module_dir: root.join(&*realized.module_dir),
        modules,
        crate_count: realized.build.rows.len(),
        files: realized.files.iter().map(|f| f.path.to_string()).collect(),
    })
}

#[cfg(test)]
mod tests {
    //! The seed's workspace functions are v1 `.dag` compiled into this crate, so their claims run
    //! here, against the compiled functions, rather than in a `.dag` witness whose source roots
    //! cannot import a v1 module. Inputs are supplied; the real emission reaching them is the
    //! v2-native-cli and self-host instruments' build.
    use std::rc::Rc;

    use crate::gunbc_rust_emitted_crate::{EmittedCrateDependencyDemand, EmittedRustCrate};
    use crate::gunbc_rust_emitted_edge::{
        declared_import_edge, rust_module_emit_filename, rust_prelude_emitted_edges, EmittedEdge,
        EmittedEdgeProvenance, EmittedEdgeTarget,
    };
    use crate::v1_compiler_emitted_workspace::{
        emitted_workspace_realization, EmittedWorkspaceRealization,
        EmittedWorkspaceRealizationRefusal,
    };
    use crate::v1_std_core::TextFile;

    fn text(path: &str, content: &str) -> Rc<TextFile> {
        Rc::new(TextFile {
            path: path.to_string(),
            content: content.to_string(),
        })
    }

    fn demand(services: bool) -> EmittedCrateDependencyDemand {
        EmittedCrateDependencyDemand {
            renders_clap_cli: false,
            renders_async_services: services,
        }
    }

    fn edges(modules: &[&str], extra: Vec<Rc<EmittedEdge>>) -> Vec<Rc<EmittedEdge>> {
        let mut all: Vec<Rc<EmittedEdge>> = std::iter::once("v1_rt")
            .chain(modules.iter().copied())
            .flat_map(|m| {
                rust_prelude_emitted_edges(m.to_string())
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect();
        all.extend(extra);
        all
    }

    fn base_files(extra: Vec<Rc<TextFile>>) -> Vec<Rc<TextFile>> {
        let mut files = vec![
            text("src/lib.rs", "pub mod fx_a;\n"),
            text("src/main.rs", "fn main() {}\n"),
            text("src/emitted_population.rs", "// src/lib.rs\n"),
            text("src/v1_rt.rs", "// runtime\n"),
            text("src/fx_a.rs", "pub fn a() {}\n"),
        ];
        files.extend(extra);
        files
    }

    fn realize(
        files: Vec<Rc<TextFile>>,
        edges: Vec<Rc<EmittedEdge>>,
        services: bool,
    ) -> Rc<EmittedWorkspaceRealization> {
        emitted_workspace_realization(
            Rc::new(files.into_iter().collect()),
            Rc::new(edges.into_iter().collect()),
            Rc::new(
                std::iter::once(Rc::new(EmittedRustCrate {
                    crate_name: "fx_app".to_string(),
                    demand: demand(services),
                }))
                .collect(),
            ),
            "fx_app".to_string(),
        )
    }

    fn realized_files(outcome: &EmittedWorkspaceRealization) -> Vec<(String, String)> {
        match outcome {
            EmittedWorkspaceRealization::EmittedWorkspaceRealizedOk { workspace } => workspace
                .files
                .iter()
                .map(|f| (f.path.to_string(), f.content.to_string()))
                .collect(),
            EmittedWorkspaceRealization::EmittedWorkspaceRealizationRefused { cause } => {
                panic!("realization refused: {cause:?}")
            }
        }
    }

    fn content_of<'a>(files: &'a [(String, String)], path: &str) -> &'a str {
        files
            .iter()
            .find(|(p, _)| p == path)
            .map(|(_, c)| c.as_str())
            .unwrap_or_else(|| {
                panic!(
                    "{path} not realized; have {:?}",
                    files.iter().map(|f| &f.0).collect::<Vec<_>>()
                )
            })
    }

    /// Every emitted file has a place: modules where the partition crates include them from, the
    /// entry at Cargo's binary path, generated tests in the top package's `tests/`, and a population
    /// manifest over the realized paths. The single-package crate root is superseded by the
    /// facade's, so `src/lib.rs` appears once.
    #[test]
    fn every_emitted_file_lands_at_its_workspace_path_and_tests_are_kept() {
        let outcome = realize(
            base_files(vec![text("tests/fx_a_test.rs", "#[test] fn t() {}\n")]),
            edges(&["fx_a"], vec![]),
            false,
        );
        let files = realized_files(&outcome);
        for path in [
            "src/v1/stage0/src/fx_a.rs",
            "src/v1/stage0/src/v1_rt.rs",
            "src/main.rs",
            "tests/fx_a_test.rs",
            "Cargo.toml",
            "src/emitted_population.rs",
        ] {
            content_of(&files, path);
        }
        assert_eq!(files.iter().filter(|(p, _)| p == "src/lib.rs").count(), 1);
        let population = content_of(&files, "src/emitted_population.rs");
        assert!(
            population.contains("tests/fx_a_test.rs")
                && population.contains("src/v1/stage0/src/fx_a.rs")
        );
        assert!(content_of(&files, "Cargo.toml").contains("[workspace]"));
    }

    #[test]
    fn an_emitted_file_with_no_workspace_place_refuses_by_path() {
        let outcome = realize(
            base_files(vec![text("assets/logo.svg", "<svg/>")]),
            edges(&["fx_a"], vec![]),
            false,
        );
        match &*outcome {
            EmittedWorkspaceRealization::EmittedWorkspaceRealizationRefused { cause } => {
                match &**cause {
                    EmittedWorkspaceRealizationRefusal::EmittedWorkspaceFileUnplaced { path } => {
                        assert_eq!(path.as_str(), "assets/logo.svg")
                    }
                    other => panic!("refused for the wrong reason: {other:?}"),
                }
            }
            EmittedWorkspaceRealization::EmittedWorkspaceRealizedOk { .. } => {
                panic!("an unplaceable file was dropped from a successful realization")
            }
        }
    }

    /// A source module named `main` is written as `main_mod.rs` beside the binary's `main.rs`, and
    /// its edges name it by that basename, so the plan finds every endpoint among the modules.
    #[test]
    fn a_module_named_main_realizes_under_its_emitted_basename() {
        let main_mod = rust_module_emit_filename("main".to_string());
        assert_eq!(main_mod, "main_mod");
        let outcome = realize(
            base_files(vec![text("src/main_mod.rs", "pub fn run() {}\n")]),
            edges(
                &["fx_a", "main_mod"],
                vec![declared_import_edge(main_mod.clone(), "fx_a".to_string())],
            ),
            false,
        );
        content_of(&realized_files(&outcome), "src/v1/stage0/src/main_mod.rs");
    }

    /// v1_rt gates on `text_lookup_work_counter`; the crate that owns it must declare the feature
    /// or `-D warnings` refuses every cfg site as unexpected (paired with test.claim
    /// .emitted_manifest_runtime_features emitted_runtime_gates_on_the_declared_feature).
    #[test]
    fn the_runtime_crate_manifest_declares_the_runtime_gated_feature() {
        let files = realized_files(&realize(
            base_files(vec![]),
            edges(&["fx_a"], vec![]),
            false,
        ));
        let manifest = content_of(&files, "src/v1/v2-emitted-runtime/Cargo.toml");
        assert!(manifest.contains("[features]"), "{manifest}");
        assert!(
            manifest.contains("text_lookup_work_counter = []"),
            "{manifest}"
        );
    }

    /// An emission that renders async services links reqwest over Rustls with the native-tls
    /// default switched off; one that renders none does not link reqwest at all.
    #[test]
    fn a_service_emission_links_reqwest_over_rustls_without_default_features() {
        let with = realized_files(&realize(base_files(vec![]), edges(&["fx_a"], vec![]), true));
        let top = content_of(&with, "Cargo.toml");
        assert!(
            top.contains("reqwest")
                && top.contains("rustls-tls")
                && top.contains("default-features = false"),
            "{top}"
        );
        let without = realized_files(&realize(
            base_files(vec![]),
            edges(&["fx_a"], vec![]),
            false,
        ));
        assert!(!content_of(&without, "Cargo.toml").contains("reqwest"));
    }

    fn inline_targets(content: &str) -> Vec<String> {
        let modules: im::HashMap<String, bool> = ["fx_a", "fx_b", "fx_c", "fx_d", "fx_e"]
            .iter()
            .map(|m| (m.to_string(), true))
            .collect();
        crate::v1_compiler_emit_rust::emitted_inline_path_edges(
            text("src/fx_a.rs", content),
            Rc::new(modules),
        )
        .iter()
        .filter(|e| matches!(e.provenance, EmittedEdgeProvenance::InlinePathReference))
        .filter_map(|e| match &*e.to {
            EmittedEdgeTarget::EmittedModuleTarget { module } => Some(module.to_string()),
            EmittedEdgeTarget::EmittedCrateRootTarget => None,
        })
        .collect()
    }

    /// A call by path is an edge; a comment, a string literal, a `use` line (returned with its own
    /// provenance) and the module's own path are not.
    #[test]
    fn a_call_by_path_is_an_edge_and_prose_strings_and_use_lines_are_not() {
        let targets = inline_targets(concat!(
            "use crate::fx_e::Thing;\n",
            "// calls crate::fx_c::nothing\n",
            "pub fn a() -> i64 { let s = \"crate::fx_d::f\"; crate::fx_b::b() + crate::fx_a::own() }\n"
        ));
        assert_eq!(targets, vec!["fx_b".to_string()]);
    }

    #[test]
    fn a_path_into_a_name_that_is_no_module_of_the_emission_is_not_an_edge() {
        assert!(
            inline_targets("pub fn a() { crate::NonEmptyVec::new(); crate::fx_zz::f(); }\n")
                .is_empty()
        );
    }
}
