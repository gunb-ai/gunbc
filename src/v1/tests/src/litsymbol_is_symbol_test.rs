use std::rc::Rc;
use v1_compiler::std_syntax::LiteralValue;
use v1_compiler::v1_compiler_infer_types::infer_literal_node;

#[test]
fn infer_literal_node_types_litsymbol_as_symbol() {
    let n = infer_literal_node(Rc::new(LiteralValue::LitSymbol {
        value: "tag".to_string(),
    }));
    assert_eq!(
        n.name.as_str(),
        "Symbol",
        "RED: infer_literal_node(LitSymbol) must mint symbol_type, not string_type"
    );
}

#[test]
fn infer_literal_node_types_litstr_as_string() {
    let n = infer_literal_node(Rc::new(LiteralValue::LitStr {
        value: "tag".to_string(),
    }));
    assert_eq!(n.name.as_str(), "String");
}
