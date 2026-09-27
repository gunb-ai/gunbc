use v1_compiled::v2_compiler_parse_engine_hooks as emitted;

// NO SEED ORACLE, ON PURPOSE. This driver used to compare each hook against a hand copy in
// v1_compiler::v2_compiler_parse_engine_hooks. src/v2/compiler/parse_engine_hooks.dag declares no
// function: its whole behavior is three Symbol constants, and its closure reaches only v2.std.node,
// so the grammar they are meant to name is not in the emitted crate. This driver therefore runs,
// over the EMITTED realization, the assertions src/v2/test/claim/parse_engine_hooks_test.dag makes
// over the .dag in the interpreter (the roster's claim-run for this row), plus the refusal that
// gives those assertions a red: no hook collapses onto another hook's production.

fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let arm = emitted::parse_engine_match_arm_body_production();
    let stmt = emitted::parse_engine_match_arm_stmt_body_production();
    let expr = emitted::parse_engine_expr_production();
    let collapsed = arm == expr || arm == stmt || stmt == expr;
    // --inject-fault asserts ONLY the planted wrong acceptance (the #12275 shape): the match-arm
    // body hook names the expr production. A correct module reds it; a module whose hook collapsed
    // onto expr greens it, which the harness rejects.
    let all_pass = if inject_fault {
        println!("parse_engine_hooks injected: match_arm_body == expr is {}", arm == expr);
        arm == expr
    } else {
        let declared = arm == "dag_production_match_arm_body"
            && stmt == "dag_production_match_arm_stmt_body"
            && expr == "dag_production_expr";
        println!("parse_engine_hooks declared values={declared}");
        println!("parse_engine_hooks distinct productions={}", !collapsed);
        declared && !collapsed
    };

    if all_pass {
        println!("SELF_HOST_PARSE_ENGINE_HOOKS_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_PARSE_ENGINE_HOOKS_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
