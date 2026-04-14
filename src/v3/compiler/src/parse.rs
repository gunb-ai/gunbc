// Surface AST + minimal hand-recursive parser.
//
// G3 guardrail: parse.rs exports SurfaceAst / SurfaceExpr; it does NOT
// mention Dag or any L1 behavior type. Lowering from surface to DAG
// happens in lower.rs — the two stages are structurally separated so
// an alternative frontend can plug in at the SurfaceAst layer without
// contaminating the DAG.
//
// M0.1 grammar (Test 1):
//   module  := stmt*
//   stmt    := `let` ident `=` expr
//   expr    := int_lit ( `+` int_lit )*
//            | ident   ( `+` ident   )*
// Pratt parsing and real operator precedence arrive when Test 2
// needs them (`<`, `>` for the `if` condition).

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceBinOp {
    Add,
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
                message: format!("expected int literal or identifier, got {other:?}"),
                span: token.span,
            }),
        }
    }
}

fn expr_span(expr: &SurfaceExpr) -> &SourceSpan {
    match expr {
        SurfaceExpr::IntLit { span, .. }
        | SurfaceExpr::Var { span, .. }
        | SurfaceExpr::BinOp { span, .. } => span,
    }
}
