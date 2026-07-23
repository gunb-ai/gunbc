//! §8-step-1 discriminating witness (namespace-resolution-design.md §13, operator-ratified
//! 2026-07-21): the executing v1 seed resolver carries a NameResolutionPolicy gate —
//! default OFF = ImportScoped (today's nearest-wins / first-hit, byte-for-byte), ON =
//! NamespaceOnlyY strict unique-on-chain with typed `AmbiguousReference` refusals.
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
use v1_compiler::v1_rt::name_resolution_policy_set_namespace_only;
use v1_compiler::v1_std_core::{diagnostic_to_message, is_error_diagnostic};

/// Panic-safe policy bracket: the gate is thread-local and each test compiles on its
/// own thread, so enable/reset needs no cross-test lock — only drop-safety.
struct NamespaceOnlyGuard;

impl NamespaceOnlyGuard {
    fn enable() -> Self {
        name_resolution_policy_set_namespace_only(true);
        NamespaceOnlyGuard
    }
}

impl Drop for NamespaceOnlyGuard {
    fn drop(&mut self) {
        name_resolution_policy_set_namespace_only(false);
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

/// Chain homonym on BOTH paths. `fixchain` and `fixchain.mid` are both ancestors of the
/// referencing module `fixchain.mid.leaf`:
/// - type/value path: bare `Homonym` — ImportScoped resolves nearest (`fixchain.mid`),
///   NamespaceOnlyY sees 2 binders on the chain and refuses;
/// - fn path: bare `pick()` with two glob imports both providing it — ImportScoped
///   first-hits over the flat parent closure, NamespaceOnlyY refuses on 2 matches.
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

/// Control fixture: a genuinely-unbound name (declared nowhere).
fn unbound_fixture() -> Vec<Rc<SourceFile>> {
    vec![src(
        "leaf.dag",
        "module fixchain.mid.leaf\nfn use_missing(x: NoSuchTypeAnywhere) -> NoSuchTypeAnywhere { x }\n",
    )]
}

#[test]
fn import_scoped_default_resolves_homonym_fixture_clean() {
    let diags = error_diag_messages(homonym_fixture());
    assert!(
        diags.is_empty(),
        "default ImportScoped policy must preserve today's behavior verbatim \
         (nearest-wins type resolution + first-hit fn resolution); got {diags:?}"
    );
}

#[test]
fn namespace_only_refuses_chain_homonym_on_type_path() {
    let _guard = NamespaceOnlyGuard::enable();
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
    let _guard = NamespaceOnlyGuard::enable();
    let diags = error_diag_messages(homonym_fixture());
    let pick_refusals: Vec<&String> = diags
        .iter()
        .filter(|m| m.contains("ambiguous reference 'pick'"))
        .collect();
    assert!(
        !pick_refusals.is_empty(),
        "NamespaceOnlyY must refuse the 2-parent-matches fn homonym (the \
         fn_parent_first_hit silent-pick class) with a typed AmbiguousReference; got {diags:?}"
    );
    let listing = pick_refusals[0];
    assert!(
        listing.contains("fixfns.one.pick") && listing.contains("fixfns.two.pick"),
        "the fn refusal must carry both parent candidates; got {listing}"
    );
}

#[test]
fn namespace_only_unique_on_chain_still_resolves() {
    let _guard = NamespaceOnlyGuard::enable();
    let diags = error_diag_messages(unique_on_chain_fixture());
    assert!(
        diags.is_empty(),
        "exactly-one-binder-on-chain must RESOLVE under NamespaceOnlyY (the strict rule \
         is unique-on-chain, not no-homonyms-anywhere); got {diags:?}"
    );
}

#[test]
fn zero_on_chain_homonym_discriminates_the_diagnostic_label() {
    let import_scoped = error_diag_messages(zero_on_chain_fixture());
    assert!(
        import_scoped
            .iter()
            .any(|m| m.contains("unresolved type 'Stray'")),
        "ImportScoped all-disjoint LCP tie refuses as UnresolvedType today; got {import_scoped:?}"
    );
    assert!(
        !import_scoped.iter().any(|m| m.contains("ambiguous")),
        "default policy must not mint AmbiguousReference; got {import_scoped:?}"
    );

    let _guard = NamespaceOnlyGuard::enable();
    let strict = error_diag_messages(zero_on_chain_fixture());
    let stray: Vec<&String> = strict
        .iter()
        .filter(|m| m.contains("ambiguous reference 'Stray'"))
        .collect();
    assert!(
        !stray.is_empty(),
        "NamespaceOnlyY labels the zero-on-chain whole-pool homonym honestly as \
         Ambiguous (census walk parity), never a mislabeled UnresolvedType; got {strict:?}"
    );
    assert!(
        stray[0].contains("fixother.Stray") && stray[0].contains("fixother2.Stray"),
        "the refusal must carry the full pool candidate list; got {}",
        stray[0]
    );
}

#[test]
fn namespace_only_keeps_genuinely_unbound_as_unresolved_not_ambiguous() {
    let _guard = NamespaceOnlyGuard::enable();
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
