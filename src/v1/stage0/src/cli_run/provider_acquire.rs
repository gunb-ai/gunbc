//! Generated-provider acquisition: the seed's one fixed-point loop around the model's decision.
//!
//! A compiler-generated declaration reference (an admitted conversion plan's route, a generated
//! collection head) demands its provider; the closure owes that declaration's module even though no
//! authored import names it (operator ruling N2b, 2026-10-15; option 1b, 2026-10-20). The model is
//! `.dag`: inference emits the demands (`v1.compiler.infer` `InferEmissions`), the pass groups them
//! once (`ReconciledPass` / `group_provider_demands`), `v1.compiler.infer`
//! `closure_provider_acquisition_step` observes them against the checked closure, and
//! `std.generated_provider` `provider_acquisition_step` decides acquire / complete / refuse. This file
//! only transports: it concatenates the per-module demand chunks, answers the one fact a closure
//! builder owns (which demanded modules its pool holds), and unpacks the decision. It never chooses a
//! provider, never rescans `conversion_plan_rows`, and never shrinks a closure.
//!
//! Seed-growth receipt: `gunbc.generated_provider_acquire_seed_growth`.

use crate::std_generated_provider::{
    group_provider_demands, GeneratedProviderDemand, ProviderAcquisitionStep,
};
use crate::v1_compiler_infer;
use crate::v1_compiler_infer_items::TypedModule;
use crate::v1_std_core::{ErrorNode, NewlineIndex};
use im::HashMap;
use std::rc::Rc;

/// One pass's demands, ungrouped per module or grouped per pass.
pub(crate) type ProviderDemands = Rc<im::Vector<Rc<GeneratedProviderDemand>>>;

/// Group a whole pass's demands once, in module order. Grouping per module would leave the same
/// declaration demanded twice across modules; grouping inside inference would make every
/// intermediate concatenation quadratic.
pub(crate) fn group_pass_demands(chunks: &[ProviderDemands]) -> ProviderDemands {
    let mut all = im::Vector::new();
    for chunk in chunks {
        all.extend(chunk.iter().cloned());
    }
    group_provider_demands(Rc::new(all))
}

/// What one checked closure owes, as the model decided it.
pub(crate) enum ClosureProviderStep {
    /// Every demanded declaration is in the closure: the closure is at its fixed point.
    Complete,
    /// These modules are outside the closure and in the pool; acquire them and check again.
    Acquire(Vec<String>),
    /// At least one demanded declaration is unavailable: one `GeneratedProviderUnavailable`
    /// diagnostic per demanding site, rendered by the caller like any other blocking diagnostic.
    Refused(Rc<im::Vector<Rc<ErrorNode>>>),
}

/// Judge a checked closure. `in_pool` is the builder's own storage answer for a module path; it is
/// asked only about demanded modules.
pub(crate) fn judge_closure(
    demands: &ProviderDemands,
    modules: Rc<im::Vector<Rc<TypedModule>>>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    in_pool: &dyn Fn(&str) -> bool,
) -> ClosureProviderStep {
    let pool_modules: im::Vector<String> = demands
        .iter()
        .map(|d| d.declaration.module_path.clone())
        .filter(|m| in_pool(m))
        .collect();
    let step = v1_compiler_infer::closure_provider_acquisition_step(
        demands.clone(),
        modules,
        source_indices,
        Rc::new(pool_modules),
    );
    match step.as_ref() {
        ProviderAcquisitionStep::AcquisitionComplete => ClosureProviderStep::Complete,
        ProviderAcquisitionStep::AcquireModules { modules } => {
            let mut out = vec![modules.head.clone()];
            out.extend(modules.tail.iter().cloned());
            ClosureProviderStep::Acquire(out)
        }
        ProviderAcquisitionStep::ProviderAcquisitionRefused { .. } => ClosureProviderStep::Refused(
            v1_compiler_infer::generated_provider_unavailable_diagnostics(step.clone()),
        ),
    }
}

/// THE SAME LOOP FOR THE `.dag` COMPILE PIPELINE (emit-check, the hermetic fixture). The builder
/// supplies its pool as two answers: whether it holds a module path, and that module's own closure.
/// A refusal is not a second channel: its `GeneratedProviderUnavailable` diagnostics join the
/// result's diagnostics, so each surface reports it exactly as it reports any blocking diagnostic.
/// A hermetic fixture's pool is empty (its manifest is its whole closure), so a demand outside the
/// manifest refuses there rather than reaching into the tree.
pub(crate) fn compile_to_provider_fixed_point(
    sources: Vec<Rc<crate::v1_compiler_compile::SourceFile>>,
    in_pool: &dyn Fn(&str) -> bool,
    closure_of: &dyn Fn(&str) -> Result<Vec<Rc<crate::v1_compiler_compile::SourceFile>>, String>,
) -> Result<Rc<crate::v1_compiler_compile::ResolvedPipelineResult>, String> {
    let mut sources = sources;
    loop {
        let resolved =
            crate::v1_compiler_compile::compile_to_resolved(Rc::new(sources.clone().into()));
        let graph = match resolved.graph.clone() {
            Some(graph) => graph,
            None => return Ok(resolved),
        };
        match judge_closure(
            &resolved.provider_demands,
            graph.modules.clone(),
            resolved.source_indices.clone(),
            in_pool,
        ) {
            ClosureProviderStep::Complete => return Ok(resolved),
            ClosureProviderStep::Refused(diagnostics) => {
                let mut all = (*resolved.diagnostics).clone();
                all.extend(diagnostics.iter().cloned());
                return Ok(Rc::new(
                    crate::v1_compiler_compile::ResolvedPipelineResult {
                        diagnostics: Rc::new(all),
                        ..(*resolved).clone()
                    },
                ));
            }
            ClosureProviderStep::Acquire(modules) => {
                sources = extend_with_acquired(sources, &modules, closure_of)?;
            }
        }
    }
}

/// ONE ACQUISITION STEP FOR EVERY BUILDER: each acquired module's own closure joins the sources,
/// deduplicated by path. Adding nothing is refused rather than looped on: the model only asks for
/// modules outside the closure, so an empty extension means the builder's pool and closure disagree.
pub(crate) fn extend_with_acquired(
    sources: Vec<Rc<crate::v1_compiler_compile::SourceFile>>,
    modules: &[String],
    closure_of: &dyn Fn(&str) -> Result<Vec<Rc<crate::v1_compiler_compile::SourceFile>>, String>,
) -> Result<Vec<Rc<crate::v1_compiler_compile::SourceFile>>, String> {
    let mut sources = sources;
    let before = sources.len();
    for module in modules {
        for s in closure_of(module)? {
            if !sources.iter().any(|x| x.path == s.path) {
                sources.push(s);
            }
        }
    }
    if sources.len() == before {
        return Err(format!(
            "generated provider acquisition made no progress: acquiring {modules:?} added no \
             source to the closure"
        ));
    }
    Ok(sources)
}

/// SUPPLIED DEMANDS THROUGH EACH REAL ADAPTER (operator ruling on PR 1). No producer emits a demand
/// yet, so each test supplies one at this file's interface -- `judge_closure` and
/// `extend_with_acquired` -- over a closure the surface really compiled and the pool it really
/// answers from. The inhabitance claim that a real producer emits this shape lands with the
/// host-text crossing that produces it.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::std_decl_ref::{DeclField, DeclarationRef};
    use crate::std_generated_provider::{GeneratedProviderCause, GeneratedProviderSite};
    use crate::v1_std_core::CompilerDiagnostic;

    fn supplied(module_path: &str, decl_name: &str) -> ProviderDemands {
        let site = Rc::new(GeneratedProviderSite {
            cause: GeneratedProviderCause::GeneratedConversionRoute,
            at: crate::v1_std_core::no_span(),
            occurrence: None,
        });
        let demand = crate::std_generated_provider::generated_provider_demand(
            Rc::new(DeclarationRef {
                module_path: module_path.to_string(),
                decl_name: decl_name.to_string(),
                field: Rc::new(DeclField::WholeDeclaration),
            }),
            site,
        );
        Rc::new(im::vector![demand])
    }

    fn refused_classes(step: &ClosureProviderStep) -> Vec<String> {
        match step {
            ClosureProviderStep::Refused(diagnostics) => diagnostics
                .iter()
                .map(|d| match d.diagnostic.as_ref() {
                    CompilerDiagnostic::GeneratedProviderUnavailable { cause, .. } => {
                        format!("GeneratedProviderUnavailable/{cause:?}")
                    }
                    other => format!("{other:?}"),
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    /// Emit-check surface: the witness-root module index is the pool. A module whose closure
    /// lacks std.coercion acquires exactly that module for a supplied route demand, then reaches
    /// the fixed point; a declaration the module does not have refuses as DeclarationAbsent.
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn emit_check_pool_acquires_the_demanded_module_then_reaches_the_fixed_point() {
        let pool = super::super::build_module_path_index_from_witness_roots();
        let fixture = "module test.fixture.generated_provider_carrier_only\n\nimport std.algebra { FreeMonoid }\nimport std.types { Char }\n\nfn keep(xs: FreeMonoid<Char>) -> FreeMonoid<Char> { xs }\n";
        let sources = super::super::resolve_virtual_source_with_imports(
            super::super::emit_host::FIXTURE_SOURCE_PATH,
            fixture,
            &pool,
        )
        .expect("fixture closure");
        let resolved =
            crate::v1_compiler_compile::compile_to_resolved(Rc::new(sources.clone().into()));
        let graph = resolved.graph.clone().expect("fixture compiles");
        // Carrier present, no crossing: no demand, and std.coercion is not in the closure.
        assert!(
            resolved.provider_demands.is_empty(),
            "a FreeMonoid<Char> carrier demanded a provider"
        );
        assert!(!sources
            .iter()
            .any(|s| s.path.ends_with("dag/std/coercion.dag")));
        let demand = supplied("std.coercion", "unicode_scalar_unfold");
        let step = judge_closure(
            &demand,
            graph.modules.clone(),
            resolved.source_indices.clone(),
            &|m: &str| pool.contains_key(m),
        );
        let modules = match step {
            ClosureProviderStep::Acquire(m) => m,
            other => panic!(
                "expected Acquire, got refused={:?}",
                refused_classes(&other)
            ),
        };
        assert_eq!(modules, vec!["std.coercion".to_string()]);
        let extended = extend_with_acquired(sources, &modules, &|m: &str| {
            super::super::emit_host::fixture_pool_closure_of(m, &pool)
        })
        .expect("acquisition extends the closure");
        let again = crate::v1_compiler_compile::compile_to_resolved(Rc::new(extended.into()));
        let graph = again.graph.clone().expect("acquired closure compiles");
        assert!(matches!(
            judge_closure(
                &demand,
                graph.modules.clone(),
                again.source_indices.clone(),
                &|m: &str| pool.contains_key(m)
            ),
            ClosureProviderStep::Complete
        ));
        let missing = supplied("std.coercion", "no_such_declaration");
        assert_eq!(
            refused_classes(&judge_closure(
                &missing,
                graph.modules.clone(),
                again.source_indices.clone(),
                &|m: &str| pool.contains_key(m)
            )),
            vec!["GeneratedProviderUnavailable/UnavailableDeclarationAbsent".to_string()]
        );
    }

    /// Hermetic fixture surface: the pool is empty, so a demand outside the manifest refuses as
    /// ModuleAbsent instead of reaching the tree.
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn hermetic_fixture_refuses_a_provider_outside_its_manifest() {
        let files = vec![Rc::new(crate::v1_compiler_compile::SourceFile {
            path: "hermetic.dag".to_string(),
            content: "module test.fixture.hermetic_provider\n\nfn one() -> Int { 1 }\n".to_string(),
        })];
        let resolved = crate::v1_compiler_compile::compile_to_resolved(Rc::new(files.into()));
        let graph = resolved.graph.clone().expect("hermetic fixture compiles");
        assert_eq!(
            refused_classes(&judge_closure(
                &supplied("std.coercion", "unicode_scalar_unfold"),
                graph.modules.clone(),
                resolved.source_indices.clone(),
                &|_: &str| false
            )),
            vec!["GeneratedProviderUnavailable/UnavailableModuleAbsent".to_string()]
        );
    }

    /// Entry-resolve surface (claims, claim_batch, whole-tree compile, demand-acquired run): the
    /// index is the pool and an acquired module joins with its own entry closure.
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn entry_resolve_pool_acquires_the_demanded_module() {
        let index = super::super::entry_resolve::try_process_shared_index(
            &super::super::default_source_roots(),
        )
        .expect("shared index");
        let entry_path = super::super::process_workspace_root().join("dag/std/optional.dag");
        let entry = entry_path.to_str().expect("utf-8 path");
        let sources = super::super::entry_resolve::load_sources_for_entry_with_pool(&index, entry)
            .expect("entry closure");
        assert!(!sources
            .iter()
            .any(|s| s.path.ends_with("dag/std/coercion.dag")));
        let (graph, si, _) = super::super::entry_resolve::resolved_graph_from_sources_with_index(
            &index,
            sources.clone(),
            entry,
            super::super::ResolvedGraphMemoShare::Ephemeral,
        )
        .expect("entry resolves");
        let demand = supplied("std.coercion", "unicode_scalar_unfold");
        let modules = match judge_closure(&demand, graph.modules.clone(), si.clone(), &|m: &str| {
            index.source_files.contains_key(m)
        }) {
            ClosureProviderStep::Acquire(m) => m,
            other => panic!(
                "expected Acquire, got refused={:?}",
                refused_classes(&other)
            ),
        };
        assert_eq!(modules, vec!["std.coercion".to_string()]);
        let extended = extend_with_acquired(sources, &modules, &|m: &str| {
            let provider = index.source_files.get(m).cloned().expect("pool answered");
            let on_disk = super::super::process_workspace_root().join(&provider.path);
            super::super::entry_resolve::load_sources_for_entry_with_pool(
                &index,
                &on_disk.to_string_lossy(),
            )
        })
        .expect("acquisition extends the closure");
        assert!(extended
            .iter()
            .any(|s| s.path.ends_with("dag/std/coercion.dag")));
        // A second acquisition of the same module adds nothing and is refused, never looped on.
        assert!(
            extend_with_acquired(extended.clone(), &modules, &|m: &str| {
                let provider = index.source_files.get(m).cloned().expect("pool answered");
                super::super::entry_resolve::load_sources_for_entry_with_pool(
                    &index,
                    &provider.path,
                )
            })
            .is_err()
        );
    }

    /// Provider already in the closure: the fixed point holds on the first pass and nothing is
    /// acquired. A chained demand on the next round acquires only the newly missing module.
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn a_present_provider_acquires_nothing_and_a_chained_demand_acquires_only_its_own_module() {
        let pool = super::super::build_module_path_index_from_witness_roots();
        let fixture = "module test.fixture.generated_provider_present\n\nimport std.coercion { unicode_scalar_unfold }\n\nfn one() -> Int { 1 }\n";
        let sources = super::super::resolve_virtual_source_with_imports(
            super::super::emit_host::FIXTURE_SOURCE_PATH,
            fixture,
            &pool,
        )
        .expect("fixture closure");
        let resolved =
            crate::v1_compiler_compile::compile_to_resolved(Rc::new(sources.clone().into()));
        let graph = resolved.graph.clone().expect("fixture compiles");
        let in_pool = |m: &str| pool.contains_key(m);
        assert!(matches!(
            judge_closure(
                &supplied("std.coercion", "unicode_scalar_unfold"),
                graph.modules.clone(),
                resolved.source_indices.clone(),
                &in_pool
            ),
            ClosureProviderStep::Complete
        ));
        let mut chained = (*supplied("std.coercion", "unicode_scalar_unfold")).clone();
        chained.extend(supplied("std.graph", "CallGraph").iter().cloned());
        match judge_closure(
            &Rc::new(chained),
            graph.modules.clone(),
            resolved.source_indices.clone(),
            &in_pool,
        ) {
            ClosureProviderStep::Acquire(modules) => {
                assert_eq!(modules, vec!["std.graph".to_string()])
            }
            other => panic!(
                "expected Acquire, got refused={:?}",
                refused_classes(&other)
            ),
        }
    }
}
