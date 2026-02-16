use daglang_syntax::lexer::{Lexer, TokenKind};

fn kinds(src: &str) -> Vec<TokenKind> {
    Lexer::tokenize(src).into_iter().map(|token| token.kind).collect()
}

#[test]
fn interpolation_handles_nested_braces_in_expression() {
    assert_eq!(
        kinds(r#""x {foo({bar: 1})} y""#),
        vec![
            TokenKind::StrBegin("x ".into()),
            TokenKind::Ident("foo".into()),
            TokenKind::LParen,
            TokenKind::LBrace,
            TokenKind::Ident("bar".into()),
            TokenKind::Colon,
            TokenKind::Int(1),
            TokenKind::RBrace,
            TokenKind::RParen,
            TokenKind::StrEnd(" y".into()),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn interpolation_does_not_start_on_escaped_brace() {
    assert_eq!(
        kinds(r#""literal \{ brace and {name}""#),
        vec![
            TokenKind::StrBegin("literal { brace and ".into()),
            TokenKind::Ident("name".into()),
            TokenKind::StrEnd(String::new()),
            TokenKind::Eof,
        ]
    );
}
