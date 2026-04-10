#[cfg(test)]
mod compiler_tests {
    use std::collections::HashMap;
    use crate::v2_compiler_tokenize::tokenize;

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
    ) -> Vec<std::rc::Rc<crate::v2_compiler_compile::SourceFile>> {
        pairs
            .iter()
            .map(|(path, content)| {
                std::rc::Rc::new(crate::v2_compiler_compile::SourceFile {
                    path: path.clone(),
                    content: content.clone(),
                })
            })
            .collect()
    }

    /// Build the self-compile source closure: dsl/ dependencies + all src/v2/*.dag.
    fn self_compile_sources() -> Vec<std::rc::Rc<crate::v2_compiler_compile::SourceFile>> {
        let dsl_deps = &[
            "dsl/extdeps/languages/go/emit.dag",
            "dsl/extdeps/languages/python/emit.dag",
            "dsl/extdeps/languages/rust/emit.dag",
            "dsl/extdeps/languages/dag/syntax.dag",
            "dsl/std/algebra.dag",
            "dsl/std/syntax.dag",
            "dsl/std/types.dag",
            "dsl/std/verification.dag",
        ];
        let root = workspace_root();
        let mut sources: Vec<std::rc::Rc<crate::v2_compiler_compile::SourceFile>> = dsl_deps
            .iter()
            .map(|p| {
                let full = root.join(p);
                let content = std::fs::read_to_string(&full)
                    .unwrap_or_else(|e| panic!("failed to read {}: {}", full.display(), e));
                std::rc::Rc::new(crate::v2_compiler_compile::SourceFile {
                    path: p.to_string(),
                    content,
                })
            })
            .collect();

        let v2_files = discover_dag_files("src/v2");
        sources.extend(source_files_from(&v2_files));
        sources
    }

    /// Build the gist pipeline source closure.
    fn gist_sources() -> Vec<std::rc::Rc<crate::v2_compiler_compile::SourceFile>> {
        let gist_deps = &[
            "dsl/extdeps/cloud/cloud.dag",
            "dsl/extdeps/cloud/gcp/gcp.dag",
            "dsl/extdeps/git.dag",
            "dsl/extdeps/github/auth.dag",
            "dsl/extdeps/github/gists.dag",
            "dsl/extdeps/github/github.dag",
            "dsl/gunbc/auth/credentials.dag",
            "dsl/gunbc/tools/gist.dag",
            "dsl/std/algebra.dag",
            "dsl/std/errors.dag",
            "dsl/std/resources.dag",
            "dsl/std/types.dag",
        ];
        let root = workspace_root();
        gist_deps
            .iter()
            .map(|p| {
                let full = root.join(p);
                let content = std::fs::read_to_string(&full)
                    .unwrap_or_else(|e| panic!("failed to read {}: {}", full.display(), e));
                std::rc::Rc::new(crate::v2_compiler_compile::SourceFile {
                    path: p.to_string(),
                    content,
                })
            })
            .collect()
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
            matches!(last.shape, crate::v2_std_core::TokenShape::ShEof),
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
            matches!(tokens[0].shape, crate::v2_std_core::TokenShape::ShKeyword),
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
        let result = crate::v2_compiler_parse::parse(tokens, None);
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
                let source = read_dag("src/v2/01_tokenize.dag");
                let tokens = tokenize(source, "src/v2/01_tokenize.dag".to_string());

                assert!(
                    !tokens.is_empty(),
                    "tokenizing 01_tokenize.dag should produce tokens"
                );

                let last = tokens.last().expect("should have tokens");
                assert!(
                    matches!(last.shape, crate::v2_std_core::TokenShape::ShEof),
                    "last token should be Eof, got {:?}",
                    last.shape
                );

                let result = crate::v2_compiler_parse::parse(tokens, None);

                assert!(
                    result.module.is_some(),
                    "parsing 01_tokenize.dag should produce a module"
                );

                let module = result.module.as_ref().unwrap();
                assert_eq!(
                    module.name, "v2.compiler.tokenize",
                    "module name should be v2.compiler.tokenize, got {}",
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
                let source = std::rc::Rc::new(crate::v2_compiler_compile::SourceFile {
                    path: "test.dag".to_string(),
                    content: "module test\ntype Foo { x: Int, name: String }\nfn add(a: Int, b: Int) -> Int { a + b }\n".to_string(),
                });
                let result = crate::v2_compiler_compile::compile_sources(std::rc::Rc::new(vec![source]), crate::v2_compiler_artifact::RenderTarget::Rust);

                assert!(
                    !result.files.is_empty(),
                    "compile_sources should produce output files, got none"
                );

                let errors: Vec<_> = result.diagnostics.iter()
                    .filter(|d| crate::v2_std_core::is_error_diagnostic(d.diagnostic.clone()))
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
    fn self_parse_all_modules() {
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let v2_files = discover_dag_files("src/v2");
                assert!(
                    !v2_files.is_empty(),
                    "should discover at least one .dag file in src/v2/"
                );

                for (file, source) in &v2_files {
                    let tokens = tokenize(source.to_string(), file.to_string());
                    assert!(!tokens.is_empty(), "{} should produce tokens", file);
                    assert!(
                        matches!(
                            tokens.last().unwrap().shape,
                            crate::v2_std_core::TokenShape::ShEof
                        ),
                        "{} should end with Eof",
                        file
                    );
                    let result = crate::v2_compiler_parse::parse(tokens, None);
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
    fn self_compile_all_modules() {
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let sources = std::rc::Rc::new(self_compile_sources());
                let source_count = sources.len();
                let result = crate::v2_compiler_compile::compile_sources(
                    sources,
                    crate::v2_compiler_artifact::RenderTarget::Rust,
                );

                let errors: Vec<_> = result
                    .diagnostics
                    .iter()
                    .filter(|d| crate::v2_std_core::is_error_diagnostic(d.diagnostic.clone()))
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

                let result = crate::v2_compiler_compile::compile_sources(
                    sources,
                    crate::v2_compiler_artifact::RenderTarget::Rust,
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
    fn gist_resolve_all_modules() {
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let sources = std::rc::Rc::new(gist_sources());
                let result = crate::v2_compiler_compile::resolve_sources(sources);

                let errors: Vec<_> = result
                    .diagnostics
                    .iter()
                    .filter(|d| crate::v2_std_core::is_error_diagnostic(d.diagnostic.clone()))
                    .collect();
                let error_count = errors.len();

                eprintln!("gist resolve error count: {}", error_count);
                for e in &errors {
                    eprintln!("  {:?}", e);
                }

                assert!(
                    error_count == 0,
                    "gist resolve errors: {} errors (expected 0): {:?}",
                    error_count,
                    errors
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("gist-resolve-all test panicked");
    }

    #[test]
    fn gist_compile_all_modules() {
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                let sources = std::rc::Rc::new(gist_sources());
                let result = crate::v2_compiler_compile::compile_sources(
                    sources,
                    crate::v2_compiler_artifact::RenderTarget::Rust,
                );

                let errors: Vec<_> = result
                    .diagnostics
                    .iter()
                    .filter(|d| crate::v2_std_core::is_error_diagnostic(d.diagnostic.clone()))
                    .collect();
                let error_count = errors.len();

                eprintln!("gist compile error count: {}", error_count);
                for e in &errors {
                    eprintln!("  {:?}", e);
                }

                assert!(
                    error_count == 0,
                    "gist compile errors: {} errors (expected 0): {:?}",
                    error_count,
                    errors
                );

                let has_content = result.files.iter().any(|f| !f.content.is_empty());
                assert!(
                    has_content,
                    "gist compile should produce at least one non-empty file"
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("gist-compile-all test panicked");
    }

    #[test]
    fn type_size_regression_check() {
        let node_size = std::mem::size_of::<crate::v2_std_core::Node>();
        let expr_size = std::mem::size_of::<crate::v2_std_core::ExprData>();
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
        use crate::v2_compiler_coercion::*;
        assert_eq!(coerce_primitive_type(RenderTarget::Rust, "Int".into()), "i64");
        assert_eq!(coerce_primitive_type(RenderTarget::Rust, "Float".into()), "f64");
        assert_eq!(coerce_primitive_type(RenderTarget::Rust, "Bool".into()), "bool");
        assert_eq!(coerce_primitive_type(RenderTarget::Rust, "Unit".into()), "()");
        assert_eq!(coerce_primitive_type(RenderTarget::Rust, "String".into()), "String");
        assert_eq!(coerce_primitive_type(RenderTarget::Rust, "Bytes".into()), "Vec<u8>");
        assert_eq!(coerce_primitive_type(RenderTarget::Rust, "Secret".into()), "String");
        assert_eq!(coerce_primitive_type(RenderTarget::Rust, "Json".into()), "serde_json::Value");
    }


    #[test]
    fn coercion_python_checkpoint_resolves_primitives() {
        use crate::v2_compiler_coercion::*;
        assert_eq!(coerce_primitive_type(RenderTarget::Python, "Int".into()), "int");
        assert_eq!(coerce_primitive_type(RenderTarget::Python, "Float".into()), "float");
        assert_eq!(coerce_primitive_type(RenderTarget::Python, "Bool".into()), "bool");
        assert_eq!(coerce_primitive_type(RenderTarget::Python, "Unit".into()), "None");
        assert_eq!(coerce_primitive_type(RenderTarget::Python, "String".into()), "str");
        assert_eq!(coerce_primitive_type(RenderTarget::Python, "Bytes".into()), "bytes");
        assert_eq!(coerce_primitive_type(RenderTarget::Python, "Secret".into()), "str");
        assert_eq!(coerce_primitive_type(RenderTarget::Python, "Json".into()), "dict");
    }


    #[test]
    fn coercion_go_checkpoint_resolves_primitives() {
        use crate::v2_compiler_coercion::*;
        assert_eq!(coerce_primitive_type(RenderTarget::Go, "Int".into()), "int64");
        assert_eq!(coerce_primitive_type(RenderTarget::Go, "Float".into()), "float64");
        assert_eq!(coerce_primitive_type(RenderTarget::Go, "Bool".into()), "bool");
        assert_eq!(coerce_primitive_type(RenderTarget::Go, "Unit".into()), "struct{}");
        assert_eq!(coerce_primitive_type(RenderTarget::Go, "String".into()), "string");
        assert_eq!(coerce_primitive_type(RenderTarget::Go, "Bytes".into()), "[]byte");
        assert_eq!(coerce_primitive_type(RenderTarget::Go, "Secret".into()), "string");
        assert_eq!(coerce_primitive_type(RenderTarget::Go, "Json".into()), "interface{}");
    }


    #[test]
    fn coercion_rust_inhabitant_resolves_containers() {
        use crate::v2_compiler_coercion::*;
        assert_eq!(coerce_container_template(RenderTarget::Rust, "BooleanAlgebra".into()), Some("std::collections::BTreeSet<{0}>".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Rust, "FreeMonoid".into()), Some("Vec<{0}>".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Rust, "List".into()), Some("Vec<{0}>".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Rust, "Map".into()), Some("HashMap<{0}, {1}>".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Rust, "NonEmptyList".into()), Some("Vec<{0}>".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Rust, "NonEmptySet".into()), Some("std::collections::BTreeSet<{0}>".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Rust, "PartialFunction".into()), Some("HashMap<{0}, {1}>".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Rust, "Set".into()), Some("std::collections::BTreeSet<{0}>".to_string()));
    }


    #[test]
    fn coercion_python_inhabitant_resolves_containers() {
        use crate::v2_compiler_coercion::*;
        assert_eq!(coerce_container_template(RenderTarget::Python, "BooleanAlgebra".into()), Some("set[{0}]".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Python, "FreeMonoid".into()), Some("list[{0}]".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Python, "List".into()), Some("list[{0}]".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Python, "Map".into()), Some("dict[{0}, {1}]".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Python, "NonEmptyList".into()), Some("list[{0}]".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Python, "NonEmptySet".into()), Some("set[{0}]".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Python, "PartialFunction".into()), Some("dict[{0}, {1}]".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Python, "Set".into()), Some("set[{0}]".to_string()));
    }


    #[test]
    fn coercion_go_inhabitant_resolves_containers() {
        use crate::v2_compiler_coercion::*;
        assert_eq!(coerce_container_template(RenderTarget::Go, "BooleanAlgebra".into()), Some("map[{0}]struct{}".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Go, "FreeMonoid".into()), Some("[]{0}".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Go, "List".into()), Some("[]{0}".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Go, "Map".into()), Some("map[{0}]{1}".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Go, "NonEmptyList".into()), Some("[]{0}".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Go, "NonEmptySet".into()), Some("map[{0}]struct{}".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Go, "PartialFunction".into()), Some("map[{0}]{1}".to_string()));
        assert_eq!(coerce_container_template(RenderTarget::Go, "Set".into()), Some("map[{0}]struct{}".to_string()));
    }


    #[test]
    fn coercion_is_copy_from_checkpoint() {
        use crate::v2_compiler_coercion::*;
        assert_eq!(is_copy(RenderTarget::Rust, "Int".into()), Some(true));
        assert_eq!(is_copy(RenderTarget::Rust, "Float".into()), Some(true));
        assert_eq!(is_copy(RenderTarget::Rust, "Bool".into()), Some(true));
        assert_eq!(is_copy(RenderTarget::Rust, "Unit".into()), Some(true));
        assert_eq!(is_copy(RenderTarget::Rust, "String".into()), Some(false));
        assert_eq!(is_copy(RenderTarget::Rust, "Bytes".into()), Some(false));
        assert_eq!(is_copy(RenderTarget::Rust, "Secret".into()), Some(false));
        assert_eq!(is_copy(RenderTarget::Rust, "Json".into()), Some(false));
    }


    #[test]
    fn coercion_template_application() {
        use crate::v2_compiler_coercion::*;
        assert_eq!(apply_inhabitant_template1("Vec<{0}>".into(), "i64".into()), "Vec<i64>");
        assert_eq!(apply_inhabitant_template1("std::collections::BTreeSet<{0}>".into(), "i64".into()), "std::collections::BTreeSet<i64>");
        assert_eq!(apply_inhabitant_template2("HashMap<{0}, {1}>".into(), "String".into(), "i64".into()), "HashMap<String, i64>");
        assert_eq!(apply_inhabitant_template1("list[{0}]".into(), "int".into()), "list[int]");
        assert_eq!(apply_inhabitant_template1("set[{0}]".into(), "int".into()), "set[int]");
        assert_eq!(apply_inhabitant_template2("dict[{0}, {1}]".into(), "str".into(), "int".into()), "dict[str, int]");
        assert_eq!(apply_inhabitant_template1("[]{0}".into(), "int64".into()), "[]int64");
        assert_eq!(apply_inhabitant_template1("map[{0}]struct{}".into(), "int64".into()), "map[int64]struct{}");
        assert_eq!(apply_inhabitant_template2("map[{0}]{1}".into(), "string".into(), "int64".into()), "map[string]int64");
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
    fn profile_gist_pipeline() {
        let result = std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(|| {
                use std::collections::HashMap;
                use std::time::Instant;

                let sources = gist_sources();

                eprintln!(
                    "\n=== GIST PIPELINE PROFILE ({} sources) ===\n",
                    sources.len()
                );

                let t_stage = Instant::now();
                let mut token_lists = Vec::new();
                for source in &sources {
                    let t = Instant::now();
                    let tokens = crate::v2_compiler_tokenize::tokenize(
                        source.content.clone(),
                        source.path.clone(),
                    );
                    let elapsed = t.elapsed();
                    eprintln!(
                        "  tokenize {:>40}: {:>8.2?}  ({:>5} tokens, {:>5} chars)",
                        source.path,
                        elapsed,
                        tokens.len(),
                        source.content.len()
                    );
                    token_lists.push(tokens);
                }
                let tokenize_total = t_stage.elapsed();
                eprintln!("  TOKENIZE TOTAL: {:?}\n", tokenize_total);

                let t_stage = Instant::now();
                let mut modules = Vec::new();
                for (i, tokens) in token_lists.iter().enumerate() {
                    let t = Instant::now();
                    let result = crate::v2_compiler_parse::parse(tokens.clone(), None);
                    let elapsed = t.elapsed();
                    let ok = result.module.is_some();
                    eprintln!(
                        "  parse   {:>40}: {:>8.2?}  (ok={})",
                        sources[i].path, elapsed, ok
                    );
                    if let Some(m) = result.module.clone() {
                        modules.push(m);
                    }
                }
                let parse_total = t_stage.elapsed();
                eprintln!("  PARSE TOTAL:    {:?}\n", parse_total);

                let t_stage = Instant::now();
                let graph = crate::v2_compiler_resolve::resolve_modules(std::rc::Rc::new(modules));
                let resolve_total = t_stage.elapsed();
                let errors: Vec<_> = graph
                    .diagnostics
                    .iter()
                    .filter(|d| crate::v2_std_core::is_error_diagnostic(d.diagnostic.clone()))
                    .collect();
                eprintln!(
                    "  RESOLVE TOTAL:  {:?}  ({} errors)\n",
                    resolve_total,
                    errors.len()
                );

                eprintln!("=== SUMMARY ===");
                eprintln!("  Tokenize: {:?}", tokenize_total);
                eprintln!("  Parse:    {:?}", parse_total);
                eprintln!("  Resolve:  {:?}", resolve_total);
                eprintln!(
                    "  Total:    {:?}",
                    tokenize_total + parse_total + resolve_total
                );
            })
            .expect("failed to spawn thread")
            .join();
        result.expect("profile test panicked");
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
                    let tokens = crate::v2_compiler_tokenize::tokenize(
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
                let mut phase2_diags = 0usize;
                for (i, tokens) in token_lists.iter().enumerate() {
                    let t = Instant::now();
                    let result = crate::v2_compiler_parse::parse(tokens.clone(), None);
                    let elapsed = t.elapsed();
                    let ok = result.module.is_some();
                    if result.error.is_some() {
                        phase2_diags += 1;
                    }
                    eprintln!(
                        "  parse   {:>40}: {:>8.2?}  (ok={})",
                        sources[i].path, elapsed, ok
                    );
                    if let Some(m) = result.module.clone() {
                        modules.push(m);
                    }
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
                let graph = crate::v2_compiler_resolve::resolve_modules(std::rc::Rc::new(modules));
                let resolve_total = t_stage.elapsed();
                let phase3_diags: usize = graph
                    .diagnostics
                    .iter()
                    .filter(|d| crate::v2_std_core::is_error_diagnostic(d.diagnostic.clone()))
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
                    HashMap::<String, std::rc::Rc<crate::v2_std_core::NewlineIndex>>::new(),
                    |mut acc, source| {
                        acc.insert(
                            source.path.clone(),
                            crate::v2_std_core::build_newline_index(
                                source.path.clone(),
                                source.content.clone(),
                            ),
                        );
                        acc
                    },
                );
                let typed = crate::v2_compiler_infer::reconcile(graph, std::rc::Rc::new(source_indices));
                let reconcile_total = t_stage.elapsed();
                let phase4_diags: usize = typed
                    .diagnostics
                    .iter()
                    .filter(|d| crate::v2_std_core::is_error_diagnostic(d.diagnostic.clone()))
                    .count();
                let rss_after_reconcile = get_rss_bytes();
                eprintln!(
                    "  RECONCILE TOTAL: {:?}  | RSS: {}  | diags: {}\n",
                    reconcile_total,
                    format_bytes(rss_after_reconcile),
                    phase4_diags
                );

                let t_stage = Instant::now();
                let emit_result = crate::v2_compiler_emit_rust::emit_rust(typed);
                let emit_total = t_stage.elapsed();
                let phase5_diags: usize = emit_result
                    .diagnostics
                    .iter()
                    .filter(|d| crate::v2_std_core::is_error_diagnostic(d.diagnostic.clone()))
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
                for source in &sources {
                    let tokens = crate::v2_compiler_tokenize::tokenize(
                        source.content.clone(),
                        source.path.clone(),
                    );
                    let result = crate::v2_compiler_parse::parse(tokens, None);
                    if let Some(m) = result.module.clone() {
                        modules.push(m);
                    } else {
                        eprintln!("  WARN: parse failed for {}", source.path);
                    }
                }
                let graph = crate::v2_compiler_resolve::resolve_modules(std::rc::Rc::new(modules));
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
                    std::rc::Rc<crate::v2_compiler_infer_items::TypedModule>,
                >::new();
                let source_indices = std::rc::Rc::new(sources.iter().fold(
                    HashMap::<String, std::rc::Rc<crate::v2_std_core::NewlineIndex>>::new(),
                    |mut acc, source| {
                        acc.insert(
                            source.path.clone(),
                            crate::v2_std_core::build_newline_index(
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
                        crate::v2_std_core::module_items(resolved.module.clone()).len();
                    let rss_before = get_rss_bytes();

                    eprint!("  {:>35} ({:>3} items) ... ", name, item_count);

                    let module_index = std::rc::Rc::new(mi_raw.clone());

                    let t_unres = Instant::now();
                    let _unres = crate::v2_compiler_infer::build_type_env_unresolved(
                        resolved.clone(),
                        module_index.clone(),
                        source_indices.clone(),
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
                    let env_result = crate::v2_compiler_infer::build_type_env(
                        resolved.clone(),
                        module_index.clone(),
                        source_indices.clone(),
                    );
                    let env_elapsed = t_env.elapsed();
                    let rss_after_env = get_rss_bytes();
                    let env_delta = rss_after_env.saturating_sub(rss_before);
                    let env_errs: usize = env_result
                        .diagnostics
                        .iter()
                        .filter(|d| crate::v2_std_core::is_error_diagnostic(d.diagnostic.clone()))
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
                    let tc_result = crate::v2_compiler_infer::typecheck_module(
                        resolved.clone(),
                        module_index,
                        source_indices.clone(),
                    );
                    let full_elapsed = t_full.elapsed();
                    let rss_after = get_rss_bytes();
                    let delta = rss_after.saturating_sub(rss_before);
                    let diag_count: usize = tc_result
                        .diagnostics
                        .iter()
                        .filter(|d| crate::v2_std_core::is_error_diagnostic(d.diagnostic.clone()))
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

}
