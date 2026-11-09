//! The KEYED module read: where is module `m` stored under these pool roots, answered from its
//! NAME, not from a walk of every file in the pool.
//!
//! gunbc.design_argument::single_authority ("the name-to-path mapping is a single authority too"): module `a.b.c` is stored
//! at `<root(a)>/b/c.dag` (`gunbc.source_layout`), so finding one module must not cost reading all
//! of them (section 2). The whole-pool answers (`build_module_path_index`, the declaration census
//! `decl_facts_corpus_walk`) lex, parse and retain every pool file; a consumer that asks about one
//! module paid ~8 GiB for it (`gunbc.integration_concern`
//! `keyed_module_reads_walk_the_whole_pool`). This is the one lookup both keyed reads
//! (`module_declaration_fact_at`, `coproduct_reflection::eval_decl_facts_at`) consult.
//!
//! It reuses `demand_acquire`'s closure-acquisition primitives (the governed-root test, the
//! layout-path read, the derived-roster read, the custom-root walk) and adds no second layout
//! table. Per canonical root:
//!   - a GOVERNED root (`dag`, `src/v2`): the layout path, read once, admitted when it lies under
//!     the root; the derived ledger rosters (no file of their own) are minted the same way
//!     `read_layout_module` mints them;
//!   - a CUSTOM root: that root alone is walked (`custom_root_module_census`), once per process,
//!     because paths there do not follow names.
//!
//! MULTIPLICITY IS PRESERVED, NOT COLLAPSED. The result is one location per DISTINCT file that
//! declares the module, in pool-root order: the census emits a declaration for every copy
//! (`DeclarationRefAmbiguous` is derived from that multiplicity), and the module-path index answers
//! with the first root's copy (primary precedence). Roots are canonicalized and an identical root
//! handed twice is one root, and a file reached through two nested roots is one location, so no
//! ambiguity is fabricated by how the pool was spelled.
//!
//! Root order is the authority on both kinds of root: this read walks the roots in order and
//! returns every distinct copy, so its first element is the first root's copy whether that root is
//! governed or custom. `read_layout_module` (closure acquisition) applies the same order and, where
//! two roots hold a module, takes the first (`FirstRootWins`) or refuses (`Refuse`); this read keeps
//! the census contract of reporting both copies, which is what its consumers were written against
//! (`gunbc.integration_concern` `keyed_module_reads_walk_the_whole_pool`).
//!
//! Seed-growth receipt: `gunbc.keyed_module_lookup_seed_growth`
//! `keyed_module_lookup_seed_growth_justification`.

use super::demand_acquire::{
    custom_root_module_census, read_derived_roster_module, read_layout_file,
    source_root_is_governed_layout, under_source_roots,
};
use super::{anchor_source_root, module_index_path_key, refuse_module_index, ModuleIndexRefusal};
use crate::module_path_index::{parse_module_binding, ModuleBindingOutcome};
use crate::v1_compiler_compile::SourceFile;
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

type CustomRootCensus = HashMap<String, Rc<SourceFile>>;

thread_local! {
    /// One walk per custom root per process: its modules by declared name. A root's contents are
    /// fixed for the process (the same assumption `MODULE_PATH_INDEX_CACHE` makes).
    static CUSTOM_ROOT_CENSUS: RefCell<HashMap<String, Rc<CustomRootCensus>>> =
        RefCell::new(HashMap::new());
}

/// Canonical roots, identical roots collapsed, order kept (root order is the index's precedence).
pub(crate) fn canonical_pool_roots(pool_roots: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for root in pool_roots {
        let anchored = anchor_source_root(root);
        if !out.contains(&anchored) {
            out.push(anchored);
        }
    }
    out
}

fn custom_root_census_for(root: &str) -> Rc<CustomRootCensus> {
    if let Some(hit) = CUSTOM_ROOT_CENSUS.with(|c| c.borrow().get(root).cloned()) {
        return hit;
    }
    let census = match custom_root_module_census(&[root.to_string()]) {
        Ok(census) => census,
        // The index refuses a module declared in two files of one root; the same refusal, same exit.
        Err(cause) => {
            refuse_module_index(&[ModuleIndexRefusal::RootUnreadable {
                path: root.to_string(),
                reason: cause,
            }]);
            unreachable!("refuse_module_index exits on a nonempty refusal set")
        }
    };
    let census = Rc::new(census);
    CUSTOM_ROOT_CENSUS.with(|c| c.borrow_mut().insert(root.to_string(), census.clone()));
    census
}

/// Every distinct file declaring `module` under `pool_roots`, in root order. `path` is the
/// module-index key spelling (`module_index_path_key`), `content` the bytes read.
pub(crate) fn keyed_module_sources(pool_roots: &[String], module: &str) -> Vec<Rc<SourceFile>> {
    let roots = canonical_pool_roots(pool_roots);
    let mut found: Vec<Rc<SourceFile>> = Vec::new();
    let mut push = |src: Rc<SourceFile>| {
        if !found.iter().any(|f| f.path == src.path) {
            found.push(src);
        }
    };
    for root in &roots {
        if source_root_is_governed_layout(root) {
            let one_root = std::slice::from_ref(root);
            let derived = read_derived_roster_module(module).unwrap_or_else(|cause| {
                refuse_unreadable(root, cause);
                unreachable!("refuse_module_index exits on a nonempty refusal set")
            });
            if let Some(src) = derived {
                // A derived roster is minted in memory; it exists under this root exactly when
                // the root contains its row directory (its path is `<row dir>/roster.dag`).
                if under_source_roots(one_root, &super::workspace_root().join(&src.path)) {
                    push(src);
                }
                continue;
            }
            let layout = read_layout_file(module, |abs| under_source_roots(one_root, abs))
                .unwrap_or_else(|cause| {
                    refuse_unreadable(root, cause);
                    unreachable!("refuse_module_index exits on a nonempty refusal set")
                });
            if let Some((abs, _rel, content)) = layout {
                push(Rc::new(SourceFile {
                    path: module_index_path_key(&abs),
                    content,
                }));
            }
        } else if let Some(src) = custom_root_census_for(root).get(module) {
            push(src.clone());
        }
    }
    found
}

fn refuse_unreadable(root: &str, cause: String) {
    refuse_module_index(&[ModuleIndexRefusal::RootUnreadable {
        path: root.to_string(),
        reason: cause,
    }]);
}

/// The module-index row for `module`: the first root's copy (primary precedence), as
/// `try_build_module_path_index_uncached` answers. The located file's head is parsed as the index
/// parses it, so a file that would have refused the whole index refuses here too.
pub(crate) fn keyed_module_path(pool_roots: &[String], module: &str) -> Option<String> {
    let first = keyed_module_sources(pool_roots, module)
        .into_iter()
        .next()?;
    match parse_module_binding(Path::new(&first.path), &first.content) {
        Ok(ModuleBindingOutcome::Bound(_)) => {}
        Ok(_) => return None,
        Err(refusal) => {
            refuse_module_index(&[ModuleIndexRefusal::Unparseable(refusal)]);
            unreachable!("refuse_module_index exits on a nonempty refusal set")
        }
    }
    Some(first.path.clone())
}

#[cfg(test)]
mod tests {
    use super::{keyed_module_path, keyed_module_sources};
    use crate::coproduct_reflection::{decl_facts_corpus_walk, keyed_decl_facts};
    use std::fs;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static FIXTURE: AtomicUsize = AtomicUsize::new(0);

    fn fixture_root(tag: &str, files: &[(&str, &str)]) -> String {
        let dir = super::super::workspace_root().join("target").join(format!(
            "gunbc-keyed-{tag}-{}-{}",
            std::process::id(),
            FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&dir).expect("fixture dir");
        for (name, body) in files {
            fs::write(dir.join(name), body).unwrap();
        }
        dir.to_string_lossy().into_owned()
    }

    const SHADOWED: &str = "module test.keyed.shadowed\n\nfn probe() -> Bool { true }\n";
    const SHADOW_COPY: &str = "module test.keyed.shadowed\n\nfn probe() -> Bool { false }\n";

    fn keyed_rows(roots: &[String], qn: &str) -> Vec<String> {
        keyed_decl_facts(roots, &[qn.to_string()])
            .unwrap_or_else(|refusal| panic!("keyed read refused for {qn}: {refusal:?}"))
            .into_iter()
            .map(|f| f.rel_path)
            .collect()
    }

    /// A governed root and a custom root that both hold a module: both copies are reported, and the
    /// primary fact is the FIRST ROOT's whichever kind it is (root order, not kind, decides).
    #[test]
    fn governed_and_custom_copies_follow_root_order() {
        let custom = fixture_root(
            "gc",
            &[(
                "algebra_copy.dag",
                "module std.algebra\n\nfn probe() -> Bool { true }\n",
            )],
        );
        let governed = "dag".to_string();
        let gov_first = keyed_module_sources(&[governed.clone(), custom.clone()], "std.algebra");
        assert_eq!(
            gov_first.len(),
            2,
            "both copies: {:?}",
            gov_first.iter().map(|f| &f.path).collect::<Vec<_>>()
        );
        assert!(
            gov_first[0].path.ends_with("dag/std/algebra.dag"),
            "{}",
            gov_first[0].path
        );
        let custom_first = keyed_module_sources(&[custom.clone(), governed], "std.algebra");
        assert_eq!(custom_first.len(), 2);
        assert!(
            custom_first[0].path.contains("-gc-"),
            "{}",
            custom_first[0].path
        );
    }

    /// (a) one module under two roots: both decl rows (ambiguity), the fact is the first root's.
    #[test]
    fn module_under_two_roots_yields_both_decl_rows_and_the_primary_fact() {
        let r1 = fixture_root("a1", &[("shadowed.dag", SHADOWED)]);
        let r2 = fixture_root("a2", &[("shadow_copy.dag", SHADOW_COPY)]);
        let roots = [r1.clone(), r2.clone()];
        let rows = keyed_rows(&roots, "test.keyed.shadowed.probe");
        assert_eq!(rows.len(), 2, "both copies are emitted: {rows:?}");
        let fact = keyed_module_path(&roots, "test.keyed.shadowed").expect("declared");
        assert!(
            fact.ends_with("shadowed.dag") && fact.contains("-a1-"),
            "{fact}"
        );
        // The census, as oracle, agrees on the multiplicity.
        let census = decl_facts_corpus_walk(&roots).facts;
        let census_rows: Vec<&str> = census
            .iter()
            .filter(|f| f.qualified_name == "test.keyed.shadowed.probe")
            .map(|f| f.rel_path.as_str())
            .collect();
        assert_eq!(
            census_rows,
            rows.iter().map(String::as_str).collect::<Vec<_>>()
        );
    }

    /// (b) declared only under the second root: resolves there.
    #[test]
    fn module_only_under_the_second_root_resolves_there() {
        let r1 = fixture_root(
            "b1",
            &[(
                "other.dag",
                "module test.keyed.other\n\nfn x() -> Bool { true }\n",
            )],
        );
        let r2 = fixture_root(
            "b2",
            &[(
                "only.dag",
                "module test.keyed.only\n\nfn y() -> Bool { true }\n",
            )],
        );
        let roots = [r1, r2.clone()];
        let fact = keyed_module_path(&roots, "test.keyed.only").expect("declared");
        assert!(fact.contains("-b2-"), "{fact}");
        assert_eq!(keyed_rows(&roots, "test.keyed.only.y").len(), 1);
    }

    /// (c) declared nowhere: absent.
    #[test]
    fn module_declared_nowhere_is_absent() {
        let r1 = fixture_root(
            "c1",
            &[("x.dag", "module test.keyed.x\n\nfn x() -> Bool { true }\n")],
        );
        let roots = [r1];
        assert!(keyed_module_path(&roots, "test.keyed.nowhere").is_none());
        assert!(keyed_rows(&roots, "test.keyed.nowhere.f").is_empty());
        assert!(keyed_module_path(&roots, "").is_none());
    }

    /// (d) the same root handed twice (and a nested spelling of it): no fabricated ambiguity.
    #[test]
    fn the_same_root_twice_fabricates_no_ambiguity() {
        let r1 = fixture_root("d1", &[("shadowed.dag", SHADOWED)]);
        let roots = [r1.clone(), r1.clone(), format!("{r1}/")];
        assert_eq!(keyed_rows(&roots, "test.keyed.shadowed.probe").len(), 1);
        assert_eq!(keyed_module_sources(&roots, "test.keyed.shadowed").len(), 1);
    }

    /// DIFFERENTIAL over the real corpus (`dag` + `src/v2`), the whole-pool read as the OFFLINE
    /// ORACLE only: every census module's keyed fact equals the index row, and for a stride of the
    /// census's modules the keyed rows equal the census rows filtered by key. Heavy (it builds the
    /// whole-pool index and census on purpose), so it is run by name:
    /// `cargo test keyed_reads_equal_the_whole_pool_census -- --ignored`.
    #[test]
    #[ignore = "builds the whole-pool index and census as the oracle (~GiB)"]
    fn keyed_reads_equal_the_whole_pool_census() {
        let roots = ["dag".to_string(), "src/v2".to_string()];
        let abs = super::super::pool_roots_abs(&roots);
        let index = super::super::build_module_path_index(&abs);
        assert!(
            index.len() > 9000,
            "corpus unexpectedly small: {}",
            index.len()
        );
        let mut bad: Vec<String> = Vec::new();
        for (module, path) in &index {
            if keyed_module_path(&roots, module).as_deref() != Some(path.as_str()) {
                bad.push(format!(
                    "{module}: index {path} keyed {:?}",
                    keyed_module_path(&roots, module)
                ));
            }
        }
        assert!(
            bad.is_empty(),
            "keyed fact != index row for {} modules: {:?}",
            bad.len(),
            &bad[..bad.len().min(5)]
        );

        let census = decl_facts_corpus_walk(&roots).facts;
        let mut by_module: std::collections::BTreeMap<String, Vec<usize>> = Default::default();
        for (i, f) in census.iter().enumerate() {
            if let Some((m, _)) = f.qualified_name.rsplit_once('.') {
                by_module.entry(m.to_string()).or_default().push(i);
            }
        }
        let stride: usize = std::env::var("KEYED_STRIDE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(7);
        let mut checked = 0usize;
        for (n, (module, idxs)) in by_module.iter().enumerate() {
            if n % stride != 0 {
                continue;
            }
            let names: Vec<String> = idxs
                .iter()
                .map(|&i| census[i].qualified_name.clone())
                .collect();
            let want: Vec<&crate::coproduct_reflection::DeclFactRaw> =
                idxs.iter().map(|&i| &census[i]).collect();
            // `expect` here means something: a file of the REAL pool that did not parse would be
            // the silent miss this read now refuses, so the differential also asserts the pool is
            // readable end to end, not merely that the two reads agree on what they read.
            let got = keyed_decl_facts(&roots, &names)
                .unwrap_or_else(|refusal| panic!("keyed read refused for {module}: {refusal:?}"));
            assert_eq!(got.len(), want.len(), "row count for {module}");
            for (g, w) in got.iter().zip(want) {
                assert_eq!(
                    (&g.qualified_name, &g.name, g.kind, &g.rel_path),
                    (&w.qualified_name, &w.name, w.kind, &w.rel_path),
                    "row for {module}"
                );
                assert_eq!(
                    format!("{:?}", g.node),
                    format!("{:?}", w.node),
                    "node bytes for {}",
                    g.qualified_name
                );
            }
            checked += 1;
        }
        assert!(
            checked > 1000,
            "differential checked too few modules: {checked}"
        );
    }

    fn vm_kib(field: &str) -> u64 {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|t| {
                t.lines()
                    .find(|l| l.starts_with(field))
                    .and_then(|l| l.split_whitespace().nth(1).and_then(|v| v.parse().ok()))
            })
            .unwrap_or(0)
    }

    /// Measurement: one keyed fact read and one keyed decl read on the public pool do not retain
    /// the pool; the census afterwards does. Prints VmHWM after each.
    #[test]
    #[ignore = "measurement: prints peak RSS after keyed reads, then after the census"]
    fn keyed_read_peak_rss_is_not_the_pool() {
        let roots = ["dag".to_string(), "src/v2".to_string()];
        let base = vm_kib("VmHWM:");
        let fact = keyed_module_path(&roots, "gunbc.source_layout");
        let rows = keyed_decl_facts(
            &roots,
            &["gunbc.source_layout.layout_tree_directory".to_string()],
        )
        .unwrap_or_else(|refusal| panic!("keyed read refused: {refusal:?}"));
        let keyed = vm_kib("VmHWM:");
        eprintln!(
            "RSS base_hwm_kib={base} after_keyed_hwm_kib={keyed} fact={fact:?} rows={}",
            rows.len()
        );
        let census = decl_facts_corpus_walk(&roots).facts.len();
        let full = vm_kib("VmHWM:");
        eprintln!("RSS after_census_hwm_kib={full} census_rows={census}");
        assert!(
            keyed - base < (full - base) / 10,
            "keyed read grew like the pool"
        );
    }
    /// The fixture's module and its one item. Two lowercase segments so the read DERIVES a storage
    /// path for the module and goes and reads the file: a key whose module spells nothing never
    /// opens the unreadable file, and a test built on one proves nothing about the fold's handling
    /// of a file it actually read.
    const PROBE_QUALIFIED: &str = "keyed_probe_root.probe.probe_authority";
    const PROBE_PARSES: &str =
        "module keyed_probe_root.probe\n\nfn probe_authority() -> Bool { true }\n";
    /// Same module head, so the custom-root census indexes it and the key locates it — and then a
    /// stray `)` opens an item the grammar has no shape for. This is the file the read used to
    /// contribute nothing for.
    const PROBE_UNREADABLE: &str =
        "module keyed_probe_root.probe\n\nfn probe_authority() -> Bool { true }\n)\n";

    /// THE TYPED SHAPE of the refusal — what a caller consumes. The interpreter-level pair
    /// (`coproduct_reflection`'s `keyed_decl_facts_unreadable_file_tests`) is the half that can also
    /// be RUN at the revision before the refusal, because it stands at an unchanged signature; this
    /// one asserts the fields, which only exist here.
    #[test]
    fn the_refusal_names_the_file_the_module_and_the_parse_own_words() {
        let root = fixture_root("unreadable-typed", &[("probe.dag", PROBE_UNREADABLE)]);
        let refusal = keyed_decl_facts(&[root], &[PROBE_QUALIFIED.to_string()])
            .expect_err("a derived file that does not parse must REFUSE, not answer no rows");
        match refusal {
            crate::coproduct_reflection::DeclFactsKeyedRefusal::FileUnreadable {
                file,
                module,
                cause,
            } => {
                assert!(
                    file.ends_with("probe.dag"),
                    "the refusal must name the file the key derives; got {file}"
                );
                assert_eq!(
                    module, "keyed_probe_root.probe",
                    "the refusal must name the module the key spelled that file from"
                );
                assert!(
                    cause.contains("probe.dag"),
                    "the refusal must carry the parse's own words for that file; got {cause}"
                );
            }
        }
    }

    /// The typed positive control: the well-formed sibling of the very same fixture answers its
    /// row. Together with the refusal above it says the read refuses unreadable files, and ONLY
    /// unreadable files — a read that refused everything would pass the first test and fail this one.
    #[test]
    fn a_readable_derived_file_still_answers_its_row() {
        let root = fixture_root("readable-typed", &[("probe.dag", PROBE_PARSES)]);
        let rows = keyed_decl_facts(&[root], &[PROBE_QUALIFIED.to_string()])
            .expect("a derived file that parses must answer, not refuse");
        assert_eq!(
            rows.len(),
            1,
            "the module's one named item must answer the key that names it"
        );
        assert_eq!(
            rows[0].qualified_name, PROBE_QUALIFIED,
            "the row is the one the key asked for"
        );
    }
}
