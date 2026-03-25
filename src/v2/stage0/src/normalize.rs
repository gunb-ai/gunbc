use crate::v2_core::*;
use crate::resolve::*;
use crate::v2_rt;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NormalizeResult {
    pub graph: Rc<ModuleGraph>,
    pub diagnostics: Rc<Vec<Rc<Diagnostic>>>,
}

pub fn normalize_graph(graph: Rc<ModuleGraph>) -> Rc<NormalizeResult> {
    Rc::new(NormalizeResult { graph: graph.clone(), diagnostics: Rc::new(Vec::new()) })
}

