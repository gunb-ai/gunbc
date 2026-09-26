use im::vector as vec;
use std::rc::Rc;
use v1_compiled::v2_compiler_tokenize as emitted;
use v1_compiled::v2_std_compilers_lexing::{LexPattern, LexRule, LexRuleSet, LexRules, TokenStream};
use v1_compiled::v2_std_diagnostic::Outcome;

// NO SEED ORACLE, ON PURPOSE. This driver used to compare the prose constant
// tokenize_module_authority_note against a hand copy in v1_compiler::v2_compiler_tokenize, which
// said nothing about what tokenize does. Its expected verdicts are read off the authority instead:
// src/v2/compiler/01_tokenize.dag tokenize over ModeledLexRules walks the source with the rule set
// (lex_walk) -- a source every character of which some rule matches is accepted as that token
// stream, and a source with a character no rule matches is refused
// (tokenize_lex_e1_unrecognized_char).

// One token rule: the literal "a".
fn a_only_rules() -> Rc<LexRules> {
    Rc::new(LexRules::ModeledLexRules {
        root: Rc::new(LexRuleSet {
            rules: Rc::new(vec![Rc::new(LexRule::TokenRule {
                token_class: "a_token".to_string(),
                pattern: Rc::new(LexPattern::LiteralPattern {
                    text: Rc::new(vec!['a' as i64]),
                }),
            })]),
        }),
    })
}

fn tokenize(text: &str) -> Rc<Outcome<Rc<TokenStream>>> {
    emitted::tokenize(text.to_string(), "witness.dag".to_string(), a_only_rules())
}

// --inject-fault asserts ONLY the planted wrong acceptance (the #12275 shape): a source carrying a
// character no rule matches is accepted. A correct module reds it; a lexer that skips or absorbs
// the unmatched character greens it, which the harness rejects.
fn main() {
    let inject_fault = std::env::args().any(|a| a == "--inject-fault");
    let unmatched = tokenize("ab");
    let unmatched_accepted = matches!(&*unmatched, Outcome::Accepted { .. });
    let all_pass = if inject_fault {
        println!("tokenize injected: unmatched char accepted={unmatched_accepted}");
        unmatched_accepted
    } else {
        let matched_tokens = match &*tokenize("aa") {
            Outcome::Accepted { value, .. } => Some(value.all.len()),
            Outcome::Rejected { .. } => None,
        };
        let refusal = match &*unmatched {
            Outcome::Rejected { diagnostics } => format!("{diagnostics:?}"),
            Outcome::Accepted { .. } => String::new(),
        };
        let refused_as_unrecognized = refusal.contains("tokenize_lex_e1_unrecognized_char");
        println!("tokenize matched source tokens={matched_tokens:?}");
        println!("tokenize unmatched char refused as unrecognized={refused_as_unrecognized}");
        matched_tokens == Some(2) && refused_as_unrecognized
    };

    if all_pass {
        println!("SELF_HOST_01_TOKENIZE_BEHAVIORAL_RECEIPT: PASS");
        std::process::exit(0);
    }
    println!("SELF_HOST_01_TOKENIZE_BEHAVIORAL_RECEIPT: FAIL");
    std::process::exit(1);
}
