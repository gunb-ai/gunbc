use crate::ast::*;
use crate::lexer::{Lexer, Token, TokenKind};
use crate::span::{Span, Spanned};

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

pub fn parse(source: &str) -> Result<SourceFile, Vec<ParseError>> {
    let tokens = Lexer::tokenize(source);
    let mut p = Parser::new(tokens);
    p.parse_source_file()
}

pub fn parse_or_panic(source: &str) -> SourceFile {
    parse(source).unwrap_or_else(|errs| {
        for e in &errs {
            eprintln!("{e}");
        }
        panic!("parse failed with {} error(s)", errs.len());
    })
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
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
        match &self.peek().kind {
            TokenKind::Ident(s) => {
                let s = s.clone();
                self.advance();
                Ok(s)
            }
            _ => Err(self.err(format!(
                "expected identifier, found {}",
                self.peek().kind.desc()
            ))),
        }
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
                | TokenKind::Pipeline => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn skip_annotation_value(&mut self) {
        let mut depth: i32 = 0;
        loop {
            if self.at_eof() {
                return;
            }
            match &self.peek().kind {
                TokenKind::LBrace | TokenKind::LParen | TokenKind::LBracket => {
                    depth += 1;
                    self.advance();
                }
                TokenKind::RBrace | TokenKind::RParen | TokenKind::RBracket => {
                    if depth <= 0 {
                        return;
                    }
                    depth -= 1;
                    self.advance();
                }
                TokenKind::At if depth == 0 => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn skip_block_contents(&mut self) {
        let mut depth = 0i32;
        while !self.at_eof() {
            match &self.peek().kind {
                TokenKind::LBrace => {
                    depth += 1;
                    self.advance();
                }
                TokenKind::RBrace => {
                    if depth <= 0 {
                        return;
                    }
                    depth -= 1;
                    self.advance();
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn end_span(&self, start: Span) -> Span {
        let end = self.tokens[self.pos.saturating_sub(1)].span;
        Span {
            start: start.start,
            end: end.end,
        }
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
            let mut leading_anns = Vec::new();
            while self.check(&TokenKind::At) {
                match self.parse_annotation() {
                    Ok(a) => leading_anns.push(a),
                    Err(e) => {
                        self.record_err(e);
                        self.skip_annotation_value();
                    }
                }
            }
            if self.at_eof() {
                break;
            }
            match self.parse_item(leading_anns) {
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
        let mut segments = vec![self.expect_ident()?];
        while self.eat(&TokenKind::Dot) {
            segments.push(self.expect_ident()?);
        }
        Ok(Spanned {
            node: ModulePath { segments },
            span: self.end_span(start),
        })
    }

    fn parse_import(&mut self) -> Result<Spanned<Import>, ParseError> {
        let start = self.span();
        self.expect(&TokenKind::Import)?;
        let mut segments = vec![self.expect_ident()?];
        while self.eat(&TokenKind::Dot) {
            segments.push(self.expect_ident()?);
        }
        let path = ModulePath { segments };
        let mut bindings = None;
        let mut alias = None;
        if self.eat(&TokenKind::LBrace) {
            let mut bs = Vec::new();
            if !self.check(&TokenKind::RBrace) {
                bs.push(self.expect_ident()?);
                while self.eat(&TokenKind::Comma) {
                    bs.push(self.expect_ident()?);
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

    fn parse_item(&mut self, leading_anns: Vec<Annotation>) -> Result<Spanned<Item>, ParseError> {
        let start = self.span();
        let item = match &self.peek().kind {
            TokenKind::Type => Item::TypeDef(self.parse_type_def()?),
            TokenKind::Fn => Item::FnDef(self.parse_fn_def()?),
            TokenKind::Func => Item::FuncDef(self.parse_func_def(leading_anns)?),
            TokenKind::Pattern => Item::PatternDef(self.parse_pattern_def()?),
            TokenKind::Service => Item::ServiceDef(self.parse_service_def(leading_anns)?),
            TokenKind::Resource => Item::ResourceDef(self.parse_resource_def()?),
            TokenKind::Interface => Item::InterfaceDef(self.parse_interface_def()?),
            TokenKind::Pipeline => Item::PipelineDef(self.parse_pipeline_def()?),
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
                let _ = self.expect_ident();
                while self.eat(&TokenKind::Plus) {
                    let _ = self.expect_ident();
                }
            }
            while self.eat(&TokenKind::Comma) {
                params.push(self.expect_ident()?);
                if self.eat(&TokenKind::Colon) {
                    let _ = self.expect_ident();
                    while self.eat(&TokenKind::Plus) {
                        let _ = self.expect_ident();
                    }
                }
            }
        }
        self.expect(&TokenKind::Gt)?;
        Ok(params)
    }

    // ── type expressions ───────────────────────────────────────────

    fn parse_type_expr(&mut self) -> Result<TypeExpr, ParseError> {
        let name = self.expect_ident()?;
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
        } else {
            TypeExpr::Named(name)
        };

        if self.eat(&TokenKind::Question) {
            ty = TypeExpr::Optional(Box::new(ty));
        }

        let mut anns = Vec::new();
        while self.check(&TokenKind::At) {
            anns.push(self.parse_annotation()?);
        }
        if !anns.is_empty() {
            ty = TypeExpr::Annotated(Box::new(ty), anns);
        }

        Ok(ty)
    }

    // ── fn / func / pattern ────────────────────────────────────────

    fn parse_fn_def(&mut self) -> Result<FnDef, ParseError> {
        self.expect(&TokenKind::Fn)?;
        let name = self.expect_ident()?;
        let _tp = self.parse_optional_type_params()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::Arrow)?;
        let return_type = self.parse_type_expr()?;
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_fn_body()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(FnDef {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_func_def(&mut self, leading: Vec<Annotation>) -> Result<FuncDef, ParseError> {
        self.expect(&TokenKind::Func)?;
        let name = self.expect_ident()?;
        let _tp = self.parse_optional_type_params()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::Arrow)?;
        let outputs = self.parse_output_fields()?;
        let (uses, provides) = self.parse_uses_provides()?;
        let mut annotations = leading;
        while self.check(&TokenKind::At) {
            annotations.push(self.parse_annotation()?);
        }
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_func_body()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(FuncDef {
            name,
            params,
            outputs,
            uses,
            provides,
            annotations,
            body,
        })
    }

    fn parse_pattern_def(&mut self) -> Result<PatternDef, ParseError> {
        self.expect(&TokenKind::Pattern)?;
        let name = self.expect_ident()?;
        let _tp = self.parse_optional_type_params()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(&TokenKind::RParen)?;
        self.expect(&TokenKind::Arrow)?;
        let outputs = self.parse_output_fields()?;
        let (uses, _provides) = self.parse_uses_provides()?;
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_func_body()?;
        self.expect(&TokenKind::RBrace)?;
        Ok(PatternDef {
            name,
            params,
            outputs,
            uses,
            body,
        })
    }

    fn parse_output_fields(&mut self) -> Result<Vec<Field>, ParseError> {
        if !self.eat(&TokenKind::LBrace) {
            return Ok(Vec::new());
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

    fn parse_service_def(&mut self, leading: Vec<Annotation>) -> Result<ServiceDef, ParseError> {
        self.expect(&TokenKind::Service)?;
        let mut name = self.expect_ident()?;
        while self.eat(&TokenKind::Dot) {
            name.push('.');
            name.push_str(&self.expect_ident()?);
        }
        let implements = if self.eat(&TokenKind::Colon) {
            Some(self.expect_ident()?)
        } else {
            None
        };
        self.expect(&TokenKind::LBrace)?;
        let mut annotations = leading;
        let mut operations = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if self.check(&TokenKind::At) {
                annotations.push(self.parse_annotation()?);
            } else if self.check(&TokenKind::Operation) {
                operations.push(self.parse_operation_def()?);
            } else if self.check(&TokenKind::Config) {
                self.advance();
                self.expect(&TokenKind::LBrace)?;
                self.skip_block_contents();
                self.expect(&TokenKind::RBrace)?;
            } else {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(ServiceDef {
            name,
            implements,
            annotations,
            operations,
        })
    }

    fn parse_operation_def(&mut self) -> Result<OperationDef, ParseError> {
        self.expect(&TokenKind::Operation)?;
        let name = self.expect_ident()?;
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();
        let mut annotations = Vec::new();

        if self.eat(&TokenKind::LParen) {
            inputs = self.parse_field_list_until_rparen()?;
            self.expect(&TokenKind::RParen)?;
        }
        if self.eat(&TokenKind::Arrow) {
            if self.eat(&TokenKind::LBrace) {
                outputs = self.parse_field_list_until_rbrace()?;
                self.expect(&TokenKind::RBrace)?;
            }
        }
        while self.check(&TokenKind::At) {
            annotations.push(self.parse_annotation()?);
        }
        if self.eat(&TokenKind::LBrace) {
            while !self.check(&TokenKind::RBrace) && !self.at_eof() {
                if self.check(&TokenKind::At) {
                    annotations.push(self.parse_annotation()?);
                } else if self.check(&TokenKind::Input) {
                    self.advance();
                    self.expect(&TokenKind::LBrace)?;
                    inputs = self.parse_field_list_until_rbrace()?;
                    self.expect(&TokenKind::RBrace)?;
                } else if self.check(&TokenKind::Output) {
                    self.advance();
                    self.expect(&TokenKind::LBrace)?;
                    outputs = self.parse_field_list_until_rbrace()?;
                    self.expect(&TokenKind::RBrace)?;
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
            annotations,
        })
    }

    fn parse_resource_def(&mut self) -> Result<ResourceDef, ParseError> {
        self.expect(&TokenKind::Resource)?;
        let name = self.expect_ident()?;
        let implements = if self.eat(&TokenKind::Colon) {
            Some(self.expect_ident()?)
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
                TokenKind::At => {
                    let _ = self.parse_annotation();
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
        let mut annotations = Vec::new();
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
            } else if self.check(&TokenKind::At) {
                annotations.push(self.parse_annotation()?);
            } else {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(CapabilityDef {
            name,
            inputs,
            outputs,
            annotations,
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
        let mut contracts = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if self.check(&TokenKind::Operation) {
                let op = self.parse_operation_def()?;
                capabilities.push(CapabilityDef {
                    name: op.name,
                    inputs: op.inputs,
                    outputs: op.outputs,
                    annotations: op.annotations,
                });
            } else if self.check(&TokenKind::Fn) {
                capabilities.push(self.parse_interface_fn()?);
            } else if self.check(&TokenKind::Type) {
                let _ = self.parse_type_def();
            } else if self.check(&TokenKind::At) {
                let ann = self.parse_annotation()?;
                contracts.push(ann);
            } else {
                self.advance();
            }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(InterfaceDef {
            name,
            type_params,
            capabilities,
            contracts,
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
                annotations: Vec::new(),
            })
            .collect();
        let mut outputs = Vec::new();
        if self.eat(&TokenKind::Arrow) {
            let ret = self.parse_type_expr()?;
            outputs.push(Field {
                name: "return".into(),
                ty: ret,
                default: None,
                annotations: Vec::new(),
            });
        }
        let mut annotations = Vec::new();
        while self.check(&TokenKind::At) {
            annotations.push(self.parse_annotation()?);
        }
        Ok(CapabilityDef {
            name,
            inputs,
            outputs,
            annotations,
        })
    }

    fn parse_pipeline_def(&mut self) -> Result<PipelineDef, ParseError> {
        self.expect(&TokenKind::Pipeline)?;
        let name = self.expect_ident()?;
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
        Ok(PipelineDef { name, stages })
    }

    fn parse_stage_def(&mut self) -> Result<StageDef, ParseError> {
        self.expect(&TokenKind::Stage)?;
        let name = self.expect_ident()?;
        let mut after = Vec::new();
        if self.eat(&TokenKind::LBracket) {
            while !self.check(&TokenKind::RBracket) && !self.at_eof() {
                if self.eat(&TokenKind::After) {
                    after.push(self.expect_ident()?);
                } else if self.check(&TokenKind::When) {
                    self.advance();
                    let _ = self.parse_expr(0);
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
        Ok(StageDef { name, body, after })
    }

    // ── fields / params ────────────────────────────────────────────

    fn parse_field_list_until_rbrace(&mut self) -> Result<Vec<Field>, ParseError> {
        let mut fields = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if self.check(&TokenKind::At) {
                let _ = self.parse_annotation();
                continue;
            }
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
        let mut annotations = Vec::new();
        while self.check(&TokenKind::At) {
            annotations.push(self.parse_annotation()?);
        }
        Ok(Field {
            name,
            ty,
            default,
            annotations,
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
        Ok(Param {
            name,
            ty,
            default,
        })
    }

    // ── annotations ────────────────────────────────────────────────

    fn parse_annotation(&mut self) -> Result<Annotation, ParseError> {
        self.expect(&TokenKind::At)?;
        let name = self.expect_ident()?;
        let mut args = Vec::new();

        if self.eat(&TokenKind::LParen) {
            if !self.check(&TokenKind::RParen) {
                if matches!(&self.peek().kind, TokenKind::Ident(_))
                    && self.peek2().kind == TokenKind::Colon
                {
                    let rec = self.parse_annotation_named_args()?;
                    args.push(rec);
                } else {
                    args.push(self.parse_expr(0)?);
                    while self.eat(&TokenKind::Comma) {
                        if self.check(&TokenKind::RParen) {
                            break;
                        }
                        args.push(self.parse_expr(0)?);
                    }
                }
            }
            self.expect(&TokenKind::RParen)?;
        }

        if self.eat(&TokenKind::LBrace) {
            let rec = self.parse_record_like_block()?;
            args.push(rec);
            self.expect(&TokenKind::RBrace)?;
        }

        if self.eat(&TokenKind::Colon) {
            match self.parse_expr(0) {
                Ok(e) => args.push(e),
                Err(_) => {}
            }
            self.skip_annotation_value();
        }

        Ok(Annotation { name, args })
    }

    fn parse_annotation_named_args(&mut self) -> Result<Expr, ParseError> {
        let mut fields = Vec::new();
        loop {
            let k = self.expect_ident()?;
            self.expect(&TokenKind::Colon)?;
            let v = self.parse_expr(0)?;
            fields.push((k, v));
            if !self.eat(&TokenKind::Comma) {
                break;
            }
            if self.check(&TokenKind::RParen) {
                break;
            }
        }
        Ok(Expr::Record(None, fields))
    }

    fn parse_record_like_block(&mut self) -> Result<Expr, ParseError> {
        let mut fields = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            if let TokenKind::Ident(_) = &self.peek().kind {
                if self.peek2().kind == TokenKind::Colon {
                    let k = self.expect_ident()?;
                    self.expect(&TokenKind::Colon)?;
                    let v = self.parse_expr(0)?;
                    fields.push((k, v));
                    self.eat(&TokenKind::Comma);
                    continue;
                }
            }
            self.advance();
        }
        Ok(Expr::Record(None, fields))
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
                        if matches!(self.peek().kind, TokenKind::Ident(_))
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
                        _ => None,
                    })
                    .collect(),
            )));
        }
        if let TokenKind::Ident(_) = &self.peek().kind {
            if self.peek2().kind == TokenKind::Eq {
                let name = self.expect_ident()?;
                self.expect(&TokenKind::Eq)?;
                let expr = self.parse_expr(0)?;
                return Ok(Stmt::Assign(name, expr));
            }
        }
        let expr = self.parse_expr(0)?;
        Ok(Stmt::Expr(expr))
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
        if self.eat(&TokenKind::LBracket) {
            while !self.check(&TokenKind::RBracket) && !self.at_eof() {
                self.advance();
            }
            self.eat(&TokenKind::RBracket);
        }
        if self.eat(&TokenKind::Colon) || self.eat(&TokenKind::Eq) {
            let expr = self.parse_expr(0)?;
            return Ok(Stmt::Assign(name, expr));
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
            if self.check(&TokenKind::LParen) {
                if 21u8 < min_bp {
                    break;
                }
                lhs = self.parse_call_expr(lhs)?;
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
                    Expr::BinOp(Box::new(lhs), BinOp::Or, Box::new(rhs))
                }
                _ => Expr::BinOp(Box::new(lhs), BinOp::Add, Box::new(rhs)),
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
            TokenKind::Ident(name) => {
                if self.peek2().kind == TokenKind::FatArrow {
                    self.advance();
                    self.advance();
                    let body = self.parse_expr(0)?;
                    return Ok(Expr::Lambda(vec![name], Box::new(body)));
                }
                self.advance();
                Ok(Expr::Ident(name))
            }
            TokenKind::SelfKw => {
                self.advance();
                Ok(Expr::Ident("self".into()))
            }
            TokenKind::LParen => {
                self.advance();
                if self.check(&TokenKind::RParen) {
                    self.advance();
                    return Ok(Expr::Record(None, Vec::new()));
                }
                let expr = self.parse_expr(0)?;
                self.expect(&TokenKind::RParen)?;
                Ok(expr)
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
                if matches!(&self.peek().kind, TokenKind::Ident(_))
                    && self.peek2().kind == TokenKind::Colon
                {
                    let mut fields = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.at_eof() {
                        let k = self.expect_ident()?;
                        self.expect(&TokenKind::Colon)?;
                        let v = self.parse_expr(0)?;
                        fields.push((k, v));
                        self.eat(&TokenKind::Comma);
                    }
                    self.expect(&TokenKind::RBrace)?;
                    Ok(Expr::Record(None, fields))
                } else if matches!(
                    &self.peek().kind,
                    TokenKind::Str(_) | TokenKind::StrBegin(_)
                ) {
                    let mut entries = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.at_eof() {
                        let key = self.parse_expr(0)?;
                        self.expect(&TokenKind::Colon)?;
                        let val = self.parse_expr(0)?;
                        entries.push((key, val));
                        self.eat(&TokenKind::Comma);
                    }
                    self.expect(&TokenKind::RBrace)?;
                    Ok(Expr::Map(entries))
                } else {
                    let expr = self.parse_expr(0)?;
                    self.expect(&TokenKind::RBrace)?;
                    Ok(expr)
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

    fn parse_call_expr(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::LParen)?;
        let mut args: Vec<(Option<String>, Expr)> = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                if matches!(&self.peek().kind, TokenKind::Ident(_))
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
        let scrutinee = self.parse_expr(0)?;
        self.expect(&TokenKind::LBrace)?;
        let mut arms = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.at_eof() {
            let pattern = self.parse_pattern()?;
            let guard = if self.eat(&TokenKind::When) {
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
            TokenKind::Ident(name)
                if name
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_uppercase()) =>
            {
                self.advance();
                if self.eat(&TokenKind::LParen) {
                    let mut fields = Vec::new();
                    while !self.check(&TokenKind::RParen) && !self.at_eof() {
                        let fname = self.expect_ident()?;
                        self.expect(&TokenKind::Colon)?;
                        let pat = self.parse_pattern()?;
                        fields.push((fname, pat));
                        self.eat(&TokenKind::Comma);
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Pattern::Variant(name, fields))
                } else {
                    Ok(Pattern::Variant(name, Vec::new()))
                }
            }
            TokenKind::Ident(name) => {
                self.advance();
                Ok(Pattern::Ident(name))
            }
            TokenKind::SelfKw => {
                self.advance();
                Ok(Pattern::Ident("self".into()))
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
            TokenKind::Str(s) => {
                self.advance();
                Ok(Pattern::Literal(Literal::String(s)))
            }
            _ => Ok(Pattern::Wildcard),
        }
    }

    fn parse_if_expr(&mut self) -> Result<Expr, ParseError> {
        self.expect(&TokenKind::If)?;
        let cond = self.parse_expr(0)?;
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
        let iter = self.parse_expr(0)?;
        self.expect(&TokenKind::LBrace)?;
        let body = self.parse_expr(0)?;
        self.expect(&TokenKind::RBrace)?;
        Ok(Expr::For(var, Box::new(iter), Box::new(body)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            Item::FnDef(f) => assert_eq!(f.name, "greet"),
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
}
