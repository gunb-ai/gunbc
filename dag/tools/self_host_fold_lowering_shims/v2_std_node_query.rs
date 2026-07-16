use std::rc::Rc;

use crate::v2_std_diagnostic::{
    node_locus, outcome_accepted, outcome_rejected, Correction, Diagnostic, NoCorrectionReason,
    Outcome,
};
use crate::v2_std_node::{
    name_occurrences, Connective, Edge, EdgeLabel, Node, NodeKind, Symbol,
};

pub fn find_named_child(root: Rc<Node>, name: Symbol) -> Rc<Outcome<Rc<Node>>> {
    let count = name_occurrences(name.clone(), root.children.clone());
    if count == 0 {
        return outcome_rejected(Rc::new(Diagnostic {
            reason: "named_child_missing".to_string(),
            at: node_locus(root),
            correction: Rc::new(Correction::Unavailable {
                reason: NoCorrectionReason::ExternalContractUnknown,
            }),
        }));
    }
    if count != 1 {
        return outcome_rejected(Rc::new(Diagnostic {
            reason: "named_child_ambiguous".to_string(),
            at: node_locus(root),
            correction: Rc::new(Correction::Unavailable {
                reason: NoCorrectionReason::ExternalContractUnknown,
            }),
        }));
    }
    for edge in root.children.iter() {
        if let EdgeLabel::Named { name: edge_name } = &*edge.label {
            if edge_name == &name {
                return outcome_accepted(edge.target.clone());
            }
        }
    }
    outcome_rejected(Rc::new(Diagnostic {
        reason: "named_child_missing".to_string(),
        at: node_locus(root),
        correction: Rc::new(Correction::Unavailable {
            reason: NoCorrectionReason::ExternalContractUnknown,
        }),
    }))
}
