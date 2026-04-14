// Surface AST + minimal hand-recursive parser.
//
// G3 guardrail: parse.rs exports SurfaceAst / SurfaceExpr; it does NOT
// mention Dag or any L1 behavior type. Lowering from surface to DAG
// happens in lower.rs — the two stages are structurally separated so
// an alternative frontend can plug in at the SurfaceAst layer without
// contaminating the DAG.
//
// M0.2 grammar:
//   module     := stmt*
//   stmt       := `let` ident `=` expr
//   expr       := comparison
//   comparison := additive ( cmp_op additive )?
//   additive   := primary ( `+` primary )*
//   primary    := int_lit
//              | ident
//              | `if` expr `then` expr `else` expr
//
// Precedence layering over Pratt because M0 scope is small.

use crate::diagnostics::{Diagnostic, SourceSpan};
use crate::tokenize::{Token, TokenKind};

#[derive(Debug, Clone)]
pub struct SurfaceModule {
    pub statements: Vec<SurfaceStmt>,
}

#[derive(Debug, Clone)]
pub enum SurfaceStmt {
    Let { name: String, expr: SurfaceExpr },
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
    BinOp {
        op: SurfaceBinOp,
        lhs: Box<SurfaceExpr>,
        rhs: Box<SurfaceExpr>,
        span: SourceSpan,
    },
    If {
        cond: Box<SurfaceExpr>,
        then_branch: Box<SurfaceExpr>,
        else_branch: Box<SurfaceExpr>,
        span: SourceSpan,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceBinOp {
    Add,
    Eq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
}

pub fn parse(tokens: &[Token], file: &str) -> Result<SurfaceModule, Diagnostic> {
    let mut parser = Parser {
        tokens,
        pos: 0,
        file,
    };
    let mut statements = Vec::new();
    while !parser.at_eof() {
        statements.push(parser.parse_stmt()?);
    }
    Ok(SurfaceModule { statements })
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

    fn parse_stmt(&mut self) -> Result<SurfaceStmt, Diagnostic> {
        match &self.peek().kind {
            TokenKind::KwLet => {
                self.bump();
                let name_token = self.bump().clone();
                let name = match &name_token.kind {
                    TokenKind::Ident(n) => n.clone(),
                    other => {
                        return Err(Diagnostic::ParseError {
                            message: format!("expected identifier after `let`, got {other:?}"),
                            span: name_token.span.clone(),
                        });
                    }
                };
                let eq_token = self.bump().clone();
                if eq_token.kind != TokenKind::Eq {
                    return Err(Diagnostic::ParseError {
                        message: format!("expected `=`, got {:?}", eq_token.kind),
                        span: eq_token.span,
                    });
                }
                let expr = self.parse_expr()?;
                Ok(SurfaceStmt::Let { name, expr })
            }
            other => Err(Diagnostic::ParseError {
                message: format!("expected statement, got {other:?}"),
                span: self.peek().span.clone(),
            }),
        }
    }

    fn parse_expr(&mut self) -> Result<SurfaceExpr, Diagnostic> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<SurfaceExpr, Diagnostic> {
        let lhs = self.parse_additive()?;
        let op = match &self.peek().kind {
            TokenKind::EqEq => Some(SurfaceBinOp::Eq),
            TokenKind::NotEq => Some(SurfaceBinOp::NotEq),
            TokenKind::Lt => Some(SurfaceBinOp::Lt),
            TokenKind::Le => Some(SurfaceBinOp::Le),
            TokenKind::Gt => Some(SurfaceBinOp::Gt),
            TokenKind::Ge => Some(SurfaceBinOp::Ge),
            _ => None,
        };
        if let Some(op) = op {
            self.bump();
            let rhs = self.parse_additive()?;
            let start = expr_span(&lhs).byte_start;
            let end = expr_span(&rhs).byte_end;
            Ok(SurfaceExpr::BinOp {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span: SourceSpan::new(self.file, start, end),
            })
        } else {
            Ok(lhs)
        }
    }

    fn parse_additive(&mut self) -> Result<SurfaceExpr, Diagnostic> {
        let mut lhs = self.parse_primary()?;
        while matches!(self.peek().kind, TokenKind::Plus) {
            self.bump();
            let rhs = self.parse_primary()?;
            let start = expr_span(&lhs).byte_start;
            let end = expr_span(&rhs).byte_end;
            lhs = SurfaceExpr::BinOp {
                op: SurfaceBinOp::Add,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
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
            TokenKind::Ident(name) => Ok(SurfaceExpr::Var {
                name,
                span: token.span,
            }),
            other => Err(Diagnostic::ParseError {
                message: format!("expected primary expression, got {other:?}"),
                span: token.span,
            }),
        }
    }

    fn parse_if(&mut self) -> Result<SurfaceExpr, Diagnostic> {
        let if_token = self.bump().clone();
        debug_assert!(matches!(if_token.kind, TokenKind::KwIf));
        let cond = self.parse_expr()?;
        let then_token = self.bump().clone();
        if then_token.kind != TokenKind::KwThen {
            return Err(Diagnostic::ParseError {
                message: format!("expected `then`, got {:?}", then_token.kind),
                span: then_token.span,
            });
        }
        let then_branch = self.parse_expr()?;
        let else_token = self.bump().clone();
        if else_token.kind != TokenKind::KwElse {
            return Err(Diagnostic::ParseError {
                message: format!("expected `else`, got {:?}", else_token.kind),
                span: else_token.span,
            });
        }
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
        | SurfaceExpr::BinOp { span, .. }
        | SurfaceExpr::If { span, .. } => span,
    }
}
