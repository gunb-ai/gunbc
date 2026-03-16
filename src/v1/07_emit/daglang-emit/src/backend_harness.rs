use crate::lower_c::{lower_to_c, CConfig};
use crate::lower_go::{lower_to_go, GoConfig};
use crate::lower_mips::{lower_to_mips, MipsConfig};
use crate::render_c::render_c_source;
use crate::render_go::render_go_source;
use crate::render_mips::render_mips_source;
use gunbc_ir::code_ir::{
    CallObligation, Expr, FnDef, Item, SourceFile, Stmt,
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

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

#[test]
fn adversarial_fixture_enforces_cross_backend_invariants() {
    let source = adversarial_transport_source();

    // Go output invariants.
    let go_cfg = GoConfig {
        package_name: "harness".to_string(),
        ..GoConfig::default()
    };
    let go_lowered = lower_to_go(&source, &go_cfg).expect("go lowering should succeed");
    let rendered_go = render_go_source(&go_lowered);
    let scoped_err_blocks = count_occurrences(
        &rendered_go,
        "\t {\n\t\t_, err := transport.Execute(request)\n",
    );
    assert_eq!(
        scoped_err_blocks, 2,
        "repeated transport expression statements should render as isolated Go block scopes"
    );
    assert_eq!(
        count_occurrences(&rendered_go, "return err"),
        3,
        "each Go transport call should preserve its explicit error return"
    );
    assert!(
        rendered_go.contains("return nil"),
        "fallible Go main should render an explicit success return"
    );

    // C output invariants.
    let c_lowered = lower_to_c(&source, &CConfig::default()).expect("c lowering should succeed");
    let rendered_c = render_c_source(&c_lowered);
    let rc_scope_count = count_occurrences(
        &rendered_c,
        "    {\n        int __rc = gunbc_transport_execute(request);\n",
    );
    assert_eq!(
        rc_scope_count, 2,
        "repeated transport expression statements should render as isolated C block scopes"
    );
    assert_eq!(
        count_occurrences(&rendered_c, "return -1;"),
        3,
        "each C transport call should preserve its explicit error return"
    );
    assert!(
        rendered_c.contains("return 0;"),
        "fallible C main should render an explicit success return"
    );

    // MIPS output invariants.
    let mips =
        lower_to_mips(&c_lowered, &MipsConfig::default()).expect("mips lowering should succeed");
    let rendered_mips = render_mips_source(&mips);
    assert!(
        rendered_mips.contains("main_epilogue:"),
        "rendered assembly should include explicit epilogue label"
    );
    let (before_epilogue, after_epilogue) = rendered_mips
        .split_once("main_epilogue:")
        .expect("epilogue label should be present");
    assert!(
        !before_epilogue.contains("\tjr $ra"),
        "assembly before epilogue label should not contain direct jr $ra"
    );
    assert!(
        before_epilogue.contains("\tj main_epilogue\n"),
        "assembly should route return paths through the explicit epilogue label"
    );
    assert!(
        after_epilogue.contains("\tjr $ra\n"),
        "epilogue should contain the only direct return instruction"
    );
}
