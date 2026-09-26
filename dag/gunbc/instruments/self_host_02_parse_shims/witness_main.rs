use std::rc::Rc;

use v1_compiled::v2_compiler_parse as emitted;
use v1_compiled::v2_extdeps_languages_dag::dag_grammar;
use v1_compiled::v2_std_diagnostic::Outcome;
use v1_compiled::v2_std_grammar::{GrammarExpr, GrammarProduction, GrammarRoot, ParseGrammar};

// NO SEED ORACLE, ON PURPOSE. This driver used to compare the prose constant
// prepared_grammar_carrier_note against v1_compiler::v2_compiler_parse, a hand copy that no longer
// exists; string parity with a copy of the module under test said nothing about what parse does.
// Its expected verdicts are read off the authority instead: src/v2/compiler/02_parse.dag
// prepare_grammar admits the modeled .dag grammar and refuses a grammar naming an undefined
// nonterminal (grammar_validate_and_analyze, parse_grammar_undefined_nonterminal).

fn prepares(grammar: Rc<ParseGrammar>) -> bool {
    matches!(
        &*emitted::prepare_grammar(grammar),
        Outcome::Accepted { .. }
    )
}

// The real .dag grammar plus one production whose body names a nonterminal no production defines.
fn grammar_with_undefined_nonterminal() -> Rc<ParseGrammar> {
    let root = dag_grammar().root();
    let donor = root.productions.head().expect("dag grammar has productions").clone();
    let mut productions = (*root.productions).clone();
    productions.push_back(Rc::new(GrammarProduction {
        name: "self_host_witness_dangling".to_string(),
        expression: Rc::new(GrammarExpr::Nonterminal {
            production: "self_host_witness_undefined".to_string(),
        }),
        emitted: donor.emitted.clone(),
    }));
    Rc::new(ParseGrammar::ModeledGrammar {
        root: Rc::new(GrammarRoot {
            start: root.start.clone(),
            productions: Rc::new(productions),
            sync_tokens: root.sync_tokens.clone(),
        }),
    })
}

// --inject-fault asserts ONLY the planted wrong acceptance (the #12275 shape): the injected run is
// PASS exactly when the emitted module accepts what the .dag refuses, so a correct module reds it
// and a module that wrongly accepts greens it -- which the harness then rejects.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let invalid_accepted = prepares(grammar_with_undefined_nonterminal());
    let all_pass = if inject_fault {
        println!("prepare_grammar injected: undefined_nonterminal accepted={invalid_accepted}");
        invalid_accepted
    } else {
        let valid_accepts = prepares(dag_grammar());
        println!("prepare_grammar dag_grammar accepts={valid_accepts}");
        println!("prepare_grammar undefined_nonterminal refuses={}", !invalid_accepted);
        valid_accepts && !invalid_accepted
    };

    if all_pass {
        println!("SELF_HOST_02_PARSE_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_02_PARSE_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
