use crate::v1_rt::VecCompat;
use im::HashMap;
use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::time::Instant;

use im::HashMap as HamtMap;
use im::OrdSet;
use im::Vector as RrbVector;

use crate::cli_run::value_to_wire_json;
use crate::std_syntax::BinOp;
use crate::std_syntax::LiteralValue;
use crate::v1_compiler_emit::{extract_string_interp_parts, has_mock_prefix};
use crate::v1_compiler_infer_items::{item_kind, ItemInfo, ItemKind, ResolvedGraph, TypedModule};
use crate::v1_rt;
use crate::v1_rt::{
    rc_empty_set as empty_set, rc_set_insert as set_insert, rc_set_union as set_union, set_contains,
};
use crate::v1_std_core::{
    arg_name_at, arg_value, arm_body, arm_pattern, authored_name_at, binop_left, binop_right,
    block_stmts, cast_expr, cast_target, expr_call_func_at, expr_field_access_summary,
    expr_method_call_semantics, expr_method_name_at, expr_var_name_at, field_access_base,
    field_access_field_at, field_binding_name_at, field_binding_pattern, field_init_node_name_at,
    field_init_node_value, find_property, find_property_string, foreach_body, foreach_collection,
    foreach_variable_at, if_condition, if_else_branch, if_then_branch, index_base, index_expr,
    is_file_transport, is_rest_transport, is_shell_transport, lambda_body, lambda_param_names_at,
    let_binding_name_at, let_body, let_value, match_arm_nodes, match_scrutinee, method_arg_nodes,
    method_receiver, param_node_default_value, param_node_name_at, record_lit_type_name_at,
    return_value, slice_base, slice_end, slice_start, transport_stdin, unaryop_operand,
    CallSemantics, Cardinality, Connective, ErrorNode, ExprData, FieldAccessStyle, FieldSummary,
    FieldValueShape, InferredNode, MatchPattern, MethodSemantics, NewlineIndex, Node, SourceSpan,
    StringPart, UnaryOpKind, VarBindingKind,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Symbol(u32);

#[derive(Debug, Default)]
pub struct SymbolInterner {
    strings: Vec<String>,
    index: HashMap<String, u32>,
    calls: u64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InternStats {
    pub calls: u64,
    pub distinct: u64,
    pub hits: u64,
    pub heap_bytes: u64,
}

impl SymbolInterner {
    pub fn intern(&mut self, s: &str) -> Symbol {
        self.calls += 1;
        if let Some(&id) = self.index.get(s) {
            return Symbol(id);
        }
        let id = self.strings.len() as u32;
        self.strings.push(s.to_string());
        self.index.insert(s.to_string(), id);
        Symbol(id)
    }

    pub fn stats(&self) -> InternStats {
        let distinct = self.strings.len() as u64;
        InternStats {
            calls: self.calls,
            distinct,
            hits: self.calls.saturating_sub(distinct),
            heap_bytes: self.heap_bytes(),
        }
    }

    pub fn resolve(&self, sym: Symbol) -> &str {
        self.strings
            .get(sym.0 as usize)
            .map(|s| s.as_str())
            .unwrap_or("<invalid-symbol>")
    }

    fn heap_bytes(&self) -> u64 {
        let mut bytes = (self.strings.len() * std::mem::size_of::<String>()) as u64;
        for s in &self.strings {
            bytes += s.len() as u64;
        }
        bytes += (self.index.len() * std::mem::size_of::<(String, u32)>()) as u64;
        for key in self.index.keys() {
            bytes += key.len() as u64;
        }
        bytes
    }
}

thread_local! {
    static ACTIVE_CTX: std::cell::Cell<Option<*const InterpContext>> =
        const { std::cell::Cell::new(None) };
    static LEXICAL_BASE_ENV: std::cell::RefCell<Option<Rc<Env>>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(any(test, feature = "interp_test_witness"))]
thread_local! {
    static CALL_ENV_DEPTH_PEAK: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

fn with_lexical_base_env<R>(base: &Rc<Env>, f: impl FnOnce() -> R) -> R {
    LEXICAL_BASE_ENV.with(|cell| {
        let prev = cell.borrow_mut().take();
        cell.borrow_mut().replace(base.clone());
        struct LexicalBaseGuard {
            prev: Option<Rc<Env>>,
        }
        impl Drop for LexicalBaseGuard {
            fn drop(&mut self) {
                LEXICAL_BASE_ENV.with(|cell| {
                    *cell.borrow_mut() = self.prev.take();
                });
            }
        }
        let _guard = LexicalBaseGuard { prev };
        #[cfg(any(test, feature = "interp_test_witness"))]
        CALL_ENV_DEPTH_PEAK.with(|peak| peak.set(0));
        f()
    })
}

fn lexical_base_env(caller_env: &Rc<Env>) -> Rc<Env> {
    LEXICAL_BASE_ENV.with(|cell| {
        cell.borrow()
            .clone()
            .unwrap_or_else(|| Env::root(caller_env))
    })
}

#[cfg(any(test, feature = "interp_test_witness"))]
fn record_call_env_depth(env: &Env) {
    let depth = env.chain_depth();
    CALL_ENV_DEPTH_PEAK.with(|peak| {
        let current = peak.get();
        if depth > current {
            peak.set(depth);
        }
    });
}

fn with_active_ctx<R>(ctx: &InterpContext, f: impl FnOnce() -> R) -> R {
    ACTIVE_CTX.with(|cell| {
        let prev = cell.replace(Some(ctx as *const InterpContext));
        struct ActiveCtxGuard<'a> {
            cell: &'a std::cell::Cell<Option<*const InterpContext>>,
            prev: Option<*const InterpContext>,
        }
        impl Drop for ActiveCtxGuard<'_> {
            fn drop(&mut self) {
                self.cell.set(self.prev);
            }
        }
        let _guard = ActiveCtxGuard { cell, prev };
        f()
    })
}

pub fn with_active_context<R>(ctx: &InterpContext, f: impl FnOnce() -> R) -> R {
    with_active_ctx(ctx, f)
}

fn active_ctx() -> Option<&'static InterpContext> {
    ACTIVE_CTX.with(|cell| cell.get().map(|ptr| unsafe { &*ptr }))
}

fn resolve_sym(sym: Symbol) -> String {
    active_ctx()
        .map(|ctx| ctx.resolve(sym).to_string())
        .unwrap_or_else(|| format!("#{}", sym.0))
}

pub fn free_monoid_symbol_value_to_dotted_string(value: &Value) -> String {
    match value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            let variant = resolve_sym(*variant_name);
            if variant == "Empty" {
                return String::new();
            }
            if variant == "Cons" {
                let head = fields
                    .iter()
                    .find(|(k, _)| resolve_sym(*k) == "head")
                    .and_then(|(_, v)| match v {
                        Value::Str(s) => Some(s.clone()),
                        Value::Variant {
                            variant_name,
                            fields: sym_fields,
                            ..
                        } => {
                            let variant = resolve_sym(*variant_name);
                            if variant == "Symbol" || variant == "Atom" {
                                sym_fields
                                    .iter()
                                    .find(|(k, _)| resolve_sym(*k) == "identity")
                                    .and_then(|(_, v)| match v {
                                        Value::Str(s) => Some(s.clone()),
                                        _ => None,
                                    })
                            } else {
                                None
                            }
                        }
                        _ => None,
                    })
                    .unwrap_or_else(|| panic!("free_monoid_symbol_to_dotted: Cons.head not Str"));
                let tail = fields
                    .iter()
                    .find(|(k, _)| resolve_sym(*k) == "tail")
                    .map(|(_, v)| v)
                    .expect("free_monoid_symbol_to_dotted: Cons.tail missing");
                let rest = free_monoid_symbol_value_to_dotted_string(tail);
                if rest.is_empty() {
                    head
                } else {
                    format!("{head}.{rest}")
                }
            } else {
                panic!("free_monoid_symbol_to_dotted: unexpected variant '{variant}'");
            }
        }
        other => {
            panic!("free_monoid_symbol_to_dotted: expected FreeMonoid variant, got {other:?}")
        }
    }
}

#[derive(Debug, Clone)]
pub struct CanonKey {
    key: Value,
}

impl CanonKey {
    fn new(key: Value) -> Option<CanonKey> {
        if key == key {
            Some(CanonKey { key })
        } else {
            None
        }
    }

    pub(crate) fn value_ref(&self) -> &Value {
        &self.key
    }
}

impl PartialEq for CanonKey {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl Eq for CanonKey {}

impl Hash for CanonKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(value_hash(&self.key));
    }
}

pub(crate) fn value_hash_public(v: &Value) -> u64 {
    value_hash(v)
}

fn value_hash(v: &Value) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut h = DefaultHasher::new();

    match v {
        Value::List(_) | Value::Str(_) | Value::Variant { .. } => {
            if let Some(items) = free_monoid_to_vec(v) {
                0xF0u8.hash(&mut h);
                items.len().hash(&mut h);
                for item in &items {
                    value_hash(item).hash(&mut h);
                }
                return h.finish();
            }
        }
        _ => {}
    }

    match v {
        Value::Null => 0u8.hash(&mut h),
        Value::Unit => 1u8.hash(&mut h),
        Value::Bool(b) => {
            2u8.hash(&mut h);
            b.hash(&mut h);
        }
        Value::Int(n) => {
            3u8.hash(&mut h);
            n.hash(&mut h);
        }
        Value::Float(f) => {
            4u8.hash(&mut h);
            let bits = if *f == 0.0 { 0u64 } else { f.to_bits() };
            bits.hash(&mut h);
        }
        Value::Set(members) => {
            5u8.hash(&mut h);
            members.len().hash(&mut h);
            for m in members.iter() {
                m.hash(&mut h);
            }
        }
        Value::Record { fields, .. } => {
            6u8.hash(&mut h);
            hash_fields_commutative(fields).hash(&mut h);
        }
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            7u8.hash(&mut h);
            variant_name.hash(&mut h);
            hash_fields_commutative(fields).hash(&mut h);
        }
        Value::Map(m) => {
            8u8.hash(&mut h);
            let mut acc: u64 = 0;
            for (k, val) in m.iter() {
                let mut eh = DefaultHasher::new();
                value_hash(&k.key).hash(&mut eh);
                value_hash(val).hash(&mut eh);
                acc = acc.wrapping_add(eh.finish());
            }
            acc.hash(&mut h);
        }
        Value::Closure { .. } => 9u8.hash(&mut h),
        Value::Fn { .. } => 10u8.hash(&mut h),
        Value::List(_) | Value::Str(_) => unreachable!("FreeMonoid handled above"),
    }
    h.finish()
}

fn hash_fields_commutative(fields: &[(Symbol, Value)]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut acc: u64 = 0;
    for (sym, val) in fields.iter() {
        let mut fh = DefaultHasher::new();
        sym.0.hash(&mut fh);
        value_hash(val).hash(&mut fh);
        acc = acc.wrapping_add(fh.finish());
    }
    acc
}

pub fn fields_get(fields: &[(Symbol, Value)], sym: Symbol) -> Option<&Value> {
    fields
        .binary_search_by_key(&sym.0, |(s, _)| s.0)
        .ok()
        .map(|i| &fields[i].1)
}

pub fn sorted_fields(mut v: Vec<(Symbol, Value)>) -> Vec<(Symbol, Value)> {
    v.sort_unstable_by_key(|(sym, _)| sym.0);
    v
}

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Rc<RrbVector<Value>>),
    Map(Rc<HamtMap<CanonKey, Value>>),
    Set(Rc<OrdSet<String>>),
    Record {
        type_name: Symbol,
        fields: Rc<Vec<(Symbol, Value)>>,
    },
    Variant {
        type_name: Symbol,
        variant_name: Symbol,
        fields: Rc<Vec<(Symbol, Value)>>,
    },
    Closure {
        params: Vec<Symbol>,
        body: Rc<Node>,
        env: Rc<Env>,
    },
    Fn {
        node: Rc<Node>,
    },
    Unit,
}

pub(crate) fn list_value(items: impl Into<RrbVector<Value>>) -> Value {
    Value::List(Rc::new(items.into()))
}

fn map_value(entries: HamtMap<CanonKey, Value>) -> Value {
    Value::Map(Rc::new(entries))
}

fn optional_present(value: Value, ctx: &InterpContext) -> Value {
    Value::Variant {
        type_name: ctx.sym("Optional"),
        variant_name: ctx.sym("Present"),
        fields: Rc::new(vec![(ctx.sym("value"), value)]),
    }
}

fn optional_absent(ctx: &InterpContext) -> Value {
    Value::Variant {
        type_name: ctx.sym("Optional"),
        variant_name: ctx.sym("Absent"),
        fields: Rc::new(vec![]),
    }
}

fn map_lookup_as_optional(raw: Value, ctx: &InterpContext) -> Value {
    if matches!(raw, Value::Null) {
        optional_absent(ctx)
    } else {
        optional_present(raw, ctx)
    }
}

impl Value {
    pub fn type_label_public(&self) -> &'static str {
        self.type_label()
    }

    fn type_label(&self) -> &'static str {
        match self {
            Value::Null => "Null",
            Value::Bool(_) => "Bool",
            Value::Int(_) => "Int",
            Value::Float(_) => "Float",
            Value::Str(_) => "String",
            Value::List(_) => "List",
            Value::Map(_) => "Map",
            Value::Set(_) => "Set",
            Value::Record { .. } => "Record",
            Value::Variant { .. } => "Variant",
            Value::Closure { .. } => "Closure",
            Value::Fn { .. } => "Fn",
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
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Map(entries) => {
                write!(f, "{{")?;
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k.key, v)?;
                }
                write!(f, "}}")
            }
            Value::Set(members) => {
                write!(f, "{{")?;
                for (i, k) in members.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", k)?;
                }
                write!(f, "}}")
            }
            Value::Record { type_name, fields } => {
                write!(f, "{} {{", resolve_sym(*type_name))?;
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, " {}: {}", resolve_sym(*k), v)?;
                }
                write!(f, " }}")
            }
            Value::Variant {
                type_name: _,
                variant_name,
                fields,
            } => {
                if fields.is_empty() {
                    write!(f, "{}", resolve_sym(*variant_name))
                } else {
                    write!(f, "{} {{", resolve_sym(*variant_name))?;
                    for (i, (k, v)) in fields.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, " {}: {}", resolve_sym(*k), v)?;
                    }
                    write!(f, " }}")
                }
            }
            Value::Closure { .. } => write!(f, "<closure>"),
            Value::Fn { node } => write!(f, "<fn {}>", node.name),
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
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::Set(a), Value::Set(b)) => a == b,
            (
                Value::Variant {
                    variant_name: a,
                    fields: af,
                    ..
                },
                Value::Variant {
                    variant_name: b,
                    fields: bf,
                    ..
                },
            ) => a == b && af == bf,
            (Value::Record { fields: af, .. }, Value::Record { fields: bf, .. }) => af == bf,
            (Value::Fn { node: a }, Value::Fn { node: b }) => Rc::ptr_eq(a, b),
            (Value::List(_), Value::Variant { .. }) | (Value::Variant { .. }, Value::List(_)) => {
                match (free_monoid_to_vec(self), free_monoid_to_vec(other)) {
                    (Some(a), Some(b)) => a == b,
                    _ => false,
                }
            }
            (Value::Str(_), Value::Variant { .. })
            | (Value::Variant { .. }, Value::Str(_))
            | (Value::Str(_), Value::List(_))
            | (Value::List(_), Value::Str(_)) => {
                match (free_monoid_to_vec(self), free_monoid_to_vec(other)) {
                    (Some(a), Some(b)) => a == b,
                    _ => false,
                }
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Env {
    bindings: HashMap<Symbol, Value>,
    parent: Option<Rc<Env>>,
}

impl Env {
    pub fn empty() -> Rc<Self> {
        Rc::new(Env {
            bindings: HashMap::new(),
            parent: None,
        })
    }

    pub fn extend(parent: &Rc<Env>, bindings: HashMap<Symbol, Value>) -> Rc<Self> {
        Rc::new(Env {
            bindings,
            parent: Some(parent.clone()),
        })
    }

    pub fn with_binding(parent: &Rc<Env>, name: Symbol, value: Value) -> Rc<Self> {
        let mut bindings = HashMap::new();
        bindings.insert(name, value);
        Rc::new(Env {
            bindings,
            parent: Some(parent.clone()),
        })
    }

    pub fn lookup(&self, name: Symbol) -> Option<&Value> {
        if let Some(v) = self.bindings.get(&name) {
            Some(v)
        } else if let Some(ref parent) = self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }

    /// Outermost frame of the parent-linked chain (globals / eager data env).
    pub fn root(env: &Rc<Self>) -> Rc<Self> {
        let mut current = env.clone();
        while let Some(parent) = current.parent.clone() {
            current = parent;
        }
        current
    }

    #[cfg(any(test, feature = "interp_test_witness"))]
    pub fn chain_depth(&self) -> usize {
        1 + self
            .parent
            .as_ref()
            .map(|parent| parent.chain_depth())
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub enum InterpError {
    NoSuchFunction {
        name: String,
    },
    NoMainFunction,
    NoSuchVariable {
        name: String,
    },
    NoSuchField {
        type_name: String,
        field: String,
    },
    TypeError {
        msg: String,
    },
    CrossRepresentationEquality {
        detail: String,
    },
    StringRealizationStraddle {
        detail: String,
    },
    PatternMatchFailure {
        value: String,
    },
    DivisionByZero,
    Unimplemented {
        what: String,
    },
    EarlyReturn {
        value: Value,
    },
    AuthDeclaredButUnwired {
        service: String,
        reason: String,
    },
    EvalBudgetExceeded {
        elapsed_ms: u64,
        budget_ms: u64,
    },
    WitnessWallBudgetExceeded {
        elapsed_ms: u64,
        budget_ms: u64,
    },
    ArgvExceedsHostArgMax {
        actual_bytes: usize,
        limit_bytes: usize,
        argv0: String,
    },
    /// Application-site contract mismatch: the caller's argument list does not match the
    /// callee's declared parameter list. Typed and located (callee + the offending label)
    /// so the line stops at the application site instead of surfacing later as a
    /// `NoSuchVariable` for an unbound parameter — or, worse, not surfacing at all when
    /// the mismatched names happen to overlap. DESIGN §5: a failure arm refuses, never widens.
    CallContractMismatch {
        callee: String,
        detail: String,
    },
}

impl fmt::Display for InterpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpError::NoSuchFunction { name } => write!(f, "no such function: {}", name),
            InterpError::CallContractMismatch { callee, detail } => {
                write!(f, "call contract mismatch calling '{}': {}", callee, detail)
            }
            InterpError::NoMainFunction => write!(f, "no main function found"),
            InterpError::NoSuchVariable { name } => write!(f, "undefined variable: {}", name),
            InterpError::NoSuchField { type_name, field } => {
                write!(f, "no field '{}' on type '{}'", field, type_name)
            }
            InterpError::TypeError { msg } => write!(f, "type error: {}", msg),
            InterpError::EvalBudgetExceeded {
                elapsed_ms,
                budget_ms,
            } => {
                write!(
                    f,
                    "eval budget exceeded: {}ms elapsed > {}ms fast-lane budget (operator 5s rule 2026-07-12: a witness this slow lives in a long/ test dir and runs via its dedicated lane, not per-PR discovery)",
                    elapsed_ms, budget_ms
                )
            }
            InterpError::WitnessWallBudgetExceeded {
                elapsed_ms,
                budget_ms,
            } => {
                write!(
                    f,
                    "wet self-host receipt wall budget exceeded: {}ms elapsed > {}ms whole-receipt budget (emit+cargo wall time; nightly falsifier Wet lane 2026-07-15)",
                    elapsed_ms, budget_ms
                )
            }
            InterpError::CrossRepresentationEquality { detail } => {
                write!(f, "cross-representation equality: {}", detail)
            }
            InterpError::StringRealizationStraddle { detail } => {
                write!(f, "string realization straddle: {}", detail)
            }
            InterpError::PatternMatchFailure { value } => {
                write!(f, "non-exhaustive pattern match on: {}", value)
            }
            InterpError::DivisionByZero => write!(f, "division by zero"),
            InterpError::Unimplemented { what } => write!(f, "not yet implemented: {}", what),
            InterpError::EarlyReturn { .. } => write!(f, "internal: uncaught early return"),
            InterpError::AuthDeclaredButUnwired { service, reason } => write!(
                f,
                "auth declared but unwired for '{}': {} — refusing to send unauthenticated request",
                service, reason
            ),
            InterpError::ArgvExceedsHostArgMax {
                actual_bytes,
                limit_bytes,
                argv0,
            } => write!(
                f,
                "argv exceeds host arg limit: '{}' invocation carries a {}-byte argument > {}-byte host MAX_ARG_STRLEN — route large payloads through stdin, not argv (Linux execve(2) E2BIG; extdeps.os.exec_arg_limit.host_exec_arg_max_strlen; DESIGN §5 typed refusal in place of an opaque os error 7)",
                argv0, actual_bytes, limit_bytes
            ),
        }
    }
}

/// Three-way auth resolution: splits the conflated `Option<String>` into named states so the
/// dispatch site cannot reach the send path with auth declared but no token (§5 construction).
#[derive(Debug, Clone)]
pub enum AuthResolution {
    /// The service declares no auth; unauthenticated send is correct.
    NoAuthDeclared,
    /// Auth is declared and a non-empty token was resolved; attach the header.
    Resolved { header: String, token: String },
    /// Auth is declared (svc_auth / svc_auth_input / svc_auth_source present) but no token
    /// resolved — the caller must raise a typed error, never send unauthenticated.
    DeclaredButUnwired { reason: String },
}

pub type InterpResult<T> = Result<T, InterpError>;

type ServiceOp = (Rc<Node>, Rc<Node>);

#[derive(Default)]
struct PureCallMemo {
    map: HashMap<(usize, Vec<usize>), Value>,
    keepalive: Vec<Value>,
    keepalive_fns: Vec<Rc<Node>>,
}

#[derive(Default)]
struct ParseTableMemo {
    map: HashMap<(String, String, i64, Symbol), Value>,
    keepalive: Vec<Value>,
    lookups: u64,
    hits: u64,
    inserts: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ParseTableMemoStats {
    pub lookups: u64,
    pub hits: u64,
    pub inserts: u64,
}

// Recompute-trace ledger (diagnostic READ mode: reports, never gates — DESIGN §5
// stopped-line audit). Counts evaluations of pure named fns (empty `uses` row) per
// (fn identity, argument identity). Keying is SOUND-ONLY: an argument without a
// cheap sound identity (composite values) puts the call in the unkeyed bucket
// instead of guessing — the ledger never merges distinct work. Durations are
// inclusive of callees. Enabled via GUNBC_RECOMPUTE_TRACE=1.
#[derive(Default)]
struct EvalRecomputeTrace {
    map: std::collections::HashMap<EvalRecomputeKey, EvalRecomputeEntry>,
    unkeyed_by_fn: std::collections::HashMap<String, u64>,
    // fn-node Rcs kept alive so fn_ptr keys stay valid for the ctx lifetime
    // (same discipline as PureCallMemo.keepalive_fns).
    keepalive_fns: Vec<Rc<Node>>,
    // fn_ptr -> interned display name, so millions of ledger entries share
    // one allocation per function instead of a String clone per key.
    fn_names: std::collections::HashMap<usize, Rc<str>>,
    keyed_calls: u64,
    unkeyed_calls: u64,
    // Calls refused a NEW ledger key once map hits EVAL_RECOMPUTE_KEY_CAP —
    // disclosed, never silently dropped (existing keys keep counting).
    overflow_calls: u64,
}

// Ceiling on distinct ledger keys so a diagnostic run cannot OOM the host;
// overflow is counted and disclosed in the report.
const EVAL_RECOMPUTE_KEY_CAP: usize = 4_000_000;

#[derive(PartialEq, Eq, Hash, Clone)]
struct EvalRecomputeKey {
    fn_ptr: usize,
    args: Vec<EvalRecomputeArgKey>,
}

#[derive(PartialEq, Eq, Hash, Clone)]
enum EvalRecomputeArgKey {
    Null,
    Bool(bool),
    Int(i64),
    FloatBits(u64),
    StrHash(u64),
    UnitVariant(u32, u32),
    EmptyList,
    // Recursive content hash of a composite value (Record/Variant/List/Map/
    // Set/Fn/Unit), memoized per allocation with Weak-liveness validation so
    // a reused address can never serve a stale hash. Closures are the one
    // remaining unkeyed class (captured-env identity is not computed).
    ContentHash(u64),
}

enum CompositeWeak {
    List(std::rc::Weak<RrbVector<Value>>),
    Fields(std::rc::Weak<Vec<(Symbol, Value)>>),
    Map(std::rc::Weak<HamtMap<CanonKey, Value>>),
    Set(std::rc::Weak<OrdSet<String>>),
}

impl CompositeWeak {
    fn alive(&self) -> bool {
        match self {
            CompositeWeak::List(w) => w.strong_count() > 0,
            CompositeWeak::Fields(w) => w.strong_count() > 0,
            CompositeWeak::Map(w) => w.strong_count() > 0,
            CompositeWeak::Set(w) => w.strong_count() > 0,
        }
    }
}

type EvalRecomputeHashMemo = std::collections::HashMap<usize, (CompositeWeak, u64)>;

struct EvalRecomputeEntry {
    fn_name: Rc<str>,
    count: u64,
    total_ns: u128,
    // Distinct call-site node ptrs (capped) with "file:offset" labels. One site
    // recomputing = same call expression re-evaluated (loop-invariant hoist or
    // value coincidence — Share/memoize territory, invisible to static analysis
    // when value-coincident). Multiple sites = a cross-site duplicate demand
    // (static rewire candidate).
    sites: Vec<(usize, String)>,
}

const EVAL_RECOMPUTE_SITE_CAP: usize = 4;

pub fn eval_recompute_trace_enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("GUNBC_RECOMPUTE_TRACE").is_ok_and(|v| v != "0"))
}

// The eval-frame memo: the ladder's single-site discharge provider, realized
// in the seed. Buckets by the ledger key (fn identity x argument identity) and
// serves only after the stored call's argument names AND values verify equal —
// a hash collision degrades to recompute, never to a wrong value. Eviction is
// ScopeExit at the WITNESS frame: batch surfaces share one ctx across an
// entry's witnesses and call eval_call_memo_frame_exit after each claim fn
// (ctx-lifetime retention of argument+result values across witnesses is
// byte-unbounded — the 2026-07-10 20GiB-class regression). Admission stops at
// the entry cap with the refusal COUNTED (overflow), never silent. Default ON
// everywhere;
// GUNBC_EVAL_MEMO=0 is a diagnostic realization switch (recompute instead of
// serve — semantics identical), and the receipt discloses hits/misses so a
// disabled memo is visible as memo_hits=0, never silently assumed working.
struct EvalCallMemo {
    // Per-ctx realization switch (read from GUNBC_EVAL_MEMO at ctx
    // construction, not a process-wide latch): provider-attribution tests pin
    // the outer eval-frame provider off on their own ctx so an inner
    // provider's hit counters stay discriminating; semantics are identical
    // either way (recompute instead of serve).
    enabled: bool,
    map: std::collections::HashMap<EvalRecomputeKey, Vec<(Vec<(Option<String>, Value)>, Value)>>,
    // fn-node Rcs kept alive so fn_ptr keys stay valid for the ctx lifetime
    // (same discipline as EvalRecomputeTrace.keepalive_fns).
    keepalive_fns: Vec<Rc<Node>>,
    hits: u64,
    misses: u64,
    overflow: u64,
}

impl Default for EvalCallMemo {
    fn default() -> Self {
        EvalCallMemo {
            enabled: eval_call_memo_env_default(),
            map: std::collections::HashMap::new(),
            keepalive_fns: Vec::new(),
            hits: 0,
            misses: 0,
            overflow: 0,
        }
    }
}

const EVAL_CALL_MEMO_ENTRY_CAP: usize = 1_000_000;

fn eval_call_memo_env_default() -> bool {
    std::env::var("GUNBC_EVAL_MEMO")
        .map(|v| v != "0")
        .unwrap_or(true)
}

/// Realization switch, per ctx: an inner provider's by-execution receipt suite
/// (e.g. the parse-table MemoTier's amortization tests) pins the eval-frame
/// provider off so pass-2 demands re-execute and the inner door's hit counters
/// keep discriminating. Values are identical either way.
pub fn set_eval_call_memo_enabled(ctx: &InterpContext, enabled: bool) {
    ctx.eval_call_memo.borrow_mut().enabled = enabled;
}

/// Frame exit for the eval-call memo: the memo's eviction scope is the WITNESS
/// frame, not the ctx. Batch surfaces (claim_batch, claim_executor) share one
/// ctx across an entry's witnesses for the resolve-side ReferenceTier share —
/// but the memo stores full argument+result VALUES, so ctx-lifetime retention
/// across N witnesses is byte-unbounded by construction (measured 2026-07-10:
/// single witness plateaus ~3.4GiB, six witnesses in one ctx climb past
/// ~20GiB to SIGKILL). Callers invoke this after each claim function; the map
/// and keepalives drain, counters stay CUMULATIVE so receipts remain honest.
/// Cross-witness serving is an outer-frame promotion that must arrive as a
/// conscious provider row with byte-bounded admission — never a default.
pub fn eval_call_memo_frame_exit(ctx: &InterpContext) {
    let mut m = ctx.eval_call_memo.borrow_mut();
    m.map.clear();
    m.keepalive_fns.clear();
}

#[derive(Default, Clone)]
pub struct MutationCounters {
    pub map_insert_calls: u64,
    pub map_insert_entries_copied: u64,
    pub map_merge_calls: u64,
    pub map_merge_entries_copied: u64,
    pub list_push_calls: u64,
    pub list_push_items_copied: u64,
    pub list_concat_calls: u64,
    pub list_concat_items_copied: u64,
    pub set_insert_calls: u64,
    pub set_insert_items_copied: u64,
    pub set_union_calls: u64,
    pub set_union_items_copied: u64,
}

impl fmt::Display for MutationCounters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rows: [(&str, u64, u64); 6] = [
            (
                "map_insert",
                self.map_insert_calls,
                self.map_insert_entries_copied,
            ),
            (
                "map_merge",
                self.map_merge_calls,
                self.map_merge_entries_copied,
            ),
            (
                "list_push",
                self.list_push_calls,
                self.list_push_items_copied,
            ),
            (
                "list_concat",
                self.list_concat_calls,
                self.list_concat_items_copied,
            ),
            (
                "set_insert",
                self.set_insert_calls,
                self.set_insert_items_copied,
            ),
            (
                "set_union",
                self.set_union_calls,
                self.set_union_items_copied,
            ),
        ];
        for (name, calls, copied) in rows {
            writeln!(
                f,
                "  {:<12} {:>12} calls  {:>16} entries copied  (avg {:.1}/call)",
                name,
                calls,
                copied,
                if calls == 0 {
                    0.0
                } else {
                    copied as f64 / calls as f64
                }
            )?;
        }
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct VariantAccounting {
    pub occurrences: u64,
    pub unique_allocations: u64,
    pub shared_references: u64,
    pub heap_bytes: u64,
}

#[derive(Default)]
pub struct MemoryAccounting {
    pub per_variant: std::collections::BTreeMap<&'static str, VariantAccounting>,
    pub total_heap_bytes: u64,
    pub total_unique_allocations: u64,
    pub total_shared_references: u64,
}

impl fmt::Display for MemoryAccounting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (label, v) in &self.per_variant {
            writeln!(
                f,
                "  {:<10} {:>12} occurrences  {:>10} unique  {:>10} shared  {:>14} bytes",
                label, v.occurrences, v.unique_allocations, v.shared_references, v.heap_bytes
            )?;
        }
        writeln!(
            f,
            "  total: {} bytes across {} unique allocations ({} shared references)",
            self.total_heap_bytes, self.total_unique_allocations, self.total_shared_references
        )
    }
}

impl MemoryAccounting {
    fn variant(&mut self, label: &'static str) -> &mut VariantAccounting {
        self.per_variant.entry(label).or_default()
    }

    fn add_unique(&mut self, label: &'static str, bytes: u64) {
        let v = self.variant(label);
        v.unique_allocations += 1;
        v.heap_bytes += bytes;
        self.total_unique_allocations += 1;
        self.total_heap_bytes += bytes;
    }

    fn add_shared(&mut self, label: &'static str) {
        self.variant(label).shared_references += 1;
        self.total_shared_references += 1;
    }
}

fn accounting_first_visit(
    ptr: usize,
    label: &'static str,
    visited: &mut std::collections::HashSet<usize>,
    acc: &mut MemoryAccounting,
) -> bool {
    if visited.insert(ptr) {
        true
    } else {
        acc.add_shared(label);
        false
    }
}

fn account_env(
    env: &Rc<Env>,
    visited: &mut std::collections::HashSet<usize>,
    acc: &mut MemoryAccounting,
) {
    if !accounting_first_visit(Rc::as_ptr(env) as usize, "(env)", visited, acc) {
        return;
    }
    let mut bytes = (env.bindings.len() * std::mem::size_of::<(Symbol, Value)>()) as u64;
    acc.add_unique("(env)", bytes);
    for value in env.bindings.values() {
        account_value(value, visited, acc);
    }
    if let Some(parent) = &env.parent {
        account_env(parent, visited, acc);
    }
}

fn account_named_fields(
    label: &'static str,
    fields: &Rc<Vec<(Symbol, Value)>>,
    visited: &mut std::collections::HashSet<usize>,
    acc: &mut MemoryAccounting,
) {
    if !accounting_first_visit(Rc::as_ptr(fields) as usize, label, visited, acc) {
        return;
    }
    let bytes = (fields.len() * std::mem::size_of::<(Symbol, Value)>()) as u64;
    acc.add_unique(label, bytes);
    for (_, value) in fields.iter() {
        account_value(value, visited, acc);
    }
}

fn account_interner(
    ctx_ptr: usize,
    interner: &SymbolInterner,
    visited: &mut std::collections::HashSet<usize>,
    acc: &mut MemoryAccounting,
) {
    if !accounting_first_visit(ctx_ptr, "(interner)", visited, acc) {
        return;
    }
    acc.add_unique("(interner)", interner.heap_bytes());
}

fn account_value(
    value: &Value,
    visited: &mut std::collections::HashSet<usize>,
    acc: &mut MemoryAccounting,
) {
    let label = value.type_label();
    acc.variant(label).occurrences += 1;
    match value {
        Value::Null | Value::Bool(_) | Value::Int(_) | Value::Float(_) | Value::Unit => {}
        Value::Str(s) => acc.add_unique(label, s.len() as u64),
        Value::List(items) => {
            if !accounting_first_visit(Rc::as_ptr(items) as usize, label, visited, acc) {
                return;
            }
            acc.add_unique(label, (items.len() * std::mem::size_of::<Value>()) as u64);
            for item in items.iter() {
                account_value(item, visited, acc);
            }
        }
        Value::Map(entries) => {
            if !accounting_first_visit(Rc::as_ptr(entries) as usize, label, visited, acc) {
                return;
            }
            acc.add_unique(
                label,
                (entries.len() * (std::mem::size_of::<CanonKey>() + std::mem::size_of::<Value>()))
                    as u64,
            );
            for (k, v) in entries.iter() {
                account_value(&k.key, visited, acc);
                account_value(v, visited, acc);
            }
        }
        Value::Set(members) => {
            if !accounting_first_visit(Rc::as_ptr(members) as usize, label, visited, acc) {
                return;
            }
            let mut bytes = (members.len() * std::mem::size_of::<String>()) as u64;
            for m in members.iter() {
                bytes += m.len() as u64;
            }
            acc.add_unique(label, bytes);
        }
        Value::Record { fields, .. } => {
            account_named_fields(label, fields, visited, acc);
        }
        Value::Variant { fields, .. } => {
            account_named_fields(label, fields, visited, acc);
        }
        Value::Closure {
            params,
            env,
            body: _,
        } => {
            let bytes = (params.len() * std::mem::size_of::<Symbol>()) as u64;
            acc.add_unique(label, bytes);
            account_env(env, visited, acc);
        }
        Value::Fn { .. } => {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExecutionMode {
    Hermetic,
    Wet,
    Record,
}

impl ExecutionMode {
    pub fn is_hermetic(self) -> bool {
        matches!(self, ExecutionMode::Hermetic)
    }

    pub fn is_record(self) -> bool {
        matches!(self, ExecutionMode::Record)
    }

    pub fn is_wet_dispatch(self) -> bool {
        matches!(self, ExecutionMode::Wet | ExecutionMode::Record)
    }
}

pub struct InterpContext {
    pub modules: Rc<im::Vector<Rc<TypedModule>>>,
    pub item_registry: Rc<HashMap<String, Rc<ItemInfo>>>,
    pub source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    fn_nodes: HashMap<String, Rc<Node>>,
    service_ops: HashMap<String, ServiceOp>,
    pub execution_mode: ExecutionMode,
    pub fixture_store: Option<Rc<crate::recorded_fixture::RecordedFixtureStore>>,
    data_cache: std::cell::RefCell<HashMap<usize, Value>>,
    // Per-call parameter-name derivation is invariant per fn_node but was re-sliced from
    // source spans on every call (authored_name_at). Memoize it per ctx, keyed by fn_node
    // pointer identity — sound because the ctx owns fn_nodes, so pointers are stable for the
    // cache's lifetime and the cache dies with the ctx (same discipline as data_cache above).
    // Value = (filtered named-param list, all-param list), matching call_function's two uses.
    param_name_cache: std::cell::RefCell<HashMap<usize, Rc<(Vec<String>, Vec<String>)>>>,
    // Same chokepoint, ExprVar arm: eval_var rebuilt the variable name String from its source
    // span (expr_var_name_at) and re-interned it (ctx.sym) on every read. Memoize the interned
    // Symbol per ExprVar node — keyed by node pointer, sound for the ctx lifetime exactly as
    // data_cache/param_name_cache above. Eval then skips the slice + re-intern and goes straight
    // to env.lookup(sym); the name String is materialized lazily only on the registry slow path.
    var_sym_cache: std::cell::RefCell<HashMap<usize, Symbol>>,
    // Same chokepoint, ExprCall callee name: eval_call re-sliced the callee name from its source
    // span (expr_call_func_at -> authored_name_at) on every call. Memoize the decoded name per
    // call node — keyed by node pointer, sound for the ctx lifetime as the caches above.
    call_func_name_cache: std::cell::RefCell<HashMap<usize, String>>,
    pure_call_memo: std::cell::RefCell<PureCallMemo>,
    parse_table_memo: std::cell::RefCell<ParseTableMemo>,
    eval_recompute_trace: std::cell::RefCell<EvalRecomputeTrace>,
    eval_call_memo: std::cell::RefCell<EvalCallMemo>,
    // Effect-dispatch odometer: incremented on every service-operation dispatch.
    // The eval-call memo compares it across a named call and refuses to memoize
    // any call during which it advanced — a WorldRead/effect is never served
    // stale from cache (the uses-empty purity gate is vacuous corpus-wide: no
    // corpus func declares a `uses` clause, so every effectful wrapper was
    // memo-eligible; found via the artifact-store List-after-Delete staleness).
    effect_dispatch_count: std::cell::Cell<u64>,
    eval_recompute_hash_memo: std::cell::RefCell<EvalRecomputeHashMemo>,
    mutation_counters: std::cell::RefCell<MutationCounters>,
    symbols: RefCell<SymbolInterner>,
    published_mock_keys: RefCell<Option<Rc<std::collections::HashSet<String>>>>,
    whole_tree_published_keys: Option<Rc<std::collections::HashSet<String>>>,
    governed_services: RefCell<Option<Rc<std::collections::HashSet<String>>>>,
    // Cooperative per-witness eval deadline (fast-lane 5s rule, operator 2026-07-12).
    // The bound must unwind from INSIDE eval as a typed error: witness evals run on
    // in-process worker threads with no kill authority, so a wall-clock bound imposed
    // from outside cannot terminate them (the Phase A governor lesson). The budget is
    // denominated in THREAD CPU TIME, not wall: the fast-lane rule targets the eval-wedge
    // (a non-terminating eval that burns a core), so a witness inflated by cold-I/O reads
    // or by governor time-slicing (many witnesses sharing a core) must not be misclassified
    // — "assuming the infra isn't the problem" is exactly the CPU-vs-wall gap. The stored
    // pair is (cpu_baseline_nanos, budget_ms).
    eval_deadline: std::cell::Cell<Option<(u128, u64)>>,
    eval_deadline_stride: std::cell::Cell<u32>,
    // Lane-level budget: when set, run_claim_measured re-arms the deadline per witness.
    witness_eval_budget_ms: std::cell::Cell<Option<u64>>,
    // Whole-receipt wall budget for Wet self-host receipts (emit+cargo subprocess I/O included).
    witness_wall_budget_ms: std::cell::Cell<Option<u64>>,
}

impl InterpContext {
    pub fn sym(&self, s: &str) -> Symbol {
        self.symbols.borrow_mut().intern(s)
    }

    pub fn resolve(&self, sym: Symbol) -> String {
        self.symbols.borrow().resolve(sym).to_string()
    }

    pub fn sym_eq(&self, sym: Symbol, name: &str) -> bool {
        self.symbols.borrow().resolve(sym) == name
    }

    pub fn field<'a>(&self, fields: &'a [(Symbol, Value)], name: &str) -> Option<&'a Value> {
        fields_get(fields, self.sym(name))
    }

    pub fn format_value(&self, val: &Value) -> String {
        with_active_ctx(self, || format!("{}", val))
    }

    pub fn mutation_counters_snapshot(&self) -> MutationCounters {
        self.mutation_counters.borrow().clone()
    }

    pub fn interner_stats_snapshot(&self) -> InternStats {
        self.symbols.borrow().stats()
    }

    pub fn parse_table_memo_stats_snapshot(&self) -> ParseTableMemoStats {
        let st = self.parse_table_memo.borrow();
        ParseTableMemoStats {
            lookups: st.lookups,
            hits: st.hits,
            inserts: st.inserts,
        }
    }

    pub fn account_retained_memory(&self, extra_roots: &[&Value]) -> MemoryAccounting {
        let mut visited = std::collections::HashSet::new();
        let mut acc = MemoryAccounting::default();
        account_interner(
            self as *const Self as usize,
            &self.symbols.borrow(),
            &mut visited,
            &mut acc,
        );
        for value in self.data_cache.borrow().values() {
            account_value(value, &mut visited, &mut acc);
        }
        let memo = self.pure_call_memo.borrow();
        for value in memo.map.values() {
            account_value(value, &mut visited, &mut acc);
        }
        for value in memo.keepalive.iter() {
            account_value(value, &mut visited, &mut acc);
        }
        for value in extra_roots {
            account_value(value, &mut visited, &mut acc);
        }
        acc
    }

    pub fn new(
        graph: &ResolvedGraph,
        source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
        execution_mode: ExecutionMode,
    ) -> Self {
        Self::with_runtime_options(graph, source_indices, execution_mode, None, None)
    }

    pub fn with_fixture_store(
        graph: &ResolvedGraph,
        source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
        execution_mode: ExecutionMode,
        fixture_store: Option<Rc<crate::recorded_fixture::RecordedFixtureStore>>,
    ) -> Self {
        Self::with_runtime_options(graph, source_indices, execution_mode, fixture_store, None)
    }

    pub fn with_runtime_options(
        graph: &ResolvedGraph,
        source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
        execution_mode: ExecutionMode,
        fixture_store: Option<Rc<crate::recorded_fixture::RecordedFixtureStore>>,
        whole_tree_published_keys: Option<Rc<std::collections::HashSet<String>>>,
    ) -> Self {
        let mut fn_nodes = HashMap::new();
        let mut service_ops = HashMap::new();
        for module in graph.modules.iter() {
            let module_path = authored_name_at(source_indices.clone(), module.module.clone());
            for item in module.items.iter() {
                let name = authored_name_at(source_indices.clone(), item.clone());
                if !name.is_empty() {
                    fn_nodes.insert(name.clone(), item.clone());
                    if !module_path.is_empty() {
                        fn_nodes.insert(format!("{}.{}", module_path, name), item.clone());
                    }
                }
                // Service-item detection is node-local: the item node carries the
                // `transport` that *defines* it as a service, so `item_kind` of the
                // node itself is the single authority. Do NOT gate on a name-keyed
                // `item_registry` lookup — two top-level items can share one authored
                // name (the `std.resources` `resource Filesystem` is an OtherItem;
                // the `extdeps.filesystem` `service Filesystem` is a ServiceItem), and
                // once both land in the same import closure the non-service entry can
                // win the registry merge and poison the lookup, silently dropping the
                // service's operations (-> "unknown service operation" at runtime).
                if item_kind(item.clone()) == ItemKind::ServiceItem {
                    for op in item.children.iter() {
                        let op_name = authored_name_at(source_indices.clone(), op.clone());
                        if op_name.is_empty() {
                            continue;
                        }
                        if !name.is_empty() {
                            let key = format!("{}.{}", name, op_name);
                            service_ops.insert(key, (item.clone(), op.clone()));
                        }
                        if !item.name.is_empty() && item.name != name {
                            let key = format!("{}.{}", item.name, op_name);
                            service_ops.insert(key, (item.clone(), op.clone()));
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
            execution_mode,
            fixture_store,
            data_cache: std::cell::RefCell::new(HashMap::new()),
            param_name_cache: std::cell::RefCell::new(HashMap::new()),
            var_sym_cache: std::cell::RefCell::new(HashMap::new()),
            call_func_name_cache: std::cell::RefCell::new(HashMap::new()),
            pure_call_memo: std::cell::RefCell::new(PureCallMemo::default()),
            parse_table_memo: std::cell::RefCell::new(ParseTableMemo::default()),
            eval_recompute_trace: std::cell::RefCell::new(EvalRecomputeTrace::default()),
            eval_call_memo: std::cell::RefCell::new(EvalCallMemo::default()),
            effect_dispatch_count: std::cell::Cell::new(0),
            eval_recompute_hash_memo: std::cell::RefCell::new(EvalRecomputeHashMemo::default()),
            mutation_counters: std::cell::RefCell::new(MutationCounters::default()),
            symbols: RefCell::new(SymbolInterner::default()),
            published_mock_keys: RefCell::new(None),
            whole_tree_published_keys,
            governed_services: RefCell::new(None),
            eval_deadline: std::cell::Cell::new(None),
            eval_deadline_stride: std::cell::Cell::new(0),
            witness_eval_budget_ms: std::cell::Cell::new(None),
            witness_wall_budget_ms: std::cell::Cell::new(None),
        }
    }

    pub fn arm_eval_deadline(&self, budget_ms: u64) {
        self.eval_deadline
            .set(Some((thread_cpu_nanos(), budget_ms)));
        self.eval_deadline_stride.set(0);
    }

    pub fn clear_eval_deadline(&self) {
        self.eval_deadline.set(None);
    }

    pub fn set_witness_eval_budget(&self, budget_ms: Option<u64>) {
        self.witness_eval_budget_ms.set(budget_ms);
    }

    pub fn witness_eval_budget(&self) -> Option<u64> {
        self.witness_eval_budget_ms.get()
    }

    pub fn set_witness_wall_budget(&self, budget_ms: Option<u64>) {
        self.witness_wall_budget_ms.set(budget_ms);
    }

    pub fn witness_wall_budget(&self) -> Option<u64> {
        self.witness_wall_budget_ms.get()
    }

    fn published_mock_keys(&self) -> InterpResult<Rc<std::collections::HashSet<String>>> {
        {
            if let Some(keys) = self.published_mock_keys.borrow().as_ref() {
                return Ok(keys.clone());
            }
        }
        let keys = if let Some(seed) = self.whole_tree_published_keys.as_ref() {
            if seed.is_empty() {
                Rc::new(resolve_published_mock_keys(self)?)
            } else {
                seed.clone()
            }
        } else {
            Rc::new(resolve_published_mock_keys(self)?)
        };
        *self.published_mock_keys.borrow_mut() = Some(keys.clone());
        Ok(keys)
    }

    fn governed_services(&self) -> InterpResult<Rc<std::collections::HashSet<String>>> {
        {
            if let Some(services) = self.governed_services.borrow().as_ref() {
                return Ok(services.clone());
            }
        }
        let published = self.published_mock_keys()?;
        let services: std::collections::HashSet<String> = published
            .iter()
            .filter_map(|k| k.rsplit_once('.').map(|(svc, _)| svc.to_string()))
            .collect();
        let services = Rc::new(services);
        *self.governed_services.borrow_mut() = Some(services.clone());
        Ok(services)
    }

    fn si(&self) -> Rc<HashMap<String, Rc<NewlineIndex>>> {
        self.source_indices.clone()
    }

    pub(crate) fn source_indices(&self) -> Rc<HashMap<String, Rc<NewlineIndex>>> {
        self.si()
    }

    fn lookup_fn(&self, name: &str) -> Option<&Rc<Node>> {
        self.fn_nodes.get(name)
    }
}

pub fn run(
    graph: &ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    entry_fn: &str,
) -> InterpResult<Value> {
    run_with_options(graph, source_indices, entry_fn, ExecutionMode::Wet, true)
}

pub fn run_with_options(
    graph: &ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    entry_fn: &str,
    execution_mode: ExecutionMode,
    eager_data_env: bool,
) -> InterpResult<Value> {
    let ctx = InterpContext::new(graph, source_indices, execution_mode);
    run_in_context(&ctx, entry_fn, eager_data_env)
}

pub fn run_in_context(
    ctx: &InterpContext,
    entry_fn: &str,
    eager_data_env: bool,
) -> InterpResult<Value> {
    with_active_ctx(ctx, || {
        let item_node = ctx
            .lookup_fn(entry_fn)
            .ok_or_else(|| InterpError::NoMainFunction)?
            .clone();

        let env = if eager_data_env {
            build_initial_env(ctx)?
        } else {
            Env::empty()
        };

        with_lexical_base_env(&env, || call_function(ctx, &item_node, &[], &env))
    })
}

pub fn run_in_context_with_args(
    ctx: &InterpContext,
    entry_fn: &str,
    args: &[(Option<String>, Value)],
    eager_data_env: bool,
) -> InterpResult<Value> {
    with_active_ctx(ctx, || {
        let item_node = ctx
            .lookup_fn(entry_fn)
            .ok_or(InterpError::NoMainFunction)?
            .clone();
        let env = if eager_data_env {
            build_initial_env(ctx)?
        } else {
            Env::empty()
        };
        with_lexical_base_env(&env, || call_function(ctx, &item_node, args, &env))
    })
}

/// Peak parent-chain depth observed across `call_function` frames in the last
/// `run_in_context*` invocation (test witness for lexical-base scoping).
#[cfg(any(test, feature = "interp_test_witness"))]
pub fn call_env_depth_peak_snapshot() -> usize {
    CALL_ENV_DEPTH_PEAK.with(|peak| peak.get())
}

fn build_initial_env(ctx: &InterpContext) -> InterpResult<Rc<Env>> {
    let mut bindings = HashMap::new();
    for (name, info) in ctx.item_registry.iter() {
        if info.kind == ItemKind::DataItem {
            if let Some(node) = ctx.lookup_fn(name) {
                if let Some(ref body) = node.body {
                    let val = eval_expr(body, &Env::empty(), ctx)?;
                    bindings.insert(ctx.sym(name), val);
                }
            }
        }
    }
    Ok(Env::extend(&Env::empty(), bindings))
}

pub fn eval_data_initializer_values(ctx: &InterpContext) -> InterpResult<Vec<Value>> {
    let mut out = Vec::new();
    for (name, info) in ctx.item_registry.iter() {
        if info.kind == ItemKind::DataItem {
            if let Some(node) = ctx.lookup_fn(name) {
                if let Some(ref body) = node.body {
                    out.push(eval_expr(body, &Env::empty(), ctx)?);
                }
            }
        }
    }
    Ok(out)
}

pub fn eval_data_item_value(ctx: &InterpContext, item_name: &str) -> InterpResult<Option<Value>> {
    let Some(info) = ctx.item_registry.get(item_name) else {
        return Ok(None);
    };
    if info.kind != ItemKind::DataItem {
        return Ok(None);
    }
    let Some(node) = ctx.lookup_fn(item_name) else {
        return Ok(None);
    };
    let Some(body) = node.body.as_ref() else {
        return Ok(None);
    };
    Ok(Some(eval_expr(body, &Env::empty(), ctx)?))
}

thread_local! {
    static CALL_DEPTH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Bounded execution (§4): a call chain deeper than this is a typed, located
/// refusal naming the frontier function — never a host stack overflow, which
/// aborts the whole process and takes every later witness's measurement with it
/// (measured: a cycle inside live_deploy script assembly under census-resolved
/// bare names killed batch-2 at entry 214/619). Genuine deep-but-terminating
/// recursion lives under `stacker::maybe_grow` guards; 100_000 interpreter
/// frames is far past any legitimate corpus chain.
const CALL_DEPTH_LIMIT: u32 = 100_000;

fn call_function(
    ctx: &InterpContext,
    fn_node: &Rc<Node>,
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
) -> InterpResult<Value> {
    let depth = CALL_DEPTH.with(|d| {
        let v = d.get() + 1;
        d.set(v);
        v
    });
    let result = call_function_guarded(ctx, fn_node, args, env, depth);
    CALL_DEPTH.with(|d| d.set(d.get() - 1));
    result
}

fn call_function_guarded(
    ctx: &InterpContext,
    fn_node: &Rc<Node>,
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
    depth: u32,
) -> InterpResult<Value> {
    if depth > CALL_DEPTH_LIMIT {
        return Err(InterpError::TypeError {
            msg: format!(
                "call depth exceeded {} at fn '{}' — unbounded recursion (a bare-name \
                 resolution cycle, or a genuinely divergent chain); refused, never a \
                 host stack overflow",
                CALL_DEPTH_LIMIT, fn_node.name
            ),
        });
    }
    // Grow the host stack in slices so DEEP-but-bounded chains below the limit
    // never abort the process between guard checks.
    stacker::maybe_grow(256 * 1024, 8 * 1024 * 1024, || {
        call_function_dispatch(ctx, fn_node, args, env)
    })
}

fn call_function_dispatch(
    ctx: &InterpContext,
    fn_node: &Rc<Node>,
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
) -> InterpResult<Value> {
    if !residual_hunt_forensics_enabled() {
        return call_function_inner(ctx, fn_node, args, env);
    }
    let started = std::time::Instant::now();
    DAG_PROF_CHILD_STACK.with(|s| s.borrow_mut().push(0));
    let result = call_function_inner(ctx, fn_node, args, env);
    let elapsed = started.elapsed().as_nanos() as u64;
    let child_nanos = DAG_PROF_CHILD_STACK.with(|s| s.borrow_mut().pop().unwrap_or(0));
    DAG_PROF_CHILD_STACK.with(|s| {
        if let Some(parent) = s.borrow_mut().last_mut() {
            *parent += elapsed;
        }
    });
    record_dag_fn_self_time(&fn_node.name, elapsed.saturating_sub(child_nanos));
    result
}

fn call_function_inner(
    ctx: &InterpContext,
    fn_node: &Rc<Node>,
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
) -> InterpResult<Value> {
    if let Some(result) = try_v2_std_collection_map_primitive_grounding(ctx, fn_node, args) {
        return result;
    }

    let body = fn_node
        .body
        .as_ref()
        .ok_or_else(|| InterpError::TypeError {
            msg: format!("'{}' has no body", fn_node.name),
        })?;

    let _dag_fn_guard = DagFnGuard::enter(fn_node.name.as_str());

    let cached_params = {
        let key = Rc::as_ptr(fn_node) as usize;
        // Bind the lookup to a local so the immutable borrow is released before the
        // None branch takes a mutable borrow (an `if let ...borrow()` would hold it through `else`).
        let hit = ctx.param_name_cache.borrow().get(&key).cloned();
        if let Some(c) = hit {
            c
        } else {
            // Each param's authored name is sliced once into `all`; `filtered` reuses it
            // (only the type-expr side is sliced again) rather than re-slicing the param name.
            let all: Vec<String> = fn_node
                .params
                .iter()
                .map(|p| authored_name_at(ctx.si(), p.clone()))
                .collect();
            let filtered: Vec<String> = fn_node
                .params
                .iter()
                .enumerate()
                .filter(|(i, p)| match p.children.first() {
                    Some(type_expr) => authored_name_at(ctx.si(), type_expr.clone()) != all[*i],
                    None => false,
                })
                .map(|(i, _)| all[i].clone())
                .collect();
            let c = Rc::new((filtered, all));
            ctx.param_name_cache.borrow_mut().insert(key, c.clone());
            c
        }
    };
    let param_names: &Vec<String> = &cached_params.0;
    let all_param_names: &Vec<String> = &cached_params.1;

    let mut bindings = HashMap::new();
    if !args.is_empty() {
        let mut positional_idx = 0;
        for (opt_name, val) in args {
            if let Some(name) = opt_name {
                // A caller label that names no declared parameter is a contract mismatch, not
                // an extra binding: inserting it would shadow nothing the body reads while the
                // real parameter stays unbound. Refuse here (typed, located) rather than let the
                // body fail later as `NoSuchVariable` — or silently compute when the stray label
                // happens to collide with another in-scope name.
                // The corpus marks a deliberately-unused parameter with a leading underscore
                // (`_ctx`, `_spelling`) or an anonymous `_`, and call sites label it WITHOUT the
                // underscore (`bash_fold_stmt_kind_tag_emit_transform(spelling: ..)` against
                // `(_spelling: String)`; the fold-step `(acc, _: Edge, child)` labelled `e:`).
                // That is the established idiom, and it is not a contract mismatch: the body
                // cannot read the parameter, so nothing is silently dropped. Accept `x` against
                // a declared `x`, `_x`, or `_`.
                let matches_param = |p: &String| {
                    p == name
                        || p == "_"
                        || p.strip_prefix('_').is_some_and(|stripped| stripped == name)
                };
                if !all_param_names.iter().any(matches_param) {
                    return Err(InterpError::CallContractMismatch {
                        callee: fn_node.name.clone(),
                        detail: format!(
                            "no parameter named '{}' (declared: [{}])",
                            name,
                            all_param_names.join(", ")
                        ),
                    });
                }
                bindings.insert(ctx.sym(name), val.clone());
            } else if positional_idx < param_names.len() {
                bindings.insert(ctx.sym(&param_names[positional_idx]), val.clone());
                positional_idx += 1;
            } else {
                // The pre-existing `else if` guard dropped surplus positional arguments on the
                // floor. Silently discarding an evaluated argument is the §5 absorbing arm.
                return Err(InterpError::CallContractMismatch {
                    callee: fn_node.name.clone(),
                    detail: format!(
                        "too many positional arguments: {} supplied, {} positional parameter(s) declared",
                        args.len(),
                        param_names.len()
                    ),
                });
            }
        }
    }

    for (i, param) in fn_node.params.iter().enumerate() {
        let pname = &all_param_names[i];
        if !bindings.contains_key(&ctx.sym(pname)) {
            if let Some(default_node) = param_node_default_value(param.clone()) {
                let default_val = eval_expr(&default_node, env, ctx)?;
                bindings.insert(ctx.sym(pname), default_val);
            }
        }
    }

    let call_env = Env::extend(&lexical_base_env(env), bindings);
    #[cfg(any(test, feature = "interp_test_witness"))]
    record_call_env_depth(&call_env);

    match eval_expr(body, &call_env, ctx) {
        Err(InterpError::EarlyReturn { value }) => Ok(value),
        other => other,
    }
}

/// Thread CPU time in nanoseconds — the metric the fast-lane eval budget is denominated in.
/// It advances only while THIS thread is actually running on a core, so it excludes both
/// blocking-I/O waits (a witness reading the live tree cold) and scheduler time-slicing (many
/// witnesses sharing cores under the adaptive governor). That is exactly the "assuming the
/// infra isn't the problem" clause of the operator's 5s rule: a genuine non-terminating eval
/// burns CPU and is still caught, while a bounded scan whose WALL time was inflated by infra is
/// not misclassified. On unix this reads `CLOCK_THREAD_CPUTIME_ID`; elsewhere (dev only — CI is
/// linux) it falls back to a process-monotonic wall clock. A clock error yields 0, which makes
/// the deadline under-count rather than fire spuriously (the witness still returns its real
/// Pass/Fail; the budget is a performance guard, not a correctness gate).
pub fn thread_cpu_nanos() -> u128 {
    #[cfg(unix)]
    {
        let mut ts = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        // SAFETY: `ts` is a valid, owned timespec; CLOCK_THREAD_CPUTIME_ID is always supported
        // on linux/macos. rc != 0 (unreachable there) falls through to 0.
        let rc = unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut ts) };
        if rc == 0 {
            return (ts.tv_sec as u128) * 1_000_000_000 + (ts.tv_nsec as u128);
        }
        0
    }
    #[cfg(not(unix))]
    {
        use std::sync::OnceLock;
        static START: OnceLock<std::time::Instant> = OnceLock::new();
        START
            .get_or_init(std::time::Instant::now)
            .elapsed()
            .as_nanos()
    }
}

fn eval_expr(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    if let Some((cpu_baseline_nanos, budget_ms)) = ctx.eval_deadline.get() {
        let stride = ctx.eval_deadline_stride.get().wrapping_add(1);
        ctx.eval_deadline_stride.set(stride);
        if stride % 4096 == 0 {
            let elapsed_ms =
                (thread_cpu_nanos().saturating_sub(cpu_baseline_nanos) / 1_000_000) as u64;
            if elapsed_ms > budget_ms {
                return Err(InterpError::EvalBudgetExceeded {
                    elapsed_ms,
                    budget_ms,
                });
            }
        }
    }
    if !eval_profile_enabled() {
        return stacker::maybe_grow(64 * 1024, 2 * 1024 * 1024, || {
            eval_expr_inner(node, env, ctx)
        });
    }

    let idx = expr_variant_index(&node.expr_data);
    EVAL_COUNTS.with(|c| c.borrow_mut()[idx] += 1);
    let saved_children = CHILD_NANOS.replace(0);
    let start = Instant::now();
    let result = stacker::maybe_grow(64 * 1024, 2 * 1024 * 1024, || {
        eval_expr_inner(node, env, ctx)
    });
    let gross = start.elapsed().as_nanos();
    let children = CHILD_NANOS.get();
    let self_time = gross.saturating_sub(children);
    if let Some(subject) = ACTIVE_SUBJECT.with(|s| s.borrow().clone()) {
        SUBJECT_SELF_NANOS.with(|m| {
            *m.borrow_mut().entry(subject).or_insert(0) += self_time;
        });
    }
    EVAL_SELF_NANOS.with(|c| c.borrow_mut()[idx] += self_time);
    CHILD_NANOS.set(saved_children + gross);
    result
}

fn eval_expr_inner(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let si = ctx.si();
    match (*node.expr_data).clone() {
        ExprData::ExprLiteral { value } => eval_literal(&value),

        ExprData::ExprVar { binding_kind } => eval_var(node, binding_kind.as_deref(), env, ctx),

        ExprData::ExprBinOp { op, .. } => {
            let left = eval_expr(&binop_left(node.clone()), env, ctx)?;
            let right = eval_expr(&binop_right(node.clone()), env, ctx)?;
            eval_binop(&op, left, right, ctx)
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
            let items: Vec<Value> = node
                .children
                .iter()
                .map(|child| eval_expr(child, env, ctx))
                .collect::<InterpResult<_>>()?;
            Ok(list_value((items)))
        }

        ExprData::ExprLambda => {
            let param_names: Vec<Symbol> = lambda_param_names_at(node.clone(), si)
                .iter()
                .map(|name| ctx.sym(name))
                .collect();
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

        ExprData::ExprError { message, .. } => Err(InterpError::TypeError {
            // Located: an inference-side error node reaching evaluation is a
            // fail-open seam (typed error without a blocking diagnostic); the
            // span is the only thread back to the source.
            msg: format!(
                "{message} at {}:{}-{}",
                node.span.file, node.span.start, node.span.end
            ),
        }),

        ExprData::NoExprData => Ok(Value::Unit),
    }
}

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
        LiteralValue::LitSymbol { value } => Ok(Value::Str(value.clone())),
        LiteralValue::LitNull => Ok(Value::Null),
    }
}

fn eval_var(
    node: &Rc<Node>,
    binding_kind: Option<&VarBindingKind>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    // Resolve and intern this ExprVar's name once, then reuse the Symbol on every eval
    // (skips the per-eval source-span slice in expr_var_name_at and the ctx.sym re-intern).
    let key = Rc::as_ptr(node) as usize;
    let sym = {
        let hit = ctx.var_sym_cache.borrow().get(&key).copied();
        match hit {
            Some(s) => s,
            None => {
                let name = expr_var_name_at(node.clone(), ctx.si());
                let s = ctx.sym(&name);
                ctx.var_sym_cache.borrow_mut().insert(key, s);
                s
            }
        }
    };

    if ctx.sym_eq(sym, "none") || ctx.sym_eq(sym, "None") {
        return Ok(Value::Null);
    }
    if ctx.sym_eq(sym, "true") {
        return Ok(Value::Bool(true));
    }
    if ctx.sym_eq(sym, "false") {
        return Ok(Value::Bool(false));
    }

    if let Some(VarBindingKind::VariantValueBinding { parent_enum }) = binding_kind {
        if ctx.sym_eq(sym, "Zero") {
            return Ok(Value::Int(0));
        }
        let vn = ctx.resolve(sym);
        return Ok(Value::Variant {
            type_name: ctx.sym(parent_enum),
            variant_name: ctx.sym(vn.rsplit('.').next().unwrap_or(&vn)),
            fields: Rc::new(vec![]),
        });
    }

    if let Some(val) = env.lookup(sym) {
        return Ok(val.clone());
    }

    // Slow path (not a bound variable): materialize the name string for the registry lookup.
    let name = ctx.resolve(sym);
    if let Some(info) = v1_rt::map_get(&ctx.item_registry, name.clone()) {
        if info.kind == ItemKind::DataItem {
            if let Some(fn_node) = ctx.lookup_fn(&name) {
                if let Some(ref body) = fn_node.body {
                    if let ExprData::ExprVar { .. } = &*body.expr_data {
                        if expr_var_name_at(body.clone(), ctx.si()) == name {
                            return Ok(Value::Str(name));
                        }
                    }
                    let key = Rc::as_ptr(fn_node) as usize;
                    if let Some(v) = ctx.data_cache.borrow().get(&key).cloned() {
                        return Ok(v);
                    }
                    let v = eval_expr(body, &Env::empty(), ctx)?;
                    ctx.data_cache.borrow_mut().insert(key, v.clone());
                    return Ok(v);
                }
            }
        }
        if matches!(info.kind, ItemKind::FuncItem | ItemKind::FnItem) {
            if let Some(fn_node) = ctx.lookup_fn(&name) {
                return Ok(Value::Fn {
                    node: fn_node.clone(),
                });
            }
        }
    }

    // Qualified module.member value projection: the flat item_registry is keyed by
    // bare name, so dotted references resolve through the qualified fn_nodes keys
    // and classify by the node itself (mirrors the service-item note above).
    if name.contains('.') {
        if let Some(fn_node) = ctx.lookup_fn(&name) {
            match item_kind(fn_node.clone()) {
                ItemKind::DataItem => {
                    if let Some(ref body) = fn_node.body {
                        let key = Rc::as_ptr(fn_node) as usize;
                        if let Some(v) = ctx.data_cache.borrow().get(&key).cloned() {
                            return Ok(v);
                        }
                        let v = eval_expr(body, &Env::empty(), ctx)?;
                        ctx.data_cache.borrow_mut().insert(key, v.clone());
                        return Ok(v);
                    }
                }
                ItemKind::FuncItem | ItemKind::FnItem => {
                    return Ok(Value::Fn {
                        node: fn_node.clone(),
                    });
                }
                _ => {}
            }
        }
        // Qualified unit-variant value (module.Variant): the compile-side projection
        // resolved the owning coproduct into this node's inferred — reuse that binding
        // rather than re-looking the name up (single authority; fail-closed otherwise).
        if let Some(inf) = node.inferred.as_deref() {
            if let crate::v1_std_core::InferredNode::Resolved { node: ty, .. } = inf {
                if ty.connective == crate::v1_std_core::Connective::Disj {
                    let last = name.rsplit('.').next().unwrap_or(&name).to_string();
                    let arm = ty
                        .children
                        .iter()
                        .find(|c| authored_name_at(ctx.si(), (*c).clone()) == last);
                    if let Some(arm) = arm {
                        if arm.children.is_empty() {
                            let ty_name = authored_name_at(ctx.si(), ty.clone());
                            return Ok(Value::Variant {
                                type_name: ctx.sym(&ty_name),
                                variant_name: ctx.sym(&last),
                                fields: Rc::new(vec![]),
                            });
                        }
                    }
                }
            }
        }
    }

    Err(InterpError::NoSuchVariable { name })
}

fn eval_binop(op: &BinOp, left: Value, right: Value, ctx: &InterpContext) -> InterpResult<Value> {
    if matches!(op, BinOp::NullCoalesce) {
        return Ok(if matches!(left, Value::Null) {
            right
        } else {
            left
        });
    }

    if matches!(op, BinOp::Add) {
        if let (Value::Str(a), Value::Str(b)) = (&left, &right) {
            return Ok(Value::Str(format!("{}{}", a, b)));
        }
    }

    if matches!(op, BinOp::Add) {
        let record_push = |copied: usize| {
            let mut counters = ctx.mutation_counters.borrow_mut();
            counters.list_push_calls += 1;
            counters.list_push_items_copied += copied as u64;
        };
        match (&left, &right) {
            (l, Value::Str(s)) => {
                // String grounding: a string-like left operand concatenated with
                // a native String realizes as a native `Value::Str`, never a
                // mixed `[codepoint.., Str]` list (model↔realization).
                if let Some(ls) = free_monoid_to_string(l) {
                    return Ok(Value::Str(format!("{}{}", ls, s)));
                }
                if let Some(mut result) = free_monoid_to_vec(l) {
                    if let Some(detail) = string_realization_straddle_detail(l, &result) {
                        return Err(InterpError::StringRealizationStraddle { detail });
                    }
                    record_push(result.len());
                    result.push(Value::Str(s.clone()));
                    return Ok(list_value((result)));
                }
            }
            (Value::Str(s), r) => {
                if let Some(rs) = free_monoid_to_string(r) {
                    return Ok(Value::Str(format!("{}{}", s, rs)));
                }
                if let Some(result) = free_monoid_to_vec(r) {
                    if let Some(detail) = string_realization_straddle_detail(r, &result) {
                        return Err(InterpError::StringRealizationStraddle { detail });
                    }
                    record_push(result.len());
                    let mut out = vec![Value::Str(s.clone())];
                    out.extend(result);
                    return Ok(list_value((out)));
                }
            }
            _ => {
                if let (Some(mut a), Some(b)) =
                    (free_monoid_to_vec(&left), free_monoid_to_vec(&right))
                {
                    let mut counters = ctx.mutation_counters.borrow_mut();
                    counters.list_concat_calls += 1;
                    counters.list_concat_items_copied += (a.len() + b.len()) as u64;
                    drop(counters);
                    a.extend(b);
                    return Ok(list_value((a)));
                }
            }
        }
    }

    if matches!(op, BinOp::Eq | BinOp::Ne) {
        let equal = left == right;
        if !equal {
            if let Some(detail) = cross_representation_numeric_straddle(&left, &right) {
                return Err(InterpError::CrossRepresentationEquality { detail });
            }
        }
        let result = if matches!(op, BinOp::Eq) {
            equal
        } else {
            !equal
        };
        return Ok(Value::Bool(result));
    }

    if matches!(op, BinOp::And) {
        return Ok(Value::Bool(left.is_truthy() && right.is_truthy()));
    }
    if matches!(op, BinOp::Or) {
        return Ok(Value::Bool(left.is_truthy() || right.is_truthy()));
    }

    match (&left, &right) {
        (Value::Int(a), Value::Int(b)) => eval_int_binop(op, *a, *b),
        (Value::Float(a), Value::Float(b)) => eval_float_binop(op, *a, *b),
        (Value::Int(a), Value::Float(b)) => eval_float_binop(op, *a as f64, *b),
        (Value::Float(a), Value::Int(b)) => eval_float_binop(op, *a, *b as f64),
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
            msg: format!(
                "cannot apply {:?} to {} and {}",
                op,
                left.type_label(),
                right.type_label()
            ),
        }),
    }
}

fn cross_representation_numeric_straddle(a: &Value, b: &Value) -> Option<String> {
    match (a, b) {
        (Value::Int(_) | Value::Float(_), v @ Value::Variant { .. })
        | (v @ Value::Variant { .. }, Value::Int(_) | Value::Float(_))
            if free_monoid_to_vec(v).is_none() =>
        {
            Some(format!(
                "{} vs {} — a number and its coproduct (Nat Zero/Succ) encoding are \
                 two representations of one value; Value::eq cannot decide them, so \
                 `==` would silently fabricate `false` (DESIGN §5). Ground the \
                 primitive into its realization to compare (DESIGN §1/§2/§7).",
                describe_repr(a),
                describe_repr(b),
            ))
        }
        (
            Value::Bool(_),
            Value::Variant {
                variant_name,
                fields,
                ..
            },
        )
        | (
            Value::Variant {
                variant_name,
                fields,
                ..
            },
            Value::Bool(_),
        ) if fields.is_empty()
            && matches!(resolve_sym(*variant_name).as_str(), "True" | "False") =>
        {
            Some(format!(
                "{} vs {} — a native Bool and its True/False coproduct encoding are \
                 two representations of one value; Value::eq cannot decide them, so \
                 `==` would silently fabricate `false` (DESIGN §5). Ground the \
                 primitive into its realization to compare (DESIGN §1/§2/§7).",
                describe_repr(a),
                describe_repr(b),
            ))
        }
        (
            Value::Int(_)
            | Value::Float(_)
            | Value::Bool(_)
            | Value::Null
            | Value::Unit
            | Value::Str(_),
            Value::Int(_)
            | Value::Float(_)
            | Value::Bool(_)
            | Value::Null
            | Value::Unit
            | Value::Str(_),
        ) => None,
        (
            Value::Variant {
                variant_name: an,
                fields: af,
                ..
            },
            Value::Variant {
                variant_name: bn,
                fields: bf,
                ..
            },
        ) if an == bn => fields_numeric_straddle(af, bf),
        (Value::Record { fields: af, .. }, Value::Record { fields: bf, .. }) => {
            fields_numeric_straddle(af, bf)
        }
        (Value::List(av), Value::List(bv)) => av
            .iter()
            .zip(bv.iter())
            .filter(|(x, y)| x != y)
            .find_map(|(x, y)| cross_representation_numeric_straddle(x, y)),
        _ => match (free_monoid_to_vec(a), free_monoid_to_vec(b)) {
            (Some(av), Some(bv)) => av
                .iter()
                .zip(bv.iter())
                .filter(|(x, y)| x != y)
                .find_map(|(x, y)| cross_representation_numeric_straddle(x, y)),
            _ => None,
        },
    }
}

fn fields_numeric_straddle(af: &[(Symbol, Value)], bf: &[(Symbol, Value)]) -> Option<String> {
    af.iter()
        .filter_map(|(k, av)| fields_get(bf, *k).map(|bv| (av, bv)))
        .filter(|(av, bv)| av != bv)
        .find_map(|(av, bv)| cross_representation_numeric_straddle(av, bv))
}

fn describe_repr(v: &Value) -> String {
    match v {
        Value::Int(n) => format!("native Int({})", n),
        Value::Float(f) => format!("native Float({})", f),
        Value::Variant { variant_name, .. } => {
            format!("coproduct Variant `{}`", resolve_sym(*variant_name))
        }
        other => other.type_label().to_string(),
    }
}

fn eval_int_binop(op: &BinOp, a: i64, b: i64) -> InterpResult<Value> {
    match op {
        BinOp::Add => Ok(Value::Int(a + b)),
        BinOp::Sub => Ok(Value::Int(a - b)),
        BinOp::Mul => Ok(Value::Int(a * b)),
        BinOp::Div => {
            if b == 0 {
                return Err(InterpError::DivisionByZero);
            }
            Ok(Value::Int(a / b))
        }
        BinOp::Mod => {
            if b == 0 {
                return Err(InterpError::DivisionByZero);
            }
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
            if b == 0.0 {
                return Err(InterpError::DivisionByZero);
            }
            Ok(Value::Float(a / b))
        }
        BinOp::Mod => {
            if b == 0.0 {
                return Err(InterpError::DivisionByZero);
            }
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

fn eval_if(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
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

fn eval_let(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let name = let_binding_name_at(node.clone(), ctx.si());
    let val = eval_expr(&let_value(node.clone()), env, ctx)?;
    let new_env = Env::with_binding(env, ctx.sym(&name), val);
    match let_body(node.clone()) {
        Some(body) => eval_expr(&body, &new_env, ctx),
        None => Ok(Value::Unit),
    }
}

fn eval_block(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let stmts = block_stmts(node.clone());
    let mut current_env = env.clone();
    let mut last_val = Value::Unit;

    for stmt in stmts.iter() {
        match (*stmt.expr_data).clone() {
            ExprData::ExprLet => {
                let name = let_binding_name_at(stmt.clone(), ctx.si());
                let val = eval_expr(&let_value(stmt.clone()), &current_env, ctx)?;
                current_env = Env::with_binding(&current_env, ctx.sym(&name), val.clone());
                last_val = val;
            }
            _ => {
                last_val = eval_expr(stmt, &current_env, ctx)?;
            }
        }
    }

    Ok(last_val)
}

fn eval_match(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let scrutinee_val = eval_expr(&match_scrutinee(node.clone()), env, ctx)?;
    let arms = match_arm_nodes(node.clone());

    for arm in arms.iter() {
        let pattern = arm_pattern(arm.clone());
        if let Some(bindings) = match_pattern(&pattern, &scrutinee_val, ctx) {
            let arm_env = Env::extend(env, bindings);
            return eval_expr(&arm_body(arm.clone()), &arm_env, ctx);
        }
    }

    Err(InterpError::PatternMatchFailure {
        value: format!("{}", scrutinee_val),
    })
}

fn char_value(c: char) -> Value {
    Value::Int(c as i64)
}

fn native_map_absent_diagnostic_value(ctx: &InterpContext) -> Value {
    let anchor = Value::Record {
        type_name: ctx.sym("LocusAnchor"),
        fields: Rc::new(vec![(
            ctx.sym("at"),
            Value::Str("map_lookup_port".to_string()),
        )]),
    };
    let locus = Value::Variant {
        type_name: ctx.sym("Locus"),
        variant_name: ctx.sym("PortLocus"),
        fields: Rc::new(vec![(ctx.sym("anchor"), anchor)]),
    };
    let correction = Value::Variant {
        type_name: ctx.sym("Correction"),
        variant_name: ctx.sym("Unavailable"),
        fields: Rc::new(vec![(
            ctx.sym("reason"),
            Value::Variant {
                type_name: ctx.sym("NoCorrectionReason"),
                variant_name: ctx.sym("ExternalContractUnknown"),
                fields: Rc::new(vec![]),
            },
        )]),
    };
    Value::Record {
        type_name: ctx.sym("Diagnostic"),
        fields: Rc::new(sorted_fields(vec![
            (ctx.sym("at"), locus),
            (ctx.sym("correction"), correction),
            (ctx.sym("reason"), Value::Str("map_key_absent".to_string())),
        ])),
    }
}

fn match_pattern(
    pattern: &MatchPattern,
    value: &Value,
    ctx: &InterpContext,
) -> Option<HashMap<Symbol, Value>> {
    match pattern {
        MatchPattern::Wildcard => Some(HashMap::new()),

        MatchPattern::Bind { name } => {
            let mut bindings = HashMap::new();
            bindings.insert(ctx.sym(name), value.clone());
            Some(bindings)
        }

        MatchPattern::LitPattern { value: lit } => {
            let lit_val = eval_literal(lit).ok()?;
            if *value == lit_val {
                Some(HashMap::new())
            } else {
                None
            }
        }

        MatchPattern::VariantPattern {
            name,
            parent_enum,
            field_bindings,
        } => {
            // Kernel-optional / witness raw representation (value-or-Null):
            // the `_ if Present+Optional` / `_ if Holds+Witness` unwrap arms
            // below the kind-specific arms were UNREACHABLE for Record/List/
            // Str/Int payloads — Value::Record etc. match their kind arm
            // first and return None from inside it, so
            // `match xs |> first { Present { value: t } => ... }` failed
            // non-exhaustive on any record element (pre-existing on main;
            // located via the interpreted-parse suite reds). Hoisted here
            // verbatim; Variant payloads are excluded so the Variant arm's
            // inline raw-value handling stays authoritative.
            if name == "Present"
                && parent_enum.as_deref() == Some("Optional")
                && !matches!(value, Value::Null)
                && !matches!(value, Value::Variant { .. })
            {
                let mut bindings = HashMap::new();
                for fb in field_bindings.iter() {
                    let fb_pat = field_binding_pattern(fb.clone());
                    let sub_bindings = match_pattern(&fb_pat, value, ctx)?;
                    bindings.extend(sub_bindings);
                }
                return Some(bindings);
            }
            if name == "Holds"
                && parent_enum.as_deref() == Some("Witness")
                && !matches!(value, Value::Null)
                && !matches!(value, Value::Variant { .. })
            {
                let mut bindings = HashMap::new();
                for fb in field_bindings.iter() {
                    let fb_pat = field_binding_pattern(fb.clone());
                    let sub_bindings = match_pattern(&fb_pat, value, ctx)?;
                    bindings.extend(sub_bindings);
                }
                return Some(bindings);
            }
            match value {
                Value::Variant {
                    variant_name,
                    fields,
                    ..
                } => {
                    if name == "Holds"
                        && parent_enum.as_deref() == Some("Witness")
                        && *variant_name != ctx.sym("Holds")
                        && *variant_name != ctx.sym("Violates")
                    {
                        let mut bindings = HashMap::new();
                        for fb in field_bindings.iter() {
                            let fb_pat = field_binding_pattern(fb.clone());
                            let sub_bindings = match_pattern(&fb_pat, value, ctx)?;
                            bindings.extend(sub_bindings);
                        }
                        return Some(bindings);
                    }
                    if name == "Present"
                        && parent_enum.as_deref() == Some("Optional")
                        && *variant_name != ctx.sym("Present")
                        && *variant_name != ctx.sym("Absent")
                    {
                        let mut bindings = HashMap::new();
                        for fb in field_bindings.iter() {
                            let fb_pat = field_binding_pattern(fb.clone());
                            let sub_bindings = match_pattern(&fb_pat, value, ctx)?;
                            bindings.extend(sub_bindings);
                        }
                        return Some(bindings);
                    }
                    if *variant_name != ctx.sym(name) {
                        // Qualified PATTERN spellings (module.Variant) carry the containment
                        // path; variant identity is the bare arm name, normalized at value
                        // construction — so only the pattern side needs the last segment.
                        let pat_last = name.rsplit('.').next().unwrap_or(name);
                        if *variant_name != ctx.sym(pat_last) {
                            return None;
                        }
                    }
                    let mut bindings = HashMap::new();
                    for fb in field_bindings.iter() {
                        let field_name =
                            field_binding_name_at(fb.clone(), ctx.source_indices.clone());
                        let fb_pat = field_binding_pattern(fb.clone());
                        let field_val = fields_get(fields, ctx.sym(&field_name))
                            .cloned()
                            .unwrap_or(Value::Null);
                        let sub_bindings = match_pattern(&fb_pat, &field_val, ctx)?;
                        bindings.extend(sub_bindings);
                    }
                    Some(bindings)
                }
                Value::Record { type_name, fields } => {
                    if *type_name != ctx.sym(name) {
                        return None;
                    }
                    let mut bindings = HashMap::new();
                    for fb in field_bindings.iter() {
                        let field_name =
                            field_binding_name_at(fb.clone(), ctx.source_indices.clone());
                        let fb_pat = field_binding_pattern(fb.clone());
                        let field_val = fields_get(fields, ctx.sym(&field_name))
                            .cloned()
                            .unwrap_or(Value::Null);
                        let sub_bindings = match_pattern(&fb_pat, &field_val, ctx)?;
                        bindings.extend(sub_bindings);
                    }
                    Some(bindings)
                }
                Value::List(items) => match name.as_str() {
                    "Empty" => {
                        if items.is_empty() {
                            Some(HashMap::new())
                        } else {
                            None
                        }
                    }
                    "Cons" => {
                        if items.is_empty() {
                            None
                        } else {
                            record_list_cons_tail_split(items.len());
                            let head = items[0].clone();
                            let tail = {
                                let mut rest = (**items).clone();
                                list_value(rest.split_off(1))
                            };
                            let mut bindings = HashMap::new();
                            for fb in field_bindings.iter() {
                                let field_name =
                                    field_binding_name_at(fb.clone(), ctx.source_indices.clone());
                                let fb_pat = field_binding_pattern(fb.clone());
                                let field_val = match field_name.as_str() {
                                    "head" => head.clone(),
                                    "tail" => tail.clone(),
                                    _ => return None,
                                };
                                let sub_bindings = match_pattern(&fb_pat, &field_val, ctx)?;
                                bindings.extend(sub_bindings);
                            }
                            Some(bindings)
                        }
                    }
                    _ => None,
                },
                Value::Str(s) if name == "Empty" || name == "Cons" => match name.as_str() {
                    "Empty" => {
                        if s.is_empty() {
                            Some(HashMap::new())
                        } else {
                            None
                        }
                    }
                    "Cons" => {
                        let mut chars = s.chars();
                        match chars.next() {
                            None => None,
                            Some(c) => {
                                let head = char_value(c);
                                let tail = Value::Str(chars.as_str().to_string());
                                let mut bindings = HashMap::new();
                                for fb in field_bindings.iter() {
                                    let field_name = field_binding_name_at(
                                        fb.clone(),
                                        ctx.source_indices.clone(),
                                    );
                                    let fb_pat = field_binding_pattern(fb.clone());
                                    let field_val = match field_name.as_str() {
                                        "head" => head.clone(),
                                        "tail" => tail.clone(),
                                        _ => return None,
                                    };
                                    let sub_bindings = match_pattern(&fb_pat, &field_val, ctx)?;
                                    bindings.extend(sub_bindings);
                                }
                                Some(bindings)
                            }
                        }
                    }
                    _ => None,
                },
                Value::Int(n) if name == "Zero" || name == "Succ" => match name.as_str() {
                    "Zero" => {
                        if *n == 0 {
                            Some(HashMap::new())
                        } else {
                            None
                        }
                    }
                    "Succ" => {
                        if *n <= 0 {
                            None
                        } else {
                            let mut bindings = HashMap::new();
                            for fb in field_bindings.iter() {
                                let field_name =
                                    field_binding_name_at(fb.clone(), ctx.source_indices.clone());
                                let fb_pat = field_binding_pattern(fb.clone());
                                let field_val = match field_name.as_str() {
                                    "prev" => Value::Int(n - 1),
                                    _ => return None,
                                };
                                let sub_bindings = match_pattern(&fb_pat, &field_val, ctx)?;
                                bindings.extend(sub_bindings);
                            }
                            Some(bindings)
                        }
                    }
                    _ => None,
                },
                Value::Null if name == "Violates" && parent_enum.as_deref() == Some("Witness") => {
                    let mut bindings = HashMap::new();
                    for fb in field_bindings.iter() {
                        let field_name =
                            field_binding_name_at(fb.clone(), ctx.source_indices.clone());
                        let fb_pat = field_binding_pattern(fb.clone());
                        let field_val = match field_name.as_str() {
                            "diagnostic" => native_map_absent_diagnostic_value(ctx),
                            _ => return None,
                        };
                        let sub_bindings = match_pattern(&fb_pat, &field_val, ctx)?;
                        bindings.extend(sub_bindings);
                    }
                    Some(bindings)
                }
                Value::Null if name == "None" && parent_enum.as_deref() == Some("Diagnostics") => {
                    Some(HashMap::new())
                }
                Value::Null if name == "Absent" && parent_enum.as_deref() == Some("Optional") => {
                    Some(HashMap::new())
                }
                _ if name == "Present" && parent_enum.as_deref() == Some("Optional") => {
                    if matches!(value, Value::Null) {
                        return None;
                    }
                    let mut bindings = HashMap::new();
                    for fb in field_bindings.iter() {
                        let fb_pat = field_binding_pattern(fb.clone());
                        let sub_bindings = match_pattern(&fb_pat, value, ctx)?;
                        bindings.extend(sub_bindings);
                    }
                    Some(bindings)
                }
                _ if name == "Holds" && parent_enum.as_deref() == Some("Witness") => {
                    if matches!(value, Value::Null) {
                        return None;
                    }
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

pub(crate) const STD_NODE_BRIDGE_FNS: &[&str] = &["resolve_type_node"];

pub(crate) const STD_LEXING_BRIDGE_FNS: &[&str] = &["symbol_intern_lexeme", "symbol_lexeme"];

pub(crate) const STD_QUALIFIED_NAME_BRIDGE_FNS: &[&str] = &["qualified_name_from_dotted_string"];

pub(crate) const STD_NODE_QUERY_BRIDGE_FNS: &[&str] = &["coproduct_nullary_inhabitants"];

pub(crate) const STD_CONCEPT_INDEX_BRIDGE_FNS: &[&str] = &["concept_decl_facts_live"];

pub(crate) const STD_FN_INDEX_BRIDGE_FNS: &[&str] = &[
    "fn_arrow_decl_facts_live",
    "fn_arrow_decl_substrate_is_whole_tree",
];

pub(crate) const CORPUS_DEPENDENCY_VIEW_BRIDGE_FNS: &[&str] =
    &["corpus_dependency_view_per_pr_substrate_refuse"];

pub(crate) const STD_DATA_INDEX_BRIDGE_FNS: &[&str] = &["data_init_decl_facts_live"];

pub(crate) const INERT_LENS_BRIDGE_FNS: &[&str] = &[
    "inert_lens_unreached_module_count",
    "inert_lens_top_level_module_count",
];

const STD_COLLECTION_MAP_GROUNDED_FNS: &[&str] =
    &["empty_map", "empty_map_primitive_delegate", "map_insert"];

const V2_STD_COLLECTION_MODULE: &str = "v2.std.collection";

pub fn std_node_bridge_fn_names() -> &'static [&'static str] {
    STD_NODE_BRIDGE_FNS
}

pub fn std_node_query_bridge_fn_names() -> &'static [&'static str] {
    STD_NODE_QUERY_BRIDGE_FNS
}

pub fn std_concept_index_bridge_fn_names() -> &'static [&'static str] {
    STD_CONCEPT_INDEX_BRIDGE_FNS
}

pub fn std_fn_index_bridge_fn_names() -> &'static [&'static str] {
    STD_FN_INDEX_BRIDGE_FNS
}

pub fn corpus_dependency_view_bridge_fn_names() -> &'static [&'static str] {
    CORPUS_DEPENDENCY_VIEW_BRIDGE_FNS
}

pub fn std_data_index_bridge_fn_names() -> &'static [&'static str] {
    STD_DATA_INDEX_BRIDGE_FNS
}

pub fn std_qualified_name_bridge_fn_names() -> &'static [&'static str] {
    STD_QUALIFIED_NAME_BRIDGE_FNS
}

fn is_v4_std_node_bridge_call(ctx: &InterpContext, func_name: &str) -> bool {
    if !STD_NODE_BRIDGE_FNS.contains(&func_name) {
        return false;
    }
    ctx.item_registry
        .get(func_name)
        .is_some_and(|info| info.module_name == "v2.std.node")
}

fn is_v4_std_node_query_bridge_call(ctx: &InterpContext, func_name: &str) -> bool {
    if !STD_NODE_QUERY_BRIDGE_FNS.contains(&func_name) {
        return false;
    }
    ctx.item_registry
        .get(func_name)
        .is_some_and(|info| info.module_name == "v2.std.node_query")
}

fn is_v4_std_concept_index_bridge_call(ctx: &InterpContext, func_name: &str) -> bool {
    if !STD_CONCEPT_INDEX_BRIDGE_FNS.contains(&func_name) {
        return false;
    }
    ctx.item_registry
        .get(func_name)
        .is_some_and(|info| info.module_name == "v2.std.concept_index")
}

fn is_v4_std_fn_index_bridge_call(ctx: &InterpContext, func_name: &str) -> bool {
    if !STD_FN_INDEX_BRIDGE_FNS.contains(&func_name) {
        return false;
    }
    ctx.item_registry
        .get(func_name)
        .is_some_and(|info| info.module_name == "v2.std.fn_index")
}

fn is_v4_corpus_dependency_view_bridge_call(ctx: &InterpContext, func_name: &str) -> bool {
    if !CORPUS_DEPENDENCY_VIEW_BRIDGE_FNS.contains(&func_name) {
        return false;
    }
    ctx.item_registry
        .get(func_name)
        .is_some_and(|info| info.module_name == "v2.lens.affected_set.corpus_dependency_view")
}

fn is_v4_std_data_index_bridge_call(ctx: &InterpContext, func_name: &str) -> bool {
    if !STD_DATA_INDEX_BRIDGE_FNS.contains(&func_name) {
        return false;
    }
    ctx.item_registry
        .get(func_name)
        .is_some_and(|info| info.module_name == "v2.std.data_index")
}

fn is_v4_std_lexing_bridge_call(ctx: &InterpContext, func_name: &str) -> bool {
    if !STD_LEXING_BRIDGE_FNS.contains(&func_name) {
        return false;
    }
    ctx.item_registry
        .get(func_name)
        .is_some_and(|info| info.module_name == "v2.std.compilers.lexing")
}

fn is_v4_std_qualified_name_bridge_call(ctx: &InterpContext, func_name: &str) -> bool {
    if !STD_QUALIFIED_NAME_BRIDGE_FNS.contains(&func_name) {
        return false;
    }
    ctx.item_registry
        .get(func_name)
        .is_some_and(|info| info.module_name == "v2.std.qualified_name")
}

fn is_v4_inert_lens_bridge_call(ctx: &InterpContext, func_name: &str) -> bool {
    if !INERT_LENS_BRIDGE_FNS.contains(&func_name) {
        return false;
    }
    ctx.item_registry
        .get(func_name)
        .is_some_and(|info| info.module_name == "v2.lens.inert_lens")
}

fn is_v2_std_collection_map_grounded_fn(ctx: &InterpContext, fn_node: &Rc<Node>) -> bool {
    if !STD_COLLECTION_MAP_GROUNDED_FNS.contains(&fn_node.name.as_str()) {
        return false;
    }
    ctx.item_registry
        .get(&fn_node.name)
        .is_some_and(|info| info.module_name == V2_STD_COLLECTION_MODULE)
}

fn try_v2_std_collection_map_primitive_grounding(
    ctx: &InterpContext,
    fn_node: &Rc<Node>,
    args: &[(Option<String>, Value)],
) -> Option<InterpResult<Value>> {
    if !is_v2_std_collection_map_grounded_fn(ctx, fn_node) {
        return None;
    }
    let builtin_name = match fn_node.name.as_str() {
        "empty_map_primitive_delegate" | "empty_map" => "empty_map",
        "map_insert" => "map_insert",
        _ => return None,
    };
    match eval_builtin(builtin_name, args, ctx) {
        Ok(Some(v)) => Some(Ok(v)),
        Ok(None) if builtin_name == "empty_map" => Some(Err(InterpError::TypeError {
            msg: format!(
                "{V2_STD_COLLECTION_MODULE}.{}: native HAMT primitive missing from eval_builtin (host misconfiguration)",
                fn_node.name
            ),
        })),
        Ok(None) => None,
        Err(e) => Some(Err(e)),
    }
}

fn eval_call(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let func_name = {
        let key = Rc::as_ptr(node) as usize;
        let hit = ctx.call_func_name_cache.borrow().get(&key).cloned();
        match hit {
            Some(s) => s,
            None => {
                let s = expr_call_func_at(node.clone(), ctx.si());
                ctx.call_func_name_cache.borrow_mut().insert(key, s.clone());
                s
            }
        }
    };
    record_call_frequency(&func_name);

    let arg_nodes = &node.children;

    let args: Vec<(Option<String>, Value)> = arg_nodes
        .iter()
        .filter(|arg_node| !arg_node.children.is_empty())
        .map(|arg_node| {
            let name = arg_name_at(arg_node.clone(), ctx.si());
            let val = eval_expr(&arg_value(arg_node.clone()), env, ctx)?;
            Ok((name, val))
        })
        .collect::<InterpResult<_>>()?;

    if is_v4_std_node_bridge_call(ctx, &func_name) {
        return match func_name.as_str() {
            "resolve_type_node" => crate::coproduct_reflection::eval_resolve_type_node(ctx, &args),
            _ => unreachable!("bridge fn set mismatch"),
        };
    }

    if is_v4_std_lexing_bridge_call(ctx, &func_name) {
        return match func_name.as_str() {
            "symbol_intern_lexeme" => {
                crate::coproduct_reflection::eval_symbol_intern_lexeme(ctx, &args)
            }
            "symbol_lexeme" => crate::coproduct_reflection::eval_symbol_lexeme(ctx, &args),
            _ => unreachable!("lexing bridge fn set mismatch"),
        };
    }

    if is_v4_std_qualified_name_bridge_call(ctx, &func_name) {
        return match func_name.as_str() {
            "qualified_name_from_dotted_string" => {
                crate::coproduct_reflection::eval_qualified_name_from_dotted_string(ctx, &args)
            }
            _ => unreachable!("qualified_name bridge fn set mismatch"),
        };
    }

    if is_v4_std_node_query_bridge_call(ctx, &func_name) {
        return match func_name.as_str() {
            "coproduct_nullary_inhabitants" => {
                crate::coproduct_reflection::eval_coproduct_nullary_inhabitants(ctx, &args)
            }
            _ => unreachable!("node_query bridge fn set mismatch"),
        };
    }

    if is_v4_std_concept_index_bridge_call(ctx, &func_name) {
        return match func_name.as_str() {
            "concept_decl_facts_live" => {
                crate::coproduct_reflection::eval_concept_decl_facts_live(ctx, &args)
            }
            _ => unreachable!("concept_index bridge fn set mismatch"),
        };
    }

    if is_v4_std_fn_index_bridge_call(ctx, &func_name) {
        return match func_name.as_str() {
            "fn_arrow_decl_facts_live" => {
                crate::coproduct_reflection::eval_fn_arrow_decl_facts_live(ctx, &args)
            }
            "fn_arrow_decl_substrate_is_whole_tree" => {
                crate::coproduct_reflection::eval_fn_arrow_decl_substrate_is_whole_tree(ctx, &args)
            }
            _ => unreachable!("fn_index bridge fn set mismatch"),
        };
    }

    if is_v4_corpus_dependency_view_bridge_call(ctx, &func_name) {
        return match func_name.as_str() {
            "corpus_dependency_view_per_pr_substrate_refuse" => {
                crate::coproduct_reflection::eval_corpus_dependency_view_per_pr_substrate_refuse(
                    ctx, &args,
                )
            }
            _ => unreachable!("corpus_dependency_view bridge fn set mismatch"),
        };
    }

    if is_v4_std_data_index_bridge_call(ctx, &func_name) {
        return match func_name.as_str() {
            "data_init_decl_facts_live" => {
                crate::coproduct_reflection::eval_data_init_decl_facts_live(ctx, &args)
            }
            _ => unreachable!("data_index bridge fn set mismatch"),
        };
    }

    if is_v4_inert_lens_bridge_call(ctx, &func_name) {
        return match func_name.as_str() {
            "inert_lens_unreached_module_count" => Ok(Value::Int(
                crate::cli_run::inert_lens_unreached_module_count(),
            )),
            "inert_lens_top_level_module_count" => Ok(Value::Int(
                crate::cli_run::inert_lens_top_level_module_count(),
            )),
            _ => unreachable!("inert_lens bridge fn set mismatch"),
        };
    }

    match func_name.as_str() {
        "fold_list" => return eval_fold_list_native(&args, env, ctx),
        "fold_list_right" => return eval_fold_list_right_native(&args, env, ctx),
        _ => {}
    }

    if let Some(result) = eval_builtin(&func_name, &args, ctx)? {
        return Ok(result);
    }

    let fn_node = if let Some(node) = ctx.lookup_fn(&func_name) {
        node.clone()
    } else {
        match env.lookup(ctx.sym(&func_name)) {
            Some(Value::Fn { node }) => node.clone(),
            Some(closure @ Value::Closure { .. }) => {
                let closure = closure.clone();
                let arg_vals: Vec<Value> = args.iter().map(|(_, v)| v.clone()).collect();
                return apply_closure(&closure, &arg_vals, env, ctx);
            }
            _ => {
                return Err(InterpError::NoSuchFunction {
                    name: func_name.clone(),
                });
            }
        }
    };

    if let Some(result) = try_parse_table_memo_dispatch(ctx, &func_name, &fn_node, &args, env)? {
        return Ok(result);
    }

    if let Some(key) = pure_call_memo_key(&fn_node, &func_name, &args) {
        if let Some(v) = pure_call_memo_get(ctx, &key) {
            return Ok(v);
        }
        let result = call_function(ctx, &fn_node, &args, env)?;
        pure_call_memo_put(ctx, &fn_node, key, &args, result.clone());
        return Ok(result);
    }
    if fn_node.uses.is_empty() {
        return eval_pure_named_call(ctx, node, &fn_node, &func_name, &args, env);
    }
    call_function(ctx, &fn_node, &args, env)
}

/// Pure named-fn calls flow through here: the demand ledger records every
/// keyed call, and the eval-frame memo (the ladder's single-site discharge
/// provider) serves repeated demands from the first evaluation. A memo hit
/// still records the DEMAND in the ledger — plurality is the fact the receipt
/// counts; the provider changes its cost, never its count. Soundness: the memo
/// is hash-bucketed on the ledger key but serves only after the stored call's
/// argument names AND values verify equal (Value::eq, the one equality
/// authority) — a hash collision degrades to recompute, never to a wrong
/// value. Unkeyed calls (closure args) stay unmemoized and are counted.
fn eval_pure_named_call(
    ctx: &InterpContext,
    call_node: &Rc<Node>,
    fn_node: &Rc<Node>,
    func_name: &str,
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
) -> InterpResult<Value> {
    let trace_on = eval_recompute_trace_enabled();
    let memo_on = ctx.eval_call_memo.borrow().enabled;
    if !trace_on && !memo_on {
        return call_function(ctx, fn_node, args, env);
    }
    let started = Instant::now();
    let key = match eval_recompute_key(ctx, fn_node, args) {
        Some(key) => key,
        None => {
            if trace_on {
                eval_recompute_record_unkeyed(ctx, func_name);
            }
            return call_function(ctx, fn_node, args, env);
        }
    };
    if memo_on {
        if let Some(v) = eval_call_memo_get(ctx, &key, args) {
            if trace_on {
                eval_recompute_record(
                    ctx,
                    call_node,
                    fn_node,
                    func_name,
                    key,
                    started.elapsed().as_nanos(),
                );
            }
            return Ok(v);
        }
    }
    let effects_before = ctx.effect_dispatch_count.get();
    let result = call_function(ctx, fn_node, args, env);
    if memo_on && ctx.effect_dispatch_count.get() == effects_before {
        if let Ok(v) = &result {
            eval_call_memo_put(ctx, fn_node, key.clone(), args, v.clone());
        }
    }
    if trace_on {
        eval_recompute_record(
            ctx,
            call_node,
            fn_node,
            func_name,
            key,
            started.elapsed().as_nanos(),
        );
    }
    result
}

fn eval_fold_list_native(
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let positional: Vec<&Value> = args.iter().map(|(_, v)| v).collect();
    let (xs, empty, cons) = match positional.as_slice() {
        [xs, empty, cons] => (xs, empty, cons),
        _ => {
            return Err(InterpError::TypeError {
                msg: "fold_list requires (xs, empty, cons)".to_string(),
            })
        }
    };
    // NOTE (nimble-otter-476, adhoc-c328b166-bca): a streaming left-fold that
    // walks the Cons-chain without materializing this Vec was BUILT, proven
    // byte-identical (parse-tree content hash equal on tiny/small across two
    // independently-built binaries), and MEASURED -- it moved neither wall-clock
    // (~20s both, within run-to-run noise) nor peak RSS (~168 MiB both) on the
    // small file. Reason: the datetime driver folds `elem=Int` codepoint lists
    // (trivial copy, not a deep clone) and the intermediate Vec is transient
    // (freed each fold, so it never contributes to peak RSS), so removing it is
    // a clean zero. The real O(n^2) is the CALLER re-folding the whole source
    // (`lex_repeat_loop`, 01_tokenize.dag:158 -- routed to the tokenize lane),
    // not this per-call materialization. Kept as `free_monoid_to_vec` rather
    // than churning the seed for a measured-zero rewrite (DESIGN §6: denominate
    // in displaced cost; a no-op displaces nothing).
    let items = free_monoid_to_vec(xs).ok_or_else(|| InterpError::TypeError {
        msg: format!("fold_list expects a list, got {}", xs.type_label()),
    })?;
    if items.len() > 1000 {
        record_big_fold_dag_site(cons, items.len());
    }
    record_fold_caller(items.len(), items.first(), "left");
    let mut acc = (*empty).clone();
    for item in items {
        acc = apply_closure(*cons, &[acc, item], env, ctx)?;
    }
    Ok(acc)
}

fn eval_fold_list_right_native(
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let positional: Vec<&Value> = args.iter().map(|(_, v)| v).collect();
    let (xs, empty, snoc) = match positional.as_slice() {
        [xs, empty, snoc] => (xs, empty, snoc),
        _ => {
            return Err(InterpError::TypeError {
                msg: "fold_list_right requires (xs, empty, snoc)".to_string(),
            })
        }
    };
    let items = free_monoid_to_vec(xs).ok_or_else(|| InterpError::TypeError {
        msg: format!("fold_list_right expects a list, got {}", xs.type_label()),
    })?;
    record_fold_caller(items.len(), items.first(), "right");
    let mut acc = (*empty).clone();
    for item in items.into_iter().rev() {
        acc = apply_closure(*snoc, &[acc, item], env, ctx)?;
    }
    Ok(acc)
}

fn witness_holds(value: Value, ctx: &InterpContext) -> Value {
    Value::Variant {
        type_name: ctx.sym("Witness"),
        variant_name: ctx.sym("Holds"),
        fields: Rc::new(vec![(ctx.sym("value"), value)]),
    }
}

fn witness_violates(diagnostic: Value, ctx: &InterpContext) -> Value {
    Value::Variant {
        type_name: ctx.sym("Witness"),
        variant_name: ctx.sym("Violates"),
        fields: Rc::new(vec![(ctx.sym("diagnostic"), diagnostic)]),
    }
}

fn parse_table_materialization_allows_memo(ctx: &InterpContext, table: &Value) -> bool {
    // SCAFFOLD (§7 seed-retained): extdeps/realization/parse_table_memo.dag
    // parse_table_memo_seed_handler_dissolution_trigger (Disposition Scaffold) — seed
    // try_parse_table_memo_dispatch gates ParseTableMemo map insert/serve on Memoize;
    // .dag authority: v2.compiler.materialization_allows_memo_store.
    let table_fields = match table {
        Value::Record { fields, .. } | Value::Variant { fields, .. } => fields,
        _ => return false,
    };
    let Some(mat) = ctx.field(table_fields, "materialization") else {
        return false;
    };
    match mat {
        Value::Variant { variant_name, .. } => resolve_sym(*variant_name) == "Memoize",
        _ => false,
    }
}

fn parse_table_memo_scope_and_key(
    ctx: &InterpContext,
    table: &Value,
    key: &Value,
) -> Option<(String, String, i64, Symbol)> {
    if !parse_table_materialization_allows_memo(ctx, table) {
        return None;
    }
    let table_fields = match table {
        Value::Record { fields, .. } | Value::Variant { fields, .. } => fields,
        _ => return None,
    };
    let grammar_digest = match ctx.field(table_fields, "grammar_digest")? {
        Value::Str(s) => s.clone(),
        _ => return None,
    };
    let token_stream_digest = match ctx.field(table_fields, "token_stream_digest")? {
        Value::Str(s) => s.clone(),
        _ => return None,
    };
    let key_fields = match key {
        Value::Record { fields, .. } | Value::Variant { fields, .. } => fields,
        _ => return None,
    };
    let position = match fields_get(key_fields, ctx.sym("position")) {
        Some(Value::Int(n)) => *n,
        _ => return None,
    };
    let production = match fields_get(key_fields, ctx.sym("production")) {
        Some(Value::Str(s)) => ctx.sym(s),
        _ => return None,
    };
    Some((grammar_digest, token_stream_digest, position, production))
}

fn try_parse_table_memo_dispatch(
    ctx: &InterpContext,
    func_name: &str,
    fn_node: &Rc<Node>,
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
) -> InterpResult<Option<Value>> {
    match func_name {
        "parse_table_lookup" => {
            let positional: Vec<&Value> = args.iter().map(|(_, v)| v).collect();
            let [table, key] = match positional.as_slice() {
                [table, key] => [table, key],
                _ => return Ok(None),
            };
            let Some(memo_key) = parse_table_memo_scope_and_key(ctx, table, key) else {
                return Ok(None);
            };
            let allows_memo = parse_table_materialization_allows_memo(ctx, table);
            let mut st = ctx.parse_table_memo.borrow_mut();
            st.lookups += 1;
            if allows_memo {
                if let Some(v) = st.map.get(&memo_key).cloned() {
                    st.hits += 1;
                    drop(st);
                    record_parse_memo_lookup(&memo_key, true);
                    return Ok(Some(witness_holds(v, ctx)));
                }
            }
            drop(st);
            record_parse_memo_lookup(&memo_key, false);
            let result = call_function(ctx, fn_node, args, env)?;
            Ok(Some(result))
        }
        "parse_table_insert" => {
            let positional: Vec<&Value> = args.iter().map(|(_, v)| v).collect();
            let [table, key, value] = match positional.as_slice() {
                [table, key, value] => [table, key, value],
                _ => return Ok(None),
            };
            let result = call_function(ctx, fn_node, args, env)?;
            if parse_table_materialization_allows_memo(ctx, table) {
                if let Some(memo_key) = parse_table_memo_scope_and_key(ctx, table, key) {
                    let mut st = ctx.parse_table_memo.borrow_mut();
                    st.keepalive.push((*table).clone());
                    st.keepalive.push((*key).clone());
                    st.keepalive.push((*value).clone());
                    st.map.insert(memo_key, (*value).clone());
                    st.inserts += 1;
                }
            }
            Ok(Some(result))
        }
        _ => Ok(None),
    }
}

fn is_structural_pure_fn(name: &str) -> bool {
    matches!(
        name,
        "content_hash"
            | "well_formed"
            | "locally_well_formed"
            | "fold_node"
            | "fold_node_content_hash"
            | "node_subtree_count"
    )
}

fn eval_recompute_str_hash(s: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

fn eval_recompute_mix(seed: u64, x: u64) -> u64 {
    (seed.rotate_left(5) ^ x).wrapping_mul(0x100000001b3)
}

fn eval_recompute_canon_key_hash(k: &CanonKey) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    k.hash(&mut h);
    h.finish()
}

// Content hash of a Value, memoized per composite allocation. Equal values
// (per Value::eq's structural semantics) hash equal; the pointer memo is
// validated by Weak-liveness so a freed-then-reused address recomputes
// instead of serving a stale hash. Returns None when the value contains a
// Closure (no computed identity for captured envs) — the caller routes that
// call to the disclosed unkeyed bucket. Iterative (explicit frame stack):
// corpus values include Cons-chain lists and deep node trees whose depth is
// data-sized, so recursion here would overflow the host stack.
enum EvalRecomputeFrameKind {
    List {
        rc: Rc<RrbVector<Value>>,
    },
    Fields {
        rc: Rc<Vec<(Symbol, Value)>>,
        type_sym: u32,
        variant_sym: u32,
        is_variant: bool,
    },
    Map {
        rc: Rc<HamtMap<CanonKey, Value>>,
        key_hashes: Vec<u64>,
        values: Vec<Value>,
    },
}

struct EvalRecomputeFrame {
    kind: EvalRecomputeFrameKind,
    idx: usize,
    h: u64,
}

fn eval_recompute_frame_child(f: &EvalRecomputeFrame, idx: usize) -> Option<Value> {
    match &f.kind {
        EvalRecomputeFrameKind::List { rc } => rc.get(idx).cloned(),
        EvalRecomputeFrameKind::Fields { rc, .. } => rc.get(idx).map(|(_, v)| v.clone()),
        EvalRecomputeFrameKind::Map { values, .. } => values.get(idx).cloned(),
    }
}

fn eval_recompute_frame_integrate(f: &mut EvalRecomputeFrame, child_h: u64) {
    match &f.kind {
        EvalRecomputeFrameKind::List { .. } => {
            f.h = eval_recompute_mix(f.h, child_h);
        }
        EvalRecomputeFrameKind::Fields { rc, .. } => {
            let sym = (rc[f.idx].0).0;
            let mixed = eval_recompute_mix(f.h, u64::from(sym));
            f.h = eval_recompute_mix(mixed, child_h);
        }
        EvalRecomputeFrameKind::Map { key_hashes, .. } => {
            // Order-independent combine: im map iteration order is not
            // content-canonical, so entries fold commutatively.
            f.h =
                f.h.wrapping_add(eval_recompute_mix(key_hashes[f.idx], child_h));
        }
    }
}

fn eval_recompute_frame_finalize(memo: &mut EvalRecomputeHashMemo, f: EvalRecomputeFrame) -> u64 {
    let EvalRecomputeFrame { kind, h, .. } = f;
    match kind {
        EvalRecomputeFrameKind::List { rc } => {
            memo.insert(
                Rc::as_ptr(&rc) as usize,
                (CompositeWeak::List(Rc::downgrade(&rc)), h),
            );
            h
        }
        EvalRecomputeFrameKind::Fields {
            rc,
            type_sym,
            variant_sym,
            is_variant,
        } => {
            // The fields-content hash is memoized independently of the owning
            // type/variant symbols (a fields Rc could in principle be shared
            // across constructions), so the memo entry never bakes in the
            // wrapper identity.
            memo.insert(
                Rc::as_ptr(&rc) as usize,
                (CompositeWeak::Fields(Rc::downgrade(&rc)), h),
            );
            if is_variant {
                eval_recompute_mix(
                    eval_recompute_mix(
                        eval_recompute_mix(0xA5A5_0080, u64::from(type_sym)),
                        u64::from(variant_sym),
                    ),
                    h,
                )
            } else {
                eval_recompute_mix(eval_recompute_mix(0xA5A5_0070, u64::from(type_sym)), h)
            }
        }
        EvalRecomputeFrameKind::Map { rc, .. } => {
            let vh = eval_recompute_mix(0xA5A5_0090, h);
            memo.insert(
                Rc::as_ptr(&rc) as usize,
                (CompositeWeak::Map(Rc::downgrade(&rc)), vh),
            );
            vh
        }
    }
}

enum EvalRecomputeStep {
    Have(u64),
    Opened,
    Bail,
}

fn eval_recompute_value_hash(memo: &mut EvalRecomputeHashMemo, root: &Value) -> Option<u64> {
    let mut frames: Vec<EvalRecomputeFrame> = Vec::new();
    let mut cursor: Value = root.clone();
    loop {
        // Phase 1: reduce cursor to a hash, opening frames for uncached composites.
        let mut child_h: u64 = loop {
            let step = match &cursor {
                Value::Null => EvalRecomputeStep::Have(0xA5A5_0001),
                Value::Unit => EvalRecomputeStep::Have(0xA5A5_0002),
                Value::Bool(b) => EvalRecomputeStep::Have(0xA5A5_0010 ^ u64::from(*b)),
                Value::Int(i) => {
                    EvalRecomputeStep::Have(eval_recompute_mix(0xA5A5_0020, *i as u64))
                }
                Value::Float(f) => {
                    EvalRecomputeStep::Have(eval_recompute_mix(0xA5A5_0030, f.to_bits()))
                }
                Value::Str(s) => EvalRecomputeStep::Have(eval_recompute_mix(
                    0xA5A5_0040,
                    eval_recompute_str_hash(s),
                )),
                Value::Fn { node } => EvalRecomputeStep::Have(eval_recompute_mix(
                    0xA5A5_0050,
                    Rc::as_ptr(node) as u64,
                )),
                Value::Closure { .. } => EvalRecomputeStep::Bail,
                Value::Set(s) => {
                    let ptr = Rc::as_ptr(s) as usize;
                    match memo.get(&ptr) {
                        Some((w, h)) if w.alive() => EvalRecomputeStep::Have(*h),
                        _ => {
                            let mut h: u64 = 0xA5A5_00A0;
                            for item in s.iter() {
                                h = eval_recompute_mix(h, eval_recompute_str_hash(item));
                            }
                            memo.insert(ptr, (CompositeWeak::Set(Rc::downgrade(s)), h));
                            EvalRecomputeStep::Have(h)
                        }
                    }
                }
                Value::List(xs) => {
                    let ptr = Rc::as_ptr(xs) as usize;
                    match memo.get(&ptr) {
                        Some((w, h)) if w.alive() => EvalRecomputeStep::Have(*h),
                        _ => {
                            frames.push(EvalRecomputeFrame {
                                kind: EvalRecomputeFrameKind::List { rc: xs.clone() },
                                idx: 0,
                                h: 0xA5A5_0060,
                            });
                            EvalRecomputeStep::Opened
                        }
                    }
                }
                Value::Record { type_name, fields } => {
                    let ptr = Rc::as_ptr(fields) as usize;
                    match memo.get(&ptr) {
                        Some((w, h)) if w.alive() => EvalRecomputeStep::Have(eval_recompute_mix(
                            eval_recompute_mix(0xA5A5_0070, u64::from(type_name.0)),
                            *h,
                        )),
                        _ => {
                            frames.push(EvalRecomputeFrame {
                                kind: EvalRecomputeFrameKind::Fields {
                                    rc: fields.clone(),
                                    type_sym: type_name.0,
                                    variant_sym: 0,
                                    is_variant: false,
                                },
                                idx: 0,
                                h: 0xA5A5_00F0,
                            });
                            EvalRecomputeStep::Opened
                        }
                    }
                }
                Value::Variant {
                    type_name,
                    variant_name,
                    fields,
                } => {
                    let ptr = Rc::as_ptr(fields) as usize;
                    match memo.get(&ptr) {
                        Some((w, h)) if w.alive() => EvalRecomputeStep::Have(eval_recompute_mix(
                            eval_recompute_mix(
                                eval_recompute_mix(0xA5A5_0080, u64::from(type_name.0)),
                                u64::from(variant_name.0),
                            ),
                            *h,
                        )),
                        _ => {
                            frames.push(EvalRecomputeFrame {
                                kind: EvalRecomputeFrameKind::Fields {
                                    rc: fields.clone(),
                                    type_sym: type_name.0,
                                    variant_sym: variant_name.0,
                                    is_variant: true,
                                },
                                idx: 0,
                                h: 0xA5A5_00F0,
                            });
                            EvalRecomputeStep::Opened
                        }
                    }
                }
                Value::Map(m) => {
                    let ptr = Rc::as_ptr(m) as usize;
                    match memo.get(&ptr) {
                        Some((w, h)) if w.alive() => EvalRecomputeStep::Have(*h),
                        _ => {
                            let mut key_hashes = Vec::with_capacity(m.len());
                            let mut values = Vec::with_capacity(m.len());
                            for (k, v) in m.iter() {
                                key_hashes.push(eval_recompute_canon_key_hash(k));
                                values.push(v.clone());
                            }
                            frames.push(EvalRecomputeFrame {
                                kind: EvalRecomputeFrameKind::Map {
                                    rc: m.clone(),
                                    key_hashes,
                                    values,
                                },
                                idx: 0,
                                h: 0,
                            });
                            EvalRecomputeStep::Opened
                        }
                    }
                }
            };
            match step {
                EvalRecomputeStep::Have(h) => break h,
                EvalRecomputeStep::Bail => return None,
                EvalRecomputeStep::Opened => {
                    let top = frames.last().expect("frame just pushed");
                    match eval_recompute_frame_child(top, 0) {
                        Some(c) => cursor = c,
                        None => {
                            let f = frames.pop().expect("frame just pushed");
                            break eval_recompute_frame_finalize(memo, f);
                        }
                    }
                }
            }
        };
        // Phase 2: feed the completed child hash upward until a frame needs
        // its next child (back to phase 1) or all frames close (done).
        loop {
            match frames.last_mut() {
                None => return Some(child_h),
                Some(f) => {
                    eval_recompute_frame_integrate(f, child_h);
                    f.idx += 1;
                    match eval_recompute_frame_child(f, f.idx) {
                        Some(next) => {
                            cursor = next;
                            break;
                        }
                        None => {
                            let done = frames.pop().expect("frame present");
                            child_h = eval_recompute_frame_finalize(memo, done);
                        }
                    }
                }
            }
        }
    }
}

fn eval_recompute_arg_key(
    memo: &mut EvalRecomputeHashMemo,
    v: &Value,
) -> Option<EvalRecomputeArgKey> {
    match v {
        Value::Null => Some(EvalRecomputeArgKey::Null),
        Value::Bool(b) => Some(EvalRecomputeArgKey::Bool(*b)),
        Value::Int(i) => Some(EvalRecomputeArgKey::Int(*i)),
        Value::Float(f) => Some(EvalRecomputeArgKey::FloatBits(f.to_bits())),
        Value::Str(s) => Some(EvalRecomputeArgKey::StrHash(eval_recompute_str_hash(s))),
        Value::Variant {
            type_name,
            variant_name,
            fields,
        } if fields.is_empty() => Some(EvalRecomputeArgKey::UnitVariant(
            type_name.0,
            variant_name.0,
        )),
        Value::List(xs) if xs.is_empty() => Some(EvalRecomputeArgKey::EmptyList),
        other => eval_recompute_value_hash(memo, other).map(EvalRecomputeArgKey::ContentHash),
    }
}

fn eval_recompute_key(
    ctx: &InterpContext,
    fn_node: &Rc<Node>,
    args: &[(Option<String>, Value)],
) -> Option<EvalRecomputeKey> {
    let mut memo = ctx.eval_recompute_hash_memo.borrow_mut();
    let mut keys = Vec::with_capacity(args.len());
    for (_, v) in args {
        keys.push(eval_recompute_arg_key(&mut memo, v)?);
    }
    Some(EvalRecomputeKey {
        fn_ptr: Rc::as_ptr(fn_node) as usize,
        args: keys,
    })
}

fn eval_recompute_record(
    ctx: &InterpContext,
    call_node: &Rc<Node>,
    fn_node: &Rc<Node>,
    func_name: &str,
    key: EvalRecomputeKey,
    elapsed_ns: u128,
) {
    let mut t = ctx.eval_recompute_trace.borrow_mut();
    let tr = &mut *t;
    tr.keyed_calls += 1;
    if !tr.map.contains_key(&key) {
        if tr.map.len() >= EVAL_RECOMPUTE_KEY_CAP {
            tr.overflow_calls += 1;
            return;
        }
        tr.keepalive_fns.push(fn_node.clone());
    }
    let fn_ptr = Rc::as_ptr(fn_node) as usize;
    let interned_name = tr
        .fn_names
        .entry(fn_ptr)
        .or_insert_with(|| Rc::from(func_name))
        .clone();
    let entry = tr.map.entry(key).or_insert_with(|| EvalRecomputeEntry {
        fn_name: interned_name,
        count: 0,
        total_ns: 0,
        sites: Vec::new(),
    });
    entry.count += 1;
    entry.total_ns += elapsed_ns;
    let site_ptr = Rc::as_ptr(call_node) as usize;
    if entry.sites.len() < EVAL_RECOMPUTE_SITE_CAP
        && !entry.sites.iter().any(|(p, _)| *p == site_ptr)
    {
        entry.sites.push((
            site_ptr,
            format!("{}:{}", call_node.span.file, call_node.span.start),
        ));
    }
}

fn eval_recompute_record_unkeyed(ctx: &InterpContext, func_name: &str) {
    let mut t = ctx.eval_recompute_trace.borrow_mut();
    t.unkeyed_calls += 1;
    *t.unkeyed_by_fn.entry(func_name.to_string()).or_insert(0) += 1;
}

/// Print the recompute-trace ledger to stderr. A no-op unless
/// GUNBC_RECOMPUTE_TRACE=1. Report-only: prints ranked re-evaluated pure calls
/// (count >= 2), the unkeyed-coverage disclosure, and totals; it never alters
/// the run's outcome.
pub fn print_eval_recompute_trace(ctx: &InterpContext) {
    if !eval_recompute_trace_enabled() {
        return;
    }
    let t = ctx.eval_recompute_trace.borrow();
    let totals = trace_totals(&t);
    let mut duplicated: Vec<(&EvalRecomputeEntry, u128)> = t
        .map
        .values()
        .filter(|e| e.count >= 2)
        .map(|e| (e, entry_wasted_ns(e)))
        .collect();
    duplicated.sort_by(|a, b| b.1.cmp(&a.1));
    eprintln!(
        "[recompute-trace] keyed_calls={} unkeyed_calls={} overflow_calls={} distinct_keys={} duplicated_keys={} wasted_ms={} (durations inclusive of callees)",
        totals.keyed_calls,
        totals.unkeyed_calls,
        totals.overflow_calls,
        totals.distinct_keys,
        totals.duplicated_keys,
        totals.wasted_ns_total / 1_000_000
    );
    eprintln!(
        "[recompute-trace] gap: single_site_keys={} wasted_ms={} (same call expression re-hit — value-coincident/loop-borne, memoize/Share territory) | multi_site_keys={} wasted_ms={} (cross-site duplicate demand — static rewire candidates)",
        totals.single_site_keys,
        totals.wasted_ns_single_site / 1_000_000,
        totals.multi_site_keys,
        totals.wasted_ns_multi_site / 1_000_000
    );
    for (e, wasted_ns) in duplicated.iter().take(20) {
        let site_labels: Vec<&str> = e
            .sites
            .iter()
            .take(2)
            .map(|(_, label)| label.as_str())
            .collect();
        eprintln!(
            "[recompute-trace] dup fn={} count={} total_ms={} wasted_ms={} sites={}{} @{}",
            e.fn_name,
            e.count,
            e.total_ns / 1_000_000,
            wasted_ns / 1_000_000,
            e.sites.len(),
            if e.sites.len() >= EVAL_RECOMPUTE_SITE_CAP {
                "+"
            } else {
                ""
            },
            site_labels.join(" @")
        );
    }
    let mut unkeyed: Vec<(&String, &u64)> = t.unkeyed_by_fn.iter().collect();
    unkeyed.sort_by(|a, b| b.1.cmp(a.1));
    for (name, count) in unkeyed.iter().take(10) {
        eprintln!(
            "[recompute-trace] unkeyed fn={} calls={} (composite args — identity not tracked in slice 1)",
            name, count
        );
    }
    let (hits, misses, overflow) = eval_call_memo_counters(ctx);
    eprintln!(
        "[recompute-trace] eval-memo: hits={} misses={} overflow={} (verified-hit serve; a hit still counts as a demand above)",
        hits, misses, overflow
    );
}

// Everything a re-evaluated key cost beyond one evaluation's amortized share.
fn entry_wasted_ns(e: &EvalRecomputeEntry) -> u128 {
    e.total_ns - e.total_ns / u128::from(e.count)
}

/// Ledger totals for one InterpContext — the materialization demand receipt at
/// the eval-frame grain. Key counts are deterministic for a fixed corpus and
/// entry set; wasted_ns durations are observational and must never gate.
#[derive(Default, Clone)]
pub struct EvalRecomputeTotals {
    pub keyed_calls: u64,
    pub unkeyed_calls: u64,
    pub overflow_calls: u64,
    pub distinct_keys: u64,
    pub duplicated_keys: u64,
    pub single_site_keys: u64,
    pub multi_site_keys: u64,
    pub wasted_ns_total: u128,
    pub wasted_ns_single_site: u128,
    pub wasted_ns_multi_site: u128,
    pub memo_hits: u64,
    pub memo_misses: u64,
    pub memo_overflow: u64,
}

impl EvalRecomputeTotals {
    pub fn absorb(&mut self, o: &EvalRecomputeTotals) {
        self.keyed_calls += o.keyed_calls;
        self.unkeyed_calls += o.unkeyed_calls;
        self.overflow_calls += o.overflow_calls;
        self.distinct_keys += o.distinct_keys;
        self.duplicated_keys += o.duplicated_keys;
        self.single_site_keys += o.single_site_keys;
        self.multi_site_keys += o.multi_site_keys;
        self.wasted_ns_total += o.wasted_ns_total;
        self.wasted_ns_single_site += o.wasted_ns_single_site;
        self.wasted_ns_multi_site += o.wasted_ns_multi_site;
        self.memo_hits += o.memo_hits;
        self.memo_misses += o.memo_misses;
        self.memo_overflow += o.memo_overflow;
    }
}

fn trace_totals(t: &EvalRecomputeTrace) -> EvalRecomputeTotals {
    let mut out = EvalRecomputeTotals {
        keyed_calls: t.keyed_calls,
        unkeyed_calls: t.unkeyed_calls,
        overflow_calls: t.overflow_calls,
        distinct_keys: t.map.len() as u64,
        ..EvalRecomputeTotals::default()
    };
    for e in t.map.values() {
        if e.count < 2 {
            continue;
        }
        let w = entry_wasted_ns(e);
        out.duplicated_keys += 1;
        out.wasted_ns_total += w;
        if e.sites.len() == 1 {
            out.single_site_keys += 1;
            out.wasted_ns_single_site += w;
        } else {
            out.multi_site_keys += 1;
            out.wasted_ns_multi_site += w;
        }
    }
    out
}

pub fn eval_recompute_totals(ctx: &InterpContext) -> EvalRecomputeTotals {
    let mut out = trace_totals(&ctx.eval_recompute_trace.borrow());
    let m = ctx.eval_call_memo.borrow();
    out.memo_hits = m.hits;
    out.memo_misses = m.misses;
    out.memo_overflow = m.overflow;
    out
}

// Process-wide accumulator fed by InterpContext::drop, so EVERY eval path in
// the process lands in the receipt by construction — harvest is not a
// per-call-site discipline a future site could forget. Sums at the totals
// grain only: raw ledger keys are address-based and single-ctx.
static PROCESS_EVAL_RECOMPUTE_TOTALS: std::sync::Mutex<Option<EvalRecomputeTotals>> =
    std::sync::Mutex::new(None);

/// Drain the process-wide ledger totals (e.g. to write a receipt file at the
/// end of a floor walk). Returns zeroed totals when tracing was disabled.
pub fn take_process_eval_recompute_totals() -> EvalRecomputeTotals {
    // A poisoned lock still holds structurally valid totals (absorb is
    // add-only), so recover the data rather than silently returning zeroes.
    PROCESS_EVAL_RECOMPUTE_TOTALS
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .take()
        .unwrap_or_default()
}

impl Drop for InterpContext {
    fn drop(&mut self) {
        if !eval_recompute_trace_enabled() {
            return;
        }
        let totals = eval_recompute_totals(self);
        if totals.keyed_calls == 0 && totals.unkeyed_calls == 0 {
            return;
        }
        // Recover a poisoned lock rather than dropping this ctx's contribution
        // without a trace — absorb is add-only, so the state stays valid.
        let mut g = PROCESS_EVAL_RECOMPUTE_TOTALS
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        g.get_or_insert_with(EvalRecomputeTotals::default)
            .absorb(&totals);
    }
}

fn value_rc_identity(v: &Value) -> Option<usize> {
    match v {
        Value::Record { fields, .. } | Value::Variant { fields, .. } => {
            Some(Rc::as_ptr(fields) as usize)
        }
        Value::List(xs) => Some(Rc::as_ptr(xs) as usize),
        Value::Map(m) => Some(Rc::as_ptr(m) as usize),
        Value::Set(s) => Some(Rc::as_ptr(s) as usize),
        _ => None,
    }
}

// Same-allocation composites are equal without a walk; everything else takes
// the full structural equality (Value::eq, the one equality authority).
fn value_fast_eq(a: &Value, b: &Value) -> bool {
    if let (Some(x), Some(y)) = (value_rc_identity(a), value_rc_identity(b)) {
        if x == y {
            return true;
        }
    }
    a == b
}

// A stored call matches only when argument NAMES and values both agree —
// names participate in parameter binding, so value-equal args under different
// labels are a different call, never served.
fn eval_call_memo_args_match(
    stored: &[(Option<String>, Value)],
    args: &[(Option<String>, Value)],
) -> bool {
    stored.len() == args.len()
        && stored
            .iter()
            .zip(args.iter())
            .all(|((sn, sv), (an, av))| sn == an && value_fast_eq(sv, av))
}

fn eval_call_memo_get(
    ctx: &InterpContext,
    key: &EvalRecomputeKey,
    args: &[(Option<String>, Value)],
) -> Option<Value> {
    let mut m = ctx.eval_call_memo.borrow_mut();
    let mm = &mut *m;
    if let Some(bucket) = mm.map.get(key) {
        for (stored_args, value) in bucket {
            if eval_call_memo_args_match(stored_args, args) {
                mm.hits += 1;
                return Some(value.clone());
            }
        }
    }
    None
}

fn eval_call_memo_put(
    ctx: &InterpContext,
    fn_node: &Rc<Node>,
    key: EvalRecomputeKey,
    args: &[(Option<String>, Value)],
    value: Value,
) {
    let mut m = ctx.eval_call_memo.borrow_mut();
    // Counter invariant: a miss means the call was NOT served (it evaluated),
    // so a cap-refused store attempt is still a miss — overflow ⊆ misses, and
    // hits + misses == keyed Ok-resulting calls through the memo path,
    // including under overflow. `misses` is NOT "entries stored".
    m.misses += 1;
    if m.map.len() >= EVAL_CALL_MEMO_ENTRY_CAP && !m.map.contains_key(&key) {
        m.overflow += 1;
        return;
    }
    m.keepalive_fns.push(fn_node.clone());
    let stored_args: Vec<(Option<String>, Value)> = args.to_vec();
    m.map.entry(key).or_default().push((stored_args, value));
}

/// Per-ctx memo counters (hits, misses, overflow) — for witnesses and
/// diagnostics; the process receipt aggregates these via ctx Drop.
pub fn eval_call_memo_counters(ctx: &InterpContext) -> (u64, u64, u64) {
    let m = ctx.eval_call_memo.borrow();
    (m.hits, m.misses, m.overflow)
}
fn pure_call_memo_key(
    fn_node: &Rc<Node>,
    func_name: &str,
    args: &[(Option<String>, Value)],
) -> Option<(usize, Vec<usize>)> {
    if !is_structural_pure_fn(func_name) {
        return None;
    }
    let fid = Rc::as_ptr(fn_node) as usize;
    let mut ids = Vec::with_capacity(args.len());
    for (_, v) in args {
        ids.push(value_rc_identity(v)?);
    }
    Some((fid, ids))
}
fn pure_call_memo_get(ctx: &InterpContext, key: &(usize, Vec<usize>)) -> Option<Value> {
    ctx.pure_call_memo.borrow().map.get(key).cloned()
}
fn pure_call_memo_put(
    ctx: &InterpContext,
    fn_node: &Rc<Node>,
    key: (usize, Vec<usize>),
    args: &[(Option<String>, Value)],
    result: Value,
) {
    let mut st = ctx.pure_call_memo.borrow_mut();
    st.keepalive_fns.push(fn_node.clone());
    for (_, v) in args {
        st.keepalive.push(v.clone());
    }
    st.map.insert(key, result);
}

fn eval_method_call(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let method_name = expr_method_name_at(node.clone(), ctx.si());
    let semantics = expr_method_call_semantics(node.clone());

    if let Some(MethodSemantics::ServiceMethodSemantics { service_name, .. }) = semantics.as_deref()
    {
        let extra_args = method_arg_nodes(node.clone());
        let named_args: Vec<(Option<String>, Value)> = extra_args
            .iter()
            .map(|a| {
                let name = arg_name_at(a.clone(), ctx.si());
                let val = eval_expr(&arg_value(a.clone()), env, ctx)?;
                Ok((name, val))
            })
            .collect::<InterpResult<_>>()?;
        return eval_service_call(service_name, &method_name, &named_args, env, ctx);
    }

    let receiver_val = eval_expr(&method_receiver(node.clone()), env, ctx)?;
    let extra_args = method_arg_nodes(node.clone());
    let args: Vec<Value> = extra_args
        .iter()
        .map(|a| eval_expr(&arg_value(a.clone()), env, ctx))
        .collect::<InterpResult<_>>()?;

    if method_name == "lookup" {
        let key = args.first().ok_or_else(|| InterpError::TypeError {
            msg: "lookup requires a key argument".to_string(),
        })?;
        return raw_map_lookup_witness(&receiver_val, key, env, ctx);
    }

    if let Value::Record { fields, .. } | Value::Variant { fields, .. } = &receiver_val {
        if let Some(field_val) = fields_get(fields, ctx.sym(&method_name)) {
            match field_val {
                Value::Closure { .. } => {
                    let f = field_val.clone();
                    return apply_closure(&f, &args, env, ctx);
                }
                Value::Fn { node } => {
                    let node = node.clone();
                    let named: Vec<(Option<String>, Value)> =
                        args.iter().map(|v| (None, v.clone())).collect();
                    return call_function(ctx, &node, &named, env);
                }
                _ => {}
            }
        }
    }

    match semantics.as_deref() {
        Some(MethodSemantics::AlgebraMethodSemantics { method_def, .. }) => {
            let mn = authored_name_at(ctx.si(), method_def.clone());
            eval_algebra_method(&mn, receiver_val, &args, env, ctx)
        }
        _ => eval_algebra_method(&method_name, receiver_val, &args, env, ctx),
    }
}

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
        Some(FieldAccessStyle::TupleFirst) => match expect_list(&base_val, "tuple.first") {
            Ok(items) => Ok(items.front().cloned().unwrap_or(Value::Null)),
            Err(_) => extract_field(&base_val, &field_name, env, ctx),
        },
        Some(FieldAccessStyle::TupleSecond) => match expect_list(&base_val, "tuple.second") {
            Ok(items) => Ok(items.get(1).cloned().unwrap_or(Value::Null)),
            Err(_) => extract_field(&base_val, &field_name, env, ctx),
        },
        Some(FieldAccessStyle::OptionalUnwrap) => match &base_val {
            Value::Null => Ok(Value::Null),
            _ => Ok(base_val),
        },
        Some(FieldAccessStyle::EnumAccessor) => extract_field(&base_val, &field_name, env, ctx),
        _ => extract_field(&base_val, &field_name, env, ctx),
    }
}

fn extract_field(
    value: &Value,
    field: &str,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let field_sym = ctx.sym(field);
    match value {
        Value::Record { type_name, fields } => {
            fields_get(fields, field_sym)
                .cloned()
                .ok_or_else(|| InterpError::NoSuchField {
                    type_name: ctx.resolve(*type_name).to_string(),
                    field: field.to_string(),
                })
        }
        Value::Variant {
            type_name, fields, ..
        } => fields_get(fields, field_sym)
            .cloned()
            .ok_or_else(|| InterpError::NoSuchField {
                type_name: ctx.resolve(*type_name).to_string(),
                field: field.to_string(),
            }),
        Value::Map(_) => raw_map_lookup(value, &Value::Str(field.to_string()), env, ctx),
        _ => Err(InterpError::TypeError {
            msg: format!("cannot access field '{}' on {}", field, value.type_label()),
        }),
    }
}

fn eval_record_lit(
    node: &Rc<Node>,
    parent_enum: Option<&str>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let type_name = record_lit_type_name_at(node.clone(), ctx.si()).unwrap_or_default();

    let mut fields: Vec<(Symbol, Value)> = Vec::new();
    for child in node.children.iter() {
        let fname = field_init_node_name_at(child.clone(), ctx.si());
        let fval = eval_expr(&field_init_node_value(child.clone()), env, ctx)?;
        fields.push((ctx.sym(&fname), fval));
    }
    fields.sort_unstable_by_key(|(k, _)| k.0);

    if let Some(pe) = parent_enum {
        if type_name == "Succ" {
            if let Some(Value::Int(p)) = fields_get(&fields, ctx.sym("prev")) {
                if *p >= 0 {
                    return Ok(Value::Int(p + 1));
                }
            }
        }
        Ok(Value::Variant {
            type_name: ctx.sym(pe),
            variant_name: ctx.sym(type_name.rsplit('.').next().unwrap_or(&type_name)),
            fields: Rc::new(fields),
        })
    } else {
        Ok(Value::Record {
            type_name: ctx.sym(&type_name),
            fields: Rc::new(fields),
        })
    }
}

fn eval_string_interp(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let parts = extract_string_interp_parts(node.clone());
    let mut result = String::new();
    for part in parts.iter() {
        let part_ref: &StringPart = part.as_ref();
        match part_ref {
            StringPart::Text { value } => result.push_str(value.as_str()),
            StringPart::Interpolation { expr } => {
                let val = eval_expr(&expr, env, ctx)?;
                result.push_str(&value_to_host_string(&val));
            }
        }
    }
    Ok(Value::Str(result))
}

fn lookup_type_item_across_modules(ctx: &InterpContext, type_name: &str) -> Option<Rc<Node>> {
    for module in ctx.modules.iter() {
        for item in module.items.iter() {
            if authored_name_at(ctx.si(), item.clone()) == type_name {
                return Some(item.clone());
            }
        }
    }
    None
}

fn alias_rhs_next_name(ctx: &InterpContext, rhs: Rc<Node>) -> Option<String> {
    let direct = authored_name_at(ctx.si(), rhs.clone());
    if !direct.is_empty() {
        return Some(direct);
    }
    if rhs.connective == Connective::Conj {
        for child in rhs.children.iter() {
            let base = authored_name_at(ctx.si(), child.clone());
            if !base.is_empty() {
                return Some(base);
            }
        }
    }
    match rhs.inferred.as_deref() {
        Some(InferredNode::Resolved { node }) => {
            let inner = authored_name_at(ctx.si(), node.clone());
            if inner.is_empty() {
                None
            } else {
                Some(inner)
            }
        }
        _ => None,
    }
}

fn type_item_alias_rhs_name(ctx: &InterpContext, item: &Rc<Node>) -> Option<String> {
    let rhs = match item.inferred.as_deref()? {
        InferredNode::Resolved { node } => node.clone(),
        _ => return None,
    };
    alias_rhs_next_name(ctx, rhs)
}

fn cast_target_seed_name(ctx: &InterpContext, target: Rc<Node>) -> String {
    let from_span = authored_name_at(ctx.si(), target.clone());
    if !from_span.is_empty() {
        return from_span;
    }
    if !target.name.is_empty() {
        return target.name.clone();
    }
    if let Some(name) = alias_rhs_next_name(ctx, target.clone()) {
        return name;
    }
    if let Some(InferredNode::Resolved { node }) = target.inferred.as_deref() {
        let from_inferred = authored_name_at(ctx.si(), node.clone());
        if !from_inferred.is_empty() {
            return from_inferred;
        }
        if !node.name.is_empty() {
            return node.name.clone();
        }
        if let Some(name) = alias_rhs_next_name(ctx, node.clone()) {
            return name;
        }
    }
    String::new()
}

fn cast_target_underlying_kernel(ctx: &InterpContext, target: Rc<Node>) -> String {
    let mut current = cast_target_seed_name(ctx, target);
    let mut seen = BTreeSet::new();

    for _ in 0..32 {
        if current.is_empty() {
            return String::new();
        }
        if !seen.insert(current.clone()) {
            return current;
        }
        if current == "String" {
            return "String".to_string();
        }

        let Some(item) = lookup_type_item_across_modules(ctx, &current) else {
            return current;
        };

        let Some(rhs_name) = type_item_alias_rhs_name(ctx, &item) else {
            return current;
        };

        if rhs_name == current {
            return current;
        }
        current = rhs_name;
    }

    current
}

fn str_identity_cast_if_string_family(
    val: &Value,
    ctx: &InterpContext,
    target: Rc<Node>,
) -> Option<Value> {
    let Value::Str(s) = val else {
        return None;
    };
    let kernel = cast_target_underlying_kernel(ctx, target);
    if kernel.is_empty() || kernel == "String" {
        Some(Value::Str(s.clone()))
    } else {
        None
    }
}

fn eval_cast(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let val = eval_expr(&cast_expr(node.clone()), env, ctx)?;
    let target_node = cast_target(node.clone());
    let target_name = cast_target_seed_name(ctx, target_node.clone());

    if let Some(v) = str_identity_cast_if_string_family(&val, ctx, target_node) {
        return Ok(v);
    }

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

fn eval_for_each(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let var_name = foreach_variable_at(node.clone(), ctx.si());
    let collection = eval_expr(&foreach_collection(node.clone()), env, ctx)?;
    let body_node = foreach_body(node.clone());

    let items = expect_list(&collection, "foreach")?;
    let mut results = Vec::with_capacity(items.len());
    for item in items.iter() {
        let iter_env = Env::with_binding(env, ctx.sym(&var_name), item.clone());
        results.push(eval_expr(&body_node, &iter_env, ctx)?);
    }
    Ok(list_value((results)))
}

fn eval_index(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let base = eval_expr(&index_base(node.clone()), env, ctx)?;
    let idx = eval_expr(&index_expr(node.clone()), env, ctx)?;

    match (&base, &idx) {
        (base_val, Value::Int(i))
            if !matches!(base_val, Value::Str(_)) && free_monoid_to_vec(base_val).is_some() =>
        {
            let items = expect_list(base_val, "index")?;
            let i = *i as usize;
            Ok(items.get(i).cloned().unwrap_or(Value::Null))
        }
        (base, key) if is_map_lookup_receiver(base) => raw_map_lookup(base, key, env, ctx),
        (Value::Str(s), Value::Int(i)) => {
            let i = *i as usize;
            Ok(s.chars()
                .nth(i)
                .map(|c| Value::Str(c.to_string()))
                .unwrap_or(Value::Null))
        }
        _ => Err(InterpError::TypeError {
            msg: format!(
                "cannot index {} with {}",
                base.type_label(),
                idx.type_label()
            ),
        }),
    }
}

fn eval_slice(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let base = eval_expr(&slice_base(node.clone()), env, ctx)?;
    let start = eval_expr(&slice_start(node.clone()), env, ctx)?;
    let end = eval_expr(&slice_end(node.clone()), env, ctx)?;

    match (&base, &start, &end) {
        (base_val, Value::Int(s), Value::Int(e))
            if !matches!(base_val, Value::Str(_)) && free_monoid_to_vec(base_val).is_some() =>
        {
            let items = expect_list(base_val, "slice")?;
            let s = *s as usize;
            let e = (*e as usize).min(items.len());
            let mut work = (*items).clone();
            Ok(list_value(work.slice(s..e)))
        }
        (Value::Str(str_val), Value::Int(s), Value::Int(e)) => {
            let s = *s as usize;
            let e = *e as usize;
            let sliced: String = str_val.chars().skip(s).take(e.saturating_sub(s)).collect();
            Ok(Value::Str(sliced))
        }
        _ => Err(InterpError::TypeError {
            msg: format!(
                "cannot slice {} with {}..{}",
                base.type_label(),
                start.type_label(),
                end.type_label()
            ),
        }),
    }
}

fn eval_algebra_method(
    method: &str,
    receiver: Value,
    args: &[Value],
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    if !residual_hunt_forensics_enabled() {
        return eval_algebra_method_inner(method, receiver, args, env, ctx);
    }
    let started = std::time::Instant::now();
    let result = eval_algebra_method_inner(method, receiver, args, env, ctx);
    record_builtin_time_inclusive(method, true, started.elapsed().as_nanos() as u64);
    result
}

fn eval_algebra_method_inner(
    method: &str,
    receiver: Value,
    args: &[Value],
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    match method {
        "lookup" => {
            let key = args.first().ok_or_else(|| InterpError::TypeError {
                msg: "lookup requires a key argument".to_string(),
            })?;
            raw_map_lookup(&receiver, key, env, ctx)
        }

        "map" => list_method_with_closure("map", receiver, args, env, ctx, |items, f, env, ctx| {
            items
                .iter()
                .map(|item| apply_closure(f, &[item.clone()], env, ctx))
                .collect::<InterpResult<Vec<Value>>>()
                .map(|v| list_value((v)))
        }),

        "filter" => {
            list_method_with_closure("filter", receiver, args, env, ctx, |items, f, env, ctx| {
                let mut result = Vec::new();
                for item in items.iter() {
                    let keep = apply_closure(f, &[item.clone()], env, ctx)?;
                    if keep.is_truthy() {
                        result.push(item.clone());
                    }
                }
                Ok(list_value((result)))
            })
        }

        "fold" => {
            let items = expect_list(&receiver, "fold")?;
            let (init, f) = match args {
                [init, f] => (init.clone(), f),
                _ => {
                    return Err(InterpError::TypeError {
                        msg: "fold requires (init, f) arguments".to_string(),
                    })
                }
            };
            let mut acc = init;
            for item in items.iter() {
                acc = apply_closure(f, &[acc, item.clone()], env, ctx)?;
            }
            Ok(acc)
        }

        "flat_map" => list_method_with_closure(
            "flat_map",
            receiver,
            args,
            env,
            ctx,
            |items, f, env, ctx| {
                let mut result = Vec::new();
                for item in items.iter() {
                    let mapped = apply_closure(f, &[item.clone()], env, ctx)?;
                    if matches!(&mapped, Value::Str(_)) {
                        result.push(mapped);
                    } else {
                        match free_monoid_to_vec(&mapped) {
                            Some(inner) => result.extend(inner),
                            None => result.push(mapped),
                        }
                    }
                }
                Ok(list_value((result)))
            },
        ),

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

        "sort_by" => {
            list_method_with_closure("sort_by", receiver, args, env, ctx, |items, f, env, ctx| {
                let mut keyed: Vec<(Value, Value)> = items
                    .iter()
                    .map(|item| {
                        let key = apply_closure(f, &[item.clone()], env, ctx)?;
                        Ok((key, item.clone()))
                    })
                    .collect::<InterpResult<_>>()?;
                keyed.sort_by(|(ka, _), (kb, _)| cmp_values(ka, kb));
                Ok(list_value(
                    keyed.into_iter().map(|(_, v)| v).collect::<Vec<_>>(),
                ))
            })
        }

        "list_push" => {
            if matches!(&receiver, Value::Str(_)) {
                return Err(InterpError::TypeError {
                    msg: "list_push not supported on String".to_string(),
                });
            }
            let item = args.first().cloned().unwrap_or(Value::Null);
            match value_to_list_carrier(&receiver) {
                Some((items, copied)) => {
                    let mut counters = ctx.mutation_counters.borrow_mut();
                    counters.list_push_calls += 1;
                    counters.list_push_items_copied += copied;
                    drop(counters);
                    let mut result = (*items).clone();
                    result.push_back(item);
                    Ok(list_value(result))
                }
                None => Err(InterpError::TypeError {
                    msg: format!("list_push on non-list: {}", receiver.type_label()),
                }),
            }
        }

        "concat" | "append" | "push" => {
            if let Value::Str(s) = &receiver {
                let mut result = s.clone();
                for arg in args {
                    result.push_str(&format!("{}", arg));
                }
                return Ok(Value::Str(result));
            }
            // String grounding (model↔realization): when a native String arg
            // participates, the whole `concat` is a String and realizes as one
            // native `Value::Str` — provided the receiver is itself string-like
            // (all-codepoint). A `List<String>` receiver (`Str` *elements*) is
            // rejected by `free_monoid_to_string` and falls through to the list
            // path below, so `["a","b"].concat("c")` stays a list.
            if method == "concat" && args.iter().any(|a| matches!(a, Value::Str(_))) {
                if let Some(base) = free_monoid_to_string(&receiver) {
                    if let Some(rest) = args
                        .iter()
                        .map(free_monoid_to_string)
                        .collect::<Option<Vec<_>>>()
                    {
                        return Ok(Value::Str(format!("{}{}", base, rest.concat())));
                    }
                }
            }
            if let Ok(items) = expect_list(&receiver, "concat") {
                // Fail-closed backstop (DESIGN §5): a native String arg meeting a
                // codepoint-bearing `Cons`-chain receiver here is the
                // model↔realization straddle that grounding above did not
                // dissolve — refuse loudly rather than push the `Str` into a
                // mixed `[codepoint.., Str]` list. A `Value::List` receiver is a
                // generic collection (`[1].append("ab")` is a legitimate
                // two-element list), and a homogeneous `List<String>` carries no
                // codepoint — both pass (the `orig` representation guard).
                if args.iter().any(|a| matches!(a, Value::Str(_))) {
                    let snapshot: Vec<Value> = items.iter().cloned().collect();
                    if let Some(detail) = string_realization_straddle_detail(&receiver, &snapshot) {
                        return Err(InterpError::StringRealizationStraddle { detail });
                    }
                }
                let mut result = (*items).clone();
                let mut merged_items = 0usize;
                let mut copied_items = 0usize;
                for arg in args {
                    if matches!(arg, Value::Str(_)) {
                        result.push_back(arg.clone());
                    } else {
                        match value_to_list_carrier(arg) {
                            Some((other, copied)) => {
                                merged_items += other.len();
                                copied_items += copied as usize;
                                result.append((*other).clone());
                            }
                            None => result.push_back(arg.clone()),
                        }
                    }
                }
                let mut counters = ctx.mutation_counters.borrow_mut();
                if merged_items > 0 {
                    counters.list_concat_calls += 1;
                    counters.list_concat_items_copied += copied_items as u64;
                } else {
                    counters.list_push_calls += 1;
                }
                drop(counters);
                return Ok(list_value(result));
            }
            Err(InterpError::TypeError {
                msg: format!("cannot concat on {}", receiver.type_label()),
            })
        }

        "length" | "count" | "size" => match native_len(&receiver) {
            Some(n) => Ok(Value::Int(n)),
            None => match free_monoid_to_vec(&receiver) {
                Some(items) => Ok(Value::Int(items.len() as i64)),
                None => match &receiver {
                    Value::Map(m) => Ok(Value::Int(m.len() as i64)),
                    _ => Err(InterpError::TypeError {
                        msg: format!("cannot get length of {}", receiver.type_label()),
                    }),
                },
            },
        },

        // Known-method bridge parity: infer rewrites bare `is_empty(xs)` on
        // import-stripped modules into a method call (the census never serves
        // algebra template names), so eval must implement the same member the
        // bridge targets — emptiness via the shared length authority above.
        "is_empty" => match native_len(&receiver) {
            Some(n) => Ok(Value::Bool(n == 0)),
            None => match free_monoid_to_vec(&receiver) {
                Some(items) => Ok(Value::Bool(items.is_empty())),
                None => match &receiver {
                    Value::Map(m) => Ok(Value::Bool(m.is_empty())),
                    _ => Err(InterpError::TypeError {
                        msg: format!("cannot check is_empty of {}", receiver.type_label()),
                    }),
                },
            },
        },

        "first" => {
            let items = expect_list(&receiver, "first")?;
            Ok(items.front().cloned().unwrap_or(Value::Null))
        }

        "last" => {
            let items = expect_list(&receiver, "last")?;
            Ok(items.last().cloned().unwrap_or(Value::Null))
        }

        "reverse" => {
            let items = expect_list(&receiver, "reverse")?;
            Ok(list_value(items.iter().rev().cloned().collect::<Vec<_>>()))
        }

        "skip" => {
            let items = expect_list(&receiver, "skip")?;
            let n = expect_int(args.first(), "skip")?;
            Ok(list_value(
                items.iter().skip(n as usize).cloned().collect::<Vec<_>>(),
            ))
        }

        "take" => {
            let items = expect_list(&receiver, "take")?;
            let n = expect_int(args.first(), "take")?;
            Ok(list_value(
                items.iter().take(n as usize).cloned().collect::<Vec<_>>(),
            ))
        }

        "enumerate" => {
            let items = expect_list(&receiver, "enumerate")?;
            let result: Vec<Value> = items
                .iter()
                .enumerate()
                .map(|(i, v)| Value::Record {
                    type_name: ctx.sym("Pair"),
                    fields: Rc::new(sorted_fields(vec![
                        (ctx.sym("first"), Value::Int(i as i64)),
                        (ctx.sym("second"), v.clone()),
                    ])),
                })
                .collect();
            Ok(list_value((result)))
        }

        "contains" | "has" => match &receiver {
            Value::Map(m) => {
                let key = args.first().ok_or_else(|| InterpError::TypeError {
                    msg: "contains requires a key argument".to_string(),
                })?;
                match CanonKey::new(key.clone()) {
                    Some(ck) => Ok(Value::Bool(m.contains_key(&ck))),
                    None => Ok(Value::Bool(false)),
                }
            }
            Value::Str(s) => {
                let sub = expect_str(args.first(), "contains")?;
                Ok(Value::Bool(s.contains(&sub)))
            }
            _ => match expect_list(&receiver, "contains") {
                Ok(items) => {
                    let target = args.first().cloned().unwrap_or(Value::Null);
                    Ok(Value::Bool(items.iter().any(|item| *item == target)))
                }
                Err(_) => Err(InterpError::TypeError {
                    msg: format!("contains not supported on {}", receiver.type_label()),
                }),
            },
        },

        "join" => {
            let items = expect_list(&receiver, "join")?;
            let sep = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let strs: Vec<String> = items.iter().map(|v| format!("{}", v)).collect();
            Ok(Value::Str(strs.join(&sep)))
        }

        "chars" => {
            // §6 residue: this materializes a string as a `Value::List` of
            // codepoint `Int`s, indistinguishable at the Value level from a
            // generic `Int` list. That is the named hole in the String-straddle
            // wall — see `string_realization_straddle_detail`'s `Value::List`
            // exemption. Closed by regrounding `Char`/codepoint-sequence so the
            // realization is distinguishable (grounding root, sibling #5428).
            let s = expect_str(Some(&receiver), "chars")?;
            let items: Vec<Value> = s.chars().map(|c| Value::Int(c as i64)).collect();
            Ok(list_value(items))
        }

        "map_get" => {
            let key = args.first().ok_or_else(|| InterpError::TypeError {
                msg: "map_get requires a key argument".to_string(),
            })?;
            let raw = raw_map_lookup(&receiver, key, env, ctx)?;
            Ok(map_lookup_as_optional(raw, ctx))
        }

        "get" => {
            if matches!(&receiver, Value::Str(_)) {
                let key = args.first().ok_or_else(|| InterpError::TypeError {
                    msg: "get requires a key argument".to_string(),
                })?;
                raw_map_lookup(&receiver, key, env, ctx)
            } else if let Ok(items) = expect_list(&receiver, "get") {
                let idx = expect_int(args.first(), "get")?;
                Ok(list_get_at_or_null(&items, idx))
            } else {
                let key = args.first().ok_or_else(|| InterpError::TypeError {
                    msg: "get requires a key argument".to_string(),
                })?;
                raw_map_lookup(&receiver, key, env, ctx)
            }
        }

        // These 4 arms were absent here but present in the free-function builtin dispatch --
        // eval_algebra_method (method/pipe calls) and that dispatch (direct calls) are two
        // surfaces over one builtin set that have diverged; they should be one authority.
        // Pure-eval logic, in scope of ROADMAP HAND kernel D (`v1_interpreter` pure-eval
        // dissolution, docs/plans/interpreter-kernel-d.md): dissolution trigger is the
        // pure-eval seam (`emit_host` transport wiring) grounding this dispatch into
        // `v2.compiler.eval`, at which point per-builtin arms stop being hand-Rust here.
        "map_keys" => {
            let m = expect_map(&receiver, "map_keys")?;
            let keys: Vec<Value> = m.keys().map(|k| k.key.clone()).collect();
            Ok(list_value((keys)))
        }

        "map_values" => {
            let m = expect_map(&receiver, "map_values")?;
            let vals: Vec<Value> = m.values().cloned().collect();
            Ok(list_value((vals)))
        }

        "map_contains_key" | "map_has" => {
            let m = expect_map(&receiver, "map_contains_key")?;
            let key = args.first().ok_or_else(|| InterpError::TypeError {
                msg: "map_contains_key requires a key argument".to_string(),
            })?;
            match CanonKey::new(key.clone()) {
                Some(ck) => Ok(Value::Bool(m.contains_key(&ck))),
                None => Ok(Value::Bool(false)),
            }
        }

        "map_is_empty" => {
            let m = expect_map(&receiver, "map_is_empty")?;
            Ok(Value::Bool(m.is_empty()))
        }

        "insert" | "map_insert" => {
            let m = expect_map(&receiver, "insert")?;
            let (key, val) = match args {
                [k, v] => (k.clone(), v.clone()),
                _ => {
                    return Err(InterpError::TypeError {
                        msg: "insert requires (key, value) arguments".to_string(),
                    })
                }
            };
            let ck = CanonKey::new(key).ok_or_else(|| InterpError::TypeError {
                msg: "insert key is not a valid map key (closure/fn/NaN)".to_string(),
            })?;
            let mut counters = ctx.mutation_counters.borrow_mut();
            counters.map_insert_calls += 1;
            drop(counters);
            Ok(map_value(m.update(ck, val)))
        }

        "merge" => {
            let base = expect_map(&receiver, "merge")?;
            let overlay = expect_map(args.first().unwrap_or(&Value::Null), "merge")?;
            let mut counters = ctx.mutation_counters.borrow_mut();
            counters.map_merge_calls += 1;
            drop(counters);
            Ok(map_value((*overlay).clone().union((*base).clone())))
        }

        "keys" => {
            let m = expect_map(&receiver, "keys")?;
            let keys: Vec<Value> = m.keys().map(|k| k.key.clone()).collect();
            Ok(list_value((keys)))
        }

        "values" => {
            let m = expect_map(&receiver, "values")?;
            let vals: Vec<Value> = m.values().cloned().collect();
            Ok(list_value((vals)))
        }

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
            Ok(list_value((parts)))
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
                    let sliced: String = s
                        .chars()
                        .skip(s_idx)
                        .take(e_idx.saturating_sub(s_idx))
                        .collect();
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
            Ok(s.chars()
                .nth(idx as usize)
                .map(|c| Value::Str(c.to_string()))
                .unwrap_or(Value::Null))
        }

        "index_by" => list_method_with_closure(
            "index_by",
            receiver,
            args,
            env,
            ctx,
            |items, f, env, ctx| {
                let mut m = HamtMap::new();
                for item in items.iter() {
                    let key = apply_closure(f, &[item.clone()], env, ctx)?;
                    let ck = CanonKey::new(key).ok_or_else(|| InterpError::TypeError {
                        msg: "index_by key is not a valid map key (closure/fn/NaN)".to_string(),
                    })?;
                    m.insert(ck, item.clone());
                }
                Ok(map_value(m))
            },
        ),

        _ => Err(InterpError::Unimplemented {
            what: format!("method '{}'", method),
        }),
    }
}

pub fn fixture_now_secs(ctx: &InterpContext) -> Result<u64, crate::recorded_fixture::FixtureError> {
    if ctx.service_ops.contains_key("Clock.UnixSecs") {
        let val = wet_service_call(ctx, "Clock", "UnixSecs", &[], &Env::empty())
            .map_err(|_| crate::recorded_fixture::FixtureError::ClockUnavailable)?;
        unix_secs_from_clock_value(&val, ctx)
    } else {
        realize_clock_unix_secs_transport()
    }
}

fn wet_service_call(
    ctx: &InterpContext,
    service_name: &str,
    op_name: &str,
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
) -> InterpResult<Value> {
    let key = format!("{}.{}", service_name, op_name);
    let (service_node, op_node) =
        ctx.service_ops
            .get(&key)
            .ok_or_else(|| InterpError::Unimplemented {
                what: format!("unknown service operation: {}", key),
            })?;
    let transport = op_node
        .transport
        .as_ref()
        .or(service_node.transport.as_ref())
        .ok_or_else(|| InterpError::TypeError {
            msg: format!("no transport for service {}", key),
        })?;
    let param_env = build_service_param_env(op_node, args, env, ctx)?;
    dispatch_service_wet(service_node, op_node, transport, &param_env, ctx)
}

fn unix_secs_from_clock_value(
    val: &Value,
    ctx: &InterpContext,
) -> Result<u64, crate::recorded_fixture::FixtureError> {
    match val {
        Value::Record { fields, .. } => {
            let raw = ctx
                .field(&fields, "unix_secs")
                .map(|v| format!("{v}"))
                .ok_or(crate::recorded_fixture::FixtureError::ClockUnavailable)?;
            raw.parse::<u64>()
                .map_err(|_| crate::recorded_fixture::FixtureError::ClockUnavailable)
        }
        Value::Str(s) => s
            .parse::<u64>()
            .map_err(|_| crate::recorded_fixture::FixtureError::ClockUnavailable),
        Value::Int(n) if *n >= 0 => Ok(*n as u64),
        _ => Err(crate::recorded_fixture::FixtureError::ClockUnavailable),
    }
}

fn realize_clock_unix_secs_transport() -> Result<u64, crate::recorded_fixture::FixtureError> {
    let output = std::process::Command::new("date")
        .args(["+%s"])
        .output()
        .map_err(|_| crate::recorded_fixture::FixtureError::ClockUnavailable)?;
    if !output.status.success() {
        return Err(crate::recorded_fixture::FixtureError::ClockUnavailable);
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    s.parse::<u64>()
        .map_err(|_| crate::recorded_fixture::FixtureError::ClockUnavailable)
}

fn wet_env_var(name: &str) -> Option<String> {
    let output = std::process::Command::new("printenv")
        .arg(name)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn resolve_env_var_token(ctx: &InterpContext, var_name: &str) -> Option<String> {
    if ctx.service_ops.contains_key("shell.Env.Get") {
        let args = [(Some("name".to_string()), Value::Str(var_name.to_string()))];
        match eval_service_call("shell.Env", "Get", &args, &Env::empty(), ctx) {
            Ok(Value::Record { fields, .. }) => ctx.field(&fields, "value").and_then(|v| match v {
                Value::Str(s) if !s.is_empty() => Some(s.clone()),
                _ => None,
            }),
            Ok(Value::Str(s)) if !s.is_empty() => Some(s),
            _ => None,
        }
    } else if ctx.execution_mode.is_hermetic() {
        None
    } else {
        wet_env_var(var_name)
    }
}

/// Decide whether a hermetic `Filesystem.Read` of `requested` is checkout-input access
/// under `root`: the canonicalized path must sit under the canonicalized root with no
/// `.git` or `target` component below it (branch state and build artifacts are not
/// commit-deterministic, so they are host state, not input). Err carries the typed
/// refusal cause; the caller never widens a failure into a canned response.
fn hermetic_checkout_read_disposition_under(
    root: &std::path::Path,
    requested: &str,
) -> Result<(), String> {
    let root = std::fs::canonicalize(root)
        .map_err(|e| format!("checkout root `{}` unresolvable: {e}", root.display()))?;
    let requested_path = std::path::Path::new(requested);
    let joined = if requested_path.is_absolute() {
        requested_path.to_path_buf()
    } else {
        root.join(requested_path)
    };
    let canon = std::fs::canonicalize(&joined)
        .map_err(|e| format!("path does not canonicalize under the checkout ({e})"))?;
    if !canon.starts_with(&root) {
        return Err(format!(
            "path resolves outside the checkout root {}",
            root.display()
        ));
    }
    let rel = canon
        .strip_prefix(&root)
        .expect("starts_with checked above");
    for comp in rel.components() {
        if let std::path::Component::Normal(name) = comp {
            if name == ".git" || name == "target" {
                return Err(format!(
                    "`{}` components are not commit-deterministic inputs",
                    name.to_string_lossy()
                ));
            }
        }
    }
    Ok(())
}

/// The runner contract binds the process cwd to the checkout root (claim_batch and
/// claim_executor both run from the repo root), so cwd IS the injected input root.
fn hermetic_checkout_read_disposition(requested: &str) -> Result<(), String> {
    let cwd =
        std::env::current_dir().map_err(|e| format!("checkout root (cwd) unresolvable: {e}"))?;
    hermetic_checkout_read_disposition_under(&cwd, requested)
}

fn eval_service_call(
    service_name: &str,
    op_name: &str,
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    ctx.effect_dispatch_count
        .set(ctx.effect_dispatch_count.get().wrapping_add(1));
    let key = format!("{}.{}", service_name, op_name);
    let (service_node, op_node) =
        ctx.service_ops
            .get(&key)
            .ok_or_else(|| InterpError::Unimplemented {
                what: format!("unknown service operation: {}", key),
            })?;

    let transport = op_node
        .transport
        .as_ref()
        .or(service_node.transport.as_ref())
        .ok_or_else(|| InterpError::TypeError {
            msg: format!("no transport for service {}", key),
        })?;

    let param_env = build_service_param_env(op_node, args, env, ctx)?;
    let inputs_hash =
        crate::recorded_fixture::content_hash_service_inputs(op_node, &param_env, ctx);
    let inputs_json =
        crate::recorded_fixture::service_inputs_fixture_json(op_node, &param_env, ctx)
            .map_err(|e| InterpError::TypeError { msg: e.to_string() })?;

    if ctx.execution_mode.is_hermetic() {
        // Checkout-read carve-out: the repo checkout is the run's injected input (the
        // commit IS the input), so a read-only Filesystem.Read of a path proven under
        // the checkout root stays a REAL read in hermetic mode — it is input access,
        // not a host effect. Everything else about the arm is fail-closed: an
        // out-of-root path, a `.git`/`target` component (branch state and build
        // artifacts are not commit-deterministic), or an unresolvable path each
        // refuse with a typed diagnostic — never a canned response.
        if service_name == "Filesystem" && op_name == "Read" {
            // Single-authority split (§3): a Filesystem.Read whose path the disposition
            // CONFIRMS is a committed checkout input reads directly — the commit is the run's
            // deterministic input, so this is input access, not a host effect, and it needs no
            // fixture. Everything the disposition cannot confirm — a recorded fixture's
            // scratch path, a `target/`/`.git` build artifact, an out-of-root or absent path —
            // is NOT decided here: it FALLS THROUGH to the fixture-store / published-mock /
            // fail-closed machinery below, which owns non-deterministic host state. So the
            // carve-out intercepts only what it is sure about; it never pre-empts the
            // recorded-fixture mechanism (record/replay/staleness) nor widens a host-state read
            // into a refusal that belongs to the mock layer. Checkout inputs are read from the
            // commit; host state is mocked or fails closed — no path is served by both.
            let confirmed_checkout_input = param_env
                .lookup(ctx.sym("path"))
                .and_then(|v| match v {
                    Value::Str(s) => Some(s.clone()),
                    _ => None,
                })
                .map(|requested| hermetic_checkout_read_disposition(&requested).is_ok())
                .unwrap_or(false);
            if confirmed_checkout_input {
                return dispatch_service_wet(service_node, op_node, transport, &param_env, ctx);
            }
        }
        let published = ctx.published_mock_keys()?;
        let governed = ctx.governed_services()?;
        let service_is_governed = governed.contains(service_name);
        if service_is_governed && !published.contains(&key) {
            let mut cases: Vec<&String> = published
                .iter()
                .filter(|k| {
                    k.rsplit_once('.')
                        .map(|(svc, _)| svc == service_name)
                        .unwrap_or(false)
                })
                .collect();
            cases.sort();
            return Err(InterpError::TypeError {
                msg: format!(
                    "hermetic mode: operation {key} is not a published mock case for \
                     corpus-governed service {service_name} — refusing to realize \
                     (published cases: {cases:?})"
                ),
            });
        }

        if let Some(store) = &ctx.fixture_store {
            eprintln!(
                "[hermetic:fixture] {}.{} inputs_hash={}",
                service_name, op_name, inputs_hash
            );
            let now_secs =
                fixture_now_secs(ctx).map_err(|e| InterpError::TypeError { msg: e.to_string() })?;
            let fixture = store
                .lookup(&key, &inputs_hash, &inputs_json, now_secs)
                .map_err(|e| InterpError::TypeError { msg: e.to_string() })?;
            return crate::recorded_fixture::value_from_fixture_json(&fixture.response, ctx)
                .map_err(|e| InterpError::TypeError { msg: e.to_string() });
        }
        trace_emit(
            OutputChannel::Instrumentation,
            &format!("[hermetic:mock] {}.{}", service_name, op_name),
        );
        return eval_mock_response(op_node, ctx);
    }

    let result = dispatch_service_wet(service_node, op_node, transport, &param_env, ctx)?;

    if ctx.execution_mode.is_record() {
        let store = ctx
            .fixture_store
            .as_ref()
            .ok_or_else(|| InterpError::TypeError {
                msg: "--record requires --fixture-store".to_string(),
            })?;
        let now_secs =
            fixture_now_secs(ctx).map_err(|e| InterpError::TypeError { msg: e.to_string() })?;
        store
            .record(&key, &inputs_hash, &inputs_json, &result, ctx, now_secs)
            .map_err(|e| InterpError::TypeError { msg: e.to_string() })?;
        eprintln!(
            "[record] {}.{} inputs_hash={}",
            service_name, op_name, inputs_hash
        );
    }

    Ok(result)
}

fn dispatch_service_wet(
    service_node: &Rc<Node>,
    op_node: &Rc<Node>,
    transport: &Rc<Node>,
    param_env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    if is_shell_transport(transport.clone()) {
        let result = dispatch_shell(transport, param_env, ctx)?;
        return map_shell_outputs(&result, op_node, ctx);
    }

    if is_file_transport(transport.clone(), ctx.si()) {
        let result = dispatch_file(op_node, transport, param_env, ctx)?;
        return map_file_outputs(&result, op_node, ctx);
    }

    dispatch_rest(service_node, op_node, transport, param_env, ctx)
}

fn build_service_param_env(
    op_node: &Rc<Node>,
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Rc<Env>> {
    let mut bindings = HashMap::new();

    for (opt_name, val) in args {
        if let Some(name) = opt_name {
            bindings.insert(ctx.sym(name), val.clone());
        }
    }

    let mut positional_idx = 0;
    let positional_args: Vec<&Value> = args
        .iter()
        .filter(|(name, _)| name.is_none())
        .map(|(_, v)| v)
        .collect();
    for param in op_node.params.iter() {
        let name = param_node_name_at(param.clone(), ctx.si());
        if !bindings.contains_key(&ctx.sym(&name)) {
            if positional_idx < positional_args.len() {
                bindings.insert(ctx.sym(&name), positional_args[positional_idx].clone());
                positional_idx += 1;
            }
        }
    }

    for param in op_node.params.iter() {
        let name = param_node_name_at(param.clone(), ctx.si());
        if !bindings.contains_key(&ctx.sym(&name)) {
            if let Some(default_node) = param_node_default_value(param.clone()) {
                let default_val = eval_expr(&default_node, env, ctx)?;
                bindings.insert(ctx.sym(&name), default_val);
            }
        }
    }

    Ok(Env::extend(env, bindings))
}

struct ShellResult {
    exit_code: i32,
    stdout: String,
    stderr: String,
}

fn push_shell_argv_tokens(argv: &mut Vec<String>, val: Value) -> InterpResult<()> {
    match &val {
        Value::Str(s) => {
            argv.push(s.clone());
            Ok(())
        }
        Value::List(items) => {
            for item in items.iter() {
                push_shell_argv_tokens(argv, item.clone())?;
            }
            Ok(())
        }
        Value::Variant { .. } => {
            if let Some(s) = value_as_host_string(&val) {
                argv.push(s);
                Ok(())
            } else if let Some(items) = free_monoid_to_vec(&val) {
                for item in items {
                    push_shell_argv_tokens(argv, item)?;
                }
                Ok(())
            } else {
                argv.push(format!("{}", val));
                Ok(())
            }
        }
        _ => {
            argv.push(format!("{}", val));
            Ok(())
        }
    }
}

fn value_as_host_string(val: &Value) -> Option<String> {
    if let Value::Str(s) = val {
        return Some(s.clone());
    }
    let items = free_monoid_to_vec(val)?;
    let mut out = String::new();
    for item in items {
        match item {
            Value::Int(code) => {
                let ch = char::from_u32(code as u32)?;
                out.push(ch);
            }
            Value::Str(s) => out.push_str(&s),
            _ => return None,
        }
    }
    Some(out)
}

fn value_to_host_string(val: &Value) -> String {
    value_as_host_string(val).unwrap_or_else(|| format!("{}", val))
}

fn materialize_argv_expr_for_bindings(
    node: &Rc<Node>,
    bindings: &HashMap<String, Value>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Result<Value, String> {
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitStr { value } => Ok(Value::Str(value.clone())),
            other => Err(format!(
                "shell argv materialize: unsupported literal {:?}",
                other
            )),
        },
        ExprData::ExprVar { .. } => {
            let name = expr_var_name_at(node.clone(), source_indices.clone());
            bindings
                .get(&name)
                .cloned()
                .ok_or_else(|| format!("shell argv materialize: unbound param `{name}`"))
        }
        ExprData::ExprStringInterp => {
            let parts = extract_string_interp_parts(node.clone());
            let mut result = String::new();
            for part in parts.iter() {
                match part.as_ref() {
                    StringPart::Text { value } => result.push_str(value),
                    StringPart::Interpolation { expr } => {
                        let val =
                            materialize_argv_expr_for_bindings(expr, bindings, source_indices)?;
                        result.push_str(&value_to_host_string(&val));
                    }
                }
            }
            Ok(Value::Str(result))
        }
        other => Err(format!(
            "shell argv materialize: unsupported expr {:?}",
            other
        )),
    }
}

pub fn materialize_shell_argv_for_operation(
    path: String,
    service: String,
    operation: String,
    param_bindings: HashMap<String, Value>,
) -> Result<Vec<String>, String> {
    let (argv_nodes, source_indices) =
        crate::cli_run::shell_argv_nodes_for_operation(path, service, operation);
    let mut argv: Vec<String> = Vec::new();
    for node in argv_nodes.iter() {
        let val = materialize_argv_expr_for_bindings(node, &param_bindings, &source_indices)?;
        push_shell_argv_tokens(&mut argv, val).map_err(|e| format!("{e:?}"))?;
    }
    Ok(argv)
}

/// SGR foreground parameters per `SemanticColor`, mirroring the
/// `extdeps.render.ansi` authority (`ansi_mappings` in `dag/extdeps/render/ansi.dag`).
/// Seed realization until the interpreter consumes that table directly; the
/// dissolution is the single checkable receipt ROADMAP §1 "interpreter
/// terminal-output de-fork" (`dag/gunbc/roadmap_authority.dag`).
pub mod sgr {
    pub const SUCCESS: &str = "38;5;34";
    pub const ERROR: &str = "38;5;196";
    pub const WARNING: &str = "38;5;208";
    pub const INFO: &str = "38;5;39";
    pub const DIM: &str = "2";
}

/// Whether the CLI should emit ANSI color, mirroring the `color` arm of
/// `extdeps.render.terminal_capability.detect_capability`: NO_COLOR (no-color
/// convention) and TERM=dumb force it off; otherwise color is on for an
/// interactive TTY or a CI log viewer (which renders SGR). CI keeps color even
/// though it loses cursor addressing — the CI/interactive split this PR models.
pub fn color_enabled() -> bool {
    use std::io::IsTerminal;
    thread_local! {
        static ENABLED: Cell<Option<bool>> = const { Cell::new(None) };
    }
    ENABLED.with(|c| match c.get() {
        Some(b) => b,
        None => {
            let no_color = std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty());
            let dumb = std::env::var("TERM").map(|t| t == "dumb").unwrap_or(false);
            let ci = std::env::var_os("CI").is_some_and(|v| v != "0" && !v.is_empty());
            let is_tty = std::io::stderr().is_terminal() || std::io::stdout().is_terminal();
            let b = !no_color && !dumb && (is_tty || ci);
            c.set(Some(b));
            b
        }
    })
}

/// Wrap `text` in the given SGR parameters when color is enabled, else return it
/// plain — the single funnel so a NO_COLOR/redirected run never leaks escapes.
pub fn paint(text: &str, sgr_params: &str) -> String {
    if color_enabled() {
        format!("\x1b[{sgr_params}m{text}\x1b[0m")
    } else {
        text.to_string()
    }
}

/// CLI output verbosity. Seed realization of the `gunbc.output_policy.Verbosity`
/// authority (`dag/gunbc/output_policy.dag`); resolution precedence mirrors that
/// module's `resolve_verbosity` (verbose wins over quiet, default Normal). When
/// the interpreter self-hosts, this dissolves into consuming the .dag policy.
#[derive(Clone, Copy, PartialEq)]
pub enum Verbosity {
    Quiet,
    Normal,
    Verbose,
}

/// Read the CLI verbosity once. The env-var names are the dispatch grounded by
/// `gunbc.output_policy.env_var_for` (verbose=`GUNBC_VERBOSE`,
/// quiet=`GUNBC_QUIET`); the precedence mirrors `resolve_verbosity`.
pub fn cli_verbosity() -> Verbosity {
    thread_local! {
        static POLICY: Cell<Option<Verbosity>> = const { Cell::new(None) };
    }
    POLICY.with(|c| match c.get() {
        Some(p) => p,
        None => {
            let p = if std::env::var("GUNBC_VERBOSE")
                .map(|v| v == "1")
                .unwrap_or(false)
            {
                Verbosity::Verbose
            } else if std::env::var("GUNBC_QUIET")
                .map(|v| v == "1")
                .unwrap_or(false)
            {
                Verbosity::Quiet
            } else {
                Verbosity::Normal
            };
            c.set(Some(p));
            p
        }
    })
}

/// The output channels of `gunbc.output_policy.OutputChannel`, as a carrier so
/// host-effect trace sites can name which channel they belong to. The *decision*
/// for each channel is NOT computed here — it is evaluated from the .dag authority
/// (`channel_decision` via `resolve_channel_policy`) by the entry binary and
/// installed with `set_output_policy`. Order matches the index used in the policy
/// array below.
#[derive(Clone, Copy, PartialEq)]
pub enum OutputChannel {
    Diagnostic = 0,
    ClaimResult = 1,
    Progress = 2,
    ShellTrace = 3,
    Instrumentation = 4,
}

/// Carrier for `gunbc.output_policy.OutputDecision`. The mapping from a channel +
/// verbosity to one of these lives entirely in the .dag authority; this enum only
/// transports the evaluated verdict across the seed↔.dag boundary.
#[derive(Clone, Copy, PartialEq)]
pub enum OutputDecision {
    Suppressed,
    Condensed,
    Full,
}

static OUTPUT_POLICY: std::sync::OnceLock<[OutputDecision; 5]> = std::sync::OnceLock::new();

/// Install the per-channel decisions the entry binary evaluated from
/// `gunbc.output_policy.resolve_channel_policy`. Set once at startup (before
/// discovery threads spawn), read process-wide. Idempotent: a second call is a
/// no-op (the first install wins).
pub fn set_output_policy(decisions: [OutputDecision; 5]) {
    let _ = OUTPUT_POLICY.set(decisions);
}

/// The installed decision for a channel. Falls back to `Full` (emit everything —
/// the pre-funnel behavior) when no entry binary has installed a policy, so bins
/// that don't opt in are unaffected.
pub fn output_decision(channel: OutputChannel) -> OutputDecision {
    match OUTPUT_POLICY.get() {
        Some(p) => p[channel as usize],
        None => OutputDecision::Full,
    }
}

/// Carrier for `extdeps.render.surface.GroupSyntax` — the per-target group-marker
/// strings the entry binary evaluated from `resolve_group_syntax(github_actions)`.
/// `close_line` is `None` for a plain terminal (a section closes implicitly) and
/// `Some("::endgroup::")` under GitHub Actions. The seed only TRANSPORTS these
/// literals; the choice of syntax per target stays the .dag authority's.
#[derive(Clone)]
pub struct InstalledGroupSyntax {
    pub open_prefix: String,
    pub open_suffix: String,
    pub close_line: Option<String>,
}

static GROUP_SYNTAX: std::sync::OnceLock<InstalledGroupSyntax> = std::sync::OnceLock::new();

/// Install the group-marker syntax evaluated from the .dag authority. Set once at
/// startup, before the parallel walk spawns. Without it, `group_begin`/`group_end`
/// are no-ops (bins that don't opt in emit ungrouped, as before).
pub fn set_group_syntax(syntax: InstalledGroupSyntax) {
    let _ = GROUP_SYNTAX.set(syntax);
}

/// Whether grouping should bracket host-effect output: a syntax is installed AND at
/// least one trace-bearing channel is actually visible. When every host-effect
/// channel is Suppressed (e.g. Quiet) there is nothing to group, so callers skip the
/// brackets and leave empty groups out of the log.
pub fn host_trace_grouping_active() -> bool {
    GROUP_SYNTAX.get().is_some()
        && (output_decision(OutputChannel::ShellTrace) != OutputDecision::Suppressed
            || output_decision(OutputChannel::Instrumentation) != OutputDecision::Suppressed)
}

/// Open a titled group on stderr — the same stream the host-effect trace lines use,
/// so the runner folds those lines under the marker. No-op when no syntax is
/// installed. Pair with `group_end`; the caller must keep the bracket tight (open →
/// run+join the effectful work → close) and defer non-trace output (PASS/FAIL) until
/// after `group_end` so it stays OUTSIDE the collapsed section.
pub fn group_begin(title: &str) {
    if let Some(s) = GROUP_SYNTAX.get() {
        eprintln!("{}{}{}", s.open_prefix, title, s.open_suffix);
    }
}

/// Close the current group. Emits the close line only when the target defines one
/// (GitHub Actions); a plain terminal closes implicitly and prints nothing.
pub fn group_end() {
    if let Some(s) = GROUP_SYNTAX.get() {
        if let Some(close) = &s.close_line {
            eprintln!("{close}");
        }
    }
}

/// Emit a host-effect trace line under its channel's installed decision:
/// Suppressed drops it, Condensed prints a dim indented summary, Full prints it
/// verbatim. The decision is the .dag authority's, not a Rust re-derivation.
fn trace_emit(channel: OutputChannel, line: &str) {
    match output_decision(channel) {
        OutputDecision::Suppressed => {}
        OutputDecision::Condensed => eprintln!("{}", paint(&format!("  {line}"), sgr::DIM)),
        OutputDecision::Full => eprintln!("{}", line),
    }
}

// The funnel for the ShellTrace channel. Consumes the installed
// `gunbc.output_policy` ShellTrace decision (Suppressed / Condensed / Full)
// rather than re-deriving it from verbosity — keeps CI logs readable instead of
// dumping every `sh -c` script.
fn render_shell_trace(argv: &[String]) {
    match output_decision(OutputChannel::ShellTrace) {
        OutputDecision::Suppressed => {}
        OutputDecision::Full => eprintln!("[shell] {}", argv.join(" ")),
        OutputDecision::Condensed => {
            // Collapse newlines/runs of whitespace into a single readable line,
            // then truncate so a multiline `sh -c` script is one tidy summary.
            let collapsed: String = argv
                .join(" ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            // Fallback column bound (no Viewport at the trace site); the single
            // authority is `gunbc.output_policy.shell_trace_summary_max_columns`.
            const MAX: usize = 100;
            let summary = if collapsed.chars().count() > MAX {
                let head: String = collapsed.chars().take(MAX).collect();
                format!("{head}…")
            } else {
                collapsed
            };
            eprintln!("{}", paint(&format!("  $ {summary}"), sgr::DIM));
        }
    }
}

fn shell_completion_trace_line(
    exit_code: i32,
    stdout_bytes: usize,
    stderr_bytes: usize,
    wall: std::time::Duration,
) -> String {
    format!(
        "[shell] done exit={exit_code} stdout={stdout_bytes} stderr={stderr_bytes} bytes wall={:.3}s",
        wall.as_secs_f64()
    )
}

/// Post-wait completion trace for every shell transport: exit, stdout/stderr bytes,
/// spawn-to-wait wall seconds. Pairs with `render_shell_trace` (pre-spawn).
///
/// On non-zero exit the captured stderr CONTENT is surfaced (tail-bounded), not just its
/// byte count: a failing op whose error text is discarded is an undiagnosable failure
/// (DESIGN §5 — a failure must be a visible, located diagnostic, never an opaque count;
/// a whole self-host build failure was invisible in CI because only `stderr=N bytes` was
/// logged). Success stays count-only so benign compiler warnings do not drown the trace.
fn render_shell_completion_trace(
    exit_code: i32,
    stdout_bytes: usize,
    stderr: &[u8],
    wall: std::time::Duration,
) {
    trace_emit(
        OutputChannel::ShellTrace,
        &shell_completion_trace_line(exit_code, stdout_bytes, stderr.len(), wall),
    );
    if let Some(block) = shell_completion_stderr_trace_block(exit_code, stderr) {
        trace_emit(OutputChannel::ShellTrace, &block);
    }
}

/// Pure tail-bounding of captured stderr for the completion trace. Returns `None` when there
/// is nothing to surface (success exit, or empty stderr); `Some(block)` is the `[shell] stderr`
/// diagnostic, its content tail-bounded with a leading elision marker when it exceeds the cap.
/// Kept pure (no `trace_emit`) so the surfacing decision is unit-testable with a RED control.
fn shell_completion_stderr_trace_block(exit_code: i32, stderr: &[u8]) -> Option<String> {
    if exit_code == 0 || stderr.is_empty() {
        return None;
    }
    const MAX_STDERR_TRACE: usize = 16384;
    let (elided, tail) = if stderr.len() > MAX_STDERR_TRACE {
        (
            stderr.len() - MAX_STDERR_TRACE,
            &stderr[stderr.len() - MAX_STDERR_TRACE..],
        )
    } else {
        (0, stderr)
    };
    let prefix = if elided > 0 {
        format!("…<{elided} earlier stderr bytes elided>…\n")
    } else {
        String::new()
    };
    Some(format!(
        "[shell] stderr (exit={exit_code}):\n{prefix}{}",
        String::from_utf8_lossy(tail)
    ))
}

fn shell_stdin_payload(val: &Value) -> InterpResult<Vec<u8>> {
    match val {
        Value::Str(s) => Ok(s.as_bytes().to_vec()),
        _ => Err(InterpError::TypeError {
            msg: format!("shell transport stdin must be String, got {val}"),
        }),
    }
}

/// Host per-argument byte ceiling. Mirrors the single authority
/// `extdeps.os.exec_arg_limit.host_exec_arg_max_strlen` (Linux execve(2)
/// MAX_ARG_STRLEN = 32 * PAGE_SIZE = 131072). A single argv (or env) string
/// longer than this makes `execve` fail with E2BIG ("Argument list too long").
/// `argv_arg_limit_test::mirror_matches_extdeps_authority` pins this to the
/// modeled value so the two cannot drift silently.
pub const HOST_ARG_MAX_STRLEN_BYTES: usize = 131072;

/// Pure argv-size wall: refuse (typed, located) an invocation whose largest
/// single argv token exceeds the host per-argument ceiling, instead of handing
/// it to `execve` and getting an opaque `os error 7`. Faithful to
/// MAX_ARG_STRLEN — the ceiling is per single argument, not the argv total
/// (that is the separate, larger ARG_MAX). Returns `None` when the invocation
/// is within the limit (proceed) and `Some(err)` when it must refuse — no
/// truncation, no widening (DESIGN §5: a failure arm refuses, never absorbs).
fn argv_arg_limit_refusal(argv: &[String], limit_bytes: usize) -> Option<InterpError> {
    let offending = argv.iter().map(|a| a.len()).max().unwrap_or(0);
    if offending > limit_bytes {
        Some(InterpError::ArgvExceedsHostArgMax {
            actual_bytes: offending,
            limit_bytes,
            argv0: argv.first().cloned().unwrap_or_default(),
        })
    } else {
        None
    }
}

fn dispatch_shell(
    transport: &Rc<Node>,
    param_env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<ShellResult> {
    let argv_nodes = &transport.children;
    let mut argv: Vec<String> = Vec::new();
    for node in argv_nodes.iter() {
        let val = eval_expr(node, param_env, ctx)?;
        push_shell_argv_tokens(&mut argv, val)?;
    }

    if argv.is_empty() {
        return Err(InterpError::TypeError {
            msg: "shell transport has empty argv".to_string(),
        });
    }

    render_shell_trace(&argv);

    // Arg-size wall: a single argv token over the host MAX_ARG_STRLEN would make
    // the spawn below die with an opaque `os error 7` (E2BIG). Refuse here with a
    // typed, located diagnostic so the deficit is diagnosable and countable. Large
    // payloads belong in stdin (see extdeps.shell shell.Exec.Run), not argv.
    if let Some(err) = argv_arg_limit_refusal(&argv, HOST_ARG_MAX_STRLEN_BYTES) {
        return Err(err);
    }

    let output = if let Some(stdin_node) = transport_stdin(transport.clone(), ctx.si()) {
        use std::io::Write;
        use std::process::Stdio;

        let stdin_val = eval_expr(&stdin_node, param_env, ctx)?;
        let stdin_bytes = shell_stdin_payload(&stdin_val)?;

        let wall_start = std::time::Instant::now();
        let mut child = std::process::Command::new(&argv[0])
            .args(&argv[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| InterpError::TypeError {
                msg: format!("failed to execute '{}': {}", argv[0], e),
            })?;

        let stdin_writer = child
            .stdin
            .take()
            .map(|mut stdin| std::thread::spawn(move || stdin.write_all(&stdin_bytes)));

        let output = child
            .wait_with_output()
            .map_err(|e| InterpError::TypeError {
                msg: format!("failed to wait on '{}': {}", argv[0], e),
            })?;

        if let Some(writer) = stdin_writer {
            // A stdin-write error (e.g. broken pipe) here is not itself the
            // failure to report: the child may have exited (successfully or
            // not) before consuming all of stdin, which is ordinary POSIX
            // pipe behavior. The child's real exit_code/stdout/stderr in
            // `output` is the authoritative result and already flows to the
            // `exit { .. }` clause in the .dag transport declaration.
            let _ = writer.join().map_err(|_| InterpError::TypeError {
                msg: format!("shell transport stdin writer for '{}' panicked", argv[0]),
            })?;
        }
        render_shell_completion_trace(
            output.status.code().unwrap_or(-1),
            output.stdout.len(),
            &output.stderr,
            wall_start.elapsed(),
        );
        output
    } else {
        let wall_start = std::time::Instant::now();
        let output = std::process::Command::new(&argv[0])
            .args(&argv[1..])
            .output()
            .map_err(|e| InterpError::TypeError {
                msg: format!("failed to execute '{}': {}", argv[0], e),
            })?;
        render_shell_completion_trace(
            output.status.code().unwrap_or(-1),
            output.stdout.len(),
            &output.stderr,
            wall_start.elapsed(),
        );
        output
    };

    let exit_code = output.status.code().unwrap_or(-1);

    Ok(ShellResult {
        exit_code,
        stdout: String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string(),
        stderr: String::from_utf8_lossy(&output.stderr)
            .trim_end()
            .to_string(),
    })
}

fn map_shell_outputs(
    result: &ShellResult,
    op_node: &Rc<Node>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let return_type = match op_node.inferred.as_deref() {
        Some(crate::v1_std_core::InferredNode::Resolved { node }) => node.clone(),
        _ => {
            return Ok(Value::Str(result.stdout.clone()));
        }
    };

    let children = &return_type.children;
    if children.is_empty() {
        return Ok(Value::Unit);
    }

    let mut fields: Vec<(Symbol, Value)> = Vec::new();
    for child in children.iter() {
        let field_name = authored_name_at(ctx.si(), child.clone());
        let from_key = extract_from_key(child, ctx);
        let is_optional_field = child.return_cardinality == Cardinality::CardOptional;
        let value = match from_key.as_deref() {
            Some("stdout") if is_optional_field && result.exit_code != 0 => Value::Null,
            Some("stderr") if is_optional_field && result.exit_code != 0 => Value::Null,
            Some("stdout") => Value::Str(result.stdout.clone()),
            Some("stderr") => Value::Str(result.stderr.clone()),
            Some("exit_success") => Value::Bool(result.exit_code == 0),
            Some("exit_code") => Value::Int(result.exit_code as i64),
            Some("stdout_lines") => {
                let lines: Vec<Value> = result
                    .stdout
                    .lines()
                    .map(|l| Value::Str(l.to_string()))
                    .collect();
                list_value((lines))
            }
            _ => match field_name.as_str() {
                "success" => Value::Bool(result.exit_code == 0),
                "exit_code" => Value::Int(result.exit_code as i64),
                "stdout" => Value::Str(result.stdout.clone()),
                "stderr" => Value::Str(result.stderr.clone()),
                "exists" => Value::Bool(result.exit_code == 0),
                _ => Value::Null,
            },
        };
        fields.push((ctx.sym(&field_name), value));
    }
    fields.sort_unstable_by_key(|(k, _)| k.0);

    Ok(Value::Record {
        type_name: ctx.sym(&authored_name_at(ctx.si(), op_node.clone())),
        fields: Rc::new(fields),
    })
}

fn extract_from_key(field_node: &Rc<Node>, ctx: &InterpContext) -> Option<String> {
    for prop in field_node.properties.iter() {
        let prop_name = field_init_node_name_at(prop.clone(), ctx.si());
        if prop_name == "from_key" || prop_name == "from" {
            let val_node = field_init_node_value(prop.clone());
            if let ExprData::ExprLiteral { ref value } = *val_node.expr_data {
                if let LiteralValue::LitStr { value: s } = value.as_ref() {
                    return Some(s.clone());
                }
            }
        }
    }
    None
}

struct FileResult {
    success: bool,
    byte_count: i64,
    path: String,
    error: String,
    content: String,
}

fn dispatch_file(
    op_node: &Rc<Node>,
    transport: &Rc<Node>,
    param_env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<FileResult> {
    let si = ctx.si();

    let path = match find_property(
        transport.properties.clone(),
        "base_path".to_string(),
        si.clone(),
    ) {
        Some(path_node) => {
            let path_val = eval_expr(&path_node, param_env, ctx)?;
            substitute_template(&format!("{}", path_val), param_env, ctx)
        }
        None => {
            return Err(InterpError::TypeError {
                msg: "file transport has no path".to_string(),
            })
        }
    };
    if path.is_empty() {
        return Err(InterpError::TypeError {
            msg: "file transport resolved to an empty path".to_string(),
        });
    }

    // Optional explicit verb on the transport row (`transport file { path: ..., verb: "delete" }`).
    // Delete/List are structurally indistinguishable from Read (path-only inputs), so the
    // transport — the realization — declares its own action; absent verb keeps the original
    // content-param convention (write iff a `content` param exists, else read).
    let verb = find_property(transport.properties.clone(), "verb".to_string(), si.clone())
        .map(|verb_node| eval_expr(&verb_node, param_env, ctx).map(|v| format!("{}", v)))
        .transpose()?;

    if let Some(verb) = verb.as_deref() {
        match verb {
            "delete" => {
                trace_emit(
                    OutputChannel::ShellTrace,
                    &format!("[file] delete {}", path),
                );
                return match std::fs::remove_file(&path) {
                    Ok(()) => Ok(FileResult {
                        success: true,
                        byte_count: 0,
                        path,
                        error: String::new(),
                        content: String::new(),
                    }),
                    Err(e) => Ok(FileResult {
                        success: false,
                        byte_count: 0,
                        path,
                        error: format!("{}", e),
                        content: String::new(),
                    }),
                };
            }
            "list" => {
                trace_emit(
                    OutputChannel::Instrumentation,
                    &format!("[file] list {}", path),
                );
                return match std::fs::read_dir(&path) {
                    Ok(entries) => {
                        let mut names: Vec<String> = entries
                            .filter_map(|e| e.ok())
                            .filter_map(|e| e.file_name().into_string().ok())
                            .collect();
                        names.sort();
                        let content = names.join("\n");
                        Ok(FileResult {
                            success: true,
                            byte_count: content.len() as i64,
                            path,
                            error: String::new(),
                            content,
                        })
                    }
                    Err(e) => Ok(FileResult {
                        success: false,
                        byte_count: 0,
                        path,
                        error: format!("{}", e),
                        content: String::new(),
                    }),
                };
            }
            other => {
                return Err(InterpError::TypeError {
                    msg: format!(
                        "file transport verb '{other}' is not a known action (delete, list)"
                    ),
                })
            }
        }
    }

    let has_content = op_node
        .params
        .iter()
        .any(|p| param_node_name_at(p.clone(), si.clone()) == "content");

    if has_content {
        let content = match param_env.lookup(ctx.sym("content")) {
            Some(v) => format!("{}", v),
            None => {
                return Err(InterpError::TypeError {
                    msg: format!(
                        "file write operation missing `content` argument for {}",
                        path
                    ),
                })
            }
        };
        let byte_count = content.len() as i64;
        trace_emit(
            OutputChannel::ShellTrace,
            &format!("[file] write {} ({} bytes)", path, byte_count),
        );
        match std::fs::write(&path, content.as_bytes()) {
            Ok(()) => Ok(FileResult {
                success: true,
                byte_count,
                path,
                error: String::new(),
                content: String::new(),
            }),
            Err(e) => Ok(FileResult {
                success: false,
                byte_count: 0,
                path,
                error: format!("{}", e),
                content: String::new(),
            }),
        }
    } else {
        trace_emit(
            OutputChannel::Instrumentation,
            &format!("[file] read {}", path),
        );
        match std::fs::read_to_string(&path) {
            Ok(s) => Ok(FileResult {
                success: true,
                byte_count: s.len() as i64,
                path,
                error: String::new(),
                content: s,
            }),
            Err(e) => Ok(FileResult {
                success: false,
                byte_count: 0,
                path,
                error: format!("{}", e),
                content: String::new(),
            }),
        }
    }
}

fn map_file_outputs(
    result: &FileResult,
    op_node: &Rc<Node>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let return_type = match op_node.inferred.as_deref() {
        Some(crate::v1_std_core::InferredNode::Resolved { node }) => node.clone(),
        _ => {
            if result.content.is_empty() {
                return Ok(Value::Bool(result.success));
            }
            return Ok(Value::Str(result.content.clone()));
        }
    };

    let children = &return_type.children;
    if children.is_empty() {
        return Ok(Value::Unit);
    }

    let mut fields: Vec<(Symbol, Value)> = Vec::new();
    for child in children.iter() {
        let field_name = authored_name_at(ctx.si(), child.clone());
        let from_key = extract_from_key(child, ctx);
        let key = from_key.as_deref().unwrap_or(field_name.as_str());
        let value = match key {
            "write_success" | "success" => Value::Bool(result.success),
            "bytes_written" | "bytes" | "byte_count" => Value::Int(result.byte_count),
            "path" => Value::Str(result.path.clone()),
            "error" => Value::Str(result.error.clone()),
            "content" => Value::Str(result.content.clone()),
            _ => Value::Null,
        };
        fields.push((ctx.sym(&field_name), value));
    }
    fields.sort_unstable_by_key(|(k, _)| k.0);

    Ok(Value::Record {
        type_name: ctx.sym(&authored_name_at(ctx.si(), op_node.clone())),
        fields: Rc::new(fields),
    })
}

/// Standard base64 (RFC 4648 §4, with `=` padding) — the fixed alphabet an HTTP Basic
/// credential requires (RFC 7617). Hand-rolled to keep the shrinking bootstrap seed free of a
/// direct base64 dependency; deterministic and total over any byte slice.
fn base64_encode_std(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(ALPHABET[((n >> 18) & 0x3f) as usize] as char);
        out.push(ALPHABET[((n >> 12) & 0x3f) as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[((n >> 6) & 0x3f) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[(n & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// The `Authorization: Basic …` header value for RFC 7617 credentials — the single-authority
/// derivation dispatch_rest uses for a transport `auth_basic` property. Pure so it is
/// execution-witnessable without a live server.
fn rest_basic_auth_header_value(username: &str, password: &str) -> String {
    format!(
        "Basic {}",
        base64_encode_std(format!("{}:{}", username, password).as_bytes())
    )
}

/// The interpreter's disposition for a rest transport `tls:` posture (extdeps.transports.rest
/// TlsPosture). `VerifyPeer` proceeds on the stock verifier; `InsecureAcceptAnyCert` is realized
/// emit-only (operator decision 2026-07-16) so the interpreter refuses it rather than carry a
/// cert-verification bypass into the retiring seed; an unrecognized posture also refuses. Pure so
/// each arm is execution-witnessable.
fn rest_tls_posture_interp_disposition(posture: &str) -> Result<(), String> {
    match posture {
        "VerifyPeer" => Ok(()),
        "InsecureAcceptAnyCert" => Err(
            "rest transport tls: InsecureAcceptAnyCert is realized emit-only (reqwest \
             danger_accept_invalid_certs); the interpreter refuses it by design rather than carry \
             a cert-verification bypass into the retiring seed — run such ops through emitted code"
                .to_string(),
        ),
        other => Err(format!(
            "rest transport tls: unrecognized posture '{}'",
            other
        )),
    }
}

/// The single-authority rule (§3): a rest operation must not declare both config-level auth and a
/// transport `auth_basic` property. Pure so the conflict rule is execution-witnessable.
fn rest_auth_authority_conflict(config_auth_resolved: bool, has_auth_basic: bool) -> bool {
    config_auth_resolved && has_auth_basic
}

fn dispatch_rest(
    service_node: &Rc<Node>,
    op_node: &Rc<Node>,
    transport: &Rc<Node>,
    param_env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let si = ctx.si();

    let base_url =
        find_service_config_string(service_node, "svc_endpoint", &si).unwrap_or_default();

    let path = match find_property(transport.properties.clone(), "path".to_string(), si.clone()) {
        Some(path_node) => {
            let path_val = eval_expr(&path_node, param_env, ctx)?;
            let path_str = format!("{}", path_val);
            substitute_template(&path_str, param_env, ctx)
        }
        None => String::new(),
    };

    let url = if path.is_empty() {
        base_url
    } else {
        format!("{}{}", base_url, path)
    };

    let method = match find_property(
        transport.properties.clone(),
        "method".to_string(),
        si.clone(),
    ) {
        Some(m_node) => {
            if let ExprData::ExprLiteral { ref value } = *m_node.expr_data {
                if let LiteralValue::LitStr { value: s } = value.as_ref() {
                    s.clone().to_uppercase()
                } else {
                    authored_name_at(si.clone(), m_node).to_uppercase()
                }
            } else {
                authored_name_at(si.clone(), m_node).to_uppercase()
            }
        }
        None => "GET".to_string(),
    };

    let auth = resolve_auth(service_node, transport, param_env, &si, ctx);
    if let AuthResolution::DeclaredButUnwired { ref reason } = auth {
        return Err(InterpError::AuthDeclaredButUnwired {
            service: find_service_config_string(service_node, "svc_endpoint", &si)
                .unwrap_or_else(|| "<unknown>".to_string()),
            reason: reason.clone(),
        });
    }

    let reserved_props = [
        "base_url",
        "method",
        "path",
        "body",
        "query",
        "response_format",
        "auth_token",
        "auth_header",
        "auth_basic",
        "tls",
        "stdin",
    ];
    let mut headers: Vec<(String, String)> = Vec::new();
    for prop in transport.properties.iter() {
        let pname = field_init_node_name_at(prop.clone(), si.clone());
        if !reserved_props.contains(&pname.as_str()) {
            let pval = eval_expr(&field_init_node_value(prop.clone()), param_env, ctx)?;
            headers.push((pname, format!("{}", pval)));
        }
    }

    let mut query_params: Vec<(String, String)> = Vec::new();
    if let Some(query_record) = find_property(
        transport.properties.clone(),
        "query".to_string(),
        si.clone(),
    ) {
        for child in query_record.children.iter() {
            let qname = field_init_node_name_at(child.clone(), si.clone());
            let qval = eval_expr(&field_init_node_value(child.clone()), param_env, ctx)?;
            match &qval {
                Value::Null => {}
                _ => query_params.push((qname, format!("{}", qval))),
            }
        }
    }

    let body_json =
        match find_property(transport.properties.clone(), "body".to_string(), si.clone()) {
            Some(body_node) => {
                let body_val = eval_expr(&body_node, param_env, ctx)?;
                Some(
                    value_to_wire_json(&body_val, ctx)
                        .map_err(|msg| InterpError::TypeError { msg })?,
                )
            }
            None => None,
        };

    let response_format = find_property_string(
        transport.properties.clone(),
        "response_format".to_string(),
        si.clone(),
    )
    .unwrap_or_else(|| "Json".to_string());

    // TLS posture (extdeps.transports.rest TlsPosture). Absent = VerifyPeer, the fail-closed
    // default (ureq's stock rustls verifier). InsecureAcceptAnyCert is the modeled dissolution of
    // curl's `-k` for self-signed BMC endpoints. Realization is EMIT-ONLY by decision (operator,
    // 2026-07-16): emitted reqwest code realizes it via `.danger_accept_invalid_certs(true)`, but
    // the interpreter refuses it by design rather than carry an accept-any rustls verifier into the
    // bootstrap seed the self-host is retiring. So a present InsecureAcceptAnyCert is a typed
    // refusal here — the interp is not a realization path for insecure-TLS ops (redfish etc. run
    // through emitted code); it fails closed, never a silent no-op that would send under VerifyPeer
    // while the row asked for insecure. An unrecognized posture also refuses.
    if let Some(tls_node) =
        find_property(transport.properties.clone(), "tls".to_string(), si.clone())
    {
        let posture = authored_name_at(si.clone(), tls_node);
        if let Err(msg) = rest_tls_posture_interp_disposition(&posture) {
            return Err(InterpError::TypeError { msg });
        }
    }

    trace_emit(
        OutputChannel::ShellTrace,
        &format!("[rest] {} {}", method, url),
    );

    let mut request = match method.as_str() {
        "GET" => ureq::get(&url),
        "POST" => ureq::post(&url),
        "PUT" => ureq::put(&url),
        "DELETE" => ureq::delete(&url),
        "PATCH" => ureq::patch(&url),
        _ => {
            return Err(InterpError::TypeError {
                msg: format!("unsupported HTTP method: {}", method),
            })
        }
    };

    if let AuthResolution::Resolved {
        ref header,
        ref token,
    } = auth
    {
        if !token.is_empty() {
            let header_val = if header == "Authorization" {
                format!("Bearer {}", token)
            } else {
                token.clone()
            };
            request = request.set(header, &header_val);
        }
    }

    // Basic auth (RFC 7617), realized from a `auth_basic: { username: <input>, password: <input> }`
    // transport-block property. This is the modeled dissolution of curl's `-u user:pass` / netrc
    // argv — the credential never touches a process argv or a temp file, and its header value is
    // derived in exactly one place (§3 rest_auth_value_single_authority_note). Fail-closed: a
    // present `auth_basic` with a non-record shape, a missing username/password field, or a
    // non-Str credential value is a typed refusal, never an unauthenticated send or a
    // stringified-debug header.
    if let Some(basic_node) = find_property(
        transport.properties.clone(),
        "auth_basic".to_string(),
        si.clone(),
    ) {
        if rest_auth_authority_conflict(matches!(auth, AuthResolution::Resolved { .. }), true) {
            return Err(InterpError::TypeError {
                msg: "rest transport declares both config-level auth and auth_basic — one auth \
                      authority per operation (§3)"
                    .to_string(),
            });
        }
        let mut username: Option<String> = None;
        let mut password: Option<String> = None;
        for child in basic_node.children.iter() {
            let fname = field_init_node_name_at(child.clone(), si.clone());
            let fval = eval_expr(&field_init_node_value(child.clone()), param_env, ctx)?;
            match (fname.as_str(), &fval) {
                ("username", Value::Str(s)) => username = Some(s.clone()),
                ("password", Value::Str(s)) => password = Some(s.clone()),
                ("username", _) | ("password", _) => {
                    return Err(InterpError::TypeError {
                        msg: format!(
                            "auth_basic.{} must resolve to a String credential, got {}",
                            fname, fval
                        ),
                    });
                }
                _ => {}
            }
        }
        match (username, password) {
            (Some(u), Some(p)) => {
                let header_val = rest_basic_auth_header_value(&u, &p);
                request = request.set("Authorization", &header_val);
            }
            _ => {
                return Err(InterpError::TypeError {
                    msg: "auth_basic requires both username and password fields".to_string(),
                });
            }
        }
    }

    for (name, val) in &headers {
        request = request.set(name, val);
    }

    for (name, val) in &query_params {
        request = request.query(name, val);
    }

    let response = if let Some(json) = body_json {
        request
            .set("Content-Type", "application/json")
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
            let json: serde_json::Value =
                serde_json::from_str(&body).unwrap_or(serde_json::Value::String(body));
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

pub fn resolve_auth(
    service_node: &Rc<Node>,
    _transport: &Rc<Node>,
    param_env: &Rc<Env>,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    ctx: &InterpContext,
) -> AuthResolution {
    let mut header_name = "Authorization".to_string();
    let mut env_var_name: Option<String> = None;
    // `auth_input: <field>` (§3): the token is an operation INPUT the caller supplies,
    // not ambient env. Resolve it from the per-call param env. Takes precedence over
    // `auth_source` (env var) when both are present.
    let mut input_field_name: Option<String> = None;
    let mut auth_declared = false;

    for prop in service_node.properties.iter() {
        let name = field_init_node_name_at(prop.clone(), si.clone());
        let val_node = field_init_node_value(prop.clone());

        match name.as_str() {
            "svc_auth" => {
                auth_declared = true;
                let scheme = authored_name_at(si.clone(), val_node.clone());
                if scheme == "Bearer" {
                    header_name = "Authorization".to_string();
                } else if scheme == "Header" || val_node.name == "Header" {
                    for child in val_node.children.iter() {
                        if let Some(s) = extract_string_value(child) {
                            header_name = s;
                        } else {
                            for grandchild in child.children.iter() {
                                if let Some(s) = extract_string_value(grandchild) {
                                    header_name = s;
                                }
                            }
                        }
                    }
                }
            }
            "svc_auth_input" => {
                auth_declared = true;
                // `auth_input: access_token` — the value node is the input field name (an identifier).
                let field = authored_name_at(si.clone(), val_node.clone());
                if !field.is_empty() {
                    input_field_name = Some(field);
                } else {
                    input_field_name = extract_string_value(&val_node);
                }
            }
            "svc_auth_source" => {
                auth_declared = true;
                for child in val_node.children.iter() {
                    let field_name = field_init_node_name_at(child.clone(), si.clone());
                    if field_name == "name" {
                        let field_val = field_init_node_value(child.clone());
                        env_var_name = extract_string_value(&field_val);
                    }
                }
                if env_var_name.is_none() {
                    env_var_name = extract_string_value(&val_node);
                }
            }
            _ => {}
        }
    }

    if !auth_declared {
        return AuthResolution::NoAuthDeclared;
    }

    // §3: caller-supplied input token wins over ambient env var when non-empty; if the input
    // field is absent or empty, fall through to auth_source so dual-declare services
    // (auth_input + auth_source) get the env-var fallback.  Extract the String payload
    // explicitly — a non-Str Value must NOT produce a stringified-debug Bearer header.
    if let Some(ref field) = input_field_name {
        if let Some(Value::Str(tok)) = param_env.lookup(ctx.sym(field)) {
            if !tok.is_empty() {
                return AuthResolution::Resolved {
                    header: header_name,
                    token: tok.clone(),
                };
            }
        }
        // input field unresolved or empty — fall through to auth_source attempt below.
    }

    match env_var_name.and_then(|var| resolve_env_var_token(ctx, &var)) {
        Some(tok) if !tok.is_empty() => AuthResolution::Resolved {
            header: header_name,
            token: tok,
        },
        _ => AuthResolution::DeclaredButUnwired {
            reason: "auth declared but no token resolved (auth_input unresolved/empty, \
                     auth_source env var absent or empty)"
                .to_string(),
        },
    }
}

fn extract_string_value(node: &Rc<Node>) -> Option<String> {
    if let ExprData::ExprLiteral { ref value } = *node.expr_data {
        if let LiteralValue::LitStr { value: s } = value.as_ref() {
            return Some(s.clone());
        }
    }
    None
}

fn find_service_config_string(
    service_node: &Rc<Node>,
    key: &str,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<String> {
    for prop in service_node.properties.iter() {
        let name = field_init_node_name_at(prop.clone(), si.clone());
        if name == key {
            let val_node = field_init_node_value(prop.clone());
            if let ExprData::ExprLiteral { ref value } = *val_node.expr_data {
                if let LiteralValue::LitStr { value: s } = value.as_ref() {
                    return Some(s.clone());
                }
            }
            let authored = authored_name_at(si.clone(), val_node);
            if !authored.is_empty() {
                return Some(authored);
            }
        }
    }
    None
}

fn substitute_template(template: &str, env: &Rc<Env>, ctx: &InterpContext) -> String {
    let mut result = String::new();
    let mut chars = template.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '{' {
            let mut var_name = String::new();
            for c2 in chars.by_ref() {
                if c2 == '}' {
                    break;
                }
                var_name.push(c2);
            }
            if let Some(val) = env.lookup(ctx.sym(&var_name)) {
                result.push_str(&value_to_host_string(&val));
            } else {
                result.push('{');
                result.push_str(&var_name);
                result.push('}');
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn value_to_json(val: &Value) -> InterpResult<serde_json::Value> {
    Ok(match val {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(n) => serde_json::json!(*n),
        Value::Float(f) => serde_json::json!(*f),
        Value::Str(s) => {
            if s.starts_with('[') || s.starts_with('{') {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(s) {
                    return Ok(parsed);
                }
            }
            serde_json::Value::String(s.clone())
        }
        Value::List(items) => {
            let mut arr = Vec::with_capacity(items.len());
            for item in items.iter() {
                arr.push(value_to_json(item)?);
            }
            serde_json::Value::Array(arr)
        }
        Value::Set(members) => serde_json::Value::Array(
            members
                .iter()
                .map(|s| serde_json::Value::String(s.clone()))
                .collect(),
        ),
        Value::Map(m) => {
            let mut obj = serde_json::Map::with_capacity(m.len());
            for (k, v) in m.iter() {
                let key = match &k.key {
                    Value::Str(s) => s.clone(),
                    other => {
                        return Err(InterpError::TypeError {
                            msg: format!(
                                "cannot serialize map with non-string key to JSON (got {} key); \
                                 JSON object keys are strings",
                                other.type_label()
                            ),
                        })
                    }
                };
                obj.insert(key, value_to_json(v)?);
            }
            serde_json::Value::Object(obj)
        }
        Value::Record { fields, .. } => {
            let mut obj = serde_json::Map::new();
            for (k, v) in fields.iter() {
                if matches!(v, Value::Null) {
                    continue;
                }
                obj.insert(resolve_sym(*k), value_to_json(v)?);
            }
            serde_json::Value::Object(obj)
        }
        Value::Variant { .. } => {
            return Err(InterpError::TypeError {
                msg: "value_to_json must not serialize coproduct variants; use value_to_wire_json"
                    .to_string(),
            });
        }
        Value::Unit => serde_json::Value::Null,
        Value::Closure { .. } => serde_json::Value::String("<closure>".to_string()),
        Value::Fn { node } => serde_json::Value::String(format!("<fn {}>", node.name)),
    })
}

fn map_response_to_value(
    text: &str,
    _json: Option<&serde_json::Value>,
    op_node: &Rc<Node>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let return_type = match op_node.inferred.as_deref() {
        Some(crate::v1_std_core::InferredNode::Resolved { node }) => node.clone(),
        _ => return Ok(Value::Str(text.to_string())),
    };
    let children = &return_type.children;
    if children.is_empty() {
        return Ok(Value::Str(text.to_string()));
    }
    if children.len() == 1 {
        return Ok(Value::Str(text.to_string()));
    }
    let mut fields: Vec<(Symbol, Value)> = Vec::new();
    for child in children.iter() {
        let field_name = authored_name_at(ctx.si(), child.clone());
        fields.push((ctx.sym(&field_name), Value::Str(text.to_string())));
    }
    fields.sort_unstable_by_key(|(k, _)| k.0);
    Ok(Value::Record {
        type_name: ctx.sym(&authored_name_at(ctx.si(), op_node.clone())),
        fields: Rc::new(fields),
    })
}

fn map_response_to_value_json(
    json: &serde_json::Value,
    op_node: &Rc<Node>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let return_type = match op_node.inferred.as_deref() {
        Some(crate::v1_std_core::InferredNode::Resolved { node }) => node.clone(),
        _ => return Ok(json_to_value(json)),
    };
    let children = &return_type.children;
    if children.is_empty() {
        return Ok(json_to_value(json));
    }

    let type_name = authored_name_at(ctx.si(), return_type.clone());
    if type_name == "List" && children.is_empty() {
        return Ok(json_to_value(json));
    }

    if json.is_array() && !children.is_empty() {
        let first_field = authored_name_at(ctx.si(), children[0].clone());
        return Ok(Value::Record {
            type_name: ctx.sym(&authored_name_at(ctx.si(), op_node.clone())),
            fields: Rc::new(vec![(ctx.sym(&first_field), json_to_value(json))]),
        });
    }

    let mut fields: Vec<(Symbol, Value)> = Vec::new();
    for child in children.iter() {
        let field_name = authored_name_at(ctx.si(), child.clone());
        let from_key = extract_from_key(child, ctx);
        let val = match from_key {
            Some(path) => {
                let pointer = format!("/{}", path);
                match json.pointer(&pointer) {
                    Some(v) => json_to_value(v),
                    None => Value::Null,
                }
            }
            None => match json.get(&field_name) {
                Some(v) => json_to_value(v),
                None => {
                    if children.len() == 1 {
                        json_to_value(json)
                    } else {
                        Value::Null
                    }
                }
            },
        };
        fields.push((ctx.sym(&field_name), val));
    }
    fields.sort_unstable_by_key(|(k, _)| k.0);

    Ok(Value::Record {
        type_name: ctx.sym(&authored_name_at(ctx.si(), op_node.clone())),
        fields: Rc::new(fields),
    })
}

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
            list_value(arr.iter().map(json_to_value).collect::<Vec<_>>())
        }
        serde_json::Value::Object(obj) => {
            let fields: HamtMap<CanonKey, Value> = obj
                .iter()
                .filter_map(|(k, v)| {
                    CanonKey::new(Value::Str(k.clone())).map(|ck| (ck, json_to_value(v)))
                })
                .collect();
            map_value(fields)
        }
    }
}

fn type_annotation_names(ctx: &InterpContext, ty: &Rc<Node>, target: &str) -> bool {
    if ty.name == target || authored_name_at(ctx.si(), ty.clone()) == target {
        return true;
    }
    ty.children
        .iter()
        .any(|c| type_annotation_names(ctx, c, target))
        || ty
            .params
            .iter()
            .any(|c| type_annotation_names(ctx, c, target))
}

pub(crate) fn resolve_published_mock_keys(
    ctx: &InterpContext,
) -> InterpResult<std::collections::HashSet<String>> {
    let mut keys = std::collections::HashSet::new();
    for (name, info) in ctx.item_registry.iter() {
        if info.kind != ItemKind::DataItem {
            continue;
        }
        let Some(node) = ctx.lookup_fn(name) else {
            continue;
        };
        let Some(ty) = node.type_annotation.as_ref() else {
            continue;
        };
        if !type_annotation_names(ctx, ty, "PublishedMockCase") {
            continue;
        }
        let Some(body) = node.body.as_ref() else {
            continue;
        };
        let val = eval_expr(body, &Env::empty(), ctx)?;
        let Value::List(items) = &val else {
            continue;
        };
        for item in items.iter() {
            if let Value::Record { type_name, fields } = item {
                if ctx.sym_eq(*type_name, "PublishedMockCase") {
                    if let Some(op_key) = published_case_operation_key(ctx, fields) {
                        keys.insert(op_key);
                    }
                }
            }
        }
    }
    Ok(keys)
}

fn published_case_operation_key(ctx: &InterpContext, fields: &[(Symbol, Value)]) -> Option<String> {
    if let (Some(Value::Str(svc)), Some(Value::Str(op))) =
        (ctx.field(fields, "service"), ctx.field(fields, "operation"))
    {
        if !svc.is_empty() && !op.is_empty() {
            return Some(format!("{svc}.{op}"));
        }
    }
    ctx.field(fields, "operation_key").and_then(|v| match v {
        Value::Str(s) if !s.is_empty() => Some(s.clone()),
        _ => None,
    })
}

fn eval_mock_response(op_node: &Rc<Node>, ctx: &InterpContext) -> InterpResult<Value> {
    for prop in op_node.properties.iter() {
        let prop_name = field_init_node_name_at(prop.clone(), ctx.si());
        if has_mock_prefix(prop_name) {
            let val_node = field_init_node_value(prop.clone());
            return eval_expr(&val_node, &Env::empty(), ctx);
        }
    }
    let op_name = authored_name_at(ctx.si(), op_node.clone());
    Err(InterpError::TypeError {
        msg: format!(
            "hermetic mode: no mock_response for operation {op_name} — refusing to fabricate Unit"
        ),
    })
}

fn eval_filesystem_read_builtin(path: String, ctx: &InterpContext) -> InterpResult<Value> {
    if !ctx.service_ops.contains_key("Filesystem.Read") {
        return Err(if ctx.execution_mode.is_hermetic() {
            InterpError::TypeError {
                msg:
                    "hermetic mode: filesystem_read requires Filesystem.Read in the import \
                      closure (import extdeps.filesystem.filesystem_io) — refusing direct disk read"
                        .to_string(),
            }
        } else {
            InterpError::Unimplemented {
                what: "filesystem_read requires Filesystem.Read in the import closure \
                       (import extdeps.filesystem.filesystem_io)"
                    .to_string(),
            }
        });
    }

    let args = [(Some("path".to_string()), Value::Str(path))];
    let result = eval_service_call("Filesystem", "Read", &args, &Env::empty(), ctx)?;

    let (content, success, error) = match result {
        Value::Record { fields, .. } => {
            let success = matches!(ctx.field(&fields, "success"), Some(Value::Bool(true)));
            let content = match ctx.field(&fields, "content") {
                Some(Value::Str(s)) => s.clone(),
                _ => String::new(),
            };
            let error = match ctx.field(&fields, "error") {
                Some(Value::Str(s)) => s.clone(),
                _ => String::new(),
            };
            (content, success, error)
        }
        _ => {
            return Err(InterpError::TypeError {
                msg: "filesystem_read: Filesystem.Read returned non-record".to_string(),
            });
        }
    };

    if !success {
        return Err(InterpError::TypeError {
            msg: if error.is_empty() {
                "filesystem_read: read failed".to_string()
            } else {
                format!("filesystem_read: {error}")
            },
        });
    }

    Ok(Value::Record {
        type_name: ctx.sym("FilesystemReadResult"),
        fields: Rc::new(vec![(ctx.sym("content"), Value::Str(content))]),
    })
}

/// Host tap for `v2.compiler.emit_host.run_host_process` (kernel-D emit_host transport):
/// materialize a workspace from resolved `{path, text}` rows, run the build argvs then the
/// run argv with typed argv (no shell), and return exit/stdout/stderr/build-log as data.
/// Wet-mode only — hermetic execution refuses instead of mocking (no fabricated receipt).
fn eval_emit_host_run_transport_builtin(
    files_arg: Option<&Value>,
    build_arg: Option<&Value>,
    run_arg: Option<&Value>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    ctx.effect_dispatch_count
        .set(ctx.effect_dispatch_count.get().wrapping_add(1));
    if ctx.execution_mode.is_hermetic() {
        return Err(InterpError::TypeError {
            msg: "hermetic mode: emit_host_run_transport refuses host process execution \
                  (no mock arm; run wet or record a fixture)"
                .to_string(),
        });
    }

    let files_val = files_arg.ok_or_else(|| InterpError::TypeError {
        msg: "emit_host_run_transport requires (files, build, run) arguments".to_string(),
    })?;
    let files = free_monoid_to_vec(files_val).ok_or_else(|| InterpError::TypeError {
        msg: "emit_host_run_transport: files must be a List of {path, text} records".to_string(),
    })?;
    let mut workspace_files: Vec<(String, String)> = Vec::with_capacity(files.len());
    for f in &files {
        match f {
            Value::Record { fields, .. } => {
                let path = ctx
                    .field(fields, "path")
                    .and_then(free_monoid_to_string)
                    .ok_or_else(|| InterpError::TypeError {
                        msg: "emit_host_run_transport: workspace file missing String path"
                            .to_string(),
                    })?;
                let text = ctx
                    .field(fields, "text")
                    .and_then(free_monoid_to_string)
                    .ok_or_else(|| InterpError::TypeError {
                        msg: "emit_host_run_transport: workspace file missing String text"
                            .to_string(),
                    })?;
                workspace_files.push((path, text));
            }
            other => {
                return Err(InterpError::TypeError {
                    msg: format!(
                        "emit_host_run_transport: workspace entry must be a record, got {}",
                        other.type_label()
                    ),
                })
            }
        }
    }

    let build_val = build_arg.ok_or_else(|| InterpError::TypeError {
        msg: "emit_host_run_transport requires (files, build, run) arguments".to_string(),
    })?;
    let build_lists = free_monoid_to_vec(build_val).ok_or_else(|| InterpError::TypeError {
        msg: "emit_host_run_transport: build must be a List of argv Lists".to_string(),
    })?;
    let mut build_argvs: Vec<Vec<String>> = Vec::with_capacity(build_lists.len());
    for argv_val in &build_lists {
        let argv_items = free_monoid_to_vec(argv_val).ok_or_else(|| InterpError::TypeError {
            msg: "emit_host_run_transport: build argv must be a List<String>".to_string(),
        })?;
        let mut argv: Vec<String> = Vec::with_capacity(argv_items.len());
        for item in &argv_items {
            let s = free_monoid_to_string(item).ok_or_else(|| InterpError::TypeError {
                msg: format!(
                    "emit_host_run_transport: build argv element must be String, got {}",
                    item.type_label()
                ),
            })?;
            argv.push(s);
        }
        if argv.is_empty() {
            return Err(InterpError::TypeError {
                msg: "emit_host_run_transport: empty build argv".to_string(),
            });
        }
        build_argvs.push(argv);
    }

    let run_arg_items =
        run_arg
            .and_then(free_monoid_to_vec)
            .ok_or_else(|| InterpError::TypeError {
                msg: "emit_host_run_transport: run must be a List<String>".to_string(),
            })?;
    let mut run_argv: Vec<String> = Vec::with_capacity(run_arg_items.len());
    for item in &run_arg_items {
        let s = free_monoid_to_string(item).ok_or_else(|| InterpError::TypeError {
            msg: format!(
                "emit_host_run_transport: run argv element must be String, got {}",
                item.type_label()
            ),
        })?;
        run_argv.push(s);
    }
    if run_argv.is_empty() {
        return Err(InterpError::TypeError {
            msg: "emit_host_run_transport: empty run argv".to_string(),
        });
    }

    use std::sync::atomic::{AtomicU64, Ordering};
    static EMIT_HOST_WORKSPACE_COUNTER: AtomicU64 = AtomicU64::new(0);
    let workspace = std::env::temp_dir().join(format!(
        "gunbc-emit-host-{}-{}",
        std::process::id(),
        EMIT_HOST_WORKSPACE_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&workspace).map_err(|e| InterpError::TypeError {
        msg: format!("emit_host_run_transport: workspace create failed: {e}"),
    })?;

    let result = emit_host_run_transport_in_workspace(
        &workspace,
        &workspace_files,
        &build_argvs,
        &run_argv,
        ctx,
    );
    if let Err(cleanup) = std::fs::remove_dir_all(&workspace) {
        eprintln!(
            "emit_host_run_transport: workspace cleanup failed ({}): {cleanup}",
            workspace.display()
        );
    }
    result
}

/// SCAFFOLD (§7 seed-retained HAND-RUST — authority: gunbc.v1_deletion_plan
/// ^witness_realization_kernel; receipt: dag/std/emit_on_demand.dag P3 kernel +
/// extdeps.realization.emit_on_demand_host + emit_on_demand_kernel_witness_test):
/// content-addressed emit_host transport persists workspace under workspace_dir and
/// skips build when `.native_ready` is present. workspace_dir is the pre-composed
/// path (native_cache_workspace_root(cache_root, key)); callers must not pass the
/// cache parent alone. Workspace reuse is keyed by the caller's content-derived
/// path (emit_on_demand_key closure_digest); a different closure MUST land in a
/// different workspace dir — file set is assumed a pure function of that digest
/// (benign-by-identity on partial writes before `.native_ready`). `.native_ready`
/// is written only after a successful run (not after build alone): the P3 kernel's
/// warm boundary is build+run proof, so a transient run failure must not skip
/// rebuild on retry. Registered in 04_method.dag as
/// emit_host_run_transport_cached; dissolve-on: witness_realization_kernel emits
/// this builtin from v2 self-hosted transport rows (same dissolution as
/// emit_host_run_transport seed handler).
fn eval_emit_host_run_transport_cached_builtin(
    workspace_dir_arg: Option<&Value>,
    files_arg: Option<&Value>,
    build_arg: Option<&Value>,
    run_arg: Option<&Value>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    ctx.effect_dispatch_count
        .set(ctx.effect_dispatch_count.get().wrapping_add(1));
    if ctx.execution_mode.is_hermetic() {
        return Err(InterpError::TypeError {
            msg: "hermetic mode: emit_host_run_transport_cached refuses host process execution \
                  (no mock arm; run wet or record a fixture)"
                .to_string(),
        });
    }

    let workspace_dir = free_monoid_to_string(workspace_dir_arg.ok_or_else(|| {
        InterpError::TypeError {
            msg:
                "emit_host_run_transport_cached requires (workspace_dir, files, build, run) arguments"
                    .to_string(),
        }
    })?)
    .ok_or_else(|| InterpError::TypeError {
        msg: "emit_host_run_transport_cached: workspace_dir must be String".to_string(),
    })?;

    let files_val = files_arg.ok_or_else(|| InterpError::TypeError {
        msg: "emit_host_run_transport_cached requires (workspace_dir, files, build, run) arguments"
            .to_string(),
    })?;
    let files = free_monoid_to_vec(files_val).ok_or_else(|| InterpError::TypeError {
        msg: "emit_host_run_transport_cached: files must be a List of {path, text} records"
            .to_string(),
    })?;
    let mut workspace_files: Vec<(String, String)> = Vec::with_capacity(files.len());
    for f in &files {
        match f {
            Value::Record { fields, .. } => {
                let path = ctx
                    .field(fields, "path")
                    .and_then(free_monoid_to_string)
                    .ok_or_else(|| InterpError::TypeError {
                        msg: "emit_host_run_transport_cached: workspace file missing String path"
                            .to_string(),
                    })?;
                let text = ctx
                    .field(fields, "text")
                    .and_then(free_monoid_to_string)
                    .ok_or_else(|| InterpError::TypeError {
                        msg: "emit_host_run_transport_cached: workspace file missing String text"
                            .to_string(),
                    })?;
                workspace_files.push((path, text));
            }
            other => {
                return Err(InterpError::TypeError {
                    msg: format!(
                        "emit_host_run_transport_cached: workspace entry must be a record, got {}",
                        other.type_label()
                    ),
                })
            }
        }
    }

    let build_val = build_arg.ok_or_else(|| InterpError::TypeError {
        msg: "emit_host_run_transport_cached requires (workspace_dir, files, build, run) arguments"
            .to_string(),
    })?;
    let build_lists = free_monoid_to_vec(build_val).ok_or_else(|| InterpError::TypeError {
        msg: "emit_host_run_transport_cached: build must be a List of argv Lists".to_string(),
    })?;
    let mut build_argvs: Vec<Vec<String>> = Vec::with_capacity(build_lists.len());
    for argv_val in &build_lists {
        let argv_items = free_monoid_to_vec(argv_val).ok_or_else(|| InterpError::TypeError {
            msg: "emit_host_run_transport_cached: build argv must be a List<String>".to_string(),
        })?;
        let mut argv: Vec<String> = Vec::with_capacity(argv_items.len());
        for item in &argv_items {
            let s = free_monoid_to_string(item).ok_or_else(|| InterpError::TypeError {
                msg: format!(
                    "emit_host_run_transport_cached: build argv element must be String, got {}",
                    item.type_label()
                ),
            })?;
            argv.push(s);
        }
        if argv.is_empty() {
            return Err(InterpError::TypeError {
                msg: "emit_host_run_transport_cached: empty build argv".to_string(),
            });
        }
        build_argvs.push(argv);
    }

    let run_arg_items =
        run_arg
            .and_then(free_monoid_to_vec)
            .ok_or_else(|| InterpError::TypeError {
                msg: "emit_host_run_transport_cached: run must be a List<String>".to_string(),
            })?;
    let mut run_argv: Vec<String> = Vec::with_capacity(run_arg_items.len());
    for item in &run_arg_items {
        let s = free_monoid_to_string(item).ok_or_else(|| InterpError::TypeError {
            msg: format!(
                "emit_host_run_transport_cached: run argv element must be String, got {}",
                item.type_label()
            ),
        })?;
        run_argv.push(s);
    }
    if run_argv.is_empty() {
        return Err(InterpError::TypeError {
            msg: "emit_host_run_transport_cached: empty run argv".to_string(),
        });
    }

    let workspace = std::path::PathBuf::from(&workspace_dir);
    std::fs::create_dir_all(&workspace).map_err(|e| InterpError::TypeError {
        msg: format!("emit_host_run_transport_cached: workspace create failed: {e}"),
    })?;

    emit_host_run_transport_cached_in_workspace(
        &workspace,
        &workspace_files,
        &build_argvs,
        &run_argv,
        ctx,
    )
}

fn emit_host_run_transport_cached_in_workspace(
    workspace: &std::path::Path,
    files: &[(String, String)],
    build_argvs: &[Vec<String>],
    run_argv: &[String],
    ctx: &InterpContext,
) -> InterpResult<Value> {
    use std::path::Component;

    let ready_marker = workspace.join(".native_ready");
    let compile_skipped = ready_marker.exists();

    let transport_result = |phase: &str,
                            success: bool,
                            exit_code: i64,
                            stdout: &[u8],
                            stderr: &[u8],
                            build_log: Vec<Value>,
                            compile_skipped: bool|
     -> Value {
        Value::Record {
            type_name: ctx.sym("EmitHostTransportResult"),
            fields: Rc::new(vec![
                (ctx.sym("phase"), Value::Str(phase.to_string())),
                (ctx.sym("success"), Value::Bool(success)),
                (ctx.sym("exit_code"), Value::Int(exit_code)),
                (ctx.sym("compile_skipped"), Value::Bool(compile_skipped)),
                (
                    ctx.sym("stdout_octets"),
                    list_value(
                        stdout
                            .iter()
                            .map(|b| Value::Int(*b as i64))
                            .collect::<Vec<Value>>(),
                    ),
                ),
                (
                    ctx.sym("stderr_octets"),
                    list_value(
                        stderr
                            .iter()
                            .map(|b| Value::Int(*b as i64))
                            .collect::<Vec<Value>>(),
                    ),
                ),
                (ctx.sym("build_log"), list_value(build_log)),
            ]),
        }
    };

    let target_dir = workspace.join("target");
    let run_command = |argv: &[String]| -> InterpResult<std::process::Output> {
        std::process::Command::new(&argv[0])
            .args(&argv[1..])
            .current_dir(workspace)
            .env("CARGO_TARGET_DIR", &target_dir)
            .output()
            .map_err(|e| InterpError::TypeError {
                msg: format!(
                    "emit_host_run_transport_cached: spawn {:?} failed: {e}",
                    argv[0]
                ),
            })
    };

    if !compile_skipped {
        for (rel, text) in files {
            let p = std::path::Path::new(rel);
            if p.is_absolute() || p.components().any(|c| matches!(c, Component::ParentDir)) {
                return Err(InterpError::TypeError {
                    msg: format!(
                        "emit_host_run_transport_cached: workspace path escapes workspace: {rel}"
                    ),
                });
            }
            let full = workspace.join(p);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).map_err(|e| InterpError::TypeError {
                    msg: format!(
                        "emit_host_run_transport_cached: mkdir {} failed: {e}",
                        parent.display()
                    ),
                })?;
            }
            std::fs::write(&full, text).map_err(|e| InterpError::TypeError {
                msg: format!(
                    "emit_host_run_transport_cached: write {} failed: {e}",
                    full.display()
                ),
            })?;
        }

        let mut build_log: Vec<Value> = Vec::new();
        for argv in build_argvs {
            let out = run_command(argv)?;
            let code = out.status.code().map(i64::from).unwrap_or(-1);
            build_log.push(Value::Str(format!("{} -> exit {code}", argv.join(" "))));
            if !out.status.success() {
                build_log.push(Value::Str(String::from_utf8_lossy(&out.stderr).to_string()));
                return Ok(transport_result(
                    "build",
                    false,
                    code,
                    &out.stdout,
                    &out.stderr,
                    build_log,
                    false,
                ));
            }
        }

        let out = run_command(run_argv)?;
        let code = out.status.code().map(i64::from).unwrap_or(-1);
        build_log.push(Value::Str(format!("{} -> exit {code}", run_argv.join(" "))));
        if out.status.success() {
            std::fs::write(&ready_marker, b"1").map_err(|e| InterpError::TypeError {
                msg: format!("emit_host_run_transport_cached: ready marker write failed: {e}"),
            })?;
        }
        return Ok(transport_result(
            "run",
            out.status.success(),
            code,
            &out.stdout,
            &out.stderr,
            build_log,
            false,
        ));
    }

    let out = run_command(run_argv)?;
    let code = out.status.code().map(i64::from).unwrap_or(-1);
    let mut build_log: Vec<Value> = Vec::new();
    build_log.push(Value::Str(format!("{} -> exit {code}", run_argv.join(" "))));
    Ok(transport_result(
        "run_cached",
        out.status.success(),
        code,
        &out.stdout,
        &out.stderr,
        build_log,
        true,
    ))
}

fn emit_host_run_transport_in_workspace(
    workspace: &std::path::Path,
    files: &[(String, String)],
    build_argvs: &[Vec<String>],
    run_argv: &[String],
    ctx: &InterpContext,
) -> InterpResult<Value> {
    use std::path::Component;

    for (rel, text) in files {
        let p = std::path::Path::new(rel);
        if p.is_absolute() || p.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err(InterpError::TypeError {
                msg: format!("emit_host_run_transport: workspace path escapes workspace: {rel}"),
            });
        }
        let full = workspace.join(p);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).map_err(|e| InterpError::TypeError {
                msg: format!(
                    "emit_host_run_transport: mkdir {} failed: {e}",
                    parent.display()
                ),
            })?;
        }
        std::fs::write(&full, text).map_err(|e| InterpError::TypeError {
            msg: format!(
                "emit_host_run_transport: write {} failed: {e}",
                full.display()
            ),
        })?;
    }

    let target_dir = workspace.join("target");
    let run_command = |argv: &[String]| -> InterpResult<std::process::Output> {
        std::process::Command::new(&argv[0])
            .args(&argv[1..])
            .current_dir(workspace)
            .env("CARGO_TARGET_DIR", &target_dir)
            .output()
            .map_err(|e| InterpError::TypeError {
                msg: format!("emit_host_run_transport: spawn {:?} failed: {e}", argv[0]),
            })
    };

    let transport_result = |phase: &str,
                            success: bool,
                            exit_code: i64,
                            stdout: &[u8],
                            stderr: &[u8],
                            build_log: Vec<Value>,
                            compile_skipped: bool|
     -> Value {
        Value::Record {
            type_name: ctx.sym("EmitHostTransportResult"),
            fields: Rc::new(vec![
                (ctx.sym("phase"), Value::Str(phase.to_string())),
                (ctx.sym("success"), Value::Bool(success)),
                (ctx.sym("exit_code"), Value::Int(exit_code)),
                (ctx.sym("compile_skipped"), Value::Bool(compile_skipped)),
                (
                    ctx.sym("stdout_octets"),
                    list_value(
                        stdout
                            .iter()
                            .map(|b| Value::Int(*b as i64))
                            .collect::<Vec<Value>>(),
                    ),
                ),
                (
                    ctx.sym("stderr_octets"),
                    list_value(
                        stderr
                            .iter()
                            .map(|b| Value::Int(*b as i64))
                            .collect::<Vec<Value>>(),
                    ),
                ),
                (ctx.sym("build_log"), list_value(build_log)),
            ]),
        }
    };

    let mut build_log: Vec<Value> = Vec::new();
    for argv in build_argvs {
        let out = run_command(argv)?;
        let code = out.status.code().map(i64::from).unwrap_or(-1);
        build_log.push(Value::Str(format!("{} -> exit {code}", argv.join(" "))));
        if !out.status.success() {
            build_log.push(Value::Str(String::from_utf8_lossy(&out.stderr).to_string()));
            return Ok(transport_result(
                "build",
                false,
                code,
                &out.stdout,
                &out.stderr,
                build_log,
                false,
            ));
        }
    }

    let out = run_command(run_argv)?;
    let code = out.status.code().map(i64::from).unwrap_or(-1);
    build_log.push(Value::Str(format!("{} -> exit {code}", run_argv.join(" "))));
    Ok(transport_result(
        "run",
        out.status.success(),
        code,
        &out.stdout,
        &out.stderr,
        build_log,
        false,
    ))
}

fn eval_builtin(
    name: &str,
    args: &[(Option<String>, Value)],
    ctx: &InterpContext,
) -> InterpResult<Option<Value>> {
    if !residual_hunt_forensics_enabled() {
        return eval_builtin_inner(name, args, ctx);
    }
    let started = std::time::Instant::now();
    let result = eval_builtin_inner(name, args, ctx);
    if matches!(result, Ok(Some(_))) {
        record_builtin_time_inclusive(name, false, started.elapsed().as_nanos() as u64);
    }
    result
}

fn eval_builtin_inner(
    name: &str,
    args: &[(Option<String>, Value)],
    ctx: &InterpContext,
) -> InterpResult<Option<Value>> {
    let positional: Vec<&Value> = args.iter().map(|(_, v)| v).collect();

    match name {
        "to_string" => {
            let v = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "to_string requires 1 argument".to_string(),
            })?;
            Ok(Some(Value::Str(format!("{}", v))))
        }

        "utf8_decode_bytes" => {
            let bytes = expect_byte_vec(positional.first().copied(), "utf8_decode_bytes")?;
            let text =
                v1_rt::utf8_decode_bytes(&bytes).map_err(|msg| InterpError::TypeError { msg })?;
            Ok(Some(Value::Str(text)))
        }

        "bytes_octets" => {
            let bytes = expect_byte_vec(positional.first().copied(), "bytes_octets")?;
            let items: Vec<Value> = bytes.iter().map(|b| Value::Int(*b as i64)).collect();
            Ok(Some(list_value(items)))
        }

        "octets_bytes" => {
            let arg = positional
                .first()
                .copied()
                .ok_or_else(|| InterpError::TypeError {
                    msg: "octets_bytes requires a List<UInt8> argument".to_string(),
                })?;
            let items = free_monoid_to_vec(arg).ok_or_else(|| InterpError::TypeError {
                msg: "octets_bytes expects a List<UInt8>".to_string(),
            })?;
            let mut out: Vec<Value> = Vec::with_capacity(items.len());
            for item in &items {
                match item {
                    Value::Int(n) if (0..=255).contains(n) => out.push(Value::Int(*n)),
                    other => {
                        return Err(InterpError::TypeError {
                            msg: format!(
                                "octets_bytes expects octets 0..255, got element {}",
                                other.type_label()
                            ),
                        })
                    }
                }
            }
            Ok(Some(list_value(out)))
        }

        "utf8_encode_bytes" => {
            let s = expect_str(positional.first().copied(), "utf8_encode_bytes")?;
            let items: Vec<Value> = s.as_bytes().iter().map(|b| Value::Int(*b as i64)).collect();
            Ok(Some(list_value(items)))
        }

        "discriminant" => match positional.first() {
            Some(Value::Variant { variant_name, .. }) => {
                Ok(Some(Value::Str(resolve_sym(*variant_name))))
            }
            Some(Value::Record { type_name, .. }) => Ok(Some(Value::Str(resolve_sym(*type_name)))),
            _ => Ok(None),
        },

        "chars_to_string" => {
            let cps = match positional.first().copied() {
                Some(v) => free_monoid_to_vec(v).ok_or_else(|| InterpError::TypeError {
                    msg: "chars_to_string expects a list of code points".to_string(),
                })?,
                None => {
                    return Err(InterpError::TypeError {
                        msg: "chars_to_string requires a code-point list".to_string(),
                    })
                }
            };
            let len = cps.len() as i64;
            let start = expect_int(positional.get(1).copied(), "chars_to_string start")?
                .max(0)
                .min(len);
            let end = expect_int(positional.get(2).copied(), "chars_to_string end")?
                .max(0)
                .min(len)
                .max(start);
            let s: String = cps[start as usize..end as usize]
                .iter()
                .filter_map(|v| match v {
                    Value::Int(cp) => char::from_u32(*cp as u32),
                    _ => None,
                })
                .collect();
            Ok(Some(Value::Str(s)))
        }

        "get" => match positional.as_slice() {
            [list_val, idx_val] if free_monoid_to_vec(list_val).is_some() => {
                let items = expect_list(list_val, "get")?;
                let idx = expect_int(Some(idx_val), "get")?;
                Ok(Some(list_get_at_or_null(&items, idx)))
            }
            _ => Ok(None),
        },

        "parse_int" => {
            let s = expect_str(positional.first().copied(), "parse_int")?;
            match s.parse::<i64>() {
                Ok(n) => Ok(Some(Value::Int(n))),
                Err(_) => Ok(Some(Value::Null)),
            }
        }

        "record_source_chars_index_lookup" => Ok(Some(Value::Unit)),

        // Scaffold arm — dissolution trigger lives on `v1_rt::trace_mark`'s doc comment
        // (realization_measurement_loop Phase 0, docs/plans/realization-measurement-loop.md):
        // delete this arm with the rest of the trace_mark deletion set named there.
        "trace_mark" => {
            if let [Value::Str(s)] = positional.as_slice() {
                v1_rt::trace_mark(s.clone());
            }
            Ok(Some(Value::Unit))
        }

        "concat" => {
            if positional.len() >= 2 && positional.iter().all(|v| matches!(v, Value::Str(_))) {
                let mut result = String::new();
                for v in &positional {
                    if let Value::Str(s) = v {
                        result.push_str(s);
                    }
                }
                return Ok(Some(Value::Str(result)));
            }
            let record_push = |copied: usize| {
                let mut counters = ctx.mutation_counters.borrow_mut();
                counters.list_push_calls += 1;
                counters.list_push_items_copied += copied as u64;
            };
            match positional.as_slice() {
                [a, b] => match (a, b) {
                    (l, Value::Str(s)) => match free_monoid_to_vec(l) {
                        Some(mut result) => {
                            record_push(result.len());
                            result.push(Value::Str(s.clone()));
                            Ok(Some(list_value((result))))
                        }
                        None => Ok(None),
                    },
                    (Value::Str(s), r) => match free_monoid_to_vec(r) {
                        Some(result) => {
                            record_push(result.len());
                            let mut out = vec![Value::Str(s.clone())];
                            out.extend(result);
                            Ok(Some(list_value((out))))
                        }
                        None => Ok(None),
                    },
                    _ => match (free_monoid_to_vec(a), free_monoid_to_vec(b)) {
                        (Some(mut a_items), Some(b_items)) => {
                            let mut counters = ctx.mutation_counters.borrow_mut();
                            counters.list_concat_calls += 1;
                            counters.list_concat_items_copied +=
                                (a_items.len() + b_items.len()) as u64;
                            drop(counters);
                            a_items.extend(b_items);
                            Ok(Some(list_value((a_items))))
                        }
                        _ => Ok(None),
                    },
                },
                _ => Ok(None),
            }
        }

        "count" => match positional.first() {
            Some(v) => match free_monoid_to_vec(v) {
                Some(items) => Ok(Some(Value::Int(items.len() as i64))),
                None => Ok(None),
            },
            None => Ok(None),
        },

        "reverse" => match positional.first() {
            Some(Value::Str(_)) => Ok(None),
            Some(v) => match free_monoid_to_vec(v) {
                Some(items) => {
                    let mut r = items;
                    r.reverse();
                    Ok(Some(list_value((r))))
                }
                None => Ok(None),
            },
            None => Ok(None),
        },

        "string_length" => {
            let s = expect_str(positional.first().copied(), "string_length")?;
            Ok(Some(Value::Int(s.chars().count() as i64)))
        }

        "substring" => {
            let s = expect_str(positional.first().copied(), "substring")?;
            let start = expect_int(positional.get(1).copied(), "substring start")?;
            let end = expect_int(positional.get(2).copied(), "substring end")?;
            Ok(Some(Value::Str(v1_rt::substring(&s, start, end))))
        }

        "char_at" => {
            let s = expect_str(positional.first().copied(), "char_at")?;
            let pos = expect_int(positional.get(1).copied(), "char_at pos")?;
            Ok(Some(Value::Str(v1_rt::char_at(&s, pos))))
        }

        "string_contains" => {
            let s = expect_str(positional.first().copied(), "contains")?;
            let sub = expect_str(positional.get(1).copied(), "contains sub")?;
            Ok(Some(Value::Bool(s.contains(&sub))))
        }

        "starts_with" => {
            let s = expect_str(positional.first().copied(), "starts_with")?;
            let prefix = expect_str(positional.get(1).copied(), "starts_with prefix")?;
            Ok(Some(Value::Bool(s.starts_with(&prefix))))
        }

        "length" => match positional.first() {
            Some(Value::Str(s)) => Ok(Some(Value::Int(s.chars().count() as i64))),
            Some(v) => match native_len(v) {
                Some(n) => Ok(Some(Value::Int(n))),
                None => match free_monoid_to_vec(v) {
                    Some(items) => Ok(Some(Value::Int(items.len() as i64))),
                    None => Ok(None),
                },
            },
            None => Ok(None),
        },

        "contains" => match positional.as_slice() {
            [Value::Str(s), Value::Str(sub), ..] => Ok(Some(Value::Bool(s.contains(sub)))),
            [xs, target, ..] => match free_monoid_to_vec(xs) {
                Some(items) => Ok(Some(Value::Bool(items.iter().any(|item| item == *target)))),
                None => Ok(None),
            },
            _ => Ok(None),
        },

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

        "is_xid_start" => {
            let cp = expect_int(positional.first().copied(), "is_xid_start")?;
            Ok(Some(Value::Bool(v1_rt::is_xid_start(cp))))
        }

        "is_xid_continue" => {
            let cp = expect_int(positional.first().copied(), "is_xid_continue")?;
            Ok(Some(Value::Bool(v1_rt::is_xid_continue(cp))))
        }

        "is_emoji_ident" => {
            let cp = expect_int(positional.first().copied(), "is_emoji_ident")?;
            Ok(Some(Value::Bool(v1_rt::is_emoji_ident(cp))))
        }

        "list_push" | "append" => match positional.as_slice() {
            [list_val, item] if matches!(list_val, Value::Str(_)) => Ok(None),
            [list_val, item] => match value_to_list_carrier(list_val) {
                Some((items, copied)) => {
                    let mut counters = ctx.mutation_counters.borrow_mut();
                    counters.list_push_calls += 1;
                    counters.list_push_items_copied += copied;
                    drop(counters);
                    let mut result = (*items).clone();
                    result.push_back((*item).clone());
                    Ok(Some(list_value(result)))
                }
                None => Ok(None),
            },
            _ => Ok(None),
        },

        "list_concat" => match positional.as_slice() {
            [a, b] if matches!(a, Value::Str(_)) || matches!(b, Value::Str(_)) => Ok(None),
            [a, b] => match (value_to_list_carrier(a), value_to_list_carrier(b)) {
                (Some((a_items, a_copied)), Some((b_items, b_copied))) => {
                    let mut counters = ctx.mutation_counters.borrow_mut();
                    counters.list_concat_calls += 1;
                    counters.list_concat_items_copied += a_copied + b_copied;
                    drop(counters);
                    let mut result = (*a_items).clone();
                    result.append((*b_items).clone());
                    Ok(Some(list_value(result)))
                }
                _ => Ok(None),
            },
            _ => Ok(None),
        },

        "empty_map" => Ok(Some(map_value(HamtMap::new()))),

        "empty_set" => Ok(Some(Value::Set(Rc::new(OrdSet::new())))),

        "set_insert" => match positional.as_slice() {
            [Value::Set(s), Value::Str(k)] => {
                let mut counters = ctx.mutation_counters.borrow_mut();
                counters.set_insert_calls += 1;
                counters.set_insert_items_copied += s.len() as u64;
                drop(counters);
                let mut result = s.as_ref().clone();
                result.insert(k.clone());
                Ok(Some(Value::Set(Rc::new(result))))
            }
            _ => Ok(None),
        },

        "set_union" => match positional.as_slice() {
            [Value::Set(a), Value::Set(b)] => {
                let mut counters = ctx.mutation_counters.borrow_mut();
                counters.set_union_calls += 1;
                counters.set_union_items_copied += (a.len() + b.len()) as u64;
                drop(counters);
                let mut result = a.as_ref().clone();
                result.extend(b.iter().cloned());
                Ok(Some(Value::Set(Rc::new(result))))
            }
            _ => Ok(None),
        },

        "set_contains" => match positional.as_slice() {
            [Value::Set(s), Value::Str(k)] => Ok(Some(Value::Bool(s.contains(k.as_str())))),
            _ => Ok(None),
        },

        "map_insert" => match positional.as_slice() {
            [Value::Map(m), k, v] => match CanonKey::new((*k).clone()) {
                Some(ck) => {
                    let mut counters = ctx.mutation_counters.borrow_mut();
                    counters.map_insert_calls += 1;
                    drop(counters);
                    Ok(Some(map_value(m.update(ck, (*v).clone()))))
                }
                None => Err(InterpError::TypeError {
                    msg: format!(
                        "map_insert key has no decidable identity (closure/fn/NaN): {}",
                        k.type_label()
                    ),
                }),
            },
            _ => Ok(None),
        },

        "lookup" => match positional.as_slice() {
            [map, key] => Ok(Some(raw_map_lookup(map, key, &Env::empty(), ctx)?)),
            _ => Ok(None),
        },

        "map_keys" => match positional.first() {
            Some(Value::Map(m)) => {
                let keys: Vec<Value> = m.keys().map(|k| k.key.clone()).collect();
                Ok(Some(list_value((keys))))
            }
            _ => Ok(None),
        },

        "map_values" => match positional.first() {
            Some(Value::Map(m)) => {
                let vals: Vec<Value> = m.values().cloned().collect();
                Ok(Some(list_value((vals))))
            }
            _ => Ok(None),
        },

        "map_contains_key" | "map_has" => match positional.as_slice() {
            [Value::Map(m), k] => match CanonKey::new((*k).clone()) {
                Some(ck) => Ok(Some(Value::Bool(m.contains_key(&ck)))),
                None => Ok(Some(Value::Bool(false))),
            },
            _ => Ok(None),
        },

        "map_is_empty" => match positional.as_slice() {
            [Value::Map(m)] => Ok(Some(Value::Bool(m.is_empty()))),
            _ => Ok(None),
        },

        "rc_ptr_eq" | "rc_vec_ptr_eq" => match positional.as_slice() {
            [a, b] => Ok(Some(Value::Bool(a == b))),
            _ => Ok(None),
        },

        "map_merge" => match positional.as_slice() {
            [Value::Map(base), Value::Map(overlay)] => {
                let mut counters = ctx.mutation_counters.borrow_mut();
                counters.map_merge_calls += 1;
                drop(counters);
                Ok(Some(map_value((**overlay).clone().union((**base).clone()))))
            }
            _ => Ok(None),
        },

        "str_eq" => match positional.as_slice() {
            [Value::Str(a), Value::Str(b)] => Ok(Some(Value::Bool(a == b))),
            _ => Ok(None),
        },

        "atom_identity_hash" => match positional.as_slice() {
            [Value::Str(s)] => Ok(Some(Value::Str(v1_rt::atom_identity_hash(s.clone())))),
            _ => Err(InterpError::TypeError {
                msg: "atom_identity_hash requires exactly one string argument".to_string(),
            }),
        },

        // ObservePeakResidentAtSubject realization seam (witness-realization plan P1):
        // process peak resident set (VmHWM) in bytes. Fail-closed when the host
        // cannot report it — a fabricated 0 would be a Measured lie (DESIGN §5).
        "observed_peak_resident_bytes" => match positional.as_slice() {
            [] => {
                let bytes = std::fs::read_to_string("/proc/self/status")
                    .ok()
                    .and_then(|status| {
                        status
                            .lines()
                            .find(|l| l.starts_with("VmHWM"))
                            .and_then(|line| line.split_whitespace().nth(1))
                            .and_then(|kb| kb.parse::<i64>().ok())
                    })
                    .map(|kb| kb.saturating_mul(1024));
                match bytes {
                    Some(b) => Ok(Some(Value::Int(b))),
                    None => Err(InterpError::TypeError {
                        msg: "observed_peak_resident_bytes: VmHWM unavailable on this host (refusing to fabricate a Measured space fact)"
                            .to_string(),
                    }),
                }
            }
            _ => Err(InterpError::TypeError {
                msg: "observed_peak_resident_bytes takes no arguments".to_string(),
            }),
        },

        "hash_combine" => match positional.as_slice() {
            [Value::Str(a), Value::Str(b)] if positional.len() == 2 => {
                if !v1_rt::is_hash_digest(a) || !v1_rt::is_hash_digest(b) {
                    return Err(InterpError::TypeError {
                        msg: "hash_combine requires exactly two Hash arguments".to_string(),
                    });
                }
                Ok(Some(Value::Str(v1_rt::hash_combine(a.clone(), b.clone()))))
            }
            _ => Err(InterpError::TypeError {
                msg: "hash_combine requires exactly two Hash arguments".to_string(),
            }),
        },

        "filesystem_read" => {
            let path = expect_str(positional.first().copied(), "filesystem_read")?;
            Ok(Some(eval_filesystem_read_builtin(path, ctx)?))
        }

        "emit_host_run_transport" => Ok(Some(eval_emit_host_run_transport_builtin(
            positional.first().copied(),
            positional.get(1).copied(),
            positional.get(2).copied(),
            ctx,
        )?)),

        "emit_host_run_transport_cached" => Ok(Some(eval_emit_host_run_transport_cached_builtin(
            positional.first().copied(),
            positional.get(1).copied(),
            positional.get(2).copied(),
            positional.get(3).copied(),
            ctx,
        )?)),

        "contiguous_loop_elementwise_kernel" => {
            let op_codes = expect_int_list_flex(positional.first().copied(), name)?;
            let a = expect_int_list_flex(positional.get(1).copied(), name)?;
            let b = expect_int_list_flex(positional.get(2).copied(), name)?;
            let c = expect_int_list_flex(positional.get(3).copied(), name)?;
            if a.len() != b.len() || b.len() != c.len() {
                return Err(InterpError::TypeError {
                    msg: format!(
                        "{name} requires equal-length List<Int> buffer arguments, got lengths {}, {}, {}",
                        a.len(),
                        b.len(),
                        c.len()
                    ),
                });
            }
            let out = v1_rt::contiguous_loop_elementwise_kernel(&op_codes, &a, &b, &c);
            Ok(Some(list_value(
                out.into_iter().map(Value::Int).collect::<Vec<_>>(),
            )))
        }

        "contiguous_loop_elementwise_float_kernel" => {
            let op_codes = expect_int_list_flex(positional.first().copied(), name)?;
            let fma_policy = expect_fma_contraction_policy_wire(positional.get(1).copied(), name)?;
            let a = expect_float_list_flex(positional.get(2).copied(), name)?;
            let b = expect_float_list_flex(positional.get(3).copied(), name)?;
            let c = expect_float_list_flex(positional.get(4).copied(), name)?;
            if a.len() != b.len() || b.len() != c.len() {
                return Err(InterpError::TypeError {
                    msg: format!(
                        "{name} requires equal-length List<Float> buffer arguments, got lengths {}, {}, {}",
                        a.len(),
                        b.len(),
                        c.len()
                    ),
                });
            }
            let out =
                v1_rt::contiguous_loop_elementwise_float_kernel(&op_codes, fma_policy, &a, &b, &c);
            Ok(Some(list_value(
                out.into_iter().map(Value::Float).collect::<Vec<_>>(),
            )))
        }

        "layer_import_facts" => {
            let std_roots = expect_str_list(positional.first().copied(), "layer_import_facts")?;
            let extdeps_roots = expect_str_list(positional.get(1).copied(), "layer_import_facts")?;
            let facts = crate::cli_run::layer_import_facts(&std_roots, &extdeps_roots);
            let mut items: Vec<Value> = Vec::new();
            for f in facts {
                let layer = Value::Variant {
                    type_name: ctx.sym("LayerPrefix"),
                    variant_name: ctx.sym(f.layer),
                    fields: Rc::new(vec![]),
                };
                items.push(Value::Record {
                    type_name: ctx.sym("LayerImportFact"),
                    fields: Rc::new(sorted_fields(vec![
                        (ctx.sym("import_module"), Value::Str(f.import_module)),
                        (ctx.sym("layer"), layer),
                        (ctx.sym("path"), Value::Str(f.path)),
                    ])),
                });
            }
            Ok(Some(list_value(items)))
        }

        "import_resolution_facts" => {
            let pool_roots =
                expect_str_list(positional.first().copied(), "import_resolution_facts")?;
            let importer_roots =
                expect_str_list(positional.get(1).copied(), "import_resolution_facts")?;
            let exclude_substrings =
                expect_str_list(positional.get(2).copied(), "import_resolution_facts")?;
            let facts = crate::cli_run::import_resolution_facts(
                &pool_roots,
                &importer_roots,
                &exclude_substrings,
            );
            let mut items: Vec<Value> = Vec::new();
            for f in facts {
                items.push(Value::Record {
                    type_name: ctx.sym("ImportResolutionFact"),
                    fields: Rc::new(sorted_fields(vec![
                        (ctx.sym("import_module"), Value::Str(f.import_module)),
                        (ctx.sym("path"), Value::Str(f.path)),
                        (ctx.sym("target_declared"), Value::Bool(f.target_declared)),
                    ])),
                });
            }
            Ok(Some(list_value(items)))
        }

        "reference_resolution_facts" => {
            let pool_roots =
                expect_str_list(positional.first().copied(), "reference_resolution_facts")?;
            let importer_roots =
                expect_str_list(positional.get(1).copied(), "reference_resolution_facts")?;
            let exclude_substrings =
                expect_str_list(positional.get(2).copied(), "reference_resolution_facts")?;
            // Selection tier: Qualified + UniqueBare only (strict = true). AmbiguousBare is
            // dropped here — same projection `reference_edges_as_import_facts` applies on the
            // host twin's `selection_adjacency` path in `build_module_graph_facts_live_uncached`.
            let facts = crate::cli_run::reference_edges_as_import_facts(
                &crate::cli_run::reference_resolution_facts(
                    &pool_roots,
                    &importer_roots,
                    &exclude_substrings,
                ),
                true,
            );
            let mut items: Vec<Value> = Vec::new();
            for f in facts {
                items.push(Value::Record {
                    type_name: ctx.sym("ImportResolutionFact"),
                    fields: Rc::new(sorted_fields(vec![
                        (ctx.sym("import_module"), Value::Str(f.import_module)),
                        (ctx.sym("path"), Value::Str(f.path)),
                        (ctx.sym("target_declared"), Value::Bool(f.target_declared)),
                    ])),
                });
            }
            Ok(Some(list_value(items)))
        }

        "concept_decl_facts" => {
            let pool_roots = expect_str_list(positional.first().copied(), "concept_decl_facts")?;
            Ok(Some(crate::coproduct_reflection::eval_concept_decl_facts(
                ctx,
                &pool_roots,
            )?))
        }

        "export_signature_facts" => {
            let pool_roots =
                expect_str_list(positional.first().copied(), "export_signature_facts")?;
            Ok(Some(
                crate::coproduct_reflection::eval_export_signature_facts(ctx, &pool_roots)?,
            ))
        }

        "decl_facts" => {
            let pool_roots = expect_str_list(positional.first().copied(), "decl_facts")?;
            Ok(Some(crate::coproduct_reflection::eval_decl_facts(
                ctx,
                &pool_roots,
            )?))
        }

        "module_declaration_facts" => {
            let pool_roots =
                expect_str_list(positional.first().copied(), "module_declaration_facts")?;
            let facts = crate::cli_run::module_declaration_facts(&pool_roots);
            let mut items: Vec<Value> = Vec::new();
            for f in facts {
                items.push(Value::Record {
                    type_name: ctx.sym("ModuleDeclarationFact"),
                    fields: Rc::new(sorted_fields(vec![
                        (ctx.sym("module"), Value::Str(f.module)),
                        (ctx.sym("path"), Value::Str(f.path)),
                    ])),
                });
            }
            Ok(Some(list_value(items)))
        }

        "fact_cardinality_decl_facts" => {
            let facts = crate::cli_run::fact_cardinality_decl_facts();
            let mut items: Vec<Value> = Vec::new();
            for f in facts {
                let tree = match f.tree.as_str() {
                    "dag" => "Dag",
                    "v2" => "V2",
                    other => panic!("fact_cardinality_decl_facts: unknown tree {other:?}"),
                };
                let tree_value = Value::Variant {
                    type_name: ctx.sym("FactCardinalityTree"),
                    variant_name: ctx.sym(tree),
                    fields: Rc::new(vec![]),
                };
                items.push(Value::Record {
                    type_name: ctx.sym("FactCardinalityDeclFact"),
                    fields: Rc::new(sorted_fields(vec![
                        (
                            ctx.sym("rel_path_decl_key"),
                            Value::Str(f.rel_path_decl_key),
                        ),
                        (ctx.sym("tree"), tree_value),
                        (ctx.sym("content_hash"), Value::Str(f.content_hash)),
                    ])),
                });
            }
            Ok(Some(list_value(items)))
        }

        "languages_consumer_census_data_decl_count" => Ok(Some(Value::Int(
            crate::cli_run::languages_consumer_census_data_decl_count(),
        ))),

        "languages_consumer_census_per_language_row_count" => Ok(Some(Value::Int(
            crate::cli_run::languages_consumer_census_per_language_row_count(),
        ))),

        "languages_consumer_census_format_row_count" => Ok(Some(Value::Int(
            crate::cli_run::languages_consumer_census_format_row_count(),
        ))),

        "languages_consumer_census_external_consumer_count" => {
            let decl_name = expect_str(
                positional.first().copied(),
                "languages_consumer_census_external_consumer_count",
            )?;
            Ok(Some(Value::Int(
                crate::cli_run::languages_consumer_census_external_consumer_count(decl_name),
            )))
        }

        "languages_consumer_census_is_composition_only" => {
            let decl_name = expect_str(
                positional.first().copied(),
                "languages_consumer_census_is_composition_only",
            )?;
            Ok(Some(Value::Bool(
                crate::cli_run::languages_consumer_census_is_composition_only(decl_name),
            )))
        }

        "languages_consumer_census_has_external_consumer" => {
            let decl_name = expect_str(
                positional.first().copied(),
                "languages_consumer_census_has_external_consumer",
            )?;
            Ok(Some(Value::Bool(
                crate::cli_run::languages_consumer_census_has_external_consumer(decl_name),
            )))
        }

        "shell_materialize_argv_for_operation" => {
            let path = expect_str(
                positional.first().copied(),
                "shell_materialize_argv_for_operation",
            )?;
            let service = expect_str(
                positional.get(1).copied(),
                "shell_materialize_argv_for_operation",
            )?;
            let operation = expect_str(
                positional.get(2).copied(),
                "shell_materialize_argv_for_operation",
            )?;
            let package = expect_str(
                positional.get(3).copied(),
                "shell_materialize_argv_for_operation",
            )?;
            let bin = expect_str(
                positional.get(4).copied(),
                "shell_materialize_argv_for_operation",
            )?;
            let extra_args = expect_str_list(
                positional.get(5).copied(),
                "shell_materialize_argv_for_operation",
            )?;
            let unit = positional
                .get(6)
                .and_then(|v| match v {
                    Value::Str(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();
            let mut param_bindings = HashMap::new();
            param_bindings.insert("package".to_string(), Value::Str(package));
            param_bindings.insert("bin".to_string(), Value::Str(bin));
            param_bindings.insert(
                "args".to_string(),
                list_value(extra_args.into_iter().map(Value::Str).collect::<Vec<_>>()),
            );
            if !unit.is_empty() {
                param_bindings.insert("unit".to_string(), Value::Str(unit));
            }
            let argv =
                materialize_shell_argv_for_operation(path, service, operation, param_bindings)
                    .map_err(|e| InterpError::TypeError { msg: e })?;
            Ok(Some(list_value(
                argv.into_iter().map(Value::Str).collect::<Vec<_>>(),
            )))
        }

        "extdeps_qualified_name_resolves_in_derived_module_set" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg:
                    "extdeps_qualified_name_resolves_in_derived_module_set requires a QualifiedName"
                        .to_string(),
            })?;
            Ok(Some(Value::Bool(
                crate::cli_run::qualified_name_resolves_in_derived_module_set(module),
            )))
        }

        "transport_script_position_facts_for_path" => {
            let path = expect_str(
                positional.first().copied(),
                "transport_script_position_facts_for_path",
            )?;
            let facts = crate::cli_run::transport_script_position_facts_for_path(path);
            let mut items: Vec<Value> = Vec::new();
            for f in facts {
                let shape = Value::Variant {
                    type_name: ctx.sym("TransportScriptArgShape"),
                    variant_name: ctx.sym(f.shape),
                    fields: Rc::new(vec![]),
                };
                items.push(Value::Record {
                    type_name: ctx.sym("TransportScriptPositionFact"),
                    fields: Rc::new(sorted_fields(vec![
                        (ctx.sym("function"), Value::Str(f.function)),
                        (ctx.sym("path"), Value::Str(f.path)),
                        (ctx.sym("shape"), shape),
                    ])),
                });
            }
            Ok(Some(list_value(items)))
        }

        "extdeps_shape_transport_policy_facts_for_qualified_name" => {
            let qn = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_shape_transport_policy_facts_for_qualified_name requires a QualifiedName"
                    .to_string(),
            })?;
            let module_path = crate::cli_run::free_monoid_symbol_value_to_dotted_string(qn);
            let facts = crate::cli_run::extdeps_shape_transport_policy_module_facts(&module_path);
            let argv_items: Vec<Value> = facts
                .argv_facts
                .iter()
                .map(|f| Value::Record {
                    type_name: ctx.sym("ExtdepsTransportArgvFact"),
                    fields: Rc::new(sorted_fields(vec![
                        (ctx.sym("argv_index"), Value::Int(f.argv_index)),
                        (ctx.sym("argv_token"), Value::Str(f.argv_token.clone())),
                        (ctx.sym("module"), (*qn).clone()),
                        (ctx.sym("operation"), Value::Str(f.operation.clone())),
                        (ctx.sym("service"), Value::Str(f.service.clone())),
                        (
                            ctx.sym("transport_kind"),
                            Value::Variant {
                                type_name: ctx.sym("ExtdepsTransportKind"),
                                variant_name: ctx.sym(f.transport_kind),
                                fields: Rc::new(vec![]),
                            },
                        ),
                    ])),
                })
                .collect();
            let fusion_items: Vec<Value> = facts
                .fusion_facts
                .iter()
                .map(|f| Value::Record {
                    type_name: ctx.sym("ExtdepsTransportFusionFact"),
                    fields: Rc::new(sorted_fields(vec![
                        (ctx.sym("endpoint_key"), Value::Str(f.endpoint_key.clone())),
                        (ctx.sym("module"), (*qn).clone()),
                        (ctx.sym("service_a"), Value::Str(f.service_a.clone())),
                        (ctx.sym("service_b"), Value::Str(f.service_b.clone())),
                    ])),
                })
                .collect();
            let input_items: Vec<Value> = facts
                .input_facts
                .iter()
                .map(|f| Value::Record {
                    type_name: ctx.sym("ExtdepsOperationInputFact"),
                    fields: Rc::new(sorted_fields(vec![
                        (ctx.sym("module"), (*qn).clone()),
                        (ctx.sym("operation"), Value::Str(f.operation.clone())),
                        (ctx.sym("param_name"), Value::Str(f.param_name.clone())),
                        (ctx.sym("service"), Value::Str(f.service.clone())),
                    ])),
                })
                .collect();
            let embedded_items: Vec<Value> = facts
                .embedded_facts
                .iter()
                .map(|f| Value::Record {
                    type_name: ctx.sym("ExtdepsEmbeddedPolicyLiteralFact"),
                    fields: Rc::new(sorted_fields(vec![
                        (ctx.sym("data_name"), Value::Str(f.data_name.clone())),
                        (ctx.sym("field_name"), Value::Str(f.field_name.clone())),
                        (
                            ctx.sym("literal_value"),
                            Value::Str(f.literal_value.clone()),
                        ),
                        (ctx.sym("module"), (*qn).clone()),
                    ])),
                })
                .collect();
            let result = Value::Record {
                type_name: ctx.sym("ExtdepsModuleFacts"),
                fields: Rc::new(sorted_fields(vec![
                    (ctx.sym("argv_facts"), list_value(argv_items)),
                    (ctx.sym("embedded_facts"), list_value(embedded_items)),
                    (ctx.sym("fusion_facts"), list_value(fusion_items)),
                    (
                        ctx.sym("gist_create_declares_filename_input"),
                        Value::Bool(facts.gist_create_declares_filename_input),
                    ),
                    (
                        ctx.sym("gist_create_files_keyed_by_filename"),
                        Value::Bool(facts.gist_create_files_keyed_by_filename),
                    ),
                    (ctx.sym("input_facts"), list_value(input_items)),
                    (
                        ctx.sym("source_nickname_literal_count"),
                        Value::Int(facts.source_nickname_literal_count),
                    ),
                ])),
            };
            Ok(Some(result))
        }

        "extdeps_external_authority_facts_for_qualified_name" => {
            let qn = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_external_authority_facts_for_qualified_name requires a QualifiedName"
                    .to_string(),
            })?;
            let module_path = crate::cli_run::free_monoid_symbol_value_to_dotted_string(qn);
            let facts = crate::cli_run::extdeps_external_authority_module_facts(&module_path);
            let result = Value::Record {
                type_name: ctx.sym("ExtdepsExternalAuthorityModuleFacts"),
                fields: Rc::new(sorted_fields(vec![
                    (ctx.sym("anchor_kind"), Value::Str(facts.anchor_kind)),
                    (
                        ctx.sym("scheme_identity"),
                        Value::Str(facts.scheme_identity),
                    ),
                    (ctx.sym("locator"), Value::Str(facts.locator)),
                ])),
            };
            Ok(Some(result))
        }

        "extdeps_external_authority_live_clean_tree_holds" => Ok(Some(Value::Bool(
            crate::cli_run::extdeps_external_authority_live_clean_tree_holds(),
        ))),
        "extdeps_external_authority_live_roster_module_count" => Ok(Some(Value::Int(
            crate::cli_run::extdeps_external_authority_live_roster_module_count(),
        ))),

        "doc_graph_orphan_count" => Ok(Some(Value::Int(crate::cli_run::doc_graph_orphan_count()))),
        "doc_graph_dangling_link_count" => Ok(Some(Value::Int(
            crate::cli_run::doc_graph_dangling_link_count(),
        ))),
        "doc_graph_doc_count" => Ok(Some(Value::Int(crate::cli_run::doc_graph_doc_count()))),

        "compile_dag_rust_emit_check" => {
            let source = expect_str(positional.first().copied(), name)?;
            let file_path = expect_str(positional.get(1).copied(), name)?;
            let includes = expect_str_list(positional.get(2).copied(), name)?;
            let excludes = expect_str_list(positional.get(3).copied(), name)?;
            Ok(Some(Value::Bool(
                crate::cli_run::compile_dag_rust_emit_check(
                    &source, &file_path, &includes, &excludes,
                ),
            )))
        }

        "witness_layer_roots_compile_clean_check" => Ok(Some(Value::Bool(
            crate::cli_run::witness_layer_roots_compile_clean_check(),
        ))),

        "witness_layer_roots_compile_clean_emit_check" => Ok(Some(Value::Bool(
            crate::cli_run::witness_layer_roots_compile_clean_emit_check(),
        ))),
        "consume_floor_compile_clean_gate_verdict" => Ok(Some(Value::Bool(
            crate::cli_run::consume_floor_compile_clean_gate_verdict(),
        ))),

        "test_migration_debt_module_count" => Ok(Some(Value::Int(
            crate::cli_run::test_migration_debt_module_count(),
        ))),
        "test_migration_debt_total_loc" => Ok(Some(Value::Int(
            crate::cli_run::test_migration_debt_total_loc(),
        ))),
        "test_migration_debt_total_test_fns" => Ok(Some(Value::Int(
            crate::cli_run::test_migration_debt_total_test_fns(),
        ))),
        "test_migration_debt_module_names" => {
            let names = crate::cli_run::test_migration_debt_module_names();
            let items: Vec<Value> = names.into_iter().map(Value::Str).collect();
            Ok(Some(list_value(items)))
        }
        "test_migration_debt_known_covered_module_is_not_debt" => Ok(Some(Value::Bool(
            crate::cli_run::test_migration_debt_known_covered_module_is_not_debt(),
        ))),
        "test_migration_delete_guard_holds" => Ok(Some(Value::Bool(
            crate::cli_run::test_migration_delete_guard_holds(),
        ))),
        "test_migration_delete_guard_uncovered_deletes" => {
            let paths = crate::cli_run::test_migration_delete_guard_uncovered_deletes();
            let items: Vec<Value> = paths.into_iter().map(Value::Str).collect();
            Ok(Some(list_value(items)))
        }

        "inert_carrier_names_live" => {
            let names = crate::cli_run::inert_carrier_names_live();
            let items: Vec<Value> = names.into_iter().map(Value::Str).collect();
            Ok(Some(list_value(items)))
        }
        "inert_carrier_declared_count" => Ok(Some(Value::Int(
            crate::cli_run::inert_carrier_declared_count_live(),
        ))),

        "inert_lens_unreached_module_count" => Ok(Some(Value::Int(
            crate::cli_run::inert_lens_unreached_module_count(),
        ))),
        "inert_lens_top_level_module_count" => Ok(Some(Value::Int(
            crate::cli_run::inert_lens_top_level_module_count(),
        ))),

        "non_fold_residue_count" => Ok(Some(Value::Int(crate::cli_run::non_fold_residue_count()))),
        "non_fold_residue_unrostered_count" => Ok(Some(Value::Int(
            crate::cli_run::non_fold_residue_unrostered_count(),
        ))),
        "non_fold_residue_stale_roster_count" => Ok(Some(Value::Int(
            crate::cli_run::non_fold_residue_stale_roster_count(),
        ))),
        "non_fold_residue_coproduct_universe_count" => Ok(Some(Value::Int(
            crate::cli_run::non_fold_residue_coproduct_universe_count(),
        ))),

        "complexity_linearity_syntactic_finding_count" => Ok(Some(Value::Int(
            crate::cli_run::complexity_linearity_syntactic_finding_count(),
        ))),
        "complexity_linearity_wildcard_facts" => {
            let facts = crate::cli_run::complexity_linearity_wildcard_facts();
            let mut items: Vec<Value> = Vec::new();
            for f in facts {
                items.push(Value::Record {
                    type_name: ctx.sym("ComplexityLinearityWildcardFact"),
                    fields: Rc::new(sorted_fields(vec![
                        (
                            ctx.sym("closed_coproduct_wildcard"),
                            Value::Bool(f.closed_coproduct_wildcard),
                        ),
                        (ctx.sym("fn_name"), Value::Str(f.fn_name.clone())),
                        (ctx.sym("rostered"), Value::Bool(f.rostered)),
                        (ctx.sym("site"), Value::Str(f.site.clone())),
                    ])),
                });
            }
            Ok(Some(list_value(items)))
        }

        "complexity_linearity_syntactic_site_fired" => {
            let site = expect_str(
                positional.first().copied(),
                "complexity_linearity_syntactic_site_fired",
            )?;
            Ok(Some(Value::Bool(
                crate::cli_run::complexity_linearity_syntactic_site_fired(&site),
            )))
        }
        "census_corpus_roots_follow_layer_authority" => Ok(Some(Value::Bool(
            crate::cli_run::census_corpus_roots_follow_layer_authority(),
        ))),

        _ => Ok(None),
    }
}

fn apply_closure(
    closure: &Value,
    args: &[Value],
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    // Same bounded-execution guard as call_function — closures are the other
    // call grain, and a recursion cycle through them never passes call_function.
    let depth = CALL_DEPTH.with(|d| {
        let v = d.get() + 1;
        d.set(v);
        v
    });
    if depth > CALL_DEPTH_LIMIT {
        CALL_DEPTH.with(|d| d.set(d.get() - 1));
        return Err(InterpError::TypeError {
            msg: format!(
                "call depth exceeded {} in closure application — unbounded recursion \
                 (a bare-name resolution cycle, or a genuinely divergent chain); \
                 refused, never a host stack overflow",
                CALL_DEPTH_LIMIT
            ),
        });
    }
    let result = stacker::maybe_grow(256 * 1024, 8 * 1024 * 1024, || {
        apply_closure_inner(closure, args, env, ctx)
    });
    CALL_DEPTH.with(|d| d.set(d.get() - 1));
    result
}

fn apply_closure_inner(
    closure: &Value,
    args: &[Value],
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    match closure {
        Value::Closure {
            params,
            body,
            env: closure_env,
        } => {
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
        Value::Fn { node } => {
            let node = node.clone();
            let named: Vec<(Option<String>, Value)> =
                args.iter().map(|v| (None, v.clone())).collect();
            call_function(ctx, &node, &named, env)
        }
        _ => Err(InterpError::TypeError {
            msg: format!("expected closure, got {}", closure.type_label()),
        }),
    }
}

fn list_method_with_closure<F>(
    method_name: &str,
    receiver: Value,
    args: &[Value],
    env: &Rc<Env>,
    ctx: &InterpContext,
    f: F,
) -> InterpResult<Value>
where
    F: FnOnce(&RrbVector<Value>, &Value, &Rc<Env>, &InterpContext) -> InterpResult<Value>,
{
    let items = expect_list(&receiver, method_name)?;
    let closure = args.first().ok_or_else(|| InterpError::TypeError {
        msg: format!("{} requires a closure argument", method_name),
    })?;
    f(&items, closure, env, ctx)
}

thread_local! {
    static FLATTEN_COUNTERS: std::cell::Cell<(u64, u64)> = const { std::cell::Cell::new((0, 0)) };
}

/// Process-global (not thread-local): a background dump thread reads this
/// concurrently with the interpreting thread, so this map must be visible
/// across threads rather than per-thread like `FLATTEN_COUNTERS` above.
static FLATTEN_BY_SITE: std::sync::Mutex<
    Option<std::collections::HashMap<(&'static str, u32), (u64, u64)>>,
> = std::sync::Mutex::new(None);

pub fn flatten_counters_snapshot() -> (u64, u64) {
    FLATTEN_COUNTERS.with(|c| c.get())
}

/// Per-call-site attribution for the `free_monoid_to_vec` O(n) materialization
/// cost, keyed by the immediate caller's `file:line` (`#[track_caller]`).
/// Residual-hunt instrumentation for adhoc-c328b166-bca's follow-on (datetime.dag
/// still DNF after the three parse-stage fixes) -- MEASURE FIRST before any cut.
pub fn flatten_by_site_snapshot() -> Vec<(&'static str, u32, u64, u64)> {
    let guard = FLATTEN_BY_SITE.lock().unwrap();
    match guard.as_ref() {
        Some(m) => m
            .iter()
            .map(|((file, line), (calls, total))| (*file, *line, *calls, *total))
            .collect(),
        None => Vec::new(),
    }
}

/// adhoc-c328b166-bca follow-on: `flatten_by_site_snapshot` attributes big
/// materializations only to the interpreter-internal call site, which for the
/// residual whale is always `eval_fold_list_native` -- useless granularity.
/// This keys the same signal by the fold closure's .dag source span instead,
/// so the dump names the v2-level fold that owns the cost.
static BIG_FOLD_BY_DAG_SITE: std::sync::Mutex<
    Option<std::collections::HashMap<String, (u64, u64)>>,
> = std::sync::Mutex::new(None);

fn record_big_fold_dag_site(closure: &Value, items: usize) {
    let key = match closure {
        Value::Closure { body, .. } => format!("{}:{}", body.span.file, body.span.start),
        Value::Fn { node } => format!("fn@{}:{}", node.span.file, node.span.start),
        other => format!("non-closure:{}", other.type_label()),
    };
    let mut guard = BIG_FOLD_BY_DAG_SITE.lock().unwrap();
    let map = guard.get_or_insert_with(std::collections::HashMap::new);
    let entry = map.entry(key).or_insert((0, 0));
    entry.0 += 1;
    entry.1 += items as u64;
}

pub fn big_fold_by_dag_site_snapshot() -> Vec<(String, u64, u64)> {
    let guard = BIG_FOLD_BY_DAG_SITE.lock().unwrap();
    match guard.as_ref() {
        Some(m) => m
            .iter()
            .map(|(site, (calls, total))| (site.clone(), *calls, *total))
            .collect(),
        None => Vec::new(),
    }
}

/// adhoc-c328b166-bca follow-on: inclusive wall-time per native builtin
/// (function-style and method-style dispatch), to localize the residual
/// whale when it lives in native code the fold counters cannot see (the
/// medium-fixture run showed a ~10-minute window with frozen fold counters
/// and climbing RSS). Inclusive: a fold's time contains its closure applies.
static BUILTIN_TIME: std::sync::Mutex<Option<std::collections::HashMap<String, (u64, u64)>>> =
    std::sync::Mutex::new(None);

fn record_builtin_time_inclusive(name: &str, method_style: bool, nanos: u64) {
    let key = if method_style {
        format!("m:{}", name)
    } else {
        name.to_string()
    };
    let mut guard = BUILTIN_TIME.lock().unwrap();
    let map = guard.get_or_insert_with(std::collections::HashMap::new);
    let entry = map.entry(key).or_insert((0, 0));
    entry.0 += 1;
    entry.1 += nanos;
}

pub fn builtin_time_snapshot() -> Vec<(String, u64, u64)> {
    let guard = BUILTIN_TIME.lock().unwrap();
    match guard.as_ref() {
        Some(m) => m
            .iter()
            .map(|(name, (calls, nanos))| (name.clone(), *calls, *nanos))
            .collect(),
        None => Vec::new(),
    }
}

// adhoc-c328b166-bca follow-on: self-time profile per .dag function. The
// builtin-time table showed native builtins near zero while wall-clock
// climbed, so the residual whale is tree-walk residency inside .dag bodies;
// this names the bodies. Self-time = inclusive minus child call_function
// frames (closure applies inside a body attribute to that body).
thread_local! {
    static DAG_PROF_CHILD_STACK: std::cell::RefCell<Vec<u64>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

static DAG_FN_SELF_TIME: std::sync::Mutex<Option<std::collections::HashMap<String, (u64, u64)>>> =
    std::sync::Mutex::new(None);

fn record_dag_fn_self_time(name: &str, self_nanos: u64) {
    let mut guard = DAG_FN_SELF_TIME.lock().unwrap();
    let map = guard.get_or_insert_with(std::collections::HashMap::new);
    if let Some(entry) = map.get_mut(name) {
        entry.0 += 1;
        entry.1 += self_nanos;
    } else {
        map.insert(name.to_string(), (1, self_nanos));
    }
}

pub fn dag_fn_self_time_snapshot() -> Vec<(String, u64, u64)> {
    let guard = DAG_FN_SELF_TIME.lock().unwrap();
    match guard.as_ref() {
        Some(m) => m
            .iter()
            .map(|(name, (calls, nanos))| (name.clone(), *calls, *nanos))
            .collect(),
        None => Vec::new(),
    }
}

// SCAFFOLD (adhoc-c328b166-bca residual hunt, nimble-otter-476): the innermost
// `.dag` function name, pushed on each `call_function` entry (RAII-popped on
// exit). `fold_list` is a builtin dispatched WITHOUT its own `call_function`
// frame, so the top of this stack names the `.dag` function that CONTAINS the
// fold_list call -- the O(n^2) re-fold caller the datetime DNF hunt is chasing.
// Gated behind `GUNBC_FLATTEN_SITE_DUMP_SECS`; a no-op (no push/pop) otherwise.
// dissolve-on: same as the recorders above -- delete with the residual-hunt
// work item, not a permanent profiler.
thread_local! {
    static CURRENT_DAG_FN: std::cell::RefCell<Vec<Rc<str>>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

pub(crate) struct DagFnGuard(bool);
impl DagFnGuard {
    pub(crate) fn enter(name: &str) -> Self {
        if residual_hunt_forensics_enabled() {
            CURRENT_DAG_FN.with(|s| s.borrow_mut().push(Rc::from(name)));
            DagFnGuard(true)
        } else {
            DagFnGuard(false)
        }
    }
}
impl Drop for DagFnGuard {
    fn drop(&mut self) {
        if self.0 {
            CURRENT_DAG_FN.with(|s| {
                s.borrow_mut().pop();
            });
        }
    }
}

fn current_dag_fn() -> String {
    CURRENT_DAG_FN.with(|s| {
        s.borrow()
            .last()
            .map(|r| r.to_string())
            .unwrap_or_else(|| "<none>".to_string())
    })
}

/// Caller attribution for LARGE left-folds (`eval_fold_list_native`, the datetime
/// driver: ~5k-element lists folded thousands of times). Keyed by the `.dag`
/// function containing the `fold_list` call. Tuple = (calls, total_items,
/// max_len, sample element `type_label`) -- the element type answers clever-koi's
/// deep-clone-vs-Rc-bump axis (Str => deep, Variant/List => Rc-bump).
static FOLD_CALLER_STATS: std::sync::Mutex<
    Option<std::collections::HashMap<String, (u64, u64, u64, &'static str)>>,
> = std::sync::Mutex::new(None);

fn record_fold_caller(items_len: usize, sample_elem: Option<&Value>, kind: &'static str) {
    if !residual_hunt_forensics_enabled() || items_len < 100 {
        return;
    }
    let caller = format!("{} [{}]", current_dag_fn(), kind);
    let tl = sample_elem
        .map(|v| v.type_label_public())
        .unwrap_or("<empty>");
    let mut guard = FOLD_CALLER_STATS.lock().unwrap();
    let m = guard.get_or_insert_with(std::collections::HashMap::new);
    let e = m.entry(caller).or_insert((0, 0, 0, tl));
    e.0 += 1;
    e.1 += items_len as u64;
    if items_len as u64 > e.2 {
        e.2 = items_len as u64;
    }
    e.3 = tl;
}

pub fn fold_caller_snapshot() -> Vec<(String, u64, u64, u64, &'static str)> {
    let guard = FOLD_CALLER_STATS.lock().unwrap();
    match guard.as_ref() {
        Some(m) => m
            .iter()
            .map(|(k, (c, t, mx, tl))| (k.clone(), *c, *t, *mx, *tl))
            .collect(),
        None => Vec::new(),
    }
}

/// SCAFFOLD (adhoc-c328b166-bca residual hunt): the recorders below are
/// opt-in, not always-on -- gated on the same env var that gates the dump
/// (`GUNBC_FLATTEN_SITE_DUMP_SECS`), read once via OnceLock so the default
/// (unset) production path pays a single relaxed load, not a mutex lock or
/// HashMap/HashSet write, per call. dissolve-on: the residual-hunt work item
/// closes (adhoc-c328b166-bca) -- delete these recorders and their call
/// sites, they are not a permanent profiler.
fn residual_hunt_forensics_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("GUNBC_FLATTEN_SITE_DUMP_SECS").is_ok())
}

fn record_flatten(items: usize) {
    FLATTEN_COUNTERS.with(|c| {
        let (calls, total) = c.get();
        c.set((calls + 1, total + items as u64));
    });
}

fn record_flatten_site(items: usize, loc: &'static std::panic::Location<'static>) {
    if !residual_hunt_forensics_enabled() {
        return;
    }
    let mut guard = FLATTEN_BY_SITE.lock().unwrap();
    let m = guard.get_or_insert_with(std::collections::HashMap::new);
    let entry = m.entry((loc.file(), loc.line())).or_insert((0, 0));
    entry.0 += 1;
    entry.1 += items as u64;
}

/// Hypothesis-B instrumentation (adhoc-c328b166-bca residual hunt): every
/// `Cons { head, tail }` pattern match against a native `Value::List` clones
/// the receiver and `split_off(1)`s it to build `tail`. `im::Vector` makes
/// this O(log n) once tree-ified, not the O(n) `free_monoid_to_vec` disease --
/// but `list_tail`'s call volume across a memoized parse (one call per
/// position, threaded through `parse_current_position`) could still sum to a
/// superlinear total. `calls` and `receiver_len_sum` let the ladder answer
/// that by execution instead of by reading `im`'s source.
static LIST_CONS_TAIL_SPLIT: (std::sync::atomic::AtomicU64, std::sync::atomic::AtomicU64) = (
    std::sync::atomic::AtomicU64::new(0),
    std::sync::atomic::AtomicU64::new(0),
);

fn record_list_cons_tail_split(receiver_len: usize) {
    if !residual_hunt_forensics_enabled() {
        return;
    }
    LIST_CONS_TAIL_SPLIT
        .0
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    LIST_CONS_TAIL_SPLIT
        .1
        .fetch_add(receiver_len as u64, std::sync::atomic::Ordering::Relaxed);
}

/// Hypothesis-A instrumentation (adhoc-c328b166-bca residual hunt): call
/// frequency for the grammar-analysis entry points S1's brief named as
/// candidates for a fixed, file-size-independent per-parse-module recompute
/// (`grammar_validate_for_parse`, `compute_nullable_set`,
/// `compute_production_first_rows`). A named, tiny watchlist (not a general
/// profiler) so the ladder answers "how many times, relative to file size"
/// by execution.
static CALL_FREQUENCY_WATCHLIST: std::sync::Mutex<
    Option<std::collections::HashMap<&'static str, u64>>,
> = std::sync::Mutex::new(None);

/// adhoc-c328b166-bca memo-effectiveness discriminator: distinct (grammar_digest,
/// token_stream_digest, position, production) keys ever looked up, vs total lookups/hits.
/// `lookups >> distinct` with `hits == 0` is the smoking gun for "memo never serves a
/// re-attempted span" (a real cache-effectiveness bug); `lookups == distinct` is the
/// benign "every position visited exactly once" signature. Global (not per-InterpContext)
/// so the periodic dump thread (GUNBC_FLATTEN_SITE_DUMP_SECS), which never enters
/// with_active_context, can still read it -- survives a DNF, unlike ctx-scoped stats.
static PARSE_MEMO_LOOKUPS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static PARSE_MEMO_HITS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static PARSE_MEMO_DISTINCT_KEYS: std::sync::Mutex<
    Option<std::collections::HashSet<(String, String, i64, Symbol)>>,
> = std::sync::Mutex::new(None);

fn record_parse_memo_lookup(key: &(String, String, i64, Symbol), hit: bool) {
    if !residual_hunt_forensics_enabled() {
        return;
    }
    PARSE_MEMO_LOOKUPS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if hit {
        PARSE_MEMO_HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    let mut guard = PARSE_MEMO_DISTINCT_KEYS.lock().unwrap();
    guard
        .get_or_insert_with(std::collections::HashSet::new)
        .insert(key.clone());
}

pub fn parse_memo_global_snapshot() -> (u64, u64, u64) {
    let lookups = PARSE_MEMO_LOOKUPS.load(std::sync::atomic::Ordering::Relaxed);
    let hits = PARSE_MEMO_HITS.load(std::sync::atomic::Ordering::Relaxed);
    let distinct = PARSE_MEMO_DISTINCT_KEYS
        .lock()
        .unwrap()
        .as_ref()
        .map(|s| s.len() as u64)
        .unwrap_or(0);
    (lookups, hits, distinct)
}

fn record_call_frequency(func_name: &str) {
    if !residual_hunt_forensics_enabled() {
        return;
    }
    const WATCHLIST: &[&str] = &[
        "grammar_validate_for_parse",
        "compute_nullable_set",
        "compute_production_first_rows",
        "parse_diags_to_diagnostics",
        "parse_diags_to_non_empty",
        "parse_diag_cons",
        "parse_production",
        "parse_expr",
        "list_snoc_item",
        "list_append",
        "parse_sync_step",
        "parse_match_arm_stmt_step",
        "parse_expr_repeat",
        "parse_expr_repeat_step",
        "parse_skip_to_sync",
        "parse_expr_sequence",
        "parse_expr_choice",
        "parse_expr_optional",
        "parse_production_memo_stats",
        "filter",
        "upsert_production_first_row",
        "parse_current_position",
        "parse_nonterminal_memoized",
        "parse_nonterminal_memoized_core",
        "parse_table_record_lookup_call",
        "parse_table_record_hit",
        "parse_table_record_miss",
        "parse_table_lookup",
        "parse_table_insert",
        "parse_choice_residue_backtrack",
    ];
    let Some(key) = WATCHLIST.iter().find(|w| **w == func_name) else {
        return;
    };
    let mut guard = CALL_FREQUENCY_WATCHLIST.lock().unwrap();
    let m = guard.get_or_insert_with(std::collections::HashMap::new);
    *m.entry(*key).or_insert(0) += 1;
}

pub fn call_frequency_snapshot() -> Vec<(&'static str, u64)> {
    let guard = CALL_FREQUENCY_WATCHLIST.lock().unwrap();
    match guard.as_ref() {
        Some(m) => m.iter().map(|(k, v)| (*k, *v)).collect(),
        None => Vec::new(),
    }
}

pub fn list_cons_tail_split_snapshot() -> (u64, u64) {
    (
        LIST_CONS_TAIL_SPLIT
            .0
            .load(std::sync::atomic::Ordering::Relaxed),
        LIST_CONS_TAIL_SPLIT
            .1
            .load(std::sync::atomic::Ordering::Relaxed),
    )
}

pub const EXPR_VARIANT_COUNT: usize = 22;

fn expr_variant_index(d: &ExprData) -> usize {
    match d {
        ExprData::NoExprData => 0,
        ExprData::ExprLiteral { .. } => 1,
        ExprData::ExprError { .. } => 2,
        ExprData::ExprVar { .. } => 3,
        ExprData::ExprFieldAccess { .. } => 4,
        ExprData::ExprCall { .. } => 5,
        ExprData::ExprMethodCall { .. } => 6,
        ExprData::ExprMatch => 7,
        ExprData::ExprIf => 8,
        ExprData::ExprLet => 9,
        ExprData::ExprRecordLit { .. } => 10,
        ExprData::ExprListLit => 11,
        ExprData::ExprBinOp { .. } => 12,
        ExprData::ExprUnaryOp { .. } => 13,
        ExprData::ExprLambda => 14,
        ExprData::ExprStringInterp => 15,
        ExprData::ExprBlock => 16,
        ExprData::ExprCast => 17,
        ExprData::ExprForEach => 18,
        ExprData::ExprIndex => 19,
        ExprData::ExprSlice => 20,
        ExprData::ExprReturn => 21,
    }
}

pub fn expr_variant_name(i: usize) -> &'static str {
    const NAMES: [&str; EXPR_VARIANT_COUNT] = [
        "NoExprData",
        "ExprLiteral",
        "ExprError",
        "ExprVar",
        "ExprFieldAccess",
        "ExprCall",
        "ExprMethodCall",
        "ExprMatch",
        "ExprIf",
        "ExprLet",
        "ExprRecordLit",
        "ExprListLit",
        "ExprBinOp",
        "ExprUnaryOp",
        "ExprLambda",
        "ExprStringInterp",
        "ExprBlock",
        "ExprCast",
        "ExprForEach",
        "ExprIndex",
        "ExprSlice",
        "ExprReturn",
    ];
    NAMES.get(i).copied().unwrap_or("?")
}

thread_local! {
    static EVAL_COUNTS: RefCell<[u64; EXPR_VARIANT_COUNT]> =
        const { RefCell::new([0; EXPR_VARIANT_COUNT]) };
    static EVAL_SELF_NANOS: RefCell<[u128; EXPR_VARIANT_COUNT]> =
        const { RefCell::new([0; EXPR_VARIANT_COUNT]) };
    static ACTIVE_SUBJECT: RefCell<Option<String>> = const { RefCell::new(None) };
    static SUBJECT_SELF_NANOS: RefCell<HashMap<String, u128>> = RefCell::new(HashMap::new());
    static CHILD_NANOS: Cell<u128> = const { Cell::new(0) };
    static PROFILE_FLAG: Cell<Option<bool>> = const { Cell::new(None) };
}

fn eval_profile_enabled() -> bool {
    PROFILE_FLAG.with(|c| match c.get() {
        Some(b) => b,
        None => {
            let b = std::env::var("GUNBC_INTERP_PROFILE")
                .map(|v| v == "1")
                .unwrap_or(false);
            c.set(Some(b));
            b
        }
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerformanceReceipt {
    pub subject_key: String,
    pub work_shape: String,
    pub wall_nanos: u128,
    pub eval_self_nanos: u128,
    pub sample_count: u64,
}

pub fn subject_self_nanos_snapshot() -> HashMap<String, u128> {
    SUBJECT_SELF_NANOS.with(|m| m.borrow().clone())
}

pub fn eval_subject_set(subject_key: String) {
    ACTIVE_SUBJECT.with(|s| *s.borrow_mut() = Some(subject_key));
}

pub fn eval_subject_clear() {
    ACTIVE_SUBJECT.with(|s| *s.borrow_mut() = None);
}

pub fn eval_subject_timing_reset() {
    SUBJECT_SELF_NANOS.with(|m| m.borrow_mut().clear());
    CHILD_NANOS.set(0);
}

pub fn performance_receipt_from_witness(
    subject_key: String,
    work_shape: &str,
    wall_nanos: u128,
) -> PerformanceReceipt {
    let eval_self_nanos = SUBJECT_SELF_NANOS
        .with(|m| m.borrow().get(&subject_key).copied())
        .unwrap_or(0);
    PerformanceReceipt {
        subject_key,
        work_shape: work_shape.to_string(),
        wall_nanos,
        eval_self_nanos,
        sample_count: 1,
    }
}

#[derive(Clone)]
pub struct EvalProfile {
    pub counts: [u64; EXPR_VARIANT_COUNT],
    pub self_nanos: [u128; EXPR_VARIANT_COUNT],
}

pub fn eval_profile_snapshot() -> EvalProfile {
    EvalProfile {
        counts: EVAL_COUNTS.with(|c| *c.borrow()),
        self_nanos: EVAL_SELF_NANOS.with(|c| *c.borrow()),
    }
}

pub fn eval_profile_reset() {
    EVAL_COUNTS.with(|c| *c.borrow_mut() = [0; EXPR_VARIANT_COUNT]);
    EVAL_SELF_NANOS.with(|c| *c.borrow_mut() = [0; EXPR_VARIANT_COUNT]);
    eval_subject_timing_reset();
    CHILD_NANOS.set(0);
}

/// O(1) length for values whose native realization already tracks it,
/// bypassing `free_monoid_to_vec`'s O(n) materialization. `parse_current_position`
/// (v2 02_parse.dag) calls `length` on the full token stream every parse
/// attempt; without this fast path that is an O(n) clone per attempt, an
/// O(n^2) tax the compiled (Rust-emitted) realization never pays.
pub(crate) fn native_len(val: &Value) -> Option<i64> {
    match val {
        Value::List(items) => Some(items.len() as i64),
        Value::Map(m) => Some(m.len() as i64),
        Value::Set(s) => Some(s.len() as i64),
        _ => None,
    }
}

#[track_caller]
pub(crate) fn free_monoid_to_vec(val: &Value) -> Option<Vec<Value>> {
    let site = std::panic::Location::caller();
    let mut out = Vec::new();
    let mut cur = val.clone();
    let monoid_syms = active_ctx().map(|ctx| {
        (
            ctx.sym("Empty"),
            ctx.sym("Cons"),
            ctx.sym("head"),
            ctx.sym("tail"),
        )
    });
    loop {
        match &cur {
            Value::List(items) => {
                out.extend(items.iter().cloned());
                record_flatten(out.len());
                record_flatten_site(out.len(), site);
                return Some(out);
            }
            Value::Str(s) => {
                out.extend(s.chars().map(char_value));
                record_flatten(out.len());
                record_flatten_site(out.len(), site);
                return Some(out);
            }
            Value::Variant {
                variant_name,
                fields,
                ..
            } => {
                let (empty_sym, cons_sym, head_sym, tail_sym) = monoid_syms?;
                if *variant_name == empty_sym {
                    record_flatten(out.len());
                    record_flatten_site(out.len(), site);
                    return Some(out);
                }
                if *variant_name == cons_sym {
                    match (fields_get(fields, head_sym), fields_get(fields, tail_sym)) {
                        (Some(head), Some(tail)) => {
                            out.push(head.clone());
                            cur = tail.clone();
                        }
                        _ => return None,
                    }
                } else {
                    return None;
                }
            }
            _ => return None,
        }
    }
}

/// Fail-closed backstop for the model↔realization String straddle (DESIGN §5).
/// At a String-meeting point (a free monoid concatenated with a native
/// `Value::Str`), grounding (`free_monoid_to_string`) has already consumed every
/// well-typed String (all-codepoint, rendered to a native `Value::Str`).
/// Reaching the list path therefore means the operand is *not* a pure codepoint
/// list — and if it nonetheless contains a `Char` codepoint (`Value::Int`), it
/// is a *mixed* `[codepoint.., non-codepoint]` value: the straddle this
/// grounding exists to dissolve. We refuse LOUDLY (turning the prior §5
/// fail-open — `Accepted` carrying a wrong-type mixed list — into a typed error)
/// rather than fabricate it. A homogeneous `List<String>` (all `Value::Str`)
/// carries no codepoint and is legitimate, so it passes. This is the
/// completeness insurance for grounding the known sites: any future un-grounded
/// `FreeMonoid<Char>` × `Str` meeting point surfaces here as a loud error
/// instead of silently straddling again.
fn string_realization_straddle_detail(orig: &Value, items: &[Value]) -> Option<String> {
    // A `Value::List` is a generic collection, never a straddled String (see
    // `free_monoid_to_string`); its `Int` elements are genuine data, so a `Str`
    // appended to it is a legitimate heterogeneous element, not a straddle. Only
    // a `Cons`-chain / `Str`-derived flattening carries codepoint semantics.
    //
    // OPEN THREAD (DESIGN §6 residue — named, not silently shipped): this
    // `Value::List` exemption makes the wall a RATCHET WITH A NAMED HOLE, not a
    // universal value-level wall. The `"chars"` method (this file) materializes
    // a string as a `Value::List` of codepoint `Int`s, structurally identical to
    // a generic `Int` list — so a `.chars()`-result straddled with a native
    // `Str` would be exempted here and fail open (the original bug, uncaught).
    // This is undecidable at the Value level (a codepoint list and a generic
    // `Int` list are element-identical), so it is honest §6 residue, the
    // `Value::Null` pattern. LATENT today: no `.dag` program evaluates the
    // interpreter `chars` method into a concat/`+` with a `Str` (the two
    // `.chars()` rows in `languages.dag` / `rust/emit.dag` are emit *templates*,
    // not interpreter calls). DISSOLVES WHEN `.chars()` / `Char` is regrounded so
    // a codepoint-sequence is distinguishable from a generic `Int` list at the
    // realization level (the grounding root, sibling to Int↔Nat #5428).
    if matches!(orig, Value::List(_)) {
        return None;
    }
    if items.iter().any(|x| matches!(x, Value::Int(_))) {
        Some(format!(
            "free monoid mixing Char codepoints with a native String at a concat/`+` meeting point ({} elements); a String must realize as a single native Value::Str, never a mixed [codepoint.., Str] list",
            items.len()
        ))
    } else {
        None
    }
}

/// String grounding (DESIGN §1/§2/§7, model↔realization fork): render a
/// string-like free monoid (`String = FreeMonoid<Char>`, `Char = Nat`) to its
/// native realization. A native `Value::Str` is already grounded; a modeled
/// `Empty`/`Cons` chain or `List` is a String **only** when every element is a
/// `Char` codepoint (`Value::Int`). A `Value::Str` *element* (not the whole
/// value) means `List<String>`, not `String`, so it returns `None` — that
/// discriminator is what keeps `List<String>` push/concat from collapsing into
/// one string. Used so a folded String concatenation realizes as a single
/// `Value::Str` instead of straddling as a mixed `[codepoint.., Str]` list that
/// fails `==` against a native String oracle (the held emit-weld debt).
pub(crate) fn free_monoid_to_string(val: &Value) -> Option<String> {
    if let Value::Str(s) = val {
        return Some(s.clone());
    }
    // A `Value::List` is a generic ordered collection (the `[1]`/`[1,2,3]` list
    // literal representation), NEVER a modeled `String`. A modeled
    // `FreeMonoid<Char>` realizes as an `Empty`/`Cons` `Value::Variant` chain.
    // Treating a `List` as string-like would collapse `List<Int>` append/`+`/
    // concat into one string — exactly what the `list_free_monoid_chokepoint`
    // tests forbid (`[1] + "ab"` stays length 2). Only a native `Str` or a
    // `Cons`-chain is a String candidate; representation is the discriminator
    // the Value level affords (a `List<Int>` and a codepoint `Cons`-chain are
    // otherwise element-identical).
    if matches!(val, Value::List(_)) {
        return None;
    }
    let items = free_monoid_to_vec(val)?;
    let mut out = String::new();
    for it in items {
        match it {
            Value::Int(n) => out.push(u32::try_from(n).ok().and_then(char::from_u32)?),
            _ => return None,
        }
    }
    Some(out)
}

fn value_to_list_carrier(val: &Value) -> Option<(Rc<RrbVector<Value>>, u64)> {
    match val {
        Value::List(items) => Some((items.clone(), 0)),
        _ => free_monoid_to_vec(val).map(|items| {
            let copied = items.len() as u64;
            (Rc::new(RrbVector::from(items)), copied)
        }),
    }
}

fn list_get_at_or_null(items: &RrbVector<Value>, idx: i64) -> Value {
    items.get(idx as usize).cloned().unwrap_or(Value::Null)
}

fn expect_list(val: &Value, context: &str) -> InterpResult<Rc<RrbVector<Value>>> {
    match val {
        Value::List(items) => Ok(items.clone()),
        _ => match free_monoid_to_vec(val) {
            Some(items) => Ok(Rc::new(RrbVector::from(items))),
            None => Err(InterpError::TypeError {
                msg: format!("{} expects a list, got {}", context, val.type_label()),
            }),
        },
    }
}

fn is_map_lookup_receiver(val: &Value) -> bool {
    match val {
        Value::Map(_) => true,
        Value::Record { fields, .. } | Value::Variant { fields, .. } => active_ctx()
            .map(|ctx| fields_get(fields, ctx.sym("lookup")).is_some())
            .unwrap_or(false),
        _ => false,
    }
}

fn raw_map_lookup_witness(
    map: &Value,
    key: &Value,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    match map {
        Value::Map(m) => match CanonKey::new(key.clone()) {
            Some(ck) => match m.get(&ck) {
                Some(v) => Ok(witness_holds(v.clone(), ctx)),
                None => Ok(witness_violates(
                    native_map_absent_diagnostic_value(ctx),
                    ctx,
                )),
            },
            None => Ok(witness_violates(
                native_map_absent_diagnostic_value(ctx),
                ctx,
            )),
        },
        _ => raw_map_lookup(map, key, env, ctx),
    }
}

fn raw_map_lookup(
    map: &Value,
    key: &Value,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    match map {
        Value::Map(m) => match CanonKey::new(key.clone()) {
            Some(ck) => Ok(m.get(&ck).cloned().unwrap_or(Value::Null)),
            None => Ok(Value::Null),
        },
        Value::Record { fields, .. } | Value::Variant { fields, .. } => {
            let lookup_sym = ctx.sym("lookup");
            match fields_get(fields, lookup_sym) {
                Some(lookup @ Value::Closure { .. }) => {
                    apply_closure(lookup, &[key.clone()], env, ctx)
                }
                Some(Value::Fn { node }) => {
                    let named = vec![(None, key.clone())];
                    call_function(ctx, node, &named, env)
                }
                Some(_) => Err(InterpError::TypeError {
                    msg: "Map.lookup field is not callable".to_string(),
                }),
                None => match key {
                    Value::Str(s) => {
                        let k = ctx.sym(s);
                        Ok(fields_get(fields, k).cloned().unwrap_or(Value::Null))
                    }
                    _ => Ok(Value::Null),
                },
            }
        }
        _ => Err(InterpError::TypeError {
            msg: format!("raw_map_lookup expects Map, got {}", map.type_label()),
        }),
    }
}

fn expect_map(val: &Value, context: &str) -> InterpResult<Rc<HamtMap<CanonKey, Value>>> {
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
            msg: format!(
                "{} expects a string argument, got {}",
                context,
                v.type_label()
            ),
        }),
        None => Err(InterpError::TypeError {
            msg: format!("{} requires a string argument", context),
        }),
    }
}

fn expect_byte_vec(val: Option<&Value>, context: &str) -> InterpResult<Vec<u8>> {
    match val {
        Some(Value::List(items)) => {
            let mut out: Vec<u8> = Vec::with_capacity(items.len());
            for item in items.iter() {
                match item {
                    Value::Int(n) if (0..=255).contains(n) => out.push(*n as u8),
                    other => {
                        return Err(InterpError::TypeError {
                            msg: format!(
                                "{} expects Bytes (List of 0..255), got element {}",
                                context,
                                other.type_label()
                            ),
                        });
                    }
                }
            }
            Ok(out)
        }
        Some(v) => Err(InterpError::TypeError {
            msg: format!("{} expects Bytes (List), got {}", context, v.type_label()),
        }),
        None => Err(InterpError::TypeError {
            msg: format!("{} requires a Bytes argument", context),
        }),
    }
}

fn expect_str_list(val: Option<&Value>, context: &str) -> InterpResult<Vec<String>> {
    match val {
        Some(Value::List(items)) => {
            let mut out: Vec<String> = Vec::new();
            for item in items.iter() {
                match item {
                    Value::Str(s) => out.push(s.clone()),
                    other => {
                        return Err(InterpError::TypeError {
                            msg: format!(
                                "{} expects a List<String>, got element {}",
                                context,
                                other.type_label()
                            ),
                        })
                    }
                }
            }
            Ok(out)
        }
        Some(v) => Err(InterpError::TypeError {
            msg: format!(
                "{} expects a List<String> argument, got {}",
                context,
                v.type_label()
            ),
        }),
        None => Err(InterpError::TypeError {
            msg: format!("{} requires a List<String> argument", context),
        }),
    }
}

fn expect_str_list_flex(val: Option<&Value>, context: &str) -> InterpResult<Vec<String>> {
    let Some(v) = val else {
        return Err(InterpError::TypeError {
            msg: format!("{} requires a List<String> argument", context),
        });
    };
    let Some(items) = free_monoid_to_vec(v) else {
        return Err(InterpError::TypeError {
            msg: format!(
                "{} expects a List<String> argument, got {}",
                context,
                v.type_label()
            ),
        });
    };
    let mut out: Vec<String> = Vec::new();
    for item in items {
        match item {
            Value::Str(s) => out.push(s),
            other => {
                return Err(InterpError::TypeError {
                    msg: format!(
                        "{} expects a List<String>, got element {}",
                        context,
                        other.type_label()
                    ),
                })
            }
        }
    }
    Ok(out)
}

fn expect_int_list_flex(val: Option<&Value>, context: &str) -> InterpResult<Vec<i64>> {
    let Some(v) = val else {
        return Err(InterpError::TypeError {
            msg: format!("{} requires a List<Int> argument", context),
        });
    };
    let Some(items) = free_monoid_to_vec(v) else {
        return Err(InterpError::TypeError {
            msg: format!(
                "{} expects a List<Int> argument, got {}",
                context,
                v.type_label()
            ),
        });
    };
    let mut out: Vec<i64> = Vec::new();
    for item in items {
        match item {
            Value::Int(n) => out.push(n),
            other => {
                return Err(InterpError::TypeError {
                    msg: format!(
                        "{} expects a List<Int>, got element {}",
                        context,
                        other.type_label()
                    ),
                })
            }
        }
    }
    Ok(out)
}

fn expect_float_list_flex(val: Option<&Value>, context: &str) -> InterpResult<Vec<f64>> {
    let Some(v) = val else {
        return Err(InterpError::TypeError {
            msg: format!("{} requires a List<Float> argument", context),
        });
    };
    let Some(items) = free_monoid_to_vec(v) else {
        return Err(InterpError::TypeError {
            msg: format!(
                "{} expects a List<Float> argument, got {}",
                context,
                v.type_label()
            ),
        });
    };
    let mut out: Vec<f64> = Vec::new();
    for item in items {
        match item {
            Value::Float(n) => out.push(n),
            other => {
                return Err(InterpError::TypeError {
                    msg: format!(
                        "{} expects a List<Float>, got element {}",
                        context,
                        other.type_label()
                    ),
                })
            }
        }
    }
    Ok(out)
}

fn expect_fma_contraction_policy_wire(val: Option<&Value>, context: &str) -> InterpResult<i64> {
    let Some(v) = val else {
        return Err(InterpError::TypeError {
            msg: format!("{context} requires FmaContractionRefused | FmaContractionPermitted"),
        });
    };
    match v {
        Value::Variant { variant_name, .. } => {
            let name = resolve_sym(*variant_name);
            if name == "FmaContractionRefused" {
                Ok(0)
            } else if name == "FmaContractionPermitted" {
                Ok(1)
            } else {
                Err(InterpError::TypeError {
                    msg: format!(
                        "{context} requires FmaContractionRefused | FmaContractionPermitted, got `{name}`"
                    ),
                })
            }
        }
        other => Err(InterpError::TypeError {
            msg: format!(
                "{context} requires FmaContractionRefused | FmaContractionPermitted, got {}",
                other.type_label()
            ),
        }),
    }
}

fn expect_int(val: Option<&Value>, context: &str) -> InterpResult<i64> {
    match val {
        Some(Value::Int(n)) => Ok(*n),
        Some(v) => Err(InterpError::TypeError {
            msg: format!(
                "{} expects an int argument, got {}",
                context,
                v.type_label()
            ),
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

#[cfg(test)]
mod base64_std_tests {
    use super::base64_encode_std;

    #[test]
    fn rfc4648_test_vectors() {
        // RFC 4648 §10 test vectors — the fixed alphabet + padding a Basic credential relies on.
        assert_eq!(base64_encode_std(b""), "");
        assert_eq!(base64_encode_std(b"f"), "Zg==");
        assert_eq!(base64_encode_std(b"fo"), "Zm8=");
        assert_eq!(base64_encode_std(b"foo"), "Zm9v");
        assert_eq!(base64_encode_std(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode_std(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode_std(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn basic_credential_shape() {
        // The exact header value a BMC Basic auth op must send for user:pass.
        assert_eq!(
            base64_encode_std(b"bmcadmin:s3cret"),
            "Ym1jYWRtaW46czNjcmV0"
        );
        // Bytes with high bits set exercise the +/ tail of the alphabet.
        assert_eq!(base64_encode_std(&[0xfb, 0xff, 0xfe]), "+//+");
    }
}

#[cfg(test)]
mod dispatch_rest_decision_tests {
    use super::rest_auth_authority_conflict;
    use super::rest_basic_auth_header_value;
    use super::rest_tls_posture_interp_disposition;

    #[test]
    fn basic_auth_header_is_exact_rfc7617_value() {
        // The header dispatch_rest sets for auth_basic — discriminating: the exact Base64(user:pass).
        assert_eq!(
            rest_basic_auth_header_value("bmcadmin", "s3cret"),
            "Basic Ym1jYWRtaW46czNjcmV0"
        );
        // A different credential must produce a different header (no fixed/empty header).
        assert_ne!(
            rest_basic_auth_header_value("bmcadmin", "s3cret"),
            rest_basic_auth_header_value("bmcadmin", "wrong")
        );
    }

    #[test]
    fn tls_posture_disposition_fails_closed() {
        // VerifyPeer proceeds; InsecureAcceptAnyCert and unknown refuse (emit-only decision).
        assert!(rest_tls_posture_interp_disposition("VerifyPeer").is_ok());
        assert!(rest_tls_posture_interp_disposition("InsecureAcceptAnyCert").is_err());
        assert!(rest_tls_posture_interp_disposition("TrustEveryone").is_err());
    }

    #[test]
    fn dual_auth_conflict_rule() {
        // Both authorities present is the only conflict; either alone (or neither) is fine.
        assert!(rest_auth_authority_conflict(true, true));
        assert!(!rest_auth_authority_conflict(true, false));
        assert!(!rest_auth_authority_conflict(false, true));
        assert!(!rest_auth_authority_conflict(false, false));
    }
}

#[cfg(test)]
mod shell_completion_trace_tests {
    use super::hermetic_checkout_read_disposition_under;
    use super::shell_completion_stderr_trace_block;
    use super::shell_completion_trace_line;
    use std::time::Duration;

    #[test]
    fn shell_completion_trace_line_formats_exit_stdout_stderr_wall() {
        let line = shell_completion_trace_line(0, 1234, 56, Duration::from_millis(5150));
        assert_eq!(
            line,
            "[shell] done exit=0 stdout=1234 stderr=56 bytes wall=5.150s"
        );
    }

    #[test]
    fn stderr_block_surfaces_content_on_nonzero_exit() {
        let block = shell_completion_stderr_trace_block(101, b"error: manifest not found\n")
            .expect("non-zero exit with stderr must surface a diagnostic block");
        assert!(block.starts_with("[shell] stderr (exit=101):\n"));
        assert!(block.contains("error: manifest not found"));
    }

    #[test]
    fn stderr_block_none_on_success_or_empty() {
        // RED control: success (even with stderr) and empty-stderr failures surface nothing,
        // so benign compiler warnings never drown the trace and there is no empty block noise.
        assert_eq!(
            shell_completion_stderr_trace_block(0, b"warning: unused\n"),
            None
        );
        assert_eq!(shell_completion_stderr_trace_block(1, b""), None);
    }

    #[test]
    fn stderr_block_tail_bounds_and_marks_elision() {
        let big = vec![b'x'; 16384 + 500];
        let block =
            shell_completion_stderr_trace_block(1, &big).expect("oversized stderr surfaces");
        assert!(block.contains("<500 earlier stderr bytes elided>"));
        // Only the 16384-byte tail is carried, not the full 16884-byte body: the trailing
        // contiguous run of stderr bytes is exactly the cap.
        assert_eq!(block.chars().rev().take_while(|c| *c == 'x').count(), 16384);
    }

    #[test]
    fn shell_completion_trace_line_surfaces_nonzero_exit() {
        let line = shell_completion_trace_line(1, 0, 4096, Duration::from_secs(2));
        assert_eq!(
            line,
            "[shell] done exit=1 stdout=0 stderr=4096 bytes wall=2.000s"
        );
    }

    #[test]
    fn hermetic_checkout_read_admits_relative_path_under_root() {
        let dir =
            std::env::temp_dir().join(format!("hermetic-carveout-admit-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("dag/std")).unwrap();
        std::fs::write(dir.join("dag/std/x.dag"), "module x\n").unwrap();
        assert_eq!(
            hermetic_checkout_read_disposition_under(&dir, "dag/std/x.dag"),
            Ok(())
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn hermetic_checkout_read_refuses_traversal_escape_and_absolute_outside() {
        let dir =
            std::env::temp_dir().join(format!("hermetic-carveout-escape-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let escape = hermetic_checkout_read_disposition_under(&dir, "../outside.txt");
        assert!(
            escape.is_err(),
            "`..` traversal must refuse, got {escape:?}"
        );
        let absolute = hermetic_checkout_read_disposition_under(&dir, "/etc/hostname");
        assert!(
            absolute.is_err(),
            "absolute out-of-root path must refuse, got {absolute:?}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn hermetic_checkout_read_refuses_git_and_target_components() {
        let dir =
            std::env::temp_dir().join(format!("hermetic-carveout-gitdir-{}", std::process::id()));
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::create_dir_all(dir.join("target")).unwrap();
        std::fs::write(dir.join(".git/HEAD"), "ref: x\n").unwrap();
        std::fs::write(dir.join("target/receipt.txt"), "r\n").unwrap();
        let git = hermetic_checkout_read_disposition_under(&dir, ".git/HEAD");
        assert!(
            git.err()
                .is_some_and(|e| e.contains("not commit-deterministic")),
            ".git read must refuse as non-commit-deterministic"
        );
        let target = hermetic_checkout_read_disposition_under(&dir, "target/receipt.txt");
        assert!(
            target
                .err()
                .is_some_and(|e| e.contains("not commit-deterministic")),
            "target read must refuse as non-commit-deterministic"
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}

#[cfg(test)]
mod argv_arg_limit_test {
    use std::rc::Rc;

    use im::{vector as im_vec, HashMap};

    use crate::v1_compiler_infer_emit_info::empty_emit_graph_info;
    use crate::v1_compiler_infer_items::ResolvedGraph;
    use crate::v1_std_core::{make_span, make_text_part_node, shell_transport_node, Node};

    use super::{
        argv_arg_limit_refusal, dispatch_shell, Env, ExecutionMode, InterpContext, InterpError,
        HOST_ARG_MAX_STRLEN_BYTES,
    };

    fn argv_limit_test_context() -> InterpContext {
        let graph = ResolvedGraph {
            modules: Rc::new(im_vec![]),
            item_registry: Rc::new(HashMap::new()),
            diagnostics: Rc::new(im_vec![]),
            emit_graph_info: empty_emit_graph_info(),
        };
        InterpContext::new(&graph, Rc::new(HashMap::new()), ExecutionMode::Wet)
    }

    /// `shell.Exec.Check`-shaped argv: `sh -c "<command>"` as three literal tokens.
    fn shell_check_style_transport(command: &str) -> Rc<Node> {
        let span = make_span(0, 0);
        shell_transport_node(
            Rc::new(im_vec![
                make_text_part_node("sh".to_string(), span.clone()),
                make_text_part_node("-c".to_string(), span.clone()),
                make_text_part_node(command.to_string(), span.clone()),
            ]),
            Rc::new(im_vec![]),
            None,
            span,
        )
    }

    // Pin the Rust seed mirror to the single authority modeled in
    // extdeps/os/exec_arg_limit.dag (host_exec_arg_max_strlen = byte_size(131072),
    // Linux execve(2) MAX_ARG_STRLEN = 32 * 4096). Drift on either side reds a test.
    #[test]
    fn mirror_matches_extdeps_authority() {
        assert_eq!(HOST_ARG_MAX_STRLEN_BYTES, 131072);
    }

    // Discriminating boundary: exactly the limit passes (proceed to exec), one byte
    // over refuses with the typed, located error — carrying the offending byte count,
    // the modeled limit, and argv0 — never a truncation or a widen (DESIGN §5).
    #[test]
    fn wall_refuses_one_byte_over_and_passes_at_limit() {
        let limit = HOST_ARG_MAX_STRLEN_BYTES;

        // GREEN control: a small command-sequence argv proceeds (no refusal).
        let small = vec!["sh".to_string(), "-c".to_string(), "echo hi".to_string()];
        assert!(argv_arg_limit_refusal(&small, limit).is_none());

        // GREEN boundary: a single argument of exactly the limit is still admitted.
        let at_limit = vec!["sh".to_string(), "-c".to_string(), "x".repeat(limit)];
        assert!(argv_arg_limit_refusal(&at_limit, limit).is_none());

        // RED: one byte over the ceiling is refused with the typed diagnostic.
        let over = vec!["sh".to_string(), "-c".to_string(), "x".repeat(limit + 1)];
        match argv_arg_limit_refusal(&over, limit) {
            Some(InterpError::ArgvExceedsHostArgMax {
                actual_bytes,
                limit_bytes,
                argv0,
            }) => {
                assert_eq!(actual_bytes, limit + 1);
                assert_eq!(limit_bytes, limit);
                assert_eq!(argv0, "sh");
            }
            other => panic!("expected ArgvExceedsHostArgMax, got {other:?}"),
        }
    }

    // The ceiling is per SINGLE argument (MAX_ARG_STRLEN), not the argv total: many
    // small args summing over the limit are admitted — refusing them would be an
    // over-approximation (ARG_MAX is a separate, larger limit not modeled here).
    #[test]
    fn wall_is_per_argument_not_argv_total() {
        let limit = HOST_ARG_MAX_STRLEN_BYTES;
        let many_small: Vec<String> = std::iter::once("echo".to_string())
            .chain((0..8).map(|_| "y".repeat(limit / 4)))
            .collect();
        // total is ~2x the limit, but no single token exceeds it.
        assert!(argv_arg_limit_refusal(&many_small, limit).is_none());
    }

    // Wiring proof: `dispatch_shell` must reach `argv_arg_limit_refusal` on the
    // evaluated argv and refuse BEFORE any exec branch. Without that guard arm the
    // same transport would proceed to `Command::spawn` and surface an opaque spawn
    // error instead of `ArgvExceedsHostArgMax` (RED control — predicate-only tests
    // do not exercise this call path).
    #[test]
    fn dispatch_shell_wiring_refuses_oversized_argv() {
        let ctx = argv_limit_test_context();
        let transport = shell_check_style_transport(&"x".repeat(HOST_ARG_MAX_STRLEN_BYTES + 1));
        let env = Env::empty();
        match dispatch_shell(&transport, &env, &ctx) {
            Err(InterpError::ArgvExceedsHostArgMax {
                actual_bytes,
                limit_bytes,
                argv0,
            }) => {
                assert_eq!(actual_bytes, HOST_ARG_MAX_STRLEN_BYTES + 1);
                assert_eq!(limit_bytes, HOST_ARG_MAX_STRLEN_BYTES);
                assert_eq!(argv0, "sh");
            }
            Err(other) => panic!(
                "expected dispatch_shell to refuse via ArgvExceedsHostArgMax before spawn, got {other:?}"
            ),
            Ok(_) => panic!("expected refusal before spawn, got Ok"),
        }
    }

    // GREEN control on the wiring path: a small argv is not refused by the wall
    // (dispatch proceeds to spawn — wet, but the discriminating RED is pre-spawn).
    #[test]
    fn dispatch_shell_wiring_admits_small_argv() {
        let ctx = argv_limit_test_context();
        let transport = shell_check_style_transport("true");
        let env = Env::empty();
        match dispatch_shell(&transport, &env, &ctx) {
            Err(InterpError::ArgvExceedsHostArgMax { .. }) => {
                panic!("small argv must not trip the arg-size wall")
            }
            Ok(_) | Err(_) => {}
        }
    }
}
