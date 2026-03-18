//! DSL FnBody → abstract IR compiler.
//!
//! Translates DSL expressions and statements (`daglang_syntax::ast::Expr` /
//! `Stmt`) into the shared abstract IR (`gunbc_ir::code_ir::Expr` / `Stmt`).
//! Because the output is target-agnostic IR, the generated functions flow
//! through all backends (Rust, Go, C, MIPS) via the existing lowering and
//! rendering pipeline.
//!
//! ## Pipe chain strategy
//!
//! DSL pipe chains (`|>`) are compiled to `For` loops inside `Expr::Block`
//! to ensure cross-target compatibility. This avoids closures (unsupported
//! in C) and method-chain idioms that not all backends can render.
//!
//! ## v2 Bootstrap Scaffolding (TEMPORARY — remove after self-hosting)
//!
//! The following functions are workarounds for the fn_codegen pipeline having
//! no type information. They exist to bootstrap the v2 compiler and should be
//! removed once self-hosting is achieved.
//!
//! **Remaining workaround functions:**
//!
//! - `clone_if_needed()`: Adds `.clone()` to variable/field expressions passed
//!   as arguments or struct fields. Prevents use-after-move. Proper fix: the
//!   v2 compiler should track ownership.
//!
//! - `escape_rust_keyword()`: Prefixes Rust keywords with `r#`. Needed because
//!   the DSL allows keywords as variable names.
//!
//! **Removed (no longer needed):**
//!
//! - `is_numeric_expr()`, `is_list_expr()`, `is_likely_list_concat()`, etc.:
//!   Guessed whether `+` was arithmetic vs concatenation. Removed because the
//!   DAG language now uses `concat()` for concatenation and `+` exclusively
//!   for arithmetic.
//!
//! - `compile_string_concat()`, `flatten_concat_parts()`: Compiled `+` chains
//!   with string literals to `format!()`. Removed — same reason.
//!
//! **Heuristic data in `v2_crate_emit.rs` (still needed):**
//!
//! - `std_types_prelude()`: Materializes types from `std.types` imports.
//! - Hardcoded `struct_field_types` entries for materialized types.
//! - `module_prelude()`: Hardcoded cross-module `use` statements.

use daglang_syntax::ast;
use daglang_syntax::ast_utils::ExprIdentity;
use gunbc_ir::code_ir;
use gunbc_ir::code_ir::IrType;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::type_codegen::to_snake_case;

// ---------------------------------------------------------------------------
// TypeExpr → IrType conversion
// ---------------------------------------------------------------------------

/// Convert a DSL `TypeExpr` to a target-agnostic `IrType`.
///
/// This is the single source of truth for mapping .dag types to the IR type
/// system. Backends then render `IrType` to their target language.
pub fn type_expr_to_ir_type(expr: &ast::TypeExpr) -> IrType {
    match expr {
        ast::TypeExpr::Named(n) => match n.as_str() {
            "Bool" => IrType::Bool,
            "Int" => IrType::Int,
            "String" => IrType::Str,
            _ => IrType::Named(n.clone()),
        },
        ast::TypeExpr::AssociatedOutput(base) => IrType::Named(format!("{base}::Output")),
        ast::TypeExpr::Generic(n, args) => {
            IrType::Generic(n.clone(), args.iter().map(type_expr_to_ir_type).collect())
        }
        ast::TypeExpr::Function(_, _) => {
            IrType::Named(daglang_syntax::ast_utils::type_expr_to_string(expr))
        }
        ast::TypeExpr::Optional(inner) => IrType::Optional(Box::new(type_expr_to_ir_type(inner))),
        ast::TypeExpr::Refined(inner, _) => type_expr_to_ir_type(inner),
        ast::TypeExpr::Record(fields) => IrType::Record(
            fields
                .iter()
                .map(|f| (f.name.clone(), type_expr_to_ir_type(&f.ty)))
                .collect(),
        ),
    }
}

/// Context for compiling DSL function bodies.
///
/// Carries the set of data table names defined in the module so that
/// identifier references can be mapped to their SCREAMING_SNAKE_CASE
/// static names in the generated output, and struct field optionality
/// information for automatic `Some()` wrapping.
#[derive(Clone)]
pub struct CompileContext {
    /// Names of `data` definitions visible in this module.
    pub data_names: HashSet<String>,
    /// Data definition name → IrType for collection element inference.
    pub data_ir_types: HashMap<String, IrType>,
    /// Names of `data` definitions that are Map types (need `&` reference, not `.clone()`).
    pub data_map_names: HashSet<String>,
    /// Map from struct name → set of field names that are `Option<T>`.
    pub optional_fields: std::collections::HashMap<String, HashSet<String>>,
    /// Map from bare variant name → parent enum name (e.g. "ZeroWidth" → "DisplayWidth").
    /// Ambiguous variants (present in multiple enums) are excluded.
    pub variant_to_enum: std::collections::HashMap<String, String>,
    /// Map from struct name → (field name → field type name) for contextual resolution.
    pub struct_field_types:
        std::collections::HashMap<String, std::collections::HashMap<String, String>>,
    /// Map from enum name → set of variant names, for field-type-based disambiguation.
    pub enum_variants: std::collections::HashMap<String, HashSet<String>>,
    /// Set of (type_name, field_name) pairs that need Box<> wrapping (recursive types).
    pub boxed_fields: HashSet<(String, String)>,
    /// Map from function name → return type name (for v2 crate emit).
    pub fn_return_types: std::collections::HashMap<String, String>,
    /// Map from function name → exact return IrType when only signature metadata is available.
    pub fn_return_ir_types: HashMap<String, IrType>,
    /// Map from function name → ordered parameter names and their type names.
    pub fn_param_types: std::collections::HashMap<String, Vec<(String, String)>>,
    /// Set of parameter names that are Optional (for v2 crate emit).
    pub optional_params: HashSet<String>,
    /// Map from parameter name → type name (for v2 crate emit).
    pub param_types: std::collections::HashMap<String, String>,
    /// Current function's return type name (for variant disambiguation in return position).
    pub current_return_type: Option<String>,
    /// Target-agnostic type scope: variable name → IrType (populated from function params
    /// and augmented as let bindings are compiled).
    pub ir_scope: HashMap<String, IrType>,
    /// Current function's return type as IrType (for fold accumulator inference).
    pub current_return_ir_type: Option<IrType>,
    /// Struct/variant name → [(field_name, IrType)] for populating `Expr::Struct.field_types`.
    pub struct_field_ir_types: HashMap<String, Vec<(String, IrType)>>,
    /// Variable name → number of `Ident` references in the current function body.
    /// Used to elide `.clone()` when a variable is referenced only once (move suffices).
    pub use_counts: HashMap<String, usize>,
    /// When compiling inside a fold lambda body, the name of the accumulator parameter.
    /// Used by concat codegen to strip the accumulator's `.clone()` (safe because the
    /// accumulator is reassigned each iteration, so try_unwrap succeeds → in-place extend).
    pub fold_accum_name: Option<String>,
    /// Map from enum name → set of field names that exist on ALL variants (common fields).
    /// Field access on these fields compiles to accessor method calls instead of direct
    /// field access, since Rust enums don't support direct field access.
    pub enum_accessor_fields: HashMap<String, HashSet<String>>,
    /// Set of function names whose return type is Optional (T?).
    pub optional_return_fns: HashSet<String>,
    /// Typecheck-produced anonymous-record constructor targets keyed by stable AST identity.
    pub anonymous_record_targets: HashMap<ExprIdentity, String>,
    /// Typecheck-produced synthesized anonymous-record types for this callable.
    pub synthesized_anonymous_record_types: Vec<SynthesizedAnonymousRecordType>,
    /// Typecheck-produced resolved expression IrTypes keyed by stable AST identity.
    pub expr_ir_types: HashMap<ExprIdentity, IrType>,
    /// Local mapping from this function body's structural expression paths back to
    /// stable identities produced by typecheck.
    pub(crate) expr_identities: HashMap<ExprPath, ExprIdentity>,
    pub(crate) expr_path: RefCell<ExprPath>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub(crate) struct ExprPath(Vec<ExprPathStep>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum StmtExprSlot {
    LetValue,
    AssignValue,
    NodeExpr,
    NodeGuard,
    ExprStmt,
    ReturnField(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ExprPathStep {
    BodyStmt { index: usize, slot: StmtExprSlot },
    BlockStmt { index: usize, slot: StmtExprSlot },
    ForBodyStmt { index: usize, slot: StmtExprSlot },
    FieldAccessBase,
    CallArg(usize),
    ServiceArg(usize),
    BinOpLeft,
    BinOpRight,
    UnaryInner,
    LambdaBody,
    ForIterable,
    ForBodyExpr,
    StringPart(usize),
    RecordField(usize),
    MatchScrutinee,
    MatchArmGuard(usize),
    MatchArmBody(usize),
    IfCond,
    IfThen,
    IfElse,
    ListItem(usize),
    MapKey(usize),
    MapValue(usize),
    GuardedInner,
    GuardedGuard,
    AfterInner,
}

#[derive(Debug, Clone)]
enum StmtPathOwner {
    Body,
    Block(ExprPath),
    ForBody(ExprPath),
}

impl ExprPath {
    fn child(&self, step: ExprPathStep) -> Self {
        let mut steps = self.0.clone();
        steps.push(step);
        Self(steps)
    }
}

fn stmt_expr_path(owner: &StmtPathOwner, index: usize, slot: StmtExprSlot) -> ExprPath {
    match owner {
        StmtPathOwner::Body => ExprPath(vec![ExprPathStep::BodyStmt { index, slot }]),
        StmtPathOwner::Block(path) => path.child(ExprPathStep::BlockStmt { index, slot }),
        StmtPathOwner::ForBody(path) => path.child(ExprPathStep::ForBodyStmt { index, slot }),
    }
}

fn record_field_path(path: &ExprPath, index: usize) -> ExprPath {
    path.child(ExprPathStep::RecordField(index))
}

fn build_expr_identities(body: &ast::FnBody) -> HashMap<ExprPath, ExprIdentity> {
    fn walk_stmt_sequence(
        stmts: &[ast::Stmt],
        owner: &StmtPathOwner,
        next_identity: &mut usize,
        expr_identities: &mut HashMap<ExprPath, ExprIdentity>,
    ) {
        for (index, stmt) in stmts.iter().enumerate() {
            match stmt {
                ast::Stmt::Let(_, expr) => walk_expr(
                    expr,
                    stmt_expr_path(owner, index, StmtExprSlot::LetValue),
                    next_identity,
                    expr_identities,
                ),
                ast::Stmt::Assign(_, expr) => walk_expr(
                    expr,
                    stmt_expr_path(owner, index, StmtExprSlot::AssignValue),
                    next_identity,
                    expr_identities,
                ),
                ast::Stmt::Expr(expr) => walk_expr(
                    expr,
                    stmt_expr_path(owner, index, StmtExprSlot::ExprStmt),
                    next_identity,
                    expr_identities,
                ),
                ast::Stmt::Node(node) => {
                    walk_expr(
                        &node.expr,
                        stmt_expr_path(owner, index, StmtExprSlot::NodeExpr),
                        next_identity,
                        expr_identities,
                    );
                    if let Some(guard) = &node.when_guard {
                        walk_expr(
                            guard,
                            stmt_expr_path(owner, index, StmtExprSlot::NodeGuard),
                            next_identity,
                            expr_identities,
                        );
                    }
                }
                ast::Stmt::Return(fields) => {
                    for (field_index, (_, expr)) in fields.iter().enumerate() {
                        walk_expr(
                            expr,
                            stmt_expr_path(owner, index, StmtExprSlot::ReturnField(field_index)),
                            next_identity,
                            expr_identities,
                        );
                    }
                }
            }
        }
    }

    fn walk_expr(
        expr: &ast::Expr,
        path: ExprPath,
        next_identity: &mut usize,
        expr_identities: &mut HashMap<ExprPath, ExprIdentity>,
    ) {
        let expr_identity = ExprIdentity(*next_identity);
        *next_identity += 1;
        expr_identities.insert(path.clone(), expr_identity);
        match expr {
            ast::Expr::Literal(_) | ast::Expr::Ident(_) => {}
            ast::Expr::FieldAccess(base, _) => walk_expr(
                base,
                path.child(ExprPathStep::FieldAccessBase),
                next_identity,
                expr_identities,
            ),
            ast::Expr::Call(_, args) => {
                for (index, (_, arg)) in args.iter().enumerate() {
                    walk_expr(
                        arg,
                        path.child(ExprPathStep::CallArg(index)),
                        next_identity,
                        expr_identities,
                    );
                }
            }
            ast::Expr::ServiceCall(_, args) => {
                for (index, (_, arg)) in args.iter().enumerate() {
                    walk_expr(
                        arg,
                        path.child(ExprPathStep::ServiceArg(index)),
                        next_identity,
                        expr_identities,
                    );
                }
            }
            ast::Expr::BinOp(left, _, right) => {
                walk_expr(
                    left,
                    path.child(ExprPathStep::BinOpLeft),
                    next_identity,
                    expr_identities,
                );
                walk_expr(
                    right,
                    path.child(ExprPathStep::BinOpRight),
                    next_identity,
                    expr_identities,
                );
            }
            ast::Expr::UnaryOp(_, inner)
            | ast::Expr::Lambda(_, inner)
            | ast::Expr::After(inner, _) => {
                let step = match expr {
                    ast::Expr::UnaryOp(_, _) => ExprPathStep::UnaryInner,
                    ast::Expr::Lambda(_, _) => ExprPathStep::LambdaBody,
                    ast::Expr::After(_, _) => ExprPathStep::AfterInner,
                    _ => unreachable!(),
                };
                walk_expr(inner, path.child(step), next_identity, expr_identities);
            }
            ast::Expr::StringInterp(parts) => {
                for (index, part) in parts.iter().enumerate() {
                    if let ast::StringPart::Expr(inner) = part {
                        walk_expr(
                            inner,
                            path.child(ExprPathStep::StringPart(index)),
                            next_identity,
                            expr_identities,
                        );
                    }
                }
            }
            ast::Expr::Record(_, fields) | ast::Expr::Return(fields) => {
                for (index, (_, field_expr)) in fields.iter().enumerate() {
                    walk_expr(
                        field_expr,
                        record_field_path(&path, index),
                        next_identity,
                        expr_identities,
                    );
                }
            }
            ast::Expr::Match(scrutinee, arms) => {
                walk_expr(
                    scrutinee,
                    path.child(ExprPathStep::MatchScrutinee),
                    next_identity,
                    expr_identities,
                );
                for (index, arm) in arms.iter().enumerate() {
                    if let Some(guard) = &arm.guard {
                        walk_expr(
                            guard,
                            path.child(ExprPathStep::MatchArmGuard(index)),
                            next_identity,
                            expr_identities,
                        );
                    }
                    walk_expr(
                        &arm.body,
                        path.child(ExprPathStep::MatchArmBody(index)),
                        next_identity,
                        expr_identities,
                    );
                }
            }
            ast::Expr::If(cond, then_expr, else_expr) => {
                walk_expr(
                    cond,
                    path.child(ExprPathStep::IfCond),
                    next_identity,
                    expr_identities,
                );
                walk_expr(
                    then_expr,
                    path.child(ExprPathStep::IfThen),
                    next_identity,
                    expr_identities,
                );
                if let Some(otherwise) = else_expr {
                    walk_expr(
                        otherwise,
                        path.child(ExprPathStep::IfElse),
                        next_identity,
                        expr_identities,
                    );
                }
            }
            ast::Expr::For(_, iterable, _, body) => {
                walk_expr(
                    iterable,
                    path.child(ExprPathStep::ForIterable),
                    next_identity,
                    expr_identities,
                );
                match body {
                    ast::ForBody::Expr(expr) => walk_expr(
                        expr,
                        path.child(ExprPathStep::ForBodyExpr),
                        next_identity,
                        expr_identities,
                    ),
                    ast::ForBody::Block(stmts) => walk_stmt_sequence(
                        stmts,
                        &StmtPathOwner::ForBody(path.clone()),
                        next_identity,
                        expr_identities,
                    ),
                }
            }
            ast::Expr::List(items) => {
                for (index, item) in items.iter().enumerate() {
                    walk_expr(
                        item,
                        path.child(ExprPathStep::ListItem(index)),
                        next_identity,
                        expr_identities,
                    );
                }
            }
            ast::Expr::Map(entries) => {
                for (index, (key, value)) in entries.iter().enumerate() {
                    walk_expr(
                        key,
                        path.child(ExprPathStep::MapKey(index)),
                        next_identity,
                        expr_identities,
                    );
                    walk_expr(
                        value,
                        path.child(ExprPathStep::MapValue(index)),
                        next_identity,
                        expr_identities,
                    );
                }
            }
            ast::Expr::Guarded(inner, guard) => {
                walk_expr(
                    inner,
                    path.child(ExprPathStep::GuardedInner),
                    next_identity,
                    expr_identities,
                );
                walk_expr(
                    guard,
                    path.child(ExprPathStep::GuardedGuard),
                    next_identity,
                    expr_identities,
                );
            }
            ast::Expr::Block(stmts) => walk_stmt_sequence(
                stmts,
                &StmtPathOwner::Block(path),
                next_identity,
                expr_identities,
            ),
        }
    }

    let mut expr_identities = HashMap::new();
    let mut next_identity = 0usize;
    walk_stmt_sequence(
        &body.stmts,
        &StmtPathOwner::Body,
        &mut next_identity,
        &mut expr_identities,
    );
    expr_identities
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynthesizedAnonymousRecordType {
    pub name: String,
    pub fields: Vec<(String, IrType)>,
}

impl Default for CompileContext {
    fn default() -> Self {
        Self::new()
    }
}

impl CompileContext {
    pub fn new() -> Self {
        Self {
            data_names: HashSet::new(),
            data_ir_types: HashMap::new(),
            data_map_names: HashSet::new(),
            optional_fields: std::collections::HashMap::new(),
            variant_to_enum: std::collections::HashMap::new(),
            struct_field_types: std::collections::HashMap::new(),
            enum_variants: std::collections::HashMap::new(),
            boxed_fields: HashSet::new(),
            fn_return_types: std::collections::HashMap::new(),
            fn_return_ir_types: HashMap::new(),
            fn_param_types: std::collections::HashMap::new(),
            optional_params: HashSet::new(),
            param_types: std::collections::HashMap::new(),
            current_return_type: None,
            current_return_ir_type: None,
            ir_scope: HashMap::new(),
            struct_field_ir_types: HashMap::new(),
            use_counts: HashMap::new(),
            fold_accum_name: None,
            enum_accessor_fields: HashMap::new(),
            optional_return_fns: HashSet::new(),
            anonymous_record_targets: HashMap::new(),
            synthesized_anonymous_record_types: Vec::new(),
            expr_ir_types: HashMap::new(),
            expr_identities: HashMap::new(),
            expr_path: RefCell::new(ExprPath::default()),
        }
    }

    fn current_expr_path(&self) -> ExprPath {
        self.expr_path.borrow().clone()
    }

    fn with_expr_path<T>(&self, path: ExprPath, f: impl FnOnce() -> T) -> T {
        let previous = self.expr_path.replace(path);
        let result = f();
        self.expr_path.replace(previous);
        result
    }

    fn with_child_expr_path<T>(&self, step: ExprPathStep, f: impl FnOnce() -> T) -> T {
        self.with_expr_path(self.current_expr_path().child(step), f)
    }

    fn anonymous_record_target(&self) -> Option<&str> {
        let expr_identity = self.expr_identities.get(&self.current_expr_path())?;
        self.anonymous_record_targets
            .get(expr_identity)
            .map(String::as_str)
    }

    fn current_expr_ir_type(&self) -> Option<IrType> {
        let expr_identity = self.expr_identities.get(&self.current_expr_path())?;
        self.expr_ir_types.get(expr_identity).cloned()
    }

    fn populate_expr_identities(&mut self, body: &ast::FnBody) {
        self.expr_identities = build_expr_identities(body);
        self.expr_path.replace(ExprPath::default());
    }
}

/// Resolve a variant name to its parent enum, preferring the current function's
/// return type when the variant is ambiguous (exists in multiple enums).
fn resolve_variant_enum(name: &str, ctx: &CompileContext) -> Option<String> {
    // Check if variant exists in the current return type's enum (priority)
    if let Some(ret_type) = &ctx.current_return_type {
        if let Some(variants) = ctx.enum_variants.get(ret_type.as_str()) {
            if variants.contains(name) {
                return Some(ret_type.clone());
            }
        }
    }
    // Fall back to global variant_to_enum map
    ctx.variant_to_enum.get(name).cloned()
}

fn qualifies_variant(
    expected_type: Option<&str>,
    variant_name: &str,
    ctx: &CompileContext,
) -> bool {
    expected_type
        .and_then(|ty| ctx.enum_variants.get(ty).map(|variants| (ty, variants)))
        .is_some_and(|(_, variants)| variants.contains(variant_name))
}

fn qualified_variant_expr(
    expected_type: Option<&str>,
    variant_name: &str,
    ctx: &CompileContext,
) -> Option<code_ir::Expr> {
    if qualifies_variant(expected_type, variant_name, ctx) {
        Some(code_ir::Expr::Path(vec![
            expected_type.expect("checked above").to_string(),
            variant_name.to_string(),
        ]))
    } else {
        None
    }
}

fn lookup_call_arg_type<'a>(
    fn_name: &str,
    arg_index: usize,
    arg_name: Option<&str>,
    ctx: &'a CompileContext,
) -> Option<&'a str> {
    let params = ctx
        .fn_param_types
        .get(fn_name)
        .or_else(|| ctx.fn_param_types.get(&to_snake_case(fn_name)))?;
    if let Some(name) = arg_name {
        return params
            .iter()
            .find(|(param_name, _)| param_name == name)
            .map(|(_, ty)| ty.as_str());
    }
    params.get(arg_index).map(|(_, ty)| ty.as_str())
}

/// Count how many times each identifier is referenced in a statement list.
/// Used to determine whether a variable can be moved (count == 1) or must be cloned.
/// Uses a weight parameter: inside for/lambda bodies, weight is 2 so any captured
/// variable is treated as multi-use (the body executes multiple times).
fn count_ident_uses(stmts: &[ast::Stmt]) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for stmt in stmts {
        count_ident_uses_stmt(stmt, &mut counts, 1);
    }
    counts
}

fn count_ident_uses_stmt(stmt: &ast::Stmt, counts: &mut HashMap<String, usize>, weight: usize) {
    match stmt {
        ast::Stmt::Let(_, expr) | ast::Stmt::Assign(_, expr) | ast::Stmt::Expr(expr) => {
            count_ident_uses_expr(expr, counts, weight);
        }
        ast::Stmt::Return(fields) => {
            for (_, expr) in fields {
                count_ident_uses_expr(expr, counts, weight);
            }
        }
        ast::Stmt::Node(node) => {
            count_ident_uses_expr(&node.expr, counts, weight);
            if let Some(guard) = &node.when_guard {
                count_ident_uses_expr(guard, counts, weight);
            }
        }
    }
}

fn count_ident_uses_expr(expr: &ast::Expr, counts: &mut HashMap<String, usize>, weight: usize) {
    match expr {
        ast::Expr::Ident(name) => {
            *counts.entry(name.clone()).or_insert(0) += weight;
        }
        ast::Expr::Literal(_) => {}
        ast::Expr::FieldAccess(base, _) => {
            count_ident_uses_expr(base, counts, weight);
        }
        ast::Expr::Call(_, args) | ast::Expr::ServiceCall(_, args) => {
            for (_, arg) in args {
                count_ident_uses_expr(arg, counts, weight);
            }
        }
        ast::Expr::BinOp(lhs, _, rhs) => {
            count_ident_uses_expr(lhs, counts, weight);
            count_ident_uses_expr(rhs, counts, weight);
        }
        ast::Expr::UnaryOp(_, operand) => {
            count_ident_uses_expr(operand, counts, weight);
        }
        ast::Expr::StringInterp(parts) => {
            for part in parts {
                if let ast::StringPart::Expr(e) = part {
                    count_ident_uses_expr(e, counts, weight);
                }
            }
        }
        ast::Expr::Record(_, fields) => {
            for (_, expr) in fields {
                count_ident_uses_expr(expr, counts, weight);
            }
        }
        ast::Expr::Match(scrutinee, arms) => {
            count_ident_uses_expr(scrutinee, counts, weight);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    count_ident_uses_expr(guard, counts, weight);
                }
                count_ident_uses_expr(&arm.body, counts, weight);
            }
        }
        ast::Expr::If(cond, then_expr, else_expr) => {
            count_ident_uses_expr(cond, counts, weight);
            count_ident_uses_expr(then_expr, counts, weight);
            if let Some(e) = else_expr {
                count_ident_uses_expr(e, counts, weight);
            }
        }
        ast::Expr::For(_, iterable, _, body) => {
            count_ident_uses_expr(iterable, counts, weight);
            // Body executes multiple times — any captured variable needs clone.
            match body {
                ast::ForBody::Expr(e) => count_ident_uses_expr(e, counts, 2),
                ast::ForBody::Block(stmts) => {
                    for s in stmts {
                        count_ident_uses_stmt(s, counts, 2);
                    }
                }
            }
        }
        ast::Expr::Lambda(params, body) => {
            // Lambda may be called multiple times — treat captures as multi-use.
            // But lambda parameters are local — count them separately and exclude
            // from the outer scope to avoid spurious .clone() after substitution.
            let mut inner_counts = HashMap::new();
            count_ident_uses_expr(body, &mut inner_counts, 1);
            for (name, inner_count) in inner_counts {
                if !params.contains(&name) {
                    // Captured variable — weight=2 since lambda may run multiple times
                    *counts.entry(name).or_insert(0) += inner_count.max(1) * 2;
                }
            }
        }
        ast::Expr::List(elems) => {
            for e in elems {
                count_ident_uses_expr(e, counts, weight);
            }
        }
        ast::Expr::Map(pairs) => {
            for (k, v) in pairs {
                count_ident_uses_expr(k, counts, weight);
                count_ident_uses_expr(v, counts, weight);
            }
        }
        ast::Expr::Guarded(expr, guard) => {
            count_ident_uses_expr(expr, counts, weight);
            count_ident_uses_expr(guard, counts, weight);
        }
        ast::Expr::After(expr, _) => {
            count_ident_uses_expr(expr, counts, weight);
        }
        ast::Expr::Return(fields) => {
            for (_, expr) in fields {
                count_ident_uses_expr(expr, counts, weight);
            }
        }
        ast::Expr::Block(stmts) => {
            for s in stmts {
                count_ident_uses_stmt(s, counts, weight);
            }
        }
    }
}

/// Compile a DSL `FnBody` into a list of abstract IR statements.
pub fn compile_fn_body(body: &ast::FnBody, ctx: &CompileContext) -> Vec<code_ir::Stmt> {
    let mut ctx = ctx.clone();
    ctx.populate_expr_identities(body);
    ctx.use_counts = count_ident_uses(&body.stmts);
    let mut counter: usize = 0;
    compile_stmt_sequence(&body.stmts, &ctx, &mut counter, &StmtPathOwner::Body)
}

/// Check if an AST expression clearly produces an Option<T> value.
fn is_clearly_optional_ast_expr(expr: &ast::Expr) -> bool {
    match expr {
        ast::Expr::Record(Some(name), _) if name == "Some" || name == "None" => true,
        ast::Expr::Call(name, _) if name == "Some" || name == "parse_int" => true,
        ast::Expr::Ident(name) if name == "null" => true,
        ast::Expr::Literal(ast::Literal::None) => true,
        ast::Expr::If(_, then_expr, Some(else_expr)) => {
            is_null_ast_expr(then_expr) || is_null_ast_expr(else_expr)
        }
        _ => false,
    }
}

/// Generate a unique temporary variable name from a monotonic counter.
fn fresh(counter: &mut usize, prefix: &str) -> String {
    let n = *counter;
    *counter += 1;
    format!("__{prefix}_{n}")
}

fn stmt_binding(stmt: &ast::Stmt) -> Option<(&str, &ast::Expr)> {
    match stmt {
        ast::Stmt::Let(name, expr) => Some((name.as_str(), expr)),
        ast::Stmt::Node(ns) => Some((ns.name.as_str(), &ns.expr)),
        _ => None,
    }
}

fn track_binding_before_compile(stmt: &ast::Stmt, ctx: &mut CompileContext) {
    let Some((name, expr)) = stmt_binding(stmt) else {
        return;
    };
    if is_clearly_optional_ast_expr(expr)
        || infer_known_expr_ir_type(expr, ctx).is_some_and(|ty| matches!(ty, IrType::Optional(_)))
    {
        ctx.optional_params.insert(name.to_string());
    }
}

fn track_binding_after_compile(
    stmt: &ast::Stmt,
    compiled: &code_ir::Stmt,
    ctx: &mut CompileContext,
) {
    let Some((name, _)) = stmt_binding(stmt) else {
        return;
    };
    if let code_ir::Stmt::Let {
        ir_type: Some(ref ty),
        ..
    } = compiled
    {
        ctx.ir_scope.insert(name.to_string(), ty.clone());
    }
}

fn compile_stmt_sequence(
    stmts: &[ast::Stmt],
    ctx: &CompileContext,
    counter: &mut usize,
    owner: &StmtPathOwner,
) -> Vec<code_ir::Stmt> {
    let len = stmts.len();
    let mut current_ctx = ctx.clone();
    let mut result = Vec::with_capacity(len);
    for (index, stmt) in stmts.iter().enumerate() {
        track_binding_before_compile(stmt, &mut current_ctx);
        let compiled = compile_stmt(stmt, index, index + 1 == len, &current_ctx, counter, owner);
        track_binding_after_compile(stmt, &compiled, &mut current_ctx);
        result.push(compiled);
    }
    result
}

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

fn compile_stmt(
    stmt: &ast::Stmt,
    stmt_index: usize,
    is_last: bool,
    ctx: &CompileContext,
    counter: &mut usize,
    owner: &StmtPathOwner,
) -> code_ir::Stmt {
    match stmt {
        ast::Stmt::Let(name, expr) => ctx.with_expr_path(
            stmt_expr_path(owner, stmt_index, StmtExprSlot::LetValue),
            || code_ir::Stmt::Let {
                name: escape_rust_keyword(name),
                mutable: false,
                expr: compile_expr(expr, ctx, counter),
                ir_type: ctx
                    .current_expr_ir_type()
                    .or_else(|| infer_known_expr_ir_type(expr, ctx)),
            },
        ),
        ast::Stmt::Assign(name, expr) => ctx.with_expr_path(
            stmt_expr_path(owner, stmt_index, StmtExprSlot::AssignValue),
            || code_ir::Stmt::Assign {
                dest: code_ir::Expr::Var(escape_rust_keyword(name)),
                value: compile_expr(expr, ctx, counter),
            },
        ),
        ast::Stmt::Node(ns) => ctx.with_expr_path(
            stmt_expr_path(owner, stmt_index, StmtExprSlot::NodeExpr),
            || code_ir::Stmt::Let {
                name: escape_rust_keyword(&ns.name),
                mutable: false,
                expr: compile_expr(&ns.expr, ctx, counter),
                ir_type: ctx
                    .current_expr_ir_type()
                    .or_else(|| infer_known_expr_ir_type(&ns.expr, ctx)),
            },
        ),
        ast::Stmt::Expr(expr) => ctx.with_expr_path(
            stmt_expr_path(owner, stmt_index, StmtExprSlot::ExprStmt),
            || {
                if is_last {
                    code_ir::Stmt::TailExpr(compile_expr(expr, ctx, counter))
                } else {
                    code_ir::Stmt::Expr(compile_expr(expr, ctx, counter))
                }
            },
        ),
        ast::Stmt::Return(fields) => {
            let ir_expr = compile_return_fields(fields, ctx, counter, |index| {
                stmt_expr_path(owner, stmt_index, StmtExprSlot::ReturnField(index))
            });
            if is_last {
                code_ir::Stmt::TailExpr(ir_expr)
            } else {
                code_ir::Stmt::Return(ir_expr)
            }
        }
    }
}

fn compile_return_fields(
    fields: &[(String, ast::Expr)],
    ctx: &CompileContext,
    counter: &mut usize,
    field_path: impl Fn(usize) -> ExprPath,
) -> code_ir::Expr {
    if fields.is_empty() {
        code_ir::Expr::Tuple(vec![])
    } else if fields.len() == 1 && (fields[0].0 == "value" || fields[0].0 == "return") {
        // Single-field return: unwrap the record to a bare expression.
        // DSL `return { return: expr }` and `return { value: expr }` both
        // mean "return this value".
        ctx.with_expr_path(field_path(0), || compile_expr(&fields[0].1, ctx, counter))
    } else {
        let field_names: HashSet<&str> = fields.iter().map(|(n, _)| n.as_str()).collect();
        let Some(struct_name) =
            explicit_record_struct_name(ctx.current_return_type.as_deref(), ctx)
        else {
            return unresolved_anonymous_record_error(&field_names);
        };
        compile_resolved_record_expr(&struct_name, fields, None, field_path, ctx, counter)
    }
}

/// Fill in missing optional struct fields.
///
/// Required fields are left missing so bad record construction fails visibly
/// downstream instead of being silently fabricated.
fn fill_missing_fields(
    struct_name: &str,
    provided: Vec<(String, code_ir::Expr)>,
    ctx: &CompileContext,
) -> Vec<(String, code_ir::Expr)> {
    let Some(field_types) = ctx.struct_field_types.get(struct_name) else {
        return provided;
    };
    let provided_names: HashSet<String> = provided.iter().map(|(n, _)| n.clone()).collect();
    // Only fill if ALL provided fields exist in the struct definition.
    // If any provided field is NOT in field_types, we may be matching the wrong
    // struct (same name, different module) — bail out.
    if !provided_names.iter().all(|n| field_types.contains_key(n)) {
        return provided;
    }
    let opt_set = ctx.optional_fields.get(struct_name);
    let mut result = provided;
    let mut field_names: Vec<&String> = field_types.keys().collect();
    field_names.sort();
    for field_name in field_names {
        if !provided_names.contains(field_name)
            && opt_set.is_some_and(|s| s.contains(field_name.as_str()))
        {
            result.push((
                field_name.clone(),
                code_ir::Expr::Path(vec!["None".to_string()]),
            ));
        }
    }
    result
}

fn expr_carries_optional_value(expr: &ast::Expr, ctx: &CompileContext) -> bool {
    match expr {
        ast::Expr::Ident(name) if ctx.optional_params.contains(name.as_str()) => true,
        _ => {
            is_clearly_optional_ast_expr(expr)
                || matches!(ctx.current_expr_ir_type(), Some(IrType::Optional(_)))
        }
    }
}

fn optional_to_required_field_error(target_struct: &str, field_name: &str) -> code_ir::Expr {
    code_ir::Expr::RawCode(format!(
        "compile_error!(\"cannot assign optional value to required field '{}.{}'; make the unwrap or fallback explicit in the source\")",
        target_struct, field_name
    ))
}

fn compile_struct_field_value(
    expr: &ast::Expr,
    expr_path: ExprPath,
    target_struct: &str,
    field_name: &str,
    field_types: Option<&std::collections::HashMap<String, String>>,
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::Expr {
    let compiled = ctx.with_expr_path(expr_path.clone(), || {
        compile_expr_in_field_context(expr, field_name, field_types, ctx, counter)
    });
    let is_opt = ctx
        .optional_fields
        .get(target_struct)
        .is_some_and(|fields| fields.contains(field_name));
    let is_none = is_none_expr(&compiled);
    let carries_optional_value =
        ctx.with_expr_path(expr_path.clone(), || expr_carries_optional_value(expr, ctx));
    if !is_opt && (is_none || carries_optional_value) {
        return optional_to_required_field_error(target_struct, field_name);
    }
    let already_optional = is_opt
        && ctx.with_expr_path(expr_path, || {
            is_already_optional_expr(expr, ctx, target_struct)
        });
    let compiled = if is_none {
        compiled
    } else {
        clone_if_needed(compiled, ctx.fold_accum_name.as_deref())
    };
    let mut result = if is_opt && !is_none && !already_optional {
        code_ir::Expr::Call {
            func: Box::new(code_ir::Expr::Var("Some".to_string())),
            args: vec![compiled],
            obligation: None,
        }
    } else {
        compiled
    };
    if needs_box_wrapping(target_struct, field_name, ctx) {
        result = code_ir::Expr::Call {
            func: Box::new(code_ir::Expr::Path(vec![
                "Box".to_string(),
                "new".to_string(),
            ])),
            args: vec![result],
            obligation: None,
        };
    }
    result
}

/// Infer a struct name from field names using the CompileContext's struct registry.
fn explicit_record_struct_name(
    preferred_struct: Option<&str>,
    ctx: &CompileContext,
) -> Option<String> {
    preferred_struct
        .filter(|struct_name| {
            !ctx.enum_variants.contains_key(*struct_name)
                && (ctx.struct_field_types.contains_key(*struct_name)
                    || ctx.struct_field_ir_types.contains_key(*struct_name))
        })
        .map(str::to_owned)
}

fn resolve_record_struct_name(
    preferred_struct: Option<&str>,
    ctx: &CompileContext,
) -> Option<String> {
    explicit_record_struct_name(preferred_struct, ctx).or_else(|| {
        ctx.anonymous_record_target()
            .and_then(|target| explicit_record_struct_name(Some(target), ctx))
    })
}

fn unresolved_anonymous_record_error(field_names: &HashSet<&str>) -> code_ir::Expr {
    let mut sorted_fields: Vec<&str> = field_names.iter().copied().collect();
    sorted_fields.sort_unstable();
    code_ir::Expr::RawCode(format!(
        "compile_error!(\"cannot resolve anonymous record type for fields [{}]; make the target struct explicit upstream\")",
        sorted_fields.join(", ")
    ))
}

fn compile_resolved_record_expr(
    struct_name: &str,
    fields: &[(String, ast::Expr)],
    rest: Option<code_ir::Expr>,
    field_path: impl Fn(usize) -> ExprPath,
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::Expr {
    let qualified_name = if let Some(enum_name) = ctx.variant_to_enum.get(struct_name) {
        format!("{enum_name}::{struct_name}")
    } else {
        struct_name.to_string()
    };
    let field_types = ctx.struct_field_types.get(struct_name);
    let ir_fields: Vec<(String, code_ir::Expr)> = fields
        .iter()
        .enumerate()
        .map(|(index, (name, expr))| {
            (
                name.clone(),
                compile_struct_field_value(
                    expr,
                    field_path(index),
                    struct_name,
                    name,
                    field_types,
                    ctx,
                    counter,
                ),
            )
        })
        .collect();
    let ir_fields = fill_missing_fields(struct_name, ir_fields, ctx);
    let ir_field_types = ctx.struct_field_ir_types.get(struct_name).cloned();
    code_ir::Expr::Struct {
        name: qualified_name,
        fields: ir_fields,
        rest: rest.map(Box::new),
        field_types: ir_field_types,
    }
}

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

fn compile_expr(expr: &ast::Expr, ctx: &CompileContext, counter: &mut usize) -> code_ir::Expr {
    match expr {
        ast::Expr::Literal(lit) => compile_literal(lit),
        ast::Expr::Ident(name) => compile_ident(name, ctx),
        ast::Expr::FieldAccess(receiver, field) => {
            // .value on an Option<T> (from Some { value: x } access) → .unwrap()
            if field == "value"
                && (ctx
                    .with_child_expr_path(ExprPathStep::FieldAccessBase, || {
                        ctx.current_expr_ir_type()
                    })
                    .is_some_and(|ty| matches!(ty, IrType::Optional(_)))
                    || is_likely_option_receiver_ctx(receiver, ctx))
            {
                return code_ir::Expr::MethodCall {
                    receiver: Box::new(
                        ctx.with_child_expr_path(ExprPathStep::FieldAccessBase, || {
                            compile_expr(receiver, ctx, counter)
                        }),
                    ),
                    method: "unwrap".to_string(),
                    args: vec![],
                };
            }
            // Enum common field access → method call (Rust enums don't support direct field access)
            if let Some(recv_type) =
                infer_known_expr_ir_type(receiver, ctx).and_then(|ty| named_type_from_ir(&ty))
            {
                if let Some(accessor_fields) = ctx.enum_accessor_fields.get(&recv_type) {
                    if accessor_fields.contains(field.as_str()) {
                        return code_ir::Expr::MethodCall {
                            receiver: Box::new(compile_expr(receiver, ctx, counter)),
                            method: field.clone(),
                            args: vec![],
                        };
                    }
                }
            } else {
                // Fallback: when the receiver type can't be inferred, check if
                // the field name is an enum accessor. Prefer method call when:
                // (a) no struct has this as a field, OR
                // (b) the receiver is a chained expression (likely a typed tree traversal).
                let is_enum_accessor = ctx
                    .enum_accessor_fields
                    .values()
                    .any(|fields| fields.contains(field.as_str()));
                if is_enum_accessor {
                    let is_struct_field = ctx
                        .struct_field_types
                        .values()
                        .any(|fields| fields.contains_key(field.as_str()));
                    // When the field name is ambiguous (exists on both enum and struct),
                    // use method call only for chained field access (x.typed.resolved_type)
                    // since that pattern is characteristic of typed tree traversal.
                    let use_accessor = !is_struct_field
                        || (matches!(receiver.as_ref(), ast::Expr::FieldAccess(_, inner) if inner == "typed"));
                    if use_accessor {
                        return code_ir::Expr::MethodCall {
                            receiver: Box::new(compile_expr(receiver, ctx, counter)),
                            method: field.clone(),
                            args: vec![],
                        };
                    }
                }
            }
            // DSL pair/tuple field names → Rust tuple indices
            let rust_field = match field.as_str() {
                "first" => "0".to_string(),
                "second" => "1".to_string(),
                other => other.to_string(),
            };
            let field_expr = code_ir::Expr::Field(
                Box::new(ctx.with_child_expr_path(ExprPathStep::FieldAccessBase, || {
                    compile_expr(receiver, ctx, counter)
                })),
                rust_field,
            );
            if is_boxed_field_access(receiver, field, ctx) {
                code_ir::Expr::Deref(Box::new(field_expr))
            } else {
                field_expr
            }
        }
        ast::Expr::Call(name, args) => {
            if let Some(intrinsic) = compile_intrinsic_call(name, args, ctx, counter) {
                intrinsic
            } else {
                compile_call(name, args, ctx, counter)
            }
        }
        ast::Expr::BinOp(left, op, right) => {
            if matches!(op, ast::BinOp::NullCoalesce) {
                // x ?? y → x.unwrap_or_else(|| y)
                code_ir::Expr::MethodCall {
                    receiver: Box::new(ctx.with_child_expr_path(ExprPathStep::BinOpLeft, || {
                        compile_expr(left, ctx, counter)
                    })),
                    method: "unwrap_or_else".to_string(),
                    args: vec![code_ir::Expr::Closure {
                        args: vec![],
                        body: Box::new(ctx.with_child_expr_path(ExprPathStep::BinOpRight, || {
                            compile_expr(right, ctx, counter)
                        })),
                    }],
                }
            } else {
                // All binary operators (including +) emit directly.
                // + is now exclusively arithmetic — string/list concat uses
                // the `concat()` intrinsic function instead.
                code_ir::Expr::BinOp {
                    left: Box::new(ctx.with_child_expr_path(ExprPathStep::BinOpLeft, || {
                        compile_expr(left, ctx, counter)
                    })),
                    op: compile_binop(op),
                    right: Box::new(ctx.with_child_expr_path(ExprPathStep::BinOpRight, || {
                        compile_expr(right, ctx, counter)
                    })),
                }
            }
        }
        ast::Expr::UnaryOp(op, expr) => code_ir::Expr::UnaryOp {
            op: compile_unaryop(op),
            expr: Box::new(ctx.with_child_expr_path(ExprPathStep::UnaryInner, || {
                compile_expr(expr, ctx, counter)
            })),
        },
        ast::Expr::Record(name, fields) => {
            // Special case: Some { value: x } → Some(compiled_x) in Rust
            if name.as_deref() == Some("Some") && fields.len() == 1 && fields[0].0 == "value" {
                let inner = ctx.with_child_expr_path(ExprPathStep::RecordField(0), || {
                    compile_expr(&fields[0].1, ctx, counter)
                });
                return code_ir::Expr::Call {
                    func: Box::new(code_ir::Expr::Var("Some".to_string())),
                    args: vec![inner],
                    obligation: None,
                };
            }
            // Special case: None { } → None
            if name.as_deref() == Some("None") && fields.is_empty() {
                return code_ir::Expr::Var("None".to_string());
            }
            if let Some(struct_name) = name.as_deref() {
                let current_path = ctx.current_expr_path();
                compile_resolved_record_expr(
                    struct_name,
                    fields,
                    None,
                    |index| record_field_path(&current_path, index),
                    ctx,
                    counter,
                )
            } else {
                let field_names: HashSet<&str> = fields.iter().map(|(n, _)| n.as_str()).collect();
                let Some(struct_name) = resolve_record_struct_name(None, ctx) else {
                    return unresolved_anonymous_record_error(&field_names);
                };
                let current_path = ctx.current_expr_path();
                compile_resolved_record_expr(
                    &struct_name,
                    fields,
                    None,
                    |index| record_field_path(&current_path, index),
                    ctx,
                    counter,
                )
            }
        }
        ast::Expr::Match(scrutinee, arms) => compile_match(scrutinee, arms, ctx, counter),
        ast::Expr::If(cond, then_expr, else_expr) => {
            compile_if(cond, then_expr, else_expr, ctx, counter)
        }
        ast::Expr::Lambda(params, body) => code_ir::Expr::Closure {
            args: params.clone(),
            body: Box::new(ctx.with_child_expr_path(ExprPathStep::LambdaBody, || {
                compile_expr(body, ctx, counter)
            })),
        },
        ast::Expr::List(elements) => {
            // DSL List<T> maps to Rust Rc<Vec<T>>, so use Rc::new(vec![...]).
            rc_wrap(code_ir::Expr::MacroCall {
                name: "vec".to_string(),
                args: elements
                    .iter()
                    .enumerate()
                    .map(|(index, e)| {
                        ctx.with_child_expr_path(ExprPathStep::ListItem(index), || {
                            compile_expr(e, ctx, counter)
                        })
                    })
                    .collect(),
            })
        }
        ast::Expr::StringInterp(parts) => compile_string_interp(parts, ctx, counter),
        ast::Expr::For(binding, iter_expr, _passthrough, body) => {
            let result_var = fresh(counter, "for_result");
            let iter = make_owned_iter(ctx.with_child_expr_path(ExprPathStep::ForIterable, || {
                compile_expr(iter_expr, ctx, counter)
            }));
            // Infer element type from body for Vec<T> annotation
            let body_ast_expr = match body {
                ast::ForBody::Expr(e) => Some(e.as_ref()),
                ast::ForBody::Block(stmts) => stmts.last().and_then(|s| match s {
                    ast::Stmt::Expr(e) => Some(e),
                    _ => None,
                }),
            };
            let list_type = match body {
                ast::ForBody::Expr(_) => body_ast_expr.and_then(|_| {
                    ctx.with_child_expr_path(ExprPathStep::ForBodyExpr, || {
                        ctx.current_expr_ir_type()
                    })
                }),
                ast::ForBody::Block(stmts) => stmts
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(index, stmt)| match stmt {
                        ast::Stmt::Expr(_) => Some(ctx.with_expr_path(
                            stmt_expr_path(
                                &StmtPathOwner::ForBody(ctx.current_expr_path()),
                                index,
                                StmtExprSlot::ExprStmt,
                            ),
                            || ctx.current_expr_ir_type(),
                        )),
                        _ => None,
                    })
                    .flatten(),
            }
            .map(|t| IrType::Generic("List".to_string(), vec![t]));
            let push_expr = match body {
                ast::ForBody::Expr(expr) => ctx
                    .with_child_expr_path(ExprPathStep::ForBodyExpr, || {
                        compile_expr(expr, ctx, counter)
                    }),
                ast::ForBody::Block(stmts) => code_ir::Expr::Block(compile_stmt_sequence(
                    stmts,
                    ctx,
                    counter,
                    &StmtPathOwner::ForBody(ctx.current_expr_path()),
                )),
            };
            code_ir::Expr::Block(vec![
                code_ir::Stmt::Let {
                    name: result_var.clone(),
                    mutable: true,
                    expr: code_ir::Expr::MacroCall {
                        name: "vec".to_string(),
                        args: vec![],
                    },
                    ir_type: list_type,
                },
                code_ir::Stmt::For {
                    binding: binding.clone(),
                    iter,
                    body: vec![code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                        receiver: Box::new(code_ir::Expr::Var(result_var.clone())),
                        method: "push".to_string(),
                        args: vec![push_expr],
                    })],
                },
                code_ir::Stmt::TailExpr(rc_wrap(code_ir::Expr::Var(result_var))),
            ])
        }
        ast::Expr::Return(fields) => {
            let current_path = ctx.current_expr_path();
            compile_return_fields(fields, ctx, counter, |index| {
                record_field_path(&current_path, index)
            })
        }
        ast::Expr::Guarded(inner, _) | ast::Expr::After(inner, _) => {
            let step = match expr {
                ast::Expr::Guarded(_, _) => ExprPathStep::GuardedInner,
                ast::Expr::After(_, _) => ExprPathStep::AfterInner,
                _ => unreachable!(),
            };
            ctx.with_child_expr_path(step, || compile_expr(inner, ctx, counter))
        }
        ast::Expr::Block(stmts) => code_ir::Expr::Block(compile_stmt_sequence(
            stmts,
            ctx,
            counter,
            &StmtPathOwner::Block(ctx.current_expr_path()),
        )),
        ast::Expr::ServiceCall(path, args) => {
            // Lower Resource.method(args) → v2_rt::resource_method(args)
            let fn_name = path
                .iter()
                .map(|s| crate::type_codegen::to_snake_case(s))
                .collect::<Vec<_>>()
                .join("_");
            let compiled_args: Vec<code_ir::Expr> = args
                .iter()
                .enumerate()
                .map(|(index, (_, e))| {
                    ctx.with_child_expr_path(ExprPathStep::ServiceArg(index), || {
                        compile_expr(e, ctx, counter)
                    })
                })
                .collect();
            code_ir::Expr::Call {
                func: Box::new(code_ir::Expr::Path(vec!["v2_rt".to_string(), fn_name])),
                args: compiled_args,
                obligation: None,
            }
        }
        ast::Expr::Map(_) => code_ir::Expr::RawCode(
            "compile_error!(\"unsupported DSL construct: Map not yet supported in fn codegen\");"
                .to_string(),
        ),
    }
}

fn compile_expr_typed(
    expr: &ast::Expr,
    ctx: &CompileContext,
    expected_type: Option<&str>,
    counter: &mut usize,
) -> code_ir::Expr {
    match expr {
        ast::Expr::Ident(name) => {
            if name == "null" {
                code_ir::Expr::Var("None".to_string())
            } else if let Some(path) = qualified_variant_expr(expected_type, name, ctx) {
                path
            } else {
                compile_ident(name, ctx)
            }
        }
        ast::Expr::Record(Some(record_name), fields)
            if qualifies_variant(expected_type, record_name, ctx) =>
        {
            let variant_field_types = ctx.struct_field_types.get(record_name.as_str());
            let ir_fields: Vec<(String, code_ir::Expr)> = fields
                .iter()
                .enumerate()
                .map(|(index, (field_name, field_expr))| {
                    (
                        field_name.clone(),
                        compile_struct_field_value(
                            field_expr,
                            record_field_path(&ctx.current_expr_path(), index),
                            record_name,
                            field_name,
                            variant_field_types,
                            ctx,
                            counter,
                        ),
                    )
                })
                .collect();
            let ir_fields = fill_missing_fields(record_name, ir_fields, ctx);
            code_ir::Expr::Struct {
                name: format!("{}::{}", expected_type.expect("checked above"), record_name),
                fields: ir_fields,
                rest: None,
                field_types: None,
            }
        }
        ast::Expr::Record(None, fields) => {
            let field_names: HashSet<&str> = fields.iter().map(|(name, _)| name.as_str()).collect();
            let Some(struct_name) = resolve_record_struct_name(expected_type, ctx) else {
                return unresolved_anonymous_record_error(&field_names);
            };
            let current_path = ctx.current_expr_path();
            compile_resolved_record_expr(
                &struct_name,
                fields,
                None,
                |index| record_field_path(&current_path, index),
                ctx,
                counter,
            )
        }
        ast::Expr::Match(scrutinee, arms) => {
            compile_match_typed(scrutinee, arms, ctx, expected_type, counter)
        }
        ast::Expr::If(cond, then_expr, else_expr) => {
            compile_if_typed(cond, then_expr, else_expr, ctx, expected_type, counter)
        }
        ast::Expr::Block(stmts) => code_ir::Expr::Block(compile_stmt_sequence_typed(
            stmts,
            ctx,
            expected_type,
            counter,
            &StmtPathOwner::Block(ctx.current_expr_path()),
        )),
        _ => compile_expr(expr, ctx, counter),
    }
}

// ---------------------------------------------------------------------------
// Literals
// ---------------------------------------------------------------------------

fn compile_literal(lit: &ast::Literal) -> code_ir::Expr {
    match lit {
        ast::Literal::Int(n) => code_ir::Expr::IntLit(*n),
        ast::Literal::Float(f) => code_ir::Expr::RawCode(format!("{f:?}_f64")),
        ast::Literal::String(s) => {
            // Emit owned String for v2 compiler compatibility.
            // The .dag language uses String everywhere, not &str.
            code_ir::Expr::MethodCall {
                receiver: Box::new(code_ir::Expr::Str(s.clone())),
                method: "to_string".to_string(),
                args: vec![],
            }
        }
        ast::Literal::Bool(b) => code_ir::Expr::BoolLit(*b),
        ast::Literal::None => code_ir::Expr::Var("None".to_string()),
    }
}

// ---------------------------------------------------------------------------
// Identifiers — bare names resolve to enum-variant paths
// ---------------------------------------------------------------------------

fn compile_ident(name: &str, ctx: &CompileContext) -> code_ir::Expr {
    if name == "null" || name == "None" {
        return code_ir::Expr::Var("None".to_string());
    }
    if ctx.data_map_names.contains(name) {
        // Map data tables are lazy_static HashMap constants. Emit `&TABLE` to produce
        // a reference — functions like `v2_rt::lookup` expect `&HashMap`.
        code_ir::Expr::Ref(Box::new(code_ir::Expr::Var(to_screaming_snake(name))))
    } else if ctx.data_names.contains(name) {
        // Non-map data tables (arrays, etc.) — clone the value.
        code_ir::Expr::MethodCall {
            receiver: Box::new(code_ir::Expr::Var(to_screaming_snake(name))),
            method: "clone".to_string(),
            args: vec![],
        }
    } else if let Some(enum_name) = resolve_variant_enum(name, ctx) {
        code_ir::Expr::Path(vec![enum_name, name.to_string()])
    } else {
        // S76: clone only when a variable is used more than once.
        // Single-use variables can be moved; multi-use need .clone().
        let escaped = escape_rust_keyword(name);
        if escaped.starts_with("__") {
            code_ir::Expr::Var(escaped)
        } else {
            let count = ctx.use_counts.get(name).copied().unwrap_or(2);
            if count <= 1 {
                code_ir::Expr::Var(escaped)
            } else {
                code_ir::Expr::MethodCall {
                    receiver: Box::new(code_ir::Expr::Var(escaped)),
                    method: "clone".to_string(),
                    args: vec![],
                }
            }
        }
    }
}

/// Escape Rust reserved keywords for use as identifiers.
fn escape_rust_keyword(name: &str) -> String {
    match name {
        "mod" | "type" | "fn" | "let" | "mut" | "ref" | "self" | "super" | "crate" | "pub"
        | "use" | "impl" | "trait" | "struct" | "enum" | "match" | "if" | "else" | "for"
        | "while" | "loop" | "break" | "continue" | "return" | "where" | "as" | "in" | "move"
        | "async" | "await" | "dyn" | "static" | "const" | "extern" | "unsafe" | "abstract"
        | "become" | "box" | "do" | "final" | "macro" | "override" | "priv" | "try" | "typeof"
        | "unsized" | "virtual" | "yield" => format!("r#{name}"),
        _ => name.to_string(),
    }
}

fn to_screaming_snake(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_ascii_uppercase());
    }
    result
}

fn with_call_arg_path<T>(ctx: &CompileContext, index: usize, f: impl FnOnce() -> T) -> T {
    ctx.with_child_expr_path(ExprPathStep::CallArg(index), f)
}

fn compile_call_arg(
    args: &[(Option<String>, ast::Expr)],
    index: usize,
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::Expr {
    with_call_arg_path(ctx, index, || compile_expr(&args[index].1, ctx, counter))
}

fn compile_call_arg_typed(
    args: &[(Option<String>, ast::Expr)],
    index: usize,
    expected_type: Option<&str>,
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::Expr {
    with_call_arg_path(ctx, index, || {
        compile_expr_typed(&args[index].1, ctx, expected_type, counter)
    })
}

fn infer_call_arg_type(
    args: &[(Option<String>, ast::Expr)],
    index: usize,
    ctx: &CompileContext,
) -> Option<IrType> {
    with_call_arg_path(ctx, index, || {
        ctx.current_expr_ir_type()
            .or_else(|| infer_known_expr_ir_type(&args[index].1, ctx))
    })
}

// ---------------------------------------------------------------------------
// Function calls
// ---------------------------------------------------------------------------

/// Intercept collection intrinsic calls that were formerly pipe methods.
///
/// These are compiled to For loops for cross-target compatibility — the
/// code_ir `For` / `Block` / `If` constructs are understood by all backend
/// renderers (Rust, Go, C), unlike language-specific iterator chains.
///
/// Returns `None` for non-intrinsic calls.
fn compile_intrinsic_call(
    name: &str,
    args: &[(Option<String>, ast::Expr)],
    ctx: &CompileContext,
    counter: &mut usize,
) -> Option<code_ir::Expr> {
    // Zero-arg intrinsics (before the args.is_empty() guard).
    // empty_map() → Rc::new(HashMap::new())
    if name == "empty_map" && args.is_empty() {
        return Some(rc_wrap(code_ir::Expr::RawCode(
            "std::collections::HashMap::new()".to_string(),
        )));
    }

    // First arg is always the collection/receiver.
    // Eagerly evaluated since counter is &mut and can't be captured in a closure.
    if args.is_empty() {
        return None;
    }
    let fold_accum_name = ctx.fold_accum_name.as_deref();
    let collection = clone_if_needed(compile_expr(&args[0].1, ctx, counter), fold_accum_name);

    match name {
        "map" if args.len() == 2 => Some(with_call_arg_path(ctx, 1, || {
            compile_map_intrinsic(
                &collection.clone(),
                &args[0].1,
                args.get(1).map(|(_, e)| e),
                ctx,
                counter,
            )
        })),
        "filter" if args.len() == 2 => Some(with_call_arg_path(ctx, 1, || {
            compile_filter_intrinsic(
                &collection.clone(),
                &args[0].1,
                args.get(1).map(|(_, e)| e),
                ctx,
                counter,
            )
        })),
        "fold" if args.len() >= 2 => {
            let call_path = ctx.current_expr_path();
            let init_index = args
                .iter()
                .position(|(n, _)| n.as_deref() == Some("init"))
                .or_else(|| (args.len() > 1).then_some(1));
            let init = init_index.map(|index| &args[index].1);
            let func_index = args
                .iter()
                .position(|(n, _)| n.as_deref() == Some("f"))
                .or_else(|| (args.len() > 2).then_some(2));
            let func = func_index.map(|index| &args[index].1);
            Some(compile_fold_intrinsic(
                &collection.clone(),
                &args[0].1,
                FoldExprContext {
                    init: init_index
                        .zip(init)
                        .map(|(index, expr)| (expr, call_path.child(ExprPathStep::CallArg(index)))),
                    func: func_index
                        .zip(func)
                        .map(|(index, expr)| (expr, call_path.child(ExprPathStep::CallArg(index)))),
                },
                ctx,
                counter,
            ))
        }
        "any" if args.len() == 2 => {
            let pred = args
                .iter()
                .find(|(n, _)| n.as_deref() == Some("predicate"))
                .or_else(|| args.get(1))
                .map(|(_, e)| e);
            Some(with_call_arg_path(ctx, 1, || {
                compile_any_intrinsic(&collection.clone(), &args[0].1, pred, ctx, counter)
            }))
        }
        "all" if args.len() == 2 => {
            let pred = args
                .iter()
                .find(|(n, _)| n.as_deref() == Some("predicate"))
                .or_else(|| args.get(1))
                .map(|(_, e)| e);
            Some(with_call_arg_path(ctx, 1, || {
                compile_all_intrinsic(&collection.clone(), &args[0].1, pred, ctx, counter)
            }))
        }
        "contains" if args.len() == 2 => {
            let target = compile_call_arg(args, 1, ctx, counter);
            let result = fresh(counter, "contains");
            let elem = fresh(counter, "elem");
            Some(code_ir::Expr::Block(vec![
                code_ir::Stmt::let_mut(&result, code_ir::Expr::BoolLit(false)),
                code_ir::Stmt::For {
                    binding: elem.clone(),
                    iter: make_owned_iter(collection.clone()),
                    body: vec![code_ir::Stmt::Expr(code_ir::Expr::If {
                        cond: Box::new(code_ir::Expr::BinOp {
                            left: Box::new(code_ir::Expr::Var(elem)),
                            op: "==".to_string(),
                            right: Box::new(target),
                        }),
                        then_body: vec![
                            code_ir::Stmt::Assign {
                                dest: code_ir::Expr::Var(result.clone()),
                                value: code_ir::Expr::BoolLit(true),
                            },
                            code_ir::Stmt::Expr(code_ir::Expr::RawCode("break".to_string())),
                        ],
                        else_body: None,
                    })],
                },
                code_ir::Stmt::TailExpr(code_ir::Expr::Var(result)),
            ]))
        }
        "sum" if args.len() == 1 => {
            let result = fresh(counter, "sum");
            let elem = fresh(counter, "elem");
            Some(code_ir::Expr::Block(vec![
                code_ir::Stmt::let_mut(&result, code_ir::Expr::IntLit(0)),
                code_ir::Stmt::For {
                    binding: elem.clone(),
                    iter: make_owned_iter(collection.clone()),
                    body: vec![code_ir::Stmt::Assign {
                        dest: code_ir::Expr::Var(result.clone()),
                        value: code_ir::Expr::BinOp {
                            left: Box::new(code_ir::Expr::Var(result.clone())),
                            op: "+".to_string(),
                            right: Box::new(code_ir::Expr::Var(elem)),
                        },
                    }],
                },
                code_ir::Stmt::TailExpr(code_ir::Expr::Var(result)),
            ]))
        }
        "join" if args.len() == 2 => {
            let sep = compile_call_arg(args, 1, ctx, counter);
            let result = fresh(counter, "joined");
            let elem = fresh(counter, "elem");
            let first = fresh(counter, "first");
            Some(code_ir::Expr::Block(vec![
                code_ir::Stmt::let_mut(
                    &result,
                    code_ir::Expr::RawCode("String::new()".to_string()),
                ),
                code_ir::Stmt::let_mut(&first, code_ir::Expr::BoolLit(true)),
                code_ir::Stmt::For {
                    binding: elem.clone(),
                    iter: make_owned_iter(collection.clone()),
                    body: vec![
                        code_ir::Stmt::Expr(code_ir::Expr::If {
                            cond: Box::new(code_ir::Expr::UnaryOp {
                                op: "!".to_string(),
                                expr: Box::new(code_ir::Expr::Var(first.clone())),
                            }),
                            then_body: vec![code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                                receiver: Box::new(code_ir::Expr::Var(result.clone())),
                                method: "push_str".to_string(),
                                args: vec![code_ir::Expr::Ref(Box::new(sep.clone()))],
                            })],
                            else_body: None,
                        }),
                        code_ir::Stmt::Assign {
                            dest: code_ir::Expr::Var(first.clone()),
                            value: code_ir::Expr::BoolLit(false),
                        },
                        code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                            receiver: Box::new(code_ir::Expr::Var(result.clone())),
                            method: "push_str".to_string(),
                            args: vec![code_ir::Expr::Ref(Box::new(code_ir::Expr::Var(elem)))],
                        }),
                    ],
                },
                code_ir::Stmt::TailExpr(code_ir::Expr::Var(result)),
            ]))
        }
        "last" if args.len() == 1 => Some(code_ir::Expr::MethodCall {
            receiver: Box::new(code_ir::Expr::MethodCall {
                receiver: Box::new(collection.clone()),
                method: "last".to_string(),
                args: vec![],
            }),
            method: "cloned".to_string(),
            args: vec![],
        }),
        "split" if args.len() == 2 => {
            let delim = compile_call_arg(args, 1, ctx, counter);
            let result = fresh(counter, "split_parts");
            let elem = fresh(counter, "part");
            Some(code_ir::Expr::Block(vec![
                code_ir::Stmt::let_mut(
                    &result,
                    code_ir::Expr::MacroCall {
                        name: "vec".to_string(),
                        args: vec![],
                    },
                ),
                code_ir::Stmt::For {
                    binding: elem.clone(),
                    iter: code_ir::Expr::MethodCall {
                        receiver: Box::new(collection.clone()),
                        method: "split".to_string(),
                        args: vec![code_ir::Expr::MethodCall {
                            receiver: Box::new(delim),
                            method: "as_str".to_string(),
                            args: vec![],
                        }],
                    },
                    body: vec![code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                        receiver: Box::new(code_ir::Expr::Var(result.clone())),
                        method: "push".to_string(),
                        args: vec![code_ir::Expr::MethodCall {
                            receiver: Box::new(code_ir::Expr::Var(elem)),
                            method: "to_string".to_string(),
                            args: vec![],
                        }],
                    })],
                },
                code_ir::Stmt::TailExpr(code_ir::Expr::Var(result)),
            ]))
        }
        "chars" if args.len() == 1 => {
            let result = fresh(counter, "chars");
            let ch = fresh(counter, "ch");
            Some(code_ir::Expr::Block(vec![
                code_ir::Stmt::let_mut(
                    &result,
                    code_ir::Expr::MacroCall {
                        name: "vec".to_string(),
                        args: vec![],
                    },
                ),
                code_ir::Stmt::For {
                    binding: ch.clone(),
                    iter: code_ir::Expr::MethodCall {
                        receiver: Box::new(collection.clone()),
                        method: "chars".to_string(),
                        args: vec![],
                    },
                    body: vec![code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                        receiver: Box::new(code_ir::Expr::Var(result.clone())),
                        method: "push".to_string(),
                        args: vec![code_ir::Expr::MethodCall {
                            receiver: Box::new(code_ir::Expr::Var(ch)),
                            method: "to_string".to_string(),
                            args: vec![],
                        }],
                    })],
                },
                code_ir::Stmt::TailExpr(rc_wrap(code_ir::Expr::Var(result))),
            ]))
        }
        // v2 compiler intrinsics: immutable record update
        "with" if args.len() == 2 => {
            // with(state, { pos: state.pos + 1 }) → State { pos: state.pos + 1, ..state.clone() }
            let base = compile_call_arg(args, 0, ctx, counter);
            let base_struct_name =
                infer_call_arg_type(args, 0, ctx).and_then(|ty| named_type_from_ir(&ty));
            // Clone the base value since struct update syntax moves the base
            let cloned_base = code_ir::Expr::MethodCall {
                receiver: Box::new(base),
                method: "clone".to_string(),
                args: vec![],
            };
            // Second arg should be a record literal with update fields
            match &args[1].1 {
                ast::Expr::Record(name, fields) => {
                    let struct_name = if let Some(struct_name) = name.as_deref() {
                        struct_name.to_string()
                    } else {
                        let field_names: HashSet<&str> =
                            fields.iter().map(|(n, _)| n.as_str()).collect();
                        let Some(struct_name) = with_call_arg_path(ctx, 1, || {
                            resolve_record_struct_name(base_struct_name.as_deref(), ctx)
                        }) else {
                            return Some(unresolved_anonymous_record_error(&field_names));
                        };
                        struct_name
                    };
                    let arg_path = ctx.current_expr_path().child(ExprPathStep::CallArg(1));
                    Some(compile_resolved_record_expr(
                        &struct_name,
                        fields,
                        Some(cloned_base),
                        |index| record_field_path(&arg_path, index),
                        ctx,
                        counter,
                    ))
                }
                _ => {
                    // Fallback: emit as runtime call
                    Some(code_ir::Expr::Call {
                        func: Box::new(code_ir::Expr::Path(vec![
                            "v2_rt".to_string(),
                            "with".to_string(),
                        ])),
                        args: vec![cloned_base, compile_call_arg(args, 1, ctx, counter)],
                        obligation: None,
                    })
                }
            }
        }
        // v2 compiler intrinsics: string builtins
        "char_at" if args.len() == 2 => {
            let s = resolve_named_or_positional(args, "s", 0, ctx, counter);
            let pos = resolve_named_or_positional(args, "pos", 1, ctx, counter);
            Some(code_ir::Expr::Call {
                func: Box::new(code_ir::Expr::Path(vec![
                    "v2_rt".to_string(),
                    "char_at".to_string(),
                ])),
                args: vec![s, pos],
                obligation: None,
            })
        }
        "string_length" if args.len() == 1 => {
            let s = resolve_named_or_positional(args, "s", 0, ctx, counter);
            Some(code_ir::Expr::Call {
                func: Box::new(code_ir::Expr::Path(vec![
                    "v2_rt".to_string(),
                    "string_length".to_string(),
                ])),
                args: vec![s],
                obligation: None,
            })
        }
        "substring" if args.len() == 3 => {
            let s = resolve_named_or_positional(args, "s", 0, ctx, counter);
            let start = resolve_named_or_positional(args, "start", 1, ctx, counter);
            let end = resolve_named_or_positional(args, "end", 2, ctx, counter);
            Some(code_ir::Expr::Call {
                func: Box::new(code_ir::Expr::Path(vec![
                    "v2_rt".to_string(),
                    "substring".to_string(),
                ])),
                args: vec![s, start, end],
                obligation: None,
            })
        }
        "string_contains" if args.len() == 2 => {
            let s = resolve_named_or_positional(args, "s", 0, ctx, counter);
            let substring = resolve_named_or_positional(args, "substring", 1, ctx, counter);
            Some(code_ir::Expr::Call {
                func: Box::new(code_ir::Expr::Path(vec![
                    "v2_rt".to_string(),
                    "string_contains".to_string(),
                ])),
                args: vec![s, substring],
                obligation: None,
            })
        }
        "lookup" if args.len() == 2 => {
            let table = resolve_named_or_positional(args, "table", 0, ctx, counter);
            let key = resolve_named_or_positional(args, "key", 1, ctx, counter);
            Some(code_ir::Expr::Call {
                func: Box::new(code_ir::Expr::Path(vec![
                    "v2_rt".to_string(),
                    "lookup".to_string(),
                ])),
                args: vec![table, key],
                obligation: None,
            })
        }
        // concat(a) → identity (single-arg concat is a no-op, avoids
        // clashing with Rust's built-in `concat!` macro).
        "concat" if args.len() == 1 => Some(collection),
        // concat(a, b) → v2_rt::concat(a, b)
        // Works for both String and Vec<T> via the Concat trait.
        "concat" if args.len() >= 2 => {
            // Fuse concat(acc, [item]) → in-place append for O(1) amortized accumulation.
            // The single-element list literal as second arg is the tail-recursive
            // accumulation pattern. Compiling as append with strip_outer_clone lets
            // Rc::try_unwrap succeed (refcount 1) → in-place Vec::push.
            if args.len() == 2 {
                if let ast::Expr::List(elements) = &args[1].1 {
                    if elements.len() == 1 {
                        let list = strip_outer_clone(collection.clone());
                        let item = with_call_arg_path(ctx, 1, || {
                            ctx.with_child_expr_path(ExprPathStep::ListItem(0), || {
                                compile_expr(&elements[0], ctx, counter)
                            })
                        });
                        let v = fresh(counter, "appended");
                        let mut stmts = rc_unwrap_stmts(&v, list, counter);
                        stmts.push(code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                            receiver: Box::new(code_ir::Expr::Var(v.clone())),
                            method: "push".to_string(),
                            args: vec![item],
                        }));
                        stmts.push(code_ir::Stmt::TailExpr(rc_wrap(code_ir::Expr::Var(v))));
                        return Some(code_ir::Expr::Block(stmts));
                    }
                }
            }
            // Binary: concat(a, b) → v2_rt::concat(a, b)
            // Variadic: concat(a, b, c) → v2_rt::concat(v2_rt::concat(a, b), c)
            // Strip fold accumulator's .clone() so try_unwrap succeeds → in-place extend.
            let first = compile_expr(&args[0].1, ctx, counter);
            let is_fold_accum = matches!(&args[0].1, ast::Expr::Ident(name) if ctx.fold_accum_name.as_deref() == Some(name.as_str()));
            let mut result = if is_fold_accum {
                strip_outer_clone(first)
            } else {
                first
            };
            for (index, _) in args[1..].iter().enumerate() {
                result = code_ir::Expr::Call {
                    func: Box::new(code_ir::Expr::Path(vec![
                        "v2_rt".to_string(),
                        "concat".to_string(),
                    ])),
                    args: vec![result, compile_call_arg(args, index + 1, ctx, counter)],
                    obligation: None,
                };
            }
            Some(result)
        }
        // count(filter(list, pred)) → counting loop (avoids materializing filtered list).
        // count(list) → { let __len = list.len(); __len as i64 }
        "count" if args.len() == 1 => {
            // Fuse count(filter(list, pred)) → for e in list { if pred(e) { count += 1; } }
            if let ast::Expr::Call(filter_name, filter_args) = &args[0].1 {
                if filter_name == "filter" && filter_args.len() == 2 {
                    let list = compile_expr(&filter_args[0].1, ctx, counter);
                    let result = fresh(counter, "count");
                    let elem = fresh(counter, "elem");
                    let cond = match &filter_args[1].1 {
                        ast::Expr::Lambda(params, body) => {
                            let compiled = compile_collection_lambda_body(
                                &filter_args[0].1,
                                params,
                                body,
                                ctx,
                                counter,
                            );
                            params
                                .first()
                                .map(|p| substitute_var(&compiled, p, &code_ir::Expr::Var(elem.clone())))
                                .unwrap_or(compiled)
                        }
                        other => code_ir::Expr::Call {
                            func: Box::new(compile_expr(other, ctx, counter)),
                            args: vec![code_ir::Expr::Var(elem.clone())],
                            obligation: None,
                        },
                    };
                    return Some(code_ir::Expr::Block(vec![
                        code_ir::Stmt::Let {
                            name: result.clone(),
                            mutable: true,
                            expr: code_ir::Expr::RawCode("0i64".to_string()),
                            ir_type: Some(IrType::Int),
                        },
                        code_ir::Stmt::For {
                            binding: elem,
                            iter: make_owned_iter(list),
                            body: vec![code_ir::Stmt::Expr(code_ir::Expr::If {
                                cond: Box::new(cond),
                                then_body: vec![code_ir::Stmt::Assign {
                                    dest: code_ir::Expr::Var(result.clone()),
                                    value: code_ir::Expr::BinOp {
                                        left: Box::new(code_ir::Expr::Var(result.clone())),
                                        op: "+".to_string(),
                                        right: Box::new(code_ir::Expr::RawCode("1i64".to_string())),
                                    },
                                }],
                                else_body: None,
                            })],
                        },
                        code_ir::Stmt::TailExpr(code_ir::Expr::Var(result)),
                    ]));
                }
            }
            let tmp = fresh(counter, "len");
            Some(code_ir::Expr::Block(vec![
                code_ir::Stmt::Let {
                    name: tmp.clone(),
                    expr: code_ir::Expr::MethodCall {
                        receiver: Box::new(collection.clone()),
                        method: "len".to_string(),
                        args: vec![],
                    },
                    mutable: false,
                    ir_type: None,
                },
                code_ir::Stmt::TailExpr(code_ir::Expr::RawCode(format!("{tmp} as i64"))),
            ]))
        }
        // first(list) → list.first().cloned()
        // Fuses first(skip(list, n)) → list.get(n as usize).cloned() — O(1) vs O(n).
        // Fuses first(filter(list, pred)) → find loop — avoids materializing filtered list.
        "first" if args.len() == 1 => {
            if let ast::Expr::Call(inner_name, inner_args) = &args[0].1 {
                // first(skip(list, n)) → list.get(n as usize).cloned()
                if inner_name == "skip" && inner_args.len() == 2 {
                    let list = clone_if_needed(compile_expr(&inner_args[0].1, ctx, counter), fold_accum_name);
                    let idx = compile_expr(&inner_args[1].1, ctx, counter);
                    return Some(code_ir::Expr::MethodCall {
                        receiver: Box::new(code_ir::Expr::MethodCall {
                            receiver: Box::new(list),
                            method: "get".to_string(),
                            args: vec![code_ir::Expr::RawCode(format!(
                                "({}) as usize",
                                render_expr_inline(&idx)
                            ))],
                        }),
                        method: "cloned".to_string(),
                        args: vec![],
                    });
                }
                // first(filter(list, pred)) → find loop
                if inner_name == "filter" && inner_args.len() == 2 {
                    let list = compile_expr(&inner_args[0].1, ctx, counter);
                    let result = fresh(counter, "found");
                    let elem = fresh(counter, "elem");
                    let cond = match &inner_args[1].1 {
                        ast::Expr::Lambda(params, body) => {
                            let compiled = compile_collection_lambda_body(
                                &inner_args[0].1,
                                params,
                                body,
                                ctx,
                                counter,
                            );
                            params
                                .first()
                                .map(|p| {
                                    substitute_var(&compiled, p, &code_ir::Expr::Var(elem.clone()))
                                })
                                .unwrap_or(compiled)
                        }
                        other => code_ir::Expr::Call {
                            func: Box::new(compile_expr(other, ctx, counter)),
                            args: vec![code_ir::Expr::Var(elem.clone())],
                            obligation: None,
                        },
                    };
                    return Some(code_ir::Expr::Block(vec![
                        code_ir::Stmt::Let {
                            name: result.clone(),
                            mutable: true,
                            expr: code_ir::Expr::Var("None".to_string()),
                            ir_type: None,
                        },
                        code_ir::Stmt::For {
                            binding: elem.clone(),
                            iter: make_owned_iter(list),
                            body: vec![code_ir::Stmt::Expr(code_ir::Expr::If {
                                cond: Box::new(cond),
                                then_body: vec![
                                    code_ir::Stmt::Assign {
                                        dest: code_ir::Expr::Var(result.clone()),
                                        value: code_ir::Expr::Call {
                                            func: Box::new(code_ir::Expr::Var("Some".to_string())),
                                            args: vec![code_ir::Expr::Var(elem)],
                                            obligation: None,
                                        },
                                    },
                                    code_ir::Stmt::Expr(code_ir::Expr::RawCode(
                                        "break".to_string(),
                                    )),
                                ],
                                else_body: None,
                            })],
                        },
                        code_ir::Stmt::TailExpr(code_ir::Expr::Var(result)),
                    ]));
                }
            }
            Some(code_ir::Expr::MethodCall {
                receiver: Box::new(code_ir::Expr::MethodCall {
                    receiver: Box::new(collection.clone()),
                    method: "first".to_string(),
                    args: vec![],
                }),
                method: "cloned".to_string(),
                args: vec![],
            })
        }
        // append(list, item) → { let __rc = list; let mut v = Rc::try_unwrap(__rc)...; v.push(item); Rc::new(v) }
        "append" if args.len() == 2 => {
            // Strip outer .clone() so Rc::try_unwrap succeeds for in-place push.
            let list = strip_outer_clone(collection.clone());
            let item = compile_call_arg(args, 1, ctx, counter);
            let v = fresh(counter, "appended");
            let mut stmts = rc_unwrap_stmts(&v, list, counter);
            stmts.push(code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                receiver: Box::new(code_ir::Expr::Var(v.clone())),
                method: "push".to_string(),
                args: vec![item],
            }));
            stmts.push(code_ir::Stmt::TailExpr(rc_wrap(code_ir::Expr::Var(v))));
            Some(code_ir::Expr::Block(stmts))
        }
        // flat_map(list, f) → { let mut r = vec![]; for e in list { r.extend(f(e)); } Rc::new(r) }
        "flat_map" if args.len() == 2 => {
            let result = fresh(counter, "flat_mapped");
            let elem = fresh(counter, "elem");
            let mapped = match &args[1].1 {
                ast::Expr::Lambda(params, body) => {
                    let compiled = with_call_arg_path(ctx, 1, || {
                        compile_collection_lambda_body(&args[0].1, params, body, ctx, counter)
                    });
                    params
                        .first()
                        .map(|p| substitute_var(&compiled, p, &code_ir::Expr::Var(elem.clone())))
                        .unwrap_or(compiled)
                }
                _other => code_ir::Expr::Call {
                    func: Box::new(compile_call_arg(args, 1, ctx, counter)),
                    args: vec![code_ir::Expr::Var(elem.clone())],
                    obligation: None,
                },
            };
            Some(code_ir::Expr::Block(vec![
                code_ir::Stmt::let_mut(
                    &result,
                    code_ir::Expr::MacroCall {
                        name: "vec".to_string(),
                        args: vec![],
                    },
                ),
                code_ir::Stmt::For {
                    binding: elem,
                    iter: make_owned_iter(collection.clone()),
                    body: vec![code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                        receiver: Box::new(code_ir::Expr::Var(result.clone())),
                        method: "extend".to_string(),
                        args: vec![code_ir::Expr::MethodCall {
                            receiver: Box::new(code_ir::Expr::MethodCall {
                                receiver: Box::new(mapped),
                                method: "iter".to_string(),
                                args: vec![],
                            }),
                            method: "cloned".to_string(),
                            args: vec![],
                        }],
                    })],
                },
                code_ir::Stmt::TailExpr(rc_wrap(code_ir::Expr::Var(result))),
            ]))
        }
        // enumerate(list) → { let mut r = vec![]; for (i, e) in list.iter().enumerate() { r.push((i as i64, e.clone())); } Rc::new(r) }
        "enumerate" if args.len() == 1 => {
            let result = fresh(counter, "enumerated");
            let idx = fresh(counter, "idx");
            let elem = fresh(counter, "elem");
            Some(code_ir::Expr::Block(vec![
                code_ir::Stmt::let_mut(
                    &result,
                    code_ir::Expr::MacroCall {
                        name: "vec".to_string(),
                        args: vec![],
                    },
                ),
                code_ir::Stmt::For {
                    binding: format!("({idx}, {elem})"),
                    iter: code_ir::Expr::MethodCall {
                        receiver: Box::new(code_ir::Expr::MethodCall {
                            receiver: Box::new(collection.clone()),
                            method: "iter".to_string(),
                            args: vec![],
                        }),
                        method: "enumerate".to_string(),
                        args: vec![],
                    },
                    body: vec![code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                        receiver: Box::new(code_ir::Expr::Var(result.clone())),
                        method: "push".to_string(),
                        args: vec![code_ir::Expr::RawCode(format!(
                            "({idx} as i64, {elem}.clone())"
                        ))],
                    })],
                },
                code_ir::Stmt::TailExpr(rc_wrap(code_ir::Expr::Var(result))),
            ]))
        }
        // sort_by(list, key_fn) → { let __rc = list; let mut v = Rc::try_unwrap(__rc)...; v.sort_by_key(key_fn); Rc::new(v) }
        "sort_by" if args.len() == 2 => {
            let list = collection.clone();
            let key_fn = compile_call_arg(args, 1, ctx, counter);
            let v = fresh(counter, "sorted");
            let mut stmts = rc_unwrap_stmts(&v, list, counter);
            stmts.push(code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                receiver: Box::new(code_ir::Expr::Var(v.clone())),
                method: "sort_by_key".to_string(),
                args: vec![key_fn],
            }));
            stmts.push(code_ir::Stmt::TailExpr(rc_wrap(code_ir::Expr::Var(v))));
            Some(code_ir::Expr::Block(stmts))
        }
        // drop_last(list) → Rc::new(list[..list.len()-1].to_vec())
        "drop_last" if args.len() == 1 => {
            let list = collection.clone();
            Some(code_ir::Expr::RawCode(format!(
                "{{ let __v = {}; Rc::new(__v[..__v.len().saturating_sub(1)].to_vec()) }}",
                render_expr_inline(&list)
            )))
        }
        // replace_last(list, value) → { let __rc = list; let mut v = Rc::try_unwrap(__rc)...; if let Some(last) = v.last_mut() { *last = value; } Rc::new(v) }
        "replace_last" if args.len() == 2 => {
            let list = collection.clone();
            let value = compile_call_arg(args, 1, ctx, counter);
            let v = fresh(counter, "replaced");
            let mut stmts = rc_unwrap_stmts(&v, list, counter);
            stmts.push(code_ir::Stmt::Expr(code_ir::Expr::RawCode(format!(
                "if let Some(__last) = {v}.last_mut() {{ *__last = {}; }}",
                render_expr_inline(&value)
            ))));
            stmts.push(code_ir::Stmt::TailExpr(rc_wrap(code_ir::Expr::Var(v))));
            Some(code_ir::Expr::Block(stmts))
        }
        // starts_with(s, prefix) → s.starts_with(prefix.as_str())
        "starts_with" if args.len() == 2 => {
            let s = compile_call_arg(args, 0, ctx, counter);
            let prefix = compile_call_arg(args, 1, ctx, counter);
            Some(code_ir::Expr::MethodCall {
                receiver: Box::new(s),
                method: "starts_with".to_string(),
                args: vec![code_ir::Expr::MethodCall {
                    receiver: Box::new(prefix),
                    method: "as_str".to_string(),
                    args: vec![],
                }],
            })
        }
        // ends_with(s, suffix) → s.ends_with(suffix.as_str())
        "ends_with" if args.len() == 2 => {
            let s = compile_call_arg(args, 0, ctx, counter);
            let suffix = compile_call_arg(args, 1, ctx, counter);
            Some(code_ir::Expr::MethodCall {
                receiver: Box::new(s),
                method: "ends_with".to_string(),
                args: vec![code_ir::Expr::MethodCall {
                    receiver: Box::new(suffix),
                    method: "as_str".to_string(),
                    args: vec![],
                }],
            })
        }
        // parse_int(s) → s.parse::<i64>().ok()
        "parse_int" if args.len() == 1 => Some(code_ir::Expr::RawCode(format!(
            "{}.parse::<i64>().ok()",
            render_expr_inline(&collection.clone())
        ))),
        // reverse(list) → { let __rc = list; let mut v = Rc::try_unwrap(__rc)...; v.reverse(); Rc::new(v) }
        "reverse" if args.len() == 1 => {
            let v = fresh(counter, "reversed");
            let mut stmts = rc_unwrap_stmts(&v, collection.clone(), counter);
            stmts.push(code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                receiver: Box::new(code_ir::Expr::Var(v.clone())),
                method: "reverse".to_string(),
                args: vec![],
            }));
            stmts.push(code_ir::Stmt::TailExpr(rc_wrap(code_ir::Expr::Var(v))));
            Some(code_ir::Expr::Block(stmts))
        }
        // list_push(list, item) → { let mut v = Rc::try_unwrap(list)...; v.push(item); Rc::new(v) }
        // O(1) amortized append — avoids O(n) concat([item], list) + reverse.
        "list_push" if args.len() == 2 => {
            let item = compile_call_arg(args, 1, ctx, counter);
            let list = strip_outer_clone(collection.clone());
            let v = fresh(counter, "appended");
            let mut stmts = rc_unwrap_stmts(&v, list, counter);
            stmts.push(code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                receiver: Box::new(code_ir::Expr::Var(v.clone())),
                method: "push".to_string(),
                args: vec![item],
            }));
            stmts.push(code_ir::Stmt::TailExpr(rc_wrap(code_ir::Expr::Var(v))));
            Some(code_ir::Expr::Block(stmts))
        }
        // is_empty(list) → list.is_empty()
        "is_empty" if args.len() == 1 => Some(code_ir::Expr::MethodCall {
            receiver: Box::new(collection.clone()),
            method: "is_empty".to_string(),
            args: vec![],
        }),
        // skip(list, n) → Rc::new(list[n as usize..].to_vec())
        "skip" if args.len() == 2 => {
            let n = compile_call_arg(args, 1, ctx, counter);
            Some(code_ir::Expr::RawCode(format!(
                "{{ let __s = {}; Rc::new(__s[({}) as usize..].to_vec()) }}",
                render_expr_inline(&collection.clone()),
                render_expr_inline(&n)
            )))
        }
        // take(list, n) → Rc::new(list[..n as usize].to_vec())
        "take" if args.len() == 2 => {
            let n = compile_call_arg(args, 1, ctx, counter);
            Some(code_ir::Expr::RawCode(format!(
                "{{ let __t = {}; Rc::new(__t[..({}) as usize].to_vec()) }}",
                render_expr_inline(&collection.clone()),
                render_expr_inline(&n)
            )))
        }
        // find(list, pred) → list.iter().find(pred).cloned()
        "find" if args.len() == 2 => {
            let pred = compile_call_arg(args, 1, ctx, counter);
            Some(code_ir::Expr::MethodCall {
                receiver: Box::new(code_ir::Expr::MethodCall {
                    receiver: Box::new(code_ir::Expr::MethodCall {
                        receiver: Box::new(collection.clone()),
                        method: "iter".to_string(),
                        args: vec![],
                    }),
                    method: "find".to_string(),
                    args: vec![pred],
                }),
                method: "cloned".to_string(),
                args: vec![],
            })
        }
        // get(list, idx) → list.get(idx as usize).cloned()
        "get" if args.len() == 2 => {
            let idx = compile_call_arg(args, 1, ctx, counter);
            Some(code_ir::Expr::MethodCall {
                receiver: Box::new(code_ir::Expr::MethodCall {
                    receiver: Box::new(collection.clone()),
                    method: "get".to_string(),
                    args: vec![code_ir::Expr::RawCode(format!(
                        "({}) as usize",
                        render_expr_inline(&idx)
                    ))],
                }),
                method: "cloned".to_string(),
                args: vec![],
            })
        }
        // index_by(list, key_fn) → HashMap<String, V> from list using key_fn to extract keys.
        // Duplicate keys: last writer wins (later elements overwrite earlier ones).
        "index_by" if args.len() == 2 => {
            let result = fresh(counter, "indexed");
            let elem = fresh(counter, "elem");
            let key_expr = match &args[1].1 {
                ast::Expr::Lambda(params, body) => {
                    let compiled =
                        compile_collection_lambda_body(&args[0].1, params, body, ctx, counter);
                    params
                        .first()
                        .map(|p| substitute_var(&compiled, p, &code_ir::Expr::Var(elem.clone())))
                        .unwrap_or(compiled)
                }
                other => code_ir::Expr::Call {
                    func: Box::new(compile_expr(other, ctx, counter)),
                    args: vec![code_ir::Expr::Var(elem.clone())],
                    obligation: None,
                },
            };
            Some(code_ir::Expr::Block(vec![
                code_ir::Stmt::Let {
                    name: result.clone(),
                    mutable: true,
                    expr: code_ir::Expr::RawCode("std::collections::HashMap::new()".to_string()),
                    ir_type: None,
                },
                code_ir::Stmt::For {
                    binding: elem.clone(),
                    iter: make_owned_iter(collection.clone()),
                    body: vec![code_ir::Stmt::Expr(code_ir::Expr::RawCode(format!(
                        "{result}.insert({key}, {elem})",
                        key = render_expr_inline(&key_expr),
                    )))],
                },
                code_ir::Stmt::TailExpr(rc_wrap(code_ir::Expr::Var(result))),
            ]))
        }
        // map_get(map, key) → map.get(&key).cloned() — O(1) HashMap lookup.
        "map_get" if args.len() == 2 => {
            let key = compile_expr(&args[1].1, ctx, counter);
            Some(code_ir::Expr::MethodCall {
                receiver: Box::new(code_ir::Expr::MethodCall {
                    receiver: Box::new(collection.clone()),
                    method: "get".to_string(),
                    args: vec![code_ir::Expr::Ref(Box::new(key))],
                }),
                method: "cloned".to_string(),
                args: vec![],
            })
        }
        // map_contains_key(map, key) → map.contains_key(&key) — O(1) membership check.
        "map_contains_key" if args.len() == 2 => {
            let key = compile_expr(&args[1].1, ctx, counter);
            Some(code_ir::Expr::MethodCall {
                receiver: Box::new(collection.clone()),
                method: "contains_key".to_string(),
                args: vec![code_ir::Expr::Ref(Box::new(key))],
            })
        }
        // map_values(map) → Rc-unwrap HashMap, into_values, collect into Rc<Vec<_>>.
        "map_values" if args.len() == 1 => {
            let rc_var = fresh(counter, "rc");
            let map_var = fresh(counter, "map_unwrapped");
            let values = fresh(counter, "values");
            Some(code_ir::Expr::Block(vec![
                code_ir::Stmt::let_bind(&rc_var, collection.clone()),
                code_ir::Stmt::let_bind(
                    &map_var,
                    code_ir::Expr::RawCode(format!(
                        "Rc::try_unwrap({rc_var}).unwrap_or_else(|rc| (*rc).clone())"
                    )),
                ),
                code_ir::Stmt::Let {
                    name: values.clone(),
                    mutable: false,
                    expr: code_ir::Expr::RawCode(format!(
                        "{map_var}.into_values().collect::<Vec<_>>()"
                    )),
                    ir_type: None,
                },
                code_ir::Stmt::TailExpr(rc_wrap(code_ir::Expr::Var(values))),
            ]))
        }
        // map_insert(map, key, value) → Rc::try_unwrap + .insert(key, value) + Rc::new.
        // O(1) amortized when refcount=1 (true in fold accumulators).
        "map_insert" if args.len() == 3 => {
            let map = strip_outer_clone(collection.clone());
            let key = compile_expr(&args[1].1, ctx, counter);
            let value = compile_expr(&args[2].1, ctx, counter);
            let v = fresh(counter, "map_ins");
            let mut stmts = rc_unwrap_stmts(&v, map, counter);
            stmts.push(code_ir::Stmt::Expr(code_ir::Expr::RawCode(format!(
                "{v}.insert({}, {})",
                render_expr_inline(&key),
                render_expr_inline(&value)
            ))));
            stmts.push(code_ir::Stmt::TailExpr(rc_wrap(code_ir::Expr::Var(v))));
            Some(code_ir::Expr::Block(stmts))
        }
        // map_merge(base, overlay) → unwrap base, extend with overlay, re-wrap.
        // O(|overlay|) amortized.
        "map_merge" if args.len() == 2 => {
            let base = strip_outer_clone(collection.clone());
            let overlay = compile_expr(&args[1].1, ctx, counter);
            let v = fresh(counter, "map_merged");
            let mut stmts = rc_unwrap_stmts(&v, base, counter);
            stmts.push(code_ir::Stmt::Expr(code_ir::Expr::RawCode(format!(
                "{v}.extend(Rc::try_unwrap({}).unwrap_or_else(|rc| (*rc).clone()))",
                render_expr_inline(&overlay)
            ))));
            stmts.push(code_ir::Stmt::TailExpr(rc_wrap(code_ir::Expr::Var(v))));
            Some(code_ir::Expr::Block(stmts))
        }
        _ => None,
    }
}

/// Quick inline rendering of a code_ir::Expr for use in RawCode interpolation.
fn render_expr_inline(expr: &code_ir::Expr) -> String {
    crate::render_rust::render_expr_pub(expr)
}

/// Resolve a named argument by name first, falling back to positional index.
fn resolve_named_or_positional(
    args: &[(Option<String>, ast::Expr)],
    name: &str,
    pos: usize,
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::Expr {
    // Try named first
    if let Some(index) = args.iter().position(|(n, _)| n.as_deref() == Some(name)) {
        return compile_call_arg(args, index, ctx, counter);
    }
    // Fall back to positional
    compile_call_arg(args, pos, ctx, counter)
}

// ---------------------------------------------------------------------------
// Intrinsic helpers — For-loop unrollings for cross-target compatibility
// ---------------------------------------------------------------------------

fn compile_map_intrinsic(
    collection: &code_ir::Expr,
    collection_expr: &ast::Expr,
    mapper: Option<&ast::Expr>,
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::Expr {
    let result = fresh(counter, "mapped");
    let elem = fresh(counter, "elem");
    let mapped_value = match mapper {
        Some(ast::Expr::Lambda(params, body)) => {
            let compiled =
                compile_collection_lambda_body(collection_expr, params, body, ctx, counter);
            params
                .first()
                .map(|p| substitute_var(&compiled, p, &code_ir::Expr::Var(elem.clone())))
                .unwrap_or(compiled)
        }
        Some(other) => code_ir::Expr::Call {
            func: Box::new(compile_expr(other, ctx, counter)),
            args: vec![code_ir::Expr::Var(elem.clone())],
            obligation: None,
        },
        None => code_ir::Expr::Var(elem.clone()),
    };
    code_ir::Expr::Block(vec![
        code_ir::Stmt::let_mut(
            &result,
            code_ir::Expr::MacroCall {
                name: "vec".to_string(),
                args: vec![],
            },
        ),
        code_ir::Stmt::For {
            binding: elem,
            iter: make_owned_iter(collection.clone()),
            body: vec![code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                receiver: Box::new(code_ir::Expr::Var(result.clone())),
                method: "push".to_string(),
                args: vec![mapped_value],
            })],
        },
        code_ir::Stmt::TailExpr(rc_wrap(code_ir::Expr::Var(result))),
    ])
}

fn compile_filter_intrinsic(
    collection: &code_ir::Expr,
    collection_expr: &ast::Expr,
    predicate: Option<&ast::Expr>,
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::Expr {
    let result = fresh(counter, "filtered");
    let elem = fresh(counter, "elem");
    let cond = match predicate {
        Some(ast::Expr::Lambda(params, body)) => {
            let compiled =
                compile_collection_lambda_body(collection_expr, params, body, ctx, counter);
            params
                .first()
                .map(|p| substitute_var(&compiled, p, &code_ir::Expr::Var(elem.clone())))
                .unwrap_or(compiled)
        }
        Some(other) => code_ir::Expr::Call {
            func: Box::new(compile_expr(other, ctx, counter)),
            args: vec![code_ir::Expr::Var(elem.clone())],
            obligation: None,
        },
        None => code_ir::Expr::BoolLit(true),
    };
    code_ir::Expr::Block(vec![
        code_ir::Stmt::let_mut(
            &result,
            code_ir::Expr::MacroCall {
                name: "vec".to_string(),
                args: vec![],
            },
        ),
        code_ir::Stmt::For {
            binding: elem.clone(),
            iter: make_owned_iter(collection.clone()),
            body: vec![code_ir::Stmt::Expr(code_ir::Expr::If {
                cond: Box::new(cond),
                then_body: vec![code_ir::Stmt::Expr(code_ir::Expr::MethodCall {
                    receiver: Box::new(code_ir::Expr::Var(result.clone())),
                    method: "push".to_string(),
                    args: vec![code_ir::Expr::Var(elem)],
                })],
                else_body: None,
            })],
        },
        code_ir::Stmt::TailExpr(rc_wrap(code_ir::Expr::Var(result))),
    ])
}

#[derive(Clone)]
struct FoldExprContext<'a> {
    init: Option<(&'a ast::Expr, ExprPath)>,
    func: Option<(&'a ast::Expr, ExprPath)>,
}

fn compile_fold_intrinsic(
    collection: &code_ir::Expr,
    collection_expr: &ast::Expr,
    fold: FoldExprContext<'_>,
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::Expr {
    let acc = fresh(counter, "acc");
    let elem = fresh(counter, "elem");
    let init_expr = fold
        .init
        .as_ref()
        .map(|(expr, path)| ctx.with_expr_path(path.clone(), || compile_expr(expr, ctx, counter)))
        .unwrap_or(code_ir::Expr::IntLit(0));
    let body_expr = match fold.func.as_ref() {
        Some((ast::Expr::Lambda(params, body), path)) => {
            // Mark the accumulator param so concat codegen can strip its .clone()
            // (safe: accumulator is reassigned each iteration → refcount=1 → in-place extend).
            let mut fold_ctx = ctx.clone();
            if let Some(p) = params.first() {
                fold_ctx.fold_accum_name = Some(p.clone());
            }
            if let (Some(param), Some(ty)) = (
                params.get(1),
                infer_list_element_ir_type(collection_expr, ctx),
            ) {
                fold_ctx.ir_scope.insert(param.clone(), ty);
            }
            // Compute use counts for lambda params with weight=1 (reassigned each iter).
            // Without this, lambda params are excluded from outer use_counts and default
            // to count=2 in compile_ident → always clone. With correct counts, single-use
            // accumulators are moved instead of cloned.
            let mut inner_counts = HashMap::new();
            count_ident_uses_expr(body, &mut inner_counts, 1);
            for param in params {
                if let Some(&count) = inner_counts.get(param) {
                    fold_ctx.use_counts.insert(param.clone(), count);
                }
            }
            let lambda_body_path = path.child(ExprPathStep::LambdaBody);
            let mut compiled =
                fold_ctx.with_expr_path(lambda_body_path, || compile_expr(body, &fold_ctx, counter));
            if let Some(p) = params.first() {
                compiled = substitute_var(&compiled, p, &code_ir::Expr::Var(acc.clone()));
            }
            if let Some(p) = params.get(1) {
                compiled = substitute_var(&compiled, p, &code_ir::Expr::Var(elem.clone()));
            }
            compiled
        }
        Some((other, path)) => code_ir::Expr::Call {
            func: Box::new(ctx.with_expr_path(path.clone(), || compile_expr(other, ctx, counter))),
            args: vec![
                code_ir::Expr::Var(acc.clone()),
                code_ir::Expr::Var(elem.clone()),
            ],
            obligation: None,
        },
        None => code_ir::Expr::Var(acc.clone()),
    };
    let acc_ir_type = ctx
        .current_expr_ir_type()
        .or_else(|| {
            fold.init.as_ref().and_then(|(expr, path)| {
                ctx.with_expr_path(path.clone(), || {
                    ctx.current_expr_ir_type()
                        .or_else(|| infer_known_expr_ir_type(expr, ctx))
                })
            })
        })
        .or_else(|| infer_fold_accumulator_ir_type(collection_expr, &fold, ctx))
        .or_else(|| {
            fold.init.as_ref().and_then(|(expr, _)| match expr {
                ast::Expr::List(items) if items.is_empty() => ctx
                    .current_return_ir_type
                    .clone()
                    .filter(|ty| matches!(ty, IrType::Generic(name, _) if name == "List")),
                _ => None,
            })
        });
    code_ir::Expr::Block(vec![
        code_ir::Stmt::Let {
            name: acc.clone(),
            mutable: true,
            expr: init_expr,
            ir_type: acc_ir_type,
        },
        code_ir::Stmt::For {
            binding: elem,
            iter: make_owned_iter(collection.clone()),
            body: vec![code_ir::Stmt::Assign {
                dest: code_ir::Expr::Var(acc.clone()),
                value: body_expr,
            }],
        },
        code_ir::Stmt::TailExpr(code_ir::Expr::Var(acc)),
    ])
}

fn compile_any_intrinsic(
    collection: &code_ir::Expr,
    collection_expr: &ast::Expr,
    predicate: Option<&ast::Expr>,
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::Expr {
    let result = fresh(counter, "any");
    let elem = fresh(counter, "elem");
    let cond = match predicate {
        Some(ast::Expr::Lambda(params, body)) => {
            let compiled =
                compile_collection_lambda_body(collection_expr, params, body, ctx, counter);
            params
                .first()
                .map(|p| substitute_var(&compiled, p, &code_ir::Expr::Var(elem.clone())))
                .unwrap_or(compiled)
        }
        Some(other) => code_ir::Expr::Call {
            func: Box::new(compile_expr(other, ctx, counter)),
            args: vec![code_ir::Expr::Var(elem.clone())],
            obligation: None,
        },
        None => code_ir::Expr::BoolLit(false),
    };
    code_ir::Expr::Block(vec![
        code_ir::Stmt::let_mut(&result, code_ir::Expr::BoolLit(false)),
        code_ir::Stmt::For {
            binding: elem,
            iter: make_owned_iter(collection.clone()),
            body: vec![code_ir::Stmt::Expr(code_ir::Expr::If {
                cond: Box::new(cond),
                then_body: vec![
                    code_ir::Stmt::Assign {
                        dest: code_ir::Expr::Var(result.clone()),
                        value: code_ir::Expr::BoolLit(true),
                    },
                    code_ir::Stmt::Expr(code_ir::Expr::RawCode("break".to_string())),
                ],
                else_body: None,
            })],
        },
        code_ir::Stmt::TailExpr(code_ir::Expr::Var(result)),
    ])
}

/// Compile `all(collection, predicate)` → for loop returning false on first non-match.
///
/// Complement of `any`: starts with `true`, becomes `false` when the predicate
/// fails, breaking early.
fn compile_all_intrinsic(
    collection: &code_ir::Expr,
    collection_expr: &ast::Expr,
    predicate: Option<&ast::Expr>,
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::Expr {
    let result = fresh(counter, "all");
    let elem = fresh(counter, "elem");
    let cond = match predicate {
        Some(ast::Expr::Lambda(params, body)) => {
            let compiled =
                compile_collection_lambda_body(collection_expr, params, body, ctx, counter);
            params
                .first()
                .map(|p| substitute_var(&compiled, p, &code_ir::Expr::Var(elem.clone())))
                .unwrap_or(compiled)
        }
        Some(other) => code_ir::Expr::Call {
            func: Box::new(compile_expr(other, ctx, counter)),
            args: vec![code_ir::Expr::Var(elem.clone())],
            obligation: None,
        },
        None => code_ir::Expr::BoolLit(true),
    };
    code_ir::Expr::Block(vec![
        code_ir::Stmt::let_mut(&result, code_ir::Expr::BoolLit(true)),
        code_ir::Stmt::For {
            binding: elem,
            iter: make_owned_iter(collection.clone()),
            body: vec![code_ir::Stmt::Expr(code_ir::Expr::If {
                cond: Box::new(code_ir::Expr::UnaryOp {
                    op: "!".to_string(),
                    expr: Box::new(cond),
                }),
                then_body: vec![
                    code_ir::Stmt::Assign {
                        dest: code_ir::Expr::Var(result.clone()),
                        value: code_ir::Expr::BoolLit(false),
                    },
                    code_ir::Stmt::Expr(code_ir::Expr::RawCode("break".to_string())),
                ],
                else_body: None,
            })],
        },
        code_ir::Stmt::TailExpr(code_ir::Expr::Var(result)),
    ])
}

/// Substitute a variable name in a code_ir expression tree.
fn substitute_var(expr: &code_ir::Expr, from: &str, to: &code_ir::Expr) -> code_ir::Expr {
    match expr {
        code_ir::Expr::Var(name) if name == from => to.clone(),
        code_ir::Expr::RawCode(s) => {
            // Substitute variable references within raw code strings.
            // Intrinsics like map_insert/map_merge render arguments inline
            // as RawCode. Without this, fold lambda parameters retain their
            // DSL names instead of being replaced with generated loop vars.
            let to_name = match to {
                code_ir::Expr::Var(name) => name.as_str(),
                _ => return expr.clone(),
            };
            let new_s = replace_word(s, from, to_name);
            if new_s == *s {
                expr.clone()
            } else {
                code_ir::Expr::RawCode(new_s)
            }
        }
        code_ir::Expr::Var(_)
        | code_ir::Expr::Str(_)
        | code_ir::Expr::IntLit(_)
        | code_ir::Expr::BoolLit(_)
        | code_ir::Expr::Value(_)
        | code_ir::Expr::Path(_) => expr.clone(),
        code_ir::Expr::Field(receiver, field) => {
            code_ir::Expr::Field(Box::new(substitute_var(receiver, from, to)), field.clone())
        }
        code_ir::Expr::Call {
            func,
            args,
            obligation,
        } => code_ir::Expr::Call {
            func: Box::new(substitute_var(func, from, to)),
            args: args.iter().map(|a| substitute_var(a, from, to)).collect(),
            obligation: *obligation,
        },
        code_ir::Expr::MethodCall {
            receiver,
            method,
            args,
        } => code_ir::Expr::MethodCall {
            receiver: Box::new(substitute_var(receiver, from, to)),
            method: method.clone(),
            args: args.iter().map(|a| substitute_var(a, from, to)).collect(),
        },
        code_ir::Expr::BinOp { left, op, right } => code_ir::Expr::BinOp {
            left: Box::new(substitute_var(left, from, to)),
            op: op.clone(),
            right: Box::new(substitute_var(right, from, to)),
        },
        code_ir::Expr::UnaryOp { op, expr: inner } => code_ir::Expr::UnaryOp {
            op: op.clone(),
            expr: Box::new(substitute_var(inner, from, to)),
        },
        code_ir::Expr::If {
            cond,
            then_body,
            else_body,
        } => code_ir::Expr::If {
            cond: Box::new(substitute_var(cond, from, to)),
            then_body: then_body
                .iter()
                .map(|s| substitute_var_in_stmt(s, from, to))
                .collect(),
            else_body: else_body.as_ref().map(|stmts| {
                stmts
                    .iter()
                    .map(|s| substitute_var_in_stmt(s, from, to))
                    .collect()
            }),
        },
        code_ir::Expr::Block(stmts) => code_ir::Expr::Block(
            stmts
                .iter()
                .map(|s| substitute_var_in_stmt(s, from, to))
                .collect(),
        ),
        code_ir::Expr::Ref(inner) => code_ir::Expr::Ref(Box::new(substitute_var(inner, from, to))),
        code_ir::Expr::RefMut(inner) => {
            code_ir::Expr::RefMut(Box::new(substitute_var(inner, from, to)))
        }
        code_ir::Expr::Deref(inner) => {
            code_ir::Expr::Deref(Box::new(substitute_var(inner, from, to)))
        }
        code_ir::Expr::Struct {
            name,
            fields,
            rest,
            field_types,
        } => code_ir::Expr::Struct {
            name: name.clone(),
            fields: fields
                .iter()
                .map(|(n, e)| (n.clone(), substitute_var(e, from, to)))
                .collect(),
            rest: rest.as_ref().map(|r| Box::new(substitute_var(r, from, to))),
            field_types: field_types.clone(),
        },
        code_ir::Expr::Match { expr, arms } => code_ir::Expr::Match {
            expr: Box::new(substitute_var(expr, from, to)),
            arms: arms
                .iter()
                .map(|arm| code_ir::MatchArm {
                    pattern: arm.pattern.clone(),
                    body: arm
                        .body
                        .iter()
                        .map(|s| substitute_var_in_stmt(s, from, to))
                        .collect(),
                })
                .collect(),
        },
        code_ir::Expr::Closure { args, body } => {
            // Don't substitute if the closure shadows the variable
            if args.iter().any(|a| a == from) {
                expr.clone()
            } else {
                code_ir::Expr::Closure {
                    args: args.clone(),
                    body: Box::new(substitute_var(body, from, to)),
                }
            }
        }
        code_ir::Expr::FormatStr { template, args } => code_ir::Expr::FormatStr {
            template: template.clone(),
            args: args.iter().map(|a| substitute_var(a, from, to)).collect(),
        },
        code_ir::Expr::MacroCall { name, args } => code_ir::Expr::MacroCall {
            name: name.clone(),
            args: args.iter().map(|a| substitute_var(a, from, to)).collect(),
        },
        code_ir::Expr::Tuple(items) => {
            code_ir::Expr::Tuple(items.iter().map(|a| substitute_var(a, from, to)).collect())
        }
        code_ir::Expr::Array(items) => {
            code_ir::Expr::Array(items.iter().map(|a| substitute_var(a, from, to)).collect())
        }
    }
}

fn substitute_var_in_stmt(stmt: &code_ir::Stmt, from: &str, to: &code_ir::Expr) -> code_ir::Stmt {
    match stmt {
        code_ir::Stmt::Let {
            name,
            expr,
            mutable,
            ir_type,
        } => code_ir::Stmt::Let {
            name: name.clone(),
            expr: substitute_var(expr, from, to),
            mutable: *mutable,
            ir_type: ir_type.clone(),
        },
        code_ir::Stmt::Assign { dest, value } => code_ir::Stmt::Assign {
            dest: substitute_var(dest, from, to),
            value: substitute_var(value, from, to),
        },
        code_ir::Stmt::Expr(expr) => code_ir::Stmt::Expr(substitute_var(expr, from, to)),
        code_ir::Stmt::TailExpr(expr) => code_ir::Stmt::TailExpr(substitute_var(expr, from, to)),
        code_ir::Stmt::Return(expr) => code_ir::Stmt::Return(substitute_var(expr, from, to)),
        code_ir::Stmt::For {
            binding,
            iter,
            body,
        } => code_ir::Stmt::For {
            binding: binding.clone(),
            iter: substitute_var(iter, from, to),
            body: body
                .iter()
                .map(|s| substitute_var_in_stmt(s, from, to))
                .collect(),
        },
        code_ir::Stmt::BlockScope(stmts) => code_ir::Stmt::BlockScope(
            stmts
                .iter()
                .map(|s| substitute_var_in_stmt(s, from, to))
                .collect(),
        ),
        code_ir::Stmt::Bind {
            targets,
            intent,
            expr,
        } => code_ir::Stmt::Bind {
            targets: targets.clone(),
            intent: *intent,
            expr: substitute_var(expr, from, to),
        },
        other => other.clone(),
    }
}

/// Replace whole-word occurrences of `from` with `to` in a string.
/// A word boundary is a position where an adjacent character is not
/// alphanumeric or underscore (i.e., not part of a Rust identifier).
/// Skips matches followed by `:` (struct field name position).
fn replace_word(s: &str, from: &str, to: &str) -> String {
    let bytes = s.as_bytes();
    let from_bytes = from.as_bytes();
    let is_word = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let mut result = String::with_capacity(s.len());
    let mut i = 0;
    while i + from_bytes.len() <= bytes.len() {
        if &bytes[i..i + from_bytes.len()] == from_bytes {
            let before_ok = i == 0 || !is_word(bytes[i - 1]);
            let after_pos = i + from_bytes.len();
            let after_ok = after_pos >= bytes.len() || !is_word(bytes[after_pos]);
            // Skip struct field names: `name:` should not be substituted
            let is_field_name = after_pos < bytes.len() && bytes[after_pos] == b':';
            if before_ok && after_ok && !is_field_name {
                result.push_str(to);
                i += from_bytes.len();
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    if i < bytes.len() {
        result.push_str(&s[i..]);
    }
    result
}

fn compile_call(
    name: &str,
    args: &[(Option<String>, ast::Expr)],
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::Expr {
    let ir_args: Vec<code_ir::Expr> = args
        .iter()
        .enumerate()
        .map(|(index, (arg_name, _))| {
            let expected_type = lookup_call_arg_type(name, index, arg_name.as_deref(), ctx);
            clone_if_needed(
                compile_call_arg_typed(args, index, expected_type, ctx, counter),
                ctx.fold_accum_name.as_deref(),
            )
        })
        .collect();
    let rust_name = to_snake_case(name);

    code_ir::Expr::Call {
        func: Box::new(code_ir::Expr::Var(rust_name)),
        args: ir_args,
        obligation: None,
    }
}

/// Add .clone() to variable/field expressions that would be consumed by a call.
/// This ensures generated code doesn't have use-after-move errors.
/// Redundant clones are optimized away by the compiler.
fn clone_if_needed(expr: code_ir::Expr, fold_accum_name: Option<&str>) -> code_ir::Expr {
    match &expr {
        // Var clone decision is already made in compile_ident — pass through.
        code_ir::Expr::Var(_) => expr,
        code_ir::Expr::Field(receiver, _) => {
            // In fold context, skip cloning field accesses on the accumulator.
            // The accumulator is owned and reassigned each iteration, so partial
            // moves are valid and keep Rc refcount at 1 for in-place mutation.
            if let Some(accum) = fold_accum_name {
                if matches!(receiver.as_ref(), code_ir::Expr::Var(name) if name == accum) {
                    return expr;
                }
            }
            code_ir::Expr::MethodCall {
                receiver: Box::new(expr),
                method: "clone".to_string(),
                args: vec![],
            }
        }
        // Literals, calls, etc. are temporary values — don't clone
        _ => expr,
    }
}

/// Strip the outer `.clone()` from a compiled expression, producing a move.
/// Used for intrinsics that consume their first argument (concat, append) so
/// that `Rc::try_unwrap` in the runtime sees refcount 1 and mutates in place.
fn strip_outer_clone(expr: code_ir::Expr) -> code_ir::Expr {
    if matches!(&expr, code_ir::Expr::MethodCall { method, args, .. } if method == "clone" && args.is_empty())
    {
        match expr {
            code_ir::Expr::MethodCall { receiver, .. } => *receiver,
            _ => unreachable!(),
        }
    } else {
        expr
    }
}

// ---------------------------------------------------------------------------
// Binary / unary operators
// ---------------------------------------------------------------------------

fn compile_binop(op: &ast::BinOp) -> String {
    match op {
        ast::BinOp::Add => "+".to_string(),
        ast::BinOp::Sub => "-".to_string(),
        ast::BinOp::Mul => "*".to_string(),
        ast::BinOp::Div => "/".to_string(),
        ast::BinOp::Mod => "%".to_string(),
        ast::BinOp::Eq => "==".to_string(),
        ast::BinOp::Ne => "!=".to_string(),
        ast::BinOp::Lt => "<".to_string(),
        ast::BinOp::Gt => ">".to_string(),
        ast::BinOp::Le => "<=".to_string(),
        ast::BinOp::Ge => ">=".to_string(),
        ast::BinOp::And => "&&".to_string(),
        ast::BinOp::Or => "||".to_string(),
        ast::BinOp::NullCoalesce => "??".to_string(),
    }
}

fn compile_unaryop(op: &ast::UnaryOp) -> String {
    match op {
        ast::UnaryOp::Not => "!".to_string(),
        ast::UnaryOp::Neg => "-".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Match
// ---------------------------------------------------------------------------

fn compile_match(
    scrutinee: &ast::Expr,
    arms: &[ast::MatchArm],
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::Expr {
    compile_match_typed(scrutinee, arms, ctx, None, counter)
}

fn compile_match_typed(
    scrutinee: &ast::Expr,
    arms: &[ast::MatchArm],
    ctx: &CompileContext,
    result_expected_type: Option<&str>,
    counter: &mut usize,
) -> code_ir::Expr {
    let has_none_arm = arms.iter().any(|a| is_null_pattern(&a.pattern));
    let mut compiled_scrutinee = ctx.with_child_expr_path(ExprPathStep::MatchScrutinee, || {
        compile_expr(scrutinee, ctx, counter)
    });
    // If all non-wildcard/non-null arms use string literal patterns and the
    // scrutinee is not a literal, add a conversion so Rust can match &str patterns
    // against a String value:
    //   - For Option<String> (has_none_arm), use .as_deref() → Option<&str>
    //   - For plain String, use .as_str() → &str
    let all_string_arms = arms.iter().all(|a| {
        matches!(&a.pattern, ast::Pattern::Literal(ast::Literal::String(_)))
            || matches!(&a.pattern, ast::Pattern::Wildcard)
            || is_null_pattern(&a.pattern)
    });
    if all_string_arms && !arms.is_empty() && !matches!(scrutinee, ast::Expr::Literal(_)) {
        let method = if has_none_arm { "as_deref" } else { "as_str" };
        compiled_scrutinee = code_ir::Expr::MethodCall {
            receiver: Box::new(compiled_scrutinee),
            method: method.to_string(),
            args: vec![],
        };
    }
    // Infer the scrutinee's type so variant patterns can be resolved to the
    // correct parent enum (e.g., LitStr -> LiteralValue::LitStr, not TokenKind::LitStr).
    let scrutinee_type =
        infer_scrutinee_type(scrutinee, ctx).or_else(|| infer_type_from_arms(arms, ctx));
    // Also infer the scrutinee's IrType so we can propagate inner types through
    // Option patterns (Some { value: binding } → binding gets the unwrapped type).
    let scrutinee_ir_type = infer_known_expr_ir_type(scrutinee, ctx);
    let result_expected_type = result_expected_type
        .map(str::to_string)
        .or_else(|| infer_match_result_type(arms, ctx));
    code_ir::Expr::Match {
        expr: Box::new(compiled_scrutinee),
        arms: arms
            .iter()
            .enumerate()
            .map(|(index, a)| {
                compile_match_arm(
                    a,
                    index,
                    has_none_arm,
                    scrutinee_type.as_deref(),
                    result_expected_type.as_deref(),
                    scrutinee_ir_type.as_ref(),
                    ctx,
                    counter,
                )
            })
            .collect(),
    }
}

/// Infer the type of a match scrutinee expression from context.
///
/// Uses parameter types, field types, and struct field types to determine
/// what enum type the scrutinee evaluates to. This enables correct variant
/// disambiguation when variants exist in multiple enums (e.g., LitStr in
/// both TokenKind and LiteralValue).
fn infer_scrutinee_type(expr: &ast::Expr, ctx: &CompileContext) -> Option<String> {
    match expr {
        // Direct variable: look up in param_types
        ast::Expr::Ident(name) => ctx
            .param_types
            .get(name.as_str())
            .cloned()
            .or_else(|| ctx.ir_scope.get(name.as_str()).and_then(named_type_from_ir)),
        // Field access: look up the field's type from the receiver's struct type
        ast::Expr::FieldAccess(receiver, field) => {
            let receiver_type = infer_scrutinee_type(receiver, ctx)?;
            ctx.struct_field_types
                .get(&receiver_type)
                .and_then(|fields| fields.get(field.as_str()))
                .cloned()
        }
        _ => None,
    }
}

/// Infer the expected type from the match arms' variant patterns.
///
/// When `infer_scrutinee_type` can't determine the type (e.g. for match
/// bindings), look at which enum contains the most variant names used
/// in the arms. When multiple enums tie for the best overlap, return `None`
/// instead of depending on `HashMap` iteration order.
fn infer_type_from_arms(arms: &[ast::MatchArm], ctx: &CompileContext) -> Option<String> {
    let variant_names: Vec<&str> = arms
        .iter()
        .filter_map(|a| match &a.pattern {
            ast::Pattern::Ident(name) if name != "null" && name != "_" => Some(name.as_str()),
            ast::Pattern::Variant(name, _) if name != "Some" && name != "None" => {
                Some(name.as_str())
            }
            _ => None,
        })
        .collect();
    if variant_names.is_empty() {
        return None;
    }
    let matches: Vec<(&String, usize)> = ctx
        .enum_variants
        .iter()
        .filter_map(|(enum_name, variants)| {
            let count = variant_names
                .iter()
                .filter(|v| variants.contains(**v))
                .count();
            (count > 0).then_some((enum_name, count))
        })
        .collect();
    let best_count = matches.iter().map(|(_, count)| *count).max()?;
    let winners: Vec<&String> = matches
        .into_iter()
        .filter(|(_, count)| *count == best_count)
        .map(|(enum_name, _)| enum_name)
        .collect();
    if winners.len() == 1 {
        Some(winners[0].clone())
    } else {
        None
    }
}

fn collect_result_variant_names(expr: &ast::Expr, out: &mut Vec<String>) {
    match expr {
        ast::Expr::Ident(name) if name != "null" && name != "None" => out.push(name.clone()),
        ast::Expr::Record(Some(name), _) if name != "Some" && name != "None" => {
            out.push(name.clone())
        }
        ast::Expr::If(_, then_expr, Some(else_expr)) => {
            collect_result_variant_names(then_expr, out);
            collect_result_variant_names(else_expr, out);
        }
        ast::Expr::If(_, then_expr, None) => collect_result_variant_names(then_expr, out),
        ast::Expr::Block(stmts) => {
            if let Some(ast::Stmt::Expr(expr)) = stmts.last() {
                collect_result_variant_names(expr, out);
            }
        }
        ast::Expr::Match(_, arms) => {
            for arm in arms {
                collect_result_variant_names(&arm.body, out);
            }
        }
        _ => {}
    }
}

fn infer_type_from_result_exprs(exprs: &[&ast::Expr], ctx: &CompileContext) -> Option<String> {
    let mut variant_names = Vec::new();
    for expr in exprs {
        collect_result_variant_names(expr, &mut variant_names);
    }
    if variant_names.is_empty() {
        return None;
    }
    let matches: Vec<(&String, usize)> = ctx
        .enum_variants
        .iter()
        .filter_map(|(enum_name, variants)| {
            let count = variant_names
                .iter()
                .filter(|variant| variants.contains(variant.as_str()))
                .count();
            (count > 0).then_some((enum_name, count))
        })
        .collect();
    let best_count = matches.iter().map(|(_, count)| *count).max()?;
    let winners: Vec<&String> = matches
        .into_iter()
        .filter(|(_, count)| *count == best_count)
        .map(|(enum_name, _)| enum_name)
        .collect();
    if winners.len() == 1 {
        Some(winners[0].clone())
    } else {
        None
    }
}

fn infer_match_result_type(arms: &[ast::MatchArm], ctx: &CompileContext) -> Option<String> {
    let body_exprs: Vec<&ast::Expr> = arms.iter().map(|arm| &arm.body).collect();
    infer_type_from_result_exprs(&body_exprs, ctx)
}

#[allow(clippy::too_many_arguments)]
fn compile_match_arm(
    arm: &ast::MatchArm,
    arm_index: usize,
    option_context: bool,
    expected_type: Option<&str>,
    result_expected_type: Option<&str>,
    scrutinee_ir_type: Option<&IrType>,
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::MatchArm {
    let mut pattern = compile_pattern_typed(&arm.pattern, ctx, expected_type);
    if option_context
        && !is_null_pattern(&arm.pattern)
        && !matches!(arm.pattern, ast::Pattern::Wildcard)
    {
        // If the pattern contains an `if guard` (from string literal matching),
        // extract it before wrapping with Some() to avoid creating an experimental
        // "guard pattern" (RFC #129967). The guard goes on the match arm, not inside Some().
        if let Some(guard_pos) = pattern.find(" if ") {
            let guard = pattern[guard_pos..].to_string();
            let base_pattern = pattern[..guard_pos].to_string();
            pattern = format!("Some({base_pattern}){guard}");
        } else {
            pattern = format!("Some({pattern})");
        }
    }
    // Collect deref let-bindings for boxed fields in variant patterns.
    // If the pattern destructures a variant with boxed fields, we need
    // `let field = *field;` to deref Box<T> → T for the body.
    // Recurses into nested variant patterns (e.g., Some { value: Optional { inner: x } }).
    let mut deref_stmts = Vec::new();
    collect_boxed_deref_stmts(&arm.pattern, ctx, &mut deref_stmts);
    // Track match bindings from optional fields so the body compilation
    // knows they're already Option<T> and doesn't double-wrap in Some().
    let body_ctx = if let ast::Pattern::Variant(variant_name, fields) = &arm.pattern {
        let mut opt_bindings = Vec::new();
        let mut type_bindings = Vec::new();
        let mut ir_type_bindings = Vec::new();
        for (field_name, field_pat) in fields {
            if let ast::Pattern::Ident(bind_name) = field_pat {
                // Check if this field is optional in the variant's struct
                if let Some(opt_fields) = ctx.optional_fields.get(variant_name.as_str()) {
                    if opt_fields.contains(field_name.as_str()) {
                        opt_bindings.push(bind_name.clone());
                    }
                }
                // Track field type for scrutinee type inference on bindings
                if let Some(field_types) = ctx.struct_field_types.get(variant_name.as_str()) {
                    if let Some(ft) = field_types.get(field_name.as_str()) {
                        type_bindings.push((bind_name.clone(), ft.clone()));
                    }
                }
                if let Some(field_types) = ctx.struct_field_ir_types.get(variant_name.as_str()) {
                    if let Some((_, ty)) = field_types.iter().find(|(name, _)| name == field_name) {
                        ir_type_bindings.push((bind_name.clone(), ty.clone()));
                    }
                }
            }
        }
        // Special case: Some { value: binding } → propagate the inner type
        // from the scrutinee's Optional type to the binding.
        if variant_name == "Some" && fields.len() == 1 && fields[0].0 == "value" {
            if let ast::Pattern::Ident(bind_name) = &fields[0].1 {
                if let Some(IrType::Optional(inner)) = scrutinee_ir_type {
                    ir_type_bindings.push((bind_name.clone(), inner.as_ref().clone()));
                }
            }
        }

        if opt_bindings.is_empty() && type_bindings.is_empty() && ir_type_bindings.is_empty() {
            std::borrow::Cow::Borrowed(ctx)
        } else {
            let mut augmented = ctx.clone();
            for name in opt_bindings {
                augmented.optional_params.insert(name);
            }
            for (name, ty) in type_bindings {
                augmented.param_types.insert(name, ty);
            }
            for (name, ty) in ir_type_bindings {
                augmented.ir_scope.insert(name, ty);
            }
            std::borrow::Cow::Owned(augmented)
        }
    } else {
        std::borrow::Cow::Borrowed(ctx)
    };

    let mut body = deref_stmts;
    body.push(code_ir::Stmt::TailExpr(body_ctx.with_child_expr_path(
        ExprPathStep::MatchArmBody(arm_index),
        || compile_expr_typed(&arm.body, &body_ctx, result_expected_type, counter),
    )));
    code_ir::MatchArm { pattern, body }
}

fn is_null_pattern(pat: &ast::Pattern) -> bool {
    matches!(pat, ast::Pattern::Ident(name) if name == "null")
        || matches!(pat, ast::Pattern::Literal(ast::Literal::None))
}

/// Compile a pattern with optional expected type context for variant resolution.
///
/// When `expected_type` is Some, ambiguous variants are resolved against that
/// type's known variants instead of the global `variant_to_enum` map.
fn compile_pattern_typed(
    pat: &ast::Pattern,
    ctx: &CompileContext,
    expected_type: Option<&str>,
) -> String {
    match pat {
        ast::Pattern::Ident(name) => {
            if name == "null" {
                "None".to_string()
            } else if let Some(et) = expected_type {
                // If we have an expected type and it's an enum containing this variant, use it
                if let Some(variants) = ctx.enum_variants.get(et) {
                    if variants.contains(name.as_str()) {
                        return format!("{et}::{name}");
                    }
                }
                // Fall back to global map
                if let Some(enum_name) = ctx.variant_to_enum.get(name.as_str()) {
                    format!("{enum_name}::{name}")
                } else {
                    name.clone()
                }
            } else if let Some(enum_name) = ctx.variant_to_enum.get(name.as_str()) {
                format!("{enum_name}::{name}")
            } else {
                name.clone()
            }
        }
        ast::Pattern::Variant(name, fields) => {
            // Resolve variant name: use expected_type if available, else global map
            let qualified = if let Some(et) = expected_type {
                if let Some(variants) = ctx.enum_variants.get(et) {
                    if variants.contains(name.as_str()) {
                        format!("{et}::{name}")
                    } else {
                        ctx.variant_to_enum
                            .get(name.as_str())
                            .map(|e| format!("{e}::{name}"))
                            .unwrap_or_else(|| name.clone())
                    }
                } else {
                    ctx.variant_to_enum
                        .get(name.as_str())
                        .map(|e| format!("{e}::{name}"))
                        .unwrap_or_else(|| name.clone())
                }
            } else {
                ctx.variant_to_enum
                    .get(name.as_str())
                    .map(|e| format!("{e}::{name}"))
                    .unwrap_or_else(|| name.clone())
            };
            if fields.is_empty() {
                qualified
            } else if name == "Some" && fields.len() == 1 && fields[0].0 == "value" {
                // Some { value: x } → Some(x) in Rust
                // If the inner pattern is a string literal, Rust can't match &str
                // against String in Some(). Use ref binding + guard.
                if let ast::Pattern::Literal(ast::Literal::String(s)) = &fields[0].1 {
                    format!("Some(ref __some_val) if __some_val == \"{s}\"")
                } else {
                    let inner = compile_pattern_typed(&fields[0].1, ctx, expected_type);
                    // If the inner pattern contains a guard (from string literal field
                    // matching), extract it so the guard is at the match arm level,
                    // not inside Some() where it would be an experimental "guard pattern".
                    if let Some(guard_pos) = inner.find(" if ") {
                        let base = &inner[..guard_pos];
                        let guard = &inner[guard_pos..];
                        format!("Some({base}){guard}")
                    } else {
                        format!("Some({inner})")
                    }
                }
            } else {
                // Look up field types for this variant to provide context to sub-patterns
                let variant_field_types = ctx.struct_field_types.get(name.as_str());
                let mut guards: Vec<String> = Vec::new();
                let field_pats: Vec<String> = fields
                    .iter()
                    .map(|(n, p)| {
                        // If a field pattern is a string literal, Rust can't match &str
                        // against a String field directly. Use a ref binding + guard instead.
                        if let ast::Pattern::Literal(ast::Literal::String(s)) = p {
                            guards.push(format!("{n} == \"{s}\""));
                            format!("ref {n}")
                        } else {
                            let field_type = variant_field_types.and_then(|ft| ft.get(n.as_str()));
                            let compiled =
                                compile_pattern_typed(p, ctx, field_type.map(|s| s.as_str()));
                            // Use shorthand field pattern when binding name matches field name
                            if compiled == *n {
                                n.clone()
                            } else {
                                format!("{n}: {compiled}")
                            }
                        }
                    })
                    .collect();
                let pattern = format!("{} {{ {}, .. }}", qualified, field_pats.join(", "));
                if guards.is_empty() {
                    pattern
                } else {
                    format!("{pattern} if {}", guards.join(" && "))
                }
            }
        }
        ast::Pattern::Wildcard => "_".to_string(),
        ast::Pattern::Literal(lit) => match lit {
            ast::Literal::Int(n) => n.to_string(),
            ast::Literal::Float(f) => format!("{f:?}"),
            ast::Literal::String(s) => format!("\"{s}\""),
            ast::Literal::Bool(b) => b.to_string(),
            ast::Literal::None => "None".to_string(),
        },
    }
}

// ---------------------------------------------------------------------------
// If / else
// ---------------------------------------------------------------------------

fn compile_if(
    cond: &ast::Expr,
    then_expr: &ast::Expr,
    else_expr: &Option<Box<ast::Expr>>,
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::Expr {
    compile_if_typed(cond, then_expr, else_expr, ctx, None, counter)
}

fn compile_if_typed(
    cond: &ast::Expr,
    then_expr: &ast::Expr,
    else_expr: &Option<Box<ast::Expr>>,
    ctx: &CompileContext,
    expected_type: Option<&str>,
    counter: &mut usize,
) -> code_ir::Expr {
    let then_stmts = ctx.with_child_expr_path(ExprPathStep::IfThen, || {
        expr_to_stmts_typed(then_expr, ctx, expected_type, counter)
    });
    let else_stmts = else_expr.as_ref().map(|expr| {
        ctx.with_child_expr_path(ExprPathStep::IfElse, || {
            expr_to_stmts_typed(expr, ctx, expected_type, counter)
        })
    });

    // If there's no else branch, any trailing TailExpr in the then-body
    // must be converted to Return — otherwise Rust requires an else clause
    // for value-producing if expressions.
    let then_body = if else_stmts.is_none() {
        convert_trailing_tailexpr_to_return(then_stmts)
    } else {
        then_stmts
    };

    code_ir::Expr::If {
        cond: Box::new(
            ctx.with_child_expr_path(ExprPathStep::IfCond, || compile_expr(cond, ctx, counter)),
        ),
        then_body,
        else_body: else_stmts,
    }
}

/// Convert any trailing TailExpr to a Return statement.
/// Used for if-then bodies without else branches.
fn convert_trailing_tailexpr_to_return(mut stmts: Vec<code_ir::Stmt>) -> Vec<code_ir::Stmt> {
    if let Some(last) = stmts.last_mut() {
        match last {
            code_ir::Stmt::TailExpr(expr) => {
                let expr = std::mem::replace(expr, code_ir::Expr::Tuple(vec![]));
                *last = code_ir::Stmt::Return(expr);
            }
            code_ir::Stmt::Expr(code_ir::Expr::If {
                then_body,
                else_body,
                ..
            }) => {
                *then_body = convert_trailing_tailexpr_to_return(std::mem::take(then_body));
                if let Some(eb) = else_body {
                    *eb = convert_trailing_tailexpr_to_return(std::mem::take(eb));
                }
            }
            _ => {}
        }
    }
    stmts
}

fn expr_to_stmts_typed(
    expr: &ast::Expr,
    ctx: &CompileContext,
    expected_type: Option<&str>,
    counter: &mut usize,
) -> Vec<code_ir::Stmt> {
    match expr {
        ast::Expr::Return(fields) => {
            let current_path = ctx.current_expr_path();
            vec![code_ir::Stmt::Return(compile_return_fields(
                fields,
                ctx,
                counter,
                |index| record_field_path(&current_path, index),
            ))]
        }
        ast::Expr::Block(stmts) => compile_stmt_sequence_typed(
            stmts,
            ctx,
            expected_type,
            counter,
            &StmtPathOwner::Block(ctx.current_expr_path()),
        ),
        _ => {
            vec![code_ir::Stmt::TailExpr(compile_expr_typed(
                expr,
                ctx,
                expected_type,
                counter,
            ))]
        }
    }
}

fn compile_stmt_sequence_typed(
    stmts: &[ast::Stmt],
    ctx: &CompileContext,
    expected_type: Option<&str>,
    counter: &mut usize,
    owner: &StmtPathOwner,
) -> Vec<code_ir::Stmt> {
    let len = stmts.len();
    let mut current_ctx = ctx.clone();
    let mut result = Vec::with_capacity(len);
    for (index, stmt) in stmts.iter().enumerate() {
        track_binding_before_compile(stmt, &mut current_ctx);
        let compiled = compile_stmt_typed(
            stmt,
            index,
            index + 1 == len,
            &current_ctx,
            expected_type,
            counter,
            owner,
        );
        track_binding_after_compile(stmt, &compiled, &mut current_ctx);
        result.push(compiled);
    }
    result
}

fn compile_stmt_typed(
    stmt: &ast::Stmt,
    stmt_index: usize,
    is_last: bool,
    ctx: &CompileContext,
    expected_type: Option<&str>,
    counter: &mut usize,
    owner: &StmtPathOwner,
) -> code_ir::Stmt {
    if is_last {
        if let ast::Stmt::Expr(expr) = stmt {
            return ctx.with_expr_path(
                stmt_expr_path(owner, stmt_index, StmtExprSlot::ExprStmt),
                || code_ir::Stmt::TailExpr(compile_expr_typed(expr, ctx, expected_type, counter)),
            );
        }
    }
    compile_stmt(stmt, stmt_index, is_last, ctx, counter, owner)
}

// ---------------------------------------------------------------------------
// String interpolation
// ---------------------------------------------------------------------------

fn compile_string_interp(
    parts: &[ast::StringPart],
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::Expr {
    let mut template = String::new();
    let mut args = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        match part {
            ast::StringPart::Literal(s) => {
                // Escape literal { and } for Rust format! strings
                template.push_str(&s.replace('{', "{{").replace('}', "}}"));
            }
            ast::StringPart::Expr(e) => {
                template.push_str("{}");
                args.push(
                    ctx.with_child_expr_path(ExprPathStep::StringPart(index), || {
                        compile_expr(e, ctx, counter)
                    }),
                );
            }
        }
    }
    code_ir::Expr::FormatStr { template, args }
}

// ---------------------------------------------------------------------------
// Helpers — static iteration, string concat, Option wrapping
// ---------------------------------------------------------------------------

/// Convert a collection expression into an owned iterator.
///
/// For static data (`STATIC.clone()`), strips the clone and uses `STATIC.iter().cloned()`.
/// For all other expressions (including `Rc<Vec<T>>`), uses `.iter().cloned()` which
/// works via Deref for Rc<Vec<T>>.
fn make_owned_iter(collection: code_ir::Expr) -> code_ir::Expr {
    match &collection {
        // Static data: STATIC.clone() → STATIC.iter().cloned()
        code_ir::Expr::MethodCall {
            receiver,
            method,
            args,
        } if method == "clone" && args.is_empty() => code_ir::Expr::MethodCall {
            receiver: Box::new(code_ir::Expr::MethodCall {
                receiver: receiver.clone(),
                method: "iter".to_string(),
                args: vec![],
            }),
            method: "cloned".to_string(),
            args: vec![],
        },
        // Rc<Vec<T>> and other collections: expr.iter().cloned()
        _ => code_ir::Expr::MethodCall {
            receiver: Box::new(code_ir::Expr::MethodCall {
                receiver: Box::new(collection),
                method: "iter".to_string(),
                args: vec![],
            }),
            method: "cloned".to_string(),
            args: vec![],
        },
    }
}

fn is_none_expr(expr: &code_ir::Expr) -> bool {
    matches!(expr, code_ir::Expr::Var(name) if name == "None")
}

/// Wrap an expression in `Rc::new(...)` for Rc<Vec<T>> list wrapping.
fn rc_wrap(expr: code_ir::Expr) -> code_ir::Expr {
    code_ir::Expr::Call {
        func: Box::new(code_ir::Expr::Path(vec![
            "Rc".to_string(),
            "new".to_string(),
        ])),
        args: vec![expr],
        obligation: None,
    }
}

/// Produce statements that Rc-unwrap an expression into a mutable Vec:
///   let __rc = expr;           // substitutable by substitute_var
///   let mut var = Rc::try_unwrap(__rc).unwrap_or_else(|rc| (*rc).clone());
///
/// The intermediate `__rc` variable ensures substitute_var can still
/// replace variable references inside `expr` (unlike RawCode).
fn rc_unwrap_stmts(var_name: &str, expr: code_ir::Expr, counter: &mut usize) -> Vec<code_ir::Stmt> {
    let rc_var = fresh(counter, "rc");
    vec![
        code_ir::Stmt::let_bind(&rc_var, expr),
        code_ir::Stmt::let_mut(
            var_name,
            code_ir::Expr::RawCode(format!(
                "Rc::try_unwrap({rc_var}).unwrap_or_else(|rc| (*rc).clone())"
            )),
        ),
    ]
}

/// Check if a receiver expression likely produces an Option<T>.
///
/// Used to convert `.value` field access (from .dag `Some { value: x }`)
/// into `.unwrap()` in Rust. Detects method chains that return Option:
/// `.last()`, `.first()`, `.get()`, `.cloned()` after these, etc.
fn is_likely_option_receiver(expr: &ast::Expr) -> bool {
    match expr {
        ast::Expr::Call(name, _) => {
            matches!(name.as_str(), "last" | "first" | "get" | "find")
        }
        ast::Expr::FieldAccess(inner, method) => {
            // Check for method-like field access chains
            // e.g., x.last().cloned() — the .cloned() is on an Option
            matches!(method.as_str(), "cloned" | "clone") && is_likely_option_receiver(inner)
        }
        _ => false,
    }
}

/// Context-aware version that can check optional_fields.
fn is_likely_option_receiver_ctx(expr: &ast::Expr, ctx: &CompileContext) -> bool {
    if is_likely_option_receiver(expr) {
        return true;
    }
    match expr {
        // A bare identifier is Option if it matches a known optional field name
        // in any struct (e.g., `return_type` from `FnDef.return_type: TypeExpr?`).
        ast::Expr::Ident(name) => {
            for opt_fields in ctx.optional_fields.values() {
                if opt_fields.contains(name.as_str()) {
                    return true;
                }
            }
            // Also check optional params
            ctx.optional_params.contains(name.as_str())
        }
        // A field access is Option if the accessed field is optional
        // (e.g., `p.module` where `ParseResult.module: Module?`).
        ast::Expr::FieldAccess(_, field) => {
            for opt_fields in ctx.optional_fields.values() {
                if opt_fields.contains(field.as_str()) {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}

/// Check if a struct field needs Box<> wrapping (recursive type).
///
/// Checks multiple key patterns because `compute_recursive_fields` uses
/// `(enum_name, "Variant::field")` for Sum type variants, while struct
/// construction code uses the variant name as `struct_name`.
fn needs_box_wrapping(struct_name: &str, field_name: &str, ctx: &CompileContext) -> bool {
    // Direct: (StructName, field_name)
    if ctx
        .boxed_fields
        .contains(&(struct_name.to_string(), field_name.to_string()))
    {
        return true;
    }
    // Variant qualified: (StructName, StructName::field_name)
    let qualified = format!("{struct_name}::{field_name}");
    if ctx
        .boxed_fields
        .contains(&(struct_name.to_string(), qualified))
    {
        return true;
    }
    // Via parent enum: if StructName is a variant of EnumName,
    // check (EnumName, StructName::field_name)
    if let Some(enum_name) = ctx.variant_to_enum.get(struct_name) {
        let variant_qualified = format!("{struct_name}::{field_name}");
        if ctx
            .boxed_fields
            .contains(&(enum_name.clone(), variant_qualified))
        {
            return true;
        }
    }
    false
}

fn is_boxed_field_access(receiver: &ast::Expr, field_name: &str, ctx: &CompileContext) -> bool {
    ctx.with_child_expr_path(ExprPathStep::FieldAccessBase, || ctx.current_expr_ir_type())
        .and_then(|ty| named_type_from_ir(&ty))
        .or_else(|| infer_known_expr_ir_type(receiver, ctx).and_then(|ty| named_type_from_ir(&ty)))
        .or_else(|| infer_scrutinee_type(receiver, ctx))
        .is_some_and(|type_name| needs_box_wrapping(&type_name, field_name, ctx))
}

/// Recursively collect deref let-bindings for boxed fields in a pattern tree.
///
/// Handles nested variant patterns such as `Some { value: Optional { inner: x } }`
/// where `inner` is a boxed field on `Optional` — the binding `x` needs `let x = *x;`
/// in the match body.
fn collect_boxed_deref_stmts(
    pat: &ast::Pattern,
    ctx: &CompileContext,
    out: &mut Vec<code_ir::Stmt>,
) {
    if let ast::Pattern::Variant(variant_name, fields) = pat {
        for (field_name, field_pat) in fields {
            match field_pat {
                ast::Pattern::Ident(bind_name) => {
                    if needs_box_wrapping(variant_name, field_name, ctx) {
                        out.push(code_ir::Stmt::Let {
                            name: bind_name.clone(),
                            expr: code_ir::Expr::Deref(Box::new(code_ir::Expr::Var(
                                bind_name.clone(),
                            ))),
                            mutable: false,
                            ir_type: None,
                        });
                    }
                }
                // Recurse into nested variant patterns
                ast::Pattern::Variant(_, _) => {
                    collect_boxed_deref_stmts(field_pat, ctx, out);
                }
                _ => {}
            }
        }
    }
}

/// Check if an AST expression will produce an already-optional value.
///
/// Returns true when the expression accesses a field that is itself `T?` in
/// the source struct, meaning the compiled Rust type is already `Option<T>`
/// and should NOT be wrapped in another `Some()`.
fn is_already_optional_expr(expr: &ast::Expr, ctx: &CompileContext, target_struct: &str) -> bool {
    // Only check optionality against the specific target struct being constructed
    let target_opt_fields = ctx.optional_fields.get(target_struct);
    match expr {
        // field_access: x.field_name — check if field_name is optional in the receiver's struct
        ast::Expr::FieldAccess(receiver, field_name) => {
            if let ast::Expr::Ident(_) = receiver.as_ref() {
                // Check the receiver's struct type for optionality
                if let Some(receiver_type) = infer_scrutinee_type(receiver, ctx) {
                    if let Some(opt_fields) = ctx.optional_fields.get(&receiver_type) {
                        return opt_fields.contains(field_name.as_str());
                    }
                }
                // Fallback: check ALL structs — field accesses are specific enough
                // that name-based matching is reliable (unlike bare identifiers).
                for opt_fields in ctx.optional_fields.values() {
                    if opt_fields.contains(field_name.as_str()) {
                        return true;
                    }
                }
            }
            false
        }
        // Ident: check if the variable is a known optional parameter or has
        // an Optional type in ir_scope.
        ast::Expr::Ident(name) => {
            if ctx.optional_params.contains(name.as_str()) {
                return true;
            }
            // Check ir_scope: if the variable's inferred type is Optional, it's
            // already wrapped — don't double-wrap.
            if let Some(ir_type) = ctx.ir_scope.get(name.as_str()) {
                return matches!(ir_type, IrType::Optional(_));
            }
            // If this name is a known parameter with a concrete type, trust param_types
            // over the heuristic — a non-optional parameter is never already optional.
            if ctx.param_types.contains_key(name.as_str()) {
                return false;
            }
            // Only check target struct's optional fields, not all structs
            if let Some(opt_fields) = target_opt_fields {
                return opt_fields.contains(name.as_str());
            }
            false
        }
        // Record with Some variant: Some { value: x } → already Option<T>
        ast::Expr::Record(Some(name), _) if name == "Some" => true,
        // Call to Some: Some(x) → already Option<T>
        ast::Expr::Call(name, _) if name == "Some" => true,
        // Call to a function known to return Optional → already Option<T>
        ast::Expr::Call(name, _) => {
            ctx.optional_return_fns.contains(name.as_str())
                || ctx.optional_return_fns.contains(&to_snake_case(name))
        }
        // Null coalesce (x ?? y): result is non-optional by definition
        ast::Expr::BinOp(_, ast::BinOp::NullCoalesce, _) => false,
        // If/else where either branch is null or both branches are already Optional
        ast::Expr::If(_, then_expr, Some(else_expr)) => {
            is_null_ast_expr(then_expr)
                || is_null_ast_expr(else_expr)
                || (is_already_optional_expr(then_expr, ctx, target_struct)
                    && is_already_optional_expr(else_expr, ctx, target_struct))
        }
        // If without else where the then branch is null
        ast::Expr::If(_, then_expr, None) => is_null_ast_expr(then_expr),
        _ => false,
    }
}

/// Check if an AST expression is null/None.
fn is_null_ast_expr(expr: &ast::Expr) -> bool {
    match expr {
        ast::Expr::Ident(name) if name == "null" => true,
        ast::Expr::Literal(ast::Literal::None) => true,
        ast::Expr::Record(Some(name), _) if name == "None" => true,
        // Block with single expression
        ast::Expr::Block(stmts) if stmts.len() == 1 => {
            if let ast::Stmt::Expr(e) = &stmts[0] {
                is_null_ast_expr(e)
            } else {
                false
            }
        }
        _ => false,
    }
}

fn named_type_from_ir(ty: &IrType) -> Option<String> {
    match ty {
        IrType::Named(name) => Some(name.clone()),
        IrType::Optional(inner) => named_type_from_ir(inner),
        _ => None,
    }
}

fn infer_list_element_ir_type(expr: &ast::Expr, ctx: &CompileContext) -> Option<IrType> {
    match infer_known_expr_ir_type(expr, ctx)? {
        IrType::Generic(name, args) if name == "List" && args.len() == 1 => Some(args[0].clone()),
        _ => None,
    }
}

fn compile_collection_lambda_body(
    collection_expr: &ast::Expr,
    params: &[String],
    body: &ast::Expr,
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::Expr {
    let mut body_ctx = ctx.clone();
    if let (Some(param), Some(elem_ir_type)) = (
        params.first(),
        infer_list_element_ir_type(collection_expr, ctx),
    ) {
        body_ctx.ir_scope.insert(param.clone(), elem_ir_type);
    }
    body_ctx.with_child_expr_path(ExprPathStep::LambdaBody, || {
        compile_expr(body, &body_ctx, counter)
    })
}

fn infer_fold_result_ir_type(
    expr: &ast::Expr,
    acc_param: Option<&str>,
    ctx: &CompileContext,
) -> Option<IrType> {
    match expr {
        ast::Expr::Ident(name) if Some(name.as_str()) == acc_param => None,
        ast::Expr::Call(name, args) if name == "concat" => args.iter().find_map(|(_, arg)| {
            if matches!(arg, ast::Expr::Ident(name) if Some(name.as_str()) == acc_param) {
                None
            } else {
                infer_known_expr_ir_type(arg, ctx)
            }
        }),
        ast::Expr::If(_, then_expr, Some(else_expr)) => {
            let then_ty = infer_fold_result_ir_type(then_expr, acc_param, ctx);
            let else_ty = infer_fold_result_ir_type(else_expr, acc_param, ctx);
            match (then_ty, else_ty) {
                (Some(left), Some(right)) if left == right => Some(left),
                (Some(ty), None) | (None, Some(ty)) => Some(ty),
                _ => None,
            }
        }
        ast::Expr::Block(stmts) => stmts.last().and_then(|stmt| match stmt {
            ast::Stmt::Expr(expr) => infer_fold_result_ir_type(expr, acc_param, ctx),
            _ => None,
        }),
        _ => infer_known_expr_ir_type(expr, ctx),
    }
}

fn infer_fold_accumulator_ir_type(
    collection_expr: &ast::Expr,
    fold: &FoldExprContext<'_>,
    ctx: &CompileContext,
) -> Option<IrType> {
    let (func_expr, _) = fold.func.as_ref()?;
    let ast::Expr::Lambda(params, body) = func_expr else {
        return None;
    };
    let acc_param = params.first().map(|name| name.as_str());
    let elem_param = params.get(1).map(|name| name.as_str());
    let elem_ir_type = infer_list_element_ir_type(collection_expr, ctx);
    let mut body_ctx = ctx.clone();
    if let (Some(param), Some(ty)) = (elem_param, elem_ir_type) {
        body_ctx.ir_scope.insert(param.to_string(), ty);
    }
    infer_fold_result_ir_type(body, acc_param, &body_ctx)
}

fn infer_known_field_ir_type(
    receiver_type: &IrType,
    field: &str,
    ctx: &CompileContext,
) -> Option<IrType> {
    match receiver_type {
        IrType::Optional(inner) if field == "value" => Some((**inner).clone()),
        IrType::Named(name) => ctx
            .struct_field_ir_types
            .get(name)
            .and_then(|fields| fields.iter().find(|(field_name, _)| field_name == field))
            .map(|(_, ty)| ty.clone()),
        IrType::Record(fields) => fields
            .iter()
            .find(|(field_name, _)| field_name == field)
            .map(|(_, ty)| ty.clone()),
        IrType::Tuple(items) if field == "first" => items.first().cloned(),
        IrType::Tuple(items) if field == "second" => items.get(1).cloned(),
        _ => None,
    }
}

fn infer_known_expr_ir_type(expr: &ast::Expr, ctx: &CompileContext) -> Option<IrType> {
    match expr {
        ast::Expr::Ident(name) => ctx.ir_scope.get(name).cloned(),
        ast::Expr::FieldAccess(receiver, field) => {
            let receiver_type = infer_known_expr_ir_type(receiver, ctx)?;
            infer_known_field_ir_type(&receiver_type, field, ctx)
        }
        ast::Expr::Call(name, args) if name == "Some" && args.len() == 1 => {
            infer_known_expr_ir_type(&args[0].1, ctx).map(|inner| IrType::Optional(Box::new(inner)))
        }
        ast::Expr::Call(name, _) => ctx.fn_return_ir_types.get(name).cloned(),
        ast::Expr::Record(Some(name), fields)
            if name == "Some" && fields.len() == 1 && fields[0].0 == "value" =>
        {
            infer_known_expr_ir_type(&fields[0].1, ctx)
                .map(|inner| IrType::Optional(Box::new(inner)))
        }
        ast::Expr::Record(Some(name), _) if name != "None" => Some(IrType::Named(name.clone())),
        ast::Expr::List(items) => {
            let first = infer_known_expr_ir_type(items.first()?, ctx)?;
            Some(IrType::Generic("List".to_string(), vec![first]))
        }
        _ => None,
    }
}

/// Compile a field value expression with type-aware variant resolution.
///
/// When the field's declared type is an enum that contains the bare identifier
/// as a variant, use `EnumType::Variant` instead of the global `variant_to_enum`
/// mapping (which may be wrong for ambiguous variants like `Info`).
fn compile_expr_in_field_context(
    expr: &ast::Expr,
    field_name: &str,
    field_types: Option<&std::collections::HashMap<String, String>>,
    ctx: &CompileContext,
    counter: &mut usize,
) -> code_ir::Expr {
    // Resolve expected type for this field
    let expected_type = field_types.and_then(|ft_map| ft_map.get(field_name));

    compile_expr_typed(expr, ctx, expected_type.map(|s| s.as_str()), counter)
}

/// Check if compiled IR contains unresolved record construction, which
/// indicates fn codegen lost the target struct name.
pub fn body_has_empty_construct(stmts: &[code_ir::Stmt]) -> bool {
    stmts.iter().any(|s| match s {
        code_ir::Stmt::TailExpr(e) | code_ir::Stmt::Expr(e) => expr_has_empty(e),
        code_ir::Stmt::Let { expr, .. } => expr_has_empty(expr),
        _ => false,
    })
}

fn expr_has_empty(e: &code_ir::Expr) -> bool {
    match e {
        code_ir::Expr::Struct { name, .. } if name.is_empty() => true,
        code_ir::Expr::Match { arms, .. } => arms.iter().any(|a| body_has_empty_construct(&a.body)),
        code_ir::Expr::If {
            then_body,
            else_body,
            ..
        } => {
            body_has_empty_construct(then_body)
                || else_body
                    .as_ref()
                    .is_some_and(|b| body_has_empty_construct(b))
        }
        code_ir::Expr::Block(stmts) => body_has_empty_construct(stmts),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Removed: v2 bootstrap heuristics for + operator disambiguation.
//
// The `+` operator is now exclusively arithmetic. String and list
// concatenation use the `concat()` intrinsic function, which maps to
// `v2_rt::concat()` with trait dispatch. See the `concat` case in
// `compile_intrinsic_call()`.
//
// The following dead code was removed:
// - is_numeric_expr() — guessed whether + was arithmetic
// - contains_string_literal() — detected string concat chains
// - compile_string_concat() / flatten_concat_parts() — compiled + chains to format!()
// ---------------------------------------------------------------------------

// compile_string_interp remains for `"hello ${name}"` interpolation syntax,
// which is distinct from concat() and not affected by the + operator change.

// ---------------------------------------------------------------------------
// Anonymous record materialization
// ---------------------------------------------------------------------------

fn ir_type_to_type_id(ty: &IrType) -> Option<String> {
    match ty {
        IrType::Named(name) => Some(name.clone()),
        IrType::Generic(name, args) => {
            let rendered_args = args
                .iter()
                .map(ir_type_to_type_id)
                .collect::<Option<Vec<_>>>()?;
            Some(format!("{name}<{}>", rendered_args.join(", ")))
        }
        IrType::Optional(inner) => Some(format!("Optional<{}>", ir_type_to_type_id(inner)?)),
        IrType::Bool => Some("Bool".to_string()),
        IrType::Int => Some("Int".to_string()),
        IrType::Str => Some("String".to_string()),
        IrType::Tuple(items) => {
            let rendered_items = items
                .iter()
                .map(ir_type_to_type_id)
                .collect::<Option<Vec<_>>>()?;
            Some(format!("({})", rendered_items.join(", ")))
        }
        IrType::Unit => Some("Unit".to_string()),
        IrType::Record(_) | IrType::Unknown => None,
    }
}

fn ir_type_to_rust_type(ty: &IrType) -> Option<String> {
    match ty {
        IrType::Named(name) => Some(crate::type_mapping::resolve_and_emit(
            name,
            None,
            crate::type_mapping::Backend::Rust,
        )),
        IrType::Generic(name, args) if name == "List" && args.len() == 1 => {
            Some(format!("Rc<Vec<{}>>", ir_type_to_rust_type(&args[0])?))
        }
        IrType::Generic(name, args) if name == "Set" && args.len() == 1 => Some(format!(
            "std::collections::HashSet<{}>",
            ir_type_to_rust_type(&args[0])?
        )),
        IrType::Generic(name, args) if name == "Map" && args.len() == 2 => Some(format!(
            "std::collections::HashMap<{}, {}>",
            ir_type_to_rust_type(&args[0])?,
            ir_type_to_rust_type(&args[1])?
        )),
        IrType::Generic(name, args) => Some(format!(
            "{}<{}>",
            crate::type_mapping::resolve_and_emit(name, None, crate::type_mapping::Backend::Rust,),
            args.iter()
                .map(ir_type_to_rust_type)
                .collect::<Option<Vec<_>>>()?
                .join(", ")
        )),
        IrType::Optional(inner) => Some(format!("Option<{}>", ir_type_to_rust_type(inner)?)),
        IrType::Bool => Some("bool".to_string()),
        IrType::Int => Some("i64".to_string()),
        IrType::Str => Some("String".to_string()),
        IrType::Tuple(items) => {
            let rendered_items = items
                .iter()
                .map(ir_type_to_rust_type)
                .collect::<Option<Vec<_>>>()?;
            let tuple_body = if rendered_items.len() == 1 {
                format!("{},", rendered_items[0])
            } else {
                rendered_items.join(", ")
            };
            Some(format!("({tuple_body})"))
        }
        IrType::Unit => Some("()".to_string()),
        IrType::Record(_) | IrType::Unknown => None,
    }
}

/// Materialize typecheck-produced synthesized anonymous-record types into code IR.
#[allow(clippy::type_complexity)]
pub fn materialize_synthesized_anonymous_record_types(
    synthesized_types: &[SynthesizedAnonymousRecordType],
) -> (
    Vec<code_ir::Item>,
    HashMap<String, HashMap<String, String>>,
    HashMap<String, Vec<(String, IrType)>>,
) {
    let mut items = Vec::new();
    let mut new_field_types: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut new_field_ir_types: HashMap<String, Vec<(String, IrType)>> = HashMap::new();

    for synthesized_type in synthesized_types {
        let typed_shape = synthesized_type
            .fields
            .iter()
            .map(|(name, ty)| Some((name.clone(), ir_type_to_type_id(ty)?)))
            .collect::<Option<Vec<_>>>();
        let Some(typed_shape) = typed_shape else {
            continue;
        };
        let fields: Vec<(String, String, bool)> = synthesized_type
            .fields
            .iter()
            .filter_map(|(field_name, ty)| {
                ir_type_to_rust_type(ty).map(|rust_ty| (field_name.clone(), rust_ty, true))
            })
            .collect();
        if fields.len() != synthesized_type.fields.len() {
            continue;
        }

        new_field_types.insert(
            synthesized_type.name.clone(),
            typed_shape.into_iter().collect(),
        );
        new_field_ir_types.insert(
            synthesized_type.name.clone(),
            synthesized_type.fields.clone(),
        );

        items.push(code_ir::Item::Struct(code_ir::StructDef {
            name: synthesized_type.name.clone(),
            is_pub: false,
            derives: vec![
                "Debug".to_string(),
                "Clone".to_string(),
                "PartialEq".to_string(),
            ],
            fields,
            doc: vec!["Synthesized anonymous record type.".to_string()],
        }));
    }
    (items, new_field_types, new_field_ir_types)
}

// ---------------------------------------------------------------------------
// Tail-Call Optimization (TCO)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
enum TcoBlocker {
    NonTailSelfCall(&'static str),
    UnsupportedRecursiveContext(&'static str),
    ResidualSelfCallAfterLowering,
}

#[derive(Debug, Clone)]
struct TcoPlan {
    body: Vec<TcoPlanStmt>,
}

#[derive(Debug, Clone)]
enum TcoPlanStmt {
    Raw(code_ir::Stmt),
    Expr(TcoPlanExpr),
    TailExpr(TcoPlanExpr),
    BlockScope(Vec<TcoPlanStmt>),
    Break(code_ir::Expr),
    Recur(Vec<code_ir::Expr>),
}

#[derive(Debug, Clone)]
enum TcoPlanExpr {
    If {
        cond: Box<code_ir::Expr>,
        then_body: Vec<TcoPlanStmt>,
        else_body: Option<Vec<TcoPlanStmt>>,
    },
    Match {
        expr: Box<code_ir::Expr>,
        arms: Vec<TcoPlanArm>,
    },
    Block(Vec<TcoPlanStmt>),
}

#[derive(Debug, Clone)]
struct TcoPlanArm {
    pattern: String,
    body: Vec<TcoPlanStmt>,
}

#[derive(Debug, Clone)]
struct Planned<T> {
    value: T,
    has_recur: bool,
}

impl<T> Planned<T> {
    fn new(value: T, has_recur: bool) -> Self {
        Self { value, has_recur }
    }
}

fn is_self_call_by_func(func: &code_ir::Expr, fn_name: &str) -> bool {
    matches!(func, code_ir::Expr::Var(name) if name == fn_name)
}

/// Attempt tail-call optimization on a compiled function body.
///
/// The implementation is intentionally split into two pure phases:
/// 1. Build a `TcoPlan` that captures where recursion exits the function.
/// 2. Lower that plan into Rust-oriented `code_ir::Stmt::Loop` / `Break` /
///    `Continue` nodes.
///
/// This keeps eligibility and rewriting in one structural pass while still
/// allowing the Rust backend to decide how the iterative form is rendered.
pub fn apply_tco(
    fn_name: &str,
    param_names: &[String],
    body: &[code_ir::Stmt],
) -> Option<Vec<code_ir::Stmt>> {
    let plan = match plan_tco(fn_name, body) {
        Ok(Some(plan)) => plan,
        Ok(None) => return None,
        Err(_) => return None,
    };

    let lowered = lower_tco_plan(plan, param_names);
    if ensure_lowered_body_has_no_self_call(&lowered, fn_name).is_err() {
        return None;
    }
    Some(lowered)
}

fn plan_tco(fn_name: &str, body: &[code_ir::Stmt]) -> Result<Option<TcoPlan>, TcoBlocker> {
    let planned = plan_tco_stmts(body, fn_name, true)?;
    if planned.has_recur {
        Ok(Some(TcoPlan {
            body: planned.value,
        }))
    } else {
        Ok(None)
    }
}

fn plan_tco_stmts(
    stmts: &[code_ir::Stmt],
    fn_name: &str,
    tail_context: bool,
) -> Result<Planned<Vec<TcoPlanStmt>>, TcoBlocker> {
    let last_idx = stmts.len().saturating_sub(1);
    let mut planned = Vec::with_capacity(stmts.len());
    let mut has_recur = false;

    for (idx, stmt) in stmts.iter().enumerate() {
        let stmt_tail = tail_context && idx == last_idx;
        let next = plan_tco_stmt(stmt, fn_name, stmt_tail)?;
        has_recur |= next.has_recur;
        planned.push(next.value);
    }

    Ok(Planned::new(planned, has_recur))
}

fn plan_tco_stmt(
    stmt: &code_ir::Stmt,
    fn_name: &str,
    tail_context: bool,
) -> Result<Planned<TcoPlanStmt>, TcoBlocker> {
    match stmt {
        code_ir::Stmt::Return(expr) => plan_function_exit_expr(expr, fn_name),
        code_ir::Stmt::TailExpr(expr) => {
            if tail_context {
                plan_function_exit_expr(expr, fn_name)
            } else {
                plan_embedded_expr(expr, fn_name, true)
            }
        }
        code_ir::Stmt::Expr(expr) => plan_embedded_expr(expr, fn_name, false),
        code_ir::Stmt::Let { expr, .. } => {
            ensure_expr_has_no_self_call(expr, fn_name, "let binding")?;
            Ok(Planned::new(TcoPlanStmt::Raw(stmt.clone()), false))
        }
        code_ir::Stmt::Bind { expr, .. } => {
            ensure_expr_has_no_self_call(expr, fn_name, "bind expression")?;
            Ok(Planned::new(TcoPlanStmt::Raw(stmt.clone()), false))
        }
        code_ir::Stmt::Assign { dest, value } => {
            ensure_expr_has_no_self_call(dest, fn_name, "assignment target")?;
            ensure_expr_has_no_self_call(value, fn_name, "assignment value")?;
            Ok(Planned::new(TcoPlanStmt::Raw(stmt.clone()), false))
        }
        code_ir::Stmt::Assert(assert) => {
            if assert_has_self_call(assert, fn_name) {
                Err(TcoBlocker::NonTailSelfCall("assert"))
            } else {
                Ok(Planned::new(TcoPlanStmt::Raw(stmt.clone()), false))
            }
        }
        code_ir::Stmt::For { iter, body, .. } => {
            ensure_expr_has_no_self_call(iter, fn_name, "for iterator")?;
            if body_has_self_call(body, fn_name) {
                Err(TcoBlocker::UnsupportedRecursiveContext("for body"))
            } else {
                Ok(Planned::new(TcoPlanStmt::Raw(stmt.clone()), false))
            }
        }
        code_ir::Stmt::Loop { body } => {
            if body_has_self_call(body, fn_name) {
                Err(TcoBlocker::UnsupportedRecursiveContext("nested loop body"))
            } else {
                Ok(Planned::new(TcoPlanStmt::Raw(stmt.clone()), false))
            }
        }
        code_ir::Stmt::BlockScope(inner) => {
            let planned = plan_tco_stmts(inner, fn_name, false)?;
            Ok(Planned::new(
                TcoPlanStmt::BlockScope(planned.value),
                planned.has_recur,
            ))
        }
        code_ir::Stmt::Item(_)
        | code_ir::Stmt::Comment(_)
        | code_ir::Stmt::Blank
        | code_ir::Stmt::Continue
        | code_ir::Stmt::Break(_) => Ok(Planned::new(TcoPlanStmt::Raw(stmt.clone()), false)),
    }
}

fn plan_function_exit_expr(
    expr: &code_ir::Expr,
    fn_name: &str,
) -> Result<Planned<TcoPlanStmt>, TcoBlocker> {
    match expr {
        code_ir::Expr::Call { func, args, .. } if is_self_call_by_func(func, fn_name) => {
            if args.iter().any(|arg| expr_has_self_call(arg, fn_name)) {
                Err(TcoBlocker::NonTailSelfCall("tail call argument"))
            } else {
                Ok(Planned::new(TcoPlanStmt::Recur(args.clone()), true))
            }
        }
        code_ir::Expr::If {
            cond,
            then_body,
            else_body,
        } => {
            ensure_expr_has_no_self_call(cond, fn_name, "if condition")?;
            let then_plan = plan_tco_stmts(then_body, fn_name, true)?;
            let else_plan = match else_body {
                Some(stmts) => Some(plan_tco_stmts(stmts, fn_name, true)?),
                None => None,
            };
            let has_recur =
                then_plan.has_recur || else_plan.as_ref().is_some_and(|planned| planned.has_recur);
            Ok(Planned::new(
                TcoPlanStmt::Expr(TcoPlanExpr::If {
                    cond: cond.clone(),
                    then_body: then_plan.value,
                    else_body: else_plan.map(|planned| planned.value),
                }),
                has_recur,
            ))
        }
        code_ir::Expr::Match {
            expr: scrutinee,
            arms,
        } => {
            ensure_expr_has_no_self_call(scrutinee, fn_name, "match scrutinee")?;
            let mut planned_arms = Vec::with_capacity(arms.len());
            let mut has_recur = false;
            for arm in arms {
                let planned = plan_tco_stmts(&arm.body, fn_name, true)?;
                has_recur |= planned.has_recur;
                planned_arms.push(TcoPlanArm {
                    pattern: arm.pattern.clone(),
                    body: planned.value,
                });
            }
            Ok(Planned::new(
                TcoPlanStmt::Expr(TcoPlanExpr::Match {
                    expr: scrutinee.clone(),
                    arms: planned_arms,
                }),
                has_recur,
            ))
        }
        code_ir::Expr::Block(stmts) => {
            let planned = plan_tco_stmts(stmts, fn_name, true)?;
            Ok(Planned::new(
                TcoPlanStmt::Expr(TcoPlanExpr::Block(planned.value)),
                planned.has_recur,
            ))
        }
        other => {
            ensure_expr_has_no_self_call(other, fn_name, "return expression")?;
            Ok(Planned::new(TcoPlanStmt::Break(other.clone()), false))
        }
    }
}

fn plan_embedded_expr(
    expr: &code_ir::Expr,
    fn_name: &str,
    tail_expr: bool,
) -> Result<Planned<TcoPlanStmt>, TcoBlocker> {
    match expr {
        code_ir::Expr::If {
            cond,
            then_body,
            else_body,
        } => {
            ensure_expr_has_no_self_call(cond, fn_name, "if condition")?;
            let then_plan = plan_tco_stmts(then_body, fn_name, false)?;
            let else_plan = match else_body {
                Some(stmts) => Some(plan_tco_stmts(stmts, fn_name, false)?),
                None => None,
            };
            let has_recur =
                then_plan.has_recur || else_plan.as_ref().is_some_and(|planned| planned.has_recur);
            let plan_expr = TcoPlanExpr::If {
                cond: cond.clone(),
                then_body: then_plan.value,
                else_body: else_plan.map(|planned| planned.value),
            };
            Ok(Planned::new(
                if tail_expr {
                    TcoPlanStmt::TailExpr(plan_expr)
                } else {
                    TcoPlanStmt::Expr(plan_expr)
                },
                has_recur,
            ))
        }
        code_ir::Expr::Match {
            expr: scrutinee,
            arms,
        } => {
            ensure_expr_has_no_self_call(scrutinee, fn_name, "match scrutinee")?;
            let mut planned_arms = Vec::with_capacity(arms.len());
            let mut has_recur = false;
            for arm in arms {
                let planned = plan_tco_stmts(&arm.body, fn_name, false)?;
                has_recur |= planned.has_recur;
                planned_arms.push(TcoPlanArm {
                    pattern: arm.pattern.clone(),
                    body: planned.value,
                });
            }
            let plan_expr = TcoPlanExpr::Match {
                expr: scrutinee.clone(),
                arms: planned_arms,
            };
            Ok(Planned::new(
                if tail_expr {
                    TcoPlanStmt::TailExpr(plan_expr)
                } else {
                    TcoPlanStmt::Expr(plan_expr)
                },
                has_recur,
            ))
        }
        code_ir::Expr::Block(stmts) => {
            let planned = plan_tco_stmts(stmts, fn_name, false)?;
            let plan_expr = TcoPlanExpr::Block(planned.value);
            Ok(Planned::new(
                if tail_expr {
                    TcoPlanStmt::TailExpr(plan_expr)
                } else {
                    TcoPlanStmt::Expr(plan_expr)
                },
                planned.has_recur,
            ))
        }
        other => {
            let context = if tail_expr {
                "tail expression"
            } else {
                "expression statement"
            };
            ensure_expr_has_no_self_call(other, fn_name, context)?;
            Ok(Planned::new(
                TcoPlanStmt::Raw(if tail_expr {
                    code_ir::Stmt::TailExpr(other.clone())
                } else {
                    code_ir::Stmt::Expr(other.clone())
                }),
                false,
            ))
        }
    }
}

fn lower_tco_plan(plan: TcoPlan, param_names: &[String]) -> Vec<code_ir::Stmt> {
    let loop_vars: Vec<String> = param_names.iter().map(|p| format!("__tco_p_{p}")).collect();

    let mut preamble: Vec<code_ir::Stmt> = param_names
        .iter()
        .zip(loop_vars.iter())
        .map(|(param, lv)| code_ir::Stmt::Let {
            name: lv.clone(),
            mutable: true,
            expr: code_ir::Expr::Var(param.clone()),
            ir_type: None,
        })
        .collect();

    let rebind_stmts: Vec<code_ir::Stmt> = param_names
        .iter()
        .zip(loop_vars.iter())
        .map(|(param, lv)| code_ir::Stmt::Let {
            name: param.clone(),
            mutable: false,
            expr: code_ir::Expr::MethodCall {
                receiver: Box::new(code_ir::Expr::Var(lv.clone())),
                method: "clone".to_string(),
                args: vec![],
            },
            ir_type: None,
        })
        .collect();

    let mut loop_body = rebind_stmts;
    loop_body.extend(lower_tco_plan_stmts(&plan.body, &loop_vars));
    preamble.push(code_ir::Stmt::Loop { body: loop_body });
    preamble
}

fn lower_tco_plan_stmts(stmts: &[TcoPlanStmt], loop_vars: &[String]) -> Vec<code_ir::Stmt> {
    stmts
        .iter()
        .map(|stmt| lower_tco_plan_stmt(stmt, loop_vars))
        .collect()
}

fn lower_tco_plan_stmt(stmt: &TcoPlanStmt, loop_vars: &[String]) -> code_ir::Stmt {
    match stmt {
        TcoPlanStmt::Raw(stmt) => stmt.clone(),
        TcoPlanStmt::Expr(expr) => code_ir::Stmt::Expr(lower_tco_plan_expr(expr, loop_vars)),
        TcoPlanStmt::TailExpr(expr) => {
            code_ir::Stmt::TailExpr(lower_tco_plan_expr(expr, loop_vars))
        }
        TcoPlanStmt::BlockScope(body) => {
            code_ir::Stmt::BlockScope(lower_tco_plan_stmts(body, loop_vars))
        }
        TcoPlanStmt::Break(expr) => code_ir::Stmt::Break(expr.clone()),
        TcoPlanStmt::Recur(args) => lower_tco_recur(args, loop_vars),
    }
}

fn lower_tco_plan_expr(expr: &TcoPlanExpr, loop_vars: &[String]) -> code_ir::Expr {
    match expr {
        TcoPlanExpr::If {
            cond,
            then_body,
            else_body,
        } => code_ir::Expr::If {
            cond: cond.clone(),
            then_body: lower_tco_plan_stmts(then_body, loop_vars),
            else_body: else_body
                .as_ref()
                .map(|body| lower_tco_plan_stmts(body, loop_vars)),
        },
        TcoPlanExpr::Match { expr, arms } => code_ir::Expr::Match {
            expr: expr.clone(),
            arms: arms
                .iter()
                .map(|arm| code_ir::MatchArm {
                    pattern: arm.pattern.clone(),
                    body: lower_tco_plan_stmts(&arm.body, loop_vars),
                })
                .collect(),
        },
        TcoPlanExpr::Block(stmts) => code_ir::Expr::Block(lower_tco_plan_stmts(stmts, loop_vars)),
    }
}

fn lower_tco_recur(args: &[code_ir::Expr], loop_vars: &[String]) -> code_ir::Stmt {
    let mut stmts = Vec::with_capacity(loop_vars.len() * 2 + 1);
    let temps: Vec<String> = (0..loop_vars.len()).map(|i| format!("__tco_{i}")).collect();

    for (i, arg) in args.iter().enumerate() {
        if i < loop_vars.len() {
            stmts.push(code_ir::Stmt::Let {
                name: temps[i].clone(),
                mutable: false,
                expr: arg.clone(),
                ir_type: None,
            });
        }
    }

    for (i, loop_var) in loop_vars.iter().enumerate() {
        if i < args.len() {
            stmts.push(code_ir::Stmt::Assign {
                dest: code_ir::Expr::Var(loop_var.clone()),
                value: code_ir::Expr::Var(temps[i].clone()),
            });
        }
    }

    stmts.push(code_ir::Stmt::Continue);
    code_ir::Stmt::BlockScope(stmts)
}

fn ensure_expr_has_no_self_call(
    expr: &code_ir::Expr,
    fn_name: &str,
    context: &'static str,
) -> Result<(), TcoBlocker> {
    if expr_has_self_call(expr, fn_name) {
        Err(TcoBlocker::NonTailSelfCall(context))
    } else {
        Ok(())
    }
}

fn ensure_lowered_body_has_no_self_call(
    stmts: &[code_ir::Stmt],
    fn_name: &str,
) -> Result<(), TcoBlocker> {
    if body_has_self_call(stmts, fn_name) {
        Err(TcoBlocker::ResidualSelfCallAfterLowering)
    } else {
        Ok(())
    }
}

fn stmt_has_self_call(stmt: &code_ir::Stmt, fn_name: &str) -> bool {
    match stmt {
        code_ir::Stmt::Let { expr, .. } => expr_has_self_call(expr, fn_name),
        code_ir::Stmt::Bind { expr, .. } => expr_has_self_call(expr, fn_name),
        code_ir::Stmt::Assign { dest, value } => {
            expr_has_self_call(dest, fn_name) || expr_has_self_call(value, fn_name)
        }
        code_ir::Stmt::Expr(expr)
        | code_ir::Stmt::Return(expr)
        | code_ir::Stmt::TailExpr(expr)
        | code_ir::Stmt::Break(expr) => expr_has_self_call(expr, fn_name),
        code_ir::Stmt::Assert(assert) => assert_has_self_call(assert, fn_name),
        code_ir::Stmt::For { iter, body, .. } => {
            expr_has_self_call(iter, fn_name) || body_has_self_call(body, fn_name)
        }
        code_ir::Stmt::BlockScope(stmts) | code_ir::Stmt::Loop { body: stmts } => {
            body_has_self_call(stmts, fn_name)
        }
        code_ir::Stmt::Item(item) => item_has_self_call(item, fn_name),
        code_ir::Stmt::Comment(_) | code_ir::Stmt::Blank | code_ir::Stmt::Continue => false,
    }
}

fn expr_has_self_call(expr: &code_ir::Expr, fn_name: &str) -> bool {
    match expr {
        code_ir::Expr::Call { func, args, .. } => {
            is_self_call_by_func(func, fn_name)
                || expr_has_self_call(func, fn_name)
                || args.iter().any(|arg| expr_has_self_call(arg, fn_name))
        }
        code_ir::Expr::If {
            cond,
            then_body,
            else_body,
        } => {
            expr_has_self_call(cond, fn_name)
                || body_has_self_call(then_body, fn_name)
                || else_body
                    .as_ref()
                    .is_some_and(|stmts| body_has_self_call(stmts, fn_name))
        }
        code_ir::Expr::Match {
            expr: scrutinee,
            arms,
        } => {
            expr_has_self_call(scrutinee, fn_name)
                || arms
                    .iter()
                    .any(|arm| body_has_self_call(&arm.body, fn_name))
        }
        code_ir::Expr::Block(stmts) => body_has_self_call(stmts, fn_name),
        code_ir::Expr::MethodCall { receiver, args, .. } => {
            expr_has_self_call(receiver, fn_name)
                || args.iter().any(|arg| expr_has_self_call(arg, fn_name))
        }
        code_ir::Expr::Field(inner, _)
        | code_ir::Expr::Deref(inner)
        | code_ir::Expr::Ref(inner)
        | code_ir::Expr::RefMut(inner) => expr_has_self_call(inner, fn_name),
        code_ir::Expr::BinOp { left, right, .. } => {
            expr_has_self_call(left, fn_name) || expr_has_self_call(right, fn_name)
        }
        code_ir::Expr::UnaryOp { expr: inner, .. } => expr_has_self_call(inner, fn_name),
        code_ir::Expr::Struct { fields, rest, .. } => {
            fields.iter().any(|(_, v)| expr_has_self_call(v, fn_name))
                || rest
                    .as_ref()
                    .is_some_and(|inner| expr_has_self_call(inner, fn_name))
        }
        code_ir::Expr::Closure { .. } => false,
        code_ir::Expr::FormatStr { args, .. }
        | code_ir::Expr::MacroCall { args, .. }
        | code_ir::Expr::Tuple(args)
        | code_ir::Expr::Array(args) => args.iter().any(|arg| expr_has_self_call(arg, fn_name)),
        code_ir::Expr::Value(_)
        | code_ir::Expr::Var(_)
        | code_ir::Expr::Str(_)
        | code_ir::Expr::Path(_)
        | code_ir::Expr::IntLit(_)
        | code_ir::Expr::BoolLit(_)
        | code_ir::Expr::RawCode(_) => false,
    }
}

fn body_has_self_call(stmts: &[code_ir::Stmt], fn_name: &str) -> bool {
    stmts.iter().any(|stmt| stmt_has_self_call(stmt, fn_name))
}

fn assert_has_self_call(assert: &code_ir::Assert, fn_name: &str) -> bool {
    match assert {
        code_ir::Assert::Eq { left, right, .. } => {
            expr_has_self_call(left, fn_name) || expr_has_self_call(right, fn_name)
        }
        code_ir::Assert::True { expr, .. } | code_ir::Assert::NonEmpty { expr, .. } => {
            expr_has_self_call(expr, fn_name)
        }
        code_ir::Assert::Contains { expr, .. } => expr_has_self_call(expr, fn_name),
    }
}

fn item_has_self_call(item: &code_ir::Item, _fn_name: &str) -> bool {
    match item {
        code_ir::Item::Fn(_)
        | code_ir::Item::Struct(_)
        | code_ir::Item::Enum(_)
        | code_ir::Item::Use(_)
        | code_ir::Item::Impl(_)
        | code_ir::Item::Raw(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_syntax::ast::{BinOp, Expr, FnBody, Literal, MatchArm, NodeStmt, Pattern, Stmt};

    fn empty_ctx() -> CompileContext {
        CompileContext::new()
    }

    fn ctx_with_data(names: &[&str]) -> CompileContext {
        let mut ctx = CompileContext::new();
        for n in names {
            ctx.data_names.insert(n.to_string());
        }
        ctx
    }

    fn annotate_expr_ir_type(
        ctx: &mut CompileContext,
        body: &FnBody,
        target: &Expr,
        ir_type: IrType,
    ) {
        let mut expr_identity = None;
        daglang_syntax::ast_utils::walk_stmts_with_expr_identities(
            &body.stmts,
            &mut |identity, candidate| {
                if std::ptr::eq(candidate, target) {
                    expr_identity = Some(identity);
                }
            },
        );
        ctx.expr_ir_types.insert(
            expr_identity.expect("expected walked expression identity"),
            ir_type,
        );
    }

    #[test]
    fn compile_literal_int() {
        let mut counter = 0usize;
        let ir = compile_expr(&Expr::Literal(Literal::Int(42)), &empty_ctx(), &mut counter);
        assert!(matches!(ir, code_ir::Expr::IntLit(42)));
    }

    #[test]
    fn compile_literal_bool() {
        let mut counter = 0usize;
        let ir = compile_expr(
            &Expr::Literal(Literal::Bool(true)),
            &empty_ctx(),
            &mut counter,
        );
        assert!(matches!(ir, code_ir::Expr::BoolLit(true)));
    }

    #[test]
    fn compile_literal_string() {
        let mut counter = 0usize;
        let ir = compile_expr(
            &Expr::Literal(Literal::String("hello".into())),
            &empty_ctx(),
            &mut counter,
        );
        // String literals compile to "hello".to_string() for v2 compatibility.
        match &ir {
            code_ir::Expr::MethodCall {
                receiver, method, ..
            } => {
                assert_eq!(method, "to_string");
                assert!(
                    matches!(receiver.as_ref(), code_ir::Expr::Str(s) if s == "hello"),
                    "expected Str(\"hello\") as receiver, got: {receiver:?}"
                );
            }
            other => panic!("expected MethodCall(.to_string()), got: {other:?}"),
        }
    }

    #[test]
    fn compile_field_access() {
        let mut counter = 0usize;
        let expr = Expr::FieldAccess(Box::new(Expr::Ident("block".into())), "start".into());
        let ir = compile_expr(&expr, &empty_ctx(), &mut counter);
        match ir {
            code_ir::Expr::Field(receiver, field) => {
                // compile_ident now wraps non-__ variables with .clone() (S76),
                // so the receiver is MethodCall(Var("block"), "clone").
                match *receiver {
                    code_ir::Expr::MethodCall {
                        ref receiver,
                        ref method,
                        ..
                    } => {
                        assert!(
                            matches!(receiver.as_ref(), code_ir::Expr::Var(ref n) if n == "block")
                        );
                        assert_eq!(method, "clone");
                    }
                    code_ir::Expr::Var(ref n) => assert_eq!(n, "block"),
                    ref other => panic!("expected Var or MethodCall(.clone()), got: {other:?}"),
                }
                assert_eq!(field, "start");
            }
            other => panic!("expected Field, got: {other:?}"),
        }
    }

    #[test]
    fn compile_boxed_field_access_derefs_recursive_wrapper() {
        let mut counter = 0usize;
        let mut ctx = CompileContext::new();
        ctx.ir_scope
            .insert("item".to_string(), IrType::Named("Node".to_string()));
        ctx.struct_field_ir_types.insert(
            "Node".to_string(),
            vec![(
                "return_type".to_string(),
                IrType::Optional(Box::new(IrType::Named("Node".to_string()))),
            )],
        );
        ctx.boxed_fields
            .insert(("Node".to_string(), "return_type".to_string()));

        let expr = Expr::FieldAccess(Box::new(Expr::Ident("item".into())), "return_type".into());
        let ir = compile_expr(&expr, &ctx, &mut counter);
        match ir {
            code_ir::Expr::Deref(inner) => match *inner {
                code_ir::Expr::Field(receiver, field) => {
                    assert_eq!(field, "return_type");
                    match *receiver {
                        code_ir::Expr::MethodCall {
                            ref receiver,
                            ref method,
                            ..
                        } => {
                            assert!(
                                matches!(receiver.as_ref(), code_ir::Expr::Var(ref n) if n == "item")
                            );
                            assert_eq!(method, "clone");
                        }
                        code_ir::Expr::Var(ref n) => assert_eq!(n, "item"),
                        ref other => {
                            panic!("expected Var or MethodCall(.clone()), got: {other:?}")
                        }
                    }
                }
                other => panic!("expected boxed field access, got: {other:?}"),
            },
            other => panic!("expected Deref(Field(..)), got: {other:?}"),
        }
    }

    #[test]
    fn compile_binop() {
        let mut counter = 0usize;
        let expr = Expr::BinOp(
            Box::new(Expr::Ident("a".into())),
            BinOp::Ge,
            Box::new(Expr::Literal(Literal::Int(10))),
        );
        let ir = compile_expr(&expr, &empty_ctx(), &mut counter);
        match ir {
            code_ir::Expr::BinOp { op, .. } => assert_eq!(op, ">="),
            other => panic!("expected BinOp, got: {other:?}"),
        }
    }

    #[test]
    fn compile_match_expression() {
        let mut counter = 0usize;
        let expr = Expr::Match(
            Box::new(Expr::Ident("w".into())),
            vec![
                MatchArm {
                    pattern: Pattern::Ident("ZeroWidth".into()),
                    guard: None,
                    body: Expr::Literal(Literal::Int(0)),
                },
                MatchArm {
                    pattern: Pattern::Ident("Narrow".into()),
                    guard: None,
                    body: Expr::Literal(Literal::Int(1)),
                },
            ],
        );
        let ir = compile_expr(&expr, &empty_ctx(), &mut counter);
        match ir {
            code_ir::Expr::Match { arms, .. } => {
                assert_eq!(arms.len(), 2);
                assert_eq!(arms[0].pattern, "ZeroWidth");
                assert_eq!(arms[1].pattern, "Narrow");
            }
            other => panic!("expected Match, got: {other:?}"),
        }
    }

    #[test]
    fn compile_if_else() {
        let mut counter = 0usize;
        let expr = Expr::If(
            Box::new(Expr::Ident("flag".into())),
            Box::new(Expr::Literal(Literal::Int(1))),
            Some(Box::new(Expr::Literal(Literal::Int(0)))),
        );
        let ir = compile_expr(&expr, &empty_ctx(), &mut counter);
        match ir {
            code_ir::Expr::If {
                then_body,
                else_body,
                ..
            } => {
                assert_eq!(then_body.len(), 1);
                assert!(else_body.is_some());
            }
            other => panic!("expected If, got: {other:?}"),
        }
    }

    #[test]
    fn compile_fn_body_let_and_return() {
        let body = FnBody {
            stmts: vec![
                Stmt::Let("x".into(), Expr::Literal(Literal::Int(1))),
                Stmt::Expr(Expr::Ident("x".into())),
            ],
        };
        let ir = compile_fn_body(&body, &empty_ctx());
        assert_eq!(ir.len(), 2);
        assert!(matches!(ir[0], code_ir::Stmt::Let { .. }));
        assert!(matches!(ir[1], code_ir::Stmt::TailExpr(_)));
    }

    #[test]
    fn compile_fn_body_tracks_node_binding_types_in_scope() {
        let mut ctx = CompileContext::new();
        ctx.struct_field_ir_types.insert(
            "Config".to_string(),
            vec![("count".to_string(), IrType::Int)],
        );

        let body = FnBody {
            stmts: vec![
                Stmt::Node(NodeStmt {
                    name: "cfg".into(),
                    expr: Expr::Record(Some("Config".into()), vec![]),
                    after: vec![],
                    when_guard: None,
                }),
                Stmt::Let(
                    "count".into(),
                    Expr::FieldAccess(Box::new(Expr::Ident("cfg".into())), "count".into()),
                ),
                Stmt::Expr(Expr::Ident("count".into())),
            ],
        };
        if let Stmt::Node(node_stmt) = &body.stmts[0] {
            annotate_expr_ir_type(
                &mut ctx,
                &body,
                &node_stmt.expr,
                IrType::Named("Config".into()),
            );
        }
        if let Stmt::Let(_, expr) = &body.stmts[1] {
            annotate_expr_ir_type(&mut ctx, &body, expr, IrType::Int);
        }

        let ir = compile_fn_body(&body, &ctx);
        match &ir[1] {
            code_ir::Stmt::Let {
                ir_type: Some(IrType::Int),
                ..
            } => {}
            other => panic!("expected Int-typed let after node binding, got: {other:?}"),
        }
    }

    #[test]
    fn compile_record_construction() {
        let mut counter = 0usize;
        let expr = Expr::Record(
            Some("Point".into()),
            vec![
                ("x".into(), Expr::Literal(Literal::Int(1))),
                ("y".into(), Expr::Literal(Literal::Int(2))),
            ],
        );
        let ir = compile_expr(&expr, &empty_ctx(), &mut counter);
        match ir {
            code_ir::Expr::Struct { name, fields, .. } => {
                assert_eq!(name, "Point");
                assert_eq!(fields.len(), 2);
            }
            other => panic!("expected Struct, got: {other:?}"),
        }
    }

    #[test]
    fn compile_record_fills_missing_optional_fields_only() {
        let mut counter = 0usize;
        let mut ctx = CompileContext::new();
        ctx.struct_field_types.insert(
            "Config".to_string(),
            [
                ("required".to_string(), "String".to_string()),
                ("optional".to_string(), "Option<String>".to_string()),
            ]
            .into_iter()
            .collect(),
        );
        ctx.optional_fields.insert(
            "Config".to_string(),
            ["optional".to_string()].into_iter().collect(),
        );

        let expr = Expr::Record(
            Some("Config".into()),
            vec![(
                "required".into(),
                Expr::Literal(Literal::String("ok".into())),
            )],
        );
        let ir = compile_expr(&expr, &ctx, &mut counter);
        match ir {
            code_ir::Expr::Struct { fields, .. } => {
                assert_eq!(fields.len(), 2);
                assert!(fields.iter().any(|(name, expr)| {
                    name == "optional"
                        && matches!(expr, code_ir::Expr::Path(parts) if parts == &["None"])
                }));
            }
            other => panic!("expected Struct, got: {other:?}"),
        }
    }

    #[test]
    fn compile_record_fills_missing_optional_fields_in_sorted_order() {
        let mut counter = 0usize;
        let mut ctx = CompileContext::new();
        ctx.struct_field_types.insert(
            "Config".to_string(),
            [
                ("required".to_string(), "String".to_string()),
                ("optional_b".to_string(), "Option<String>".to_string()),
                ("optional_a".to_string(), "Option<String>".to_string()),
            ]
            .into_iter()
            .collect(),
        );
        ctx.optional_fields.insert(
            "Config".to_string(),
            ["optional_b".to_string(), "optional_a".to_string()]
                .into_iter()
                .collect(),
        );

        let expr = Expr::Record(
            Some("Config".into()),
            vec![(
                "required".into(),
                Expr::Literal(Literal::String("ok".into())),
            )],
        );
        let ir = compile_expr(&expr, &ctx, &mut counter);
        match ir {
            code_ir::Expr::Struct { fields, .. } => {
                let names: Vec<_> = fields.into_iter().map(|(name, _)| name).collect();
                assert_eq!(names, vec!["required", "optional_a", "optional_b"]);
            }
            other => panic!("expected Struct, got: {other:?}"),
        }
    }

    #[test]
    fn compile_record_does_not_fabricate_missing_required_fields() {
        let mut counter = 0usize;
        let mut ctx = CompileContext::new();
        ctx.struct_field_types.insert(
            "Config".to_string(),
            [
                ("required".to_string(), "String".to_string()),
                ("optional".to_string(), "Option<String>".to_string()),
            ]
            .into_iter()
            .collect(),
        );
        ctx.optional_fields.insert(
            "Config".to_string(),
            ["optional".to_string()].into_iter().collect(),
        );

        let expr = Expr::Record(
            Some("Config".into()),
            vec![(
                "optional".into(),
                Expr::Literal(Literal::String("present".into())),
            )],
        );
        let ir = compile_expr(&expr, &ctx, &mut counter);
        match ir {
            code_ir::Expr::Struct { fields, .. } => {
                assert_eq!(fields.len(), 1);
                assert!(fields.iter().all(|(name, _)| name != "required"));
            }
            other => panic!("expected Struct, got: {other:?}"),
        }
    }

    #[test]
    fn anonymous_record_without_resolved_type_emits_compile_error() {
        let mut counter = 0usize;
        let expr = Expr::Record(None, vec![("value".into(), Expr::Literal(Literal::Int(1)))]);

        let ir = compile_expr(&expr, &empty_ctx(), &mut counter);
        match ir {
            code_ir::Expr::RawCode(code) => assert!(
                code.contains("compile_error!")
                    && code.contains("cannot resolve anonymous record type"),
                "expected unresolved-record compile_error!, got: {code}"
            ),
            other => panic!("expected compile_error! marker, got: {other:?}"),
        }
    }

    #[test]
    fn field_context_prefers_expected_struct_type_for_anonymous_record() {
        let mut counter = 0usize;
        let mut ctx = CompileContext::new();
        ctx.struct_field_types.insert(
            "Target".to_string(),
            [("config".to_string(), "Config".to_string())]
                .into_iter()
                .collect(),
        );
        ctx.struct_field_types.insert(
            "Config".to_string(),
            [
                ("required".to_string(), "String".to_string()),
                ("optional".to_string(), "Option<String>".to_string()),
            ]
            .into_iter()
            .collect(),
        );
        ctx.struct_field_types.insert(
            "Other".to_string(),
            [("required".to_string(), "String".to_string())]
                .into_iter()
                .collect(),
        );
        ctx.optional_fields.insert(
            "Config".to_string(),
            ["optional".to_string()].into_iter().collect(),
        );

        let expr = Expr::Record(
            Some("Target".into()),
            vec![(
                "config".into(),
                Expr::Record(
                    None,
                    vec![(
                        "required".into(),
                        Expr::Literal(Literal::String("ok".into())),
                    )],
                ),
            )],
        );

        let ir = compile_expr(&expr, &ctx, &mut counter);
        match ir {
            code_ir::Expr::Struct { fields, .. } => match &fields[0].1 {
                code_ir::Expr::Struct { name, fields, .. } => {
                    assert_eq!(name, "Config");
                    assert!(fields.iter().any(|(field_name, expr)| {
                        field_name == "optional"
                            && matches!(expr, code_ir::Expr::Path(parts) if parts == &["None"])
                    }));
                }
                other => panic!("expected nested Config struct, got: {other:?}"),
            },
            other => panic!("expected Struct, got: {other:?}"),
        }
    }

    #[test]
    fn return_record_prefers_declared_return_struct_type() {
        let mut ctx = CompileContext::new();
        ctx.current_return_type = Some("Target".to_string());
        ctx.struct_field_types.insert(
            "Target".to_string(),
            [
                ("required".to_string(), "String".to_string()),
                ("optional".to_string(), "Option<String>".to_string()),
            ]
            .into_iter()
            .collect(),
        );
        ctx.struct_field_types.insert(
            "Other".to_string(),
            [("required".to_string(), "String".to_string())]
                .into_iter()
                .collect(),
        );
        ctx.optional_fields.insert(
            "Target".to_string(),
            ["optional".to_string()].into_iter().collect(),
        );

        let body = FnBody {
            stmts: vec![Stmt::Return(vec![(
                "required".into(),
                Expr::Literal(Literal::String("ok".into())),
            )])],
        };

        let ir = compile_fn_body(&body, &ctx);
        match &ir[0] {
            code_ir::Stmt::TailExpr(code_ir::Expr::Struct { name, fields, .. }) => {
                assert_eq!(name, "Target");
                assert!(fields.iter().any(|(field_name, expr)| {
                    field_name == "optional"
                        && matches!(expr, code_ir::Expr::Path(parts) if parts == &["None"])
                }));
            }
            other => panic!("expected TailExpr(Struct), got: {other:?}"),
        }
    }

    #[test]
    fn with_intrinsic_prefers_base_struct_type_for_anonymous_update_record() {
        let mut ctx = CompileContext::new();
        ctx.struct_field_types.insert(
            "State".to_string(),
            [
                ("pos".to_string(), "Int".to_string()),
                ("kind".to_string(), "String".to_string()),
            ]
            .into_iter()
            .collect(),
        );
        ctx.struct_field_types.insert(
            "Position".to_string(),
            [("pos".to_string(), "Int".to_string())]
                .into_iter()
                .collect(),
        );
        ctx.ir_scope
            .insert("state".to_string(), IrType::Named("State".to_string()));

        let body = FnBody {
            stmts: vec![Stmt::Expr(Expr::Call(
                "with".into(),
                vec![
                    (None, Expr::Ident("state".into())),
                    (
                        None,
                        Expr::Record(None, vec![("pos".into(), Expr::Literal(Literal::Int(1)))]),
                    ),
                ],
            ))],
        };
        if let Stmt::Expr(Expr::Call(_, args)) = &body.stmts[0] {
            annotate_expr_ir_type(&mut ctx, &body, &args[0].1, IrType::Named("State".into()));
        }

        let ir = compile_fn_body(&body, &ctx);
        match &ir[0] {
            code_ir::Stmt::TailExpr(code_ir::Expr::Struct { name, rest, .. }) => {
                assert_eq!(name, "State");
                assert!(
                    rest.is_some(),
                    "expected struct update rest to be preserved"
                );
            }
            other => panic!("expected Struct update, got: {other:?}"),
        }
    }

    #[test]
    fn compile_block_updates_scope_for_inner_lets() {
        let mut ctx = CompileContext::new();
        ctx.struct_field_ir_types.insert(
            "Config".to_string(),
            vec![("count".to_string(), IrType::Int)],
        );

        let body = FnBody {
            stmts: vec![Stmt::Expr(Expr::Block(vec![
                Stmt::Let("cfg".into(), Expr::Record(Some("Config".into()), vec![])),
                Stmt::Let(
                    "count".into(),
                    Expr::FieldAccess(Box::new(Expr::Ident("cfg".into())), "count".into()),
                ),
                Stmt::Expr(Expr::Ident("count".into())),
            ]))],
        };
        if let Stmt::Expr(Expr::Block(stmts)) = &body.stmts[0] {
            if let Stmt::Let(_, expr) = &stmts[0] {
                annotate_expr_ir_type(&mut ctx, &body, expr, IrType::Named("Config".into()));
            }
            if let Stmt::Let(_, expr) = &stmts[1] {
                annotate_expr_ir_type(&mut ctx, &body, expr, IrType::Int);
            }
        }

        let ir = compile_fn_body(&body, &ctx);
        match &ir[0] {
            code_ir::Stmt::TailExpr(code_ir::Expr::Block(stmts)) => match &stmts[1] {
                code_ir::Stmt::Let {
                    ir_type: Some(IrType::Int),
                    ..
                } => {}
                other => panic!("expected Int-typed inner let, got: {other:?}"),
            },
            other => panic!("expected Block, got: {other:?}"),
        }
    }

    #[test]
    fn compile_data_table_ident_uses_screaming_snake_with_clone() {
        let mut counter = 0usize;
        let ctx = ctx_with_data(&["zero_width_blocks"]);
        let ir = compile_expr(&Expr::Ident("zero_width_blocks".into()), &ctx, &mut counter);
        match &ir {
            code_ir::Expr::MethodCall {
                receiver, method, ..
            } => {
                assert_eq!(method, "clone");
                assert!(
                    matches!(receiver.as_ref(), code_ir::Expr::Var(ref n) if n == "ZERO_WIDTH_BLOCKS")
                );
            }
            other => panic!("expected MethodCall(clone), got: {other:?}"),
        }
    }

    #[test]
    fn compile_data_map_ident_uses_screaming_snake_with_ref() {
        let mut counter = 0usize;
        let mut ctx = CompileContext::new();
        ctx.data_map_names.insert("single_punct".to_string());
        let ir = compile_expr(&Expr::Ident("single_punct".into()), &ctx, &mut counter);
        match &ir {
            code_ir::Expr::Ref(inner) => {
                assert!(
                    matches!(inner.as_ref(), code_ir::Expr::Var(ref n) if n == "SINGLE_PUNCT"),
                    "expected Ref(Var(SINGLE_PUNCT)), got Ref({inner:?})"
                );
            }
            other => panic!("expected Ref(Var(SINGLE_PUNCT)), got: {other:?}"),
        }
    }

    #[test]
    fn compile_null_ident_becomes_none() {
        let mut counter = 0usize;
        let ir = compile_expr(&Expr::Ident("null".into()), &empty_ctx(), &mut counter);
        assert!(matches!(ir, code_ir::Expr::Var(ref n) if n == "None"));
    }

    #[test]
    fn option_match_wraps_non_null_patterns_in_some() {
        let mut counter = 0usize;
        let expr = Expr::Match(
            Box::new(Expr::Ident("color".into())),
            vec![
                MatchArm {
                    pattern: Pattern::Ident("null".into()),
                    guard: None,
                    body: Expr::Literal(Literal::Int(0)),
                },
                MatchArm {
                    pattern: Pattern::Ident("c".into()),
                    guard: None,
                    body: Expr::Literal(Literal::Int(1)),
                },
            ],
        );
        let ir = compile_expr(&expr, &empty_ctx(), &mut counter);
        match ir {
            code_ir::Expr::Match { arms, .. } => {
                assert_eq!(arms[0].pattern, "None");
                assert_eq!(arms[1].pattern, "Some(c)");
            }
            other => panic!("expected Match, got: {other:?}"),
        }
    }

    #[test]
    fn field_context_resolves_ambiguous_variant() {
        let mut counter = 0usize;
        let mut ctx = CompileContext::new();
        ctx.struct_field_types.insert(
            "BoxConfig".to_string(),
            [("color".to_string(), "SemanticColor".to_string())]
                .into_iter()
                .collect(),
        );
        ctx.enum_variants.insert(
            "SemanticColor".to_string(),
            ["Info".to_string(), "Error".to_string()]
                .into_iter()
                .collect(),
        );
        ctx.enum_variants.insert(
            "SymbolId".to_string(),
            ["Info".to_string(), "Error".to_string()]
                .into_iter()
                .collect(),
        );
        let expr = Expr::Record(
            Some("BoxConfig".into()),
            vec![("color".into(), Expr::Ident("Info".into()))],
        );
        let ir = compile_expr(&expr, &ctx, &mut counter);
        match ir {
            code_ir::Expr::Struct { fields, .. } => {
                assert!(
                    matches!(&fields[0].1, code_ir::Expr::Path(parts) if parts == &["SemanticColor", "Info"]),
                    "expected SemanticColor::Info, got: {:?}",
                    fields[0].1
                );
            }
            other => panic!("expected Struct, got: {other:?}"),
        }
    }

    #[test]
    fn match_scrutinee_uses_ir_scope_for_variant_resolution() {
        let mut counter = 0usize;
        let mut ctx = CompileContext::new();
        ctx.ir_scope.insert(
            "value".to_string(),
            IrType::Named("LiteralValue".to_string()),
        );
        ctx.enum_variants.insert(
            "TokenKind".to_string(),
            ["LitStr".to_string()].into_iter().collect(),
        );
        ctx.enum_variants.insert(
            "LiteralValue".to_string(),
            ["LitStr".to_string()].into_iter().collect(),
        );
        ctx.variant_to_enum
            .insert("LitStr".to_string(), "TokenKind".to_string());

        let expr = Expr::Match(
            Box::new(Expr::Ident("value".into())),
            vec![
                MatchArm {
                    pattern: Pattern::Ident("LitStr".into()),
                    guard: None,
                    body: Expr::Literal(Literal::Int(1)),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Expr::Literal(Literal::Int(0)),
                },
            ],
        );

        let ir = compile_expr(&expr, &ctx, &mut counter);
        match ir {
            code_ir::Expr::Match { arms, .. } => {
                assert_eq!(arms[0].pattern, "LiteralValue::LitStr");
            }
            other => panic!("expected Match, got: {other:?}"),
        }
    }

    #[test]
    fn some_pattern_uses_inner_expected_type_for_variant_resolution() {
        let mut counter = 0usize;
        let mut ctx = CompileContext::new();
        ctx.ir_scope
            .insert("value".to_string(), IrType::Named("TokenKind".to_string()));
        ctx.enum_variants.insert(
            "TokenKind".to_string(),
            ["LitStr".to_string()].into_iter().collect(),
        );
        ctx.enum_variants.insert(
            "LiteralValue".to_string(),
            ["LitStr".to_string()].into_iter().collect(),
        );
        ctx.variant_to_enum
            .insert("LitStr".to_string(), "LiteralValue".to_string());

        let expr = Expr::Match(
            Box::new(Expr::Ident("value".into())),
            vec![
                MatchArm {
                    pattern: Pattern::Variant(
                        "Some".into(),
                        vec![("value".into(), Pattern::Ident("LitStr".into()))],
                    ),
                    guard: None,
                    body: Expr::Literal(Literal::Int(1)),
                },
                MatchArm {
                    pattern: Pattern::Wildcard,
                    guard: None,
                    body: Expr::Literal(Literal::Int(0)),
                },
            ],
        );

        let ir = compile_expr(&expr, &ctx, &mut counter);
        match ir {
            code_ir::Expr::Match { arms, .. } => {
                assert_eq!(arms[0].pattern, "Some(TokenKind::LitStr)");
            }
            other => panic!("expected Match, got: {other:?}"),
        }
    }

    #[test]
    fn let_bound_call_result_uses_declared_return_ir_type_for_option_match_patterns() {
        let mut ctx = CompileContext::new();
        ctx.enum_variants.insert(
            "TokenKind".to_string(),
            ["LitStr".to_string()].into_iter().collect(),
        );
        ctx.enum_variants.insert(
            "LiteralValue".to_string(),
            ["LitStr".to_string()].into_iter().collect(),
        );
        ctx.variant_to_enum
            .insert("LitStr".to_string(), "LiteralValue".to_string());
        ctx.fn_return_ir_types.insert(
            "peek_kind".to_string(),
            IrType::Optional(Box::new(IrType::Named("TokenKind".to_string()))),
        );

        let body = FnBody {
            stmts: vec![
                Stmt::Let("k".into(), Expr::Call("peek_kind".into(), vec![])),
                Stmt::Expr(Expr::Match(
                    Box::new(Expr::Ident("k".into())),
                    vec![
                        MatchArm {
                            pattern: Pattern::Variant(
                                "Some".into(),
                                vec![(
                                    "value".into(),
                                    Pattern::Variant(
                                        "LitStr".into(),
                                        vec![("value".into(), Pattern::Ident("key".into()))],
                                    ),
                                )],
                            ),
                            guard: None,
                            body: Expr::Literal(Literal::Int(1)),
                        },
                        MatchArm {
                            pattern: Pattern::Wildcard,
                            guard: None,
                            body: Expr::Literal(Literal::Int(0)),
                        },
                    ],
                )),
            ],
        };

        let ir = compile_fn_body(&body, &ctx);
        match &ir[0] {
            code_ir::Stmt::Let {
                ir_type: Some(IrType::Optional(inner)),
                ..
            } => assert!(matches!(inner.as_ref(), IrType::Named(name) if name == "TokenKind")),
            other => panic!("expected optional let binding type, got: {other:?}"),
        }
        match &ir[1] {
            code_ir::Stmt::TailExpr(code_ir::Expr::Match { arms, .. }) => {
                assert_eq!(
                    arms[0].pattern,
                    "Some(TokenKind::LitStr { value: key, .. })"
                );
            }
            other => panic!("expected tail match, got: {other:?}"),
        }
    }

    #[test]
    fn with_uses_base_scope_type_when_expr_metadata_is_missing() {
        let mut counter = 0usize;
        let mut ctx = CompileContext::new();
        ctx.ir_scope.insert(
            "state".to_string(),
            IrType::Named("ParserState".to_string()),
        );
        ctx.struct_field_types.insert(
            "ParserState".to_string(),
            [("pos".to_string(), "Int".to_string())]
                .into_iter()
                .collect(),
        );
        ctx.struct_field_ir_types.insert(
            "ParserState".to_string(),
            vec![("pos".to_string(), IrType::Int)],
        );

        let expr = Expr::Call(
            "with".into(),
            vec![
                (None, Expr::Ident("state".into())),
                (
                    None,
                    Expr::Record(None, vec![("pos".into(), Expr::Literal(Literal::Int(1)))]),
                ),
            ],
        );

        let ir = compile_expr(&expr, &ctx, &mut counter);
        match ir {
            code_ir::Expr::Struct {
                name,
                fields,
                rest: Some(rest),
                ..
            } => {
                assert_eq!(name, "ParserState");
                assert!(matches!(&fields[0].1, code_ir::Expr::IntLit(1)));
                let cloned_state = match rest.as_ref() {
                    code_ir::Expr::MethodCall {
                        receiver,
                        method,
                        args,
                    } if method == "clone" && args.is_empty() => match receiver.as_ref() {
                        code_ir::Expr::Var(name) => name == "state",
                        code_ir::Expr::MethodCall {
                            receiver,
                            method,
                            args,
                        } => {
                            method == "clone"
                                && args.is_empty()
                                && matches!(receiver.as_ref(), code_ir::Expr::Var(name) if name == "state")
                        }
                        _ => false,
                    },
                    _ => false,
                };
                assert!(
                    cloned_state,
                    "expected struct update to preserve cloned base, got: {rest:?}"
                );
            }
            other => panic!("expected struct update, got: {other:?}"),
        }
    }

    #[test]
    fn fold_uses_function_return_ir_type_for_empty_list_accumulator() {
        let mut counter = 0usize;
        let mut ctx = CompileContext::new();
        ctx.current_return_ir_type = Some(IrType::Generic("List".to_string(), vec![IrType::Str]));
        ctx.ir_scope.insert(
            "items".to_string(),
            IrType::Generic("List".to_string(), vec![IrType::Str]),
        );

        let expr = Expr::Call(
            "fold".into(),
            vec![
                (None, Expr::Ident("items".into())),
                (Some("init".into()), Expr::List(vec![])),
                (
                    Some("f".into()),
                    Expr::Lambda(
                        vec!["acc".into(), "item".into()],
                        Box::new(Expr::Ident("acc".into())),
                    ),
                ),
            ],
        );

        let ir = compile_expr(&expr, &ctx, &mut counter);
        match ir {
            code_ir::Expr::Block(stmts) => match &stmts[0] {
                code_ir::Stmt::Let {
                    ir_type: Some(IrType::Generic(name, args)),
                    ..
                } => {
                    assert_eq!(name, "List");
                    assert!(matches!(args.as_slice(), [IrType::Str]));
                }
                other => panic!("expected typed fold accumulator let, got: {other:?}"),
            },
            other => panic!("expected fold block, got: {other:?}"),
        }
    }

    #[test]
    fn fold_prefers_lambda_structure_over_function_return_type_for_empty_list_accumulator() {
        let mut counter = 0usize;
        let mut ctx = CompileContext::new();
        ctx.current_return_ir_type = Some(IrType::Generic(
            "List".to_string(),
            vec![IrType::Named("Diagnostic".to_string())],
        ));
        ctx.ir_scope.insert(
            "modules".to_string(),
            IrType::Generic(
                "List".to_string(),
                vec![IrType::Named("TypedModule".to_string())],
            ),
        );
        ctx.struct_field_ir_types.insert(
            "TypedModule".to_string(),
            vec![("type_env".to_string(), IrType::Named("TypeEnv".to_string()))],
        );
        ctx.struct_field_ir_types.insert(
            "TypeEnv".to_string(),
            vec![(
                "bindings".to_string(),
                IrType::Generic(
                    "List".to_string(),
                    vec![IrType::Named("TypeBinding".to_string())],
                ),
            )],
        );

        let expr = Expr::Call(
            "fold".into(),
            vec![
                (None, Expr::Ident("modules".into())),
                (Some("init".into()), Expr::List(vec![])),
                (
                    Some("f".into()),
                    Expr::Lambda(
                        vec!["acc".into(), "tm".into()],
                        Box::new(Expr::Call(
                            "concat".into(),
                            vec![
                                (None, Expr::Ident("acc".into())),
                                (
                                    None,
                                    Expr::FieldAccess(
                                        Box::new(Expr::FieldAccess(
                                            Box::new(Expr::Ident("tm".into())),
                                            "type_env".into(),
                                        )),
                                        "bindings".into(),
                                    ),
                                ),
                            ],
                        )),
                    ),
                ),
            ],
        );

        let ir = compile_expr(&expr, &ctx, &mut counter);
        match ir {
            code_ir::Expr::Block(stmts) => match &stmts[0] {
                code_ir::Stmt::Let {
                    ir_type: Some(IrType::Generic(name, args)),
                    ..
                } => {
                    assert_eq!(name, "List");
                    assert!(matches!(
                        args.as_slice(),
                        [IrType::Named(inner)] if inner == "TypeBinding"
                    ));
                }
                other => panic!("expected typed fold accumulator let, got: {other:?}"),
            },
            other => panic!("expected fold block, got: {other:?}"),
        }
    }

    #[test]
    fn call_arguments_use_parameter_type_for_ambiguous_variants() {
        let mut counter = 0usize;
        let mut ctx = CompileContext::new();
        ctx.enum_variants.insert(
            "TokenKind".to_string(),
            ["NullCoalesce".to_string()].into_iter().collect(),
        );
        ctx.enum_variants.insert(
            "BinOpKind".to_string(),
            ["NullCoalesce".to_string()].into_iter().collect(),
        );
        ctx.variant_to_enum
            .insert("NullCoalesce".to_string(), "BinOpKind".to_string());
        ctx.fn_param_types.insert(
            "emit".to_string(),
            vec![
                ("state".to_string(), "TokenizerState".to_string()),
                ("kind".to_string(), "TokenKind".to_string()),
                ("len".to_string(), "Int".to_string()),
            ],
        );

        let expr = Expr::Call(
            "emit".into(),
            vec![
                (None, Expr::Ident("state".into())),
                (None, Expr::Ident("NullCoalesce".into())),
                (None, Expr::Literal(Literal::Int(2))),
            ],
        );

        let ir = compile_expr(&expr, &ctx, &mut counter);
        match ir {
            code_ir::Expr::Call { args, .. } => {
                assert!(
                    matches!(&args[1], code_ir::Expr::Path(parts) if parts == &["TokenKind", "NullCoalesce"]),
                    "expected TokenKind::NullCoalesce, got: {:?}",
                    args[1]
                );
            }
            other => panic!("expected Call, got: {other:?}"),
        }
    }

    #[test]
    fn field_context_propagates_expected_type_through_match_bodies() {
        let mut counter = 0usize;
        let mut ctx = CompileContext::new();
        ctx.ir_scope.insert("ok".to_string(), IrType::Bool);
        ctx.struct_field_types.insert(
            "Token".to_string(),
            [("kind".to_string(), "TokenKind".to_string())]
                .into_iter()
                .collect(),
        );
        ctx.struct_field_types.insert(
            "LitInt".to_string(),
            [("value".to_string(), "Int".to_string())]
                .into_iter()
                .collect(),
        );
        ctx.enum_variants.insert(
            "TokenKind".to_string(),
            ["LitInt".to_string()].into_iter().collect(),
        );
        ctx.enum_variants.insert(
            "LiteralValue".to_string(),
            ["LitInt".to_string()].into_iter().collect(),
        );
        ctx.variant_to_enum
            .insert("LitInt".to_string(), "LiteralValue".to_string());

        let expr = Expr::Record(
            Some("Token".into()),
            vec![(
                "kind".into(),
                Expr::Match(
                    Box::new(Expr::Ident("ok".into())),
                    vec![
                        MatchArm {
                            pattern: Pattern::Literal(Literal::Bool(true)),
                            guard: None,
                            body: Expr::Record(
                                Some("LitInt".into()),
                                vec![("value".into(), Expr::Literal(Literal::Int(1)))],
                            ),
                        },
                        MatchArm {
                            pattern: Pattern::Wildcard,
                            guard: None,
                            body: Expr::Record(
                                Some("LitInt".into()),
                                vec![("value".into(), Expr::Literal(Literal::Int(0)))],
                            ),
                        },
                    ],
                ),
            )],
        );

        let ir = compile_expr(&expr, &ctx, &mut counter);
        match ir {
            code_ir::Expr::Struct { fields, .. } => match &fields[0].1 {
                code_ir::Expr::Match { arms, .. } => {
                    let first_arm_expr = match &arms[0].body[0] {
                        code_ir::Stmt::TailExpr(expr) => expr,
                        other => panic!("expected TailExpr, got: {other:?}"),
                    };
                    assert!(
                        matches!(first_arm_expr, code_ir::Expr::Struct { name, .. } if name == "TokenKind::LitInt"),
                        "expected TokenKind::LitInt in match arm, got: {:?}",
                        first_arm_expr
                    );
                }
                other => panic!("expected match field expr, got: {other:?}"),
            },
            other => panic!("expected Struct, got: {other:?}"),
        }
    }

    #[test]
    fn infer_type_from_arms_returns_none_on_tie() {
        let mut ctx = CompileContext::new();
        ctx.enum_variants.insert(
            "SemanticColor".to_string(),
            ["Info".to_string(), "Error".to_string()]
                .into_iter()
                .collect(),
        );
        ctx.enum_variants.insert(
            "SymbolId".to_string(),
            ["Info".to_string(), "Error".to_string()]
                .into_iter()
                .collect(),
        );

        let arms = vec![
            MatchArm {
                pattern: Pattern::Ident("Info".into()),
                guard: None,
                body: Expr::Literal(Literal::Int(1)),
            },
            MatchArm {
                pattern: Pattern::Ident("Error".into()),
                guard: None,
                body: Expr::Literal(Literal::Int(0)),
            },
        ];

        assert_eq!(infer_type_from_arms(&arms, &ctx), None);
    }

    #[test]
    fn infer_match_result_type_prefers_enum_covering_more_arm_bodies() {
        let mut ctx = CompileContext::new();
        ctx.enum_variants.insert(
            "TokenKind".to_string(),
            ["LitInt".to_string(), "Unknown".to_string()]
                .into_iter()
                .collect(),
        );
        ctx.enum_variants.insert(
            "LiteralValue".to_string(),
            ["LitInt".to_string()].into_iter().collect(),
        );

        let arms = vec![
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Expr::Record(
                    Some("LitInt".into()),
                    vec![("value".into(), Expr::Literal(Literal::Int(1)))],
                ),
            },
            MatchArm {
                pattern: Pattern::Wildcard,
                guard: None,
                body: Expr::Record(
                    Some("Unknown".into()),
                    vec![("char".into(), Expr::Literal(Literal::String("x".into())))],
                ),
            },
        ];

        assert_eq!(
            infer_match_result_type(&arms, &ctx),
            Some("TokenKind".to_string())
        );
    }

    #[test]
    fn infer_type_from_arms_prefers_unique_best_match() {
        let mut ctx = CompileContext::new();
        ctx.enum_variants.insert(
            "SemanticColor".to_string(),
            ["Info".to_string(), "Error".to_string()]
                .into_iter()
                .collect(),
        );
        ctx.enum_variants.insert(
            "SymbolId".to_string(),
            ["Info".to_string()].into_iter().collect(),
        );

        let arms = vec![
            MatchArm {
                pattern: Pattern::Ident("Info".into()),
                guard: None,
                body: Expr::Literal(Literal::Int(1)),
            },
            MatchArm {
                pattern: Pattern::Ident("Error".into()),
                guard: None,
                body: Expr::Literal(Literal::Int(0)),
            },
        ];

        assert_eq!(
            infer_type_from_arms(&arms, &ctx),
            Some("SemanticColor".to_string())
        );
    }

    #[test]
    fn service_call_lowers_to_function_call() {
        let body = FnBody {
            stmts: vec![Stmt::Expr(Expr::ServiceCall(
                vec!["Filesystem".into(), "read".into()],
                vec![(Some("path".into()), Expr::Ident("p".into()))],
            ))],
        };
        let ir = compile_fn_body(&body, &empty_ctx());
        match &ir[0] {
            code_ir::Stmt::TailExpr(code_ir::Expr::Call { func, args, .. }) => {
                assert!(
                    matches!(func.as_ref(), code_ir::Expr::Path(parts) if parts == &["v2_rt", "filesystem_read"]),
                    "expected v2_rt::filesystem_read, got: {func:?}"
                );
                assert_eq!(args.len(), 1);
            }
            other => panic!("expected Call in stmt, got: {other:?}"),
        }
    }

    #[test]
    fn map_intrinsic_preserves_optional_mapper_results() {
        let mut counter = 0usize;
        let expr = Expr::Call(
            "map".into(),
            vec![
                (None, Expr::Ident("items".into())),
                (
                    None,
                    Expr::Lambda(
                        vec!["item".into()],
                        Box::new(Expr::Call(
                            "parse_int".into(),
                            vec![(None, Expr::Ident("item".into()))],
                        )),
                    ),
                ),
            ],
        );
        let ir = compile_expr(&expr, &empty_ctx(), &mut counter);
        match ir {
            code_ir::Expr::Block(stmts) => match &stmts[1] {
                code_ir::Stmt::For { body, .. } => match &body[0] {
                    code_ir::Stmt::Expr(code_ir::Expr::MethodCall { method, args, .. }) => {
                        assert_eq!(method, "push");
                        assert_eq!(args.len(), 1);
                        assert!(
                            !matches!(args[0], code_ir::Expr::Match { .. }),
                            "map should preserve optional mapper results instead of rewriting to filter_map"
                        );
                    }
                    other => panic!("expected push call in for-body, got: {other:?}"),
                },
                other => panic!("expected For in map lowering, got: {other:?}"),
            },
            other => panic!("expected Block, got: {other:?}"),
        }
    }

    #[test]
    fn record_construction_rejects_optional_value_for_required_field() {
        let mut ctx = CompileContext::new();
        ctx.struct_field_types.insert(
            "Target".to_string(),
            [("required".to_string(), "String".to_string())]
                .into_iter()
                .collect(),
        );
        ctx.struct_field_ir_types.insert(
            "Source".to_string(),
            vec![("maybe".to_string(), IrType::Optional(Box::new(IrType::Str)))],
        );
        ctx.ir_scope
            .insert("src".to_string(), IrType::Named("Source".to_string()));

        let body = FnBody {
            stmts: vec![Stmt::Expr(Expr::Record(
                Some("Target".into()),
                vec![(
                    "required".into(),
                    Expr::FieldAccess(Box::new(Expr::Ident("src".into())), "maybe".into()),
                )],
            ))],
        };
        if let Stmt::Expr(Expr::Record(_, fields)) = &body.stmts[0] {
            annotate_expr_ir_type(
                &mut ctx,
                &body,
                &fields[0].1,
                IrType::Optional(Box::new(IrType::Str)),
            );
        }

        let ir = compile_fn_body(&body, &ctx);
        match &ir[0] {
            code_ir::Stmt::TailExpr(code_ir::Expr::Struct { fields, .. }) => match &fields[0].1 {
                code_ir::Expr::RawCode(code) => assert!(
                    code.contains("compile_error!")
                        && code.contains("Target.required")
                        && code.contains("optional value"),
                    "expected compile_error! marker, got: {code}"
                ),
                other => panic!("expected compile_error! marker, got: {other:?}"),
            },
            other => panic!("expected Struct, got: {other:?}"),
        }
    }

    #[test]
    fn return_record_rejects_optional_value_for_required_field() {
        let mut ctx = CompileContext::new();
        ctx.current_return_type = Some("Target".to_string());
        ctx.struct_field_types.insert(
            "Target".to_string(),
            [("required".to_string(), "String".to_string())]
                .into_iter()
                .collect(),
        );
        ctx.struct_field_ir_types.insert(
            "Source".to_string(),
            vec![("maybe".to_string(), IrType::Optional(Box::new(IrType::Str)))],
        );
        ctx.ir_scope
            .insert("src".to_string(), IrType::Named("Source".to_string()));

        let body = FnBody {
            stmts: vec![Stmt::Return(vec![(
                "required".into(),
                Expr::FieldAccess(Box::new(Expr::Ident("src".into())), "maybe".into()),
            )])],
        };
        if let Stmt::Return(fields) = &body.stmts[0] {
            annotate_expr_ir_type(
                &mut ctx,
                &body,
                &fields[0].1,
                IrType::Optional(Box::new(IrType::Str)),
            );
        }

        let ir = compile_fn_body(&body, &ctx);
        match &ir[0] {
            code_ir::Stmt::TailExpr(code_ir::Expr::Struct { fields, .. }) => match &fields[0].1 {
                code_ir::Expr::RawCode(code) => assert!(
                    code.contains("compile_error!") && code.contains("Target.required"),
                    "expected compile_error! marker, got: {code}"
                ),
                other => panic!("expected compile_error! marker, got: {other:?}"),
            },
            other => panic!("expected TailExpr(Struct), got: {other:?}"),
        }
    }

    #[test]
    fn unsupported_map_produces_compile_error() {
        let body_map = FnBody {
            stmts: vec![Stmt::Expr(Expr::Map(vec![]))],
        };
        let ir_map = compile_fn_body(&body_map, &empty_ctx());
        match &ir_map[0] {
            code_ir::Stmt::TailExpr(code_ir::Expr::RawCode(s))
            | code_ir::Stmt::Expr(code_ir::Expr::RawCode(s)) => {
                assert!(
                    s.contains("compile_error!"),
                    "Map: expected compile_error! marker, got: {s}"
                );
            }
            other => panic!("Map: expected RawCode, got: {other:?}"),
        }
    }

    #[test]
    fn empty_construct_detected_in_match_arms() {
        let stmts = vec![code_ir::Stmt::TailExpr(code_ir::Expr::Match {
            expr: Box::new(code_ir::Expr::Var("x".into())),
            arms: vec![code_ir::MatchArm {
                pattern: "A".into(),
                body: vec![code_ir::Stmt::TailExpr(code_ir::Expr::Struct {
                    name: String::new(),
                    fields: vec![("value".into(), code_ir::Expr::IntLit(1))],
                    rest: None,
                    field_types: None,
                })],
            }],
        })];
        assert!(body_has_empty_construct(&stmts));
    }

    // =====================================================================
    // Tail-Call Optimization (TCO) tests
    // =====================================================================

    /// Build a self-call expression: `fn_name(args...)`
    fn self_call(fn_name: &str, args: Vec<code_ir::Expr>) -> code_ir::Expr {
        code_ir::Expr::Call {
            func: Box::new(code_ir::Expr::Var(fn_name.to_string())),
            args,
            obligation: None,
        }
    }

    #[test]
    fn tco_detects_simple_tail_recursion() {
        // fn f(n) { return f(n - 1); }
        let body = vec![code_ir::Stmt::Return(self_call(
            "f",
            vec![code_ir::Expr::BinOp {
                left: Box::new(code_ir::Expr::Var("n".into())),
                op: "-".into(),
                right: Box::new(code_ir::Expr::IntLit(1)),
            }],
        ))];
        let result = apply_tco("f", &["n".into()], &body);
        assert!(result.is_some(), "simple tail recursion should be eligible");
        let transformed = result.unwrap();
        // Structure: let mut __tco_p_n = n; loop { let n = __tco_p_n.clone(); ... }
        assert_eq!(transformed.len(), 2, "should have 1 preamble let + 1 loop");
        assert!(
            matches!(&transformed[0], code_ir::Stmt::Let { name, mutable: true, .. } if name == "__tco_p_n"),
            "first stmt should be mutable let for loop var"
        );
        assert!(
            matches!(&transformed[1], code_ir::Stmt::Loop { .. }),
            "second stmt should be a loop"
        );
    }

    #[test]
    fn tco_rejects_non_tail_recursion() {
        // fn f(n) { let x = f(n - 1); return x + 1; }
        let body = vec![
            code_ir::Stmt::Let {
                name: "x".into(),
                mutable: false,
                expr: self_call(
                    "f",
                    vec![code_ir::Expr::BinOp {
                        left: Box::new(code_ir::Expr::Var("n".into())),
                        op: "-".into(),
                        right: Box::new(code_ir::Expr::IntLit(1)),
                    }],
                ),
                ir_type: None,
            },
            code_ir::Stmt::Return(code_ir::Expr::BinOp {
                left: Box::new(code_ir::Expr::Var("x".into())),
                op: "+".into(),
                right: Box::new(code_ir::Expr::IntLit(1)),
            }),
        ];
        let result = apply_tco("f", &["n".into()], &body);
        assert!(
            result.is_none(),
            "non-tail recursion should NOT be eligible"
        );
    }

    #[test]
    fn tco_rejects_mixed_tail_and_non_tail() {
        // fn f(n) { let x = f(n - 1); return f(x); }
        // First call is non-tail (result stored in x), second is tail.
        let body = vec![
            code_ir::Stmt::Let {
                name: "x".into(),
                mutable: false,
                expr: self_call(
                    "f",
                    vec![code_ir::Expr::BinOp {
                        left: Box::new(code_ir::Expr::Var("n".into())),
                        op: "-".into(),
                        right: Box::new(code_ir::Expr::IntLit(1)),
                    }],
                ),
                ir_type: None,
            },
            code_ir::Stmt::Return(self_call("f", vec![code_ir::Expr::Var("x".into())])),
        ];
        let result = apply_tco("f", &["n".into()], &body);
        assert!(
            result.is_none(),
            "mixed tail/non-tail should NOT be eligible"
        );
    }

    #[test]
    fn tco_skips_non_recursive_function() {
        // fn f(n) { return n + 1; }
        let body = vec![code_ir::Stmt::Return(code_ir::Expr::BinOp {
            left: Box::new(code_ir::Expr::Var("n".into())),
            op: "+".into(),
            right: Box::new(code_ir::Expr::IntLit(1)),
        })];
        let result = apply_tco("f", &["n".into()], &body);
        assert!(
            result.is_none(),
            "non-recursive function should return None"
        );
    }

    #[test]
    fn tco_does_not_optimize_mutual_recursion() {
        // fn f(n) { return g(n); }
        // Since g != f, there are no self-calls, so apply_tco returns None.
        let body = vec![code_ir::Stmt::Return(code_ir::Expr::Call {
            func: Box::new(code_ir::Expr::Var("g".into())),
            args: vec![code_ir::Expr::Var("n".into())],
            obligation: None,
        })];
        let result = apply_tco("f", &["n".into()], &body);
        assert!(
            result.is_none(),
            "mutual recursion (call to different fn) should return None"
        );
    }

    #[test]
    fn tco_tail_call_in_if_branches() {
        // fn f(state) {
        //   if cond { return f(new_state); }
        //   else { return state; }
        // }
        let body = vec![code_ir::Stmt::TailExpr(code_ir::Expr::If {
            cond: Box::new(code_ir::Expr::Var("cond".into())),
            then_body: vec![code_ir::Stmt::Return(self_call(
                "f",
                vec![code_ir::Expr::Var("new_state".into())],
            ))],
            else_body: Some(vec![code_ir::Stmt::Return(code_ir::Expr::Var(
                "state".into(),
            ))]),
        })];
        let result = apply_tco("f", &["state".into()], &body);
        assert!(
            result.is_some(),
            "tail calls inside if branches should be eligible"
        );

        // Verify the structure: the else branch should have break, then branch should have continue.
        let transformed = result.unwrap();
        let loop_body = match &transformed[1] {
            code_ir::Stmt::Loop { body } => body,
            other => panic!("expected Loop, got: {other:?}"),
        };
        // After the rebind stmt, there should be a Break(If{...}) or Expr(If{...}).
        match &loop_body[1] {
            code_ir::Stmt::Break(code_ir::Expr::If {
                then_body,
                else_body,
                ..
            }) => {
                // Then branch has a tail call → should have BlockScope with continue.
                assert!(
                    then_body.iter().any(|s| matches!(s, code_ir::Stmt::BlockScope(inner) if inner.iter().any(|s2| matches!(s2, code_ir::Stmt::Continue)))),
                    "then branch should contain continue for the tail call"
                );
                // Else branch has a non-recursive return → should have break.
                assert!(
                    else_body
                        .as_ref()
                        .unwrap()
                        .iter()
                        .any(|s| matches!(s, code_ir::Stmt::Break(_))),
                    "else branch should contain break for the non-recursive return"
                );
            }
            code_ir::Stmt::Expr(code_ir::Expr::If {
                then_body,
                else_body,
                ..
            }) => {
                assert!(
                    then_body.iter().any(|s| matches!(s, code_ir::Stmt::BlockScope(inner) if inner.iter().any(|s2| matches!(s2, code_ir::Stmt::Continue)))),
                    "then branch should contain continue for the tail call"
                );
                assert!(
                    else_body
                        .as_ref()
                        .unwrap()
                        .iter()
                        .any(|s| matches!(s, code_ir::Stmt::Break(_))),
                    "else branch should contain break for the non-recursive return"
                );
            }
            other => panic!("expected Break(If) or Expr(If), got: {other:?}"),
        }
    }

    #[test]
    fn tco_transform_generates_parameter_reassignment() {
        // fn f(a, b) { return f(b, a); }
        let body = vec![code_ir::Stmt::Return(self_call(
            "f",
            vec![
                code_ir::Expr::Var("b".into()),
                code_ir::Expr::Var("a".into()),
            ],
        ))];
        let result = apply_tco("f", &["a".into(), "b".into()], &body);
        assert!(result.is_some());
        let transformed = result.unwrap();

        // Structure:
        //   let mut __tco_p_a = a;
        //   let mut __tco_p_b = b;
        //   loop {
        //     let a = __tco_p_a.clone();
        //     let b = __tco_p_b.clone();
        //     { let __tco_0 = b; let __tco_1 = a; __tco_p_a = __tco_0; __tco_p_b = __tco_1; continue; }
        //   }
        assert_eq!(transformed.len(), 3, "2 preamble lets + 1 loop");
        let loop_body = match &transformed[2] {
            code_ir::Stmt::Loop { body } => body,
            other => panic!("expected Loop, got: {other:?}"),
        };

        // First 2 stmts are rebinds, then the transformed tail call.
        assert!(
            loop_body.len() >= 3,
            "loop body should have rebinds + transformed stmt"
        );
        let block = match &loop_body[2] {
            code_ir::Stmt::BlockScope(stmts) => stmts,
            other => panic!("expected BlockScope for tail call, got: {other:?}"),
        };

        // Should have: 2 let bindings + 2 assignments + 1 continue = 5 stmts
        assert_eq!(
            block.len(),
            5,
            "expected 5 stmts (2 temps + 2 assigns + continue), got {}",
            block.len()
        );
        assert!(
            matches!(&block[4], code_ir::Stmt::Continue),
            "last stmt should be Continue"
        );
    }

    #[test]
    fn tco_transform_non_recursive_return_becomes_break() {
        // fn f(n) { if n == 0 { return 0; } return f(n - 1); }
        let body = vec![
            code_ir::Stmt::Expr(code_ir::Expr::If {
                cond: Box::new(code_ir::Expr::BinOp {
                    left: Box::new(code_ir::Expr::Var("n".into())),
                    op: "==".into(),
                    right: Box::new(code_ir::Expr::IntLit(0)),
                }),
                then_body: vec![code_ir::Stmt::Return(code_ir::Expr::IntLit(0))],
                else_body: None,
            }),
            code_ir::Stmt::TailExpr(self_call(
                "f",
                vec![code_ir::Expr::BinOp {
                    left: Box::new(code_ir::Expr::Var("n".into())),
                    op: "-".into(),
                    right: Box::new(code_ir::Expr::IntLit(1)),
                }],
            )),
        ];
        let result = apply_tco("f", &["n".into()], &body);
        assert!(result.is_some());
        let transformed = result.unwrap();

        // Structure: let mut __tco_p_n = n; loop { let n = ...; <body> }
        let loop_body = match &transformed[1] {
            code_ir::Stmt::Loop { body } => body,
            other => panic!("expected Loop, got: {other:?}"),
        };

        // Skip rebind stmt (index 0), look at the if (index 1).
        match &loop_body[1] {
            code_ir::Stmt::Expr(code_ir::Expr::If { then_body, .. }) => {
                assert!(
                    matches!(
                        &then_body[0],
                        code_ir::Stmt::Break(code_ir::Expr::IntLit(0))
                    ),
                    "non-recursive return should become break, got: {:?}",
                    then_body[0]
                );
            }
            other => panic!("expected Expr(If), got: {other:?}"),
        }
    }

    #[test]
    fn tco_rejects_tail_expr_inside_non_tail_if_expression() {
        let body = vec![
            code_ir::Stmt::Expr(code_ir::Expr::If {
                cond: Box::new(code_ir::Expr::Var("cond".into())),
                then_body: vec![code_ir::Stmt::TailExpr(self_call(
                    "f",
                    vec![code_ir::Expr::BinOp {
                        left: Box::new(code_ir::Expr::Var("n".into())),
                        op: "-".into(),
                        right: Box::new(code_ir::Expr::IntLit(1)),
                    }],
                ))],
                else_body: Some(vec![code_ir::Stmt::TailExpr(code_ir::Expr::IntLit(0))]),
            }),
            code_ir::Stmt::Return(code_ir::Expr::Var("n".into())),
        ];

        let result = apply_tco("f", &["n".into()], &body);
        assert!(
            result.is_none(),
            "tail expressions nested under a non-tail if-expression must block TCO"
        );
    }

    #[test]
    fn tco_rejects_self_call_inside_assert() {
        let body = vec![
            code_ir::Stmt::Assert(code_ir::Assert::True {
                expr: self_call("f", vec![code_ir::Expr::Var("n".into())]),
                message: "recursive assert".into(),
            }),
            code_ir::Stmt::Return(code_ir::Expr::Var("n".into())),
        ];

        let result = apply_tco("f", &["n".into()], &body);
        assert!(
            result.is_none(),
            "self-calls used in assertions must block TCO"
        );
    }

    #[test]
    fn tco_rejects_tail_expr_inside_final_block_scope() {
        let body = vec![code_ir::Stmt::BlockScope(vec![code_ir::Stmt::TailExpr(
            self_call("f", vec![code_ir::Expr::Var("n".into())]),
        )])];

        let result = apply_tco("f", &["n".into()], &body);
        assert!(
            result.is_none(),
            "lexical block statements must not inherit function tail position"
        );
    }

    #[test]
    fn tco_transform_rejects_self_call_inside_nested_loop() {
        let body = vec![code_ir::Stmt::Loop {
            body: vec![code_ir::Stmt::Return(self_call(
                "f",
                vec![code_ir::Expr::Var("n".into())],
            ))],
        }];

        let result = apply_tco("f", &["n".into()], &body);
        assert!(
            result.is_none(),
            "nested loop bodies containing self-calls must fail TCO rewriting"
        );
    }
}
