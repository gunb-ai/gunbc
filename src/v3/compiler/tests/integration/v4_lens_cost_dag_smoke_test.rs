//! **Layer:** integration
//!
//! Ratchet P9's LLVM instruction cost-table move to the cost-lens authority.

#[test]
fn v4_lens_cost_owns_llvm_instruction_cost_table() {
    let cost_lens = include_str!("../../../../v4/lens/cost.dag");
    let llvm_ir = include_str!("../../../../v4/extdeps/languages/llvm_ir.dag");

    assert!(
        cost_lens.contains("fn llvm_instruction_cost(i: LlvmInstruction) -> Int"),
        "P9 requires the LLVM instruction cost table to live under v4/lens/cost.dag"
    );
    assert!(
        !llvm_ir.contains("fn llvm_instruction_cost"),
        "llvm_ir.dag must not keep a second llvm_instruction_cost authority"
    );
    assert!(
        !llvm_ir.lines().any(|line| {
            line.starts_with("// Owns:") && line.contains("llvm_instruction_cost")
        }),
        "llvm_ir.dag Owns header must not list llvm_instruction_cost"
    );
}
