//! THE SCOPE RANK-VIEW CARRIERS — step 2 of `docs/plans/scope-rank-view-design.md` §8.
//!
//! These land BESIDE the fold, not in place of it. Nothing here is wired into a production
//! reader: the design's §8 forbids a leaf-first cutover ("a surviving map is DESIGN §3's
//! attractor"), so the readers and the materialized maps move together in step 3, in one
//! motion. What step 2 owes is the carriers plus an equivalence law that executes — which is
//! what makes these declarations consumed (DESIGN §3c) rather than dangling.
//!
//! THE DERIVATION THEY ENCODE. A per-scope winner map stores the answer to "of the subject's
//! declarers of this key, which one ranks first in this scope?". That answer is already
//! determined by two facts held upstream: the subject-wide claimant relation, and the scope's
//! ordered admission. There is no single winner to share across scopes — the winner legitimately
//! differs with order — so DESIGN §2's "carry the first value" applies to the CLAIMANT RELATION,
//! and the winner is derived from its join with rank at the ask.
//!
//! A PHASE BOUNDARY STEP 3 MUST PRESERVE, discovered by a failing fixture in this file rather
//! than predicted. `AmbiguousBareNameRead` and the ambiguity census are NOT two renderings of one
//! event. They are distinct mechanisms over the same structural collision, in different phases:
//!
//!   ambiguous bare name IS read  -> `claim_scope_for` REFUSES; no scope reaches any reader.
//!   ambiguous bare name NOT read -> scope is admitted; the collision stays in the census.
//!
//! So for any ADMITTED scope, the census is exactly the residual population of ambiguous
//! declarations that no accepted bare read selected. An ambiguity-census reader is therefore NOT
//! enforcing the safety property that ambiguous bare reads refuse — it consumes what survived it.
//!
//! The obligation on the reader cut: the census must not become a substitute implementation of
//! `AmbiguousBareNameRead`, and the refusal must not be inferred from census membership after
//! scope admission. The two may share claimant and rank facts under the view; their outputs,
//! consumers and phase must stay distinct. ("Not read" here means not read BARE through the
//! refusing path — a qualified reference is a separate resolution channel.)
//!
//! SCOPE OF THIS FILE: the `item_registry` slot only. `scope_rank_view_slots` records six slots
//! at three polarities, and `service_ops` / `file_module_paths` resolve HIGHEST rank while this
//! one resolves lowest. A uniform cutover is silently wrong (that model's §3), so the remaining
//! slots land with their own laws rather than by generalizing this one.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;

use crate::v1_compiler_infer_items::ItemInfo;

/// One module's claim on a bare leaf name, carrying the declaring module so a resolved winner is
/// attributable without a second lookup.
#[derive(Clone)]
pub struct ItemDeclarer {
    pub module: String,
    pub info: Rc<ItemInfo>,
    pub kind: &'static str,
}

/// THE SUBJECT-WIDE CLAIMANT RELATION, one per prepared subject.
///
/// This is the fragment population inverted from per-module lists into per-name lists. It
/// introduces no new derivation and no new authority — only a different join order over facts
/// the subject already carries.
///
/// WITHIN ONE MODULE THE FIRST CLAIM WINS, mirroring the fold: it inserts under `or_insert`
/// semantics keyed by name and skips a module that already holds the name. That first claim is
/// drawn from `module.item_registry`, a `HashMap`, so where ONE module declares one leaf twice
/// the surviving claim is iteration-dependent in the fold and equally so here. That
/// nondeterminism is PRE-EXISTING and is not introduced by the view; it is recorded here because
/// an equivalence law over a corpus containing such a module would be flaky for a reason that
/// has nothing to do with the rank view, which is why the laws below use controlled fixtures.
pub struct SubjectItemDeclarers {
    by_name: HashMap<String, Vec<ItemDeclarer>>,
}

/// The winner of one name in one scope, with the two facts the fold's `winner_of` carried
/// beside the value.
#[derive(Clone)]
pub struct ItemWinner {
    pub info: Rc<ItemInfo>,
    pub module: String,
    pub authored: bool,
    pub kind: &'static str,
}

impl SubjectItemDeclarers {
    /// Built over the subject's modules in the graph's own order, which is stable for a given
    /// prepared subject. Corpus order decides nothing about a winner — rank does — but it makes
    /// the declarer list itself reproducible, and a list whose order varied run to run would
    /// make the within-module first-claim rule above vary with it.
    pub fn from_modules<'a, I>(modules: I) -> Self
    where
        I: IntoIterator<Item = &'a Rc<crate::v1_compiler_compile::TypedModule>>,
    {
        let mut by_name: HashMap<String, Vec<ItemDeclarer>> = HashMap::new();
        for module in modules {
            let module_name = module.func_env.name.clone();
            let mut claimed_here: BTreeSet<String> = BTreeSet::new();
            for (_identity, info) in module.item_registry.iter() {
                let name = info.name.clone();
                // One claim per (module, name): the fold's `winner == module_name` arm treats a
                // module's second spelling of its own leaf as not-a-collision, so the relation
                // must not carry it as a second declarer either.
                if !claimed_here.insert(name.clone()) {
                    continue;
                }
                let kind = super::item_kind_census_label(&info.kind);
                by_name.entry(name).or_default().push(ItemDeclarer {
                    module: module_name.clone(),
                    info: info.clone(),
                    kind,
                });
            }
        }
        Self { by_name }
    }

    pub fn declarers(&self, name: &str) -> &[ItemDeclarer] {
        self.by_name.get(name).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.by_name.keys()
    }
}

/// THE PER-SCOPE FACT, and the only thing a scope materializes under the view.
///
/// Its size is the scope's module count, not its item count — which is the whole difference
/// between this and the map it replaces.
pub struct ScopeRanks {
    rank: HashMap<String, usize>,
    authored_region: usize,
}

impl ScopeRanks {
    /// `order` is the scope's precedence order — own module, then the author's import closure,
    /// then the reference closure — and `authored_region` is the boundary between the second and
    /// third, exactly as `claim_scope_for` computes them. Both are taken as given rather than
    /// recomputed: recomputing them here would be a second authority for the scope's order, which
    /// is the fork this migration exists to remove.
    pub fn from_order(order: &[String], authored_region: usize) -> Self {
        let mut rank = HashMap::with_capacity(order.len());
        for (position, module) in order.iter().enumerate() {
            // First position wins if a module were ever repeated: rank is "where this scope first
            // admits the module", and a later duplicate must not demote it.
            rank.entry(module.clone()).or_insert(position);
        }
        Self {
            rank,
            authored_region,
        }
    }

    /// The scope's whole per-scope contribution to a lookup. Membership is deliberately NOT a
    /// second method here: it is `rank_of(..).is_some()`, and the design's membership readers
    /// ("the same resolution asked for less") move onto that in step 3. A separate `admits` would
    /// be a pub declaration with no call site -- DESIGN §3c's dangling tell -- kept alive only by
    /// this module being publicly reachable.
    pub fn rank_of(&self, module: &str) -> Option<usize> {
        self.rank.get(module).copied()
    }
}

/// LOWEST RANK WINS. The polarity is named at the call rather than assumed globally, because
/// `scope_rank_view_slots` records two slots that resolve the other way and one that is
/// admission-gated.
pub fn resolve_item(
    declarers: &SubjectItemDeclarers,
    ranks: &ScopeRanks,
    name: &str,
) -> Option<ItemWinner> {
    let mut best: Option<(usize, &ItemDeclarer)> = None;
    for d in declarers.declarers(name) {
        let Some(r) = ranks.rank_of(&d.module) else {
            continue;
        };
        // Strictly-less keeps the FIRST declarer at a given rank, which matters only if one
        // module could appear twice in the relation; `claimed_here` above forbids that, so this
        // is belt-and-braces rather than a live discriminator.
        if best.is_none_or(|(br, _)| r < br) {
            best = Some((r, d));
        }
    }
    best.map(|(r, d)| ItemWinner {
        info: d.info.clone(),
        module: d.module.clone(),
        authored: r < ranks.authored_region,
        kind: d.kind,
    })
}

/// THE AMBIGUOUS SET, derived rather than accumulated.
///
/// The fold's rule, restated over the join: a name is ambiguous when its winner won OUTSIDE the
/// authored region and at least one other admitted module also claims it. A winner inside the
/// authored region is ordinary shadowing the author's imports rank, so it is never ambiguous
/// however many modules claim it — and the losing claimants of an authored winner are therefore
/// absent from this map by construction, not filtered out of it.
///
/// BOTH SIDES ARE RECORDED, matching the fold: a census naming only the newcomer could not say
/// what it collided with.
pub fn ambiguous_items(
    declarers: &SubjectItemDeclarers,
    ranks: &ScopeRanks,
) -> BTreeMap<String, BTreeSet<(String, &'static str)>> {
    let mut out: BTreeMap<String, BTreeSet<(String, &'static str)>> = BTreeMap::new();
    for name in declarers.names() {
        let mut admitted: Vec<(usize, &ItemDeclarer)> = declarers
            .declarers(name)
            .iter()
            .filter_map(|d| ranks.rank_of(&d.module).map(|r| (r, d)))
            .collect();
        if admitted.len() < 2 {
            continue;
        }
        admitted.sort_by_key(|(r, _)| *r);
        let (winner_rank, winner) = admitted[0];
        if winner_rank < ranks.authored_region {
            continue;
        }
        let claimants = out.entry(name.clone()).or_default();
        claimants.insert((winner.module.clone(), winner.kind));
        for (_, d) in admitted.iter().skip(1) {
            claimants.insert((d.module.clone(), d.kind));
        }
    }
    out
}

/// THE EQUIVALENCE LAW — what makes the carriers above consumed rather than dangling (§3c).
///
/// It executes the REAL fold via `claim_scope_for_without_memos` and compares the view's answer
/// against it. A law that compared the view to a second hand-written fold would be measuring the
/// author's two transcriptions of one rule against each other.
#[cfg(test)]
mod equivalence {
    use super::*;
    use crate::cli_run::{claim_scope_for_without_memos, PreparedRepository};

    /// A minimal prepared subject over in-memory sources — the shape
    /// `scope_fragment_memo_equivalence` established for gunbc#9809.
    fn prepared_from(sources: &[(&str, &str)]) -> PreparedRepository {
        let files: Vec<Rc<crate::v1_compiler_compile::SourceFile>> = sources
            .iter()
            .map(|(path, content)| {
                Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: path.to_string(),
                    content: content.to_string(),
                })
            })
            .collect();
        let result = crate::v1_compiler_compile::compile_to_resolved(Rc::new(files.into()));
        let graph = result.graph.as_ref().expect("fixture graph").clone();
        PreparedRepository {
            graph,
            source_indices: result.source_indices.clone(),
            // Distinct per fixture: the fragment memo is keyed by it, and a shared spelling would
            // let one fixture's fragments answer another's scopes.
            subject_digest: format!("rank-view-fixture-{}", sources.len()),
            modules_resolved: sources.len(),
            modules_excluded: 0,
            full_inventory: Vec::new(),
            discovery_exclusions: Default::default(),
        }
    }

    /// TWO MODULES CLAIMING ONE BARE NAME, EACH ITS OWN ENTRY — the shape
    /// `scope_fragment_memo_equivalence::colliding_corpus` established and this adopts rather
    /// than reinvents.
    ///
    /// WHY THIS SHAPE AND NOT AN IMPORT-ORDER ONE. My first fixture made one entry import the two
    /// colliding modules in one order and a second entry import them in the other, relying on
    /// "the last direct import's closure first" — a precedence rule I had inferred from a comment
    /// rather than observed. This shape needs no such inference: the ENTRY'S OWN MODULE ranks
    /// first by construction, so `fx.alpha` resolves the collision to alpha and `fx.beta` to beta,
    /// and the flip is a property of the scope's definition rather than of my reading of it.
    ///
    /// WHAT THIS FIXTURE ESTABLISHES ABOUT AMBIGUITY, CORRECTED. Each entry's own declaration wins
    /// INSIDE the authored region, at rank 0. The later reference-closure claimant is therefore
    /// skipped by the fold's authored-winner arm, so this corpus establishes ORDINARY SHADOWING and
    /// an EMPTY ambiguity census -- which is a real assertion, not a weak one: a view that reported
    /// every collision as ambiguous fails here. `ambiguous_corpus` independently exercises the
    /// non-empty unauthored-winner arm.
    ///
    /// AN EARLIER REVISION OF THIS PARAGRAPH CLAIMED THE OPPOSITE -- that the loser arriving through
    /// the reference closure made the census "exercised for real instead of being pinned at empty".
    /// That was reasoned from the closure's position and never traced through `winner_of`; the
    /// entry declares the colliding name itself, so it always wins authored and the claim could not
    /// have been true. Recorded rather than deleted because the same wrong reading was relayed to
    /// two readers before it was traced.
    fn colliding_corpus() -> PreparedRepository {
        prepared_from(&[
            (
                "workspace/src/alpha.dag",
                "module fx.alpha\n\
                 import fx.shared { helper }\n\
                 fn shared_name() -> Bool { true }\n\
                 fn alpha_entry() -> Bool { shared_name() }\n",
            ),
            (
                "workspace/src/beta.dag",
                "module fx.beta\n\
                 import fx.shared { helper }\n\
                 fn shared_name() -> Bool { false }\n\
                 fn beta_entry() -> Bool { shared_name() }\n",
            ),
            (
                "workspace/src/shared.dag",
                "module fx.shared\n\
                 fn helper() -> Bool { true }\n\
                 data shared_datum: Bool = true\n",
            ),
        ])
    }

    /// AN ENTRY THAT PULLS IN TWO COLLIDING MODULES WITHOUT READING THE COLLIDING NAME.
    ///
    /// MY FIRST VERSION OF THIS CORPUS FAILED, AND THE FAILURE IS THE REASON THIS COMMENT IS
    /// LONG. It had the entry read `shared_name` bare. `claim_scope_for` REFUSED to build the
    /// scope: `cause=AmbiguousBareNameRead ... sites=1`, because "the shared name slot would pick
    /// one by scope precedence, which is a resolution nothing in the source authorizes".
    ///
    /// So the refusal and the census are DIFFERENT MECHANISMS over the same collision, and I had
    /// conflated them. The refusal fires on an ambiguous name that is READ bare; the census
    /// records an ambiguous name that is NOT. A corpus cannot exercise the census by reading the
    /// name, because reading it is precisely what trips the wall instead.
    ///
    /// Hence this shape: the entry reads `a_only` and `b_only`, which pulls both modules into the
    /// REFERENCE closure — outside the authored region — while `shared_name`, declared by both, is
    /// never read. The winner is therefore unauthored, the fold's `Some((held, false, _))` arm
    /// fires, and both claimants are recorded with no refusal.
    fn ambiguous_corpus() -> PreparedRepository {
        prepared_from(&[
            (
                "workspace/src/amb_entry.dag",
                "module fx.amb_entry\n\
                 fn amb_entry() -> Bool { a_only() }\n\
                 fn amb_entry_two() -> Bool { b_only() }\n",
            ),
            (
                "workspace/src/amb_a.dag",
                "module fx.amb_a\n\
                 fn a_only() -> Bool { true }\n\
                 fn shared_name() -> Bool { true }\n",
            ),
            (
                "workspace/src/amb_b.dag",
                "module fx.amb_b\n\
                 fn b_only() -> Bool { false }\n\
                 fn shared_name() -> Bool { false }\n",
            ),
        ])
    }

    /// The winner the FOLD chose for `name`, read off the scope the fold built.
    ///
    /// IDENTIFIED BY POINTER, NOT BY `module_name`. Both scopes read the same prepared graph, so
    /// the winning `ItemInfo` is literally the same allocation and `Rc::ptr_eq` is exact. The
    /// name-valued alternative is a trap this very fold records: its own comment describes a
    /// control catching that "`module_name` is not the module-path spelling the inventory
    /// carries". A law keyed on that spelling could agree while the two sides disagreed, or
    /// disagree while they agreed.
    fn fold_winner(scope: &crate::cli_run::PreparedClaimScope, name: &str) -> Option<Rc<ItemInfo>> {
        scope.indexes.item_registry.get(name).cloned()
    }

    /// THE POSITIVE CONTROL, asserted FIRST: the fixture actually discriminates. If both entries
    /// resolved `shared` to the same module, every assertion below would hold for a view with the
    /// polarity inverted, and the law would be permanently green by construction (DESIGN §4b).
    #[test]
    fn the_fixture_ranks_one_colliding_name_oppositely() {
        let prepared = colliding_corpus();
        let s1 = claim_scope_for_without_memos(&prepared, "fx.alpha").expect("scope alpha");
        let s2 = claim_scope_for_without_memos(&prepared, "fx.beta").expect("scope beta");
        let w1 = fold_winner(&s1, "shared_name").expect("alpha resolves shared_name");
        let w2 = fold_winner(&s2, "shared_name").expect("beta resolves shared_name");
        assert!(
            !Rc::ptr_eq(&w1, &w2),
            "fixture is not discriminating: both entries resolved `shared` to the same \
             declaration ({}), so this law cannot tell minimum-rank from maximum-rank and would \
             pass against an inverted view",
            w1.module_name
        );
    }

    /// THE LAW: the view's winner equals the fold's, for every name the fold resolved, in BOTH
    /// scopes — including the one whose order is the reverse of the other's.
    #[test]
    fn rank_view_resolves_every_name_as_the_fold_does() {
        let prepared = colliding_corpus();
        for entry in ["fx.alpha", "fx.beta"] {
            let scope = claim_scope_for_without_memos(&prepared, entry).expect("scope");
            let declarers = SubjectItemDeclarers::from_modules(prepared.graph.modules.iter());
            let ranks = ScopeRanks::from_order(&scope.scope_order, scope.authored_region);
            assert!(
                !scope.indexes.item_registry.is_empty(),
                "{entry}: empty registry — an empty comparison is not an equivalence"
            );
            for (name, info) in scope.indexes.item_registry.iter() {
                let view = resolve_item(&declarers, &ranks, name)
                    .unwrap_or_else(|| panic!("{entry}: view resolved nothing for `{name}`"));
                assert!(
                    Rc::ptr_eq(&view.info, info),
                    "{entry}: `{name}` — fold chose the declaration in {}, view chose {}",
                    info.module_name,
                    view.module
                );
            }
        }
    }

    /// The ambiguity census derived from the join equals the one the fold accumulated.
    ///
    /// ON THIS CORPUS THE CENSUS IS EMPTY ON BOTH SIDES, AND THAT IS NOT THE WEAK CASE IT LOOKS
    /// LIKE — but it is not the ambiguous one either. I claimed twice that the reference-closure
    /// arrival made it non-empty; tracing the fold shows otherwise, and the trace is the reason
    /// the separate corpus below exists.
    ///
    /// The entry declares the colliding name itself, so it wins at rank 0, INSIDE the authored
    /// region. The later claimant arriving through the reference closure therefore lands on the
    /// fold's `Some((_, true, _)) => {}` arm and records nothing. So what this pins is the
    /// AUTHORED-WINNER SKIP: a collision that must NOT be reported as ambiguous. A view that
    /// reported every collision would fail here, which makes an empty census a real assertion
    /// rather than a vacuous one.
    ///
    /// The non-empty arm needs the WINNER outside the authored region, which needs an entry that
    /// does not declare the name at all. That is `ambiguous_corpus`.
    #[test]
    fn ambiguity_census_agrees_with_the_fold() {
        let prepared = colliding_corpus();
        for entry in ["fx.alpha", "fx.beta"] {
            let scope = claim_scope_for_without_memos(&prepared, entry).expect("scope");
            let declarers = SubjectItemDeclarers::from_modules(prepared.graph.modules.iter());
            let ranks = ScopeRanks::from_order(&scope.scope_order, scope.authored_region);
            let view: BTreeMap<String, BTreeSet<(String, &'static str)>> =
                ambiguous_items(&declarers, &ranks);
            let fold: BTreeMap<String, BTreeSet<(String, &'static str)>> = scope
                .ambiguous_bare_names
                .iter()
                .map(|a| (a.name.clone(), a.claimants.iter().cloned().collect()))
                .collect();
            assert_eq!(view, fold, "{entry}: ambiguity census diverged");
        }
    }

    /// THE NON-EMPTY AMBIGUITY LAW, and its own positive control.
    ///
    /// The control is asserted first for the same reason as the winner law's: if the fold records
    /// nothing on this corpus then the corpus does not reach the arm, and an `assert_eq` of two
    /// empty maps would green while proving nothing about ambiguity at all.
    #[test]
    fn ambiguity_census_agrees_on_a_corpus_that_reaches_the_unauthored_arm() {
        let prepared = ambiguous_corpus();
        let scope = claim_scope_for_without_memos(&prepared, "fx.amb_entry").expect("scope");
        let fold: BTreeMap<String, BTreeSet<(String, &'static str)>> = scope
            .ambiguous_bare_names
            .iter()
            .map(|a| (a.name.clone(), a.claimants.iter().cloned().collect()))
            .collect();
        assert!(
            fold.contains_key("shared_name"),
            "corpus does not reach the unauthored-winner arm: the fold recorded {fold:?}, so an \
             equality assertion here would compare two empty maps and prove nothing"
        );
        let declarers = SubjectItemDeclarers::from_modules(prepared.graph.modules.iter());
        let ranks = ScopeRanks::from_order(&scope.scope_order, scope.authored_region);
        assert_eq!(
            ambiguous_items(&declarers, &ranks),
            fold,
            "ambiguity census diverged on the unauthored-winner arm"
        );
    }
}
