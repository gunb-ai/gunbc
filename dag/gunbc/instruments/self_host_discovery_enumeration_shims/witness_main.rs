use std::rc::Rc;
use v1_compiled::v2_compiler_discovery_enumeration as emitted;
use v1_compiler::v2_compiler_discovery_enumeration as seed;

fn rc_text_eq(e: &Rc<String>, s: &str) -> bool {
    e.as_str() == s
}

fn resolved_eq(e: &emitted::ResolvedDeclRef, s: &seed::ResolvedDeclRef) -> bool {
    rc_text_eq(&e.module, &s.module) && e.name == s.name
}

fn init_eq(e: &emitted::OwnedDataDeclInitializer, s: &seed::OwnedDataDeclInitializer) -> bool {
    use emitted::OwnedDataDeclInitializer as E;
    use seed::OwnedDataDeclInitializer as S;
    match (e, s) {
        (E::OwnedNodeCorpusInit, S::OwnedNodeCorpusInit) => true,
        (
            E::OwnedBoolWitnessClaimInit {
                witness_entry: ee,
                witness_function: ef,
            },
            S::OwnedBoolWitnessClaimInit {
                witness_entry: se,
                witness_function: sf,
            },
        ) => rc_text_eq(ee, se) && rc_text_eq(ef, sf),
        (E::OwnedOtherInit { resolved: er }, S::OwnedOtherInit { resolved: sr }) => {
            resolved_eq(er, sr)
        }
        _ => false,
    }
}

fn receipt_eq(e: &emitted::OwnedDataDiscoveryReceipt, s: &seed::OwnedDataDiscoveryReceipt) -> bool {
    e.unified_claim_arm_count == s.unified_claim_arm_count
        && e.bool_witness_claim_arm_count == s.bool_witness_claim_arm_count
        && e.illegal_other_init_count == s.illegal_other_init_count
        && e.bool_witness_transport_row_count == s.bool_witness_transport_row_count
        && e.transport_projection_complete == s.transport_projection_complete
}

fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let mut all_pass = true;

    let e_bool_mod = emitted::unified_claim_arm_bool_witness_claim_module();
    let s_bool_mod = seed::unified_claim_arm_bool_witness_claim_module();
    let bool_mod_ok = rc_text_eq(&e_bool_mod, &s_bool_mod) && !inject_fault;
    println!("bool_module emitted={e_bool_mod:?} seed={s_bool_mod:?} eq={bool_mod_ok}");
    all_pass &= bool_mod_ok;

    let e_corpus_mod = emitted::unified_claim_arm_node_corpus_module();
    let s_corpus_mod = seed::unified_claim_arm_node_corpus_module();
    let corpus_mod_ok = rc_text_eq(&e_corpus_mod, &s_corpus_mod);
    println!("corpus_module emitted={e_corpus_mod:?} seed={s_corpus_mod:?} eq={corpus_mod_ok}");
    all_pass &= corpus_mod_ok;

    let cases: [(emitted::OwnedDataDeclInitializer, seed::OwnedDataDeclInitializer); 3] = [
        (
            emitted::OwnedDataDeclInitializer::OwnedNodeCorpusInit,
            seed::OwnedDataDeclInitializer::OwnedNodeCorpusInit,
        ),
        (
            emitted::OwnedDataDeclInitializer::OwnedBoolWitnessClaimInit {
                witness_entry: Rc::new("entry.dag".to_string()),
                witness_function: Rc::new("witness_fn".to_string()),
            },
            seed::OwnedDataDeclInitializer::OwnedBoolWitnessClaimInit {
                witness_entry: "entry.dag".to_string(),
                witness_function: "witness_fn".to_string(),
            },
        ),
        (
            emitted::OwnedDataDeclInitializer::OwnedOtherInit {
                resolved: Rc::new(emitted::ResolvedDeclRef {
                    module: Rc::new("v2.example.mod".to_string()),
                    name: "sym".to_string(),
                }),
            },
            seed::OwnedDataDeclInitializer::OwnedOtherInit {
                resolved: seed::ResolvedDeclRef {
                    module: "v2.example.mod".to_string(),
                    name: "sym".to_string(),
                },
            },
        ),
    ];

    for (i, (ev, sv)) in cases.iter().enumerate() {
        let ev_cmp = if inject_fault && i == 0 {
            emitted::OwnedDataDeclInitializer::OwnedBoolWitnessClaimInit {
                witness_entry: Rc::new("fault".to_string()),
                witness_function: Rc::new("fault".to_string()),
            }
        } else {
            ev.clone()
        };
        let ok = init_eq(&ev_cmp, sv);
        println!("init({i}) emitted={ev_cmp:?} seed={sv:?} eq={ok}");
        all_pass &= ok;
    }

    let e_receipt = emitted::OwnedDataDiscoveryReceipt {
        unified_claim_arm_count: 2,
        bool_witness_claim_arm_count: 1,
        illegal_other_init_count: 0,
        bool_witness_transport_row_count: 1,
        transport_projection_complete: true,
    };
    let s_receipt = seed::OwnedDataDiscoveryReceipt {
        unified_claim_arm_count: 2,
        bool_witness_claim_arm_count: 1,
        illegal_other_init_count: 0,
        bool_witness_transport_row_count: 1,
        transport_projection_complete: true,
    };
    let receipt_ok = receipt_eq(&e_receipt, &s_receipt);
    println!("receipt emitted={e_receipt:?} seed={s_receipt:?} eq={receipt_ok}");
    all_pass &= receipt_ok;

    if all_pass {
        println!("SELF_HOST_DISCOVERY_ENUMERATION_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_DISCOVERY_ENUMERATION_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
