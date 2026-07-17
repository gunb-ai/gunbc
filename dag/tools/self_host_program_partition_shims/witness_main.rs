use v1_compiled::v2_compiler_program_partition as emitted;
use v1_compiler::v2_compiler_program_partition as seed;

fn sym_eq(a: &str, b: &str) -> bool {
    a == b
}

fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let cases: [(&str, String, String); 1] = [(
        "partition_structural_value_carriers_dissolution_trigger",
        emitted::partition_structural_value_carriers_dissolution_trigger(),
        seed::partition_structural_value_carriers_dissolution_trigger(),
    )];
    let mut all_pass = true;
    for (i, (label, e, s)) in cases.iter().enumerate() {
        let e_cmp = if inject_fault && i == 0 {
            "INJECTED_FAULT".to_string()
        } else {
            e.clone()
        };
        let ok = sym_eq(&e_cmp, s);
        println!("{label} emitted_len={} seed_len={} eq={ok}", e_cmp.len(), s.len());
        all_pass &= ok;
    }
    if all_pass {
        println!("SELF_HOST_PROGRAM_PARTITION_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_PROGRAM_PARTITION_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
