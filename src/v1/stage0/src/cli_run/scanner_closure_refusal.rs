//! Host realization of `gunbc.namespace_host_closure_refusal`.
//! The scanners remain observations for this prepared cut. Only the input source
//! set can be returned: a scanner cannot construct a larger loaded closure here.

use super::{v1_compiler_compile::SourceFile, workspace_relative_repo_path};
use im::HashMap;
use serde::Serialize;
use std::collections::{BTreeSet, HashSet};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(super) enum HostClosureScanner {
    DottedModulePathScanner,
    BareReferenceScanner,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub(super) struct HostScannerClosureAddition {
    source_path: String,
    proposed_dependency_path: String,
}

#[derive(Debug, Serialize)]
pub(super) struct HostScannerClosureRefusal {
    scanner: HostClosureScanner,
    first: HostScannerClosureAddition,
    rest: Vec<HostScannerClosureAddition>,
}

impl HostScannerClosureRefusal {
    pub(super) fn render(&self) -> String {
        // The count is derived from the nonempty located population, never an
        // independently writable observation. JSON preserves unusual path bytes.
        serde_json::json!({
            "cause": "HostScannerClosureRefused",
            "count": 1 + self.rest.len(),
            "refusal": self,
        })
        .to_string()
    }
}

pub(super) fn refuse_scanner_extension(
    sources: Vec<Rc<SourceFile>>,
    proposals: &HashMap<String, Vec<String>>,
    eligible: Option<&HashSet<String>>,
    scanner: HostClosureScanner,
) -> Result<Vec<Rc<SourceFile>>, HostScannerClosureRefusal> {
    let known: HashSet<String> = sources
        .iter()
        .map(|source| workspace_relative_repo_path(&source.path))
        .collect();
    let mut additions = BTreeSet::new();
    for source in &known {
        if eligible.is_some_and(|paths| !paths.contains(source)) {
            continue;
        }
        if let Some(targets) = proposals.get(source) {
            for target in targets {
                let target = workspace_relative_repo_path(target);
                if !known.contains(&target) {
                    additions.insert((source.clone(), target));
                }
            }
        }
    }
    let mut additions = additions
        .into_iter()
        .map(
            |(source_path, proposed_dependency_path)| HostScannerClosureAddition {
                source_path,
                proposed_dependency_path,
            },
        );
    match additions.next() {
        None => Ok(sources),
        Some(first) => Err(HostScannerClosureRefusal {
            scanner,
            first,
            rest: additions.collect(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    struct Corpus(std::path::PathBuf);

    impl Corpus {
        fn new(consumer: &str) -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = workspace_root().join("target").join(format!(
                "gunbc-scanner-refusal-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed),
            ));
            std::fs::create_dir_all(&root).unwrap();
            std::fs::write(root.join("consumer.dag"), consumer).unwrap();
            std::fs::write(
                root.join("provider.dag"),
                "module scanner.provider\nfn scanner_answer() -> Int { 7 }\n",
            )
            .unwrap();
            Self(root)
        }

        fn index(&self) -> MultiEntryIndex {
            build_multi_entry_index(&[self.0.to_string_lossy().into_owned()])
        }

        fn entry(&self) -> String {
            self.0.join("consumer.dag").to_string_lossy().into_owned()
        }
    }

    impl Drop for Corpus {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn assert_refusal(error: &str, scanner: &str, corpus: &Corpus) {
        let diagnostic: serde_json::Value = serde_json::from_str(error).unwrap();
        assert_eq!(diagnostic["cause"], "HostScannerClosureRefused");
        assert_eq!(diagnostic["count"], 1);
        assert_eq!(diagnostic["refusal"]["scanner"], scanner);
        assert_eq!(
            diagnostic["refusal"]["first"]["source_path"],
            workspace_relative_repo_path(&corpus.entry())
        );
        assert_eq!(
            diagnostic["refusal"]["first"]["proposed_dependency_path"],
            workspace_relative_repo_path(&corpus.0.join("provider.dag").to_string_lossy())
        );
    }

    #[test]
    fn dotted_scanner_cannot_load_reference_only_provider_on_either_path() {
        let corpus = Corpus::new(
            "module scanner.consumer\nfn probe() -> Int { scanner.provider.scanner_answer() }\n",
        );
        let index = corpus.index();
        let direct = load_sources_for_entry_with_index(
            &index.source_files,
            &index.module_graph_facts,
            &corpus.entry(),
        )
        .unwrap_err();
        assert_refusal(&direct, "DottedModulePathScanner", &corpus);

        // Reach the cached scanner path directly: the public pool loader first
        // reaches the direct refusal above, which would mask this second door.
        let source = entry_source_from_index_or_disk(&index.source_files, &corpus.entry()).unwrap();
        let cached = extend_with_reference_closure_for_pool(vec![source], &index).unwrap_err();
        assert_refusal(&cached, "DottedModulePathScanner", &corpus);
    }

    #[test]
    fn bare_scanner_cannot_load_reference_only_provider() {
        let corpus =
            Corpus::new("module scanner.consumer\nfn probe() -> Int { scanner_answer() }\n");
        let index = corpus.index();
        // Direct dotted scan adds nothing. The pool loader must reach and refuse
        // the independent bare scanner, without a loaded-file fallback behind it.
        let direct = load_sources_for_entry_with_index(
            &index.source_files,
            &index.module_graph_facts,
            &corpus.entry(),
        )
        .unwrap();
        assert_eq!(direct.len(), 1);
        let error = load_sources_for_entry_with_pool(&index, &corpus.entry()).unwrap_err();
        assert_refusal(&error, "BareReferenceScanner", &corpus);
    }

    #[test]
    fn import_supplied_provider_remains_loaded_without_scanner_additions() {
        let corpus = Corpus::new("module scanner.consumer\nimport scanner.provider { scanner_answer }\nfn probe() -> Int { scanner.provider.scanner_answer() }\n");
        let index = corpus.index();
        let sources = load_sources_for_entry_with_pool(&index, &corpus.entry()).unwrap();
        let names: BTreeSet<_> = sources
            .iter()
            .map(|s| extract_module_path(&s.content).unwrap())
            .collect();
        assert_eq!(
            names,
            BTreeSet::from(["scanner.consumer".to_owned(), "scanner.provider".to_owned()])
        );
    }

    #[test]
    fn floor_preparation_cannot_add_a_provider_skipped_by_import_bearing_entry_scan() {
        let corpus = Corpus::new("module scanner.consumer\nimport scanner.unused { unrelated }\nfn probe() -> Int { scanner_answer() }\n");
        std::fs::write(
            corpus.0.join("unused.dag"),
            "module scanner.unused\nfn unrelated() -> Int { 0 }\n",
        )
        .unwrap();
        let index = corpus.index();
        let loaded = load_sources_for_entry_with_pool(&index, &corpus.entry()).unwrap();
        assert_eq!(
            loaded.len(),
            2,
            "entry loader must pass to reach the independent floor door"
        );
        let error = assemble_prepared_subject_closure(
            &[corpus.0.to_string_lossy().into_owned()],
            &[],
            Some((&index, &["scanner.consumer".to_owned()])),
        )
        .err()
        .expect("floor scanner addition must refuse");
        assert_refusal(&error, "BareReferenceScanner", &corpus);
    }

    #[test]
    fn refused_population_is_deduplicated_located_and_never_partially_loaded() {
        let sources = vec![Rc::new(SourceFile {
            path: "entry.dag".into(),
            content: String::new(),
        })];
        let proposals = HashMap::from_iter([
            (
                "entry.dag".into(),
                vec!["b.dag".into(), "a.dag".into(), "b.dag".into()],
            ),
            ("unloaded.dag".into(), vec!["unreached.dag".into()]),
        ]);
        let error = refuse_scanner_extension(
            sources,
            &proposals,
            None,
            HostClosureScanner::BareReferenceScanner,
        )
        .unwrap_err();
        let diagnostic: serde_json::Value = serde_json::from_str(&error.render()).unwrap();
        assert_eq!(diagnostic["count"], 2);
        assert_eq!(error.first.proposed_dependency_path, "a.dag");
        assert_eq!(error.rest[0].proposed_dependency_path, "b.dag");
    }

    #[test]
    #[ignore = "explicit stopped-line audit over dag and src/v2; requires SCANNER_AUDIT_REPORT"]
    fn real_corpus_scanner_additions_are_reported_without_loading_them() {
        let report_path = std::env::var("SCANNER_AUDIT_REPORT").expect("audit output path");
        let roots = ["dag", "src/v2"]
            .map(|root| workspace_root().join(root).to_string_lossy().into_owned());
        let index = build_multi_entry_index_primary_precedence(&roots);
        let edges =
            both_closure_edge_index(&index).expect("scanner observations must be available");
        let mut floor_bare = edges.bare_out.clone();
        for source in index.source_files.values() {
            let path = workspace_relative_repo_path(&source.path);
            if !floor_bare.contains_key(&path) {
                floor_bare.insert(
                    path,
                    bare_reference_pull_paths_for_source(source, &index)
                        .expect("floor scanner observation must be available"),
                );
            }
        }
        let mut entries: Vec<_> = index.source_files.values().cloned().collect();
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        assert!(
            !entries.is_empty(),
            "an empty audit cannot establish coverage"
        );
        let mut rows = Vec::new();
        let mut refused_entries = 0;
        for entry in &entries {
            let sources = resolve_transitively(
                vec![entry.clone()],
                &index.source_files,
                &index.module_graph_facts,
            )
            .expect("import closure must be accounted before scanner comparison");
            let mut refusals = Vec::new();
            // Both observations read the SAME import-only subject. The first
            // refusal must not prevent the audit from observing the other arm.
            for (door, scanner, proposals, eligible) in [
                (
                    "entry_dotted",
                    HostClosureScanner::DottedModulePathScanner,
                    &edges.ref_out,
                    None,
                ),
                (
                    "entry_bare",
                    HostClosureScanner::BareReferenceScanner,
                    &edges.bare_out,
                    Some(&edges.bare_scan_eligible),
                ),
                (
                    "floor_bare",
                    HostClosureScanner::BareReferenceScanner,
                    &floor_bare,
                    None,
                ),
            ] {
                if let Err(refusal) =
                    refuse_scanner_extension(sources.clone(), proposals, eligible, scanner)
                {
                    refusals.push(serde_json::json!({"door": door, "refusal": refusal}));
                }
            }
            if !refusals.is_empty() {
                refused_entries += 1;
            }
            rows.push(serde_json::json!({
                "entry": workspace_relative_repo_path(&entry.path),
                "import_closure_modules": sources.len(),
                "refusals": refusals,
            }));
        }
        let report = serde_json::json!({
            "state": "StoppedLineAudit",
            "compared_modules": entries.len(),
            "refused_entries": refused_entries,
            "entries_without_scanner_additions": entries.len() - refused_entries,
            "rows": rows,
        });
        std::fs::write(report_path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
        eprintln!("SCANNER_AUDIT compared_modules={} refused_entries={} entries_without_scanner_additions={}",
            entries.len(), refused_entries, entries.len() - refused_entries);
    }

    #[test]
    #[ignore = "paired source-compilation audit; requires SCANNER_COMPILE_AUDIT_REPORT"]
    fn real_corpus_withholding_scanner_additions_compiles_both_subjects() {
        use std::io::Write;
        let report_path = std::env::var("SCANNER_COMPILE_AUDIT_REPORT").expect("audit output path");
        let mut report = std::fs::File::create(report_path).unwrap();
        let roots = ["dag", "src/v2"]
            .map(|root| workspace_root().join(root).to_string_lossy().into_owned());
        let index = build_multi_entry_index_primary_precedence(&roots);
        let edges = both_closure_edge_index(&index).expect("scanner observations");
        let lookup = path_to_source_lookup(&index.source_files);
        let mut entries: Vec<_> = index.source_files.values().cloned().collect();
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        assert!(!entries.is_empty(), "empty comparison");
        assert!(
            v1_compiler_compile::default_compile_pipeline_options()
                .census_only_sources
                .is_empty(),
            "an external census would mask withheld sources"
        );
        writeln!(report, "{}", serde_json::json!({
            "state": "Started", "indexed_modules": entries.len(),
            "boundary": "source compile_to_resolved through resolve/type/ownership/complexity; graph present and no is_interpreter_blocking_diagnostic; no external census; NOT emitted-Rust cargo acceptance",
            "diagnostic_identity": "serialized ErrorNode: full CompilerDiagnostic variant/payload and module_name",
        })).unwrap();

        // Entries can demand the same complete program (in particular through
        // scanner cycles). Within this immutable source index those are one
        // compilation, even though each entry retains its own comparison row.
        let mut programs =
            std::collections::HashMap::<Vec<String>, (bool, BTreeSet<String>)>::new();
        let mut compile = |sources: Vec<Rc<SourceFile>>| {
            let key: Vec<_> = sources.iter().map(|source| source.path.clone()).collect();
            if let Some(observed) = programs.get(&key) {
                return observed.clone();
            }
            let result = v1_compiler_compile::compile_to_resolved(Rc::new(sources.into()));
            let passes = result.graph.is_some()
                && !result.diagnostics.iter().any(|d| {
                    crate::v1_std_core::is_interpreter_blocking_diagnostic(d.diagnostic.clone())
                });
            let diagnostics: BTreeSet<_> = result
                .diagnostics
                .iter()
                .map(|d| serde_json::to_string(d).expect("typed diagnostic serialization"))
                .collect();
            let observed = (passes, diagnostics);
            programs.insert(key, observed.clone());
            observed
        };
        let mut pairs = 0;
        let mut partitions = std::collections::BTreeMap::<&str, usize>::new();
        for entry in &entries {
            let mut imports = resolve_transitively(
                vec![entry.clone()],
                &index.source_files,
                &index.module_graph_facts,
            )
            .expect("import closure provenance");
            imports.sort_by(|a, b| a.path.cmp(&b.path));
            let dotted = refuse_scanner_extension(
                imports.clone(),
                &edges.ref_out,
                None,
                HostClosureScanner::DottedModulePathScanner,
            );
            let bare = refuse_scanner_extension(
                imports.clone(),
                &edges.bare_out,
                Some(&edges.bare_scan_eligible),
                HostClosureScanner::BareReferenceScanner,
            );
            if dotted.is_ok() && bare.is_ok() {
                continue;
            }

            // OFFLINE HISTORICAL ORACLE ONLY. This reproduces the removed entry
            // loader's union/fixpoint over its original scanner edge maps. It is
            // never returned to a production loader, and the withholding arm
            // below receives only the import-derived sources.
            let mut old_paths: HashSet<String> = imports
                .iter()
                .map(|s| workspace_relative_repo_path(&s.path))
                .collect();
            let mut queue: VecDeque<_> = old_paths.iter().cloned().collect();
            while let Some(path) = queue.pop_front() {
                let bare_targets = if edges.bare_scan_eligible.contains(&path) {
                    edges.bare_out.get(&path)
                } else {
                    None
                };
                for target in edges
                    .ref_out
                    .get(&path)
                    .into_iter()
                    .chain(bare_targets)
                    .flatten()
                {
                    if old_paths.insert(target.clone()) {
                        queue.push_back(target.clone());
                    }
                }
            }
            let mut old_sources: Vec<_> = old_paths
                .iter()
                .map(|path| {
                    lookup
                        .get(path)
                        .expect("historical scanner source provenance")
                        .clone()
                })
                .collect();
            old_sources.sort_by(|a, b| a.path.cmp(&b.path));
            let old_count = old_sources.len();
            let import_count = imports.len();
            let (withheld_passes, withheld_diagnostics) = compile(imports);
            let (historical_passes, historical_diagnostics) = compile(old_sources);
            let partition = match (historical_passes, withheld_passes) {
                (true, false) => "FailureExposedByWithholding",
                (true, true) => "BothPass",
                (false, false) if historical_diagnostics == withheld_diagnostics => {
                    "BothFailIdentically"
                }
                (false, false) => "BothFailDifferently",
                (false, true) => "HistoricalScannerClosureOnlyFails",
            };
            pairs += 1;
            *partitions.entry(partition).or_default() += 1;
            writeln!(report, "{}", serde_json::json!({
                "entry": workspace_relative_repo_path(&entry.path), "partition": partition,
                "import_closure_modules": import_count, "historical_scanner_closure_modules": old_count,
                "withheld_diagnostic_identities": withheld_diagnostics, "historical_diagnostic_identities": historical_diagnostics,
                "new_when_withheld": withheld_diagnostics.difference(&historical_diagnostics).collect::<Vec<_>>(),
                "removed_when_withheld": historical_diagnostics.difference(&withheld_diagnostics).collect::<Vec<_>>(),
            })).unwrap();
            report.flush().unwrap();
            if pairs % 25 == 0 {
                eprintln!(
                    "SCANNER_COMPILE_AUDIT completed_pairs={pairs} partitions={partitions:?}"
                );
            }
        }
        assert!(pairs > 0, "no scanner-addition candidate was compiled");
        assert_eq!(partitions.values().sum::<usize>(), pairs);
        writeln!(
            report,
            "{}",
            serde_json::json!({
            "state": "Completed", "indexed_modules": entries.len(),
            "compared_entry_pairs": pairs, "compiled_distinct_programs": programs.len(), "partitions": partitions,
            })
        )
        .unwrap();
        eprintln!("SCANNER_COMPILE_AUDIT indexed_modules={} compared_entry_pairs={pairs} partitions={partitions:?}", entries.len());
    }
}
