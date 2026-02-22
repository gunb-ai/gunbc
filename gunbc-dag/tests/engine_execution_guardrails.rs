//! Guardrails for execution entrypoints.
//!
//! Roadmap alignment: forbid new ad-hoc execution paths outside the engine
//! and explicit adapter surfaces.

use gunbc_ir::resource::ResourceIo;
use gunbc_lib_transport::TransportIo;
use std::path::Path;

const FORBIDDEN_CALLS: &[&str] = &[
    "execute_with_mode_and_inputs(",
    "execute_with_mode(",
    "execute_with_progress(",
    "execute_with_progress_and_mode(",
    "execute_single_node(",
];

const ALLOWED_FILES: &[&str] = &[
    "core/exec/src/execute.rs",
    "core/daglang/daglang-emit/src/rust_exec_runtime.rs",
    "core/daglang/daglang-cli/src/compile/context.rs",
    "core/codegen/src/cli_gen.rs",
    "core/test/src/boundary.rs",
    "gunbc-dag/src/bin/infra.rs",
    "gunbc-dag/src/bin/sdlc.rs",
    "gunbc-dag/src/mock_defaults.rs",
    "gunbc-dag/src/resolve.rs", // SubDagExecutorOp: inner DAG execution surface
];

#[test]
fn no_new_direct_execution_helpers_outside_engine_surfaces() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root");
    let io = TransportIo::new();
    let pattern = format!("{}/**/*.rs", root.display());
    let paths = io
        .glob_paths(&pattern)
        .expect("workspace rust source glob should succeed");

    let mut violations = Vec::new();
    for path in paths {
        let rel = path.strip_prefix(root).unwrap_or(&path);
        let rel_norm = rel.to_string_lossy().replace('\\', "/");
        if rel_norm.starts_with("target/")
            || rel_norm.contains("/target/")
            || rel_norm.contains("/buck-out/")
            || rel_norm.starts_with("docs/")
            || rel_norm.starts_with("TODO/")
            || rel_norm.contains("/tests/")
            || rel_norm.contains("generated_tests")
            || ALLOWED_FILES.contains(&rel_norm.as_str())
        {
            continue;
        }

        let Ok(bytes) = io.read_file(&path) else {
            continue;
        };
        let Ok(content) = String::from_utf8(bytes) else {
            continue;
        };

        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            for needle in FORBIDDEN_CALLS {
                if line.contains(needle) {
                    violations.push(format!("{}:{} {}", rel_norm, idx + 1, needle));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "direct execution helper usage is restricted; add routing via engine/adapters instead:\n{}",
        violations.join("\n")
    );
}
