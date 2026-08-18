use crate::v1_rt::VecCompat;
use im::HashMap;
use std::cell::{Cell, RefCell};
use std::collections::BTreeSet;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use im::HashMap as HamtMap;
use im::OrdSet;
use im::Vector as RrbVector;

use crate::cli_run::value_to_wire_json;
use crate::std_syntax::BinOp;
use crate::std_syntax::LiteralValue;
use crate::v1_compiler_emit::{extract_string_interp_parts, has_mock_prefix};
use crate::v1_compiler_infer_emit_info::EmitGraphInfo;
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
    method_receiver, param_node_default_value, param_node_name_at, qualified_last_segment,
    record_lit_type_name_at, return_value, slice_base, slice_end, slice_start, transport_stdin,
    unaryop_operand, CallSemantics, Cardinality, Connective, ErrorNode, ExprData, FieldAccessStyle,
    FieldSummary, FieldValueShape, InferredNode, MatchPattern, MethodSemantics, NewlineIndex, Node,
    SourceSpan, StringPart, UnaryOpKind, VarBindingKind,
};

#[path = "bounded_shell_host_drain.rs"]
pub mod bounded_shell_host_drain;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Symbol(u32);

#[derive(Debug, Default)]
pub struct SymbolInterner {
    strings: Vec<String>,
    index: HashMap<String, u32>,
    calls: u64,
}

#[cfg(test)]
mod selected_identity_path_tests {
    use super::{ExecutionMode, InterpContext};
    use crate::v1_compiler_compile::SourceFile;
    use im::HashMap;
    use std::rc::Rc;

    #[test]
    fn selected_function_identity_refuses_suffix_collision_on_actual_node() {
        let result =
            crate::v1_compiler_compile::compile_to_resolved(Rc::new(im::vector![Rc::new(
                SourceFile {
                    path: "workspace/src/common.dag".to_string(),
                    content: "module fixture.common\nfn check() -> Bool { true }\n".to_string(),
                },
            )]));
        let graph = result.graph.as_ref().expect("fixture graph");
        let ctx = InterpContext::new(
            graph,
            result.source_indices.clone(),
            ExecutionMode::Hermetic,
        );
        let mut index = HashMap::new();
        index.insert("one".to_string(), "src/common.dag".to_string());
        index.insert("two".to_string(), "common.dag".to_string());
        assert_eq!(ctx.selected_function_identity("check", &index), None);
    }
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

/// Default-off: when `interp_test_witness` is compiled in, the hook is a guarded no-op (one
/// relaxed atomic load per `eval_call`) until a test arms it.
#[cfg(any(test, feature = "interp_test_witness"))]
static CALL_ENV_DEPTH_WITNESS_ARMED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(any(test, feature = "interp_test_witness"))]
fn call_env_depth_witness_enabled() -> bool {
    CALL_ENV_DEPTH_WITNESS_ARMED.load(std::sync::atomic::Ordering::Relaxed)
}

#[cfg(any(test, feature = "interp_test_witness"))]
pub fn arm_call_env_depth_witness_for_test() {
    CALL_ENV_DEPTH_WITNESS_ARMED.store(true, std::sync::atomic::Ordering::Relaxed);
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
        if call_env_depth_witness_enabled() {
            CALL_ENV_DEPTH_PEAK.with(|peak| peak.set(0));
        }
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
    if !call_env_depth_witness_enabled() {
        return;
    }
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

fn coproduct_arm_name_matches(value_name: String, pattern_name: String) -> bool {
    qualified_last_segment(value_name.clone()) == qualified_last_segment(pattern_name)
}

fn resolve_coproduct_type_node(ctx: &InterpContext, parent_enum: &str) -> Option<Rc<Node>> {
    lookup_type_item_across_modules(ctx, parent_enum).or_else(|| {
        lookup_type_item_across_modules(ctx, &qualified_last_segment(parent_enum.to_string()))
    })
}

fn coproduct_parent_spellings_match(
    ctx: &InterpContext,
    value_parent: String,
    pattern_parent: &str,
) -> bool {
    if value_parent == pattern_parent {
        return true;
    }
    let coproduct = resolve_coproduct_type_node(ctx, pattern_parent);
    match coproduct {
        Some(coproduct_node) => authored_name_at(ctx.si(), coproduct_node.clone()) == value_parent,
        None => false,
    }
}

fn record_nominal_is_declared_variant_of_coproduct(
    ctx: &InterpContext,
    record_nominal: String,
    pattern_parent: &str,
) -> bool {
    let coproduct = match resolve_coproduct_type_node(ctx, pattern_parent) {
        Some(node) => node,
        None => return false,
    };
    if coproduct.connective != Connective::Disj {
        return false;
    }
    for child in coproduct.children.iter() {
        if authored_name_at(ctx.si(), child.clone()) == record_nominal {
            return true;
        }
    }
    false
}

fn record_pattern_type_name_matches(
    ctx: &InterpContext,
    record_type_name: Symbol,
    pattern_name: &str,
    parent_enum: Option<&String>,
) -> bool {
    let resolved = resolve_sym(record_type_name);
    if record_type_name == ctx.sym(pattern_name) || resolved == pattern_name {
        return true;
    }
    match parent_enum {
        Some(parent) => record_nominal_is_declared_variant_of_coproduct(ctx, resolved, parent),
        None => false,
    }
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
                        Value::Str(s) => Some(s.to_string()),
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
                                        Value::Str(s) => Some(s.to_string()),
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
    Str(Rc<str>),
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

pub fn str_value(s: impl AsRef<str>) -> Value {
    Value::Str(Rc::from(s.as_ref()))
}

/// Project an observed child-process status onto `std.process_termination` `ProcessTermination`.
///
/// A signalled process has no exit code, so it gets the signal arm rather than a
/// fabricated integer: the seed used to render `.code().unwrap_or(-1)` for both, which
/// made a runner OOM-kill indistinguishable from a process that chose to exit -1.
/// `ProcessTerminationUnobserved` is unreachable from an `ExitStatus` (having one means
/// the process ran); it is the arm a caller supplies when the spawn itself refused.
pub(crate) fn process_termination_value(
    status: &std::process::ExitStatus,
    ctx: &InterpContext,
) -> Value {
    let termination = |variant: &str, field: &str, value: i64| Value::Variant {
        type_name: ctx.sym("ProcessTermination"),
        variant_name: ctx.sym(variant),
        fields: Rc::new(vec![(ctx.sym(field), Value::Int(value))]),
    };
    if let Some(code) = status.code() {
        return termination("ProcessExited", "code", i64::from(code));
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return termination("ProcessSignaled", "signal", i64::from(signal));
        }
    }
    Value::Variant {
        type_name: ctx.sym("ProcessTermination"),
        variant_name: ctx.sym("ProcessTerminationUnobserved"),
        fields: Rc::new(Vec::new()),
    }
}

/// Human-facing rendering of a termination for a build log line. Kept beside the
/// projection above so the two spellings of one observation cannot drift.
pub(crate) fn process_termination_label(status: &std::process::ExitStatus) -> String {
    if let Some(code) = status.code() {
        return format!("exit {code}");
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return format!("signal {signal}");
        }
    }
    "termination unobserved".to_string()
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

/// Whether a `raw_map_lookup` result already carries the `Optional<V>` contract
/// (a `.dag`-authored `Map.lookup` closure returns `Optional<V>` by construction)
/// or is a bare storage read that still needs the `Optional` wrap applied
/// (native `Value::Map`/field storage, where a miss is `Value::Null`).
/// Distinguishing by the call site — not by sniffing the value's shape — is
/// required so a stored `V = Optional<T>` payload isn't mistaken for an
/// already-wrapped lookup result (DESIGN §5: construction, not validation).
enum RawMapLookup {
    AlreadyOptional(Value),
    NeedsWrap(Value),
}

fn map_lookup_as_optional(raw: RawMapLookup, ctx: &InterpContext) -> Value {
    match raw {
        RawMapLookup::AlreadyOptional(v) => v,
        RawMapLookup::NeedsWrap(Value::Null) => optional_absent(ctx),
        RawMapLookup::NeedsWrap(v) => optional_present(v, ctx),
    }
}

impl RawMapLookup {
    /// Bare-value callers (field access, `[]` indexing, the raw `lookup`/`get`
    /// builtins) don't observe the `Optional` contract at all — they want
    /// exactly what was found, miss-as-`Null` included.
    fn into_raw(self) -> Value {
        match self {
            RawMapLookup::AlreadyOptional(v) | RawMapLookup::NeedsWrap(v) => v,
        }
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
    ServiceConfigUnresolved {
        key: String,
        spelled: String,
    },
    ServiceConfigMissing {
        key: String,
        service: String,
    },
    /// The caller-agnostic evaluation-budget result. This is what the kernel RAISES; the two
    /// witness-named variants below are domain refusals that the witness lane maps this into at
    /// its own boundary, never things `eval_expr` produces directly.
    ///
    /// The split exists because the kernel had no neutral result to raise: a served HTTP request
    /// is not a witness, so raising `EvalBudgetExceeded` at a serve route would carry the
    /// fast-lane witness ruling — including its "relocating the file does not discharge it"
    /// guidance — into an HTTP 5xx body, which is the DESIGN §3 nickname failure applied to a
    /// diagnostic (operator review 2026-08-09). `entry` names which evaluation crossed, `clock`
    /// names which bound fired (a consumer that cannot tell CPU from wall cannot tell a spin
    /// from a stall, and those have different remedies), and `elapsed_nanos` is nanoseconds
    /// rather than floored milliseconds per `std.measure`'s `nanosecond_millisecond_projection_note`:
    /// a declared limit is policy and is milliseconds, an observed crossing is a measurement and
    /// is never floored on its way into the carrier.
    EvaluationBudgetExceeded {
        entry: String,
        clock: EvaluationClock,
        elapsed_nanos: u128,
        limit_ms: u64,
    },
    /// The fast-lane per-witness eval budget, enforced on THREAD CPU by the cooperative
    /// stride-poll in `eval_expr`. The measured field is named for its clock deliberately:
    /// this budget and the wall-clock one below are different quantities of the same
    /// occurrence, and a shared `elapsed_ms` spelling let a CPU figure be read as wall at
    /// every downstream consumer (2026-08-05 — the same conflation that leaves the enforced
    /// quantity absent from every cost receipt; see `witness_cost_clock_basis_note`).
    EvalBudgetExceeded {
        cpu_ms: u64,
        budget_ms: u64,
    },
    /// The whole-receipt wall budget (falsifier wet/silent-pick lane). Genuinely wall:
    /// emit+cargo subprocess I/O counts against it, which is why it cannot share the CPU
    /// carrier above.
    WitnessWallBudgetExceeded {
        wall_ms: u64,
        budget_ms: u64,
    },
    ArgvExceedsHostArgMax {
        actual_bytes: usize,
        limit_bytes: usize,
        argv0: String,
    },
    /// A host-tool program could not be resolved to an existing executable path.
    /// `probed` carries every candidate location examined so the refusal is located
    /// and countable by class rather than by grepping a format string.
    HostToolUnresolved {
        name: String,
        probed: Vec<String>,
    },
    /// A slash-containing tool name that is not an absolute path. Refused because
    /// `is_file()` is cwd-relative while emit-host spawns set `.current_dir(workspace)`,
    /// so a relative path would mean different things at check vs spawn time.
    HostToolRelativePathAmbiguous {
        name: String,
    },
    /// Shell child stdout exceeded the seed complete-within bound — never surfaced as a prefix.
    ShellOutputLimitExceeded {
        stream: &'static str,
        total_bytes: u64,
        limit_bytes: u64,
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
            InterpError::NoSuchVariable { name } => write!(f, "undefined variable: {}", name),
            InterpError::NoSuchField { type_name, field } => {
                write!(f, "no field '{}' on type '{}'", field, type_name)
            }
            InterpError::TypeError { msg } => write!(f, "type error: {}", msg),
            InterpError::EvaluationBudgetExceeded {
                entry,
                clock,
                elapsed_nanos,
                limit_ms,
            } => {
                write!(
                    f,
                    "evaluation budget exceeded: entry={} clock={} elapsed_ns={} limit_ms={}",
                    entry,
                    clock.key(),
                    elapsed_nanos,
                    limit_ms
                )
            }
            InterpError::EvalBudgetExceeded {
                cpu_ms: elapsed_ms,
                budget_ms,
            } => {
                write!(
                    f,
                    "eval budget exceeded: {}ms thread-CPU > {}ms fast-lane budget (operator 5s rule 2026-07-12). This budget is enforced on THREAD CPU, not wall. RELOCATING THE FILE DOES NOT DISCHARGE IT: moving a witness under a long/ dir removes it from per-PR discovery without giving it an executing consumer, which deletes the coverage while retaining the source (the gunbc#7762 specimen behind the 2026-08-04 admission ruling). Either reduce the witness's cost, or enroll it in a lane that declares its own dated ceiling AND names the row as an executing consumer.",
                    elapsed_ms, budget_ms
                )
            }
            InterpError::WitnessWallBudgetExceeded {
                wall_ms: elapsed_ms,
                budget_ms,
            } => {
                write!(
                    f,
                    "witness receipt wall budget exceeded: {}ms wall > {}ms whole-receipt budget (falsifier wet/silent-pick lane; kill-at-deadline)",
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
            InterpError::ServiceConfigUnresolved { key, spelled } => write!(
                f,
                "service config '{}' did not resolve to a value (spelled '{}') — refusing to send a request against an unresolved endpoint",
                key, spelled
            ),
            InterpError::ServiceConfigMissing { key, service } => write!(
                f,
                "service '{}' declares no '{}' in its config — refusing to send a request against an empty base",
                service, key
            ),
            InterpError::ArgvExceedsHostArgMax {
                actual_bytes,
                limit_bytes,
                argv0,
            } => write!(
                f,
                "argv exceeds host arg limit: '{}' invocation carries a {}-byte argument > {}-byte host MAX_ARG_STRLEN — route large payloads through stdin, not argv (Linux execve(2) E2BIG; extdeps.exec.exec_arg_limit.host_exec_arg_max_strlen; DESIGN §5 typed refusal in place of an opaque os error 7)",
                argv0, actual_bytes, limit_bytes
            ),
            InterpError::HostToolUnresolved { name, probed } => write!(
                f,
                "host tool unresolved: {:?} (probed: {})",
                name,
                probed.join(", ")
            ),
            InterpError::HostToolRelativePathAmbiguous { name } => write!(
                f,
                "host tool relative path ambiguous at cwd-dependent boundary: {:?}",
                name
            ),
            InterpError::ShellOutputLimitExceeded {
                stream,
                total_bytes,
                limit_bytes,
                argv0,
            } => write!(
                f,
                "shell {} exceeded {} byte limit (total={} bytes) for '{}'",
                stream, limit_bytes, total_bytes, argv0
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
struct PrepareGrammarCrossClaimMemo {
    map: HashMap<(usize, u64), Value>,
}

thread_local! {
    static PREPARE_GRAMMAR_CROSS_CLAIM_MEMO: RefCell<PrepareGrammarCrossClaimMemo> =
        RefCell::new(PrepareGrammarCrossClaimMemo::default());
    static ZERO_ARG_PURE_CROSS_CLAIM_MEMO: RefCell<HashMap<usize, Value>> =
        RefCell::new(HashMap::new());
    static CROSS_CLAIM_FN_KEEPALIVE: RefCell<Vec<Rc<Node>>> = RefCell::new(Vec::new());
}

pub fn clear_cross_claim_pure_memos() {
    PREPARE_GRAMMAR_CROSS_CLAIM_MEMO
        .with(|m| *m.borrow_mut() = PrepareGrammarCrossClaimMemo::default());
    ZERO_ARG_PURE_CROSS_CLAIM_MEMO.with(|m| m.borrow_mut().clear());
    CROSS_CLAIM_FN_KEEPALIVE.with(|k| k.borrow_mut().clear());
}

fn keep_cross_claim_fn(fn_node: &Rc<Node>) {
    CROSS_CLAIM_FN_KEEPALIVE.with(|k| {
        let mut keepalive = k.borrow_mut();
        let ptr = Rc::as_ptr(fn_node) as usize;
        if !keepalive.iter().any(|n| Rc::as_ptr(n) as usize == ptr) {
            keepalive.push(fn_node.clone());
        }
    });
}

fn try_cross_claim_pure_memo(
    ctx: &InterpContext,
    fn_node: &Rc<Node>,
    func_name: &str,
    args: &[(Option<String>, Value)],
) -> Option<Value> {
    // 🟡 dissolve-on: gunbc.roadmap_authority five_minute_ci_gate_program_note — a generic
    // *cross-claim* pure memo keyed on fn-node identity + content-hashable args. `eval_call_memo`
    // cannot be that authority: its eviction scope is the witness frame
    // (`eval_call_memo_frame_exit`), so it cannot amortize the same pure call across the floor
    // fold. These two name arms exist only because that lifetime gap does; a third arm is
    // evidence the generic memo has not landed, not a reason to grow the list.
    if func_name == "prepare_grammar" && args.len() == 1 {
        let mut hash_memo = ctx.eval_recompute_hash_memo.borrow_mut();
        let key = eval_recompute_arg_key(&mut hash_memo, &args[0].1)?;
        let content_hash = match key {
            EvalRecomputeArgKey::ContentHash(h) => h,
            _ => return None,
        };
        let memo_key = (Rc::as_ptr(fn_node) as usize, content_hash);
        return PREPARE_GRAMMAR_CROSS_CLAIM_MEMO.with(|m| m.borrow().map.get(&memo_key).cloned());
    }
    if args.is_empty() && func_name == "ci_heal_binary_source_skew_guard_script" {
        let ptr = Rc::as_ptr(fn_node) as usize;
        return ZERO_ARG_PURE_CROSS_CLAIM_MEMO.with(|m| m.borrow().get(&ptr).cloned());
    }
    None
}

fn store_cross_claim_pure_memo(
    ctx: &InterpContext,
    fn_node: &Rc<Node>,
    func_name: &str,
    args: &[(Option<String>, Value)],
    result: &Value,
) {
    if func_name == "prepare_grammar" && args.len() == 1 {
        let mut hash_memo = ctx.eval_recompute_hash_memo.borrow_mut();
        if let Some(key) = eval_recompute_arg_key(&mut hash_memo, &args[0].1) {
            if let EvalRecomputeArgKey::ContentHash(h) = key {
                keep_cross_claim_fn(fn_node);
                PREPARE_GRAMMAR_CROSS_CLAIM_MEMO.with(|m| {
                    m.borrow_mut()
                        .map
                        .insert((Rc::as_ptr(fn_node) as usize, h), result.clone())
                });
            }
        }
        return;
    }
    if args.is_empty() && func_name == "ci_heal_binary_source_skew_guard_script" {
        keep_cross_claim_fn(fn_node);
        ZERO_ARG_PURE_CROSS_CLAIM_MEMO.with(|m| {
            m.borrow_mut()
                .insert(Rc::as_ptr(fn_node) as usize, result.clone());
        });
    }
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

static EVAL_RECOMPUTE_TRACE_CACHED: std::sync::atomic::AtomicU8 =
    std::sync::atomic::AtomicU8::new(2);

fn eval_recompute_trace_read_env() -> bool {
    std::env::var("GUNBC_RECOMPUTE_TRACE").is_ok_and(|v| v != "0")
}

fn eval_recompute_trace_refresh_cache() {
    EVAL_RECOMPUTE_TRACE_CACHED.store(
        u8::from(eval_recompute_trace_read_env()),
        std::sync::atomic::Ordering::SeqCst,
    );
}

pub fn eval_recompute_trace_enabled() -> bool {
    match EVAL_RECOMPUTE_TRACE_CACHED.load(std::sync::atomic::Ordering::SeqCst) {
        1 => true,
        0 => false,
        _ => {
            eval_recompute_trace_refresh_cache();
            eval_recompute_trace_enabled()
        }
    }
}

/// Test harness only: re-read `GUNBC_RECOMPUTE_TRACE` into the process-wide
/// cache. Needed because the production cache is initialized once per process;
/// claim_executor's parallel tests set the env var after siblings may have
/// latched tracing off (review 45756).
#[doc(hidden)]
pub fn refresh_eval_recompute_trace_enabled_cache_for_tests() {
    eval_recompute_trace_refresh_cache();
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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

/// THE IMMUTABLE HALF OF AN EVALUATION CONTEXT — built once per distinct scope, shared by
/// every claim that scope serves.
///
/// These indexes are a pure function of the module population: given the same modules they are
/// the same maps. Building them belongs to preparing a scope, not to running a claim.
///
/// The split exists because the obvious reading of "a fresh context per claim, so witnesses
/// cannot contaminate each other" rebuilds ALL of this per claim. On the required floor that is
/// 9,573 reconstructions of maps that only 1,155 distinct scopes can possibly differ in — the
/// entry-major cost shape reproduced one layer below the compiler, after the compiler's own
/// copy of it was removed. Fresh state per claim is correct; fresh INDEXES per claim is the
/// same defect wearing the word "fresh".
pub struct PreparedScopeIndexes {
    pub modules: Rc<im::Vector<Rc<TypedModule>>>,
    pub item_registry: Rc<HashMap<String, Rc<ItemInfo>>>,
    pub source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    pub emit_graph_info: Rc<EmitGraphInfo>,
    fn_nodes: HashMap<String, Rc<Node>>,
    ambiguous_bare_function_names: std::collections::HashSet<String>,
    service_ops: HashMap<String, ServiceOp>,
}

thread_local! {
    /// How many times the immutable index set has been constructed. The acceptance bar asks
    /// for `full interpreter index constructions <= distinct prepared scope identities`, and a
    /// bound nothing counts is a bound nobody can check — this is the counter that makes the
    /// per-claim rebuild observable instead of inferable from a profile.
    static SCOPE_INDEX_CONSTRUCTIONS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

pub fn scope_index_construction_count() -> u64 {
    SCOPE_INDEX_CONSTRUCTIONS.with(|c| c.get())
}

pub fn reset_scope_index_construction_count() {
    SCOPE_INDEX_CONSTRUCTIONS.with(|c| c.set(0));
}

pub struct InterpContext {
    /// The heavy maps, SHARED across every claim this scope serves. The four `Rc` handles
    /// below are cloned per frame because cloning a handle is free; these three are behind one
    /// handle because BUILDING them is not.
    indexes: Rc<PreparedScopeIndexes>,
    pub modules: Rc<im::Vector<Rc<TypedModule>>>,
    pub item_registry: Rc<HashMap<String, Rc<ItemInfo>>>,
    pub source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    pub emit_graph_info: Rc<EmitGraphInfo>,
    pub execution_mode: ExecutionMode,
    pub fixture_store: Option<Rc<crate::recorded_fixture::RecordedFixtureStore>>,
    data_cache: std::cell::RefCell<HashMap<usize, Value>>,
    // Per-call parameter-name derivation is invariant per fn_node but was re-sliced from
    // source spans on every call (authored_name_at). Memoize it per fn_node, keyed by fn_node
    // pointer identity. The key alone is not sound: the ctx does not own fn_nodes (they are
    // borrowed `Rc<Node>`s that can be dropped while the ctx lives), so a freed node's address
    // can be reused by an unrelated later node and silently collide on this key. keepalive_fns
    // retains the `Rc<Node>` behind each new key for the ctx's lifetime (same discipline as
    // PureCallMemo.keepalive_fns / EvalRecomputeTrace.keepalive_fns / EvalCallMemo.keepalive_fns),
    // making the pointer stable and the cache dies with the ctx (same discipline as data_cache).
    // Value = (filtered named-param list, all-param list), matching call_function's two uses.
    param_name_cache: std::cell::RefCell<HashMap<usize, Rc<(Vec<String>, Vec<String>)>>>,
    param_name_cache_keepalive: std::cell::RefCell<Vec<Rc<Node>>>,
    // Same chokepoint, ExprVar arm: eval_var rebuilt the variable name String from its source
    // span (expr_var_name_at) and re-interned it (ctx.sym) on every read. Memoize the interned
    // Symbol per ExprVar node — keyed by node pointer, kept alive via var_sym_cache_keepalive
    // exactly as param_name_cache above. Eval then skips the slice + re-intern and goes straight
    // to env.lookup(sym); the name String is materialized lazily only on the registry slow path.
    var_sym_cache: std::cell::RefCell<HashMap<usize, Symbol>>,
    var_sym_cache_keepalive: std::cell::RefCell<Vec<Rc<Node>>>,
    // Same chokepoint, ExprCall callee name: eval_call re-sliced the callee name from its source
    // span (expr_call_func_at -> authored_name_at) on every call. Memoize the decoded name per
    // call node — keyed by node pointer, kept alive via call_func_name_cache_keepalive as above.
    call_func_name_cache: std::cell::RefCell<HashMap<usize, String>>,
    call_func_name_cache_keepalive: std::cell::RefCell<Vec<Rc<Node>>>,
    // Same chokepoint, ExprCast arm — the one the three caches above missed. A cast resolved
    // its target type per EVALUATION: cast_target_seed_name re-sliced authored source text
    // (authored_name_at), and for any target whose name is not already "String" the alias-chain
    // walk called lookup_type_item_across_modules, which SCANS every item of every module and
    // extracts authored source text for each item it compares — once per hop, up to 32 hops.
    // Both names are pure functions of the target node and the module set, and both are fixed
    // for a ctx, so they are memoized per target node — keyed by node pointer, kept alive via
    // cast_kernel_cache_keepalive exactly as call_func_name_cache above.
    cast_kernel_cache: std::cell::RefCell<HashMap<usize, Rc<CastTargetNames>>>,
    cast_kernel_cache_keepalive: std::cell::RefCell<Vec<Rc<Node>>>,
    // The alias walk's per-hop `lookup_type_item_across_modules` was a LINEAR SCAN over every
    // item of every module, extracting authored source text per item compared. Measured on one
    // daily-page render: 700 lookups scanned 1,967,155 items (~2,810 each), which accounted for
    // essentially all of ExprCast's 2,027ms — and the term grows with the closure, not the
    // request. A name->item map is the same fact indexed instead of searched, built once per
    // ctx. `or_insert` preserves the scan's first-match-wins order.
    type_item_index: std::cell::RefCell<Option<Rc<HashMap<String, Rc<Node>>>>>,
    // The cast's SOURCE-side name, same class as cast_kernel_cache above (see
    // cast_expr_inferred_type_name).
    cast_source_name_cache: std::cell::RefCell<HashMap<usize, String>>,
    cast_source_name_cache_keepalive: std::cell::RefCell<Vec<Rc<Node>>>,
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
    /// Entry identity the armed budget belongs to, so the neutral `EvaluationBudgetExceeded`
    /// can name what crossed rather than leaving the caller to infer it.
    budget_entry: std::cell::RefCell<Option<String>>,
    // Lane-level budget: when set, run_claim_measured re-arms the deadline per witness.
    witness_eval_budget_ms: std::cell::Cell<Option<u64>>,
    // Whole-receipt wall budget for Wet self-host receipts (emit+cargo subprocess I/O included).
    witness_wall_budget_ms: std::cell::Cell<Option<u64>>,
    // Kill-at-deadline arm for the wall budget (Finding 1, 2026-07-25): (start, budget_ms).
    // Shell waits poll this and SIGKILL the process group at the ceiling — the completion-
    // side `wall_budget_completion_outcome` remains a backstop for non-subprocess spend.
    // Without this arm the refusal fires only after the overrun is fully spent (707s on a
    // 600s budget; 21–34min receipts in the original finding).
    witness_wall_deadline: std::cell::Cell<Option<(Instant, u64)>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedFunctionIdentity {
    pub module_path: String,
    pub decl_name: String,
    pub bare_name_ambiguous: bool,
}

/// The module path a source file authors, or `None` when the index cannot name
/// exactly one. Made public for the FLOOR2 qualified-witness lookup: under one
/// shared prepared subject a witness must be invoked by `module.function`, and
/// deriving that mapping a second time in the caller would fork this one.
pub fn selected_module_path(
    file: &str,
    module_path_index: &HashMap<String, String>,
) -> Option<String> {
    let normalize = |path: &str| {
        path.replace('\\', "/")
            .split("/./")
            .collect::<Vec<_>>()
            .join("/")
            .trim_start_matches("./")
            .to_string()
    };
    let file = normalize(file);
    let exact: Vec<_> = module_path_index
        .iter()
        .filter(|(_, path)| normalize(path) == file)
        .map(|(module, _)| module.clone())
        .collect();
    if exact.len() == 1 {
        return exact.into_iter().next();
    }
    if !exact.is_empty() {
        return None;
    }
    let suffix: Vec<_> = module_path_index
        .iter()
        .filter(|(_, path)| file.ends_with(&normalize(path)))
        .map(|(module, _)| module.clone())
        .collect();
    (suffix.len() == 1).then(|| suffix.into_iter().next().expect("one suffix"))
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
        Self::over_scope_indexes(
            Self::build_scope_indexes(graph, source_indices),
            execution_mode,
            fixture_store,
            whole_tree_published_keys,
        )
    }

    /// Build the immutable index set for one module population. THE EXPENSIVE HALF: it walks
    /// every module and every item, so its cost is denominated in the scope, and a caller that
    /// runs it per claim has re-introduced a multiplier.
    pub fn build_scope_indexes(
        graph: &ResolvedGraph,
        source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    ) -> Rc<PreparedScopeIndexes> {
        SCOPE_INDEX_CONSTRUCTIONS.with(|c| c.set(c.get() + 1));
        let mut fn_nodes = HashMap::new();
        let mut bare_name_counts = HashMap::<String, usize>::new();
        let mut service_ops = HashMap::new();
        for module in graph.modules.iter() {
            let module_path = authored_name_at(source_indices.clone(), module.module.clone());
            for item in module.items.iter() {
                let name = authored_name_at(source_indices.clone(), item.clone());
                if !name.is_empty() {
                    *bare_name_counts.entry(name.clone()).or_default() += 1;
                    fn_nodes.insert(name.clone(), item.clone());
                    if !module_path.is_empty() {
                        let qualified = format!("{}.{}", module_path, name);
                        fn_nodes.insert(qualified.clone(), item.clone());
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
        let ambiguous_bare_function_names = bare_name_counts
            .into_iter()
            .filter_map(|(name, count)| (count > 1).then_some(name))
            .collect();
        Rc::new(PreparedScopeIndexes {
            modules: graph.modules.clone(),
            item_registry: graph.item_registry.clone(),
            source_indices,
            emit_graph_info: graph.emit_graph_info.clone(),
            fn_nodes,
            ambiguous_bare_function_names,
            service_ops,
        })
    }

    /// Join shared immutable indexes with FRESH MUTABLE STATE. This is the per-claim
    /// constructor, and it is cheap by construction: it clones `Rc` handles and allocates empty
    /// caches. Nothing here walks a module.
    ///
    /// Every mutable field below is deliberately fresh rather than shared. Memos, name caches,
    /// the effect odometer and the deadline arms all carry state from the claim that ran
    /// before, and sharing them across claims is how one witness's evaluation becomes another
    /// witness's answer.
    pub fn over_scope_indexes(
        indexes: Rc<PreparedScopeIndexes>,
        execution_mode: ExecutionMode,
        fixture_store: Option<Rc<crate::recorded_fixture::RecordedFixtureStore>>,
        whole_tree_published_keys: Option<Rc<std::collections::HashSet<String>>>,
    ) -> Self {
        InterpContext {
            modules: indexes.modules.clone(),
            item_registry: indexes.item_registry.clone(),
            source_indices: indexes.source_indices.clone(),
            emit_graph_info: indexes.emit_graph_info.clone(),
            indexes,
            execution_mode,
            fixture_store,
            data_cache: std::cell::RefCell::new(HashMap::new()),
            param_name_cache: std::cell::RefCell::new(HashMap::new()),
            param_name_cache_keepalive: std::cell::RefCell::new(Vec::new()),
            var_sym_cache: std::cell::RefCell::new(HashMap::new()),
            var_sym_cache_keepalive: std::cell::RefCell::new(Vec::new()),
            call_func_name_cache: std::cell::RefCell::new(HashMap::new()),
            call_func_name_cache_keepalive: std::cell::RefCell::new(Vec::new()),
            cast_kernel_cache: std::cell::RefCell::new(HashMap::new()),
            cast_kernel_cache_keepalive: std::cell::RefCell::new(Vec::new()),
            type_item_index: std::cell::RefCell::new(None),
            cast_source_name_cache: std::cell::RefCell::new(HashMap::new()),
            cast_source_name_cache_keepalive: std::cell::RefCell::new(Vec::new()),
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
            budget_entry: std::cell::RefCell::new(None),
            witness_eval_budget_ms: std::cell::Cell::new(None),
            witness_wall_budget_ms: std::cell::Cell::new(None),
            witness_wall_deadline: std::cell::Cell::new(None),
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

    /// Milliseconds left on the armed CPU deadline, or `None` when none is armed.
    /// `Some(0)` means already past.
    pub fn eval_deadline_remaining_ms(&self) -> Option<u64> {
        let (baseline, budget_ms) = self.eval_deadline.get()?;
        let elapsed_ms = (thread_cpu_nanos().saturating_sub(baseline) / 1_000_000) as u64;
        Some(budget_ms.saturating_sub(elapsed_ms))
    }

    /// The entry identity the currently-armed budget belongs to, for the neutral result.
    pub fn budget_entry(&self) -> Option<String> {
        self.budget_entry.borrow().clone()
    }

    /// Enter a scoped evaluation budget that can only ever TIGHTEN what is already armed.
    ///
    /// This exists because the paired `arm_*` / `clear_*` calls compose wrongly, and wrongly in
    /// the fail-open direction (verified 2026-08-09). `arm_eval_deadline` sets its cell
    /// unconditionally with a FRESH baseline, so a nested arm does not shorten an outer bound —
    /// it restarts the clock and grants the outer evaluation a whole new budget. `clear_*` then
    /// sets `None` rather than restoring what it displaced, so an inner clear disarms an outer
    /// deadline entirely. Both are silent.
    ///
    /// The guard fixes composition in one place: the effective limit is the smaller of what
    /// REMAINS on the outer deadline and what this scope requests (remaining, not declared —
    /// two scopes carrying the same declared limit but armed at different instants have
    /// different time left, and it is the time left that decides which fires first), and every
    /// exit path restores the displaced state because `Drop` runs on early return and on unwind.
    /// A leaked deadline is not merely an absent bound: the CPU baseline is captured at arm
    /// time, so a deadline surviving into a later evaluation measures that evaluation against a
    /// baseline already spent, and refuses it immediately. On a long-lived process sharing one
    /// `InterpContext` across requests — `gunbc serve` — that would refuse every subsequent
    /// request for the life of the process.
    pub fn enter_evaluation_budget(
        &self,
        entry: &str,
        cpu_limit_ms: Option<u64>,
        wall_limit_ms: Option<u64>,
    ) -> EvaluationBudgetScope<'_> {
        let prior_eval = self.eval_deadline.get();
        let prior_wall = self.witness_wall_deadline.get();
        let prior_stride = self.eval_deadline_stride.get();
        let prior_entry = self.budget_entry.borrow().clone();

        // Earliest-deadline-wins on both clocks. `None` requested means "this scope declares no
        // bound on this clock" — which must NOT disarm an outer bound, so the prior survives.
        if let Some(requested) = cpu_limit_ms {
            let effective = match self.eval_deadline_remaining_ms() {
                Some(remaining) => remaining.min(requested),
                None => requested,
            };
            self.eval_deadline
                .set(Some((thread_cpu_nanos(), effective)));
            self.eval_deadline_stride.set(0);
        }
        if let Some(requested) = wall_limit_ms {
            let effective = match self.wall_deadline_remaining_ms() {
                Some(remaining) => remaining.min(requested),
                None => requested,
            };
            self.witness_wall_deadline
                .set(Some((Instant::now(), effective)));
        }
        if cpu_limit_ms.is_some() || wall_limit_ms.is_some() {
            *self.budget_entry.borrow_mut() = Some(entry.to_string());
        }

        EvaluationBudgetScope {
            ctx: self,
            prior_eval,
            prior_wall,
            prior_stride,
            prior_entry,
        }
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

    pub fn arm_wall_deadline(&self, budget_ms: u64) {
        self.witness_wall_deadline
            .set(Some((Instant::now(), budget_ms)));
    }

    pub fn clear_wall_deadline(&self) {
        self.witness_wall_deadline.set(None);
    }

    /// Remaining wall-budget milliseconds, or `None` when no deadline is armed.
    /// `Some(0)` means the ceiling is already past — callers must refuse now.
    pub fn wall_deadline_remaining_ms(&self) -> Option<u64> {
        let (start, budget_ms) = self.witness_wall_deadline.get()?;
        let elapsed_ms = start.elapsed().as_millis() as u64;
        Some(budget_ms.saturating_sub(elapsed_ms))
    }

    pub fn wall_deadline_exceeded_error(&self) -> Option<InterpError> {
        let (start, budget_ms) = self.witness_wall_deadline.get()?;
        let elapsed = start.elapsed();
        if elapsed.as_millis() as u64 > budget_ms {
            Some(InterpError::EvaluationBudgetExceeded {
                entry: self.budget_entry_or_unnamed(),
                clock: EvaluationClock::MonotonicWall,
                elapsed_nanos: elapsed.as_nanos(),
                limit_ms: budget_ms,
            })
        } else {
            None
        }
    }

    /// Entry identity for a budget result. `arm_*` callers that set no entry (the witness lane,
    /// which maps this result into its own refusal and supplies the witness name there) get an
    /// explicit placeholder rather than an empty string, so an unnamed entry is visibly unnamed
    /// instead of looking like a successfully-read empty name.
    fn budget_entry_or_unnamed(&self) -> String {
        self.budget_entry
            .borrow()
            .clone()
            .unwrap_or_else(|| "<unnamed-evaluation>".to_string())
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
        self.indexes.fn_nodes.get(name)
    }

    pub fn lookup_fn_node(&self, qualified_name: &str) -> Option<Rc<Node>> {
        self.indexes.fn_nodes.get(qualified_name).cloned()
    }

    pub fn resolved_graph(&self) -> ResolvedGraph {
        ResolvedGraph {
            modules: self.modules.clone(),
            item_registry: self.item_registry.clone(),
            diagnostics: Rc::new(im::Vector::new()),
            emit_graph_info: self.emit_graph_info.clone(),
        }
    }

    /// Report identity from the exact fn_nodes entry used by lookup_fn. The
    /// module path comes from the existing collision-checked module index; this
    /// accessor does not select, resolve, traverse the graph, or alter lookup.
    pub fn selected_function_identity(
        &self,
        name: &str,
        module_path_index: &HashMap<String, String>,
    ) -> Option<SelectedFunctionIdentity> {
        let node = self.lookup_fn(name)?;
        let file = node.span.file.as_str();
        let module_path = selected_module_path(file, module_path_index)?;
        Some(SelectedFunctionIdentity {
            module_path,
            decl_name: authored_name_at(self.source_indices.clone(), node.clone()),
            bare_name_ambiguous: !name.contains('.')
                && self.indexes.ambiguous_bare_function_names.contains(name),
        })
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
            .ok_or_else(|| InterpError::NoSuchFunction {
                name: entry_fn.to_string(),
            })?
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
            .ok_or_else(|| InterpError::NoSuchFunction {
                name: entry_fn.to_string(),
            })?
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
            ctx.param_name_cache_keepalive
                .borrow_mut()
                .push(fn_node.clone());
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
                // A duplicate caller label silently overwrote the earlier binding via
                // HashMap::insert, so the earlier argument's evaluated value vanished
                // unlocatably (DESIGN §5: the compile-side wall must not report a fact the
                // runtime keeps quiet about). Refuse instead of taking the last value.
                //
                // An anonymous binding key ("_") is excluded from the collision check (but
                // still inserted, harmlessly overwriting any earlier "_" — the body cannot
                // read a parameter named "_", so two anonymous parameters bound at different
                // positions/labels are two distinct, unreadable slots, not a collision; the
                // insert itself must still happen so the required-argument "supplied" check
                // below, which reads `bindings.contains_key`, keeps seeing an anonymous
                // parameter as filled) (review from parent session loyal-ant-382, 2026-08-05
                // — the prior form both keyed AND refused every anonymous param under the
                // literal "_", false-refusing a signature with two or more anonymous
                // parameters).
                if name != "_" && bindings.contains_key(&ctx.sym(name)) {
                    return Err(InterpError::CallContractMismatch {
                        callee: fn_node.name.clone(),
                        detail: format!("argument '{}' supplied more than once", name),
                    });
                }
                bindings.insert(ctx.sym(name), val.clone());
            } else if positional_idx < param_names.len() {
                // A positional actual is keyed by its resolved declared parameter, exactly as
                // the named branch above is — so a positional actual filling a parameter an
                // earlier named actual already bound (`two(a: 1, 2)` against `fn two(a, b)`,
                // where the positional slot 0's declared name is `a`, already bound by the
                // named actual) must refuse the same way, not silently overwrite last-write-wins
                // (DESIGN §5 fail-closed; review 48817).
                //
                // An anonymous declared parameter ("_") is excluded from the collision check
                // (see the named-branch note above): two anonymous parameters filled
                // positionally are two distinct, unreadable slots, not a duplicate. The
                // insert still happens unconditionally so the required-argument check below
                // sees the slot as filled.
                let pname = &param_names[positional_idx];
                if pname != "_" && bindings.contains_key(&ctx.sym(pname)) {
                    return Err(InterpError::CallContractMismatch {
                        callee: fn_node.name.clone(),
                        detail: format!("argument '{}' supplied more than once", pname),
                    });
                }
                bindings.insert(ctx.sym(pname), val.clone());
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

    let caller_label_matches_param = |param_name: &str, arg_label: &str| {
        param_name == arg_label
            || param_name == "_"
            || param_name
                .strip_prefix('_')
                .is_some_and(|stripped| stripped == arg_label)
    };
    let param_supplied_at_call = |pname: &str| {
        if bindings.contains_key(&ctx.sym(pname)) {
            return true;
        }
        for (opt_name, _) in args.iter() {
            if let Some(label) = opt_name {
                if caller_label_matches_param(pname, label) {
                    return true;
                }
            }
        }
        false
    };

    let required_count = fn_node
        .params
        .iter()
        .enumerate()
        .filter(|(i, p)| {
            param_node_default_value((*p).clone()).is_none()
                && match p.children.first() {
                    Some(type_expr) => {
                        authored_name_at(ctx.si(), type_expr.clone()) != all_param_names[*i]
                    }
                    None => false,
                }
        })
        .count();
    let supplied_required = fn_node
        .params
        .iter()
        .enumerate()
        .filter(|(i, p)| {
            param_node_default_value((*p).clone()).is_none()
                && match p.children.first() {
                    Some(type_expr) => {
                        authored_name_at(ctx.si(), type_expr.clone()) != all_param_names[*i]
                    }
                    None => false,
                }
                && param_supplied_at_call(&all_param_names[*i])
        })
        .count();
    for (i, param) in fn_node.params.iter().enumerate() {
        let pname = &all_param_names[i];
        let is_value_param = match param.children.first() {
            Some(type_expr) => authored_name_at(ctx.si(), type_expr.clone()) != *pname,
            None => false,
        };
        if is_value_param
            && param_node_default_value(param.clone()).is_none()
            && !param_supplied_at_call(pname)
        {
            return Err(InterpError::CallContractMismatch {
                callee: fn_node.name.clone(),
                detail: format!(
                    "missing required argument '{}' ({} of {} required argument(s) supplied)",
                    pname, supplied_required, required_count
                ),
            });
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
/// Maps the kernel's caller-agnostic budget result into the WITNESS lane's refusal vocabulary.
///
/// The kernel raises `EvaluationBudgetExceeded` for every caller (see that variant's comment for
/// why it must not raise a witness concept). The witness lane's diagnostics carry operator
/// rulings its consumers depend on — the 5s fast-lane rule, and the "relocating the file does not
/// discharge it" guidance that exists because a witness was once re-homed under `long/` to
/// silence exactly this error — so that text stays here, at the witness boundary, rather than
/// leaking into an HTTP response or being deleted.
///
/// Any non-budget error passes through untouched.
pub fn map_budget_error_to_witness_refusal(err: InterpError) -> InterpError {
    match err {
        InterpError::EvaluationBudgetExceeded {
            clock,
            elapsed_nanos,
            limit_ms,
            ..
        } => match clock {
            EvaluationClock::ThreadCpu => InterpError::EvalBudgetExceeded {
                cpu_ms: (elapsed_nanos / 1_000_000) as u64,
                budget_ms: limit_ms,
            },
            EvaluationClock::MonotonicWall => InterpError::WitnessWallBudgetExceeded {
                wall_ms: (elapsed_nanos / 1_000_000) as u64,
                budget_ms: limit_ms,
            },
        },
        other => other,
    }
}

/// Which bound fired. Carried in the neutral result because CPU and wall are different
/// quantities of the same occurrence and the remedies differ: a CPU crossing is a spin, a wall
/// crossing with small CPU is a stall. Mirrors `std.evaluation_budget.EvaluationClock`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvaluationClock {
    ThreadCpu,
    MonotonicWall,
}

impl EvaluationClock {
    /// Stable wire key. Must equal `std.evaluation_budget.evaluation_clock_key`.
    pub fn key(&self) -> &'static str {
        match self {
            EvaluationClock::ThreadCpu => "thread_cpu",
            EvaluationClock::MonotonicWall => "monotonic_wall",
        }
    }
}

/// Restores the evaluation-budget state it displaced, on every exit path including unwind.
/// See `InterpContext::enter_evaluation_budget` for why paired arm/clear calls are not enough.
pub struct EvaluationBudgetScope<'a> {
    ctx: &'a InterpContext,
    prior_eval: Option<(u128, u64)>,
    prior_wall: Option<(Instant, u64)>,
    prior_stride: u32,
    prior_entry: Option<String>,
}

impl Drop for EvaluationBudgetScope<'_> {
    fn drop(&mut self) {
        self.ctx.eval_deadline.set(self.prior_eval);
        self.ctx.witness_wall_deadline.set(self.prior_wall);
        self.ctx.eval_deadline_stride.set(self.prior_stride);
        *self.ctx.budget_entry.borrow_mut() = self.prior_entry.take();
    }
}

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
    // The stride poll runs when EITHER clock is armed. Gating it on the CPU deadline alone was a
    // real defect, found by executing a wall-only serve process rather than by reading: with no
    // CPU limit the whole poll became unreachable, so a wall-only caller — precisely the
    // configuration that bounds a low-CPU stall — silently had no in-eval crossing point at all.
    //
    // Neither clock contains an evaluation blocked inside a single native primitive: that never
    // returns to `eval_expr`, so nothing is polled. That residue is why worker isolation, not a
    // budget, is what bounds the listener unconditionally.
    let cpu_armed = ctx.eval_deadline.get();
    let wall_armed = ctx.witness_wall_deadline.get().is_some();
    if cpu_armed.is_some() || wall_armed {
        let stride = ctx.eval_deadline_stride.get().wrapping_add(1);
        ctx.eval_deadline_stride.set(stride);
        if stride % 4096 == 0 {
            if let Some((cpu_baseline_nanos, budget_ms)) = cpu_armed {
                let elapsed_nanos = thread_cpu_nanos().saturating_sub(cpu_baseline_nanos);
                if (elapsed_nanos / 1_000_000) as u64 > budget_ms {
                    return Err(InterpError::EvaluationBudgetExceeded {
                        entry: ctx.budget_entry_or_unnamed(),
                        clock: EvaluationClock::ThreadCpu,
                        elapsed_nanos,
                        limit_ms: budget_ms,
                    });
                }
            }
            if let Some(err) = ctx.wall_deadline_exceeded_error() {
                return Err(err);
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
        LiteralValue::LitStr { value } => Ok(str_value(value.clone())),
        LiteralValue::LitSymbol { value } => Ok(str_value(value.clone())),
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
    let sym = {
        let key = Rc::as_ptr(node) as usize;
        let existing = ctx.var_sym_cache.borrow().get(&key).copied();
        if let Some(s) = existing {
            s
        } else {
            let name = expr_var_name_at(node.clone(), ctx.si());
            let fresh = ctx.sym(&name);
            ctx.var_sym_cache_keepalive.borrow_mut().push(node.clone());
            ctx.var_sym_cache.borrow_mut().insert(key, fresh);
            fresh
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
                            return Ok(str_value(name));
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
            return Ok(str_value(format!("{}{}", a, b)));
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
                    return Ok(str_value(format!("{}{}", ls, s)));
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
                    return Ok(str_value(format!("{}{}", s, rs)));
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
            if let Some(detail) = cross_family_content_hash_straddle(&left, &right) {
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

fn is_content_hash_family_variant(name: &str) -> bool {
    matches!(name, "Fnv1a64" | "Sha256Hash" | "Sha1Hash" | "Sha512Hash")
}

fn is_content_hash_value(v: &Value) -> bool {
    match v {
        Value::Variant {
            type_name,
            variant_name,
            ..
        } => {
            resolve_sym(*type_name) == "ContentHash"
                && is_content_hash_family_variant(&resolve_sym(*variant_name))
        }
        _ => false,
    }
}

fn cross_family_content_hash_straddle(a: &Value, b: &Value) -> Option<String> {
    match (a, b) {
        (va, vb) if is_content_hash_value(va) && is_content_hash_value(vb) => {
            let (an, bn) = match (va, vb) {
                (
                    Value::Variant {
                        variant_name: an, ..
                    },
                    Value::Variant {
                        variant_name: bn, ..
                    },
                ) => (an, bn),
                _ => return None,
            };
            if an == bn {
                None
            } else {
                Some(format!(
                    "{} vs {} — ContentHash families are not comparable; bare `==` would \
                     silently fabricate `false` whereas structural fnv1a64 and cited \
                     SHA-256/SHA-1/SHA-512 digests are different kinds with different remedies \
                     (DESIGN §5 / feature:content-hash-family-grounded). Match on family \
                     and use per-family eq, or admit_pin_integrity at union carriers.",
                    describe_repr(a),
                    describe_repr(b),
                ))
            }
        }
        _ => None,
    }
}

#[cfg(any(test, feature = "interp_test_witness"))]
pub fn cross_family_content_hash_straddle_for_witness(a: &Value, b: &Value) -> Option<String> {
    cross_family_content_hash_straddle(a, b)
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
            str_value("map_lookup_port".to_string()),
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
            (ctx.sym("reason"), str_value("map_key_absent".to_string())),
        ])),
    }
}

// HAND-RUST GATE explicit deferral (review 46616): bounded growth in the existing
// seed interpreter, not a new Rust authority. The evaluation-boundary POLICY is
// modeled — `v2.std.witness_evaluation` owns `WitnessEvaluation`/`WitnessEvaluationFrame`
// and `extdeps.transports.rest` `rest_exchange_resolution` owns the lookup, equality,
// and handler-selection decisions; what lives here is only the dynamic-extent
// realization of pushing and popping a frame, which no modeled construct can express
// while the seed is the evaluator.
//
// Lane: ROADMAP `v1-materialization-kernel` (rn_53JPH6BB7G588K7DMZNWM0E3AS,
// docs/plans/witness-realization-plan.md) — the same lane
// `extdeps.realization.emit_on_demand_host` `emit_on_demand_host_seed_deferral_note`
// defers to; counted against `v1-honest-frontier` and terminating at
// `v1-interpreter-quarantine` → `v1-interpreter-delete`.
//
// Deletion condition, checkable by execution: witnesses emit to native code and the
// emitted runtime realizes the evaluation frame, at which point this stack, its
// `WITNESS_EVALUATION_MODULE` dispatch, and `witness_evaluation_diagnostic_value`
// delete together while `rest_replay_binding_does_not_escape_its_frame` stays green
// without them. That witness is the regression control for the deletion, not just for
// the frame — it fails if a binding survives its frame under either realization.
//
// Citation note: the two sibling deferrals in this file and in
// `emit_on_demand_host_seed_deferral_note` name a
// `dag/gunbc/v1_deletion_plan.dag ^witness_realization_kernel` deletion row. That row
// no longer exists — the brick ledger it belonged to was retired 2026-07-28 by that
// file's own `v1_exit_model_doc`, which moved per-node acceptance onto the roadmap
// tickets. This deferral therefore names the live roadmap node instead of copying a
// dead row forward; repointing the two stale siblings is left to the lane that owns
// them rather than smuggled into this diff.
thread_local! {
    /// Dynamically scoped witness frames. The .dag carrier owns their contents;
    /// this stack is only the v1 seed realization of the evaluation boundary.
    /// A frame is pushed immediately before its subject closure and removed by
    /// `WitnessFramePop`'s Drop on every exit path — returned, refused, or
    /// unwound — so replay bindings cannot become ambient.
    static WITNESS_EVALUATION_FRAMES: RefCell<Vec<Value>> = const { RefCell::new(Vec::new()) };
}

/// Pops the frame its constructor's caller pushed, on EVERY exit path including an
/// unwind. The pop used to be an ordinary statement after `apply_closure`, which held
/// only because no path between the two returned early — a property of the current
/// body, not of the code, so the block comment's "removed on both returned/refused
/// paths" was a promise the shape did not keep (review 46767).
///
/// The leak is worth closing by construction rather than by care because a leaked
/// frame is no longer merely a stale binding: `dispatch_service` consults
/// `current_witness_evaluation_frame()` to decide whether a hermetic op routes to the
/// real dispatcher, so an escaped frame would silently route *subsequent* ops out of
/// the mock layer. Drop makes the escape unwritable instead of merely unlikely (§5).
struct WitnessFramePop;

impl Drop for WitnessFramePop {
    fn drop(&mut self) {
        WITNESS_EVALUATION_FRAMES.with(|frames| {
            let _ = frames.borrow_mut().pop();
        });
    }
}

const WITNESS_EVALUATION_MODULE: &str = "v2.std.witness_evaluation";

fn witness_evaluation_diagnostic_value(
    ctx: &InterpContext,
    call_node: &Rc<Node>,
    error: &InterpError,
) -> Value {
    let at = format!("{}:{}", call_node.span.file, call_node.span.start);
    let locus = Value::Variant {
        type_name: ctx.sym("Locus"),
        variant_name: ctx.sym("PortLocus"),
        fields: Rc::new(vec![(
            ctx.sym("anchor"),
            Value::Record {
                type_name: ctx.sym("LocusAnchor"),
                fields: Rc::new(vec![(ctx.sym("at"), str_value(at))]),
            },
        )]),
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
            (ctx.sym("reason"), str_value(error.to_string())),
        ])),
    }
}

fn witness_evaluation_variant(
    ctx: &InterpContext,
    variant: &str,
    field: &str,
    value: Value,
) -> Value {
    Value::Variant {
        type_name: ctx.sym("WitnessEvaluation"),
        variant_name: ctx.sym(variant),
        fields: Rc::new(vec![(ctx.sym(field), value)]),
    }
}

fn current_witness_evaluation_frame() -> Option<Value> {
    WITNESS_EVALUATION_FRAMES.with(|frames| frames.borrow().last().cloned())
}

fn try_witness_evaluation_dispatch(
    ctx: &InterpContext,
    call_node: &Rc<Node>,
    fn_node: &Rc<Node>,
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
) -> Option<InterpResult<Value>> {
    let is_authority = ctx
        .item_registry
        .get(&fn_node.name)
        .is_some_and(|info| info.module_name == WITNESS_EVALUATION_MODULE);
    if !is_authority {
        return None;
    }
    match fn_node.name.as_str() {
        "witness_evaluation_frame_active" => Some(Ok(Value::Bool(
            WITNESS_EVALUATION_FRAMES.with(|frames| !frames.borrow().is_empty()),
        ))),
        "witness_diagnostic_rendered_reason" => {
            let diagnostic = args.first().map(|(_, value)| value);
            let rendered = match diagnostic {
                Some(Value::Record { fields, .. }) => ctx.field(fields, "reason").cloned(),
                _ => None,
            };
            Some(rendered.ok_or_else(|| InterpError::TypeError {
                msg: "witness diagnostic is malformed".to_string(),
            }))
        }
        "evaluate_in_witness_frame" => {
            let argument = |name: &str, position: usize| {
                args.iter()
                    .find(|(label, _)| label.as_deref() == Some(name))
                    .or_else(|| args.get(position))
                    .map(|(_, value)| value.clone())
            };
            let Some(frame) = argument("frame", 0) else {
                return Some(Err(InterpError::TypeError {
                    msg: "evaluate_in_witness_frame requires a frame".to_string(),
                }));
            };
            let Some(subject) = argument("subject", 1) else {
                return Some(Err(InterpError::TypeError {
                    msg: "evaluate_in_witness_frame requires a subject".to_string(),
                }));
            };
            WITNESS_EVALUATION_FRAMES.with(|frames| frames.borrow_mut().push(frame));
            let evaluated = {
                let _pop = WitnessFramePop;
                apply_closure(&subject, &[Value::Bool(true)], env, ctx)
            };
            Some(Ok(match evaluated {
                Ok(value) => witness_evaluation_variant(ctx, "WitnessReturned", "value", value),
                Err(error) => witness_evaluation_variant(
                    ctx,
                    "WitnessRefused",
                    "diagnostic",
                    witness_evaluation_diagnostic_value(ctx, call_node, &error),
                ),
            }))
        }
        _ => None,
    }
}

fn match_pattern(
    pattern: &MatchPattern,
    value: &Value,
    ctx: &InterpContext,
) -> Option<HashMap<Symbol, Value>> {
    match pattern {
        MatchPattern::Wildcard => Some(HashMap::new()),

        MatchPattern::Bind { declaration } => {
            let mut bindings = HashMap::new();
            bindings.insert(ctx.sym(&declaration.name), value.clone());
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
            // A qualified pattern spelling (`module.Variant`) resolves the arm name to its
            // containment path, but values are constructed with the bare last segment
            // (see the short-name normalization at value construction). Every name-vs-literal
            // reconciliation below — the native Int/Str/List coproducts and the Optional/
            // Witness raw (value-or-Null) unwraps — must compare that short segment, mirroring
            // the `Value::Variant` arm's fallback; otherwise a qualified `Zero`/`Succ` (Nat
            // grounded to native Int), `Empty`/`Cons`, or `Present`/`Absent` pattern misses
            // its native value and the match falls through non-exhaustive.
            let name_last = name.rsplit('.').next().unwrap_or(name);
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
            if name_last == "Present"
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
            if name_last == "Holds"
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
                    type_name,
                    variant_name,
                    fields,
                } => {
                    if name_last == "Holds"
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
                    if name_last == "Present"
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
                    if let Some(parent) = parent_enum.as_ref() {
                        if !coproduct_parent_spellings_match(ctx, resolve_sym(*type_name), parent) {
                            return None;
                        }
                    }
                    if !coproduct_arm_name_matches(resolve_sym(*variant_name), name.clone()) {
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
                Value::Record { type_name, fields } => {
                    if !record_pattern_type_name_matches(
                        ctx,
                        *type_name,
                        name,
                        parent_enum.as_ref(),
                    ) {
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
                Value::List(items) => match name_last {
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
                Value::Str(s) if name_last == "Empty" || name_last == "Cons" => match name_last {
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
                                let tail = str_value(chars.as_str().to_string());
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
                // GroupCompletion{pos,neg} destructuring against a native Value::Int is
                // deliberately unhandled here (no corpus site exercises it yet, #5-scoped
                // deferral) — an unmatched pattern name falls through to `_ => None` below,
                // refusing rather than fabricating a wrong (pos, neg) pair.
                Value::Int(n) if name_last == "Zero" || name_last == "Succ" => match name_last {
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
                Value::Null
                    if name_last == "Violates" && parent_enum.as_deref() == Some("Witness") =>
                {
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
                Value::Null
                    if name_last == "None" && parent_enum.as_deref() == Some("Diagnostics") =>
                {
                    Some(HashMap::new())
                }
                Value::Null
                    if name_last == "Absent" && parent_enum.as_deref() == Some("Optional") =>
                {
                    Some(HashMap::new())
                }
                _ if name_last == "Present" && parent_enum.as_deref() == Some("Optional") => {
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
                _ if name_last == "Holds" && parent_enum.as_deref() == Some("Witness") => {
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

/// Handler bodies for v4 std-bridge dispatch. Roster authority is
/// `v1_interpreter_authored_roster_arms()` in `.dag`; generated
/// `lookup_eval_call_bridge` routes spellings before this macro matches on
/// the generated enum variant for each arm identity.
macro_rules! v1_bridge_family_arms {
    ($cb:ident, $fname:ident, $args:ident, $node:ident, $ctx:ident) => {
        $cb! {
            $fname, $args, $node, $ctx;
            family STD_NODE_BRIDGE_FNS "v2.std.node"
                lookup_eval_call_bridge_std_node eval_call_bridge__v2_std_node_arm {
                arm "v4_bridge.resolve_type_node" { "resolve_type_node" } =>
                    crate::coproduct_reflection::eval_resolve_type_node($ctx, &$args),
            }
            family STD_LEXING_BRIDGE_FNS "v2.std.compilers.lexing"
                lookup_eval_call_bridge_std_compilers_lexing eval_call_bridge__v2_std_compilers_lexing_arm {
                arm "v4_bridge.symbol_intern_lexeme" { "symbol_intern_lexeme" } =>
                    crate::coproduct_reflection::eval_symbol_intern_lexeme($ctx, &$args),
                arm "v4_bridge.symbol_lexeme" { "symbol_lexeme" } =>
                    crate::coproduct_reflection::eval_symbol_lexeme($ctx, &$args),
            }
            family STD_QUALIFIED_NAME_BRIDGE_FNS "v2.std.qualified_name"
                lookup_eval_call_bridge_std_qualified_name eval_call_bridge__v2_std_qualified_name_arm {
                arm "v4_bridge.qualified_name_from_dotted_string" { "qualified_name_from_dotted_string" } =>
                    crate::coproduct_reflection::eval_qualified_name_from_dotted_string($ctx, &$args),
            }
            family STD_NODE_QUERY_BRIDGE_FNS "v2.std.node_query"
                lookup_eval_call_bridge_std_node_query eval_call_bridge__v2_std_node_query_arm {
                arm "v4_bridge.coproduct_nullary_inhabitants" { "coproduct_nullary_inhabitants" } =>
                    crate::coproduct_reflection::eval_coproduct_nullary_inhabitants($ctx, $node, &$args),
            }
            family STD_CONCEPT_INDEX_BRIDGE_FNS "v2.std.concept_index"
                lookup_eval_call_bridge_std_concept_index eval_call_bridge__v2_std_concept_index_arm {
                arm "v4_bridge.concept_decl_facts_live" { "concept_decl_facts_live" } =>
                    crate::coproduct_reflection::eval_concept_decl_facts_live($ctx, &$args),
            }
            family STD_FN_INDEX_BRIDGE_FNS "v2.std.fn_index"
                lookup_eval_call_bridge_std_fn_index eval_call_bridge__v2_std_fn_index_arm {
                arm "v4_bridge.fn_arrow_decl_facts_live" { "fn_arrow_decl_facts_live" } =>
                    crate::coproduct_reflection::eval_fn_arrow_decl_facts_live($ctx, &$args),
                arm "v4_bridge.fn_arrow_decl_substrate_is_whole_tree" { "fn_arrow_decl_substrate_is_whole_tree" } =>
                    crate::coproduct_reflection::eval_fn_arrow_decl_substrate_is_whole_tree($ctx, &$args),
            }
            family CORPUS_DEPENDENCY_VIEW_BRIDGE_FNS "v2.lens.affected_set.corpus_dependency_view"
                lookup_eval_call_bridge_lens_affected_set_corpus_dependency_view eval_call_bridge__v2_lens_affected_set_corpus_dependency_view_arm {
                arm "v4_bridge.corpus_dependency_view_per_pr_substrate_refuse" { "corpus_dependency_view_per_pr_substrate_refuse" } =>
                    crate::coproduct_reflection::eval_corpus_dependency_view_per_pr_substrate_refuse($ctx, &$args),
            }
            family STD_DATA_INDEX_BRIDGE_FNS "v2.std.data_index"
                lookup_eval_call_bridge_std_data_index eval_call_bridge__v2_std_data_index_arm {
                arm "v4_bridge.data_init_decl_facts_live" { "data_init_decl_facts_live" } =>
                    crate::coproduct_reflection::eval_data_init_decl_facts_live($ctx, &$args),
            }
        }
    };
}

/// Expansion 1: the name lists the guard predicate tests.
macro_rules! v1_bridge_consts {
    ($f:ident, $a:ident, $n:ident, $c:ident;
     $(family $cname:ident $module:literal $lookup_fn:ident $arm_macro:ident {
         $(arm $id:tt { $lit:literal } => $body:expr ,)*
     })*) => {
        $( pub(crate) const $cname: &[&str] = &[$($lit),*]; )*
    };
}

v1_bridge_family_arms!(v1_bridge_consts, func_name, args, node, ctx);

macro_rules! v1_bridge_dispatch {
    ($f:ident, $a:ident, $n:ident, $c:ident;
     $(family $cname:ident $module:literal $lookup_fn:ident $arm_macro:ident {
         $(arm $id:tt { $lit:literal } => $body:expr ,)*
     })*) => {
        $(
            if is_v4_bridge_family($c, &$f, $cname, $module) {
                return match $crate::v1_interpreter_dispatch_generated::$lookup_fn(&$f) {
                    Some(arm) => match arm {
                        $( $arm_macro!($id) => $body , )*
                    },
                    None => unreachable!("bridge fn set mismatch: {}", $module),
                };
            }
        )*
    };
}

/// One guard for every bridge family.
/// predicates that differed only in their name list and module string.
fn is_v4_bridge_family(ctx: &InterpContext, func_name: &str, names: &[&str], module: &str) -> bool {
    if !names.contains(&func_name) {
        return false;
    }
    ctx.item_registry
        .get(func_name)
        .is_some_and(|info| info.module_name == module)
}

/// Handler bodies for v2.std.collection map grounding. Roster authority is
/// `v1_interpreter_authored_roster_arms()`; generated lookup routes spellings
/// before this macro matches on the generated enum variant.
macro_rules! v1_map_grounding_arms {
    ($cb:ident, $fname:ident) => {
        $cb! {
            $fname;
            arm "map_grounding.empty_map" { "empty_map_primitive_delegate" | "empty_map" } => "empty_map",
            arm "map_grounding.map_insert" { "map_insert" } => "map_insert",
        }
    };
}

/// Expansion 1: the name list the guard predicate tests.
macro_rules! v1_map_grounding_names {
    ($f:ident; $(arm $id:tt { $($lit:literal)|+ } => $body:expr ,)*) => {
        const STD_COLLECTION_MAP_GROUNDED_FNS: &[&str] = &[$($($lit),+),*];
    };
}

v1_map_grounding_arms!(v1_map_grounding_names, name);

/// Expansion 2: the spelling -> builtin mapping (R1: roster-generated lookup).
macro_rules! v1_map_grounding_dispatch {
    ($f:ident; $(arm $id:tt { $($lit:literal)|+ } => $body:expr ,)*) => {
        match $crate::v1_interpreter_dispatch_generated::lookup_try_v2_std_collection_map_primitive_grounding($f) {
            Some(arm) => match arm {
                $( try_v2_std_collection_map_primitive_grounding_arm!($id) => $body , )*
            },
            None => return None,
        }
    };
}

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
    let grounded_name = fn_node.name.as_str();
    let builtin_name = v1_map_grounding_arms!(v1_map_grounding_dispatch, grounded_name);
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

/// Handler bodies for native fold intercepts (run before free-call dispatch).
/// Roster authority is `v1_interpreter_authored_roster_arms()`; generated
/// `lookup_eval_call_native_intercept` routes spellings before this macro
/// matches on the generated enum variant.
macro_rules! v1_native_intercept_arms {
    ($cb:ident, $fname:ident, $args:ident, $env:ident, $ctx:ident) => {
        $cb! {
            $fname, $args, $env, $ctx;
            arm "native_intercept.fold_list" { "fold_list" } =>
                return eval_fold_list_native(&$args, $env, $ctx),
            arm "native_intercept.fold_list_right" { "fold_list_right" } =>
                return eval_fold_list_right_native(&$args, $env, $ctx),
        }
    };
}

macro_rules! v1_native_intercept_dispatch {
    ($f:ident, $a:ident, $e:ident, $c:ident; $(arm $id:tt { $lit:literal } => $body:expr ,)*) => {
        match $crate::v1_interpreter_dispatch_generated::lookup_eval_call_native_intercept(&$f) {
            Some(arm) => match arm {
                $( eval_call_native_intercept_arm!($id) => $body , )*
            },
            None => {}
        }
    };
}

fn eval_call(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let func_name = {
        let key = Rc::as_ptr(node) as usize;
        let existing = ctx.call_func_name_cache.borrow().get(&key).cloned();
        if let Some(s) = existing {
            s
        } else {
            let fresh = expr_call_func_at(node.clone(), ctx.si());
            ctx.call_func_name_cache_keepalive
                .borrow_mut()
                .push(node.clone());
            ctx.call_func_name_cache
                .borrow_mut()
                .insert(key, fresh.clone());
            fresh
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

    v1_bridge_family_arms!(v1_bridge_dispatch, func_name, args, node, ctx);

    v1_native_intercept_arms!(v1_native_intercept_dispatch, func_name, args, env, ctx);

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

    if let Some(result) = try_witness_evaluation_dispatch(ctx, node, &fn_node, &args, env) {
        return result;
    }

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
    if let Some(v) = try_cross_claim_pure_memo(ctx, fn_node, func_name, args) {
        return Ok(v);
    }
    let trace_on = eval_recompute_trace_enabled();
    let memo_on = ctx.eval_call_memo.borrow().enabled;
    if !trace_on && !memo_on {
        let effects_before = ctx.effect_dispatch_count.get();
        let result = call_function(ctx, fn_node, args, env);
        if let Ok(v) = &result {
            if ctx.effect_dispatch_count.get() == effects_before {
                store_cross_claim_pure_memo(ctx, fn_node, func_name, args, v);
            }
        }
        return result;
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
    if let Ok(v) = &result {
        if ctx.effect_dispatch_count.get() == effects_before {
            store_cross_claim_pure_memo(ctx, fn_node, func_name, args, v);
        }
    }
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
        Value::Str(s) => s.to_string(),
        _ => return None,
    };
    let token_stream_digest = match ctx.field(table_fields, "token_stream_digest")? {
        Value::Str(s) => s.to_string(),
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
        Some(Value::Str(s)) => ctx.sym(s.as_ref()),
        _ => return None,
    };
    Some((grammar_digest, token_stream_digest, position, production))
}

/// Handler bodies for parse-table memo dispatch. Roster authority is
/// `v1_interpreter_authored_roster_arms()`; generated lookup routes spellings
/// before this macro matches on the generated enum variant.
macro_rules! v1_parse_table_arms {
    ($cb:ident, $func_name:ident, $ctx:ident, $fn_node:ident, $args:ident, $env:ident) => {
        $cb! {
            $func_name, $ctx, $fn_node, $args, $env;
                arm "parse_table_memo.parse_table_lookup" { "parse_table_lookup" } => {
                    let positional: Vec<&Value> = $args.iter().map(|(_, v)| v).collect();
                    let [table, key] = match positional.as_slice() {
                        [table, key] => [table, key],
                        _ => return Ok(None),
                    };
                    let Some(memo_key) = parse_table_memo_scope_and_key($ctx, table, key) else {
                        return Ok(None);
                    };
                    let allows_memo = parse_table_materialization_allows_memo($ctx, table);
                    let mut st = $ctx.parse_table_memo.borrow_mut();
                    st.lookups += 1;
                    if allows_memo {
                        if let Some(v) = st.map.get(&memo_key).cloned() {
                            st.hits += 1;
                            drop(st);
                            record_parse_memo_lookup(&memo_key, true);
                            return Ok(Some(witness_holds(v, $ctx)));
                        }
                    }
                    drop(st);
                    record_parse_memo_lookup(&memo_key, false);
                    let result = call_function($ctx, $fn_node, $args, $env)?;
                    Ok(Some(result))
                },
                arm "parse_table_memo.parse_table_insert" { "parse_table_insert" } => {
                    let positional: Vec<&Value> = $args.iter().map(|(_, v)| v).collect();
                    let [table, key, value] = match positional.as_slice() {
                        [table, key, value] => [table, key, value],
                        _ => return Ok(None),
                    };
                    let result = call_function($ctx, $fn_node, $args, $env)?;
                    if parse_table_materialization_allows_memo($ctx, table) {
                        if let Some(memo_key) = parse_table_memo_scope_and_key($ctx, table, key) {
                            let mut st = $ctx.parse_table_memo.borrow_mut();
                            st.keepalive.push((*table).clone());
                            st.keepalive.push((*key).clone());
                            st.keepalive.push((*value).clone());
                            st.map.insert(memo_key, (*value).clone());
                            st.inserts += 1;
                        }
                    }
                    Ok(Some(result))
                },
        }
    };
}

/// Expansion 1: the dispatch.
macro_rules! v1_parse_table_dispatch {
    ($f:ident, $c:ident, $n:ident, $a:ident, $e:ident; $(arm $id:tt { $lit:literal } => $body:expr ,)*) => {
        match $crate::v1_interpreter_dispatch_generated::lookup_try_parse_table_memo_dispatch(&$f) {
            Some(arm) => match arm {
                $( try_parse_table_memo_dispatch_arm!($id) => $body , )*
            },
            None => Ok(None),
        }
    };
}

fn try_parse_table_memo_dispatch(
    ctx: &InterpContext,
    func_name: &str,
    fn_node: &Rc<Node>,
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
) -> InterpResult<Option<Value>> {
    v1_parse_table_arms!(v1_parse_table_dispatch, func_name, ctx, fn_node, args, env)
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
        // The declared-expectation edge is read off THIS node — the call site — and
        // partitioned out before anything becomes a bound param. See
        // EFFECT_EXPECTATION_ARG for why the call node is the only correct home.
        let mut named_args: Vec<(Option<String>, Value)> = Vec::new();
        let mut declared_expectation: Option<ExpectedOutcome> = None;
        for a in extra_args.iter() {
            let name = arg_name_at(a.clone(), ctx.si());
            let val = eval_expr(&arg_value(a.clone()), env, ctx)?;
            if name.as_deref() == Some(EFFECT_EXPECTATION_ARG) {
                declared_expectation = Some(expectation_from_declared_arg(&val)?);
                continue;
            }
            named_args.push((name, val));
        }
        return eval_service_call(
            service_name,
            &method_name,
            &named_args,
            env,
            ctx,
            match declared_expectation {
                Some(e) => ExpectationDeclaration::Declared(e),
                None => ExpectationDeclaration::UntracedDefault,
            },
        );
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
        let raw = raw_map_lookup(&receiver_val, key, env, ctx)?;
        return Ok(map_lookup_as_optional(raw, ctx));
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
        Value::Map(_) => raw_map_lookup(value, &str_value(field.to_string()), env, ctx)
            .map(RawMapLookup::into_raw),
        _ => Err(InterpError::TypeError {
            msg: format!("cannot access field '{}' on {}", field, value.type_label()),
        }),
    }
}

/// HAND-RUST GATE explicit deferral (review 50372), covering this function and the
/// keyed-collection branch it is dispatched from in `eval_record_lit`: bounded growth
/// in the existing seed interpreter, not a new Rust authority. Every DECISION here is
/// modeled and read back out of `.dag` — whether the literal is a keyed collection is
/// `04_types` `node_is_keyed_collection`, whether its keys may be the authored field
/// names is `05_emit_rust` `map_literal_key_is_string`, and both are the SAME functions
/// the emitter consults about the same literal, which is the point: the seed is not
/// deciding anything, it is projecting one modeled decision onto the interpreter's own
/// `Value` representation. Removing this code without grounding that representation
/// would reopen the fork it closes — infer saying map, eval building a record.
///
/// Lane: ROADMAP `v1-interpreter-quarantine` → `v1-interpreter-delete`, counted against
/// `v1-honest-frontier`; the underlying class is DESIGN's model↔realization fork thread
/// (every primitive modeled as a coproduct and realized as a native `Value`, reconciled
/// by per-site bridges), of which this is one bridge repaired rather than added.
///
/// Checkable receipt, by execution: `w_map_typed_literal_is_a_map` in
/// `src/v1/tests/claim/ordinary_frontend_observation_test.dag` goes RED without this
/// code (`map_keys expects a map, got Record` — the refusal that made the ordinary front
/// end unreachable), and `w_record_literal_is_still_a_record` goes RED if it
/// over-converts. Both are enrolled on the v1 claim scoped roster, so the deferral is
/// counted rather than asserted.
///
/// Deletion condition, narrower than the lane's: when a brace literal's representation
/// is DERIVED from its inferred type rather than reconstructed per consumer — the
/// grounding half of the model↔realization thread, the same move `#5428` made for the
/// numeric tower — this function has nothing left to project and deletes outright. The
/// witness above is then REPLACED by one over the grounded representation, not retired.
///
/// Build the `Value::Map` a keyed-collection literal denotes. Keys are the
/// authored field names, and string-likeness must be POSITIVELY established
/// before they may be: the test is `map_literal_key_is_string`, the very
/// function `05_emit_rust` asks about the same literal when it decides whether
/// to render the key quoted-and-owned or bare, so the interpreter and the
/// emitter cannot disagree about one literal's keys. Anything it does not
/// establish is a typed, located refusal rather than a guessed key — a
/// deny-list of known-bad key types would let every unlisted one through, which
/// is the partial refusal that later fails open (DESIGN §5, and codex review
/// 50168 which caught exactly that shape here). The refusal arm itself has no
/// witness: a refusing data initializer stops module evaluation rather than
/// returning a Bool, so it cannot be an ordinary green arm. That is
/// can-climb-now-but-unbuilt, not cannot-climb — the trigger is an
/// expecting-red quarantine probe declaring a non-string-keyed map literal,
/// the mechanism named beside the witnesses in
/// `src/v1/tests/claim/ordinary_frontend_observation_test.dag`.
fn eval_map_lit(
    node: &Rc<Node>,
    map_type: &Rc<Node>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    if !crate::v1_compiler_emit_rust::map_literal_key_is_string(map_type.clone(), ctx.si()) {
        let key_name = match map_type.children.iter().next() {
            Some(key_type) => authored_name_at(
                ctx.si(),
                crate::v1_compiler_infer_types::normalize_access_type_node(key_type.clone()),
            ),
            None => String::new(),
        };
        return Err(InterpError::TypeError {
            msg: format!(
                "map literal key type '{}' is not established as string-like, so the authored \
                 field names cannot be its keys; declared at {}:{}",
                key_name, node.span.file, node.span.start
            ),
        });
    }
    let mut entries = HamtMap::new();
    for child in node.children.iter() {
        let fname = field_init_node_name_at(child.clone(), ctx.si());
        let fval = eval_expr(&field_init_node_value(child.clone()), env, ctx)?;
        match CanonKey::new(str_value(fname.clone())) {
            Some(ck) => {
                entries = entries.update(ck, fval);
            }
            None => {
                return Err(InterpError::TypeError {
                    msg: format!("map literal key '{}' is not a valid map key", fname),
                })
            }
        }
    }
    Ok(map_value(entries))
}

fn eval_record_lit(
    node: &Rc<Node>,
    parent_enum: Option<&str>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let type_name = record_lit_type_name_at(node.clone(), ctx.si()).unwrap_or_default();

    // A brace literal in a keyed-collection position IS a map, and the single
    // authority for that fact is the literal's own inferred type — the same
    // `node_is_keyed_collection` relation `05_emit_rust` reads when it renders
    // the identical literal as a `HashMap`. Reading it here is what stops the
    // interpreter and the emitter disagreeing about one value's representation
    // (DESIGN §3/§5: one authority, and the representation derived from it
    // rather than reconciled per consumer). Before this, the interpreter built
    // a `Record`, `map_get` limped through `raw_map_lookup`'s Record arm, and
    // `map_keys(kernel_type_set)` refused — so the whole ordinary front end was
    // unreachable from interpreted `.dag`.
    if type_name.is_empty() {
        if let Some(InferredNode::Resolved { node: ty, .. }) = node.inferred.as_deref() {
            if crate::v1_compiler_infer_types::node_is_keyed_collection(ty.clone(), ctx.si()) {
                return eval_map_lit(node, ty, env, ctx);
            }
        }
    }

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
        if type_name == "GroupCompletion" {
            if let (Some(Value::Int(pos)), Some(Value::Int(neg))) = (
                fields_get(&fields, ctx.sym("pos")),
                fields_get(&fields, ctx.sym("neg")),
            ) {
                return Ok(Value::Int(pos - neg));
            }
        }
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
    Ok(str_value(result))
}

fn lookup_type_item_across_modules(ctx: &InterpContext, type_name: &str) -> Option<Rc<Node>> {
    if eval_profile_enabled() {
        TYPE_LOOKUP_CALLS.with(|c| c.set(c.get() + 1));
    }
    let existing = ctx.type_item_index.borrow().clone();
    let index = match existing {
        Some(index) => index,
        None => {
            let mut built: HashMap<String, Rc<Node>> = HashMap::new();
            for module in ctx.modules.iter() {
                for item in module.items.iter() {
                    if eval_profile_enabled() {
                        TYPE_LOOKUP_ITEMS.with(|c| c.set(c.get() + 1));
                    }
                    let name = authored_name_at(ctx.si(), item.clone());
                    if name.is_empty() {
                        continue;
                    }
                    // First declaration wins, matching the scan's early return.
                    built.entry(name).or_insert_with(|| item.clone());
                }
            }
            let built = Rc::new(built);
            *ctx.type_item_index.borrow_mut() = Some(built.clone());
            built
        }
    };
    index.get(type_name).cloned()
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

/// Both authored names a cast needs for its target, resolved once per target node.
struct CastTargetNames {
    seed: String,
    kernel: String,
}

/// Resolve-and-memoize the cast target's authored names. Sound because both are functions of
/// the target node and `ctx.modules` / `ctx.source_indices`, all fixed for the ctx's lifetime.
fn cast_target_names(ctx: &InterpContext, target: Rc<Node>) -> Rc<CastTargetNames> {
    let key = Rc::as_ptr(&target) as usize;
    let existing = ctx.cast_kernel_cache.borrow().get(&key).cloned();
    if let Some(hit) = existing {
        // A pointer-keyed memo's one silent-wrongness class is address reuse: if a cached node
        // were freed and a new node landed at the same address, this would answer a cast with
        // ANOTHER type's name — no crash, just a wrong type. On the normal path that is
        // unreachable (cast_target returns an AST-owned child, alive for the ctx's lifetime),
        // but expr_child_at SYNTHESIZES an error node when a child is missing, and that node is
        // temporary. Rather than inherit the keepalive discipline by imitation, GUNBC_MEMO_VERIFY=1
        // recomputes the uncached answer on every hit and REFUSES on divergence, so the class is
        // checked by execution over the real corpus instead of assumed.
        if memo_verify_enabled() {
            let recomputed_seed = cast_target_seed_name_uncached(ctx, target.clone());
            if recomputed_seed != hit.seed {
                panic!(
                    "cast memo divergence at key {key:#x}: cached seed {:?} != recomputed {:?} \
                     — pointer-keyed cache collision (keepalive failed to retain the node)",
                    hit.seed, recomputed_seed
                );
            }
        }
        return hit;
    }
    let seed = cast_target_seed_name_uncached(ctx, target.clone());
    let kernel = cast_target_underlying_kernel_uncached(ctx, seed.clone());
    let fresh = Rc::new(CastTargetNames { seed, kernel });
    // DEGENERATE RESOLUTIONS ARE NEVER CACHED, and this bound is load-bearing rather than
    // tidiness. expr_child_at falls back to make_expr_error_node for a malformed cast, which
    // Rc::new's a FRESH node per call (name "", ident_span None, inferred CompilerError). That
    // resolves to an empty seed, and eval_cast's identity arm then returns a Value::Str
    // unchanged on an empty kernel — so the run does NOT terminate, and a malformed cast inside
    // a loop would allocate a new address every iteration: permanent cache miss, unbounded
    // keepalive growth. Skipping the insert bounds the cache by the AST's real cast nodes.
    // It costs only recomputation on a path that short-circuits before any module scan, and an
    // empty seed is exactly the signature of a target carrying no resolvable authored name.
    if fresh.seed.is_empty() {
        return fresh;
    }
    ctx.cast_kernel_cache_keepalive
        .borrow_mut()
        .push(target.clone());
    ctx.cast_kernel_cache
        .borrow_mut()
        .insert(key, fresh.clone());
    fresh
}

fn cast_target_seed_name(ctx: &InterpContext, target: Rc<Node>) -> String {
    cast_target_names(ctx, target).seed.clone()
}

fn cast_target_underlying_kernel(ctx: &InterpContext, target: Rc<Node>) -> String {
    cast_target_names(ctx, target).kernel.clone()
}

fn cast_target_seed_name_uncached(ctx: &InterpContext, target: Rc<Node>) -> String {
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

fn cast_target_underlying_kernel_uncached(ctx: &InterpContext, seed: String) -> String {
    if eval_profile_enabled() {
        CAST_KERNEL_CALLS.with(|c| c.set(c.get() + 1));
    }
    let mut current = seed;
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

/// The cast's SOURCE type name. Same defect as the target side and the same repair: this is a
/// pure function of the expression node, but was re-extracting authored source text on every
/// evaluation. Measured on one daily-page render: +27.1ms across 59,858 casts (~452ns each,
/// ExprCast 42.8ms -> 69.9ms) once #8098 added this second `authored_name_at` beside the
/// target-side one. Memoized per expression node under the same pointer-key + keepalive
/// discipline as `cast_kernel_cache`.
fn cast_expr_inferred_type_name(ctx: &InterpContext, expr: Rc<Node>) -> String {
    let key = Rc::as_ptr(&expr) as usize;
    let existing = ctx.cast_source_name_cache.borrow().get(&key).cloned();
    if let Some(hit) = existing {
        if memo_verify_enabled() {
            let recomputed = cast_expr_inferred_type_name_uncached(ctx, expr.clone());
            if recomputed != hit {
                panic!(
                    "cast source-name memo divergence at key {key:#x}: cached {:?} != recomputed {:?} \
                     — pointer-keyed cache collision (keepalive failed to retain the node)",
                    hit, recomputed
                );
            }
        }
        return hit;
    }
    let fresh = cast_expr_inferred_type_name_uncached(ctx, expr.clone());
    // Same unbounded-growth bound as cast_target_names: an error node's inferred is
    // CompilerError rather than Resolved, so it yields "" — never cache it, or a malformed
    // cast in a loop grows this keepalive without bound on freshly allocated nodes.
    if fresh.is_empty() {
        return fresh;
    }
    ctx.cast_source_name_cache_keepalive
        .borrow_mut()
        .push(expr.clone());
    ctx.cast_source_name_cache
        .borrow_mut()
        .insert(key, fresh.clone());
    fresh
}

fn cast_expr_inferred_type_name_uncached(ctx: &InterpContext, expr: Rc<Node>) -> String {
    match expr.inferred.as_deref() {
        Some(InferredNode::Resolved { node }) => authored_name_at(ctx.si(), node.clone()),
        _ => String::new(),
    }
}

/// Runtime identity casts mirror `validate_cast`'s `source_name == target_name` arm,
/// plus String-valued casts to types whose alias chain grounds on `String`.
fn cast_identity_result(
    val: &Value,
    ctx: &InterpContext,
    source_name: &str,
    target_node: Rc<Node>,
    target_name: &str,
) -> Option<Value> {
    if !source_name.is_empty() && source_name == target_name {
        return Some(val.clone());
    }
    if let Value::Str(s) = val {
        let kernel = cast_target_underlying_kernel(ctx, target_node);
        if kernel.is_empty() || kernel == "String" {
            return Some(Value::Str(s.clone()));
        }
    }
    None
}

fn eval_cast(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let inner = cast_expr(node.clone());
    let source_name = cast_expr_inferred_type_name(ctx, inner.clone());
    let val = eval_expr(&inner, env, ctx)?;
    let target_node = cast_target(node.clone());
    let target_name = cast_target_seed_name(ctx, target_node.clone());

    if let Some(v) =
        cast_identity_result(&val, ctx, &source_name, target_node.clone(), &target_name)
    {
        return Ok(v);
    }

    match target_name.as_str() {
        "Float" => match val {
            Value::Int(n) => Ok(Value::Float(n as f64)),
            v => Err(InterpError::TypeError {
                msg: format!("cannot cast {} to Float", v.type_label()),
            }),
        },
        "Int" => match val {
            Value::Float(n) => Ok(Value::Int(n as i64)),
            v => Err(InterpError::TypeError {
                msg: format!("cannot cast {} to Int", v.type_label()),
            }),
        },
        "String" => match val {
            Value::Int(n) => Ok(str_value(n.to_string())),
            Value::Float(n) => Ok(str_value(n.to_string())),
            Value::Bool(b) => Ok(str_value(b.to_string())),
            Value::Str(s) => Ok(Value::Str(Rc::clone(&s))),
            // Corpus wire/debug casts for structured values — not the blanket Display
            // fallback that silently stringified List/Map (§5 fabricated plausible output).
            Value::Variant { .. } | Value::Record { .. } => Ok(str_value(format!("{}", val))),
            v => Err(InterpError::TypeError {
                msg: format!("cannot cast {} to String", v.type_label()),
            }),
        },
        t => Err(InterpError::TypeError {
            msg: format!("cannot cast {} to {}", val.type_label(), t),
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
        (base, key) if is_map_lookup_receiver(base) => {
            raw_map_lookup(base, key, env, ctx).map(RawMapLookup::into_raw)
        }
        (Value::Str(s), Value::Int(i)) => {
            let i = *i as usize;
            Ok(s.chars()
                .nth(i)
                .map(|c| str_value(c.to_string()))
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
            Ok(str_value(sliced))
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

/// Handler bodies for algebra method dispatch. Roster authority is
/// `v1_interpreter_authored_roster_arms()`; generated `lookup_eval_algebra_method_inner`
/// routes spellings before this macro matches on the generated enum variant.
macro_rules! v1_algebra_method_arms {
    ($cb:ident, $method:ident, $receiver:ident, $args:ident, $env:ident, $ctx:ident) => {
        $cb! {
            $method, $receiver, $args, $env, $ctx;
            arm "method_call.lookup" { "lookup" } => {
                let key = $args.first().ok_or_else(|| InterpError::TypeError {
                    msg: "lookup requires a key argument".to_string(),
                })?;
                raw_map_lookup(&$receiver, key, $env, $ctx).map(RawMapLookup::into_raw)
            },

            arm "method_call.map" { "map" } => list_method_with_closure("map", $receiver, $args, $env, $ctx, |items, f, $env, $ctx| {
                items
                    .iter()
                    .map(|item| apply_closure(f, &[item.clone()], $env, $ctx))
                    .collect::<InterpResult<Vec<Value>>>()
                    .map(|v| list_value((v)))
            }),

            arm "method_call.filter" { "filter" } => {
                list_method_with_closure("filter", $receiver, $args, $env, $ctx, |items, f, $env, $ctx| {
                    let mut result = Vec::new();
                    for item in items.iter() {
                        let keep = apply_closure(f, &[item.clone()], $env, $ctx)?;
                        if keep.is_truthy() {
                            result.push(item.clone());
                        }
                    }
                    Ok(list_value((result)))
                })
            },

            arm "method_call.fold" { "fold" } => {
                let items = expect_list(&$receiver, "fold")?;
                let (init, f) = match $args {
                    [init, f] => (init.clone(), f),
                    _ => {
                        return Err(InterpError::TypeError {
                            msg: "fold requires (init, f) arguments".to_string(),
                        })
                    }
                };
                let mut acc = init;
                for item in items.iter() {
                    acc = apply_closure(f, &[acc, item.clone()], $env, $ctx)?;
                }
                Ok(acc)
            },

            arm "method_call.flat_map" { "flat_map" } => list_method_with_closure(
                "flat_map",
                $receiver,
                $args,
                $env,
                $ctx,
                |items, f, $env, $ctx| {
                    let mut result = Vec::new();
                    for item in items.iter() {
                        let mapped = apply_closure(f, &[item.clone()], $env, $ctx)?;
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

            arm "method_call.any" { "any" } => list_method_with_closure("any", $receiver, $args, $env, $ctx, |items, f, $env, $ctx| {
                for item in items.iter() {
                    if apply_closure(f, &[item.clone()], $env, $ctx)?.is_truthy() {
                        return Ok(Value::Bool(true));
                    }
                }
                Ok(Value::Bool(false))
            }),

            arm "method_call.all" { "all" } => list_method_with_closure("all", $receiver, $args, $env, $ctx, |items, f, $env, $ctx| {
                for item in items.iter() {
                    if !apply_closure(f, &[item.clone()], $env, $ctx)?.is_truthy() {
                        return Ok(Value::Bool(false));
                    }
                }
                Ok(Value::Bool(true))
            }),

            arm "method_call.sort_by" { "sort_by" } => {
                list_method_with_closure("sort_by", $receiver, $args, $env, $ctx, |items, f, $env, $ctx| {
                    let mut keyed: Vec<(Value, Value)> = items
                        .iter()
                        .map(|item| {
                            let key = apply_closure(f, &[item.clone()], $env, $ctx)?;
                            Ok((key, item.clone()))
                        })
                        .collect::<InterpResult<_>>()?;
                    keyed.sort_by(|(ka, _), (kb, _)| cmp_values(ka, kb));
                    Ok(list_value(
                        keyed.into_iter().map(|(_, v)| v).collect::<Vec<_>>(),
                    ))
                })
            },

            arm "method_call.list_push" { "list_push" } => {
                if matches!(&$receiver, Value::Str(_)) {
                    return Err(InterpError::TypeError {
                        msg: "list_push not supported on String".to_string(),
                    });
                }
                let item = $args.first().cloned().unwrap_or(Value::Null);
                match value_to_list_carrier(&$receiver) {
                    Some((items, copied)) => {
                        let mut counters = $ctx.mutation_counters.borrow_mut();
                        counters.list_push_calls += 1;
                        counters.list_push_items_copied += copied;
                        drop(counters);
                        let mut result = (*items).clone();
                        result.push_back(item);
                        Ok(list_value(result))
                    }
                    None => Err(InterpError::TypeError {
                        msg: format!("list_push on non-list: {}", $receiver.type_label()),
                    }),
                }
            },

            arm "method_call.concat" { "concat" | "append" | "push" } => {
                if let Value::Str(s) = &$receiver {
                    let mut result = s.to_string();
                    for arg in $args {
                        result.push_str(&format!("{}", arg));
                    }
                    return Ok(str_value(result));
                }
                // String grounding (model↔realization): when a native String arg
                // participates, the whole `concat` is a String and realizes as one
                // native `Value::Str` — provided the receiver is itself string-like
                // (all-codepoint). A `List<String>` receiver (`Str` *elements*) is
                // rejected by `free_monoid_to_string` and falls through to the list
                // path below, so `["a","b"].concat("c")` stays a list.
                if $method == "concat" && $args.iter().any(|a| matches!(a, Value::Str(_))) {
                    if let Some(base) = free_monoid_to_string(&$receiver) {
                        if let Some(rest) = $args
                            .iter()
                            .map(free_monoid_to_string)
                            .collect::<Option<Vec<_>>>()
                        {
                            return Ok(str_value(format!("{}{}", base, rest.concat())));
                        }
                    }
                }
                if let Ok(items) = expect_list(&$receiver, "concat") {
                    // Fail-closed backstop (DESIGN §5): a native String arg meeting a
                    // codepoint-bearing `Cons`-chain receiver here is the
                    // model↔realization straddle that grounding above did not
                    // dissolve — refuse loudly rather than push the `Str` into a
                    // mixed `[codepoint.., Str]` list. A `Value::List` receiver is a
                    // generic collection (`[1].append("ab")` is a legitimate
                    // two-element list), and a homogeneous `List<String>` carries no
                    // codepoint — both pass (the `orig` representation guard).
                    if $args.iter().any(|a| matches!(a, Value::Str(_))) {
                        let snapshot: Vec<Value> = items.iter().cloned().collect();
                        if let Some(detail) = string_realization_straddle_detail(&$receiver, &snapshot) {
                            return Err(InterpError::StringRealizationStraddle { detail });
                        }
                    }
                    let mut result = (*items).clone();
                    let mut merged_items = 0usize;
                    let mut copied_items = 0usize;
                    for arg in $args {
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
                    let mut counters = $ctx.mutation_counters.borrow_mut();
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
                    msg: format!("cannot concat on {}", $receiver.type_label()),
                })
            },

            arm "method_call.length" { "length" | "count" | "size" } => match native_len(&$receiver) {
                Some(n) => Ok(Value::Int(n)),
                None => match free_monoid_to_vec(&$receiver) {
                    Some(items) => Ok(Value::Int(items.len() as i64)),
                    None => match &$receiver {
                        Value::Map(m) => Ok(Value::Int(m.len() as i64)),
                        _ => Err(InterpError::TypeError {
                            msg: format!("cannot get length of {}", $receiver.type_label()),
                        }),
                    },
                },
            },

            // Known-method bridge parity: infer rewrites bare `is_empty(xs)` on
            // import-stripped modules into a method call (the census never serves
            // algebra template names), so eval must implement the same member the
            // bridge targets — emptiness via the shared length authority above.
            arm "method_call.is_empty" { "is_empty" } => match native_len(&$receiver) {
                Some(n) => Ok(Value::Bool(n == 0)),
                None => match free_monoid_to_vec(&$receiver) {
                    Some(items) => Ok(Value::Bool(items.is_empty())),
                    None => match &$receiver {
                        Value::Map(m) => Ok(Value::Bool(m.is_empty())),
                        _ => Err(InterpError::TypeError {
                            msg: format!("cannot check is_empty of {}", $receiver.type_label()),
                        }),
                    },
                },
            },

            arm "method_call.first" { "first" } => {
                let items = expect_list(&$receiver, "first")?;
                Ok(items.front().cloned().unwrap_or(Value::Null))
            },

            arm "method_call.last" { "last" } => {
                let items = expect_list(&$receiver, "last")?;
                Ok(items.last().cloned().unwrap_or(Value::Null))
            },

            arm "method_call.reverse" { "reverse" } => {
                let items = expect_list(&$receiver, "reverse")?;
                Ok(list_value(items.iter().rev().cloned().collect::<Vec<_>>()))
            },

            arm "method_call.skip" { "skip" } => {
                let items = expect_list(&$receiver, "skip")?;
                let n = expect_int($args.first(), "skip")?;
                Ok(list_value(
                    items.iter().skip(n as usize).cloned().collect::<Vec<_>>(),
                ))
            },

            arm "method_call.take" { "take" } => {
                let items = expect_list(&$receiver, "take")?;
                let n = expect_int($args.first(), "take")?;
                Ok(list_value(
                    items.iter().take(n as usize).cloned().collect::<Vec<_>>(),
                ))
            },

            arm "method_call.enumerate" { "enumerate" } => {
                let items = expect_list(&$receiver, "enumerate")?;
                let result: Vec<Value> = items
                    .iter()
                    .enumerate()
                    .map(|(i, v)| Value::Record {
                        type_name: $ctx.sym("Pair"),
                        fields: Rc::new(sorted_fields(vec![
                            ($ctx.sym("first"), Value::Int(i as i64)),
                            ($ctx.sym("second"), v.clone()),
                        ])),
                    })
                    .collect();
                Ok(list_value((result)))
            },

            arm "method_call.contains" { "contains" | "has" } => match &$receiver {
                Value::Map(m) => {
                    let key = $args.first().ok_or_else(|| InterpError::TypeError {
                        msg: "contains requires a key argument".to_string(),
                    })?;
                    match CanonKey::new(key.clone()) {
                        Some(ck) => Ok(Value::Bool(m.contains_key(&ck))),
                        None => Ok(Value::Bool(false)),
                    }
                }
                Value::Str(s) => {
                    let sub = expect_str($args.first(), "contains")?;
                    Ok(Value::Bool(s.contains(&sub)))
                }
                _ => match expect_list(&$receiver, "contains") {
                    Ok(items) => {
                        let target = $args.first().cloned().unwrap_or(Value::Null);
                        Ok(Value::Bool(items.iter().any(|item| *item == target)))
                    }
                    Err(_) => Err(InterpError::TypeError {
                        msg: format!("contains not supported on {}", $receiver.type_label()),
                    }),
                },
            },

            arm "method_call.join" { "join" } => {
                let items = expect_list(&$receiver, "join")?;
                let sep = $args.first().map(|v| format!("{}", v)).unwrap_or_default();
                let strs: Vec<String> = items.iter().map(|v| format!("{}", v)).collect();
                Ok(str_value(strs.join(&sep)))
            },

            arm "method_call.chars" { "chars" } => {
                // §6 residue: this materializes a string as a `Value::List` of
                // codepoint `Int`s, indistinguishable at the Value level from a
                // generic `Int` list. That is the named hole in the String-straddle
                // wall — see `string_realization_straddle_detail`'s `Value::List`
                // exemption. Closed by regrounding `Char`/codepoint-sequence so the
                // realization is distinguishable (grounding root, sibling #5428).
                let s = expect_str(Some(&$receiver), "chars")?;
                let items: Vec<Value> = s.chars().map(|c| Value::Int(c as i64)).collect();
                Ok(list_value(items))
            },

            arm "method_call.map_get" { "map_get" } => {
                let key = $args.first().ok_or_else(|| InterpError::TypeError {
                    msg: "map_get requires a key argument".to_string(),
                })?;
                let raw = raw_map_lookup(&$receiver, key, $env, $ctx)?;
                Ok(map_lookup_as_optional(raw, $ctx))
            },

            arm "method_call.get" { "get" } => {
                if matches!(&$receiver, Value::Str(_)) {
                    let key = $args.first().ok_or_else(|| InterpError::TypeError {
                        msg: "get requires a key argument".to_string(),
                    })?;
                    raw_map_lookup(&$receiver, key, $env, $ctx).map(RawMapLookup::into_raw)
                } else if let Ok(items) = expect_list(&$receiver, "get") {
                    let idx = expect_int($args.first(), "get")?;
                    Ok(list_get_at_or_null(&items, idx))
                } else {
                    let key = $args.first().ok_or_else(|| InterpError::TypeError {
                        msg: "get requires a key argument".to_string(),
                    })?;
                    raw_map_lookup(&$receiver, key, $env, $ctx).map(RawMapLookup::into_raw)
                }
            },

            // These 4 arms were absent here but present in the free-function builtin dispatch --
            // eval_algebra_method (method/pipe calls) and that dispatch (direct calls) are two
            // surfaces over one builtin set that have diverged; they should be one authority.
            // Pure-eval logic, in scope of ROADMAP HAND kernel D (`v1_interpreter` pure-eval
            // dissolution, docs/plans/interpreter-kernel-d.md): dissolution trigger is the
            // pure-eval seam (`emit_host` transport wiring) grounding this dispatch into
            // `v2.compiler.eval`, at which point per-builtin arms stop being hand-Rust here.
            arm "method_call.map_keys" { "map_keys" } => {
                let m = expect_map(&$receiver, "map_keys")?;
                let keys: Vec<Value> = m.keys().map(|k| k.key.clone()).collect();
                Ok(list_value((keys)))
            },

            arm "method_call.map_values" { "map_values" } => {
                let m = expect_map(&$receiver, "map_values")?;
                let vals: Vec<Value> = m.values().cloned().collect();
                Ok(list_value((vals)))
            },

            arm "method_call.map_contains_key" { "map_contains_key" | "map_has" } => {
                let m = expect_map(&$receiver, "map_contains_key")?;
                let key = $args.first().ok_or_else(|| InterpError::TypeError {
                    msg: "map_contains_key requires a key argument".to_string(),
                })?;
                match CanonKey::new(key.clone()) {
                    Some(ck) => Ok(Value::Bool(m.contains_key(&ck))),
                    None => Ok(Value::Bool(false)),
                }
            },

            arm "method_call.map_is_empty" { "map_is_empty" } => {
                let m = expect_map(&$receiver, "map_is_empty")?;
                Ok(Value::Bool(m.is_empty()))
            },

            arm "method_call.insert" { "insert" | "map_insert" } => {
                let m = expect_map(&$receiver, "insert")?;
                let (key, val) = match $args {
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
                let mut counters = $ctx.mutation_counters.borrow_mut();
                counters.map_insert_calls += 1;
                drop(counters);
                Ok(map_value(m.update(ck, val)))
            },

            arm "method_call.merge" { "merge" } => {
                let base = expect_map(&$receiver, "merge")?;
                let overlay = expect_map($args.first().unwrap_or(&Value::Null), "merge")?;
                let mut counters = $ctx.mutation_counters.borrow_mut();
                counters.map_merge_calls += 1;
                drop(counters);
                Ok(map_value((*overlay).clone().union((*base).clone())))
            },

            arm "method_call.keys" { "keys" } => {
                let m = expect_map(&$receiver, "keys")?;
                let keys: Vec<Value> = m.keys().map(|k| k.key.clone()).collect();
                Ok(list_value((keys)))
            },

            arm "method_call.values" { "values" } => {
                let m = expect_map(&$receiver, "values")?;
                let vals: Vec<Value> = m.values().cloned().collect();
                Ok(list_value((vals)))
            },

            arm "method_call.replace" { "replace" } => {
                let s = expect_string(&$receiver, "replace")?;
                match $args {
                    [from, to] => {
                        let from_s = format!("{}", from);
                        let to_s = format!("{}", to);
                        Ok(str_value(s.replace(&from_s, &to_s)))
                    }
                    _ => Err(InterpError::TypeError {
                        msg: "replace requires (from, to) arguments".to_string(),
                    }),
                }
            },

            arm "method_call.split" { "split" } => {
                let s = expect_string(&$receiver, "split")?;
                let sep = expect_str($args.first(), "split")?;
                let parts: Vec<Value> = s.split(&sep).map(|p| str_value(p.to_string())).collect();
                Ok(list_value((parts)))
            },

            arm "method_call.trim" { "trim" } => {
                let s = expect_string(&$receiver, "trim")?;
                Ok(str_value(s.trim().to_string()))
            },

            arm "method_call.starts_with" { "starts_with" } => {
                let s = expect_string(&$receiver, "starts_with")?;
                let prefix = expect_str($args.first(), "starts_with")?;
                Ok(Value::Bool(s.starts_with(&prefix)))
            },

            arm "method_call.ends_with" { "ends_with" } => {
                let s = expect_string(&$receiver, "ends_with")?;
                let suffix = expect_str($args.first(), "ends_with")?;
                Ok(Value::Bool(s.ends_with(&suffix)))
            },

            arm "method_call.substring" { "substring" } => {
                let s = expect_string(&$receiver, "substring")?;
                match $args {
                    [start, end] => {
                        let s_idx = expect_int(Some(start), "substring start")? as usize;
                        let e_idx = expect_int(Some(end), "substring end")? as usize;
                        let sliced: String = s
                            .chars()
                            .skip(s_idx)
                            .take(e_idx.saturating_sub(s_idx))
                            .collect();
                        Ok(str_value(sliced))
                    }
                    _ => Err(InterpError::TypeError {
                        msg: "substring requires (start, end) arguments".to_string(),
                    }),
                }
            },

            arm "method_call.char_at" { "char_at" } => {
                let s = expect_string(&$receiver, "char_at")?;
                let idx = expect_int($args.first(), "char_at")?;
                Ok(s.chars()
                    .nth(idx as usize)
                    .map(|c| str_value(c.to_string()))
                    .unwrap_or(Value::Null))
            },

            arm "method_call.index_by" { "index_by" } => list_method_with_closure(
                "index_by",
                $receiver,
                $args,
                $env,
                $ctx,
                |items, f, $env, $ctx| {
                    let mut m = HamtMap::new();
                    for item in items.iter() {
                        let key = apply_closure(f, &[item.clone()], $env, $ctx)?;
                        let ck = CanonKey::new(key).ok_or_else(|| InterpError::TypeError {
                            msg: "index_by key is not a valid map key (closure/fn/NaN)".to_string(),
                        })?;
                        m.insert(ck, item.clone());
                    }
                    Ok(map_value(m))
                },
            ),

        }
    };
}

/// Dispatch via roster-generated lookup + exhaustive enum match (R1).
macro_rules! v1_algebra_dispatch {
    ($m:ident, $r:ident, $a:ident, $e:ident, $c:ident; $(arm $id:tt { $($lit:literal)|+ } => $body:expr ,)*) => {
        match $crate::v1_interpreter_dispatch_generated::lookup_eval_algebra_method_inner($m) {
            Some(arm) => match arm {
                $( eval_algebra_method_inner_arm!($id) => $body , )*
            },
            None => Err(InterpError::Unimplemented {
                what: format!("method '{}'", $m),
            }),
        }
    };
}

fn eval_algebra_method_inner(
    method: &str,
    receiver: Value,
    args: &[Value],
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    v1_algebra_method_arms!(v1_algebra_dispatch, method, receiver, args, env, ctx)
}

pub fn fixture_now_secs(ctx: &InterpContext) -> Result<u64, crate::recorded_fixture::FixtureError> {
    if ctx.indexes.service_ops.contains_key("Clock.UnixSecs") {
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
        ctx.indexes
            .service_ops
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
    // No call node here: this is the interpreter's own internal seam (Clock.UnixSecs
    // and friends), not a .dag call site, so it declares the default explicitly
    // rather than inheriting one.
    dispatch_service_wet(
        service_node,
        op_node,
        transport,
        &param_env,
        ctx,
        &key,
        ExpectedOutcome::ExpectSuccess,
    )
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

/// Native read of THIS process's own environment. Semantics match the former
/// `printenv` subprocess exactly: unset → None, empty → None, value trimmed.
/// `dispatch_service_wet` routes `shell.Env.Get` here for OnTarget locality
/// (shell-to-dag residual census §0b); the shell argv remains the remote handler.
fn wet_env_var(name: &str) -> Option<String> {
    let s = std::env::var(name).ok()?.trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// WHY THIS SEAM STAYS `ExpectSuccess` EVEN THOUGH ITS OUTCOME LOOKS LIKE DATA.
///
/// The consumer below swallows every failure arm into `None` (`_ => None`), which
/// is the classic tell that an exit code is an observation rather than a verdict
/// — the shape `OutcomeIsData` exists for. Re-arming it on that tell alone would
/// be wrong, and the reason is a state-space conflation one level down: `None`
/// here means BOTH "the variable is unset" (a legitimate answer) and "the
/// `shell.Env.Get` dispatch itself failed" (a broken probe). Annotating
/// `OutcomeIsData` would render the second case ambient, so a broken env service
/// would become indistinguishable from an empty environment — trading a crude
/// loud arm for a silent conflation, which is the worse of the two.
///
/// The correct fix is therefore a PAIR, not an annotation: split the consumer's
/// `None` into probe-broken (refuses) versus answer-absent (`None`), and only
/// then re-arm this site to `OutcomeIsData`. Recorded here rather than done here
/// because the split changes this function's return type and every caller's
/// handling, which is its own change with its own witnesses.
fn resolve_env_var_token(ctx: &InterpContext, var_name: &str) -> Option<String> {
    if ctx.indexes.service_ops.contains_key("shell.Env.Get") {
        let args = [(Some("name".to_string()), str_value(var_name.to_string()))];
        match eval_service_call(
            "shell.Env",
            "Get",
            &args,
            &Env::empty(),
            ctx,
            // Interpreter-own seam: no .dag call node exists to carry `expect:`,
            // so the arm is stated here rather than defaulted. See
            // `resolve_env_var_token_expectation_note` below for why this one is
            // ExpectSuccess and not OutcomeIsData despite the swallowing consumer.
            ExpectationDeclaration::Declared(ExpectedOutcome::ExpectSuccess),
        ) {
            Ok(Value::Record { fields, .. }) => ctx.field(&fields, "value").and_then(|v| match v {
                Value::Str(s) if !s.is_empty() => Some(s.to_string()),
                _ => None,
            }),
            Ok(Value::Str(s)) if !s.is_empty() => Some(s.to_string()),
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
    declared: ExpectationDeclaration,
) -> InterpResult<Value> {
    let expected = declared.resolve(service_name, op_name);
    ctx.effect_dispatch_count
        .set(ctx.effect_dispatch_count.get().wrapping_add(1));
    let key = format!("{}.{}", service_name, op_name);
    let (service_node, op_node) =
        ctx.indexes
            .service_ops
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
                    Value::Str(s) => Some(s.to_string()),
                    _ => None,
                })
                .map(|requested| hermetic_checkout_read_disposition(&requested).is_ok())
                .unwrap_or(false);
            if confirmed_checkout_input {
                return dispatch_service_wet(
                    service_node,
                    op_node,
                    transport,
                    &param_env,
                    ctx,
                    &key,
                    expected,
                );
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
        // An active witness replay frame OUTRANKS the published-mock layer, because the
        // two answer different questions and only one of them is the seam under test.
        // `eval_mock_response` replays the operation RESULT off the declaration; a replay
        // frame supplies the transport OBSERVATION and requires the real dispatcher fold
        // to run on top of it. Letting the mock answer first is the exact fail-open the
        // seam exists to close: the fixture greens while the dispatcher is never reached,
        // so a broken dispatcher is unobservable in the mode CI actually runs (hermetic).
        // Measured: `rest_transport_failure_is_persistable` returns true under `gunbc run
        // --claim-run` (Wet, reaches `dispatch_rest`) and false under `claim_batch`
        // (Hermetic, answered here) on one binary and one tree.
        //
        // Fail-closed on both arms, never a widen (§5): for a REST transport this routes
        // to the ordinary wet dispatch so `rest_exchange_selection` decides, and that
        // selection already refuses `RestReplayExchangeAbsent`/`Ambiguous` BEFORE any
        // socket is opened — so an active frame with no matching fixture is a typed
        // refusal, not a live request escaping hermetic mode. Every other transport
        // refuses here rather than falling through: a declared replay intent this
        // machinery cannot honor must stop the line, not silently degrade to the mock
        // (which would fabricate a plausible answer) nor to a real shell/file effect.
        //
        // HAND-RUST GATE — seed-retained, lane `v1-materialization-kernel`
        // (rn_53JPH6BB7G588K7DMZNWM0E3AS, docs/plans/witness-realization-plan.md),
        // terminating at `v1-interpreter-quarantine` → `v1-interpreter-delete`; the same
        // lane the `WITNESS_EVALUATION_FRAMES` deferral above names. Deletion condition,
        // checkable by execution: when witnesses emit to native code and the emitted
        // runtime realizes the evaluation frame, this arm deletes with that stack while
        // `rest_transport_failure_is_persistable` stays green under the corpus runner
        // without it. That witness is this arm's regression control, not merely the
        // frame's — it reds if the mock layer ever preempts a replay frame again.
        if current_witness_evaluation_frame().is_some() {
            if is_shell_transport(transport.clone())
                || is_file_transport(transport.clone(), ctx.si())
            {
                return Err(InterpError::TypeError {
                    msg: format!(
                        "witness replay frame is active for {key}, but its transport has no \
                         replay realization — refusing (only REST exchanges are replayable; \
                         the frame is not silently ignored)"
                    ),
                });
            }
            return dispatch_service_wet(
                service_node,
                op_node,
                transport,
                &param_env,
                ctx,
                &key,
                expected,
            );
        }

        trace_emit(
            OutputChannel::Instrumentation,
            &format!("[hermetic:mock] {}.{}", service_name, op_name),
        );
        return eval_mock_response(op_node, ctx);
    }

    let result = dispatch_service_wet(
        service_node,
        op_node,
        transport,
        &param_env,
        ctx,
        &key,
        expected,
    )?;

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
    intent: &str,
    expected: ExpectedOutcome,
) -> InterpResult<Value> {
    // Local Env.Get: reading THIS process's own environment is not a host effect
    // (shell-to-dag residual census §0b / DESIGN §3(b)). `printenv` was the wrong
    // single hardwired transport — unset vars exited 1 and, under ExpectSuccess,
    // every optional floor_diff_observe injection (GUNBC_CI_DIFF_*) painted Anomaly
    // Failed lines (operator live-log 2026-07-25). Native handler; shell printenv
    // remains the remote-target realization.
    if intent == "shell.Env.Get" {
        return dispatch_env_get_native(op_node, param_env, ctx);
    }

    if is_shell_transport(transport.clone()) {
        let result = dispatch_shell(transport, param_env, ctx, intent, expected)?;
        return map_shell_outputs(&result, op_node, ctx);
    }

    if is_file_transport(transport.clone(), ctx.si()) {
        let result = dispatch_file(op_node, transport, param_env, ctx)?;
        return map_file_outputs(&result, op_node, ctx);
    }

    dispatch_rest(service_node, op_node, transport, param_env, ctx)
}

/// Native realization of `shell.Env.Get` for OnTarget locality — same Absent/Present
/// semantics as printenv (unset/empty → Null optional; value trimmed), with no
/// ObservationEvent and no child process.
fn dispatch_env_get_native(
    op_node: &Rc<Node>,
    param_env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let name = match param_env.lookup(ctx.sym("name")) {
        Some(Value::Str(s)) => s.to_string(),
        Some(other) => {
            return Err(InterpError::TypeError {
                msg: format!("shell.Env.Get name must be String, got {other}"),
            });
        }
        None => {
            return Err(InterpError::TypeError {
                msg: "shell.Env.Get missing name parameter".to_string(),
            });
        }
    };
    let value = match wet_env_var(&name) {
        Some(s) => str_value(s),
        None => Value::Null,
    };
    Ok(Value::Record {
        type_name: ctx.sym(&authored_name_at(ctx.si(), op_node.clone())),
        fields: Rc::new(vec![(ctx.sym("value"), value)]),
    })
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

#[derive(Debug)]
pub(crate) struct ShellResult {
    pub(crate) exit_code: i32,
    pub(crate) stdout: bounded_shell_host_drain::CapturedStreamEvidence,
    pub(crate) stderr: bounded_shell_host_drain::CapturedStreamEvidence,
}

fn push_shell_argv_tokens(argv: &mut Vec<String>, val: Value) -> InterpResult<()> {
    match &val {
        Value::Str(s) => {
            argv.push(s.to_string());
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
        return Some(s.to_string());
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

/// Why a generic argv materialization refused. Mirrors `ArgvRefusalCause` in
/// `src/v2/std/operation_argv.dag` arm for arm — the .dag module is the authority for
/// the vocabulary, this enum is the seed realization of it. Every arm refuses; none
/// widens, defaults, or sanitizes (DESIGN §5).
#[derive(Debug, Clone)]
pub enum ArgvRefusalCause {
    OperationNotFound,
    UndeclaredInputBound(String),
    DuplicateInputBinding(String),
    DeclaredInputUnbound(String),
    ArgvEmpty,
    ExecutablePositionNotLiteral(String),
    TokenListInStringPosition(String),
    ArgvExpressionUnsupported(String),
    BindingMalformed(String),
}

fn argv_expr_kind_label(node: &Rc<Node>) -> &'static str {
    match node.expr_data.as_ref() {
        ExprData::NoExprData => "no-expr",
        ExprData::ExprLiteral { .. } => "literal",
        ExprData::ExprError { .. } => "error",
        ExprData::ExprVar { .. } => "var",
        ExprData::ExprFieldAccess { .. } => "field-access",
        ExprData::ExprCall { .. } => "call",
        ExprData::ExprMethodCall { .. } => "method-call",
        ExprData::ExprMatch => "match",
        ExprData::ExprIf => "if",
        ExprData::ExprLet => "let",
        ExprData::ExprRecordLit { .. } => "record-literal",
        ExprData::ExprListLit => "list-literal",
        ExprData::ExprBinOp { .. } => "binop",
        ExprData::ExprUnaryOp { .. } => "unary-op",
        ExprData::ExprLambda => "lambda",
        ExprData::ExprStringInterp => "string-interpolation",
        ExprData::ExprBlock => "block",
        ExprData::ExprCast => "cast",
        ExprData::ExprForEach => "for-each",
        ExprData::ExprIndex => "index",
        ExprData::ExprSlice => "slice",
        ExprData::ExprReturn => "return",
    }
}

/// A declared default, read from the declaration itself. Only shapes that are literal
/// *data* are admitted (a string literal, or a list literal of string literals): a
/// default that is a call or a reference is left UNBOUND, so an argv position that
/// needs it refuses by name rather than being filled with a guess (DESIGN §5 — a
/// fabricated plausible default is the failure this avoids).
fn declared_default_value(node: &Rc<Node>) -> Option<Value> {
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitStr { value } => Some(str_value(value.clone())),
            _ => None,
        },
        ExprData::ExprListLit => {
            let mut items: Vec<Value> = Vec::new();
            for child in node.children.iter() {
                match child.expr_data.as_ref() {
                    ExprData::ExprLiteral { value } => match value.as_ref() {
                        LiteralValue::LitStr { value } => items.push(str_value(value.clone())),
                        _ => return None,
                    },
                    _ => return None,
                }
            }
            Some(list_value(items))
        }
        _ => None,
    }
}

/// Read one `OperationInputBinding` record from the `.dag` side.
fn operation_input_binding_entry(
    item: &Value,
    ctx: &InterpContext,
) -> Result<(String, Value), ArgvRefusalCause> {
    let Value::Record { fields, .. } = item else {
        return Err(ArgvRefusalCause::BindingMalformed(format!(
            "binding list element is {}, expected an OperationInputBinding record",
            item.type_label()
        )));
    };
    let Some(Value::Str(name)) = fields_get(fields, ctx.sym("name")).cloned() else {
        return Err(ArgvRefusalCause::BindingMalformed(
            "OperationInputBinding.name must be a String".to_string(),
        ));
    };
    let Some(value) = fields_get(fields, ctx.sym("value")).cloned() else {
        return Err(ArgvRefusalCause::BindingMalformed(format!(
            "OperationInputBinding `{name}` carries no value"
        )));
    };
    let bound = match &value {
        Value::Variant {
            variant_name,
            fields,
            ..
        } => {
            if *variant_name == ctx.sym("InputText") {
                match fields_get(fields, ctx.sym("text")).cloned() {
                    Some(Value::Str(text)) => Value::Str(Rc::clone(&text)),
                    _ => {
                        return Err(ArgvRefusalCause::BindingMalformed(format!(
                            "InputText for `{name}` carries no String text"
                        )))
                    }
                }
            } else if *variant_name == ctx.sym("InputTextList") {
                match fields_get(fields, ctx.sym("items")).cloned() {
                    Some(items) => match free_monoid_to_vec(&items) {
                        Some(vals) => list_value(vals),
                        None => {
                            return Err(ArgvRefusalCause::BindingMalformed(format!(
                                "InputTextList for `{name}` carries no List<String> items"
                            )))
                        }
                    },
                    None => {
                        return Err(ArgvRefusalCause::BindingMalformed(format!(
                            "InputTextList for `{name}` carries no items field"
                        )))
                    }
                }
            } else {
                return Err(ArgvRefusalCause::BindingMalformed(format!(
                    "OperationInputValue for `{name}` is an unknown variant"
                )));
            }
        }
        other => {
            return Err(ArgvRefusalCause::BindingMalformed(format!(
                "OperationInputValue for `{name}` is {}, expected InputText | InputTextList",
                other.type_label()
            )))
        }
    };
    Ok((name.to_string(), bound))
}

/// Bindings ∪ declared defaults, validated against the operation's OWN declared inputs.
/// A binding naming an input the operation does not declare is refused rather than
/// injected — the seed's previous materializer unconditionally injected `package`,
/// `bin`, `args`, `unit` and `property` into every operation, which is exactly the
/// channel a generic binder must not keep open.
fn operation_input_binding_env(
    bindings: &Value,
    declared: &[(String, Option<Rc<Node>>)],
    ctx: &InterpContext,
) -> Result<HashMap<String, Value>, ArgvRefusalCause> {
    let Some(items) = free_monoid_to_vec(bindings) else {
        return Err(ArgvRefusalCause::BindingMalformed(format!(
            "bindings argument is {}, expected a List<OperationInputBinding>",
            bindings.type_label()
        )));
    };
    let mut env: HashMap<String, Value> = HashMap::new();
    for item in items.iter() {
        let (name, value) = operation_input_binding_entry(item, ctx)?;
        if !declared
            .iter()
            .any(|(declared_name, _)| *declared_name == name)
        {
            return Err(ArgvRefusalCause::UndeclaredInputBound(name));
        }
        if env.contains_key(&name) {
            return Err(ArgvRefusalCause::DuplicateInputBinding(name));
        }
        env.insert(name, value);
    }
    for (name, default) in declared.iter() {
        if env.contains_key(name) {
            continue;
        }
        if let Some(default_value) = default.as_ref().and_then(declared_default_value) {
            env.insert(name.clone(), default_value);
        }
    }
    Ok(env)
}

fn bind_argv_expr(
    node: &Rc<Node>,
    env: &HashMap<String, Value>,
    source_indices: &Rc<HashMap<String, Rc<crate::v1_std_core::NewlineIndex>>>,
) -> Result<Value, ArgvRefusalCause> {
    match node.expr_data.as_ref() {
        ExprData::ExprLiteral { value } => match value.as_ref() {
            LiteralValue::LitStr { value } => Ok(str_value(value.clone())),
            other => Err(ArgvRefusalCause::ArgvExpressionUnsupported(format!(
                "argv element literal is {other:?}, expected a string literal"
            ))),
        },
        ExprData::ExprVar { .. } => {
            let name = expr_var_name_at(node.clone(), source_indices.clone());
            env.get(&name)
                .cloned()
                .ok_or(ArgvRefusalCause::DeclaredInputUnbound(name))
        }
        ExprData::ExprStringInterp => {
            let parts = extract_string_interp_parts(node.clone());
            let mut result = String::new();
            for part in parts.iter() {
                match part.as_ref() {
                    StringPart::Text { value } => result.push_str(value),
                    StringPart::Interpolation { expr } => {
                        let val = bind_argv_expr(expr, env, source_indices)?;
                        match &val {
                            Value::List(_) => {
                                return Err(ArgvRefusalCause::TokenListInStringPosition(
                                    expr_var_name_at(expr.clone(), source_indices.clone()),
                                ))
                            }
                            _ => result.push_str(&value_to_host_string(&val)),
                        }
                    }
                }
            }
            Ok(str_value(result))
        }
        _ => Err(ArgvRefusalCause::ArgvExpressionUnsupported(format!(
            "argv element is a {} expression; materialization binds declared inputs, it does not evaluate expressions",
            argv_expr_kind_label(node)
        ))),
    }
}

/// Materialize an operation's transport argv by binding its own declared inputs.
///
/// The executable position is a construction wall, not a check with a lenient arm:
/// `argv[0]` must be a string literal in the declaration, so no binding — declared or
/// not — can decide which program runs.
pub fn materialize_operation_argv(
    path: &str,
    service: &str,
    operation: &str,
    bindings: &Value,
    ctx: &InterpContext,
) -> Result<Vec<String>, ArgvRefusalCause> {
    let Some(declaration) =
        crate::cli_run::shell_transport_operation_declaration(path, service, operation)
    else {
        return Err(ArgvRefusalCause::OperationNotFound);
    };
    let env = operation_input_binding_env(bindings, &declaration.declared_inputs, ctx)?;

    let Some(executable) = declaration.argv.iter().next() else {
        return Err(ArgvRefusalCause::ArgvEmpty);
    };
    let executable_is_literal = matches!(
        executable.expr_data.as_ref(),
        ExprData::ExprLiteral { value } if matches!(value.as_ref(), LiteralValue::LitStr { .. })
    );
    if !executable_is_literal {
        return Err(ArgvRefusalCause::ExecutablePositionNotLiteral(format!(
            "argv[0] is a {} expression; the executable must be a literal in the declaration",
            argv_expr_kind_label(executable)
        )));
    }

    let mut argv: Vec<String> = Vec::new();
    for node in declaration.argv.iter() {
        let val = bind_argv_expr(node, &env, &declaration.source_indices)?;
        push_shell_argv_tokens(&mut argv, val).map_err(|e| {
            ArgvRefusalCause::ArgvExpressionUnsupported(format!("argv token flatten failed: {e:?}"))
        })?;
    }
    Ok(argv)
}

fn argv_refusal_cause_value(cause: &ArgvRefusalCause, ctx: &InterpContext) -> Value {
    let variant = |name: &str, fields: Vec<(Symbol, Value)>| Value::Variant {
        type_name: ctx.sym("ArgvRefusalCause"),
        variant_name: ctx.sym(name),
        fields: Rc::new(sorted_fields(fields)),
    };
    match cause {
        ArgvRefusalCause::OperationNotFound => variant("OperationNotFound", vec![]),
        ArgvRefusalCause::ArgvEmpty => variant("ArgvEmpty", vec![]),
        ArgvRefusalCause::UndeclaredInputBound(name) => variant(
            "UndeclaredInputBound",
            vec![(ctx.sym("name"), str_value(name.clone()))],
        ),
        ArgvRefusalCause::DuplicateInputBinding(name) => variant(
            "DuplicateInputBinding",
            vec![(ctx.sym("name"), str_value(name.clone()))],
        ),
        ArgvRefusalCause::DeclaredInputUnbound(name) => variant(
            "DeclaredInputUnbound",
            vec![(ctx.sym("name"), str_value(name.clone()))],
        ),
        ArgvRefusalCause::TokenListInStringPosition(name) => variant(
            "TokenListInStringPosition",
            vec![(ctx.sym("name"), str_value(name.clone()))],
        ),
        ArgvRefusalCause::ExecutablePositionNotLiteral(detail) => variant(
            "ExecutablePositionNotLiteral",
            vec![(ctx.sym("detail"), str_value(detail.clone()))],
        ),
        ArgvRefusalCause::ArgvExpressionUnsupported(detail) => variant(
            "ArgvExpressionUnsupported",
            vec![(ctx.sym("detail"), str_value(detail.clone()))],
        ),
        ArgvRefusalCause::BindingMalformed(detail) => variant(
            "BindingMalformed",
            vec![(ctx.sym("detail"), str_value(detail.clone()))],
        ),
    }
}

fn operation_ref_value(path: &str, service: &str, operation: &str, ctx: &InterpContext) -> Value {
    Value::Record {
        type_name: ctx.sym("OperationRef"),
        fields: Rc::new(sorted_fields(vec![
            (ctx.sym("path"), str_value(path.to_string())),
            (ctx.sym("service"), str_value(service.to_string())),
            (ctx.sym("operation"), str_value(operation.to_string())),
        ])),
    }
}

/// Projects a host diagnostic census into the `gunbc.compile_diagnostic_census` coproduct.
/// The two arms stay distinct all the way to the substrate — `CensusNotRunnable` must never
/// arrive as `CensusObserved` with an empty row list, because that is byte-identical to a clean
/// compile and would let could-not-measure read as the subject passing (DESIGN §5).
fn compile_diagnostic_census_value(
    census: crate::cli_run::CompileDiagnosticCensus,
    ctx: &InterpContext,
) -> Value {
    match census {
        crate::cli_run::CompileDiagnosticCensus::Observed(rows) => Value::Variant {
            type_name: ctx.sym("CompileDiagnosticCensus"),
            variant_name: ctx.sym("CensusObserved"),
            fields: Rc::new(sorted_fields(vec![(
                ctx.sym("rows"),
                list_value(
                    rows.into_iter()
                        .map(|r| Value::Record {
                            type_name: ctx.sym("CompileDiagnosticCensusRow"),
                            fields: Rc::new(sorted_fields(vec![
                                (ctx.sym("diagnostic_class"), str_value(r.diagnostic_class)),
                                (ctx.sym("subject_name"), str_value(r.subject_name)),
                                (ctx.sym("blocking"), Value::Bool(r.blocking)),
                                (ctx.sym("count"), Value::Int(r.count)),
                            ])),
                        })
                        .collect::<Vec<_>>(),
                ),
            )])),
        },
        crate::cli_run::CompileDiagnosticCensus::NotRunnable(cause) => Value::Variant {
            type_name: ctx.sym("CompileDiagnosticCensus"),
            variant_name: ctx.sym("CensusNotRunnable"),
            fields: Rc::new(sorted_fields(vec![(ctx.sym("cause"), str_value(cause))])),
        },
    }
}

fn unlisted_import_binding_source_value(
    source: crate::cli_run::UnlistedImportBindingSource,
    ctx: &InterpContext,
) -> Value {
    let variant = match source {
        crate::cli_run::UnlistedImportBindingSource::ListedImport => "ListedImport",
        crate::cli_run::UnlistedImportBindingSource::PoolCoincidence => "PoolCoincidence",
        crate::cli_run::UnlistedImportBindingSource::DefinerResolvable => "DefinerResolvable",
    };
    Value::Variant {
        type_name: ctx.sym("UnlistedImportBindingSource"),
        variant_name: ctx.sym(variant),
        fields: Rc::new(vec![]),
    }
}

fn declared_import_closure_binding_value(
    observation: crate::cli_run::DeclaredImportClosureBindingObservation,
    ctx: &InterpContext,
) -> Value {
    match observation {
        crate::cli_run::DeclaredImportClosureBindingObservation::Observed(observed) => {
            let binding_source = match observed.binding_source {
                Some(source) => {
                    optional_present(unlisted_import_binding_source_value(source, ctx), ctx)
                }
                None => optional_absent(ctx),
            };
            Value::Variant {
                type_name: ctx.sym("DeclaredImportClosureBindingObservation"),
                variant_name: ctx.sym("BindingObserved"),
                fields: Rc::new(sorted_fields(vec![
                    (ctx.sym("binding_source"), binding_source),
                    (
                        ctx.sym("definer_module"),
                        str_value(observed.definer_module.unwrap_or_default()),
                    ),
                    (
                        ctx.sym("symbol_resolves"),
                        Value::Bool(observed.symbol_resolves),
                    ),
                    (
                        ctx.sym("blocking_hard_diagnostic_count"),
                        Value::Int(observed.blocking_hard_diagnostic_count),
                    ),
                ])),
            }
        }
        crate::cli_run::DeclaredImportClosureBindingObservation::NotRunnable(cause) => {
            Value::Variant {
                type_name: ctx.sym("DeclaredImportClosureBindingObservation"),
                variant_name: ctx.sym("BindingNotRunnable"),
                fields: Rc::new(sorted_fields(vec![(ctx.sym("cause"), str_value(cause))])),
            }
        }
    }
}

fn argv_materialization_value(
    result: Result<Vec<String>, ArgvRefusalCause>,
    path: &str,
    service: &str,
    operation: &str,
    ctx: &InterpContext,
) -> Value {
    match result {
        Ok(argv) => Value::Variant {
            type_name: ctx.sym("ArgvMaterialization"),
            variant_name: ctx.sym("ArgvMaterialized"),
            fields: Rc::new(sorted_fields(vec![(
                ctx.sym("argv"),
                list_value(argv.into_iter().map(str_value).collect::<Vec<_>>()),
            )])),
        },
        Err(cause) => Value::Variant {
            type_name: ctx.sym("ArgvMaterialization"),
            variant_name: ctx.sym("ArgvMaterializationRefused"),
            fields: Rc::new(sorted_fields(vec![
                (
                    ctx.sym("at"),
                    operation_ref_value(path, service, operation, ctx),
                ),
                (ctx.sym("cause"), argv_refusal_cause_value(&cause, ctx)),
            ])),
        },
    }
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

/// Whether a call site DECLARED its expectation, kept distinct from WHAT it declared.
///
/// The dispatch boundary takes this rather than a bare `ExpectedOutcome`, so
/// "the site said nothing" is a value the caller must construct rather than a
/// default the boundary manufactures. Before 2026-07-26 the boundary closed the
/// gap itself with `declared_expectation.unwrap_or(ExpectSuccess)`, which is the
/// shape DESIGN §5 names an absorbing fallback: the missing declaration was
/// answered with a plausible one, and because the substitution happened inside
/// the callee it left no trace, so the frequency of undeclared sites was zero by
/// construction and could never rank for fixing.
///
/// `UntracedDefault` resolves to the SAME `ExpectSuccess` — this is deliberately
/// not a behaviour change, and deliberately not a floor-time refusal (operator
/// guardrail, 2026-07-26: the wall is construction at the boundary, not a
/// refusal sweep that would red the corpus on sites nobody has looked at yet).
/// What changes is that the substitution is now typed, located and COUNTED, so
/// the frontier is observable and shrinks under trace evidence. A site re-arms
/// to `Declared` only when someone has actually established what it expects —
/// never speculatively, since a wrong `ExpectFailure` would silence a real fault.
///
/// 🟡 dissolve-on: when the counted frontier reaches zero corpus-wide, this arm
/// is deleted and an undeclared site becomes a hard typed refusal at the
/// boundary — at which point absence is unwritable rather than merely counted.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExpectationDeclaration {
    Declared(ExpectedOutcome),
    UntracedDefault,
}

impl ExpectationDeclaration {
    /// Resolve to the outcome the dispatch grades against, counting the untraced
    /// case at its located call site. The count is the whole point: an absorbed
    /// default that is tallied is a declared interim frontier; one that is not is
    /// the fail-open this type exists to delete.
    fn resolve(self, service_name: &str, op_name: &str) -> ExpectedOutcome {
        match self {
            ExpectationDeclaration::Declared(e) => e,
            ExpectationDeclaration::UntracedDefault => {
                record_untraced_expectation_site(service_name, op_name);
                ExpectedOutcome::ExpectSuccess
            }
        }
    }
}

thread_local! {
    /// `service.op` -> times that site dispatched without a declared expectation.
    static UNTRACED_EXPECTATION_SITES: std::cell::RefCell<std::collections::BTreeMap<String, u64>> =
        const { std::cell::RefCell::new(std::collections::BTreeMap::new()) };
}

fn record_untraced_expectation_site(service_name: &str, op_name: &str) {
    let key = format!("{}.{}", service_name, op_name);
    UNTRACED_EXPECTATION_SITES.with(|m| {
        *m.borrow_mut().entry(key).or_insert(0) += 1;
    });
}

/// The frontier as `(service.op, dispatch_count)`, ascending by key. Empty means
/// every effect that ran declared its expectation — the dissolution condition.
pub fn untraced_expectation_frontier() -> Vec<(String, u64)> {
    UNTRACED_EXPECTATION_SITES.with(|m| m.borrow().iter().map(|(k, v)| (k.clone(), *v)).collect())
}

/// Carrier for `gunbc.output_policy.ExpectedOutcome` — what the caller DECLARED an
/// effect would do. There is deliberately no `ExpectAny`: an unknown expectation
/// would make every observation agree, which is the empty set the untyped
/// `exit != 0` proxy assumed (DESIGN §5 — a failure arm refuses, never widens).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExpectedOutcome {
    ExpectSuccess,
    ExpectFailure,
    /// The exit code is an OBSERVATION consumed by a typed downstream verdict, not a
    /// pass/fail judgment of its own — a probe whose non-zero exit means "subject
    /// absent" rather than "something broke". Renders ambient regardless of code;
    /// only dispatch failure (the probe could not run) stays anomalous. Admissible
    /// ONLY where the annotated helper returns a typed observation or verdict, never
    /// unit — see `gunbc.output_policy` outcome_is_data_note for the guard.
    OutcomeIsData,
}

/// The reserved call-site argument through which a caller DECLARES what it expects
/// an effect to do — the sibling edge on the CALL node (operator decision
/// 2026-07-25), resolving the open question this file carried at `dispatch_shell`.
///
/// It is stripped from the argument list before `build_service_param_env`, so it
/// never becomes a bound param and therefore never reaches
/// `content_hash_service_inputs` — which iterates `op_node.params` and looks each up
/// in `param_env`. The exclusion is structural, not a remembered filter, which is
/// exactly why a service-op `input {}` was the wrong home: that IS a param, so it
/// would join the digest and two invocations differing only in what the caller
/// expected would become different cache identities for the same request.
///
/// It is equally NOT a transport property, despite `transport_stdin` /
/// `transport_response_format` being the nearest local precedent. `dispatch_service_wet`
/// receives the transport from `op_node.transport.or(service_node.transport)` — the
/// extdeps service-op DECLARATION, shared by every caller. Hanging the expectation
/// there would make it a per-operation fact, so a red control and a genuine check
/// both calling `shell.Test.IsFile` could not differ, and caller policy declared in
/// extdeps is the DESIGN §3 layer inversion. That shape would have typechecked and
/// read as green while being wrong in both directions.
pub const EFFECT_EXPECTATION_ARG: &str = "expect";

/// Read the caller's declared expectation off the reserved argument's value.
///
/// Absent is handled by the caller as `ExpectSuccess` — the DECLARED migration
/// default, behaviour-identical to the untyped `exit != 0` proxy this replaces, so
/// no existing call site changes meaning. A PRESENT but unreadable value REFUSES
/// instead of falling back: `gunbc.output_policy` deliberately has no `ExpectAny`
/// arm, so a value we cannot read is ignorance, and answering it with a default
/// would be ⊤-as-answer conflated with ⊤-as-ignorance (DESIGN §5 — a failure arm
/// refuses, never widens).
fn expectation_from_declared_arg(val: &Value) -> InterpResult<ExpectedOutcome> {
    match val {
        Value::Variant { variant_name, .. } => match resolve_sym(*variant_name).as_str() {
            "ExpectSuccess" => Ok(ExpectedOutcome::ExpectSuccess),
            "ExpectFailure" => Ok(ExpectedOutcome::ExpectFailure),
            "OutcomeIsData" => Ok(ExpectedOutcome::OutcomeIsData),
            other => Err(InterpError::TypeError {
                msg: format!(
                    "`{EFFECT_EXPECTATION_ARG}:` must be an ExpectedOutcome (ExpectSuccess | ExpectFailure | OutcomeIsData), got variant `{other}`"
                ),
            }),
        },
        _ => Err(InterpError::TypeError {
            msg: format!(
                "`{EFFECT_EXPECTATION_ARG}:` must be an ExpectedOutcome (ExpectSuccess | ExpectFailure), got a non-variant value"
            ),
        }),
    }
}

/// Carrier for `gunbc.output_policy.StreamDisposition` — what becomes of an effect's
/// CAPTURED SUBJECT STREAMS. Distinct from `OutputDecision`, which grades a trace
/// line this repo authored; see the `.dag` authority's note for why it is not a
/// second spelling of it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StreamDisposition {
    SurfaceContent,
    SummarizeCounts,
    StreamSuppressed,
}

/// The `.dag` authority's four corners of `effect_stream_disposition` at the
/// ShellTrace channel and the run's verbosity, plus the guard literal
/// `neutralize_workflow_commands` prefixes each subject line with. The seed holds
/// evaluated verdicts and one literal — never the rule.
#[derive(Clone)]
pub struct InstalledEffectStreamPolicy {
    /// Indexed by `stream_policy_index(expected, observed_success)`.
    pub dispositions: [StreamDisposition; 6],
    pub subject_line_guard: String,
}

static EFFECT_STREAM_POLICY: std::sync::OnceLock<InstalledEffectStreamPolicy> =
    std::sync::OnceLock::new();

/// Uninstalled default, as DATA rather than a re-derivation of the `.dag` rule:
/// the four corners at `Normal` verbosity. It is behaviour-preserving — at the
/// migration default `ExpectSuccess` it says "content on non-zero exit, counts on
/// zero", exactly what this file did before the expectation axis existed.
/// `effect_stream_policy_mirror_matches_dag_authority` pins it to the same golden
/// the `.dag` witness asserts, so the two cannot drift silently.
const EFFECT_STREAM_POLICY_FALLBACK: [StreamDisposition; 6] = [
    StreamDisposition::SummarizeCounts, // ExpectSuccess × observed success
    StreamDisposition::SurfaceContent,  // ExpectSuccess × observed failure
    StreamDisposition::SurfaceContent,  // ExpectFailure × observed success
    StreamDisposition::SummarizeCounts, // ExpectFailure × observed failure
    StreamDisposition::SummarizeCounts, // OutcomeIsData × observed success
    StreamDisposition::SummarizeCounts, // OutcomeIsData × observed failure — the exit is
                                        // an answer, so neither pole is an anomaly
];

/// Mirror of `extdeps.github.log_annotations.subject_text_line_guard`, used when no
/// entry binary installed a policy. Pinned by the mirror test above.
const SUBJECT_LINE_GUARD_FALLBACK: &str = "| ";

fn stream_policy_index(expected: ExpectedOutcome, observed_success: bool) -> usize {
    match (expected, observed_success) {
        (ExpectedOutcome::ExpectSuccess, true) => 0,
        (ExpectedOutcome::ExpectSuccess, false) => 1,
        (ExpectedOutcome::ExpectFailure, true) => 2,
        (ExpectedOutcome::ExpectFailure, false) => 3,
        (ExpectedOutcome::OutcomeIsData, true) => 4,
        (ExpectedOutcome::OutcomeIsData, false) => 5,
    }
}

/// Install the effect-stream policy the entry binary evaluated from
/// `gunbc.output_policy.resolve_shell_trace_stream_policy`. Same lifecycle as
/// `set_output_policy`: set once at startup, first install wins.
pub fn set_effect_stream_policy(policy: InstalledEffectStreamPolicy) {
    let _ = EFFECT_STREAM_POLICY.set(policy);
}

/// The installed disposition for an effect whose caller declared `expected` and
/// whose observed exit is `exit_code`.
pub fn effect_stream_disposition(expected: ExpectedOutcome, exit_code: i32) -> StreamDisposition {
    let idx = stream_policy_index(expected, exit_code == 0);
    match EFFECT_STREAM_POLICY.get() {
        Some(p) => p.dispositions[idx],
        None => EFFECT_STREAM_POLICY_FALLBACK[idx],
    }
}

fn subject_line_guard() -> String {
    match EFFECT_STREAM_POLICY.get() {
        Some(p) => p.subject_line_guard.clone(),
        None => SUBJECT_LINE_GUARD_FALLBACK.to_string(),
    }
}

/// Transport of `extdeps.github.log_annotations.neutralize_workflow_commands`:
/// prefix every line of relayed subject text with the guard so it cannot occupy the
/// line-initial `::` position GitHub reads as a workflow command. Unconditional —
/// there is no target probe that could be wrong, and the guard is readable on a
/// plain terminal. The transformation is the `.dag` authority's; this only applies
/// the literal it published.
fn neutralize_workflow_commands(text: &str) -> String {
    let guard = subject_line_guard();
    format!("{guard}{}", text.replace('\n', &format!("\n{guard}")))
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

/// Tracks an open host-effect group so `group_end` is idempotent — law 4 closes the
/// group before an Anomaly shell failure, and the batch-end `group_end` must not emit
/// a second `::endgroup::`.
static GROUP_OPEN: AtomicBool = AtomicBool::new(false);

/// Open a titled group on stderr — the same stream the host-effect trace lines use,
/// so the runner folds those lines under the marker. No-op when no syntax is
/// installed. Pair with `group_end`; the caller must keep the bracket tight (open →
/// run+join the effectful work → close) and defer non-trace output (PASS/FAIL) until
/// after `group_end` so it stays outside the collapsed section.
pub fn group_begin(title: &str) {
    if let Some(s) = GROUP_SYNTAX.get() {
        eprintln!("{}{}{}", s.open_prefix, title, s.open_suffix);
        GROUP_OPEN.store(true, Ordering::SeqCst);
    }
}

/// Close the current group. Emits the close line only when the target defines one
/// (GitHub Actions) and a group is actually open. Idempotent: a second call is a
/// no-op (law 4 may have already closed for an Anomaly).
pub fn group_end() {
    if !GROUP_OPEN.swap(false, Ordering::SeqCst) {
        return;
    }
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

/// Census hygiene marker for the `[shell]` emit family — kept after the wiring flip so
/// the observation_emit_census roster cannot go stale.
pub const SHELL_CENSUS_MARKER: &str = "[shell]";

/// Collapse argv into one readable line — runs of whitespace become a single space —
/// so a multiline `sh -c` script reads as one command. Used in Failed.error (uncapped:
/// an anomaly expands fully). Ambient subjects are named intents, not argv.
fn shell_argv_collapsed(argv: &[String]) -> String {
    argv.join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_obs_emoji() -> bool {
    std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true")
}

fn shell_obs_human_duration(ms: u64) -> String {
    if ms < 1_000 {
        format!("{ms}ms")
    } else if ms < 60_000 {
        format!("{} seconds", ms / 1_000)
    } else {
        format!("{} minutes", ms / 60_000)
    }
}

/// Mirror of `gunbc.observation_seed_render.shell_effect_begin_line`.
pub fn render_shell_effect_begin_line_mirror(intent: &str, emoji: bool) -> String {
    let _ = SHELL_CENSUS_MARKER;
    let glyph = if emoji { "🔄" } else { "◐" };
    format!("{glyph} started {intent}")
}

/// Mirror of `gunbc.observation_seed_render.shell_effect_done_line`.
pub fn render_shell_effect_done_line_mirror(intent: &str, elapsed_ms: u64, emoji: bool) -> String {
    let _ = SHELL_CENSUS_MARKER;
    let glyph = if emoji { "✅" } else { "✓" };
    format!(
        "{glyph} {intent} done in {}",
        shell_obs_human_duration(elapsed_ms)
    )
}

/// Mirror of `gunbc.observation_seed_render.shell_effect_failed_line`.
/// Subject is the named intent; `argv_collapsed` (WITHOUT `$ `) feeds Failed.error only.
pub fn render_shell_effect_failed_line_mirror(
    intent: &str,
    argv_collapsed: &str,
    exit_code: u64,
    elapsed_ms: u64,
    emoji: bool,
) -> String {
    let _ = SHELL_CENSUS_MARKER;
    let glyph = if emoji { "❌" } else { "✗" };
    format!(
        "{glyph} {intent} failed: $ {argv_collapsed} (exit={exit_code}) in {}",
        shell_obs_human_duration(elapsed_ms)
    )
}

// Ambient shell Begin — named intent subject, ShellTrace-gated (Suppressed at Normal).
fn render_shell_trace(intent: &str) {
    if output_decision(OutputChannel::ShellTrace) == OutputDecision::Suppressed {
        return;
    }
    let line = render_shell_effect_begin_line_mirror(intent, shell_obs_emoji());
    trace_emit(OutputChannel::ShellTrace, &line);
}

/// Post-wait completion: Ambient Done via ShellTrace, or Anomaly Failed via
/// `effect_stream_disposition` alone (never silenced by ShellTrace Suppressed).
/// Law 4: `group_end` before an Anomaly so it lands OutsideGroup.
///
/// Subject is the typed service.op intent. Failed.error carries self-describing
/// `$ <argv> (exit=N)`; empty stderr still surfaces via the Failed line alone.
fn render_shell_completion_trace(
    expected: ExpectedOutcome,
    exit_code: i32,
    _stdout_bytes: usize,
    stderr: &[u8],
    wall: std::time::Duration,
    argv: &[String],
    intent: &str,
) {
    let disposition = effect_stream_disposition(expected, exit_code);
    let emoji = shell_obs_emoji();
    let elapsed_ms = wall.as_millis() as u64;
    let collapsed = shell_argv_collapsed(argv);

    if disposition == StreamDisposition::SurfaceContent {
        group_end();
        let code = if exit_code < 0 {
            exit_code.unsigned_abs() as u64
        } else {
            exit_code as u64
        };
        let line =
            render_shell_effect_failed_line_mirror(intent, &collapsed, code, elapsed_ms, emoji);
        eprintln!("{line}");
        if let Some(block) = shell_completion_stderr_content(stderr) {
            eprintln!("{block}");
        }
        return;
    }

    if output_decision(OutputChannel::ShellTrace) == OutputDecision::Suppressed {
        return;
    }
    let line = render_shell_effect_done_line_mirror(intent, elapsed_ms, emoji);
    trace_emit(OutputChannel::ShellTrace, &line);
}

/// Whether a SurfaceContent failure should emit a Failed observation line.
/// Pure: disposition alone — never the ShellTrace channel. Empty stderr still
/// returns true (the Failed line is the sole signal). RED control for the
/// installed-policy path: pair with `effect_stream_disposition`.
fn shell_failure_surfaces(disposition: StreamDisposition) -> bool {
    disposition == StreamDisposition::SurfaceContent
}

/// Pure tail-bounding of captured stderr that follows a Failed observation line.
/// Returns `None` when there is no stderr to surface; `Some(block)` is the content
/// only (the Failed line already carries `$ argv (exit=N)`). Subject lines are
/// guarded so relayed text cannot mint workflow commands in the parent run.
fn shell_completion_stderr_content(stderr: &[u8]) -> Option<String> {
    if stderr.is_empty() {
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
        "{prefix}{}",
        // Trailing newlines are stripped before guarding so a subject whose stderr
        // ends in `\n` (almost all of them) does not render a stray guard-only line.
        neutralize_workflow_commands(String::from_utf8_lossy(tail).trim_end_matches('\n'))
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
/// `extdeps.exec.exec_arg_limit.host_exec_arg_max_strlen` (Linux execve(2)
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

/// When a whole-receipt wall deadline is armed, put the child in its own process
/// group so a mid-wait kill reaps cargo→rustc descendants, not only the parent.
fn configure_shell_process_group_for_wall_kill(
    cmd: &mut std::process::Command,
    ctx: &InterpContext,
) {
    if ctx.witness_wall_deadline.get().is_none() {
        return;
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // SAFETY: runs in the child after fork, before exec — setpgid(0,0) is the
        // standard isolate-for-kill pattern; no shared mutable state is touched.
        unsafe {
            cmd.pre_exec(|| {
                if libc::setpgid(0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (cmd, ctx);
    }
}

fn kill_shell_process_group(pid: u32) {
    #[cfg(unix)]
    {
        // SAFETY: kill the child itself, then its process group (set when a wall
        // deadline is armed) so cargo→rustc descendants die with the parent.
        // The direct pid kill covers the no-setpgid path (tests / non-unix fallbacks).
        unsafe {
            let _ = libc::kill(pid as i32, libc::SIGKILL);
            let _ = libc::kill(-(pid as i32), libc::SIGKILL);
        }
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
    }
}

/// Wait for a shell child, killing the process group when the whole-receipt wall
/// deadline elapses. Streams are drained concurrently with bounded capture policies
/// while the child runs — never `wait_with_output` (srv1 jq stderr wedge, 2026-08-09).
fn wait_child_honoring_wall_deadline(
    child: std::process::Child,
    ctx: &InterpContext,
    argv0: &str,
    stdout_policy: bounded_shell_host_drain::StreamCapturePolicy,
    stderr_policy: bounded_shell_host_drain::StreamCapturePolicy,
) -> InterpResult<bounded_shell_host_drain::ShellCaptureResult> {
    if ctx.witness_wall_deadline.get().is_none() {
        return bounded_shell_host_drain::capture_child_output(child, stdout_policy, stderr_policy)
            .map_err(|e| InterpError::TypeError {
                msg: format!("failed to wait on '{}': {}", argv0, e),
            });
    }

    let pid = child.id();
    let (tx, rx) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result =
            bounded_shell_host_drain::capture_child_output(child, stdout_policy, stderr_policy);
        let _ = tx.send(result);
    });

    loop {
        if let Some(err) = ctx.wall_deadline_exceeded_error() {
            kill_shell_process_group(pid);
            let _ = rx.recv_timeout(std::time::Duration::from_secs(2));
            let _ = worker.join();
            return Err(err);
        }
        let remaining_ms = ctx.wall_deadline_remaining_ms().unwrap_or(0);
        let poll = std::time::Duration::from_millis(remaining_ms.min(250).max(1));
        match rx.recv_timeout(poll) {
            Ok(Ok(output)) => {
                let _ = worker.join();
                return Ok(output);
            }
            Ok(Err(e)) => {
                let _ = worker.join();
                return Err(InterpError::TypeError {
                    msg: format!("failed to wait on '{}': {}", argv0, e),
                });
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                let _ = worker.join();
                return Err(InterpError::TypeError {
                    msg: format!("shell wait worker for '{}' disconnected", argv0),
                });
            }
        }
    }
}

fn dispatch_shell(
    transport: &Rc<Node>,
    param_env: &Rc<Env>,
    ctx: &InterpContext,
    intent: &str,
    expected: ExpectedOutcome,
) -> InterpResult<ShellResult> {
    // `expected` is the caller's DECLARED expectation, threaded from the sibling edge
    // on the call node (EFFECT_EXPECTATION_ARG). Absent at the call site it is
    // ExpectSuccess, which keeps every undeclared site behaviour-identical to the
    // untyped `exit != 0` proxy this replaces.
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

    render_shell_trace(intent);

    // Arg-size wall: a single argv token over the host MAX_ARG_STRLEN would make
    // the spawn below die with an opaque `os error 7` (E2BIG). Refuse here with a
    // typed, located diagnostic so the deficit is diagnosable and countable. Large
    // payloads belong in stdin (see extdeps.shell shell.Exec.Run), not argv.
    if let Some(err) = argv_arg_limit_refusal(&argv, HOST_ARG_MAX_STRLEN_BYTES) {
        return Err(err);
    }

    // Refuse before spawn when the whole-receipt wall ceiling is already past
    // (prior subprocess spent the budget; don't start another cargo).
    if let Some(err) = ctx.wall_deadline_exceeded_error() {
        return Err(err);
    }

    let stdout_policy = bounded_shell_host_drain::default_shell_stdout_capture_policy();
    let stderr_policy = bounded_shell_host_drain::default_shell_stderr_capture_policy();

    let capture = if let Some(stdin_node) = transport_stdin(transport.clone(), ctx.si()) {
        use std::io::Write;
        use std::process::Stdio;

        let stdin_val = eval_expr(&stdin_node, param_env, ctx)?;
        let stdin_bytes = shell_stdin_payload(&stdin_val)?;

        let wall_start = std::time::Instant::now();
        let mut cmd = std::process::Command::new(&argv[0]);
        cmd.args(&argv[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        configure_shell_process_group_for_wall_kill(&mut cmd, ctx);
        let mut child = cmd.spawn().map_err(|e| InterpError::TypeError {
            msg: format!("failed to execute '{}': {}", argv[0], e),
        })?;

        let stdin_writer = child
            .stdin
            .take()
            .map(|mut stdin| std::thread::spawn(move || stdin.write_all(&stdin_bytes)));

        let capture =
            wait_child_honoring_wall_deadline(child, ctx, &argv[0], stdout_policy, stderr_policy)?;

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
            expected,
            capture.exit_status.code().unwrap_or(-1),
            capture.stdout.retained_bytes(),
            &capture.stderr.retained,
            wall_start.elapsed(),
            &argv,
            intent,
        );
        capture
    } else {
        use std::process::Stdio;
        let wall_start = std::time::Instant::now();
        let mut cmd = std::process::Command::new(&argv[0]);
        cmd.args(&argv[1..])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        configure_shell_process_group_for_wall_kill(&mut cmd, ctx);
        let child = cmd.spawn().map_err(|e| InterpError::TypeError {
            msg: format!("failed to execute '{}': {}", argv[0], e),
        })?;
        let capture =
            wait_child_honoring_wall_deadline(child, ctx, &argv[0], stdout_policy, stderr_policy)?;
        render_shell_completion_trace(
            expected,
            capture.exit_status.code().unwrap_or(-1),
            capture.stdout.retained_bytes(),
            &capture.stderr.retained,
            wall_start.elapsed(),
            &argv,
            intent,
        );
        capture
    };

    shell_result_from_capture(&capture, &argv[0])
}

pub(crate) fn shell_result_from_capture(
    capture: &bounded_shell_host_drain::ShellCaptureResult,
    argv0: &str,
) -> InterpResult<ShellResult> {
    if capture.stdout.truncated {
        return Err(InterpError::ShellOutputLimitExceeded {
            stream: "stdout",
            total_bytes: capture.stdout.total_bytes,
            limit_bytes: bounded_shell_host_drain::DEFAULT_SHELL_STDOUT_MAX_BYTES as u64,
            argv0: argv0.to_string(),
        });
    }
    Ok(ShellResult {
        exit_code: capture.exit_status.code().unwrap_or(-1),
        stdout: bounded_shell_host_drain::CapturedStreamEvidence::from_observation(&capture.stdout),
        stderr: bounded_shell_host_drain::CapturedStreamEvidence::from_observation(&capture.stderr),
    })
}

fn shell_evidence_value(result: &ShellResult, from_key: &str) -> Option<Value> {
    match from_key {
        "stdout" => Some(str_value(result.stdout.retained_text.clone())),
        "stderr" => Some(str_value(result.stderr.retained_text.clone())),
        "stdout_total_bytes" => Some(Value::Int(result.stdout.total_bytes as i64)),
        "stderr_total_bytes" => Some(Value::Int(result.stderr.total_bytes as i64)),
        "stdout_retained_bytes" => Some(Value::Int(result.stdout.retained_bytes as i64)),
        "stderr_retained_bytes" => Some(Value::Int(result.stderr.retained_bytes as i64)),
        "stdout_truncated" => Some(Value::Bool(result.stdout.truncated)),
        "stderr_truncated" => Some(Value::Bool(result.stderr.truncated)),
        "stdout_digest_hex" => Some(match &result.stdout.digest_hex {
            Some(digest) => str_value(digest.clone()),
            None => Value::Null,
        }),
        "stderr_digest_hex" => Some(match &result.stderr.digest_hex {
            Some(digest) => str_value(digest.clone()),
            None => Value::Null,
        }),
        _ => None,
    }
}

fn map_shell_outputs(
    result: &ShellResult,
    op_node: &Rc<Node>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let return_type = match op_node.inferred.as_deref() {
        Some(crate::v1_std_core::InferredNode::Resolved { node }) => node.clone(),
        _ => {
            return Ok(str_value(result.stdout.retained_text.clone()));
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
        if is_optional_field
            && result.exit_code != 0
            && matches!(from_key.as_deref(), Some("stdout" | "stderr"))
        {
            fields.push((ctx.sym(&field_name), Value::Null));
            continue;
        }
        let value = match from_key.as_deref() {
            Some(key) => shell_evidence_value(result, key).unwrap_or_else(|| match key {
                "exit_success" => Value::Bool(result.exit_code == 0),
                "exit_code" => Value::Int(result.exit_code as i64),
                "stdout_lines" => {
                    let lines: Vec<Value> = result
                        .stdout
                        .retained_text
                        .lines()
                        .map(|l| str_value(l.to_string()))
                        .collect();
                    list_value((lines))
                }
                _ => Value::Null,
            }),
            None => Value::Null,
        };
        if matches!(value, Value::Null) {
            if let Some(v) = match field_name.as_str() {
                "success" => Some(Value::Bool(result.exit_code == 0)),
                "exit_code" => Some(Value::Int(result.exit_code as i64)),
                "stdout" => Some(str_value(result.stdout.retained_text.clone())),
                "stderr" => Some(str_value(result.stderr.retained_text.clone())),
                "exists" => Some(Value::Bool(result.exit_code == 0)),
                _ => None,
            } {
                fields.push((ctx.sym(&field_name), v));
                continue;
            }
        }
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
                    return Some(s.to_string());
                }
            }
        }
    }
    None
}

fn write_file_owner_only(path: &str, content: &[u8]) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        // create_new (O_EXCL): mode applies at creation; an existing path refuses rather than
        // truncating with stale permissions (create+truncate would leave prior mode on reuse).
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)?;
        file.write_all(content)?;
        Ok(())
    }
    #[cfg(not(unix))]
    {
        use std::io::{Error, ErrorKind};
        Err(Error::new(
            ErrorKind::Unsupported,
            "write_owner_only refused: owner-only mode-at-creation is unavailable on this platform",
        ))
    }
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
            "write_owner_only" => {
                let content = match param_env.lookup(ctx.sym("content")) {
                    Some(v) => format!("{}", v),
                    None => {
                        return Err(InterpError::TypeError {
                            msg: format!(
                                "file write_owner_only operation missing `content` argument for {}",
                                path
                            ),
                        })
                    }
                };
                let byte_count = content.len() as i64;
                trace_emit(
                    OutputChannel::ShellTrace,
                    &format!("[file] write_owner_only {} ({} bytes)", path, byte_count),
                );
                return match write_file_owner_only(&path, content.as_bytes()) {
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
                };
            }
            other => {
                return Err(InterpError::TypeError {
                    msg: format!(
                        "file transport verb '{other}' is not a known action (delete, list, write_owner_only)"
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
            return Ok(str_value(result.content.clone()));
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
            "path" => str_value(result.path.clone()),
            "error" => str_value(result.error.clone()),
            "content" => str_value(result.content.clone()),
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

/// HAND-RUST GATE explicit deferral (review 46616), covering this function and the
/// REST outcome/replay bridge below it through `dispatch_rest`: bounded growth in the
/// existing seed interpreter, not a new Rust authority and not a second transport
/// convention. Every DECISION this bridge makes is modeled — the outcome states are
/// `extdeps.transports.rest` `RestOutcome`, the observation states are
/// `RestExchangeObservation`, replay identity and its 0/1/many lookup are
/// `rest_bound_invocation_eq` / `rest_exchange_fixture_lookup`, and the resolution is
/// selected by calling `rest_exchange_resolution` back into `.dag`. What is seed-side
/// is the projection of those decisions onto the operation's declared output record,
/// which requires the interpreter's own `Value`/`Node` representation.
///
/// Lane: ROADMAP `v1-interpreter-quarantine` → `v1-interpreter-delete`, counted against
/// `v1-honest-frontier`.
///
/// EARLIER, NARROWER deletion condition than the lane's, and the one that should fire
/// first — stated in the SCOPE paragraph of `rest_outcome_note`: when the `response`
/// block becomes the single authority for a result and `output` is DERIVED from its 2xx
/// arm, every operation carries its outcome without declaring one. At that point the
/// opt-in disappears and `rest_outcome_output_field` deletes outright, because there is
/// no longer a field to detect; the `if status >= 400` raise below it deletes in the
/// same motion, since it exists only to serve operations that declared no outcome.
/// Checkable by execution: `rest_operation_without_outcome_still_refuses` is the witness
/// that pins the opt-in's existence, so it is the one that must be REPLACED (not merely
/// kept green) when the seam dissolves — a `Legacy` operation with no outcome field can
/// no longer exist.
///
/// The opt-in migration seam declared by extdeps.transports.rest.RestOutcome.
///
/// An operation asks for transport observations by declaring an output field whose
/// type is RestOutcome. Operations without that field retain the legacy raise-on-
/// failure behavior until the response table itself becomes the universal result
/// authority; see rest_outcome_note. Inspect the field's TYPE, not its spelling, so
/// callers may choose a domain-appropriate field name without creating another
/// transport convention.
fn rest_outcome_output_field(op_node: &Rc<Node>, ctx: &InterpContext) -> Option<String> {
    let return_type = match op_node.inferred.as_deref()? {
        InferredNode::Resolved { node } => node,
        _ => return None,
    };
    return_type.children.iter().find_map(|field| {
        let field_type = match field.inferred.as_deref()? {
            InferredNode::Resolved { node } => node,
            _ => return None,
        };
        let type_name = authored_name_at(ctx.si(), field_type.clone());
        (type_name.rsplit('.').next() == Some("RestOutcome"))
            .then(|| authored_name_at(ctx.si(), field.clone()))
    })
}

fn rest_outcome_variant(
    ctx: &InterpContext,
    variant: &str,
    mut fields: Vec<(Symbol, Value)>,
) -> Value {
    fields.sort_unstable_by_key(|(name, _)| name.0);
    Value::Variant {
        type_name: ctx.sym("RestOutcome"),
        variant_name: ctx.sym(variant),
        fields: Rc::new(fields),
    }
}

fn rest_status_refused_value(ctx: &InterpContext, status: u16, body: String) -> Value {
    rest_outcome_variant(
        ctx,
        "RestStatusRefused",
        vec![
            (ctx.sym("status"), Value::Int(status as i64)),
            (ctx.sym("body"), str_value(body)),
        ],
    )
}

fn rest_transport_refused_value(ctx: &InterpContext, cause: String) -> Value {
    rest_outcome_variant(
        ctx,
        "RestTransportRefused",
        vec![(ctx.sym("cause"), str_value(cause))],
    )
}

fn rest_body_undecodable_value(ctx: &InterpContext, status: u16, cause: String) -> Value {
    rest_outcome_variant(
        ctx,
        "RestBodyUndecodable",
        vec![
            (ctx.sym("status"), Value::Int(status as i64)),
            (ctx.sym("cause"), str_value(cause)),
        ],
    )
}

#[derive(Clone)]
enum RestBodyObservationHost {
    Read(String),
    ReadRefused(String),
}

#[derive(Clone)]
enum RestExchangeObservationHost {
    Response {
        status: u16,
        body: RestBodyObservationHost,
    },
    ExchangeRefused(String),
}

enum RestExchangeSelectionHost {
    Real,
    Replay(RestExchangeObservationHost),
}

fn rest_variant(ctx: &InterpContext, type_name: &str, variant_name: &str) -> Value {
    Value::Variant {
        type_name: ctx.sym(type_name),
        variant_name: ctx.sym(variant_name),
        fields: Rc::new(vec![]),
    }
}

fn rest_auth_identity_value(
    auth: &AuthResolution,
    basic_header: Option<&str>,
    ctx: &InterpContext,
) -> Value {
    let secret_identity = match basic_header {
        Some(header) => Some(("BasicCredentials".to_string(), header.to_string())),
        None => match auth {
            AuthResolution::Resolved { header, token } => Some((
                if header == "Authorization" {
                    "BearerToken".to_string()
                } else {
                    format!("HeaderToken:{}", header)
                },
                token.clone(),
            )),
            AuthResolution::NoAuthDeclared | AuthResolution::DeclaredButUnwired { .. } => None,
        },
    };
    match secret_identity {
        None => rest_variant(ctx, "RestAuthSensitiveIdentity", "RestUnauthenticated"),
        Some((scheme, secret)) => {
            // Only the hash crosses into the authored fixture carrier. The secret is
            // never persisted in, or made displayable through, a replay identity.
            //
            // The digest is minted through `v1_rt::atom_identity_hash` — the SAME fnv1a64
            // primitive `std.content_hash.content_hash_atom` is realized by — for two
            // load-bearing reasons: (1) the model types `RestAuthenticated.digest` as
            // `Fnv1a64Structural`, and a `DefaultHasher` (SipHash) hex here would be a value
            // from outside that family wearing the family's carrier (the labeling the
            // constructor-wall note forbids); (2) an authored fixture can reproduce this
            // digest through the modeled surface — `content_hash_atom(value:
            // "<scheme>\0<secret>")` — so authenticated replay identities are expressible
            // in .dag without pinning opaque literals. Pinned by
            // `rest_authenticated_identity_matches_dag_constructed_value` in
            // src/v1/tests/src/cross_representation_equality_test.rs.
            let digest = v1_rt::atom_identity_hash(format!("{}\0{}", scheme, secret));
            Value::Variant {
                type_name: ctx.sym("RestAuthSensitiveIdentity"),
                variant_name: ctx.sym("RestAuthenticated"),
                fields: Rc::new(sorted_fields(vec![
                    (ctx.sym("scheme"), str_value(scheme)),
                    (ctx.sym("digest"), fnv1a64_structural_value(digest, ctx)),
                ])),
            }
        }
    }
}

/// The runtime `Value` shape of `std.content_hash.Fnv1a64Structural` — the single mint
/// for every seed-side crossing into a `Fnv1a64Structural`-typed carrier of the REST
/// replay model (`RestBoundOperationInvocation.input_digest`, `RestAuthenticated.digest`).
/// A bare `Value::Str` at either position is the model↔realization fork: fixture matching
/// compares a record against a string and silently never matches (DESIGN §5).
fn fnv1a64_structural_value(digest: String, ctx: &InterpContext) -> Value {
    Value::Record {
        type_name: ctx.sym("Fnv1a64Structural"),
        fields: Rc::new(sorted_fields(vec![(ctx.sym("digest"), str_value(digest))])),
    }
}

/// Witness export: lets the tests crate pin `rest_auth_identity_value`'s authenticated arm
/// `==`-equal to a dag-authored `RestAuthenticated { scheme, digest: content_hash_atom(…) }`,
/// so a drift on either side of the seam (mint shape, hash family, or preimage layout) goes
/// red instead of silently failing every authenticated fixture match.
#[cfg(any(test, feature = "interp_test_witness"))]
pub fn rest_authenticated_identity_for_witness(token: &str, ctx: &InterpContext) -> Value {
    rest_auth_identity_value(
        &AuthResolution::Resolved {
            header: "Authorization".to_string(),
            token: token.to_string(),
        },
        None,
        ctx,
    )
}

fn rest_uri_value(url: &str, ctx: &InterpContext) -> InterpResult<Value> {
    let (scheme, locator) = if let Some(locator) = url.strip_prefix("https://") {
        ("Https", locator)
    } else if let Some(locator) = url.strip_prefix("http://") {
        ("Http", locator)
    } else {
        return Err(InterpError::TypeError {
            msg: format!("REST target is not an absolute HTTP(S) URI: {}", url),
        });
    };
    Ok(Value::Record {
        type_name: ctx.sym("Uri"),
        fields: Rc::new(sorted_fields(vec![
            (ctx.sym("scheme"), rest_variant(ctx, "UriScheme", scheme)),
            (ctx.sym("locator"), str_value(locator.to_string())),
        ])),
    })
}

fn rest_bound_invocation_value(
    service_node: &Rc<Node>,
    op_node: &Rc<Node>,
    method: &str,
    url: &str,
    param_env: &Rc<Env>,
    auth: &AuthResolution,
    basic_header: Option<&str>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let service = authored_name_at(ctx.si(), service_node.clone());
    let operation = authored_name_at(ctx.si(), op_node.clone());
    let at = operation_ref_value(&op_node.span.file, &service, &operation, ctx);
    let input_digest =
        crate::recorded_fixture::content_hash_service_inputs(op_node, param_env, ctx);
    Ok(Value::Record {
        type_name: ctx.sym("RestBoundOperationInvocation"),
        fields: Rc::new(sorted_fields(vec![
            (ctx.sym("at"), at),
            (ctx.sym("method"), rest_variant(ctx, "HttpMethod", method)),
            (ctx.sym("target"), rest_uri_value(url, ctx)?),
            (
                ctx.sym("input_digest"),
                // Grounding, gunbc#7480 Phase A: RestBoundOperationInvocation.input_digest is
                // modelled as std.content_hash Fnv1a64Structural (the structural family member
                // content_hash_service_inputs actually produces), not as bare text. The realization
                // must construct the SAME shape the model declares, or fixture matching compares a
                // record against a string and silently never matches -- the model/realization fork.
                fnv1a64_structural_value(input_digest, ctx),
            ),
            (
                ctx.sym("auth_identity"),
                rest_auth_identity_value(auth, basic_header, ctx),
            ),
        ])),
    })
}

fn rest_observation_from_value(
    value: &Value,
    ctx: &InterpContext,
) -> InterpResult<RestExchangeObservationHost> {
    let malformed = || InterpError::TypeError {
        msg: "REST replay fixture carried a malformed RestExchangeObservation".to_string(),
    };
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = value
    else {
        return Err(malformed());
    };
    match ctx.resolve(*variant_name).as_str() {
        "RestExchangeRefused" => match ctx.field(fields, "cause") {
            Some(Value::Str(cause)) if !cause.is_empty() => Ok(
                RestExchangeObservationHost::ExchangeRefused(cause.to_string()),
            ),
            _ => Err(malformed()),
        },
        "RestResponseObserved" => {
            let status = match ctx.field(fields, "status") {
                Some(Value::Int(status)) if (100..=599).contains(status) => *status as u16,
                _ => return Err(malformed()),
            };
            let body = match ctx.field(fields, "body") {
                Some(Value::Variant {
                    variant_name,
                    fields,
                    ..
                }) if ctx.sym_eq(*variant_name, "RestBodyRead") => {
                    match ctx.field(fields, "body") {
                        Some(Value::Str(body)) => RestBodyObservationHost::Read(body.to_string()),
                        _ => return Err(malformed()),
                    }
                }
                Some(Value::Variant {
                    variant_name,
                    fields,
                    ..
                }) if ctx.sym_eq(*variant_name, "RestBodyReadRefused") => {
                    match ctx.field(fields, "cause") {
                        Some(Value::Str(cause)) if !cause.is_empty() => {
                            RestBodyObservationHost::ReadRefused(cause.to_string())
                        }
                        _ => return Err(malformed()),
                    }
                }
                _ => return Err(malformed()),
            };
            Ok(RestExchangeObservationHost::Response { status, body })
        }
        _ => Err(malformed()),
    }
}

fn rest_exchange_selection(
    invocation: Value,
    ctx: &InterpContext,
) -> InterpResult<RestExchangeSelectionHost> {
    let Some(frame) = current_witness_evaluation_frame() else {
        return Ok(RestExchangeSelectionHost::Real);
    };
    let Value::Record { fields, .. } = frame else {
        return Err(InterpError::TypeError {
            msg: "witness evaluation frame is malformed".to_string(),
        });
    };
    let envelope =
        ctx.field(&fields, "envelope")
            .cloned()
            .ok_or_else(|| InterpError::TypeError {
                msg: "witness evaluation frame has no envelope".to_string(),
            })?;
    let fixtures = ctx
        .field(&fields, "rest_fixtures")
        .cloned()
        .ok_or_else(|| InterpError::TypeError {
            msg: "witness evaluation frame has no REST fixtures".to_string(),
        })?;
    let resolution = run_in_context_with_args(
        ctx,
        "rest_exchange_resolution",
        &[
            (Some("env".to_string()), envelope),
            (Some("fixtures".to_string()), fixtures),
            (Some("invocation".to_string()), invocation),
        ],
        false,
    )?;
    let Value::Variant {
        variant_name,
        fields,
        ..
    } = resolution
    else {
        return Err(InterpError::TypeError {
            msg: "REST handler selection returned a malformed resolution".to_string(),
        });
    };
    match ctx.resolve(variant_name).as_str() {
        "RestRealExchangeRequired" => Ok(RestExchangeSelectionHost::Real),
        "RestReplayExchangeFound" => {
            let observation =
                ctx.field(&fields, "observation")
                    .ok_or_else(|| InterpError::TypeError {
                        msg: "REST replay resolution omitted its observation".to_string(),
                    })?;
            Ok(RestExchangeSelectionHost::Replay(
                rest_observation_from_value(observation, ctx)?,
            ))
        }
        "RestReplayExchangeAbsent" => Err(InterpError::TypeError {
            msg: "missing exact REST replay fixture for bound invocation; real transport refused"
                .to_string(),
        }),
        "RestReplayExchangeAmbiguous" => {
            let count = match ctx.field(&fields, "count") {
                Some(Value::Int(count)) => *count,
                _ => 0,
            };
            Err(InterpError::TypeError {
                msg: format!(
                    "ambiguous exact REST replay fixture for bound invocation: {} matches; real transport refused",
                    count
                ),
            })
        }
        "RestExchangeHandlerUncovered" => Err(InterpError::TypeError {
            msg: "REST invocation is not covered by the witness frame; real transport refused"
                .to_string(),
        }),
        other => Err(InterpError::TypeError {
            msg: format!("unrecognized REST exchange resolution: {}", other),
        }),
    }
}

fn observe_rest_exchange(
    selection: RestExchangeSelectionHost,
    request: ureq::Request,
    body_json: Option<serde_json::Value>,
) -> RestExchangeObservationHost {
    if let RestExchangeSelectionHost::Replay(observation) = selection {
        return observation;
    }
    let response = if let Some(json) = body_json {
        request
            .set("Content-Type", "application/json")
            .send_string(&json.to_string())
    } else {
        request.call()
    };
    match response {
        Ok(response) => {
            let status = response.status();
            let body = response.into_string().map_or_else(
                |error| RestBodyObservationHost::ReadRefused(error.to_string()),
                RestBodyObservationHost::Read,
            );
            RestExchangeObservationHost::Response { status, body }
        }
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().map_or_else(
                |error| RestBodyObservationHost::ReadRefused(error.to_string()),
                RestBodyObservationHost::Read,
            );
            RestExchangeObservationHost::Response { status, body }
        }
        Err(error) => RestExchangeObservationHost::ExchangeRefused(error.to_string()),
    }
}

fn decide_rest_exchange(
    observation: RestExchangeObservationHost,
    op_node: &Rc<Node>,
    response_format: &str,
    outcome_field: Option<&str>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let (status, body) = match observation {
        RestExchangeObservationHost::ExchangeRefused(cause) => {
            return match outcome_field {
                Some(field) => Ok(attach_rest_outcome(
                    None,
                    op_node,
                    field,
                    rest_transport_refused_value(ctx, cause),
                    ctx,
                )),
                None => Err(InterpError::TypeError {
                    msg: format!("HTTP request failed: {}", cause),
                }),
            };
        }
        RestExchangeObservationHost::Response {
            status,
            body: RestBodyObservationHost::ReadRefused(cause),
        } => {
            return match outcome_field {
                Some(field) => Ok(attach_rest_outcome(
                    None,
                    op_node,
                    field,
                    rest_body_undecodable_value(ctx, status, cause),
                    ctx,
                )),
                None => Err(InterpError::TypeError {
                    msg: format!("HTTP {} body unreadable: {}", status, cause),
                }),
            };
        }
        RestExchangeObservationHost::Response {
            status,
            body: RestBodyObservationHost::Read(body),
        } => (status, body),
    };
    if !(200..300).contains(&status) {
        if let Some(field) = outcome_field {
            return Ok(attach_rest_outcome(
                None,
                op_node,
                field,
                rest_status_refused_value(ctx, status, body),
                ctx,
            ));
        }
    }
    if status >= 400 {
        return Err(InterpError::TypeError {
            msg: format!("HTTP {}: {}", status, body),
        });
    }
    let mapped = if response_format == "Text" {
        map_response_to_value(&body, None, op_node, ctx)?
    } else {
        let json: serde_json::Value =
            serde_json::from_str(&body).unwrap_or_else(|_| serde_json::Value::String(body));
        map_response_to_value_json(&json, op_node, ctx)?
    };
    match outcome_field {
        Some(field) => Ok(attach_rest_outcome(
            Some(mapped),
            op_node,
            field,
            rest_outcome_variant(ctx, "RestOk", vec![]),
            ctx,
        )),
        None => Ok(mapped),
    }
}

/// Project an observation into the operation's declared output record. On a
/// non-success outcome the ordinary body-derived fields are deliberately Null:
/// RestOutcome is the only inhabited branch and therefore the only fact a caller
/// can consume. On RestOk, preserve the already-decoded body fields and replace
/// just the outcome field.
fn attach_rest_outcome(
    mapped: Option<Value>,
    op_node: &Rc<Node>,
    outcome_field: &str,
    outcome: Value,
    ctx: &InterpContext,
) -> Value {
    let mut fields = match mapped {
        Some(Value::Record { fields, .. }) => (*fields).clone(),
        _ => op_node
            .inferred
            .as_deref()
            .and_then(|inferred| match inferred {
                InferredNode::Resolved { node } => Some(node),
                _ => None,
            })
            .map(|return_type| {
                return_type
                    .children
                    .iter()
                    .map(|field| {
                        (
                            ctx.sym(&authored_name_at(ctx.si(), field.clone())),
                            Value::Null,
                        )
                    })
                    .collect()
            })
            .unwrap_or_default(),
    };
    let outcome_sym = ctx.sym(outcome_field);
    match fields.iter_mut().find(|(name, _)| *name == outcome_sym) {
        Some((_, value)) => *value = outcome,
        None => fields.push((outcome_sym, outcome)),
    }
    fields.sort_unstable_by_key(|(name, _)| name.0);
    Value::Record {
        type_name: ctx.sym(&authored_name_at(ctx.si(), op_node.clone())),
        fields: Rc::new(fields),
    }
}

fn dispatch_rest(
    service_node: &Rc<Node>,
    op_node: &Rc<Node>,
    transport: &Rc<Node>,
    param_env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let si = ctx.si();

    // An unresolvable endpoint REFUSES here rather than defaulting to "". An empty
    // base produces the same `RelativeUrlWithoutBase` failure as a garbage one, so
    // `unwrap_or_default()` was a second way for the same defect to arrive unlocated.
    // An ABSENT key is its own refusal rather than the same one: "declared nothing"
    // and "declared something unreadable" are different authoring mistakes, and the
    // fix for each names a different edit.
    let base_url =
        match find_service_config_string(service_node, "svc_endpoint", &si, param_env, ctx) {
            Some(Ok(url)) => url,
            Some(Err(spelled)) => {
                return Err(InterpError::ServiceConfigUnresolved {
                    key: "endpoint".to_string(),
                    spelled,
                })
            }
            None => {
                return Err(InterpError::ServiceConfigMissing {
                    key: "endpoint".to_string(),
                    service: service_node.name.clone(),
                })
            }
        };

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
            service: match find_service_config_string(
                service_node,
                "svc_endpoint",
                &si,
                param_env,
                ctx,
            ) {
                Some(Ok(url)) => url,
                Some(Err(spelled)) => format!("<unresolved: {}>", spelled),
                None => "<unknown>".to_string(),
            },
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
    let mut basic_auth_header: Option<String> = None;
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
                ("username", Value::Str(s)) => username = Some(s.to_string()),
                ("password", Value::Str(s)) => password = Some(s.to_string()),
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
                basic_auth_header = Some(header_val.clone());
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

    // Bind replay identity to the HTTP client's fully realized target, including
    // its encoded query string, rather than the pre-query service/path join.
    let realized_target = request.url().to_string();
    let invocation = rest_bound_invocation_value(
        service_node,
        op_node,
        &method,
        &realized_target,
        param_env,
        &auth,
        basic_auth_header.as_deref(),
        ctx,
    )?;
    let selection = rest_exchange_selection(invocation, ctx)?;
    let observation = observe_rest_exchange(selection, request, body_json);
    let outcome_field = rest_outcome_output_field(op_node, ctx);
    decide_rest_exchange(
        observation,
        op_node,
        &response_format,
        outcome_field.as_deref(),
        ctx,
    )
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
                    token: tok.to_string(),
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
            return Some(s.to_string());
        }
    }
    None
}

// A service-config value is EVALUATED, exactly like the `path` template two lines
// below its only caller — one authority for "what does this config entry say", not
// two. The previous reading had a literal fast-path plus a fallback that returned
// `authored_name_at`, i.e. the SOURCE TEXT of the identifier. So a config written as
// a data reference resolved to its own spelling: `endpoint: default_api_base` became
// the string "default_api_base", which is a plausible non-empty value and a nonsense
// base URL. Every `github.Pulls` caller in the corpus has been failing on it with
// `RelativeUrlWithoutBase` — the service is modeled, cited, mock-covered and has
// production callers, and its live path had never once succeeded.
//
// The fallback is deleted rather than repaired because it was the thing that hid the
// defect: "the configured literal" and "the name of something I could not resolve"
// were both returned as `Some(String)`, so the failure could only surface downstream
// as a malformed URL instead of as a located refusal at the config read.
fn find_service_config_string(
    service_node: &Rc<Node>,
    key: &str,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    param_env: &Rc<Env>,
    ctx: &InterpContext,
) -> Option<Result<String, String>> {
    for prop in service_node.properties.iter() {
        let name = field_init_node_name_at(prop.clone(), si.clone());
        if name == key {
            let val_node = field_init_node_value(prop.clone());
            let spelled = authored_name_at(si.clone(), val_node.clone());
            // Narrowed to Str for the same reason the deleted branch narrowed to LitStr:
            // Display renders every Value, so `format!` would turn Null into "null" and
            // Int into its digits, and the non-empty check would wave both through as a
            // base URL. That is this function's original defect one layer down.
            return Some(match eval_expr(&val_node, param_env, ctx) {
                Ok(Value::Str(s)) if !s.is_empty() => Ok(s.to_string()),
                Ok(_) => Err(spelled),
                Err(_) => Err(spelled),
            });
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
            serde_json::Value::String(s.to_string())
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
                .map(|s| serde_json::Value::String(s.to_string()))
                .collect(),
        ),
        Value::Map(m) => {
            let mut obj = serde_json::Map::with_capacity(m.len());
            for (k, v) in m.iter() {
                let key = match &k.key {
                    Value::Str(s) => s.to_string(),
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
        _ => return Ok(str_value(text.to_string())),
    };
    let children = &return_type.children;
    if children.is_empty() {
        return Ok(str_value(text.to_string()));
    }
    if children.len() == 1 {
        return Ok(str_value(text.to_string()));
    }
    let mut fields: Vec<(Symbol, Value)> = Vec::new();
    for child in children.iter() {
        let field_name = authored_name_at(ctx.si(), child.clone());
        fields.push((ctx.sym(&field_name), str_value(text.to_string())));
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
        serde_json::Value::String(s) => str_value(s.clone()),
        serde_json::Value::Array(arr) => {
            list_value(arr.iter().map(json_to_value).collect::<Vec<_>>())
        }
        serde_json::Value::Object(obj) => {
            let fields: HamtMap<CanonKey, Value> = obj
                .iter()
                .filter_map(|(k, v)| {
                    CanonKey::new(str_value(k.clone())).map(|ck| (ck, json_to_value(v)))
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
        Value::Str(s) if !s.is_empty() => Some(s.to_string()),
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
    if !ctx.indexes.service_ops.contains_key("Filesystem.Read") {
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

    let args = [(Some("path".to_string()), str_value(path))];
    let result = eval_service_call(
        "Filesystem",
        "Read",
        &args,
        &Env::empty(),
        ctx,
        // Interpreter-own seam (import resolution reading a source file): no
        // .dag call node, and a failed read here IS a fault, so ExpectSuccess is
        // the stated arm rather than an inherited default.
        ExpectationDeclaration::Declared(ExpectedOutcome::ExpectSuccess),
    )?;

    let (content, success, error) = match result {
        Value::Record { fields, .. } => {
            let success = matches!(ctx.field(&fields, "success"), Some(Value::Bool(true)));
            let content = match ctx.field(&fields, "content") {
                Some(Value::Str(s)) => s.to_string(),
                _ => String::new(),
            };
            let error = match ctx.field(&fields, "error") {
                Some(Value::Str(s)) => s.to_string(),
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
        fields: Rc::new(vec![(ctx.sym("content"), str_value(content))]),
    })
}

/// Host tap for `v2.compiler.emit_host.run_host_process` (kernel-D emit_host transport):
/// materialize a workspace from resolved `{path, text}` rows, run the build argvs then the
/// run argv with typed argv (no shell), and return exit/stdout/stderr/build-log as data.
/// Wet-mode only — hermetic execution refuses instead of mocking (no fabricated receipt).
/// The effects flip (build_transport_admission.dag: the intrinsic "runs only on an
/// Permit verdict"): host builds are admitted by the modeled build_workspace_grant
/// envelope, not by execution mode — the verdict is path containment, mode-independent,
/// so the same law holds hermetic and wet. Anything but Permit is a typed refusal;
/// the per-file escape guard below stays as the realization-side belt.
fn require_permitted_transport(
    admission_arg: Option<&Value>,
    ctx: &InterpContext,
    intrinsic: &str,
) -> InterpResult<()> {
    match admission_arg {
        Some(Value::Variant {
            type_name,
            variant_name,
            ..
        }) if ctx.sym_eq(*type_name, "AccessDecision") && ctx.sym_eq(*variant_name, "Permit") => {
            Ok(())
        }
        Some(Value::Variant {
            type_name,
            variant_name,
            ..
        }) => Err(InterpError::TypeError {
            msg: format!(
                "{intrinsic} refuses: transport not permitted (decision {}::{}, expected \
                 AccessDecision::Permit from build_transport_admissible)",
                ctx.resolve(*type_name),
                ctx.resolve(*variant_name)
            ),
        }),
        _ => Err(InterpError::TypeError {
            msg: format!(
                "{intrinsic} refuses: missing authorization decision (AccessDecision required; \
                 route through run_host_process_admitted)"
            ),
        }),
    }
}

fn eval_emit_host_run_transport_builtin(
    admission_arg: Option<&Value>,
    files_arg: Option<&Value>,
    build_arg: Option<&Value>,
    run_arg: Option<&Value>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    ctx.effect_dispatch_count
        .set(ctx.effect_dispatch_count.get().wrapping_add(1));
    require_permitted_transport(admission_arg, ctx, "emit_host_run_transport")?;

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

/// SCAFFOLD (§7 seed-retained HAND-RUST — authority: gunbc.roadmap_authority
/// v1-materialization-kernel; receipt: dag/std/emit_on_demand.dag P3 kernel +
/// extdeps.realization.emit_on_demand_host + emit_on_demand_kernel_witness_test):
/// content-addressed emit_host transport persists workspace under workspace_dir and
/// skips build when `.native_ready` is present. workspace_dir carries the caller's
/// computation and input-realization segments; this boundary derives the actual
/// resolved build-context identity and appends it before consulting the marker
/// (the effective path modeled by
/// extdeps.realization.emit_on_demand_host.native_cache_resolved_build_context_workspace_root).
/// A different closure, materialized input, build argv, resolved compiler,
/// admitted subprocess environment, or Cargo configuration MUST therefore land
/// in a different workspace (benign-by-identity on partial writes before
/// `.native_ready`). `.native_ready`
/// is written only after a successful run (not after build alone): the P3 kernel's
/// warm boundary is build+run proof, so a transient run failure must not skip
/// rebuild on retry. Registered in 04_method.dag as
/// emit_host_run_transport_cached; dissolve-on: witness_realization_kernel emits
/// this builtin from v2 self-hosted transport rows (same dissolution as
/// emit_host_run_transport seed handler).
/// HAND-RUST GATE explicit deferral: this is bounded growth in the existing seed
/// file, not a census-shrink receipt and not a new Rust authority. Its lane is
/// ROADMAP "Make native materialization the shared execution kernel",
/// docs/plans/witness-realization-plan.md P3/P6, with the concrete deletion row
/// dag/gunbc/v1_deletion_plan.dag ^witness_realization_kernel. Delete these
/// observation/apply helpers when the self-emitted transport consumes the modeled
/// ResolvedBuildContext and the dispatcher-change, environment-change, and
/// cold/warm agreement witnesses remain green without them.
/// Durable re-root (realization-side config, GUNBC_RESOLVED_GRAPH_CACHE_DIR
/// precedent): the root is WHERE the cache lives, never WHAT identifies an
/// artifact — the content-hash path component stays the key. Opt-in; only the
/// declared /tmp/gunbc_ scratch prefix (std.emit_on_demand root authority) is
/// rebased, so an arbitrary caller path never silently moves. SINGLE authority
/// for every host op on the native-cache namespace: the cached run transport AND
/// emit_host_native_cache_evict share this mapping, so eviction always targets
/// the same workspace the transport warms (a fork here silently un-evicts).
fn native_cache_rebase_workspace_dir(workspace_dir: String) -> String {
    match std::env::var("GUNBC_NATIVE_CACHE_ROOT") {
        Ok(root) if !root.trim().is_empty() => match workspace_dir.strip_prefix("/tmp/") {
            Some(rest) if rest.starts_with("gunbc_") => {
                format!("{}/{}", root.trim_end_matches('/'), rest)
            }
            _ => workspace_dir,
        },
        _ => workspace_dir,
    }
}

/// Evict one native-cache workspace (the witness content-change/cold legs' evictor).
/// Lives beside the cached transport so both sides of the cache lifecycle read the
/// SAME rebase mapping; a shell.Remove on the .dag-composed /tmp path would miss a
/// rebased workspace and falsely leave it warm. Wet-only like the transport's other
/// host effects; removing an absent workspace is a no-op success (idempotent evict).
fn eval_emit_host_native_cache_evict_builtin(
    workspace_dir_arg: Option<&Value>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    ctx.effect_dispatch_count
        .set(ctx.effect_dispatch_count.get().wrapping_add(1));
    if ctx.execution_mode.is_hermetic() {
        return Err(InterpError::TypeError {
            msg: "hermetic mode: emit_host_native_cache_evict refuses filesystem removal \
                  (no mock arm; run wet)"
                .to_string(),
        });
    }
    let workspace_dir =
        free_monoid_to_string(workspace_dir_arg.ok_or_else(|| InterpError::TypeError {
            msg: "emit_host_native_cache_evict requires a workspace_dir argument".to_string(),
        })?)
        .ok_or_else(|| InterpError::TypeError {
            msg: "emit_host_native_cache_evict: workspace_dir must be String".to_string(),
        })?;
    let workspace_dir = native_cache_rebase_workspace_dir(workspace_dir);
    match std::fs::remove_dir_all(&workspace_dir) {
        Ok(()) => Ok(Value::Bool(true)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Value::Bool(true)),
        Err(e) => Err(InterpError::TypeError {
            msg: format!("emit_host_native_cache_evict: {workspace_dir}: {e}"),
        }),
    }
}

fn eval_emit_host_run_transport_cached_builtin(
    admission_arg: Option<&Value>,
    workspace_dir_arg: Option<&Value>,
    files_arg: Option<&Value>,
    build_arg: Option<&Value>,
    run_arg: Option<&Value>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    ctx.effect_dispatch_count
        .set(ctx.effect_dispatch_count.get().wrapping_add(1));
    require_permitted_transport(admission_arg, ctx, "emit_host_run_transport_cached")?;

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

    run_cached_process_spec(
        ctx,
        workspace_dir,
        &workspace_files,
        &build_argvs,
        &run_argv,
        false,
    )
}

/// Native-bundle execution seam used by the production selector. The `.dag` selector owns
/// the exact files and argv values; this seed helper only realizes that typed process spec
/// through the same cache/toolchain identity path as `emit_host_run_transport_cached`.
pub fn run_native_bundle_process_cached(
    ctx: &InterpContext,
    workspace_dir: String,
    workspace_files: &[(String, String)],
    build_argvs: &[Vec<String>],
    run_argv: &[String],
) -> InterpResult<Value> {
    if !ctx.execution_mode.is_wet_dispatch() {
        return Err(InterpError::TypeError {
            msg: "native bundle process refuses outside Wet/Record execution mode".to_string(),
        });
    }
    if build_argvs.iter().any(|argv| argv.is_empty()) || run_argv.is_empty() {
        return Err(InterpError::TypeError {
            msg: "native bundle process refuses an empty build/run argv".to_string(),
        });
    }
    run_cached_process_spec(
        ctx,
        workspace_dir,
        workspace_files,
        build_argvs,
        run_argv,
        true,
    )
}

fn run_cached_process_spec(
    ctx: &InterpContext,
    workspace_dir: String,
    workspace_files: &[(String, String)],
    build_argvs: &[Vec<String>],
    run_argv: &[String],
    require_transition_timing: bool,
) -> InterpResult<Value> {
    let workspace_dir = native_cache_rebase_workspace_dir(workspace_dir);
    let realization_workspace = std::path::PathBuf::from(&workspace_dir);
    std::fs::create_dir_all(&realization_workspace).map_err(|e| InterpError::TypeError {
        msg: format!("emit_host_run_transport_cached: workspace create failed: {e}"),
    })?;
    emit_host_materialize_workspace_files(&realization_workspace, &workspace_files)?;
    let build_environment = emit_host_constructed_build_environment();
    let resolved_build_context_identity = emit_host_resolved_build_context_identity(
        &build_argvs,
        &realization_workspace,
        &build_environment,
    )?;
    let workspace = realization_workspace.join(resolved_build_context_identity);
    std::fs::create_dir_all(&workspace).map_err(|e| InterpError::TypeError {
        msg: format!("emit_host_run_transport_cached: workspace create failed: {e}"),
    })?;

    emit_host_run_transport_cached_in_workspace(
        &workspace,
        &workspace_files,
        &build_argvs,
        &run_argv,
        &build_environment,
        ctx,
        require_transition_timing,
    )
}

/// Host-tool program resolution for the emit-host transports (fleet incident
/// 2026-07-22: srv2 runner env has no `cargo` on PATH — the repo-checkout build
/// steps get it via the CI prelude, but the transport spawns from an emitted
/// workspace with only the process env). Resolution order: bare name if it
/// resolves on PATH; else $CARGO_HOME/bin/<name>; else $HOME/.cargo/bin/<name>;
/// else refuse (DESIGN §5: never return the bare name and widen to ambient PATH
/// at spawn time — the absorbing fallback hermetic-tool-provisioning-design.md
/// §1 names).
///
/// HAND-RUST GATE explicit deferral (review 44883): this function is seed
/// retained, not a new resolver authority. Its lane is ROADMAP
/// `toolchain-single-resolver` (gunbc.roadmap_authority,
/// docs/plans/hermetic-tool-provisioning-design.md P2 — "one resolver",
/// handback: delete `resolve_host_tool_program` and the bash ladder). This PR
/// repairs only the fail-open terminal arm; it does not admit a parallel key or
/// grow the census. Delete the whole function when P2's `membership_reconcile`
/// instantiation routes emit-host spawns and the P2 RED control (unpinned tool
/// refuses before spawn) is witnessed in `.dag`.
///
/// A name containing `/` is treated as one of three cases:
/// - **`./<rel>`** — the `ProducedProgram` wire format from
///   `emit_host.dag` `process_program_name`; passed through because emit-host
///   spawns set `.current_dir(workspace)` and the path is workspace-relative.
/// - **Absolute path** — caller-declared executable; must exist as a file.
/// - **Other relative paths** (e.g. `target/release/foo`) — refused as
///   `HostToolRelativePathAmbiguous`: `is_file()` is process-cwd-relative but
///   spawn uses the workspace, so check and spawn would disagree.
/// Bare names are ambient divination; absolute paths are declared intent, but a
/// nonexistent path still refuses before `Command::new`.
fn resolve_host_tool_program(name: &str) -> InterpResult<String> {
    if name.contains('/') {
        if name.starts_with("./") {
            return Ok(name.to_string());
        }
        let path = std::path::Path::new(name);
        if !path.is_absolute() {
            return Err(InterpError::HostToolRelativePathAmbiguous {
                name: name.to_string(),
            });
        }
        if path.is_file() {
            return Ok(name.to_string());
        }
        return Err(InterpError::HostToolUnresolved {
            name: name.to_string(),
            probed: vec![name.to_string()],
        });
    }
    let mut probed = Vec::new();
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            if dir.is_empty() {
                continue;
            }
            let candidate = std::path::Path::new(dir).join(name);
            let candidate_str = candidate.to_string_lossy().into_owned();
            probed.push(candidate_str.clone());
            if candidate.is_file() {
                return Ok(candidate_str);
            }
        }
    }
    if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
        let candidate = std::path::Path::new(&cargo_home).join("bin").join(name);
        let candidate_str = candidate.to_string_lossy().into_owned();
        probed.push(candidate_str.clone());
        if candidate.is_file() {
            return Ok(candidate_str);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let candidate = std::path::Path::new(&home).join(".cargo/bin").join(name);
        let candidate_str = candidate.to_string_lossy().into_owned();
        probed.push(candidate_str.clone());
        if candidate.is_file() {
            return Ok(candidate_str);
        }
    }
    Err(InterpError::HostToolUnresolved {
        name: name.to_string(),
        probed,
    })
}

#[derive(Clone)]
struct EmitHostBuildEnvironment {
    entries: Vec<(std::ffi::OsString, std::ffi::OsString)>,
    digest: String,
}

/// Construct the complete environment admitted to build and run subprocesses.
/// Commands use env_clear() and receive exactly these rows, so an undeclared
/// ambient variable cannot affect an artifact outside the realization identity.
fn emit_host_constructed_build_environment() -> EmitHostBuildEnvironment {
    use std::os::unix::ffi::OsStrExt;

    fn admitted(name: &str) -> bool {
        const EXACT: &[&str] = &[
            "PATH",
            "HOME",
            "TMPDIR",
            "CC",
            "CXX",
            "AR",
            "LD_LIBRARY_PATH",
            "LIBRARY_PATH",
            "CPATH",
            "PKG_CONFIG_PATH",
            "SDKROOT",
            "MACOSX_DEPLOYMENT_TARGET",
        ];
        const PREFIXES: &[&str] = &[
            "CARGO_",
            "RUST",
            "CC_",
            "CXX_",
            "AR_",
            "CFLAGS",
            "CXXFLAGS",
            "CPPFLAGS",
            "LDFLAGS",
            "PKG_CONFIG_",
            "GO",
            "NODE_",
            "NPM_",
            "PYTHON",
        ];
        if matches!(
            name,
            "CARGO_TARGET_DIR" | "RUSTC_WRAPPER" | "RUSTC_WORKSPACE_WRAPPER"
        ) {
            return false;
        }
        EXACT.contains(&name) || PREFIXES.iter().any(|prefix| name.starts_with(prefix))
    }

    let mut entries: Vec<_> = std::env::vars_os()
        .filter(|(name, _)| name.to_str().map(admitted).unwrap_or(false))
        .collect();
    entries.sort_by(|(a, _), (b, _)| a.as_bytes().cmp(b.as_bytes()));

    let mut digest =
        v1_rt::atom_identity_hash("emit-host-constructed-build-environment-v1".to_string());
    for (name, value) in &entries {
        digest = v1_rt::hash_combine(digest, v1_rt::bytes_identity_hash(name.as_bytes()));
        digest = v1_rt::hash_combine(digest, v1_rt::bytes_identity_hash(value.as_bytes()));
    }
    EmitHostBuildEnvironment { entries, digest }
}

fn emit_host_apply_build_environment(
    command: &mut std::process::Command,
    environment: &EmitHostBuildEnvironment,
) {
    command.env_clear();
    command.envs(environment.entries.iter().cloned());
}

/// Seed mirror of std.artifact_store.artifact_realization_digest. The `.dag`
/// function is the authority for the ordered/tagged shape; this helper disappears
/// with the enclosing witness-realization HAND-RUST boundary.
fn emit_host_artifact_realization_digest(inputs: &[(&str, String)]) -> String {
    let mut digest = v1_rt::atom_identity_hash("artifact_store.realization".to_string());
    for (identity, content_digest) in inputs {
        let tagged = v1_rt::hash_combine(
            v1_rt::atom_identity_hash((*identity).to_string()),
            content_digest.clone(),
        );
        digest = v1_rt::hash_combine(digest, tagged);
    }
    digest
}

fn emit_host_cargo_configuration_digest(
    environment: &EmitHostBuildEnvironment,
    probe_workspace: &std::path::Path,
) -> InterpResult<String> {
    fn environment_path(
        environment: &EmitHostBuildEnvironment,
        name: &str,
    ) -> Option<std::path::PathBuf> {
        environment
            .entries
            .iter()
            .find(|(key, _)| key.to_str() == Some(name))
            .map(|(_, value)| std::path::PathBuf::from(value))
    }

    let mut candidates = Vec::new();
    if let Some(cargo_home) = environment_path(environment, "CARGO_HOME") {
        candidates.push(cargo_home.join("config"));
        candidates.push(cargo_home.join("config.toml"));
    } else if let Some(home) = environment_path(environment, "HOME") {
        candidates.push(home.join(".cargo/config"));
        candidates.push(home.join(".cargo/config.toml"));
    }
    for ancestor in probe_workspace.ancestors() {
        candidates.push(ancestor.join(".cargo/config"));
        candidates.push(ancestor.join(".cargo/config.toml"));
    }

    let mut digest = v1_rt::atom_identity_hash("emit-host-cargo-configuration-v1".to_string());
    for path in candidates {
        match std::fs::read(&path) {
            Ok(content) => {
                digest = v1_rt::hash_combine(
                    digest,
                    v1_rt::atom_identity_hash(path.to_string_lossy().into_owned()),
                );
                digest = v1_rt::hash_combine(digest, v1_rt::bytes_identity_hash(&content));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(InterpError::TypeError {
                    msg: format!(
                        "emit_host_run_transport_cached: read Cargo configuration {} failed: {e}",
                        path.display()
                    ),
                })
            }
        }
    }
    Ok(digest)
}

/// Observe the host tools which will realize a cached build. This is deliberately
/// inside the existing wet host-transport boundary: the `.dag` substrate owns the
/// effective path shape, while only the host can resolve PATH/rustup shims and read
/// executable bytes. Failure to resolve, read, or execute a version probe refuses
/// the cached realization; substituting a nominal label would recreate srv2-05.
///
/// Cargo is a driver, not the compiler identity. Its observation is therefore
/// paired with the rustc selected by the same process environment. The transport
/// removes RUSTC_WRAPPER and RUSTC_WORKSPACE_WRAPPER when building, so wrappers are
/// intentionally not part of this identity.
#[derive(Debug, Clone)]
struct ObservedToolIdentity {
    tool_name: String,
    observed_identity: String,
}

fn observe_tool_identity(
    requested: &str,
    version_args: &[&str],
    probe_workspace: &std::path::Path,
    environment: &EmitHostBuildEnvironment,
) -> InterpResult<ObservedToolIdentity> {
    let resolved = resolve_host_tool_program(requested)?;
    let canonical = std::fs::canonicalize(&resolved).map_err(|e| InterpError::TypeError {
        msg: format!(
            "emit_host_run_transport_cached: resolve build tool {requested:?} \
                 ({resolved:?}) failed: {e}"
        ),
    })?;
    let executable = std::fs::read(&canonical).map_err(|e| InterpError::TypeError {
        msg: format!(
            "emit_host_run_transport_cached: read resolved build tool {} failed: {e}",
            canonical.display()
        ),
    })?;
    let mut command = std::process::Command::new(&resolved);
    command.args(version_args).current_dir(probe_workspace);
    emit_host_apply_build_environment(&mut command, environment);
    let output = command.output().map_err(|e| InterpError::TypeError {
        msg: format!("emit_host_run_transport_cached: version probe for {requested:?} failed: {e}"),
    })?;
    if !output.status.success() {
        return Err(InterpError::TypeError {
            msg: format!(
                "emit_host_run_transport_cached: version probe for {requested:?} \
                 exited {}: {}",
                output.status,
                String::from_utf8_lossy(&output.stderr)
            ),
        });
    }

    let logical_name = std::path::Path::new(requested)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(requested);
    let mut digest = v1_rt::atom_identity_hash("emit-host-resolved-build-tool-v1".to_string());
    for field in [
        v1_rt::atom_identity_hash(logical_name.to_string()),
        v1_rt::bytes_identity_hash(&executable),
        v1_rt::bytes_identity_hash(&output.stdout),
        v1_rt::bytes_identity_hash(&output.stderr),
    ] {
        digest = v1_rt::hash_combine(digest, field);
    }
    Ok(ObservedToolIdentity {
        tool_name: logical_name.to_string(),
        observed_identity: digest,
    })
}

fn fold_observed_toolchain_identity(rows: &[ObservedToolIdentity]) -> String {
    debug_assert!(rows.iter().all(|row| !row.tool_name.is_empty()));
    rows.iter().fold(
        v1_rt::atom_identity_hash("emit-host-resolved-build-toolchain-v1".to_string()),
        |acc, observed_tool_identity| {
            v1_rt::hash_combine(acc, observed_tool_identity.observed_identity.clone())
        },
    )
}

fn emit_host_resolved_build_context_identity(
    build_argvs: &[Vec<String>],
    probe_workspace: &std::path::Path,
    environment: &EmitHostBuildEnvironment,
) -> InterpResult<String> {
    let cargo_configuration_identity =
        emit_host_cargo_configuration_digest(environment, probe_workspace)?;
    let mut observed_tool_identities = Vec::new();
    for argv in build_argvs {
        let requested = argv.first().ok_or_else(|| InterpError::TypeError {
            msg: "emit_host_run_transport_cached: empty build argv".to_string(),
        })?;
        let requested_name = std::path::Path::new(requested)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(requested);
        let version_args: &[&str] = if requested_name == "cargo" {
            &["-Vv"]
        } else {
            &["--version"]
        };
        observed_tool_identities.push(observe_tool_identity(
            requested,
            version_args,
            probe_workspace,
            environment,
        )?);

        if requested_name == "cargo" {
            let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
            observed_tool_identities.push(observe_tool_identity(
                &rustc,
                &["-vV"],
                probe_workspace,
                environment,
            )?);
        }
    }
    let toolchain_identity = fold_observed_toolchain_identity(&observed_tool_identities);
    Ok(emit_host_artifact_realization_digest(&[
        ("resolved-build-toolchain", toolchain_identity),
        ("constructed-build-environment", environment.digest.clone()),
        ("cargo-configuration", cargo_configuration_identity),
    ]))
}

fn emit_host_materialize_workspace_files(
    workspace: &std::path::Path,
    files: &[(String, String)],
) -> InterpResult<()> {
    use std::path::Component;

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
    Ok(())
}

fn emit_host_run_transport_cached_in_workspace(
    workspace: &std::path::Path,
    files: &[(String, String)],
    build_argvs: &[Vec<String>],
    run_argv: &[String],
    build_environment: &EmitHostBuildEnvironment,
    ctx: &InterpContext,
    require_transition_timing: bool,
) -> InterpResult<Value> {
    let ready_marker = workspace.join(".native_ready");
    let cold_compile_receipt = workspace.join(".native_cold_compile_nanos");
    let artifact_lookup_started = std::time::Instant::now();
    // Cold control (falsifier cadence): widen-only — ignoring the ready marker can
    // only force a FULL cold rebuild, never skip work (the compile-clean cold-control
    // pattern). Not an escape hatch: no value of the env makes the run do less.
    let cold_control = std::env::var("GUNBC_CI_NATIVE_CACHE_COLD_CONTROL")
        .map(|v| v == "1")
        .unwrap_or(false);
    let recorded_cold_compile_nanos = std::fs::read_to_string(&cold_compile_receipt)
        .ok()
        .and_then(|s| s.trim().parse::<u128>().ok())
        .filter(|n| *n > 0);
    // The timing receipt is part of readiness for the production transition: an old marker
    // without its measured cold wall is a warm miss and widens to a rebuild, never a zero.
    let compile_skipped = !cold_control
        && ready_marker.exists()
        && (!require_transition_timing || recorded_cold_compile_nanos.is_some());
    let artifact_lookup_nanos = artifact_lookup_started.elapsed().as_nanos();
    eprintln!(
        "[native-cache] key={} compile_skipped={} cold_control={}",
        workspace
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "?".to_string()),
        compile_skipped,
        cold_control
    );

    let transport_result = |phase: &str,
                            termination: Value,
                            stdout: &[u8],
                            stderr: &[u8],
                            build_log: Vec<Value>,
                            compile_skipped: bool,
                            cold_compile_nanos: u128,
                            native_execution_nanos: u128|
     -> Value {
        Value::Record {
            type_name: ctx.sym("EmitHostTransportResult"),
            // fields_get is a binary search by Symbol id, so a multi-field record
            // MUST be sorted at construction; declaration order is interning-order-
            // dependent and broke .success lookups when #6904 shifted interning.
            fields: Rc::new(sorted_fields(vec![
                (ctx.sym("phase"), str_value(phase.to_string())),
                (ctx.sym("termination"), termination),
                (ctx.sym("compile_skipped"), Value::Bool(compile_skipped)),
                (
                    ctx.sym("artifact_lookup_nanos"),
                    Value::Int(artifact_lookup_nanos.min(i64::MAX as u128) as i64),
                ),
                (
                    ctx.sym("cold_compile_nanos"),
                    Value::Int(cold_compile_nanos.min(i64::MAX as u128) as i64),
                ),
                (
                    ctx.sym("native_execution_nanos"),
                    Value::Int(native_execution_nanos.min(i64::MAX as u128) as i64),
                ),
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
            ])),
        }
    };

    let target_dir = workspace.join("target");
    let run_command = |argv: &[String]| -> InterpResult<std::process::Output> {
        let mut command = std::process::Command::new(resolve_host_tool_program(&argv[0])?);
        command.args(&argv[1..]).current_dir(workspace);
        emit_host_apply_build_environment(&mut command, build_environment);
        command.env("CARGO_TARGET_DIR", &target_dir);
        command.output().map_err(|e| InterpError::TypeError {
            msg: format!(
                "emit_host_run_transport_cached: spawn {:?} failed: {e}",
                argv[0]
            ),
        })
    };

    if !compile_skipped {
        emit_host_materialize_workspace_files(workspace, files)?;

        let mut build_log: Vec<Value> = Vec::new();
        let compile_started = std::time::Instant::now();
        for argv in build_argvs {
            let out = run_command(argv)?;
            build_log.push(str_value(format!(
                "{} -> {}",
                argv.join(" "),
                process_termination_label(&out.status)
            )));
            if !out.status.success() {
                build_log.push(str_value(String::from_utf8_lossy(&out.stderr).to_string()));
                return Ok(transport_result(
                    "build",
                    process_termination_value(&out.status, ctx),
                    &out.stdout,
                    &out.stderr,
                    build_log,
                    false,
                    compile_started.elapsed().as_nanos(),
                    0,
                ));
            }
        }

        let cold_compile_nanos = compile_started.elapsed().as_nanos();
        let native_started = std::time::Instant::now();
        let out = run_command(run_argv)?;
        let native_execution_nanos = native_started.elapsed().as_nanos();
        build_log.push(str_value(format!(
            "{} -> {}",
            run_argv.join(" "),
            process_termination_label(&out.status)
        )));
        if out.status.success() {
            std::fs::write(&ready_marker, b"1").map_err(|e| InterpError::TypeError {
                msg: format!("emit_host_run_transport_cached: ready marker write failed: {e}"),
            })?;
            std::fs::write(&cold_compile_receipt, cold_compile_nanos.to_string()).map_err(|e| {
                InterpError::TypeError {
                    msg: format!(
                        "emit_host_run_transport_cached: cold compile receipt write failed: {e}"
                    ),
                }
            })?;
        }
        return Ok(transport_result(
            "run",
            process_termination_value(&out.status, ctx),
            &out.stdout,
            &out.stderr,
            build_log,
            false,
            cold_compile_nanos,
            native_execution_nanos,
        ));
    }

    let native_started = std::time::Instant::now();
    let out = run_command(run_argv)?;
    let native_execution_nanos = native_started.elapsed().as_nanos();
    let mut build_log: Vec<Value> = Vec::new();
    build_log.push(str_value(format!(
        "{} -> {}",
        run_argv.join(" "),
        process_termination_label(&out.status)
    )));
    Ok(transport_result(
        "run_cached",
        process_termination_value(&out.status, ctx),
        &out.stdout,
        &out.stderr,
        build_log,
        true,
        recorded_cold_compile_nanos.unwrap_or(0),
        native_execution_nanos,
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
        let program = resolve_host_tool_program(&argv[0])?;
        std::process::Command::new(&program)
            .args(&argv[1..])
            .current_dir(workspace)
            .env("CARGO_TARGET_DIR", &target_dir)
            .env_remove("RUSTC_WRAPPER")
            .env_remove("RUSTC_WORKSPACE_WRAPPER")
            .output()
            .map_err(|e| InterpError::TypeError {
                msg: format!("emit_host_run_transport: spawn {:?} failed: {e}", argv[0]),
            })
    };

    let transport_result = |phase: &str,
                            termination: Value,
                            stdout: &[u8],
                            stderr: &[u8],
                            build_log: Vec<Value>,
                            compile_skipped: bool|
     -> Value {
        Value::Record {
            type_name: ctx.sym("EmitHostTransportResult"),
            // fields_get is a binary search by Symbol id, so a multi-field record
            // MUST be sorted at construction; declaration order is interning-order-
            // dependent and broke .success lookups when #6904 shifted interning.
            fields: Rc::new(sorted_fields(vec![
                (ctx.sym("phase"), str_value(phase.to_string())),
                (ctx.sym("termination"), termination),
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
            ])),
        }
    };

    let mut build_log: Vec<Value> = Vec::new();
    for argv in build_argvs {
        let out = run_command(argv)?;
        build_log.push(str_value(format!(
            "{} -> {}",
            argv.join(" "),
            process_termination_label(&out.status)
        )));
        if !out.status.success() {
            build_log.push(str_value(String::from_utf8_lossy(&out.stderr).to_string()));
            return Ok(transport_result(
                "build",
                process_termination_value(&out.status, ctx),
                &out.stdout,
                &out.stderr,
                build_log,
                false,
            ));
        }
    }

    let out = run_command(run_argv)?;
    build_log.push(str_value(format!(
        "{} -> {}",
        run_argv.join(" "),
        process_termination_label(&out.status)
    )));
    Ok(transport_result(
        "run",
        process_termination_value(&out.status, ctx),
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

/// Handler bodies for free-call builtin dispatch. Roster authority is
/// `v1_interpreter_authored_roster_arms()` in `.dag`; generated
/// `lookup_eval_builtin_inner` routes spellings before this macro matches on
/// the generated enum variant for each arm identity.
///
/// Call-site locals are passed in as identifiers (`$name`, `$positional`,
/// `$ctx`) because macro_rules hygiene would otherwise not resolve them: arm
/// bodies live in this definition, the values live at the expansion site.
///
/// `name` is additionally re-bound here rather than only threaded as `$name`:
/// two arm bodies use it as an inline format capture (`"{name} requires ..."`),
/// which no token substitution can reach. Binding it inside THIS definition
/// gives it the same hygiene context as the arms, so the capture resolves.
macro_rules! v1_builtin_arms {
    ($cb:ident, $name:ident, $positional:ident, $ctx:ident) => {{
        #[allow(unused_variables)]
        let name = $name;
        $cb! {
            $name, $positional, $ctx;

            arm "free_call.parse_stage0_cargo_manifest_bins" { "parse_stage0_cargo_manifest_bins" } => {
                let manifest = expect_str(
                    $positional.first().copied(),
                    "parse_stage0_cargo_manifest_bins manifest",
                )?;
                let parsed = crate::cli_run::parse_stage0_cargo_manifest_bin_paths(&manifest);
                let variant = match parsed {
                    crate::cli_run::Stage0CargoManifestBinParse::Parsed {
                        authored_relative_paths,
                    } => Value::Variant {
                        type_name: $ctx.sym("CargoManifestBinParse"),
                        variant_name: $ctx.sym("CargoManifestBinsParsed"),
                        fields: Rc::new(sorted_fields(vec![(
                            $ctx.sym("authored_relative_paths"),
                            list_value(
                                authored_relative_paths
                                    .into_iter()
                                    .map(str_value)
                                    .collect::<Vec<_>>(),
                            ),
                        )])),
                    },
                    crate::cli_run::Stage0CargoManifestBinParse::Refused { detail } => {
                        Value::Variant {
                            type_name: $ctx.sym("CargoManifestBinParse"),
                            variant_name: $ctx.sym("CargoManifestBinsParseRefused"),
                            fields: Rc::new(sorted_fields(vec![(
                                $ctx.sym("detail"),
                                str_value(detail),
                            )])),
                        }
                    }
                };
                Ok(Some(variant))
            },

            arm "free_call.parse_roadmap_acceptance_event_history_jsonl" { "parse_roadmap_acceptance_event_history_jsonl" } => {
                let text = expect_str(
                    $positional.first().copied(),
                    "parse_roadmap_acceptance_event_history_jsonl text",
                )?;
                crate::cli_run::roadmap_acceptance_history_carrier::parse_roadmap_acceptance_event_history_jsonl_builtin(
                    &text,
                    $ctx,
                )
                .map(Some)
            },

            arm "free_call.project_roadmap_acceptance_event_history_from_authority_text_host" { "project_roadmap_acceptance_event_history_from_authority_text_host" } => {
                let authority_text = expect_str(
                    $positional.first().copied(),
                    "project_roadmap_acceptance_event_history_from_authority_text_host authority_text",
                )?;
                crate::cli_run::project_roadmap_acceptance_event_history_from_authority_text_builtin(
                    &authority_text,
                    $ctx,
                )
                .map(Some)
            },

            // DECLARED SCAFFOLD supplying gunbc.stage0_emit_plan with SOURCE identities only.
            // It parses cli_run::regen_input_sources through the module-binding authority path;
            // it never observes EmitResult. Dissolve-on: generated_artifact_gate accepts a
            // v2.compiler.source_authority.ModuleStorageIndex.
            arm "free_call.stage0_emission_source_identities_host" { "stage0_emission_source_identities_host" } => {
                if !$positional.is_empty() {
                    return Err(InterpError::TypeError {
                        msg: "stage0_emission_source_identities_host takes no arguments".to_string(),
                    });
                }
                let workspace = crate::cli_run::workspace_root();
                let identities = crate::cli_run::stage0_emission_source_identities(&workspace)
                    .map_err(|msg| InterpError::TypeError { msg })?;
                let items = identities
                    .into_iter()
                    .map(|identity| Value::Record {
                        type_name: $ctx.sym("Stage0SourceModuleIdentity"),
                        fields: Rc::new(sorted_fields(vec![
                            ($ctx.sym("module_path"), str_value(identity.module_path)),
                            (
                                $ctx.sym("provenance"),
                                Value::Variant {
                                    type_name: $ctx.sym("Stage0SourceIdentityProvenance"),
                                    variant_name: $ctx.sym("ParsedFromRegenSourceClosure"),
                                    fields: Rc::new(Vec::new()),
                                },
                            ),
                            (
                                $ctx.sym("source_tree"),
                                Value::Variant {
                                    type_name: $ctx.sym("Stage0SourceTree"),
                                    variant_name: $ctx.sym(identity.source_tree),
                                    fields: Rc::new(Vec::new()),
                                },
                            ),
                            ($ctx.sym("storage_path"), str_value(identity.storage_path)),
                        ])),
                    })
                    .collect::<Vec<_>>();
                Ok(Some(Value::Variant {
                    type_name: $ctx.sym("Stage0SourceIdentitySupply"),
                    variant_name: $ctx.sym("Stage0SourceIdentitySupplyAvailable"),
                    fields: Rc::new(vec![($ctx.sym("identities"), list_value(items))]),
                }))
            },

            arm "free_call.to_string" { "to_string" } => {
                let v = $positional.first().ok_or_else(|| InterpError::TypeError {
                    msg: "to_string requires 1 argument".to_string(),
                })?;
                Ok(Some(str_value(format!("{}", v))))
            },

            arm "free_call.utf8_decode_bytes" { "utf8_decode_bytes" } => {
                let bytes = expect_byte_vec($positional.first().copied(), "utf8_decode_bytes")?;
                let text =
                    v1_rt::utf8_decode_bytes(&bytes).map_err(|msg| InterpError::TypeError { msg })?;
                Ok(Some(str_value(text)))
            },

            arm "free_call.bytes_octets" { "bytes_octets" } => {
                let bytes = expect_byte_vec($positional.first().copied(), "bytes_octets")?;
                let items: Vec<Value> = bytes.iter().map(|b| Value::Int(*b as i64)).collect();
                Ok(Some(list_value(items)))
            },

            arm "free_call.octets_bytes" { "octets_bytes" } => {
                let arg = $positional
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
            },

            arm "free_call.utf8_encode_bytes" { "utf8_encode_bytes" } => {
                let s = expect_str($positional.first().copied(), "utf8_encode_bytes")?;
                let items: Vec<Value> = s.as_bytes().iter().map(|b| Value::Int(*b as i64)).collect();
                Ok(Some(list_value(items)))
            },

            arm "free_call.discriminant" { "discriminant" } => match $positional.first() {
                Some(Value::Variant { variant_name, .. }) => {
                    Ok(Some(str_value(resolve_sym(*variant_name))))
                }
                Some(Value::Record { type_name, .. }) => Ok(Some(str_value(resolve_sym(*type_name)))),
                _ => Ok(None),
            },

            arm "free_call.chars_to_string" { "chars_to_string" } => {
                let cps = match $positional.first().copied() {
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
                let start = expect_int($positional.get(1).copied(), "chars_to_string start")?
                    .max(0)
                    .min(len);
                let end = expect_int($positional.get(2).copied(), "chars_to_string end")?
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
                Ok(Some(str_value(s)))
            },

            arm "free_call.get" { "get" } => match $positional.as_slice() {
                [list_val, idx_val] if free_monoid_to_vec(list_val).is_some() => {
                    let items = expect_list(list_val, "get")?;
                    let idx = expect_int(Some(idx_val), "get")?;
                    Ok(Some(list_get_at_or_null(&items, idx)))
                }
                _ => Ok(None),
            },

            arm "free_call.parse_int" { "parse_int" } => {
                let s = expect_str($positional.first().copied(), "parse_int")?;
                match s.parse::<i64>() {
                    Ok(n) => Ok(Some(Value::Int(n))),
                    Err(_) => Ok(Some(Value::Null)),
                }
            },

            arm "free_call.record_source_chars_index_lookup" { "record_source_chars_index_lookup" } => Ok(Some(Value::Unit)),

            // Scaffold arm — dissolution trigger lives on `v1_rt::trace_mark`'s doc comment
            // (realization_measurement_loop Phase 0, docs/plans/realization-measurement-loop.md):
            // delete this arm with the rest of the trace_mark deletion set named there.
            arm "free_call.trace_mark" { "trace_mark" } => {
                if let [Value::Str(s)] = $positional.as_slice() {
                    v1_rt::trace_mark(s.to_string());
                }
                Ok(Some(Value::Unit))
            },

            arm "free_call.concat" { "concat" } => {
                if $positional.len() >= 2 && $positional.iter().all(|v| matches!(v, Value::Str(_))) {
                    let mut result = String::new();
                    for v in &$positional {
                        if let Value::Str(s) = v {
                            result.push_str(s);
                        }
                    }
                    return Ok(Some(str_value(result)));
                }
                let record_push = |copied: usize| {
                    let mut counters = $ctx.mutation_counters.borrow_mut();
                    counters.list_push_calls += 1;
                    counters.list_push_items_copied += copied as u64;
                };
                match $positional.as_slice() {
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
                                let mut counters = $ctx.mutation_counters.borrow_mut();
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
            },

            arm "free_call.count" { "count" } => match $positional.first() {
                Some(v) => match free_monoid_to_vec(v) {
                    Some(items) => Ok(Some(Value::Int(items.len() as i64))),
                    None => Ok(None),
                },
                None => Ok(None),
            },

            arm "free_call.reverse" { "reverse" } => match $positional.first() {
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

            arm "free_call.string_length" { "string_length" } => {
                let s = expect_str($positional.first().copied(), "string_length")?;
                Ok(Some(Value::Int(s.chars().count() as i64)))
            },

            arm "free_call.substring" { "substring" } => {
                let s = expect_str($positional.first().copied(), "substring")?;
                let start = expect_int($positional.get(1).copied(), "substring start")?;
                let end = expect_int($positional.get(2).copied(), "substring end")?;
                Ok(Some(str_value(v1_rt::substring(&s, start, end))))
            },

            arm "free_call.char_at" { "char_at" } => {
                let s = expect_str($positional.first().copied(), "char_at")?;
                let pos = expect_int($positional.get(1).copied(), "char_at pos")?;
                Ok(Some(str_value(v1_rt::char_at(&s, pos))))
            },

            arm "free_call.string_contains" { "string_contains" } => {
                let s = expect_str($positional.first().copied(), "contains")?;
                let sub = expect_str($positional.get(1).copied(), "contains sub")?;
                Ok(Some(Value::Bool(s.contains(&sub))))
            },

            arm "free_call.starts_with" { "starts_with" } => {
                let s = expect_str($positional.first().copied(), "starts_with")?;
                let prefix = expect_str($positional.get(1).copied(), "starts_with prefix")?;
                Ok(Some(Value::Bool(s.starts_with(&prefix))))
            },

            arm "free_call.trim" { "trim" } => {
                let s = expect_str($positional.first().copied(), "trim")?;
                Ok(Some(str_value(v1_rt::trim(s))))
            },

            arm "free_call.length" { "length" } => match $positional.first() {
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

            arm "free_call.contains" { "contains" } => match $positional.as_slice() {
                [Value::Str(s), Value::Str(sub), ..] => Ok(Some(Value::Bool(s.contains(sub.as_ref())))),
                [xs, target, ..] => match free_monoid_to_vec(xs) {
                    Some(items) => Ok(Some(Value::Bool(items.iter().any(|item| item == *target)))),
                    None => Ok(None),
                },
                _ => Ok(None),
            },

            arm "free_call.replace" { "replace" } => {
                let s = expect_str($positional.first().copied(), "replace")?;
                let from = expect_str($positional.get(1).copied(), "replace from")?;
                let to = expect_str($positional.get(2).copied(), "replace to")?;
                Ok(Some(str_value(s.replace(&from, &to))))
            },

            arm "free_call.code_point" { "code_point" } => {
                let s = expect_str($positional.first().copied(), "code_point")?;
                let cp = s.chars().next().map(|c| c as i64).unwrap_or(0);
                Ok(Some(Value::Int(cp)))
            },

            arm "free_call.from_code_point" { "from_code_point" } => {
                let cp = expect_int($positional.first().copied(), "from_code_point")?;
                let c = char::from_u32(cp as u32).unwrap_or('\0');
                Ok(Some(str_value(c.to_string())))
            },

            arm "free_call.is_xid_start" { "is_xid_start" } => {
                let cp = expect_int($positional.first().copied(), "is_xid_start")?;
                Ok(Some(Value::Bool(v1_rt::is_xid_start(cp))))
            },

            arm "free_call.is_xid_continue" { "is_xid_continue" } => {
                let cp = expect_int($positional.first().copied(), "is_xid_continue")?;
                Ok(Some(Value::Bool(v1_rt::is_xid_continue(cp))))
            },

            arm "free_call.is_emoji_ident" { "is_emoji_ident" } => {
                let cp = expect_int($positional.first().copied(), "is_emoji_ident")?;
                Ok(Some(Value::Bool(v1_rt::is_emoji_ident(cp))))
            },

            arm "free_call.list_push" { "list_push" | "append" } => match $positional.as_slice() {
                [list_val, item] if matches!(list_val, Value::Str(_)) => Ok(None),
                [list_val, item] => match value_to_list_carrier(list_val) {
                    Some((items, copied)) => {
                        let mut counters = $ctx.mutation_counters.borrow_mut();
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

            arm "free_call.list_concat" { "list_concat" } => match $positional.as_slice() {
                [a, b] if matches!(a, Value::Str(_)) || matches!(b, Value::Str(_)) => Ok(None),
                [a, b] => match (value_to_list_carrier(a), value_to_list_carrier(b)) {
                    (Some((a_items, a_copied)), Some((b_items, b_copied))) => {
                        let mut counters = $ctx.mutation_counters.borrow_mut();
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

            arm "free_call.empty_map" { "empty_map" } => Ok(Some(map_value(HamtMap::new()))),

            arm "free_call.empty_set" { "empty_set" } => Ok(Some(Value::Set(Rc::new(OrdSet::new())))),

            arm "free_call.set_insert" { "set_insert" } => match $positional.as_slice() {
                [Value::Set(s), Value::Str(k)] => {
                    let mut counters = $ctx.mutation_counters.borrow_mut();
                    counters.set_insert_calls += 1;
                    counters.set_insert_items_copied += s.len() as u64;
                    drop(counters);
                    let mut result = s.as_ref().clone();
                    result.insert(k.to_string());
                    Ok(Some(Value::Set(Rc::new(result))))
                }
                _ => Ok(None),
            },

            arm "free_call.set_union" { "set_union" } => match $positional.as_slice() {
                [Value::Set(a), Value::Set(b)] => {
                    let mut counters = $ctx.mutation_counters.borrow_mut();
                    counters.set_union_calls += 1;
                    counters.set_union_items_copied += (a.len() + b.len()) as u64;
                    drop(counters);
                    let mut result = a.as_ref().clone();
                    result.extend(b.iter().cloned());
                    Ok(Some(Value::Set(Rc::new(result))))
                }
                _ => Ok(None),
            },

            arm "free_call.set_contains" { "set_contains" } => match $positional.as_slice() {
                [Value::Set(s), Value::Str(k)] => Ok(Some(Value::Bool(s.contains(k.as_ref())))),
                _ => Ok(None),
            },

            arm "free_call.map_insert" { "map_insert" } => match $positional.as_slice() {
                [Value::Map(m), k, v] => match CanonKey::new((*k).clone()) {
                    Some(ck) => {
                        let mut counters = $ctx.mutation_counters.borrow_mut();
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

            arm "free_call.lookup" { "lookup" } => match $positional.as_slice() {
                [map, key] => {
                    let raw = raw_map_lookup(map, key, &Env::empty(), $ctx)?;
                    Ok(Some(map_lookup_as_optional(raw, $ctx)))
                }
                _ => Ok(None),
            },

            arm "free_call.map_keys" { "map_keys" } => match $positional.first() {
                Some(Value::Map(m)) => {
                    let keys: Vec<Value> = m.keys().map(|k| k.key.clone()).collect();
                    Ok(Some(list_value((keys))))
                }
                _ => Ok(None),
            },

            arm "free_call.map_values" { "map_values" } => match $positional.first() {
                Some(Value::Map(m)) => {
                    let vals: Vec<Value> = m.values().cloned().collect();
                    Ok(Some(list_value((vals))))
                }
                _ => Ok(None),
            },

            arm "free_call.map_contains_key" { "map_contains_key" | "map_has" } => match $positional.as_slice() {
                [Value::Map(m), k] => match CanonKey::new((*k).clone()) {
                    Some(ck) => Ok(Some(Value::Bool(m.contains_key(&ck)))),
                    None => Ok(Some(Value::Bool(false))),
                },
                _ => Ok(None),
            },

            arm "free_call.map_is_empty" { "map_is_empty" } => match $positional.as_slice() {
                [Value::Map(m)] => Ok(Some(Value::Bool(m.is_empty()))),
                _ => Ok(None),
            },

            arm "free_call.rc_ptr_eq" { "rc_ptr_eq" | "rc_vec_ptr_eq" } => match $positional.as_slice() {
                [a, b] => Ok(Some(Value::Bool(a == b))),
                _ => Ok(None),
            },

            arm "free_call.map_merge" { "map_merge" } => match $positional.as_slice() {
                [Value::Map(base), Value::Map(overlay)] => {
                    let mut counters = $ctx.mutation_counters.borrow_mut();
                    counters.map_merge_calls += 1;
                    drop(counters);
                    Ok(Some(map_value((**overlay).clone().union((**base).clone()))))
                }
                _ => Ok(None),
            },

            arm "free_call.str_eq" { "str_eq" } => match $positional.as_slice() {
                [Value::Str(a), Value::Str(b)] => Ok(Some(Value::Bool(a == b))),
                _ => Ok(None),
            },

            arm "free_call.atom_identity_hash" { "atom_identity_hash" } => match $positional.as_slice() {
                [Value::Str(s)] => Ok(Some(str_value(v1_rt::atom_identity_hash(s.to_string())))),
                _ => Err(InterpError::TypeError {
                    msg: "atom_identity_hash requires exactly one string argument".to_string(),
                }),
            },

            // ObservePeakResidentAtSubject realization seam (witness-realization plan P1):
            // process peak resident set (VmHWM) in bytes. Fail-closed when the host
            // cannot report it — a fabricated 0 would be a Measured lie (DESIGN §5).
            arm "free_call.observed_peak_resident_bytes" { "observed_peak_resident_bytes" } => match $positional.as_slice() {
                [] => {
                    // Routed through the single portable reader rather than re-inlining a
                    // procfs parse here. This arm previously carried its OWN copy of the
                    // /proc/self/status VmHWM read — a second implementation of one
                    // observation (section 3), and the copy that actually executes for
                    // witnesses, so fixing only cli_run's would have left this one Linux-only.
                    // Authority for the interface and its per-implementation units:
                    // dag/extdeps/posix/rusage.dag with dag/extdeps/{linux,darwin}/rusage.dag.
                    let bytes = crate::cli_run::peak_rss_vhwm_bytes().and_then(|b| i64::try_from(b).ok());
                    match bytes {
                        Some(b) => Ok(Some(Value::Int(b))),
                        None => Err(InterpError::TypeError {
                            msg: "observed_peak_resident_bytes: getrusage(RUSAGE_SELF).ru_maxrss unavailable on this host (refusing to fabricate a Measured space fact)"
                                .to_string(),
                        }),
                    }
                }
                _ => Err(InterpError::TypeError {
                    msg: "observed_peak_resident_bytes takes no arguments".to_string(),
                }),
            },

            // ObserveElapsedAtSubject realization seam: a process-relative monotonic
            // reading. Only differences between two observations are meaningful.
            arm "free_call.observed_monotonic_nanos" { "observed_monotonic_nanos" } => match $positional.as_slice() {
                [Value::Str(_boundary)] => {
                    static EPOCH: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
                    let nanos = EPOCH
                        .get_or_init(std::time::Instant::now)
                        .elapsed()
                        .as_nanos()
                        .min(i64::MAX as u128) as i64;
                    Ok(Some(Value::Int(nanos)))
                }
                _ => Err(InterpError::TypeError {
                    msg: "observed_monotonic_nanos takes exactly one boundary label".to_string(),
                }),
            },

            arm "free_call.hash_combine" { "hash_combine" } => match $positional.as_slice() {
                [Value::Str(a), Value::Str(b)] if $positional.len() == 2 => {
                    if !v1_rt::is_hash_digest(a) || !v1_rt::is_hash_digest(b) {
                        return Err(InterpError::TypeError {
                            msg: "hash_combine requires exactly two Hash arguments".to_string(),
                        });
                    }
                    Ok(Some(str_value(v1_rt::hash_combine(a.to_string(), b.to_string()))))
                }
                _ => Err(InterpError::TypeError {
                    msg: "hash_combine requires exactly two Hash arguments".to_string(),
                }),
            },

            arm "free_call.filesystem_read" { "filesystem_read" } => {
                let path = expect_str($positional.first().copied(), "filesystem_read")?;
                Ok(Some(eval_filesystem_read_builtin(path, $ctx)?))
            },

            arm "free_call.emit_host_run_transport" { "emit_host_run_transport" } => Ok(Some(eval_emit_host_run_transport_builtin(
                $positional.first().copied(),
                $positional.get(1).copied(),
                $positional.get(2).copied(),
                $positional.get(3).copied(),
                $ctx,
            )?)),

            arm "free_call.emit_host_run_transport_cached" { "emit_host_run_transport_cached" } => Ok(Some(eval_emit_host_run_transport_cached_builtin(
                $positional.first().copied(),
                $positional.get(1).copied(),
                $positional.get(2).copied(),
                $positional.get(3).copied(),
                $positional.get(4).copied(),
                $ctx,
            )?)),

            arm "free_call.emit_host_native_cache_evict" { "emit_host_native_cache_evict" } => Ok(Some(eval_emit_host_native_cache_evict_builtin(
                $positional.first().copied(),
                $ctx,
            )?)),

            arm "free_call.contiguous_loop_elementwise_kernel" { "contiguous_loop_elementwise_kernel" } => {
                let op_codes = expect_int_list_flex($positional.first().copied(), $name)?;
                let a = expect_int_list_flex($positional.get(1).copied(), $name)?;
                let b = expect_int_list_flex($positional.get(2).copied(), $name)?;
                let c = expect_int_list_flex($positional.get(3).copied(), $name)?;
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
            },

            arm "free_call.contiguous_loop_elementwise_float_kernel" { "contiguous_loop_elementwise_float_kernel" } => {
                let op_codes = expect_int_list_flex($positional.first().copied(), $name)?;
                let fma_policy = expect_fma_contraction_policy_wire($positional.get(1).copied(), $name)?;
                let a = expect_float_list_flex($positional.get(2).copied(), $name)?;
                let b = expect_float_list_flex($positional.get(3).copied(), $name)?;
                let c = expect_float_list_flex($positional.get(4).copied(), $name)?;
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
            },

            arm "free_call.layer_import_facts" { "layer_import_facts" } => {
                let std_roots = expect_str_list($positional.first().copied(), "layer_import_facts")?;
                let extdeps_roots = expect_str_list($positional.get(1).copied(), "layer_import_facts")?;
                let facts = crate::cli_run::layer_import_facts(&std_roots, &extdeps_roots);
                let mut items: Vec<Value> = Vec::new();
                for f in facts {
                    let layer = Value::Variant {
                        type_name: $ctx.sym("LayerPrefix"),
                        variant_name: $ctx.sym(f.layer),
                        fields: Rc::new(vec![]),
                    };
                    items.push(Value::Record {
                        type_name: $ctx.sym("LayerImportFact"),
                        fields: Rc::new(sorted_fields(vec![
                            ($ctx.sym("import_module"), str_value(f.import_module)),
                            ($ctx.sym("layer"), layer),
                            ($ctx.sym("path"), str_value(f.path)),
                        ])),
                    });
                }
                Ok(Some(list_value(items)))
            },

            arm "free_call.import_resolution_facts" { "import_resolution_facts" } => {
                let pool_roots =
                    expect_str_list($positional.first().copied(), "import_resolution_facts")?;
                let importer_roots =
                    expect_str_list($positional.get(1).copied(), "import_resolution_facts")?;
                let exclude_substrings =
                    expect_str_list($positional.get(2).copied(), "import_resolution_facts")?;
                let facts = crate::cli_run::import_resolution_facts(
                    &pool_roots,
                    &importer_roots,
                    &exclude_substrings,
                );
                let mut items: Vec<Value> = Vec::new();
                for f in facts {
                    items.push(Value::Record {
                        type_name: $ctx.sym("ImportResolutionFact"),
                        fields: Rc::new(sorted_fields(vec![
                            ($ctx.sym("import_module"), str_value(f.import_module)),
                            ($ctx.sym("path"), str_value(f.path)),
                            ($ctx.sym("target_declared"), Value::Bool(f.target_declared)),
                        ])),
                    });
                }
                Ok(Some(list_value(items)))
            },

            arm "free_call.reference_resolution_facts" { "reference_resolution_facts" } => {
                let pool_roots =
                    expect_str_list($positional.first().copied(), "reference_resolution_facts")?;
                let importer_roots =
                    expect_str_list($positional.get(1).copied(), "reference_resolution_facts")?;
                let exclude_substrings =
                    expect_str_list($positional.get(2).copied(), "reference_resolution_facts")?;
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
                        type_name: $ctx.sym("ImportResolutionFact"),
                        fields: Rc::new(sorted_fields(vec![
                            ($ctx.sym("import_module"), str_value(f.import_module)),
                            ($ctx.sym("path"), str_value(f.path)),
                            ($ctx.sym("target_declared"), Value::Bool(f.target_declared)),
                        ])),
                    });
                }
                Ok(Some(list_value(items)))
            },

            arm "free_call.dependency_resolution_facts" { "dependency_resolution_facts" } => {
                let pool_roots =
                    expect_str_list($positional.first().copied(), "dependency_resolution_facts")?;
                let importer_roots =
                    expect_str_list($positional.get(1).copied(), "dependency_resolution_facts")?;
                let exclude_substrings =
                    expect_str_list($positional.get(2).copied(), "dependency_resolution_facts")?;
                // Reference-first exact union through the ONE dedup authority, then the
                // import_module -> target_module rename. Both halves are the host twin of what
                // `v2.lens.module_graph` used to compose in the interpreter; the composition moved
                // because it measured 104,943ms against 151ms for the two leaves it combines.
                let facts = crate::cli_run::dependency_resolution_facts(
                    &pool_roots,
                    &importer_roots,
                    &exclude_substrings,
                );
                let mut items: Vec<Value> = Vec::new();
                for f in facts {
                    items.push(Value::Record {
                        type_name: $ctx.sym("ModuleDependencyEdge"),
                        fields: Rc::new(sorted_fields(vec![
                            ($ctx.sym("path"), str_value(f.path)),
                            ($ctx.sym("target_declared"), Value::Bool(f.target_declared)),
                            ($ctx.sym("target_module"), str_value(f.import_module)),
                        ])),
                    });
                }
                Ok(Some(list_value(items)))
            },

            arm "free_call.concept_decl_facts" { "concept_decl_facts" } => {
                let pool_roots = expect_str_list($positional.first().copied(), "concept_decl_facts")?;
                Ok(Some(crate::coproduct_reflection::eval_concept_decl_facts(
                    $ctx,
                    &pool_roots,
                )?))
            },

            arm "free_call.data_decl_type_facts" { "data_decl_type_facts" } => {
                let pool_roots =
                    expect_str_list($positional.first().copied(), "data_decl_type_facts")?;
                Ok(Some(crate::coproduct_reflection::eval_data_decl_type_facts(
                    $ctx,
                    &pool_roots,
                )?))
            },

            arm "free_call.export_signature_facts" { "export_signature_facts" } => {
                let pool_roots =
                    expect_str_list($positional.first().copied(), "export_signature_facts")?;
                Ok(Some(
                    crate::coproduct_reflection::eval_export_signature_facts($ctx, &pool_roots)?,
                ))
            },

            arm "free_call.decl_facts" { "decl_facts" } => {
                let pool_roots = expect_str_list($positional.first().copied(), "decl_facts")?;
                Ok(Some(crate::coproduct_reflection::eval_decl_facts(
                    $ctx,
                    &pool_roots,
                )?))
            },

            arm "free_call.module_declaration_facts" { "module_declaration_facts" } => {
                let pool_roots =
                    expect_str_list($positional.first().copied(), "module_declaration_facts")?;
                let facts = crate::cli_run::module_declaration_facts(&pool_roots);
                let mut items: Vec<Value> = Vec::new();
                for f in facts {
                    items.push(Value::Record {
                        type_name: $ctx.sym("ModuleDeclarationFact"),
                        fields: Rc::new(sorted_fields(vec![
                            ($ctx.sym("module"), str_value(f.module)),
                            ($ctx.sym("path"), str_value(f.path)),
                        ])),
                    });
                }
                Ok(Some(list_value(items)))
            },

            arm "free_call.fact_cardinality_decl_facts" { "fact_cardinality_decl_facts" } => {
                let facts = crate::cli_run::fact_cardinality_decl_facts();
                let mut items: Vec<Value> = Vec::new();
                for f in facts {
                    let tree = match f.tree.as_str() {
                        "dag" => "Dag",
                        "v2" => "V2",
                        other => panic!("fact_cardinality_decl_facts: unknown tree {other:?}"),
                    };
                    let tree_value = Value::Variant {
                        type_name: $ctx.sym("FactCardinalityTree"),
                        variant_name: $ctx.sym(tree),
                        fields: Rc::new(vec![]),
                    };
                    items.push(Value::Record {
                        type_name: $ctx.sym("FactCardinalityDeclFact"),
                        fields: Rc::new(sorted_fields(vec![
                            (
                                $ctx.sym("rel_path_decl_key"),
                                str_value(f.rel_path_decl_key),
                            ),
                            ($ctx.sym("tree"), tree_value),
                            ($ctx.sym("content_hash"), str_value(f.content_hash)),
                        ])),
                    });
                }
                Ok(Some(list_value(items)))
            },

            arm "free_call.languages_consumer_census_data_decl_count" { "languages_consumer_census_data_decl_count" } => Ok(Some(Value::Int(
                crate::cli_run::languages_consumer_census_data_decl_count(),
            ))),

            arm "free_call.languages_consumer_census_per_language_row_count" { "languages_consumer_census_per_language_row_count" } => Ok(Some(Value::Int(
                crate::cli_run::languages_consumer_census_per_language_row_count(),
            ))),

            arm "free_call.languages_consumer_census_format_row_count" { "languages_consumer_census_format_row_count" } => Ok(Some(Value::Int(
                crate::cli_run::languages_consumer_census_format_row_count(),
            ))),

            arm "free_call.languages_consumer_census_external_consumer_count" { "languages_consumer_census_external_consumer_count" } => {
                let decl_name = expect_str(
                    $positional.first().copied(),
                    "languages_consumer_census_external_consumer_count",
                )?;
                Ok(Some(Value::Int(
                    crate::cli_run::languages_consumer_census_external_consumer_count(decl_name),
                )))
            },

            arm "free_call.languages_consumer_census_is_composition_only" { "languages_consumer_census_is_composition_only" } => {
                let decl_name = expect_str(
                    $positional.first().copied(),
                    "languages_consumer_census_is_composition_only",
                )?;
                Ok(Some(Value::Bool(
                    crate::cli_run::languages_consumer_census_is_composition_only(decl_name),
                )))
            },

            arm "free_call.languages_consumer_census_has_external_consumer" { "languages_consumer_census_has_external_consumer" } => {
                let decl_name = expect_str(
                    $positional.first().copied(),
                    "languages_consumer_census_has_external_consumer",
                )?;
                Ok(Some(Value::Bool(
                    crate::cli_run::languages_consumer_census_has_external_consumer(decl_name),
                )))
            },

            arm "free_call.shell_materialize_operation_argv" { "shell_materialize_operation_argv" } => {
                let path = expect_str(
                    $positional.first().copied(),
                    "shell_materialize_operation_argv",
                )?;
                let service = expect_str(
                    $positional.get(1).copied(),
                    "shell_materialize_operation_argv",
                )?;
                let operation = expect_str(
                    $positional.get(2).copied(),
                    "shell_materialize_operation_argv",
                )?;
                let bindings = $positional
                    .get(3)
                    .copied()
                    .cloned()
                    .unwrap_or_else(|| list_value(Vec::<Value>::new()));
                let result = materialize_operation_argv(&path, &service, &operation, &bindings, $ctx);
                Ok(Some(argv_materialization_value(
                    result, &path, &service, &operation, $ctx,
                )))
            },

            arm "free_call.shell_transport_operation_rows" { "shell_transport_operation_rows" } => {
                let mut items: Vec<Value> = Vec::new();
                for row in crate::cli_run::shell_transport_operation_rows() {
                    items.push(Value::Record {
                        type_name: $ctx.sym("ShellTransportOperationRow"),
                        fields: Rc::new(sorted_fields(vec![
                            (
                                $ctx.sym("at"),
                                operation_ref_value(&row.path, &row.service, &row.operation, $ctx),
                            ),
                            (
                                $ctx.sym("declared_inputs"),
                                list_value(
                                    row.declared_inputs
                                        .into_iter()
                                        .map(str_value)
                                        .collect::<Vec<_>>(),
                                ),
                            ),
                            (
                                $ctx.sym("argv_input_refs"),
                                list_value(
                                    row.argv_input_refs
                                        .into_iter()
                                        .map(str_value)
                                        .collect::<Vec<_>>(),
                                ),
                            ),
                        ])),
                    });
                }
                Ok(Some(list_value(items)))
            },

            arm "free_call.extdeps_qualified_name_resolves_in_derived_module_set" { "extdeps_qualified_name_resolves_in_derived_module_set" } => {
                let module = $positional.first().ok_or_else(|| InterpError::TypeError {
                    msg:
                        "extdeps_qualified_name_resolves_in_derived_module_set requires a QualifiedName"
                            .to_string(),
                })?;
                Ok(Some(Value::Bool(
                    crate::cli_run::qualified_name_resolves_in_derived_module_set(module),
                )))
            },

            arm "free_call.transport_script_position_facts_for_path" { "transport_script_position_facts_for_path" } => {
                let path = expect_str(
                    $positional.first().copied(),
                    "transport_script_position_facts_for_path",
                )?;
                let facts = crate::cli_run::transport_script_position_facts_for_path(path);
                let mut items: Vec<Value> = Vec::new();
                for f in facts {
                    let shape = Value::Variant {
                        type_name: $ctx.sym("TransportScriptArgShape"),
                        variant_name: $ctx.sym(f.shape),
                        fields: Rc::new(vec![]),
                    };
                    items.push(Value::Record {
                        type_name: $ctx.sym("TransportScriptPositionFact"),
                        fields: Rc::new(sorted_fields(vec![
                            ($ctx.sym("function"), str_value(f.function)),
                            ($ctx.sym("path"), str_value(f.path)),
                            ($ctx.sym("shape"), shape),
                        ])),
                    });
                }
                Ok(Some(list_value(items)))
            },

            arm "free_call.extdeps_shape_transport_policy_facts_for_qualified_name" { "extdeps_shape_transport_policy_facts_for_qualified_name" } => {
                let qn = $positional.first().ok_or_else(|| InterpError::TypeError {
                    msg: "extdeps_shape_transport_policy_facts_for_qualified_name requires a QualifiedName"
                        .to_string(),
                })?;
                let module_path = crate::cli_run::free_monoid_symbol_value_to_dotted_string(qn);
                let facts = crate::cli_run::extdeps_shape_transport_policy_module_facts(&module_path);
                let argv_items: Vec<Value> = facts
                    .argv_facts
                    .iter()
                    .map(|f| Value::Record {
                        type_name: $ctx.sym("ExtdepsTransportArgvFact"),
                        fields: Rc::new(sorted_fields(vec![
                            ($ctx.sym("argv_index"), Value::Int(f.argv_index)),
                            ($ctx.sym("argv_token"), str_value(f.argv_token.clone())),
                            ($ctx.sym("module"), (*qn).clone()),
                            ($ctx.sym("operation"), str_value(f.operation.clone())),
                            ($ctx.sym("service"), str_value(f.service.clone())),
                            (
                                $ctx.sym("transport_kind"),
                                Value::Variant {
                                    type_name: $ctx.sym("ExtdepsTransportKind"),
                                    variant_name: $ctx.sym(f.transport_kind),
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
                        type_name: $ctx.sym("ExtdepsTransportFusionFact"),
                        fields: Rc::new(sorted_fields(vec![
                            ($ctx.sym("endpoint_key"), str_value(f.endpoint_key.clone())),
                            ($ctx.sym("module"), (*qn).clone()),
                            ($ctx.sym("service_a"), str_value(f.service_a.clone())),
                            ($ctx.sym("service_b"), str_value(f.service_b.clone())),
                        ])),
                    })
                    .collect();
                let input_items: Vec<Value> = facts
                    .input_facts
                    .iter()
                    .map(|f| Value::Record {
                        type_name: $ctx.sym("ExtdepsOperationInputFact"),
                        fields: Rc::new(sorted_fields(vec![
                            ($ctx.sym("module"), (*qn).clone()),
                            ($ctx.sym("operation"), str_value(f.operation.clone())),
                            ($ctx.sym("param_name"), str_value(f.param_name.clone())),
                            ($ctx.sym("service"), str_value(f.service.clone())),
                        ])),
                    })
                    .collect();
                let embedded_items: Vec<Value> = facts
                    .embedded_facts
                    .iter()
                    .map(|f| Value::Record {
                        type_name: $ctx.sym("ExtdepsEmbeddedPolicyLiteralFact"),
                        fields: Rc::new(sorted_fields(vec![
                            ($ctx.sym("data_name"), str_value(f.data_name.clone())),
                            ($ctx.sym("field_name"), str_value(f.field_name.clone())),
                            (
                                $ctx.sym("literal_value"), str_value(f.literal_value.clone()),
                            ),
                            ($ctx.sym("module"), (*qn).clone()),
                        ])),
                    })
                    .collect();
                let result = Value::Record {
                    type_name: $ctx.sym("ExtdepsModuleFacts"),
                    fields: Rc::new(sorted_fields(vec![
                        ($ctx.sym("argv_facts"), list_value(argv_items)),
                        ($ctx.sym("embedded_facts"), list_value(embedded_items)),
                        ($ctx.sym("fusion_facts"), list_value(fusion_items)),
                        (
                            $ctx.sym("gist_create_declares_filename_input"),
                            Value::Bool(facts.gist_create_declares_filename_input),
                        ),
                        (
                            $ctx.sym("gist_create_files_keyed_by_filename"),
                            Value::Bool(facts.gist_create_files_keyed_by_filename),
                        ),
                        ($ctx.sym("input_facts"), list_value(input_items)),
                        (
                            $ctx.sym("source_nickname_literal_count"),
                            Value::Int(facts.source_nickname_literal_count),
                        ),
                    ])),
                };
                Ok(Some(result))
            },

            arm "free_call.extdeps_external_authority_facts_for_qualified_name" { "extdeps_external_authority_facts_for_qualified_name" } => {
                let qn = $positional.first().ok_or_else(|| InterpError::TypeError {
                    msg: "extdeps_external_authority_facts_for_qualified_name requires a QualifiedName"
                        .to_string(),
                })?;
                let module_path = crate::cli_run::free_monoid_symbol_value_to_dotted_string(qn);
                let facts = crate::cli_run::extdeps_external_authority_module_facts(&module_path);
                let result = Value::Record {
                    type_name: $ctx.sym("ExtdepsExternalAuthorityModuleFacts"),
                    fields: Rc::new(sorted_fields(vec![
                        ($ctx.sym("anchor_kind"), str_value(facts.anchor_kind)),
                        (
                            $ctx.sym("scheme_identity"),
                            str_value(facts.scheme_identity),
                        ),
                        ($ctx.sym("locator"), str_value(facts.locator)),
                    ])),
                };
                Ok(Some(result))
            },

            arm "free_call.extdeps_external_authority_live_clean_tree_holds" { "extdeps_external_authority_live_clean_tree_holds" } => Ok(Some(Value::Bool(
                crate::cli_run::extdeps_external_authority_live_clean_tree_holds(),
            ))),
            arm "free_call.extdeps_external_authority_live_roster_module_count" { "extdeps_external_authority_live_roster_module_count" } => Ok(Some(Value::Int(
                crate::cli_run::extdeps_external_authority_live_roster_module_count(),
            ))),

            arm "free_call.seed_runner_bool_false_failure_detail" { "seed_runner_bool_false_failure_detail" } => {
                let witness = expect_str($positional.first().copied(), $name)?;
                Ok(Some(str_value(crate::cli_run::seed_runner_bool_false_failure_detail(
                    $ctx, &witness,
                ))))
            },

            arm "free_call.doc_graph_orphan_count" { "doc_graph_orphan_count" } => {
                let extra_roots = expect_str_list($positional.first().copied(), $name)?;
                Ok(Some(Value::Int(crate::cli_run::doc_graph_orphan_count(
                    extra_roots,
                ))))
            },
            arm "free_call.doc_graph_admitted_root_count" { "doc_graph_admitted_root_count" } => {
                let extra_roots = expect_str_list($positional.first().copied(), $name)?;
                Ok(Some(Value::Int(
                    crate::cli_run::doc_graph_admitted_root_count(extra_roots),
                )))
            },
            arm "free_call.doc_graph_dangling_link_count" { "doc_graph_dangling_link_count" } => Ok(Some(Value::Int(
                crate::cli_run::doc_graph_dangling_link_count(),
            ))),
            arm "free_call.doc_graph_doc_count" { "doc_graph_doc_count" } => Ok(Some(Value::Int(crate::cli_run::doc_graph_doc_count()))),

            arm "free_call.compile_dag_rust_emit_check" { "compile_dag_rust_emit_check" } => {
                let source = expect_str($positional.first().copied(), $name)?;
                let file_path = expect_str($positional.get(1).copied(), $name)?;
                let includes = expect_str_list($positional.get(2).copied(), $name)?;
                let excludes = expect_str_list($positional.get(3).copied(), $name)?;
                Ok(Some(Value::Bool(
                    crate::cli_run::compile_dag_rust_emit_check(
                        &source, &file_path, &includes, &excludes,
                    ),
                )))
            },

            arm "free_call.compile_dag_diagnostic_census" { "compile_dag_diagnostic_census" } => {
                let source = expect_str($positional.first().copied(), $name)?;
                Ok(Some(compile_diagnostic_census_value(
                    crate::cli_run::compile_dag_diagnostic_census(&source),
                    $ctx,
                )))
            },

            arm "free_call.observe_declared_import_closure_symbol_binding" { "observe_declared_import_closure_symbol_binding" } => {
                let pool_roots = expect_str_list($positional.first().copied(), $name)?;
                let entry_path = expect_str($positional.get(1).copied(), $name)?;
                let consumer_module = expect_str($positional.get(2).copied(), $name)?;
                let symbol = expect_str($positional.get(3).copied(), $name)?;
                Ok(Some(declared_import_closure_binding_value(
                    crate::cli_run::observe_declared_import_closure_symbol_binding(
                        &pool_roots,
                        &entry_path,
                        &consumer_module,
                        &symbol,
                    ),
                    $ctx,
                )))
            },

            arm "free_call.class_b_import_closure_gate_not_affected_skip" { "class_b_import_closure_gate_not_affected_skip" } => Ok(Some(Value::Bool(
                crate::cli_run::class_b_import_closure_gate_not_affected_skip_for_ci(),
            ))),

            arm "free_call.witness_layer_roots_compile_clean_check" { "witness_layer_roots_compile_clean_check" } => Ok(Some(Value::Bool(
                crate::cli_run::witness_layer_roots_compile_clean_check(),
            ))),

            arm "free_call.witness_layer_roots_compile_clean_emit_check" { "witness_layer_roots_compile_clean_emit_check" } => Ok(Some(Value::Bool(
                crate::cli_run::witness_layer_roots_compile_clean_emit_check(),
            ))),
            arm "free_call.consume_floor_compile_clean_gate_verdict" { "consume_floor_compile_clean_gate_verdict" } => Ok(Some(Value::Bool(
                crate::cli_run::consume_floor_compile_clean_gate_verdict(),
            ))),

            arm "free_call.consume_floor_compile_clean_gate_failure_detail" { "consume_floor_compile_clean_gate_failure_detail" } => Ok(Some(str_value(
                crate::cli_run::consume_floor_compile_clean_gate_failure_detail(),
            ))),

            arm "free_call.record_regen_verify_gate_failure_detail" { "record_regen_verify_gate_failure_detail" } => {
                if let [Value::Str(detail)] = $positional.as_slice() {
                    crate::cli_run::record_regen_verify_gate_failure_detail(detail.to_string());
                }
                Ok(Some(Value::Unit))
            },

            arm "free_call.consume_regen_verify_gate_failure_detail" { "consume_regen_verify_gate_failure_detail" } => Ok(Some(str_value(
                crate::cli_run::consume_regen_verify_gate_failure_detail(),
            ))),

            arm "free_call.record_generated_artifact_drift_gate_failure_detail" { "record_generated_artifact_drift_gate_failure_detail" } => {
                if let [Value::Str(detail)] = $positional.as_slice() {
                    crate::cli_run::record_generated_artifact_drift_gate_failure_detail(detail.to_string());
                }
                Ok(Some(Value::Unit))
            },

            arm "free_call.consume_generated_artifact_drift_gate_failure_detail" { "consume_generated_artifact_drift_gate_failure_detail" } => Ok(Some(str_value(
                crate::cli_run::consume_generated_artifact_drift_gate_failure_detail(),
            ))),

            arm "free_call.witness_compile_clean_cli_floor_verdicts_agree" { "witness_compile_clean_cli_floor_verdicts_agree" } => Ok(Some(Value::Bool(
                crate::cli_run::witness_compile_clean_cli_floor_verdicts_agree(),
            ))),

            arm "free_call.test_migration_debt_module_names" { "test_migration_debt_module_names" } => {
                let names = crate::cli_run::test_migration_debt_module_names();
                let items: Vec<Value> = names.into_iter().map(str_value).collect();
                Ok(Some(list_value(items)))
            },
            arm "free_call.test_migration_legacy_behavior_ids" { "test_migration_legacy_behavior_ids" } => {
                let ids = crate::cli_run::test_migration_legacy_behavior_ids();
                let items: Vec<Value> = ids.into_iter().map(str_value).collect();
                Ok(Some(list_value(items)))
            },
            arm "free_call.test_migration_witness_behavior_ids" { "test_migration_witness_behavior_ids" } => {
                let ids = crate::cli_run::test_migration_witness_behavior_ids();
                let items: Vec<Value> = ids.into_iter().map(str_value).collect();
                Ok(Some(list_value(items)))
            },
            arm "free_call.test_migration_behavior_discovery_holds" { "test_migration_behavior_discovery_holds" } => Ok(Some(Value::Bool(
                crate::cli_run::test_migration_behavior_discovery_holds(),
            ))),
            arm "free_call.inert_carrier_names_live" { "inert_carrier_names_live" } => {
                let names = crate::cli_run::inert_carrier_names_live();
                let items: Vec<Value> = names.into_iter().map(str_value).collect();
                Ok(Some(list_value(items)))
            },
            arm "free_call.inert_carrier_declared_count" { "inert_carrier_declared_count" } => Ok(Some(Value::Int(
                crate::cli_run::inert_carrier_declared_count_live(),
            ))),

            arm "free_call.non_fold_residue_count" { "non_fold_residue_count" } => Ok(Some(Value::Int(crate::cli_run::non_fold_residue_count()))),
            arm "free_call.non_fold_residue_unrostered_count" { "non_fold_residue_unrostered_count" } => Ok(Some(Value::Int(
                crate::cli_run::non_fold_residue_unrostered_count(),
            ))),
            arm "free_call.non_fold_residue_stale_roster_count" { "non_fold_residue_stale_roster_count" } => Ok(Some(Value::Int(
                crate::cli_run::non_fold_residue_stale_roster_count(),
            ))),
            arm "free_call.non_fold_residue_coproduct_universe_count" { "non_fold_residue_coproduct_universe_count" } => Ok(Some(Value::Int(
                crate::cli_run::non_fold_residue_coproduct_universe_count(),
            ))),

            arm "free_call.commit_witness_claim_roster_unresolvable_count" { "commit_witness_claim_roster_unresolvable_count" } => Ok(Some(Value::Int(
                crate::cli_run::commit_witness_claim_roster_unresolvable_count(),
            ))),
            arm "free_call.commit_witness_claim_pair_resolvable" { "commit_witness_claim_pair_resolvable" } => {
                let entry = expect_str(
                    $positional.first().copied(),
                    "commit_witness_claim_pair_resolvable entry",
                )?;
                let function = expect_str(
                    $positional.get(1).copied(),
                    "commit_witness_claim_pair_resolvable function",
                )?;
                Ok(Some(Value::Bool(
                    crate::cli_run::commit_witness_claim_pair_resolvable(&entry, &function),
                )))
            },
            arm "free_call.non_fold_residue_wildcard_red_fixture_holds" { "non_fold_residue_wildcard_red_fixture_holds" } => Ok(Some(Value::Bool(
                crate::cli_run::non_fold_residue_wildcard_red_fixture_holds(),
            ))),
            arm "free_call.non_fold_residue_total_fold_green_fixture_holds" { "non_fold_residue_total_fold_green_fixture_holds" } => Ok(Some(Value::Bool(
                crate::cli_run::non_fold_residue_total_fold_green_fixture_holds(),
            ))),
            arm "free_call.non_fold_residue_roster_red_fixture_holds" { "non_fold_residue_roster_red_fixture_holds" } => Ok(Some(Value::Bool(
                crate::cli_run::non_fold_residue_roster_red_fixture_holds(),
            ))),
            arm "free_call.non_fold_residue_synthetic_unrostered_red_holds" { "non_fold_residue_synthetic_unrostered_red_holds" } => Ok(Some(Value::Bool(
                crate::cli_run::non_fold_residue_synthetic_unrostered_red_holds(),
            ))),

            arm "free_call.complexity_linearity_syntactic_finding_count" { "complexity_linearity_syntactic_finding_count" } => Ok(Some(Value::Int(
                crate::cli_run::complexity_linearity_syntactic_finding_count(),
            ))),
            arm "free_call.complexity_linearity_wildcard_facts" { "complexity_linearity_wildcard_facts" } => {
                let facts = crate::cli_run::complexity_linearity_wildcard_facts();
                let mut items: Vec<Value> = Vec::new();
                for f in facts {
                    items.push(Value::Record {
                        type_name: $ctx.sym("ComplexityLinearityWildcardFact"),
                        fields: Rc::new(sorted_fields(vec![
                            (
                                $ctx.sym("closed_coproduct_wildcard"),
                                Value::Bool(f.closed_coproduct_wildcard),
                            ),
                            ($ctx.sym("fn_name"), str_value(f.fn_name.clone())),
                            ($ctx.sym("rostered"), Value::Bool(f.rostered)),
                            ($ctx.sym("site"), str_value(f.site.clone())),
                        ])),
                    });
                }
                Ok(Some(list_value(items)))
            },

            arm "free_call.fallback_arm_census_facts" { "fallback_arm_census_facts" } => {
                let facts = crate::cli_run::fallback_arm_census_facts();
                let mut items: Vec<Value> = Vec::new();
                for f in facts {
                    items.push(Value::Record {
                        type_name: $ctx.sym("FallbackArmCensusFact"),
                        fields: Rc::new(sorted_fields(vec![
                            (
                                $ctx.sym("closed_coproduct_scrutinee"),
                                Value::Bool(f.closed_coproduct_scrutinee),
                            ),
                            ($ctx.sym("class"), str_value(f.class.clone())),
                            ($ctx.sym("fn_name"), str_value(f.fn_name.clone())),
                            ($ctx.sym("owning_lane"), str_value(f.owning_lane.clone())),
                            ($ctx.sym("rel_path"), str_value(f.rel_path.clone())),
                            ($ctx.sym("site"), str_value(f.site.clone())),
                        ])),
                    });
                }
                Ok(Some(list_value(items)))
            },
            arm "free_call.fallback_arm_census_class_count" { "fallback_arm_census_class_count" } => {
                let class = expect_str(
                    $positional.first().copied(),
                    "fallback_arm_census_class_count",
                )?;
                Ok(Some(Value::Int(
                    crate::cli_run::fallback_arm_census_class_count(&class),
                )))
            },
            arm "free_call.fallback_arm_census_total" { "fallback_arm_census_total" } => Ok(Some(Value::Int(
                crate::cli_run::fallback_arm_census_total(),
            ))),
            arm "free_call.fallback_arm_census_reconciliation_holds" { "fallback_arm_census_reconciliation_holds" } => Ok(Some(Value::Bool(
                crate::cli_run::fallback_arm_census_reconciliation_holds(),
            ))),

            arm "free_call.complexity_linearity_syntactic_site_fired" { "complexity_linearity_syntactic_site_fired" } => {
                let site = expect_str(
                    $positional.first().copied(),
                    "complexity_linearity_syntactic_site_fired",
                )?;
                Ok(Some(Value::Bool(
                    crate::cli_run::complexity_linearity_syntactic_site_fired(&site),
                )))
            },
            arm "free_call.census_corpus_roots_follow_layer_authority" { "census_corpus_roots_follow_layer_authority" } => Ok(Some(Value::Bool(
                crate::cli_run::census_corpus_roots_follow_layer_authority(),
            ))),

        }
    }};
}

/// Dispatch via roster-generated lookup + exhaustive enum match (R1).
macro_rules! v1_builtin_dispatch {
    ($n:ident, $p:ident, $c:ident; $(arm $id:tt { $($lit:literal)|+ } => $body:expr ,)*) => {
        match $crate::v1_interpreter_dispatch_generated::lookup_eval_builtin_inner($n) {
            Some(arm) => match arm {
                $( eval_builtin_inner_arm!($id) => $body , )*
            },
            None => Ok(None),
        }
    };
}

fn eval_builtin_inner(
    name: &str,
    args: &[(Option<String>, Value)],
    ctx: &InterpContext,
) -> InterpResult<Option<Value>> {
    let positional: Vec<&Value> = args.iter().map(|(_, v)| v).collect();
    v1_builtin_arms!(v1_builtin_dispatch, name, positional, ctx)
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
#[cfg(any(test, feature = "test_hooks"))]
static FORCE_FORENSICS_FOR_TEST: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

#[cfg(any(test, feature = "test_hooks"))]
pub fn set_call_frequency_forensics_for_test(enabled: bool) {
    FORCE_FORENSICS_FOR_TEST.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

fn residual_hunt_forensics_enabled() -> bool {
    #[cfg(any(test, feature = "test_hooks"))]
    if FORCE_FORENSICS_FOR_TEST.load(std::sync::atomic::Ordering::Relaxed) {
        return true;
    }
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
        "uri_percent_encode_scalar_fragment",
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
    static MEMO_VERIFY_FLAG: Cell<Option<bool>> = const { Cell::new(None) };
    /// DIAGNOSTIC (2026-08-10 wedge RCA, behind GUNBC_INTERP_PROFILE=1 only).
    /// `ExprCast` measured at 72.9% of daily-page render self-time at ~38.5us/cast. The
    /// suspected shape is that a cast resolves its target type by SCANNING every item of
    /// every module and extracting authored source text per item, once per alias-chain hop.
    /// These three counters make the multiplier observable instead of argued: calls to the
    /// kernel walk, lookups it drives, and items those lookups actually touch.
    static CAST_KERNEL_CALLS: Cell<u64> = const { Cell::new(0) };
    static TYPE_LOOKUP_CALLS: Cell<u64> = const { Cell::new(0) };
    static TYPE_LOOKUP_ITEMS: Cell<u64> = const { Cell::new(0) };
}

/// Diagnostic counters for the cast cost center. Zero-cost when profiling is off.
pub fn cast_lookup_counters() -> (u64, u64, u64) {
    (
        CAST_KERNEL_CALLS.with(|c| c.get()),
        TYPE_LOOKUP_CALLS.with(|c| c.get()),
        TYPE_LOOKUP_ITEMS.with(|c| c.get()),
    )
}

/// Verification mode for the pointer-keyed cast memo (see cast_target_names). Off by default;
/// when on, every cache hit is checked against a fresh uncached resolution.
fn memo_verify_enabled() -> bool {
    MEMO_VERIFY_FLAG.with(|c| match c.get() {
        Some(b) => b,
        None => {
            let b = std::env::var("GUNBC_MEMO_VERIFY")
                .map(|v| v == "1")
                .unwrap_or(false);
            c.set(Some(b));
            b
        }
    })
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
    /// Wall-clock duration of the witness — the MEASUREMENT basis, and what every
    /// existing receipt column projects.
    pub wall_nanos: u128,
    /// Thread-CPU duration of the witness — the ENFORCEMENT basis: the quantity the
    /// fast-lane cap is actually compared against, by both the cooperative stride-poll
    /// (`EvalBudgetExceeded`) and the completion-side backstop (`BudgetKind::Cpu`).
    ///
    /// Carried beside `wall_nanos` rather than replacing it because they are two clocks
    /// on one occurrence and neither substitutes for the other. Before this field existed
    /// the enforced quantity was computed, spent on the budget decision, and dropped — so
    /// no artifact in the tree recorded the number the cap reads, and any threshold built
    /// on a cost receipt selected a different population than the cap kills.
    ///
    /// Recording both clocks is correct; what is provisional is that this one says which
    /// clock it is only by its NAME. See `WITNESS_COST_CLOCK_BASIS_NOTE` for the ruled
    /// model that replaces it (a basis-carrying measurement) and the dissolution trigger.
    ///
    /// The useful bound, since eval is single-threaded and CPU is therefore bounded above
    /// by wall: a recorded wall UNDER the cap proves CPU under the cap, so budget triggers
    /// stated as "lands under the fast-lane budget" were always decidable from wall alone.
    /// What wall cannot answer is the other direction — how near the cap a row sits, or
    /// whether an over-cap wall figure reflects CPU at all — which is exactly the ranking
    /// question the per-witness cost-envelope lane needs.
    pub cpu_nanos: u128,
    pub eval_self_nanos: u128,
    pub sample_count: u64,
}

pub const WITNESS_COST_CLOCK_BASIS_NOTE: &str = "\
Witness cost carries TWO clocks and they are not interchangeable. wall is the measurement \
basis; cpu is the enforcement basis (thread CPU, what the fast-lane cap is compared \
against). Recording only wall meant the enforced quantity appeared in no artifact. Eval is \
single-threaded, so cpu <= wall: a wall figure under the cap PROVES cpu under the cap \
(firing direction, decidable), while an over-cap wall figure proves nothing about cpu \
(ranking direction, blind). \
\
RECORDING BOTH CLOCKS IS CORRECT AND IS NOT THE DEFECT (operator ruling 2026-08-05). The \
defect a second bare field would introduce is a duration whose meaning lives only in its \
field NAME. The ruled model is that each observed duration CARRIES ITS OWN BASIS -- \
ClockBasis = CpuClock | WallClock, a basis-carrying elapsed measurement, and a basis on the \
TimedOut outcome so a crossing says which cap it crossed. Under that model a receipt may \
legitimately hold both a cpu and a wall observation, because neither is relying on its name \
to say what it is. \
\
SEED DISPOSITION: the u128 fields below are seed instrumentation, and they are the ONLY \
reason this note exists -- they carry their basis in a field name, which is exactly what the \
ruled model replaces. They are retained because the enforced quantity had to stop being \
dropped before the authority lands, and the seed is not where the authority belongs. \
DISSOLVE-ON: ClockBasis lands in the std.observation authority and the declaration-grain \
receipt projects cpu and wall through it; at that point these two bare fields are replaced \
by basis-carrying measurements and this note is deleted with them.";

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

/// Both clocks are REQUIRED parameters. Defaulting `cpu_nanos` would reintroduce exactly
/// the defect this field closes: a caller that forgot it would silently record zero for the
/// enforced quantity, which reads as "measured, and free" rather than "not measured".
pub fn performance_receipt_from_witness(
    subject_key: String,
    work_shape: &str,
    wall_nanos: u128,
    cpu_nanos: u128,
) -> PerformanceReceipt {
    let eval_self_nanos = SUBJECT_SELF_NANOS
        .with(|m| m.borrow().get(&subject_key).copied())
        .unwrap_or(0);
    PerformanceReceipt {
        subject_key,
        work_shape: work_shape.to_string(),
        wall_nanos,
        cpu_nanos,
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
/// O(n^2) tax the compiled (Rust-emitted) realization never pays. Method-call
/// `.length()` on native `Value::Str` routes through `string_length_ascii_aware`
/// so it does not flatten strings into per-codepoint `Value`s (LIST-CARRIER-0 /
/// materialize OOM). Free-call `length`/`string_length` already avoided
/// `free_monoid_to_vec` on `Str` via `chars().count()`; this arm closes the
/// method-call gap only.
pub(crate) fn native_len(val: &Value) -> Option<i64> {
    match val {
        Value::List(items) => Some(items.len() as i64),
        Value::Map(m) => Some(m.len() as i64),
        Value::Set(s) => Some(s.len() as i64),
        // Method-call `.length()` on a native `Value::Str` must not fall through to
        // `free_monoid_to_vec` (which materializes one `Value` per codepoint). JSON
        // parsing alone calls `.length()` O(n) times on the input buffer; without this
        // arm that is O(n^2) allocations and pins multi-gigabyte RSS on ~500KB inputs
        // (srv1 materialize_codex_runtime_bundle bisect, 2026-08-14).
        //
        // LIMIT: non-ASCII .length()/.count() remains O(n) per call via the chars() walk.
        // REASON: the ASCII fast path covers the dominant repeated-query case, and genuinely
        // non-ASCII strings in this corpus are constructed-then-queried-once-or-never, so
        // precomputing a codepoint count at construction would not amortize. Flag the
        // ASCII-in-practice half explicitly AS AN ASSUMPTION about workloads, not a modeled
        // fact — §6 is clear that "n is small here" is not time-stable.
        // NEXT-RUNG TRIGGER: a workload that repeatedly length-queries the same non-ASCII
        // string. If that appears, the amortization argument inverts and a carried count
        // becomes correct.
        Value::Str(s) => Some(v1_rt::string_length_ascii_aware(&s, s.is_ascii())),
        _ => None,
    }
}

#[cfg(test)]
mod native_len_tests {
    use super::*;

    #[test]
    fn native_len_str_avoids_free_monoid_materialization() {
        let big = str_value(&"a".repeat(50_000));
        let (calls_before, items_before) = flatten_counters_snapshot();
        let n = native_len(&big).expect("native Str length");
        assert_eq!(n, 50_000);
        let (calls_after, items_after) = flatten_counters_snapshot();
        assert_eq!(
            (calls_after, items_after),
            (calls_before, items_before),
            "native_len on Value::Str must not call free_monoid_to_vec"
        );
    }

    #[test]
    fn native_len_str_counts_unicode_scalar_length() {
        let s = str_value("é"); // one scalar, two UTF-8 bytes
        assert_eq!(native_len(&s), Some(1));
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
        return Some(s.to_string());
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

fn raw_map_lookup(
    map: &Value,
    key: &Value,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<RawMapLookup> {
    match map {
        Value::Map(m) => match CanonKey::new(key.clone()) {
            Some(ck) => Ok(RawMapLookup::NeedsWrap(
                m.get(&ck).cloned().unwrap_or(Value::Null),
            )),
            None => Ok(RawMapLookup::NeedsWrap(Value::Null)),
        },
        Value::Record { fields, .. } | Value::Variant { fields, .. } => {
            let lookup_sym = ctx.sym("lookup");
            match fields_get(fields, lookup_sym) {
                Some(lookup @ Value::Closure { .. }) => Ok(RawMapLookup::AlreadyOptional(
                    apply_closure(lookup, &[key.clone()], env, ctx)?,
                )),
                Some(Value::Fn { node }) => {
                    let named = vec![(None, key.clone())];
                    Ok(RawMapLookup::AlreadyOptional(call_function(
                        ctx, node, &named, env,
                    )?))
                }
                Some(_) => Err(InterpError::TypeError {
                    msg: "Map.lookup field is not callable".to_string(),
                }),
                None => match key {
                    Value::Str(s) => {
                        let k = ctx.sym(s);
                        Ok(RawMapLookup::NeedsWrap(
                            fields_get(fields, k).cloned().unwrap_or(Value::Null),
                        ))
                    }
                    _ => Ok(RawMapLookup::NeedsWrap(Value::Null)),
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
        Value::Str(s) => Ok(s.to_string()),
        _ => Err(InterpError::TypeError {
            msg: format!("{} expects a string, got {}", context, val.type_label()),
        }),
    }
}

fn expect_str(val: Option<&Value>, context: &str) -> InterpResult<String> {
    match val {
        Some(Value::Str(s)) => Ok(s.to_string()),
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
                    Value::Str(s) => out.push(s.to_string()),
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
            Value::Str(s) => out.push(s.to_string()),
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
    use super::neutralize_workflow_commands;
    use super::render_shell_effect_begin_line_mirror;
    use super::render_shell_effect_done_line_mirror;
    use super::render_shell_effect_failed_line_mirror;
    use super::shell_argv_collapsed;
    use super::shell_completion_stderr_content;
    use super::shell_failure_surfaces;
    use super::{
        effect_stream_disposition, ExpectedOutcome, StreamDisposition,
        EFFECT_STREAM_POLICY_FALLBACK, SUBJECT_LINE_GUARD_FALLBACK,
    };

    /// Convenience for the tests below: the disposition a failing effect gets when
    /// its caller declared success — the migration default, and what every call
    /// site issues today.
    fn surfacing() -> StreamDisposition {
        StreamDisposition::SurfaceContent
    }

    fn av(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn shell_effect_begin_mirror_formats_started_subject() {
        let line = render_shell_effect_begin_line_mirror("shell.Exec.Run", true);
        assert_eq!(line, "🔄 started shell.Exec.Run");
        let unicode = render_shell_effect_begin_line_mirror("git.Inspect.HeadCommit", false);
        assert_eq!(unicode, "◐ started git.Inspect.HeadCommit");
    }

    #[test]
    fn shell_effect_done_mirror_formats_duration() {
        let line = render_shell_effect_done_line_mirror("shell.Exec.Run", 5150, true);
        assert_eq!(line, "✅ shell.Exec.Run done in 5 seconds");
    }

    #[test]
    fn shell_effect_failed_mirror_is_self_describing() {
        // Failed.error carries `$ <argv> (exit=N)` so the line stands alone when
        // stderr is empty (the common CI miss).
        let line =
            render_shell_effect_failed_line_mirror("shell.Exec.Run", "echo hi", 1, 2000, true);
        assert_eq!(
            line,
            "❌ shell.Exec.Run failed: $ echo hi (exit=1) in 2 seconds"
        );
    }

    #[test]
    fn shell_argv_collapsed_squeezes_whitespace() {
        assert_eq!(
            shell_argv_collapsed(&av(&["sh", "-c", "echo  hi\nthere"])),
            "sh -c echo hi there"
        );
    }

    #[test]
    fn stderr_content_surfaces_body_on_nonzero_exit() {
        let block = shell_completion_stderr_content(b"error: manifest not found\n")
            .expect("non-empty stderr must surface a content block");
        assert!(block.contains("error: manifest not found"));
        assert!(!block.contains("[shell]"));
    }

    #[test]
    fn stderr_content_none_when_empty() {
        assert_eq!(shell_completion_stderr_content(b""), None);
    }

    #[test]
    fn failure_surfaces_from_disposition_alone_not_channel() {
        // RED control: the Failed observation is gated on the .dag disposition, not
        // on a local `exit != 0` re-derivation — and empty stderr does NOT suppress it
        // (the Failed line is the sole signal once Ambient counts are observation Done).
        assert!(shell_failure_surfaces(surfacing()));
        assert!(!shell_failure_surfaces(StreamDisposition::SummarizeCounts));
        assert!(!shell_failure_surfaces(StreamDisposition::StreamSuppressed));
    }

    #[test]
    fn at_normal_a_failing_effect_surfaces_its_command_a_passing_one_is_silent() {
        // Composes the .dag disposition (ExpectSuccess × exit → the Normal four
        // corners via the uninstalled fallback that mirrors Normal) with the
        // failure-surfaces predicate. GREEN: passing → SummarizeCounts → silent.
        // Discriminating RED: failing → SurfaceContent → Failed line must fire
        // even with empty stderr (self-describing `$ argv`).
        assert!(!shell_failure_surfaces(effect_stream_disposition(
            ExpectedOutcome::ExpectSuccess,
            0
        )));
        assert!(shell_failure_surfaces(effect_stream_disposition(
            ExpectedOutcome::ExpectSuccess,
            1
        )));
        let collapsed = shell_argv_collapsed(&av(&["git", "rev-parse", "--show-toplevel"]));
        let line =
            render_shell_effect_failed_line_mirror("git.Inspect.Toplevel", &collapsed, 1, 0, false);
        assert!(line.contains("failed: $ git rev-parse --show-toplevel (exit=1)"));
        assert!(line.starts_with("✗ git.Inspect.Toplevel failed:"));
        assert_eq!(shell_completion_stderr_content(b""), None);
    }

    #[test]
    fn stderr_content_tail_bounds_and_marks_elision() {
        let big = vec![b'x'; 16384 + 500];
        let block = shell_completion_stderr_content(&big).expect("oversized stderr surfaces");
        assert!(block.contains("<500 earlier stderr bytes elided>"));
        assert_eq!(block.chars().rev().take_while(|c| *c == 'x').count(), 16384);
    }

    #[test]
    fn surfaced_subject_text_cannot_mint_workflow_commands() {
        // The priced incident: a child's stderr legitimately carrying `::error::`
        // annotated the PARENT run as failing. Every relayed line is guarded, so no
        // subject text can occupy the line-initial `::` position GitHub parses.
        let block = shell_completion_stderr_content(
            b"::error::build verification: artifact absent\n::warning::next\n",
        )
        .expect("failing effect surfaces its stderr");
        for line in block.lines() {
            assert!(
                !line.trim_start().starts_with("::"),
                "relayed subject line is command-bearing: {line:?}"
            );
        }
        assert!(block.contains("| ::error::build verification: artifact absent"));
    }

    #[test]
    fn effect_stream_policy_mirror_matches_dag_authority() {
        // Mirror pin for the uninstalled fallback: these are the six corners the
        // .dag witness `w_shell_trace_stream_policy_projects_the_four_corners`
        // asserts at Normal verbosity, and the guard literal
        // `extdeps.github.log_annotations.subject_text_line_guard` publishes. If the
        // authority moves and this does not, the two go red together rather than
        // drifting silently.
        assert_eq!(
            EFFECT_STREAM_POLICY_FALLBACK,
            [
                StreamDisposition::SummarizeCounts,
                StreamDisposition::SurfaceContent,
                StreamDisposition::SurfaceContent,
                StreamDisposition::SummarizeCounts,
                // OutcomeIsData: neither pole is an anomaly — the exit is an answer.
                StreamDisposition::SummarizeCounts,
                StreamDisposition::SummarizeCounts,
            ]
        );
        assert_eq!(SUBJECT_LINE_GUARD_FALLBACK, "| ");
        assert_eq!(neutralize_workflow_commands("a\nb"), "| a\n| b");
    }

    #[test]
    fn divergence_surfaces_and_agreement_counts() {
        // The whole rule, both directions. The second pair is the leak nobody had
        // hit: an effect declared to fail that SUCCEEDS is the most
        // diagnosis-worthy event in the system and previously logged nothing.
        assert_eq!(
            effect_stream_disposition(ExpectedOutcome::ExpectSuccess, 1),
            StreamDisposition::SurfaceContent
        );
        assert_eq!(
            effect_stream_disposition(ExpectedOutcome::ExpectSuccess, 0),
            StreamDisposition::SummarizeCounts
        );
        assert_eq!(
            effect_stream_disposition(ExpectedOutcome::ExpectFailure, 1),
            StreamDisposition::SummarizeCounts
        );
        assert_eq!(
            effect_stream_disposition(ExpectedOutcome::ExpectFailure, 0),
            StreamDisposition::SurfaceContent
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
mod shell_stdout_overflow_refusal_tests {
    use super::bounded_shell_host_drain::{
        ShellCaptureResult, StreamCaptureObservation, DEFAULT_SHELL_STDOUT_MAX_BYTES,
    };
    use super::{shell_result_from_capture, InterpError};
    use std::process::{Command, Stdio};

    #[test]
    fn shell_result_from_capture_refuses_truncated_stdout_observation() {
        let capture = ShellCaptureResult {
            exit_status: Command::new("true").status().expect("true status"),
            stdout: StreamCaptureObservation {
                total_bytes: (DEFAULT_SHELL_STDOUT_MAX_BYTES as u64) + 1,
                retained: Vec::new(),
                truncated: true,
                digest_hex: None,
            },
            stderr: StreamCaptureObservation {
                total_bytes: 0,
                retained: Vec::new(),
                truncated: false,
                digest_hex: None,
            },
        };
        let err = shell_result_from_capture(&capture, "fixture")
            .expect_err("truncated stdout must refuse");
        match err {
            InterpError::ShellOutputLimitExceeded {
                stream,
                total_bytes,
                limit_bytes,
                argv0,
            } => {
                assert_eq!(stream, "stdout");
                assert_eq!(total_bytes, (DEFAULT_SHELL_STDOUT_MAX_BYTES as u64) + 1);
                assert_eq!(limit_bytes, DEFAULT_SHELL_STDOUT_MAX_BYTES as u64);
                assert_eq!(argv0, "fixture");
            }
            other => panic!("expected ShellOutputLimitExceeded, got {other:?}"),
        }
    }

    #[test]
    fn capture_then_shell_result_refuses_oversized_stdout_child() {
        let child = Command::new("sh")
            .arg("-c")
            .arg("dd if=/dev/zero bs=1048576 count=9 2>/dev/null | tr '\\0' 'a'")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn stdout overflow child");
        let capture = super::bounded_shell_host_drain::capture_child_output(
            child,
            super::bounded_shell_host_drain::default_shell_stdout_capture_policy(),
            super::bounded_shell_host_drain::default_shell_stderr_capture_policy(),
        )
        .expect("bounded capture");
        assert!(
            capture.stdout.truncated,
            "stdout child must report overflow"
        );
        assert!(
            capture.stdout.retained.is_empty(),
            "overflow must not retain a prefix"
        );
        let err = shell_result_from_capture(&capture, "sh").expect_err("overflow must refuse");
        match err {
            InterpError::ShellOutputLimitExceeded { stream, .. } => {
                assert_eq!(stream, "stdout");
            }
            other => panic!("expected ShellOutputLimitExceeded, got {other:?}"),
        }
    }

    #[test]
    fn shell_result_preserves_capture_evidence_on_success() {
        let child = Command::new("sh")
            .arg("-c")
            .arg("printf small; printf x >&2")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn small child");
        let capture = super::bounded_shell_host_drain::capture_child_output(
            child,
            super::bounded_shell_host_drain::default_shell_stdout_capture_policy(),
            super::bounded_shell_host_drain::default_shell_stderr_capture_policy(),
        )
        .expect("bounded capture");
        let result = shell_result_from_capture(&capture, "sh").expect("small stdout fits");
        assert_eq!(result.stdout.total_bytes, 5);
        assert_eq!(result.stdout.retained_bytes, 5);
        assert!(!result.stdout.truncated);
        assert_eq!(result.stderr.total_bytes, 1);
        assert_eq!(result.stderr.retained_bytes, 1);
        assert!(result.stderr.digest_hex.is_some());
    }
}

#[cfg(test)]
mod map_shell_outputs_optional_stream_tests {
    use std::rc::Rc;

    use im::{vector as im_vec, HashMap};

    use crate::v1_compiler_infer_emit_info::empty_emit_graph_info;
    use crate::v1_compiler_infer_items::ResolvedGraph;
    use crate::v1_std_core::{
        make_field_init_node, make_field_node, make_span, make_text_part_node, Cardinality,
        Connective, ExprData, InferredNode, Node,
    };

    use super::bounded_shell_host_drain::CapturedStreamEvidence;
    use super::{map_shell_outputs, str_value, ExecutionMode, InterpContext, ShellResult, Value};

    fn map_shell_outputs_test_context() -> InterpContext {
        let graph = ResolvedGraph {
            modules: Rc::new(im_vec![]),
            item_registry: Rc::new(HashMap::new()),
            diagnostics: Rc::new(im_vec![]),
            emit_graph_info: empty_emit_graph_info(),
        };
        InterpContext::new(&graph, Rc::new(HashMap::new()), ExecutionMode::Hermetic)
    }

    fn bare_type_node(name: &str, span: Rc<crate::v1_std_core::SourceSpan>) -> Rc<Node> {
        Rc::new(Node {
            name: name.to_string(),
            span: span.clone(),
            ident_span: None,
            children: Rc::new(im_vec![]),
            connective: Connective::NoConnective,
            params: Rc::new(im_vec![]),
            inferred: None,
            return_cardinality: Cardinality::Required,
            uses: Rc::new(im_vec![]),
            body: None,
            transport: None,
            properties: Rc::new(im_vec![]),
            type_annotation: None,
            is_self_recursive: false,
            has_non_tail_self_call: false,
            match_pattern: None,
            expr_data: Rc::new(ExprData::NoExprData),
            ident: None,
        })
    }

    fn shell_result_fixture(exit_code: i32, stdout_text: &str, stderr_text: &str) -> ShellResult {
        ShellResult {
            exit_code,
            stdout: CapturedStreamEvidence {
                total_bytes: stdout_text.len() as u64,
                retained_bytes: stdout_text.len() as u64,
                truncated: false,
                digest_hex: None,
                retained_text: stdout_text.to_string(),
            },
            stderr: CapturedStreamEvidence {
                total_bytes: stderr_text.len() as u64,
                retained_bytes: stderr_text.len() as u64,
                truncated: false,
                digest_hex: None,
                retained_text: stderr_text.to_string(),
            },
        }
    }

    fn map_optional_stream_field(exit_code: i32, from_key: &str) -> Value {
        let ctx = map_shell_outputs_test_context();
        let span = make_span(0, 0);
        let str_type = bare_type_node("String", span.clone());
        let mut field = make_field_node(
            from_key.to_string(),
            str_type,
            Cardinality::CardOptional,
            None,
            Some(from_key.to_string()),
            span.clone(),
            span.clone(),
        );
        // make_field_node's from_key stub is not a LitStr; extract_from_key needs one.
        let from_key_prop = make_field_init_node(
            "from_key".to_string(),
            make_text_part_node(from_key.to_string(), span.clone()),
            span.clone(),
            span.clone(),
        );
        Rc::make_mut(&mut field).properties = Rc::new(im_vec![from_key_prop]);
        let return_type = Rc::new(Node {
            name: "FixtureShellRecord".to_string(),
            span: span.clone(),
            ident_span: None,
            children: Rc::new(im_vec![field]),
            connective: Connective::NoConnective,
            params: Rc::new(im_vec![]),
            inferred: None,
            return_cardinality: Cardinality::Required,
            uses: Rc::new(im_vec![]),
            body: None,
            transport: None,
            properties: Rc::new(im_vec![]),
            type_annotation: None,
            is_self_recursive: false,
            has_non_tail_self_call: false,
            match_pattern: None,
            expr_data: Rc::new(ExprData::NoExprData),
            ident: None,
        });
        let op_node = Rc::new(Node {
            name: "fixture_shell_op".to_string(),
            span: span.clone(),
            ident_span: None,
            children: Rc::new(im_vec![]),
            connective: Connective::NoConnective,
            params: Rc::new(im_vec![]),
            inferred: Some(Rc::new(InferredNode::Resolved { node: return_type })),
            return_cardinality: Cardinality::Required,
            uses: Rc::new(im_vec![]),
            body: None,
            transport: None,
            properties: Rc::new(im_vec![]),
            type_annotation: None,
            is_self_recursive: false,
            has_non_tail_self_call: false,
            match_pattern: None,
            expr_data: Rc::new(ExprData::NoExprData),
            ident: None,
        });
        let mapped = map_shell_outputs(
            &shell_result_fixture(exit_code, "captured-stdout", "captured-stderr"),
            &op_node,
            &ctx,
        )
        .expect("map_shell_outputs");
        match mapped {
            Value::Record { fields, .. } => fields
                .iter()
                .find(|(sym, _)| ctx.sym(from_key) == *sym)
                .map(|(_, v)| v.clone())
                .unwrap_or(Value::Null),
            other => panic!("expected record, got {other:?}"),
        }
    }

    #[test]
    fn optional_stdout_is_null_on_failed_shell_not_captured_text() {
        assert_eq!(
            map_optional_stream_field(1, "stdout"),
            Value::Null,
            "optional stdout must stay absent when the shell failed"
        );
    }

    #[test]
    fn optional_stderr_is_null_on_failed_shell_not_captured_text() {
        assert_eq!(
            map_optional_stream_field(1, "stderr"),
            Value::Null,
            "optional stderr must stay absent when the shell failed"
        );
    }

    #[test]
    fn optional_stdout_surfaces_text_on_success() {
        assert_eq!(
            map_optional_stream_field(0, "stdout"),
            str_value("captured-stdout".to_string())
        );
    }
}

#[cfg(test)]
mod wall_deadline_kill_tests {
    use std::process::{Command, Stdio};
    use std::rc::Rc;
    use std::time::Instant;

    use im::{vector as im_vec, HashMap};

    use crate::v1_compiler_infer_emit_info::empty_emit_graph_info;
    use crate::v1_compiler_infer_items::ResolvedGraph;

    use super::{
        map_budget_error_to_witness_refusal, wait_child_honoring_wall_deadline, EvaluationClock,
        ExecutionMode, InterpContext, InterpError,
    };

    fn wet_ctx() -> InterpContext {
        let graph = ResolvedGraph {
            modules: Rc::new(im_vec![]),
            item_registry: Rc::new(HashMap::new()),
            diagnostics: Rc::new(im_vec![]),
            emit_graph_info: empty_emit_graph_info(),
        };
        InterpContext::new(&graph, Rc::new(HashMap::new()), ExecutionMode::Wet)
    }

    /// REGRESSION CONTROL ON EXISTING BEHAVIOR, not new coverage. Against the paired
    /// `arm_eval_deadline` / `clear_eval_deadline` calls this replaces, the second assertion
    /// fails: `arm_*` overwrites with a fresh baseline, so an inner scope asking for a LONGER
    /// limit wins and extends its caller's bound. Asserting only the tightening direction would
    /// pass against that inverted behavior and prove nothing.
    #[test]
    fn nested_budget_scope_can_only_tighten() {
        let ctx = wet_ctx();

        let _outer = ctx.enter_evaluation_budget("outer", Some(50), None);
        let outer_remaining = ctx.eval_deadline_remaining_ms().expect("outer armed");
        assert!(outer_remaining <= 50, "outer_remaining={outer_remaining}");

        {
            let _inner = ctx.enter_evaluation_budget("inner", Some(10), None);
            let inner = ctx.eval_deadline_remaining_ms().expect("inner armed");
            assert!(inner <= 10, "a tighter inner scope must win: {inner}");
        }

        {
            // The direction that fails against the displaced implementation.
            let _inner = ctx.enter_evaluation_budget("inner", Some(5_000), None);
            let inner = ctx.eval_deadline_remaining_ms().expect("inner armed");
            assert!(
                inner <= 50,
                "a looser inner scope must NOT extend the outer bound: {inner}"
            );
        }
    }

    /// PINS THE DEFECT THE GUARD EXISTS FOR, so `nested_budget_scope_can_only_tighten` cannot be
    /// mistaken for decoration. These are the raw paired calls, exercised directly: a nested
    /// `arm_eval_deadline` with a LOOSER limit replaces the tighter outer bound, and a nested
    /// `clear_eval_deadline` disarms it entirely. Both are the fail-open direction and both are
    /// silent. If either assertion ever flips, the raw calls have been fixed and the guard's
    /// tightening logic can be re-examined; until then this is why callers must not use them
    /// directly around a nestable evaluation.
    #[test]
    fn raw_arm_and_clear_compose_fail_open() {
        let ctx = wet_ctx();

        ctx.arm_eval_deadline(50);
        ctx.arm_eval_deadline(5_000);
        let after_looser_arm = ctx.eval_deadline_remaining_ms().expect("armed");
        assert!(
            after_looser_arm > 50,
            "raw arm was expected to EXTEND the outer bound (the defect); got {after_looser_arm}"
        );

        ctx.arm_eval_deadline(50);
        ctx.clear_eval_deadline();
        assert!(
            ctx.eval_deadline_remaining_ms().is_none(),
            "raw clear was expected to disarm rather than restore (the defect)"
        );
    }

    /// The poisoning case. `gunbc serve` shares one `InterpContext` across every request, and the
    /// CPU baseline is captured at arm time — so a deadline that survived its scope would measure
    /// the NEXT request against a baseline already spent and refuse it immediately. That is worse
    /// than no bound: it converts one stuck request into a permanently broken process.
    #[test]
    fn budget_scope_restores_prior_state_on_exit() {
        let ctx = wet_ctx();
        assert!(ctx.eval_deadline_remaining_ms().is_none());

        {
            let _scope = ctx.enter_evaluation_budget("request-1", Some(25), Some(25));
            assert!(ctx.eval_deadline_remaining_ms().is_some());
            assert!(ctx.wall_deadline_remaining_ms().is_some());
            assert_eq!(ctx.budget_entry().as_deref(), Some("request-1"));
        }

        assert!(
            ctx.eval_deadline_remaining_ms().is_none(),
            "CPU deadline leaked past its scope"
        );
        assert!(
            ctx.wall_deadline_remaining_ms().is_none(),
            "wall deadline leaked past its scope"
        );
        assert_eq!(ctx.budget_entry(), None, "entry identity leaked past scope");
    }

    /// An unset clock is a declared policy state and must not disarm what an outer scope armed.
    #[test]
    fn unset_inner_clock_does_not_disarm_outer_bound() {
        let ctx = wet_ctx();
        let _outer = ctx.enter_evaluation_budget("outer", Some(50), Some(50));
        {
            let _inner = ctx.enter_evaluation_budget("inner", None, None);
            assert!(
                ctx.eval_deadline_remaining_ms().is_some(),
                "an unset inner CPU limit disarmed the outer bound"
            );
            assert!(
                ctx.wall_deadline_remaining_ms().is_some(),
                "an unset inner wall limit disarmed the outer bound"
            );
        }
    }

    /// The kernel result must stay caller-agnostic, and the witness lane must still be able to
    /// recover its own vocabulary from it. Both directions asserted: neutral in, witness out.
    #[test]
    fn budget_error_maps_into_witness_refusal_per_clock() {
        let cpu = InterpError::EvaluationBudgetExceeded {
            entry: "w".to_string(),
            clock: EvaluationClock::ThreadCpu,
            elapsed_nanos: 7_000_000,
            limit_ms: 5,
        };
        match map_budget_error_to_witness_refusal(cpu) {
            InterpError::EvalBudgetExceeded { cpu_ms, budget_ms } => {
                assert_eq!((cpu_ms, budget_ms), (7, 5));
            }
            other => panic!("expected EvalBudgetExceeded, got {other:?}"),
        }

        let wall = InterpError::EvaluationBudgetExceeded {
            entry: "w".to_string(),
            clock: EvaluationClock::MonotonicWall,
            elapsed_nanos: 9_000_000,
            limit_ms: 5,
        };
        match map_budget_error_to_witness_refusal(wall) {
            InterpError::WitnessWallBudgetExceeded { wall_ms, budget_ms } => {
                assert_eq!((wall_ms, budget_ms), (9, 5));
            }
            other => panic!("expected WitnessWallBudgetExceeded, got {other:?}"),
        }

        // Non-budget errors pass through untouched.
        match map_budget_error_to_witness_refusal(InterpError::DivisionByZero) {
            InterpError::DivisionByZero => {}
            other => panic!("pass-through broken: {other:?}"),
        }
    }

    /// The clock keys are a wire contract shared with `std.evaluation_budget.evaluation_clock_key`.
    /// Pinned here so the two go red together rather than drifting silently.
    #[test]
    fn evaluation_clock_keys_match_dag_authority() {
        assert_eq!(EvaluationClock::ThreadCpu.key(), "thread_cpu");
        assert_eq!(EvaluationClock::MonotonicWall.key(), "monotonic_wall");
    }

    #[test]
    fn wall_deadline_kills_sleep_before_completion_backstop() {
        let ctx = wet_ctx();
        // 200ms ceiling vs a 5s sleep: kill-at-deadline must refuse near the
        // ceiling, not after the sleep finishes (the measure-after-spend defect).
        ctx.arm_wall_deadline(200);
        let started = Instant::now();
        let child = Command::new("sleep")
            .arg("5")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn sleep");
        let err = wait_child_honoring_wall_deadline(
            child,
            &ctx,
            "sleep",
            super::bounded_shell_host_drain::default_shell_stdout_capture_policy(),
            super::bounded_shell_host_drain::default_shell_stderr_capture_policy(),
        )
        .expect_err("over-budget sleep must refuse");
        let elapsed_ms = started.elapsed().as_millis() as u64;
        // The KERNEL result is caller-agnostic: this helper is generic shell-wait machinery, and
        // the wall budget being armed only by the witness lane today is a fact about its callers,
        // not about the bound. The witness lane maps this into its own refusal at the claim
        // boundary (`map_budget_error_to_witness_refusal`), which is where its guidance text lives.
        match err {
            InterpError::EvaluationBudgetExceeded {
                clock,
                elapsed_nanos,
                limit_ms,
                ..
            } => {
                assert_eq!(limit_ms, 200);
                assert_eq!(clock, EvaluationClock::MonotonicWall);
                assert!(
                    elapsed_nanos / 1_000_000 >= 200,
                    "elapsed_nanos={elapsed_nanos}"
                );
            }
            other => panic!("expected EvaluationBudgetExceeded, got {other:?}"),
        }
        assert!(
            elapsed_ms < 2000,
            "kill-at-deadline must stop near the 200ms ceiling, spent {elapsed_ms}ms (measure-after-spend would burn ~5000ms)"
        );
    }

    #[test]
    fn no_wall_deadline_waits_to_completion() {
        let ctx = wet_ctx();
        let child = Command::new("true")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn true");
        let output = wait_child_honoring_wall_deadline(
            child,
            &ctx,
            "true",
            super::bounded_shell_host_drain::default_shell_stdout_capture_policy(),
            super::bounded_shell_host_drain::default_shell_stderr_capture_policy(),
        )
        .expect("no-deadline wait must succeed");
        assert!(output.exit_status.success());
    }
}

#[cfg(test)]
mod emit_host_admission_flip_test {
    use std::rc::Rc;

    use im::{vector as im_vec, HashMap};

    use crate::v1_compiler_infer_emit_info::empty_emit_graph_info;
    use crate::v1_compiler_infer_items::ResolvedGraph;

    use super::{require_permitted_transport, ExecutionMode, InterpContext, Value};

    fn ctx_in(mode: ExecutionMode) -> InterpContext {
        let graph = ResolvedGraph {
            modules: Rc::new(im_vec![]),
            item_registry: Rc::new(HashMap::new()),
            diagnostics: Rc::new(im_vec![]),
            emit_graph_info: empty_emit_graph_info(),
        };
        InterpContext::new(&graph, Rc::new(HashMap::new()), mode)
    }

    fn verdict(ctx: &InterpContext, variant: &str) -> Value {
        Value::Variant {
            type_name: ctx.sym("AccessDecision"),
            variant_name: ctx.sym(variant),
            fields: Rc::new(vec![]),
        }
    }

    /// The flip: a Permit decision runs in EVERY mode — hermetic included — because
    /// the law is path containment, not a mode bit. The old blanket is_hermetic()
    /// refusal is gone; this is its replacement's discriminating input.
    #[test]
    fn permitted_passes_in_hermetic_mode() {
        let ctx = ctx_in(ExecutionMode::Hermetic);
        let v = verdict(&ctx, "Permit");
        assert!(require_permitted_transport(Some(&v), &ctx, "emit_host_run_transport").is_ok());
    }

    #[test]
    fn outside_grant_refuses_typed_in_every_mode() {
        for mode in [ExecutionMode::Hermetic, ExecutionMode::Wet] {
            let ctx = ctx_in(mode);
            let v = verdict(&ctx, "Deny");
            let err = require_permitted_transport(Some(&v), &ctx, "emit_host_run_transport")
                .expect_err("outside-grant transport must refuse");
            let msg = format!("{err:?}");
            assert!(
                msg.contains("not permitted"),
                "typed refusal names the cause: {msg}"
            );
            assert!(msg.contains("Deny"), "refusal locates the verdict: {msg}");
        }
    }

    #[test]
    fn permit_arm_from_another_carrier_refuses() {
        let ctx = ctx_in(ExecutionMode::Wet);
        let forged = Value::Variant {
            type_name: ctx.sym("UnrelatedDecision"),
            variant_name: ctx.sym("Permit"),
            fields: Rc::new(vec![]),
        };
        let err = require_permitted_transport(Some(&forged), &ctx, "emit_host_run_transport")
            .expect_err("a Permit arm outside AccessDecision must not authorize a host effect");
        let msg = format!("{err:?}");
        assert!(msg.contains("UnrelatedDecision::Permit"), "{msg}");
        assert!(msg.contains("AccessDecision::Permit"), "{msg}");
    }

    #[test]
    fn missing_verdict_refuses() {
        let ctx = ctx_in(ExecutionMode::Wet);
        let err = require_permitted_transport(None, &ctx, "emit_host_run_transport_cached")
            .expect_err("missing admission must refuse");
        assert!(format!("{err:?}").contains("missing authorization decision"));
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
        argv_arg_limit_refusal, dispatch_shell, Env, ExecutionMode, ExpectedOutcome, InterpContext,
        InterpError, HOST_ARG_MAX_STRLEN_BYTES,
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
        match dispatch_shell(
            &transport,
            &env,
            &ctx,
            "shell.Exec.Check",
            ExpectedOutcome::ExpectSuccess,
        ) {
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
        match dispatch_shell(
            &transport,
            &env,
            &ctx,
            "shell.Exec.Check",
            ExpectedOutcome::ExpectSuccess,
        ) {
            Err(InterpError::ArgvExceedsHostArgMax { .. }) => {
                panic!("small argv must not trip the arg-size wall")
            }
            Ok(_) | Err(_) => {}
        }
    }
}

/// Interim seed witnesses for the fail-closed arms above. HAND-RUST GATE
/// explicit deferral (review 44883): not a permanent test surface — delete with
/// `resolve_host_tool_program` when ROADMAP `toolchain-single-resolver` lands
/// (hermetic-tool-provisioning-design.md P2 RED: unpinned tool refuses before
/// spawn, witnessed in `.dag`).
#[cfg(test)]
mod resolve_host_tool_program_tests {
    use super::resolve_host_tool_program;
    use super::InterpError;
    use std::path::PathBuf;
    use std::sync::{Mutex, MutexGuard};

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvRestore {
        path: Option<String>,
        cargo_home: Option<String>,
        home: Option<String>,
        _guard: MutexGuard<'static, ()>,
    }

    impl EnvRestore {
        fn capture() -> Self {
            let guard = ENV_LOCK
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            Self {
                path: std::env::var("PATH").ok(),
                cargo_home: std::env::var("CARGO_HOME").ok(),
                home: std::env::var("HOME").ok(),
                _guard: guard,
            }
        }
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            restore_env("PATH", self.path.as_deref());
            restore_env("CARGO_HOME", self.cargo_home.as_deref());
            restore_env("HOME", self.home.as_deref());
        }
    }

    fn restore_env(name: &str, value: Option<&str>) {
        match value {
            Some(v) => std::env::set_var(name, v),
            None => std::env::remove_var(name),
        }
    }

    fn isolated_probe_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("gunbc_resolve_host_tool_{label}"))
    }

    #[test]
    fn resolve_host_tool_program_refuses_missing_bare_name() {
        let _env = EnvRestore::capture();
        let root = isolated_probe_root("refuse");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("cargo_home/bin")).expect("cargo_home/bin");
        std::fs::create_dir_all(&root.join("home")).expect("home");

        std::env::set_var("PATH", root.join("empty_path"));
        std::env::set_var("CARGO_HOME", root.join("cargo_home"));
        std::env::set_var("HOME", root.join("home"));

        let missing = "__gunbc_resolve_host_tool_missing__";
        match resolve_host_tool_program(missing) {
            Err(InterpError::HostToolUnresolved { name, probed }) => {
                assert_eq!(name, missing);
                assert!(!probed.is_empty());
            }
            Ok(path) => panic!("expected refusal, got resolved path {path:?}"),
            Err(other) => panic!("expected HostToolUnresolved refusal, got {other:?}"),
        }
    }

    #[test]
    fn resolve_host_tool_program_returns_resolved_path_from_path_probe() {
        let _env = EnvRestore::capture();
        let root = isolated_probe_root("path_hit");
        let _ = std::fs::remove_dir_all(&root);
        let bin_dir = root.join("bin");
        std::fs::create_dir_all(&bin_dir).expect("bin dir");
        let tool_name = "__gunbc_resolve_host_tool_present__";
        let tool_path = bin_dir.join(tool_name);
        std::fs::write(&tool_path, b"").expect("tool file");

        std::env::set_var("PATH", &bin_dir);
        std::env::set_var("CARGO_HOME", root.join("unused_cargo_home"));
        std::env::set_var("HOME", root.join("unused_home"));

        let resolved = resolve_host_tool_program(tool_name)
            .unwrap_or_else(|e| panic!("expected PATH resolution, got {e:?}"));
        assert_eq!(resolved, tool_path.to_string_lossy());
        assert_ne!(resolved, tool_name);
    }

    #[test]
    fn resolve_host_tool_program_accepts_produced_program_wire_path() {
        let resolved = resolve_host_tool_program("./fixture")
            .unwrap_or_else(|e| panic!("ProducedProgram wire path must pass through, got {e:?}"));
        assert_eq!(resolved, "./fixture");
    }

    #[test]
    fn resolve_host_tool_program_refuses_relative_explicit_path() {
        match resolve_host_tool_program("target/release/foo") {
            Err(InterpError::HostToolRelativePathAmbiguous { name }) => {
                assert_eq!(name, "target/release/foo");
            }
            Ok(path) => panic!("expected refusal, got resolved path {path:?}"),
            Err(other) => panic!("expected HostToolRelativePathAmbiguous, got {other:?}"),
        }
    }

    #[test]
    fn resolve_host_tool_program_resolves_existing_explicit_path() {
        let root = isolated_probe_root("explicit_hit");
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("explicit root");
        let tool_path = root.join("__gunbc_resolve_host_tool_explicit__");
        std::fs::write(&tool_path, b"").expect("tool file");
        let absolute = tool_path.to_string_lossy().into_owned();

        let resolved = resolve_host_tool_program(&absolute)
            .unwrap_or_else(|e| panic!("existing explicit path must resolve, got {e:?}"));
        assert_eq!(resolved, absolute);
    }

    #[test]
    fn resolve_host_tool_program_refuses_missing_explicit_path() {
        let missing = "/tmp/__gunbc_resolve_host_tool_explicit_missing__";
        match resolve_host_tool_program(missing) {
            Err(InterpError::HostToolUnresolved { name, probed }) => {
                assert_eq!(name, missing);
                assert_eq!(probed, vec![missing.to_string()]);
            }
            Ok(path) => panic!("expected refusal, got resolved path {path:?}"),
            Err(other) => panic!("expected HostToolUnresolved refusal, got {other:?}"),
        }
    }
}

#[cfg(test)]
mod process_termination_tests {
    use super::process_termination_label;

    /// The host transport observes a child; a child killed by a signal has NO exit
    /// code. The seed used to render `.code().unwrap_or(-1)` for both, so an
    /// OOM-killed cargo build and a process that chose to exit -1 produced the same
    /// bytes. This is the discriminating control for that split: the same raw wait
    /// status that carries a signal must never render as an exit.
    #[cfg(unix)]
    #[test]
    fn signal_death_is_not_flattened_to_an_exit_code() {
        use std::os::unix::process::ExitStatusExt;
        use std::process::ExitStatus;

        // Raw wait status encoding: low 7 bits are the terminating signal, and
        // `code()` is None for those. 9 = SIGKILL (the OOM-killer's signal),
        // 11 = SIGSEGV.
        for signal in [9, 11] {
            let status = ExitStatus::from_raw(signal);
            assert_eq!(status.code(), None, "expected a signalled status");
            assert_eq!(
                process_termination_label(&status),
                format!("signal {signal}")
            );
            assert!(
                !process_termination_label(&status).contains("exit"),
                "signal {signal} rendered as an exit"
            );
        }

        // An ordinary exit still reports its code: status >> 8 is the exit code.
        assert_eq!(
            process_termination_label(&ExitStatus::from_raw(0)),
            "exit 0"
        );
        assert_eq!(
            process_termination_label(&ExitStatus::from_raw(101 << 8)),
            "exit 101"
        );
    }
}

/// Semantic parity receipts for `Value::Str(Rc<str>)` — shared-allocation invariant
/// (clone shares `Rc` allocation; buffer immutable under sharing; equality/hash by
/// content), plus map/set keys, display, and CanonKey surfaces.
#[cfg(test)]
mod value_str_rc_semantic_parity_tests {
    use super::*;
    use im::{HashMap as HamtMap, OrdSet};
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::rc::Rc;

    #[test]
    fn str_eq_compares_by_content_not_pointer() {
        let a = str_value("hello");
        let b = str_value("hello");
        let c = str_value("world");
        assert_eq!(a, b);
        assert_ne!(a, c);
        if let (Value::Str(ra), Value::Str(rb)) = (&a, &b) {
            assert!(
                !Rc::ptr_eq(ra, rb),
                "distinct Rc allocations must still compare equal by content"
            );
        }
    }

    #[test]
    fn str_hash_stable_for_equal_content() {
        let a = str_value("key");
        let b = str_value("key");
        assert_eq!(value_hash_public(&a), value_hash_public(&b));
    }

    #[test]
    fn canon_key_map_lookup_uses_content_equality() {
        let k1 = str_value("alpha");
        let k2 = str_value("alpha");
        let ck1 = CanonKey::new(k1.clone()).expect("string keys are valid CanonKey");
        let ck2 = CanonKey::new(k2.clone()).expect("string keys are valid CanonKey");
        assert_eq!(ck1, ck2);

        let mut entries = HamtMap::new();
        entries = entries.update(ck1, str_value("v"));
        let map = map_value(entries);

        let Value::Map(stored) = map else {
            panic!("expected Map");
        };
        let ck = CanonKey::new(k2).expect("lookup key");
        assert_eq!(stored.get(&ck), Some(&str_value("v")));
    }

    #[test]
    fn set_membership_uses_decoded_string_content() {
        let mut members = OrdSet::new();
        members.insert("x".to_string());
        let set = Value::Set(Rc::new(members));
        let probe = match str_value("x") {
            Value::Str(s) => s.to_string(),
            _ => panic!("expected Str"),
        };
        let Value::Set(members) = set else {
            panic!("expected Set");
        };
        assert!(members.contains(&probe));
        assert_eq!(members.len(), 1);
    }

    #[test]
    fn value_clone_shares_str_rc_allocation() {
        let v = str_value("shared");
        let cloned = v.clone();
        if let (Value::Str(a), Value::Str(b)) = (&v, &cloned) {
            assert!(
                Rc::ptr_eq(a, b),
                "Value::Str clone must share the same Rc allocation (not deep-copy); \
                 content-equality tests alone would stay green if clone reintroduced String copy"
            );
        } else {
            panic!("expected Str");
        }
    }

    #[test]
    fn str_shared_buffer_immutable_under_multiple_refs() {
        let v = str_value("immutable");
        let cloned = v.clone();
        let Value::Str(a) = &v else {
            panic!("expected Str");
        };
        let Value::Str(b) = &cloned else {
            panic!("expected Str");
        };
        assert!(Rc::ptr_eq(a, b));
        let mut lone = Rc::clone(a);
        let mut peer = Rc::clone(b);
        assert!(
            Rc::get_mut(&mut lone).is_none(),
            "shared Rc<str> must refuse in-place mutation while another handle exists"
        );
        assert!(
            Rc::get_mut(&mut peer).is_none(),
            "shared Rc<str> must refuse in-place mutation while another handle exists"
        );
        assert_eq!(a.as_ref(), b.as_ref());
    }

    #[test]
    fn str_display_and_as_ref_interpolation() {
        let v = str_value("roadmap-7");
        assert_eq!(format!("{v}"), "roadmap-7");
        if let Value::Str(s) = &v {
            assert_eq!(s.as_ref(), "roadmap-7");
        }
    }

    #[test]
    fn canon_key_hash_stable_for_equal_keys() {
        let ck1 = CanonKey::new(str_value("map-key")).expect("CanonKey");
        let ck2 = CanonKey::new(str_value("map-key")).expect("CanonKey");
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        ck1.hash(&mut h1);
        ck2.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }
}
