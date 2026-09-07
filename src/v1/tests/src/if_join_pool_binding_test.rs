//! Consumer of `fixtures/if_join_pool_binding`.
//!
//! `gunbc.recurring_failure_mode`
//! `binding_chosen_by_pool_membership_rather_than_by_the_declared_rule`:
//! a coproduct arm in `declaring_module` must typecheck the same whether or not
//! an unrelated same-spelled product is in the compiled closure.
//!
//! SCAFFOLD (DESIGN §7 HAND-RUST GATE — explicit deferral): the fixture is
//! deliberately outside `dag/` / `src/v2` so the collision is source handed to
//! the compiler, not a corpus fork, and the producer is
//! `compile_declared_import_closure_only_with_pool` over two entry files. That
//! shape is not a discovered `dag/test/claim/*_test.dag` row. This module is
//! compiled by clippy and executed by nobody — the standing drop
//! `gunbc.rung_drop` `rust_unit_tests_off_the_merge_path`. Lane: none enrolled;
//! the 2026-09-04 runner-capacity ruling closed the job roster to growth.
//! Sole dissolution: a required-lane producer that compiles both fixture
//! entries on the real acceptance path, then delete this Rust module.

use v1_compiler::cli_run::compile_declared_import_closure_only_with_pool;
use v1_compiler::v1_std_core::{diagnostic_to_message, is_error_diagnostic};

use crate::helpers::workspace_root;

const WITH_COLLISION: &str = "fixtures/if_join_pool_binding/closure_with_collision.dag";
const WITHOUT_COLLISION: &str = "fixtures/if_join_pool_binding/closure_without_collision.dag";

fn pool_roots() -> Vec<String> {
    vec![
        "dag/std".to_string(),
        "fixtures/if_join_pool_binding".to_string(),
    ]
}

fn hard_messages(
    compiled: &v1_compiler::v1_compiler_compile::ResolvedPipelineResult,
) -> Vec<String> {
    compiled
        .diagnostics
        .iter()
        .filter(|d| is_error_diagnostic(d.diagnostic.clone()))
        .map(|d| diagnostic_to_message(d.diagnostic.clone()))
        .collect()
}

#[test]
fn declaring_module_is_invariant_under_unrelated_pool_product() {
    std::env::set_current_dir(workspace_root()).expect("cwd");
    let without =
        compile_declared_import_closure_only_with_pool(&pool_roots(), WITHOUT_COLLISION, None)
            .expect("compile without collision");
    let with = compile_declared_import_closure_only_with_pool(&pool_roots(), WITH_COLLISION, None)
        .expect("compile with collision");
    let without_msgs = hard_messages(&without);
    let with_msgs = hard_messages(&with);
    assert!(
        without_msgs.is_empty(),
        "positive control must accept; got {without_msgs:?}"
    );
    assert!(
        with_msgs.is_empty(),
        "adding colliding_module to the closure must not rebind declaring_module's arm; got {with_msgs:?}"
    );
}
