use v1_compiled::v2_compiler_parse_engine_hooks as emitted;
use v1_compiler::v2_compiler_parse_engine_hooks as seed;

fn sym_eq(a: &str, b: &str) -> bool {
    a == b
}

fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let cases: [(&str, String, String); 3] = [
        (
            "match_arm_body",
            emitted::parse_engine_match_arm_body_production(),
            seed::parse_engine_match_arm_body_production(),
        ),
        (
            "match_arm_stmt_body",
            emitted::parse_engine_match_arm_stmt_body_production(),
            seed::parse_engine_match_arm_stmt_body_production(),
        ),
        (
            "expr",
            emitted::parse_engine_expr_production(),
            seed::parse_engine_expr_production(),
        ),
    ];
    let mut all_pass = true;
    for (i, (label, e, s)) in cases.iter().enumerate() {
        let e_cmp = if inject_fault && i == 0 {
            "INJECTED_FAULT".to_string()
        } else {
            e.clone()
        };
        let ok = sym_eq(&e_cmp, s);
        println!("{} emitted={} seed={} eq={}", label, e_cmp, s, ok);
        all_pass &= ok;
    }
    if all_pass {
        println!("SELF_HOST_PARSE_ENGINE_HOOKS_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_PARSE_ENGINE_HOOKS_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
