//! Live witness: v4 `build_parse_table` / `parse` under the v2 interpreter.
//!
//! **CI (#4957 ExpectFail defer):** `scripts/v4-interpreted-parse-termination-expect-fail-gate.sh`
//! runs this witness CI-live (honest budget RED → defer ::notice; stale-fail-closed if GREEN).
//! Oracle (`fold_list_native_semantics_test`) gates merge. Dissolve-on: node://adhoc-fc63cf25-e45.
//!
//! Without native `fold_list`/`fold_list_right` fast paths, `bisect_parse_terminates` exceeds
//! its wall budget (O(n) interpreter frames per list element in the grammar fold hot loop).
//! Authority: `validate_ingest_staging_stage_bisect.dag` (provenance: #4953, #4954).

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult, SourceFile};
use v2_compiler::v2_interpreter::{self, InterpContext, Value};

use crate::helpers::{resolve_imports_transitively_with_source_roots, workspace_root};

const BISECT_ENTRY: &str = "src/v4/test/claim/manual/validate_ingest_staging_stage_bisect.dag";
const BISECT_PARSE_FN: &str = "bisect_parse_terminates";
const WITNESS_BUDGET: Duration = Duration::from_secs(30);

fn v4_source_roots() -> Vec<std::path::PathBuf> {
    vec![workspace_root().join("src/v4")]
}

fn bisect_source_pairs() -> &'static Vec<(String, String)> {
    static CACHE: OnceLock<Vec<(String, String)>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let entry_content = std::fs::read_to_string(workspace_root().join(BISECT_ENTRY))
            .unwrap_or_else(|e| panic!("read {BISECT_ENTRY}: {e}"));
        resolve_imports_transitively_with_source_roots(
            BISECT_ENTRY,
            &entry_content,
            &v4_source_roots(),
        )
        .iter()
        .map(|s| (s.path.clone(), s.content.clone()))
        .collect()
    })
}

fn bisect_sources() -> Vec<Rc<SourceFile>> {
    bisect_source_pairs()
        .iter()
        .map(|(path, content)| {
            Rc::new(SourceFile {
                path: path.clone(),
                content: content.clone(),
            })
        })
        .collect()
}

thread_local! {
    static BISECT_CTX: RefCell<Option<InterpContext>> = const { RefCell::new(None) };
}

fn assert_resolved_ok(resolved: &ResolvedPipelineResult) {
    let msgs: Vec<String> = resolved
        .diagnostics
        .iter()
        .map(|d| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))
        .filter(|m| !m.starts_with("complexity: "))
        .collect();
    assert!(
        msgs.is_empty() && resolved.graph.is_some(),
        "expected resolved graph for {BISECT_ENTRY}, got {msgs:?}"
    );
}

fn with_bisect_ctx<F, R>(f: F) -> R
where
    F: FnOnce(&InterpContext) -> R,
{
    BISECT_CTX.with(|cell| {
        if cell.borrow().is_none() {
            let resolved = compile_to_resolved(Rc::new(bisect_sources()));
            assert_resolved_ok(&resolved);
            let graph = resolved.graph.as_ref().expect("graph");
            *cell.borrow_mut() = Some(InterpContext::new(
                graph,
                resolved.source_indices.clone(),
                false,
            ));
        }
        f(cell.borrow().as_ref().expect("bisect ctx"))
    })
}

/// Primary witness — CI-live via ExpectFail defer gate (honest budget RED expected on arm64).
#[test]
fn interpreted_parse_bisect_parse_terminates_within_budget() {
    with_bisect_ctx(|ctx| {
        v2_interpreter::fold_native_hit_counts_reset();
        let start = Instant::now();
        match v2_interpreter::run_in_context(ctx, BISECT_PARSE_FN, false) {
            Ok(Value::Bool(true)) => {}
            other => panic!("expected Bool(true) from {BISECT_PARSE_FN}, got {other:?}"),
        }
        let elapsed = start.elapsed();
        let (fold_list, fold_list_right) = v2_interpreter::fold_native_hit_counts_snapshot();
        eprintln!(
            "witness {BISECT_PARSE_FN} elapsed {elapsed:?} (budget {:?}); native_fold_hits fold_list={fold_list} fold_list_right={fold_list_right}",
            WITNESS_BUDGET
        );
        assert!(
            fold_list > 0 && fold_list_right > 0,
            "native fold fast paths must be engaged during {BISECT_PARSE_FN} (FLAG1); \
             got fold_list={fold_list} fold_list_right={fold_list_right}"
        );
        assert!(
            elapsed <= WITNESS_BUDGET,
            "{BISECT_PARSE_FN} exceeded {:?} budget (elapsed {elapsed:?})",
            WITNESS_BUDGET
        );
    });
}

/// Perturbation teeth — proven manually; CI-wire on dissolve-on (node://adhoc-fc63cf25-e45).
#[test]
#[ignore = "teeth-deferred-not-CI-wired: native-off must exceed budget on same in-timer path"]
fn interpreted_parse_witness_exceeds_budget_with_native_disabled() {
    let resolved = compile_to_resolved(Rc::new(bisect_sources()));
    assert_resolved_ok(&resolved);
    let graph = resolved.graph.as_ref().expect("graph");
    let ctx = InterpContext::new(graph, resolved.source_indices.clone(), false);

    unsafe { std::env::set_var("GUNBC_INTERP_DISABLE_FOLD_NATIVE", "1") };
    v2_interpreter::fold_native_hit_counts_reset();
    let start = Instant::now();
    let run_result = v2_interpreter::run_in_context(&ctx, BISECT_PARSE_FN, false);
    let elapsed = start.elapsed();
    let (fold_list, fold_list_right) = v2_interpreter::fold_native_hit_counts_snapshot();
    unsafe { std::env::remove_var("GUNBC_INTERP_DISABLE_FOLD_NATIVE") };

    assert_eq!(
        (fold_list, fold_list_right),
        (0, 0),
        "perturbation path must not invoke native fold fast paths; \
         got fold_list={fold_list} fold_list_right={fold_list_right}"
    );

    assert!(
        elapsed > WITNESS_BUDGET,
        "expected interpreted path to exceed {:?} budget without native-fold (elapsed {elapsed:?}); \
         run_result={run_result:?}",
        WITNESS_BUDGET
    );
}
