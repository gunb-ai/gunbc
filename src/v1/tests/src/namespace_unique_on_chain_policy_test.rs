//! §8-step-4 discriminating witness (namespace-resolution-design.md §13, operator-ratified
//! 2026-07-21): the executing v1 seed resolver carries a NameResolutionPolicy gate —
//! default ON = NamespaceOnlyY strict unique-on-chain with typed `AmbiguousReference`
//! refusals, OFF (host-bracketed) = ImportScoped (nearest-wins / first-hit, byte-for-byte).
//!
//! The homonym fixtures below are the discriminating inputs: the SAME sources compile
//! clean under ImportScoped and refuse (typed, located, full candidate list) under
//! NamespaceOnlyY — on both the type/value path (ancestor-chain homonym resolved by
//! nearest-wins today) and the fn path (first-hit over `func_env.parents`, the
//! `fn_parent_first_hit` silent-pick class, which had NO refusal arm at all).
//! Controls pin the boundary: a chain-unique homonym still resolves under the strict
//! policy, and a genuinely-unbound name stays `UnresolvedType` — the refusal is typed,
//! never a blanket.

use std::rc::Rc;

use v1_compiler::v1_compiler_compile::{compile_to_resolved, SourceFile};
use v1_compiler::v1_rt::{
    name_resolution_policy_is_namespace_only, name_resolution_policy_set_namespace_only,
};
use v1_compiler::v1_std_core::{diagnostic_to_message, is_error_diagnostic};

/// Panic-safe policy bracket: save/restore the thread-local gate so ImportScoped cases
/// bracket `false` explicitly and drop always restores the pre-test default.
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

/// The module every fixture below references from.
const LEAF: &str = "fixchain.mid.leaf";

/// Premise guard. Each test's NAME states a condition its FIXTURE encodes -- two
/// binders on the chain, exactly one on the chain, none on the chain -- and until
/// this existed no predicate checked the encoding still held, so an edit to a
/// fixture could silently change what a test proved while its name went on
/// claiming the original.
///
/// Returns `(declarations_of_name, how_many_declare_from_an_ancestor_of chain_of)`.
/// A module is on the chain if it IS `chain_of` or is a proper prefix of it.
fn declaration_census(sources: &[Rc<SourceFile>], name: &str, chain_of: &str) -> (usize, usize) {
    let mut total = 0usize;
    let mut on_chain = 0usize;
    for source in sources {
        let mut module_path = String::new();
        for line in source.content.lines() {
            if let Some(rest) = line.strip_prefix("module ") {
                module_path = rest.trim().to_string();
                continue;
            }
            let declared = line
                .strip_prefix("type ")
                .or_else(|| line.strip_prefix("fn "))
                .and_then(|rest| {
                    rest.split(|c: char| !(c.is_alphanumeric() || c == '_'))
                        .next()
                });
            if declared == Some(name) {
                total += 1;
                if module_path == chain_of || chain_of.starts_with(&format!("{module_path}.")) {
                    on_chain += 1;
                }
            }
        }
    }
    (total, on_chain)
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

/// Chain homonym on BOTH paths. `fixchain` and `fixchain.mid` are both ancestors of the
/// referencing module `fixchain.mid.leaf`:
/// - type/value path: bare `Homonym` — ImportScoped resolves nearest (`fixchain.mid`),
///   NamespaceOnlyY sees 2 binders on the chain and refuses;
/// - fn path: bare `pick()` with two OFF-chain binders (`fixfns.one` / `fixfns.two`) —
///   ImportScoped first-hits over the flat parent closure. Under containment lookup
///   these are not ancestors of the caller, so the name is not reached at all and the
///   refusal is `not found in scope`, not `ambiguous`. The on-chain fn homonym that
///   still exercises the silent-pick class moved to `fn_chain_homonym_fixture`.
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
            "module fixchain.mid.leaf\nfn use_homonym(x: Homonym) -> Homonym { x }\nfn call_pick() -> Int { pick() }\n",
        ),
    ]
}

/// Control fixture: homonym whose candidates put EXACTLY ONE binder on the referencing
/// chain (`Duo` in ancestor `fixchain.mid` vs non-ancestor `fixother`) — must resolve
/// under BOTH policies (the strict rule is unique-on-chain, not no-homonyms-anywhere).
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

/// Control fixture: whole-pool homonym with ZERO binders on the referencing chain
/// (`Stray` declared only in two non-ancestor siblings). ImportScoped already refuses
/// (all-disjoint LCP tie) as `UnresolvedType`; NamespaceOnlyY refuses as the honest
/// `AmbiguousReference` with the full pool — mirroring the census containment walk
/// (lexical Unbound + whole-pool-ambiguous => Ambiguous, never a fabricated bind).
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

/// Two `pick` binders BOTH on the caller's ancestor chain, so containment
/// lookup reaches both and must refuse rather than silently pick one.
fn fn_chain_homonym_fixture() -> Vec<Rc<SourceFile>> {
    vec![
        src("fixchain.dag", "module fixchain\nfn pick() -> Int { 1 }\n"),
        src(
            "fixchain_mid.dag",
            "module fixchain.mid\nfn pick() -> Int { 2 }\n",
        ),
        src(
            "leaf.dag",
            "module fixchain.mid.leaf\nfn call_pick() -> Int { pick() }\n",
        ),
    ]
}

/// Control fixture: a genuinely-unbound name (declared nowhere).
fn unbound_fixture() -> Vec<Rc<SourceFile>> {
    vec![src(
        "leaf.dag",
        "module fixchain.mid.leaf\nfn use_missing(x: NoSuchTypeAnywhere) -> NoSuchTypeAnywhere { x }\n",
    )]
}

#[test]
fn namespace_only_refuses_chain_homonym_on_type_path() {
    assert_eq!(
        declaration_census(&homonym_fixture(), "Homonym", LEAF),
        (2, 2),
        "fixture premise: both Homonym binders must be ON the chain -- that is the state this refusal is about"
    );
    let _guard = ResolutionPolicyGuard::set(true);
    let diags = error_diag_messages(homonym_fixture());
    let homonym_refusals: Vec<&String> = diags
        .iter()
        .filter(|m| m.contains("ambiguous reference 'Homonym'"))
        .collect();
    assert!(
        !homonym_refusals.is_empty(),
        "NamespaceOnlyY must refuse the 2-binders-on-chain type homonym with a typed \
         AmbiguousReference; got {diags:?}"
    );
    let listing = homonym_refusals[0];
    assert!(
        listing.contains("fixchain.Homonym") && listing.contains("fixchain.mid.Homonym"),
        "the refusal must carry the FULL candidate list (fix menu, §13); got {listing}"
    );
}

#[test]
fn namespace_only_refuses_fn_parent_homonym_at_call_site() {
    assert_eq!(
        declaration_census(&fn_chain_homonym_fixture(), "pick", LEAF),
        (2, 2),
        "fixture premise: both pick binders must be ON the chain, else this is not the silent-pick class"
    );
    let _guard = ResolutionPolicyGuard::set(true);
    let diags = error_diag_messages(fn_chain_homonym_fixture());
    let pick_refusals: Vec<&String> = diags
        .iter()
        .filter(|m| m.contains("ambiguous reference 'pick'"))
        .collect();
    assert!(
        !pick_refusals.is_empty(),
        "NamespaceOnlyY must refuse the 2-binders-on-chain fn homonym (the \
         fn_parent_first_hit silent-pick class) with a typed AmbiguousReference; got {diags:?}"
    );
    let listing = pick_refusals[0];
    assert!(
        listing.contains("fixchain.pick") && listing.contains("fixchain.mid.pick"),
        "the fn refusal must carry both candidates; got {listing}"
    );
}

/// The OFF-chain companion, and the reason the fixture above changed.
///
/// `fixfns.one.pick` / `fixfns.two.pick` are not ancestors of the caller, so
/// under containment lookup a bare `pick` does not reach them at all. This is
/// not the silent-pick class getting weaker: refusing an off-chain name
/// outright is strictly safer than choosing between two candidates, and the
/// members of a sibling module are reachable as `fixfns.one.pick`, never bare.
#[test]
fn namespace_only_does_not_reach_off_chain_fn_homonyms_at_all() {
    assert_eq!(
        declaration_census(&homonym_fixture(), "pick", LEAF),
        (2, 0),
        "fixture premise: both pick binders must be OFF the chain -- an on-chain one would make this the ambiguity case"
    );
    let _guard = ResolutionPolicyGuard::set(true);
    let diags = error_diag_messages(homonym_fixture());
    assert!(
        diags
            .iter()
            .any(|m| m.contains("'pick' not found in scope")),
        "an off-chain fn homonym must not resolve; got {diags:?}"
    );
    assert!(
        !diags
            .iter()
            .any(|m| m.contains("ambiguous reference 'pick'")),
        "an unreachable name is not an ambiguous one -- the two states have \
         different remedies (qualify vs. it is not visible here); got {diags:?}"
    );
}

#[test]
fn namespace_only_unique_on_chain_still_resolves() {
    assert_eq!(
        declaration_census(&unique_on_chain_fixture(), "Duo", LEAF),
        (2, 1),
        "fixture premise: a homonym must exist (2 binders) with exactly ONE on the chain; drop the off-chain one and this test proves only that a uniquely-declared type resolves"
    );
    let _guard = ResolutionPolicyGuard::set(true);
    let diags = error_diag_messages(unique_on_chain_fixture());
    assert!(
        diags.is_empty(),
        "exactly-one-binder-on-chain must RESOLVE under NamespaceOnlyY (the strict rule \
         is unique-on-chain, not no-homonyms-anywhere); got {diags:?}"
    );
}

#[test]
fn zero_on_chain_homonym_discriminates_the_diagnostic_label() {
    assert_eq!(
        declaration_census(&zero_on_chain_fixture(), "Stray", LEAF),
        (2, 0),
        "fixture premise: two pool binders, none on the chain -- two is what gives a whole-pool fallback something to fabricate an ambiguity FROM"
    );
    let import_scoped = {
        let _guard = ResolutionPolicyGuard::set(false);
        error_diag_messages(zero_on_chain_fixture())
    };
    assert!(
        import_scoped
            .iter()
            .any(|m| m.contains("unresolved type 'Stray'")),
        "ImportScoped all-disjoint LCP tie refuses as UnresolvedType today; got {import_scoped:?}"
    );
    assert!(
        !import_scoped.iter().any(|m| m.contains("ambiguous")),
        "ImportScoped bracket must not mint AmbiguousReference; got {import_scoped:?}"
    );

    let strict = {
        let _guard = ResolutionPolicyGuard::set(true);
        error_diag_messages(zero_on_chain_fixture())
    };
    assert!(
        strict.iter().any(|m| m.contains("unresolved type 'Stray'")),
        "NamespaceOnlyY supplies the empty on-chain population to the cardinality adapter, \
         so the result is UnresolvedType; got {strict:?}"
    );
    assert!(
        !strict.iter().any(|m| m.contains("ambiguous")),
        "zero supplied candidates must not be relabeled as ambiguity from the whole-pool fallback; got {strict:?}"
    );
}

#[test]
fn namespace_only_keeps_genuinely_unbound_as_unresolved_not_ambiguous() {
    assert_eq!(
        declaration_census(&unbound_fixture(), "NoSuchTypeAnywhere", LEAF),
        (0, 0),
        "fixture premise: the name must be declared NOWHERE; with zero binders there is no candidate set, which is why this row is weaker than zero_on_chain and must not be read as covering it"
    );
    let _guard = ResolutionPolicyGuard::set(true);
    let diags = error_diag_messages(unbound_fixture());
    assert!(
        diags
            .iter()
            .any(|m| m.contains("unresolved type 'NoSuchTypeAnywhere'")),
        "a name bound NOWHERE stays UnresolvedType under the strict policy \
         (Ambiguous and Unresolved are distinct states, §5); got {diags:?}"
    );
    assert!(
        !diags.iter().any(|m| m.contains("ambiguous")),
        "no fabricated ambiguity for an unbound name; got {diags:?}"
    );
}
