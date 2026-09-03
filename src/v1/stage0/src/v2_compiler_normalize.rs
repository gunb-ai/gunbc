// Seed realization for v2.compiler.normalize (Wave 2 parallel flip).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.normalize
// is emitted-only and the behavioral harness is modeled (sn_scaffold_dissolution_trigger).

// CLIPPY ROSTER -- 3 finding(s) this module trips today, listed one lint per line with
// its count. Until this commit the generated crate root allowed `clippy::all` plus six
// rustc groups on behalf of every module under it, so `cargo clippy --all-targets -- -D
// warnings` decided nothing here; the root now excuses only the generated modules it
// speaks for (v1.compiler.emit_rust generated_rust_lint_relaxations), and this is what
// that leaves visible. The list is MONOTONE NON-INCREASING: a name leaves when its last
// site is repaired, and a lint not named below reds the build, which is the whole point.
#![allow(
    unused_imports,  // 3
)]

use im::{vector as vec, Vector as Vec};
use std::rc::Rc;

use crate::usv_pilot_v2_std_algebra::list_snoc_item;
use crate::usv_pilot_v2_std_node::{node_rebuild, Behavior, Connective, Node, NodeKind};

fn well_formed(_n: Rc<Node>) -> bool {
    true
}

#[derive(Debug, Clone, PartialEq)]
pub enum Outcome<T> {
    Accepted { value: T, diagnostics: () },
    Rejected { diagnostics: () },
}

pub type NormalizedTree = Rc<Node>;
pub type ParseTree = Rc<Node>;

pub fn normalize_fold_init(n: Rc<Node>) -> Rc<Outcome<Rc<Node>>> {
    Rc::new(Outcome::Accepted {
        value: node_rebuild(n, Rc::new(vec![])),
        diagnostics: (),
    })
}

pub fn normalize(parse_tree: ParseTree) -> Rc<Outcome<NormalizedTree>> {
    let init = normalize_fold_init(parse_tree);
    match init.as_ref() {
        Outcome::Accepted { value, .. } => {
            if well_formed(value.clone()) {
                Rc::new(Outcome::Accepted {
                    value: value.clone(),
                    diagnostics: (),
                })
            } else {
                Rc::new(Outcome::Rejected { diagnostics: () })
            }
        }
        Outcome::Rejected { diagnostics } => Rc::new(Outcome::Rejected {
            diagnostics: *diagnostics,
        }),
    }
}
