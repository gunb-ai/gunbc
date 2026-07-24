//! CLI-vs-floor compile-clean agreement (issue 11).
//!
//! Pins that both realizations read `gunbc.compile_clean_diagnostic_policy` and
//! agree on verdict for the whole-tree closure.

use std::rc::Rc;

use v1_compiler::cli_run::{
    compile_clean_cli_floor_verdicts_agree, compile_clean_diagnostic_is_hard,
    compile_clean_unlisted_import_use_blocks_from_policy,
};
use v1_compiler::std_types::SourceSpan;
use v1_compiler::v1_std_core::{CompilerDiagnostic, ErrorNode};

fn span() -> Rc<SourceSpan> {
    Rc::new(SourceSpan {
        file: "test.dag".to_string(),
        start: 0,
        end: 0,
    })
}

fn unlisted_import_use() -> Rc<ErrorNode> {
    Rc::new(ErrorNode {
        diagnostic: Rc::new(CompilerDiagnostic::UnlistedImportUse {
            name: "NonEmptyStr".to_string(),
            span: span(),
        }),
        module_name: "test.mod".to_string(),
    })
}

#[test]
fn policy_row_floor_not_yet_is_non_blocking() {
    let blocks =
        compile_clean_unlisted_import_use_blocks_from_policy().expect("policy row must resolve");
    assert!(
        !blocks,
        "FloorNotYet must not block UnlistedImportUse (issue 11 staging row)"
    );
    assert!(
        !compile_clean_diagnostic_is_hard(&unlisted_import_use()),
        "gate must delegate UnlistedImportUse hardness to the policy row"
    );
}

/// Heavyweight whole-tree receipt — run explicitly:
/// `cargo test -p v1-compiler-tests compile_clean_cli_floor_verdicts_agree_whole_tree -- --ignored --nocapture`
#[test]
#[ignore = "heavyweight whole-tree compile-clean; run explicitly"]
fn compile_clean_cli_floor_verdicts_agree_whole_tree() {
    let agree = compile_clean_cli_floor_verdicts_agree()
        .expect("whole-tree agreement check must not refuse");
    assert!(
        agree,
        "floor and CLI compile-clean verdicts must agree under FloorNotYet policy"
    );
}
