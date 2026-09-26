use std::rc::Rc;

use v1_compiled::v2_compiler_discovery_enumeration as emitted;
use v1_compiled::v2_compiler_discovery_enumeration::{OwnedDataDeclInitializer, ResolvedDeclRef};

// NO SEED ORACLE, ON PURPOSE. This driver used to compare emitted values against
// v1_compiler::v2_compiler_discovery_enumeration, a hand copy, and it had gone stale against the
// emitter (it built the String fields as Rc<String>, so it failed to compile). src/v2/compiler/
// discovery_enumeration.dag declares no function: its whole behavior is two data constants and the
// construction and elimination of its types. So this driver runs, over the EMITTED realization,
// the same assertions src/v2/test/claim/discovery_enumeration_test.dag makes over the .dag in the
// interpreter (the roster's claim-run for this row) -- one contract, two realizations, no copy of
// the module under test.

fn classify(init: &OwnedDataDeclInitializer) -> &'static str {
    match init {
        OwnedDataDeclInitializer::OwnedBoolWitnessClaimInit { .. } => "bool_witness",
        OwnedDataDeclInitializer::OwnedNodeCorpusInit => "node_corpus",
        OwnedDataDeclInitializer::OwnedOtherInit { .. } => "other",
    }
}

// The planted fault expects the node-corpus arm module to differ from the .dag's declared value,
// so the fault run goes red only if the emitted constant really carries that value.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let expected_module = "v2.std.verification";
    let corpus_expected = if inject_fault { "v2.std.fault" } else { expected_module };
    let modules_ok = emitted::unified_claim_arm_bool_witness_claim_module() == expected_module
        && emitted::unified_claim_arm_node_corpus_module() == corpus_expected;
    println!("claim arm modules inject_fault={inject_fault} ok={modules_ok}");

    let corpus = OwnedDataDeclInitializer::OwnedNodeCorpusInit;
    let bool_witness = OwnedDataDeclInitializer::OwnedBoolWitnessClaimInit {
        witness_entry: "entry.dag".to_string(),
        witness_function: "witness_fn".to_string(),
    };
    let other = OwnedDataDeclInitializer::OwnedOtherInit {
        resolved: Rc::new(ResolvedDeclRef {
            module: "v2.example".to_string(),
            name: "sym".to_string(),
        }),
    };
    let bool_fields_ok = matches!(&bool_witness,
        OwnedDataDeclInitializer::OwnedBoolWitnessClaimInit { witness_entry, witness_function }
            if witness_entry == "entry.dag" && witness_function == "witness_fn");
    let other_fields_ok = matches!(&other,
        OwnedDataDeclInitializer::OwnedOtherInit { resolved }
            if resolved.module == "v2.example" && resolved.name == "sym");
    let arms_ok = classify(&corpus) == "node_corpus"
        && classify(&bool_witness) == "bool_witness"
        && classify(&other) == "other"
        && bool_fields_ok
        && other_fields_ok;
    println!("initializer arms round-trip ok={arms_ok}");

    if modules_ok && arms_ok {
        println!("SELF_HOST_DISCOVERY_ENUMERATION_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_DISCOVERY_ENUMERATION_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
