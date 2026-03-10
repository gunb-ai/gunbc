// Binary entrypoint — eprintln is used for user-facing CLI diagnostics.
#![allow(clippy::disallowed_macros)]

//! Pure Rust testgen binary.
//!
//! Discovers all compilable `.dag` modules, renders auto-generated tests,
//! and writes them with a content-freshness check (content_upsert semantics).

use gunbc_codegen::testgen_dag::{
    auto_testgen_for_module, discover_compilable_modules, AutoTestgenResult, CompilableModule,
    RenderedTestgenModule,
};
use gunbc_ir::WorkspaceLayout;
use std::path::Path;
use std::process;

/// Modules excluded from auto-testgen because they use DSL features the
/// lowerer does not yet support (higher-order fn params, generic type params
/// as values, service calls in `for` loops, associated output types).
///
/// Remove entries as the lowerer catches up. See POSTMORTEM.md §"std/patterns.dag".
const TESTGEN_SKIP_MODULES: &[&str] = &["std.patterns", "gunbc.auth.patterns"];

/// Workspace-relative path to the generated-tests source directory.
///
/// The testgen binary writes auto-generated test modules here, inside the
/// tracked `gunbc-tests` crate.
// TODO(T14): promote to WorkspaceLayout accessor once the generated-tests
// crate layout is stabilized.
const GENERATED_TESTS_SRC_REL: &str = "src/10_test/generated-tests/src/generated";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run");

    let layout = WorkspaceLayout::from_env_manifest_dir()
        .or_else(|_| WorkspaceLayout::from_cargo_metadata())
        .unwrap_or_else(|e| {
            eprintln!("error: workspace layout: {e}");
            process::exit(1);
        });

    let dsl_root = layout.dsl_root();
    let output_dir = layout.workspace_root.join(GENERATED_TESTS_SRC_REL);

    let all_modules = discover_compilable_modules(&dsl_root).unwrap_or_else(|e| {
        eprintln!("error: module discovery failed: {e}");
        process::exit(1);
    });
    let skipped: Vec<_> = all_modules
        .iter()
        .filter(|m| TESTGEN_SKIP_MODULES.contains(&m.module_name.as_str()))
        .collect();
    for m in &skipped {
        eprintln!(
            "testgen: skipping {} (in TESTGEN_SKIP_MODULES — lowerer gaps)",
            m.module_name
        );
    }
    let modules: Vec<_> = all_modules
        .into_iter()
        .filter(|m| !TESTGEN_SKIP_MODULES.contains(&m.module_name.as_str()))
        .collect();
    let total = modules.len();

    let rendered_modules = collect_rendered_modules(&modules, &output_dir).unwrap_or_else(|errs| {
        eprintln!("error: auto-testgen failed for {} module(s):", errs.len());
        for (module_name, reason) in errs {
            eprintln!("  {module_name}");
            for line in reason.lines() {
                eprintln!("    {line}");
            }
        }
        process::exit(1);
    });

    let mut written = 0usize;
    let mut fresh = 0usize;
    for rendered in &rendered_modules {
        let path = Path::new(&rendered.path);

        if dry_run {
            let existing = std::fs::read_to_string(path).unwrap_or_default();
            if existing == rendered.content {
                fresh += 1;
            } else {
                eprintln!("  would write: {}", rendered.path);
                written += 1;
            }
        } else {
            let existing = std::fs::read_to_string(path).unwrap_or_default();
            if existing == rendered.content {
                fresh += 1;
            } else {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                std::fs::write(path, &rendered.content).unwrap_or_else(|e| {
                    eprintln!("error: cannot write {}: {e}", rendered.path);
                    process::exit(1);
                });
                eprintln!("  wrote: {}", rendered.path);
                written += 1;
            }
        }
    }

    eprintln!(
        "testgen: {total} modules, {written} written, {fresh} fresh{}",
        if dry_run { " (dry-run)" } else { "" }
    );
}

fn collect_rendered_modules(
    modules: &[CompilableModule],
    output_dir: &Path,
) -> Result<Vec<RenderedTestgenModule>, Vec<(String, String)>> {
    let mut rendered = Vec::with_capacity(modules.len());
    let mut failures = Vec::new();

    for module in modules {
        match auto_testgen_for_module(module, output_dir) {
            AutoTestgenResult::Generated {
                target_def,
                test_code,
            } => rendered.push(RenderedTestgenModule {
                content: test_code,
                path: target_def.output_path.into_owned(),
            }),
            AutoTestgenResult::Skipped { reason } => {
                failures.push((module.module_name.clone(), reason));
            }
        }
    }

    if failures.is_empty() {
        Ok(rendered)
    } else {
        Err(failures)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_rendered_modules_fails_closed_when_any_module_is_uncompilable() {
        let modules = vec![
            CompilableModule {
                dsl_path: "tools/bootstrap.dag".to_string(),
                module_name: "tools.bootstrap".to_string(),
                callable_count: 1,
                has_test_blocks: true,
            },
            CompilableModule {
                dsl_path: "nonexistent/fake.dag".to_string(),
                module_name: "nonexistent.fake".to_string(),
                callable_count: 1,
                has_test_blocks: false,
            },
        ];

        let output_dir = std::path::Path::new("src/10_test/generated-tests/src/generated");
        let err = collect_rendered_modules(&modules, output_dir)
            .expect_err("any uncompilable module should stop auto-testgen");

        assert!(
            err.iter()
                .any(|(module_name, reason)| module_name == "nonexistent.fake"
                    && reason.contains("compile error")),
            "expected compile failure for nonexistent module, got: {err:?}"
        );
    }
}
