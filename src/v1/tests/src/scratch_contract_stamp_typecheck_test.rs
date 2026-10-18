use v1_compiler::v1_compiler_artifact::RenderTarget;

use crate::helpers::{
    compile_dag_named_with_source_roots, diagnostic_messages, read_v2_file, v2_layer_roots,
};

// SCRATCH (royal-badger-840): type-checks the contract-stamp authority and its witness against the
// real corpus. Delete before merge; the permanent coverage is the .dag witness itself.
fn compile_repo_module(path: &str) -> String {
    let source = read_v2_file(path);
    let result =
        compile_dag_named_with_source_roots(path, &source, RenderTarget::Rust, &v2_layer_roots());
    diagnostic_messages(&result).join("\n")
}

#[test]
fn scratch_contract_stamp_authority_typechecks() {
    println!(
        "=== STAMP AUTHORITY ===\n{}\n=== END ===",
        compile_repo_module("dag/gunbc/witness/compiler_gate_contract_stamp.dag")
    );
}

#[test]
fn scratch_contract_stamp_witness_typechecks() {
    println!(
        "=== STAMP WITNESS ===\n{}\n=== END ===",
        compile_repo_module("dag/test/claim/witness/compiler_gate_contract_stamp_witness.dag")
    );
}

#[test]
fn scratch_compiler_gate_workflow_typechecks() {
    println!(
        "=== WORKFLOW GENERATOR ===\n{}\n=== END ===",
        compile_repo_module("dag/gunbc/witness/compiler_gate_workflow.dag")
    );
}
