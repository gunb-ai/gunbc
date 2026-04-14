// Surface AST + hand-recursive parser for the v3 surface grammar.
//
// G3 guardrail: parse.rs exports SurfaceModule / SurfaceItem / SurfaceExpr /
// SurfaceType; it does NOT mention Dag or any L1 behavior type. Lowering from
// surface to DAG happens in lower.rs.
//
// Operators compile to identifier-shaped Calls per M1_DESIGN.md §8.9 Option A:
// `1 + 2` → Call { target: "+", args: [1, 2] }. Resolution to the concrete Arrow
// happens later during inference via inhabitance walks (not at parse time).
//
// Grammar (M1(2.5)):
//   module     := item*
//   item       := let_item | fn_item | type_item
//   let_item   := `let` ident (`:` type_expr)? `=` expr
//   fn_item    := `fn` ident `(` params `)` `->` type_expr `=` expr
//   type_item  := `type` ident type_params? type_body?
//   type_body  := `{` record_fields `}`                       -- TypeRecord
//              |  `=` ( sum_variants | type_expr )             -- TypeSum | TypeAlias
//                                                              -- (no body) TypeAtom
//   type_params := `<` ident ( `,` ident )* `>`
//   type_expr  := atom_type ( `?` )?
//   atom_type  := ident type_args?                             -- Named | Parameterized
//              |  `fn` `(` type_expr_list `)` `->` type_expr   -- Arrow
//   type_args  := `<` type_expr ( `,` type_expr )* `>`
//   record_fields := field_decl*                               -- whitespace-separated
//   field_decl := ident `:` type_expr (`,` | `;`)?
//   sum_variants := variant ( `|` variant )*
//   variant    := ident ( `(` type_expr_list `)` )?
//   expr       := comparison
//   comparison := additive ( cmp_op additive )?
//   additive   := term ( (`+` | `-`) term )*
//   term       := primary ( (`*` | `/`) primary )*
//   primary    := int_lit | bool_lit | string_lit
//              |  ident ( `(` args `)` )?
//              |  `if` expr `then` expr `else` expr

use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::tokenize::{Token, TokenKind};

#[derive(Debug, Clone)]
pub struct SurfaceModule {
    pub items: Vec<SurfaceItem>,
}

#[derive(Debug, Clone)]
pub enum SurfaceItem {
    Let {
        name: String,
        type_ann: Option<SurfaceType>,
        expr: SurfaceExpr,
    },
    /// Function definition. `body` is `Some` for expression-body form
    /// (`fn f(x) -> T = expr`); `None` for block-body form
    /// (`fn f(x) -> T { body }`) where the body is consumed opaquely as
    /// a brace-balanced token range. Block-body fn items lower to an
    /// `ArrowBody::Pending` declaration with no computation sub-DAG —
    /// their bodies are out of scope until the full surface grammar
    /// (match/pipe/lambda/named-args) lands in M2+.
    Fn {
        name: String,
        params: Vec<SurfaceParam>,
        return_type: SurfaceType,
        body: Option<SurfaceExpr>,
        span: SourceSpan,
    },
    TypeAtom {
        name: String,
        #[allow(dead_code)]
        type_params: Vec<String>,
        span: SourceSpan,
    },
    TypeRecord {
        name: String,
        type_params: Vec<String>,
        fields: Vec<SurfaceField>,
        span: SourceSpan,
    },
    TypeSum {
        name: String,
        type_params: Vec<String>,
        variants: Vec<SurfaceVariant>,
        span: SourceSpan,
    },
    TypeAlias {
        name: String,
        type_params: Vec<String>,
        target: SurfaceType,
        span: SourceSpan,
    },
    /// `module std.foo` — no-op item. Captured for span purposes; no
    /// semantic effect on the declaration graph. Lowering discards.
    Module {
        #[allow(dead_code)]
        path: Vec<String>,
        span: SourceSpan,
    },
    /// `import std.foo { Name1, Name2 }` — no-op item. Name resolution
    /// against the full bootstrap declaration table happens via
    /// `resolve_pending_identifiers`, so explicit import lists are
    /// unnecessary at M1(2.6); the declaration would still appear in
    /// `declaration_by_name` regardless of whether the current file
    /// listed it in its imports.
    Import {
        #[allow(dead_code)]
        path: Vec<String>,
        #[allow(dead_code)]
        names: Vec<String>,
        span: SourceSpan,
    },
    /// `data name: Type = { body }` — the body is consumed opaquely as
    /// a brace-balanced token range. Lowering produces a placeholder
    /// Declaration whose connective is a bare `Conj` with empty
    /// children; the actual body contents (record literals, list
    /// literals, etc.) are M2+ work.
    DataDecl {
        name: String,
        #[allow(dead_code)]
        ty: SurfaceType,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone)]
pub struct SurfaceParam {
    pub name: String,
    pub ty: SurfaceType,
}

#[derive(Debug, Clone)]
pub struct SurfaceField {
    pub name: String,
    pub ty: SurfaceType,
}

#[derive(Debug, Clone)]
pub struct SurfaceVariant {
    pub name: String,
    pub payload: VariantPayload,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub enum VariantPayload {
    /// Unit variant — no payload, e.g. `True` in `type Classical = True | False`.
    Unit,
    /// Positional payload — e.g. `Ok(T)` in `type Result<T, E> = Ok(T) | Err(E)`.
    Positional(Vec<SurfaceType>),
    /// Record-style payload — e.g. `WorkloadIdentity { audience: NonEmptyStr, ... }`.
    Record(Vec<SurfaceField>),
}

#[derive(Debug, Clone)]
pub enum SurfaceType {
    Named {
        name: String,
        span: SourceSpan,
    },
    Parameterized {
        name: String,
        args: Vec<SurfaceType>,
        span: SourceSpan,
    },
    Optional {
        inner: Box<SurfaceType>,
        span: SourceSpan,
    },
    Arrow {
        inputs: Vec<SurfaceType>,
        output: Box<SurfaceType>,
        span: SourceSpan,
    },
}

impl SurfaceType {
    pub fn span(&self) -> &SourceSpan {
        match self {
            SurfaceType::Named { span, .. }
            | SurfaceType::Parameterized { span, .. }
            | SurfaceType::Optional { span, .. }
            | SurfaceType::Arrow { span, .. } => span,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SurfaceExpr {
    IntLit {
        value: i64,
        span: SourceSpan,
    },
    BoolLit {
        value: bool,
        span: SourceSpan,
    },
    StringLit {
        value: String,
        span: SourceSpan,
    },
    Var {
        name: String,
        span: SourceSpan,
    },
    Call {
        target: String,
        args: Vec<SurfaceExpr>,
        span: SourceSpan,
    },
    If {
        cond: Box<SurfaceExpr>,
        then_branch: Box<SurfaceExpr>,
        else_branch: Box<SurfaceExpr>,
        span: SourceSpan,
    },
}

pub fn parse(tokens: &[Token], file: &str) -> Result<SurfaceModule, Diagnostic> {
    let mut parser = Parser {
        tokens,
        pos: 0,
        file,
    };
    let mut items = Vec::new();
    while !parser.at_eof() {
        items.push(parser.parse_item()?);
    }
    Ok(SurfaceModule { items })
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    file: &'a str,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn bump(&mut self) -> &Token {
        let t = &self.tokens[self.pos];
        self.pos += 1;
        t
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Eof)
    }

    fn expect_kind(&mut self, expected: TokenKind) -> Result<Token, Diagnostic> {
        let token = self.bump().clone();
        if token.kind == expected {
            Ok(token)
        } else {
            Err(Diagnostic::ParseError {
                message: format!("expected {expected:?}, got {:?}", token.kind),
                span: token.span,
            })
        }
    }

    fn parse_item(&mut self) -> Result<SurfaceItem, Diagnostic> {
        match &self.peek().kind {
            TokenKind::KwLet => self.parse_let_item(),
            TokenKind::KwFn => self.parse_fn_item(),
            TokenKind::KwType => self.parse_type_item(),
            TokenKind::KwModule => self.parse_module_item(),
            TokenKind::KwImport => self.parse_import_item(),
            TokenKind::KwData => self.parse_data_item(),
            other => Err(Diagnostic::ParseError {
                message: format!(
                    "expected `let`, `fn`, `type`, `module`, `import`, or `data`, got {other:?}"
                ),
                span: self.peek().span.clone(),
            }),
        }
    }

    fn parse_module_item(&mut self) -> Result<SurfaceItem, Diagnostic> {
        let module_kw = self.expect_kind(TokenKind::KwModule)?;
        let path = self.parse_dotted_path()?;
        let end = self
            .tokens
            .get(self.pos.saturating_sub(1))
            .map(|t| t.span.byte_end)
            .unwrap_or(module_kw.span.byte_end);
        Ok(SurfaceItem::Module {
            path,
            span: SourceSpan::new(self.file, module_kw.span.byte_start, end),
        })
    }

    fn parse_import_item(&mut self) -> Result<SurfaceItem, Diagnostic> {
        let import_kw = self.expect_kind(TokenKind::KwImport)?;
        let path = self.parse_dotted_path()?;
        let mut names = Vec::new();
        let end = if matches!(self.peek().kind, TokenKind::LBrace) {
            self.bump();
            if !matches!(self.peek().kind, TokenKind::RBrace) {
                loop {
                    names.push(self.parse_ident()?);
                    if matches!(self.peek().kind, TokenKind::Comma) {
                        self.bump();
                        continue;
                    }
                    break;
                }
            }
            let close = self.expect_kind(TokenKind::RBrace)?;
            close.span.byte_end
        } else {
            self.tokens
                .get(self.pos.saturating_sub(1))
                .map(|t| t.span.byte_end)
                .unwrap_or(import_kw.span.byte_end)
        };
        Ok(SurfaceItem::Import {
            path,
            names,
            span: SourceSpan::new(self.file, import_kw.span.byte_start, end),
        })
    }

    fn parse_data_item(&mut self) -> Result<SurfaceItem, Diagnostic> {
        let data_kw = self.expect_kind(TokenKind::KwData)?;
        let name = self.parse_ident()?;
        self.expect_kind(TokenKind::Colon)?;
        let ty = self.parse_type_expr()?;
        self.expect_kind(TokenKind::Eq)?;
        // Body is brace-balanced and consumed opaquely — data value
        // semantics (record literals, keys, etc.) are M2+ work.
        let end = self.skip_brace_balanced()?;
        Ok(SurfaceItem::DataDecl {
            name,
            ty,
            span: SourceSpan::new(self.file, data_kw.span.byte_start, end),
        })
    }

    fn parse_dotted_path(&mut self) -> Result<Vec<String>, Diagnostic> {
        let mut path = vec![self.parse_ident()?];
        while matches!(self.peek().kind, TokenKind::Dot) {
            self.bump();
            path.push(self.parse_ident()?);
        }
        Ok(path)
    }

    /// Consume a brace-balanced token range starting at the current
    /// `{` and returning the byte offset of the matching `}`. Used for
    /// opaque fn/data bodies at M1(2.6). Errors if EOF is reached
    /// before the braces balance.
    fn skip_brace_balanced(&mut self) -> Result<u32, Diagnostic> {
        let open = self.expect_kind(TokenKind::LBrace)?;
        let mut depth: i32 = 1;
        loop {
            if self.at_eof() {
                return Err(Diagnostic::ParseError {
                    message: "unterminated block body: reached EOF before closing `}`".to_string(),
                    span: open.span,
                });
            }
            let token = self.bump().clone();
            match token.kind {
                TokenKind::LBrace => depth += 1,
                TokenKind::RBrace => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(token.span.byte_end);
                    }
                }
                _ => {}
            }
        }
    }

    fn parse_let_item(&mut self) -> Result<SurfaceItem, Diagnostic> {
        self.expect_kind(TokenKind::KwLet)?;
        let name = self.parse_ident()?;
        let type_ann = if matches!(self.peek().kind, TokenKind::Colon) {
            self.bump();
            Some(self.parse_type_expr()?)
        } else {
            None
        };
        self.expect_kind(TokenKind::Eq)?;
        let expr = self.parse_expr()?;
        Ok(SurfaceItem::Let {
            name,
            type_ann,
            expr,
        })
    }

    fn parse_fn_item(&mut self) -> Result<SurfaceItem, Diagnostic> {
        let fn_kw = self.expect_kind(TokenKind::KwFn)?;
        let name = self.parse_ident()?;
        self.expect_kind(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect_kind(TokenKind::RParen)?;
        self.expect_kind(TokenKind::Arrow)?;
        let return_type = self.parse_type_expr()?;
        match &self.peek().kind {
            TokenKind::Eq => {
                // Expression-body form: `fn f(x) -> T = expr`.
                self.bump();
                let body_expr = self.parse_expr()?;
                let end = expr_span(&body_expr).byte_end;
                Ok(SurfaceItem::Fn {
                    name,
                    params,
                    return_type,
                    body: Some(body_expr),
                    span: SourceSpan::new(self.file, fn_kw.span.byte_start, end),
                })
            }
            TokenKind::LBrace => {
                // Block-body form: `fn f(x) -> T { body }`. Body is
                // consumed opaquely; match/pipe/lambda/named-args/etc.
                // inside the body are out of scope at M1(2.6).
                let end = self.skip_brace_balanced()?;
                Ok(SurfaceItem::Fn {
                    name,
                    params,
                    return_type,
                    body: None,
                    span: SourceSpan::new(self.file, fn_kw.span.byte_start, end),
                })
            }
            other => Err(Diagnostic::ParseError {
                message: format!(
                    "expected `=` or `{{` after fn return type, got {other:?}"
                ),
                span: self.peek().span.clone(),
            }),
        }
    }

    fn parse_type_item(&mut self) -> Result<SurfaceItem, Diagnostic> {
        let type_kw = self.expect_kind(TokenKind::KwType)?;
        let name_token = self.bump().clone();
        let name = match &name_token.kind {
            TokenKind::Ident(n) => n.clone(),
            other => {
                return Err(Diagnostic::ParseError {
                    message: format!("expected type name, got {other:?}"),
                    span: name_token.span.clone(),
                });
            }
        };
        let type_params = self.parse_optional_type_params()?;

        match &self.peek().kind {
            TokenKind::LBrace => {
                self.bump();
                let fields = self.parse_record_fields()?;
                let close = self.expect_kind(TokenKind::RBrace)?;
                Ok(SurfaceItem::TypeRecord {
                    name,
                    type_params,
                    fields,
                    span: SourceSpan::new(
                        self.file,
                        type_kw.span.byte_start,
                        close.span.byte_end,
                    ),
                })
            }
            TokenKind::Eq => {
                self.bump();
                self.parse_type_rhs_after_eq(name, type_params, type_kw.span)
            }
            _ => Ok(SurfaceItem::TypeAtom {
                name,
                type_params,
                span: SourceSpan::new(
                    self.file,
                    type_kw.span.byte_start,
                    name_token.span.byte_end,
                ),
            }),
        }
    }

    fn parse_optional_type_params(&mut self) -> Result<Vec<String>, Diagnostic> {
        if !matches!(self.peek().kind, TokenKind::Lt) {
            return Ok(Vec::new());
        }
        self.bump();
        let mut params = vec![self.parse_ident()?];
        while matches!(self.peek().kind, TokenKind::Comma) {
            self.bump();
            params.push(self.parse_ident()?);
        }
        self.expect_kind(TokenKind::Gt)?;
        Ok(params)
    }

    fn parse_record_fields(&mut self) -> Result<Vec<SurfaceField>, Diagnostic> {
        let mut fields = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RBrace) {
            let name = self.parse_ident()?;
            self.expect_kind(TokenKind::Colon)?;
            let ty = self.parse_type_expr()?;
            fields.push(SurfaceField { name, ty });
            if matches!(
                self.peek().kind,
                TokenKind::Comma | TokenKind::Semicolon
            ) {
                self.bump();
            }
        }
        Ok(fields)
    }

    /// After `type Name<T> =`, decide between TypeSum (one-or-more variants
    /// separated by `|`) and TypeAlias (a single type expression). A variant
    /// looks like `Ident` or `Ident(payload)`. A type expression can start
    /// with `Ident<...>` (parameterized), `fn(...)` (arrow), or a bare
    /// `Ident` that happens not to be followed by `|`.
    ///
    /// Handles optional `where constraint(...) [, constraint(...)]` clauses
    /// on alias forms by consuming tokens until the next item boundary —
    /// refinement semantics are M2+ work.
    fn parse_type_rhs_after_eq(
        &mut self,
        name: String,
        type_params: Vec<String>,
        type_kw_span: SourceSpan,
    ) -> Result<SurfaceItem, Diagnostic> {
        if !self.rhs_is_sum() {
            let target = self.parse_type_expr()?;
            let mut end = target.span().byte_end;
            if matches!(self.peek().kind, TokenKind::KwWhere) {
                end = self.skip_where_clause()?;
            }
            return Ok(SurfaceItem::TypeAlias {
                name,
                type_params,
                target,
                span: SourceSpan::new(self.file, type_kw_span.byte_start, end),
            });
        }

        let variants = self.parse_sum_variants()?;
        let end = variants
            .last()
            .map(|v| v.span.byte_end)
            .unwrap_or(type_kw_span.byte_end);
        Ok(SurfaceItem::TypeSum {
            name,
            type_params,
            variants,
            span: SourceSpan::new(self.file, type_kw_span.byte_start, end),
        })
    }

    /// Consume a `where constraint1(args), constraint2(args)` clause
    /// and return the final byte offset. The clause ends at the next
    /// top-level item keyword (`let`/`fn`/`type`/`data`/`module`/
    /// `import`) or EOF. Refinement predicates land in M2+; at M1(2.6)
    /// we drop them after consuming their tokens.
    fn skip_where_clause(&mut self) -> Result<u32, Diagnostic> {
        let where_kw = self.expect_kind(TokenKind::KwWhere)?;
        let mut end = where_kw.span.byte_end;
        let mut depth: i32 = 0;
        while !self.at_eof() {
            match &self.peek().kind {
                TokenKind::LParen | TokenKind::LBrace | TokenKind::LBracket => {
                    depth += 1;
                }
                TokenKind::RParen | TokenKind::RBrace | TokenKind::RBracket => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
                TokenKind::KwLet
                | TokenKind::KwFn
                | TokenKind::KwType
                | TokenKind::KwData
                | TokenKind::KwModule
                | TokenKind::KwImport
                    if depth == 0 =>
                {
                    break;
                }
                _ => {}
            }
            end = self.peek().span.byte_end;
            self.bump();
        }
        Ok(end)
    }

    /// Lookahead: after `=`, is the RHS a sum (contains `|` at top level before
    /// the next item boundary)? Tracks paren/brace depth so a `|` inside a
    /// payload list doesn't confuse the scan.
    fn rhs_is_sum(&self) -> bool {
        let mut i = self.pos;
        let mut depth: i32 = 0;
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::LParen | TokenKind::LBrace | TokenKind::LBracket => {
                    depth += 1;
                }
                TokenKind::RParen | TokenKind::RBrace | TokenKind::RBracket => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                }
                TokenKind::Pipe if depth == 0 => return true,
                TokenKind::KwLet
                | TokenKind::KwFn
                | TokenKind::KwType
                | TokenKind::KwData
                | TokenKind::KwModule
                | TokenKind::KwImport
                | TokenKind::KwWhere
                | TokenKind::Eof
                    if depth == 0 =>
                {
                    return false;
                }
                _ => {}
            }
            i += 1;
        }
        false
    }

    fn parse_sum_variants(&mut self) -> Result<Vec<SurfaceVariant>, Diagnostic> {
        let mut variants = vec![self.parse_variant()?];
        while matches!(self.peek().kind, TokenKind::Pipe) {
            self.bump();
            variants.push(self.parse_variant()?);
        }
        Ok(variants)
    }

    fn parse_variant(&mut self) -> Result<SurfaceVariant, Diagnostic> {
        let name_token = self.bump().clone();
        let name = match &name_token.kind {
            TokenKind::Ident(n) => n.clone(),
            other => {
                return Err(Diagnostic::ParseError {
                    message: format!("expected variant name, got {other:?}"),
                    span: name_token.span.clone(),
                });
            }
        };
        match &self.peek().kind {
            TokenKind::LParen => {
                self.bump();
                let payload = self.parse_type_expr_list_until(TokenKind::RParen)?;
                let close = self.expect_kind(TokenKind::RParen)?;
                Ok(SurfaceVariant {
                    name,
                    payload: VariantPayload::Positional(payload),
                    span: SourceSpan::new(
                        self.file,
                        name_token.span.byte_start,
                        close.span.byte_end,
                    ),
                })
            }
            TokenKind::LBrace => {
                self.bump();
                let fields = self.parse_record_fields()?;
                let close = self.expect_kind(TokenKind::RBrace)?;
                Ok(SurfaceVariant {
                    name,
                    payload: VariantPayload::Record(fields),
                    span: SourceSpan::new(
                        self.file,
                        name_token.span.byte_start,
                        close.span.byte_end,
                    ),
                })
            }
            _ => Ok(SurfaceVariant {
                name,
                payload: VariantPayload::Unit,
                span: name_token.span,
            }),
        }
    }

    fn parse_params(&mut self) -> Result<Vec<SurfaceParam>, Diagnostic> {
        let mut params = Vec::new();
        if matches!(self.peek().kind, TokenKind::RParen) {
            return Ok(params);
        }
        loop {
            let name = self.parse_ident()?;
            self.expect_kind(TokenKind::Colon)?;
            let ty = self.parse_type_expr()?;
            params.push(SurfaceParam { name, ty });
            if matches!(self.peek().kind, TokenKind::Comma) {
                self.bump();
                continue;
            }
            break;
        }
        Ok(params)
    }

    fn parse_type_expr(&mut self) -> Result<SurfaceType, Diagnostic> {
        let mut ty = self.parse_atom_type()?;
        while matches!(self.peek().kind, TokenKind::Question) {
            let q = self.bump().clone();
            let start = ty.span().byte_start;
            ty = SurfaceType::Optional {
                inner: Box::new(ty),
                span: SourceSpan::new(self.file, start, q.span.byte_end),
            };
        }
        Ok(ty)
    }

    fn parse_atom_type(&mut self) -> Result<SurfaceType, Diagnostic> {
        if matches!(self.peek().kind, TokenKind::KwFn) {
            let fn_tok = self.bump().clone();
            self.expect_kind(TokenKind::LParen)?;
            let inputs = self.parse_type_expr_list_until(TokenKind::RParen)?;
            self.expect_kind(TokenKind::RParen)?;
            self.expect_kind(TokenKind::Arrow)?;
            let output = self.parse_type_expr()?;
            let end = output.span().byte_end;
            return Ok(SurfaceType::Arrow {
                inputs,
                output: Box::new(output),
                span: SourceSpan::new(self.file, fn_tok.span.byte_start, end),
            });
        }

        let token = self.bump().clone();
        let name = match token.kind {
            TokenKind::Ident(n) => n,
            other => {
                return Err(Diagnostic::ParseError {
                    message: format!("expected type name, got {other:?}"),
                    span: token.span,
                });
            }
        };

        if matches!(self.peek().kind, TokenKind::Lt) && self.looks_like_type_args() {
            self.bump();
            let args = self.parse_type_expr_list_until(TokenKind::Gt)?;
            let close = self.expect_kind(TokenKind::Gt)?;
            Ok(SurfaceType::Parameterized {
                name,
                args,
                span: SourceSpan::new(
                    self.file,
                    token.span.byte_start,
                    close.span.byte_end,
                ),
            })
        } else {
            Ok(SurfaceType::Named {
                name,
                span: token.span,
            })
        }
    }

    /// `<` is ambiguous: type-parameter delimiter vs. less-than operator. In
    /// type position (parse_atom_type), we only see `<` after a bare Ident, so
    /// it's always type args. In expression position (parse_comparison),
    /// parse_atom_type is not called. This helper exists for defensive future
    /// callers and currently always returns true.
    fn looks_like_type_args(&self) -> bool {
        true
    }

    fn parse_type_expr_list_until(
        &mut self,
        end: TokenKind,
    ) -> Result<Vec<SurfaceType>, Diagnostic> {
        let mut types = Vec::new();
        if self.peek().kind == end {
            return Ok(types);
        }
        loop {
            types.push(self.parse_type_expr()?);
            if matches!(self.peek().kind, TokenKind::Comma) {
                self.bump();
                continue;
            }
            break;
        }
        Ok(types)
    }

    fn parse_ident(&mut self) -> Result<String, Diagnostic> {
        let token = self.bump().clone();
        match token.kind {
            TokenKind::Ident(name) => Ok(name),
            other => Err(Diagnostic::ParseError {
                message: format!("expected identifier, got {other:?}"),
                span: token.span,
            }),
        }
    }

    fn parse_expr(&mut self) -> Result<SurfaceExpr, Diagnostic> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<SurfaceExpr, Diagnostic> {
        let lhs = self.parse_additive()?;
        let target = match &self.peek().kind {
            TokenKind::EqEq => Some("=="),
            TokenKind::NotEq => Some("!="),
            TokenKind::Lt => Some("<"),
            TokenKind::Le => Some("<="),
            TokenKind::Gt => Some(">"),
            TokenKind::Ge => Some(">="),
            _ => None,
        };
        if let Some(target) = target {
            self.bump();
            let rhs = self.parse_additive()?;
            let start = expr_span(&lhs).byte_start;
            let end = expr_span(&rhs).byte_end;
            Ok(SurfaceExpr::Call {
                target: target.to_string(),
                args: vec![lhs, rhs],
                span: SourceSpan::new(self.file, start, end),
            })
        } else {
            Ok(lhs)
        }
    }

    fn parse_additive(&mut self) -> Result<SurfaceExpr, Diagnostic> {
        let mut lhs = self.parse_term()?;
        loop {
            let target = match &self.peek().kind {
                TokenKind::Plus => "+",
                TokenKind::Minus => "-",
                _ => break,
            };
            self.bump();
            let rhs = self.parse_term()?;
            let start = expr_span(&lhs).byte_start;
            let end = expr_span(&rhs).byte_end;
            lhs = SurfaceExpr::Call {
                target: target.to_string(),
                args: vec![lhs, rhs],
                span: SourceSpan::new(self.file, start, end),
            };
        }
        Ok(lhs)
    }

    fn parse_term(&mut self) -> Result<SurfaceExpr, Diagnostic> {
        let mut lhs = self.parse_primary()?;
        loop {
            let target = match &self.peek().kind {
                TokenKind::Star => "*",
                TokenKind::Slash => "/",
                _ => break,
            };
            self.bump();
            let rhs = self.parse_primary()?;
            let start = expr_span(&lhs).byte_start;
            let end = expr_span(&rhs).byte_end;
            lhs = SurfaceExpr::Call {
                target: target.to_string(),
                args: vec![lhs, rhs],
                span: SourceSpan::new(self.file, start, end),
            };
        }
        Ok(lhs)
    }

    fn parse_primary(&mut self) -> Result<SurfaceExpr, Diagnostic> {
        if matches!(self.peek().kind, TokenKind::KwIf) {
            return self.parse_if();
        }
        let token = self.bump().clone();
        match token.kind {
            TokenKind::IntLit(value) => Ok(SurfaceExpr::IntLit {
                value,
                span: token.span,
            }),
            TokenKind::KwTrue => Ok(SurfaceExpr::BoolLit {
                value: true,
                span: token.span,
            }),
            TokenKind::KwFalse => Ok(SurfaceExpr::BoolLit {
                value: false,
                span: token.span,
            }),
            TokenKind::StringLit(value) => Ok(SurfaceExpr::StringLit {
                value,
                span: token.span,
            }),
            TokenKind::Ident(name) => {
                if matches!(self.peek().kind, TokenKind::LParen) {
                    self.bump();
                    let args = self.parse_call_args()?;
                    let close = self.expect_kind(TokenKind::RParen)?;
                    let start = token.span.byte_start;
                    let end = close.span.byte_end;
                    Ok(SurfaceExpr::Call {
                        target: name,
                        args,
                        span: SourceSpan::new(self.file, start, end),
                    })
                } else {
                    Ok(SurfaceExpr::Var {
                        name,
                        span: token.span,
                    })
                }
            }
            other => Err(Diagnostic::ParseError {
                message: format!("expected primary expression, got {other:?}"),
                span: token.span,
            }),
        }
    }

    fn parse_call_args(&mut self) -> Result<Vec<SurfaceExpr>, Diagnostic> {
        let mut args = Vec::new();
        if matches!(self.peek().kind, TokenKind::RParen) {
            return Ok(args);
        }
        loop {
            args.push(self.parse_expr()?);
            if matches!(self.peek().kind, TokenKind::Comma) {
                self.bump();
                continue;
            }
            break;
        }
        Ok(args)
    }

    fn parse_if(&mut self) -> Result<SurfaceExpr, Diagnostic> {
        let if_token = self.bump().clone();
        debug_assert!(matches!(if_token.kind, TokenKind::KwIf));
        let cond = self.parse_expr()?;
        self.expect_kind(TokenKind::KwThen)?;
        let then_branch = self.parse_expr()?;
        self.expect_kind(TokenKind::KwElse)?;
        let else_branch = self.parse_expr()?;
        let start = if_token.span.byte_start;
        let end = expr_span(&else_branch).byte_end;
        Ok(SurfaceExpr::If {
            cond: Box::new(cond),
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
            span: SourceSpan::new(self.file, start, end),
        })
    }
}

fn expr_span(expr: &SurfaceExpr) -> &SourceSpan {
    match expr {
        SurfaceExpr::IntLit { span, .. }
        | SurfaceExpr::BoolLit { span, .. }
        | SurfaceExpr::StringLit { span, .. }
        | SurfaceExpr::Var { span, .. }
        | SurfaceExpr::Call { span, .. }
        | SurfaceExpr::If { span, .. } => span,
    }
}
