// v1_interpreter.rs — Tree-walking interpreter for .dag programs.
// Hand-written infrastructure (same category as parser, tokenizer, v1_rt).
// I-1: pure evaluation. I-2: shell service dispatch.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::time::Instant;

// Persistent value carriers (ctrl#1533 phase 2), implementing the
// v2.std.value_carrier declarations: HamtMap is a hash array mapped trie
// (map_carrier), RrbVector a relaxed radix balanced tree (list_carrier).
// Both are Rc-backed (im-rc), matching the thread-confined Rc-discipline of
// Value. Updates share structure with the prior version instead of copying
// it — the quadratic-allocation term ctrl#1533 phase 0 measured.
use im_rc::HashMap as HamtMap;
use im_rc::Vector as RrbVector;

use crate::std_syntax::BinOp;
use crate::std_syntax::LiteralValue;
use crate::v1_compiler_emit::{extract_string_interp_parts, has_mock_prefix};
use crate::v1_compiler_infer_items::{ItemInfo, ItemKind, ResolvedGraph, TypedModule};
use crate::v1_rt;
use crate::v1_rt::{
    rc_empty_set as empty_set, rc_set_insert as set_insert, rc_set_union as set_union, set_contains,
};
use crate::v1_std_core::{
    arg_name_at,
    arg_value,
    arm_body,
    arm_pattern,
    // Accessor functions
    authored_name_at,
    binop_left,
    binop_right,
    block_stmts,
    cast_expr,
    cast_target,
    expr_call_func_at,
    expr_field_access_summary,
    expr_method_call_semantics,
    expr_method_name_at,
    expr_var_name_at,
    field_access_base,
    field_access_field_at,
    field_binding_name_at,
    field_binding_pattern,
    field_init_node_name_at,
    field_init_node_value,
    find_property,
    find_property_string,
    foreach_body,
    foreach_collection,
    foreach_variable_at,
    if_condition,
    if_else_branch,
    if_then_branch,
    index_base,
    index_expr,
    is_file_transport,
    is_rest_transport,
    is_shell_transport,
    lambda_body,
    lambda_param_names_at,
    let_binding_name_at,
    let_body,
    let_value,
    match_arm_nodes,
    match_scrutinee,
    method_arg_nodes,
    method_receiver,
    param_node_default_value,
    param_node_name_at,
    record_lit_type_name_at,
    return_value,
    slice_base,
    slice_end,
    slice_start,
    unaryop_operand,
    CallSemantics,
    Cardinality,
    Connective,
    ErrorNode,
    ExprData,
    FieldAccessStyle,
    FieldSummary,
    FieldValueShape,
    InferredNode,
    MatchPattern,
    MethodSemantics,
    NewlineIndex,
    Node,
    SourceSpan,
    StringPart,
    UnaryOpKind,
    VarBindingKind,
};
use crate::wire_value_serialize::value_to_wire_json;

// ---------------------------------------------------------------------------
// Symbol interning (ctrl#1533 phase 3)
// ---------------------------------------------------------------------------
//
// Runtime identity carriers (type names, variant names, record field keys,
// env binding keys) are references into a single per-evaluation intern table,
// not owned heap strings. Integer Symbol ids replace string hashing on the hot
// path (v2.std.value_carrier M-D).

/// Opaque interned identifier; meaningful only within the owning `SymbolInterner`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Symbol(u32);

/// Single intern table for one evaluation context. All `Symbol` values produced
/// during an evaluation draw ids from this table.
#[derive(Debug, Default)]
pub struct SymbolInterner {
    strings: Vec<String>,
    index: HashMap<String, u32>,
    /// Total `intern` calls over this table's lifetime — the linear term.
    calls: u64,
}

/// Interning receipt (ctrl#1533 phase 3 / #4799): every `intern` call for a
/// name already in the table is a heap `String` allocation that the
/// pre-#4799 carrier WOULD have made (each occurrence of a type/variant/field
/// name was its own owned `String`) and that interning elides. `distinct` is
/// the number of allocations actually retained; `hits` (= `calls − distinct`)
/// is the count of allocations avoided — the dedup factor `calls/distinct` is
/// the before/after receipt for the carrier swap.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct InternStats {
    /// Total `intern` invocations (linear in occurrences of identity names).
    pub calls: u64,
    /// Distinct symbols retained — `String` allocations actually made.
    pub distinct: u64,
    /// Repeat lookups served from the table — allocations interning avoided.
    pub hits: u64,
    /// Estimated heap bytes the table retains (payloads + index keys).
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

    /// Snapshot the interning receipt for this table (see [`InternStats`]).
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

    /// Estimated heap bytes for the intern table (string payloads + index keys).
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
    /// Active evaluation context for `Value` formatting (ctrl#1533 phase 3).
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

/// Host-transport helper: keep `InterpContext` active while inspecting or
/// printing `Value`s whose identity carriers are interned symbols.
pub fn with_active_context<R>(ctx: &InterpContext, f: impl FnOnce() -> R) -> R {
    with_active_ctx(ctx, f)
}

fn active_ctx() -> Option<&'static InterpContext> {
    // SAFETY: borrows the pointer installed by `with_active_ctx`'s ActiveCtxGuard.
    // Sound only while that guard is on-stack; callers must read-and-drop within the
    // guard scope — do not return or store this reference past the closure.
    ACTIVE_CTX.with(|cell| cell.get().map(|ptr| unsafe { &*ptr }))
}

fn resolve_sym(sym: Symbol) -> String {
    active_ctx()
        .map(|ctx| ctx.resolve(sym).to_string())
        .unwrap_or_else(|| format!("#{}", sym.0))
}

/// Serialize a runtime `QualifiedName` value to dotted module path (no filepath).
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
                    .find(|(k, _)| resolve_sym(**k) == "head")
                    .and_then(|(_, v)| match v {
                        Value::Str(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| {
                        panic!("qualified_name_to_module_path: QnCons.head not Str")
                    });
                let tail = fields
                    .iter()
                    .find(|(k, _)| resolve_sym(**k) == "tail")
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

// ---------------------------------------------------------------------------
// CanonKey — finite-map key keyed by the single Value-equality authority
// ---------------------------------------------------------------------------
//
// A finite `Map<K, V>` is a finite set of key→value pairs (a finite functional
// relation): the `lookup` partial function is *derived* from that set, not the
// primitive. The std type `Map<K, V> { lookup: fn(K) -> Witness<V> }` declares the
// observation interface; the runtime realizes a finite map as data so both
// `lookup` (by application) AND whole-map `==` (extensional, set equality) are
// decidable.
//
// `CanonKey` wraps the original key `Value` and uses it as the map key. Equality is
// NOT a second authority: `PartialEq`/`Eq` delegate to `Value::eq` — the same
// language `==` every other consumer reads (P2 single-authority). Only `Hash` is
// derived here, and it is built to be *consistent* with `Value::eq` (equal values
// hash equally): the FreeMonoid alias (`Str` ≡ `List` ≡ `Empty`/`Cons` chain that
// flatten equally), `+0.0 == -0.0`, and the order-independence of record/variant
// `fields` (a nondeterministic `HashMap`) are all reflected in the hash, mirroring
// exactly the cases `Value::eq` unifies. `Value::eq` ignores `type_name` for
// records/variants, so the hash does too. Original keys are preserved, so
// `keys`/iteration recover real keys, not strings.
//
// A value is a valid map key iff it is reflexive under `Value::eq` (`v == v`). That
// rejects `Closure`/`Fn` (the `_ => false` arm) and `Float` NaN — and anything
// transitively containing them — using the same authority, with no parallel
// classification. Insertion fails closed (P3) on an invalid key.
#[derive(Debug, Clone)]
pub struct CanonKey {
    key: Value,
}

impl CanonKey {
    /// Build a key, or `None` if `key` has no decidable identity under `Value::eq`
    /// (not reflexive — e.g. closures, fn references, `Float` NaN, or a structure
    /// containing one). Reflexivity is checked through the single equality authority.
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
        // Single equality authority: the language `==` (Value::eq).
        self.key == other.key
    }
}

impl Eq for CanonKey {}

impl Hash for CanonKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u64(value_hash(&self.key));
    }
}

/// Structural hash of a key `Value`, consistent with `Value::eq`: every pair of
/// values that `Value::eq` considers equal hashes identically. Collisions are
/// permitted (that is what equality resolves), but eq-inconsistency is not.
pub(crate) fn value_hash_public(v: &Value) -> u64 {
    value_hash(v)
}

fn value_hash(v: &Value) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut h = DefaultHasher::new();

    // FreeMonoid alias: a List, a String, or a well-formed `Empty`/`Cons` chain that
    // flatten to the same element sequence are `==` (see Value::eq), so all hash
    // through the flattened sequence. `Str` flattens to its codepoints (Char = Nat),
    // matching a List/chain of the same Int codepoints — mirroring `char_value`.
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
            // Value::eq compares with `==`, so +0.0 and -0.0 are equal: normalize the
            // zero bit-pattern. (NaN is not reflexive and is rejected as a key.)
            let bits = if *f == 0.0 { 0u64 } else { f.to_bits() };
            bits.hash(&mut h);
        }
        Value::Set(members) => {
            5u8.hash(&mut h);
            // BTreeSet iterates in sorted order — already stable.
            members.len().hash(&mut h);
            for m in members.iter() {
                m.hash(&mut h);
            }
        }
        // Value::eq ignores `type_name` for records/variants and compares `fields` as
        // (order-independent) HashMaps, so hash the fields commutatively and omit
        // `type_name`.
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
            // Order-independent over entries.
            let mut acc: u64 = 0;
            for (k, val) in m.iter() {
                let mut eh = DefaultHasher::new();
                value_hash(&k.key).hash(&mut eh);
                value_hash(val).hash(&mut eh);
                acc = acc.wrapping_add(eh.finish());
            }
            acc.hash(&mut h);
        }
        // Not valid keys (rejected at CanonKey::new); hashed defensively for totality.
        Value::Closure { .. } => 9u8.hash(&mut h),
        Value::Fn { .. } => 10u8.hash(&mut h),
        // Handled by the FreeMonoid branch above.
        Value::List(_) | Value::Str(_) => unreachable!("FreeMonoid handled above"),
    }
    h.finish()
}

/// Order-independent combination of `(field_name, field_value)` hashes, so two
/// records/variants whose `fields` HashMaps iterate in different orders (but hold the
/// same entries) combine to the same value — matching `Value::eq`'s HashMap compare.
fn hash_fields_commutative(fields: &HashMap<Symbol, Value>) -> u64 {
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
    // List/Map keep an Rc handle around the persistent carrier: Rc::as_ptr
    // remains the value-identity used by pure-call memo keys
    // (value_rc_identity) and the accounting walker's sharing dedup. The
    // carrier inside the Rc shares structure across versions; the handle
    // clone needed before an update is O(1).
    List(Rc<RrbVector<Value>>),
    Map(Rc<HamtMap<CanonKey, Value>>),
    /// String membership sets (`Set<String>` in .dag).
    Set(Rc<BTreeSet<String>>),
    Record {
        type_name: Symbol,
        fields: Rc<HashMap<Symbol, Value>>,
    },
    Variant {
        type_name: Symbol,
        variant_name: Symbol,
        fields: Rc<HashMap<Symbol, Value>>,
    },
    Closure {
        params: Vec<Symbol>,
        body: Rc<Node>,
        env: Rc<Env>,
    },
    /// Reference to a module-level `fn` / `func` item (first-class function value).
    Fn {
        node: Rc<Node>,
    },
    Unit,
}

/// Wrap a list carrier (or anything convertible to one, e.g. `Vec<Value>`)
/// into a `Value::List`.
pub(crate) fn list_value(items: impl Into<RrbVector<Value>>) -> Value {
    Value::List(Rc::new(items.into()))
}

/// Wrap a map carrier into a `Value::Map`.
fn map_value(entries: HamtMap<CanonKey, Value>) -> Value {
    Value::Map(Rc::new(entries))
}

/// Build `Optional<T>::Present { value }` for structural `map_get` results.
fn optional_present(value: Value, ctx: &InterpContext) -> Value {
    let mut fields = HashMap::new();
    fields.insert(ctx.sym("value"), value);
    Value::Variant {
        type_name: ctx.sym("Optional"),
        variant_name: ctx.sym("Present"),
        fields: Rc::new(fields),
    }
}

/// Build `Optional<T>::Absent` for a native-map miss (`raw_map_lookup` -> Null).
fn optional_absent(ctx: &InterpContext) -> Value {
    Value::Variant {
        type_name: ctx.sym("Optional"),
        variant_name: ctx.sym("Absent"),
        fields: Rc::new(HashMap::new()),
    }
}

/// Wrap a `raw_map_lookup` probe as the typed `map_get` surface (`Optional<V>`).
fn map_lookup_as_optional(raw: Value, ctx: &InterpContext) -> Value {
    if matches!(raw, Value::Null) {
        optional_absent(ctx)
    } else {
        optional_present(raw, ctx)
    }
}

impl Value {
    /// Public name of this value's kind (e.g. "List", "Record", "Variant"),
    /// for host diagnostics that walk interpreter values (see `claim_executor`).
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
            // List <-> FreeMonoid alias-transparency. `List<T>` IS `FreeMonoid<T>` (std), and
            // the alias is already honored in pattern matching (the Value::List -> Empty/Cons
            // bridge) and in every list operation (free_monoid_to_vec / expect_list accept
            // either representation). Equality is the single site it was never honored: a list
            // literal builds Value::List, while snoc-built sequences (list_snoc_item — e.g.
            // Node.children rebuilt by a fold) build an Empty/Cons Variant chain. Flatten BOTH
            // sides through the canonical free_monoid_to_vec and compare element-wise (this
            // recurses through `==`, so nested mixed representations reconcile too). A Variant
            // that is not a well-formed Empty/Cons chain flattens to None, so a genuine
            // non-list Variant (e.g. Some/None) still never equals a List.
            (Value::List(_), Value::Variant { .. }) | (Value::Variant { .. }, Value::List(_)) => {
                match (free_monoid_to_vec(self), free_monoid_to_vec(other)) {
                    (Some(a), Some(b)) => a == b,
                    _ => false,
                }
            }
            // Same alias-transparency for `type String = FreeMonoid<Char>` (std/text.dag): a
            // native Value::Str and a snoc/Cons-built (or list-literal) char sequence denote the
            // same FreeMonoid<Char>. Flatten both through free_monoid_to_vec (Str -> codepoint
            // Ints because Char = Nat) and compare. (Str,Str) is handled natively above; this
            // only adds the cross-representation pairings, so it never slows the common
            // string-equality path.
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

// ---------------------------------------------------------------------------
// Environment
// ---------------------------------------------------------------------------

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
            InterpError::NoSuchFunction { name } => write!(f, "no such function: {}", name),
            InterpError::NoMainFunction => write!(f, "no main function found"),
            InterpError::NoSuchVariable { name } => write!(f, "undefined variable: {}", name),
            InterpError::NoSuchField { type_name, field } => {
                write!(f, "no field '{}' on type '{}'", field, type_name)
            }
            InterpError::TypeError { msg } => write!(f, "type error: {}", msg),
            InterpError::PatternMatchFailure { value } => {
                write!(f, "non-exhaustive pattern match on: {}", value)
            }
            InterpError::DivisionByZero => write!(f, "division by zero"),
            InterpError::Unimplemented { what } => write!(f, "not yet implemented: {}", what),
            InterpError::EarlyReturn { .. } => write!(f, "internal: uncaught early return"),
        }
    }
}

pub type InterpResult<T> = Result<T, InterpError>;

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

/// (service_node, operation_node) pair for service dispatch.
type ServiceOp = (Rc<Node>, Rc<Node>);

/// Memo for the explicitly-verified pure structural predicates (see
/// `is_structural_pure_fn`), keyed by (fn node address, arg Rc addresses).
/// The keepalive vecs are load-bearing for the address keys: a memoized arg's
/// Rc must stay alive while its address is a live key, or a freed-and-reused
/// allocation would alias a stale entry. Scoped to an `InterpContext`, both
/// the map and the pins drop with the evaluation instead of growing for the
/// life of the process.
#[derive(Default)]
struct PureCallMemo {
    map: HashMap<(usize, Vec<usize>), Value>,
    // Args kept alive so their Rc addresses (used in keys) can't be freed and aliased.
    keepalive: Vec<Value>,
    // Resolved fn nodes kept alive for the same reason (the key includes their address).
    keepalive_fns: Vec<Rc<Node>>,
}

/// In-process parse-table memo — minimal Realization host handler for
/// `ParseTableRealization` (§2 sccache pattern; spec in `extdeps/realization/parse_table_memo.dag`
/// `parse_table_memo_facts` + `02_parse.dag` `parse_table_lookup`/`parse_table_insert`).
///
/// Rust owns only lookup/insert glue keyed by the carrier's content-address fields;
/// miss/compute/insert *decision* stays in `parse_nonterminal_memoized` (.dag). No
/// `length()`-style substrate builtin was added (#5084 negative receipt).
///
/// Scoped to `InterpContext` like `PureCallMemo`. Keyed by (grammar_digest,
/// token_stream_digest, position, production) — same tuple `parse_table_memo_scope_and_key`
/// extracts from the carrier record the pure spec constructs.
#[derive(Default)]
struct ParseTableMemo {
    map: HashMap<(String, String, i64, Symbol), Value>,
    keepalive: Vec<Value>,
    lookups: u64,
    hits: u64,
    inserts: u64,
}

/// Parse-table memo counters for amortization witnesses (grammar-scoped reuse across files).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ParseTableMemoStats {
    pub lookups: u64,
    pub hits: u64,
    pub inserts: u64,
}

// ---------------------------------------------------------------------------
// Phase-0 memory measurement (ctrl#1533)
// ---------------------------------------------------------------------------

/// Copy-work counters for the collection-update primitives.
///
/// `*_calls` is the linear term; `*_copied` counts entries copied from
/// already-existing collections into the freshly allocated result — the
/// quadratic term a persistent carrier removes. Fold-building an n-entry
/// collection shows `copied ≈ n·(n−1)/2` here; that ratio (copied/calls) is
/// the before/after receipt for the ctrl#1533 carrier work.
///
/// One definition per counter, chosen by operation SEMANTICS — not by which
/// dispatch path (method, builtin function, binop) executed it:
/// - add-one ops (`map_insert`, `list_push`, `set_insert`): `*_copied` is the
///   receiver's pre-existing entries. The added element is excluded — it must
///   be written under any carrier, so it is not copy-on-update overhead.
/// - merge ops (`map_merge`, `list_concat`, `set_union`): `*_copied` is BOTH
///   operands' entries — every element of the result is a copy of a
///   pre-existing collection member.
///
/// A `.concat`/`.append`/`.push` method call is bucketed by what its argument
/// IS: a collection argument merges (`list_concat`), an atomic argument
/// appends one element (`list_push`).
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

/// Per-variant slice of a retained-value walk.
#[derive(Default, Clone)]
pub struct VariantAccounting {
    /// Value occurrences reached (including re-encounters of shared payloads).
    pub occurrences: u64,
    /// Distinct heap allocations behind those occurrences.
    pub unique_allocations: u64,
    /// Occurrences whose payload was already visited (structural sharing hits).
    pub shared_references: u64,
    /// Estimated heap bytes of the unique allocations (buffers + key strings;
    /// excludes allocator/bucket overhead and `Rc` headers — a floor, not RSS).
    pub heap_bytes: u64,
}

/// Sharing-aware byte accounting over a set of retained root values.
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

/// Marks `ptr` visited; returns false (and records a sharing hit) if it was
/// already visited, so each allocation is accounted exactly once.
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
    fields: &Rc<HashMap<Symbol, Value>>,
    visited: &mut std::collections::HashSet<usize>,
    acc: &mut MemoryAccounting,
) {
    if !accounting_first_visit(Rc::as_ptr(fields) as usize, label, visited, acc) {
        return;
    }
    let mut bytes = (fields.len() * std::mem::size_of::<(Symbol, Value)>()) as u64;
    acc.add_unique(label, bytes);
    for value in fields.values() {
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
        // Strings are owned per occurrence (not Rc-shared): every occurrence
        // is its own allocation. type/variant/field names are accounted at
        // their carriers below.
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
        // The AST body is program text (shared, graph-owned), not value heap;
        // only the captured environment chain is retained value memory.
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

/// How service operations are executed: hermetic replays recorded fixtures (when a
/// fixture store is configured) or modeled `mock_response` data; wet dispatches
/// live transports; record is wet capture into the fixture store.
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
    /// Service execution mode: hermetic replays fixtures or mock responses; wet/record dispatch live.
    pub execution_mode: ExecutionMode,
    /// Optional fixture store for hermetic replay and `--record` wet capture.
    pub fixture_store: Option<Rc<crate::recorded_fixture::RecordedFixtureStore>>,
    /// Cache for evaluated `data` items (immutable global constants), keyed by
    /// the data item's node identity. Preserves structural sharing across
    /// references so a `data` referenced N times yields ONE Value, not N
    /// rebuilds. Scoped to this context — a `data` item's cached value is valid
    /// exactly as long as its resolved graph is the one being evaluated, and
    /// the context is the owner with that lifetime. Keys (`Rc::as_ptr` of nodes
    /// in `fn_nodes`) cannot dangle or alias across graphs: `fn_nodes` holds
    /// the nodes alive as long as the cache, and the cache drops with the
    /// context.
    data_cache: std::cell::RefCell<HashMap<usize, Value>>,
    /// Memo for pure structural predicates, with the same context scoping as
    /// `data_cache` (see `PureCallMemo`).
    pure_call_memo: std::cell::RefCell<PureCallMemo>,
    /// Lazy parse-table memo for `parse_table_lookup` / `parse_table_insert` (Realization handler).
    parse_table_memo: std::cell::RefCell<ParseTableMemo>,
    /// Phase-0 measurement counters (ctrl#1533): copy work done by the
    /// copy-on-update collection primitives, scoped to this context like the
    /// caches above. Always on — a few integer adds next to O(n) clones.
    mutation_counters: std::cell::RefCell<MutationCounters>,
    /// Single name intern table for runtime identity carriers (ctrl#1533 phase 3).
    symbols: RefCell<SymbolInterner>,
    /// Memoized membership set of the PUBLISHED mock corpus — the §2 Realization pure-spec
    /// the M2 mock_totality_lens also reads (the operation keys that are hermetically
    /// realizable). The M4 hermetic-realization fold makes eval_service_call decide replay
    /// from this ONE model rather than a parallel ledger. Computed once on first hermetic
    /// service call via `resolve_published_mock_keys`; the corpus is an immutable global
    /// constant for this context, with the same scoping as `data_cache`.
    published_mock_keys: RefCell<Option<Rc<std::collections::HashSet<String>>>>,
    /// When set (M4.1), the published mock corpus was precomputed from the whole dsl/ tree —
    /// independent of this context's entry import closure. Supersedes closure-scoped
    /// `resolve_published_mock_keys` on first access.
    whole_tree_published_keys: Option<Rc<std::collections::HashSet<String>>>,
    /// Memoized set of service names that publish at least one mock case — O(1) membership for
    /// the M4 hermetic-realization fold's corpus-governed check.
    governed_services: RefCell<Option<Rc<std::collections::HashSet<String>>>>,
}

impl InterpContext {
    /// Intern a source name into this context's symbol table.
    pub fn sym(&self, s: &str) -> Symbol {
        self.symbols.borrow_mut().intern(s)
    }

    /// Resolve an interned symbol back to its source spelling.
    pub fn resolve(&self, sym: Symbol) -> String {
        self.symbols.borrow().resolve(sym).to_string()
    }

    /// True when `sym` resolves to `name` in this context's intern table.
    pub fn sym_eq(&self, sym: Symbol, name: &str) -> bool {
        self.symbols.borrow().resolve(sym) == name
    }

    /// Lookup a named field in an intern-keyed field map.
    pub fn field<'a>(&self, fields: &'a HashMap<Symbol, Value>, name: &str) -> Option<&'a Value> {
        fields.get(&self.sym(name))
    }

    /// Render a `Value` for host diagnostics after evaluation (resolves interned
    /// type/variant/field names via this context's symbol table).
    pub fn format_value(&self, val: &Value) -> String {
        with_active_ctx(self, || format!("{}", val))
    }

    /// Snapshot of the copy-work counters accumulated by evaluations in this
    /// context (phase-0 measurement, ctrl#1533).
    pub fn mutation_counters_snapshot(&self) -> MutationCounters {
        self.mutation_counters.borrow().clone()
    }

    /// Snapshot the symbol-interning receipt for this context (see
    /// [`InternStats`]): how many identity-name `intern` calls ran, how many
    /// distinct symbols were retained, and how many repeat `String`
    /// allocations interning avoided (the #4799 dedup receipt).
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

    /// Sharing-aware byte accounting over everything this context retains
    /// (`data_cache`, pure-call memo results and keepalive pins) plus any
    /// `extra_roots` (e.g. final claim values). One visited set across all
    /// roots, so cross-root structural sharing is counted once.
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

    /// Build an interpreter context. When `whole_tree_published_keys` is `Some`, hermetic
    /// governance reads that precomputed set (M4.1 whole-tree corpus) instead of scanning only
    /// the entry closure's `item_registry`.
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
                // Index service operations by checking ItemInfo kind
                if let Some(info) = graph.item_registry.get(&name) {
                    if info.kind == ItemKind::ServiceItem {
                        for op in item.children.iter() {
                            let op_name = authored_name_at(source_indices.clone(), op.clone());
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
                            let op_name = authored_name_at(source_indices.clone(), op.clone());
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
            execution_mode,
            fixture_store,
            data_cache: std::cell::RefCell::new(HashMap::new()),
            pure_call_memo: std::cell::RefCell::new(PureCallMemo::default()),
            parse_table_memo: std::cell::RefCell::new(ParseTableMemo::default()),
            mutation_counters: std::cell::RefCell::new(MutationCounters::default()),
            symbols: RefCell::new(SymbolInterner::default()),
            published_mock_keys: RefCell::new(None),
            whole_tree_published_keys,
            governed_services: RefCell::new(None),
        }
    }

    /// The published mock corpus as an operation-key set (the §2 Realization pure-spec the
    /// M2 mock_totality_lens also reads). Memoized: the corpus is an immutable global
    /// constant for this context, so it is resolved at most once.
    fn published_mock_keys(&self) -> InterpResult<Rc<std::collections::HashSet<String>>> {
        {
            if let Some(keys) = self.published_mock_keys.borrow().as_ref() {
                return Ok(keys.clone());
            }
        }
        let keys = if let Some(seed) = self.whole_tree_published_keys.as_ref() {
            if seed.is_empty() {
                // dsl/ precompute found no corpora (or no dsl root) — fall back to the entry
                // closure scan rather than treating universal governance as empty.
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

    /// Service names that publish at least one hermetic mock case — precomputed once from the
    /// published key set for O(1) corpus-governed checks in `eval_service_call`.
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

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

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

/// Run an entry function against an existing context. Callers that evaluate
/// many functions over the SAME resolved graph (see the `claim_batch` bin)
/// build one `InterpContext` and loop this — sharing the per-context `data`
/// cache and fn index across calls instead of rebuilding them per function.
pub fn run_in_context(
    ctx: &InterpContext,
    entry_fn: &str,
    eager_data_env: bool,
) -> InterpResult<Value> {
    with_active_ctx(ctx, || {
        // Find the entry function
        let item_node = ctx
            .lookup_fn(entry_fn)
            .ok_or_else(|| InterpError::NoMainFunction)?
            .clone();

        // Default: evaluate all `data` items up front (legacy `dag run` behavior).
        // Claim-run mode skips this — src/v2 has hundreds of TestClaim data graphs;
        // witnesses pull only what they need via lazy data-item resolution in eval_var.
        let env = if eager_data_env {
            build_initial_env(ctx)?
        } else {
            Env::empty()
        };

        // Call the entry function with no arguments
        call_function(ctx, &item_node, &[], &env)
    })
}

/// Run an entry function with bound arguments, returning its `Value`. This is the
/// embedding seam for a Rust HOST that drives a `.dag` function with inputs read from
/// the native environment — e.g. the CI timings collector reads `GITHUB_RUN_ID` via
/// `std::env::var` and binds it as the `run_id` arg of `collect_run_job_windows`, which
/// then fires its REST call shell-free through `dispatch_rest`. It adds NO language
/// primitive: it only exposes the arg-bound `call_function` that `run_in_context`
/// already performs with an empty arg list. Args are `(Option<name>, Value)` — `None`
/// binds positionally, `Some(name)` binds by parameter name.
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

/// Evaluate all `data` items to build the initial environment.
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

/// Evaluate every `data` initializer in `ctx` (the same values `build_initial_env` binds).
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

/// Evaluate one named `data` initializer (fail-closed if missing or not data).
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

// ---------------------------------------------------------------------------
// Function call
// ---------------------------------------------------------------------------

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

    // Bind parameters
    // Positional argument binding must target VALUE params only. A generic type param
    // (e.g. `<T>` in `fn outcome_rejected<T>(d: Diagnostic)`) also appears in
    // `fn_node.params`; a value param is exactly one whose type-expr exists and differs from
    // its own name (`d`'s type-expr is `Diagnostic`), whereas a type param's type-expr is
    // itself (`T`'s is `T`). Counting type params would shift the positional index so the
    // real value param never receives its arg.
    let param_names: Vec<String> = fn_node
        .params
        .iter()
        .filter(|p| {
            let name = authored_name_at(ctx.si(), (*p).clone());
            match p.children.first() {
                Some(type_expr) => authored_name_at(ctx.si(), type_expr.clone()) != name,
                None => false,
            }
        })
        .map(|p| authored_name_at(ctx.si(), p.clone()))
        .collect();

    let mut bindings = HashMap::new();
    if !args.is_empty() {
        // Named argument matching
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

    // Fill default values for unbound parameters
    for param in fn_node.params.iter() {
        let pname = authored_name_at(ctx.si(), param.clone());
        if !bindings.contains_key(&ctx.sym(&pname)) {
            if let Some(default_node) = param_node_default_value(param.clone()) {
                let default_val = eval_expr(&default_node, env, ctx)?;
                bindings.insert(ctx.sym(&pname), default_val);
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

fn eval_expr(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    // Fast path: no profiling. Zero added work on the green/CI eval path — the
    // per-instruction profiler (below) only engages under GUNBC_INTERP_PROFILE=1.
    if !eval_profile_enabled() {
        return stacker::maybe_grow(64 * 1024, 2 * 1024 * 1024, || {
            eval_expr_inner(node, env, ctx)
        });
    }

    // Profiling path: record one eval of this instruction, then attribute SELF
    // time = gross frame time minus time spent in child `eval_expr` frames. All
    // recursive sub-evaluation routes through this same function, so each frame's
    // gross time bubbles up to its parent via CHILD_NANOS — the classic
    // self-time decomposition. Runs on both the Ok and Err (`?`-propagated)
    // paths because the result is captured, not early-returned.
    //
    // Phase-0 keystone: when an active cache-subject is set, self-time rolls up
    // to that content-hash key (PerformanceReceipt grain) instead of per-variant
    // buckets. Variant counters remain as an opt-in deep projection when no
    // subject is active.
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
    // Phase-0 keystone: v1 proving handler for std.realization_measurement
    // RealizationMeasureEffect::ObserveElapsedAtSubject (see v1_eval_expr_measure_handler_id).
    // Durable model + roll-up lens live in .dag; this tap is host-effect realization only.
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
        LiteralValue::LitSymbol { value } => Ok(Value::Str(value.clone())),
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
            type_name: ctx.sym(parent_enum),
            variant_name: ctx.sym(&name),
            fields: Rc::new(HashMap::new()),
        });
    }

    // Environment lookup
    if let Some(val) = env.lookup(ctx.sym(&name)) {
        return Ok(val.clone());
    }

    // Data item lookup (evaluated lazily if not in env)
    if let Some(info) = v1_rt::map_get(&ctx.item_registry, name.clone()) {
        if info.kind == ItemKind::DataItem {
            if let Some(fn_node) = ctx.lookup_fn(&name) {
                if let Some(ref body) = fn_node.body {
                    // Symbol-declaration idiom: `data X: Symbol = X` is self-referential.
                    // Resolve it to the symbol value (its interned name) instead of
                    // evaluating the body, which would recurse `eval_var(X) -> eval_var(X)`
                    // forever. Symbol values are their name; equality is by name.
                    if let ExprData::ExprVar { .. } = &*body.expr_data {
                        if expr_var_name_at(body.clone(), ctx.si()) == name {
                            return Ok(Value::Str(name));
                        }
                    }
                    // Data items are immutable global constants: evaluate the body once and
                    // cache the resulting Value, returning the shared Rc on later references.
                    // A `data` referenced N times then yields ONE Value instead of N rebuilds,
                    // removing the dominant re-derivation cost in the emit pipeline.
                    //
                    // Evaluate against Env::empty(), NOT the caller's env: a data item is
                    // module-scoped and must not resolve names against caller locals, or the
                    // cached value would depend on whichever env first referenced it (unsound).
                    // This matches the eager-preload path (build_initial_env) which also uses
                    // Env::empty(), so lazy and eager resolution agree.
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
        // Module-level fn/func items used as first-class values (higher-order refs).
        // Precedence: eval_call resolves the callee via ctx.lookup_fn before env-bound
        // Value::Fn, so a local `let f = …` does not shadow a same-named module fn at call sites.
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

// ---------------------------------------------------------------------------
// Binary operators
// ---------------------------------------------------------------------------

fn eval_binop(op: &BinOp, left: Value, right: Value, ctx: &InterpContext) -> InterpResult<Value> {
    // NullCoalesce: short-circuit
    if matches!(op, BinOp::NullCoalesce) {
        return Ok(if matches!(left, Value::Null) {
            right
        } else {
            left
        });
    }

    // String concatenation
    if matches!(op, BinOp::Add) {
        if let (Value::Str(a), Value::Str(b)) = (&left, &right) {
            return Ok(Value::Str(format!("{}{}", a, b)));
        }
    }

    // List concatenation — List<T> IS FreeMonoid<T>; flatten operands (Option-B chokepoint).
    // Str operands stay atomic when mixed with lists (ctrl#1476 B1; same as .append/.concat).
    if matches!(op, BinOp::Add) {
        // Counter buckets follow operation semantics (see `MutationCounters`):
        // an atomic Str operand appends as ONE new element (push: copy-work is
        // the list operand only), list⊕list merges (concat: both operands).
        let record_push = |copied: usize| {
            let mut counters = ctx.mutation_counters.borrow_mut();
            counters.list_push_calls += 1;
            counters.list_push_items_copied += copied as u64;
        };
        match (&left, &right) {
            (l, Value::Str(s)) => {
                if let Some(mut result) = free_monoid_to_vec(l) {
                    record_push(result.len());
                    result.push(Value::Str(s.clone()));
                    return Ok(list_value((result)));
                }
            }
            (Value::Str(s), r) => {
                if let Some(result) = free_monoid_to_vec(r) {
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
            msg: format!(
                "cannot apply {:?} to {} and {}",
                op,
                left.type_label(),
                right.type_label()
            ),
        }),
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

// ---------------------------------------------------------------------------
// Let expression
// ---------------------------------------------------------------------------

fn eval_let(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let name = let_binding_name_at(node.clone(), ctx.si());
    let val = eval_expr(&let_value(node.clone()), env, ctx)?;
    let new_env = Env::with_binding(env, ctx.sym(&name), val);
    match let_body(node.clone()) {
        Some(body) => eval_expr(&body, &new_env, ctx),
        None => Ok(Value::Unit),
    }
}

// ---------------------------------------------------------------------------
// Block expression
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Match expression
// ---------------------------------------------------------------------------

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
    let mut anchor_fields = HashMap::new();
    anchor_fields.insert(ctx.sym("at"), Value::Str("map_lookup_port".to_string()));

    let mut locus_fields = HashMap::new();
    locus_fields.insert(
        ctx.sym("anchor"),
        Value::Record {
            type_name: ctx.sym("LocusAnchor"),
            fields: Rc::new(anchor_fields),
        },
    );

    let mut correction_fields = HashMap::new();
    correction_fields.insert(
        ctx.sym("reason"),
        Value::Variant {
            type_name: ctx.sym("NoCorrectionReason"),
            variant_name: ctx.sym("ExternalContractUnknown"),
            fields: Rc::new(HashMap::new()),
        },
    );

    let mut diagnostic_fields = HashMap::new();
    diagnostic_fields.insert(ctx.sym("reason"), Value::Str("map_key_absent".to_string()));
    diagnostic_fields.insert(
        ctx.sym("at"),
        Value::Variant {
            type_name: ctx.sym("Locus"),
            variant_name: ctx.sym("PortLocus"),
            fields: Rc::new(locus_fields),
        },
    );
    diagnostic_fields.insert(
        ctx.sym("correction"),
        Value::Variant {
            type_name: ctx.sym("Correction"),
            variant_name: ctx.sym("Unavailable"),
            fields: Rc::new(correction_fields),
        },
    );

    Value::Record {
        type_name: ctx.sym("Diagnostic"),
        fields: Rc::new(diagnostic_fields),
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
            match value {
                // Match on variant
                Value::Variant {
                    variant_name,
                    fields,
                    ..
                } => {
                    // Symmetric Witness projection: a *present* coproduct value bridges to
                    // `v1_rt::Witness::Holds { value: <self> }`. A native `Value::Map` lookup returns the raw
                    // stored value, and a `-> Witness`-annotated `.lookup` consumer (e.g.
                    // parse_table_lookup / lookup_table) matches `Holds`/`Violates`, so a
                    // present coproduct value must satisfy the `Holds` arm. Same absent-arm
                    // exclusions (`Violates`/`Absent`/`none` stay absent -> the `Violates` arm);
                    // a genuine `Holds` is handled by nominal matching below.
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
                        let field_name =
                            field_binding_name_at(fb.clone(), ctx.source_indices.clone());
                        let fb_pat = field_binding_pattern(fb.clone());
                        let field_val = fields
                            .get(&ctx.sym(&field_name))
                            .cloned()
                            .unwrap_or(Value::Null);
                        // Recursively match the field's binding pattern
                        let sub_bindings = match_pattern(&fb_pat, &field_val, ctx)?;
                        bindings.extend(sub_bindings);
                    }
                    Some(bindings)
                }
                // Destructure a record-typed value: `match r { TypeName { f, g } => ... }`.
                // Records build as Value::Record (no parent enum), so a VariantPattern whose
                // name is the record type must bind its fields here — the Value::Variant arm
                // above only covers coproduct variants.
                Value::Record { type_name, fields } => {
                    if *type_name != ctx.sym(name) {
                        return None;
                    }
                    let mut bindings = HashMap::new();
                    for fb in field_bindings.iter() {
                        let field_name =
                            field_binding_name_at(fb.clone(), ctx.source_indices.clone());
                        let fb_pat = field_binding_pattern(fb.clone());
                        let field_val = fields
                            .get(&ctx.sym(&field_name))
                            .cloned()
                            .unwrap_or(Value::Null);
                        let sub_bindings = match_pattern(&fb_pat, &field_val, ctx)?;
                        bindings.extend(sub_bindings);
                    }
                    Some(bindings)
                }
                // Bridge list literals (Value::List) to FreeMonoid Empty/Cons patterns:
                // `match xs { Empty => ..., Cons { head, tail } => ... }`. List literals build
                // as Value::List, but fold_list (std/algebra) matches the FreeMonoid coproduct.
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
                                // Cons has exactly head/tail; an unknown field (e.g. a typo
                                // `Cons { hd, tl }`) fails the match rather than binding null.
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
                // Bridge String values (Value::Str) to FreeMonoid<Char> Empty/Cons patterns:
                // `type String = FreeMonoid<Char>` and `type Char = Nat` (std/text.dag), so
                // fold_list/list_append walk a String as codepoint Int heads plus a String tail.
                // [Recurring class: List=FreeMonoid alias honored per-operation — this is the
                // String/Char surface of it; the representation-level dissolution is tracked
                // separately.]
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
                // Bridge host Int values into the modeled Nat coproduct. Numeric literals and
                // String/Char codepoint heads enter the interpreter as Value::Int, while std/nat.dag
                // matches on Zero/Succ. Negative values are not Nat inhabitants and fail the match.
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
                // Bridge the native-map-miss sentinel (Value::Null) into the Witness
                // coproduct, without reopening the deleted Optional Some/None bridge. A native
                // `Value::Map` lookup returns Null on a missing key (raw_map_lookup), but
                // the std contract is `Map.lookup: fn(K) -> Witness<V>` (v2.std.collection)
                // and the record-form empty_map presents an absent key as `Violates`. When a
                // record-form map delegates its miss to a native base map (empty_map builtin
                // shadows the .dag record form), the Null sentinel must still present as the
                // `Violates` (absent) arm rather than falling through a Holds/Violates match
                // non-exhaustively. `Holds` requires a present value and so never matches Null
                // (it falls to the `_ => None` arm below).
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
                // Diagnostics.None is represented by the interpreter's null sentinel.
                // Keep this parent-scoped so legacy Optional None stays rejected.
                Value::Null if name == "None" && parent_enum.as_deref() == Some("Diagnostics") => {
                    Some(HashMap::new())
                }
                // Match cardinality Optional through the canonical std surface.
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
                // Symmetric to the Witness value projection above: a present
                // non-Variant value (e.g. a native-map `Int` lookup hit) bridges to
                // `v1_rt::Witness::Holds { value: <self> }`. `Null` (a miss) does not match `Holds` — it
                // falls through to the `Null -> Violates` projection above.
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

// ---------------------------------------------------------------------------
// v2.std.node coproduct reflection intercept (module-scoped bootstrap seam)
// ---------------------------------------------------------------------------

pub(crate) const STD_NODE_BRIDGE_FNS: &[&str] = &[
    "resolve_type_node",
    "syntactic_coproduct_arm_keys",
    "syntactic_coproduct_arm_pairs",
];

// v2.std.compilers.lexing bootstrap bridge — dissolve-on: substrate Lexeme→Symbol intern carrier.
pub(crate) const STD_LEXING_BRIDGE_FNS: &[&str] = &["symbol_intern_lexeme"];

pub(crate) const STD_NODE_QUERY_BRIDGE_FNS: &[&str] = &["coproduct_nullary_inhabitants"];

// v2.std.concept_index corpus-enumeration SOURCE bridge — materializes each decl as a Node
// (substrate currency); ALL ConceptStruct/FieldRef shaping is .dag. dissolve-on: v2 compile-graph
// access (same trigger as resolve_type_node). forbidden: parallel corpus enumeration in Rust.
pub(crate) const STD_CONCEPT_INDEX_BRIDGE_FNS: &[&str] = &["concept_decl_facts_live"];

/// Sentinel surface for tests: every name here must be wired in `eval_call`'s bridge intercept.
pub fn std_node_bridge_fn_names() -> &'static [&'static str] {
    STD_NODE_BRIDGE_FNS
}

pub fn std_node_query_bridge_fn_names() -> &'static [&'static str] {
    STD_NODE_QUERY_BRIDGE_FNS
}

pub fn std_concept_index_bridge_fn_names() -> &'static [&'static str] {
    STD_CONCEPT_INDEX_BRIDGE_FNS
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

fn is_v4_std_lexing_bridge_call(ctx: &InterpContext, func_name: &str) -> bool {
    if !STD_LEXING_BRIDGE_FNS.contains(&func_name) {
        return false;
    }
    ctx.item_registry
        .get(func_name)
        .is_some_and(|info| info.module_name == "v2.std.compilers.lexing")
}

// ---------------------------------------------------------------------------
// Function call (ExprCall)
// ---------------------------------------------------------------------------

fn eval_call(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let func_name = expr_call_func_at(node.clone(), ctx.si());
    let arg_nodes = &node.children;

    // Evaluate arguments
    let args: Vec<(Option<String>, Value)> = arg_nodes
        .iter()
        .map(|arg_node| {
            let name = arg_name_at(arg_node.clone(), ctx.si());
            let val = eval_expr(&arg_value(arg_node.clone()), env, ctx)?;
            Ok((name, val))
        })
        .collect::<InterpResult<_>>()?;

    // R-reflect Phase 2a/2b: minimal v2.std.node bridges (resolve_type_node + Path-3 syntactic scan).
    if is_v4_std_node_bridge_call(ctx, &func_name) {
        return match func_name.as_str() {
            "resolve_type_node" => crate::coproduct_reflection::eval_resolve_type_node(ctx, &args),
            "syntactic_coproduct_arm_keys" => {
                crate::coproduct_reflection::eval_syntactic_coproduct_arm_keys(ctx, &args)
            }
            "syntactic_coproduct_arm_pairs" => {
                crate::coproduct_reflection::eval_syntactic_coproduct_arm_pairs(ctx, &args)
            }
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

    // std/algebra list folds: the interpreted recursive .dag bodies are correct but
    // pathological under the table-build hot loop (O(n) interpreter frames per element).
    // Native iterators preserve FreeMonoid semantics via `free_monoid_to_vec`.
    match func_name.as_str() {
        "fold_list" => return eval_fold_list_native(&args, env, ctx),
        "fold_list_right" => return eval_fold_list_right_native(&args, env, ctx),
        _ => {}
    }

    // Check for built-in runtime functions
    if let Some(result) = eval_builtin(&func_name, &args, ctx)? {
        return Ok(result);
    }

    // Look up user-defined function: module item wins over env-bound Value::Fn (see eval_var).
    let fn_node = if let Some(node) = ctx.lookup_fn(&func_name) {
        node.clone()
    } else {
        match env.lookup(ctx.sym(&func_name)) {
            Some(Value::Fn { node }) => node.clone(),
            // Calling a closure-valued parameter directly, e.g. fold_list's `cons(empty, h)`.
            // The arg names don't bind closure params (closures are positional), so pass values.
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

    // parse_table_lookup/insert: in-process Realization memo (content-keyed by table scope).
    if let Some(result) = try_parse_table_memo_dispatch(ctx, &func_name, &fn_node, &args, env)? {
        return Ok(result);
    }

    // Sharing-preservation memo: cache results of pure module functions keyed by the
    // RESOLVED function identity (collision-free) plus argument identities. Sound because a
    // module fn is deterministic in its args and same-Rc args ⇒ same value, so a hit is
    // never wrong. Builtins/closures returned above bypass this (they may carry effects or
    // captured env). Covers (a) nullary constructors and (b) the single-node structural
    // predicates (content_hash/well_formed/...), collapsing the emit pipeline's redundant
    // re-derivation and re-traversal. Args are kept alive so their Rc pointers can't be reused.
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

/// Build `Witness<T>::Holds { value }` for parse-table memo hits.
fn witness_holds(value: Value, ctx: &InterpContext) -> Value {
    let mut fields = HashMap::new();
    fields.insert(ctx.sym("value"), value);
    Value::Variant {
        type_name: ctx.sym("Witness"),
        variant_name: ctx.sym("Holds"),
        fields: Rc::new(fields),
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
    let position = match key_fields.get(&ctx.sym("position")) {
        Some(Value::Int(n)) => *n,
        _ => return None,
    };
    let production = match key_fields.get(&ctx.sym("production")) {
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
    // Purity is NOT assumed from arity: a nullary module fn can wrap an effectful service
    // call (eval_method_call -> transport dispatch), so caching it would run the effect once
    // and skip it thereafter. Restrict the memo to an explicitly-verified pure surface — the
    // structural Node predicates below, which contain no service/effect dispatch. Aggressive
    // pure-constructor caching is deferred until it has a checkable purity basis.
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

// ---------------------------------------------------------------------------
// Method call (ExprMethodCall)
// ---------------------------------------------------------------------------

fn eval_method_call(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let method_name = expr_method_name_at(node.clone(), ctx.si());
    let semantics = expr_method_call_semantics(node.clone());

    // Service calls: skip receiver evaluation (it's a service namespace, not a value).
    // Preserve named args for correct param binding (positional misaligns with defaults).
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

    // Non-service calls: evaluate receiver and args
    let receiver_val = eval_expr(&method_receiver(node.clone()), env, ctx)?;
    let extra_args = method_arg_nodes(node.clone());
    let args: Vec<Value> = extra_args
        .iter()
        .map(|a| eval_expr(&arg_value(a.clone()), env, ctx))
        .collect::<InterpResult<_>>()?;

    // Option-C dual-dispatch (ctrl#1476 B6): native `Value::Map` and record-form
    // `Map { lookup: fn }` share one raw key-probe chokepoint.
    if method_name == "lookup" {
        let key = args.first().ok_or_else(|| InterpError::TypeError {
            msg: "lookup requires a key argument".to_string(),
        })?;
        return raw_map_lookup(&receiver_val, key, env, ctx);
    }

    // Record/Variant field holding a function: `r.field(args)` calls the field's closure/fn.
    // Checked BEFORE semantics dispatch because it applies whether or not the method was
    // tagged AlgebraMethodSemantics — e.g. fold_node's `algebra.init(n)`/`algebra.step(...)`
    // over a NodeFold record.
    if let Value::Record { fields, .. } | Value::Variant { fields, .. } = &receiver_val {
        if let Some(field_val) = fields.get(&ctx.sym(&method_name)) {
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
            match expect_list(&base_val, "tuple.first") {
                Ok(items) => Ok(items.front().cloned().unwrap_or(Value::Null)),
                Err(_) => extract_field(&base_val, &field_name, env, ctx),
            }
        }
        Some(FieldAccessStyle::TupleSecond) => match expect_list(&base_val, "tuple.second") {
            Ok(items) => Ok(items.get(1).cloned().unwrap_or(Value::Null)),
            Err(_) => extract_field(&base_val, &field_name, env, ctx),
        },
        Some(FieldAccessStyle::OptionalUnwrap) => {
            // .value on Optional — unwrap or return Null
            match &base_val {
                Value::Null => Ok(Value::Null),
                _ => Ok(base_val),
            }
        }
        Some(FieldAccessStyle::EnumAccessor) => {
            // Accessing a discriminant field on an enum value
            extract_field(&base_val, &field_name, env, ctx)
        }
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
            fields
                .get(&field_sym)
                .cloned()
                .ok_or_else(|| InterpError::NoSuchField {
                    type_name: ctx.resolve(*type_name).to_string(),
                    field: field.to_string(),
                })
        }
        Value::Variant {
            type_name, fields, ..
        } => fields
            .get(&field_sym)
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

// ---------------------------------------------------------------------------
// Record literal
// ---------------------------------------------------------------------------

fn eval_record_lit(
    node: &Rc<Node>,
    parent_enum: Option<&str>,
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let type_name = record_lit_type_name_at(node.clone(), ctx.si()).unwrap_or_default();

    let mut fields = HashMap::new();
    for child in node.children.iter() {
        let fname = field_init_node_name_at(child.clone(), ctx.si());
        let fval = eval_expr(&field_init_node_value(child.clone()), env, ctx)?;
        fields.insert(ctx.sym(&fname), fval);
    }

    if let Some(pe) = parent_enum {
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

// ---------------------------------------------------------------------------
// String interpolation
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Cast
// ---------------------------------------------------------------------------

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

/// Next alias step from a type-decl `inferred` RHS (`String`, `Secret`, or the
/// base of `Base where …` before brand preservation).
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

/// Walk alias chains via type-decl items (pre-brand `inferred` RHS), not the
/// brand-preserved `TypeEnv` bindings used at use sites.
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

// ---------------------------------------------------------------------------
// ForEach
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Index
// ---------------------------------------------------------------------------

fn eval_index(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let base = eval_expr(&index_base(node.clone()), env, ctx)?;
    let idx = eval_expr(&index_expr(node.clone()), env, ctx)?;

    match (&base, &idx) {
        // Exclude Value::Str: a String IS a FreeMonoid<Char>, but indexing must keep its
        // dedicated Str arm (returns a one-char Str, not a char-list element via the
        // chokepoint) (ctrl#1476 B1; same Str-representation rule as concat/contains/slice).
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

// ---------------------------------------------------------------------------
// Slice
// ---------------------------------------------------------------------------

fn eval_slice(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let base = eval_expr(&slice_base(node.clone()), env, ctx)?;
    let start = eval_expr(&slice_start(node.clone()), env, ctx)?;
    let end = eval_expr(&slice_end(node.clone()), env, ctx)?;

    match (&base, &start, &end) {
        // Exclude Value::Str: slicing a String must return a substring (Str) via its
        // dedicated arm below, not a char-list via the FreeMonoid chokepoint
        // (ctrl#1476 B1; same Str-representation rule as concat/contains).
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
        // Option-C dual-dispatch chokepoint (ctrl#1476 B6): see `raw_map_lookup`.
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
                    // Cons/List flatten only — Value::Str stays one element (ctrl#1476 B1).
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

        "concat" | "append" | "push" => {
            // String concat preserves the String representation: a String IS a
            // FreeMonoid<Char>, but its canonical value form is Value::Str — concat must
            // not explode it to a char list via the FreeMonoid chokepoint (ctrl#1476 B1).
            if let Value::Str(s) = &receiver {
                let mut result = s.clone();
                for arg in args {
                    result.push_str(&format!("{}", arg));
                }
                return Ok(Value::Str(result));
            }
            if let Ok(items) = expect_list(&receiver, "concat") {
                let mut result = (*items).clone();
                // Counter bucket is decided by what the args ARE, not which
                // method name dispatched here (see `MutationCounters`): a
                // flattening collection arg merges (concat: both operands'
                // elements are copy-work), atomic args append one element each
                // (push: receiver elements only). Under the persistent carrier
                // (ctrl#1533 phase 2) native-List operands concatenate by
                // structural sharing, so *_items_copied counts only elements
                // actually copied through the FreeMonoid flatten.
                let mut merged_items = 0usize;
                let mut copied_items = 0usize;
                for arg in args {
                    // Non-list Str args append as one element, not char-exploded (ctrl#1476 B1).
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
                .map(|(i, v)| {
                    let mut fields = HashMap::new();
                    fields.insert(ctx.sym("index"), Value::Int(i as i64));
                    fields.insert(ctx.sym("value"), v.clone());
                    Value::Record {
                        type_name: ctx.sym("Pair"),
                        fields: Rc::new(fields),
                    }
                })
                .collect();
            Ok(list_value((result)))
        }

        // String/Map membership is checked BEFORE the FreeMonoid list path: a String IS a
        // FreeMonoid<Char>, but `.contains` on a String means substring containment, not
        // char-list membership — exploding it to chars would break multi-char queries
        // (ctrl#1476 B1; same Str-representation rule as `concat`).
        "contains" | "has" => match &receiver {
            Value::Map(m) => {
                // Keep the one-argument diagnostic boundary (P3): a missing key argument is
                // a typed error, not a silently-coerced `Null` membership probe. Arity is
                // validated FIRST, then the provided key is canonicalized. The key may be
                // any value; an un-keyable key (closure/fn/NaN) cannot be a member of a map
                // (insert rejects it), so it soundly answers false rather than fabricating.
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

        // Map key lookup is checked BEFORE the FreeMonoid list path: a String IS a
        // FreeMonoid<Char>, but `.get` on a String is not char-list indexing (ctrl#1476 B1;
        // same Str-representation rule as `index` / `slice` / `contains`).
        //
        // `map_get` is the structural-method name the typechecker desugars from bare
        // `map_get(m, k)` calls (std.graph adjacency, complexity.dag, …). It returns an
        // explicit `Optional` variant so empty-list hits (`Present { value: [] }`) do not
        // fall into the List/Empty/Cons pattern arm before the Optional bridge.
        // NOT added to eval_builtin — B-LOOKUP-1 routes typed v2.std.collection `map_get`
        // (Outcome<Optional<V>>) through eval_call as a user function instead.
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
            // Persistent update (ctrl#1533 phase 2): O(log32 n) path copy with
            // structural sharing — no entries are copied, so the
            // *_entries_copied counter is not incremented.
            let mut counters = ctx.mutation_counters.borrow_mut();
            counters.map_insert_calls += 1;
            drop(counters);
            Ok(map_value(m.update(ck, val)))
        }

        "merge" => {
            let base = expect_map(&receiver, "merge")?;
            let overlay = expect_map(args.first().unwrap_or(&Value::Null), "merge")?;
            // Persistent merge (ctrl#1533 phase 2): structural union sharing
            // unchanged subtrees — no entries are copied. im union keeps
            // self's value on key collision, so overlay-as-self preserves the
            // overlay-wins (last-write-wins) semantics.
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

// ---------------------------------------------------------------------------
// Service dispatch (I-2)
// ---------------------------------------------------------------------------

/// Infra clock for RecordedFixture `recorded_at` / freshness.
/// Routes through `Clock.UnixSecs` when the op is in the compiled closure; otherwise
/// uses the same transport realization (bootstrap — fixture stamping cannot recurse
/// through the fixture seam). Never `SystemTime`. `Clock.Now` (ISO) is for DAG consumers only.
pub fn fixture_now_secs(ctx: &InterpContext) -> Result<u64, crate::recorded_fixture::FixtureError> {
    if ctx.service_ops.contains_key("Clock.UnixSecs") {
        let val = wet_service_call(ctx, "Clock", "UnixSecs", &[], &Env::empty())
            .map_err(|_| crate::recorded_fixture::FixtureError::ClockUnavailable)?;
        unix_secs_from_clock_value(&val, ctx)
    } else {
        realize_clock_unix_secs_transport()
    }
}

/// Wet dispatch only — no hermetic fixture lookup (bootstrap / infra paths).
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

/// Transport realization for `Clock.UnixSecs` when the op is absent from the closure.
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

/// Wet shell.Env.Get transport — printenv realization (extdeps/shell/shell.dag).
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
        // Fail-closed: no raw printenv outside RecordedFixture on hermetic path.
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

    // Get effective transport (operation-level overrides service-level)
    let transport = op_node
        .transport
        .as_ref()
        .or(service_node.transport.as_ref())
        .ok_or_else(|| InterpError::TypeError {
            msg: format!("no transport for service {}", key),
        })?;

    // Bind input params to arg values
    let param_env = build_service_param_env(op_node, args, env, ctx)?;
    let inputs_hash =
        crate::recorded_fixture::content_hash_service_inputs(op_node, &param_env, ctx);
    let inputs_json =
        crate::recorded_fixture::service_inputs_fixture_json(op_node, &param_env, ctx)
            .map_err(|e| InterpError::TypeError { msg: e.to_string() })?;

    if ctx.execution_mode.is_hermetic() {
        // M4 hermetic-realization fold (§2 Realization): the PUBLISHED mock corpus is the
        // single authority for WHICH operations are hermetically realizable — the SAME model
        // the M2 mock_totality_lens reads. A service is "corpus-governed" iff it publishes at
        // least one case; for such a service the published model — NOT inline mock_* props —
        // decides realizability. One decision procedure, shared by the runtime and the lens.
        let published = ctx.published_mock_keys()?;
        let governed = ctx.governed_services()?;
        let service_is_governed = governed.contains(service_name);
        if service_is_governed && !published.contains(&key) {
            // §5 fail-closed on disagreement: the service publishes a corpus but THIS operation
            // is not a published case. Refuse loudly — even if a fixture or an inline mock_*
            // prop still exists for it. A runtime that fabricated/replayed here would let the
            // runtime decision and the published model diverge silently; this is the teeth.
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

        // Hermetic with fixture store: replay recorded response (fail-closed on miss/stale).
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
        // Retained inline-mock fallback for extdeps layers that do NOT yet publish a dsl mock
        // corpus (M4.0 migrated filesystem only). 🟡 dissolved-by —
        // feature:m4-eval-service-call-folds-hermetic-realization — dissolve-on: every extdeps
        // layer publishes a dsl-side PublishedMockCase corpus; when no corpus-free service
        // relies on inline mock_* props, this branch and eval_mock_response delete together and
        // the RecordedFixture store becomes the sole payload authority (§3).
        eprintln!("[hermetic:mock] {}.{}", service_name, op_name);
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
    // Shell transport dispatch
    if is_shell_transport(transport.clone()) {
        let result = dispatch_shell(transport, param_env, ctx)?;
        return map_shell_outputs(&result, op_node, ctx);
    }

    // File transport dispatch (filesystem read/write at the configured path)
    if is_file_transport(transport.clone(), ctx.si()) {
        let result = dispatch_file(op_node, transport, param_env, ctx)?;
        return map_file_outputs(&result, op_node, ctx);
    }

    // REST transport dispatch (any non-shell transport with service config endpoint)
    dispatch_rest(service_node, op_node, transport, param_env, ctx)
}

/// Build an environment with service operation params bound to arg values.
/// Uses named matching, with positional fallback + default values.
fn build_service_param_env(
    op_node: &Rc<Node>,
    args: &[(Option<String>, Value)],
    env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<Rc<Env>> {
    let mut bindings = HashMap::new();

    // First pass: bind named args by name
    for (opt_name, val) in args {
        if let Some(name) = opt_name {
            bindings.insert(ctx.sym(name), val.clone());
        }
    }

    // Second pass: bind remaining positional args to unbound params
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

    // Third pass: fill defaults for any remaining unbound params
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

/// Push one evaluated argv expression onto `argv`. List carriers splice
/// element-wise; strings stay atomic (never exploded to codepoints).
/// FreeMonoid<Char> chains whose heads are scalar char codes materialize to one
/// host string (v2 emit → shell.Exec.Run script boundary).
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

// **SCAFFOLD (DESIGN.md §7)** — host string materialization at shell/REST `{param}` templates.
//
// Hand-Rust scaffold receipt (P5 / §7):
// - **Deleted wrong path:** `scripts/source-root-ingest-gate.sh` (deleted on main); slice-3 gate
//   transport now emits `TargetSource` via row-driven bash emit (`src/v2/workflow/
//   source_root_ingest_transport.dag`) instead of the retired sidecar script.
// - **Red-by-execution (pre-scaffold):** `format!("{}", val)` on `FreeMonoid<Char>` Cons chains at
//   `substitute_template` / `push_shell_argv_tokens` printed Cons debug text, not script bytes —
//   `source_root_ingest_gate_passes` went RED on the CI floor until this flatten landed.
// - **Green witnesses:** `v2.compiler.manual.bash_emit_command_test.bash_emit_source_root_ingest_gate_holds`
//   (emit-time byte identity) + floor-plan `source_root_ingest_gate_passes` (Wet claim-run legs).
// - **This site:** `value_as_host_string` / `value_to_host_string` flatten FreeMonoid chains at
//   `eval_string_interp`, `substitute_template`, and `push_shell_argv_tokens`.
// - **Dissolve-on (ROADMAP):** `ROADMAP.md` Now → Lane 3a `SourceRootIngest` (#5126 / #5155 slice 3) —
//   delete these helpers when `TargetSource` gains a substrate coercion to flat Unicode at the shell
//   boundary, or transports take `ShellProgram` not raw emit output.

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

/// Materialize a runtime value for host-side string interpolation (shell argv
/// `{param}` templates, REST path templates). FreeMonoid<Char> / TargetSource
/// chains flatten to Unicode text; other values use Display.
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

/// Evaluate a shell transport's argv template with concrete param bindings — the same
/// materialization `dispatch_shell` performs before `Command::new`, without executing.
pub fn materialize_shell_argv_for_operation(
    path: String,
    service: String,
    operation: String,
    param_bindings: HashMap<String, Value>,
) -> Result<Vec<String>, String> {
    let (argv_nodes, source_indices) =
        crate::extdeps_shape_transport_policy_project::shell_argv_nodes_for_operation(
            path, service, operation,
        );
    let mut argv: Vec<String> = Vec::new();
    for node in argv_nodes.iter() {
        let val = materialize_argv_expr_for_bindings(node, &param_bindings, &source_indices)?;
        push_shell_argv_tokens(&mut argv, val).map_err(|e| format!("{e:?}"))?;
    }
    Ok(argv)
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
        push_shell_argv_tokens(&mut argv, val)?;
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
        stdout: String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string(),
        stderr: String::from_utf8_lossy(&output.stderr)
            .trim_end()
            .to_string(),
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
        Some(crate::v1_std_core::InferredNode::Resolved { node }) => node.clone(),
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
        let field_name = authored_name_at(ctx.si(), child.clone());
        // Check from_key property
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
            _ => {
                // Default: map by field name
                match field_name.as_str() {
                    "success" => Value::Bool(result.exit_code == 0),
                    "exit_code" => Value::Int(result.exit_code as i64),
                    "stdout" => Value::Str(result.stdout.clone()),
                    "stderr" => Value::Str(result.stderr.clone()),
                    "exists" => Value::Bool(result.exit_code == 0),
                    _ => Value::Null,
                }
            }
        };
        fields.insert(ctx.sym(&field_name), value);
    }

    // Return as record with the type name
    Ok(Value::Record {
        type_name: ctx.sym(&authored_name_at(ctx.si(), op_node.clone())),
        fields: Rc::new(fields),
    })
}

/// Extract the `from` key from a field's properties (e.g., `from "stdout"`).
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

// ---------------------------------------------------------------------------
// File transport dispatch (filesystem read/write — POSIX.1-2017 read(2)/write(2))
// ---------------------------------------------------------------------------

/// Outcome of a filesystem operation. Mirrors `ShellResult`: the host captures
/// the effect's observable result into a flat record and NEVER throws on an
/// expected failure (a missing dir, a permission denial) — exactly as the shell
/// transport returns its record even on a nonzero exit. The consumer branches on
/// `success`. An *unexpected* condition (no path configured) is still a loud
/// `InterpError` per DESIGN.md §5.
struct FileResult {
    /// True when the underlying read/write syscall succeeded.
    success: bool,
    /// Bytes written (write) or read (read). 0 on failure.
    byte_count: i64,
    /// The resolved POSIX pathname the effect acted on.
    path: String,
    /// Empty on success; the OS error text otherwise.
    error: String,
    /// Read payload; empty for writes.
    content: String,
}

/// Execute a file transport. The operation's static signature selects the
/// direction, deterministically (no runtime state-space conflation):
///   * a `content` input param present  -> WRITE that content to the path;
///   * absent                           -> READ the file at the path.
/// The path is the transport's `base_path` expression with `{param}`
/// placeholders substituted from the bound inputs — the same interpolation the
/// REST transport applies to its path template.
fn dispatch_file(
    op_node: &Rc<Node>,
    transport: &Rc<Node>,
    param_env: &Rc<Env>,
    ctx: &InterpContext,
) -> InterpResult<FileResult> {
    let si = ctx.si();

    // Resolve the target path: `base_path` (the key the file-transport parser
    // stores `path:`/`base_path:` under), evaluated then `{param}`-substituted.
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

    // Direction = does the operation declare a `content` input?
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
        eprintln!("[file] write {} ({} bytes)", path, byte_count);
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
        eprintln!("[file] read {}", path);
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

/// Map a `FileResult` onto the operation's output fields. Recognized `from`
/// keys (and, as a fallback, field names) name the channel each output reads
/// from — the file-transport analogue of shell's `from "stdout_lines"`:
///   `write_success` / `success` -> Bool   (syscall succeeded)
///   `bytes_written` / `bytes`   -> Int    (byte count)
///   `path`                      -> String (resolved pathname)
///   `error`                     -> String (OS error text; "" on success)
///   `content`                   -> String (read payload)
fn map_file_outputs(
    result: &FileResult,
    op_node: &Rc<Node>,
    ctx: &InterpContext,
) -> InterpResult<Value> {
    let return_type = match op_node.inferred.as_deref() {
        Some(crate::v1_std_core::InferredNode::Resolved { node }) => node.clone(),
        _ => {
            // No structured return type — a write reports success, a read its content.
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

    let mut fields = HashMap::new();
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
        fields.insert(ctx.sym(&field_name), value);
    }

    Ok(Value::Record {
        type_name: ctx.sym(&authored_name_at(ctx.si(), op_node.clone())),
        fields: Rc::new(fields),
    })
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
    let base_url =
        find_service_config_string(service_node, "svc_endpoint", &si).unwrap_or_default();

    // 2. Path template — evaluate as expression, then substitute {param} placeholders
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

    // 3. HTTP method — try string literal, fall back to authored name
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

    // 4. Auth header
    let (auth_header_name, auth_token) = resolve_auth(service_node, transport, &si, ctx);

    // 5. Custom headers from transport properties.
    // Non-reserved properties on the transport node are custom headers.
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

    // 6. Query params
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
                Value::Null => {} // skip null query params
                _ => query_params.push((qname, format!("{}", qval))),
            }
        }
    }

    // 7. Request body
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

    // 8. Response format
    let response_format = find_property_string(
        transport.properties.clone(),
        "response_format".to_string(),
        si.clone(),
    )
    .unwrap_or_else(|| "Json".to_string());

    // Build and send request
    eprintln!("[rest] {} {}", method, url);

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

/// Resolve auth header name and token from service config + environment.
fn resolve_auth(
    service_node: &Rc<Node>,
    _transport: &Rc<Node>,
    si: &Rc<HashMap<String, Rc<NewlineIndex>>>,
    ctx: &InterpContext,
) -> (String, Option<String>) {
    let mut header_name = "Authorization".to_string();
    let mut env_var_name: Option<String> = None;

    // Walk service config properties looking for auth-related declarations
    for prop in service_node.properties.iter() {
        let name = field_init_node_name_at(prop.clone(), si.clone());
        let val_node = field_init_node_value(prop.clone());

        match name.as_str() {
            "svc_auth" => {
                // Auth scheme: Bearer, Header("x-api-key"), etc.
                let scheme = authored_name_at(si.clone(), val_node.clone());
                if scheme == "Bearer" {
                    header_name = "Authorization".to_string();
                } else if scheme == "Header" || val_node.name == "Header" {
                    // Header("x-api-key") — extract the header name from children
                    // The string arg is in the first child (or its nested value)
                    for child in val_node.children.iter() {
                        if let Some(s) = extract_string_value(child) {
                            header_name = s;
                        } else {
                            // Might be an arg node with a child
                            for grandchild in child.children.iter() {
                                if let Some(s) = extract_string_value(grandchild) {
                                    header_name = s;
                                }
                            }
                        }
                    }
                }
            }
            "svc_auth_source" => {
                // Auth source: EnvVar { name: "GITHUB_TOKEN" } — extract the env var name
                // The value is a record node. Look for the "name" field in its children.
                for child in val_node.children.iter() {
                    let field_name = field_init_node_name_at(child.clone(), si.clone());
                    if field_name == "name" {
                        let field_val = field_init_node_value(child.clone());
                        env_var_name = extract_string_value(&field_val);
                    }
                }
                // Fallback: try the node's own name or literal value
                if env_var_name.is_none() {
                    env_var_name = extract_string_value(&val_node);
                }
            }
            _ => {}
        }
    }

    let token = env_var_name.and_then(|var| resolve_env_var_token(ctx, &var));
    (header_name, token)
}

/// Extract a string value from a node (literal or authored name).
fn extract_string_value(node: &Rc<Node>) -> Option<String> {
    if let ExprData::ExprLiteral { ref value } = *node.expr_data {
        if let LiteralValue::LitStr { value: s } = value.as_ref() {
            return Some(s.clone());
        }
    }
    None
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
            let val_node = field_init_node_value(prop.clone());
            // Try literal string
            if let ExprData::ExprLiteral { ref value } = *val_node.expr_data {
                if let LiteralValue::LitStr { value: s } = value.as_ref() {
                    return Some(s.clone());
                }
            }
            // Try authored name (for enum-like values)
            let authored = authored_name_at(si.clone(), val_node);
            if !authored.is_empty() {
                return Some(authored);
            }
        }
    }
    None
}

/// Substitute `{param}` placeholders in a template string with values from the environment.
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
                // Leave unresolved placeholders as-is
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

/// Convert a Value to serde_json::Value for request bodies. Fails closed (P3) on a
/// structural-key map: JSON object keys are strings, and rendering a non-String key
/// (e.g. `Int(1)` vs `Str("1")`) via its Display form would collapse distinct keys
/// and silently drop entries. Such a map has no faithful JSON-object representation.
/// Legacy JSON dump without wire contracts — retained only for non-REST paths if needed.
/// REST request bodies must use `wire_value_serialize::value_to_wire_json`.
fn value_to_json(val: &Value) -> InterpResult<serde_json::Value> {
    Ok(match val {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(n) => serde_json::json!(*n),
        Value::Float(f) => serde_json::json!(*f),
        Value::Str(s) => {
            // If the string contains JSON, parse it (bridge for Json-typed params)
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

/// Map a text response to the operation's return type.
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
    // Single field → return text
    if children.len() == 1 {
        return Ok(Value::Str(text.to_string()));
    }
    // Multi-field: map by from_key, defaulting to text for "text" and "body" fields
    let mut fields = HashMap::new();
    for child in children.iter() {
        let field_name = authored_name_at(ctx.si(), child.clone());
        fields.insert(ctx.sym(&field_name), Value::Str(text.to_string()));
    }
    Ok(Value::Record {
        type_name: ctx.sym(&authored_name_at(ctx.si(), op_node.clone())),
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
        Some(crate::v1_std_core::InferredNode::Resolved { node }) => node.clone(),
        _ => return Ok(json_to_value(json)),
    };
    let children = &return_type.children;
    if children.is_empty() {
        return Ok(json_to_value(json));
    }

    // If the return type itself is a List (not a record), return the JSON directly
    let type_name = authored_name_at(ctx.si(), return_type.clone());
    if type_name == "List" && children.is_empty() {
        return Ok(json_to_value(json));
    }

    // If response is an array but return type is a record with a list field,
    // wrap the array in that field.
    if json.is_array() && !children.is_empty() {
        let mut fields = HashMap::new();
        let first_field = authored_name_at(ctx.si(), children[0].clone());
        fields.insert(ctx.sym(&first_field), json_to_value(json));
        return Ok(Value::Record {
            type_name: ctx.sym(&authored_name_at(ctx.si(), op_node.clone())),
            fields: Rc::new(fields),
        });
    }

    // Multi-field record: extract fields via from_key JSON paths
    let mut fields = HashMap::new();
    for child in children.iter() {
        let field_name = authored_name_at(ctx.si(), child.clone());
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
                    None => {
                        // Single-field output wrapping: if the return type has
                        // exactly one field and the JSON doesn't have that key,
                        // wrap the entire response in that field.
                        if children.len() == 1 {
                            json_to_value(json)
                        } else {
                            Value::Null
                        }
                    }
                }
            }
        };
        fields.insert(ctx.sym(&field_name), val);
    }

    Ok(Value::Record {
        type_name: ctx.sym(&authored_name_at(ctx.si(), op_node.clone())),
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

/// True iff a type-annotation node tree names `target` anywhere (the type itself or a type
/// argument, e.g. the `PublishedMockCase` element of `List<PublishedMockCase>`). Reads SHAPE
/// over the small type expression — cheap, no evaluation.
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

/// Resolve the PUBLISHED mock corpus (the §2 Realization pure-spec) into its operation-key set.
/// Reads SHAPE, not contents (DESIGN.md lens law): every `data` item DECLARED over
/// `PublishedMockCase` contributes its rows' operation keys. This is the SAME corpus the M2
/// mock_totality_lens reads — the v1 runtime and the compile-time lens now decide hermetic
/// realizability from ONE model. The declared-type pre-filter keeps this cheap: only corpus data
/// items are evaluated, not every `data` constant in the loaded tree. Evaluated against
/// `Env::empty()` like other `data` constants; data items are pure (no service calls), so this
/// never re-enters `eval_service_call`.
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
        // Declared-type gate: skip (without evaluating) any data item not declared over
        // PublishedMockCase. Corpus data items always carry the explicit annotation.
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

/// Prefer grounded `service` + `operation` fields; fall back to legacy `operation_key`.
fn published_case_operation_key(
    ctx: &InterpContext,
    fields: &HashMap<Symbol, Value>,
) -> Option<String> {
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

/// Evaluate mock_response from an operation's properties for hermetic execution.
fn eval_mock_response(op_node: &Rc<Node>, ctx: &InterpContext) -> InterpResult<Value> {
    // Find first mock_* property
    for prop in op_node.properties.iter() {
        let prop_name = field_init_node_name_at(prop.clone(), ctx.si());
        if has_mock_prefix(prop_name) {
            // The mock property value is a record literal
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

// ---------------------------------------------------------------------------
// Built-in functions (v1_rt equivalents)
// ---------------------------------------------------------------------------

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

        // `discriminant(v)` reifies a coproduct/record value's own constructor name as a
        // `Symbol` (Symbol values are interned strings — see eval_var's `data X: Symbol = X`
        // idiom at Value::Str). This is the single intrinsic that dissolves the hand-written
        // `fn ..._discriminant(v) -> Symbol { match v { Ctor{..} => ctor_tag ... } }` bridges
        // that shadow a coproduct's arm-set with a parallel `data ctor_tag: Symbol` vocabulary.
        // The constructor's own name IS the discriminant; no per-type code is required.
        "discriminant" => match positional.first() {
            Some(Value::Variant { variant_name, .. }) => {
                Ok(Some(Value::Str(resolve_sym(*variant_name))))
            }
            Some(Value::Record { type_name, .. }) => Ok(Some(Value::Str(resolve_sym(*type_name)))),
            _ => Ok(None),
        },

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
            // Same semantic bucketing as the binop/method paths (see
            // `MutationCounters`): atomic Str operand → push, list⊕list → concat.
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

        // String IS FreeMonoid<Char>, but list builtins must not char-explode Str operands
        // (ctrl#1476 B1; same Str-representation rule as concat/contains/slice).
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

        // Function-call `contains` mirrors method `.contains`: strings use substring
        // containment, while FreeMonoid/List values use element membership.
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

        "list_push" | "append" => match positional.as_slice() {
            [list_val, item] if matches!(list_val, Value::Str(_)) => Ok(None),
            // Persistent O(log n) push_back (ctrl#1533 phase 2): a native
            // List shares structure with the prior version — only a chain
            // form copies (through the flatten), and only that is counted.
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
            // Persistent O(log(n+m)) RRB concatenation (ctrl#1533 phase 2):
            // native Lists share structure — only chain-form operands copy
            // (through the flatten), and only those items are counted.
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

        // Structural keys included: a finite map keys by any value with a decidable
        // identity (CanonKey), not just `Value::Str`. This keeps maps in the native
        // data representation (so whole-map `==` is decidable) instead of falling
        // through to the `.dag` closure form on a non-String key.
        "map_insert" => match positional.as_slice() {
            // A recognized native map_insert with an invalid key fails closed (P3) — it
            // does NOT return Ok(None), which would fall through to the `.dag` closure-form
            // map_insert and silently build an unmatchable map. `Ok(None)` here means only
            // "not this builtin shape" (a non-Map receiver).
            [Value::Map(m), k, v] => match CanonKey::new((*k).clone()) {
                Some(ck) => {
                    // Persistent update (ctrl#1533 phase 2): O(log32 n) path
                    // copy with structural sharing — no entries are copied.
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

        // `lookup` is the low-level raw map probe (present -> value, missing -> Null). The typed
        // std `map_get` (v2.std.collection, Outcome<Optional<V>>) wraps that probe into the
        // canonical Optional surface. `map_get` is NOT handled here on purpose: the builtin
        // arm previously SHADOWED the typed std map_get (eval_builtin wins over user fns at
        // eval_call), so `map_get(...)` returned the RAW value and any `match { Accepted; Rejected }`
        // consumer crashed non-exhaustively (B-LOOKUP-1). Dropping `map_get` here routes it to the
        // typed v2.std.collection authority. [List=FreeMonoid/Option-alias recurrence: the bridge
        // is honored per-operation (matching, ==, zip_eq, lookup) rather than once at the
        // representation — tracked for the post-R2 representation-level dissolution.]
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

        // Sound identity hint (substitute_generics short-circuit): true => the two are
        // structurally equal. The compiled seed answers with Rc pointer identity
        // (conservative); the interpreter has no Rc-of-Node identity, so it answers
        // with exact value equality -- still never true-when-unequal, so a caller that
        // returns the original on true preserves output structure under either
        // realization (content_hash / self-host fixed point unaffected).
        "rc_ptr_eq" | "rc_vec_ptr_eq" => match positional.as_slice() {
            [a, b] => Ok(Some(Value::Bool(a == b))),
            _ => Ok(None),
        },

        "map_merge" => match positional.as_slice() {
            [Value::Map(base), Value::Map(overlay)] => {
                // Persistent merge (ctrl#1533 phase 2): structural union, no
                // entry copies. im union keeps self's value on key collision,
                // so overlay-as-self preserves overlay-wins semantics.
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
            let result = v1_rt::filesystem_read(path);
            let mut fields = HashMap::new();
            fields.insert(ctx.sym("content"), Value::Str(result.content));
            Ok(Some(Value::Record {
                type_name: ctx.sym("FilesystemReadResult"),
                fields: Rc::new(fields),
            }))
        }

        "layer_import_facts" => {
            let std_roots = expect_str_list(positional.first().copied(), "layer_import_facts")?;
            let extdeps_roots = expect_str_list(positional.get(1).copied(), "layer_import_facts")?;
            let facts =
                crate::layering_imports_project::layer_import_facts(&std_roots, &extdeps_roots);
            let mut items: Vec<Value> = Vec::new();
            for f in facts {
                let layer = Value::Variant {
                    type_name: ctx.sym("LayerPrefix"),
                    variant_name: ctx.sym(f.layer),
                    fields: Rc::new(HashMap::new()),
                };
                let mut fields = HashMap::new();
                fields.insert(ctx.sym("layer"), layer);
                fields.insert(ctx.sym("path"), Value::Str(f.path));
                fields.insert(ctx.sym("import_module"), Value::Str(f.import_module));
                items.push(Value::Record {
                    type_name: ctx.sym("LayerImportFact"),
                    fields: Rc::new(fields),
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
            let facts = crate::import_resolution_project::import_resolution_facts(
                &pool_roots,
                &importer_roots,
                &exclude_substrings,
            );
            let mut items: Vec<Value> = Vec::new();
            for f in facts {
                let mut fields = HashMap::new();
                fields.insert(ctx.sym("path"), Value::Str(f.path));
                fields.insert(ctx.sym("import_module"), Value::Str(f.import_module));
                fields.insert(ctx.sym("target_declared"), Value::Bool(f.target_declared));
                items.push(Value::Record {
                    type_name: ctx.sym("ImportResolutionFact"),
                    fields: Rc::new(fields),
                });
            }
            Ok(Some(list_value(items)))
        }

        "fact_cardinality_cross_tree_coexistence_count" => Ok(Some(Value::Int(
            crate::fact_cardinality_census::cross_tree_coexistence_count(),
        ))),

        "fact_cardinality_cross_tree_diverged_fork_count" => Ok(Some(Value::Int(
            crate::fact_cardinality_census::cross_tree_diverged_fork_count(),
        ))),

        "fact_cardinality_cross_tree_is_coexistence" => {
            let key = expect_str(
                positional.first().copied(),
                "fact_cardinality_cross_tree_is_coexistence",
            )?;
            Ok(Some(Value::Bool(
                crate::fact_cardinality_census::cross_tree_is_coexistence(key),
            )))
        }

        "fact_cardinality_cross_tree_is_diverged_fork" => {
            let key = expect_str(
                positional.first().copied(),
                "fact_cardinality_cross_tree_is_diverged_fork",
            )?;
            Ok(Some(Value::Bool(
                crate::fact_cardinality_census::cross_tree_is_diverged_fork(key),
            )))
        }

        "languages_consumer_census_data_decl_count" => Ok(Some(Value::Int(
            crate::languages_consumer_census::languages_consumer_census_data_decl_count(),
        ))),

        "languages_consumer_census_per_language_row_count" => Ok(Some(Value::Int(
            crate::languages_consumer_census::languages_consumer_census_per_language_row_count(),
        ))),

        "languages_consumer_census_format_row_count" => Ok(Some(Value::Int(
            crate::languages_consumer_census::languages_consumer_census_format_row_count(),
        ))),

        "languages_consumer_census_external_consumer_count" => {
            let decl_name = expect_str(
                positional.first().copied(),
                "languages_consumer_census_external_consumer_count",
            )?;
            Ok(Some(Value::Int(
                crate::languages_consumer_census::languages_consumer_census_external_consumer_count(
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
                crate::languages_consumer_census::languages_consumer_census_is_composition_only(
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
                crate::languages_consumer_census::languages_consumer_census_has_external_consumer(
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
                crate::extdeps_shape_transport_policy_project::dead_param_count_for_operation(
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
                crate::extdeps_shape_transport_policy_project::dead_param_count_for_path(path);
            Ok(Some(Value::Int(count)))
        }

        "extdeps_embedded_policy_literal_count_for_path" => {
            let path = expect_str(
                positional.first().copied(),
                "extdeps_embedded_policy_literal_count_for_path",
            )?;
            let count =
                crate::extdeps_shape_transport_policy_project::embedded_policy_literal_count_for_path(
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
                crate::extdeps_shape_transport_policy_project::qualified_name_resolves_in_derived_module_set(
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
                crate::extdeps_shape_transport_policy_project::dead_param_count_for_qualified_name(
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
                crate::transport_script_position_project::transport_script_literal_violation_count_for_path(
                    path,
                );
            Ok(Some(Value::Int(count)))
        }

        "extdeps_embedded_policy_literal_count_for_qualified_name" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_embedded_policy_literal_count_for_qualified_name requires a QualifiedName"
                    .to_string(),
            })?;
            let count = crate::extdeps_shape_transport_policy_project::embedded_policy_literal_count_for_qualified_name(
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
                crate::extdeps_shape_transport_policy_project::module_source_nickname_literal_count_for_qualified_name(
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
                crate::extdeps_shape_transport_policy_project::policy_leak_count_for_qualified_name(
                    module,
                );
            Ok(Some(Value::Int(count)))
        }

        "extdeps_transport_fusion_fork_count_for_qualified_name" => {
            let module = positional.first().ok_or_else(|| InterpError::TypeError {
                msg: "extdeps_transport_fusion_fork_count_for_qualified_name requires a QualifiedName"
                    .to_string(),
            })?;
            let count = crate::extdeps_shape_transport_policy_project::transport_fusion_fork_count_for_qualified_name(
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
                crate::extdeps_shape_transport_policy_project::gist_create_declares_filename_input_for_qualified_name(
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
                crate::extdeps_shape_transport_policy_project::gist_create_files_keyed_by_filename_placeholder_for_qualified_name(
                    module,
                ),
            )))
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
        // A top-level named function (`Value::Fn`) is a first-class `fn(...) -> ...` value: the
        // typechecker admits it wherever a closure type is expected, so a higher-order argument
        // bound to a bare named function must be applied here too — positionally, mirroring the
        // Record/Variant field-fn dispatch in `eval_method_call`. Without this arm the program
        // typechecks but the evaluator rejects it at apply time ("expected closure, got Fn"),
        // a fail-open spec-without-execution gap (DESIGN.md §5).
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
    F: FnOnce(&RrbVector<Value>, &Value, &Rc<Env>, &InterpContext) -> InterpResult<Value>,
{
    let items = expect_list(&receiver, method_name)?;
    let closure = args.first().ok_or_else(|| InterpError::TypeError {
        msg: format!("{} requires a closure argument", method_name),
    })?;
    f(&items, closure, env, ctx)
}

thread_local! {
    /// Flattening counters for `free_monoid_to_vec` (phase-0 measurement,
    /// ctrl#1533). Thread-local rather than context-scoped because the
    /// chokepoint also fires inside `Value::eq` (`impl PartialEq`), which has
    /// no context — and unlike the caches #4644 scoped, these are two
    /// fixed-size integers: no keys, no keepalive, no growth.
    static FLATTEN_COUNTERS: std::cell::Cell<(u64, u64)> = const { std::cell::Cell::new((0, 0)) };
}

/// Snapshot of (calls, items materialized) by `free_monoid_to_vec` on this
/// thread since process start. Sample before/after an evaluation for a delta.
pub fn flatten_counters_snapshot() -> (u64, u64) {
    FLATTEN_COUNTERS.with(|c| c.get())
}

fn record_flatten(items: usize) {
    FLATTEN_COUNTERS.with(|c| {
        let (calls, total) = c.get();
        c.set((calls + 1, total + items as u64));
    });
}

// ---------------------------------------------------------------------------
// Per-instruction eval profiler
// ---------------------------------------------------------------------------
// The v2 lens CI gate runs Bool witnesses whose eval — AFTER resolve — can run
// for seconds (the cost/complexity lenses fold symbolically over an AST) even
// though the witness returns a trivial Bool. "Where does that eval time go?"
// could not be answered with an external profiler: CI runners ship no
// perf/valgrind and run under `perf_event_paranoid` > 2, so sampling is blocked.
//
// Instead the interpreter self-reports an instruction histogram. Because every
// (sub-)expression evaluation routes through `eval_expr`, instrumenting that one
// function captures the whole tree walk. For each `ExprData` variant we record:
//   - COUNT: how many nodes of that variant were evaluated, and
//   - SELF nanoseconds: gross frame time minus the time charged to child
//     `eval_expr` frames (so a hot `ExprCall` that mostly waits on its callee's
//     body shows the body's cost under the body's variants, not under the call).
// Counts pinpoint the hot instruction; self-time confirms where the wall goes.
//
// All of it is gated behind GUNBC_INTERP_PROFILE=1 (cached per-thread on first
// read) so the default eval path pays nothing — see `eval_expr`'s fast path.

/// Number of `ExprData` variants. Keep in sync with `expr_variant_index`.
pub const EXPR_VARIANT_COUNT: usize = 22;

/// Stable index for an `ExprData` variant (0..EXPR_VARIANT_COUNT). The order is
/// internal to the profiler — `expr_variant_name` is the only public mapping.
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

/// Human-readable name for a profiler variant index (see `expr_variant_index`).
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
    /// Per-variant eval counts and self-nanoseconds for the active thread.
    static EVAL_COUNTS: RefCell<[u64; EXPR_VARIANT_COUNT]> =
        const { RefCell::new([0; EXPR_VARIANT_COUNT]) };
    static EVAL_SELF_NANOS: RefCell<[u128; EXPR_VARIANT_COUNT]> =
        const { RefCell::new([0; EXPR_VARIANT_COUNT]) };
    /// Content-hash cache-subject key for the active witness eval (Phase-0 keystone).
    static ACTIVE_SUBJECT: RefCell<Option<String>> = const { RefCell::new(None) };
    /// Self-time nanoseconds accumulated per cache-subject during the active profile window.
    static SUBJECT_SELF_NANOS: RefCell<HashMap<String, u128>> = RefCell::new(HashMap::new());
    /// Gross nanoseconds charged by the child `eval_expr` frames of the frame
    /// currently running — read by a parent frame to subtract its children's
    /// time. Reset to 0 on frame entry, restored (plus the frame's own gross)
    /// on exit. See `eval_expr`.
    static CHILD_NANOS: Cell<u128> = const { Cell::new(0) };
    /// Cached GUNBC_INTERP_PROFILE flag (None until first read).
    static PROFILE_FLAG: Cell<Option<bool>> = const { Cell::new(None) };
}

/// True when GUNBC_INTERP_PROFILE=1 (read once per thread, then cached).
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

/// SCAFFOLD — v1 proving handler mirror of `compute_fabric.PerformanceReceipt` (§3 peripheral).
/// P5 receipt: `eval_measurement_purity_test.rs` (verdict unchanged under measurement).
/// dissolve-on: v2 eval fold binds `RealizationMeasureEffect::ObserveElapsedAtSubject`;
/// delete this struct + thread-local tap (`docs/plans/realization-measurement-loop.md` Phase 0 table).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerformanceReceipt {
    pub subject_key: String,
    pub work_shape: String,
    pub wall_nanos: u128,
    pub eval_self_nanos: u128,
    pub sample_count: u64,
}

/// Snapshot of per-subject eval self-time nanoseconds for the active thread.
pub fn subject_self_nanos_snapshot() -> HashMap<String, u128> {
    SUBJECT_SELF_NANOS.with(|m| m.borrow().clone())
}

/// Set the active cache-subject key for eval self-time attribution (`eval_expr` tap).
pub fn eval_subject_set(subject_key: String) {
    ACTIVE_SUBJECT.with(|s| *s.borrow_mut() = Some(subject_key));
}

/// Clear the active cache-subject key.
pub fn eval_subject_clear() {
    ACTIVE_SUBJECT.with(|s| *s.borrow_mut() = None);
}

/// Zero subject self-time counters and the child-time accumulator.
pub fn eval_subject_timing_reset() {
    SUBJECT_SELF_NANOS.with(|m| m.borrow_mut().clear());
    CHILD_NANOS.set(0);
}

/// Build a PerformanceReceipt from witness boundary timing + eval tap self-time.
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

/// A snapshot of the per-instruction eval profile (per-variant count + self-ns).
#[derive(Clone)]
pub struct EvalProfile {
    pub counts: [u64; EXPR_VARIANT_COUNT],
    pub self_nanos: [u128; EXPR_VARIANT_COUNT],
}

/// Snapshot the current thread's per-instruction profile. Pair with
/// `eval_profile_reset` around one witness eval for a per-witness breakdown.
pub fn eval_profile_snapshot() -> EvalProfile {
    EvalProfile {
        counts: EVAL_COUNTS.with(|c| *c.borrow()),
        self_nanos: EVAL_SELF_NANOS.with(|c| *c.borrow()),
    }
}

/// Zero the current thread's per-instruction profile (counts + self-ns + the
/// child-time accumulator) and subject self-time map.
pub fn eval_profile_reset() {
    EVAL_COUNTS.with(|c| *c.borrow_mut() = [0; EXPR_VARIANT_COUNT]);
    EVAL_SELF_NANOS.with(|c| *c.borrow_mut() = [0; EXPR_VARIANT_COUNT]);
    eval_subject_timing_reset();
    CHILD_NANOS.set(0);
}

/// Flatten a FreeMonoid value into a Vec, or None if `val` is neither a list nor a
/// well-formed Empty/Cons chain. Lists build as Value::List; FreeMonoid values constructed
/// via Cons/Empty (e.g. list_snoc_item chains in Node.children) are Variant chains. The list
/// builtins (fold/map/filter/foreach) accept either. Fails closed (P3): a non-list value —
/// including Null and a Cons with a missing/non-list `tail` — returns None so the caller
/// raises a type error rather than fabricating an empty/partial list.
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
            // `type String = FreeMonoid<Char>` and `type Char = Nat` (std/text.dag): a String IS
            // its codepoint sequence. Explode to Value::Int codepoints so list ops and `==` treat
            // it as a FreeMonoid<Char> (matches the Value::Str Empty/Cons pattern bridge above).
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
                    match (fields.get(&head_sym), fields.get(&tail_sym)) {
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

/// Bridge a list-denoting value (native List, FreeMonoid Variant chain, or
/// Str-as-FreeMonoid<Char>) to the list carrier, honoring the B1 alias
/// transparency through ONE code path. A native List hands back its carrier
/// handle without copying; chain forms flatten through free_monoid_to_vec —
/// the second tuple element reports how many items that flatten copied (the
/// phase-0 *_items_copied accounting). Non-list values return None.
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
            .map(|ctx| fields.contains_key(&ctx.sym("lookup")))
            .unwrap_or(false),
        _ => false,
    }
}

/// Low-level map key probe for Option-C dual-dispatch (ctrl#1476 B6).
///
/// Native `Value::Map`: present -> value, missing -> `Null`.
/// Record-form `Map { lookup: fn }`: invoke the `lookup` field (closure or fn).
/// The pattern bridge (`Null` -> `None`, value -> `Some`) lets std `map_get`
/// (v2.std.collection, `Outcome<Optional<V>>`) wrap the raw probe; `map_get` is
/// intentionally absent from `eval_builtin` (B-LOOKUP-1).
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
            let lookup = fields
                .get(&lookup_sym)
                .ok_or_else(|| InterpError::TypeError {
                    msg: format!("raw_map_lookup expects Map, got {}", map.type_label()),
                })?;
            match lookup {
                Value::Closure { .. } => apply_closure(lookup, &[key.clone()], env, ctx),
                Value::Fn { node } => {
                    let named = vec![(None, key.clone())];
                    call_function(ctx, node, &named, env)
                }
                _ => Err(InterpError::TypeError {
                    msg: "Map.lookup field is not callable".to_string(),
                }),
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
