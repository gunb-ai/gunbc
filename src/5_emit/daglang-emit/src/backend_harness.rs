use crate::lower_c::{lower_to_c, CConfig};
use crate::lower_go::{lower_to_go, GoConfig};
use crate::lower_mips::{lower_to_mips, MipsConfig};
use crate::render_mips::render_mips_source;
use gunbc_ir::code_ir::c_ir::{CItem, CStmt};
use gunbc_ir::code_ir::register_ir::{Instruction, Register};
use gunbc_ir::code_ir::{
    BindIntent, BindTarget, CallObligation, Expr, FnDef, Item, SourceFile, Stmt,
};

fn adversarial_transport_source() -> SourceFile {
    SourceFile {
        doc: vec!["Backend adversarial transport fixture".to_string()],
        items: vec![Item::Fn(FnDef {
            name: "main".to_string(),
            is_pub: true,
            params: vec![("path".to_string(), "String".to_string())],
            return_type: None,
            body: vec![
                Stmt::let_bind(
                    "request",
                    Expr::call_with_obligation(
                        "prepare_file_read",
                        vec![Expr::var("path")],
                        CallObligation::ServiceTransportPrepare,
                    ),
                ),
                Stmt::Expr(Expr::call_with_obligation(
                    "execute_file_read",
                    vec![Expr::var("request")],
                    CallObligation::ServiceTransportExecute,
                )),
                Stmt::Expr(Expr::call_with_obligation(
                    "execute_file_read",
                    vec![Expr::var("request")],
                    CallObligation::ServiceTransportExecute,
                )),
            ],
            doc: vec![],
            attributes: vec![],
        })],
    }
}

#[test]
fn adversarial_fixture_enforces_cross_backend_invariants() {
    let source = adversarial_transport_source();

    // Go structural invariants.
    let go_cfg = GoConfig {
        package_name: "harness".to_string(),
        ..GoConfig::default()
    };
    let go_lowered = lower_to_go(&source, &go_cfg).expect("go lowering should succeed");
    let go_main = go_lowered
        .items
        .iter()
        .find_map(|item| match item {
            Item::Fn(f) if f.name == "Main" => Some(f),
            _ => None,
        })
        .expect("lowered Go should contain Main");
    let scoped_err_blocks = go_main
        .body
        .iter()
        .filter(|stmt| {
            matches!(
                stmt,
                Stmt::BlockScope(inner)
                    if matches!(
                        inner.first(),
                        Some(Stmt::Bind {
                            targets,
                            intent: BindIntent::Declare,
                            ..
                        }) if matches!(
                            targets.as_slice(),
                            [BindTarget::Discard, BindTarget::Name(err)] if err == "err"
                        )
                    )
            )
        })
        .count();
    assert_eq!(
        scoped_err_blocks, 2,
        "repeated transport expression statements should isolate err in block scope"
    );

    // C structural invariants.
    let c_lowered = lower_to_c(&source, &CConfig::default()).expect("c lowering should succeed");
    let c_main = c_lowered
        .items
        .iter()
        .find_map(|item| match item {
            CItem::FnDef(f) if f.name == "main" => Some(f),
            _ => None,
        })
        .expect("lowered C should contain main");
    let rc_scope_count = c_main
        .body
        .iter()
        .filter(|stmt| {
            matches!(
                stmt,
                CStmt::BlockScope(inner)
                    if matches!(
                        inner.first(),
                        Some(CStmt::Decl { name, .. }) if name == "__rc"
                    )
            )
        })
        .count();
    assert_eq!(
        rc_scope_count, 2,
        "repeated transport expression statements should isolate __rc in C block scope"
    );

    // MIPS structural invariants.
    let mips =
        lower_to_mips(&c_lowered, &MipsConfig::default()).expect("mips lowering should succeed");
    let main_fn = mips
        .functions
        .iter()
        .find(|f| f.label == "main")
        .expect("mips program should contain main");
    let jump_epilogue_count = main_fn
        .body
        .iter()
        .filter(|inst| matches!(inst, Instruction::JumpEpilogue))
        .count();
    assert!(
        jump_epilogue_count >= 3,
        "main should route all returns through epilogue (expected >=3 jumps, got {jump_epilogue_count})"
    );
    assert!(
        !main_fn
            .body
            .iter()
            .any(|inst| matches!(inst, Instruction::JumpReg(Register::Ra))),
        "lowered main body should not contain direct jr $ra"
    );

    let rendered_mips = render_mips_source(&mips);
    assert!(
        rendered_mips.contains("main_epilogue:"),
        "rendered assembly should include explicit epilogue label"
    );
    let (before_epilogue, _) = rendered_mips
        .split_once("main_epilogue:")
        .expect("epilogue label should be present");
    assert!(
        !before_epilogue.contains("\tjr $ra"),
        "assembly before epilogue label should not contain direct jr $ra"
    );
}
