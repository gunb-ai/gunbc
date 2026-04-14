// Surface AST + minimal hand-recursive parser.
//
// G3 guardrail: parse.rs exports SurfaceAst / SurfaceExpr; it does
// NOT mention Dag or any L1 behavior type. Lowering from surface to
// DAG happens in lower.rs — the two stages are structurally separated
// so an alternative frontend can plug in at the SurfaceAst layer
// without contaminating the DAG.
//
// Operators and function calls share a single SurfaceExpr::Call
// variant. `1 + 2` produces Call { target: "std::int::add", args:
// [1, 2] }. `f(x, y)` produces Call { target: "f", args: [x, y] }.
// The substrate sees no operator-vs-call distinction, and parse.rs
// doesn't either once the initial dispatch is done.
//
// M0.3 grammar:
//   module     := item*
//   item       := `fn` ident `(` params `)` `->` type_ann `=` expr
//              | `let` ident `=` expr
//   params     := ( ident `:` type_ann ( `,` ident `:` type_ann )* )?
//   type_ann   := ident                          (Int | Bool | String)
//   expr       := comparison
//   comparison := additive ( cmp_op additive )?
//   additive   := term ( (`+` | `-`) term )*
//   term       := primary ( (`*` | `/`) primary )*
//   primary    := int_lit
//              | ident ( `(` args `)` )?
//              | `if` expr `then` expr `else` expr
//   args       := ( expr ( `,` expr )* )?

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
        expr: SurfaceExpr,
    },
    Fn {
        name: String,
        params: Vec<SurfaceParam>,
        return_type: SurfaceType,
        body: SurfaceExpr,
    },
}

#[derive(Debug, Clone)]
pub struct SurfaceParam {
    pub name: String,
    pub ty: SurfaceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceType {
    Named(String),
}

#[derive(Debug, Clone)]
pub enum SurfaceExpr {
    IntLit {
        value: i64,
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
            other => Err(Diagnostic::ParseError {
                message: format!("expected `let` or `fn`, got {other:?}"),
                span: self.peek().span.clone(),
            }),
        }
    }

    fn parse_let_item(&mut self) -> Result<SurfaceItem, Diagnostic> {
        self.expect_kind(TokenKind::KwLet)?;
        let name = self.parse_ident()?;
        self.expect_kind(TokenKind::Eq)?;
        let expr = self.parse_expr()?;
        Ok(SurfaceItem::Let { name, expr })
    }

    fn parse_fn_item(&mut self) -> Result<SurfaceItem, Diagnostic> {
        self.expect_kind(TokenKind::KwFn)?;
        let name = self.parse_ident()?;
        self.expect_kind(TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect_kind(TokenKind::RParen)?;
        self.expect_kind(TokenKind::Arrow)?;
        let return_type = self.parse_type_ann()?;
        self.expect_kind(TokenKind::Eq)?;
        let body = self.parse_expr()?;
        Ok(SurfaceItem::Fn {
            name,
            params,
            return_type,
            body,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<SurfaceParam>, Diagnostic> {
        let mut params = Vec::new();
        if matches!(self.peek().kind, TokenKind::RParen) {
            return Ok(params);
        }
        loop {
            let name = self.parse_ident()?;
            self.expect_kind(TokenKind::Colon)?;
            let ty = self.parse_type_ann()?;
            params.push(SurfaceParam { name, ty });
            if matches!(self.peek().kind, TokenKind::Comma) {
                self.bump();
                continue;
            }
            break;
        }
        Ok(params)
    }

    fn parse_type_ann(&mut self) -> Result<SurfaceType, Diagnostic> {
        let name = self.parse_ident()?;
        Ok(SurfaceType::Named(name))
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
            TokenKind::EqEq => Some("std::int::eq"),
            TokenKind::NotEq => Some("std::int::ne"),
            TokenKind::Lt => Some("std::int::lt"),
            TokenKind::Le => Some("std::int::le"),
            TokenKind::Gt => Some("std::int::gt"),
            TokenKind::Ge => Some("std::int::ge"),
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
                TokenKind::Plus => "std::int::add",
                TokenKind::Minus => "std::int::sub",
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
                TokenKind::Star => "std::int::mul",
                TokenKind::Slash => "std::int::div",
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
        | SurfaceExpr::Var { span, .. }
        | SurfaceExpr::Call { span, .. }
        | SurfaceExpr::If { span, .. } => span,
    }
}
