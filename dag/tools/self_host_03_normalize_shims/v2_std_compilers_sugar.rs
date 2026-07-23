use std::rc::Rc;

use crate::v2_std_collection::Optional;
use crate::v2_std_diagnostic::{Diagnostics, Outcome};
use crate::v2_std_node::{Connective, Node, Symbol};

#[derive(Debug, Clone, PartialEq)]
pub enum SugarKey {
    SurfaceAtomKey { atom: String },
    ProductionIdentityKey { identity: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SugarLowering {
    SugarRewriteConnective {
        connective: Rc<Connective>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SugarRule {
    pub key: Rc<SugarKey>,
    pub lowering: Rc<SugarLowering>,
}

pub fn sugar_rule_for_key(_key: Rc<SugarKey>) -> Option<Rc<SugarRule>> {
    None
}

pub fn apply_sugar(
    _rule: Rc<SugarRule>,
    _n: Rc<Node>,
    folded: Rc<Node>,
) -> Rc<Outcome<Rc<Node>>> {
    Rc::new(Outcome::Accepted {
        value: folded,
        diagnostics: Rc::new(Diagnostics::None),
    })
}
