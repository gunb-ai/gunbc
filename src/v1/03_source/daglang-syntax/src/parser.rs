use crate::ast::*;
use crate::diagnostic::{Diagnostic, DiagnosticKind};
use crate::lexer::{Lexer, Token, TokenKind};
use crate::span::{Span, Spanned};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "parse error at {}-{}: {}",
            self.span.start, self.span.end, self.message
        )
    }
}

impl ParseError {
    /// Convert the start byte of this error span to a 1-based line/column pair.
    pub fn line_col(&self, source: &str) -> (usize, usize) {
        byte_to_line_col(source, self.span.start)
    }

    /// Format this parse error with file + line/column information.
    pub fn format_with_source(&self, file: &Path, source: &str) -> String {
        self.to_diagnostic(file, source).render()
    }

    pub fn to_diagnostic(&self, file: &Path, source: &str) -> Diagnostic {
        let (line, col) = self.line_col(source);
        Diagnostic::new(DiagnosticKind::Parse, self.message.clone())
            .with_file(file)
            .with_span(self.span)
            .with_line_col(line, col)
    }
}

/// Convert a byte offset into a 1-based (line, column) pair.
///
/// Offsets beyond EOF are clamped to EOF.
pub fn byte_to_line_col(source: &str, byte_offset: usize) -> (usize, usize) {
    let clamped = byte_offset.min(source.len());
    let mut line = 1usize;
    let mut col = 1usize;

    for (idx, ch) in source.char_indices() {
        if idx >= clamped {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }

    (line, col)
}

/// Parse result that always returns an AST (possibly partial) plus diagnostics (CP-26).
///
/// Unlike `Result<SourceFile, Vec<ParseError>>`, this type never throws away
/// successfully-parsed items. Callers can inspect `diagnostics` to decide
/// whether to proceed with a partial AST.
#[derive(Debug)]
pub struct ParseResult {
    /// The parsed AST (may be partial if diagnostics are present).
    pub ast: SourceFile,
    /// Parse errors encountered during parsing (empty if fully successful).
    pub diagnostics: Vec<ParseError>,
}

impl ParseResult {
    /// Returns `true` if parsing completed without errors.
    pub fn is_ok(&self) -> bool {
        self.diagnostics.is_empty()
    }

    /// Convert to `Result`, discarding the partial AST if there were errors.
    ///
    /// Backward-compatible with the old `parse()` API.
    pub fn into_result(self) -> Result<SourceFile, Vec<ParseError>> {
        if self.diagnostics.is_empty() {
            Ok(self.ast)
        } else {
            Err(self.diagnostics)
        }
    }
}

/// Parse source text, always returning an AST (possibly partial) plus diagnostics.
///
/// Preferred over `parse()` when callers want to inspect partial results
/// (e.g., IDE/LSP, diagnostic rendering, incremental compilation).
pub fn parse_to_result(source: &str) -> ParseResult {
    let (tokens, lex_diagnostics) = Lexer::tokenize_with_diagnostics(source);
    if !lex_diagnostics.is_empty() {
        return ParseResult {
            ast: SourceFile {
                module_path: None,
                imports: Vec::new(),
                items: Vec::new(),
            },
            diagnostics: lex_diagnostics
                .into_iter()
                .map(|diagnostic| ParseError {
                    message: diagnostic.message,
                    span: diagnostic.span.unwrap_or(Span { start: 0, end: 0 }),
                })
                .collect(),
        };
    }
    let mut p = Parser::new(tokens);
    p.parse_source_file_partial()
}

pub fn parse(source: &str) -> Result<SourceFile, Vec<ParseError>> {
    let (tokens, lex_diagnostics) = Lexer::tokenize_with_diagnostics(source);
    if !lex_diagnostics.is_empty() {
        return Err(lex_diagnostics
            .into_iter()
            .map(|diagnostic| ParseError {
                message: diagnostic.message,
                span: diagnostic.span.unwrap_or(Span { start: 0, end: 0 }),
            })
            .collect());
    }
    let mut p = Parser::new(tokens);
    p.parse_source_file()
}

pub fn parse_with_file_diagnostics(
    file: &Path,
    source: &str,
) -> Result<SourceFile, Vec<Diagnostic>> {
    let (tokens, lex_diagnostics) = Lexer::tokenize_with_diagnostics(source);
    if !lex_diagnostics.is_empty() {
        return Err(lex_diagnostics
            .into_iter()
            .map(|diagnostic| {
                let diagnostic = diagnostic.with_file(file);
                if let Some(span) = diagnostic.span {
                    let (line, col) = byte_to_line_col(source, span.start);
                    diagnostic.with_line_col(line, col)
                } else {
                    diagnostic
                }
            })
            .collect());
    }

    let mut parser = Parser::new(tokens);
    parser.parse_source_file().map_err(|errors| {
        errors
            .iter()
            .map(|error| error.to_diagnostic(file, source))
            .collect()
    })
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
    allow_named_record_suffix: bool,
}

// Flatten an expression into a dotted path (for Call vs ServiceCall).
fn flatten_path(expr: &Expr) -> Option<Vec<String>> {
    match expr {
        Expr::Ident(name) => Some(vec![name.clone()]),
        Expr::FieldAccess(base, field) => {
            let mut path = flatten_path(base)?;
            path.push(field.clone());
            Some(path)
        }
        _ => None,
    }
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            pos: 0,
            errors: Vec::new(),
            allow_named_record_suffix: true,
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn peek2(&self) -> &Token {
        let next = (self.pos + 1).min(self.tokens.len() - 1);
        &self.tokens[next]
    }

    fn span(&self) -> Span {
        self.peek().span
    }

    fn at_eof(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn check(&self, kind: &TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(kind)
    }

    fn eat(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) -> Token {
        let t = self.peek().clone();
        if self.pos < self.tokens.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn expect(&mut self, kind: &TokenKind) -> Result<Token, ParseError> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            Err(self.err(format!(
                "expected {}, found {}",
                kind.desc(),
                self.peek().kind.desc()
            )))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        if let Some(s) = Self::token_kind_as_ident(&self.peek().kind) {
            self.advance();
            Ok(s)
        } else {
            Err(self.err(format!(
                "expected identifier, found {}",
                self.peek().kind.desc()
            )))
        }
    }

    fn token_kind_as_ident(kind: &TokenKind) -> Option<String> {
        let text = match kind {
            TokenKind::Ident(s) => return Some(s.clone()),
            TokenKind::Module => "module",
            TokenKind::Import => "import",
            TokenKind::Type => "type",
            TokenKind::Fn => "fn",
            TokenKind::Func => "func",
            TokenKind::Project => "project",
            TokenKind::Feature => "feature",
            TokenKind::Task => "task",
            TokenKind::Design => "design",
            TokenKind::Component => "component",
            TokenKind::Environment => "environment",
            TokenKind::Pattern => "pattern",
            TokenKind::Service => "service",
            TokenKind::Resource => "resource",
            TokenKind::Extern => "extern",
            TokenKind::Interface => "interface",
            TokenKind::Pipeline => "pipeline",
            TokenKind::Profile => "profile",
            TokenKind::Bind => "bind",
            TokenKind::Let => "let",
            TokenKind::Return => "return",
            TokenKind::Match => "match",
            TokenKind::If => "if",
            TokenKind::Else => "else",
            TokenKind::For => "for",
            TokenKind::In => "in",
            TokenKind::When => "when",
            TokenKind::After => "after",
            TokenKind::Node => "node",
            TokenKind::Uses => "uses",
            TokenKind::Provides => "provides",
            TokenKind::Acquire => "acquire",
            TokenKind::Release => "release",
            TokenKind::Capability => "capability",
            TokenKind::Operation => "operation",
            TokenKind::Input => "input",
            TokenKind::Output => "output",
            TokenKind::Stage => "stage",
            TokenKind::True => "true",
            TokenKind::False => "false",
            TokenKind::NoneLit => "none",
            TokenKind::As => "as",
            TokenKind::Implements => "implements",
            TokenKind::Parallel => "parallel",
            TokenKind::Config => "config",
            TokenKind::With => "with",
            TokenKind::SelfKw => "self",
            TokenKind::Test => "test",
            TokenKind::Fixture => "fixture",
            TokenKind::Mock => "mock",
            TokenKind::Expect => "expect",
            TokenKind::Contains => "contains",
            TokenKind::DataKw => "data",
            TokenKind::ParamKw => "param",
            TokenKind::From => "from",
            TokenKind::Where => "where",
            TokenKind::Transport => "transport",
            TokenKind::Response => "response",
            TokenKind::Idempotent => "idempotent",
            TokenKind::Readonly => "readonly",
            TokenKind::Hermetic => "hermetic",
            TokenKind::Contract => "contract",
            TokenKind::Tier => "tier",
            TokenKind::Skip => "skip",
            _ => return None,
        };
        Some(text.to_string())
    }

    /// Try to parse a multi-param lambda parameter list: `ident, ident, ...) =>`.
    ///
    /// Called after the opening `(` has been consumed. On success, consumes
    /// through the `=>` and returns the parameter names. On failure, returns
    /// `None` without advancing (caller must restore `self.pos`).
    fn try_parse_lambda_params(&mut self) -> Option<Vec<String>> {
        let mut params = Vec::new();
        // First param
        let first = Self::token_kind_as_ident(&self.peek().kind)?;
        params.push(first);
        self.advance();
        // Subsequent comma-separated params
        while self.check(&TokenKind::Comma) {
            self.advance(); // skip comma
                            // Allow trailing comma: `(a, b,) =>`
            if self.check(&TokenKind::RParen) {
                break;
            }
            let name = Self::token_kind_as_ident(&self.peek().kind)?;
            params.push(name);
            self.advance();
        }
        // Need at least 2 params (single-param uses `name => body` syntax)
        if params.len() < 2 {
            return None;
        }
        // Must see `) =>`
        if !self.check(&TokenKind::RParen) {
            return None;
        }
        self.advance(); // skip )
        if !self.check(&TokenKind::FatArrow) {
            return None;
        }
        self.advance(); // skip =>
        Some(params)
    }

    fn err(&self, message: String) -> ParseError {
        ParseError {
            message,
            span: self.span(),
        }
    }

    fn record_err(&mut self, e: ParseError) {
        self.errors.push(e);
    }

    fn sync_to_item(&mut self) {
        loop {
            if self.at_eof() {
                return;
            }
            match self.peek().kind {
                TokenKind::Type
                | TokenKind::Fn
                | TokenKind::Func
                | TokenKind::Pattern
                | TokenKind::Service
                | TokenKind::Resource
                | TokenKind::Interface
                | TokenKind::Pipeline
                | TokenKind::Profile
                | TokenKind::Test
                | TokenKind::Fixture
                | TokenKind::Project
                | TokenKind::Feature
                | TokenKind::Task
                | TokenKind::Design
                | TokenKind::Component
                | TokenKind::Environment
                | TokenKind::Extern => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn parse_service_config(&mut self) -> Result<ServiceConfig, ParseError> {
        let mut config = ServiceConfig::default();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                let name = self.expect_ident()?;
                if self.eat(&TokenKind::Colon) {
                    match name.as_str() {
                        "endpoint" => {
                            if let TokenKind::Str(s) = &self.peek().kind {
                                config.endpoint = Some(s.clone());
                                self.advance();
                            } else if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                                // Typed field syntax: `endpoint: String = "https://..."`
                                // Skip type identifier and `=`, then extract default string.
                                self.advance(); // skip type ident
                                if self.eat(&TokenKind::Eq) {
                                    if let TokenKind::Str(s) = &self.peek().kind {
                                        config.endpoint = Some(s.clone());
                                        self.advance();
                                    }
                                }
                            } else {
                                return Err(self.err(format!(
                                    "expected string literal for `endpoint`, found {}",
                                    self.peek().kind.desc()
                                )));
                            }
                        }
                        "auth" => {
                            if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                                let mut auth_value = self.expect_ident()?;
                                // Support `Header("x-api-key")` style auth specs
                                if self.eat(&TokenKind::LParen) {
                                    if let TokenKind::Str(s) = &self.peek().kind {
                                        auth_value = format!("{auth_value}({})", s);
                                        self.advance();
                                    }
                                    self.expect(&TokenKind::RParen)?;
                                }
                                config.auth = Some(auth_value);
                            } else {
                                return Err(self.err(format!(
                                    "expected identifier for `auth`, found {}",
                                    self.peek().kind.desc()
                                )));
                            }
                        }
                        "auth_input" => {
                            if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                                config.auth_input = Some(self.expect_ident()?);
                            } else {
                                return Err(self.err(format!(
                                    "expected identifier for `auth_input`, found {}",
                                    self.peek().kind.desc()
                                )));
                            }
                        }
                        "rate_limit" => {
                            // Inline: `rate_limit: { ... }` or block: `rate_limit { ... }`
                            self.expect(&TokenKind::LBrace)?;
                            let rate_limit = self.parse_rate_limit_block()?;
                            self.expect(&TokenKind::RBrace)?;
                            config.rate_limits.push(rate_limit);
                        }
                        "retry" => {
                            self.expect(&TokenKind::LBrace)?;
                            config.retry = Some(self.parse_retry_block()?);
                            self.expect(&TokenKind::RBrace)?;
                        }
                        "error_shape" => {
                            self.expect(&TokenKind::LBrace)?;
                            let error_shape = self.parse_error_shape_block()?;
                            self.expect(&TokenKind::RBrace)?;
                            config.error_shapes.push(error_shape);
                        }
                        "credential" => {
                            self.expect(&TokenKind::LBrace)?;
                            config.credential = Some(self.parse_credential_block()?);
                            self.expect(&TokenKind::RBrace)?;
                        }
                        "response_provider" => {
                            if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                                config.response_provider = Some(self.expect_ident()?);
                            } else {
                                return Err(self.err(format!(
                                    "expected identifier for `response_provider`, found {}",
                                    self.peek().kind.desc()
                                )));
                            }
                        }
                        other => {
                            // Provider-specific config fields (e.g., bucket,
                            // base_dir, model, project_id). Parse the type
                            // annotation and optional default value:
                            // `field: Type` or `field: Type = default`.
                            let field_name = other.to_string();
                            let ty = self.parse_type_expr()?;
                            let default = if self.eat(&TokenKind::Eq) {
                                Some(self.parse_expr(0)?)
                            } else {
                                None
                            };
                            config.extra.push(ProviderConfigField {
                                name: field_name,
                                ty,
                                default,
                            });
                        }
                    }
                }
            } else {
                return Err(self.err(format!(
                    "expected field name in config block, found {}",
                    self.peek().kind.desc()
                )));
            }
            self.eat(&TokenKind::Comma);
        }
        Ok(config)
    }

    // ── Transport block parsers (TL-11) ─────────────────────────────

    fn parse_rate_limit_block(&mut self) -> Result<RateLimitDef, ParseError> {
        let mut requests: Option<i64> = None;
        let mut per: Option<RateLimitUnit> = None;
        let mut scope: Option<String> = None;

        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                let field_name = self.expect_ident()?;
                self.expect(&TokenKind::Colon)?;
                match field_name.as_str() {
                    "requests" => {
                        if let TokenKind::Int(n) = &self.peek().kind {
                            let n = *n;
                            if n <= 0 {
                                return Err(self.err("rate_limit `requests` must be > 0".into()));
                            }
                            requests = Some(n);
                            self.advance();
                        } else {
                            return Err(self.err(format!(
                                "expected integer for `requests`, found {}",
                                self.peek().kind.desc()
                            )));
                        }
                    }
                    "per" => {
                        let unit = self.expect_ident()?;
                        per = Some(match unit.to_lowercase().as_str() {
                            "second" => RateLimitUnit::Second,
                            "minute" => RateLimitUnit::Minute,
                            "hour" => RateLimitUnit::Hour,
                            "day" => RateLimitUnit::Day,
                            _ => {
                                return Err(self.err(format!(
                                    "unknown rate_limit unit `{unit}` — \
                                     expected one of: second, minute, hour, day"
                                )));
                            }
                        });
                    }
                    "scope" => {
                        if let TokenKind::Str(s) = &self.peek().kind {
                            scope = Some(s.clone());
                            self.advance();
                        } else if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                            scope = Some(self.expect_ident()?);
                        } else {
                            return Err(self.err(format!(
                                "expected string or identifier for `scope`, found {}",
                                self.peek().kind.desc()
                            )));
                        }
                    }
                    other => {
                        return Err(self.err(format!(
                            "unknown field `{other}` in rate_limit block — \
                             expected one of: requests, per, scope"
                        )));
                    }
                }
            } else {
                return Err(self.err(format!(
                    "expected field name in rate_limit block, found {}",
                    self.peek().kind.desc()
                )));
            }
            self.eat(&TokenKind::Comma);
        }

        let requests = requests
            .ok_or_else(|| self.err("rate_limit block requires `requests` field".into()))?;
        let per = per.ok_or_else(|| self.err("rate_limit block requires `per` field".into()))?;

        Ok(RateLimitDef {
            requests,
            per,
            scope,
        })
    }

    fn parse_retry_block(&mut self) -> Result<RetryDef, ParseError> {
        let mut max_attempts: Option<i64> = None;
        let mut backoff = BackoffStrategy::Exponential;
        let mut base_delay_ms: Option<i64> = None;
        let mut max_delay_ms: Option<i64> = None;
        let mut retry_on: Vec<i64> = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                let field_name = self.expect_ident()?;
                self.expect(&TokenKind::Colon)?;
                match field_name.as_str() {
                    "max_attempts" => {
                        if let TokenKind::Int(n) = &self.peek().kind {
                            let n = *n;
                            if n < 1 {
                                return Err(self.err("retry `max_attempts` must be >= 1".into()));
                            }
                            max_attempts = Some(n);
                            self.advance();
                        } else {
                            return Err(self.err(format!(
                                "expected integer for `max_attempts`, found {}",
                                self.peek().kind.desc()
                            )));
                        }
                    }
                    "backoff" => {
                        let strategy = self.expect_ident()?;
                        backoff = match strategy.to_lowercase().as_str() {
                            "constant" => BackoffStrategy::Constant,
                            "linear" => BackoffStrategy::Linear,
                            "exponential" => BackoffStrategy::Exponential,
                            _ => {
                                return Err(self.err(format!(
                                    "unknown backoff strategy `{strategy}` — \
                                     expected one of: constant, linear, exponential"
                                )));
                            }
                        };
                    }
                    "base_delay_ms" => {
                        if let TokenKind::Int(n) = &self.peek().kind {
                            let n = *n;
                            if n < 0 {
                                return Err(self.err("retry `base_delay_ms` must be >= 0".into()));
                            }
                            base_delay_ms = Some(n);
                            self.advance();
                        } else {
                            return Err(self.err(format!(
                                "expected integer for `base_delay_ms`, found {}",
                                self.peek().kind.desc()
                            )));
                        }
                    }
                    "max_delay_ms" => {
                        if let TokenKind::Int(n) = &self.peek().kind {
                            let n = *n;
                            if n < 0 {
                                return Err(self.err("retry `max_delay_ms` must be >= 0".into()));
                            }
                            max_delay_ms = Some(n);
                            self.advance();
                        } else {
                            return Err(self.err(format!(
                                "expected integer for `max_delay_ms`, found {}",
                                self.peek().kind.desc()
                            )));
                        }
                    }
                    "retry_on" => {
                        self.expect(&TokenKind::LBracket)?;
                        while !self.check(&TokenKind::RBracket) && !self.at_eof() {
                            if let TokenKind::Int(n) = &self.peek().kind {
                                retry_on.push(*n);
                                self.advance();
                            } else {
                                return Err(self.err(format!(
                                    "expected integer status code in retry_on list, found {}",
                                    self.peek().kind.desc()
                                )));
                            }
                            self.eat(&TokenKind::Comma);
                        }
                        self.expect(&TokenKind::RBracket)?;
                    }
                    other => {
                        return Err(self.err(format!(
                            "unknown field `{other}` in retry block — \
                             expected one of: max_attempts, backoff, base_delay_ms, \
                             max_delay_ms, retry_on"
                        )));
                    }
                }
            } else {
                return Err(self.err(format!(
                    "expected field name in retry block, found {}",
                    self.peek().kind.desc()
                )));
            }
            self.eat(&TokenKind::Comma);
        }

        let max_attempts = max_attempts
            .ok_or_else(|| self.err("retry block requires `max_attempts` field".into()))?;

        Ok(RetryDef {
            max_attempts,
            backoff,
            base_delay_ms,
            max_delay_ms,
            retry_on,
        })
    }

    fn parse_error_shape_block(&mut self) -> Result<ErrorShapeDef, ParseError> {
        let mut status: Option<String> = None;
        let mut error_type_path: Option<String> = None;
        let mut message_path: Option<String> = None;
        let mut retryable = false;

        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                let field_name = self.expect_ident()?;
                self.expect(&TokenKind::Colon)?;
                match field_name.as_str() {
                    "status" => {
                        if let TokenKind::Int(n) = &self.peek().kind {
                            status = Some(n.to_string());
                            self.advance();
                        } else if let TokenKind::Str(s) = &self.peek().kind {
                            status = Some(s.clone());
                            self.advance();
                        } else {
                            return Err(self.err(format!(
                                "expected integer or string for `status`, found {}",
                                self.peek().kind.desc()
                            )));
                        }
                    }
                    "error_type_path" => {
                        if let TokenKind::Str(s) = &self.peek().kind {
                            error_type_path = Some(s.clone());
                            self.advance();
                        } else {
                            return Err(self.err(format!(
                                "expected string for `error_type_path`, found {}",
                                self.peek().kind.desc()
                            )));
                        }
                    }
                    "message_path" => {
                        if let TokenKind::Str(s) = &self.peek().kind {
                            message_path = Some(s.clone());
                            self.advance();
                        } else {
                            return Err(self.err(format!(
                                "expected string for `message_path`, found {}",
                                self.peek().kind.desc()
                            )));
                        }
                    }
                    "retryable" => {
                        if self.check(&TokenKind::True) {
                            retryable = true;
                            self.advance();
                        } else if self.check(&TokenKind::False) {
                            retryable = false;
                            self.advance();
                        } else {
                            return Err(self.err(format!(
                                "expected true or false for `retryable`, found {}",
                                self.peek().kind.desc()
                            )));
                        }
                    }
                    other => {
                        return Err(self.err(format!(
                            "unknown field `{other}` in error_shape block — \
                             expected one of: status, error_type_path, message_path, retryable"
                        )));
                    }
                }
            } else {
                return Err(self.err(format!(
                    "expected field name in error_shape block, found {}",
                    self.peek().kind.desc()
                )));
            }
            self.eat(&TokenKind::Comma);
        }

        let status =
            status.ok_or_else(|| self.err("error_shape block requires `status` field".into()))?;

        Ok(ErrorShapeDef {
            status,
            error_type_path,
            message_path,
            retryable,
        })
    }

    fn parse_credential_block(&mut self) -> Result<CredentialDef, ParseError> {
        let mut credential_type: Option<String> = None;
        let mut header: Option<String> = None;
        let mut source: Option<String> = None;

        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                let field_name = self.expect_ident()?;
                self.expect(&TokenKind::Colon)?;
                match field_name.as_str() {
                    "type" => {
                        credential_type = Some(self.expect_ident()?);
                    }
                    "header" => {
                        if let TokenKind::Str(s) = &self.peek().kind {
                            header = Some(s.clone());
                            self.advance();
                        } else {
                            return Err(self.err(format!(
                                "expected string for `header`, found {}",
                                self.peek().kind.desc()
                            )));
                        }
                    }
                    "source" => {
                        if let TokenKind::Str(s) = &self.peek().kind {
                            source = Some(s.clone());
                            self.advance();
                        } else {
                            return Err(self.err(format!(
                                "expected string for `source`, found {}",
                                self.peek().kind.desc()
                            )));
                        }
                    }
                    other => {
                        return Err(self.err(format!(
                            "unknown field `{other}` in credential block — \
                             expected one of: type, header, source"
                        )));
                    }
                }
            } else {
                return Err(self.err(format!(
                    "expected field name in credential block, found {}",
                    self.peek().kind.desc()
                )));
            }
            self.eat(&TokenKind::Comma);
        }

        let credential_type = credential_type
            .ok_or_else(|| self.err("credential block requires `type` field".into()))?;

        Ok(CredentialDef {
            credential_type,
            header,
            source,
        })
    }

    fn parse_transport_binding(&mut self) -> Result<TransportBinding, ParseError> {
        self.expect(&TokenKind::Transport)?;
        let kind = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let binding = match kind.as_str() {
            "rest" => {
                let mut method = String::new();
                let mut path = String::new();
                let mut body = None;
                let mut headers = None;
                while !self.check(&TokenKind::RBrace) && !self.at_eof() {
                    if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                        let field_name = self.expect_ident()?;
                        if self.eat(&TokenKind::Colon) {
                            match field_name.as_str() {
                                "method" => {
                                    method = self.expect_ident()?;
                                }
                                "path" => {
                                    if let TokenKind::Str(s) = &self.peek().kind {
                                        path = s.clone();
                                        self.advance();
                                    } else {
                                        path = self.expect_ident()?;
                                    }
                                }
                                "body" => {
                                    body = Some(self.parse_expr(0)?);
                                }
                                "headers" => {
                                    headers = Some(self.parse_expr(0)?);
                                }
                                other => {
                                    return Err(self.err(format!(
                                        "unknown REST transport field `{other}`: expected `method`, `path`, `body`, or `headers`"
                                    )));
                                }
                            }
                        }
                    } else {
                        self.advance();
                    }
                    self.eat(&TokenKind::Comma);
                }
                TransportBinding::Rest {
                    method,
                    path,
                    body,
                    headers,
                }
            }
            "shell" => {
                let mut argv = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.at_eof() {
                    if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                        let field_name = self.expect_ident()?;
                        if self.eat(&TokenKind::Colon) && field_name == "argv" {
                            if self.eat(&TokenKind::LBracket) {
                                while !self.check(&TokenKind::RBracket) && !self.at_eof() {
                                    argv.push(self.parse_expr(0)?);
                                    self.eat(&TokenKind::Comma);
                                }
                                self.expect(&TokenKind::RBracket)?;
                            }
                        } else if field_name != "argv" {
                            return Err(self.err(format!(
                                "unknown shell transport field `{field_name}`: expected `argv`"
                            )));
                        }
                    } else {
                        self.advance();
                    }
                    self.eat(&TokenKind::Comma);
                }
                TransportBinding::Shell { argv }
            }
            "file" => {
                let mut op = String::new();
                let mut fpath = String::new();
                while !self.check(&TokenKind::RBrace) && !self.at_eof() {
                    if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                        let field_name = self.expect_ident()?;
                        if self.eat(&TokenKind::Colon) {
                            match field_name.as_str() {
                                "op" => {
                                    op = self.expect_ident()?;
                                }
                                "path" => {
                                    if let TokenKind::Str(s) = &self.peek().kind {
                                        fpath = s.clone();
                                        self.advance();
                                    } else {
                                        fpath = self.expect_ident()?;
                                    }
                                }
                                other => {
                                    return Err(self.err(format!(
                                        "unknown file transport field `{other}`: expected `op` or `path`"
                                    )));
                                }
                            }
                        }
                    } else {
                        self.advance();
                    }
                    self.eat(&TokenKind::Comma);
                }
                TransportBinding::File { op, path: fpath }
            }
            "local" => {
                self.consume_brace_block_contents()?;
                return Ok(TransportBinding::Local);
            }
            other => {
                return Err(self.err(format!(
                    "unknown transport type `{other}`: expected `rest`, `shell`, `file`, or `local`"
                )));
            }
        };
        self.expect(&TokenKind::RBrace)?;
        Ok(binding)
    }

    /// Parse a response contract block: `response { STATUS => TYPE, ... }`
    ///
    /// Syntax:
    /// ```text
    /// response {
    ///     200 => SuccessType
    ///     201 => CreatedType "Created resource"
    ///     4xx => ClientError
    ///     5xx => ServerError
    /// }
    /// ```
    fn parse_response_block(&mut self) -> Result<Vec<ResponseEntry>, ParseError> {
        self.expect(&TokenKind::Response)?;
        self.expect(&TokenKind::LBrace)?;

        let mut entries = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            // Parse status pattern (200, 2xx, 4xx, 5xx, etc.)
            let status = self.parse_status_pattern()?;

            // Expect =>
            self.expect(&TokenKind::FatArrow)?;

            // Parse response type
            let response_type = self.parse_type_expr()?;

            // Optional description string
            let description = if let TokenKind::Str(s) = &self.peek().kind {
                let desc = s.clone();
                self.advance();
                Some(desc)
            } else {
                None
            };

            entries.push(ResponseEntry {
                status,
                response_type,
                description,
            });

            // Optional comma separator
            self.eat(&TokenKind::Comma);
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(entries)
    }

    /// Parse a status pattern: exact code (200, 404) or wildcard (2xx, 4xx, 5xx).
    ///
    /// Note: Wildcard patterns like "4xx" are lexed as Int(4) + Ident("xx"),
    /// so we need to handle this two-token sequence.
    fn parse_status_pattern(&mut self) -> Result<StatusPattern, ParseError> {
        // Check if we have an Int followed by "xx" (wildcard pattern)
        if let TokenKind::Int(n) = &self.peek().kind {
            let code = *n;
            self.advance();

            // Check if followed by "xx" suffix for wildcard patterns
            if let TokenKind::Ident(suffix) = &self.peek().kind {
                if suffix == "xx" {
                    self.advance();
                    return match code {
                        2 => Ok(StatusPattern::Success2xx),
                        3 => Ok(StatusPattern::Redirect3xx),
                        4 => Ok(StatusPattern::ClientError4xx),
                        5 => Ok(StatusPattern::ServerError5xx),
                        _ => Err(ParseError {
                            message: format!("Invalid wildcard pattern: {}xx", code),
                            span: self.peek().span,
                        }),
                    };
                }
            }

            // No "xx" suffix - it's an exact status code
            return Ok(StatusPattern::Exact(code as u16));
        }

        Err(ParseError {
            message: format!(
                "Expected status code (200, 404) or pattern (2xx, 4xx, 5xx), found {:?}",
                self.peek().kind
            ),
            span: self.peek().span,
        })
    }

    /// Parse an `exit` block for shell operations:
    /// ```text
    /// exit {
    ///     0 => Success
    ///     1 => GeneralError "Command failed"
    ///     nonzero => Error
    /// }
    /// ```
    fn parse_exit_block(&mut self) -> Result<Vec<ExitEntry>, ParseError> {
        self.expect(&TokenKind::Exit)?;
        self.expect(&TokenKind::LBrace)?;

        let mut entries = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            // Parse exit code (0, 1, 2, ...) or "nonzero"
            let code = self.parse_exit_code()?;

            // Expect =>
            self.expect(&TokenKind::FatArrow)?;

            // Parse output type
            let output_type = self.parse_type_expr()?;

            // Optional description string
            let description = if let TokenKind::Str(s) = &self.peek().kind {
                let desc = s.clone();
                self.advance();
                Some(desc)
            } else {
                None
            };

            entries.push(ExitEntry {
                code,
                output_type,
                description,
            });

            // Optional comma separator
            self.eat(&TokenKind::Comma);
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(entries)
    }

    /// Parse an exit code: exact value (0, 1, 2) or "nonzero" wildcard.
    fn parse_exit_code(&mut self) -> Result<ExitCode, ParseError> {
        // Check for "nonzero" identifier
        if let TokenKind::Ident(name) = &self.peek().kind {
            if name == "nonzero" {
                self.advance();
                return Ok(ExitCode::NonZero);
            }
        }

        // Check for integer exit code
        if let TokenKind::Int(n) = &self.peek().kind {
            let code = *n as i32;
            self.advance();
            return Ok(ExitCode::Exact(code));
        }

        Err(ParseError {
            message: format!(
                "Expected exit code (0, 1, 2, ...) or 'nonzero', found {:?}",
                self.peek().kind
            ),
            span: self.peek().span,
        })
    }

    /// Parse a `mock_response` block for operation test data:
    /// ```text
    /// mock_response {
    ///     200 => { "id": "123", "name": "test" }
    ///     401 => { "error": "unauthorized" }
    /// }
    /// ```
    fn parse_mock_response_block(&mut self) -> Result<Vec<MockResponseDef>, ParseError> {
        // Consume the "mock_response" identifier
        self.advance();
        self.expect(&TokenKind::LBrace)?;

        let mut entries = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            // Parse status code (integer only — no wildcards for mock responses)
            let status = match &self.peek().kind {
                TokenKind::Int(n) => {
                    let code = *n as u16;
                    self.advance();
                    code
                }
                _ => {
                    return Err(self.err(format!(
                        "expected HTTP status code (200, 401, etc.) in mock_response, found {}",
                        self.peek().kind.desc()
                    )));
                }
            };

            // Expect =>
            self.expect(&TokenKind::FatArrow)?;

            // Parse body expression (record literal, string, etc.)
            let body = self.parse_expr(0)?;

            // Optional description string after body
            let description = if let TokenKind::Str(s) = &self.peek().kind {
                let desc = s.clone();
                self.advance();
                Some(desc)
            } else {
                None
            };

            entries.push(MockResponseDef {
                status,
                body,
                description,
            });

            // Optional comma separator
            self.eat(&TokenKind::Comma);
        }

        self.expect(&TokenKind::RBrace)?;
        Ok(entries)
    }

    fn end_span(&self, start: Span) -> Span {
        let end = self.tokens[self.pos.saturating_sub(1)].span;
        Span {
            start: start.start,
            end: end.end,
        }
    }

    fn parse_dotted_ident(&mut self) -> Result<String, ParseError> {
        let mut name = self.expect_ident()?;
        while self.eat(&TokenKind::Dot) {
            name.push('.');
            name.push_str(&self.expect_ident()?);
        }
        Ok(name)
    }

    fn parse_type_name_ref(&mut self) -> Result<String, ParseError> {
        let mut name = self.parse_dotted_ident()?;
        if self.eat(&TokenKind::Lt) {
            name.push('<');
            let mut depth = 1usize;
            while depth > 0 && !self.at_eof() {
                let tok = self.advance();
                match tok.kind {
                    TokenKind::Lt => depth += 1,
                    TokenKind::Gt => depth -= 1,
                    _ => {}
                }
                name.push_str(&Self::token_text(&tok.kind));
            }
        }
        Ok(name)
    }

    // ── top-level ──────────────────────────────────────────────────

    fn parse_source_file(&mut self) -> Result<SourceFile, Vec<ParseError>> {
        let module_path = if self.check(&TokenKind::Module) {
            match self.parse_module_decl() {
                Ok(mp) => Some(mp),
                Err(e) => {
                    self.record_err(e);
                    self.sync_to_item();
                    None
                }
            }
        } else {
            None
        };

        let mut imports = Vec::new();
        while self.check(&TokenKind::Import) {
            match self.parse_import() {
                Ok(imp) => imports.push(imp),
                Err(e) => {
                    self.record_err(e);
                    self.sync_to_item();
                }
            }
        }

        let mut items = Vec::new();
        while !self.at_eof() {
            if self.at_eof() {
                break;
            }
            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(e) => {
                    self.record_err(e);
                    self.sync_to_item();
                }
            }
        }

        if self.errors.is_empty() {
            Ok(SourceFile {
                module_path,
                imports,
                items,
            })
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// Parse a source file, always returning the (possibly partial) AST plus diagnostics.
    fn parse_source_file_partial(&mut self) -> ParseResult {
        let module_path = if self.check(&TokenKind::Module) {
            match self.parse_module_decl() {
                Ok(mp) => Some(mp),
                Err(e) => {
                    self.record_err(e);
                    self.sync_to_item();
                    None
                }
            }
        } else {
            None
        };

        let mut imports = Vec::new();
        while self.check(&TokenKind::Import) {
            match self.parse_import() {
                Ok(imp) => imports.push(imp),
                Err(e) => {
                    self.record_err(e);
                    self.sync_to_item();
                }
            }
        }

        let mut items = Vec::new();
        while !self.at_eof() {
            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(e) => {
                    self.record_err(e);
                    self.sync_to_item();
                }
            }
        }

        ParseResult {
            ast: SourceFile {
                module_path,
                imports,
                items,
            },
            diagnostics: std::mem::take(&mut self.errors),
        }
    }

    fn parse_module_decl(&mut self) -> Result<Spanned<ModulePath>, ParseError> {
        let start = self.span();
        self.expect(&TokenKind::Module)?;
        let segments = self
            .parse_dotted_ident()?
            .split('.')
            .map(|s| s.to_string())
            .collect();
        Ok(Spanned {
            node: ModulePath { segments },
            span: self.end_span(start),
        })
    }

    fn parse_import(&mut self) -> Result<Spanned<Import>, ParseError> {
        let start = self.span();
        self.expect(&TokenKind::Import)?;
        let segments = self
            .parse_dotted_ident()?
            .split('.')
            .map(|s| s.to_string())
            .collect();
        let path = ModulePath { segments };
        let mut bindings = None;
        let mut alias = None;
        if self.eat(&TokenKind::LBrace) {
            let mut bs = Vec::new();
            if !self.check(&TokenKind::RBrace) {
                bs.push(self.parse_dotted_ident()?);
                while self.eat(&TokenKind::Comma) {
                    bs.push(self.parse_dotted_ident()?);
                }
            }
            self.expect(&TokenKind::RBrace)?;
            bindings = Some(bs);
        } else if self.eat(&TokenKind::As) {
            alias = Some(self.expect_ident()?);
        }
        Ok(Spanned {
            node: Import {
                path,
                bindings,
                alias,
            },
            span: self.end_span(start),
        })
    }

    fn parse_item(&mut self) -> Result<Spanned<Item>, ParseError> {
        let start = self.span();
        let item = match &self.peek().kind {
            TokenKind::Type => Item::TypeDef(self.parse_type_def()?),
            TokenKind::Fn => Item::FnDef(self.parse_fn_def()?),
            TokenKind::Func => Item::FuncDef(self.parse_func_def()?),
            TokenKind::Pattern => Item::PatternDef(self.parse_pattern_def()?),
            TokenKind::Service => Item::ServiceDef(Box::new(self.parse_service_def()?)),
            TokenKind::Resource => Item::ResourceDef(self.parse_resource_def()?),
            TokenKind::Interface => Item::InterfaceDef(self.parse_interface_def()?),
            TokenKind::Pipeline => Item::PipelineDef(self.parse_pipeline_def()?),
            TokenKind::Profile => Item::ProfileDef(self.parse_profile_def()?),
            TokenKind::Test => Item::TestDef(self.parse_test_def()?),
            TokenKind::Fixture => Item::FixtureDef(self.parse_fixture_def()?),
            TokenKind::Project => Item::ProjectDef(self.parse_project_def()?),
            TokenKind::Feature => Item::FeatureDef(self.parse_feature_def()?),
            TokenKind::Task => Item::TaskDef(self.parse_task_def()?),
            TokenKind::Design => Item::DesignDef(self.parse_design_def()?),
            TokenKind::Component => Item::ComponentDef(self.parse_component_def()?),
            TokenKind::Environment => Item::EnvironmentDef(self.parse_environment_def()?),
            TokenKind::ParamKw => Item::ParamDecl(self.parse_param_decl()?),
            TokenKind::DataKw => Item::DataDef(self.parse_data_def()?),
            TokenKind::Extern => self.parse_extern_decl()?,
            _ => {
                return Err(self.err(format!(
                    "expected item declaration, found {}",
                    self.peek().kind.desc()
                )))
            }
        };
        Ok(Spanned {
            node: item,
            span: self.end_span(start),
        })
    }

    // ── param declarations ─────────────────────────────────────────

    fn parse_param_decl(&mut self) -> Result<ParamDecl, ParseError> {
        self.expect(&TokenKind::ParamKw)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type_expr()?;
        let default = if self.eat(&TokenKind::Eq) {
            Some(self.parse_expr(0)?)
        } else {
            None
        };
        Ok(ParamDecl { name, ty, default })
    }

    // ── data declarations ──────────────────────────────────────────

    fn parse_data_def(&mut self) -> Result<DataDef, ParseError> {
        self.expect(&TokenKind::DataKw)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type_expr()?;
        self.expect(&TokenKind::Eq)?;
        let value = self.parse_expr(0)?;
        Ok(DataDef { name, ty, value })
    }

    // ── extern declarations ────────────────────────────────────────

    fn parse_extern_decl(&mut self) -> Result<Item, ParseError> {
        self.expect(&TokenKind::Extern)?;
        match &self.peek().kind {
            TokenKind::Func => Err(self.err(
                "`extern func` is no longer supported — use func parameters instead".to_string(),
            )),
            TokenKind::Ident(s) if s == "asset" => {
                Ok(Item::ExternAssetDecl(self.parse_extern_asset_decl()?))
            }
            _ => Err(self.err("expected `asset` after `extern`".to_string())),
        }
    }

    fn parse_extern_asset_decl(&mut self) -> Result<ExternAssetDecl, ParseError> {
        self.advance(); // consume "asset" identifier
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type_expr()?;
        Ok(ExternAssetDecl { name, ty })
    }

    // ── type definitions ───────────────────────────────────────────

    fn parse_type_def(&mut self) -> Result<TypeDef, ParseError> {
        self.expect(&TokenKind::Type)?;
        let name = self.expect_ident()?;
        let params = self.parse_optional_type_params()?;

        if self.eat(&TokenKind::LBrace) {
            let fields = self.parse_field_list_until_rbrace()?;
            self.expect(&TokenKind::RBrace)?;
            return Ok(TypeDef {
                name,
                params,
                body: TypeBody::Record(fields),
            });
        }

        if self.eat(&TokenKind::Eq) {
            if self.at_eof() {
                return Err(self.err("unexpected EOF after =".into()));
            }
            let first_name = self.expect_ident()?;
            if self.check(&TokenKind::LBrace) || self.check(&TokenKind::Pipe) {
                let first = self.parse_variant_fields(first_name)?;
                let mut variants = vec![first];
                while self.eat(&TokenKind::Pipe) {
                    let vn = self.expect_ident()?;
                    variants.push(self.parse_variant_fields(vn)?);
                }
                return Ok(TypeDef {
                    name,
                    params,
                    body: TypeBody::Sum(variants),
                });
            }
            let ty = self.finish_type_expr(first_name)?;
            return Ok(TypeDef {
                name,
                params,
                body: TypeBody::Alias(ty),
            });
        }

        Err(self.err("expected { or = after type name".into()))
    }

    fn parse_variant_fields(&mut self, name: String) -> Result<Variant, ParseError> {
        let fields = if self.eat(&TokenKind::LBrace) {
            let f = self.parse_field_list_until_rbrace()?;
            self.expect(&TokenKind::RBrace)?;
            f
        } else {
            Vec::new()
        };
        Ok(Variant { name, fields })
    }

    fn parse_optional_type_params(&mut self) -> Result<Vec<String>, ParseError> {
        if !self.eat(&TokenKind::Lt) {
            return Ok(Vec::new());
        }
        let mut params = Vec::new();
        if !self.check(&TokenKind::Gt) {
            params.push(self.expect_ident()?);
            // optional bounds `: Bound + Bound`
            if self.eat(&TokenKind::Colon) {
                if self.eat(&TokenKind::Arrow) {
                    let output_type = self.expect_ident()?;
                    if !params.contains(&output_type) {
                        params.push(output_type);
                    }
                }
                self.skip_type_bound();
            }
            while self.eat(&TokenKind::Comma) {
                params.push(self.expect_ident()?);
                if self.eat(&TokenKind::Colon) {
                    if self.eat(&TokenKind::Arrow) {
                        let output_type = self.expect_ident()?;
                        if !params.contains(&output_type) {
                            params.push(output_type);
                        }
                    }
                    self.skip_type_bound();
                }
            }
        }
        self.expect(&TokenKind::Gt)?;
        Ok(params)
    }

    fn skip_type_bound(&mut self) {
        let mut depth = 0usize;
        while !self.at_eof() {
            match self.peek().kind {
                TokenKind::Lt | TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => {
                    depth += 1;
                    self.advance();
                }
                TokenKind::Gt | TokenKind::RParen | TokenKind::RBracket | TokenKind::RBrace => {
                    if depth == 0 {
                        return;
                    }
                    depth -= 1;
                    self.advance();
                }
                TokenKind::Comma if depth == 0 => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    // ── type expressions ───────────────────────────────────────────

    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParseError> {
        if self.eat(&TokenKind::LBrace) {
            let fields = self.parse_field_list_until_rbrace()?;
            self.expect(&TokenKind::RBrace)?;
            return Ok(TypeExpr::Record(fields));
        }
        let name = self.parse_dotted_ident()?;
        self.finish_type_expr(name)
    }

    fn finish_type_expr(&mut self, name: String) -> Result<TypeExpr, ParseError> {
        let mut ty = if self.eat(&TokenKind::Lt) {
            let mut args = vec![self.parse_type_expr()?];
            while self.eat(&TokenKind::Comma) {
                args.push(self.parse_type_expr()?);
            }
            self.expect(&TokenKind::Gt)?;
            TypeExpr::Generic(name, args)
        } else if self.check(&TokenKind::LParen)
            || self.check(&TokenKind::Arrow)
            || self.check(&TokenKind::Colon)
            || self.check(&TokenKind::Plus)
        {
            let suffix = self.consume_type_suffix();
            TypeExpr::Named(format!("{name}{suffix}"))
        } else {
            TypeExpr::Named(name)
        };

        if self.eat(&TokenKind::Question) {
            ty = TypeExpr::Optional(Box::new(ty));
        }

        // Parse optional `where` clause for typed refinements
        if self.check(&TokenKind::Where) {
            self.advance();
            let mut refinements = Vec::new();
            refinements.push(self.parse_refinement()?);
            while self.eat(&TokenKind::Comma) {
                if self.check(&TokenKind::RBrace) || self.check(&TokenKind::RParen) || self.at_eof()
                {
                    break;
                }
                refinements.push(self.parse_refinement()?);
            }
            ty = TypeExpr::Refined(Box::new(ty), refinements);
        }

        Ok(ty)
    }

    fn parse_refinement(&mut self) -> Result<Refinement, ParseError> {
        let name = self.expect_ident()?;
        match name.as_str() {
            "pattern" => {
                self.expect(&TokenKind::LParen)?;
                let s = match &self.peek().kind {
                    TokenKind::Str(v) => {
                        let r = v.clone();
                        self.advance();
                        r
                    }
                    _ => return Err(self.err("expected string for pattern".into())),
                };
                self.expect(&TokenKind::RParen)?;
                Ok(Refinement::Pattern(s))
            }
            "range" => {
                self.expect(&TokenKind::LParen)?;
                let mut min = None;
                let mut max = None;
                if !self.check(&TokenKind::RParen) {
                    // Check for named args (min: N, max: M)
                    if Self::token_kind_as_ident(&self.peek().kind).is_some()
                        && self.peek2().kind == TokenKind::Colon
                    {
                        while !self.check(&TokenKind::RParen) && !self.at_eof() {
                            let key = self.expect_ident()?;
                            self.expect(&TokenKind::Colon)?;
                            let val = self.parse_expr(0)?;
                            match key.as_str() {
                                "min" => min = Some(val),
                                "max" => max = Some(val),
                                _ => {}
                            }
                            if !self.eat(&TokenKind::Comma) {
                                break;
                            }
                        }
                    } else {
                        // Positional: range(min, max)
                        min = Some(self.parse_expr(0)?);
                        if self.eat(&TokenKind::Comma) {
                            max = Some(self.parse_expr(0)?);
                        }
                    }
                }
                self.expect(&TokenKind::RParen)?;
                Ok(Refinement::Range { min, max })
            }
            "brand" => {
                self.expect(&TokenKind::LParen)?;
                let s = match &self.peek().kind {
                    TokenKind::Str(v) => {
                        let r = v.clone();
                        self.advance();
                        r
                    }
                    _ => return Err(self.err("expected string for brand".into())),
                };
                self.expect(&TokenKind::RParen)?;
                Ok(Refinement::Brand(s))
            }
            "non_empty" => Ok(Refinement::NonEmpty),
            "content" => {
                self.expect(&TokenKind::LParen)?;
                let enc = self.expect_ident()?;
                self.expect(&TokenKind::RParen)?;
                Ok(Refinement::Content(enc))
            }
            "format" => {
                self.expect(&TokenKind::LParen)?;
                let fmt = self.expect_ident()?;
                self.expect(&TokenKind::RParen)?;
                Ok(Refinement::Format(fmt))
            }
            "raw_body" => Ok(Refinement::RawBody),
            "width" => {
                self.expect(&TokenKind::LParen)?;
                let val = self.parse_expr(0)?;
                self.expect(&TokenKind::RParen)?;
                Ok(Refinement::Width(val))
            }
            "length" => {
                self.expect(&TokenKind::LParen)?;
                let val = self.parse_expr(0)?;
                self.expect(&TokenKind::RParen)?;
                Ok(Refinement::Length(val))
            }
            "signed" => {
                if self.check(&TokenKind::LParen) {
                    self.advance();
                    let repr = match &self.peek().kind {
                        TokenKind::Str(v) => {
                            let r = v.clone();
                            self.advance();
                            r
                        }
                        _ => {
                            return Err(self.err("expected string for signed representation".into()))
                        }
                    };
                    self.expect(&TokenKind::RParen)?;
                    Ok(Refinement::Signed(Some(repr)))
                } else {
                    Ok(Refinement::Signed(None))
                }
            }
            "unsigned" => Ok(Refinement::Unsigned),
            "arithmetic" => Ok(Refinement::Arithmetic),
            "domain" => {
                self.expect(&TokenKind::LParen)?;
                let dom = match &self.peek().kind {
                    TokenKind::Str(v) => {
                        let r = v.clone();
                        self.advance();
                        r
                    }
                    _ => return Err(self.err("expected string for domain".into())),
                };
                self.expect(&TokenKind::RParen)?;
                Ok(Refinement::Domain(dom))
            }
            "retry" => Err(self.err(
                "@retry is not supported — retry policy should be modeled via \
                 transport middleware (see Lane 5: TL-3)"
                    .into(),
            )),
            "error_map" => Err(self.err(
                "@error_map is not supported — error mapping should use \
                 response {} blocks (see Lane 6: SL-6)"
                    .into(),
            )),
            "requires" => Err(self.err(
                "@requires is not supported — use `uses` declarations for \
                 resource/capability requirements"
                    .into(),
            )),
            "file_types" => {
                self.expect(&TokenKind::LParen)?;
                let mut exts = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.at_eof() {
                    if let TokenKind::Str(s) = &self.peek().kind {
                        exts.push(s.clone());
                        self.advance();
                    } else {
                        return Err(self.err("expected string literal for file_types".into()));
                    }
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RParen)?;
                Ok(Refinement::FileTypes(exts))
            }
            other => {
                // Generic predicate — consume optional parens, preserving arguments
                let mut pred = other.to_string();
                if self.check(&TokenKind::LParen) {
                    pred.push('(');
                    self.advance();
                    let mut depth = 1usize;
                    while depth > 0 && !self.at_eof() {
                        match &self.peek().kind {
                            TokenKind::LParen => {
                                depth += 1;
                                pred.push('(');
                                self.advance();
                            }
                            TokenKind::RParen => {
                                depth -= 1;
                                if depth > 0 {
                                    pred.push(')');
                                    self.advance();
                                }
                            }
                            TokenKind::Str(s) => {
                                pred.push('"');
                                pred.push_str(s);
                                pred.push('"');
                                self.advance();
                            }
                            TokenKind::Int(i) => {
                                pred.push_str(&i.to_string());
                                self.advance();
                            }
                            TokenKind::Float(f) => {
                                pred.push_str(&f.to_string());
                                self.advance();
                            }
                            TokenKind::Ident(id) => {
                                pred.push_str(id);
                                self.advance();
                            }
                            TokenKind::Comma => {
                                pred.push_str(", ");
                                self.advance();
                            }
                            _ => {
                                self.advance();
                            }
                        }
                    }
                    pred.push(')');
                    self.expect(&TokenKind::RParen)?;
                }
                Ok(Refinement::Predicate(pred))
            }
        }
    }

    fn consume_type_suffix(&mut self) -> String {
        let mut out = String::new();
        let mut paren_depth = 0usize;
        let mut angle_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;

        while !self.at_eof() {
            let stop = paren_depth == 0
                && angle_depth == 0
                && bracket_depth == 0
                && brace_depth == 0
                && (self.check(&TokenKind::Comma)
                    || self.check(&TokenKind::RParen)
                    || self.check(&TokenKind::RBrace)
                    || self.check(&TokenKind::LBrace)
                    || self.check(&TokenKind::LBracket)
                    || self.check(&TokenKind::Eq)
                    || self.check(&TokenKind::Pipe)
                    || self.check(&TokenKind::Uses)
                    || self.check(&TokenKind::Provides));
            if stop {
                break;
            }

            let tok = self.advance();
            match tok.kind {
                TokenKind::LParen => paren_depth += 1,
                TokenKind::RParen => paren_depth = paren_depth.saturating_sub(1),
                TokenKind::Lt => angle_depth += 1,
                TokenKind::Gt => angle_depth = angle_depth.saturating_sub(1),
                TokenKind::LBracket => bracket_depth += 1,
                TokenKind::RBracket => bracket_depth = bracket_depth.saturating_sub(1),
                TokenKind::LBrace => brace_depth += 1,
                TokenKind::RBrace => brace_depth = brace_depth.saturating_sub(1),
                _ => {}
            }
            out.push_str(&Self::token_text(&tok.kind));
        }

        out
    }

    fn token_text(kind: &TokenKind) -> String {
        match kind {
            TokenKind::Ident(s)
            | TokenKind::Str(s)
            | TokenKind::StrBegin(s)
            | TokenKind::StrMid(s)
            | TokenKind::StrEnd(s) => s.clone(),
            TokenKind::Int(n) => n.to_string(),
            TokenKind::Float(f) => f.to_string(),
            _ => kind.desc().to_string(),
        }
    }

    // ── fn / func / pattern ────────────────────────────────────────

    fn parse_fn_def(&mut self) -> Result<FnDef, ParseError> {
        self.expect(&TokenKind::Fn)?;
        let name = self.expect_ident()?;
        let type_params = self.parse_optional_type_params()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::Arrow)?;
        let return_type = self.parse_return_type_expr()?;
        let body = if self.eat(&TokenKind::LBrace) {
            let body = self.parse_fn_body()?;
            self.expect(&TokenKind::RBrace)?;
            body
        } else {
            FnBody { stmts: Vec::new() }
        };
        Ok(FnDef {
            name,
            type_params,
            params,
            return_type,
            body,
        })
    }

    fn parse_func_def(&mut self) -> Result<FuncDef, ParseError> {
        self.expect(&TokenKind::Func)?;
        let name = self.expect_ident()?;
        let type_params = self.parse_optional_type_params()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::Arrow)?;
        let outputs = self.parse_output_fields()?;
        let (uses, provides) = self.parse_uses_provides()?;
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_func_body()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(FuncDef {
            name,
            type_params,
            params,
            outputs,
            uses,
            provides,
            body,
        })
    }

    fn parse_pattern_def(&mut self) -> Result<PatternDef, ParseError> {
        self.expect(&TokenKind::Pattern)?;
        let name = self.expect_ident()?;
        let type_params = self.parse_optional_type_params()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::Arrow)?;
        let outputs = self.parse_output_fields()?;
        let (uses, provides) = self.parse_uses_provides()?;
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_func_body()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(PatternDef {
            name,
            type_params,
            params,
            outputs,
            uses,
            provides,
            body,
        })
    }

    fn parse_output_fields(&mut self) -> Result<Vec<Field>, ParseError> {
        if !self.eat(&TokenKind::LBrace) {
            if self.check(&TokenKind::Uses)
                || self.check(&TokenKind::Provides)
                || self.check(&TokenKind::LBrace)
            {
                return Ok(Vec::new());
            }
            let ty = self.parse_type_expr()?;
            return Ok(vec![Field {
                name: "return".into(),
                ty,
                default: None,
                from_path: None,
            }]);
        }
        let fields = self.parse_field_list_until_rbrace()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(fields)
    }

    fn parse_uses_provides(
        &mut self,
    ) -> Result<(Vec<UsesClause>, Vec<ProvidesClause>), ParseError> {
        let mut uses = Vec::new();
        let mut provides = Vec::new();
        loop {
            if self.eat(&TokenKind::Uses) {
                uses.push(self.parse_uses_clause()?);
            } else if self.eat(&TokenKind::Provides) {
                provides.push(self.parse_provides_clause()?);
            } else {
                break;
            }
        }
        Ok((uses, provides))
    }

    fn parse_uses_clause(&mut self) -> Result<UsesClause, ParseError> {
        let binding = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let resource_type = self.parse_type_expr()?;
        let config = if self.eat(&TokenKind::LParen) {
            let mut kvs = Vec::new();
            if !self.check(&TokenKind::RParen) {
                loop {
                    let k = self.expect_ident()?;
                    self.expect(&TokenKind::Colon)?;
                    let v = self.parse_expr(0)?;
                    kvs.push((k, v));
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(&TokenKind::RParen)?;
            Some(kvs)
        } else {
            None
        };
        Ok(UsesClause {
            binding,
            resource_type,
            config,
        })
    }

    fn parse_provides_clause(&mut self) -> Result<ProvidesClause, ParseError> {
        let binding = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let resource_type = self.parse_type_expr()?;
        Ok(ProvidesClause {
            binding,
            resource_type,
        })
    }

    // ── service / resource / interface / pipeline ───────────────────

    fn parse_service_def(&mut self) -> Result<ServiceDef, ParseError> {
        self.expect(&TokenKind::Service)?;
        let name = self.parse_dotted_ident()?;
        let implements = if self.eat(&TokenKind::Colon) || self.eat(&TokenKind::Implements) {
            Some(self.parse_type_name_ref()?)
        } else {
            None
        };
        self.expect(&TokenKind::LBrace)?;
        let mut operations = Vec::new();
        let mut config = ServiceConfig::default();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if self.check(&TokenKind::Operation) {
                operations.push(self.parse_operation_def()?);
            } else if self.check(&TokenKind::Config) {
                self.advance();
                self.expect(&TokenKind::LBrace)?;
                config = self.parse_service_config()?;
                self.expect(&TokenKind::RBrace)?;
            } else {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(ServiceDef {
            name,
            implements,
            operations,
            config,
        })
    }

    fn parse_operation_def(&mut self) -> Result<OperationDef, ParseError> {
        self.expect(&TokenKind::Operation)?;
        let name = self.expect_ident()?;
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut idempotent = false;
        let mut readonly = false;
        let mut transport: Option<TransportBinding> = None;
        let mut response: Vec<ResponseEntry> = Vec::new();
        let mut exit: Vec<ExitEntry> = Vec::new();
        let mut mock_responses: Vec<MockResponseDef> = Vec::new();
        let mut output_parsing: Option<String> = None;

        if self.eat(&TokenKind::LParen) {
            inputs = self.parse_field_list_until_rparen()?;
            self.expect(&TokenKind::RParen)?;
        }
        if self.eat(&TokenKind::Arrow) && self.eat(&TokenKind::LBrace) {
            outputs = self.parse_field_list_until_rbrace()?;
            self.expect(&TokenKind::RBrace)?;
        }
        loop {
            if self.check(&TokenKind::Idempotent) {
                self.advance();
                idempotent = true;
            } else if self.check(&TokenKind::Readonly) {
                self.advance();
                readonly = true;
            } else if self.check(&TokenKind::Hermetic) {
                // Silently accept — hermeticity is structurally derived from transport.
                // Keeping as a no-op avoids breaking existing .dag files.
                self.advance();
            } else {
                break;
            }
        }
        if self.eat(&TokenKind::LBrace) {
            while !self.check(&TokenKind::RBrace) && !self.at_eof() {
                if self.check(&TokenKind::Input) {
                    self.advance();
                    self.expect(&TokenKind::LBrace)?;
                    inputs = self.parse_field_list_until_rbrace()?;
                    self.expect(&TokenKind::RBrace)?;
                } else if self.check(&TokenKind::Output) {
                    self.advance();
                    self.expect(&TokenKind::LBrace)?;
                    outputs = self.parse_field_list_until_rbrace()?;
                    self.expect(&TokenKind::RBrace)?;
                } else if self.check(&TokenKind::Idempotent) {
                    self.advance();
                    idempotent = true;
                } else if self.check(&TokenKind::Readonly) {
                    self.advance();
                    readonly = true;
                } else if self.check(&TokenKind::Hermetic) {
                    // Silently accept — hermeticity is structurally derived from transport.
                    self.advance();
                } else if self.check(&TokenKind::Transport) {
                    transport = Some(self.parse_transport_binding()?);
                } else if self.check(&TokenKind::Response) {
                    response = self.parse_response_block()?;
                } else if self.check(&TokenKind::Exit) {
                    exit = self.parse_exit_block()?;
                } else if matches!(&self.peek().kind, TokenKind::Ident(s) if s == "mock_response") {
                    mock_responses = self.parse_mock_response_block()?;
                } else if matches!(&self.peek().kind, TokenKind::Ident(s) if s == "output_parsing")
                {
                    self.advance(); // consume "output_parsing"
                    self.expect(&TokenKind::Colon)?;
                    output_parsing = Some(self.expect_ident()?);
                } else if self.eat(&TokenKind::Uses) {
                    // Consume `uses` clause — resource requirements are structurally
                    // derived from the operation's transport, not stored in the AST.
                    let _uses = self.parse_uses_clause()?;
                } else if self.eat(&TokenKind::Provides) {
                    let _provides = self.parse_provides_clause()?;
                } else if self.check(&TokenKind::RBrace) {
                    break;
                } else {
                    let tok_desc = format!("{:?}", self.peek().kind);
                    return Err(self.err(format!("unexpected token in operation body: {tok_desc}")));
                }
            }
            self.expect(&TokenKind::RBrace)?;
        }
        Ok(OperationDef {
            name,
            inputs,
            outputs,
            idempotent,
            readonly,
            transport,
            response,
            exit,
            mock_responses,
            output_parsing,
        })
    }

    fn parse_resource_def(&mut self) -> Result<ResourceDef, ParseError> {
        self.expect(&TokenKind::Resource)?;
        let name = self.expect_ident()?;
        let implements = if self.eat(&TokenKind::Colon) || self.eat(&TokenKind::Implements) {
            Some(self.parse_type_name_ref()?)
        } else {
            None
        };
        self.expect(&TokenKind::LBrace)?;
        let mut properties = Vec::new();
        let mut config = Vec::new();
        let mut acquire = None;
        let mut release = None;
        let mut capabilities = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            match &self.peek().kind {
                TokenKind::Acquire => {
                    self.advance();
                    self.expect(&TokenKind::LBrace)?;
                    let body = self.parse_func_body()?;
                    self.expect(&TokenKind::RBrace)?;
                    acquire = Some(body);
                }
                TokenKind::Release => {
                    self.advance();
                    self.expect(&TokenKind::LBrace)?;
                    let body = self.parse_func_body()?;
                    self.expect(&TokenKind::RBrace)?;
                    release = Some(body);
                }
                TokenKind::Capability => {
                    capabilities.push(self.parse_capability_def()?);
                }
                TokenKind::Config => {
                    self.advance();
                    self.expect(&TokenKind::LBrace)?;
                    config = self.parse_field_list_until_rbrace()?;
                    self.expect(&TokenKind::RBrace)?;
                }
                TokenKind::Ident(_) => {
                    let k = self.expect_ident()?;
                    self.expect(&TokenKind::Colon)?;
                    let v = self.parse_expr(0)?;
                    properties.push((k, v));
                }
                _ => {
                    self.advance();
                }
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(ResourceDef {
            name,
            implements,
            properties,
            config,
            acquire,
            release,
            capabilities,
        })
    }

    fn parse_capability_def(&mut self) -> Result<CapabilityDef, ParseError> {
        self.expect(&TokenKind::Capability)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut idempotent = false;
        let mut readonly = false;
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if self.check(&TokenKind::Input) {
                self.advance();
                self.expect(&TokenKind::LBrace)?;
                inputs = self.parse_field_list_until_rbrace()?;
                self.expect(&TokenKind::RBrace)?;
            } else if self.check(&TokenKind::Output) {
                self.advance();
                self.expect(&TokenKind::LBrace)?;
                outputs = self.parse_field_list_until_rbrace()?;
                self.expect(&TokenKind::RBrace)?;
            } else if self.check(&TokenKind::Idempotent) {
                self.advance();
                idempotent = true;
            } else if self.check(&TokenKind::Readonly) {
                self.advance();
                readonly = true;
            } else {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(CapabilityDef {
            name,
            inputs,
            outputs,
            idempotent,
            readonly,
        })
    }

    fn parse_interface_def(&mut self) -> Result<InterfaceDef, ParseError> {
        self.expect(&TokenKind::Interface)?;
        let name = self.expect_ident()?;
        let type_params = self.parse_optional_type_params()?;
        if self.eat(&TokenKind::Colon) {
            let _ = self.expect_ident();
        }
        self.expect(&TokenKind::LBrace)?;
        let mut capabilities = Vec::new();
        let mut typed_contracts: Vec<ContractDef> = Vec::new();
        let mut type_defs = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if self.check(&TokenKind::Operation) {
                let op = self.parse_operation_def()?;
                capabilities.push(CapabilityDef {
                    name: op.name,
                    inputs: op.inputs,
                    outputs: op.outputs,
                    idempotent: op.idempotent,
                    readonly: op.readonly,
                });
            } else if self.check(&TokenKind::Capability) {
                capabilities.push(self.parse_interface_capability_signature()?);
            } else if self.check(&TokenKind::Fn) {
                capabilities.push(self.parse_interface_fn()?);
            } else if self.check(&TokenKind::Type) {
                if let Ok(td) = self.parse_type_def() {
                    type_defs.push(td);
                }
            } else if self.check(&TokenKind::Contract) {
                // Typed contract: `contract <text>`
                self.advance();
                // Consume the rest of the contract text up to the next keyword or annotation
                let mut text_parts = Vec::new();
                while !self.at_eof()
                    && !self.check(&TokenKind::RBrace)
                    && !self.check(&TokenKind::Capability)
                    && !self.check(&TokenKind::Operation)
                    && !self.check(&TokenKind::Fn)
                    && !self.check(&TokenKind::Type)
                    && !self.check(&TokenKind::Contract)
                {
                    text_parts.push(format!("{:?}", self.peek().kind));
                    self.advance();
                }
                typed_contracts.push(ContractDef {
                    text: text_parts.join(" "),
                });
            } else {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(InterfaceDef {
            name,
            type_params,
            capabilities,
            contracts: typed_contracts,
            type_defs,
        })
    }

    fn parse_interface_fn(&mut self) -> Result<CapabilityDef, ParseError> {
        self.expect(&TokenKind::Fn)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LParen)?;
        let inputs_raw = self.parse_param_list()?;
        self.expect(&TokenKind::RParen)?;
        let inputs: Vec<Field> = inputs_raw
            .into_iter()
            .map(|p| Field {
                name: p.name,
                ty: p.ty,
                default: p.default,
                from_path: None,
            })
            .collect();
        let mut outputs = Vec::new();
        if self.eat(&TokenKind::Arrow) {
            let ret = self.parse_return_type_expr()?;
            outputs.push(Field {
                name: "return".into(),
                ty: ret,
                default: None,
                from_path: None,
            });
        }
        Ok(CapabilityDef {
            name,
            inputs,
            outputs,
            idempotent: false,
            readonly: false,
        })
    }

    fn parse_interface_capability_signature(&mut self) -> Result<CapabilityDef, ParseError> {
        self.expect(&TokenKind::Capability)?;
        let name = self.expect_ident()?;
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut idempotent = false;
        let mut readonly = false;

        if self.eat(&TokenKind::LParen) {
            inputs = self.parse_field_list_until_rparen()?;
            self.expect(&TokenKind::RParen)?;
        }
        if self.eat(&TokenKind::Arrow) && self.eat(&TokenKind::LBrace) {
            outputs = self.parse_field_list_until_rbrace()?;
            self.expect(&TokenKind::RBrace)?;
        }
        loop {
            if self.check(&TokenKind::Idempotent) {
                self.advance();
                idempotent = true;
            } else if self.check(&TokenKind::Readonly) {
                self.advance();
                readonly = true;
            } else {
                break;
            }
        }

        if self.eat(&TokenKind::LBrace) {
            while !self.check(&TokenKind::RBrace) && !self.at_eof() {
                if self.check(&TokenKind::Input) {
                    self.advance();
                    self.expect(&TokenKind::LBrace)?;
                    inputs = self.parse_field_list_until_rbrace()?;
                    self.expect(&TokenKind::RBrace)?;
                } else if self.check(&TokenKind::Output) {
                    self.advance();
                    self.expect(&TokenKind::LBrace)?;
                    outputs = self.parse_field_list_until_rbrace()?;
                    self.expect(&TokenKind::RBrace)?;
                } else if self.check(&TokenKind::Idempotent) {
                    self.advance();
                    idempotent = true;
                } else if self.check(&TokenKind::Readonly) {
                    self.advance();
                    readonly = true;
                } else {
                    self.advance();
                }
            }
            self.expect(&TokenKind::RBrace)?;
        }

        Ok(CapabilityDef {
            name,
            inputs,
            outputs,
            idempotent,
            readonly,
        })
    }

    fn parse_return_type_expr(&mut self) -> Result<TypeExpr, ParseError> {
        if self.eat(&TokenKind::LBrace) {
            let fields = self.parse_field_list_until_rbrace()?;
            self.expect(&TokenKind::RBrace)?;
            Ok(TypeExpr::Record(fields))
        } else {
            self.parse_type_expr()
        }
    }

    fn parse_pipeline_def(&mut self) -> Result<PipelineDef, ParseError> {
        self.expect(&TokenKind::Pipeline)?;
        let name = self.expect_ident()?;
        let (uses, _provides) = self.parse_uses_provides()?;
        self.expect(&TokenKind::LBrace)?;
        let mut stages = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if self.check(&TokenKind::Stage) {
                stages.push(self.parse_stage_def()?);
            } else {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(PipelineDef { name, uses, stages })
    }

    fn parse_profile_def(&mut self) -> Result<ProfileDef, ParseError> {
        self.expect(&TokenKind::Profile)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let mut binds = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if self.check(&TokenKind::Bind) {
                binds.push(self.parse_profile_bind()?);
            } else {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(ProfileDef { name, binds })
    }

    fn parse_profile_bind(&mut self) -> Result<ProfileBind, ParseError> {
        self.expect(&TokenKind::Bind)?;
        let interface_type = self.parse_dotted_ident()?;
        self.expect(&TokenKind::Arrow)?;
        let implementation_type = self.parse_dotted_ident()?;
        let mut config_entries = Vec::new();
        if self.eat(&TokenKind::LBrace) {
            while !self.check(&TokenKind::RBrace) && !self.at_eof() {
                let key = self.expect_ident()?;
                self.expect(&TokenKind::Colon)?;
                let value = self.parse_expr(0)?;
                config_entries.push((key, value));
                self.eat(&TokenKind::Comma);
            }
            self.expect(&TokenKind::RBrace)?;
        }
        Ok(ProfileBind {
            interface_type,
            implementation_type,
            config_entries,
        })
    }

    fn parse_stage_def(&mut self) -> Result<StageDef, ParseError> {
        self.expect(&TokenKind::Stage)?;
        let name = self.expect_ident()?;
        let mut after = Vec::new();
        let mut when = None;
        if self.eat(&TokenKind::LBracket) {
            while !self.check(&TokenKind::RBracket) && !self.at_eof() {
                if self.eat(&TokenKind::After) {
                    after.push(self.expect_ident()?);
                } else if self.check(&TokenKind::When) {
                    self.advance();
                    if let Ok(expr) = self.parse_expr(0) {
                        when = match when.take() {
                            None => Some(expr),
                            Some(existing) => Some(Expr::BinOp(
                                Box::new(existing),
                                crate::ast::BinOp::And,
                                Box::new(expr),
                            )),
                        };
                    }
                } else if self.eat(&TokenKind::Comma) {
                    continue;
                } else {
                    self.advance();
                }
            }
            self.expect(&TokenKind::RBracket)?;
        }
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_func_body()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(StageDef {
            name,
            body,
            after,
            when,
        })
    }
    // ── Profile & Infra Blocks ─────────────────────────────────────────

    fn parse_property_list_until_rbrace(&mut self) -> Result<Vec<(String, Expr)>, ParseError> {
        let mut props = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            let name = self.expect_ident()?;
            self.expect(&TokenKind::Eq)?;
            let expr = self.parse_expr(0)?;
            props.push((name, expr));
            // Comma is optional; newline-separated properties are valid.
            self.eat(&TokenKind::Comma);
        }
        Ok(props)
    }

    fn parse_project_def(&mut self) -> Result<ProjectDef, ParseError> {
        self.expect(&TokenKind::Project)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let properties = self.parse_property_list_until_rbrace()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(ProjectDef { name, properties })
    }

    fn parse_feature_def(&mut self) -> Result<FeatureDef, ParseError> {
        self.expect(&TokenKind::Feature)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let properties = self.parse_property_list_until_rbrace()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(FeatureDef { name, properties })
    }

    fn parse_task_def(&mut self) -> Result<TaskDef, ParseError> {
        self.expect(&TokenKind::Task)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let properties = self.parse_property_list_until_rbrace()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(TaskDef { name, properties })
    }

    fn parse_design_def(&mut self) -> Result<DesignDef, ParseError> {
        self.expect(&TokenKind::Design)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let properties = self.parse_property_list_until_rbrace()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(DesignDef { name, properties })
    }

    fn parse_component_def(&mut self) -> Result<ComponentDef, ParseError> {
        self.expect(&TokenKind::Component)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let properties = self.parse_property_list_until_rbrace()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(ComponentDef { name, properties })
    }

    fn parse_environment_def(&mut self) -> Result<EnvironmentDef, ParseError> {
        self.expect(&TokenKind::Environment)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let properties = self.parse_property_list_until_rbrace()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(EnvironmentDef { name, properties })
    }

    // ── test DSL ───────────────────────────────────────────────────

    /// Parse a fixture definition: `fixture <name> { mock* }`
    fn parse_fixture_def(&mut self) -> Result<FixtureDef, ParseError> {
        self.expect(&TokenKind::Fixture)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let mut mocks = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if self.check(&TokenKind::Mock) {
                mocks.push(self.parse_mock_decl()?);
            } else {
                return Err(self.err(format!(
                    "expected 'mock' inside fixture, found {}",
                    self.peek().kind.desc()
                )));
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(FixtureDef { name, mocks })
    }

    /// Parse a test definition:
    /// `test <name> [: <fixture>] { annotation* (let | mock | input | expect)* }`
    fn parse_test_def(&mut self) -> Result<TestDef, ParseError> {
        self.expect(&TokenKind::Test)?;
        let name = self.expect_ident()?;
        let fixture = if self.eat(&TokenKind::Colon) {
            Some(self.expect_ident()?)
        } else {
            None
        };
        self.expect(&TokenKind::LBrace)?;

        let mut tier = None;
        let mut hermetic = false;
        let mut skip = false;

        // Parse typed test metadata keywords before mock/input/expect
        loop {
            match &self.peek().kind {
                TokenKind::Tier => {
                    self.advance();
                    if self.eat(&TokenKind::Colon) {
                        if let Ok(t) = self.expect_ident() {
                            tier = Some(t);
                        }
                    }
                }
                TokenKind::Hermetic => {
                    self.advance();
                    hermetic = true;
                }
                TokenKind::Skip => {
                    self.advance();
                    skip = true;
                }
                _ => break,
            }
        }

        let mut lets = Vec::new();
        let mut mocks = Vec::new();
        let mut inputs = Vec::new();
        let mut expects = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            match &self.peek().kind {
                TokenKind::Let => lets.push(self.parse_test_let_decl()?),
                TokenKind::Mock => mocks.push(self.parse_mock_decl()?),
                TokenKind::Input => inputs.push(self.parse_input_decl()?),
                TokenKind::Expect => expects.push(self.parse_expect_stmt()?),
                _ => {
                    return Err(self.err(format!(
                        "expected 'let', 'mock', 'input', or 'expect' inside test, found {}",
                        self.peek().kind.desc()
                    )));
                }
            }
        }
        self.expect(&TokenKind::RBrace)?;

        Ok(TestDef {
            name,
            fixture,
            lets,
            mocks,
            inputs,
            expects,
            tier,
            hermetic,
            skip,
        })
    }

    /// Parse: `let <ident> = <expr>`
    fn parse_test_let_decl(&mut self) -> Result<LetDecl, ParseError> {
        self.expect(&TokenKind::Let)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Eq)?;
        let value = self.parse_expr(0)?;
        Ok(LetDecl { name, value })
    }

    /// Parse a mock/input target path.
    ///
    /// Local targets use `seg1/seg2/.../segN.port`. Qualified targets use
    /// `module.path::seg1/seg2/.../segN.port`.
    ///
    /// Segments before the last `.` are joined with `/` to form the node ID.
    /// The segment after the last `.` is the port name. If no `.` is present,
    /// the identifier is treated as a bare port name with an empty node path
    /// (broadcast-style input).
    fn parse_mock_target(&mut self) -> Result<(TestNodeRef, String), ParseError> {
        let checkpoint = self.pos;
        let first = self.expect_ident()?;
        let mut module_segments = vec![first];
        while self.eat(&TokenKind::Dot) {
            module_segments.push(self.expect_ident()?);
        }
        if self.eat(&TokenKind::DoubleColon) {
            let (node_segments, port) = self.parse_local_mock_target_tail()?;
            return Ok((
                TestNodeRef::qualified(module_segments.into(), node_segments),
                port,
            ));
        }

        self.pos = checkpoint;
        let (node_segments, port) = self.parse_local_mock_target_tail()?;
        Ok((TestNodeRef::local(node_segments), port))
    }

    fn parse_local_mock_target_tail(&mut self) -> Result<(Vec<String>, String), ParseError> {
        let first = self.expect_ident()?;

        // Collect path segments separated by `/`
        let mut segments = vec![first];
        while self.eat(&TokenKind::Slash) {
            segments.push(self.expect_ident()?);
        }

        // If no `.` follows, the single identifier is a bare port name
        if !self.check(&TokenKind::Dot) {
            let port = segments.pop().unwrap();
            return Ok((segments, port));
        }

        // The port is after the last `.`
        self.expect(&TokenKind::Dot)?;
        let port = self.expect_ident()?;

        // There may be more `.` separated path parts before the final port
        // e.g., `gist_upload/cloud_credential/gcp_wif_secret/parse_set_iam.ok`
        // In this case node = gist_upload/cloud_credential/gcp_wif_secret/parse_set_iam
        //                port = ok
        // But there could also be `gist_upload.execute.response`:
        //   node = gist_upload/execute, port = response
        // We use `.` for both sub-path separators and the final node/port separator.
        // Resolution: collect all dotted segments, the last one is the port.
        let mut dotted = vec![port];
        while self.eat(&TokenKind::Dot) {
            dotted.push(self.expect_ident()?);
        }

        let port = dotted.pop().unwrap();
        // Remaining dotted segments become additional path segments
        for seg in dotted {
            segments.push(seg);
        }

        Ok((segments, port))
    }

    /// Parse: `mock <target> -> <expr>`
    fn parse_mock_decl(&mut self) -> Result<MockDecl, ParseError> {
        self.expect(&TokenKind::Mock)?;
        let (node_ref, port) = self.parse_mock_target()?;
        self.expect(&TokenKind::Arrow)?;
        let value = self.parse_expr(0)?;
        Ok(MockDecl {
            node_ref,
            port,
            value,
        })
    }

    /// Parse: `input <target> -> <expr>`
    fn parse_input_decl(&mut self) -> Result<InputDecl, ParseError> {
        self.expect(&TokenKind::Input)?;
        let (node_ref, port) = self.parse_mock_target()?;
        self.expect(&TokenKind::Arrow)?;
        let value = self.parse_expr(0)?;
        Ok(InputDecl {
            node_ref,
            port,
            value,
        })
    }

    /// Parse the target of an expect assertion as a structured path.
    ///
    /// `result.port` is recognized as the terminal output. Everything else
    /// is parsed as a mock-style node reference (`node.port` or
    /// `module::node.port`).
    fn parse_expect_target(&mut self) -> Result<ExpectTarget, ParseError> {
        let checkpoint = self.pos;
        let first = self.expect_ident()?;

        if first == "result" {
            self.expect(&TokenKind::Dot)?;
            let port = self.expect_ident()?;
            return Ok(ExpectTarget::Result { port });
        }

        // Not `result` — backtrack and parse as a mock-style target.
        self.pos = checkpoint;
        let (node_ref, port) = self.parse_mock_target()?;
        Ok(ExpectTarget::Node { node_ref, port })
    }

    /// Parse: `expect <target> <comparison> <expr>`
    ///        `expect <target> contains <expr>`
    ///        `expect <target> is <TypeName>`
    ///        `expect <target>`
    fn parse_expect_stmt(&mut self) -> Result<ExpectStmt, ParseError> {
        self.expect(&TokenKind::Expect)?;
        let target = self.parse_expect_target()?;

        match &self.peek().kind {
            TokenKind::EqEq => {
                self.advance();
                let rhs = self.parse_expr(0)?;
                Ok(ExpectStmt::Eq(target, rhs))
            }
            TokenKind::Ne => {
                self.advance();
                let rhs = self.parse_expr(0)?;
                Ok(ExpectStmt::Ne(target, rhs))
            }
            TokenKind::Lt => {
                self.advance();
                let rhs = self.parse_expr(0)?;
                Ok(ExpectStmt::Lt(target, rhs))
            }
            TokenKind::Gt => {
                self.advance();
                let rhs = self.parse_expr(0)?;
                Ok(ExpectStmt::Gt(target, rhs))
            }
            TokenKind::Le => {
                self.advance();
                let rhs = self.parse_expr(0)?;
                Ok(ExpectStmt::Le(target, rhs))
            }
            TokenKind::Ge => {
                self.advance();
                let rhs = self.parse_expr(0)?;
                Ok(ExpectStmt::Ge(target, rhs))
            }
            TokenKind::Contains => {
                self.advance();
                let rhs = self.parse_expr(0)?;
                Ok(ExpectStmt::Contains(target, rhs))
            }
            // `is <TypeName>` — keyword `is` parsed as Ident("is")
            TokenKind::Ident(s) if s == "is" => {
                self.advance();
                let type_name = self.expect_ident()?;
                Ok(ExpectStmt::Is(target, type_name))
            }
            _ => {
                // Just a truthiness check
                Ok(ExpectStmt::Truthy(target))
            }
        }
    }

    fn consume_brace_block_contents(&mut self) -> Result<(), ParseError> {
        let mut depth = 1usize;
        while depth > 0 && !self.at_eof() {
            match self.peek().kind {
                TokenKind::LBrace => depth += 1,
                TokenKind::RBrace => depth = depth.saturating_sub(1),
                _ => {}
            }
            self.advance();
        }
        if depth == 0 {
            Ok(())
        } else {
            Err(self.err("unterminated block".into()))
        }
    }

    // ── fields / params ────────────────────────────────────────────

    fn parse_field_list_until_rbrace(&mut self) -> Result<Vec<Field>, ParseError> {
        let mut fields = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            fields.push(self.parse_field()?);
            self.eat(&TokenKind::Comma);
        }
        Ok(fields)
    }

    fn parse_field_list_until_rparen(&mut self) -> Result<Vec<Field>, ParseError> {
        let mut fields = Vec::new();
        if self.check(&TokenKind::RParen) {
            return Ok(fields);
        }
        fields.push(self.parse_field()?);
        while self.eat(&TokenKind::Comma) {
            if self.check(&TokenKind::RParen) {
                break;
            }
            fields.push(self.parse_field()?);
        }
        Ok(fields)
    }

    fn parse_field(&mut self) -> Result<Field, ParseError> {
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type_expr()?;
        let default = if self.eat(&TokenKind::Eq) {
            Some(self.parse_expr(0)?)
        } else {
            None
        };
        let from_path = if self.check(&TokenKind::From) {
            self.advance();
            if let TokenKind::Str(s) = &self.peek().kind {
                let path = s.clone();
                self.advance();
                Some(path)
            } else {
                None
            }
        } else {
            None
        };
        Ok(Field {
            name,
            ty,
            default,
            from_path,
        })
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        if self.check(&TokenKind::RParen) {
            return Ok(params);
        }
        if self.check(&TokenKind::SelfKw) {
            self.advance();
            params.push(Param {
                name: "self".into(),
                ty: TypeExpr::Named("Self".into()),
                default: None,
            });
            if !self.eat(&TokenKind::Comma) {
                return Ok(params);
            }
            if self.check(&TokenKind::RParen) {
                return Ok(params);
            }
        }
        params.push(self.parse_param()?);
        while self.eat(&TokenKind::Comma) {
            if self.check(&TokenKind::RParen) {
                break;
            }
            params.push(self.parse_param()?);
        }
        Ok(params)
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let name = self.expect_ident()?;
        self.expect(&TokenKind::Colon)?;
        let ty = self.parse_type_expr()?;
        let default = if self.eat(&TokenKind::Eq) {
            Some(self.parse_expr(0)?)
        } else {
            None
        };
        Ok(Param { name, ty, default })
    }

    // ── bodies / statements ────────────────────────────────────────

    fn parse_fn_body(&mut self) -> Result<FnBody, ParseError> {
        Ok(FnBody {
            stmts: self.parse_stmts()?,
        })
    }

    fn parse_func_body(&mut self) -> Result<FuncBody, ParseError> {
        Ok(FuncBody {
            stmts: self.parse_stmts()?,
        })
    }

    fn parse_stmts(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            match self.parse_stmt() {
                Ok(s) => stmts.push(s),
                Err(e) => {
                    self.record_err(e);
                    while !self.at_eof()
                        && !self.check(&TokenKind::RBrace)
                        && !self.check(&TokenKind::Let)
                        && !self.check(&TokenKind::Node)
                        && !self.check(&TokenKind::Return)
                    {
                        if Self::token_kind_as_ident(&self.peek().kind).is_some()
                            && (self.peek2().kind == TokenKind::Eq
                                || self.peek2().kind == TokenKind::Colon)
                        {
                            break;
                        }
                        self.advance();
                    }
                }
            }
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        if self.check(&TokenKind::Return) {
            return self.parse_return_stmt();
        }
        if self.check(&TokenKind::Let) {
            self.advance();
            let name = self.expect_ident()?;
            self.expect(&TokenKind::Eq)?;
            let expr = self.parse_expr(0)?;
            return Ok(Stmt::Let(name, expr));
        }
        if self.check(&TokenKind::Node) {
            return self.parse_node_stmt();
        }
        if self.check(&TokenKind::Parallel) {
            self.advance();
            self.expect(&TokenKind::LBrace)?;
            let inner = self.parse_stmts()?;
            self.expect(&TokenKind::RBrace)?;
            return Ok(Stmt::Expr(Expr::Record(
                None,
                inner
                    .into_iter()
                    .filter_map(|s| match s {
                        Stmt::Assign(n, e) | Stmt::Let(n, e) => Some((n, e)),
                        Stmt::Node(ns) => Some((ns.name, ns.expr)),
                        _ => None,
                    })
                    .collect(),
            )));
        }
        if self.looks_like_assignment_stmt() {
            let name = self.expect_ident()?;
            while self.eat(&TokenKind::LBracket) {
                while !self.check(&TokenKind::RBracket) && !self.at_eof() {
                    self.advance();
                }
                self.eat(&TokenKind::RBracket);
            }
            if self.eat(&TokenKind::Colon) {
                let _ = self.parse_type_expr();
            }
            self.expect(&TokenKind::Eq)?;
            let expr = self.parse_expr(0)?;
            return Ok(Stmt::Assign(name, expr));
        }
        let expr = self.parse_expr(0)?;
        Ok(Stmt::Expr(expr))
    }

    fn looks_like_assignment_stmt(&self) -> bool {
        if Self::token_kind_as_ident(&self.peek().kind).is_none() {
            return false;
        }
        let mut idx = self.pos + 1;
        while idx < self.tokens.len() {
            match self.tokens[idx].kind {
                TokenKind::LBracket => {
                    let mut depth = 1usize;
                    idx += 1;
                    while idx < self.tokens.len() && depth > 0 {
                        match self.tokens[idx].kind {
                            TokenKind::LBracket => depth += 1,
                            TokenKind::RBracket => depth -= 1,
                            _ => {}
                        }
                        idx += 1;
                    }
                }
                TokenKind::Eq | TokenKind::Colon => return true,
                _ => return false,
            }
        }
        false
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&TokenKind::Return)?;
        if self.eat(&TokenKind::LBrace) {
            let mut fields = Vec::new();
            while !self.check(&TokenKind::RBrace) && !self.at_eof() {
                let k = self.expect_ident()?;
                self.expect(&TokenKind::Colon)?;
                let v = self.parse_expr(0)?;
                fields.push((k, v));
                self.eat(&TokenKind::Comma);
            }
            self.expect(&TokenKind::RBrace)?;
            Ok(Stmt::Return(fields))
        } else {
            let expr = self.parse_expr(0)?;
            Ok(Stmt::Return(vec![("return".into(), expr)]))
        }
    }

    fn parse_node_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&TokenKind::Node)?;
        let name = self.expect_ident()?;
        let mut after = Vec::new();
        let mut when_guard = None;
        if self.eat(&TokenKind::LBracket) {
            while !self.check(&TokenKind::RBracket) && !self.at_eof() {
                if self.check(&TokenKind::After) {
                    self.advance(); // consume 'after'
                    let dep = self.expect_ident()?;
                    after.push(dep);
                } else if self.check(&TokenKind::When) {
                    self.advance(); // consume 'when'
                    when_guard = Some(self.parse_expr(0)?);
                } else if self.eat(&TokenKind::Comma) {
                    continue;
                } else {
                    self.advance(); // skip unknown tokens in guard
                }
            }
            self.eat(&TokenKind::RBracket);
        }
        if self.eat(&TokenKind::Colon) || self.eat(&TokenKind::Eq) {
            let expr = self.parse_expr(0)?;
            if self.eat(&TokenKind::Arrow) && self.eat(&TokenKind::LBrace) {
                while !self.check(&TokenKind::RBrace) && !self.at_eof() {
                    self.advance();
                }
                self.eat(&TokenKind::RBrace);
            }
            return Ok(Stmt::Node(NodeStmt {
                name,
                expr,
                after,
                when_guard,
            }));
        }
        Err(self.err("expected : or = after node name".into()))
    }

    // ── expression parser (Pratt) ──────────────────────────────────

    fn parse_expr(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_prefix()?;
        loop {
            if self.at_eof() {
                break;
            }
            if self.check(&TokenKind::Question) {
                return Err(self.err(
                    "ternary operator is not supported; did you mean an optional type?".into(),
                ));
            }
            if self.allow_named_record_suffix && self.check(&TokenKind::LBrace) {
                if 21u8 < min_bp {
                    break;
                }
                if let Expr::Ident(name) = lhs {
                    lhs = self.parse_named_record_expr(name)?;
                    continue;
                }
            }
            if self.check(&TokenKind::LParen) {
                if 21u8 < min_bp {
                    break;
                }
                lhs = self.parse_call_expr(lhs)?;
                continue;
            }
            if self.check(&TokenKind::LBracket) {
                if 18u8 < min_bp {
                    break;
                }
                lhs = self.parse_guard_after_suffix(lhs)?;
                continue;
            }
            if self.check(&TokenKind::With) {
                if 19u8 < min_bp {
                    break;
                }
                self.advance();
                self.expect(&TokenKind::LBrace)?;
                let update = self.parse_record_literal_fields()?;
                self.expect(&TokenKind::RBrace)?;
                lhs = Expr::Call(
                    "with".into(),
                    vec![(None, lhs), (None, Expr::Record(None, update))],
                );
                continue;
            }
            if self.check(&TokenKind::As) {
                if 19u8 < min_bp {
                    break;
                }
                self.advance();
                let tn = self.expect_ident()?;
                lhs = Expr::Call("as".into(), vec![(None, lhs), (None, Expr::Ident(tn))]);
                continue;
            }
            // Pipe chain: `a |> f(args)` desugars to `f(a, args)`,
            // `a |> f` desugars to `f(a)`.
            // Binding power 17/18: tighter than comparison (9-12) and arithmetic (13-16),
            // but looser than dot (19-20) and call (21).
            if self.check(&TokenKind::Pipe) && self.peek2().kind == TokenKind::Gt {
                let l_bp: u8 = 17;
                let r_bp: u8 = 18;
                if l_bp < min_bp {
                    break;
                }
                self.advance(); // consume |
                self.advance(); // consume >
                                // RHS: parse with high binding power so we get just the function call
                let rhs = self.parse_expr(r_bp)?;
                lhs = match rhs {
                    // `a |> f(b, c)` → `f(a, b, c)`
                    Expr::Call(name, mut args) => {
                        args.insert(0, (None, lhs));
                        Expr::Call(name, args)
                    }
                    // `a |> f` → `f(a)`
                    Expr::Ident(name) => Expr::Call(name, vec![(None, lhs)]),
                    other => {
                        return Err(self.err(format!(
                            "expected function call or identifier after |>, got {:?}",
                            other
                        )));
                    }
                };
                continue;
            }
            if let Some((l_bp, r_bp)) = self.infix_bp() {
                if l_bp < min_bp {
                    break;
                }
                let op_tok = self.advance();
                if op_tok.kind == TokenKind::Dot {
                    let field = self.expect_ident()?;
                    lhs = Expr::FieldAccess(Box::new(lhs), field);
                    continue;
                }
                let rhs = self.parse_expr(r_bp)?;
                lhs = self.make_binop(lhs, &op_tok.kind, rhs);
                continue;
            }
            break;
        }
        Ok(lhs)
    }

    fn parse_named_record_expr(&mut self, name: String) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::LBrace)?;
        let fields = self.parse_record_literal_fields()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(Expr::Record(Some(name), fields))
    }

    fn parse_record_literal_fields(&mut self) -> Result<Vec<(String, Expr)>, ParseError> {
        let mut fields = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if self.eat(&TokenKind::Dot) {
                // tolerate spread-ish syntax (`...`) in examples
                self.eat(&TokenKind::Dot);
                self.eat(&TokenKind::Dot);
                if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                    let _ = self.expect_ident()?;
                }
                self.eat(&TokenKind::Comma);
                continue;
            }
            if Self::token_kind_as_ident(&self.peek().kind).is_some()
                && self.peek2().kind == TokenKind::Colon
            {
                let k = self.expect_ident()?;
                self.expect(&TokenKind::Colon)?;
                let v = self.parse_expr(0)?;
                fields.push((k, v));
                self.eat(&TokenKind::Comma);
                continue;
            }
            // shorthand field syntax: `{ field_name }`
            if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                let k = self.expect_ident()?;
                fields.push((k.clone(), Expr::Ident(k)));
                self.eat(&TokenKind::Comma);
                continue;
            }
            self.advance();
        }
        Ok(fields)
    }

    fn parse_guard_after_suffix(&mut self, lhs: Expr) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::LBracket)?;
        if self.eat(&TokenKind::When) {
            let cond = self.parse_expr(0)?;
            self.expect(&TokenKind::RBracket)?;
            return Ok(Expr::Guarded(Box::new(lhs), Box::new(cond)));
        }

        let mut after = Vec::new();
        while !self.check(&TokenKind::RBracket) && !self.at_eof() {
            if self.eat(&TokenKind::After) {
                after.push(self.expect_ident()?);
            } else if self.eat(&TokenKind::Comma) {
                continue;
            } else {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBracket)?;
        if after.is_empty() {
            Ok(lhs)
        } else {
            Ok(Expr::After(Box::new(lhs), after))
        }
    }

    fn infix_bp(&self) -> Option<(u8, u8)> {
        match &self.peek().kind {
            TokenKind::NullCoalesce => Some((3, 4)),
            TokenKind::Or => Some((5, 6)),
            TokenKind::And => Some((7, 8)),
            TokenKind::EqEq | TokenKind::Ne => Some((9, 10)),
            TokenKind::Lt | TokenKind::Gt | TokenKind::Le | TokenKind::Ge => Some((11, 12)),
            TokenKind::Plus | TokenKind::Minus => Some((13, 14)),
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some((15, 16)),
            TokenKind::Dot => Some((19, 20)),
            _ => None,
        }
    }

    fn make_binop(&self, lhs: Expr, op: &TokenKind, rhs: Expr) -> Expr {
        let binop = match op {
            TokenKind::Plus => Some(BinOp::Add),
            TokenKind::Minus => Some(BinOp::Sub),
            TokenKind::Star => Some(BinOp::Mul),
            TokenKind::Slash => Some(BinOp::Div),
            TokenKind::Percent => Some(BinOp::Mod),
            TokenKind::EqEq => Some(BinOp::Eq),
            TokenKind::Ne => Some(BinOp::Ne),
            TokenKind::Lt => Some(BinOp::Lt),
            TokenKind::Gt => Some(BinOp::Gt),
            TokenKind::Le => Some(BinOp::Le),
            TokenKind::Ge => Some(BinOp::Ge),
            TokenKind::And => Some(BinOp::And),
            TokenKind::Or => Some(BinOp::Or),
            _ => None,
        };
        if let Some(bop) = binop {
            Expr::BinOp(Box::new(lhs), bop, Box::new(rhs))
        } else {
            match op {
                TokenKind::NullCoalesce => {
                    Expr::BinOp(Box::new(lhs), BinOp::NullCoalesce, Box::new(rhs))
                }
                _ => unreachable!("unhandled infix operator: {op:?}"),
            }
        }
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        match &self.peek().kind {
            TokenKind::Bang => {
                self.advance();
                let e = self.parse_expr(17)?;
                Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(e)))
            }
            TokenKind::Minus => {
                self.advance();
                let e = self.parse_expr(17)?;
                Ok(Expr::UnaryOp(UnaryOp::Neg, Box::new(e)))
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek().kind.clone() {
            TokenKind::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            TokenKind::NoneLit => {
                self.advance();
                Ok(Expr::Literal(Literal::None))
            }
            TokenKind::Int(n) => {
                self.advance();
                Ok(Expr::Literal(Literal::Int(n)))
            }
            TokenKind::Float(f) => {
                self.advance();
                Ok(Expr::Literal(Literal::Float(f)))
            }
            TokenKind::Str(s) => {
                self.advance();
                Ok(Expr::Literal(Literal::String(s)))
            }
            TokenKind::StrBegin(s) => self.parse_string_interp(s),
            kind if Self::token_kind_as_ident(&kind).is_some()
                && !matches!(
                    kind,
                    TokenKind::Match
                        | TokenKind::If
                        | TokenKind::For
                        | TokenKind::Return
                        | TokenKind::Fn
                ) =>
            {
                let Some(name) = Self::token_kind_as_ident(&kind) else {
                    return Err(self.err(format!(
                        "expected identifier, found {}",
                        self.peek().kind.desc()
                    )));
                };
                if self.peek2().kind == TokenKind::FatArrow {
                    self.advance();
                    self.advance();
                    let body = self.parse_lambda_body()?;
                    return Ok(Expr::Lambda(vec![name], Box::new(body)));
                }
                self.advance();
                Ok(Expr::Ident(name))
            }
            TokenKind::LParen => {
                self.advance();
                if self.check(&TokenKind::RParen) {
                    self.advance();
                    return Ok(Expr::Record(None, Vec::new()));
                }
                // Try multi-param lambda: (a, b) => body
                // Use save/restore backtracking with normal parser methods.
                let save_pos = self.pos;
                let save_errors = self.errors.len();
                let params = self.try_parse_lambda_params();
                if let Some(params) = params {
                    let body = self.parse_lambda_body()?;
                    Ok(Expr::Lambda(params, Box::new(body)))
                } else {
                    // Not a lambda — restore and parse as parenthesized expression
                    self.pos = save_pos;
                    self.errors.truncate(save_errors);
                    let expr = self.parse_expr(0)?;
                    self.expect(&TokenKind::RParen)?;
                    Ok(expr)
                }
            }
            TokenKind::LBracket => {
                self.advance();
                let mut items = Vec::new();
                while !self.check(&TokenKind::RBracket) && !self.at_eof() {
                    items.push(self.parse_expr(0)?);
                    self.eat(&TokenKind::Comma);
                }
                self.expect(&TokenKind::RBracket)?;
                Ok(Expr::List(items))
            }
            TokenKind::LBrace => {
                self.advance();
                if self.check(&TokenKind::RBrace) {
                    self.advance();
                    return Ok(Expr::Record(None, Vec::new()));
                }
                if self.starts_block_content() {
                    return self.consume_brace_block_expr();
                }
                if Self::token_kind_as_ident(&self.peek().kind).is_some()
                    && self.peek2().kind == TokenKind::Colon
                {
                    let fields = self.parse_record_literal_fields()?;
                    self.expect(&TokenKind::RBrace)?;
                    Ok(Expr::Record(None, fields))
                } else if self.brace_contains_top_level_colon() {
                    let entries = self.parse_map_literal_entries()?;
                    self.expect(&TokenKind::RBrace)?;
                    Ok(Expr::Map(entries))
                } else {
                    let fields = self.parse_record_literal_fields()?;
                    self.expect(&TokenKind::RBrace)?;
                    Ok(Expr::Record(None, fields))
                }
            }
            TokenKind::Match => self.parse_match_expr(),
            TokenKind::If => self.parse_if_expr(),
            TokenKind::For => self.parse_for_expr(),
            // fn(params) { body } lambda syntax
            TokenKind::Fn if self.peek2().kind == TokenKind::LParen => {
                self.advance(); // consume fn
                self.expect(&TokenKind::LParen)?;
                let mut params = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.at_eof() {
                    params.push(self.expect_ident()?);
                    self.eat(&TokenKind::Comma);
                }
                self.expect(&TokenKind::RParen)?;
                self.expect(&TokenKind::LBrace)?;
                let body = if self.starts_brace_block_expr() {
                    Expr::Block(self.parse_stmts()?)
                } else {
                    self.parse_expr(0)?
                };
                self.expect(&TokenKind::RBrace)?;
                Ok(Expr::Lambda(params, Box::new(body)))
            }
            TokenKind::Return => {
                self.advance();
                if self.eat(&TokenKind::LBrace) {
                    let mut fields = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.at_eof() {
                        let k = self.expect_ident()?;
                        self.expect(&TokenKind::Colon)?;
                        let v = self.parse_expr(0)?;
                        fields.push((k, v));
                        self.eat(&TokenKind::Comma);
                    }
                    self.expect(&TokenKind::RBrace)?;
                    Ok(Expr::Return(fields))
                } else {
                    let e = self.parse_expr(0)?;
                    Ok(Expr::Return(vec![("return".into(), e)]))
                }
            }
            TokenKind::Fn => {
                self.advance();
                self.expect(&TokenKind::LParen)?;
                let mut ps = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.at_eof() {
                    ps.push(self.parse_expr(0)?);
                    self.eat(&TokenKind::Comma);
                }
                self.expect(&TokenKind::RParen)?;
                if self.eat(&TokenKind::Arrow) {
                    let ret = self.parse_expr(0)?;
                    ps.push(ret);
                }
                Ok(Expr::Call(
                    "fn".into(),
                    ps.into_iter().map(|e| (None, e)).collect(),
                ))
            }
            _ => Err(self.err(format!(
                "expected expression, found {}",
                self.peek().kind.desc()
            ))),
        }
    }

    fn starts_brace_block_expr(&self) -> bool {
        if self.check(&TokenKind::Let)
            || self.check(&TokenKind::Return)
            || self.check(&TokenKind::Node)
            || self.check(&TokenKind::Parallel)
        {
            return true;
        }
        Self::token_kind_as_ident(&self.peek().kind).is_some()
            && (self.peek2().kind == TokenKind::Eq || self.peek2().kind == TokenKind::LBracket)
    }

    /// Check if content after `{` looks like a block rather than a record/map.
    /// Includes expression-starting keywords like match/if/for.
    fn starts_block_content(&self) -> bool {
        self.starts_brace_block_expr()
            || self.check(&TokenKind::Match)
            || self.check(&TokenKind::If)
            || self.check(&TokenKind::For)
    }

    /// Check if the current position looks like the start of a new match arm.
    /// Used to stop implicit block parsing in match arm bodies.
    /// Only returns true when we're confident this starts a new arm — requires
    /// the pattern to be followed by `=>` (possibly after destructuring).
    fn looks_like_match_arm_start(&self) -> bool {
        if let Some(name) = Self::token_kind_as_ident(&self.peek().kind) {
            if name.starts_with(|c: char| c.is_uppercase()) {
                // `Name =>` → definitely a new arm
                if self.peek2().kind == TokenKind::FatArrow {
                    return true;
                }
                // `Name { ... } =>` → scan for matching } then =>
                if self.peek2().kind == TokenKind::LBrace {
                    return self.scan_for_fat_arrow_after_braces();
                }
            }
        }
        // `_ =>` wildcard pattern
        if let TokenKind::Ident(ref s) = self.peek().kind {
            if s == "_" && self.peek2().kind == TokenKind::FatArrow {
                return true;
            }
        }
        false
    }

    /// Scan forward from current position to check if `{ ... } =>` follows.
    /// Used to distinguish match arm patterns from expressions.
    fn scan_for_fat_arrow_after_braces(&self) -> bool {
        let mut idx = self.pos + 2; // skip ident and {
        let mut depth = 1;
        while idx < self.tokens.len() && depth > 0 {
            match self.tokens[idx].kind {
                TokenKind::LBrace => depth += 1,
                TokenKind::RBrace => depth -= 1,
                _ => {}
            }
            idx += 1;
        }
        // After closing }, check for =>
        idx < self.tokens.len() && self.tokens[idx].kind == TokenKind::FatArrow
    }

    /// Parse a lambda body: either a single expression or an implicit block
    /// (multi-statement body delimited by `)` or `}` from enclosing context).
    fn parse_lambda_body(&mut self) -> Result<Expr, ParseError> {
        if self.starts_brace_block_expr() {
            // Multi-statement lambda body: `x => let a = 1\n a + 1`
            // Statements until `)` or `}`
            let mut stmts = Vec::new();
            while !self.check(&TokenKind::RParen)
                && !self.check(&TokenKind::RBrace)
                && !self.at_eof()
            {
                match self.parse_stmt() {
                    Ok(s) => stmts.push(s),
                    Err(e) => {
                        self.record_err(e);
                        break;
                    }
                }
            }
            Ok(Expr::Block(stmts))
        } else {
            self.parse_expr(0)
        }
    }

    /// Like `starts_brace_block_expr` but checks the token after the opening `{`.
    /// Used when we've seen `{` and want to decide if the content is a statement
    /// block vs a record/map literal.
    fn starts_brace_block_expr_after_lbrace(&self) -> bool {
        let next = (self.pos + 1).min(self.tokens.len() - 1);
        let after = (self.pos + 2).min(self.tokens.len() - 1);
        matches!(
            self.tokens[next].kind,
            TokenKind::Let
                | TokenKind::Return
                | TokenKind::Node
                | TokenKind::Parallel
                | TokenKind::Match
                | TokenKind::If
                | TokenKind::For
                // Literals indicate a block expression, not a record
                | TokenKind::Str(_)
                | TokenKind::Int(_)
                | TokenKind::Float(_)
                | TokenKind::True
                | TokenKind::False
                | TokenKind::StrBegin(_)
                | TokenKind::NoneLit
        ) || (Self::token_kind_as_ident(&self.tokens[next].kind).is_some()
            && matches!(
                self.tokens[after].kind,
                TokenKind::Eq | TokenKind::LBracket | TokenKind::LParen
            ))
    }

    fn consume_brace_block_expr(&mut self) -> Result<Expr, ParseError> {
        let stmts = self.parse_stmts()?;
        Ok(Expr::Block(stmts))
    }

    fn brace_contains_top_level_colon(&self) -> bool {
        let mut idx = self.pos;
        let mut depth = 1usize;
        while idx < self.tokens.len() {
            match self.tokens[idx].kind {
                TokenKind::LBrace => depth += 1,
                TokenKind::RBrace => {
                    depth -= 1;
                    if depth == 0 {
                        return false;
                    }
                }
                TokenKind::Colon if depth == 1 => return true,
                _ => {}
            }
            idx += 1;
        }
        false
    }

    fn parse_map_literal_entries(&mut self) -> Result<Vec<(Expr, Expr)>, ParseError> {
        let mut entries = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            let key = self.parse_expr(0)?;
            self.expect(&TokenKind::Colon)?;
            let val = self.parse_expr(0)?;
            entries.push((key, val));
            self.eat(&TokenKind::Comma);
        }
        Ok(entries)
    }

    fn parse_call_expr(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::LParen)?;
        let mut args: Vec<(Option<String>, Expr)> = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                if Self::token_kind_as_ident(&self.peek().kind).is_some()
                    && self.peek2().kind == TokenKind::Colon
                {
                    let name = self.expect_ident()?;
                    self.expect(&TokenKind::Colon)?;
                    let val = self.parse_expr(0)?;
                    args.push((Some(name), val));
                } else {
                    let val = self.parse_expr(0)?;
                    args.push((None, val));
                }
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
                if self.check(&TokenKind::RParen) {
                    break;
                }
            }
        }
        self.expect(&TokenKind::RParen)?;
        match flatten_path(&callee) {
            Some(path) if path.len() == 1 => Ok(Expr::Call(path[0].clone(), args)),
            Some(path) => Ok(Expr::ServiceCall(path, args)),
            None => {
                Err(self.err("call expressions require an identifier or dotted path callee".into()))
            }
        }
    }

    fn parse_string_interp(&mut self, first: String) -> Result<Expr, ParseError> {
        self.advance();
        let mut parts: Vec<StringPart> = Vec::new();
        if !first.is_empty() {
            parts.push(StringPart::Literal(first));
        }
        loop {
            let expr = self.parse_expr(0)?;
            parts.push(StringPart::Expr(expr));
            match self.peek().kind.clone() {
                TokenKind::StrMid(mid) => {
                    self.advance();
                    if !mid.is_empty() {
                        parts.push(StringPart::Literal(mid));
                    }
                }
                TokenKind::StrEnd(end) => {
                    self.advance();
                    if !end.is_empty() {
                        parts.push(StringPart::Literal(end));
                    }
                    break;
                }
                _ => break,
            }
        }
        Ok(Expr::StringInterp(parts))
    }

    fn parse_match_expr(&mut self) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::Match)?;
        let scrutinee = self.parse_for_iterable_expr()?;
        self.expect(&TokenKind::LBrace)?;
        let mut arms = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            let pattern = self.parse_pattern()?;
            let guard = if self.eat(&TokenKind::If) {
                Some(self.parse_expr(0)?)
            } else {
                None
            };
            self.expect(&TokenKind::FatArrow)?;
            let body =
                if self.check(&TokenKind::LBrace) && self.starts_brace_block_expr_after_lbrace() {
                    // Match arm bodies with { let ... } or { if ... } are parsed as blocks.
                    self.advance(); // consume {
                    let mut stmts = self.parse_stmts()?;
                    self.expect(&TokenKind::RBrace)?;
                    if stmts.len() == 1 {
                        match stmts.remove(0) {
                            Stmt::Expr(expr) => expr,
                            other => Expr::Block(vec![other]),
                        }
                    } else {
                        Expr::Block(stmts)
                    }
                } else if self.starts_brace_block_expr() {
                    // Implicit block: multi-statement body without braces
                    // (e.g., `Text { v } => let x = ...\n expr`)
                    // Parse statements until next pattern or closing }.
                    let mut stmts = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.at_eof() {
                        // Stop if we see what looks like a new match arm
                        if self.looks_like_match_arm_start() {
                            break;
                        }
                        match self.parse_stmt() {
                            Ok(s) => stmts.push(s),
                            Err(e) => {
                                self.record_err(e);
                                break;
                            }
                        }
                    }
                    Expr::Block(stmts)
                } else {
                    self.parse_expr(0)?
                };
            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });
            self.eat(&TokenKind::Comma);
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(Expr::Match(Box::new(scrutinee), arms))
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.peek().kind.clone() {
            ref kind if Self::token_kind_as_ident(kind).is_some() => {
                let name = self.expect_ident()?;
                if name == "_" {
                    return Ok(Pattern::Wildcard);
                }
                if self.eat(&TokenKind::LParen) {
                    let mut args = Vec::new();
                    let mut index = 0;
                    while !self.check(&TokenKind::RParen) && !self.at_eof() {
                        let inner = self.parse_pattern()?;
                        args.push((index.to_string(), inner));
                        index += 1;
                        self.eat(&TokenKind::Comma);
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Pattern::Variant(name, args))
                } else if self.eat(&TokenKind::LBrace) {
                    let mut args = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.at_eof() {
                        if self.eat(&TokenKind::Dot) {
                            // tolerate spread-ish syntax (`...`) in match patterns
                            self.eat(&TokenKind::Dot);
                            self.eat(&TokenKind::Dot);
                            if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                                let _ = self.expect_ident()?;
                            }
                            self.eat(&TokenKind::Comma);
                            continue;
                        }
                        let field = self.expect_ident()?;
                        if self.eat(&TokenKind::Colon) {
                            let inner = self.parse_pattern()?;
                            args.push((field, inner));
                        } else {
                            // Shorthand: `{ name }` means `{ name: name }`
                            args.push((field.clone(), Pattern::Ident(field)));
                        }
                        self.eat(&TokenKind::Comma);
                    }
                    self.expect(&TokenKind::RBrace)?;
                    Ok(Pattern::Variant(name, args))
                } else {
                    Ok(Pattern::Ident(name))
                }
            }
            TokenKind::True => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(false)))
            }
            TokenKind::Int(n) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Int(n)))
            }
            TokenKind::Float(f) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Float(f)))
            }
            TokenKind::Str(s) => {
                self.advance();
                Ok(Pattern::Literal(Literal::String(s)))
            }
            _ => Err(self.err("expected pattern".into())),
        }
    }

    fn parse_if_expr(&mut self) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::If)?;
        let cond = self.parse_for_iterable_expr()?;
        self.expect(&TokenKind::LBrace)?;
        let then_b = self.parse_if_branch_body()?;
        self.expect(&TokenKind::RBrace)?;
        let else_b = if self.eat(&TokenKind::Else) {
            if self.check(&TokenKind::If) {
                // else if chain → recurse
                Some(Box::new(self.parse_if_expr()?))
            } else {
                self.expect(&TokenKind::LBrace)?;
                let e = self.parse_if_branch_body()?;
                self.expect(&TokenKind::RBrace)?;
                Some(Box::new(e))
            }
        } else {
            None
        };
        Ok(Expr::If(Box::new(cond), Box::new(then_b), else_b))
    }

    /// Parse the body of an if/else branch: either a single expression or a
    /// multi-statement block (like `for` bodies).
    fn parse_if_branch_body(&mut self) -> Result<Expr, ParseError> {
        if self.starts_brace_block_expr() {
            let stmts = self.parse_stmts()?;
            Ok(Expr::Block(stmts))
        } else {
            self.parse_expr(0)
        }
    }

    fn parse_for_expr(&mut self) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::For)?;
        let var = self.expect_ident()?;
        self.expect(&TokenKind::In)?;
        let iter = self.parse_for_iterable_expr()?;
        let (iter, passthrough) = self.split_for_iterable_and_passthrough(&var, iter)?;
        self.expect(&TokenKind::LBrace)?;
        let body = if self.starts_brace_block_expr() {
            crate::ast::ForBody::Block(self.parse_stmts()?)
        } else {
            crate::ast::ForBody::Expr(Box::new(self.parse_expr(0)?))
        };
        self.expect(&TokenKind::RBrace)?;
        Ok(Expr::For(var, Box::new(iter), passthrough, body))
    }

    fn parse_for_iterable_expr(&mut self) -> Result<Expr, ParseError> {
        let previous = self.allow_named_record_suffix;
        self.allow_named_record_suffix = false;
        let parsed = self.parse_expr(0);
        self.allow_named_record_suffix = previous;
        parsed
    }

    fn split_for_iterable_and_passthrough(
        &self,
        loop_var: &str,
        iter_expr: Expr,
    ) -> Result<(Expr, Vec<String>), ParseError> {
        let Expr::Call(name, args) = &iter_expr else {
            return Ok((iter_expr, Vec::new()));
        };
        if name != "with" || args.len() != 2 || args[0].0.is_some() || args[1].0.is_some() {
            return Ok((iter_expr, Vec::new()));
        }
        let Expr::Record(None, fields) = &args[1].1 else {
            return Ok((iter_expr, Vec::new()));
        };

        let mut passthrough = Vec::with_capacity(fields.len());
        for (field_name, field_expr) in fields {
            match field_expr {
                Expr::Ident(ident) if ident == field_name => {}
                _ => return Ok((iter_expr, Vec::new())),
            }

            if field_name == loop_var {
                return Err(self.err(format!(
                    "loop passthrough cannot include loop variable '{loop_var}'"
                )));
            }
            if passthrough.iter().any(|existing| existing == field_name) {
                return Err(self.err(format!("duplicate loop passthrough binding '{field_name}'")));
            }
            passthrough.push(field_name.clone());
        }

        Ok((args[0].1.clone(), passthrough))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_or_panic(source: &str) -> SourceFile {
        parse(source)
            .unwrap_or_else(|errs| panic!("parse failed with {} error(s): {:?}", errs.len(), errs))
    }

    fn parse_expr_only(source: &str) -> Expr {
        let tokens = Lexer::tokenize(source);
        let mut parser = Parser::new(tokens);
        parser.parse_expr(0).expect("expression should parse")
    }

    fn parse_expr_only_err(source: &str) -> ParseError {
        let tokens = Lexer::tokenize(source);
        let mut parser = Parser::new(tokens);
        parser.parse_expr(0).expect_err("expression should fail")
    }

    fn parse_source_err(source: &str) -> ParseError {
        let mut errs = parse(source).expect_err("source should fail");
        errs.remove(0)
    }

    #[test]
    fn parse_module_decl() {
        let sf = parse_or_panic("module tools.makegen");
        let mp = sf.module_path.unwrap();
        assert_eq!(mp.node.segments, vec!["tools", "makegen"]);
    }

    #[test]
    fn parse_import_with_bindings() {
        let sf = parse_or_panic("module foo\nimport std.types { ToolRegistry }");
        assert_eq!(sf.imports.len(), 1);
        assert_eq!(
            sf.imports[0].node.bindings,
            Some(vec!["ToolRegistry".into()])
        );
    }

    #[test]
    fn parse_service_config_auth_input_rejects_string_literal() {
        let err = parse_source_err(
            r#"module services.example
service github.Gist {
  config {
    endpoint: "https://api.github.com"
    auth: BearerToken
    auth_input: "token"
  }
  operation Create() -> { id: String }
}"#,
        );
        assert!(
            err.message.contains("expected identifier for `auth_input`"),
            "unexpected parse error: {}",
            err.message
        );
    }

    #[test]
    fn parse_call_expr_rejects_non_path_callee() {
        let err = parse_expr_only_err("(1 + 2)(3)");
        assert!(
            err.message
                .contains("call expressions require an identifier or dotted path callee"),
            "unexpected parse error: {}",
            err.message
        );
    }

    #[test]
    fn parse_simple_fn() {
        let sf = parse_or_panic("module test\nfn greet(name: String) -> String { name }");
        assert_eq!(sf.items.len(), 1);
        match &sf.items[0].node {
            Item::FnDef(f) => {
                assert_eq!(f.name, "greet");
                assert!(f.type_params.is_empty());
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_fn_type_params_are_preserved() {
        let sf = parse_or_panic("module test\nfn identity<T>(value: T) -> T { value }");
        match &sf.items[0].node {
            Item::FnDef(f) => assert_eq!(f.type_params, vec!["T".to_string()]),
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_pattern_type_params_capture_arrow_output_variable() {
        let sf = parse_or_panic(
            "module test\npattern upsert<Check, Create, Resolve: -> R>(value: Check) -> { value: R } { return { value: value } }",
        );
        match &sf.items[0].node {
            Item::PatternDef(p) => {
                assert_eq!(
                    p.type_params,
                    vec![
                        "Check".to_string(),
                        "Create".to_string(),
                        "Resolve".to_string(),
                        "R".to_string()
                    ]
                );
            }
            other => panic!("expected PatternDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_fn_body_preserves_match_expression_shape() {
        let sf = parse_or_panic(
            r#"module test
type CloudConfig = GcpConfig { project: String } | AwsConfig { account: String }
type CloudProvider = Gcp | Aws
fn provider_of(config: CloudConfig) -> CloudProvider {
  match config {
    GcpConfig { ... } => Gcp
    AwsConfig { ... } => Aws
  }
}"#,
        );
        match &sf.items[2].node {
            Item::FnDef(f) => {
                assert!(matches!(
                    f.body.stmts.first(),
                    Some(Stmt::Expr(Expr::Match(_, arms))) if arms.len() == 2
                ));
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_if_expression_in_fn_body() {
        let sf = parse_or_panic(
            r#"module test
fn choose(gate: Bool) -> Int {
  let value = if gate { 1 } else { 0 }
  value
}"#,
        );
        match &sf.items[0].node {
            Item::FnDef(def) => {
                assert!(matches!(
                    def.body.stmts.first(),
                    Some(Stmt::Let(_, Expr::If(_, _, Some(_))))
                ));
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_match_expression_in_fn_body() {
        let sf = parse_or_panic(
            r#"module test
fn choose(mode: String) -> Int {
  let value = match mode {
    "hot" => 1
    _ => 0
  }
  value
}"#,
        );
        match &sf.items[0].node {
            Item::FnDef(def) => {
                assert!(matches!(
                    def.body.stmts.first(),
                    Some(Stmt::Let(_, Expr::Match(_, _)))
                ));
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_fold_pipe_in_fn_body() {
        let sf = parse_or_panic(
            r#"module test
type FermiDepth = Xs | S | M | L | Xl
fn fermi_max(lhs: FermiDepth, rhs: FermiDepth) -> FermiDepth {
  lhs
}
fn fermi_max_of(depths: List<FermiDepth>) -> FermiDepth {
  fold(depths, Xs, (acc, d) => fermi_max(lhs: acc, rhs: d))
}"#,
        );
        // fermi_max_of should be the third item (after module, type, fermi_max)
        let fn_item = sf
            .items
            .iter()
            .find(|item| matches!(&item.node, Item::FnDef(def) if def.name == "fermi_max_of"))
            .expect("fermi_max_of fn should exist");
        assert!(
            matches!(&fn_item.node, Item::FnDef(def) if !def.body.stmts.is_empty()),
            "fermi_max_of fn body should have statements"
        );
    }

    #[test]
    fn parse_classify_transports_body() {
        let sf = parse_or_panic(
            r#"module test
type FermiDepth = Xs | S | M | L | Xl
type TransportClass = LocalDirect | ShellLocal | FileBoundary | RestNetwork | InterfaceStub | Unknown
type TestClass = Unit | Hermetic | Integration
type DerivedClassification { test_class: TestClass, depth: FermiDepth, hermetic: Bool }
fn transport_depth(tc: TransportClass) -> FermiDepth {
  match tc { LocalDirect => Xs, InterfaceStub => Xs, ShellLocal => S, FileBoundary => S, RestNetwork => L, Unknown => Xl }
}
fn transport_hermetic(tc: TransportClass) -> Bool {
  match tc { LocalDirect => true, InterfaceStub => true, ShellLocal => true, FileBoundary => true, RestNetwork => false, Unknown => false }
}
fn fermi_ordinal(depth: FermiDepth) -> Int {
  match depth { Xs => 0, S => 1, M => 2, L => 3, Xl => 4 }
}
fn fermi_gt(lhs: FermiDepth, rhs: FermiDepth) -> Bool {
  fermi_ordinal(depth: lhs) > fermi_ordinal(depth: rhs)
}
fn fermi_max(lhs: FermiDepth, rhs: FermiDepth) -> FermiDepth {
  if fermi_gt(lhs: lhs, rhs: rhs) { lhs } else { rhs }
}
fn fermi_max_of(depths: List<FermiDepth>) -> FermiDepth {
  fold(depths, Xs, (acc, d) => fermi_max(lhs: acc, rhs: d))
}
fn classify_transports(transports: List<TransportClass>) -> DerivedClassification {
  let depths = map(transports, tc => transport_depth(tc: tc))
  let max_depth = fermi_max_of(depths: depths)
  let all_hermetic = all(transports, tc => transport_hermetic(tc: tc))
  let n = count(transports)
  let test_class = if n == 0 { Unit } else { if all_hermetic { Hermetic } else { Integration } }
  let depth = if n == 0 { Xs } else { max_depth }
  let hermetic = if n == 0 { true } else { all_hermetic }
  DerivedClassification { test_class: test_class, depth: depth, hermetic: hermetic }
}"#,
        );
        let fn_item = sf
            .items
            .iter()
            .find(
                |item| matches!(&item.node, Item::FnDef(def) if def.name == "classify_transports"),
            )
            .expect("classify_transports fn should exist");
        assert!(
            matches!(&fn_item.node, Item::FnDef(def) if !def.body.stmts.is_empty()),
            "classify_transports fn body should have statements"
        );
    }

    #[test]
    fn parse_type_alias() {
        let sf = parse_or_panic("module test\ntype Url = String");
        match &sf.items[0].node {
            Item::TypeDef(td) => {
                assert_eq!(td.name, "Url");
                assert!(
                    matches!(td.body, TypeBody::Alias(TypeExpr::Named(ref n)) if n == "String")
                );
            }
            other => panic!("expected TypeDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_type_sum() {
        let sf = parse_or_panic("module test\ntype Color = Red | Green | Blue");
        match &sf.items[0].node {
            Item::TypeDef(td) => {
                if let TypeBody::Sum(vs) = &td.body {
                    assert_eq!(vs.len(), 3);
                    assert_eq!(vs[0].name, "Red");
                } else {
                    panic!("expected Sum");
                }
            }
            other => panic!("expected TypeDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_type_record() {
        let sf = parse_or_panic("module test\ntype Foo { a: Int, b: String }");
        match &sf.items[0].node {
            Item::TypeDef(td) => {
                if let TypeBody::Record(fs) = &td.body {
                    assert_eq!(fs.len(), 2);
                    assert_eq!(fs[0].name, "a");
                } else {
                    panic!("expected Record");
                }
            }
            other => panic!("expected TypeDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_data_declaration() {
        let sf = parse_or_panic(
            r#"module test
data tool_registry: List<String> = ["makegen", "testgen"]
"#,
        );
        match &sf.items[0].node {
            Item::DataDef(def) => {
                assert_eq!(def.name, "tool_registry");
                assert!(matches!(
                    def.ty,
                    TypeExpr::Generic(ref name, _) if name == "List"
                ));
                assert!(matches!(def.value, Expr::List(_)));
            }
            other => panic!("expected DataDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_profile_definition_with_binds_and_config() {
        let sf = parse_or_panic(
            r#"module profiles.local
profile unit_test {
  bind IssueProvider -> stub.IssueProvider
  bind AgentProvider -> stub.AgentProvider {
    approval_mode: "full_auto"
  }
}
"#,
        );
        match &sf.items[0].node {
            Item::ProfileDef(def) => {
                assert_eq!(def.name, "unit_test");
                assert_eq!(def.binds.len(), 2);
                assert_eq!(def.binds[0].interface_type, "IssueProvider");
                assert_eq!(def.binds[0].implementation_type, "stub.IssueProvider");
                assert!(def.binds[0].config_entries.is_empty());
                assert_eq!(def.binds[1].interface_type, "AgentProvider");
                assert_eq!(def.binds[1].implementation_type, "stub.AgentProvider");
                assert_eq!(def.binds[1].config_entries.len(), 1);
            }
            other => panic!("expected ProfileDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_pipeline_stage_preserves_after_and_when_clauses() {
        let sf = parse_or_panic(
            r#"module workflows.gist
pipeline gist {
  stage list_files [after codegen_ensure, when mode == "gist" || mode == "gist-snapshot"] {}
}"#,
        );
        match &sf.items[0].node {
            Item::PipelineDef(def) => {
                assert_eq!(def.stages.len(), 1);
                let stage = &def.stages[0];
                assert_eq!(stage.name, "list_files");
                assert_eq!(stage.after, vec!["codegen_ensure".to_string()]);
                assert!(matches!(stage.when, Some(Expr::BinOp(_, BinOp::Or, _))));
            }
            other => panic!("expected PipelineDef, got {other:?}"),
        }
    }

    #[test]
    fn expression_precedence_multiplicative_over_additive() {
        let expr = parse_expr_only("a + b * c");
        match expr {
            Expr::BinOp(lhs, BinOp::Add, rhs) => {
                assert!(matches!(*lhs, Expr::Ident(ref name) if name == "a"));
                assert!(matches!(*rhs, Expr::BinOp(_, BinOp::Mul, _)));
            }
            other => panic!("unexpected expression tree: {other:?}"),
        }
    }

    #[test]
    fn expression_logical_and_binds_tighter_than_or() {
        let expr = parse_expr_only("x || y && z");
        match expr {
            Expr::BinOp(lhs, BinOp::Or, rhs) => {
                assert!(matches!(*lhs, Expr::Ident(ref name) if name == "x"));
                assert!(matches!(*rhs, Expr::BinOp(_, BinOp::And, _)));
            }
            other => panic!("unexpected expression tree: {other:?}"),
        }
    }

    #[test]
    fn expression_unary_binds_after_field_and_call_suffixes() {
        let expr = parse_expr_only("-a.b(c)");
        match expr {
            Expr::UnaryOp(UnaryOp::Neg, inner) => match *inner {
                Expr::ServiceCall(path, args) => {
                    assert_eq!(path, vec!["a".to_string(), "b".to_string()]);
                    assert_eq!(args.len(), 1);
                }
                other => panic!("unexpected unary operand: {other:?}"),
            },
            other => panic!("unexpected expression tree: {other:?}"),
        }
    }

    #[test]
    fn expression_additive_binds_tighter_than_comparison() {
        let expr = parse_expr_only("a + b < c");
        match expr {
            Expr::BinOp(lhs, BinOp::Lt, rhs) => {
                assert!(matches!(*lhs, Expr::BinOp(_, BinOp::Add, _)));
                assert!(matches!(*rhs, Expr::Ident(ref name) if name == "c"));
            }
            other => panic!("unexpected expression tree: {other:?}"),
        }
    }

    #[test]
    fn parse_for_expr_with_passthrough_clause() {
        let expr = parse_expr_only("for item in items with { repo, branch } { render(item) }");
        match expr {
            Expr::For(var, iter, passthrough, body) => {
                assert_eq!(var, "item");
                assert!(matches!(*iter, Expr::Ident(ref ident) if ident == "items"));
                assert_eq!(passthrough, vec!["repo".to_string(), "branch".to_string()]);
                assert!(matches!(
                    body,
                    crate::ast::ForBody::Expr(inner)
                        if matches!(inner.as_ref(), Expr::Call(ref name, _) if name == "render")
                ));
            }
            other => panic!("unexpected expression tree: {other:?}"),
        }
    }

    #[test]
    fn parse_for_expr_rejects_passthrough_loop_var_collision() {
        let err = parse_expr_only_err("for item in items with { item } { item }");
        assert!(err.message.contains("cannot include loop variable 'item'"));
    }

    #[test]
    fn parse_for_expr_rejects_duplicate_passthrough_bindings() {
        let err = parse_expr_only_err("for item in items with { repo, repo } { item }");
        assert!(err
            .message
            .contains("duplicate loop passthrough binding 'repo'"));
    }

    #[test]
    fn parse_for_expr_without_passthrough_remains_supported() {
        let expr = parse_expr_only("for item in items { item }");
        match expr {
            Expr::For(var, iter, passthrough, body) => {
                assert_eq!(var, "item");
                assert!(matches!(*iter, Expr::Ident(ref ident) if ident == "items"));
                assert!(passthrough.is_empty());
                assert!(matches!(
                    body,
                    crate::ast::ForBody::Expr(inner)
                        if matches!(inner.as_ref(), Expr::Ident(ref ident) if ident == "item")
                ));
            }
            other => panic!("unexpected expression tree: {other:?}"),
        }
    }

    #[test]
    fn parse_for_expr_with_statement_block_body() {
        let expr = parse_expr_only(
            "for item in items { rendered = render(item) return { value: rendered.output } }",
        );
        match expr {
            Expr::For(var, iter, passthrough, crate::ast::ForBody::Block(stmts)) => {
                assert_eq!(var, "item");
                assert!(matches!(*iter, Expr::Ident(ref ident) if ident == "items"));
                assert!(passthrough.is_empty());
                assert!(matches!(
                    stmts.as_slice(),
                    [
                        Stmt::Assign(name, Expr::Call(call_name, _)),
                        Stmt::Return(fields)
                    ] if name == "rendered"
                        && call_name == "render"
                        && matches!(
                            fields.as_slice(),
                            [(field_name, Expr::FieldAccess(base, output_field))]
                                if field_name == "value"
                                    && matches!(base.as_ref(), Expr::Ident(base_name) if base_name == "rendered")
                                    && output_field == "output"
                        )
                ));
            }
            other => panic!("unexpected expression tree: {other:?}"),
        }
    }

    #[test]
    fn question_in_expression_is_targeted_error() {
        let err = parse_expr_only_err("a ? b : c");
        assert!(err.message.contains("ternary operator is not supported"));
    }

    #[test]
    fn extern_func_is_rejected() {
        let err = parse_source_err(
            "module test.m\nextern func fetch_data(url: String) -> { body: String }",
        );
        assert!(
            err.message.contains("no longer supported"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn parse_extern_asset_decl() {
        let source = r#"
            module test.extern_mod
            extern asset config: Config
        "#;
        let ast = parse_or_panic(source);
        assert_eq!(ast.items.len(), 1);
        match &ast.items[0].node {
            Item::ExternAssetDecl(def) => {
                assert_eq!(def.name, "config");
                assert!(matches!(def.ty, TypeExpr::Named(ref n) if n == "Config"));
            }
            other => panic!("expected ExternAssetDecl, got {other:?}"),
        }
    }

    #[test]
    fn retry_refinement_is_rejected() {
        let err = parse_source_err("module test\ntype Req = String where retry(max: 3)");
        assert!(
            err.message.contains("@retry is not supported"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn error_map_refinement_is_rejected() {
        let err =
            parse_source_err("module test\ntype Req = String where error_map(401: Unauthorized)");
        assert!(
            err.message.contains("@error_map is not supported"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn requires_refinement_is_rejected() {
        let err = parse_source_err(
            r#"module test
type Req = String where requires(env: "TOKEN")"#,
        );
        assert!(
            err.message.contains("@requires is not supported"),
            "unexpected error: {}",
            err.message
        );
    }

    #[test]
    fn hermetic_on_operation_is_silently_accepted() {
        // hermetic is a no-op on operations (hermeticity is derived from transport),
        // but it must not break parsing of existing .dag files.
        let _ast = parse_or_panic(
            r#"module test
service foo.Bar {
  operation Baz hermetic {
    input { x: String }
    output { y: String }
  }
}"#,
        );
    }

    #[test]
    fn parse_service_config_with_rate_limit() {
        let source = r#"
            module services.example
            service github.Gist {
                config {
                    endpoint: "https://api.github.com"
                    auth: BearerToken
                    rate_limit: { requests: 5000, per: hour, scope: core }
                }
                operation Create {
                    input { content: String }
                    output { id: String }
                    transport rest { method: POST, path: "/gists" }
                }
            }
        "#;
        let ast = parse_or_panic(source);
        assert_eq!(ast.items.len(), 1);
        match &ast.items[0].node {
            Item::ServiceDef(def) => {
                assert_eq!(def.config.rate_limits.len(), 1);
                let rl = &def.config.rate_limits[0];
                assert_eq!(rl.requests, 5000);
                assert_eq!(rl.per, RateLimitUnit::Hour);
                assert_eq!(rl.scope, Some("core".to_string()));
            }
            other => panic!("expected ServiceDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_service_config_with_retry() {
        let source = r#"
            module services.example
            service llm.Anthropic {
                config {
                    endpoint: "https://api.anthropic.com"
                    retry: { max_attempts: 3, backoff: exponential, retry_on: [429, 500, 502] }
                }
                operation Messages {
                    input { model: String }
                    output { content: String }
                    transport rest { method: POST, path: "/v1/messages" }
                }
            }
        "#;
        let ast = parse_or_panic(source);
        match &ast.items[0].node {
            Item::ServiceDef(def) => {
                assert!(def.config.retry.is_some());
                let retry = def.config.retry.as_ref().unwrap();
                assert_eq!(retry.max_attempts, 3);
                assert_eq!(retry.backoff, BackoffStrategy::Exponential);
                assert_eq!(retry.retry_on, vec![429, 500, 502]);
            }
            other => panic!("expected ServiceDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_operation_with_response_block() {
        // Simple test without string interpolation to avoid StrBegin
        let source = r#"
            module services.example
            service github.Issues {
                config { endpoint: "https://api.github.com" }
                operation Get {
                    input { id: Int }
                    output { issue: Issue }
                    transport rest { method: GET, path: "/issues/123" }
                    response {
                        200 => Issue
                        4xx => ClientError
                        5xx => ServerError
                    }
                }
            }
        "#;
        let ast = parse_or_panic(source);
        match &ast.items[0].node {
            Item::ServiceDef(def) => {
                assert_eq!(def.operations.len(), 1);
                let op = &def.operations[0];
                assert_eq!(op.name, "Get");
                assert_eq!(op.response.len(), 3);

                // Check exact status code
                assert_eq!(op.response[0].status, StatusPattern::Exact(200));
                assert!(
                    matches!(op.response[0].response_type, TypeExpr::Named(ref n) if n == "Issue")
                );

                // Check wildcard patterns
                assert_eq!(op.response[1].status, StatusPattern::ClientError4xx);
                assert_eq!(op.response[2].status, StatusPattern::ServerError5xx);
            }
            other => panic!("expected ServiceDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_operation_with_exit_block() {
        let source = r#"
            module services.example
            service shell.Git {
                operation CurrentBranch {
                    input {}
                    output { branch: String }
                    readonly
                    transport shell { argv: ["git", "rev-parse", "--abbrev-ref", "HEAD"] }
                    exit {
                        0 => Unit
                        1 => String "Not a git repository"
                        nonzero => Error "Command failed"
                    }
                }
            }
        "#;
        let ast = parse_or_panic(source);
        match &ast.items[0].node {
            Item::ServiceDef(def) => {
                assert_eq!(def.operations.len(), 1);
                let op = &def.operations[0];
                assert_eq!(op.name, "CurrentBranch");
                assert_eq!(op.exit.len(), 3);

                // Check exact exit code 0
                assert_eq!(op.exit[0].code, ExitCode::Exact(0));
                assert!(matches!(op.exit[0].output_type, TypeExpr::Named(ref n) if n == "Unit"));
                assert!(op.exit[0].description.is_none());

                // Check exact exit code 1 with description
                assert_eq!(op.exit[1].code, ExitCode::Exact(1));
                assert!(matches!(op.exit[1].output_type, TypeExpr::Named(ref n) if n == "String"));
                assert_eq!(
                    op.exit[1].description.as_deref(),
                    Some("Not a git repository")
                );

                // Check nonzero wildcard
                assert_eq!(op.exit[2].code, ExitCode::NonZero);
                assert!(matches!(op.exit[2].output_type, TypeExpr::Named(ref n) if n == "Error"));
                assert_eq!(op.exit[2].description.as_deref(), Some("Command failed"));
            }
            other => panic!("expected ServiceDef, got {other:?}"),
        }
    }

    // ── Strict block validation tests ────────────────────────────────

    #[test]
    fn rate_limit_rejects_unknown_field() {
        let err = parse_source_err(
            r#"module t
service rest.T {
    config {
        endpoint: "http://x"
        rate_limit: { requests: 10, per: minute, bogus: 1 }
    }
    operation X { input {} output { r: String } transport rest { method: GET, path: "/" } }
}"#,
        );
        assert!(
            err.message.contains("unknown field `bogus`"),
            "expected unknown field error, got: {}",
            err.message
        );
    }

    #[test]
    fn rate_limit_requires_requests_field() {
        let err = parse_source_err(
            r#"module t
service rest.T {
    config {
        endpoint: "http://x"
        rate_limit: { per: minute }
    }
    operation X { input {} output { r: String } transport rest { method: GET, path: "/" } }
}"#,
        );
        assert!(
            err.message.contains("requires `requests`"),
            "expected missing requests error, got: {}",
            err.message
        );
    }

    #[test]
    fn rate_limit_requires_per_field() {
        let err = parse_source_err(
            r#"module t
service rest.T {
    config {
        endpoint: "http://x"
        rate_limit: { requests: 10 }
    }
    operation X { input {} output { r: String } transport rest { method: GET, path: "/" } }
}"#,
        );
        assert!(
            err.message.contains("requires `per`"),
            "expected missing per error, got: {}",
            err.message
        );
    }

    #[test]
    fn rate_limit_rejects_zero_requests() {
        let err = parse_source_err(
            r#"module t
service rest.T {
    config {
        endpoint: "http://x"
        rate_limit: { requests: 0, per: minute }
    }
    operation X { input {} output { r: String } transport rest { method: GET, path: "/" } }
}"#,
        );
        assert!(
            err.message.contains("must be > 0"),
            "expected positive requests error, got: {}",
            err.message
        );
    }

    #[test]
    fn rate_limit_rejects_unknown_unit() {
        let err = parse_source_err(
            r#"module t
service rest.T {
    config {
        endpoint: "http://x"
        rate_limit: { requests: 10, per: fortnight }
    }
    operation X { input {} output { r: String } transport rest { method: GET, path: "/" } }
}"#,
        );
        assert!(
            err.message.contains("unknown rate_limit unit"),
            "expected unknown unit error, got: {}",
            err.message
        );
    }

    #[test]
    fn retry_rejects_unknown_field() {
        let err = parse_source_err(
            r#"module t
service rest.T {
    config {
        endpoint: "http://x"
        retry: { max_attempts: 3, jitter: true }
    }
    operation X { input {} output { r: String } transport rest { method: GET, path: "/" } }
}"#,
        );
        assert!(
            err.message.contains("unknown field `jitter`"),
            "expected unknown field error, got: {}",
            err.message
        );
    }

    #[test]
    fn retry_requires_max_attempts() {
        let err = parse_source_err(
            r#"module t
service rest.T {
    config {
        endpoint: "http://x"
        retry: { backoff: linear }
    }
    operation X { input {} output { r: String } transport rest { method: GET, path: "/" } }
}"#,
        );
        assert!(
            err.message.contains("requires `max_attempts`"),
            "expected missing max_attempts error, got: {}",
            err.message
        );
    }

    #[test]
    fn retry_rejects_zero_max_attempts() {
        let err = parse_source_err(
            r#"module t
service rest.T {
    config {
        endpoint: "http://x"
        retry: { max_attempts: 0 }
    }
    operation X { input {} output { r: String } transport rest { method: GET, path: "/" } }
}"#,
        );
        assert!(
            err.message.contains("must be >= 1"),
            "expected positive max_attempts error, got: {}",
            err.message
        );
    }

    #[test]
    fn retry_rejects_unknown_backoff_strategy() {
        let err = parse_source_err(
            r#"module t
service rest.T {
    config {
        endpoint: "http://x"
        retry: { max_attempts: 3, backoff: fibonacci }
    }
    operation X { input {} output { r: String } transport rest { method: GET, path: "/" } }
}"#,
        );
        assert!(
            err.message.contains("unknown backoff strategy"),
            "expected unknown backoff error, got: {}",
            err.message
        );
    }

    #[test]
    fn retry_rejects_negative_delay() {
        // Negative numbers tokenize as `-` then `100`, so the parser
        // rejects `-` as a non-integer token for base_delay_ms.
        let err = parse_source_err(
            r#"module t
service rest.T {
    config {
        endpoint: "http://x"
        retry: { max_attempts: 3, base_delay_ms: -100 }
    }
    operation X { input {} output { r: String } transport rest { method: GET, path: "/" } }
}"#,
        );
        assert!(
            err.message.contains("base_delay_ms"),
            "expected base_delay_ms error, got: {}",
            err.message
        );
    }

    #[test]
    fn error_shape_rejects_unknown_field() {
        let err = parse_source_err(
            r#"module t
service rest.T {
    config {
        endpoint: "http://x"
        error_shape: { status: 400, code_path: "$.code" }
    }
    operation X { input {} output { r: String } transport rest { method: GET, path: "/" } }
}"#,
        );
        assert!(
            err.message.contains("unknown field `code_path`"),
            "expected unknown field error, got: {}",
            err.message
        );
    }

    #[test]
    fn error_shape_requires_status() {
        let err = parse_source_err(
            r#"module t
service rest.T {
    config {
        endpoint: "http://x"
        error_shape: { message_path: "$.message" }
    }
    operation X { input {} output { r: String } transport rest { method: GET, path: "/" } }
}"#,
        );
        assert!(
            err.message.contains("requires `status`"),
            "expected missing status error, got: {}",
            err.message
        );
    }

    #[test]
    fn credential_rejects_unknown_field() {
        let err = parse_source_err(
            r#"module t
service rest.T {
    config {
        endpoint: "http://x"
        credential: { type: BearerToken, prefix: "Bearer" }
    }
    operation X { input {} output { r: String } transport rest { method: GET, path: "/" } }
}"#,
        );
        assert!(
            err.message.contains("unknown field `prefix`"),
            "expected unknown field error, got: {}",
            err.message
        );
    }

    #[test]
    fn credential_requires_type() {
        let err = parse_source_err(
            r#"module t
service rest.T {
    config {
        endpoint: "http://x"
        credential: { header: "Authorization" }
    }
    operation X { input {} output { r: String } transport rest { method: GET, path: "/" } }
}"#,
        );
        assert!(
            err.message.contains("requires `type`"),
            "expected missing type error, got: {}",
            err.message
        );
    }

    #[test]
    fn valid_rate_limit_and_retry_parse_successfully() {
        let sf = parse_or_panic(
            r#"module t
service rest.T {
    config {
        endpoint: "http://x"
        auth: BearerToken
        rate_limit: { requests: 100, per: minute, scope: global }
        retry: { max_attempts: 3, backoff: exponential, base_delay_ms: 1000, max_delay_ms: 30000, retry_on: [429, 503] }
    }
    operation X {
        input {}
        output { r: String }
        transport rest { method: GET, path: "/" }
    }
}"#,
        );
        match &sf.items[0].node {
            Item::ServiceDef(def) => {
                assert_eq!(def.config.rate_limits.len(), 1);
                assert_eq!(def.config.rate_limits[0].requests, 100);
                assert!(matches!(
                    def.config.rate_limits[0].per,
                    RateLimitUnit::Minute
                ));
                assert_eq!(def.config.rate_limits[0].scope.as_deref(), Some("global"));

                let retry = def.config.retry.as_ref().expect("retry should be present");
                assert_eq!(retry.max_attempts, 3);
                assert!(matches!(retry.backoff, BackoffStrategy::Exponential));
                assert_eq!(retry.base_delay_ms, Some(1000));
                assert_eq!(retry.max_delay_ms, Some(30000));
                assert_eq!(retry.retry_on, vec![429, 503]);
            }
            other => panic!("expected ServiceDef, got {other:?}"),
        }
    }

    #[test]
    fn valid_error_shape_and_credential_parse_successfully() {
        let sf = parse_or_panic(
            r#"module t
service rest.T {
    config {
        endpoint: "http://x"
        error_shape: { status: 400, error_type_path: "$.error.type", message_path: "$.error.message", retryable: false }
        credential: { type: ApiKey, header: "x-api-key", source: "env" }
    }
    operation X {
        input {}
        output { r: String }
        transport rest { method: GET, path: "/" }
    }
}"#,
        );
        match &sf.items[0].node {
            Item::ServiceDef(def) => {
                assert_eq!(def.config.error_shapes.len(), 1);
                assert_eq!(def.config.error_shapes[0].status, "400");
                assert_eq!(
                    def.config.error_shapes[0].error_type_path.as_deref(),
                    Some("$.error.type")
                );
                assert_eq!(
                    def.config.error_shapes[0].message_path.as_deref(),
                    Some("$.error.message")
                );
                assert!(!def.config.error_shapes[0].retryable);

                let cred = def
                    .config
                    .credential
                    .as_ref()
                    .expect("credential should be present");
                assert_eq!(cred.credential_type, "ApiKey");
                assert_eq!(cred.header.as_deref(), Some("x-api-key"));
                assert_eq!(cred.source.as_deref(), Some("env"));
            }
            other => panic!("expected ServiceDef, got {other:?}"),
        }
    }

    // RT-1: @mock_response parsing tests

    #[test]
    fn parse_operation_with_mock_response_block() {
        let source = r#"
            module services.example
            service github.Gist {
                operation Create {
                    input { description: String, content: String }
                    output { id: String, url: String }
                    transport rest { method: POST, path: "/gists" }
                    response {
                        201 => Gist
                        401 => Error
                    }
                    mock_response {
                        201 => { id: "gist-123", html_url: "https://gist.github.com/gist-123" }
                        401 => { message: "Bad credentials", documentation_url: "https://docs.github.com" }
                    }
                }
            }
        "#;
        let ast = parse_or_panic(source);
        match &ast.items[0].node {
            Item::ServiceDef(def) => {
                assert_eq!(def.operations.len(), 1);
                let op = &def.operations[0];
                assert_eq!(op.name, "Create");
                assert_eq!(op.mock_responses.len(), 2);

                // Check success mock
                assert_eq!(op.mock_responses[0].status, 201);
                match &op.mock_responses[0].body {
                    Expr::Record(_, fields) => {
                        assert_eq!(fields.len(), 2);
                        assert_eq!(fields[0].0, "id");
                        assert_eq!(fields[1].0, "html_url");
                    }
                    other => panic!("expected Record expr for body, got {other:?}"),
                }

                // Check error mock
                assert_eq!(op.mock_responses[1].status, 401);
                match &op.mock_responses[1].body {
                    Expr::Record(_, fields) => {
                        assert_eq!(fields.len(), 2);
                        assert_eq!(fields[0].0, "message");
                    }
                    other => panic!("expected Record expr for body, got {other:?}"),
                }
            }
            other => panic!("expected ServiceDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_mock_response_with_description() {
        let source = r#"
            module services.example
            service api.Example {
                operation Get {
                    input { id: String }
                    output { name: String }
                    transport rest { method: GET, path: "/things/123" }
                    mock_response {
                        200 => { name: "test-item" } "success response"
                        404 => { error: "not found" } "item not found"
                    }
                }
            }
        "#;
        let ast = parse_or_panic(source);
        match &ast.items[0].node {
            Item::ServiceDef(def) => {
                let op = &def.operations[0];
                assert_eq!(op.mock_responses.len(), 2);
                assert_eq!(
                    op.mock_responses[0].description.as_deref(),
                    Some("success response")
                );
                assert_eq!(
                    op.mock_responses[1].description.as_deref(),
                    Some("item not found")
                );
            }
            other => panic!("expected ServiceDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_operation_without_mock_response_has_empty_vec() {
        let source = r#"
            module services.example
            service api.Example {
                operation Get {
                    input { id: String }
                    output { name: String }
                    transport rest { method: GET, path: "/things/123" }
                }
            }
        "#;
        let ast = parse_or_panic(source);
        match &ast.items[0].node {
            Item::ServiceDef(def) => {
                let op = &def.operations[0];
                assert!(op.mock_responses.is_empty());
            }
            other => panic!("expected ServiceDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_test_targets_preserve_local_and_qualified_node_refs() {
        let source = r#"
            test node_refs {
                mock local/execute.response -> rest_response(200, { ok: true })
                input tools.shared::shared_node.prepare.arg -> "value"
            }
        "#;
        let ast = parse_or_panic(source);
        match &ast.items[0].node {
            Item::TestDef(def) => {
                assert_eq!(
                    def.mocks[0].node_ref,
                    TestNodeRef::Local {
                        node_segments: vec!["local".into(), "execute".into()],
                    }
                );
                assert_eq!(
                    def.inputs[0].node_ref,
                    TestNodeRef::Qualified {
                        module: ModulePath::new(vec!["tools".into(), "shared".into()]),
                        node_segments: vec!["shared_node".into(), "prepare".into()],
                    }
                );
                assert_eq!(def.inputs[0].port, "arg");
            }
            other => panic!("expected TestDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_expect_targets_preserve_structured_paths() {
        let source = r#"
            test expect_paths {
                expect result.content is String
                expect result.model == "gpt-4o"
                expect local.execute.ok
                expect tools.shared::shared_node.execute.done
            }
        "#;
        let ast = parse_or_panic(source);
        match &ast.items[0].node {
            Item::TestDef(def) => {
                assert_eq!(def.expects.len(), 4);

                // result.content → ExpectTarget::Result
                match &def.expects[0] {
                    ExpectStmt::Is(ExpectTarget::Result { port }, type_name) => {
                        assert_eq!(port, "content");
                        assert_eq!(type_name, "String");
                    }
                    other => panic!("expected Is(Result), got {other:?}"),
                }

                // result.model → ExpectTarget::Result
                match &def.expects[1] {
                    ExpectStmt::Eq(ExpectTarget::Result { port }, _) => {
                        assert_eq!(port, "model");
                    }
                    other => panic!("expected Eq(Result), got {other:?}"),
                }

                // local.execute.ok → ExpectTarget::Node with local ref
                match &def.expects[2] {
                    ExpectStmt::Truthy(ExpectTarget::Node { node_ref, port }) => {
                        assert_eq!(
                            *node_ref,
                            TestNodeRef::Local {
                                node_segments: vec!["local".into(), "execute".into()],
                            }
                        );
                        assert_eq!(port, "ok");
                    }
                    other => panic!("expected Truthy(Node/Local), got {other:?}"),
                }

                // tools.shared::shared_node.execute.done → ExpectTarget::Node with qualified ref
                match &def.expects[3] {
                    ExpectStmt::Truthy(ExpectTarget::Node { node_ref, port }) => {
                        assert_eq!(
                            *node_ref,
                            TestNodeRef::Qualified {
                                module: ModulePath::new(vec![
                                    "tools".into(),
                                    "shared".into()
                                ]),
                                node_segments: vec![
                                    "shared_node".into(),
                                    "execute".into()
                                ],
                            }
                        );
                        assert_eq!(port, "done");
                    }
                    other => panic!("expected Truthy(Node/Qualified), got {other:?}"),
                }
            }
            other => panic!("expected TestDef, got {other:?}"),
        }
    }

    // ── Diagnostic behavior (was tests/diagnostics.rs) ─────────────────

    #[test]
    fn byte_to_line_col_handles_multiline_offsets_and_eof_clamp() {
        let src = "module test\nfn broken(\n";
        assert_eq!(byte_to_line_col(src, 0), (1, 1));
        assert_eq!(byte_to_line_col(src, 7), (1, 8));
        assert_eq!(byte_to_line_col(src, 12), (2, 1));
        assert_eq!(
            byte_to_line_col(src, src.len() + 100),
            byte_to_line_col(src, src.len()),
            "offsets beyond EOF should clamp to EOF"
        );
    }

    #[test]
    fn byte_to_line_col_handles_utf8_byte_offsets() {
        let src = "éx\n";
        assert_eq!(byte_to_line_col(src, 0), (1, 1));
        assert_eq!(byte_to_line_col(src, 1), (1, 2));
        assert_eq!(byte_to_line_col(src, 2), (1, 2));
        assert_eq!(byte_to_line_col(src, 3), (1, 3));
    }

    #[test]
    fn parse_error_formats_with_file_line_col() {
        let src = "module test\nfn broken( -> String {\n";
        let err = parse(src)
            .expect_err("should fail")
            .into_iter()
            .next()
            .unwrap();
        let rendered = err.format_with_source(std::path::Path::new("sample.dag"), src);
        assert!(rendered.contains("sample.dag:2:12"));
    }

    #[test]
    fn parse_error_converts_to_parse_diagnostic() {
        use crate::diagnostic::DiagnosticKind;
        let src = "module test\nfn broken( -> String {\n";
        let err = parse(src)
            .expect_err("should fail")
            .into_iter()
            .next()
            .unwrap();
        let diag = err.to_diagnostic(std::path::Path::new("sample.dag"), src);
        assert_eq!(diag.kind, DiagnosticKind::Parse);
        assert_eq!(
            diag.file.as_ref().and_then(|f| f.to_str()),
            Some("sample.dag")
        );
        assert!(diag.span.is_some());
        assert_eq!(diag.line, Some(2));
    }

    #[test]
    fn parse_with_file_diagnostics_preserves_lex_diagnostic_kind() {
        use crate::diagnostic::DiagnosticKind;
        let src = "module test\n$\n";
        let diagnostics = parse_with_file_diagnostics(std::path::Path::new("s.dag"), src)
            .expect_err("should fail with lex diagnostic");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].kind, DiagnosticKind::Lex);
        assert_eq!(diagnostics[0].line, Some(2));
        assert_eq!(diagnostics[0].column, Some(1));
    }

    #[test]
    fn parse_with_file_diagnostics_preserves_parse_diagnostic_kind() {
        use crate::diagnostic::DiagnosticKind;
        let src = "module test\nfn broken( -> String {\n";
        let diagnostics = parse_with_file_diagnostics(std::path::Path::new("s.dag"), src)
            .expect_err("should fail with parse diagnostic");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].kind, DiagnosticKind::Parse);
        assert_eq!(diagnostics[0].line, Some(2));
        assert_eq!(diagnostics[0].column, Some(12));
    }

    #[test]
    fn parse_with_file_diagnostics_aggregates_multiple_lex_diagnostics() {
        use crate::diagnostic::DiagnosticKind;
        let src = "module test\n$\n&\n";
        let diagnostics = parse_with_file_diagnostics(std::path::Path::new("s.dag"), src)
            .expect_err("should fail with lex diagnostics");
        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics.iter().all(|d| d.kind == DiagnosticKind::Lex));
        assert_eq!(diagnostics[0].line, Some(2));
        assert_eq!(diagnostics[1].line, Some(3));
    }

    #[test]
    fn parse_with_file_diagnostics_reports_utf8_adjacent_lex_column() {
        use crate::diagnostic::DiagnosticKind;
        let src = "é$\n";
        let diagnostics = parse_with_file_diagnostics(std::path::Path::new("s.dag"), src)
            .expect_err("should fail with lex diagnostic");
        assert!(diagnostics.iter().all(|d| d.kind == DiagnosticKind::Lex));
        let dollar = diagnostics
            .iter()
            .find(|d| d.message.contains("unexpected character '$'"))
            .expect("expected diagnostic for '$'");
        assert_eq!(dollar.line, Some(1));
        assert_eq!(dollar.column, Some(2));
    }

    // ── Failure paths (was tests/failure_paths.rs) ─────────────────────

    #[test]
    fn malformed_inputs_return_errors_without_panicking() {
        for source in [
            "module bad\nfn",
            "module bad\nimport",
            "module bad\ntype",
            "module bad\n@",
        ] {
            let result = std::panic::catch_unwind(|| parse(source));
            assert!(result.is_ok(), "parser panicked on: {source:?}");
            assert!(
                result.unwrap().is_err(),
                "should return error for: {source:?}"
            );
        }
    }

    #[test]
    fn lexer_unknown_character_surfaces_as_parser_diagnostic() {
        let errors = parse("module bad\n$").expect_err("should fail");
        assert!(
            errors
                .iter()
                .any(|e| e.message.contains("unexpected character '$'")),
            "lex diagnostic should surface through parser: {errors:?}"
        );
    }

    #[test]
    fn parser_recovers_to_next_top_level_item_and_reports_multiple_errors() {
        let errors = parse("module bad\nimport\nfn broken(\ntype Broken =\n")
            .expect_err("should produce parse errors");
        assert!(
            errors.len() >= 2,
            "expected multiple diagnostics via recovery, got: {errors:?}"
        );
    }

    // ── Item variant coverage (was tests/item_coverage.rs + representative_ast.rs) ──

    #[test]
    fn parser_handles_all_core_item_variants() {
        let source = r#"
            module test.coverage

            type MyRecord { field: String }
            type MySum = A | B { value: Int }
            type MyAlias = String

            data entries: List<String> = ["a", "b"]

            fn pure_fn(x: Int) -> Int { x }

            func effectful_fn(name: String) -> { ok: Bool }
              uses net: Network
            {
              result = svc.Op(name: name)
              return { ok: true }
            }

            pattern my_pattern(x: String) -> { done: Bool } {
              return { done: true }
            }

            service svc.Example {
              config { endpoint: "https://example.com" }
              operation Op {
                input { name: String }
                output { id: String }
                transport rest { method: POST, path: "/op" }
              }
            }

            resource MyResource {
              kind: Capability
              mode: ReadWrite
              acquire {}
              release {}
            }
        "#;
        let ast = parse_or_panic(source);

        let has = |pred: fn(&Item) -> bool| ast.items.iter().any(|i| pred(&i.node));
        assert!(has(
            |i| matches!(i, Item::TypeDef(d) if matches!(d.body, TypeBody::Record(_)))
        ));
        assert!(has(
            |i| matches!(i, Item::TypeDef(d) if matches!(d.body, TypeBody::Sum(_)))
        ));
        assert!(has(
            |i| matches!(i, Item::TypeDef(d) if matches!(d.body, TypeBody::Alias(_)))
        ));
        assert!(has(|i| matches!(i, Item::DataDef(_))));
        assert!(has(|i| matches!(i, Item::FnDef(_))));
        assert!(has(|i| matches!(i, Item::FuncDef(_))));
        assert!(has(|i| matches!(i, Item::PatternDef(_))));
        assert!(has(|i| matches!(i, Item::ServiceDef(_))));
        assert!(has(|i| matches!(i, Item::ResourceDef(_))));
    }

    // S44: output_parsing annotation on operations
    #[test]
    fn parse_operation_with_output_parsing() {
        let source = r#"
            module services.example
            service cargo.Build {
                operation Check {
                    input { package: String }
                    output { success: Bool, stdout: String, stderr: String }
                    transport shell { argv: ["cargo", "check"] }
                    output_parsing: TrimStdout
                }
            }
        "#;
        let ast = parse_or_panic(source);
        match &ast.items[0].node {
            Item::ServiceDef(def) => {
                assert_eq!(def.operations.len(), 1);
                let op = &def.operations[0];
                assert_eq!(op.output_parsing, Some("TrimStdout".to_string()));
            }
            other => panic!("expected ServiceDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_operation_without_output_parsing_defaults_to_none() {
        let source = r#"
            module services.example
            service cargo.Build {
                operation Check {
                    input { package: String }
                    output { success: Bool }
                    transport shell { argv: ["cargo", "check"] }
                }
            }
        "#;
        let ast = parse_or_panic(source);
        match &ast.items[0].node {
            Item::ServiceDef(def) => {
                assert_eq!(def.operations[0].output_parsing, None);
            }
            other => panic!("expected ServiceDef, got {other:?}"),
        }
    }

    // S45: response_provider in service config
    #[test]
    fn parse_service_config_with_response_provider() {
        let source = r#"
            module services.example
            service custom.MyApi {
                config {
                    endpoint: "https://api.example.com"
                    response_provider: GitHub
                }
                operation Fetch {
                    output { data: String }
                    transport rest { method: GET, path: "/data" }
                }
            }
        "#;
        let ast = parse_or_panic(source);
        match &ast.items[0].node {
            Item::ServiceDef(def) => {
                assert_eq!(def.config.response_provider, Some("GitHub".to_string()));
            }
            other => panic!("expected ServiceDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_service_config_without_response_provider_defaults_to_none() {
        let source = r#"
            module services.example
            service github.Gist {
                config { endpoint: "https://api.github.com" }
                operation Create {
                    output { url: String }
                    transport rest { method: POST, path: "/gists" }
                }
            }
        "#;
        let ast = parse_or_panic(source);
        match &ast.items[0].node {
            Item::ServiceDef(def) => {
                assert_eq!(def.config.response_provider, None);
            }
            other => panic!("expected ServiceDef, got {other:?}"),
        }
    }
}
