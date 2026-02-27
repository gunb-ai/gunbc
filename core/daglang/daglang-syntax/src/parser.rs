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
            TokenKind::Outputs => "outputs",
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
                            } else {
                                self.advance();
                            }
                        }
                        "auth" => {
                            if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                                config.auth = Some(self.expect_ident()?);
                            } else {
                                self.record_err(self.err(
                                    "expected identifier for `auth` (e.g., BearerToken)".to_string(),
                                ));
                                self.advance();
                            }
                        }
                        "auth_input" => {
                            if Self::token_kind_as_ident(&self.peek().kind).is_some() {
                                config.auth_input = Some(self.expect_ident()?);
                            } else {
                                self.record_err(self.err(
                                    "expected identifier for `auth_input` (e.g., auth_token)".to_string(),
                                ));
                                self.advance();
                            }
                        }
                        _ => {
                            self.advance();
                        }
                    }
                }
            } else {
                self.advance();
            }
            self.eat(&TokenKind::Comma);
        }
        Ok(config)
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
                                _ => {
                                    self.advance();
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
                        } else {
                            self.advance();
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
                                _ => {
                                    self.advance();
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
            _ => {
                // Unknown transport kind — consume block and treat as Local
                self.consume_brace_block_contents()?;
                return Ok(TransportBinding::Local);
            }
        };
        self.expect(&TokenKind::RBrace)?;
        Ok(binding)
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
            TokenKind::Service => Item::ServiceDef(self.parse_service_def()?),
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
            TokenKind::Func => Ok(Item::ExternFuncDecl(self.parse_extern_func_decl()?)),
            TokenKind::Ident(s) if s == "asset" => {
                Ok(Item::ExternAssetDecl(self.parse_extern_asset_decl()?))
            }
            _ => Err(self.err("expected `func` or `asset` after `extern`".to_string())),
        }
    }

    fn parse_extern_func_decl(&mut self) -> Result<ExternFuncDecl, ParseError> {
        self.expect(&TokenKind::Func)?;
        let name = self.expect_ident()?;
        self.expect(&TokenKind::LParen)?;
        let inputs = self.parse_field_list_until_rparen()?;
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::Arrow)?;
        let outputs = self.parse_output_fields()?;
        Ok(ExternFuncDecl {
            name,
            inputs,
            outputs,
        })
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
            while !self.check(&TokenKind::RBrace) && !self.at_eof() {
                self.advance();
            }
            self.expect(&TokenKind::RBrace)?;
            return Ok(TypeExpr::Named("Record".into()));
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
            self.parse_fn_body_lossy()?
        } else {
            FnBody {
                stmts: Vec::new(),
                lossy: false,
            }
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
        let body = self.parse_func_body_lossy()?;
        Ok(FuncDef {
            name,
            type_params,
            params,
            outputs,
            uses,
            provides,
            body,
            declared_outputs: Vec::new(),
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
        let body = self.parse_func_body_lossy()?;
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
        let mut hermetic = false;
        let permissions: Vec<String> = Vec::new();
        let mut transport: Option<TransportBinding> = None;
        let mock_response: Vec<MockResponseDef> = Vec::new();

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
                self.advance();
                hermetic = true;
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
                    self.advance();
                    hermetic = true;
                } else if self.check(&TokenKind::Transport) {
                    transport = Some(self.parse_transport_binding()?);
                } else {
                    self.advance();
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
            hermetic,
            permissions,
            transport,
            mock_response,
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
                    let body = self.parse_func_body_lossy()?;
                    acquire = Some(body);
                }
                TokenKind::Release => {
                    self.advance();
                    self.expect(&TokenKind::LBrace)?;
                    let body = self.parse_func_body_lossy()?;
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
        let mock_response: Vec<MockResponseDef> = Vec::new();
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
            mock_response,
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
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if self.check(&TokenKind::Operation) {
                let op = self.parse_operation_def()?;
                capabilities.push(CapabilityDef {
                    name: op.name,
                    inputs: op.inputs,
                    outputs: op.outputs,
                    idempotent: op.idempotent,
                    readonly: op.readonly,
                    mock_response: op.mock_response,
                });
            } else if self.check(&TokenKind::Capability) {
                capabilities.push(self.parse_interface_capability_signature()?);
            } else if self.check(&TokenKind::Fn) {
                capabilities.push(self.parse_interface_fn()?);
            } else if self.check(&TokenKind::Type) {
                let _ = self.parse_type_def();
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
            mock_response: Vec::new(),
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
            mock_response: Vec::new(),
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
        let body = self.parse_func_body_lossy()?;
        Ok(StageDef {
            name,
            body,
            after,
            when,
        })
    }
    // ── SDLC & Infra Blocks ─────────────────────────────────────────

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
    /// `test <name> [: <fixture>] { annotation* (mock | input | expect)* }`
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
        let auto_mock = false;
        let mock_helpers = None;

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

        let mut mocks = Vec::new();
        let mut inputs = Vec::new();
        let mut expects = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            match &self.peek().kind {
                TokenKind::Mock => mocks.push(self.parse_mock_decl()?),
                TokenKind::Input => inputs.push(self.parse_input_decl()?),
                TokenKind::Expect => expects.push(self.parse_expect_stmt()?),
                _ => {
                    return Err(self.err(format!(
                        "expected 'mock', 'input', or 'expect' inside test, found {}",
                        self.peek().kind.desc()
                    )));
                }
            }
        }
        self.expect(&TokenKind::RBrace)?;

        Ok(TestDef {
            name,
            fixture,
            mocks,
            inputs,
            expects,
            tier,
            hermetic,
            skip,
            auto_mock,
            mock_helpers,
        })
    }

    /// Parse a mock target path: `seg1/seg2/.../segN.port` or bare `port`
    ///
    /// Segments before the last `.` are joined with `/` to form the node ID.
    /// The segment after the last `.` is the port name.
    /// If no `.` is present, the identifier is treated as a bare port name
    /// with an empty node path (broadcast-style input).
    fn parse_mock_target(&mut self) -> Result<(Vec<String>, String), ParseError> {
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
        let (node_segments, port) = self.parse_mock_target()?;
        self.expect(&TokenKind::Arrow)?;
        let value = self.parse_expr(0)?;
        Ok(MockDecl {
            node_segments,
            port,
            value,
        })
    }

    /// Parse: `input <target> -> <expr>`
    fn parse_input_decl(&mut self) -> Result<InputDecl, ParseError> {
        self.expect(&TokenKind::Input)?;
        let (node_segments, port) = self.parse_mock_target()?;
        self.expect(&TokenKind::Arrow)?;
        let value = self.parse_expr(0)?;
        Ok(InputDecl {
            node_segments,
            port,
            value,
        })
    }

    /// Parse: `expect <expr> <comparison> <expr>`
    ///        `expect <expr> contains <expr>`
    ///        `expect <expr> is <TypeName>`
    ///        `expect <expr>`
    fn parse_expect_stmt(&mut self) -> Result<ExpectStmt, ParseError> {
        self.expect(&TokenKind::Expect)?;
        let lhs = self.parse_expr(0)?;

        match &self.peek().kind {
            TokenKind::EqEq => {
                self.advance();
                let rhs = self.parse_expr(0)?;
                Ok(ExpectStmt::Eq(lhs, rhs))
            }
            TokenKind::Ne => {
                self.advance();
                let rhs = self.parse_expr(0)?;
                Ok(ExpectStmt::Ne(lhs, rhs))
            }
            TokenKind::Lt => {
                self.advance();
                let rhs = self.parse_expr(0)?;
                Ok(ExpectStmt::Lt(lhs, rhs))
            }
            TokenKind::Gt => {
                self.advance();
                let rhs = self.parse_expr(0)?;
                Ok(ExpectStmt::Gt(lhs, rhs))
            }
            TokenKind::Le => {
                self.advance();
                let rhs = self.parse_expr(0)?;
                Ok(ExpectStmt::Le(lhs, rhs))
            }
            TokenKind::Ge => {
                self.advance();
                let rhs = self.parse_expr(0)?;
                Ok(ExpectStmt::Ge(lhs, rhs))
            }
            TokenKind::Contains => {
                self.advance();
                let rhs = self.parse_expr(0)?;
                Ok(ExpectStmt::Contains(lhs, rhs))
            }
            // `is <TypeName>` — keyword `is` parsed as Ident("is")
            TokenKind::Ident(s) if s == "is" => {
                self.advance();
                let type_name = self.expect_ident()?;
                Ok(ExpectStmt::Is(lhs, type_name))
            }
            _ => {
                // Just a truthiness check
                Ok(ExpectStmt::Truthy(lhs))
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

    fn parse_body_lossy<T>(
        &mut self,
        parse: impl FnOnce(&mut Self) -> Result<T, ParseError>,
        make_lossy: impl FnOnce() -> T,
    ) -> Result<T, ParseError> {
        let start_pos = self.pos;
        let start_errors = self.errors.len();
        let parsed = parse(self);

        let should_fallback = parsed.is_err() || self.errors.len() > start_errors;
        if !should_fallback {
            self.expect(&TokenKind::RBrace)?;
            return parsed;
        }

        self.pos = start_pos;
        self.errors.truncate(start_errors);
        self.consume_brace_block_contents()?;
        Ok(make_lossy())
    }

    fn parse_fn_body_lossy(&mut self) -> Result<FnBody, ParseError> {
        self.parse_body_lossy(Self::parse_fn_body, || FnBody {
            stmts: Vec::new(),
            lossy: true,
        })
    }

    fn parse_func_body_lossy(&mut self) -> Result<FuncBody, ParseError> {
        self.parse_body_lossy(Self::parse_func_body, || FuncBody {
            stmts: Vec::new(),
            lossy: true,
        })
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
            lossy: false,
        })
    }

    fn parse_func_body(&mut self) -> Result<FuncBody, ParseError> {
        Ok(FuncBody {
            stmts: self.parse_stmts()?,
            lossy: false,
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
            Ok(Stmt::Return(vec![("value".into(), expr)]))
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
            TokenKind::PipeArrow => Some((1, 2)),
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
                TokenKind::PipeArrow => Expr::Pipe(Box::new(lhs), Box::new(rhs)),
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

    /// Try to parse a multi-param lambda: `(ident, ident, ...) => body`.
    ///
    /// Called after `(` has been consumed. Uses two-phase approach:
    /// 1. **Lookahead**: scan tokens without mutating parser state to detect
    ///    the `ident, ident, ...) =>` pattern.
    /// 2. **Commit**: if the pattern matches (>= 2 params), advance `self.pos`
    ///    past the params, commas, `)`, and `=>`.
    ///
    /// Returns `Some(params)` on success, `None` if no lambda pattern detected
    /// (parser state unchanged in that case).
    fn try_parse_multi_param_lambda(&mut self) -> Option<Vec<String>> {
        // Phase 1: Lookahead — read-only scan from current position.
        let mut scan = self.pos;
        let first = Self::token_kind_as_ident(&self.tokens.get(scan)?.kind)?;
        let mut params = vec![first];
        scan += 1;

        while scan < self.tokens.len() && self.tokens[scan].kind == TokenKind::Comma {
            scan += 1; // skip comma
            let name = Self::token_kind_as_ident(&self.tokens.get(scan)?.kind)?;
            params.push(name);
            scan += 1; // skip ident
        }

        // Require >= 2 params, followed by `) =>`
        if params.len() < 2 {
            return None;
        }
        if scan >= self.tokens.len() || self.tokens[scan].kind != TokenKind::RParen {
            return None;
        }
        if scan + 1 >= self.tokens.len() || self.tokens[scan + 1].kind != TokenKind::FatArrow {
            return None;
        }

        // Phase 2: Commit — advance past `) =>`
        self.pos = scan + 2;
        Some(params)
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
                    let body = self.parse_expr(0)?;
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
                if let Some(params) = self.try_parse_multi_param_lambda() {
                    let body = self.parse_expr(0)?;
                    Ok(Expr::Lambda(params, Box::new(body)))
                } else {
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
                if self.starts_brace_block_expr() {
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
                    Ok(Expr::Return(vec![("value".into(), e)]))
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

    fn consume_brace_block_expr(&mut self) -> Result<Expr, ParseError> {
        let mut depth = 1usize;
        while depth > 0 && !self.at_eof() {
            match self.peek().kind {
                TokenKind::LBrace => depth += 1,
                TokenKind::RBrace => depth = depth.saturating_sub(1),
                _ => {}
            }
            self.advance();
        }
        if depth != 0 {
            return Err(self.err("unterminated block expression".into()));
        }
        Ok(Expr::Record(None, Vec::new()))
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
            None => Ok(Expr::Call("<expr>".into(), args)),
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
            let body = self.parse_expr(0)?;
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
            TokenKind::Ident(_) => {
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
                        self.expect(&TokenKind::Colon)?;
                        let inner = self.parse_pattern()?;
                        args.push((field, inner));
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
        let then_b = self.parse_expr(0)?;
        self.expect(&TokenKind::RBrace)?;
        let else_b = if self.eat(&TokenKind::Else) {
            self.expect(&TokenKind::LBrace)?;
            let e = self.parse_expr(0)?;
            self.expect(&TokenKind::RBrace)?;
            Some(Box::new(e))
        } else {
            None
        };
        Ok(Expr::If(Box::new(cond), Box::new(then_b), else_b))
    }

    fn parse_for_expr(&mut self) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::For)?;
        let var = self.expect_ident()?;
        self.expect(&TokenKind::In)?;
        let iter = self.parse_for_iterable_expr()?;
        let (iter, passthrough) = self.split_for_iterable_and_passthrough(&var, iter)?;
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_expr(0)?;
        self.expect(&TokenKind::RBrace)?;
        Ok(Expr::For(var, Box::new(iter), passthrough, Box::new(body)))
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
                assert!(!f.body.lossy);
                assert!(matches!(
                    f.body.stmts.first(),
                    Some(Stmt::Expr(Expr::Match(_, arms))) if arms.len() == 2
                ));
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_if_expression_in_fn_body_is_not_lossy() {
        let sf = parse_or_panic(
            r#"module test
fn choose(gate: Bool) -> Int {
  let value = if gate { 1 } else { 0 }
  value
}"#,
        );
        match &sf.items[0].node {
            Item::FnDef(def) => {
                assert!(!def.body.lossy);
                assert!(matches!(
                    def.body.stmts.first(),
                    Some(Stmt::Let(_, Expr::If(_, _, Some(_))))
                ));
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_match_expression_in_fn_body_is_not_lossy() {
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
                assert!(!def.body.lossy);
                assert!(matches!(
                    def.body.stmts.first(),
                    Some(Stmt::Let(_, Expr::Match(_, _)))
                ));
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_fold_pipe_in_fn_body_is_not_lossy() {
        let sf = parse_or_panic(
            r#"module test
type FermiDepth = Xs | S | M | L | Xl
fn fermi_max(lhs: FermiDepth, rhs: FermiDepth) -> FermiDepth {
  lhs
}
fn fermi_max_of(depths: List<FermiDepth>) -> FermiDepth {
  depths |> fold(init: Xs, f: (acc, d) => fermi_max(lhs: acc, rhs: d))
}"#,
        );
        // fermi_max_of should be the third item (after module, type, fermi_max)
        let fn_item = sf
            .items
            .iter()
            .find(|item| matches!(&item.node, Item::FnDef(def) if def.name == "fermi_max_of"))
            .expect("fermi_max_of fn should exist");
        match &fn_item.node {
            Item::FnDef(def) => {
                assert!(
                    !def.body.lossy,
                    "fold pipe expression in fn body should NOT be lossy"
                );
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
    }

    #[test]
    fn parse_classify_transports_body_is_not_lossy() {
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
  depths |> fold(init: Xs, f: (acc, d) => fermi_max(lhs: acc, rhs: d))
}
fn classify_transports(transports: List<TransportClass>) -> DerivedClassification {
  let depths = transports |> map(tc => transport_depth(tc: tc))
  let max_depth = fermi_max_of(depths: depths)
  let all_hermetic = transports |> all(tc => transport_hermetic(tc: tc))
  let n = transports |> count()
  let test_class = if n == 0 { Unit } else { if all_hermetic { Hermetic } else { Integration } }
  let depth = if n == 0 { Xs } else { max_depth }
  let hermetic = if n == 0 { true } else { all_hermetic }
  DerivedClassification { test_class: test_class, depth: depth, hermetic: hermetic }
}"#,
        );
        let fn_item = sf
            .items
            .iter()
            .find(|item| matches!(&item.node, Item::FnDef(def) if def.name == "classify_transports"))
            .expect("classify_transports fn should exist");
        match &fn_item.node {
            Item::FnDef(def) => {
                assert!(
                    !def.body.lossy,
                    "classify_transports fn body should NOT be lossy"
                );
            }
            other => panic!("expected FnDef, got {other:?}"),
        }
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
            r#"module profiles.sdlc
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
    fn expression_pipe_is_left_associative() {
        let expr = parse_expr_only("a |> f |> g");
        match expr {
            Expr::Pipe(lhs, rhs) => {
                assert!(matches!(*rhs, Expr::Ident(ref name) if name == "g"));
                assert!(matches!(*lhs, Expr::Pipe(_, _)));
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
                assert!(matches!(*body, Expr::Call(ref name, _) if name == "render"));
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
                assert!(matches!(*body, Expr::Ident(ref ident) if ident == "item"));
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
    fn parse_extern_func_decl() {
        let source = r#"
            module test.extern_mod
            extern func fetch_data(url: String, timeout: Int) -> { body: String, status: Int }
        "#;
        let ast = parse_or_panic(source);
        assert_eq!(ast.items.len(), 1);
        match &ast.items[0].node {
            Item::ExternFuncDecl(def) => {
                assert_eq!(def.name, "fetch_data");
                assert_eq!(def.inputs.len(), 2);
                assert_eq!(def.inputs[0].name, "url");
                assert_eq!(def.inputs[1].name, "timeout");
                assert_eq!(def.outputs.len(), 2);
                assert_eq!(def.outputs[0].name, "body");
                assert_eq!(def.outputs[1].name, "status");
            }
            other => panic!("expected ExternFuncDecl, got {other:?}"),
        }
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
}
