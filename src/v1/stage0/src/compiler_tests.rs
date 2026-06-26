#[cfg(test)]
mod compiler_tests {
    use crate::v1_compiler_tokenize::tokenize;
    use std::collections::HashMap;

    /// Find workspace root by walking up from the current directory looking for Cargo.toml + dsl/
    fn workspace_root() -> std::path::PathBuf {
        let mut dir = std::env::current_dir().expect("no current dir");
        loop {
            if dir.join("Cargo.toml").exists() && dir.join("dsl").exists() {
                return dir;
            }
            if !dir.pop() {
                panic!("could not find workspace root (no Cargo.toml + dsl/ found)");
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

        let mut result: Vec<_> = seen.into_values().collect();
        result.sort_by(|a, b| a.path.cmp(&b.path));
        result
    }

    /// Build the self-compile source closure from src/v1 entry modules with dsl as a dependency pool.
    fn self_compile_sources() -> Vec<std::rc::Rc<crate::v1_compiler_compile::SourceFile>> {
        resolve_source_closure(discover_dag_files("src/v1"), &["src/v1", "dsl"])
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
        let result = crate::v1_compiler_parse::parse(
            tokens,
            std::rc::Rc::new(std::collections::HashMap::new()),
        );
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

                let result = crate::v1_compiler_parse::parse(
                    tokens,
                    std::rc::Rc::new(std::collections::HashMap::new()),
                );

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
                let result = crate::v1_compiler_compile::compile_sources(std::rc::Rc::new(vec![source]), crate::v1_compiler_artifact::RenderTarget::Rust);

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
                    std::rc::Rc::new(vec![module_a, module_b]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                let sole_ctor_errors: Vec<_> = result
                    .diagnostics
                    .iter()
                    .filter(|d| {
                        matches!(
                            *d.diagnostic,
                            crate::v1_std_core::CompilerDiagnostic::SoleConstructorViolation { .. }
                        )
                    })
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
                let all_errors_in_a: Vec<_> = result
                    .diagnostics
                    .iter()
                    .filter(|d| {
                        d.module_name == "module_a"
                            && matches!(
                                *d.diagnostic,
                                crate::v1_std_core::CompilerDiagnostic::SoleConstructorViolation {
                                    ..
                                }
                            )
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
                    // field-less sole_constructor record; factory + identity fn to exercise type-equality
                    content: "module module_a\ntype FieldlessFoo sole_constructor { }\nfn make_fieldless() -> FieldlessFoo { FieldlessFoo { } }\nfn identity(f: FieldlessFoo) -> FieldlessFoo { f }\n".to_string(),
                });
                let module_b = std::rc::Rc::new(crate::v1_compiler_compile::SourceFile {
                    path: "module_b.dag".to_string(),
                    content: "module module_b\nimport module_a { FieldlessFoo }\nfn bad_ctor() -> FieldlessFoo { FieldlessFoo { } }\n".to_string(),
                });
                let result = crate::v1_compiler_compile::compile_sources(
                    std::rc::Rc::new(vec![module_a, module_b]),
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                // enforcement check: cross-module construction must produce SoleConstructorViolation
                let sole_ctor_errors: Vec<_> = result
                    .diagnostics
                    .iter()
                    .filter(|d| {
                        matches!(
                            *d.diagnostic,
                            crate::v1_std_core::CompilerDiagnostic::SoleConstructorViolation { .. }
                        )
                    })
                    .collect();
                let violation_in_b = sole_ctor_errors.iter().any(|e| e.module_name == "module_b");
                // enforcement check result
                assert!(
                    violation_in_b,
                    "FieldlessFoo: expected SoleConstructorViolation in module_b, got: {:?}",
                    result.diagnostics
                );
                // leaf-classification check: no TypeMismatch on the identity fn in module_a
                let type_mismatch_in_a: Vec<_> = result
                    .diagnostics
                    .iter()
                    .filter(|d| {
                        d.module_name == "module_a"
                            && matches!(
                                *d.diagnostic,
                                crate::v1_std_core::CompilerDiagnostic::TypeMismatch { .. }
                            )
                    })
                    .collect();
                assert!(
                    type_mismatch_in_a.is_empty(),
                    "FieldlessFoo: TypeMismatch in module_a identity fn — leaf mis-classified by phantom property, got: {:?}",
                    type_mismatch_in_a
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("sole_constructor_fieldless_newtype_witness panicked");
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
                        std::rc::Rc::new(std::collections::HashMap::new()),
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
                let sources = std::rc::Rc::new(self_compile_sources());
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
                let sources = std::rc::Rc::new(self_compile_sources());
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
                let sources = std::rc::Rc::new(self_compile_sources());

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
        assert_eq!(
            coerce_primitive_type(RenderTarget::Rust, "Witness".into()),
            "Witness"
        );
        assert_eq!(
            coerce_primitive_type(RenderTarget::Rust, "witness".into()),
            "Witness"
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
            Some("std::collections::BTreeSet<{0}>".to_string())
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
            Some("std::collections::BTreeSet<{0}>".to_string())
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
        assert_eq!(is_copy(RenderTarget::Rust, "Witness".into()), Some(false));
        assert_eq!(is_copy(RenderTarget::Rust, "witness".into()), Some(false));
    }

    #[test]
    fn coercion_template_application() {
        use crate::v1_compiler_coercion::*;
        assert_eq!(
            apply_inhabitant_template1("Vec<{0}>".into(), "i64".into()),
            "Vec<i64>"
        );
        assert_eq!(
            apply_inhabitant_template1("std::collections::BTreeSet<{0}>".into(), "i64".into()),
            "std::collections::BTreeSet<i64>"
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
            children: std::rc::Rc::new(children),
            connective: crate::v1_std_core::Connective::NoConnective,
            params: std::rc::Rc::new(Vec::new()),
            inferred: None,
            return_cardinality: crate::v1_std_core::Cardinality::Required,
            uses: std::rc::Rc::new(Vec::new()),
            body: None,
            transport: None,
            properties: std::rc::Rc::new(Vec::new()),
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
        let symbol = named_type_node("Symbol");
        let diff_id = shaped_type_node("DiffId", vec![symbol.clone()]);
        assert!(
            crate::v1_compiler_emit_rust::rust_btree_set_element_ord_eligible(
                symbol.clone(),
                source_indices.clone()
            )
        );
        assert!(
            crate::v1_compiler_emit_rust::rust_btree_set_element_ord_eligible(
                diff_id.clone(),
                source_indices.clone()
            )
        );
        assert!(
            !crate::v1_compiler_emit_rust::rust_btree_set_element_ord_eligible(
                shaped_type_node("Symbol", vec![named_type_node("Float")]),
                source_indices.clone()
            )
        );
        assert!(
            !crate::v1_compiler_emit_rust::rust_btree_set_element_ord_eligible(
                shaped_type_node("DiffId", vec![named_type_node("Float")]),
                source_indices.clone()
            )
        );
        assert!(
            !crate::v1_compiler_emit_rust::rust_btree_set_element_ord_eligible(
                named_type_node("TestClaimId"),
                source_indices.clone()
            )
        );
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
                use std::collections::HashMap;
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
                    std::rc::Rc::new(modules),
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
                    std::rc::Rc::new(modules),
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
                use std::collections::HashMap;
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
                    std::rc::Rc::new(modules),
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
                        source_indices.clone(),
                        intern_table.clone(),
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

    // THROWAWAY PROBE (do not commit): characterizes the per-shard "pig" — the
    // parsed-AST footprint of the floor corpus that accumulates monotonically in
    // MultiEntryIndex.parse_cache. Measures (1) total distinct parsed nodes and the
    // Node-struct footprint (compact-repr target), (2) the Node.name global-interner
    // ceiling (total name bytes vs distinct — Node.name is an owned String, so every
    // node carries its own copy), (3) backbone (std/extdeps/compiler) vs entry-specific
    // (test) node split. Parse-side only; the typed cache adds a similar split on top.
    #[test]
    #[ignore]
    fn profile_floor_corpus_ast_pig() {
        let result = std::thread::Builder::new()
            .stack_size(256 * 1024 * 1024)
            .spawn(|| {
                use std::collections::{HashMap, HashSet};

                let mut files = discover_dag_files("src/v2");
                files.extend(discover_dag_files("dsl"));
                let mut seen_paths = HashSet::new();
                files.retain(|(p, _)| seen_paths.insert(p.clone()));
                eprintln!("\n=== FLOOR CORPUS AST PIG PROBE ({} modules) ===", files.len());

                fn walk(
                    n: &std::rc::Rc<crate::v1_std_core::Node>,
                    seen: &mut HashSet<usize>,
                    cnt: &mut usize,
                    name_occ: &mut usize,
                    total_name_bytes: &mut usize,
                    distinct_names: &mut HashSet<String>,
                    distinct_name_bytes: &mut usize,
                ) {
                    if !seen.insert(std::rc::Rc::as_ptr(n) as usize) {
                        return;
                    }
                    *cnt += 1;
                    *name_occ += 1;
                    *total_name_bytes += n.name.len();
                    if distinct_names.insert(n.name.clone()) {
                        *distinct_name_bytes += n.name.len();
                    }
                    for c in n.children.iter() {
                        walk(c, seen, cnt, name_occ, total_name_bytes, distinct_names, distinct_name_bytes);
                    }
                    for c in n.params.iter() {
                        walk(c, seen, cnt, name_occ, total_name_bytes, distinct_names, distinct_name_bytes);
                    }
                    for c in n.uses.iter() {
                        walk(c, seen, cnt, name_occ, total_name_bytes, distinct_names, distinct_name_bytes);
                    }
                    for c in n.properties.iter() {
                        walk(c, seen, cnt, name_occ, total_name_bytes, distinct_names, distinct_name_bytes);
                    }
                    if let Some(b) = &n.body {
                        walk(b, seen, cnt, name_occ, total_name_bytes, distinct_names, distinct_name_bytes);
                    }
                    if let Some(t) = &n.transport {
                        walk(t, seen, cnt, name_occ, total_name_bytes, distinct_names, distinct_name_bytes);
                    }
                    if let Some(ta) = &n.type_annotation {
                        walk(ta, seen, cnt, name_occ, total_name_bytes, distinct_names, distinct_name_bytes);
                    }
                }

                let mut total_nodes = 0usize;
                let mut name_occ = 0usize;
                let mut total_name_bytes = 0usize;
                let mut distinct_names: HashSet<String> = HashSet::new();
                let mut distinct_name_bytes = 0usize;
                let mut total_src_bytes = 0usize;
                let mut prefix_nodes: HashMap<&'static str, usize> = HashMap::new();
                let mut per_module: Vec<(String, usize)> = Vec::new();
                let mut parsed_ok = 0usize;

                for (path, content) in &files {
                    total_src_bytes += content.len();
                    let tokens =
                        crate::v1_compiler_tokenize::tokenize(content.clone(), path.clone());
                    let si = crate::v1_std_core::build_newline_index(path.clone(), content.clone());
                    let parsed = crate::v1_compiler_parse::parse_with_table(
                        tokens,
                        crate::v1_rt::rc_map_insert(
                            crate::v1_rt::rc_empty_map::<
                                String,
                                std::rc::Rc<crate::v1_std_core::NewlineIndex>,
                            >(),
                            si.file.clone(),
                            si.clone(),
                        ),
                        crate::v1_std_core::empty_intern_table(),
                    );
                    let module = match parsed.result.module.clone() {
                        Some(m) => m,
                        None => continue,
                    };
                    parsed_ok += 1;
                    let mut seen = HashSet::new();
                    let mut cnt = 0usize;
                    walk(
                        &module,
                        &mut seen,
                        &mut cnt,
                        &mut name_occ,
                        &mut total_name_bytes,
                        &mut distinct_names,
                        &mut distinct_name_bytes,
                    );
                    total_nodes += cnt;
                    per_module.push((path.clone(), cnt));
                    let bucket = if path.starts_with("dsl/std") {
                        "backbone:std"
                    } else if path.starts_with("dsl/extdeps") {
                        "backbone:extdeps"
                    } else if path.starts_with("dsl/product") {
                        "backbone:product"
                    } else if path.contains("/compiler/") {
                        "backbone:compiler"
                    } else if path.contains("test") || path.contains("/claim/") {
                        "specific:test"
                    } else {
                        "other"
                    };
                    *prefix_nodes.entry(bucket).or_insert(0) += cnt;
                }

                let node_struct_bytes = std::mem::size_of::<crate::v1_std_core::Node>();
                let mib = |b: usize| format!("{:.1} MiB", b as f64 / 1_048_576.0);

                eprintln!("  modules parsed ok           : {}/{}", parsed_ok, files.len());
                eprintln!("  total source bytes          : {}", mib(total_src_bytes));
                eprintln!("  total distinct parsed nodes : {}", total_nodes);
                eprintln!("  sizeof(Node)                : {} bytes", node_struct_bytes);
                eprintln!(
                    "  node-struct footprint        : {}  (= nodes x sizeof(Node), parse-side; compact-repr target)",
                    mib(total_nodes * node_struct_bytes)
                );
                eprintln!(
                    "  nodes per source byte (blowup): {:.1}x",
                    total_nodes as f64 / (total_src_bytes.max(1)) as f64
                );

                eprintln!("\n  -- Node.name global-interner ceiling (Lever B1) --");
                eprintln!("    name occurrences           : {}", name_occ);
                eprintln!("    distinct names             : {}", distinct_names.len());
                eprintln!("    total name bytes (owned)   : {}", mib(total_name_bytes));
                eprintln!("    distinct name bytes        : {}", mib(distinct_name_bytes));
                let cur = total_name_bytes + name_occ * 24;
                let interned = distinct_name_bytes + name_occ * 4 + distinct_names.len() * 24;
                eprintln!("    name cost now (str+hdr)    : {}", mib(cur));
                eprintln!("    name cost interned (u32)   : {}", mib(interned));
                eprintln!(
                    "    >>> interner reclaim ceiling : {} ({:.0}% of name cost) <<<",
                    mib(cur.saturating_sub(interned)),
                    100.0 * (cur.saturating_sub(interned)) as f64 / (cur.max(1)) as f64
                );

                eprintln!("\n  -- backbone vs entry-specific node split (Lever A context) --");
                let mut buckets: Vec<(&&'static str, &usize)> = prefix_nodes.iter().collect();
                buckets.sort_by(|a, b| b.1.cmp(a.1));
                let backbone: usize = prefix_nodes
                    .iter()
                    .filter(|(k, _)| k.starts_with("backbone"))
                    .map(|(_, v)| *v)
                    .sum();
                let specific: usize = prefix_nodes
                    .iter()
                    .filter(|(k, _)| k.starts_with("specific"))
                    .map(|(_, v)| *v)
                    .sum();
                for (k, v) in &buckets {
                    eprintln!(
                        "    {:<20} {:>10} nodes  ({})",
                        k,
                        v,
                        mib(*v * node_struct_bytes)
                    );
                }
                eprintln!(
                    "    => backbone {} nodes / specific {} nodes  (backbone = {:.0}% of classified)",
                    backbone,
                    specific,
                    100.0 * backbone as f64 / ((backbone + specific).max(1)) as f64
                );

                per_module.sort_by(|a, b| b.1.cmp(&a.1));
                eprintln!("\n  -- top 8 modules by node count --");
                for (p, c) in per_module.iter().take(8) {
                    eprintln!("    {:>9} nodes  {}", c, p);
                }
                eprintln!("=== DONE ===\n");
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("profile_floor_corpus_ast_pig panicked");
    }

    // THROWAWAY PROBE (do not commit): the TYPED half of the pig. Resolves a sample
    // of floor entries through ONE shared index (reproducing per-shard accumulation)
    // and walks the typed graph INCLUDING each node's inferred-type subtree
    // (InferredNode::Resolved{node}). Measures: typed node count, source-vs-type-subtree
    // split, current Rc sharing of inferred types (ptr), and the hash-cons ceiling
    // (content-distinct type-roots) — the likely real Lever B1 win.
    #[test]
    #[ignore]
    fn profile_floor_typed_pig() {
        let result = std::thread::Builder::new()
            .stack_size(512 * 1024 * 1024)
            .spawn(|| {
                use std::collections::{HashMap, HashSet};
                use std::rc::Rc;
                type N = crate::v1_std_core::Node;

                // floor binaries run from workspace root; discover_floor_corpus_rows
                // resolves source-roots/scan-dirs relative to CWD.
                std::env::set_current_dir(workspace_root()).expect("chdir to workspace root");

                let source_roots = vec!["src/v2".to_string(), "dsl".to_string()];
                let scan_dirs = vec!["dsl/test/claim".to_string()];
                let rows = crate::cli_run::discover_floor_corpus_rows(&source_roots, &scan_dirs)
                    .expect("discover floor corpus");
                let mut entries: Vec<String> = Vec::new();
                let mut seen_e = HashSet::new();
                for r in &rows {
                    if seen_e.insert(r.entry.clone()) {
                        entries.push(r.entry.clone());
                    }
                }
                let step = (entries.len() / 24).max(1);
                let sample: Vec<String> = entries.iter().step_by(step).cloned().collect();
                eprintln!(
                    "\n=== FLOOR TYPED PIG PROBE (entries={}, sampling {} thru one index) ===",
                    entries.len(),
                    sample.len()
                );

                let index = crate::cli_run::build_multi_entry_index(&source_roots);
                let mut typed_modules: HashMap<
                    usize,
                    Rc<crate::v1_compiler_infer_items::TypedModule>,
                > = HashMap::new();
                let mut resolved_ok = 0usize;
                for e in &sample {
                    if let Ok((graph, _si)) =
                        crate::cli_run::resolve_entry_with_index_for_discovery_corpus(&index, e.as_str())
                    {
                        resolved_ok += 1;
                        for tm in graph.modules.iter() {
                            typed_modules.insert(Rc::as_ptr(tm) as usize, tm.clone());
                        }
                    }
                }
                eprintln!(
                    "  resolved {}/{} sampled entries; distinct typed modules: {}",
                    resolved_ok,
                    sample.len(),
                    typed_modules.len()
                );

                struct Acc {
                    node_seen: HashSet<usize>,
                    source_nodes: usize,
                    type_nodes: usize,
                    name_occ: usize,
                    name_bytes: usize,
                    distinct_names: HashSet<String>,
                    distinct_name_bytes: usize,
                    inferred_occ: usize,
                    inferred_ptr: HashSet<usize>,
                    type_root_occ: usize,
                    type_root_ptr: HashSet<usize>,
                    type_struct_keys: HashMap<String, usize>,
                }
                impl Acc {
                    fn key(n: &Rc<N>) -> String {
                        let mut s = n.name.clone();
                        if !n.children.is_empty() || !n.params.is_empty() {
                            s.push('(');
                            for c in n.children.iter() {
                                s.push_str(&Acc::key(c));
                                s.push(',');
                            }
                            for c in n.params.iter() {
                                s.push_str(&Acc::key(c));
                                s.push(';');
                            }
                            s.push(')');
                        }
                        s
                    }
                    fn walk(&mut self, n: &Rc<N>, in_type: bool) {
                        if !self.node_seen.insert(Rc::as_ptr(n) as usize) {
                            return;
                        }
                        if in_type {
                            self.type_nodes += 1;
                        } else {
                            self.source_nodes += 1;
                        }
                        self.name_occ += 1;
                        self.name_bytes += n.name.len();
                        if self.distinct_names.insert(n.name.clone()) {
                            self.distinct_name_bytes += n.name.len();
                        }
                        for c in n.children.iter() {
                            self.walk(c, in_type);
                        }
                        for c in n.params.iter() {
                            self.walk(c, in_type);
                        }
                        for c in n.uses.iter() {
                            self.walk(c, in_type);
                        }
                        for c in n.properties.iter() {
                            self.walk(c, in_type);
                        }
                        if let Some(b) = &n.body {
                            self.walk(b, in_type);
                        }
                        if let Some(t) = &n.transport {
                            self.walk(t, in_type);
                        }
                        if let Some(ta) = &n.type_annotation {
                            self.walk(ta, in_type);
                        }
                        if let Some(inf) = &n.inferred {
                            self.inferred_occ += 1;
                            self.inferred_ptr.insert(Rc::as_ptr(inf) as usize);
                            if let Some(tn) = crate::v1_std_core::inferred_to_node(inf.clone()) {
                                if !in_type {
                                    self.type_root_occ += 1;
                                    self.type_root_ptr.insert(Rc::as_ptr(&tn) as usize);
                                    *self.type_struct_keys.entry(Acc::key(&tn)).or_insert(0) += 1;
                                }
                                self.walk(&tn, true);
                            }
                        }
                    }
                }

                let mut acc = Acc {
                    node_seen: HashSet::new(),
                    source_nodes: 0,
                    type_nodes: 0,
                    name_occ: 0,
                    name_bytes: 0,
                    distinct_names: HashSet::new(),
                    distinct_name_bytes: 0,
                    inferred_occ: 0,
                    inferred_ptr: HashSet::new(),
                    type_root_occ: 0,
                    type_root_ptr: HashSet::new(),
                    type_struct_keys: HashMap::new(),
                };
                for tm in typed_modules.values() {
                    acc.walk(&tm.module, false);
                    for it in tm.items.iter() {
                        acc.walk(it, false);
                    }
                }

                let node_struct_bytes = std::mem::size_of::<N>();
                let total_nodes = acc.source_nodes + acc.type_nodes;
                let mib = |b: usize| format!("{:.1} MiB", b as f64 / 1_048_576.0);
                eprintln!(
                    "\n  total distinct typed nodes   : {}  (source {} + type-subtree {})",
                    total_nodes, acc.source_nodes, acc.type_nodes
                );
                eprintln!(
                    "  node-struct footprint        : {}  (x sizeof(Node)={}B)",
                    mib(total_nodes * node_struct_bytes),
                    node_struct_bytes
                );
                eprintln!(
                    "  type-subtree share of nodes  : {:.0}%",
                    100.0 * acc.type_nodes as f64 / (total_nodes.max(1)) as f64
                );

                eprintln!("\n  -- inferred-type SHARING (already shared, or hash-cons-able?) --");
                eprintln!("    nodes carrying inferred      : {}", acc.inferred_occ);
                eprintln!("    distinct InferredNode (ptr)  : {}", acc.inferred_ptr.len());
                eprintln!("    type-root occurrences        : {}", acc.type_root_occ);
                eprintln!(
                    "    distinct type-roots (ptr)    : {}  (current Rc sharing)",
                    acc.type_root_ptr.len()
                );
                eprintln!(
                    "    distinct type-roots (content): {}  (hash-cons floor)",
                    acc.type_struct_keys.len()
                );
                let hashcons_dedup = acc.type_root_occ.saturating_sub(acc.type_struct_keys.len());
                eprintln!(
                    "    >>> type hash-cons ceiling   : {} dup type-roots collapsible ({:.0}% of type-roots) <<<",
                    hashcons_dedup,
                    100.0 * hashcons_dedup as f64 / (acc.type_root_occ.max(1)) as f64
                );

                eprintln!("\n  -- Node.name interner ceiling (TYPED) --");
                eprintln!("    name occurrences             : {}", acc.name_occ);
                eprintln!("    distinct names               : {}", acc.distinct_names.len());
                eprintln!("    total name bytes             : {}", mib(acc.name_bytes));
                eprintln!("    distinct name bytes          : {}", mib(acc.distinct_name_bytes));
                let cur = acc.name_bytes + acc.name_occ * 24;
                let interned =
                    acc.distinct_name_bytes + acc.name_occ * 4 + acc.distinct_names.len() * 24;
                eprintln!(
                    "    interner reclaim ceiling     : {} ({:.0}%)",
                    mib(cur.saturating_sub(interned)),
                    100.0 * (cur.saturating_sub(interned)) as f64 / (cur.max(1)) as f64
                );
                eprintln!("=== DONE ===\n");
            })
            .expect("spawn")
            .join();
        result.expect("profile_floor_typed_pig panicked");
    }

    // THROWAWAY PROBE (do not commit): locates the per-shard pig by PHASE. Reproduces
    // run_discovery_rows (resolve -> make_eval_context -> run_claim) over a sample of
    // floor entries through one index, attributing process RSS (Linux /proc VmRSS/VmHWM)
    // across the three phases. The phase with the dominant delta is the OOM lever's home;
    // distinguishes resolver (AST, measured <1 GiB) from interpreter eval state.
    #[test]
    #[ignore]
    fn profile_floor_eval_phase_rss() {
        let result = std::thread::Builder::new()
            .stack_size(512 * 1024 * 1024)
            .spawn(|| {
                use std::collections::HashMap;

                fn rss_kb(field: &str) -> u64 {
                    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
                    for line in s.lines() {
                        if let Some(rest) = line.strip_prefix(field) {
                            return rest
                                .split_whitespace()
                                .next()
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(0);
                        }
                    }
                    0
                }
                let mb = |kb: u64| format!("{:.0} MiB", kb as f64 / 1024.0);

                std::env::set_current_dir(workspace_root()).expect("chdir");
                let source_roots = vec!["src/v2".to_string(), "dsl".to_string()];
                let scan_dirs = vec!["dsl/test/claim".to_string()];
                let rows = crate::cli_run::discover_floor_corpus_rows(&source_roots, &scan_dirs)
                    .expect("discover");
                // group functions by entry, preserve first-seen order
                let mut order: Vec<String> = Vec::new();
                let mut by_entry: HashMap<String, Vec<String>> = HashMap::new();
                for r in &rows {
                    by_entry
                        .entry(r.entry.clone())
                        .or_insert_with(|| {
                            order.push(r.entry.clone());
                            Vec::new()
                        })
                        .push(r.function.clone());
                }
                let sample: Vec<String> = order.clone();
                eprintln!(
                    "\n=== FLOOR EVAL-PHASE TRANSIENT PROBE (entries={}, ALL, width=1) ===",
                    order.len()
                );
                eprintln!("  baseline VmRSS: {}", mb(rss_kb("VmRSS:")));

                // Reset VmHWM (peak RSS) to current VmRSS (clear_refs value 5, Linux >=4.0) so each
                // phase's VmHWM delta measures THAT phase's transient working-set peak, not the
                // global running max. Lets us localize a freed-after transient spike per entry+phase.
                fn reset_peak() {
                    let _ = std::fs::write("/proc/self/clear_refs", "5\n");
                }
                let kb_mib = |kb: i64| format!("{:.0} MiB", kb as f64 / 1024.0);

                let index = crate::cli_run::build_multi_entry_index(&source_roots);
                let mut spikes: Vec<(i64, &'static str, String, usize)> = Vec::new();
                let (mut sum_res_net, mut sum_eval_net) = (0i64, 0i64);
                let mut global_peak = rss_kb("VmRSS:") as i64;
                let mut peak_entry = (String::new(), String::new(), 0i64);

                for (idx, entry) in sample.iter().enumerate() {
                    let fns = by_entry.get(entry).cloned().unwrap_or_default();

                    reset_peak();
                    let rss0 = rss_kb("VmRSS:") as i64;
                    // Name the entry BEFORE resolving so an OOM-kill (cap) leaves the
                    // offender as the last line on stderr. Flushed by eprintln.
                    eprintln!("  -> [{:>3}/{}] resolve {}", idx, sample.len(), entry);
                    let resolved = crate::cli_run::resolve_entry_with_index_for_discovery_corpus(
                        &index,
                        entry.as_str(),
                    );
                    let (graph, si) = match resolved {
                        Ok(g) => g,
                        Err(_) => continue,
                    };
                    let res_peak = rss_kb("VmHWM:") as i64; // transient peak during resolve
                    let rss1 = rss_kb("VmRSS:") as i64;
                    if res_peak - rss0 > 500_000 {
                        eprintln!(
                            "     !! RESOLVE TRANSIENT {} on {}",
                            kb_mib(res_peak - rss0),
                            entry
                        );
                    }

                    reset_peak();
                    let ctx = crate::cli_run::make_eval_context(
                        graph.as_ref(),
                        si,
                        crate::v1_interpreter::ExecutionMode::Hermetic,
                    );
                    for f in &fns {
                        let _ = crate::cli_run::run_claim(&ctx, f.as_str());
                    }
                    let eval_peak = rss_kb("VmHWM:") as i64; // transient peak during ctx+eval
                    let rss3 = rss_kb("VmRSS:") as i64;
                    if eval_peak - rss1 > 500_000 {
                        eprintln!(
                            "     !! EVAL TRANSIENT {} on {} ({} fns)",
                            kb_mib(eval_peak - rss1),
                            entry,
                            fns.len()
                        );
                    }
                    drop(ctx);
                    drop(graph);

                    let res_transient = res_peak - rss0;
                    let eval_transient = eval_peak - rss1;
                    sum_res_net += rss1 - rss0;
                    sum_eval_net += rss3 - rss1;
                    if rss0 + res_transient > global_peak {
                        global_peak = rss0 + res_transient;
                        peak_entry = (entry.clone(), "resolve".to_string(), res_transient);
                    }
                    if rss1 + eval_transient > global_peak {
                        global_peak = rss1 + eval_transient;
                        peak_entry = (entry.clone(), "eval".to_string(), eval_transient);
                    }
                    if res_transient > 300_000 {
                        spikes.push((res_transient, "resolve", entry.clone(), fns.len()));
                    }
                    if eval_transient > 300_000 {
                        spikes.push((eval_transient, "eval", entry.clone(), fns.len()));
                    }
                }

                spikes.sort_by(|a, b| b.0.cmp(&a.0));
                let final_rss = rss_kb("VmRSS:") as i64;
                eprintln!(
                    "\n  PERSISTENT (net VmRSS): resolve {}  eval {}  final-resident {}",
                    kb_mib(sum_res_net),
                    kb_mib(sum_eval_net),
                    kb_mib(final_rss)
                );
                eprintln!("\n  TRANSIENT PEAKS (per-entry VmHWM above entry-start RSS, top 18):");
                for (sp, ph, e, n) in spikes.iter().take(18) {
                    eprintln!("    {:>9} | {:<7} | [{:>4} fns] {}", kb_mib(*sp), ph, n, e);
                }
                let max_res_tr = spikes.iter().filter(|s| s.1 == "resolve").map(|s| s.0).max().unwrap_or(0);
                let max_eval_tr = spikes.iter().filter(|s| s.1 == "eval").map(|s| s.0).max().unwrap_or(0);
                let res_cnt = spikes.iter().filter(|s| s.1 == "resolve").count();
                let eval_cnt = spikes.iter().filter(|s| s.1 == "eval").count();
                eprintln!("\n  BUDGET (residual check, width=1):");
                eprintln!("    persistent resident at end                 : {}", kb_mib(final_rss));
                eprintln!("    max single TRANSIENT resolve ({:>3} >300MiB)   : {}", res_cnt, kb_mib(max_res_tr));
                eprintln!("    max single TRANSIENT eval    ({:>3} >300MiB)   : {}", eval_cnt, kb_mib(max_eval_tr));
                eprintln!(
                    "    reconstructed GLOBAL PEAK                   : {}  (caches + {} {} transient on {})",
                    kb_mib(global_peak),
                    kb_mib(peak_entry.2),
                    peak_entry.1,
                    peak_entry.0
                );
                eprintln!("=== DONE ===\n");
            })
            .expect("spawn")
            .join();
        result.expect("profile_floor_eval_phase_rss panicked");
    }

    #[test]
    fn contracts_sidecar_wired_into_emit_scope() {
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let entry_pairs = discover_dag_files("dsl/extdeps/llm");
                let sources = std::rc::Rc::new(resolve_source_closure(entry_pairs, &["dsl"]));
                let result = crate::v1_compiler_compile::compile_sources(
                    sources,
                    crate::v1_compiler_artifact::RenderTarget::Rust,
                );
                let anthropic_file = result
                    .files
                    .iter()
                    .find(|f| {
                        f.path.contains("extdeps_llm_anthropic") && !f.path.contains("_contracts")
                    })
                    .expect(
                        "emitted file for extdeps.llm.anthropic not found — \
                         module must be present in dsl/extdeps/llm source closure",
                    );
                assert!(
                    anthropic_file.content.contains("#[serde(tag = \"role\""),
                    "AnthropicChatMessage serde tag annotation must be present in emitted Rust — \
                     contracts_items_for_module must be merged into emit scope; \
                     missing from: {}\nfile content (first 2000 chars):\n{}",
                    anthropic_file.path,
                    &anthropic_file.content[..anthropic_file.content.len().min(2000)]
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("contracts_sidecar_wired_into_emit_scope panicked");
    }
}
