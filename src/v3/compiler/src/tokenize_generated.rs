// AUTO-GENERATED from `src/v3/compiler/tokenize.dag` via
// `regen_tokenize`. Regenerate instead of hand-editing.

use crate::diagnostics::{Diagnostic, SourceSpan};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScannerCharClass {
    Whitespace,
    Digit,
    IdentStart,
    IdentContinue,
}

#[inline]
pub(crate) fn byte_matches(byte: u8, class: ScannerCharClass) -> bool {
    match class {
        ScannerCharClass::Whitespace => matches!(byte, b'\t' | b'\n' | b'\x0c' | b'\r' | b' '),
        ScannerCharClass::Digit => byte.is_ascii_digit(),
        ScannerCharClass::IdentStart => {
            byte.is_ascii_lowercase() || byte.is_ascii_uppercase() || byte == 0x5f
        }
        ScannerCharClass::IdentContinue => byte.is_ascii_alphanumeric() || byte == 0x5f,
    }
}

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
    IntLit(String),
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

fn minus_prefixed_decimal_allowed(prev: Option<&TokenKind>) -> bool {
    !matches!(
        prev,
        Some(
            TokenKind::Ident(_)
                | TokenKind::IntLit(_)
                | TokenKind::StringLit(_)
                | TokenKind::KwTrue
                | TokenKind::KwFalse
                | TokenKind::RParen
                | TokenKind::RBracket
                | TokenKind::RBrace
        )
    )
}

pub fn tokenize(source: &str, file: &str) -> Result<Vec<Token>, Diagnostic> {
    let bytes = source.as_bytes();
    let mut pos: usize = 0;
    let mut tokens = Vec::new();

    const LINE_COMMENT_PREFIX: &[u8] = b"//";
    while pos < bytes.len() {
        let byte = bytes[pos];

        let start = pos;

        if byte == b'-'
            && bytes
                .get(pos + 1)
                .is_some_and(|b| byte_matches(*b, ScannerCharClass::Digit))
            && minus_prefixed_decimal_allowed(tokens.last().map(|t: &Token| &t.kind))
        {
            let mut end = pos + 1;
            while end < bytes.len() && byte_matches(bytes[end], ScannerCharClass::Digit) {
                end += 1;
            }
            let literal = &source[pos + 1..end];
            let magnitude: u128 = literal.parse().map_err(|_| Diagnostic::TokenizerError {
                message: format!("{}{}{}", "invalid integer literal `", literal, "`"),
                span: SourceSpan::new(file, start as u32, end as u32),
                fixes: Vec::new(),
            })?;
            const MAX_SIGNED_ABS: u128 = 1u128 << 127;
            if magnitude > MAX_SIGNED_ABS {
                return Err(Diagnostic::TokenizerError {
                    message: format!("integer literal out of range for signed decimal literal: `-{}` (|m| > 2^127)", literal),
                    span: SourceSpan::new(file, start as u32, end as u32),
                    fixes: Vec::new(),
                });
            }
            let value = if magnitude == 0 {
                "0".to_string()
            } else {
                format!("-{}", magnitude)
            };
            tokens.push(Token {
                kind: TokenKind::IntLit(value),
                span: SourceSpan::new(file, start as u32, end as u32),
            });
            pos = end;
            continue;
        }

        if byte_matches(byte, ScannerCharClass::Whitespace) {
            pos += 1;
            continue;
        }

        if byte_matches(byte, ScannerCharClass::Digit) {
            let mut end = pos;
            while end < bytes.len() && byte_matches(bytes[end], ScannerCharClass::Digit) {
                end += 1;
            }
            let literal = &source[start..end];
            let magnitude: u128 = literal.parse().map_err(|_| Diagnostic::TokenizerError {
                message: format!("{}{}{}", "invalid integer literal `", literal, "`"),
                span: SourceSpan::new(file, start as u32, end as u32),
                fixes: Vec::new(),
            })?;
            let value = magnitude.to_string();
            tokens.push(Token {
                kind: TokenKind::IntLit(value),
                span: SourceSpan::new(file, start as u32, end as u32),
            });
            pos = end;
            continue;
        }

        if byte_matches(byte, ScannerCharClass::IdentStart) {
            let mut end = pos;
            while end < bytes.len() && byte_matches(bytes[end], ScannerCharClass::IdentContinue) {
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

        if byte_matches(byte, ScannerCharClass::IdentContinue) {
            let mut end = pos;
            while end < bytes.len() && byte_matches(bytes[end], ScannerCharClass::IdentContinue) {
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

        // Line comment prefix from `tokenize.dag` (`line_comment_prefix`).
        if bytes.len() >= pos + LINE_COMMENT_PREFIX.len()
            && bytes[pos..pos + LINE_COMMENT_PREFIX.len()].eq(LINE_COMMENT_PREFIX)
        {
            pos += LINE_COMMENT_PREFIX.len();
            while pos < bytes.len() && bytes[pos] != b'\n' {
                pos += 1;
            }
            continue;
        }

        if let Some((kind, width)) = punctuation_token(bytes, pos) {
            tokens.push(Token {
                kind,
                span: SourceSpan::new(file, start as u32, (start + width) as u32),
            });
            pos += width;
            continue;
        }

        // String literal (`string_literal_delimiter` + `StringEscapeSpec` rows).
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
                            b'"' => content.push(core::char::from_u32(34_u32).unwrap()),
                            b'\\' => content.push(core::char::from_u32(92_u32).unwrap()),
                            b'n' => content.push(core::char::from_u32(10_u32).unwrap()),
                            b'r' => content.push(core::char::from_u32(13_u32).unwrap()),
                            b't' => content.push(core::char::from_u32(9_u32).unwrap()),
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
