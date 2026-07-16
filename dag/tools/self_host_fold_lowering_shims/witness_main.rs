use v1_compiled::v2_compiler_fold_lowering as emitted;
use v1_compiler::v2_compiler_fold_lowering as seed;

fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let cases = [
        "fold",
        "fold_list",
        "fold_list_right",
        "fold_node",
        "map",
        "g",
    ];
    let mut all_pass = true;
    for (i, sym) in cases.iter().enumerate() {
        let e = if inject_fault && i == 0 {
            !emitted::fold_family_head(sym.to_string())
        } else {
            emitted::fold_family_head(sym.to_string())
        };
        let s = seed::fold_family_head(sym.to_string());
        let ok = e == s;
        println!("fold_family_head({sym}) emitted={e} seed={s} eq={ok}");
        all_pass &= ok;
    }
    if all_pass {
        println!("SELF_HOST_FOLD_LOWERING_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_FOLD_LOWERING_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
