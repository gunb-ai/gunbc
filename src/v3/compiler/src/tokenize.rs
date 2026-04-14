// Tokenizer for the v3 surface grammar.
//
// Keywords: `let`, `if`, `then`, `else`, `fn`, `type`, `module`, `import`,
// `match`, `data`, `where`, `true`, `false`. Identifiers, integer literals,
// double-quoted string literals, and punctuation (`=`, `==`, `!=`, `<`, `<=`,
// `>`, `>=`, `+`, `-`, `*`, `/`, `:`, `->`, `=>`, `.`, `(`, `)`, `{`, `}`,
// `[`, `]`, `,`, `;`, `|`, `?`). Whitespace and `//` line comments are
// skipped. Tokenizer errors flow through the Diagnostic path — no panics.
//
// `<`/`>` tokenize as comparison operators; the parser disambiguates them as
// type-parameter delimiters by context (M1_DESIGN.md §8.8).

use crate::diagnostics::{Diagnostic, SourceSpan};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    KwLet,
    KwIf,
    KwThen,
    KwElse,
    KwFn,
    KwType,
    KwModule,
    KwImport,
    KwMatch,
    KwData,
    KwWhere,
    KwTrue,
    KwFalse,
    Ident(String),
    IntLit(i64),
    StringLit(String),
    Eq,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    Plus,
    Minus,
    Star,
    Slash,
    Colon,
    Arrow,
    FatArrow,
    Dot,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Semicolon,
    Pipe,
    Question,
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

        // Line comments: `// ...` to end of line.
        if byte == b'/' && bytes.get(pos + 1) == Some(&b'/') {
            pos += 2;
            while pos < bytes.len() && bytes[pos] != b'\n' {
                pos += 1;
            }
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
                "fn" => TokenKind::KwFn,
                "type" => TokenKind::KwType,
                "module" => TokenKind::KwModule,
                "import" => TokenKind::KwImport,
                "match" => TokenKind::KwMatch,
                "data" => TokenKind::KwData,
                "where" => TokenKind::KwWhere,
                "true" => TokenKind::KwTrue,
                "false" => TokenKind::KwFalse,
                _ => TokenKind::Ident(text.to_string()),
            };
            tokens.push(Token {
                kind,
                span: SourceSpan::new(file, start as u32, end as u32),
            });
            pos = end;
            continue;
        }

        // String literal: "..."
        //
        // M0: no escape sequences. A `\` inside the string is a
        // literal backslash. The string terminates at the next `"`
        // or fails with TokenizerError if the source ends first.
        if byte == b'"' {
            let content_start = pos + 1;
            let mut end = content_start;
            while end < bytes.len() && bytes[end] != b'"' {
                end += 1;
            }
            if end >= bytes.len() {
                return Err(Diagnostic::TokenizerError {
                    message: "unterminated string literal".to_string(),
                    span: SourceSpan::new(file, start as u32, end as u32),
                });
            }
            let content = source[content_start..end].to_string();
            let close = end + 1;
            tokens.push(Token {
                kind: TokenKind::StringLit(content),
                span: SourceSpan::new(file, start as u32, close as u32),
            });
            pos = close;
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
        (b'=', Some(b'>')) => Some((TokenKind::FatArrow, 2)),
        (b'!', Some(b'=')) => Some((TokenKind::NotEq, 2)),
        (b'<', Some(b'=')) => Some((TokenKind::Le, 2)),
        (b'>', Some(b'=')) => Some((TokenKind::Ge, 2)),
        (b'-', Some(b'>')) => Some((TokenKind::Arrow, 2)),
        (b'=', _) => Some((TokenKind::Eq, 1)),
        (b'<', _) => Some((TokenKind::Lt, 1)),
        (b'>', _) => Some((TokenKind::Gt, 1)),
        (b'+', _) => Some((TokenKind::Plus, 1)),
        (b'-', _) => Some((TokenKind::Minus, 1)),
        (b'*', _) => Some((TokenKind::Star, 1)),
        (b'/', _) => Some((TokenKind::Slash, 1)),
        (b':', _) => Some((TokenKind::Colon, 1)),
        (b'.', _) => Some((TokenKind::Dot, 1)),
        (b'(', _) => Some((TokenKind::LParen, 1)),
        (b')', _) => Some((TokenKind::RParen, 1)),
        (b'{', _) => Some((TokenKind::LBrace, 1)),
        (b'}', _) => Some((TokenKind::RBrace, 1)),
        (b'[', _) => Some((TokenKind::LBracket, 1)),
        (b']', _) => Some((TokenKind::RBracket, 1)),
        (b',', _) => Some((TokenKind::Comma, 1)),
        (b';', _) => Some((TokenKind::Semicolon, 1)),
        (b'|', _) => Some((TokenKind::Pipe, 1)),
        (b'?', _) => Some((TokenKind::Question, 1)),
        _ => None,
    }
}
