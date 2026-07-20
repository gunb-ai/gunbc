//! Rc→Arc share spike — structural memory accounting for `MultiEntryIndex` fields.
//!
//! Measures shareable (`typed_module_cache` transitive closure) vs per-worker residue
//! (parse_cache, resolved_graph_memo, intern_table, normalize/ownership diag caches).
//! Used by `measure_rc_arc_share_spike` and future migration oracles.

use crate::cli_run::MultiEntryIndex;
use crate::v1_compiler_infer::TypecheckModuleResult;
use crate::v1_compiler_infer_env::{TypeBinding, TypeEnv, TypeEnvCache};
use crate::v1_compiler_infer_items::{ItemInfo, ResolvedFuncEnv, TypedModule};
use crate::v1_compiler_parse::ParseResult;
use crate::v1_std_core::{InferredNode, InternTable, NewlineIndex, Node};
use std::collections::{HashMap as StdHashMap, HashSet};
use std::rc::Rc;

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
        self.visited.insert(ptr as *const () as usize)
    }

    fn add_shell<T>(&mut self, ptr: *const T) {
        if self.first_visit(ptr) {
            self.bytes += std::mem::size_of::<T>() as u64;
        }
    }

    fn add_rc_shell<T>(&mut self, rc: &Rc<T>) {
        let ptr = Rc::as_ptr(rc);
        if self.first_visit(ptr) {
            self.bytes += std::mem::size_of::<Rc<T>>() as u64;
        }
    }

    fn add_string(&mut self, s: &str) {
        self.bytes += (std::mem::size_of::<String>() + s.len()) as u64;
    }

    fn add_vec_shell<T>(&mut self, len: usize) {
        self.bytes += (std::mem::size_of::<Vec<T>>() + len * std::mem::size_of::<T>()) as u64;
    }

    fn add_hashmap_shell<K, V>(&mut self, map: &StdHashMap<K, V>) {
        self.bytes += (std::mem::size_of::<StdHashMap<K, V>>()
            + map.capacity() * std::mem::size_of::<(K, V)>()) as u64;
    }

    fn add_im_hashmap_shell<K, V>(&mut self, map: &im_rc::HashMap<K, V>) {
        self.bytes += (std::mem::size_of::<im_rc::HashMap<K, V>>()
            + map.len() * std::mem::size_of::<(K, V)>()) as u64;
    }

    fn finish(self) -> u64 {
        self.bytes
    }
}

fn account_intern_table(acc: &mut Accountant, table: &InternTable) {
    let ptr = table as *const InternTable;
    if !acc.first_visit(ptr) {
        return;
    }
    acc.add_shell::<InternTable>(ptr);
    let sptr = Rc::as_ptr(&table.strings) as usize;
    if acc.first_visit(sptr as *const Vec<String>) {
        acc.add_vec_shell::<String>(table.strings.len());
        for s in table.strings.iter() {
            acc.add_string(s);
        }
    }
    let iptr = Rc::as_ptr(&table.index) as usize;
    if acc.first_visit(iptr as *const im_rc::HashMap<String, i64>) {
        acc.add_im_hashmap_shell(table.index.as_ref());
    }
}

fn account_newline_index(acc: &mut Accountant, nl: &NewlineIndex) {
    let ptr = nl as *const NewlineIndex;
    if !acc.first_visit(ptr) {
        return;
    }
    acc.add_shell::<NewlineIndex>(ptr);
    acc.add_string(&nl.file);
    acc.add_vec_shell::<i64>(nl.offsets.len());
    acc.add_vec_shell::<i64>(nl.char_codes.len());
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

fn account_from_typed_cache(
    cache: &StdHashMap<String, Rc<crate::v1_compiler_infer::TypecheckModuleResult>>,
) -> (u64, usize, u64) {
    // Shareable bucket: map shell + keys only. Peak RSS is the honest total (§2.2);
    // serde transport is NOT extrapolated here — sample encode OOMs / misleads.
    let mut acc = Accountant::new();
    acc.add_hashmap_shell(cache);
    for key in cache.keys() {
        acc.add_string(key);
    }
    (acc.finish(), cache.len(), 0)
}

fn account_parse_cache_shallow(
    cache: &StdHashMap<String, (Rc<ParseResult>, Rc<NewlineIndex>)>,
) -> (u64, usize) {
    let mut acc = Accountant::new();
    acc.add_hashmap_shell(cache);
    for (path, _) in cache.iter() {
        acc.add_string(path);
    }
    (acc.finish(), cache.len())
}

fn account_resolved_graph_memo_shallow(
    memo: &im_rc::HashMap<
        String,
        (
            Rc<crate::v1_compiler_infer_items::ResolvedGraph>,
            Rc<im_rc::HashMap<String, Rc<NewlineIndex>>>,
        ),
    >,
) -> (u64, usize) {
    let mut acc = Accountant::new();
    acc.add_im_hashmap_shell(memo);
    for subject in memo.keys() {
        acc.add_string(subject);
    }
    (acc.finish(), memo.len())
}

fn account_intern_table_shallow(table: &Rc<InternTable>) -> (u64, usize) {
    let mut acc = Accountant::new();
    let ptr = table.as_ref() as *const InternTable;
    if acc.first_visit(ptr) {
        acc.add_shell::<InternTable>(ptr);
        acc.bytes += (table.strings.len() * 24) as u64;
    }
    (acc.finish(), table.strings.len())
}

fn account_diag_cache_map_shallow(
    cache: &StdHashMap<String, Rc<im_rc::Vector<Rc<crate::v1_std_core::ErrorNode>>>>,
) -> (u64, usize) {
    let mut acc = Accountant::new();
    acc.add_hashmap_shell(cache);
    for (key, diags) in cache.iter() {
        acc.add_string(key);
        acc.bytes +=
            diags.len() as u64 * std::mem::size_of::<Rc<crate::v1_std_core::ErrorNode>>() as u64;
    }
    (acc.finish(), cache.len())
}

fn account_string_map(map: &StdHashMap<String, String>) -> (u64, usize) {
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
    let (typed_cache, parse_cache, memo, intern, normalize_cache, ownership_cache) =
        index.memory_receipt_snapshot();

    let (shareable, typed_entries, serde_transport_bytes) = account_from_typed_cache(&typed_cache);

    let (parse_bytes, parse_entries) = account_parse_cache_shallow(&parse_cache);
    let (memo_bytes, memo_entries) = account_resolved_graph_memo_shallow(&memo);
    let (intern_bytes, intern_entries) = account_intern_table_shallow(&intern);
    let (normalize_bytes, normalize_entries) = account_diag_cache_map_shallow(&normalize_cache);
    let (ownership_bytes, ownership_entries) = account_diag_cache_map_shallow(&ownership_cache);

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

pub fn emit_host_metadata(
    hostname: &str,
    mem_available_kb: Option<u64>,
    cgroup_max_bytes: Option<u64>,
) {
    eprintln!(
        "[rc-arc-spike] kind=host-metadata hostname={} mem_available_kb={} cgroup_max_bytes={}",
        hostname,
        mem_available_kb
            .map(|b| b.to_string())
            .unwrap_or_else(|| "unavailable".into()),
        cgroup_max_bytes
            .map(|b| b.to_string())
            .unwrap_or_else(|| "unlimited".into()),
    );
}

pub fn emit_timing_point(label: &str, elapsed_ms: u128, peak_rss_bytes: Option<u64>) {
    eprintln!(
        "[rc-arc-spike] kind=timing label={label} elapsed_ms={elapsed_ms} peak_rss_bytes={}",
        peak_rss_bytes
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
    let stage0_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut total_rs = 0usize;
    let mut im_rc_files = 0usize;
    fn walk(dir: &std::path::Path, total_rs: &mut usize, im_rc_files: &mut usize) {
        let Ok(read) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in read.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, total_rs, im_rc_files);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                *total_rs += 1;
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if content.contains("im_rc::") || content.contains("use im_rc") {
                        *im_rc_files += 1;
                    }
                }
            }
        }
    }
    walk(&stage0_root, &mut total_rs, &mut im_rc_files);
    const NOTE: &str = "im-rc (Rc-backed HAMT) is aliased as HashMap/Vector/BTreeSet in lib.rs; \
                        the Arc-backed sibling crate is `im`. Collections remain !Send even if \
                        outer Rc→Arc lands — swapping im-rc→im touches every persistent map/list \
                        in stage0 .rs files plus serde feature parity.";
    (im_rc_files, total_rs, NOTE)
}

pub fn emit_im_rc_census() {
    let (im_rc_files, total_rs_files, note) = im_rc_blocker_census();
    eprintln!(
        "[rc-arc-spike] kind=im-rc-census im_rc_files={} stage0_rs_files={} note={}",
        im_rc_files, total_rs_files, note
    );
}
