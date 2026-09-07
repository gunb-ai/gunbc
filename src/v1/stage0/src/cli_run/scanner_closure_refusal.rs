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
    fn regeneration_expands_imports_before_refusing_scanner_widening() {
        let corpus = Corpus::new("module scanner.consumer\nimport scanner.provider { scanner_answer }\nfn probe() -> Int { scanner.provider.scanner_answer() }\n");
        let entry_root = corpus.0.join("entry");
        std::fs::create_dir(&entry_root).unwrap();
        std::fs::rename(
            corpus.0.join("consumer.dag"),
            entry_root.join("consumer.dag"),
        )
        .unwrap();
        let index = corpus.index();
        let consumer = index.source_files.get("scanner.consumer").unwrap().clone();
        assert!(
            extend_sources_to_both_closure_fixpoint(vec![consumer], &index).is_err(),
            "raw seeds reproduce the pre-repair regeneration refusal"
        );
        let sources =
            regen_input_sources_over_roots(&entry_root, &[corpus.0.to_string_lossy().into_owned()])
                .unwrap();
        let names: BTreeSet<_> = sources
            .iter()
            .map(|(_, content)| extract_module_path(content).unwrap())
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

    // Invoked only as a child of the paired audit. A fresh process makes an
    // execution deadline enforceable without interrupting compiler state in place.
    #[test]
    #[ignore = "isolated paired-audit worker; requires input and output paths"]
    fn scanner_compile_audit_program() {
        let input = std::env::var("SCANNER_COMPILE_PROGRAM_INPUT").unwrap();
        let output = std::env::var("SCANNER_COMPILE_PROGRAM_OUTPUT").unwrap();
        let paths: Vec<String> = serde_json::from_slice(&std::fs::read(input).unwrap()).unwrap();
        let sources: Vec<_> = paths
            .into_iter()
            .map(|path| {
                let content = std::fs::read_to_string(workspace_root().join(&path)).unwrap();
                Rc::new(SourceFile { path, content })
            })
            .collect();
        assert!(v1_compiler_compile::default_compile_pipeline_options()
            .census_only_sources
            .is_empty());
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
        std::fs::write(output, serde_json::to_vec(&(passes, diagnostics)).unwrap()).unwrap();
    }

    #[test]
    #[ignore = "paired source-compilation audit; requires SCANNER_COMPILE_AUDIT_REPORT"]
    fn real_corpus_withholding_scanner_additions_compiles_both_subjects() {
        use std::io::Write;
        let report_path = std::env::var("SCANNER_COMPILE_AUDIT_REPORT").expect("audit output path");
        let mut report = std::fs::File::create(&report_path).unwrap();
        let budget = |name| {
            let seconds: u64 = std::env::var(name)
                .expect("explicit audit budget")
                .parse()
                .unwrap();
            assert!(seconds > 0, "zero execution budget");
            std::time::Duration::from_secs(seconds)
        };
        let program_budget = budget("SCANNER_COMPILE_PROGRAM_BUDGET_SECONDS");
        let audit_budget = budget("SCANNER_COMPILE_AUDIT_BUDGET_SECONDS");
        let audit_started = std::time::Instant::now();
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
            "program_budget_seconds": program_budget.as_secs(), "audit_budget_seconds": audit_budget.as_secs(),
            "execution_refusals_are_compilation_failures": false,
        })).unwrap();

        // Entries can demand the same complete program (in particular through
        // scanner cycles). Within this immutable source index those are one
        // compilation, even though each entry retains its own comparison row.
        let mut programs = std::collections::HashMap::<
            Vec<String>,
            (usize, Option<(bool, BTreeSet<String>)>),
        >::new();
        let scratch = workspace_root()
            .join("target")
            .join(format!("scanner-program-{}", std::process::id()));
        std::fs::create_dir_all(&scratch).unwrap();
        let input = scratch.join("input.json");
        let output = scratch.join("output.json");
        let log = std::fs::File::create(format!("{report_path}.programs.log")).unwrap();
        let mut program_report = report.try_clone().unwrap();
        let mut compile = |sources: Vec<Rc<SourceFile>>| {
            let key: Vec<_> = sources.iter().map(|source| source.path.clone()).collect();
            if let Some(observed) = programs.get(&key) {
                return observed.clone();
            }
            let program = programs.len();
            writeln!(
                program_report,
                "{}",
                serde_json::json!({
                    "state": "ProgramRequested", "program": program, "sources": key,
                })
            )
            .unwrap();
            program_report.flush().unwrap();
            let mut observation = None;
            let mut exit_status = None;
            let cause;
            if audit_started.elapsed() >= audit_budget {
                cause = "NotAttemptedWithinAuditBudget";
            } else {
                std::fs::write(&input, serde_json::to_vec(&key).unwrap()).unwrap();
                if output.exists() {
                    std::fs::remove_file(&output).unwrap();
                }
                let started = std::time::Instant::now();
                let mut program_log = log.try_clone().unwrap();
                writeln!(program_log, "PROGRAM {program} sources={}", key.len()).unwrap();
                program_log.flush().unwrap();
                let mut child = std::process::Command::new(std::env::current_exe().unwrap())
                    .args([
                        "--exact",
                        "cli_run::scanner_closure_refusal::tests::scanner_compile_audit_program",
                        "--ignored",
                        "--nocapture",
                        "--test-threads=1",
                    ])
                    .env("SCANNER_COMPILE_PROGRAM_INPUT", &input)
                    .env("SCANNER_COMPILE_PROGRAM_OUTPUT", &output)
                    .stdout(log.try_clone().unwrap())
                    .stderr(log.try_clone().unwrap())
                    .spawn()
                    .unwrap();
                loop {
                    if let Some(status) = child.try_wait().unwrap() {
                        exit_status = Some(status.to_string());
                        if status.success() && output.exists() {
                            observation = Some(
                                serde_json::from_slice(&std::fs::read(&output).unwrap()).unwrap(),
                            );
                            cause = "Observed";
                        } else {
                            cause = "ProgramExecutionFailed";
                        }
                        break;
                    }
                    if started.elapsed() >= program_budget
                        || audit_started.elapsed() >= audit_budget
                    {
                        // An exit can race the deadline observation; waiting still reaps it.
                        let _ = child.kill();
                        exit_status = Some(child.wait().unwrap().to_string());
                        cause = "ProgramExecutionTimedOut";
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
            writeln!(program_report, "{}", serde_json::json!({
                "state": "ProgramExecution", "program": program, "cause": cause, "exit_status": exit_status, "observation": observation,
            })).unwrap();
            program_report.flush().unwrap();
            let observed = (program, observation);
            programs.insert(key, observed.clone());
            observed
        };
        let mut pairs = 0;
        let mut partitions = std::collections::BTreeMap::from([
            ("BothPass", 0usize),
            ("FailureExposedByWithholding", 0),
            ("BothFailIdentically", 0),
            ("BothFailDifferently", 0),
            ("HistoricalScannerClosureOnlyFails", 0),
            ("UnobservedComparison", 0),
        ]);
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
            let (withheld_program, withheld) = compile(imports);
            let (historical_program, historical) = compile(old_sources);
            let partition = match (&historical, &withheld) {
                (Some((true, _)), Some((false, _))) => "FailureExposedByWithholding",
                (Some((true, _)), Some((true, _))) => "BothPass",
                (Some((false, a)), Some((false, b))) if a == b => "BothFailIdentically",
                (Some((false, _)), Some((false, _))) => "BothFailDifferently",
                (Some((false, _)), Some((true, _))) => "HistoricalScannerClosureOnlyFails",
                _ => "UnobservedComparison",
            };
            pairs += 1;
            *partitions.entry(partition).or_default() += 1;
            let differences = match (&historical, &withheld) {
                (Some((_, historical_diags)), Some((_, withheld_diags))) => {
                    Some(serde_json::json!({
                        "new_when_withheld": withheld_diags.difference(historical_diags).collect::<Vec<_>>(),
                        "removed_when_withheld": historical_diags.difference(withheld_diags).collect::<Vec<_>>(),
                    }))
                }
                _ => None,
            };
            writeln!(report, "{}", serde_json::json!({
                "entry": workspace_relative_repo_path(&entry.path), "partition": partition,
                "import_closure_modules": import_count, "historical_scanner_closure_modules": old_count,
                "withheld_program": withheld_program, "historical_program": historical_program,
                "withheld_observation": withheld, "historical_observation": historical,
                "diagnostic_differences": differences,
            })).unwrap();
            report.flush().unwrap();
            if pairs % 25 == 0 {
                eprintln!(
                    "SCANNER_COMPILE_AUDIT completed_pairs={pairs} partitions={partitions:?}"
                );
            }
        }
        assert!(pairs > 0, "no scanner-addition candidate was enumerated");
        let unobserved = partitions.get("UnobservedComparison").copied().unwrap_or(0);
        assert_eq!(partitions.values().sum::<usize>(), pairs);
        writeln!(
            report,
            "{}",
            serde_json::json!({
            "state": if unobserved == 0 { "Completed" } else { "IncompleteExecutionCoverage" }, "indexed_modules": entries.len(),
            "candidate_entry_pairs": pairs, "compared_entry_pairs": pairs - unobserved,
            "unobserved_entry_pairs": unobserved, "requested_distinct_programs": programs.len(),
            "observed_distinct_programs": programs.values().filter(|(_, result)| result.is_some()).count(), "partitions": partitions,
            })
        )
        .unwrap();
        std::fs::remove_dir_all(scratch).unwrap();
        eprintln!("SCANNER_COMPILE_AUDIT indexed_modules={} candidate_pairs={pairs} compared_pairs={} unobserved_pairs={unobserved} partitions={partitions:?}", entries.len(), pairs - unobserved);
    }
}
