#[cfg(test)]
mod compiler_tests {
    use crate::v1_compiler_tokenize::tokenize;
    use im::HashMap;

    /// Find workspace root by walking up from the current directory looking for Cargo.toml + dag/
    fn workspace_root() -> std::path::PathBuf {
        let mut dir = std::env::current_dir().expect("no current dir");
        loop {
            if dir.join("Cargo.toml").exists() && dir.join("dag").exists() {
                return dir;
            }
            if !dir.pop() {
                panic!("could not find workspace root (no Cargo.toml + dag/ found)");
            }
        }
    }

    /// Read a .dag file from the workspace
    fn read_dag(path: &str) -> String {
        let full = workspace_root().join(path);
        std::fs::read_to_string(&full)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", full.display(), e))
    }

    /// Discover all .dag files under a directory, returning (relative_path, content) pairs.
    /// The relative path is relative to the workspace root.
    fn discover_dag_files(dir: &str) -> Vec<(String, String)> {
        let root = workspace_root();
        let base = root.join(dir);
        let mut results = Vec::new();
        collect_dag_recursive(&base, &root, &mut results);
        results.sort_by(|a, b| a.0.cmp(&b.0));
        results
    }

    fn collect_dag_recursive(
        dir: &std::path::Path,
        root: &std::path::Path,
        out: &mut Vec<(String, String)>,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_dag_recursive(&path, root, out);
                } else if path.extension().map_or(false, |e| e == "dag") {
                    let rel = path
                        .strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .to_string();
                    // Test fixtures (e.g. fact_cardinality_split_brace.dag) are not modules.
                    if rel.contains("/tests/") {
                        continue;
                    }
                    let content = std::fs::read_to_string(&path)
                        .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
                    out.push((rel, content));
                }
            }
        }
    }

    /// Build SourceFile vec from discovered .dag files.
    fn source_files_from(
        pairs: &[(String, String)],
    ) -> Vec<std::rc::Rc<crate::v1_compiler_compile::SourceFile>> {
        pairs
            .iter()
            .map(|(path, content)| {
                std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: path.clone(),
                    content: content.clone(),
                })
            })
            .collect()
    }

    fn parse_module_or_panic(path: &str, content: &str) -> std::rc::Rc<crate::v1_std_core::Node> {
        let tokens = tokenize(content.to_string(), path.to_string());
        let mut source_indices = HashMap::new();
        source_indices.insert(
            path.to_string(),
            crate::v1_std_core::build_newline_index(path.to_string(), content.to_string()),
        );
        let parsed = crate::v1_compiler_parse::parse_with_table(
            tokens.clone(),
            std::rc::Rc::new(source_indices),
            crate::v1_std_core::empty_intern_table(),
        );
        if let Some(err) = parsed.result.error.as_ref() {
            panic!(
                "failed to parse {} while building source closure: {}",
                path,
                crate::v1_std_core::diagnostic_to_message(err.diagnostic.clone())
            );
        }
        parsed
            .result
            .module
            .clone()
            .unwrap_or_else(|| panic!("{} produced no module while building source closure", path))
    }

    fn module_path_from_source(path: &str, content: &str) -> String {
        parse_module_or_panic(path, content).name.clone()
    }

    fn import_paths_from_source(path: &str, content: &str) -> Vec<String> {
        let module = parse_module_or_panic(path, content);
        crate::v1_std_core::module_imports(module)
            .iter()
            .map(|imp| imp.name.clone())
            .collect()
    }

    fn build_source_index(roots: &[&str]) -> HashMap<String, (String, String)> {
        let mut index = HashMap::new();
        for root in roots {
            for (path, content) in discover_dag_files(root) {
                let module_path = module_path_from_source(&path, &content);
                if let Some((existing, _)) = index.get(&module_path) {
                    panic!(
                        "duplicate module path '{}': declared in both {} and {}",
                        module_path, existing, path
                    );
                }
                index.insert(module_path, (path, content));
            }
        }
        index
    }

    fn resolve_source_closure(
        entry_pairs: Vec<(String, String)>,
        roots: &[&str],
    ) -> Vec<std::rc::Rc<crate::v1_compiler_compile::SourceFile>> {
        let index = build_source_index(roots);
        let mut seen =
            HashMap::<String, std::rc::Rc<crate::v1_compiler_compile::SourceFile>>::new();
        let mut queue = Vec::new();

        for (path, content) in entry_pairs {
            let module_path = module_path_from_source(&path, &content);
            seen.insert(
                module_path,
                std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: path.clone(),
                    content: content.clone(),
                }),
            );
            queue.push((path, content));
        }

        while let Some((_path, content)) = queue.pop() {
            for module_path in import_paths_from_source(&_path, &content) {
                if seen.contains_key(&module_path) {
                    continue;
                }
                if let Some((path, file_content)) = index.get(&module_path).cloned() {
                    seen.insert(
                        module_path,
                        std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                            path: path.clone(),
                            content: file_content.clone(),
                        }),
                    );
                    queue.push((path, file_content));
                }
            }
        }

        let mut result: Vec<_> = seen.into_iter().map(|(_, v)| v).collect();
        result.sort_by(|a, b| a.path.cmp(&b.path));
        result
    }

    /// Build the self-compile source closure from src/v1 entry modules with dag as a dependency pool.
    fn self_compile_sources() -> Vec<std::rc::Rc<crate::v1_compiler_compile::SourceFile>> {
        resolve_source_closure(discover_dag_files("src/v1"), &["src/v1", "dag"])
    }

    #[test]
    fn tokenize_produces_tokens() {
        let tokens = tokenize("fn foo() -> Int { 42 }".to_string(), "test.dag".to_string());
        assert!(
            !tokens.is_empty(),
            "tokenize should produce at least one token"
        );
    }

    #[test]
    fn tokenize_ends_with_eof() {
        let tokens = tokenize("type Foo { x: Int }".to_string(), "test.dag".to_string());
        let last = tokens.last().expect("should have tokens");
        assert!(
            matches!(last.shape, crate::v1_std_core::TokenShape::ShEof),
            "last token should be Eof, got {:?}",
            last.shape
        );
    }

    #[test]
    fn tokenize_fn_keyword() {
        let tokens = tokenize("fn".to_string(), "test.dag".to_string());
        assert!(
            tokens.len() >= 2,
            "expected at least 2 tokens, got {}",
            tokens.len()
        );
        assert!(
            matches!(tokens[0].shape, crate::v1_std_core::TokenShape::ShKeyword),
            "first token should be KwFn, got {:?}",
            tokens[0].shape
        );
    }

    #[test]
    fn tokenize_count_stable() {
        let tokens = tokenize(
            "module test\ntype Foo { x: Int }".to_string(),
            "test.dag".to_string(),
        );
        assert!(
            tokens.len() > 5,
            "non-trivial input should produce multiple tokens, got {}",
            tokens.len()
        );
    }

    #[test]
    fn parse_trivial_module() {
        let tokens = tokenize(
            "module test\ntype Foo { x: Int }\n".to_string(),
            "test.dag".to_string(),
        );
        let result = crate::v1_compiler_parse::parse(tokens, std::rc::Rc::new(im::HashMap::new()));
        assert!(
            result.module.is_some(),
            "valid module should parse successfully"
        );
    }

    #[test]
    fn self_parse_tokenize_dag() {
        let result = std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let source = read_dag("src/v1/01_tokenize.dag");
                let tokens = tokenize(source, "src/v1/01_tokenize.dag".to_string());

                assert!(
                    !tokens.is_empty(),
                    "tokenizing 01_tokenize.dag should produce tokens"
                );

                let last = tokens.last().expect("should have tokens");
                assert!(
                    matches!(last.shape, crate::v1_std_core::TokenShape::ShEof),
                    "last token should be Eof, got {:?}",
                    last.shape
                );

                let result =
                    crate::v1_compiler_parse::parse(tokens, std::rc::Rc::new(im::HashMap::new()));

                assert!(
                    result.module.is_some(),
                    "parsing 01_tokenize.dag should produce a module"
                );

                let module = result.module.as_ref().unwrap();
                assert_eq!(
                    module.name, "v1.compiler.tokenize",
                    "module name should be v1.compiler.tokenize, got {}",
                    module.name
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("self-parse test panicked");
    }

    #[test]
    fn pipeline_trivial_module() {
        let result = std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let source = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "test.dag".to_string(),
                    content: "module test\ntype Foo { x: Int, name: String }\nfn add(a: Int, b: Int) -> Int { a + b }\n".to_string(),
                });
                let result = crate::v1_compiler_compile::compile_sources(std::rc::Rc::new(im::vector![source]), crate::v1_compiler_artifact::RenderTarget::Rust);

                assert!(
                    !result.files.is_empty(),
                    "compile_sources should produce output files, got none"
                );

                let errors: Vec<_> = result.diagnostics.iter()
                    .filter(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone()))
                    .collect();
                assert!(
                    errors.is_empty(),
                    "compile_sources should produce no errors, got {:?}",
                    errors
                );

                let has_content = result.files.iter().any(|f| !f.content.is_empty());
                assert!(
                    has_content,
                    "at least one output file should have non-empty content"
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("pipeline_trivial_module test panicked");
    }

    #[test]
    fn pipeline_occurrence_sidecar_serde_round_trip() {
        let result = std::thread::Builder::new()
            .stack_size(16 * 1024 * 1024)
            .spawn(|| {
                let left_source = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "occurrence_sidecar_left.dag".to_string(),
                    content: "module occurrence.sidecar_left\nfn shared(x: Int) -> Int { x }\n".to_string(),
                });
                let right_source = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "occurrence_sidecar_right.dag".to_string(),
                    content: "module occurrence.sidecar_right\nfn shared(x: Int) -> Int { x }\n".to_string(),
                });
                let mut expected_source_indices = im::HashMap::new();
                expected_source_indices.insert(
                    left_source.path.clone(),
                    crate::v1_std_core::build_newline_index(
                        left_source.path.clone(),
                        left_source.content.clone(),
                    ),
                );
                let expected_left = crate::v1_compiler_parse::parse_with_table_in_occurrence_scope(
                    crate::v1_compiler_tokenize::tokenize(
                        left_source.content.clone(),
                        left_source.path.clone(),
                    ),
                    std::rc::Rc::new(expected_source_indices),
                    crate::v1_std_core::empty_intern_table(),
                    crate::std_occurrence_identity::occurrence_id_allocator_initial(),
                );
                assert!(expected_left.result.error.is_none());
                let resolved = crate::v1_compiler_compile::compile_to_resolved(
                    std::rc::Rc::new(im::vector![left_source.clone(), right_source]),
                );
                let graph = resolved
                    .graph
                    .as_ref()
                    .expect("production compile must preserve a resolved graph");
                assert_eq!(graph.modules.len(), 2);
                let mut occurrence_ids = std::collections::HashSet::new();
                for typed_module in graph.modules.iter() {
                    let transport = typed_module
                        .occurrence_transport
                        .as_ref()
                        .expect("typed module must preserve occurrence transport");
                    assert!(!transport.index.entries.is_empty());
                    assert!(!transport.declarations.is_empty());
                    assert!(!transport.references.is_empty());
                    for entry in transport.index.entries.iter() {
                        assert!(
                            occurrence_ids.insert(entry.projection.occurrence.value),
                            "graph-scoped occurrence ids must remain disjoint across modules"
                        );
                    }
                    let module_bytes = serde_json::to_vec(typed_module)
                        .expect("serialize typed module occurrence sidecar");
                    let decoded_module: std::rc::Rc<crate::v1_compiler_infer_items::TypedModule> =
                        serde_json::from_slice(&module_bytes)
                            .expect("deserialize typed module occurrence sidecar");
                    assert_eq!(&decoded_module, typed_module);
                }

                let rebuilt_left = graph.modules.iter()
                    .filter_map(|typed_module| typed_module.occurrence_transport.as_ref())
                    .find(|transport| transport.index.entries.iter().any(|entry| {
                        entry.projection.diagnostic_span.file == left_source.path
                    }))
                    .expect("compiled left module must retain its occurrence sidecar");
                assert_eq!(
                    rebuilt_left,
                    &expected_left.occurrence_transport,
                    "production reference rebuild must preserve exact sidecar ids and containment paths"
                );

                let graph_bytes = serde_json::to_vec(graph)
                    .expect("serialize resolved graph occurrence sidecar");
                let decoded_graph: std::rc::Rc<crate::v1_compiler_infer_items::ResolvedGraph> =
                    serde_json::from_slice(&graph_bytes)
                        .expect("deserialize resolved graph occurrence sidecar");
                assert_eq!(&decoded_graph, graph);
                for (before, after) in graph.modules.iter().zip(decoded_graph.modules.iter()) {
                    assert_eq!(after.occurrence_transport, before.occurrence_transport);
                }

                let resolved_bytes = serde_json::to_vec(&resolved)
                    .expect("serialize resolved pipeline occurrence sidecar");
                let decoded: std::rc::Rc<crate::v1_compiler_compile::ResolvedPipelineResult> =
                    serde_json::from_slice(&resolved_bytes)
                        .expect("deserialize resolved pipeline occurrence sidecar");
                assert_eq!(decoded, resolved);
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("pipeline_occurrence_sidecar_serde_round_trip panicked");
    }

    #[test]
    fn call_shape_wall_witness() {
        // DISCRIMINATING RED for direct_call_shape_wall_note (04_infer). Before the
        // wall, `sub(a: 10, bb: 3)` against `fn sub(a: Int, b: Int)` compiled with
        // ZERO diagnostics: the per-param type walk absorbed the mislabeled arg into
        // its position, the interpreter refused the same call at runtime
        // (CallContractMismatch in call_function_inner), and the Rust emitter
        // silently REORDERED it positionally — two realizations of one program
        // disagreeing silently. The wall makes the compile seam agree with the
        // runtime authority, on both of the classes the runtime refuses.
        let result = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let compile_one = |path: &str, content: &str| {
                    crate::v1_compiler_compile::compile_sources(
                        std::rc::Rc::new(im::vector![std::rc::Rc::new(
                            crate::v1_compiler_compile::SourceFile {
                                path: path.to_string(),
                                content: content.to_string(),
                            }
                        )]),
                        crate::v1_compiler_artifact::RenderTarget::Rust,
                    )
                };
                let mislabeled = compile_one(
                    "mislabel.dag",
                    "module mislabel\nfn sub(a: Int, b: Int) -> Int { a - b }\nfn f() -> Int { sub(a: 10, bb: 3) }\n",
                );
                let unknown: Vec<_> = mislabeled.diagnostics.iter()
                    .filter(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::CallArgumentNameUnknown { .. }))
                    .collect();
                assert!(
                    !unknown.is_empty(),
                    "an argument labeling a parameter that does not exist must refuse at the compile seam — the interpreter already refuses this call at runtime, and the emitter silently reorders it, got: {:?}",
                    mislabeled.diagnostics
                );
                assert!(
                    unknown.iter().all(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone())
                        && crate::v1_std_core::is_interpreter_blocking_diagnostic(d.diagnostic.clone())),
                    "CallArgumentNameUnknown must BLOCK — a counted advisory would still emit the silently-reordered realization"
                );
                let surplus = compile_one(
                    "surplus.dag",
                    "module surplus\nfn two(a: Int, b: Int) -> Int { a + b }\nfn f() -> Int { two(1, 2, 3) }\n",
                );
                assert!(
                    surplus.diagnostics.iter().any(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::CallPositionalSurplus { .. })),
                    "a positional argument beyond the declared positional parameters must refuse — the interpreter refuses the same call (too many positional arguments), got: {:?}",
                    surplus.diagnostics
                );
                // POSITIVE CONTROLS at ZERO diagnostics of any severity (the filtering
                // lesson of codex review 45357: asserting only the absence of the
                // blocking variant lets an advisory pass unnoticed). Correct labels,
                // correct positional binding, and the deliberately-unused-parameter
                // idiom (label `ctx` against declared `_ctx` — the interpreter accepts
                // it, so the compile seam must too) all stay silent.
                let green = compile_one(
                    "green.dag",
                    "module green\nfn sub(a: Int, b: Int) -> Int { a - b }\nfn ignore_ctx(_ctx: Int, b: Int) -> Int { b }\nfn named() -> Int { sub(a: 10, b: 3) }\nfn positional() -> Int { sub(10, 3) }\nfn underscore_idiom() -> Int { ignore_ctx(ctx: 1, b: 2) }\n",
                );
                assert!(
                    green.diagnostics.is_empty(),
                    "correct call shapes must compile with NO diagnostic of any severity, got: {:?}",
                    green.diagnostics
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("call_shape_wall_witness panicked");
    }

    #[test]
    fn call_deficit_red_witness() {
        let result = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let compile_one = |path: &str, content: &str| {
                    crate::v1_compiler_compile::compile_sources(
                        std::rc::Rc::new(im::vector![std::rc::Rc::new(
                            crate::v1_compiler_compile::SourceFile {
                                path: path.to_string(),
                                content: content.to_string(),
                            }
                        )]),
                        crate::v1_compiler_artifact::RenderTarget::Rust,
                    )
                };
                let deficit = compile_one(
                    "deficit.dag",
                    "module deficit\nfn two(a: Int, b: Int) -> Int { a + b }\nfn f() -> Int { two(1) }\n",
                );
                assert!(
                    deficit.diagnostics.iter().any(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::CallPositionalDeficit { .. })),
                    "a call supplying fewer required arguments than declared must refuse — the interpreter refuses the same call (missing required argument), got: {:?}",
                    deficit.diagnostics
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("call_deficit_red_witness panicked");
    }

    #[test]
    fn call_shape_duplicate_wall_witness() {
        // DISCRIMINATING RED for the duplicate-label half of
        // call-missing-and-duplicate-wall (roadmap rn_EXUHLON5V24YU7Z4XLJFWEZO33,
        // first_slice). Before the wall, `two(a: 1, a: 2)` against
        // `fn two(a: Int, b: Int)` compiled with ZERO diagnostics while the runtime
        // authority (call_function_inner's bindings.insert) silently overwrote the
        // first binding — the second `a` value winning with no trace of the first —
        // and the interpreter now refuses the same call (CallContractMismatch). The
        // wall makes the compile seam agree with that runtime refusal, mirroring
        // the CallArgumentNameUnknown/CallPositionalSurplus wall above on the third
        // of the four bijection failure modes.
        let result = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let compile_one = |path: &str, content: &str| {
                    crate::v1_compiler_compile::compile_sources(
                        std::rc::Rc::new(im::vector![std::rc::Rc::new(
                            crate::v1_compiler_compile::SourceFile {
                                path: path.to_string(),
                                content: content.to_string(),
                            }
                        )]),
                        crate::v1_compiler_artifact::RenderTarget::Rust,
                    )
                };
                let duplicate = compile_one(
                    "duplicate.dag",
                    "module duplicate\nfn two(a: Int, b: Int) -> Int { a + b }\nfn f() -> Int { two(a: 1, a: 2) }\n",
                );
                let dup: Vec<_> = duplicate.diagnostics.iter()
                    .filter(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::CallArgumentDuplicate { .. }))
                    .collect();
                assert!(
                    !dup.is_empty(),
                    "a caller label supplied more than once must refuse at the compile seam — the interpreter already refuses this call at runtime, got: {:?}",
                    duplicate.diagnostics
                );
                assert!(
                    dup.iter().all(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone())
                        && crate::v1_std_core::is_interpreter_blocking_diagnostic(d.diagnostic.clone())),
                    "CallArgumentDuplicate must BLOCK — a counted advisory would still emit the silently-overwritten realization"
                );
                // Runtime authority: the same duplicate-label call must refuse via
                // call_function_inner (InterpError::CallContractMismatch), never
                // silently pick the last-bound value. Single-parameter fixture,
                // deliberately: a second declared-but-unlabeled parameter (e.g. `b`)
                // would stay unbound whether or not the duplicate check fires, so a
                // two-param fixture's `run.is_err()` is satisfied by an unrelated
                // `NoSuchVariable` from reading that unbound param — a false-discriminator
                // caught by mutation testing (removing the duplicate-check block left
                // this test green for the wrong reason). With one parameter, disabling
                // the check leaves `bindings` fully populated (last-write-wins) and `run`
                // succeeds, so the test only reds when the duplicate check itself fires.
                let resolved = crate::v1_compiler_compile::compile_to_resolved(
                    std::rc::Rc::new(im::vector![std::rc::Rc::new(
                        crate::v1_compiler_compile::SourceFile {
                            path: "duplicate_rt.dag".to_string(),
                            content: "module duplicate_rt\nfn two(a: Int) -> Int { a }\nfn f() -> Int { two(a: 1, a: 2) }\n".to_string(),
                        }
                    )]),
                );
                let graph = resolved.graph.clone().expect("duplicate-label fixture must resolve — it is a runtime-authority probe, not a compile-seam one");
                let run = crate::v1_interpreter::run(&graph, resolved.source_indices.clone(), "f");
                // Assert the typed outcome, never the polarity: match the specific
                // CallContractMismatch variant AND its duplicate-specific detail text,
                // since the same variant also covers unknown-label and
                // positional-surplus refusals at other sites in call_function_inner —
                // bare `run.is_err()` would pass under any of those unrelated causes too.
                match &run {
                    Err(crate::v1_interpreter::InterpError::CallContractMismatch { detail, .. })
                        if detail.contains("supplied more than once") => {}
                    other => panic!(
                        "the runtime authority must refuse a duplicate-label call with CallContractMismatch{{detail: \"argument 'a' supplied more than once\"}}, never silently overwrite the earlier binding, got: {:?}",
                        other
                    ),
                }
                // Runtime authority, NAMED-THEN-POSITIONAL shape (review 48817): a named
                // actual and a later positional actual can resolve to the SAME declared
                // parameter (`two(a: 1, 2)` — the named `a` binds param `a`, and the lone
                // positional value fills positional slot 0, whose declared name is also
                // `a`). The positional insert branch used to skip the collision check the
                // named branch already had, so this call silently overwrote `a` and
                // succeeded. Single-parameter fixture again, deliberately: a second
                // declared param would go unbound regardless of whether the duplicate
                // check fires, producing an unrelated "missing required argument" error
                // that would mask the true defect (the same false-discriminator this
                // file's named+named check above already guards against).
                let resolved_np = crate::v1_compiler_compile::compile_to_resolved(
                    std::rc::Rc::new(im::vector![std::rc::Rc::new(
                        crate::v1_compiler_compile::SourceFile {
                            path: "duplicate_named_then_positional_rt.dag".to_string(),
                            content: "module duplicate_named_then_positional_rt\nfn two(a: Int) -> Int { a }\nfn f() -> Int { two(a: 1, 2) }\n".to_string(),
                        }
                    )]),
                );
                let graph_np = resolved_np.graph.clone().expect("named-then-positional duplicate fixture must resolve — it is a runtime-authority probe, not a compile-seam one");
                let run_np = crate::v1_interpreter::run(&graph_np, resolved_np.source_indices.clone(), "f");
                match &run_np {
                    Err(crate::v1_interpreter::InterpError::CallContractMismatch { detail, .. })
                        if detail.contains("supplied more than once") => {}
                    other => panic!(
                        "a named actual and a later positional actual resolving to the same declared parameter must refuse with CallContractMismatch{{detail: \"argument 'a' supplied more than once\"}}, never silently overwrite, got: {:?}",
                        other
                    ),
                }
                // POSITIVE CONTROLS at ZERO diagnostics: distinct labels, positional
                // args, the reordered-named-args idiom, and the deliberately-unused
                // underscore idiom (two DIFFERENT surface labels for the SAME
                // parameter, `x` and `_x`/`_`, are not a duplicate — only exact
                // caller-label equality is, matching the runtime HashMap::insert
                // condition exactly) all stay silent.
                let green = compile_one(
                    "green_dup.dag",
                    "module green_dup\nfn two(a: Int, b: Int) -> Int { a + b }\nfn ignore_ctx(_ctx: Int, b: Int) -> Int { b }\nfn distinct() -> Int { two(a: 1, b: 2) }\nfn reordered() -> Int { two(b: 2, a: 1) }\nfn positional() -> Int { two(1, 2) }\nfn underscore_idiom() -> Int { ignore_ctx(ctx: 1, b: 2) }\n",
                );
                assert!(
                    green.diagnostics.is_empty(),
                    "distinct labels, positional args, reordered named args, and the underscore idiom must compile with NO diagnostic of any severity, got: {:?}",
                    green.diagnostics
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("call_shape_duplicate_wall_witness panicked");
    }

    #[test]
    fn function_value_named_application_controls_witness() {
        // Operator-required controls for higher-order named application (P0).
        // Direct declaration calls keep named args; function-value calls are positional-only.
        let result = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let compile_one = |path: &str, content: &str| {
                    crate::v1_compiler_compile::compile_sources(
                        std::rc::Rc::new(im::vector![std::rc::Rc::new(
                            crate::v1_compiler_compile::SourceFile {
                                path: path.to_string(),
                                content: content.to_string(),
                            }
                        )]),
                        crate::v1_compiler_artifact::RenderTarget::Rust,
                    )
                };
                // 1. Direct declaration call with reordered named arguments -> ADMIT
                let direct_reordered = compile_one(
                    "direct_reordered.dag",
                    "module direct_reordered\nfn sub(a: Int, b: Int) -> Int { a - b }\nfn witness() -> Int { sub(b: 3, a: 10) }\n",
                );
                assert!(
                    direct_reordered.diagnostics.is_empty(),
                    "direct declaration with reordered named args must ADMIT, got: {:?}",
                    direct_reordered.diagnostics
                );
                // 2. Higher-order callback declared left/right, applied positionally -> ADMIT
                let hof_positional = compile_one(
                    "hof_positional.dag",
                    "module hof_positional\nfn cmp(left: Int, right: Int) -> Bool { left < right }\nfn host(agree: fn(Int, Int) -> Bool) -> Bool { agree(1, 2) }\nfn witness() -> Bool { host(cmp) }\n",
                );
                assert!(
                    hof_positional.diagnostics.is_empty(),
                    "positional function-value application must ADMIT, got: {:?}",
                    hof_positional.diagnostics
                );
                // 3. Higher-order named application without labeled function type -> REFUSE
                let named_on_value = compile_one(
                    "named_on_value.dag",
                    "module named_on_value\nfn host(agree: fn(Int, Int) -> Bool) -> Bool { agree(a: 1, b: 2) }\n",
                );
                let named: Vec<_> = named_on_value.diagnostics.iter()
                    .filter(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::CallNamedArgOnFunctionValue { .. }))
                    .collect();
                assert!(
                    named.len() >= 2,
                    "named args on function-value call must REFUSE, got: {:?}",
                    named_on_value.diagnostics
                );
                assert!(
                    named.iter().all(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone())
                        && crate::v1_std_core::is_interpreter_blocking_diagnostic(d.diagnostic.clone())),
                    "CallNamedArgOnFunctionValue must BLOCK"
                );
                // 4. Wrong higher-order arity -> REFUSE
                let wrong_arity = compile_one(
                    "wrong_arity.dag",
                    "module wrong_arity\nfn host(agree: fn(Int, Int) -> Bool) -> Bool { agree(1, 2, 3) }\n",
                );
                assert!(
                    wrong_arity.diagnostics.iter().any(|d| matches!(
                        *d.diagnostic,
                        crate::v1_std_core::CompilerDiagnostic::CallPositionalSurplus { .. }
                    )),
                    "surplus args on function-value call must REFUSE, got: {:?}",
                    wrong_arity.diagnostics
                );
                // 5. Swapped positional callback arguments -> semantic RED (compile admits; order matters)
                let semantic = compile_one(
                    "semantic_swap.dag",
                    "module semantic_swap\nfn cmp(left: Int, right: Int) -> Bool { left < right }\nfn host(agree: fn(Int, Int) -> Bool, a: Int, b: Int) -> Bool { agree(a, b) }\nfn correct_order() -> Bool { host(cmp, 1, 2) }\nfn swapped_order() -> Bool { host(cmp, 2, 1) }\n",
                );
                assert!(
                    semantic.diagnostics.is_empty(),
                    "swapped positional controls must compile clean for semantic RED, got: {:?}",
                    semantic.diagnostics
                );
                let resolved = crate::v1_compiler_compile::compile_to_resolved(
                    std::rc::Rc::new(im::vector![std::rc::Rc::new(
                        crate::v1_compiler_compile::SourceFile {
                            path: "semantic_swap.dag".to_string(),
                            content: "module semantic_swap\nfn cmp(left: Int, right: Int) -> Bool { left < right }\nfn host(agree: fn(Int, Int) -> Bool, a: Int, b: Int) -> Bool { agree(a, b) }\nfn correct_order() -> Bool { host(cmp, 1, 2) }\nfn swapped_order() -> Bool { host(cmp, 2, 1) }\n".to_string(),
                        }
                    )]),
                );
                let graph = resolved.graph.as_ref().expect("graph");
                let ctx = crate::cli_run::make_eval_context(
                    graph,
                    resolved.source_indices.clone(),
                    crate::v1_interpreter::ExecutionMode::Wet,
                );
                let correct = crate::v1_interpreter::run_in_context(&ctx, "correct_order", false)
                    .expect("correct_order should run");
                let swapped = crate::v1_interpreter::run_in_context(&ctx, "swapped_order", false)
                    .expect("swapped_order should run");
                assert!(
                    matches!(correct, crate::v1_interpreter::Value::Bool(true)),
                    "correct positional order must be true, got {:?}",
                    correct
                );
                assert!(
                    matches!(swapped, crate::v1_interpreter::Value::Bool(false)),
                    "swapped positional order must be false — semantic RED proving bind order, got {:?}",
                    swapped
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("function_value_named_application_controls_witness panicked");
    }

    #[test]
    fn function_value_field_method_known_hole_probe() {
        // KNOWN-HOLE PROBE (not a desired-behavior control): coverage path (5) in
        // function_value_named_application_wall_note. cfg.callback(a:, b:) parses as
        // ExprMethodCall (make_call_expr on ExprFieldAccess), bypasses BOTH the
        // body_locals wall and #7519 direct_call_shape_diags. Today it compiles clean
        // with named actuals on a record-field function value. When the method-call
        // argument-label wall lands, this probe must FLIP (refusal) and become a
        // permanent regression control per DESIGN §4b(4).
        let field_named = crate::v1_compiler_compile::compile_sources(
            std::rc::Rc::new(im::vector![std::rc::Rc::new(
                crate::v1_compiler_compile::SourceFile {
                    path: "field_method_named_hole.dag".to_string(),
                    content: "module field_method_named_hole\ntype Cfg { callback: fn(Int, Int) -> Bool }\nfn cmp(left: Int, right: Int) -> Bool { left < right }\nfn witness() -> Bool { host(Cfg { callback: cmp }) }\nfn host(cfg: Cfg) -> Bool { cfg.callback(a: 1, b: 2) }\n".to_string(),
                }
            )]),
            crate::v1_compiler_artifact::RenderTarget::Rust,
        );
        assert!(
            field_named.diagnostics.is_empty(),
            "KNOWN HOLE today: field-held fn via method syntax with named actuals must compile clean until the method-call label wall lands, got: {:?}",
            field_named.diagnostics
        );
        assert!(
            !field_named.diagnostics.iter().any(|d| matches!(
                *d.diagnostic,
                crate::v1_std_core::CompilerDiagnostic::CallNamedArgOnFunctionValue { .. }
            )),
            "this path is ExprMethodCall — CallNamedArgOnFunctionValue must not fire here"
        );
    }

    #[test]
    fn method_existence_wall_witness() {
        // DISCRIMINATING RED for method_existence_wall_note. Before the wall an
        // unresolved method inherited the RECEIVER's type with no diagnostic, so
        // `xs |> filter_map(..)` on List<Int> typed as List<Int>, compiled clean, and
        // died at InterpError::Unimplemented in live dispatch (#7479, HTTP 500).
        let result = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let red = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "red.dag".to_string(),
                    content: "module red\nfn f(xs: List<Int>) -> List<Int> { xs |> filter_map(x => x) }\nfn g(xs: List<Int>) -> Bool { xs |> starts_with(\"x\") }\nfn h(xs: List<Int>) -> String { xs |> to_upper() }\n".to_string(),
                });
                let red_result = crate::v1_compiler_compile::compile_sources(
                    std::rc::Rc::new(im::vector![red]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                let missing: Vec<_> = red_result.diagnostics.iter()
                    .filter(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::MethodNotFound { .. }))
                    .collect();
                assert!(
                    missing.len() >= 3,
                    "expected MethodNotFound for the unresolved method AND for rostered names on a receiver that does not offer them (starts_with / to_upper on List<Int> — codex review 45327: a name-grain predicate admitted these), got: {:?}",
                    red_result.diagnostics
                );
                // POSITIVE CONTROLS the wall must not touch: algebra method templates
                // resolve at tier0, and `count` is in the declared std.methods roster
                // even though free_monoid_scalar_templates omits it (the measured fork).
                let green = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "green.dag".to_string(),
                    content: "module green\nfn p(xs: List<Int>) -> List<Int> { xs |> filter(x => x > 1) |> map(x => x + 1) }\nfn q(xs: List<Int>) -> Int { xs |> fold(0, (a, x) => a + x) }\nfn r(s: String) -> Int { s |> count }\n".to_string(),
                });
                let green_result = crate::v1_compiler_compile::compile_sources(
                    std::rc::Rc::new(im::vector![green]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                // The positive control asserts ZERO diagnostics, not merely no
                // MethodNotFound. Filtering to the blocking variant let an ADVISORY
                // MethodExistenceUndecided pass unnoticed, which is how `String |> count`
                // was reported as a passing control while it was actually resolving
                // through the non-blocking arm (codex review 45357).
                assert!(
                    green_result.diagnostics.is_empty(),
                    "legitimate methods must resolve with NO diagnostic of any severity — an advisory here means the call is not actually resolving, got: {:?}",
                    green_result.diagnostics
                );
                // DISCRIMINATING RED for the declared frontier (codex reviews 45357,
                // 45383). An undecided method existence must BLOCK, so the graph is
                // never emitted on an unestablished judgment; the only non-blocking
                // path is a DECLARED frontier row, which is countable and carries its
                // own dissolution trigger. The two halves must be discriminated by
                // MODULE NAME on otherwise identical source — an admission that fires
                // for any module is an escape hatch, not a frontier.
                let frontier_src = |module: &str, method: &str| {
                    format!("module {}\ntype T {{ a: Int }}\nfn f(t: T) -> Int {{ t |> {}(1) }}\n", module, method)
                };
                let compile_one = |path: &str, content: String| {
                    crate::v1_compiler_compile::compile_sources(
                        std::rc::Rc::new(im::vector![std::rc::Rc::new(
                            crate::v1_compiler_compile::SourceFile {
                                path: path.to_string(),
                                content,
                            }
                        )]),
                        crate::v1_compiler_artifact::RenderTarget::Rust,
                    )
                };
                let unlisted = compile_one("unlisted.dag", frontier_src("unlisted.module", "list_push"));
                let undecided: Vec<_> = unlisted.diagnostics.iter()
                    .filter(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::MethodExistenceUndecided { .. }))
                    .collect();
                assert!(
                    !undecided.is_empty(),
                    "a method whose existence is undecided in a module with NO declared frontier row must refuse, got: {:?}",
                    unlisted.diagnostics
                );
                assert!(
                    undecided.iter().all(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone())
                        && crate::v1_std_core::is_interpreter_blocking_diagnostic(d.diagnostic.clone())),
                    "MethodExistenceUndecided must BLOCK both typecheck and the interpreter — a non-blocking undecided judgment is exactly the fail-open the wall exists to close"
                );
                // The SHAPE is part of the key, and this is the half that answers
                // codex review 45398: the module and the method both match a declared
                // row, and the call is STILL refused, because the receiver does not
                // fail to resolve in the way the row was measured on. A (module, method)
                // key passed this input; that is the fail-open the third component closes.
                let listed_wrong_shape = compile_one("listed.dag", frontier_src("extdeps.dns.domain_name", "list_push"));
                assert!(
                    listed_wrong_shape.diagnostics.iter().any(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::MethodExistenceUndecided { .. })),
                    "a declared row must NOT admit a new call in the same module on a receiver whose shape it was never measured on, got: {:?}",
                    listed_wrong_shape.diagnostics
                );
                // The admission half, checked against the frontier data itself rather
                // than a synthetic source, because the residual shapes (an unnamed
                // lambda-parameter receiver, a coproduct payload typed as its variant)
                // are upstream resolution defects that cannot be conjured on demand.
                // Every declared row must be admitted by its own exact key, and
                // perturbing ANY of the three components must withdraw the admission.
                let rows = crate::v1_compiler_infer::unresolved_method_frontier();
                assert!(!rows.is_empty(), "the frontier must be a declared, countable roster");
                for r in rows.iter() {
                    assert!(
                        crate::v1_compiler_infer::unresolved_method_frontier_trigger(
                            r.module_name.clone(), r.method.clone(), r.receiver_shape.clone()).is_some(),
                        "row {}/{}/{} must be admitted by its own key", r.module_name, r.method, r.receiver_shape
                    );
                    for perturbed in [
                        (format!("{}.other", r.module_name), r.method.clone(), r.receiver_shape.clone()),
                        (r.module_name.clone(), format!("{}_other", r.method), r.receiver_shape.clone()),
                        (r.module_name.clone(), r.method.clone(), format!("Product(Other{})", r.receiver_shape.len())),
                    ] {
                        assert!(
                            crate::v1_compiler_infer::unresolved_method_frontier_trigger(
                                perturbed.0.clone(), perturbed.1.clone(), perturbed.2.clone()).is_none(),
                            "perturbing the key to {:?} must WITHDRAW the admission — each component is load-bearing", perturbed
                        );
                    }
                }
                // DISCRIMINATING RED for where_refinement_receiver_peel_note. A
                // where-refinement alias reached method lookup as Product(<alias>)
                // because resolve_method_receiver_type short-circuits on Conj, so the
                // base's algebra profile was never consulted. The peel must make the
                // receiver decidable in BOTH directions on the SAME alias — a green
                // that only proves the refusal stopped is indistinguishable from
                // deleting the wall.
                let peel_src = |call: &str| {
                    format!("module peel\ntype Tight = String where non_empty\nfn f(s: Tight) -> Int {{ s |> {} }}\n", call)
                };
                let peel_green = compile_one("peel_green.dag", peel_src("count"));
                assert!(
                    peel_green.diagnostics.is_empty(),
                    "a method on the refinement's String base must RESOLVE once the base is peeled — no diagnostic of any severity, got: {:?}",
                    peel_green.diagnostics
                );
                // DISCRIMINATING RED for the two special-case arms (codex review
                // 45430). The wall was reached only from the FINAL else, so an
                // unresolved map_keys / map_values still returned the RECEIVER type
                // with an empty diagnostic list — the exact success-shaped fallback
                // this PR deletes, surviving in the two branches that ran before it.
                let special = compile_one("special.dag",
                    "module special\nfn f(xs: List<Int>) -> List<Int> {{ xs |> map_keys }}\nfn g(xs: List<Int>) -> List<Int> {{ xs |> map_values }}\n".replace("{{", "{").replace("}}", "}"));
                let special_missing: Vec<_> = special.diagnostics.iter()
                    .filter(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::MethodNotFound { .. }))
                    .collect();
                assert!(
                    special_missing.len() >= 2,
                    "map_keys and map_values must route through the SAME refusal as every other method when the receiver is not a keyed collection — a wall reachable only from the final else is not a wall, got: {:?}",
                    special.diagnostics
                );
                // ReceiverTypeUnestablished CLASSIFICATION control. The behavioural
                // evidence for this class is the corpus census, not a synthetic
                // source: 18 sites whole-tree, counted as advisories with the gate at
                // zero blocking diagnostics. A synthetic reproducer WAS attempted --
                // an untyped lambda parameter inside a record-field fn, the shape the
                // real sites have -- and it produced no diagnostic at all, so it did
                // not reproduce the shape and is not asserted here as though it did.
                // ADMISSION is roster-gated, not automatic: an anonymous receiver is
                // admitted only where a declared row names (module, method,
                // Primitive()), and the perturbation loop above proves that gate for
                // every row. Admitting on the CAUSE alone was the earlier version and
                // was an unbounded green path — sound about decidability, wrong about
                // admission, since it made every future anonymous-receiver call pass
                // including one whose method does not exist (codex review 45459).
                // What THIS control pins is the remaining half, the classification of
                // an admitted one: COUNTED and NON-BLOCKING. Blocking it fabricates a
                // refusal over the 18 measured sites; dropping it restores #7479 silence.
                let unestablished_diag = std::rc::Rc::new(
                    crate::v1_std_core::CompilerDiagnostic::ReceiverTypeUnestablished {
                        method: "probe".to_string(),
                        span: std::rc::Rc::new(crate::std_types::SourceSpan {
                            file: "probe.dag".to_string(), start: 0, end: 0,
                        }),
                    });
                assert!(
                    !crate::v1_std_core::is_error_diagnostic(unestablished_diag.clone())
                        && !crate::v1_std_core::is_interpreter_blocking_diagnostic(unestablished_diag.clone()),
                    "ReceiverTypeUnestablished must not block — the receiver's type is an upstream deficit, so refusing here fabricates a refusal over correct code"
                );
                assert!(
                    crate::v1_std_core::is_discovery_corpus_advisory_typecheck_diagnostic(unestablished_diag.clone()),
                    "...and it must be a COUNTED advisory, never absent — an uncounted degradation is the absorbing fallback DESIGN §5 forbids"
                );
                // DISCRIMINATING CONTROL for the occurrence ratchet (codex reviews
                // 45464 and 45491). A (module, method, receiver_shape) key bounds
                // WHERE an unresolved call may live but not HOW MANY, and the rows
                // are not singletons — v2.compiler.tokenize admits seven `apply`.
                // The comparison is EQUALITY, not a ceiling: a ceiling lets seven
                // shrink to six and a seventh come back silently, which is a static
                // limit rather than a ratchet. Equality is safe because the check runs
                // per MODULE, and every occurrence in a module is present whenever
                // that module is typechecked at all — a closure that omits the module
                // never runs its rows. Exercised at the mechanism so the boundary is
                // exact on both sides.
                let budget_row = rows.iter()
                    .find(|r| r.receiver_shape == "Primitive()")
                    .expect("expected at least one anonymous-receiver row to bound");
                let probe_span = || std::rc::Rc::new(crate::std_types::SourceSpan {
                    file: "probe.dag".to_string(), start: 0, end: 0,
                });
                let unestablished_at = |n: usize| {
                    std::rc::Rc::new((0..n).map(|_| std::rc::Rc::new(crate::v1_std_core::ErrorNode {
                        diagnostic: std::rc::Rc::new(crate::v1_std_core::CompilerDiagnostic::ReceiverTypeUnestablished {
                            method: budget_row.method.clone(),
                            span: probe_span(),
                        }),
                        module_name: budget_row.module_name.clone(),
                    })).collect::<im::Vector<_>>())
                };
                let budget_at = |n: usize| crate::v1_compiler_infer::frontier_occurrence_budget_diags(
                    budget_row.module_name.clone(), probe_span(), unestablished_at(n));
                assert!(
                    budget_at(budget_row.occurrences as usize).is_empty(),
                    "exactly the declared occurrence count must pass"
                );
                assert!(
                    budget_at(budget_row.occurrences as usize + 1).iter().any(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::FrontierOccurrenceBudgetExceeded { .. })),
                    "one MORE than the declared count must refuse — otherwise the row bounds where an unresolved call may live but not how many"
                );
                assert!(
                    budget_at(budget_row.occurrences as usize + 1).iter().all(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone())),
                    "...and that refusal must BLOCK, or the ratchet is decorative"
                );
                assert!(
                    budget_at(0).iter().any(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::FrontierOccurrenceBudgetExceeded { .. })),
                    "FEWER than declared must ALSO refuse, and this is the half that makes it a ratchet rather than a ceiling: fixing a call forces the declared count DOWN, so a later reintroduction has no headroom to slip back into (codex review 45491)"
                );
                // DISCRIMINATING PAIR for the early-return walk. An early return is a
                // second exit from the same declaration and must meet its declared
                // type; a return inside a nested LAMBDA belongs to the lambda's own
                // callable return, so checking it against the enclosing declaration
                // fabricates a refusal — the mirror image of the hole the walk closes
                // (codex reviews 45472 and 45481, both confirmed by execution).
                let early_return = compile_one("early_return.dag",
                    "module early_return\nfn f(cond: Bool) -> Int { if cond { return \"wrong\" } 1 }\n".to_string());
                assert!(
                    early_return.diagnostics.iter().any(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::TypeMismatch { .. })),
                    "an early return of the wrong type must refuse — the trailing expression conforming says nothing about the other exit, got: {:?}",
                    early_return.diagnostics
                );
                let lambda_return = compile_one("lambda_return.dag",
                    "module lambda_return\nfn apply_it(g: fn(Int) -> String, v: Int) -> String { g(v) }\nfn outer(v: Int) -> Int { apply_it(g: x => { return \"inner\" }, v: v) 1 }\n".to_string());
                assert!(
                    lambda_return.diagnostics.iter().all(|d| !matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::TypeMismatch { .. })),
                    "a return inside a nested lambda belongs to the LAMBDA's declared return, not the enclosing function's — checking it here fabricates a refusal, got: {:?}",
                    lambda_return.diagnostics
                );
                let peel_red = compile_one("peel_red.dag", peel_src("filter_map(x => x)"));
                assert!(
                    peel_red.diagnostics.iter().any(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::MethodNotFound { .. })),
                    "peeling must make the receiver DECIDABLE, not merely quiet: a method absent from the peeled base must refuse as MethodNotFound rather than resting in the frontier, got: {:?}",
                    peel_red.diagnostics
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("method_existence_wall_witness panicked");
    }

    #[test]
    fn declared_type_conformance_witness() {
        // DISCRIMINATING RED for declared_type_conformance_note. infer_item kept the
        // declaration's inferred return regardless of what the body produced, so
        // `fn f() -> Int { \"wrong\" }` typechecked with ZERO diagnostics.
        let result = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let red = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "red.dag".to_string(),
                    content: "module red\nfn f() -> Int { \"a string\" }\ndata d: Int = \"a string\"\n".to_string(),
                });
                let red_result = crate::v1_compiler_compile::compile_sources(
                    std::rc::Rc::new(im::vector![red]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                let mismatches: Vec<_> = red_result.diagnostics.iter()
                    .filter(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::TypeMismatch { .. }))
                    .collect();
                assert!(
                    mismatches.len() >= 2,
                    "expected a TypeMismatch for BOTH the fn return and the data annotation, got: {:?}",
                    red_result.diagnostics
                );
                // POSITIVE CONTROLS: conforming declarations, and the optional-cardinality
                // case (`first` yields Int? for a declared Int?) which must not red.
                let green = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "green.dag".to_string(),
                    content: "module green\nfn a() -> Int { 42 }\nfn b() -> String { \"fine\" }\ndata c: Int = 7\nfn e(xs: List<Int>) -> Int? { xs |> first }\n".to_string(),
                });
                let green_result = crate::v1_compiler_compile::compile_sources(
                    std::rc::Rc::new(im::vector![green]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                assert!(
                    green_result.diagnostics.is_empty(),
                    "conforming declarations must produce NO diagnostic of any severity — filtering to the blocking variant would let an advisory pass unnoticed (codex review 45357), got: {:?}",
                    green_result.diagnostics
                );
                // DISCRIMINATING RED for the container widening (codex review 45398:
                // a provable mismatch in container ELEMENT types was indistinguishable
                // from a valid declaration, because the ground-scalar gate required a
                // plain shape and a List is not one). A List of a ground kernel scalar
                // carries no alias, brand, coproduct or cardinality representation
                // between the two sides either, so the same positive-establishment
                // argument that admits Int-vs-String admits List<Int>-vs-List<String>.
                let container_red = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "container_red.dag".to_string(),
                    content: "module container_red\nfn f() -> List<Int> { [\"a\", \"b\"] }\ndata d: List<String> = [1, 2]\n".to_string(),
                });
                let container_result = crate::v1_compiler_compile::compile_sources(
                    std::rc::Rc::new(im::vector![container_red]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                let container_mismatches: Vec<_> = container_result.diagnostics.iter()
                    .filter(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::TypeMismatch { .. }))
                    .collect();
                assert!(
                    container_mismatches.len() >= 2,
                    "a declared container whose ELEMENT type the body contradicts must refuse, for BOTH the fn return and the data annotation, got: {:?}",
                    container_result.diagnostics
                );
                // POSITIVE CONTROL for the same widening: matching element types, and a
                // container of a NON-ground element, which stays unjudged rather than
                // guessed at.
                let container_green = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "container_green.dag".to_string(),
                    content: "module container_green\nfn g() -> List<Int> { [1, 2] }\ndata h: List<String> = [\"x\"]\n".to_string(),
                });
                let container_green_result = crate::v1_compiler_compile::compile_sources(
                    std::rc::Rc::new(im::vector![container_green]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                assert!(
                    container_green_result.diagnostics.is_empty(),
                    "a conforming container declaration must produce NO diagnostic, got: {:?}",
                    container_green_result.diagnostics
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("declared_type_conformance_witness panicked");
    }

    #[test]
    fn sole_constructor_violation_outside_module() {
        let result = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let module_a = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "module_a.dag".to_string(),
                    content: "module module_a\ntype Sealed sole_constructor { x: String }\nfn make_sealed(v: String) -> Sealed { Sealed { x: v } }\n".to_string(),
                });
                let module_b = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "module_b.dag".to_string(),
                    content: "module module_b\nimport module_a { Sealed }\nfn bad_ctor(v: String) -> Sealed { Sealed { x: v } }\n".to_string(),
                });
                let result = crate::v1_compiler_compile::compile_sources(
                    std::rc::Rc::new(im::vector![module_a, module_b]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                let sole_ctor_errors: Vec<_> = result.diagnostics.iter()
                    .filter(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::SoleConstructorViolation { .. }))
                    .collect();
                assert!(
                    !sole_ctor_errors.is_empty(),
                    "expected SoleConstructorViolation for cross-module construction, got diagnostics: {:?}",
                    result.diagnostics
                );
                let in_module_b = sole_ctor_errors.iter().any(|e| e.module_name == "module_b");
                assert!(
                    in_module_b,
                    "SoleConstructorViolation should be reported in module_b, got: {:?}",
                    sole_ctor_errors
                );
                let all_errors_in_a: Vec<_> = result.diagnostics.iter()
                    .filter(|d| {
                        d.module_name == "module_a" &&
                        matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::SoleConstructorViolation { .. })
                    })
                    .collect();
                assert!(
                    all_errors_in_a.is_empty(),
                    "module_a's own construction should be allowed, got violations: {:?}",
                    all_errors_in_a
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("sole_constructor_violation_outside_module test panicked");
    }

    #[test]
    fn constructor_call_admission_refuses_unlisted_cross_module_caller() {
        let result = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let mint_mod = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "mint_mod.dag".to_string(),
                    content: "module mint_mod\ntype Sealed sole_constructor { tag: String }\nfn mint(tag: String) -> Sealed admit_callers: [decl_ref(module_path: \"caller_ok\", decl_name: \"ok_call\")] = Sealed { tag: tag }\n".to_string(),
                });
                let caller_ok = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "caller_ok.dag".to_string(),
                    content: "module caller_ok\nimport mint_mod { mint, Sealed }\nfn ok_call() -> Sealed { mint(\"ok\") }\n".to_string(),
                });
                let caller_bad = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "caller_bad.dag".to_string(),
                    content: "module caller_bad\nimport mint_mod { mint, Sealed }\nfn bad_call() -> Sealed { mint(\"forged\") }\n".to_string(),
                });
                let ok_result = crate::v1_compiler_compile::compile_sources(
                    std::rc::Rc::new(im::vector![mint_mod.clone(), caller_ok.clone()]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                assert!(
                    ok_result.diagnostics.iter().all(|d| !matches!(
                        *d.diagnostic,
                        crate::v1_std_core::CompilerDiagnostic::ConstructorCallAdmissionRefused { .. }
                    )),
                    "listed caller should compile clean, got: {:?}",
                    ok_result.diagnostics
                );
                let bad_result = crate::v1_compiler_compile::compile_sources(
                    std::rc::Rc::new(im::vector![mint_mod, caller_bad]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                let admission_errors: Vec<_> = bad_result.diagnostics.iter()
                    .filter(|d| matches!(
                        *d.diagnostic,
                        crate::v1_std_core::CompilerDiagnostic::ConstructorCallAdmissionRefused { .. }
                    ))
                    .collect();
                assert!(
                    !admission_errors.is_empty(),
                    "expected ConstructorCallAdmissionRefused for unlisted caller, got: {:?}",
                    bad_result.diagnostics
                );
                assert!(
                    admission_errors.iter().any(|e| e.module_name == "caller_bad"),
                    "refusal should be reported in caller_bad, got: {:?}",
                    admission_errors
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("constructor_call_admission_refuses_unlisted_cross_module_caller panicked");
    }

    #[test]
    fn constructor_call_admission_refuses_same_module_unlisted_sibling() {
        let result = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let mint_mod = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "mint_mod.dag".to_string(),
                    content: "module mint_mod\ntype Sealed sole_constructor { tag: String }\nfn mint(tag: String) -> Sealed admit_callers: [decl_ref(module_path: \"caller_ok\", decl_name: \"ok_call\")] = Sealed { tag: tag }\n".to_string(),
                });
                let caller_ok = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "caller_ok.dag".to_string(),
                    content: "module caller_ok\nimport mint_mod { mint, Sealed }\nfn ok_call() -> Sealed { mint(\"ok\") }\nfn sneak_call() -> Sealed { mint(\"sneak\") }\n".to_string(),
                });
                let result = crate::v1_compiler_compile::compile_sources(
                    std::rc::Rc::new(im::vector![mint_mod, caller_ok]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                let admission_errors: Vec<_> = result.diagnostics.iter()
                    .filter_map(|d| match &*d.diagnostic {
                        crate::v1_std_core::CompilerDiagnostic::ConstructorCallAdmissionRefused {
                            caller_decl_name, ..
                        } => Some(caller_decl_name.clone()),
                        _ => None,
                    })
                    .collect();
                assert!(
                    admission_errors.iter().any(|n| n == "sneak_call"),
                    "expected ConstructorCallAdmissionRefused naming sneak_call (same module as the admitted sibling, but not admitted); got: {:?}",
                    result.diagnostics
                );
                assert!(
                    !admission_errors.iter().any(|n| n == "ok_call"),
                    "ok_call is admitted by exact decl_ref and must not be refused; got: {:?}",
                    admission_errors
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("constructor_call_admission_refuses_same_module_unlisted_sibling panicked");
    }

    #[test]
    fn constructor_call_admission_refuses_unlisted_function_value_reference() {
        let result = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let mint_mod = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "mint_mod.dag".to_string(),
                    content: "module mint_mod\ntype Sealed sole_constructor { tag: String }\nfn mint(tag: String) -> Sealed admit_callers: [decl_ref(module_path: \"caller_ok\", decl_name: \"ok_call\")] = Sealed { tag: tag }\n".to_string(),
                });
                let caller_ok = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "caller_ok.dag".to_string(),
                    content: "module caller_ok\nimport mint_mod { mint, Sealed }\nfn plain(n: String) -> String = n\nfn ok_call() -> Sealed { let f = mint  mint(\"ok\") }\nfn sneak_ref() -> String { let g = mint  \"x\" }\nfn use_plain() -> String { let h = plain  \"y\" }\n".to_string(),
                });
                let result = crate::v1_compiler_compile::compile_sources(
                    std::rc::Rc::new(im::vector![mint_mod, caller_ok]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                let refused: Vec<_> = result.diagnostics.iter()
                    .filter_map(|d| match &*d.diagnostic {
                        crate::v1_std_core::CompilerDiagnostic::ConstructorCallAdmissionRefused {
                            caller_decl_name, ..
                        } => Some(caller_decl_name.clone()),
                        _ => None,
                    })
                    .collect();
                assert!(
                    refused.iter().any(|n| n == "sneak_ref"),
                    "an unlisted caller taking the sealed constructor as a FUNCTION VALUE must be refused; got: {:?}",
                    result.diagnostics
                );
                assert!(
                    !refused.iter().any(|n| n == "ok_call"),
                    "the listed caller must still be able to reference the constructor as a value; got: {:?}",
                    refused
                );
                assert!(
                    !refused.iter().any(|n| n == "use_plain"),
                    "an ordinary non-sealed function must remain usable as a value; got: {:?}",
                    refused
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect(
            "constructor_call_admission_refuses_unlisted_function_value_reference panicked",
        );
    }

    #[test]
    fn constructor_call_admission_decides_zero_arity_references() {
        let result = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let seal_mod = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "seal_mod.dag".to_string(),
                    content: "module seal_mod\ntype Token sole_constructor { tag: String }\nfn token() -> Token admit_callers: [decl_ref(module_path: \"z_caller\", decl_name: \"ok_bare\"), decl_ref(module_path: \"z_caller\", decl_name: \"ok_call\")] = Token { tag: \"t\" }\nfn plain() -> String = \"p\"\n".to_string(),
                });
                let z_caller = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "z_caller.dag".to_string(),
                    content: "module z_caller\nimport seal_mod { token, Token, plain }\nfn ok_bare() -> Token { token }\nfn bad_bare() -> Token { token }\nfn ok_call() -> Token { token() }\nfn bad_call() -> Token { token() }\nfn bad_qual() -> Token { seal_mod.token() }\nfn ordinary() -> String { plain }\n".to_string(),
                });
                let result = crate::v1_compiler_compile::compile_sources(
                    std::rc::Rc::new(im::vector![seal_mod, z_caller]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                let refused: Vec<_> = result.diagnostics.iter()
                    .filter_map(|d| match &*d.diagnostic {
                        crate::v1_std_core::CompilerDiagnostic::ConstructorCallAdmissionRefused {
                            caller_decl_name, constructor_decl_name, ..
                        } => Some((caller_decl_name.clone(), constructor_decl_name.clone())),
                        _ => None,
                    })
                    .collect();
                let refused_callers: Vec<String> = refused.iter().map(|(c, _)| c.clone()).collect();
                for unlisted in ["bad_bare", "bad_call", "bad_qual"] {
                    assert!(
                        refused_callers.iter().any(|n| n == unlisted),
                        "unlisted caller {} of a ZERO-ARITY sealed constructor must be refused; got: {:?}",
                        unlisted,
                        result.diagnostics
                    );
                }
                for permitted in ["ok_bare", "ok_call", "ordinary"] {
                    assert!(
                        !refused_callers.iter().any(|n| n == permitted),
                        "{} must NOT be refused -- a wall that refuses listed callers or ordinary \
                         zero-arity functions is a fabricated refusal, not a fix; got: {:?}",
                        permitted,
                        refused_callers
                    );
                }
                let qualified: Vec<&String> = refused.iter()
                    .filter(|(c, _)| c == "bad_qual")
                    .map(|(_, k)| k)
                    .collect();
                assert!(
                    qualified.iter().all(|k| *k == "token"),
                    "a qualified zero-arity call must report the exact constructor identity \
                     'token', not a doubled or call-site spelling; got: {:?}",
                    qualified
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("constructor_call_admission_decides_zero_arity_references panicked");
    }

    #[test]
    fn constructor_call_admission_lets_a_body_binding_shadow_the_constructor() {
        let result = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let seal_mod = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "seal_mod.dag".to_string(),
                    content: "module seal_mod\ntype Token sole_constructor { tag: String }\nfn token() -> Token admit_callers: [decl_ref(module_path: \"z_caller\", decl_name: \"ok_use\")] = Token { tag: \"t\" }\n".to_string(),
                });
                let z_caller = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "z_caller.dag".to_string(),
                    content: "module z_caller\nimport seal_mod { token, Token }\nfn ok_use() -> Token { token }\nfn shadowed_param(token: String) -> String { token }\nfn shadowed_let() -> String { let token = \"x\"  token }\n".to_string(),
                });
                let result = crate::v1_compiler_compile::compile_sources(
                    std::rc::Rc::new(im::vector![seal_mod, z_caller]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                let refused: Vec<_> = result.diagnostics.iter()
                    .filter_map(|d| match &*d.diagnostic {
                        crate::v1_std_core::CompilerDiagnostic::ConstructorCallAdmissionRefused {
                            caller_decl_name, ..
                        } => Some(caller_decl_name.clone()),
                        _ => None,
                    })
                    .collect();
                for shadowing in ["shadowed_param", "shadowed_let"] {
                    assert!(
                        !refused.iter().any(|n| n == shadowing),
                        "{} binds its own token, so the sealed constructor of that spelling is not \
                         referenced at all; refusing it is a fabricated refusal. got: {:?}",
                        shadowing,
                        result.diagnostics
                    );
                }
                assert!(
                    !refused.iter().any(|n| n == "ok_use"),
                    "the listed caller must still succeed; got: {:?}",
                    refused
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect(
            "constructor_call_admission_lets_a_body_binding_shadow_the_constructor panicked",
        );
    }

    #[test]
    fn constructor_call_admission_qualified_caller_reports_undoubled_identity() {
        let result = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let mint_mod = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "mint_mod.dag".to_string(),
                    content: "module mint_mod\ntype Sealed sole_constructor { tag: String }\nfn mint(tag: String) -> Sealed admit_callers: [decl_ref(module_path: \"caller_ok\", decl_name: \"ok_call\")] = Sealed { tag: tag }\n".to_string(),
                });
                let caller_bare = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "caller_bare.dag".to_string(),
                    content: "module caller_bare\nfn bare_qualified() -> mint_mod.Sealed { mint_mod.mint(\"bare\") }\n".to_string(),
                });
                let result = crate::v1_compiler_compile::compile_sources(
                    std::rc::Rc::new(im::vector![mint_mod, caller_bare]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                let refusals: Vec<(String, String, String)> = result.diagnostics.iter()
                    .filter_map(|d| match &*d.diagnostic {
                        crate::v1_std_core::CompilerDiagnostic::ConstructorCallAdmissionRefused {
                            constructor_module_path, constructor_decl_name, caller_decl_name, ..
                        } => Some((
                            constructor_module_path.clone(),
                            constructor_decl_name.clone(),
                            caller_decl_name.clone(),
                        )),
                        _ => None,
                    })
                    .collect();
                let hit = refusals.iter().find(|(_, _, caller)| caller == "bare_qualified");
                assert!(
                    hit.is_some(),
                    "an unlisted caller reaching the constructor by qualified projection must be refused; got: {:?}",
                    result.diagnostics
                );
                let (module_path, decl_name, _) = hit.unwrap();
                assert_eq!(
                    module_path, "mint_mod",
                    "constructor_module_path must be the owning module; got: {:?}",
                    refusals
                );
                assert_eq!(
                    decl_name, "mint",
                    "constructor_decl_name must be the declaration's own name, not the call-site spelling -- a qualified call must not render as mint_mod.mint_mod.mint; got: {:?}",
                    refusals
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect(
            "constructor_call_admission_qualified_caller_reports_undoubled_identity panicked",
        );
    }

    #[test]
    fn sole_constructor_fieldless_newtype_witness() {
        // Discriminating witness: a field-less (empty-body) sole_constructor record.
        // EmittableGraph is Conj (multi-field) so the phantom property is inert there.
        // A field-less type has connective==NoConnective and children==[], so the
        // phantom property flips __is_leaf false and breaks node_type_equals_core.
        // This test proves whether that mis-classification occurs.
        let result = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                let module_a = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "module_a.dag".to_string(),
                    content: "module module_a\ntype FieldlessFoo sole_constructor { }\nfn make_fieldless() -> FieldlessFoo { FieldlessFoo { } }\nfn identity(f: FieldlessFoo) -> FieldlessFoo { f }\n".to_string(),
                });
                let module_b = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "module_b.dag".to_string(),
                    content: "module module_b\nimport module_a { FieldlessFoo }\nfn bad_ctor() -> FieldlessFoo { FieldlessFoo { } }\n".to_string(),
                });
                let result = crate::v1_compiler_compile::compile_sources(
                    std::rc::Rc::new(im::vector![module_a, module_b]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                let sole_ctor_errors: Vec<_> = result.diagnostics.iter()
                    .filter(|d| matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::SoleConstructorViolation { .. }))
                    .collect();
                let violation_in_b = sole_ctor_errors.iter().any(|e| e.module_name == "module_b");
                assert!(
                    violation_in_b,
                    "FieldlessFoo: expected SoleConstructorViolation in module_b, got: {:?}",
                    result.diagnostics
                );
                let type_mismatch_in_a: Vec<_> = result.diagnostics.iter()
                    .filter(|d| {
                        d.module_name == "module_a" &&
                        matches!(*d.diagnostic, crate::v1_std_core::CompilerDiagnostic::TypeMismatch { .. })
                    })
                    .collect();
                assert!(
                    type_mismatch_in_a.is_empty(),
                    "FieldlessFoo: TypeMismatch in module_a identity fn -- leaf mis-classified by phantom property, got: {:?}",
                    type_mismatch_in_a
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("sole_constructor_fieldless_newtype_witness panicked");
    }

    #[test]
    fn contracts_sidecar_wired_into_emit_scope() {
        // Discriminating witness: AnthropicChatMessage is declared in
        // extdeps.llm.anthropic; its tag = "role" wire_contract lives in the
        // anthropic_contracts.dag sidecar. This proves contracts_items_for_module
        // and the wire_contract alias-resolution scope merge the sidecar into the
        // emitted module -- red if the sidecar wiring or alias scope regresses.
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let entry_pairs = discover_dag_files("dag/extdeps/llm");
                let sources = std::rc::Rc::new(resolve_source_closure(entry_pairs, &["dag"]).into());
                let result = crate::v1_compiler_compile::compile_sources(
                    sources,
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                let anthropic_file = result.files.iter()
                    .find(|f| f.path.ends_with("extdeps_llm_anthropic.rs"))
                    .expect("emitted file for extdeps.llm.anthropic not found in source closure");
                assert!(
                    anthropic_file.content.contains("#[serde(tag = \"role\""),
                    "AnthropicChatMessage serde tag = role must be present in emitted Rust (contracts_items_for_module merged into emit scope); missing from: {}",
                    anthropic_file.path
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("contracts_sidecar_wired_into_emit_scope panicked");
    }

    #[test]
    fn caret_parse_smoke_native_compile_emit_witnesses() {
        use crate::v1_tests_claim_caret_parse_smoke_test::*;
        assert!(w_caret_tokenizes_as_sh_caret());
        assert!(w_caret_paren_tokenizes_as_caret_then_lparen());
        assert!(w_parse_caret_ident_produces_literal());
        assert!(w_parse_caret_paren_produces_discriminant_call());
        assert!(w_parse_expr_caret_paren_full_pipeline());
        assert!(w_parse_expr_caret_var_arg_produces_discriminant_call());
        assert!(w_parse_module_let_caret_paren());
        assert!(w_compile_to_resolved_caret_probe5b_has_no_caret_function_error());
        assert!(w_emit_caret_ident_symbol_literal());
        assert!(w_emit_caret_paren_discriminant_sugar());
    }

    #[test]
    fn self_parse_all_modules() {
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let v1_files = discover_dag_files("src/v1");
                assert!(
                    !v1_files.is_empty(),
                    "should discover at least one .dag file in src/v1/"
                );

                for (file, source) in &v1_files {
                    let tokens = tokenize(source.to_string(), file.to_string());
                    assert!(!tokens.is_empty(), "{} should produce tokens", file);
                    assert!(
                        matches!(
                            tokens.last().unwrap().shape,
                            crate::v1_std_core::TokenShape::ShEof
                        ),
                        "{} should end with Eof",
                        file
                    );
                    let result = crate::v1_compiler_parse::parse(
                        tokens,
                        std::rc::Rc::new(im::HashMap::new()),
                    );
                    assert!(
                        result.module.is_some(),
                        "{} should parse successfully, error: {:?}",
                        file,
                        result.error
                    );
                    let module = result.module.as_ref().unwrap();
                    assert!(
                        !module.name.is_empty(),
                        "{} should have a non-empty module name",
                        file
                    );
                }
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("self-parse-all test panicked");
    }

    #[test]
    fn self_resolve_all_modules() {
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let sources = std::rc::Rc::new(self_compile_sources().into());
                let result = crate::v1_compiler_compile::resolve_sources(sources);

                let errors: Vec<_> = result
                    .diagnostics
                    .iter()
                    .filter(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone()))
                    .collect();
                let error_count = errors.len();

                eprintln!("self-resolve error count: {}", error_count);
                for e in &errors {
                    eprintln!("  {:?}", e.diagnostic);
                }

                assert!(
                    error_count == 0,
                    "self-resolve errors: {} errors (expected 0): {:?}",
                    error_count,
                    errors
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("self-resolve-all test panicked");
    }

    #[test]
    fn self_compile_all_modules() {
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let sources: std::rc::Rc<im::Vector<_>> =
                    std::rc::Rc::new(self_compile_sources().into());
                let source_count = sources.len();
                let result = crate::v1_compiler_compile::compile_sources(
                    sources,
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );

                let errors: Vec<_> = result
                    .diagnostics
                    .iter()
                    .filter(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone()))
                    .collect();
                let error_count = errors.len();

                eprintln!(
                    "self-compile completed: {} errors, {} files emitted from {} sources",
                    error_count,
                    result.files.len(),
                    source_count
                );
                for (i, e) in errors.iter().enumerate() {
                    eprintln!("  error[{}]: {:?}", i, e.diagnostic);
                }

                assert!(
                    source_count >= 13,
                    "self-compile should process at least 13 sources, got {}",
                    source_count
                );

                if !result.files.is_empty() {
                    assert!(
                        result.files.iter().all(|f| !f.content.is_empty()),
                        "all self-compiled output files must have non-empty content"
                    );
                }

                const SELF_COMPILE_ERROR_RATCHET: usize = 2700;
                assert!(
                    error_count <= SELF_COMPILE_ERROR_RATCHET,
                    "self-compile error count regression: {} > {} ratchet",
                    error_count,
                    SELF_COMPILE_ERROR_RATCHET
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("self-compile-all test panicked");
    }

    #[test]
    #[ignore]
    fn self_compile_cargo_check() {
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let sources = std::rc::Rc::new(self_compile_sources().into());

                let result = crate::v1_compiler_compile::compile_sources(
                    sources,
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );

                if result.files.is_empty() {
                    eprintln!("self-compile-cargo-check: 0 files emitted (resolve errors gate emission), skipping cargo check");
                    return;
                }

                let tmp_dir = std::env::temp_dir().join("v2-self-compile-check");
                let _ = std::fs::remove_dir_all(&tmp_dir);
                std::fs::create_dir_all(tmp_dir.join("src"))
                    .expect("failed to create temp src dir");

                for file in result.files.iter() {
                    let dest = tmp_dir.join(&file.path);
                    if let Some(parent) = dest.parent() {
                        std::fs::create_dir_all(parent).expect("failed to create parent dir");
                    }
                    std::fs::write(&dest, &file.content)
                        .expect(&format!("failed to write {}", file.path));
                }

                let cargo_toml = tmp_dir.join("Cargo.toml");
                if !cargo_toml.exists() {
                    std::fs::write(&cargo_toml,
                        "[package]\nname = \"v2-self-compile-check\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"
                    ).expect("failed to write Cargo.toml");
                }

                eprintln!("self-compile-cargo-check: wrote {} files to {}",
                    result.files.len(), tmp_dir.display());

                let output = std::process::Command::new("cargo")
                    .arg("check")
                    .current_dir(&tmp_dir)
                    .output()
                    .expect("failed to run cargo check");

                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("cargo check stderr:\n{}", stderr);

                if !output.status.success() {
                    panic!(
                        "cargo check failed on self-compiled output (dir: {}):\n{}",
                        tmp_dir.display(),
                        stderr
                    );
                }

                let _ = std::fs::remove_dir_all(&tmp_dir);
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("self-compile-cargo-check test panicked");
    }

    #[test]
    fn type_size_regression_check() {
        let node_size = std::mem::size_of::<crate::v1_std_core::Node>();
        let expr_size = std::mem::size_of::<crate::v1_std_core::ExprData>();
        eprintln!("  Node: {} bytes", node_size);
        eprintln!("  Expr: {} bytes", expr_size);
        assert!(
            node_size <= 192,
            "Node size regression: {} bytes (limit: 192). Check for unboxed rare fields.",
            node_size
        );
        assert!(
            expr_size <= 800,
            "ExprData size regression: {} bytes (limit: 800). Node size likely regressed.",
            expr_size
        );
    }

    // =========================================================================
    // Coercion registry tests (auto-generated from data declarations)
    // =========================================================================

    #[test]
    fn coercion_rust_checkpoint_resolves_primitives() {
        use crate::v1_compiler_coercion::*;
        assert_eq!(
            coerce_primitive_type(RenderTarget::Rust, "Int".into()),
            "i64"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Rust, "Float".into()),
            "f64"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Rust, "Bool".into()),
            "bool"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Rust, "Symbol".into()),
            "String"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Rust, "Unit".into()),
            "()"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Rust, "String".into()),
            "String"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Rust, "Bytes".into()),
            "Vec<u8>"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Rust, "Secret".into()),
            "String"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Rust, "Json".into()),
            "serde_json::Value"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Rust, "Hash".into()),
            "v1_rt::Hash"
        );
    }

    #[test]
    fn coercion_python_checkpoint_resolves_primitives() {
        use crate::v1_compiler_coercion::*;
        assert_eq!(
            coerce_primitive_type(RenderTarget::Python, "Int".into()),
            "int"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Python, "Float".into()),
            "float"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Python, "Bool".into()),
            "bool"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Python, "Unit".into()),
            "None"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Python, "String".into()),
            "str"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Python, "Bytes".into()),
            "bytes"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Python, "Secret".into()),
            "str"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Python, "Json".into()),
            "dict"
        );
    }

    #[test]
    fn coercion_go_checkpoint_resolves_primitives() {
        use crate::v1_compiler_coercion::*;
        assert_eq!(
            coerce_primitive_type(RenderTarget::Go, "Int".into()),
            "int64"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Go, "Float".into()),
            "float64"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Go, "Bool".into()),
            "bool"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Go, "Unit".into()),
            "struct{}"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Go, "String".into()),
            "string"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Go, "Bytes".into()),
            "[]byte"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Go, "Secret".into()),
            "string"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Go, "Json".into()),
            "interface{}"
        );
    }

    #[test]
    fn coercion_rust_inhabitant_resolves_containers() {
        use crate::v1_compiler_coercion::*;
        assert_eq!(
            coerce_container_template(RenderTarget::Rust, "BooleanAlgebra".into()),
            Some("BTreeSet<{0}>".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Rust, "FreeMonoid".into()),
            Some("Vec<{0}>".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Rust, "List".into()),
            Some("Vec<{0}>".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Rust, "Map".into()),
            Some("HashMap<{0}, {1}>".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Rust, "PartialFunction".into()),
            Some("HashMap<{0}, {1}>".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Rust, "Set".into()),
            Some("BTreeSet<{0}>".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Rust, "PointwisePower".into()),
            Some("BTreeSet<{0}>".to_string())
        );
    }

    #[test]
    fn coercion_python_inhabitant_resolves_containers() {
        use crate::v1_compiler_coercion::*;
        assert_eq!(
            coerce_container_template(RenderTarget::Python, "BooleanAlgebra".into()),
            Some("set[{0}]".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Python, "FreeMonoid".into()),
            Some("list[{0}]".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Python, "List".into()),
            Some("list[{0}]".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Python, "Map".into()),
            Some("dict[{0}, {1}]".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Python, "PartialFunction".into()),
            Some("dict[{0}, {1}]".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Python, "Set".into()),
            Some("set[{0}]".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Python, "PointwisePower".into()),
            Some("set[{0}]".to_string())
        );
    }

    #[test]
    fn coercion_go_inhabitant_resolves_containers() {
        use crate::v1_compiler_coercion::*;
        assert_eq!(
            coerce_container_template(RenderTarget::Go, "BooleanAlgebra".into()),
            Some("map[{0}]struct{}".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Go, "FreeMonoid".into()),
            Some("[]{0}".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Go, "List".into()),
            Some("[]{0}".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Go, "Map".into()),
            Some("map[{0}]{1}".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Go, "PartialFunction".into()),
            Some("map[{0}]{1}".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Go, "Set".into()),
            Some("map[{0}]struct{}".to_string())
        );
        assert_eq!(
            coerce_container_template(RenderTarget::Go, "PointwisePower".into()),
            Some("map[{0}]struct{}".to_string())
        );
    }

    #[test]
    fn coercion_is_copy_from_checkpoint() {
        use crate::v1_compiler_coercion::*;
        assert_eq!(is_copy(RenderTarget::Rust, "Int".into()), Some(true));
        assert_eq!(is_copy(RenderTarget::Rust, "Float".into()), Some(true));
        assert_eq!(is_copy(RenderTarget::Rust, "Bool".into()), Some(true));
        assert_eq!(is_copy(RenderTarget::Rust, "Symbol".into()), Some(false));
        assert_eq!(is_copy(RenderTarget::Rust, "Unit".into()), Some(true));
        assert_eq!(is_copy(RenderTarget::Rust, "String".into()), Some(false));
        assert_eq!(is_copy(RenderTarget::Rust, "Bytes".into()), Some(false));
        assert_eq!(is_copy(RenderTarget::Rust, "Secret".into()), Some(false));
        assert_eq!(is_copy(RenderTarget::Rust, "Json".into()), Some(false));
        assert_eq!(is_copy(RenderTarget::Rust, "Hash".into()), Some(false));
    }

    #[test]
    fn coercion_template_application() {
        use crate::v1_compiler_coercion::*;
        assert_eq!(
            apply_inhabitant_template1("Vec<{0}>".into(), "i64".into()),
            "Vec<i64>"
        );
        assert_eq!(
            apply_inhabitant_template1("BTreeSet<{0}>".into(), "i64".into()),
            "BTreeSet<i64>"
        );
        assert_eq!(
            apply_inhabitant_template2("HashMap<{0}, {1}>".into(), "String".into(), "i64".into()),
            "HashMap<String, i64>"
        );
        assert_eq!(
            apply_inhabitant_template1("list[{0}]".into(), "int".into()),
            "list[int]"
        );
        assert_eq!(
            apply_inhabitant_template1("set[{0}]".into(), "int".into()),
            "set[int]"
        );
        assert_eq!(
            apply_inhabitant_template2("dict[{0}, {1}]".into(), "str".into(), "int".into()),
            "dict[str, int]"
        );
        assert_eq!(
            apply_inhabitant_template1("[]{0}".into(), "int64".into()),
            "[]int64"
        );
        assert_eq!(
            apply_inhabitant_template1("map[{0}]struct{}".into(), "int64".into()),
            "map[int64]struct{}"
        );
        assert_eq!(
            apply_inhabitant_template2("map[{0}]{1}".into(), "string".into(), "int64".into()),
            "map[string]int64"
        );
    }

    fn shaped_type_node(
        name: &str,
        children: Vec<std::rc::Rc<crate::v1_std_core::Node>>,
    ) -> std::rc::Rc<crate::v1_std_core::Node> {
        let span = crate::v1_std_core::make_span(0, name.len() as i64);
        std::rc::Rc::new(crate::v1_std_core::Node {
            name: name.to_string(),
            ident: None,
            span: span.clone(),
            ident_span: Some(span),
            children: std::rc::Rc::new(children.into()),
            connective: crate::v1_std_core::Connective::NoConnective,
            params: std::rc::Rc::new(im::Vector::new()),
            inferred: None,
            return_cardinality: crate::v1_std_core::Cardinality::Required,
            uses: std::rc::Rc::new(im::Vector::new()),
            body: None,
            transport: None,
            properties: std::rc::Rc::new(im::Vector::new()),
            type_annotation: None,
            is_self_recursive: false,
            has_non_tail_self_call: false,
            match_pattern: None,
            expr_data: std::rc::Rc::new(crate::v1_std_core::ExprData::NoExprData),
        })
    }

    fn named_type_node(name: &str) -> std::rc::Rc<crate::v1_std_core::Node> {
        shaped_type_node(name, Vec::new())
    }

    #[test]
    fn rust_btree_set_ord_eligibility_requires_nominal_carrier_shape() {
        let source_indices = std::rc::Rc::new(HashMap::new());
        let empty_emit = crate::v1_compiler_infer_emit_info::empty_emit_graph_info();
        let symbol = named_type_node("Symbol");
        let diff_id = shaped_type_node("DiffId", vec![symbol.clone()]);
        assert!(
            crate::v1_compiler_emit_rust::rust_btree_set_element_ord_eligible(
                symbol.clone(),
                source_indices.clone(),
                empty_emit.clone()
            )
        );
        assert!(
            crate::v1_compiler_emit_rust::rust_btree_set_element_ord_eligible(
                diff_id.clone(),
                source_indices.clone(),
                empty_emit.clone()
            )
        );
        let formal_nonterminal = shaped_type_node("FormalNonterminal", vec![symbol.clone()]);
        let formal_terminal = shaped_type_node("FormalTerminal", vec![symbol.clone()]);
        assert!(
            crate::v1_compiler_emit_rust::rust_btree_set_element_ord_eligible(
                formal_nonterminal.clone(),
                source_indices.clone(),
                empty_emit.clone()
            )
        );
        assert!(
            crate::v1_compiler_emit_rust::rust_btree_set_element_ord_eligible(
                formal_terminal.clone(),
                source_indices.clone(),
                empty_emit.clone()
            )
        );
        assert!(
            !crate::v1_compiler_emit_rust::rust_btree_set_element_ord_eligible(
                shaped_type_node("Symbol", vec![named_type_node("Float")]),
                source_indices.clone(),
                empty_emit.clone()
            )
        );
        assert!(
            !crate::v1_compiler_emit_rust::rust_btree_set_element_ord_eligible(
                shaped_type_node("DiffId", vec![named_type_node("Float")]),
                source_indices.clone(),
                empty_emit.clone()
            )
        );
        assert!(
            !crate::v1_compiler_emit_rust::rust_btree_set_element_ord_eligible(
                named_type_node("TestClaimId"),
                source_indices.clone(),
                empty_emit.clone()
            )
        );
    }

    #[test]
    fn diagnostics_carrier_grounds_to_native_option() {
        assert!(
            crate::v1_compiler_emit_rust::is_host_diagnostics_carrier_alias(
                "Diagnostics".to_string()
            )
        );
        assert!(
            !crate::v1_compiler_emit_rust::is_host_diagnostics_carrier_alias(
                "Optional".to_string()
            )
        );
        assert!(
            crate::v1_compiler_emit_rust::is_grounded_coproduct_native_alias(
                "Diagnostics".to_string()
            )
        );
        let empty_shared = std::rc::Rc::new(im::OrdSet::new());
        assert_eq!(
            crate::v1_compiler_emit_rust::render_rust_diagnostics_carrier_applied(
                empty_shared.clone()
            ),
            "Option<NonEmptyDiagnostics>"
        );
        let mut shared_ned_inner = im::OrdSet::new();
        shared_ned_inner.insert("NonEmptyDiagnostics".to_string());
        let shared_ned = std::rc::Rc::new(shared_ned_inner);
        assert_eq!(
            crate::v1_compiler_emit_rust::render_rust_diagnostics_carrier_applied(shared_ned),
            "Option<Rc<NonEmptyDiagnostics>>"
        );
        assert!(crate::v1_compiler_emit_rust::is_some_like_variant_name(
            "Some".to_string()
        ));
        assert!(crate::v1_compiler_emit_rust::is_some_like_variant_name(
            "Present".to_string()
        ));
        assert!(!crate::v1_compiler_emit_rust::is_some_like_variant_name(
            "None".to_string()
        ));
        assert!(!crate::v1_compiler_emit_rust::is_some_like_variant_name(
            "Absent".to_string()
        ));
        assert!(crate::v1_compiler_emit_rust::is_optional_like_parent_name(
            "Diagnostics".to_string()
        ));
        assert!(crate::v1_compiler_emit_rust::is_optional_like_parent_name(
            "Optional".to_string()
        ));
        assert!(!crate::v1_compiler_emit_rust::is_optional_like_parent_name(
            "Witness".to_string()
        ));
        let diagnostics_node = named_type_node("Diagnostics");
        let source_indices = std::rc::Rc::new(HashMap::new());
        assert!(
            crate::v1_compiler_emit_rust::is_host_diagnostics_carrier_type(
                diagnostics_node.clone(),
                source_indices.clone()
            )
        );
        let empty_emit = crate::v1_compiler_infer_emit_info::empty_emit_graph_info();
        assert_eq!(
            crate::v1_compiler_emit_rust::render_rust_type(
                diagnostics_node,
                empty_shared,
                crate::v1_compiler_infer_emit_info::RustCorpusRepr::HostNative,
                source_indices,
                empty_emit
            ),
            "Option<NonEmptyDiagnostics>"
        );
    }

    #[test]
    fn groupcompletion_int_checkpoint_fires_under_faithful_corpus() {
        // Discriminating witness for the (b) checkpoint-order fix (sharp-bee-290 sign-off,
        // msg_6fc2ba88-549b-491e-9b6f-ab949539d682): emit_typed_item's zero-param alias-decl
        // branch calls rust_scalar_checkpoint_render_base (the single-authority checkpoint
        // lookup), not the HostNative-only rust_seed_host_numeric_alias, so the Int -> i64
        // checkpoint row (dag/extdeps/languages/rust/types.dag) fires BEFORE the RHS
        // (GroupCompletion<Nat>) is unfolded — under BOTH corpus representations. A
        // regression that narrows this back to the HostNative-only alias makes the
        // FaithfulFreeMonoid arm return None, which is what this witness guards.
        assert_eq!(
            crate::v1_compiler_emit_rust::rust_scalar_checkpoint_render_base(
                "Int".to_string(),
                crate::v1_compiler_infer_emit_info::RustCorpusRepr::FaithfulFreeMonoid
            ),
            Some("i64".to_string())
        );
        assert_eq!(
            crate::v1_compiler_emit_rust::rust_scalar_checkpoint_render_base(
                "Int".to_string(),
                crate::v1_compiler_infer_emit_info::RustCorpusRepr::HostNative
            ),
            Some("i64".to_string())
        );
        // GroupCompletion itself has no checkpoint row and is not the seed host numeric
        // alias, so the checkpoint correctly declines to render it directly (the RHS
        // unfolding path handles it as a real 2-field struct) — the checkpoint fires ONLY
        // for the Int/Nat leaf name, never widening to the container type.
        assert_eq!(
            crate::v1_compiler_emit_rust::rust_scalar_checkpoint_render_base(
                "GroupCompletion".to_string(),
                crate::v1_compiler_infer_emit_info::RustCorpusRepr::FaithfulFreeMonoid
            ),
            None
        );
    }

    #[test]
    fn render_rust_applied_type_routes_qualified_base_through_leaf_name() {
        // Discriminating witness (PR #7269 / sharp-bee-290 msg_6c27c10b): namespace-qualified
        // applied-type bases must route through rust_fn_sig_leaf_name, not authored_name_at
        // verbatim — rustc reports 'expected one of `,` or `>`, found `.`' in generic position.
        // RED if render_rust_applied_type regresses to dotted verbatim emit.
        let source_indices = std::rc::Rc::new(HashMap::new());
        let shared = std::rc::Rc::new(im::OrdSet::new());
        let generics = std::rc::Rc::new(im::Vector::new());
        let variant_to_enum = std::rc::Rc::new(HashMap::new());
        let env = crate::v1_compiler_infer_env::empty_type_env();
        let arg = named_type_node("Int");
        let applied = shaped_type_node("std.algebra.FreeMonoid", vec![arg]);
        let rendered = crate::v1_compiler_emit_rust::render_rust_applied_type(
            applied,
            generics,
            shared,
            crate::v1_compiler_infer_emit_info::RustCorpusRepr::FaithfulFreeMonoid,
            source_indices,
            variant_to_enum,
            env,
        );
        assert!(
            !rendered.contains('.'),
            "applied-type base must not emit namespace dots in generic position"
        );
        assert_eq!(rendered, "Vec<i64>");
    }

    /// Return current process RSS in bytes (macOS via mach_task_basic_info).
    fn get_rss_bytes() -> u64 {
        #[cfg(target_os = "macos")]
        {
            #[allow(non_camel_case_types)]
            #[repr(C)]
            struct mach_task_basic_info {
                virtual_size: u64,
                resident_size: u64,
                resident_size_max: u64,
                user_time: [u64; 2],
                system_time: [u64; 2],
                policy: i32,
                suspend_count: i32,
            }
            extern "C" {
                fn mach_task_self() -> u32;
                fn task_info(
                    target_task: u32,
                    flavor: u32,
                    task_info_out: *mut mach_task_basic_info,
                    task_info_count: *mut u32,
                ) -> i32;
            }
            const MACH_TASK_BASIC_INFO: u32 = 20;
            const MACH_TASK_BASIC_INFO_COUNT: u32 =
                (std::mem::size_of::<mach_task_basic_info>() / std::mem::size_of::<u32>()) as u32;
            let mut info: mach_task_basic_info = unsafe { std::mem::zeroed() };
            let mut count = MACH_TASK_BASIC_INFO_COUNT;
            let kr = unsafe {
                task_info(
                    mach_task_self(),
                    MACH_TASK_BASIC_INFO,
                    &mut info,
                    &mut count,
                )
            };
            if kr == 0 {
                info.resident_size
            } else {
                0
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            0
        }
    }

    fn format_bytes(bytes: u64) -> String {
        if bytes >= 1_073_741_824 {
            format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
        } else if bytes >= 1_048_576 {
            format!("{:.1} MB", bytes as f64 / 1_048_576.0)
        } else if bytes >= 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        }
    }

    #[test]
    #[ignore]
    fn profile_self_compile() {
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                use im::HashMap;
                use std::time::Instant;

                let sources = self_compile_sources();

                let source_count = sources.len();
                let rss_start = get_rss_bytes();
                eprintln!(
                    "\n=== SELF-COMPILE PIPELINE PROFILE ({} sources) ===",
                    source_count
                );
                eprintln!("  RSS at start: {}\n", format_bytes(rss_start));

                let t_stage = Instant::now();
                let mut token_lists = Vec::new();
                let phase1_diags = 0usize;
                for source in &sources {
                    let t = Instant::now();
                    let tokens = crate::v1_compiler_tokenize::tokenize(
                        source.content.clone(),
                        source.path.clone(),
                    );
                    let elapsed = t.elapsed();
                    eprintln!(
                        "  tokenize {:>40}: {:>8.2?}  ({:>5} tokens, {:>6} chars)",
                        source.path,
                        elapsed,
                        tokens.len(),
                        source.content.len()
                    );
                    token_lists.push(tokens);
                }
                let tokenize_total = t_stage.elapsed();
                let rss_after_tokenize = get_rss_bytes();
                eprintln!(
                    "  TOKENIZE TOTAL: {:?}  | RSS: {}  | diags: {}\n",
                    tokenize_total,
                    format_bytes(rss_after_tokenize),
                    phase1_diags
                );

                let t_stage = Instant::now();
                let mut modules = Vec::new();
                let mut intern_table_p = crate::v1_std_core::empty_intern_table();
                let mut phase2_diags = 0usize;
                for (i, tokens) in token_lists.iter().enumerate() {
                    let t = Instant::now();
                    let si = crate::v1_std_core::build_newline_index(
                        sources[i].path.clone(),
                        sources[i].content.clone(),
                    );
                    let parsed = crate::v1_compiler_parse::parse_with_table(
                        tokens.clone(),
                        crate::v1_rt::rc_map_insert(
                            crate::v1_rt::rc_empty_map::<
                                String,
                                std::rc::Rc<crate::v1_std_core::NewlineIndex>,
                            >(),
                            si.file.clone(),
                            si.clone(),
                        ),
                        intern_table_p.clone(),
                    );
                    let result = parsed.result.clone();
                    intern_table_p = parsed.intern_table.clone();
                    let elapsed = t.elapsed();
                    let ok = result.module.is_some();
                    if result.error.is_some() {
                        phase2_diags += 1;
                    }
                    eprintln!(
                        "  parse   {:>40}: {:>8.2?}  (ok={})",
                        sources[i].path, elapsed, ok
                    );
                    let m = result
                        .module
                        .clone()
                        .unwrap_or_else(|| panic!("parse failed for source {}", sources[i].path));
                    modules.push(m);
                }
                let parse_total = t_stage.elapsed();
                let rss_after_parse = get_rss_bytes();
                eprintln!(
                    "  PARSE TOTAL:    {:?}  | RSS: {}  | diags: {}\n",
                    parse_total,
                    format_bytes(rss_after_parse),
                    phase2_diags
                );

                let t_stage = Instant::now();
                let resolve_si = sources.iter().fold(
                    crate::v1_rt::rc_empty_map::<
                        String,
                        std::rc::Rc<crate::v1_std_core::NewlineIndex>,
                    >(),
                    |acc, s| {
                        crate::v1_rt::rc_map_insert(
                            acc,
                            s.path.clone(),
                            crate::v1_std_core::build_newline_index(
                                s.path.clone(),
                                s.content.clone(),
                            ),
                        )
                    },
                );
                let graph = crate::v1_compiler_resolve::resolve_modules(
                    std::rc::Rc::new(modules.into()),
                    resolve_si,
                );
                let resolve_total = t_stage.elapsed();
                let phase3_diags: usize = graph
                    .diagnostics
                    .iter()
                    .filter(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone()))
                    .count();
                let rss_after_resolve = get_rss_bytes();
                eprintln!(
                    "  RESOLVE TOTAL:  {:?}  | RSS: {}  | diags: {}\n",
                    resolve_total,
                    format_bytes(rss_after_resolve),
                    phase3_diags
                );

                let t_stage = Instant::now();
                let source_indices = sources.iter().fold(
                    HashMap::<String, std::rc::Rc<crate::v1_std_core::NewlineIndex>>::new(),
                    |mut acc, source| {
                        acc.insert(
                            source.path.clone(),
                            crate::v1_std_core::build_newline_index(
                                source.path.clone(),
                                source.content.clone(),
                            ),
                        );
                        acc
                    },
                );
                let typed = crate::v1_compiler_infer::reconcile(
                    graph,
                    std::rc::Rc::new(source_indices),
                    intern_table_p.clone(),
                );
                let reconcile_total = t_stage.elapsed();
                let phase4_diags: usize = typed
                    .diagnostics
                    .iter()
                    .filter(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone()))
                    .count();
                let rss_after_reconcile = get_rss_bytes();
                eprintln!(
                    "  RECONCILE TOTAL: {:?}  | RSS: {}  | diags: {}\n",
                    reconcile_total,
                    format_bytes(rss_after_reconcile),
                    phase4_diags
                );

                let t_stage = Instant::now();
                let emit_result = crate::v1_compiler_emit_rust::emit_rust(typed);
                let emit_total = t_stage.elapsed();
                let phase5_diags: usize = emit_result
                    .diagnostics
                    .iter()
                    .filter(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone()))
                    .count();
                let emitted_files = emit_result.files.len();
                let emitted_bytes: usize = emit_result.files.iter().map(|f| f.content.len()).sum();
                let rss_after_emit = get_rss_bytes();
                eprintln!(
                    "  EMIT TOTAL:     {:?}  | RSS: {}  | diags: {}\n",
                    emit_total,
                    format_bytes(rss_after_emit),
                    phase5_diags
                );

                let total =
                    tokenize_total + parse_total + resolve_total + reconcile_total + emit_total;
                let total_diags =
                    phase1_diags + phase2_diags + phase3_diags + phase4_diags + phase5_diags;
                eprintln!("=== SUMMARY ===");
                eprintln!("  Tokenize:   {:?}", tokenize_total);
                eprintln!("  Parse:      {:?}", parse_total);
                eprintln!("  Resolve:    {:?}", resolve_total);
                eprintln!("  Reconcile:  {:?}", reconcile_total);
                eprintln!("  Emit:       {:?}", emit_total);
                eprintln!("  Total:      {:?}", total);
                eprintln!("  Diagnostics: {}", total_diags);
                eprintln!(
                    "  Emitted: {} files, {}",
                    emitted_files,
                    format_bytes(emitted_bytes as u64)
                );
                eprintln!("");
                eprintln!("=== RSS CHECKPOINTS ===");
                eprintln!("  Start:          {}", format_bytes(rss_start));
                eprintln!("  After tokenize: {}", format_bytes(rss_after_tokenize));
                eprintln!("  After parse:    {}", format_bytes(rss_after_parse));
                eprintln!("  After resolve:  {}", format_bytes(rss_after_resolve));
                eprintln!("  After reconcile:{}", format_bytes(rss_after_reconcile));
                eprintln!("  After emit:     {}", format_bytes(rss_after_emit));
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("profile_self_compile test panicked");
    }

    #[test]
    #[ignore]
    fn profile_full_pipeline() {
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                use std::time::Instant;

                let sources = self_compile_sources();
                let source_count = sources.len();
                let total_chars: usize = sources.iter().map(|s| s.content.len()).sum();

                eprintln!(
                    "\n=== FULL PIPELINE PROFILE ({} sources, {} chars) ===\n",
                    source_count, total_chars
                );

                // 1a. Tokenize + parse
                let t = Instant::now();
                let mut token_lists = Vec::new();
                for source in &sources {
                    let tokens = crate::v1_compiler_tokenize::tokenize(
                        source.content.clone(),
                        source.path.clone(),
                    );
                    token_lists.push(tokens);
                }
                let tokenize_elapsed = t.elapsed();

                let t = Instant::now();
                let mut modules = Vec::new();
                let mut intern_table_fp = crate::v1_std_core::empty_intern_table();
                for (i, tokens) in token_lists.iter().enumerate() {
                    let si = crate::v1_std_core::build_newline_index(
                        sources[i].path.clone(),
                        sources[i].content.clone(),
                    );
                    let parsed = crate::v1_compiler_parse::parse_with_table(
                        tokens.clone(),
                        crate::v1_rt::rc_map_insert(
                            crate::v1_rt::rc_empty_map::<
                                String,
                                std::rc::Rc<crate::v1_std_core::NewlineIndex>,
                            >(),
                            si.file.clone(),
                            si.clone(),
                        ),
                        intern_table_fp.clone(),
                    );
                    let result = parsed.result.clone();
                    intern_table_fp = parsed.intern_table.clone();
                    let m = result
                        .module
                        .clone()
                        .unwrap_or_else(|| panic!("parse failed for source {}", sources[i].path));
                    modules.push(m);
                }
                let parse_elapsed = t.elapsed();
                // 1b. Resolve
                let t = Instant::now();
                let resolve_si = sources.iter().fold(
                    crate::v1_rt::rc_empty_map::<
                        String,
                        std::rc::Rc<crate::v1_std_core::NewlineIndex>,
                    >(),
                    |acc, s| {
                        crate::v1_rt::rc_map_insert(
                            acc,
                            s.path.clone(),
                            crate::v1_std_core::build_newline_index(
                                s.path.clone(),
                                s.content.clone(),
                            ),
                        )
                    },
                );
                let graph = crate::v1_compiler_resolve::resolve_modules(
                    std::rc::Rc::new(modules.into()),
                    resolve_si,
                );
                let resolve_elapsed = t.elapsed();
                let resolve_errors: usize = graph
                    .diagnostics
                    .iter()
                    .filter(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone()))
                    .count();

                // 1c. Newline indices
                let t = Instant::now();
                let newline_indices: Vec<_> = sources
                    .iter()
                    .map(|s| {
                        crate::v1_std_core::build_newline_index(s.path.clone(), s.content.clone())
                    })
                    .collect();
                let newline_elapsed = t.elapsed();

                let frontend_elapsed =
                    tokenize_elapsed + parse_elapsed + resolve_elapsed + newline_elapsed;
                eprintln!("  Tokenize:                     {:>8.2?}", tokenize_elapsed);
                eprintln!("  Parse:                        {:>8.2?}", parse_elapsed);
                eprintln!(
                    "  Resolve:                      {:>8.2?}  ({} resolve errors)",
                    resolve_elapsed, resolve_errors
                );
                eprintln!("  Newline indices:               {:>8.2?}", newline_elapsed);
                eprintln!("  Frontend total:               {:>8.2?}", frontend_elapsed);

                // 2. Normalize
                let t = Instant::now();
                let source_indices = newline_indices.iter().cloned().fold(
                    crate::v1_rt::rc_empty_map::<
                        String,
                        std::rc::Rc<crate::v1_std_core::NewlineIndex>,
                    >(),
                    |acc, index| {
                        crate::v1_rt::rc_map_insert(acc, index.file.clone(), index.clone())
                    },
                );
                let norm = crate::v1_compiler_normalize::normalize_graph(
                    graph.clone(),
                    source_indices.clone(),
                );
                let normalize_elapsed = t.elapsed();
                eprintln!(
                    "  Normalize:                    {:>8.2?}",
                    normalize_elapsed
                );

                // 3. Reconcile
                let t = Instant::now();
                let typed = crate::v1_compiler_infer::reconcile(
                    norm.graph.clone(),
                    source_indices.clone(),
                    intern_table_fp.clone(),
                );
                let reconcile_elapsed = t.elapsed();
                let type_errors: usize = typed
                    .diagnostics
                    .iter()
                    .filter(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone()))
                    .count();
                eprintln!(
                    "  Reconcile:                    {:>8.2?}  ({} type errors)",
                    reconcile_elapsed, type_errors
                );

                // 4. Complexity
                let t = Instant::now();
                let func_entries = crate::v1_compiler_compile::extract_func_entries(typed.clone());
                let func_count = func_entries.len();
                let recursion_ctx =
                    crate::v1_compiler_compile::build_recursion_context(typed.clone());
                let complexity = crate::v1_compiler_complexity::build_complexity_report(
                    func_entries,
                    recursion_ctx,
                    source_indices.clone(),
                );
                let complexity_elapsed = t.elapsed();
                let cx_diags =
                    crate::v1_compiler_compile::complexity_diagnostics(complexity.clone());
                eprintln!(
                    "  Complexity:                   {:>8.2?}  ({} funcs, {} diagnostics)",
                    complexity_elapsed,
                    func_count,
                    cx_diags.len()
                );

                // 5. Ownership
                let t = Instant::now();
                let ownership = crate::v1_compiler_compile::extract_ownership_proofs(typed.clone());
                let ownership_elapsed = t.elapsed();
                let own_diags =
                    crate::v1_compiler_compile::ownership_diagnostics(ownership.clone());
                eprintln!(
                    "  Ownership:                    {:>8.2?}  ({} proofs, {} diagnostics)",
                    ownership_elapsed,
                    ownership.len(),
                    own_diags.len()
                );

                // 6. Emit
                let artifact_plan = crate::v1_compiler_compile::default_artifact_plan(
                    std::rc::Rc::new(
                        typed
                            .modules
                            .iter()
                            .map(|m| m.module.clone().name.clone())
                            .collect(),
                    ),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                let t = Instant::now();
                let emit_result = crate::v1_compiler_compile::emit_from_artifact_plan(
                    crate::v1_compiler_compile::emittable_graph_from_graph(typed.clone()),
                    artifact_plan,
                );
                let emit_elapsed = t.elapsed();
                let emitted_files = emit_result.files.len();
                let emitted_bytes: usize = emit_result.files.iter().map(|f| f.content.len()).sum();
                eprintln!(
                    "  Emit:                         {:>8.2?}  ({} files, {} bytes)",
                    emit_elapsed, emitted_files, emitted_bytes
                );

                let total = frontend_elapsed
                    + normalize_elapsed
                    + reconcile_elapsed
                    + complexity_elapsed
                    + ownership_elapsed
                    + emit_elapsed;
                eprintln!("\n=== SUMMARY ===");
                eprintln!(
                    "  Tokenize:   {:>8.2?}  ({:.0}%)",
                    tokenize_elapsed,
                    tokenize_elapsed.as_secs_f64() / total.as_secs_f64() * 100.0
                );
                eprintln!(
                    "  Parse:      {:>8.2?}  ({:.0}%)",
                    parse_elapsed,
                    parse_elapsed.as_secs_f64() / total.as_secs_f64() * 100.0
                );
                eprintln!(
                    "  Resolve:    {:>8.2?}  ({:.0}%)",
                    resolve_elapsed,
                    resolve_elapsed.as_secs_f64() / total.as_secs_f64() * 100.0
                );
                eprintln!(
                    "  Newline:    {:>8.2?}  ({:.0}%)",
                    newline_elapsed,
                    newline_elapsed.as_secs_f64() / total.as_secs_f64() * 100.0
                );
                eprintln!(
                    "  Normalize:  {:>8.2?}  ({:.0}%)",
                    normalize_elapsed,
                    normalize_elapsed.as_secs_f64() / total.as_secs_f64() * 100.0
                );
                eprintln!(
                    "  Reconcile:  {:>8.2?}  ({:.0}%)",
                    reconcile_elapsed,
                    reconcile_elapsed.as_secs_f64() / total.as_secs_f64() * 100.0
                );
                eprintln!(
                    "  Complexity: {:>8.2?}  ({:.0}%)",
                    complexity_elapsed,
                    complexity_elapsed.as_secs_f64() / total.as_secs_f64() * 100.0
                );
                eprintln!(
                    "  Ownership:  {:>8.2?}  ({:.0}%)",
                    ownership_elapsed,
                    ownership_elapsed.as_secs_f64() / total.as_secs_f64() * 100.0
                );
                eprintln!(
                    "  Emit:       {:>8.2?}  ({:.0}%)",
                    emit_elapsed,
                    emit_elapsed.as_secs_f64() / total.as_secs_f64() * 100.0
                );
                eprintln!("  TOTAL:      {:>8.2?}", total);
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("profile_full_pipeline panicked");
    }

    #[test]
    #[ignore]
    fn profile_reconcile_per_module() {
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                use im::HashMap;
                use std::time::Instant;

                let sources = self_compile_sources();

                eprintln!(
                    "\n=== PER-MODULE RECONCILE PROFILE ({} sources) ===",
                    sources.len()
                );

                let t0 = Instant::now();
                let mut modules = Vec::new();
                let mut intern_table = crate::v1_std_core::empty_intern_table();
                for source in &sources {
                    let tokens = crate::v1_compiler_tokenize::tokenize(
                        source.content.clone(),
                        source.path.clone(),
                    );
                    let si = crate::v1_std_core::build_newline_index(
                        source.path.clone(),
                        source.content.clone(),
                    );
                    let parsed = crate::v1_compiler_parse::parse_with_table(
                        tokens.clone(),
                        crate::v1_rt::rc_map_insert(
                            crate::v1_rt::rc_empty_map::<
                                String,
                                std::rc::Rc<crate::v1_std_core::NewlineIndex>,
                            >(),
                            si.file.clone(),
                            si.clone(),
                        ),
                        intern_table.clone(),
                    );
                    let result = parsed.result.clone();
                    intern_table = parsed.intern_table.clone();
                    let m = result
                        .module
                        .clone()
                        .unwrap_or_else(|| panic!("parse failed for source {}", source.path));
                    modules.push(m);
                }
                let resolve_si = sources.iter().fold(
                    crate::v1_rt::rc_empty_map::<
                        String,
                        std::rc::Rc<crate::v1_std_core::NewlineIndex>,
                    >(),
                    |acc, s| {
                        crate::v1_rt::rc_map_insert(
                            acc,
                            s.path.clone(),
                            crate::v1_std_core::build_newline_index(
                                s.path.clone(),
                                s.content.clone(),
                            ),
                        )
                    },
                );
                let graph = crate::v1_compiler_resolve::resolve_modules(
                    std::rc::Rc::new(modules.into()),
                    resolve_si,
                );
                let setup_time = t0.elapsed();
                let rss_baseline = get_rss_bytes();
                eprintln!(
                    "  Setup (tok+parse+resolve): {:?}  | RSS: {}",
                    setup_time,
                    format_bytes(rss_baseline)
                );
                eprintln!("  Modules to reconcile: {}\n", graph.modules.len());

                let mut mi_raw = HashMap::<
                    String,
                    std::rc::Rc<crate::v1_compiler_infer_items::TypedModule>,
                >::new();
                let source_indices = std::rc::Rc::new(sources.iter().fold(
                    HashMap::<String, std::rc::Rc<crate::v1_std_core::NewlineIndex>>::new(),
                    |mut acc, source| {
                        acc.insert(
                            source.path.clone(),
                            crate::v1_std_core::build_newline_index(
                                source.path.clone(),
                                source.content.clone(),
                            ),
                        );
                        acc
                    },
                ));
                let mut variant_surfaces = crate::v1_rt::rc_empty_map::<
                    String,
                    std::rc::Rc<crate::v1_compiler_infer::VariantExportSurface>,
                >();

                for resolved in graph.modules.iter() {
                    let name = resolved.module.name.to_string();
                    let item_count =
                        crate::v1_std_core::module_items(resolved.module.clone()).len();
                    let rss_before = get_rss_bytes();

                    eprint!("  {:>35} ({:>3} items) ... ", name, item_count);

                    let module_index = std::rc::Rc::new(mi_raw.clone());

                    let t_unres = Instant::now();
                    let _unres = crate::v1_compiler_infer::build_type_env_unresolved(
                        resolved.clone(),
                        module_index.clone(),
                        source_indices.clone(),
                        intern_table.clone(),
                    );
                    let unres_elapsed = t_unres.elapsed();
                    let rss_after_unres = get_rss_bytes();
                    let unres_delta = rss_after_unres.saturating_sub(rss_before);

                    eprint!(
                        "cycles={:>8.2?}(+{}) ",
                        unres_elapsed,
                        format_bytes(unres_delta)
                    );

                    if unres_delta > 256 * 1024 * 1024 {
                        eprintln!("");
                        panic!(
                            "ABORT: '{}' cycle detection grew RSS by {}",
                            name,
                            format_bytes(unres_delta)
                        );
                    }

                    let t_env = Instant::now();
                    let env_result = crate::v1_compiler_infer::build_type_env(
                        resolved.clone(),
                        module_index.clone(),
                        source_indices.clone(),
                        intern_table.clone(),
                        crate::v1_compiler_infer_env::empty_symbol_index(),
                    );
                    let env_elapsed = t_env.elapsed();
                    let rss_after_env = get_rss_bytes();
                    let env_delta = rss_after_env.saturating_sub(rss_before);
                    let env_errs: usize = env_result
                        .diagnostics
                        .iter()
                        .filter(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone()))
                        .count();

                    eprint!(
                        "env={:>8.2?}(+{},e={}) ",
                        env_elapsed,
                        format_bytes(env_delta),
                        env_errs
                    );

                    if env_delta > 512 * 1024 * 1024 {
                        eprintln!("");
                        panic!(
                            "ABORT: '{}' build_type_env grew RSS by {}",
                            name,
                            format_bytes(env_delta)
                        );
                    }
                    if env_elapsed.as_secs() > 10 {
                        eprintln!("");
                        panic!("ABORT: '{}' build_type_env took {:?}", name, env_elapsed);
                    }

                    let t_full = Instant::now();
                    let tc_result = crate::v1_compiler_infer::typecheck_module(
                        resolved.clone(),
                        module_index.clone(),
                        variant_surfaces.clone(),
                        source_indices.clone(),
                        intern_table.clone(),
                        crate::v1_compiler_infer_env::empty_symbol_index(),
                        crate::v1_rt::rc_empty_map::<
                            String,
                            std::rc::Rc<crate::v1_compiler_infer_env::TypeBinding>,
                        >(),
                    );
                    let full_elapsed = t_full.elapsed();
                    let rss_after = get_rss_bytes();
                    let delta = rss_after.saturating_sub(rss_before);
                    let diag_count: usize = tc_result
                        .diagnostics
                        .iter()
                        .filter(|d| crate::v1_std_core::is_error_diagnostic(d.diagnostic.clone()))
                        .count();

                    eprintln!(
                        "full={:>8.2?}  | RSS: {} (+{})  | errs: {}",
                        full_elapsed,
                        format_bytes(rss_after),
                        format_bytes(delta),
                        diag_count
                    );

                    if delta > 512 * 1024 * 1024 {
                        panic!(
                            "ABORT: '{}' grew RSS by {} (>512MB)",
                            name,
                            format_bytes(delta)
                        );
                    }
                    if full_elapsed.as_secs() > 10 {
                        panic!("ABORT: '{}' took {:?} (>10s)", name, full_elapsed);
                    }

                    let typed = tc_result.typed.clone();
                    let typed_path = crate::v1_std_core::authored_name_at(
                        source_indices.clone(),
                        typed.module.clone(),
                    );
                    variant_surfaces = crate::v1_rt::rc_map_insert(
                        variant_surfaces.clone(),
                        typed_path,
                        crate::v1_compiler_infer::build_variant_export_surface(
                            typed.clone(),
                            source_indices.clone(),
                        ),
                    );
                    mi_raw.insert(name, typed);
                }

                let rss_final = get_rss_bytes();
                eprintln!(
                    "\n  RSS final: {} (from baseline: +{})",
                    format_bytes(rss_final),
                    format_bytes(rss_final.saturating_sub(rss_baseline))
                );
                eprintln!("=== DONE ===\n");
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("profile_reconcile_per_module panicked");
    }
}
