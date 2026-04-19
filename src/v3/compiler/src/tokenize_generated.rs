// AUTO-GENERATED from `src/v3/compiler/tokenize.dag` via
// `regen_tokenize`. Regenerate instead of hand-editing.

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
    AmpAmp,
    PipePipe,
    Colon,
    Arrow,
    FatArrow,
    PipeArrow,
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
                message: format!("invalid integer literal `{}`", literal),
                span: SourceSpan::new(file, start as u32, end as u32),
                fixes: Vec::new(),
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
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_') {
                end += 1;
            }
            let text = &source[start..end];
            let kind = match text {
                "data" => TokenKind::KwData,
                "else" => TokenKind::KwElse,
                "false" => TokenKind::KwFalse,
                "fn" => TokenKind::KwFn,
                "if" => TokenKind::KwIf,
                "import" => TokenKind::KwImport,
                "let" => TokenKind::KwLet,
                "match" => TokenKind::KwMatch,
                "module" => TokenKind::KwModule,
                "then" => TokenKind::KwThen,
                "true" => TokenKind::KwTrue,
                "type" => TokenKind::KwType,
                "where" => TokenKind::KwWhere,
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
        // Minimal escape surface for bootstrap-staged structural data:
        // `\"`, `\\`, `\n`, `\r`, `\t`. Unknown `\x` pairs preserve the
        // old M0 behavior and stay literal as `\` + `x`. Raw newlines
        // are preserved until the closing `"`.
        if byte == b'"' {
            let mut end = pos + 1;
            let mut content = String::new();
            let mut terminated = false;
            while end < bytes.len() {
                match bytes[end] {
                    b'"' => {
                        terminated = true;
                        end += 1;
                        break;
                    }
                    b'\\' => {
                        let Some(escaped) = bytes.get(end + 1).copied() else {
                            return Err(Diagnostic::TokenizerError {
                                message: "unterminated string escape".to_string(),
                                span: SourceSpan::new(file, start as u32, (end + 1) as u32),
                                fixes: Vec::new(),
                            });
                        };
                        match escaped {
                            b'"' => content.push('"'),
                            b'\\' => content.push('\\'),
                            b'n' => content.push('\n'),
                            b'r' => content.push('\r'),
                            b't' => content.push('\t'),
                            other => {
                                content.push('\\');
                                content.push(other as char);
                            }
                        }
                        end += 2;
                    }
                    other => {
                        content.push(other as char);
                        end += 1;
                    }
                }
            }
            if !terminated {
                return Err(Diagnostic::TokenizerError {
                    message: "unterminated string literal".to_string(),
                    span: SourceSpan::new(file, start as u32, end as u32),
                    fixes: Vec::new(),
                });
            }
            tokens.push(Token {
                kind: TokenKind::StringLit(content),
                span: SourceSpan::new(file, start as u32, end as u32),
            });
            pos = end;
            continue;
        }

        return Err(Diagnostic::TokenizerError {
            message: format!("unexpected byte `{}`", byte as char),
            span: SourceSpan::new(file, start as u32, (start + 1) as u32),
            fixes: Vec::new(),
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
        (b'!', Some(b'=')) => Some((TokenKind::NotEq, 2)),
        (b'&', Some(b'&')) => Some((TokenKind::AmpAmp, 2)),
        (b'-', Some(b'>')) => Some((TokenKind::Arrow, 2)),
        (b'<', Some(b'=')) => Some((TokenKind::Le, 2)),
        (b'=', Some(b'=')) => Some((TokenKind::EqEq, 2)),
        (b'=', Some(b'>')) => Some((TokenKind::FatArrow, 2)),
        (b'>', Some(b'=')) => Some((TokenKind::Ge, 2)),
        (b'|', Some(b'>')) => Some((TokenKind::PipeArrow, 2)),
        (b'|', Some(b'|')) => Some((TokenKind::PipePipe, 2)),
        (b'(', _) => Some((TokenKind::LParen, 1)),
        (b')', _) => Some((TokenKind::RParen, 1)),
        (b'*', _) => Some((TokenKind::Star, 1)),
        (b'+', _) => Some((TokenKind::Plus, 1)),
        (b',', _) => Some((TokenKind::Comma, 1)),
        (b'-', _) => Some((TokenKind::Minus, 1)),
        (b'.', _) => Some((TokenKind::Dot, 1)),
        (b'/', _) => Some((TokenKind::Slash, 1)),
        (b':', _) => Some((TokenKind::Colon, 1)),
        (b';', _) => Some((TokenKind::Semicolon, 1)),
        (b'<', _) => Some((TokenKind::Lt, 1)),
        (b'=', _) => Some((TokenKind::Eq, 1)),
        (b'>', _) => Some((TokenKind::Gt, 1)),
        (b'?', _) => Some((TokenKind::Question, 1)),
        (b'[', _) => Some((TokenKind::LBracket, 1)),
        (b']', _) => Some((TokenKind::RBracket, 1)),
        (b'{', _) => Some((TokenKind::LBrace, 1)),
        (b'|', _) => Some((TokenKind::Pipe, 1)),
        (b'}', _) => Some((TokenKind::RBrace, 1)),
        _ => None,
    }
}
