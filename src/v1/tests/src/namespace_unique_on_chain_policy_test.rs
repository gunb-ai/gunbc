//! Canonical binding witness (namespace-canonical-binding / namespace-resolution-design.md §13):
//! production global-bare and fn-parent cardinality always routes through the one
//! `module_path_owner_binding_decide` fold on chain-filtered populations — no policy
//! bracket may re-enable nearest-wins or first-hit silent picks on these paths
//! (roadmap `namespace-canonical-binding` out_of_scope: no configuration switch).

use std::rc::Rc;

use v1_compiler::cli_run::{containment_resolve_fn_v1_for_module, ContainmentResolve};
use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_rt::{
    name_resolution_policy_is_namespace_only, name_resolution_policy_set_namespace_only,
};
use v1_compiler::v1_std_core::{diagnostic_to_message, is_error_diagnostic};

/// Panic-safe policy bracket: retained only to prove the host gate no longer bypasses
/// canonical binding on the production paths this PR routes.
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
fn policy_bracket_does_not_bypass_canonical_homonym_refusal() {
    let _guard = ResolutionPolicyGuard::set(false);
    let diags = error_diag_messages(homonym_fixture());
    assert!(
        diags.iter().any(|m| m.contains("ambiguous reference 'Homonym'")),
        "host policy bracket false must not restore nearest-wins for on-chain homonyms; got {diags:?}"
    );
    assert!(
        diags
            .iter()
            .any(|m| m.contains("ambiguous reference 'pick'")),
        "host policy bracket false must not restore first-hit fn resolution; got {diags:?}"
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
fn single_off_chain_unique_refuses_regardless_of_policy_bracket() {
    for namespace_only in [false, true] {
        let _guard = ResolutionPolicyGuard::set(namespace_only);
        let diags = error_diag_messages(single_off_chain_unique_fixture());
        assert!(
            diags.iter().any(|m| m.contains("unresolved type 'Solo'")),
            "corpus-unique off-chain name must refuse as UnresolvedType (policy={namespace_only}); got {diags:?}"
        );
        assert!(
            !diags.iter().any(|m| m.contains("ambiguous")),
            "single off-chain candidate is Unresolved, not Ambiguous (policy={namespace_only}); got {diags:?}"
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
