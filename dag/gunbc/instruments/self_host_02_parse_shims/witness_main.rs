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

// The planted fault feeds the invalid grammar where the modeled one is expected, so the fault run
// goes red only if emitted prepare_grammar really refuses it.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let valid_input = if inject_fault {
        grammar_with_undefined_nonterminal()
    } else {
        dag_grammar()
    };
    let valid_accepts = prepares(valid_input);
    println!("prepare_grammar dag_grammar inject_fault={inject_fault} accepts={valid_accepts}");
    let invalid_refuses = !prepares(grammar_with_undefined_nonterminal());
    println!("prepare_grammar undefined_nonterminal refuses={invalid_refuses}");
    if valid_accepts && invalid_refuses {
        println!("SELF_HOST_02_PARSE_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_02_PARSE_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
