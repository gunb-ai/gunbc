// v2_interpreter.rs — Tree-walking interpreter for .dag programs.
// Hand-written infrastructure (same category as parser, tokenizer, v2_rt).
// I-1: pure evaluation. I-2: shell service dispatch.

use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use crate::std_syntax::BinOp;
use crate::std_syntax::LiteralValue;
use crate::v2_compiler_emit::{extract_string_interp_parts, has_mock_prefix};
use crate::v2_compiler_infer_items::{ItemInfo, ItemKind, ResolvedGraph, TypedModule};
use crate::v2_rt;
use crate::v2_rt::{
    rc_empty_set as empty_set, rc_set_insert as set_insert, rc_set_union as set_union, set_contains,
};
use crate::v2_std_core::{
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
    MatchPattern,
    MethodSemantics,
    NewlineIndex,
    Node,
    SourceSpan,
    StringPart,
    UnaryOpKind,
    VarBindingKind,
};

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
fn hash_fields_commutative(fields: &HashMap<String, Value>) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    let mut acc: u64 = 0;
    for (name, val) in fields.iter() {
        let mut fh = DefaultHasher::new();
        name.hash(&mut fh);
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
    List(Rc<Vec<Value>>),
    Map(Rc<HashMap<CanonKey, Value>>),
    /// String membership sets (`Set<String>` in .dag).
    Set(Rc<BTreeSet<String>>),
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
    /// Reference to a module-level `fn` / `func` item (first-class function value).
    Fn {
        node: Rc<Node>,
    },
    Unit,
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
                write!(f, "{} {{", type_name)?;
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, " {}: {}", k, v)?;
                }
                write!(f, " }}")
            }
            Value::Variant {
                type_name: _,
                variant_name,
                fields,
            } => {
                if fields.is_empty() {
                    write!(f, "{}", variant_name)
                } else {
                    write!(f, "{} {{", variant_name)?;
                    for (i, (k, v)) in fields.iter().enumerate() {
                        if i > 0 {
                            write!(f, ",")?;
                        }
                        write!(f, " {}: {}", k, v)?;
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
    bindings: HashMap<String, Value>,
    parent: Option<Rc<Env>>,
}

impl Env {
    pub fn empty() -> Rc<Self> {
        Rc::new(Env {
            bindings: HashMap::new(),
            parent: None,
        })
    }

    pub fn extend(parent: &Rc<Env>, bindings: HashMap<String, Value>) -> Rc<Self> {
        Rc::new(Env {
            bindings,
            parent: Some(parent.clone()),
        })
    }

    pub fn with_binding(parent: &Rc<Env>, name: String, value: Value) -> Rc<Self> {
        let mut bindings = HashMap::new();
        bindings.insert(name, value);
        Rc::new(Env {
            bindings,
            parent: Some(parent.clone()),
        })
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
}

impl fmt::Display for MutationCounters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rows: [(&str, u64, u64); 5] = [
            ("map_insert", self.map_insert_calls, self.map_insert_entries_copied),
            ("map_merge", self.map_merge_calls, self.map_merge_entries_copied),
            ("list_push", self.list_push_calls, self.list_push_items_copied),
            ("list_concat", self.list_concat_calls, self.list_concat_items_copied),
            ("set_insert", self.set_insert_calls, self.set_insert_items_copied),
        ];
        for (name, calls, copied) in rows {
            writeln!(
                f,
                "  {:<12} {:>12} calls  {:>16} entries copied  (avg {:.1}/call)",
                name,
                calls,
                copied,
                if calls == 0 { 0.0 } else { copied as f64 / calls as f64 }
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
    let mut bytes = (env.bindings.len() * std::mem::size_of::<(String, Value)>()) as u64;
    for name in env.bindings.keys() {
        bytes += name.len() as u64;
    }
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
    fields: &Rc<HashMap<String, Value>>,
    visited: &mut std::collections::HashSet<usize>,
    acc: &mut MemoryAccounting,
) {
    if !accounting_first_visit(Rc::as_ptr(fields) as usize, label, visited, acc) {
        return;
    }
    let mut bytes = (fields.len() * std::mem::size_of::<(String, Value)>()) as u64;
    for key in fields.keys() {
        bytes += key.len() as u64;
    }
    acc.add_unique(label, bytes);
    for value in fields.values() {
        account_value(value, visited, acc);
    }
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
        Value::Record { type_name, fields } => {
            acc.variant(label).heap_bytes += type_name.len() as u64;
            acc.total_heap_bytes += type_name.len() as u64;
            account_named_fields(label, fields, visited, acc);
        }
        Value::Variant {
            type_name,
            variant_name,
            fields,
        } => {
            let names = (type_name.len() + variant_name.len()) as u64;
            acc.variant(label).heap_bytes += names;
            acc.total_heap_bytes += names;
            account_named_fields(label, fields, visited, acc);
        }
        // The AST body is program text (shared, graph-owned), not value heap;
        // only the captured environment chain is retained value memory.
        Value::Closure { params, env, body: _ } => {
            let mut bytes = (params.len() * std::mem::size_of::<String>()) as u64;
            for p in params {
                bytes += p.len() as u64;
            }
            acc.add_unique(label, bytes);
            account_env(env, visited, acc);
        }
        Value::Fn { .. } => {}
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
    /// Dry-run mode: use mock responses instead of executing services.
    pub dry_run: bool,
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
    /// Phase-0 measurement counters (ctrl#1533): copy work done by the
    /// copy-on-update collection primitives, scoped to this context like the
    /// caches above. Always on — a few integer adds next to O(n) clones.
    mutation_counters: std::cell::RefCell<MutationCounters>,
}

impl InterpContext {
    /// Snapshot of the copy-work counters accumulated by evaluations in this
    /// context (phase-0 measurement, ctrl#1533).
    pub fn mutation_counters_snapshot(&self) -> MutationCounters {
        self.mutation_counters.borrow().clone()
    }

    /// Sharing-aware byte accounting over everything this context retains
    /// (`data_cache`, pure-call memo results and keepalive pins) plus any
    /// `extra_roots` (e.g. final claim values). One visited set across all
    /// roots, so cross-root structural sharing is counted once.
    pub fn account_retained_memory(&self, extra_roots: &[&Value]) -> MemoryAccounting {
        let mut visited = std::collections::HashSet::new();
        let mut acc = MemoryAccounting::default();
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
        dry_run: bool,
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
            dry_run,
            data_cache: std::cell::RefCell::new(HashMap::new()),
            pure_call_memo: std::cell::RefCell::new(PureCallMemo::default()),
            mutation_counters: std::cell::RefCell::new(MutationCounters::default()),
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
    run_with_options(graph, source_indices, entry_fn, false, true)
}

pub fn run_with_options(
    graph: &ResolvedGraph,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
    entry_fn: &str,
    dry_run: bool,
    eager_data_env: bool,
) -> InterpResult<Value> {
    let ctx = InterpContext::new(graph, source_indices, dry_run);
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
    // Find the entry function
    let item_node = ctx
        .lookup_fn(entry_fn)
        .ok_or_else(|| InterpError::NoMainFunction)?
        .clone();

    // Default: evaluate all `data` items up front (legacy `dag run` behavior).
    // Claim-run mode skips this — src/v4 has hundreds of TestClaim data graphs;
    // witnesses pull only what they need via lazy data-item resolution in eval_var.
    let env = if eager_data_env {
        build_initial_env(ctx)?
    } else {
        Env::empty()
    };

    // Call the entry function with no arguments
    call_function(ctx, &item_node, &[], &env)
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
                bindings.insert(name.clone(), val.clone());
            } else if positional_idx < param_names.len() {
                bindings.insert(param_names[positional_idx].clone(), val.clone());
                positional_idx += 1;
            }
        }
    }

    // Fill default values for unbound parameters
    for param in fn_node.params.iter() {
        let pname = authored_name_at(ctx.si(), param.clone());
        if !bindings.contains_key(&pname) {
            if let Some(default_node) = param_node_default_value(param.clone()) {
                let default_val = eval_expr(&default_node, env, ctx)?;
                bindings.insert(pname, default_val);
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
    stacker::maybe_grow(64 * 1024, 2 * 1024 * 1024, || {
        eval_expr_inner(node, env, ctx)
    })
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
            Ok(Value::List(Rc::new(items)))
        }

        ExprData::ExprLambda => {
            let param_names: Vec<String> = lambda_param_names_at(node.clone(), si)
                .iter()
                .cloned()
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
        let record_concat = |copied: usize| {
            let mut counters = ctx.mutation_counters.borrow_mut();
            counters.list_concat_calls += 1;
            counters.list_concat_items_copied += copied as u64;
        };
        match (&left, &right) {
            (l, Value::Str(s)) => {
                if let Some(mut result) = free_monoid_to_vec(l) {
                    record_concat(result.len());
                    result.push(Value::Str(s.clone()));
                    return Ok(Value::List(Rc::new(result)));
                }
            }
            (Value::Str(s), r) => {
                if let Some(result) = free_monoid_to_vec(r) {
                    record_concat(result.len());
                    let mut out = vec![Value::Str(s.clone())];
                    out.extend(result);
                    return Ok(Value::List(Rc::new(out)));
                }
            }
            _ => {
                if let (Some(mut a), Some(b)) =
                    (free_monoid_to_vec(&left), free_monoid_to_vec(&right))
                {
                    record_concat(a.len() + b.len());
                    a.extend(b);
                    return Ok(Value::List(Rc::new(a)));
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
    let new_env = Env::with_binding(env, name, val);
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

fn native_map_absent_diagnostic_value() -> Value {
    let mut anchor_fields = HashMap::new();
    anchor_fields.insert("at".to_string(), Value::Str("map_lookup_port".to_string()));

    let mut locus_fields = HashMap::new();
    locus_fields.insert(
        "anchor".to_string(),
        Value::Record {
            type_name: "LocusAnchor".to_string(),
            fields: Rc::new(anchor_fields),
        },
    );

    let mut correction_fields = HashMap::new();
    correction_fields.insert(
        "reason".to_string(),
        Value::Variant {
            type_name: "NoCorrectionReason".to_string(),
            variant_name: "ExternalContractUnknown".to_string(),
            fields: Rc::new(HashMap::new()),
        },
    );

    let mut diagnostic_fields = HashMap::new();
    diagnostic_fields.insert(
        "reason".to_string(),
        Value::Str("map_key_absent".to_string()),
    );
    diagnostic_fields.insert(
        "at".to_string(),
        Value::Variant {
            type_name: "Locus".to_string(),
            variant_name: "PortLocus".to_string(),
            fields: Rc::new(locus_fields),
        },
    );
    diagnostic_fields.insert(
        "correction".to_string(),
        Value::Variant {
            type_name: "Correction".to_string(),
            variant_name: "Unavailable".to_string(),
            fields: Rc::new(correction_fields),
        },
    );

    Value::Record {
        type_name: "Diagnostic".to_string(),
        fields: Rc::new(diagnostic_fields),
    }
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
                    // `v2_rt::Witness::Holds { value: <self> }`. A native `Value::Map` lookup returns the raw
                    // stored value, and a `-> Witness`-annotated `.lookup` consumer (e.g.
                    // parse_table_lookup / lookup_table) matches `Holds`/`Violates`, so a
                    // present coproduct value must satisfy the `Holds` arm. Same absent-arm
                    // exclusions (`Violates`/`Absent`/`none` stay absent -> the `Violates` arm);
                    // a genuine `Holds` is handled by nominal matching below.
                    if name == "Holds"
                        && parent_enum.as_deref() == Some("Witness")
                        && variant_name != "Holds"
                        && variant_name != "Violates"
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
                        && variant_name != "Present"
                        && variant_name != "Absent"
                    {
                        let mut bindings = HashMap::new();
                        for fb in field_bindings.iter() {
                            let fb_pat = field_binding_pattern(fb.clone());
                            let sub_bindings = match_pattern(&fb_pat, value, ctx)?;
                            bindings.extend(sub_bindings);
                        }
                        return Some(bindings);
                    }
                    if variant_name != name {
                        return None;
                    }
                    let mut bindings = HashMap::new();
                    for fb in field_bindings.iter() {
                        let field_name =
                            field_binding_name_at(fb.clone(), ctx.source_indices.clone());
                        let fb_pat = field_binding_pattern(fb.clone());
                        let field_val = fields.get(&field_name).cloned().unwrap_or(Value::Null);
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
                    if type_name != name {
                        return None;
                    }
                    let mut bindings = HashMap::new();
                    for fb in field_bindings.iter() {
                        let field_name =
                            field_binding_name_at(fb.clone(), ctx.source_indices.clone());
                        let fb_pat = field_binding_pattern(fb.clone());
                        let field_val = fields.get(&field_name).cloned().unwrap_or(Value::Null);
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
                            let tail = Value::List(Rc::new(items[1..].to_vec()));
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
                // the std contract is `Map.lookup: fn(K) -> Witness<V>` (v4.std.collection)
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
                            "diagnostic" => native_map_absent_diagnostic_value(),
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
                // `v2_rt::Witness::Holds { value: <self> }`. `Null` (a miss) does not match `Holds` — it
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

    // Check for built-in runtime functions
    if let Some(result) = eval_builtin(&func_name, &args, ctx)? {
        return Ok(result);
    }

    // Look up user-defined function: module item wins over env-bound Value::Fn (see eval_var).
    let fn_node = if let Some(node) = ctx.lookup_fn(&func_name) {
        node.clone()
    } else {
        match env.lookup(&func_name) {
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
        if let Some(field_val) = fields.get(&method_name) {
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
                Ok(items) => Ok(items.first().cloned().unwrap_or(Value::Null)),
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
    match value {
        Value::Record { type_name, fields } => {
            fields
                .get(field)
                .cloned()
                .ok_or_else(|| InterpError::NoSuchField {
                    type_name: type_name.clone(),
                    field: field.to_string(),
                })
        }
        Value::Variant {
            type_name, fields, ..
        } => fields
            .get(field)
            .cloned()
            .ok_or_else(|| InterpError::NoSuchField {
                type_name: type_name.clone(),
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

fn eval_string_interp(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
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

fn eval_cast(node: &Rc<Node>, env: &Rc<Env>, ctx: &InterpContext) -> InterpResult<Value> {
    let val = eval_expr(&cast_expr(node.clone()), env, ctx)?;
    let target_node = cast_target(node.clone());
    let target_name = authored_name_at(ctx.si(), target_node);

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
        let iter_env = Env::with_binding(env, var_name.clone(), item.clone());
        results.push(eval_expr(&body_node, &iter_env, ctx)?);
    }
    Ok(Value::List(Rc::new(results)))
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
            Ok(Value::List(Rc::new(items[s..e].to_vec())))
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
                .map(|v| Value::List(Rc::new(v)))
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
                Ok(Value::List(Rc::new(result)))
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
                Ok(Value::List(Rc::new(result)))
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
                Ok(Value::List(Rc::new(
                    keyed.into_iter().map(|(_, v)| v).collect(),
                )))
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
                let mut result = items.to_vec();
                for arg in args {
                    // Non-list Str args append as one element, not char-exploded (ctrl#1476 B1).
                    if matches!(arg, Value::Str(_)) {
                        result.push(arg.clone());
                    } else {
                        match free_monoid_to_vec(arg) {
                            Some(other) => result.extend(other),
                            None => result.push(arg.clone()),
                        }
                    }
                }
                let mut counters = ctx.mutation_counters.borrow_mut();
                counters.list_concat_calls += 1;
                counters.list_concat_items_copied += items.len() as u64;
                drop(counters);
                return Ok(Value::List(Rc::new(result)));
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
            Ok(Value::List(Rc::new(
                items.iter().skip(n as usize).cloned().collect(),
            )))
        }

        "take" => {
            let items = expect_list(&receiver, "take")?;
            let n = expect_int(args.first(), "take")?;
            Ok(Value::List(Rc::new(
                items.iter().take(n as usize).cloned().collect(),
            )))
        }

        "enumerate" => {
            let items = expect_list(&receiver, "enumerate")?;
            let result: Vec<Value> = items
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    let mut fields = HashMap::new();
                    fields.insert("index".to_string(), Value::Int(i as i64));
                    fields.insert("value".to_string(), v.clone());
                    Value::Record {
                        type_name: "Pair".to_string(),
                        fields: Rc::new(fields),
                    }
                })
                .collect();
            Ok(Value::List(Rc::new(result)))
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
            counters.map_insert_entries_copied += m.len() as u64;
            drop(counters);
            let mut new_map = HashMap::clone(&m);
            new_map.insert(ck, val);
            Ok(Value::Map(Rc::new(new_map)))
        }

        "merge" => {
            let base = expect_map(&receiver, "merge")?;
            let overlay = expect_map(args.first().unwrap_or(&Value::Null), "merge")?;
            let mut counters = ctx.mutation_counters.borrow_mut();
            counters.map_merge_calls += 1;
            counters.map_merge_entries_copied += (base.len() + overlay.len()) as u64;
            drop(counters);
            let mut result = HashMap::clone(&base);
            for (k, v) in overlay.iter() {
                result.insert(k.clone(), v.clone());
            }
            Ok(Value::Map(Rc::new(result)))
        }

        "keys" => {
            let m = expect_map(&receiver, "keys")?;
            let keys: Vec<Value> = m.keys().map(|k| k.key.clone()).collect();
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
                let mut m = HashMap::new();
                for item in items.iter() {
                    let key = apply_closure(f, &[item.clone()], env, ctx)?;
                    let ck = CanonKey::new(key).ok_or_else(|| InterpError::TypeError {
                        msg: "index_by key is not a valid map key (closure/fn/NaN)".to_string(),
                    })?;
                    m.insert(ck, item.clone());
                }
                Ok(Value::Map(Rc::new(m)))
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
            bindings.insert(name.clone(), val.clone());
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
        if !bindings.contains_key(&name) {
            if positional_idx < positional_args.len() {
                bindings.insert(name, positional_args[positional_idx].clone());
                positional_idx += 1;
            }
        }
    }

    // Third pass: fill defaults for any remaining unbound params
    for param in op_node.params.iter() {
        let name = param_node_name_at(param.clone(), ctx.si());
        if !bindings.contains_key(&name) {
            if let Some(default_node) = param_node_default_value(param.clone()) {
                let default_val = eval_expr(&default_node, env, ctx)?;
                bindings.insert(name, default_val);
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
                Value::List(Rc::new(lines))
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
        fields.insert(field_name, value);
    }

    // Return as record with the type name
    Ok(Value::Record {
        type_name: authored_name_at(ctx.si(), op_node.clone()),
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
            substitute_template(&path_str, param_env)
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
    let (auth_header_name, auth_token) = resolve_auth(service_node, transport, &si);

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
                Some(value_to_json(&body_val)?)
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

    let token = env_var_name.and_then(|var| std::env::var(&var).ok());
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
fn substitute_template(template: &str, env: &Rc<Env>) -> String {
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
            if let Some(val) = env.lookup(&var_name) {
                result.push_str(&format!("{}", val));
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
        Value::Record { fields, .. } | Value::Variant { fields, .. } => {
            let mut obj = serde_json::Map::new();
            for (k, v) in fields.iter() {
                if matches!(v, Value::Null) {
                    continue;
                }
                obj.insert(k.clone(), value_to_json(v)?);
            }
            serde_json::Value::Object(obj)
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
        let field_name = authored_name_at(ctx.si(), child.clone());
        fields.insert(field_name, Value::Str(text.to_string()));
    }
    Ok(Value::Record {
        type_name: authored_name_at(ctx.si(), op_node.clone()),
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
        fields.insert(first_field, json_to_value(json));
        return Ok(Value::Record {
            type_name: authored_name_at(ctx.si(), op_node.clone()),
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
        fields.insert(field_name, val);
    }

    Ok(Value::Record {
        type_name: authored_name_at(ctx.si(), op_node.clone()),
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
            let fields: HashMap<CanonKey, Value> = obj
                .iter()
                .filter_map(|(k, v)| {
                    CanonKey::new(Value::Str(k.clone())).map(|ck| (ck, json_to_value(v)))
                })
                .collect();
            Value::Map(Rc::new(fields))
        }
    }
}

/// Evaluate mock_response from an operation's properties for dry-run mode.
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
    // No mock response — return Unit
    Ok(Value::Unit)
}

// ---------------------------------------------------------------------------
// Built-in functions (v2_rt equivalents)
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

        // `discriminant(v)` reifies a coproduct/record value's own constructor name as a
        // `Symbol` (Symbol values are interned strings — see eval_var's `data X: Symbol = X`
        // idiom at Value::Str). This is the single intrinsic that dissolves the hand-written
        // `fn ..._discriminant(v) -> Symbol { match v { Ctor{..} => ctor_tag ... } }` bridges
        // that shadow a coproduct's arm-set with a parallel `data ctor_tag: Symbol` vocabulary.
        // The constructor's own name IS the discriminant; no per-type code is required.
        "discriminant" => match positional.first() {
            Some(Value::Variant { variant_name, .. }) => Ok(Some(Value::Str(variant_name.clone()))),
            Some(Value::Record { type_name, .. }) => Ok(Some(Value::Str(type_name.clone()))),
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
            let record_concat = |copied: usize| {
                let mut counters = ctx.mutation_counters.borrow_mut();
                counters.list_concat_calls += 1;
                counters.list_concat_items_copied += copied as u64;
            };
            match positional.as_slice() {
                [a, b] => match (a, b) {
                    (l, Value::Str(s)) => match free_monoid_to_vec(l) {
                        Some(mut result) => {
                            record_concat(result.len());
                            result.push(Value::Str(s.clone()));
                            Ok(Some(Value::List(Rc::new(result))))
                        }
                        None => Ok(None),
                    },
                    (Value::Str(s), r) => match free_monoid_to_vec(r) {
                        Some(result) => {
                            record_concat(result.len());
                            let mut out = vec![Value::Str(s.clone())];
                            out.extend(result);
                            Ok(Some(Value::List(Rc::new(out))))
                        }
                        None => Ok(None),
                    },
                    _ => match (free_monoid_to_vec(a), free_monoid_to_vec(b)) {
                        (Some(mut a_items), Some(b_items)) => {
                            record_concat(a_items.len() + b_items.len());
                            a_items.extend(b_items);
                            Ok(Some(Value::List(Rc::new(a_items))))
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
                    Ok(Some(Value::List(Rc::new(r))))
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
            Ok(Some(Value::Str(v2_rt::substring(&s, start, end))))
        }

        "char_at" => {
            let s = expect_str(positional.first().copied(), "char_at")?;
            let pos = expect_int(positional.get(1).copied(), "char_at pos")?;
            Ok(Some(Value::Str(v2_rt::char_at(&s, pos))))
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
            [list_val, item] => match free_monoid_to_vec(list_val) {
                Some(items) => {
                    let mut counters = ctx.mutation_counters.borrow_mut();
                    counters.list_push_calls += 1;
                    counters.list_push_items_copied += items.len() as u64;
                    drop(counters);
                    let mut result = items;
                    result.push((*item).clone());
                    Ok(Some(Value::List(Rc::new(result))))
                }
                None => Ok(None),
            },
            _ => Ok(None),
        },

        "list_concat" => match positional.as_slice() {
            [a, b] if matches!(a, Value::Str(_)) || matches!(b, Value::Str(_)) => Ok(None),
            [a, b] => match (free_monoid_to_vec(a), free_monoid_to_vec(b)) {
                (Some(a_items), Some(b_items)) => {
                    let mut counters = ctx.mutation_counters.borrow_mut();
                    counters.list_concat_calls += 1;
                    counters.list_concat_items_copied += (a_items.len() + b_items.len()) as u64;
                    drop(counters);
                    let mut result = a_items;
                    result.extend(b_items);
                    Ok(Some(Value::List(Rc::new(result))))
                }
                _ => Ok(None),
            },
            _ => Ok(None),
        },

        "empty_map" => Ok(Some(Value::Map(Rc::new(HashMap::new())))),

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
                counters.set_insert_calls += 1;
                counters.set_insert_items_copied += (a.len() + b.len()) as u64;
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
                    let mut counters = ctx.mutation_counters.borrow_mut();
                    counters.map_insert_calls += 1;
                    counters.map_insert_entries_copied += m.len() as u64;
                    drop(counters);
                    let mut result = HashMap::clone(m);
                    result.insert(ck, (*v).clone());
                    Ok(Some(Value::Map(Rc::new(result))))
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
        // std `map_get` (v4.std.collection, Outcome<Optional<V>>) wraps that probe into the
        // canonical Optional surface. `map_get` is NOT handled here on purpose: the builtin
        // arm previously SHADOWED the typed std map_get (eval_builtin wins over user fns at
        // eval_call), so `map_get(...)` returned the RAW value and any `match { Accepted; Rejected }`
        // consumer crashed non-exhaustively (B-LOOKUP-1). Dropping `map_get` here routes it to the
        // typed v4.std.collection authority. [List=FreeMonoid/Option-alias recurrence: the bridge
        // is honored per-operation (matching, ==, zip_eq, lookup) rather than once at the
        // representation — tracked for the post-R2 representation-level dissolution.]
        "lookup" => match positional.as_slice() {
            [map, key] => Ok(Some(raw_map_lookup(map, key, &Env::empty(), ctx)?)),
            _ => Ok(None),
        },

        "map_keys" => match positional.first() {
            Some(Value::Map(m)) => {
                let keys: Vec<Value> = m.keys().map(|k| k.key.clone()).collect();
                Ok(Some(Value::List(Rc::new(keys))))
            }
            _ => Ok(None),
        },

        "map_values" => match positional.first() {
            Some(Value::Map(m)) => {
                let vals: Vec<Value> = m.values().cloned().collect();
                Ok(Some(Value::List(Rc::new(vals))))
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

        "map_merge" => match positional.as_slice() {
            [Value::Map(base), Value::Map(overlay)] => {
                let mut result = HashMap::clone(&base);
                for (k, v) in overlay.iter() {
                    result.insert(k.clone(), v.clone());
                }
                Ok(Some(Value::Map(Rc::new(result))))
            }
            _ => Ok(None),
        },

        "str_eq" => match positional.as_slice() {
            [Value::Str(a), Value::Str(b)] => Ok(Some(Value::Bool(a == b))),
            _ => Ok(None),
        },

        "atom_identity_hash" => match positional.as_slice() {
            [Value::Str(s)] => Ok(Some(Value::Str(v2_rt::atom_identity_hash(s.clone())))),
            _ => Err(InterpError::TypeError {
                msg: "atom_identity_hash requires exactly one string argument".to_string(),
            }),
        },

        "hash_combine" => match positional.as_slice() {
            [Value::Str(a), Value::Str(b)] if positional.len() == 2 => {
                if !v2_rt::is_hash_digest(a) || !v2_rt::is_hash_digest(b) {
                    return Err(InterpError::TypeError {
                        msg: "hash_combine requires exactly two Hash arguments".to_string(),
                    });
                }
                Ok(Some(Value::Str(v2_rt::hash_combine(a.clone(), b.clone()))))
            }
            _ => Err(InterpError::TypeError {
                msg: "hash_combine requires exactly two Hash arguments".to_string(),
            }),
        },

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

/// Flatten a FreeMonoid value into a Vec, or None if `val` is neither a list nor a
/// well-formed Empty/Cons chain. Lists build as Value::List; FreeMonoid values constructed
/// via Cons/Empty (e.g. list_snoc_item chains in Node.children) are Variant chains. The list
/// builtins (fold/map/filter/foreach) accept either. Fails closed (P3): a non-list value —
/// including Null and a Cons with a missing/non-list `tail` — returns None so the caller
/// raises a type error rather than fabricating an empty/partial list.
fn free_monoid_to_vec(val: &Value) -> Option<Vec<Value>> {
    let mut out = Vec::new();
    let mut cur = val.clone();
    loop {
        match &cur {
            Value::List(items) => {
                out.extend(items.iter().cloned());
                return Some(out);
            }
            // `type String = FreeMonoid<Char>` and `type Char = Nat` (std/text.dag): a String IS
            // its codepoint sequence. Explode to Value::Int codepoints so list ops and `==` treat
            // it as a FreeMonoid<Char> (matches the Value::Str Empty/Cons pattern bridge above).
            Value::Str(s) => {
                out.extend(s.chars().map(char_value));
                return Some(out);
            }
            Value::Variant {
                variant_name,
                fields,
                ..
            } => match variant_name.as_str() {
                "Empty" => return Some(out),
                "Cons" => match (fields.get("head"), fields.get("tail")) {
                    (Some(head), Some(tail)) => {
                        out.push(head.clone());
                        cur = tail.clone();
                    }
                    _ => return None,
                },
                _ => return None,
            },
            _ => return None,
        }
    }
}

fn expect_list(val: &Value, context: &str) -> InterpResult<Rc<Vec<Value>>> {
    match val {
        Value::List(items) => Ok(items.clone()),
        _ => match free_monoid_to_vec(val) {
            Some(items) => Ok(Rc::new(items)),
            None => Err(InterpError::TypeError {
                msg: format!("{} expects a list, got {}", context, val.type_label()),
            }),
        },
    }
}

fn is_map_lookup_receiver(val: &Value) -> bool {
    match val {
        Value::Map(_) => true,
        Value::Record { fields, .. } | Value::Variant { fields, .. } => {
            fields.contains_key("lookup")
        }
        _ => false,
    }
}

/// Low-level map key probe for Option-C dual-dispatch (ctrl#1476 B6).
///
/// Native `Value::Map`: present -> value, missing -> `Null`.
/// Record-form `Map { lookup: fn }`: invoke the `lookup` field (closure or fn).
/// The pattern bridge (`Null` -> `None`, value -> `Some`) lets std `map_get`
/// (v4.std.collection, `Outcome<Optional<V>>`) wrap the raw probe; `map_get` is
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
            let lookup = fields.get("lookup").ok_or_else(|| InterpError::TypeError {
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

fn expect_map(val: &Value, context: &str) -> InterpResult<Rc<HashMap<CanonKey, Value>>> {
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
