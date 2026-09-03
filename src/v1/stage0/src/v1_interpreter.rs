// CLIPPY ROSTER -- 121 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    clippy::clone_on_copy,  // 1
    clippy::cloned_ref_to_slice_refs,  // 8
    clippy::collapsible_if,  // 1
    clippy::collapsible_match,  // 1
    clippy::disallowed_macros,  // 15
    clippy::doc_lazy_continuation,  // 2
    clippy::double_parens,  // 21
    clippy::explicit_auto_deref,  // 2
    clippy::manual_clamp,  // 1
    clippy::manual_div_ceil,  // 1
    clippy::manual_is_multiple_of,  // 1
    clippy::manual_repeat_n,  // 1
    clippy::missing_const_for_thread_local,  // 1
    clippy::needless_borrow,  // 9
    clippy::needless_borrows_for_generic_args,  // 2
    clippy::only_used_in_recursion,  // 1
    clippy::single_match,  // 1
    clippy::too_many_arguments,  // 1
    clippy::type_complexity,  // 10
    clippy::unnecessary_lazy_evaluations,  // 1
    clippy::unnecessary_to_owned,  // 20
    clippy::useless_format,  // 6
    dead_code,  // 4
    unused_imports,  // 2
    unused_mut,  // 2
    unused_parens,  // 6
)]

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
use crate::v1_rt::RcStr;
use crate::v1_rt::{
    rc_empty_set as empty_set, rc_set_insert as set_insert, rc_set_union as set_union, set_contains,
};
use crate::v1_std_core::{
    arg_name_at, arg_value, arm_body, arm_pattern, authored_name_at, binop_left, binop_right,
    block_stmts, cast_expr, cast_target, expr_call_func_at, expr_field_access_summary,
    expr_method_call_semantics, expr_method_name_at, expr_var_name_at, field_access_base,
    field_access_field_at, field_binding_name_at, field_binding_pattern, field_init_node_name_at,
    field_init_node_value, find_property, find_property_string, foreach_body, foreach_collection,
    foreach_variable_at, if_condition, if_else_branch, if_then_branch, import_is_all,
    import_specific_names_at, index_base, index_expr, is_file_transport, is_rest_transport,
    is_shell_transport, lambda_body, lambda_param_names_at, let_binding_name_at, let_body,
    let_value, match_arm_nodes, match_scrutinee, method_arg_nodes, method_receiver,
    param_node_default_value, param_node_name_at, qualified_last_segment, record_lit_type_name_at,
    return_value, slice_base, slice_end, slice_start, transport_stdin, type_name_compatible,
    unaryop_operand, CallSemantics, Cardinality, Connective, ErrorNode, ExprData, FieldAccessStyle,
    FieldSummary, FieldValueShape, InferredNode, MatchPattern, MethodSemantics, NewlineIndex, Node,
    SourceSpan, StringPart, UnaryOpKind, VarBindingKind,
};

#[path = "bounded_shell_host_drain.rs"]
pub mod bounded_shell_host_drain;

/// A spelling's process-stable identity.
///
/// Symbols used to be per-`InterpContext` integer ordinals.  A `Value` can cross an
/// evaluation-frame boundary (the required floor deliberately creates one frame per claim),
/// so structural equality over records and variants could then compare two unrelated ordinal
/// spaces. Carry one process-canonical spelling: equality and hashing stay pointer-fast on the
/// interpreter's hottest path, while ordering is lexical and therefore deterministic rather
/// than encounter-ordered.
#[derive(Debug, Clone, Copy, PartialOrd, Ord)]
pub struct Symbol(&'static str);

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl Eq for Symbol {}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::ptr::hash(self.0 as *const str, state);
    }
}

#[derive(Debug, Default)]
pub struct SymbolInterner {
    index: HashMap<String, Symbol>,
    calls: u64,
}

const CANONICAL_SYMBOL_RETAINED_SPELLING_BYTE_CAP: usize = 16 * 1024 * 1024;

#[derive(Default)]
struct CanonicalSymbolTable {
    spellings: std::collections::HashSet<&'static str>,
    retained_spelling_bytes: usize,
}

impl CanonicalSymbolTable {
    fn intern_with_cap(
        &mut self,
        spelling: &str,
        spelling_cap_bytes: usize,
    ) -> Result<&'static str, (usize, usize)> {
        if let Some(existing) = self.spellings.get(spelling) {
            return Ok(*existing);
        }
        let projected = self.retained_spelling_bytes.saturating_add(spelling.len());
        if projected > spelling_cap_bytes {
            return Err((projected, spelling_cap_bytes));
        }
        let canonical = Box::leak(spelling.to_string().into_boxed_str());
        self.spellings.insert(canonical);
        self.retained_spelling_bytes = projected;
        Ok(canonical)
    }
}

static CANONICAL_SYMBOLS: std::sync::OnceLock<std::sync::Mutex<CanonicalSymbolTable>> =
    std::sync::OnceLock::new();

fn canonical_symbol_spelling(s: &str) -> &'static str {
    let table =
        CANONICAL_SYMBOLS.get_or_init(|| std::sync::Mutex::new(CanonicalSymbolTable::default()));
    let mut table = table
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    table
        .intern_with_cap(s, CANONICAL_SYMBOL_RETAINED_SPELLING_BYTE_CAP)
        .unwrap_or_else(|(projected, cap)| {
            panic!(
                "canonical symbol retention refused before allocation: projected_spelling_bytes={projected} spelling_cap_bytes={cap}"
            )
        })
}

#[cfg(test)]
mod selected_identity_path_tests {
    use super::{CanonicalSymbolTable, ExecutionMode, InterpContext};
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

    #[test]
    fn canonical_symbol_table_refuses_before_allocation_and_reuses_at_cap() {
        let mut table = CanonicalSymbolTable::default();
        let first = table.intern_with_cap("four", 4).expect("below cap stores");
        assert_eq!(table.spellings.len(), 1);
        assert_eq!(table.retained_spelling_bytes, 4);

        let reused = table
            .intern_with_cap("four", 4)
            .expect("reuse at cap adds no billing");
        assert!(std::ptr::eq(first, reused));
        assert_eq!(table.spellings.len(), 1);
        assert_eq!(table.retained_spelling_bytes, 4);

        assert_eq!(table.intern_with_cap("x", 4), Err((5, 4)));
        assert_eq!(table.spellings.len(), 1);
        assert_eq!(table.retained_spelling_bytes, 4);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InternStats {
    pub calls: u64,
    pub distinct: u64,
    pub hits: u64,
    pub heap_bytes: u64,
    pub canonical_entries: u64,
    pub canonical_retained_spelling_bytes: u64,
    pub canonical_spelling_cap_bytes: u64,
}

impl SymbolInterner {
    pub fn intern(&mut self, s: &str) -> Symbol {
        self.calls += 1;
        if let Some(&symbol) = self.index.get(s) {
            return symbol;
        }
        let symbol = Symbol(canonical_symbol_spelling(s));
        self.index.insert(s.to_string(), symbol);
        symbol
    }

    pub fn stats(&self) -> InternStats {
        let distinct = self.index.len() as u64;
        let canonical = CANONICAL_SYMBOLS
            .get_or_init(|| std::sync::Mutex::new(CanonicalSymbolTable::default()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        InternStats {
            calls: self.calls,
            distinct,
            hits: self.calls.saturating_sub(distinct),
            heap_bytes: self.heap_bytes(),
            canonical_entries: canonical.spellings.len() as u64,
            canonical_retained_spelling_bytes: canonical.retained_spelling_bytes as u64,
            canonical_spelling_cap_bytes: CANONICAL_SYMBOL_RETAINED_SPELLING_BYTE_CAP as u64,
        }
    }

    pub fn resolve(&self, sym: Symbol) -> &str {
        sym.0
    }

    // Read-only counterpart to `intern` (no mutable borrow), for callers reentered while an
    // immutable borrow of this interner is held elsewhere on the stack (`free_monoid_ctx_syms`).
    pub fn get(&self, s: &str) -> Option<Symbol> {
        self.index.get(s).copied()
    }

    fn heap_bytes(&self) -> u64 {
        let mut bytes = (self.index.len() * std::mem::size_of::<(String, Symbol)>()) as u64;
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
        .unwrap_or_else(|| sym.0.to_string())
}

fn coproduct_arm_name_matches(value_name: String, pattern_name: String) -> bool {
    qualified_last_segment(value_name.clone()) == qualified_last_segment(pattern_name)
}

fn coproduct_disj_node(ctx: &InterpContext, item: &Rc<Node>) -> Option<Rc<Node>> {
    if item.connective == Connective::Disj && !item.children.is_empty() {
        return Some(item.clone());
    }
    if let Some(InferredNode::Resolved { node }) = item.inferred.as_deref() {
        if node.connective == Connective::Disj && !node.children.is_empty() {
            return Some(node.clone());
        }
    }
    if let Some(rhs) = type_item_alias_rhs_name(ctx, item) {
        return resolve_coproduct_type_node(ctx, &rhs);
    }
    None
}

fn resolve_coproduct_type_node(ctx: &InterpContext, parent_enum: &str) -> Option<Rc<Node>> {
    let bare = qualified_last_segment(parent_enum.to_string());
    if let Some(item) = lookup_type_item_across_modules(ctx, parent_enum)
        .or_else(|| lookup_type_item_across_modules(ctx, &bare))
    {
        if let Some(disj) = coproduct_disj_node(ctx, &item) {
            return Some(disj);
        }
        return Some(item);
    }
    for module in ctx.modules.iter() {
        let env = module.type_env.clone();
        let node =
            crate::v1_compiler_infer_env::lookup_type_by_name(env.clone(), parent_enum.to_string())
                .or_else(|| crate::v1_compiler_infer_env::lookup_type_by_name(env, bare.clone()))?;
        if let Some(disj) = coproduct_disj_node(ctx, &node) {
            return Some(disj);
        }
        if node.connective == Connective::Disj {
            return Some(node);
        }
    }
    None
}

fn coproduct_parent_spellings_match(
    ctx: &InterpContext,
    value_parent: String,
    pattern_parent: &str,
) -> bool {
    if value_parent == pattern_parent {
        return true;
    }
    if qualified_last_segment(value_parent.clone())
        == qualified_last_segment(pattern_parent.to_string())
    {
        if let (Some(value_coproduct), Some(pattern_coproduct)) = (
            resolve_coproduct_type_node(ctx, &value_parent),
            resolve_coproduct_type_node(ctx, pattern_parent),
        ) {
            if Rc::ptr_eq(&value_coproduct, &pattern_coproduct) {
                return true;
            }
            let value_authored = authored_name_at(ctx.si(), value_coproduct);
            let pattern_authored = authored_name_at(ctx.si(), pattern_coproduct);
            if value_authored == pattern_authored {
                return true;
            }
        }
    }
    let coproduct = resolve_coproduct_type_node(ctx, pattern_parent);
    match coproduct {
        Some(coproduct_node) => {
            let authored = authored_name_at(ctx.si(), coproduct_node.clone());
            authored == value_parent
                || qualified_last_segment(authored) == qualified_last_segment(value_parent.clone())
        }
        None => false,
    }
}

fn variant_arm_is_declared_in_coproduct(
    ctx: &InterpContext,
    variant_name: Symbol,
    pattern_parent: &str,
) -> bool {
    let coproduct = match resolve_coproduct_type_node(ctx, pattern_parent) {
        Some(node) => node,
        None => return false,
    };
    if coproduct.connective != Connective::Disj {
        return false;
    }
    let variant_last = qualified_last_segment(resolve_sym(variant_name));
    for child in coproduct.children.iter() {
        if qualified_last_segment(authored_name_at(ctx.si(), child.clone())) == variant_last {
            return true;
        }
    }
    false
}

fn parent_enum_is(parent: Option<&String>, expected_last: &str) -> bool {
    parent.is_some_and(|p| qualified_last_segment(p.clone()) == expected_last)
}

fn record_pattern_type_name_matches(
    ctx: &InterpContext,
    record_type_name: Symbol,
    pattern_name: &str,
    parent_enum: Option<&String>,
) -> bool {
    let resolved = resolve_sym(record_type_name);
    let name_matches = record_type_name == ctx.sym(pattern_name)
        || resolved == pattern_name
        || type_name_compatible(resolved.clone(), pattern_name.to_string());
    match parent_enum {
        Some(parent) => {
            record_nominal_is_declared_variant_of_coproduct(ctx, resolved.clone(), parent)
                || variant_arm_is_declared_in_coproduct(ctx, record_type_name, parent)
                || name_matches
        }
        None => name_matches,
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
    let record_last = qualified_last_segment(record_nominal.clone());
    for child in coproduct.children.iter() {
        let child_name = authored_name_at(ctx.si(), child.clone());
        if child_name == record_nominal || qualified_last_segment(child_name) == record_last {
            return true;
        }
    }
    false
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
        if key.is_reflexive() {
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
            type_name,
            variant_name,
            fields,
        } => {
            7u8.hash(&mut h);
            type_name.hash(&mut h);
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
    // Field identity is the canonical spelling, never the order in which one evaluation
    // frame encountered it. Keep the logarithmic path for values built through
    // `sorted_fields`, then scan as the total fallback for host-built values whose constructors
    // preserve a modeled field order instead. A binary search alone silently made lookup
    // depend on those two unrelated orders agreeing.
    if let Ok(i) = fields.binary_search_by_key(&sym, |(field, _)| *field) {
        return Some(&fields[i].1);
    }
    fields
        .iter()
        .find(|(field, _)| *field == sym)
        .map(|(_, value)| value)
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
    Str(RcStr),
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
    Value::Str(RcStr::new(Rc::from(s.as_ref())))
}

/// Project an observed child-process status onto `std.process_termination` `ProcessTermination`.
///
/// A signalled process has no exit code, so it gets the signal arm, not a fabricated integer:
/// the seed's `.code().unwrap_or(-1)` made a runner OOM-kill indistinguishable from exit -1.
/// `ProcessTerminationUnobserved` is unreachable from an `ExitStatus` (the process ran); a
/// caller supplies it when the spawn itself refused.
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

/// Whether a `raw_map_lookup` result already carries the `Optional<V>` contract (a
/// `.dag`-authored `Map.lookup` closure returns `Optional<V>` by construction) or is a bare
/// storage read still needing the wrap (native `Value::Map`/field storage, miss = `Value::Null`).
/// Decided by call site, not value shape, so a stored `V = Optional<T>` payload is never
/// mistaken for an already-wrapped result (DESIGN §5: construction, not validation).
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

impl Value {
    /// A value may key a map only if it equals ITSELF, and `Value` has inhabitants that do not:
    /// `Float(f64::NAN)` compares false to itself under IEEE semantics, and the variants this
    /// `PartialEq` does not match (a `Closure`, say) fall to its `false` arm. Admitting either
    /// would make `impl Eq for CanonKey` a lie and its `Hash` unreachable for its own key.
    ///
    /// The test is `self == self`, spelled ONCE and named, because that is the only expression
    /// that stays correct as `PartialEq` above grows arms: a hand-written structural check would
    /// be a second authority for reflexivity and would silently disagree the day a variant is
    /// added. `clippy::eq_op` is deny-by-default and cannot see that a non-reflexive inhabitant
    /// exists, so the lint is refused here, at the one site whose whole content is that test --
    /// not at a crate root on behalf of everything under it.
    #[allow(clippy::eq_op)]
    fn is_reflexive(&self) -> bool {
        self == self
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
                    type_name: at,
                    variant_name: a,
                    fields: af,
                },
                Value::Variant {
                    type_name: bt,
                    variant_name: b,
                    fields: bf,
                },
            ) => at == bt && a == b && af == bf,
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
    /// A pool root contributed NO `.dag` files to a parse-only corpus walk.
    ///
    /// Its own variant, not a `TypeError`: this is DESIGN's empty-observation narrow — a pool
    /// that silently lost its subject looked like one that matched nothing, so every row over it
    /// passed on a smaller population than declared.
    ///
    /// The variant CARRIES the classification (missing / names a file / directory with no `.dag`)
    /// rather than a sentence: the three have different causes and fixes, and a `String` would
    /// re-commit the state-space conflation at the boundary. `Display` derives the message from
    /// the fields — the one direction that cannot lose them.
    PoolRootContributesNothing {
        caller: &'static str,
        declared: usize,
        defects: Vec<(String, crate::coproduct_reflection::PoolRootDefect)>,
    },
    PatternMatchFailure {
        value: String,
    },
    DivisionByZero,
    /// A native `Int` binop's true result does not fit `i64` (`std/integer.dag`'s
    /// `Compose<Int, MachineWidth<64>>` row). Wrapping would answer a different number — the
    /// fabricated-plausible-output failure DESIGN section 5 forbids, the same class
    /// `DivisionByZero` refuses above.
    IntegerOverflow {
        op: &'static str,
        lhs: i64,
        rhs: i64,
    },
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
    /// The caller-agnostic evaluation-budget result the kernel RAISES; the two witness-named
    /// variants below are domain refusals the witness lane maps this into at its boundary, never
    /// produced by `eval_expr` directly.
    ///
    /// Without a neutral result, a served HTTP request (not a witness) raising `EvalBudgetExceeded`
    /// would carry the fast-lane witness ruling and its "relocating the file does not discharge
    /// it" guidance into an HTTP 5xx body — the DESIGN §3 nickname failure applied to a diagnostic
    /// (operator review 2026-08-09). `entry` names which evaluation crossed; `clock` which bound
    /// fired (CPU vs wall is spin vs stall, different remedies); `elapsed_nanos` is nanoseconds,
    /// not floored milliseconds, per `std.measure`'s `nanosecond_millisecond_projection_note`: a
    /// declared limit is policy in milliseconds, an observed crossing is a measurement and is
    /// never floored into the carrier.
    EvaluationBudgetExceeded {
        entry: String,
        clock: EvaluationClock,
        elapsed_nanos: u128,
        limit_ms: u64,
    },
    /// The CPU safety ceiling fired while an admitted cross-claim fill was in flight. The
    /// computation has not published, so it is not a shared artifact; naming the producer keeps
    /// the refusal on the prospective fill instead of silently presenting first-touch order as
    /// intrinsic claim cost.
    FillBudgetExceeded {
        entry: String,
        producer: String,
        fill_cpu_nanos: u128,
        marginal_cpu_nanos: u128,
        limit_ms: u64,
    },
    /// The fast-lane per-witness eval budget, enforced on THREAD CPU by the cooperative
    /// stride-poll in `eval_expr`. The measured field is named for its clock: this and the
    /// wall-clock budget below are different quantities of one occurrence, and a shared
    /// `elapsed_ms` spelling let CPU be read as wall downstream (2026-08-05 — the conflation that
    /// leaves the enforced quantity absent from every cost receipt; `witness_cost_clock_basis_note`).
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
    /// Application-site contract mismatch between the caller's argument list and the callee's
    /// parameters. Typed and located (callee + offending label) so the line stops here rather
    /// than later as `NoSuchVariable` for an unbound parameter — or never, when mismatched names
    /// overlap. DESIGN §5: a failure arm refuses, never widens.
    CallContractMismatch {
        callee: String,
        detail: String,
    },
    /// A HOST EFFECT THAT THE HERMETIC ROUTE HAS NO ARM FOR — a fact about which EXECUTION
    /// ROUTE the caller must supply, never a fact about the caller's verdict.
    ///
    /// Its own variant for the reason `TimedOut`/`HostToolUnresolved` are at the witness
    /// boundary: a route fact recovered by substring-matching prose is one fact in two
    /// representations (DESIGN §2/§3). Previously the three refusal sites below all produced
    /// `TypeError { msg: "hermetic mode: …" }`, so telling "ASSERTED false" from "never given a
    /// runnable route" meant matching the sentence or conflating them; the required floor
    /// conflated them by not executing the population at all.
    ///
    /// `ground` carries WHY, because the remedies differ: an unpublished mock case is closed by
    /// publishing it, a missing `mock_response` by authoring one, a filesystem REMOVAL by a wet
    /// route (removal has no mock arm). One sentence for all three is the state-space
    /// conflation DESIGN's recurring-failure list names.
    HermeticHostEffectRefused {
        operation: String,
        ground: HermeticEffectGround,
    },
}

/// WHY THE HERMETIC ROUTE HAS NO ARM FOR ONE OPERATION. Closed, and each arm names a
/// different remedy — see `InterpError::HermeticHostEffectRefused`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HermeticEffectGround {
    /// The service is corpus-governed and no published mock case names this operation.
    /// `published_cases` carries the cases that DO exist for the service, so the refusal
    /// states its own remedy rather than sending the reader to look for it.
    UnpublishedMockCase { published_cases: Vec<String> },
    /// The operation node carries no `mock_response` property, so the hermetic arm would
    /// have to fabricate a Unit — the fabricated-plausible-output failure DESIGN §5 forbids.
    NoMockResponse,
    /// A filesystem REMOVAL. Distinct from the two above because there is no mock arm to
    /// author: the operation's whole content is the effect, so only a wet route can run it.
    FilesystemRemoval,
}

impl fmt::Display for InterpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InterpError::NoSuchFunction { name } => write!(
                f,
                "no declaration named '{}' in this execution's loaded index \
                 (searched the modules loaded for this run; the name may be \
                 declared in a module this run did not load)",
                name
            ),
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
            InterpError::FillBudgetExceeded {
                entry,
                producer,
                fill_cpu_nanos,
                marginal_cpu_nanos,
                limit_ms,
            } => write!(
                f,
                "in-flight shared fill exceeded CPU safety budget: entry={} producer={} \
                 fill_cpu_ns={} marginal_cpu_ns={} limit_ms={}",
                entry, producer, fill_cpu_nanos, marginal_cpu_nanos, limit_ms
            ),
            InterpError::EvalBudgetExceeded {
                cpu_ms: elapsed_ms,
                budget_ms,
            } => {
                write!(
                    f,
                    "eval budget exceeded: {}ms thread-CPU > {}ms fast-lane budget (operator ruling 2026-08-17, superseding the 5s rule of 2026-07-12; in the required-floor claim loop the ceiling is required_floor_claim_cpu_safety_limit_ms, an independent deadline from required_floor_claim_wall_safety_limit_ms per the 2026-08-19 budget policy cut's superseding correction — CPU and wall are never one scalar copied into both clocks). This budget is enforced on THREAD CPU, not wall. RELOCATING THE FILE DOES NOT DISCHARGE IT: moving a witness under a long/ dir removes it from per-PR discovery without giving it an executing consumer, which deletes the coverage while retaining the source (the gunbc#7762 specimen behind the 2026-08-04 admission ruling). Either reduce the witness's cost, or enroll it in a lane that declares its own dated ceiling AND names the row as an executing consumer.",
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
            InterpError::PoolRootContributesNothing {
                caller,
                declared,
                defects,
            } => {
                write!(
                    f,
                    "pool root contributes nothing: {}",
                    crate::coproduct_reflection::pool_root_refusal_message(
                        defects, *declared, caller
                    )
                )
            }
            InterpError::PatternMatchFailure { value } => {
                write!(f, "non-exhaustive pattern match on: {}", value)
            }
            InterpError::DivisionByZero => write!(f, "division by zero"),
            InterpError::IntegerOverflow { op, lhs, rhs } => write!(
                f,
                "integer overflow: {} {} {} does not fit in a 64-bit Int",
                lhs, op, rhs
            ),
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
            InterpError::HermeticHostEffectRefused { operation, ground } => match ground {
                HermeticEffectGround::UnpublishedMockCase { published_cases } => write!(
                    f,
                    "hermetic mode: operation {operation} is not a published mock case for its \
                     corpus-governed service \u{2014} refusing to realize (published cases: \
                     {published_cases:?})"
                ),
                HermeticEffectGround::NoMockResponse => write!(
                    f,
                    "hermetic mode: no mock_response for operation {operation} \u{2014} refusing \
                     to fabricate Unit"
                ),
                HermeticEffectGround::FilesystemRemoval => write!(
                    f,
                    "hermetic mode: {operation} refuses filesystem removal (no mock arm; the \
                     operation's whole content is the effect, so only a wet route can run it)"
                ),
            },
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

/// Origin-free mirror of `Value`, used as the cross-claim memo's storage shape.
///
/// Required-floor builds a FRESH `InterpContext` per claim by design (`cli_run.rs`
/// `evaluation_frame`, "FRESH PER CLAIM"), while this memo survives across claims for one
/// prepared-subject run. Before `Symbol` identity became spelling-backed, caching a raw `Value`
/// here leaked producer-frame ordinals into a consumer frame (gunbc#8505). Symbols themselves
/// are now frame-independent, but this positive portable shape remains the store's authority
/// for total publication, structural collision verification, byte accounting, and exclusion of
/// origin-bound closures/functions.
///
/// The fix is the boundary translation the Realization pattern (DESIGN.md §4) already
/// prescribes: reify to `PortableValue` at store time while the producing ctx is still alive.
/// Process-canonical `Symbol`s are themselves frame-independent, so the portable shape carries
/// them directly and a served value does not walk a second global-interner path. `Closure`/`Fn`
/// are not portable this way (their `Env` chain isn't a content snapshot) and are refused rather
/// than guessed at — `prepare_grammar` never returns them.
#[derive(Debug, Clone)]
enum PortableValue {
    Null,
    Unit,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(Rc<str>),
    List(Vec<PortableValue>),
    Map(Vec<(PortableValue, PortableValue)>),
    Set(Rc<OrdSet<String>>),
    Record {
        type_name: Symbol,
        fields: Vec<(Symbol, PortableValue)>,
    },
    Variant {
        type_name: Symbol,
        variant_name: Symbol,
        fields: Vec<(Symbol, PortableValue)>,
    },
}

/// Structural equality over portable values — the cross-claim tier's verification relation.
/// `Fn` compares by shared-graph identity (the same relation the memo key uses); floats by
/// bits, so a NaN-carrying argument row still verifies against itself.
fn portable_value_eq(a: &PortableValue, b: &PortableValue) -> bool {
    match (a, b) {
        (PortableValue::Null, PortableValue::Null) | (PortableValue::Unit, PortableValue::Unit) => {
            true
        }
        (PortableValue::Bool(x), PortableValue::Bool(y)) => x == y,
        (PortableValue::Int(x), PortableValue::Int(y)) => x == y,
        (PortableValue::Float(x), PortableValue::Float(y)) => x.to_bits() == y.to_bits(),
        (PortableValue::Str(x), PortableValue::Str(y)) => x == y,
        (PortableValue::List(xs), PortableValue::List(ys)) => {
            xs.len() == ys.len() && xs.iter().zip(ys).all(|(x, y)| portable_value_eq(x, y))
        }
        (PortableValue::Map(xs), PortableValue::Map(ys)) => {
            xs.len() == ys.len()
                && xs.iter().zip(ys).all(|((xk, xv), (yk, yv))| {
                    portable_value_eq(xk, yk) && portable_value_eq(xv, yv)
                })
        }
        (PortableValue::Set(x), PortableValue::Set(y)) => x == y,
        (
            PortableValue::Record {
                type_name: tx,
                fields: fx,
            },
            PortableValue::Record {
                type_name: ty,
                fields: fy,
            },
        ) => {
            tx == ty
                && fx.len() == fy.len()
                && fx
                    .iter()
                    .zip(fy)
                    .all(|((kx, vx), (ky, vy))| kx == ky && portable_value_eq(vx, vy))
        }
        (
            PortableValue::Variant {
                type_name: tx,
                variant_name: vx,
                fields: fx,
            },
            PortableValue::Variant {
                type_name: ty,
                variant_name: vy,
                fields: fy,
            },
        ) => {
            tx == ty
                && vx == vy
                && fx.len() == fy.len()
                && fx
                    .iter()
                    .zip(fy)
                    .all(|((kx, vxv), (ky, vyv))| kx == ky && portable_value_eq(vxv, vyv))
        }
        _ => false,
    }
}

/// The full argument row in portable form, or `None` when any argument is not portable —
/// such a call can be neither verified nor stored.
fn portable_args_from_ctx(
    ctx: &InterpContext,
    args: &[(Option<String>, Value)],
) -> Option<Vec<(Option<String>, PortableValue)>> {
    let mut out = Vec::with_capacity(args.len());
    for (name, value) in args {
        out.push((name.clone(), portable_value_from_ctx(ctx, value)?));
    }
    Some(out)
}

/// A typed reification refusal: the first non-portable child, named by its path into the
/// value and the kind encountered. Publication into the cross-claim store is TOTAL — a value
/// is either fully reified or refused HERE, never discovered unportable at retrieval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServeCacheValueNotPortable {
    pub path_into_value: String,
    pub encountered_kind: &'static str,
}

fn portable_value_from_ctx_at(
    ctx: &InterpContext,
    value: &Value,
    path: &mut String,
) -> Result<PortableValue, ServeCacheValueNotPortable> {
    macro_rules! descend {
        ($seg:expr, $inner:expr) => {{
            let len_before = path.len();
            path.push_str(&$seg);
            let out = portable_value_from_ctx_at(ctx, $inner, path);
            path.truncate(len_before);
            out?
        }};
    }
    Ok(match value {
        Value::Null => PortableValue::Null,
        Value::Unit => PortableValue::Unit,
        Value::Bool(b) => PortableValue::Bool(*b),
        Value::Int(i) => PortableValue::Int(*i),
        Value::Float(f) => PortableValue::Float(*f),
        Value::Str(s) => PortableValue::Str(s.rc()),
        Value::List(items) => {
            let mut out = Vec::with_capacity(items.len());
            for (i, item) in items.iter().enumerate() {
                out.push(descend!(format!("[{i}]"), item));
            }
            PortableValue::List(out)
        }
        Value::Map(m) => {
            let mut out = Vec::with_capacity(m.len());
            for (k, v) in m.iter() {
                out.push((
                    descend!(".<map-key>".to_string(), &k.key),
                    descend!(".<map-value>".to_string(), v),
                ));
            }
            PortableValue::Map(out)
        }
        Value::Set(members) => PortableValue::Set(members.clone()),
        Value::Record { type_name, fields } => {
            let mut out = Vec::with_capacity(fields.len());
            for (k, v) in fields.iter() {
                let name = ctx.resolve(*k);
                out.push((*k, descend!(format!(".{name}"), v)));
            }
            PortableValue::Record {
                type_name: *type_name,
                fields: out,
            }
        }
        Value::Variant {
            type_name,
            variant_name,
            fields,
        } => {
            let mut out = Vec::with_capacity(fields.len());
            for (k, v) in fields.iter() {
                let name = ctx.resolve(*k);
                out.push((*k, descend!(format!(".{name}"), v)));
            }
            PortableValue::Variant {
                type_name: *type_name,
                variant_name: *variant_name,
                fields: out,
            }
        }
        Value::Closure { .. } => {
            return Err(ServeCacheValueNotPortable {
                path_into_value: path.clone(),
                encountered_kind: "Closure",
            })
        }
        Value::Fn { .. } => {
            return Err(ServeCacheValueNotPortable {
                path_into_value: path.clone(),
                encountered_kind: "OriginBoundNode",
            })
        }
    })
}

fn portable_value_from_ctx(ctx: &InterpContext, value: &Value) -> Option<PortableValue> {
    portable_value_from_ctx_at(ctx, value, &mut String::new()).ok()
}

fn value_from_portable_ctx(ctx: &InterpContext, portable: &PortableValue) -> Value {
    match portable {
        PortableValue::Null => Value::Null,
        PortableValue::Unit => Value::Unit,
        PortableValue::Bool(b) => Value::Bool(*b),
        PortableValue::Int(i) => Value::Int(*i),
        PortableValue::Float(f) => Value::Float(*f),
        PortableValue::Str(s) => Value::Str(RcStr::new(Rc::clone(s))),
        PortableValue::List(items) => list_value(
            items
                .iter()
                .map(|p| value_from_portable_ctx(ctx, p))
                .collect::<Vec<_>>(),
        ),
        PortableValue::Map(entries) => {
            let fields: HamtMap<CanonKey, Value> = entries
                .iter()
                .filter_map(|(k, v)| {
                    CanonKey::new(value_from_portable_ctx(ctx, k))
                        .map(|ck| (ck, value_from_portable_ctx(ctx, v)))
                })
                .collect();
            map_value(fields)
        }
        PortableValue::Set(members) => Value::Set(members.clone()),
        // Re-sort after reconstruction so the representation invariant stays local to this
        // constructor. Symbols are already process-canonical; no consuming-frame re-interning
        // is needed. Retaining the sort keeps `fields_get`'s binary-search contract explicit.
        PortableValue::Record { type_name, fields } => Value::Record {
            type_name: *type_name,
            fields: Rc::new(sorted_fields(
                fields
                    .iter()
                    .map(|(k, v)| (*k, value_from_portable_ctx(ctx, v)))
                    .collect(),
            )),
        },
        PortableValue::Variant {
            type_name,
            variant_name,
            fields,
        } => Value::Variant {
            type_name: *type_name,
            variant_name: *variant_name,
            // Sorted for the same reason as the Record arm above.
            fields: Rc::new(sorted_fields(
                fields
                    .iter()
                    .map(|(k, v)| (*k, value_from_portable_ctx(ctx, v)))
                    .collect(),
            )),
        },
    }
}

thread_local! {
    /// The most recent typed store refusal, retained for the warm path's line-stop message.
    static CROSS_CLAIM_LAST_UNPORTABLE: RefCell<Option<(String, ServeCacheValueNotPortable)>> =
        const { RefCell::new(None) };
}

/// The most recent typed store refusal `(producer bare name, refusal)`, taken (cleared).
pub fn cross_claim_take_last_unportable() -> Option<(String, ServeCacheValueNotPortable)> {
    CROSS_CLAIM_LAST_UNPORTABLE.with(|c| c.borrow_mut().take())
}

#[derive(Default)]
struct CrossClaimPureMemo {
    /// Hash-bucketed like the eval-frame memo, served ONLY after the stored argument row
    /// verifies structurally equal in portable (interner-free) form — a collision degrades to
    /// recompute, never a wrong value. Served-on-hash-alone, main's floor produced six emit
    /// failures whose values belonged to OTHER calls of the same producer (no field
    /// 'produced_decl_support' on type 'TargetModel', run 33269961629).
    /// THE RETAINED VALUE IS THE REIFIED `Value`, NOT THE PORTABLE FORM, AND THAT IS WHAT
    /// MAKES A SERVE ZERO-WALK. Publication still reifies to `PortableValue` first — that walk
    /// is the TOTAL portability check and the byte-budget measurement — but the portable form
    /// is then converted ONCE, here, and every later claim is handed an `Rc` clone. `Value`'s
    /// containers are all `Rc`-backed and `Symbol` is a process-canonical `&'static str`, so a
    /// value reconstructed from a portable form carries nothing frame-bound: field order sorts
    /// on process-global pointer identity, and equality no longer depends on which frame
    /// interned a spelling. Serving the stored `Value` is therefore the SAME value the walk
    /// used to rebuild per consuming frame, at O(1) instead of O(size).
    map: HashMap<(usize, u64), Vec<(Vec<(Option<String>, PortableValue)>, Value)>>,
    /// Stores refused at `CROSS_CLAIM_PURE_MEMO_ENTRY_CAP` or because the entry would push
    /// `bytes` past `CROSS_CLAIM_PURE_MEMO_BYTE_BUDGET`. Counted, never silent: the producer
    /// recomputes, and the receipt reads the count so saturation is visible, not inferred
    /// from missing hits.
    overflow: u64,
    /// Estimated retained bytes over every stored entry (argument rows AND values, collision
    /// buckets included), measured on the `PortableValue` at publication — reification is
    /// total, so the size is known before the store lands. It is measured on the portable form
    /// and CHARGED for the `Value` retained in its place: the two carry the same reachable
    /// content (the same `Rc<str>` spellings, the same numbers, the same field count), so the
    /// portable walk is a sound estimator of what the retained value holds. The ACTUAL byte bound review
    /// 57446's F2 demanded: the entry cap bounded bucket count while each value was unbounded.
    bytes: usize,
    /// Stores refused because the value failed TOTAL reification (`ServeCacheValueNotPortable`).
    /// Counted, and the most recent refusal is retained for the warm path's diagnostics.
    unportable_refusals: u64,
}

/// Entry-count admission for the cross-claim tier: distinct (fn, args) keys stop being STORED
/// past this. Enrolled producers are roster-declared and mostly nullary (tens of keys); the cap
/// guards a mis-enrolled parametric producer with a claim-shaped argument space. Bounds
/// POPULATION, not RETENTION — the byte budget below bounds retention.
const CROSS_CLAIM_PURE_MEMO_ENTRY_CAP: usize = 4096;

/// Byte budget on the tier's RETAINED representation, enforced at publication against the
/// estimated reified portable entry (arguments + value). The store lives for a whole prepared
/// floor run (harm class: the 2026-07-10 20GiB ctx-lifetime regression), so the bound is bytes,
/// not entries — one mis-enrolled producer returning whole-corpus values exhausts memory under
/// any entry count. 256 MiB is ~25x observed retention (run 33273530722: 320 fills of
/// target-model/grammar values) and well under the session container's headroom; an
/// over-budget store refuses counted (`overflow`) and the producer recomputes per claim as if
/// never enrolled.
const CROSS_CLAIM_PURE_MEMO_BYTE_BUDGET: usize = 256 * 1024 * 1024;

#[cfg(test)]
thread_local! {
    /// Test-only budget override: the production budget is far above any value a unit test
    /// should allocate, so the budget RED shrinks the line instead of the fixture. Read
    /// only through `cross_claim_byte_budget`, never by production code directly.
    static CROSS_CLAIM_BYTE_BUDGET_TEST_OVERRIDE: std::cell::Cell<Option<usize>> =
        const { std::cell::Cell::new(None) };
}

fn cross_claim_byte_budget() -> usize {
    #[cfg(test)]
    if let Some(b) = CROSS_CLAIM_BYTE_BUDGET_TEST_OVERRIDE.with(|c| c.get()) {
        return b;
    }
    CROSS_CLAIM_PURE_MEMO_BYTE_BUDGET
}

/// Estimated retained size of one portable value: enum footprint plus every reachable owned
/// heap allocation. Allocator overhead and `Vec` spare capacity are not modeled, but it walks
/// the same total reification the store publishes, so unlike a shallow `size_of` it cannot
/// miss a child.
fn portable_value_size_bytes(v: &PortableValue) -> usize {
    use std::mem::size_of;
    size_of::<PortableValue>()
        + match v {
            PortableValue::Null
            | PortableValue::Unit
            | PortableValue::Bool(_)
            | PortableValue::Int(_)
            | PortableValue::Float(_) => 0,
            PortableValue::Str(s) => s.len(),
            PortableValue::List(items) => {
                items.iter().map(portable_value_size_bytes).sum::<usize>()
            }
            PortableValue::Map(pairs) => pairs
                .iter()
                .map(|(k, v)| portable_value_size_bytes(k) + portable_value_size_bytes(v))
                .sum(),
            PortableValue::Set(s) => s.iter().map(|x| x.len() + size_of::<String>()).sum(),
            // Symbols point into the process-wide canonical table. Their pointer-sized slots
            // are already included in the enum/Vec allocation; no spelling allocation is
            // retained per portable entry.
            PortableValue::Record { fields, .. } => fields
                .iter()
                .map(|(_, f)| portable_value_size_bytes(f))
                .sum::<usize>(),
            PortableValue::Variant { fields, .. } => fields
                .iter()
                .map(|(_, f)| portable_value_size_bytes(f))
                .sum::<usize>(),
        }
}

/// Observes cross-claim memo traffic for the floor's shared-fill ledger. Installed by
/// required-floor preparation; `None` outside it. The interpreter nets fill CPU via
/// `record_shared_artifact_fill_cpu_nanos`; the observer carries the ledger rows and wall
/// clock, which live in `cli_run`.
pub struct CrossClaimShareObserver {
    /// Called when an admitted call MAY become a fill — before it computes. Paired with
    /// exactly one `on_fill` or `on_fill_abandon`, so a nesting ledger on the other side
    /// (the floor's shared-fill child stack) stays balanced.
    pub on_fill_begin: Box<dyn Fn()>,
    /// (producer name, inclusive fill wall nanos, SELF fill wall nanos — inclusive minus
    /// stored descendants) — called once per store that landed. CPU is netted inside the
    /// interpreter (`record_shared_artifact_fill_cpu_nanos`) and appears nowhere here.
    pub on_fill: Box<dyn Fn(&str, u128, u128)>,
    /// The admitted call computed but did not store (effects, unportable value, cap,
    /// duplicate key) — closes the `on_fill_begin`.
    pub on_fill_abandon: Box<dyn Fn()>,
    /// producer name — called once per served hit.
    pub on_hit: Box<dyn Fn(&str)>,
}

thread_local! {
    static CROSS_CLAIM_PURE_MEMO: RefCell<CrossClaimPureMemo> =
        RefCell::new(CrossClaimPureMemo::default());
    static CROSS_CLAIM_FN_KEEPALIVE: RefCell<Vec<Rc<Node>>> = RefCell::new(Vec::new());
    /// Fn-NODE identities admitted to the cross-claim tier beyond the built-in
    /// `prepare_grammar` arm. Admission is by RESOLVED DECLARATION IDENTITY: qualified roster
    /// spellings resolve to fn nodes at install, so a bare-name homonym in a non-rostered
    /// module is never eligible — review 57446's F1 (name-set admission cached any same-named
    /// fn in the subject). Nodes are kept alive by `CROSS_CLAIM_FN_KEEPALIVE` at install.
    static CROSS_CLAIM_PURE_ROSTER: RefCell<std::collections::HashSet<usize>> =
        RefCell::new(std::collections::HashSet::new());
    static CROSS_CLAIM_SHARE_OBSERVER: RefCell<Option<CrossClaimShareObserver>> =
        const { RefCell::new(None) };
}

/// Clears stored values, roster and observer together: the tier's lifetime is ONE prepared
/// execution frame (called from register/clear of the floor's prepared authority), and a
/// roster outliving its subject would let a later, differently-prepared evaluation store
/// under it.
pub fn clear_cross_claim_pure_memos() {
    CROSS_CLAIM_PURE_MEMO.with(|m| *m.borrow_mut() = CrossClaimPureMemo::default());
    CROSS_CLAIM_FN_KEEPALIVE.with(|k| k.borrow_mut().clear());
    CROSS_CLAIM_PURE_ROSTER.with(|r| r.borrow_mut().clear());
    CROSS_CLAIM_SHARE_OBSERVER.with(|o| *o.borrow_mut() = None);
    // The retained refusal is tier state too: leaving it across a reset is how a stale path
    // outlives the store that produced it (review 57554).
    CROSS_CLAIM_LAST_UNPORTABLE.with(|c| *c.borrow_mut() = None);
}

/// Install the declared producer roster as RESOLVED fn nodes (the caller resolves each
/// qualified spelling in a frame over the prepared subject). Replaces any previous roster and
/// keeps the nodes alive for the tier's lifetime; the built-in `prepare_grammar` arm stays
/// admitted, so roster-less surfaces (claim_batch, `gunbc run`) are unchanged.
pub fn install_cross_claim_pure_share_roster<I: IntoIterator<Item = Rc<Node>>>(nodes: I) {
    let nodes: Vec<Rc<Node>> = nodes.into_iter().collect();
    CROSS_CLAIM_PURE_ROSTER.with(|r| {
        *r.borrow_mut() = nodes.iter().map(|n| Rc::as_ptr(n) as usize).collect();
    });
    for node in &nodes {
        keep_cross_claim_fn(node);
    }
}

/// Install the shared-fill observer for the cross-claim tier. `None` uninstalls.
pub fn install_cross_claim_share_observer(observer: Option<CrossClaimShareObserver>) {
    CROSS_CLAIM_SHARE_OBSERVER.with(|o| *o.borrow_mut() = observer);
}

/// (stores, overflow) for the cross-claim tier on this thread — receipt fodder only.
pub fn cross_claim_pure_memo_counts() -> (usize, u64) {
    CROSS_CLAIM_PURE_MEMO.with(|m| {
        let m = m.borrow();
        (m.map.len(), m.overflow)
    })
}

fn cross_claim_portable_args_match(
    stored: &[(Option<String>, PortableValue)],
    args: &[(Option<String>, PortableValue)],
) -> bool {
    stored.len() == args.len()
        && stored
            .iter()
            .zip(args.iter())
            .all(|((sn, sv), (an, av))| sn == an && portable_value_eq(sv, av))
}

fn cross_claim_pure_admitted(fn_node: &Rc<Node>, func_name: &str) -> bool {
    func_name == "prepare_grammar"
        || CROSS_CLAIM_PURE_ROSTER.with(|r| r.borrow().contains(&(Rc::as_ptr(fn_node) as usize)))
}

fn cross_claim_observe_hit(func_name: &str) {
    CROSS_CLAIM_SHARE_OBSERVER.with(|o| {
        if let Some(obs) = o.borrow().as_ref() {
            (obs.on_hit)(func_name);
        }
    });
}

thread_local! {
    /// One accumulator per admitted fill in flight, innermost last, holding the inclusive CPU
    /// of STORED fills completed inside it. Enrolled producers compose (a full target model
    /// reads its staging core), and netting outer and inner inclusive CPU from the paying claim
    /// would net the inner twice — the shared-fill ledger's child stack, at this tier's grain.
    static CROSS_CLAIM_FILL_FRAMES: RefCell<Vec<CrossClaimFillFrame>> = const { RefCell::new(Vec::new()) };
}

struct CrossClaimFillFrame {
    producer: String,
    cpu_started: u128,
    steps_started: u64,
    stored_children_cpu: u128,
    stored_children_wall: u128,
    stored_children_steps: u64,
}

/// RAII scope for one admitted call that missed the cross-claim memo. Every path out of the
/// call — store, non-store, error — runs `Drop`, which keeps the child stack and the
/// observer's begin/record/abandon protocol balanced by construction.
struct CrossClaimFillGuard {
    func_name: String,
    cpu_started: u128,
    steps_started: u64,
    wall_started: Instant,
    stored: std::cell::Cell<bool>,
}

impl CrossClaimFillGuard {
    fn enter(func_name: &str) -> CrossClaimFillGuard {
        let cpu_started = thread_cpu_nanos();
        let steps_started = evaluator_steps();
        CROSS_CLAIM_FILL_FRAMES.with(|s| {
            s.borrow_mut().push(CrossClaimFillFrame {
                producer: func_name.to_string(),
                cpu_started,
                steps_started,
                stored_children_cpu: 0,
                stored_children_wall: 0,
                stored_children_steps: 0,
            })
        });
        CROSS_CLAIM_SHARE_OBSERVER.with(|o| {
            if let Some(obs) = o.borrow().as_ref() {
                (obs.on_fill_begin)();
            }
        });
        CrossClaimFillGuard {
            func_name: func_name.to_string(),
            cpu_started,
            steps_started,
            wall_started: Instant::now(),
            stored: std::cell::Cell::new(false),
        }
    }

    fn mark_stored(&self) {
        self.stored.set(true);
    }
}

impl Drop for CrossClaimFillGuard {
    fn drop(&mut self) {
        let inclusive_cpu = thread_cpu_nanos().saturating_sub(self.cpu_started);
        let inclusive_wall = self.wall_started.elapsed().as_nanos();
        let inclusive_steps = evaluator_steps().wrapping_sub(self.steps_started);
        let frame = CROSS_CLAIM_FILL_FRAMES
            .with(|s| s.borrow_mut().pop())
            .unwrap_or(CrossClaimFillFrame {
                producer: self.func_name.clone(),
                cpu_started: self.cpu_started,
                steps_started: self.steps_started,
                stored_children_cpu: 0,
                stored_children_wall: 0,
                stored_children_steps: 0,
            });
        let children_cpu = frame.stored_children_cpu;
        let children_wall = frame.stored_children_wall;
        let children_steps = frame.stored_children_steps;
        if self.stored.get() {
            // Net SELF time only: stored descendants already netted their own inclusive time.
            record_shared_artifact_fill_cpu_nanos(inclusive_cpu.saturating_sub(children_cpu));
            record_shared_artifact_fill_eval_steps(inclusive_steps.saturating_sub(children_steps));
            let self_wall = inclusive_wall.saturating_sub(children_wall);
            CROSS_CLAIM_FILL_FRAMES.with(|s| {
                if let Some(parent) = s.borrow_mut().last_mut() {
                    parent.stored_children_cpu =
                        parent.stored_children_cpu.saturating_add(inclusive_cpu);
                    parent.stored_children_wall =
                        parent.stored_children_wall.saturating_add(inclusive_wall);
                    parent.stored_children_steps =
                        parent.stored_children_steps.saturating_add(inclusive_steps);
                }
            });
            CROSS_CLAIM_SHARE_OBSERVER.with(|o| {
                if let Some(obs) = o.borrow().as_ref() {
                    (obs.on_fill)(&self.func_name, inclusive_wall, self_wall);
                }
            });
        } else {
            // Not stored: this frame's own cost stays the caller's, but its STORED
            // descendants were netted, so their total rolls up for the parent to subtract.
            CROSS_CLAIM_FILL_FRAMES.with(|s| {
                if let Some(parent) = s.borrow_mut().last_mut() {
                    parent.stored_children_cpu =
                        parent.stored_children_cpu.saturating_add(children_cpu);
                    parent.stored_children_wall =
                        parent.stored_children_wall.saturating_add(children_wall);
                    parent.stored_children_steps =
                        parent.stored_children_steps.saturating_add(children_steps);
                }
            });
            CROSS_CLAIM_SHARE_OBSERVER.with(|o| {
                if let Some(obs) = o.borrow().as_ref() {
                    (obs.on_fill_abandon)();
                }
            });
        }
    }
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

/// One content hash over ALL of a call's arguments (names and values). `None` when any
/// argument is not content-hashable — such a call is never admitted to the cross-claim tier.
/// The interner/hash-memo borrows stay local to this function. Portable reconstruction now
/// carries process-canonical symbols directly, but keeping these borrows narrow also keeps the
/// hashing boundary explicit and independent from the caller's cache mutations.
fn cross_claim_args_hash(ctx: &InterpContext, args: &[(Option<String>, Value)]) -> Option<u64> {
    use std::hash::{Hash, Hasher};
    let mut hash_memo = ctx.eval_recompute_hash_memo.borrow_mut();
    let interner = ctx.symbols.borrow();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    args.len().hash(&mut hasher);
    for (name, value) in args {
        name.hash(&mut hasher);
        eval_recompute_arg_key(&mut hash_memo, &interner, value)?.hash(&mut hasher);
    }
    Some(hasher.finish())
}

// 🟡 dissolve-on (narrowed, not discharged): gunbc.roadmap_authority
// five_minute_ci_gate_program_note — this tier is the generic cross-claim pure memo that note
// asked for, keyed on fn-node identity + content-hashed args, admission held to a DECLARED
// roster (`v2.workflow.floor_pure_producer_share`) plus the built-in `prepare_grammar` arm,
// not every pure call: cross-claim retention is byte-unbounded by construction (the
// 2026-07-10 20GiB ctx-lifetime regression), so admission stays a bounded row. Widening
// beyond the roster is that note's remaining work, not this arm's.
fn try_cross_claim_pure_memo(
    ctx: &InterpContext,
    fn_node: &Rc<Node>,
    func_name: &str,
    args: &[(Option<String>, Value)],
) -> Option<Value> {
    if !cross_claim_pure_admitted(fn_node, func_name) {
        return None;
    }
    let args_hash = cross_claim_args_hash(ctx, args)?;
    let memo_key = (Rc::as_ptr(fn_node) as usize, args_hash);
    // The per-ctx hit cache is verified the same way the global bucket is: hash first, then
    // the full portable argument row, so an intra-frame hash collision cannot alias either.
    let portable_args = portable_args_from_ctx(ctx, args)?;
    // A SERVE IS AN `Rc` CLONE. There is no per-frame reconstruction left to amortize, so the
    // per-context hit cache this path used to maintain is gone with the walk it existed to
    // avoid — the DESIGN section 4b(4) dissolution: a climb deletes the lower-rung production
    // machinery it obsoletes.
    let value = CROSS_CLAIM_PURE_MEMO.with(|m| {
        m.borrow().map.get(&memo_key).and_then(|bucket| {
            bucket.iter().find_map(|(stored_args, stored)| {
                cross_claim_portable_args_match(stored_args, &portable_args).then(|| stored.clone())
            })
        })
    })?;
    cross_claim_observe_hit(func_name);
    Some(value)
}

/// Store a just-computed admitted call. Fill cost is recorded (netted from the paying claim,
/// reported to the ledger) by the guard's `Drop`, only when the store lands — an
/// overflow-refused store leaves the cost the caller's, since every later caller recomputes.
/// What one cross-claim publication attempt did. A BOOLEAN CONFLATED TWO DIFFERENT FACTS
/// (#9721): `AlreadyPresent` — the value is in the tier under this exact key with a
/// structurally equal argument row — SATISFIES the warm's obligation ("later claims can serve
/// this", not "this call put it there"); a rostered producer reachable from an earlier one is
/// stored by that traversal, so its own warm finds its work done. The `Refused*` arms are the
/// opposite: nothing is servable, a warm hitting one would relocate its fill onto the first
/// toucher and must stop the line. Reporting both as `stored=false` made the floor refuse a
/// correctly populated tier and print "duplicate key, entry cap, or byte budget" for the cause
/// — the `diagnostic_name_mechanism_silent` failure mode, named in DESIGN.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossClaimStoreOutcome {
    /// This call published the entry.
    Stored,
    /// An entry for this key with a structurally equal argument row was already retained.
    /// Servable, and therefore not a refusal.
    AlreadyPresent,
    /// The producer is not on the installed roster (or its identity does not match).
    NotAdmitted,
    /// The argument row is not content-hashable, so no key exists to store under.
    RefusedArgsNotHashable,
    /// The argument row failed TOTAL reification.
    RefusedArgsNotPortable,
    /// The VALUE failed TOTAL reification. THE LOCATED REFUSAL IS CARRIED ON THE VARIANT, not
    /// a side channel: review 57554 found that reading `CROSS_CLAIM_LAST_UNPORTABLE` for EVERY
    /// non-servable outcome decorated a byte-budget or entry-cap refusal with a stale path/kind
    /// from an earlier producer (only this arm writes the slot, nothing clears it) — one cause
    /// carrying another's evidence, the fabrication this type removes. Binding the detail to
    /// its arm makes the mismatch UNCONSTRUCTIBLE, not guarded (DESIGN section 5).
    RefusedValueNotPortable(ServeCacheValueNotPortable),
    /// The key population is at `CROSS_CLAIM_PURE_MEMO_ENTRY_CAP`.
    RefusedEntryCap,
    /// Landing the entry would push retention past `CROSS_CLAIM_PURE_MEMO_BYTE_BUDGET`.
    RefusedByteBudget,
}

impl CrossClaimStoreOutcome {
    /// Whether the value is retained and servable to later claims after this attempt —
    /// true for a fresh store AND for one that found its entry already present.
    pub fn is_servable(&self) -> bool {
        matches!(
            self,
            CrossClaimStoreOutcome::Stored | CrossClaimStoreOutcome::AlreadyPresent
        )
    }

    /// The located detail for the ONE arm that has any, so a caller cannot pair a cause with
    /// another cause's evidence.
    pub fn not_portable_detail(&self) -> Option<&ServeCacheValueNotPortable> {
        match self {
            CrossClaimStoreOutcome::RefusedValueNotPortable(refusal) => Some(refusal),
            _ => None,
        }
    }

    /// The cause, for a refusal's located line-stop message.
    pub fn cause(&self) -> &'static str {
        match self {
            CrossClaimStoreOutcome::Stored => "Stored",
            CrossClaimStoreOutcome::AlreadyPresent => "AlreadyPresent",
            CrossClaimStoreOutcome::NotAdmitted => "NotAdmitted",
            CrossClaimStoreOutcome::RefusedArgsNotHashable => "ArgumentRowNotHashable",
            CrossClaimStoreOutcome::RefusedArgsNotPortable => "ArgumentRowNotPortable",
            CrossClaimStoreOutcome::RefusedValueNotPortable(_) => "ServeCacheValueNotPortable",
            CrossClaimStoreOutcome::RefusedEntryCap => "EntryCapReached",
            CrossClaimStoreOutcome::RefusedByteBudget => "ByteBudgetExceeded",
        }
    }
}

fn store_cross_claim_pure_memo(
    ctx: &InterpContext,
    fn_node: &Rc<Node>,
    func_name: &str,
    args: &[(Option<String>, Value)],
    result: &Value,
    fill_guard: Option<&CrossClaimFillGuard>,
) -> CrossClaimStoreOutcome {
    if !cross_claim_pure_admitted(fn_node, func_name) {
        return CrossClaimStoreOutcome::NotAdmitted;
    }
    let Some(args_hash) = cross_claim_args_hash(ctx, args) else {
        return CrossClaimStoreOutcome::RefusedArgsNotHashable;
    };
    let memo_key = (Rc::as_ptr(fn_node) as usize, args_hash);
    let Some(portable_args) = portable_args_from_ctx(ctx, args) else {
        return CrossClaimStoreOutcome::RefusedArgsNotPortable;
    };
    let portable = match portable_value_from_ctx_at(ctx, result, &mut String::new()) {
        Ok(p) => p,
        Err(refusal) => {
            // TOTAL publication check: the first origin-bound child refuses the whole store,
            // typed and located, and is retained so the warm path can stop the line with the
            // exact path instead of a bare "not stored".
            CROSS_CLAIM_LAST_UNPORTABLE
                .with(|c| *c.borrow_mut() = Some((func_name.to_string(), refusal.clone())));
            CROSS_CLAIM_PURE_MEMO.with(|m| m.borrow_mut().unportable_refusals += 1);
            return CrossClaimStoreOutcome::RefusedValueNotPortable(refusal);
        }
    };
    let outcome = CROSS_CLAIM_PURE_MEMO.with(|m| {
        let mut m = m.borrow_mut();
        if let Some(bucket) = m.map.get(&memo_key) {
            if bucket.iter().any(|(stored_args, _)| {
                cross_claim_portable_args_match(stored_args, &portable_args)
            }) {
                return CrossClaimStoreOutcome::AlreadyPresent;
            }
        }
        if m.map.len() >= CROSS_CLAIM_PURE_MEMO_ENTRY_CAP {
            m.overflow += 1;
            return CrossClaimStoreOutcome::RefusedEntryCap;
        }
        // Byte budget on the retained representation, charged for the WHOLE entry —
        // argument row and value, collision-bucket entries included — before it lands.
        let entry_bytes = portable_value_size_bytes(&portable)
            + portable_args
                .iter()
                .map(|(n, v)| n.as_ref().map_or(0, |n| n.len()) + portable_value_size_bytes(v))
                .sum::<usize>();
        if m.bytes.saturating_add(entry_bytes) > cross_claim_byte_budget() {
            m.overflow += 1;
            return CrossClaimStoreOutcome::RefusedByteBudget;
        }
        m.bytes += entry_bytes;
        // Reify ONCE, at publication, and retain the value every later claim will be handed.
        // The portable form has done its two jobs by here — it proved total portability and it
        // measured the entry — and nothing downstream needs it again.
        let served = value_from_portable_ctx(ctx, &portable);
        m.map
            .entry(memo_key)
            .or_default()
            .push((portable_args, served));
        CrossClaimStoreOutcome::Stored
    });
    // Only a FRESH store bills a fill: an already-present entry did no work to charge, and
    // marking the guard would double-count the fill that actually landed the value.
    if outcome == CrossClaimStoreOutcome::Stored {
        keep_cross_claim_fn(fn_node);
        if let Some(guard) = fill_guard {
            guard.mark_stored();
        }
    }
    outcome
}

/// Evaluate one rostered NULLARY producer in `ctx` and seed the cross-claim tier, under the
/// same guard protocol as a claim-forced fill — so a preparation warm lands in the ledger as an
/// outside-fold fill, not on the first claim. Returns the TYPED outcome: a servable tier
/// (`Stored`, `AlreadyPresent`) vs each refusal by name, not one boolean.
pub fn warm_cross_claim_pure_producer(
    ctx: &InterpContext,
    qualified_fn: &str,
) -> Result<CrossClaimStoreOutcome, String> {
    with_active_ctx(ctx, || {
        let fn_node = ctx
            .lookup_fn(qualified_fn)
            .ok_or_else(|| format!("no declaration named '{qualified_fn}' in this frame"))?
            .clone();
        let bare = qualified_fn.rsplit('.').next().unwrap_or(qualified_fn);
        if !cross_claim_pure_admitted(&fn_node, bare) {
            return Err(format!(
                "'{qualified_fn}' did not resolve to an installed cross-claim roster identity"
            ));
        }
        let guard = CrossClaimFillGuard::enter(bare);
        let env = Env::empty();
        let value = with_lexical_base_env(&env, || call_function(ctx, &fn_node, &[], &env))
            .map_err(|e| format!("{qualified_fn}: {e}"))?;
        Ok(store_cross_claim_pure_memo(
            ctx,
            &fn_node,
            bare,
            &[],
            &value,
            Some(&guard),
        ))
    })
}

#[cfg(test)]
mod cross_claim_demand_census_tests {
    //! THE CENSUS'S OWN EVIDENCE. The discriminating fact is not that a fold aggregates — it is
    //! that the aggregate SEES a demand shape the per-frame ledger cannot represent, so each
    //! test below asserts the frame ledger's blindness beside the census's answer. Re-scope the
    //! census back to one frame and the first test reds; that is what makes it a wall rather
    //! than a decoration.
    use std::rc::Rc;

    use im::{vector as im_vec, HashMap};

    use crate::v1_compiler_infer_emit_info::empty_emit_graph_info;
    use crate::v1_compiler_infer_items::ResolvedGraph;
    use crate::v1_std_core::{make_expr_node, ExprData, SourceSpan};

    use super::{
        absorb_claim_recompute_demand, clear_cross_claim_demand_census, cross_claim_demand_rows,
        eval_recompute_key, eval_recompute_record, trace_totals, ExecutionMode, InterpContext,
        Value,
    };

    fn fresh_ctx() -> InterpContext {
        let graph = ResolvedGraph {
            modules: Rc::new(im_vec![]),
            item_registry: Rc::new(HashMap::new()),
            diagnostics: Rc::new(im_vec![]),
            emit_graph_info: empty_emit_graph_info(),
        };
        InterpContext::new(&graph, Rc::new(HashMap::new()), ExecutionMode::Hermetic)
    }

    fn node_at(file: &str, start: i64) -> Rc<crate::v1_std_core::Node> {
        make_expr_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            Rc::new(ExprData::NoExprData),
            Rc::new(im_vec![]),
            None,
            Rc::new(SourceSpan {
                file: file.to_string(),
                start,
                end: start,
            }),
        )
    }

    /// Evaluate `producer` once inside one claim's frame, at the given cost, and fold that frame
    /// into the census exactly as the floor's claim loop does.
    fn one_claim(
        producer: &str,
        fn_node: &Rc<crate::v1_std_core::Node>,
        arg: i64,
        ns: u128,
        claim: &str,
        module: &str,
    ) -> super::EvalRecomputeTotals {
        let ctx = fresh_ctx();
        let call_site = node_at("caller.dag", 1);
        let args = [(None, Value::Int(arg))];
        let key = eval_recompute_key(&ctx, fn_node, &args).expect("Int args key soundly");
        eval_recompute_record(&ctx, &call_site, fn_node, producer, key, ns);
        absorb_claim_recompute_demand(&ctx, claim, module);
        let totals = trace_totals(&ctx.eval_recompute_trace.borrow());
        totals
    }

    /// THE RED THIS INSTRUMENT EXISTS FOR. One producer, evaluated ONCE in each of two claims —
    /// the shape that dominates the required floor's tail cost — is invisible to every frame's
    /// own ledger (`duplicated_keys = 0`, because `count = 1` in each), and the census reports
    /// it as one identity demanded by two claims across two modules.
    #[test]
    fn a_producer_demanded_once_per_claim_is_invisible_per_frame_and_visible_across_claims() {
        super::refresh_eval_recompute_trace_enabled_cache_for_tests();
        if !super::eval_recompute_trace_enabled() {
            std::env::set_var("GUNBC_RECOMPUTE_TRACE", "1");
            super::refresh_eval_recompute_trace_enabled_cache_for_tests();
        }
        clear_cross_claim_demand_census();
        let producer_node = node_at("producer.dag", 100);
        let first = one_claim(
            "tm_target_model",
            &producer_node,
            7,
            250_000_000,
            "m1.claim_a",
            "m1",
        );
        let second = one_claim(
            "tm_target_model",
            &producer_node,
            7,
            250_000_000,
            "m2.claim_b",
            "m2",
        );
        assert_eq!(
            (first.duplicated_keys, second.duplicated_keys),
            (0, 0),
            "the per-frame ledger must report NO duplication — this is the blindness under test, \
             and if it ever reports some, this test is measuring something else"
        );
        let rows = cross_claim_demand_rows();
        let row = rows
            .iter()
            .find(|r| r.producer == "tm_target_model")
            .expect("the census must carry the producer both claims demanded");
        assert_eq!(row.claims, 2, "one demand per claim, two claims");
        assert_eq!(row.modules, 2, "two consumer modules");
        assert_eq!(
            row.cross_claim_wasted_ns(),
            250_000_000,
            "a second claim re-deriving the same identity wastes exactly one derivation"
        );
        clear_cross_claim_demand_census();
    }

    /// THE CONTROL THAT KEEPS `claims > 1` MEANINGFUL: a producer demanded by ONE claim is
    /// retained and ranks at zero. Its cost is that claim's own work and belongs to the cost
    /// receipt; a census that scored it would be reporting every expensive call as shareable.
    #[test]
    fn a_single_claim_producer_ranks_at_zero_cross_claim_waste() {
        super::refresh_eval_recompute_trace_enabled_cache_for_tests();
        if !super::eval_recompute_trace_enabled() {
            std::env::set_var("GUNBC_RECOMPUTE_TRACE", "1");
            super::refresh_eval_recompute_trace_enabled_cache_for_tests();
        }
        clear_cross_claim_demand_census();
        let only = node_at("producer.dag", 200);
        one_claim("tm_only_once", &only, 1, 400_000_000, "m1.claim_a", "m1");
        let rows = cross_claim_demand_rows();
        let row = rows
            .iter()
            .find(|r| r.producer == "tm_only_once")
            .expect("retained, so the shared population has a control to be measured against");
        assert_eq!(row.claims, 1);
        assert_eq!(row.cross_claim_wasted_ns(), 0);
        clear_cross_claim_demand_census();
    }

    /// TWO DECLARATIONS SPELLED THE SAME ARE TWO PRODUCERS. Keying on the name alone would merge
    /// them into one row reporting a producer that does not exist — and it would do so in the
    /// direction that manufactures cross-claim sharing out of two unrelated single-claim costs.
    #[test]
    fn same_named_producers_at_distinct_declaration_sites_do_not_merge() {
        super::refresh_eval_recompute_trace_enabled_cache_for_tests();
        if !super::eval_recompute_trace_enabled() {
            std::env::set_var("GUNBC_RECOMPUTE_TRACE", "1");
            super::refresh_eval_recompute_trace_enabled_cache_for_tests();
        }
        clear_cross_claim_demand_census();
        let here = node_at("alpha.dag", 10);
        let there = node_at("beta.dag", 10);
        one_claim("tm_homonym", &here, 1, 200_000_000, "m1.claim_a", "m1");
        one_claim("tm_homonym", &there, 1, 200_000_000, "m2.claim_b", "m2");
        let rows: Vec<_> = cross_claim_demand_rows()
            .into_iter()
            .filter(|r| r.producer == "tm_homonym")
            .collect();
        assert_eq!(rows.len(), 2, "two declarations, two rows");
        assert!(
            rows.iter().all(|r| r.claims == 1),
            "neither is shared: merging them would invent cross-claim demand"
        );
        clear_cross_claim_demand_census();
    }

    /// THE RETENTION FLOOR DISCLOSES WHAT IT DROPS. A sub-millisecond first sighting is not
    /// retained at identity grain, and the omitted count and its summed cost are reported — an
    /// artifact that truncated silently would be read as the population.
    #[test]
    fn the_retention_floor_omits_loudly_rather_than_silently() {
        super::refresh_eval_recompute_trace_enabled_cache_for_tests();
        if !super::eval_recompute_trace_enabled() {
            std::env::set_var("GUNBC_RECOMPUTE_TRACE", "1");
            super::refresh_eval_recompute_trace_enabled_cache_for_tests();
        }
        clear_cross_claim_demand_census();
        let tiny = node_at("producer.dag", 300);
        one_claim("tm_tiny", &tiny, 1, 1_000, "m1.claim_a", "m1");
        let d = super::cross_claim_demand_disclosure();
        assert_eq!(d.claims_absorbed, 1);
        assert_eq!(d.overflow_keys, 0);
        assert_eq!(
            d.omitted_keys, 1,
            "the sub-floor key is counted, not forgotten"
        );
        assert_eq!(
            d.omitted_ns, 1_000,
            "and its cost is carried in the disclosure"
        );
        assert!(
            !cross_claim_demand_rows()
                .iter()
                .any(|r| r.producer == "tm_tiny"),
            "it is genuinely not retained — the disclosure is the only place it exists"
        );
        clear_cross_claim_demand_census();
    }
}

#[cfg(test)]
mod cross_claim_memo_tests {
    use crate::v1_rt::RcStr;
    use std::rc::Rc;

    use im::{vector as im_vec, HashMap};

    use crate::v1_compiler_infer_emit_info::empty_emit_graph_info;
    use crate::v1_compiler_infer_items::ResolvedGraph;
    use crate::v1_std_core::{make_expr_node, no_span, ExprData};

    use super::{
        list_value, store_cross_claim_pure_memo, try_cross_claim_pure_memo, ExecutionMode,
        InterpContext, Value,
    };

    fn fresh_ctx() -> InterpContext {
        let graph = ResolvedGraph {
            modules: Rc::new(im_vec![]),
            item_registry: Rc::new(HashMap::new()),
            diagnostics: Rc::new(im_vec![]),
            emit_graph_info: empty_emit_graph_info(),
        };
        InterpContext::new(&graph, Rc::new(HashMap::new()), ExecutionMode::Hermetic)
    }

    // Discriminating RED for the required-floor failure in
    // `english_emit_add_ingest_round_trip_holds`: three source spellings resolve to one declared
    // arm and are evaluated in three fresh frames, while a fourth frame evaluates a distinct
    // declaration with the same short arm name. Equality follows the resolved parent+arm, not
    // encounter order and not the nickname used at the source site.
    #[test]
    fn structural_variant_equality_is_independent_of_evaluation_frame() {
        use crate::v1_compiler_compile::SourceFile;
        let result = crate::v1_compiler_compile::compile_to_resolved(Rc::new(im_vec![
            Rc::new(SourceFile {
                path: "fixture/diagnostic.dag".to_string(),
                content: "module fixture.diagnostic\ntype Diagnostics = None | Some {}\nfn bare() -> Diagnostics { None }\nfn qualified() -> Diagnostics { Diagnostics.None {} }\n"
                    .to_string(),
            }),
            Rc::new(SourceFile {
                path: "fixture/imported.dag".to_string(),
                content: "module fixture.imported\nimport fixture.diagnostic { Diagnostics, None }\nfn imported() -> Diagnostics { None }\n"
                    .to_string(),
            }),
            Rc::new(SourceFile {
                path: "fixture/other.dag".to_string(),
                content: "module fixture.other\ntype Other = None | Present {}\nfn other_none() -> Other { None }\n"
                    .to_string(),
            }),
        ]));
        let graph = result.graph.as_ref().expect("fixture graph");
        let eval = |name| {
            let ctx = InterpContext::new(
                graph,
                result.source_indices.clone(),
                ExecutionMode::Hermetic,
            );
            super::run_in_context(&ctx, name, false).expect(name)
        };
        let bare = eval("bare");
        let qualified = eval("qualified");
        let imported = eval("imported");
        let other = eval("other_none");
        assert_eq!(bare, qualified);
        assert_eq!(bare, imported);
        assert_ne!(bare, other);
    }

    #[test]
    fn typed_bool_literals_do_not_need_lexeme_shorthands_but_zero_still_does() {
        use crate::v1_compiler_compile::SourceFile;
        let result = crate::v1_compiler_compile::compile_to_resolved(Rc::new(im_vec![Rc::new(
            SourceFile {
                path: "fixture/shorthand.dag".to_string(),
                content: "module fixture.shorthand\ntype UserToken = Zero | Other\nfn literal_true() -> Bool { true }\nfn literal_false() -> Bool { false }\nfn user_zero() -> UserToken { Zero }\n"
                    .to_string(),
            },
        )]));
        let graph = result.graph.as_ref().expect("fixture graph");
        let eval = |name| {
            let ctx = InterpContext::new(
                graph,
                result.source_indices.clone(),
                ExecutionMode::Hermetic,
            );
            super::run_in_context(&ctx, name, false).expect(name)
        };
        assert_eq!(eval("literal_true"), Value::Bool(true));
        assert_eq!(eval("literal_false"), Value::Bool(false));
        // Executable bounded residual: `Zero` is still the intentional native Nat encoding, but
        // the seed receives no declaration identity with which to distinguish it from this user
        // coproduct. NS-0B must make this select the native representation by declaration rather
        // than by lexeme.
        assert_eq!(eval("user_zero"), Value::Int(0));
    }

    // Bounded residual: the runtime carrier still has only parent and arm spellings. This
    // executable specimen stays equal until NS-0B threads owner-module declaration identity.
    #[test]
    fn same_spelled_variants_from_distinct_owners_reproduce_identity_residual() {
        use crate::v1_compiler_compile::SourceFile;
        let result = crate::v1_compiler_compile::compile_to_resolved(Rc::new(im_vec![
            Rc::new(SourceFile {
                path: "fixture/left.dag".to_string(),
                content: "module fixture.left\ntype Diagnostics = None\nfn left_none() -> Diagnostics { None }\n".to_string(),
            }),
            Rc::new(SourceFile {
                path: "fixture/right.dag".to_string(),
                content: "module fixture.right\ntype Diagnostics = None\nfn right_none() -> Diagnostics { None }\n".to_string(),
            }),
        ]));
        let graph = result.graph.as_ref().expect("fixture graph");
        let eval = |name| {
            let ctx = InterpContext::new(
                graph,
                result.source_indices.clone(),
                ExecutionMode::Hermetic,
            );
            super::run_in_context(&ctx, name, false).expect(name)
        };
        assert_eq!(eval("left_none"), eval("right_none"));
    }

    // The mirror-image defect is silent wrongness rather than a false negative: before symbols
    // were process-canonical, two different first-seen spellings occupied the same ordinal in
    // two frames and compared equal. Keep this direction separate so an always-equal repair
    // cannot satisfy the reported `None == None` case.
    #[test]
    fn distinct_spellings_at_the_same_frame_ordinal_are_not_equal() {
        let left_frame = fresh_ctx();
        let right_frame = fresh_ctx();
        let left = Value::Variant {
            type_name: left_frame.sym("Diagnostics"),
            variant_name: left_frame.sym("None"),
            fields: Rc::new(vec![]),
        };
        let right = Value::Variant {
            type_name: right_frame.sym("Diagnostics"),
            variant_name: right_frame.sym("Some"),
            fields: Rc::new(vec![]),
        };
        assert_ne!(left, right);

        let same_arm_name_from_another_declaration = Value::Variant {
            type_name: right_frame.sym("OtherDiagnostics"),
            variant_name: right_frame.sym("None"),
            fields: Rc::new(vec![]),
        };
        assert_ne!(left, same_arm_name_from_another_declaration);
    }

    // RED1 (operator ruling, 2026-08-29): a value carrying an origin-bound fn ANYWHERE
    // refuses at PUBLICATION with the exact path — never admitted and later discovered as
    // a no-such-field at retrieval (six emit reds on run 33269961629 were this class).
    #[test]
    fn an_origin_bound_fn_refuses_at_store_with_its_path() {
        use super::{portable_value_from_ctx_at, Env};
        let ctx = fresh_ctx();
        let fn_node = make_expr_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            Rc::new(ExprData::NoExprData),
            Rc::new(im_vec![]),
            None,
            no_span(),
        );
        let with_fn = Value::Record {
            type_name: ctx.sym("TargetModelish"),
            fields: Rc::new(vec![(
                ctx.sym("transform"),
                Value::Fn {
                    node: fn_node.clone(),
                },
            )]),
        };
        let refusal = portable_value_from_ctx_at(&ctx, &with_fn, &mut String::new())
            .expect_err("a fn-carrying record must refuse reification");
        assert_eq!(refusal.path_into_value, ".transform");
        assert_eq!(refusal.encountered_kind, "OriginBoundNode");

        // RED2: contamination several levels down names the whole path.
        let nested = Value::Record {
            type_name: ctx.sym("Outer"),
            fields: Rc::new(vec![(
                ctx.sym("a"),
                list_value(vec![Value::Record {
                    type_name: ctx.sym("Inner"),
                    fields: Rc::new(vec![(
                        ctx.sym("function"),
                        Value::Closure {
                            params: vec![],
                            body: fn_node,
                            env: Env::empty(),
                        },
                    )]),
                }]),
            )]),
        };
        let refusal = portable_value_from_ctx_at(&ctx, &nested, &mut String::new())
            .expect_err("nested contamination must refuse");
        assert_eq!(refusal.path_into_value, ".a[0].function");
        assert_eq!(refusal.encountered_kind, "Closure");
    }

    // RED3: the same argument row under a DIFFERENT producer identity (a re-prepared
    // subject builds new fn nodes) MISSES — emitter implementation identity is part of the
    // key, so a stale subject's value can never serve a new subject's call.
    #[test]
    fn the_same_args_under_a_different_fn_identity_miss() {
        use super::{store_cross_claim_pure_memo, try_cross_claim_pure_memo};
        super::clear_cross_claim_pure_memos();
        let ctx = fresh_ctx();
        let mk = || {
            make_expr_node(
                Rc::new(
                    crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
                ),
                Rc::new(ExprData::NoExprData),
                Rc::new(im_vec![]),
                None,
                no_span(),
            )
        };
        let fn_a = mk();
        let fn_b = mk();
        let args = [(None, list_value(vec![Value::Int(7)]))];
        store_cross_claim_pure_memo(&ctx, &fn_a, "prepare_grammar", &args, &Value::Int(1), None);
        assert!(
            try_cross_claim_pure_memo(&ctx, &fn_a, "prepare_grammar", &args).is_some(),
            "control: the storing identity serves"
        );
        let ctx_fresh = fresh_ctx();
        assert!(
            try_cross_claim_pure_memo(&ctx_fresh, &fn_b, "prepare_grammar", &args).is_none(),
            "a different fn identity must miss"
        );
        super::clear_cross_claim_pure_memos();
    }

    // GREEN control beside RED1, and regression control for the field-ORDER defect: a portable
    // record stored from frame A must have EVERY field readable in frame B via `fields_get`,
    // which binary-searches on the consuming interner's ordinals — frame A's order is unsorted
    // in B and misses every lookup (run 33269961629's "no field 'produced_decl_support' on
    // type 'TargetModel'").
    #[test]
    fn every_field_of_a_served_record_resolves_under_a_reordered_interner() {
        use super::{fields_get, portable_value_from_ctx, value_from_portable_ctx};
        let ctx_a = fresh_ctx();
        // Field names interned in one order in frame A...
        let value = Value::Record {
            type_name: ctx_a.sym("TargetModelish"),
            fields: Rc::new(super::sorted_fields(vec![
                (ctx_a.sym("zeta"), Value::Int(1)),
                (ctx_a.sym("alpha"), Value::Int(2)),
                (ctx_a.sym("midfield"), Value::Int(3)),
            ])),
        };
        let portable = portable_value_from_ctx(&ctx_a, &value).expect("portable");

        let ctx_b = fresh_ctx();
        // ...and pre-interned in a DIFFERENT order in frame B, so B's ordinals disagree
        // with A's field order.
        let _ = ctx_b.sym("alpha");
        let _ = ctx_b.sym("midfield");
        let _ = ctx_b.sym("zeta");
        let served = value_from_portable_ctx(&ctx_b, &portable);
        let Value::Record { fields, .. } = &served else {
            panic!("expected Record, got {served:?}");
        };
        for (name, want) in [("zeta", 1), ("alpha", 2), ("midfield", 3)] {
            match fields_get(fields, ctx_b.sym(name)) {
                Some(Value::Int(got)) => assert_eq!(*got, want, "{name}"),
                other => panic!("field '{name}' must resolve via fields_get, got {other:?}"),
            }
        }
    }

    // RED (review 57446 F1): admission is by RESOLVED DECLARATION IDENTITY, so a bare-name
    // HOMONYM in a non-rostered module must NOT store — name-set admission cached any
    // same-named fn in the subject. Fn-node identity in the key stopped cross-SERVING between
    // homonyms but not CACHING the unintended fn; this red does.
    #[test]
    fn a_homonym_outside_the_roster_identity_does_not_store() {
        use super::{
            install_cross_claim_pure_share_roster, store_cross_claim_pure_memo,
            try_cross_claim_pure_memo,
        };
        super::clear_cross_claim_pure_memos();
        let ctx = fresh_ctx();
        let mk = || {
            make_expr_node(
                Rc::new(
                    crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
                ),
                Rc::new(ExprData::NoExprData),
                Rc::new(im_vec![]),
                None,
                no_span(),
            )
        };
        let rostered = mk();
        let homonym = mk(); // same bare name, different declaration — a different module's fn
        install_cross_claim_pure_share_roster([rostered.clone()]);
        let args = [(None, list_value(vec![Value::Int(3)]))];
        store_cross_claim_pure_memo(
            &ctx,
            &homonym,
            "tm_shared_name",
            &args,
            &Value::Int(9),
            None,
        );
        assert!(
            try_cross_claim_pure_memo(&ctx, &homonym, "tm_shared_name", &args).is_none(),
            "a homonym outside the rostered identity must never be cached"
        );
        store_cross_claim_pure_memo(
            &ctx,
            &rostered,
            "tm_shared_name",
            &args,
            &Value::Int(9),
            None,
        );
        assert!(
            try_cross_claim_pure_memo(&ctx, &rostered, "tm_shared_name", &args).is_some(),
            "control: the resolved roster identity stores and serves"
        );
        super::clear_cross_claim_pure_memos();
    }

    // RED FOR THE ZERO-WALK SERVE. The re-enrol trigger `v2.workflow.floor_pure_producer_share`
    // carries is "a serve that does not re-intern and re-sort per consuming frame — an
    // interner-stable representation the tier can hand over without walking the whole value".
    // This is the executed evidence that it holds, and it discriminates: under the previous
    // serve every consuming frame ran `value_from_portable_ctx` over the whole value, so two
    // frames received two DISTINCT allocations of an equal value and `Rc::ptr_eq` was false.
    // Structural equality cannot see the difference — it was equal before and is equal now —
    // so the assertion is on ALLOCATION IDENTITY, which is what "did not walk it" means.
    #[test]
    fn two_frames_are_served_the_same_allocation_rather_than_two_reconstructions() {
        use super::{
            store_cross_claim_pure_memo, try_cross_claim_pure_memo, CrossClaimStoreOutcome,
        };
        super::clear_cross_claim_pure_memos();
        let filling = fresh_ctx();
        let fn_node = make_expr_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            Rc::new(ExprData::NoExprData),
            Rc::new(im_vec![]),
            None,
            no_span(),
        );
        let args: [(Option<String>, Value); 0] = [];
        // A nested value, so a walk would have to rebuild an inner container too.
        let value = list_value(vec![
            Value::Str(RcStr::from("alpha")),
            list_value(vec![Value::Int(1), Value::Int(2)]),
        ]);
        assert_eq!(
            store_cross_claim_pure_memo(&filling, &fn_node, "prepare_grammar", &args, &value, None),
            CrossClaimStoreOutcome::Stored
        );

        let consumer_a = fresh_ctx();
        let consumer_b = fresh_ctx();
        let served_a = try_cross_claim_pure_memo(&consumer_a, &fn_node, "prepare_grammar", &args)
            .expect("frame A is served");
        let served_b = try_cross_claim_pure_memo(&consumer_b, &fn_node, "prepare_grammar", &args)
            .expect("frame B is served");

        let (Value::List(a), Value::List(b)) = (&served_a, &served_b) else {
            panic!("expected two served lists, got {served_a:?} and {served_b:?}");
        };
        assert!(
            Rc::ptr_eq(a, b),
            "two consuming frames must share one allocation: a serve that reconstructs the \
             value per frame is the cost this tier's re-enrol trigger names"
        );
        // And the served value is still the value that was stored, in both frames.
        assert_eq!(served_a, value);
        assert_eq!(served_b, value);
        super::clear_cross_claim_pure_memos();
    }

    // RED (#9721): AN ALREADY-PRESENT ENTRY IS SERVABLE AND IS NOT A REFUSAL, and each decline
    // names its OWN cause. The replaced boolean reported a correctly populated tier as
    // `stored=false`, which the floor printed as "PureProducerShareWarmNotStored ... duplicate
    // key, entry cap, or byte budget" — one line for three causes, wrong verdict for the fourth.
    // That refused #9721's floor run: `rust_target_model_staging` is reachable from
    // `rust_target_model`, so warming the first stored it and its own warm found it present.
    //
    // The discriminating pair: AlreadyPresent and RefusedByteBudget both decline to write and
    // the OLD boolean could not tell them apart; one is servable, one is not, so re-conflating
    // them reds.
    #[test]
    fn an_already_present_entry_is_servable_while_a_declined_one_is_not() {
        use super::{store_cross_claim_pure_memo, CrossClaimStoreOutcome};
        super::clear_cross_claim_pure_memos();
        let ctx = fresh_ctx();
        let fn_node = make_expr_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            Rc::new(ExprData::NoExprData),
            Rc::new(im_vec![]),
            None,
            no_span(),
        );
        let args = [(None, list_value(vec![Value::Int(7)]))];
        let value = Value::Str(RcStr::from("shared"));

        let first =
            store_cross_claim_pure_memo(&ctx, &fn_node, "prepare_grammar", &args, &value, None);
        assert_eq!(first, CrossClaimStoreOutcome::Stored);
        assert!(first.is_servable());

        // The SECOND publication of the same key with a structurally equal argument row.
        let second =
            store_cross_claim_pure_memo(&ctx, &fn_node, "prepare_grammar", &args, &value, None);
        assert_eq!(
            second,
            CrossClaimStoreOutcome::AlreadyPresent,
            "a duplicate must be named as already-present, not lumped with the refusals"
        );
        assert!(
            second.is_servable(),
            "the value IS retained, so the warm's obligation is satisfied"
        );
        let (_, overflow) = super::cross_claim_pure_memo_counts();
        assert_eq!(overflow, 0, "an already-present entry is not an overflow");

        // The discriminating half: a genuine decline is NOT servable and names its cause.
        super::CROSS_CLAIM_BYTE_BUDGET_TEST_OVERRIDE.with(|c| c.set(Some(512)));
        let big_args = [(None, list_value(vec![Value::Int(8)]))];
        let big = Value::Str(RcStr::from("x".repeat(4096)));
        let refused =
            store_cross_claim_pure_memo(&ctx, &fn_node, "prepare_grammar", &big_args, &big, None);
        assert_eq!(refused, CrossClaimStoreOutcome::RefusedByteBudget);
        assert!(
            !refused.is_servable(),
            "nothing is retained, so a warm hitting this must stop the line"
        );
        assert_eq!(refused.cause(), "ByteBudgetExceeded");
        assert_ne!(
            refused.cause(),
            second.cause(),
            "the two declines must not report the same cause"
        );
        super::CROSS_CLAIM_BYTE_BUDGET_TEST_OVERRIDE.with(|c| c.set(None));
        super::clear_cross_claim_pure_memos();
    }

    // RED (review 57554): A CAUSE MUST NEVER CARRY ANOTHER CAUSE'S EVIDENCE. The located
    // path/kind lives on the RefusedValueNotPortable arm, so a later byte-budget or entry-cap
    // refusal has nothing to borrow. `CROSS_CLAIM_LAST_UNPORTABLE` is written ONLY by the
    // unportable arm and cleared by nothing, so a runner reading it for every non-servable
    // outcome printed a stale `path=.produced_decl_support.render kind=OriginBoundNode` beside
    // a ByteBudgetExceeded cause.
    //
    // ORDER is the discriminator: the unportable refusal happens FIRST and populates the slot;
    // the budget refusal must still report no detail.
    #[test]
    fn a_budget_refusal_after_an_unportable_one_carries_no_borrowed_detail() {
        use super::{store_cross_claim_pure_memo, CrossClaimStoreOutcome};
        super::clear_cross_claim_pure_memos();
        let ctx = fresh_ctx();
        let fn_node = make_expr_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            Rc::new(ExprData::NoExprData),
            Rc::new(im_vec![]),
            None,
            no_span(),
        );

        // A fn-carrying record refuses reification and populates the retained slot.
        let unportable = Value::Record {
            type_name: ctx.sym("TargetModelish"),
            fields: Rc::new(vec![(
                ctx.sym("transform"),
                Value::Fn {
                    node: fn_node.clone(),
                },
            )]),
        };
        let first_args = [(None, list_value(vec![Value::Int(1)]))];
        let first = store_cross_claim_pure_memo(
            &ctx,
            &fn_node,
            "prepare_grammar",
            &first_args,
            &unportable,
            None,
        );
        assert_eq!(first.cause(), "ServeCacheValueNotPortable");
        let carried = first
            .not_portable_detail()
            .expect("the unportable arm carries its own located refusal");
        assert_eq!(carried.path_into_value, ".transform");
        assert_eq!(carried.encountered_kind, "OriginBoundNode");

        // Now a DIFFERENT refusal, with the slot still populated by the one above.
        super::CROSS_CLAIM_BYTE_BUDGET_TEST_OVERRIDE.with(|c| c.set(Some(512)));
        let big_args = [(None, list_value(vec![Value::Int(2)]))];
        let big = Value::Str(RcStr::from("x".repeat(4096)));
        let second =
            store_cross_claim_pure_memo(&ctx, &fn_node, "prepare_grammar", &big_args, &big, None);
        assert_eq!(second, CrossClaimStoreOutcome::RefusedByteBudget);
        assert_eq!(second.cause(), "ByteBudgetExceeded");
        assert!(
            second.not_portable_detail().is_none(),
            "a byte-budget refusal must not borrow the unportable arm's path and kind"
        );
        super::CROSS_CLAIM_BYTE_BUDGET_TEST_OVERRIDE.with(|c| c.set(None));
        super::clear_cross_claim_pure_memos();
    }

    // RED (#9721): an already-present publication must not bill a second fill. The guard
    // marks stored only on a FRESH write, so the value's cost is charged once, to the
    // call that actually did the work.
    #[test]
    fn an_already_present_publication_bills_no_second_fill() {
        use super::{store_cross_claim_pure_memo, CrossClaimFillGuard, CrossClaimStoreOutcome};
        super::clear_cross_claim_pure_memos();
        let ctx = fresh_ctx();
        let fn_node = make_expr_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            Rc::new(ExprData::NoExprData),
            Rc::new(im_vec![]),
            None,
            no_span(),
        );
        let args = [(None, list_value(vec![Value::Int(9)]))];
        let value = Value::Str(RcStr::from("once"));

        let first_guard = CrossClaimFillGuard::enter("prepare_grammar");
        let first = store_cross_claim_pure_memo(
            &ctx,
            &fn_node,
            "prepare_grammar",
            &args,
            &value,
            Some(&first_guard),
        );
        assert_eq!(first, CrossClaimStoreOutcome::Stored);
        assert!(first_guard.stored.get(), "the fresh store bills its fill");
        drop(first_guard);

        let second_guard = CrossClaimFillGuard::enter("prepare_grammar");
        let second = store_cross_claim_pure_memo(
            &ctx,
            &fn_node,
            "prepare_grammar",
            &args,
            &value,
            Some(&second_guard),
        );
        assert_eq!(second, CrossClaimStoreOutcome::AlreadyPresent);
        assert!(
            !second_guard.stored.get(),
            "an already-present entry did no work, so it must not bill a fill"
        );
        drop(second_guard);
        super::clear_cross_claim_pure_memos();
    }

    // RED (review 57446 F2): the retention bound is a BYTE budget on the reified entry
    // (args + value), not an entry count — a store that would cross the budget refuses
    // counted, and later smaller stores still land within the remaining budget.
    #[test]
    fn a_store_past_the_byte_budget_refuses_counted() {
        use super::{
            cross_claim_pure_memo_counts, store_cross_claim_pure_memo, try_cross_claim_pure_memo,
        };
        super::clear_cross_claim_pure_memos();
        super::CROSS_CLAIM_BYTE_BUDGET_TEST_OVERRIDE.with(|c| c.set(Some(512)));
        let ctx = fresh_ctx();
        let mk = || {
            make_expr_node(
                Rc::new(
                    crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
                ),
                Rc::new(ExprData::NoExprData),
                Rc::new(im_vec![]),
                None,
                no_span(),
            )
        };
        let fn_node = mk();
        let big_args = [(None, list_value(vec![Value::Int(1)]))];
        let big = Value::Str(RcStr::from("x".repeat(4096)));
        store_cross_claim_pure_memo(&ctx, &fn_node, "prepare_grammar", &big_args, &big, None);
        assert!(
            try_cross_claim_pure_memo(&ctx, &fn_node, "prepare_grammar", &big_args).is_none(),
            "an over-budget value must refuse to store"
        );
        let (_, overflow) = cross_claim_pure_memo_counts();
        assert_eq!(overflow, 1, "the refusal is counted, never silent");
        let small_args = [(None, list_value(vec![Value::Int(2)]))];
        let small = Value::Str(RcStr::from("ok"));
        store_cross_claim_pure_memo(&ctx, &fn_node, "prepare_grammar", &small_args, &small, None);
        assert!(
            try_cross_claim_pure_memo(&ctx, &fn_node, "prepare_grammar", &small_args).is_some(),
            "control: a within-budget value still stores"
        );
        super::CROSS_CLAIM_BYTE_BUDGET_TEST_OVERRIDE.with(|c| c.set(None));
        super::clear_cross_claim_pure_memos();
    }

    // Regression for gunbc#8505: the cross-claim `prepare_grammar` memo cached a raw `Value`
    // (`Symbol`s indexing the PRODUCING ctx's interner) keyed on fn-node identity + content
    // hash, unscoped to the consuming `InterpContext`. Required-floor builds a fresh ctx per
    // claim, so a value stored under `ctx_a` and served to `ctx_b` resolved against the wrong
    // interner. Pre-`PortableValue` this failed: `ctx_b`'s decoy vocabulary is interned in a
    // different order, so a leaked `Symbol` index resolves to the WRONG name (or out of bounds)
    // rather than happening to work.
    #[test]
    fn cross_claim_memo_survives_a_fresh_consuming_context() {
        let ctx_a = fresh_ctx();
        // Decoy vocabulary interned into ctx_a BEFORE the memoized value's symbols, so
        // "PreparedModeledGrammar"/"GrammarFirstAnalysis"/"stage" do not land at index 0.
        let _ = ctx_a.sym("alpha_decoy_a");
        let _ = ctx_a.sym("beta_decoy_a");

        let ctx_b = fresh_ctx();
        // A DIFFERENT decoy vocabulary, interned in a different order, so any symbol
        // index that leaked unresolved from ctx_a would resolve to a different (or
        // out-of-bounds) name under ctx_b rather than accidentally lining up.
        let _ = ctx_b.sym("zzz_decoy_b_one");
        let _ = ctx_b.sym("zzz_decoy_b_two");
        let _ = ctx_b.sym("zzz_decoy_b_three");

        let fn_node = make_expr_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            Rc::new(ExprData::NoExprData),
            Rc::new(im_vec![]),
            None,
            no_span(),
        );

        // A non-empty List argument routes through `EvalRecomputeArgKey::ContentHash`
        // (a bare `Value::Str` would take the `StrHash` arm instead and never reach the
        // cross-claim memo at all).
        let arg = list_value(vec![Value::Int(42)]);
        let args = [(None, arg)];

        let result = Value::Variant {
            type_name: ctx_a.sym("PreparedModeledGrammar"),
            variant_name: ctx_a.sym("GrammarFirstAnalysis"),
            fields: Rc::new(vec![(ctx_a.sym("stage"), Value::Str(RcStr::from("first")))]),
        };

        store_cross_claim_pure_memo(&ctx_a, &fn_node, "prepare_grammar", &args, &result, None);

        let loaded = try_cross_claim_pure_memo(&ctx_b, &fn_node, "prepare_grammar", &args)
            .expect("cross-claim memo hit for the same fn identity + content hash");

        match loaded {
            Value::Variant {
                type_name,
                variant_name,
                fields,
            } => {
                assert_eq!(ctx_b.resolve(type_name), "PreparedModeledGrammar");
                assert_eq!(ctx_b.resolve(variant_name), "GrammarFirstAnalysis");
                assert_eq!(fields.len(), 1);
                assert_eq!(ctx_b.resolve(fields[0].0), "stage");
                match &fields[0].1 {
                    Value::Str(s) => assert_eq!(s.as_ref(), "first"),
                    other => panic!("expected Str field, got {other:?}"),
                }
            }
            other => panic!("expected Variant, got {other:?}"),
        }
    }

    // Regression for the SECOND half of gunbc#8505's follow-up (dashboard adhoc-e78c4260-d3a):
    // the test above covers the STORED VALUE (PortableValue), not the memo KEY, which
    // `eval_recompute_arg_key`/`eval_recompute_value_hash` still built from interner-local
    // `Symbol` ordinals (`type_name.0`, `variant_name.0`) rather than resolved TEXT. Ordinals
    // are per-context encounter order, so distinct arguments from independently-interned
    // contexts can share an ordinal pattern for DIFFERENT strings, aliasing one memo entry onto
    // an unrelated call -- silent wrongness one level above the PortableValue bug. Under the
    // ordinal-keyed code this passed wrongly (returning ctx_a's result): "TypeFoo" is ctx_a's
    // first NON-well-known symbol and "TypeBar" is ctx_c's -- the SAME ordinal (well-known
    // free-monoid symbols are pre-interned identically in every fresh context), so the old
    // `UnitVariant(u32, u32)` key was identical for two types.
    #[test]
    fn cross_claim_memo_key_is_content_addressed_not_ordinal_addressed() {
        let ctx_a = fresh_ctx();
        let type_a = ctx_a.sym("TypeFoo"); // ctx_a's first non-well-known symbol
        let variant_a = ctx_a.sym("VariantX"); // ctx_a's second non-well-known symbol

        let ctx_c = fresh_ctx();
        let type_c = ctx_c.sym("TypeBar"); // same ordinal as type_a -- collides
        let variant_c = ctx_c.sym("VariantY"); // same ordinal as variant_a -- collides

        let fn_node = make_expr_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            Rc::new(ExprData::NoExprData),
            Rc::new(im_vec![]),
            None,
            no_span(),
        );

        let args_a = [(
            None,
            Value::Variant {
                type_name: type_a,
                variant_name: variant_a,
                fields: Rc::new(vec![]),
            },
        )];
        let result_a = Value::Str(RcStr::from("result-for-TypeFoo"));
        store_cross_claim_pure_memo(
            &ctx_a,
            &fn_node,
            "prepare_grammar",
            &args_a,
            &result_a,
            None,
        );

        let args_c = [(
            None,
            Value::Variant {
                type_name: type_c,
                variant_name: variant_c,
                fields: Rc::new(vec![]),
            },
        )];
        // A genuinely different argument (TypeBar/VariantY, not TypeFoo/VariantX)
        // must NOT hit the entry stored for ctx_a's argument, even though both
        // interners assigned the identical ordinal pattern (0, 1) to their symbols.
        let loaded = try_cross_claim_pure_memo(&ctx_c, &fn_node, "prepare_grammar", &args_c);
        assert!(
            loaded.is_none(),
            "ordinal-collision aliased TypeBar/VariantY onto TypeFoo/VariantX's memo entry"
        );
    }

    // Same defect class via the Record/Fields-frame path (`eval_recompute_frame_integrate`'s
    // per-field mixing) rather than the `UnitVariant` fast path: a FIELD name ordinal
    // collision must not alias two records of one type with different field identity.
    #[test]
    fn cross_claim_memo_key_distinguishes_field_names_across_ordinal_collisions() {
        let ctx_a = fresh_ctx();
        let type_a = ctx_a.sym("Wrapper"); // ctx_a's first non-well-known symbol
        let field_a = ctx_a.sym("alpha"); // ctx_a's second non-well-known symbol

        let ctx_c = fresh_ctx();
        let type_c = ctx_c.sym("Wrapper"); // same ordinal as type_a, same string
        let field_c = ctx_c.sym("beta"); // same ordinal as field_a, DIFFERENT string

        let fn_node = make_expr_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            Rc::new(ExprData::NoExprData),
            Rc::new(im_vec![]),
            None,
            no_span(),
        );

        let args_a = [(
            None,
            Value::Record {
                type_name: type_a,
                fields: Rc::new(vec![(field_a, Value::Int(1))]),
            },
        )];
        let result_a = Value::Str(RcStr::from("result-for-alpha"));
        store_cross_claim_pure_memo(
            &ctx_a,
            &fn_node,
            "prepare_grammar",
            &args_a,
            &result_a,
            None,
        );

        let args_c = [(
            None,
            Value::Record {
                type_name: type_c,
                fields: Rc::new(vec![(field_c, Value::Int(1))]),
            },
        )];
        let loaded = try_cross_claim_pure_memo(&ctx_c, &fn_node, "prepare_grammar", &args_c);
        assert!(
            loaded.is_none(),
            "field-name ordinal collision (alpha vs beta at the same ordinal) aliased across contexts"
        );
    }

    // Regression for the `RefCell already borrowed` panic CI surfaced on gunbc#8565 (dashboard
    // adhoc-e78c4260-d3a): `eval_recompute_value_hash` hashes a `Value::Map` arg via each
    // `CanonKey`, whose `hash` calls `value_hash`, which for String/List/Variant calls
    // `free_monoid_to_vec`. That used to intern Cons/Empty/head/tail via `ctx.sym()` (MUTABLE
    // borrow of `ctx.symbols`) while the memo-key computation held an IMMUTABLE
    // `ctx.symbols.borrow()` -- a same-thread double-borrow that panics. Needs `active_ctx()` to
    // be the SAME ctx whose `symbols` is borrowed, hence `with_active_context` (as production
    // evaluation always is).
    #[test]
    fn cross_claim_memo_key_hashes_a_map_arg_with_string_keys_without_panicking() {
        let ctx = fresh_ctx();
        let fn_node = make_expr_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            Rc::new(ExprData::NoExprData),
            Rc::new(im_vec![]),
            None,
            no_span(),
        );
        let result = Value::Str(RcStr::from("ok"));

        super::with_active_context(&ctx, || {
            let mut entries: HashMap<super::CanonKey, Value> = HashMap::new();
            let key =
                super::CanonKey::new(Value::Str(RcStr::from("a"))).expect("Str is a valid map key");
            entries.insert(key, Value::Int(1));
            let args = [(None, super::map_value(entries))];

            store_cross_claim_pure_memo(&ctx, &fn_node, "prepare_grammar", &args, &result, None);
            let loaded = try_cross_claim_pure_memo(&ctx, &fn_node, "prepare_grammar", &args)
                .expect("cross-claim memo hit for the same fn identity + content hash");
            match loaded {
                Value::Str(s) => assert_eq!(s.as_ref(), "ok"),
                other => panic!("expected Str, got {other:?}"),
            }
        });
    }

    // Regression for the follow-up on 975f2b166d (dashboard warm-boar-256):
    // `free_monoid_ctx_syms`'s read-only fallback (taken when the interner is already borrowed)
    // must FIND a real Cons/Empty-encoded value's symbols, not merely not panic. Missing or
    // wrong pre-interning at context construction would let a free-monoid `Value::Variant`
    // under a held borrow fall through `value_hash`'s generic Variant arm (DESIGN §5's
    // empty-observation narrow: a contention miss reported as "not a list"). Builds a
    // Cons(1, Cons(2, Empty)) chain as a `Value::Map` key -- hashed inside
    // `eval_recompute_key`'s held borrow -- and proves the memo round-trips, which holds only
    // if Cons/head/tail resolved.
    #[test]
    fn cross_claim_memo_key_hashes_a_map_arg_with_a_free_monoid_list_key_under_held_borrow() {
        let ctx = fresh_ctx();
        let fn_node = make_expr_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            Rc::new(ExprData::NoExprData),
            Rc::new(im_vec![]),
            None,
            no_span(),
        );
        let result = Value::Str(RcStr::from("ok"));

        super::with_active_context(&ctx, || {
            let list_type = ctx.sym("List");
            let empty = Value::Variant {
                type_name: list_type,
                variant_name: ctx.sym("Empty"),
                fields: Rc::new(vec![]),
            };
            let cons_inner = Value::Variant {
                type_name: list_type,
                variant_name: ctx.sym("Cons"),
                fields: Rc::new(vec![
                    (ctx.sym("head"), Value::Int(2)),
                    (ctx.sym("tail"), empty),
                ]),
            };
            let cons_outer = Value::Variant {
                type_name: list_type,
                variant_name: ctx.sym("Cons"),
                fields: Rc::new(vec![
                    (ctx.sym("head"), Value::Int(1)),
                    (ctx.sym("tail"), cons_inner),
                ]),
            };

            let mut entries: HashMap<super::CanonKey, Value> = HashMap::new();
            let key = super::CanonKey::new(cons_outer).expect("Variant is a valid map key");
            entries.insert(key, Value::Int(1));
            let args = [(None, super::map_value(entries))];

            store_cross_claim_pure_memo(&ctx, &fn_node, "prepare_grammar", &args, &result, None);
            let loaded = try_cross_claim_pure_memo(&ctx, &fn_node, "prepare_grammar", &args)
                .expect("cross-claim memo hit for the same fn identity + content hash");
            match loaded {
                Value::Str(s) => assert_eq!(s.as_ref(), "ok"),
                other => panic!("expected Str, got {other:?}"),
            }
        });
    }

    // Sharper version of the test above: calls `free_monoid_to_vec` directly under a held
    // immutable `ctx.symbols` borrow (the shape `eval_recompute_key`/`eval_recompute_value_hash`
    // create) and asserts the FLATTENED CONTENT, not just no panic. A silent narrow (fallback
    // missing a symbol and reporting `None`, or a stale offset assumption) returns `None` or a
    // wrong vec rather than panicking.
    #[test]
    fn free_monoid_to_vec_resolves_well_known_syms_under_a_held_immutable_borrow() {
        let ctx = fresh_ctx();
        // Decoy vocabulary interned before the free-monoid symbols come up again, so
        // a wrong assumption about a fixed well-known ordinal would not accidentally
        // still work.
        let _ = ctx.sym("decoy_one");
        let _ = ctx.sym("decoy_two");

        let list_type = ctx.sym("List");
        let empty = Value::Variant {
            type_name: list_type,
            variant_name: ctx.sym("Empty"),
            fields: Rc::new(vec![]),
        };
        let cons = Value::Variant {
            type_name: list_type,
            variant_name: ctx.sym("Cons"),
            fields: Rc::new(vec![
                (ctx.sym("head"), Value::Int(7)),
                (ctx.sym("tail"), empty),
            ]),
        };

        super::with_active_context(&ctx, || {
            // Held for the whole call -- forces free_monoid_ctx_syms's read-only
            // fallback rather than its mutable intern path.
            let _interner_guard = ctx.symbols.borrow();
            let items = super::free_monoid_to_vec(&cons)
                .expect("Cons/Empty chain must resolve under a held immutable interner borrow");
            assert_eq!(items, vec![Value::Int(7)]);
        });
    }

    /// The real-path discriminating pair for the in-flight boundary. Both arms evaluate the
    /// same compiled producer through `eval_pure_named_call` and cross the same 5ms CPU
    /// ceiling. Resolved-identity admission is the only changed fact: without it the ordinary
    /// entry budget fires; with it the refusal names the prospective shared-fill producer.
    #[test]
    fn admitted_in_flight_fill_crossing_is_typed_and_unadmitted_control_is_ordinary() {
        use super::InterpError;
        use crate::v1_compiler_compile::SourceFile;

        super::clear_cross_claim_pure_memos();
        let items = std::iter::repeat("0")
            .take(200_000)
            .collect::<Vec<_>>()
            .join(",");
        let result =
            crate::v1_compiler_compile::compile_to_resolved(Rc::new(im::vector![Rc::new(
                SourceFile {
                    path: "workspace/src/fill_budget_fixture.dag".to_string(),
                    content: format!(
                        "module fixture.fill_budget\nfn producer() -> List<Int> {{ [{items}] }}\n\
                     fn use_producer() -> List<Int> {{ producer() }}\n"
                    ),
                }
            ),]));
        let graph = result.graph.as_ref().expect("fixture graph");
        let fresh = || {
            InterpContext::new(
                graph,
                result.source_indices.clone(),
                ExecutionMode::Hermetic,
            )
        };

        let control = fresh();
        control.arm_eval_deadline(5);
        match super::run_in_context(&control, "fixture.fill_budget.use_producer", false) {
            Err(InterpError::EvaluationBudgetExceeded { .. }) => {}
            other => panic!("unadmitted control must keep the ordinary refusal: {other:?}"),
        }

        let admitted = fresh();
        let producer = admitted
            .lookup_fn_node("fixture.fill_budget.producer")
            .expect("fixture producer");
        super::install_cross_claim_pure_share_roster([producer]);
        admitted.arm_eval_deadline(5);
        match super::run_in_context(&admitted, "fixture.fill_budget.use_producer", false) {
            Err(InterpError::FillBudgetExceeded {
                producer,
                fill_cpu_nanos,
                limit_ms,
                ..
            }) => {
                assert_eq!(producer, "producer");
                assert!(fill_cpu_nanos >= 5_000_000, "fill CPU must cross 5ms");
                assert_eq!(limit_ms, 5);
            }
            other => panic!("admitted specimen must name its in-flight fill: {other:?}"),
        }
        super::clear_cross_claim_pure_memos();
    }
}

#[derive(Default)]
struct ParseTableMemo {
    map: HashMap<(String, String, i64, Symbol), Value>,
    keepalive: Vec<Value>,
}

// Recompute-trace ledger (diagnostic READ mode: reports, never gates — DESIGN §5 stopped-line
// audit). Counts evaluations of pure named fns (empty `uses` row) per (fn identity, argument
// identity). Keying is SOUND-ONLY: an argument without a cheap sound identity (composite
// values) goes to the unkeyed bucket — the ledger never merges distinct work. Durations
// include callees. Enabled via GUNBC_RECOMPUTE_TRACE=1.
#[derive(Default)]
struct EvalRecomputeTrace {
    map: std::collections::HashMap<EvalRecomputeKey, EvalRecomputeEntry>,
    // (calls, inclusive nanos, declaration site) per composite-argument producer. The nanos
    // half is here for the CROSS-CLAIM census below: a producer whose arguments have no cheap
    // sound identity is exactly as capable of costing a claim 300ms once as a nullary one, and
    // a bucket that counted calls without duration could name it and never rank it.
    unkeyed_by_fn: std::collections::HashMap<String, (u64, u128, String)>,
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
    // Content hashes of resolved symbol TEXT, not interner ordinals (see eval_recompute_str_hash
    // callers): ordinals are stable only within one SymbolInterner and this key must compare
    // equal across InterpContexts (gunbc#8505 follow-up: the cross-claim prepare_grammar memo).
    UnitVariant(u64, u64),
    EmptyList,
    // Recursive content hash of a composite value (Record/Variant/List/Map/Set/Fn/Unit),
    // memoized per allocation with Weak-liveness validation so a reused address never serves a
    // stale hash. Closures remain unkeyed (captured-env identity is not computed).
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
    // Distinct call-site node ptrs (capped) with "file:offset" labels. One site recomputing =
    // the same call expression re-evaluated (loop-invariant hoist or value coincidence —
    // Share/memoize territory, invisible to static analysis when value-coincident). Multiple
    // sites = cross-site duplicate demand (static rewire candidate).
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

/// Test harness only: re-read `GUNBC_RECOMPUTE_TRACE` into the process-wide cache, which
/// production initializes once per process; claim_executor's parallel tests set the env var
/// after siblings may have latched tracing off (review 45756).
#[doc(hidden)]
pub fn refresh_eval_recompute_trace_enabled_cache_for_tests() {
    eval_recompute_trace_refresh_cache();
}

// The eval-frame memo: the ladder's single-site discharge provider, realized in the seed.
// Buckets by the ledger key (fn identity x argument identity), serves only after the stored
// call's argument names AND values verify equal — a collision degrades to recompute, never a
// wrong value. Eviction is ScopeExit at the WITNESS frame: batch surfaces share one ctx across
// an entry's witnesses and call eval_call_memo_frame_exit after each claim fn (ctx-lifetime
// retention of argument+result values is byte-unbounded — the 2026-07-10 20GiB-class
// regression). Admission stops at the entry cap with the refusal COUNTED (overflow). Default
// ON everywhere; GUNBC_EVAL_MEMO=0 is a diagnostic realization switch (recompute instead of
// serve, semantics identical), and the receipt discloses hits/misses so a disabled memo shows
// as memo_hits=0, never assumed working.
struct EvalCallMemo {
    // Per-ctx realization switch (GUNBC_EVAL_MEMO read at ctx construction, not a
    // process-wide latch): provider-attribution tests pin the outer eval-frame provider off on
    // their own ctx so an inner provider's hit counters stay discriminating; semantics are
    // identical either way.
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

/// Realization switch, per ctx: an inner provider's by-execution receipt suite (e.g. the
/// parse-table MemoTier's amortization tests) pins the eval-frame provider off so pass-2
/// demands re-execute and the inner door's hit counters keep discriminating. Values are
/// identical either way.
pub fn set_eval_call_memo_enabled(ctx: &InterpContext, enabled: bool) {
    ctx.eval_call_memo.borrow_mut().enabled = enabled;
}

/// Frame exit for the eval-call memo: eviction scope is the WITNESS frame, not the ctx. Batch
/// surfaces (claim_batch, claim_executor) share one ctx across an entry's witnesses for the
/// resolve-side ReferenceTier share, but the memo stores full argument+result VALUES, so
/// ctx-lifetime retention across N witnesses is byte-unbounded (measured 2026-07-10: one
/// witness plateaus ~3.4GiB, six in one ctx pass ~20GiB to SIGKILL). Called after each claim
/// function; map and keepalives drain, counters stay CUMULATIVE so receipts remain honest.
/// Cross-witness serving is an outer-frame promotion that must arrive as a conscious provider
/// row with byte-bounded admission — never a default.
pub fn eval_call_memo_frame_exit(ctx: &InterpContext) {
    let mut m = ctx.eval_call_memo.borrow_mut();
    m.map.clear();
    m.keepalive_fns.clear();
}

#[derive(Default, Clone)]
pub struct MutationCounters {
    pub map_insert_calls: u64,
    pub map_merge_calls: u64,
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
        // The map rows report calls only: no entries-copied quantity is incremented, and a
        // literal 0 would read as a measurement. The quantity is well defined (rc_map_merge
        // clones every overlay entry, so it is the overlay's size) but never computed.
        writeln!(
            f,
            "  {:<12} {:>12} calls",
            "map_insert", self.map_insert_calls
        )?;
        writeln!(
            f,
            "  {:<12} {:>12} calls",
            "map_merge", self.map_merge_calls
        )?;
        let rows: [(&str, u64, u64); 4] = [
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
/// These indexes are a pure function of the module population, so building them belongs to
/// preparing a scope, not running a claim.
///
/// The naive "fresh context per claim so witnesses cannot contaminate each other" rebuilt ALL
/// of this per claim: on the required floor, 9,573 reconstructions of maps that only 1,155
/// distinct scopes can differ in — the entry-major cost shape reproduced one layer below the
/// compiler after the compiler's own copy was removed. Fresh state per claim is correct; fresh
/// INDEXES per claim is the same defect wearing the word "fresh".
/// ONE MODULE'S CONTRIBUTION TO A SCOPE'S INDEXES, derived from that module alone.
///
/// WHY THIS EXISTS. `build_scope_indexes_with_module_order` used to re-read every item of
/// every module of every scope: on the required floor that is 660 scopes over a mean of 427
/// modules each, and the derivation per item — `authored_name_at`, `item_kind`, the import
/// list, a `format!` per qualified name — was repaid once per scope the module appeared in.
/// The measured cost was 44.5s of a 75.3s scope-construction total (`[floor-scope-split]`).
///
/// The derivation reads nothing but the module, so its result is the same in every scope that
/// contains it. What is NOT module-local is which module WINS a colliding bare name, and that
/// is exactly what stayed in the per-scope fold: the fragment carries the module's entries in
/// the module's own order, and the fold applies the scope's precedence to them.
///
/// ORDER IS PRESERVED ACROSS THE TWO KINDS, not grouped by kind: the original walk emitted a
/// bare slot and then its qualified slot per item, and grouping would change which write lands
/// last if a bare name ever spells a qualified one.
pub struct ModuleScopeFragment {
    fn_entries: Vec<FragmentFnEntry>,
    file_module_paths: Vec<(String, String)>,
    file_import_bindings: Vec<((String, String), String)>,
    service_ops: Vec<(String, ServiceOp)>,
}

/// A `fn_nodes` write, tagged with which slot it targets. The bare slot is subject to the
/// scope's precedence rule (first-write-wins under an order); the qualified slot names exactly
/// one declaration and is written unconditionally, as the original walk did.
enum FragmentFnEntry {
    Bare { name: String, item: Rc<Node> },
    Qualified { name: String, item: Rc<Node> },
}

/// Per-prepared-subject memo for [`ModuleScopeFragment`], keyed by the module's authored name.
///
/// Optional at every call site: passing `None` derives each fragment on the spot, which is what
/// the entry-major (`build_scope_indexes`) path does. A cache is a caller's fact about how many
/// scopes it is about to build over ONE subject, not a property of the fold.
pub type ScopeFragmentCache = std::cell::RefCell<HashMap<String, Rc<ModuleScopeFragment>>>;

fn module_scope_fragment(
    module: &Rc<TypedModule>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    cache: Option<&ScopeFragmentCache>,
) -> Rc<ModuleScopeFragment> {
    if let Some(cache) = cache {
        if let Some(hit) = cache.borrow().get(module.func_env.name.as_str()) {
            return hit.clone();
        }
    }
    let built = Rc::new(derive_module_scope_fragment(module, source_indices));
    if let Some(cache) = cache {
        cache
            .borrow_mut()
            .insert(module.func_env.name.clone(), built.clone());
    }
    built
}

/// The module-local derivation, lifted VERBATIM out of the per-scope walk it used to sit in.
fn derive_module_scope_fragment(
    module: &Rc<TypedModule>,
    source_indices: &Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> ModuleScopeFragment {
    let module_path = authored_name_at(source_indices.clone(), module.module.clone());
    let mut fn_entries: Vec<FragmentFnEntry> = Vec::new();
    let mut file_module_paths: Vec<(String, String)> = Vec::new();
    let mut file_import_bindings: Vec<((String, String), String)> = Vec::new();
    let mut service_ops: Vec<(String, ServiceOp)> = Vec::new();
    // WHAT THE AUTHOR WROTE ABOUT WHERE EACH NAME COMES FROM, read once per module from
    // the parser's import list. FIRST WRITE WINS within a file: two imports of one
    // spelling from different modules is a double bind the grammar does not yet refuse,
    // and picking the later would make this tier depend on statement order exactly as it
    // exists to stop depending on walk order. The or_insert that enforces it stays in the
    // fold, so the rule still spans every module in the scope and not just this one.
    for imp in crate::v1_std_core::module_imports(module.module.clone()).iter() {
        if import_is_all(imp.clone()) {
            continue;
        }
        let source_module = authored_name_at(source_indices.clone(), imp.clone());
        if source_module.is_empty() {
            continue;
        }
        for imported in import_specific_names_at(imp.clone(), source_indices.clone()).iter() {
            if imported.is_empty() || module.module.span.file.is_empty() {
                continue;
            }
            file_import_bindings.push((
                (module.module.span.file.to_string(), imported.clone()),
                source_module.clone(),
            ));
        }
    }
    for item in module.items.iter() {
        let name = authored_name_at(source_indices.clone(), item.clone());
        if !module_path.is_empty() && !item.span.file.is_empty() {
            file_module_paths.push((item.span.file.to_string(), module_path.clone()));
        }
        if !name.is_empty() {
            fn_entries.push(FragmentFnEntry::Bare {
                name: name.clone(),
                item: item.clone(),
            });
            if !module_path.is_empty() {
                fn_entries.push(FragmentFnEntry::Qualified {
                    name: format!("{}.{}", module_path, name),
                    item: item.clone(),
                });
            }
        }
        // Service-item detection is node-local: the node carries the `transport` that
        // *defines* it as a service, so its own `item_kind` is the single authority.
        // Do NOT gate on a name-keyed `item_registry` lookup — two top-level items can
        // share a name (`std.resources` `resource Filesystem` is an OtherItem;
        // `extdeps.filesystem` `service Filesystem` is a ServiceItem), and in one import
        // closure the non-service entry can win the registry merge and silently drop the
        // service's operations (-> "unknown service operation" at runtime).
        if item_kind(item.clone()) == ItemKind::ServiceItem {
            for op in item.children.iter() {
                let op_name = authored_name_at(source_indices.clone(), op.clone());
                if op_name.is_empty() {
                    continue;
                }
                if !name.is_empty() {
                    service_ops.push((format!("{}.{}", name, op_name), (item.clone(), op.clone())));
                }
                if !item.name.is_empty() && item.name != name {
                    service_ops.push((
                        format!("{}.{}", item.name, op_name),
                        (item.clone(), op.clone()),
                    ));
                }
            }
        }
    }
    ModuleScopeFragment {
        fn_entries,
        file_module_paths,
        file_import_bindings,
        service_ops,
    }
}

pub struct PreparedScopeIndexes {
    pub modules: Rc<im::Vector<Rc<TypedModule>>>,
    pub item_registry: Rc<HashMap<String, Rc<ItemInfo>>>,
    pub source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    pub emit_graph_info: Rc<EmitGraphInfo>,
    fn_nodes: HashMap<String, Rc<Node>>,
    ambiguous_bare_function_names: std::collections::HashSet<String>,
    /// SOURCE FILE -> DECLARING MODULE PATH, for the one lookup tier that needs to know where a
    /// reference was authored.
    ///
    /// `fn_nodes` holds one bare slot per name and one qualified slot per declaration, so two
    /// modules declaring `section_1` fight over the bare slot and the loser's callers execute
    /// the winner's body. This map lets a reference reach ITS OWN module's declaration first --
    /// the `local`-before-`parents` order `v1_compiler_infer_sigs::lookup_resolved_sig` applies
    /// at typecheck, which is why the collapse was silent: the type layer resolved per module,
    /// the runtime did not.
    file_module_paths: HashMap<String, String>,
    /// (SOURCE FILE, IMPORTED NAME) -> THE MODULE PATH THAT FILE IMPORTED THE NAME FROM.
    ///
    /// The braced list of `import a.b.c { x, y }` survives on the module node's `params`, so
    /// WHERE the author said a name comes from is recoverable rather than guessed from
    /// precedence. It cannot come from `ResolvedFuncEnv.parents`, the FLATTENED TRANSITIVE
    /// closure: that keeps module identity only, drops the per-import name list, and does not
    /// separate a direct import from a transitively reachable module — a first-hit fold over it
    /// is the same silent pick one level out.
    ///
    /// Wildcard (`import a.b.c` with no braces) binds no names and contributes nothing here.
    file_import_bindings: HashMap<(String, String), String>,
    service_ops: HashMap<String, ServiceOp>,
}

impl PreparedScopeIndexes {
    /// EVERY RESOLUTION THIS INDEX SET CAN ANSWER, rendered at identity grain and sorted, so
    /// two index sets can be compared for equality of ANSWERS rather than of construction path.
    /// Items are identified by `Rc` address: the same declaration node, not merely an equal
    /// spelling — a fold that resolved a colliding bare name to the other module's declaration
    /// differs here even though every key is identical.
    #[cfg(any(test, feature = "interp_test_witness"))]
    pub fn resolution_fingerprint(&self) -> Vec<String> {
        let mut lines: Vec<String> = Vec::new();
        for (name, item) in self.fn_nodes.iter() {
            lines.push(format!("fn\t{}\t{:p}", name, Rc::as_ptr(item)));
        }
        for name in self.ambiguous_bare_function_names.iter() {
            lines.push(format!("ambiguous\t{}", name));
        }
        for (file, module) in self.file_module_paths.iter() {
            lines.push(format!("file_module\t{}\t{}", file, module));
        }
        for ((file, imported), source) in self.file_import_bindings.iter() {
            lines.push(format!("import_bind\t{}\t{}\t{}", file, imported, source));
        }
        for (key, (item, op)) in self.service_ops.iter() {
            lines.push(format!(
                "service_op\t{}\t{:p}\t{:p}",
                key,
                Rc::as_ptr(item),
                Rc::as_ptr(op)
            ));
        }
        for (name, info) in self.item_registry.iter() {
            lines.push(format!("registry\t{}\t{:p}", name, Rc::as_ptr(info)));
        }
        lines.sort();
        lines
    }
}

thread_local! {
    /// How many times the immutable index set has been constructed. The acceptance bar is
    /// `full interpreter index constructions <= distinct prepared scope identities`; this
    /// counter makes the per-claim rebuild observable rather than inferred from a profile.
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
    // Parameter-name derivation is invariant per fn_node but was re-sliced from source spans
    // per call (authored_name_at). Memoized per fn_node pointer. The pointer alone is unsound:
    // the ctx does not own fn_nodes (borrowed `Rc<Node>`s droppable while the ctx lives), so a
    // freed address can be reused and collide. keepalive_fns retains the `Rc<Node>` behind each
    // key for the ctx's lifetime (as PureCallMemo.keepalive_fns / EvalRecomputeTrace.keepalive_fns
    // / EvalCallMemo.keepalive_fns), and the cache dies with the ctx (as data_cache).
    // Value = (filtered named-param list, all-param list), matching call_function's two uses.
    param_name_cache: std::cell::RefCell<HashMap<usize, Rc<(Vec<String>, Vec<String>)>>>,
    param_name_cache_keepalive: std::cell::RefCell<Vec<Rc<Node>>>,
    // Same chokepoint, ExprVar arm: eval_var rebuilt the name String from its span
    // (expr_var_name_at) and re-interned it (ctx.sym) per read. The interned Symbol is memoized
    // per ExprVar node pointer, kept alive via var_sym_cache_keepalive as param_name_cache above;
    // eval goes straight to env.lookup(sym), materializing the String only on the registry slow path.
    var_sym_cache: std::cell::RefCell<HashMap<usize, Symbol>>,
    var_sym_cache_keepalive: std::cell::RefCell<Vec<Rc<Node>>>,
    // Same chokepoint, ExprCall callee name: eval_call re-sliced the callee name from its source
    // span (expr_call_func_at -> authored_name_at) on every call. Memoize the decoded name per
    // call node — keyed by node pointer, kept alive via call_func_name_cache_keepalive as above.
    call_func_name_cache: std::cell::RefCell<HashMap<usize, String>>,
    call_func_name_cache_keepalive: std::cell::RefCell<Vec<Rc<Node>>>,
    // Same chokepoint, ExprCast arm, missed by the three caches above. A cast resolved its
    // target per EVALUATION: cast_target_seed_name re-sliced source (authored_name_at), and for
    // any target not named "String" the alias-chain walk called lookup_type_item_across_modules,
    // which SCANS every item of every module extracting source text per comparison — per hop, up
    // to 32 hops. Both names are pure functions of target node + module set, fixed per ctx, so
    // they are memoized per target node pointer, kept alive via cast_kernel_cache_keepalive as
    // call_func_name_cache above.
    cast_kernel_cache: std::cell::RefCell<HashMap<usize, Rc<CastTargetNames>>>,
    cast_kernel_cache_keepalive: std::cell::RefCell<Vec<Rc<Node>>>,
    // The alias walk's per-hop `lookup_type_item_across_modules` was a LINEAR SCAN over every
    // item of every module, extracting source text per comparison. One daily-page render: 700
    // lookups scanned 1,967,155 items (~2,810 each), essentially all of ExprCast's 2,027ms — and
    // the term grows with the closure, not the request. A name->item map is the same fact
    // indexed, built once per ctx. `or_insert` preserves the scan's first-match-wins order.
    type_item_index: std::cell::RefCell<Option<Rc<HashMap<String, Rc<Node>>>>>,
    // The cast's SOURCE-side name, same class as cast_kernel_cache above (see
    // cast_expr_inferred_type_name).
    cast_source_name_cache: std::cell::RefCell<HashMap<usize, String>>,
    cast_source_name_cache_keepalive: std::cell::RefCell<Vec<Rc<Node>>>,
    pure_call_memo: std::cell::RefCell<PureCallMemo>,
    parse_table_memo: std::cell::RefCell<ParseTableMemo>,
    eval_recompute_trace: std::cell::RefCell<EvalRecomputeTrace>,
    eval_call_memo: std::cell::RefCell<EvalCallMemo>,
    // Effect-dispatch odometer, incremented per service-operation dispatch. The eval-call memo
    // compares it across a named call and refuses to memoize any call during which it advanced
    // — a WorldRead/effect is never served stale (the uses-empty purity gate is vacuous
    // corpus-wide: no corpus func declares `uses`, so every effectful wrapper was memo-eligible;
    // found via the artifact-store List-after-Delete staleness).
    effect_dispatch_count: std::cell::Cell<u64>,
    eval_recompute_hash_memo: std::cell::RefCell<EvalRecomputeHashMemo>,
    mutation_counters: std::cell::RefCell<MutationCounters>,
    symbols: RefCell<SymbolInterner>,
    published_mock_keys: RefCell<Option<Rc<std::collections::HashSet<String>>>>,
    whole_tree_published_keys: Option<Rc<std::collections::HashSet<String>>>,
    governed_services: RefCell<Option<Rc<std::collections::HashSet<String>>>>,
    // Cooperative per-witness eval deadline (operator ruling 2026-08-17; ceiling supplied by the
    // caller from `v2.workflow.required_floor` `required_floor_claim_cpu_safety_limit_ms`, the
    // CPU safety deadline — independent of the wall deadline arming the sibling clock below, per
    // the 2026-08-19 budget policy cut's superseding correction).
    // It must unwind from INSIDE eval as a typed error: witness evals run on in-process worker
    // threads with no kill authority, so an outside wall-clock bound cannot terminate them (the
    // Phase A governor lesson). Denominated in THREAD CPU TIME, not wall: the fast-lane rule
    // targets the eval-wedge (a non-terminating eval burning a core), so a witness inflated by
    // cold I/O or governor time-slicing must not be misclassified — "assuming the infra isn't
    // the problem" is the CPU-vs-wall gap. Stored pair: (cpu_baseline_nanos, budget_ms).
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
    // Shell waits poll this and SIGKILL the process group at the ceiling; the completion-side
    // `wall_budget_completion_outcome` remains a backstop for non-subprocess spend. Without it
    // the refusal fires only after the overrun is spent (707s on a 600s budget; 21–34min
    // receipts in the original finding).
    witness_wall_deadline: std::cell::Cell<Option<(Instant, u64)>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectedFunctionIdentity {
    pub module_path: String,
    pub decl_name: String,
    pub bare_name_ambiguous: bool,
}

/// The module path a source file authors, or `None` when the index cannot name exactly one.
/// Public for the FLOOR2 qualified-witness lookup: under one shared prepared subject a witness
/// is invoked by `module.function`, and re-deriving the mapping in the caller would fork this.
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
        Self::build_scope_indexes_with_module_order(graph, source_indices, None, None)
    }

    /// Same walk as [`build_scope_indexes`], but with `module_order` present modules are visited
    /// in that precedence and bare `fn_nodes` keys are first-write-wins — the resolution
    /// `claim_scope_for` applies to `item_registry`. Without an order the walk follows
    /// `graph.modules` and bare keys stay last-write-wins for entry-major callers.
    ///
    /// THIS IS STILL NAME-BASED RESOLUTION WITH A PRECEDENCE RULE, not a wall. An entry module
    /// wins its own colliding helper, making the compute_board `refusal_is` theft unwritable
    /// for that caller.
    ///
    /// AND PRECEDENCE NO LONGER DECIDES A NON-ENTRY MODULE'S OWN REFERENCES. The former
    /// "a non-entry homonym in the same scope still binds by order" state emitted one plan
    /// document carrying another's entire body: two carriers declaring `section_1` ..
    /// `section_9`, one bare slot, the loser's body fold executing the winner's sections.
    /// [`InterpContext::lookup_fn_from`] now resolves to the referring file's OWN module first
    /// whenever a bare name is claimed by more than one module. The named residue: a bare
    /// reference to a name the referring module does NOT declare, claimed by two OTHER modules,
    /// is still picked by order with nothing said.
    ///
    /// Next rung: DESIGN §3 namespace-only — a qualified reference has exactly one declarer, so
    /// ambiguous bare binding has no constructor (`floor_bare_name_ambiguity_next_rung_trigger`).
    pub fn build_scope_indexes_with_module_order(
        graph: &ResolvedGraph,
        source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
        module_order: Option<&[String]>,
        fragments: Option<&ScopeFragmentCache>,
    ) -> Rc<PreparedScopeIndexes> {
        SCOPE_INDEX_CONSTRUCTIONS.with(|c| c.set(c.get() + 1));
        let first_write_wins = module_order.is_some();
        let modules_to_walk: Vec<&Rc<crate::v1_compiler_infer_items::TypedModule>> =
            match module_order {
                Some(order) => {
                    let by_name: HashMap<&str, &Rc<crate::v1_compiler_infer_items::TypedModule>> =
                        graph
                            .modules
                            .iter()
                            .map(|m| (m.func_env.name.as_str(), m))
                            .collect();
                    // Walk `order`, not the graph. `claim_scope_for` built this graph's
                    // `modules` from the same `order` (`in_scope` is that list), so the
                    // filter_map cannot drop a scoped member.
                    order
                        .iter()
                        .filter_map(|name| by_name.get(name.as_str()).copied())
                        .collect()
                }
                None => graph.modules.iter().collect(),
            };
        let mut fn_nodes = HashMap::new();
        let mut bare_name_counts = HashMap::<String, usize>::new();
        let mut file_module_paths = HashMap::<String, String>::new();
        let mut file_import_bindings = HashMap::<(String, String), String>::new();
        let mut service_ops = HashMap::new();
        for module in modules_to_walk {
            let fragment = module_scope_fragment(module, &source_indices, fragments);
            for ((file, imported), source_module) in fragment.file_import_bindings.iter() {
                file_import_bindings
                    .entry((file.clone(), imported.clone()))
                    .or_insert_with(|| source_module.clone());
            }
            for (file, module_path) in fragment.file_module_paths.iter() {
                file_module_paths.insert(file.clone(), module_path.clone());
            }
            for entry in fragment.fn_entries.iter() {
                match entry {
                    FragmentFnEntry::Bare { name, item } => {
                        *bare_name_counts.entry(name.clone()).or_default() += 1;
                        if first_write_wins {
                            fn_nodes.entry(name.clone()).or_insert(item.clone());
                        } else {
                            fn_nodes.insert(name.clone(), item.clone());
                        }
                    }
                    FragmentFnEntry::Qualified { name, item } => {
                        fn_nodes.insert(name.clone(), item.clone());
                    }
                }
            }
            for (key, op) in fragment.service_ops.iter() {
                service_ops.insert(key.clone(), op.clone());
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
            file_module_paths,
            file_import_bindings,
            service_ops,
        })
    }

    /// Join shared immutable indexes with FRESH MUTABLE STATE — the per-claim constructor,
    /// cheap by construction: clones `Rc` handles, allocates empty caches, walks no module.
    ///
    /// Every mutable field is fresh, not shared: memos, name caches, the effect odometer and the
    /// deadline arms carry state from the previous claim, and sharing them is how one witness's
    /// evaluation becomes another's answer.
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
            symbols: RefCell::new({
                let mut interner = SymbolInterner::default();
                for s in FREE_MONOID_WELL_KNOWN_SYMS {
                    interner.intern(s);
                }
                interner
            }),
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
            .set(Some((budgeted_cpu_nanos(), budget_ms)));
        self.eval_deadline_stride.set(0);
    }

    pub fn clear_eval_deadline(&self) {
        self.eval_deadline.set(None);
    }

    /// Milliseconds left on the armed CPU deadline, or `None` when none is armed.
    /// `Some(0)` means already past.
    pub fn eval_deadline_remaining_ms(&self) -> Option<u64> {
        let (baseline, budget_ms) = self.eval_deadline.get()?;
        let elapsed_ms = (budgeted_cpu_nanos().saturating_sub(baseline) / 1_000_000) as u64;
        Some(budget_ms.saturating_sub(elapsed_ms))
    }

    /// The entry identity the currently-armed budget belongs to, for the neutral result.
    pub fn budget_entry(&self) -> Option<String> {
        self.budget_entry.borrow().clone()
    }

    /// Enter a scoped evaluation budget that can only ever TIGHTEN what is already armed.
    ///
    /// The paired `arm_*` / `clear_*` calls compose wrongly, in the fail-open direction
    /// (verified 2026-08-09): `arm_eval_deadline` sets its cell unconditionally with a FRESH
    /// baseline, so a nested arm restarts the clock and grants the outer evaluation a new budget;
    /// `clear_*` sets `None` rather than restoring, so an inner clear disarms an outer deadline.
    /// Both silently.
    ///
    /// The guard fixes composition once: the effective limit is the smaller of what REMAINS on
    /// the outer deadline and what this scope requests (remaining, not declared — equal declared
    /// limits armed at different instants have different time left, which decides which fires
    /// first), and every exit path restores the displaced state because `Drop` runs on early
    /// return and unwind. A leaked deadline is worse than an absent bound: its CPU baseline is
    /// captured at arm time, so surviving into a later evaluation it measures against a baseline
    /// already spent and refuses immediately — on `gunbc serve`, sharing one `InterpContext`
    /// across requests, every subsequent request for the life of the process.
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
                .set(Some((budgeted_cpu_nanos(), effective)));
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

    /// Entry identity for a budget result. `arm_*` callers setting no entry (the witness lane,
    /// which supplies the witness name in its own refusal) get an explicit placeholder, not an
    /// empty string, so an unnamed entry is visibly unnamed rather than a read empty name.
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

    /// Is this bare spelling claimed by more than one module in this scope?
    pub(crate) fn bare_name_is_ambiguous(&self, name: &str) -> bool {
        self.indexes.ambiguous_bare_function_names.contains(name)
    }

    /// [`lookup_fn`], but a BARE name claimed by more than one module resolves through the
    /// referring file's own declarations, then its explicit imports, before the shared bare slot
    /// — the `local`-then-declared-source order the type layer applies in `lookup_resolved_sig`.
    /// The two layers DISAGREEING made this class silent: typecheck bound per module, execution
    /// rebound globally, nothing refused, the program ran a different function.
    ///
    /// The bare slot is one map entry owned by the last-visited module (first, under precedence);
    /// every other module's references to that spelling execute the winner's body. Not authored
    /// shadowing -- a module losing its own declaration to a homonym. It produced
    /// `docs/plans/import-namespace-program.md` carrying `v2-corpus-self-host`'s entire body,
    /// both plan carriers declaring `section_1` .. `section_9` and `status_block`.
    ///
    /// THE TIER IS GATED ON `ambiguous_bare_function_names`: every name exactly one module
    /// declares resolves byte-for-byte as before; only the colliding population moves, toward
    /// the module that authored the reference.
    ///
    /// WHY THE IMPORT TIER IS NOT OPTIONAL (established by a floor red, not reasoning):
    /// `test.claim.ilm4926_designation_witness` imports `extdeps_external_authority_anchor`
    /// explicitly from `extdeps.cpu_attachment.ilm4926` — a name 630 modules declare, one per
    /// extdeps module by convention. With the own-module tier alone the import was inert: the
    /// reference landed on `extdeps.vendor.lotes` (imported one line earlier, winning the shared
    /// slot) while the attestation row inside `ilm4926` reached ilm4926's own; row and query
    /// disagreed and the filter returned empty. Before the own-module tier BOTH sides read lotes,
    /// so the witness was green on a query that meant nothing.
    ///
    /// THIS IS NOT THE NAMESPACE PROGRAM. Two residues stay on the shared slot, picked by
    /// precedence with nothing said: a bare reference to a name the referring module neither
    /// declares nor imports, and a name reached through a WILDCARD import (binds no names, says
    /// nothing about origin). That residue is `PreparedClaimScope::ambiguous_bare_names`'s
    /// subject and retires with namespace-only resolution, where a reference has exactly one
    /// declarer by construction.
    fn lookup_fn_from(&self, name: &str, site_file: &str) -> Option<&Rc<Node>> {
        if !name.contains('.')
            && !site_file.is_empty()
            && self.indexes.ambiguous_bare_function_names.contains(name)
        {
            if let Some(module_path) = self.indexes.file_module_paths.get(site_file) {
                let qualified = format!("{}.{}", module_path, name);
                if let Some(node) = self.indexes.fn_nodes.get(&qualified) {
                    return Some(node);
                }
            }
            // THEN WHERE THE AUTHOR SAID IT COMES FROM: an explicitly imported name resolves to
            // the module it was imported FROM, a fact the shared slot discards. Without this tier
            // `import a.b.c { anchor }` is inert whenever another module in scope declares
            // `anchor` -- the reference lands on the precedence winner and the import line reads
            // as though it decided something.
            if let Some(source_module) = self
                .indexes
                .file_import_bindings
                .get(&(site_file.to_string(), name.to_string()))
            {
                let qualified = format!("{}.{}", source_module, name);
                if let Some(node) = self.indexes.fn_nodes.get(&qualified) {
                    return Some(node);
                }
            }
        }
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

/// Pre-evaluate module-level `data` into the base environment.
///
/// A NAME CLAIMED BY MORE THAN ONE MODULE IS DELIBERATELY NOT PRE-BOUND — that is what makes
/// [`InterpContext::lookup_fn_from`] reachable for `data`. This binds by BARE name via the
/// global `lookup_fn`, and `eval_var` consults the environment BEFORE the item registry, so a
/// pre-bound ambiguous name shadows per-module resolution everywhere — the `fn_nodes`
/// single-slot collapse one layer earlier, disguised as a variable lookup succeeding. Measured:
/// with the tiers in place and this loop unchanged, a `data` reference explicitly imported from
/// one of two declarers still read the other's value, while the function form resolved
/// correctly — functions are not pre-bound here.
///
/// Skipping them is unobservable: the reference falls through to `eval_var`'s registry path,
/// resolving per module and memoizing via `data_cache`; a name ONE module declares keeps this
/// fast path. Locals live in extended scopes and shadow lexically either way.
fn build_initial_env(ctx: &InterpContext) -> InterpResult<Rc<Env>> {
    let mut bindings = HashMap::new();
    for (name, info) in ctx.item_registry.iter() {
        if ctx.bare_name_is_ambiguous(name) {
            continue;
        }
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

/// Bounded execution (§4): a call chain deeper than this is a typed, located refusal naming
/// the frontier function — never a host stack overflow, which aborts the process and every
/// later witness's measurement (measured: a cycle in live_deploy script assembly under
/// census-resolved bare names killed batch-2 at entry 214/619). Deep-but-terminating recursion
/// lives under `stacker::maybe_grow`; 100_000 frames is far past any legitimate corpus chain.
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
                // A caller label naming no declared parameter is a contract mismatch, not an
                // extra binding: it would shadow nothing while the real parameter stays unbound.
                // Refuse here (typed, located) rather than fail later as `NoSuchVariable` — or
                // silently compute when the stray label collides with an in-scope name.
                // The corpus marks an unused parameter with a leading underscore (`_ctx`,
                // `_spelling`) or `_`, and call sites label it WITHOUT the underscore
                // (`bash_fold_stmt_kind_tag_emit_transform(spelling: ..)` against
                // `(_spelling: String)`; the fold-step `(acc, _: Edge, child)` labelled `e:`).
                // Not a mismatch: the body cannot read it, so nothing is dropped. Accept `x`
                // against a declared `x`, `_x`, or `_`.
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
                // HashMap::insert, losing the earlier value unlocatably (DESIGN §5: the
                // compile-side wall must not report a fact the runtime keeps quiet about).
                // Refuse instead of taking the last value.
                //
                // An anonymous key ("_") is excluded from the collision check but still inserted:
                // the body cannot read "_", so two anonymous parameters are distinct unreadable
                // slots, not a collision, and the insert keeps the required-argument "supplied"
                // check below (`bindings.contains_key`) seeing it as filled (review from parent
                // session loyal-ant-382, 2026-08-05 — the prior form keyed AND refused every
                // anonymous param under the literal "_", false-refusing two or more of them).
                if name != "_" && bindings.contains_key(&ctx.sym(name)) {
                    return Err(InterpError::CallContractMismatch {
                        callee: fn_node.name.clone(),
                        detail: format!("argument '{}' supplied more than once", name),
                    });
                }
                bindings.insert(ctx.sym(name), val.clone());
            } else if positional_idx < param_names.len() {
                // A positional actual is keyed by its resolved declared parameter, as the named
                // branch is — so one filling a parameter an earlier named actual bound
                // (`two(a: 1, 2)` against `fn two(a, b)`: slot 0 is `a`, already bound) must
                // refuse the same way, not overwrite last-write-wins (DESIGN §5 fail-closed;
                // review 48817).
                //
                // An anonymous declared parameter ("_") is excluded from the collision check (see
                // the named-branch note): two anonymous slots are distinct and unreadable. The
                // insert still happens so the required-argument check below sees it filled.
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

/// Thread CPU time in nanoseconds — the fast-lane eval budget's metric. Advances only while
/// THIS thread runs on a core, so it excludes blocking-I/O waits (a cold live-tree read) and
/// scheduler time-slicing (many witnesses sharing cores under the adaptive governor) — the
/// "assuming the infra isn't the problem" clause of the operator's eval-budget ruling: a
/// non-terminating eval burns CPU and is caught; a bounded scan with infra-inflated WALL time
/// is not misclassified. Unix reads `CLOCK_THREAD_CPUTIME_ID`; elsewhere (dev only — CI is
/// linux) a process-monotonic wall clock. A clock error yields 0, so the deadline under-counts
/// rather than fires spuriously (the witness still returns its real Pass/Fail; the budget is a
/// performance guard, not a correctness gate).
/// Maps the kernel's caller-agnostic budget result into the WITNESS lane's refusal vocabulary.
///
/// The kernel raises `EvaluationBudgetExceeded` for every caller (see that variant for why it
/// must not raise a witness concept). The witness lane's diagnostics carry operator rulings its
/// consumers depend on — the 5s fast-lane rule, and the "relocating the file does not discharge
/// it" guidance that exists because a witness was once re-homed under `long/` to silence this
/// error — so that text stays at the witness boundary, neither leaking into an HTTP response
/// nor deleted.
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

thread_local! {
    /// CPU SPENT FILLING SHARED MEMOIZED ARTIFACTS ON THIS THREAD. Lives here, not beside the
    /// memos, because it has TWO readers with opposite lifetimes: the claim loop nets it at
    /// completion, the evaluation deadline WHILE the claim runs. A counter with two homes is the
    /// §3 failure this work closes, so there is one cell and `cli_run` delegates to it.
    static SHARED_ARTIFACT_FILL_CPU_NANOS: std::cell::Cell<u128> = const { std::cell::Cell::new(0) };
}

thread_local! {
    /// EVALUATOR STEPS TAKEN ON THIS THREAD — one per `eval_expr` entry, counted
    /// UNCONDITIONALLY. This is a WORK measure, not a time measure: its value is a property of
    /// the program and the arguments it was evaluated on, and it carries no term for clock
    /// frequency, cache state, co-tenant pressure or scheduler preemption. That is the whole
    /// point — `gunbc.rung_drop` `floor_cost_claim_qualification_unavailable` names "a deterministic work
    /// measure such as evaluator steps" as one of the three arms of its restoration trigger,
    /// and a measure that is only available under a profiling flag is not available in the
    /// envelopes the row is about. So this counter is NOT gated on `eval_profile_enabled`: the
    /// per-variant `EVAL_COUNTS` array beside it is a profiling instrument that pays two
    /// `Instant::now()` calls per node, while this is a single wrapping increment.
    ///
    /// IT IS A COUNT AND NOT YET A VERDICT. Nothing compares it against a line, and this
    /// declaration deliberately does not introduce one: there is no calibrated step ceiling in
    /// the tree, and inventing one would be the "looks principled and is not" threshold the
    /// same row already refuses on the calibration arm.
    static EVAL_STEPS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };

    /// Evaluator steps taken inside STORED shared-artifact fills, netted by exactly the rule
    /// `SHARED_ARTIFACT_FILL_CPU_NANOS` nets CPU by. Without this the measure would not be
    /// envelope-invariant after all: a claim that happened to be the one to fill a shared memo
    /// would count the fill's steps while every later claim reading the same artifact counted
    /// none, so the value would be a function of EXECUTION ORDER — the same defect the
    /// 2026-08-27 fill-attribution ruling closed on the CPU clock, at the same grain.
    static SHARED_ARTIFACT_FILL_EVAL_STEPS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// One evaluator step. Called from `eval_expr` and from nowhere else, so the count has one
/// producer and its unit is stated by that single call site rather than by convention.
fn record_eval_step() {
    EVAL_STEPS.with(|c| c.set(c.get().wrapping_add(1)));
}

/// The running evaluator-step total for this thread. Deltas across a claim, never the absolute.
pub fn evaluator_steps() -> u64 {
    EVAL_STEPS.with(|c| c.get())
}

/// The running total of steps taken inside stored shared-artifact fills on this thread.
pub fn shared_artifact_fill_eval_steps() -> u64 {
    SHARED_ARTIFACT_FILL_EVAL_STEPS.with(|c| c.get())
}

fn record_shared_artifact_fill_eval_steps(steps: u64) {
    SHARED_ARTIFACT_FILL_EVAL_STEPS.with(|c| c.set(c.get().wrapping_add(steps)));
}

/// Accumulate CPU spent filling a shared memoized artifact. Called only from a memo MISS path.
pub fn record_shared_artifact_fill_cpu_nanos(nanos: u128) {
    SHARED_ARTIFACT_FILL_CPU_NANOS.with(|c| c.set(c.get().saturating_add(nanos)));
}

/// The running fill total for this thread.
pub fn shared_artifact_fill_cpu_nanos() -> u128 {
    SHARED_ARTIFACT_FILL_CPU_NANOS.with(|c| c.get())
}

fn in_flight_cross_claim_fill(raw_cpu_nanos: u128) -> Option<(String, u128)> {
    CROSS_CLAIM_FILL_FRAMES.with(|frames| {
        frames.borrow().first().map(|outermost| {
            (
                outermost.producer.clone(),
                raw_cpu_nanos
                    .saturating_sub(outermost.cpu_started)
                    .saturating_sub(outermost.stored_children_cpu),
            )
        })
    })
}

/// THE CLOCK EVERY CPU BUDGET IS MEASURED ON: thread CPU, less what this thread spent filling
/// shared artifacts that every later claim naming the same source reads free.
///
/// WHY A CLOCK RATHER THAN A BASELINE THREADED THROUGH EACH DEADLINE: arming and polling
/// instants both read this function, so fill accrued between them cancels in the subtraction
/// every site already does — no second baseline, and no site can forget to net.
///
/// MONOTONE, as a deadline requires: fill is measured on this same thread clock inside the miss
/// path, so thread CPU rises at least as much as fill over any interval. The saturating
/// subtraction covers only sampling skew between the two reads.
///
/// WHAT THIS DOES NOT REACH: the WALL deadline (`witness_wall_deadline`) is still armed on a raw
/// `Instant` and still charges a fill to whichever claim paid it. Every interruption in the
/// repaired population was on the CPU clock — 44 of 44 `Cpu` on run 33185280160 — so the wall
/// half is a real, currently unexercised residue, not a fix silently omitted.
pub fn budgeted_cpu_nanos() -> u128 {
    thread_cpu_nanos().saturating_sub(shared_artifact_fill_cpu_nanos())
}

#[cfg(test)]
mod budgeted_cpu_clock_tests {
    use super::*;

    /// THE DISCRIMINATING RED FOR THE DEADLINE HALF OF THE FILL-ATTRIBUTION RULING. A claim
    /// paying a shared fill must not have it counted against its own CPU deadline — every later
    /// claim naming the same source reads the artifact free, so charging it makes an interrupt a
    /// function of discovery order, not the row.
    ///
    /// ASSERTED AS A DECREASE, NOT A RATIO AGAINST MEASURED WORK. An earlier version burned CPU
    /// in a spin loop, called the measured cost a fill, and compared the clock's advance against
    /// `burned / 2`; on the remote runner `burned` was ZERO (thread CPU did not advance across the
    /// loop), making the assertion unsatisfiable for an unsigned quantity — it measured clock
    /// granularity, not netting. Recording a KNOWN fill makes the fill an input, not an
    /// observation.
    #[test]
    fn a_recorded_fill_moves_the_budget_clock_backwards_relative_to_raw_cpu() {
        const FILL_NANOS: u128 = 1_000_000;

        // SPEND MORE CPU THAN THE FILL BEFORE RECORDING IT: a fill exceeding the thread's own
        // CPU is unreachable in production (fill is measured FROM CPU spent inside the miss
        // path, so the raw clock bounds it). An earlier version recorded 1ms of fill on a fresh
        // thread that had not spent 1ms, saturated the clock to zero, and failed on an input the
        // system cannot produce — a defect in the test's premise, not the netting.
        let mut spin = 0u64;
        let mut rounds = 0u32;
        while thread_cpu_nanos() < FILL_NANOS * 4 && rounds < 100_000 {
            for i in 0..100_000u64 {
                spin = spin.wrapping_add(i);
            }
            rounds += 1;
        }
        std::hint::black_box(spin);
        assert!(
            thread_cpu_nanos() >= FILL_NANOS * 4,
            "could not accumulate {}ns of thread CPU in {rounds} rounds; the clock is not running \
             and nothing below would be measuring the netting",
            FILL_NANOS * 4
        );

        let budgeted_before = budgeted_cpu_nanos();
        record_shared_artifact_fill_cpu_nanos(FILL_NANOS);
        let budgeted_after = budgeted_cpu_nanos();

        // THE DISCRIMINATING ASSERTION. Raw CPU only ever rises, so a budget clock that moved
        // BACKWARDS across a recorded fill can only have subtracted it. Without the netting the
        // two clocks move together and this fails.
        assert!(
            budgeted_after < budgeted_before,
            "recording a {FILL_NANOS}ns fill must move the budget clock backwards: \
             {budgeted_before} -> {budgeted_after}"
        );

        // AND THE EXACT RELATION, BRACKETED RATHER THAN DIFFERENCED. `budget_drop + raw_advance
        // == FILL` (an earlier version) is false: the two deltas span different overlapping
        // intervals, so their CPU does not cancel. Bracketing needs no tolerance or clock-speed
        // assumption — `budgeted + fill` IS a raw CPU reading, so it falls between two raw reads
        // taken either side.
        let lo = thread_cpu_nanos();
        let budgeted = budgeted_cpu_nanos();
        let hi = thread_cpu_nanos();
        let reconstructed = budgeted + shared_artifact_fill_cpu_nanos();
        assert!(
            reconstructed >= lo && reconstructed <= hi,
            "budgeted + fill ({reconstructed}) must be a raw CPU reading taken between {lo} and {hi}"
        );
    }

    /// THE CONTROL THAT KEEPS THE ONE ABOVE HONEST: a frozen clock would satisfy "fills do not
    /// advance it" and disarm every deadline, so ordinary CPU (no fill recorded) must still
    /// move it.
    ///
    /// The loop runs until the RAW clock advances, not for a fixed count — a fixed count made
    /// the previous version depend on runner clock granularity. The cap makes a never-advancing
    /// clock a loud failure, not a hang.
    #[test]
    fn ordinary_work_still_advances_the_budget_clock() {
        let raw_start = thread_cpu_nanos();
        let budgeted_start = budgeted_cpu_nanos();

        let mut spin = 0u64;
        let mut rounds = 0u32;
        while thread_cpu_nanos() == raw_start && rounds < 10_000 {
            for i in 0..100_000u64 {
                spin = spin.wrapping_add(i);
            }
            rounds += 1;
        }
        std::hint::black_box(spin);

        assert!(
            thread_cpu_nanos() > raw_start,
            "the thread CPU clock never advanced across {rounds} rounds of work; the budget \
             clock cannot be tested against a clock that does not run"
        );
        assert!(
            budgeted_cpu_nanos() > budgeted_start,
            "ordinary work must advance the budget clock, or every CPU deadline is disarmed"
        );
    }

    /// The clock a deadline is armed on must never run backwards, or an armed deadline could
    /// stop being reachable. Fill is measured on the same thread clock inside the miss path, so
    /// the difference is monotone; this pins the saturating floor that covers sampling skew.
    #[test]
    fn the_budget_clock_never_runs_backwards_even_if_fill_overshoots() {
        let before = budgeted_cpu_nanos();
        record_shared_artifact_fill_cpu_nanos(u128::MAX / 2);
        assert!(budgeted_cpu_nanos() <= before);
        assert_eq!(budgeted_cpu_nanos(), 0, "saturates rather than wrapping");
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
    // THE DETERMINISTIC WORK MEASURE, counted before anything below can return early: a step is
    // an ENTRY to this function, so the count is the same whether the budget poll fires, whether
    // a clock is armed at all, and whether profiling is on.
    record_eval_step();
    // The stride poll runs when EITHER clock is armed. Gating on the CPU deadline alone was a
    // real defect, found by executing a wall-only serve process: with no CPU limit the poll was
    // unreachable, so a wall-only caller — the configuration that bounds a low-CPU stall — had
    // no in-eval crossing point.
    //
    // Neither clock contains an evaluation blocked inside one native primitive: it never returns
    // to `eval_expr`, so nothing is polled. That residue is why worker isolation, not a budget,
    // bounds the listener unconditionally.
    let cpu_armed = ctx.eval_deadline.get();
    let wall_armed = ctx.witness_wall_deadline.get().is_some();
    if cpu_armed.is_some() || wall_armed {
        let stride = ctx.eval_deadline_stride.get().wrapping_add(1);
        ctx.eval_deadline_stride.set(stride);
        if stride % 4096 == 0 {
            if let Some((cpu_baseline_nanos, budget_ms)) = cpu_armed {
                let raw_cpu_nanos = thread_cpu_nanos();
                let elapsed_nanos = raw_cpu_nanos
                    .saturating_sub(shared_artifact_fill_cpu_nanos())
                    .saturating_sub(cpu_baseline_nanos);
                if (elapsed_nanos / 1_000_000) as u64 > budget_ms {
                    if let Some((producer, fill_cpu_nanos)) =
                        in_flight_cross_claim_fill(raw_cpu_nanos)
                    {
                        let marginal_cpu_nanos = elapsed_nanos.saturating_sub(fill_cpu_nanos);
                        if (marginal_cpu_nanos / 1_000_000) as u64 <= budget_ms {
                            return Err(InterpError::FillBudgetExceeded {
                                entry: ctx.budget_entry_or_unnamed(),
                                producer,
                                fill_cpu_nanos,
                                marginal_cpu_nanos,
                                limit_ms: budget_ms,
                            });
                        }
                    }
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

        // An elaborated literal (std.literal_elaboration) evaluates as the kernel value: this
        // interpreter realizes the structural destinations natively by its own grounding
        // (Zero/Succ as Int per #5428, v2.std.logic Bool as bool), so the image of the literal
        // under that grounding IS the literal, and evaluating the constructor tree instead would
        // route a natively-realized Bool through variant patterns that have no runtime form here.
        // The structural image is consumed by emission, which is where the destination is
        // structural; the emitted-bytes witnesses exercise that path.
        ExprData::ExprElaboratedLiteral { value, .. } => eval_literal(&value),

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

    if let Some(VarBindingKind::VariantValueBinding { parent_enum }) = binding_kind {
        // Bounded residual: Nat's intentional native representation is still selected by the
        // arm lexeme because the seed binding carrier lacks exact owner declaration identity.
        // The executable shorthand test and GuaranteeStall keep that silent-wrongness path
        // visible until NS-0B can select the representation from typed identity instead.
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

    // Only the untyped null/optional shorthand collapses to the host Null carrier. A resolved
    // coproduct arm named `None` was handled above and retains its declaration identity
    // (`parent_enum`, terminal arm), so `Diagnostics.None`, an imported `None`, and a re-export
    // construct the same value while an unrelated coproduct's `None` remains distinct.
    if ctx.sym_eq(sym, "none") || ctx.sym_eq(sym, "None") {
        return Ok(Value::Null);
    }

    if let Some(val) = env.lookup(sym) {
        return Ok(val.clone());
    }

    // Slow path (not a bound variable): materialize the name string for the registry lookup.
    let name = ctx.resolve(sym);
    if let Some(info) = v1_rt::map_get(&ctx.item_registry, name.clone()) {
        if info.kind == ItemKind::DataItem {
            if let Some(fn_node) = ctx.lookup_fn_from(&name, node.span.file.as_str()) {
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
            if let Some(fn_node) = ctx.lookup_fn_from(&name, node.span.file.as_str()) {
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
        BinOp::Add => a
            .checked_add(b)
            .map(Value::Int)
            .ok_or(InterpError::IntegerOverflow {
                op: "+",
                lhs: a,
                rhs: b,
            }),
        BinOp::Sub => a
            .checked_sub(b)
            .map(Value::Int)
            .ok_or(InterpError::IntegerOverflow {
                op: "-",
                lhs: a,
                rhs: b,
            }),
        BinOp::Mul => a
            .checked_mul(b)
            .map(Value::Int)
            .ok_or(InterpError::IntegerOverflow {
                op: "*",
                lhs: a,
                rhs: b,
            }),
        BinOp::Div => {
            if b == 0 {
                return Err(InterpError::DivisionByZero);
            }
            a.checked_div(b)
                .map(Value::Int)
                .ok_or(InterpError::IntegerOverflow {
                    op: "/",
                    lhs: a,
                    rhs: b,
                })
        }
        BinOp::Mod => {
            if b == 0 {
                return Err(InterpError::DivisionByZero);
            }
            a.checked_rem(b)
                .map(Value::Int)
                .ok_or(InterpError::IntegerOverflow {
                    op: "%",
                    lhs: a,
                    rhs: b,
                })
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
            Value::Int(n) => n
                .checked_neg()
                .map(Value::Int)
                .ok_or(InterpError::IntegerOverflow {
                    op: "-",
                    lhs: 0,
                    rhs: n,
                }),
            Value::Float(n) => Ok(Value::Float(-n)),
            _ => Err(InterpError::TypeError {
                msg: format!("cannot negate {}", val.type_label()),
            }),
        },
    }
}

#[cfg(test)]
mod argv_representation_ambiguity_tests {
    //! THE RED FOR THE ARGV AMBIGUITY REFUSAL.
    //!
    //! A unit-level construction, not an executed process, by necessity: in hermetic mode
    //! `eval_mock_response` replays an operation's RESULT off its declaration and never touches
    //! argv, so NO argv is constructed in the mode CI runs; a witness waiting for one would
    //! assert nothing.
    //!
    //! What discriminates: both cases carry the SAME two strings and declared meaning; only the
    //! runtime representation differs. The native list expands to two argv words; the
    //! monoid-encoded form used to collapse to the single word "mainHEAD" and now refuses. Delete
    //! the refusal and `monoid_encoded_string_sequence_refuses` fails with one argv word.
    use crate::v1_rt::RcStr;
    use std::rc::Rc;

    use im::{vector as im_vec, HashMap};

    use crate::v1_compiler_infer_emit_info::empty_emit_graph_info;
    use crate::v1_compiler_infer_items::ResolvedGraph;

    use super::{
        list_value, push_shell_argv_tokens, with_active_ctx, ExecutionMode, InterpContext,
        InterpError, Value,
    };

    fn fresh_ctx() -> InterpContext {
        let graph = ResolvedGraph {
            modules: Rc::new(im_vec![]),
            item_registry: Rc::new(HashMap::new()),
            diagnostics: Rc::new(im_vec![]),
            emit_graph_info: empty_emit_graph_info(),
        };
        InterpContext::new(&graph, Rc::new(HashMap::new()), ExecutionMode::Hermetic)
    }

    /// `Cons { head, tail }` over `Empty` — the representation a fold produces.
    fn monoid_of(ctx: &InterpContext, items: &[&str]) -> Value {
        let mut cur = Value::Variant {
            type_name: ctx.sym("FreeMonoid"),
            variant_name: ctx.sym("Empty"),
            fields: Rc::new(Vec::new()),
        };
        for s in items.iter().rev() {
            cur = Value::Variant {
                type_name: ctx.sym("FreeMonoid"),
                variant_name: ctx.sym("Cons"),
                fields: Rc::new(vec![
                    (ctx.sym("head"), Value::Str(RcStr::from(*s))),
                    (ctx.sym("tail"), cur),
                ]),
            };
        }
        cur
    }

    #[test]
    fn native_list_of_two_strings_expands_to_two_argv_words() {
        let ctx = fresh_ctx();
        with_active_ctx(&ctx, || {
            let mut argv = Vec::new();
            push_shell_argv_tokens(
                &mut argv,
                list_value(vec![
                    Value::Str(RcStr::from("main")),
                    Value::Str(RcStr::from("HEAD")),
                ]),
            )
            .expect("a native list is unambiguous and must expand");
            assert_eq!(argv, vec!["main".to_string(), "HEAD".to_string()]);
        });
    }

    #[test]
    fn monoid_encoded_string_sequence_refuses() {
        let ctx = fresh_ctx();
        with_active_ctx(&ctx, || {
            let mut argv = Vec::new();
            let result = push_shell_argv_tokens(&mut argv, monoid_of(&ctx, &["main", "HEAD"]));
            match result {
                Err(InterpError::TypeError { msg }) => {
                    assert!(
                        msg.contains("ambiguous"),
                        "the refusal must name the ambiguity, got: {msg}"
                    );
                }
                // The pre-refusal behaviour: one concatenated word. Named explicitly so the
                // regression is legible rather than appearing as a bare count mismatch.
                Ok(()) => panic!(
                    "expected a refusal; the ambiguity was resolved silently into argv {argv:?} \
                     (pre-fix this was the single word \"mainHEAD\")"
                ),
                Err(other) => panic!("expected a TypeError naming the ambiguity, got {other:?}"),
            }
        });
    }

    #[test]
    fn codepoint_monoid_still_decodes_to_one_word() {
        let ctx = fresh_ctx();
        with_active_ctx(&ctx, || {
            // An Int-element monoid is a code-point sequence: unambiguous under one reading only,
            // so it must keep decoding. This is the control that keeps the refusal from widening
            // into every modeled string.
            let mut cur = Value::Variant {
                type_name: ctx.sym("FreeMonoid"),
                variant_name: ctx.sym("Empty"),
                fields: Rc::new(Vec::new()),
            };
            for c in "hi".chars().rev() {
                cur = Value::Variant {
                    type_name: ctx.sym("FreeMonoid"),
                    variant_name: ctx.sym("Cons"),
                    fields: Rc::new(vec![
                        (ctx.sym("head"), Value::Int(c as i64)),
                        (ctx.sym("tail"), cur),
                    ]),
                };
            }
            let mut argv = Vec::new();
            push_shell_argv_tokens(&mut argv, cur).expect("a codepoint monoid is a host string");
            assert_eq!(argv, vec!["hi".to_string()]);
        });
    }
}

#[cfg(test)]
mod eval_int_binop_overflow_tests {
    use super::{eval_int_binop, InterpError, Value};
    use crate::std_syntax::BinOp;

    #[test]
    fn mul_overflow_refuses_instead_of_wrapping() {
        // i64::MAX is ~9.2e18; 4_000_000_000 * 4_000_000_000 = 1.6e19 does not fit and
        // must not silently wrap to a negative value (the fabrication this guards against).
        match eval_int_binop(&BinOp::Mul, 4_000_000_000, 4_000_000_000) {
            Err(InterpError::IntegerOverflow { op: "*", lhs, rhs }) => {
                assert_eq!((lhs, rhs), (4_000_000_000, 4_000_000_000));
            }
            other => panic!("expected IntegerOverflow, got {other:?}"),
        }
    }

    #[test]
    fn add_overflow_refuses() {
        match eval_int_binop(&BinOp::Add, i64::MAX, 1) {
            Err(InterpError::IntegerOverflow { op: "+", .. }) => {}
            other => panic!("expected IntegerOverflow, got {other:?}"),
        }
    }

    #[test]
    fn sub_overflow_refuses() {
        match eval_int_binop(&BinOp::Sub, i64::MIN, 1) {
            Err(InterpError::IntegerOverflow { op: "-", .. }) => {}
            other => panic!("expected IntegerOverflow, got {other:?}"),
        }
    }

    #[test]
    fn in_range_mul_still_succeeds() {
        assert_eq!(eval_int_binop(&BinOp::Mul, 6, 7).unwrap(), Value::Int(42));
    }

    #[test]
    fn div_min_by_negative_one_refuses_instead_of_overflowing() {
        // i64::MIN / -1 is not representable as i64 and must not panic or wrap.
        match eval_int_binop(&BinOp::Div, i64::MIN, -1) {
            Err(InterpError::IntegerOverflow { op: "/", .. }) => {}
            other => panic!("expected IntegerOverflow, got {other:?}"),
        }
    }

    #[test]
    fn mod_min_by_negative_one_refuses_instead_of_overflowing() {
        match eval_int_binop(&BinOp::Mod, i64::MIN, -1) {
            Err(InterpError::IntegerOverflow { op: "%", .. }) => {}
            other => panic!("expected IntegerOverflow, got {other:?}"),
        }
    }

    #[test]
    fn in_range_div_still_succeeds() {
        assert_eq!(eval_int_binop(&BinOp::Div, 84, 2).unwrap(), Value::Int(42));
    }
}

#[cfg(test)]
mod eval_unaryop_overflow_tests {
    use super::{eval_unaryop, InterpError, UnaryOpKind, Value};

    #[test]
    fn neg_of_i64_min_refuses_instead_of_wrapping() {
        // -i64::MIN is not representable as i64 (wraps to i64::MIN in release).
        match eval_unaryop(&UnaryOpKind::Neg, Value::Int(i64::MIN)) {
            Err(InterpError::IntegerOverflow { op: "-", rhs, .. }) => {
                assert_eq!(rhs, i64::MIN);
            }
            other => panic!("expected IntegerOverflow, got {other:?}"),
        }
    }

    #[test]
    fn neg_in_range_still_succeeds() {
        assert_eq!(
            eval_unaryop(&UnaryOpKind::Neg, Value::Int(42)).unwrap(),
            Value::Int(-42)
        );
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

// HAND-RUST GATE explicit deferral (review 46616): bounded growth in the seed interpreter, not
// a new Rust authority. The evaluation-boundary POLICY is modeled — `v2.std.witness_evaluation`
// owns `WitnessEvaluation`/`WitnessEvaluationFrame`, `extdeps.transports.rest`
// `rest_exchange_resolution` owns lookup, equality and handler selection; here lives only the
// dynamic-extent realization of pushing/popping a frame, which no modeled construct can express
// while the seed is the evaluator.
//
// Lane: ROADMAP `v1-materialization-kernel` (rn_53JPH6BB7G588K7DMZNWM0E3AS,
// witness-realization-plan (plan doc deleted 2026-08-28)) — the lane
// `extdeps.realization.emit_on_demand_host` the `emit_on_demand_host_seed_deferral_note` annotation defers
// to; counted against `v1-honest-frontier`, terminating at `v1-interpreter-quarantine` →
// `v1-interpreter-delete`.
//
// Deletion condition, checkable by execution: witnesses emit to native code and the emitted
// runtime realizes the evaluation frame; then this stack, its `WITNESS_EVALUATION_MODULE`
// dispatch, and `witness_evaluation_diagnostic_value` delete together while
// `rest_replay_binding_does_not_escape_its_frame` stays green without them — that witness is
// the regression control for the deletion as well as the frame, failing if a binding survives
// its frame under either realization.
//
// Citation note: the two sibling deferrals here and in the `emit_on_demand_host_seed_deferral_note` annotation
// name a `dag/gunbc/v1/v1_deletion_plan.dag ^witness_realization_kernel` deletion row that no
// longer exists — its brick ledger was retired 2026-07-28 by that file's the `v1_exit_model_doc` annotation,
// which moved per-node acceptance onto roadmap tickets. This deferral names the live roadmap
// node instead; repointing the two stale siblings is left to their owning lane.
thread_local! {
    /// Dynamically scoped witness frames; the .dag carrier owns their contents, this stack is
    /// only the v1 seed realization of the boundary. Pushed immediately before the subject
    /// closure, removed by `WitnessFramePop`'s Drop on every exit path — returned, refused, or
    /// unwound — so replay bindings cannot become ambient.
    static WITNESS_EVALUATION_FRAMES: RefCell<Vec<Value>> = const { RefCell::new(Vec::new()) };
}

/// Pops the frame its constructor's caller pushed, on EVERY exit path including unwind. The
/// pop used to be a statement after `apply_closure`, holding only because no path between
/// returned early — a property of the current body, not the code, so the "removed on both
/// returned/refused paths" comment was a promise the shape did not keep (review 46767).
///
/// Closed by construction because a leaked frame is not merely a stale binding:
/// `dispatch_service` consults `current_witness_evaluation_frame()` to decide whether a
/// hermetic op routes to the real dispatcher, so an escaped frame would silently route
/// *subsequent* ops out of the mock layer. Drop makes the escape unwritable (§5).
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
            // containment path, but values are constructed with the bare last segment (the
            // short-name normalization at value construction). Every name-vs-literal
            // reconciliation below — native Int/Str/List coproducts, Optional/Witness raw
            // (value-or-Null) unwraps — compares that short segment, as the `Value::Variant`
            // arm's fallback does; otherwise a qualified `Zero`/`Succ` (Nat grounded to native
            // Int), `Empty`/`Cons`, or `Present`/`Absent` pattern misses and the match falls
            // through non-exhaustive.
            let name_last = name.rsplit('.').next().unwrap_or(name);
            // Kernel-optional / witness raw representation (value-or-Null): the `_ if
            // Present+Optional` / `_ if Holds+Witness` unwrap arms below the kind-specific arms
            // were UNREACHABLE for Record/List/Str/Int payloads — Value::Record etc. match
            // their kind arm first and return None inside it, so `match xs |> first { Present
            // { value: t } => ... }` failed non-exhaustive on any record element (pre-existing
            // on main; located via the interpreted-parse suite reds). Hoisted here verbatim;
            // Variant payloads excluded so the Variant arm's inline raw-value handling stays
            // authoritative.
            if name_last == "Present"
                && parent_enum_is(parent_enum.as_ref(), "Optional")
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
                && parent_enum_is(parent_enum.as_ref(), "Witness")
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
            if name_last == "Absent" && field_bindings.is_empty() {
                return match value {
                    Value::Null => Some(HashMap::new()),
                    Value::Variant {
                        type_name,
                        variant_name,
                        ..
                    } => {
                        if !coproduct_arm_name_matches(resolve_sym(*variant_name), name.clone()) {
                            None
                        } else if let Some(parent) = parent_enum.as_ref() {
                            if coproduct_parent_spellings_match(
                                ctx,
                                resolve_sym(*type_name),
                                parent,
                            ) || variant_arm_is_declared_in_coproduct(ctx, *variant_name, parent)
                            {
                                Some(HashMap::new())
                            } else {
                                None
                            }
                        } else {
                            Some(HashMap::new())
                        }
                    }
                    _ => None,
                };
            }
            match value {
                Value::Variant {
                    type_name,
                    variant_name,
                    fields,
                } => {
                    if name_last == "Holds"
                        && parent_enum_is(parent_enum.as_ref(), "Witness")
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
                        && parent_enum_is(parent_enum.as_ref(), "Optional")
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
                        name_last,
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
                // deliberately unhandled (no corpus site exercises it, #5-scoped deferral) — an
                // unmatched pattern name falls through to `_ => None` below, refusing rather
                // than fabricating a (pos, neg) pair.
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
                    if name_last == "Violates"
                        && parent_enum_is(parent_enum.as_ref(), "Witness") =>
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
                    if name_last == "None"
                        && parent_enum_is(parent_enum.as_ref(), "Diagnostics") =>
                {
                    Some(HashMap::new())
                }
                Value::Null
                    if name_last == "Absent"
                        && (parent_enum.is_none()
                            || parent_enum_is(parent_enum.as_ref(), "Optional")) =>
                {
                    Some(HashMap::new())
                }
                _ if name_last == "Present" && parent_enum_is(parent_enum.as_ref(), "Optional") => {
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
                _ if name_last == "Holds" && parent_enum_is(parent_enum.as_ref(), "Witness") => {
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
/// `v1_interpreter_authored_roster_arms()` in `.dag`; generated `lookup_eval_call_bridge` routes
/// spellings before this macro matches on the generated enum variant per arm identity.
macro_rules! v1_bridge_family_arms {
    ($cb:ident, $fname:ident, $args:ident, $node:ident, $ctx:ident) => {
        $cb! {
            $fname, $args, $node, $ctx;
            // ONE FAMILY, TWO ARMS, AND THE MODULE IS v2.std.node_reflection RATHER THAN THE TWO
            // MODULES THESE ARMS USED TO SIT IN. Both are host type-reflection seams whose body is
            // a self-call, so emission (membership at MODULE grain) wrote them into the v2
            // compiler's emitted crate as unrealized-seam refusals only because v2.std.node and
            // v2.std.node_query carry vocabulary the compile closure needs. One reflection module
            // drops both out of that closure by identity. `is_v4_bridge_family` matches the item
            // registry's module name against this literal, so the literal moves the bridge; the
            // family enum, lookup fn and arm macro are generated from the same
            // `EvalCallBridgeFamilySite.module` in gunbc.v1_interpreter_primitive_surface, and a
            // name disagreeing with the roster fails to compile.
            family STD_NODE_REFLECTION_BRIDGE_FNS "v2.std.node_reflection"
                lookup_eval_call_bridge_std_node_reflection eval_call_bridge__v2_std_node_reflection_arm {
                arm "v4_bridge.resolve_type_node" { "resolve_type_node" } =>
                    crate::coproduct_reflection::eval_resolve_type_node($ctx, &$args),
                arm "v4_bridge.coproduct_nullary_inhabitants" { "coproduct_nullary_inhabitants" } =>
                    crate::coproduct_reflection::eval_coproduct_nullary_inhabitants($ctx, $node, &$args),
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

pub fn std_node_reflection_bridge_fn_names() -> &'static [&'static str] {
    STD_NODE_REFLECTION_BRIDGE_FNS
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

/// Handler bodies for native fold intercepts (run before free-call dispatch). Roster authority
/// is `v1_interpreter_authored_roster_arms()`; generated `lookup_eval_call_native_intercept`
/// routes spellings before this macro matches on the generated enum variant.
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

    // A LEXICAL BINDING SHADOWS EVERY NAME-KEYED TIER (nearest-first, the law 04_infer states at
    // call_locals_shadow_note). A parameter or let named `lookup`, `count`, `filter`, ... is a
    // function VALUE; answering its call from the builtin or module free-function table by
    // spelling silently calls a different function wherever arities agree.
    //
    // THE GATE IS THE BINDING, NOT THE VALUE'S REPRESENTATION. It formerly matched
    // `Value::Closure` only, so the law held for a LAMBDA and failed for a NAMED top-level
    // function -- `Value::Fn` (the `ItemKind::FuncItem | FnItem` arm of `eval_expr`'s identifier
    // path), which fell past every tier and was answered by `ctx.lookup_fn` at the FREE FUNCTION
    // sharing its spelling. Measured on the 3x2 grid (let / parameter / pattern x named-fn /
    // lambda) in `v2.test.claim.local_binding_shadow`: the three named-fn cells reached the free
    // function, the three lambda cells the local. Both variants are one concept -- a lexical
    // binding holding something callable -- so the gate names that.
    //
    // `ctx.lookup_fn` (module-level declarations) stays BELOW the builtins: this widens the
    // lexical tier only.
    let lexically_bound_fn: Option<Rc<Node>> = match env.lookup(ctx.sym(&func_name)) {
        Some(closure @ Value::Closure { .. }) => {
            let closure = closure.clone();
            let arg_vals: Vec<Value> = args.iter().map(|(_, v)| v.clone()).collect();
            return apply_closure(&closure, &arg_vals, env, ctx);
        }
        Some(Value::Fn { node: bound_fn }) => Some(bound_fn.clone()),
        _ => None,
    };

    if lexically_bound_fn.is_none() {
        v1_bridge_family_arms!(v1_bridge_dispatch, func_name, args, node, ctx);

        v1_native_intercept_arms!(v1_native_intercept_dispatch, func_name, args, env, ctx);

        if let Some(result) = eval_builtin(&func_name, &args, ctx)? {
            return Ok(result);
        }
    }

    // Every callable lexical binding was served by the gate above, so this tier reads only the
    // module free-function table. The former `else`-arm env re-lookup (serving `Value::Fn` and
    // `Value::Closure` AFTER `ctx.lookup_fn`) is deleted, not kept beside the gate: it was the
    // lower-rung duplicate of the same decision (DESIGN §2/§3) and where the shadowing defect
    // landed, since reaching it required the free-function table to answer first.
    let fn_node = match lexically_bound_fn {
        Some(node) => node,
        None => match ctx.lookup_fn_from(&func_name, node.span.file.as_str()) {
            Some(node) => node.clone(),
            None => {
                return Err(InterpError::NoSuchFunction {
                    name: func_name.clone(),
                });
            }
        },
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

/// Pure named-fn calls flow through here: the demand ledger records every keyed call, and the
/// eval-frame memo (the ladder's single-site discharge provider) serves repeats from the first
/// evaluation. A hit still records the DEMAND — the receipt counts plurality; the provider
/// changes cost, never count. Soundness: hash-bucketed on the ledger key but served only after
/// argument names AND values verify equal (Value::eq, the one equality authority) — a collision
/// degrades to recompute, never a wrong value. Unkeyed calls (closure args) stay unmemoized
/// and are counted.
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
    // Guard runs only for admitted calls: a store that lands must carry what it cost, so the
    // paying claim's receipt can net it and the shared-fill ledger can attribute it. Its
    // `Drop` closes the fill on every path out of this function.
    let fill_guard = if cross_claim_pure_admitted(fn_node, func_name) {
        Some(CrossClaimFillGuard::enter(func_name))
    } else {
        None
    };
    let trace_on = eval_recompute_trace_enabled();
    let memo_on = ctx.eval_call_memo.borrow().enabled;
    if !trace_on && !memo_on {
        let effects_before = ctx.effect_dispatch_count.get();
        let result = call_function(ctx, fn_node, args, env);
        if let Ok(v) = &result {
            if ctx.effect_dispatch_count.get() == effects_before {
                // The ordinary call path publishes opportunistically: every outcome,
                // servable or refused, is already counted inside the store, and this
                // call recomputes on a refusal exactly as if never enrolled.
                let _ = store_cross_claim_pure_memo(
                    ctx,
                    fn_node,
                    func_name,
                    args,
                    v,
                    fill_guard.as_ref(),
                );
            }
        }
        return result;
    }
    let started = Instant::now();
    let key = match eval_recompute_key(ctx, fn_node, args) {
        Some(key) => key,
        None => {
            if !trace_on {
                return call_function(ctx, fn_node, args, env);
            }
            // TIMED, not merely counted: the composite-argument bucket feeds the cross-claim
            // census, which ranks by duration. Recording after the call is what makes the two
            // buckets comparable; the earlier count-only form could name a producer it could
            // never rank.
            let result = call_function(ctx, fn_node, args, env);
            eval_recompute_record_unkeyed(ctx, fn_node, func_name, started.elapsed().as_nanos());
            return result;
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
            let _ =
                store_cross_claim_pure_memo(ctx, fn_node, func_name, args, v, fill_guard.as_ref());
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
    // NOTE (nimble-otter-476, adhoc-c328b166-bca): a streaming left-fold over the Cons-chain
    // without this Vec was BUILT, proven byte-identical (parse-tree content hash equal on
    // tiny/small across two independently-built binaries), and MEASURED: neither wall-clock
    // (~20s both, within noise) nor peak RSS (~168 MiB both) moved on the small file. The
    // datetime driver folds `elem=Int` codepoint lists (trivial copy) and the Vec is transient
    // (freed each fold, never in peak RSS). The real O(n^2) is the CALLER re-folding the whole
    // source (`lex_repeat_loop`, 01_tokenize.dag:158 -- routed to the tokenize lane). Kept as
    // `free_monoid_to_vec` rather than churning the seed for a measured-zero rewrite (DESIGN
    // §6: denominate in displaced cost; a no-op displaces nothing).
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
                    if allows_memo {
                        if let Some(v) = st.map.get(&memo_key).cloned() {
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

// Content hash of a Value, memoized per composite allocation. Equal values (Value::eq) hash
// equal; the pointer memo is validated by Weak-liveness so a freed-then-reused address
// recomputes rather than serving a stale hash. None when the value contains a Closure (no
// identity for captured envs) — the caller routes to the disclosed unkeyed bucket. Iterative
// (explicit frame stack): Cons-chain lists and deep node trees have data-sized depth, so
// recursion would overflow the host stack.
enum EvalRecomputeFrameKind {
    List {
        rc: Rc<RrbVector<Value>>,
    },
    Fields {
        rc: Rc<Vec<(Symbol, Value)>>,
        // Content hashes of the resolved symbol TEXT (type/variant name, each field name) —
        // never the interner ordinal. The cross-claim prepare_grammar memo (gunbc#8505) is a
        // thread_local outliving any InterpContext, so an ordinal key is a function of
        // interning ORDER, and two same-shaped, semantically different grammars from different
        // contexts could alias onto one memo entry.
        type_sym_hash: u64,
        variant_sym_hash: u64,
        is_variant: bool,
        field_name_hashes: Vec<u64>,
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
        EvalRecomputeFrameKind::Fields {
            field_name_hashes, ..
        } => {
            let mixed = eval_recompute_mix(f.h, field_name_hashes[f.idx]);
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
            type_sym_hash,
            variant_sym_hash,
            is_variant,
            ..
        } => {
            // The fields-content hash is memoized independently of the owning type/variant
            // symbols (a fields Rc could be shared across constructions), so the entry never
            // bakes in the wrapper identity.
            memo.insert(
                Rc::as_ptr(&rc) as usize,
                (CompositeWeak::Fields(Rc::downgrade(&rc)), h),
            );
            if is_variant {
                eval_recompute_mix(
                    eval_recompute_mix(
                        eval_recompute_mix(0xA5A5_0080, type_sym_hash),
                        variant_sym_hash,
                    ),
                    h,
                )
            } else {
                eval_recompute_mix(eval_recompute_mix(0xA5A5_0070, type_sym_hash), h)
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

fn eval_recompute_value_hash(
    memo: &mut EvalRecomputeHashMemo,
    interner: &SymbolInterner,
    root: &Value,
) -> Option<u64> {
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
                    let type_sym_hash = eval_recompute_str_hash(interner.resolve(*type_name));
                    match memo.get(&ptr) {
                        Some((w, h)) if w.alive() => EvalRecomputeStep::Have(eval_recompute_mix(
                            eval_recompute_mix(0xA5A5_0070, type_sym_hash),
                            *h,
                        )),
                        _ => {
                            let field_name_hashes = fields
                                .iter()
                                .map(|(s, _)| eval_recompute_str_hash(interner.resolve(*s)))
                                .collect();
                            frames.push(EvalRecomputeFrame {
                                kind: EvalRecomputeFrameKind::Fields {
                                    rc: fields.clone(),
                                    type_sym_hash,
                                    variant_sym_hash: 0,
                                    is_variant: false,
                                    field_name_hashes,
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
                    let type_sym_hash = eval_recompute_str_hash(interner.resolve(*type_name));
                    let variant_sym_hash = eval_recompute_str_hash(interner.resolve(*variant_name));
                    match memo.get(&ptr) {
                        Some((w, h)) if w.alive() => EvalRecomputeStep::Have(eval_recompute_mix(
                            eval_recompute_mix(
                                eval_recompute_mix(0xA5A5_0080, type_sym_hash),
                                variant_sym_hash,
                            ),
                            *h,
                        )),
                        _ => {
                            let field_name_hashes = fields
                                .iter()
                                .map(|(s, _)| eval_recompute_str_hash(interner.resolve(*s)))
                                .collect();
                            frames.push(EvalRecomputeFrame {
                                kind: EvalRecomputeFrameKind::Fields {
                                    rc: fields.clone(),
                                    type_sym_hash,
                                    variant_sym_hash,
                                    is_variant: true,
                                    field_name_hashes,
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
    interner: &SymbolInterner,
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
            eval_recompute_str_hash(interner.resolve(*type_name)),
            eval_recompute_str_hash(interner.resolve(*variant_name)),
        )),
        Value::List(xs) if xs.is_empty() => Some(EvalRecomputeArgKey::EmptyList),
        other => {
            eval_recompute_value_hash(memo, interner, other).map(EvalRecomputeArgKey::ContentHash)
        }
    }
}

fn eval_recompute_key(
    ctx: &InterpContext,
    fn_node: &Rc<Node>,
    args: &[(Option<String>, Value)],
) -> Option<EvalRecomputeKey> {
    let mut memo = ctx.eval_recompute_hash_memo.borrow_mut();
    let interner = ctx.symbols.borrow();
    let mut keys = Vec::with_capacity(args.len());
    for (_, v) in args {
        keys.push(eval_recompute_arg_key(&mut memo, &interner, v)?);
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

fn eval_recompute_record_unkeyed(
    ctx: &InterpContext,
    fn_node: &Rc<Node>,
    func_name: &str,
    elapsed_ns: u128,
) {
    let mut t = ctx.eval_recompute_trace.borrow_mut();
    t.unkeyed_calls += 1;
    let row = t
        .unkeyed_by_fn
        .entry(func_name.to_string())
        .or_insert_with(|| (0, 0, eval_recompute_decl_site(fn_node)));
    row.0 += 1;
    row.1 += elapsed_ns;
}

/// A producer's DECLARATION site, `file:offset`. The cross-claim census below keys on it beside
/// the name because `func_name` is the call's spelling: two modules may declare the same bare
/// name, and merging them would report one producer that does not exist. Unlike `fn_ptr`, a span
/// is derived from the source and is therefore equal across the fresh evaluation frame the floor
/// builds per claim — which is the whole boundary this census has to cross.
fn eval_recompute_decl_site(fn_node: &Rc<Node>) -> String {
    format!("{}:{}", fn_node.span.file, fn_node.span.start)
}

/// Print the recompute-trace ledger to stderr. No-op unless GUNBC_RECOMPUTE_TRACE=1.
/// Report-only: ranked re-evaluated pure calls (count >= 2), the unkeyed-coverage disclosure,
/// and totals; never alters the run's outcome.
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
    let mut unkeyed: Vec<(&String, &(u64, u128, String))> = t.unkeyed_by_fn.iter().collect();
    unkeyed.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));
    for (name, (count, ns, site)) in unkeyed.iter().take(10) {
        eprintln!(
            "[recompute-trace] unkeyed fn={} calls={} total_ms={} @{} (composite args — identity not tracked in slice 1)",
            name,
            count,
            ns / 1_000_000,
            site
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

// ── THE CROSS-CLAIM DEMAND CENSUS ────────────────────────────────────────────────────────────
//
// WHAT IT IS FOR, stated first because the mechanism is small and the reason is the whole point:
// it names, as a RUN PRODUCT, the pure producers the required floor re-derives once per claim
// across many claims — the candidate population for `v2.workflow.floor_pure_producer_share`.
//
// THE BLINDNESS IT CLOSES. `EvalRecomputeTrace` is scoped to one `InterpContext` and reports
// keys with `count >= 2`; the floor builds a FRESH FRAME PER CLAIM and prints and drops the
// ledger at every claim boundary. So a producer evaluated EXACTLY ONCE per claim, in thousands
// of claims, has `count = 1` in every ledger and appears in none of them — the instrument that
// exists to rank redundant recompute cannot see the one recurrence shape that dominates the
// floor's per-claim cost. Measured on main run 33615659836 (`required_floor_claim_cost.tsv`,
// 3477 rows): a witness and its discriminating RED, which do materially different work, cost
// within 2ms of each other at 294/296, 331/333 and 382/384 while p50 for the corpus is 2ms. A
// cost that does not move when the assertion changes is not the assertion's cost; it is the
// closure's fixed re-derivation, and DESIGN §2 prices it as authored duplication whose least
// common ancestor is the RUN. This census is the demand side of that: it does not share, cache
// or enrol anything — it reports which identity is being re-demanded across the frame boundary,
// so roster candidacy stops being discovered by a red landing on an unrelated lane's PR.
//
// IT IS AN OBSERVATION AND NEVER A GATE. Nothing refuses on these figures. Durations are
// wall-of-the-calling-thread inclusive of callees and are as environment-sensitive as every
// other cost reading on a shared runner (`gunbc.rung_drop floor_cost_claim_qualification_unavailable`); the
// CLAIM COUNT is the stable half, and it is a property of the corpus and the plan rather than
// of the machine.
//
// THE COST COLUMN IS NOT ADDITIVE, AND THIS PARAGRAPH EXISTS BECAUSE THE ARTIFACT INVITES THE
// ERROR. Durations are INCLUSIVE OF CALLEES — the ledger times a producer's whole subtree — so a
// producer and everything it calls both appear, and their figures OVERLAP. Summing the column
// therefore counts the same nanoseconds once per level of nesting. Measured on the first run that
// produced the artifact: summing cross-claim over the shared rows gives ~1850s against a run
// whose ENTIRE claim-side CPU was 130s — a 14x overcount, which is the nesting and nothing else.
// So a per-row figure is a valid statement about THAT producer, and no sum of rows is a valid
// statement about the run. The run's own total claim CPU travels in the summary line as the
// ceiling any true total must sit under.
//
// SO THE DISPLACED-COST SENTENCE IS UNDERIVABLE TODAY, NOT MERELY UNSTATED, and the difference is
// the whole reason this paragraph is a trigger rather than a caveat. "This floor recomputes N
// seconds of pure producer work per run" is the sentence that makes the stake legible, and NO
// arithmetic over this artifact produces it: the quantity it needs does not exist in the column.
//
// NEXT-RUNG TRIGGER, AS A CAPABILITY: SELF TIME — a producer's inclusive duration minus the
// callees the same pass already counted. With it the column is ADDITIVE, the sum becomes a true
// statement about the run, and the displaced-cost sentence is derivable. WHERE IT ALREADY LIVES,
// so this is a tracked stall and not a wish: `CrossClaimFillGuard`'s `Drop` in this file computes
// exactly that quantity one tier over — `inclusive_cpu.saturating_sub(children_cpu)` against the
// `CROSS_CLAIM_FILL_FRAMES` child stack — for the shared-fill ledger. The mechanism exists; what
// is missing is a child stack over the RECOMPUTE ledger's frames. Deliberately not built here:
// this bridge is what stops the next four lanes paying, and a new measurement tier would put it
// behind a fresh review cycle on the fleet condition it exists to explain.
//
// AN OUT-OF-SCOPE OBSERVATION THE CEILING COLUMN MADE, RECORDED HERE BECAUSE ITS ONLY OTHER HOME
// IS TWO LOGS THAT AGE OUT. `claim_cpu_total_ms` — added purely as a bound against summing the
// cost column — measured 130335 on run 33615659836 and 99943 on run 33631458679 (both
// `run_attempt=1`), over the same corpus. A THIRTY PERCENT SWING IN A RUN'S TOTAL CLAIM CPU, and
// a per-closure constant cannot produce it: the constant is by construction identical on two runs
// of one tree. It is the same variance term measured independently as byte-identical evaluator
// steps against 1.31-1.80x cpu, reached here from the opposite direction and at whole-run grain.
// UNEXPLAINED AND DELIBERATELY NOT PURSUED HERE — this census's subject is the LEVEL — but it is
// a property of the ENVIRONMENT measured from inside the run, which is what a fleet-variance
// account has so far lacked. THE TWO FIGURES ARE TRANSCRIBED, as a declared exception to naming
// the instrument instead of copying its output, for the reason the floor-cost carrier grants the
// same exception: THE SUBJECT IS THE DIVERGENCE BETWEEN TWO RUNS, which no single producer on
// this side of the boundary can re-derive once the logs expire.
//
// WHAT THIS CENSUS CANNOT SEE, DECLARED HERE RATHER THAN DISCOVERED BY A LATER READER. Two
// boundaries, and neither is a defect in the aggregation — they are properties of the ledger it
// folds, and a reader who does not know them will read absence as evidence.
//
// (1) IT EXPLAINS THE LEVEL, NOT THE VARIANCE. A per-closure constant is BY CONSTRUCTION the
// same on two runs of the same tree, so it cannot produce a run-to-run delta — and one is
// measured: 32 rows with byte-identical `eval_steps` and cpu up 1.31-1.80x between two runs over
// a byte-identical payload module (gentle-wolf-793, 2026-09-02). The two compose into the shape
// the floor keeps showing: the constant puts a family AT the line, and something run-to-run
// decides WHICH of its rows cross it, and there are TWO INDEPENDENT RECEIPTS for that, neither of
// which is a cross-run comparison. (i) ZERO MARGIN, single attempt: two rows of one module
// COMPLETED over budget — so these are measured values, not preemption bounds — at 502ms and
// EXACTLY 500ms against the 500ms budget. (ii) A ROW OBSERVED CROSSING THE
// TWO POPULATIONS ON ONE TREE: run 33620893203, attempt 1, records
// `test.claim.self_host_compile_phase_live_gate_witness.an_empty_receipt_series_leaves_the_live_tree_unmeasured_rather_than_held`
// as INTERRUPTED-BEFORE-VERDICT with `cost=UNMEASURED`, and ATTEMPT 2 of the SAME RUN records it
// as COMPLETED-OVER-COST-REQUIREMENT at cpu_ms=500 — reaching its verdict. Those are the two
// populations the 2026-08-19 budget cut separated BECAUSE THEY ARE DIFFERENT FACTS WITH DIFFERENT
// REMEDIES, and at this margin which one a row lands in is decided by whether the poll fired
// before or after the work finished. A fact about the poll, not about the row.
//
// AND THE PAIR IS NOT TWO MEASUREMENTS OF ONE QUANTITY — one is a BOUND and one is a VALUE, which
// is why no cost figure is written for attempt 1 here. The interrupted arm knows only that the
// cost exceeded 500ms by an unknown amount; its `interrupt_point` names where the poll observed
// the ceiling, so it is a property of the BUDGET and the diagnostic says so in three clauses.
// Writing it as "502ms in attempt 1 against 500ms in attempt 2" would say the row got two
// milliseconds cheaper and crossed a line — a story about the row, and the instrument-property-
// as-subject-property misreading this very census exists to stop. What changed between the
// attempts is what the OBSERVER could say.
//
// CITE THE ATTEMPT, NOT THE RUN. Both readings above are pinned to
// `/actions/runs/<id>/attempts/<n>/logs`, because a bare run id and a bare job id BOTH resolve to
// the latest attempt: a rerun silently changes what a stable citation serves, with no error and
// nothing visible from the citing end. A near-disjointness reading was retracted and then
// partially restored on exactly that discovery. This file's other figures come from run
// 33615659836 and run 33622954427, both `run_attempt=1` at the time of writing — recorded so a
// reader can tell whether the citation still points at what was read. Neither half alone predicts
// the wandering — a constant alone gives
// the same rows every run, and runner noise alone over a corpus with p50=2ms tips nobody. This
// census addresses the charge; the tipping variable is separately open and is not this
// instrument's subject.
//
// (2) A COST THAT LIVES INSIDE A NATIVE BUILTIN CALLED DIRECTLY IS NOT A ROW HERE. The ledger
// keys PURE NAMED FN evaluations; a `free_call.*` arm dispatched straight to Rust from a claim
// body is not one, so its cost is invisible to this census — while the same work reached THROUGH
// a .dag producer is captured, because the producer's own timing spans it. `std.evaluation_budget`
// `evaluation_budget_opaque_host_call_note` records the matching fact for the deadline: no poll
// stride falls inside an opaque host call, so such a claim cannot be interrupted at all and
// surfaces as completed-over-cost instead. So a producer missing from this ranking is not
// evidence that nothing is re-derived under it.
//
// ADMISSION TO THIS CENSUS IS NOT ADMISSION TO THE SHARE ROSTER. A row here is a CANDIDATE.
// `v2.workflow.floor_pure_producer_share` records the criterion that decides enrolment and the
// measurement that refuted the obvious case: the two rust target models were enrolled and
// REMOVED because SERVING them costs more than recomputing them. So this artifact ranks demand;
// it does not price the serve, and reading a top row as an enrolment instruction would trade a
// measured red for an unmeasured regression.

/// One producer identity, aggregated across claim frames.
#[derive(Clone)]
pub struct CrossClaimDemandRow {
    /// The call's spelling.
    pub producer: String,
    /// `file:offset` of the DECLARATION. Frame-independent, and the disambiguator that keeps two
    /// same-named producers in different modules from merging into one row that does not exist.
    pub decl_site: String,
    /// `keyed` — one row per (declaration, argument row) with sound argument identity — or
    /// `unkeyed`, the composite-argument bucket, where one row covers ALL argument rows of that
    /// declaration and the claim count is therefore an upper bound on any single identity's.
    pub arg_shape: &'static str,
    /// Distinct claims in which this identity was evaluated at least once.
    pub claims: u64,
    /// Evaluations across all of them.
    pub evals: u64,
    /// Inclusive nanos across all of them.
    pub total_ns: u128,
    /// Distinct consumer modules, and a bounded sample of their names.
    pub modules: u64,
    pub module_sample: Vec<String>,
}

impl CrossClaimDemandRow {
    /// What a provider whose scope reached the RUN would have saved: everything beyond one
    /// evaluation's amortized share across the claims that demanded it. Zero for a producer
    /// demanded by exactly one claim, which is the point — a single-claim cost is a claim's own
    /// work and belongs to the cost receipt, not to this census.
    pub fn cross_claim_wasted_ns(&self) -> u128 {
        if self.claims <= 1 {
            return 0;
        }
        self.total_ns - self.total_ns / u128::from(self.claims)
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
struct CrossClaimDemandKey {
    producer: String,
    decl_site: String,
    /// THE CANONICAL ARGUMENT ROW, NOT A DIGEST OF IT. A 64-bit hash stood here and it could
    /// merge two distinct argument rows into one row reporting cross-claim demand that never
    /// happened — a fabricated identity, which DESIGN §5 forbids outright and which the header
    /// above simultaneously promised not to do ("sound argument identity"). The ledger already
    /// keys on this exact vector; carrying it costs the memory the retention floor and key cap
    /// are here to bound, and it makes the collision unrepresentable rather than unlikely.
    /// Found by review 58673.
    args: Vec<EvalRecomputeArgKey>,
    /// `keyed` and `unkeyed` are DIFFERENT SUBJECTS and must not share a key: a nullary keyed
    /// call and the composite-argument bucket of the same declaration both carry an empty
    /// argument vector, and merging them would sum one identity's cost with all of another's.
    arg_shape: &'static str,
}

struct CrossClaimDemandCell {
    claims: u64,
    evals: u64,
    total_ns: u128,
    last_claim: String,
    /// THE COMPLETE consumer-module set, interned so one name costs one allocation for the whole
    /// census. It is complete because the reported count must be exact: the earlier form kept a
    /// bounded SAMPLE and counted against it, so once the sample filled, every later claim from
    /// an unsampled module incremented the count again and the column reported CLAIM OCCURRENCES
    /// while promising distinct modules. Bounding and counting cannot share one container.
    /// Found by review 58673.
    modules: std::collections::BTreeSet<Rc<str>>,
}

/// Rows below this are not retained at identity grain. The census's subject is a producer whose
/// per-claim cost is a visible share of a 500ms ceiling; a sub-millisecond frame cost cannot be
/// one however often it recurs at this grain, and retaining every such key across thousands of
/// claims is the memory the floor does not have. THE OMITTED MASS IS DISCLOSED — count and
/// summed nanos — because an artifact that truncates silently is read as a population
/// (`gunbc.recurring_failure_mode instrument_output_read_as_subject_content`).
const CROSS_CLAIM_DEMAND_RETENTION_FLOOR_NS: u128 = 1_000_000;

/// Distinct census keys retained. Overflow is counted and disclosed, never silent.
const CROSS_CLAIM_DEMAND_KEY_CAP: usize = 200_000;

/// Consumer-module names RENDERED per row. The count is exact and comes from the complete set;
/// this bounds only how many names a row shows.
const CROSS_CLAIM_DEMAND_MODULE_SAMPLE_CAP: usize = 8;

#[derive(Default)]
struct CrossClaimDemandCensus {
    map: std::collections::HashMap<CrossClaimDemandKey, CrossClaimDemandCell>,
    claims_absorbed: u64,
    omitted_keys: u64,
    omitted_ns: u128,
    overflow_keys: u64,
    /// One `Rc<str>` per distinct module name, shared by every row that names it.
    module_names: std::collections::HashMap<String, Rc<str>>,
    /// WHAT THE INSTRUMENT ITSELF COSTS, measured rather than argued. A discovery instrument that
    /// materially raised every claim's cost would recreate the incident it explains, on a floor
    /// that preempts on marginal cost. Two facts make that checkable rather than assumed: the
    /// absorb runs AFTER `run_claim_measured` returns, so it is outside the measured window and
    /// cannot enter any claim's charged clocks or trip the deadline — and its own total and worst
    /// single claim are reported beside the census, so "outside the window" is not asked to stand
    /// alone.
    absorb_ns_total: u128,
    absorb_ns_max: u128,
}

thread_local! {
    /// Process-lifetime on the thread that runs the claim loop. The floor executes its claims
    /// sequentially in one process over one prepared subject, which is exactly the scope this
    /// census is about; a run whose claims were distributed would observe only its own share and
    /// the artifact would say so by its claim count rather than by a silent partial answer.
    static CROSS_CLAIM_DEMAND: RefCell<CrossClaimDemandCensus> =
        RefCell::new(CrossClaimDemandCensus::default());
}

fn cross_claim_demand_absorb_one(
    census: &mut CrossClaimDemandCensus,
    key: CrossClaimDemandKey,
    evals: u64,
    total_ns: u128,
    claim: &str,
    module_path: &str,
) {
    if total_ns < CROSS_CLAIM_DEMAND_RETENTION_FLOOR_NS && !census.map.contains_key(&key) {
        census.omitted_keys += 1;
        census.omitted_ns += total_ns;
        return;
    }
    if !census.map.contains_key(&key) && census.map.len() >= CROSS_CLAIM_DEMAND_KEY_CAP {
        census.overflow_keys += 1;
        return;
    }
    let module = match census.module_names.get(module_path) {
        Some(name) => name.clone(),
        None => {
            let name: Rc<str> = Rc::from(module_path);
            census
                .module_names
                .insert(module_path.to_string(), name.clone());
            name
        }
    };
    let cell = census
        .map
        .entry(key)
        .or_insert_with(|| CrossClaimDemandCell {
            claims: 0,
            evals: 0,
            total_ns: 0,
            last_claim: String::new(),
            modules: std::collections::BTreeSet::new(),
        });
    // One claim contributes ONE to `claims` however many times it evaluated the identity —
    // within-frame repetition is the existing ledger's subject and double-counting it here would
    // make an intra-claim loop look like cross-claim recurrence.
    if cell.last_claim != claim {
        cell.claims += 1;
        cell.last_claim = claim.to_string();
        // A set insert, so a module already present is not counted twice however many of its
        // claims arrive — the count is `len()` of the complete set and never an incremented
        // tally that can drift from it.
        cell.modules.insert(module);
    }
    cell.evals += evals;
    cell.total_ns += total_ns;
}

/// Fold ONE finished claim's recompute ledger into the cross-claim census, before the claim's
/// frame is dropped. Call it after the claim has run and before the next frame is built; a
/// no-op unless `GUNBC_RECOMPUTE_TRACE=1`, which the floor sets for itself.
///
/// THE CALLER MUST OWN A FRESH FRAME PER CLAIM, and that is a real precondition rather than a
/// style note. The ledger accumulates for the lifetime of its `InterpContext` and is NOT
/// cleared by `eval_call_memo_frame_exit`, so a surface that shares one context across several
/// claims — `claim_batch` shares one per ENTRY — would fold each claim's ledger again on the
/// next call, inflating both `evals` and `total_ns` and, worse, reporting cross-claim demand
/// where there is only one frame's. The required floor builds a frame per claim
/// (`required_floor_runner`, "FRESH PER CLAIM"), which is exactly the boundary this census
/// exists to measure across, so it is the only caller today. A shared-context surface wanting
/// this census needs a per-claim ledger reset first; getting that wrong would commit the class
/// this instrument was built to close, one grain up.
pub fn absorb_claim_recompute_demand(ctx: &InterpContext, claim: &str, module_path: &str) {
    if !eval_recompute_trace_enabled() {
        return;
    }
    let started = Instant::now();
    let t = ctx.eval_recompute_trace.borrow();
    // One declaration-site lookup per fn pointer for this frame, rather than a linear scan of
    // the keepalive vector per ledger key: the map holds one entry per (fn, argument row) and
    // the vector one per fn, so the scan was quadratic in a frame's producer count -- the
    // cost-shape defect DESIGN section 6 says is always fixed, inside an instrument whose whole
    // subject is cost.
    let mut decl_sites: std::collections::HashMap<usize, String> = std::collections::HashMap::new();
    for node in t.keepalive_fns.iter() {
        decl_sites
            .entry(Rc::as_ptr(node) as usize)
            .or_insert_with(|| eval_recompute_decl_site(node));
    }
    CROSS_CLAIM_DEMAND.with(|c| {
        let mut census = c.borrow_mut();
        census.claims_absorbed += 1;
        for (key, entry) in t.map.iter() {
            cross_claim_demand_absorb_one(
                &mut census,
                CrossClaimDemandKey {
                    producer: entry.fn_name.to_string(),
                    decl_site: decl_sites.get(&key.fn_ptr).cloned().unwrap_or_default(),
                    args: key.args.clone(),
                    arg_shape: "keyed",
                },
                entry.count,
                entry.total_ns,
                claim,
                module_path,
            );
        }
        for (name, (calls, ns, site)) in t.unkeyed_by_fn.iter() {
            cross_claim_demand_absorb_one(
                &mut census,
                CrossClaimDemandKey {
                    producer: name.clone(),
                    decl_site: site.clone(),
                    args: Vec::new(),
                    arg_shape: "unkeyed",
                },
                *calls,
                *ns,
                claim,
                module_path,
            );
        }
        let spent = started.elapsed().as_nanos();
        census.absorb_ns_total += spent;
        if spent > census.absorb_ns_max {
            census.absorb_ns_max = spent;
        }
    });
}

/// The census in deterministic identity order. EVERY retained row is returned,
/// single-claim rows included — they rank at zero and they are the control population that makes
/// `claims > 1` mean something, so filtering them here would leave an artifact in which every
/// row looks shared. Selecting the shared ones is the READER's move and it is a view rather than
/// the population: the runner applies it at the print site and the TSV keeps both.
///
/// (An earlier revision of this sentence said single-claim rows were dropped here. They never
/// were, the tests and the writer both assert they are not, and the sentence pointed a reader at
/// the wrong side of the census's one load-bearing claim. Found by review 58659.)
pub fn cross_claim_demand_rows() -> Vec<CrossClaimDemandRow> {
    CROSS_CLAIM_DEMAND.with(|c| {
        let census = c.borrow();
        let mut rows: Vec<CrossClaimDemandRow> = census
            .map
            .iter()
            .map(|(key, cell)| CrossClaimDemandRow {
                producer: key.producer.clone(),
                decl_site: key.decl_site.clone(),
                arg_shape: key.arg_shape,
                claims: cell.claims,
                evals: cell.evals,
                total_ns: cell.total_ns,
                // EXACT, read from the complete set. The bound below is on how many NAMES a
                // reader is shown and it cannot reach the count.
                modules: cell.modules.len() as u64,
                module_sample: cell
                    .modules
                    .iter()
                    .take(CROSS_CLAIM_DEMAND_MODULE_SAMPLE_CAP)
                    .map(|m| m.to_string())
                    .collect(),
            })
            .collect();
        // IDENTITY ORDER, NOT COST ORDER, and that is a DESIGN section 3 distinction rather than
        // a taste. An ordering by cost is a RANKING, and a ranking is a judgment about which
        // demands matter -- meaning, which belongs to .dag folding this artifact, not to the seed
        // whose warrant here is carrying facts across a frame boundary only the seed can reach.
        // So rows leave in a deterministic identity order with every cost column beside them, and
        // the runner's log preview sorts a COPY and says in band that it is a preview.
        rows.sort_by(|a, b| {
            a.producer
                .cmp(&b.producer)
                .then_with(|| a.decl_site.cmp(&b.decl_site))
                .then_with(|| a.arg_shape.cmp(b.arg_shape))
                .then_with(|| b.total_ns.cmp(&a.total_ns))
        });
        rows
    })
}

/// What the census did not retain, and what it cost to run. Both halves travel with the rows,
/// because either one read alone invites a wrong conclusion: the omissions bound what the ranking
/// cannot contain, and the overhead answers whether a discovery instrument on a cost-preempting
/// floor is paying for itself.
pub struct CrossClaimDemandDisclosure {
    pub claims_absorbed: u64,
    pub omitted_keys: u64,
    pub omitted_ns: u128,
    pub overflow_keys: u64,
    pub absorb_ns_total: u128,
    pub absorb_ns_max: u128,
}

/// The census's own statement of what it did not retain and what it cost. Every consumer of the
/// rows above must report this beside them.
pub fn cross_claim_demand_disclosure() -> CrossClaimDemandDisclosure {
    CROSS_CLAIM_DEMAND.with(|c| {
        let census = c.borrow();
        CrossClaimDemandDisclosure {
            claims_absorbed: census.claims_absorbed,
            omitted_keys: census.omitted_keys,
            omitted_ns: census.omitted_ns,
            overflow_keys: census.overflow_keys,
            absorb_ns_total: census.absorb_ns_total,
            absorb_ns_max: census.absorb_ns_max,
        }
    })
}

/// Reset — for tests, and for any caller that runs two independent floors in one process.
pub fn clear_cross_claim_demand_census() {
    CROSS_CLAIM_DEMAND.with(|c| *c.borrow_mut() = CrossClaimDemandCensus::default());
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

// Process-wide accumulator fed by InterpContext::drop, so EVERY eval path lands in the receipt
// by construction — harvest is not a per-call-site discipline a future site could forget.
// Sums at the totals grain only: raw ledger keys are address-based and single-ctx.
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
    // Counter invariant: a miss means the call was NOT served (it evaluated), so a cap-refused
    // store is still a miss — overflow ⊆ misses, and hits + misses == keyed Ok-resulting calls
    // through the memo path, including under overflow. `misses` is NOT "entries stored".
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

    // `field_summary_for_type` decides field access in TWO parts, and this evaluator used to
    // consume only the first. `value_shape: OptionalValue` is the checker's FUNCTOR LIFT: field
    // access through an `Optional` yields an `Optional`, PRESERVING the empty case rather than
    // erasing it. Reading the field directly off the `Optional` value instead -- which is what
    // dropping `value_shape` amounts to -- refuses with `NoSuchField { "Optional", .. }` on every
    // site the checker typed as a lift, so the two arms disagreed by construction. One authority,
    // both directions: the checker's decision is consumed here rather than re-derived.
    if matches!(
        summary.map(|s| &s.value_shape),
        Some(FieldValueShape::OptionalValue)
    ) {
        if let Value::Variant {
            type_name,
            variant_name,
            fields,
        } = &base_val
        {
            if *type_name == ctx.sym("Optional") {
                if *variant_name == ctx.sym("Absent") {
                    return Ok(optional_absent(ctx));
                }
                let inner = fields_get(fields, ctx.sym("value"))
                    .cloned()
                    .unwrap_or(Value::Null);
                let lifted = field_access_by_style(access_style, inner, &field_name, env, ctx)?;
                return Ok(optional_present(lifted, ctx));
            }
        }
    }

    field_access_by_style(access_style, base_val, &field_name, env, ctx)
}

/// The `access_style` half of a `FieldSummary`, applied to one already-selected value. Split out
/// so the `OptionalValue` lift above can run it on the payload rather than on the wrapper.
fn field_access_by_style(
    access_style: Option<&FieldAccessStyle>,
    base_val: Value,
    field_name: &str,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    match access_style {
        Some(FieldAccessStyle::TupleFirst) => match expect_list(&base_val, "tuple.first") {
            Ok(items) => Ok(items.front().cloned().unwrap_or(Value::Null)),
            Err(_) => extract_field(&base_val, field_name, env, ctx),
        },
        Some(FieldAccessStyle::TupleSecond) => match expect_list(&base_val, "tuple.second") {
            Ok(items) => Ok(items.get(1).cloned().unwrap_or(Value::Null)),
            Err(_) => extract_field(&base_val, field_name, env, ctx),
        },
        // This arm was written against the NULL-SENTINEL encoding, where an absent optional was
        // `Value::Null` and a present one was the bare element -- so returning `base_val` WAS the
        // unwrap. Constructing a real `Optional` variant for `first`/`last` retires that encoding
        // here, and returning `base_val` would now hand back the wrapper instead of its payload.
        // The `Absent` case keeps the pre-existing `Value::Null` result rather than being
        // repaired: the unwrap style has no plain value to yield there, and that gap is the
        // separately-filed `.value`-spelling ambiguity, not this change's subject.
        Some(FieldAccessStyle::OptionalUnwrap) => match &base_val {
            Value::Null => Ok(Value::Null),
            Value::Variant {
                type_name,
                variant_name,
                fields,
            } if *type_name == ctx.sym("Optional") => {
                if *variant_name == ctx.sym("Absent") {
                    Ok(Value::Null)
                } else {
                    Ok(fields_get(fields, ctx.sym("value"))
                        .cloned()
                        .unwrap_or(Value::Null))
                }
            }
            _ => Ok(base_val),
        },
        Some(FieldAccessStyle::EnumAccessor) => extract_field(&base_val, field_name, env, ctx),
        _ => extract_field(&base_val, field_name, env, ctx),
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
/// keyed-collection branch dispatching to it in `eval_record_lit`: bounded growth in the seed
/// interpreter, not a new Rust authority. Every DECISION is modeled and read out of `.dag` —
/// keyed-collection-ness is `04_types` `node_is_keyed_collection`, key string-likeness is
/// `05_emit_rust` `map_literal_key_is_string`, the SAME functions the emitter consults about
/// the same literal: the seed decides nothing, it projects one modeled decision onto its own
/// `Value` representation. Removing this without grounding that representation reopens the
/// fork — infer saying map, eval building a record.
///
/// Lane: ROADMAP `v1-interpreter-quarantine` → `v1-interpreter-delete`, counted against
/// `v1-honest-frontier`; the class is DESIGN's model↔realization fork thread (every primitive
/// modeled as a coproduct, realized as a native `Value`, reconciled by per-site bridges), of
/// which this is one bridge repaired, not added.
///
/// Checkable receipt, by execution: `w_map_typed_literal_is_a_map` in
/// `src/v1/tests/claim/ordinary_frontend_observation_test.dag` goes RED without this code
/// (`map_keys expects a map, got Record` — the refusal that made the ordinary front end
/// unreachable); `w_record_literal_is_still_a_record` goes RED if it over-converts. Both are
/// enrolled on the v1 claim scoped roster, so the deferral is counted, not asserted.
///
/// Deletion condition, narrower than the lane's: when a brace literal's representation is
/// DERIVED from its inferred type rather than reconstructed per consumer — the grounding half
/// of the thread, the move `#5428` made for the numeric tower — this function deletes outright
/// and the witness above is REPLACED by one over the grounded representation, not retired.
///
/// Build the `Value::Map` a keyed-collection literal denotes. Keys are the authored field
/// names, and string-likeness must be POSITIVELY established first via
/// `map_literal_key_is_string` — the function `05_emit_rust` asks about the same literal when
/// deciding quoted-and-owned vs bare — so interpreter and emitter cannot disagree about one
/// literal's keys. Anything not established is a typed, located refusal, not a guessed key: a
/// deny-list of bad key types lets every unlisted one through, the partial refusal that later
/// fails open (DESIGN §5; codex review 50168 caught exactly that shape here). The refusal arm
/// has no witness: a refusing data initializer stops module evaluation rather than returning a
/// Bool. That is can-climb-now-but-unbuilt, not cannot-climb — the trigger is an expecting-red
/// quarantine probe declaring a non-string-keyed map literal, the mechanism named beside the
/// witnesses in `src/v1/tests/claim/ordinary_frontend_observation_test.dag`.
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

    // A brace literal in a keyed-collection position IS a map; the single authority is the
    // literal's inferred type — the `node_is_keyed_collection` relation `05_emit_rust` reads
    // when rendering the same literal as a `HashMap`. Reading it here keeps interpreter and
    // emitter agreeing on one value's representation (DESIGN §3/§5: derived from one authority,
    // not reconciled per consumer). Before this the interpreter built a `Record`, `map_get`
    // limped through `raw_map_lookup`'s Record arm, and `map_keys(kernel_type_set)` refused —
    // the whole ordinary front end unreachable from interpreted `.dag`.
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
        // A pointer-keyed memo's one silent-wrongness class is address reuse: a freed cached
        // node replaced at the same address would answer a cast with ANOTHER type's name — no
        // crash, wrong type. Unreachable on the normal path (cast_target returns an AST-owned
        // child alive for the ctx's lifetime), but expr_child_at SYNTHESIZES a temporary error
        // node when a child is missing. Rather than imitate the keepalive discipline,
        // GUNBC_MEMO_VERIFY=1 recomputes on every hit and REFUSES on divergence, so the class is
        // checked by execution over the real corpus.
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
    // DEGENERATE RESOLUTIONS ARE NEVER CACHED, and the bound is load-bearing. expr_child_at
    // falls back to make_expr_error_node for a malformed cast, Rc::new-ing a FRESH node per
    // call (name "", ident_span None, inferred CompilerError). That resolves to an empty seed,
    // eval_cast's identity arm returns the Value::Str unchanged, the run continues, and a
    // malformed cast in a loop allocates a new address per iteration: permanent miss, unbounded
    // keepalive growth. Skipping the insert bounds the cache by the AST's real cast nodes, at
    // the cost of recomputing a path that short-circuits before any module scan; an empty seed
    // is exactly the signature of a target with no resolvable authored name.
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

/// The cast's SOURCE type name. Same defect and repair as the target side: a pure function of
/// the expression node that re-extracted source text per evaluation. One daily-page render:
/// +27.1ms across 59,858 casts (~452ns each, ExprCast 42.8ms -> 69.9ms) once #8098 added this
/// second `authored_name_at`. Memoized per node under the same pointer-key + keepalive
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
///
/// node://adhoc-897a90b6-a9c item 1: this also used to treat an EMPTY resolved kernel as
/// identity -- answering "the target's alias chain could not be resolved" with a value instead
/// of falling through to `eval_cast`'s typed `TypeError` arm (DESIGN §5, ⊥-as-ignorance as an
/// answer). Measured by execution (an unconditional counter, a positive/negative-control unit
/// test proving the counter wired, then 1507 requested witnesses across every
/// `*cast*`/`*refinement*`-named file in `dag/test/claim` plus a sixth-sample of the rest):
/// zero hits, `kernel_calls=19` real resolutions. The arm is reachable only when the target AST
/// node is itself a `CompilerError` node (malformed, missing-child target) -- a resolve-time
/// defect elsewhere, and resolve already refuses before such a node reaches eval on any real
/// program. Clean deletion; the fallthrough below answers instead.
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
        if kernel == "String" {
            return Some(Value::Str(s.clone()));
        }
    }
    None
}

#[cfg(test)]
mod cast_identity_empty_kernel_tests {
    //! Regression control for node://adhoc-897a90b6-a9c item 1: a cast target with an
    //! unresolvable alias chain (empty kernel) must NOT get silent Str identity. A malformed
    //! target node -- the shape `expr_child_at`'s fallback produces for a cast missing its
    //! target child -- makes `cast_identity_result` return `None`, so `eval_cast` falls through
    //! to its typed `TypeError` arm.
    use crate::v1_rt::RcStr;
    use std::rc::Rc;

    use im::{vector as im_vec, HashMap};

    use crate::v1_compiler_infer_emit_info::empty_emit_graph_info;
    use crate::v1_compiler_infer_items::ResolvedGraph;
    use crate::v1_std_core::{make_expr_error_node, no_span, ExprErrorKind};

    use super::{cast_identity_result, ExecutionMode, InterpContext, Value};

    fn fresh_ctx() -> InterpContext {
        let graph = ResolvedGraph {
            modules: Rc::new(im_vec![]),
            item_registry: Rc::new(HashMap::new()),
            diagnostics: Rc::new(im_vec![]),
            emit_graph_info: empty_emit_graph_info(),
        };
        InterpContext::new(&graph, Rc::new(HashMap::new()), ExecutionMode::Hermetic)
    }

    #[test]
    fn malformed_cast_target_no_longer_returns_silent_identity() {
        let ctx = fresh_ctx();
        // Exactly the node `expr_child_at`/`make_expr_error_node` produce for a cast whose
        // target child is missing: name "", inferred CompilerError (not Resolved) — the only
        // shape `cast_target_seed_name_uncached` returns "" for, i.e. an empty kernel.
        let malformed_target = make_expr_error_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            ExprErrorKind::InternalExprError,
            "malformed node: missing cast target".to_string(),
            no_span(),
        );
        let val = Value::Str(RcStr::from("payload"));
        let result = cast_identity_result(&val, &ctx, "", malformed_target, "");
        assert_eq!(
            result, None,
            "an unresolvable cast target must fall through to eval_cast's typed TypeError arm, \
             not return silent Str identity"
        );
    }

    #[test]
    fn well_formed_string_kernel_cast_still_returns_identity() {
        let ctx = fresh_ctx();
        // A target node named "String" resolves the seed directly (no alias walk), so the
        // kernel is non-empty and lands on the `kernel == "String"` arm. Negative control: the
        // deletion above did not remove the legitimate String-kernel identity case.
        let string_target = make_expr_error_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            ExprErrorKind::InternalExprError,
            "unused".to_string(),
            no_span(),
        );
        let string_target = Rc::new(crate::v1_std_core::Node {
            name: "String".to_string(),
            ..(*string_target).clone()
        });
        let val = Value::Str(RcStr::from("payload"));
        let result = cast_identity_result(&val, &ctx, "", string_target, "");
        assert_eq!(result, Some(Value::Str(RcStr::from("payload"))));
    }
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
            Value::Str(s) => Ok(Value::Str(s.clone())),
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
            if *i < 0 {
                return Ok(Value::Null);
            }
            let ch = s.char_at(*i);
            if ch.is_empty() {
                Ok(Value::Null)
            } else {
                Ok(str_value(ch))
            }
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
            if *s >= 0 && *e >= 0 {
                Ok(str_value(str_val.substring(*s, *e)))
            } else {
                // Negative indices wrap under the pre-carrier `as usize` cast; preserved
                // verbatim off the carrier path, which clamps to 0.
                let s = *s as usize;
                let e = *e as usize;
                let sliced: String = str_val.chars().skip(s).take(e.saturating_sub(s)).collect();
                Ok(str_value(sliced))
            }
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
                // String grounding (model↔realization): a native String arg makes the whole
                // `concat` a String realized as one `Value::Str` — provided the receiver is
                // string-like (all-codepoint). A `List<String>` receiver (`Str` *elements*) is
                // rejected by `free_monoid_to_string` and falls to the list path below, so
                // `["a","b"].concat("c")` stays a list.
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
                    // codepoint-bearing `Cons`-chain receiver here is the model↔realization
                    // straddle grounding above did not dissolve — refuse rather than push the
                    // `Str` into a mixed `[codepoint.., Str]` list. A `Value::List` receiver
                    // is a generic collection (`[1].append("ab")` is a legitimate two-element
                    // list) and a homogeneous `List<String>` carries no codepoint — both pass
                    // (the `orig` representation guard).
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

            // Known-method bridge parity: infer rewrites bare `is_empty(xs)` on import-stripped
            // modules into a method call (the census never serves algebra template names), so
            // eval implements the same member — emptiness via the shared length authority above.
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
                // std.algebra free_monoid_collection_templates declares
                //   first [ReceiverSelf] -> OptionalOf { inner: ReceiverElement }
                // so the result is an Optional OF THE ELEMENT: at element type U? the two
                // absences are distinct levels. Constructing them here -- rather than
                // returning a bare element or a Value::Null sentinel -- is the same
                // call-site construction `map_lookup_as_optional` already performs for maps.
                let items = expect_list(&$receiver, "first")?;
                Ok(match items.front().cloned() {
                    Some(v) => optional_present(v, $ctx),
                    None => optional_absent($ctx),
                })
            },

            arm "method_call.last" { "last" } => {
                // Same declaration, same construction: last [ReceiverSelf] ->
                // OptionalOf { inner: ReceiverElement }.
                let items = expect_list(&$receiver, "last")?;
                Ok(match items.last().cloned() {
                    Some(v) => optional_present(v, $ctx),
                    None => optional_absent($ctx),
                })
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

            arm "method_call.contains" { "contains" } => match &$receiver {
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
                // §6 residue: materializes a string as a `Value::List` of codepoint `Int`s,
                // indistinguishable from a generic `Int` list — the named hole in the
                // String-straddle wall (`string_realization_straddle_detail`'s `Value::List`
                // exemption). Closed by regrounding `Char`/codepoint-sequence so the realization
                // is distinguishable (grounding root, sibling #5428).
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
            // diverged surfaces over one builtin set that should be one authority. Pure-eval
            // logic in scope of ROADMAP HAND kernel D (`v1_interpreter` pure-eval dissolution,
            // interpreter-kernel-d (plan doc deleted 2026-08-28)): dissolution trigger is the
            // pure-eval seam (`emit_host` transport wiring) grounding this dispatch into
            // `v2.compiler.eval`, after which per-builtin arms stop being hand-Rust here.
            arm "method_call.map_keys" { "map_keys" } => {
                let m = expect_map(&$receiver, "map_keys")?;
                let keys: Vec<Value> = m.keys().map(|k| k.key.clone()).collect();
                Ok(list_value((keys)))
            },

            arm "method_call.sorted_map_keys" { "sorted_map_keys" } => {
                let m = expect_map(&$receiver, "sorted_map_keys")?;
                let keys: Vec<Value> = m.keys().map(|k| k.key.clone()).collect();
                Ok(list_value((sorted_map_keys_in_emitted_order(keys, "sorted_map_keys")?)))
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
                let s = expect_value_str(Some(&$receiver), "substring")?;
                match $args {
                    [start, end] => {
                        let s_idx = expect_int(Some(start), "substring start")?;
                        let e_idx = expect_int(Some(end), "substring end")?;
                        if s_idx >= 0 && e_idx >= 0 {
                            Ok(str_value(s.substring(s_idx, e_idx)))
                        } else {
                            // Negative indices wrap under the pre-carrier `as usize` cast;
                            // preserved verbatim off the carrier path, which clamps to 0.
                            let s_idx = s_idx as usize;
                            let e_idx = e_idx as usize;
                            let sliced: String = s
                                .chars()
                                .skip(s_idx)
                                .take(e_idx.saturating_sub(s_idx))
                                .collect();
                            Ok(str_value(sliced))
                        }
                    }
                    _ => Err(InterpError::TypeError {
                        msg: "substring requires (start, end) arguments".to_string(),
                    }),
                }
            },

            arm "method_call.char_at" { "char_at" } => {
                let s = expect_value_str(Some(&$receiver), "char_at")?;
                let idx = expect_int($args.first(), "char_at")?;
                if idx < 0 {
                    return Ok(Value::Null);
                }
                let ch = s.char_at(idx);
                if ch.is_empty() {
                    Ok(Value::Null)
                } else {
                    Ok(str_value(ch))
                }
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

/// Native read of THIS process's environment, matching the former `printenv` subprocess:
/// unset → None, empty → None, value trimmed. `dispatch_service_wet` routes `shell.Env.Get`
/// here for OnTarget locality (shell-to-dag residual census §0b); shell argv remains the
/// remote handler.
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
/// The consumer below swallows every failure arm into `None` (`_ => None`) — the tell that an
/// exit code is an observation, the shape `OutcomeIsData` exists for. Re-arming on that tell
/// alone would be wrong, by a state-space conflation one level down: `None` means BOTH "the
/// variable is unset" (a legitimate answer) and "the `shell.Env.Get` dispatch failed" (a broken
/// probe). `OutcomeIsData` would make a broken env service indistinguishable from an empty
/// environment — a silent conflation, worse than the crude loud arm.
///
/// The fix is a PAIR: split the consumer's `None` into probe-broken (refuses) vs answer-absent
/// (`None`), then re-arm this site to `OutcomeIsData`. Recorded rather than done here because
/// the split changes this function's return type and every caller — its own change with its
/// own witnesses.
fn resolve_env_var_token(ctx: &InterpContext, var_name: &str) -> Option<String> {
    if ctx.indexes.service_ops.contains_key("shell.Env.Get") {
        let args = [(Some("name".to_string()), str_value(var_name.to_string()))];
        match eval_service_call(
            "shell.Env",
            "Get",
            &args,
            &Env::empty(),
            ctx,
            // Interpreter-own seam: no .dag call node carries `expect:`, so the arm is stated
            // here, not defaulted. See `resolve_env_var_token_expectation_note` below for why
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

/// Decide whether a hermetic readonly `Filesystem` operation on `requested` is checkout-input
/// access under `root`: the canonicalized path must sit under the canonicalized root with no
/// `.git` or `target` component below it (branch state and build artifacts are host state, not
/// commit-deterministic input). Err carries the typed refusal cause; the caller never widens
/// a failure into a canned response.
fn hermetic_checkout_input_disposition_under(
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
    // AN ABSENT PATH UNDER THE ROOT IS AS COMMIT-DETERMINISTIC AS A PRESENT ONE, which
    // canonicalizing the REQUESTED leaf could not express: `canonicalize` requires the whole
    // path to exist, so it failed identically for three dispositions with different remedies —
    // a file the commit says is NOT THERE, a path RESOLVING OUTSIDE the root, and one
    // UNRESOLVABLE otherwise. One error string for three states is the conflation
    // `HermeticEffectGround` was split to prevent, twenty lines above the enum recording it.
    //
    // The commit determines absence as it determines contents: "no such file" is read OFF THE
    // INPUT, not host state, so it belongs on the same wet arm as a successful read, not in the
    // mock layer — a canned response there would serve every unconfirmed path, fabricating
    // "not found" for out-of-root and `.git` reads too.
    //
    // So resolve the deepest ancestor that DOES exist and carry the absent tail separately. The
    // existing prefix is what symlinks can move, so it is what must be proven under the root;
    // the absent tail cannot be a symlink. A `..` in the tail yields no `file_name`, so it
    // exits through the unresolvable arm -- refusing what cannot be confirmed.
    let mut probe = joined.as_path();
    let mut absent_tail: Vec<std::ffi::OsString> = Vec::new();
    let canon = loop {
        match std::fs::canonicalize(probe) {
            Ok(resolved) => break resolved,
            Err(e) => {
                // ONLY A GENUINE ABSENCE MAY PEEL A COMPONENT. A permission-denied or
                // symlink-loop error from `canonicalize` says the path CANNOT BE CONFIRMED —
                // the unresolvable arm, not a missing component. Treating every error as
                // absence would climb past a component that is really there.
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(format!(
                        "path does not canonicalize under the checkout ({e})"
                    ));
                }
                // A DANGLING SYMLINK IS PRESENT, NOT ABSENT, AND `canonicalize` CANNOT SAY SO:
                // it resolves symlinks, so a link with a missing target returns the SAME
                // NotFound as an empty name. `symlink_metadata` does not follow, answering the
                // real question: is there an entry here? If so, its resolution depends on a
                // target the commit does not contain -- host state whose later appearance
                // would silently change this read -- so it refuses rather than peeling.
                if std::fs::symlink_metadata(probe).is_ok() {
                    return Err(format!(
                        "path does not canonicalize under the checkout (`{}` exists but does \
                         not resolve -- a dangling symlink's target is host state, not a \
                         commit input)",
                        probe.display()
                    ));
                }
                let (parent, name) = match (probe.parent(), probe.file_name()) {
                    (Some(parent), Some(name)) if parent != probe => (parent, name.to_os_string()),
                    _ => {
                        return Err(format!(
                            "path does not canonicalize under the checkout ({e})"
                        ))
                    }
                };
                absent_tail.push(name);
                probe = parent;
            }
        }
    };
    absent_tail.reverse();

    if !canon.starts_with(&root) {
        return Err(format!(
            "path resolves outside the checkout root {}",
            root.display()
        ));
    }
    let rel = canon
        .strip_prefix(&root)
        .expect("starts_with checked above");
    let existing_names = rel.components().filter_map(|comp| match comp {
        std::path::Component::Normal(name) => Some(name.to_os_string()),
        _ => None,
    });
    // The tail is checked on the SAME rule as the existing prefix: `target/does-not-exist` is
    // no more commit-deterministic than `target/receipt.txt`, and admitting it because it is
    // missing would make the wall depend on the state of the directory it exists to exclude.
    for name in existing_names.chain(absent_tail.into_iter()) {
        if name == std::ffi::OsStr::new(".git") || name == std::ffi::OsStr::new("target") {
            return Err(format!(
                "`{}` components are not commit-deterministic inputs",
                name.to_string_lossy()
            ));
        }
    }
    Ok(())
}

/// The runner contract binds the process cwd to the checkout root (claim_batch and
/// claim_executor both run from the repo root), so cwd IS the injected input root.
fn hermetic_checkout_input_disposition(requested: &str) -> Result<(), String> {
    let cwd =
        std::env::current_dir().map_err(|e| format!("checkout root (cwd) unresolvable: {e}"))?;
    hermetic_checkout_input_disposition_under(&cwd, requested)
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
        // Checkout-input carve-out: the commit IS the run's injected input, so a readonly
        // Filesystem.Read or Filesystem.List of a path proven under the checkout root stays a
        // REAL read in hermetic mode — input access, not a host effect. Everything else is
        // fail-closed: an out-of-root path, a `.git`/`target` component (not
        // commit-deterministic), or an unresolvable path each refuse with a typed diagnostic —
        // never a canned response.
        if service_name == "Filesystem" && matches!(op_name, "Read" | "List") {
            // Single-authority split (§3): a readonly Filesystem operation whose path the
            // disposition CONFIRMS is a committed checkout input reads directly — input access,
            // not a host effect, needing no fixture. Everything it cannot confirm — a recorded
            // fixture's scratch path, a `target/`/`.git` artifact, an out-of-root or absent
            // path — FALLS THROUGH to the fixture-store / published-mock / fail-closed machinery
            // below, which owns non-deterministic host state. The carve-out never pre-empts the
            // recorded-fixture mechanism (record/replay/staleness) nor widens a host-state read
            // into a refusal belonging to the mock layer. Checkout inputs read from the commit;
            // host state is mocked or fails closed — no path is served by both.
            let confirmed_checkout_input = param_env
                .lookup(ctx.sym("path"))
                .and_then(|v| match v {
                    Value::Str(s) => Some(s.to_string()),
                    _ => None,
                })
                .map(|requested| hermetic_checkout_input_disposition(&requested).is_ok())
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
            return Err(InterpError::HermeticHostEffectRefused {
                operation: key.clone(),
                ground: HermeticEffectGround::UnpublishedMockCase {
                    published_cases: cases.into_iter().cloned().collect(),
                },
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
        // An active witness replay frame OUTRANKS the published-mock layer: `eval_mock_response`
        // replays the operation RESULT off the declaration; a replay frame supplies the
        // transport OBSERVATION and requires the real dispatcher fold on top of it. Mock-first
        // is the fail-open the seam closes: the fixture greens while the dispatcher is never
        // reached, so a broken dispatcher is unobservable in the mode CI runs (hermetic).
        // Measured: `rest_transport_failure_is_persistable` returns true under `gunbc run
        // --claim-run` (Wet, reaches `dispatch_rest`) and false under `claim_batch` (Hermetic,
        // answered here) on one binary and one tree.
        //
        // Fail-closed on both arms, never a widen (§5): a REST transport routes to ordinary wet
        // dispatch so `rest_exchange_selection` decides, and that already refuses
        // `RestReplayExchangeAbsent`/`Ambiguous` BEFORE any socket opens — an active frame with
        // no matching fixture is a typed refusal, not a live request escaping hermetic mode.
        // Every other transport refuses here: a replay intent this machinery cannot honor stops
        // the line rather than degrading to the mock (a fabricated answer) or a real shell/file
        // effect.
        //
        // HAND-RUST GATE — seed-retained, lane `v1-materialization-kernel`
        // (rn_53JPH6BB7G588K7DMZNWM0E3AS, witness-realization-plan (plan doc deleted 2026-08-28)),
        // terminating at `v1-interpreter-quarantine` → `v1-interpreter-delete`; the lane the
        // `WITNESS_EVALUATION_FRAMES` deferral above names. Deletion condition, checkable by
        // execution: when witnesses emit to native code and the emitted runtime realizes the
        // evaluation frame, this arm deletes with that stack while
        // `rest_transport_failure_is_persistable` stays green under the corpus runner without
        // it — that witness is this arm's regression control, not merely the frame's, and reds
        // if the mock layer ever preempts a replay frame again.
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
    // Local Env.Get: reading THIS process's environment is not a host effect (shell-to-dag
    // residual census §0b / DESIGN §3(b)). `printenv` was the wrong hardwired transport —
    // unset vars exited 1 and, under ExpectSuccess, every optional floor_diff_observe
    // injection (GUNBC_CI_DIFF_*) painted Anomaly Failed lines (operator live-log 2026-07-25).
    // Native handler; shell printenv remains the remote-target realization.
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

/// Expand a `ProcessArgvExpansion` into argv words, one word per `CliArgument`.
///
/// SEED REALIZATION OF A MODELED LAW, not a seed-only special case. The authority is
/// `v2.std.compilers.cli_surface` `ProcessArgvExpansion`, which states:
///
///     across the surface's arguments   -- ITERATE, never concatenate
///     within one argument's fragments  -- CONCATENATE into exactly one word
///
/// WHY AN EXPLICIT ARM RATHER THAN A BETTER GUESS. `push_shell_argv_tokens` below decides
/// "one word or many?" from the RUNTIME ENCODING, which erases the distinction: a
/// `FreeMonoid<Str>` is equally a String assembled from fragments and a list of strings.
/// `value_as_host_string` and `free_monoid_to_vec` BOTH succeed on it, so no branch order
/// recovers intent -- reordering splices lists correctly and shreds modeled strings. The
/// missing fact is the transport ROLE, supplied nominally by this carrier.
///
/// FAIL-CLOSED. Every shape mismatch is a typed error; no arm falls through to the guessing
/// path — a carrier saying "expand this" that silently produced one joined word would be the
/// defect it exists to remove (DESIGN section 5).
fn push_process_argv_expansion(
    argv: &mut Vec<String>,
    fields: &[(Symbol, Value)],
) -> InterpResult<()> {
    let surface = fields
        .iter()
        .find(|(name, _)| resolve_sym(*name).rsplit('.').next() == Some("surface"))
        .map(|(_, v)| v)
        .ok_or_else(|| InterpError::TypeError {
            msg: "ProcessArgvExpansion carries no `surface` field; argv expansion refuses rather \
                  than emitting a guessed word"
                .to_string(),
        })?;
    let arguments = match surface {
        Value::Record { type_name, fields } => {
            if resolve_sym(*type_name).rsplit('.').next() != Some("CliSurface") {
                return Err(InterpError::TypeError {
                    msg: format!(
                        "ProcessArgvExpansion.surface must be a CliSurface record, found `{}`",
                        resolve_sym(*type_name)
                    ),
                });
            }
            fields
                .iter()
                .find(|(name, _)| resolve_sym(*name).rsplit('.').next() == Some("arguments"))
                .map(|(_, v)| v.clone())
                .ok_or_else(|| InterpError::TypeError {
                    msg: "CliSurface carries no `arguments` field".to_string(),
                })?
        }
        other => {
            return Err(InterpError::TypeError {
                msg: format!(
                    "ProcessArgvExpansion.surface must be a CliSurface record, found `{other}`"
                ),
            })
        }
    };
    let items = match &arguments {
        Value::List(items) => items.iter().cloned().collect::<Vec<_>>(),
        variant @ Value::Variant { .. } => {
            free_monoid_to_vec(variant).ok_or_else(|| InterpError::TypeError {
                msg: "CliSurface.arguments is neither a native list nor a list-shaped value"
                    .to_string(),
            })?
        }
        other => {
            return Err(InterpError::TypeError {
                msg: format!("CliSurface.arguments must be a list, found `{other}`"),
            })
        }
    };
    for item in items {
        let text = match &item {
            Value::Record { type_name, fields } => {
                if resolve_sym(*type_name).rsplit('.').next() != Some("CliArgument") {
                    return Err(InterpError::TypeError {
                        msg: format!(
                            "CliSurface.arguments member must be a CliArgument record, found `{}`",
                            resolve_sym(*type_name)
                        ),
                    });
                }
                fields
                    .iter()
                    .find(|(name, _)| resolve_sym(*name).rsplit('.').next() == Some("text"))
                    .map(|(_, v)| v.clone())
                    .ok_or_else(|| InterpError::TypeError {
                        msg: "CliArgument carries no `text` field".to_string(),
                    })?
            }
            other => {
                return Err(InterpError::TypeError {
                    msg: format!(
                        "CliSurface.arguments member must be a CliArgument, found `{other}`"
                    ),
                })
            }
        };
        // Concatenation is CORRECT here and only here: this is one argument's own text.
        let word = value_as_host_string(&text).ok_or_else(|| InterpError::TypeError {
            msg: "CliArgument.text is not a host string".to_string(),
        })?;
        // An EMPTY argument is real and pushed unchanged: `jq ""` passes one empty word, and
        // dropping it would change arity -- the concatenating path's boundary destruction in
        // the other direction.
        //
        // An embedded NUL REFUSES before the exec boundary: a C argv is NUL-terminated, so
        // std::process would reject it at spawn with an errno-shaped error naming neither the
        // argument nor the surface. Refusing here yields a located diagnostic (DESIGN section
        // 5: refuse, never widen or fabricate).
        if word.contains('\0') {
            return Err(InterpError::TypeError {
                msg: format!(
                    "CliArgument.text contains an embedded NUL at argv position {}; a process \
                     argument vector is NUL-terminated and cannot carry one",
                    argv.len()
                ),
            });
        }
        argv.push(word);
    }
    Ok(())
}

fn push_shell_argv_tokens(argv: &mut Vec<String>, val: Value) -> InterpResult<()> {
    match &val {
        Value::Record { type_name, fields }
            if resolve_sym(*type_name).rsplit('.').next() == Some("ProcessArgvExpansion") =>
        {
            push_process_argv_expansion(argv, fields)
        }
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
        // THE AMBIGUITY IS REFUSED HERE, NOT RESOLVED — both readings are well-formed, so no
        // ordering of the old arms could be correct.
        //
        // A free monoid of `Str` satisfies BOTH: `value_as_host_string` folds it into one word
        // (its `Value::Str(s) => out.push_str(&s)` arm), `free_monoid_to_vec` splices it into N.
        // The value records no choice, so the previous code took the first — one declared
        // `List<String>` produced ONE argv word monoid-encoded and N words as a native list.
        // Same type, opposite arity, decided by a representation the author never selected.
        //
        // The measured specimen: `extdeps.git.git` `git_diff_range_argv` returns `[base, head]`
        // on its TwoDot arm, spliced into `git diff -U0 <range>`. Monoid-encoded it reaches the
        // process as `mainHEAD` — and on any pair whose concatenation names a real object git
        // succeeds with a diff of the WRONG RANGE: fabricated plausible output, not a crash
        // (DESIGN §5).
        //
        // A state-space conflation, not a missing wall: "one argument whose text is the
        // concatenation" and "N arguments" are different states with different remedies, and a
        // position that cannot tell them apart must refuse rather than pick.
        //
        // WHAT DELIBERATELY DOES NOT CHANGE:
        //   * Int-element monoids stay char-decoded into one word: a code-point sequence is a
        //     host string under one reading only.
        //   * A native `Value::List` keeps its N-word expansion (arm above) — it states the list
        //     reading structurally.
        //   * `ProcessArgvExpansion` stays authoritative (arm at the top) — it states the role.
        //   * `value_as_host_string` is UNTOUCHED. `value_to_host_string` wraps it for general
        //     use, and narrowing a shared helper for one caller is the forked-logic trap this
        //     lane removes. The discrimination belongs to the argv position, so it lives here.
        //
        // An EMPTY monoid is the empty string, preserving existing behaviour. Stated explicitly:
        // it is the one input where the two readings differ in arity (one empty word vs none)
        // and this arm still picks — a deliberate narrow choice with no observed consumer.
        Value::Variant { .. } => {
            if let Some(items) = free_monoid_to_vec(&val) {
                let has_str_element = items.iter().any(|i| matches!(i, Value::Str(_)));
                if has_str_element {
                    return Err(InterpError::TypeError {
                        msg: format!(
                            "argv position {}: a modeled sequence of {} string element(s) is \
                             ambiguous here — it reads BOTH as one argument whose text is their \
                             concatenation AND as {} separate arguments, and the value records no \
                             choice. Refusing rather than picking. Say which is meant: wrap the \
                             surface in ProcessArgvExpansion for the argument-list reading, or \
                             join the parts explicitly for the single-word reading",
                            argv.len(),
                            items.len(),
                            items.len()
                        ),
                    });
                }
            }
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
/// `src/v2/std/operation_argv.dag` arm for arm — the .dag module owns the vocabulary, this
/// enum is its seed realization. Every arm refuses; none widens, defaults, or sanitizes
/// (DESIGN §5).
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

/// The AUTHORED name of an `ExprData` form, TOTAL over the closed `.dag` vocabulary
/// (`v1.core` `ExprData`), and the single such authority in the seed.
///
/// Lets a projection with no rule for a form REFUSE BY NAME instead of substituting a
/// plausible value, using the `.dag` declaration's own name. Replaced `argv_expr_kind_label`,
/// which returned hyphenated nicknames (`call`, `record-literal`): a refusal naming `call`
/// cannot be grepped back to `ExprCall`, and two spellings of one closed vocabulary is the
/// DESIGN section 3 nickname at the diagnostic layer.
///
/// NO catch-all, on purpose: a form added to the `.dag` coproduct must break this compile,
/// keeping the seed's vocabulary equal to the substrate's rather than merely older.
pub(crate) fn expr_data_form_name(expr_data: &ExprData) -> &'static str {
    match expr_data {
        ExprData::NoExprData => "NoExprData",
        ExprData::ExprLiteral { .. } => "ExprLiteral",
        ExprData::ExprElaboratedLiteral { .. } => "ExprElaboratedLiteral",
        ExprData::ExprError { .. } => "ExprError",
        ExprData::ExprVar { .. } => "ExprVar",
        ExprData::ExprFieldAccess { .. } => "ExprFieldAccess",
        ExprData::ExprCall { .. } => "ExprCall",
        ExprData::ExprMethodCall { .. } => "ExprMethodCall",
        ExprData::ExprMatch => "ExprMatch",
        ExprData::ExprIf => "ExprIf",
        ExprData::ExprLet => "ExprLet",
        ExprData::ExprRecordLit { .. } => "ExprRecordLit",
        ExprData::ExprListLit => "ExprListLit",
        ExprData::ExprBinOp { .. } => "ExprBinOp",
        ExprData::ExprUnaryOp { .. } => "ExprUnaryOp",
        ExprData::ExprLambda => "ExprLambda",
        ExprData::ExprStringInterp => "ExprStringInterp",
        ExprData::ExprBlock => "ExprBlock",
        ExprData::ExprCast => "ExprCast",
        ExprData::ExprForEach => "ExprForEach",
        ExprData::ExprIndex => "ExprIndex",
        ExprData::ExprSlice => "ExprSlice",
        ExprData::ExprReturn => "ExprReturn",
    }
}

/// The authored name of a `LiteralValue` form, TOTAL over the closed `.dag` vocabulary
/// (`std.syntax` `LiteralValue`). Same construction and same reason as
/// `expr_data_form_name`: no catch-all, so a new literal form stops the compile here.
pub(crate) fn literal_value_form_name(value: &crate::std_syntax::LiteralValue) -> &'static str {
    use crate::std_syntax::LiteralValue;
    match value {
        LiteralValue::LitStr { .. } => "LitStr",
        LiteralValue::LitInt { .. } => "LitInt",
        LiteralValue::LitFloat { .. } => "LitFloat",
        LiteralValue::LitBool { .. } => "LitBool",
        LiteralValue::LitNull => "LitNull",
        LiteralValue::LitSymbol { .. } => "LitSymbol",
    }
}

/// A declared default, read from the declaration. Only literal *data* is admitted (a string
/// literal, or a list literal of string literals): a call or reference default is left
/// UNBOUND, so an argv position needing it refuses by name rather than being filled with a
/// guess (DESIGN §5 — a fabricated plausible default is the failure this avoids).
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
                    Some(Value::Str(text)) => Value::Str(text.clone()),
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

/// Bindings ∪ declared defaults, validated against the operation's OWN declared inputs. A
/// binding naming an undeclared input is refused, not injected — the previous materializer
/// unconditionally injected `package`, `bin`, `args`, `unit` and `property` into every
/// operation, the channel a generic binder must not keep open.
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
                "argv element literal is `{}`, expected a string literal",
                literal_value_form_name(other)
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
            expr_data_form_name(node.expr_data.as_ref())
        ))),
    }
}

/// Materialize an operation's transport argv by binding its own declared inputs.
///
/// The executable position is a construction wall, not a lenient check: `argv[0]` must be
/// a string literal in the declaration, so no binding — declared or not — decides which
/// program runs.
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
            expr_data_form_name(executable.expr_data.as_ref())
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
/// The two arms stay distinct to the substrate — `CensusNotRunnable` must never arrive as
/// `CensusObserved` with an empty row list, byte-identical to a clean compile, letting
/// could-not-measure read as the subject passing (DESIGN §5).
/// Encode one `ReferenceDerivedClosureAdmission` as an interpreter value.
///
/// Pure encoding. The judgement was made by
/// `gunbc.namespace_reference_derived_closure_admission assess_reference_binding_observation`,
/// which the `.dag` producer calls; nothing here decides, reclassifies, or defaults. The
/// three payload vocabularies are nullary coproducts, carried by name.
fn reference_derived_closure_admission_value(
    admission: &crate::gunbc_namespace_reference_derived_closure_admission::ReferenceDerivedClosureAdmission,
    ctx: &InterpContext,
) -> Value {
    use crate::gunbc_namespace_reference_derived_closure_admission::ReferenceDerivedClosureAdmission as A;
    let nullary = |type_name: &str, variant_name: String| Value::Variant {
        type_name: ctx.sym(type_name),
        variant_name: ctx.sym(&variant_name),
        fields: Rc::new(Vec::new()),
    };
    // The variant NAME is spelled by a total match, never derived from `Debug` (review 57100).
    // Debug prints exactly the variant name for these nullary enums today and stops the moment
    // a variant gains a payload (`Foo { .. }`), minting a name no `.dag` match accepts — loud,
    // but at RUNTIME; a total match makes it a COMPILE error (construction over validation,
    // DESIGN 5) and stops the Rust identifier being a second representation of the variant's
    // identity (DESIGN 3).
    let capability = |c: &crate::gunbc_namespace_reference_derived_closure_admission::ReferenceDerivedClosureCapability| {
        use crate::gunbc_namespace_reference_derived_closure_admission::ReferenceDerivedClosureCapability as C;
        let name = match c {
            C::SameFileEarlierNeighbourVisible => "SameFileEarlierNeighbourVisible",
            C::SiblingDecisionBranchExcluded => "SiblingDecisionBranchExcluded",
            C::LaterDeclarationExcluded => "LaterDeclarationExcluded",
            C::DistinctSameSpelledDeclarationsPreserved => "DistinctSameSpelledDeclarationsPreserved",
            C::RepeatedMentionsCollapseDependency => "RepeatedMentionsCollapseDependency",
            C::UnrelatedLoadedFileExcluded => "UnrelatedLoadedFileExcluded",
        };
        nullary("ReferenceDerivedClosureCapability", name.to_string())
    };
    let trigger_value = |t: &crate::gunbc_namespace_reference_derived_closure_admission::ReferenceDerivedClosureTrigger| {
        use crate::gunbc_namespace_reference_derived_closure_admission::ReferenceDerivedClosureTrigger as T;
        let name = match t {
            T::P2aStructuralCandidateProducer7515 => "P2aStructuralCandidateProducer7515",
            T::P2aReferenceDependencyProjection7515 => "P2aReferenceDependencyProjection7515",
            T::P2aPoolIndependentDependencyProjection7515 => "P2aPoolIndependentDependencyProjection7515",
        };
        nullary("ReferenceDerivedClosureTrigger", name.to_string())
    };
    let scenario_failure = |f: &crate::gunbc_namespace_reference_derived_closure_admission::ReferenceDerivedClosureScenarioFailure| {
        use crate::gunbc_namespace_reference_derived_closure_admission::ReferenceDerivedClosureScenarioFailure as F;
        let name = match f {
            F::SameFileNeighbourMissing => "SameFileNeighbourMissing",
            F::SiblingBranchLeaked => "SiblingBranchLeaked",
            F::LaterDeclarationLeaked => "LaterDeclarationLeaked",
            F::DistinctDeclarationCollapsed => "DistinctDeclarationCollapsed",
            F::RepeatedMentionDuplicatedDependency => "RepeatedMentionDuplicatedDependency",
            F::UnrelatedLoadedFileDependencyLeaked => "UnrelatedLoadedFileDependencyLeaked",
        };
        nullary("ReferenceDerivedClosureScenarioFailure", name.to_string())
    };
    match admission {
        A::ReferenceDerivedClosureEstablished { receipt } => Value::Variant {
            type_name: ctx.sym("ReferenceDerivedClosureAdmission"),
            variant_name: ctx.sym("ReferenceDerivedClosureEstablished"),
            fields: Rc::new(sorted_fields(vec![(
                ctx.sym("receipt"),
                Value::Record {
                    type_name: ctx.sym("ReferenceDerivedClosureAcceptanceReceipt"),
                    fields: Rc::new(sorted_fields(vec![(
                        ctx.sym("capability"),
                        capability(&receipt.capability()),
                    )])),
                },
            )])),
        },
        A::ReferenceDerivedClosureRefused {
            capability: c,
            failure,
        } => Value::Variant {
            type_name: ctx.sym("ReferenceDerivedClosureAdmission"),
            variant_name: ctx.sym("ReferenceDerivedClosureRefused"),
            fields: Rc::new(sorted_fields(vec![
                (ctx.sym("capability"), capability(c)),
                (ctx.sym("failure"), scenario_failure(failure)),
            ])),
        },
        A::ReferenceDerivedClosureUnavailable {
            capability: c,
            trigger,
        } => Value::Variant {
            type_name: ctx.sym("ReferenceDerivedClosureAdmission"),
            variant_name: ctx.sym("ReferenceDerivedClosureUnavailable"),
            fields: Rc::new(sorted_fields(vec![
                (ctx.sym("capability"), capability(c)),
                (ctx.sym("trigger"), trigger_value(trigger)),
            ])),
        },
    }
}

/// Encode the rows for one in-memory source: each admission the `.dag` producer computed,
/// paired with the module path that producer read out of the same parse.
///
/// Both producer exports are pure functions of `(file, source)` and the caller passes one
/// pair to both, so this join cannot pair a module path with another subject's admissions.
/// Mechanical, never a decision; the split exists because a `.dag` record embedding a
/// `sole_constructor`-bearing coproduct does not emit compilably today (see the producer's
/// note).
/// Encode the subject's identity as the observation it is: the module the parse declared, or
/// a typed refusal naming the gap. No spelling makes "the parse refused" and "the module
/// path is empty" the same value.
fn subject_module_value(
    subject: &crate::std_reference_binding_observation::StructuralObservationSubjectModule,
    ctx: &InterpContext,
) -> Value {
    use crate::std_reference_binding_observation::ReferenceBindingProductionGap as G;
    use crate::std_reference_binding_observation::StructuralObservationSubjectModule as S;
    match subject {
        S::SubjectModuleProduced { module_path } => Value::Variant {
            type_name: ctx.sym("StructuralObservationSubjectModule"),
            variant_name: ctx.sym("SubjectModuleProduced"),
            fields: Rc::new(sorted_fields(vec![(
                ctx.sym("module_path"),
                str_value(module_path.clone()),
            )])),
        },
        S::SubjectModuleProductionRefused { gap } => {
            let gap_name = match gap {
                G::ReferenceBindingParserTransportRefused => {
                    "ReferenceBindingParserTransportRefused"
                }
                G::ReferenceBindingNamedReferenceAbsent => "ReferenceBindingNamedReferenceAbsent",
                G::ReferenceBindingNamedDeclarationAbsent => {
                    "ReferenceBindingNamedDeclarationAbsent"
                }
            };
            Value::Variant {
                type_name: ctx.sym("StructuralObservationSubjectModule"),
                variant_name: ctx.sym("SubjectModuleProductionRefused"),
                fields: Rc::new(sorted_fields(vec![(
                    ctx.sym("gap"),
                    Value::Variant {
                        type_name: ctx.sym("ReferenceBindingProductionGap"),
                        variant_name: ctx.sym(gap_name),
                        fields: Rc::new(Vec::new()),
                    },
                )])),
            }
        }
    }
}

/// Encode one source's parsed import statements: the parser's delimited spans, or the typed
/// refusal. A source that did not parse and a source with no imports are different values here
/// because they are different facts, and a caller that stripped nothing from the first would
/// report a clean rewrite of a file it never read.
fn parsed_import_statements_value(
    outcome: &crate::std_import::ParsedImportStatements,
    ctx: &InterpContext,
) -> Value {
    use crate::std_import::ParsedImportStatements as P;
    match outcome {
        P::ImportStatementsParsed { statements } => Value::Variant {
            type_name: ctx.sym("ParsedImportStatements"),
            variant_name: ctx.sym("ImportStatementsParsed"),
            fields: Rc::new(sorted_fields(vec![(
                ctx.sym("statements"),
                list_value(
                    statements
                        .iter()
                        .map(|statement| Value::Record {
                            type_name: ctx.sym("ParsedImportStatement"),
                            fields: Rc::new(sorted_fields(vec![
                                (
                                    ctx.sym("span"),
                                    Value::Record {
                                        type_name: ctx.sym("SourceSpan"),
                                        fields: Rc::new(sorted_fields(vec![
                                            (
                                                ctx.sym("file"),
                                                str_value(statement.span.file.clone()),
                                            ),
                                            (ctx.sym("start"), Value::Int(statement.span.start)),
                                            (ctx.sym("end"), Value::Int(statement.span.end)),
                                        ])),
                                    },
                                ),
                                (
                                    ctx.sym("imported_module"),
                                    str_value(statement.imported_module.clone()),
                                ),
                            ])),
                        })
                        .collect::<Vec<_>>(),
                ),
            )])),
        },
        P::ImportStatementParseRefused { cause } => Value::Variant {
            type_name: ctx.sym("ParsedImportStatements"),
            variant_name: ctx.sym("ImportStatementParseRefused"),
            fields: Rc::new(sorted_fields(vec![(
                ctx.sym("cause"),
                import_statement_parse_cause_value(cause.clone(), ctx),
            )])),
        },
    }
}

fn import_statement_parse_cause_value(
    cause: Rc<crate::std_import::ImportStatementParseCause>,
    ctx: &InterpContext,
) -> Value {
    use crate::std_import::ImportStatementParseCause as C;
    let (variant, fields) = match &*cause {
        C::SourceHasNoModuleDeclaration => ("SourceHasNoModuleDeclaration", vec![]),
        C::ModuleDeclarationPathMalformed => ("ModuleDeclarationPathMalformed", vec![]),
        C::ImportStatementMalformed => ("ImportStatementMalformed", vec![]),
        C::ImportParseInstrumentAnomaly { detail } => (
            "ImportParseInstrumentAnomaly",
            vec![(ctx.sym("detail"), str_value(detail.clone()))],
        ),
    };
    Value::Variant {
        type_name: ctx.sym("ImportStatementParseCause"),
        variant_name: ctx.sym(variant),
        fields: Rc::new(sorted_fields(fields)),
    }
}

fn namespace_structural_observation_admissions_value(
    compiled_module: &crate::std_reference_binding_observation::StructuralObservationSubjectModule,
    admissions: &[Rc<crate::gunbc_namespace_reference_derived_closure_admission::ReferenceDerivedClosureAdmission>],
    ctx: &InterpContext,
) -> Value {
    list_value(
        admissions
            .iter()
            .map(|admission| Value::Record {
                type_name: ctx.sym("OrdinaryCompileStructuralAdmission"),
                fields: Rc::new(sorted_fields(vec![
                    (
                        ctx.sym("compiled_module"),
                        subject_module_value(compiled_module, ctx),
                    ),
                    (
                        ctx.sym("admission"),
                        reference_derived_closure_admission_value(admission, ctx),
                    ),
                ])),
            })
            .collect::<Vec<_>>(),
    )
}

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

/// Projects a host multi-module fixture outcome into the `tools.multi_module_compile_fixture`
/// coproduct. The three arms stay distinct to the substrate: a broken harness must never wear
/// the compiler's verdict, and a compile that never ran must never arrive as
/// `FixtureCompileCompleted` with an empty diagnostic list (DESIGN §5 — could-not-measure
/// conflated with passing).
fn multi_module_compile_fixture_value(
    outcome: crate::cli_run::MultiModuleCompileFixtureOutcome,
    ctx: &InterpContext,
) -> Value {
    let rows = |rows: Vec<crate::cli_run::CompileDiagnosticCensusRow>| {
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
        )
    };
    let variant = |name: &str, fields: Vec<(Symbol, Value)>| Value::Variant {
        type_name: ctx.sym("MultiModuleCompileFixtureOutcome"),
        variant_name: ctx.sym(name),
        fields: Rc::new(sorted_fields(fields)),
    };
    match outcome {
        crate::cli_run::MultiModuleCompileFixtureOutcome::InstrumentRefused { cause } => variant(
            "FixtureInstrumentRefused",
            vec![(ctx.sym("cause"), str_value(cause))],
        ),
        crate::cli_run::MultiModuleCompileFixtureOutcome::CompileRefused {
            module_count,
            diagnostics,
            source_digest,
            compiler_digest,
        } => variant(
            "FixtureCompileRefused",
            vec![
                (ctx.sym("module_count"), Value::Int(module_count)),
                (ctx.sym("diagnostics"), rows(diagnostics)),
                (ctx.sym("source_digest"), str_value(source_digest)),
                (ctx.sym("compiler_digest"), str_value(compiler_digest)),
            ],
        ),
        crate::cli_run::MultiModuleCompileFixtureOutcome::CompileCompleted {
            module_count,
            emitted_files,
            diagnostics,
            source_digest,
            compiler_digest,
        } => variant(
            "FixtureCompileCompleted",
            vec![
                (ctx.sym("module_count"), Value::Int(module_count)),
                (
                    ctx.sym("emitted_files"),
                    list_value(emitted_files.into_iter().map(str_value).collect::<Vec<_>>()),
                ),
                (ctx.sym("diagnostics"), rows(diagnostics)),
                (ctx.sym("source_digest"), str_value(source_digest)),
                (ctx.sym("compiler_digest"), str_value(compiler_digest)),
            ],
        ),
    }
}

/// Projects a host gate receipt into the `gunbc.ci_gate` `GateReceipt` coproduct. Arms stay
/// distinct to the substrate as `compile_diagnostic_census_value`'s do: `GateNotRun` must
/// never arrive as a clean verdict — could-not-measure and passing are different facts with
/// different owners, and a `Bool` at this seam made them one value.
fn gate_receipt_value(receipt: crate::cli_run::GateReceipt, ctx: &InterpContext) -> Value {
    let observed = |outcome: Value| Value::Variant {
        type_name: ctx.sym("GateReceipt"),
        variant_name: ctx.sym("GateObserved"),
        fields: Rc::new(sorted_fields(vec![(ctx.sym("outcome"), outcome)])),
    };
    match receipt {
        crate::cli_run::GateReceipt::Clean => observed(Value::Variant {
            type_name: ctx.sym("GateOutcome"),
            variant_name: ctx.sym("GateClean"),
            fields: Rc::new(vec![]),
        }),
        crate::cli_run::GateReceipt::Failed { detail } => observed(Value::Variant {
            type_name: ctx.sym("GateOutcome"),
            variant_name: ctx.sym("GateFailed"),
            fields: Rc::new(sorted_fields(vec![(ctx.sym("detail"), str_value(detail))])),
        }),
        crate::cli_run::GateReceipt::NotApplicable { reason } => Value::Variant {
            type_name: ctx.sym("GateReceipt"),
            variant_name: ctx.sym("GateNotApplicable"),
            fields: Rc::new(sorted_fields(vec![(ctx.sym("reason"), str_value(reason))])),
        },
        crate::cli_run::GateReceipt::NotRun { cause } => Value::Variant {
            type_name: ctx.sym("GateReceipt"),
            variant_name: ctx.sym("GateNotRun"),
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

/// SGR foreground parameters per `SemanticColor`, mirroring the `extdeps.render.ansi`
/// authority (`ansi_mappings` in `dag/extdeps/render/ansi.dag`). Seed realization until the
/// interpreter consumes that table directly; dissolution is the checkable receipt ROADMAP §1
/// "interpreter terminal-output de-fork" (`dag/gunbc/roadmap/roadmap_authority.dag`).
pub mod sgr {
    pub const SUCCESS: &str = "38;5;34";
    pub const ERROR: &str = "38;5;196";
    pub const WARNING: &str = "38;5;208";
    pub const INFO: &str = "38;5;39";
    pub const DIM: &str = "2";
}

/// Whether the CLI should emit ANSI color, mirroring the `color` arm of
/// `extdeps.render.terminal_capability.detect_capability`: NO_COLOR and TERM=dumb force it
/// off; otherwise on for an interactive TTY or a CI log viewer (renders SGR). CI keeps color
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

/// CLI output verbosity. Seed realization of `gunbc.output_policy.Verbosity`
/// (`dag/gunbc/output_policy.dag`); precedence mirrors `resolve_verbosity` (verbose wins over
/// quiet, default Normal). Dissolves into consuming the .dag policy when the interpreter
/// self-hosts.
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

/// The output channels of `gunbc.output_policy.OutputChannel`, as a carrier so host-effect
/// trace sites can name their channel. The *decision* per channel is NOT computed here — the
/// entry binary evaluates it from the .dag authority (`channel_decision` via
/// `resolve_channel_policy`) and installs it with `set_output_policy`. Order matches the
/// policy array index below.
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
/// `gunbc.output_policy.resolve_channel_policy`. Set once at startup (before discovery threads
/// spawn), read process-wide. Idempotent: the first install wins.
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
/// The dispatch boundary takes this rather than a bare `ExpectedOutcome`, so "the site said
/// nothing" is a value the caller constructs, not a default the boundary manufactures. Before
/// 2026-07-26 the boundary used `declared_expectation.unwrap_or(ExpectSuccess)` — DESIGN §5's
/// absorbing fallback: the substitution inside the callee left no trace, so undeclared sites
/// counted zero by construction and never ranked for fixing.
///
/// `UntracedDefault` resolves to the SAME `ExpectSuccess` — not a behaviour change, and not a
/// floor-time refusal (operator guardrail, 2026-07-26: the wall is construction at the
/// boundary, not a refusal sweep redding sites nobody has looked at). The substitution is now
/// typed, located and COUNTED, so the frontier is observable and shrinks under trace evidence.
/// A site re-arms to `Declared` only once its expectation is established — never
/// speculatively, since a wrong `ExpectFailure` would silence a real fault.
///
/// 🟡 dissolve-on: when the counted frontier reaches zero corpus-wide, this arm is deleted
/// and an undeclared site becomes a hard typed refusal at the boundary — absence unwritable
/// rather than counted.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExpectationDeclaration {
    Declared(ExpectedOutcome),
    UntracedDefault,
}

impl ExpectationDeclaration {
    /// Resolve to the outcome the dispatch grades against, counting the untraced case at its
    /// located call site. A tallied absorbed default is a declared interim frontier; an
    /// untallied one is the fail-open this type deletes.
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

/// Carrier for `gunbc.output_policy.ExpectedOutcome` — what the caller DECLARED an effect
/// would do. Deliberately no `ExpectAny`: an unknown expectation makes every observation
/// agree, the empty set the untyped `exit != 0` proxy assumed (DESIGN §5 — a failure arm
/// refuses, never widens).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ExpectedOutcome {
    ExpectSuccess,
    ExpectFailure,
    /// The exit code is an OBSERVATION consumed by a typed downstream verdict, not a pass/fail
    /// — a probe whose non-zero exit means "subject absent". Renders ambient regardless of
    /// code; only dispatch failure (the probe could not run) stays anomalous. Admissible ONLY
    /// where the annotated helper returns a typed observation or verdict, never unit — see
    /// `gunbc.output_policy` outcome_is_data_note.
    OutcomeIsData,
}

/// The reserved call-site argument through which a caller DECLARES what it expects
/// an effect to do — the sibling edge on the CALL node (operator decision
/// 2026-07-25), resolving the open question this file carried at `dispatch_shell`.
///
/// Stripped from the argument list before `build_service_param_env`, so it never becomes a
/// bound param and never reaches `content_hash_service_inputs` (which iterates
/// `op_node.params` against `param_env`). The exclusion is structural, not a remembered
/// filter — why a service-op `input {}` was the wrong home: that IS a param, would join the
/// digest, and two invocations differing only in expectation would get different cache
/// identities for one request.
///
/// Equally NOT a transport property, despite `transport_stdin` / `transport_response_format`
/// as local precedent: `dispatch_service_wet` takes the transport from
/// `op_node.transport.or(service_node.transport)`, the extdeps DECLARATION shared by every
/// caller. There it would be per-operation, so a red control and a genuine check both calling
/// `shell.Test.IsFile` could not differ, and caller policy in extdeps is the DESIGN §3 layer
/// inversion — typechecking and reading green while wrong in both directions.
pub const EFFECT_EXPECTATION_ARG: &str = "expect";

/// Read the caller's declared expectation off the reserved argument's value.
///
/// Absent is handled by the caller as `ExpectSuccess` — the DECLARED migration default,
/// behaviour-identical to the untyped `exit != 0` proxy it replaces. A PRESENT but unreadable
/// value REFUSES: `gunbc.output_policy` has no `ExpectAny` arm, so an unreadable value is
/// ignorance, and a default would be ⊤-as-answer conflated with ⊤-as-ignorance (DESIGN §5 —
/// a failure arm refuses, never widens).
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

/// Carrier for `gunbc.output_policy.StreamDisposition` — what becomes of an effect's CAPTURED
/// SUBJECT STREAMS. Distinct from `OutputDecision`, which grades a trace line this repo
/// authored; the `.dag` authority's note says why it is not a second spelling.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StreamDisposition {
    SurfaceContent,
    SummarizeCounts,
    StreamSuppressed,
}

/// The `.dag` authority's four corners of `effect_stream_disposition` at the ShellTrace
/// channel and the run's verbosity, plus the guard literal `neutralize_workflow_commands`
/// prefixes each subject line with. The seed holds evaluated verdicts and one literal —
/// never the rule.
#[derive(Clone)]
pub struct InstalledEffectStreamPolicy {
    /// Indexed by `stream_policy_index(expected, observed_success)`.
    pub dispositions: [StreamDisposition; 6],
    pub subject_line_guard: String,
}

static EFFECT_STREAM_POLICY: std::sync::OnceLock<InstalledEffectStreamPolicy> =
    std::sync::OnceLock::new();

/// Uninstalled default, as DATA rather than a re-derivation of the `.dag` rule: the four
/// corners at `Normal` verbosity. Behaviour-preserving — at the migration default
/// `ExpectSuccess` it says "content on non-zero exit, counts on zero", as before the
/// expectation axis. `effect_stream_policy_mirror_matches_dag_authority` pins it to the golden
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

/// Transport of `extdeps.github.log_annotations.neutralize_workflow_commands`: prefix every
/// relayed subject line with the guard so it cannot occupy the line-initial `::` position
/// GitHub reads as a workflow command. Unconditional — no target probe to be wrong, and the
/// guard reads fine on a plain terminal. The transformation is the `.dag` authority's; this
/// applies the literal it published.
fn neutralize_workflow_commands(text: &str) -> String {
    let guard = subject_line_guard();
    format!("{guard}{}", text.replace('\n', &format!("\n{guard}")))
}

/// Carrier for `extdeps.render.surface.GroupSyntax` — the per-target group-marker strings the
/// entry binary evaluated from `resolve_group_syntax(github_actions)`. `close_line` is `None`
/// on a plain terminal (implicit close) and `Some("::endgroup::")` under GitHub Actions. The
/// seed only TRANSPORTS these literals; syntax choice per target stays the .dag authority's.
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

/// Whether grouping should bracket host-effect output: a syntax is installed AND at least one
/// trace-bearing channel is visible. With every host-effect channel Suppressed (e.g. Quiet)
/// there is nothing to group, so callers skip the brackets and leave no empty groups.
pub fn host_trace_grouping_active() -> bool {
    GROUP_SYNTAX.get().is_some()
        && (output_decision(OutputChannel::ShellTrace) != OutputDecision::Suppressed
            || output_decision(OutputChannel::Instrumentation) != OutputDecision::Suppressed)
}

/// Tracks an open host-effect group so `group_end` is idempotent — law 4 closes the
/// group before an Anomaly shell failure, and the batch-end `group_end` must not emit
/// a second `::endgroup::`.
static GROUP_OPEN: AtomicBool = AtomicBool::new(false);

/// Open a titled group on stderr — the host-effect trace stream, so the runner folds those
/// lines under the marker. No-op without an installed syntax. Pair with `group_end`; keep the
/// bracket tight (open → run+join the effectful work → close) and defer non-trace output
/// (PASS/FAIL) until after `group_end` so it stays outside the collapsed section.
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
/// `effect_stream_disposition` alone (never silenced by ShellTrace Suppressed). Law 4:
/// `group_end` before an Anomaly so it lands OutsideGroup.
///
/// Subject is the typed service.op intent. Failed.error carries `$ <argv> (exit=N)`; empty
/// stderr still surfaces via the Failed line alone.
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

/// Whether a SurfaceContent failure should emit a Failed observation line. Pure: disposition
/// alone, never the ShellTrace channel. Empty stderr still returns true (the Failed line is
/// the sole signal). RED control for the installed-policy path: pair with
/// `effect_stream_disposition`.
fn shell_failure_surfaces(disposition: StreamDisposition) -> bool {
    disposition == StreamDisposition::SurfaceContent
}

/// Pure tail-bounding of captured stderr following a Failed observation line. `None` with no
/// stderr to surface; `Some(block)` is content only (the Failed line carries `$ argv
/// (exit=N)`). Subject lines are guarded so relayed text cannot mint workflow commands.
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

/// Host per-argument byte ceiling, mirroring the single authority
/// `extdeps.exec.exec_arg_limit.host_exec_arg_max_strlen` (Linux execve(2) MAX_ARG_STRLEN =
/// 32 * PAGE_SIZE = 131072). A longer argv (or env) string fails `execve` with E2BIG
/// ("Argument list too long"). `argv_arg_limit_test::mirror_matches_extdeps_authority` pins
/// this to the modeled value.
pub const HOST_ARG_MAX_STRLEN_BYTES: usize = 131072;

/// Pure argv-size wall: refuse (typed, located) an invocation whose largest single argv token
/// exceeds the host per-argument ceiling, rather than get an opaque `os error 7` from
/// `execve`. Per single argument as MAX_ARG_STRLEN is, not the argv total (the separate,
/// larger ARG_MAX). `None` = within limit, `Some(err)` = refuse — no truncation, no widening
/// (DESIGN §5: a failure arm refuses, never absorbs).
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
    // `expected` is the caller's DECLARED expectation from the call node's sibling edge
    // (EFFECT_EXPECTATION_ARG). Absent it is ExpectSuccess, keeping every undeclared site
    // behaviour-identical to the untyped `exit != 0` proxy this replaces.
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

    // Arg-size wall: an argv token over the host MAX_ARG_STRLEN would kill the spawn below with
    // an opaque `os error 7` (E2BIG). Refuse here, typed and located, so the deficit is
    // diagnosable and countable. Large payloads belong in stdin (extdeps.shell
    // shell.Exec.Run), not argv.
    if let Some(err) = argv_arg_limit_refusal(&argv, HOST_ARG_MAX_STRLEN_BYTES) {
        return Err(err);
    }

    // Refuse before spawn when the whole-receipt wall ceiling is already past
    // (prior subprocess spent the budget; don't start another cargo).
    if let Some(err) = ctx.wall_deadline_exceeded_error() {
        return Err(err);
    }

    let stdout_policy = bounded_shell_host_drain::default_shell_stdout_capture_policy();
    let stderr_complete_limit = match param_env.lookup(ctx.sym("stderr_capture")) {
        Some(Value::Variant {
            variant_name,
            fields: _,
            ..
        }) if ctx.resolve(*variant_name).as_str() == "Complete" => {
            let (budget, source) = crate::memory_governor::read_host_budget_bytes();
            Some(budget.ok_or_else(|| InterpError::TypeError {
                msg: format!(
                    "WitnessStderrCaptureCompleteBudgetUnreadable: Complete stderr capture requires the active GUNBC_MEMORY_BUDGET_BYTES authority ({source})"
                ),
            })? as usize)
        }
        Some(Value::Variant {
            variant_name,
            fields,
            ..
        }) if ctx.resolve(*variant_name).as_str() == "BoundedTail" => {
            let bytes = fields
                .iter()
                .find(|(name, _)| ctx.resolve(*name).as_str() == "bytes")
                .and_then(|(_, value)| match value {
                    Value::Record { fields: mfields, .. } => {
                        mfields.iter().find(|(mname, _)| ctx.resolve(*mname).as_str() == "count").and_then(
                            |(_, mv)| match mv {
                                Value::Int(n) => Some(*n),
                                _ => None,
                            },
                        )
                    }
                    _ => None,
                })
                .ok_or_else(|| InterpError::TypeError {
                    msg: "WitnessStderrCapturePolicy.BoundedTail requires a ByteSize bytes field (Measure record with an integer count)"
                        .to_string(),
                })?;
            if bytes < 0 {
                return Err(InterpError::TypeError {
                    msg: "WitnessStderrCapturePolicy.BoundedTail bytes must be non-negative"
                        .to_string(),
                });
            }
            None
        }
        Some(Value::Record { type_name, fields })
            if ctx.resolve(*type_name).as_str() == "BoundedTail" =>
        {
            let bytes = fields
                .iter()
                .find(|(name, _)| ctx.resolve(*name).as_str() == "bytes")
                .and_then(|(_, value)| match value {
                    Value::Record { fields: mfields, .. } => {
                        mfields.iter().find(|(mname, _)| ctx.resolve(*mname).as_str() == "count").and_then(
                            |(_, mv)| match mv {
                                Value::Int(n) => Some(*n),
                                _ => None,
                            },
                        )
                    }
                    _ => None,
                })
                .ok_or_else(|| InterpError::TypeError {
                    msg: "WitnessStderrCapturePolicy.BoundedTail requires a ByteSize bytes field (Measure record with an integer count)"
                        .to_string(),
                })?;
            if bytes < 0 {
                return Err(InterpError::TypeError {
                    msg: "WitnessStderrCapturePolicy.BoundedTail bytes must be non-negative"
                        .to_string(),
                });
            }
            None
        }
        Some(other) => {
            return Err(InterpError::TypeError {
                msg: format!("unknown WitnessStderrCapturePolicy value: {other}"),
            })
        }
        None => None,
    };
    let stderr_policy = match stderr_complete_limit {
        Some(max_bytes) => {
            bounded_shell_host_drain::StreamCapturePolicy::CompleteWithin { max_bytes }
        }
        None => match param_env.lookup(ctx.sym("stderr_capture")) {
            Some(Value::Variant {
                variant_name,
                fields,
                ..
            }) if ctx.resolve(*variant_name).as_str() == "BoundedTail" => {
                let bytes = fields
                    .iter()
                    .find(|(name, _)| ctx.resolve(*name).as_str() == "bytes")
                    .and_then(|(_, value)| match value {
                        Value::Record { fields: mfields, .. } => mfields
                            .iter()
                            .find(|(mname, _)| ctx.resolve(*mname).as_str() == "count")
                            .and_then(|(_, mv)| match mv {
                                Value::Int(n) if *n >= 0 => Some(*n as usize),
                                _ => None,
                            }),
                        _ => None,
                    })
                    .ok_or_else(|| InterpError::TypeError {
                        msg: "WitnessStderrCapturePolicy.BoundedTail requires a ByteSize bytes field (Measure record with a non-negative integer count); refusing rather than defaulting"
                            .to_string(),
                    })?;
                bounded_shell_host_drain::StreamCapturePolicy::DigestAndBoundedTail {
                    max_tail_bytes: bytes,
                }
            }
            Some(Value::Record { type_name, fields })
                if ctx.resolve(*type_name).as_str() == "BoundedTail" =>
            {
                let bytes = fields
                    .iter()
                    .find(|(name, _)| ctx.resolve(*name).as_str() == "bytes")
                    .and_then(|(_, value)| match value {
                        Value::Record { fields: mfields, .. } => mfields
                            .iter()
                            .find(|(mname, _)| ctx.resolve(*mname).as_str() == "count")
                            .and_then(|(_, mv)| match mv {
                                Value::Int(n) if *n >= 0 => Some(*n as usize),
                                _ => None,
                            }),
                        _ => None,
                    })
                    .ok_or_else(|| InterpError::TypeError {
                        msg: "WitnessStderrCapturePolicy.BoundedTail requires a ByteSize bytes field (Measure record with a non-negative integer count); refusing rather than defaulting"
                            .to_string(),
                    })?;
                bounded_shell_host_drain::StreamCapturePolicy::DigestAndBoundedTail {
                    max_tail_bytes: bytes,
                }
            }
            _ => bounded_shell_host_drain::default_shell_stderr_capture_policy(),
        },
    };

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
            // A stdin-write error (e.g. broken pipe) is not the failure to report: the child
            // may have exited before consuming all of stdin, ordinary POSIX pipe behavior. The
            // child's exit_code/stdout/stderr in `output` is authoritative and already flows
            // to the `exit { .. }` clause of the .dag transport declaration.
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

    if let Some(limit_bytes) = stderr_complete_limit {
        if capture.stderr.truncated {
            return Err(InterpError::ShellOutputLimitExceeded {
                stream: "stderr",
                total_bytes: capture.stderr.total_bytes,
                limit_bytes: limit_bytes as u64,
                argv0: argv[0].clone(),
            });
        }
    }
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

// ONE SPELLING, read from the authority the parser mints against
// (`v1_std_core::field_from_key_property_name`). This accepted BOTH "from_key"
// and "from" while every emitter reader compared against "from_key" alone, which
// is span-derived and therefore never matched: the two directions of one
// procedure disagreed and nothing went red. Accepting both here is what kept the
// nickname alive, so the lenient arm is gone rather than mirrored (DESIGN §3).
fn extract_from_key(field_node: &Rc<Node>, ctx: &InterpContext) -> Option<String> {
    for prop in field_node.properties.iter() {
        let prop_name = field_init_node_name_at(prop.clone(), ctx.si());
        if prop_name == crate::v1_std_core::field_from_key_property_name() {
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

/// Create-only write: the existence test and the creation are ONE syscall (O_CREAT|O_EXCL), so a
/// path that appears between an observation and this call refuses instead of being truncated.
///
/// Deliberately NOT `write_file_owner_only` with a different name. That function sets mode 0600 at
/// creation and uses `create_new` because a creation-time mode is meaningless on an existing path --
/// its O_EXCL is incidental to owner-only mode, not a contract. Owner-only MODE and create-only
/// EXISTENCE are independent facts; a caller that obtained create-only from it would silently also
/// get 0600, and would break if owner-only ever stopped needing O_EXCL. This is portable and sets no
/// mode, because setting one would be the same conflation in reverse.
/// THE TARGET IS PUBLISHED ONLY WHEN ITS CONTENT IS COMPLETE.
///
/// The first cut of this was `create_new(true).open(path)` followed by `write_all`, and review on
/// gunbc#10026 found the state that construction cannot describe: if the open SUCCEEDS and the
/// write then fails, the target has already been created and holds zero or partial bytes. The
/// helper returned an ordinary error, the transport reported `success=false, bytes_written=0`, and
/// the caller classified it as merely unwritable -- so the model said the create did not happen
/// while a new, incomplete artifact sat at the path. That collapses "nothing was created" into "a
/// truncated repository now exists", which is a worse lie than the overwrite race this operation
/// was added to close: the original race could destroy someone else's bytes, and this one
/// FABRICATES a repository nobody wrote.
///
/// Two dispositions are not modeled here, because a construction that cannot reach the bad state
/// is available (DESIGN section 4b: construction over proof, proof over validation). Content is
/// written to a sibling temporary that is itself created with O_EXCL, and the target name is
/// claimed by `hard_link`, which FAILS IF THE TARGET EXISTS -- so exclusivity is preserved by the
/// publish step rather than by the open. Every failure before the link leaves the target absent,
/// which is exactly what the operation reports.
///
/// The temporary is removed on every path. Its removal failure is deliberately NOT propagated: it
/// leaves a stray sibling and does not affect what the target is, and reporting it as a write
/// failure would say the repository was not created when it was.
// THE CREATE-NEW REALIZATION IS NOT WRITTEN HERE ANY MORE, AND THAT IS THE POINT.
//
// This function used to be a hand-written twin of the Rust source string
// v1.compiler.emit_rust emitted into every compiled program. One fact, two authorities: review
// 5089156132 on gunbc#10069 repaired the defect in THIS one while the emitted spelling still
// opened the target directly, so a failed emitted write left a partial repository the model
// reported as never created. The two were brought back into agreement by hand, and #10069's own
// annotation admitted that nothing but review held them there.
//
// The implementation now has ONE home, extdeps.filesystem.rust_realization, and both consumers
// receive its exact bytes: the emitted crate through its lib.rs root, the seed through the
// committed generated artifact this call resolves to. The regeneration and fixed-point gates
// refuse drift on that artifact, so the agreement is machine-held rather than review-held.
use crate::gunbc_file_transport_generated::gunbc_file_write_create_new as write_file_create_new;

// ------------------------------------------------------------------------------------------------
// THE PRIMITIVE'S OWN EVIDENCE. gunbc.scm.init's witnesses cover the DECISION -- which observation
// arms may write -- and cannot reach this, because the decision is pure and the race lives in the
// write. These two cells are the write's half: an existing path refuses AND KEEPS ITS BYTES, and an
// absent one is created. Without the second, the first is satisfied by a function that never writes.
// ------------------------------------------------------------------------------------------------
#[cfg(test)]
mod write_file_create_new_tests {
    // ---------------------------------------------------------------------------------------
    // PROJECTION INTEGRITY, EXECUTED RATHER THAN ASSERTED.
    //
    // This module's whole claim is that ONE authority
    // (extdeps.filesystem.rust_realization rust_file_write_create_new_fn_def) reaches two committed
    // consumers verbatim. A one-time manual comparison is a receipt about the tree that existed when
    // someone ran it; it is not enforcement, and external review required this to execute. A future
    // rustfmt release or a generator edit could reopen the difference without touching the authority
    // at all, and nothing would say so.
    //
    // THIS IS NOT THE FORBIDDEN EQUALITY WITNESS. That pattern pins two independently authored
    // implementations to each other and calls the agreement a guarantee. There is one implementation
    // here; this checks that its two PROJECTIONS carry it unchanged, which is necessary precisely
    // because one of those paths passes through an external canonicalizer this repo does not own.
    //
    // The fourth assertion is the load-bearing one: `cargo fmt --all --check` is a declared rung
    // drop as a merge gate, so nothing else in CI would notice the seed artifact drifting out of
    // rustfmt's fixed point.
    fn repo_root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(3)
            .expect("stage0 crate sits three levels under the repo root")
            .to_path_buf()
    }

    // THE BLOCK, NOT THE FUNCTION. Extraction starts at the generated CONSTANT, not at `pub fn`.
    // Starting at the function would leave a real blind spot the moment the limit became a named
    // authority: seed renders 1024, v1_rt renders a stale 512, the function bytes match, BOTH
    // projections compile, this control stays green, and the two behave differently. "It would not
    // compile without the constant" proves presence, never agreement.
    const CANONICAL_BLOCK_MARKER: &str = "pub const GUNBC_CREATE_STAGING_CANDIDATE_ATTEMPT_LIMIT";

    fn extract_create_new_definition(source: &str, what: &str) -> String {
        let start = source
            .find(CANONICAL_BLOCK_MARKER)
            .unwrap_or_else(|| panic!("{what} does not carry the create-new canonical block"));
        let rest = &source[start..];
        let end = rest
            .find("\n}\n")
            .unwrap_or_else(|| panic!("{what}'s definition is not brace-terminated"));
        rest[..end + 3].to_string()
    }

    #[test]
    fn both_projections_carry_the_one_authority_verbatim() {
        let root = repo_root();
        let seed_path = root.join("src/v1/stage0/src/gunbc_file_transport_generated.rs");
        let runtime_path = root.join("src/v1/stage0/src/v1_rt.rs");

        let seed_src = std::fs::read_to_string(&seed_path).expect("seed projection");
        let runtime_src = std::fs::read_to_string(&runtime_path).expect("runtime projection");

        let seed = extract_create_new_definition(&seed_src, "the generated seed module");
        let runtime = extract_create_new_definition(&runtime_src, "emitted v1_rt");

        // (2) exactly once in the runtime projection -- a second copy would be a second authority,
        // checked for the constant AND the function, since either alone could be duplicated.
        for marker in [CANONICAL_BLOCK_MARKER, "pub fn gunbc_file_write_create_new"] {
            assert_eq!(
                runtime_src.matches(marker).count(),
                1,
                "v1_rt must carry {marker} exactly once",
            );
        }
        // The block really must carry the limit, or the extraction range proves nothing about it.
        assert!(
            seed.starts_with(CANONICAL_BLOCK_MARKER),
            "the compared range must begin at the generated limit, not at the function",
        );
        // (3) the two extracted byte ranges are identical
        assert_eq!(
            seed, runtime,
            "the seed artifact and emitted v1_rt must carry byte-identical definitions",
        );

        // (4) the seed artifact is already at rustfmt's fixed point
        let fmt = std::process::Command::new("rustfmt")
            .args(["--edition", "2021", "--emit", "stdout", "--quiet"])
            .arg(&seed_path)
            .output()
            .expect("rustfmt must be available: this check refuses rather than skipping");
        assert!(fmt.status.success(), "rustfmt failed on the seed artifact");
        let formatted = String::from_utf8(fmt.stdout).expect("rustfmt output is utf-8");
        assert_eq!(
            extract_create_new_definition(&formatted, "rustfmt output"),
            seed,
            "the seed artifact must already be at rustfmt's fixed point, or the two projections \
             will drift the next time a formatter runs over one of them",
        );
    }

    // ---------------------------------------------------------------------------------------
    // THE SAME-PROCESS STAGING COLLISION, AND WHY THE FIRST FORMULATION OF THIS CONTROL WAS
    // NOT DISCRIMINATING.
    //
    // Review 58836 on gunbc#10069 found that the staging name was `{path}.gunbc-create-{pid}`:
    // unique per TARGET, but not per THREAD. Two threads of one process creating the same target
    // derived the SAME staging name, so the loser failed its exclusive open on the TEMPORARY and
    // reported AlreadyExists against an internal name it never asked to write -- and if the winner
    // then failed too, both calls could refuse with no target ever published.
    //
    // The obvious control -- race N threads and assert exactly one wins -- PASSES under the
    // defective construction, because there too exactly one wins and the losers report
    // AlreadyExists. Same verdict, same ErrorKind, no information. It would have been a decoration
    // (DESIGN section 4b), and #10069 already paid for one of those.
    //
    // THIS ONE IS DETERMINISTIC AND SEPARATES THE TWO CONSTRUCTIONS. It plants exactly the staging
    // file the OLD naming rule would have chosen and leaves the TARGET ABSENT. The old rule
    // collides with that leftover and refuses a create that was entirely legitimate; the sequence
    // suffix makes each attempt's staging name its own, so the create proceeds. Red on the defect,
    // green on the repair, with no timing dependence.
    // THE GENERALIZED CONTROL: AN OCCUPIED CANDIDATE UNDER THE *CURRENT* RULE.
    //
    // External review established that the previous control, which plants the PID-only name #10069
    // used, does not prove the class closed -- it only proves the literal suffix changed. Under the
    // current rule that planted file is not a candidate at all, so the test passes without ever
    // exercising an occupied candidate.
    //
    // The defect it failed to catch is real: a per-attempt sequence makes the name unique among LIVE
    // calls in one process, but this function deliberately ignores removal failure after
    // publication, so stale staging files are an admitted physical state, and a later process may
    // reuse the pid and restart its sequence at zero. A one-shot candidate that returned its own
    // AlreadyExists would reproduce the identical defect -- target absent, legitimate create refused
    // because an INTERNAL name was occupied -- at a frequency low enough to be harder to observe.
    //
    // So this control plants the FIRST candidate the current rule derives, in a freshly spawned
    // process so the sequence is known to start at zero, and requires the operation to SKIP it,
    // acquire the next, and publish the requested target -- leaving the planted file untouched,
    // because it belongs to another attempt and deleting it would turn a naming collision into data
    // loss. It goes red against a one-shot candidate, which the previous control could not do.
    #[test]
    fn an_occupied_staging_candidate_is_skipped_rather_than_refused() {
        // A FRESH PROCESS, NOT A FORK. The previous version of this control forked and asserted the
        // child's first candidate was seq 0. That was wrong, and external review caught it: fork
        // COPIES the counter's current value, and cargo runs these tests as threads of ONE process
        // where sibling tests have already advanced it. The planted name was then not a candidate at
        // all, so the control passed without ever exercising an occupied candidate -- the identical
        // vacuity this test exists to rule out. Only a new process gives the static its initializer.
        let exe = std::env::current_exe().expect("test binary");
        let helper = exact_helper_name(&exe, HELPER_LEAF);
        let out = std::process::Command::new(exe)
            .args(["--exact", &helper, "--nocapture", "--test-threads=1"])
            .env(HELPER_ENV, "1")
            .output()
            .expect("re-exec the test binary");
        let rendered = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
        // The helper is inert without the variable, so prove the child actually RAN it -- otherwise a
        // renamed test would leave this green while asserting nothing.
        assert!(
            rendered.contains("1 passed"),
            "the helper child must run exactly one test; got:\n{rendered}",
        );
        assert!(
            out.status.success(),
            "an occupied staging candidate must be skipped, the next acquired, the target \
             published, and the planted file left untouched; got:\n{rendered}",
        );
    }

    // THE BUDGET REFUSAL HAS TO BE REACHED, NOT ASSERTED. Nothing above this exercises the arm that
    // ends the candidate search, so without it the limit is a number no executed path ever meets.
    // It occupies exactly the CONFIGURED number of candidates -- read from the one authority, never
    // retyped -- so that changing the limit changes this control rather than leaving it stale.
    #[test]
    fn the_candidate_budget_refuses_rather_than_publishing_or_looping() {
        let exe = std::env::current_exe().expect("test binary");
        let helper = exact_helper_name(&exe, BUDGET_HELPER_LEAF);
        let out = std::process::Command::new(exe)
            .args(["--exact", &helper, "--nocapture", "--test-threads=1"])
            .env(BUDGET_HELPER_ENV, "1")
            .output()
            .expect("re-exec the test binary");
        let rendered = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        );
        assert!(
            rendered.contains("1 passed"),
            "the budget helper child must run exactly one test; got:\n{rendered}",
        );
        assert!(
            out.status.success(),
            "an exhausted candidate budget must refuse, leave the target absent, and leave every \
             planted candidate untouched; got:\n{rendered}",
        );
    }

    const BUDGET_HELPER_ENV: &str = "GUNBC_STAGING_CANDIDATE_BUDGET_CHILD";
    const BUDGET_HELPER_LEAF: &str = "::staging_candidate_budget_child_helper";

    #[test]
    fn staging_candidate_budget_child_helper() {
        if std::env::var(BUDGET_HELPER_ENV).is_err() {
            return;
        }
        let limit =
            crate::gunbc_file_transport_generated::GUNBC_CREATE_STAGING_CANDIDATE_ATTEMPT_LIMIT;
        let dir = std::env::temp_dir().join(format!("gunbc-budget-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("repo.json");
        let target = path.to_str().unwrap().to_string();

        // Occupy every candidate this call is permitted to try, and no more.
        let planted: Vec<String> = (0..limit)
            .map(|seq| format!("{}.gunbc-create-{}-{}", target, std::process::id(), seq))
            .collect();
        for (seq, name) in planted.iter().enumerate() {
            std::fs::write(name, format!("occupant {seq}")).expect("plant a candidate");
        }

        let refusal = super::write_file_create_new(&target, b"a fresh repository")
            .expect_err("an exhausted candidate budget must refuse");
        // NOT AlreadyExists: the target is absent, and conflating the two is the defect this
        // whole module exists to remove.
        assert_eq!(refusal.kind(), std::io::ErrorKind::Other, "{refusal}");
        let rendered = refusal.to_string();
        assert!(
            rendered.contains("StagingCandidateBudgetExhausted")
                && rendered.contains(&format!("attempted={limit}"))
                && rendered.contains(&format!("limit={limit}")),
            "the refusal must name the cause and the budget it reached: {rendered}",
        );
        assert!(
            !path.exists(),
            "no target may be published when the budget is exhausted"
        );
        for (seq, name) in planted.iter().enumerate() {
            assert_eq!(
                std::fs::read_to_string(name).expect("a planted candidate must survive"),
                format!("occupant {seq}"),
                "a refusal must not delete another attempt's file",
            );
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    // Ask the harness for a helper's own full path rather than spelling it here: a hardcoded module
    // path would go stale on a rename and leave the control running nothing at all.
    fn exact_helper_name(exe: &std::path::Path, leaf: &str) -> String {
        let listed = std::process::Command::new(exe)
            .args(["--list"])
            .output()
            .expect("list tests");
        let listing = String::from_utf8_lossy(&listed.stdout);
        let matches: Vec<&str> = listing
            .lines()
            .filter_map(|line| line.strip_suffix(": test"))
            .filter(|name| name.ends_with(leaf))
            .collect();
        assert_eq!(
            matches.len(),
            1,
            "expected exactly one helper named {leaf}, found {matches:?}",
        );
        matches[0].to_string()
    }

    const HELPER_ENV: &str = "GUNBC_OCCUPIED_STAGING_CANDIDATE_CHILD";
    const HELPER_LEAF: &str = "::occupied_staging_candidate_child_helper";

    // Runs for real ONLY in the child process the control above spawns, where this is the first and
    // only caller and the sequence therefore starts at its initializer.
    #[test]
    fn occupied_staging_candidate_child_helper() {
        if std::env::var(HELPER_ENV).is_err() {
            return;
        }
        let dir = std::env::temp_dir().join(format!("gunbc-occupied-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("repo.json");
        let target = path.to_str().unwrap().to_string();

        let planted = format!("{}.gunbc-create-{}-0", target, std::process::id());
        std::fs::write(&planted, b"a stale internal candidate").expect("plant the first candidate");

        super::write_file_create_new(&target, b"a fresh repository")
            .expect("an occupied staging candidate must be skipped, not refused");
        assert_eq!(
            std::fs::read(&path).expect("target must be published"),
            b"a fresh repository",
        );
        // The occupied candidate must survive: it is another attempt's file, and removing it would
        // convert a naming collision into data loss.
        assert_eq!(
            std::fs::read(&planted).expect("the planted candidate must survive"),
            b"a stale internal candidate",
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    // Kept as the historical mutation control for #10069's PID-only naming. It is NOT sufficient on
    // its own -- see the generalized control above -- but it still pins the specific regression.
    #[test]
    fn a_leftover_staging_file_does_not_refuse_a_legitimate_create() {
        let dir = std::env::temp_dir().join(format!("gunbc-stale-staging-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("repo.json");
        let target = path.to_str().unwrap().to_string();

        // Exactly the name the pre-repair rule derived, and nothing else.
        let collided = format!("{}.gunbc-create-{}", target, std::process::id());
        std::fs::write(&collided, b"a leftover from an earlier attempt")
            .expect("plant the leftover");

        super::write_file_create_new(&target, b"a fresh repository")
            .expect("a leftover staging file must not refuse a create whose TARGET is absent");
        assert_eq!(
            std::fs::read(&path).expect("target must exist"),
            b"a fresh repository",
        );

        let _ = std::fs::remove_file(&collided);
        std::fs::remove_dir_all(&dir).ok();
    }

    // AND THE RACE ITSELF STILL HAS TO BEHAVE. This one is not discriminating on its own -- see
    // above -- but it is the positive control for the claim the repair makes about concurrency:
    // every refusal must name a target that REALLY exists, so no caller is told AlreadyExists about
    // a target nobody published, and no staging file may survive the run.
    #[test]
    fn concurrent_same_target_creates_produce_one_winner_and_no_residue() {
        let dir = std::env::temp_dir().join(format!("gunbc-race-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("repo.json");
        let target = path.to_str().unwrap().to_string();

        let payload = vec![b'x'; 512 * 1024];
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let t = target.clone();
                let p = payload.clone();
                std::thread::spawn(move || super::write_file_create_new(&t, &p))
            })
            .collect();
        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("thread"))
            .collect();

        let winners = results.iter().filter(|r| r.is_ok()).count();
        assert_eq!(winners, 1, "exactly one creator may publish the target");
        for r in results.iter().filter(|r| r.is_err()) {
            let e = r.as_ref().unwrap_err();
            assert_eq!(e.kind(), std::io::ErrorKind::AlreadyExists);
        }
        assert!(path.exists(), "the winner's target must be published");
        assert_eq!(std::fs::read(&path).expect("target").len(), payload.len());

        let residue: Vec<_> = std::fs::read_dir(&dir)
            .expect("list")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| n.contains(".gunbc-create-"))
            .collect();
        assert!(
            residue.is_empty(),
            "staging files must not survive: {residue:?}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn create_new_refuses_a_path_that_already_exists_and_leaves_its_bytes() {
        let dir = std::env::temp_dir().join(format!("gunbc-create-new-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("repo.json");
        std::fs::write(&path, b"SOMEONE ELSE'S BYTES").expect("seed the path");

        let err = super::write_file_create_new(path.to_str().unwrap(), b"a fresh repository")
            .expect_err("a path that exists must refuse, not be truncated");
        assert_eq!(err.kind(), std::io::ErrorKind::AlreadyExists);

        // THE ASSERTION THAT CARRIES THE FINDING. A refusal that had already truncated would still
        // return an error; what makes this a fix rather than a report is that the bytes survive.
        let after = std::fs::read(&path).expect("the file is still there");
        assert_eq!(
            after, b"SOMEONE ELSE'S BYTES",
            "the existing bytes must be untouched by a refused create"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    // THE POST-OPEN FAILURE CONTROL, and it took two attempts to make it real.
    //
    // Review on gunbc#10026 found that neither existing cell observes a failure AFTER the target
    // name would have been claimed -- exactly where the old open-then-write construction left a
    // created-but-incomplete file while reporting that nothing was created.
    //
    // THE FIRST ATTEMPT WAS A DECORATION AND IS RECORDED HERE SO IT IS NOT REBUILT. It put a
    // DIRECTORY at the target and asserted the target was untouched. Measured against the old
    // construction, it PASSED: `create_new` fails at the OPEN when the name exists, so nothing was
    // ever created and the assertion was satisfied by the defect. A check that cannot go red on the
    // fault it names is worse than absent (DESIGN section 4b) because it gets cited as coverage.
    //
    // THIS ONE REACHES THE FAULT. RLIMIT_FSIZE makes `write_all` fail with EFBIG on a path that is
    // ABSENT, so the create genuinely succeeds and the write genuinely fails -- the one ordering
    // that distinguishes the two constructions. It runs in a forked child because the limit and the
    // SIGXFSZ disposition are process-wide and this binary runs tests concurrently; the parent only
    // reads the filesystem afterwards.
    #[test]
    fn a_write_failure_after_creation_leaves_no_target_behind() {
        let dir =
            std::env::temp_dir().join(format!("gunbc-create-new-efbig-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let target = dir.join("repo.json");
        let target_s = target.to_str().unwrap().to_string();

        // 64 bytes allowed, 4 KiB written: the create succeeds, the write cannot.
        let content = vec![b'x'; 4096];

        let pid = unsafe { libc::fork() };
        assert!(pid >= 0, "fork failed");
        if pid == 0 {
            unsafe {
                libc::signal(libc::SIGXFSZ, libc::SIG_IGN);
                let lim = libc::rlimit {
                    rlim_cur: 64,
                    rlim_max: 64,
                };
                libc::setrlimit(libc::RLIMIT_FSIZE, &lim);
            }
            let _ = super::write_file_create_new(&target_s, &content);
            unsafe { libc::_exit(0) };
        }
        let mut status: libc::c_int = 0;
        unsafe { libc::waitpid(pid, &mut status, 0) };

        // THE LOAD-BEARING ASSERTION. Under the old construction the target was created by the open
        // and survived the failed write as a zero-or-partial file -- a repository nobody wrote.
        // Publishing only when the content is complete means there is nothing at the target at all.
        assert!(
            !target.exists(),
            "a write that failed after creation must leave NO target behind"
        );
        let strays: Vec<String> = std::fs::read_dir(&dir)
            .expect("list")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        assert!(
            strays.is_empty(),
            "no staging temporary may survive either: {strays:?}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn create_new_writes_when_nothing_is_there() {
        let dir = std::env::temp_dir().join(format!("gunbc-create-new-ok-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("repo.json");
        super::write_file_create_new(path.to_str().unwrap(), b"a fresh repository")
            .expect("an absent path is created");
        assert_eq!(
            std::fs::read(&path).expect("written"),
            b"a fresh repository"
        );
        std::fs::remove_dir_all(&dir).ok();
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
    // transport declares its own action; absent verb keeps the content-param convention
    // (write iff a `content` param exists, else read).
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
            "write_create_new" => {
                let content = match param_env.lookup(ctx.sym("content")) {
                    Some(v) => format!("{}", v),
                    None => {
                        return Err(InterpError::TypeError {
                            msg: format!(
                                "file write_create_new operation missing `content` argument for {}",
                                path
                            ),
                        })
                    }
                };
                let byte_count = content.len() as i64;
                trace_emit(
                    OutputChannel::ShellTrace,
                    &format!("[file] write_create_new {} ({} bytes)", path, byte_count),
                );
                return match write_file_create_new(&path, content.as_bytes()) {
                    Ok(()) => Ok(FileResult {
                        success: true,
                        byte_count,
                        path,
                        error: String::new(),
                        content: String::new(),
                    }),
                    // The refusal carries the host's message verbatim and does NOT classify itself.
                    // Deciding "already existed" from the error TEXT would be a heuristic standing in
                    // for an observation; the caller learns the create did not happen and why the
                    // host said so, which is what it needs to refuse.
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
                        "file transport verb '{other}' is not a known action (delete, list, write_owner_only, write_create_new)"
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
            "write_success" | "read_success" | "delete_success" | "list_success" | "success" => {
                Value::Bool(result.success)
            }
            "bytes_written" | "bytes" | "byte_count" => Value::Int(result.byte_count),
            "path" => str_value(result.path.clone()),
            "error" => str_value(result.error.clone()),
            "content" | "entries" => str_value(result.content.clone()),
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
/// TlsPosture). `VerifyPeer` proceeds on the stock verifier; `InsecureAcceptAnyCert` is
/// emit-only (operator decision 2026-07-16), so the interpreter refuses it rather than carry
/// a cert-verification bypass into the retiring seed; an unrecognized posture also refuses.
/// Pure so each arm is execution-witnessable.
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

/// HAND-RUST GATE explicit deferral (review 46616), covering this function and the REST
/// outcome/replay bridge below it through `dispatch_rest`: bounded growth in the seed
/// interpreter, not a new Rust authority nor a second transport convention. Every DECISION is
/// modeled — outcome states `extdeps.transports.rest` `RestOutcome`, observation states
/// `RestExchangeObservation`, replay identity and 0/1/many lookup `rest_bound_invocation_eq` /
/// `rest_exchange_fixture_lookup`, resolution selected by calling `rest_exchange_resolution`
/// back into `.dag`. Seed-side is only the projection onto the operation's declared output
/// record, which needs the interpreter's `Value`/`Node` representation.
///
/// Lane: ROADMAP `v1-interpreter-quarantine` → `v1-interpreter-delete`, counted against
/// `v1-honest-frontier`.
///
/// EARLIER, NARROWER deletion condition than the lane's, which should fire first (SCOPE
/// paragraph of the `rest_outcome_note` annotation): when the `response` block becomes the single authority
/// and `output` is DERIVED from its 2xx arm, every operation carries its outcome without
/// declaring one; the opt-in disappears, `rest_outcome_output_field` deletes outright (no
/// field to detect), and the `if status >= 400` raise below it deletes in the same motion
/// (it serves only operations declaring no outcome). Checkable by execution:
/// `rest_operation_without_outcome_still_refuses` pins the opt-in's existence, so it must be
/// REPLACED (not kept green) when the seam dissolves — a `Legacy` operation with no outcome
/// field can no longer exist.
///
/// The opt-in migration seam declared by extdeps.transports.rest.RestOutcome.
///
/// An operation asks for transport observations by declaring an output field of type
/// RestOutcome. Operations without it keep the legacy raise-on-failure behavior until the
/// response table becomes the universal result authority; see the rest_outcome_note annotation. Inspect the
/// field's TYPE, not its spelling, so callers may pick a domain-appropriate name without
/// another transport convention.
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
            // primitive realizing `std.content_hash.content_hash_atom` — because (1) the model
            // types `RestAuthenticated.digest` as `Fnv1a64Structural`, and a `DefaultHasher`
            // (SipHash) hex would be a value outside that family wearing its carrier (the
            // labeling the constructor-wall note forbids); (2) an authored fixture can
            // reproduce it via `content_hash_atom(value: "<scheme>\0<secret>")`, so
            // authenticated replay identities are expressible in .dag without opaque literals.
            // Pinned by `rest_authenticated_identity_matches_dag_constructed_value` in
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

/// The runtime `Value` shape of `std.content_hash.Fnv1a64Structural` — the single mint for
/// every seed-side crossing into a `Fnv1a64Structural`-typed carrier of the REST replay model
/// (`RestBoundOperationInvocation.input_digest`, `RestAuthenticated.digest`). A bare
/// `Value::Str` at either position is the model↔realization fork: fixture matching compares
/// a record against a string and silently never matches (DESIGN §5).
fn fnv1a64_structural_value(digest: String, ctx: &InterpContext) -> Value {
    Value::Record {
        type_name: ctx.sym("Fnv1a64Structural"),
        fields: Rc::new(sorted_fields(vec![(ctx.sym("digest"), str_value(digest))])),
    }
}

/// Witness export: lets the tests crate pin `rest_auth_identity_value`'s authenticated arm
/// `==`-equal to a dag-authored `RestAuthenticated { scheme, digest: content_hash_atom(…) }`,
/// so drift on either side (mint shape, hash family, preimage layout) goes red instead of
/// silently failing every authenticated fixture match.
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
                // modelled as std.content_hash Fnv1a64Structural (the member
                // content_hash_service_inputs produces), not bare text. The realization must
                // construct the SAME shape, or fixture matching compares a record against a
                // string and silently never matches -- the model/realization fork.
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

/// Project an observation into the operation's declared output record. On a non-success
/// outcome the body-derived fields are Null: RestOutcome is the only inhabited branch and so
/// the only consumable fact. On RestOk, keep the decoded body fields and replace just the
/// outcome field.
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

    // An unresolvable endpoint REFUSES rather than defaulting to "": an empty base fails with
    // the same `RelativeUrlWithoutBase` as a garbage one, so `unwrap_or_default()` was a
    // second unlocated route for one defect. An ABSENT key is its own refusal: "declared
    // nothing" and "declared something unreadable" are different mistakes with different fixes.
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
    // default (ureq's stock rustls verifier). InsecureAcceptAnyCert is the modeled dissolution
    // of curl's `-k` for self-signed BMC endpoints, realized EMIT-ONLY (operator, 2026-07-16):
    // emitted reqwest code uses `.danger_accept_invalid_certs(true)`; the interpreter refuses
    // it rather than carry an accept-any rustls verifier into the retiring seed. So a present
    // InsecureAcceptAnyCert is a typed refusal here (redfish etc. run through emitted code) —
    // never a silent no-op sending under VerifyPeer while the row asked for insecure. An
    // unrecognized posture also refuses.
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

    // Basic auth (RFC 7617), from a `auth_basic: { username: <input>, password: <input> }`
    // transport-block property — the modeled dissolution of curl's `-u user:pass` / netrc: the
    // credential never touches argv or a temp file, and the header value is derived in one
    // place (§3 the rest_auth_value_single_authority_note annotation). Fail-closed: a non-record `auth_basic`,
    // a missing username/password field, or a non-Str credential is a typed refusal, never an
    // unauthenticated send or a stringified-debug header.
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

    // §3: a non-empty caller-supplied input token wins over the ambient env var; absent or
    // empty falls through to auth_source so dual-declare services (auth_input + auth_source)
    // get the env-var fallback. Extract the String payload explicitly — a non-Str Value must
    // NOT produce a stringified-debug Bearer header.
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

// A service-config value is EVALUATED, like the `path` template two lines below its only
// caller — one authority for "what does this config entry say". The previous reading had a
// literal fast-path plus a fallback returning `authored_name_at`, the SOURCE TEXT of the
// identifier: `endpoint: default_api_base` became the string "default_api_base", a plausible
// non-empty value and a nonsense base URL. Every `github.Pulls` caller in the corpus failed
// on it with `RelativeUrlWithoutBase` — modeled, cited, mock-covered, with production
// callers, and its live path had never once succeeded.
//
// The fallback is deleted, not repaired, because it hid the defect: "the configured literal"
// and "the name of something unresolvable" were both `Some(String)`, so the failure surfaced
// only downstream as a malformed URL instead of a located refusal at the config read.
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
            // Narrowed to Str as the deleted branch narrowed to LitStr: Display renders every
            // Value, so `format!` would turn Null into "null" and Int into digits, and the
            // non-empty check would pass both as a base URL — the original defect one layer down.
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
    Err(InterpError::HermeticHostEffectRefused {
        operation: op_name.to_string(),
        ground: HermeticEffectGround::NoMockResponse,
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

fn toolchain_probe_variant(
    ctx: &InterpContext,
    variant: &str,
    fields: Vec<(&str, Value)>,
) -> Value {
    Value::Variant {
        type_name: ctx.sym("ToolchainInterferenceProbeResult"),
        variant_name: ctx.sym(variant),
        fields: Rc::new(sorted_fields(
            fields
                .into_iter()
                .map(|(name, value)| (ctx.sym(name), value))
                .collect(),
        )),
    }
}

fn toolchain_probe_refused(ctx: &InterpContext, cause: &str, fields: Vec<(&str, Value)>) -> Value {
    let cause_value = Value::Variant {
        type_name: ctx.sym("ToolchainInterferenceProbeRefusal"),
        variant_name: ctx.sym(cause),
        fields: Rc::new(sorted_fields(
            fields
                .into_iter()
                .map(|(name, value)| (ctx.sym(name), value))
                .collect(),
        )),
    };
    toolchain_probe_variant(
        ctx,
        "ToolchainInterferenceProbeRefused",
        vec![("cause", cause_value)],
    )
}

fn eval_toolchain_home_interference_probe_builtin(ctx: &InterpContext) -> Value {
    use std::path::Path;
    use std::process::Command;
    use std::sync::{Arc, Barrier};

    fn run_arm(reader_home: &Path, legacy_home: &Path, tag: &str) -> Result<bool, String> {
        let reader_bin = reader_home.join("bin");
        let legacy_bin = legacy_home.join("bin");
        std::fs::create_dir_all(&reader_bin).map_err(|e| format!("create reader bin: {e}"))?;
        std::fs::create_dir_all(&legacy_bin).map_err(|e| format!("create legacy bin: {e}"))?;
        let probe_name = format!("gunbc-toolchain-interference-{}-{tag}", std::process::id());
        let reader_probe = reader_bin.join(&probe_name);
        let legacy_probe = legacy_bin.join(&probe_name);
        std::fs::copy("/bin/true", &reader_probe)
            .map_err(|e| format!("install safe probe: {e}"))?;

        // The hostile tool replacement makes the discriminator deterministic. Mutating the same
        // tool again during exec is not portable (Linux may return ETXTBSY), so the writer
        // mutates a neighboring bin entry inside the readers' start/finish interval — real
        // concurrent toolchain-home mutation without an executable-open race becoming fixture
        // failure.
        let start = Arc::new(Barrier::new(3));
        let finish = Arc::new(Barrier::new(3));
        let spawn_reader =
            |probe: std::path::PathBuf, start: Arc<Barrier>, finish: Arc<Barrier>| {
                std::thread::spawn(move || {
                    start.wait();
                    let result = Command::new(probe)
                        .status()
                        .map(|s| s.success())
                        .map_err(|e| e.to_string());
                    finish.wait();
                    result
                })
            };
        let a = spawn_reader(reader_probe.clone(), start.clone(), finish.clone());
        let b = spawn_reader(reader_probe.clone(), start.clone(), finish.clone());
        let writer_start = start.clone();
        let writer_finish = finish.clone();
        let writer = std::thread::spawn(move || -> Result<(), String> {
            let replace = || -> Result<(), String> {
                let replacement = legacy_probe.with_extension("replacement");
                std::fs::copy("/bin/false", &replacement)
                    .map_err(|e| format!("stage hostile probe: {e}"))?;
                std::fs::rename(&replacement, &legacy_probe)
                    .map_err(|e| format!("replace probe: {e}"))?;
                Ok(())
            };
            let initial = replace();
            writer_start.wait();
            let concurrent = initial.and_then(|_| {
                let staged = legacy_probe.with_extension("concurrent-staged");
                let installed = legacy_probe.with_extension("concurrent");
                std::fs::write(&staged, b"hostile-writer")
                    .map_err(|e| format!("stage concurrent toolchain mutation: {e}"))?;
                std::fs::rename(staged, installed)
                    .map_err(|e| format!("install concurrent toolchain mutation: {e}"))
            });
            writer_finish.wait();
            concurrent
        });
        writer.join().map_err(|_| "writer panicked".to_string())??;
        let ar = a.join().map_err(|_| "reader-a panicked".to_string())??;
        let br = b.join().map_err(|_| "reader-b panicked".to_string())??;
        let _ = std::fs::remove_file(reader_probe);
        Ok(ar && br)
    }

    let runner_temp = std::env::var("RUNNER_TEMP").unwrap_or_default();
    let observed = std::env::var("CARGO_HOME").unwrap_or_default();
    let expected = if runner_temp.is_empty() {
        String::new()
    } else {
        format!("{runner_temp}/cargo")
    };
    if observed != expected || expected.is_empty() {
        return toolchain_probe_refused(
            ctx,
            "ToolchainPrivateHomeBindingMissing",
            vec![
                ("observed", str_value(observed)),
                ("expected", str_value(expected)),
            ],
        );
    }
    let root = std::path::PathBuf::from(&runner_temp).join(format!(
        "gunbc-toolchain-interference-{}",
        std::process::id()
    ));
    let result = (|| -> Result<Value, String> {
        std::fs::create_dir_all(&root).map_err(|e| format!("create fixture root: {e}"))?;
        let shared = root.join("shared");
        if run_arm(&shared, &shared, "shared")? {
            return Ok(toolchain_probe_refused(
                ctx,
                "ToolchainSharedArmDidNotObserveInterference",
                vec![],
            ));
        }
        if !run_arm(Path::new(&observed), &root.join("legacy"), "private")? {
            return Ok(toolchain_probe_refused(
                ctx,
                "ToolchainPrivateHomeIsolationBreached",
                vec![],
            ));
        }
        Ok(toolchain_probe_variant(
            ctx,
            "ToolchainInterferenceDiscriminated",
            vec![],
        ))
    })();
    let _ = std::fs::remove_dir_all(&root);
    result.unwrap_or_else(|detail| {
        toolchain_probe_refused(
            ctx,
            "ToolchainInterferenceFixtureFailed",
            vec![("detail", str_value(detail))],
        )
    })
}

/// Host tap for `v2.compiler.emit_host.run_host_process` (kernel-D emit_host transport):
/// materialize a workspace from resolved `{path, text}` rows, run the build argvs then the run
/// argv with typed argv (no shell), return exit/stdout/stderr/build-log as data. Wet-mode
/// only — hermetic refuses instead of mocking (no fabricated receipt). The effects flip
/// (build_transport_admission.dag: "runs only on an Permit verdict"): host builds are admitted
/// by the modeled build_workspace_grant envelope, not execution mode — the verdict is path
/// containment, mode-independent, so one law holds hermetic and wet. Anything but Permit is a
/// typed refusal; the per-file escape guard below stays as the realization-side belt.
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
    environment_arg: Option<&Value>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    ctx.effect_dispatch_count
        .set(ctx.effect_dispatch_count.get().wrapping_add(1));
    require_permitted_transport(admission_arg, ctx, "emit_host_run_transport")?;
    let admitted_names: Vec<String> = {
        let val = environment_arg.ok_or_else(|| InterpError::TypeError {
            msg: format!("emit_host_run_transport requires an `environment` argument: the target's HostTransportDescriptor.build_environment.ambient_names (v2.std.host_transport HostBuildEnvironment)"),
        })?;
        let items = free_monoid_to_vec(val).ok_or_else(|| InterpError::TypeError {
            msg: format!("emit_host_run_transport: environment must be a List<String> of admitted variable names"),
        })?;
        items
            .iter()
            .map(|v| {
                free_monoid_to_string(v).ok_or_else(|| InterpError::TypeError {
                    msg: format!("emit_host_run_transport: environment entries must be Strings"),
                })
            })
            .collect::<InterpResult<Vec<String>>>()?
    };
    let build_environment = emit_host_constructed_build_environment(&admitted_names);

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
        &build_environment,
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
/// content-addressed emit_host transport persists the workspace under workspace_dir and skips
/// build when `.native_ready` is present. workspace_dir carries the caller's computation and
/// input-realization segments; this boundary derives the resolved build-context identity and
/// appends it before consulting the marker (the effective path modeled by
/// extdeps.realization.emit_on_demand_host.native_cache_resolved_build_context_workspace_root).
/// A different closure, materialized input, build argv, resolved compiler, admitted
/// subprocess environment, or Cargo configuration MUST land in a different workspace
/// (benign-by-identity on partial writes before `.native_ready`). `.native_ready` is written
/// only after a successful run, not build alone: the P3 kernel's warm boundary is build+run
/// proof, so a transient run failure must not skip rebuild on retry. Registered in
/// 04_method.dag as emit_host_run_transport_cached; dissolve-on: witness_realization_kernel
/// emits this builtin from v2 self-hosted transport rows (same dissolution as
/// emit_host_run_transport seed handler).
/// HAND-RUST GATE explicit deferral: bounded growth in the existing seed file, not a
/// census-shrink receipt nor a new Rust authority. Lane: ROADMAP "Make native materialization
/// the shared execution kernel", witness-realization-plan (plan doc deleted 2026-08-28) P3/P6,
/// deletion row dag/gunbc/v1/v1_deletion_plan.dag ^witness_realization_kernel. Delete these
/// observation/apply helpers when the self-emitted transport consumes the modeled
/// ResolvedBuildContext and the dispatcher-change, environment-change, and cold/warm
/// agreement witnesses remain green without them.
/// Durable re-root (realization-side config, GUNBC_RESOLVED_GRAPH_CACHE_DIR precedent): the
/// root is WHERE the cache lives, never WHAT identifies an artifact — the content-hash path
/// component stays the key. Opt-in; only the declared /tmp/gunbc_ scratch prefix
/// (std.emit_on_demand root authority) is rebased, so an arbitrary caller path never silently
/// moves. SINGLE authority for every host op on the native-cache namespace: the cached run
/// transport AND emit_host_native_cache_evict share this mapping, so eviction targets the
/// workspace the transport warms (a fork here silently un-evicts).
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

/// Evict one native-cache workspace (the witness content-change/cold legs' evictor). Beside
/// the cached transport so both sides of the lifecycle read the SAME rebase mapping; a
/// shell.Remove on the .dag-composed /tmp path would miss a rebased workspace and leave it
/// falsely warm. Wet-only like the transport's other host effects; an absent workspace is a
/// no-op success (idempotent).
fn eval_emit_host_native_cache_evict_builtin(
    workspace_dir_arg: Option<&Value>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    ctx.effect_dispatch_count
        .set(ctx.effect_dispatch_count.get().wrapping_add(1));
    if ctx.execution_mode.is_hermetic() {
        return Err(InterpError::HermeticHostEffectRefused {
            operation: "emit_host_native_cache_evict".to_string(),
            ground: HermeticEffectGround::FilesystemRemoval,
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
    environment_arg: Option<&Value>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    ctx.effect_dispatch_count
        .set(ctx.effect_dispatch_count.get().wrapping_add(1));
    require_permitted_transport(admission_arg, ctx, "emit_host_run_transport_cached")?;
    let admitted_names: Vec<String> = {
        let val = environment_arg.ok_or_else(|| InterpError::TypeError {
            msg: format!("emit_host_run_transport_cached requires an `environment` argument: the target's HostTransportDescriptor.build_environment.ambient_names (v2.std.host_transport HostBuildEnvironment)"),
        })?;
        let items = free_monoid_to_vec(val).ok_or_else(|| InterpError::TypeError {
            msg: format!("emit_host_run_transport_cached: environment must be a List<String> of admitted variable names"),
        })?;
        items
            .iter()
            .map(|v| {
                free_monoid_to_string(v).ok_or_else(|| InterpError::TypeError {
                    msg: format!(
                        "emit_host_run_transport_cached: environment entries must be Strings"
                    ),
                })
            })
            .collect::<InterpResult<Vec<String>>>()?
    };

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
        &admitted_names,
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
    admitted_names: &[String],
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
        &admitted_names,
    )
}

fn run_cached_process_spec(
    ctx: &InterpContext,
    workspace_dir: String,
    workspace_files: &[(String, String)],
    build_argvs: &[Vec<String>],
    run_argv: &[String],
    require_transition_timing: bool,
    admitted_names: &[String],
) -> InterpResult<Value> {
    let workspace_dir = native_cache_rebase_workspace_dir(workspace_dir);
    let realization_workspace = std::path::PathBuf::from(&workspace_dir);
    std::fs::create_dir_all(&realization_workspace).map_err(|e| InterpError::TypeError {
        msg: format!("emit_host_run_transport_cached: workspace create failed: {e}"),
    })?;
    emit_host_materialize_workspace_files(&realization_workspace, &workspace_files)?;
    let build_environment = emit_host_constructed_build_environment(admitted_names);
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

/// Host-tool program resolution for the emit-host transports (fleet incident 2026-07-22: srv2
/// runner env has no `cargo` on PATH — repo-checkout build steps get it via the CI prelude,
/// but the transport spawns from an emitted workspace with only the process env). Order: bare
/// name if it resolves on PATH; else $CARGO_HOME/bin/<name>; else $HOME/.cargo/bin/<name>;
/// else refuse (DESIGN §5: never return the bare name and widen to ambient PATH at spawn —
/// the absorbing fallback hermetic-tool-provisioning-design (deleted) §1 names).
///
/// HAND-RUST GATE explicit deferral (review 44883): seed retained, not a new resolver
/// authority. Lane: ROADMAP `toolchain-single-resolver` (gunbc.roadmap_authority,
/// hermetic-tool-provisioning-design (plan doc deleted 2026-08-28) P2 — "one resolver",
/// handback: delete `resolve_host_tool_program` and the bash ladder). This PR repairs only the
/// fail-open terminal arm; no parallel key, no census growth. Delete the whole function when
/// P2's `membership_reconcile` instantiation routes emit-host spawns and the P2 RED control
/// (unpinned tool refuses before spawn) is witnessed in `.dag`.
///
/// A name containing `/` is one of three cases:
/// - **`./<rel>`** — the `ProducedProgram` wire format from `emit_host.dag`
///   `process_program_name`; passed through because emit-host spawns set
///   `.current_dir(workspace)` and the path is workspace-relative.
/// - **Absolute path** — caller-declared executable; must exist as a file.
/// - **Other relative paths** (e.g. `target/release/foo`) — refused as
///   `HostToolRelativePathAmbiguous`: `is_file()` is process-cwd-relative but spawn uses the
///   workspace, so check and spawn would disagree.
/// Bare names are ambient divination; absolute paths are declared intent, but a nonexistent
/// path still refuses before `Command::new`.
// Both host-tool spawn sites resolve argv[0] to a concrete path and exec THAT path, so a
// spawn failure is a fact about the resolved file, not the authored spelling. Reporting only
// the spelling discards the fact discriminating the mechanism -- a rustup shim, a system
// cargo and a per-job copy fail identically under `spawn "cargo"` with different remedies --
// and the resolved path is a live local at both sites. Carrying BOTH keeps the spelling
// greppable and the next occurrence self-diagnosing.
fn host_tool_spawn_failure(
    operation: &str,
    spelling: &str,
    resolved: &str,
    err: &std::io::Error,
) -> InterpError {
    InterpError::TypeError {
        msg: format!("{operation}: spawn {spelling:?} (resolved to {resolved:?}) failed: {err}"),
    }
}

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
///
/// THE ADMITTED NAMES ARE A DECLARED ROW, NOT A RUST PREFIX TABLE. `admitted_names` is the
/// target's `HostTransportDescriptor.build_environment.ambient_names`
/// (`v2.std.host_transport HostBuildEnvironment`), passed by every transport call; a variable
/// is admitted only by exact name. Prefix admission of `RUST*` / `CARGO_*` / `*FLAGS` let the
/// required floor's seed-build policy (`RUSTFLAGS=-D warnings`) reach the EMITTED program's
/// build: four emit_host expected-red rows built clean on every flagless runner and were
/// `RunFailed` only in CI, on `-D dead-code` (gunbc#9727). A build flag is policy; the
/// emitted program's build environment is realization; a prefix must not smuggle one into
/// the other (DESIGN §3).
fn emit_host_constructed_build_environment(admitted_names: &[String]) -> EmitHostBuildEnvironment {
    use std::os::unix::ffi::OsStrExt;
    // No seed-side veto: the row is the whole policy. CARGO_TARGET_DIR is constructed at the
    // realization boundary below (the workspace's own target dir), and a wrapper variable is
    // admitted iff a row names it.
    let admitted = |name: &str| -> bool { admitted_names.iter().any(|n| n == name) };
    let mut entries: Vec<_> = std::env::vars_os()
        .filter(|(name, _)| name.to_str().map(admitted).unwrap_or(false))
        .collect();
    entries.sort_by(|(a, _), (b, _)| a.as_bytes().cmp(b.as_bytes()));
    let mut digest =
        v1_rt::atom_identity_hash("emit-host-constructed-build-environment-v2".to_string());
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

/// Observe the host tools that will realize a cached build, inside the wet host-transport
/// boundary: the `.dag` substrate owns the effective path shape, only the host can resolve
/// PATH/rustup shims and read executable bytes. Failure to resolve, read, or execute a version
/// probe refuses the cached realization; a nominal label would recreate srv2-05.
///
/// Cargo is a driver, not the compiler identity, so its observation is paired with the rustc
/// selected by the same process environment. The transport runs under the target's declared
/// build environment, where no shipped row names RUSTC_WRAPPER or RUSTC_WORKSPACE_WRAPPER,
/// so wrappers are not part of this identity.
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
        let program = resolve_host_tool_program(&argv[0])?;
        let mut command = std::process::Command::new(&program);
        command.args(&argv[1..]).current_dir(workspace);
        emit_host_apply_build_environment(&mut command, build_environment);
        command.env("CARGO_TARGET_DIR", &target_dir);
        command.output().map_err(|e| {
            host_tool_spawn_failure("emit_host_run_transport_cached", &argv[0], &program, &e)
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
    build_environment: &EmitHostBuildEnvironment,
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
        let mut command = std::process::Command::new(&program);
        // The uncached transport inherited the WHOLE ambient environment until gunbc#9727;
        // it now receives exactly the declared admitted rows, as the cached transport does.
        emit_host_apply_build_environment(&mut command, build_environment);
        command
            .args(&argv[1..])
            .current_dir(workspace)
            .env("CARGO_TARGET_DIR", &target_dir)
            .output()
            .map_err(|e| host_tool_spawn_failure("emit_host_run_transport", &argv[0], &program, &e))
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
/// Call-site locals are passed as identifiers (`$name`, `$positional`, `$ctx`) because
/// macro_rules hygiene would not resolve them otherwise: arm bodies live here, values at the
/// expansion site.
///
/// `name` is also re-bound here, not only threaded as `$name`: two arm bodies use it as an
/// inline format capture (`"{name} requires ..."`), unreachable by token substitution; binding
/// it inside THIS definition gives it the arms' hygiene context.
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
                let s = expect_value_str($positional.first().copied(), "string_length")?;
                Ok(Some(Value::Int(s.string_length())))
            },

            arm "free_call.substring" { "substring" } => {
                // `v1_rt::substring` clamps negative start/end to 0, and `RcStr::substring`
                // clamps identically, so routing through the carrier preserves this arm exactly.
                let s = expect_value_str($positional.first().copied(), "substring")?;
                let start = expect_int($positional.get(1).copied(), "substring start")?;
                let end = expect_int($positional.get(2).copied(), "substring end")?;
                Ok(Some(str_value(s.substring(start, end))))
            },

            arm "free_call.char_at" { "char_at" } => {
                // `v1_rt::char_at` clamps a negative pos to 0, and `RcStr::char_at` clamps
                // identically, so routing through the carrier preserves this arm exactly.
                let s = expect_value_str($positional.first().copied(), "char_at")?;
                let pos = expect_int($positional.get(1).copied(), "char_at pos")?;
                Ok(Some(str_value(s.char_at(pos))))
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
                Some(Value::Str(s)) => Ok(Some(Value::Int(s.string_length()))),
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

            arm "free_call.sorted_map_keys" { "sorted_map_keys" } => match $positional.first() {
                Some(Value::Map(m)) => {
                    let keys: Vec<Value> = m.keys().map(|k| k.key.clone()).collect();
                    Ok(Some(list_value((sorted_map_keys_in_emitted_order(keys, "sorted_map_keys")?))))
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
                    // Routed through the single portable reader. This arm previously carried
                    // its OWN /proc/self/status VmHWM read — a second implementation of one
                    // observation (section 3), and the copy executing for witnesses, so fixing
                    // only cli_run's would have left this Linux-only. Authority for the
                    // interface and per-implementation units: dag/extdeps/posix/rusage.dag with
                    // dag/extdeps/{linux,darwin}/rusage.dag.
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

            arm "free_call.toolchain_home_interference_probe" { "toolchain_home_interference_probe" } => {
                if !$positional.is_empty() {
                    return Err(InterpError::TypeError { msg: "toolchain_home_interference_probe takes no arguments".to_string() });
                }
                Ok(Some(eval_toolchain_home_interference_probe_builtin($ctx)))
            },

            arm "free_call.emit_host_run_transport" { "emit_host_run_transport" } => Ok(Some(eval_emit_host_run_transport_builtin(
                $positional.first().copied(),
                $positional.get(1).copied(),
                $positional.get(2).copied(),
                $positional.get(3).copied(),
                $positional.get(4).copied(),
                $ctx,
            )?)),

            arm "free_call.emit_host_run_transport_cached" { "emit_host_run_transport_cached" } => Ok(Some(eval_emit_host_run_transport_cached_builtin(
                $positional.first().copied(),
                $positional.get(1).copied(),
                $positional.get(2).copied(),
                $positional.get(3).copied(),
                $positional.get(4).copied(),
                $positional.get(5).copied(),
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
                // `v2.lens.module_graph` composed in the interpreter; moved because it measured
                // 104,943ms against 151ms for the two leaves it combines.
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

            arm "free_call.parsed_import_statements" { "parsed_import_statements" } => {
                let file = expect_str($positional.first().copied(), $name)?;
                let source = expect_str($positional.get(1).copied(), $name)?;
                let observed =
                    crate::v1_gunbc_parsed_import_statements::parsed_import_statements(
                        file, source,
                    );
                Ok(Some(parsed_import_statements_value(&observed, $ctx)))
            },

            arm "free_call.namespace_structural_observation_admissions" { "namespace_structural_observation_admissions" } => {
                let file = expect_str($positional.first().copied(), $name)?;
                let source = expect_str($positional.get(1).copied(), $name)?;
                let neighbour_name = expect_str($positional.get(2).copied(), $name)?;
                let branch_binder_name = expect_str($positional.get(3).copied(), $name)?;
                let later_name = expect_str($positional.get(4).copied(), $name)?;
                let homonym_name = expect_str($positional.get(5).copied(), $name)?;
                let producer = crate::v1_gunbc_namespace_reference_derived_closure_production_observations::namespace_structural_observation_admissions(
                    file.clone(),
                    source.clone(),
                    neighbour_name,
                    branch_binder_name,
                    later_name,
                    homonym_name,
                );
                let compiled_module = crate::v1_gunbc_namespace_reference_derived_closure_production_observations::namespace_structural_observation_compiled_module(
                    file,
                    source,
                );
                let admissions: Vec<_> = producer.iter().cloned().collect();
                Ok(Some(namespace_structural_observation_admissions_value(
                    &compiled_module,
                    &admissions,
                    $ctx,
                )))
            },

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

            arm "free_call.compile_dag_multi_module_fixture" { "compile_dag_multi_module_fixture" } => {
                let paths = expect_str_list($positional.first().copied(), $name)?;
                let contents = expect_str_list($positional.get(1).copied(), $name)?;
                let entry = expect_str($positional.get(2).copied(), $name)?;
                Ok(Some(multi_module_compile_fixture_value(
                    crate::cli_run::compile_dag_multi_module_fixture(&paths, &contents, &entry),
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
            arm "free_call.install_or_consume_floor_compile_clean_gate_receipt" { "install_or_consume_floor_compile_clean_gate_receipt" } => Ok(Some(gate_receipt_value(
                crate::cli_run::install_or_consume_floor_compile_clean_gate_receipt(),
                $ctx,
            ))),

            arm "free_call.record_generated_artifact_drift_gate_failure_detail" { "record_generated_artifact_drift_gate_failure_detail" } => {
                if let [Value::Str(detail)] = $positional.as_slice() {
                    crate::cli_run::record_generated_artifact_drift_gate_failure_detail(detail.to_string());
                }
                Ok(Some(Value::Unit))
            },

            arm "free_call.record_generated_artifact_drift_gate_clean" { "record_generated_artifact_drift_gate_clean" } => {
                crate::cli_run::record_generated_artifact_drift_gate_clean();
                Ok(Some(Value::Unit))
            },

            arm "free_call.consume_generated_artifact_drift_gate_receipt" { "consume_generated_artifact_drift_gate_receipt" } => Ok(Some(gate_receipt_value(
                crate::cli_run::consume_generated_artifact_drift_gate_receipt(),
                $ctx,
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

/// Per-call-site attribution for the `free_monoid_to_vec` O(n) materialization cost, keyed by
/// the immediate caller's `file:line` (`#[track_caller]`). Residual-hunt instrumentation for
/// adhoc-c328b166-bca's follow-on (datetime.dag still DNF after the three parse-stage fixes)
/// -- MEASURE FIRST before any cut.
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

/// adhoc-c328b166-bca follow-on: `flatten_by_site_snapshot` attributes big materializations
/// to the interpreter-internal call site, always `eval_fold_list_native` for the residual
/// whale -- useless granularity. This keys the same signal by the fold closure's .dag source
/// span, so the dump names the v2-level fold owning the cost.
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

/// adhoc-c328b166-bca follow-on: inclusive wall-time per native builtin (function-style and
/// method-style dispatch), to localize the residual whale in native code the fold counters
/// cannot see (the medium-fixture run showed ~10 minutes of frozen fold counters and climbing
/// RSS). Inclusive: a fold's time contains its closure applies.
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

// adhoc-c328b166-bca follow-on: self-time profile per .dag function. The builtin-time table
// showed native builtins near zero while wall-clock climbed, so the residual whale is
// tree-walk residency inside .dag bodies; this names them. Self-time = inclusive minus child
// call_function frames (closure applies inside a body attribute to that body).
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

// SCAFFOLD (adhoc-c328b166-bca residual hunt, nimble-otter-476): the innermost `.dag`
// function name, pushed on each `call_function` entry (RAII-popped). `fold_list` is a builtin
// dispatched WITHOUT its own `call_function` frame, so the stack top names the `.dag`
// function CONTAINING the fold_list call -- the O(n^2) re-fold caller the datetime DNF hunt
// is chasing. Gated behind `GUNBC_FLATTEN_SITE_DUMP_SECS`; no push/pop otherwise.
// dissolve-on: as the recorders above -- delete with the residual-hunt work item, not a
// permanent profiler.
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

/// Caller attribution for LARGE left-folds (`eval_fold_list_native`, the datetime driver:
/// ~5k-element lists folded thousands of times), keyed by the `.dag` function containing the
/// `fold_list` call. Tuple = (calls, total_items, max_len, sample element `type_label`) --
/// the element type answers clever-koi's deep-clone-vs-Rc-bump axis (Str => deep,
/// Variant/List => Rc-bump).
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

/// SCAFFOLD (adhoc-c328b166-bca residual hunt): the recorders below are opt-in, gated on the
/// dump's env var (`GUNBC_FLATTEN_SITE_DUMP_SECS`), read once via OnceLock so the unset
/// production path pays one relaxed load per call, not a mutex lock or HashMap/HashSet write.
/// dissolve-on: the residual-hunt work item closes (adhoc-c328b166-bca) -- delete these
/// recorders and their call sites, not a permanent profiler.
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

/// Hypothesis-B instrumentation (adhoc-c328b166-bca residual hunt): every `Cons { head, tail
/// }` match against a native `Value::List` clones the receiver and `split_off(1)`s it for
/// `tail`. `im::Vector` makes this O(log n) once tree-ified, not the O(n) `free_monoid_to_vec`
/// disease -- but `list_tail`'s call volume across a memoized parse (one per position, via
/// `parse_current_position`) could still sum superlinear. `calls` and `receiver_len_sum`
/// answer that by execution instead of reading `im`'s source.
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

/// Hypothesis-A instrumentation (adhoc-c328b166-bca residual hunt): call frequency for the
/// grammar-analysis entry points S1's brief named as candidates for a fixed,
/// file-size-independent per-parse-module recompute (`grammar_validate_for_parse`,
/// `compute_nullable_set`, `compute_production_first_rows`). A tiny named watchlist, not a
/// general profiler, answering "how many times, relative to file size" by execution.
static CALL_FREQUENCY_WATCHLIST: std::sync::Mutex<
    Option<std::collections::HashMap<&'static str, u64>>,
> = std::sync::Mutex::new(None);

/// adhoc-c328b166-bca memo-effectiveness discriminator: distinct (grammar_digest,
/// token_stream_digest, position, production) keys ever looked up vs total lookups/hits.
/// `lookups >> distinct` with `hits == 0` is the smoking gun for "memo never serves a
/// re-attempted span"; `lookups == distinct` is the benign "every position visited once"
/// signature. Global (not per-InterpContext) so the periodic dump thread
/// (GUNBC_FLATTEN_SITE_DUMP_SECS), which never enters with_active_context, can read it --
/// survives a DNF, unlike ctx-scoped stats.
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

pub const EXPR_VARIANT_COUNT: usize = 23;

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
        ExprData::ExprElaboratedLiteral { .. } => 22,
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
        "ExprElaboratedLiteral",
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
    /// DIAGNOSTIC (2026-08-10 wedge RCA, behind GUNBC_INTERP_PROFILE=1 only). `ExprCast`
    /// measured 72.9% of daily-page render self-time at ~38.5us/cast; suspected shape: a cast
    /// resolves its target by SCANNING every item of every module, extracting source text per
    /// item, once per alias-chain hop. These three counters make the multiplier observable:
    /// kernel-walk calls, lookups it drives, items those lookups touch.
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

pub fn eval_profile_enabled() -> bool {
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
    /// Thread-CPU duration of the witness — the ENFORCEMENT basis the fast-lane cap compares
    /// against, in both the cooperative stride-poll (`EvalBudgetExceeded`) and the
    /// completion-side backstop (`BudgetKind::Cpu`).
    ///
    /// Beside `wall_nanos`, not replacing it: two clocks on one occurrence, neither
    /// substitutes. Before this field the enforced quantity was computed, spent on the budget
    /// decision, and dropped — no artifact recorded the number the cap reads, so a threshold
    /// built on a cost receipt selected a different population than the cap kills.
    ///
    /// Recording both is correct; provisional is that this one names its clock only by its
    /// NAME. See `WITNESS_COST_CLOCK_BASIS_NOTE` for the ruled replacement (a basis-carrying
    /// measurement) and the dissolution trigger.
    ///
    /// Since eval is single-threaded, CPU <= wall: a wall UNDER the cap proves CPU under it, so
    /// "lands under the fast-lane budget" triggers were always decidable from wall alone. Wall
    /// cannot answer the other direction — how near the cap a row sits, or whether an over-cap
    /// wall reflects CPU at all — the ranking question the per-witness cost-envelope lane needs.
    pub cpu_nanos: u128,
    pub eval_self_nanos: u128,
    /// THE THIRD BASIS, AND THE ONLY ONE THAT IS NOT A CLOCK: evaluator steps taken by this
    /// witness, marginal of stored shared-artifact fills — the same netting rule `cpu_nanos`
    /// carries, so the two quantities describe the same window.
    ///
    /// It is here BESIDE the clocks and replaces neither. A step count says how much evaluation
    /// work the claim performed and says nothing about how long that took on this machine; the
    /// clocks say the opposite. `gunbc.rung_drop` `floor_cost_claim_qualification_unavailable` is the row
    /// this field answers to, and it is answered only in part: the field makes a deterministic
    /// work measure EXIST and be RECORDED per claim. It is not yet compared against anything,
    /// so no cost verdict rests on it and the row stays standing.
    pub eval_steps: u64,
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
SEED DISPOSITION: the two u128 fields below are seed instrumentation -- they carry their \
basis in a field name, which is exactly what the ruled model replaces. They are retained because the enforced quantity had to stop being \
dropped before the authority lands, and the seed is not where the authority belongs. \
DISSOLVE-ON: ClockBasis lands in the std.observation authority and the declaration-grain \
receipt projects cpu and wall through it; at that point these two bare fields are replaced \
by basis-carrying measurements and the clock half of this note is deleted with them. \
\
`eval_steps` IS SEED INSTRUMENTATION TOO AND ITS DISPOSITION IS NOT THAT ONE. This paragraph \
exists because the sentence above once said the two u128 fields were the ONLY reason this \
note exists, and that stopped being true the moment a third bare field landed in the same \
struct (codex review 58626, which caught it). The distinction is not cosmetic: `eval_steps` \
is a COUNT, not a duration. It has no clock basis, so ClockBasis landing neither replaces it \
nor says anything about it -- and retiring this note on the DISSOLVE-ON above would delete \
the only disposition the field has while the field itself survives, which is a trigger \
discharging more than it covers. \
\
ITS OWN DISSOLVE-ON, named at a row a reader can evaluate rather than at a judgment about \
how far v2 has got: the roadmap row `v1-zero-hand-maintained-rust`, the v1-exit lane's \
finish line whose acceptance condition is that no hand-maintained Rust remains in the seed. \
This counter counts THIS evaluator's steps, so it lives wherever that evaluator lives; when \
the seed evaluator is gone the counter goes with it, and while that row is unaccepted this \
surface is still owed. EARLIER REMOVAL IS PERMITTED AND EXPECTED -- v2 projecting the \
per-claim work measure from a modeled receipt subsumes this -- but that is not the trigger, \
because a scaffold may always dissolve early and what a trigger fixes is the point by which \
it MUST be gone. \
\
WHAT DOES NOT DISSOLVE WITH IT: `evaluator_step_work_measure_tests`. Per DESIGN 4b(4) a \
climb deletes the redundant lower-rung PRODUCTION handling and never the evidence -- the \
envelope-invariance red, its size control and the fill-netting arm stay enrolled as the \
executing proof that the measure is still invariant wherever it comes to be computed.";

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
///
/// `eval_steps` is required for the same reason and one more: a defaulted zero on a work measure
/// reads as "this claim evaluated nothing", which is a claim about the program rather than a
/// gap in the instrument.
pub fn performance_receipt_from_witness(
    subject_key: String,
    work_shape: &str,
    wall_nanos: u128,
    cpu_nanos: u128,
    eval_steps: u64,
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
        eval_steps,
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

/// O(1) length for values whose native realization tracks it, bypassing
/// `free_monoid_to_vec`'s O(n) materialization. `parse_current_position` (v2 02_parse.dag)
/// calls `length` on the full token stream every parse attempt — an O(n) clone per attempt,
/// an O(n^2) tax the Rust-emitted realization never pays. Method-call `.length()` on native
/// `Value::Str` routes through `v1_rt::string_length` so strings are not flattened into
/// per-codepoint `Value`s (LIST-CARRIER-0 / materialize OOM). Free-call
/// `length`/`string_length` already used `chars().count()` on `Str`; this closes the
/// method-call gap only.
pub(crate) fn native_len(val: &Value) -> Option<i64> {
    match val {
        Value::List(items) => Some(items.len() as i64),
        Value::Map(m) => Some(m.len() as i64),
        Value::Set(s) => Some(s.len() as i64),
        // Method-call `.length()` on a native `Value::Str` must not fall through to
        // `free_monoid_to_vec` (one `Value` per codepoint). JSON parsing alone calls
        // `.length()` O(n) times on the input buffer — O(n^2) allocations pinning
        // multi-gigabyte RSS on ~500KB inputs (srv1 materialize_codex_runtime_bundle bisect,
        // 2026-08-14).
        //
        // LIMIT: non-ASCII .length()/.count() stays O(n) per call via the chars() walk.
        // REASON: the ASCII fast path covers the dominant repeated-query case, and non-ASCII
        // strings in this corpus are constructed-then-queried-once-or-never, so a precomputed
        // codepoint count would not amortize. The ASCII-in-practice half is AN ASSUMPTION about
        // workloads, not a modeled fact — §6: "n is small here" is not time-stable.
        // NEXT-RUNG TRIGGER: a workload repeatedly length-querying the same non-ASCII string;
        // then the amortization inverts and a carried count becomes correct.
        Value::Str(s) => Some(s.string_length()),
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

// The well-known free-monoid encoding symbols. Pre-interned at context construction
// (`over_scope_indexes`) so a lookup for any of them can never miss -- see
// `free_monoid_ctx_syms` for why a miss must not be possible, only detected.
const FREE_MONOID_WELL_KNOWN_SYMS: [&str; 4] = ["Empty", "Cons", "head", "tail"];

#[track_caller]
// `free_monoid_to_vec` uses the ambient `active_ctx()` to resolve the well-known
// Cons/Empty/head/tail symbols. It is reached (via `value_hash`/`CanonKey::hash`, for a Map
// key that is itself free-monoid) from `eval_recompute_value_hash` while that holds an
// immutable `ctx.symbols.borrow()` -- so the mutable `ctx.sym()` intern would double-borrow
// and panic. The fallback takes a read-only lookup, which requires the four symbols already
// interned. That is NOT "a free-monoid value must already carry them" (no help if IT is
// about to be constructed, or lookup races an unrelated borrow) -- it is guaranteed by
// pre-interning at context construction, so a `.get()` miss is an invariant violation, not
// "not a list". Reporting it as `None` would be DESIGN §5's empty-observation narrow
// (⊥-as-ignorance conflated with ⊥-as-answer), so this panics rather than misreport a
// Cons/Empty value as not one.
fn free_monoid_ctx_syms(ctx: &InterpContext) -> Option<(Symbol, Symbol, Symbol, Symbol)> {
    if let Ok(mut symbols) = ctx.symbols.try_borrow_mut() {
        return Some((
            symbols.intern("Empty"),
            symbols.intern("Cons"),
            symbols.intern("head"),
            symbols.intern("tail"),
        ));
    }
    let symbols = ctx
        .symbols
        .try_borrow()
        .expect("free_monoid_ctx_syms: read-only fallback taken only when the mutable borrow failed, so an immutable one must succeed");
    let get = |name: &str| {
        symbols.get(name).unwrap_or_else(|| {
            panic!(
                "free_monoid_ctx_syms: '{name}' missing from interner -- \
                 FREE_MONOID_WELL_KNOWN_SYMS pre-interning invariant violated"
            )
        })
    };
    Some((get("Empty"), get("Cons"), get("head"), get("tail")))
}

pub(crate) fn free_monoid_to_vec(val: &Value) -> Option<Vec<Value>> {
    let site = std::panic::Location::caller();
    let mut out = Vec::new();
    let mut cur = val.clone();
    let monoid_syms = active_ctx().and_then(free_monoid_ctx_syms);
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

/// Fail-closed backstop for the model↔realization String straddle (DESIGN §5). At a
/// String-meeting point (a free monoid concatenated with a native `Value::Str`), grounding
/// (`free_monoid_to_string`) has already consumed every well-typed all-codepoint String, so
/// reaching the list path means the operand is *not* a pure codepoint list — and if it still
/// contains a `Char` codepoint (`Value::Int`) it is a *mixed* `[codepoint.., non-codepoint]`
/// value, the straddle grounding dissolves. Refuse LOUDLY (the prior fail-open: `Accepted`
/// carrying a wrong-type mixed list) rather than fabricate. A homogeneous `List<String>`
/// carries no codepoint and passes. Completeness insurance: any future un-grounded
/// `FreeMonoid<Char>` × `Str` meeting point surfaces here as a loud error.
fn string_realization_straddle_detail(orig: &Value, items: &[Value]) -> Option<String> {
    // A `Value::List` is a generic collection, never a straddled String (see
    // `free_monoid_to_string`); its `Int` elements are data, so an appended `Str` is a
    // legitimate heterogeneous element. Only a `Cons`-chain / `Str`-derived flattening carries
    // codepoint semantics.
    //
    // OPEN THREAD (DESIGN §6 residue, named): this `Value::List` exemption makes the wall a
    // RATCHET WITH A NAMED HOLE. The `"chars"` method (this file) materializes a string as a
    // `Value::List` of codepoint `Int`s, identical to a generic `Int` list — so a `.chars()`
    // result straddled with a native `Str` would be exempted and fail open (the original bug).
    // Undecidable at the Value level (element-identical), so honest §6 residue, the
    // `Value::Null` pattern. LATENT today: no `.dag` program evaluates the interpreter `chars`
    // method into a concat/`+` with a `Str` (the two `.chars()` rows in `languages.dag` /
    // `rust/emit.dag` are emit *templates*). DISSOLVES WHEN `.chars()` / `Char` is regrounded
    // so a codepoint-sequence is distinguishable from a generic `Int` list at the realization
    // level (the grounding root, sibling to Int↔Nat #5428).
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

/// String grounding (DESIGN §1/§2/§7, model↔realization fork): render a string-like free
/// monoid (`String = FreeMonoid<Char>`, `Char = Nat`) to its native realization. A native
/// `Value::Str` is already grounded; a modeled `Empty`/`Cons` chain or `List` is a String
/// **only** when every element is a `Char` codepoint (`Value::Int`). A `Value::Str` *element*
/// means `List<String>`, so it returns `None` — the discriminator keeping `List<String>`
/// push/concat from collapsing into one string. Lets a folded String concatenation realize
/// as one `Value::Str` instead of a mixed `[codepoint.., Str]` list failing `==` against a
/// native String oracle (the held emit-weld debt).
pub(crate) fn free_monoid_to_string(val: &Value) -> Option<String> {
    if let Value::Str(s) = val {
        return Some(s.to_string());
    }
    // A `Value::List` is a generic ordered collection (the `[1]`/`[1,2,3]` literal
    // representation), NEVER a modeled `String`; a `FreeMonoid<Char>` realizes as an
    // `Empty`/`Cons` `Value::Variant` chain. Treating a `List` as string-like would collapse
    // `List<Int>` append/`+`/concat into one string — what the `list_free_monoid_chokepoint`
    // tests forbid (`[1] + "ab"` stays length 2). Only a native `Str` or a `Cons`-chain is a
    // String candidate; representation is the only discriminator the Value level affords.
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

/// Like `expect_str`, but returns the `Value::Str` carrier itself instead of an owned
/// copy: the caller gets the `&str` view AND the carried ASCII fact, and pays no O(n)
/// `.to_string()` for a read-only use (STRING-INDEX-0).
fn expect_value_str<'a>(val: Option<&'a Value>, context: &str) -> InterpResult<&'a RcStr> {
    match val {
        Some(Value::Str(s)) => Ok(s),
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

/// Order map keys exactly as the EMITTED Rust realization orders them.
///
/// The emitted realization is `v1_rt::sorted_map_keys<K: Ord + Clone, V>` -- `map_keys(m)`
/// then `Vec::sort()`, i.e. `K`'s own `Ord`. This arm is the interpreter's side of that one
/// primitive, so the ORDER must be the same order, or the two realizations of one `.dag`
/// program disagree.
///
/// Admitted key kinds are exactly those whose interpreter carrier has a proven-identical
/// `Ord` in the emitted realization: `Value::Str(Rc<str>)` vs `String` (both
/// byte-lexicographic over UTF-8), `Value::Int(i64)` vs `i64`, `Value::Bool` vs `bool`
/// (`false < true`). Everything else REFUSES with a typed diagnostic:
///
/// * `Value::Float` -- `f64` is not `Ord`, so the emitted call does not compile; an order
///   here is one the other realization cannot express.
/// * records, variants, lists, sets, maps, null -- the emitted order comes from a
///   `derive(Ord)` this arm cannot observe (variant declaration order, field order), so any
///   order here is a guess.
/// * a heterogeneous key set -- `HashMap<K, V>` has one `K`, so no emitted ordering exists.
///
/// A fabricated order is the worst wrong: `sorted_map_keys` exists to make a fold
/// deterministic, so a silently-different permutation is a plausible, stable, WRONG artifact
/// (DESIGN.md 5 -- no fabricated plausible output).
///
/// SO `cmp_values` IS DELIBERATELY NOT REUSED HERE. It answers `Ordering::Equal` for every
/// pair it does not recognise -- mismatched kinds, records, variants, lists -- exactly the
/// silent permutation above: a never-refusing comparator produces *an* order for key sets
/// the emitted realization cannot represent, and `sort_by` with a non-total-order comparator
/// leaves those keys wherever map iteration put them, so the answer is not even stable across
/// runs. Refusing is the only honest arm. (`sort_by`'s own use of `cmp_values` is a separate
/// caller contract, untouched here.)
fn sorted_map_keys_in_emitted_order(
    keys: Vec<Value>,
    what: &str,
) -> Result<Vec<Value>, InterpError> {
    #[derive(PartialEq, Eq)]
    enum KeyKind {
        Str,
        Int,
        Bool,
    }
    fn kind_of(v: &Value) -> Option<KeyKind> {
        match v {
            Value::Str(_) => Some(KeyKind::Str),
            Value::Int(_) => Some(KeyKind::Int),
            Value::Bool(_) => Some(KeyKind::Bool),
            _ => None,
        }
    }

    let mut kind: Option<KeyKind> = None;
    for k in &keys {
        let this = kind_of(k).ok_or_else(|| InterpError::TypeError {
            msg: format!(
                "{what}: map key of type '{}' has no emitted-Rust ordering to agree with \
                 (emitted `sorted_map_keys<K: Ord>` orders by K's own Ord; only Str, Int and \
                 Bool keys are proven to order identically in both realizations)",
                k.type_label()
            ),
        })?;
        match &kind {
            None => kind = Some(this),
            Some(seen) if *seen == this => {}
            Some(_) => {
                return Err(InterpError::TypeError {
                    msg: format!(
                        "{what}: map has keys of more than one type, so there is no emitted \
                         `HashMap<K, V>` key ordering to agree with"
                    ),
                })
            }
        }
    }

    let mut keys = keys;
    keys.sort_by(|a, b| match (a, b) {
        // `str`'s Ord is byte-lexicographic, and so is `String`'s in the emitted realization.
        (Value::Str(x), Value::Str(y)) => x.as_ref().cmp(y.as_ref()),
        (Value::Int(x), Value::Int(y)) => x.cmp(y),
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        // Unreachable: the loop above refused every other kind and every mixed key set.
        _ => std::cmp::Ordering::Equal,
    });
    Ok(keys)
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
    use super::hermetic_checkout_input_disposition_under;
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
        // Composes the .dag disposition (ExpectSuccess × exit → the Normal four corners via
        // the uninstalled fallback mirroring Normal) with the failure-surfaces predicate.
        // GREEN: passing → SummarizeCounts → silent. Discriminating RED: failing →
        // SurfaceContent → Failed line fires even with empty stderr (self-describing `$ argv`).
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
        // Mirror pin for the uninstalled fallback: the six corners the .dag witness
        // `w_shell_trace_stream_policy_projects_the_four_corners` asserts at Normal verbosity,
        // and the guard literal `extdeps.github.log_annotations.subject_text_line_guard`
        // publishes. If the authority moves and this does not, both go red together.
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
    fn hermetic_checkout_input_admits_file_and_directory_under_root() {
        let dir =
            std::env::temp_dir().join(format!("hermetic-carveout-admit-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("dag/std")).unwrap();
        std::fs::write(dir.join("dag/std/x.dag"), "module x\n").unwrap();
        assert_eq!(
            hermetic_checkout_input_disposition_under(&dir, "dag/std/x.dag"),
            Ok(())
        );
        assert_eq!(
            hermetic_checkout_input_disposition_under(&dir, "dag/std"),
            Ok(())
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn hermetic_checkout_input_admits_an_absent_path_under_root() {
        // THE COMMIT DETERMINES ABSENCE: a repository lacking a file is as deterministic an
        // input as one containing it, so an absent path under the root confirms and the read
        // runs wet -- `success: false` on its own, not off a canned response.
        let dir =
            std::env::temp_dir().join(format!("hermetic-carveout-absent-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("dag/test/fixture")).unwrap();
        assert_eq!(
            hermetic_checkout_input_disposition_under(&dir, "dag/test/fixture/never_written.json"),
            Ok(()),
            "an absent file under the root is a commit-deterministic input"
        );
        // Absent INTERMEDIATE directories too: nothing in the tail exists, and nothing in the
        // tail can be a symlink to anywhere, precisely because it is not there.
        assert_eq!(
            hermetic_checkout_input_disposition_under(&dir, "dag/absent_dir/absent_file.json"),
            Ok(()),
            "an absent path whose parent is also absent is still under the root"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    // GATED ON UNIX BECAUSE AUTHORING THE RED REQUIRES A SYMLINK. `std::os::unix::fs::symlink`
    // is the only API that constructs the present-but-unresolvable state, and an unguarded
    // reference breaks test COMPILATION on non-unix targets (review 57247). The guard is on
    // the test, not the wall: the peel loop's symlink refusal in
    // `hermetic_checkout_input_disposition_under` is unconditional; only the witness is lost.
    #[cfg(unix)]
    #[test]
    fn hermetic_checkout_refuses_a_dangling_symlink_rather_than_peeling_it_as_absent() {
        // A DANGLING SYMLINK IS PRESENT AND UNRESOLVABLE, WHICH IS NOT ABSENCE. `canonicalize`
        // follows links, so a missing target returns the SAME NotFound as an empty name.
        // Peeling it would climb past a real entry, admit the path, and dispatch a host read
        // whose result CHANGES IF THE TARGET LATER APPEARS -- host state in a hermetic run.
        // BLOCKING in review 57219 against the first cut of this repair.
        let dir =
            std::env::temp_dir().join(format!("hermetic-carveout-dangling-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let outside_target = std::env::temp_dir().join(format!(
            "hermetic-carveout-absent-target-{}",
            std::process::id()
        ));
        std::os::unix::fs::symlink(&outside_target, dir.join("dangling_out")).unwrap();
        std::os::unix::fs::symlink("./never_created_inside", dir.join("dangling_in")).unwrap();

        // Control first: the discriminator must not be "any ENOENT refuses", or the
        // absent-path admission this whole change exists for would be gone.
        assert_eq!(
            hermetic_checkout_input_disposition_under(&dir, "genuinely_absent.json"),
            Ok(()),
            "a name with nothing behind it is still a commit-deterministic absence"
        );

        let out = hermetic_checkout_input_disposition_under(&dir, "dangling_out");
        assert!(
            out.as_ref()
                .err()
                .is_some_and(|e| e.contains("exists but does not resolve")),
            "a dangling symlink to an ABSENT OUTSIDE path must refuse, not be peeled as \
             absent, got {out:?}"
        );

        let inside = hermetic_checkout_input_disposition_under(&dir, "dangling_in");
        assert!(
            inside
                .as_ref()
                .err()
                .is_some_and(|e| e.contains("exists but does not resolve")),
            "a dangling symlink inside the checkout must refuse, got {inside:?}"
        );

        // And it must not be defeated by sitting mid-path rather than at the leaf.
        let through = hermetic_checkout_input_disposition_under(&dir, "dangling_out/under.json");
        assert!(
            through.as_ref().err().is_some(),
            "a path THROUGH a dangling symlink must refuse, got {through:?}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn hermetic_checkout_absent_paths_keep_the_three_dispositions_distinct() {
        // THE DISCRIMINATING RED FOR THE ABSENT-PATH REPAIR. Admitting absent-under-root is
        // correct only if the other two refusals survive AT THEIR OWN MESSAGES; if an absent
        // out-of-root or `.git` path started confirming, the repair moved the conflation rather
        // than removed it. Every assertion is on a path that DOES NOT EXIST -- the case the old
        // canonicalize-the-leaf form could not tell apart.
        let dir =
            std::env::temp_dir().join(format!("hermetic-carveout-absent3-{}", std::process::id()));
        std::fs::create_dir_all(dir.join(".git")).unwrap();
        std::fs::create_dir_all(dir.join("target")).unwrap();

        let outside = hermetic_checkout_input_disposition_under(&dir, "../absent_outside.txt");
        assert!(
            outside
                .as_ref()
                .err()
                .is_some_and(|e| e.contains("outside the checkout root")),
            "an ABSENT path that resolves outside the root must still refuse as out-of-root, \
             got {outside:?}"
        );

        let absolute = hermetic_checkout_input_disposition_under(&dir, "/etc/absent_hostname");
        assert!(
            absolute
                .as_ref()
                .err()
                .is_some_and(|e| e.contains("outside the checkout root")),
            "an ABSENT absolute path outside the root must still refuse, got {absolute:?}"
        );

        let git = hermetic_checkout_input_disposition_under(&dir, ".git/absent_HEAD");
        assert!(
            git.as_ref()
                .err()
                .is_some_and(|e| e.contains("not commit-deterministic")),
            "an ABSENT `.git` path must still refuse as non-commit-deterministic, got {git:?}"
        );

        let target = hermetic_checkout_input_disposition_under(&dir, "target/absent_receipt.txt");
        assert!(
            target
                .as_ref()
                .err()
                .is_some_and(|e| e.contains("not commit-deterministic")),
            "an ABSENT `target` path must still refuse as non-commit-deterministic, got {target:?}"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn hermetic_checkout_read_refuses_traversal_escape_and_absolute_outside() {
        let dir =
            std::env::temp_dir().join(format!("hermetic-carveout-escape-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let escape = hermetic_checkout_input_disposition_under(&dir, "../outside.txt");
        assert!(
            escape.is_err(),
            "`..` traversal must refuse, got {escape:?}"
        );
        let absolute = hermetic_checkout_input_disposition_under(&dir, "/etc/hostname");
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
        let git = hermetic_checkout_input_disposition_under(&dir, ".git/HEAD");
        assert!(
            git.err()
                .is_some_and(|e| e.contains("not commit-deterministic")),
            ".git read must refuse as non-commit-deterministic"
        );
        let target = hermetic_checkout_input_disposition_under(&dir, "target/receipt.txt");
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
        make_field_init_node, make_field_node, make_text_part_node, no_span, Cardinality,
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
            occurrence_identity: Rc::new(
                crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
            ),
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
        let span = no_span();
        let str_type = bare_type_node("String", span.clone());
        let mut field = make_field_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            from_key.to_string(),
            str_type,
            Cardinality::CardOptional,
            None,
            Rc::new(vec![].into()),
            span.clone(),
            span.clone(),
        );
        // make_field_node's from_key stub is not a LitStr; extract_from_key needs one.
        // MINTED UNDER THE AUTHORITY, not under a literal. The fixture used to
        // spell "from_key" here and passed only because extract_from_key carried
        // a lenient arm accepting that spelling alongside "from". With the arm
        // gone the literal names a property nothing reads, so the fixture would
        // be testing a field the mechanism never sees.
        let from_key_prop = make_field_init_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            crate::v1_std_core::field_from_key_property_name(),
            make_text_part_node(
                Rc::new(
                    crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
                ),
                from_key.to_string(),
                span.clone(),
            ),
            span.clone(),
            span.clone(),
        );
        Rc::make_mut(&mut field).properties = Rc::new(im_vec![from_key_prop]);
        let return_type = Rc::new(Node {
            occurrence_identity: Rc::new(
                crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
            ),
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
            occurrence_identity: Rc::new(
                crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic,
            ),
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
    /// limit extends its caller's bound. Asserting only the tightening direction would pass
    /// against that and prove nothing.
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

    /// PINS THE DEFECT THE GUARD EXISTS FOR, so `nested_budget_scope_can_only_tighten` is not
    /// decoration. The raw paired calls, directly: a nested `arm_eval_deadline` with a LOOSER
    /// limit replaces the tighter outer bound, and a nested `clear_eval_deadline` disarms it.
    /// Both fail-open, both silent. If either assertion flips, the raw calls have been fixed
    /// and the guard's tightening can be re-examined; until then callers must not use them
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

    /// The poisoning case. `gunbc serve` shares one `InterpContext` across requests and the CPU
    /// baseline is captured at arm time, so a deadline surviving its scope measures the NEXT
    /// request against a spent baseline and refuses it immediately — worse than no bound: one
    /// stuck request becomes a permanently broken process.
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
        // The KERNEL result is caller-agnostic: this is generic shell-wait machinery, and the
        // wall budget being armed only by the witness lane today is a fact about callers, not
        // the bound. The witness lane maps it into its own refusal at the claim boundary
        // (`map_budget_error_to_witness_refusal`), where its guidance text lives.
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
    use crate::v1_std_core::{make_text_part_node, no_span, shell_transport_node, Node};

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
        let span = no_span();
        shell_transport_node(
            Rc::new(crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic),
            Rc::new(im_vec![
                make_text_part_node(
                    Rc::new(
                        crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic
                    ),
                    "sh".to_string(),
                    span.clone()
                ),
                make_text_part_node(
                    Rc::new(
                        crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic
                    ),
                    "-c".to_string(),
                    span.clone()
                ),
                make_text_part_node(
                    Rc::new(
                        crate::std_occurrence_identity::NodeOccurrenceIdentity::OccurrenceSynthetic
                    ),
                    command.to_string(),
                    span.clone()
                ),
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

    // Wiring proof: `dispatch_shell` must reach `argv_arg_limit_refusal` on the evaluated argv
    // and refuse BEFORE any exec branch; without the guard arm it proceeds to `Command::spawn`
    // and surfaces an opaque spawn error instead of `ArgvExceedsHostArgMax` (RED control —
    // predicate-only tests do not exercise this path).
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

/// Interim seed witnesses for the fail-closed arms above. HAND-RUST GATE explicit deferral
/// (review 44883): not permanent — delete with `resolve_host_tool_program` when ROADMAP
/// `toolchain-single-resolver` lands (hermetic-tool-provisioning-design (deleted) P2 RED:
/// unpinned tool refuses before spawn, witnessed in `.dag`).
#[cfg(test)]
mod resolve_host_tool_program_tests {
    use super::host_tool_spawn_failure;
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

    // The root carries the process id: `temp_dir()` is the HOST's shared /tmp on a self-hosted
    // runner, so a fixed name collides with a directory another runner slot (another uid) left
    // behind, and the write refuses PermissionDenied on a test that never touched it (observed
    // on the first CI run of this crate's unit tests).
    fn isolated_probe_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "gunbc_resolve_host_tool_{label}_{}",
            std::process::id()
        ))
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

    // The discriminating property: spelling and resolved path are DIFFERENT strings and the
    // message must carry both. Asserting only "cargo" would pass against the old
    // spelling-only text, so the resolved path is asserted as a distinct substring.
    #[test]
    fn host_tool_spawn_failure_names_the_resolved_path_not_only_the_spelling() {
        let err = std::io::Error::from_raw_os_error(26); // ETXTBSY
        let refusal = host_tool_spawn_failure(
            "emit_host_run_transport",
            "cargo",
            "/home/runner/.cargo/bin/cargo",
            &err,
        );
        let InterpError::TypeError { msg } = refusal else {
            panic!("expected TypeError, got {refusal:?}");
        };
        assert!(
            msg.contains("/home/runner/.cargo/bin/cargo"),
            "message must name the file that was actually exec'd, got {msg:?}"
        );
        assert!(
            msg.contains("\"cargo\""),
            "message must keep the authored spelling greppable, got {msg:?}"
        );
        assert!(
            msg.contains("emit_host_run_transport"),
            "message must name the operation, got {msg:?}"
        );
    }

    // A resolved path that merely repeats the spelling must not be mistaken for
    // evidence: this pins that the two positions are rendered independently, so a
    // future edit collapsing them back into one value fails here.
    #[test]
    fn host_tool_spawn_failure_renders_spelling_and_resolution_independently() {
        let err = std::io::Error::from_raw_os_error(2);
        let shim = host_tool_spawn_failure("op", "cargo", "/rustup/shims/cargo", &err);
        let system = host_tool_spawn_failure("op", "cargo", "/opt/cargo/bin/cargo", &err);
        let (InterpError::TypeError { msg: shim_msg }, InterpError::TypeError { msg: system_msg }) =
            (shim, system)
        else {
            panic!("expected TypeError from both");
        };
        assert_ne!(
            shim_msg, system_msg,
            "two different exec'd files must not produce one indistinguishable message"
        );
    }
}

#[cfg(test)]
mod process_termination_tests {
    use super::process_termination_label;

    /// A child killed by a signal has NO exit code. The seed rendered `.code().unwrap_or(-1)`
    /// for both, so an OOM-killed cargo build and a process exiting -1 produced the same
    /// bytes. Discriminating control for that split: a raw wait status carrying a signal must
    /// never render as an exit.
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

/// Discriminating controls for the `RcStr` string carrier (STRING-INDEX-0).
///
/// The carrier's point is that `char_at`/`substring`/`string_length` read a PRECOMPUTED
/// ascii flag instead of testing per call, which adds exactly one way to be wrong: the byte
/// path over text where byte index is not code-point index. These tests are that RED: each
/// carrier method against the free `v1_rt` function it shadows (pre-carrier semantics) over
/// multibyte text at every index, so a wrong flag or an unconditional byte path disagrees at
/// the first non-ASCII code point.
///
/// The flag is unforgeable, not checked: `RcStr`'s field is private and `RcStr::new` its
/// only producer, so "carrier says ASCII, string is not" has no constructor (DESIGN §4b --
/// structurally impossible, so no probe can author its RED).
#[cfg(test)]
mod rc_str_carrier_tests {
    use super::*;

    const ASCII: &str = "hello world";
    // 5 bytes, 4 code points: a byte-offset implementation disagrees from index 2 on.
    const MIXED: &str = "ab\u{e9}c";

    fn carrier(s: &str) -> RcStr {
        RcStr::new(Rc::from(s))
    }

    #[test]
    fn carrier_records_ascii_ness_of_its_own_content() {
        assert!(carrier(ASCII).is_ascii());
        assert!(!carrier(MIXED).is_ascii());
        assert!(carrier("").is_ascii());
    }

    #[test]
    fn carrier_char_at_agrees_with_the_free_function_it_shadows() {
        for s in [ASCII, MIXED] {
            let c = carrier(s);
            for pos in 0..(s.len() as i64 + 2) {
                assert_eq!(
                    c.char_at(pos),
                    v1_rt::char_at(s, pos),
                    "char_at disagrees at {pos} on {s:?}"
                );
            }
        }
    }

    #[test]
    fn carrier_substring_agrees_with_the_free_function_it_shadows() {
        for s in [ASCII, MIXED] {
            let c = carrier(s);
            let n = s.len() as i64 + 2;
            for start in 0..n {
                for end in 0..n {
                    assert_eq!(
                        c.substring(start, end),
                        v1_rt::substring(s, start, end),
                        "substring disagrees at {start}..{end} on {s:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn carrier_string_length_agrees_with_the_free_function_it_shadows() {
        for s in [ASCII, MIXED, ""] {
            assert_eq!(carrier(s).string_length(), v1_rt::string_length(s));
        }
    }

    #[test]
    fn carrier_indexes_code_points_not_bytes_on_multibyte_text() {
        // The receipt behind the differential above, stated absolutely so it survives a
        // change to the free functions: MIXED is 5 bytes and 4 code points.
        let c = carrier(MIXED);
        assert_eq!(c.string_length(), 4);
        assert_eq!(c.char_at(2), "\u{e9}");
        assert_eq!(c.char_at(3), "c");
        assert_eq!(c.char_at(4), "");
        assert_eq!(c.substring(0, 3), "ab\u{e9}");
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
                !RcStr::ptr_eq(ra, rb),
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
                RcStr::ptr_eq(a, b),
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
        assert!(RcStr::ptr_eq(a, b));
        let mut lone = a.rc();
        let mut peer = b.rc();
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

#[cfg(test)]
mod sorted_map_keys_order_tests {
    use super::{sorted_map_keys_in_emitted_order, InterpError, Value};
    use crate::v1_rt;
    use im::HashMap;

    /// The discriminating key set: every pair separates Rust's byte-lexicographic `Ord` from
    /// a plausible alternative — `"B" < "a"` fails case-insensitive collation, `"Z10" < "Z9"`
    /// fails natural/numeric sorting, `"z"` before `"\u{e9}"` fails a Unicode collation filing
    /// `é` next to `e`. An arm that merely *returns something sorted* goes red; only the
    /// emitted realization's order passes.
    const KEYS: [&str; 7] = ["b", "B", "Z9", "Z10", "\u{e9}", "z", "a"];

    fn interpreted() -> std::vec::Vec<String> {
        let keys: std::vec::Vec<Value> = KEYS.iter().map(|s| Value::Str((*s).into())).collect();
        sorted_map_keys_in_emitted_order(keys, "sorted_map_keys")
            .expect("String keys are admitted")
            .into_iter()
            .map(|v| match v {
                Value::Str(s) => s.to_string(),
                other => panic!("expected Str, got {other:?}"),
            })
            .collect()
    }

    /// The oracle is the EMITTED realization itself -- `v1_rt::sorted_map_keys`, the very
    /// function `.dag` `sorted_map_keys` compiles into -- not a golden literal transcribed
    /// from a run. Comparing to a literal would only pin whatever this arm happens to do.
    #[test]
    fn interpreter_order_equals_emitted_rust_order() {
        let mut m: HashMap<String, i64> = HashMap::new();
        for (i, k) in KEYS.iter().enumerate() {
            m.insert((*k).to_string(), i as i64);
        }
        let emitted: std::vec::Vec<String> = v1_rt::sorted_map_keys(&m).into_iter().collect();
        assert_eq!(interpreted(), emitted);
    }

    /// The order the two realizations agree ON, spelled out once — RED control for the oracle
    /// above: if `v1_rt::sorted_map_keys` and this arm drifted TOGETHER (say to
    /// case-insensitive), the equality test would still pass. Byte order is a fact about
    /// UTF-8, statable independently of either implementation.
    #[test]
    fn agreed_order_is_utf8_byte_lexicographic() {
        let expected: std::vec::Vec<String> = ["B", "Z10", "Z9", "a", "b", "z", "\u{e9}"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        assert_eq!(interpreted(), expected);
    }

    /// `sorted_map_keys` makes a fold deterministic, so the output must be a function of the
    /// key SET alone. `HashMap` iteration order is unspecified in both realizations; this
    /// controls that the sort, not the incidental traversal, produces the answer.
    #[test]
    fn order_is_independent_of_insertion_order_in_both_realizations() {
        let mut forward: HashMap<String, i64> = HashMap::new();
        for (i, k) in KEYS.iter().enumerate() {
            forward.insert((*k).to_string(), i as i64);
        }
        let mut reverse: HashMap<String, i64> = HashMap::new();
        for (i, k) in KEYS.iter().rev().enumerate() {
            reverse.insert((*k).to_string(), i as i64);
        }
        let forward_keys: std::vec::Vec<String> =
            v1_rt::sorted_map_keys(&forward).into_iter().collect();
        let reverse_keys: std::vec::Vec<String> =
            v1_rt::sorted_map_keys(&reverse).into_iter().collect();
        assert_eq!(forward_keys, reverse_keys);

        let mut reversed_input: std::vec::Vec<Value> =
            KEYS.iter().map(|s| Value::Str((*s).into())).collect();
        reversed_input.reverse();
        let interpreted_reversed: std::vec::Vec<String> =
            sorted_map_keys_in_emitted_order(reversed_input, "sorted_map_keys")
                .expect("String keys are admitted")
                .into_iter()
                .map(|v| match v {
                    Value::Str(s) => s.to_string(),
                    other => panic!("expected Str, got {other:?}"),
                })
                .collect();
        assert_eq!(interpreted_reversed, interpreted());
        assert_eq!(interpreted_reversed, forward_keys);
    }

    #[test]
    fn float_keys_refuse_because_emitted_rust_has_no_ord_for_them() {
        let err = sorted_map_keys_in_emitted_order(
            vec![Value::Float(2.0), Value::Float(1.0)],
            "sorted_map_keys",
        )
        .expect_err("f64 is not Ord, so the emitted call does not compile");
        match err {
            InterpError::TypeError { msg } => assert!(msg.contains("Float"), "{msg}"),
            other => panic!("expected a typed refusal, got {other:?}"),
        }
    }

    #[test]
    fn heterogeneous_keys_refuse_because_there_is_no_single_emitted_k() {
        let err = sorted_map_keys_in_emitted_order(
            vec![Value::Int(1), Value::Str("a".into())],
            "sorted_map_keys",
        )
        .expect_err("HashMap<K, V> has exactly one K");
        match err {
            InterpError::TypeError { msg } => assert!(msg.contains("more than one type"), "{msg}"),
            other => panic!("expected a typed refusal, got {other:?}"),
        }
    }

    #[test]
    fn int_and_bool_keys_order_as_their_emitted_carriers_do() {
        let mut ints: HashMap<i64, ()> = HashMap::new();
        for k in [3i64, -1, 10, 0] {
            ints.insert(k, ());
        }
        let interpreted_ints = sorted_map_keys_in_emitted_order(
            [3i64, -1, 10, 0].iter().map(|i| Value::Int(*i)).collect(),
            "sorted_map_keys",
        )
        .expect("Int keys are admitted");
        let emitted_ints: std::vec::Vec<Value> = v1_rt::sorted_map_keys(&ints)
            .into_iter()
            .map(Value::Int)
            .collect();
        assert_eq!(
            format!("{interpreted_ints:?}"),
            format!("{emitted_ints:?}"),
            "-1 < 0 < 3 < 10 -- numeric, not the lexicographic order a string key would give"
        );

        let mut bools: HashMap<bool, ()> = HashMap::new();
        bools.insert(true, ());
        bools.insert(false, ());
        let interpreted_bools = sorted_map_keys_in_emitted_order(
            vec![Value::Bool(true), Value::Bool(false)],
            "sorted_map_keys",
        )
        .expect("Bool keys are admitted");
        let emitted_bools: std::vec::Vec<Value> = v1_rt::sorted_map_keys(&bools)
            .into_iter()
            .map(Value::Bool)
            .collect();
        assert_eq!(
            format!("{interpreted_bools:?}"),
            format!("{emitted_bools:?}")
        );
    }
}

#[cfg(test)]
mod evaluator_step_work_measure_tests {
    use super::*;
    use crate::v1_compiler_compile::SourceFile;
    use std::sync::Arc;

    /// One fixture, parameterised by the size of the list it builds, so a work CONTROL is
    /// authorable: the same shape at a different size must produce a different count. Without
    /// that control every assertion below is satisfied by a counter frozen at any constant --
    /// including zero -- and a frozen counter is exactly the failure a work measure fails
    /// toward.
    fn list_fixture(
        module: &str,
        items: usize,
    ) -> Rc<crate::v1_compiler_compile::ResolvedPipelineResult> {
        let list = (0..items)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        crate::v1_compiler_compile::compile_to_resolved(Rc::new(im::vector![Rc::new(SourceFile {
            path: format!("workspace/src/{module}.dag"),
            content: format!("module fixture.{module}\nfn total() -> List<Int> {{ [{list}] }}\n"),
        }),]))
    }

    /// Evaluate `fixture.<module>.total` once in a fresh context and return the evaluator steps
    /// it took. The delta is taken across the call, never the absolute, because the counter is
    /// per-thread and cumulative and every other test in this file also evaluates.
    fn steps_of(
        result: &crate::v1_compiler_compile::ResolvedPipelineResult,
        module: &str,
        budget: Option<u64>,
    ) -> u64 {
        let graph = result.graph.as_ref().expect("fixture graph");
        let ctx = InterpContext::new(
            graph,
            result.source_indices.clone(),
            ExecutionMode::Hermetic,
        );
        if let Some(ms) = budget {
            ctx.arm_eval_deadline(ms);
        }
        let before = evaluator_steps();
        let value = run_in_context(&ctx, &format!("fixture.{module}.total"), false)
            .expect("fixture must evaluate");
        std::hint::black_box(value);
        evaluator_steps().wrapping_sub(before)
    }

    /// THE DISCRIMINATING RED FOR THE WHOLE MEASURE. `gunbc.rung_drop`
    /// `floor_cost_claim_qualification_unavailable` says an attempt's CPU duration is not an invariant
    /// property of the witness, and names a deterministic work measure as one of the three arms
    /// that would restore a claim-owned cost basis. This is that arm's evidence, and it is
    /// asserted as EXACT EQUALITY rather than as a tolerance: a work measure that needed a
    /// tolerance across envelopes would be a slow clock, not a count.
    ///
    /// THE TWO ENVELOPES ARE REAL AND DIFFERENT, not two calls in a row. The second arm runs
    /// with the CPU deadline ARMED -- which is a different code path through `eval_expr`, taking
    /// the stride poll and the two clock reads the first arm never executes -- and with a
    /// co-tenant thread spinning beside it for the whole evaluation, which is the contention the
    /// row's subject names. Neither perturbation may move the count by one.
    #[test]
    fn the_step_count_is_identical_across_two_execution_envelopes() {
        let result = list_fixture("step_envelope", 2_000);

        let quiet = steps_of(&result, "step_envelope", None);

        let stop = Arc::new(AtomicBool::new(false));
        let co_tenant = {
            let stop = stop.clone();
            std::thread::spawn(move || {
                let mut spin = 0u64;
                while !stop.load(Ordering::Relaxed) {
                    for i in 0..10_000u64 {
                        spin = spin.wrapping_add(i);
                    }
                }
                std::hint::black_box(spin);
            })
        };
        // A budget far above anything this fixture can spend: the point is that the deadline is
        // ARMED, not that it fires. A fired deadline would end the evaluation early and the two
        // arms would be measuring different amounts of work.
        let contended = steps_of(&result, "step_envelope", Some(60_000));
        stop.store(true, Ordering::Relaxed);
        co_tenant.join().expect("co-tenant thread");

        assert!(
            quiet > 0,
            "the step counter never advanced across a 2000-element list; every equality below \
             would then be 0 == 0 and this file would assert nothing"
        );
        assert_eq!(
            quiet, contended,
            "evaluator steps must be identical across execution envelopes: quiet={quiet}, \
             deadline-armed under co-tenant load={contended}"
        );
    }

    /// THE WORK CONTROL, and the reason the equality above is readable. A counter frozen at a
    /// constant satisfies every invariance assertion in this file; only a size change can tell
    /// an invariant measure from a dead one. Asserted as strict growth rather than as a ratio,
    /// because the per-element step cost is an implementation fact of the evaluator and not
    /// something this test is entitled to pin.
    #[test]
    fn a_larger_workload_takes_strictly_more_steps() {
        let small = list_fixture("step_control_small", 100);
        let large = list_fixture("step_control_large", 1_000);
        let small_steps = steps_of(&small, "step_control_small", None);
        let large_steps = steps_of(&large, "step_control_large", None);
        assert!(
            large_steps > small_steps,
            "a tenfold larger list must take strictly more evaluator steps: \
             small={small_steps}, large={large_steps}"
        );
    }

    /// THE ORDER-INVARIANCE HALF, WHICH IS A SEPARATE CLAIM FROM THE ENVELOPE HALF. A count
    /// that included shared-artifact fills would be a function of WHICH claim ran first: the one
    /// that filled the memo would carry the producer's steps and every later reader would carry
    /// none. That is the same defect the 2026-08-27 fill-attribution ruling closed on the CPU
    /// clock, and this asserts the work measure is netted by the same rule.
    ///
    /// THE RED IS THE RAW COUNT BESIDE THE NETTED ONE. The raw counts of the two runs are
    /// asserted to differ by a wide margin -- proving the order dependence is real and large,
    /// so the netted equality is not a vacuous comparison of two identical numbers.
    #[test]
    fn stored_shared_fills_are_netted_out_of_the_step_count() {
        super::clear_cross_claim_pure_memos();
        let items = (0..20_000)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let result =
            crate::v1_compiler_compile::compile_to_resolved(Rc::new(im::vector![Rc::new(
                SourceFile {
                    path: "workspace/src/step_fill_fixture.dag".to_string(),
                    content: format!(
                        "module fixture.step_fill\nfn producer() -> List<Int> {{ [{items}] }}\n\
                         fn use_producer() -> List<Int> {{ producer() }}\n"
                    ),
                }
            ),]));
        let graph = result.graph.as_ref().expect("fixture graph");
        let fresh = || {
            InterpContext::new(
                graph,
                result.source_indices.clone(),
                ExecutionMode::Hermetic,
            )
        };
        let enrolled = fresh();
        let producer = enrolled
            .lookup_fn_node("fixture.step_fill.producer")
            .expect("fixture producer");
        super::install_cross_claim_pure_share_roster([producer]);

        // One measurement, exactly as `run_claim_measured` takes it: raw delta, fill delta,
        // and the marginal figure that is the difference.
        let measure = |ctx: &InterpContext| -> (u64, u64) {
            let steps_before = evaluator_steps();
            let fill_before = shared_artifact_fill_eval_steps();
            let value = run_in_context(ctx, "fixture.step_fill.use_producer", false)
                .expect("fixture must evaluate");
            std::hint::black_box(value);
            let raw = evaluator_steps().wrapping_sub(steps_before);
            let fill = shared_artifact_fill_eval_steps().wrapping_sub(fill_before);
            (raw, raw.saturating_sub(fill))
        };

        let filler = fresh();
        let (raw_filler, marginal_filler) = measure(&filler);
        let reader = fresh();
        let (raw_reader, marginal_reader) = measure(&reader);
        super::install_cross_claim_pure_share_roster(Vec::<Rc<Node>>::new());
        super::clear_cross_claim_pure_memos();

        assert!(
            raw_filler > raw_reader.saturating_mul(10),
            "the memo must actually make the second run's RAW work far smaller, or the netted \
             equality below compares two numbers that were never going to differ: \
             filler={raw_filler}, reader={raw_reader}"
        );
        assert_eq!(
            marginal_filler, marginal_reader,
            "the claim that PAID for a shared fill and the claim that read it warm must carry \
             the same marginal step count, or the measure is a function of execution order: \
             filler={marginal_filler}, reader={marginal_reader}"
        );
    }
}
