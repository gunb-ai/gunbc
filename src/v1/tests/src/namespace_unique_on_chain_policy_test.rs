//! Cardinality-kernel witness (namespace-resolution-design.md §13): NamespaceOnlyY
//! routes ambiguous chain and fn-parent populations through the one 0/1/many fold.
//! The corpus-unique fallback remains live until reference-derived closure can replace it.

use std::rc::Rc;

use v1_compiler::cli_run::{containment_resolve_fn_v1_for_module, ContainmentResolve};
use v1_compiler::std_occurrence_identity::{OccurrenceId, OccurrenceTransportRefusal};
use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_rt::{
    name_resolution_policy_is_namespace_only, name_resolution_policy_set_namespace_only,
};
use v1_compiler::v1_std_core::{
    diagnostic_to_message, diagnostic_to_span, is_error_diagnostic, no_span, CompilerDiagnostic,
};

/// Panic-safe policy bracket: save and restore the pre-existing host policy.
struct ResolutionPolicyGuard(bool);

impl ResolutionPolicyGuard {
    fn set(namespace_only: bool) -> Self {
        let saved = name_resolution_policy_is_namespace_only();
        name_resolution_policy_set_namespace_only(namespace_only);
        ResolutionPolicyGuard(saved)
    }
}

impl Drop for ResolutionPolicyGuard {
    fn drop(&mut self) {
        name_resolution_policy_set_namespace_only(self.0);
    }
}

fn src(path: &str, content: &str) -> Rc<SourceFile> {
    Rc::new(SourceFile {
        path: path.to_string(),
        content: content.to_string(),
    })
}

fn error_diag_messages(sources: Vec<Rc<SourceFile>>) -> Vec<String> {
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    resolved
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

/// Chain homonym on BOTH paths under `fixchain.mid.leaf`:
/// - type/value path: bare `Homonym` — 2 binders on the chain → AmbiguousReference;
/// - fn path: bare `pick()` with two glob imports → AmbiguousReference.
fn homonym_fixture() -> Vec<Rc<SourceFile>> {
    vec![
        src(
            "fixchain.dag",
            "module fixchain\ntype Homonym { tag: Int }\n",
        ),
        src(
            "fixchain_mid.dag",
            "module fixchain.mid\ntype Homonym { other: Int }\n",
        ),
        src(
            "fixfns_one.dag",
            "module fixfns.one\nfn pick() -> Int { 1 }\n",
        ),
        src(
            "fixfns_two.dag",
            "module fixfns.two\nfn pick() -> Int { 2 }\n",
        ),
        src(
            "leaf.dag",
            "module fixchain.mid.leaf\nimport fixfns.one\nimport fixfns.two\nfn use_homonym(x: Homonym) -> Homonym { x }\nfn call_pick() -> Int { pick() }\n",
        ),
    ]
}

fn unique_on_chain_fixture() -> Vec<Rc<SourceFile>> {
    vec![
        src(
            "fixchain_mid.dag",
            "module fixchain.mid\ntype Duo { d: Int }\n",
        ),
        src("fixother.dag", "module fixother\ntype Duo { e: Int }\n"),
        src(
            "leaf.dag",
            "module fixchain.mid.leaf\nfn use_duo(x: Duo) -> Duo { x }\n",
        ),
    ]
}

fn zero_on_chain_fixture() -> Vec<Rc<SourceFile>> {
    vec![
        src("fixother.dag", "module fixother\ntype Stray { a: Int }\n"),
        src("fixother2.dag", "module fixother2\ntype Stray { b: Int }\n"),
        src(
            "leaf.dag",
            "module fixchain.mid.leaf\nfn use_stray(x: Stray) -> Stray { x }\n",
        ),
    ]
}

fn single_off_chain_unique_fixture() -> Vec<Rc<SourceFile>> {
    vec![
        src("fixother.dag", "module fixother\ntype Solo { a: Int }\n"),
        src(
            "leaf.dag",
            "module fixchain.mid.leaf\nfn use_solo(x: Solo) -> Solo { x }\n",
        ),
    ]
}

fn unbound_fixture() -> Vec<Rc<SourceFile>> {
    vec![src(
        "leaf.dag",
        "module fixchain.mid.leaf\nfn use_missing(x: NoSuchTypeAnywhere) -> NoSuchTypeAnywhere { x }\n",
    )]
}

#[test]
/// PRE-FLIP EXPECTATION: ImportScoped retains nearest-wins and first-hit resolution.
/// INVERT WHEN: canonical-binding-as-production-flip lands after reference-derived
/// provider closure can replace the whole-pool fallback.
fn import_scoped_resolves_homonym_fixture_clean() {
    let _guard = ResolutionPolicyGuard::set(false);
    let diags = error_diag_messages(homonym_fixture());
    assert!(
        diags.is_empty(),
        "ImportScoped must retain nearest-wins and first-hit behavior until the downstream production flip; got {diags:?}"
    );
}

#[test]
fn canonical_refuses_chain_homonym_on_type_path() {
    let _guard = ResolutionPolicyGuard::set(true);
    let diags = error_diag_messages(homonym_fixture());
    let homonym_refusals: Vec<&String> = diags
        .iter()
        .filter(|m| m.contains("ambiguous reference 'Homonym'"))
        .collect();
    assert!(
        !homonym_refusals.is_empty(),
        "2-binders-on-chain type homonym must refuse with AmbiguousReference; got {diags:?}"
    );
    let listing = homonym_refusals[0];
    assert!(
        listing.contains("fixchain.Homonym") && listing.contains("fixchain.mid.Homonym"),
        "the refusal must carry the FULL candidate list; got {listing}"
    );
}

#[test]
fn canonical_refuses_fn_parent_homonym_at_call_site() {
    let _guard = ResolutionPolicyGuard::set(true);
    let diags = error_diag_messages(homonym_fixture());
    let pick_refusals: Vec<&String> = diags
        .iter()
        .filter(|m| m.contains("ambiguous reference 'pick'"))
        .collect();
    assert!(
        !pick_refusals.is_empty(),
        "2-parent-matches fn homonym must refuse with AmbiguousReference; got {diags:?}"
    );
    let listing = pick_refusals[0];
    assert!(
        listing.contains("fixfns.one.pick") && listing.contains("fixfns.two.pick"),
        "the fn refusal must carry both parent candidates; got {listing}"
    );
}

#[test]
fn unique_on_chain_still_resolves() {
    let _guard = ResolutionPolicyGuard::set(true);
    let diags = error_diag_messages(unique_on_chain_fixture());
    assert!(
        diags.is_empty(),
        "exactly-one-binder-on-chain must resolve; got {diags:?}"
    );
}

#[test]
fn zero_on_chain_homonym_is_unresolved_not_ambiguous() {
    for namespace_only in [false, true] {
        let _guard = ResolutionPolicyGuard::set(namespace_only);
        let diags = error_diag_messages(zero_on_chain_fixture());
        assert!(
            diags.iter().any(|m| m.contains("unresolved type 'Stray'")),
            "zero on-chain homonym must be UnresolvedType (policy={namespace_only}); got {diags:?}"
        );
        assert!(
            !diags.iter().any(|m| m.contains("ambiguous")),
            "zero on-chain must not widen to AmbiguousReference (policy={namespace_only}); got {diags:?}"
        );
    }
}

#[test]
fn zero_on_chain_containment_census_matches_inference_unresolved() {
    let _guard = ResolutionPolicyGuard::set(true);
    let sources = zero_on_chain_fixture();
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    let graph = resolved.graph.as_ref().expect("graph");
    let leaf = graph
        .modules
        .iter()
        .find(|m| m.type_env.module_path == "fixchain.mid.leaf")
        .expect("leaf module");
    let containment = containment_resolve_fn_v1_for_module(
        &leaf.type_env.symbol_index,
        "fixchain.mid.leaf",
        "Stray",
        None,
    );
    assert!(
        matches!(containment, ContainmentResolve::Unresolved),
        "containment census must mirror inference for zero on-chain homonym; got {containment:?}"
    );
}

#[test]
/// PRE-FLIP EXPECTATION: a corpus-unique declaration remains the degenerate global-bare
/// fallback even when it is off the referencing module's containment chain.
/// INVERT WHEN: canonical-binding-as-production-flip lands after reference-derived
/// provider closure can replace that fallback.
fn single_off_chain_unique_uses_corpus_fallback_until_production_flip() {
    for namespace_only in [false, true] {
        let _guard = ResolutionPolicyGuard::set(namespace_only);
        let diags = error_diag_messages(single_off_chain_unique_fixture());
        assert!(
            diags.is_empty(),
            "corpus-unique fallback must remain live before reference-derived closure (policy={namespace_only}); got {diags:?}"
        );
    }
}

#[test]
fn single_off_chain_unique_containment_census_matches_inference() {
    let _guard = ResolutionPolicyGuard::set(true);
    let sources = single_off_chain_unique_fixture();
    let resolved = compile_to_resolved(Rc::new(sources.into()));
    let graph = resolved.graph.as_ref().expect("graph");
    let leaf = graph
        .modules
        .iter()
        .find(|m| m.type_env.module_path == "fixchain.mid.leaf")
        .expect("leaf module");
    let containment = containment_resolve_fn_v1_for_module(
        &leaf.type_env.symbol_index,
        "fixchain.mid.leaf",
        "Solo",
        None,
    );
    assert!(
        matches!(containment, ContainmentResolve::Unresolved),
        "containment census must mirror inference for corpus-unique off-chain name; got {containment:?}"
    );
}

#[test]
fn genuinely_unbound_stays_unresolved_not_ambiguous() {
    let _guard = ResolutionPolicyGuard::set(true);
    let diags = error_diag_messages(unbound_fixture());
    assert!(
        diags
            .iter()
            .any(|m| m.contains("unresolved type 'NoSuchTypeAnywhere'")),
        "unbound name stays UnresolvedType; got {diags:?}"
    );
    assert!(
        !diags.iter().any(|m| m.contains("ambiguous")),
        "no fabricated ambiguity for an unbound name; got {diags:?}"
    );
}

/// `UnknownOccurrenceIdentity` is the one refusal in the transport that carries no
/// authored span (the id is in neither index, so none exists — absence by construction).
/// It must render through the corpus's single no-location authority, `no_span()`, and
/// must NOT launder the occurrence id into a minted pseudo-file (review 45364).
///
/// Discriminating: this goes RED if the `<unknown-occurrence:N>` placeholder is
/// reintroduced, because the assertions below reject any span whose file names the id.
#[test]
fn unknown_occurrence_refusal_renders_no_span_never_a_minted_pseudo_file() {
    let diagnostic = Rc::new(CompilerDiagnostic::OccurrenceTransportViolation {
        refusal: Rc::new(OccurrenceTransportRefusal::UnknownOccurrenceIdentity {
            occurrence: OccurrenceId { value: 4926 },
        }),
    });

    let span = diagnostic_to_span(diagnostic.clone());

    assert_eq!(
        span,
        no_span(),
        "spanless refusal must render as the single no-location authority"
    );
    assert!(
        !span.file.contains("4926"),
        "span fabricated the occurrence id into a file name: {}",
        span.file
    );
    assert!(
        !span.file.contains("unknown-occurrence"),
        "span minted a placeholder file name: {}",
        span.file
    );

    // The identity is not lost — the message owns it, and is its only carrier.
    assert!(
        diagnostic_to_message(diagnostic).contains("4926"),
        "occurrence identity must still be reported, via the message"
    );
}

/// Span-carrying refusals are unaffected: they still report their authored location,
/// so the fix above removed a fabrication rather than flattening real spans to nothing.
#[test]
fn span_carrying_refusal_still_reports_its_authored_span() {
    let authored = v1_compiler::std_types::SourceSpan {
        file: "dag/std/occurrence_identity.dag".to_string(),
        start: 120,
        end: 148,
    };
    let diagnostic = Rc::new(CompilerDiagnostic::OccurrenceTransportViolation {
        refusal: Rc::new(
            OccurrenceTransportRefusal::DuplicateAuthoredOccurrenceIdentity {
                occurrence: OccurrenceId { value: 7 },
                diagnostic_span: Rc::new(authored.clone()),
            },
        ),
    });

    let span = diagnostic_to_span(diagnostic);

    assert_eq!(*span, authored, "authored spans must survive unchanged");
    assert_ne!(
        *span,
        *no_span(),
        "a real span must not collapse to no_span"
    );
}
