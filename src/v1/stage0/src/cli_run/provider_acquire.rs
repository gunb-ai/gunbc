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
                let before = sources.len();
                for module in &modules {
                    for s in closure_of(module)? {
                        if !sources.iter().any(|x| x.path == s.path) {
                            sources.push(s);
                        }
                    }
                }
                if sources.len() == before {
                    return Err(format!(
                        "generated provider acquisition made no progress: acquiring {modules:?} \
                         added no source to the closure"
                    ));
                }
            }
        }
    }
}
