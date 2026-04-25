//! **Layer:** integration
//!
//! PB-1-e: the runtime fresh-parse vs `bootstrap_std_generated.rs` drift harness
//! retired in favor of `regen_bootstrap --verify`. These tests pin cheap
//! structural facts about the committed std snapshot.

use v3_compiler::generated_std_bootstrap_dag;

#[test]
fn generated_std_bootstrap_snapshot_is_clean_and_substantive() {
    let dag = generated_std_bootstrap_dag();
    assert!(
        dag.diagnostics().is_empty(),
        "expected clean std snapshot bootstrap, got {:?}",
        dag.diagnostics()
    );
    assert!(
        dag.declaration_by_name("Bool").is_some(),
        "std snapshot should include kernel Bool"
    );
}
