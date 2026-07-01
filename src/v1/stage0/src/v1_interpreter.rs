use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::time::Instant;

use im_rc::HashMap as HamtMap;
use im_rc::Vector as RrbVector;

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
    return_value, slice_base, slice_end, slice_start, unaryop_operand, CallSemantics, Cardinality,
    Connective, ErrorNode, ExprData, FieldAccessStyle, FieldSummary, FieldValueShape, InferredNode,
    MatchPattern, MethodSemantics, NewlineIndex, Node, SourceSpan, StringPart, UnaryOpKind,
    VarBindingKind,
};
use crate::wire_value_serialize::value_to_wire_json;

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

pub fn qualified_name_value_to_module_path(value: &Value) -> String {
    match value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            let variant = resolve_sym(*variant_name);
            if variant == "QnEmpty" {
                return String::new();
            }
            if variant == "QnCons" {
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
                    .unwrap_or_else(|| {
                        panic!("qualified_name_to_module_path: QnCons.head not Str")
                    });
                let tail = fields
                    .iter()
                    .find(|(k, _)| resolve_sym(*k) == "tail")
                    .map(|(_, v)| v)
                    .expect("qualified_name_to_module_path: QnCons.tail missing");
                let rest = qualified_name_value_to_module_path(tail);
                if rest.is_empty() {
                    head
                } else {
                    format!("{head}.{rest}")
                }
            } else {
                panic!("qualified_name_to_module_path: unexpected variant '{variant}'");
            }
        }
        other => {
            panic!("qualified_name_to_module_path: expected QualifiedName variant, got {other:?}")
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
    Set(Rc<BTreeSet<String>>),
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
}

#[derive(Debug, Clone)]
pub enum InterpError {
    NoSuchFunction { name: String },
    NoMainFunction,
    NoSuchVariable { name: String },
    NoSuchField { type_name: String, field: String },
    TypeError { msg: String },
    CrossRepresentationEquality { detail: String },
    StringRealizationStraddle { detail: String },
    PatternMatchFailure { value: String },
    DivisionByZero,
    Unimplemented { what: String },
    EarlyReturn { value: Value },
    AuthDeclaredButUnwired { service: String, reason: String },
}

impl fmt::Display for InterpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpError::NoSuchFunction { name } => write!(f, "no such function: {}", name),
            InterpError::NoMainFunction => write!(f, "no main function found"),
            InterpError::NoSuchVariable { name } => write!(f, "undefined variable: {}", name),
            InterpError::NoSuchField { type_name, field } => {
                write!(f, "no field '{}' on type '{}'", field, type_name)
            }
            InterpError::TypeError { msg } => write!(f, "type error: {}", msg),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    pub modules: Rc<Vec<Rc<TypedModule>>>,
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
    mutation_counters: std::cell::RefCell<MutationCounters>,
    symbols: RefCell<SymbolInterner>,
    published_mock_keys: RefCell<Option<Rc<std::collections::HashSet<String>>>>,
    whole_tree_published_keys: Option<Rc<std::collections::HashSet<String>>>,
    governed_services: RefCell<Option<Rc<std::collections::HashSet<String>>>>,
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
            for item in module.items.iter() {
                let name = authored_name_at(source_indices.clone(), item.clone());
                if !name.is_empty() {
                    fn_nodes.insert(name.clone(), item.clone());
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
            mutation_counters: std::cell::RefCell::new(MutationCounters::default()),
            symbols: RefCell::new(SymbolInterner::default()),
            published_mock_keys: RefCell::new(None),
            whole_tree_published_keys,
            governed_services: RefCell::new(None),
        }
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

        call_function(ctx, &item_node, &[], &env)
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
        call_function(ctx, &item_node, args, &env)
    })
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

fn call_function(
    ctx: &InterpContext,
    fn_node: &Rc<Node>,
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
) -> InterpResult<Value> {
    let body = fn_node
        .body
        .as_ref()
        .ok_or_else(|| InterpError::TypeError {
            msg: format!("'{}' has no body", fn_node.name),
        })?;

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
                bindings.insert(ctx.sym(name), val.clone());
            } else if positional_idx < param_names.len() {
                bindings.insert(ctx.sym(&param_names[positional_idx]), val.clone());
                positional_idx += 1;
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

    let call_env = Env::extend(env, bindings);

    match eval_expr(body, &call_env, ctx) {
        Err(InterpError::EarlyReturn { value }) => Ok(value),
        other => other,
    }
}

fn eval_expr(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
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

        ExprData::ExprError { message, .. } => Err(InterpError::TypeError { msg: message }),

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
        return Ok(Value::Variant {
            type_name: ctx.sym(parent_enum),
            variant_name: sym,
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
        } => match value {
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
                    return None;
                }
                let mut bindings = HashMap::new();
                for fb in field_bindings.iter() {
                    let field_name = field_binding_name_at(fb.clone(), ctx.source_indices.clone());
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
                    let field_name = field_binding_name_at(fb.clone(), ctx.source_indices.clone());
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
                    let field_name = field_binding_name_at(fb.clone(), ctx.source_indices.clone());
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
        },
    }
}

pub(crate) const STD_NODE_BRIDGE_FNS: &[&str] = &["resolve_type_node"];

pub(crate) const STD_LEXING_BRIDGE_FNS: &[&str] = &["symbol_intern_lexeme"];

pub(crate) const STD_QUALIFIED_NAME_BRIDGE_FNS: &[&str] = &["qualified_name_from_dotted_string"];

pub(crate) const STD_NODE_QUERY_BRIDGE_FNS: &[&str] = &["coproduct_nullary_inhabitants"];

pub(crate) const STD_CONCEPT_INDEX_BRIDGE_FNS: &[&str] = &["concept_decl_facts_live"];

pub(crate) const STD_FN_INDEX_BRIDGE_FNS: &[&str] = &["fn_arrow_decl_facts_live"];

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
            _ => unreachable!("fn_index bridge fn set mismatch"),
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
    call_function(ctx, &fn_node, &args, env)
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
    let items = free_monoid_to_vec(xs).ok_or_else(|| InterpError::TypeError {
        msg: format!("fold_list expects a list, got {}", xs.type_label()),
    })?;
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

fn parse_table_memo_scope_and_key(
    ctx: &InterpContext,
    table: &Value,
    key: &Value,
) -> Option<(String, String, i64, Symbol)> {
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
            let mut st = ctx.parse_table_memo.borrow_mut();
            st.lookups += 1;
            if let Some(v) = st.map.get(&memo_key).cloned() {
                st.hits += 1;
                return Ok(Some(witness_holds(v, ctx)));
            }
            drop(st);
            let result = call_function(ctx, fn_node, args, env)?;
            Ok(Some(result))
        }
        "parse_table_insert" => {
            let positional: Vec<&Value> = args.iter().map(|(_, v)| v).collect();
            let [table, key, value] = match positional.as_slice() {
                [table, key, value] => [table, key, value],
                _ => return Ok(None),
            };
            if let Some(memo_key) = parse_table_memo_scope_and_key(ctx, table, key) {
                let mut st = ctx.parse_table_memo.borrow_mut();
                st.keepalive.push((*table).clone());
                st.keepalive.push((*key).clone());
                st.keepalive.push((*value).clone());
                st.map.insert(memo_key, (*value).clone());
                st.inserts += 1;
            }
            let result = call_function(ctx, fn_node, args, env)?;
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
        return raw_map_lookup(&receiver_val, key, env, ctx);
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
            variant_name: ctx.sym(&type_name),
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

fn cast_target_underlying_kernel(ctx: &InterpContext, target: Rc<Node>) -> String {
    let mut current = authored_name_at(ctx.si(), target);
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
    if cast_target_underlying_kernel(ctx, target) == "String" {
        Some(Value::Str(s.clone()))
    } else {
        None
    }
}

fn eval_cast(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let val = eval_expr(&cast_expr(node.clone()), env, ctx)?;
    let target_node = cast_target(node.clone());
    let target_name = authored_name_at(ctx.si(), target_node.clone());

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

        "length" | "count" | "size" => match free_monoid_to_vec(&receiver) {
            Some(items) => Ok(Value::Int(items.len() as i64)),
            None => match &receiver {
                Value::Map(m) => Ok(Value::Int(m.len() as i64)),
                _ => Err(InterpError::TypeError {
                    msg: format!("cannot get length of {}", receiver.type_label()),
                }),
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
                Ok(items.get(idx as usize).cloned().unwrap_or(Value::Null))
            } else {
                let key = args.first().ok_or_else(|| InterpError::TypeError {
                    msg: "get requires a key argument".to_string(),
                })?;
                raw_map_lookup(&receiver, key, env, ctx)
            }
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

fn eval_service_call(
    service_name: &str,
    op_name: &str,
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
    ctx: &InterpContext,
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
    let inputs_hash =
        crate::recorded_fixture::content_hash_service_inputs(op_node, &param_env, ctx);
    let inputs_json =
        crate::recorded_fixture::service_inputs_fixture_json(op_node, &param_env, ctx)
            .map_err(|e| InterpError::TypeError { msg: e.to_string() })?;

    if ctx.execution_mode.is_hermetic() {
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
        crate::module_path_index::extdeps_shape_transport_policy_census::shell_argv_nodes_for_operation(
            path, service, operation,
        );
    let mut argv: Vec<String> = Vec::new();
    for node in argv_nodes.iter() {
        let val = materialize_argv_expr_for_bindings(node, &param_bindings, &source_indices)?;
        push_shell_argv_tokens(&mut argv, val).map_err(|e| format!("{e:?}"))?;
    }
    Ok(argv)
}

/// SGR foreground parameters per `SemanticColor`, mirroring the
/// `extdeps.render.ansi` authority (`ansi_mappings` in `dsl/extdeps/render/ansi.dag`).
/// Seed realization until the interpreter consumes that table directly; the
/// dissolution is the single checkable receipt ROADMAP §1 "interpreter
/// terminal-output de-fork" (`dsl/gunbc/roadmap_authority.dag`).
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
/// authority (`dsl/gunbc/output_policy.dag`); resolution precedence mirrors that
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

    let output = std::process::Command::new(&argv[0])
        .args(&argv[1..])
        .output()
        .map_err(|e| InterpError::TypeError {
            msg: format!("failed to execute '{}': {}", argv[0], e),
        })?;

    Ok(ShellResult {
        exit_code: output.status.code().unwrap_or(-1),
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
        let value = match from_key.as_deref() {
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

fn eval_builtin(
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
            let arg = positional.first().copied().ok_or_else(|| InterpError::TypeError {
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

        "parse_int" => {
            let s = expect_str(positional.first().copied(), "parse_int")?;
            match s.parse::<i64>() {
                Ok(n) => Ok(Some(Value::Int(n))),
                Err(_) => Ok(Some(Value::Null)),
            }
        }

        "record_source_chars_index_lookup" => Ok(Some(Value::Unit)),

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

        "empty_set" => Ok(Some(Value::Set(Rc::new(BTreeSet::new())))),

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
            let out = v1_rt::contiguous_loop_elementwise_float_kernel(
                &op_codes,
                fma_policy,
                &a,
                &b,
                &c,
            );
            Ok(Some(list_value(
                out.into_iter().map(Value::Float).collect::<Vec<_>>(),
            )))
        }

        "layer_import_facts" => {
            let std_roots = expect_str_list(positional.first().copied(), "layer_import_facts")?;
            let extdeps_roots = expect_str_list(positional.get(1).copied(), "layer_import_facts")?;
            let facts =
                crate::cli_run::layer_import_facts(&std_roots, &extdeps_roots);
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

        "concept_decl_facts" => {
            let pool_roots = expect_str_list(positional.first().copied(), "concept_decl_facts")?;
            Ok(Some(crate::coproduct_reflection::eval_concept_decl_facts(
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

        "medium_structure_leak_facts" => {
            let emit_roots =
                expect_str_list_flex(positional.first().copied(), "medium_structure_leak_facts")?;
            let check_roots =
                expect_str_list_flex(positional.get(1).copied(), "medium_structure_leak_facts")?;
            let markers =
                expect_str_list_flex(positional.get(2).copied(), "medium_structure_leak_facts")?;
            let emit_fns =
                expect_str_list_flex(positional.get(3).copied(), "medium_structure_leak_facts")?;
            let string_ops =
                expect_str_list_flex(positional.get(4).copied(), "medium_structure_leak_facts")?;
            let facts = crate::module_path_index::medium_structure_census::medium_structure_leak_facts(
                &emit_roots,
                &check_roots,
                &markers,
                &emit_fns,
                &string_ops,
            );
            let mut items: Vec<Value> = Vec::new();
            for f in facts {
                let face = Value::Variant {
                    type_name: ctx.sym("MediumLeakFace"),
                    variant_name: ctx.sym(f.face),
                    fields: Rc::new(vec![]),
                };
                items.push(Value::Record {
                    type_name: ctx.sym("MediumStructureLeakFact"),
                    fields: Rc::new(sorted_fields(vec![
                        (ctx.sym("detail"), Value::Str(f.detail)),
                        (ctx.sym("face"), face),
                        (ctx.sym("path"), Value::Str(f.path)),
                    ])),
                });
            }
            Ok(Some(list_value(items)))
        }

        "fact_cardinality_cross_tree_coexistence_count" => Ok(Some(Value::Int(
            crate::module_path_index::fact_cardinality_census::cross_tree_coexistence_count(),
        ))),

        "fact_cardinality_cross_tree_diverged_fork_count" => Ok(Some(Value::Int(
            crate::module_path_index::fact_cardinality_census::cross_tree_diverged_fork_count(),
        ))),

        "fact_cardinality_cross_tree_is_coexistence" => {
            let key = expect_str(
                positional.first().copied(),
                "fact_cardinality_cross_tree_is_coexistence",
            )?;
            Ok(Some(Value::Bool(
                crate::module_path_index::fact_cardinality_census::cross_tree_is_coexistence(key),
            )))
        }

        "fact_cardinality_cross_tree_is_diverged_fork" => {
            let key = expect_str(
                positional.first().copied(),
                "fact_cardinality_cross_tree_is_diverged_fork",
            )?;
            Ok(Some(Value::Bool(
                crate::module_path_index::fact_cardinality_census::cross_tree_is_diverged_fork(key),
            )))
        }

        "languages_consumer_census_data_decl_count" => Ok(Some(Value::Int(
            crate::module_path_index::languages_consumer_census::languages_consumer_census_data_decl_count(),
        ))),

        "languages_consumer_census_per_language_row_count" => Ok(Some(Value::Int(
            crate::module_path_index::languages_consumer_census::languages_consumer_census_per_language_row_count(),
        ))),

        "languages_consumer_census_format_row_count" => Ok(Some(Value::Int(
            crate::module_path_index::languages_consumer_census::languages_consumer_census_format_row_count(),
        ))),

        "languages_consumer_census_external_consumer_count" => {
            let decl_name = expect_str(
                positional.first().copied(),
                "languages_consumer_census_external_consumer_count",
            )?;
            Ok(Some(Value::Int(
                crate::module_path_index::languages_consumer_census::languages_consumer_census_external_consumer_count(
                    decl_name,
                ),
            )))
        }

        "languages_consumer_census_is_composition_only" => {
            let decl_name = expect_str(
                positional.first().copied(),
                "languages_consumer_census_is_composition_only",
            )?;
            Ok(Some(Value::Bool(
                crate::module_path_index::languages_consumer_census::languages_consumer_census_is_composition_only(
                    decl_name,
                ),
            )))
        }

        "languages_consumer_census_has_external_consumer" => {
            let decl_name = expect_str(
                positional.first().copied(),
                "languages_consumer_census_has_external_consumer",
            )?;
            Ok(Some(Value::Bool(
                crate::module_path_index::languages_consumer_census::languages_consumer_census_has_external_consumer(
                    decl_name,
                ),
            )))
        }

        "extdeps_dead_param_count_for_operation" => {
            let path = expect_str(
                positional.first().copied(),
                "extdeps_dead_param_count_for_operation",
            )?;
            let service = expect_str(
                positional.get(1).copied(),
                "extdeps_dead_param_count_for_operation",
            )?;
            let operation = expect_str(
                positional.get(2).copied(),
                "extdeps_dead_param_count_for_operation",
            )?;
            let count =
                crate::module_path_index::extdeps_shape_transport_policy_census::dead_param_count_for_operation(
                    path, service, operation,
                );
            Ok(Some(Value::Int(count)))
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
            let mut param_bindings = HashMap::new();
            param_bindings.insert("package".to_string(), Value::Str(package));
            param_bindings.insert("bin".to_string(), Value::Str(bin));
            param_bindings.insert(
                "args".to_string(),
                list_value(extra_args.into_iter().map(Value::Str).collect::<Vec<_>>()),
            );
            let argv =
                materialize_shell_argv_for_operation(path, service, operation, param_bindings)
                    .map_err(|e| InterpError::TypeError { msg: e })?;
            Ok(Some(list_value(
                argv.into_iter().map(Value::Str).collect::<Vec<_>>(),
            )))
        }

        "extdeps_dead_param_count_for_path" => {
            let path = expect_str(
                positional.first().copied(),
                "extdeps_dead_param_count_for_path",
            )?;
            let count =
                crate::module_path_index::extdeps_shape_transport_policy_census::dead_param_count_for_path(path);
            Ok(Some(Value::Int(count)))
        }

        "extdeps_embedded_policy_literal_count_for_path" => {
            let path = expect_str(
                positional.first().copied(),
                "extdeps_embedded_policy_literal_count_for_path",
            )?;
            let count =
                crate::module_path_index::extdeps_shape_transport_policy_census::embedded_policy_literal_count_for_path(
                    path,
                );
            Ok(Some(Value::Int(count)))
        }

        "extdeps_qualified_name_resolves_in_derived_module_set" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg:
                    "extdeps_qualified_name_resolves_in_derived_module_set requires a QualifiedName"
                        .to_string(),
            })?;
            Ok(Some(Value::Bool(
                crate::module_path_index::extdeps_shape_transport_policy_census::qualified_name_resolves_in_derived_module_set(
                    module,
                ),
            )))
        }

        "extdeps_dead_param_count_for_qualified_name" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_dead_param_count_for_qualified_name requires module, service, operation"
                    .to_string(),
            })?;
            let service = expect_str(
                positional.get(1).copied(),
                "extdeps_dead_param_count_for_qualified_name",
            )?;
            let operation = expect_str(
                positional.get(2).copied(),
                "extdeps_dead_param_count_for_qualified_name",
            )?;
            let count =
                crate::module_path_index::extdeps_shape_transport_policy_census::dead_param_count_for_qualified_name(
                    module, service, operation,
                );
            Ok(Some(Value::Int(count)))
        }

        "transport_script_literal_violation_count_for_path" => {
            let path = expect_str(
                positional.first().copied(),
                "transport_script_literal_violation_count_for_path",
            )?;
            let count =
                crate::module_path_index::transport_script_position_census::transport_script_literal_violation_count_for_path(
                    path,
                );
            Ok(Some(Value::Int(count)))
        }

        "extdeps_embedded_policy_literal_count_for_qualified_name" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_embedded_policy_literal_count_for_qualified_name requires a QualifiedName"
                    .to_string(),
            })?;
            let count = crate::module_path_index::extdeps_shape_transport_policy_census::embedded_policy_literal_count_for_qualified_name(
                module,
            );
            Ok(Some(Value::Int(count)))
        }

        "module_source_nickname_literal_count_for_qualified_name" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "module_source_nickname_literal_count_for_qualified_name requires a QualifiedName"
                    .to_string(),
            })?;
            let count =
                crate::module_path_index::extdeps_shape_transport_policy_census::module_source_nickname_literal_count_for_qualified_name(
                    module,
                );
            Ok(Some(Value::Int(count)))
        }

        "extdeps_policy_leak_count_for_qualified_name" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_policy_leak_count_for_qualified_name requires a QualifiedName"
                    .to_string(),
            })?;
            let count =
                crate::module_path_index::extdeps_shape_transport_policy_census::policy_leak_count_for_qualified_name(
                    module,
                );
            Ok(Some(Value::Int(count)))
        }

        "extdeps_transport_fusion_fork_count_for_qualified_name" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_transport_fusion_fork_count_for_qualified_name requires a QualifiedName"
                    .to_string(),
            })?;
            let count = crate::module_path_index::extdeps_shape_transport_policy_census::transport_fusion_fork_count_for_qualified_name(
                module,
            );
            Ok(Some(Value::Int(count)))
        }

        "extdeps_gist_create_declares_filename_input_for_qualified_name" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_gist_create_declares_filename_input_for_qualified_name requires a QualifiedName"
                    .to_string(),
            })?;
            Ok(Some(Value::Bool(
                crate::module_path_index::extdeps_shape_transport_policy_census::gist_create_declares_filename_input_for_qualified_name(
                    module,
                ),
            )))
        }

        "extdeps_gist_create_files_keyed_by_filename_for_qualified_name" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_gist_create_files_keyed_by_filename_for_qualified_name requires a QualifiedName"
                    .to_string(),
            })?;
            Ok(Some(Value::Bool(
                crate::module_path_index::extdeps_shape_transport_policy_census::gist_create_files_keyed_by_filename_placeholder_for_qualified_name(
                    module,
                ),
            )))
        }

        "extdeps_external_authority_anchor_kind_for_qualified_name" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_external_authority_anchor_kind_for_qualified_name requires a QualifiedName"
                    .to_string(),
            })?;
            Ok(Some(Value::Str(
                crate::module_path_index::extdeps_shape_transport_policy_census::external_authority_anchor_kind_for_qualified_name(
                    module,
                ),
            )))
        }

        "extdeps_external_authority_scheme_identity_for_qualified_name" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_external_authority_scheme_identity_for_qualified_name requires a QualifiedName"
                    .to_string(),
            })?;
            Ok(Some(Value::Str(
                crate::module_path_index::extdeps_shape_transport_policy_census::external_authority_scheme_identity_for_qualified_name(
                    module,
                ),
            )))
        }

        "extdeps_external_authority_locator_for_qualified_name" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg:
                    "extdeps_external_authority_locator_for_qualified_name requires a QualifiedName"
                        .to_string(),
            })?;
            Ok(Some(Value::Str(
                crate::module_path_index::extdeps_shape_transport_policy_census::external_authority_locator_for_qualified_name(
                    module,
                ),
            )))
        }

        "extdeps_derived_extdeps_modules" => {
            let ctx = active_ctx().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_derived_extdeps_modules requires an active interpreter context"
                    .to_string(),
            })?;
            Ok(Some(
                crate::module_path_index::extdeps_shape_transport_policy_census::derived_extdeps_modules_value(ctx),
            ))
        }

        "extdeps_external_authority_backfill_entries" => {
            let ctx = active_ctx().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_external_authority_backfill_entries requires an active interpreter context"
                    .to_string(),
            })?;
            Ok(Some(
                crate::module_path_index::extdeps_shape_transport_policy_census::backfill_pending_entries_value(ctx),
            ))
        }

        "extdeps_external_authority_is_backfill_pending_for_qualified_name" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_external_authority_is_backfill_pending_for_qualified_name requires a QualifiedName"
                    .to_string(),
            })?;
            Ok(Some(Value::Bool(
                crate::module_path_index::extdeps_shape_transport_policy_census::is_backfill_pending_for_qualified_name(
                    module,
                ),
            )))
        }
        "extdeps_external_authority_is_machinery_exempt_for_qualified_name" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_external_authority_is_machinery_exempt_for_qualified_name requires a QualifiedName"
                    .to_string(),
            })?;
            Ok(Some(Value::Bool(
                crate::module_path_index::extdeps_shape_transport_policy_census::is_machinery_exempt_for_qualified_name(
                    module,
                ),
            )))
        }
        "extdeps_external_authority_is_clean_tree_roster_excluded_for_qualified_name" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_external_authority_is_clean_tree_roster_excluded_for_qualified_name requires a QualifiedName"
                    .to_string(),
            })?;
            Ok(Some(Value::Bool(
                crate::module_path_index::extdeps_shape_transport_policy_census::is_clean_tree_roster_excluded_for_qualified_name(
                    module,
                ),
            )))
        }
        "extdeps_external_authority_live_clean_tree_holds" => Ok(Some(Value::Bool(
            crate::module_path_index::extdeps_shape_transport_policy_census::external_authority_live_clean_tree_holds(),
        ))),
        "extdeps_external_authority_anchor_shadow_masked_for_qualified_name" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_external_authority_anchor_shadow_masked_for_qualified_name requires a QualifiedName"
                    .to_string(),
            })?;
            Ok(Some(Value::Bool(
                crate::module_path_index::extdeps_shape_transport_policy_census::external_authority_anchor_shadow_masked_for_qualified_name(
                    module,
                ),
            )))
        }
        "extdeps_external_authority_live_shadow_mask_holds" => Ok(Some(Value::Bool(
            crate::module_path_index::extdeps_shape_transport_policy_census::external_authority_live_shadow_mask_holds(),
        ))),
        "extdeps_external_authority_live_roster_module_count" => Ok(Some(Value::Int(
            crate::module_path_index::extdeps_shape_transport_policy_census::external_authority_live_roster_module_count(),
        ))),

        "doc_graph_orphan_count" => Ok(Some(Value::Int(
            crate::cli_run::doc_graph_orphan_count(),
        ))),
        "doc_graph_dangling_link_count" => Ok(Some(Value::Int(
            crate::cli_run::doc_graph_dangling_link_count(),
        ))),
        "doc_graph_doc_count" => Ok(Some(Value::Int(
            crate::cli_run::doc_graph_doc_count(),
        ))),

<<<<<<< HEAD
        "inert_carrier_names_live" => {
            let names = crate::cli_run::inert_carrier_names_live();
            let items: Vec<Value> = names.into_iter().map(Value::Str).collect();
            Ok(Some(list_value(items)))
        }
        "inert_carrier_declared_count" => Ok(Some(Value::Int(
            crate::cli_run::inert_carrier_declared_count_live(),
=======
        "inert_carrier_count" => Ok(Some(Value::Int(
            crate::module_path_index::inert_carrier_census::inert_carrier_count(),
        ))),
        "inert_carrier_unrostered_count" => Ok(Some(Value::Int(
            crate::module_path_index::inert_carrier_census::inert_carrier_unrostered_count(),
        ))),
        "inert_carrier_stale_roster_count" => Ok(Some(Value::Int(
            crate::module_path_index::inert_carrier_census::inert_carrier_stale_roster_count(),
        ))),
        "inert_carrier_declared_count" => Ok(Some(Value::Int(
            crate::module_path_index::inert_carrier_census::inert_carrier_declared_count(),
>>>>>>> origin/main
        ))),

        "inert_lens_unreached_module_count" => Ok(Some(Value::Int(
            crate::cli_run::inert_lens_unreached_module_count(),
        ))),
        "inert_lens_top_level_module_count" => Ok(Some(Value::Int(
            crate::cli_run::inert_lens_top_level_module_count(),
        ))),

        "non_fold_residue_count" => Ok(Some(Value::Int(
            crate::module_path_index::non_fold_residue_census::non_fold_residue_count(),
        ))),
        "non_fold_residue_unrostered_count" => Ok(Some(Value::Int(
            crate::module_path_index::non_fold_residue_census::non_fold_residue_unrostered_count(),
        ))),
        "non_fold_residue_stale_roster_count" => Ok(Some(Value::Int(
            crate::module_path_index::non_fold_residue_census::non_fold_residue_stale_roster_count(),
        ))),
        "non_fold_residue_coproduct_universe_count" => Ok(Some(Value::Int(
            crate::module_path_index::non_fold_residue_census::non_fold_residue_coproduct_universe_count(),
        ))),

        "complexity_linearity_syntactic_finding_count" => Ok(Some(Value::Int(
            crate::complexity_linearity_audit_project::complexity_linearity_syntactic_finding_count(
            ),
        ))),
        "complexity_linearity_syntactic_wildcard_finding_count" => Ok(Some(Value::Int(
            crate::complexity_linearity_audit_project::complexity_linearity_syntactic_wildcard_finding_count(
            ),
        ))),
        "complexity_linearity_syntactic_site_fired" => {
            let site = expect_str(
                positional.first().copied(),
                "complexity_linearity_syntactic_site_fired",
            )?;
            Ok(Some(Value::Bool(
                crate::complexity_linearity_audit_project::complexity_linearity_syntactic_site_fired(
                    &site,
                ),
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

pub fn flatten_counters_snapshot() -> (u64, u64) {
    FLATTEN_COUNTERS.with(|c| c.get())
}

fn record_flatten(items: usize) {
    FLATTEN_COUNTERS.with(|c| {
        let (calls, total) = c.get();
        c.set((calls + 1, total + items as u64));
    });
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

pub(crate) fn free_monoid_to_vec(val: &Value) -> Option<Vec<Value>> {
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
                return Some(out);
            }
            Value::Str(s) => {
                out.extend(s.chars().map(char_value));
                record_flatten(out.len());
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
