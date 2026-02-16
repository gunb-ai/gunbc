use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Module, Import, Type, Fn, Func, Pattern, Service, Resource, Interface, Pipeline,
    Let, Return, Match, If, Else, For, In, When, After, Node, Uses, Provides,
    Acquire, Release, Capability, Operation, Input, Output, Stage,
    True, False, NoneLit, As, Parallel, Config, With, SelfKw,
    // Delimiters
    LBrace, RBrace, LParen, RParen, LBracket, RBracket,
    // Operators
    Lt, Gt, PipeArrow, FatArrow, Arrow, Colon, Comma, Dot, Eq, Pipe,
    Plus, Minus, Star, Slash, Percent, Bang, And, Or,
    EqEq, Ne, Le, Ge, At, Question, NullCoalesce,
    // Strings
    Str(String), StrBegin(String), StrMid(String), StrEnd(String),
    // Literals
    Int(i64), Float(f64),
    // Identifier
    Ident(String),
    // End
    Eof,
}

impl TokenKind {
    pub fn desc(&self) -> &'static str {
        match self {
            Self::Module => "module", Self::Import => "import", Self::Type => "type",
            Self::Fn => "fn", Self::Func => "func", Self::Pattern => "pattern",
            Self::Service => "service", Self::Resource => "resource",
            Self::Interface => "interface", Self::Pipeline => "pipeline",
            Self::Let => "let", Self::Return => "return", Self::Match => "match",
            Self::If => "if", Self::Else => "else", Self::For => "for",
            Self::In => "in", Self::When => "when", Self::After => "after",
            Self::Node => "node", Self::Uses => "uses", Self::Provides => "provides",
            Self::Acquire => "acquire", Self::Release => "release",
            Self::Capability => "capability", Self::Operation => "operation",
            Self::Input => "input", Self::Output => "output", Self::Stage => "stage",
            Self::True => "true", Self::False => "false", Self::NoneLit => "none",
            Self::As => "as", Self::Parallel => "parallel", Self::Config => "config",
            Self::With => "with", Self::SelfKw => "self",
            Self::LBrace => "{", Self::RBrace => "}", Self::LParen => "(",
            Self::RParen => ")", Self::LBracket => "[", Self::RBracket => "]",
            Self::Lt => "<", Self::Gt => ">", Self::PipeArrow => "|>",
            Self::FatArrow => "=>", Self::Arrow => "->", Self::Colon => ":",
            Self::Comma => ",", Self::Dot => ".", Self::Eq => "=", Self::Pipe => "|",
            Self::Plus => "+", Self::Minus => "-", Self::Star => "*", Self::Slash => "/",
            Self::Percent => "%", Self::Bang => "!", Self::And => "&&", Self::Or => "||",
            Self::EqEq => "==", Self::Ne => "!=", Self::Le => "<=", Self::Ge => ">=",
            Self::At => "@", Self::Question => "?", Self::NullCoalesce => "??",
            Self::Str(_) => "string", Self::StrBegin(_) => "string-begin",
            Self::StrMid(_) => "string-mid", Self::StrEnd(_) => "string-end",
            Self::Int(_) => "integer", Self::Float(_) => "float",
            Self::Ident(_) => "identifier", Self::Eof => "EOF",
        }
    }
}

pub struct Lexer<'a> {
    source: &'a [u8],
    pos: usize,
    interp_depth: Vec<usize>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source: source.as_bytes(), pos: 0, interp_depth: Vec::new() }
    }

    pub fn tokenize(source: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(source);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            let done = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if done { break; }
        }
        tokens
    }

    fn peek(&self) -> u8 {
        if self.pos < self.source.len() { self.source[self.pos] } else { 0 }
    }

    fn peek_at(&self, off: usize) -> u8 {
        let i = self.pos + off;
        if i < self.source.len() { self.source[i] } else { 0 }
    }

    fn advance(&mut self) -> u8 {
        let ch = self.peek();
        if self.pos < self.source.len() { self.pos += 1; }
        ch
    }

    fn slice(&self, start: usize, end: usize) -> &str {
        std::str::from_utf8(&self.source[start..end]).unwrap_or("")
    }

    fn tok(&self, kind: TokenKind, start: usize) -> Token {
        Token { kind, span: Span { start, end: self.pos } }
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
        let start = self.pos;
        if self.pos >= self.source.len() {
            return self.tok(TokenKind::Eof, start);
        }
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
            b'{' => { self.advance(); if let Some(d) = self.interp_depth.last_mut() { *d += 1; } self.tok(TokenKind::LBrace, start) }
            b'}' => { self.advance(); if let Some(d) = self.interp_depth.last_mut() { *d -= 1; } self.tok(TokenKind::RBrace, start) }
            b'(' => { self.advance(); self.tok(TokenKind::LParen, start) }
            b')' => { self.advance(); self.tok(TokenKind::RParen, start) }
            b'[' => { self.advance(); self.tok(TokenKind::LBracket, start) }
            b']' => { self.advance(); self.tok(TokenKind::RBracket, start) }
            b':' => { self.advance(); self.tok(TokenKind::Colon, start) }
            b',' => { self.advance(); self.tok(TokenKind::Comma, start) }
            b'.' => { self.advance(); self.tok(TokenKind::Dot, start) }
            b'+' => { self.advance(); self.tok(TokenKind::Plus, start) }
            b'*' => { self.advance(); self.tok(TokenKind::Star, start) }
            b'%' => { self.advance(); self.tok(TokenKind::Percent, start) }
            b'@' => { self.advance(); self.tok(TokenKind::At, start) }
            b'/' => { self.advance(); self.tok(TokenKind::Slash, start) }
            b'-' => { self.advance(); if self.peek() == b'>' { self.advance(); self.tok(TokenKind::Arrow, start) } else { self.tok(TokenKind::Minus, start) } }
            b'=' => { self.advance(); if self.peek() == b'=' { self.advance(); self.tok(TokenKind::EqEq, start) } else if self.peek() == b'>' { self.advance(); self.tok(TokenKind::FatArrow, start) } else { self.tok(TokenKind::Eq, start) } }
            b'!' => { self.advance(); if self.peek() == b'=' { self.advance(); self.tok(TokenKind::Ne, start) } else { self.tok(TokenKind::Bang, start) } }
            b'<' => { self.advance(); if self.peek() == b'=' { self.advance(); self.tok(TokenKind::Le, start) } else { self.tok(TokenKind::Lt, start) } }
            b'>' => { self.advance(); if self.peek() == b'=' { self.advance(); self.tok(TokenKind::Ge, start) } else { self.tok(TokenKind::Gt, start) } }
            b'|' => { self.advance(); if self.peek() == b'>' { self.advance(); self.tok(TokenKind::PipeArrow, start) } else if self.peek() == b'|' { self.advance(); self.tok(TokenKind::Or, start) } else { self.tok(TokenKind::Pipe, start) } }
            b'&' => { self.advance(); if self.peek() == b'&' { self.advance(); self.tok(TokenKind::And, start) } else { self.tok(TokenKind::Ident("&".into()), start) } }
            b'?' => { self.advance(); if self.peek() == b'?' { self.advance(); self.tok(TokenKind::NullCoalesce, start) } else { self.tok(TokenKind::Question, start) } }
            _ => { self.advance(); self.tok(TokenKind::Ident(format!("<unknown:{}>", ch as char)), start) }
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
            "module" => TokenKind::Module, "import" => TokenKind::Import,
            "type" => TokenKind::Type, "fn" => TokenKind::Fn, "func" => TokenKind::Func,
            "pattern" => TokenKind::Pattern, "service" => TokenKind::Service,
            "resource" => TokenKind::Resource, "interface" => TokenKind::Interface,
            "pipeline" => TokenKind::Pipeline, "let" => TokenKind::Let,
            "return" => TokenKind::Return, "match" => TokenKind::Match,
            "if" => TokenKind::If, "else" => TokenKind::Else, "for" => TokenKind::For,
            "in" => TokenKind::In, "when" => TokenKind::When, "after" => TokenKind::After,
            "node" => TokenKind::Node, "uses" => TokenKind::Uses,
            "provides" => TokenKind::Provides, "acquire" => TokenKind::Acquire,
            "release" => TokenKind::Release, "capability" => TokenKind::Capability,
            "operation" => TokenKind::Operation, "input" => TokenKind::Input,
            "output" => TokenKind::Output, "stage" => TokenKind::Stage,
            "true" => TokenKind::True, "false" => TokenKind::False,
            "none" => TokenKind::NoneLit, "as" => TokenKind::As,
            "parallel" => TokenKind::Parallel, "config" => TokenKind::Config,
            "with" => TokenKind::With, "self" | "Self" => TokenKind::SelfKw,
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
        if self.pos >= self.source.len() { return; }
        match self.peek() {
            b'n' => { buf.push('\n'); self.pos += 1; }
            b't' => { buf.push('\t'); self.pos += 1; }
            b'\\' => { buf.push('\\'); self.pos += 1; }
            b'"' => { buf.push('"'); self.pos += 1; }
            b'{' => { buf.push('{'); self.pos += 1; }
            b'}' => { buf.push('}'); self.pos += 1; }
            other => { buf.push('\\'); buf.push(other as char); self.pos += 1; }
        }
    }

    fn scan_str(&mut self, start: usize) -> Token {
        self.pos += 1; // skip opening "
        let mut buf = String::new();
        loop {
            if self.pos >= self.source.len() { break; }
            match self.peek() {
                b'"' => { self.pos += 1; return self.tok(TokenKind::Str(buf), start); }
                b'{' => { self.pos += 1; self.interp_depth.push(0); return self.tok(TokenKind::StrBegin(buf), start); }
                b'\\' => { self.pos += 1; self.scan_escape(&mut buf); }
                ch => { buf.push(ch as char); self.pos += 1; }
            }
        }
        self.tok(TokenKind::Str(buf), start)
    }

    fn scan_str_cont(&mut self, start: usize) -> Token {
        let mut buf = String::new();
        loop {
            if self.pos >= self.source.len() { break; }
            match self.peek() {
                b'"' => { self.pos += 1; return self.tok(TokenKind::StrEnd(buf), start); }
                b'{' => { self.pos += 1; self.interp_depth.push(0); return self.tok(TokenKind::StrMid(buf), start); }
                b'\\' => { self.pos += 1; self.scan_escape(&mut buf); }
                ch => { buf.push(ch as char); self.pos += 1; }
            }
        }
        self.tok(TokenKind::StrEnd(buf), start)
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
        assert_eq!(kinds("module foo import bar"), vec![
            TokenKind::Module, TokenKind::Ident("foo".into()),
            TokenKind::Import, TokenKind::Ident("bar".into()), TokenKind::Eof,
        ]);
    }

    #[test]
    fn operators() {
        assert_eq!(kinds("|> => -> == != <= >= && || ??"), vec![
            TokenKind::PipeArrow, TokenKind::FatArrow, TokenKind::Arrow,
            TokenKind::EqEq, TokenKind::Ne, TokenKind::Le, TokenKind::Ge,
            TokenKind::And, TokenKind::Or, TokenKind::NullCoalesce, TokenKind::Eof,
        ]);
    }

    #[test]
    fn numbers() {
        assert_eq!(kinds("42 3.14"), vec![
            TokenKind::Int(42), TokenKind::Float(3.14), TokenKind::Eof,
        ]);
    }

    #[test]
    fn simple_string() {
        assert_eq!(kinds(r#""hello""#), vec![TokenKind::Str("hello".into()), TokenKind::Eof]);
    }

    #[test]
    fn string_interp() {
        assert_eq!(kinds(r#""hi {name}!""#), vec![
            TokenKind::StrBegin("hi ".into()), TokenKind::Ident("name".into()),
            TokenKind::StrEnd("!".into()), TokenKind::Eof,
        ]);
    }

    #[test]
    fn string_interp_multi() {
        assert_eq!(kinds(r#""{a}_{b}""#), vec![
            TokenKind::StrBegin(String::new()), TokenKind::Ident("a".into()),
            TokenKind::StrMid("_".into()), TokenKind::Ident("b".into()),
            TokenKind::StrEnd(String::new()), TokenKind::Eof,
        ]);
    }

    #[test]
    fn comments_skipped() {
        assert_eq!(kinds("foo // comment\nbar"), vec![
            TokenKind::Ident("foo".into()), TokenKind::Ident("bar".into()), TokenKind::Eof,
        ]);
    }
}
