// Minimal tokenizer for M0 test subset.
//
// Recognizes: `let`, `if`, `then`, `else`, identifiers, integer
// literals, `=`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `+`, whitespace
// (skipped), EOF. Everything else is a tokenizer error routed
// through the Diagnostic path — no panics.

use crate::diagnostics::{Diagnostic, SourceSpan};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    KwLet,
    KwIf,
    KwThen,
    KwElse,
    Ident(String),
    IntLit(i64),
    Eq,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: SourceSpan,
}

pub fn tokenize(source: &str, file: &str) -> Result<Vec<Token>, Diagnostic> {
    let bytes = source.as_bytes();
    let mut pos: usize = 0;
    let mut tokens = Vec::new();

    while pos < bytes.len() {
        let byte = bytes[pos];

        if byte.is_ascii_whitespace() {
            pos += 1;
            continue;
        }

        let start = pos;

        if let Some((kind, width)) = punctuation_token(bytes, pos) {
            tokens.push(Token {
                kind,
                span: SourceSpan::new(file, start as u32, (start + width) as u32),
            });
            pos += width;
            continue;
        }

        if byte.is_ascii_digit() {
            let mut end = pos;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            let literal = &source[start..end];
            let value: i64 = literal.parse().map_err(|_| Diagnostic::TokenizerError {
                message: format!("invalid integer literal `{literal}`"),
                span: SourceSpan::new(file, start as u32, end as u32),
            })?;
            tokens.push(Token {
                kind: TokenKind::IntLit(value),
                span: SourceSpan::new(file, start as u32, end as u32),
            });
            pos = end;
            continue;
        }

        if byte.is_ascii_alphabetic() || byte == b'_' {
            let mut end = pos;
            while end < bytes.len()
                && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_')
            {
                end += 1;
            }
            let text = &source[start..end];
            let kind = match text {
                "let" => TokenKind::KwLet,
                "if" => TokenKind::KwIf,
                "then" => TokenKind::KwThen,
                "else" => TokenKind::KwElse,
                _ => TokenKind::Ident(text.to_string()),
            };
            tokens.push(Token {
                kind,
                span: SourceSpan::new(file, start as u32, end as u32),
            });
            pos = end;
            continue;
        }

        return Err(Diagnostic::TokenizerError {
            message: format!("unexpected byte `{}`", byte as char),
            span: SourceSpan::new(file, start as u32, (start + 1) as u32),
        });
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        span: SourceSpan::new(file, bytes.len() as u32, bytes.len() as u32),
    });
    Ok(tokens)
}

fn punctuation_token(bytes: &[u8], pos: usize) -> Option<(TokenKind, usize)> {
    let first = bytes[pos];
    let second = bytes.get(pos + 1).copied();
    match (first, second) {
        (b'=', Some(b'=')) => Some((TokenKind::EqEq, 2)),
        (b'!', Some(b'=')) => Some((TokenKind::NotEq, 2)),
        (b'<', Some(b'=')) => Some((TokenKind::Le, 2)),
        (b'>', Some(b'=')) => Some((TokenKind::Ge, 2)),
        (b'=', _) => Some((TokenKind::Eq, 1)),
        (b'<', _) => Some((TokenKind::Lt, 1)),
        (b'>', _) => Some((TokenKind::Gt, 1)),
        (b'+', _) => Some((TokenKind::Plus, 1)),
        _ => None,
    }
}
