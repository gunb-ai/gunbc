//! Scoped pool-corpus duplicate bare-coproduct index for `eval_decl_facts` marshaling.
//!
//! `decl_facts` marshals against a parse-only pool walk while `InterpContext.modules`
//! may be only the witness entry closure. Duplicate bare type names across pool modules
//! (e.g. fixture `SharedBareArm` in two modules) are invisible to `ctx.modules` scans.

use std::cell::RefCell;
use std::rc::Rc;

use im::HashMap;

use crate::v1_compiler_infer_items::ItemKind;
use crate::v1_std_core::{authored_name_at, Connective, NewlineIndex, Node};

#[derive(Clone)]
pub struct PoolCoproductDupEntry {
    pub module_path: String,
    pub type_name: String,
    pub item: Rc<Node>,
    pub source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
}

#[derive(Clone, Default)]
struct PoolCoproductDupIndex {
    by_bare_name: HashMap<String, Vec<PoolCoproductDupEntry>>,
}

thread_local! {
    static POOL_COPRODUCT_DUP_INDEX: RefCell<Option<PoolCoproductDupIndex>> =
        const { RefCell::new(None) };
}

fn bare_symbol_tail(name: &str) -> &str {
    name.rsplit_once('.').map(|(_, tail)| tail).unwrap_or(name)
}

pub fn set_pool_coproduct_dup_index(entries: Vec<PoolCoproductDupEntry>) {
    let mut by_bare_name: HashMap<String, Vec<PoolCoproductDupEntry>> = HashMap::new();
    for entry in entries {
        let bare = bare_symbol_tail(&entry.type_name);
        if bare.is_empty() {
            continue;
        }
        by_bare_name
            .entry(bare.to_string())
            .or_default()
            .push(entry);
    }
    POOL_COPRODUCT_DUP_INDEX.with(|cell| {
        *cell.borrow_mut() = Some(PoolCoproductDupIndex { by_bare_name });
    });
}

pub fn clear_pool_coproduct_dup_index() {
    POOL_COPRODUCT_DUP_INDEX.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

pub fn pool_duplicate_bare_coproduct_groups() -> Vec<Vec<PoolCoproductDupEntry>> {
    POOL_COPRODUCT_DUP_INDEX.with(|cell| {
        cell.borrow()
            .as_ref()
            .map(|index| {
                index
                    .by_bare_name
                    .values()
                    .filter(|entries| entries.len() >= 2)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    })
}

pub fn pool_coproduct_dup_entry_from_decl_fact_raw(
    qualified_name: &str,
    name: &str,
    kind: ItemKind,
    node: Rc<Node>,
    source_indices: Rc<HashMap<String, Rc<NewlineIndex>>>,
) -> Option<PoolCoproductDupEntry> {
    if kind != ItemKind::TypeItem || node.connective != Connective::Disj {
        return None;
    }
    let type_name = authored_name_at(source_indices.clone(), node.clone());
    if type_name.is_empty() {
        return None;
    }
    let suffix = format!(".{name}");
    let module_path = qualified_name
        .strip_suffix(&suffix)
        .unwrap_or(qualified_name);
    Some(PoolCoproductDupEntry {
        module_path: module_path.to_string(),
        type_name,
        item: node,
        source_indices,
    })
}
