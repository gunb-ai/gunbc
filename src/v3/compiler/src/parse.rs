// Surface AST + hand-recursive parser for the v3 surface grammar.
//
// G3 guardrail: parse.rs exports SurfaceModule / SurfaceItem / SurfaceExpr /
// SurfaceType; it does NOT mention Dag or any L1 behavior type. Lowering from
// surface to DAG happens in lower.rs.
//
// Operators compile to a structural `SurfaceExpr::Operator` variant.
// `1 + 2` → `Operator { op: OperatorKind::Arithmetic(ArithmeticOp::Add),
// args: [1, 2] }`. The parser commits to the operator's enum variant at
// parse time (it already knows, because operator symbols come from
// different grammar productions than identifiers); `lower.rs` emits a
// `TransformNode { target: TransformTarget::Operator(OperatorKind) }`;
// `infer::resolve_operator_arrow` walks the LHS type's algebra chain in
// `std/algebra.dag` to read the concrete Arrow signature.
//
// This replaces the M1(2.5)-era design in which operators compiled to
// identifier-shaped Calls (`Call { target: "+" }`) that were resolved
// through an `OPERATOR_FIELD_MAP` bridge. See
// `DOWNSTREAM_REQUIREMENTS.md` M1(2.7) Class 2 for the dissolution.
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

/// Dissolution ledger — **SurfaceItem**:
///
/// 🟢 **Terminal at M1(2.7).** Ten variants. The earlier six-variant
/// shape from M1(2.6) grew four variants at M1(2.7) to resolve the
/// scaffold-honesty gaps (QW1–QW3):
///
/// - **`Fn`** now always carries a body `SurfaceExpr`. Block-bodied
///   fn declarations in std/ files become `FnExternalBody` —
///   structurally separate variants, not an `Option<SurfaceExpr>`
///   with the discriminator in the Option.
/// - **`FnExternalBody`** records `name + params + return_type +
///   body_span`. Lowers to a declaration whose connective is an
///   `Arrow` with `ArrowBody::Unparsed(body_span)`. The signature
///   flows forward; the body is preserved by span.
/// - **`Data`**, **`Module`**, **`Import`** replace the three former
///   parser-absorbed items. `Data` lowers to a declaration whose
///   connective is the resolved type; `Module` and `Import` lower
///   to no-ops at M1(2.7) but preserve the parsed facts for M2
///   module scoping.
///
/// 4-pattern check:
/// - Pattern 1 (fact placement): fails. Each variant has a distinct
///   lowering path.
/// - Pattern 2 (variant-is-data): fails. Different payload types.
/// - Pattern 3 (algebraic form): partial. The four `Type*` variants
///   still could collapse into `Type { name, type_params, shape }`
///   in M2; `Fn` + `FnExternalBody` could collapse into
///   `Fn { body: FnBody }` where `FnBody` is a coproduct. Both
///   restructures are tracked as M2 work.
/// - Pattern 4 (dimensional): fails.
///
/// Verdict: terminal at M1(2.7) modulo the two M2 collapses noted
/// above. `FnExternalBody` has its own dissolution trigger (when
/// match/pipe/lambda land in the parser, block bodies become real
/// `Fn` items with full `SurfaceExpr` bodies).
#[derive(Debug, Clone)]
pub enum SurfaceItem {
    Let {
        name: String,
        type_ann: Option<SurfaceType>,
        expr: SurfaceExpr,
    },
    /// Expression-body function definition: `fn f(x) -> T = expr`.
    /// The body is always present and always a `SurfaceExpr`. Lowers
    /// to a declaration whose connective is an `Arrow` with
    /// `ArrowBody::UserDefined(NodeId)` pointing at the lowered
    /// sub-DAG.
    Fn {
        name: String,
        params: Vec<SurfaceParam>,
        return_type: SurfaceType,
        body: SurfaceExpr,
        span: SourceSpan,
    },
    /// Block-body scaffold: `fn f(x) -> T { body }` where the body is
    /// not expressible in the M1(2.7) surface grammar. The parser
    /// brace-skipped the body range and records its span. Lowers to
    /// a declaration whose connective is an `Arrow` with
    /// `ArrowBody::Unparsed(body_span)`. The signature flows forward
    /// so callers can type-check against it; the body stays
    /// scaffolded until the parser adopts match/pipe/lambda.
    FnExternalBody {
        name: String,
        params: Vec<SurfaceParam>,
        return_type: SurfaceType,
        body_span: SourceSpan,
        span: SourceSpan,
    },
    /// `data name: Type = { body }` — a typed constant. At M1(3)
    /// PR-B the parser attempts to lower the body as a record
    /// literal (via a 3-token lookahead on `{`, `Ident`, `:`); if
    /// successful, `body` is `Some(SurfaceExpr::Record { .. })` and
    /// lowering validates the record against the type annotation
    /// via inhabitance checking. If the body doesn't match the
    /// record-literal shape, `body` is `None` and the body is
    /// brace-skipped, preserving only the span (M1(2.7) R14's
    /// ValueBody::Unparsed path).
    ///
    /// `body_span` always reflects the full range of the brace
    /// group so downstream span-anchored diagnostics and future
    /// parser extensions have a stable anchor.
    Data {
        name: String,
        ty: SurfaceType,
        body: Option<SurfaceExpr>,
        body_span: SourceSpan,
        span: SourceSpan,
    },
    /// `module foo.bar.baz` — parsed into a dotted path. At M1(2.7)
    /// lowering is a no-op; M2+ consumes it as a scope boundary.
    Module {
        path: Vec<String>,
        span: SourceSpan,
    },
    /// `import foo.bar { Name1, Name2 }` — parsed into a dotted path
    /// plus an optional name list. At M1(2.7) lowering is a no-op
    /// (the declaration table is flat); M2+ consumes it as a
    /// scoped symbol-table seed.
    Import {
        path: Vec<String>,
        names: Vec<String>,
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

/// Dissolution ledger — **VariantPayload**:
///
/// 🟢 **Terminal at M1(2.6).** Two variants: `Positional` and
/// `Record`. Unit variants like `True | False` are represented as
/// `Positional(vec![])` — there is no separate `Unit` variant
/// because it was structurally equivalent to an empty positional.
///
/// 4-pattern check:
/// - Pattern 1 (fact placement): fails. Record vs positional
///   have different downstream field shapes (labeled vs
///   anonymous).
/// - Pattern 2 (variant-is-data): fails. Record carries
///   `Vec<SurfaceField>`, positional carries `Vec<SurfaceType>`.
/// - Pattern 3 (algebraic form): fails.
/// - Pattern 4 (dimensional): fails.
///
/// Verdict: terminal. A future `Tuple` variant or named-indexed
/// payload would extend through §8.10's substrate-extension
/// audit, not replace this enum.
#[derive(Debug, Clone)]
pub enum VariantPayload {
    /// Positional payload — e.g. `Ok(T)` in `type Result<T, E> = Ok(T) | Err(E)`,
    /// or an empty `vec![]` for unit variants like `True` / `False`.
    Positional(Vec<SurfaceType>),
    /// Record-style payload — e.g. `WorkloadIdentity { audience: NonEmptyStr, ... }`.
    Record(Vec<SurfaceField>),
}

/// Dissolution ledger — **SurfaceType**:
///
/// 🟢 **Terminal at M1(2.6).** Four variants cover the four surface
/// type-expression shapes: bare name, parameterized, optional,
/// function type. Each maps onto a distinct `TypeConnective` variant
/// during lowering (Atom(Identifier), Instantiation, Cardinality,
/// Arrow) so the coproduct mirrors the substrate.
///
/// 4-pattern check:
/// - Pattern 1 (fact placement): fails. Each variant lowers via a
///   distinct `type_to_declaration_id` arm.
/// - Pattern 2 (variant-is-data): fails. Different payload types.
/// - Pattern 3 (algebraic form): the variants correspond 1:1 to
///   distinct `TypeConnective` variants — this is the surface
///   projection of the type substrate. Collapsing is a category
///   error.
/// - Pattern 4 (dimensional): fails.
///
/// STOP SIGNAL: adding a variant that doesn't correspond to a
/// `TypeConnective` shape (e.g., intersection types, type-level
/// literals) triggers a substrate question — is the surface
/// grammar running ahead of the substrate, or is the substrate
/// missing a sixth connective? Re-run the `TypeConnective`
/// dissolution audit in `dag.rs` before extending this enum.
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

/// Dissolution ledger — **SurfaceExpr**:
///
/// 🟡 **Scaffold at M1(3) PR-B.** Seven variants for the seven
/// currently supported expression forms: Literal, Var, Call,
/// Operator, If, Match, Record. Each has a distinct lowering
/// target:
///   Literal  → Value(LiteralBits::*)
///   Var      → scope lookup (no new node)
///   Call     → Transform with TransformTarget::Callable
///   Operator → Transform with TransformTarget::Operator
///   If       → Branch (lowered as match on Bool with two arms)
///   Match    → Branch with one Path per arm
///   Record   → **data-body only** at M1(3) PR-B; emits a
///              ValueBody::Structural on the enclosing
///              Declaration via `lower_data_item`. In user-code
///              expression position `lower_expr::Record` emits
///              a fail-closed diagnostic pointing at class-5
///              gap #3 (user-code record literals land when
///              list/map/map-body parsing follows in M2+).
///
/// **M1(3) PR-B change:** `SurfaceExpr::Record { fields, span }`
/// joined the enum to let `src/v3/spec/rust.dag` exist
/// as a structurally-grounded language spec file that the Rust
/// emitter reads at compile time. The parser accepts record
/// literals only in `data foo: T = { ... }` body position via
/// a 3-token lookahead (`{`, `Ident`, `:`) in `parse_data_item`;
/// `parse_primary` does NOT dispatch to record literals, so
/// user-code expressions like `let x = { a: 1 }` still fail at
/// parse time (with a better follow-up diagnostic forthcoming).
///
/// **M1(2.8) change:** `SurfaceExpr::Match { scrutinee, arms }`
/// joined the enum as part of the parser catch-up to v2's
/// grammar (see `DOWNSTREAM_REQUIREMENTS.md` M1(2.8) section).
/// It lowers to a `Branch` — no new L1 behavior variant — and
/// its arms carry `BranchPattern` discriminators on the emitted
/// `Path`s, so `if`/`else` and `match` share the same dispatch
/// substrate.
///
/// **M1(2.7) change:** the former `Call { target: String, .. }` was
/// doing double duty — representing both user function calls AND
/// primitive operator applications (with the target string acting
/// as discriminator: `"+"` vs `"foo"`). That's the same shape the
/// enumeration pass flagged as Q3: one field, two jobs, string as
/// discriminator. The new `Operator { op: OperatorKind, .. }`
/// variant puts the distinction on the type. Parser commits to
/// which variant to emit at parse time — it already knows, because
/// operator symbols come from different grammar productions than
/// identifiers.
///
/// The former IntLit/BoolLit/StringLit trio is still collapsed
/// into a single `Literal(SurfaceLiteral)` variant.
///
/// 4-pattern check:
/// - Pattern 1 (fact placement): fails. Each variant has a distinct
///   downstream lowering path.
/// - Pattern 2 (variant-is-data): fails. Different payload types
///   per variant.
/// - Pattern 3 (algebraic form): these are the six expression
///   kinds that M1(2.8) supports — collapsing would erase
///   structure, not dissolve it. The enum will grow as the
///   parser catches up to v2's grammar (pipe `|>`, lambda
///   `=> expr`, record/map/list literals, field access).
/// - Pattern 4 (dimensional): fails.
///
/// Verdict: 🟡 scaffold, not terminal. The enum is in flight —
/// the M1(2.8) addition was itself a scaffold-honesty fix for
/// `FnExternalBody` / `ArrowBody::Unparsed` bodies that the
/// parser couldn't previously consume. Further additions go
/// through §8.10's substrate-extension audit as each grammar
/// surface lands. Dissolution trigger: the enum reaches terminal
/// shape only when the surface grammar subsumes the set of
/// forms found in the fully-loaded v3 std/ set, with no
/// remaining `Unparsed` scaffolds in the bootstrap. When the
/// M2+ parser covers record/map/list literals and the data
/// body gap (class-5 gap #3) closes, this ledger is re-run.
/// The `Operator` variant also dissolves at M2+ into
/// `Call` once explicit algebra-field access syntax lands.
#[derive(Debug, Clone)]
pub enum SurfaceExpr {
    Literal {
        value: SurfaceLiteral,
        span: SourceSpan,
    },
    Var {
        name: String,
        span: SourceSpan,
    },
    /// Dotted-path identifier — `OrderedRing.add`,
    /// `dsl.std.v3_l1.Bind`, etc. Produced by `parse_primary` when
    /// it sees `Ident . Ident (. Ident)*`. Distinct from `Var` so
    /// downstream lowering can tell whether it was looking at a
    /// bare name (top-level scope lookup) or a dotted reference
    /// (top-level lookup followed by Conj-child walk by label).
    /// At M1(3) PR-B-unwind only `lower_record_to_structural`
    /// resolves Path values — it accepts them as field values when
    /// the declared field type is the `DeclarationRef` sentinel from
    /// `src/v3/spec/v3_l1.dag`. User-code expression position lowers
    /// Path through `lower_expr` with a fail-closed diagnostic
    /// (no semantics yet beyond the realization spec use case).
    Path {
        segments: Vec<String>,
        span: SourceSpan,
    },
    Call {
        target: String,
        args: Vec<SurfaceExpr>,
        span: SourceSpan,
    },
    /// Primitive binary operator application. Distinct from `Call`
    /// because the dispatch is via algebra inhabitance, not a
    /// declaration lookup. At parse time the operator is committed
    /// to a structural `OperatorKind` variant; downstream code
    /// never re-parses the operator symbol.
    Operator {
        op: crate::operators::OperatorKind,
        args: Vec<SurfaceExpr>,
        span: SourceSpan,
    },
    If {
        cond: Box<SurfaceExpr>,
        then_branch: Box<SurfaceExpr>,
        else_branch: Box<SurfaceExpr>,
        span: SourceSpan,
    },
    /// `match <scrutinee> { <pattern> => <expr> <pattern> => <expr> ... }`
    /// Lowers to a `Branch` behavior with the scrutinee as input
    /// and one `Path` per arm carrying the arm's pattern.
    Match {
        scrutinee: Box<SurfaceExpr>,
        arms: Vec<SurfaceMatchArm>,
        span: SourceSpan,
    },
    /// `{ field: expr, field: expr, ... }` — record literal. At
    /// M1(3) PR-B this variant is produced ONLY by
    /// `parse_data_item` when the body of a `data foo: T = {...}`
    /// declaration matches the 3-token lookahead record-literal
    /// shape. `parse_primary` does not dispatch to record literal
    /// parsing, so user-code expressions containing `{ ... }` still
    /// fail at parse time. Lowering for user-code position
    /// (`lower_expr::Record`) emits a fail-closed diagnostic
    /// pointing at class-5 gap #3; lowering for data-body position
    /// (`lower_data_item`) walks the type annotation to a Conj and
    /// runs inhabitance checking, producing a
    /// `ValueBody::Structural { fields }` on the declaration.
    Record {
        fields: Vec<SurfaceRecordField>,
        span: SourceSpan,
    },
}

/// A single field in a record literal: a label and the expression
/// assigned to it. At M1(3) PR-B lowering requires field values to
/// be literal scalars (Int/Bool/String) — nested records,
/// references, and computed expressions are class-5 gap #3
/// follow-ups.
#[derive(Debug, Clone)]
pub struct SurfaceRecordField {
    pub name: String,
    pub value: SurfaceExpr,
    pub span: SourceSpan,
}

/// A single arm of a `match` expression. Pattern plus body.
#[derive(Debug, Clone)]
pub struct SurfaceMatchArm {
    pub pattern: SurfacePattern,
    pub body: SurfaceExpr,
    pub span: SourceSpan,
}

/// Dissolution ledger — **SurfacePattern**:
///
/// 🟡 **M1(2.8) scaffold.** Single variant today: bare variant
/// patterns like `True`, `None`, `Plus` — a parse-time name that
/// the inference pass resolves scoped against the scrutinee's
/// `Disj` connective. Future extensions (wildcard `_`, record
/// destructure `Some { value: x }`, nested patterns, literal
/// patterns) go through §8.10's substrate-extension audit.
///
/// 4-pattern check against the current single variant:
/// - Pattern 1 (fact placement): fails. The pattern is per-arm
///   data that `BranchPattern::UnresolvedVariant` consumes; no
///   alternative structural placement.
/// - Pattern 2 (variant-is-data): not applicable (single variant).
/// - Pattern 3 (algebraic form): fails.
/// - Pattern 4 (dimensional): fails.
///
/// Verdict: scaffold. The set of patterns grows as the parser
/// grammar catches up to v2.
#[derive(Debug, Clone)]
pub enum SurfacePattern {
    /// A bare constructor name, e.g. the `True` in
    /// `match a { True => x, False => y }`. Resolves at infer
    /// time against the scrutinee's Disj children by label.
    BareVariant { name: String, span: SourceSpan },
}

/// Parse-local literal value. Mirrors `dag::LiteralBits` but lives
/// in the parse layer so the G3 guardrail ("parse.rs does not
/// mention any Dag type") is preserved: an alternative frontend
/// that plugs in at the SurfaceAst boundary works against
/// `SurfaceLiteral`, not `LiteralBits`. The surface→substrate
/// translation in `lower.rs` is the bridge.
///
/// Dissolution ledger — 🟢 terminal. Each variant is a distinct
/// user-input boundary (integer literal, boolean literal, string
/// literal) with no shared structure. A future `Char` or `Float`
/// addition extends through §8.10.
#[derive(Debug, Clone)]
pub enum SurfaceLiteral {
    Int(i64),
    Bool(bool),
    String(String),
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

    /// Parse the next top-level item. Every surface form emits a real
    /// `SurfaceItem` — the earlier "parser-absorbed" path was deleted
    /// at M1(2.7) per the QW1/QW2/QW3 scaffold-honesty fix. Module,
    /// import, and data items lower to no-ops (or declaration-only
    /// scaffolds) at M1(2.7), but the parsed facts flow forward.
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

    /// Parse `module foo.bar.baz` into a `SurfaceItem::Module { path }`.
    /// At M1(2.7) lowering is a no-op, but the parsed path survives
    /// for M2 module scoping to consume.
    fn parse_module_item(&mut self) -> Result<SurfaceItem, Diagnostic> {
        let kw = self.expect_kind(TokenKind::KwModule)?;
        let path = self.parse_dotted_path()?;
        // The dotted path ends at the last parsed identifier; use its
        // span's end as the item's end. `parse_dotted_path` doesn't
        // return spans, so fall back to the current position's
        // previous token span. The simplest correct thing: use the
        // last token consumed by peek-1.
        let end = self.tokens[self.pos.saturating_sub(1)].span.byte_end;
        Ok(SurfaceItem::Module {
            path,
            span: SourceSpan::new(self.file, kw.span.byte_start, end),
        })
    }

    /// Parse `import foo.bar { Name1, Name2 }` into
    /// `SurfaceItem::Import { path, names }`. The names list is
    /// optional (bare `import foo.bar` imports everything).
    fn parse_import_item(&mut self) -> Result<SurfaceItem, Diagnostic> {
        let kw = self.expect_kind(TokenKind::KwImport)?;
        let path = self.parse_dotted_path()?;
        let mut names: Vec<String> = Vec::new();
        if matches!(self.peek().kind, TokenKind::LBrace) {
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
            self.expect_kind(TokenKind::RBrace)?;
        }
        let end = self.tokens[self.pos.saturating_sub(1)].span.byte_end;
        Ok(SurfaceItem::Import {
            path,
            names,
            span: SourceSpan::new(self.file, kw.span.byte_start, end),
        })
    }

    /// Parse `data name: Type = { body }` into `SurfaceItem::Data`.
    ///
    /// **M1(3) PR-B change:** the body is now attempted as a record
    /// literal first. A 3-token lookahead (`{`, `Ident`, `:`) decides
    /// between:
    ///
    /// - **Record literal**: the body is parsed as a
    ///   `SurfaceExpr::Record { fields, span }` and stored in
    ///   `SurfaceItem::Data.body = Some(...)`. Lowering runs
    ///   inhabitance checking against the type annotation (walks
    ///   the Conj children, matches labels, validates literal-only
    ///   field values) and produces a
    ///   `ValueBody::Structural { fields }` on the declaration.
    /// - **Unparseable body**: any body whose first three tokens
    ///   don't match `{ Ident :` falls back to the brace-skip path
    ///   (preserving the span) and `SurfaceItem::Data.body = None`.
    ///   Lowering produces a `ValueBody::Unparsed(span)` scaffold
    ///   which `reject_user_unparsed_scaffolds` rejects for user
    ///   code and tolerates for bootstrap fixtures (R14 behavior).
    ///
    /// The lookahead is unambiguous because record field syntax
    /// starts with `Ident :` and no other valid body shape does.
    fn parse_data_item(&mut self) -> Result<SurfaceItem, Diagnostic> {
        let kw = self.expect_kind(TokenKind::KwData)?;
        let name = self.parse_ident()?;
        self.expect_kind(TokenKind::Colon)?;
        let ty = self.parse_type_expr()?;
        self.expect_kind(TokenKind::Eq)?;
        // Peek three tokens for the record-literal lookahead.
        let open_span = self.peek().span.clone();
        let is_record_literal = self.looks_like_record_literal();
        if is_record_literal {
            let body_expr = self.parse_record_literal()?;
            let body_end = match &body_expr {
                SurfaceExpr::Record { span, .. } => span.byte_end,
                _ => unreachable!("parse_record_literal always returns Record"),
            };
            return Ok(SurfaceItem::Data {
                name,
                ty,
                body: Some(body_expr),
                body_span: SourceSpan::new(
                    self.file,
                    open_span.byte_start,
                    body_end,
                ),
                span: SourceSpan::new(
                    self.file,
                    kw.span.byte_start,
                    body_end,
                ),
            });
        }
        let end = self.skip_brace_balanced()?;
        let body_span = SourceSpan::new(self.file, open_span.byte_start, end);
        Ok(SurfaceItem::Data {
            name,
            ty,
            body: None,
            body_span,
            span: SourceSpan::new(self.file, kw.span.byte_start, end),
        })
    }

    /// 3-token lookahead for the record-literal disambiguation in
    /// `parse_data_item`. Returns `true` when the next three tokens
    /// are `{`, then `Ident`, then `:` — the unambiguous start of a
    /// record literal field. Any other shape (empty `{}`, `{` followed
    /// by a non-identifier, or `{ ident` without a colon) returns
    /// false and the caller falls back to the brace-skip path.
    fn looks_like_record_literal(&self) -> bool {
        let t0 = self.tokens.get(self.pos);
        let t1 = self.tokens.get(self.pos + 1);
        let t2 = self.tokens.get(self.pos + 2);
        match (t0, t1, t2) {
            (Some(a), Some(b), Some(c)) => {
                matches!(a.kind, TokenKind::LBrace)
                    && matches!(b.kind, TokenKind::Ident(_))
                    && matches!(c.kind, TokenKind::Colon)
            }
            _ => false,
        }
    }

    /// Parse a record literal starting at `{`. Called from
    /// `parse_data_item` after `looks_like_record_literal` confirms
    /// the lookahead. Reads `label: expr` pairs separated by `,` (or
    /// whitespace), terminated by `}`. Each field's value is any
    /// `SurfaceExpr` — lowering (not parsing) enforces the literal-
    /// only restriction.
    fn parse_record_literal(&mut self) -> Result<SurfaceExpr, Diagnostic> {
        let open = self.expect_kind(TokenKind::LBrace)?;
        let mut fields: Vec<SurfaceRecordField> = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RBrace) {
            let name_token = self.bump().clone();
            let field_name = match name_token.kind {
                TokenKind::Ident(n) => n,
                other => {
                    return Err(Diagnostic::ParseError {
                        message: format!(
                            "expected field name in record literal, got {other:?}"
                        ),
                        span: name_token.span,
                    });
                }
            };
            self.expect_kind(TokenKind::Colon)?;
            let value = self.parse_expr()?;
            let field_end = expr_span(&value).byte_end;
            fields.push(SurfaceRecordField {
                name: field_name,
                value,
                span: SourceSpan::new(
                    self.file,
                    name_token.span.byte_start,
                    field_end,
                ),
            });
            // Accept an optional comma between fields (whitespace
            // alone is also permitted).
            if matches!(self.peek().kind, TokenKind::Comma) {
                self.bump();
            }
        }
        let close = self.expect_kind(TokenKind::RBrace)?;
        Ok(SurfaceExpr::Record {
            fields,
            span: SourceSpan::new(self.file, open.span.byte_start, close.span.byte_end),
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
                    body: body_expr,
                    span: SourceSpan::new(self.file, fn_kw.span.byte_start, end),
                })
            }
            TokenKind::LBrace => {
                // Block-body scaffold form: `fn f(x) -> T { body }`.
                // The body is brace-skipped and preserved as a span;
                // the declaration it lowers to carries
                // `ArrowBody::Unparsed(body_span)` so its signature
                // flows forward and callers can type-check against
                // it, but the body stays scaffolded until the M2+
                // surface grammar covers match/pipe/lambda/etc.
                let open = self.peek().span.clone();
                let end = self.skip_brace_balanced()?;
                let body_span = SourceSpan::new(self.file, open.byte_start, end);
                Ok(SurfaceItem::FnExternalBody {
                    name,
                    params,
                    return_type,
                    body_span,
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
                payload: VariantPayload::Positional(Vec::new()),
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
        let op = match &self.peek().kind {
            TokenKind::EqEq => Some(crate::operators::OperatorKind::Comparison(
                crate::operators::ComparisonOp::Eq,
            )),
            TokenKind::NotEq => Some(crate::operators::OperatorKind::Comparison(
                crate::operators::ComparisonOp::Ne,
            )),
            TokenKind::Lt => Some(crate::operators::OperatorKind::Comparison(
                crate::operators::ComparisonOp::Lt,
            )),
            TokenKind::Le => Some(crate::operators::OperatorKind::Comparison(
                crate::operators::ComparisonOp::Le,
            )),
            TokenKind::Gt => Some(crate::operators::OperatorKind::Comparison(
                crate::operators::ComparisonOp::Gt,
            )),
            TokenKind::Ge => Some(crate::operators::OperatorKind::Comparison(
                crate::operators::ComparisonOp::Ge,
            )),
            _ => None,
        };
        if let Some(op) = op {
            self.bump();
            let rhs = self.parse_additive()?;
            let start = expr_span(&lhs).byte_start;
            let end = expr_span(&rhs).byte_end;
            Ok(SurfaceExpr::Operator {
                op,
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
            let op = match &self.peek().kind {
                TokenKind::Plus => crate::operators::OperatorKind::Arithmetic(
                    crate::operators::ArithmeticOp::Add,
                ),
                TokenKind::Minus => crate::operators::OperatorKind::Arithmetic(
                    crate::operators::ArithmeticOp::Sub,
                ),
                _ => break,
            };
            self.bump();
            let rhs = self.parse_term()?;
            let start = expr_span(&lhs).byte_start;
            let end = expr_span(&rhs).byte_end;
            lhs = SurfaceExpr::Operator {
                op,
                args: vec![lhs, rhs],
                span: SourceSpan::new(self.file, start, end),
            };
        }
        Ok(lhs)
    }

    fn parse_term(&mut self) -> Result<SurfaceExpr, Diagnostic> {
        let mut lhs = self.parse_primary()?;
        loop {
            let op = match &self.peek().kind {
                TokenKind::Star => crate::operators::OperatorKind::Arithmetic(
                    crate::operators::ArithmeticOp::Mul,
                ),
                TokenKind::Slash => crate::operators::OperatorKind::Arithmetic(
                    crate::operators::ArithmeticOp::Div,
                ),
                _ => break,
            };
            self.bump();
            let rhs = self.parse_primary()?;
            let start = expr_span(&lhs).byte_start;
            let end = expr_span(&rhs).byte_end;
            lhs = SurfaceExpr::Operator {
                op,
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
        if matches!(self.peek().kind, TokenKind::KwMatch) {
            return self.parse_match();
        }
        let token = self.bump().clone();
        match token.kind {
            TokenKind::IntLit(value) => Ok(SurfaceExpr::Literal {
                value: SurfaceLiteral::Int(value),
                span: token.span,
            }),
            TokenKind::KwTrue => Ok(SurfaceExpr::Literal {
                value: SurfaceLiteral::Bool(true),
                span: token.span,
            }),
            TokenKind::KwFalse => Ok(SurfaceExpr::Literal {
                value: SurfaceLiteral::Bool(false),
                span: token.span,
            }),
            TokenKind::StringLit(value) => Ok(SurfaceExpr::Literal {
                value: SurfaceLiteral::String(value),
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
                } else if matches!(self.peek().kind, TokenKind::Dot) {
                    // Member-access chain: Ident (. Ident)+. Always
                    // reads at least one additional segment because
                    // the Dot is in the peek. Lowers to
                    // `SurfaceExpr::Path` for downstream resolution
                    // via top-level symbol + Conj-child walk by label.
                    let mut segments = vec![name];
                    let start = token.span.byte_start;
                    let mut end = token.span.byte_end;
                    while matches!(self.peek().kind, TokenKind::Dot) {
                        self.bump();
                        let next = self.bump().clone();
                        match next.kind {
                            TokenKind::Ident(n) => {
                                end = next.span.byte_end;
                                segments.push(n);
                            }
                            other => {
                                return Err(Diagnostic::ParseError {
                                    message: format!(
                                        "expected identifier after `.` in dotted path, got {other:?}"
                                    ),
                                    span: next.span,
                                });
                            }
                        }
                    }
                    Ok(SurfaceExpr::Path {
                        segments,
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

    /// Parse a `match <scrutinee> { <pattern> => <expr> ... }`
    /// expression. At M1(2.8) patterns are limited to bare variant
    /// constructors (`Ident`) — no wildcards, destructuring, or
    /// nested patterns. Arms are brace-separated with optional
    /// comma between them for readability; we accept either.
    fn parse_match(&mut self) -> Result<SurfaceExpr, Diagnostic> {
        let match_token = self.bump().clone();
        debug_assert!(matches!(match_token.kind, TokenKind::KwMatch));
        let scrutinee = self.parse_expr()?;
        self.expect_kind(TokenKind::LBrace)?;
        let mut arms: Vec<SurfaceMatchArm> = Vec::new();
        while !matches!(self.peek().kind, TokenKind::RBrace) {
            arms.push(self.parse_match_arm()?);
            // Accept an optional comma between arms.
            if matches!(self.peek().kind, TokenKind::Comma) {
                self.bump();
            }
        }
        let close = self.expect_kind(TokenKind::RBrace)?;
        let start = match_token.span.byte_start;
        let end = close.span.byte_end;
        Ok(SurfaceExpr::Match {
            scrutinee: Box::new(scrutinee),
            arms,
            span: SourceSpan::new(self.file, start, end),
        })
    }

    fn parse_match_arm(&mut self) -> Result<SurfaceMatchArm, Diagnostic> {
        let pattern_token = self.bump().clone();
        let pattern = match &pattern_token.kind {
            TokenKind::Ident(name) => SurfacePattern::BareVariant {
                name: name.clone(),
                span: pattern_token.span.clone(),
            },
            other => {
                return Err(Diagnostic::ParseError {
                    message: format!(
                        "expected variant name in match pattern, got {other:?} — M1(2.8) supports bare-variant patterns only (e.g. `True => expr`)"
                    ),
                    span: pattern_token.span,
                });
            }
        };
        self.expect_kind(TokenKind::FatArrow)?;
        let body = self.parse_expr()?;
        let start = pattern_token.span.byte_start;
        let end = expr_span(&body).byte_end;
        Ok(SurfaceMatchArm {
            pattern,
            body,
            span: SourceSpan::new(self.file, start, end),
        })
    }
}

fn expr_span(expr: &SurfaceExpr) -> &SourceSpan {
    match expr {
        SurfaceExpr::Literal { span, .. }
        | SurfaceExpr::Var { span, .. }
        | SurfaceExpr::Path { span, .. }
        | SurfaceExpr::Call { span, .. }
        | SurfaceExpr::Operator { span, .. }
        | SurfaceExpr::If { span, .. }
        | SurfaceExpr::Match { span, .. }
        | SurfaceExpr::Record { span, .. } => span,
    }
}
