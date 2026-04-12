// v2_interpreter.rs — Tree-walking interpreter for .dag programs.
// Hand-written infrastructure (same category as parser, tokenizer, v2_rt).
// I-1: pure evaluation. I-2: shell service dispatch.

use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use crate::v2_rt;
use crate::v2_std_core::{
    Node, NewlineIndex, SourceSpan, ErrorNode,
    ExprData, MatchPattern, Connective, Cardinality,
    FieldAccessStyle, FieldValueShape, FieldSummary,
    MethodSemantics, VarBindingKind, CallSemantics,
    UnaryOpKind, StringPart,
    // Accessor functions
    authored_name_at, expr_var_name_at, expr_call_func_at,
    arg_name_at, arg_value,
    match_scrutinee, match_arm_nodes, arm_pattern, arm_body,
    if_condition, if_then_branch, if_else_branch,
    let_value, let_body, let_binding_name_at,
    field_access_base, field_access_field_at, expr_field_access_summary,
    binop_left, binop_right, unaryop_operand,
    block_stmts,
    method_receiver, method_arg_nodes, expr_method_name_at, expr_method_call_semantics,
    lambda_body, lambda_param_names_at,
    record_lit_type_name_at, field_init_node_name_at, field_init_node_value,
    cast_expr, cast_target,
    foreach_collection, foreach_body, foreach_variable_at,
    index_base, index_expr,
    slice_base, slice_start, slice_end,
    return_value,
    field_binding_name_at, field_binding_pattern,
    is_shell_transport, is_rest_transport, param_node_name_at,
    find_property, find_property_string,
};
use crate::v2_compiler_emit::{
    extract_string_interp_parts, has_mock_prefix,
};
use crate::std_syntax::BinOp;
use crate::std_syntax::LiteralValue;
use crate::v2_compiler_infer_items::{
    ResolvedGraph, TypedModule, ItemInfo, ItemKind,
};

// ---------------------------------------------------------------------------
// Value
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Rc<Vec<Value>>),
    Map(Rc<HashMap<String, Value>>),
    Record {
        type_name: String,
        fields: Rc<HashMap<String, Value>>,
    },
    Variant {
        type_name: String,
        variant_name: String,
        fields: Rc<HashMap<String, Value>>,
    },
    Closure {
        params: Vec<String>,
        body: Rc<Node>,
        env: Rc<Env>,
    },
    Unit,
}

impl Value {
    fn type_label(&self) -> &'static str {
        match self {
            Value::Null => "Null",
            Value::Bool(_) => "Bool",
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Str(_) => "String",
            Value::List(_) => "List",
            Value::Map(_) => "Map",
            Value::Record { .. } => "Record",
            Value::Variant { .. } => "Variant",
            Value::Closure { .. } => "Closure",
            Value::Unit => "Unit",
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Null => false,
            _ => true,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Map(entries) => {
                write!(f, "{{")?;
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Record { type_name, fields } => {
                write!(f, "{} {{", type_name)?;
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 { write!(f, ",")?; }
                    write!(f, " {}: {}", k, v)?;
                }
                write!(f, " }}")
            }
            Value::Variant { type_name: _, variant_name, fields } => {
                if fields.is_empty() {
                    write!(f, "{}", variant_name)
                } else {
                    write!(f, "{} {{", variant_name)?;
                    for (i, (k, v)) in fields.iter().enumerate() {
                        if i > 0 { write!(f, ",")?; }
                        write!(f, " {}: {}", k, v)?;
                    }
                    write!(f, " }}")
                }
            }
            Value::Closure { .. } => write!(f, "<closure>"),
            Value::Unit => write!(f, "()"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Variant { variant_name: a, fields: af, .. },
             Value::Variant { variant_name: b, fields: bf, .. }) => a == b && af == bf,
            (Value::Record { fields: af, .. },
             Value::Record { fields: bf, .. }) => af == bf,
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// Environment
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Env {
    bindings: HashMap<String, Value>,
    parent: Option<Rc<Env>>,
}

impl Env {
    pub fn empty() -> Rc<Self> {
        Rc::new(Env { bindings: HashMap::new(), parent: None })
    }

    pub fn extend(parent: &Rc<Env>, bindings: HashMap<String, Value>) -> Rc<Self> {
        Rc::new(Env { bindings, parent: Some(parent.clone()) })
    }

    pub fn with_binding(parent: &Rc<Env>, name: String, value: Value) -> Rc<Self> {
        let mut bindings = HashMap::new();
        bindings.insert(name, value);
        Rc::new(Env { bindings, parent: Some(parent.clone()) })
    }

    pub fn lookup(&self, name: &str) -> Option<&Value> {
        if let Some(v) = self.bindings.get(name) {
            Some(v)
        } else if let Some(ref parent) = self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum InterpError {
    NoSuchFunction { name: String },
    NoMainFunction,
    NoSuchVariable { name: String },
    NoSuchField { type_name: String, field: String },
    TypeError { msg: String },
    PatternMatchFailure { value: String },
    DivisionByZero,
    Unimplemented { what: String },
    EarlyReturn { value: Value },
}

impl fmt::Display for InterpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpError::NoSuchFunction { name } =>
                write!(f, "no such function: {}", name),
            InterpError::NoMainFunction =>
                write!(f, "no main function found"),
            InterpError::NoSuchVariable { name } =>
                write!(f, "undefined variable: {}", name),
            InterpError::NoSuchField { type_name, field } =>
                write!(f, "no field '{}' on type '{}'", field, type_name),
            InterpError::TypeError { msg } =>
                write!(f, "type error: {}", msg),
            InterpError::PatternMatchFailure { value } =>
                write!(f, "non-exhaustive pattern match on: {}", value),
            InterpError::DivisionByZero =>
                write!(f, "division by zero"),
            InterpError::Unimplemented { what } =>
                write!(f, "not yet implemented: {}", what),
            InterpError::EarlyReturn { .. } =>
                write!(f, "internal: uncaught early return"),
        }
    }
}

pub type InterpResult<T> = Result<T, InterpError>;

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

/// (service_node, operation_node) pair for service dispatch.
type ServiceOp = (Rc<Node>, Rc<Node>);

pub struct InterpContext {
    /// All typed modules from the compiler pipeline.
    pub modules: Rc<Vec<Rc<TypedModule>>>,
    /// Global item registry (function name → ItemInfo).
    pub item_registry: Rc<HashMap<String, Rc<ItemInfo>>>,
    /// Source indices for name resolution (authored_name_at).
    pub source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    /// Function bodies: name → Node.
    fn_nodes: HashMap<String, Rc<Node>>,
    /// Service registry: "service.Operation" → (service_node, op_node).
    service_ops: HashMap<String, ServiceOp>,
    /// Dry-run mode: use mock responses instead of executing services.
    pub dry_run: bool,
}

impl InterpContext {
    pub fn new(
        graph: &ResolvedGraph,
        source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
        dry_run: bool,
    ) -> Self {
        let mut fn_nodes = HashMap::new();
        let mut service_ops = HashMap::new();
        for module in graph.modules.iter() {
            for item in module.items.iter() {
                let name = authored_name_at(source_indices.clone(), item);
                if !name.is_empty() {
                    fn_nodes.insert(name.clone(), item.clone());
                }
                // Index service operations by checking ItemInfo kind
                if let Some(info) = graph.item_registry.get(&name) {
                    if info.kind == ItemKind::ServiceItem {
                        for op in item.children.iter() {
                            let op_name = authored_name_at(source_indices.clone(), op);
                            if !op_name.is_empty() {
                                let key = format!("{}.{}", name, op_name);
                                service_ops.insert(key, (item.clone(), op.clone()));
                            }
                        }
                    }
                }
                // Also index via item_registry name (which may differ from authored_name)
                if let Some(info) = graph.item_registry.get(&item.name) {
                    if info.kind == ItemKind::ServiceItem && !item.name.is_empty() {
                        for op in item.children.iter() {
                            let op_name = authored_name_at(source_indices.clone(), op);
                            if !op_name.is_empty() {
                                let key = format!("{}.{}", item.name, op_name);
                                service_ops.insert(key, (item.clone(), op.clone()));
                            }
                        }
                    }
                }
            }
        }
        InterpContext {
            modules: graph.modules.clone(),
            item_registry: graph.item_registry.clone(),
            source_indices,
            fn_nodes,
            service_ops,
            dry_run,
        }
    }

    fn si(&self) -> Rc<HashMap<String, Rc<NewlineIndex>>> {
        self.source_indices.clone()
    }

    fn lookup_fn(&self, name: &str) -> Option<&Rc<Node>> {
        self.fn_nodes.get(name)
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(
    graph: &ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    entry_fn: &str,
) -> InterpResult<Value> {
    run_with_options(graph, source_indices, entry_fn, false)
}

pub fn run_with_options(
    graph: &ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    entry_fn: &str,
    dry_run: bool,
) -> InterpResult<Value> {
    let ctx = InterpContext::new(graph, source_indices, dry_run);

    // Find the entry function
    let item_node = ctx.lookup_fn(entry_fn)
        .ok_or_else(|| InterpError::NoMainFunction)?
        .clone();

    // Build initial environment with data items
    let env = build_initial_env(&ctx)?;

    // Call the entry function with no arguments
    call_function(&ctx, &item_node, &[], &env)
}

/// Evaluate all `data` items to build the initial environment.
fn build_initial_env(ctx: &InterpContext) -> InterpResult<Rc<Env>> {
    let mut bindings = HashMap::new();
    for (name, info) in ctx.item_registry.iter() {
        if info.kind == ItemKind::DataItem {
            if let Some(node) = ctx.lookup_fn(name) {
                if let Some(ref body) = node.body {
                    let val = eval_expr(body, &Env::empty(), ctx)?;
                    bindings.insert(name.clone(), val);
                }
            }
        }
    }
    Ok(Env::extend(&Env::empty(), bindings))
}

// ---------------------------------------------------------------------------
// Function call
// ---------------------------------------------------------------------------

fn call_function(
    ctx: &InterpContext,
    fn_node: &Rc<Node>,
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
) -> InterpResult<Value> {
    let body = fn_node.body.as_ref()
        .ok_or_else(|| InterpError::TypeError {
            msg: format!("'{}' has no body", fn_node.name),
        })?;

    // Bind parameters
    let param_names: Vec<String> = fn_node.params.iter()
        .map(|p| authored_name_at(ctx.si(), p))
        .collect();

    let mut bindings = HashMap::new();
    if !args.is_empty() {
        // Named argument matching
        let mut positional_idx = 0;
        for (opt_name, val) in args {
            if let Some(name) = opt_name {
                bindings.insert(name.clone(), val.clone());
            } else if positional_idx < param_names.len() {
                bindings.insert(param_names[positional_idx].clone(), val.clone());
                positional_idx += 1;
            }
        }
    }

    let call_env = Env::extend(env, bindings);

    match eval_expr(body, &call_env, ctx) {
        Err(InterpError::EarlyReturn { value }) => Ok(value),
        other => other,
    }
}

// ---------------------------------------------------------------------------
// Expression evaluator
// ---------------------------------------------------------------------------

fn eval_expr(
    node: &Rc<Node>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    stacker::maybe_grow(64 * 1024, 2 * 1024 * 1024, || {
        eval_expr_inner(node, env, ctx)
    })
}

fn eval_expr_inner(
    node: &Rc<Node>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let si = ctx.si();
    match (*node.expr_data).clone() {
        ExprData::ExprLiteral { value } => eval_literal(&value),

        ExprData::ExprVar { binding_kind } => {
            eval_var(node, binding_kind.as_deref(), env, ctx)
        }

        ExprData::ExprBinOp { op, .. } => {
            let left = eval_expr(&binop_left(node.clone()), env, ctx)?;
            let right = eval_expr(&binop_right(node.clone()), env, ctx)?;
            eval_binop(&op, left, right)
        }

        ExprData::ExprUnaryOp { op } => {
            let operand = eval_expr(&unaryop_operand(node.clone()), env, ctx)?;
            eval_unaryop(&op, operand)
        }

        ExprData::ExprIf => eval_if(node, env, ctx),
        ExprData::ExprLet => eval_let(node, env, ctx),
        ExprData::ExprBlock => eval_block(node, env, ctx),
        ExprData::ExprMatch => eval_match(node, env, ctx),

        ExprData::ExprCall { .. } => eval_call(node, env, ctx),
        ExprData::ExprMethodCall { .. } => eval_method_call(node, env, ctx),

        ExprData::ExprFieldAccess { summary } => {
            eval_field_access(node, summary.as_deref(), env, ctx)
        }

        ExprData::ExprRecordLit { parent_enum } => {
            eval_record_lit(node, parent_enum.as_deref(), env, ctx)
        }

        ExprData::ExprListLit => {
            let items: Vec<Value> = node.children.iter()
                .map(|child| eval_expr(child, env, ctx))
                .collect::<InterpResult<_>>()?;
            Ok(Value::List(Rc::new(items)))
        }

        ExprData::ExprLambda { .. } => {
            let param_names: Vec<String> = lambda_param_names_at(node.clone(), si)
                .iter().cloned().collect();
            let body = lambda_body(node.clone());
            Ok(Value::Closure {
                params: param_names,
                body,
                env: env.clone(),
            })
        }

        ExprData::ExprStringInterp => eval_string_interp(node, env, ctx),
        ExprData::ExprCast => eval_cast(node, env, ctx),
        ExprData::ExprForEach => eval_for_each(node, env, ctx),
        ExprData::ExprIndex => eval_index(node, env, ctx),
        ExprData::ExprSlice => eval_slice(node, env, ctx),

        ExprData::ExprReturn => {
            let val = eval_expr(&return_value(node.clone()), env, ctx)?;
            Err(InterpError::EarlyReturn { value: val })
        }

        ExprData::ExprError { message, .. } => {
            Err(InterpError::TypeError { msg: message })
        }

        ExprData::NoExprData => Ok(Value::Unit),
    }
}

// ---------------------------------------------------------------------------
// Literals
// ---------------------------------------------------------------------------

fn eval_literal(lit: &LiteralValue) -> InterpResult<Value> {
    match lit {
        LiteralValue::LitBool { value } => Ok(Value::Bool(*value)),
        LiteralValue::LitInt { value } => Ok(Value::Int(*value)),
        LiteralValue::LitFloat { value } => {
            let f = value.parse::<f64>().map_err(|_| InterpError::TypeError {
                msg: format!("invalid float literal: {}", value),
            })?;
            Ok(Value::Float(f))
        }
        LiteralValue::LitStr { value } => Ok(Value::Str(value.clone())),
        LiteralValue::LitNull => Ok(Value::Null),
    }
}

// ---------------------------------------------------------------------------
// Variables
// ---------------------------------------------------------------------------

fn eval_var(
    node: &Rc<Node>,
    binding_kind: Option<&VarBindingKind>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let name = expr_var_name_at(node.clone(), ctx.si());

    // Special keywords
    if name == "none" || name == "None" {
        return Ok(Value::Null);
    }
    if name == "true" {
        return Ok(Value::Bool(true));
    }
    if name == "false" {
        return Ok(Value::Bool(false));
    }

    // Variant constructor (unit variant, no fields)
    if let Some(VarBindingKind::VariantValueBinding { parent_enum }) = binding_kind {
        return Ok(Value::Variant {
            type_name: parent_enum.clone(),
            variant_name: name,
            fields: Rc::new(HashMap::new()),
        });
    }

    // Environment lookup
    if let Some(val) = env.lookup(&name) {
        return Ok(val.clone());
    }

    // Data item lookup (evaluated lazily if not in env)
    if let Some(info) = v2_rt::map_get(&ctx.item_registry, name.clone()) {
        if info.kind == ItemKind::DataItem {
            if let Some(fn_node) = ctx.lookup_fn(&name) {
                if let Some(ref body) = fn_node.body {
                    return eval_expr(body, env, ctx);
                }
            }
        }
    }

    Err(InterpError::NoSuchVariable { name })
}

// ---------------------------------------------------------------------------
// Binary operators
// ---------------------------------------------------------------------------

fn eval_binop(op: &BinOp, left: Value, right: Value) -> InterpResult<Value> {
    // NullCoalesce: short-circuit
    if matches!(op, BinOp::NullCoalesce) {
        return Ok(if matches!(left, Value::Null) { right } else { left });
    }

    // String concatenation
    if matches!(op, BinOp::Add) {
        if let (Value::Str(a), Value::Str(b)) = (&left, &right) {
            return Ok(Value::Str(format!("{}{}", a, b)));
        }
    }

    // List concatenation
    if matches!(op, BinOp::Add) {
        if let (Value::List(a), Value::List(b)) = (&left, &right) {
            let mut result: Vec<Value> = a.to_vec();
            result.extend(b.iter().cloned());
            return Ok(Value::List(Rc::new(result)));
        }
    }

    // Equality (works on all comparable types)
    if matches!(op, BinOp::Eq) {
        return Ok(Value::Bool(left == right));
    }
    if matches!(op, BinOp::Ne) {
        return Ok(Value::Bool(left != right));
    }

    // Boolean operators
    if matches!(op, BinOp::And) {
        return Ok(Value::Bool(left.is_truthy() && right.is_truthy()));
    }
    if matches!(op, BinOp::Or) {
        return Ok(Value::Bool(left.is_truthy() || right.is_truthy()));
    }

    // Arithmetic / comparison on numeric types
    match (&left, &right) {
        (Value::Int(a), Value::Int(b)) => eval_int_binop(op, *a, *b),
        (Value::Float(a), Value::Float(b)) => eval_float_binop(op, *a, *b),
        (Value::Int(a), Value::Float(b)) => eval_float_binop(op, *a as f64, *b),
        (Value::Float(a), Value::Int(b)) => eval_float_binop(op, *a, *b as f64),
        // String comparison
        (Value::Str(a), Value::Str(b)) => match op {
            BinOp::Lt => Ok(Value::Bool(a < b)),
            BinOp::Gt => Ok(Value::Bool(a > b)),
            BinOp::Le => Ok(Value::Bool(a <= b)),
            BinOp::Ge => Ok(Value::Bool(a >= b)),
            _ => Err(InterpError::TypeError {
                msg: format!("unsupported operator {:?} on String", op),
            }),
        },
        _ => Err(InterpError::TypeError {
            msg: format!("cannot apply {:?} to {} and {}", op, left.type_label(), right.type_label()),
        }),
    }
}

fn eval_int_binop(op: &BinOp, a: i64, b: i64) -> InterpResult<Value> {
    match op {
        BinOp::Add => Ok(Value::Int(a + b)),
        BinOp::Sub => Ok(Value::Int(a - b)),
        BinOp::Mul => Ok(Value::Int(a * b)),
        BinOp::Div => {
            if b == 0 { return Err(InterpError::DivisionByZero); }
            Ok(Value::Int(a / b))
        }
        BinOp::Mod => {
            if b == 0 { return Err(InterpError::DivisionByZero); }
            Ok(Value::Int(a % b))
        }
        BinOp::Lt => Ok(Value::Bool(a < b)),
        BinOp::Gt => Ok(Value::Bool(a > b)),
        BinOp::Le => Ok(Value::Bool(a <= b)),
        BinOp::Ge => Ok(Value::Bool(a >= b)),
        _ => Err(InterpError::TypeError {
            msg: format!("unsupported int operator {:?}", op),
        }),
    }
}

fn eval_float_binop(op: &BinOp, a: f64, b: f64) -> InterpResult<Value> {
    match op {
        BinOp::Add => Ok(Value::Float(a + b)),
        BinOp::Sub => Ok(Value::Float(a - b)),
        BinOp::Mul => Ok(Value::Float(a * b)),
        BinOp::Div => {
            if b == 0.0 { return Err(InterpError::DivisionByZero); }
            Ok(Value::Float(a / b))
        }
        BinOp::Mod => {
            if b == 0.0 { return Err(InterpError::DivisionByZero); }
            Ok(Value::Float(a % b))
        }
        BinOp::Lt => Ok(Value::Bool(a < b)),
        BinOp::Gt => Ok(Value::Bool(a > b)),
        BinOp::Le => Ok(Value::Bool(a <= b)),
        BinOp::Ge => Ok(Value::Bool(a >= b)),
        _ => Err(InterpError::TypeError {
            msg: format!("unsupported float operator {:?}", op),
        }),
    }
}

// ---------------------------------------------------------------------------
// Unary operators
// ---------------------------------------------------------------------------

fn eval_unaryop(op: &UnaryOpKind, val: Value) -> InterpResult<Value> {
    match op {
        UnaryOpKind::Not => Ok(Value::Bool(!val.is_truthy())),
        UnaryOpKind::Neg => match val {
            Value::Int(n) => Ok(Value::Int(-n)),
            Value::Float(n) => Ok(Value::Float(-n)),
            _ => Err(InterpError::TypeError {
                msg: format!("cannot negate {}", val.type_label()),
            }),
        },
    }
}

// ---------------------------------------------------------------------------
// If expression
// ---------------------------------------------------------------------------

fn eval_if(
    node: &Rc<Node>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let cond = eval_expr(&if_condition(node.clone()), env, ctx)?;
    if cond.is_truthy() {
        eval_expr(&if_then_branch(node.clone()), env, ctx)
    } else {
        match if_else_branch(node.clone()) {
            Some(else_branch) => eval_expr(&else_branch, env, ctx),
            None => Ok(Value::Unit),
        }
    }
}

// ---------------------------------------------------------------------------
// Let expression
// ---------------------------------------------------------------------------

fn eval_let(
    node: &Rc<Node>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let name = let_binding_name_at(node.clone(), ctx.si());
    let val = eval_expr(&let_value(node.clone()), env, ctx)?;
    let new_env = Env::with_binding(env, name, val);
    match let_body(node.clone()) {
        Some(body) => eval_expr(&body, &new_env, ctx),
        None => Ok(Value::Unit),
    }
}

// ---------------------------------------------------------------------------
// Block expression
// ---------------------------------------------------------------------------

fn eval_block(
    node: &Rc<Node>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let stmts = block_stmts(node.clone());
    let mut current_env = env.clone();
    let mut last_val = Value::Unit;

    for stmt in stmts.iter() {
        match (*stmt.expr_data).clone() {
            ExprData::ExprLet => {
                let name = let_binding_name_at(stmt.clone(), ctx.si());
                let val = eval_expr(&let_value(stmt.clone()), &current_env, ctx)?;
                current_env = Env::with_binding(&current_env, name, val.clone());
                last_val = val;
            }
            _ => {
                last_val = eval_expr(stmt, &current_env, ctx)?;
            }
        }
    }

    Ok(last_val)
}

// ---------------------------------------------------------------------------
// Match expression
// ---------------------------------------------------------------------------

fn eval_match(
    node: &Rc<Node>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let scrutinee_val = eval_expr(&match_scrutinee(node.clone()), env, ctx)?;
    let arms = match_arm_nodes(node.clone());

    for arm in arms.iter() {
        let pattern = arm_pattern(arm.clone());
        if let Some(bindings) = match_pattern(&pattern, &scrutinee_val, ctx) {
            let arm_env = Env::extend(env, bindings);
            return eval_expr(&arm_body(arm), &arm_env, ctx);
        }
    }

    Err(InterpError::PatternMatchFailure {
        value: format!("{}", scrutinee_val),
    })
}

fn match_pattern(
    pattern: &MatchPattern,
    value: &Value,
    ctx: &InterpContext,
) -> Option<HashMap<String, Value>> {
    match pattern {
        MatchPattern::Wildcard => Some(HashMap::new()),

        MatchPattern::Bind { name } => {
            let mut bindings = HashMap::new();
            bindings.insert(name.clone(), value.clone());
            Some(bindings)
        }

        MatchPattern::LitPattern { value: lit } => {
            let lit_val = eval_literal(lit).ok()?;
            if *value == lit_val { Some(HashMap::new()) } else { None }
        }

        MatchPattern::VariantPattern { name, parent_enum: _, field_bindings } => {
            match value {
                // Match on variant
                Value::Variant { variant_name, fields, .. } => {
                    if variant_name != name { return None; }
                    let mut bindings = HashMap::new();
                    for fb in field_bindings.iter() {
                        let field_name = field_binding_name_at(fb.clone(), ctx.source_indices.clone());
                        let fb_pat = field_binding_pattern(fb.clone());
                        let field_val = fields.get(&field_name).cloned().unwrap_or(Value::Null);
                        // Recursively match the field's binding pattern
                        let sub_bindings = match_pattern(&fb_pat, &field_val, ctx)?;
                        bindings.extend(sub_bindings);
                    }
                    Some(bindings)
                }
                // Match on Option: Some { value: x } pattern
                Value::Null if name == "None" || name == "none" => {
                    Some(HashMap::new())
                }
                _ if name == "Some" => {
                    if matches!(value, Value::Null) { return None; }
                    let mut bindings = HashMap::new();
                    for fb in field_bindings.iter() {
                        let fb_pat = field_binding_pattern(fb.clone());
                        let sub_bindings = match_pattern(&fb_pat, value, ctx)?;
                        bindings.extend(sub_bindings);
                    }
                    Some(bindings)
                }
                _ => None,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Function call (ExprCall)
// ---------------------------------------------------------------------------

fn eval_call(
    node: &Rc<Node>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let func_name = expr_call_func_at(node.clone(), ctx.si());
    let arg_nodes = &node.children;

    // Evaluate arguments
    let args: Vec<(Option<String>, Value)> = arg_nodes.iter()
        .map(|arg_node| {
            let name = arg_name_at(arg_node.clone(), ctx.si());
            let val = eval_expr(&arg_value(arg_node), env, ctx)?;
            Ok((name, val))
        })
        .collect::<InterpResult<_>>()?;

    // Check for built-in runtime functions
    if let Some(result) = eval_builtin(&func_name, &args, ctx)? {
        return Ok(result);
    }

    // Look up user-defined function
    let fn_node = ctx.lookup_fn(&func_name)
        .ok_or_else(|| InterpError::NoSuchFunction { name: func_name.clone() })?
        .clone();

    call_function(ctx, &fn_node, &args, env)
}

// ---------------------------------------------------------------------------
// Method call (ExprMethodCall)
// ---------------------------------------------------------------------------

fn eval_method_call(
    node: &Rc<Node>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let method_name = expr_method_name_at(node.clone(), ctx.si());
    let semantics = expr_method_call_semantics(node.clone());

    // Service calls: skip receiver evaluation (it's a service namespace, not a value).
    if let Some(MethodSemantics::ServiceMethodSemantics { service_name, .. }) = semantics.as_deref() {
        let extra_args = method_arg_nodes(node.clone());
        let args: Vec<Value> = extra_args.iter()
            .map(|a| eval_expr(&arg_value(a), env, ctx))
            .collect::<InterpResult<_>>()?;
        return eval_service_call(service_name, &method_name, &args, env, ctx);
    }

    // Non-service calls: evaluate receiver and args
    let receiver_val = eval_expr(&method_receiver(node.clone()), env, ctx)?;
    let extra_args = method_arg_nodes(node.clone());
    let args: Vec<Value> = extra_args.iter()
        .map(|a| eval_expr(&arg_value(a), env, ctx))
        .collect::<InterpResult<_>>()?;

    match semantics.as_deref() {
        Some(MethodSemantics::AlgebraMethodSemantics { method_def, .. }) => {
            let mn = authored_name_at(ctx.si(), method_def);
            eval_algebra_method(&mn, receiver_val, &args, env, ctx)
        }
        _ => {
            // Plain method — try as algebra by method name
            eval_algebra_method(&method_name, receiver_val, &args, env, ctx)
        }
    }
}

// ---------------------------------------------------------------------------
// Field access
// ---------------------------------------------------------------------------

fn eval_field_access(
    node: &Rc<Node>,
    summary: Option<&FieldSummary>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let base_val = eval_expr(&field_access_base(node.clone()), env, ctx)?;
    let field_name = field_access_field_at(node.clone(), ctx.si());

    let access_style = summary.map(|s| &s.access_style);

    match access_style {
        Some(FieldAccessStyle::TupleFirst) => {
            // Map entry: (key, value).first → key
            // Or list: first element
            match &base_val {
                Value::List(items) => Ok(items.first().cloned().unwrap_or(Value::Null)),
                _ => extract_field(&base_val, &field_name),
            }
        }
        Some(FieldAccessStyle::TupleSecond) => {
            match &base_val {
                Value::List(items) => Ok(items.get(1).cloned().unwrap_or(Value::Null)),
                _ => extract_field(&base_val, &field_name),
            }
        }
        Some(FieldAccessStyle::OptionalUnwrap) => {
            // .value on Optional — unwrap or return Null
            match &base_val {
                Value::Null => Ok(Value::Null),
                _ => Ok(base_val),
            }
        }
        Some(FieldAccessStyle::EnumAccessor) => {
            // Accessing a discriminant field on an enum value
            extract_field(&base_val, &field_name)
        }
        _ => extract_field(&base_val, &field_name),
    }
}

fn extract_field(value: &Value, field: &str) -> InterpResult<Value> {
    match value {
        Value::Record { type_name, fields } => {
            fields.get(field).cloned().ok_or_else(|| InterpError::NoSuchField {
                type_name: type_name.clone(),
                field: field.to_string(),
            })
        }
        Value::Variant { type_name, fields, .. } => {
            fields.get(field).cloned().ok_or_else(|| InterpError::NoSuchField {
                type_name: type_name.clone(),
                field: field.to_string(),
            })
        }
        Value::Map(m) => {
            Ok(m.get(field).cloned().unwrap_or(Value::Null))
        }
        _ => Err(InterpError::TypeError {
            msg: format!("cannot access field '{}' on {}", field, value.type_label()),
        }),
    }
}

// ---------------------------------------------------------------------------
// Record literal
// ---------------------------------------------------------------------------

fn eval_record_lit(
    node: &Rc<Node>,
    parent_enum: Option<&str>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let type_name = record_lit_type_name_at(node.clone(), ctx.si())
        .unwrap_or_default();

    let mut fields = HashMap::new();
    for child in node.children.iter() {
        let fname = field_init_node_name_at(child.clone(), ctx.si());
        let fval = eval_expr(&field_init_node_value(child), env, ctx)?;
        fields.insert(fname, fval);
    }

    if let Some(pe) = parent_enum {
        Ok(Value::Variant {
            type_name: pe.to_string(),
            variant_name: type_name,
            fields: Rc::new(fields),
        })
    } else {
        Ok(Value::Record {
            type_name,
            fields: Rc::new(fields),
        })
    }
}

// ---------------------------------------------------------------------------
// String interpolation
// ---------------------------------------------------------------------------

fn eval_string_interp(
    node: &Rc<Node>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let parts = extract_string_interp_parts(node.clone());
    let mut result = String::new();
    for part in parts.iter() {
        let part_ref: &StringPart = part.as_ref();
        match part_ref {
            StringPart::Text { value } => result.push_str(value.as_str()),
            StringPart::Interpolation { expr } => {
                let val = eval_expr(&expr, env, ctx)?;
                result.push_str(&format!("{}", val));
            }
        }
    }
    Ok(Value::Str(result))
}

// ---------------------------------------------------------------------------
// Cast
// ---------------------------------------------------------------------------

fn eval_cast(
    node: &Rc<Node>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let val = eval_expr(&cast_expr(node.clone()), env, ctx)?;
    let target_node = cast_target(node.clone());
    let target_name = authored_name_at(ctx.si(), &target_node);

    match (val, target_name.as_str()) {
        (Value::Int(n), "Float") => Ok(Value::Float(n as f64)),
        (Value::Float(n), "Int") => Ok(Value::Int(n as i64)),
        (Value::Int(n), "String") => Ok(Value::Str(n.to_string())),
        (Value::Float(n), "String") => Ok(Value::Str(n.to_string())),
        (Value::Bool(b), "String") => Ok(Value::Str(b.to_string())),
        (v, "String") => Ok(Value::Str(format!("{}", v))),
        (v, t) => Err(InterpError::TypeError {
            msg: format!("cannot cast {} to {}", v.type_label(), t),
        }),
    }
}

// ---------------------------------------------------------------------------
// ForEach
// ---------------------------------------------------------------------------

fn eval_for_each(
    node: &Rc<Node>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let var_name = foreach_variable_at(node.clone(), ctx.si());
    let collection = eval_expr(&foreach_collection(node.clone()), env, ctx)?;
    let body_node = foreach_body(node.clone());

    match collection {
        Value::List(items) => {
            let mut results = Vec::with_capacity(items.len());
            for item in items.iter() {
                let iter_env = Env::with_binding(env, var_name.clone(), item.clone());
                results.push(eval_expr(&body_node, &iter_env, ctx)?);
            }
            Ok(Value::List(Rc::new(results)))
        }
        _ => Err(InterpError::TypeError {
            msg: format!("foreach expects a list, got {}", collection.type_label()),
        }),
    }
}

// ---------------------------------------------------------------------------
// Index
// ---------------------------------------------------------------------------

fn eval_index(
    node: &Rc<Node>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let base = eval_expr(&index_base(node.clone()), env, ctx)?;
    let idx = eval_expr(&index_expr(node.clone()), env, ctx)?;

    match (&base, &idx) {
        (Value::List(items), Value::Int(i)) => {
            let i = *i as usize;
            Ok(items.get(i).cloned().unwrap_or(Value::Null))
        }
        (Value::Map(m), Value::Str(k)) => {
            Ok(m.get(k.as_str()).cloned().unwrap_or(Value::Null))
        }
        (Value::Str(s), Value::Int(i)) => {
            let i = *i as usize;
            Ok(s.chars().nth(i)
                .map(|c| Value::Str(c.to_string()))
                .unwrap_or(Value::Null))
        }
        _ => Err(InterpError::TypeError {
            msg: format!("cannot index {} with {}", base.type_label(), idx.type_label()),
        }),
    }
}

// ---------------------------------------------------------------------------
// Slice
// ---------------------------------------------------------------------------

fn eval_slice(
    node: &Rc<Node>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let base = eval_expr(&slice_base(node.clone()), env, ctx)?;
    let start = eval_expr(&slice_start(node.clone()), env, ctx)?;
    let end = eval_expr(&slice_end(node.clone()), env, ctx)?;

    match (&base, &start, &end) {
        (Value::List(items), Value::Int(s), Value::Int(e)) => {
            let s = *s as usize;
            let e = (*e as usize).min(items.len());
            Ok(Value::List(Rc::new(items[s..e].to_vec())))
        }
        (Value::Str(str_val), Value::Int(s), Value::Int(e)) => {
            let s = *s as usize;
            let e = *e as usize;
            let sliced: String = str_val.chars().skip(s).take(e.saturating_sub(s)).collect();
            Ok(Value::Str(sliced))
        }
        _ => Err(InterpError::TypeError {
            msg: format!("cannot slice {} with {}..{}", base.type_label(), start.type_label(), end.type_label()),
        }),
    }
}

// ---------------------------------------------------------------------------
// Algebra methods (collection operations)
// ---------------------------------------------------------------------------

fn eval_algebra_method(
    method: &str,
    receiver: Value,
    args: &[Value],
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    match method {
        "map" => list_method_with_closure("map", receiver, args, env, ctx, |items, f, env, ctx| {
            items.iter()
                .map(|item| apply_closure(f, &[item.clone()], env, ctx))
                .collect::<InterpResult<Vec<Value>>>()
                .map(|v| Value::List(Rc::new(v)))
        }),

        "filter" => list_method_with_closure("filter", receiver, args, env, ctx, |items, f, env, ctx| {
            let mut result = Vec::new();
            for item in items.iter() {
                let keep = apply_closure(f, &[item.clone()], env, ctx)?;
                if keep.is_truthy() {
                    result.push(item.clone());
                }
            }
            Ok(Value::List(Rc::new(result)))
        }),

        "fold" => {
            let items = expect_list(&receiver, "fold")?;
            let (init, f) = match args {
                [init, f] => (init.clone(), f),
                _ => return Err(InterpError::TypeError {
                    msg: "fold requires (init, f) arguments".to_string(),
                }),
            };
            let mut acc = init;
            for item in items.iter() {
                acc = apply_closure(f, &[acc, item.clone()], env, ctx)?;
            }
            Ok(acc)
        }

        "flat_map" => list_method_with_closure("flat_map", receiver, args, env, ctx, |items, f, env, ctx| {
            let mut result = Vec::new();
            for item in items.iter() {
                let mapped = apply_closure(f, &[item.clone()], env, ctx)?;
                match mapped {
                    Value::List(inner) => result.extend(inner.iter().cloned()),
                    _ => result.push(mapped),
                }
            }
            Ok(Value::List(Rc::new(result)))
        }),

        "any" => list_method_with_closure("any", receiver, args, env, ctx, |items, f, env, ctx| {
            for item in items.iter() {
                if apply_closure(f, &[item.clone()], env, ctx)?.is_truthy() {
                    return Ok(Value::Bool(true));
                }
            }
            Ok(Value::Bool(false))
        }),

        "all" => list_method_with_closure("all", receiver, args, env, ctx, |items, f, env, ctx| {
            for item in items.iter() {
                if !apply_closure(f, &[item.clone()], env, ctx)?.is_truthy() {
                    return Ok(Value::Bool(false));
                }
            }
            Ok(Value::Bool(true))
        }),

        "sort_by" => list_method_with_closure("sort_by", receiver, args, env, ctx, |items, f, env, ctx| {
            let mut keyed: Vec<(Value, Value)> = items.iter()
                .map(|item| {
                    let key = apply_closure(f, &[item.clone()], env, ctx)?;
                    Ok((key, item.clone()))
                })
                .collect::<InterpResult<_>>()?;
            keyed.sort_by(|(ka, _), (kb, _)| cmp_values(ka, kb));
            Ok(Value::List(Rc::new(keyed.into_iter().map(|(_, v)| v).collect())))
        }),

        "concat" | "append" | "push" => {
            match &receiver {
                Value::List(items) => {
                    let mut result = items.to_vec();
                    for arg in args {
                        match arg {
                            Value::List(other) => result.extend(other.iter().cloned()),
                            _ => result.push(arg.clone()),
                        }
                    }
                    Ok(Value::List(Rc::new(result)))
                }
                Value::Str(s) => {
                    let mut result = s.clone();
                    for arg in args {
                        result.push_str(&format!("{}", arg));
                    }
                    Ok(Value::Str(result))
                }
                _ => Err(InterpError::TypeError {
                    msg: format!("cannot concat on {}", receiver.type_label()),
                }),
            }
        }

        "length" | "count" | "size" => {
            match &receiver {
                Value::List(items) => Ok(Value::Int(items.len() as i64)),
                Value::Map(m) => Ok(Value::Int(m.len() as i64)),
                Value::Str(s) => Ok(Value::Int(s.chars().count() as i64)),
                _ => Err(InterpError::TypeError {
                    msg: format!("cannot get length of {}", receiver.type_label()),
                }),
            }
        }

        "first" => {
            let items = expect_list(&receiver, "first")?;
            Ok(items.first().cloned().unwrap_or(Value::Null))
        }

        "last" => {
            let items = expect_list(&receiver, "last")?;
            Ok(items.last().cloned().unwrap_or(Value::Null))
        }

        "reverse" => {
            let items = expect_list(&receiver, "reverse")?;
            let mut result = items.to_vec();
            result.reverse();
            Ok(Value::List(Rc::new(result)))
        }

        "skip" => {
            let items = expect_list(&receiver, "skip")?;
            let n = expect_int(args.first(), "skip")?;
            Ok(Value::List(Rc::new(items.iter().skip(n as usize).cloned().collect())))
        }

        "take" => {
            let items = expect_list(&receiver, "take")?;
            let n = expect_int(args.first(), "take")?;
            Ok(Value::List(Rc::new(items.iter().take(n as usize).cloned().collect())))
        }

        "enumerate" => {
            let items = expect_list(&receiver, "enumerate")?;
            let result: Vec<Value> = items.iter().enumerate()
                .map(|(i, v)| {
                    let mut fields = HashMap::new();
                    fields.insert("index".to_string(), Value::Int(i as i64));
                    fields.insert("value".to_string(), v.clone());
                    Value::Record { type_name: "Pair".to_string(), fields: Rc::new(fields) }
                })
                .collect();
            Ok(Value::List(Rc::new(result)))
        }

        "contains" | "has" => {
            match &receiver {
                Value::List(items) => {
                    let target = args.first().cloned().unwrap_or(Value::Null);
                    Ok(Value::Bool(items.iter().any(|item| *item == target)))
                }
                Value::Map(m) => {
                    let key = expect_str(args.first(), "contains")?;
                    Ok(Value::Bool(m.contains_key(&key)))
                }
                Value::Str(s) => {
                    let sub = expect_str(args.first(), "contains")?;
                    Ok(Value::Bool(s.contains(&sub)))
                }
                _ => Err(InterpError::TypeError {
                    msg: format!("contains not supported on {}", receiver.type_label()),
                }),
            }
        }

        "join" => {
            let items = expect_list(&receiver, "join")?;
            let sep = args.first()
                .map(|v| format!("{}", v))
                .unwrap_or_default();
            let strs: Vec<String> = items.iter().map(|v| format!("{}", v)).collect();
            Ok(Value::Str(strs.join(&sep)))
        }

        "get" => {
            match &receiver {
                Value::Map(m) => {
                    let key = expect_str(args.first(), "get")?;
                    Ok(m.get(&key).cloned().unwrap_or(Value::Null))
                }
                Value::List(items) => {
                    let idx = expect_int(args.first(), "get")?;
                    Ok(items.get(idx as usize).cloned().unwrap_or(Value::Null))
                }
                _ => Err(InterpError::TypeError {
                    msg: format!("get not supported on {}", receiver.type_label()),
                }),
            }
        }

        "insert" | "map_insert" => {
            let m = expect_map(&receiver, "insert")?;
            let (key, val) = match args {
                [k, v] => (format!("{}", k), v.clone()),
                _ => return Err(InterpError::TypeError {
                    msg: "insert requires (key, value) arguments".to_string(),
                }),
            };
            let mut new_map = HashMap::clone(&m);
            new_map.insert(key, val);
            Ok(Value::Map(Rc::new(new_map)))
        }

        "merge" => {
            let base = expect_map(&receiver, "merge")?;
            let overlay = expect_map(args.first().unwrap_or(&Value::Null), "merge")?;
            let mut result = HashMap::clone(&base);
            for (k, v) in overlay.iter() {
                result.insert(k.clone(), v.clone());
            }
            Ok(Value::Map(Rc::new(result)))
        }

        "keys" => {
            let m = expect_map(&receiver, "keys")?;
            let keys: Vec<Value> = m.keys().map(|k| Value::Str(k.clone())).collect();
            Ok(Value::List(Rc::new(keys)))
        }

        "values" => {
            let m = expect_map(&receiver, "values")?;
            let vals: Vec<Value> = m.values().cloned().collect();
            Ok(Value::List(Rc::new(vals)))
        }

        // String-specific methods
        "replace" => {
            let s = expect_string(&receiver, "replace")?;
            match args {
                [from, to] => {
                    let from_s = format!("{}", from);
                    let to_s = format!("{}", to);
                    Ok(Value::Str(s.replace(&from_s, &to_s)))
                }
                _ => Err(InterpError::TypeError {
                    msg: "replace requires (from, to) arguments".to_string(),
                }),
            }
        }

        "split" => {
            let s = expect_string(&receiver, "split")?;
            let sep = expect_str(args.first(), "split")?;
            let parts: Vec<Value> = s.split(&sep).map(|p| Value::Str(p.to_string())).collect();
            Ok(Value::List(Rc::new(parts)))
        }

        "trim" => {
            let s = expect_string(&receiver, "trim")?;
            Ok(Value::Str(s.trim().to_string()))
        }

        "starts_with" => {
            let s = expect_string(&receiver, "starts_with")?;
            let prefix = expect_str(args.first(), "starts_with")?;
            Ok(Value::Bool(s.starts_with(&prefix)))
        }

        "ends_with" => {
            let s = expect_string(&receiver, "ends_with")?;
            let suffix = expect_str(args.first(), "ends_with")?;
            Ok(Value::Bool(s.ends_with(&suffix)))
        }

        "substring" => {
            let s = expect_string(&receiver, "substring")?;
            match args {
                [start, end] => {
                    let s_idx = expect_int(Some(start), "substring start")? as usize;
                    let e_idx = expect_int(Some(end), "substring end")? as usize;
                    let sliced: String = s.chars().skip(s_idx).take(e_idx.saturating_sub(s_idx)).collect();
                    Ok(Value::Str(sliced))
                }
                _ => Err(InterpError::TypeError {
                    msg: "substring requires (start, end) arguments".to_string(),
                }),
            }
        }

        "char_at" => {
            let s = expect_string(&receiver, "char_at")?;
            let idx = expect_int(args.first(), "char_at")?;
            Ok(s.chars().nth(idx as usize)
                .map(|c| Value::Str(c.to_string()))
                .unwrap_or(Value::Null))
        }

        "index_by" => list_method_with_closure("index_by", receiver, args, env, ctx, |items, f, env, ctx| {
            let mut m = HashMap::new();
            for item in items.iter() {
                let key = apply_closure(f, &[item.clone()], env, ctx)?;
                let key_str = format!("{}", key);
                m.insert(key_str, item.clone());
            }
            Ok(Value::Map(Rc::new(m)))
        }),

        _ => Err(InterpError::Unimplemented {
            what: format!("method '{}'", method),
        }),
    }
}

// ---------------------------------------------------------------------------
// Service dispatch (I-2)
// ---------------------------------------------------------------------------

fn eval_service_call(
    service_name: &str,
    op_name: &str,
    args: &[Value],
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let key = format!("{}.{}", service_name, op_name);
    let (service_node, op_node) = ctx.service_ops.get(&key)
        .ok_or_else(|| InterpError::Unimplemented {
            what: format!("unknown service operation: {}", key),
        })?;

    // Get effective transport (operation-level overrides service-level)
    let transport = op_node.transport.as_ref()
        .or(service_node.transport.as_ref())
        .ok_or_else(|| InterpError::TypeError {
            msg: format!("no transport for service {}", key),
        })?;

    // Bind input params to arg values
    let param_env = build_service_param_env(op_node, args, env, ctx)?;

    // Dry-run: return mock response
    if ctx.dry_run {
        eprintln!("[dry-run] {}.{}", service_name, op_name);
        return eval_mock_response(op_node, ctx);
    }

    // Shell transport dispatch
    if is_shell_transport(transport.clone()) {
        let result = dispatch_shell(transport, &param_env, ctx)?;
        return map_shell_outputs(&result, op_node, ctx);
    }

    // REST transport dispatch (any non-shell transport with service config endpoint)
    return dispatch_rest(service_node, op_node, transport, &param_env, ctx);
}

/// Build an environment with service operation params bound to arg values.
fn build_service_param_env(
    op_node: &Rc<Node>,
    args: &[Value],
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Rc<Env>> {
    let mut bindings = HashMap::new();
    for (i, param) in op_node.params.iter().enumerate() {
        let name = param_node_name_at(param.clone(), ctx.si());
        if let Some(val) = args.get(i) {
            bindings.insert(name, val.clone());
        }
    }
    Ok(Env::extend(env, bindings))
}

struct ShellResult {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

/// Execute a shell transport: evaluate argv template, run command, capture output.
fn dispatch_shell(
    transport: &Rc<Node>,
    param_env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<ShellResult> {
    // Evaluate argv elements as expressions
    let argv_nodes = &transport.children;
    let mut argv: Vec<String> = Vec::new();
    for node in argv_nodes.iter() {
        let val = eval_expr(node, param_env, ctx)?;
        argv.push(format!("{}", val));
    }

    if argv.is_empty() {
        return Err(InterpError::TypeError {
            msg: "shell transport has empty argv".to_string(),
        });
    }

    eprintln!("[shell] {}", argv.join(" "));

    let output = std::process::Command::new(&argv[0])
        .args(&argv[1..])
        .output()
        .map_err(|e| InterpError::TypeError {
            msg: format!("failed to execute '{}': {}", argv[0], e),
        })?;

    Ok(ShellResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).trim_end().to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim_end().to_string(),
    })
}

/// Map shell stdout/stderr/exit_code to the operation's return type fields.
fn map_shell_outputs(
    result: &ShellResult,
    op_node: &Rc<Node>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    // Get the return type's fields from the inferred type
    let return_type = match op_node.inferred.as_deref() {
        Some(crate::v2_std_core::InferredNode::Resolved { node }) => node.clone(),
        _ => {
            // No structured return type — return stdout as string
            return Ok(Value::Str(result.stdout.clone()));
        }
    };

    // Single-field or multi-field product type
    let children = &return_type.children;
    if children.is_empty() {
        return Ok(Value::Unit);
    }

    let mut fields = HashMap::new();
    for child in children.iter() {
        let field_name = authored_name_at(ctx.si(), child);
        // Check from_key property
        let from_key = extract_from_key(child, ctx);
        let value = match from_key.as_deref() {
            Some("stdout") => Value::Str(result.stdout.clone()),
            Some("stderr") => Value::Str(result.stderr.clone()),
            Some("exit_success") => Value::Bool(result.exit_code == 0),
            Some("stdout_lines") => {
                let lines: Vec<Value> = result.stdout.lines()
                    .map(|l| Value::Str(l.to_string()))
                    .collect();
                Value::List(Rc::new(lines))
            }
            _ => {
                // Default: map by field name
                match field_name.as_str() {
                    "success" => Value::Bool(result.exit_code == 0),
                    "stdout" => Value::Str(result.stdout.clone()),
                    "stderr" => Value::Str(result.stderr.clone()),
                    "exists" => Value::Bool(result.exit_code == 0),
                    _ => Value::Null,
                }
            }
        };
        fields.insert(field_name, value);
    }

    // Return as record with the type name
    Ok(Value::Record {
        type_name: authored_name_at(ctx.si(), op_node),
        fields: Rc::new(fields),
    })
}

/// Extract the `from` key from a field's properties (e.g., `from "stdout"`).
fn extract_from_key(field_node: &Rc<Node>, ctx: &InterpContext) -> Option<String> {
    for prop in field_node.properties.iter() {
        let prop_name = field_init_node_name_at(prop.clone(), ctx.si());
        if prop_name == "from_key" || prop_name == "from" {
            let val_node = field_init_node_value(prop);
            if let ExprData::ExprLiteral { ref value } = *val_node.expr_data {
                if let LiteralValue::LitStr { value: s } = value.as_ref() {
                    return Some(s.clone());
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// REST transport dispatch (I-3)
// ---------------------------------------------------------------------------

/// Execute a REST transport: build URL, set headers/auth, send request, parse response.
fn dispatch_rest(
    service_node: &Rc<Node>,
    op_node: &Rc<Node>,
    transport: &Rc<Node>,
    param_env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let si = ctx.si();

    // 1. Base URL from service config
    let base_url = find_service_config_string(service_node, "svc_endpoint", &si)
        .unwrap_or_default();

    // 2. Path template — evaluate as expression (handles string interpolation)
    // find_property returns the value node directly (already unwrapped).
    let path = match find_property(transport.properties.clone(), "path".to_string(), si.clone()) {
        Some(path_node) => {
            let path_val = eval_expr(&path_node, param_env, ctx)?;
            format!("{}", path_val)
        }
        None => String::new(),
    };

    let url = if path.is_empty() {
        base_url
    } else {
        format!("{}{}", base_url, path)
    };

    // 3. HTTP method — try string literal, fall back to authored name
    let method = match find_property(transport.properties.clone(), "method".to_string(), si.clone()) {
        Some(m_node) => {
            if let ExprData::ExprLiteral { ref value } = *m_node.expr_data {
                if let LiteralValue::LitStr { value: s } = value.as_ref() {
                    s.clone().to_uppercase()
                } else {
                    authored_name_at(si.clone(), &m_node).to_uppercase()
                }
            } else {
                authored_name_at(si.clone(), &m_node).to_uppercase()
            }
        }
        None => "GET".to_string(),
    };

    // 4. Auth header
    let (auth_header_name, auth_token) = resolve_auth(service_node, transport, &si);

    // 5. Custom headers from transport
    // find_property returns the headers record node; its children are field-init nodes.
    let mut headers: Vec<(String, String)> = Vec::new();
    if let Some(hdrs_record) = find_property(transport.properties.clone(), "headers".to_string(), si.clone()) {
        for child in hdrs_record.children.iter() {
            let hname = field_init_node_name_at(child.clone(), si.clone());
            let hval = eval_expr(&field_init_node_value(child), param_env, ctx)?;
            headers.push((hname, format!("{}", hval)));
        }
    }

    // 6. Query params
    let mut query_params: Vec<(String, String)> = Vec::new();
    if let Some(query_record) = find_property(transport.properties.clone(), "query".to_string(), si.clone()) {
        for child in query_record.children.iter() {
            let qname = field_init_node_name_at(child.clone(), si.clone());
            let qval = eval_expr(&field_init_node_value(child), param_env, ctx)?;
            match &qval {
                Value::Null => {} // skip null query params
                _ => query_params.push((qname, format!("{}", qval))),
            }
        }
    }

    // 7. Request body
    let body_json = match find_property(transport.properties.clone(), "body".to_string(), si.clone()) {
        Some(body_node) => {
            let body_val = eval_expr(&body_node, param_env, ctx)?;
            Some(value_to_json(&body_val))
        }
        None => None,
    };

    // 8. Response format
    let response_format = find_property_string(
        transport.properties.clone(), "response_format".to_string(), si.clone(),
    ).unwrap_or_else(|| "Json".to_string());

    // Build and send request
    eprintln!("[rest] {} {}", method, url);

    let mut request = match method.as_str() {
        "GET" => ureq::get(&url),
        "POST" => ureq::post(&url),
        "PUT" => ureq::put(&url),
        "DELETE" => ureq::delete(&url),
        "PATCH" => ureq::patch(&url),
        _ => return Err(InterpError::TypeError {
            msg: format!("unsupported HTTP method: {}", method),
        }),
    };

    // Set auth
    if let Some(token) = &auth_token {
        if !token.is_empty() {
            let header_val = if auth_header_name == "Authorization" {
                format!("Bearer {}", token)
            } else {
                token.clone()
            };
            request = request.set(&auth_header_name, &header_val);
        }
    }

    // Set custom headers
    for (name, val) in &headers {
        request = request.set(name, val);
    }

    // Set query params
    for (name, val) in &query_params {
        request = request.query(name, val);
    }

    // Send
    let response = if let Some(json) = body_json {
        request.set("Content-Type", "application/json")
            .send_string(&json.to_string())
    } else {
        request.call()
    };

    match response {
        Ok(resp) => {
            let status = resp.status();
            if status >= 400 {
                let body = resp.into_string().unwrap_or_default();
                return Err(InterpError::TypeError {
                    msg: format!("HTTP {}: {}", status, body),
                });
            }
            if response_format == "Text" {
                let body = resp.into_string().unwrap_or_default();
                return map_response_to_value(&body, None, op_node, ctx);
            }
            let body = resp.into_string().unwrap_or_default();
            let json: serde_json::Value = serde_json::from_str(&body)
                .unwrap_or(serde_json::Value::String(body));
            map_response_to_value_json(&json, op_node, ctx)
        }
        Err(ureq::Error::Status(status, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            Err(InterpError::TypeError {
                msg: format!("HTTP {}: {}", status, body),
            })
        }
        Err(e) => Err(InterpError::TypeError {
            msg: format!("HTTP request failed: {}", e),
        }),
    }
}

/// Resolve auth header name and token from service config + environment.
fn resolve_auth(
    service_node: &Rc<Node>,
    transport: &Rc<Node>,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> (String, Option<String>) {
    // Check service config for auth scheme and source
    let auth_scheme = find_service_config_string(service_node, "svc_auth", si);
    let auth_source = find_service_config_string(service_node, "svc_auth_source", si);

    // Determine header name
    let header_name = if let Some(ref scheme) = auth_scheme {
        if scheme.starts_with("Header(") {
            // Header("x-api-key") → extract the header name
            scheme.trim_start_matches("Header(\"")
                .trim_end_matches("\")")
                .to_string()
        } else {
            "Authorization".to_string()
        }
    } else {
        // Check transport-level auth header
        find_property_string(transport.properties.clone(), "auth_header".to_string(), si.clone())
            .unwrap_or_else(|| "Authorization".to_string())
    };

    // Resolve token from environment
    let token = if let Some(ref source) = auth_source {
        // Parse EnvVar { name: "VAR_NAME" } or just use as env var name
        let env_var = if source.contains('{') {
            // Extract name from "EnvVar { name: \"GITHUB_TOKEN\" }"
            source.split('"').nth(1).unwrap_or(source).to_string()
        } else {
            source.clone()
        };
        std::env::var(&env_var).ok()
    } else {
        None
    };

    (header_name, token)
}

/// Find a string value from a service's config properties.
fn find_service_config_string(
    service_node: &Rc<Node>,
    key: &str,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<String> {
    for prop in service_node.properties.iter() {
        let name = field_init_node_name_at(prop.clone(), si.clone());
        if name == key {
            let val_node = field_init_node_value(prop);
            // Try literal string
            if let ExprData::ExprLiteral { ref value } = *val_node.expr_data {
                if let LiteralValue::LitStr { value: s } = value.as_ref() {
                    return Some(s.clone());
                }
            }
            // Try authored name (for enum-like values)
            let authored = authored_name_at(si.clone(), &val_node);
            if !authored.is_empty() {
                return Some(authored);
            }
        }
    }
    None
}

/// Convert a Value to serde_json::Value for request bodies.
fn value_to_json(val: &Value) -> serde_json::Value {
    match val {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(n) => serde_json::json!(*n),
        Value::Float(f) => serde_json::json!(*f),
        Value::Str(s) => serde_json::Value::String(s.clone()),
        Value::List(items) => {
            serde_json::Value::Array(items.iter().map(value_to_json).collect())
        }
        Value::Map(m) => {
            let obj: serde_json::Map<String, serde_json::Value> = m.iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        Value::Record { fields, .. } | Value::Variant { fields, .. } => {
            let obj: serde_json::Map<String, serde_json::Value> = fields.iter()
                .filter(|(_, v)| !matches!(v, Value::Null))
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        Value::Unit => serde_json::Value::Null,
        Value::Closure { .. } => serde_json::Value::String("<closure>".to_string()),
    }
}

/// Map a text response to the operation's return type.
fn map_response_to_value(
    text: &str,
    _json: Option<&serde_json::Value>,
    op_node: &Rc<Node>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let return_type = match op_node.inferred.as_deref() {
        Some(crate::v2_std_core::InferredNode::Resolved { node }) => node.clone(),
        _ => return Ok(Value::Str(text.to_string())),
    };
    let children = &return_type.children;
    if children.is_empty() {
        return Ok(Value::Str(text.to_string()));
    }
    // Single field → return text
    if children.len() == 1 {
        return Ok(Value::Str(text.to_string()));
    }
    // Multi-field: map by from_key, defaulting to text for "text" and "body" fields
    let mut fields = HashMap::new();
    for child in children.iter() {
        let field_name = authored_name_at(ctx.si(), child);
        fields.insert(field_name, Value::Str(text.to_string()));
    }
    Ok(Value::Record {
        type_name: authored_name_at(ctx.si(), op_node),
        fields: Rc::new(fields),
    })
}

/// Map a JSON response to the operation's return type using from_key paths.
fn map_response_to_value_json(
    json: &serde_json::Value,
    op_node: &Rc<Node>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let return_type = match op_node.inferred.as_deref() {
        Some(crate::v2_std_core::InferredNode::Resolved { node }) => node.clone(),
        _ => return Ok(json_to_value(json)),
    };
    let children = &return_type.children;
    if children.is_empty() {
        return Ok(json_to_value(json));
    }

    // If the return type is a List, the response is an array
    let type_name = authored_name_at(ctx.si(), &return_type);
    if type_name == "List" || json.is_array() {
        return Ok(json_to_value(json));
    }

    // Multi-field record: extract fields via from_key JSON paths
    let mut fields = HashMap::new();
    for child in children.iter() {
        let field_name = authored_name_at(ctx.si(), child);
        let from_key = extract_from_key(child, ctx);
        let val = match from_key {
            Some(path) => {
                // Convert "content/0/text" to JSON pointer "/content/0/text"
                let pointer = format!("/{}", path);
                match json.pointer(&pointer) {
                    Some(v) => json_to_value(v),
                    None => Value::Null,
                }
            }
            None => {
                // Try direct field name as JSON key
                match json.get(&field_name) {
                    Some(v) => json_to_value(v),
                    None => Value::Null,
                }
            }
        };
        fields.insert(field_name, val);
    }

    Ok(Value::Record {
        type_name: authored_name_at(ctx.si(), op_node),
        fields: Rc::new(fields),
    })
}

/// Convert serde_json::Value to interpreter Value.
fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else {
                Value::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => Value::Str(s.clone()),
        serde_json::Value::Array(arr) => {
            Value::List(Rc::new(arr.iter().map(json_to_value).collect()))
        }
        serde_json::Value::Object(obj) => {
            let fields: HashMap<String, Value> = obj.iter()
                .map(|(k, v)| (k.clone(), json_to_value(v)))
                .collect();
            Value::Map(Rc::new(fields))
        }
    }
}

/// Evaluate mock_response from an operation's properties for dry-run mode.
fn eval_mock_response(
    op_node: &Rc<Node>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    // Find first mock_* property
    for prop in op_node.properties.iter() {
        let prop_name = field_init_node_name_at(prop.clone(), ctx.si());
        if has_mock_prefix(&prop_name) {
            // The mock property value is a record literal
            let val_node = field_init_node_value(prop);
            return eval_expr(&val_node, &Env::empty(), ctx);
        }
    }
    // No mock response — return Unit
    Ok(Value::Unit)
}

// ---------------------------------------------------------------------------
// Built-in functions (v2_rt equivalents)
// ---------------------------------------------------------------------------

fn eval_builtin(
    name: &str,
    args: &[(Option<String>, Value)],
    _ctx: &InterpContext,
) -> InterpResult<Option<Value>> {
    let positional: Vec<&Value> = args.iter().map(|(_, v)| v).collect();

    match name {
        "to_string" => {
            let v = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "to_string requires 1 argument".to_string(),
            })?;
            Ok(Some(Value::Str(format!("{}", v))))
        }

        "parse_int" => {
            let s = expect_str(positional.first().copied(), "parse_int")?;
            match s.parse::<i64>() {
                Ok(n) => Ok(Some(Value::Int(n))),
                Err(_) => Ok(Some(Value::Null)),
            }
        }

        "concat" => {
            // Variadic string concat (common in .dag code)
            if positional.len() >= 2 && positional.iter().all(|v| matches!(v, Value::Str(_))) {
                let mut result = String::new();
                for v in &positional {
                    if let Value::Str(s) = v {
                        result.push_str(s);
                    }
                }
                return Ok(Some(Value::Str(result)));
            }
            match positional.as_slice() {
                [Value::List(a), Value::List(b)] => {
                    let mut result = a.to_vec();
                    result.extend(b.iter().cloned());
                    Ok(Some(Value::List(Rc::new(result))))
                }
                _ => Ok(None),
            }
        }

        "count" => {
            match positional.first() {
                Some(Value::List(items)) =>
                    Ok(Some(Value::Int(items.len() as i64))),
                _ => Ok(None),
            }
        }

        "reverse" => {
            match positional.first() {
                Some(Value::List(items)) => {
                    let mut r = items.to_vec();
                    r.reverse();
                    Ok(Some(Value::List(Rc::new(r))))
                }
                _ => Ok(None),
            }
        }

        "string_length" => {
            let s = expect_str(positional.first().copied(), "string_length")?;
            Ok(Some(Value::Int(s.chars().count() as i64)))
        }

        "substring" => {
            let s = expect_str(positional.first().copied(), "substring")?;
            let start = expect_int(positional.get(1).copied(), "substring start")?;
            let end = expect_int(positional.get(2).copied(), "substring end")?;
            Ok(Some(Value::Str(v2_rt::substring(&s, start, end))))
        }

        "char_at" => {
            let s = expect_str(positional.first().copied(), "char_at")?;
            let pos = expect_int(positional.get(1).copied(), "char_at pos")?;
            Ok(Some(Value::Str(v2_rt::char_at(&s, pos))))
        }

        "string_contains" | "contains" => {
            let s = expect_str(positional.first().copied(), "contains")?;
            let sub = expect_str(positional.get(1).copied(), "contains sub")?;
            Ok(Some(Value::Bool(s.contains(&sub))))
        }

        "replace" => {
            let s = expect_str(positional.first().copied(), "replace")?;
            let from = expect_str(positional.get(1).copied(), "replace from")?;
            let to = expect_str(positional.get(2).copied(), "replace to")?;
            Ok(Some(Value::Str(s.replace(&from, &to))))
        }

        "code_point" => {
            let s = expect_str(positional.first().copied(), "code_point")?;
            let cp = s.chars().next().map(|c| c as i64).unwrap_or(0);
            Ok(Some(Value::Int(cp)))
        }

        "from_code_point" => {
            let cp = expect_int(positional.first().copied(), "from_code_point")?;
            let c = char::from_u32(cp as u32).unwrap_or('\0');
            Ok(Some(Value::Str(c.to_string())))
        }

        "list_push" | "append" => {
            match positional.as_slice() {
                [Value::List(items), item] => {
                    let mut result = items.to_vec();
                    result.push((*item).clone());
                    Ok(Some(Value::List(Rc::new(result))))
                }
                _ => Ok(None),
            }
        }

        "list_concat" => {
            match positional.as_slice() {
                [Value::List(a), Value::List(b)] => {
                    let mut result = a.to_vec();
                    result.extend(b.iter().cloned());
                    Ok(Some(Value::List(Rc::new(result))))
                }
                _ => Ok(None),
            }
        }

        "empty_map" => Ok(Some(Value::Map(Rc::new(HashMap::new())))),

        "map_insert" => {
            match positional.as_slice() {
                [Value::Map(m), Value::Str(k), v] => {
                    let mut result = HashMap::clone(&m);
                    result.insert(k.clone(), (*v).clone());
                    Ok(Some(Value::Map(Rc::new(result))))
                }
                _ => Ok(None),
            }
        }

        "map_get" | "lookup" => {
            match positional.as_slice() {
                [Value::Map(m), Value::Str(k)] =>
                    Ok(Some(m.get(k.as_str()).cloned().unwrap_or(Value::Null))),
                _ => Ok(None),
            }
        }

        "map_keys" => {
            match positional.first() {
                Some(Value::Map(m)) => {
                    let keys: Vec<Value> = m.keys().map(|k| Value::Str(k.clone())).collect();
                    Ok(Some(Value::List(Rc::new(keys))))
                }
                _ => Ok(None),
            }
        }

        "map_values" => {
            match positional.first() {
                Some(Value::Map(m)) => {
                    let vals: Vec<Value> = m.values().cloned().collect();
                    Ok(Some(Value::List(Rc::new(vals))))
                }
                _ => Ok(None),
            }
        }

        "map_contains_key" | "map_has" => {
            match positional.as_slice() {
                [Value::Map(m), Value::Str(k)] =>
                    Ok(Some(Value::Bool(m.contains_key(k.as_str())))),
                _ => Ok(None),
            }
        }

        "map_merge" => {
            match positional.as_slice() {
                [Value::Map(base), Value::Map(overlay)] => {
                    let mut result = HashMap::clone(&base);
                    for (k, v) in overlay.iter() {
                        result.insert(k.clone(), v.clone());
                    }
                    Ok(Some(Value::Map(Rc::new(result))))
                }
                _ => Ok(None),
            }
        }

        "str_eq" => {
            match positional.as_slice() {
                [Value::Str(a), Value::Str(b)] =>
                    Ok(Some(Value::Bool(a == b))),
                _ => Ok(None),
            }
        }

        // Not a built-in — fall through to user-defined function lookup
        _ => Ok(None),
    }
}

// ---------------------------------------------------------------------------
// Closure application
// ---------------------------------------------------------------------------

fn apply_closure(
    closure: &Value,
    args: &[Value],
    _env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    match closure {
        Value::Closure { params, body, env: closure_env } => {
            let mut bindings = HashMap::new();
            for (i, param) in params.iter().enumerate() {
                let val = args.get(i).cloned().unwrap_or(Value::Null);
                bindings.insert(param.clone(), val);
            }
            let call_env = Env::extend(closure_env, bindings);
            match eval_expr(body, &call_env, ctx) {
                Err(InterpError::EarlyReturn { value }) => Ok(value),
                other => other,
            }
        }
        _ => Err(InterpError::TypeError {
            msg: format!("expected closure, got {}", closure.type_label()),
        }),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn list_method_with_closure<F>(
    method_name: &str,
    receiver: Value,
    args: &[Value],
    env: &Rc<Env>,
    ctx: &InterpContext,
    f: F,
) -> InterpResult<Value>
where
    F: FnOnce(&Rc<Vec<Value>>, &Value, &Rc<Env>, &InterpContext) -> InterpResult<Value>,
{
    let items = expect_list(&receiver, method_name)?;
    let closure = args.first().ok_or_else(|| InterpError::TypeError {
        msg: format!("{} requires a closure argument", method_name),
    })?;
    f(&items, closure, env, ctx)
}

fn expect_list(val: &Value, context: &str) -> InterpResult<Rc<Vec<Value>>> {
    match val {
        Value::List(items) => Ok(items.clone()),
        _ => Err(InterpError::TypeError {
            msg: format!("{} expects a list, got {}", context, val.type_label()),
        }),
    }
}

fn expect_map(val: &Value, context: &str) -> InterpResult<Rc<HashMap<String, Value>>> {
    match val {
        Value::Map(m) => Ok(m.clone()),
        _ => Err(InterpError::TypeError {
            msg: format!("{} expects a map, got {}", context, val.type_label()),
        }),
    }
}

fn expect_string(val: &Value, context: &str) -> InterpResult<String> {
    match val {
        Value::Str(s) => Ok(s.clone()),
        _ => Err(InterpError::TypeError {
            msg: format!("{} expects a string, got {}", context, val.type_label()),
        }),
    }
}

fn expect_str(val: Option<&Value>, context: &str) -> InterpResult<String> {
    match val {
        Some(Value::Str(s)) => Ok(s.clone()),
        Some(v) => Err(InterpError::TypeError {
            msg: format!("{} expects a string argument, got {}", context, v.type_label()),
        }),
        None => Err(InterpError::TypeError {
            msg: format!("{} requires a string argument", context),
        }),
    }
}

fn expect_int(val: Option<&Value>, context: &str) -> InterpResult<i64> {
    match val {
        Some(Value::Int(n)) => Ok(*n),
        Some(v) => Err(InterpError::TypeError {
            msg: format!("{} expects an int argument, got {}", context, v.type_label()),
        }),
        None => Err(InterpError::TypeError {
            msg: format!("{} requires an int argument", context),
        }),
    }
}

fn cmp_values(a: &Value, b: &Value) -> std::cmp::Ordering {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x.cmp(y),
        (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Str(x), Value::Str(y)) => x.cmp(y),
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        _ => std::cmp::Ordering::Equal,
    }
}
