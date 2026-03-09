use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Project,
    Feature,
    Task,
    Design,
    Component,
    Environment,
    Module,
    Import,
    Type,
    Fn,
    Func,
    Pattern,
    Service,
    Resource,
    Extern,
    Interface,
    Pipeline,
    Profile,
    Bind,
    Let,
    Return,
    Match,
    If,
    Else,
    For,
    In,
    When,
    After,
    Node,
    Uses,
    Provides,
    Acquire,
    Release,
    Capability,
    Operation,
    Input,
    Output,
    Stage,
    True,
    False,
    NoneLit,
    As,
    Implements,
    Parallel,
    Config,
    With,
    SelfKw,
    ParamKw,
    DataKw,
    // Test DSL keywords
    Test,
    Fixture,
    Mock,
    Expect,
    Contains,
    // Typed syntax keywords
    From,
    Where,
    Transport,
    Response,
    Exit,
    Idempotent,
    Readonly,
    Hermetic,
    Contract,
    Tier,
    Skip,
    // Delimiters
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,
    // Operators
    Lt,
    Gt,
    PipeArrow,
    FatArrow,
    Arrow,
    Colon,
    Comma,
    Dot,
    Eq,
    Pipe,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    And,
    Or,
    EqEq,
    Ne,
    Le,
    Ge,
    Question,
    NullCoalesce,
    // Strings
    Str(String),
    StrBegin(String),
    StrMid(String),
    StrEnd(String),
    // Literals
    Int(i64),
    Float(f64),
    // Identifier
    Ident(String),
    // Invalid token encountered during lexing.
    Unknown(char),
    // End
    Eof,
}

impl TokenKind {
    pub fn desc(&self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Feature => "feature",
            Self::Task => "task",
            Self::Design => "design",
            Self::Component => "component",
            Self::Environment => "environment",
            Self::Module => "module",
            Self::Import => "import",
            Self::Type => "type",
            Self::Fn => "fn",
            Self::Func => "func",
            Self::Pattern => "pattern",
            Self::Service => "service",
            Self::Resource => "resource",
            Self::Extern => "extern",
            Self::Interface => "interface",
            Self::Pipeline => "pipeline",
            Self::Profile => "profile",
            Self::Bind => "bind",
            Self::Let => "let",
            Self::Return => "return",
            Self::Match => "match",
            Self::If => "if",
            Self::Else => "else",
            Self::For => "for",
            Self::In => "in",
            Self::When => "when",
            Self::After => "after",
            Self::Node => "node",
            Self::Uses => "uses",
            Self::Provides => "provides",
            Self::Acquire => "acquire",
            Self::Release => "release",
            Self::Capability => "capability",
            Self::Operation => "operation",
            Self::Input => "input",
            Self::Output => "output",
            Self::Stage => "stage",
            Self::True => "true",
            Self::False => "false",
            Self::NoneLit => "none",
            Self::As => "as",
            Self::Implements => "implements",
            Self::Parallel => "parallel",
            Self::Config => "config",
            Self::With => "with",
            Self::SelfKw => "self",
            Self::ParamKw => "param",
            Self::DataKw => "data",
            Self::Test => "test",
            Self::Fixture => "fixture",
            Self::Mock => "mock",
            Self::Expect => "expect",
            Self::Contains => "contains",
            Self::From => "from",
            Self::Where => "where",
            Self::Transport => "transport",
            Self::Response => "response",
            Self::Exit => "exit",
            Self::Idempotent => "idempotent",
            Self::Readonly => "readonly",
            Self::Hermetic => "hermetic",
            Self::Contract => "contract",
            Self::Tier => "tier",
            Self::Skip => "skip",
            Self::LBrace => "{",
            Self::RBrace => "}",
            Self::LParen => "(",
            Self::RParen => ")",
            Self::LBracket => "[",
            Self::RBracket => "]",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::PipeArrow => "|>",
            Self::FatArrow => "=>",
            Self::Arrow => "->",
            Self::Colon => ":",
            Self::Comma => ",",
            Self::Dot => ".",
            Self::Eq => "=",
            Self::Pipe => "|",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Star => "*",
            Self::Slash => "/",
            Self::Percent => "%",
            Self::Bang => "!",
            Self::And => "&&",
            Self::Or => "||",
            Self::EqEq => "==",
            Self::Ne => "!=",
            Self::Le => "<=",
            Self::Ge => ">=",
            Self::Question => "?",
            Self::NullCoalesce => "??",
            Self::Str(_) => "string",
            Self::StrBegin(_) => "string-begin",
            Self::StrMid(_) => "string-mid",
            Self::StrEnd(_) => "string-end",
            Self::Int(_) => "integer",
            Self::Float(_) => "float",
            Self::Ident(_) => "identifier",
            Self::Unknown(_) => "unknown-token",
            Self::Eof => "EOF",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

impl LexError {
    pub fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic::new(DiagnosticKind::Lex, self.message.clone()).with_span(self.span)
    }
}

pub struct Lexer<'a> {
    source: &'a [u8],
    pos: usize,
    interp_depth: Vec<usize>,
    errors: Vec<LexError>,
}

// FC-9: Hex escape helpers for \xHH sequences.
fn is_hex_digit(b: u8) -> bool {
    b.is_ascii_hexdigit()
}

fn hex_val(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0,
    }
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source: source.as_bytes(),
            pos: 0,
            interp_depth: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn tokenize(source: &str) -> Vec<Token> {
        let (tokens, _) = Self::tokenize_with_errors(source);
        tokens
    }

    pub fn tokenize_with_diagnostics(source: &str) -> (Vec<Token>, Vec<Diagnostic>) {
        let (tokens, errors) = Self::tokenize_with_errors(source);
        let diagnostics = errors
            .into_iter()
            .map(|error| error.to_diagnostic())
            .collect();
        (tokens, diagnostics)
    }

    pub fn tokenize_with_errors(source: &str) -> (Vec<Token>, Vec<LexError>) {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            let done = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if done {
                break;
            }
        }
        (tokens, lexer.errors)
    }

    fn peek(&self) -> u8 {
        if self.pos < self.source.len() {
            self.source[self.pos]
        } else {
            0
        }
    }

    fn peek_at(&self, off: usize) -> u8 {
        let i = self.pos + off;
        if i < self.source.len() {
            self.source[i]
        } else {
            0
        }
    }

    fn advance(&mut self) -> u8 {
        let ch = self.peek();
        if self.pos < self.source.len() {
            self.pos += 1;
        }
        ch
    }

    fn slice(&self, start: usize, end: usize) -> &str {
        std::str::from_utf8(&self.source[start..end]).unwrap_or("")
    }

    /// Decode a single UTF-8 character starting at `self.pos`.
    /// Returns the character and the number of bytes it occupies.
    fn decode_utf8_char(&self) -> (char, usize) {
        let b0 = self.source[self.pos];
        let seq_len = if b0 < 0x80 {
            1
        } else if b0 < 0xE0 {
            2
        } else if b0 < 0xF0 {
            3
        } else {
            4
        };
        let end = (self.pos + seq_len).min(self.source.len());
        match std::str::from_utf8(&self.source[self.pos..end]) {
            Ok(s) => (s.chars().next().unwrap_or('\u{FFFD}'), seq_len),
            Err(_) => ('\u{FFFD}', 1),
        }
    }

    fn tok(&self, kind: TokenKind, start: usize) -> Token {
        Token {
            kind,
            span: Span {
                start,
                end: self.pos,
            },
        }
    }

    fn push_error(&mut self, message: impl Into<String>, span: Span) {
        self.errors.push(LexError {
            message: message.into(),
            span,
        });
    }

    fn skip_ws(&mut self) {
        loop {
            while self.pos < self.source.len()
                && matches!(self.peek(), b' ' | b'\t' | b'\n' | b'\r')
            {
                self.pos += 1;
            }
            if self.pos + 1 < self.source.len() && self.peek() == b'/' && self.peek_at(1) == b'/' {
                while self.pos < self.source.len() && self.peek() != b'\n' {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_ws();
        if self.pos >= self.source.len() {
            if !self.interp_depth.is_empty() {
                self.push_error(
                    "unterminated interpolated string literal".to_string(),
                    Span {
                        start: self.pos,
                        end: self.pos,
                    },
                );
                self.interp_depth.clear();
            }
            return self.tok(TokenKind::Eof, self.pos);
        }

        // Inside string interpolation expressions, authors often escape inner
        // quotes as `\"` because the interpolation is lexed from within an
        // outer string literal. Treat that pair as a normal quote in interp
        // mode by dropping the escape slash.
        if !self.interp_depth.is_empty() && self.peek() == b'\\' && self.peek_at(1) == b'"' {
            self.pos += 1;
        }

        let start = self.pos;
        let ch = self.peek();

        if ch == b'}' {
            if let Some(&depth) = self.interp_depth.last() {
                if depth == 0 {
                    self.pos += 1;
                    self.interp_depth.pop();
                    return self.scan_str_cont(start);
                }
            }
        }

        match ch {
            b'"' => self.scan_str(start),
            b'0'..=b'9' => self.scan_num(start),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.scan_ident(start),
            b'{' => {
                self.advance();
                if let Some(d) = self.interp_depth.last_mut() {
                    *d += 1;
                }
                self.tok(TokenKind::LBrace, start)
            }
            b'}' => {
                self.advance();
                if let Some(d) = self.interp_depth.last_mut() {
                    *d -= 1;
                }
                self.tok(TokenKind::RBrace, start)
            }
            b'(' => {
                self.advance();
                self.tok(TokenKind::LParen, start)
            }
            b')' => {
                self.advance();
                self.tok(TokenKind::RParen, start)
            }
            b'[' => {
                self.advance();
                self.tok(TokenKind::LBracket, start)
            }
            b']' => {
                self.advance();
                self.tok(TokenKind::RBracket, start)
            }
            b':' => {
                self.advance();
                self.tok(TokenKind::Colon, start)
            }
            b',' => {
                self.advance();
                self.tok(TokenKind::Comma, start)
            }
            b'.' => {
                self.advance();
                self.tok(TokenKind::Dot, start)
            }
            b'+' => {
                self.advance();
                self.tok(TokenKind::Plus, start)
            }
            b'*' => {
                self.advance();
                self.tok(TokenKind::Star, start)
            }
            b'%' => {
                self.advance();
                self.tok(TokenKind::Percent, start)
            }
            // b'@' — no longer valid syntax, falls through to error handler
            b'/' => {
                self.advance();
                self.tok(TokenKind::Slash, start)
            }
            b'-' => {
                self.advance();
                if self.peek() == b'>' {
                    self.advance();
                    self.tok(TokenKind::Arrow, start)
                } else {
                    self.tok(TokenKind::Minus, start)
                }
            }
            b'=' => {
                self.advance();
                if self.peek() == b'=' {
                    self.advance();
                    self.tok(TokenKind::EqEq, start)
                } else if self.peek() == b'>' {
                    self.advance();
                    self.tok(TokenKind::FatArrow, start)
                } else {
                    self.tok(TokenKind::Eq, start)
                }
            }
            b'!' => {
                self.advance();
                if self.peek() == b'=' {
                    self.advance();
                    self.tok(TokenKind::Ne, start)
                } else {
                    self.tok(TokenKind::Bang, start)
                }
            }
            b'<' => {
                self.advance();
                if self.peek() == b'=' {
                    self.advance();
                    self.tok(TokenKind::Le, start)
                } else {
                    self.tok(TokenKind::Lt, start)
                }
            }
            b'>' => {
                self.advance();
                if self.peek() == b'=' {
                    self.advance();
                    self.tok(TokenKind::Ge, start)
                } else {
                    self.tok(TokenKind::Gt, start)
                }
            }
            b'|' => {
                self.advance();
                if self.peek() == b'>' {
                    self.advance();
                    self.tok(TokenKind::PipeArrow, start)
                } else if self.peek() == b'|' {
                    self.advance();
                    self.tok(TokenKind::Or, start)
                } else {
                    self.tok(TokenKind::Pipe, start)
                }
            }
            b'&' => {
                self.advance();
                if self.peek() == b'&' {
                    self.advance();
                    self.tok(TokenKind::And, start)
                } else {
                    self.push_error(
                        "unexpected character '&'".to_string(),
                        Span {
                            start,
                            end: self.pos,
                        },
                    );
                    self.tok(TokenKind::Unknown('&'), start)
                }
            }
            b'?' => {
                self.advance();
                if self.peek() == b'?' {
                    self.advance();
                    self.tok(TokenKind::NullCoalesce, start)
                } else {
                    self.tok(TokenKind::Question, start)
                }
            }
            _ => {
                self.advance();
                self.push_error(
                    format!("unexpected character '{}'", ch as char),
                    Span {
                        start,
                        end: self.pos,
                    },
                );
                self.tok(TokenKind::Unknown(ch as char), start)
            }
        }
    }

    fn scan_ident(&mut self, start: usize) -> Token {
        while self.pos < self.source.len()
            && matches!(self.peek(), b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_')
        {
            self.pos += 1;
        }
        let text = self.slice(start, self.pos);
        let kind = match text {
            "project" => TokenKind::Project,
            "feature" => TokenKind::Feature,
            "task" => TokenKind::Task,
            "design" => TokenKind::Design,
            "component" => TokenKind::Component,
            "environment" => TokenKind::Environment,
            "module" => TokenKind::Module,
            "import" => TokenKind::Import,
            "type" => TokenKind::Type,
            "fn" => TokenKind::Fn,
            "func" => TokenKind::Func,
            "pattern" => TokenKind::Pattern,
            "service" => TokenKind::Service,
            "resource" => TokenKind::Resource,
            "extern" => TokenKind::Extern,
            "interface" => TokenKind::Interface,
            "pipeline" => TokenKind::Pipeline,
            "profile" => TokenKind::Profile,
            "bind" => TokenKind::Bind,
            "let" => TokenKind::Let,
            "return" => TokenKind::Return,
            "match" => TokenKind::Match,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "when" => TokenKind::When,
            "after" => TokenKind::After,
            "node" => TokenKind::Node,
            "uses" => TokenKind::Uses,
            "provides" => TokenKind::Provides,
            "acquire" => TokenKind::Acquire,
            "release" => TokenKind::Release,
            "capability" => TokenKind::Capability,
            "operation" => TokenKind::Operation,
            "input" => TokenKind::Input,
            "output" => TokenKind::Output,
            "stage" => TokenKind::Stage,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "none" => TokenKind::NoneLit,
            "as" => TokenKind::As,
            "implements" => TokenKind::Implements,
            "parallel" => TokenKind::Parallel,
            "config" => TokenKind::Config,
            "with" => TokenKind::With,
            "self" | "Self" => TokenKind::SelfKw,
            "param" => TokenKind::ParamKw,
            "data" => TokenKind::DataKw,
            "test" => TokenKind::Test,
            "fixture" => TokenKind::Fixture,
            "mock" => TokenKind::Mock,
            "expect" => TokenKind::Expect,
            "contains" => TokenKind::Contains,
            "from" => TokenKind::From,
            "where" => TokenKind::Where,
            "transport" => TokenKind::Transport,
            "response" => TokenKind::Response,
            "exit" => TokenKind::Exit,
            "idempotent" => TokenKind::Idempotent,
            "readonly" => TokenKind::Readonly,
            "hermetic" => TokenKind::Hermetic,
            "contract" => TokenKind::Contract,
            "tier" => TokenKind::Tier,
            "skip" => TokenKind::Skip,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            _ => TokenKind::Ident(text.to_string()),
        };
        self.tok(kind, start)
    }

    fn scan_num(&mut self, start: usize) -> Token {
        while self.pos < self.source.len() && self.peek().is_ascii_digit() {
            self.pos += 1;
        }
        if self.peek() == b'.' && self.peek_at(1).is_ascii_digit() {
            self.pos += 1;
            while self.pos < self.source.len() && self.peek().is_ascii_digit() {
                self.pos += 1;
            }
            let text = self.slice(start, self.pos);
            self.tok(TokenKind::Float(text.parse().unwrap_or(0.0)), start)
        } else {
            let text = self.slice(start, self.pos);
            self.tok(TokenKind::Int(text.parse().unwrap_or(0)), start)
        }
    }

    fn scan_escape(&mut self, buf: &mut String) {
        if self.pos >= self.source.len() {
            return;
        }
        match self.peek() {
            b'n' => {
                buf.push('\n');
                self.pos += 1;
            }
            b't' => {
                buf.push('\t');
                self.pos += 1;
            }
            b'\\' => {
                buf.push('\\');
                self.pos += 1;
            }
            b'"' => {
                buf.push('"');
                self.pos += 1;
            }
            b'{' => {
                buf.push('{');
                self.pos += 1;
            }
            b'}' => {
                buf.push('}');
                self.pos += 1;
            }
            // FC-9: \xHH hex escape — interpret as the byte value.
            b'x' => {
                self.pos += 1; // consume 'x'
                let hi = self.peek();
                let lo = self.peek_at(1);
                if is_hex_digit(hi) && is_hex_digit(lo) {
                    let byte = (hex_val(hi) << 4) | hex_val(lo);
                    buf.push(byte as char);
                    self.pos += 2;
                } else {
                    // Malformed \x escape — preserve literally for fail-closed diagnostics.
                    buf.push('\\');
                    buf.push('x');
                }
            }
            other if other >= 0x80 => {
                buf.push('\\');
                let (ch, len) = self.decode_utf8_char();
                buf.push(ch);
                self.pos += len;
            }
            other => {
                buf.push('\\');
                buf.push(other as char);
                self.pos += 1;
            }
        }
    }

    fn scan_str(&mut self, start: usize) -> Token {
        self.pos += 1; // skip opening "
        let mut buf = String::new();
        loop {
            if self.pos >= self.source.len() {
                break;
            }
            match self.peek() {
                b'"' => {
                    self.pos += 1;
                    return self.tok(TokenKind::Str(buf), start);
                }
                b'{' => {
                    if self.should_start_interpolation() {
                        self.pos += 1;
                        self.interp_depth.push(0);
                        return self.tok(TokenKind::StrBegin(buf), start);
                    }
                    buf.push('{');
                    self.pos += 1;
                }
                b'\\' => {
                    if !self.interp_depth.is_empty() && self.peek_at(1) == b'"' {
                        self.pos += 1;
                        continue;
                    }
                    self.pos += 1;
                    self.scan_escape(&mut buf);
                }
                b if b >= 0x80 => {
                    let (ch, len) = self.decode_utf8_char();
                    buf.push(ch);
                    self.pos += len;
                }
                ch => {
                    buf.push(ch as char);
                    self.pos += 1;
                }
            }
        }
        self.push_error(
            "unterminated string literal".to_string(),
            Span {
                start,
                end: self.pos,
            },
        );
        self.tok(TokenKind::Str(buf), start)
    }

    fn scan_str_cont(&mut self, start: usize) -> Token {
        let mut buf = String::new();
        loop {
            if self.pos >= self.source.len() {
                break;
            }
            match self.peek() {
                b'"' => {
                    self.pos += 1;
                    return self.tok(TokenKind::StrEnd(buf), start);
                }
                b'{' => {
                    if self.should_start_interpolation() {
                        self.pos += 1;
                        self.interp_depth.push(0);
                        return self.tok(TokenKind::StrMid(buf), start);
                    }
                    buf.push('{');
                    self.pos += 1;
                }
                b'\\' => {
                    self.pos += 1;
                    self.scan_escape(&mut buf);
                }
                b if b >= 0x80 => {
                    let (ch, len) = self.decode_utf8_char();
                    buf.push(ch);
                    self.pos += len;
                }
                ch => {
                    buf.push(ch as char);
                    self.pos += 1;
                }
            }
        }
        self.push_error(
            "unterminated interpolated string literal".to_string(),
            Span {
                start,
                end: self.pos,
            },
        );
        self.tok(TokenKind::StrEnd(buf), start)
    }

    fn should_start_interpolation(&self) -> bool {
        let next = self.peek_at(1);
        matches!(next, b'a'..=b'z' | b'A'..=b'Z' | b'_') || matches!(next, b'(' | b'!' | b'-')
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(src: &str) -> Vec<TokenKind> {
        Lexer::tokenize(src).into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn keywords_and_idents() {
        assert_eq!(
            kinds("module foo import bar"),
            vec![
                TokenKind::Module,
                TokenKind::Ident("foo".into()),
                TokenKind::Import,
                TokenKind::Ident("bar".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn operators() {
        assert_eq!(
            kinds("|> => -> == != <= >= && || ??"),
            vec![
                TokenKind::PipeArrow,
                TokenKind::FatArrow,
                TokenKind::Arrow,
                TokenKind::EqEq,
                TokenKind::Ne,
                TokenKind::Le,
                TokenKind::Ge,
                TokenKind::And,
                TokenKind::Or,
                TokenKind::NullCoalesce,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn longest_match_operators() {
        assert_eq!(
            kinds("|>| => = > -> - !="),
            vec![
                TokenKind::PipeArrow,
                TokenKind::Pipe,
                TokenKind::FatArrow,
                TokenKind::Eq,
                TokenKind::Gt,
                TokenKind::Arrow,
                TokenKind::Minus,
                TokenKind::Ne,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn longest_match_comparison_and_question_operators() {
        assert_eq!(
            kinds("<= < >= > ?? ? != !"),
            vec![
                TokenKind::Le,
                TokenKind::Lt,
                TokenKind::Ge,
                TokenKind::Gt,
                TokenKind::NullCoalesce,
                TokenKind::Question,
                TokenKind::Ne,
                TokenKind::Bang,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn numbers() {
        assert_eq!(
            kinds("42 2.5"),
            vec![TokenKind::Int(42), TokenKind::Float(2.5), TokenKind::Eof,]
        );
    }

    #[test]
    fn simple_string() {
        assert_eq!(
            kinds(r#""hello""#),
            vec![TokenKind::Str("hello".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn string_interp() {
        assert_eq!(
            kinds(r#""hi {name}!""#),
            vec![
                TokenKind::StrBegin("hi ".into()),
                TokenKind::Ident("name".into()),
                TokenKind::StrEnd("!".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn string_interp_multi() {
        assert_eq!(
            kinds(r#""{a}_{b}""#),
            vec![
                TokenKind::StrBegin(String::new()),
                TokenKind::Ident("a".into()),
                TokenKind::StrMid("_".into()),
                TokenKind::Ident("b".into()),
                TokenKind::StrEnd(String::new()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn comments_skipped() {
        assert_eq!(
            kinds("foo // comment\nbar"),
            vec![
                TokenKind::Ident("foo".into()),
                TokenKind::Ident("bar".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn unknown_token_reports_lex_error() {
        let (_tokens, errors) = Lexer::tokenize_with_errors("module test\n$");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("unexpected character '$'"));
    }

    #[test]
    fn unknown_token_reports_lex_diagnostic() {
        let (_tokens, diagnostics) = Lexer::tokenize_with_diagnostics("module test\n$");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].kind, DiagnosticKind::Lex);
        assert!(diagnostics[0].message.contains("unexpected character '$'"));
    }

    #[test]
    fn unterminated_string_reports_lex_error() {
        let (_tokens, errors) = Lexer::tokenize_with_errors("module test\n\"unterminated");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("unterminated string literal"));
    }

    #[test]
    fn unterminated_interpolated_string_reports_lex_error() {
        let (_tokens, errors) = Lexer::tokenize_with_errors("module test\n\"hello {name");
        assert_eq!(errors.len(), 1);
        assert!(errors[0]
            .message
            .contains("unterminated interpolated string literal"));
    }

    #[test]
    fn single_ampersand_reports_lex_error() {
        let (tokens, errors) = Lexer::tokenize_with_errors("module test\n&");
        assert!(matches!(tokens[2].kind, TokenKind::Unknown('&')));
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("unexpected character '&'"));
    }

    // ── FC-9: \xHH hex escape contract tests ──────────────────────────

    #[test]
    fn hex_escape_produces_byte_value() {
        let tokens = Lexer::tokenize(r#""hello \x1b[0m""#);
        let s = match &tokens[0].kind {
            TokenKind::Str(s) => s.clone(),
            other => panic!("expected string token, got {other:?}"),
        };
        // \x1b should be interpreted as the ESC character (byte 0x1b)
        assert!(
            s.contains('\x1b'),
            "\\x1b should produce ESC byte, got: {:?}",
            s
        );
    }

    #[test]
    fn hex_escape_uppercase_produces_byte_value() {
        let tokens = Lexer::tokenize(r#""\x41""#);
        let s = match &tokens[0].kind {
            TokenKind::Str(s) => s.clone(),
            other => panic!("expected string token, got {other:?}"),
        };
        assert_eq!(s, "A", "\\x41 should produce 'A'");
    }

    #[test]
    fn extern_keyword() {
        assert_eq!(
            kinds("extern func extern asset"),
            vec![
                TokenKind::Extern,
                TokenKind::Func,
                TokenKind::Extern,
                TokenKind::Ident("asset".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn malformed_hex_escape_preserved_literally() {
        let tokens = Lexer::tokenize(r#""\xZZ""#);
        let s = match &tokens[0].kind {
            TokenKind::Str(s) => s.clone(),
            other => panic!("expected string token, got {other:?}"),
        };
        assert_eq!(s, "\\xZZ", "malformed \\xHH should preserve literally");
    }

    // ── Interpolation (was tests/lexer_interpolation.rs) ───────────────

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

    #[test]
    fn interpolation_does_not_start_on_numeric_brace_sequences() {
        assert_eq!(
            kinds(r#""regex \d{2}-\w{3}""#),
            vec![TokenKind::Str(r"regex \d{2}-\w{3}".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn interpolation_lexes_escaped_quotes_in_inner_string_consistently() {
        assert_eq!(
            kinds(r#""outer {inner_fn(\"arg\")} end""#),
            vec![
                TokenKind::StrBegin("outer ".into()),
                TokenKind::Ident("inner_fn".into()),
                TokenKind::LParen,
                TokenKind::Str("arg".into()),
                TokenKind::RParen,
                TokenKind::StrEnd(" end".into()),
                TokenKind::Eof,
            ]
        );
    }
}
