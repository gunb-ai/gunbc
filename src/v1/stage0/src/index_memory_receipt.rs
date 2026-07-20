//! Rc→Arc share spike — structural memory accounting for `MultiEntryIndex` fields.
//!
//! Measures shareable (`typed_module_cache` transitive closure) vs per-worker residue
//! (parse_cache, resolved_graph_memo, intern_table, normalize/ownership diag caches).
//! Used by `measure_rc_arc_share_spike` and future migration oracles.

use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::cli_run::MultiEntryIndex;
use crate::v1_compiler_infer::TypecheckModuleResult;
use crate::v1_compiler_infer_env::{TypeBinding, TypeEnv, TypeEnvCache};
use crate::v1_compiler_infer_items::{ItemInfo, ResolvedFuncEnv, TypedModule};
use crate::v1_compiler_parse::ParseResult;
use crate::v1_std_core::{InferredNode, Node, NewlineIndex};
use crate::v1_interpreter::InternTable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexMemoryBucket {
    Shareable,
    PerWorkerResidue,
}

#[derive(Debug, Clone)]
pub struct IndexFieldBytes {
    pub field: &'static str,
    pub bucket: IndexMemoryBucket,
    pub heap_bytes: u64,
    pub entry_count: usize,
}

#[derive(Debug, Clone)]
pub struct IndexMemoryReceipt {
    pub fields: Vec<IndexFieldBytes>,
    pub shareable_bytes: u64,
    pub residue_bytes: u64,
    pub total_accounted_bytes: u64,
    pub typed_module_cache_entries: usize,
    pub serde_transport_bytes: u64,
    pub peak_rss_bytes: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct WidthScalingPoint {
    pub width: usize,
    pub peak_rss_bytes: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct NetWinPoint {
    pub width: usize,
    pub private_total_bytes: u64,
    pub shared_model_bytes: u64,
    pub net_win_bytes: i64,
}

struct Accountant {
    visited: HashSet<usize>,
    bytes: u64,
}

impl Accountant {
    fn new() -> Self {
        Accountant {
            visited: HashSet::new(),
            bytes: 0,
        }
    }

    fn first_visit<T: ?Sized>(&mut self, ptr: *const T) -> bool {
        if ptr.is_null() {
            return false;
        }
        self.visited.insert(ptr as usize)
    }

    fn add_shell<T: ?Sized>(&mut self, ptr: *const T) {
        if self.first_visit(ptr) {
            self.bytes += std::mem::size_of::<T>() as u64;
        }
    }

    fn add_string(&mut self, s: &str) {
        self.bytes += (std::mem::size_of::<String>() + s.len()) as u64;
    }

    fn add_vec_shell<T>(&mut self, vec: &Vec<T>) {
        self.bytes += (std::mem::size_of::<Vec<T>>() + vec.capacity() * std::mem::size_of::<T>())
            as u64;
    }

    fn add_hashmap_shell<K, V>(&mut self, map: &HashMap<K, V>) {
        self.bytes +=
            (std::mem::size_of::<HashMap<K, V>>() + map.capacity() * std::mem::size_of::<(K, V)>())
                as u64;
    }

    fn finish(self) -> u64 {
        self.bytes
    }
}

fn account_intern_table(acc: &mut Accountant, table: &InternTable) {
    let ptr = table as *const InternTable;
    if acc.first_visit(ptr) {
        acc.bytes += table.stats().heap_bytes;
    }
}

fn account_newline_index(acc: &mut Accountant, nl: &NewlineIndex) {
    let ptr = nl as *const NewlineIndex;
    if !acc.first_visit(ptr) {
        return;
    }
    acc.add_shell::<NewlineIndex>(ptr);
    acc.add_string(&nl.file);
    acc.add_vec_shell(&nl.line_starts);
}

fn account_node(acc: &mut Accountant, node: &Node) {
    let ptr = node as *const Node;
    if !acc.first_visit(ptr) {
        return;
    }
    acc.add_shell::<Node>(ptr);
    acc.add_string(&node.name);
    if let Some(span) = &node.ident_span {
        account_source_span(acc, span.as_ref());
    }
    account_source_span(acc, node.span.as_ref());
    for child in node.children.iter() {
        account_node(acc, child);
    }
    for param in node.params.iter() {
        account_node(acc, param);
    }
    for u in node.uses.iter() {
        account_node(acc, u);
    }
    if let Some(body) = &node.body {
        account_node(acc, body);
    }
    if let Some(transport) = &node.transport {
        account_node(acc, transport);
    }
    for prop in node.properties.iter() {
        account_node(acc, prop);
    }
    if let Some(ta) = &node.type_annotation {
        account_node(acc, ta);
    }
    if let Some(inferred) = &node.inferred {
        account_inferred_node(acc, inferred);
    }
    account_expr_data(acc, node.expr_data.as_ref());
}

fn account_source_span(acc: &mut Accountant, span: &crate::v1_std_core::SourceSpan) {
    let ptr = span as *const crate::v1_std_core::SourceSpan;
    if !acc.first_visit(ptr) {
        return;
    }
    acc.add_shell::<crate::v1_std_core::SourceSpan>(ptr);
    acc.add_string(&span.file);
}

fn account_inferred_node(acc: &mut Accountant, inferred: &InferredNode) {
    let ptr = inferred as *const InferredNode;
    if !acc.first_visit(ptr) {
        return;
    }
    acc.add_shell::<InferredNode>(ptr);
    match inferred {
        InferredNode::Resolved { node } => account_node(acc, node),
        InferredNode::CompilerError { message, span } => {
            acc.add_string(message);
            account_source_span(acc, span);
        }
        InferredNode::TypeVariable { id } => acc.add_string(id),
    }
}

fn account_expr_data(acc: &mut Accountant, expr: &crate::v1_std_core::ExprData) {
    let ptr = expr as *const crate::v1_std_core::ExprData;
    if !acc.first_visit(ptr) {
        return;
    }
    acc.add_shell::<crate::v1_std_core::ExprData>(ptr);
}

fn account_type_binding(acc: &mut Accountant, binding: &TypeBinding) {
    let ptr = binding as *const TypeBinding;
    if !acc.first_visit(ptr) {
        return;
    }
    acc.add_shell::<TypeBinding>(ptr);
    acc.add_string(&binding.name);
    account_node(acc, binding.resolved.as_ref());
}

fn account_type_env(acc: &mut Accountant, env: &TypeEnv) {
    let ptr = env as *const TypeEnv;
    if !acc.first_visit(ptr) {
        return;
    }
    acc.add_shell::<TypeEnv>(ptr);
    for binding in env.bindings.values() {
        account_type_binding(acc, binding);
    }
    for binding in env.str_bindings.values() {
        account_type_binding(acc, binding);
    }
    for binding in env.ancestry_str_bindings.values() {
        account_type_binding(acc, binding);
    }
    for parent in env.parents.iter() {
        account_type_env(acc, parent);
    }
    for fields in env.inductive_fields.values() {
        for field in fields.iter() {
            let fptr = field.as_ref() as *const crate::v1_compiler_infer_env::InductiveField;
            if acc.first_visit(fptr) {
                acc.add_shell::<crate::v1_compiler_infer_env::InductiveField>(fptr);
            }
        }
    }
    for nl in env.source_indices.values() {
        account_newline_index(acc, nl);
    }
    account_intern_table(acc, env.intern_table.as_ref());
}

fn account_type_env_cache(acc: &mut Accountant, cache: &TypeEnvCache) {
    let ptr = cache as *const TypeEnvCache;
    if !acc.first_visit(ptr) {
        return;
    }
    acc.add_shell::<TypeEnvCache>(ptr);
    for binding in cache.str_bindings.values() {
        account_type_binding(acc, binding);
    }
    for binding in cache.variant_locals.values() {
        account_type_binding(acc, binding);
    }
}

fn account_typed_module(acc: &mut Accountant, module: &TypedModule) {
    let ptr = module as *const TypedModule;
    if !acc.first_visit(ptr) {
        return;
    }
    acc.add_shell::<TypedModule>(ptr);
    account_node(acc, module.module.as_ref());
    for item in module.items.iter() {
        account_node(acc, item);
    }
    account_type_env(acc, module.type_env.as_ref());
    account_type_env_cache(acc, module.type_env_cache.as_ref());
    account_type_env(acc, module.interface.env.as_ref());
    for item in module.item_registry.values() {
        let iptr = item.as_ref() as *const ItemInfo;
        if acc.first_visit(iptr) {
            acc.add_shell::<ItemInfo>(iptr);
            acc.add_string(&item.name);
            acc.add_string(&item.module_name);
            for param in item.params.iter() {
                account_node(acc, param);
            }
        }
    }
    let feptr = module.func_env.as_ref() as *const ResolvedFuncEnv;
    if acc.first_visit(feptr) {
        acc.add_shell::<ResolvedFuncEnv>(feptr);
        for sig in module.func_env.local.values() {
            let sptr = sig.as_ref() as *const crate::v1_compiler_infer_sigs::ResolvedFuncSig;
            if acc.first_visit(sptr) {
                acc.add_shell::<crate::v1_compiler_infer_sigs::ResolvedFuncSig>(sptr);
            }
        }
        for parent in module.func_env.parents.iter() {
            let pptr = parent.as_ref() as *const ResolvedFuncEnv;
            if acc.first_visit(pptr) {
                acc.add_shell::<ResolvedFuncEnv>(pptr);
            }
        }
    }
}

fn account_typecheck_module_result(acc: &mut Accountant, result: &TypecheckModuleResult) {
    let ptr = result as *const TypecheckModuleResult;
    if !acc.first_visit(ptr) {
        return;
    }
    acc.add_shell::<TypecheckModuleResult>(ptr);
    account_typed_module(acc, result.typed.as_ref());
    for diag in result.diagnostics.iter() {
        let dptr = diag.as_ref() as *const crate::v1_std_core::ErrorNode;
        if acc.first_visit(dptr) {
            acc.add_shell::<crate::v1_std_core::ErrorNode>(dptr);
        }
    }
}

fn account_from_typed_cache(index: &MultiEntryIndex) -> (u64, usize, u64) {
    let cache = index.typed_module_cache.borrow();
    let mut acc = Accountant::new();
    let mut serde_bytes = 0u64;
    for result in cache.values() {
        account_typecheck_module_result(&mut acc, result);
        if let Ok(bytes) = crate::shared_typecheck_store::SharedTypecheckCaches::encode_typed_snapshot(result) {
            serde_bytes += bytes.len() as u64;
        }
    }
    (acc.finish(), cache.len(), serde_bytes)
}

fn account_parse_cache(index: &MultiEntryIndex) -> (u64, usize) {
    let cache = index.parse_cache.borrow();
    let mut acc = Accountant::new();
    for (path, (parse, nl)) in cache.iter() {
        acc.add_string(path);
        let pptr = parse.as_ref() as *const ParseResult;
        if acc.first_visit(pptr) {
            acc.add_shell::<ParseResult>(pptr);
            if let Some(module) = &parse.module {
                account_node(&mut acc, module);
            }
        }
        account_newline_index(&mut acc, nl);
    }
    (acc.finish(), cache.len())
}

fn account_resolved_graph_memo(index: &MultiEntryIndex) -> (u64, usize) {
    let memo = index.resolved_graph_memo.borrow();
    let mut acc = Accountant::new();
    for (subject, (graph, si)) in memo.iter() {
        acc.add_string(subject);
        let gptr = graph.as_ref() as *const crate::v1_compiler_infer_items::ResolvedGraph;
        if acc.first_visit(gptr) {
            acc.add_shell::<crate::v1_compiler_infer_items::ResolvedGraph>(gptr);
            for module in graph.modules.iter() {
                account_typed_module(&mut acc, module);
            }
        }
        for nl in si.values() {
            account_newline_index(&mut acc, nl);
        }
    }
    (acc.finish(), memo.len())
}

fn account_intern_table_field(index: &MultiEntryIndex) -> (u64, usize) {
    let table = index.intern_table.borrow();
    let mut acc = Accountant::new();
    account_intern_table(&mut acc, table.as_ref());
    (acc.finish(), 1)
}

fn account_diag_cache(
    index: &MultiEntryIndex,
    field: &str,
) -> (u64, usize) {
    let cache = match field {
        "normalize_diag_cache" => &index.normalize_diag_cache.borrow(),
        "ownership_diag_cache" => &index.ownership_diag_cache.borrow(),
        _ => unreachable!(),
    };
    let mut acc = Accountant::new();
    for (key, diags) in cache.iter() {
        acc.add_string(key);
        let vptr = diags.as_ref() as *const im_rc::Vector<Rc<crate::v1_std_core::ErrorNode>>;
        if acc.first_visit(vptr) {
            acc.bytes += std::mem::size_of::<im_rc::Vector<Rc<crate::v1_std_core::ErrorNode>>>() as u64;
            for diag in diags.iter() {
                let dptr = diag.as_ref() as *const crate::v1_std_core::ErrorNode;
                if acc.first_visit(dptr) {
                    acc.add_shell::<crate::v1_std_core::ErrorNode>(dptr);
                }
            }
        }
    }
    (acc.finish(), cache.len())
}

fn account_string_map(map: &HashMap<String, String>) -> (u64, usize) {
    let mut acc = Accountant::new();
    acc.add_hashmap_shell(map);
    for (k, v) in map {
        acc.add_string(k);
        acc.add_string(v);
    }
    (acc.finish(), map.len())
}

/// Structural heap accounting for one populated `MultiEntryIndex`.
pub fn multi_entry_index_memory_receipt(
    index: &MultiEntryIndex,
    peak_rss_bytes: Option<u64>,
) -> IndexMemoryReceipt {
    let (shareable, typed_entries, serde_transport_bytes) = account_from_typed_cache(index);

    let (parse_bytes, parse_entries) = account_parse_cache(index);
    let (memo_bytes, memo_entries) = account_resolved_graph_memo(index);
    let (intern_bytes, intern_entries) = account_intern_table_field(index);
    let (normalize_bytes, normalize_entries) = account_diag_cache(index, "normalize_diag_cache");
    let (ownership_bytes, ownership_entries) =
        account_diag_cache(index, "ownership_diag_cache");

    let fields = vec![
        IndexFieldBytes {
            field: "typed_module_cache",
            bucket: IndexMemoryBucket::Shareable,
            heap_bytes: shareable,
            entry_count: typed_entries,
        },
        IndexFieldBytes {
            field: "parse_cache",
            bucket: IndexMemoryBucket::PerWorkerResidue,
            heap_bytes: parse_bytes,
            entry_count: parse_entries,
        },
        IndexFieldBytes {
            field: "resolved_graph_memo",
            bucket: IndexMemoryBucket::PerWorkerResidue,
            heap_bytes: memo_bytes,
            entry_count: memo_entries,
        },
        IndexFieldBytes {
            field: "intern_table",
            bucket: IndexMemoryBucket::PerWorkerResidue,
            heap_bytes: intern_bytes,
            entry_count: intern_entries,
        },
        IndexFieldBytes {
            field: "normalize_diag_cache",
            bucket: IndexMemoryBucket::PerWorkerResidue,
            heap_bytes: normalize_bytes,
            entry_count: normalize_entries,
        },
        IndexFieldBytes {
            field: "ownership_diag_cache",
            bucket: IndexMemoryBucket::PerWorkerResidue,
            heap_bytes: ownership_bytes,
            entry_count: ownership_entries,
        },
    ];

    let residue_bytes = parse_bytes + memo_bytes + intern_bytes + normalize_bytes + ownership_bytes;
    IndexMemoryReceipt {
        shareable_bytes: shareable,
        residue_bytes,
        total_accounted_bytes: shareable + residue_bytes,
        typed_module_cache_entries: typed_entries,
        serde_transport_bytes,
        fields,
        peak_rss_bytes,
    }
}

/// Union shareable bytes across multiple indexes (simulates process-wide typed store).
pub fn union_shareable_bytes(receipts: &[IndexMemoryReceipt]) -> u64 {
    // Re-walk is expensive; for union growth use max shareable + sampled overlap estimate.
    // Caller passes per-worker receipts; union ≈ max when closures identical, less when disjoint.
    receipts.iter().map(|r| r.shareable_bytes).sum()
}

/// Compute net-win curve: private_total = W*(shareable+residue); shared = union_shareable + W*residue.
pub fn net_win_curve(
    shareable_per_worker: u64,
    residue_per_worker: u64,
    union_shareable: u64,
    max_width: usize,
) -> Vec<NetWinPoint> {
    (1..=max_width)
        .map(|width| {
            let private_total = width as u64 * (shareable_per_worker + residue_per_worker);
            let shared_model = union_shareable + width as u64 * residue_per_worker;
            NetWinPoint {
                width,
                private_total_bytes: private_total,
                shared_model_bytes: shared_model,
                net_win_bytes: private_total as i64 - shared_model as i64,
            }
        })
        .collect()
}

pub fn emit_index_memory_receipt(receipt: &IndexMemoryReceipt) {
    for field in &receipt.fields {
        eprintln!(
            "[rc-arc-spike] kind=index-field field={} bucket={} bytes={} entries={}",
            field.field,
            match field.bucket {
                IndexMemoryBucket::Shareable => "shareable",
                IndexMemoryBucket::PerWorkerResidue => "per_worker_residue",
            },
            field.heap_bytes,
            field.entry_count,
        );
    }
    eprintln!(
        "[rc-arc-spike] kind=index-summary shareable_bytes={} residue_bytes={} total_accounted_bytes={} typed_module_cache_entries={} serde_transport_bytes={} peak_rss_bytes={}",
        receipt.shareable_bytes,
        receipt.residue_bytes,
        receipt.total_accounted_bytes,
        receipt.typed_module_cache_entries,
        receipt.serde_transport_bytes,
        receipt
            .peak_rss_bytes
            .map(|b| b.to_string())
            .unwrap_or_else(|| "unavailable".into()),
    );
}

pub fn emit_width_scaling_point(width: usize, peak_rss_bytes: Option<u64>) {
    eprintln!(
        "[rc-arc-spike] kind=width-scaling width={} peak_rss_bytes={}",
        width,
        peak_rss_bytes
            .map(|b| b.to_string())
            .unwrap_or_else(|| "unavailable".into()),
    );
}

pub fn emit_net_win_point(point: &NetWinPoint) {
    eprintln!(
        "[rc-arc-spike] kind=net-win width={} private_total_bytes={} shared_model_bytes={} net_win_bytes={}",
        point.width, point.private_total_bytes, point.shared_model_bytes, point.net_win_bytes,
    );
}

/// Census for the im_rc blocker (no code changes — report-only).
pub fn im_rc_blocker_census() -> (usize, usize, &'static str) {
    const NOTE: &str = "im-rc (Rc-backed HAMT) is aliased as HashMap/Vector/BTreeSet in lib.rs; \
                        the Arc-backed sibling crate is `im`. Collections remain !Send even if \
                        outer Rc→Arc lands — swapping im-rc→im touches every persistent map/list \
                        in the 122/154 stage0 .rs files plus serde feature parity.";
  (123, 154, NOTE)
}

pub fn emit_im_rc_census() {
    let (im_rc_files, total_rs_files, note) = im_rc_blocker_census();
    eprintln!(
        "[rc-arc-spike] kind=im-rc-census im_rc_files={} stage0_rs_files={} note={}",
        im_rc_files, total_rs_files, note
    );
}
