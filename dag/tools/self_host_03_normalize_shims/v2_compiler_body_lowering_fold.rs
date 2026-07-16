use std::rc::Rc;

use crate::v2_std_collection::Optional;
use crate::v2_std_diagnostic::{diag_none, Diagnostics, Outcome};
use crate::v2_std_node::{Node, Symbol};

pub fn body_lower_is_deferred_lower_emitted(emitted: Symbol) -> bool {
    emitted == "dag_surface_fn_body"
        || emitted == "dag_surface_pattern"
        || emitted == "dag_surface_match_arm"
        || emitted == "dag_surface_match_arm_stmt_body"
}

pub fn body_lower_finish(
    _n: Rc<Node>,
    folded: Rc<Node>,
    cd: Rc<Diagnostics>,
) -> Rc<Outcome<Rc<Node>>> {
    Rc::new(Outcome::Accepted {
        value: folded,
        diagnostics: cd,
    })
}

// Emitted normalize passes `Diagnostics::None` (import shadow) where `Rc<Diagnostics>` is
// required — rustc rejects verbatim entry without emitter wrap fix. Witness transport records
// this as emit-surface debt; shim keeps the faithful signature for the cd.clone() call sites.
pub fn body_lower_finish_none_folded(
    n: Rc<Node>,
    folded: Rc<Node>,
) -> Rc<Outcome<Rc<Node>>> {
    body_lower_finish(n, folded, diag_none())
}
