//! **Layer:** integration

use v3_compiler::{
    compile_std_bootstrap_dag, generated_std_bootstrap_dag, serialize::first_difference,
};

#[test]
fn generated_std_bootstrap_snapshot_matches_runtime_std_bootstrap() {
    let runtime = compile_std_bootstrap_dag();
    let generated = generated_std_bootstrap_dag();
    assert!(
        first_difference(&runtime, &generated).is_none(),
        "generated std bootstrap drifted from runtime std bootstrap"
    );
}
