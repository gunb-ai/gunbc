pub use crate::std_types::NonEmptyStr;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum DeclField {
    WholeDeclaration,
    NamedField { field_name: NonEmptyStr },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DeclarationRef {
    pub module_path: NonEmptyStr,
    pub decl_name: NonEmptyStr,
    pub field: Rc<DeclField>,
}
