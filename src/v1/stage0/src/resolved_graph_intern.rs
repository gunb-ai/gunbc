//! Cache-boundary content-addressed interning for the resolved-graph artifact
//! (Layer B of `docs/plans/resolved-graph-representation-minimization.md`).
//!
//! `ResolvedGraph` retains, per module, a fully-merged copy of every binding /
//! source-index / function it transitively sees (DESIGN §2 *duplicated* work):
//! measured 73% of a 233 MB witness artifact is duplicated import-closure
//! (`type_env` 54% + `func_env` 19%; bindings 8,015 stored / 434 distinct;
//! `source_indices` up to 59×). In RAM these are `Rc`-shared, but `serde`'s `rc`
//! feature does not preserve identity, so a naive cache round-trip both bloats
//! the artifact (211 MB JSON) and shatters the sharing on deserialize.
//!
//! This module is the §2 *horizontal* dedup at the cache seam: a
//! content-addressed pool of the heavy repeated sub-objects, with modules
//! referencing pool entries by index. Decode rebuilds exactly one `Rc` per pool
//! entry, so structural sharing is restored on load (smaller artifact AND lower
//! warm-cache-hit RAM). It is a faithful encoding — `decode(encode(g))` is value
//! identical to `g` — so the DESIGN §5 `warm==cold` purity oracle
//! (`cache_purity_oracle_test`) holds: the oracle compares graph *values*
//! (canonical re-serialization), and a value-faithful encoding cannot move them.
//!
//! Pools are keyed and ordered by a *canonical* (key-sorted) content hash:
//! `serde_json::to_value` builds `serde_json::Value::Object` as a sorted
//! `BTreeMap`, so two structurally-equal values that differ only in `HashMap`
//! iteration order hash identically and dedup to one pool entry. (This canonical
//! key is what an earlier non-canonical `serde_json::to_vec`-over-`HashMap`
//! pool hash lacked.) Pools are then sorted by that hash, making the on-disk
//! encoding a deterministic function of the graph value.

use std::collections::HashMap;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::v1_compiler_infer_emit_info::EmitGraphInfo;
use crate::v1_compiler_infer_env::{InductiveField, TypeBinding, TypeEnv};
use crate::v1_compiler_infer_items::{ItemInfo, ResolvedGraph, TypedModule};
use crate::v1_compiler_infer_sigs::{ResolvedFuncEnv, ResolvedFuncSig};
use crate::v1_rt;
use crate::v1_std_core::{ErrorNode, InternTable, NewlineIndex, Node};

type SourceIndices = HashMap<String, Rc<NewlineIndex>>;
type InductiveFields = HashMap<String, Rc<Vec<Rc<InductiveField>>>>;

/// Canonical, key-sorted content hash of any serializable value. `to_value`
/// materializes maps as sorted `BTreeMap`s, so the hash is invariant under
/// `HashMap` iteration order — the property the pool dedup depends on.
fn canonical_hash<T: Serialize>(value: &T) -> String {
    let canonical: serde_json::Value =
        serde_json::to_value(value).expect("intern: value is serializable");
    let bytes = serde_json::to_vec(&canonical).expect("intern: canonical value re-serializes");
    v1_rt::bytes_identity_hash(&bytes)
}

/// A content-addressed pool: distinct values keyed by canonical hash, assigned a
/// stable index in insertion order during the walk, then re-ordered by hash at
/// finish time so the emitted pool is a deterministic function of the graph.
struct Pool<T: Serialize + Clone> {
    by_hash: HashMap<String, u32>,
    items: Vec<(String, T)>,
}

impl<T: Serialize + Clone> Pool<T> {
    fn new() -> Self {
        Self {
            by_hash: HashMap::new(),
            items: Vec::new(),
        }
    }

    fn intern(&mut self, value: &T) -> u32 {
        let hash = canonical_hash(value);
        if let Some(&idx) = self.by_hash.get(&hash) {
            return idx;
        }
        let idx = self.items.len() as u32;
        self.items.push((hash.clone(), value.clone()));
        self.by_hash.insert(hash, idx);
        idx
    }

    /// Sort the pool by content hash and return (sorted values, remap[old]->new).
    fn finish(self) -> (Vec<T>, Vec<u32>) {
        let mut order: Vec<u32> = (0..self.items.len() as u32).collect();
        order.sort_by(|&a, &b| self.items[a as usize].0.cmp(&self.items[b as usize].0));
        let mut remap = vec![0u32; self.items.len()];
        for (new_idx, &old_idx) in order.iter().enumerate() {
            remap[old_idx as usize] = new_idx as u32;
        }
        let values = order
            .into_iter()
            .map(|old| self.items[old as usize].1.clone())
            .collect();
        (values, remap)
    }
}

#[derive(Serialize, Deserialize)]
pub struct InternedPayload {
    nl_pool: Vec<NewlineIndex>,
    si_pool: Vec<Vec<(String, u32)>>,
    indfields_pool: Vec<Vec<(String, Rc<Vec<Rc<InductiveField>>>)>>,
    intern_pool: Vec<InternTable>,
    binding_pool: Vec<TypeBinding>,
    funcsig_pool: Vec<ResolvedFuncSig>,
    modules: Vec<InternedModule>,
    item_registry: Rc<HashMap<String, Rc<ItemInfo>>>,
    diagnostics: Rc<Vec<Rc<ErrorNode>>>,
    emit_graph_info: Rc<EmitGraphInfo>,
    top_source_indices: u32,
}

#[derive(Serialize, Deserialize)]
struct InternedModule {
    module: Rc<Node>,
    items: Rc<Vec<Rc<Node>>>,
    type_env: InternedTypeEnv,
    func_env: Vec<(String, u32)>,
    item_registry: Rc<HashMap<String, Rc<ItemInfo>>>,
}

#[derive(Serialize, Deserialize)]
struct InternedTypeEnv {
    bindings: Vec<(i64, u32)>,
    recursive_types: Rc<Vec<i64>>,
    recursive_type_set: Rc<HashMap<i64, bool>>,
    inductive_fields: u32,
    source_indices: u32,
    intern_table: u32,
}

struct Encoder {
    nl: Pool<NewlineIndex>,
    si: Pool<Vec<(String, u32)>>,
    indfields: Pool<Vec<(String, Rc<Vec<Rc<InductiveField>>>)>>,
    intern: Pool<InternTable>,
    binding: Pool<TypeBinding>,
    funcsig: Pool<ResolvedFuncSig>,
}

impl Encoder {
    fn intern_source_indices(&mut self, si: &SourceIndices) -> u32 {
        let mut entries: Vec<(String, u32)> = si
            .iter()
            .map(|(k, v)| (k.clone(), self.nl.intern(v.as_ref())))
            .collect();
        entries.sort();
        self.si.intern(&entries)
    }

    fn intern_inductive_fields(&mut self, fields: &InductiveFields) -> u32 {
        let mut entries: Vec<(String, Rc<Vec<Rc<InductiveField>>>)> = fields
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect::<Vec<_>>();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        self.indfields.intern(&entries)
    }

    fn encode_type_env(&mut self, env: &TypeEnv) -> InternedTypeEnv {
        let mut bindings: Vec<(i64, u32)> = env
            .bindings
            .iter()
            .map(|(k, v)| (*k, self.binding.intern(v.as_ref())))
            .collect();
        bindings.sort();
        InternedTypeEnv {
            bindings,
            recursive_types: env.recursive_types.clone(),
            recursive_type_set: env.recursive_type_set.clone(),
            inductive_fields: self.intern_inductive_fields(&env.inductive_fields),
            source_indices: self.intern_source_indices(&env.source_indices),
            intern_table: self.intern.intern(env.intern_table.as_ref()),
        }
    }

    fn encode_func_env(&mut self, env: &ResolvedFuncEnv) -> Vec<(String, u32)> {
        let mut sigs: Vec<(String, u32)> = env
            .signatures
            .iter()
            .map(|(k, v)| (k.clone(), self.funcsig.intern(v.as_ref())))
            .collect();
        sigs.sort_by(|a, b| a.0.cmp(&b.0));
        sigs
    }

    fn encode_module(&mut self, m: &TypedModule) -> InternedModule {
        InternedModule {
            module: m.module.clone(),
            items: m.items.clone(),
            type_env: self.encode_type_env(&m.type_env),
            func_env: self.encode_func_env(&m.func_env),
            item_registry: m.item_registry.clone(),
        }
    }
}

/// Remap a `Vec<(K, u32)>`'s pool indices through a `finish()` remap table.
fn remap_kv<K: Clone>(entries: &mut [(K, u32)], remap: &[u32]) {
    for (_, idx) in entries.iter_mut() {
        *idx = remap[*idx as usize];
    }
}

pub fn encode(graph: &ResolvedGraph, source_indices: &SourceIndices) -> InternedPayload {
    let mut enc = Encoder {
        nl: Pool::new(),
        si: Pool::new(),
        indfields: Pool::new(),
        intern: Pool::new(),
        binding: Pool::new(),
        funcsig: Pool::new(),
    };

    let top_source_indices = enc.intern_source_indices(source_indices);
    let mut modules: Vec<InternedModule> =
        graph.modules.iter().map(|m| enc.encode_module(m)).collect();

    // `si` and `indfields` pools hold `nl` indices; finishing `nl` first lets us
    // remap them before the `si`/`indfields` pools are themselves finished.
    let (nl_pool, nl_remap) = enc.nl.finish();
    for (_, entries) in enc.si.items.iter_mut() {
        remap_kv(entries, &nl_remap);
    }
    let (si_pool, si_remap) = enc.si.finish();
    let (indfields_pool, indfields_remap) = enc.indfields.finish();
    let (intern_pool, intern_remap) = enc.intern.finish();
    let (binding_pool, binding_remap) = enc.binding.finish();
    let (funcsig_pool, funcsig_remap) = enc.funcsig.finish();

    for m in modules.iter_mut() {
        remap_kv(&mut m.type_env.bindings, &binding_remap);
        m.type_env.inductive_fields = indfields_remap[m.type_env.inductive_fields as usize];
        m.type_env.source_indices = si_remap[m.type_env.source_indices as usize];
        m.type_env.intern_table = intern_remap[m.type_env.intern_table as usize];
        remap_kv(&mut m.func_env, &funcsig_remap);
    }

    InternedPayload {
        nl_pool,
        si_pool,
        indfields_pool,
        intern_pool,
        binding_pool,
        funcsig_pool,
        modules,
        item_registry: graph.item_registry.clone(),
        diagnostics: graph.diagnostics.clone(),
        emit_graph_info: graph.emit_graph_info.clone(),
        top_source_indices: si_remap[top_source_indices as usize],
    }
}

fn rebuild_source_indices(
    entries: &[(String, u32)],
    nl_pool: &[Rc<NewlineIndex>],
) -> SourceIndices {
    entries
        .iter()
        .map(|(k, idx)| (k.clone(), nl_pool[*idx as usize].clone()))
        .collect()
}

pub fn decode(payload: &InternedPayload) -> (ResolvedGraph, Rc<SourceIndices>) {
    // One `Rc` per pool entry — this is where structural sharing is restored.
    let nl_pool: Vec<Rc<NewlineIndex>> =
        payload.nl_pool.iter().map(|v| Rc::new(v.clone())).collect();
    let si_pool: Vec<Rc<SourceIndices>> = payload
        .si_pool
        .iter()
        .map(|entries| Rc::new(rebuild_source_indices(entries, &nl_pool)))
        .collect();
    let indfields_pool: Vec<Rc<InductiveFields>> = payload
        .indfields_pool
        .iter()
        .map(|entries| Rc::new(entries.iter().cloned().collect::<InductiveFields>()))
        .collect();
    let intern_pool: Vec<Rc<InternTable>> = payload
        .intern_pool
        .iter()
        .map(|v| Rc::new(v.clone()))
        .collect();
    let binding_pool: Vec<Rc<TypeBinding>> = payload
        .binding_pool
        .iter()
        .map(|v| Rc::new(v.clone()))
        .collect();
    let funcsig_pool: Vec<Rc<ResolvedFuncSig>> = payload
        .funcsig_pool
        .iter()
        .map(|v| Rc::new(v.clone()))
        .collect();

    let modules: Vec<Rc<TypedModule>> = payload
        .modules
        .iter()
        .map(|m| {
            let bindings: HashMap<i64, Rc<TypeBinding>> = m
                .type_env
                .bindings
                .iter()
                .map(|(k, idx)| (*k, binding_pool[*idx as usize].clone()))
                .collect();
            let type_env = TypeEnv {
                bindings: Rc::new(bindings),
                recursive_types: m.type_env.recursive_types.clone(),
                recursive_type_set: m.type_env.recursive_type_set.clone(),
                inductive_fields: indfields_pool[m.type_env.inductive_fields as usize].clone(),
                source_indices: si_pool[m.type_env.source_indices as usize].clone(),
                intern_table: intern_pool[m.type_env.intern_table as usize].clone(),
            };
            let signatures: HashMap<String, Rc<ResolvedFuncSig>> = m
                .func_env
                .iter()
                .map(|(k, idx)| (k.clone(), funcsig_pool[*idx as usize].clone()))
                .collect();
            Rc::new(TypedModule {
                module: m.module.clone(),
                items: m.items.clone(),
                type_env: Rc::new(type_env),
                func_env: Rc::new(ResolvedFuncEnv {
                    signatures: Rc::new(signatures),
                }),
                item_registry: m.item_registry.clone(),
            })
        })
        .collect();

    let graph = ResolvedGraph {
        modules: Rc::new(modules),
        item_registry: payload.item_registry.clone(),
        diagnostics: payload.diagnostics.clone(),
        emit_graph_info: payload.emit_graph_info.clone(),
    };
    // Return the pooled `Rc` directly so the top-level source-indices share with
    // the per-module type-envs that reference the same pool entry.
    let top_si = si_pool[payload.top_source_indices as usize].clone();
    (graph, top_si)
}
