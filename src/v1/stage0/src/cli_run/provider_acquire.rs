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
    group_provider_demands, stalled_acquisition_step, GeneratedProviderDemand,
    ProviderAcquisitionStep, ProviderModuleFact, ProviderPoolObservation,
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

/// WHAT A BUILDER'S POOL CAN ANSWER for one demanded module outside the closure. There is no
/// ambiguous arm, and that is an admitted fact of both pools rather than an omission: the entry
/// index refuses two files for one module path when it is built (`module_source_identity`), and
/// the witness-root module index panics on the same collision (`insert_module_path`), so neither
/// pool can hold several candidates by the time a demand reaches it.
pub(crate) enum PoolAnswer {
    Holds,
    Lacks,
    /// The pool names the module and its source cannot be read.
    Unreadable,
}

fn pool_observation(module_path: &str, answer: PoolAnswer) -> Rc<ProviderPoolObservation> {
    Rc::new(ProviderPoolObservation {
        module_path: module_path.to_string(),
        fact: Rc::new(match answer {
            PoolAnswer::Holds => ProviderModuleFact::ModuleInPool,
            PoolAnswer::Lacks => ProviderModuleFact::ModuleNotInPool,
            PoolAnswer::Unreadable => ProviderModuleFact::ModuleUnreadableInPool,
        }),
    })
}

/// Judge a checked closure. The builder answers for every demanded module path (the model reads
/// the closure fact first, so an answer for a module already in the closure is never used);
/// `diagnostics` are the closure's own, from which the model reads refused declarations.
pub(crate) fn judge_closure(
    demands: &ProviderDemands,
    modules: Rc<im::Vector<Rc<TypedModule>>>,
    diagnostics: Rc<im::Vector<Rc<ErrorNode>>>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    pool: &dyn Fn(&str) -> PoolAnswer,
) -> ClosureProviderStep {
    let mut answered: Vec<String> = Vec::new();
    let mut observations: im::Vector<Rc<ProviderPoolObservation>> = im::Vector::new();
    for d in demands.iter() {
        let module_path = d.declaration.module_path.clone();
        if !answered.contains(&module_path) {
            observations.push_back(pool_observation(&module_path, pool(&module_path)));
            answered.push(module_path);
        }
    }
    let step = v1_compiler_infer::closure_provider_acquisition_step(
        demands.clone(),
        modules,
        diagnostics,
        source_indices,
        Rc::new(observations),
    );
    step_outcome(step)
}

/// The model's refusal for an acquisition whose pool closures added nothing.
pub(crate) fn stalled(demands: &ProviderDemands, modules: &[String]) -> ClosureProviderStep {
    step_outcome(stalled_acquisition_step(
        demands.clone(),
        Rc::new(modules.iter().cloned().collect()),
    ))
}

fn step_outcome(step: Rc<ProviderAcquisitionStep>) -> ClosureProviderStep {
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
    pool: &dyn Fn(&str) -> PoolAnswer,
    closure_of: &dyn Fn(
        &str,
    )
        -> Result<Vec<Rc<crate::v1_compiler_compile::SourceFile>>, ClosureOfError>,
) -> Result<Rc<crate::v1_compiler_compile::ResolvedPipelineResult>, String> {
    let mut sources = sources;
    loop {
        let resolved =
            crate::v1_compiler_compile::compile_to_resolved(Rc::new(sources.clone().into()));
        let graph = match resolved.graph.clone() {
            Some(graph) => graph,
            None => return Ok(resolved),
        };
        let refused = |diagnostics: Rc<im::Vector<Rc<ErrorNode>>>| {
            let mut all = (*resolved.diagnostics).clone();
            all.extend(diagnostics.iter().cloned());
            Rc::new(crate::v1_compiler_compile::ResolvedPipelineResult {
                diagnostics: Rc::new(all),
                ..(*resolved).clone()
            })
        };
        match judge_closure(
            &resolved.provider_demands,
            graph.modules.clone(),
            resolved.diagnostics.clone(),
            resolved.source_indices.clone(),
            pool,
        ) {
            ClosureProviderStep::Complete => return Ok(resolved),
            ClosureProviderStep::Refused(diagnostics) => return Ok(refused(diagnostics)),
            ClosureProviderStep::Acquire(modules) => {
                let typed_refusal =
                    match extend_with_acquired(sources.clone(), &modules, closure_of)? {
                        Extension::Extended(extended) => {
                            sources = extended;
                            continue;
                        }
                        Extension::Stalled => stalled(&resolved.provider_demands, &modules),
                        // Emit-check and the hermetic fixture hold their whole pool, so their closure
                        // step never reports a provider outside their index.
                        Extension::OutsideIndex(outside) => {
                            return Err(format!(
                                "generated providers {outside:?} are outside a pool this builder holds whole"
                            ))
                        }
                        Extension::Unreadable(failed) => unreadable(
                            &resolved.provider_demands,
                            graph.modules.clone(),
                            resolved.diagnostics.clone(),
                            resolved.source_indices.clone(),
                            &failed,
                            pool,
                        ),
                    };
                match typed_refusal {
                    ClosureProviderStep::Refused(diagnostics) => return Ok(refused(diagnostics)),
                    _ => unreachable!("a stalled or unreadable acquisition always refuses"),
                }
            }
        }
    }
}

/// WHY A PROVIDER'S CLOSURE COULD NOT JOIN. A read failure is the model's ModuleUnreadable, decided by
/// re-judging the closure with that module answered Unreadable; anything else is an ordinary closure
/// failure (the provider's own imports did not resolve), loud and located like any other.
#[derive(Debug)]
pub(crate) enum ClosureOfError {
    Unreadable,
    /// The pool holds the module but this index's acquired closure does not (the demand-acquired
    /// path): the builder rebuilds its closure with it rather than reading it here.
    OutsideIndex,
    Closure(String),
}

/// What one acquisition step produced.
pub(crate) enum Extension {
    Extended(Vec<Rc<crate::v1_compiler_compile::SourceFile>>),
    /// The pools' closures added nothing: the model's AcquisitionMadeNoProgress ([`stalled`]).
    Stalled,
    /// This module's source could not be read: the model's ModuleUnreadable ([`unreadable`]).
    Unreadable(String),
    /// These modules are in the pool but outside the builder's index; the builder rebuilds.
    OutsideIndex(Vec<String>),
}

/// ONE ACQUISITION STEP FOR EVERY BUILDER: each acquired module's own closure joins the sources,
/// deduplicated by path. Neither an empty extension nor a read failure is decided here: both are
/// returned typed and refused through the model, never looped on and never rendered as prose.
pub(crate) fn extend_with_acquired(
    sources: Vec<Rc<crate::v1_compiler_compile::SourceFile>>,
    modules: &[String],
    closure_of: &dyn Fn(
        &str,
    )
        -> Result<Vec<Rc<crate::v1_compiler_compile::SourceFile>>, ClosureOfError>,
) -> Result<Extension, String> {
    let mut sources = sources;
    let before = sources.len();
    let mut outside: Vec<String> = Vec::new();
    for module in modules {
        let closure = match closure_of(module) {
            Ok(closure) => closure,
            Err(ClosureOfError::Unreadable) => return Ok(Extension::Unreadable(module.clone())),
            Err(ClosureOfError::OutsideIndex) => {
                outside.push(module.clone());
                continue;
            }
            Err(ClosureOfError::Closure(cause)) => return Err(cause),
        };
        for s in closure {
            if !sources.iter().any(|x| x.path == s.path) {
                sources.push(s);
            }
        }
    }
    if !outside.is_empty() {
        return Ok(Extension::OutsideIndex(outside));
    }
    Ok(if sources.len() == before {
        Extension::Stalled
    } else {
        Extension::Extended(sources)
    })
}

/// The model's refusal for a provider whose source the pool named but could not read: the same
/// closure judged again through the builder's own pool, with only the failed module's answer replaced
/// by Unreadable. Every other demanded module keeps the answer its pool gives -- a readable provider
/// beside the failed one is never reported absent.
pub(crate) fn unreadable(
    demands: &ProviderDemands,
    modules: Rc<im::Vector<Rc<TypedModule>>>,
    diagnostics: Rc<im::Vector<Rc<ErrorNode>>>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    failed: &str,
    pool: &dyn Fn(&str) -> PoolAnswer,
) -> ClosureProviderStep {
    judge_closure(demands, modules, diagnostics, source_indices, &|m: &str| {
        if m == failed {
            PoolAnswer::Unreadable
        } else {
            pool(m)
        }
    })
}

/// SUPPLIED DEMANDS THROUGH EACH REAL ADAPTER (operator ruling on PR 1). No producer emits a demand
/// yet, so each test supplies one at this file's interface -- `judge_closure`, `stalled` and
/// `extend_with_acquired` -- over a closure the surface really compiled and the pool it really
/// answers from. The inhabitance claim that a real producer emits this shape lands with the
/// host-text crossing that produces it (PR 2).
#[cfg(test)]
mod tests {
    use super::*;
    use crate::std_decl_ref::{DeclField, DeclarationRef};
    use crate::std_generated_provider::{GeneratedProviderCause, GeneratedProviderSite};
    use crate::v1_compiler_compile::{ResolvedPipelineResult, SourceFile};
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

    fn extended_sources(e: Extension) -> Vec<Rc<SourceFile>> {
        match e {
            Extension::Extended(sources) => sources,
            Extension::Stalled => panic!("expected an extension, got Stalled"),
            Extension::Unreadable(m) => panic!("expected an extension, got Unreadable({m})"),
            Extension::OutsideIndex(m) => panic!("expected an extension, got OutsideIndex({m:?})"),
        }
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

    fn judge(
        demand: &ProviderDemands,
        resolved: &ResolvedPipelineResult,
        pool: &dyn Fn(&str) -> PoolAnswer,
    ) -> ClosureProviderStep {
        let graph = resolved.graph.clone().expect("the closure compiles");
        judge_closure(
            demand,
            graph.modules.clone(),
            resolved.diagnostics.clone(),
            resolved.source_indices.clone(),
            pool,
        )
    }

    fn compile(sources: &[Rc<SourceFile>]) -> Rc<ResolvedPipelineResult> {
        crate::v1_compiler_compile::compile_to_resolved(Rc::new(sources.to_vec().into()))
    }

    fn emit_check_sources(fixture: &str, pool: &HashMap<String, String>) -> Vec<Rc<SourceFile>> {
        super::super::resolve_virtual_source_with_imports(
            super::super::emit_host::FIXTURE_SOURCE_PATH,
            fixture,
            pool,
        )
        .expect("fixture closure")
    }

    /// Emit-check surface: the witness-root module index is the pool. A carrier-only closure has no
    /// demand and no std.coercion; a supplied route demand acquires exactly that module, then
    /// reaches the fixed point; a declaration the module does not have refuses as DeclarationAbsent.
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn emit_check_pool_acquires_the_demanded_module_then_reaches_the_fixed_point() {
        let pool = super::super::build_module_path_index_from_witness_roots();
        let answer = |m: &str| super::super::emit_host::fixture_pool_answer(m, &pool);
        let sources = emit_check_sources("module test.fixture.generated_provider_carrier_only\n\nimport std.algebra { FreeMonoid }\nimport std.types { Char }\n\nfn keep(xs: FreeMonoid<Char>) -> FreeMonoid<Char> { xs }\n", &pool);
        let resolved = compile(&sources);
        assert!(
            resolved.provider_demands.is_empty(),
            "a FreeMonoid<Char> carrier demanded a provider"
        );
        assert!(!sources
            .iter()
            .any(|s| s.path.ends_with("dag/std/coercion.dag")));
        let demand = supplied("std.coercion", "unicode_scalar_unfold");
        let modules = match judge(&demand, &resolved, &answer) {
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
        .expect("the provider's closure loads");
        let again = compile(&extended_sources(extended));
        assert!(matches!(
            judge(&demand, &again, &answer),
            ClosureProviderStep::Complete
        ));
        assert_eq!(
            refused_classes(&judge(
                &supplied("std.coercion", "no_such_declaration"),
                &again,
                &answer
            )),
            vec!["GeneratedProviderUnavailable/UnavailableDeclarationAbsent".to_string()]
        );
    }

    /// The emit-check pool's typed answers: a module the index names at a file that cannot be read
    /// refuses as ModuleUnreadable, never as absent and never by acquiring.
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn emit_check_pool_reports_an_unreadable_provider_as_unreadable() {
        let mut pool = super::super::build_module_path_index_from_witness_roots();
        pool.insert(
            "test.fixture.generated_provider_unreadable".to_string(),
            "dag/test/fixture/generated_provider_unreadable_does_not_exist.dag".to_string(),
        );
        let resolved = compile(&emit_check_sources(
            "module test.fixture.generated_provider_plain\n\nfn one() -> Int { 1 }\n",
            &pool,
        ));
        assert_eq!(
            refused_classes(&judge(
                &supplied("test.fixture.generated_provider_unreadable", "anything"),
                &resolved,
                &|m: &str| super::super::emit_host::fixture_pool_answer(m, &pool)
            )),
            vec!["GeneratedProviderUnavailable/UnavailableModuleUnreadable".to_string()]
        );
    }

    /// Hermetic fixture surface: the pool is empty, so a demand outside the manifest refuses as
    /// ModuleAbsent; a demanded declaration that is in the manifest but refused (a blocking
    /// diagnostic located inside it) refuses as DeclarationRefused, read from the closure's own
    /// diagnostics.
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn hermetic_fixture_refuses_absent_and_refused_providers_by_their_own_causes() {
        let lacks = |_: &str| PoolAnswer::Lacks;
        let files = vec![Rc::new(SourceFile {
            path: "hermetic.dag".to_string(),
            content: "module test.fixture.hermetic_provider\n\nfn one() -> Int { 1 }\n\nfn broken() -> Int { \"not an int\" }\n".to_string(),
        })];
        let resolved = compile(&files);
        assert_eq!(
            refused_classes(&judge(
                &supplied("std.coercion", "unicode_scalar_unfold"),
                &resolved,
                &lacks
            )),
            vec!["GeneratedProviderUnavailable/UnavailableModuleAbsent".to_string()]
        );
        assert_eq!(
            refused_classes(&judge(
                &supplied("test.fixture.hermetic_provider", "broken"),
                &resolved,
                &lacks
            )),
            vec!["GeneratedProviderUnavailable/UnavailableDeclarationRefused".to_string()]
        );
        assert!(matches!(
            judge(
                &supplied("test.fixture.hermetic_provider", "one"),
                &resolved,
                &lacks
            ),
            ClosureProviderStep::Complete
        ));
    }

    /// Entry-resolve surface (claims, claim_batch, whole-tree compile, demand-acquired run): the
    /// index is the pool and an acquired module joins with its own entry closure; an acquisition
    /// that adds nothing is the model's AcquisitionMadeNoProgress, not a loop and not prose.
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn entry_resolve_pool_acquires_the_demanded_module_and_a_stalled_extension_refuses_typed() {
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
        let holds = |m: &str| {
            if index.source_files.contains_key(m) {
                PoolAnswer::Holds
            } else {
                PoolAnswer::Lacks
            }
        };
        let modules = match judge_closure(
            &demand,
            graph.modules.clone(),
            graph.diagnostics.clone(),
            si.clone(),
            &holds,
        ) {
            ClosureProviderStep::Acquire(m) => m,
            other => panic!(
                "expected Acquire, got refused={:?}",
                refused_classes(&other)
            ),
        };
        assert_eq!(modules, vec!["std.coercion".to_string()]);
        let closure_of = |m: &str| {
            let provider = index.source_files.get(m).cloned().expect("pool answered");
            let walked = super::super::resolve_transitively(
                vec![provider],
                &index.source_files,
                &index.module_graph_facts,
            )
            .map_err(ClosureOfError::Closure)?;
            super::super::extend_sources_to_both_closure_fixpoint(walked, &index)
                .map_err(ClosureOfError::Closure)
        };
        let extended = extended_sources(
            extend_with_acquired(sources, &modules, &closure_of)
                .expect("the provider's closure loads"),
        );
        assert!(extended
            .iter()
            .any(|s| s.path.ends_with("dag/std/coercion.dag")));
        assert!(matches!(
            extend_with_acquired(extended, &modules, &closure_of)
                .expect("the provider's closure loads"),
            Extension::Stalled
        ));
        assert_eq!(
            refused_classes(&stalled(&demand, &modules)),
            vec!["GeneratedProviderUnavailable/AcquisitionMadeNoProgress".to_string()]
        );
    }

    /// Provider already in the closure: the fixed point holds on the first pass and nothing is
    /// acquired. A chained demand on the next round acquires only the newly missing module.
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn a_present_provider_acquires_nothing_and_a_chained_demand_acquires_only_its_own_module() {
        let pool = super::super::build_module_path_index_from_witness_roots();
        let answer = |m: &str| super::super::emit_host::fixture_pool_answer(m, &pool);
        let resolved = compile(&emit_check_sources("module test.fixture.generated_provider_present\n\nimport std.coercion { unicode_scalar_unfold }\n\nfn one() -> Int { 1 }\n", &pool));
        assert!(matches!(
            judge(
                &supplied("std.coercion", "unicode_scalar_unfold"),
                &resolved,
                &answer
            ),
            ClosureProviderStep::Complete
        ));
        let mut chained = (*supplied("std.coercion", "unicode_scalar_unfold")).clone();
        chained.extend(supplied("std.graph", "CallGraph").iter().cloned());
        match judge(&Rc::new(chained), &resolved, &answer) {
            ClosureProviderStep::Acquire(modules) => {
                assert_eq!(modules, vec!["std.graph".to_string()])
            }
            other => panic!(
                "expected Acquire, got refused={:?}",
                refused_classes(&other)
            ),
        }
    }

    /// STRUCTURAL CENSUS OF THE EMISSIONS CARRIER (review finding 2, the structural half; PR 2's
    /// crossing is the nonempty-transport half). Over the parsed `src/v1/04_infer.dag`:
    /// an `InferEmissions` record literal appears only inside its authority functions, and a
    /// `.diagnostics` / `.provider_demands` read off an emissions value appears only there and at
    /// `typecheck_module`'s stage-boundary finalization. Construction is identified exactly, by the
    /// literal's type name. Projection is identified SYNTACTICALLY -- a field read whose base is a
    /// `.emissions` field or a local whose name ends in `emissions` -- so a projection through a
    /// differently-named local escapes it; the next rung is the same reader over the typed graph,
    /// keyed on the base's inferred type. Runs in the required unit run: it parses one file.
    #[test]
    fn emissions_are_built_and_projected_only_at_their_named_sites() {
        use crate::v1_std_core::{ExprData, Node};
        const AUTHORITY: &[&str] = &[
            "infer_emissions_empty",
            "infer_emissions_of_diagnostics",
            "infer_emissions_of_provider_demand",
            "infer_emissions_concat",
            "infer_emissions_filter_diagnostics",
            "infer_emissions_concat_all",
        ];
        const FINALIZATION: &[&str] = &["typecheck_module"];
        let path = "src/v1/04_infer.dag";
        let content = std::fs::read_to_string(super::super::process_workspace_root().join(path))
            .expect("04_infer.dag is readable");
        let artifact = super::super::pool_acquire::artifact_for(path, &content);
        let nl = super::super::pool_acquire::newline_index_for(path, &content);
        let mut si = HashMap::new();
        si.insert(path.to_string(), nl);
        let si = Rc::new(si);
        let parsed = crate::v1_compiler_parse::parse_with_table(
            artifact.tokens.clone(),
            si.clone(),
            crate::v1_std_core::empty_intern_table(),
        );
        let module = parsed.result.module.clone().expect("04_infer.dag parses");

        fn walk(n: &Rc<Node>, f: &mut dyn FnMut(&Rc<Node>)) {
            f(n);
            for c in n.children.iter() {
                walk(c, f);
            }
            if let Some(b) = &n.body {
                walk(b, f);
            }
        }
        let mut constructions: Vec<String> = Vec::new();
        let mut projections: Vec<String> = Vec::new();
        let mut authority_seen: Vec<String> = Vec::new();
        for item in crate::v1_std_core::module_items(module).iter() {
            let owner = crate::v1_std_core::authored_name_at(si.clone(), item.clone());
            if AUTHORITY.contains(&owner.as_str()) {
                authority_seen.push(owner.clone());
            }
            walk(item, &mut |n| match n.expr_data.as_ref() {
                ExprData::ExprRecordLit { .. } => {
                    if crate::v1_std_core::record_lit_type_name_at(n.clone(), si.clone()).as_deref()
                        == Some("InferEmissions")
                        && !AUTHORITY.contains(&owner.as_str())
                    {
                        constructions.push(owner.clone());
                    }
                }
                ExprData::ExprFieldAccess { .. } => {
                    let field = crate::v1_std_core::authored_name_at(si.clone(), n.clone());
                    if field != "diagnostics" && field != "provider_demands" {
                        return;
                    }
                    let Some(base) = n.children.get(0) else {
                        return;
                    };
                    let base_name = crate::v1_std_core::authored_name_at(si.clone(), base.clone());
                    let on_emissions = match base.expr_data.as_ref() {
                        ExprData::ExprFieldAccess { .. } => base_name == "emissions",
                        ExprData::ExprVar { .. } => base_name.ends_with("emissions"),
                        _ => false,
                    };
                    if on_emissions
                        && !AUTHORITY.contains(&owner.as_str())
                        && !FINALIZATION.contains(&owner.as_str())
                    {
                        projections.push(format!("{owner}: {base_name}.{field}"));
                    }
                }
                _ => {}
            });
        }
        // The census reads what it claims to read: every authority function was found.
        authority_seen.sort();
        let mut expected: Vec<String> = AUTHORITY.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(
            authority_seen, expected,
            "the census did not find every authority function"
        );
        assert!(
            constructions.is_empty(),
            "InferEmissions built outside its authority: {constructions:?}"
        );
        assert!(
            projections.is_empty(),
            "emissions projected outside a named site: {projections:?}"
        );
    }

    /// A provider whose ACTUAL read fails during acquisition, after its pool answered Holds, refuses
    /// as the model's ModuleUnreadable at its own demanding site -- through the real emit-check pool
    /// answer and the real emit-check read, not an injected error -- and a readable provider demanded
    /// beside it is not reported absent. The provider file is a regular file throughout; its bytes
    /// become invalid UTF-8 between the observation and the acquisition read.
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn a_read_failure_during_acquisition_refuses_as_module_unreadable() {
        use crate::std_types::SourceSpan;
        let rel_dir = format!(
            "target/generated_provider_read_probe_{}",
            std::process::id()
        );
        let rel_file = format!("{rel_dir}/read_probe_provider.dag");
        let ws = super::super::process_workspace_root();
        std::fs::create_dir_all(ws.join(&rel_dir)).expect("probe dir");
        std::fs::write(
            ws.join(&rel_file),
            "module test.fixture.read_probe_provider\n\nfn probe_route(s: String) -> Int { 0 }\n",
        )
        .expect("probe provider");
        let mut pool = super::super::build_module_path_index_from_witness_roots();
        pool.insert(
            "test.fixture.read_probe_provider".to_string(),
            rel_file.clone(),
        );
        let answer = |m: &str| super::super::emit_host::fixture_pool_answer(m, &pool);
        assert!(matches!(
            answer("test.fixture.read_probe_provider"),
            PoolAnswer::Holds
        ));
        let site = |start: i64| {
            Rc::new(GeneratedProviderSite {
                cause: GeneratedProviderCause::GeneratedConversionRoute,
                at: Rc::new(SourceSpan {
                    file: "test.dag".to_string(),
                    start,
                    end: start + 5,
                }),
                occurrence: None,
            })
        };
        let demand = |module: &str, decl: &str, start: i64| {
            crate::std_generated_provider::generated_provider_demand(
                Rc::new(DeclarationRef {
                    module_path: module.to_string(),
                    decl_name: decl.to_string(),
                    field: Rc::new(DeclField::WholeDeclaration),
                }),
                site(start),
            )
        };
        let demands: ProviderDemands = Rc::new(im::vector![
            demand("test.fixture.read_probe_provider", "probe_route", 10),
            demand("std.graph", "CallGraph", 30)
        ]);
        let sources = emit_check_sources(
            "module test.fixture.read_probe_consumer\n\nfn one() -> Int { 1 }\n",
            &pool,
        );
        let resolved = compile(&sources);
        let graph = resolved.graph.clone().expect("the consumer compiles");
        let modules = match judge(&demands, &resolved, &answer) {
            ClosureProviderStep::Acquire(m) => m,
            other => panic!(
                "expected Acquire, got refused={:?}",
                refused_classes(&other)
            ),
        };
        assert_eq!(
            modules,
            vec![
                "test.fixture.read_probe_provider".to_string(),
                "std.graph".to_string()
            ]
        );
        // The model judged the pool's Holds; the acquisition read that follows will not succeed.
        std::fs::write(ws.join(&rel_file), [0xffu8, 0xfe, 0xfd]).expect("corrupt the provider");
        let failed = match extend_with_acquired(sources, &modules, &|m: &str| {
            super::super::emit_host::fixture_pool_closure_of(m, &pool)
        })
        .expect("a read failure is not a closure error")
        {
            Extension::Unreadable(failed) => failed,
            Extension::Extended(_) => panic!("the corrupted provider joined the closure"),
            Extension::Stalled => panic!("expected Unreadable, got Stalled"),
            Extension::OutsideIndex(m) => panic!("expected Unreadable, got OutsideIndex({m:?})"),
        };
        assert_eq!(failed, "test.fixture.read_probe_provider");
        let step = unreadable(
            &demands,
            graph.modules.clone(),
            resolved.diagnostics.clone(),
            resolved.source_indices.clone(),
            &failed,
            &answer,
        );
        let refused: Vec<String> = match &step {
            ClosureProviderStep::Refused(diagnostics) => diagnostics
                .iter()
                .map(|d| match d.diagnostic.as_ref() {
                    CompilerDiagnostic::GeneratedProviderUnavailable {
                        declaration,
                        cause,
                        span,
                    } => format!(
                        "{}.{} {cause:?} {}:{}..{}",
                        declaration.module_path,
                        declaration.decl_name,
                        span.file,
                        span.start,
                        span.end
                    ),
                    other => format!("{other:?}"),
                })
                .collect(),
            _ => Vec::new(),
        };
        let _ = std::fs::remove_dir_all(ws.join(&rel_dir));
        assert_eq!(
            refused,
            vec!["test.fixture.read_probe_provider.probe_route UnavailableModuleUnreadable test.dag:10..15".to_string()]
        );
    }

    /// COST RECEIPT, EXACT DEMAND VERSUS CARRIER PRESENCE (option 1b ruling; deferred to PR 2, where a
    /// producer exists). Over the entries listed in GUNBC_COST_ENTRIES (one workspace-relative .dag
    /// path per line), each entry's import closure is compiled through the pipeline and classified:
    /// whether its pass DEMANDS std.coercion.unicode_scalar_unfold (the exact rule) and whether its
    /// closure merely CONTAINS std.algebra, the FreeMonoid carrier's declaring module (the rejected
    /// carrier rule). Printed, never asserted: the numbers are the receipt; the ruling, not the count,
    /// rejected the carrier rule. Also printed: how many modules std.coercion's own closure adds to a
    /// closure that lacks it, the per-closure cost the carrier rule would have paid.
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn cost_receipt_exact_demand_versus_carrier_presence() {
        // No population is no observation: it fails rather than passing with nothing measured.
        let list = std::env::var("GUNBC_COST_ENTRIES")
            .expect("cost receipt: GUNBC_COST_ENTRIES unset; no population, no observation");
        let pool = super::super::build_module_path_index_from_witness_roots();
        let route_closure = super::super::emit_host::fixture_pool_closure_of("std.coercion", &pool)
            .expect("std.coercion's closure loads");
        let mut entries = 0usize;
        let mut demanding = 0usize;
        let mut carrier_present = 0usize;
        let mut carrier_without_demand = 0usize;
        let mut added_by_carrier_rule = 0usize;
        let mut skipped = 0usize;
        for path in std::fs::read_to_string(&list)
            .expect("entry list")
            .lines()
            .filter(|l| !l.trim().is_empty())
        {
            let on_disk = super::super::process_workspace_root().join(path.trim());
            let content = match std::fs::read_to_string(&on_disk) {
                Ok(content) => content,
                Err(e) => {
                    skipped += 1;
                    eprintln!("cost receipt skipped: {} unreadable: {e}", path.trim());
                    continue;
                }
            };
            let sources = match super::super::resolve_virtual_source_with_imports(
                path.trim(),
                &content,
                &pool,
            ) {
                Ok(sources) => sources,
                Err(e) => {
                    skipped += 1;
                    eprintln!(
                        "cost receipt skipped: {} closure unresolved: {e}",
                        path.trim()
                    );
                    continue;
                }
            };
            let resolved = compile(&sources);
            entries += 1;
            let demands = resolved.provider_demands.iter().any(|d| {
                d.declaration.module_path == "std.coercion"
                    && d.declaration.decl_name == "unicode_scalar_unfold"
            });
            let has_carrier = sources
                .iter()
                .any(|s| s.path.ends_with("dag/std/algebra.dag"));
            let has_route = sources
                .iter()
                .any(|s| s.path.ends_with("dag/std/coercion.dag"));
            demanding += demands as usize;
            carrier_present += has_carrier as usize;
            if has_carrier && !demands {
                carrier_without_demand += 1;
            }
            let added = if has_carrier && !has_route {
                route_closure
                    .iter()
                    .filter(|r| !sources.iter().any(|s| s.path == r.path))
                    .count()
            } else {
                0
            };
            added_by_carrier_rule += added;
            eprintln!(
                "cost receipt entry: {} closure={} demands_route={demands} carrier_present={has_carrier} carrier_rule_would_add={added}",
                path.trim(),
                sources.len()
            );
        }
        eprintln!(
            "cost receipt: entries={entries} skipped={skipped} demanding_route={demanding} carrier_present={carrier_present} carrier_present_without_demand={carrier_without_demand} modules_the_carrier_rule_would_add={added_by_carrier_rule}"
        );
    }

    /// THE REAL CROSSING THROUGH TWO REAL ADAPTERS (PR 2). The probe is the emit-check u_ row's shape:
    /// a host-text value at a v2.std.text.String formal, importing only v2.std.text, so its authored
    /// closure lacks std.coercion. Through the hermetic adapter (pool empty) the crossing's demand
    /// cannot be met and the compile refuses GeneratedProviderUnavailable -- a class only a generated
    /// demand produces, so the red proves the crossing was entered. Through the emit-check adapter
    /// (the witness-root pool) the same closure acquires std.coercion and compiles with no blocking
    /// diagnostic. Removing either the producer's demand or the adapter's acquisition reds one arm.
    #[test]
    #[ignore = "live-corpus: prepares or builds over the live tree (minutes per test); the receipts lane runs these with --ignored, the required unit run does not"]
    fn the_real_crossing_refuses_hermetically_and_acquires_through_the_pool() {
        let pool = super::super::build_module_path_index_from_witness_roots();
        let probe = "module test.claim.text_unfold_qualified_probe\nimport v2.std.text { string_is_empty }\nfn take(t: v2.std.text.String) -> Int { 0 }\nfn host_into(s: String) -> Int { take(t: s) }\n";
        let sources = emit_check_sources(probe, &pool);
        // The crossed value is the `s` argument of `take(t: s)`: the refusal is owed at its span.
        let crossed = probe.find("take(t: s)").expect("probe call") + "take(t: ".len();
        assert!(
            !sources
                .iter()
                .any(|s| s.path.ends_with("dag/std/coercion.dag")),
            "the probe's authored closure already holds the route"
        );
        let classes = |r: &ResolvedPipelineResult| -> Vec<String> {
            r.diagnostics
                .iter()
                .filter(|d| {
                    crate::v1_std_core::is_interpreter_blocking_diagnostic(d.diagnostic.clone())
                })
                .map(|d| match d.diagnostic.as_ref() {
                    CompilerDiagnostic::GeneratedProviderUnavailable {
                        cause,
                        declaration,
                        span,
                    } => {
                        format!(
                            "GeneratedProviderUnavailable/{cause:?}/{}.{}/{}:{}..{}",
                            declaration.module_path,
                            declaration.decl_name,
                            span.file,
                            span.start,
                            span.end
                        )
                    }
                    other => format!("{other:?}"),
                })
                .collect()
        };
        let hermetic = compile_to_provider_fixed_point(
            sources.clone(),
            &|_: &str| PoolAnswer::Lacks,
            &|m: &str| {
                Err(ClosureOfError::Closure(format!(
                    "hermetic: no pool holds '{m}'"
                )))
            },
        )
        .expect("hermetic compile runs");
        // The typed refusal names the route and is located at the crossed value's authored span.
        // The uncrossed value also trips the text wall and the inhabitance check at the same
        // argument; by ruling (a) those consequences stay (no span-based suppression), and the
        // acquiring arm below shows acquisition eliminates them.
        let refusal = format!(
            "GeneratedProviderUnavailable/UnavailableModuleAbsent/std.coercion.unicode_scalar_unfold/{}:{}..{}",
            super::super::emit_host::FIXTURE_SOURCE_PATH,
            crossed,
            crossed + 1
        );
        assert_eq!(
            classes(&hermetic)
                .iter()
                .filter(|c| c.starts_with("GeneratedProviderUnavailable"))
                .cloned()
                .collect::<Vec<_>>(),
            vec![refusal]
        );
        let acquired = compile_to_provider_fixed_point(
            sources,
            &|m: &str| super::super::emit_host::fixture_pool_answer(m, &pool),
            &|m: &str| super::super::emit_host::fixture_pool_closure_of(m, &pool),
        )
        .expect("acquiring compile runs");
        assert!(
            classes(&acquired).is_empty(),
            "the acquired closure still refuses: {:?}",
            classes(&acquired)
        );
    }
}
