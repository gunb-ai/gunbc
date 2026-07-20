// Seed realization for v2.compiler.normalize (Wave 2 parallel flip).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.normalize
// is emitted-only and the behavioral harness is modeled (sn_scaffold_dissolution_trigger).

use im::{vector as vec, Vector as Vec};
use std::sync::Arc;

use crate::usv_pilot_v2_std_algebra::list_snoc_item;
use crate::usv_pilot_v2_std_node::{node_rebuild, Behavior, Connective, Node, NodeKind};

fn well_formed(_n: Arc<Node>) -> bool {
    true
}

#[derive(Debug, Clone, PartialEq)]
pub enum Outcome<T> {
    Accepted { value: T, diagnostics: () },
    Rejected { diagnostics: () },
}

pub type NormalizedTree = Arc<Node>;
pub type ParseTree = Arc<Node>;

pub fn normalize_fold_init(n: Arc<Node>) -> Arc<Outcome<Arc<Node>>> {
    Arc::new(Outcome::Accepted {
        value: node_rebuild(n, Arc::new(vec![])),
        diagnostics: (),
    })
}

pub fn normalize(parse_tree: ParseTree) -> Arc<Outcome<NormalizedTree>> {
    let init = normalize_fold_init(parse_tree);
    match init.as_ref() {
        Outcome::Accepted { value, .. } => {
            if well_formed(value.clone()) {
                Arc::new(Outcome::Accepted {
                    value: value.clone(),
                    diagnostics: (),
                })
            } else {
                Arc::new(Outcome::Rejected { diagnostics: () })
            }
        }
        Outcome::Rejected { diagnostics } => Arc::new(Outcome::Rejected {
            diagnostics: *diagnostics,
        }),
    }
}
