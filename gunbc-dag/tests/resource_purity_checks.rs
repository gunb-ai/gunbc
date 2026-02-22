//! Registry-wide resource purity checks.
//!
//! This test links all DAG crates that currently participate in workflow
//! execution and enforces:
//! - resource access derivation succeeds
//! - no conflicting resource access pairs in the same DAG
//! - all `res:*` resource ports are wired

use gunbc_ir::resource::ResourceIo;
use gunbc_ir::{
    derive_resource_accesses, detect_resource_conflicts, validate_resource_wiring_recursive,
};
use gunbc_lib_transport::TransportIo;
use gunbc_testgen_registry::iter_resource_tests;
use std::path::Path;

// Force-link crates with `#[resource_test_target]` registrations used by CI/tooling.
//
// Most crates are force-linked via explicit symbol touches below; cloud-ops has
// no mock symbol touched in this file, so keep the import to retain its
// registration objects.
use gunbc_lib_cloud_ops as _;

#[test]
fn resource_purity_registry_wide() {
    // Touch representative symbols so linker keeps object files that contain
    // inventory submissions from graph + graph_mock modules.
    let _: fn() -> gunbc_test::MockSpec = gunbc_clippy::graph_mock::clippy_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_deps::graph_mock::deps_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_gist::graph_mock::gist_snapshot_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_gist::graph_mock::gist_diff_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_gist::graph_mock::gist_recent_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_lib_gcp_ops::graph_mock::gcp_github_mock_spec;
    let _: fn() -> gunbc_test::MockSpec =
        gunbc_lib_gcp_ops::graph_mock::gcp_github_upsert_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_lib_llm_ops::graph_mock::openai_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_lib_review::graph_mock::inline_review_mock_spec;
    let _: fn() -> gunbc_test::MockSpec = gunbc_lib_review::graph_mock::diff_review_mock_spec;

    let mut defs: Vec<_> = iter_resource_tests().collect();
    defs.sort_by(|a, b| {
        (a.origin_crate, a.name)
            .cmp(&(b.origin_crate, b.name))
            .then_with(|| a.name.cmp(b.name))
    });

    assert!(
        !defs.is_empty(),
        "no resource test targets were registered in this test binary"
    );

    let mut failures = Vec::new();

    for def in defs {
        let dag = (def.build)();

        if let Err(err) = derive_resource_accesses(&dag) {
            failures.push(format!(
                "{} ({}): derive_resource_accesses failed: {:?}",
                def.name, def.origin_crate, err
            ));
            continue;
        }

        match detect_resource_conflicts(&dag) {
            Ok(conflicts) => {
                if !conflicts.is_empty() {
                    failures.push(format!(
                        "{} ({}): {} resource conflict(s): {:?}",
                        def.name,
                        def.origin_crate,
                        conflicts.len(),
                        conflicts
                    ));
                }
            }
            Err(err) => failures.push(format!(
                "{} ({}): detect_resource_conflicts failed: {:?}",
                def.name, def.origin_crate, err
            )),
        }

        let unwired = validate_resource_wiring_recursive(&dag);
        if !unwired.is_empty() {
            failures.push(format!(
                "{} ({}): {} unwired resource port(s): {:?}",
                def.name,
                def.origin_crate,
                unwired.len(),
                unwired
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "registry-wide resource purity checks failed:\n{}",
        failures.join("\n")
    );
}

#[test]
fn resource_ids_have_no_legacy_aliases() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("gunbc-dag should live under repo root");
    let io = TransportIo::new();

    let mut files = Vec::new();
    for root in ["core", "gunbc-dag", "lib"] {
        let pattern = format!("{}/{}/**/*.rs", repo_root.display(), root);
        let Ok(paths) = io.glob_paths(&pattern) else {
            continue;
        };
        files.extend(paths.into_iter().filter(|path| {
            let path_str = path.to_string_lossy();
            !path_str.contains("/target/") && !path_str.contains("/buck-out/")
        }));
    }
    files.sort();
    files.dedup();

    let banned = vec![
        format!("res:{}", "fs"),
        format!("res:{}", "net"),
        format!("res:{}", "pkg"),
        format!("res:{}_path", "repo"),
        format!("{}:{}", "fs", "read"),
        format!("{}:{}", "fs", "write"),
        format!("resource(\"{}\"", "fs"),
        format!("resource(\"{}\"", "net"),
        format!("resource(\"{}\"", "pkg"),
        format!("resource_lock(\"{}:", "fs"),
        format!("resource_lock(\"{}:", "net"),
        format!("resource_lock(\"{}:", "pkg"),
        format!("resource_lock(\"{}:", "cargo"),
        format!("resource_lock_fails(\"{}:", "fs"),
        format!("resource_lock_fails(\"{}:", "net"),
        format!("resource_lock_fails(\"{}:", "pkg"),
        format!("resource_lock_fails(\"{}:", "cargo"),
        format!("port(\"{}\", \"{}\")", "net", "NetworkHandle"),
        format!("out(\"{}\")", "net"),
        format!("boundary(\"{}\", \"{}\"", "net_env", "net"),
        format!("set_value(\"{}\", \"{}\"", "net_env", "net"),
        format!("{}{}{}{}", "with", "_check", "_deprecated", "("),
        format!("{} {} {}", "Deprecated alias", "for --mode", "=verify"),
    ];

    let mut hits = Vec::new();
    for path in files {
        let Ok(bytes) = io.read_file(&path) else {
            continue;
        };
        let Ok(content) = String::from_utf8(bytes) else {
            continue;
        };
        for (idx, line) in content.lines().enumerate() {
            for pat in &banned {
                if line.contains(pat) {
                    let rel = path.strip_prefix(repo_root).unwrap_or(&path);
                    hits.push(format!(
                        "{}:{}: legacy resource alias '{}': {}",
                        rel.display(),
                        idx + 1,
                        pat,
                        line.trim()
                    ));
                }
            }
        }
    }

    assert!(
        hits.is_empty(),
        "legacy resource alias forms are forbidden; migrate to canonical resource ids now:\n{}",
        hits.join("\n")
    );
}
